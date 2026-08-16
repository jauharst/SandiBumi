# Gate 2 blocker decision packet

This is the human-readable companion to the machine-owned blocker set in
`gate2-program.json`. The requirement evidence remains authoritative in
`requirements.csv`; product-owner choices remain authoritative in `DECISIONS.md`.

## Live snapshot

- Gate: `G2 - SILENT-WRONGNESS CLOSURE`
- Scope: `222` Gate 2 requirements plus `20` later-gate-only requirements
- Handled: `165 / 222`
- Done: `122`
- Blocked: `43`
- Remaining unhandled: `57`
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
| `SB-POR-003` | DEC-039, DEC-038 and protected-file boundary | The singular branch/limit stream has no complete stable vocabulary, simultaneous-limit encoding, class metadata or unknown-code rule. Binary flags cannot carry branch identity, while unregistered categorical numbers are magic. SSC/SSPW branches and clamps also live in protected `ssc.rs`; T41 depends on later conditioning behavior. | Approve one exact versioned representation and every initial token/code or group member, including combinations and missing/unknown handling; settle DEC-038; then authorize the required narrow protected edits and prove the categorical output through write, reload, reframe and export. |
| `SB-POR-008` | Protected-file boundary and deferred consumer | `phi_den` and `phi_dn` already share the formation-water helper and `phi_son` has no equivalent term, but `ssc.rs:259` and `ssc.rs:464` each define a local `phit_sh` from FLUID density where the requirement requires FORMATION WATER density. That file is prohibited and exposes no `RHO_W` at all. The CLY export target `clsr_porosity_corrected` (SB-CLY-044) is outside the approved manifest. The two forms agree at the shipped defaults and separate only once salt water is selected, which is why it is silent. | Authorize a narrow `ssc.rs` edit adding a formation-water parameter and routing SSC/SSPW through the shared helper while preserving their existing limited arithmetic, or explicitly re-adjudicate SB-POR-008 to the `modules.rs` paths while SSC/SSPW remain first-pilot exclusions; the CLY export arm additionally needs SB-CLY-044 admitted or re-scoped. |

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
