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

### SB-SAT-006

- **Specified contract:** Indonesia with a parameterised shale exponent. Owned test intention(s): `SB-SAT-T09`, `SB-SAT-T10`, `SB-SAT-T30`.
- **Current implementation:** the standalone module exposes three fixed options corresponding to the three exponent forms, while the solver and Results QC hard-code the `k=1` form. There is no typed `k` parameter.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_indo_full_vs_simple`, `sw_indonesia_round_trips` and hand-computed points passed for current branches.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** one parameterized model and cross-engine parity are absent.
- **Next action:** replace fixed local branches with a typed cited `k` route and prove k=0,1,2 plus cross-engine equality.

### SB-SAT-007

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

### SB-SAT-023

- **Specified contract:** The effective back-out is per model, never blanket. Owned test intention(s): `SB-SAT-T34`, `SB-SAT-T35`, `SB-SAT-T36`.
- **Current implementation:** standalone Archie and solver post-processing apply generic effective conversions; multimin applies the same porosity-volume back-out to Archie, dual water, Juhasz and Waxman-Smits.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_dual_nonlinear_hand_computed_and_conversion` and post-solve tests pin the current blanket conversion.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** per-model inverse definitions and degeneracy handling are absent.
- **Next action:** define and test each model's sourced inverse separately, then remove the blanket post-solve conversion.

### SB-SAT-024

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

### SB-SAT-025

- **Specified contract:** Every method emits a clipped and an unclipped curve. Owned test intention(s): SB-SAT-T38.
- **Current implementation:** standalone modules retain method-specific raw saturation plus clipped working outputs, but the solver and LRLC paths generally expose only clipped results.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T38 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** standalone output tests pass, but no whole-family clipped/unclipped inventory exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the pair is not universal or semantically typed.
- **Next action:** add explicit raw and clipped roles to the common saturation result and inventory every model.

### SB-SAT-026

- **Specified contract:** Never emit a bare `SW`; always emit a method-flag curve. Owned test intention(s): `SB-SAT-T39`, `SB-SAT-T40`.
- **Current implementation:** current standalone and solver names avoid exact bare `SW`, but shared generic `SWE`/`SWT` identities are reused and no saturation method-flag curve is emitted.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T39`, `SB-SAT-T40` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** local output-key tests do not prove canonical identity or a flag through persistence/export.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PARTIAL`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** method-specific identity and flag transport are incomplete.
- **Next action:** define canonical suffixed outputs and one method-flag schema, then prove no bare/generic collision survives.

### SB-SAT-027

- **Specified contract:** One shared root-finder with Geolog's guards. Owned test intention(s): `SB-SAT-T12`, `SB-SAT-T41`.
- **Current implementation:** Juhasz, Waxman-Smits and nonlinear dual water share one guarded root helper with a closed form and bisection branch; standalone and LRLC iterative paths remain separate.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T12`, `SB-SAT-T41` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** general-exponent and nonphysical-input tests passed, but no cross-engine shared-root acceptance test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the shared helper is integrated only within one engine and lacks the owned guard-suite proof.
- **Next action:** route every applicable equation through the shared guarded solver and test closed-form/general equivalence plus all guards.

### SB-SAT-028

- **Specified contract:** Non-convergence MUST return null, never a partial iterate. Owned test intention(s): SB-SAT-T41.
- **Current implementation:** the standalone iterative helper returns missing after its cap, but IMTS retains its last finite iterate after 100 iterations because convergence is not recorded.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T41 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no exact cap-hit test observes missing versus a partial iterate.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DEGRADED-RESULT`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** non-convergence semantics disagree across engines.
- **Next action:** return an explicit convergence state from one shared solver, emit missing on cap, and test a forced cap-hit path.

### SB-SAT-029

- **Specified contract:** Inherit the documented guard rails, including the volume detail. Owned test intention(s): `SB-SAT-T42`, `SB-SAT-T43`.
- **Current implementation:** zero porosity and non-positive resistivity guards exist in several paths; standalone modules size unflushed-water volumes, while solver/LRLC paths omit the complete volume contract and variable-m guards.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T42`, `SB-SAT-T43` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf` and three nonpositive-resistivity tests passed; they cover only part of the compound contract.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-OK`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** the complete guard-and-volume matrix is not proven across models.
- **Next action:** centralize the guard result shape and test low porosity, total zero, missing/nonpositive Rt and missing variable-m inputs with all outputs.

### SB-SAT-030

- **Specified contract:** `Vsh → 1` MUST flag before the singularity, not silently return water. Owned test intention(s): SB-SAT-T44.
- **Current implementation:** the standalone modified-Schlumberger branch returns all water at pure shale to avoid division by zero; it emits no pre-singularity flag.
- **Qualifying acceptance tests:** no full owned acceptance proof; the available oracle is implementation characterization, not correctness. Test class `CHARACTERIZATION`.
- **Supporting evidence:** CHARACTERIZATION: `sw_sim_schlumberger_pure_shale_is_all_water` pins the current silent fallback.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `CHARACTERIZATION`; commit state `INTEGRATED`.
- **Blocker or decision:** the singularity is numerically hidden.
- **Next action:** detect the near-pure-shale condition before evaluation, flag it, and prove a nearby valid sample remains evaluated.

### SB-SAT-031

- **Specified contract:** `Rw` ships with no default. Owned test intention(s): `SB-SAT-T31`, `SB-SAT-T45`.
- **Current implementation:** standalone saturation ships Rw=0.1 while LRLC and solver surfaces ship other concrete defaults; none represents absence.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T31`, `SB-SAT-T45` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** zone-override and fluid tests passed using explicit/current defaults, not no-default refusal.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** multiple uncited defaults can silently choose different saturation answers.
- **Next action:** remove every Rw default, require measured/correlation/user custody, and preserve the selected route and temperature reference.

### SB-SAT-032

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

### SB-SAT-034

- **Specified contract:** `a`, `m`, `n`, `m*`, `n*` ship with no default. Owned test intention(s): `SB-SAT-T31`, `SB-SAT-T49`.
- **Current implementation:** a, m, n, m-star and n-star all ship concrete defaults across the engines.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T31`, `SB-SAT-T49` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** hand-computed equation tests pass with explicit values; no missing-parameter refusal test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** uncited defaults remain active calculation inputs.
- **Next action:** make all five parameters absent by default, resolve only cited/user values, and test each missing refusal separately.

### SB-SAT-035

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

### SB-SAT-038

- **Specified contract:** Every parameter carries a source string, and the build fails without one. Owned test intention(s): SB-SAT-T31.
- **Current implementation:** generic source-topic infrastructure exists, but saturation parameters have no complete source strings/tiers and builds do not reject missing metadata.
- **Qualifying acceptance tests:** none; the owned intentions SB-SAT-T31 are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** no saturation parameter-inventory build test exists.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** current Rust/TypeScript surfaces, tests and reachable code history contain no complete implementation; documentation-only mentions are not credited.
- **Verdict:** `ABSENT`; `PILOT-BLOCKER`; `DATA-INTEGRITY`; test class `MISSING`; commit state `UNIMPLEMENTED`.
- **Blocker or decision:** 71 parameter rows are not enforced at compile/build time and 20 ABSENT-bearing plus 8 tierless rows need explicit custody.
- **Next action:** create a generated saturation parameter registry from admissible sources and fail validation when source/no-default metadata is missing.

### SB-SAT-039

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

### SB-SAT-047

- **Specified contract:** One model, one number, whichever engine computes it. Owned test intention(s): `SB-SAT-T30`, `SB-SAT-T61`.
- **Current implementation:** same-looking methods use different equations, defaults, porosity bases, clamps and back-outs; no exact cross-engine parity harness exists.
- **Qualifying acceptance tests:** none; the owned intentions `SB-SAT-T30`, `SB-SAT-T61` are not executable as full contracts. Test class `MISSING`.
- **Supporting evidence:** local module, solver and Results-QC tests all pass separately; none compares identical typed inputs and quantities across engines.
- **Manual evidence:** saturation 2/97; workflow 0/23; verification-stewardship 0/24; no manual scenario was added or checked in this lane.
- **Source/parameter boundary:** chapter sections 4 through 6 and their cited sources govern every expected value; no current literal is promoted to authority, and every ABSENT/no-default state remains fenced.
- **History/reachability:** the current implementation and cited supporting tests are reachable from the accepted implementation anchor; no unmerged branch is credited.
- **Verdict:** `PRESENT-DIVERGENT`; `PILOT-BLOCKER`; `SILENT-WRONGNESS`; test class `MISSING`; commit state `INTEGRATED`.
- **Blocker or decision:** parallel implementations can return different plausible numbers under the same displayed name.
- **Next action:** build independent reference cases per canonical model and require bit/precision parity across every engine before release.

### SB-SAT-048

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
