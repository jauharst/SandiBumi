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
- **Current source:** many module and stochastic tests are deterministic for a fixed fixture, and several queries use explicit ordering. There is no complete query/collection inventory and no two-process different-hash-seed project comparison.
- **Qualifying acceptance tests:** none; T16's process-level byte and aggregate comparison is missing. Test class is `MISSING`.
- **Supporting tests:** `workflow.rs::test_full_deterministic_chain` and fixed-seed module tests are same-process, bounded-path evidence.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains deterministic fragments; universal order independence is unverified.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the complete rerun manifest from SB-DBM-015 is required to drive the specified comparison.
- **Next action:** inventory unordered reads/collections and implement T16 as two fresh processes with different hash seeds over the same manifest.

## SB-DBM-017 - A metadata attribute that drives physics is an input of the module that consumes it

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T17`; sections 4.2 and 6.3.
- **Atomic obligations:** declare every physics-driving metadata attribute as a module input; record runtime value; mark prior output stale on change; refuse a named unset attribute instead of defaulting.
- **Current source:** no registry attribute declares metadata-to-physics dependencies, no run record stores them generically, and no stale-output invalidation follows such changes.
- **Qualifying acceptance tests:** none; T17's changed and unset controls are missing. Test class is `MISSING`.
- **Supporting tests:** bespoke module arguments and warnings do not prove the universal metadata dependency contract.
- **Manual evidence:** `workflow` 0/23, `data-conventions` 0/45 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; the registry/run/staleness mechanism is absent at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-003` must identify pilot methods before their physics-driving attributes can be source-audited; uncited values remain absent.
- **Next action:** for the selected pilot methods only, declare source-cited attribute inputs, add stale tracking and implement both arms of T17.

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
- **Current source:** `equations.rs` and `curve_edit.rs` still declare independent standard-column lists with different membership (`DEPTH` is included in only one). Frame, sampling, datum, audit and absent-state vocabularies are also not generated from one registry.
- **Qualifying acceptance tests:** none; T23's source-tree failure/one-item propagation/second-literal controls are missing. Test class is `MISSING`.
- **Supporting tests:** current output-shadow tests compare behavior, not derivation from one source.
- **Manual evidence:** `data-conventions` 0/45 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains the duplicate declarations and therefore the live divergence.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** none; centralization must preserve deliberate membership differences as derived projections, not copy another list.
- **Next action:** build one typed vocabulary registry with derived projections and implement every T23 mutation control.

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
- **Current source:** there is no central source-bearing petrophysical constant registry or build-time duplicate guard. Module-local arguments/constants remain independently declared.
- **Qualifying acceptance tests:** none; the registry and duplicate-declaration arms of T23/T24 are missing. Test class is `MISSING`.
- **Supporting tests:** module numerical tests verify individual formulae, not cross-module identity/source custody.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no qualifying registry exists at the accepted anchor.
- **Verdict:** `ABSENT`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-003` must identify pilot methods; every registry value still needs a named primary/cited source.
- **Next action:** inventory only selected pilot cross-module constants, leave uncited values absent, then add one source-bearing registry and duplicate-declaration gate.

## SB-DBM-026 - Two samples may not share a depth in one curve, and the resolution is declared

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DBM-T25`, `SB-DBM-T26`; sections 4.5 and 6.6.
- **Atomic obligations:** continuous-set duplicate depths refuse with both source rows named; POINT sets can retain duplicates only under a declared/logged resolution; storage distinguishes set types and resolution.
- **Current source:** DIO ingest paths detect duplicates and require a selected policy, while the generic/standard stores have primary-key constraints. Deliberately PK-less `computed_curves` has no set-type/sampling-style field or store-boundary duplicate refusal, so direct writes can append duplicate `(well, curve, depth)` rows.
- **Qualifying acceptance tests:** none; T25's continuous-versus-POINT two-sided store fixture and T26 integrity arm are missing. Test class is `MISSING`.
- **Supporting tests:** ingest duplicate-policy tests and `batch_write_overwrites_without_duplicating` cover cooperative writers, not adversarial store-boundary duplicates or POINT legality.
- **Manual evidence:** `data-conventions` 0/45, `generic-curve-store` 0/18 and `delivery-sets` 0/33 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains DIO policy fragments and the PK-less central counter-boundary.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** a set-type/resolution vocabulary is missing. The fix must preserve PK-less `computed_curves` and its delete-then-append performance discipline.
- **Next action:** validate and record duplicate policy before the existing transaction writes, then implement T25 from continuous-refusal and POINT-acceptance sides without a PK/upsert.

## SB-DBM-027 - A referential-integrity checker exists, reports every dangling class by name and count, and never reports "clean" without checking

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T26`; sections 4.5 and 6.6.
- **Atomic obligations:** enumerate every reference class; report named counts including zero; offer bounded prune/repair; never emit a bare clean result without the class inventory.
- **Current source:** no central integrity-check command/report inventories dangling archive set ids, group members, curve samples or other references. Foreign keys are not a substitute for the required all-class report.
- **Qualifying acceptance tests:** none; T26's two dangling classes plus zero-count class are missing. Test class is `MISSING`.
- **Supporting tests:** individual delete/lookup tests cover local behavior only.
- **Manual evidence:** `security-integrity` 0/63 and `database-tools` 0/2 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no checker surface exists at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the reference-class inventory must be generated from the live schema/writer model rather than hand-waved.
- **Next action:** build a read-only named-count checker first, add separately authorized prune actions, and implement the full three-class T26 fixture.

## SB-DBM-028 - A declared sampling style is verified against the reference column on ingest, and the verdict is stored

- **Chapter evidence:** P0; chapter status `ABSENT`; owned test `SB-DBM-T27`; sections 4.5, 5.2 and 6.6.
- **Atomic obligations:** persist declared style; verify it against the actual reference samples; store effective verdict and warning; prevent frame-indexed misplacement after a contradicted regular declaration.
- **Current source:** log/curve set schemas do not store declared/effective sampling style or a verification verdict, and ingest has no shared contradiction check before frame-indexed use.
- **Qualifying acceptance tests:** none; T27 is deliberately unwritten because its verification tolerance is an input and no production default is cited. Test class is `MISSING`.
- **Supporting tests:** non-increasing/duplicate and reframe tests cover different structural conditions.
- **Manual evidence:** `data-conventions` 0/45, `delivery-sets` 0/33 and `reframe` 0/34 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; no sampling-style schema/guard exists at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `SAMPLING_STYLE_VERIFY_TOLERANCE` is deliberately absent; a cited tolerance or explicit input contract is required.
- **Next action:** obtain/adjudicate the source or require the tolerance explicitly, then store declared/effective style and implement T27's 40-row-gap control.

## SB-DBM-029 - A module never writes to the reference column of a frame it reads

- **Chapter evidence:** P1; chapter status `PRESENT-UNVERIFIED`; owned test `SB-DBM-T28`; sections 4.5 and 6.6.
- **Atomic obligations:** refuse any module output targeting the input frame's reference column at the API boundary; name the frame; leave every other curve unmoved; a new basis creates an `OWN` frame.
- **Current source:** `workflow.rs::resolve_output_names` refuses names shadowing `STANDARD_COLUMNS`, whose registry includes `DEPTH`; Reframe can create `OWN` output. The error/test does not target `DEPTH`, name a frame or prove all curves remain unchanged, and no full writer inventory establishes universality.
- **Qualifying acceptance tests:** none; T28's DEPTH-targeted API/no-movement/OWN controls are missing. Test class is `MISSING`.
- **Supporting tests:** `an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs` proves GR/RHOB shadow and collision refusal, not the exact reference-column contract.
- **Manual evidence:** `curve-editing` 5/5, `reframe` 0/34 and `data-conventions` 0/45.
- **Git evidence:** accepted anchor `b332026c` contains the preventive mechanism; the compound contract remains unverified.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no parameter is missing; an exact acceptance test and complete reference-writer inventory are missing.
- **Next action:** implement T28 with `DEPTH`, a named frame, unchanged peer curves and a separate `OWN`-frame success control.

## SB-DBM-030 - Null discipline: a threshold, not an equality; and "no value" is not "no parameter"

- **Chapter evidence:** P0; chapter status `ABSENT`; owned tests `SB-DBM-T29`, `SB-DBM-T30`; sections 4.5, 5.1 and 6.6.
- **Atomic obligations:** store-side vendor-null detection uses the strict computed threshold and exact boundary; missing curve samples and absent parameters remain distinct through store, byte IPC, UI and export.
- **Current source:** DIO readers have source-specific sentinel logic, but no shared store-side `v < MISS_FLOAT / 10` screen exists. Numeric arrays correctly use `f32::NAN` and bytemuck bytes, yet no structured parameter-absence state exists through all four layers.
- **Qualifying acceptance tests:** none; T29's exact-boundary/computed-bound and T30's four-layer distinction are missing. Test class is `MISSING`.
- **Supporting tests:** parser null tests do not prove the store-side vendor family, and parameter-pack structural tests do not supply absent-parameter semantics.
- **Manual evidence:** `data-conventions` 0/45, `generic-curve-store` 0/18 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** `UNIMPLEMENTED`; the store threshold and cross-layer parameter-absence contract are absent at the accepted anchor.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** specification friction is explicit: chapter T30's prose must be implemented without violating the binding `f32::NAN` plus byte-IPC array contract; `Option<f32>`/JSON arrays remain forbidden.
- **Next action:** add the cited strict store screen and a tagged parameter-state channel beside unchanged sample arrays, then implement both exact-boundary and four-layer tests.

## SB-DBM-031 - Every depth quantity declares its datum, and cross-datum comparison is refused

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T31`; sections 4.5 and 6.6.
- **Atomic obligations:** every depth-bearing value declares datum/reference and sign convention; cross-datum comparison refuses and names both datums unless a reference frame/survey resolves them.
- **Current source:** the project carries depth units, `well_surveys` has a datum elevation, and selected contact/image paths carry reference flags. Tops parsing can still collapse MD/TVD aliases into an untyped depth, and zones, curves and other quantities lack a universal datum field and comparison guard.
- **Qualifying acceptance tests:** none; T31's no-frame refusal, framed success and sign assertions are missing. Test class is `MISSING`.
- **Supporting tests:** measured-depth contact refusal and survey-specific tests cover isolated consumers only.
- **Manual evidence:** `data-conventions` 0/45, `correlation-tops` 0/36 and `core-depth-registration` 0/39 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains partial survey/reference structures and untyped counter-paths.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the datum vocabulary and migration for existing untyped depths require explicit adjudication; no datum is inferred from a unit or mnemonic.
- **Next action:** carry a typed datum through every depth schema/IPC boundary, require a declared transform for cross-datum use and implement both T31 cases.

## SB-DBM-032 - A stored parameter carries a dual handle, and a disagreement is a load failure

- **Chapter evidence:** P1; chapter status `ABSENT`; owned test `SB-DBM-T32`; sections 4.5, 5.4 and 6.6.
- **Atomic obligations:** persist permanent sparse ordinal plus semantic key, unit, source and tilt; mismatch hard-fails naming both rows; single-handle legacy loads with warning; tilt evaluates within-zone and steps at boundaries.
- **Current source:** `parameter_pack.rs` stores semantic id plus ordinal and hard-fails disagreement naming both schema rows. It instead refuses a missing ordinal, and its row format has no required unit/tilt/source evaluation contract or append-only ordinal-evolution proof.
- **Qualifying acceptance tests:** none; the full mismatch, single-handle warning and tilt/unit round-trip/evaluation fixture in T32 is missing. Test class is `MISSING`.
- **Supporting tests:** `an_identifier_ordinal_disagreement_stops_loading_and_names_both_schema_rows` correctly pins the mismatch arm; `missing_ordinals_duplicate_keys_unsupported_schemas_and_empty_keys_are_all_refused_without_partial_activation` demonstrates the current single-handle divergence.
- **Manual evidence:** `workflow` 0/23 and `verification-stewardship` 0/24 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains the dual-handle/mismatch fragment and divergent single-handle behavior.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` must determine whether parameter-pack import is pilot-reachable; unit/source/tilt still require their cited schemas.
- **Next action:** reconcile the single-handle policy with the binding chapter, add unit/source/tilt and append-only ordinal rules, then implement all T32 arms.

## SB-DBM-033 - A categorical curve is a distinct type and is never linearly interpolated

- **Chapter evidence:** P2; chapter status `ABSENT`; owned test `SB-DBM-T33`; sections 4.5 and 6.6.
- **Atomic obligations:** categorical type is explicit; resampling produces only existing codes and reports boundary crossings; arithmetic refuses categorical inputs; all relevant writers/readers preserve the type.
- **Current source:** `curve_class` explicitly declares computed class curves. Reframe coerces categorical interpolation/mean to nearest/mode, frame blocking uses mode and refuses averages, and smoothing/despiking refuse class curves. Values still occupy the numeric curve store; boundary-crossing reports and universal equation arithmetic refusal are absent.
- **Qualifying acceptance tests:** none; T33's 0.1524-to-0.1 resample, boundary report and arithmetic refusal fixture is missing. Test class is `MISSING`.
- **Supporting tests:** `a_declared_class_curve_is_never_averaged_and_an_undeclared_one_keeps_the_method_asked_for`, `a_class_curve_is_carried_by_its_commonest_value_rather_than_averaged`, `a_class_curve_is_blocked_by_its_commonest_code_and_refuses_every_average`, and `a_class_curve_is_refused_by_smooth_and_despike_and_an_undeclared_one_is_not` prove important fragments from both sides.
- **Manual evidence:** `reframe` 0/34, `workflow` 0/23 and `data-conventions` 0/45 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains explicit class metadata and safe selected consumers; full type behavior is not integrated.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-003` must decide whether categorical/facies workflows are pilot scope.
- **Next action:** if selected, carry class type through every expression/resample boundary, add boundary reporting and implement the complete T33 fixture.

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
- **Current source:** `restore_log_set` is a first-class backend/Tauri/UI operation, but it replaces current rows with archive rows under the old `set_id` rather than creating a new version. `delete_log_set` deletes archive rows and the run record, so the archive is not append-only.
- **Qualifying acceptance tests:** none; T35's refused UPDATE/DELETE and version-4 restore/source-link fixture is missing. Test class is `MISSING`.
- **Supporting tests:** `db.rs::log_set_versioning_never_overwrites` and `workflow.rs::a_restored_log_set_version_feeds_the_next_module_run` prove current restore and history behavior while also characterizing archive deletion/old-id reuse.
- **Manual evidence:** `delivery-sets` 0/33, `project-lifecycle` 3/24 and `security-integrity` 0/63.
- **Git evidence:** accepted anchor `b332026c` contains the divergent restore/delete implementation.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `RECOVERY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** archive retention/prune policy must distinguish immutable history from separately authorized space reclamation.
- **Next action:** make restore append a new version with `restored_from`, prohibit ordinary archive mutation and implement every T35 arm; keep any future prune explicit and auditable.

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
- **Current source:** well groups and active membership are persisted in `db.rs`, but `list_wells` and many backend commands remain project-wide. `src/state.ts`/`wellScope.ts` filter or pass selected ids from the client, so a direct backend invocation can bypass the active-group policy.
- **Qualifying acceptance tests:** none; T37's direct-invocation inventory over 540/12 wells is missing. Test class is `MISSING`.
- **Supporting tests:** UI scope tests and selected command fixtures prove caller-supplied subsets, not backend enforcement across every iterator.
- **Manual evidence:** `well-scope` 3/9, `workflow` 0/23 and `security-integrity` 0/63.
- **Git evidence:** accepted anchor `b332026c` contains persisted groups and the client-enforced divergent boundary.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** a command-scope registry must distinguish deliberately project-wide operations from group-scoped operations.
- **Next action:** enforce/declare scope in one backend wrapper and implement T37 by direct command invocation with instrumented touched-well counts.

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
- **Current source:** `jobs.rs::ItemState` includes `Warned` and preserves severity; workflow all-NaN outputs can warn. ML writes selected cancelled/partial markers into `log_sets.params_json`. There is no general durable degradation field or universal clamp/substitution path, and finished jobs are pruned after 24.
- **Qualifying acceptance tests:** none; T39's clamp/substitution/clean batch plus 25-job prune and the store arm of T41 are missing. Test class is `MISSING`.
- **Supporting tests:** job-state and ML cancellation tests prove transient or subsystem-specific fragments only.
- **Manual evidence:** `workflow` 0/23, `processing-history` 0/7 and `machine-learning` 7/189.
- **Git evidence:** accepted anchor `b332026c` contains the transient and ML-specific fragments; universal durable honesty is not integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** a controlled, durable degradation vocabulary and its run-record column are missing.
- **Next action:** carry structured degradation reasons from computation to job view and run record, then implement T39 across the prune boundary.

## SB-DBM-040 - Cancellation honesty is regression-locked

- **Chapter evidence:** P1; chapter status `PRESENT-OK`; owned test `SB-DBM-T40` (`CHARACTERIZATION`); sections 4.6 and 6.7.
- **Atomic obligations:** an observing worker finalizes cancelled; a non-observing worker reports its actual outcome; non-cancellable jobs expose no cancel control; one regression test pins all three.
- **Current source:** `jobs.rs` separately records request and worker observation; `run_job` uses observation for final status; `JobView.cancellable` reaches `processingPanel.ts`, which creates a cancel button only for active cancellable jobs.
- **Qualifying acceptance tests:** none covers all three end-to-end outcomes. T40 as one complete characterization is missing, so test class is `MISSING`.
- **Supporting tests:** `cancellable_flag_reaches_the_view_both_ways`, `cancel_counts_as_cancelled_only_once_a_worker_observes_it` and `note_cancel_observed_marks_it_for_raw_flag_readers` pin backend halves. There is no DOM test for the absent control and no one test drives actual final phases for both worker behaviors.
- **Manual evidence:** `workflow` 0/23 and `machine-learning` 7/189.
- **Git evidence:** accepted anchor `b332026c` contains the intended behavior; complete regression closure is unverified.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** none; only the complete owned characterization is missing.
- **Next action:** implement T40 end to end with polling, non-polling and non-cancellable jobs, asserting both final phase and rendered control.

## SB-DBM-041 - A count presented as a total is a total; the inspector exposes the provenance tables

- **Chapter evidence:** P1; chapter status `PRESENT-DIVERGENT`; owned tests `SB-DBM-T41`, `SB-DBM-T42`; sections 4.6 and 6.7.
- **Atomic obligations:** `total_rows` has one meaning across inspector and SQL console or the capped count uses a different field; inspector whitelist includes all provenance/model/audit tables and can trace a curve without leaving it.
- **Current source:** paginated `get_table_page` computes the true count. `run_readonly_query` fetches `limit + 1`, sets `truncated`, then still puts the returned-row count in the same `total_rows` field. `TABLE_SPECS` omits `log_sets`, `computed_curves_archive`, `ml_models`, `curve_meta` and the absent audit tables.
- **Qualifying acceptance tests:** none; T41's 10,000/100 same-field contract and T42's full trace are missing. Test class is `MISSING`.
- **Supporting tests:** `readonly_query_flags_truncation_at_the_cap` intentionally defends the current alternative, and `every_inspector_table_returns_the_columns_it_declares` checks only the incomplete whitelist.
- **Manual evidence:** `database-tools` 0/2, `processing-history` 0/7 and `security-integrity` 0/63 - unexercised.
- **Git evidence:** accepted anchor `b332026c` contains both live divergences.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** T42 also depends on the structured audit tables in SB-DBM-011.
- **Next action:** rename/separate capped count semantics, derive the inspector inventory from the provenance registry and implement T41/T42 without weakening read-only SQL.

## SB-DBM-042 - The format-version gate and the pre-migration backup are contractual, and the backup names the format it can restore

- **Chapter evidence:** P0; chapter status `PRESENT-OK`; owned tests `SB-DBM-T01`, `SB-DBM-T02`, `SB-DBM-T43`; sections 4.6 and 6.1.
- **Atomic obligations:** a newer format refuses before mutation and leaves bytes identical; destructive migration first creates a fail-closed, non-overwriting, user-visible backup; additive migration creates none; filename names the source format restored at each step.
- **Current source:** `check_and_stamp_format` refuses newer files before schema work and names file/current versions and writer. `backup_before_destructive_migration` uses engine copy, aborts on copy error, suffixes collisions and posts a boot note; additive/no-op migration skips backup. Its filename uses current target `FORMAT_VERSION`, not the source version, so sequential shelves do not satisfy T43.
- **Qualifying acceptance tests:** none covers the complete compound contract. T01 lacks an actual before/after byte hash, T02 does not inject backup failure, and T43 is absent; test class is `MISSING`.
- **Supporting tests:** `future_format_is_refused_and_left_unmodified`, `destructive_migration_backs_up_the_project_file_first` and `fresh_project_open_writes_no_backup` characterize the implemented safety clauses and collision behavior.
- **Manual evidence:** `project-lifecycle` 3/24, `security-integrity` 0/63 and `verification-stewardship` 0/24.
- **Git evidence:** accepted anchor `b332026c` contains the strong gate/backup behavior and the source-versus-target naming divergence.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `RECOVERY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** no parameter is missing; the backup must derive the source stamp before migration, and failure/byte-identity tests are missing.
- **Next action:** name backups from the pre-migration source version, add fail-injection and byte-hash controls, and implement T43 across two destructive version steps.

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

- All 43 live SB-DBM rows were adjudicated exactly once against the accepted tree.
- As-built: 1 `PRESENT-OK`, 3 `PRESENT-UNVERIFIED`, 11 `PRESENT-DIVERGENT`, 13 `PARTIAL`, 15 `ABSENT`.
- Release disposition: 32 `PILOT-BLOCKER`, 8 `UNDECIDED`, 3 `DEFERRED`, 0 `OUT`.
- Test evidence: 1 `OPTIONAL-PACKAGE-IGNORED`; 42 `MISSING` qualifying whole-contract proofs. Supporting tests are retained above but are not counted as owned closure.
- Hard evidence blocks preserved: `MODULE_VERSION_SOURCE`, `ARTIFACT_HASH_ALGORITHM`, `SAMPLING_STYLE_VERIFY_TOLERANCE`, `INTERACTIVE_SET_CEILING`, the legacy UTC migration policy, real-well N-scale evidence and `DEC-003` pilot-method/workflow scope.
- No production, schema, PRD, generated verification, `REVIEW.md`, database write-discipline or model behavior was changed.
