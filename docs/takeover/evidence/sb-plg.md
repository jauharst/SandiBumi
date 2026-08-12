# SB-PLG live adjudication evidence

Date: 2026-08-12
Branch: `codex/g1-sb-plg-adjudication`
Plan commit: `279347dcddb3da2376b2c9ade8fce74a96a86c19`
Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)
Origin/merge-base anchor: `29833735816d9e5be954afafd9ceb71fd856e3f0`

## Scope and immutable custody

This receipt adjudicates all 48 contiguous requirements, SB-PLG-001 through SB-PLG-048, against the accepted live tree. It changes documentation evidence only. Production code, tests, PRD text, parameter values, manual evidence, and generated artifacts are unchanged.

- Priorities: 24 P0, 17 P1, 7 P2.
- Historical chapter states: 43 ABSENT and 5 PARTIAL. These were evidence inputs, not copied verdicts.
- Frozen chapter SHA-256: `4ff1d534977621784494f444bc1a49d7d10ca2b425e4d38637d72e7e6fa0b6b9`.
- Frozen six source-owned ledger columns SHA-256: `437fc033b9ff179a4a85f8aeaa91677b6934d4ec9df86d95d35e7b8a4f352b31`.
- The chapter defines T01 through T68. The immutable ledger has 69 ownership references covering all 68 unique IDs; T31 is shared by 019 and 048.
- Parameters: 132 rows, including 32 deliberate ABSENT values.
- Governance: 10 open items, 10 escalations, 18 refusal bullets, no Tier-C item, and 239 section-8 traceability rows retained.

## Baseline and executable evidence

- Sole worktree: `D:\XX. SandiBumi`; execution branch created serially from committed plan `279347d`.
- Before this domain: 805 adjudicated, 126 unadjudicated, 521 pilot blockers.
- Registry/dispatch/command/UI/report/export/test/history search found no production-specific `prodlog`, `cement_eval`, or `casing_integrity` implementation candidate.
- Five focused generic seam tests passed once each: array sample-order round trip, wide-table axis parsing, unknown-unit verbatim retention, generic null recognition, and generic log-set versioning.
- Those tests prove only narrower infrastructure. None asserts a complete SB-PLG calculation, domain gate, sensor schema, typed unit chain, report, or end-to-end axis/validity contract.
- Manual evidence: production logging/cement evaluation/casing integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these.

## Live result

- As built: 41 ABSENT, 6 PARTIAL, 1 PRESENT-DIVERGENT.
- Release: 1 DEFERRED, 41 PILOT-BLOCKER, 6 UNDECIDED.
- Test class: 48 MISSING, 0 qualifying correctness/characterization/optional-package whole-contract proofs.
- Risk: 22 DATA-INTEGRITY, 1 DEGRADED-RESULT, 1 REQUESTED-CAPABILITY, 24 SILENT-WRONGNESS.
- Mechanically after this receipt: 853 adjudicated, 78 unadjudicated, 562 pilot blockers, 220 undecided, 149 deferred.

## Harsh-truth findings

1. The chapter describes three separately gated products, not one missing menu item: production flow, cement evaluation, and casing integrity.
2. No production-specific implementation or executable acceptance test exists. Generic curve/array plumbing cannot prove domain interpretation.
3. The array axis is accepted and written to DuckDB, then silently discarded by the normal reader and IPC. Values can reach display without the physical coordinate that says what each bin means.
4. Current absence avoids fabricated phase rates, but SB-PLG-006 remains absent because the specified safe behavior is to compute Vapp and explicitly refuse unsupported downstream quantities by name.
5. Thirty-two parameters deliberately ship absent. Familiar equations, vendor defaults, and neighboring domains do not authorize filling them.
6. Production logging, cement evaluation, and casing integrity have no listed manual capability rows. That is 0/0 coverage, not evidence of completion.

## Unique acceptance-test routing

Each T01-T68 intention is routed once. Shared immutable ownership is shown in one row and does not create a second evidence route.

| Test intention | Receipt owner(s) | Live class |
|---|---|---|
| SB-PLG-T01 | SB-PLG-001 | MISSING |
| SB-PLG-T02 | SB-PLG-001 | MISSING |
| SB-PLG-T03 | SB-PLG-002 | MISSING |
| SB-PLG-T04 | SB-PLG-002 | MISSING |
| SB-PLG-T05 | SB-PLG-002 | MISSING |
| SB-PLG-T06 | SB-PLG-003 | MISSING |
| SB-PLG-T07 | SB-PLG-003 | MISSING |
| SB-PLG-T08 | SB-PLG-003 | MISSING |
| SB-PLG-T09 | SB-PLG-004 | MISSING |
| SB-PLG-T10 | SB-PLG-004 | MISSING |
| SB-PLG-T11 | SB-PLG-005 | MISSING |
| SB-PLG-T12 | SB-PLG-006 | MISSING |
| SB-PLG-T13 | SB-PLG-007 | MISSING |
| SB-PLG-T14 | SB-PLG-007 | MISSING |
| SB-PLG-T15 | SB-PLG-008 | MISSING |
| SB-PLG-T16 | SB-PLG-008 | MISSING |
| SB-PLG-T17 | SB-PLG-009 | MISSING |
| SB-PLG-T18 | SB-PLG-010 | MISSING |
| SB-PLG-T19 | SB-PLG-011 | MISSING |
| SB-PLG-T20 | SB-PLG-012 | MISSING |
| SB-PLG-T21 | SB-PLG-012 | MISSING |
| SB-PLG-T22 | SB-PLG-012 | MISSING |
| SB-PLG-T23 | SB-PLG-013 | MISSING |
| SB-PLG-T24 | SB-PLG-014 | MISSING |
| SB-PLG-T25 | SB-PLG-015 | MISSING |
| SB-PLG-T26 | SB-PLG-016 | MISSING |
| SB-PLG-T27 | SB-PLG-016 | MISSING |
| SB-PLG-T28 | SB-PLG-017 | MISSING |
| SB-PLG-T29 | SB-PLG-017 | MISSING |
| SB-PLG-T30 | SB-PLG-018 | MISSING |
| SB-PLG-T31 | SB-PLG-019, SB-PLG-048 | MISSING |
| SB-PLG-T32 | SB-PLG-019 | MISSING |
| SB-PLG-T33 | SB-PLG-020 | MISSING |
| SB-PLG-T34 | SB-PLG-021 | MISSING |
| SB-PLG-T35 | SB-PLG-021 | MISSING |
| SB-PLG-T36 | SB-PLG-022 | MISSING |
| SB-PLG-T37 | SB-PLG-023 | MISSING |
| SB-PLG-T38 | SB-PLG-023 | MISSING |
| SB-PLG-T39 | SB-PLG-024 | MISSING |
| SB-PLG-T40 | SB-PLG-025 | MISSING |
| SB-PLG-T41 | SB-PLG-026 | MISSING |
| SB-PLG-T42 | SB-PLG-027 | MISSING |
| SB-PLG-T43 | SB-PLG-027 | MISSING |
| SB-PLG-T44 | SB-PLG-028 | MISSING |
| SB-PLG-T45 | SB-PLG-029 | MISSING |
| SB-PLG-T46 | SB-PLG-030 | MISSING |
| SB-PLG-T47 | SB-PLG-031 | MISSING |
| SB-PLG-T48 | SB-PLG-032 | MISSING |
| SB-PLG-T49 | SB-PLG-032 | MISSING |
| SB-PLG-T50 | SB-PLG-033 | MISSING |
| SB-PLG-T51 | SB-PLG-034 | MISSING |
| SB-PLG-T52 | SB-PLG-035 | MISSING |
| SB-PLG-T53 | SB-PLG-035 | MISSING |
| SB-PLG-T54 | SB-PLG-036 | MISSING |
| SB-PLG-T55 | SB-PLG-036 | MISSING |
| SB-PLG-T56 | SB-PLG-037 | MISSING |
| SB-PLG-T57 | SB-PLG-038 | MISSING |
| SB-PLG-T58 | SB-PLG-039 | MISSING |
| SB-PLG-T59 | SB-PLG-040 | MISSING |
| SB-PLG-T60 | SB-PLG-041 | MISSING |
| SB-PLG-T61 | SB-PLG-042 | MISSING |
| SB-PLG-T62 | SB-PLG-043 | MISSING |
| SB-PLG-T63 | SB-PLG-043 | MISSING |
| SB-PLG-T64 | SB-PLG-044 | MISSING |
| SB-PLG-T65 | SB-PLG-045 | MISSING |
| SB-PLG-T66 | SB-PLG-046 | MISSING |
| SB-PLG-T67 | SB-PLG-047 | MISSING |
| SB-PLG-T68 | SB-PLG-048 | MISSING |

## Parameter, open-item, and source custody

All 132 parameter rows remain exactly as specified. The following 32 symbols stay absent: `SPIN_CPR`, `SPIN_DISC`, `SPIN_THRESHOLD`, `PASS_WEIGHTS`, `DEV_WEIGHT_TABLE`, `VMIX_METHOD`, `SLIP_METHOD`, `GAS_REGIME`, `ARRAY_DIAM_UNIT`, `ARRAY_CLOCKWISE`, `SENSOR_AZ`, `SENSOR_RADIUS`, `SENSOR_LENGTH`, `COV_CUTOFF`, `BOND_INTERP`, `AI_LIQUID`, `ATT_ENDPOINTS`, `USE_BOND`, `USE_CHANNEL`, `DISABLED_FACTOR`, `CONF_DENOM`, `DERIV_D`, `DERIV_ARRAY_WIDTH`, `COLLAR_INFLUENCE`, `CASING_GRADE`, `YIELD_STRENGTH`, `CASING_GEOMETRY`, `PATCH_AZIMUTH`, `PENE_MERGE`, `MERGE_METHOD`, `ENV_CORRECTION`, `RST_CDV`.

The parameter table remains routed by contract family: flow/spinner/geometry/time/station/phase inputs to 001-015; cement endpoints, indices, coverage, masking, impedance, probability, channeling, statistics, classification and waveform inputs to 016-031; casing loss, geometry, strength, grading, correction, calibration, weight/tension and collar inputs to 032-044; and complete provenance/identity/report/array-custody requirements to 045-048. An absent parameter remains a refusal or dependency, never a guessed implementation value.

O-1 through O-10 remain open. E-1 through E-10 remain active or confirmatory exactly as chaptered. All 18 refusal bullets remain binding. Section 7.4 records no Tier-C item. The requirement map, inventory/canonical-form disposition, discrepancy/open-item disposition, gap/escalation disposition, all 111 dossier parameter-row dispositions, and all 26 critique rows in sections 8.1 through 8.6 remain source custody; none is converted into as-built behavior.

## Requirement receipts

### SB-PLG-001

- Specified contract: Ship three independently gated domain units. Immutable ownership: SB-PLG-T01, SB-PLG-T02.
- Current implementation: No `cement_eval`, `casing_integrity`, or `prodlog` manifest, dispatch branch, command, or UI capability gate exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `REQUESTED-CAPABILITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: All three domain units and their independently disableable gates are missing; a generic module registry is not a substitute.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: All three domain units and their independently disableable gates are missing; a generic module registry is not a substitute.
- Next action: Implement “Ship three independently gated domain units” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-002

- Specified contract: Type every production unit at ingest. Immutable ownership: SB-PLG-T03, SB-PLG-T04, SB-PLG-T05.
- Current implementation: Generic import preserves unknown declared units verbatim and reports them as unconverted, but the typed registry has no production velocity, spinner slope, rate, tension, casing-weight, or acoustic-impedance quantities or transforms.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. `ingest::tests::an_unknown_declared_unit_is_stored_verbatim_and_flagged_unconverted` passed once and proves only generic verbatim retention plus an unconverted warning.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Production quantity families and their reviewed transforms/refusals must exist before any PLG computation can accept these values.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Production quantity families and their reviewed transforms/refusals must exist before any PLG computation can accept these values.
- Next action: Complete every missing domain-specific limb of “Type every production unit at ingest” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-003

- Specified contract: Calibrate spinner slopes from zonal averages. Immutable ownership: SB-PLG-T06, SB-PLG-T07, SB-PLG-T08.
- Current implementation: No spinner zone averaging, slope fitting, branch inheritance, interpolation, or calibration record exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Use only the chapter's cited zonal-average and slope contracts; no calibration constant or branch rule may be inferred.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Use only the chapter's cited zonal-average and slope contracts; no calibration constant or branch rule may be inferred.
- Next action: Implement “Calibrate spinner slopes from zonal averages” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-004

- Specified contract: Compute apparent fluid velocity exactly. Immutable ownership: SB-PLG-T09, SB-PLG-T10.
- Current implementation: No apparent-fluid-velocity computation, sign guard, output identity, or provenance route exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The cited Vapp equation and sign refusal are required together before this output is exposed.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The cited Vapp equation and sign refusal are required together before this output is exposed.
- Next action: Implement “Compute apparent fluid velocity exactly” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-005

- Specified contract: Normalize multi-pass weights. Immutable ownership: SB-PLG-T11.
- Current implementation: No production pass-weight input, validation, normalization, or combined Vapp output exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pass weights remain explicit and default-absent; never assume equal weighting.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pass weights remain explicit and default-absent; never assume equal weighting.
- Next action: Implement “Normalize multi-pass weights” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-006

- Specified contract: Stop before unsupported phase rates. Immutable ownership: SB-PLG-T12.
- Current implementation: Neither the required Vapp output nor named refusals for Vmix, slippage, gas regime, and phase rates exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Absence avoids fabrication but does not satisfy the compound safe behavior; downstream methods remain source-blocked by O/E/refusal custody.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Absence avoids fabrication but does not satisfy the compound safe behavior; downstream methods remain source-blocked by O/E/refusal custody.
- Next action: Implement “Stop before unsupported phase rates” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-007

- Specified contract: Store sensor geometry per family and tool. Immutable ownership: SB-PLG-T13, SB-PLG-T14.
- Current implementation: Generic array intake and storage retain sample order and may store one common axis, but there is no per-tool/per-family sensor identity, angle, radius, handedness, or geometry transform.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. The wide-table axis parser and generic array sample-order round trip each passed once; neither proves a sensor geometry schema.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Per-tool geometry values remain required and absent; ordinal position must never be promoted into physical sensor identity.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Per-tool geometry values remain required and absent; ordinal position must never be promoted into physical sensor identity.
- Next action: Complete every missing domain-specific limb of “Store sensor geometry per family and tool” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-008

- Specified contract: Use an explicit three-phase holdup schema. Immutable ownership: SB-PLG-T15, SB-PLG-T16.
- Current implementation: No RST three-phase module, 24-output manifest, mandatory-input validator, or output semantics exist.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided; if included, implement exactly the held explicit schema without suffix inference.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; if included, implement exactly the held explicit schema without suffix inference.
- Next action: Decide pilot inclusion for “Use an explicit three-phase holdup schema”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-009

- Specified contract: Keep temperature-flow assumptions visible. Immutable ownership: SB-PLG-T17.
- Current implementation: No temperature-flow calculation or persisted assumption surface exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided; cited presets must remain scoped fields, not universal constants.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; cited presets must remain scoped fields, not universal constants.
- Next action: Decide pilot inclusion for “Keep temperature-flow assumptions visible”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-010

- Specified contract: Enforce selective-inflow data sufficiency. Immutable ownership: SB-PLG-T18.
- Current implementation: No selective-inflow-performance route or flowing/shut-in sufficiency gate exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The cited three-flowing-plus-observed-crossflow condition must be enforced before SIP output exists.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The cited three-flowing-plus-observed-crossflow condition must be enforced before SIP output exists.
- Next action: Implement “Enforce selective-inflow data sufficiency” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-011

- Specified contract: Differentiate cumulative inflow with a declared length. Immutable ownership: SB-PLG-T19.
- Current implementation: No cumulative-inflow differentiation route, window input, or persisted window provenance exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided; the differentiation length must remain declared and sourced.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; the differentiation length must remain declared and sourced.
- Next action: Decide pilot inclusion for “Differentiate cumulative inflow with a declared length”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-012

- Specified contract: Make Chronolog epochs and operation order explicit. Immutable ownership: SB-PLG-T20, SB-PLG-T21, SB-PLG-T22.
- Current implementation: Generic text intake exists, but no Chronolog epoch selector, station-time decoder, discriminator, reducer, or operation-order record exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Epoch identity and discriminate-before-estimate order must be explicit; filename or mnemonic guessing is not allowed.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Epoch identity and discriminate-before-estimate order must be explicit; filename or mnemonic guessing is not allowed.
- Next action: Implement “Make Chronolog epochs and operation order explicit” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-013

- Specified contract: Restrict station import to evidenced grammars. Immutable ownership: SB-PLG-T23.
- Current implementation: There is no production-station ASCII importer or grammar-specific decimal refusal.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Generic delimited-text intake cannot establish the station grammar; only evidenced grammars may register.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Generic delimited-text intake cannot establish the station grammar; only evidenced grammars may register.
- Next action: Implement “Restrict station import to evidenced grammars” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-014

- Specified contract: Normalize nulls before station reduction. Immutable ownership: SB-PLG-T24.
- Current implementation: Generic imports normalize declared nulls to `f32::NAN`, but there is no station suspected-null detector, exclusion audit, or station reduction performed after normalization.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. `parsers::las_depth_tests::null_recognition_is_one_relative_tolerance_transform_and_recognition_never_rewrites` passed once and proves only the generic LAS null transform.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Declared and suspected station nulls require separate provenance and must be excluded before the reducer.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Declared and suspected station nulls require separate provenance and must be excluded before the reducer.
- Next action: Complete every missing domain-specific limb of “Normalize nulls before station reduction” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-015

- Specified contract: Preserve phase semantics. Immutable ownership: SB-PLG-T25.
- Current implementation: No gas/oil/water holdup schema, range validator, terminal semantics, or phase-colour UI exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Phase names, V/V units, colours, and [0,1] guards must remain one typed contract.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Phase names, V/V units, colours, and [0,1] guards must remain one typed contract.
- Next action: Implement “Preserve phase semantics” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-016

- Specified contract: Bind cutoff polarity to measurement family. Immutable ownership: SB-PLG-T26, SB-PLG-T27.
- Current implementation: No cement measurement-family type, endpoint validator, cutoff polarity binding, or classification route exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: No endpoint or cutoff may cross amplitude, attenuation, impedance, or coverage families.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: No endpoint or cutoff may cross amplitude, attenuation, impedance, or coverage families.
- Next action: Implement “Bind cutoff polarity to measurement family” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-017

- Specified contract: Implement logarithmic attenuation bond index. Immutable ownership: SB-PLG-T28, SB-PLG-T29.
- Current implementation: No logarithmic attenuation bond-index implementation or named output exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Implement only the cited logarithmic form and its boundary refusals.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Implement only the cited logarithmic form and its boundary refusals.
- Next action: Implement “Implement logarithmic attenuation bond index” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-018

- Specified contract: Name and require the bond interpolation method. Immutable ownership: SB-PLG-T30.
- Current implementation: No bond interpolation selector, required-method gate, or method provenance exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: `BOND_INTERP` remains absent; attenuation and amplitude interpolation must be explicit and cannot silently substitute.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: `BOND_INTERP` remains absent; attenuation and amplitude interpolation must be explicit and cannot silently substitute.
- Next action: Implement “Name and require the bond interpolation method” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-019

- Specified contract: Derive coverage from valid array width. Immutable ownership: SB-PLG-T31, SB-PLG-T32.
- Current implementation: Generic array sample vectors and NaN slots survive storage, but no cement-coverage computation, per-row finite denominator, width-normalized statistic, or validity record exists.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. The generic array sample-order round trip passed once; it does not compute coverage or persist a valid-count denominator.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Coverage must derive from each row's finite valid width; no fixed 72/360 denominator is legal.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Coverage must derive from each row's finite valid width; no fixed 72/360 denominator is legal.
- Next action: Complete every missing domain-specific limb of “Derive coverage from valid array width” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-020

- Specified contract: Exclude collars without deleting data. Immutable ownership: SB-PLG-T33.
- Current implementation: No collar mask, mask-aware cement statistic, or report/export path exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Collar samples must remain stored and exported; only a reversible mask may exclude them from statistics.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Collar samples must remain stored and exported; only a reversible mask may exclude them from statistics.
- Next action: Implement “Exclude collars without deleting data” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-021

- Specified contract: Compute slurry acoustic impedance in declared units. Immutable ownership: SB-PLG-T34, SB-PLG-T35.
- Current implementation: No slurry acoustic-impedance calculation or supporting density/transit-time production unit bridge exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Use only the chapter's cited unit conversions and preserve the recorded comparator gap.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Use only the chapter's cited unit conversions and preserve the recorded comparator gap.
- Next action: Implement “Compute slurry acoustic impedance in declared units” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-022

- Specified contract: Keep expected-CBL correlation optional and attributed. Immutable ownership: SB-PLG-T36.
- Current implementation: No expected-CBL correlation route or coefficient-provenance warning exists.
- Verdict: `ABSENT`; release `DEFERRED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: This P2 initializer remains deferred and absent until its coefficient provenance is accepted; it must never become an unattributed house method.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: This P2 initializer remains deferred and absent until its coefficient provenance is accepted; it must never become an unattributed house method.
- Next action: Keep “Keep expected-CBL correlation optional and attributed” absent for the first pilot; revisit only when its named source/provenance dependency is closed.

### SB-PLG-023

- Specified contract: Keep probability and confidence separate. Immutable ownership: SB-PLG-T37, SB-PLG-T38.
- Current implementation: No cement probability terms, confidence weights, arithmetic denominator, or separate output identities exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: O-1 remains open; probability and confidence must not be collapsed or silently disabled.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: O-1 remains open; probability and confidence must not be collapsed or silently disabled.
- Next action: Implement “Keep probability and confidence separate” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-024

- Specified contract: Validate probability-term switches. Immutable ownership: SB-PLG-T39.
- Current implementation: No service-term switch model or pre-calculation dependency validator exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The channeling-on/bonding-off invalid combination must refuse before any probability is calculated.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The channeling-on/bonding-off invalid combination must refuse before any probability is calculated.
- Next action: Implement “Validate probability-term switches” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-025

- Specified contract: Explain the single-service ceiling. Immutable ownership: SB-PLG-T40.
- Current implementation: No single-service cement-index UI, reachable-ceiling calculation, or threshold explanation exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DEGRADED-RESULT`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The user-visible ceiling must derive from the selected service terms and remain distinct from the accepted-probability parameter.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The user-visible ceiling must derive from the selected service terms and remain distinct from the accepted-probability parameter.
- Next action: Implement “Explain the single-service ceiling” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-026

- Specified contract: Implement channel detection with an explicit direction warning. Immutable ownership: SB-PLG-T41.
- Current implementation: No channel detector, collar-free window statistic, adopted comparison direction, or direction warning exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: O-3 remains open; the adopted algorithm direction may ship only with the specified visible warning.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: O-3 remains open; the adopted algorithm direction may ship only with the specified visible warning.
- Next action: Implement “Implement channel detection with an explicit direction warning” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-027

- Specified contract: Separate derivative, smoothing and vertical statistics. Immutable ownership: SB-PLG-T42, SB-PLG-T43.
- Current implementation: Generic conditioning does not provide the specified array derivative, x-element smoothing, vertical-depth statistic, or persisted inconsistency warning.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Derivative estimator O-2 and cutoff/display O-4 remain open; the three window semantics must stay distinct.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Derivative estimator O-2 and cutoff/display O-4 remain open; the three window semantics must stay distinct.
- Next action: Implement “Separate derivative, smoothing and vertical statistics” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-028

- Specified contract: Preserve four-direction microdebond evidence. Immutable ownership: SB-PLG-T44.
- Current implementation: No four-direction microdebond statistic, neighborhood definition, or evidence-preserving image output exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided and E-7 remains open; implement only from the cited/independently sourced method.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided and E-7 remains open; implement only from the cited/independently sourced method.
- Next action: Decide pilot inclusion for “Preserve four-direction microdebond evidence”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-029

- Specified contract: Keep cement classifications distinct. Immutable ownership: SB-PLG-T45.
- Current implementation: No distinct bond-score and cement-bond classification identities or definitions exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Equal numeric bands do not authorize one merged semantic identity.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Equal numeric bands do not authorize one merged semantic identity.
- Next action: Implement “Keep cement classifications distinct” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-030

- Specified contract: Enforce isolation-report interval length. Immutable ownership: SB-PLG-T46.
- Current implementation: No cement isolation interval aggregator, minimum-length rule, or machine-readable report result exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The 5 m report preset must apply to contiguous passing depth, not sample count or display span.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The 5 m report preset must apply to contiguous passing depth, not sample count or display span.
- Next action: Implement “Enforce isolation-report interval length” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-031

- Specified contract: Make waveform extraction reproducible. Immutable ownership: SB-PLG-T47.
- Current implementation: Generic arrays can carry waveform-like samples, but no waveform window, peak picker, gain, delay, transit-time output, or method provenance exists.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided; raw carriage is not reproducible waveform extraction.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; raw carriage is not reproducible waveform extraction.
- Next action: Decide pilot inclusion for “Make waveform extraction reproducible”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-032

- Specified contract: Emit four named casing-loss quantities. Immutable ownership: SB-PLG-T48, SB-PLG-T49.
- Current implementation: No casing-loss engine or four canonical penetration, signed loss, area-loss percentage, and absolute-area-loss identities exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Ambiguous `MLOSS` must remain raw and unusable; four dimensionally distinct quantities are required.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Ambiguous `MLOSS` must remain raw and unusable; four dimensionally distinct quantities are required.
- Next action: Implement “Emit four named casing-loss quantities” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-033

- Specified contract: Retain signed apparent loss. Immutable ownership: SB-PLG-T50.
- Current implementation: No signed apparent-loss output, negative-value retention, or scale/buildup/noise flag exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Negative apparent loss must remain signed and flagged; it must never be clamped to zero.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Negative apparent loss must remain signed and flagged; it must never be clamped to zero.
- Next action: Implement “Retain signed apparent loss” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-034

- Specified contract: Make prior-survey merge explicit. Immutable ownership: SB-PLG-T51.
- Current implementation: No prior-survey input, four-mode merge selector, current-absent behavior, or merge provenance exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: `PENE_MERGE` and `MERGE_METHOD` remain absent; no default merge mode is permitted.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: `PENE_MERGE` and `MERGE_METHOD` remain absent; no default merge mode is permitted.
- Next action: Implement “Make prior-survey merge explicit” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-035

- Specified contract: Require an ovality definition. Immutable ownership: SB-PLG-T52, SB-PLG-T53.
- Current implementation: Generic import can retain a raw curve named `OVALITY`, but no typed definition, zero-based formula, ellipticity identity, or threshold-use refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: O-7 remains open for indexed-radius definitions; raw ambiguous data may be viewed but not interpreted.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: O-7 remains open for indexed-radius definitions; raw ambiguous data may be viewed but not interpreted.
- Next action: Implement “Require an ovality definition” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-036

- Specified contract: Compute Barlow only from sourced strength. Immutable ownership: SB-PLG-T54, SB-PLG-T55.
- Current implementation: No Barlow computation, strength source field, safety-factor provenance, or missing-strength refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Yield strength remains absent; no grade strength may be guessed or carried from a neighboring vendor.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Yield strength remains absent; no grade strength may be guessed or carried from a neighboring vendor.
- Next action: Implement “Compute Barlow only from sourced strength” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-037

- Specified contract: Source nominal casing geometry. Immutable ownership: SB-PLG-T56.
- Current implementation: No sourced nominal-casing geometry registry, measured-ID alternative, edition record, or geometry-dependent refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: E-5 remains open; nominal geometry must be measured or come from an exact cited table and edition.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: E-5 remains open; nominal geometry must be measured or come from an exact cited table and edition.
- Next action: Implement “Source nominal casing geometry” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-038

- Specified contract: Bind grades to their measurement quantity. Immutable ownership: SB-PLG-T57.
- Current implementation: No separately named penetration and thickness-loss grade outputs or quantity-bound thresholds exist.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The two grade families must not be merged merely because their percentages are numerically close.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The two grade families must not be merged merely because their percentages are numerically close.
- Next action: Implement “Bind grades to their measurement quantity” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-039

- Specified contract: Keep three despike stages distinct and auditable. Immutable ownership: SB-PLG-T58.
- Current implementation: Generic conditioning is not the three specified array, scalar, and bad-azimuth stages and does not emit their separate masks.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Each stage must be default-off, typed to its object, reversible/auditable, and independently sourced; E-10 remains open.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Each stage must be default-off, typed to its object, reversible/auditable, and independently sourced; E-10 remains open.
- Next action: Implement “Keep three despike stages distinct and auditable” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-040

- Specified contract: Preserve named correction recipes. Immutable ownership: SB-PLG-T59.
- Current implementation: No named production correction recipes, persisted stage order, or automatic-reorder refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Only the two cited recipes may be registered and their order must remain immutable per run.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Only the two cited recipes may be registered and their order must remain immutable per run.
- Next action: Implement “Preserve named correction recipes” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-041

- Specified contract: Distinguish one- and two-depth calibration. Immutable ownership: SB-PLG-T60.
- Current implementation: No one-depth offset calibration or two-depth gain-and-offset calibration route/provenance exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Calibration behavior must depend on the actual number of distinct cited depths and persist those depths.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Calibration behavior must depend on the actual number of distinct cited depths and persist those depths.
- Next action: Implement “Distinguish one- and two-depth calibration” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-042

- Specified contract: Refuse untracked environmental correction. Immutable ownership: SB-PLG-T61.
- Current implementation: No environmental-correction state, already-corrected declaration, or double-correction refusal exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: `ENV_CORRECTION` remains absent; source or already-corrected status is mandatory before applying a correction.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: `ENV_CORRECTION` remains absent; source or already-corrected status is mandatory before applying a correction.
- Next action: Implement “Refuse untracked environmental correction” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-043

- Specified contract: Canonicalize casing weight and tension. Immutable ownership: SB-PLG-T62, SB-PLG-T63.
- Current implementation: Generic import retains `lbf/ft`, `lb/ft`, `lbm/ft`, `KLBF`, and `LBF` as unknown spellings; none is a typed mass-per-length or force conversion in the registry.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Casing weight and tension are different quantities and require independent canonicalization; raw numeric pooling must refuse.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Casing weight and tension are different quantities and require independent canonicalization; raw numeric pooling must refuse.
- Next action: Implement “Canonicalize casing weight and tension” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-044

- Specified contract: Detect collars with correct window semantics. Immutable ownership: SB-PLG-T64.
- Current implementation: No normalized CCL collar picker, cutoff, jump-ahead search window, or no-smoothing invariant exists.
- Verdict: `ABSENT`; release `PILOT-BLOCKER`; risk `SILENT-WRONGNESS`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: The 10 m window is a search-eligibility rule, not a smoothing kernel.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: The 10 m window is a search-eligibility rule, not a smoothing kernel.
- Next action: Implement “Detect collars with correct window semantics” only from the chapter's cited contract and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-045

- Specified contract: Stamp full run provenance. Immutable ownership: SB-PLG-T65.
- Current implementation: Generic log sets persist module, parameter JSON, input JSON, version, and timestamp, but no PLG run can stamp complete method, units, sources, masks, correction order, and warnings.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. `equations::tests::re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run` passed once and proves only generic versioned log-set behavior.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Complete PLG provenance must be written atomically with every valid output; generic fields prove only a subset.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Complete PLG provenance must be written atomically with every valid output; generic fields prove only a subset.
- Next action: Complete every missing domain-specific limb of “Stamp full run provenance” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-046

- Specified contract: Separate computed, imported and interpreted identities. Immutable ownership: SB-PLG-T66.
- Current implementation: Catalogs distinguish generic imported and computed curves and unknown mnemonics remain raw, but there is no typed interpreted identity or computation/report eligibility gate for ambiguous PLG names.
- Verdict: `PARTIAL`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. Generic unknown-unit retention and generic versioned log-set tests passed once; neither creates a typed interpreted PLG identity.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Raw `BPI`, `OVALITY`, `MLOSS`, and unitless velocity stay viewable but unusable until explicitly typed/defined.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Raw `BPI`, `OVALITY`, `MLOSS`, and unitless velocity stay viewable but unusable until explicitly typed/defined.
- Next action: Complete every missing domain-specific limb of “Separate computed, imported and interpreted identities” and add the owned observable acceptance test(s) before pilot clearance.

### SB-PLG-047

- Specified contract: Export machine-readable reports with masks. Immutable ownership: SB-PLG-T67.
- Current implementation: No machine-readable cement/casing interval report or PLG export carries numeric curves, definitions, aggregation, masks, and provenance.
- Verdict: `ABSENT`; release `UNDECIDED`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. No narrower executable candidate qualifies as this contract.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Pilot inclusion is undecided; raster or traffic-light output alone can never be the truth surface.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Pilot inclusion is undecided; raster or traffic-light output alone can never be the truth surface.
- Next action: Decide pilot inclusion for “Export machine-readable reports with masks”; if included, implement only from the chapter's cited contract and add the owned observable acceptance test(s).

### SB-PLG-048

- Specified contract: Preserve array width and per-row validity end to end. Immutable ownership: SB-PLG-T31, SB-PLG-T68.
- Current implementation: Array write stores an optional axis blob and samples retain NaN slots, but `db::ArrayRow` has no axis, `read_array_log` selects only depth/samples, IPC sends no axis or per-row valid count, ragged rows are padded to one maximum width, and no array export closes the contract.
- Verdict: `PRESENT-DIVERGENT`; release `PILOT-BLOCKER`; risk `DATA-INTEGRITY`.
- Automated evidence: `MISSING` — No executable test proves the complete chapter contract. The wide-table axis parser and generic sample-order round trip passed once, but the latter never reads the stored axis or exercises IPC/display/export.
- Manual evidence: production-logging/cement-evaluation/casing-integrity 0/0 and not listed; array-logs 0/16; data-conventions 0/45; generic-curve-store 0/18; conditioning 0/27; workflow 0/23; report 6/53; office-deliverables 0/39; LAS-export 0/2; processing-history 0/7; portfolio-performance 0/50; security-integrity 0/63; verification-stewardship 0/24. Automated evidence closes none of these manual scenarios.
- Source/parameter boundary: Gate 2 must carry axis, original width, NaN position, and valid count through read, IPC, display, and export; the current storage-only success must not be called end-to-end preservation.
- UI/IPC/provenance surface: Registration, input typing, computation, persistence, IPC/display, report/export, and run provenance were checked as applicable; generic carriage was not promoted into a domain result.
- History/reachability: The accepted implementation anchor is reachable; targeted reachable-history review found no production-specific implementation that closes this row.
- Decision/dependency: Gate 2 must carry axis, original width, NaN position, and valid count through read, IPC, display, and export; the current storage-only success must not be called end-to-end preservation.
- Next action: In Gate 2, carry the original axis and per-row validity semantics through storage read, byte IPC, display and machine-readable export, then add T31/T68 without weakening current sample-order guards.
