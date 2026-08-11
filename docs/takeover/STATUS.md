# SandiBumi takeover status

This is the one-minute program dashboard. Requirement evidence lives in
`docs/takeover/requirements.csv`; manual field evidence remains in `REVIEW.md` and
`docs/VERIFICATION_MATRIX.md`.

## Now

- Product target: paid offline Windows pilot
- Current gate: `G1 — BASELINE RECONCILIATION`
- Baseline foundation: `COMPLETE`
- Active increment: `G1-DOM-POR-P - SB-POR LIVE-ADJUDICATION PLAN COMPLETE; exact 62-row evidence map and all 41 real test intentions routed; execution approval required; no verdicts changed`
- Accepted baseline: `b332026cb498c105f36eade0bf7899bc0c1309f0 — implementation evidence anchor; this docs-only planning increment is recorded by the current topic-branch commit`
- Automated gate: `GREEN — 2026-08-11 on the G1-DOM-POR-P planning tree; 16 takeover-ledger + 13 frontend + 917 Rust passed, 0 failed, 36 ignored; production build and verification matrix green`
- Pilot field evidence: `OPEN`
- Open blockers: `652 live domain adjudications; 207 total pilot-blocker dispositions (10 SB-CORE, 46 SB-DIO, 32 SB-DBM, 29 SB-PLT, 50 SB-ENV and 40 SB-CLY), including satisfied safety contracts that still need field evidence; all 62 SB-POR rows remain unadjudicated and frozen for the planned execution pass at 17 P0, 25 P1, 17 P2 and 3 P3, with all 62 chapter-status and owned-test fields blank; the chapter defines 41 real test IDs while numeric T26/T27 are intentionally absent, and its 74-row parameter table mechanically contains 15 ABSENT-bearing and 8 NON-ADOPTABLE rows while chapter prose claims 18 ABSENT; SP-009, SP-012 through SP-014, the SP-015 citation follow-up, the PHIE-floor evidence conflict, the compaction clamp-versus-refusal choice, the approximation rename-versus-RHG80 choice, source escalations ESC-1/2/3/5/7/POR-8 and all manual-evidence gates remain open; 52 SB-GEO rows remain unadjudicated but are outside current-version execution by DEC-011; 1 branch follow-up; PRD structural findings — 2 roll-up mismatches, 15 blank priorities, 62 blank statuses, 2 invalid statuses, 137 missing owned-test IDs, 1 missing promised artifact, 1 stale RESUME claim; 24 of 29 release claims are not PROVEN`
- Next increment: `after Jauhar reviews and explicitly approves the plan, execute G1-DOM-POR as the next serial docs-only live adjudication; do not start it automatically, change porosity code, select a parameter, resolve an open product decision, or treat automated evidence as manual/UI acceptance`

## Gate dashboard

| Gate | State | Exit evidence |
|---|---|---|
| G1 — Baseline reconciliation | IN PROGRESS | 279/931 rows adjudicated; 652 remain, alongside branch, gate, field-evidence and claims receipts |
| G2 — Silent-wrongness closure | NOT STARTED | no known pilot-reachable silent-wrongness path remains enabled |
| G3 — Windows/offline deployment and recovery | NOT STARTED | clean-machine, offline-runtime, rollback and recovery matrix |
| G4 — Real-data pilot verification | NOT STARTED | Jauhar-confirmed representative workflow evidence |
| G5 — Release freeze and pilot acceptance | NOT STARTED | one frozen candidate accepted through deployment and pilot use |

## Requirement ledger

The generated summary is re-measured by `node tools/takeover-ledger.mjs --summary-json`.
Do not replace it with an estimated percentage.

- Consolidated requirements: `931`.
- Adjudicated: `279`.
- Unadjudicated: `652`.
- As-built states: `51` present-OK, `18` present-unverified, `59` present-divergent, `71` partial,
  `80` absent and `652` unadjudicated.
- Release dispositions: `207` pilot blockers, `706` undecided and `18` deferred. Disposition is not
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
- SB-POR: `0/62` adjudicated - all rows remain `UNADJUDICATED`; `17` P0, `25` P1, `17` P2 and
  `3` P3; all `62` chapter-status and owned-test fields remain blank; the execution plan routes all
  `41` real chapter test intentions without inventing numeric `T26` or `T27`.

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
| G1-DOM-POR-P - SB-POR live-adjudication plan | DONE; AWAITING EXECUTION APPROVAL | exact 62-row map: 17 P0, 25 P1, 17 P2, 3 P3; all 41 real test intentions routed once and numeric T26/T27 left absent; blank source-owned status/test fields, 74-row parameter inventory, ABSENT-count discrepancy, source escalations, sonic/PHIE-floor product decisions and manual-evidence boundaries fenced; no verdict changed | current topic-branch commit |
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
