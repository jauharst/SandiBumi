# SandiBumi takeover status

This is the one-minute program dashboard. Requirement evidence lives in
`docs/takeover/requirements.csv`; manual field evidence remains in `REVIEW.md` and
`docs/VERIFICATION_MATRIX.md`.

## Now

- Product target: paid offline Windows pilot
- Current gate: `G1 — BASELINE RECONCILIATION`
- Active increment: `G1-I003 — BRANCH RECONCILIATION`
- Accepted baseline: `706fe59d50fa673066a1dbfb87f0948074656f4e`
- Automated gate: `GREEN — 2026-08-10 at 706fe59; 9 tracker + 12 frontend + 910 Rust passed, 0 failed, 36 ignored`
- Pilot field evidence: `OPEN`
- Open blockers: `UNMEASURED — baseline reconciliation not complete`
- Next increment: `G1-I004 — PRD STRUCTURAL INTEGRITY`

## Gate dashboard

| Gate | State | Exit evidence |
|---|---|---|
| G1 — Baseline reconciliation | IN PROGRESS | 931 live adjudications, branch inventory, gate receipt, field-evidence and claims audits |
| G2 — Silent-wrongness closure | NOT STARTED | no known pilot-reachable silent-wrongness path remains enabled |
| G3 — Windows/offline deployment and recovery | NOT STARTED | clean-machine, offline-runtime, rollback and recovery matrix |
| G4 — Real-data pilot verification | NOT STARTED | Jauhar-confirmed representative workflow evidence |
| G5 — Release freeze and pilot acceptance | NOT STARTED | one frozen candidate accepted through deployment and pilot use |

## Requirement ledger

The generated summary is re-measured by `node tools/takeover-ledger.mjs --summary-json`.
Do not replace it with an estimated percentage.

## Recent increments

| Increment | State | Evidence | Commit |
|---|---|---|---|
| G1-I002 — Dated baseline receipt | DONE | exact Git, ledger, manual-evidence, capability-matrix and gate measurements | this commit |
| G1-I001 — Tracker foundation | DONE | 931-row ledger; 9 named tracker tests; ledger check and full gate green | `706fe59` |

## Decisions needed from Jauhar

See `docs/takeover/DECISIONS.md`. Only rows marked `NEEDS-JAUHAR` require an answer.

## Worktree protection

The pre-existing dirty and untracked paths recorded in the dated baseline receipt are not takeover
inputs and remain unstaged unless Jauhar explicitly assigns them.
