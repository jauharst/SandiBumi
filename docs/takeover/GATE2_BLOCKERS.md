# Gate 2 blocker decision packet

This is the human-readable companion to the machine-owned blocker set in
`gate2-program.json`. The requirement evidence remains authoritative in
`requirements.csv`; product-owner choices remain authoritative in `DECISIONS.md`.

## Live snapshot

- Gate: `G2 - SILENT-WRONGNESS CLOSURE`
- Scope: `222` Gate 2 requirements plus `20` later-gate-only requirements
- Handled: `215 / 222`
- Done: `153`
- Blocked: `62`
- Remaining unhandled: `7`

---

## What I actually need from you (plain language)

There are 62 blocked rows, but they are not 62 separate problems. They collapse into four kinds of
thing, and only two of them need you.

### 1. Decisions only you can make — about 22 rows

These are stopped because answering them is a petrophysical or product call, not an engineering one.
I will not guess these: every one of them is *silently* wrong if guessed, meaning the wrong answer
still computes, still plots, and still ships into a client report with nothing to catch it.

The big ones, each blocking several rows at once:

- **DEC-039 — RULED 2026-08-16. ✅** You chose: record it as a **comment on the curve**, carried
  **per curve version**, since each curve already has versions and each version gets its own comment
  reflecting what the user did on that run. That is the same mechanism you ruled for SB-POR-026, so
  the two share one answer. It is a much simpler answer than the one engineering had drafted (a
  categorical class curve with a closed code registry) and it dissolves that draft's hardest parts:
  nothing to enumerate, no combination rule when several limits bind at once — the text just says so
  — no unknown-code refusal to design, and no categorical-export contract. **One narrow thing left:**
  `log_sets`, which is the per-version record, has no free-text comment column, and adding one means
  editing `db.rs`. **That edit was explicitly authorized on 2026-08-16 (DEC-045)**, so SB-POR-003,
  026, 028, 047 and 048 are now implementation-pending rather than decision-blocked — five rows on
  one authorization. Two constraints survive it: `params_json` must not be reused (that column is the
  run's *parameters*, and mixing a narrative into it would make the two indistinguishable to every
  reader), and `computed_curves` stays deliberately primary-key-less.
  *(DEC-039's own row in `DECISIONS.md` is now closed there too — you added that file to my allowed
  paths on 2026-08-16, so the ruling and the register finally agree.)*
- **DEC-048 — RULED 2026-08-16. ✅** You authorized narrow edits to five protected files —
  `multimin2.rs`, `lrlc.rs`, `multimin.rs`, `satheight.rs`, `montecarlo.rs`. That converts seven rows
  from *waiting on you* to *waiting on me*: **SB-SAT-002, 023, 025, 026, 027** and **SB-CUT-001, 002**.
  Two more were unblocked by it and have already shipped this session: **SB-SAT-028** (a
  non-converged IMTS saturation is now blank instead of its last iterate) and **SB-SAT-034** (the one
  surviving default exponent, the solver's `a = 1`, is gone). Nothing further is needed from you on
  any of these; they are now ordinary engineering work.
- **DEC-025 — where a neutron curve's matrix basis is stored.** A limestone-unit neutron read
  against a sandstone matrix is about 0.04 v/v low in clean water sand. We can convert between
  bases, but nothing records which basis a delivered curve is on, so nothing can refuse a wrong one.
  The owner of that metadata sits outside the approved scope, so you either authorize a narrow seam
  or widen the manifest. **Blocks SB-POR-024, SB-DBM-017.**
- **DEC-018 follow-ups — three rows still point at it.** Two of these look stale to me; see §3.
- **DEC-021/023/024** — you said hold, so SB-POR-010 stays parked. No action unless you want it moved.

### 2. Documents I need you to find — about 7 rows

These are not decisions. They are stopped because the source that fixes the number is not in the
repo and I will not invent it. Each names exactly what would unblock it. This is the same shape as
SB-POR-021 earlier this session: you sent the 1977 Bateman & Konen paper and the block dissolved in
one message. Mostly `SB-DIO-*` and `SB-ENV-004`.

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
| `SB-CORE-015` | Source and scope | LAS semantic self-round-trip is covered, but exact T15 requires an unshipped DLIS writer and the withheld normative RP66 writer source. | Supply the API RP66 V1 writer source and explicitly approve or reject DLIS export for the first pilot. |
| `SB-CORE-044` | Source and legal | Chart payloads, the merged endpoint library and branded themes still lack primary-source, design-around or counsel closure. | Obtain the named source and counsel dispositions; remove or independently re-source uncleared routes; then add the fail-closed release inventory proof. |
| `SB-DBM-002` | DEC-021 | `CARGO_PKG_VERSION` is hand-maintained and cannot identify which module implementation produced a curve. | Choose a build-derived module artefact boundary, derivation, stored representation and cross-machine stability rule. |
| `SB-DBM-005` | Source map | No complete source-controlled map assigns every shipping module a primary citation or approved first-principles derivation document. | Approve or provide the complete registered-module derivation-source map before adding fail-closed registration and run/deliverable custody. |
| `SB-DBM-009` | DEC-022 | Old local timestamps carry no recorded authoring zone, while current paths mix UTC storage and local/UTC rendering. | Classify old values `ZONE_UNKNOWN` or explicitly accept mixed legacy meaning; store all new values as UTC instants and convert only for display. |
| `SB-DBM-010` | Dependency on source map | Parameter sources and legacy labels travel through current outputs, but free-form derivation text is not a source-controlled citation. | Close SB-DBM-005, then establish one machine-readable provenance/sidecar contract across pilot exports, reports and Office deliverables. |
| `SB-DBM-011` | DEC-022 and DEC-023 | The structured relational audit store is absent; exact T11 also needs legacy timestamp treatment and a zone-set identity outside the immutable pilot manifest. | Settle DEC-022 and authorize the narrow zone-set identity/version seam or revise the manifest; then implement the backend-owned atomic audit writer. |
| `SB-DBM-015` | DEC-021, DEC-023 and DEC-024 | A complete rerun manifest requires build, zone-set, conditional stochastic and learned-model identities, including seams owned by deferred capabilities. | Settle build and zone-set identity, then authorize identity schemas/resolvers as narrow infrastructure or revise and reapprove the pilot scope. |
| `SB-DBM-017` | DEC-025 | Neutron matrix scale drives physics, but its typed curve metadata owner is outside the immutable pilot manifest. | Authorize the narrow cited neutron-scale metadata/persistence seam with explicit absence and no inferred contractor, tool, salinity or matrix default. |
| `SB-DBM-025` | DEC-026 and source precedence | Binding project guidance uses PHIE floor `0.001`, while the PRD requires the default absent because held material attests conflicting values. | Explicitly settle source precedence and documentation before registry or behavior changes. Safest interim state is absent with an explicit sourced run value. |
| `SB-DBM-030` | DEC-027 | One contract requires a queryable `REQUIRED_UNSET` row while another says absence-of-row; the requested Geolog export bound is also explicitly non-adopted. | Choose the named-row contract or correct it, and authorize only a cited production magnitude or a clearly test-only verification constant. |
| `SB-DBM-031` | Source declaration | Untyped legacy/imported depth frames have no recoverable datum or source sign. | Require explicit datum at every remaining boundary; migrate only source-declared rows; preserve unknown legacy meaning and refuse cross-datum comparison. |
| `SB-DBM-032` | DEC-028 | The closed installer contract refuses a missing ordinal while the DBM test expects a one-handle legacy row to load with warning. | Define semantic-only and ordinal-only policy without reinterpreting an ordinal or weakening disagreement refusal. Recommended: refuse both one-handle forms. |
| `SB-DBM-041` | Dependency on structured audit | The count divergence is closed, but the inspector cannot claim a complete provenance inventory before `audit_entry` and `audit_detail` exist. | Close DEC-022/023 and SB-DBM-011, then derive the inspector inventory from the complete registered audit/provenance schema. |
| `SB-DIO-007` | Product contract | Empty and explicitly nulled cells collapse to the same arithmetic/export state; the project forbids `Option<f32>`. | Approve a versioned bytemuck source-cell-state mask beside the numeric `f32` array and define preservation or sidecar/refusal per deliverable. |
| `SB-DIO-010` | DEC-029 | The helper resolves a structural index, but no real production reader consumes the cited Geolog flat-ASCII `REFERENCE` declaration. | Authorize a narrow source-faithful Geolog structural reader as pilot infrastructure or explicitly reconcile T15 with the LAS/delimited pilot surface. |
| `SB-DIO-011` | Source | The deviation index alias list has no documented source; the prior test said every while enumerating only three sourced lists. | Supply a named source for every accepted deviation alias, then make T17 discover all index-alias declarations mechanically. |
| `SB-DIO-031` | DEC-030 | One untyped request string conflates exact mnemonic absence with intentional semantic-family fallback. | Approve typed `EXACT_MNEMONIC` and `SEMANTIC_FAMILY` requests; exact never falls back, while family resolution returns the concrete curve identity and rule. |
| `SB-DIO-034` | DEC-030 | Workflow family selection can return a different curve without returning its concrete identity. | Classify every resolver caller under the approved typed request split and prove all resolver surfaces, without disabling legitimate family workflows. |
| `SB-DIO-056` | Source | The writer checks only the first interval and the chapter supplies no whole-index `STEP` comparison tolerance. | Supply a cited tolerance or explicitly adopt an exact-equality contract; never insert a plausible epsilon. |
| `SB-DIO-057` | Source | Logarithmic-family membership is a scientific classification and cannot be inferred from mnemonic or display settings. | Publish a versioned ENV-reviewed family registry with a source for every member, then add pre-commit zero handling and custody. |
| `SB-DIO-060` | Scope dependency | Exact signature-collision proof needs a real BIFF5 table read, but SB-DIO-059 is deferred. | Promote the narrow published-spec BIFF reader or keep BIFF unsupported and re-adjudicate the dependent acceptance boundary. |
| `SB-DIO-061` | Inventory and allocation contract | The current corpus matrix is not a universal reader diagnostic proof and no sourced maximum whole-file allocation exists. | Publish the complete reader inventory and either a cited maximum size or an approved bounded-streaming contract; require location, rule and affected count per reader. |
| `SB-DIO-062` | Source and selection contract | The chapter names plural Windows code pages but not the exact supported pages or how ambiguous bytes select one. | Publish the page inventory and deterministic selection rule, or require an explicit ambiguity decision/refusal; retain the existing UTF and CP1252 controls. |
| `SB-PLT-024` | Legal | Nineteen vendor-derived numeric chart definitions remain imported and bundled; fail-closed rendering does not remove distributed bytes. | Counsel chooses licence, independent primary-source re-derivation, or removal from the paid build/repository. Engineering cannot declare permission. |
| `SB-ENV-004` | Source and specification | Nineteen environmental parameters lack admissible sources and T07 says 31 absent identities while the authoritative parameter table says 32. | Supply/adjudicate the 19 sources and perform a docs-only 31/32 identity reconciliation before implementing the exact inventory tests. |
| `SB-ENV-005` | OI-4 and dependency | Corrected curves have no complete typed list of applied, unavailable, disabled or refused steps, and the persistence owner is open. | Select one authoritative run-level correction manifest linked to output curves, with source-complete step identities, versions, parameters and sample coverage. |
| `SB-ENV-007` | DEC-031 | The requirement needs full, partial, not-applied and refused states plus per-sample step custody; only binary flag polarity is defined. | Choose a typed one-hot binary group or define a categorical type with exact stable codes; settle OI-4 and explicitly permit covered-interval correction while all-uncovered input refuses. |
| `SB-ENV-022` | DEC-031 and DEC-032 | Availability channels cannot identify whether caliper, DRHO or both actually caused bad-hole classification. | Approve a typed binary cause group or a fully enumerated categorical reason type that also distinguishes evaluated-good and neither-evaluable. |
| `SB-ENV-023` | DEC-032 | `abs(DRHO)` collapses positive and negative triggers into the same bit. | Define signed DRHO cause custody in the shared reason representation and prove equal-magnitude opposite-sign controls through persistence/export. |
| `SB-ENV-027` | DEC-033 | The global mask defeats the repair path, but module-wide/category-wide exemptions would also bypass unrelated or non-repair modes. | Approve an exact module/output/mode declaration. Recommended initial scope is `log_predict.SYN` only when `OPT_COMBINE = MAX_RAW`, plus a typed reconstructed-sample marker. |
| `SB-ENV-029` | DEC-025 | Conditioning code cannot validate neutron scale against density matrix because the curve-owned scale metadata seam is deferred. | Authorize the narrow metadata seam, then test matched, mismatched, absent and unknown values at every consumer; keep the uncited numeric offset characterization-only. |
| `SB-ENV-033` | DEC-034 | Exact T42 asks a four-sample window to emit fallback output, while the shipped safety guard and regression refuse an under-five window. | Retain the refusal and re-adjudicate T42 to separate a typed zero-MAD fallback diagnostic from an undersized-window structured refusal, or explicitly accept the riskier four-sample fallback. |
| `SB-ENV-037` | DEC-035 | Exact recovery includes deferred absent culling, and current batch flags do not contain the original values needed for bit-exact restoration. | Add culling to the pilot or re-adjudicate first-pilot recovery to the shipped operations; in either case define one persisted bit-exact change record including missing-value bits. |
| `SB-CLY-001` | DEC-036 | The generic binary precondition flag is not the specified categorical `ENDPOINT_INVALID` reason and carries no zone identity. | Add SB-CLY-031/032 to the pilot or authorize their exact categorical schema as narrow infrastructure, including versioned wire/LAS codes and separate substitution custody. |
| `SB-CLY-034` | DEC-036 and DEC-037 | Provenance export is absent, and a global undeclared `-999` screen would delete legitimate values and violate explicit `NoNull`. | Settle categorical provenance plus a source-scoped rule where `NoNull` wins and matched undeclared values are quarantined behind a named import decision or exact approved automatic policy. |
| `SB-CLY-055` | DEC-036 and DEC-037 | Exact T35 requires the deferred CLY provenance output and an authorized LAS token representation; exact T44 needs a source-scoped undeclared-`-999` policy compatible with per-channel `NoNull`. | Authorize the complete versioned categorical provenance schema as pilot infrastructure or add SB-CLY-031/032 to the manifest, then define the exact CLY source/mnemonic signatures and `NoNull`-first sentinel policy before writing the all-output round trip. |
| `SB-POR-002` | DEC-038 and protected-file boundary | Sonic has no unlimited twin. SSC/SSPW discard pre-limit values inside protected `ssc.rs`, and “unlimited” is ambiguous because upstream component and geometry clamps may already have bound the final value. Independent storage/export proof also depends on SB-POR-004's collision-safe custody. | Decide whether SSC/SSPW are methods under SB-POR-002 or separately typed workflows. If included, define the exact unlimited boundary, finish SB-POR-004, and authorize one narrow `ssc.rs` edit that preserves all existing limited arithmetic while exposing the approved pre-limit lineage. |
| `SB-POR-003` | DEC-039 RULED 2026-08-16 - one narrow db.rs authorization left | The singular branch/limit stream has no complete stable vocabulary, simultaneous-limit encoding, class metadata or unknown-code rule. Binary flags cannot carry branch identity, while unregistered categorical numbers are magic. SSC/SSPW branches and clamps also live in protected `ssc.rs`; T41 depends on later conditioning behavior. | Approve one exact versioned representation and every initial token/code or group member, including combinations and missing/unknown handling; settle DEC-038; then authorize the required narrow protected edits and prove the categorical output through write, reload, reframe and export. |
| `SB-POR-010` | Dependency on SB-DBM-015 | Method identity, parameter values with source and tier, per-output convention and resolved input-curve identities are all already persisted. What is absent is the re-derivability clause: no stored manifest resolves module identity, options and defaults into one replayable record. | Settle DEC-021, DEC-023 and DEC-024, close SB-DBM-015, then prove a POR curve replays from its stored manifest alone without querying any mutable default. |
| `SB-POR-021` | Implementation pending; ESC-POR-8 CLOSED | **Source resolved 2026-08-16.** Jauhar supplied Bateman & Konen, SPWLA Eighteenth Annual Logging Symposium, June 5-8 1977. Appendix B pp.19-21 carries the full derivation and all nine section 5.6 constants verbatim, so they are primary-sourced T1p rather than Geolog's rendering. The analytic evaluator itself is still unwritten. | Implement B-5/B-6/B-7 and the B-9..B-12 pseudo-mineral branches as a **typed deterministic method distinct from the D-N comparison producer** - not another `OPT_XPLOT` mode, or SB-POR-023's quick-look boundary collapses. Pin with the hand-derived witness `phi_x = 0.245219` at rho_b 2.30 / rho_mf 1.00 / phi_N 0.25. |
| `SB-POR-024` | DEC-025 RULED - now only DEC-045 | The N-D crossplot must refuse an NPHI curve whose matrix units are not declared and must state the declared basis in provenance. `nphimat` already performs the conversion, but nothing stores the delivered basis: the live choices are explicit module parameters, not curve metadata, so the refusal has nothing to read. A limestone-unit neutron against a sandstone matrix reads about 0.04 v/v low in clean water sand. | Settle DEC-025 - authorize the narrow SB-ENV-012 typed neutron-scale metadata/persistence seam, or revise the manifest to include it. Then require the declaration at the `phi_dn` boundary, refuse an undeclared or wrong basis by name, and emit the basis in per-output provenance. Do not infer a basis from the mnemonic or supply a default. |
| `SB-POR-025` | Dependency on SB-POR-021 / SB-POR-022 | The fresh-and-salt lever rule has no admissible values to interpolate between. Its endpoint sources are SB-POR-022, the gated chart digitisation, which is DEFERRED and outside the manifest; and SB-POR-021, whose 1977 primary source is now held but whose evaluator is unwritten. `nphimat` is a Prep converter, not a POR endpoint source. | Implement SB-POR-021's evaluator now the source is held, or promote SB-POR-022 into the manifest; then add a typed fluid-condition input with no default and persist the resolved salinity response. |
| `SB-POR-026` | RULED 2026-08-16 - needs a place to put the comment | The wiring is fully scoped - three specs (`phi_den`, `phi_dn`, `phi_son`) and `gascorr`'s shipped `log_in(GAS_FLAG, .., XOVER_FLAG, false)` idiom at `modules.rs:4404` - but its target is undecided. `11_porosity.md:951-952` reads as a new output CURVE; this register's own `next_action` reads as a PROVENANCE record. **Jauhar ruled PROVENANCE RECORD**, so the contract is settled and only the wiring remains - the same state as SB-POR-021. | Declare an optional crossover input on the three `phi_*` specs per `gascorr`'s idiom, CONSUME `condflag`'s flag rather than recomputing it so the coal and washout exclusions survive, and write the result as a **direct comment on the curve's own description** (Jauhar, 2026-08-16: no flag curve, no flag-shaped key - the text an analyst reads in the catalogue and the LAS header), pinning both sides: absent must say nobody looked, never a 0 and never silence. **There is nowhere to write it today**: `computed_curves` is `(well_id, depth, curve_name, value)` and `curve_meta` has no free-text description; the only description that exists is per-MODULE in the manifest, identical every run. Adding the column means editing prohibited `db.rs`. Authorize a narrow `db.rs` edit, or fall back to the ancestry record that already exists near `workflow.rs:920`. |
| `SB-POR-028` | Dependency on SB-POR-003 / DEC-039 | Narrowed this session. The clamp VALUES are cited after all - `11_porosity.md` SS5:1231-1232, tier T1, Geolog `phi_dn.lls` / `phi_dnbk.lls` - and SB-POR-007 closed, so the parameter half is source-ready. The second clause still fails: hitting a clamp must raise SB-POR-003's flag, and that stream does not exist pending DEC-039. | Settle DEC-039, then promote the four literals mode-aware (chart vs Bateman-Konen clamps differ, linking this to SB-POR-021) and prove just-inside versus just-outside on both axes. |
| `SB-POR-044` | PhiMax identity in the smooth form | The DEC-018 reading was stale - the row is in the approved 242. The real block: `11_porosity.md:1048-1050` says the smooth form's **three** parameters ship with no defaults, but its `PhiMax` collides with SandiBumi's `PHIE_MAX`, which already carries a Geolog-sourced 0.3. Reusing it hands the mode a default the chapter forbids; adding a second invents a parameter and leaves two ceilings that can disagree. | Owner rules which `PhiMax` is meant. Then add `SMOOTH_ROLLOFF` as a third `OPT_PHIEMAX` mode with `param_open` parameters that refuse unsupplied, and pin step, smooth and refusal. |
| `SB-POR-045` | RULED 0.001 on 2026-08-16 - implementation pending | The chapter (`:1052-1056`) says the floor value **MUST** ship with no default and be a documented user decision, because IP's manual gives **0.001 and 0.0001 for the same quantity in three places** and the chapter calls it unresolvable. A later product record picked 0.001; SandiBumi hard-codes 0.001. Does the later record supersede the chapter? Only you can say. It bites in tight and zero-porosity intervals - exactly where a pay cutoff sits. **Jauhar ruled 0.001.** The adjudication half is closed. Remaining: the chapter states TWO clauses - no default **and** documented user decision - and the ruling settles the second. Confirm whether 0.001 becomes a cited **default** (matching the SB-POR-011 and SB-POR-023 rulings) or a `param_open` that refuses unsupplied. | Confirm which reading, then move it out of the compile-time `PHIE_FLOOR` constant into source-labelled configuration and prove two values give distinct LIMITED outputs while the UNLIMITED twin is untouched. |
| `SB-POR-047` | Dependency on SB-POR-003 / DEC-039 | The chapter (`:1061-1063`) requires porosity to accept `BADHOLE` as a declared input **and** record its effect **through SB-POR-003**. The declaration is ordinary wiring; the recording names a stream that does not exist pending DEC-039. The row's whole point is *not* depending on the analyst remembering a generic Mask, so the existing mask route does not satisfy it. | Settle DEC-039, then declare and consume `BADHOLE` per `gascorr`'s idiom and prove clean, flagged and flag-absent - absent recording that nobody looked, not a zero. |
| `SB-POR-048` | Per-flag policy ruling, then DEC-039 | The chapter (`:1065-1072`) requires porosity to consume `COAL_FLAG`, `TIGHT_FLAG` and `COND_FLAG` **with defined branch behaviour** - and does not define it. Per flag: does it **mask**, **select a branch**, or only **annotate**? Three different porosity curves from identical data, none cited. Coal is the sharp case: a coal bed has a real very high apparent density porosity, and blanking it versus computing-and-labelling it is a method preference. Recording the outcome then needs DEC-039. | Rule the policy per flag; settle DEC-039; then declare the three as typed inputs, implement without deleting any existing guard, and prove each consumed flag plus an unflagged control. |
| `SB-POR-054` | Which spelling is canonical | The `SB-POR-005`/`SB-POR-040` dependency was stale - neither appears in the requirement text. The real gap is evidenced: density writes `(rho_ma - r)/(rho_ma - rho_fl)` (`modules.rs:3160`) while sonic writes `(d - dt_ma)/(dt_fl - dt_ma)` (`:3392`). Identical numbers, two spellings, no stated convention - which is what the MUST forbids. Choosing one is an API-convention call across many modules, not a petrophysical one. | Pick the canonical spelling and state it at the typed API boundary. The identity test needs no decision and is ready: both published forms give 0.2121212 at rho_ma 2.65 / rho_fl 1.0 / rho_b 2.3, and a numerator-only flip giving -0.2121212 must be caught. |
| `SB-POR-055` | One ruling on `RHO_DSH` | Narrowed twice - both stated blockers cleared this session, and the *no POR source topics* claim is now false since SB-POR-007 and 043 registered nine. What remains is substantive: the chapter records that **`RHO_DSH = 2.65` matches no held source at all** and sets `PHIT_SH` a factor **1.73 low**. Its own rule then requires ABSENT - which stops every porosity run until the user supplies it. That moves PHIE and therefore pay. | Rule `RHO_DSH`: ABSENT per the standing decision, or adjudicate a cited value as DEC-041 did. Then build the universal inventory gate, which needs no decision. |
| `SB-POR-057` | Confirm DEC-042 supersedes its pay clause | Clause 3 says quick-look curves are **excluded by default from pay summation**. **Your ruling (b) / DEC-042 says the opposite** - same curves (`AVERAGE`, `GAS_RMS`), and the pay-eligible behaviour already shipped. This is the **second** record that ruling contradicts; the first was `PILOT_SCOPE.md` item 6. Implementing clause 3 as written would silently revert your ruling in code. | Confirm DEC-042 supersedes clause 3 here too, or state the narrower reading. Then build the comparison-only output class and provenance flag, and pin the ruled pay behaviour from both sides. |
| `SB-SAT-002` | Narrow `multimin2.rs` authorization | **P0.** No effective-porosity Archie exists; the chapter puts the two forms **25.0 saturation units and HCPV 3.15x apart** and calls it the largest cross-tool trap in the domain. **The physics needs nothing** - SB-SAT-023 already supplies the inverse `SwT = Sw(1-Swb) + Swb`, `Swb = 1 - phie/phit`. What blocks it is that `SW_METHOD` codes come from `SwModel::flag_code()` and `SwModel` (prohibited `multimin2.rs`) has no `ArchieEffective` variant. Minting a code in `modules.rs` would break SB-SAT-001 arm D. | Authorize the narrow `multimin2.rs` variant (DEC-040 pattern). Then add the option with `archie_total` as default, lift `SWT` via SB-SAT-023's inverse, and pin that the branches disagree on the reference case and the round-trip is the identity. |
| `SB-SAT-023` | Narrow `multimin2.rs` + `lrlc.rs` authorization | The Juhasz rule exists (`multimin2.rs:456` computes the right `Qvn`) but a blanket post-solve back-out overrides it, and the **inverse pair does not exist at all**. On the dossier fixture `Qvn` 0.42 vs `1-phie/phit` 0.20 makes `SWE` differ by **tens of saturation units while `SWT` matches exactly**. Every part of the fix is in a prohibited file. **Third row on this same authorization** - and SB-SAT-002 needs this row's inverse pair. | Authorize (DEC-040 pattern). Then per-model `Swb`, ship the inverse, record which rule applied, and pin `Swb=1 -> SWE=1`, the round-trip identity, and Juhasz-vs-Archie disagreement on the fixture. |
| `SB-SAT-025` | Narrow `lrlc.rs` authorization | Half is **ready**: `sw_arch` has `SWT_ARCH` but no `SWE_ARCH`, and that fix is in allowed `modules.rs`. The other half - LRLC emitting clamped values only (`lrlc.rs:183`, `:365`) - is prohibited. Held atomic because the MUST covers *every* method. A clipped-only curve cannot distinguish *the rock is wet* from *the model went out of range*. | Authorize `lrlc.rs` alongside the `multimin2.rs` request. Then add `SWE_ARCH` and the LRLC twins, and pin that an out-of-range sample shows the clipped curve at its bound AND the diagnostic beyond it. |
| `SB-SAT-026` | Narrow `lrlc.rs` + `multimin.rs` + `satheight.rs` | As-built said *no method-flag curve exists*; one does - `SW_METHOD`, on 3 of 7 saturation modules. The gap is **coverage**: `sw_rtc`, `sw_imts`, `multimin` and `sw_height` lack it, and all four are in prohibited files. The naming clause (no bare `SW`/`SXO`) is already true and only needs its enforcement test, which needs no authorization. | Authorize the three files so the remaining four modules emit `SW_METHOD`. Then pin both clauses universally, so a future saturation module cannot ship without a method flag. |
| `SB-SAT-027` | Same `lrlc.rs` / `multimin2.rs` authorization | Behaviour is `PRESENT-OK` - the guards are transcribed - but *every* polynomial-form model must run through **one** solver, and the standalone and LRLC iterative paths are separate. `multimin2.rs:391` `sw_cond_root` is a second solver that is **not cross-asserted** against the first. The existing quadratic test compares the module to a closed form, not engine to engine. | Authorize, route every equation through the shared solver, and pin the guard suite plus `n = 2` closed-form equality **engine against engine**. |
| `SB-CUT-001` | Narrow `montecarlo.rs` authorization | The discretisation model is not exposed at all and the clip rule is implemented **three times** - twice in allowed `workflow.rs`, once in prohibited `montecarlo.rs`. The requirement demands **one** implementation shared by all three consumers, so a partial fix would make the engines disagree rather than agree. Worth **0.25 ft per zone contact** on a 0.5 ft grid. Nothing needs inventing - `CENTRED` has four vendor votes. | Authorize `montecarlo.rs` (new file in the blocked set). Then one rule, model parameter defaulting to `CENTRED`, all three consumers routed through it. |
| `SB-CUT-002` | Same `montecarlo.rs` authorization | `PaySummaryRow` (`workflow.rs:2637`) carries no discretisation model and no sample interval; Monte Carlo's `net`/`ntg` bundles (`montecarlo.rs:272-273`) are equally silent. IP ships **two different definitions of Net in one product** under the same heading and labels neither. The sample interval matters separately because net-to-gross is **not scale-invariant** - 0.55 -> 0.75 -> 1.0 across three blocking steps. | Authorize `montecarlo.rs`. Then record model + step on both, carry into report and workbook, and pin that two records computed at different steps are distinguishable. |

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
