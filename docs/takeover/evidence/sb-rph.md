# SB-RPH live adjudication evidence

Date: 2026-08-12  
Branch: `codex/g1-sb-rph-adjudication`  
Plan commit: `23efceda8e3eb3e178aee706ae1754c724b85e8d`  
Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)  
Origin/merge-base anchor: `29833735816d9e5be954afafd9ceb71fd856e3f0`

## Scope and immutable custody

This receipt adjudicates all 52 contiguous requirements, SB-RPH-001 through SB-RPH-052, against the accepted live tree. It changes documentation evidence only. Production code, tests, PRD text, parameter values, manual evidence, and generated artifacts are unchanged.

- Priorities: 15 P0, 24 P1, 11 P2, 2 P3.
- Historical chapter states: 44 ABSENT, 1 PARTIAL, 7 PRESENT-OK. These were evidence inputs, not copied verdicts.
- Frozen chapter SHA-256: `960b34709d6c2b36711b2036ba77b700e194381023baf20f3d2b2ec418ada65e`.
- Frozen six source-owned ledger columns SHA-256: `245bf0aac65e0beb80895972623958f96aa8e167084a7c731e8d961e00aaf2ef`.
- The chapter defines T01 through T77. The immutable ledger has 82 ownership references covering all 77 unique IDs, with no ownership gap or unknown test.
- Parameters: 76 rows, including 43 deliberate ABSENT values and one NON-ADOPTABLE coefficient family.
- Governance: 5 open items, 12 escalation bullets, 11 refusals, 2 C-3 independent-derivation items, and sections 8.1 through 8.6 retained.

## Baseline and executable evidence

- Sole worktree: `D:\\XX. SandiBumi`; execution branch created serially from the committed plan.
- Before this domain: 688 adjudicated, 243 unadjudicated, 455 pilot blockers.
- Registry/dispatch/source/history search found no generic Gassmann, fluid-property, elastic-bound, dry-frame, anisotropy, AVO, synthetic, borehole-image, fracture, or RPH module.
- Normal focused core-photo run: 24 passed, 0 failed, 8 ignored.
- Separate optional-package run: 8 passed, 0 failed. Those tests remain ignored in the normal gate because their subject requires numpy and Pillow.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7. Automated evidence closes none of these.

## Live result

- As built: 41 ABSENT, 4 PARTIAL, 6 PRESENT-DIVERGENT, 1 PRESENT-OK.
- Release: 22 PILOT-BLOCKER, 17 UNDECIDED, 13 DEFERRED.
- Test class: 43 MISSING, 8 CHARACTERIZATION, 1 OPTIONAL-PACKAGE-IGNORED, 0 CORRECTNESS whole-contract proofs.
- Risk: 34 REQUESTED-CAPABILITY, 12 DATA-INTEGRITY, 5 SILENT-WRONGNESS, 1 FIELD-EVIDENCE.
- Mechanically after this receipt: 740 adjudicated, 191 unadjudicated, 477 pilot blockers, 310 undecided, 144 deferred.

## Harsh-truth findings

1. Core-photo is a substantial shipped subsystem, but it is not evidence that the other 41 RPH requirements exist.
2. The current chapter says CP_ILLUMINATION and CP_LANES ship absent, while the live backend/UI select white and one lane. Unknown light/axis and invalid step/cut values also fall back.
3. Active fluorescence, saturation, lane, unfold, and conditioning-advice thresholds are described as round in code but have no chapter parameter/source row. Round is not cited.
4. A flat unfold scan retains a best value and the UI offers Use. The current specified T73 behavior is no proposal for flat evidence.
5. Core-photo log-set provenance omits effective recipes, derivative IDs, layouts, sampling, classes, cuts, unfold and resolution, and physical curves still write after provenance creation fails.
6. Several passing tests defend the earlier behavior. They are characterization evidence of divergence, not correctness evidence for the current contract.

## Unique acceptance-test routing

Each T01-T77 intention is routed once below. Shared immutable ownership is noted in the row sections without creating a second evidence route.

| Test intention | Primary receipt row |
|---|---|
| SB-RPH-T01 | SB-RPH-001 |
| SB-RPH-T02 | SB-RPH-001 |
| SB-RPH-T03 | SB-RPH-001 |
| SB-RPH-T04 | SB-RPH-002 |
| SB-RPH-T05 | SB-RPH-002 |
| SB-RPH-T06 | SB-RPH-003 |
| SB-RPH-T07 | SB-RPH-003 |
| SB-RPH-T08 | SB-RPH-003 |
| SB-RPH-T09 | SB-RPH-004 |
| SB-RPH-T10 | SB-RPH-005 |
| SB-RPH-T11 | SB-RPH-005 |
| SB-RPH-T12 | SB-RPH-006 |
| SB-RPH-T13 | SB-RPH-007 |
| SB-RPH-T14 | SB-RPH-007 |
| SB-RPH-T15 | SB-RPH-008 |
| SB-RPH-T16 | SB-RPH-009 |
| SB-RPH-T17 | SB-RPH-010 |
| SB-RPH-T18 | SB-RPH-010 |
| SB-RPH-T19 | SB-RPH-011 |
| SB-RPH-T20 | SB-RPH-012 |
| SB-RPH-T21 | SB-RPH-013 |
| SB-RPH-T22 | SB-RPH-014 |
| SB-RPH-T23 | SB-RPH-014 |
| SB-RPH-T24 | SB-RPH-015 |
| SB-RPH-T25 | SB-RPH-016 |
| SB-RPH-T26 | SB-RPH-017 |
| SB-RPH-T27 | SB-RPH-018 |
| SB-RPH-T28 | SB-RPH-019 |
| SB-RPH-T29 | SB-RPH-020 |
| SB-RPH-T30 | SB-RPH-020 |
| SB-RPH-T31 | SB-RPH-021 |
| SB-RPH-T32 | SB-RPH-022 |
| SB-RPH-T33 | SB-RPH-022 |
| SB-RPH-T34 | SB-RPH-023 |
| SB-RPH-T35 | SB-RPH-023 |
| SB-RPH-T36 | SB-RPH-024 |
| SB-RPH-T37 | SB-RPH-025 |
| SB-RPH-T38 | SB-RPH-025 |
| SB-RPH-T39 | SB-RPH-026 |
| SB-RPH-T40 | SB-RPH-027 |
| SB-RPH-T41 | SB-RPH-028 |
| SB-RPH-T42 | SB-RPH-028 |
| SB-RPH-T43 | SB-RPH-029 |
| SB-RPH-T44 | SB-RPH-030 |
| SB-RPH-T45 | SB-RPH-031 |
| SB-RPH-T46 | SB-RPH-032 |
| SB-RPH-T47 | SB-RPH-032 |
| SB-RPH-T48 | SB-RPH-033 |
| SB-RPH-T49 | SB-RPH-034 |
| SB-RPH-T50 | SB-RPH-034 |
| SB-RPH-T51 | SB-RPH-035 |
| SB-RPH-T52 | SB-RPH-035 |
| SB-RPH-T53 | SB-RPH-036 |
| SB-RPH-T54 | SB-RPH-037 |
| SB-RPH-T55 | SB-RPH-037 |
| SB-RPH-T56 | SB-RPH-038 |
| SB-RPH-T57 | SB-RPH-038 |
| SB-RPH-T58 | SB-RPH-039 |
| SB-RPH-T59 | SB-RPH-040 |
| SB-RPH-T60 | SB-RPH-041 |
| SB-RPH-T61 | SB-RPH-041 |
| SB-RPH-T62 | SB-RPH-042 |
| SB-RPH-T63 | SB-RPH-042 |
| SB-RPH-T64 | SB-RPH-043 |
| SB-RPH-T65 | SB-RPH-044 |
| SB-RPH-T66 | SB-RPH-045 |
| SB-RPH-T67 | SB-RPH-045 |
| SB-RPH-T68 | SB-RPH-046 |
| SB-RPH-T69 | SB-RPH-046 |
| SB-RPH-T70 | SB-RPH-047 |
| SB-RPH-T71 | SB-RPH-047 |
| SB-RPH-T72 | SB-RPH-048 |
| SB-RPH-T73 | SB-RPH-048 |
| SB-RPH-T74 | SB-RPH-049 |
| SB-RPH-T75 | SB-RPH-050 |
| SB-RPH-T76 | SB-RPH-051 |
| SB-RPH-T77 | SB-RPH-052 |

## Parameter, open-item, and source custody

All 76 parameter rows remain exactly as specified. The following 43 symbols stay absent: `EPS_GASSMANN`, `P_FL`, `T_FL`, `SAL`, `FL_COMP`, `FLUID_MIX`, `ENDPOINT_SET`, `PHI_CRIT`, `PHI_DEPOSITIONAL`, `HM_ADHESION`, `COORDINATION`, `P_EFFECTIVE`, `KT_ASPECT`, `PORE_FRACTIONS`, `MUDROCK_A_B`, `HAN_CFG`, `BACKUS_WINDOW`, `ANISO_MODEL`, `SH_ASSIGNMENT`, `AZIM_FAST_FRAME`, `DFD`, `DTF`, `AVO_METHOD`, `DECL_VALUE`, `DECL_SOURCE`, `IMG_SPEED_MODEL`, `IMG_WINDOW`, `SAND_IS`, `A_IMG`, `M_IMG`, `N_IMG`, `POR_FIT`, `TERZ_POLICY`, `TERZ_CAP_WEIGHT`, `LS_B`, `LS_C`, `LS_R_TERM`, `LS_STDDEV`, `CP_ILLUMINATION`, `CP_LANES`, `CP_UNFOLD`, `CP_LITH_CUT`, and `CP_MIN_BED`. `GC_A_B_C` remains NON-ADOPTABLE pending the primary source.

O-1 through O-5 remain open. All 12 escalation bullets remain active or confirmatory exactly as chaptered. RF-1 through RF-11 remain refusals. The C-3 image speed-correction and fitted-dispersion routes remain primary-source-gated. The 82-row dossier parameter disposition, discrepancy ledger, gap disposition, critique disposition, and completeness statement in sections 8.1 through 8.6 remain source custody; none is converted into as-built behavior.

## Requirement receipts

### SB-RPH-001

- Specified contract: Use one typed SI-with-GPa elastic state. Immutable ownership: T01-T03.
- Current implementation: No typed RPH elastic-state type, registered module, IPC request, or typed curve bundle exists; generic computation and storage move untagged numeric vectors.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the chapter's typed state or three conversion controls.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Typed quantity/unit custody is a prerequisite; no unit value is inferred from neighboring modules.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Typed quantity/unit custody is a prerequisite; no unit value is inferred from neighboring modules.
- Next action: Implement the typed state and all three boundary controls before any pilot RPH calculation is exposed.

### SB-RPH-002

- Specified contract: Derive the complete isotropic elastic suite. Immutable ownership: T04-T05.
- Current implementation: No complete isotropic suite is registered. A private unconventional brittleness helper derives only dynamic YME and PR and does not emit the required RPH attributes.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No whole-suite executable test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Pilot inclusion is undecided; the local helper is not promoted into an RPH contract.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; the local helper is not promoted into an RPH contract.
- Next action: Decide pilot inclusion; if included, implement the full typed suite and named domain refusals.

### SB-RPH-003

- Specified contract: Implement guarded Gassmann forward and inverse. Immutable ownership: T06-T08.
- Current implementation: No Gassmann module, selector, UI/IPC path, forward/inverse state, or output schema exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No executable forward, inverse, singular, or invalid-state fixture exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: EPS_GASSMANN remains absent and no textbook or historical value is adopted.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: EPS_GASSMANN remains absent and no textbook or historical value is adopted.
- Next action: Keep the route unavailable until the guarded method and cited-or-absent tolerance contract are implemented.

### SB-RPH-004

- Specified contract: Re-synthesize density and velocities from the substituted state. Immutable ownership: T09; T06 is shared with 003.
- Current implementation: No substituted-state density/velocity re-synthesis path or dry-shear preservation record exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No executable re-synthesis test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on the typed state and guarded substitution; no adjacent density or velocity formula substitutes.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on the typed state and guarded substitution; no adjacent density or velocity formula substitutes.
- Next action: Decide pilot inclusion after 001/003; if included, implement T09 and preserve the shared T06 ownership.

### SB-RPH-005

- Specified contract: Persist method, state and failure provenance. Immutable ownership: T10-T11.
- Current implementation: No RPH result schema persists method/state/failure fields or separates semantic flags from physical curves.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No output-surface test proves the complete provenance or flag separation.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Generic workflow Result values and notes do not satisfy this contract.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Generic workflow Result values and notes do not satisfy this contract.
- Next action: Define the RPH result/provenance schema before enabling any physical output.

### SB-RPH-006

- Specified contract: Derive fluid properties from published physics. Immutable ownership: T12.
- Current implementation: No Batzle-Wang or direct sourced-property RPH route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No direct-property acceptance or missing-coefficient refusal test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: P_FL, T_FL, SAL, and FL_COMP remain absent; E-7 remains open.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: P_FL, T_FL, SAL, and FL_COMP remain absent; E-7 remains open.
- Next action: Decide pilot inclusion; acquire primary equations before any derived-property route, while keeping direct sourced values a separate contract.

### SB-RPH-007

- Specified contract: Select a named fluid-mixing law. Immutable ownership: T13-T14.
- Current implementation: No RPH fluid mixer or named REUSS/VOIGT/BRIE selector exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No named-law fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: FLUID_MIX remains absent and no continuous blend is adopted.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: FLUID_MIX remains absent and no continuous blend is adopted.
- Next action: Decide pilot inclusion; implement only named laws with T13-T14.

### SB-RPH-008

- Specified contract: Preserve Brie's liquid-lumping semantics. Immutable ownership: T15; T13-T14 are shared with 007.
- Current implementation: No Brie implementation or liquid/gas lumping path exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No executable identity, three-phase, or monotonic-exponent proof exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: The cited exponent remains specification only; no alternate vendor semantics are inferred.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: The cited exponent remains specification only; no alternate vendor semantics are inferred.
- Next action: If 007 is included, add Brie with the unique T15 route and preserve shared ownership.

### SB-RPH-009

- Specified contract: Compute mineral bounds beside every mixture. Immutable ownership: T16.
- Current implementation: No elastic mineral mixer or Voigt/Reuss/VRH/Hashin-Shtrikman bound output exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No two-mineral sweep is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: ENDPOINT_SET remains absent; neighboring mineral-volume solvers are not elastic bounds.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: ENDPOINT_SET remains absent; neighboring mineral-volume solvers are not elastic bounds.
- Next action: Decide pilot inclusion and implement versioned endpoints plus T16 without vendor-table transcription.

### SB-RPH-010

- Specified contract: Govern elastic endpoints without copying vendor tables. Immutable ownership: T17-T18.
- Current implementation: No independent endpoint registry, version identity, or incomplete-shear refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No endpoint-identity or missing-Vs control exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: O-3 remains open; vendor tables stay evidence-only and untranscribed.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: O-3 remains open; vendor tables stay evidence-only and untranscribed.
- Next action: Build source-bearing endpoint custody and refusal behavior before any RPH model can run.

### SB-RPH-011

- Specified contract: Keep critical and depositional porosity distinct. Immutable ownership: T19.
- Current implementation: No RPH dry-frame parameter surface or separate critical/depositional identities exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No swap-one-porosity control exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: PHI_CRIT and PHI_DEPOSITIONAL both remain absent; POR-domain values are not carried over.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: PHI_CRIT and PHI_DEPOSITIONAL both remain absent; POR-domain values are not carried over.
- Next action: Introduce distinct typed absent inputs before dry-frame implementation.

### SB-RPH-012

- Specified contract: Implement critical-porosity and suspension domains. Immutable ownership: T20.
- Current implementation: No critical-porosity or suspension branch exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No equality/above-boundary fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on 010/011 and their absent inputs.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on 010/011 and their absent inputs.
- Next action: Decide pilot inclusion; implement the cited domains and exact branch boundary only after prerequisites.

### SB-RPH-013

- Specified contract: Implement Krief with the cited exponent. Immutable ownership: T21.
- Current implementation: No Krief RPH route exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No executable Krief fixture exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: M_KRIEF remains a chapter-cited future parameter; it is not evidence of shipped capability.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: M_KRIEF remains a chapter-cited future parameter; it is not evidence of shipped capability.
- Next action: Retain absence until the deferred dry-frame increment is scheduled.

### SB-RPH-014

- Specified contract: Require Hertz-Mindlin adhesion explicitly. Immutable ownership: T22-T23.
- Current implementation: No Hertz-Mindlin branch or adhesion input exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No cube-root or adhesion-endpoint proof exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: HM_ADHESION remains absent because cited variants conflict.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: HM_ADHESION remains absent because cited variants conflict.
- Next action: Do not enable the branch until explicit adhesion custody and both tests exist.

### SB-RPH-015

- Specified contract: Keep empirical shear scaling separate. Immutable ownership: T24.
- Current implementation: No modified-HS shear-scale route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No identity/scaled-output provenance test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: SHEAR_SCALE is not conflated with HM adhesion.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: SHEAR_SCALE is not conflated with HM adhesion.
- Next action: Decide pilot inclusion with the dry-frame family; if included, preserve distinct identities.

### SB-RPH-016

- Specified contract: Distinguish soft, stiff and external dry frames. Immutable ownership: T25.
- Current implementation: No soft/stiff/external frame selector or external provenance path exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No branch-separation fixture exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on source-governed endpoints and dry-frame contracts.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on source-governed endpoints and dry-frame contracts.
- Next action: Retain absence until the deferred model family is scheduled.

### SB-RPH-017

- Specified contract: Gate effective-medium models by their validity domains. Immutable ownership: T26.
- Current implementation: No KT/SCA/DEM route or pore-configuration validator exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No boundary/exceedance test is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: KT_ASPECT and PORE_FRACTIONS remain absent; E-6 remains open.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: KT_ASPECT and PORE_FRACTIONS remain absent; E-6 remains open.
- Next action: Retain absence until primary equations and sourced pore configuration are held.

### SB-RPH-018

- Specified contract: Support finite-shear pore fillers only from primary equations. Immutable ownership: T27.
- Current implementation: No finite-shear filler substitution route exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No source-gate refusal is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: E-6 remains open; compiled/vendor behavior is not reconstructed.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: E-6 remains open; compiled/vendor behavior is not reconstructed.
- Next action: Retain explicit unavailability until primary equations are acquired.

### SB-RPH-019

- Specified contract: Specify Bayesian inversion without cloning its solver. Immutable ownership: T28.
- Current implementation: No Bayesian inversion interface or solver exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No inspectable-interface/blocked-solver test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: E-14 remains open; no objective is inferred from an API or binary.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: E-14 remains open; no objective is inferred from an API or binary.
- Next action: Keep deferred and absent until published mathematics is held.

### SB-RPH-020

- Specified contract: Lock empirical shear correlations to their native units. Immutable ownership: T29-T30.
- Current implementation: No Greenberg-Castagna route, unit gate, or coefficient-source refusal exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No wrong-unit or unsourced-coefficient dispatch test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: GC_A_B_C remains NON-ADOPTABLE; raster digits are not adopted.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: GC_A_B_C remains NON-ADOPTABLE; raster digits are not adopted.
- Next action: Decide pilot inclusion; acquire the primary source before any coefficient-bearing route.

### SB-RPH-021

- Specified contract: Keep alternative shear methods semantically addressed. Immutable ownership: T31.
- Current implementation: No six-method shear selector or unknown-method refusal exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No selector inventory test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: MUDROCK_A_B and HAN_CFG remain absent; alternatives are not aliased.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: MUDROCK_A_B and HAN_CFG remain absent; alternatives are not aliased.
- Next action: Retain absence until primary coefficients and method contracts are held.

### SB-RPH-022

- Specified contract: Produce a complete Backus TI tensor. Immutable ownership: T32-T33.
- Current implementation: No Backus module, complete tensor, anisotropy attributes, or isotropic identity path exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No two-layer or identical-layer fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: BACKUS_WINDOW remains absent and E-3 remains confirmatory.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: BACKUS_WINDOW remains absent and E-3 remains confirmatory.
- Next action: Decide pilot inclusion; implement the complete source-governed tensor rather than a partial helper.

### SB-RPH-023

- Specified contract: Make SH/SV assignment an explicit measured decision. Immutable ownership: T34-T35.
- Current implementation: No measured-anisotropy route, FAST/SLOW selection, caution band, or durable decision exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No assignment-swap or caution-band test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: SH_ASSIGNMENT and AZIM_FAST_FRAME remain absent; no auto-assignment is invented.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: SH_ASSIGNMENT and AZIM_FAST_FRAME remain absent; no auto-assignment is invented.
- Next action: Require the measured decision and provenance before any anisotropy output is enabled.

### SB-RPH-024

- Specified contract: Separate TIV, tilted-TIV and orthotropic input contracts. Immutable ownership: T36.
- Current implementation: No anisotropy-model selector or model-specific input contract exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No plain-TIV minimal-input test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: ANISO_MODEL, DFD, and DTF remain absent.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: ANISO_MODEL, DFD, and DTF remain absent.
- Next action: Retain absence until anisotropic substitution is source-qualified.

### SB-RPH-025

- Specified contract: Reject non-positive-definite stiffness states. Immutable ownership: T37-T38.
- Current implementation: No stiffness-state validator or anisotropic substitution route exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No positive-definite refusal or delta-boundary test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: No tolerance beyond the cited DELTA_MAX boundary is invented.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: No tolerance beyond the cited DELTA_MAX boundary is invented.
- Next action: Implement validation before any anisotropic output can be surfaced.

### SB-RPH-026

- Specified contract: Emit the full named elastic-attribute suite. Immutable ownership: T39; T04 is shared with 002.
- Current implementation: No RPH attribute bundle exists; the local unconventional helper does not emit the named suite or units.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No complete output-identity test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on 001/002 typed state and units.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on 001/002 typed state and units.
- Next action: Decide pilot inclusion; if included, emit every named attribute from one typed state.

### SB-RPH-027

- Specified contract: Treat Elastic Impedance as unit-system-dependent. Immutable ownership: T40.
- Current implementation: No Elastic Impedance route, unit-system stamp, or post-hoc conversion refusal exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No conversion-refusal test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: No pointwise EI factor is inferred from plotting or unit helpers.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: No pointwise EI factor is inferred from plotting or unit helpers.
- Next action: Decide pilot inclusion; implement only with explicit unit-system custody.

### SB-RPH-028

- Specified contract: Provide exact and declared approximate reflectivity. Immutable ownership: T41-T42.
- Current implementation: No Zoeppritz, Shuey, Aki-Richards, or approximation dispatcher exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No normal-incidence or unavailable-branch test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: O-5 remains open; unvalidated branches stay unavailable.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: O-5 remains open; unvalidated branches stay unavailable.
- Next action: Retain absence until primary numeric fixtures are held.

### SB-RPH-029

- Specified contract: Build synthetics from explicit wavelet and sampling state. Immutable ownership: T43.
- Current implementation: No synthetic workflow records wavelet, phase, sampling, reflectivity, or depth/time transform.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No Ricker-boundary fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: The cited frequency envelope remains specification only.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: The cited frequency envelope remains specification only.
- Next action: Retain absence until the synthetic workflow is scheduled.

### SB-RPH-030

- Specified contract: Keep simple dispersion distinct from fitted dispersion. Immutable ownership: T44.
- Current implementation: Neither SIMPLE_FACTOR nor a fitted dispersion route is registered.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No scalar identity/source-gate test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: DISP_SIMPLE is distinct from the source-gated C-3 fit; no vendor fit is inferred.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: DISP_SIMPLE is distinct from the source-gated C-3 fit; no vendor fit is inferred.
- Next action: Keep fitted dispersion absent; revisit the scalar only in a separately named deferred increment.

### SB-RPH-031

- Specified contract: Consume array logs without flattening their geometry. Immutable ownership: T45.
- Current implementation: DIO stores array samples and geometry, but no RPH image-frame consumer loads pad/azimuth/orientation/missingness for processing.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No RPH round-trip test consumes the DIO array contract.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: O-4 remains open; storage presence is not processing capability.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: O-4 remains open; storage presence is not processing capability.
- Next action: Decide borehole-image pilot inclusion and schema before adding an RPH consumer.

### SB-RPH-032

- Specified contract: Make image geometry corrections reversible. Immutable ownership: T46-T47.
- Current implementation: No borehole-image speed/channel-offset/residual/orientation correction or reversible displacement output exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No correction/inversion or channel-offset provenance test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: C-3 remains source-gated; core-photo crop/deskew is a different image class.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: C-3 remains source-gated; core-photo crop/deskew is a different image class.
- Next action: Keep unavailable until primary motion-correction sources and reversible output custody exist.

### SB-RPH-033

- Specified contract: Condition buttons and pads after speed correction. Immutable ownership: T48.
- Current implementation: No button/pad harmonization, repair, inpainting, residual, or ordered correction route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No operation-order/mask/window test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on 031/032; core-photo conditioning is not borehole-image conditioning.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on 031/032; core-photo conditioning is not borehole-image conditioning.
- Next action: Decide pilot inclusion only after the image-geometry foundation.

### SB-RPH-034

- Specified contract: Recover dip direction with full quadrants. Immutable ownership: T49-T50.
- Current implementation: No borehole-image dip recovery or axial planar-mean route exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No four-quadrant or opposite-pole test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: No generic trig/statistics helper is credited without the RPH observable path.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: No generic trig/statistics helper is credited without the RPH observable path.
- Next action: Decide pilot inclusion; implement atan2 and axial means together.

### SB-RPH-035

- Specified contract: Prevent magnetic-declination double application. Immutable ownership: T51-T52.
- Current implementation: No navigation stamp carrying DECL_APPLIED/value/source or second-application refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No stamped interlock test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: DECL_VALUE and DECL_SOURCE remain absent; no checkbox substitutes for provenance.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: DECL_VALUE and DECL_SOURCE remain absent; no checkbox substitutes for provenance.
- Next action: Implement the stamp-derived interlock before any corrected navigation output.

### SB-RPH-036

- Specified contract: Calibrate image porosity to interval electrical parameters. Immutable ownership: T53.
- Current implementation: No Archie-per-pixel, conductivity-fit, or Newberry image-porosity route exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No method-separation/calibration test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: A_IMG, M_IMG, N_IMG, and POR_FIT remain absent; vendor/example values are not adopted.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: A_IMG, M_IMG, N_IMG, and POR_FIT remain absent; vendor/example values are not adopted.
- Next action: Retain absence until interval calibration and source custody are scheduled.

### SB-RPH-037

- Specified contract: Require calibrated fracture-aperture constants and convention. Immutable ownership: T54-T55.
- Current implementation: No Luthi-Souhaite aperture route or RM/RMF convention exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No constant/convention sensitivity test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: LS_B, LS_C, LS_R_TERM, and LS_STDDEV remain absent; E-4/E-15 remain open.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: LS_B, LS_C, LS_R_TERM, and LS_STDDEV remain absent; E-4/E-15 remain open.
- Next action: Decide pilot inclusion; acquire primary source and calibration before implementation.

### SB-RPH-038

- Specified contract: Expose all three Terzaghi policies. Immutable ownership: T56-T57.
- Current implementation: No Terzaghi correction route or required policy selector exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No angle/weight policy test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: TERZ_POLICY and TERZ_CAP_WEIGHT remain absent; the opt-in state alone is not capability.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: TERZ_POLICY and TERZ_CAP_WEIGHT remain absent; the opt-in state alone is not capability.
- Next action: Decide fracture-workflow inclusion; if included, implement all policies without a default selection.

### SB-RPH-039

- Specified contract: Compute fracture intensity with explicit geometry. Immutable ownership: T58.
- Current implementation: No P21/P22/P32/P33 output or covered-arc contract exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No geometry/refusal fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Depends on image geometry, aperture, and pick coverage.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Depends on image geometry, aperture, and pick coverage.
- Next action: Retain absence until the fracture workflow is scheduled.

### SB-RPH-040

- Specified contract: Name pooled and area-weight statistics separately. Immutable ownership: T59.
- Current implementation: No RPH image/fracture aggregation route emits the two named means.
- Verdict: `ABSENT`; release `DEFERRED`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No unequal-area fixture is executable.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: No unrelated mean helper is credited as the output contract.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: No unrelated mean helper is credited as the output contract.
- Next action: Retain absence until the image/fracture statistics workflow is scheduled.

### SB-RPH-041

- Specified contract: Refuse metadata-free fracture outputs. Immutable ownership: T60-T61.
- Current implementation: No fracture density/spacing writer or required metadata schema exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No window-change or empty-spacing test exists.
- Manual evidence: rock-physics 0/0 and not listed; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Undefined empty spacing must remain null; no zero/infinity substitute is adopted.
- UI/IPC/provenance surface: No registered end-user RPH route or durable RPH output surface closes this contract.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Undefined empty spacing must remain null; no zero/infinity substitute is adopted.
- Next action: Define metadata and null semantics before any fracture output can be written.

### SB-RPH-042

- Specified contract: Preserve non-destructive core-photo conditioning. Immutable ownership: T62-T63.
- Current implementation: Source bytes, source metadata, explicit CoreRecipe, backend preview/bake, identity restore, and UI undo/reset are implemented; the optional-package crop/reset fixture passed on this machine.
- Verdict: `PRESENT-OK`; release `PILOT-BLOCKER`; risk `FIELD-EVIDENCE`.
- Automated evidence: `OPTIONAL-PACKAGE-IGNORED` — T62-T63 are direct optional-package round trips; normal gate still ignores them and field calibration is absent.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: O-1 core-photo calibration remains open even though the storage/edit contract is present.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: O-1 core-photo calibration remains open even though the storage/edit contract is present.
- Next action: Execute representative manual source/preview/apply/reset/report evidence in Gate 4; do not change the present contract.

### SB-RPH-043

- Specified contract: Separate colour correction from detail-changing operations. Immutable ownership: T64.
- Current implementation: CoreRecipe separates light/detail effects and run notes warn on the relevant transforms, but persisted log-set params omit consumed image/derivative IDs and recipes.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — Existing tests characterize classification/warnings but do not assert the consumed derivative required by T64.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: A returned warning is not durable derivative provenance.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: A returned warning is not durable derivative provenance.
- Next action: Persist exact consumed derivative and recipe identities, then add the missing T64 whole-contract test.

### SB-RPH-044

- Specified contract: Require interval geometry before core-log extraction. Immutable ownership: T65.
- Current implementation: plan_lanes refuses point/no-base and half-labelled layouts and preserves explicit barrel intervals, but CoreLogSpec/UI default lanes to one and axis/direction have fallback states instead of requiring the absent CP_LANES decision.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — Current tests strongly pin interval/refusal behavior while also characterizing one-lane fallback; they do not prove the current no-default contract.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: CP_LANES is specified absent; the shipped one-lane default is not source-authorized.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: CP_LANES is specified absent; the shipped one-lane default is not source-authorized.
- Next action: Remove silent geometry defaults or require explicit confirmation, preserve the point refusal, and add T65 from both sides.

### SB-RPH-045

- Specified contract: Keep white-light and ultraviolet meanings distinct. Immutable ownership: T66-T67.
- Current implementation: UI sends explicit white/UV and output mnemonics/conditioning warnings differ, but Rust defaults omitted or unknown light to white and ships uncited fluorescence bands/guards.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — Existing tests pin distinct outputs and explicitly pin unknown light as white; that passing control characterizes the divergence.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: CP_ILLUMINATION is specified absent. Default fluorescence thresholds and the 0.95 guard have no chapter parameter/source row.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: CP_ILLUMINATION is specified absent. Default fluorescence thresholds and the 0.95 guard have no chapter parameter/source row.
- Next action: Require declared known illumination, source or remove active thresholds, and replace the old fallback characterization with T66-T67 contract tests.

### SB-RPH-046

- Specified contract: Keep image-derived lithology a labeled proxy. Immutable ownership: T68-T69.
- Current implementation: CPHOTO_LITH naming, 0/1 codes, Otsu proposal, no VSH identity, and UV no-write note exist; cut source/counts are returned notes rather than durable output provenance, and the UV test does not execute the writer path.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — Current tests pin class arithmetic and the absence of a DARK measure under UV, not the full recorded-cut/refusal contract.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: CP_LITH_CUT remains absent/Otsu-proposed as specified; its effective source must be persisted.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: CP_LITH_CUT remains absent/Otsu-proposed as specified; its effective source must be persisted.
- Next action: Persist cut source/effective value with the curve and add end-to-end white and UV writer controls.

### SB-RPH-047

- Specified contract: Preserve fractional lane geometry and inspectable strips. Immutable ownership: T70-T71.
- Current implementation: Fractional PlateLayout and separate depth-registered strip images exist; strip builds preserve top/base and the optional round trip passes, but strip input still uses equal lane count and output provenance omits layouts, unfold, derivative, and effective geometry.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `CHARACTERIZATION` — T71's optional-package geometry test is strong for strip/read agreement; existing unfold tests do not prove durable T70 recording.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: CP_UNFOLD remains user supplied/proposed, but its applied value and geometry are not fully persisted.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: CP_UNFOLD remains user supplied/proposed, but its applied value and geometry are not fully persisted.
- Next action: Unify strip/trace on persisted fractional layout and record effective unfold/derivative geometry before pilot use.

### SB-RPH-048

- Specified contract: Keep automatic core advice proposal-only. Immutable ownership: T72-T73.
- Current implementation: Lane and conditioning advice report measurements and require later run/apply, but uncited thresholds govern proposals; score_unfold_scan keeps best on a flat scan and the UI offers Use, contradicting T73's no-proposal rule.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — The passing flat-scan test explicitly asserts best is Some and therefore characterizes the old behavior rather than current correctness.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Advisor, lane, fluorescence, and unfold thresholds lack current chapter parameter/source custody.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Advisor, lane, fluorescence, and unfold thresholds lack current chapter parameter/source custody.
- Next action: Make flat/weak evidence return no proposal, source or remove thresholds, and add T72-T73 controls without hidden application.

### SB-RPH-049

- Specified contract: Make every method batch-safe and versioned. Immutable ownership: T74.
- Current implementation: Generic workflow supports multi-well/zone/cancellation/versioning, and core-photo writes versioned sets, but core-photo uses a separate single-well IPC route and no generic RPH module is registered.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No T74 two-well/two-zone/cancellation RPH test exists.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: Core-photo versioning is one limb; it does not prove shared execution.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: Core-photo versioning is one limb; it does not prove shared execution.
- Next action: Move surfaced RPH methods behind the shared runner and add T74 before pilot release.

### SB-RPH-050

- Specified contract: Validate method-specific inputs before calculation. Immutable ownership: T75.
- Current implementation: Many generic RPH methods are absent, while shipped core-photo maps unknown light/axis and invalid step/cut inputs to different valid behavior instead of hard-failing before calculation.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — Existing tests pin invalid-step and unknown-light fallback; no T75 total-contract test exists.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: No new validation bound is invented; known selectors must reject unknown values and uncited defaults must be removed.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: No new validation bound is invented; known selectors must reject unknown values and uncited defaults must be removed.
- Next action: Implement one total validator for every core-photo selector/domain, then apply it to future RPH methods.

### SB-RPH-051

- Specified contract: Persist enough provenance to reproduce every number. Immutable ownership: T76; T10 is shared with 005.
- Current implementation: Core-photo log sets record dataset, axis, reverse, lanes, and light only; they omit effective recipes/layouts/step/classes/cut/unfold/resolution and write physical curves without provenance when log-set creation fails.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No provenance-only rerun reproduces values; T76 is absent.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: All effective parameters and sources must be durable; write-on-provenance-failure violates the MUST contract.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: All effective parameters and sources must be durable; write-on-provenance-failure violates the MUST contract.
- Next action: Make provenance creation atomic with physical output and add T76 before any RPH output is pilot-releasable.

### SB-RPH-052

- Specified contract: Never accept and ignore a parameter. Immutable ownership: T77.
- Current implementation: Core-photo accepts fluorescence inputs on white runs, lithology-only inputs when lithology is off, and several invalid values that are normalized or replaced; no universal accepted-input audit exists.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `CHARACTERIZATION` — Current tests expose selected fallback behavior but no T77 branch-unused refusal test exists.
- Manual evidence: core-photo calibration 0/0 and not listed; image-data 0/30; workflow 0/23; processing-history 0/7; no checked RPH closure.
- Source/parameter boundary: QC-only versus calculation inputs are not consistently typed or separated.
- UI/IPC/provenance surface: The shipped core-photo route was inspected through Rust, TypeScript IPC/UI, storage, returned notes, and tests.
- History/reachability: accepted implementation anchor is reachable; targeted reachable-history review found no later implementation that closes this row.
- Decision/dependency: QC-only versus calculation inputs are not consistently typed or separated.
- Next action: Hide or refuse branch-unused inputs, label QC-only fields, and add an exhaustive T77 selector audit.

## Self-review contract

- 52 ordered requirement sections are present.
- The source-owned fields and frozen hash are unchanged.
- All 77 test intentions are routed once; shared ownership is preserved.
- No parameter was invented, rounded, borrowed, or activated.
- No optional-package test was relabeled as normal-gate or field evidence.
- No production, test, PRD, manual-evidence, or unrelated receipt file changed.
- Final ledger and full-gate counts are recorded in the dashboard after verification.
