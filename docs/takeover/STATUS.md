# SandiBumi takeover status

This is the one-minute program dashboard. Requirement evidence lives in
`docs/takeover/requirements.csv`; manual field evidence remains in `REVIEW.md` and
`docs/VERIFICATION_MATRIX.md`.

## Now

- Product target: paid offline Windows pilot
- Current gate: `G1 — BASELINE RECONCILIATION`
- Baseline foundation: `COMPLETE`
- Active increment: `G1-DOM-NMR - DONE; 38/38 requirements adjudicated, all 57 chapter test intentions routed exactly once, and all 42 parameters, 11 open items, 18 active escalation identifiers, 15 refusals and section 8 traceability blocks retained; structural checks and the full execution gate are green; local commit pending`
- Accepted baseline: `b332026cb498c105f36eade0bf7899bc0c1309f0 — implementation evidence anchor; SB-TBD execution is committed at 4082926, the SB-NMR plan is committed at 7515309, and codex/g1-sb-nmr-adjudication is the current sole worktree`
- Automated gate: `GREEN — 2026-08-12 execution refresh; 16 takeover-ledger + 13 frontend + 917 Rust passed = 946 passed, 0 failed, 36 ignored; production build and verification matrix green; the ledger reports 645 adjudicated / 286 unadjudicated / 432 pilot blockers and preserves frozen source-owned hash a6bdd774779db3db04e0f32706a2023caf97bc22cf03126e7f6f037350021670`
- Pilot field evidence: `OPEN`
- Open blockers: `286 live domain adjudications and 432 total pilot-blocker dispositions; all 38 NMR rows are deferred after first sale, but the receipt preserves 16 absent parameters, six source-index test-ownership gaps, axis-loss and value-heatmap divergence, missing NMR interpretation/provenance/QC capability, 11 open items, 18 active escalations, 15 refusals, the distribution-consumer-only inversion boundary and an unmeasured device/performance gate; 52 SB-GEO rows remain deferred to the next product version; 1 branch follow-up and the recorded PRD structural/release-claim findings remain open`
- Next increment: `finish the SB-NMR execution gate and local commit, then recompute the remaining petrophysics inventory and create the next serial domain planning branch in D:\XX. SandiBumi; do not begin Gate 2 remediation while Gate 1 evidence reconciliation remains open.`

## Gate dashboard

| Gate | State | Exit evidence |
|---|---|---|
| G1 — Baseline reconciliation | IN PROGRESS | 645/931 rows adjudicated; 286 remain, alongside branch, gate, field-evidence and claims receipts |
| G2 — Silent-wrongness closure | NOT STARTED | no known pilot-reachable silent-wrongness path remains enabled |
| G3 — Windows/offline deployment and recovery | NOT STARTED | clean-machine, offline-runtime, rollback and recovery matrix |
| G4 — Real-data pilot verification | NOT STARTED | Jauhar-confirmed representative workflow evidence |
| G5 — Release freeze and pilot acceptance | NOT STARTED | one frozen candidate accepted through deployment and pilot use |

## Requirement ledger

The generated summary is re-measured by `node tools/takeover-ledger.mjs --summary-json`.
Do not replace it with an estimated percentage.

- Consolidated requirements: `931`.
- Adjudicated: `645`.
- Unadjudicated: `286`.
- As-built states: `72` present-OK, `22` present-unverified, `131` present-divergent, `137` partial,
  `283` absent and `286` unadjudicated.
- Release dispositions: `432` pilot blockers, `378` undecided and `121` deferred. Disposition is not
  defect state: a satisfied safety contract can remain a pilot blocker until field evidence closes.
- SB-DIO: `63/63` adjudicated - `42` present-OK, `6` present-unverified, `6` present-divergent,
  `2` partial and `7` absent; `46` pilot blockers, `14` undecided and `3` deferred; `42`
  correctness-tested and `21` missing qualifying owned proof.
- SB-DBM: `43/43` adjudicated - `1` present-OK, `3` present-unverified, `11` present-divergent,
  `13` partial and `15` absent; `32` pilot blockers, `8` undecided and `3` deferred; `1`
  optional-package proof and `42` missing qualifying whole-contract proofs.
- SB-PLT: `35/35` adjudicated - `4` present-unverified, `10` present-divergent, `14` partial and
  `7` absent; `29` pilot blockers, `4` undecided and `2` deferred; `6` characterization tests and
  `29` missing qualifying whole-contract proofs.
- SB-ENV: `58/58` adjudicated - `5` present-OK, `4` present-unverified, `15` present-divergent,
  `15` partial and `19` absent; `50` pilot blockers, `7` undecided and `1` deferred; `5`
  correctness tests, `4` characterizations and `49` missing qualifying whole-contract proofs.
- SB-CLY: `55/55` adjudicated - `13` present-divergent, `15` partial and `27` absent; `40` pilot
  blockers, `8` undecided and `7` deferred; `3` characterizations and `52` missing qualifying
  whole-contract proofs.
- SB-POR: `62/62` adjudicated - `21` present-divergent, `15` partial, `25` absent and `1`
  present-unverified; `44` pilot blockers, `13` undecided and `5` deferred; `6` characterizations
  and `56` missing qualifying whole-contract proofs; all `62` chapter-status and owned-test fields
  remain blank, all `41` real test intentions are routed, and numeric `T26`/`T27` remain absent.
- SB-SAT: `51/51` adjudicated - `6` present-OK, `1` present-unverified, `16` present-divergent,
  `9` partial and `19` absent; `41` pilot blockers, `8` undecided and `2` deferred; `14`
  characterizations, `37` missing qualifying proofs and `0` correctness proofs; all `63` test
  intentions are routed, all `71` parameter rows remain source-fenced, and manual saturation
  evidence remains `2/97`.
- SB-MIN: `46/46` adjudicated - `5` present-OK, `1` present-unverified, `7` present-divergent,
  `6` partial and `27` absent; `34` pilot blockers, `8` undecided and `4` deferred; `1` correctness
  proof, `16` characterizations and `29` missing qualifying whole-contract proofs; all `44` test
  intentions and `78` parameter rows are routed, while manual SandiMin evidence remains `0/28`.
- SB-CUT: `61/61` adjudicated - `10` present-OK, `1` present-unverified, `8` present-divergent,
  `14` partial and `28` absent; `42` pilot blockers, `9` undecided and `10` deferred; `4`
  correctness proofs, `15` characterizations and `42` missing qualifying whole-contract proofs;
  all `44` test intentions and `44` parameter rows are routed, while manual cutoffs/pay evidence
  remains `0/23` and Monte Carlo remains `2/14`.
- SB-SHR: `42/42` adjudicated - `10` present-divergent, `14` partial and `18` absent; `34` pilot
  blockers and `8` deferred; `17` characterizations and `25` missing qualifying whole-contract
  proofs; all `44` test intentions and `61` parameter rows are routed, five escalations remain open,
  and manual saturation-height and rock-typing evidence remains `0/6` and `0/26` respectively.
- SB-TBD: `66/66` adjudicated - `8` present-divergent, `7` partial and `51` absent; `30` pilot
  blockers and `36` deferred; `11` characterizations and `55` missing qualifying whole-contract
  proofs; all `66` test intentions, `52` parameter rows, `11` open items, `6` escalations, `15`
  refusals, `4` derivation/legal items and `327` traceability dispositions are routed, while manual
  thin-bed evidence remains `0/1` and Thomas-Stieber remains `0/0` and not listed.
- SB-NMR: `38/38` adjudicated - `2` present-divergent, `1` partial and `35` absent; all `38` are
  deferred after first sale; `3` characterizations and `35` missing qualifying whole-contract
  proofs; all `57` test intentions, `42` parameter rows, `11` open items, `18` active escalation
  identifiers and `15` refusals are routed, while manual array-log evidence remains `0/16`.

## PRD structural integrity

- Consolidated requirements: `931`
- Roll-up mismatches: `2`
- Blank priorities: `15`
- Blank statuses: `62`
- Invalid statuses: `2`
- Requirements without an owned test ID: `137`
- Missing promised artifacts: `1`
- Stale RESUME claims: `1`
- Chapter references not resolving exactly once: `0`
- Domain chapter files not represented by consolidated rows: `0`

## Manual and field verification

- Checked scenarios: `78 / 1,479` (`5.3%`).
- Unchecked scenarios: `1,401`.
- Capabilities with recorded exercise: `14 / 54`.
- Fully exercised capabilities: `1 / 54`.
- Capability states: `1` exercised, `13` partial, `38` not exercised, `1` not recorded,
  `1` not listed.
- Evidence report: `docs/takeover/evidence/field-verification.md`.
- Boundary: automated and desktop-harness evidence does not close an unchecked manual scenario.

## Release claims

- Registered claims: `29`.
- States: `5` proven, `6` qualified, `3` unmeasured, `11` remove-recommended, `3` legal-review,
  `1` undecided.
- Register: `docs/takeover/CLAIMS.md`.
- Boundary: a proven narrow behavior is not automatically field-verified or releasable; all five
  gates still apply.

## Recent increments

| Increment | State | Evidence | Commit |
|---|---|---|---|
| G1-DOM-NMR - SB-NMR live adjudication | DONE; REVIEW REQUIRED | 38/38 rows: 35 absent, 1 partial and 2 divergent; all 38 deferred after first sale; 3 characterizations and 35 missing qualifying whole-contract proofs; all 57 test intentions, 42 parameters, 11 opens, 18 active escalation identifiers and 15 refusals routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch worktree |
| G1-DOM-NMR-P - SB-NMR live-adjudication plan | DONE; EXECUTED | exact 38-row map: 26 P1, 10 P2, 2 P3 and no P0; all 57 chapter tests routed, including six source-index owned-test gaps; 42 parameter rows with 16 ABSENT entries fenced; 11 opens, 18 active escalation identifiers, 15 refusals, zero Tier-C items and all section 8 blocks preserved; axis-loss, geometry, amplitude-heatmap, interpretation, unit, provenance, inversion and manual-evidence boundaries explicit; focused candidates 10 passed / 0 failed / 0 ignored; full planning gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `7515309` |
| G1-DOM-TBD - SB-TBD live adjudication | DONE; REVIEW REQUIRED | 66/66 rows: 51 absent, 7 partial, 8 divergent; 30 blockers and 36 deferred; 11 characterizations and 55 missing qualifying whole-contract proofs; all 66 tests, 52 parameters, 11 opens, 6 escalations, 15 refusals, 4 derivation/legal items and 327 traceability dispositions routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | `4082926` |
| G1-DOM-TBD-P - SB-TBD live-adjudication plan | DONE; EXECUTED | exact 66-row map: 4 P0, 45 P1, 14 P2, 3 P3; all 66 named test intentions routed; 52 parameter rows with 9 ABSENT, 2 WITHDRAWN, 1 SEAM and 0 NON-ADOPTABLE entries fenced; 11 opens, 6 escalations, 15 refusals, 4 derivation/legal items and 327 traceability dispositions preserved; live picker-provenance partial closure, formula/range divergence, absent tensor/dip/sand-reference capability, naming and patent boundaries explicit; focused candidates 30 passed / 0 failed / 0 ignored; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `9c868d8` |
| G1-DOM-SHR - SB-SHR live adjudication | DONE; REVIEW REQUIRED | 42/42 rows: 18 absent, 14 partial, 10 divergent; 34 blockers and 8 deferred; 17 characterizations and 25 missing qualifying whole-contract proofs; all 44 tests and 61 parameters routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-SHR-P - SB-SHR live-adjudication plan | DONE; EXECUTED | exact 42-row map: 13 P0, 23 P1, 6 P2; all 44 named test intentions routed; 61 parameter rows with 9 ABSENT, 6 NON-ADOPTABLE, 14 UNSOURCED and 1 PRESENT-UNVERIFIED exposure fenced; unit-family, fitted-object, FWL uncertainty, convention, SCAL correction, flow-unit, report, model-selection and manual-evidence boundaries explicit; direct candidate suite 58 passed / 0 failed / 0 ignored; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `4d4a793` |
| G1-DOM-CUT - SB-CUT live adjudication | DONE; REVIEW REQUIRED | 61/61 rows: 28 absent, 14 partial, 8 divergent, 10 present-OK, 1 present-unverified; 42 blockers, 9 undecided, 10 deferred; 4 correctness proofs, 15 characterizations and 42 missing whole-contract proofs; full gate 946 passed / 0 failed / 36 ignored; 499/931 adjudicated and 432 remain; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch worktree |
| G1-DOM-CUT-P - SB-CUT live-adjudication plan | DONE; EXECUTED | exact 61-row map: 9 P0, 23 P1, 22 P2, 7 P3; all 44 named test intentions routed; 44 parameter rows with 8 ABSENT and 11 NON-ADOPTABLE entries fenced; 12 opens, 6 escalations and 13 refusals preserved; live no-default, SD_MULT, discretisation, result-custody, reporting, IPC and manual-evidence boundaries explicit; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `25ee7af` |
| G1-DOM-MIN - SB-MIN live adjudication | DONE; REVIEW REQUIRED | 46/46 rows: 27 absent, 6 partial, 7 divergent, 5 present-OK, 1 present-unverified; 34 blockers, 8 undecided, 4 deferred; 1 correctness proof, 16 characterizations and 29 missing whole-contract proofs; full gate 946 passed / 0 failed / 36 ignored; 438/931 adjudicated and 493 remain; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-MIN-P - SB-MIN live-adjudication plan | DONE; EXECUTED; COUNT DEFECT RECORDED | exact 46-row/44-test/78-parameter plan, but it stated ten acquisition gaps while execution reverified eleven in the immutable chapter; the execution receipt preserves all eleven and does not rewrite the historical plan or PRD | `f409928` |
| G1-DOM-SAT - SB-SAT live adjudication | DONE; REVIEW REQUIRED | 51/51 rows: 19 absent, 9 partial, 16 divergent, 6 present-OK, 1 present-unverified; 41 blockers, 8 undecided, 2 deferred; 52 exact candidate tests passed, but only 14 characterizations and 37 missing qualifying whole-contract proofs; 539/931 remain; no production, PRD, parameter or manual-evidence change | current topic-branch commit |
| G1-DOM-SAT-P - SB-SAT live-adjudication plan | DONE; REVIEW REQUIRED; NOT EXECUTED | exact 51-row map: 13 P0, 18 P1, 12 P2, 6 P3, 2 P4; all 63 named test intentions routed; 71 parameter rows with 20 ABSENT-bearing and 8 tierless rows fenced; two-engine parity, source, unit, calibration, export, legal and manual-evidence boundaries explicit; section 7.1's ten escalations versus section 8.16's nine recorded without PRD edit; no verdict or production behavior changed | current topic-branch commit |
| G1-DOM-POR-D3/RP - RHG80 direction and serial remediation plan | DONE; REVIEW REQUIRED; PRODUCTION NOT STARTED | DEC-017 closes SP-013's product choice on the original three-segment RHG80 route; original-scan verification and the separate SB-POR-020 vendor-rendering disposition remain open; the Gate-2 plan serializes 44 POR pilot blockers behind source, TDD, UI/provenance and full-gate boundaries; full gate 946 passed / 0 failed / 36 ignored; no production, PRD, ledger verdict, parameter value or manual evidence changed | current topic-branch commit |
| G1-DOM-POR-D2 - POR common-contract boundary | DONE; REVIEW REQUIRED | DEC-015 closes on option 2: common typed custody/observability envelope with method-specific source-bound correction limits and validity rules; SB-POR-001 remains an implementation blocker; no production, PRD, parameter value, test result or manual evidence changed | current topic-branch commit |
| G1-DOM-POR-D1 - POR product-stand adjudication | DONE; REVIEW REQUIRED | DEC-012 through DEC-016 record Cp refusal, user-configurable/versioned output naming, five separate comparison/conditioning/HC/iterative roles and required POR capability inclusion; SB-POR-051/052 promoted to pilot blockers; DEC-015 was left open and is closed by D2; no production, PRD, parameter value, test result or manual evidence changed | `fb78a90` |
| G1-DOM-POR - SB-POR live adjudication | DONE; REVIEW REQUIRED | 62/62 rows: 21 divergent, 15 partial, 25 absent, 1 present-unverified; 44 blockers, 13 undecided, 5 deferred after DEC-014; 6 characterizations and 56 missing qualifying proofs; 590/931 remain; no production, PRD, protected vendor data, parameter choice or manual evidence changed | `2d77fde` plus current decision addendum |
| G1-DOM-POR-P - SB-POR live-adjudication plan | DONE; EXECUTED | exact 62-row map: 17 P0, 25 P1, 17 P2, 3 P3; all 41 real test intentions routed once and numeric T26/T27 left absent; blank source-owned status/test fields, 74-row parameter inventory, ABSENT-count discrepancy, source escalations, sonic/PHIE-floor product decisions and manual-evidence boundaries fenced; no verdict changed | `381fadf` |
| G1-DOM-CLY - SB-CLY live adjudication | DONE; REVIEW REQUIRED | 55/55 rows: 27 absent, 15 partial, 13 divergent; 40 blockers, 8 undecided, 7 deferred; 52 missing qualifying proofs and 3 characterizations; 652/931 remain; no production, PRD, protected vendor data or manual evidence changed | current topic-branch commit |
| G1-DOM-CLY-P - SB-CLY live-adjudication plan | DONE; EXECUTED | exact 55-row map: 13 P0, 15 P1, 19 P2, 6 P3, 2 P4; 44 test intentions routed once; 15 ABSENT and 1 NON-ADOPTABLE parameter findings preserved; transform, endpoint, type, provenance, sentinel and protected-source boundaries fenced; no verdict changed | current topic-branch commit |
| G1-DOM-ENV - SB-ENV live adjudication | DONE; REVIEW REQUIRED | 58/58 rows: 19 absent, 15 partial, 15 divergent, 4 present-unverified, 5 present-OK; 50 pilot blockers, 7 undecided, 1 deferred; 49 missing qualifying proofs and 4 characterizations; 707/931 rows remain; no production, PRD or protected chart data changed | current topic-branch commit |
| G1-DOM-ENV-P - SB-ENV live-adjudication plan | DONE; EXECUTED | exact 58-row map: 23 P0, 23 P1, 11 P2, 1 P3; 70 test intentions routed once; 32 ABSENT, 29 SHIPPED-UNCITED and 16 NON-ADOPTABLE parameter findings preserved; protected chart data and open order/source/legal decisions fenced; no verdict changed | current topic-branch commit |
| G1-SCOPE-PETRO - Petrophysics-first scope remap | DONE; REVIEW REQUIRED | explicit product-owner direction defers SB-GEO execution to the next product version; existing v1 scope doctrine identifies open-hole petrophysics as the current product; dependency evidence orders the next planning sequence ENV → CLY → POR → SAT; no requirement verdict changed | current topic-branch commit |
| G1-DOM-GEO-P - SB-GEO live-adjudication plan | DONE; EXECUTION DEFERRED TO NEXT PRODUCT VERSION | exact 52-row map retained as evidence: 33 P0, 17 P1, 2 P2; 73 named test intentions; historical chapter state 50 absent and 2 partial; no row-level verdict changed | `e5e86b8` |
| G1-DOM-PLT - SB-PLT live adjudication | DONE; REVIEW REQUIRED | 35/35 rows: 7 absent, 14 partial, 10 divergent, 4 present-unverified; 29 pilot blockers, 4 undecided, 2 deferred; 29 missing qualifying proofs and 6 characterizations; 765/931 rows remain | current topic-branch commit |
| G1-DOM-PLT-P - SB-PLT live-adjudication plan | DONE; EXECUTED | exact 35-row map: 18 P0, 13 P1, 4 P2; 43 chapter test intentions but 35 blank source-owned test fields; observable-integration, absent-parameter, performance and chart-rights gates preserved | current topic-branch commit |
| G1-DOM-DBM - SB-DBM live adjudication | DONE; REVIEW REQUIRED | 43/43 rows: 15 absent, 13 partial, 11 divergent, 1 present-OK, 3 present-unverified; 32 pilot blockers, 8 undecided, 3 deferred; source/tolerance/UTC/real-scale gaps preserved; 800/931 rows remain | current topic-branch commit |
| G1-DOM-DBM-P - SB-DBM live-adjudication plan | DONE; EXECUTED | exact 43-row evidence map; PK-less write-discipline boundary; provenance, integrity, model-custody and scale evidence gates; serial handoff | `c283f47` |
| G1-DOM-DIO - SB-DIO live adjudication | DONE; REVIEW REQUIRED | 63/63 rows: 7 absent, 2 partial, 6 divergent, 42 present-OK, 6 present-unverified; 46 pilot blockers, 14 undecided, 3 deferred; explicit O-4/O-5, RP66, LAS 3 and STEP-tolerance blocks; 843/931 rows remain | current topic-branch commit |
| G1-DOM-DIO-P — SB-DIO live-adjudication plan | DONE; EXECUTED | exact 63-row evidence map; immutable-source boundary; explicit O-4/O-5 parameter blocks; executable checks and serial handoff | current topic-branch commit |
| G1-DOM-CORE — SB-CORE live adjudication | DONE; REVIEW REQUIRED | 25/25 rows: 5 absent, 12 partial, 4 divergent, 3 present-OK, 1 present-unverified; 10 pilot blockers, 13 undecided, 2 deferred; 906/931 rows remain | current topic-branch commit |
| G1-DOM-CORE-P — SB-CORE live-adjudication plan | DONE | exact 25-row evidence map, immutable-source boundary, executable checks and serial handoff | `9539351` |
| G1-I006 — Customer-facing claim inventory | DONE | 29 claims traced: 5 proven, 6 qualified, 3 unmeasured, 11 remove-recommended, 3 legal-review, 1 undecided; Gate 1 remains open | `b332026` |
| G1-I005 — Manual and field-evidence baseline | DONE | 78/1,479 scenarios checked; 14/54 capabilities recorded; 1/54 fully exercised; pilot evidence remains open | `dc88986` |
| G1-I004A — GitHub master baseline anchor | DONE | clean origin/master merge; SB-CORE-T04 atomic-import adjudication; checkout-stable PRD-audit bytes; full gate 946 passed, 0 failed, 36 ignored | `02b59ea` |
| G1-I004 — PRD structural integrity | DONE | generated byte-current audit; 16 tracker tests pin roll-ups, artifacts, chapter counts, status vocabulary and stale-report refusal | `54bc938` |
| G1-I003 — Branch reconciliation | DONE | 62 refs and 66 distinct patches classified: 53 equivalent, 8 accepted candidates, 2 superseded, 2 rejected, 1 unresolved | `20851f0` |
| G1-I002 — Dated baseline receipt | DONE | exact Git, ledger, manual-evidence, capability-matrix and gate measurements | `32115da` |
| G1-I001 — Tracker foundation | DONE | 931-row ledger; 9 named tracker tests; ledger check and full gate green | `706fe59` |

## Decisions needed from Jauhar

See `docs/takeover/DECISIONS.md`. Only rows marked `NEEDS-JAUHAR` require an answer.

## Worktree protection

Active development is rooted only at `D:\XX. SandiBumi`, the sole registered Git worktree. The
empty, locked `D:\XX. SandiBumi-check` folder remains untouched by explicit direction and is not a
Git worktree. The previously authorized auxiliary documentation/index folders have been removed.
