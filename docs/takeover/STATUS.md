# SandiBumi takeover status

This is the one-minute program dashboard. Requirement evidence lives in
`docs/takeover/requirements.csv`; manual field evidence remains in `REVIEW.md` and
`docs/VERIFICATION_MATRIX.md`.

## Now

- Product target: paid offline Windows pilot
- Current gate: `G2 — SILENT-WRONGNESS CLOSURE — IN PROGRESS`
- Gate 2 requirement progress: `11 / 222 handled — 7 DONE, 4 BLOCKED, 211 remaining`
- Baseline foundation: `COMPLETE`
- Active increment: `G2-T01 / SB-CORE-015 - BLOCKED; LAS passes its non-default self-reader round trips, but T15 requires a DLIS writer that does not ship and 21_data-io.md §7.2 A-1 explicitly withholds the RP66 write source needed to build it correctly`
- Accepted baseline: `ca4f8c924373adbc3c0362202b7a914a56bd2b48 — GitHub merge of Gate 1 PR #48; its tree is byte-identical to verified Gate 1 head 5080d416dc38b325700d9981c314055a0c0cf356`
- Automated gate: `SB-CORE-015 evidence-only blocked increment is green at 981 passed / 0 failed / 36 ignored in 150s; its focused export module passed 13/0/0, no DLIS writer symbol exists, and Rust retains the owned 56-warning inventory`
- Pilot field evidence: `OPEN`
- Open blockers: `211 Gate 2 rows remain unhandled; SB-CORE-003 is BLOCKED on the complete cited pilot-method inventory; SB-CORE-005 is BLOCKED on exact per-value endpoint custody plus CLAIM-012 counsel disposition; SB-CORE-007 is BLOCKED on T23 semantics for ABSENT-default producers and intentional working-curve replacement/method flags; SB-CORE-015 is BLOCKED on the absent RP66 writer source and DLIS export path required by T15; the approved 222-row scope remains immutable; 20 approved rows are explicitly owned by Gate 3 or Gate 4`
- Next increment: `Start G2-T01 / SB-CORE-035 from the verified SB-CORE-015 evidence boundary; do not turn a missing DLIS writer into an invented format contract.`

## Gate dashboard

| Gate | State | Exit evidence |
|---|---|---|
| G1 — Baseline reconciliation | COMPLETE | Final audit: 7/7 PASS; 931 rows accounted for exactly once; 879 live-adjudicated plus the exact approved 52-row GEO exception; 242 blockers / 689 deferred / 0 undecided; fresh gate 957 passed / 0 failed / 36 ignored on b4ebe09; zero production-path changes |
| G2 — Silent-wrongness closure | IN PROGRESS | exact program: 222 owned here / 20 later-gate-only; current live classes 36 implement-or-refuse / 118 remediate / 19 prove / 49 retain; initial routing was 36 / 124 / 19 / 43; ten serial tranches; final audit remains open |
| G3 — Windows/offline deployment and recovery | NOT STARTED | clean-machine, offline-runtime, rollback and recovery matrix |
| G4 — Real-data pilot verification | NOT STARTED | Jauhar-confirmed representative workflow evidence |
| G5 — Release freeze and pilot acceptance | NOT STARTED | one frozen candidate accepted through deployment and pilot use |

## Requirement ledger

The generated summary is re-measured by `node tools/takeover-ledger.mjs --summary-json`.
Do not replace it with an estimated percentage.

- Consolidated requirements: `931`.
- Adjudicated: `879`.
- Unadjudicated: `52`.
- As-built states: `105` present-OK, `24` present-unverified, `171` present-divergent, `178` partial,
  `401` absent and `52` unadjudicated.
- Release dispositions: `242` pilot blockers, `0` undecided and `689` deferred. Disposition is not
  defect state: a satisfied safety contract can remain a pilot blocker until field evidence closes.
- SB-INS: `26/26` adjudicated - `4` present-OK, `1` present-unverified, `3` present-divergent,
  `12` partial and `6` absent; `22` pilot blockers and `4` deferred; `4` correctness proofs, `3`
  characterizations and `19` missing qualifying whole-contract proofs; all `30` test intentions,
  `18` parameter rows, `6` opens, `4` escalations, `6` refusals and `95` traceability rows are
  routed, while all `16` SB-INS manual scenarios remain unchecked and installation/deployment is
  `0/0` and not listed.
- SB-GEO: `0/52` live-adjudicated; all `52` release dispositions are `DEFERRED` to the next product
  version under DEC-011 and risk-classed `LATER`; all remain as-built `UNADJUDICATED`, test class
  `MISSING-OR-UNCLASSIFIED`, commit state `UNVERIFIED`, with no last-reverified date. The preserved
  plan records `33` P0, `17` P1, `2` P2 and `73` test intentions, but was deliberately not executed.
- SB-CORE: `25/25` adjudicated - `6` present-OK, `1` present-unverified, `4` present-divergent,
  `9` partial and `5` absent; `17` pilot blockers and `8` deferred; `6` correctness proofs and
  `19` missing qualifying whole-contract proofs.
- SB-DIO: `63/63` adjudicated - `42` present-OK, `6` present-unverified, `6` present-divergent,
  `2` partial and `7` absent; `49` pilot blockers and `14` deferred; `42`
  correctness-tested and `21` missing qualifying owned proof.
- SB-DBM: `43/43` adjudicated - `1` present-OK, `3` present-unverified, `11` present-divergent,
  `13` partial and `15` absent; `30` pilot blockers and `13` deferred; `1`
  optional-package proof and `42` missing qualifying whole-contract proofs.
- SB-PLT: `35/35` adjudicated - `4` present-unverified, `10` present-divergent, `14` partial and
  `7` absent; `18` pilot blockers and `17` deferred; `6` characterization tests and
  `29` missing qualifying whole-contract proofs.
- SB-ENV: `58/58` adjudicated - `5` present-OK, `4` present-unverified, `15` present-divergent,
  `15` partial and `19` absent; `31` pilot blockers and `27` deferred; `5`
  correctness tests, `4` characterizations and `49` missing qualifying whole-contract proofs.
- SB-CLY: `55/55` adjudicated - `13` present-divergent, `15` partial and `27` absent; `11` pilot
  blockers and `44` deferred; `3` characterizations and `52` missing qualifying
  whole-contract proofs.
- SB-POR: `62/62` adjudicated - `21` present-divergent, `15` partial, `25` absent and `1`
  present-unverified; `26` pilot blockers and `36` deferred; `6` characterizations
  and `56` missing qualifying whole-contract proofs; all `62` chapter-status and owned-test fields
  remain blank, all `41` real test intentions are routed, and numeric `T26`/`T27` remain absent.
- SB-SAT: `51/51` adjudicated - `6` present-OK, `1` present-unverified, `16` present-divergent,
  `9` partial and `19` absent; `15` pilot blockers and `36` deferred; `14`
  characterizations, `37` missing qualifying proofs and `0` correctness proofs; all `63` test
  intentions are routed, all `71` parameter rows remain source-fenced, and manual saturation
  evidence remains `2/97`.
- SB-MIN: `46/46` adjudicated - `5` present-OK, `1` present-unverified, `7` present-divergent,
  `6` partial and `27` absent; all `46` deferred; `1` correctness
  proof, `14` characterizations and `31` missing qualifying whole-contract proofs; all `44` test
  intentions and `78` parameter rows are routed, while manual SandiMin evidence remains `0/28`.
- SB-CUT: `61/61` adjudicated - `10` present-OK, `1` present-unverified, `8` present-divergent,
  `14` partial and `28` absent; `23` pilot blockers and `38` deferred; `4`
  correctness proofs, `11` characterizations and `46` missing qualifying whole-contract proofs;
  all `44` test intentions and `44` parameter rows are routed, while manual cutoffs/pay evidence
  remains `0/23` and Monte Carlo remains `2/14`.
- SB-SHR: `42/42` adjudicated - `10` present-divergent, `14` partial and `18` absent; all `42`
  deferred; `17` characterizations and `25` missing qualifying whole-contract
  proofs; all `44` test intentions and `61` parameter rows are routed, five escalations remain open,
  and manual saturation-height and rock-typing evidence remains `0/6` and `0/26` respectively.
- SB-TBD: `66/66` adjudicated - `8` present-divergent, `7` partial and `51` absent; all `66`
  deferred; `8` characterizations and `58` missing qualifying whole-contract
  proofs; all `66` test intentions, `52` parameter rows, `11` open items, `6` escalations, `15`
  refusals, `4` derivation/legal items and `327` traceability dispositions are routed, while manual
  thin-bed evidence remains `0/1` and Thomas-Stieber remains `0/0` and not listed.
- SB-NMR: `38/38` adjudicated - `2` present-divergent, `1` partial and `35` absent; all `38` are
  deferred after first sale; `3` characterizations and `35` missing qualifying whole-contract
  proofs; all `57` test intentions, `42` parameter rows, `11` open items, `18` active escalation
  identifiers and `15` refusals are routed, while manual array-log evidence remains `0/16`.
- SB-TOC: `43/43` adjudicated - `1` present-OK, `12` present-divergent, `8` partial and `22` absent;
  all `43` deferred; `1` correctness proof, `15`
  characterizations and `27` missing qualifying whole-contract proofs; all `58` test intentions,
  `76` parameter rows, `9` open items, `7` escalation bullets and `10` refusals are routed, while
  manual unconventional evidence remains `0/4`.
- SB-RPH: `52/52` adjudicated - `1` present-OK, `6` present-divergent, `4` partial and `41` absent;
  all `52` deferred; `1` optional-package proof, `8`
  characterizations and `43` missing qualifying whole-contract proofs; all `77` test intentions,
  `76` parameter rows, `5` open items, `12` escalation bullets, `11` refusals and `2` C-3 items are
  routed, while core-photo calibration remains `0/0` and not listed.
- SB-MLA: `65/65` adjudicated - `24` present-OK, `1` present-unverified, `18` present-divergent,
  `14` partial and `8` absent; all `65` deferred; `14`
  correctness proofs, `8` characterizations, `2` optional-package proofs and `41` missing qualifying
  whole-contract proofs; all `61` test intentions, `105` parameter rows, `6` open items, `12`
  escalations, `13` refusals, `24` no-antecedent requirements and `218` traceability rows are routed,
  while manual machine-learning evidence remains `7/189` and electrofacies remains `2/26`.
- SB-PLG: `48/48` adjudicated - `1` present-divergent, `6` partial and `41` absent; all `48`
  deferred; all `48` whole-contract proof classes are missing; all
  `68` test intentions, `132` parameter rows, `10` open items, `10` escalations, `18` refusals and
  `239` traceability rows are routed, while production logging/cement evaluation/casing integrity
  remains `0/0` and not listed and the stored array axis is discarded by the normal read/IPC path.

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

- Checked scenarios: `78 / 1,485` (`5.3%`).
- Unchecked scenarios: `1,407`.
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
| G2-T01 SB-CORE-013 — cited parameter disagreement at choice and in ancestry | DONE FOR DEC-003 PILOT; REVIEW REQUIRED | exact 15-topic pilot registry; all source rows retain values/absence, context, source and tier without becoming defaults; module, workflow, cutoff/report/dashboard/QC/Monte-Carlo editors use the shared registry; ancestry persists a snapshot plus cited-match/interpreter-own classification; one owned correctness test pins both sides and a real run/pay record; isolated real-Tauri visual inspection proved collapsed and expanded states and caught the repaired hidden-body CSS bug; full gate 981 passed / 0 failed / 36 ignored; no manual or field scenario marked complete | current topic-branch worktree |
| G1-FINAL-AUDIT - seven-criterion exit proof | COMPLETE | exact audit reports 7/7 PASS with no diagnostics; fresh full-gate receipt is tied to b4ebe09 and permits only the later STATUS/receipt/audit evidence; 957 passed / 0 failed / 36 ignored in 254s; no production path differs from the accepted baseline | evidence-only successor commit |
| G1-SCOPE-PILOT-APPROVAL - exact first-pilot boundary | DONE; OWNER APPROVED; FOCUSED CHECKS GREEN | DEC-018 approves the unchanged exact 242-ID manifest at SHA-256 `0412de0cc43fabbe0c5e32d4c831d65e90536ee1c348802ab67cb0f3dcd70b6b`; the ledger is exactly 242 PILOT-BLOCKER / 689 DEFERRED / 0 UNDECIDED with disposition SHA-256 `295e48ecf3b661c75e3aacf677867098287b2c6264d63daa8eab1e27d5b82319`; 28/28 Gate 1 verifier tests and 27/27 ledger tests pass, TypeScript and Rust checks are green, and approval changes no product-inclusion decision, scientific value, evidence verdict or production behavior | `b4ebe09` |
| G1-SCOPE-GEO-APPROVAL - exact SB-GEO exception | DONE; OWNER APPROVED | DEC-019 permits only the exact 52 DEC-011-deferred SB-GEO rows hashed as `e1eed8f713f5449926b7e5c840d06d9c061eef8c7f420a868434dc57c2978ffc` to remain visibly unadjudicated for Gate 1; all remain mandatory next-version work; DEC-018 and the pilot ledger are unchanged | `63f9d9e` |
| G1-SCOPE-CONSISTENCY - pilot-minimum versus later-lineage boundary | DONE; REVIEW REQUIRED | live DEC-009 says lineage beyond the pilot audit need, while the SB-CORE-010 ledger row incorrectly made it co-author the pilot minimum; the requirement itself enumerates that minimum, DEC-003 selects the representative workflow and DEC-009 now governs only additional lineage; ledger and CORE receipt corrected with no verdict, scope, production, PRD, parameter or manual-evidence change; 27/27 ledger tests and 28/28 Gate 1 verifier tests pass, tracker/PRD audit green and live collector has no diagnostics | `538cc9f` |
| G1-PILOT-SCOPE-PROPOSAL - exact first-pilot and GEO owner boundary | DONE; SUPERSEDED BY OWNER APPROVAL | `pilot-scope.json` names exactly 242 unique, known, live-adjudicated requirements across 10 capability groups and defaults the other 689 to explicit deferral; the test-first validator rejects duplicate, unknown, unadjudicated, structurally ambiguous, count-drifted, ledger-mismatched, hash-drifted or approval-mismatched scope; DEC-018 and DEC-019 subsequently approved the exact hashes without changing production, PRD, parameters or manual evidence | `caa9f92` |
| G1-BRANCH-FOLLOWUP - sole unresolved patch disposition | DONE; REVIEW REQUIRED | current source confirms the raw Tops colour still reaches an `innerHTML` style attribute and no maintained observable regression owns the boundary; `0d5389e` is reclassified from UNRESOLVED to ACCEPTED-CANDIDATE for a narrow Gate 2 TDD port; stale docs/CSP claims make whole-commit cherry-pick forbidden; full gate 946 passed / 0 failed / 36 ignored in 60s; no production, test, PRD, parameter or manual evidence changed | current topic-branch worktree |
| G1-SCOPE-GEO-D - SB-GEO next-version deferral | DONE; REVIEW REQUIRED; NOT LIVE-ADJUDICATED | DEC-011 applied to all 52 release dispositions as DEFERRED/LATER with next-version action; every row remains UNADJUDICATED/MISSING-OR-UNCLASSIFIED/UNVERIFIED with blank last-reverified and evidence fields; all 931 rows now have release timing, but the original all-931 as-built Gate 1 criterion remains formally open; full gate 946 passed / 0 failed / 36 ignored in 69s; no source-owned field, production, test, PRD, parameter or manual evidence changed | current topic-branch worktree |
| G1-DOM-INS - SB-INS live adjudication | DONE; REVIEW REQUIRED | 26/26 rows: 6 absent, 12 partial, 4 present-OK, 3 divergent and 1 present-unverified; 22 blockers and 4 undecided; 4 correctness proofs, 3 characterizations and 19 missing qualifying whole-contract proofs; all 30 tests, 18 parameters, 6 opens, 4 escalations, 6 refusals and 95 traceability rows routed; product-owner Windows/MSI/offline-pack direction retained but real artifact/network/matrix/legal/lifecycle evidence remains absent; full gate 946 passed / 0 failed / 36 ignored in 73s; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch worktree |
| G1-DOM-INS-P - SB-INS live-adjudication plan | DONE; EXECUTED; COLUMN CONTRACT CORRECTED | exact 26-row map: 11 P0 and 15 blank-priority rows; all 30 test intentions, 18 parameters with 5 deliberate ABSENT values, 6 opens, 4 escalations, 6 refusals and 95 traceability rows routed; real MSI/pack/network/matrix evidence separated from fixture validators; parameter-pack loader reachability, unit-token drift, incomplete capability inventory, lifecycle/legal and manual-evidence boundaries explicit; full planning gate 946 passed / 0 failed / 36 ignored in 71s; a follow-up commit corrected receipt concepts that were mistakenly named as CSV columns; no verdict or production behavior changed | `57bf948`; correction `d1df1b5` |
| G1-DOM-PLG - SB-PLG live adjudication | DONE; REVIEW REQUIRED | 48/48 rows: 41 absent, 6 partial and 1 divergent; 41 blockers, 6 undecided and 1 deferred; all 48 qualifying whole-contract proofs missing; all 68 tests, 132 parameters, 10 opens, 10 escalations, 18 refusals and 239 traceability rows routed; source-owned hash and non-PLG records preserved; production/cement/casing remains 0/0 and not listed; full gate 946 passed / 0 failed / 36 ignored in 68s; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-PLG-P - SB-PLG live-adjudication plan | DONE; EXECUTED | exact 48-row map: 24 P0, 17 P1 and 7 P2; all 68 unique test intentions routed across 69 ownership references; 132 parameter rows with 32 deliberate ABSENT values fenced; 10 opens, 10 escalations, 18 refusals and 239 traceability rows preserved; no production-specific source/history candidate found; generic array storage is separated from the live axis-loss divergence; five supporting seam tests passed but prove no whole PLG contract; full planning gate 946 passed / 0 failed / 36 ignored in 278s; no verdict or production behavior changed | `279347d` |
| G1-DOM-MLA - SB-MLA live adjudication | DONE; REVIEW REQUIRED | 65/65 rows: 8 absent, 14 partial, 18 divergent, 24 present-OK and 1 present-unverified; 44 blockers, 17 undecided and 4 deferred; final exact-test audit leaves 14 correctness proofs, 8 characterizations, 2 optional-package proofs and 41 missing whole-contract proofs; all 61 tests, 105 parameters, 6 opens, 12 escalations, 13 refusals, 24 no-antecedent requirements and 218 traceability rows routed; source-owned hash preserved; manual ML evidence remains 7/189 and electrofacies 2/26; full gate 946 passed / 0 failed / 36 ignored in 73s; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-MLA-P - SB-MLA live-adjudication plan | DONE; EXECUTED | exact 65-row map: 10 P0, 34 P1, 17 P2 and 4 P3; all 61 unique chapter test intentions routed across 73 ownership references; 105 parameter rows with 15 ABSENT, 5 NON-ADOPTABLE and 3 SB-SHR cross-references fenced; 6 opens, 12 escalations, 13 refusals, 24 no-antecedent requirements and 218 traceability rows preserved; focused default suites 67/0/7, 13/0/0 and 7/0/0 plus all 7 optional ML tests passed separately; full planning gate 946 passed / 0 failed / 36 ignored in 277s; no verdict or production behavior changed | `3bfb5a2` |
| G1-DOM-RPH - SB-RPH live adjudication | DONE; REVIEW REQUIRED | 52/52 rows: 41 absent, 4 partial, 6 divergent and 1 present-OK; 22 blockers, 17 undecided and 13 deferred; 1 optional-package proof, 8 characterizations and 43 missing whole-contract proofs; all 77 tests, 76 parameters, 5 opens, 12 escalation bullets, 11 refusals, 2 C-3 items and section 8.1-8.6 custody routed; core-photo calibration remains 0/0 and not listed; full gate 946 passed / 0 failed / 36 ignored in 264s; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-RPH-P - SB-RPH live-adjudication plan | DONE; EXECUTED | exact 52-row map: 15 P0, 24 P1, 11 P2 and 2 P3; all 77 named test intentions routed; 76 parameter rows with 43 ABSENT and 1 NON-ADOPTABLE family fenced; 5 opens, 12 escalation bullets, 11 refusals, 2 C-3 items and section 8.1-8.6 custody preserved; absent generic RPH modules separated from shipped core-photo behavior; illumination/lane defaults, uncited thresholds, flat-scan proposal, provenance fallback, accepted-unused inputs and manual-evidence boundaries explicit; focused core-photo candidates 24 passed / 0 failed / 8 ignored plus 8 optional-package tests passed separately; full planning gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `23efced` |
| G1-DOM-TOC - SB-TOC live adjudication | DONE; REVIEW REQUIRED | 43/43 rows: 22 absent, 8 partial, 12 divergent and 1 present-OK; 23 blockers, 10 undecided and 10 deferred; 1 correctness proof, 15 characterizations and 27 missing qualifying whole-contract proofs; all 58 tests, 76 parameters, 9 opens, 7 escalation bullets, 10 refusals and section 8.1-8.6 custody routed; manual unconventional evidence remains 0/4; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-TOC-P - SB-TOC live-adjudication plan | DONE; EXECUTED | exact 43-row map: 18 P0, 15 P1, 8 P2 and 2 P3; all 58 chapter tests routed, including two source-index owned-test gaps; 76 parameter rows with 23 ABSENT entries fenced; 9 opens, 7 escalation bullets, 10 refusals, zero Tier-C items and section 8.1-8.6 custody; typed-unit, baseline, maturity, calibration, gas-property, content/GIP naming, UI-parameter, QC, migration, provenance and manual-evidence boundaries explicit; focused candidates 26 passed / 0 failed / 0 ignored; full planning gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `ef970bf` |
| G1-DOM-NMR - SB-NMR live adjudication | DONE; REVIEW REQUIRED | 38/38 rows: 35 absent, 1 partial and 2 divergent; all 38 deferred after first sale; 3 characterizations and 35 missing qualifying whole-contract proofs; all 57 test intentions, 42 parameters, 11 opens, 18 active escalation identifiers and 15 refusals routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch worktree |
| G1-DOM-NMR-P - SB-NMR live-adjudication plan | DONE; EXECUTED | exact 38-row map: 26 P1, 10 P2, 2 P3 and no P0; all 57 chapter tests routed, including six source-index owned-test gaps; 42 parameter rows with 16 ABSENT entries fenced; 11 opens, 18 active escalation identifiers, 15 refusals, zero Tier-C items and all section 8 blocks preserved; axis-loss, geometry, amplitude-heatmap, interpretation, unit, provenance, inversion and manual-evidence boundaries explicit; focused candidates 10 passed / 0 failed / 0 ignored; full planning gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `7515309` |
| G1-DOM-TBD - SB-TBD live adjudication | DONE; REVIEW REQUIRED | 66/66 rows: 51 absent, 7 partial, 8 divergent; 30 blockers and 36 deferred; final exact-test audit leaves 8 characterizations and 58 missing qualifying whole-contract proofs; all 66 tests, 52 parameters, 11 opens, 6 escalations, 15 refusals, 4 derivation/legal items and 327 traceability dispositions routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | `4082926` |
| G1-DOM-TBD-P - SB-TBD live-adjudication plan | DONE; EXECUTED | exact 66-row map: 4 P0, 45 P1, 14 P2, 3 P3; all 66 named test intentions routed; 52 parameter rows with 9 ABSENT, 2 WITHDRAWN, 1 SEAM and 0 NON-ADOPTABLE entries fenced; 11 opens, 6 escalations, 15 refusals, 4 derivation/legal items and 327 traceability dispositions preserved; live picker-provenance partial closure, formula/range divergence, absent tensor/dip/sand-reference capability, naming and patent boundaries explicit; focused candidates 30 passed / 0 failed / 0 ignored; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `9c868d8` |
| G1-DOM-SHR - SB-SHR live adjudication | DONE; REVIEW REQUIRED | 42/42 rows: 18 absent, 14 partial, 10 divergent; 34 blockers and 8 deferred; 17 characterizations and 25 missing qualifying whole-contract proofs; all 44 tests and 61 parameters routed; source-owned hash preserved; full gate 946 passed / 0 failed / 36 ignored; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
| G1-DOM-SHR-P - SB-SHR live-adjudication plan | DONE; EXECUTED | exact 42-row map: 13 P0, 23 P1, 6 P2; all 44 named test intentions routed; 61 parameter rows with 9 ABSENT, 6 NON-ADOPTABLE, 14 UNSOURCED and 1 PRESENT-UNVERIFIED exposure fenced; unit-family, fitted-object, FWL uncertainty, convention, SCAL correction, flow-unit, report, model-selection and manual-evidence boundaries explicit; direct candidate suite 58 passed / 0 failed / 0 ignored; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `4d4a793` |
| G1-DOM-CUT - SB-CUT live adjudication | DONE; REVIEW REQUIRED | 61/61 rows: 28 absent, 14 partial, 8 divergent, 10 present-OK, 1 present-unverified; 42 blockers, 9 undecided, 10 deferred; final exact-test audit leaves 4 correctness proofs, 11 characterizations and 46 missing whole-contract proofs; full gate 946 passed / 0 failed / 36 ignored; 499/931 adjudicated and 432 remain; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch worktree |
| G1-DOM-CUT-P - SB-CUT live-adjudication plan | DONE; EXECUTED | exact 61-row map: 9 P0, 23 P1, 22 P2, 7 P3; all 44 named test intentions routed; 44 parameter rows with 8 ABSENT and 11 NON-ADOPTABLE entries fenced; 12 opens, 6 escalations and 13 refusals preserved; live no-default, SD_MULT, discretisation, result-custody, reporting, IPC and manual-evidence boundaries explicit; full gate 946 passed / 0 failed / 36 ignored; no verdict or production behavior changed | `25ee7af` |
| G1-DOM-MIN - SB-MIN live adjudication | DONE; REVIEW REQUIRED | 46/46 rows: 27 absent, 6 partial, 7 divergent, 5 present-OK, 1 present-unverified; 34 blockers, 8 undecided, 4 deferred; final exact-test audit leaves 1 correctness proof, 14 characterizations and 31 missing whole-contract proofs; full gate 946 passed / 0 failed / 36 ignored; 438/931 adjudicated and 493 remain; no production, test, PRD, parameter or manual-evidence state changed | current topic-branch commit |
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
