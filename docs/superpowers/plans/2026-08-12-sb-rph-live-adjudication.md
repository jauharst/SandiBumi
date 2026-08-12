# SB-RPH Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 52 live `SB-RPH` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 77 chapter acceptance-test intentions, and preserve every unit, endpoint, fluid, frame, anisotropy, image-geometry, core-photo, parameter, source, provenance, and manual-evidence boundary without changing rock-physics or core-photo behavior.

**Architecture:** This is a documentation-only evidence pass. Requirements 001-030 cover elastic state, fluid substitution, fluid/mineral mixing, dry-frame physics, anisotropy, elastic attributes, reflectivity, synthetics, and dispersion. Requirements 031-041 cover array-image conditioning, geometry, dip, porosity, fractures, and statistics. Requirements 042-048 cover the shipped core-photo workflow. Requirements 049-052 impose domain-wide execution, validation, provenance, and accepted-input discipline. The immutable PRD supplies the intended contract, 76 parameters, 5 open items, 12 escalation bullets, 11 refusals, 2 Tier-C independent-derivation items, and 77 named test intentions. Current source, independent tests, manual evidence, and reachable Git history supply the separate live verdict. A neighboring modulus helper, DIO array store, optional-package pixel round trip, UI label, returned note, or historical implementation record does not by itself prove a whole RPH contract.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, tests, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on GPT-5.6 Sol at xhigh with `superpowers:executing-plans`. Do not delegate or spawn subagents unless Jauhar explicitly authorizes it. Petrophysical method interpretation, parameter custody, source boundaries, and final sign-off stay with the primary session. Reserve Sol max for the final all-931-row Gate 1 audit.
- Work only in `D:\XX. SandiBumi`. It MUST remain the sole registered Git worktree. The retired `D:\XX. SandiBumi-check` path remains untouched.
- The accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `a7b9a01c721c57e6b8ef72ef3c61c356a3a13113`; `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution.
- The local planning branch is `codex/g1-sb-rph-plan`. The serial Gate 1 chain remains local and unpushed; do not merge, rebase, rewrite history, push, or open a pull request. After the planning commit, create `codex/g1-sb-rph-adjudication` in the same worktree.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential Rust/TypeScript absence MUST be confirmed across the module registry, dispatch, IPC/UI, curve metadata, reports/exports, tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/25_fluidsub-rockphysics.md`, `docs/record_core_imaging.md`, `docs/record_parallel_lanes.md`, `docs/record_fixes.md`, `docs/research_2026-07/ref_rock_physics.md` as historical rationale rather than current parameter authority, the current verification matrix, takeover receipts/status, and the exact source/tests about to be cited.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 52 ordered rows is `245bf0aac65e0beb80895972623958f96aa8e167084a7c731e8d961e00aaf2ef`. The immutable chapter SHA-256 is `960b34709d6c2b36711b2036ba77b700e194381023baf20f3d2b2ec418ada65e`.
- The chapter and ledger agree on 52 contiguous requirements: `P0=15`, `P1=24`, `P2=11`, `P3=2`. Historical states are `ABSENT=44`, `PARTIAL=1`, and `PRESENT-OK=7`; all 52 live verdicts are still `UNADJUDICATED`. Reverify live behavior independently rather than copying historical status.
- The chapter defines 77 contiguous test IDs, `SB-RPH-T01` through `SB-RPH-T77`. The ledger contains 82 references covering all 77 unique IDs; there is no defined-but-unowned or owned-but-undefined ID. Route all 77 in the receipt and do not alter the immutable ownership field.
- Section 5 contains exactly 76 parameter rows, 43 `ABSENT` values, and one `NON-ADOPTABLE` Greenberg-Castagna coefficient family. Preserve every absence. A vendor table, historical implementation note, round number, compiled behavior, or neighboring method never becomes parameter authority.
- Preserve O-1 through O-5, all 12 escalation bullets, RF-1 through RF-11, both C-3 independent-derivation items, and every section 8.1 through 8.6 disposition. No source-gated advanced method may be inferred from vendor behavior.
- The historical rock-physics record is a source-contamination hazard: it includes formulas, tables, example values, and vendor defaults that PRD v2 deliberately declines to adopt. Use it only to explain earlier decisions; do not promote any number from it.
- The module registry and shared dispatcher contain no Gassmann, fluid-property, elastic-bound, dry-frame, anisotropy, AVO, synthetic, borehole-image, fracture, or generic RPH module. A local dynamic-moduli helper inside the unconventional brittleness calculation is not the complete typed elastic suite.
- DIO can retain array geometry, but RPH has no consumer that performs the image contracts in 031-041. Storage capability is supporting evidence, not method presence.
- Core-photo is real shipped capability, not evidence for the rest of RPH. Its focused tests pass `24/0/8` in the normal run and the eight optional-package tests pass separately, but each requirement must still be checked limb by limb.
- The current core-photo source conflicts with current PRD parameter custody: `CP_ILLUMINATION` and `CP_LANES` are specified absent, while deserialization/UI preselect white light and one lane; unknown light/axis strings and invalid steps silently fall back. Record the live behavior; do not rationalize it from the older implementation record.
- Additional core-photo numeric decisions are active without a matching chapter parameter/source row: the 0.02 depth step, fluorescence colour/brightness bands and 0.95 saturation guard, lane-width threshold, unfold scan grid/coverage/flatness/rival thresholds, and conditioning-advice targets/guards. Do not call a code comment saying “round” a citation.
- `score_unfold_scan` retains `best=Some(...)` on a flat scan and the UI still offers `Use`; SB-RPH-048/T73 require weak or flat evidence to produce no proposal. This is a live specified-behavior divergence, not merely missing prose.
- Core-photo output log-set provenance records only dataset, axis, reverse, lanes, and light. It omits layouts, derivative identities/recipes, step, fluorescence classes, comparison curve, lithology cut/source, minimum bed, unfold/scan, resolution, notes, and several user-confirmed choices. If log-set creation fails, physical curves are still written without provenance. Returned notes are not durable provenance.
- Parameters accepted but unused in a selected core-photo branch include fluorescence classes on white light and lithology-only inputs when lithology is off. Invalid `lith_cut`, step, light, and axis values may be normalized or replaced instead of refused. These are evidence for 050/052, not adjacent remediation work.
- Manual/field core-photo calibration is not listed in the current manual matrix. Record it as `0/0 — not listed`, retain O-1, and never convert automated pixel tests into manual or field verification.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record:

1. branch `codex/g1-sb-rph-adjudication`, created serially from the committed plan;
2. one clean worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, `origin/master`, merge base, and anchor reachability;
4. exactly 52 ledger rows, `SB-RPH-001` through `SB-RPH-052`, with no gap or duplicate;
5. priorities `P0=15`, `P1=24`, `P2=11`, `P3=2`;
6. historical source states `ABSENT=44`, `PARTIAL=1`, `PRESENT-OK=7`;
7. all 52 live as-built, implementation-path, test-class, evidence, manual, dependency, commit-state, decision, action, and reverified fields still unadjudicated or placeholder-only;
8. exactly 77 defined chapter test IDs, 82 ledger references, and all 77 unique IDs owned;
9. exactly 76 parameters, 43 absent values, and one non-adoptable family;
10. exactly 5 open items, 12 escalation bullets, 11 refusals, 2 C-3 items, and section 8.1 through 8.6 traceability custody;
11. takeover summary `688` adjudicated, `243` unadjudicated, and `455` pilot blockers before SB-RPH;
12. core-photo manual evidence `0/0 — not listed`; and
13. fresh focused evidence: normal `coreimage::tests` at `24 passed / 0 failed / 8 ignored`, then its eight optional-package tests separately at `8 passed / 0 failed`.

The only mechanically predictable post-adjudication ledger count is `740` adjudicated and `191` unadjudicated. Do not predict as-built, blocker, test-class, risk, or manual-evidence totals before row-by-row classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-rph.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/coreimage.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/workflow.rs`, `src-tauri/src/unconventional.rs`, `src-tauri/src/curves.rs`, `src-tauri/src/report.rs`, `src-tauri/src/export.rs`, `src-tauri/src/lib.rs`, `src/ipc.ts`, `src/ui/coreConditionDialog.ts`, `src/ui/coreTraceDialog.ts`, current tests, manual evidence, immutable source chapter, historical implementation records, cited local evidence, and reachable Git history
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-RPH-NNN` section per requirement in numeric order. Every section MUST include the specified contract, current implementation, as-built status, release disposition/risk, exact automated evidence class, manual evidence, source/parameter boundary, UI/IPC/provenance surface, history/reachability, blocking decision/dependency, and next action. Separate every observable limb of a compound requirement. Name whether evidence is correctness, characterization, optional-package, structural, manual, or missing; never inflate a helper-level test into whole-contract proof.

## Requirement Evidence Map

| ID | Tests | Exact contract focus | Primary live candidates and adjudication guard |
|---|---|---|---|
| `001` | T01-T03 | One typed SI-with-GPa elastic state and exact boundary conversions | Registry/IPC/curve units and negative RPH search. Generic numeric vectors and scattered units are not a typed state. |
| `002` | T04-T05 | Complete isotropic elastic suite, units, and NaN/refusal domain | Negative suite search; unconventional dynamic-moduli helper is local and incomplete, not reusable RPH output. |
| `003` | T06-T08 | Guarded Gassmann forward/inverse with explicit singular/invalid states | Registry/dispatch/history negative evidence. No textbook re-derivation or copied legacy formula. |
| `004` | T06,T09 | Density and velocity re-synthesis with unchanged dry shear | Negative RPH route; do not count unrelated velocity or density equations. |
| `005` | T10-T11 | Durable method/state/failure provenance and separate semantic flags | Workflow generic provenance plus negative RPH schema. Internal `Result` or notes do not satisfy output provenance. |
| `006` | T12 | Published fluid-property derivation, with direct sourced-property route while coefficients remain absent | Registry/dispatch/IPC negative evidence and E-7. Never infer Batzle-Wang coefficients. |
| `007` | T13-T14 | Named Reuss/Voigt/Brie fluid laws, never a continuous blend | Negative RPH mixer search; unrelated solid mixing is not fluid mixing. |
| `008` | T13-T15 | Brie liquid lumping and monotonic exponent behavior | Negative implementation search; preserve cited exponent but do not treat parameter text as executable proof. |
| `009` | T16 | Voigt/Reuss/VRH plus Hashin-Shtrikman bounds beside every mineral mixture | Negative RPH bounds search; neighboring mineral-volume solver is not an elastic-bounds product. |
| `010` | T17-T18 | Versioned independent endpoint sets and refusal of incomplete shear state | Endpoint registries/history plus O-3. Vendor tables remain untranscribed and missing never becomes zero. |
| `011` | T19 | Distinct critical and depositional porosity identities/provenance | Negative RPH parameter surface; do not carry POR-domain values into RPH. |
| `012` | T20 | Critical-porosity solid/suspension domains and boundary | Negative implementation search; chapter equations are specification, not as-built evidence. |
| `013` | T21 | Krief with cited exponent and guarded domain | Negative route; parameter presence in PRD is not shipped method. |
| `014` | T22-T23 | Hertz-Mindlin cube root and explicit adhesion fraction | Negative route; conflicting vendor variants keep adhesion absent. |
| `015` | T24 | Empirical shear scale separate from adhesion and identity by default | Negative route; no conflation with any existing scale or clamp. |
| `016` | T25 | Soft, stiff, and external dry frames with distinct provenance | Negative route; no adjacent solver substitutes. |
| `017` | T26 | KT/SCA/DEM validity gates and pore configuration | Negative route; primary-equation and aspect-ratio gaps remain source-gated. |
| `018` | T27 | Finite-shear pore fillers only from primary equations | Negative route and E-6; compiled behavior/vendor binary is not derivation authority. |
| `019` | T28 | Inspectable Bayesian interface while solver remains source-gated | Registry/UI/history negative evidence and E-14. No inferred objective. |
| `020` | T29-T30 | Native units and primary-sourced Greenberg-Castagna coefficients | Negative route; coefficient family stays non-adoptable. |
| `021` | T31 | Six semantically addressed shear alternatives and hard-fail unknown | Registry/selector negative evidence; no aliasing one correlation to another. |
| `022` | T32-T33 | Complete Backus TI tensor, density, velocities, isotropic identity | Negative route and E-3; chapter fixture is future correctness evidence, not current proof. |
| `023` | T34-T35 | Explicit FAST/SLOW SH/SV assignment and caution without auto-reassignment | Negative anisotropy UI/IPC; no default assignment. |
| `024` | T36 | Separate TIV, tilted-TIV, orthotropic input contracts | Negative route; do not require inputs absent from the selected contract. |
| `025` | T37-T38 | Positive-definite stiffness guard and weak-anisotropy validity | Negative route; never manufacture a tolerance. |
| `026` | T04,T39 | Full named elastic-attribute suite with identities and units | Negative curve/output search; local YME/PR pair is not the suite. |
| `027` | T40 | Elastic Impedance computed pointwise in its chosen unit system and non-convertible post hoc | Negative route and curve-unit registry. |
| `028` | T41-T42 | Exact Zoeppritz plus separately named validated approximations | Negative dispatch; unavailable branches remain unavailable until O-5 fixtures. |
| `029` | T43 | Synthetic wavelet/sampling/phase/transform state and cited frequency envelope | Negative route; plot or convolution helpers are not a reproducible synthetic workflow. |
| `030` | T44 | Named scalar dispersion distinct from source-gated fitted dispersion | Negative route and C-3. Keep 0.98 scalar separate from absent fit. |
| `031` | T45 | Consume DIO arrays with pad/azimuth/orientation/missingness intact | DIO array IPC/storage exists; no RPH image consumer. Never credit storage as processing. |
| `032` | T46-T47 | Reversible image geometry with displacement/orientation curves and channel offsets | Negative image-correction route and C-3. Core-photo crop/deskew is a different image class. |
| `033` | T48 | Speed correction before harmonization/inpainting/equalization with residuals/masks/windows | Negative borehole-image processing search; core-photo conditioning is not button/pad processing. |
| `034` | T49-T50 | Full-quadrant dip and axial/eigenvector planar means | Negative dip implementation search; generic trig or statistics helpers do not close it. |
| `035` | T51-T52 | `DECL_APPLIED`, value/source stamp, and second-application refusal | Negative stamp/dispatch/UI search. Checkbox or prose is not an interlock. |
| `036` | T53 | Separately named image-porosity routes with interval calibration and absent coefficients | Negative route; no vendor Archie defaults or example regression. |
| `037` | T54-T55 | Calibrated Luthi-Souhaite constants, resistivity convention, radius/sampling, qualitative starts | Negative route, E-4/E-15, and absent constants. |
| `038` | T56-T57 | Opt-in EXCLUDE/CAP_ANGLE/CAP_WEIGHT Terzaghi policies with correct geometry | Negative route; chapter values remain specification until implementation. |
| `039` | T58 | Counts plus P21/P22/P32/P33 with explicit covered geometry | Negative fracture workflow; generic picks/counts are insufficient. |
| `040` | T59 | Separately named pooled and area-weight means | Negative image/fracture statistics; unrelated aggregation helpers do not prove naming/output. |
| `041` | T60-T61 | Required fracture window/step/policy/limit/correction/convention and null empty spacing | Negative writer/schema evidence; zero and infinity are forbidden substitutes for undefined spacing. |
| `042` | T62-T63 | Source-preserving recipe, preview-before-apply, non-cumulative reset/restore | CoreRecipe, source/live image storage, backend preview/bake, UI undo/reset, optional-package round trip. Confirm every limb and do not inflate it into calibration evidence. |
| `043` | T64 | Separate colour/detail provenance and exact consumed derivative | `touches_light/detail` and warnings exist; log-set provenance omits derivative IDs/recipes. T64's required consumed-derivative assertion is not present. |
| `044` | T65 | Finite interval, explicit lane order/direction, point refusal | `plan_lanes` gives strong interval/refusal behavior, but CP_LANES is specified absent while backend/UI default one lane; distinguish geometry success from parameter divergence. |
| `045` | T66-T67 | Declared white/UV choice with distinct mnemonics, thresholds, and conditioning | Explicit UI toggle and output names exist, but backend defaults missing/unknown light to white and uncited fluorescence thresholds ship. T66 auto-detection advice is not proof of declaration. |
| `046` | T68-T69 | `CPHOTO_LITH` proxy name, cut source, no VSH/definitive identity, UV refusal | Naming and no-curve UV behavior exist; cut source/counts live only in returned notes and T69 does not execute the full refusal/write path. Check durable custody. |
| `047` | T70-T71 | Fractional lane/unfold geometry, preserved intervals, separate renderer-independent strips | Fractional layouts and strips exist; strip route uses equal lane count, while unfold/layout/derivative provenance is incomplete. T71 is strong optional-package geometry evidence; T70's “records unfold” limb is not durable. |
| `048` | T72-T73 | Advice reports measurements/reasons, requires application, and produces no proposal for weak/flat evidence | Lane/recipe advice is proposal-like, but active uncited thresholds govern it. Flat unfold retains `best` and UI offers `Use`, directly contradicting T73. |
| `049` | T74 | Shared multi-well runner, per-zone parameters, versioned new sets, cancellation, no import overwrite | Generic shared runner exists; no generic RPH module uses it and core-photo uses separate single-well IPC. Versioned core-photo writes are only one limb. |
| `050` | T75 | Total named input contracts, pre-calculation domain validation, hard-fail unknown options | Other RPH methods are absent; core-photo silently maps unknown axis/light and invalid step/cut values to different valid behavior. Record divergence rather than historical absence alone. |
| `051` | T10,T76 | Reproducible method/version, sourced effective parameters, inputs, conversions, flags, resolution, picks/conventions | Generic workflow and partial core-photo params exist; many effective choices/recipes are omitted and version failure still writes unprovenanced curves. No provenance-only rerun proof. |
| `052` | T77 | Hide/refuse unused parameters and keep QC-only inputs out of calculation provenance | Absent generic RPH routes do not satisfy the contract; core-photo accepts branch-unused inputs and records some presentation/QC inputs ambiguously. Check every exposed field. |

## Test-Intention Routing Contract

- Route T01-T12 to typed state, elastic suite, Gassmann, re-synthesis, failure provenance, and fluid-property source gates.
- Route T13-T31 to fluid/mineral mixing, endpoints, dry-frame models, inversion, empirical shear semantics, units, and unknown-selector refusal.
- Route T32-T44 to Backus/anisotropy, elastic attributes, EI, reflectivity, synthetics, and the scalar-versus-fitted dispersion split.
- Route T45-T61 to DIO array preservation, reversible image correction, pad conditioning, dip/declination, image porosity, aperture, Terzaghi, intensity/statistics, and metadata/null discipline.
- Route T62-T73 individually against the current core-photo source and tests. Mark optional-package tests as optional-package evidence, helper-only tests as characterization unless independently sourced, and missing compound limbs as missing proof.
- Route T74-T77 to shared execution, total validation, reproducibility, and accepted-input discipline across the whole domain, including the shipped core-photo exceptions.
- Every T01-T77 identifier MUST appear in the receipt exactly once as a routed intention even where multiple requirements own it. Ownership duplication in the immutable ledger is not duplicated evidence.

## Execution Tasks

### Task 1: Re-freeze the accepted tree and source inventories

Re-run the baseline/count/hash checks, focused normal and ignored core-photo tests, and targeted absence searches. Stop on branch/worktree drift, source hash drift, or a count mismatch. Record exact commands and outputs in the receipt.

### Task 2: Adjudicate SB-RPH-001 through SB-RPH-010

Classify typed elastic state, complete attributes, guarded substitution, re-synthesis, provenance, fluid properties, mixing, bounds, and endpoints. Preserve every cited/absent boundary and do not count chapter equations as live code.

### Task 3: Adjudicate SB-RPH-011 through SB-RPH-021

Classify porosity identities, dry-frame models, effective-medium/source gates, inversion, empirical shear units, and selector semantics. Keep all primary-source escalations active.

### Task 4: Adjudicate SB-RPH-022 through SB-RPH-030

Classify Backus/anisotropy, elastic attributes, EI, reflectivity, synthetics, and dispersion. Keep unvalidated approximations and Tier-C fitted dispersion unavailable.

### Task 5: Adjudicate SB-RPH-031 through SB-RPH-041

Separate existing DIO array custody from absent RPH image processing. Classify reversible geometry, pad order, dip/declination, porosity, aperture, Terzaghi, fracture metrics/statistics, and writer metadata/null behavior without importing vendor schemas or constants.

### Task 6: Adjudicate SB-RPH-042 through SB-RPH-048

Inspect every shipped core-photo limb against source, UI, IPC, tests, output storage, and provenance. Explicitly record the parameter-default conflicts, uncited thresholds, missing consumed-derivative/cut/unfold custody, flat-scan proposal divergence, optional-package evidence, and absent manual calibration. Do not inherit the chapter's historical `PRESENT-OK` labels.

### Task 7: Adjudicate SB-RPH-049 through SB-RPH-052

Apply the domain-wide contracts to both absent generic RPH modules and shipped core-photo routes. Record partial/versioned behavior without overlooking shared-runner absence, silent option fallback, incomplete provenance, unprovenanced write fallback, and accepted-but-unused parameters.

### Task 8: Write the receipt and update the ledger atomically

Create `docs/takeover/evidence/sb-rph.md`, update only the 52 mutable ledger fields, preserve the six source-owned fields/hash, and update `docs/takeover/STATUS.md`. Include exact pre/post totals, row-level status/disposition/test/risk totals, all 77 tests, 76 parameters, 5 opens, 12 escalations, 11 refusals, 2 C-3 items, section 8 custody, manual `0/0 — not listed`, and the no-production-change boundary.

### Task 9: Self-review the evidence before running the gate

Verify 52 ordered receipt sections, 52 non-placeholder ledger rows, allowed vocabularies, exact test routing, no invented value, no source-owned drift, no identifiers prohibited by repository rules, no unrelated file, and no claim that automated evidence closed manual/field evidence. Recompute the global ledger summary rather than estimating it.

### Task 10: Verify and commit

Run, in order:

```powershell
npx tsc --noEmit
Set-Location src-tauri
cargo check
Set-Location ..
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Require zero failures. Record exact passed/failed/ignored counts and ledger totals, then commit the execution increment locally with a message naming G1-DOM-RPH. Do not push. Recompute the remaining non-GEO Gate 1 inventory and continue serially at Sol xhigh; use Sol max only for the final 931-row audit.

## Mandatory Harsh-Truth Review

- A large, well-tested core-photo subsystem does not make the RPH domain broadly implemented; most of the domain remains absent.
- Historical `PRESENT-OK` is not a verdict. Current source contradicts current absent-parameter and flat-evidence contracts in several places.
- A warning, returned note, or UI label is not durable provenance. A successful physical write after provenance failure is specifically unsafe under SB-RPH-051.
- A “round” threshold is still a parameter. Without a cited source or an explicitly absent/user-confirmed value, it cannot be defended as a product default.
- Running ignored optional-package tests successfully proves that this machine has the package and that the narrow fixture passes. It does not remove the package dependency, establish field calibration, or prove the compound requirement.
- Gate 1 records these facts; it must not repair them, soften the PRD, or pre-spend a Gate 2 product decision.

## Completion Contract

The SB-RPH execution increment is complete only when all 52 rows are adjudicated, all 77 test intentions and 76 parameter rows are routed, all open/escalation/refusal/Tier-C/traceability custody is explicit, the source-owned hash is unchanged, manual evidence remains honestly separate, the full gate is green, and the work is committed locally. The final Gate 1 audit remains a later Sol-max increment over all 931 rows.
