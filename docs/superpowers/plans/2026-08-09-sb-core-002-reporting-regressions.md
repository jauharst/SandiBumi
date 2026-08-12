# SB-CORE-002 Reporting Regressions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development while
> implementing each contract. This plan is executed inline in the current session; no subagents.

**Goal:** Regression-lock the seven recorded SB-CORE-002 degraded-result paths at their reporting
surfaces and close the one remaining report-batch defect.

**Architecture:** Rust acceptance tests drive the Monte Carlo job result, LAS `ImportResult`, and
emitted PDF plus batch result. Frontend acceptance tests load the real TypeScript modules through
Vite and inspect the text written to DOM reporting surfaces and the real processing-history store.
Only T07 changes behavior; the other six tests pin already-shipped corrections.

**Tech Stack:** Rust, DuckDB in-memory fixtures, Node `node:test`, Vite SSR module loading,
TypeScript, Tauri IPC payload types.

## Global Constraints

- Missing continuous values remain `f32::NAN`, never `Option<f32>`.
- No raw arrays cross IPC as JSON.
- No frontend SQL writes; no `db.rs` changes; no `computed_curves` upsert or `ON CONFLICT` path.
- No `ml.rs` or `montecarlo.rs` edit; tests exercise their existing public reporting surfaces.
- Every new test has the exact sentence assigned in `04_CORE_REQUIREMENTS.md` and cites the
  recorded R4/R18/R19/R21 source of its expected behavior.
- A reporting test reads the user/job/batch artefact, never only an internal `Result` or helper.
- Closed-behavior tests are mutation-proven; T07 is watched failing before production code changes.
- One SB-CORE-002 commit, unpushed; unrelated `Cargo.toml` and untracked directories stay unstaged.

---

### Task 1: Monte Carlo job/result reporting — SB-CORE-T03

**Files:**
- Create: `src-tauri/src/core_reporting_tests.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `montecarlo::run_monte_carlo`, `jobs::{register,list}`, an in-memory DuckDB fixture.
- Produces: test `a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result`.

- [ ] Add a root test module so the acceptance test can call the existing private crate modules
      without editing `montecarlo.rs`.
- [ ] Seed finite inputs for a `gascorr` chain but no gas flag; its documented `FLAGGED` guard must
      fail every realization. Assert `McResult.errors` names the well and underlying guard, and the
      registered job item is `Failed` with the same reason.
- [ ] Run `cargo test core_reporting_tests::a_monte_carlo_chain_failure -- --nocapture`.
- [ ] Run the same fixture with `OPT_GATE=EVERYWHERE` and assert no error plus an `Ok` job item, so
      an implementation that always fails cannot pass.
- [ ] Confirm the historical pre-R4 swallow branch would violate both reporting assertions without
      editing `montecarlo.rs`.

### Task 2: Partial generic import reporting — SB-CORE-T04

**Files:**
- Modify: `src-tauri/src/ingest.rs`

**Interfaces:**
- Consumes: `import_las_files_with` and its returned `ImportResult`.
- Produces: test `a_partial_generic_curve_import_returns_a_named_warning_while_the_standard_curves_remain_successful`.

- [ ] Import a clean LAS control and assert success with no full-curve warning.
- [ ] Remove only the generic sample table in the in-memory fixture, import a second LAS, and assert
      the well and six standard curves committed while `ImportResult.warning` names the failed
      full-curve load and `error` stays absent.
- [ ] Run the named Cargo test.
- [ ] Temporarily remove the `notes.push` reporting branch, verify the test fails for a missing
      warning, restore the branch, and verify green.

### Task 3: Pay-summary and ML reporting surfaces — SB-CORE-T05/T06/T09

**Files:**
- Create: `src/ui/reportingHonesty.ts`
- Modify: `src/ui/summaryDialog.ts`
- Modify narrowly: `src/ui/mlDialog.ts`
- Modify: `tools/frontend-acceptance.test.mjs`

**Interfaces:**
- Consumes: real `renderPaySummaryTable`, `reportMlWriteOutcome`, `renderResults`, and process log.
- Produces: the exact T05, T06 and T09 named frontend tests.

- [ ] Export the existing pay-table renderer under a descriptive name; do not change its rendering.
- [ ] Move the existing ML status/History calculation unchanged into `reportMlWriteOutcome`, which
      writes the real status element, calls the real global status callback, and records through
      the real process log only when at least one well succeeded.
- [ ] Add a minimal test DOM implementation to the acceptance harness.
- [ ] T05: render one `n_classified=0` row and one classified zero row; inspect the actual row HTML
      and note so absent marks and numeric zeros cannot be conflated.
- [ ] T06: report one-of-two success and then zero-of-two; inspect visible status, global status,
      and the real process-history entries.
- [ ] T09: render one zero-contributor advisory and a clean control; inspect the actual rendered
      warning element, not `MlResult.notes` alone.
- [ ] Run the three named Node tests, mutation-prove each historical failure, restore, and rerun.

### Task 4: Report PDF plus batch/run record — SB-CORE-T07

**Files:**
- Modify: `src-tauri/src/report.rs`

**Interfaces:**
- Consumes: `export_report_batch`, the actual emitted PDF bytes, and returned `(written, errors)`.
- Produces: test `a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record`.

- [ ] Seed a renderable well, then make only the Pay Summary dependency fail in the in-memory DB.
- [ ] Add the named test asserting the PDF is still written and contains the Pay Summary heading
      plus unavailable note, while the returned batch record names the well and exactly one
      degradation.
- [ ] Run the named test and verify RED because `errors` is empty.
- [ ] Introduce a private render outcome carrying PDF bytes plus section degradations. Preserve the
      existing public `render_report_pdf` byte-only API; make only batch export consume and report
      degradations after a successful file write.
- [ ] Rerun the named test and the existing report batch tests; verify GREEN.

### Task 5: Field Dashboard status — SB-CORE-T08

**Files:**
- Modify: `src/ui/reportingHonesty.ts`
- Modify: `src/ui/dashboardPanel.ts`
- Modify: `tools/frontend-acceptance.test.mjs`

**Interfaces:**
- Consumes: the actual dashboard status element.
- Produces: test `a_stats_only_dashboard_run_says_no_flag_curves_were_written`.

- [ ] Route the existing completion sentence through a function that writes the status element.
- [ ] Assert the real element says no FLAG curves were written, names Cutoffs & Summary as the
      persisting action, and does not contain the old bare success claim.
- [ ] Mutation-prove by temporarily restoring the R19 false sentence, verify RED, restore, rerun.

### Task 6: Documentation and manual handoff

**Files:**
- Modify: `docs/PRD_v2/04_CORE_REQUIREMENTS.md`
- Modify: `docs/PRD_v2/91_REQUIREMENTS_INDEX.md`
- Modify: `docs/PRD_v2/_SPINE_PENDING.md`
- Modify: `docs/PRD_v2/RESUME.md`
- Modify: `REVIEW.md`

- [ ] Mark T03–T09 `CLOSED — regression test present` only after their named tests pass.
- [ ] Change SB-CORE-002 to `PRESENT-OK — regression-locked` in the chapter and both index rows.
- [ ] Update the closed SP-008 and RESUME record so they no longer claim missing/open locks.
- [ ] Add one top REVIEW entry describing the seven reporting surfaces for desktop click-through.

### Task 7: Verification and commit

- [ ] Run `npx tsc --noEmit`.
- [ ] Run `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from the repository root.
- [ ] Confirm the new total grew by exactly seven passing tests and remains zero failed.
- [ ] Run `git diff --check`, inspect every changed path, and ensure pre-existing unrelated paths
      are unstaged.
- [ ] Commit as `SB-CORE-002 regression-lock degraded-result reporting surfaces`; do not push.
