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

## SB-SAT-002 - Effective and total Archie as separate named methods

- **Specified contract:** ship `archie_effective` (`Sw = (a*Rw/(Rt*phie^m))^(1/n)`) and `archie_total` (the same on phit, followed by the effective back-out) as two distinct, separately selectable methods; **MUST NOT** expose a method named `archie` with an undeclared porosity system (`12_saturation.md:893-906`).
- **Why it is P0:** on the reference case the two answer **0.884 vs 0.634 - 25.0 saturation units, HCPV 3.15x apart**. The chapter calls it the largest single cross-tool trap in the domain, and it is invisible in the output.
- **Current implementation:** `ABSENT`, confirmed in code. `sw_arch` (`modules.rs:4853` spec, `:4889` body) is total-only - `SWT = (A*Rw/(PHIT^M*RT))^(1/N)`, with `SWE` merely backed out via `swtsh = 1 - pe/pt`. There is no effective-porosity Archie.
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Source/parameter boundary: nothing needs inventing.** The effective equation is cited above, and the missing half - what `SWT` becomes under it - is supplied by **SB-SAT-023** (`:1337-1344`): `SwT = Sw(1 - Swb) + Swb` with `Swb = 1 - phie/phit` for the Archie family, with a round-trip through the pair required to be the identity and `Swb = 1` required to yield `SWE = 1` rather than a divide-by-zero. `sw_arch` already computes exactly that `Swb` as its local `swtsh`. So the physics is complete and no ruling is needed on it.
- **Blocker or decision:** `BLOCKED-BOUNDARY`, and this **corrects an earlier assessment in this same session** that recorded the row as implementable-not-blocked. It is not. `SW_METHOD` is a categorical class curve whose codes come from `SwModel::flag_code()`, and `SwModel` (`multimin2.rs`) has **no `ArchieEffective` variant** - its members are `LinearDw`, `DualWaterNonlinear`, `ArchieTotal`, `Indonesia`, `SimandouxBardonPied`, `SimandouxModifiedSlb`, `Juhasz`. `multimin2.rs` is a prohibited file. Minting a code inside `modules.rs` instead would put the module vocabulary out of step with the solver registry - which is precisely what **SB-SAT-001 arm D** now forbids, so the workaround would fail a test shipped hours earlier.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Next action:** Jauhar authorizes a narrow `multimin2.rs` edit adding an `ArchieEffective` variant with its own `flag_code()` and catalogue entry - the DEC-040 pattern. Then add an equation-identity option to `sw_arch` carrying `archie_total` as the **default** so no saved run changes, branch the body to compute `SWE` directly on `PHIE` and lift `SWT` through SB-SAT-023's inverse, and pin from both sides: the two branches must DISAGREE on the chapter's reference case rather than quietly returning the same curve, and the round-trip must be the identity.

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

- **Specified contract:** every saturation method **MUST** emit both a clipped curve (`SWE`/`SWT`, bounded to `[SWE_IRR, 1]` / `[SWT_IRR, 1]`) and an unclipped diagnostic (`SWE_<METHOD>` / `SWT_<METHOD>`). Where a method produces both a total and an effective result, **both** MUST have an unclipped counterpart (`12_saturation.md:1352-1366`).
- **Why it matters:** a clipped-only curve cannot distinguish *the rock is wet* from *the model went out of range*. Geolog and Techlog both ship unclipped diagnostics; IP does not, and IP's own comparison-curve caveat exists precisely because it lacks them.
- **Current implementation - both gaps verified in code:** `SWT_ARCH`, `SWE_INDO` and `SWE_SIM` are emitted unclipped. But `sw_arch` declares `SWT_ARCH` and has **no `SWE_ARCH`** - its effective result is clipped-only, even though the same module produces both a total and an effective answer, which is exactly the case the requirement singles out. Separately the LRLC modules emit clamped values only (`lrlc.rs:183`, `:365` show `limit(..., 0.0, 1.0)` with no unclipped twin).
- **Qualifying acceptance tests:** none. Test class `MISSING`.
- **Manual evidence:** saturation 0/31.
- **Source/parameter boundary:** no parameter is involved; this is an output-surface contract.
- **Blocker or decision:** `BLOCKED-BOUNDARY`, and the split is worth stating because half of it is ready. The `sw_arch` half is in **`modules.rs`, an ALLOWED file** - adding the missing `SWE_ARCH` unclipped twin needs no authorization and no decision. The LRLC half is in **`lrlc.rs`, a prohibited file**. Because the requirement is one MUST over *every* method, shipping only the `sw_arch` half would leave the contract unmet while adding a new output curve, so the row is held atomic rather than half-delivered. **`lrlc.rs` now joins `multimin2.rs` as the second prohibited file gating this group** - SB-SAT-023 needs both.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; test class `MISSING`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes the narrow `lrlc.rs` edit alongside the `multimin2.rs` one already requested by SB-SAT-002 and SB-SAT-023. Then add `SWE_ARCH` to `sw_arch` and unclipped twins to the LRLC outputs, and pin from both sides: a sample the model drives out of range must show the clipped curve at its bound AND the unclipped diagnostic beyond it - one arm alone would pass a module that simply copies the clipped value into the diagnostic.

## SB-SAT-026 - Never emit a bare SW; always emit a method-flag curve

- **Specified contract:** no emitted mnemonic may equal bare `SW` or `SXO`; every saturation curve **MUST** carry an `E` or `T` designator. Every saturation run **MUST** additionally emit a **method-flag curve** recording which model produced each sample, and **MUST** emit `VOL_UWAT`/`VOL_XWAT` (or `BVWE`/`BVWT`) alongside (`12_saturation.md:1368-1381`).
- **Current implementation - the as-built is PART stale, and the correction narrows the row.** It says *no method-flag curve exists*. One does: `SW_METHOD` is a declared output of `sw_arch`, `sw_indo` and `sw_sim`, and `sw_arch` already emits `SwModel::ArchieTotal.flag_code()` per sample. The real gap is COVERAGE, not absence - **3 of the 7 saturation modules emit it**. The naming clause is separately satisfied: `grep` finds no `log_out("SW")` or `log_out("SXO")` anywhere, so no bare mnemonic ships. `VOL_UWAT` is emitted by the same 3.
- **The 4 modules without a method flag are ALL in prohibited files:** `sw_rtc` and `sw_imts` (`lrlc.rs`), `multimin` (`multimin.rs`) and `sw_height` (`satheight.rs`).
- **Qualifying acceptance tests:** none. Test class `MISSING`. Note the as-built's own words - the naming rule is *unenforced by any test* - which is the half that needs no authorization.
- **Manual evidence:** saturation 0/31.
- **Source/parameter boundary:** no parameter is involved. Ledger D-15 and Geolog's `OPT_SW` scheme are cited by the chapter; nothing was inferred.
- **Blocker or decision:** `BLOCKED-BOUNDARY`. The method-flag clause needs edits to **three** prohibited files - `lrlc.rs`, `multimin.rs` and `satheight.rs`. The naming clause is already true and its enforcement test needs no authorization, since a test only READS module specs. It is held with the row rather than landed alone, for the same reason SB-SAT-025 was held atomic: one MUST, and a half-delivery reads as progress without meeting the contract. **`multimin.rs` and `satheight.rs` now join `multimin2.rs` and `lrlc.rs`** in the set of prohibited files gating this group.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; test class `MISSING`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes narrow edits to `lrlc.rs`, `multimin.rs` and `satheight.rs` to emit `SW_METHOD` (and `VOL_UWAT`/`VOL_XWAT` where missing) from the remaining four modules. Then pin both clauses universally over the shipping catalogue: no declared output anywhere equals bare `SW` or `SXO`, every saturation curve carries an `E`/`T` designator, and **every** module in the Saturation category declares a method-flag output - the universal form so a future saturation module cannot ship without one.

## SB-SAT-027 - One shared root-finder with Geolog's guards

- **Specified contract:** solve **every** polynomial-form saturation model with **one** shared root-finder using seed 0.5, **maximum 20 iterations**, tolerance `|d| < 1e-5`, and `sat = MAX(0, sat)` at each step. Where a closed form exists for `n = 2` it MAY be a fast path and **MUST be asserted equal to the general solver** (`12_saturation.md:1383-1397`).
- **Why one solver:** Techlog's Levenberg-Marquardt is over-engineered for a scalar monotone root and its behaviour is undocumented and uninspectable (ESC-8); Geolog's guards are explicit and testable.
- **Current implementation:** the module side transcribes Geolog's `CALC_SW`, and `multimin2.rs:391` `sw_cond_root` is a second, different solver - closed quadratic at `n = 2`, else bisection - which the chapter calls defensible for the solver's monotone forms but **not cross-asserted** against the first. Juhasz, Waxman-Smits and nonlinear dual water share that one guarded helper; the standalone and LRLC iterative paths remain separate.
- **Qualifying acceptance tests:** none for the shared-solver contract. Test class `MISSING`.
- **Supporting tests:** `sw_sim_matches_quadratic_solution` already asserts the module against the analytic quadratic at `N = 2`, which is one half of the chapter's *MUST be asserted equal* - but it compares the module to a closed form, not the two ENGINES to each other, which is the assertion the chapter actually asks for.
- **Manual evidence:** saturation 0/31.
- **Source/parameter boundary:** every guard is cited verbatim - seed 0.5, 20 iterations, 1e-5, `MAX(0, sat)`, from `sw_sim.lls:256-271`. Nothing needs inventing.
- **Note on stale line references:** the chapter cites `modules.rs:2218-2230` for the transcription; that range no longer holds it, and the `for _ in 0..20` loop at `modules.rs:4541` is `gascorr`'s density iteration with a `1e-4` tolerance, not the saturation root-finder. A reader following the citation lands on the wrong loop - worth correcting when the row is built.
- **Blocker or decision:** `BLOCKED-BOUNDARY`. The requirement is *every* polynomial-form model through *one* solver, and the standalone and LRLC iterative paths are separate today - routing them means editing **`lrlc.rs`**, prohibited. The cross-assertion between the two engines also needs the solver side reachable on equal terms. This is the same authorization already requested by SB-SAT-023, 025 and 026.
- **Verdict:** `PRESENT-OK` behaviour with `MISSING` proof; `PILOT-BLOCKER`; commit state `INTEGRATED`.
- **Next action:** Jauhar authorizes the narrow `lrlc.rs` (and `multimin2.rs`) edits. Then route every applicable equation through the shared guarded solver and pin the guard suite owned rather than assumed: seed 0.5, the 20-iteration cap actually binding, `|d| < 1e-5`, `MAX(0, sat)` at each step, and the closed form at `n = 2` equal to the general solver **engine against engine** - the arm that is missing today.

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

- **Specified contract:** A saturation result carries its parameters, their sources and their papers. Owned test intention(s): SB-SAT-T59.
- **Current implementation:** runs preserve module, parameter JSON, input JSON and version; LAS export carries that generic ancestry but not parameter sources, papers, calibration state or saturation flags.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T59 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** `every_las_export_carries_measured_computed_and_model_provenance_in_the_file` passed for generic ancestry only.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** scientific provenance is absent from both stored and exported saturation results.
- **Next action:** define a typed saturation provenance record and verify every required field survives save, reload, Inspector, report and LAS export.

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
