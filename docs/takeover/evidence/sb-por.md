# SB-POR live-adjudication evidence receipt

## Execution baseline

- Date: `2026-08-11` (`Asia/Jakarta`).
- Branch: `codex/g1-sb-geo-adjudication`.
- Execution HEAD before adjudication: `381fadf8fc072eff58d224c631d0e7bb38ca8fb9`.
- Accepted implementation evidence anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0`, reachable from the execution HEAD.
- `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Merge base with `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Worktree: clean at entry; sole registered worktree `D:\XX. SandiBumi`.
- Pre-edit ledger: `279 / 931` adjudicated, `652` unadjudicated, `207` pilot blockers.
- Scope guard: exactly 62 source-owned rows, `SB-POR-001` through `SB-POR-062`; seventeen P0, twenty-five P1, seventeen P2 and three P3; all 62 `chapter_status` and `owned_tests` fields blank; all 62 live verdict fields untouched at entry.
- Test-intent guard: 41 actual IDs - T01 through T25, T14b, T18b and T28 through T41. Numeric T26 and T27 do not exist and are not invented.
- Parameter guard: 74 section-5 table rows; 15 rows literally carry `ABSENT`, 8 carry `NON-ADOPTABLE`, while front matter and closing prose claim 18 ABSENT parameters/rows. The discrepancy remains visible and no value is introduced here.
- Manual evidence boundary: Jauhar owns and will perform manual review. This pass neither exercises nor checks a manual scenario; it reads only the generated matrix and existing checked scenarios.
- Retrieval boundary: Clauding retrieval returned a weak chapter-only match. The live repository, immutable chapter, independently sourced tests, generated manual matrix and reachable history are the evidence; the retrieval gap is not filled from model memory.
- Navigation boundary: the codebase-index MCP server is not callable in this task. Consequential negative findings are therefore confirmed with targeted current-source, exact-test and reachable-history searches.

## Product-owner adjudication addendum - 2026-08-11

This addendum records the product direction supplied after review of the 62-row receipt. It changes
release intent and closes named product decisions; it does not claim any production behavior, test
or manual scenario changed.

| Decision | Current stand | Engineering consequence |
|---|---|---|
| `DEC-012` - invalid Wyllie compaction factor | **DECIDED:** refuse when `Cp < 1` | SB-POR-017 stays `PRESENT-DIVERGENT` and a pilot blocker until the refusal is observable and T29 pins invalid and valid controls. Help text is explanatory only. |
| `DEC-013` - POR output names | **DECIDED:** user-configurable names; intentional same-name reuse is explicit replacement, never silent collision | Distinct user names must preserve parallel POR results. Explicit replacement must preserve version/undo custody. Imported and computed identities remain distinguishable. |
| `DEC-014` - POR method separation | **DECIDED:** Arithmetic and RMS remain available; Gaymard-Poupon HC response and coupled porosity-`Sxo`/`Sw` iteration are mandatory separate contracts | The five roles below must be separately named, selected, proven and recorded. Exact equations, parameters, tolerances and endpoints remain governed by the chapter and cited sources. |
| `DEC-015` - SB-POR-001 envelope | **DECIDED 2026-08-12:** method-specific correction limits and validity rules beneath a common typed envelope | The common contract standardizes POR family, provenance, output roles and observable flags/reasons. Each method retains its own cited numerical limits and correction rules; bounds are never borrowed across methods and an uncited bound remains absent. |
| `DEC-016` - required POR capability set | **DECIDED:** analytic N-D, HC response, excavation and neutron-sonic belong in the product | Inclusion does not close ESC-POR-8, tool/source custody, any absent parameter, an implementation, a test or a separately deferred pilot-timing row. |

### Distinct POR contracts now carried

1. **Arithmetic comparison output** - available as an explicitly labelled comparison quantity; it is not an authoritative analytic crossplot method or pay default.
2. **RMS comparison output** - available under the same comparison/pay-exclusion boundary; it is not silently promoted to the analytic N-D answer.
3. **SSC/SSPW RMS conditioning** - the narrow parity contract already specified by SB-POR-059; this internal role is not evidence that RMS is an authoritative crossplot method.
4. **Gaymard-Poupon hydrocarbon response** - mandatory, with the source-bound electron-density, hydrogen-index, validity, provenance and double-correction protections of SB-POR-029 through 038. The UI and provenance must use the full method identity because `Gaymard` alone is also used in the literature for an RMS quick rule.
5. **Coupled porosity-`Sxo`/`Sw` iterative solve** - mandatory and separate from all three shortcut/conditioning roles, with no partial iterate on failure and with explicit convergence, precedence and configuration refusal under SB-POR-035 and SB-POR-050 through 052.

Current measured stand after this addendum: 62/62 POR rows remain adjudicated; 21 are
`PRESENT-DIVERGENT`, 15 `PARTIAL`, 25 `ABSENT` and 1 `PRESENT-UNVERIFIED`. Promoting the two
solver-discipline rows made mandatory by `DEC-014` changes the POR release split to 44
`PILOT-BLOCKER`, 13 `UNDECIDED` and 5 `DEFERRED`. Test evidence remains 6
`CHARACTERIZATION`, 56 `MISSING`, 0 `CORRECTNESS`; manual POR evidence remains 0/33.

### Current stand at a glance

- **Architecture:** a POR quantity family and complete method/convention/run provenance are required. User-configurable output names are settled by `DEC-013`. `DEC-015` settles a common typed custody/observability envelope with method-specific correction limits and validity rules.
- **Numerical limits and flags:** method-specific physics is not automatically an error, but a silent branch or clamp is. Existing hard bounds become cited, visible parameters and binding is observable; an uncited endpoint remains absent.
- **Sonic:** the chapter's truthful naming and per-method shale conventions remain the adopted target. `Cp < 1` is a hard refusal. `DEC-017` closes SP-013's product choice on the genuine original three-segment RHG80 route, not rename-only; exact equation typography still requires verification against the original scan, and SB-POR-020's separate vendor-rendering choice remains open.
- **N-D and gas:** Arithmetic and RMS remain available only in their explicitly named roles. Gaymard-Poupon HC response and the coupled porosity-`Sxo`/`Sw` iteration are mandatory separate contracts. SB-POR-059's RMS parity fix remains narrow and does not implement either rigorous contract.
- **Missing capability:** analytic N-D, HC response, excavation and neutron-sonic are required product capabilities under `DEC-016`; no missing source or parameter is supplied by that inclusion decision.
- **Proof:** every atomic contract still needs an independent correctness oracle. No implementation-derived snapshot is promoted. SB-POR-001 now has one qualifying architecture correctness test; the other 61 POR rows retain their recorded evidence classes, and Jauhar retains ownership of all 33 manual POR checks.

### DEC-015 operational boundary

| Common to every POR result | Owned by the selected method/correction |
|---|---|
| POR quantity family and semantic output role | Equation and correction direction |
| Method, convention, input-curve and run provenance | Admissible input basis and physical validity domain |
| User-configurable output name with version/undo custody | Source-bound floors, ceilings, clamps and other numerical limits |
| One observable per-sample branch/limit reason shape and run-level reporting shape | Which declared reason fires when that method's rule binds or refuses |
| Uniform missing/refusal and no-silent-write behavior | Correction ordering and iterative configuration, where applicable |

SB-POR-001's "one limiting contract" is therefore carried as one interface and custody contract,
not one universal set of endpoints. No method may silently inherit another method's correction,
limit or validity range. If the selected method lacks an admissible cited value, that value remains
absent and the method refuses rather than falling back to a neighboring method.

## Chapter and cross-domain findings carried into every row

- SP-009 remains open: all source-owned POR status and test fields stay blank, and numeric T26/T27 stay absent.
- SP-012's product decision is closed by `DEC-012`: `Cp < 1` is refused. The shipped Wyllie path remains divergent until the refusal and its regression test are implemented.
- SP-013's product choice is closed by `DEC-017`: the shipped one-segment approximation is not retained as SandiBumi's RHG method; the product direction is the original three-segment RHG80 transform. Production remains blocked until the exact scanned equation typography is reverified. This does not choose IP, Geolog or Techlog as SB-POR-020's authoritative vendor rendering.
- SP-014 remains open: one user-visible sonic description contains a prohibited geographic parenthetical. This receipt names the surface without reproducing the proper name.
- SP-015 supplies primary-source citations for the compaction estimator and RHG's no-compaction-correction property; it does not authorize an implementation choice.
- The chapter's 18-ABSENT claim does not mechanically match its 15 ABSENT-bearing rows. The mismatch is not normalized and every parameter is adjudicated from its own row.
- Current `PHIE_FLOOR = 0.001` implements a later direct product decision, while SB-POR-045 requires the conflicting vendor values to ship with no default. Current behavior, chapter contract and later decision remain separate pending product-owner precedence.
- `DEC-013` permits user-configurable POR output names and intentional, versioned replacement; it does not permit a silent collision or loss of imported/computed identity.
- `DEC-014` makes Gaymard-Poupon HC response and the coupled porosity-`Sxo`/`Sw` iterative path mandatory separate contracts. Arithmetic, RMS comparison and SSC/SSPW RMS conditioning remain separately identified roles.
- `DEC-015` is closed by Jauhar's explicit option-2 selection on 2026-08-12: method-specific numerical correction/validity rules under one common POR family/provenance/output-role/flag envelope.
- ESC-1, ESC-2, ESC-3, ESC-5, ESC-7 and ESC-POR-8 remain source/custody boundaries. Protected vendor charts and binaries are not opened or copied, and non-adoptable constants do not become defaults.
- Manual capability baseline: porosity 0/33, generic-curve-store 0/18, conditioning 0/27, workflow 0/23, las-export 0/2 and processing-history 0/7; histogram 5/22 and crossplot 6/13 do not prove POR custody or correctness.

## SB-POR-001 - One deterministic POR family and contract

- **Specified contract:** every deterministic porosity method belongs to one POR family and uses one limiting, flag and output-naming contract; T39 is the primary discriminator, with T11 and T31 as cross-support.
- **Current implementation:** every live `Porosity` module and each of its 21 porosity outputs is registered under one serialized `POR` envelope carrying module role, method, convention, semantic output role, common limiting-interface identity, method-specific source-linked limit policy, common reason-schema identity and the existing workflow output-naming contract. `phi_dn` is explicitly a comparison producer rather than an authoritative analytic method; `phimax` is a limit producer rather than a porosity interpretation method; sonic discloses its current mixed convention pending SB-POR-013. The dialog shows POR role and method beside each output and exposes the remaining policy detail in its tooltip. No numeric bound was moved into the common envelope.
- **Qualifying acceptance tests:** `every_porosity_module_uses_one_envelope_while_each_result_producer_keeps_its_own_limit_policy` was witnessed RED before the contract fields/registry existed and again on an incorrect lowercase rename expectation, then passed. It independently inventories all six live POR modules and 21 porosity outputs, proves the common identities, distinct policies, D-N comparison role, `phimax` limit-producer role, user-configurable uppercase naming and explicit `PENDING_SB_POR_003` emission state. Removing `phi_son.PHIE_SON` and borrowing density's policy both fail the immutable registry gate. Test class `CORRECTNESS`; sources are SB-POR-001/T39 and DEC-015.
- **Supporting tests:** `every_module_returns_the_output_keys_its_manifest_declares`, the density and D-N branch tests, and the sonic option test each passed exactly once; they prove local manifest/arithmetic behavior only.
- **Manual evidence:** porosity 0/33; workflow 0/23; generic-curve-store 0/18.
- **Source/parameter boundary:** no new value is needed; this is an architecture contract.
- **History/reachability:** the common envelope, central fail-closed registry and owned test are introduced by the current Gate 2 increment; the existing method arithmetic remains unchanged.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for SB-POR-001 after DEC-015. SB-POR-002 still owns missing unlimited/limited pairs, SB-POR-003 owns actual per-sample reason emission, and SB-POR-004 owns persisted POR-family/method/convention curve custody and collision discipline.
- **Next action:** preserve the common envelope and distinct policy identities; continue SB-POR-002 without claiming the pending reason stream or persisted curve provenance is already implemented.

## SB-POR-002 - Unlimited and limited pairs for every method

- **Specified contract:** every method must preserve both unlimited `PHIT/PHIE` and limited `PHIT/PHIE`, with meanings visible through write, reload and export; T11, T19, T31 and T39 jointly constrain it.
- **Current implementation:** density and D-N retain unlimited method-specific twins and shared limited outputs. Sonic has only one method-specific pair and clamps it in place. The SB-POR-001 registry classifies SSC/SSPW as deterministic methods, but both discard pre-limit values inside protected `ssc.rs`; their final porosity is already downstream of component and geometry clamps, so copying the value immediately before only the last clamp would not establish a complete unlimited lineage.
- **Qualifying acceptance tests:** none. No whole-family test can presently prove both pairs and their custody without either mislabelling limited SSC/SSPW values or first deciding DEC-038. Test class `MISSING`.
- **Supporting tests:** `a_negative_density_porosity_is_floored_but_stays_visible_in_the_unlimited_twin` closes only density and D-N examples. Manifest-output parity proves declared keys, not semantic unlimited meaning; generic LAS round trip proves numeric transport, not that the right lineage was stored.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18; las-export 0/2.
- **Source/parameter boundary:** the contract introduces no endpoint or default.
- **History/reachability:** current source was reverified at parent `349be592`; this blocker increment changes no production behavior, no numeric limit and no protected file.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`; Gate 2 `BLOCKED-DECISION/BOUNDARY` on DEC-038, the protected-file rule and SB-POR-004; no false closure.
- **Blocker or decision:** DEC-038 must decide whether SSC/SSPW are methods governed by the twin contract or separately typed workflows. If governed, it must define which upstream limits the unlimited lineage bypasses; implementation then requires an explicit narrow exception for `ssc.rs`. Distinct write/reload/export custody still depends on SB-POR-004.
- **Next action:** settle DEC-038 and SB-POR-004; then preserve separately named unlimited and limited pairs at the approved calculation boundary and prove both through write, reload and export, including distinct-name survival and explicit replacement/restore controls.

## SB-POR-003 - Per-sample branch and limit flag

- **Specified contract:** one POR flag must identify every material branch and every binding floor/ceiling per sample; T12, T38, T39 and T41 require positive and negative controls.
- **Current implementation:** POR methods clamp numeric outputs or leave `NaN` without a POR flag. `BADHOLE` and conditioning flags are separate binary detector outputs, workflow masking is generic, and `SW_METHOD` demonstrates a categorical `f32` class curve only for a different domain. None carries POR method branch plus every simultaneously binding limit. SSC/SSPW branch and clamp sites remain inside protected `ssc.rs`.
- **Qualifying acceptance tests:** none. A binary flag would lose branch/limit identity; choosing arbitrary class codes would invent a wire contract; and no exact whole-family test can populate protected SSC/SSPW paths. Test class `MISSING`.
- **Supporting tests:** binary flag polarity and saturation method-code tests prove the two existing storage shapes separately. `badhole`/`condflag` tests prove detectors only; none proves the singular POR semantics, combinations, class metadata, persistence or export.
- **Manual evidence:** porosity 0/33; conditioning 0/27; processing-history 0/7.
- **Source/parameter boundary:** IP codes 6/7/9/16 and Geolog's `MTH_PHI = SHALE` are evidence, not a complete SandiBumi combination vocabulary. Numeric schema identifiers are not petrophysical parameters, but their stable meaning is still a product contract and cannot be inferred from current code.
- **History/reachability:** current source was reverified at parent `f8d1d0eb`; this blocker increment changes no production behavior, no flag value and no protected file.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`; Gate 2 `BLOCKED-DECISION/BOUNDARY` on DEC-039, DEC-038, the protected-file rule and later conditioning behavior; no false closure.
- **Blocker or decision:** DEC-039 must choose the singular representation, complete initial vocabulary, simultaneous-limit rule, categorical metadata and unknown/fractional-code behavior. DEC-038 decides SSC/SSPW scope; implementation across every registered method then needs explicit protected-file authority. T41's conditioning branches remain owned by SB-POR-047/048.
- **Next action:** approve DEC-039/038, then populate one registered POR reason state at every method branch and clamp; prove normal, missing, each single limit, simultaneous limits, unknown-code refusal, categorical reframe, write/reload and export from both sides.

## SB-POR-004 - Typed POR family, provenance and collision-free names

- **Specified contract:** porosity curves carry POR quantity-family typing plus method/convention provenance; sequential methods preserve distinct outputs and imported-versus-computed identity; T31 and T32 pin both sides.
- **Current implementation:** the generated unit registry carries canonical `POR`/`v/v` metadata for the shipping POR mnemonic set. Density keeps default `PHIE`/`PHIT`, while the D-N comparison producer defaults to `PHIE_DN_LIM`/`PHIT_DN_LIM`; resolved user renames and prefixes remain authoritative. The shared input resolver and pay summary prefer exact canonical PHIE/PHIT when present and otherwise follow the exact D-N limited alias; an explicit interpreter selection always wins, and no generic POR-family scan silently elects among methods. Before each POR run, the workflow resolves every emitted curve name and persists a curve-specific `POROSITY_OUTPUT.<resolved-name>` ancestry parameter containing its family, method, volume convention, role, limit/flag contract and naming contract. Imported `PHIE` remains source-identified in `curve_meta`; a computed `PHIE` is separately identified by its run ancestry. An intentional same-name run still uses the existing append-only versioned replacement and restore discipline.
- **Qualifying acceptance tests:** `porosity_methods_keep_distinct_default_names_and_each_curve_carries_family_method_and_convention_while_explicit_replacement_stays_versioned_and_restorable` was witnessed RED first when `PHIE`, `PHIT`, `PHIA` and `DPHI` lacked a family and again when a sole `PHIE_DN_LIM` producer could not satisfy a downstream logical PHIE role, then GREEN. It independently pins T31 and T32 from both sides: distinct defaults and sequential survival, POR family resolution, rename-plus-prefix custody, canonical-first/exact-D-N-fallback input resolution, per-output density versus D-N method/convention, imported-versus-computed identity, explicit same-name replacement, versions 1/2 and restore as version 3. Test class `CORRECTNESS`; expected identities and behavior come from SB-POR-004, F16, T31/T32 and DEC-013, while the numeric witness uses the chapter-cited endpoint fixture already owned by the module suite.
- **Supporting tests:** `every_porosity_module_uses_one_envelope_while_each_result_producer_keeps_its_own_limit_policy`, `a_restored_log_set_version_feeds_the_next_module_run`, `an_output_pattern_is_the_default_name_and_a_rename_replaces_it`, `families_resolve_common_mnemonics`, `chain_runs_steps_in_order_and_completes`, both `core_determinism_tests` and `the_monte_carlo_chain_ignores_a_step_mask_the_real_chain_honours` passed in focused regression runs; `tools/unit-registry.test.mjs` also passed. The chain and determinism fixtures now require the physical `PHIE_DN_LIM` identity instead of the deliberately retired D-N `PHIE` collision, without weakening their original completion or byte-determinism assertions.
- **Manual evidence:** generic-curve-store 0/18; workflow 0/23; las-export 0/2.
- **Source/parameter boundary:** not numeric; imported and computed identity must remain distinct.
- **History/reachability:** parent `691a0055` lacked POR family custody and collision-free D-N defaults; this increment adds both without changing the protected database write discipline.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. `DEC-013` is implemented: distinct defaults preserve parallel results; user-resolved names remain configurable; intentional same-name reuse is explicit, append-only and restorable.
- **Next action:** Jauhar performs the open Visual, Manual and Field checks; automated Gate 2 work proceeds to SB-POR-006, the first unhandled row of the approved 222-row program. SB-POR-005 is DEFERRED and outside that manifest.

## SB-POR-005 - Separately named correction and forward functions

- **Specified contract:** whenever a correction has both inverse and forward directions, each direction is a separately named public function and round-trips under T07.
- **Current implementation:** no POR hydrocarbon or excavation correction/forward pair exists in Rust modules, Tauri commands, TypeScript callers or tests. Private algebra or generic transforms cannot satisfy the named-direction contract.
- **Qualifying acceptance tests:** none; T07 is not executable. Test class `MISSING`.
- **Supporting tests:** no POR-direction test exists.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no direction is implemented and no parameter is selected.
- **History/reachability:** exact current, test and reachable-history searches found no POR correction/forward pair.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion and the underlying correction model must be decided before an API is created.
- **Next action:** if admitted to pilot scope, define distinct typed APIs and prove both directional formulas independently before round-trip testing.

## SB-POR-006 - Typed VSH/VCL input with wrong-family refusal

- **Specified contract:** POR methods consume an explicitly typed VSH or VCL quantity, preserve that distinction, and refuse untyped or wrong-family inputs rather than trusting mnemonic text.
- **Current implementation:** the 2026-08-11 reading is superseded. `modules::apply_shale_clay_quantity_contracts` types every POR shale/clay consumer by module argument identity rather than by mnemonic, and `workflow::validate_shale_clay_input_quantities` reads the producer-owned quantity for the exact resolved curve - `curve_meta.family` for imported curves, versioned run ancestry for computed ones - and refuses per well, before any write, when that quantity is absent or is the other family. `workflow::complete_module_log_spec` refuses again at the batch stage. The guard lives in `workflow.rs` rather than the `curves.rs` location the original row expected; the requirement fixes the contract, not the file.
- **Qualifying acceptance tests:** `every_porosity_method_that_consumes_a_shale_or_clay_volume_declares_the_quantity_it_accepts_and_refuses_an_untyped_or_wrong_family_curve`. Test class `CORRECTNESS`; the expected identities and the refusal-is-the-requirement rule come from `docs/PRD_v2/11_porosity.md` SB-POR-006 and F15. The oracle is acceptance versus refusal, never a number. Three mutations were witnessed RED and restored: replacing the untyped refusal with `continue` (the accepted control then failed, because deleting the per-well guard promotes one well's untyped volume into a whole-batch `build_error`), dropping `phi_son` from the typed inventory, and additionally loosening the expected inventory so only the independent registry-family sweep could catch it. Sides A and B therefore cover each other and neither alone would pass.
- **Supporting tests:** the SB-CLY-043 pair `a_required_shale_volume_accepts_renamed_shale_metadata_and_refuses_clay_metadata_even_under_a_vsh_name` and `a_clay_volume_consumer_accepts_clay_refuses_shale_and_records_which_quantity_it_received` prove the shared typed seam on `thin_bed_ts` and `brittleness`. Neither exercises a porosity method and neither covers the untyped arm, which is SB-POR-006's headline.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18; workflow 0/23.
- **Source/parameter boundary:** no shale endpoint is inferred; this is input custody. The `CSR` bridge that would convert a clay volume into a shale volume is SB-POR-012, outside the approved 222-row program, so nothing here converts between the families - it refuses.
- **History/reachability:** the typed seam arrived with SB-CLY-043 and the POR family with SB-POR-004 (`12d03b8b`), both after this row was last read on 2026-08-11. This increment adds the POR-specific proof and changes no production behavior.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. The named dependency - the quantity-family registry and an observable refusal surface - is satisfied.
- **Named residual, not claimed as covered:** `montecarlo.rs:1287` calls `modules::run_module` directly and therefore never reaches `validate_shale_clay_input_quantities`, so an in-memory Monte Carlo realization would consume an untyped volume without refusal. That file is on the prohibited list and Monte Carlo is an explicit first-pilot exclusion (`PILOT_SCOPE.md`), so no edit was attempted and no coverage is claimed for it. The manifest-level typing proven by sides A and B does hold for those specs. Closing this residual belongs to whichever gate admits Monte Carlo.
- **Next action:** Jauhar performs the open Visual, Manual and Field checks; automated Gate 2 work proceeds to SB-POR-007.

## SB-POR-007 - Parameter citation and evidence tier in the dialog

- **Specified contract:** every POR parameter carries its source citation and evidence tier, the dialog displays both, and the chosen value/source is retained in run provenance.
- **Current implementation:** the seam was already complete and the tracker was stale about it. `ArgSpec.sources_topic` exists, `ParamSource` already carries a `tier` field, `ParameterEvidence`/`ParameterDecision` are already persisted into run ancestry by `workflow.rs:718` and `:958`, and `paramSources.ts:109` already renders `tier · source` beside the input from both `moduleDialog.ts:446` and `workflowDialog.ts:458`. What was missing was POR population. Eight section 5 topics are now registered - fluid density, formation-water density, maximum effective porosity, porosity limiting mode, matrix/fluid/shale transit time and sonic compaction correction - and attached across `phi_den`, `phi_dn` and `phi_son`, joining the four density and neutron topics already carried. Parameters section 5 registers no row for stay untopiced: `OPT_XPLOT` (owned by SB-POR-023 under DEC-014), `OPT_SON` (the SB-POR-013..020 sonic group under DEC-017) and `phimax`'s SandiBumi-own compaction trend, whose ceiling is deliberately not merged with section 5.3's IP shale roll-off triple.
- **Qualifying acceptance tests:** `every_cited_porosity_parameter_carries_its_section_five_source_and_tier_while_an_absent_default_stays_absent` pins the contract from four sides: the exact section 5 topic map across the three modules; every named topic resolving to completely attributed, tiered positions; five deliberately ABSENT parameters keeping an empty default and `ABSENT` default source *after* being sourced; and the parameters section 5 does not cover staying untopiced. A fifth arm proves the tier survives into the run record through the exact `decision_for` call the runner makes. Three mutations produced RED at three different assertions - a dropped `phi_son` OPT_CP attachment, a sourced `DT_SH` given the attested Techlog 100 as a default, and an invented citation on `OPT_XPLOT`. Test class `CORRECTNESS`; every expected value, source and tier is transcribed from `docs/PRD_v2/11_porosity.md` section 5 and its tier key at lines 7-19, never read back from the manifests.
- **Supporting tests:** the dialog's `tier · source` rendering and its no-silent-selection behavior are already proven by the SB-CLY-050 frontend test `a_disputed_parameter_stays_empty_beside_every_source_and_failed_evidence_loading_stays_visible`; this increment does not duplicate it, because what changed is that POR arguments now have a topic for that component to render.
- **Manual evidence:** porosity 0/33; workflow 0/23; processing-history 0/7.
- **Source/parameter boundary:** no value was requested from the product owner and none was invented. Section 5 supplies a source and tier for all 74 rows, and the 18 `ABSENT` and the `NON-ADOPTABLE` rows are first-class states, so disclosure was registered without creating a default anywhere. `RHO_DSH` still ships 2.65, which matches neither attested value; making it `ABSENT` is SB-POR-055's row and was deliberately not done here.
- **History/reachability:** parent `1a7535d0` had the generic seam and no POR topics. This increment adds only registry entries and manifest attachments.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none for the registered scope. Two residuals are named rather than counted as covered: `ssc`/`sspw` parameters live in prohibited `src-tauri/src/ssc.rs` and remain unsourced (both are first-pilot exclusions, per the owner's scope decision for this row); and the same primary tier is spelled `T1p` in this chapter, `T1′` in the registry rows pinned by earlier CLY/CORE increments and `T1-prime` in a frontend fixture, which is recorded rather than resolved because re-spelling another requirement's evidence is its own change.
- **Next action:** Jauhar performs the open Visual, Manual and Field checks; automated Gate 2 work proceeds to SB-POR-008.

## SB-POR-008 - One formation-water PHIT_SH across the CLY seam

- **Specified contract:** every POR formation path uses and exports one shared formation-water-based `PHIT_SH`, kept distinct from shale subtraction; T15, T19 and T30 compare all paths.
- **Current implementation:** the quantity now has exactly one definition in the tree, `modules::shale_total_porosity(rho_dsh, rho_sh, rho_w)`, carrying its own contract note. `phi_den` and `phi_dn` reach it through `phit_sh_at`; `sspw` now reaches it directly and declares a cited `RHO_W`, replacing the fluid-anchored local formula at the old `ssc.rs:464`. `phi_son` has no shale-porosity term, so nothing there needed unifying.
- **Owner authorization:** Jauhar authorized the narrow protected-file edit on 2026-08-16, choosing option (a): add a formation-water parameter and route SSC/SSPW through the shared helper while preserving all existing limited arithmetic. **Investigation then narrowed that authorization, and the narrowing is the substantive finding.** `ssc`'s own `(rhob_dsi - rhob_wsi)/(rhob_dsi - rhob_fl)` is not this quantity: `rhob_dsi` is the intersection of the fluid-anchored line `m3` with the dry-clay line `m4`, so the expression is the wet-silt point's fractional distance along `m3` from dry silt toward the fluid point. Its denominator must remain `rhob_fl`, because changing it would stop the expression being a fraction along the very line that defined its numerator. Routing that site through the shared helper as literally authorized would have introduced a new silent error while fixing another, so its arithmetic is unchanged and only its colliding local name was retired to `silt_water_fraction` - which is what F16 actually requires.
- **Qualifying acceptance tests:** `one_formation_water_clay_bound_water_porosity_serves_every_porosity_method_and_the_silt_and_shale_subtraction_terms_keep_their_own_identities` pins five sides: the shared helper equals the chapter's own form evaluated independently; the formation-water and fluid anchors are genuinely separable at the cited salt-water density; every method carrying the quantity declares `RHO_W`; the whole production tree holds exactly **one** definition, so a module re-deriving it locally fails even though every other assertion would still pass; and F16's naming rule holds from both sides. Two mutations produced RED at two different assertions - `sspw` reverted to the fluid anchor, and the silt term reclaiming the shale name. Test class `CORRECTNESS`.
- **Supporting tests:** all six existing SSC/SSPW behaviour tests pass with **unchanged expected values**. That is the deliberate control: the fixture holds `RHO_W` equal to `RHOB_FL`, so their unchanged results prove the routing change is behaviour-neutral wherever the two anchors agree, and the new test proves it is not neutral where they differ.
- **Manual evidence:** porosity 0/33; shale-volume 0/17; workflow 0/23.
- **Source/parameter boundary:** `docs/PRD_v2/11_porosity.md` SB-POR-008 supplies the required form and F16 the naming rule. Section 5.1 supplies the witness values - `RHO_DSH` 2.78 (IP `Rho Dry Clay`), `RHO_SH` 2.50 (Techlog `DEN_shale`) - and the fluid fresh `1.00` versus salt `1.10` spread that makes the anchors separable. The new `RHO_W` parameter copies `phi_den`'s cited declaration verbatim (Geolog V14 `phi_den.info` `RHO_W` DEFAULT 1000 k/m3), so no value was invented and the default is unchanged at 1.00.
- **History/reachability:** parent `e65271f2` recorded this row BLOCKED on the protected file; the owner authorization lifted that boundary for this narrow purpose only.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none remaining for this row's contract. The CLY consumer `clsr_porosity_corrected` (SB-CLY-044) stays deferred outside the manifest and is not required for the "defined once, formation-water anchored, distinctly named" contract this row owns; when SB-CLY-044 is admitted it must consume `modules::shale_total_porosity` rather than re-derive it, which arm D of the test now enforces mechanically.
- **Next action:** Jauhar performs the open Visual, Manual and Field checks; automated Gate 2 work proceeds to SB-POR-009. A `DEC` row recording the 2026-08-16 `ssc.rs` authorization still needs adding to `DECISIONS.md`, which was outside this program's allowed paths.

## SB-POR-009 - PHIT is never below PHIE

- **Specified contract:** `PHIT >= PHIE` holds at every sample by construction - limit `PHIE` first, then rebuild `PHIT` from the limited value (Geolog's ordering, F21) - and the invariant must additionally be asserted, not merely relied on.
- **Current implementation:** `phi_den` and `phi_dn` hold it for free because they rebuild `PHIT` from the limited `PHIE`. `phi_son` computed the two independently and **did violate it**: `pe = pt - VSH*(DT_SH - DT_MA)/(DT_FL - DT_MA)/Cp`, so when `DT_SH < DT_MA` the shale term is negative and the subtraction becomes an addition. `DT_MA` 70 and `DT_SH` 60 are both inside the shipped declared ranges (`DT_MA` 40..70, `DT_SH` 60..150), giving `PHIT_SON 0.0840` against `PHIE_SON 0.1008` - effective porosity 20 percent above total with every input nominally valid. `phi_son` now bounds `PHIE` by the sample's own already-limited `PHIT`, which is the same construction `ssc` and `sspw` already use.
- **Qualifying acceptance tests:** `every_porosity_method_keeps_total_porosity_at_or_above_effective_porosity_at_every_sample` pins four sides: it executes `phi_den`, `phi_dn` and `phi_son` across WYLLIE/RHG, Cp on/off and both PHIE-limiting modes, pairing outputs by declared name so limited and unlimited pairs are both checked; it proves separately that the chosen in-range sonic parameters really do invert the ordering before enforcement, so the sweep cannot pass by never stressing it; it proves `ssc`/`sspw` bound effective by total porosity structurally; and it refuses to let any registered Porosity-category method emitting a PHIT/PHIE pair escape coverage. Removing the enforcement reproduces the exact violation as RED. Test class `CORRECTNESS`.
- **Supporting tests:** `phi_son_wyllie_cp_opt_in_only_scales_wyllie` passes unchanged, so the ordering guard does not disturb the compaction behaviour it pins.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** SB-POR-009 and F21 supply the invariant and the ordering; the adversarial witness comes from the shipped manifest's own declared ranges rather than a chosen number. **No new numeric bound was introduced** - the ceiling imposed is the sample's own total porosity, not a constant. The invariant is an ordering contract and is independent of the unresolved `SB-POR-045` floor VALUE, so DEC-026 does not gate it.
- **History/reachability:** the violation was reachable in shipped code for every sonic run whose shale slowness fell below its matrix slowness.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none. `ssc`/`sspw` are proved structurally rather than executed because `ssc.rs` is protected and the 2026-08-16 authorization covered SB-POR-008 only; the invariant holds there by construction, so no edit is required.
- **Next action:** Jauhar performs the open Visual, Manual and Field checks; automated Gate 2 work proceeds to SB-POR-010.

## SB-POR-010 - Re-derivable porosity audit trail

- **Specified contract:** every porosity curve should carry, in the project audit trail, the method name, the full parameter set and the input curve identities that produced it, sufficient to re-derive it without the session.
- **Current implementation:** most of the record now exists and the row's old description understates it. `SB-DBM-003` persists each parameter value with its source state; `SB-POR-007` adds the evidence tier for every section 5 parameter; `SB-POR-004` persists a per-output `POROSITY_OUTPUT.<name>` contract carrying family, method, volume convention, output role and naming contract; `SB-DBM-006` records the resolved input-curve identities. What remains absent is the **re-derivability** clause: no stored manifest resolves module identity, options and defaults into one replayable record, so a curve still cannot be re-derived without its authoring session.
- **Qualifying acceptance tests:** none, and deliberately so. Test class `MISSING`. A proof assembled from the fields that happen to be stored would assert re-derivability while no manifest resolver exists - asserting exactly the clause this row is about.
- **Supporting tests:** the SB-DBM-003, SB-DBM-006, SB-POR-004 and SB-POR-007 proofs each pin their own arm of the record and all pass; none of them, alone or together, establishes replay.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** not numeric. No value was requested or invented.
- **History/reachability:** the custody arms are integrated and reachable; the manifest resolver is absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED-DEPENDENCY`. The re-derivability clause is `SB-DBM-015`'s complete re-run manifest, which is itself blocked on `DEC-021` build-derived module identity, `DEC-023` zone-set identity and `DEC-024` manifest identity seams. Method, parameter, source/tier, convention and input-identity custody are already in place and are not what is missing.
- **Next action:** settle DEC-021, DEC-023 and DEC-024, close SB-DBM-015, then prove a POR curve replays from its stored manifest alone without querying any mutable default.

## SB-POR-011 - One shared matrix density across chained modules

- **Specified contract:** matrix density must be a single shared parameter across modules that a documented workflow chains.
- **Current implementation:** the divergence the requirement names is still live and reachable. `phi_den` (`modules.rs:3056`), `phi_dn` (`:3186`) and `condflag` (`:3951`) each ship `RHO_MA` 2.645; `gascorr` (`:4372`) ships 2.65, and `gascorr`'s own doc instructs chaining it with the porosity modules. A further POR manifest declares `RHO_MA` deliberately ABSENT (`param_open` at `:2879`). SB-POR-007 put every one of them on the shared `MATRIX_DENSITY` source topic, so the competing positions are now disclosed at the point of entry - but disclosure is not unification, and the shipped defaults still disagree.
- **Qualifying acceptance tests:** none. Test class `MISSING`. A test could prove propagation of a user-set override, but it could not establish the shared DEFAULT the requirement asks for without first choosing a value.
- **Supporting tests:** the SB-POR-007 registry proof pins that all four consumers expose the same cited `MATRIX_DENSITY` positions with their tiers.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** section 5.1 cites BOTH values and adjudicates neither - 2.65 as the three-way endpoint agreement across IP MINDEF, Techlog `QM_MineralTable` and SandiMin (tier T3), and 2.645 as `Geolog phi_den.info RHO_MA DEFAULT = 2645 k/m3`, explicitly recorded as a shipped module default that **differs from the endpoint libraries** (tier T1). Neither is marked ABSENT or NON-ADOPTABLE. One shared parameter carries one default, so the value is a product decision and registering either as authority would override the other's citation.
- **History/reachability:** all four literals are live in shipped manifests.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `BLOCKED-DECISION`. Jauhar chooses which cited value the shared matrix density carries, or elects ABSENT so a run refuses without a user value.
- **Next action:** record the choice and its source label, then introduce one typed shared reference and prove a non-default override reaches `phi_den`, `phi_dn`, `condflag` and `gascorr` unchanged.

## SB-POR-012 - CSR bridge with no default

- **Specified contract:** implement the four specified CSR relationships, require explicit CSR, refuse its absence, clamp only where specified, and ship no CSR default; T23 pins both refusal and explicit-value paths.
- **Current implementation:** no CSR argument, bridge, family conversion, dialog control or refusal exists in the POR/CLY path. A direct VSH/VCL mnemonic or implicit CSR of one is not implementation.
- **Qualifying acceptance tests:** none; T23 is not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; shale-volume 0/17; workflow 0/23.
- **Source/parameter boundary:** CSR deliberately ships ABSENT; no current literal or plausible ratio is adopted.
- **History/reachability:** exact source, test and reachable-history searches found no POR CSR implementation.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion plus an explicit user/cited CSR input are required.
- **Next action:** if included, implement the four named relations behind an absent-by-default required input and prove refusal plus both-sided clamping behavior.

## SB-POR-013 - Explicit shale-correction convention

- **Specified contract:** each method selects exactly one named `NORMALISED` or `SUBTRACTIVE` convention, never mixes them, and persists the choice in the run header; T20 observes arithmetic and provenance.
- **Current implementation:** no `SHALE_CONV` option or run field exists. Density/D-N and sonic embed different arithmetic without naming it, so users cannot audit or reproduce the convention.
- **Qualifying acceptance tests:** none; T20 is not executable. Test class `MISSING`.
- **Supporting tests:** local arithmetic tests exercise current embedded formulas but do not select or persist a convention.
- **Manual evidence:** porosity 0/33; workflow 0/23; processing-history 0/7.
- **Source/parameter boundary:** convention is categorical and must be explicit; none is inferred from existing formulas.
- **History/reachability:** no current or reachable `SHALE_CONV` implementation was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires method-specific convention support plus run custody.
- **Next action:** expose only chapter-admissible conventions per method, forbid mixed branches, and prove each selection is visible after save/reload.

## SB-POR-014 - Honest sonic method identities

- **Specified contract:** method names must match the published computation; field-observed behavior must expose its coefficient, and an RHG label may be used only for a true sourced RHG rendering; T28 is the direct discriminator.
- **Current implementation:** the sonic selector offers `WYLLIE` and `RHG`; `RHG` computes one fixed approximation, not the published three-segment RHG80 rendering. No field-observed coefficient is exposed. One user-visible description also contains the separately recorded prohibited parenthetical.
- **Qualifying acceptance tests:** none for the specified identity contract. Test class `CHARACTERIZATION`: `phi_son_wyllie_cp_opt_in_only_scales_wyllie` pins the current fixed branch and its immunity to `OPT_CP`, not published RHG correctness.
- **Supporting tests:** that characterization passed exactly once.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** SP-013 and SP-015 distinguish the current approximation, proposed field-observed form and RHG80 source. `DEC-017` chooses the original three-segment RHG80 product route; no coefficient, vendor-specific rendering or scan typography is inferred.
- **History/reachability:** the misnamed branch is integrated; no true RHG80 or `FIELD_OBSERVED` path was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the rename-versus-RHG80 choice is closed by `DEC-017`. Exact paper typography must be verified before code; SB-POR-020's vendor-rendering choice and SP-014's wording cleanup remain open.
- **Next action:** verify the original scan, implement the three-segment RHG80 path without borrowing a vendor closed form, keep any field-observed route separately sourced, and prove label, formula, parameter and run provenance together.

## SB-POR-015 - Method-correct sonic shale treatment

- **Specified contract:** non-Wyllie methods first form shale-reduced slowness, floor it at matrix slowness, evaluate the method and rescale; Wyllie remains on its cited subtractive control path. T18, T18b and T21 distinguish the paths.
- **Current implementation:** `phi_son` feeds raw DT into both current branches, then applies a shared effective-porosity subtraction. No shale-reduced/matrix-floored non-Wyllie path or iterative field-observed seed exists.
- **Qualifying acceptance tests:** none; the three specified discriminators are not executable. Test class `MISSING`.
- **Supporting tests:** the current sonic option test passed exactly once, but it asserts current embedded behavior rather than the chapter's split convention.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** SP-015 supplies newer primary-source direction; it does not choose a method or coefficient.
- **History/reachability:** no compliant non-Wyllie reduction/rescale branch was found in current source, tests or reachable history.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the RHG80 identity choice is closed by `DEC-017`; the method-specific shale-convention implementation and original-scan verification remain open.
- **Next action:** separate Wyllie and non-Wyllie preprocessing explicitly and prove raw-DT and stale shared-subtraction controls fail.

## SB-POR-016 - Cited lithology-selected matrix transit time

- **Specified contract:** `DT_MA` is selected from the cited lithology family, carries its source, and has no lithology-agnostic global default; T10 proves distinct choices and absence of a default.
- **Current implementation:** sonic exposes one freely editable `DT_MA` with a numeric default and no lithology selector, source topic or matched mineral identity.
- **Qualifying acceptance tests:** none; T10 is not executable. Test class `MISSING`.
- **Supporting tests:** sonic arithmetic uses the current literal but does not source or select it.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** the chapter's cited family may be offered; this pass does not choose a lithology or carry the existing literal forward as authority.
- **History/reachability:** no lithology-selected `DT_MA` family was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires typed lithology/mineral selection and source custody.
- **Next action:** replace the unqualified default with cited named choices plus explicit user input, then prove no selection is silently made.

## SB-POR-017 - Compaction correction cannot inflate porosity

- **Specified contract:** Wyllie compaction correction may only reduce porosity; `Cp < 1` must refuse or hard-flag and T29 must pin the invalid and valid sides.
- **Current implementation:** `OPT_CP=ON` computes `Cp = DT_SH/100` and accepts any positive `DT_SH`; `DT_SH=90` gives `Cp=0.9` and silently increases porosity.
- **Qualifying acceptance tests:** none for specified behavior. Test class `CHARACTERIZATION`: `phi_son_wyllie_cp_opt_in_only_scales_wyllie` explicitly expects the current inflation and therefore records the divergence rather than proving correctness.
- **Supporting tests:** the characterization passed exactly once and also proves the option does not affect the current `RHG` branch.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** SP-012 records the direction, SP-015 the source and `DEC-012` selects refusal; no threshold or parameter is introduced by this adjudication.
- **History/reachability:** the silent inflation path and defending test are integrated.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-012` closes the product choice in favor of hard refusal; implementation and proof remain open.
- **Next action:** refuse `Cp < 1` before evaluation and any computed write, then retain a `Cp >= 1` control proving the accepted branch cannot inflate porosity.

## SB-POR-018 - Floor shale-corrected slowness before every transform

- **Specified contract:** every shale-corrected sonic input is floored at its selected `DT_MA` before a method evaluates it; T18 and T21 distinguish this from an output clamp.
- **Current implementation:** no shale-reduced input path exists. Current `[0,1]` output limiting occurs after raw-DT transforms and cannot satisfy the input-domain floor.
- **Qualifying acceptance tests:** none; T18/T21 are not executable. Test class `MISSING`.
- **Supporting tests:** current sonic arithmetic provides no qualifying control for the missing preprocessing stage.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** depends on a cited `DT_MA`; no floor endpoint is invented.
- **History/reachability:** no matrix-slowness preprocessing floor was found in current or reachable source.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-015 and SB-POR-016.
- **Next action:** implement a named pre-transform floor and prove a sub-matrix corrected input differs from an after-the-fact output clamp.

## SB-POR-019 - Matched sonic endpoint and exponent pair

- **Specified contract:** a non-Wyllie matrix endpoint and fitted exponent are selected together as one cited mineral pair and persisted together.
- **Current implementation:** no matched-pair object, exponent parameter, source topic or paired provenance exists in the sonic module.
- **Qualifying acceptance tests:** none; no source-backed pair inventory is executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** only chapter-cited matched pairs may ship; independent editable numbers or a guessed exponent are forbidden.
- **History/reachability:** current, test and reachable-history searches found no matched sonic pair.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** product must choose whether a matched-pair method enters the pilot; the pair must remain source-bound.
- **Next action:** if included, model the pair as one immutable sourced choice and test that cross-pair combinations refuse.

## SB-POR-020 - Exactly one sourced RHG rendering is authoritative

- **Specified contract:** one published RHG rendering is the authoritative default and any other rendering is a labelled comparison curve; T28 plus the primary source pin identity and output custody.
- **Current implementation:** the only `RHG` selector is the fixed non-RHG80 approximation. There is no true sourced rendering, comparison typing or authoritative-source record.
- **Qualifying acceptance tests:** none; T28 is not executable for the specified method. Test class `MISSING`.
- **Supporting tests:** the sonic characterization proves only current approximation behavior.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18.
- **Source/parameter boundary:** `DEC-017` selects the original three-segment RHG80 product route, while SP-015's source still does not authorize a vendor rendering, an unverified transcription or an inferred default.
- **History/reachability:** no published RHG implementation or comparison-curve mechanism was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** SP-013's method disposition is closed by `DEC-017`; SB-POR-020 still separately requires an explicit vendor-rendering disposition, and the original scan must be verified before RHG80 code.
- **Next action:** implement the verified original RHG80 route, keep vendor renderings distinctly labelled if later selected, and add a label/formula/provenance acceptance test that cannot pass on the current approximation.

## SB-POR-021 - Chart-free analytic neutron-density crossplot

- **Specified contract:** implement a chart-free analytic N-D crossplot as the primary N-D porosity method, following the Bateman & Konen (1977) family, so SandiBumi ships a real crossplot porosity without transcribing a single vendor chart value. The arithmetic average currently standing in for it costs 1.64-1.79 p.u.
- **Current implementation:** `phi_dn` still offers only the `AVERAGE` and `GAS_RMS` shortcuts, which SB-POR-001 correctly types as a comparison producer rather than an interpretation method, and which SB-POR-004 gave collision-safe `PHIE_DN_LIM`/`PHIT_DN_LIM` identities. No Bateman-Konen analytic evaluator exists. The Bateman-Konen salinity/Rw code in the saturation path is a different equation family and is not POR.
- **Qualifying acceptance tests:** none. Test class `MISSING`. A test built on the constants that are actually held would assert exactly the adoption ESC-POR-8 forbids.
- **Supporting tests:** the SB-POR-001 envelope proof pins that D-N is typed as a comparison producer, so the shortcut is not currently masquerading as the analytic method.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** section 5.6 ships the nine crossplot constants (`2.71 / 4.00 / 0.7 / -5 / -0.16 / -2.06 / -1.17 / -16 / -0.4`) as `NON-ADOPTABLE - cited for verification`: they are **Geolog's rendering** of the method in `phi_dnbk.lls` `DN_XPLOT` (tier T1), not the paper's. ESC-POR-8 states the position exactly - the 1977 paper is not held locally, so the constants are ABSENT and the method cannot ship as a default. Adopting a vendor's fitted constants for a published method without the publication is the "carried over from a neighbouring vendor" failure the parameter discipline exists to prevent, and it would also carry the same provenance exposure recorded for the vendor chart payloads under SB-PLT-024.
- **History/reachability:** no analytic evaluator is present or reachable.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `BLOCKED-SOURCE`, ESC-POR-8. This is a source-intake item, not a design decision: no product choice unblocks it.
- **Next action:** obtain Bateman, R.M. and Konen, C.E., *The Log Analyst*, Nov-Dec 1977, or an equivalent admissible primary source for the analytic form and its constants; then implement the evaluator and compare it independently against both the chart and shortcut discriminators. Meanwhile the pilot keeps the shortcuts explicitly labelled as comparisons and excluded from pay, per PILOT_SCOPE item 6.

## SB-POR-022 - SandiBumi-owned gated chart pipeline

- **Specified contract:** chart-based N-D validation must use a separately gated SandiBumi-owned digitization pipeline, with admissible custody and no protected chart data in ordinary source.
- **Current implementation:** `nphimat` contains SandiBumi-owned monotone digitized tables and conversions with tests. They are neutron-matrix conversion tables, not a POR N-D chart-validation pipeline wired to `phi_dn` or the analytic method.
- **Qualifying acceptance tests:** none for the complete POR gate/custody/validation contract. Test class `MISSING`.
- **Supporting tests:** `nphimat_reproduces_por5_worked_example`, `nphimat_round_trip_ss_back_to_ls`, `nphimat_dolomite_input_inverts_through_the_chart`, `nphimat_thermal_dolomite_bow_and_salinity_scope`, and `nphimat_tables_are_strictly_monotone` passed exactly once.
- **Manual evidence:** porosity 0/33; crossplot 6/13.
- **Source/parameter boundary:** ESC-1/2/3/5/7 and ESC-POR-8 remain; protected vendor charts are not opened or copied.
- **History/reachability:** owned `nphimat` support is integrated; no POR analytic-versus-chart gate was found.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot inclusion and admissible chart-validation custody must be decided after ESC-POR-8.
- **Next action:** keep the owned conversion tables separate; if included, add an explicitly gated POR validation adapter with independent source and no production dependence on protected material.

## SB-POR-023 - Average and RMS are comparison curves only

- **Specified contract:** arithmetic average and gas RMS remain visibly labelled comparison curves, never authoritative POR methods or pay defaults; T36 pins method registration and downstream exclusion.
- **Current implementation:** `phi_dn` exposes `AVERAGE` and `GAS_RMS` as the only selectable methods and writes ordinary POR outputs. The documentation calls them analytic shortcuts rather than comparison-only quantities; no pay-exclusion type exists.
- **Qualifying acceptance tests:** none for the specified comparison/pay contract. Test class `CHARACTERIZATION`: `phi_dn_crossplot_shale_reduction_and_branches` pins the current selectable shortcuts and their arithmetic.
- **Supporting tests:** the characterization passed exactly once; it does not prove comparison typing or exclusion.
- **Manual evidence:** porosity 0/33; crossplot 6/13; no pay-selection evidence.
- **Source/parameter boundary:** the current shortcut formulas do not become authority for the absent analytic method.
- **History/reachability:** current shortcut registration is integrated; no POR comparison family or pay guard was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-021 and SB-POR-057.
- **Next action:** reclassify both shortcuts as method-qualified comparison outputs, make them ineligible for pay defaults, and prove the authoritative registry rejects them.

## SB-POR-024 - Neutron matrix-basis refusal and provenance

- **Specified contract:** N-D porosity must know the neutron matrix basis, convert it through an admissible path when requested, refuse missing/wrong basis, and record input/output basis.
- **Current implementation:** `nphimat` can convert named matrix bases, but `phi_dn` accepts NPHI without inspecting or requiring basis metadata. No wrong-basis refusal or per-output basis provenance is emitted.
- **Qualifying acceptance tests:** none; no accepted-versus-refused POR basis test exists. Test class `MISSING`.
- **Supporting tests:** the five `nphimat` tests passed exactly once and prove conversion arithmetic only.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18; workflow 0/23.
- **Source/parameter boundary:** conversion must use the owned/cited basis tables; no basis is guessed from mnemonic.
- **History/reachability:** the converter is integrated; no POR wiring or refusal was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires neutron-basis metadata at import/storage plus a typed POR input guard.
- **Next action:** carry basis through curve metadata, require it at `phi_dn`, and test explicit compatible conversion against absent and wrong-basis refusal controls.

## SB-POR-025 - Salinity interpolation for neutron response

- **Specified contract:** use the cited fresh/salt endpoints with continuous fluid-density/salinity interpolation and retain the resolved condition in provenance.
- **Current implementation:** `nphimat` offers binary fresh/salt choices and matrix conversions. `phi_dn` neither consumes the choice nor interpolates by fluid density, and the run record does not retain a resolved salinity response.
- **Qualifying acceptance tests:** none for interpolation and POR custody. Test class `MISSING`.
- **Supporting tests:** `nphimat_thermal_dolomite_bow_and_salinity_scope` passed exactly once for the current binary scope.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** only cited endpoints may be used; no intermediate relation is invented beyond the chapter's specified interpolation.
- **History/reachability:** binary conversion support is integrated; no POR salinity interpolation was found.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** decide pilot inclusion and define typed fluid-condition input without creating a default.
- **Next action:** if included, add the chapter-specified interpolation and prove both endpoints plus one independent interior arithmetic case and provenance round-trip.

## SB-POR-026 - Crossover conditioning is consumed by POR

- **Specified contract:** POR methods declare, consume and record the conditioning crossover decision rather than recomputing or ignoring it.
- **Current implementation:** `condflag` emits `XOVER` and related flags, but POR manifests do not declare those inputs and their bodies do not consume or persist them.
- **Qualifying acceptance tests:** none; T41 does not execute declaration-through-run custody. Test class `MISSING`.
- **Supporting tests:** `condflag_detects_coal_tight_and_crossover`, `condflag_washout_is_not_coal_and_xcond_option`, and the generic empty-flag refusal passed exactly once.
- **Manual evidence:** conditioning 0/27; porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no threshold is adopted; only the already-produced decision is under review.
- **History/reachability:** XOVER generation is integrated; no POR consumer was found.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires the common flag contract and a declared policy for what each POR method does with XOVER.
- **Next action:** add a typed required/optional conditioning input, record the branch taken, and prove both present and absent-policy paths.

## SB-POR-027 - Neutron-sonic crossplot

- **Specified contract:** provide the cited neutron-sonic crossplot method as a distinct POR method with typed inputs, limits, provenance and acceptance evidence.
- **Current implementation:** no neutron-sonic POR module, dispatch branch, UI option, command, test or history exists.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; crossplot 6/13, none for neutron-sonic POR.
- **Source/parameter boundary:** no source-derived parameter is inferred.
- **History/reachability:** current, test and reachable-history searches confirmed absence.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** explicitly held outside the Windows-first pilot.
- **Next action:** retain as a later-version candidate; do not create a placeholder method or default.

## SB-POR-028 - Hard bounds are cited parameters and binding is flagged

- **Specified contract:** every shale-reduction/crossplot clamp endpoint is a cited, visible parameter and a binding clamp raises the POR flag/log; T12 covers both bound and unbound samples.
- **Current implementation:** `phi_dn` hard-clamps RHOB and NPHI to four literals before arithmetic. The values are not visible/source-bound parameters and no POR clamp flag or run message is emitted.
- **Qualifying acceptance tests:** none for the specified parameter/flag contract. Test class `CHARACTERIZATION`: `phi_dn_crossplot_shale_reduction_and_branches` exercises the current bounded implementation without proving cited custody or observable binding.
- **Supporting tests:** the characterization passed exactly once.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** current literals are not promoted as citations; only chapter-sourced endpoints may become parameters.
- **History/reachability:** hard-coded bounds are integrated; no source topic or POR flag was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-003 and SB-POR-007.
- **Next action:** expose only the cited endpoints with source/tier, emit binding state, and test just-inside versus just-outside samples from both input axes.

## SB-POR-029 - Apparent hydrocarbon electron density

- **Specified contract:** compute apparent hydrocarbon electron density from the sourced model, stay within its cited envelope, and expose it as an auditable intermediate; T01 supplies the discriminator.
- **Current implementation:** no POR HC electron-density model or intermediate exists. The separate `gascorr` density-log correction is not this quantity.
- **Qualifying acceptance tests:** none; T01 is not executable. Test class `MISSING`.
- **Supporting tests:** `gascorr` tests are explicitly supporting-only because they solve a different correction.
- **Manual evidence:** porosity 0/33; conditioning 0/27.
- **Source/parameter boundary:** no HC endpoint or envelope is inferred from `gascorr` fixtures.
- **History/reachability:** exact current, test and reachable-history searches found no POR HC electron-density path.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** requires the chapter-sourced HC model and typed fluid inputs.
- **Next action:** implement the auditable intermediate only from admissible sources and prove one in-envelope value plus refusal/flag at the model boundary.

## SB-POR-030 - Hydrocarbon hydrogen-index model and compatibility isolation

- **Specified contract:** compute sourced gas/oil HI envelopes and isolate any divergent legacy neutron behavior behind explicit compatibility metadata; T02 and T03 cover correct and compatibility paths.
- **Current implementation:** no POR HC HI model, gas/oil selector, `HI_HC` custody or compatibility branch exists. Unrelated retired/history references cannot supply it.
- **Qualifying acceptance tests:** none; T02/T03 are not executable. Test class `MISSING`.
- **Supporting tests:** none; current `gascorr` and SSC gas handling are different contracts.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** no gas/oil HI value or envelope is copied from retired code or fixtures.
- **History/reachability:** live and reachable searches found no POR HI implementation.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** admissible source and compatibility policy must both be explicit.
- **Next action:** implement the sourced path first; only add a separately named compatibility path if a preserved delivery requires it, with warning and provenance.

## SB-POR-031 - Explicit A/B hydrocarbon architecture

- **Specified contract:** the POR HC chain exposes the specified A/B architecture, separately named forward/correction directions and auditable intermediates; T01, T02 and T07 jointly constrain it.
- **Current implementation:** the complete HC chain is absent, including A/B factors, typed intermediate records and directional APIs.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** no current test reaches this architecture.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no coefficient is inferred.
- **History/reachability:** no current or reachable A/B POR implementation was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-POR-029/030 source custody and SB-POR-005 API direction.
- **Next action:** build a typed intermediate model only after those prerequisites close, then test each factor independently before end-to-end round-trip.

## SB-POR-032 - RHO_MF and Pmf are governed parameters

- **Specified contract:** `RHO_MF` and `Pmf` are distinct source-bound parameters with units, ranges, visible custody and run provenance.
- **Current implementation:** neither POR parameter exists in manifests, source topics, dialog, run metadata or HC code.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** both remain ABSENT unless supplied by the chapter's admissible source/user path; no plausible values are chosen.
- **History/reachability:** exact searches found no live or reachable POR parameters.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** parameter custody must close before the HC chain can be configured.
- **Next action:** add typed, source-labelled, absent-by-default inputs only with the model that consumes them and prove complete run persistence.

## SB-POR-033 - Hydrocarbon validity guards

- **Specified contract:** every reachable HC form enforces its cited low-density and physical validity envelope and refuses or flags outside-domain inputs; T17 pins inside and outside controls.
- **Current implementation:** no POR HC forms or guards exist. `gascorr` guards are for a separate density-log correction and cannot close the universal POR contract.
- **Qualifying acceptance tests:** none; T17 is not executable. Test class `MISSING`.
- **Supporting tests:** `gascorr_guards_stay_missing_or_error` passed exactly once for the separate module.
- **Manual evidence:** porosity 0/33; conditioning 0/27.
- **Source/parameter boundary:** validity endpoints must remain sourced; none is imported from a test fixture.
- **History/reachability:** no POR HC validity implementation was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the exact admitted HC models and their cited domains.
- **Next action:** attach guards to each source-bound model, with distinguishable missing-input, outside-domain and computed states.

## SB-POR-034 - HC model selection, provenance and intermediates

- **Specified contract:** explicit model selection records compatibility warnings, excavation interaction, validity, all intermediates and the exact branch used; T03, T13 and T17 observe those surfaces.
- **Current implementation:** no POR HC selector, compatibility mode, excavation-suppression state, intermediate output set or provenance record exists.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** generic module options and run metadata do not contain this model state.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** no model is chosen by this adjudication.
- **History/reachability:** current, test and reachable-history searches confirmed absence.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-POR-029 through 033 and the excavation decision.
- **Next action:** design one explicit model-state record and require it before evaluation; prove every branch and warning survives save/reload.

## SB-POR-035 - Flushed-zone saturation exponent has no default

- **Specified contract:** conflicting cited `Sxo` exponents remain visible, user/source selection is required, and no default ships; T25 asserts refusal and explicit selections.
- **Current implementation:** the POR HC chain and exponent parameter are absent; therefore there is no accidental current default, but there is also no required refusal or visible conflict.
- **Qualifying acceptance tests:** none; T25 is not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** the conflict is preserved and no midpoint, neighbor or familiar exponent is selected.
- **History/reachability:** no live/reachable POR `Sxo` exponent path was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** an explicit source/user choice is required when the dependent model is implemented.
- **Next action:** expose the competing cited choices without a default and refuse evaluation until one is selected and recorded.

## SB-POR-036 - Explicit force-wet branch

- **Specified contract:** provide the specified force-wet control as an explicit, recorded branch rather than silently forcing or assuming water response.
- **Current implementation:** no POR HC chain or force-wet option exists.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** no POR test reaches this branch.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** no fluid response is assumed.
- **History/reachability:** source, tests and reachable history contain no POR force-wet branch.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion and interaction with the admitted HC model require product definition.
- **Next action:** if included, make the branch opt-in, visible in outputs/provenance, and test it against an unforced control.

## SB-POR-037 - Physical ceiling on hydrocarbon response

- **Specified contract:** enforce the cited physical HI/response ceiling with visible binding state across every HC form.
- **Current implementation:** no POR HC response model or ceiling exists.
- **Qualifying acceptance tests:** none; the physical-bound limbs of T02/T17 are not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** the ceiling must be cited; no plausible physical maximum is invented.
- **History/reachability:** no current or reachable POR ceiling implementation was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on source custody for each admitted HC form.
- **Next action:** if the HC chain enters pilot scope, implement its cited bound with a flag and one below/above-bound discriminator.

## SB-POR-038 - Keep gascorr distinct and prevent double correction

- **Specified contract:** distinguish density-log gas correction from POR HC correction, record whether correction was already applied, refuse double application, and return missing plus a flag on non-convergence.
- **Current implementation:** `gascorr` is a separate iterative density-log module with guards and non-convergence discipline. Its documentation distinguishes the method and cautions against feeding corrected RHOB into `phi_dn`, but no machine-readable already-applied provenance or POR double-correction refusal exists; the POR HC chain is absent.
- **Qualifying acceptance tests:** none for the complete distinction/provenance/refusal contract. Test class `MISSING`.
- **Supporting tests:** `gascorr_converges_on_gas_sand_and_skips_water`, `gascorr_flag_gate_and_missing_inputs`, and `gascorr_guards_stay_missing_or_error` passed exactly once.
- **Manual evidence:** conditioning 0/27; porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** current solver literals are not promoted to POR defaults.
- **History/reachability:** `gascorr` is integrated; no correction-state provenance or POR double-application guard was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires typed correction-state custody before a POR HC chain is introduced.
- **Next action:** add an explicit correction-state field and observable refusal, then prove raw and already-corrected inputs take distinct paths and cap-hit remains missing plus flagged.

## SB-POR-039 - Additive excavation family only

- **Specified contract:** implement the cited additive excavation family with lithology-sensitive exponent behavior, reproduce T04/T05, and keep the multiplied rendering unreachable under T06.
- **Current implementation:** no POR excavation function, module, command, caller, test or history exists. Absence of the multiplied form does not substitute for presence of the required additive one.
- **Qualifying acceptance tests:** none; T04-T06 are not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** only cited lithology relations may be used; no exponent is inferred.
- **History/reachability:** current, test and reachable-history searches confirmed the full excavation family is absent.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** admissible source custody and the tool gate in SB-POR-041 must close.
- **Next action:** implement the additive direction from cited relations only, prove lithology discrimination, and add a negative inventory guard against the multiplied form.

## SB-POR-040 - Separately named excavation directions

- **Specified contract:** expose excavation forward modeling and correction as separately named typed directions, each recorded and round-trippable under T07.
- **Current implementation:** neither excavation direction exists; there is no public API or provenance field to distinguish them.
- **Qualifying acceptance tests:** none; T07 is not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no sign convention is guessed.
- **History/reachability:** no current or reachable directional excavation API was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on SB-POR-039 and a pilot decision for forward-model exposure.
- **Next action:** if both directions are admitted, define separately named APIs and independently derived expectations before a round-trip test.

## SB-POR-041 - Resolved tool applicability gate

- **Specified contract:** excavation is gated by SandiBumi's resolved, case-stable tool identity; known applicable/non-applicable tools route deterministically and an unresolved token emits a diagnostic rather than a guess; T14/T14b cover both.
- **Current implementation:** no excavation tool gate or resolved-tool integration exists. Generic mnemonic text cannot establish tool identity.
- **Qualifying acceptance tests:** none; T14/T14b are not executable. Test class `MISSING`.
- **Supporting tests:** none for POR tool identity.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18.
- **Source/parameter boundary:** ESC-7 remains; protected vendor material is not opened and unresolved tokens are not mapped by intuition.
- **History/reachability:** source, tests and reachable history contain no POR excavation tool gate.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** requires an admissible SandiBumi tool register and explicit unknown-token behavior.
- **Next action:** add the register-backed gate before exposing excavation, with positive applicable, negative non-applicable and unresolved diagnostics.

## SB-POR-042 - Excavation constants remain source-gated

- **Specified contract:** any excavation constants must come from admissible primary custody, never reverse engineering or protected binaries.
- **Current implementation:** no excavation constants or evaluator ship, so no prohibited constant is embedded; the requested capability is also absent.
- **Qualifying acceptance tests:** none; source custody is not executable by itself. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** none.
- **Source/parameter boundary:** ESC-1 and related custody escalations remain open; no constant is adopted.
- **History/reachability:** no live or reachable POR excavation constants were found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** primary-source evidence is the explicit blocker.
- **Next action:** retain the gap for a later version until admissible custody exists; do not create guessed placeholders.

## SB-POR-043 - High-shale threshold is a governed parameter

- **Specified contract:** the high-shale branch threshold is cited, visible, configurable and recorded, with branch state observable.
- **Current implementation:** density and D-N use a hard-coded `VSH >= 0.95` branch. It is not an argument, has no source topic/run field, and raises no POR branch flag.
- **Qualifying acceptance tests:** none for the specified parameter/provenance contract. Test class `CHARACTERIZATION`: `phi_den_shale_branch_limits_and_missing` and the density/D-N branch checks pin current threshold behavior.
- **Supporting tests:** those characterizations passed exactly once and prove only today's literal branch.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** the current literal is not promoted as its own citation; any shipped threshold must follow the chapter source.
- **History/reachability:** the hard-coded branch is integrated; no governed argument or flag was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on parameter-source custody and the common POR flag.
- **Next action:** expose the cited threshold, record it, and test below/at/above samples plus branch provenance.

## SB-POR-044 - Optional smooth roll-off with no defaults

- **Specified contract:** offer the specified smooth high-shale roll-off as an explicit option only when its three parameters are provided; ship no roll-off defaults and retain the hard-branch alternative.
- **Current implementation:** no roll-off option or parameter set exists; only the hard-coded step branch ships.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** hard-branch characterizations do not prove a roll-off.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** the bundled three-parameter set remains ABSENT; no plausible transition values are invented.
- **History/reachability:** no current or reachable POR roll-off implementation was found.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** product must decide pilot inclusion; any use requires explicit user/cited parameters.
- **Next action:** if included, implement it as opt-in with all parameters required and test both hard-branch and smooth paths without a default selection.

## SB-POR-045 - PHIE floor is configuration, not a compile-time authority

- **Specified contract:** conflicting cited floor values remain visible, no chapter default ships, the selected floor is recorded configuration, and T40 distinguishes it from a compile-time constant.
- **Current implementation:** `PHIE_FLOOR` is compile-time `0.001` and density/D-N always use it. A later direct product record also chooses `0.001`, while the immutable chapter deliberately withholds a default because it records conflicting values.
- **Qualifying acceptance tests:** none for the specified no-default/configuration contract. Test class `CHARACTERIZATION`: `a_negative_density_porosity_is_floored_but_stays_visible_in_the_unlimited_twin` pins the current constant and only sanity-checks that it is below `0.01`.
- **Supporting tests:** the characterization passed exactly once for density and D-N.
- **Manual evidence:** porosity 0/33; workflow 0/23; processing-history 0/7.
- **Source/parameter boundary:** current code, chapter conflict and later direct product decision remain three separate facts; this lane does not adjudicate precedence.
- **History/reachability:** the compile-time floor and later decision record are reachable.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** Jauhar must explicitly decide whether the later direct decision supersedes the chapter's no-default contract and how that decision is recorded.
- **Next action:** after adjudication, move the authorized choice into source-labelled run configuration and prove two explicit values produce distinct limited outputs while unlimited data stays unchanged.

## SB-POR-046 - VSILT is warning-bearing and non-authoritative

- **Specified contract:** any VSILT output must carry the specified warning, provenance and non-authoritative status so it cannot be mistaken for a sourced POR result.
- **Current implementation:** SSC exposes `VSILT`; no required warning, comparison typing or per-output provenance accompanies it.
- **Qualifying acceptance tests:** none for warning/provenance custody. Test class `MISSING`.
- **Supporting tests:** SSC closure tests passed exactly once and prove numerical component closure, not the warning contract.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** no additional silt endpoint is adopted.
- **History/reachability:** VSILT is integrated; the required warning surface was not found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires a typed warning/provenance surface shared by output catalog, dialog and export.
- **Next action:** attach the specified warning and non-authoritative classification, then prove it survives save/reload and remains ineligible as an authoritative POR curve.

## SB-POR-047 - BADHOLE is declared, consumed and recorded

- **Specified contract:** every applicable POR method declares the bad-hole flag input, consumes it according to policy and records the branch; T41 checks detector-through-POR custody.
- **Current implementation:** `badhole` emits `BADHOLE`, but POR manifests neither declare nor consume it. Generic masking can use an arbitrary flag but does not record a POR bad-hole decision.
- **Qualifying acceptance tests:** none; T41 is not executable end to end. Test class `MISSING`.
- **Supporting tests:** `badhole_flags_washout_and_drho` and the generic empty-flag refusal passed exactly once.
- **Manual evidence:** conditioning 0/27; porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** detector thresholds are not changed or reinterpreted.
- **History/reachability:** BADHOLE generation is integrated; no POR consumption path was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires a declared POR flag policy and provenance schema.
- **Next action:** wire the typed flag into each applicable method and prove clean, flagged and missing-flag behavior with recorded reasons.

## SB-POR-048 - Conditioning flags are declared, consumed and recorded

- **Specified contract:** POR methods consume the applicable coal, tight, crossover, shoulder and conditioning flags through declared inputs and retain the selected branch/reason.
- **Current implementation:** `condflag` emits `COAL`, `TIGHT`, `XOVER`, `SHOULDER` and `COND`; POR manifests declare none and bodies consume none.
- **Qualifying acceptance tests:** none for declaration-through-provenance. Test class `MISSING`.
- **Supporting tests:** the three `condflag` tests passed exactly once for detector behavior.
- **Manual evidence:** conditioning 0/27; porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** existing detector decisions are not weakened and no new thresholds are selected.
- **History/reachability:** detector outputs are integrated; no POR wiring was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the POR family needs an explicit per-flag policy, including whether a flag masks, selects a branch or only annotates.
- **Next action:** define those semantics without deleting existing guards, then prove each consumed flag and an unflagged control.

## SB-POR-049 - No hard-coded lithology kill

- **Specified contract:** POR must not contain an implicit lithology kill; any exclusion is explicit, source/user-owned, visible and recorded.
- **Current implementation:** targeted POR source inventory found no hard-coded lithology-kill branch, but there is no executable universal registry test or manual evidence proving every reachable method and future registration remains free of one.
- **Qualifying acceptance tests:** none; absence is not covered by an executable method-inventory assertion. Test class `MISSING`.
- **Supporting tests:** current POR branch tests did not encounter a lithology kill, which is weaker than a universal negative contract.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no lithology threshold or exclusion is introduced.
- **History/reachability:** current and reachable searches found no such branch.
- **Verdict:** `PRESENT-UNVERIFIED`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** needs an inventory-based regression test and observable explicit-exclusion policy if exclusions are later added.
- **Next action:** add a registry-level assertion that every POR lithology exclusion is declared configuration with provenance and that no hidden kill branch exists.

## SB-POR-050 - Iterative solver discipline

- **Specified contract:** every iterative POR solver exposes sourced/configured tolerance and cap, uses an absolute convergence inequality, returns no partial iterate on cap hit, flags non-convergence and records settings; T16 and T24 cover the full discipline.
- **Current implementation:** `gascorr` demonstrates missing/error guard and no partial result on failure, but it is not the POR HC/N-D solver and hard-codes tolerance `1e-4` and cap `20`. The required POR iterative branches do not exist.
- **Qualifying acceptance tests:** none for every POR solver and visible configuration. Test class `MISSING`.
- **Supporting tests:** all three `gascorr` tests passed exactly once and provide solver precedent only.
- **Manual evidence:** porosity 0/33; conditioning 0/27; workflow 0/23.
- **Source/parameter boundary:** current literals are not promoted as cited defaults.
- **History/reachability:** partial solver discipline is integrated in `gascorr`; no complete POR implementation was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** admitted POR solvers and source-owned configuration must be defined first.
- **Next action:** centralize the discipline for POR solvers, expose admissible settings, and prove convergence, invalid settings and cap-hit behavior independently.

## SB-POR-051 - Explicit solver precedence

- **Specified contract:** when more than one POR correction/solver applies, a single documented precedence order is selected, visible and recorded; ambiguous combinations do not execute silently.
- **Current implementation:** the relevant HC, analytic N-D and excavation solvers are absent, and no POR precedence model or run field exists.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** generic sequential workflow behavior is not solver precedence.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** no ordering is inferred from current module catalog order.
- **History/reachability:** no current or reachable POR solver-precedence implementation was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-014` makes the coupled porosity-`Sxo`/`Sw` solver mandatory and separate; its admissible unknowns and deterministic precedence remain source/configuration work, not an inferred ordering.
- **Next action:** specify one source-faithful precedence graph with forbidden combinations, then make it part of the validated iterative run configuration.

## SB-POR-052 - Invalid POR configurations refuse observably

- **Specified contract:** contradictory or incomplete POR configurations refuse before evaluation with an actionable user-facing error and no computed write.
- **Current implementation:** generic argument parsing and some module-local guards exist, but there is no POR-wide configuration model covering convention, basis, correction state, solver precedence, absent parameters and method compatibility.
- **Qualifying acceptance tests:** none for both refusal and zero-write behavior. Test class `MISSING`.
- **Supporting tests:** local `gascorr` errors and generic output-name refusal prove narrower guards only.
- **Manual evidence:** porosity 0/33; workflow 0/23; generic-curve-store 0/18.
- **Source/parameter boundary:** an absent source-bound value must remain a refusal, never be auto-filled.
- **History/reachability:** no complete POR configuration validator was found.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** `DEC-014` makes the iterative path mandatory; it still depends on the typed POR configuration schema and a source-faithful solver precedence.
- **Next action:** validate the full resolved request before calculation/write and test wrong-family, missing-source, conflicting-method and invalid-iterative controls.

## SB-POR-053 - Canonical neutron-sonic shale porosity

- **Specified contract:** implement the chapter's canonical bounded neutron-sonic shale term and keep the documented defective form unreachable; T22 pins a correct case and a negative inventory control.
- **Current implementation:** no neutron-sonic POR method or canonical shale-term helper exists. The defective form is not found either, but absence of a bad form cannot substitute for the required good form.
- **Qualifying acceptance tests:** none; T22 is not executable. Test class `MISSING`.
- **Supporting tests:** none.
- **Manual evidence:** porosity 0/33; crossplot 6/13, none for neutron-sonic POR.
- **Source/parameter boundary:** the chapter equation is the only admissible definition; no neighboring form is substituted.
- **History/reachability:** current, test and reachable-history searches found neither the required method nor the forbidden form.
- **Verdict:** `ABSENT`; `UNDECIDED`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on whether SB-POR-027 enters pilot scope.
- **Next action:** if included, implement the canonical term and pair a sourced numeric fixture with an explicit forbidden-form inventory guard.

## SB-POR-054 - Canonical correction sign and algebraic identity

- **Specified contract:** every POR correction uses one named canonical sign convention and has an independently derived identity test that distinguishes forward from inverse/correction direction.
- **Current implementation:** local formulas have embedded signs, but there is no POR-wide sign type, named convention, directional API or independent identity proof.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** local arithmetic tests can show current results but are circular for a universal sign identity.
- **Manual evidence:** porosity 0/33; processing-history 0/7.
- **Source/parameter boundary:** no sign is inferred from a current implementation result.
- **History/reachability:** current and reachable searches found no canonical POR sign contract.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on separately named directions from SB-POR-005/SB-POR-040.
- **Next action:** define the sign convention at the typed API boundary and derive the test expectation independently from the chapter equation.

## SB-POR-055 - Source, tier and default discipline for all parameters

- **Specified contract:** every one of the 74 parameter rows is represented with source/tier, unit and default state; ABSENT stays absent, NON-ADOPTABLE stays verification-only, and every resolved choice is visible and persisted.
- **Current implementation:** POR manifests contain many numeric literals/defaults but no POR source topics or evidence tiers. Generic dialog support exists. Several chapter-absent capabilities have no parameters, and current hard-coded bounds/floor cannot express the chapter's custody states.
- **Qualifying acceptance tests:** none; no complete 74-row source/default inventory exists. Test class `MISSING`.
- **Supporting tests:** generic source-panel and manifest-shape tests prove infrastructure only.
- **Manual evidence:** porosity 0/33; workflow 0/23; processing-history 0/7.
- **Source/parameter boundary:** mechanically 15 rows contain `ABSENT` and 8 `NON-ADOPTABLE`, while chapter prose says 18 ABSENT; the mismatch is preserved and no row is normalized by guess.
- **History/reachability:** generic `sources_topic` is reachable; no complete POR registry was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires row-by-row custody plus explicit resolution of PHIE-floor precedence; ESC-POR-8 remains open.
- **Next action:** create a generated POR parameter inventory from admissible chapter rows and fail its gate if any live argument lacks source/tier/default-state or any absent value acquires a default.

## SB-POR-056 - Canonical porosity and input units

- **Specified contract:** canonical POR fraction, density and slowness units flow through import, storage, evaluation, output metadata, display and export; equivalent units yield invariant T09 results.
- **Current implementation:** import and curve infrastructure canonicalize relevant NPHI, RHOB and DT units. Computed PHIE/PHIT outputs have no POR family metadata, so canonical POR units cannot be resolved reliably by catalog/export.
- **Qualifying acceptance tests:** none for end-to-end POR unit invariance and export metadata. Test class `MISSING`.
- **Supporting tests:** `recognised_length_and_slowness_bridges_convert_only_within_their_quantity_kind` and `families_resolve_common_mnemonics` passed exactly once; neither covers POR outputs.
- **Manual evidence:** generic-curve-store 0/18; las-export 0/2; porosity 0/33.
- **Source/parameter boundary:** unit conversions are type-defined; no petrophysical value is introduced.
- **History/reachability:** input bridges are integrated; POR family/output metadata is absent.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-004 family typing.
- **Next action:** define canonical POR fraction metadata and test equivalent NPHI/RHOB/DT inputs through compute, reload and LAS export.

## SB-POR-057 - Comparison curves cannot become pay curves

- **Specified contract:** comparison outputs are separately typed and visually/provenance-labelled, and downstream pay selection rejects them by default.
- **Current implementation:** `AVERAGE` and `GAS_RMS` are ordinary `phi_dn` method choices writing the same POR outputs as an authoritative method would. There is no comparison family, visual identity or pay-exclusion guard.
- **Qualifying acceptance tests:** none; T36 is not executable across registry, display and pay selection. Test class `MISSING`.
- **Supporting tests:** current `phi_dn` arithmetic characterizes the shortcuts but not their required exclusion.
- **Manual evidence:** porosity 0/33; crossplot 6/13; no pay-selection evidence.
- **Source/parameter boundary:** plausible shortcut output does not confer correctness authority.
- **History/reachability:** ordinary shortcut registration is integrated; no comparison/pay type was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on POR family typing and downstream selection policy.
- **Next action:** create a comparison-only output class, preserve method identity visually and in provenance, and make pay inputs reject it unless an explicit reviewed override exists.

## SB-POR-058 - No silently dead parameters

- **Specified contract:** every declared POR/related module argument is read on a reachable path or is explicitly inactive with a visible reason; T34 inventories both manifest and execution.
- **Current implementation:** `sspw` declares `NPHI_MAT`, `NPHI_SH` and `NPHI_FL` but its reachable body reads none of them. They are presented as active parameters without a visible inactive state.
- **Qualifying acceptance tests:** none; T34 is not executable. Test class `MISSING`.
- **Supporting tests:** `every_module_returns_the_output_keys_its_manifest_declares` checks outputs, not argument consumption; SSPW numeric tests do not vary the unused arguments.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** unused defaults are not treated as evidence-backed inputs.
- **History/reachability:** the declarations and missing reads are both integrated; exact history search found no alternate reachable consumption.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** decide whether each argument belongs in SSPW's actual method; removal or implementation must preserve saved-run compatibility.
- **Next action:** add an argument-use inventory gate, then either consume each parameter according to the cited method or mark/migrate it as explicitly inactive.

## SB-POR-059 - SSPW uses the sourced RMS gas midpoint

- **Specified contract:** SSPW and SSC use the independently sourced RMS gas-conditioning midpoint and reproduce T33 from a separate oracle.
- **Current implementation:** SSC uses RMS. SSPW uses a different weighted expression, so the two branches disagree for the same inputs.
- **Qualifying acceptance tests:** none; T33 is not executable as an independently sourced cross-module fixture. Test class `MISSING`.
- **Supporting tests:** SSC clean/shale tests and SSPW bound-water tests passed exactly once but do not compare the gas branch to an independent RMS calculation.
- **Manual evidence:** porosity 0/33; conditioning 0/27.
- **Source/parameter boundary:** the chapter's cited RMS definition is the oracle; neither implementation is used to validate the other.
- **History/reachability:** both divergent expressions are integrated at the accepted anchor.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** none beyond implementing the already sourced expression without changing unrelated SSPW behavior.
- **Next action:** replace only SSPW's gas midpoint with the cited RMS helper and add one independently calculated fixture that both modules must match.

## SB-POR-060 - Vendor parameter-set import

- **Specified contract:** import a vendor parameter set through explicit origin-qualified mappings, retain every source/default/conflict state, and refuse unknown or ambiguous fields.
- **Current implementation:** no POR vendor parameter-set parser, mapping, command, dialog or test exists.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Supporting tests:** generic saved-run parameter JSON is not an external parameter-set import.
- **Manual evidence:** porosity 0/33; workflow 0/23.
- **Source/parameter boundary:** vendor neighbors and protected material cannot supply missing defaults; ambiguous origin must remain a refusal.
- **History/reachability:** exact current, test and reachable-history searches confirmed absence.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** held outside the pilot and dependent on an admissible interchange contract.
- **Next action:** revisit in a later version; do not implement a heuristic mapper.

## SB-POR-061 - POR method audit report

- **Specified contract:** produce an auditable report covering selected method, source/tier, complete parameters, conventions, units, branches/flags, input/output identities and comparison status.
- **Current implementation:** generic run metadata and processing-history seams exist, but no POR audit report or complete resolved POR record exists.
- **Qualifying acceptance tests:** none; no report-schema or observable export test exists. Test class `MISSING`.
- **Supporting tests:** generic workflow/version tests prove only partial metadata storage.
- **Manual evidence:** processing-history 0/7; porosity 0/33.
- **Source/parameter boundary:** reports must expose gaps and ABSENT values, not fill them.
- **History/reachability:** no current or reachable POR audit-report implementation was found.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on closing the family, parameter-custody, provenance and flag schemas first.
- **Next action:** retain as later-version work; when prerequisites exist, generate the report from canonical run data rather than duplicating state.

## SB-POR-062 - Core comparison is a post-check and never auto-adjusts

- **Specified contract:** compare each POR method against core porosity with method-by-method bias/scatter and sample custody, while never automatically changing a parameter.
- **Current implementation:** generic plug QC, interpolation/statistics and `score_against_plugs` can compare caller-held samples to CPOR, but no POR method report or UI workflow wires computed method outputs to it. The calibration record explicitly preserves manual, post-check use and forbids automatic adjustment.
- **Qualifying acceptance tests:** none for the POR report and no-adjustment boundary. Test class `MISSING`.
- **Supporting tests:** `scoring_a_run_in_hand_matches_scoring_it_after_it_is_saved` passed exactly once for generic scoring persistence.
- **Manual evidence:** porosity 0/33; crossplot 6/13; no method-by-method core comparison scenario is checked.
- **Source/parameter boundary:** core observations are comparison evidence, never authority to auto-tune a petrophysical parameter.
- **History/reachability:** generic comparison infrastructure and the explicit no-auto-adjust record are reachable; no POR report was found.
- **Verdict:** `PARTIAL`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** held for a later version after POR method/provenance identity is stable.
- **Next action:** later, bind typed POR outputs to the existing core sample custody and report bias/scatter per method while asserting parameter records remain byte-identical.

## Test-intent classification summary

- All 41 real chapter test IDs were routed once: T01->029, T02->030, T03->030, T04-T06->039, T07->005, T08->021, T09->056, T10->016, T11->009, T12->028, T13->034, T14/T14b->041, T15->008, T16->050, T17->033, T18/T18b->015, T19->008, T20->013, T21->015, T22->053, T23->012, T24->050, T25->035, T28->014, T29->017, T30->008, T31/T32->004, T33->059, T34->058, T35->011, T36->023, T37->021, T38->003, T39->001, T40->045 and T41->047.
- Numeric T26 and T27 do not exist. They remain absent and are not invented or backfilled into the source-owned `owned_tests` fields.
- Six rows are `CHARACTERIZATION`: 014, 017, 023, 028, 043 and 045. Their current tests pin divergent shipped behavior and are not correctness oracles.
- The other 56 rows are `MISSING`. No whole-contract POR test qualifies as `CORRECTNESS`; narrow executable tests are recorded only as supporting evidence.
- Cross-support was also applied where the chapter makes contracts universal: especially 001-004, 008-011, 013-018, 021-028, 031-038, 047-050 and 053-059. A repeated test reference never counts as a second owned proof.

## Manual capability evidence

- `porosity`: 0/33 checked.
- `generic-curve-store`: 0/18 checked.
- `conditioning`: 0/27 checked.
- `workflow`: 0/23 checked.
- `las-export`: 0/2 checked.
- `processing-history`: 0/7 checked.
- `histogram`: 5/22 checked; no checked item proves POR parameter custody or method correctness.
- `crossplot`: 6/13 checked; no checked item proves the analytic N-D, neutron-sonic, HC, excavation, provenance or core-comparison contracts.
- Jauhar owns manual review. This increment changes no manual checkbox and promotes no automated pass into field evidence.

## Parameter conflicts, source gaps and hard refusals

- Section 5 has 74 mechanically visible rows: 15 values contain `ABSENT` and 8 contain `NON-ADOPTABLE`; chapter prose says 18 ABSENT parameters/rows. Both facts remain visible and no row is guessed into either group.
- Every ABSENT value remains absent. Current literals, familiar values, neighboring vendors, ranges, means and test fixtures are not citations.
- The nine Bateman-Konen constants remain non-adoptable until ESC-POR-8 is closed with admissible custody.
- PHIE floor remains a three-way evidence conflict: chapter no-default contract, current compile-time `0.001`, and later direct product decision `0.001`. No precedence is chosen here.
- SP-012's product choice is closed by `DEC-012`: `Cp < 1` is refused. The production refusal and qualifying regression remain open.
- SP-013's product choice is closed by `DEC-017` on true, original three-segment RHG80. The exact scan-verification gate and the separate SB-POR-020 vendor-rendering choice remain open.
- SP-014 remains open for the user-visible sonic wording surface; the prohibited proper name is not repeated in this receipt.
- ESC-1, ESC-2, ESC-3, ESC-5, ESC-7 and ESC-POR-8 remain hard custody boundaries. No protected chart, binary or confidential delivery was opened.

## Product-owner decisions and current stand

- Decide PHIE-floor evidence precedence and its visible run-configuration form.
- `DEC-012` is decided: refuse `Cp < 1`; Help text is supplementary, never the guard.
- `DEC-013` is decided: POR output names are user-configurable; distinct names preserve parallel results, while intentional same-name reuse is explicit, versioned and undoable replacement.
- `DEC-014` is decided: Arithmetic comparison, RMS comparison, SSC/SSPW RMS conditioning, Gaymard-Poupon HC response and coupled porosity-`Sxo`/`Sw` iteration are separately identified contracts; the last two are mandatory.
- `DEC-015` is decided: method-specific correction limits and validity rules operate beneath one common POR family/provenance/output-role/flag envelope; no method inherits another method's bound.
- `DEC-016` is decided: analytic N-D, HC response, excavation and neutron-sonic belong in the product; source custody and any separately deferred pilot timing remain explicit.
- `DEC-017` is decided: implement the original three-segment RHG80 product path rather than relabelling the current approximation; verify the original scan before code and keep SB-POR-020's vendor-rendering choice separate.
- Decide whether CSR, chart validation, salinity interpolation, force-wet, HC physical ceiling and smooth roll-off belong in the pilot; all remain without invented defaults.
- Neutron-sonic inclusion is settled by `DEC-016`, but its existing deferred pilot timing is not silently promoted. Keep vendor parameter-set import, POR audit report and core comparison deferred as recorded; no decision supplies a missing source or pulls every neighboring POR item into the pilot.

## Measured totals and completeness guard

- Receipt coverage: 62/62 IDs exactly once, from `SB-POR-001` through `SB-POR-062`; no duplicate or gap.
- As-built classification: 21 `PRESENT-DIVERGENT`, 15 `PARTIAL`, 25 `ABSENT`, 1 `PRESENT-UNVERIFIED`, 0 `PRESENT-OK`.
- Release disposition: 44 `PILOT-BLOCKER`, 13 `UNDECIDED`, 5 `DEFERRED`.
- Test class: 6 `CHARACTERIZATION`, 56 `MISSING`, 0 `CORRECTNESS`.
- Commit state: 37 `INTEGRATED`, 25 `UNIMPLEMENTED`.
- Ledger target after the atomic row update: 341/931 adjudicated and 590 unadjudicated. Exact cross-domain totals are reported from the validator rather than inferred here.
