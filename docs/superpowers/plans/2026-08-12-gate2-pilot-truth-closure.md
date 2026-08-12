# Gate 2 Pilot Truth Closure Implementation Plan

> **For agentic workers:** Execute inline and serially in the single `D:\XX. SandiBumi`
> checkout. Do not dispatch petrophysical math, parameter, storage-invariant, or final-judgment work.
> Each behavior change uses `superpowers:test-driven-development`; every completion claim uses
> `superpowers:verification-before-completion`.

**Goal:** Close every approved Gate 2 truth-and-safety obligation without stealing Windows/offline
qualification from Gate 3 or representative field evidence from Gate 4.

**Architecture:** `docs/takeover/gate2-program.json` is the machine-checked routing and progress
boundary. It resolves the approved 242-row pilot manifest into 222 Gate 2 rows and 20 later-gate-only
rows, then orders the 222 rows into ten dependency-aware tranches. The requirement ledger remains
the source for each row's exact contract, implementation paths, owned tests, citations, dependencies,
blocker, and next action; the immutable PRD chapter remains the scientific specification.

**Tech Stack:** Rust/Tauri, TypeScript, DuckDB, Node test runner, PowerShell release gate, Git/GitHub.

## Global constraints

- Baseline is GitHub merge commit `ca4f8c924373adbc3c0362202b7a914a56bd2b48`.
- DEC-018's exact 242-ID scope remains fixed at SHA-256
  `0412de0cc43fabbe0c5e32d4c831d65e90536ee1c348802ab67cb0f3dcd70b6b`.
- Implementation uses GPT-5.6 Sol xhigh. The final Gate 2 audit uses GPT-5.6 Sol max.
- A parameter, endpoint, cutoff, tolerance, conversion, or method constant is cited or remains
  absent. A missing source closes only through a safe refusal or pilot exclusion, never a guess.
- Missing continuous-log samples remain `f32::NAN`, never `Option<f32>`.
- Numerical arrays cross IPC as bytemuck bytes, never JSON arrays.
- The frontend never sends write SQL. Computed curves remain PK-less and receive no upsert path.
- Python stays a subprocess and every runner reads `sys.stdin.buffer`.
- Every text import uses `parsers::read_text_file`.
- Data edits are undoable; module runs are re-runnable and versioned.
- One requirement implementation per commit. Shared infrastructure is assigned to the first
  requirement that needs it and is referenced by later requirements.
- One serial branch and PR per reviewable tranche. Do not self-approve. Do not mark manual or field
  evidence; Jauhar owns those judgments.
- The checked-in vendor research corpus remains untracked and must never be staged.

## Measured boundary and forecast

The routing is not an estimate:

| Outcome class | Rows | Meaning |
|---|---:|---|
| `IMPLEMENT-OR-REFUSE` | 36 | capability is absent; implement the sourced contract or make the pilot path unavailable with an actionable refusal |
| `REMEDIATE` | 124 | shipped behavior is partial or divergent and needs an observable correction |
| `PROVE` | 19 | behavior is unverified or lacks a correctness-level whole-contract proof |
| `RETAIN` | 43 | behavior is present-OK with correctness evidence; inspect and preserve it while adjacent work changes |
| **Gate 2 total** | **222** | all four classes remain accounted for through Gate 2 |
| Gate 3 only | 18 | clean clone, machine-enforced release, installer, runtime, upgrade, uninstall, support, and legal package evidence |
| Gate 4 only | 2 | capability-indexed field execution and declared-hardware plot performance |
| **Approved pilot total** | **242** | exact DEC-018 set |

There are 179 rows requiring implementation, remediation, or new proof. At a sustained three to
five verified requirements per full working day, raw implementation is 36-60 days. Cross-cutting
dependencies, UI review artifacts, source blocks, full gates, and PR review make **10-16 full-time
working weeks** the defensible initial Gate 2 estimate, or **3-5 calendar months** when the work is
not full-time. After every tranche, replace this estimate with measured throughput and named blocked
days; never preserve an optimistic estimate after evidence contradicts it.

---

## Task 0: Program-control foundation

**Files:**

- Create: `docs/takeover/gate2-program.json`
- Create: `tools/gate2-program.mjs`
- Create: `tools/gate2-program.test.mjs`
- Modify: `package.json`
- Modify: `tools/check.ps1`
- Modify: `docs/takeover/STATUS.md`

**Produces:** A gate-enforced 222/20 routing, derived action-mode counts, ten ordered tranches,
explicit model/verification contracts, and progress arrays that cannot contain a non-Gate-2 row.

- [x] Write tests proving duplicate, omitted, misclassified, and later-gate-stolen rows fail.
- [x] Run the tests before implementation and observe the missing-module failure.
- [x] Implement routing validation and the exact program manifest.
- [x] Run the focused test and observe all six controls pass.
- [x] Add the program check to the full repository gate.
- [x] Update the one-minute dashboard without changing any requirement verdict.
- [x] Run TypeScript, Rust check, and the full gate; record exact totals: 963 passed, 0 failed,
  36 ignored in 85 seconds.
- [x] Commit as `G2-PLAN route the approved pilot truth-closure program` (`02c8ac5`).

## Task 1: Warning and ignored-test ownership

**Files:**

- Create: `docs/takeover/evidence/gate2-warning-inventory.json`
- Create: `docs/takeover/evidence/gate2-ignored-test-inventory.json`
- Create: `tools/gate2-hygiene.mjs`
- Create: `tools/gate2-hygiene.test.mjs`
- Modify: `package.json`
- Modify: `tools/check.ps1`
- Modify: `docs/takeover/STATUS.md`

**Produces:** One stable owner and disposition for every live Rust warning and ignored test. The
inventory distinguishes optional-package execution, field-fixture execution, and non-test artifact
generation; no ignored item is counted as passed.

- [x] Capture `cargo check --message-format=json` and prove the accepted baseline contains 56
  `dead_code` warnings, including all 45 current `plotting.rs` warnings.
- [x] Scan executable Rust tests and prove the accepted baseline contains exactly 36 ignored tests:
  26 optional-package tests, nine field/real-delivery tests, and one manual artifact generator.
- [x] Make the verifier fail on an unclassified warning, a broad `allow(dead_code)`, a bare new
  `#[ignore]`, a duplicated test owner, or a field test classified as package-optional.
- [x] Route pilot-reachable disconnected code to its owning Gate 2 requirement. Route qualified
  package execution to Gate 3 and controlled-corpus execution to Gate 4.
- [x] Preserve the known failing status of `test_full_deterministic_chain` and
  `pipeline_field_full_run`; do not call either passed until its owning gate runs it successfully.
- [x] Run focused tests, TypeScript, Rust check, and the full gate; record exact totals: 970
  passed, 0 failed, 36 ignored in 65 seconds.
- [x] Commit as `G2-I001 own every Rust warning and ignored test`.

## Requirement execution loop

Repeat this loop for each non-`RETAIN` requirement in the active tranche. The ledger's
`implementation_paths`, `owned_tests`, `expected_value_source`, `dependencies`, `blocking_decision`,
and `next_action` are mandatory inputs, not suggestions.

1. Read the owning PRD chapter completely for the selected requirement's §4 contract, §5 sources,
   §6 tests, and §8 traceability; read the relevant `docs/record_*.md` before touching its files.
2. Reverify the ledger's current-source statement at the accepted Gate 2 head. If source and ledger
   differ, commit a docs-only adjudication before production work.
3. Name the production change that would make the new regression fail.
4. Write exactly the named observable acceptance test. Pin both sides wherever a lazier
   implementation could pass. Cite the expected value or independently derived arithmetic.
5. Run only that test and observe the expected RED failure. A compile error, typo, or unavailable
   package is not the required RED result.
6. Implement the smallest complete contract. Do not add adjacent methods, defaults, options, or
   refactors.
7. Run the focused test and affected module suite GREEN. Revert the production change once and prove
   the regression returns RED, then restore and rerun GREEN when the failure mode is not otherwise
   self-evident.
8. Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
9. Add one top `REVIEW.md` increment entry, labelling automated evidence separately and leaving
   visual/manual/field states unchecked.
10. Update the requirement's Gate 2 state and evidence without rewriting the immutable PRD.
11. Stage only owned files and commit with the `SB-*` ID in the message.

## Task 2: SB-CORE-001 — declared depth units on every depth-bearing path

**Files:** `units.rs`, `modules.rs`, `workflow.rs`, `montecarlo.rs`, `ingest.rs`, `intake.rs`,
`images.rs`, `shf_fit.rs`, `export.rs`, `office.rs`, their tests, and Gate 2 evidence/receipts.

- [x] Reverify the PARTIAL verdict against the live module catalogue and every reusable project
  depth-unit fallback caller.
- [x] Write the observable workflow test and see the required behavioral RED: `depth_shift` runs
  when the project unit is undeclared.
- [x] Add an exhaustive dependent/independent module registry; an unknown module refuses instead
  of inheriting the independent class.
- [x] Make workflow and Monte Carlo planning use the same declared-unit resolver.
- [x] Remove the reusable metres fallback and add actionable undeclared-unit refusals to import,
  saturation-height fitting, image depth handling, LAS export, workbook, report and deck paths.
- [x] Pin both sides: every registered dependent module refuses while a registered independent
  module still runs, and an undeclared project never becomes metres.
- [x] Add NIST-SP-811 metre/foot equivalence for metre-qualified temperature, depth-shift and
  splice parameters; observe the pre-fix 50.00 °C versus 118.43 °C RED result and the fixed GREEN.
- [x] Update only under-specified fixtures with explicit metre declarations; preserve tests whose
  subject is unit adoption and preserve depth-independent workflow fixtures without declarations.
- [x] Run the affected Rust library suite: 916 passed / 0 failed / 36 ignored.
- [x] Run TypeScript, Rust check and the full exact-tree repository gate: 972 passed / 0 failed /
  36 ignored in 38 seconds.

Required commit: `SB-CORE-001 enforce declared depth-unit contracts`.

## Task 3: SB-CORE-002 — retain seven observable degraded-result proofs

**Files:** existing tests in `core_reporting_tests.rs`, `ingest.rs`, `report.rs`, and
`tools/frontend-acceptance.test.mjs`; Gate 2 evidence/receipts only.

- [x] Reverify the PRESENT-OK / CORRECTNESS row and its explicit SB-CORE-T03 through T09 surface
  assignments against current source and exact executable tests.
- [x] Run the three Rust proofs: failed Monte Carlo job/result, atomic full-curve import rollback,
  and degraded Pay Summary in both the delivered PDF and batch record.
- [x] Run the four rendered frontend proofs: absent versus real-zero pay, partial/all-failed ML
  status and History, stats-only Dashboard refusal, and zero-contributor warning plus clean control.
- [x] Confirm the SB-CORE-001 dependency change did not bypass, weaken, or rewrite any surface.
- [x] Add no duplicate test and change no production behavior for this RETAIN obligation.
- [x] Run TypeScript, Rust check and the full exact-tree repository gate: 972 passed / 0 failed /
  36 ignored.

Required commit: `SB-CORE-002 reverify degraded-result reporting surfaces`.

For a source or decision block, implement no guessed behavior. If the current pilot path can run and
produce a plausible answer, add the sourced actionable refusal or remove it from the pilot
capability surface. Record the requirement under `blocked_requirements` only after that safety state
is proved. A row is not Gate-2-safe merely because its ideal capability remains blocked.

For each `RETAIN` row, rerun the exact correctness proof and inspect its reporting surface after all
dependent edits. Add no ceremonial duplicate test. Mark it complete only when the original proof is
still executable and the pilot path remains unchanged or more restrictive.

## Tranche order

### G2-T01 — Core truth (14 rows)

**Authority:** `04_CORE_REQUIREMENTS.md`, `docs/takeover/evidence/sb-core.md`.

Close degraded-result reporting, source/default custody, deterministic replay, scope, cancellation,
batch behavior, integrity, verification discipline, and Tier-C policy. `SB-CORE-040`, `-041`, and
`-042` remain outside this tranche under the explicit later-gate routes. The narrow raw Tops colour
DOM boundary from `docs/takeover/evidence/branches.md` is ported with a new observable regression;
the stale historical commit is not cherry-picked whole.

### G2-T02 — Unit and parameter registry (6 rows)

**Authority:** `27_ip-install-blockers.md` SB-INS-014 through -019 and
`docs/takeover/evidence/sb-ins.md`.

Connect semantic identifier plus ordinal activation, conflict refusal, canonical typed units,
observed token/encoding custody, explicit missing-unit state, and generated consumers. This tranche
owns the current unused `validate_unit_registry` warning because the release-wide registry check is
the first production consumer.

### G2-T03 — Project store (30 rows)

**Authority:** `22_database-model.md`, `docs/takeover/evidence/sb-dbm.md`.

Close universal provenance, one-writer discipline, atomic versions, model/run identity, restore and
backup semantics, typed curve metadata, stale-job prevention, and truthful partial results. Never add
`ON CONFLICT`, an upsert, or a primary key to `computed_curves`.

### G2-T04 — Data I/O (49 rows)

**Authority:** `21_data-io.md`, `docs/record_data_tools.md`,
`docs/takeover/evidence/sb-dio.md`.

Retain the 42 correctness-proved contracts and close the seven absent, six unverified, six divergent,
and two partial rows selected by the exact manifest. Every reader uses `read_text_file`; native grids
and `(set_name, mnemonic)` survive; Reframe remains explicit; conversion and omission records remain
observable; exports refuse invalid provenance.

### G2-T05 — Plotting and reporting (17 rows)

**Authority:** `23_plotting-interactivity.md`, `docs/takeover/evidence/sb-plt.md`.

Wire the current Rust plotting contract helpers into real screen/save/template/export surfaces or
move genuinely test-only helpers behind a test boundary. Close axis precedence, range policy,
histograms, statistics, channel policy, shared reduction, depth reconciliation, refetch,
invalidation, render provenance, lawful payload boundaries, paper-scale export, async safety,
accessibility, and truncation. `SB-PLT-032` remains Gate 4. Vendor chart coordinates are removed from
the paid pilot unless legal/licensing evidence authorizes them; no substitute payload is invented.

### G2-T06 — QC and conditioning (31 rows)

**Authority:** `20_envcorr-qc.md`, `docs/takeover/evidence/sb-env.md`.

Close typed preconditions, flags/masks, physical window custody, despike, smoothing, gaps, clipping,
recoverability, output identity, provenance, and no-hidden-repair behavior. Environmental/chart
corrections excluded by DEC-018 remain unreachable rather than receiving new defaults.

### G2-T07 — Linear-GR clay (11 rows)

**Authority:** `10_clay-volume.md`, `docs/takeover/evidence/sb-cly.md`.

Expose only the approved linear-GR path. Close endpoint refusal/reason custody, VSH/VCL type
identity, input/output units, flags, missing-data discipline, source custody, and reporting. Stieber,
Larionov, Clavier, curved, SP, neutron, resistivity, and double-indicator methods remain excluded.

### G2-T08 — Density and analytic D-N porosity (26 rows)

**Authority:** `11_porosity.md`, DEC-012 through DEC-018,
`docs/takeover/evidence/sb-por.md`, and
`docs/superpowers/plans/2026-08-12-sb-por-pilot-remediation.md`.

Implement only density and chart-free analytic neutron-density porosity. Preserve user-defined output
names with explicit versioned replacement, method-specific limits, typed inputs, sourced endpoints,
unlimited diagnostic twins, and full method/convention provenance. Sonic, RHG80, hydrocarbon,
excavation, neutron-sonic, chart-derived, SSC/SSPW, Gaymard-Poupon, and coupled Sxo/Sw remain excluded
from this pilot even though their later product decisions remain recorded.

### G2-T09 — Archie and Indonesia saturation (15 rows)

**Authority:** `12_saturation.md`, `docs/takeover/evidence/sb-sat.md`.

Close effective/total Archie and explicitly parameterized Indonesia only. Preserve required inputs,
guard results, unclipped diagnostics, clipped delivered curves, method provenance, SWE/SWT quantity
identity, and no automatic Rw. No deferred saturation model may satisfy an approved row by alias.

### G2-T10 — Deterministic cutoffs and pay (23 rows)

**Authority:** `14_cutoffs-summation-mc.md`, `docs/takeover/evidence/sb-cut.md`.

Close fixed sourced per-zone VSH, PHIE, and SWE/SWT cutoffs, thickness partition, NTG/pay summaries,
typed missingness, run custody, IPC, report/export failure honesty, and deterministic aggregation.
Monte Carlo, sweeps, automatic optimization, arbitrary tiers, expression cutoffs, and probabilistic
perturbations remain unreachable.

## Tranche close and PR sequence

After all rows in one tranche have a safe outcome:

1. Re-read every tranche row and confirm its code, named test, source, refusal/exclusion, and progress
   state independently; passing the full gate alone is insufficient.
2. Run `npx tsc --noEmit` from the repository root.
3. Run `cargo check` from `src-tauri` and record the warning inventory delta.
4. Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` and record exact passed, failed,
   and ignored totals.
5. Generate visual-review artifacts for every changed user-facing surface. Label them Automated and
   Visual pending; they do not close Manual or Field evidence.
6. Push the serial tranche branch and open a ready-for-review PR against the current GitHub base.
   Preserve one-requirement commits. Do not squash and do not self-approve.
7. Merge only after exact-head, green-check, conflict, and review-state verification. Start the next
   tranche from the resulting GitHub merge commit in the same folder.

## Final Gate 2 audit — GPT-5.6 Sol max

The final audit is a new proof, not a summary of tranche reports:

- Verify all 222 Gate 2 IDs have exactly one completed or safely blocked outcome.
- Recompute the approved 242-ID hash and prove the 20 later-gate routes remain unchanged.
- Reinspect every `BLOCKED-SOURCE`, `BLOCKED-DECISION`, and `EXCLUDED-SAFE` row and prove the pilot
  path refuses or is unreachable at UI, command, persistence, export, and report surfaces.
- Resolve every pilot-reachable Rust warning; preserve explicit later-gate ownership for the rest.
- Prove no ignored test is counted as passed and no non-package failure was hidden with `#[ignore]`.
- Re-run every exact correctness proof and verify test-evidence mappings resolve to executable tests.
- Run TypeScript, Rust, the full repository gate, and the Gate 2 exit verifier on the exact candidate.
- Report automated, visual, manual, and field evidence separately. Gate 2 may close with Manual and
  Field open; it may not call either complete.
- Commit the final receipt, publish a reviewable PR, and stop at the Gate 2 boundary. Gate 3 begins
  only from the accepted merge commit.
