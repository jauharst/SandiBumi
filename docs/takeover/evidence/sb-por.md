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
| `DEC-015` - SB-POR-001 envelope | **NEEDS-JAUHAR:** literal common limiting contract versus method-specific numerical limits under a common typed envelope | No wording is inferred from the reply labelled `805`, because that reply unambiguously selected the separate `Cp < 1` refusal. SB-POR-001 through 003 remain blocked on this boundary. |
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

- **Architecture:** a POR quantity family and complete method/convention/run provenance are required. User-configurable output names are settled by `DEC-013`. The exact common-limit boundary remains open under `DEC-015`.
- **Numerical limits and flags:** method-specific physics is not automatically an error, but a silent branch or clamp is. Existing hard bounds become cited, visible parameters and binding is observable; an uncited endpoint remains absent.
- **Sonic:** the chapter's truthful naming and per-method shale conventions remain the adopted target. `Cp < 1` is now a hard refusal. SP-013 still needs the separate rename-with-source versus true-RHG80 choice.
- **N-D and gas:** Arithmetic and RMS remain available only in their explicitly named roles. Gaymard-Poupon HC response and the coupled porosity-`Sxo`/`Sw` iteration are mandatory separate contracts. SB-POR-059's RMS parity fix remains narrow and does not implement either rigorous contract.
- **Missing capability:** analytic N-D, HC response, excavation and neutron-sonic are required product capabilities under `DEC-016`; no missing source or parameter is supplied by that inclusion decision.
- **Proof:** every atomic contract still needs an independent correctness oracle. No implementation-derived snapshot is promoted. Automated evidence remains 0 qualifying POR correctness tests, and Jauhar retains ownership of all 33 manual POR checks.

## Chapter and cross-domain findings carried into every row

- SP-009 remains open: all source-owned POR status and test fields stay blank, and numeric T26/T27 stay absent.
- SP-012's product decision is closed by `DEC-012`: `Cp < 1` is refused. The shipped Wyllie path remains divergent until the refusal and its regression test are implemented.
- SP-013 remains open: the shipped `RHG` option is a one-segment approximation under a three-segment published name. Rename-with-source versus implementing RHG80 is not chosen here.
- SP-014 remains open: one user-visible sonic description contains a prohibited geographic parenthetical. This receipt names the surface without reproducing the proper name.
- SP-015 supplies primary-source citations for the compaction estimator and RHG's no-compaction-correction property; it does not authorize an implementation choice.
- The chapter's 18-ABSENT claim does not mechanically match its 15 ABSENT-bearing rows. The mismatch is not normalized and every parameter is adjudicated from its own row.
- Current `PHIE_FLOOR = 0.001` implements a later direct product decision, while SB-POR-045 requires the conflicting vendor values to ship with no default. Current behavior, chapter contract and later decision remain separate pending product-owner precedence.
- `DEC-013` permits user-configurable POR output names and intentional, versioned replacement; it does not permit a silent collision or loss of imported/computed identity.
- `DEC-014` makes Gaymard-Poupon HC response and the coupled porosity-`Sxo`/`Sw` iterative path mandatory separate contracts. Arithmetic, RMS comparison and SSC/SSPW RMS conditioning remain separately identified roles.
- `DEC-015` remains open for the exact SB-POR-001 common-contract boundary; no answer is inferred from a mismatched line reference.
- ESC-1, ESC-2, ESC-3, ESC-5, ESC-7 and ESC-POR-8 remain source/custody boundaries. Protected vendor charts and binaries are not opened or copied, and non-adoptable constants do not become defaults.
- Manual capability baseline: porosity 0/33, generic-curve-store 0/18, conditioning 0/27, workflow 0/23, las-export 0/2 and processing-history 0/7; histogram 5/22 and crossplot 6/13 do not prove POR custody or correctness.

## SB-POR-001 - One deterministic POR family and contract

- **Specified contract:** every deterministic porosity method belongs to one POR family and uses one limiting, flag and output-naming contract; T39 is the primary discriminator, with T11 and T31 as cross-support.
- **Current implementation:** `phi_den` and `phi_dn` emit method-specific unlimited pairs plus shared limited `PHIE`/`PHIT`; `phi_son` emits only `PHIT_SON`/`PHIE_SON` and applies its own `[0,1]` clamps. `ssc` and `sspw` add still different porosity paths. A shared catalog category and runner exist, but no POR-wide limiter, flag stream or naming policy does.
- **Qualifying acceptance tests:** none; no executable `SB-POR-T39` or whole-family inventory proves every method. Test class `MISSING`.
- **Supporting tests:** `every_module_returns_the_output_keys_its_manifest_declares`, the density and D-N branch tests, and the sonic option test each passed exactly once; they prove local manifest/arithmetic behavior only.
- **Manual evidence:** porosity 0/33; workflow 0/23; generic-curve-store 0/18.
- **Source/parameter boundary:** no new value is needed; this is an architecture contract.
- **History/reachability:** all current paths are integrated at the accepted anchor; current and reachable-history searches found no unified POR contract.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-015` remains open: literal common numerical limiting versus method-specific limits beneath one common family/provenance/flag envelope.
- **Next action:** Jauhar selects that boundary; then define the common POR result and inventory every registered method against it before migrating any individual method.

## SB-POR-002 - Unlimited and limited pairs for every method

- **Specified contract:** every method must preserve both unlimited `PHIT/PHIE` and limited `PHIT/PHIE`, with meanings visible through write, reload and export; T11, T19, T31 and T39 jointly constrain it.
- **Current implementation:** density and D-N retain unlimited method-specific twins and shared limited outputs. Sonic has only one method-specific pair and clamps it in place; SSC/SSPW do not follow the twin convention.
- **Qualifying acceptance tests:** none; no whole-family test proves both pairs and their custody for every method. Test class `MISSING`.
- **Supporting tests:** `a_negative_density_porosity_is_floored_but_stays_visible_in_the_unlimited_twin` passed exactly once and closes only density and D-N examples; manifest-output parity passed but cannot prove semantic meaning.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18; las-export 0/2.
- **Source/parameter boundary:** the contract introduces no endpoint or default.
- **History/reachability:** the density twin change is reachable; no corresponding sonic or full-family implementation was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-001 and collision-free custody from SB-POR-004.
- **Next action:** add semantic unlimited/limited fields to the common POR result, then prove both are independently stored and exported for every method.

## SB-POR-003 - Per-sample branch and limit flag

- **Specified contract:** one POR flag must identify every material branch and every binding floor/ceiling per sample; T12, T38, T39 and T41 require positive and negative controls.
- **Current implementation:** POR methods clamp numeric outputs or leave `NaN` without a POR flag. `BADHOLE` and conditioning flags exist as separate detector outputs, and workflow masking is generic, but none is the required branch-and-limit stream.
- **Qualifying acceptance tests:** none; executable tests do not observe a POR flag through persistence/export. Test class `MISSING`.
- **Supporting tests:** `badhole_flags_washout_and_drho`, three `condflag` tests, and `the_empty_flag_refusal_names_the_users_curve_and_its_remedy_works` passed exactly once; they prove detector and generic-mask behavior only.
- **Manual evidence:** porosity 0/33; conditioning 0/27; processing-history 0/7.
- **Source/parameter boundary:** flag vocabulary is specified behavior, not a numeric parameter.
- **History/reachability:** no POR `PHIFLAG` or equivalent was found in current source, tests or reachable history.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** depends on the common POR result and declared consumption of conditioning flags.
- **Next action:** define one machine-readable POR reason/branch schema, populate it at each branch and clamp, and prove an unbound sample remains distinguishable.

## SB-POR-004 - Typed POR family, provenance and collision-free names

- **Specified contract:** porosity curves carry POR quantity-family typing plus method/convention provenance; sequential methods preserve distinct outputs and imported-versus-computed identity; T31 and T32 pin both sides.
- **Current implementation:** `curves.rs::FAMILIES` has no porosity family. `resolve_output_names` blocks collisions inside one run and standard-column shadowing, but sequential `phi_den` and `phi_dn` both write current `PHIE`/`PHIT`; versioned writes replace those current names. Run metadata records the module but not a per-output method/convention identity.
- **Qualifying acceptance tests:** none; T31/T32 are not executable. Test class `MISSING`.
- **Supporting tests:** `families_resolve_common_mnemonics`, `an_output_name_that_would_be_shadowed_is_refused_before_a_single_well_runs` and `a_restored_log_set_version_feeds_the_next_module_run` passed exactly once. None exercises sequential POR collision plus family/provenance.
- **Manual evidence:** generic-curve-store 0/18; workflow 0/23; las-export 0/2.
- **Source/parameter boundary:** not numeric; imported and computed identity must remain distinct.
- **History/reachability:** no POR family, `MTH_PHI`, convention field or collision-free sequential scheme was found in current or reachable source.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** `DEC-013` settles the policy: names are user-configurable; distinct names preserve parallel results; intentional same-name reuse is explicit, versioned replacement; silent collision remains forbidden.
- **Next action:** add POR family metadata, method/convention provenance and user-configurable output names; prove distinct-name preservation, explicit same-name replacement plus restore, and imported-versus-computed identity.

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
- **Current implementation:** POR manifests request log mnemonics and workflow resolves curve identity, but `curves.rs` supplies no VSH/VCL-to-POR type validation at the module boundary. Matching text is accepted without a family proof.
- **Qualifying acceptance tests:** none; neither typed acceptance nor wrong-family refusal is executable for POR. Test class `MISSING`.
- **Supporting tests:** generic family recognition and empty-flag refusal passed; neither binds VSH/VCL type to POR input resolution.
- **Manual evidence:** porosity 0/33; generic-curve-store 0/18; workflow 0/23.
- **Source/parameter boundary:** no shale endpoint is inferred; this is input custody.
- **History/reachability:** current and reachable-history searches found no typed POR shale-input guard.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on the quantity-family registry and an observable refusal surface.
- **Next action:** require a typed VSH/VCL identity in POR requests and test one accepted typed curve against same-named wrong-family and untyped controls.

## SB-POR-007 - Parameter citation and evidence tier in the dialog

- **Specified contract:** every POR parameter carries its source citation and evidence tier, the dialog displays both, and the chosen value/source is retained in run provenance.
- **Current implementation:** `ArgSpec.sources_topic`, the backend source registry and the dialog source panel are generic working seams. POR arguments have no topics; the registry has no POR entries or evidence-tier field, and run persistence does not retain source/tier selection.
- **Qualifying acceptance tests:** none; no universal POR parameter-source inventory or observable run round-trip exists. Test class `MISSING`.
- **Supporting tests:** source-panel infrastructure tests prove the generic seam only.
- **Manual evidence:** porosity 0/33; workflow 0/23; processing-history 0/7.
- **Source/parameter boundary:** chapter citations remain authoritative; missing topics must not be filled from current literals.
- **History/reachability:** generic source-topic support is reachable; no POR topic/tier coverage was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires a POR parameter-to-source/tier inventory that preserves ABSENT and NON-ADOPTABLE states.
- **Next action:** register every admissible POR parameter source and tier, expose it in the dialog, and persist the selected source alongside the value.

## SB-POR-008 - One formation-water PHIT_SH across the CLY seam

- **Specified contract:** every POR formation path uses and exports one shared formation-water-based `PHIT_SH`, kept distinct from shale subtraction; T15, T19 and T30 compare all paths.
- **Current implementation:** density and D-N call shared `phit_sh_at` with `RHO_W`. `sspw` calculates a similar term with its neighboring fluid-density argument and does not call the helper; sonic follows no equivalent shared path. No exported seam proves identity across modules.
- **Qualifying acceptance tests:** none; T15/T19/T30 are not executable as cross-module controls. Test class `MISSING`.
- **Supporting tests:** density/D-N shale tests and `sspw_phie_removes_only_clay_bound_water` passed exactly once, but their implementations are not an independent oracle for one another.
- **Manual evidence:** porosity 0/33; shale-volume 0/17; workflow 0/23.
- **Source/parameter boundary:** formation-water density and shale quantities must remain typed; no neighboring fluid value is substituted.
- **History/reachability:** shared density helper and separate SSPW expression are both integrated.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires one exported helper/result contract across POR and CLY, then removal of parallel formulations.
- **Next action:** route all eligible paths through one typed PHIT_SH calculation and prove identical output from one cited parameter set while preserving shale-subtraction distinction.

## SB-POR-009 - Limit PHIE before rebuilding PHIT

- **Specified contract:** at every finite sample, limited `PHIE` is formed first and `PHIT` is rebuilt so `PHIT >= PHIE`; T11 covers floor, ceiling, shale branch and missing data for every method.
- **Current implementation:** `phi_den` and `phi_dn` use the required order. `phi_son` independently clamps both outputs, and SSC/SSPW use separate paths, so the universal ordering is not established.
- **Qualifying acceptance tests:** none; no all-method, all-branch test exists. Test class `MISSING`.
- **Supporting tests:** density/D-N flooring and shale-branch tests passed exactly once and show local ordering; they cannot close sonic or SSC/SSPW.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** the ordering is specified; no floor value is adopted by this receipt.
- **History/reachability:** the density/D-N ordering is integrated; no common family implementation was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-001 and the unresolved SB-POR-045 floor custody.
- **Next action:** place order enforcement in the common POR limiter and test floor, ceiling, shale, naturally ordered and missing samples for every registered method.

## SB-POR-010 - Re-derivable audit trail for every POR curve

- **Specified contract:** every computed POR curve records method, complete resolved parameters including defaults and zones, exact input set/mnemonic identities, and convention so the run is re-derivable.
- **Current implementation:** `LogSetSpec` stores module, explicit `req.params`, and input set/mnemonic identities. It omits defaults, fully resolved zone parameters, options/method selectors held outside that map, source/tier and per-output convention.
- **Qualifying acceptance tests:** none; no POR run round-trip proves complete reconstruction. Test class `MISSING`.
- **Supporting tests:** `a_restored_log_set_version_feeds_the_next_module_run` passed exactly once and proves version reuse, not full POR provenance.
- **Manual evidence:** workflow 0/23; processing-history 0/7; generic-curve-store 0/18.
- **Source/parameter boundary:** provenance must retain cited/absent status rather than manufacture defaults.
- **History/reachability:** partial generic run metadata is integrated; no complete POR audit record was found.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** requires a resolved-run schema spanning values, options, sources, zones, identities and outputs.
- **Next action:** persist a canonical resolved POR invocation and prove save/reload reproduces every field without querying mutable defaults.

## SB-POR-011 - One shared matrix-density decision

- **Specified contract:** one matrix-density selection propagates unchanged through every documented POR, gas-correction and conditioning module in a chain; T35 tests one override end to end.
- **Current implementation:** multiple manifests expose independent density-like defaults and parameters. No shared typed matrix-density object or chain-level override custody connects POR, `gascorr`, `condflag` and saved workflows.
- **Qualifying acceptance tests:** none; T35 is not executable. Test class `MISSING`.
- **Supporting tests:** local module tests exercise their own defaults; equal-looking values are not shared custody.
- **Manual evidence:** porosity 0/33; conditioning 0/27; workflow 0/23.
- **Source/parameter boundary:** matrix density is petrophysical and must come from a cited/user decision; this pass adopts none.
- **History/reachability:** current and reachable-history searches found no chain-level matrix-density decision.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the typed shared parameter and precedence rules are absent.
- **Next action:** introduce one cited/user-owned matrix-density reference and prove a non-default override reaches every documented consumer unchanged.

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
- **Source/parameter boundary:** SP-013 and SP-015 distinguish the current approximation, proposed field-observed form and RHG80 source; no coefficient or rename decision is inferred.
- **History/reachability:** the misnamed branch is integrated; no true RHG80 or `FIELD_OBSERVED` path was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** Jauhar must choose rename-with-source versus implementing RHG80; SP-014's wording cleanup is also open.
- **Next action:** adjudicate SP-013, then expose only names backed by their actual equations and prove label, formula, parameter and run provenance together.

## SB-POR-015 - Method-correct sonic shale treatment

- **Specified contract:** non-Wyllie methods first form shale-reduced slowness, floor it at matrix slowness, evaluate the method and rescale; Wyllie remains on its cited subtractive control path. T18, T18b and T21 distinguish the paths.
- **Current implementation:** `phi_son` feeds raw DT into both current branches, then applies a shared effective-porosity subtraction. No shale-reduced/matrix-floored non-Wyllie path or iterative field-observed seed exists.
- **Qualifying acceptance tests:** none; the three specified discriminators are not executable. Test class `MISSING`.
- **Supporting tests:** the current sonic option test passed exactly once, but it asserts current embedded behavior rather than the chapter's split convention.
- **Manual evidence:** porosity 0/33.
- **Source/parameter boundary:** SP-015 supplies newer primary-source direction; it does not choose a method or coefficient.
- **History/reachability:** no compliant non-Wyllie reduction/rescale branch was found in current source, tests or reachable history.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** depends on SB-POR-013 conventions and the SP-013 method identity decision.
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
- **Source/parameter boundary:** SP-013 leaves rename versus RHG80 open; SP-015's source does not authorize an adjudication-time default choice.
- **History/reachability:** no published RHG implementation or comparison-curve mechanism was found.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** Jauhar must decide the SP-013 method disposition before an authoritative rendering can be named.
- **Next action:** implement that decision with one source-bound authoritative method, distinctly typed comparisons, and a label/formula/provenance acceptance test.

## SB-POR-021 - Chart-free analytic neutron-density method

- **Specified contract:** provide the cited Bateman-Konen analytic N-D method, independently validate it against the gated chart branch, and keep the average shortcut measurably distinct; T08 and T37 constrain both sides.
- **Current implementation:** `phi_dn` offers only `AVERAGE` and `GAS_RMS` shortcuts. No Bateman-Konen evaluator exists; unrelated Bateman-Konen salinity/Rw code is not POR. The nine chapter constants remain non-adoptable.
- **Qualifying acceptance tests:** none; T08/T37 are not executable. Test class `MISSING`.
- **Supporting tests:** `phi_dn_crossplot_shale_reduction_and_branches` passed exactly once for the current shortcuts, not the analytic method.
- **Manual evidence:** porosity 0/33; crossplot 6/13, with no POR analytic-method evidence.
- **Source/parameter boundary:** ESC-POR-8 blocks adoption of the nine constants; they remain verification-only and no chart value is copied.
- **History/reachability:** source, test and reachable-history searches found no POR Bateman-Konen method.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** close ESC-POR-8 with admissible primary custody before the analytic method can ship.
- **Next action:** obtain the admissible constants/source, implement the analytic evaluator, and compare it independently with both chart and shortcut discriminators.

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
- SP-013 remains open: rename the current approximation with a source versus implement true RHG80 is unchosen.
- SP-014 remains open for the user-visible sonic wording surface; the prohibited proper name is not repeated in this receipt.
- ESC-1, ESC-2, ESC-3, ESC-5, ESC-7 and ESC-POR-8 remain hard custody boundaries. No protected chart, binary or confidential delivery was opened.

## Product-owner decisions and current stand

- Decide PHIE-floor evidence precedence and its visible run-configuration form.
- `DEC-012` is decided: refuse `Cp < 1`; Help text is supplementary, never the guard.
- `DEC-013` is decided: POR output names are user-configurable; distinct names preserve parallel results, while intentional same-name reuse is explicit, versioned and undoable replacement.
- `DEC-014` is decided: Arithmetic comparison, RMS comparison, SSC/SSPW RMS conditioning, Gaymard-Poupon HC response and coupled porosity-`Sxo`/`Sw` iteration are separately identified contracts; the last two are mandatory.
- `DEC-015` remains `NEEDS-JAUHAR`: literal common limiting contract versus method-specific numerical limits under one common POR family/provenance/flag envelope.
- `DEC-016` is decided: analytic N-D, HC response, excavation and neutron-sonic belong in the product; source custody and any separately deferred pilot timing remain explicit.
- Decide SP-013 rename-with-source versus implementing RHG80.
- Decide whether CSR, chart validation, salinity interpolation, force-wet, HC physical ceiling and smooth roll-off belong in the pilot; all remain without invented defaults.
- Neutron-sonic inclusion is settled by `DEC-016`, but its existing deferred pilot timing is not silently promoted. Keep vendor parameter-set import, POR audit report and core comparison deferred as recorded; no decision supplies a missing source or pulls every neighboring POR item into the pilot.

## Measured totals and completeness guard

- Receipt coverage: 62/62 IDs exactly once, from `SB-POR-001` through `SB-POR-062`; no duplicate or gap.
- As-built classification: 21 `PRESENT-DIVERGENT`, 15 `PARTIAL`, 25 `ABSENT`, 1 `PRESENT-UNVERIFIED`, 0 `PRESENT-OK`.
- Release disposition: 44 `PILOT-BLOCKER`, 13 `UNDECIDED`, 5 `DEFERRED`.
- Test class: 6 `CHARACTERIZATION`, 56 `MISSING`, 0 `CORRECTNESS`.
- Commit state: 37 `INTEGRATED`, 25 `UNIMPLEMENTED`.
- Ledger target after the atomic row update: 341/931 adjudicated and 590 unadjudicated. Exact cross-domain totals are reported from the validator rather than inferred here.
