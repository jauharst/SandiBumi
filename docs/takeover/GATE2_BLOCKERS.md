# Gate 2 blocker decision packet

This is the human-readable companion to the machine-owned blocker set in
`gate2-program.json`. The requirement evidence remains authoritative in
`requirements.csv`; product-owner choices remain authoritative in `DECISIONS.md`.

## Live snapshot

- Gate: `G2 - SILENT-WRONGNESS CLOSURE`
- Scope: `222` Gate 2 requirements plus `20` later-gate-only requirements
- Handled: `222 / 222`
- Done: `164`
- Blocked: `58`
- Remaining unhandled: `0`

---

## What was solved on 2026-08-17, and what it changed

A working session closed three rows outright, materially advanced three more, and corrected the
register against its own chapters **seven times**. Read this before trusting a blocker below.

### Rows closed

- **`SB-DIO-056`** — the LAS writer took `depth[1] - depth[0]` as the `STEP`. On a merged or
  depth-shifted well that is not the step, and a conforming reader is entitled to rebuild depths
  from `STRT`/`STEP`, silently re-gridding the data. Now verified across the whole index, declaring
  `STEP 0` when it varies (LAS 2.0's own provision) and naming the depth where it changes.
  **The comparison is made on the emitted fixed-decimal-4 TEXT, not the stored `f32`s** — at ~1000 m
  an `f32` resolves to ~0.00006, so a perfect 0.1524 m frame is not bit-identical under subtraction
  and would be called irregular. Proven by mutation, not argued.
- **`SB-DIO-062`** — the declared text-encoding inventory is published as a contract and pinned.
  No ambiguity rule was invented, because the decode order is TOTAL. The cp1252 fallback's
  inability to fail is pinned as a **property over every byte**, not one example.
- **`SB-POR-054`** — density and sonic wrote the same identity two ways.
  `modules::two_endpoint_fraction` is now the one subtraction order and **no call site writes a
  subtraction**, so the half-flipped form — which returns a NEGATIVE porosity from an expression
  that reads like both textbooks — is unwritable rather than discouraged. Bit-identical: negating
  both halves of a quotient is exact in IEEE-754, and all backend tests passed unchanged.

### Rows advanced, with the remaining question stated exactly

- **`SB-SAT-027`** — the register claimed two un-cross-asserted solvers. **There is one**: `sw_sim`
  delegates to `multimin2`, and 15 `modules.rs` sites route there. The `n = 2` fast path is now
  proven equal to the general root-finder. **One clause left, and it is a method ruling:** the
  chapter specifies Geolog's guards (seed 0.5, 20 iterations, `|delta| < 1e-5`) while the solver
  uses 60-step bisection — arguably better, since bisection on a monotone function is
  unconditionally convergent, but swapping a specified numerical method is Jauhar's call.
- **`SB-CUT-001`** — the net-pay clip rule existed **three times**. All three consumers now route
  through `workflow::sample_incl_thickness`; proven number-neutral. **One clause left, and it moves
  reserves:** the requirement wants a model parameter defaulting to CENTRED while every shipped path
  uses FORWARD, and the two differ by half a sample step at each zone boundary.
- **`SB-DBM-032`** — DEC-028 ruled both one-handle parameter forms are refused, and a test now pins
  it. Honest limit: defeating the structural ordinal guard did NOT turn the test red, because schema
  validation refuses too. That is defence in depth, and it means the test proves the CONTRACT rather
  than any single guard.

### The register was wrong seven times, and in one direction

Every correction made the work SMALLER or different, never larger: a fixed arity mistaken for a cap;
a landed dependency recorded as outstanding; a chapter permitting a fast path recorded as forbidding
it; two engines that were one; a stale `SB-POR-005`/`SB-POR-040` link in neither requirement text.
**Read the chapter before believing a blocker.**

### The stale class column

Roughly **31 of the 59** blocked rows cite a `DEC-` number in their Blocker class. Those decisions
were RULED on 2026-08-17. Those rows are ordinary engineering waiting on implementation, not
questions waiting on Jauhar — but the class column still reads as though the decision were open, and
that is what made an earlier summary mis-state the shape of this gate.

## What I actually need from you (plain language)

There are 58 blocked rows, but they are not 58 separate problems. They collapse into four kinds of
thing. **As of 2026-08-17 the decision bucket is empty** - 38 of them are now ordinary engineering
work; SB-DIO-056, SB-DIO-062 and SB-POR-054 have since shipped. What still needs you is 8 documents
and 1 legal question.

### 1. Decisions only you can make — all cleared 2026-08-17

**Every decision that blocked a Gate 2 row is now ruled.** You answered twenty-five of them in one
sitting on 2026-08-17, with a twenty-sixth closed by reference (DEC-026 asked the PHIE-floor
precedence question DEC-043 had already answered), and **41 of the 62 blocked rows are
decision-cleared** — one of them (SB-DIO-056) has since shipped and the rest are ordinary
engineering work waiting to be done, not questions waiting on you.

Twice you rejected the question's premise rather than picking one of my options, and both times you
were right:

- **SB-POR-054 → DEC-049.** I asked which of two spellings of the porosity identity was canonical.
  You said *"both true, its different type of porosity."* Density and sonic porosity are different
  measurements with different endpoints, each carrying its own conventional form. The MUST is
  satisfied by declaring each type's convention, not by forcing one spelling onto both. What is
  actually dangerous here is neither published form — it is the MIXED ordering, which returns a
  negative porosity from an expression that looks like both textbooks at a glance.
- **SB-CORE-007 → DEC-051.** I asked how strict the output-name collision rule should be. You said
  modules already carry distinct defaults (`PHIE_DEN`, `PHIE_SON`) and a user typing `PHIE` is
  deliberately replacing a result. So a default collision is a registry bug and a typed one is a
  supported workflow — which dissolves the row's difficulty instead of answering it: with unique
  defaults, no rule has to tell canonical results from working aliases from categorical flags.

Five rulings each cleared several rows at once. **DEC-031** — one coded correction-state curve, the
applied-step manifest in the log-set archive, partial correction over the intervals a caliper
actually covers — settled three storage requirements on one answer and, through the chapter's own
OI-7, settled **DEC-032** as well. **DEC-021** (a per-module source digest, replacing a package
version that does not move when a module's arithmetic does) with **DEC-023/024** closed all four
arms of the re-run manifest. **DEC-025** authorized the neutron matrix-basis seam. **DEC-036**
authorized a narrow versioned CLY provenance registry, because unlike DEC-035's deferred cull these
deferred rows own infrastructure that already-approved rows cannot be built without.

Two rulings carry a stated risk rather than a buried one. **DEC-022** — legacy timestamps are WIB —
is only correct if every legacy record came off a UTC+7 machine, and a timestamp carries no evidence
of the clock that made it. **DEC-056** deliberately does *not* take that route for depth: legacy
depths are typed MD and stay untied to any datum, because the KB-to-GL difference is commonly 5–15 m
and a wrongly declared reference offsets a whole well with nothing in the numbers to show it.

**Six decisions remain open — DEC-004 through DEC-009 — and none of them blocks a Gate 2 row.** They
are G5 commercial questions: licence unit, commercial model, support hours, update window, benchmark
hardware profile, lineage granularity.

### 2. Documents I need you to find — 7 rows

These are not decisions. They are stopped because the source that fixes the number is not in the
repo and I will not invent it. This is the same shape as SB-POR-021 earlier: you sent the 1977
Bateman & Konen paper and the block dissolved in one message.

- **`SB-CORE-015`** — **RESOLVED 2026-08-17. The spec is found.** You supplied
  `energistics.org/sites/default/files/RP66/V1/Toc/main.html`, which is the full normative RP66 V1
  document — sections 1-7 plus Appendix B (Representation Codes) and Appendix E (Checksum), and
  section 2 carries the Storage Unit Label, Visible Record and Logical Record Segment framing. This
  row is no longer waiting on you. It is now a **substantial engineering row**, not a quick one: a
  conforming writer means storage-unit framing, record segmentation, EFLR/IFLR, representation codes
  and the checksum. Nothing partial ships in the meantime.
- **`SB-ENV-004`** — the largest of these: 19 sources to supply or adjudicate, plus a docs-only
  31/32 identity reconciliation before the inventory tests can be written.
- **`SB-DBM-005`** — the complete registered-module derivation-source map. **`SB-DBM-010` unblocks
  itself the moment this lands**, so it is two rows on one document.
- **`SB-DIO-011`** — a named source for every accepted deviation alias.
- **`SB-DIO-057`** — a versioned ENV-reviewed family registry with a source for every member.
- **`SB-CORE-005`** and **`SB-CORE-044`** — sources *and* counsel dispositions; these two sit
  partly in the legal bucket below.

### 3. Rows where the tracking file is simply wrong

Worth knowing before you spend time on any of these. **Four Group F rows in a row** carried blockers
that the chapter itself does not support — SB-POR-026, 028, 043 and 044 — and one of them
(**SB-POR-043**) turned out to be fully implementable once I read `11_porosity.md` instead of the
summary. It shipped this session. So if a row here says "needs a decision", it is worth one minute
checking the chapter before you spend an hour deciding.

### 4. Rows waiting on other rows — no action from you

These unblock themselves as their prerequisite lands. Example: **SB-POR-025** needs somewhere to get
salinity-dependent endpoints from, and both of its sources are shut — one is deferred outside the
manifest, the other is SB-POR-021, whose paper you already sent but whose evaluator I have not
written yet. Build that, and 025 opens on its own.

### One legal item

**SB-PLT-024** is `BLOCKED-LEGAL`. That is a lawyer question, not a petrophysics one.

### Honest caveat

I read the POR rows closely this session. The SAT and CUT rows still carry the original audit's
wording, and given what I found in Group F, some of those blockers are probably stale too. Do not
treat a blocker here as proven until it has been re-read against its chapter.

---

- Evidence boundary: Automated, Visual, Manual and Field evidence are separate.
- Scientific boundary: a value, limit, tolerance, endpoint or family classification is cited or
  remains absent. Current code is never its own authority.

`BLOCKED` does not mean one thing. A row can require a product-owner decision, an authoritative
source, legal disposition, or later engineering follow-through. Treating all four as ordinary
coding gaps would invite silent wrongness or transfer engineering responsibility to Jauhar.

## Exact blocked-requirement inventory

The first column is mechanically compared with `gate2-program.json`. Every live blocked
requirement appears exactly once.

| Requirement | Blocker class | What is missing | Exact unblocking input or action |
|---|---|---|---|
| `SB-CORE-003` | Engineering follow-through | The common validity schema, refusal path and linear-GR conditions exist, but not every selected pilot method yet carries its own cited valid and invalid conditions. | Populate cited conditions while executing the remaining ENV, CLY, POR, SAT and CUT rows, then re-audit the whole pilot registry. Jauhar supplies a source only when a selected method's condition is not cited. |
| `SB-CORE-005` | Source and legal | The merged endpoint library has no exact per-value origin map. Equal vendor values do not prove origin. | Provide exact construction custody or primary sources per value; keep unresolved values absent; obtain counsel disposition for CLAIM-012. |
| `SB-CORE-007` | Product contract | A no-parameter execution fixture cannot run producers with required absent parameters, and raw repeated output names conflate canonical results, working aliases and categorical flags. | Approve registry-level semantic identities and declaration inspection without forcing unlike outputs equal or executing a module whose required values are absent. |
| `SB-CORE-015` | Engineering follow-through | LAS semantic self-round-trip is covered, but exact T15 requires a DLIS writer that does not exist. **Both original blockers are cleared**: DEC-054 put DLIS export in the first pilot, and the normative API RP66 V1 source was supplied on 2026-08-17 and verified complete. | Build the writer from RP66 V1 — storage-unit and visible-record framing, logical record segmentation, EFLR/IFLR, Appendix B representation codes, Appendix E checksum — citing section numbers rather than copying specification text. Nothing partial ships. No further input from Jauhar. |
| `SB-CORE-044` | Source and legal | Chart payloads, the merged endpoint library and branded themes still lack primary-source, design-around or counsel closure. | Obtain the named source and counsel dispositions; remove or independently re-source uncleared routes; then add the fail-closed release inventory proof. |
| `SB-DBM-005` | Source map | No complete source-controlled map assigns every shipping module a primary citation or approved first-principles derivation document. | Approve or provide the complete registered-module derivation-source map before adding fail-closed registration and run/deliverable custody. |
| `SB-DBM-010` | Dependency on source map | Parameter sources and legacy labels travel through current outputs, but free-form derivation text is not a source-controlled citation. | Close SB-DBM-005, then establish one machine-readable provenance/sidecar contract across pilot exports, reports and Office deliverables. |
| `SB-DBM-011` | Engineering - DEC-022, DEC-023 RULED | The structured relational audit store is absent; exact T11 also needs legacy timestamp treatment and a zone-set identity outside the immutable pilot manifest. | Settle DEC-022 and authorize the narrow zone-set identity/version seam or revise the manifest; then implement the backend-owned atomic audit writer. |
| `SB-DBM-015` | Engineering - DEC-021, DEC-023, DEC-024 RULED | A complete rerun manifest requires build, zone-set, conditional stochastic and learned-model identities, including seams owned by deferred capabilities. | Settle build and zone-set identity, then authorize identity schemas/resolvers as narrow infrastructure or revise and reapprove the pilot scope. |
| `SB-DBM-031` | Source declaration | Untyped legacy/imported depth frames have no recoverable datum or source sign. | Require explicit datum at every remaining boundary; migrate only source-declared rows; preserve unknown legacy meaning and refuse cross-datum comparison. |
| `SB-DBM-041` | Dependency on structured audit | The count divergence is closed, but the inspector cannot claim a complete provenance inventory before `audit_entry` and `audit_detail` exist. | Close DEC-022/023 and SB-DBM-011, then derive the inspector inventory from the complete registered audit/provenance schema. |
| `SB-DIO-007` | Product contract | Empty and explicitly nulled cells collapse to the same arithmetic/export state; the project forbids `Option<f32>`. | Approve a versioned bytemuck source-cell-state mask beside the numeric `f32` array and define preservation or sidecar/refusal per deliverable. |
| `SB-DIO-011` | Source | The deviation index alias list has no documented source; the prior test said every while enumerating only three sourced lists. | Supply a named source for every accepted deviation alias, then make T17 discover all index-alias declarations mechanically. |
| `SB-DIO-057` | Source | Logarithmic-family membership is a scientific classification and cannot be inferred from mnemonic or display settings. | Publish a versioned ENV-reviewed family registry with a source for every member, then add pre-commit zero handling and custody. |
| `SB-DIO-060` | Scope dependency | Exact signature-collision proof needs a real BIFF5 table read, but SB-DIO-059 is deferred. | Promote the narrow published-spec BIFF reader or keep BIFF unsupported and re-adjudicate the dependent acceptance boundary. |
| `SB-DIO-061` | Inventory and allocation contract | The current corpus matrix is not a universal reader diagnostic proof and no sourced maximum whole-file allocation exists. | Publish the complete reader inventory and either a cited maximum size or an approved bounded-streaming contract; require location, rule and affected count per reader. |
| `SB-PLT-024` | Legal | Nineteen vendor-derived numeric chart definitions remain imported and bundled; fail-closed rendering does not remove distributed bytes. | Counsel chooses licence, independent primary-source re-derivation, or removal from the paid build/repository. Engineering cannot declare permission. |
| `SB-ENV-004` | Source and specification | Nineteen environmental parameters lack admissible sources and T07 says 31 absent identities while the authoritative parameter table says 32. | Supply/adjudicate the 19 sources and perform a docs-only 31/32 identity reconciliation before implementing the exact inventory tests. |
| `SB-ENV-005` | OI-4 and dependency | Corrected curves have no complete typed list of applied, unavailable, disabled or refused steps, and the persistence owner is open. | Select one authoritative run-level correction manifest linked to output curves, with source-complete step identities, versions, parameters and sample coverage. |
| `SB-CLY-001` | Engineering - DEC-036 RULED | The generic binary precondition flag is not the specified categorical `ENDPOINT_INVALID` reason and carries no zone identity. | Add SB-CLY-031/032 to the pilot or authorize their exact categorical schema as narrow infrastructure, including versioned wire/LAS codes and separate substitution custody. |
| `SB-CLY-034` | Engineering - DEC-036, DEC-037 RULED | Provenance export is absent, and a global undeclared `-999` screen would delete legitimate values and violate explicit `NoNull`. | Settle categorical provenance plus a source-scoped rule where `NoNull` wins and matched undeclared values are quarantined behind a named import decision or exact approved automatic policy. |
| `SB-CLY-055` | Engineering - DEC-036, DEC-037 RULED | Exact T35 requires the deferred CLY provenance output and an authorized LAS token representation; exact T44 needs a source-scoped undeclared-`-999` policy compatible with per-channel `NoNull`. | Authorize the complete versioned categorical provenance schema as pilot infrastructure or add SB-CLY-031/032 to the manifest, then define the exact CLY source/mnemonic signatures and `NoNull`-first sentinel policy before writing the all-output round trip. |
| `SB-POR-002` | Engineering - DEC-038 RULED | Sonic has no unlimited twin. SSC/SSPW discard pre-limit values inside protected `ssc.rs`, and “unlimited” is ambiguous because upstream component and geometry clamps may already have bound the final value. Independent storage/export proof also depends on SB-POR-004's collision-safe custody. | Decide whether SSC/SSPW are methods under SB-POR-002 or separately typed workflows. If included, define the exact unlimited boundary, finish SB-POR-004, and authorize one narrow `ssc.rs` edit that preserves all existing limited arithmetic while exposing the approved pre-limit lineage. |
| `SB-POR-010` | Dependency on SB-DBM-015 | Method identity, parameter values with source and tier, per-output convention and resolved input-curve identities are all already persisted. What is absent is the re-derivability clause: no stored manifest resolves module identity, options and defaults into one replayable record. | Settle DEC-021, DEC-023 and DEC-024, close SB-DBM-015, then prove a POR curve replays from its stored manifest alone without querying any mutable default. |
| `SB-POR-024` | Engineering - DEC-025, DEC-045 RULED | The N-D crossplot must refuse an NPHI curve whose matrix units are not declared and must state the declared basis in provenance. `nphimat` already performs the conversion, but nothing stores the delivered basis: the live choices are explicit module parameters, not curve metadata, so the refusal has nothing to read. A limestone-unit neutron against a sandstone matrix reads about 0.04 v/v low in clean water sand. | Settle DEC-025 - authorize the narrow SB-ENV-012 typed neutron-scale metadata/persistence seam, or revise the manifest to include it. Then require the declaration at the `phi_dn` boundary, refuse an undeclared or wrong basis by name, and emit the basis in per-output provenance. Do not infer a basis from the mnemonic or supply a default. |
| `SB-POR-025` | Dependency on SB-POR-021 / SB-POR-022 | The fresh-and-salt lever rule has no admissible values to interpolate between. Its endpoint sources are SB-POR-022, the gated chart digitisation, which is DEFERRED and outside the manifest; and SB-POR-021, whose 1977 primary source is now held but whose evaluator is unwritten. `nphimat` is a Prep converter, not a POR endpoint source. | Implement SB-POR-021's evaluator now the source is held, or promote SB-POR-022 into the manifest; then add a typed fluid-condition input with no default and persist the resolved salinity response. |
| `SB-POR-044` | PhiMax identity in the smooth form | The DEC-018 reading was stale - the row is in the approved 242. The real block: `11_porosity.md:1048-1050` says the smooth form's **three** parameters ship with no defaults, but its `PhiMax` collides with SandiBumi's `PHIE_MAX`, which already carries a Geolog-sourced 0.3. Reusing it hands the mode a default the chapter forbids; adding a second invents a parameter and leaves two ceilings that can disagree. | Owner rules which `PhiMax` is meant. Then add `SMOOTH_ROLLOFF` as a third `OPT_PHIEMAX` mode with `param_open` parameters that refuse unsupplied, and pin step, smooth and refusal. |
| `SB-POR-045` | RULED 0.001 on 2026-08-16 - implementation pending | The chapter (`:1052-1056`) says the floor value **MUST** ship with no default and be a documented user decision, because IP's manual gives **0.001 and 0.0001 for the same quantity in three places** and the chapter calls it unresolvable. A later product record picked 0.001; SandiBumi hard-codes 0.001. Does the later record supersede the chapter? Only you can say. It bites in tight and zero-porosity intervals - exactly where a pay cutoff sits. **Jauhar ruled 0.001.** The adjudication half is closed. Remaining: the chapter states TWO clauses - no default **and** documented user decision - and the ruling settles the second. Confirm whether 0.001 becomes a cited **default** (matching the SB-POR-011 and SB-POR-023 rulings) or a `param_open` that refuses unsupplied. | Confirm which reading, then move it out of the compile-time `PHIE_FLOOR` constant into source-labelled configuration and prove two values give distinct LIMITED outputs while the UNLIMITED twin is untouched. |
| `SB-POR-048` | Engineering - DEC-039 RULED | The chapter (`:1065-1072`) requires porosity to consume `COAL_FLAG`, `TIGHT_FLAG` and `COND_FLAG` **with defined branch behaviour** - and does not define it. Per flag: does it **mask**, **select a branch**, or only **annotate**? Three different porosity curves from identical data, none cited. Coal is the sharp case: a coal bed has a real very high apparent density porosity, and blanking it versus computing-and-labelling it is a method preference. Recording the outcome then needs DEC-039. | Rule the policy per flag. **DEC-039 itself was RULED 2026-08-16 and the demand to settle it was stale.** Then declare the three as typed inputs, implement without deleting any existing guard, and prove each consumed flag plus an unflagged control. |
| `SB-POR-055` | One ruling on `RHO_DSH` | Narrowed twice - both stated blockers cleared this session, and the *no POR source topics* claim is now false since SB-POR-007 and 043 registered nine. What remains is substantive: the chapter records that **`RHO_DSH = 2.65` matches no held source at all** and sets `PHIT_SH` a factor **1.73 low**. Its own rule then requires ABSENT - which stops every porosity run until the user supplies it. That moves PHIE and therefore pay. | Rule `RHO_DSH`: ABSENT per the standing decision, or adjudicate a cited value as DEC-041 did. Then build the universal inventory gate, which needs no decision. |
| `SB-POR-057` | Engineering - DEC-042 RULED | Clause 3 says quick-look curves are **excluded by default from pay summation**. **Your ruling (b) / DEC-042 says the opposite** - same curves (`AVERAGE`, `GAS_RMS`), and the pay-eligible behaviour already shipped. This is the **second** record that ruling contradicts; the first was `PILOT_SCOPE.md` item 6. Implementing clause 3 as written would silently revert your ruling in code. | Confirm DEC-042 supersedes clause 3 here too, or state the narrower reading. Then build the comparison-only output class and provenance flag, and pin the ruled pay behaviour from both sides. |
| `SB-SAT-026` | Engineering advance 2026-08-17; one naming ruling and the persistence proof left | **The flag-coverage half SHIPPED.** sw_rtc, sw_imts and sw_height now emit `SW_METHOD` (codes 10/11/12), finite exactly where the saturation is, resolvable through the ONE shared registry - which this increment split from the SandiMin selector: `sw_model_catalog` carries every method's identity, `solver_selectable_models` is what the dialog offers, and `run_multimin` REFUSES a registry-only identity by name. The universal clauses are pinned: no live module ships a bare `SW`/`SXO` output, and no live Saturation module ships without a method flag - the universal form, so a future module cannot. Five removal probes each fire a distinct assertion. multimin is retired and exempt. | Two things remain, and only one is Jauhar's. **(1) His ruling: does `SWH` take an E or T designator?** T39's clause says every saturation mnemonic carries one; `SWH` carries neither, and which porosity system a height-function Sw belongs to is a method identity call, not an engineering one. **(2) SB-SAT-T40's persistence/export proof** - the flag surviving write, reload and LAS export as the categorical it is. `VOL_XWAT` is noted not applicable (no flushed-zone module ships); `VOL_UWAT` for the LRLC pair needs the effective-volume identity confirmed with the SWH ruling. |
| `SB-SAT-027` | Engineering - DEC-048 RULED, one method ruling left | **Re-read against source 2026-08-17; the register was wrong twice and the row is now narrow.** (1) It claimed `multimin2.rs` is *a second, different solver* not cross-asserted against `modules.rs`. **There is only ONE engine**: `sw_sim` delegates to `multimin2::sw_simandoux_*`, both of which call the single `solve_simandoux_root`, and 15 `modules.rs` sites route into `multimin2`. The *one shared root-finder* clause is met by construction. (2) It claimed every equation must be re-routed; `12_saturation.md:1425-1431` explicitly permits an `n = 2` closed form as a fast path provided it is asserted equal. **That assertion now exists** - `the_n_equals_two_closed_form_agrees_with_the_general_root_finder_on_the_same_inputs` straddles the fast-path guard over four cases, requires an interior root so agreement cannot be between two clamps, and was mutation-proved. **ONE clause remains, and it is a METHOD DECISION, not engineering.** The chapter specifies Geolog guards - seed 0.5, maximum 20 iterations, tolerance abs(delta) below 1e-5, `sat = MAX(0, sat)` each step - but the shipped solver uses 60-step bisection on `[0,1]` with a clamp. | Jauhar rules on whether bisection substitutes for the literal Geolog guards. It is arguably BETTER - bisection on a monotone function is unconditionally convergent where Newton from a fixed seed is not, and `solve_simandoux_root` proves monotonicity before bisecting - but swapping a specified numerical method is his call, not engineering's. If he accepts it, the chapter clause is amended and the row closes on the test already written; if not, the solver is rewritten to the literal guards and re-proved. No further source or authorization is needed either way. |
| `SB-CUT-001` | Engineering - DEC-048 RULED, one method ruling left | **The triplication is GONE as of 2026-08-17.** `workflow::sample_incl_thickness` was already the shared rule with its own test; the two inline copies - `workflow.rs` pay summary and `montecarlo.rs` - now route through it, so there is **one implementation and three consumers**, which is what the requirement asked. Proven number-neutral: all 1,047 backend tests pass unchanged. `montecarlo.rs` was edited under DEC-048 narrow authorization. **What remains is ONE clause and it changes reserves numbers**: the requirement wants a discretisation-model parameter **defaulting to CENTRED**, while every shipped path uses the FORWARD interval `[depth, depth+step)`. | Jauhar rules on the CENTRED default, because adopting it **moves every existing net-pay and NTG number** - the two models differ by half a sample step at each zone boundary, which is small on 0.1524 m data and material on coarse data. Engineering will not switch a shipped reserves convention on a requirement text alone. Separately and independently of that ruling, one contract is now guaranteed by the shared rule but still UNPINNED: that a Monte Carlo net agrees with the deterministic pay summary for the same inputs. Worth a test, because an MC P50 that disagreed for this reason would read as uncertainty rather than as a bug. |

## Product-owner decision packet

The detailed alternatives and affected tests for `DEC-021` through `DEC-037` live in
`DECISIONS.md`. The recommended architecture bundle is:

1. Use build-derived per-module semantic identities rather than package version or whole-executable
   identity.
2. Label legacy timestamps `ZONE_UNKNOWN`; store new timestamps as UTC instants and convert only at
   display.
3. Authorize narrow identity and metadata seams required by approved requirements without silently
   enabling deferred capabilities.
4. Keep the PHIE floor absent until the conflicting binding contract and sources are adjudicated.
5. Preserve queryable `REQUIRED_UNSET`; refuse unsafe legacy one-handle parameter packs.
6. Split exact mnemonic requests from semantic-family requests.
7. Keep correction and reason channels typed; do not invent categorical `f32` codes.
8. Exempt only explicitly enumerated repair module/output/mode paths from masking.
9. Retain the under-five Hampel refusal and keep culling deferred unless the pilot manifest is
   explicitly revised.
10. Authorize narrow categorical CLY provenance infrastructure; require source-scoped sentinel
    handling in which explicit `NoNull` wins.
11. Classify SSC/SSPW under the POR twin contract and define exactly what their unlimited lineage
    bypasses before authorizing any protected-file change.
12. Approve the singular POR branch/limit representation and its complete initial registry; a
    binary bit and an undocumented integer code are both insufficient.

Approval of a decision authorizes the engineering contract; it does not itself mark any affected
requirement done. Each requirement still needs its named test, full gate, evidence update and
separate commit.

## Source and legal intake packet

The following evidence can be supplied without choosing a software design:

- Exact per-value endpoint-library construction custody or primary sources.
- API RP66 V1 writer material and the first-pilot DLIS-export scope decision.
- One primary citation or approved first-principles derivation document per shipping module.
- The source for every accepted deviation-survey index alias.
- A cited whole-index `STEP` tolerance, or an explicit exact-equality rule.
- A versioned source-cited logarithmic curve-family registry.
- A complete Windows single-byte code-page inventory and ambiguity rule.
- Sources for the 19 source-required environmental parameters.
- Counsel dispositions for the endpoint library, chart payloads, branded themes and dependency
  attention items.

Providing a number without its source does not unblock a row. Providing a legal opinion without
the asset and distribution route it covers does not unblock a legal row.

## Maintenance contract

- `gate2-program.json` owns the blocked ID set and counts.
- `requirements.csv` owns the detailed current evidence, blocker and next action per requirement.
- `DECISIONS.md` owns Jauhar's product choices.
- This file owns the one-place human explanation and intake checklist.
- `STATUS.md` links here and carries only the one-minute roll-up.
- The Gate 2 program test fails when this file omits, duplicates or invents a blocked ID, or when
  the dashboard stops linking here.

When a blocker closes, update all four stores in the same requirement increment. Manual and Field
acceptance remain Jauhar-owned and cannot be inferred from a green automated gate.
