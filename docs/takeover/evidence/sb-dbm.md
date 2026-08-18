# Gate 1 SB-DBM live adjudication

- Branch: `codex/g1-sb-dbm-adjudication`
- Adjudication start HEAD: `c283f47d02de44a7d7f67cd66c41c737309c64d0`
- Accepted evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`
- `origin/master` at evidence freeze: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Adjudication date: `2026-08-11`
- Worktree at evidence freeze: clean; `D:\XX. SandiBumi` was the only registered Git worktree.
- Row guard: passed - exactly 43 planned `SB-DBM` rows, all 43 initially `UNADJUDICATED`, in numeric order.
- Evidence boundary: this receipt classifies the accepted implementation tree. It does not amend PRD v2, change schema or product behavior, choose a missing parameter, add a primary key/upsert to `computed_curves`, or decide pilot scope silently.
- Source-navigation boundary: the codebase index was not callable in this task, so targeted `rg`, direct source reads, executable tests and reachable Git history were used as the declared fallback. Consequential negative findings were checked in the expected Rust, TypeScript, schema, test and history paths.
- Verification boundary: a supporting test is named only for the clause it exercises. A test that does not cover every clause of its owned DBM contract is not promoted to qualifying proof. Manual checkboxes remain separate from automated evidence.
- Fresh verification: 21 focused supporting Rust tests passed. The repository gate passed 16 takeover-ledger + 13 frontend + 917 Rust tests, with 0 failed and 36 ignored; production build and generated verification matrix were green.
- Gate 2 update (2026-08-13): SB-DBM-001, SB-DBM-003, SB-DBM-004, SB-DBM-006 and SB-DBM-007 now have owned
  correctness proofs. The repository gate passes 992 / 0 / 36. Automated, visual, manual and field evidence
  remain separate.

## SB-DBM-001 - One run record per computed curve, resolvable in one hop

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T03`, `SB-DBM-T10`; sections 4.1 and 6.2.
- **Atomic obligations:** every current computed row resolves to exactly one run record; every legacy row is counted and visibly labelled `LEGACY_UNRECORDED`; displays and exports preserve that state.
- **Current source:** `src-tauri/src/equations.rs::computed_provenance_groups` classifies every live row through its actual `log_sets` join, preserves exact counts, and exposes recorded or `LEGACY_UNRECORDED` state through catalog and deliverable disclosures. `export.rs::provenance_lines` writes the same class/count into LAS `~O` plus an export summary; `inspectorPanel.ts` and `ribbon.ts` surface it. All production computed writers remain behind the complete-ancestry inventory in `core_ancestry_tests.rs`; `skip_version` still refuses.
- **Qualifying acceptance tests:** `every_computed_value_resolves_to_one_run_or_is_counted_and_labelled_legacy_unrecorded` is `CORRECTNESS`. It pins a recorded curve and a seeded legacy curve from both sides through the resolver, catalog, general disclosure, LAS file and export summary. The complete multi-run parameter-source/derivation fixture in T10 remains owned by SB-DBM-003/SB-DBM-005/SB-DBM-010.
- **Supporting tests:** `every_computed_curve_written_by_any_module_has_a_complete_ancestry_record` inventories every production writer; `every_las_export_carries_measured_computed_and_model_provenance_in_the_file` now requires the explicit legacy class without weakening the saved-model refusal.
- **Manual evidence:** `delivery-sets` 0/33, `generic-curve-store` 0/18 and `las-export` 0/2 - unexercised.
- **Git evidence:** Gate 2 topic branch contains the recorded/legacy resolver, surfaces and owned proof without changing the deliberately PK-less schema or adding an upsert.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** no numerical source or product decision is missing. Visual/manual/field review remains open and cannot be inferred from the automated proof.
- **Next action:** retain T03 and the production-writer inventory; continue with SB-DBM-002. Do not infer a `MODULE_VERSION_SOURCE` while that chapter parameter remains ABSENT.

## SB-DBM-002 - The run record pins module identity by version, not by name

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DBM-T04`, `SB-DBM-T15`; sections 4.1 and 6.2-6.3.
- **Atomic obligations:** persist a build-derived module identity that changes when the compiled module changes; include it in the re-run manifest and refuse a mismatch.
- **Current source:** `CurveAncestry` now carries `module` plus `module_version`, and every current complete-run builder fills the latter with `env!("CARGO_PKG_VERSION")`. That value is hand-maintained and shared by every built-in module, so it can remain unchanged when one module's compiled artefact changes. No build-derived identity or re-run manifest comparator exists.
- **Qualifying acceptance tests:** none; T04 and the module-version arm of T15 are missing. Test class is `MISSING`.
- **Supporting tests:** version-number tests cover log-set generations, not module binary identity.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** Gate 2 live-source re-verification supersedes the accepted-anchor absence: SB-CORE-010 added the populated field, but its package-version source violates SB-DBM-002's explicit no-hand-maintained-version clause.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED` — §5 deliberately leaves `MODULE_VERSION_SOURCE` absent. Choosing a whole-binary digest, per-module source digest, build id, algorithm, encoding or stability rule here would be an unapproved architecture decision. `DEC-021` records the exact decision boundary.
- **Next action:** decide `DEC-021`, replace every hand-maintained producer with the adopted build-derived identity, then implement T04 and the module-version mismatch arm of T15. No fake acceptance test is added while its expected identity transition is unspecified.

## SB-DBM-003 - Every petrophysical parameter in a run record carries a source string

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DBM-T05`, `SB-DBM-T09`, `SB-DBM-T30`; sections 4.1, 5.4 and 6.2/6.6.
- **Atomic obligations:** store value, source and a named absent state relationally/queryably; refuse any numeric petrophysical value without a source; never encode state as an empty string or numeric sentinel.
- **Current source:** `run_parameters` is the relational, position-preserving view of every complete ancestry parameter and `idx_run_parameters_state` supports direct unset queries. Complete single and batch run creation writes those rows in the same transaction as `log_sets`; deletion removes both atomically. `AncestryParameter` serializes an unsupplied required input only as `value: null`, `source: null`, `state: REQUIRED_UNSET`, while sourced values require a non-blank source. Schema open conservatively backfills only source-proven legacy ancestry and the exact historical `ABSENT`/`ABSENT` pair; malformed history is not guessed into compliance. Typed IPC permits the null source only alongside the named state, and the ancestry surface describes sourced values versus named absence.
- **Qualifying acceptance test:** `a_parameter_without_a_source_is_queryable_required_unset_and_never_a_number` is `CORRECTNESS`. Synthetic fixture inputs exercise one sourced value and one absent required input; the test requires the exact index, direct query result, canonical ancestry JSON, blank-source write refusal and pre-index project backfill from both sides. Its expected state comes from SB-DBM-T05/T09/T30 and F-11, not from current output.
- **Supporting tests:** all 14 equations tests pass, including saved-set versioning and ordinary text parameter values; focused complete-ancestry, LAS provenance and project round-trip controls remain green.
- **Manual evidence:** `workflow` 0/23, `processing-history` 0/7 and `verification-stewardship` 0/24 - still unexercised.
- **Git evidence:** Gate 2 topic branch carries the schema, migration, atomic writer, typed wire contract and owned proof. No primary key or upsert was added to `computed_curves`.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for this source/state contract. Uncited parameters remain absent with NULL value/source; no parameter value, physical endpoint or default was selected.
- **Next action:** retain the indexed source/state contract; visually inspect the ancestry presentation, manually query a disposable project and field-verify representative pilot runs. These open evidence classes do not reopen the automated contract.

## SB-DBM-004 - The run record stores the effective parameter set, not only the overrides

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T06`, `SB-DBM-T15`; sections 4.1 and 6.2-6.3.
- **Atomic obligations:** persist every effective value; distinguish explicit from defaulted; pin the manifest version/source of each default; keep old records invariant after a default changes.
- **Current source:** `workflow.rs::effective_module_parameters` records every configurable value from the exact `ModuleSpec` the runner used. Numeric parameters and string options are labelled `EXPLICIT` or `DEFAULTED`; absent numeric defaults remain `REQUIRED_UNSET`. Every defaulted value carries the deterministic configurable-manifest hash already owned by `parameter_pack.rs`, while explicit and zone-override values carry no default-manifest version. Ordinary runs and workflow chains call the same recorder. `AncestryParameter` and typed IPC preserve the fields, and `run_parameters` stores them queryably in the same transaction as the run record. Existing databases gain the two nullable columns additively; old rows remain unclassified rather than being relabelled without evidence.
- **Qualifying acceptance test:** `a_run_records_all_effective_parameters_and_keeps_the_default_manifest_version_after_that_manifest_changes` is `CORRECTNESS`. It independently derives the manifest hash for a synthetic five-parameter module, requires two explicit plus three defaulted records, changes one default, and proves both the new run's changed value/version and the original run's unchanged value/version. All values are declared synthetic fixture inputs, not product defaults.
- **Supporting tests:** SB-DBM-003's source/absence/migration proof, all chain controls and all parameter-pack controls remain green. The ML-specific effective-parameter test remains supporting evidence for that separate subsystem.
- **Manual evidence:** `workflow` 0/23, `machine-learning` 7/189 and `processing-history` 0/7.
- **Git evidence:** the Gate 2 topic branch contains the typed record, additive schema migration, shared ordinary-run/chain recorder and owned proof. `computed_curves` remains deliberately PK-less and no upsert path was added.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for T06. SB-DBM-002's deliberately absent build-derived module identity remains blocked and is not substituted by the parameter-manifest hash. The full T15 rerun manifest remains owned by SB-DBM-015.
- **Next action:** retain T06 and the shared recorder; visually/manual-review the effective-set presentation and proceed to SB-DBM-005 without claiming T15 or SB-DBM-002 closed.

## SB-DBM-005 - The run record carries a method-derivation citation, not only parameter values

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DBM-T07`, `SB-DBM-T10`; sections 4.1 and 6.2.
- **Atomic obligations:** require a literature citation or `FIRST-PRINCIPLES` marker at registration; persist it per run; propagate it into the deliverable.
- **Current source:** live re-verification after SB-DBM-004 confirms `ModuleSpec` still has no derivation-citation field or fail-closed registration result, `CurveAncestry` has no method-derivation field, and LAS/report/Office provenance can only propagate the ancestry fields that exist. Module comments and method-chapter prose are not a registered, complete, source-controlled metadata inventory.
- **Qualifying acceptance tests:** none; the registration refusal, run-record and export arms of T07/T10 are missing. Test class is `MISSING`.
- **Supporting tests:** module-specific comments/citations and model-citation UI rows are not durable derivation records for every run. A synthetic T07 mechanism test alone would not make the shipping registry compliant and therefore is not added as a costume for implementation.
- **Manual evidence:** `workflow` 0/23, `las-export` 0/2 and `office-deliverables` 0/39 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no universal registration/run/export field exists at the Gate 2 live source.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `BLOCKED` — neither the chapter nor the current registry supplies a complete map assigning every registered shipping module one primary citation or one approved `FIRST-PRINCIPLES` marker naming the module's own derivation document. Choosing labels from comments, adjacent chapters or engineering memory would write unsupported audit claims into client deliverables. SB-CORE-003's complete cited pilot-method inventory remains blocked on the same source-adjudication boundary.
- **Next action:** Jauhar must approve the complete registered-module derivation-source map, or adopt a named architecture/source record that supplies it. Then add the fail-closed registration field, run-record persistence, deliverable propagation and both sides of T07/T10.

## SB-DBM-006 - Inputs are recorded as resolved identities, with the rule that chose them and the candidates it rejected

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned test `SB-DBM-T08`; sections 4.1, 5.3 and 6.2.
- **Atomic obligations:** store the chosen curve id and set version, controlled decision-rule name and every rejected candidate identity; a changed choice creates a changed record.
- **Current source:** `equations.rs` now owns one staged generic-curve resolver and uses its decision for calculation inputs, plotting identity and schema-v2 ancestry. Each input records its exact stored curve identity, set/version, controlled rule and every rejected curve identity/set/version. The resolver records F-04's controlled decision stages while preserving SandiBumi's binding import-set contract: RAW is the absolute working-set tier, then exact mnemonic/alias, manual pin, Final and MRU decide within that tier; another set becomes eligible only when RAW lacks the requested mnemonic or family. It never treats a mnemonic as identity or infers Final from a set label. `db.rs` stores additive set versions, explicit Final state and modification order; Final changes are one-per-family and reversible. Workflow chains replace planning-only SELF references with exact deterministic stored set/curve identities, preserving replay determinism. Ordinary blank-set track reads retain their established standard projection contract.
- **Qualifying acceptance test:** `a_module_run_records_the_final_curve_identity_and_both_rejected_candidates_then_records_the_reflagged_choice` is `CORRECTNESS`. Three synthetic GR arrays across two sets exercise both sides of a Final reflag. The test independently derives the flip outputs and requires the exact chosen UUID/set/version, `FINAL_FLAG`, both rejected UUID/set/version identities, changed choice with the same rule, undo displacement, mnemonic-not-identity and fail-closed schema-v2 validation.
- **Supporting tests:** `a_raw_family_match_beats_an_exact_mnemonic_outside_the_working_set_until_raw_is_absent` pins RAW priority and its attached-set fallback from both sides. The native-grid/blank-set track regression, deterministic raw-import chain replay, generic promote/family controls, plotting suite and complete-ancestry inventory all pass. They preserve adjacent contracts but are not substituted for T08.
- **Manual evidence:** `generic-curve-store` 0/18, `delivery-sets` 0/33 and `workflow` 0/23 - unexercised.
- **Git evidence:** the Gate 2 topic branch contains the additive schema, central resolver, exact decision record, typed IPC/UI Final action and owned T08 proof. `computed_curves` remains PK-less and no upsert or duplicate-tolerant path was added.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for this contract. No petrophysical value, endpoint, cutoff, range or default was selected. Visual/manual/field evidence remains open.
- **Next action:** retain T08 and the shared resolver; visually inspect the Final/rejected-candidate presentation, manually run/query both reflag states and field-verify a representative duplicated curve family before any pilot claim.

## SB-DBM-007 - A missing provenance element is a named state, never an empty string

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned test `SB-DBM-T09`; sections 4.1 and 6.2.
- **Atomic obligations:** represent genuinely not-applicable and required-but-unset states distinctly; fail serialization rather than writing an empty string; make readers deterministic.
- **Current source:** `equations.rs::ProvenanceAbsentState` is the one typed `NOT_APPLICABLE` / `REQUIRED_UNSET` / `LEGACY_UNRECORDED` vocabulary. Schema-v3 ancestry requires every current empty parameter collection to carry `NOT_APPLICABLE`; parameterless equations retain their definition metadata without misclassifying it as parameters, and schema-v1/v2 empty collections are normalised by the reader to `LEGACY_UNRECORDED`. Named required inputs keep the relational `REQUIRED_UNSET` state from SB-DBM-003. `workflow.rs` serializes module parameters through a fallible boundary before batch set allocation, and any error returns a failed run with no run or curve rows.
- **Qualifying acceptance test:** `absent_is_a_named_state_never_an_empty_string` is `CORRECTNESS`. It executes a real parameterless equation and requires the persisted schema-v3 reader surface to carry `NOT_APPLICABLE` with no parameter entries; the other half injects a module-parameter serialization error through the real runner and requires an error plus zero `log_sets` and zero computed VSH rows. The named states and fail-closed behavior come directly from SB-DBM-T09/F-11, while all numeric values are synthetic reachability inputs.
- **Supporting tests:** the complete-ancestry round trip, production-writer inventory, queryable REQUIRED_UNSET proof and effective-parameter manifest proof remain green. They protect adjacent states but are not substituted for T09.
- **Manual evidence:** `processing-history` 0/7 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** the Gate 2 topic branch carries the schema-v3 typed state, legacy-reader normalisation, parameterless equation writer, fail-closed module boundary and owned T09 proof. It does not change the SQL schema, missing-sample representation or computed-curve write discipline.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. No petrophysical value, endpoint, cutoff, range or default was selected. Visual/manual/field evidence remains open.
- **Next action:** retain T09; visually inspect the ancestry presentation, manually query current and pre-v3 records, and field-verify representative pilot provenance. Do not infer those evidence classes from the automated serializer fault.

## SB-DBM-008 - The run record names the operator and the zone set in force

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T11`; sections 4.1 and 6.2.
- **Atomic obligations:** persist operator identity and zone-set identity/version on every applicable run and audit entry.
- **Current source:** `log_sets` has neither field; zones have no versioned zone-set identity; no controlled operator source is written with a run.
- **Qualifying acceptance tests:** none; the operator/zone-set arm of T11 is missing. Test class is `MISSING`.
- **Supporting tests:** zone parameter and versioned-curve tests do not persist who ran them or which zone-set revision applied.
- **Manual evidence:** `processing-history` 0/7 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no schema fields exist at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** Jauhar must decide the pilot operator-identity source and whether zone sets are in the pilot data model.
- **Next action:** after that decision, version zone sets and stamp controlled operator/zone-set identities atomically with runs and audit entries.

## SB-DBM-009 - Provenance timestamps are stored UTC and displayed local

- **Chapter evidence:** P2; chapter status `PRESENT-DIVERGENT`; owned test `SB-DBM-T11`; sections 4.1, 5.5 and 6.2.
- **Atomic obligations:** store UTC; render local only at the display edge; migrate or explicitly classify legacy local timestamps without guessing.
- **Current source:** schema-v3 curve ancestry stores Unix-epoch milliseconds from `SystemTime`, but Inspector renders that instant with `toISOString()` rather than in the viewer's local zone. Process history stores `Date.now()` milliseconds and the History panel renders locally, while its text export emits zone-less UTC text. `log_sets.created_at` still uses DuckDB `TIMESTAMP DEFAULT now()` without a UTC contract or offset. No policy classifies its existing local/unspecified values.
- **Qualifying acceptance tests:** none; T11's cross-zone UTC storage/local-display fixture is missing. Test class is `MISSING`.
- **Supporting tests:** timestamp presence/order tests do not prove UTC semantics.
- **Manual evidence:** `processing-history` 0/7 and `project-lifecycle` 3/24.
- **Git evidence:** live re-verification on `codex/g2-program-plan` finds a UTC instant in current ancestry and process-history records, but local-display and legacy `log_sets` semantics remain divergent.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-022` must classify legacy local/unspecified values before any migration can avoid inventing their zone. The full T11 audit fixture also depends on SB-DBM-011's structured audit store.
- **Next action:** after Jauhar settles DEC-022, introduce one unambiguous UTC representation, label legacy values without guessing, convert only at display, reuse that timestamp contract in SB-DBM-011's audit store and implement the time-zone arm of T11.

## SB-DBM-010 - Provenance travels into the deliverable

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-DBM-T10`; sections 4.1 and 6.2.
- **Atomic obligations:** every exported computed curve resolves to run, effective parameter sources and derivation citation; legacy curves are labelled/countable; formats that cannot carry provenance disclose the omission at export.
- **Current source:** `export.rs::provenance_lines` writes machine-readable JSON into LAS `~O` for measured, versioned computed and saved-model curves; schema-v3 ancestry now carries parameter source strings, and SB-DBM-001 keeps legacy computed curves exported under `LEGACY_UNRECORDED` with an exact summary count. PDF, workbook, DOCX and deck paths use the shared `curve_ancestry_disclosures` rows, but those are human-readable tables rather than one registered machine-readable sidecar contract. `AncestryOutput.derivation` contains implementation descriptions, not the source-controlled method citations SB-DBM-005 requires.
- **Qualifying acceptance tests:** none; the complete 20-well, multi-run, equation and legacy fixture from T10 is missing. Test class is `MISSING`.
- **Supporting tests:** `export.rs::every_las_export_carries_measured_computed_and_model_provenance_in_the_file` proves the narrower LAS JSON and saved-model refusal; SB-DBM-001's owned test proves legacy labels/counts through LAS and number-carrying disclosure surfaces. Neither can manufacture the missing method citations or prove one format-wide sidecar contract.
- **Manual evidence:** `las-export` 0/2, `office-deliverables` 0/39 and `processing-history` 0/7 - unexercised.
- **Git evidence:** live re-verification on `codex/g2-program-plan` finds the LAS machine-readable record and shared report/Office ancestry rows integrated; complete cited provenance across deliverable formats is not integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-DBM-003's parameter sources now exist, but SB-DBM-005 remains blocked on the complete registered-module derivation-source map. Existing free-form derivation descriptions cannot be relabelled as citations.
- **Next action:** after SB-DBM-005 supplies source-controlled citations, extend the export/report/Office registries with one provenance-capability and machine-readable-sidecar contract, then implement all T10 arms across the selected pilot deliverables.

## SB-DBM-011 - Structured audit entries, as name-value pairs with a controlled vocabulary

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned tests `SB-DBM-T11`, `SB-DBM-T12`; sections 4.1, 5.5 and 6.2.
- **Atomic obligations:** store relational audit entry/detail rows using controlled location/mode vocabulary, typed values/units, coalescing and UTC/operator/zone-set fields.
- **Current source:** `src/processLog.ts` persists `{ts, kind, detail, well}` as one capped JSON document and more than 70 UI call sites emit free-text details. It survives Save Project As because it lives in the project database, but there are no relational `audit_entry`/`audit_detail` tables, controlled location/mode types, typed value/unit rows or explicit gesture boundary for uninterrupted coalescing.
- **Qualifying acceptance tests:** none; T11 and the structured-diff setup of T12 are missing. Test class is `MISSING`.
- **Supporting tests:** process-log rendering and Save Project As coverage prove visible history is retained in the copied project, not that entries are relational, controlled, coalesced or complete.
- **Manual evidence:** `processing-history` 0/7, `security-integrity` 0/63 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** live re-verification on `codex/g2-program-plan` confirms the divergent JSON history and its broad call-site inventory; no structured audit store is integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** DEC-022 must settle UTC legacy classification. DEC-023 must reconcile exact T11's required zone-set identity with DEC-018 excluding SB-DBM-008. DEC-020 already settles the operator source. No time-based coalescing window is needed or allowed: the cited rule is explicit gesture uninterruptedness.
- **Next action:** after DEC-022 and DEC-023, add controlled relational entry/detail types and one backend-owned atomic writer, retain visible legacy history without pretending it is structured, migrate selected action surfaces using explicit gesture boundaries, and implement T11 without importing SB-DBM-012's deferred diff contract.

## SB-DBM-012 - A parameter-state diff is a database join, not an external differ

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T12`; sections 4.1 and 6.2.
- **Atomic obligations:** return structured zone/parameter old/new/unit rows through a database join; spawn no external process; embed the result in reports.
- **Current source:** there is no relational audit-detail store or native parameter-state diff query. Current free-text/JSON history would require parsing outside the specified join.
- **Qualifying acceptance tests:** none; T12, including the process-table assertion, is missing. Test class is `MISSING`.
- **Supporting tests:** read-only SQL joins are generic database-tool behavior and cannot manufacture absent audit rows.
- **Manual evidence:** `database-tools` 0/2, `processing-history` 0/7 and `office-deliverables` 0/39 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no audit join or report surface exists at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the structured audit schema in SB-DBM-011.
- **Next action:** after SB-DBM-011, add a native join-backed diff and report block, then implement T12 with a no-child-process control.

## SB-DBM-013 - No configuration, deployment mode or preference may disable the provenance record

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned test `SB-DBM-T13`; sections 4.1 and 6.2.
- **Atomic obligations:** run-record failure rolls back curve writes and reports failure; every computed writer is atomic with provenance; no setting/input/environment switch bypasses it.
- **Current source:** every production computed writer requires an opaque live complete-ancestry set. Legacy writer helpers are test-only, `PaySummaryRequest.skip_version` refuses rather than writing, and the shared whole-corpus scan rejects production calls to legacy/raw computed writers. A second scan enumerates Rust environment, DuckDB, project-document and installed-settings reads plus TypeScript local/session preference reads, so the no-bypass proof is not limited to a hand-picked setting list.
- **Qualifying acceptance tests:** `workflow::tests::provenance_cannot_be_switched_off_and_a_failed_record_fails_the_write` is `CORRECTNESS`. Its expected transaction/refusal contract is SB-DBM-T13, sourced there to F-03. It proves a normal paired record/curve write, forces the second `log_sets` insert in one batch to fail after the first insert, requires rollback of both records and every output, requires both serialized job items to be `Failed`, executes the `skip_version` refusal, enumerates configuration reads and reuses the independent writer inventory.
- **Supporting tests:** `core_ancestry_tests::every_computed_curve_written_by_any_module_has_a_complete_ancestry_record` independently retains the shared whole-production writer inventory; `pay_summary_versions_flags_with_cutoffs_in_provenance` retains the ordinary versioned path and explicit refusal.
- **Manual evidence:** `workflow` 0/23, `delivery-sets` 0/33 and `security-integrity` 0/63 remain unexercised; synthetic fault injection is automated evidence only.
- **Git evidence:** current topic branch; one SB-DBM-013 commit will carry T13, inventory reuse and tracker correction after the full gate passes.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` handled by Gate 2; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. No petrophysical value, deployment default or scientific assumption is required.
- **Next action:** retain T13 and the shared writer/configuration inventories; Jauhar performs visual/manual review without promoting the synthetic fault to field evidence.

## SB-DBM-014 - Every stochastic operation records its seed and its seeding rule

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T14`, `SB-DBM-T15`; sections 4.2 and 6.3.
- **Atomic obligations:** store root seed, exact seed/index derivation rule and generator identity for every stochastic operation; replay bit-identically from the record alone in another process/machine.
- **Current source:** Monte Carlo and ML persist a root seed in parameter JSON and use deterministic code paths, but neither stores a general generator identity plus derivation rule. Monte Carlo's custom RNG and index derivation remain implementation details rather than durable run data.
- **Qualifying acceptance tests:** none; cross-process record-alone T14 and the stochastic manifest arm of T15 are missing. Test class is `MISSING`.
- **Supporting tests:** Monte Carlo/ML same-seed tests prove in-process repeatability for selected paths, not record sufficiency or cross-machine generator identity.
- **Manual evidence:** `monte-carlo` 2/14, `machine-learning` 7/189 and `workflow` 0/23.
- **Git evidence:** accepted anchor `b332026c` contains seed persistence and deterministic fragments; the full recorded triple is not integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the durable generator-identity and seeding-rule vocabulary must be specified; no identifier is invented here.
- **Next action:** register and persist the exact stochastic implementation identity/rule beside each seed and implement cross-process bitwise T14.

## SB-DBM-015 - The re-run manifest is enumerated, stored, and checkable

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DBM-T15`, `SB-DBM-T16`; sections 4.2 and 6.3.
- **Atomic obligations:** one manifest enumerates module, effective parameters/sources, resolved inputs/frames, zone set, seeds, models and physics-driving attributes; a rerun command resolves every element or refuses it by name.
- **Current source:** those elements exist only as incomplete fragments across `log_sets`, ancestry JSON and model metadata. No complete manifest schema, resolver or "re-run this set" command exists. The approved pilot manifest excludes SB-DBM-008, SB-DBM-014 and the model-custody rows rather than supplying their identities implicitly.
- **Qualifying acceptance tests:** none; all mutated-project arms of T15 and manifest-driven T16 are missing. Test class is `MISSING`.
- **Supporting tests:** log-set restore feeding a later run and model drift warnings do not enumerate or enforce the complete manifest.
- **Manual evidence:** `workflow` 0/23, `processing-history` 0/7 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** live re-verification on `codex/g2-program-plan` confirms `UNIMPLEMENTED`; no complete manifest/re-run feature exists, and SB-DBM-014 is not among the immutable pilot IDs.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** T15 depends on DEC-021/SB-DBM-002 for build-derived module identity; DEC-023/SB-DBM-008 for versioned zone-set identity; DEC-024 for the conditional stochastic/model identity seam owned by SB-DBM-014 and SB-DBM-019/020; and SB-DBM-017 for physics-driving attributes. Those identity owners are outside DEC-018's immutable first-pilot scope, so they cannot be imported silently and none may be omitted from the exact test.
- **Next action:** settle DEC-021, DEC-023 and DEC-024, explicitly re-approving any required scope change. Only then add one stored resolver-backed manifest, an unmutated byte-identical replay and all four element-naming refusals.

## SB-DBM-016 - Re-run output does not depend on iteration order

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DBM-T16`; sections 4.2 and 6.3.
- **Atomic obligations:** output curves and aggregates are byte-identical across processes with different hash seeds and unordered database row traversal.
- **Current source:** `core_determinism_tests.rs` launches the real Rust test executable against copies of one imported two-well project. Each child executes the approved VSH-density/neutron-saturation chain and pay summary, emits every computed curve and aggregate field in a binary artifact and exposes a 64-key live HashMap-order witness. Query-side output packing is explicitly ordered.
- **Qualifying acceptance tests:** `a_project_run_in_fresh_processes_with_different_hash_orders_produces_identical_curve_bytes_and_aggregate_statistics` is a `CORRECTNESS` proof of T16. It requires two fresh-process witnesses to differ while every packed curve and aggregate byte agrees, then changes recorded Rw in a third process and requires both products to differ.
- **Supporting tests:** `a_recorded_raw_import_to_pay_summary_rerun_produces_byte_identical_curve_blobs_and_an_identical_pay_summary` remains the same-process SB-CORE-011 proof; T16 adds the process boundary and observed order difference.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** the Gate 2 topic branch contains the fresh-process binary comparison and an opposite-side sensitivity control; no production behavior or scientific value changed.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the exact approved deterministic pilot chain. This does not close SB-DBM-015's absent stored re-run manifest or extend proof to the 689 deferred requirements.
- **Next action:** retain T16 in the default gate; Jauhar may field-verify a representative sanitized delivery in Gate 4 without relabelling the automated fixture as field evidence.

## SB-DBM-017 - A metadata attribute that drives physics is an input of the module that consumes it

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T17`; sections 4.2 and 6.3.
- **Atomic obligations:** declare every physics-driving metadata attribute as a module input; record runtime value; mark prior output stale on change; refuse a named unset attribute instead of defaulting.
- **Current source:** no registry attribute declares metadata-to-physics dependencies, no run record stores them generically, and no stale-output invalidation follows such changes. `nphimat` consumes explicit `TOOL`, `SALINITY` and `MATRIX_IN` options; those are not persisted curve attributes and missing options resolve through module defaults rather than T17's named refusal.
- **Qualifying acceptance tests:** none; T17's changed and unset controls are missing. Test class is `MISSING`.
- **Supporting tests:** bespoke module arguments and warnings do not prove the universal metadata dependency contract.
- **Manual evidence:** `workflow` 0/23, `data-conventions` 0/45 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; the registry/run/staleness mechanism is absent. DEC-003 already fixes the pilot chain, and DEC-018 includes SB-DBM-017 and SB-POR-024 but excludes the owning SB-ENV-012 metadata contract.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** DEC-025. The chapter-cited first-pilot candidate is neutron matrix scale: SB-POR-024 requires the basis and provenance, while SB-ENV-012 owns the typed curve attribute, persistence and every-consumer validation. SB-ENV-012 is outside DEC-018's immutable manifest. No source authorizes inventing a Logging Contractor field or default tool, salinity or matrix selection.
- **Next action:** authorize SB-ENV-012's typed neutron-scale seam as required infrastructure or revise and re-approve DEC-018 to include it; then implement T17's changed and unset controls without a default.

## SB-DBM-018 - Training-set identity is recorded as ids and intervals, not as names

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T18`, `SB-DBM-T20`; sections 4.3 and 6.4.
- **Atomic obligations:** store stable well ids, a depth interval per well and the exact training-curve set identity/version; survive rename; report changed versions; refuse an unresolvable deleted well.
- **Current source:** `ml_models.training_json` records well ids and names, row counts, masks, incompleteness and one input set id/name/version. It does not record each well's training interval or each training curve's version. Drift warnings cover moved/missing sets, but the exact rename/delete/unresolvable contract is incomplete.
- **Qualifying acceptance tests:** none; the five-well rename, rerun, delete and interval arms of T18 are missing. Test class is `MISSING`.
- **Supporting tests:** `a_model_records_the_log_set_its_rows_came_from_and_names_it_when_it_has_moved` and `a_model_carries_the_log_set_it_was_fitted_on_and_never_guesses_between_two` prove selected set identity, not intervals or all lifecycle arms.
- **Manual evidence:** `machine-learning` 7/189, `delivery-sets` 0/33 and `generic-curve-store` 0/18.
- **Git evidence:** accepted anchor `b332026c` contains the stable-id/set-version fragment; the complete training identity is not integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no numeric source is missing; per-well interval and per-training-curve version identity are absent.
- **Next action:** persist those exact identities and implement every rename/rerun/delete/interval arm of T18.

## SB-DBM-019 - A stored model carries its seed, its full library set and an artifact hash

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T19`, `SB-DBM-T21`; sections 4.3 and 6.4.
- **Atomic obligations:** persist seed/generator identity, every numerically relevant library version and a model-artifact hash; verify hash on load and refuse corruption by name.
- **Current source:** ML effective parameters include the seed; `runtime_json` probes Python and the relevant numerical libraries; `train_hash` fingerprints the training matrix, not the artifact blob. LAS export computes SHA-256 transiently, but `ml_models` does not store or verify an artifact hash on load.
- **Qualifying acceptance tests:** none; T19's pinned/unpinned train-store-reload and corrupt-blob arms are missing. Test class is `MISSING`.
- **Supporting tests:** runtime metadata and training-fingerprint tests prove different objects; export-time hashing does not protect model load.
- **Manual evidence:** `machine-learning` 7/189 and `security-integrity` 0/63.
- **Git evidence:** accepted anchor `b332026c` contains seed/runtime/training-hash fragments; artifact custody is not integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `ARTIFACT_HASH_ALGORITHM` is deliberately absent; this adjudication does not choose one.
- **Next action:** adjudicate the algorithm, store hash/generator identity with the model, verify before deserialization and implement both T19 corruption controls.

## SB-DBM-020 - Both apply paths stamp the model identity into the produced curve's provenance

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned test `SB-DBM-T20`; sections 4.3 and 6.4.
- **Atomic obligations:** train-and-apply always persists the fitted model; train-and-apply and apply-saved both stamp a resolvable `model_id` in the curve's run record.
- **Current source:** apply-saved resolves and stamps a stored model. Train-and-apply persists/stamps only when `save_model_as` is supplied; otherwise produced curves legitimately carry no model id under current behavior.
- **Qualifying acceptance tests:** none; T20's same-estimator two-path resolvability test is missing. Test class is `MISSING`.
- **Supporting tests:** `a_curve_from_a_fitting_run_names_the_model_and_a_run_that_kept_none_names_none` characterizes the current divergent optional-save behavior; it does not prove T20.
- **Manual evidence:** `machine-learning` 7/189, `delivery-sets` 0/33 and `processing-history` 0/7.
- **Git evidence:** accepted anchor `b332026c` contains both apply paths and the optional-persistence counter-path.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** Jauhar must confirm that every train-and-apply fit becomes a durable native model, as the chapter specifies.
- **Next action:** persist train-and-apply artifacts unconditionally under a collision-safe native identity, stamp both paths and implement T20.

## SB-DBM-021 - Model artifacts are native-only; a foreign artifact is refused at the store boundary

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-DBM-T21`; sections 4.3 and 6.4.
- **Atomic obligations:** store an immutable native origin; reject foreign blobs through every command/dialog/SQL path; reject non-native rows again at apply; user input cannot set origin.
- **Current source:** the SQL console is read-only and no vendor-model file reader is registered, but `ml_models` has no origin column and `insert_ml_model` accepts bytes without a native-origin invariant. Apply cannot reject a non-native origin that the schema does not record.
- **Qualifying acceptance tests:** none; T21's every-path and apply-boundary fixture is missing. Test class is `MISSING`.
- **Supporting tests:** `no_code_path_reads_a_vendor_model_or_weight_file` is a source inventory and the read-only SQL tests close only two entry surfaces, not the store boundary.
- **Manual evidence:** `machine-learning` 7/189 and `security-integrity` 0/63.
- **Git evidence:** `UNIMPLEMENTED`; the central origin/store/apply contract is absent at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** none; `CONTRACT.md` already supplies the native-only policy.
- **Next action:** make origin store-generated and immutable, enforce it at insert and apply, then implement every T21 entry path including direct SQL refusal.

## SB-DBM-022 - The feature-vector order contract is verified at apply time, not assumed

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DBM-T22` (`CHARACTERIZATION`); sections 4.3 and 6.4.
- **Atomic obligations:** the model artifact owns one ordered feature list; apply verifies exact membership and order; reordered-complete and missing-feature wells fail by name.
- **Current source:** `MlApplyRequest` carries no caller-controlled feature list; the host fetches in the stored model order, and the Python artifact compares the supplied ordered feature list with its own before prediction. Per-well missing inputs are named and fail the run.
- **Qualifying acceptance tests:** `ml.rs::a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order` directly exercises the artifact refusal and is `#[ignore]`d because its subject needs Python, NumPy, scikit-learn and joblib; missing-well behavioral fixtures share that optional runtime. Test class is `OPTIONAL-PACKAGE-IGNORED`.
- **Supporting tests:** `an_apply_request_cannot_state_a_feature_order_for_the_model_to_refuse` is a nonignored structural guard against a second order source, but it cannot alone prove the runner's named behavioral refusal.
- **Manual evidence:** `machine-learning` 7/189 and `generic-curve-store` 0/18.
- **Git evidence:** accepted anchor `b332026c` contains the stored-order runner check and named missing-input path.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied safety contract pending field evidence); `SILENT-WRONGNESS`; test class `OPTIONAL-PACKAGE-IGNORED`; commit state `INTEGRATED`.
- **Blocker or decision:** automated default-gate closure cannot depend on optional ML packages; representative installed-runtime exercise remains a pilot field item.
- **Next action:** preserve the structural default-gate test and run the ignored behavioral T22 fixture in the qualified offline Python pack before pilot release.

## SB-DBM-023 - Schema-level vocabularies live in one registry, and every consumer resolves through it

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned test `SB-DBM-T23`; sections 4.4 and 6.5.
- **Atomic obligations:** one registry owns every schema vocabulary and every projection derives from it; a second literal declaration fails; adding one registry item reaches every consumer.
- **Current source:** `schema_vocab.rs` owns one typed registry for standard columns plus the exact sampling-style, log-set-frame, depth-datum, audit-location, audit-mode and provenance-absence populations. Schema DDL/migration, standard-frame reads and inserts, curve editing, plotting resolution, inspector columns, output-shadow refusal and frame reads/writes consume exported entries or derived projections; the former `equations.rs` and `curve_edit.rs` duplicate declarations are gone.
- **Qualifying acceptance tests:** `vocabularies_have_one_source_and_every_projection_derives_from_it` passed and is `CORRECTNESS`. It adds a synthetic eighth schema member and independently requires its select, editable, inspector, DDL and migration projections, checks the six exact vocabulary populations, scans for exactly one declaration owner and rejects a second full literal standard-column declaration.
- **Supporting tests:** existing standard-curve read, edit, inspector and Reframe regressions remain green, but only T23 owns the source-of-truth mutation contract.
- **Manual evidence:** `data-conventions` 0/45 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** current topic-branch increment centralizes the registry and removes the divergent declarations without adding a scientific value, computed-curve key or upsert path.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` satisfied; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. `DEPTH` remains non-editable by a derived projection rather than disappearing from a copied list.
- **Next action:** preserve the source-tree guard; Gate 4 may inspect representative imported and own-frame sets, but manual inspection is not automated proof of registry completeness.

## SB-DBM-024 - Every capacity limit is unit-typed, carries a source string, and is the source of its own documentation

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T24`; sections 4.4 and 6.5.
- **Atomic obligations:** inventory every capacity/tolerance/limit; attach a unit type and source; generate published documentation from the declarations; forbid bare duplicated values.
- **Current source:** bare limits remain across Rust and TypeScript, including finished-job retention and SQL-console row caps. No typed limit registry, source field or generated limits table exists.
- **Qualifying acceptance tests:** none; T24's source-tree and generated-doc controls are missing. Test class is `MISSING`.
- **Supporting tests:** clamp/cap tests characterize isolated values and do not source or generate them.
- **Manual evidence:** `portfolio-performance` 0/50 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; the registry/generator mechanism is absent at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** every carried limit needs its own source or explicit SandiBumi characterization; this adjudication supplies none.
- **Next action:** inventory the limits, classify source versus characterization, introduce unit types and generate the table before publishing any new capacity claim.

## SB-DBM-025 - A constant that crosses a module boundary is registered with its source

- **Chapter evidence:** P2; chapter status `ABSENT`; owned tests `SB-DBM-T23`, `SB-DBM-T24`; sections 4.4 and 6.5.
- **Atomic obligations:** every cross-module petrophysical constant resolves through one source-carrying registry; modules may not duplicate or privately default it.
- **Current source:** there is no central source-bearing petrophysical constant registry or build-time duplicate guard. The selected pilot inventory found a decisive counterexample: `modules::PHIE_FLOOR = 0.001` crosses density, analytic D-N and downstream pay paths. `CLAUDE.md` requires that value, while immutable `11_porosity.md` SB-POR-045 and §5 require the floor to ship `ABSENT` because one held source attests both `0.001` and `0.0001`.
- **Qualifying acceptance tests:** none; T23 proves schema vocabulary ownership, not physical-constant value/source custody, and T24's whole limits registry remains absent. A test that registered only the non-conflicting subset would weaken this contract. Test class is `MISSING`.
- **Supporting tests:** module numerical tests verify individual formulae and the current floor behavior, not cross-module identity/source custody or the required source-precedence decision.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** live re-verification on `codex/g2-program-plan` confirms `UNIMPLEMENTED`; no qualifying registry exists and the contradictory floor contracts remain live.
- **Verdict:** `ABSENT`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** DEC-003 now identifies the pilot methods, but DEC-026 must decide whether the binding 0.001 product rule or SB-POR-045's no-default contract governs. Registering either cited candidate without that decision would manufacture authority; making the value absent now would violate the current binding rule.
- **Next action:** settle DEC-026, then inventory every selected cross-module value, register only the explicitly authorized cited/default state with unit and source, route every consumer through it and add a two-sided duplicate-declaration/default-absence proof.

## SB-DBM-026 - Two samples may not share a depth in one curve, and the resolution is declared

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DBM-T25`, `SB-DBM-T26`; sections 4.5 and 6.6.
- **Atomic obligations:** continuous-set duplicate depths refuse with both source rows named; POINT sets can retain duplicates only under a declared/logged resolution; storage distinguishes set types and resolution.
- **Current source:** `schema_vocab.rs` owns typed continuous/POINT and REFUSE/PRESERVE/PERTURB vocabularies. `log_sets` records a conservative `CONTINUOUS_IRREGULAR`/`REFUSE` declaration for current producers; every single, clearing, batched, OWN-frame and Restore boundary validates declared continuous uniqueness before mutation. `aux_sets` is the shipped sparse point-delivery registry; its real import writer declares `POINT`/`PRESERVE`, preserves legitimate same-depth rows and logs each duplicate source row. An explicit PERTURB route requires a positive unit-typed offset and records original/stored depth, resolution, magnitude and unit per duplicate row. Historical aux rows are labelled from their actual PK-less preserve-all writer behavior; historical log-set style remains unrecorded rather than guessed.
- **Qualifying acceptance tests:** exact correctness test `continuous_duplicates_name_both_source_rows_while_point_duplicates_require_and_record_their_resolution` implements SB-DBM-T25 from both sides: regular and irregular duplicate continuous writes and a corrupted-archive Restore refuse with curve, depth and both rows before current mutation; the production aux writer accepts and logs duplicate POINT observations; PERTURB refuses with no offset and logs both rows under the cited 0.01 ft fixture; `computed_curves` remains PK-less. Test class is `CORRECTNESS`. SB-DBM-T26 is not claimed here; it remains the next SB-DBM-027 integrity-checker increment.
- **Supporting tests:** ingest duplicate-policy tests, `batch_write_overwrites_without_duplicating`, log-set Restore tests, auxiliary import tests and the pre-column point-set migration regression remain green.
- **Manual evidence:** `data-conventions` 0/45, `generic-curve-store` 0/18 and `delivery-sets` 0/33 - unexercised.
- **Git evidence:** live Gate 2 implementation on `codex/g2-program-plan`; commit receipt follows the mandatory full gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for SB-DBM-026. `DUPLICATE_DEPTH_PERTURBATION` remains ABSENT by design; 0.01 ft appears only as the cited T25 fixture input. The deliberately PK-less `computed_curves` table and delete-then-append discipline remain unchanged, with no uniqueness index or upsert path.
- **Next action:** implement SB-DBM-027/SB-DBM-T26 as the separate read-only integrity checker, then retain Visual, Manual and Field review as distinct Gate 4 evidence.

## SB-DBM-027 - A referential-integrity checker exists, reports every dangling class by name and count, and never reports "clean" without checking

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T26`; sections 4.5 and 6.6.
- **Atomic obligations:** enumerate every reference class; report named counts including zero; offer bounded prune/repair; never emit a bare clean result without the class inventory.
- **Current source:** `db.rs::check_referential_integrity` returns all seven live classes on every run: current and archive log-set references, missing-well group membership, orphan curve samples, unresolved `ml_models.trained_on`, and current/archive duplicate-depth keys. It counts legacy current `set_id IS NULL` rows but deliberately excludes them from pruning. Typed quarantine tables preserve exact numeric rows in-project; `prune_referential_integrity`, restore and reapply are one-transaction backend-whitelisted actions. `lib.rs`, `ipc.ts` and `dbInspectorPanel.ts` expose the read-only check, explicit selected-class quarantine, persisted recovery and Ctrl+Z/Ctrl+Y without client SQL or sample arrays over IPC.
- **Qualifying acceptance tests:** exact correctness test `the_integrity_checker_names_every_class_including_zero_counts_offers_a_reversible_prune_and_never_says_clean_without_checking` seeds the cited dangling archive row and missing-well group member, requires the empty curve-sample class by name, asserts every remaining class at zero, proves the check does not mutate the three stores, and exercises quarantine, restore and reapply. Test class is `CORRECTNESS`.
- **Supporting tests:** none credited; the one owned T26 proof carries the whole contract.
- **Manual evidence:** `security-integrity` 0/63 and `database-tools` 0/2 - unexercised.
- **Git evidence:** live Gate 2 implementation on `codex/g2-program-plan`; commit receipt follows the mandatory full gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` handled by Gate 2; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated closure. ML provenance and duplicate-depth classes remain report-only because automatic deletion or survivor selection would invent identity policy. Legacy `NULL set_id` rows remain labelled findings rather than data to erase.
- **Next action:** Jauhar visually and manually exercises check, selected quarantine, restart recovery, Ctrl+Z and Ctrl+Y in Gate 4; a green synthetic fixture is not field acceptance.

## SB-DBM-028 - A declared sampling style is verified against the reference column on ingest, and the verdict is stored

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-DBM-T27`; sections 4.5, 5.2 and 6.6.
- **Atomic obligations:** persist declared style; verify it against the actual reference samples; store effective verdict and warning; prevent frame-indexed misplacement after a contradicted regular declaration.
- **Current source:** `ingest.rs` requires a declared continuous style for every new LAS set. A regular declaration additionally requires a unit-typed tolerance with no default, verifies the delivery's declared STEP against the sanitized native reference samples and atomically writes declared/effective style, verdict, input tolerance, warning, gap depth and missing-row count to `import_sets`. `equations.rs` refuses an explicit imported-set frame read without that verdict and uses verified native reference samples rather than a synthesized row index.
- **Qualifying acceptance tests:** exact SB-DBM-T27 `a_forty_row_gap_contradicts_a_regular_sampling_declaration_while_a_verified_regular_set_stays_regular_and_an_unverified_set_cannot_be_frame_read`; test class `CORRECTNESS`. It pins the cited 0.1524 m / 40-row / 6.1 m fixture, stored contradiction, native post-gap depth, genuinely regular control and unverified-read refusal. The 0.0001 m fixture tolerance is supplied only by the test and is not a product default.
- **Supporting tests:** all 45 default ingest tests pass; existing non-increasing/duplicate and reframe tests remain separate structural controls.
- **Manual evidence:** `data-conventions` 0/45, `delivery-sets` 0/33 and `reframe` 0/34 - unexercised.
- **Git evidence:** live Gate 2 implementation on `codex/g2-program-plan`; commit receipt follows the mandatory full gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` handled by Gate 2; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated closure. `SAMPLING_STYLE_VERIFY_TOLERANCE` remains deliberately absent as a product default; every regular import must supply its own explicit unit-typed value.
- **Next action:** Jauhar visually and manually checks declaration/tolerance refusal and the named contradiction in Gate 4, then confirms a representative post-gap sample retains its delivered depth. Automated evidence is not field evidence.

## SB-DBM-029 - A module never writes to the reference column of a frame it reads

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DBM-T28`; sections 4.5 and 6.6.
- **Atomic obligations:** refuse any module output targeting the input frame's reference column at the API boundary; name the frame; leave every other curve unmoved; a new basis creates an `OWN` frame.
- **Current source:** `workflow.rs::resolve_output_names` is the one output-name boundary shared by every deterministic module and now gives `DEPTH` a specific refusal naming the existing `STANDARD` frame and the explicit Reframe recovery path. `reframe.rs::run_reframe` delegates its complete write to `equations.rs::write_complete_own_frame`, which marks the new set `OWN` and writes it to the archive only.
- **Qualifying acceptance tests:** exact SB-DBM-T28 `a_module_cannot_write_an_existing_reference_column_and_a_different_depth_basis_is_a_new_own_frame`; test class `CORRECTNESS`. It drives both real APIs, byte-snapshots every standard column plus a computed peer across refusal and OWN-frame creation, and verifies the distinct archived depth basis.
- **Supporting tests:** `an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs` separately preserves generic GR/RHOB shadow and same-run collision refusal; the older Reframe round trip separately pins set-qualified readback.
- **Manual evidence:** `curve-editing` 5/5, `reframe` 0/34 and `data-conventions` 0/45.
- **Git evidence:** live Gate 2 implementation on `codex/g2-program-plan`; commit receipt follows the mandatory full gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` handled by Gate 2; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated closure. No depth step or other petrophysical value was defaulted; the OWN control supplies its synthetic step explicitly.
- **Next action:** Jauhar visually and manually checks the named refusal and explicit Reframe path in Gate 4, then compares source and reframed depth inventories on the representative delivery. Automated evidence is not field evidence.

## SB-DBM-030 - Null discipline: a threshold, not an equality; and "no value" is not "no parameter"

- **Specified contract:** an undeclared large-negative vendor null is screened by a strict threshold (never an equality against one sentinel), a screened value is FLAGGED, never silently coerced; and measurement-absent versus parameter-not-supplied are distinguishable at store, IPC, UI and export as SQL NULL and absence-of-row, neither representable as a number.
- **Current implementation (2026-08-18):** the screen and its flag channel SHIPPED as one piece. `db::is_large_negative_null` screens strictly below a COMPUTED bound one decade inside Geolog `cgg.h` `MISS_FLOAT = -1.0e30`; a value exactly ON the bound is DATA (bit-for-bit pinned). The generic-store writer and `insert_standard_curves` both screen and both bind SQL NULL for NaN - one delivery lands in both projections, and one screened / one kept would be two truths about one sample. Every screened count travels to the importer by DELIVERED mnemonic: LAS new-well and attach warnings, DLIS's existing `DlisSkip` channel, intake notes. The standard-to-generic migration now copies a data-bearing column's FULL frame (a NULL sample row is "logged but missing", never "never sampled"). Two NULL-intolerant `curve_samples` readers fixed. SB-DBM-003/007 cover the parameter-not-supplied half (REQUIRED_UNSET row + tagged ancestry at IPC/UI/export).
- **Qualifying tests (in code; register entry follows on promotion):** `db::inspector_tests::an_undeclared_large_negative_null_is_screened_to_sql_null_and_counted_and_a_value_on_the_bound_stays_data` and `ingest::tests::a_screened_import_names_the_curve_and_count_in_its_own_warning_never_silently` (full production LAS import; warning names mnemonic and count; both projections agree).
- **Mutation evidence:** six probes, six DISTINCT assertions fired - equality screen (misses -1.0D38), bound coerced (`<` to `<=`), NaN kept as a float, importer warning silenced, standard projection unscreened, migration dropping missing rows.
- **Manual evidence:** none claimed. Automated only.
- **Completed same day (2026-08-18):** the computed stores joined the discipline - all six `computed_curves`/archive appender loops bind SQL NULL for NaN (write_versioned_rows_raw, clearing and archive-only variants, the batched multi-well writer, reframe's archive writer); the ~135 reader sites were audited (most were counts/copies/already-tolerant; seven value readers fixed to `Option<f32>`), and `equations::tests::a_computed_curves_missing_sample_is_sql_null_at_the_store_and_nan_at_the_reader` pins store, archive and reader with two further mutations killed (each store's loop reverted to float NaN).
- **Verdict:** `PRESENT-OK`; test class `CORRECTNESS`; Gate 2 DONE 2026-08-18 @ codex/g2-program-plan (pre-PR).

## SB-DBM-031 - Every depth quantity declares its datum, and cross-datum comparison is refused

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T31`; sections 4.5 and 6.6.
- **Atomic obligations:** every depth-bearing value declares datum/reference and sign convention; cross-datum comparison refuses and names both datums unless a reference frame/survey resolves them.
- **Current source:** `schema_vocab::DepthDatum` owns the exact seven-value vocabulary. Zones and contacts persist it; new zone writers declare MD or another explicit datum and an untyped legacy zone remains NULL/refused. `well_path` intrinsically names MD/TVD/TVDSS, deviation and materialized system TVDSS are positive down, and format 2 converts the explicitly declared legacy TVDSS stores once after an engine backup. `compare_zone_top_to_contact` refuses unlike datums without a covering well frame. Correlation no longer treats an absent TVDSS map as an identity/vertical-well transform.
- **Qualifying acceptance tests:** `contacts::tests::an_md_zone_top_and_a_tvdss_contact_are_refused_without_a_frame_and_compare_with_positive_down_tvdss_with_one` is exact `CORRECTNESS`: it pins both named-datum refusal and framed success, derives 1000 MD/TVD minus 100 positive-up elevation as 900 positive-down TVDSS, proves the old sign is converted once in path/contact/materialized-curve stores, and proves an untyped legacy zone is neither called MD nor readable.
- **Supporting tests:** the full Rust library suite remains green across survey materialization, format stamping, project migration and every existing zone consumer; TypeScript compiles with the nullable correlation conversions. The finished repository gate is 1000 passed / 0 failed / 36 ignored with the unchanged 55 owned Rust warnings.
- **Manual evidence:** `data-conventions` 4/68, `correlation-tops` 0/39 and `core-depth-registration` 0/39. The new SB-DBM-031 scenarios are all pending; the four data-convention checks predate this increment.
- **Git evidence:** live Gate 2 worktree; one SB-DBM-031 commit will carry the typed path, exact test, sign migration and refusal evidence together.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; Gate 2 outcome `BLOCKED`.
- **Blocker or decision:** legacy/generic imports never stored their datum or source TVDSS sign. The remaining generic depth stores include standard/high-resolution/LQR/array/current/archive curves, tops, highlights, core/aux/image/SCAL data, import evidence and quarantine rows. Their values cannot be classified from a unit, mnemonic, numeric sign or neighbouring table. The deferred saturation-height surface also still describes the old negative-TVDSS convention and is not changed inside this database-model increment.
- **Next action:** require a source/operator datum declaration at every remaining import/frame boundary, add typed custody to those stores and their IPC consumers, migrate only rows whose source declaration is present, and separately reconcile the deferred saturation-height wording/test before that capability is re-enabled. Do not infer or bulk-label legacy rows.

## SB-DBM-032 - A stored parameter carries a dual handle, and a disagreement is a load failure

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T32`; sections 4.5, 5.4 and 6.6.
- **Atomic obligations:** persist permanent sparse ordinal plus semantic key, unit, source and tilt; mismatch hard-fails naming both rows; single-handle legacy loads with warning; tilt evaluates within-zone and steps at boundaries.
- **Current source:** `parameter_pack.rs` carries the whole contract now. The dual handle and the hard mismatch naming both rows were already in, and `DEC-028` ruled the one-handle conflict: BOTH one-handle forms are refused, matching the closed `SB-INS-015`/`SB-INS-T18` installer contract rather than the chapter's "loads with a warning". This increment adds the three per-value fields §5.4 requires and the append-only ordinal custody. `ParameterPackRow` gains `unit`, `source` and a typed `ParameterTilt` (`NONE | LINEAR | LOG`); `value_at_depth` evaluates a tilt within its own zone.
- **`tilt` is parsed from its own token, deliberately:** an unrecognised tilt is refused BY NAME rather than deserialized into a default. Falling through to `NONE` is the one failure mode here that returns a plausible number instead of an error, and a plausible `Rw` is not a failure anybody notices.
- **Refusals added, none removed:** a numeric or tilted value with no `source` is refused at load (§5.4 - a silently defaulted value is not a legal state); a tilted value must carry a two-endpoint range, because a scalar carrying a tilt token has already lost the physics the token claims; and a `LOG` tilt needs two positive endpoints, since refusing beats handing back a NaN a caller has to notice.
- **Within-zone only, and it returns absence rather than a clamp.** `value_at_depth` gives `None` outside `[top, base]` in both directions. Clamping is the tempting mistake and it is the wrong one: a parameter that clamps at the zone base silently spreads one zone's calibration into the next, which is exactly the physics F-19 says a scalar loses. The parameter STEPS, and the neighbouring zone supplies its own endpoints.
- **Qualifying acceptance tests:** `parameter_pack::tests::a_stored_parameter_carries_its_unit_and_source_and_a_tilted_value_never_interpolates_across_a_zone_boundary`, beside the already-registered `a_parameter_row_carrying_only_one_of_its_two_handles_is_refused_by_name`. Test class `CORRECTNESS`.
- **The witness is the chapter's own, and it names the WRONG answer.** F-19 (`22_database-model.md:433-441`) states that `Rw` tilted logarithmically between 0.28 and 0.19 across a zone **is not 0.235**. 0.235 is the LINEAR midpoint of those same endpoints, so a tilt stored as a display mode - or a `LOG` tilt quietly evaluated linearly - lands exactly on the number the chapter names as wrong, off by 0.0043 ohm.m on `Rw`, which propagates into Sw. The test pins both sides: the log answer must be the geometric mean, the linear answer must not be produced, and the LINEAR twin on the same endpoints must give exactly 0.235 - otherwise the arm proves only that some arithmetic happened.
- **Append-only ordinals** are pinned by asking for the DECLARED ordinal rather than the position: a sparse pack `1/2/5/9` survives the load uncompacted, `by_ordinal(9)` finds the row that declared 9, and a retired slot resolves to nothing rather than to a neighbour. Renumbering is precisely how ledger R-10's ClayVol #41 bound one parameter's value to another.
- **Mutation record:** five mutations, five distinct assertions, none surviving - evaluating `LOG` linearly (:558), clamping outside the zone instead of refusing (:596), an unknown token falling through to `NONE` (:661), dropping the mandatory-source rule (:611), and dropping the two-endpoint requirement (:648). Each was applied and restored from a sha256-verified byte copy; `git diff --stat` showed 300 insertions and 0 deletions.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised. Automated only; no manual or field evidence is claimed.
- **Git evidence:** additive throughout. The new raw fields are `#[serde(default)]`, so every existing pack still parses, and all pre-existing fixtures use object values and are untouched by the numeric-source rule. Full suite 1052 passed / 0 failed / 37 ignored.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none outstanding. `DEC-028` ruled the conflict and needed nothing further; `SB-INS-015`/`T18` is untouched and is the contract this row was corrected TO, not away from.
- **Next action:** wire `value_at_depth` into the zone-parameter read path when an authorization covers it. That binding lives in `zone_params` in `db.rs`, and `DEC-061`'s narrow authorization is scoped to `SB-DBM-030` only, so it is deliberately not taken here.

## SB-DBM-033 - A categorical curve is a distinct type and is never linearly interpolated

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T33`; sections 4.5 and 6.6.
- **Atomic obligations:** categorical type is explicit; resampling produces only existing codes and reports boundary crossings; arithmetic refuses categorical inputs; all relevant writers/readers preserve the type.
- **Current source:** `curve_class` explicitly declares producer-known class curves. Reframe refuses an unreadable registry, coerces unsafe methods to nearest/mode, returns a structured record for every target sample bracketed by unlike source codes, and renders those records in the Reframe report. Frame blocking uses mode and refuses averages; smoothing/despiking refuse class curves. Both Rhai and Python equation runners preflight the declaration and refuse categorical inputs before evaluation, interpreter discovery or writes.
- **Qualifying acceptance tests:** exact correctness test `a_categorical_curve_resamples_only_to_existing_codes_reports_every_boundary_crossing_and_is_refused_by_every_equation_language` implements T33's cited 0.1524-to-0.1 m fixture, asserts only source codes 1/4, both crossed target samples and their source bracket, retained declaration, both equation-language refusals, zero arithmetic writes and fail-closed unreadable metadata. Test class is `CORRECTNESS`.
- **Supporting tests:** `a_declared_class_curve_is_never_averaged_and_an_undeclared_one_keeps_the_method_asked_for`, `a_class_curve_is_carried_by_its_commonest_value_rather_than_averaged`, `a_class_curve_is_blocked_by_its_commonest_code_and_refuses_every_average`, and `a_class_curve_is_refused_by_smooth_and_despike_and_an_undeclared_one_is_not` prove important fragments from both sides.
- **Manual evidence:** `reframe` 0/34, `workflow` 0/23 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** the accepted anchor `b332026c` supplied producer declarations and selected safe consumers; the current Gate 2 branch adds the complete T33 reporting and equation refusal boundary.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the selected producer-declared categorical workflow. This increment does not claim an arbitrary imported curve can be manually retyped; no heuristic may silently create that declaration.
- **Next action:** retain T33; Jauhar visually and manually verifies the rendered Reframe report and equation refusal, then Gate 4 exercises one sanitized producer-declared class curve.

## SB-DBM-034 - Every bulk operation returns `{matched, unmatched, ambiguous}` and drops nothing silently

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T34`; sections 4.5 and 6.6.
- **Atomic obligations:** every bulk operation returns the three counts; all unmatched/ambiguous rows enter an addressable review queue; nothing is fuzzy-matched or dropped silently.
- **Current source:** individual import/paste paths expose different result shapes; there is no universal three-count response type or shared review queue covering every bulk operation.
- **Qualifying acceptance tests:** none; T34's exact 95/3/2 counts and five addressable exceptions are missing. Test class is `MISSING`.
- **Supporting tests:** DIO import reports can count selected failure classes but do not cover the universal bulk contract or review queue.
- **Manual evidence:** `correlation-tops` 0/36, `security-integrity` 0/63 and `workflow` 0/23 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; the shared result/queue contract is absent at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-003` must identify pilot bulk operations before the finite first inventory can be enforced.
- **Next action:** inventory the selected bulk commands, return one typed three-way outcome plus review queue and implement T34 with exact counts.

## SB-DBM-035 - The archive is append-only, and restoring a prior version is a first-class operation

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DBM-T35`; sections 4.5 and 6.6.
- **Atomic obligations:** archive UPDATE/DELETE refuse; restore creates the next version with a source-version link; all earlier versions remain unchanged.
- **Current source:** `equations.rs::restore_log_set` validates the archived source, then in one transaction appends a new log-set row, copies its queryable run parameters, replaces only the current projection, and appends a new archive copy under the next version and a fresh `set_id`. `_sandibumi_restore_v1` records the immediate source set/version. `delete_log_set` is now an explicit append-only refusal retained behind the Tauri command for stale clients. `ipc.ts` returns the typed source/new-version receipt; `inspectorPanel.ts` names it and exposes no ordinary Delete action.
- **Qualifying acceptance tests:** exact correctness test `archive_updates_and_deletes_are_refused_and_restoring_version_one_creates_version_four_without_changing_versions_one_through_three` starts with archived v1/v2 and current v3, refuses SQL-console UPDATE/DELETE plus the stale ordinary-delete command, restores v1 as v4, verifies the source record and new current identity, and compares every v1-v3 archive row before/after. Test class is `CORRECTNESS`.
- **Supporting tests:** `db.rs::log_set_versioning_never_overwrites` now proves restore appends the next catalog version and deletion refuses; `workflow.rs::a_restored_log_set_version_feeds_the_next_module_run` proves the restored values remain consumable downstream. Neither is credited as T35's owned proof.
- **Manual evidence:** `delivery-sets` 0/33, `project-lifecycle` 3/24 and `security-integrity` 0/63.
- **Git evidence:** live Gate 2 implementation on `codex/g2-program-plan`; commit receipt follows the mandatory full gate.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` handled by Gate 2; `RECOVERY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for ordinary history operations. The backed-up format migration and the typed, logged, reversible integrity quarantine are separately bounded maintenance paths; neither authorizes ordinary version deletion.
- **Next action:** retain exact T35; Jauhar manually restores v1 while v3 is current and visually verifies the v4/source disclosure; Gate 4 repeats on a sanitized project without treating synthetic automation as field evidence.

## SB-DBM-036 - No operation whose duration scales with well count holds the global lock

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DBM-T36`, `SB-DBM-T38`; sections 4.6 and 6.7.
- **Atomic obligations:** retain one writer but release the global database mutex before well-count-scaled computation; interactive-command worst-case latency must not scale with N; publish the distribution.
- **Current source:** selected ML paths snapshot data and release the mutex before per-well computation, and batched writers bound transaction work. Many synchronous commands and long workflows still hold `DbState`'s global mutex across work whose duration grows with project/well size. No cross-N latency distribution exists.
- **Qualifying acceptance tests:** none; T36 and the relevant real-well T38 measurements are missing. Test class is `MISSING`.
- **Supporting tests:** lock-count/source inventories and isolated concurrency tests identify exposure but do not prove latency shape.
- **Manual evidence:** `portfolio-performance` 0/50, `workflow` 0/23 and `well-scope` 3/9.
- **Git evidence:** accepted anchor `b332026c` contains both improved snapshot paths and remaining long-held-lock paths.
- **Verdict:** `PRESENT-DIVERGENT`; `UNDECIDED`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** representative pilot scale and workflows must be selected before the bounded lock refactor/measurement set is known; single-writer discipline remains non-negotiable.
- **Next action:** inventory the selected pilot commands, move pure computation outside the lock without adding writers, then publish T36 latency distributions across N.

## SB-DBM-037 - Well scoping is enforced in the backend, not in the client

- **Chapter evidence:** P1; chapter status `PARTIAL`; owned test `SB-DBM-T37`; sections 4.6 and 6.7.
- **Atomic obligations:** every well-iterating backend command enforces the active group or explicitly declares project-wide scope; direct invocation cannot bypass it; evidence uses query/row counts.
- **Current source:** `well_scope.rs` owns a 44-entry command registry: 43 operations are `BACKEND_SCOPED`, while the deliberately exhaustive referential-integrity command is `PROJECT_WIDE`. Registered scoped commands accept a typed scope identity and resolve current membership inside their Tauri boundary; an unknown operation fails closed. `db.rs::list_wells_by_ids`, the scoped contact loader, `tops.rs`, `statistics.rs` and job-name loaders constrain the authorized ids in SQL rather than resolving 12 and then materializing 540. TypeScript defaults ordinary well inventory to `ActiveGroup`; project-administration surfaces request `All` explicitly. The integrity response carries `scope: PROJECT_WIDE` and `wells_touched` to the Database Inspector.
- **Qualifying acceptance test:** `every_well_iterating_backend_command_scopes_the_sql_to_the_active_twelve_of_five_hundred_and_forty_or_declares_project_wide` passed; test class is `CORRECTNESS`. It builds the exact 540-well project and active group of 12 from T37, directly invokes every registered backend authorization boundary, proves the current 12 identities are returned, inventories the corresponding Tauri wrapper, pins the downstream `WHERE well_id IN (...)` loaders and proves the exhaustive integrity path declares `PROJECT_WIDE` with 540 wells touched. This is shared-boundary plus source-inventory evidence; it does not pretend to execute every expensive scientific job end to end.
- **Supporting tests:** `every_backend_scoped_operation_uses_current_group_membership_and_refuses_stale_or_unknown_scope` preserves the distinct Group, ActiveGroup, All and Explicit alternatives, proves stale membership disappears, and refuses missing or repeated identities. TypeScript compilation pins callers to the typed scope API.
- **Manual evidence:** `well-scope` 3/9, `workflow` 0/23 and `security-integrity` 0/63.
- **Git evidence:** implementation is on `codex/g2-program-plan` pending the per-requirement commit and PR; the accepted anchor remains the earlier client-enforced boundary.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER` (satisfied engineering contract); `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED` after the requirement commit.
- **Blocker or decision:** none for Gate 2 engineering. The synthetic 540/12 fixture is automated evidence, not manual or representative-field qualification.
- **Next action:** retain exact T37 and SB-CORE-035; Jauhar visually and manually exercises Active Group, named Group, All and Explicit modes plus a membership change, then confirms the integrity command visibly declares project-wide scope. Gate 4 repeats the scope contract on sanitized representative data.

## SB-DBM-038 - The interactive set is the only thing materialised

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T38` (`CHARACTERIZATION`); sections 4.6, 5.2 and 6.7.
- **Atomic obligations:** project open/list/facet/first-paint materialize only the interactive set; publish real-well scaling at N = 100, 500, 1000, 2000 and 5000; derive the ceiling from evidence.
- **Current source:** backend well listing/project queries can materialize the whole project and no interactive-set materialization boundary or published scale curve exists.
- **Qualifying acceptance tests:** none; T38 must use real wells and cannot be replaced by synthetic timing, source inspection or a small fixture. Test class is `MISSING`.
- **Supporting tests:** historic 100-well workflow timing and UI subsets do not cover the four specified operations or N range.
- **Manual evidence:** `portfolio-performance` 0/50 and `well-scope` 3/9.
- **Git evidence:** `UNIMPLEMENTED`; no materialization boundary or accepted scale evidence exists at the anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `FIELD-EVIDENCE`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `INTERACTIVE_SET_CEILING` deliberately ships absent; real representative wells and the full N curve are required to settle it.
- **Next action:** prepare reversible real-project scale fixtures, instrument the four operations, run T38 at every N and let the measured curve set the ceiling.

## SB-DBM-039 - A job result distinguishes clean, degraded and failed, and the store records which

- **Chapter evidence:** P0; chapter status `PARTIAL`; owned tests `SB-DBM-T39`, `SB-DBM-T41`; sections 4.6 and 6.7.
- **Atomic obligations:** clean, warned/degraded and failed are distinct per well and in aggregate; clamp/substitution degradations cannot appear clean; degradation remains queryable after transient job pruning.
- **Current source:** `workflow.rs::ModuleRunResult` and `jobs.rs::JobView` expose typed clean/degraded/failed outcomes. The module boundary captures actual `CLAMPED`, `DEFAULTED`, `TRUNCATED` and `SUBSTITUTED_INPUT` events; the complete writer commits `log_sets.outcome_state` and ordered `run_degradations` in the same transaction as curve rows. `run_readonly_query` now returns `returned_rows` plus `count_is_total = false`, while inspector `total_rows` retains its exact meaning.
- **Qualifying acceptance tests:** `a_clamped_well_and_a_substituted_input_well_are_warned_and_leave_durable_degradation_records_after_their_job_is_pruned_while_a_clean_well_stays_clean` exercises the three per-well outcomes, aggregate warning and 25-job prune. `the_inspector_reports_the_true_ten_thousand_row_total_while_the_hundred_row_console_page_names_its_count_as_returned_not_total` pins the 10,000/100 reporting boundary. Both are `CORRECTNESS` from SB-DBM-T39/T41.
- **Supporting tests:** existing job-state, SQL-truncation and module-workflow tests remain active behind the exact contracts.
- **Manual evidence:** `workflow` 0/23, `processing-history` 0/7 and `machine-learning` 7/189.
- **Git evidence:** `codex/g2-program-plan` pre-PR implements the owned engineering contract; the accepted baseline remains unchanged until review and merge.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; the chapter supplies the closed four-value degradation vocabulary.
- **Next action:** retain exact T39/T41; Jauhar visually and manually checks warning/history/inspector rendering, and Gate 4 repeats durable-record recovery on sanitized representative output.

## SB-DBM-040 - Cancellation honesty is regression-locked

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned test `SB-DBM-T40` (`CHARACTERIZATION`); sections 4.6 and 6.7.
- **Atomic obligations:** an observing worker finalizes cancelled; a non-observing worker reports its actual outcome; non-cancellable jobs expose no cancel control; one regression test pins all three.
- **Current source:** `jobs.rs` separately records request and worker observation; `run_job` uses observation for final status; `JobView.cancellable` reaches `processingPanel.ts`, which creates a cancel button only for active cancellable jobs.
- **Qualifying acceptance tests:** `every_displayed_cancel_reaches_an_observing_worker_and_completed_work_is_never_reported_cancelled` drives the observing, non-observing and non-cancellable jobs, asserts their final views, inventories every cancellable registration/observer and pins the live panel's control condition. It is `CHARACTERIZATION` for SB-DBM-T40 exactly as the chapter requires.
- **Supporting tests:** `cancellable_flag_reaches_the_view_both_ways`, `cancel_counts_as_cancelled_only_once_a_worker_observes_it` and `note_cancel_observed_marks_it_for_raw_flag_readers` continue to pin the individual state-model halves.
- **Manual evidence:** `workflow` 0/23 and `machine-learning` 7/189.
- **Git evidence:** the exact integrated regression was added during SB-CORE-036 closure and is reverified on `codex/g2-program-plan`; the accepted baseline remains unchanged until review and merge.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** none for automated characterization. Source-level UI inventory is not a rendered click-through, so Visual and Manual evidence remain open.
- **Next action:** retain exact T40 and the cancellation-registration inventory; Jauhar visually and manually verifies the live Processing controls.

## SB-DBM-041 - A count presented as a total is a total; the inspector exposes the provenance tables

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DBM-T41`, `SB-DBM-T42`; sections 4.6 and 6.7.
- **Atomic obligations:** `total_rows` has one meaning across inspector and SQL console or the capped count uses a different field; inspector whitelist includes all provenance/model/audit tables and can trace a curve without leaving it.
- **Current source:** T41 is integrated: paginated `get_table_page` computes the true count, while `run_readonly_query` exposes `returned_rows` plus `count_is_total = false`. `TABLE_SPECS` still omits `log_sets`, `run_parameters`, `run_degradations`, `computed_curves_archive`, `curve_meta`, `ml_models` and the absent `audit_entry`/`audit_detail` tables required by T42.
- **Qualifying acceptance tests:** exact T42 cannot be written because its required SB-DBM-011 audit tables do not exist, so the requirement remains test class `MISSING`.
- **Supporting tests:** `the_inspector_reports_the_true_ten_thousand_row_total_while_the_hundred_row_console_page_names_its_count_as_returned_not_total` proves T41's count-meaning half. `every_inspector_table_returns_the_columns_it_declares` checks only the incomplete whitelist and cannot prove the required trace.
- **Manual evidence:** `database-tools` 0/2, `processing-history` 0/7 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** `5527c8c` closes T41 on `codex/g2-program-plan`; live schema and whitelist inspection confirms T42 remains unavailable. The accepted baseline remains unchanged until review and merge.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SB-DBM-011 is blocked on DEC-022's legacy timestamp classification and DEC-023's zone-set scope. T42 explicitly requires its structured `audit_entry`/`audit_detail` tables; placeholder tables or a reduced whitelist would weaken the exact contract.
- **Next action:** after DEC-022/023 unblock SB-DBM-011, implement its controlled audit tables first; then derive the inspector inventory from the complete provenance/audit registry and write exact T42 without weakening T41.

## SB-DBM-042 - The format-version gate and the pre-migration backup are contractual, and the backup names the format it can restore

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; owned tests `SB-DBM-T01`, `SB-DBM-T02`, `SB-DBM-T43`; sections 4.6 and 6.1.
- **Atomic obligations:** a newer format refuses before mutation and leaves bytes identical; destructive migration first creates a fail-closed, non-overwriting, user-visible backup; additive migration creates none; filename names the source format restored at each step.
- **Current source:** `check_and_stamp_format` returns the observed source version before schema work; `init_db` preserves it in a connection-local temporary row across the later target stamp. `backup_before_destructive_migration` derives the recovery name from that source identity, copies with DuckDB, selects a fresh collision suffix and returns before any destructive statement on copy failure. Additive/no-op migration still skips backup.
- **Qualifying acceptance tests:** `consecutive_destructive_upgrades_name_each_backup_for_the_source_format_it_restores` is `CORRECTNESS`, independently deriving the `pre-0`/`pre-1` shelf and contents from F-07. The exact T01/T02 characterizations are also executable and jointly pin byte-stable refusal, writer/current/source reporting, backup-before-write, non-overwrite, user-visible path, deterministic copy failure and additive exemption.
- **Supporting tests:** `fresh_project_open_writes_no_backup` remains a narrower additive-path regression; it is not counted as a second owned proof.
- **Manual evidence:** `project-lifecycle` 3/24, `security-integrity` 0/63 and `verification-stewardship` 0/24.
- **Git evidence:** current `codex/g2-program-plan` increment carries the source-labelled backup contract and exact tests; accepted baseline remains unchanged until review and merge.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `RECOVERY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none; no petrophysical or uncited parameter is involved.
- **Next action:** retain exact T01/T02/T43 and run Jauhar's manual legacy-project recovery check separately from automated evidence.

## SB-DBM-043 - A deterministic parameter sweep records every trial, uncapped and ordered

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T44`; sections 4.6 and 6.7.
- **Atomic obligations:** enumerate a declared grid deterministically without a depth cap; persist every trial's vector/result/sweep reference under full provenance; record any subsampling seed; reruns are bit-identical.
- **Current source:** selected workflows can calculate series or tune method-specific values, but there is no generic sweep/trial store, full per-trial run identity, uncapped 3,000-level contract or manifest-driven replay.
- **Qualifying acceptance tests:** none; T44's 3 x 4 x 5, 60-trial, 3,000-level two-run fixture is missing. Test class is `MISSING`.
- **Supporting tests:** method-specific deterministic loops do not create the specified generic durable sweep.
- **Manual evidence:** `workflow` 0/23, `processing-history` 0/7 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no generic sweep/trial schema or command exists at the accepted anchor.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the provenance and rerun-manifest contracts in SB-DBM-001 through 006 and 015; no method parameter is selected here.
- **Next action:** after those foundations, design one generic ordered trial store and implement T44 exactly without inheriting a vendor depth cap.

## Domain result

- All `30 / 30` approved Gate 2 SB-DBM pilot blockers are handled: `18 DONE`, `12 BLOCKED`; the other 13 domain requirements remain explicitly deferred beyond the first pilot.
- Current as-built: 19 `PRESENT-OK`, 5 `PRESENT-DIVERGENT`, 8 `PARTIAL`, 11 `ABSENT`; no row is unadjudicated or silently omitted.
- Current test evidence: 18 `CORRECTNESS`, 1 `CHARACTERIZATION`, 1 `OPTIONAL-PACKAGE-IGNORED` and 23 `MISSING`. A missing proof on a deferred or blocked row remains visible rather than being converted into completion.
- Remaining pilot blocks are exactly SB-DBM-002, 005, 009, 010, 011, 015, 017, 025, 030, 031, 032 and 041, with their source or product decisions named in the corresponding evidence rows and `STATUS.md`.
- Automated evidence is not manual recovery, inspector, operator or representative-field evidence; those remain for Jauhar and Gates 3-4. No database guard was weakened, and `computed_curves` remains PK-less with no upsert path.
