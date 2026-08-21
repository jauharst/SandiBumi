# SB-SAT live adjudication receipt

## Execution baseline

- Working tree: `D:\XX. SandiBumi`
- Branch: `codex/g1-sb-sat-adjudication`
- Planning baseline: `74a9d596b21c06df8d1aa83fadf3053af058813d`
- Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)
- `origin/master` and merge base: `29833735816d9e5be954afafd9ceb71fd856e3f0`
- Scope: exactly 51 `SB-SAT` requirements, 63 chapter acceptance-test intentions, and 71 parameter rows.
- Baseline ledger: 341 adjudicated / 590 unadjudicated of 931.
- Manual evidence remains source-owned: saturation 2/97, saturation-height 0/6, workflow 0/23, crossplot 6/13, pickett 0/8, LAS export 0/2, processing history 0/7, verification stewardship 0/24.

## Governing boundaries

- A petrophysical parameter is cited or remains absent. Existing defaults are evidence of shipped behavior, not authority.
- Deterministic modules, the multimin solver, LRLC methods, and Results QC are separate engines until identical typed inputs and quantities prove parity.
- Clipped and unclipped saturation, effective and total saturation, and method guidance and method selection are separate contracts.
- Generic JSON ancestry is not complete scientific provenance unless method, inputs, parameters, sources, papers, calibration state, and flags survive export.
- The live chapter contains ten section 7.1 escalations while section 8.16 says nine; this receipt records the mismatch and does not silently correct the specification.
- `SP-003` remains open for independent literature/patent evidence and does not authorize invention of a Tier-C capability.
- Of 71 parameter rows, 20 carry an ABSENT state and 8 have no tier. Neither group may be silently populated.

## Requirement receipts

### SB-SAT-001 - Name every saturation model by its equation

- **Specified contract:** every saturation model **MUST** be identified by a stable identifier naming its EQUATION, never a vendor's adjective, and no selector may offer a bare `Modified` / `Simandoux` / `Modified Simandoux` (`12_saturation.md:867-890`). Every internal function, doc comment and enum variant **MUST** use the same identifier as the user-facing name.
- **Why it is P0:** *Modified* means Geolog's `Vsh.Sw` shale term in one product and IP's/Techlog's `(1-Vcl)` divisor in another. Selecting by adjective costs **7.3 saturation units and +19 % HCPV**.
- **Current implementation - the as-built was STALE and this was verified in code, not assumed.** The row said `multimin2.rs:115,164` mislabel the Schlumberger form as Bardon-Pied. They do not: `:115` reads *Simandoux / Bardon-Pied form without a `(1-Vsh)` divisor* and the Schlumberger variant is described with the divisor, which is correct. `sw_sim`'s `OPT_SIM` already offers `simandoux_bardon_pied` and `simandoux_modified_slb`, each label leading with its own id and the vendor adjective only trailing it in parentheses. `canonical_option_value` (`modules.rs:2078`) accepts legacy vendor tokens at the input boundary so saved chains keep running, and returns only equation ids. The row was a **PROVE**.
- **Qualifying acceptance tests:** `every_saturation_model_is_named_by_its_equation_and_no_selector_offers_a_bare_vendor_adjective` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Supporting tests:** the `sw_sim` arithmetic tests exercise both branches; none of them could fail on a NAME, which is why the naming contract had no proof.
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Source/parameter boundary:** no parameter value is involved. The fourteen canonical ids are quoted from the chapter, not invented.
- **Four-armed pin.** (A) both Simandoux forms are offered under equation ids and every label LEADS with its own id, so an adjective can only trail it. (B) legacy tokens still resolve and resolve the RIGHT way round - `MODIFIED` is **Geolog's** name for Bardon-Pied, not the Schlumberger form - and an already-canonical id passes through unchanged so re-running a new chain is stable. (C) universal: NO shipped option on ANY module offers a bare vendor adjective as its stored value, so a future module cannot reintroduce the ambiguity. (D) the solver engine agrees with the UI - every `sw_model_catalog` entry is an equation id whose label leads with it. Two engines, one vocabulary.
- **Verified by mutation:** swapping arm B's mapping so Geolog's `MODIFIED` selects the Schlumberger form fails the test. That swap is exactly the 7.3-saturation-unit error, and it computes and plots either way.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that a saved chain storing `MODIFIED` still runs and now reports `simandoux_bardon_pied`.

## SB-SAT-002 - Ship effective and total Archie as separate named methods

- **Chapter evidence:** P0; `12_saturation.md:893-909`; owned tests `SB-SAT-T03`, `SB-SAT-T04`; the dossier 3.1 reference case at `:121-123`.
- **Scope ruling:** DEC-048 (2026-08-16) authorized `multimin2.rs` for exactly this family of rows; DEC-062 (2026-08-17) opens the tree. Nothing petrophysical was decided here - the effective equation is cited at `:893-896` and the SWT lift is SB-SAT-023's inverse at `:1337-1344`.
- **Current source:** `sw_arch` carries `OPT_EQN` with the equation identities `archie_total` (default - what every saved run has always computed) and `archie_effective`. The effective branch computes `SWE = (A*Rw/(PHIE^M*RT))^(1/N)` directly on effective porosity and lifts SWT through `SwT = Sw(1-Swb)+Swb`, `Swb = 1-PHIE/PHIT`; `PHIE < 0.005` keeps the module's standing all-water convention, which is also SB-SAT-023's `Swb = 1 -> SWE = 1` clause. `SW_METHOD` stamps `ArchieEffective`'s own code 9.0. `multimin2` gains the `ArchieEffective` variant, id `archie_effective`, its own `flag_code`, a catalogue entry, and a post-solve branch computing Sw on phie directly - the result IS the free-water fraction, so no total->effective conversion is applied, which on any other branch would be the trap itself.
- **Qualifying acceptance tests:** `modules::tests::effective_and_total_archie_are_two_separately_named_methods_that_disagree_on_the_reference_case` - exact `SB-SAT-T03` with T04's identity arm. **Both witnesses are pinned on ONE fixture** (phit 0.25, phie 0.20, Rw 0.25, a=1, m=n=2, Rt=8 -> 0.634 and 0.884, +-0.002): a test checking either branch alone passes an implementation wiring both selections to one equation, which is the defect the row names - the chapter calls it the largest single cross-tool trap in the domain because nothing errors. Test class `CORRECTNESS`.
- **Mutation record, including one hole found and closed:** the pre-implementation RED was itself the both-branches-one-equation probe (effective returned 0.6338835). Five further mutations: a flipped spec default **SURVIVED the first pass** - the test harness bypasses spec defaults, so the bit-for-bit arm proved only the body's absent-option fallback - and arm B now also pins the DECLARED default on the spec itself, after which the same mutation fails at the named assertion. The shared-flag, dropped-inverse, phie-swapped-for-phit and missing-catalogue-entry probes each fire their own arm.
- **Manual evidence:** saturation 0/41 - unexercised. Automated only; no manual or field evidence is claimed.
- **Git evidence:** additive; the default preserves every saved run; no assertion weakened. Full suite 1053 passed / 0 failed / 37 ignored.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none outstanding.
- **Next action:** map `IP Archie -> archie_effective` and `IP Archie PhiT -> archie_total` when SB-SAT-003's alias table is built.

## SB-SAT-003

- **Specified contract:** Ship a vendor alias table and resolve imports through it. Owned test intention(s): `SB-SAT-T01`, `SB-SAT-T05`.
- **Current implementation:** no saturation alias table or import resolver exists; current strings are local option labels and direct enum values.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T01`, `SB-SAT-T05` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no mapped/unmapped alias acceptance test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** external-name ownership and unknown-name refusal are absent.
- **Next action:** add one source-backed alias table, resolve imports only through it, and test known aliases against explicit unknown refusal.

### SB-SAT-004

- **Specified contract:** Simandoux: two variants, with `C` on the Schlumberger variant only. Owned test intention(s): `SB-SAT-T06`, `SB-SAT-T07`.
- **Current implementation:** the standalone module separates Bardon-Pied and modified-Schlumberger branches and applies `C` only to the latter, but permits `C` from 0.5; the solver calls a denominator-bearing equation Bardon-Pied and exposes no `C`.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_sim_matches_quadratic_solution`, `sw_simandoux_round_trips` and `sw_equations_match_hand_computed_points` passed for the current equations.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** variant identity, `C` range and cross-engine equation parity disagree.
- **Next action:** assign canonical IDs to both equations, bind `C` only to the sourced branch with range 1..2, then cross-assert both engines.

### SB-SAT-005

- **Specified contract:** Simandoux `a` ships with no default. Owned test intention(s): `SB-SAT-T08`, `SB-SAT-T31`.
- **Current implementation:** standalone Simandoux ships `A=1.0`; the solver ships `archie_a=1.0`. Neither represents absence.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T08`, `SB-SAT-T31` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** equation tests supply explicit `a`; none proves no-default refusal.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the uncited shipped default violates parameter custody.
- **Next action:** remove the default from resolved-run construction, require a cited or user value, and pin missing-value refusal plus explicit-value success.

### SB-SAT-006 - Indonesia with a parameterised shale exponent

- **Specified contract:** Indonesia is `v = Vsh^(2 - k*Vsh)` with `SWE = (1/(Rt*(1/(ff*Rw) + 2*sqrt(v/(Rw*ff*Rsh)) + v/Rsh)))^(1/n)`, `ff = a/phie^m`, exposing `k` with presets `FULL (k=1)`, `SIMPLE (k=0)` and `TAR_SAND/Woodhouse (k=2)`. **Both the deterministic module and the solver MUST use the same parameterised form** (`12_saturation.md:908-920`).
- **Current implementation - the as-built was STALE, verified in code.** It said `multimin2.rs:154` hard-codes `Vsh^(1 - Vsh/2)`, i.e. k=1 only, so the solver could not run SIMPLE or TAR_SAND. It does not: `multimin2.rs:277` reads `vsh.powf(1.0 - k * vsh / 2.0)`, and `:523-525` documents `indonesia_k` as *Geolog's FULL/SIMPLE/TAR_SAND presets are k=1/0/2*. The solver row is written for `1/sqrt(Rt)`, so its shale factor is the square ROOT of the module's - squaring returns `Vsh^(2 - k*Vsh)`, the same family. The module (`modules.rs:5049-5052`) already implements all three. The row was a **PROVE**.
- **Qualifying acceptance tests:** `the_three_indonesia_presets_are_the_chapter_k_values_and_the_solver_shares_the_same_form` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Supporting tests:** `sw_indo_full_vs_simple` and `sw_indonesia_round_trips` (both CHARACTERIZATION, moved out of the qualifying register when this row's class became CORRECTNESS - neither test was deleted), plus `sw_indo_nonpositive_rt_is_missing_not_inf`, which pins a guard rather than the preset identity.
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Source/parameter boundary:** every expectation is evaluated from the CHAPTER's equation with an explicit `k`, never read back from the module. That is what makes arm A a check of the named presets rather than a restatement of whatever the code does.
- **Four-armed pin.** (A) each named preset IS its cited k - FULL 1, SIMPLE 0, TAR_SAND 2 - against an independent evaluation of the chapter equation. (B) the three presets give genuinely DIFFERENT answers, so arm A cannot be satisfied by a module that ignores the option and returns one curve for all three. (C) the solver shares the form: `(Vsh^(1 - k*Vsh/2))^2 == Vsh^(2 - k*Vsh)` at every preset - this is the clause that says both engines run the same equation. (D) the shipped default is `FULL`, so an unconfigured run uses the cited preset.
- **Deliberate limit, stated rather than hidden:** the SOLVER's own default k is documented as 1 at `multimin2.rs:523-525` but is not asserted, because reaching it means deserializing `FluidProps`, which has many required fields - a test that builds a whole fluid model to read one default would break for reasons unrelated to this contract. Arm D asserts the module default instead.
- **Verified by mutation:** changing `TAR_SAND` from the cited k=2 to k=1.5 fails the test. That is the realistic error - a preset quietly rebound to a different exponent still computes and still plots.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that SIMPLE and TAR_SAND give visibly different SWE from FULL on a shaly interval, in both the module and a SandiMin solve.

## SB-SAT-007

- **Specified contract:** Woodhouse Tar as a cited alias of Indonesia `k = 2`. Owned test intention(s): `SB-SAT-T10`, `SB-SAT-T35`.
- **Current implementation:** a `TAR_SAND` option exists, but it is not carried as the cited Woodhouse Tar alias through a shared registry or solver path.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T10`, `SB-SAT-T35` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** current Indonesia tests do not pin the alias identity and cited `k=2` mapping.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** pilot inclusion of the external alias is undecided and alias custody depends on SB-SAT-003.
- **Next action:** if included, register the cited alias to canonical Indonesia k=2 and test both alias resolution and canonical provenance.

### SB-SAT-008

- **Specified contract:** Total Shale as a preset of `simandoux_modified_slb` with `n` fixed at 2. Owned test intention(s): `SB-SAT-T11`, `SB-SAT-T12`.
- **Current implementation:** no Total Shale preset exists in Rust, TypeScript, tests or reachable code history.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T11`, `SB-SAT-T12` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** current Simandoux tests do not exercise the specified preset.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion is undecided; the preset must depend on the corrected modified-Schlumberger identity.
- **Next action:** if included, add only a declarative preset fixing C=1 and n=2 after SB-SAT-004 is corrected.

### SB-SAT-009

- **Specified contract:** Juhász: shale-derived coefficient, shale-based normalization, model's own `m*`. Owned test intention(s): `SB-SAT-T13`, `SB-SAT-T14`, `SB-SAT-T30`.
- **Current implementation:** the solver derives a shale-normalized coefficient from VSH, shale porosity and Rsh, but uses the generic `m`, fixed `a=1`, shipped shale defaults and no distinct `m*` custody.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_juhasz_hand_computed` passed for the current normalized formula.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** model-specific exponent and source-free shale inputs remain unresolved.
- **Next action:** make shale normalization, m-star, Rsh and shale porosity explicit sourced inputs and preserve their identity through the run.

### SB-SAT-010

- **Specified contract:** Juhász MUST flag a negative excess-conductivity coefficient. Owned test intention(s): SB-SAT-T15.
- **Current implementation:** a negative `(Cwsh-Cw)` term can enter the root equation; no per-sample or run flag records that condition.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T15 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no negative-coefficient flag test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the required observable safety signal is absent.
- **Next action:** detect the negative coefficient before solving, emit the specified flag, and test flagged versus positive controls.

### SB-SAT-011

- **Specified contract:** Waxman-Smits with `a` exposed. Owned test intention(s): SB-SAT-T16.
- **Current implementation:** Waxman-Smits is implemented with `a=1` fixed inside the helper and no exposed tortuosity parameter.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_waxman_smits_hand_computed` and the shaly post-solve test passed for fixed `a=1`.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the equation parameter is not exposed or recorded.
- **Next action:** add typed source-bound `a`, thread it through every engine and prove a non-unit value changes only the intended term.

### SB-SAT-012

- **Specified contract:** `B` MUST be a unit-typed quantity, canonically `L·S/(eq·m)`. Owned test intention(s): `SB-SAT-T17`, `SB-SAT-T18`.
- **Current implementation:** `B` crosses all interfaces as an untyped `f64`; comments state units but the wrong hundred-fold scale remains representable.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_waxman_smits_hand_computed` and `waxman_b_matches_juhasz_fit` passed with raw scalars.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** canonical unit typing and wrong-scale refusal do not exist.
- **Next action:** introduce a canonical typed B quantity and exact converter, then reject the wrong scale at the boundary.

### SB-SAT-013

- **Specified contract:** `Qv` MUST be unit-typed, canonically meq/mL. Owned test intention(s): `SB-SAT-T19`, `SB-SAT-T20`.
- **Current implementation:** Qv crosses module, solver, UI and Results QC as raw scalars; routes use comments but no machine-enforced meq/mL identity.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: hand-computed Waxman-Smits and dual-water tests passed with raw Qv scalars.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** unit-safe Qv construction and wrong-unit refusal are absent.
- **Next action:** define canonical Qv typing, convert every admitted source explicitly, and reject a thousand-fold unit error.

### SB-SAT-014

- **Specified contract:** `B(T,Rw)` MUST consume typed °C and clamp `B ≥ 0`. Owned test intention(s): `SB-SAT-T21`, `SB-SAT-T22`.
- **Current implementation:** both helpers assume Celsius `f64` and clamp non-negative, while UI data begins in Fahrenheit and converts outside the type system; typed Fahrenheit can be passed accidentally.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `waxman_b_matches_juhasz_fit`, `juhasz_b_is_positive_and_grows_with_temperature` and formation-temperature tests passed.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the arithmetic is locally pinned but the temperature unit is not enforced.
- **Next action:** make Celsius a typed input, centralize the sourced formula and clamp, and add explicit wrong-unit refusal.

### SB-SAT-015

- **Specified contract:** `B` method ships with no default and four named options. Owned test intention(s): `SB-SAT-T23`, `SB-SAT-T31`.
- **Current implementation:** the product auto-selects one formula unless a raw override is positive; four named methods, user-defined provenance and no-default behavior do not exist.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T23`, `SB-SAT-T31` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** current B tests prove one automatic formula only.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** method choice and source custody are absent.
- **Next action:** model the four cited methods plus user-defined as explicit choices with no elected default and persist the selection.

### SB-SAT-016

- **Specified contract:** Dual water ships in two named forms. Owned test intention(s): `SB-SAT-T24`, `SB-SAT-T25`.
- **Current implementation:** linear and nonlinear dual-water variants exist, but naming, aliases, parameterization and flags are not unified across the product.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_dual_nonlinear_hand_computed_and_conversion` and Results-QC dual-water tests passed for current helpers.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the two forms lack one explicit canonical registry and full discriminating proof.
- **Next action:** register both forms separately, map parameters and aliases explicitly, and prove their sourced equality and difference cases.

### SB-SAT-017

- **Specified contract:** The excess-conductivity coefficient MUST be `Swb·(Cwb − Cw)`. Owned test intention(s): SB-SAT-T26.
- **Current implementation:** the nonlinear helper uses the required `Swb*(Cwb-Cw)` excess-conductivity coefficient.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_dual_nonlinear_hand_computed_and_conversion` independently checks the current arithmetic, but does not name the chapter source or isolate one owned contract.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the coefficient is integrated, but the owned sourced regression test and surrounding alpha/beta/unit custody are still missing.
- **Next action:** add the one-contract sourced regression test, then keep this coefficient fixed while repairing adjacent unit and water-property paths.

### SB-SAT-018

- **Specified contract:** `vQ` MUST switch temperature form on the expansion branch. Owned test intention(s): SB-SAT-T27.
- **Current implementation:** one collapsed temperature expression is applied uniformly; the specified expanded and saline vQ branches are not represented.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T27 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** fluid and temperature tests passed for current behavior, not the two sourced branches.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** branch identity and source-bound equations are absent.
- **Next action:** implement separately named expanded and saline vQ temperature branches and prove the cited divergence.

### SB-SAT-019

- **Specified contract:** α MUST include the Debye-Hückel activity ratio. Owned test intention(s): SB-SAT-T28.
- **Current implementation:** alpha is computed from a simplified square-root expression with a hard ceiling and no Debye-Huckel activity ratio.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T28 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no activity-ratio acceptance test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the sourced activity term is missing and the ceiling has no admitted source.
- **Next action:** add the sourced activity ratio; keep the ceiling absent until a cited value exists, then test the cited salinity points.

### SB-SAT-020

- **Specified contract:** β MUST carry the salinity dilution factor. Owned test intention(s): SB-SAT-T29.
- **Current implementation:** no beta salinity-dilution factor is present in current Rust, TypeScript, tests or reachable code history.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T29 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no beta discriminator test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the fresh-water branch is incomplete.
- **Next action:** implement only the cited beta dilution term and pin its sourced factor against a no-dilution control.

### SB-SAT-021

- **Specified contract:** `Qv > 1/vQ` MUST flag; `Swb ≤ 1 − φe/φt` MUST clamp. Owned test intention(s): SB-SAT-T32.
- **Current implementation:** Qv is numerically assembled and bound-water quantities are clamped through local expressions, but neither the Qv validity condition nor a distinct Swb clamp flag is emitted.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T32 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no test observes the two conditions as separate flags.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** both validity signals and their result transport are absent.
- **Next action:** add separate machine-readable Qv-invalid and Swb-clamped reasons and prove each can fire alone.

### SB-SAT-022

- **Specified contract:** `vQ0` ships absent. Owned test intention(s): `SB-SAT-T31`, `SB-SAT-T33`.
- **Current implementation:** no explicit vQ0 parameter exists, but a simplified hard-coded expansion expression and ceiling prevent the required explicit absence/candidate choice.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T31`, `SB-SAT-T33` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no no-default and two-candidate test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the unresolved parameter is hidden inside implementation rather than shipped absent.
- **Next action:** remove implicit custody, offer only the cited candidates without a default, and refuse runs that require an unselected vQ0.

### SB-SAT-023 - The effective back-out is per model, never blanket

- **Specified contract:** apply `SWE = MAX((SWT - Swb)/(1 - Swb), 0)` with a **per-model** `Swb`: `1 - phie/phit` for `archie_total`, `waxman_smits` and both dual-water forms; **`Qvn = clamp(Vsh*phit_sh/phit, 0, 1)`** for `juhasz`. `Swb = 1` MUST yield `SWE = 1`, not a divide-by-zero. Where the solver's construction collapses the two rules, SandiBumi MUST record which rule was applied. It MUST also ship the inverse pair `SwT = Sw(1 - Swb) + Swb` and `SxoT = Sxo(1 - Swb) + Swb`, and a round-trip through the pair MUST be the identity (`12_saturation.md:1337-1350`).
- **Why it matters:** for the first group IP's E78 and Geolog's form are algebraically identical and all three tools agree. For Juhasz they are **not** equal - on the dossier fixture `Qvn` 0.42 against `1 - phie/phit` 0.20, `SWE` differs by **tens of saturation units while `SWT` matches exactly**. The chapter calls this the purest example of the silent, method-specific divergence the whole chapter exists to prevent.
- **Current implementation - verified in code:** `sw_juhasz` (`multimin2.rs:443`) does compute the right `qvn = (vsh * phit_sh / phit).clamp(0.0, 1.0)` at `:456`, so the Juhasz rule itself exists. What does not is per-model custody: the solver applies the same porosity-volume back-out to Archie, dual water, Juhasz and Waxman-Smits alike, so the correct `Qvn` is computed and then overridden. The **inverse pair is absent entirely** - `grep` finds the forward back-out in `lrlc.rs:183`, `:365` and `sw_arch`, and no `Sw(1 - Swb) + Swb` anywhere - so there is nothing for a round-trip identity to be tested against.
- **Qualifying acceptance tests:** none for the per-model contract. Test class `CHARACTERIZATION`.
- **Manual evidence:** saturation 0/31.
- **Source/parameter boundary:** both rules and the inverse are cited verbatim in the chapter; nothing needs inventing, and no value was.
- **Blocker or decision:** `BLOCKED-BOUNDARY`. Every part of the fix lands in prohibited files. The blanket post-solve conversion, the per-model `Swb` selection and `sw_juhasz` are all in `multimin2.rs`; the other live back-outs are in `lrlc.rs`. A shared helper in an allowed file would not help, because making those call it is itself the prohibited edit. **This is the third row blocked on the same narrow `multimin2.rs` authorization** - SB-SAT-002 and SB-SAT-006 sit in the same place, and SB-SAT-002 specifically needs *this* row's inverse pair, so one authorization would move several rows at once.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes the narrow `multimin2.rs` (and `lrlc.rs`) edits on the DEC-040 pattern. Then select `Swb` per model rather than blanket, ship the inverse pair, record which rule was applied where the construction collapses them, and pin: `Swb = 1` gives `SWE = 1` and not a divide-by-zero; the round-trip is the identity; and Juhasz and Archie **disagree** on the dossier fixture (`Qvn` 0.42 vs 0.20) while `SWT` matches - the arm that proves the back-out is genuinely per model.

## SB-SAT-024

- **Specified contract:** `SWE_IRR` is an effective quantity, transformed per model. Owned test intention(s): SB-SAT-T37.
- **Current implementation:** Archie uses `SWT_IRR`; Indonesia and both Simandoux branches use `SWE_IRR`, preserving the declared quantity distinction in the standalone engine.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T37 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no exact owned test proves the effective transform per model; current branch tests use zero thresholds.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** implementation is present but lacks its source-bound regression test and solver-wide parity.
- **Next action:** add a nonzero sourced discriminator for every model and prove the threshold survives run provenance.

### SB-SAT-025 - Every method emits a clipped and an unclipped curve

- **Chapter evidence:** P1; `12_saturation.md:1385-1401`; owned test `SB-SAT-T38`; dossier 2.9.
- **Scope ruling:** DEC-048 (2026-08-16) covered `lrlc.rs`; DEC-062 (2026-08-17) opens the tree. The mnemonic pattern is the requirement's own text (`SWE_<METHOD>`) and the shipped family convention; the `_UNCL` respelling question of 7.2 item 11 stays with SB-SAT-026, deliberately undecided here.
- **Current source:** `sw_arch` gains `SWE_ARCH` beside `SWT_ARCH` - the effective diagnostic backs the UNCLIPPED SWT out and keeps its sign, since a negative value is exactly the out-of-range evidence the clipped curve erases; the `archie_effective` branch emits its raw equation value. `sw_rtc` and `sw_imts` are restructured onto the family convention: the method-named curves become the UNCLIPPED diagnostics and a plain clipped `SWT`/`SWE` pair is added carrying **bit-identically the values the method names always emitted** - no number any consumer read has changed, only its name now says what it is. `sw_imts`'s diagnostic is the converged evaluation UNPROJECTED: interior fixed points are unchanged, and a solve that converged at the bound shows how far past it the model reads.
- **Qualifying acceptance tests:** `modules::tests::every_saturation_method_emits_a_clipped_curve_and_an_unclipped_diagnostic_that_exceeds_the_bounds` - exact `SB-SAT-T38`, pinned from BOTH sides at every pair (clipped exactly at its bound AND diagnostic beyond it), because one arm alone passes a module that copies the clipped value into the diagnostic. Below-floor side pinned on `sw_arch` with `SWT_IRR` 0.3. A whole-family inventory arm sweeps every live Saturation module in both directions with retired modules excluded and a minimum pair count, so the sweep cannot silently go empty. Test class `CORRECTNESS`.
- **Mutation record:** six probes, six distinct assertion sites - clip-copied diagnostics in all three touched modules, the imts projection kept, the rtc plain pair unclipped, the arch diagnostic floored, and the unregistered pair caught by the inventory. Byte-copy restore, hashes verified.
- **Existing assertions:** the two `SWE <= SWT` assertions in `lrlc::tests` were repointed at the plain pair, which carries bit-identically the values they always compared; the diagnostics legitimately break that inequality above 1, which is the evidence they exist to carry. Nothing weakened.
- **Manual evidence:** saturation 0/41 - unexercised. Automated only.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** **two, opened 2026-08-21 by AUDIT-2026-08-20 finding 37, both TAKEN the same day on the record's own reading and logged as **DEC-092 (NEEDS-JAUHAR)** - implemented, reversible in one function, awaiting his word.** This row's `PRESENT-OK` was reached over the MODULE REGISTRY, and SandiMin is not in it - it is its own command (`run_sandimin`), so the T38 sweep, which iterates `list_modules()` and guards with a floor count, could never have seen it. SandiMin emitted `<prefix>_SWE`/`_SWT`/`_SXOT` clipped-only. (1) The mnemonic is the family pattern under SandiMin's run prefix, `<prefix>_SWE_<METHOD>`; 7.2 item 11's `_UNCL` respelling is unaffected, because it would apply to the whole family at once and here it is a change to one function. (2) The requirement does NOT reach the pure-inversion path: `linear_dw` puts the conductivity row inside the least-squares system, where the answer is bounded by the solver's hard box and by unity rather than by a clamp on an equation, so there is no discarded root to publish and an unconstrained re-solve would be a different mathematical object. `SwModel::diagnostic_token` returns `None` there and the test asserts that it does, so the exclusion is pinned rather than assumed.
- **Current source (2026-08-21, finding 37 closed):** every SandiMin model whose answer is a CLIPPED CLOSED FORM publishes `<prefix>_SWE_<METHOD>`, `<prefix>_SWT_<METHOD>` and, where the flushed zone is solved, `<prefix>_SXOT_<METHOD>` - tokens `ARCH`, `INDO`, `SIM`, `DW`, `JUH`, `WS`, one per EQUATION, which is why the two Archie branches and the two Simandoux branches share theirs exactly as the shipped modules do. Built so the pair cannot drift: each branch calls its `_unlimited` entry point and the SINGLE clamp lives at the one call site where the root is applied to the volumes, so the working curve and its twin are the same number with and without one operation. The three excess-conductivity models state their coefficient (`Swb x (Cwb-Cw)`, `QVN x (Cwsh-Cw)`, `B x Qv`) exactly once and differ only in which root solver receives it; `sw_arch`'s own back-out pair now routes through the shared `swe_from_swt` / `swe_from_swt_unlimited` instead of a second copy of the formula. Working curves are unchanged for every reading - `MAX(x, 0)` then `clamp(0, 1)` is `clamp(0, 1)`, and the conductivity root's finding-11 refusal below zero is unchanged, since a root ABOVE 1 is the ordinary wet-zone clamp while a root BELOW 0 has no saturation reading at all. Both survive unclipped in the twin.
- **Qualifying acceptance test (SandiMin half):** `sandimin::tests::sandimin_publishes_the_unclipped_twin_of_every_saturation_curve_it_clips` - end-to-end against a real solver run, the same fixture twice at the true Rw and at an Rw a decade high. Three arms, each defeating a different lazy implementation: out of range the twin EXCEEDS the bound where the working curve sits on it (a copied value fails); in range the twin EQUALS its working curve (a different equation fails); and where no closed form ran - this fixture supplies no flushed-zone resistivity, so the X split comes from the linear inversion - the twin is MISSING (writing the inversion's own split there would state that a model ran and stayed in range). A fourth arm pins `linear_dw` to no token. T38 arm F carries the structural half: no floor, so a solver model added without a twin fails it.
- **Next action:** SB-SAT-026 owes `sw_rtc`/`sw_imts` their `SW_METHOD` flags and owns the `_UNCL` nomenclature question. SB-SAT-025 itself is now `MET` in the chapter; if `_UNCL` is later adopted it applies to the whole family at once, and on this side it is a change to `SwModel::diagnostic_token` alone.

## SB-SAT-026 - No bare SW; every saturation curve designated; a method flag on every run

- **Specified contract:** no emitted mnemonic equals bare `SW`/`SXO`; every saturation curve carries an E or T designator; every run emits a method-flag curve resolvable through the shared alias table; `VOL_UWAT`/`VOL_XWAT` accompany.
- **Current implementation (2026-08-18):** DONE. The flag-coverage half shipped 2026-08-17 (codes 10/11/12, one shared registry, both universal pins). **DEC-064** closed the naming ruling - "it principally SWT" - so the height module now follows the family pattern: limited `SWT` (exactly the values the single `SWH` output always carried) plus the SB-SAT-025 unclipped `SWT_HGT` diagnostic. **`VOL_UWAT` landed on the LRLC pair** (DEC-048 narrow authorization): the effective-volume identity `PHIE x limited SWE` with `PHIE = PHIT - CBW`, degenerating to `PHIT x SWT` without CBW, MISSING where the saturation is - matching the Archie family. `VOL_XWAT` stays not applicable (no flushed-zone module ships). **T40's persistence half is proven**: `SW_METHOD` survives write, reload and LAS export bit-exact as a categorical, absences stay MISSING, every surviving code resolves through the registry.
- **Qualifying tests:** `the_sw_method_flag_survives_write_reload_and_las_export_as_the_categorical_it_is` (export.rs), `vol_uwat_is_the_effective_volume_identity_on_both_lrlc_modules` (lrlc.rs, with clip-binding samples following the LIMITED curve), `the_height_diagnostic_keeps_what_the_fit_said_while_swt_clips` (satheight.rs). Five mutations killed on distinct assertions; two initial survivors exposed fixture gaps that were closed with clip-binding arms before the set was re-run clean.
- **Verdict:** `PRESENT-OK`; Gate 2 DONE 2026-08-18 @ codex/g2-program-plan (pre-PR).

## SB-SAT-027 - One shared root-finder for every polynomial-form saturation model

- **Specified contract:** every polynomial-form Sw model solves through ONE shared, guarded root helper; a closed form MAY serve as an n = 2 fast path provided it is asserted equal to the general solver (`12_saturation.md:1425-1431`); Geolog's literal guards named as the reference procedure.
- **Current implementation (2026-08-18):** DONE. One engine by construction: `sw_sim` delegates to `multimin2::sw_simandoux_*`, both call the single `solve_simandoux_root`, 15 `modules.rs` sites route into `multimin2`. The n = 2 closed form is asserted equal to the general root finder engine-against-engine. The bisection-for-Newton substitution (60-step bisection on `[0, 1]` after proving monotonicity - unconditionally convergent where Newton from a fixed seed is not) was put to Jauhar as the one remaining method call and ACCEPTED: **DEC-065, RULED 2026-08-18**. The chapter clause is amended by that ruling; the ruling record lives in DECISIONS.md and the test doc, and no solver was rewritten.
- **Qualifying test:** `modules::tests::the_n_equals_two_closed_form_agrees_with_the_general_root_finder_on_the_same_inputs` - four cases straddling the `|n - 2| < 1e-9` fast-path guard, interior root required so agreement is never between two clamps; mutation-proved 2026-08-17. Test class `CORRECTNESS`.
- **Verdict:** `PRESENT-OK`; Gate 2 DONE 2026-08-18 @ codex/g2-program-plan (pre-PR).

## SB-SAT-028 - Non-convergence returns null, never a partial iterate

- **Specified contract:** a saturation solver that fails to converge within its iteration budget **MUST** return null for that sample. SandiBumi **MUST NOT** emit the last iterate of a non-converged solve (`12_saturation.md:1399-1410`). Geolog sets `sat = MISSING` on non-convergence.
- **Why P0:** a partial iterate is **indistinguishable from a converged answer on the log**. It is not a visible error - it is a plausible number in the right range that a petrophysicist will read, map and book. This is the silent-failure class CONTRACT SS5.3 and IP FINDINGS rule 14 both target.
- **Current implementation - VERIFIED IN CODE, the finding stands exactly as written.** `sw_imts` in `lrlc.rs` runs `for _ in 0..100 { ... }` and then writes `swt_o[i] = sw as f32;` **unconditionally**. There is no convergence flag and no guard on falling out of the loop; only a NaN from a non-positive denominator is caught. The contrast is inside this same repository: `gascorr` (`modules.rs:4539-4556`) sets a `converged` flag and `continue`s on failure, with a comment stating that writing the last pass would be *an internally inconsistent triple masquerading as a converged answer* - which is precisely the defect `sw_imts` has. **SandiBumi's own in-house method has the defect its vendor-derived module avoids.**
- **Qualifying acceptance tests:** none. Test class `MISSING`. No existing test could catch it: a non-converged iterate is a finite number in range, so every assertion about finiteness or bounds passes.
- **Manual evidence:** saturation 0/31.
- **Source/parameter boundary:** no parameter is involved and no tolerance needs choosing - the module already has an iteration budget and a convergence test; what is missing is *acting* on the failure.
- **Blocker or decision:** `BLOCKED-BOUNDARY`. The defect and its fix are both inside `lrlc.rs`, a prohibited file, and the fix is small and needs no decision: track convergence and leave the sample `f32::NAN` when the budget is exhausted, exactly as `gascorr` already does. This is the **sixth** row waiting on the same narrow authorization.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes the narrow `lrlc.rs` edit. Then make `sw_imts` leave a non-converged sample MISSING, and pin from both sides: a sample that converges keeps its value, and a sample driven past the budget comes back `NaN` rather than a plausible last iterate. One arm alone would pass a module that nulls everything.

## SB-SAT-029 - Inherit the documented guard rails, including the volume detail

- **Specified contract:** `phie < 0.005` => all saturations 1 **and `VOL_UWAT = VOL_XWAT = phie`, not 0**; `phie = phit = 0` => all saturations 1, all volumes 0; `Rt` missing or <= 0 => every saturation output null; a missing variable-`m` input curve => every output null with a message (`12_saturation.md:1412-1425`).
- **Why the volume detail matters:** setting volumes to 0 there would silently zero bulk-volume water over tight streaks that still carry porosity. The interval is declared **wet**, not declared **empty** - two different answers that look identical in a summation (dossier MN-4).
- **Current implementation:** `PRESENT-OK`. The guards are in place across the standalone modules, including `VOL_UWAT = phie` at the low-porosity rule. Rule 4 is **vacuous by construction** - no variable-`m` route exists - so it is deliberately not asserted rather than faked with a placeholder. The row was a **PROVE**.
- **Qualifying acceptance tests:** `every_standalone_saturation_guard_declares_a_tight_streak_wet_rather_than_empty` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Source/parameter boundary:** all four rules are Geolog's documented behaviours, the low-porosity rule appearing in **all nine** `sw_*` modules. The 0.005 threshold is quoted from the chapter, not chosen.
- **Three-armed pin, and arm B is what makes arm A honest.** (A) across **every** standalone saturation module - `sw_arch`, `sw_indo`, `sw_sim` - a tight streak at `phie = 0.002` is all water AND its `VOL_UWAT` equals that porosity, never 0. (B) at `phie = phit = 0` the volume genuinely IS 0 - so a module that returned `phie` unconditionally would pass arm A and fail here, which is the lazier implementation arm A alone would admit. (C) `RT` at 0, negative, or missing nulls every saturation output rather than emitting an infinity.
- **Verified by mutation:** flipping the tight-streak rule so `VOL_UWAT` comes back 0 fails the test - that mutation is precisely MN-4, and it produces a plausible zero rather than an error.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that a tight streak with a little porosity reports bulk-volume water equal to that porosity rather than zero.

## SB-SAT-030 - Vsh -> 1 flags before the singularity

- **Specified contract:** when `Vsh -> 1` in `simandoux_modified_slb` (whose `1/(1-Vsh)` term is singular) or in `indonesia` (where water and effective porosity both go to zero), SandiBumi **MUST** raise a flagged condition. It **MAY** additionally return `Sw = 1`; it **MUST NOT** return `Sw = 1` unflagged (`12_saturation.md:1427-1440`). Techlog Elan is the only vendor documenting this failure mode.
- **What changed:** the values are **unchanged** - all-water is permitted and still returned. What was added is the flag. `sw_sim`'s Schlumberger branch previously set all-water for `VSH >= 1` with the reason in a **source comment and nothing in the output**; it now records a `Clamped` degradation naming the condition. `sw_indo` gained the same flag, placed **above** its low-porosity early return so the degenerate case is reported rather than absorbed by the tight-streak rule.
- **Qualifying acceptance tests:** `a_pure_shale_saturation_is_flagged_rather_than_quietly_returned_as_water` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Supporting tests:** `sw_sim_schlumberger_pure_shale_is_all_water` (CHARACTERIZATION, moved out of the qualifying register when this row's class became CORRECTNESS - the test was not deleted, and it still pins the VALUE this row deliberately left unchanged).
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Source/parameter boundary:** no parameter and no threshold was introduced. `RunDegradationKind` was **not** extended - `Clamped` is one of the four existing members, and CLAUDE.md states that adding a fifth is a contract change rather than an ad-hoc message choice.
- **Three-armed pin.** (A) the condition is raised, for both models. (B) the answer is still all water, so the flag did not come at the cost of the value the chapter permits - arm A alone would pass a module that flagged and then emitted something else. (C) a clean sand comes back **unflagged**, or the condition carries no information.
- **A false assumption of mine was caught by the test and corrected.** The first draft asserted all-water for `indonesia` at `VSH = 1` with `PHIE = 0.10`. It returns a **computed 0.373** there. The chapter is explicit that indonesia's degeneracy is where water **and effective porosity** both go to zero, so pairing `VSH = 1` with a healthy porosity asserts a physically inconsistent sample rather than the documented case. Each module now gets the case its own failure mode describes, and the reasoning is recorded in the test.
- **Verified by mutation:** weakening the flag so it no longer names the `VSH >= 1` condition fails the test - a flag that does not say what happened is the same silence in a different costume.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that a pure-shale interval reports the clamped condition in the run record rather than passing as an ordinary all-water result.

## SB-SAT-031 - Rw ships with no default

- **Specified contract:** `Rw` **MUST** ship as `NoDefault` in every saturation module and in the solver. SandiBumi **MUST NOT** inherit IP's `0.1` or Techlog's `0.03`, and **MUST NOT** substitute a value derived from a formation-water environment band (`12_saturation.md:1442-1456`).
- **Why P0:** IP's 0.1 and Techlog's 0.03 differ by **1.83x on Sw** at m = n = 2. IP at least warns that it *must be adjusted to the correct value*; Techlog does not. The dossier explicitly **withdrew** a project-kb `Rw ~ 0.21` as unsound corroboration - cross-basin, ambiguous header, and three salinity methods disagreeing at 24/12/25 kppm in the same record - so no default rests on it either.
- **Current implementation - the as-built is stale on BOTH counts, verified in code.** It said `modules.rs` ships 0.1 and `lrlc.rs` ships 0.3, leaving SandiBumi's two engines `sqrt(3) = 1.73x` apart on Sw before the user touches anything. Neither holds: `modules.rs:4769` uses `param_open_when` and `lrlc.rs:94` and `:220` both use `param_open` - the defaultless family throughout. The row was a **PROVE**.
- **Qualifying acceptance tests:** `no_saturation_engine_ships_a_formation_water_resistivity_the_user_did_not_supply` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Source/parameter boundary:** no value was adopted, and the test names the four numbers that must never appear as a default - IP's 0.1, Techlog's 0.03, the 0.3 the as-built reported, and the withdrawn 0.21.
- **Three-armed pin.** (A) every `RW`/`RWS` argument on every Saturation-category module has an EMPTY default and declares `ABSENT_DEFAULT_SOURCE` - blank alone is not enough, the absence must be declared. (B) a count guard requires at least three such arguments, so the loop cannot pass vacuously if the category is ever renamed or the arguments are moved. (C) the SOLVER proves it differently and more strongly: `FluidProps` **refuses to deserialize** without `rw`, and the refusal must name `rw` - a caller cannot forget to set it, which is a better guarantee than a blank field.
- **Verified by mutation:** giving the saturation modules IP's rejected `0.1` fails the test.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that a saturation run with no Rw supplied refuses or reports the parameter as absent, rather than quietly producing a curve.

## SB-SAT-032

- **Specified contract:** `Rw` correlations with the temperature conversion bound to the branch. Owned test intention(s): `SB-SAT-T45`, `SB-SAT-T46`, `SB-SAT-T48`.
- **Current implementation:** measured, Kennedy and Bateman-Konen branches use their branch-specific temperature conversions and switch at the specified salinity.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T45`, `SB-SAT-T46`, `SB-SAT-T48` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no exact source-cited switch, continuity and degree-scale regression test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the implementation is present but unprotected against branch/temperature drift.
- **Next action:** add sourced tests on both sides of the switch and bind Celsius/Fahrenheit conversions to their branches.

### SB-SAT-033

- **Specified contract:** The Kennedy floor is 0.0412 and the vendor doc is wrong. Owned test intention(s): SB-SAT-T47.
- **Current implementation:** the Kennedy high-salinity branch uses the required 0.0412 floor.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T47 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no exact floor-versus-factor-ten anti-fix test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the correct literal is integrated but has no owned regression proof.
- **Next action:** add the source-cited floor test and a factor-ten negative control.

### SB-SAT-034 - a, m, n, m*, n* ship with no default

- **Specified contract:** `a`, `m`, `n` and the Waxman-Smits/dual-water `m*`, `n*` **MUST** ship as `NoDefault` - a first-class state distinct from any numeric value. A run requesting a saturation model without them **MUST** fail with a message naming the missing parameter (`12_saturation.md:1470-1487`).
- **Why P0:** IP's own manual states **no default for a/m/n at all** - the 1.0/2.0/2.0 commonly quoted are Basic Log Analysis values only. A cementation exponent is a rock property measured on core, and Jauhar's own delivered studies use SCAL-derived values per field. The chapter's phrase: **a shipped exponent is the highest-consequence silent default in petrophysics.**
- **Current implementation - the as-built is stale for every module, verified in code.** It reports `modules.rs:1991-1993` shipping `A 1.0 / M 2.0 / N 2.0`, `lrlc.rs:94-95` shipping `M 2.0 / N 2.0` and `lrlc.rs:203-204` shipping `MSTAR 1.9 / NSTAR 1.9`. None holds. Every one of those declarations now uses `param_open` - the defaultless family - at `modules.rs:4409-4411`, `:4855-4857`, `:4980-4982`, `:5109-5111` and `lrlc.rs:102-103`, `:229-245`. The `1.9` still visible at `lrlc.rs:1283` is a **test fixture**, not a shipped default.
- **What DID remain, and what was changed:** the solver defaulted `archie_a = 1.0` (`multimin2.rs`, wired by `#[serde(default = "default_archie_a")]`). Its doc note said the dual-water models use `a = 1`, which is a physical constant for those forms - but the same field serves Indonesia and Simandoux in the solver, where `a` is a free parameter, so the default was live for models the chapter forbids it on. **That is the hardest kind of default to spot: one that is right for the most common caller.** The attribute and its helper are removed, so `FluidProps` now refuses to deserialize without `archie_a`, exactly as it already refuses without `rw`. `src/ui/multiminDialog.ts` sent `Number(archieAInp.value) || 1`, which converted an empty box into a silent `1` before the backend ever saw it; it now OMITS the field when the box is empty, so an empty box reaches the refusal.
- **Qualifying acceptance tests:** `no_saturation_engine_ships_a_cementation_or_tortuosity_exponent_the_user_did_not_supply` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`. Arm A sweeps every `Saturation` module for `A`/`M`/`N`/`MSTAR`/`NSTAR` and requires each to carry an empty default AND declare `ABSENT_DEFAULT_SOURCE`, refusing to pass vacuously on fewer than eight such declarations. Arm B deserializes a `FluidProps` payload with every other field present and `archie_a` omitted, and requires the refusal to NAME the missing parameter - the clause the chapter states explicitly, and a stronger guarantee than a blank manifest field, because a caller cannot forget to set a required field.
- **Mutation evidence:** two probes, each read for WHICH assertion fired. Restoring `#[serde(default = "default_archie_a")]` turned arm B RED on *the solver accepted a fluid model with no tortuosity factor*. Giving one module's `M` a numeric `2.0` turned arm A RED on `assert_eq!(arg.default, "")`. A first attempt at that second probe was DISCARDED as no proof at all: it went red on the SB-CORE-004 source gate rather than on this test's own assertion.
- **Manual evidence:** saturation 0/31. Automated only; no manual or field evidence is claimed.
- **Source/parameter boundary:** no value was adopted. The four numbers this row must never ship - `1.0`, `2.0`, `1.9` and the solver's `archie_a` 1.0 - are recorded as forbidden, not as candidates.
- **Blocker or decision:** cleared. DEC-048 (2026-08-16) authorized the narrow `multimin2.rs` edit; this is the first row landed under it.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Next action:** Jauhar field-verifies that a SandiMin run with the `a` box left empty refuses and names `archie_a`, rather than quietly solving with `a = 1`.

## SB-SAT-035

- **Specified contract:** `Rsh` and `φt_sh` ship with no default, and the current values are withdrawn. Owned test intention(s): `SB-SAT-T31`, `SB-SAT-T50`.
- **Current implementation:** Rsh and shale porosity ship concrete withdrawn defaults and Results QC consumes them automatically.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T31`, `SB-SAT-T50` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** current spread tests use explicit/default shale values; no no-default or range-policy test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** withdrawn values still compute and can appear authoritative.
- **Next action:** remove both defaults, implement the cited accept/warn/reject policy, and record the user's source.

### SB-SAT-036

- **Specified contract:** Two named `m*`/`n*` routes, with core preferred. Owned test intention(s): SB-SAT-T51.
- **Current implementation:** no separate core-derived and Qv-derived m-star/n-star routes or core-preferred precedence exist.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T51 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no route-selection test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion is undecided and source-owned exponent routes are absent.
- **Next action:** if included, implement two named routes, explicit availability metadata and deterministic core preference.

### SB-SAT-037

- **Specified contract:** Shell / Elan variable `m` as one parameterised route with no default coefficient. Owned test intention(s): `SB-SAT-T31`, `SB-SAT-T52`.
- **Current implementation:** no Shell/Elan variable-m route or absent-by-default coefficient exists in current source, tests or reachable code history.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T31`, `SB-SAT-T52` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no variable-m acceptance test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion and coefficient custody are undecided.
- **Next action:** if included, add one parameterized cited route with no coefficient default and explicit missing-input refusal.

### SB-SAT-038 - Every parameter carries a source string, and the build fails without one

- **Specified contract:** every saturation parameter **MUST** resolve to either a value with a **non-empty source string** or the explicit `NoDefault` state. A default with an empty source **MUST fail the build**. The source **MUST** be a specific checkable reference - a file and section, a module and parameter name, or a full literature citation (`12_saturation.md:1522-1537`).
- **Why P0:** CONTRACT SS2 makes this the rule that outranks everything else. The domain's own evidence is the argument: three vendors ship three `Rw` defaults, three `B` method defaults, two `vQ0` values from the same paper, and a Simandoux `a` no cited paper supports - and none of them tells the user. A plausible-but-wrong endpoint computes, plots and ships into a reserves number without failing.
- **Current implementation - the as-built was stale in both of its claims.** It says *no parameter carries a source string* and *`ArgSpec` has no field for one*. `ArgSpec` has `default_source`, and `validate_parameter_sources` (`modules.rs:1953`) already gated EVERY module for three of the four rules: non-empty source, `ABSENT` consistency (a source of ABSENT may not ship a default), and a finite numeric default wherever a source is cited.
- **What this increment changed:** the fourth rule - the **checkable-artefact** test - was scoped to `category == "VSH"` alone. It now also covers `Saturation`. The whole shipping catalogue already satisfies it: the full suite passed unchanged at **1020 tests, 0 failures**, so this closed a contract without moving a single number.
- **Qualifying acceptance tests:** `a_saturation_default_without_a_checkable_source_fails_the_build` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Manual evidence:** none needed - this is a build-time gate, not a screen.
- **Source/parameter boundary:** no value was adopted or changed. The rule is about whether a source can be CHECKED, not about what it says.
- **Four-armed pin.** (A) the shipping catalogue passes the stricter rule today. (B) a **bare product name** is refused - the clause that makes the rule bite, because that is exactly how an uncited vendor default looks when somebody writes it down in good faith - and the refusal must say *why*, so the fix is obvious. (C) an empty source is refused even though the number looks ordinary. (D) a proper file-and-section citation is **accepted** - without this arm the rule could block everything and teach the next author to route around it.
- **Verified by mutation:** scoping the rule back to `VSH` alone fails the test.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** none for Jauhar. Worth knowing for later rows: this gate now refuses an uncited saturation default at build time, so a future parameter cannot ship one by accident.

## SB-SAT-039

- **Specified contract:** `MUDBASE` is model-scoped. Owned test intention(s): SB-SAT-T53.
- **Current implementation:** no MUDBASE option exists in saturation production code or reachable code history.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T53 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no scoped positive/negative MUDBASE test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion and the documented method subset are undecided.
- **Next action:** if included, add MUDBASE only to the documented models and prove rejection everywhere else.

### SB-SAT-040

- **Specified contract:** Clay-bound-water `F`: both unit forms, `ρ_brine` open, `Swb = 1 − F` opt-in only. Owned test intention(s): `SB-SAT-T54`, `SB-SAT-T55`.
- **Current implementation:** no dual-form F bridge, typed brine density or explicit opt-in `Swb=1-F` conversion exists.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T54`, `SB-SAT-T55` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no unit-bridge or opt-in test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion is undecided and brine density remains intentionally open.
- **Next action:** if included, implement both cited F forms; keep brine density absent and require explicit conversion opt-in.

### SB-SAT-041

- **Specified contract:** Poupon-Aguilera / Poupon-Tixier with the laminated interlock. Owned test intention(s): `SB-SAT-T56`, `SB-SAT-T57`.
- **Current implementation:** Poupon-Aguilera and Poupon-Tixier saturation methods are absent from current production code and reachable history.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T56`, `SB-SAT-T57` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no method-distinction or laminated-interlock test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** held outside the current pilot until source and product inclusion are separately decided.
- **Next action:** retain as later work; if admitted, implement distinct cited methods and refuse violation of the laminated interlock.

### SB-SAT-042

- **Specified contract:** The SSM bound-water cap fires and is flagged. Owned test intention(s): SB-SAT-T58.
- **Current implementation:** no SSM bound-water cap, total-porosity reset or observable cap flag exists.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T58 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no cap-fired versus not-fired test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion is undecided and the firing point remains owned by the POR domain.
- **Next action:** if the SSM path enters the pilot, implement the POR-owned cap and carry its flag into saturation provenance.

### SB-SAT-043

- **Specified contract:** every saturation run **MUST** emit, alongside its curves, a machine-readable record of the model identifier, every parameter value used, each value's source string, the literature citation the method traces to, and the Worthington 1985 type where one is stated by a source; for the LRLC methods an explicit unfitted-coefficient flag; **zero fields empty**; and that record **MUST** survive export into the deliverable (`12_saturation.md:1776-1795`, SB-SAT-T59 at `:2523-2532`).
- **Why P0:** Geolog ships published references inside every module manifest, and **no vendor carries the reference through to the answer** (`:481-484`). A parameter that carries the paper it came from, through the computation, into the deliverable is a claim no incumbent can make - and it is the only thing that makes SB-SAT-038's build-time source gate auditable downstream rather than only at compile time.
- **The as-built was half stale, and the half that held is the interesting half.** `CurveAncestry` already carried the model identifier, every parameter value and every value's source; already refused to be switched off; and already reached the PDF, Word, workbook and deck through `curve_ancestry_disclosures`. What was genuinely absent was the LITERATURE - the citation, the Worthington type, and any statement of the LRLC coefficients' calibration standing. This row is therefore not a new record; it is the paper finally travelling on the record that already existed.
- **Current implementation:** `SaturationMethod {module, method_id, citation, worthington, worthington_source, caution}` plus the `SATURATION_METHODS` registry, the `RETIRED_METHOD` / `METHOD_OWNED_ELSEWHERE` / `WORTHINGTON_NONE_STATED` tokens, `validate_saturation_methods` and the `saturation_method` lookup, all in `param_sources.rs`; the gate runs at catalog build in `modules.rs` beside `validate_parameter_sources` and `validate_domain_defaults`, so a violation panics the build. `workflow.rs` gains `saturation_method_id` (replacing the inline match, and now covering `sw_rtc` and `sw_imts`), `lrlc_calibration_coefficients`, and four run-provenance keys pushed into the existing ancestry parameter list - `method_citation`, `worthington_1985_type`, `method_attribution_caution`, `unfitted_coefficients`.
- **Nothing in `ancestry.rs` changed, deliberately.** The four fields ride the parameter list as derived metadata, which is the pattern `MASK`, `FLAG_KIND.*`, `OUTPUT_QUANTITY.*` and `POROSITY_OUTPUT.*` already established, each with the same reserved-key collision guard. `cells()` renders every parameter as `name=value [source: ...]` and summarizes nothing away, so the citation reached every deliverable surface without one new export path - which is also why the export half of T59 needed no new plumbing.
- **The gate is over the CATEGORY, not a list.** `validate_saturation_methods` walks every module whose category is `Saturation` and fails the build for any that is unregistered. Same reasoning as SB-CUT-018's pane enumeration: a hand-kept list goes stale the day somebody adds a model, and the model that ships without a citation is exactly the one nobody remembered to add.
- **`NONE-STATED` is a word, not a null, and that is load-bearing twice.** `CurveAncestry::validate` refuses a parameter with no recorded value - an existing guard, not weakened here - but the better reason is that *"no source classifies this model"* and *"nobody recorded a classification"* are different claims and only one is checkable. Archie's record carries the field, states `NONE-STATED`, and its source names what was consulted to reach that.
- **Qualifying acceptance tests:** `a_saturation_run_carries_its_model_citation_worthington_type_and_unfitted_coefficient_flag_into_the_deliverable` (`src-tauri/src/paysummary.rs`). Test class `CORRECTNESS`. Six arms: the Indonesia leg names the paper and carries Worthington type 4; zero recorded fields are empty; Archie SAYS it has no classification rather than omitting the field; an LRLC run states its coefficients' calibration standing; the record survives into the exported disclosure cells; and the build gate refuses a bare author name, a silent Worthington field, an unexplained hand-off token, and an unregistered saturation module.
- **Mutation evidence:** six probes, each read for WHICH assertion fired, all six landing on distinct assertions. Renaming the pushed citation key turned the lookup red, naming the missing field. Registering Indonesia as type 2 turned the classification arm red. Making the absent classification an empty string instead of the token turned the Archie arm red - the *zero fields empty* clause, defeated and caught. Emptying the LRLC coefficient list turned the flag arm red. Disabling the category-coverage clause let an unregistered `sw_indo` pass and turned the orphan arm red. Disabling the checkable-artefact clause let the bare string `"Archie"` pass as a citation.
- **One probe was void and was re-run.** The `wrongtype` revert anchored on `worthington: Some(2)`, which matches the two Simandoux rows as well as the mutated Indonesia one, so it reverted nothing and the next four probes all fired at the same stale assertion. The anchor now carries the module identity, every probe verifies its own revert, and the six results above come from the repaired run.
- **NOT mutated, and stated rather than glossed:** the export arm. What it checks is `CurveAncestryDisclosure::cells()`, which lives in `ancestry.rs` - a protected file - so no mutation of the rendering was available to this lane. The assertion is real; its mutation is not.
- **What the LRLC flag HONESTLY says.** The chapter's premise for it - *"a run using an unreplaced shipped coefficient"* - describes a state that no longer exists: `A_CAP`/`B_QV`/`C0`/`RSF` and `S_FACTOR` are all `param_open`, so nothing ships as a default and `lrlc.rs`'s own doc string says so. The flag therefore reports the stronger fact, `NO_SHIPPED_DEFAULT`, per coefficient, and would report `SHIPPED_DEFAULT_IN_USE` the moment a default were reintroduced.
- **Named limit, carried IN the record rather than only here:** `ENTERED` does not distinguish a coefficient accepted from this project's own RtC/S fit from one typed by hand, because `db::set_zone_param_batch` writes `value_text = NULL`, so a fitted coefficient arrives with no source custody. `db.rs` is protected, so fitted-versus-entered is **not claimed** - the flag says so in its own `limit` field, where a reader of the deliverable sees it rather than only a reader of this dossier.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** no citation invented. The chapter pairs papers to MODULES (`:469-479`) and equations to BRANCHES (`:137-142`) and never pairs a paper to a branch, so both Simandoux entries carry the module's References block and the equation identity separates them; Schlumberger 1989 appears in Geolog's reference set but is attached to the `SCHLUM` branch by no source, so it is not claimed. IMTS is not given `sw_ws`'s type 2: no source classifies SandiBumi's own scaling of that family, and carrying a neighbour's number across would be a classification nobody published.
- **A disputed attribution travels with the citation.** IMTS's entry carries the ESC-1 caution in substance: IP attributes the clay-bound-water relation to Hill, Shirley & Klein 1979 Paper AA while Geolog attributes a paper of that exact title, same symposium, same year, same letter, to Juhasz, and Techlog cites Juhasz 1979 *The Log Analyst* p 3-14. Both readings ship and neither is chosen (`:486-491`), so the record states the dispute rather than resolving it on the user's behalf.
- **UI/IPC/provenance surface:** every run of `sw_arch`, `sw_indo`, `sw_sim` (both equations), `sw_rtc` and `sw_imts`. `sw_height` and the retired `multimin` are registered so the gate accounts for them but get no citation pushed into a run record: saturation-height's literature and its fitted-object provenance belong to `15_sat-height-rocktyping.md`, and printing this chapter's hand-off token in a client deliverable would say less than nothing.
- **History/reachability:** the record-emitting path existed and was integrated; only the literature was missing from it. The as-built's claim that export "carries that generic ancestry but not parameter sources" was stale - sources were already carried; papers were not.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** cleared.
- **Next action:** Jauhar field-verifies that a saturation run's report carries a "Computed curve ancestry" section naming the paper the equation traces to, and that the Simandoux entries state which of the two equations ran. Two neighbouring rows are OUTSIDE the Gate 2 manifest and were not done here: SB-SAT-049 (carry the Worthington type as model metadata and expose it in a chooser) and SB-SAT-048 (present the LRLC coefficients as one field's calibration). The record built here already carries both fields, so those rows are about the UI surface rather than the record.

### SB-SAT-044

- **Specified contract:** Surface the cross-tool disagreement to the interpreter. Owned test intention(s): SB-SAT-T60.
- **Current implementation:** Results QC displays a numeric envelope and divergence summary across its local helper models, but not equation/parameter causes, source conflicts or cross-engine custody.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: all eight Results-QC exact tests passed for current model-spread behavior.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the interpreter sees disagreement magnitude without enough identity/cause evidence to adjudicate it.
- **Next action:** surface canonical model IDs, exact parameter/source differences and recorded capability gaps beside the spread.

### SB-SAT-045

- **Specified contract:** Model-selection guidance, exposed as guidance and never as an automatic switch. Owned test intention(s): SB-SAT-T60.
- **Current implementation:** UI text says model choice matters, but no source-backed selection guidance exists; current defaults still preselect models.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T60 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** Results-QC tests prove spread display, not guidance presence and absence of automatic switching.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** guidance and selection are not separated as an observable contract.
- **Next action:** add source-backed guidance that never mutates the selected method and test both presentation and no-auto-switch behavior.

### SB-SAT-046

- **Specified contract:** Sxo and the flushed zone. Owned test intention(s): SB-SAT-T61.
- **Current implementation:** the solver can produce total flushed saturation `SXOT`, but standalone Sxo methods, effective Sxo, mud-base limits, flags and both flushed-zone water volumes are absent.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `xu_split_recovers_sw_and_sxo_from_conductivity` passed for the current solver split.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the user-required coupled Sxo/Sw path lacks a complete typed output and failure contract.
- **Next action:** complete the source-bound iterative Sxo/Sw contract with effective/total outputs, volumes, flags, convergence and mud-base guards.

### SB-SAT-047 - One model, one number, whichever engine computes it

- **Specified contract:** a named saturation model **MUST** return the same value from the deterministic module and from the mineral solver, given the same inputs and parameters, to a **stated tolerance**. Where the two contexts genuinely differ, the difference **MUST** be documented at the model level and asserted by a test that names it, not left to a source comment (`12_saturation.md:1832-1846`).
- **Why P0:** the product was failing this in the most expensive possible way - the two engines computed **different Simandoux equations under the same name, 7.3 saturation units apart**. This is a SandiBumi-internal requirement with no vendor counterpart.
- **Current implementation:** the engines now agree. `SB-SAT-001` closed the naming half earlier this session; this row closes the half naming alone cannot - that the NUMBERS match. Every solver equation is already public (`sw_archie`, `sw_simandoux_bardon_pied`, `sw_simandoux_modified_slb`, `sw_indonesia`, `sw_juhasz`, `sw_waxman_smits`), so the cross-assertion needed no change to `multimin2.rs` at all - only a test that calls both sides.
- **Qualifying acceptance tests:** `a_named_saturation_model_returns_one_number_from_either_engine` (`src-tauri/src/modules.rs`). Test class `CORRECTNESS`.
- **Manual evidence:** none yet - Jauhar owns the field check.
- **Tolerance is STATED, not implied:** `1e-6` in saturation units. Both engines solve the same closed forms in `f64`, so anything looser would hide a real divergence and demanding bit equality would fail on operation ordering alone. The chapter asks for a stated tolerance and this is it.
- **Three-armed pin, and arm C is the one that matters.** (A) Archie agrees between engines - compared on the module's UNCLIPPED diagnostic, since the clipped curve carries irreducible-saturation bounds the solver form does not know about. (B) both Simandoux forms agree, each against its OWN counterpart. (C) the two Simandoux forms are genuinely DIFFERENT numbers on the sample - because without it, arm B would still pass if both engines had collapsed onto one equation, which is precisely the original failure just relocated.
- **Verified by mutation:** swapping `sw_sim`'s branch so each option computes the other equation fails the test - that mutation IS the 7.3-saturation-unit bug.
- **Verdict:** `PRESENT-OK`; `DONE`; test class `CORRECTNESS`; commit state `INTEGRATED`.
- **Blocker or decision:** none.
- **Next action:** Jauhar field-verifies that a Simandoux run through SandiMin and the same run through the module report the same SWE on a real well.

## SB-SAT-048

- **Specified contract:** LRLC coefficients are declared as one field's calibration. Owned test intention(s): `SB-SAT-T59`, `SB-SAT-T62`.
- **Current implementation:** RtC and IMTS disclose placeholder/study-derived coefficients in documentation and explicit fits exist, but unfitted runs emit no observable calibration-status flag and generic provenance omits fit state.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: synthetic inverse-fit and interval-selection tests passed; they prove fit plumbing, not independent coefficient validity.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-UNVERIFIED`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** placeholder-versus-fitted state is not machine-visible through the result and export.
- **Next action:** attach declared calibration identity, fitted interval scope, held constants and an unfitted warning flag to every LRLC run.

### SB-SAT-049

- **Specified contract:** Carry the Worthington 1985 type per model. Owned test intention(s): SB-SAT-T59.
- **Current implementation:** no per-model Worthington type exists in runtime metadata, UI, saved runs or export.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T59 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** generic export provenance does not carry this classification.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `DEFERRED`; `LATER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** held outside the current pilot as later metadata work.
- **Next action:** retain as later work; when admitted, add machine-readable per-model type to the canonical registry and export.

### SB-SAT-050

- **Specified contract:** Apparent-`Rw` inversion, one per saturation model. Owned test intention(s): SB-SAT-T63.
- **Current implementation:** no per-model apparent-Rw inversion exists in production code, tests or reachable history.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T63 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no independently checked inversion test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `UNDECIDED`; `REQUESTED-CAPABILITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** pilot inclusion is undecided; each inversion depends on stable canonical forward models and no-default Rw custody.
- **Next action:** after model identities stabilize, decide pilot inclusion and implement one independent inversion per admitted model.

### SB-SAT-051

- **Specified contract:** Per-mineral conductivity is a recorded capability gap, not a silent one. Owned test intention(s): SB-SAT-T60.
- **Current implementation:** conductive minerals can be represented as components, but affected saturation results do not record that per-mineral conductivity is unmodelled.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T60 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no run-level limitation-disclosure test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** the omission can silently bias saturation where a conductive mineral is present.
- **Next action:** detect affected runs and carry a machine-readable limitation through UI guidance, provenance, report and export.

## Test-intention summary

- All 63 chapter intentions route exactly once through the plan's primary-ownership table. Requirements `049` and `051` retain shared support from T59/T60; no test ID was invented.
- Fifty-two exact current candidate tests passed: 9 standalone/temperature candidates, 18 solver/fluid/Sxo candidates, 14 LRLC candidates, 8 Results-QC candidates, 2 workflow candidates and 1 export candidate. The command-level total is 52 passed / 0 failed.
- None is a complete source-named owned acceptance proof for a whole SB-SAT contract. Fourteen rows are therefore `CHARACTERIZATION`; thirty-seven are `MISSING`; zero are `CORRECTNESS`.
- Round trips generated from the same forward equation, synthetic inverse fits, local output-key checks and generic ancestry export are supporting evidence only.

## Parameter and source summary

- All 71 chapter parameter rows were traced across standalone manifests, solver structs, LRLC specs, UI serialization, stored runs, Results QC and export.
- Twenty rows carry an ABSENT state and eight have no tier. No value, range, ceiling, source or default was supplied by this adjudication.
- Current source still ships conflicting Rw defaults, concrete Archie-family exponents/factors, withdrawn shale inputs, automatic B selection, an untyped B/Qv boundary and a hard alpha ceiling. These are recorded as shipped behavior, never as authority.
- The parameter-source infrastructure is generic and incomplete for saturation; there is no build-failing inventory that enforces source/no-default metadata for all 71 rows.

## Escalation and manual-evidence summary

- Section 7.1 contains ten live escalations while section 8.16 says nine. The mismatch remains explicit and the specification was not edited.
- `SP-003` remains open for independent literature/patent evidence; no Tier-C capability or parameter was inferred.
- Manual evidence remains source-owned and unchanged: saturation 2/97, saturation-height 0/6, workflow 0/23, crossplot 6/13, pickett 0/8, LAS export 0/2, processing history 0/7 and verification stewardship 0/24.
- Automated evidence does not close any of those manual scenarios.

## Reachable-history summary

- The accepted implementation anchor is reachable. Current integrated behavior is credited only where live source and exact tests agree.
- Production-code history has no hit for the missing effective-Archie identity, canonical modified-Schlumberger ID, Woodhouse/Total-Shale registry, MUDBASE, vQ0, Poupon methods, Worthington metadata, per-model apparent-Rw inversion, mineral-conductivity disclosure, flushed-zone water volume or saturation method flag.
- Documentation/planning mentions found in history are not implementation evidence.

## Blockers and follow-up

- All 51 rows are adjudicated: 19 `ABSENT`, 9 `PARTIAL`, 16 `PRESENT-DIVERGENT`, 6 `PRESENT-OK` and 1 `PRESENT-UNVERIFIED`.
- Release disposition is 41 `PILOT-BLOCKER`, 8 `UNDECIDED` and 2 `DEFERRED`.
- Risk classification is 20 `SILENT-WRONGNESS`, 20 `DATA-INTEGRITY`, 8 `REQUESTED-CAPABILITY`, 1 `DEGRADED-RESULT` and 2 `LATER`.
- The smallest safe remediation order is canonical model/quantity identity, parameter and unit custody, shared guarded solver/result envelope, cross-engine parity, then scientific provenance and interpreter-facing disagreement guidance.
- Harsh truth: this receipt makes the risk auditable but repairs no saturation equation, default, unit boundary, flag, export record or UI behavior.
