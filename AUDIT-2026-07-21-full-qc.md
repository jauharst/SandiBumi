# SandiBumi full QC audit — 2026-07-21

Parallel audit of every backend tool/module (33 deterministic modules + SandiMin + batch/ML/equation
tools + importers + viz/reporting), plus three foundational substrate checks (DB write/versioning
discipline, workflow-chain engine, well-group scoping), run via 24 independent review agents with
every individual finding adversarially re-verified by a separate agent before being counted here.
Methodology mirrors this repo's own AUDIT-2026-07-20.md house style.

**Only CONFIRMED findings are listed below** (the verifier independently re-read the code and could
not refute the claim). Refuted findings are omitted from the main body but tallied in the appendix
for transparency.

## Summary

- **65 confirmed findings** across 24 audit units (3 substrate + 21 tool groups)
- 3 raised findings were independently refuted and are not included below
- This document is meant to be fed to the working (`D:\XX. SandiBumi`) session for serial, one-at-a-time triage and fixing — per the project's established convention, fixes land directly in the working tree, no branches, commit only when asked.

| Unit | Confirmed findings |
|---|---|
| Substrate — DB write / versioning discipline | 1 |
| Substrate — workflow-chain / job engine | 2 |
| Substrate — well-group scoping sweep | 1 |
| SSC / SSPW / sw_rtc / sw_imts (LRLC group) | 3 |
| precalc / SandiMin (multimin2) / gascorr | 3 |
| Dead & stub code (petrophysics.rs, inversion.rs) | 2 |
| VSH (vsh_gr, vsh_dn) | 3 |
| Porosity (phi_den, phi_dn, phi_son, phimax) | 2 |
| Prep corrections (ftemp_grad, badhole, condflag, nphimat, gr_hole_corr, nphi_env_corr, rhob_hole_corr) | 2 |
| Prep statistical (gr_normalize, log_predict) | 4 |
| Classic Sw (sw_arch, sw_indo, sw_sim) | 3 |
| Permeability (perm_wyllie_rose, perm_coates, perm_transform) | 4 |
| Misc analysis (thin_bed_ts, depth_shift, splice, sw_height) | 1 |
| Facies (electrofacies, gmm_facies) | 3 |
| Legacy multimin (multimin.rs) | 2 |
| Pay summary & cutoff sweep | 3 |
| Monte Carlo | 4 |
| ML bridge | 4 |
| Equations engine (Rhai + Python) | 4 |
| Importers A (LAS, Core CSV, Tops CSV) | 4 |
| Importers B (Aux data, Deviation, SCAL, Well locations) | 2 |
| DLIS import | 2 |
| Viz / reporting (Composite, Report, LAS export) | 3 |
| Curve edit / undo | 3 |

---

## Substrate — DB write / versioning discipline

### 1. computed_curves writers DELETE by exact curve_name while every read path resolves case-insensitively — a re-cased curve name leaves a stale shadow row that can silently win over the fresh value

**Where:** src-tauri/src/equations.rs: write_computed_curves_batch (DELETE at line 941), write_computed_curves_versioned (DELETE at line 588), write_computed_curves_versioned_batch (DELETE at line 676), restore_log_set (DELETE at line 845) — all filter `curve_name IN (...)` on the exact string. Every read path normalizes instead: fetch_computed_curves_batch's `upper(curve_name) IN (...)` (line 376-377), fetch_computed_curve_aligned's `upper(curve_name) = upper(?2)` (line 412), fetch_computed_only_aligned (lines 480, 499), fetch_curve_frame_from_set (line 730, 750). The user-facing trigger is EquationDef.output_curve, which is stored verbatim: save_equation never normalizes it (equations.rs lines 184-212), and the frontend field only `.trim()`s it (src/ui/inspectorPanel.ts line 253, `#eq-output` is free text, editable on an existing equation via the picker at line 206-211 which reuses the same equation_id).

**Evidence:** Concrete repro: define equation E with output_curve="phie" and run it on well W → write_equation_output → write_computed_curves_versioned writes rows (W, depth, 'phie', v). Edit the SAME equation's Output curve field to "PHIE" (case-only change, allowed — save_equation upserts by name/equation_id, no case check) and re-run: the DELETE is `curve_name IN ('PHIE')`, which does not match the existing 'phie' rows, so they are never removed; fresh rows (W, depth, 'PHIE', v_new) are appended alongside them. computed_curves now holds two rows per depth for what every reader treats as one curve. Any subsequent case-insensitive read of "PHIE" (e.g. another module/equation using it as an input, via fetch_curve_frame -> fetch_computed_curves_batch) issues `SELECT upper(curve_name), depth, value FROM computed_curves WHERE well_id=? AND upper(curve_name) IN ('PHIE')` with no ORDER BY, and the Rust fold (`by_name.entry(nm).or_default().insert(d.to_bits(), v)`) lets whichever row DuckDB happens to return last for that depth silently overwrite the other in the HashMap — an unspecified, non-guaranteed order, not a deterministic 'latest wins'. This directly contradicts the schema comment on computed_curves (db.rs lines 118-125) that 'no code path ever inserts a duplicate' / uniqueness is guaranteed by the delete-then-append discipline: for a (well_id, depth, curve) key as understood by every reader (case-insensitively), a duplicate can in fact accumulate and persist indefinitely, because no future write ever again uses the old casing to clean it up. The identical exact-vs-normalized asymmetry also exists one level up in create_log_set's version-count query (`WHERE well_id=?1 AND set_name=?2`, exact) versus fetch_curve_frame_from_set's latest-version lookup (`upper(set_name)=upper(?2)`), so a differently-cased 'output cons' name can likewise fragment version numbering — same root cause, not verified to the same level of certainty as the curve_name case above.

**Suggested fix:** Normalize curve_name (and set_name) to one canonical case at the single choke point where they're written: either change the DELETE clauses in write_computed_curves_batch/_versioned/_versioned_batch and restore_log_set to filter on `upper(curve_name) IN (...)` (binding uppercased parameters) so a write always reclaims any prior casing's rows, and/or uppercase EquationDef.output_curve in save_equation before it's persisted so the stored value is already canonical. Doing only the latter fixes new saves but not curves already duplicated in existing project databases, so the read-side-consistent DELETE fix is the more complete remediation.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual source at D:\XX. SandiBumi\src-tauri\src\equations.rs (the live repo; "D:\XX. SandiBumi Pre" is an empty placeholder folder) and D:\XX. SandiBumi\src\ui\inspectorPanel.ts, and cross-checked every line number cited. All of them match exactly:

Write paths (exact-string DELETE, unnormalized):
- write_computed_curves_batch, line 941: `DELETE FROM computed_curves WHERE well_id = ? AND curve_name IN ({placeholders})` — confirmed verbatim.
- write_computed_curves_versioned, line 588: identical DELETE text — confirmed verbatim.
- write_computed_curves_versioned_batch, line 676: identical DELETE text — confirmed verbatim.
- restore_log_set, lines 843-845: `DELETE FROM computed_curves WHERE well_id = (...) AND curve_name IN (SELECT DISTINCT curve_name FROM computed_curves_archive WHERE set_id = ?1)` — confirmed, exact-string subquery, no upper().

Read paths (case-insensitive):
- fetch_computed_curves_batch, lines 376-377: `upper(curve_name) IN (...)` — confirmed.
- fetch_computed_curve_aligned, line 412: `upper(curve_name) = upper(?2)` — confirmed.
- fetch_computed_only_aligned, lines 480 and 499: both `upper(curve_name) = ?2` — confirmed.
- fetch_curve_frame_from_set, lines 730 and 750: `upper(curve_name)` used both ways — confirmed.

Trigger mechanism, verified end-to-end:
- inspectorPanel.ts line 167: `#eq-output` is a free `<input type="text">` (not a locked/derived field).
- inspectorPanel.ts line 253: `output_curve: val("#eq-output").trim()` — only `.trim()`, no case normalization, confirmed.
- Picker at lines 206-211 reuses the existing `equation_id` on selection, so editing output_curve's case is an edit to the same equation record, confirmed.
- save_equation, lines 185-212: `ON CONFLICT (name) DO UPDATE SET ... output_curve = excluded.output_curve ...` — upserts by `name`, writes `output_curve` verbatim, no case check anywhere — confirmed.
- run_equation → write_equation_output (line 1042): `write_computed_curves_versioned(conn, well_id, depth, &[(equation.output_curve.as_str(), values)], &set_id)` — passes the raw, possibly re-cased string straight into the exact-match DELETE path. This is the causal link the finding's repro depends on, and it's exactly as described.
- Note: python_engine.rs line 275 also lowercases `output_curve` (`output_name = equation.output_curve.trim().to_lowercase()`), but that's only the variable name handed to the Python script executor (`exec_script`), not the value written to the DB — the actual DB write still goes through `write_equation_output` using the verbatim-cased `equation.output_curve`. This does not contradict or mitigate the finding.

Supporting details also verified:
- fetch_computed_curves_batch's fold (line 392): `by_name.entry(nm).or_default().insert(d.to_bits(), v)` with no `ORDER BY` in the SQL — so which duplicate row wins per depth is unspecified, not a real "latest wins," confirmed.
- db.rs lines 118-125: the schema comment claiming "no code path ever inserts a duplicate" / uniqueness via "the WRITE DISCIPLINE" — confirmed present, and confirmed contradicted by the asymmetry above since the discipline only cleans up the exact casing being rewritten.
- The same exact-vs-normalized asymmetry one level up: create_log_set (line 557) does `WHERE well_id = ?1 AND set_name = ?2` (exact), while fetch_curve_frame_from_set (line 739) does `upper(set_name) = upper(?2)` — confirmed, matching the finding's secondary (lower-confidence) claim.
- Searched the whole src-tauri tree for any normalization I might have missed (to_uppercase/to_lowercase near curve_name/output_curve, migrations, triggers, unique constraints) — db.rs explicitly documents computed_curves has NO primary key/uniqueness index by design, and migrate_drop_computed_curves_pk only restructures the table (drops the PK) with no case-folding or dedup logic. Nothing anywhere folds curve_name case at write time.

I found no counter-evidence anywhere in the codebase. Every cited line, quoted SQL string, and causal step in the finding matches the real file content exactly, and the described repro (re-case an equation's output_curve, re-run, get a silently duplicated/racing shadow row) is mechanically valid as traced through the actual call chain. I was unable to refute the finding; it stands confirmed.

</details>

---

## Substrate — workflow-chain / job engine

### 1. inversion.rs's start_inversion/get_inversion_status are confirmed hardcoded-stub commands, still live on the Tauri IPC surface with zero frontend wiring

**Area:** Backend (Rust) — dead/demo code exposed over IPC

**Effort:** small to remove/quarantine; large to actually implement the solver

**Where:** src-tauri/src/inversion.rs lines 26-64 (run_stochastic_inversion, dispatch_inversion); src-tauri/src/lib.rs lines 790-805 (command wrappers) and lines 1017-1018 (registered in tauri::generate_handler!)

**Evidence:** run_stochastic_inversion ignores its own iteration loop's purpose entirely: `let model = [0.25f32, 0.15, 0.20, 0.40];` is returned unconditionally regardless of `iterations`, with the comment "// Real solver step (annealing/MCMC update) goes here." left where a solver update would go; the function doesn't even take well/curve data as input (only an iteration count). start_inversion and get_inversion_status ARE wired into tauri::generate_handler! (lib.rs 1017-1018), so they are callable over IPC by anything that invokes them. A full grep of src/ (ipc.ts, ribbon.ts, every src/ui/*.ts) turns up zero references to start_inversion, get_inversion_status, startInversion, or getInversionStatus — nothing in the frontend calls it; it is unreachable dead functionality masquerading as a real command. It also has no cancellation mechanism at all (no cancel_inversion command, no AtomicBool) and never registers with chain.rs's or jobs.rs's any_active guards, so it stands completely outside the shared-cancel-flag design those two registries correctly implement.

**Suggested fix:** Per the project's own docs/qc_audit_prompt_template.md item 4 (which already flags this as 'STUB — flag as dead/demo'): either implement the real stochastic solver, or remove start_inversion/get_inversion_status from tauri::generate_handler! and delete/quarantine inversion.rs so a future audit or a UI author doesn't mistake it for a working feature.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual files at D:\XX. SandiBumi (the SandiBumi/Arshilla repo) and could not refute the finding — every specific factual claim checks out:

1. src-tauri/src/inversion.rs (65 lines total): `run_stochastic_inversion` (lines 29-44) declares `let model = [0.25f32, 0.15, 0.20, 0.40];` (line 30) and returns `model.to_vec()` (line 43) completely independent of the `iterations` loop — the loop body (lines 32-41) only sleeps 1ms per iteration and periodically writes progress, it never touches `model`. Line 33 contains verbatim the comment `// Real solver step (annealing/MCMC update) goes here.` The function signature `fn run_stochastic_inversion(registry: &JobRegistry, job_id: Uuid, iterations: u32)` confirms it takes no well/curve data, only an iteration count.

2. lib.rs: `start_inversion` and `get_inversion_status` wrapper commands exist (at lines 793-806, not exactly 790-805 as claimed, but same functions) and ARE both listed inside `tauri::generate_handler![...]` (at lines 1026-1027, not exactly 1017-1018 — a minor line-number drift, likely from edits made after the finding was written, but the substantive claim "registered in generate_handler!" is correct).

3. Frontend grep: searching all of src/ (including ipc.ts and ui/ribbon.ts) for `start_inversion`, `get_inversion_status`, `startInversion`, `getInversionStatus` finds zero real references — the only hits for the bare word "inversion" are unrelated comments about the separate, legacy `multimin` (multi-mineral inversion) feature, which uses entirely different command names (run_multimin, multimin_library, etc.). So the "zero frontend wiring" claim holds.

4. No cancellation mechanism: inversion.rs has no `cancel_inversion` command and no `AtomicBool`; grepping src-tauri/src for `AtomicBool`/`cancel_inversion` only turns up hits in workflow.rs, jobs.rs, and chain.rs — confirming inversion.rs's JobRegistry (its own isolated `Arc<Mutex<HashMap<Uuid, InversionStatus>>>`) never participates in the shared-cancel design.

5. The `any_active` guard claim: `chain::any_active` (chain.rs:98) and `jobs::any_active` (jobs.rs:321) are the real cancel/lock guards, both invoked together in lib.rs (lines 95, 115); inversion.rs defines its own separate registry type and is never referenced by either, confirming it stands outside that shared design.

6. The docs/qc_audit_prompt_template.md citation is accurate almost verbatim: line 60 ("inversion.rs's start_inversion is a hardcoded stub still exposed over IPC"), line 185 (table row: "Legacy inversion (STUB — flag as dead/demo) | inversion.rs | start_inversion, get_inversion_status"), and line 234 ("inversion.rs (hardcoded stub still on the IPC surface)") — the project's own docs already flag exactly this issue, corroborating the finding independently.

The only discrepancy I found is trivial: the cited line ranges (lib.rs 790-805 and 1017-1018) are each off by a handful of lines from the current file (actual: 793-806 and 1026-1027) — almost certainly due to unrelated lines added/removed elsewhere in lib.rs since the finding was drafted, not a misidentification of the functions themselves. This does not undermine the substance of the claim. I was unable to find any basis to refute the finding; it is accurate and should stand as confirmed.

</details>

### 2. Cancelling a workflow chain never bumps dataVersion, even though a cancelled run routinely leaves newly-computed curves committed to the database

**Area:** Frontend wiring (TypeScript) — dataVersion cross-cutting behavior, workflow-chain/job engine

**Effort:** small (one additional bumpDataVersion call, ideally moved to/duplicated in processingPanel.ts's job-transition handling)

**Where:** src/ui/workflowDialog.ts lines 796-816 (applyStatus) and lines 858-869/893-898 (poll loop / dispose); src/ui/processingPanel.ts (no bumpDataVersion call anywhere in the file)

**Evidence:** Backend behavior: chain::run_chain (chain.rs) only checks the cancel flag BETWEEN steps, and workflow::run_workflow_module_into writes each step's completed wells in one batched transaction (Phase 2) as soon as that step's compute finishes — so a chain cancelled after step 1 (or mid-step, after some wells finished) has real, already-committed computed_curves/log_sets rows for the wells/steps that finished before the flag was noticed; this is by design ("drains in a well or two"), not a bug in itself. But in workflowDialog.ts's applyStatus(), `bumpDataVersion()` (state.ts) is called ONLY in the `state === "completed"` branch (line 805); the `"cancelled"` and `"failed"` branches (lines 807-815) call finishRun()/setStatus() only, never bumpDataVersion(). processingPanel.ts — the shared, dialog-independent Processing panel that both Cancel buttons drive via the same underlying AtomicBool — contains zero calls to bumpDataVersion for any phase. Also, closing the Workflow Builder dialog mid-run auto-cancels the chain (dispose()/interval-close both call cancelWorkflowChain then finishRun(), bypassing applyStatus entirely), which hits the same gap. Net effect: after any Cancel (from either panel, or from closing the dialog), every other open panel (Log view, Crossplot, Histogram, Curve Catalog) keeps showing pre-run data even though new curve versions now exist in the DB for whichever wells/steps completed, with nothing prompting a refresh until an unrelated action elsewhere happens to bump dataVersion.

**Suggested fix:** Call bumpDataVersion() in the "cancelled" branch of applyStatus() too (any real writes should be reflected), or — more robustly — have chain::run_chain's Cancelled/Completed status report whether curves_written > 0 and bump dataVersion whenever that's nonzero regardless of final phase. Since processingPanel.ts is the dialog-independent tracker meant to survive the Workflow Builder closing, it should own this responsibility (bump dataVersion whenever any job it tracks transitions to a terminal phase with progress > 0), not rely solely on workflowDialog.ts's own poll.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual source at D:\XX. SandiBumi (the real repo behind the "SandiBumi" project referenced in memory) and could not refute any part of the finding — every specific claim checks out against the real files.

Frontend (src/ui/workflowDialog.ts):
- applyStatus() spans exactly lines 796-816. bumpDataVersion() is called at line 805, strictly inside the `state === "completed"` branch (800-806). The `cancelled` branch (807-810) calls only finishRun()/setStatus(); the `failed` branch (811-814) does the same. Neither calls bumpDataVersion.
- The poll-interval's dialog-closed-mid-run path (lines ~861-869) calls `cancelWorkflowChain(currentJob)` then `finishRun()` directly, returning before ever calling `getChainStatus`/`applyStatus`.
- `dispose()` (lines 893-898) does the same: cancels then calls `finishRun()` only — never touches applyStatus or bumpDataVersion.
- grep confirms exactly one bumpDataVersion call in the whole file (line 805).

src/ui/processingPanel.ts: grepped and read in full (228 lines) — zero references to bumpDataVersion, and it doesn't even import from state.ts. It only polls listJobs()/renders/handles Cancel via cancelJob(). Confirmed dialog-independent tracker has no data-refresh responsibility at all today.

Backend:
- src-tauri/src/chain.rs `run_chain`: the cancel flag is checked only at the top of the step loop (line 159) and once more after the loop for the last-step edge case (line 216) — i.e., between steps, exactly as claimed.
- src-tauri/src/workflow.rs `run_workflow_module_into`: cancellation is cooperative per-well inside the rayon `par_iter` (line 167, checked before each well starts), but Phase 2 (lines 305-379) collects every well whose `Outcome::Computed` outputs are non-empty and writes them ALL in one batched transaction (`create_log_sets_batch` + `write_computed_curves_versioned_batch`) with no re-check of the cancel flag before that commit. So any wells that finished computing before the flag was noticed get their curves committed regardless of the subsequent Cancelled status — exactly the mechanism the finding describes.
- `ChainStatus::Cancelled` only carries `at_step`, no curves_written count, consistent with the suggested-fix's premise.

I searched the rest of src/ for any other consumer of the job registry or any other place bumpDataVersion might be triggered on job/chain transitions and found none — workflowDialog.ts and processingPanel.ts are the only two consumers, and only the former's "completed" branch bumps data version.

Every line-range citation in the finding lines up with the real file to within a line or two (e.g., poll loop cited as 858-869 vs. actual 859-869), and the substantive technical claims (cooperative per-well cancellation, unconditional Phase-2 batched commit, applyStatus's completed-only bump, processingPanel's total absence of bumpDataVersion, dispose/interval bypassing applyStatus) are all accurate. I could not find any refuting detail — no other refresh path exists, no cancel-recheck exists before the Phase 2 write, and no bumpDataVersion call exists outside the completed branch. The finding stands as an accurate, verified bug.

</details>

---

## Substrate — well-group scoping sweep

### 1. No batch-run dialog re-scopes to a new active well group while it's already open — only the Wells sidebar tree and Map pane react live to a group switch

**Area:** Frontend wiring / well-group scoping

**Effort:** medium (mechanical, same pattern repeated across ~8 files)

**Where:** src/state.ts:67-69 (wellGroupsVersion contract) and src/ui/wellGroups.ts:41,255 (the only two places that bump it)

**Evidence:** First, the literal ask — does each dialog gate the well list it actually sends to the backend through filterByActiveGroup/defaultRunWellIds, rather than through a picker that bypasses it? All 11 pass: moduleDialog.ts:41,238,271 -> wellChecks (277,303); workflowDialog.ts:52,699 -> wellChecks (832,849-852); monteCarloDialog.ts:52,241 -> wellChecks (282,289); mlDialog.ts:142,207 -> train/apply checks (356-357,377-378); multiminDialog.ts:84,591 -> applyWells (657,669); summaryDialog.ts:25,32 -> wellChecks (76,86); cutoffDialog.ts:30,39 -> checkedWellIds (53,517,569); dashboardPanel.ts:227 re-fetches+filters fresh inside the Compute click handler itself (best-in-class, no cached list to go stale); autoCorrDialog.ts:43 -> targets (101); correlationPanel.ts:48,330 (a viewer, not a compute run, but its well set is correctly filtered throughout); reportDialog.ts:53 -> wells.map (392). None fetch an unfiltered list for the picker and a separately-filtered list for the run, or vice versa.

The real defect is a live-refresh gap that undermines this filtering for the entire time a pane stays open. state.ts:67-68 documents the contract explicitly: wellGroupsVersion is 'Bumped whenever the set of groups or their membership changes, so the Wells pane and batch dialogs reload their group list.' Both call sites that actually change the active group or its membership — activateWellGroup (wellGroups.ts:38-45, called from objectTree.ts:137's group dropdown) and the group manager's reload() (wellGroups.ts:253-257, called after setWellGroupMembers/create/rename/delete) — call bumpWellGroupsVersion() ONLY, never bumpDataVersion(). A repo-wide grep for wellGroupsVersion shows exactly two subscribers outside state.ts: mapPanel.ts:440 (repaints its own polygon overlay) and workspace.ts:744 (refreshes the Wells & Tops sidebar tree). None of moduleDialog.ts, workflowDialog.ts, monteCarloDialog.ts, mlDialog.ts, multiminDialog.ts, summaryDialog.ts, cutoffDialog.ts, correlationPanel.ts, or the workspace.ts wellPane wrapper shared by autoCorrDialog.ts/reportDialog.ts (dataVersion sub at workspace.ts:512) ever subscribes to it — 'batch dialogs reload their group list' is unimplemented everywhere the comment claims it happens.

This matters because every one of these panes is a session-persistent singleton: workspace.ts's openSingleton (911-924) is focus-or-open — re-invoking the ribbon action on an already-open panel just calls existing.api.setActive() (915) and returns without rebuilding, so whatever well list was fetched at first-open (or last dataVersion-triggered refresh) lives for the rest of the session. Six of the eleven — workflowDialog.ts:52, monteCarloDialog.ts:52, mlDialog.ts:142, multiminDialog.ts:84, summaryDialog.ts:25, cutoffDialog.ts:30 — fetch `wells` exactly once at construction with no refresh subscription at all (not even dataVersion), so they go stale on the very next well-group action. The other four (moduleDialog.ts:258, correlationPanel.ts:528, and autoCorrDialog.ts/reportDialog.ts via workspace.ts:512) only refresh on unrelated data events (a module run, an import); a pure group switch with no such event never reaches them.

Concrete scenario: Workflow Builder is open with 'Group North' active; wellChecks pre-ticks its wells (workflowDialog.ts:699-709). The user switches the Wells & Tops dropdown to 'Group South' (objectTree.ts:137 -> wellGroups.ts:41 bumps wellGroupsVersion only). Nothing in workflowDialog.ts listens for that. The pane still shows Group North's wells ticked. Clicking 'Run chain' reads wellIds straight from wellChecks (workflowDialog.ts:832) and calls runWorkflowChain — the backend enforces zero well-group scoping (per the audit brief), so it silently computes the whole chain across Group North, the wrong sector of the field, with no error and no visual cue, even though the Wells sidebar, ribbon and Map pane all now show Group South active. Checked REVIEW.md/ROADMAP.md and AUDIT-2026-07-20.md: ROADMAP.md's well-groups entry (line ~178-182) and #123 (line 676-679) only address correlationPanel's dataVersion-triggered staleness (imports/deletes); the audit's own verification note for #123 ('group-membership changes flow through wellGroupsVersion which the panel doesn't subscribe to, so the suggested reload()-only fix wouldn't cover that half') was never resolved for correlationPanel and was never generalized to the other nine dialogs — this is not a documented deferral, it's an open gap.

**Suggested fix:** Either (a) have activateWellGroup (wellGroups.ts:41) and the manager's reload() (wellGroups.ts:255) also call bumpDataVersion(), piggybacking on the refresh plumbing the four dialogs that already listen for it, or (b) the more surgical fix matching state.ts's own stated intent: add an appState.wellGroupsVersion.subscribe(...) to each of moduleDialog.ts, workflowDialog.ts, monteCarloDialog.ts, mlDialog.ts, multiminDialog.ts, summaryDialog.ts, cutoffDialog.ts, correlationPanel.ts, and workspace.ts's wellPane wrapper, re-running listWells().then(filterByActiveGroup) and rebuilding the checklist (preserving any user ticks still valid) the same way moduleDialog.ts:235-256's refreshData already does for dataVersion.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently verified every load-bearing claim against the actual source in D:\XX. SandiBumi (the SandiBumi codebase), and could not refute the finding.

Verified facts:
1. state.ts:67-69 has the exact comment quoted ("Bumped whenever the set of groups or their membership changes, so the Wells pane and batch dialogs reload their group list"), and bumpWellGroupsVersion (line 97-99) is a distinct counter from bumpDataVersion (line 93-95).
2. wellGroups.ts: activateWellGroup (line 38-44) calls bumpWellGroupsVersion() at line 41 only, never bumpDataVersion. The manager's reload() (line 253-257) calls bumpWellGroupsVersion() at line 255 only. Confirmed.
3. Repo-wide grep for "wellGroupsVersion" shows exactly two subscribers outside state.ts: mapPanel.ts:440 and workspace.ts:752 (the Wells & Tops sidebar tree, close to the cited ~744). No other file subscribes to it.
4. Checked all 9 dialogs/panels named as gap-affected:
   - moduleDialog.ts: subscribes to dataVersion (258) only, not wellGroupsVersion.
   - workflowDialog.ts, monteCarloDialog.ts, mlDialog.ts, multiminDialog.ts, summaryDialog.ts, cutoffDialog.ts: each fetches `listWells().then(filterByActiveGroup)` exactly once at construction, with zero subscribe() calls to either dataVersion or wellGroupsVersion anywhere in the file.
   - correlationPanel.ts: subscribes to dataVersion (528) only; no wellGroupsVersion subscription exists in the file.
   - autoCorrDialog.ts / reportDialog.ts: both wired through workspace.ts's wellPane() wrapper with followData=true, which subscribes only to appState.dataVersion (line 518), never wellGroupsVersion.
5. dashboardPanel.ts:227 does re-filter a freshly fetched well list inside its Compute click handler, matching the "best-in-class" characterization.
6. workspace.ts's openSingleton (919-932) confirmed focus-or-open: an already-open panel just calls existing.api.setActive() and returns, never rebuilding — so a stale well list persists for the pane's lifetime.
7. objectTree.ts:136-137 confirmed: the group dropdown's change handler calls activateWellGroup(...).
8. workflowDialog.ts:699/706/832 confirmed: wellChecks map built once from the one-time `wells` fetch, and the Run-chain handler at 832 reads checked state straight from that same map — exactly the staleness path described.
9. AUDIT-2026-07-20.md's #123 section contains the exact verifier note quoted in the finding almost verbatim: "group-membership changes flow through wellGroupsVersion which the panel doesn't subscribe to, so the suggested reload()-only fix wouldn't cover that half" — and correlationPanel.ts today still has no wellGroupsVersion subscription, confirming this was never resolved or generalized.
10. ROADMAP.md's well-groups entry (~178-183) and the #123 polish entry (676-679) only ever mention dataVersion-triggered staleness (imports/deletes) for correlationPanel, never the wellGroupsVersion gap — matching the finding's claim that this is an open, undocumented gap rather than a recorded deferral.

Every specific file/line/mechanism claim in the finding is verifiably true in the current codebase. I found no counter-evidence (e.g., no hidden subscription, no alternate refresh path, no version-history detail contradicting the claim).

</details>

---

## SSC / SSPW / sw_rtc / sw_imts (LRLC group)

### 1. sw_rtc/sw_imts default input wiring points only to SSC's curve names; running them against an SSPW-only well silently produces an all-NaN 'success'

**Area:** Cross-function provenance / Frontend wiring (dims D+F)

**Effort:** medium

**Where:** src-tauri\src\lrlc.rs:59-62 (sw_rtc_spec log_in defaults) and :160-164 (sw_imts_spec log_in defaults); src\ui\moduleDialog.ts:96,102,250 (logChoiceNames); src-tauri\src\workflow.rs:199-202 (missing-mnemonic NaN fallback), :414 (rows_written)

**Evidence:** sw_rtc/sw_imts declare log_in("PHIT", ..., "PHIT_SSC", true), log_in("CAPBW", ..., "CWSH", false), log_in("CBW", ..., "CBW", false) — every default alias names an SSC output curve. SSPW is the other supported porosity workflow in this same file (sspw_spec, ssc.rs) and produces the physically-equivalent quantities under different names: PHIT_SSPW, CBW_SSPW, CAPBW_SSPW. There is no fallback anywhere that tries the SSPW names when the SSC ones are absent (the lrlc.rs top-of-file doc comment even flags this ambiguity: 'CAPBW pairs naturally with SSC's CWSH or SSPW's CAPBW_SSPW', but nothing in code resolves it). moduleDialog.ts's `logChoiceNames = (keep) => (curveNames.includes(keep) ? curveNames : [keep, ...curveNames])` (line 96), used at line 102 (`fillSelect(select, logChoiceNames(arg.default), arg.default)`) and again at line 250 on every dataVersion refresh, means that if 'PHIT_SSC' was never computed (the well only ran SSPW), the dropdown still shows 'PHIT_SSC' as a normal, pre-selected option — `fillSelect` (lines 84-93) adds no class/title/styling to distinguish a phantom (not-yet-computed) entry from a real one. If the user clicks Run with defaults, workflow.rs resolves the missing mnemonic via `columns.get(...).cloned().unwrap_or_else(|| vec![f32::NAN; depth.len()])` (lines 199-202), so `ctx.log("PHIT")` is all-NaN; sw_rtc's per-sample guard `if rt_i.is_nan() || pt.is_nan() || ... { continue; }` and sw_imts's equivalent guard then skip every single sample, leaving every output curve all-NaN. The run nonetheless reports success: `ModuleRunResult { ..., rows_written: depth.len(), ... }` (workflow.rs:414) is not gated on any output being finite, so moduleDialog.ts (line ~321) prints a green '✓ Well: <N> samples → SWT_RTC, SWE_RTC, RT_CORR, CEX_RTC' line — a full, upbeat success message for a run where nothing was actually computed.

**Suggested fix:** Either add an SSPW-aware fallback in sw_rtc_spec/sw_imts_spec (try PHIT_SSPW/CBW_SSPW/CAPBW_SSPW when the SSC-named curve isn't in the catalog) or surface the mismatch to the user: style/flag a log_in default option in moduleDialog.ts when it isn't in the live curve catalog, and have the run summary report the count/fraction of finite samples per output curve rather than just rows_written = depth.len().

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read every file cited in the finding directly (project root: D:\XX. SandiBumi) and could not refute any part of the claim. All cited evidence checks out almost line-for-line against the real source:

1. src-tauri/src/lrlc.rs:58-62 — sw_rtc_spec's log_in defaults are exactly: PHIT→"PHIT_SSC" (required=true), CAPBW→"CWSH" (SSC's capillary-water output name), CBW→"CBW". Lines 159-164 for sw_imts_spec: PHIT→"PHIT_SSC" again, CBW→"CBW". Every default alias is an SSC output curve name.

2. src-tauri/src/ssc.rs:311-347 — sspw_spec's outputs are literally named PHIT_SSPW, CBW_SSPW, CAPBW_SSPW (lines 338-341) — different names for the physically equivalent quantities, exactly as claimed. No code anywhere resolves the SSC-vs-SSPW naming ambiguity (the doc comment at lrlc.rs:44-45 flags it but nothing acts on it).

3. src/ui/moduleDialog.ts — logChoiceNames (line 96) and fillSelect (lines 84-93) confirmed verbatim: fillSelect adds no class/title/styling to distinguish a not-yet-computed curve from a real one. logChoiceNames is used at line 102 (initial render) and again at line 250 inside refreshData (dataVersion refresh), matching the cited call sites exactly.

4. src-tauri/src/workflow.rs:135-143 — the dialog's chosen (or default) mnemonic is used with zero validation against the live catalog (`req.log_inputs.get(&a.name).cloned().unwrap_or_else(|| a.default.clone())`). Lines 198-204 (finding said 199-202, essentially the same block) confirm the missing-mnemonic fallback: `columns.get(&mnemonic...).cloned().unwrap_or_else(|| vec![f32::NAN; depth.len()])`.

5. lrlc.rs:88 (sw_rtc) and :191 (sw_imts) confirm the per-sample guard `if rt_i.is_nan() || pt.is_nan() || ... { continue; }` — a NaN PHIT alone skips every sample, leaving every output vector all-NaN while still returning a populated HashMap (non-empty map of curve names, just NaN-filled).

6. workflow.rs — Outcome success/failure is gated only on `outputs.is_empty()` (map cardinality, i.e. did the module return its named curves at all), never on whether any value in those curves is finite. rows_written = depth.len() at line 414 confirmed unconditional.

7. moduleDialog.ts:316-322 confirms the run-result rendering prints a green "✓ ... N samples → <curve list>" line keyed only on `!r.error`, with no finite-sample check.

8. I additionally checked whether `required: true` on the log_in args is enforced anywhere (grepped `.required` and modules.rs run_module dispatch) — it is not; `required` only suppresses the "(optional)" UI label (moduleDialog.ts:104) and is otherwise inert. There is no backend or frontend gate that blocks a run when a required curve is absent from the catalog.

9. I also checked equations::list_curve_catalog (equations.rs:247-269) and found it queries `computed_curves` with NO well_id filter — the catalog is global across the whole project, not scoped to the well(s) being run. This doesn't refute the finding; if anything it broadens the failure mode described (a curve computed on any other well in the project will appear as a normal-looking option for a well that never computed it, and the backend's per-well fetch will still silently NaN-fill it).

I found no gating logic, validation, or downstream QC check anywhere in the pipeline that would catch or surface this failure mode. The finding's evidence, line references, and causal chain all hold up under direct inspection.

</details>

### 2. docs/method_lrlc_rtc_imts.md still states the IMTS clay term multiplied by Sw — the exact transcription error the code already fixed and tested

**Area:** Domain correctness (dim C)

**Effort:** small

**Where:** docs\method_lrlc_rtc_imts.md:39; src-tauri\src\lrlc.rs:222 (fixed code) and :144 (fixed in-app spec string)

**Evidence:** docs/method_lrlc_rtc_imts.md line 39 reads: "Ct = Sw^n*/F* · [Cw + B·S·(ΣVmin_i·CEC_lit_i)·ρg·(1−φt)/(100·φt·(1−Swirr))·Sw]" — the trailing ·Sw multiplies the clay/Qv term. The actual code at lrlc.rs:222 computes `let denom = cw + b * qv_eff / sw.max(1e-6);` — Sw divides the term — with an adjacent comment explicitly noting this was corrected from an old `* sw` bug, and the in-app spec doc string at lrlc.rs:144 already reads '...Cw + B·Qv_eff/SwT...'. This is the same formula AUDIT-2026-07-20.md's medium-severity finding ('sw_imts clay term scales as Sw^(n*+1)...') confirmed as wrong by extracting the OMML from the primary docx source (Sw in the denominator is correct; the headline result 'IMTS SwE at/slightly below Waxman-Smits' only holds under division) — that finding is now marked fixed in REVIEW.md/ROADMAP.md with regression test `imts_credits_clay_conductivity_in_pay_zone`. But that fix's own suggested-fix text only named 'the docstring at lrlc.rs:144' and 'the memory file method_lrlc_imts_rtc.md' for correction — it never touched this docs/ file, which still carries the disproven multiplicative form. Per this audit's own convention that docs win over code on conflict, a reader trusting this file over the code would conclude the current (correct) code is wrong.

**Suggested fix:** Edit docs/method_lrlc_rtc_imts.md line 39 to divide the clay/Qv term by Sw (matching lrlc.rs:144 and :222), so the doc, the in-app spec string, and the code all agree.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I tried to refute this and could not — every piece of evidence checks out against the real files in D:\XX. SandiBumi (the SandiBumi repo).

1. docs\method_lrlc_rtc_imts.md line 39 (read directly, then byte-inspected with xxd to rule out a rendering artifact): the line literally ends "...ρg·(1−φt)/(100·φt·(1−Swirr))·Sw]**" — plain text, no hidden fraction-bar markup, no LaTeX. This unambiguously multiplies the clay/Qv term by Sw, exactly as the finding states.

2. src-tauri\src\lrlc.rs:222 (read directly): `let denom = cw + b * qv_eff / sw.max(1e-6);` — Sw divides the clay term. Lines 218-221 carry an explicit comment: "the excess-conductivity term is referenced to the ACTIVE water, so it DIVIDES by Sw ... (the old `* sw` gave Sw^(n*+1), understating clay conductivity and overstating Sw in pay)." This is the fixed code.

3. lrlc.rs:144 (read directly): the in-app docstring reads "Ct = SwT^N*/F*·(Cw + B·Qv_eff/SwT)" — division, matching the fixed code, not the doc file.

4. A regression test exists — `imts_credits_clay_conductivity_in_pay_zone` (lrlc.rs:375-401) — whose own comment says the assertion "fails under that bug" (the old ·Sw form), confirming the fix is tested.

5. AUDIT-2026-07-20.md (the prior audit) confirmed this against the primary docx source via OMML extraction ("Sw is in the DENOMINATOR... the source model is Ct = Sw^n*/F*·(Cw + B·Qv_eff/Sw)") and its own suggested fix explicitly named only two locations to correct: "the docstring at lrlc.rs:144" and "the memory file method_lrlc_imts_rtc.md" — it never mentions docs\method_lrlc_rtc_imts.md, even though that audit's confirmation note says "both the module docstring (lrlc.rs:144) and the memory file method_lrlc_imts_rtc.md line 29 carry the same trailing-·Sw misreading" — again omitting the docs/ file, which in fact carries the identical misreading.

6. REVIEW.md:123-128 marks this fixed ("IMTS clay-conductivity direction fixed"), citing the code change and the regression test only — no mention of docs\method_lrlc_rtc_imts.md.

7. Git history confirms the timeline: docs\method_lrlc_rtc_imts.md's only commit is f8a07d2 (2026-07-20 02:05:43), which predates the fix commit 8f185b3 (2026-07-20 23:27:21) that modified lrlc.rs, REVIEW.md, and ROADMAP.md. The doc file was never touched by the fix and still carries the pre-fix, disproven multiplicative formula.

Every load-bearing claim in the finding (doc text, code text, docstring text, test existence/intent, audit's fix scope, and doc file's git history) is verified accurate. I found no discrepancy to refute the finding on.

</details>

### 3. SSC's 8-curve GR-equivalent output family (*_GR) has zero unit-test coverage

**Area:** Backend test coverage (dim B)

**Effort:** small

**Where:** src-tauri\src\ssc.rs:253-276 (computation), :425-543 (test module)

**Evidence:** ssc_spec() declares 23 log_out curves; 8 of them — VSAND_GR, VSILT_GR, VDCL_GR, CBW_GR, CWSH_GR, PHIFF_GR, PHIE_GR, PHIT_GR — are produced entirely by the GR-rescaling block at ssc.rs:253-276, which has three independent 0.005 thresholds (on vsand, cwsh, phiff, each gating a different NaN/zero branch) plus a 1e-9 degenerate-vwsh guard that skips the whole block (leaving all 8 curves NaN) when vwsh is at or near 0 or 1. None of the five tests in ssc.rs's #[cfg(test)] module (ssc_clean_sand_is_mostly_sand, ssc_shale_point_is_clay_dominated_with_low_phie, ssc_swirr_floor_pads_capillary_water, sspw_phie_removes_only_clay_bound_water, sspw_clean_sand_has_no_bound_water) reads or asserts on any *_GR output, and none set VSHGR to a live GR-derived value that would exercise the vshgr.is_nan() branch guard. This is distinct from the open ROADMAP item (ROADMAP §Phase 8.5 / REVIEW ~line 1249-1255) asking Jauhar to manually validate SSC against a real the reference suite LAS export for domain sign-off — that's a one-time accuracy check; this is that the automated regression suite would not notice if the entire GR-equivalent computation were broken, silently changed, or deleted.

**Suggested fix:** Add at least one regression test that sets a GR curve/VSH_MA/GR_SH so vshgr is finite, and asserts the closure/branch behavior of the *_GR family (e.g., VSAND_GR+VSILT_GR+VDCL_GR+PHIT_GR ≈ 1, plus one case each hitting the vsand<=0.005 and phiff<=0.005 branches and the vwsh-near-0/1 all-NaN guard).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the real files at D:\XX. SandiBumi\src-tauri\src\ssc.rs (543 lines total) and lrlc.rs. I could NOT refute the finding — every checkable claim holds up:

1. Curve counts: ssc_spec() (lines 82-104) declares exactly 23 log_out curves; exactly 8 are the *_GR family (VSAND_GR, VSILT_GR, VDCL_GR, CBW_GR, CWSH_GR, PHIFF_GR, PHIE_GR, PHIT_GR) — confirmed by direct count.

2. Location citations are exact: the GR-rescaling block is lines 253-276 verbatim (comment "GR-equivalent volumes: rescale..." at 253 through `let phit_g = phie_g + cbw_g;` at 276); the test module is lines 425-543 verbatim (file is exactly 543 lines, `mod tests {` at 425, closing brace at 543).

3. The 0.005 thresholds and the 1e-9 vwsh guard are real: line 257 `if !vshgr.is_nan() && vwsh > 1e-9 && vwsh < 1.0 - 1e-9` gates the entire block; lines 261-273 have threshold checks on cwsh, vsand, and phiff (phiff's check appears twice — once nested in the vsand_g branch, once for phiff_g itself — so it's arguably 4 threshold evaluations across 3 variables rather than a clean "3 independent thresholds," but the characterization is materially accurate).

4. Zero-coverage claim confirmed two ways: (a) none of the 5 tests in the `#[cfg(test)]` module reference any `_GR` name (grep confirms `_GR` only appears in the spec/computation, never in the test module); (b) a repo-wide grep for VSAND_GR/PHIT_GR/etc. across all of D:\XX. SandiBumi returns matches only in ssc.rs itself — no test file, frontend test, or integration test anywhere touches these outputs.

5. I hand-translated the actual algorithm (gas conditioning → N-D projection → fraction split → PHIT → bound-water split → GR rescale block) into Python and ran it against the exact 3 SSC test fixtures (clean sand, shale point, swirr-floor). Confirmed the shale-point test (RHOB=2.30/NPHI=0.60, exactly the wet-clay anchor) drives vwsh to exactly 1.0, tripping the 1e-9 upper guard and leaving all 8 *_GR curves NaN — exactly as the evidence describes.

6. One inaccuracy in the evidence, which does not weaken the finding: the write-up says "none set VSHGR to a live GR-derived value that would exercise the vshgr.is_nan() branch guard." My replica shows this is wrong — in the clean-sand tests (test 1 and test 3), GR=15.0 with default GR_MA=10/GR_SH=150 already yields a finite VSHGR≈0.0357, and vwsh≈0.00484 clears the 1e-9 guard, so the GR block actually executes and produces real, non-NaN numbers for all 8 curves in those two tests already — completely unasserted. This makes the coverage gap arguably worse (real computed values are already flowing through untouched), not better, so it doesn't rescue the finding.

7. Distinctness from the open ROADMAP/REVIEW item is correctly cited: REVIEW.md lines 1249-1255 (file is 1296 lines) under "## Phase 8.5 — your method suite in core (remaining validations)" is exactly the manual "validate against your the reference suite run" item mentioning `*_GR` GR-equivalent volumes — a one-time domain sign-off task, distinct from automated regression coverage.

8. Scope check on the "audit together" framing: lrlc.rs's sw_rtc/sw_imts only consume PHIT_SSC, CBW, and CWSH(as CAPBW)/SWIRR_T from ssc.rs — none of the 8 *_GR curves — so there is no indirect test coverage of the *_GR family via the lrlc.rs test suite either.

Net: the finding's core claim (zero unit-test coverage of the 8-curve *_GR family, with the vwsh/threshold guards described) is accurate and precisely cited. The suggested fix (closure test VSAND_GR+VSILT_GR+VDCL_GR+PHIT_GR≈1, plus branch cases) is also numerically sound — I verified the closure holds to ~0.4% in the clean-sand case using my replica.

</details>

---

## precalc / SandiMin (multimin2) / gascorr

### 1. Module-run status reports "✓ success" even when every output sample is MISSING

**Area:** precalc/gascorr backend + frontend (shared workflow.rs module-run path)

**Effort:** small-medium

**Where:** src-tauri/src/workflow.rs:395-418 (run_workflow_module_into, esp. line 414 `rows_written: depth.len()`); displayed at src/ui/moduleDialog.ts:321 and totalled at src/ui/inspectorPanel.ts:550 for History

**Evidence:** gascorr(ctx) in src-tauri/src/modules.rs:1428-1522 always returns Ok(HashMap) with all 4 keys populated (RHOB_GC/PHIT_GC/SWT_GC/GASDEN), even when every value is f32::NAN — e.g. the documented/tested no-precalc case (`gascorr_flag_gate_and_missing_inputs`, modules.rs:3543-3550: `out["RHOB_GC"][0].is_nan()`). Because `outputs.is_empty()` is false (4 keys exist), workflow.rs's run_workflow_module_into falls into the final branch and sets `rows_written: depth.len()` — the full well sample count — with no check that any value is finite. moduleDialog.ts then renders `✓ {well}: {rows_written} samples → RHOB_GC, PHIT_GC, SWT_GC, GASDEN`, and inspectorPanel.ts sums the same number into the History entry. So running gascorr on a well that never had precalc run (REVIEW.md's own documented click-through case, "outputs stay MISSING") shows a normal-looking green success line with a full sample count, with nothing distinguishing it from a real correction. Contrast with multimin2.rs's own MultiminWellResult (multimin2.rs:690-838), which counts only actually-solved samples (`rows_solved`) and returns an explicit error when `solved == 0` — i.e. a sibling subsystem in the same codebase already solved this correctly, so the gap looks like an oversight rather than an intentional design. No test in modules.rs or workflow.rs exercises `rows_written` for an all-NaN-output run (all gascorr tests call `gascorr(&ctx)` directly, bypassing workflow.rs).

**Suggested fix:** In run_workflow_module_into's Computed branch, count samples where at least one output curve is finite (or require all declared outputs to have >=1 finite value) and report that as rows_written; if zero, surface it distinctly (e.g. a warning string) instead of a bare success with the full depth count.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every cited line against D:\XX. SandiBumi (the actual SandiBumi/Arshilla repo, confirmed via memory note that the folder name is unchanged post-rename). All claims in the finding hold up.

1. src-tauri/src/workflow.rs, run_workflow_module_into (fn starts at line 100, matches the cited 395-418 region for the tail): the final match arm is exactly
   `ModuleRunResult { well_id: well_id.clone(), rows_written: depth.len(), output_curves: names, error: None }` at line 414 — verbatim match to the finding. This branch is reached whenever `Outcome::Computed{outputs,..}` has `!outputs.is_empty()` (checked at lines 314 and 403), with zero inspection of whether any value inside those output Vecs is finite.

2. src-tauri/src/modules.rs, `fn gascorr` spans exactly 1428-1522 as cited. It always returns `Ok(HashMap::from([("RHOB_GC",...), ("PHIT_GC",...), ("SWT_GC",...), ("GASDEN",...)]))` (lines 1516-1521) — four keys, each a `Vec<f32>` of length `ctx.n`, populated up front with `f32::NAN` (lines 1445-1448) and only overwritten per-sample when the physics actually converges. There is no code path that returns an empty HashMap or an Err purely because every sample stayed NaN (the only Err path is the FLAGGED-with-no-flag-data guard at 1434-1443, unrelated to missing precalc outputs). So `outputs.is_empty()` is always false for gascorr, confirming the finding's central mechanism.

3. Test `gascorr_flag_gate_and_missing_inputs` (starts line 3514): the no-precalc case build at lines 3543-3549 (RHOB/RT/FTEMP only, FPRESS absent) calls `gascorr(&ctx).unwrap()` and asserts `out["RHOB_GC"][0].is_nan() && out["PHIT_GC"][0].is_nan()` at line 3550 — matching the cited evidence almost verbatim (the finding names line 3543-3550, which is exactly this block). REVIEW.md line 605 independently documents "Without precalc run: outputs stay MISSING (never uncorrected pass-through)" — confirming this is a known, tested, intentional *computation* behavior, but nothing in that checklist addresses the *reporting* layer.

4. src/ui/moduleDialog.ts line 321 is exactly: `` `✓ ${well?.well_name ?? r.well_id}: ${r.rows_written} samples → ${r.output_curves.join(", ")}` `` with `className = "result-ok"` (line 322) whenever `r.error` is falsy — confirmed verbatim.

5. src/ui/inspectorPanel.ts line 550 is exactly: `const totalRows = ok.reduce((sum, r) => sum + r.rows_written, 0);` — confirmed verbatim, feeding the History summary text at line 551.

6. multimin2.rs contrast: `solved` is only incremented inside the per-sample loop after `solve_bounded_lsq` actually returns `Some(x)` (line 767, inside the loop starting ~694); the well result at lines 833-838 sets `rows_solved: solved` and `error: write_err.or_else(|| (solved == 0).then(|| "no solvable samples (too few live input logs)".to_string()))` (line 837) — i.e., sibling code in the same file distinguishes "wrote rows" from "actually solved anything" and surfaces a zero-solved run as an explicit error rather than a bare success. This is a materially different, more correct pattern than workflow.rs's blanket `depth.len()`, exactly as the finding contrasts.

7. Checked for mitigating factors that might refute the claim and found none: (a) no frontend gating in moduleDialog.ts stops a user from running gascorr on a well without precalc — grepped for FTEMP/FPRESS/precalc references there, none exist; (b) `write_computed_curves_versioned_batch` (equations.rs:675-730) writes every row including NaN values with no finiteness check and no error, so `write_err` stays `None` for the all-NaN case, meaning the code genuinely reaches the `rows_written: depth.len()` success arm; (c) grepped workflow.rs for any test exercising `rows_written` for an all-NaN run — none found; the only test resembling end-to-end coverage (`test_full_deterministic_chain`, line 1609) is `#[ignore]`d, machine-path-specific, and never invokes gascorr with missing precalc inputs, so the "no test covers this" claim also holds.

One very minor imprecision in the finding: it cites "multimin2.rs's own MultiminWellResult (multimin2.rs:690-838)" as if that's the struct's declaration range, but the struct itself is declared at line 139; lines 690-838 are actually the well-processing/solve loop that constructs and returns `MultiminWellResult` values. This is a cosmetic mislabeling, not a substantive error — the described counting behavior and line 835/837 content are exactly where claimed.

No code, test, or UI logic contradicts the finding. The bug is real and precisely as described: `rows_written` (and everything downstream that displays or totals it) reports full success based purely on HashMap-key presence, never on whether any written value is finite.

</details>

### 2. SandiMin's free-text Output Prefix isn't case-normalized, giving the confirmed db-write-versioning-discipline bug a second live trigger

**Area:** multimin2.rs backend / multiminDialog.ts frontend

**Effort:** small

**Where:** src-tauri/src/multimin2.rs:644-645 (prefix taken verbatim from req.output_prefix, no .to_uppercase()) feeding curve names at lines 792-812 (`format!("{prefix}_PHIE")` etc.) written via write_computed_curves_versioned at line 830; contrast with curve_token() at multimin2.rs:158-166 which DOES uppercase component names for the VOL_<component> curves in the same output set; frontend field at src/ui/multiminDialog.ts:616-618 (`prefixInp.value = "MM"`, free-text input) and :670 (`output_prefix: prefixInp.value.trim() || "MM"`)

**Evidence:** The already-confirmed substrate finding [db-write-versioning-discipline] establishes that write_computed_curves_versioned's DELETE (equations.rs:588) matches curve_name by exact case while every read path resolves case-insensitively, so a re-cased curve name leaves a stale shadow row that can silently win over the fresh value. multimin2.rs's own component-derived curve names (VOL_QUARTZ, VOL_ILLITE, ...) are protected against this because curve_token() forces `.to_uppercase()`. But the aggregate curves in the exact same output batch — `<prefix>_PHIE`, `_PHIT`, `_SWE`, `_SWT`, `_SXOT`, `_MOVEDHC`, `_VSH`, `_RECON` — are built directly from the user-typed Output Prefix field with zero case normalization on either the frontend (a plain text input, no forced uppercase) or backend (`prefix.trim()` only). Re-running SandiMin on the same well with a differently-cased prefix (e.g. typing "mm_zone2" one session and "MM_Zone2" another, or just "mm" vs the field's own default "MM") creates exactly the re-cased-name scenario the substrate finding describes: a stale shadow row in computed_curves that a case-insensitive reader (fetch_computed_curve_aligned, fetch_computed_curves_batch) can pick over the intended fresh value.

**Suggested fix:** Uppercase (or otherwise canonicalize) `output_prefix` the same way curve_token() does for component names, before building the aggregate curve names — one line in run_multimin alongside the existing `let prefix = ...` block.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently read the actual source and every cited line matches the finding exactly:\n\n1. multimin2.rs:644-645 — `let prefix = req.output_prefix.trim(); let prefix = if prefix.is_empty() { \"MM\" } else { prefix };` confirmed: only `.trim()`, no `.to_uppercase()`.\n2. multimin2.rs:158-166 `curve_token()` confirmed to `.to_uppercase()` component names for VOL_<component> curves — the asymmetry cited is real.\n3. multimin2.rs:792-812 confirmed: `format!(\"{prefix}_PHIE\")`, `_PHIT`, `_SWE`, `_SWT`, `_SXOT`, `_MOVEDHC`, `_VSH`, `_RECON` all built directly from the unnormalized `prefix`.\n4. multimin2.rs:829-830 confirmed: writes go through `create_log_set` + `write_computed_curves_versioned`.\n5. multiminDialog.ts:616-618 confirmed: `prefixInp` is a free-text `<input>` defaulting to \"MM\", no forced casing, no persistence of a prior run's exact prefix across dialog opens.\n6. multiminDialog.ts:670 confirmed: `output_prefix: prefixInp.value.trim() || \"MM\"` — trim only.\n7. Tauri command wrapper (lib.rs:566-568) is a bare passthrough with no normalization inserted at the boundary.\n8. Re-verified the substrate mechanism itself rather than trusting it: equations.rs:588 DELETE matches `curve_name IN (...)` with no `upper()` (exact case), while equations.rs:376-377 (`fetch_computed_curves_batch`) and :412 (`fetch_computed_curve_aligned`) both read via `upper(curve_name)` (case-insensitive). `fetch_computed_curves_batch` (389-392) resolves collisions via an unordered HashMap insert (\"last row wins\"), so a stale differently-cased row and a fresh row landing under the same case-folded name resolve non-deterministically.\n9. db.rs:118-125 confirms `computed_curves` has NO primary key / uniqueness constraint at all — uniqueness depends entirely on the exact-case delete-then-append write discipline, so there is no schema-level (e.g. collation) safety net that would neutralize the case mismatch.\n10. Confirmed gascorr/precalc (modules.rs) have no `output_prefix` concept, so this is specific to SandiMin as the finding states, not a shared gascorr bug.\n\nNo mitigating code path exists anywhere in the call chain (backend, frontend, or Tauri command layer). Every specific claim — file paths, line numbers, code content, and the causal link back to the already-confirmed db-write-versioning-discipline bug — holds up under direct inspection of the real files.

</details>

### 3. SandiMin's "Autofill from precalc" Read button has no stale-response race guard, unlike refreshZones() in the same file

**Area:** multiminDialog.ts frontend

**Effort:** small

**Where:** src/ui/multiminDialog.ts:304-343 (autofillBtn click handler — applies `pf.ftemp_f`/`pf.rmf` unconditionally after the await) vs :275-293 (refreshZones(), which guards with `if (selectedWell?.well_id !== wid) return;` at line 283)

**Evidence:** refreshZones() in the same file explicitly protects against a stale async response by capturing `wid` before the await and bailing if `selectedWell` has changed by the time `listZones` resolves — proving the pattern (and its necessity) is already recognized in this file. The autofillBtn handler a few lines below does not: it captures `well` at click time, awaits `multiminFluidFromPrecalc(well.well_id, ...)`, and on resolution writes directly into `ftInp.value`/`rmfInp.value`/`rmfTInp.value` with no check that `well` is still the active selection. Sequence: click Read for Well A (slow — e.g. contending with the single global Mutex<Connection> under a concurrent batch run, the already-known #129 condition), switch to Well B via the Wells panel, click Read for Well B (fast, resolves first and fills the fields correctly), then Well A's late response arrives and silently overwrites the fields with Well A's FTEMP/RMF while the dialog still displays Well B's context — with no error, no stale-check, and no visual cue.

**Suggested fix:** Capture `well.well_id` before the await and skip applying the result if `selectedWell?.well_id !== well.well_id` when it resolves, mirroring refreshZones()'s existing guard a few lines above.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read D:\\XX. SandiBumi\\src\\ui\\multiminDialog.ts directly (this is the SandiBumi repo, folder retains its pre-rename name per user memory). Confirmed line-for-line: refreshZones() (lines 275-293) captures `wid` before the `listZones` await and guards with `if (selectedWell?.well_id !== wid) return;` at line 283. The autofillBtn click handler (lines 304-343) captures `well` at line 305, awaits the async Tauri IPC call `multiminFluidFromPrecalc` (line 312, which invokes the Rust command `multimin_fluid_from_precalc` per src/ipc.ts:782-788), and then unconditionally writes `pf.ftemp_f`/`pf.rmf` into `ftInp.value`/`rmfInp.value`/`rmfTInp.value` (lines 326-332) with no check that `well.well_id` still matches the live `selectedWell`. I additionally verified there is no other mitigating mechanism: no generation/request-id counter, no AbortController, and the button is never disabled during the in-flight request (all greps returned no matches), so nothing prevents a second click for a different well while the first request is still pending, and the same closure-level `selectedWell` variable that refreshZones checks is indeed updated on well switch via appState.selectedWell.subscribe (lines 296-299). The backend does use a shared Mutex<Connection> pattern (confirmed present across multiple src-tauri/src/*.rs files including multimin2.rs and lib.rs), which is consistent with the finding's premise that request latency can vary and responses can arrive out of order. The finding's described sequence (slow Read for well A, switch to well B, fast Read for B applies correctly, then A's stale response silently overwrites B's fields) is therefore a genuine, reproducible defect, and the suggested fix (capture well.well_id before the await, bail if selectedWell?.well_id !== well.well_id on resolve, mirroring refreshZones' existing guard) is accurate and appropriately scoped.

</details>

---

## Dead & stub code (petrophysics.rs, inversion.rs)

### 1. petrophysics.rs confirmed fully dead — never compiled, zero references anywhere; recommend outright deletion, not quarantine

**Area:** backend/dead-code (dimension B/F)

**Where:** src-tauri/src/petrophysics.rs (whole file, 137 lines); src-tauri/src/lib.rs lines 1-34 (mod list — no `mod petrophysics;`); src-tauri/Cargo.toml (no [[bin]]/[[test]] path override)

**Evidence:** Confirmed the premise, not just repeated it. `mod petrophysics;` does not exist anywhere in src-tauri/src (grep for `mod petrophysics|petrophysics::` across the whole src tree: zero hits; lib.rs's mod list at lines 1-34 declares `mod inversion;` at line 15 but never `mod petrophysics`). No `#[path = "petrophysics.rs"]` override or `include!()` macro exists anywhere in src-tauri/src that could sneak it into the build another way. Cargo.toml has no [[bin]]/[[test]] target naming it. Since rustc only compiles files reachable from a `mod` chain rooted at lib.rs, this file is not part of the build at all today — not compiled, not type-checked, not linted, not runnable. The two frontend-side identifiers it would expose if wired (run_multi_well_evaluation/WellEvaluationResult) have zero matches anywhere under src/. Beyond "dead," its design is architecturally obsolete relative to the current module system, so there is nothing worth preserving as a quarantined reference: run_multi_well_evaluation takes a raw `&Mutex<Connection>` directly instead of going through ModuleContext/the modules.rs manifest, never writes computed_curves/log_sets (no provenance, unlike every live module), has no MASK support, and silently drops any well whose curve fetch errs (`fetch_well_curves(&conn, well_id).ok()?` inside a `filter_map`, no error surfaced) — the opposite of this codebase's per-item-isolated-with-visible-error convention (ImportResult/ModuleRunResult). Its actual math (linear Vsh, density porosity, plain Archie Sw) is already implemented, live-wired, MASK-aware and versioned via modules.rs's vsh_gr/phi_dn(phi_den)/sw_arch, per docs/qc_audit_prompt_template.md's own tool inventory (lines 141-162) — so nothing here is a unique capability.

**Suggested fix:** Delete src-tauri/src/petrophysics.rs outright. Quarantining it (e.g. `#[allow(dead_code)] mod petrophysics;` or moving it to a `dead_code/` folder) adds no safety it doesn't already have — it can't run either way — and there's no unique logic worth keeping as a template since modules.rs already has the live, better-integrated equivalents.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I attempted to refute this finding by independently re-deriving every piece of evidence from the actual repository (D:\XX. SandiBumi, the current SandiBumi/ex-Arshilla folder) rather than trusting the write-up, and could not find anything that contradicts it.

Verified directly:
1. Read src-tauri/src/lib.rs lines 1-34 myself: the mod list has 32 entries (chain, composite, curve_edit, curves, db, decimate, deviation, dlis, equations, export, facies, geo, health, ingest, inversion, jobs, layout, lrlc, ml, modules, montecarlo, multimin, multimin2, neutron_charts, parsers, pipeline_blso_test [cfg(test)], project, report, satheight, ssc, python_engine, tops, workflow) — no `mod petrophysics;` anywhere. `mod inversion;` is present at line 15, matching the claim.
2. Grepped src-tauri/src for `#[path = "petrophysics` and `include!(` — zero hits, ruling out any alternate way to pull the file into the build.
3. Read Cargo.toml in full — only [package]/[lib]/[build-dependencies]/[dependencies]/[target.'cfg(windows)'.dependencies]/[profile.release]; no [[bin]] or [[test]] section at all, let alone one naming petrophysics.rs.
4. Glob'd the whole project tree for Cargo.toml — only one exists (src-tauri/Cargo.toml), so there's no workspace member trick either.
5. Went one step further than the original evidence: checked src-tauri/tests/ (Cargo auto-compiles any tests/*.rs as its own integration-test binary without needing a Cargo.toml [[test]] entry, which the original evidence didn't address). That directory contains only two binary DuckDB fixture files (corrupt_torn.duckdb/.wal), no .rs files — so Cargo's implicit test-discovery convention doesn't sneak petrophysics.rs in either.
6. main.rs is a one-line `sandibumi_lib::run()` and build.rs a one-line `tauri_build::build()` — nothing hidden there.
7. Read petrophysics.rs in full (137 lines) and confirmed the architectural claims: `run_multi_well_evaluation` takes `db: &Mutex<Connection>` directly (not ModuleContext), contains no SQL writes/INSERTs at all (so no computed_curves/log_sets provenance), no MASK handling, and line 112 is exactly `fetch_well_curves(&conn, well_id).ok()?` inside a `.par_iter().filter_map(...)` — a silent per-well error drop, as described.
8. Grepped the actual frontend source (src/, confirmed real via Glob — ~50 .ts files) for `run_multi_well_evaluation|WellEvaluationResult|EvaluationParams` (and camelCase/snake_case variants like multiWell/multi_well/evaluateWell): zero matches. Also read src/ipc.ts in full — it's the single exhaustive file binding every Tauri `invoke()` call the frontend makes, and it has no binding for any petrophysics command (nor, incidentally, for inversion's start_inversion/get_inversion_status, corroborating the adjacent inversion.rs stub-exposure context you mentioned).
9. Spot-checked modules.rs and confirmed vsh_gr, phi_den/phi_dn, and sw_arch are real, live, wired module implementations (matching the reference suite .lls names in the header comment), supporting the claim that petrophysics.rs's math is already superseded by the live module system.

Every specific factual claim in the "Evidence given" held up against the real files, and my one extra check (tests/ directory) closed a gap the original evidence hadn't covered rather than opening one. I found no basis to refute the finding — it stands as accurate.

</details>

### 2. inversion.rs's start_inversion is not just a fake-data stub — its dispatch_inversion calls bare tokio::spawn from a sync #[tauri::command], which this project's own code proves panics in that exact context; recommend deleting the IPC exposure entirely rather than quarantining

**Area:** backend/IPC-async-runtime (dimension B, cross-referenced with AUDIT-2026-07-20.md and ROADMAP.md #128)

**Where:** src-tauri/src/inversion.rs lines 49-64 (dispatch_inversion: `tokio::spawn(async move { ... tokio::task::spawn_blocking(...) ... })`); src-tauri/src/lib.rs lines 793-796 (start_inversion — a bare, non-`async` `#[tauri::command] fn`), lines 856-868 (the corroborating comment on run_workflow_chain), lines 956 (.manage(inversion::new_registry())), 1026-1027 (generate_handler! registration)

**Evidence:** Confirmed start_inversion/get_inversion_status are live on the Tauri IPC surface (registered in generate_handler!, registry `.manage`d) with zero frontend callers anywhere in src/ (grep for start_inversion|get_inversion_status across the repo: only lib.rs's own wrapper and docs/qc_audit_prompt_template.md, which is the audit checklist itself). Beyond that already-known fact, I found the stub is actively unsafe to invoke: start_inversion is a plain (non-async) `fn`, so Tauri v2 runs it directly on its own command-dispatch thread with no Tokio runtime entered on that thread. dispatch_inversion's body then calls the bare `tokio::spawn(...)` free function (inversion.rs:53), which requires `Handle::current()` to succeed on the CALLING thread or it panics. I reproduced this exact failure in an isolated repro (plain `std::thread::spawn` invoking `tokio::spawn` with no runtime entered): panic message `there is no reactor running, must be called from the context of a Tokio 1.x runtime` — the identical message class Tokio raises for this situation. This project's own code independently corroborates the mechanism: lib.rs lines 860-867 (on run_workflow_chain, written after the 2026-07-20 audit) explicitly documents hitting this exact wall for the sibling API — "a sync #[tauri::command] runs on the main event-loop thread, which is NOT a Tokio runtime worker, so tokio::task::spawn_blocking panics there" — which is why they replaced `tokio::task::spawn_blocking` with `std::thread::spawn` for that command. `tokio::spawn` and `tokio::task::spawn_blocking` share the same runtime-context requirement, and start_inversion is a sync command exactly like the pre-fix state of run_workflow_chain. No test exercises this path (grep for `#[test]` in inversion.rs: zero matches), so the crash has evidently never been triggered — but it would fire on the first real invocation (devtools `invoke('start_inversion', ...)`, or a future dev wiring a button to it, which is a real risk since AUDIT-2026-07-20.md's own confirmed HIGH finding (lib.rs:670-690, verifier note at line 75) literally recommends copying "the dispatch_inversion pattern" to fix 7 other sync commands, and ROADMAP.md line 628 (#128, still open) proposes the same "async + spawn_blocking" approach for run_ml/run_multimin/etc. Both point at the broken pattern.

**Suggested fix:** Delete inversion.rs's IPC exposure entirely: remove the `start_inversion`/`get_inversion_status` `#[tauri::command]` fns (lib.rs:793-806), their two `generate_handler!` entries (lib.rs:1026-1027), and `.manage(inversion::new_registry())` (lib.rs:956); then remove `mod inversion;` and the file. Do not quarantine-in-place as a "reference pattern" — it isn't one: jobs.rs (Phase 11) already generalizes the same registry+poll+cancel idea using the working `std::thread::spawn` approach (jobs.rs doc comment: "Generalises the chain-specific registry in chain.rs"), and that is the pattern actually proven in production by run_workflow_chain today. Separately flag for whoever next picks up ROADMAP #128 (run_ml/run_multimin/import_las_files/export_report_batch/run_equation/run_workflow_module still sync per AUDIT-2026-07-20.md line 75): copy run_workflow_chain's std::thread::spawn + jobs.rs registration (lib.rs:852-880), not dispatch_inversion's tokio::spawn — the latter will panic the app the same way it would for start_inversion.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the real repo at D:\XX. SandiBumi\src-tauri and against the actual cached Tauri v2.11.5 / tokio 1.52.3 crate source (the exact versions this project's Cargo.lock pins).

Source-line verification (all exact matches to the finding): petrophysics.rs has zero `mod petrophysics;` declaration anywhere in lib.rs's 34 mod statements (confirmed dead code); `mod inversion;` IS declared (lib.rs:15); inversion.rs:49-64 dispatch_inversion body matches verbatim (bare tokio::spawn wrapping tokio::task::spawn_blocking); lib.rs:793-796 start_inversion is a plain non-async fn calling dispatch_inversion directly; lib.rs:860-867 comment on run_workflow_chain matches the finding's paraphrase almost verbatim; lib.rs:956 and 1026-1027 match exactly. Zero #[test] in inversion.rs. Zero non-lib.rs/non-doc callers of start_inversion/get_inversion_status repo-wide. AUDIT-2026-07-20.md and ROADMAP.md #128 both corroborate exactly as cited, including the ironic detail that the prior audit's own verifier endorsed dispatch_inversion as "the correct" pattern to copy elsewhere. jobs.rs doc comment confirms it already generalizes the registry+poll idea using std::thread (the working approach), matching the suggested fix.

Mechanism verification (read library source directly rather than trusting the claim): Tauri's proc-macro (tauri-macros-2.6.3/src/command/wrapper.rs) shows a non-async #[tauri::command] fn executes synchronously inline via body_blocking -> resolver.respond(), no automatic executor offload. Tauri's own async_runtime.rs shows its RuntimeHandle::spawn wrapper explicitly does `let _guard = h.enter(); tokio::spawn(task)` before calling bare tokio::spawn — proof that Tauri itself treats ambient runtime context as NOT guaranteed on arbitrary calling threads, which is exactly why it built that entering wrapper instead of exposing raw tokio::spawn to users. tauri-runtime-wry-2.11.4/src/lib.rs has zero .enter()/Handle::current calls around its event loop/IPC dispatch. tokio 1.52.3's util/error.rs contains the exact CONTEXT_MISSING_ERROR panic string ("there is no reactor running, must be called from the context of a Tokio 1.x runtime") that the finding says it reproduced, in the exact tokio version this project locks (verified via Cargo.lock).

Nothing in the finding could be refuted; every checkable factual and technical claim held up under independent verification of both the project's own code and the actual third-party library internals it depends on.

</details>

---

## VSH (vsh_gr, vsh_dn)

### 1. vsh_gr / vsh_dn standalone module runs never leave the Tauri main thread, unlike the (now-fixed) workflow chain — no Processing-panel progress and no Cancel, on the exact same rayon batch engine that just froze the app at 540 wells

**Area:** Backend (Rust) / UI-UX — dimensions B & E

**Effort:** medium

**Where:** src-tauri/src/lib.rs:530-533 (`run_workflow_module` command) vs. lib.rs:812-882 (`run_workflow_chain`, esp. the `std::thread::spawn` at line 868 and its comment); src-tauri/src/workflow.rs:92-94 (`pub fn run_workflow_module` forwards `None, None, None` for preset_sets/cancel/progress into `run_workflow_module_into`, which does accept a `crate::jobs::JobHandle` — line 105 — when called from the chain path); src/ui/moduleDialog.ts:223-332 (Run button just does `await runWorkflowModule(req)` with no cancel affordance, only `runBtn.disabled = true`)

**Evidence:** `run_workflow_module` is a plain, non-async `#[tauri::command] fn` that calls `workflow::run_workflow_module(&db.0, &req)` — the exact same rayon-par_iter-then-one-batched-write engine `run_workflow_chain` uses — and blocks until every well finishes, returning the whole `Vec<ModuleRunResult>` in one IPC round trip. This is precisely the shape the project's own comment at lib.rs:860-867 says is dangerous: "a sync #[tauri::command] runs on the main event-loop thread... As a sync command this blocked the event loop for the whole multi-minute chain" — which is why `run_workflow_chain` was just converted (today, per REVIEW.md's dated 2026-07-21 top section) to `std::thread::spawn` + a pollable `jobs` registry. `run_workflow_module` — the command moduleDialog.ts's Run button calls for every vsh_gr/vsh_dn invocation — was not converted: no thread spawn, no job registration, no cancel flag reaching `run_workflow_module_into`'s `cancel: Option<&AtomicBool>` parameter (it's hardcoded `None`). Since the module dialog's Wells checklist explicitly supports selecting many/all wells for a single-module run (defaultRunWellIds pulls the sidebar's multi-selection, which can be the whole field), a user running VSH from Gamma Ray or VSH from Density-Neutron across a large well set directly (not via a saved Workflow chain) is exposed to the identical freeze symptom just fixed for chains — and has no Cancel button to recover, whereas chains now do. REVIEW.md's own follow-up list ("import, dashboard, multimin, Monte Carlo and equations will follow the *same* pattern") does not name this command, so it isn't a documented/deliberate deferral either.

**Suggested fix:** Give `run_workflow_module` the same treatment as `run_workflow_chain`: accept a `job_id`, register with `jobs::JobRegistry`, run `workflow::run_workflow_module_into(..., cancel: Some(&cancel), progress: Some(&job))` on a `std::thread::spawn`'d worker, and have moduleDialog.ts poll status / expose Cancel like workflowDialog.ts already does for chains.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently read every file/line cited and the finding's core technical claim holds up completely; I could not refute it on the code merits.

Verified against the actual source (D:\XX. SandiBumi):

1. lib.rs:530-533 — `run_workflow_module` is a plain, non-async `#[tauri::command] fn` that calls `workflow::run_workflow_module(&db.0, &req)` and returns `Vec<ModuleRunResult>` directly. No thread spawn, no job registration. Confirmed verbatim.

2. lib.rs:812-882 — `run_workflow_chain` was indeed converted today: it registers with `chain::register` and `jobs::register`, then does the real work inside `std::thread::spawn(move || { chain::run_chain(...) })` at line 868. The comment at lib.rs:860-867 explicitly states the team's own understanding that "a sync #[tauri::command] runs on the main event-loop thread" — this is the codebase's own documented rationale for why the conversion was necessary, which supports (not undermines) the finding's premise.

3. workflow.rs:92-94 — `pub fn run_workflow_module` is exactly `run_workflow_module_into(db, req, None, None, None)` — confirmed the hardcoded `None, None, None` for preset_sets/cancel/progress.

4. workflow.rs:100-106 — `run_workflow_module_into` signature confirmed: takes `cancel: Option<&std::sync::atomic::AtomicBool>` and `progress: Option<&crate::jobs::JobHandle>` as real, usable parameters — proving the plumbing already exists and is simply not wired up for standalone module runs.

5. workflow.rs:161-167 — confirmed `.par_iter()` (rayon) is the shared engine, with a cooperative cancel check (`cancel.map_or(false, ...)`) gated on the `cancel` option being `Some` — which it never is from `run_workflow_module`.

6. moduleDialog.ts:276-332 — Run button handler confirmed: builds `req`, sets `runBtn.disabled = true`, does `await runWorkflowModule(req)`, no job_id, no polling, no Cancel button — matches the finding exactly.

7. moduleDialog.ts:53-77 + state.ts:84-91 (`defaultRunWellIds`) — confirmed the Wells checklist is a per-well checkbox list that pre-ticks from `appState.multiSelectedWellIds`, which can be the whole field's multi-selection — so a large/whole-field standalone module run is a real, easily reachable user path.

8. jobs.rs:107 — `pub(crate) type JobRegistry = Arc<Mutex<JobStore>>` — confirms the suggested-fix's `jobs::JobRegistry` reference is accurate, not just approximate.

One nuance I found that slightly softens (but doesn't invalidate) the finding's framing: REVIEW.md line 66-67 ("This is the reusable spine: import, module runs, multimin, Monte Carlo and reports will each report into it...") DOES explicitly name "module runs" as planned future work — so this gap is arguably a documented/tracked deferral after all, just not in the specific bullet (line 35) the finding's evidence quoted. This is a minor correction to one sentence of the narrative, not to the technical substance: the vulnerability (freeze risk + no Cancel on large standalone module runs) is real, present in the code today, and unaddressed regardless of whether it's on a roadmap.

</details>

### 2. Batch module runs (vsh_gr, vsh_dn, or any manifest module) attribute their History-panel entry to the globally 'selected' well, not to the wells actually run — can name a well untouched by the run, or misrepresent a multi-well batch as single-well

**Area:** Frontend wiring — dimension D

**Effort:** small

**Where:** src/ui/workspace.ts:363-369 (`onRunComplete: () => { recordProcess("Module", \`Ran ${spec.title}\`, appState.selectedWell.get()?.well_name ?? null); ... }`); src/state.ts:84-91 (`defaultRunWellIds` — pre-ticks the checklist from `multiSelectedWellIds`, a set independent of `selectedWell`); src/processLog.ts:16-17,42 (`ProcessEntry.well` doc: "Well it applied to, when it is well-scoped (null for field-wide/batch actions)")

**Evidence:** moduleDialog.ts's Wells checklist is genuinely multi-select and is pre-ticked from the sidebar's independent multi-selection (`defaultRunWellIds`), not from the single `selectedWell`. Concretely: user has Well-A as the active/selected well (used by other well-following panes) but multi-selects Well-B/C/D in the sidebar and opens the VSH from Gamma Ray pane — the checklist ticks B, C, D. Clicking Run computes and writes VSH_GR/VSH only for B, C, D. `onRunComplete` in workspace.ts then calls `recordProcess("Module", "Ran VSH from Gamma Ray", appState.selectedWell.get()?.well_name ?? null)`, which records well = "Well-A" — a well the run never touched — while B, C, D (the wells actually written) get no History entry naming them. This violates the contract processLog.ts itself documents (batch actions should pass `null`, not a plausible-but-wrong name) and diverges from the sibling pattern in src/ui/multiminDialog.ts:715-719, which passes `applyWells.join(", ")` — the real list of wells the run applied to. vsh_gr and vsh_dn, run through this exact generic "module" dialog wiring, inherit the bug on every multi-well run.

**Suggested fix:** Thread the actual `wellIds` (or well names) run through `callbacks.onRunComplete` from moduleDialog.ts, and have workspace.ts's handler pass that list (or null when it's a genuine multi-well batch) to `recordProcess`, matching multiminDialog.ts's pattern instead of reading `appState.selectedWell`.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read every file cited in the finding at D:\XX. SandiBumi (the actual SandiBumi repo) and could not refute the claim — all the mechanics check out exactly as described:

1. src/ui/objectTree.ts:78-105 — `handleWellClick` confirms Ctrl-click builds `multiSelectedWellIds` explicitly "without moving the active well" (comment at line 80-81); a plain click is the only thing that changes `selectedWellId`/fires `onSelectWell`. So multi-select and the single "active" well are genuinely independent pieces of state, exactly as claimed.

2. src/state.ts:82-91 — `defaultRunWellIds` prioritizes `multiSelectedWellIds` when non-empty, falling back to `selectedWell` only when the multi-selection is empty. This is the pre-tick logic.

3. src/ui/moduleDialog.ts:53-77 — the Wells checklist (`wellBox`/`wellChecks`) is a genuine multi-select checkbox list, pre-ticked via `defaultRunWellIds(wells)` at line 73, independent of `selectedWell`. Lines 265-274 re-confirm the checklist tracks the multi-selection, not the active well, on well-switch.

4. src/ui/moduleDialog.ts:276-332 — on Run, `wellIds` (line 277) is taken strictly from whichever boxes are checked, sent as `well_ids` in the request, and results are computed/written only for those wells. Critically, `callbacks.onRunComplete(outputs)` (line 326) and the callback type itself (`onRunComplete: (outputCurves: string[]) => void`, line 15) pass only the output curve names — never the wellIds that were actually run. There is exactly one definition and one call site of `onRunComplete` in the whole src tree (verified by grep), so there's no alternate path that does pass the well list.

5. src/ui/workspace.ts:363-369 — the only consumer of that callback calls `recordProcess("Module", \`Ran ${spec.title}\`, appState.selectedWell.get()?.well_name ?? null)`, i.e. it has no way to know which wells were run and falls back to the independent, possibly-stale `selectedWell`.

6. src/processLog.ts:16-17 — `ProcessEntry.well` is documented as "null for field-wide/batch actions," which the module-run path violates by supplying a specific (and potentially wrong) well name instead of null or the real list.

7. src/ui/multiminDialog.ts:657,715-719 — the sibling dialog computes `applyWells` from its own checked wells and passes `applyWells.join(", ")` directly into `recordProcess`, proving the correct/available pattern that moduleDialog.ts's generic wiring does not follow.

8. src-tauri/src/modules.rs confirms `vsh_gr` and `vsh_dn` are ordinary per-well modules registered in the generic module list/dispatch with no special single-well restriction, so they go through this exact generic moduleDialog.ts/workspace.ts wiring and inherit the bug on every multi-well run, as claimed.

Every file:line reference in the finding matches the real source, the causal chain (selectedWell vs. multiSelectedWellIds independence → pre-tick → run → lost wellIds across the callback boundary → wrong/misleading recordProcess call) is real and reproducible from the code as written, and the suggested fix direction (thread wellIds through onRunComplete, mirror multiminDialog.ts's applyWells.join pattern) is a valid, non-trivial fix that isn't already implemented elsewhere. I found no mitigating code path (e.g., no logic ties selectedWell to the run, no module-specific single-well restriction) that would refute the finding.

</details>

### 3. vsh_dn's density-neutron crossplot divides by (c − d) with no guard against a degenerate matrix/shale/fluid triangle — a nearby, in-range parameter choice drives VSH_DN to ±Infinity on every sample, unlike vsh_gr and every other singularity-prone module in the same file

**Area:** Domain correctness / Backend singularity handling — dimensions B & C

**Effort:** small

**Where:** src-tauri/src/modules.rs:370-377 (the `a,b,c,d` / `v = (a - b) / (c - d)` block inside `vsh_dn`, no test coverage), contrasted with modules.rs:281 (`vsh_gr`'s own `gr_ma >= gr_sh` guard) and modules.rs:389 (`vsh_dn`'s own GR-divergence branch, which DOES guard `(gr_sh - gr_ma).abs() > 1e-6` a few lines below the unguarded primary division)

**Evidence:** `c - d = (RHO_MA-RHO_FL)(NPHI_FL-NPHI_SH) - (RHO_SH-RHO_FL)(NPHI_FL-NPHI_MA)` is (up to sign) the collinearity determinant of the three crossplot endpoints (matrix, shale, fluid) in RHOB-NPHI space — it is exactly zero whenever those three points fall on one line, and gets numerically large-valued (not just literally zero) whenever they're nearly collinear, i.e. whenever the matrix and shale endpoints sit close together in NPHI-RHOB space (a real condition in silty/low-radioactivity shale settings). Concrete in-range reproduction: RHO_MA=2.65, RHO_SH=2.30, RHO_FL=1.00, NPHI_MA=0.00, NPHI_FL=1.00, NPHI_SH≈0.2121 (all within the manifest's own min/max bounds, modules.rs:332-337) makes c−d≈0, so every sample's raw `VSH_DN` becomes ±Infinity. `limit()` (modules.rs:148-154) clamps the derived `VSH` output to 0/1 via `f64::clamp`, so the flagged/limited output looks sane, but the manifest explicitly exposes `VSH_DN` itself as a real, user-facing 'unlimited' output curve (modules.rs:344) that goes straight into the Curve Catalog, plots, and any equation referencing it directly — the same class of bug the codebase's own `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf` test (modules.rs:2866-2881) explicitly calls out as unacceptable ("otherwise the raw curve poisons catalog min/max and plot autoscale") and that `vsh_gr` guards against for its own analogous singularity (`gr_ma >= gr_sh`). No test in the file exercises this path.

**Suggested fix:** Guard the vsh_dn division the same way vsh_gr and vsh_dn's own GR-check already do: skip the sample (leave MISSING) when `(c - d).abs()` is below a small epsilon, and add a unit test for a near-collinear matrix/shale/fluid triangle asserting VSH_DN/VSH stay finite (or MISSING), not ±Infinity.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual vsh_dn function (modules.rs:351-404) and confirmed the a,b,c,d/v=(a-b)/(c-d) block at lines 370-377 has no guard on (c-d), unlike vsh_gr's gr_ma>=gr_sh guard (line 281) and vsh_dn's own GR-divergence branch guard (line 389). Hand-verified the reproduction numbers (RHO_MA=2.65, RHO_SH=2.30, RHO_FL=1.00, NPHI_MA=0.00, NPHI_FL=1.00, NPHI_SH=0.212121) produce c-d≈0 and correspond to a genuinely collinear matrix/shale/fluid triangle in RHOB-NPHI space, with all values inside the manifest's declared min/max (modules.rs:332-337) and passing moduleDialog.ts's per-parameter-only validation (no cross-field collinearity check). VSH_DN is confirmed exposed as a real unlimited output curve (line 344). Found strong codebase precedent for guarding exactly this class of degenerate-division bug (condflag lines 1108-1114 explicitly citing 'den <= 0' guard with matching test; a gas-density module at 1457-1462 explicitly citing 'condflag precedent'; sw_arch's test at 2866-2881 whose comment matches the finding's quote verbatim), which vsh_dn does not follow. Confirmed via grep that the only vsh_dn test (lines 2609-2659) uses non-degenerate values and does not cover this path. Nothing in the code contradicts or weakens the claim; the finding is accurate and well-evidenced.

</details>

---

## Porosity (phi_den, phi_dn, phi_son, phimax)

### 1. phi_son OPT_CP lack-of-compaction correction is missing the DT_SH>100 us/ft gate — it inflates porosity instead of no-op below the threshold, including at the module's own default

**Area:** domain-correctness

**Effort:** small

**Where:** src-tauri/src/modules.rs:637 (cp computation) and :608 (DT_SH default=90.0); locked in by the test at :2588-2606 (phi_son_wyllie_cp_opt_in_only_scales_wyllie)

**Evidence:** The classical Wyllie/Hilchie lack-of-compaction correction only applies when the shale is undercompacted, i.e. DT_SH > 100 us/ft (328.084 us/m). This gate is explicit in the actual the reference suite phi_son.lls source (mined into this project's own geolog-loglan skill cookbook, references/cookbook/porosity-sonic.md:156-157: "if DT_SH>328.084 us/m (100 us/ft) then PHIE *= 328.084/DT_SH"), and the project's own audit note that recommended shipping this feature says the same thing (AUDIT-2026-07-20.md:504: "Cp ≈ DT_sh/100, classically applied when adjacent shale DT > 100 us/ft"). The shipped code has no such gate: `let cp = if cp_on && !rhg && dt_sh > 0.0 { dt_sh / 100.0 } else { 1.0 };` (modules.rs:637) fires for ANY positive DT_SH. When DT_SH < 100 (normally/over-compacted shale — including the module's own manifest default of 90.0 at modules.rs:608, and any zone override below 100 in a mixed-compaction well), cp = DT_SH/100 < 1, so dividing porosity by cp INCREASES it (e.g. at the shipped default DT_SH=90, cp=0.9, so PHIT is multiplied by 1/0.9 ≈ 1.111 — an 11% inflation). This is the opposite of the correction's purpose: below the undercompaction threshold there is no basis to correct at all (the reference suite's Cp stays 1), yet toggling OPT_CP=ON with the module's out-of-the-box default silently inflates every reported PHIT/PHIE by ~11% instead of leaving them unchanged. The existing unit test (modules.rs:2598-2600) locks in and asserts this +11%-at-DT_SH=90 behavior as correct, so it will not catch a fix; the same false doc claim is repeated in the module's own doc string (modules.rs:598-601), which never mentions a threshold, and in REVIEW.md:353-358.

**Suggested fix:** Add the missing gate: `let cp = if cp_on && !rhg && dt_sh > 100.0 { dt_sh / 100.0 } else { 1.0 };` (unit is us/ft per the manifest, so the threshold is 100, not 328.084). Update the doc string to state the >100 us/ft condition explicitly, and change the existing test to also assert cp=1.0 (no-op) at DT_SH=90 and DT_SH=100, with the +11% inflation case moved to a DT_SH>100 value (e.g. 130).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified against D:\XX. SandiBumi\src-tauri\src\modules.rs directly. Line 637 reads `let cp = if cp_on && !rhg && dt_sh > 0.0 { dt_sh / 100.0 } else { 1.0 };` with no DT_SH>100 (us/ft) gate — matches the finding exactly. Line 608 confirms the manifest default DT_SH=90.0 (units us/ft per the param decl, range 60-150), so cp=0.9 at default, and lines 639-643 show PHIT is divided by cp, meaning cp<1 (i.e. DT_SH<100) inflates porosity by 1/cp (~1.111x at default) instead of no-op — the opposite of the correction's intended purpose. The doc string (596-601) states no threshold. The existing test `phi_son_wyllie_cp_opt_in_only_scales_wyllie` (2588-2606) explicitly asserts PHIT_SON ≈ raw/0.9 at DT_SH=90 as the correct/expected output, locking in the buggy behavior; it never exercises DT_SH>100. Cross-checked the two external citations verbatim: the project's own mined the reference suite cookbook (C:\Users\ARUNIKA\.claude\skills\geolog-loglan\references\cookbook\porosity-sonic.md:156-157) states real the reference suite's rule is "if DT_SH>328.084 us/m (100 us/ft) then PHIE *= 328.084/DT_SH" (i.e. gated, unit-consistent with 100 us/ft since the shipped param is already in us/ft, not us/m — the finding correctly avoids a wrong unit-conversion trap). AUDIT-2026-07-20.md:504 and REVIEW.md:353-358 were both read and confirmed to state/repeat the ungated Cp=DT_SH/100 description with the same +11%-at-DT_SH=90 example, consistent with the finding's claim that prior project docs already knew the threshold conceptually but the shipped code/test/doc-string never encode it. I found no counter-evidence (no hidden gate elsewhere in phi_son, no unit subtlety that neutralizes the gap, no reason the default/test values are unreachable). The finding stands unrefuted.

</details>

### 2. phi_den and phi_dn have zero unit tests, unlike every other porosity sibling (phi_son, phimax) — the VSH>=0.95 shale branch, OPT_PHIEMAX SHALE_REDUCED/MAXIMUM switch, and phi_dn's shale-reduction clamps are all unverified by any test

**Area:** test-coverage

**Effort:** small

**Where:** src-tauri/src/modules.rs:445-489 (phi_den) and :527-585 (phi_dn); confirmed by grepping for phi_den(&ctx / phi_dn(&ctx across the whole repo — zero matches outside the function definitions themselves

**Evidence:** Every call site of phi_den() and phi_dn() in the codebase is the dispatcher (modules.rs:207-208) — there is no `#[test]` anywhere that constructs a ModuleContext and calls phi_den or phi_dn directly (confirmed by grepping src-tauri/src/*.rs for both names; the only other hits are workflow.rs/chain.rs/jobs.rs/montecarlo.rs dispatch-by-name strings, and pipeline_blso_test.rs's #[ignore]'d real-LAS smoke test which runs phi_dn as one step of a 3-4-module chain against live field data — not a targeted edge-case unit test). By contrast, phi_son has a dedicated Cp test (modules.rs:2588) and phimax has five (modules.rs:2734-2828, covering constant/linear/athy modes, [0,1] clamping, and partial-NaN TVDSS pass-through) — the project clearly treats this class of test as standard practice for the porosity family, and this audit's own dimension guidance calls singularity/edge-case coverage "the single most repeated defect class in this codebase." Concretely unverified: (1) the VSH>=0.95 shale short-circuit in both modules (phie=0, phit=phit_sh) — never exercised by an assertion; (2) OPT_PHIEMAX's SHALE_REDUCED vs MAXIMUM branch (phie_lim = phie_max*(1-v) vs flat phie_max) — no test distinguishes the two; (3) phi_dn's shale-reduced RHOB/NPHI clamps (`limit((r - v*rho_sh)/(1-v), 1.95, 3.0)` and the NPHI analog) at their range boundaries; (4) OPT_XPLOT's GAS_RMS vs AVERAGE combination formula, never asserted against a hand-computed value; (5) basic all-missing-input propagation for these two specific modules. A regression in any of these (e.g. a future refactor of the shared phit_sh_at() or the >=0.95 threshold) would ship silently.

**Suggested fix:** Add unit tests for phi_den and phi_dn mirroring phimax's coverage style: one hand-computed happy-path value per formula (matching the phi_den/phi_dn worked examples in the geolog-loglan cookbook if convenient), one at VSH=0.95 exactly and VSH=1.0 (assert phie=0/phit=phit_sh), one comparing OPT_PHIEMAX=SHALE_REDUCED vs MAXIMUM at the same VSH, and one comparing OPT_XPLOT=AVERAGE vs GAS_RMS at the same PHID/NPHI pair.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against D:\XX. SandiBumi\src-tauri\src\modules.rs (the Arshilla/SandiBumi repo). Line numbers check out exactly: phi_den spans 445-489, phi_dn spans 527-585, dispatcher lines 207-208 are the only call sites ("phi_den" => Ok(phi_den(ctx)), "phi_dn" => Ok(phi_dn(ctx))).

Grep evidence reproduced independently:
- phi_den( appears in exactly 2 places in the whole repo: the dispatcher (line 207) and the function definition (line 445). Zero test call sites -- not even in the ignored integration smoke test.
- phi_dn (whole-word) appears in modules.rs (def/dispatch), chain.rs, jobs.rs, montecarlo.rs, workflow.rs, and pipeline_blso_test.rs -- all dispatch-by-name strings or integration-style runs, never a direct phi_dn(&ctx) unit-test call.
- pipeline_blso_test.rs confirmed #[ignore]'d (checked lines 61-63, 272-274: #[test] immediately followed by #[ignore]).
- Read the actual function bodies: both contain the claimed logic -- the v >= 0.95 shale short-circuit (phi_den line 465, phi_dn line 550) setting phie=0/phit=phit_sh; the OPT_PHIEMAX SHALE_REDUCED-vs-MAXIMUM branch (phie_lim = if shale_reduced { phie_max*(1-v) } else { phie_max }, lines 475 and 571); phi_dn's shale-reduced clamps limit((r - v*rho_sh)/(1-v), 1.95, 3.0) and the NPHI analog (lines 559-560); and OPT_XPLOT's GAS_RMS vs AVERAGE formula (lines 563-567) -- none of these are touched by any assertion anywhere.
- Checked the integration-style tests that DO reference phi_dn (chain.rs:271-302, workflow.rs:1617-1660, montecarlo.rs ~558-578): all of them only assert 'no error' / 'finite count > 0' on a single fixed clean-sand input with default option values -- they never hit VSH>=0.95, never compare OPT_PHIEMAX modes, never probe the RHOB/NPHI clamp boundaries. db.rs:1448's 'phi_den' hit is an unrelated string label inside a DB-write-batching test, not a call to the function.
- Confirmed by contrast: phi_son has its dedicated Cp test at modules.rs:2588 (phi_son_wyllie_cp_opt_in_only_scales_wyllie), and phimax has five dedicated tests at lines 2734, 2756, 2774, 2792, 2818 covering constant/linear/athy modes, [0,1] clamping, and partial-NaN TVDSS -- matching the claim's line ranges and count exactly.

Every concrete sub-claim in the finding (line ranges, grep results, the #[ignore] attribute, the sibling test counts/locations, and the specific unverified code paths) checks out against the real file. No test anywhere in the repo exercises phi_den or phi_dn as a targeted unit test. The finding stands unrefuted.

</details>

---

## Prep corrections (ftemp_grad, badhole, condflag, nphimat, gr_hole_corr, nphi_env_corr, rhob_hole_corr)

### 1. nphi_env_corr's FTEMP input is a plain log_in, not computed_only — a raw-imported degF FTEMP can silently masquerade as degC

**Area:** Backend domain correctness / unit-contract safety

**Effort:** small

**Where:** src-tauri/src/modules.rs — nphi_env_corr_spec (line 1594: `log_in("FTEMP", "Formation temperature", "degC", "FTEMP", false)`), consumed in fn nphi_env_corr (lines 1600-1617, esp. `corr += ctx.p("K_TEMP", i) * (ft - ctx.p("T_REF", i))`)

**Evidence:** The codebase already has a named mechanism for exactly this failure mode: ArgSpec.computed_only / log_in_computed, whose doc comment (modules.rs lines 43-47) says it exists 'for unit-contract inputs like FTEMP/FPRESS where a raw curve with the same mnemonic (a commercial LAS export's degF FTEMP) would silently masquerade as the degC/psi curve the module assumes.' gascorr — added later and needing the identical degC FTEMP — uses `log_in_computed("FTEMP", ...)` (line 1374) specifically to force resolution through precalc's computed output only. nphi_env_corr needs the same degC contract (K_TEMP is 'v/v per degC', T_REF default 24 degC) but still uses a plain log_in. Via fetch_curve_frame's precedence (equations.rs 316-357), computed_curves is tried before the generic/raw store for a non-standard-six name like FTEMP, so the hole only opens on a well that has NOT yet had ftemp_grad/precalc run but does carry a raw 'FTEMP' mnemonic from LAS import (a scenario the project's own gascorr doc and REVIEW.md line 605-606 confirm is real, not hypothetical). In that case nphi_env_corr silently applies the temperature term to a possibly-degF value as if it were degC, producing a systematically wrong NPHI_EC with no error or warning.

**Suggested fix:** Change modules.rs line 1594 to `log_in_computed("FTEMP", "Formation temperature (precalc/ftemp_grad)", "degC", "FTEMP", false)`, matching gascorr's pattern, and add a test for 'raw FTEMP present, no computed FTEMP yet' resolving to no correction rather than a wrong one.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently read the actual files at D:\XX. SandiBumi\src-tauri\src\{modules.rs,equations.rs,workflow.rs} and REVIEW.md and could not refute the finding — every cited line and the causal mechanism check out exactly.

Verified facts:
1. modules.rs:43-47 — ArgSpec.computed_only doc comment reads exactly as quoted, naming FTEMP/FPRESS as the motivating unit-contract example.
2. modules.rs:1594 — `nphi_env_corr_spec()` declares `log_in("FTEMP", "Formation temperature", "degC", "FTEMP", false)` — a plain LogIn, computed_only defaults to false (confirmed via the `log_in()` constructor at lines 89-102).
3. modules.rs:1600-1617 — `fn nphi_env_corr` uses `ctx.log("FTEMP")` directly and applies `corr += ctx.p("K_TEMP", i) * (ft - ctx.p("T_REF", i))` with no unit check whatsoever. K_TEMP is declared "v/v per degC" (line 1589) and T_REF defaults to 24.0 degC (line 1590) — exactly as claimed.
4. modules.rs:1374-1375 — `gascorr_spec()` uses `log_in_computed("FTEMP", ...)` / `log_in_computed("FPRESS", ...)`, explicitly commented "a raw import named FTEMP/FPRESS may be degF/kPa — precalc outputs only" (lines 1372-1373). This is the real, already-existing precedent the finding cites.
5. equations.rs:316-357 (`fetch_curve_frame_from_set`'s inner logic) — confirmed the precedence: for a non-standard-six name like FTEMP, it first tries `fetch_computed_curves_batch` (computed_curves store) and only if that yields nothing usable falls back to `fetch_generic_curve_aligned` (line 353), which queries the RAW `curve_meta`/`curve_samples` store by mnemonic-or-family (lines 428-459) with **no unit awareness at all**.
6. workflow.rs:196-225 — confirmed the actual enforcement point: plain LogIn args (incl. nphi_env_corr's FTEMP) are populated straight from the `fetch_curve_frame_from_set` columns (raw-fallback-eligible); only args where `a.computed_only` is true (line 208) get re-resolved via `fetch_computed_only_aligned`, which per its own doc (equations.rs:462-466) and code (never touches curve_meta/curve_samples) explicitly excludes the RAW store. nphi_env_corr's FTEMP is not in that filtered set, so it never gets the safe re-resolution gascorr's FTEMP/FPRESS get.
7. Both `ftemp_grad` (line 761) and `precalc` (line 836) write their computed output under the exact mnemonic `"FTEMP"` in degC — the same name a raw LAS import would plausibly use, maximizing collision risk.
8. REVIEW.md:605-606 confirms the team already treats "the well's LAS carries its own raw FTEMP/FPRESS curves" as a real, non-hypothetical scenario worth guarding against (stated in the gascorr QC checklist).
9. Test coverage check: modules.rs:3443-3447 has an explicit regression test (`gascorr_spec_shape`) asserting `ftemp.computed_only` is true for gascorr. No analogous assertion exists for `nphi_env_corr_spec`, and the existing `nphi_env_corr` unit test (~line 3423-3431) only exercises the compute function with FTEMP injected directly into the context — it never exercises the resolution-precedence path, so the gap is real and untested.
10. No mitigating control exists elsewhere: moduleDialog.ts has no reference to computed_only (no UI-level warning/restriction), and workflow.rs has no module-ordering/dependency validation that would force ftemp_grad/precalc to run before nphi_env_corr.

Net: the finding's every factual claim (line numbers, code content, mechanism, precedent, doc quotes) matches the real codebase precisely, and the exploit path is genuinely reachable (no other guard rail blocks it). The suggested fix (swap to `log_in_computed`, matching gascorr's established pattern) is correct and low-risk — `log_in_computed` preserves the existing `required: false` flag, so the documented "without FTEMP only the salinity term applies" fallback behavior is unchanged; it merely closes the RAW-store fallback. I could not construct a valid refutation.

</details>

### 2. ftemp_grad's BHT mode divides by TD_BHT with no degenerate-value guard — a zone override of TD_BHT <= 0 yields ±Infinity that evades every is_missing()/is_nan() check downstream

**Area:** Backend singularity handling

**Effort:** small

**Where:** src-tauri/src/modules.rs, fn ftemp_grad lines 766-786, specifically line 779: `tsurf + (bht - tsurf) * d / td`. TD_BHT is declared as a Param (line 760, range 100-10000) resolved per-sample by workflow.rs's resolve_param_arrays (lines 47-88), which applies zone_params.value_num verbatim with no min/max clamp.

**Evidence:** resolve_param_arrays has no range check at all — it just does `arr[i] = v as f64` for any zone override value. src/ui/zonesDialog.ts's zone-parameter entry (lines 163-182) only checks `!Number.isNaN(value)` before calling setZoneParam; there is no range validation on the free-text PARAM/VALUE pair, so a zone override can set TD_BHT to 0 (or negative) even though the module dialog's own numeric input enforces 100-10000. This is the exact 'zone overrides bypass dialog range checks' hazard the codebase has already had to guard against twice in sibling Prep modules added later: condflag's `if den <= 0.0 { continue; }` (around line 1112, comment: 'Zone overrides bypass dialog range checks... a degenerate matrix/fluid pair... can still arrive') and gascorr's `if !(rma - rfl > 0.0) || rb <= rfl { continue; }` (around line 1460). ftemp_grad has no equivalent guard on TD_BHT, and has zero unit tests at all (grep for 'fn ftemp_grad' / '#[test]' shows only the spec/dispatch, no test function ever exercises it). f32::INFINITY.is_nan() is false, so the is_missing()-based MISSING convention this entire codebase relies on does not catch the resulting value — it silently writes as a finite-looking (actually infinite) FTEMP that then feeds nphi_env_corr, gascorr's computed_only FTEMP requirement, and any other FTEMP consumer.

**Suggested fix:** Guard the BHT branch: `if td <= 0.0 { continue; }` (or skip to MISSING) before computing the interpolation, mirroring condflag/gascorr's precedent. Add a unit test for TD_BHT <= 0 (currently the module has none at all).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual files at D:\XX. SandiBumi and could not refute the finding — every specific claim checks out against the real code.

1. `src-tauri/src/modules.rs` fn `ftemp_grad` (lines 766-786) confirmed verbatim: line 779 is exactly `tsurf + (bht - tsurf) * d / td` with `td = ctx.p("TD_BHT", i)` and no check on `td` before the division. `TD_BHT` is declared at line 760 as `param("TD_BHT", ..., 100.0, 100.0, 10000.0)` — dialog range 100-10000, matching the claim.

2. `is_missing` (line 158-160) is literally `v.is_nan()` — confirmed it does NOT catch infinity (`f32::INFINITY.is_nan() == false` is basic IEEE-754 fact).

3. `resolve_param_arrays` in `src-tauri/src/workflow.rs` (lines 47-88) confirmed to have zero range/clamp logic — zone overrides do `arr[i] = v as f64` (line 80) verbatim, no min/max check against the module spec at all. `ModuleContext::p()` (modules.rs lines 138-140) just reads straight from this array, so `ctx.p("TD_BHT", i)` can legitimately be 0 or negative if a zone override sets it so.

4. `src/ui/zonesDialog.ts` zone-parameter entry (lines 174-182) confirmed: only `!Number.isNaN(value)` is checked before `setZoneParam(...)` — free-text PARAM name, no range tied to the module's arg spec, unlike `src/ui/moduleDialog.ts` (lines 283-289) which does enforce `arg.min`/`arg.max` on the normal dialog input. This confirms the "zone overrides bypass dialog range checks" bypass path is real.

5. Sibling-module precedent confirmed exactly, down to line numbers: `condflag` at line 1112 has `if den <= 0.0 { continue; }` guarding `RHO_MA - RHO_FL`, with the comment "Zone overrides bypass dialog range checks..." — exact match. `gascorr` at line 1460 has `if !(rma - rfl > 0.0) || rb <= rfl { continue; }` — exact match. `ftemp_grad` has no analogous guard on `TD_BHT`.

6. Downstream evasion confirmed by tracing FTEMP consumers: `nphi_env_corr` (line 1611) only checks `!is_missing(ft)` before using FTEMP in `corr += K_TEMP * (ft - T_REF)` — an infinite FTEMP passes this check and silently produces an infinite NPHI_EC. `gascorr`'s `gas_density_gcc` (line 1410-1424) guards `!(temp_c > -273.15)` which is FALSE (doesn't trigger) for `temp_c = +Infinity`, so it also passes through, ultimately producing a silently-wrong (though not infinite) GASDEN=0 rather than an error — a second, independently-verified silent-corruption path.

7. Confirmed zero test coverage: searched the entire `mod tests` block in modules.rs (45 `#[test]` functions) and none reference `ftemp_grad`.

One nuance worth flagging: the finding's phrasing "TD_BHT <= 0 yields ±Infinity" is slightly imprecise — strictly, only TD_BHT == 0 produces ±Infinity (or NaN, which WOULD be caught, if depth or bht-tsurf is also 0); negative TD_BHT values instead produce a finite-but-physically-wrong FTEMP that doesn't trip is_missing either, which is arguably just as dangerous. This doesn't undermine the core defect — TD_BHT = 0 is a very plausible zone-override value (users disabling/zeroing a param) and reproducibly causes the exact unguarded-infinity hazard described, matching the fix pattern already established twice in condflag/gascorr. The finding is confirmed as accurate and well-evidenced.

</details>

---

## Prep statistical (gr_normalize, log_predict)

### 1. log_predict's MAX_RAW/repaired-synthetic value is unconditionally re-blanked at masked (washout) depths by workflow.rs's output-masking step — contradicts the project's own documented fix and defeats the module's core purpose

**Area:** Backend correctness (B) / domain intent (C) — MASK on inputs vs outputs

**Effort:** small-medium

**Where:** src-tauri/src/workflow.rs lines 251-278 (run_workflow_module_into: input-masking at 253-263, module run at 265-266, output-masking at 268-278, both unconditional over every arg/output with no req.module branch); src-tauri/src/modules.rs lines 2417-2530 (log_predict, OPT_COMBINE match at 2521-2526)

**Evidence:** workflow.rs blanks flagged (MASK==1) samples in the module INPUTS before the run (251-263) — this correctly makes log_predict's TARGET NaN at washed-out depths, so inside log_predict's own OPT_COMBINE match (modules.rs:2521-2526) `raw` is missing there, so MAX_RAW/FILL_MISSING both fall through to `_ => syn as f32` and the module genuinely computes a usable repaired synthetic at that depth. But immediately after `run_module` returns, workflow.rs's output-masking step (268-278) iterates every output curve and NaNs any sample where mask==1, with no exception for log_predict or for SYNTHETIC/MAX_RAW-computed fill values — so that same repaired value is instantly overwritten back to NaN before it's written to computed_curves. This directly contradicts REVIEW.md's own claim (lines 98-104): 'For log_predict the repaired synthetic now survives *inside* the masked (washout) interval it exists to fill, instead of being blanked there,' and ROADMAP.md lines 483-485 ('the masked log_predict synthetic survives inside the washout'). Grepping workflow.rs for 'log_predict'/'SYN' turns up only the explanatory comment at line 230 — no code path treats log_predict's output differently. The only workflow.rs integration test for this MASK behavior, `mask_excludes_flagged_samples_from_gr_normalize_percentiles` (line 1242), exercises gr_normalize only; log_predict's two unit tests (modules.rs 3658-3696) call `log_predict(&ctx)` directly and never go through workflow.rs's masking pipeline, so this regression/gap has no test coverage anywhere. Net effect: running Log Predict with the recommended Mask (module's own doc: 'Mask the run to good-hole intervals so bad samples never train the model') produces a `<TARGET>_SYN` curve that is NaN at exactly the bad-hole/washout depths it exists to fill — the module's one deliverable use case (patching RHOB/NPHI/DT gaps in washout intervals) is silently defeated whenever used as documented.

**Suggested fix:** Give log_predict (and any future fill-type module) an explicit way to opt its own output out of the blanket output-masking step — e.g. a per-module flag or an opts key like OUTPUT_MASK_EXEMPT — so the repaired synthetic at masked depths is preserved for OPT_COMBINE in {SYNTHETIC, MAX_RAW, FILL_MISSING}, while still masking the TRAINING inputs as today. Add a workflow.rs integration test mirroring mask_excludes_flagged_samples_from_gr_normalize_percentiles but for log_predict + MAX_RAW, asserting the output at the masked/washout depth is a finite repaired value, not NaN.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the real source — I could not refute the finding; every specific claim checks out.

1. workflow.rs:253-263 (input-masking) — confirmed. For each `arg_name` in `log_args`, samples where `mask==1.0` are set to NaN in `logs`, unconditionally, before `ModuleContext` is built. `TARGET` is one such log arg for log_predict, so at masked depths `target[i]` is NaN going into the module.

2. modules.rs:2417-2530 (log_predict) — confirmed. `raw = target[i] as f64` (line 2521) is read directly from the (now masked) TARGET input. The OPT_COMBINE match (2522-2526) is:
   - `"FILL_MISSING" if !is_missing(raw) => raw as f32`
   - `"MAX_RAW" if !is_missing(raw) => raw.max(syn) as f32`
   - `_ => syn as f32` (catch-all, hit whenever raw is missing/NaN — i.e., exactly at masked depths)
   So at every masked/washout depth, regardless of OPT_COMBINE mode, the module falls through to the pure KNN-repaired `syn` value — a genuine, non-NaN, usable repair. This matches the finding precisely, and also matches the module's own doc comment (2399-2400) and the leave-one-out training design (2435-2438), which exists specifically so masked depths get real, non-self-echoed predictions.

3. workflow.rs:265-278 — confirmed. `run_module` is called once (line 266), and immediately after, lines 270-278 iterate `outputs.values_mut()` (every output curve, log_predict's `<TARGET>_SYN` included) and set any sample where `mask==1.0` back to NaN. There is no `req.module` check, no opt-in/opt-out key, no exemption of any kind — I grepped the whole file for `OPT_COMBINE`, `OUTPUT_MASK`, `exempt`, `SYNTHETIC`, `MAX_RAW`, `FILL_MISSING`, `req.module` and found nothing that branches this step by module. `run_module` is called from exactly one place in the file (confirmed via grep), so there is no alternate chain-path that skips this masking.

4. Net effect confirmed: the repaired synthetic that log_predict computes at masked depths is unconditionally overwritten back to NaN one step later, in the same function, before anything is written to `computed_curves`.

5. Documentation contradiction confirmed verbatim: REVIEW.md lines 98-104 states "For log_predict the repaired synthetic now survives *inside* the masked (washout) interval it exists to fill, instead of being blanked there" and ROADMAP.md lines 483-485 states "the masked log_predict synthetic survives inside the washout" — both describe behavior the code does not actually deliver, given the unconditional re-blanking at 268-278.

6. Test-coverage gap confirmed: workflow.rs's only masking-behavior test is `mask_excludes_flagged_samples_from_gr_normalize_percentiles` (line 1242), which grep confirms is gr_normalize-only — no workflow.rs test exercises log_predict through the masking pipeline. modules.rs's two log_predict unit tests (`log_predict_learns_association_and_fills_gaps` at 3659 and `log_predict_max_raw_keeps_raw_where_higher` at 3679) call `log_predict(&ctx)` directly, bypassing workflow.rs's input/output masking entirely, so they cannot catch this regression.

Files examined: D:\XX. SandiBumi\src-tauri\src\workflow.rs (lines 92-320), D:\XX. SandiBumi\src-tauri\src\modules.rs (lines 2391-2530, 3655-3700), D:\XX. SandiBumi\REVIEW.md (lines 70-120), D:\XX. SandiBumi\ROADMAP.md (lines 460-500). Every line reference and quote in the finding matched the actual file contents I read. I found no conditional, branch, or alternate code path anywhere in workflow.rs that would exempt log_predict's output from the blanket output-masking step, so I cannot refute the finding — it stands confirmed.

</details>

### 2. moduleDialog.ts's persistent-pane data refresh has no race-guard generation counter — a slow-resolving refresh can overwrite a fresher one with stale Wells/curve/mask data

**Area:** Frontend wiring (D) — async race guard on reload

**Effort:** small

**Where:** src/ui/moduleDialog.ts lines 235-264 (refreshData() and its dataVersion subscription); contrast with src/ui/histogramPanel.ts lines 503-521 (reloadGen pattern)

**Evidence:** refreshData() (moduleDialog.ts:235-256) does `await Promise.all([listWells()..., listCurveCatalog()])` then unconditionally overwrites `wells`/`catalog`/`curveNames` and rebuilds the well checklist and log/mask dropdowns, guarded only by a `disposed` flag — there is no monotonic token comparable to histogramPanel.ts's `reloadGen` (captured as `const gen = ++reloadGen` before the await, checked as `if (gen !== reloadGen) return` after — histogramPanel.ts:506,512,517,521) to detect and drop a reload that a newer one has since superseded. Because this pane is explicitly designed to stay open long-term ('The pane is persistent', file header comment) and `appState.dataVersion` is bumped by many unrelated operations across the app (imports, edits, this pane's own runs via onRunComplete in workspace.ts:365-368), two refreshData() calls can overlap; if the earlier-triggered call's network round-trip is slower, it resolves after the newer one and clobbers the fresher wells/curve list — e.g. a well imported or a curve just computed by a second, faster-resolving trigger can transiently vanish from the Wells checklist / log-input / mask dropdowns until the next dataVersion bump corrects it.

**Suggested fix:** Add the same `reloadGen`-style monotonic counter used in histogramPanel.ts: capture it before the `Promise.all`, and bail out of applying the fetched results if a newer refresh has started in the meantime.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the actual source in D:\XX. SandiBumi (the SandiBumi repo).

1. src/ui/moduleDialog.ts:235-256 — `refreshData()` does exactly `await Promise.all([listWells().then(filterByActiveGroup), listCurveCatalog()])`, then unconditionally reassigns `wells = freshWells; catalog = freshCatalog; curveNames = catalog.map(...)`, calls `rebuildWellChecklist(checkedIds)`, refills every `logSelects` dropdown, and calls `rebuildMaskOptions(...)`. The only post-await guard is `if (disposed) return;` (line 241). I grepped the whole file for any generation/token/epoch counter (`Gen\b|generation|token|reloadGen|refreshToken|epoch`) — zero matches. `disposed` is set only once, in the pane's `dispose()` at lines 336-339, i.e. it guards against the pane being closed, not against a newer in-flight refresh superseding an older one. So the finding's core technical claim — "no monotonic token, guarded only by `disposed`" — is accurate.

2. src/ui/moduleDialog.ts:258-264 — the `dataVersion.subscribe` callback: after the initial no-op `dataPrimed` skip, every subsequent bump calls `void refreshData()` with no de-duplication or in-flight check, so two bumps close together do start two overlapping `refreshData()` calls.

3. src/ui/histogramPanel.ts:503-521 — the contrast case is exactly as cited: `let reloadGen = 0` (506), `const gen = ++reloadGen` captured before the await (512), and `if (gen !== reloadGen) return` checked both after the success path (517) and the catch path (521) — a real, working example of the guard the finding says is missing from moduleDialog.ts. This is clearly a deliberate, already-solved pattern in this same codebase for the identical class of race.

4. src/ui/workspace.ts:365-368 — `onRunComplete` (the module pane's own run completion) calls `this.notifyDataChanged()`, and a repo-wide grep shows 11 other call sites (mostly in ribbon.ts — imports, edits, etc.) also invoking `notifyDataChanged()`, which (src/state.ts:94) does `appState.dataVersion.set(appState.dataVersion.get() + 1)`. So the finding's claim that dataVersion is bumped by many unrelated operations, including the pane's own runs, is confirmed verbatim down to the cited line numbers.

5. `listWells`/`listCurveCatalog` (src/ipc.ts:40-41, 59-60) are raw Tauri `invoke()` calls with no frontend single-flight/serialization wrapper, so overlapping calls have no ordering guarantee — out-of-order resolution (a later-triggered refresh finishing before an earlier-triggered one) is a realistic failure mode, not just theoretical, especially since one likely trigger is this pane's own module run (log_predict/gr_normalize backend work) racing against a concurrent import or another module's completion.

Every specific claim in the finding — file, line ranges, function names, the exact contrasted `reloadGen` mechanism in histogramPanel.ts, and the causal chain via workspace.ts/state.ts — checks out against the real code. I found no mitigating mechanism elsewhere (no debounce, no request cancellation, no dedup) that would neutralize the race. I could not refute the finding; it is CONFIRMED.

</details>

### 3. run_workflow_module — the Tauri command behind every module Run click, including gr_normalize/log_predict — is still a synchronous main-thread-blocking command, and doesn't register with the Processing panel

**Area:** Backend architecture (B) / UI-UX (E) — long-op / Processing-panel registration

**Effort:** medium (pattern already established for run_workflow_chain: DbState is already Arc<Mutex<Connection>>)

**Where:** src-tauri/src/lib.rs lines 528-533 (`run_workflow_module` command, plain `#[tauri::command]`, no async/thread::spawn); src-tauri/src/workflow.rs lines 92-94 (`run_workflow_module` wrapper always passes `progress: None`); src/ui/moduleDialog.ts lines 301-313 (`runWorkflowModule(req)` awaited directly, no polling/progress UI); src/ipc.ts lines 412-413

**Evidence:** AUDIT-2026-07-20.md (lines 69-75) already diagnosed and named `run_workflow_module` (alongside run_workflow_chain, run_equation, run_ml, run_multimin, run_monte_carlo, import_las_files, export_report_batch) as a plain sync command that 'executes inline on the main/event-loop thread' in Tauri v2, freezing the app for the run's duration. REVIEW.md's 2026-07-21 fix wave converted only `run_workflow_chain` — moved to `std::thread::spawn` (lib.rs:868) with its own comment at lib.rs:860-867 reaffirming 'a sync #[tauri::command] runs on the main event-loop thread' — and its explicitly stated follow-up queue is 'import, dashboard, multimin, Monte Carlo and equations' (REVIEW.md line 35-36); `run_workflow_module` is not in that list. I confirmed the command is still an unconverted plain sync fn today (lib.rs:530-533: `fn run_workflow_module(db: tauri::State<DbState>, req: ...) -> Vec<...> { workflow::run_workflow_module(&db.0, &req) }`), and moduleDialog.ts's Run button (lines 301-313) calls it directly via a single awaited invoke with no job id / progress poll, so it also never appears in the universal Processing panel (jobs.rs) the way chain runs now do. moduleDialog.ts's Wells checklist is not capped to one well — `defaultRunWellIds` (state.ts:84-91) pre-ticks the full Wells & Tops multi-selection or an entire active well group, and gr_normalize's own doc explicitly frames it as a whole-field operation ('QC across wells with a GRN histogram overlay — the P3/P97 of every normalized well should coincide') — so a gr_normalize or log_predict run across the same 500+-well scale that motivated the chain fix hits the identical unfixed freeze path.

**Suggested fix:** Apply the same std::thread::spawn + jobs::register pattern used for run_workflow_chain to run_workflow_module (or route single-module runs through the chain machinery as a one-step chain), and have moduleDialog.ts poll/attach to the Processing panel instead of awaiting the invoke directly.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified every cited location in D:\XX. SandiBumi against the live source; nothing refutes the finding.

1. src-tauri/src/lib.rs:530-533 — `run_workflow_module` is a plain `#[tauri::command] fn run_workflow_module(db, req) -> Vec<ModuleRunResult> { workflow::run_workflow_module(&db.0, &req) }`. No async, no thread::spawn, no job registry state. Confirmed sync.

2. src-tauri/src/workflow.rs:92-94 — `pub fn run_workflow_module(db, req) { run_workflow_module_into(db, req, None, None, None) }` — the third arg is `progress: Option<&JobHandle>`, always None on this path. The underlying run_workflow_module_into genuinely supports progress (used by chain.rs:192 with Some(job)), but the standalone command never passes it — confirmed.

3. Contrast with run_workflow_chain (lib.rs:813-874): this command IS now async-shaped — it registers a job via jobs::register (line 852) and executes the actual work inside std::thread::spawn (line 862), with an explicit comment explaining why. Project-wide grep shows this is the ONLY command touching jobs::register/JobHandle/thread::spawn — confirming run_workflow_module remains unconverted.

4. REVIEW.md's 2026-07-21 entry states verbatim: "This is the first of the async conversions — import, dashboard, multimin, Monte Carlo and equations will follow the same pattern." run_workflow_module (module runs) is not in that list, and no other REVIEW.md entry converts it.

5. AUDIT-2026-07-20.md's threading finding explicitly names run_workflow_module (at its then-line :440) alongside run_workflow_chain, run_equation, run_ml, run_multimin, run_monte_carlo, import_las_files, export_report_batch as sync commands blocking the main/event-loop thread — matching the finding's characterization.

6. src/ui/moduleDialog.ts:313 — `const results = await runWorkflowModule(req);` inside the Run click handler, a single direct await with no job id, no polling loop, no Processing-panel attachment — confirmed by reading lines 276-330.

7. src/ipc.ts:411-413 — runWorkflowModule is `return invoke<ModuleRunResult[]>("run_workflow_module", { req });`, a bare one-shot invoke returning the final result array, no job_id in the request or response shape.

8. state.ts:84-91 defaultRunWellIds pre-ticks the full multi-selection (appState.multiSelectedWellIds) when non-empty, else falls back to only the single active well — so the Wells checklist is not capped to one well, confirming runs can span many/all wells.

9. gr_normalize's doc string in modules.rs:2342 quotes verbatim: "QC across wells with a GRN histogram overlay — the P3/P97 of every normalized well should coincide" — exact match. Both gr_normalize and log_predict (modules.rs:2333-2460) are ordinary entries in the generic module dispatch (matched by name string in the same match statement used by every other module) with zero special-case async/job handling anywhere in lib.rs, ipc.ts, or moduleDialog.ts (grepped, no hits) — so they ride the identical unfixed synchronous path as every other module.

Every claim in the finding — the specific line numbers, the always-None progress wiring, the direct-await UI call, the absent Processing-panel registration, the REVIEW.md follow-up-queue omission, and the multi-well exposure via defaultRunWellIds plus the gr_normalize doc quote — checks out against the actual repository state as of today. I found no mitigating code path (no hidden async wrapper, no alternate job-aware command for modules) that would refute or weaken it.

</details>

### 4. Substrate cross-reference notes for gr_normalize / log_predict / moduleDialog.ts (not new findings — confirming applicability of already-tracked issues per audit instructions)

**Area:** Cross-function (F) / substrate cross-reference

**Effort:** n/a — already tracked elsewhere; listed here only for applicability per audit instructions

**Where:** src/ui/moduleDialog.ts (whole file); src/ui/workspace.ts lines 363-369; src-tauri/src/workflow.rs line 147

**Evidence:** (1) well-group-scoping-sweep APPLIES: moduleDialog.ts subscribes to appState.dataVersion (line 258) and appState.selectedWell (line 265) but never appState.wellGroupsVersion (grep confirms no reference), so switching the active well group while a gr_normalize/log_predict pane is already open does not re-scope its Wells checklist — matches the already-confirmed substrate finding, no new mechanism found. (2) db-write-versioning-discipline does NOT meaningfully apply here: unlike EquationDef.output_curve (free-text, only .trim()'d), these two modules' output curve names are forced/fixed — gr_normalize always writes the literal 'GRN', and log_predict's name is `format!("{src}_SYN")` where `src` comes from workflow.rs:147 `mnemonic.trim().to_uppercase()` — so there is no re-casing vector into the DELETE-by-exact-string-vs-read-case-insensitive mismatch for these two tools specifically. (3) The already-documented module-run history mis-attribution (AUDIT-2026-07-20.md: batch runs recorded as one entry against the currently selected well, firing only on partial success) is still present and unfixed at its new location post-refactor: workspace.ts:365-368 `recordProcess("Module", "Ran ${spec.title}", appState.selectedWell.get()?.well_name ?? null)` records the gr_normalize/log_predict/any-module run against whichever well happens to be selected, not the actual (possibly multi-well, possibly non-overlapping) batch that moduleDialog.ts ran — and ROADMAP.md's Polish-3 recordProcess sweep (#124) added coverage elsewhere (equation/ML/MC/workflow/log-set/zone/tops/imports/cutoffs/map) without touching this call site.

**Suggested fix:** No new action beyond what's already tracked under well-group-scoping-sweep and the AUDIT-2026-07-20.md history-attribution finding.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every claim against the actual source in D:\XX. SandiBumi (the SandiBumi repo):\n\n1. src/ui/moduleDialog.ts subscribes only to appState.dataVersion (line 258) and appState.selectedWell (line 265) - confirmed by direct read. A repo-wide grep for wellGroupsVersion shows it's referenced only in state.ts (definition), mapPanel.ts, and workspace.ts:752 (which refreshes only the Wells & Tops object-tree pane, not the generic \"module\" pane built at workspace.ts:355-372). So switching the active well group indeed does not re-scope an already-open gr_normalize/log_predict pane's Wells checklist - the well-group-scoping-sweep class of issue genuinely applies here with no new mechanism.\n\n2. gr_normalize (modules.rs:2356-2383) unconditionally returns HashMap::from([(\"GRN\".to_string(), out)]) - a hardcoded literal. log_predict (modules.rs:2417-2421) derives out_name from ctx.o(\"__IN_TARGET\"), which workflow.rs:147 sets via `opts.insert(format!(\"__IN_{arg_name}\"), mnemonic.trim().to_uppercase())` - exact text match. A grep of src/ui/ for these two module names returns zero hits, confirming there's no bespoke frontend and no free-text output-curve field for either (unlike EquationDef.output_curve, which is user-typed and only .trim()'d). This structurally rules out the DELETE-exact-vs-read-case-insensitive mismatch vector for these two modules specifically, exactly as claimed.\n\n3. workspace.ts:363-369 reads exactly as cited: `onRunComplete: () => { recordProcess(\"Module\", \\`Ran ${spec.title}\\`, appState.selectedWell.get()?.well_name ?? null); this.notifyDataChanged(); }`. AUDIT-2026-07-20.md documents the identical mis-attribution issue at ribbon.ts:419 (pre-refactor); a grep of the current ribbon.ts shows no module-run recordProcess call remains there, confirming it moved to workspace.ts during the dockview refactor. ROADMAP.md's Polish-3 (#124) entry explicitly enumerates what its recordProcess sweep covered (equation/chain/ML/MC, log-set restore/delete, zones, tops, DLIS/deviation/SCAL/core imports, cutoffs, map polygon->group) and module runs are absent from that list, matching the finding's claim that #124 never touched this call site.\n\nAll cited line numbers, verbatim code, and cross-document references (AUDIT-2026-07-20.md, ROADMAP.md) check out exactly. I could not find any inaccuracy, overstatement, or missing counter-evidence in the finding - it holds up as an accurate confirmation of already-tracked issues' applicability to gr_normalize/log_predict/moduleDialog.ts, not a refutable new claim.

</details>

---

## Classic Sw (sw_arch, sw_indo, sw_sim)

### 1. sw_arch/sw_indo store +Infinity in their unlimited curves when RT is exactly 0 (or negative) — the same catalog-poisoning bug already fixed for PHIT=0/VSH=1, but for a different singular input

**Area:** Backend (Rust) — singularity handling

**Effort:** small

**Where:** src-tauri/src/modules.rs: sw_arch() lines 1768-1779 (`if is_missing(r) || is_missing(rw) { continue; }` then `let swt = (ff * rw / r).powf(1.0 / n_exp);`), sw_indo() lines 1866-1885 (`if is_missing(r) || is_missing(vs) || is_missing(rw) { continue; }` then `let swe = (1.0 / (r * (f1 + f2 + f3))).powf(1.0 / n_exp);`)

**Evidence:** Both functions only screen RT via is_missing() (NaN check), never against r<=0.0. If a sample's RT curve value is a literal 0.0 (not NaN) — e.g. a clipped/short-circuit resistivity reading or a stray zero surviving import/QC — sw_arch computes `(ff*rw/0.0).powf(1/n)` = +Infinity and stores it straight into the unlimited SWT_ARCH curve; sw_indo computes `(1.0/(0.0*(f1+f2+f3))).powf(1/n)` = +Infinity into SWE_INDO. I confirmed this with a direct arithmetic replay of both formulas (ff=6.25, rw=0.05 example): SWT_ARCH -> Infinity, SWE_INDO -> Infinity, both `isFinite()==false`, while the *limited* SWT/SWE curves happen to clamp to 1.0 via limit()'s clamp (so only the unlimited companion curve is corrupted). This is the exact class of bug the team already found and fixed twice in this very file — REVIEW.md/ROADMAP.md record 'Archie SWT_ARCH no longer writes +Infinity' (PHIT=0 case, with regression test `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf` asserting `out["SWT_ARCH"][0].is_finite()`) and 'Simandoux SCHLUMBERGER no longer divides by zero at VSH=1' — but neither fix, nor any test, covers RT=0. There is also a direct in-file precedent that RT<=0 is a recognized singularity needing an explicit guard: gascorr's inline Archie sub-solve (modules.rs lines 1471-1488) explicitly checks `r <= 0.0` before using RT, and even double-checks the result with `if s.is_finite() { s.min(1.0) } else { f64::NAN }` specifically to avoid propagating an infinite/garbage Sw forward — a defensive pattern absent from the primary sw_arch/sw_indo saturation modules. (sw_sim's Newton-Raphson solver degrades more gracefully to MISSING for RT=0, per my `calc_sw` replay with g3=-Infinity converging to NaN after 20 iterations, so it is not part of this finding.)

**Suggested fix:** In sw_arch and sw_indo, add an `r <= 0.0` guard next to the existing `is_missing(r)` check (mirroring gascorr's precedent) so a non-positive RT resolves to the same 'coal/all-water' MISSING or SWT=1/SWE=1 fallback used for the other singularities, instead of falling through to the raw division. Add regression tests analogous to `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf`, e.g. `sw_arch_zero_rt_is_finite_not_inf` / `sw_indo_zero_rt_is_finite_not_inf`.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified directly against D:\\XX. SandiBumi\\src-tauri\\src\\modules.rs (the actual SandiBumi/Arshilla repo). is_missing() (line 158-160) is only v.is_nan(), so a literal RT=0.0 passes both guards untouched: sw_arch's line 1769 `is_missing(r) || is_missing(rw)` and sw_indo's line 1866 `is_missing(r) || is_missing(vs) || is_missing(rw)`. sw_arch line 1778 computes swt=(ff*rw/r).powf(1/n_exp) and line 1779 stores it into swt_arch[i] with no is_finite check; sw_indo line 1884 computes swe=(1/(r*(f1+f2+f3))).powf(1/n_exp) and stores it unchecked into swe_indo[i]. I compiled and ran a standalone Rust program replicating these exact formulas (rustc 1.97.0): at RT=0.0, sw_arch's swt = inf (is_finite=false, and as f32 is_infinite=true), sw_indo's swe = inf — confirming real +Infinity is written into the unlimited SWT_ARCH/SWE_INDO curves, while the companion limited SWT/SWE clamp to 1.0 via limit()'s v.clamp(lo,hi), matching the claim that only the unlimited curve is poisoned. gascorr's inline Archie solve at lines 1471-1488 does contain the exact cited precedent: `r <= 0.0` guard (line 1474) plus a defensive `if s.is_finite() {...} else { f64::NAN }` (line 1487) — a pattern genuinely absent from sw_arch/sw_indo. REVIEW.md lines 297-305 verbatim confirm both cited prior fixes (PHIT=0 Archie SWT_ARCH +Infinity fix with regression test sw_arch_zero_porosity_missing_phie_is_all_water_not_inf at line 2867, and the Simandoux SCHLUMBERGER VSH=1 fix) — both real, neither covering RT=0. Grepping the whole file confirms no test sets RT=0 anywhere. I also numerically replayed sw_sim's calc_sw Newton-Raphson at RT=0 (g3=-1/0=-inf) and confirmed it returns NaN/MISSING after exhausting 20 iterations, matching the finding's carve-out that sw_sim is not part of this bug. Only minor imprecision: the finding's '(or negative)' parenthetical isn't uniformly true — with the default N=2, negative RT actually yields NaN (already a normal MISSING sentinel) rather than +Infinity, since powf(0.5) of a negative base is NaN in IEEE-754/Rust; only certain N values (e.g. N=1) would let a negative RT slip through as a finite garbage value. This doesn't undermine the core, well-evidenced claim about RT=0.0 exactly, which reliably yields +Infinity regardless of N. Overall the finding's code locations, mechanism, precedent, and test-gap claims are all accurate and independently reproduced.

</details>

### 2. Module-pane batch runs are logged to the History panel against the wrong well (or null), not the wells actually run

**Area:** Frontend wiring — History/Process log correctness

**Effort:** small-medium

**Where:** src/ui/workspace.ts lines 355-372 (the "module" pane case's `onRunComplete` wrapper) and src/ui/moduleDialog.ts lines 13-17 (`ModulePaneCallbacks.onRunComplete: (outputCurves: string[]) => void`) and lines 276-332 (the Run handler, which knows `wellIds` but never surfaces it to the callback)

**Evidence:** moduleDialog.ts's Wells checklist lets a user run sw_arch/sw_indo/sw_sim (or any manifest module) against an arbitrary multi-well selection (`wellIds`), but `onRunComplete` is only ever called with `outputs` (the manifest's output-curve names) — never the wellIds or results — so workspace.ts's wrapper has no way to know which/how many wells were actually processed. It falls back to `recordProcess("Module", \`Ran ${spec.title}\`, appState.selectedWell.get()?.well_name ?? null)`, i.e. it always attributes the run to whatever well happens to be the single globally-active/pinned well at that moment — which can be a well that was never in the run's checklist at all (or null, if none is selected), while the actual N run wells go unrecorded. This directly contradicts the documented convention: `ProcessEntry.well`'s own doc comment in src/processLog.ts (line 16-17) says 'Well it applied to, when it is well-scoped (null for field-wide/batch actions)', and every other batch-capable tool in the codebase honors it — e.g. multiminDialog.ts lines 715-719 passes the real `applyWells.join(", ")` as the well argument for its batch run, and workflowDialog.ts/mlDialog.ts/monteCarloDialog.ts all bake the well count into the `detail` string instead (e.g. `Ran chain (3 step(s) × 5 well(s))`). moduleDialog's auto-generated pane does neither, so the History panel and its text export (`processLogToText`) show a misleading single-well attribution for every batch module run.

**Suggested fix:** Widen `ModulePaneCallbacks.onRunComplete` to also pass the wellIds/well names actually run (or the RunModuleResult[] already computed in the Run handler), and have workspace.ts's wrapper build the recordProcess well argument from that list (comma-joined, like multiminDialog) or fold the count into the detail string, rather than reading `appState.selectedWell`.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified against the actual source in D:\XX. SandiBumi (the SandiBumi codebase) — every cited line matches exactly:

1. src/ui/workspace.ts lines 355-372 (the "module" pane case): the callback passed to buildModuleContent is
   `onRunComplete: () => { recordProcess("Module", \`Ran ${spec.title}\`, appState.selectedWell.get()?.well_name ?? null); this.notifyDataChanged(); }`
   — confirmed verbatim. It reads only `appState.selectedWell`, never any wellIds from the run itself.

2. src/ui/moduleDialog.ts lines 13-17: `ModulePaneCallbacks.onRunComplete: (outputCurves: string[]) => void;` — confirmed verbatim, the callback's only parameter is output-curve names.

3. src/ui/moduleDialog.ts Run handler (lines 276-332): `wellIds` is computed locally at line 277 from the checked wells and used to build the RunModuleRequest and to fetch `results` (which include per-well `well_id`/`rows_written`/`output_curves`/`error`). But the call at line 326 is `callbacks.onRunComplete(outputs)`, where `outputs` (defined at line 216: `spec.args.filter(a => a.kind === "log_out").map(a => a.name)`) is just the manifest's static output-curve-name list — computed once at pane-build time, completely independent of wellIds or the run results. Neither wellIds nor the RunModuleResult[] is ever surfaced to the callback.

4. src/processLog.ts lines 16-17: `/** Well it applied to, when it is well-scoped (null for field-wide/batch actions). */ well: string | null;` — confirmed verbatim.

5. src/ui/multiminDialog.ts lines 715-719: `recordProcess("Module", ..., applyWells.join(", "))` — confirmed, batch multimin runs pass the real joined well list as the well argument.

6. Confirmed the other cited batch tools bake the well count into `detail` instead: workflowDialog.ts:843 `Ran chain (${steps.length} step(s) × ${wellIds.length} well(s))`; mlDialog.ts:392 `...to ${applyIds.length} well(s)`; monteCarloDialog.ts:310 `...across ${wellIds.length} well(s)...`.

7. Also confirmed appState.selectedWell is an `Observable<WellSummary | null>` (state.ts:46) that is independent of the module pane's own multi-well checklist — the checklist only auto-syncs from selectedWell/defaultRunWellIds when nothing is checked yet (`if (wellChecks.some(w => w.input.checked)) return;`), so a user can freely check an arbitrary multi-well subset unrelated to whatever well is globally "selected," and selectedWell can be null. This substantiates the claim that the recorded well can be wrong or null while the actually-run wells go unattributed.

8. processLogToText (processLog.ts:84-90) does render `e.well` when present, confirming the History/export text would show the misleading single-well (or blank) attribution.

No mitigating code was found (no post-hoc correction, no TODO/comment acknowledging or overriding this behavior). Every piece of evidence in the finding checks out against the real files, so I could not refute it.

</details>

### 3. moduleDialog's live data refresh has no generation-counter race guard, unlike sibling async-reload code elsewhere in the app

**Area:** Frontend wiring — async-reload correctness

**Effort:** small

**Where:** src/ui/moduleDialog.ts lines 235-264 (`refreshData` and its `appState.dataVersion.subscribe` handler)

**Evidence:** `refreshData` re-fetches `listWells()`/`listCurveCatalog()` on every dataVersion bump and only guards against the pane having been disposed (`if (disposed) return;`) before unconditionally overwriting `wells`/`catalog`/`curveNames` and rebuilding the well checklist and dropdowns. There is no per-call generation token, so if dataVersion bumps twice in quick succession (e.g. two module/workflow runs finishing close together), two overlapping `refreshData` invocations can resolve out of order and the later-firing call's (stale) response can overwrite the newer one's state, leaving the well checklist/curve dropdowns silently out of date until the next data change. This is exactly the pattern the codebase elsewhere recognizes as needing a guard: `wellPane` in the same file (src/ui/workspace.ts lines 460-467) keeps its own `generation` counter for exactly this race on rebuild, ROADMAP.md explicitly records 'loadWell/reload/createPlot race guards (generation tokens)' as a shipped fix, and src/ui/cutoffDialog.ts (lines 68-87) implements the identical pattern with an `optionsEpoch` counter (`const epoch = ++optionsEpoch; ... if (epoch !== optionsEpoch) return;`) around its own async `Promise.all` reload. moduleDialog.ts's `refreshData` is the one live-refreshing async reload in this pane family that omits it.

**Suggested fix:** Add a generation counter to `refreshData` (increment on entry, bail if a newer call has started by the time the awaited `Promise.all` resolves), mirroring `cutoffDialog.ts`'s `optionsEpoch` pattern.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently verified every factual claim in the finding against the actual source in D:\XX. SandiBumi (the live SandiBumi/Arshilla repo):

1. src/ui/moduleDialog.ts lines 235-264: `refreshData` (lines 235-256) does `await Promise.all([listWells().then(filterByActiveGroup), listCurveCatalog()])`, then only checks `if (disposed) return;` (line 241) before unconditionally overwriting `wells`, `catalog`, `curveNames` and rebuilding the well checklist, log-input dropdowns, and mask options. No per-call generation/epoch token exists anywhere in this function or its enclosing scope. The subscribe wiring (lines 257-264) confirms `refreshData()` is invoked on every `appState.dataVersion` bump (after the initial "priming" fire), un-awaited (`void refreshData()`), so nothing prevents two overlapping invocations in flight simultaneously.

2. The race is not merely theoretical: `state.ts`'s `Observable.set()` (lines 21-24) fires all listeners synchronously and has no serialization; `listWells`/`listCurveCatalog` (ipc.ts lines 40-61) are Tauri `invoke()` round-trips to the Rust backend with no ordering guarantee on resolution. `dataVersion` is bumped from many sites (workflowDialog.ts, multiminDialog.ts, cutoffDialog.ts, topsEditor.ts, etc.), and critically, `workspace.ts`'s `onRunComplete` handler for a module pane calls `this.notifyDataChanged()` → `bumpDataVersion()` (workspace.ts lines 365-368, 1066-1068), which is broadcast to every open module pane — so two module runs (e.g. sw_arch and sw_sim panes open side by side) finishing close together is exactly the scenario described, not a contrived one.

3. The sibling-pattern claims are verbatim accurate: `wellPane` in src/ui/workspace.ts (lines 460-538) maintains its own `generation` counter, incrementing on each rebuild (`const gen = ++generation`, line 467) and checking `gen !== generation` after each async resolution (lines 482, 490) before applying results — the identical guard the finding says is missing from moduleDialog. `src/ui/cutoffDialog.ts` lines 68-87 implements the same idiom via `optionsEpoch`: `const epoch = ++optionsEpoch; ... if (epoch !== optionsEpoch) return;` around its own `Promise.all` reload. ROADMAP.md line 516 does record `- [x] loadWell/reload/createPlot race guards (generation tokens) + a sticky reset flag` as a shipped fix, exactly as cited.

4. The sw-classic-group linkage holds: src-tauri/src/modules.rs registers `sw_arch_spec()`, `sw_indo_spec()`, `sw_sim_spec()` (lines 184-186, 1714, 1814, 1902) as `ModuleSpec`s dispatched generically; workspace.ts's `case "module":` handler (lines 355-372) builds every module's pane — including these three — through the shared `buildModuleContent` in moduleDialog.ts, so the finding's UI-wiring scope is correct.

Every cited line range, code snippet, and mechanism in the finding matches the real file contents exactly, and the described race condition is mechanically real given the codebase's own concurrency primitives (synchronous Observable broadcast + unguaranteed-order async IPC + un-awaited fire-and-forget refresh). I found no inaccuracy, exaggeration, or missing mitigating guard that would refute the finding. The suggested fix (mirror cutoffDialog's optionsEpoch pattern) is a direct, proportionate match to the existing codebase convention.

</details>

---

## Permeability (perm_wyllie_rose, perm_coates, perm_transform)

### 1. perm_coates default CONST_COATES (100) doesn't match the the reference suite source it claims to port (documented default is 70)

**Area:** Domain correctness (C)

**Where:** src-tauri/src/modules.rs lines 2062-2076 (perm_coates_spec, `param("CONST_COATES", ..., 100.0, 1.0, 1000.0)`); module-file header at lines 1-3 claims parity with `perm_coates.lls`

**Evidence:** The file's own top-of-file doc comment states this library is 'ported from Loglan sources ... perm_coates.lls ... with the same MISSING semantics, LIMIT clamping, and per-frame evaluation model' — i.e. it claims formula-and-defaults parity, not just formula-shape parity. The geolog-loglan skill's cookbook (references/cookbook/perm-lith.md), which distills the real the reference suite V14 .lls/.info source, documents PERM_COATES's formula as `(CONST_COATES * PHIE**2 * (1-SWE_IRR)/SWE_IRR)**2` implemented via a self-multiply square (`PERM=C*PHIE*PHIE*(1-sw)/sw; PERM=PERM*PERM`) — exactly matching modules.rs's `let k = c*pe*pe*(1.0-swirr)/swirr; perm[i] = (k*k)`, confirming this was a line-for-line port. But the cookbook records the real default as `CONST_COATES (default 70)`, while modules.rs defaults it to 100.0. A user who runs perm_coates without touching the pre-filled dialog value gets permeability (100/70)^2 ≈ 2.04x higher than the reference suite's real default would give for the same PHIE/SWE_IRR.

**Suggested fix:** Change the default in `param("CONST_COATES", "Coates constant", "", 100.0, 1.0, 1000.0)` (perm_coates_spec, modules.rs ~line 2069) to 70.0 to match the ported the reference suite module, or if 100 was a deliberate recalibration, document that intent in the module's doc string so it isn't mistaken for a transcription error.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I tried to refute the finding by reading modules.rs directly and cross-checking against the actual the reference install installation present on this machine (not just the skill's cookbook). Code confirmed: modules.rs perm_coates_spec (line 2069) sets CONST_COATES default = 100.0 via param(\"CONST_COATES\", \"Coates constant\", \"\", 100.0, 1.0, 1000.0), and perm_coates() implements k=c*pe*pe*(1-swirr)/swirr; perm=k*k — a self-multiply square matching the reference suite's idiom exactly. I then read the primary source itself: C:\\Program Files\\AspenTech\\the reference install\\loglan\\perm_coates.lls (doc comment: 'CONST_COATES is defaulted to 70'; code: PERM_COATES_FFI = CONST_COATES*PHIE*PHIE*(1-swirrtemp)/swirrtemp; PERM_COATES_FFI = PERM_COATES_FFI*PERM_COATES_FFI) and perm_coates.info (parameter table row: CONST_COATES default column = 70, validation 0:1000). This independently confirms, from the real the reference suite source rather than just the cookbook, that the true default is 70 while SandiBumi hardcodes 100.0 — a genuine 2.04x-permeability discrepancy for users who accept the pre-filled default. I also checked src/ui/moduleDialog.ts and found no override/correction layer; it just does input.value = arg.default, so the wrong 100.0 flows straight into the user-facing dialog. I could not find any evidence contradicting the finding, no comment indicating deliberate recalibration, and the formula-port claim in the file's own header (which the finding relies on for context) is itself verified true. The finding stands confirmed.

</details>

### 2. perm_wyllie_rose: identical negative-PHIE input is silently NaN'd under the default TIMUR method but silently produces a finite, plausible-looking PERM under the other three OPT_WR variants

**Area:** Backend singularity handling (B)

**Where:** src-tauri/src/modules.rs lines 2037-2056 (fn perm_wyllie_rose)

**Evidence:** The function computes `k = (c * pe.powf(d) / swirr.powf(e)).powi(2)` with no guard on `pe < 0.0` (only `is_missing(pe) || is_missing(swirr) || swirr <= 0.0` are checked). Verified directly with rustc: for TIMUR (default, d=2.25, non-integer exponent), `(-0.05_f64).powf(2.25)` evaluates to `NaN` (correct IEEE-754 behavior for a non-integer power of a negative base), so a negative PHIE correctly propagates as missing. But for MORRIS_BIGGS_OIL/GAS and TIXIER (d=3.0, an integer exponent), `(-0.05_f64).powf(3.0)` evaluates to `-0.000125` (a valid real cube), which is then divided by swirr^1 and squared via `.powi(2)`, yielding a positive, physically-plausible-looking PERM value instead of NaN. So switching only the OPT_WR dropdown — with the exact same (invalid, negative) PHIE input — changes whether the module correctly flags the sample as missing or silently fabricates a number. PHIE is a free log-picker (not restricted to phi_den/phi_dn's floor-clamped output), so a raw or noise-bearing porosity curve with small negative values in tight/dense intervals can reach this path.

**Suggested fix:** Add an explicit `pe < 0.0` (or `pe <= 0.0` per convention) guard alongside the existing is_missing/swirr<=0 check in perm_wyllie_rose so all four OPT_WR branches treat invalid porosity identically (skip/NaN), rather than relying on the accidental parity/non-parity of each branch's exponent.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the real file D:\XX. SandiBumi\src-tauri\src\modules.rs (lines 2037-2056, matching exactly). Confirmed: (1) is_missing(v) = v.is_nan() only (line 158-160), so a negative-but-finite PHIE is never caught by the existing guard; (2) no pe<0.0 guard exists in perm_wyllie_rose; (3) compiled and ran the exact expressions with rustc 1.97.0: (-0.05).powf(2.25) [TIMUR default, non-integer exponent] = NaN, correctly propagating as missing, while (-0.05).powf(3.0) [MORRIS_BIGGS_OIL/GAS, TIXIER, integer exponent] = -0.000125, a finite value that survives division by swirr and .powi(2) to yield a finite positive PERM (0.0434 mD in the test case) — reproducing the claimed parity-dependent asymmetry exactly; (4) verified via src/ui/moduleDialog.ts (lines 98-104) that PHIE is populated from the entire curve catalog with no value-range or provenance restriction (computed_only=false for this arg, and ctx.log() at lines 135-137 does no clamping on retrieval); (5) confirmed phi_den's own PHIE output IS floor-clamped via limit(pe,0.0,phie_lim) (lines 476-479), supporting the claim that other curves (PHIE_DEN unclamped, raw/edited porosity) routed through the free picker would not carry that floor. No upstream sanitization in workflow.rs and no existing unit test for this path. Every specific factual claim in the finding held up under direct inspection and empirical (rustc) verification; I found nothing that refutes it.

</details>

### 3. perm_transform can silently emit +Infinity (not NaN) for parameter/porosity combinations within its own validated dialog ranges

**Area:** Backend singularity handling (B)

**Where:** src-tauri/src/modules.rs lines 2098-2129 (perm_transform_spec / fn perm_transform)

**Evidence:** perm_transform computes `perm[i] = 10.0_f64.powf(a*pe+b) as f32` with only an `is_missing(pe)` guard — no finiteness check on the result, and no `limit()` clamp anywhere in the function (unlike neighboring porosity/VSH modules, which do clamp their outputs). PT_A is validated to [1.0, 100.0] and PT_B to [-10.0, 5.0] (param() calls at lines 2107-2108), both within the auto-generated dialog's own accepted range. Verified directly with rustc: PT_A=100, PT_B=5, PHIE=0.6 gives exponent 65, `10.0_f64.powf(65.0) as f32` == `f32::INFINITY` (confirmed `is_infinite()==true`, `is_nan()==false`). Since `is_missing()` only tests `.is_nan()`, this +Infinity is treated as a valid present value and would be written straight to computed_curves, then consumed downstream (e.g. sw_height's LEVERETT PERM input at src-tauri/src/satheight.rs line 118/130) as a legitimate number — corrupting any min/max, statistics, or plot that assumes finite curve values. The overflow threshold (exponent > ~38.5) is reachable well inside the validated PT_A range once PHIE exceeds roughly a third of PT_A's value, e.g. a steep, otherwise-plausible high-quality-sand calibration (PT_A near the upper part of its range) combined with ordinary reservoir porosity.

**Suggested fix:** Clamp perm_transform's output to a sane physical ceiling (e.g. `limit(k, 0.0, some_max_md)`) or at minimum replace non-finite results with `f32::NAN` (`if !k.is_finite() { NaN } else { k as f32 }`) so an extreme parameter/porosity combination degrades to 'missing' instead of silently propagating +Infinity.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently read the real source and could not refute the finding — every load-bearing claim checks out.

1. Code matches exactly. D:\XX. SandiBumi\src-tauri\src\modules.rs lines 2098-2129 contain perm_transform_spec/perm_transform verbatim as described:
   - `param("PT_A", "Slope", "", 20.0, 1.0, 100.0)` and `param("PT_B", "Intercept", "", -3.0, -10.0, 5.0)` (lines 2107-2108) — validated ranges PT_A∈[1,100], PT_B∈[-10,5], exactly as claimed.
   - The body (lines 2116-2129) does `perm[i] = 10.0_f64.powf(a * pe + b) as f32;` guarded only by `if is_missing(pe) { continue; }` (line 2123-2125) — no check on `a`, `b`, or the result.
   - `is_missing()` (line 158) is `v.is_nan()` only — it does not catch infinities.
   - No `limit()`/clamp call anywhere in perm_wyllie_rose, perm_coates, or perm_transform (grepped every `limit(` call site in modules.rs: lines 287-2190 — all hits are in vsh_gr, phi_den, phi_dn, phi_son, phimax, gascorr, sw_arch/indo/sim, thin_bed_ts; none in the three permeability modules). This confirms the "unlike neighboring porosity/VSH modules" contrast.

2. Dialog validation is real, not just documentation: src\ui\moduleDialog.ts lines 288-289 reject any PT_A/PT_B outside [min,max] before the request is even sent, so PT_A=100, PT_B=5 are legitimately reachable through the UI's own validated range. Backend `resolve_param_arrays` (workflow.rs lines 47-88) does not re-clamp to min/max either — it just takes the dialog value (or default) verbatim.

3. Overflow math independently verified by compiling and running actual Rust (rustc 1.97.0) rather than trusting the finding's transcript:
   - PT_A=100, PT_B=5, PHIE=0.6 → exponent 65 → `10.0_f64.powf(65.0) as f32` = `inf`, `is_infinite()=true`, `is_nan()=false`. Matches the finding's cited rustc verification exactly.
   - Also confirmed the threshold: f32::MAX ≈ 3.4028e38, log10(f32::MAX) ≈ 38.532. With PT_A=100 (upper part of range) and PT_B=5, even an "ordinary" PHIE=0.35 gives exponent=40 > 38.53 → overflow. This corroborates the finding's claim that overflow is reachable "once PHIE exceeds roughly a third of PT_A's value" — not just an exotic edge case.

4. No write-path sanitization exists: equations::write_computed_curves_versioned / _batch (equations.rs lines 574-716) append whatever f32 values are given straight into `computed_curves` with no is_finite() filter — I grepped every is_finite/is_infinite usage in workflow.rs/modules.rs/db.rs and none guard the generic curve-write path.

5. Downstream consumption confirmed at the cited location: satheight.rs line 118 (`log_in("PERM", ...)`) and line 130 (`let perm = ctx.log("PERM");`) — PERM is read and used as a legitimate value (only `k.is_nan() || k <= 0.0` is checked, not `is_finite()`). I also found a second, cleaner real corruption path: workflow.rs line 692 and montecarlo.rs line 250 do `pay = pay && !perm.is_nan() && (perm as f64) >= perm_min.unwrap()` for the pay-flag/cutoff logic — an infinite PERM value passes this check trivially, silently marking every sample as pay regardless of the actual cutoff. (Note: in the one specific sw_height Leverett arithmetic path I traced through, the final `.clamp()` on SWH happens to save that particular output curve from also going infinite/NaN in the default-negative-SWH_B case — but that doesn't rescue the underlying PERM/PERM_XFM curve itself, which is what actually gets written to computed_curves and consumed by pay-flag cutoffs and any min/max/statistics query.)

Nothing I found contradicts the finding; the suggested fix (clamp to a physical ceiling, or replace non-finite results with NaN before storing) is appropriate and consistent with the `limit()` helper already used by sibling modules.

</details>

### 4. perm_wyllie_rose, perm_coates, and perm_transform have zero dedicated unit tests — no edge cases (phi=0, negative phi, swirr boundary, missing input, non-default OPT_WR variants) are ever exercised

**Area:** Test coverage (B)

**Where:** src-tauri/src/modules.rs (fn tests module starts at line 2533); src-tauri/src/workflow.rs lines 1605-1679 (test_full_deterministic_chain)

**Evidence:** The file has 45 `#[test]` functions total (many covering other modules' edge cases, e.g. phimax_constant_caps_and_preserves_missing), but grepping the whole file (and specifically the `mod tests` block from line 2533 onward) for perm_wyllie_rose/perm_coates/perm_transform, their param names (CONST_COATES, OPT_WR), or their output curve names (PERM_WR, PERM_COATES, PERM_XFM) returns zero matches. The only place any of the three is ever exercised is workflow.rs's `test_full_deterministic_chain` (lines 1609-1679), which is `#[ignore]`'d (machine-specific LAS paths, not run in CI/normal `cargo test`) and only runs perm_wyllie_rose's default TIMUR branch once, on real field data, asserting just that PERM stays within [0, f64::MAX] — no assertion touches phi=0, negative phi, swirr=0/boundary, all-missing input, or the MORRIS_BIGGS/TIXIER/coates/transform code paths at all.

**Suggested fix:** Add unit tests for each of the three modules covering: phi=0 (expect PERM=0, not NaN/panic), negative/garbage PHIE (expect consistent NaN across all OPT_WR branches once fixed), swirr at its 0.01 floor and swirr<=0 defensive path, all-missing PHIE input, and at least one assertion per OPT_WR variant (MORRIS_BIGGS_OIL/GAS, TIXIER) and per perm_transform parameter extreme (PT_A/PT_B at range bounds) to catch the overflow case above.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual source at D:\XX. SandiBumi\src-tauri\src\modules.rs and workflow.rs (the repo referenced by the finding) and could not refute the claim — every specific in the evidence checks out, and I found nothing that contradicts the substantive conclusion.

Verified directly:
1. `mod tests` in modules.rs does start at line 2533 (confirmed via grep).
2. `#[test]` count in modules.rs is exactly 45 (confirmed via grep count), and the cited example `phimax_constant_caps_and_preserves_missing` exists at line 2734.
3. Grepping the whole file for `perm_wyllie_rose|perm_coates|perm_transform|CONST_COATES|OPT_WR|PERM_WR|PERM_COATES|PERM_XFM|PT_A|PT_B` returns matches only at lines 3, 189-235, and 2015-2129 — i.e. only in the module-registry/spec/implementation code (lines < 2533). Zero hits anywhere in the `mod tests` block (2533 onward). This directly confirms "zero dedicated unit tests" for these three modules.
4. Read the actual implementations (lines 2018-2129): perm_wyllie_rose has 4 OPT_WR branches (TIMUR default, MORRIS_BIGGS_OIL, MORRIS_BIGGS_GAS, TIXIER) and a `swirr <= 0.0` defensive skip; perm_coates has a CONST_COATES param and the same swirr<=0 defensive skip; perm_transform has PT_A/PT_B with only a missing-PHIE check (no swirr guard at all). None of these branches/guards are referenced anywhere in the tests module.
5. Read workflow.rs lines 1605-1703: `test_full_deterministic_chain` is `#[test] #[ignore]`, uses hardcoded machine-specific LAS paths (e.g. `D:\01. Work\2023\10. LQR Balam South...`), calls `run("perm_wyllie_rose", &[("SWE_IRR", 0.15)], &[("OPT_WR", "TIMUR")]);` exactly once (line 1660), and the only assertion touching PERM is the range check `("PERM", 0.0, f64::MAX)` at line 1666/1677 — no phi=0, negative phi, swirr-boundary, missing-input, or non-TIMUR-variant assertion exists.
6. Grepped the whole repo for `MORRIS_BIGGS|TIXIER` — matches only in modules.rs itself (the spec/match-arm definitions), confirming those OPT_WR variants are never exercised by any test anywhere.
7. Confirmed no other test infrastructure exists that could cover this: `src-tauri/tests/` only holds DB fixture files (no .rs), and there are zero `*.test.ts`/`*.spec.ts` files anywhere in the project (so moduleDialog.ts likewise has no test coverage, consistent with the finding's framing).

One minor correction to the evidence (does not change the verdict): grepping `*.rs` project-wide turned up a third file, `src-tauri/src/pipeline_blso_test.rs` (registered as `mod pipeline_blso_test;` in lib.rs), whose `pipeline_blso_full_run` test loops over `modules::list_modules()` and runs every module including perm_coates/perm_transform with default params against real BLSO field LAS files. So the finding's phrase "the only place any of the three is ever exercised is workflow.rs's test_full_deterministic_chain" is not literally complete. However this file is also `#[test] #[ignore]`'d, uses hardcoded machine-specific paths, and only asserts "no hard error" / "not all-NaN" per module — it contains no assertion touching phi=0, negative phi, swirr boundaries, missing input, or any non-default OPT_WR/CONST_COATES/PT_A/PT_B variant. So it reinforces rather than refutes the core claim of zero dedicated edge-case unit tests.

Conclusion: the finding's category, location, and specific evidence are all verifiably accurate against the real code. The suggested fix is sound and addresses genuine, confirmed gaps (untested phi=0/negative-phi behavior difference between perm_transform (no swirr floor, non-NaN at phi=0) vs perm_wyllie_rose/perm_coates (swirr<=0 skip, phi=0 -> 0), and untested MORRIS_BIGGS/TIXIER/OPT_WR branches).

</details>

---

## Misc analysis (thin_bed_ts, depth_shift, splice, sw_height)

### 1. sw_height's TVD input has no producer anywhere in the app — the deviated-well fix (marked DONE, unit-tested) is a no-op in real use

**Area:** domain correctness / cross-function data provenance

**Effort:** medium

**Where:** src-tauri/src/satheight.rs:117-144 (log_in("TVD",...) and the per-sample dv fallback); src-tauri/src/deviation.rs:75-100 (tvd_at, #[allow(dead_code)]); src-tauri/src/db.rs:1756-1780 (well_path insert/get, survey-station only); src-tauri/src/equations.rs (fetch_curve_frame_from_set / fetch_generic_curve_aligned — zero references to well_path or tvd_at); src/ui/moduleDialog.ts:94-104 (logChoiceNames always injects the manifest default, so "TVD" appears selectable even though it never exists)

**Evidence:** AUDIT-2026-07-20.md's confirmed HIGH finding said MD-based height biases SWH optimistic in deviated wells and suggested EITHER an optional TVD LogIn OR auto-resolving TVD from well_path via tvd_at. ROADMAP/REVIEW mark the fix DONE by implementing only the first option: satheight.rs:119 adds log_in("TVD",...,"TVD",false), and lines 141-144 do `let t = tvd[i] as f64; if t.is_nan() { depth[i] as f64 } else { t }`. But nothing in the codebase ever materializes a per-sample "TVD" curve for a module to read. The app's own deviation-survey import (ribbon "Import Deviation…" -> ingest::import_deviation_csv -> db::insert_well_path) writes ONLY into the well_path table at survey-station granularity (db.rs:1756-1768); get_well_path (db.rs:1769) is a plain read used elsewhere. deviation.rs's tvd_at (line 81) is explicitly `#[allow(dead_code)]` with a comment (lines 75-79) saying it's "Consumed by the Phase 6c TVD-depth-scale option in the log/correlation views" — a separate, still-deferred DISPLAY feature (ROADMAP.md:249-255), not a curve materializer. Grepping equations.rs (the module input-resolution path: fetch_curve_frame_from_set, fetch_curve_frame, fetch_generic_curve_aligned, fetch_named_curve_aligned) for "well_path"/"tvd_at" returns zero hits — there is no bridge from the deviation survey into curve_meta/curve_samples/computed_curves/standard_curves. So the only way a well could ever have a curve literally named "TVD" is if the source LAS/DLIS file itself already contained that channel (vendor-dependent, incidental — not something this app's own deviation-import feature produces). When no such curve exists, ctx.log("TVD") returns an all-NaN column (ModuleContext::log, modules.rs:135-137: `self.logs.get(name).cloned().unwrap_or_else(|| vec![f32::NAN; self.n])`), so satheight.rs:141-144 falls back to measured depth for every single sample, silently reproducing the exact MD-based bias the fix was supposed to eliminate — with no error, warning, or status message anywhere (workflow.rs's runner does no existence check on resolved mnemonics). Compounding this, moduleDialog.ts's logChoiceNames (line 96: `curveNames.includes(keep) ? curveNames : [keep, ...curveNames]`) always injects the manifest default into the dropdown even when it isn't a real curve in the catalog, so the TVD picker shows "TVD" as a normal selectable option in every well, giving false affordance that the deviated-well correction is active. ROADMAP.md's own "Done when" line (302) — "SWH tracks core Sw (field click-through pending, REVIEW.md)" — and REVIEW.md's P0 checklist item (marked done only at the unit-test level, per its own "implemented and unit-tested... has not been clicked through with real field data" framing) corroborate that nobody has yet exercised this on an actual deviated Mahakam well, where the gap would surface immediately.

**Suggested fix:** Auto-resolve TVD from well_path when no explicit TVD curve is selected: in workflow.rs's module-input resolution (alongside the existing log_args loop), if the well has well_path stations, interpolate tvd_at(stations, md) onto the fetched depth grid and inject it as the "TVD" log whenever the user's TVD picker is left at its default/blank — the same auto-resolve alternative the original AUDIT-2026-07-20.md finding already proposed. Independently, add a materializer (an IPC command or a step of the deviation import) that writes the minimum-curvature TVD onto the log's full depth grid into curve_meta/curve_samples (set RAW, mnemonic TVD) so it also shows up correctly in the Curve Catalog and is selectable like any other curve. At minimum, have moduleDialog.ts/workflow.rs surface a warning when a selected LogIn mnemonic resolves to an all-missing column, so a silent MD fallback is never mistaken for a working TVD-based run.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read every cited file/line directly (satheight.rs, deviation.rs, db.rs, equations.rs, modules.rs, moduleDialog.ts, workflow.rs, ROADMAP.md, REVIEW.md) plus adjacent code not cited in the finding (workflow.rs's actual log-resolution loop at lines 198-204 and its only warning path at 289-293, ipc.ts's getWellPath usage, parsers.rs's TVD alias) specifically looking for a producer of a per-sample TVD curve that would refute the claim. Found none. All quoted code snippets match the real source verbatim: log_in(\"TVD\",...) and the NaN-fallback in satheight.rs; tvd_at's #[allow(dead_code)] and zero external callers in deviation.rs (confirmed via grep across src-tauri/src); well_path being survey-station-only in db.rs; zero well_path/tvd_at references in equations.rs; ModuleContext::log's NaN-fallback in modules.rs; moduleDialog.ts's logChoiceNames always injecting the manifest default even when absent from the real curve catalog (catalog confirmed to only list actually-computed/imported curves). Additionally verified workflow.rs's module-input resolver has the identical silent-NaN pattern with no missing-curve warning (the only 'Warned' status fires solely when a module's whole output map is empty, which sw_height's is not), and confirmed ROADMAP.md/REVIEW.md text matches almost verbatim, including REVIEW.md's own header stating nothing in it has been clicked through with real field data. Also found corroborating evidence beyond the finding itself: the frontend getWellPath() wrapper is never called anywhere in src/, meaning even the display bridge is currently dead on the frontend too. I could not construct any refutation — the finding holds up against the actual codebase.

</details>

---

## Facies (electrofacies, gmm_facies)

### 1. History-panel entry for a module run is attributed to the wrong well (or none) whenever the run is the intended multi-well batch case

**Area:** Frontend wiring (D) / cross-function (F) — moduleDialog.ts + workspace.ts

**Where:** src/ui/workspace.ts lines 363-369 (onRunComplete callback for the "module" pane kind); src/ui/objectTree.ts lines 78-88 (handleWellClick, Ctrl/Shift-click multi-select) and its comment at line 80 "...without moving the active well, so a batch set can be built"; src/state.ts lines 84-91 (defaultRunWellIds pre-ticks the checklist from multiSelectedWellIds); src/processLog.ts line 16 (documented contract: well = "null for field-wide/batch actions")

**Evidence:** workspace.ts's pane host records every module run (electrofacies, gmm_facies, and every other manifest module run through moduleDialog.ts) as: `recordProcess("Module", `Ran ${spec.title}`, appState.selectedWell.get()?.well_name ?? null)`. This ignores the actual `wellIds` the run executed against (moduleDialog.ts's checklist, built by `defaultRunWellIds` from `appState.multiSelectedWellIds`) and instead reads `appState.selectedWell` — a *different*, independent piece of state that objectTree.ts's own comment confirms is deliberately left unchanged during a Ctrl/Shift-click multi-select ("without moving the active well, so a batch set can be built while every view stays put"), and whose status message explicitly advertises this as the path into batch dialogs ("batch dialogs will pre-tick them"). Concretely: Ctrl-click 10 wells in the Wells tree (the designed way to batch-run a module across many wells), open Electrofacies, Run — the pane correctly pre-ticks and processes all 10 wells, but the History entry reads e.g. "WellX: Ran Electrofacies (K-means)" where WellX is whatever well was last single-clicked (possibly none of the 10, or none at all). This both misattributes the action to an untouched well and omits how many wells were actually run — unlike workflowDialog.ts's own recordProcess call (`Ran chain (${steps.length} step(s) × ${wellIds.length} well(s))`), which correctly reports scope and never claims a single well. It also breaks processLog.ts's own documented contract that the well field should be null for field-wide/batch actions, which REVIEW.md's audit-trail sweep entry (line 233) states is deliberately how other batch dialogs (ML, MC, workflow, cutoffs) already behave.

**Suggested fix:** In workspace.ts's "module" case, pass the well scope actually used by the run (e.g. have moduleDialog.ts report the well_ids/well_names of a completed run via onRunComplete, or the count) instead of appState.selectedWell; when more than one well was run, log null (or a count) per the field-wide/batch convention rather than a single well name that may not have been touched at all.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified every cited line against the real source at D:\XX. SandiBumi\src\ (git repo root; SandiBumi is the renamed Arshilla folder per user memory).

1. workspace.ts:366 — exact code confirmed: `recordProcess("Module", \`Ran ${spec.title}\`, appState.selectedWell.get()?.well_name ?? null);` inside the "module" pane case's onRunComplete callback (lines 355-372).

2. moduleDialog.ts — confirmed the run button handler (line 277) builds `wellIds` from `wellChecks` (the checklist), which is initialized via `defaultRunWellIds(wells)` (line 73) and re-applied on well-change (line 271). Confirmed `runWorkflowModule(req)` is called with `well_ids: wellIds` (an array — genuinely multi-well, one `ModuleRunResult` per well, line 313-324). Critically, confirmed `onRunComplete`'s type signature is `(outputCurves: string[]) => void` (line 15) and it's invoked as `callbacks.onRunComplete(outputs)` (line 326) — the actual well_ids/well_names run are never passed back to workspace.ts at all, so workspace.ts has no way to know the real scope even if it wanted to.

3. state.ts:84-91 — `defaultRunWellIds` confirmed verbatim: uses `multiSelectedWellIds` when non-empty, else falls back to `selectedWell`.

4. objectTree.ts:78-88 — `handleWellClick` confirmed: the Ctrl/Meta-click branch (line 79-87) only toggles `appState.multiSelectedWellIds` and calls `setMulti`; it never calls `onSelectWell` or touches `appState.selectedWell`. The comment at line 80-81 reads verbatim "Toggle in/out of the multi-selection without moving the active well, so a batch set can be built while every view stays put." Confirmed `appState.selectedWell.set(...)` is only invoked from the plain-click path via `onSelectWell` (workspace.ts:729) — never during Ctrl/Shift-click. `setMulti`'s status message (line 111) confirmed verbatim: "N well(s) selected — batch dialogs will pre-tick them."

5. processLog.ts:16-17 — documented contract confirmed verbatim: "Well it applied to, when it is well-scoped (null for field-wide/batch actions)." `recordProcess`'s third parameter defaults to `null` (processLog.ts:42).

6. workflowDialog.ts:843 — confirmed verbatim: `recordProcess("Workflow", \`Ran chain (${steps.length} step(s) × ${wellIds.length} well(s))\`)` — omits the well arg (defaults null), reports scope in the message instead. Same correct pattern confirmed independently in mlDialog.ts:392, monteCarloDialog.ts:310, and inspectorPanel.ts:320 (Equation runs) — all report a count, none pass a single well name.

7. REVIEW.md:232-233 — confirmed verbatim: "(Batch/field-wide actions — equation, ML, MC, workflow, log-set, cutoffs — intentionally show no well name.)" "Module" is conspicuously absent from that list, consistent with module runs having been left on the old single-well logging pattern even after gaining multi-well batch capability via the checklist/defaultRunWellIds mechanism.

8. Confirmed electrofacies and gmm_facies (facies.rs) are registered as ordinary manifest modules (modules.rs:197-198, 228-229) and therefore routed through the exact same generic workspace.ts "module" pane / moduleDialog.ts path being flagged — not a special case.

No contradicting code was found anywhere (no alternate recordProcess call for "Module" kind exists, no test covers/guards this path). Every file, line number, quoted comment, and code snippet in the finding matches the real repository exactly. The bug is real: a Ctrl-click batch run across N wells via Electrofacies/GMM-facies (or any manifest module) logs a History entry against whatever well was last plain-clicked (or null), not the batch actually run, and omits the well count — violating processLog.ts's own documented null-for-batch contract that every sibling batch dialog (ML, MC, workflow, equations) correctly follows. I was unable to refute the finding.

</details>

### 2. facies.rs's "can't cluster this well" cases (no input curve present, or fewer complete samples than K) are silently reported as a full successful run with a plausible row count, not a warning or error

**Area:** Backend correctness / error surfacing (B, E) — facies.rs + workflow.rs + moduleDialog.ts

**Where:** src-tauri/src/facies.rs lines 72-74 (`if present.is_empty() { return None; }`) and lines 95-97 (`if pts.len() < k { return None; }`), both feeding electrofacies (line 137) and gmm_facies (line 196); src-tauri/src/workflow.rs line 351/403 (`if outputs.is_empty()`) and line 414 (`rows_written: depth.len()`); src/ui/moduleDialog.ts line 321 (result line rendering)

**Evidence:** When prep_samples can't build a clustering input (every CURVE1-5 slot is entirely missing for that well, or the well's complete-sample count is below K), electrofacies/gmm_facies return `HashMap::from([("FACIES", vec![NaN; n])])` (and similarly FACIES_GMM/FPROB) rather than an Err or an actually-empty map. workflow.rs's success gate is `outputs.is_empty()` (lines 289, 314, 351, 403) — since every module in modules.rs (not just facies) pre-populates its output map with NaN-filled vectors before any per-sample logic runs, this map is never actually empty, so the well is always counted as a produced/`Outcome::Computed` success, gets a new log-set version allocated, and `rows_written` is set to `depth.len()` (the well's total sample count, not the count of non-NaN outputs). The result: running Electrofacies on a well that lacks every one of GR/RHOB/NPHI/DT/SP (or whose chosen interval has fewer complete rows than K) shows `✓ WellX: 3000 samples → FACIES` in the module pane — a green success line — while FACIES is 100% NaN for that well, and a new (useless) constellation version was still written. Nothing in the Processing-panel per-well state, the resultBox text, or the module's own unit test suite (facies.rs lines 454-581, which covers missing-sample-level NaN propagation but never the whole-well "no curve present"/"too few samples" branches) distinguishes this from a real run.

**Suggested fix:** Have prep_samples's None case surface as an explicit per-well error/warning (e.g. "no input curves present" / "fewer than K complete samples") through run_module's Result, so workflow.rs reports it as Failed/Warned instead of silently green; add a unit test asserting the all-absent-curves and pts.len()<k paths are distinguishable from a successful cluster.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently read facies.rs, workflow.rs, and moduleDialog.ts in D:\XX. SandiBumi\src-tauri and D:\XX. SandiBumi\src\ui. Every specific claim in the finding matches the real code exactly:\n\n1. facies.rs lines 72-74 (`if present.is_empty() { return None; }`) and lines 95-97 (`if pts.len() < k { return None; }`) are verbatim as described.\n2. electrofacies (line 137) and gmm_facies (line 196) both handle prep_samples' None via `let Some(...) = prep_samples(ctx) else { return HashMap::from([NaN-filled vectors]); }` — confirming the returned map is non-empty (contains FACIES / FACIES_GMM+FPROB keys with all-NaN values) even on the "can't cluster" paths.\n3. workflow.rs's success gates at lines 289, 314, 351, and 403 all test `outputs.is_empty()` — never any is_nan/all-NaN check on the actual output values. I grepped the whole file for NaN-aware logic and confirmed none of it touches the Computed/Ok/Warned/write-path branching (only the unrelated pay-summary cutoff engine inspects is_nan).\n4. workflow.rs line 414 sets `rows_written: depth.len()`, i.e. the well's total sample count, not a count of non-NaN outputs — confirmed.\n5. moduleDialog.ts lines 319-322 render `✓ ... {rows_written} samples → {output_curves}` keyed purely off `!r.error`, with no inspection of curve values — confirmed.\n6. Checked whether `ArgSpec.required` (CURVE1) is enforced anywhere at runtime that might prevent reaching this state — it is not; the only two usages outside its definition are unit-test assertions in modules.rs. moduleDialog.ts's run handler also has no pre-flight curve-availability check.\n7. Reviewed facies.rs's full unit-test module (lines 454-581): tests cover single-sample NaN propagation (missing_inputs_yield_missing_facies, gmm_deterministic_and_missing_propagates) but none construct an all-curves-absent or pts.len()<k well, confirming the stated test gap.\n\nI could not find any refuting evidence — no intervening check, validation, or test that catches this case before it reaches the UI as a green success line. The finding's mechanism, line numbers, and consequence chain all verify against the actual source.

</details>

### 3. The auto-generated module dialog gives no way to leave an optional log-input slot blank when a curve of that name exists in the project, contradicting electrofacies/gmm_facies's documented "any unwanted curve slot is dropped" behavior

**Area:** UI/UX (E) / frontend wiring (D) — moduleDialog.ts

**Where:** src/ui/moduleDialog.ts lines 96, 98-104 (log_in `<select>` construction via `fillSelect(select, logChoiceNames(arg.default), arg.default)`), contrasted with lines 129-146 (the Mask `<select>` explicitly gets a `(none)` option)

**Evidence:** For every `log_in` arg — required or optional — the dropdown is populated only from `logChoiceNames`, i.e. real catalog curve names plus the manifest default; there is no blank/"(none)" entry the way the Mask selector (built two sections later, lines 129-146) explicitly adds. electrofacies_spec/gmm_facies_spec (facies.rs lines 44-47/182-185) mark CURVE2-5 optional and the module doc says "Any curve slot with no data is dropped" (and ROADMAP.md line 396/398: "leave a slot blank/absent and it's dropped"), but the only way a slot actually ends up NaN today is if the *selected mnemonic itself doesn't exist for that particular well* — a user cannot deliberately exclude, say, SP from the clustering feature space through the dialog when an SP curve exists in the project, since the select box literally has no way to express "no curve". This directly undercuts the one piece of UX flexibility the facies modules advertise as their headline behavior.

**Suggested fix:** Prepend a "(none)" option (mirroring the Mask selector) to every optional (`required: false`) log_in `<select>`, and have the run handler send an empty/omitted mnemonic for that arg so the backend resolves it as absent, matching what the module doc already promises.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the real files and traced the full path end-to-end; could not refute the finding.\n\n1. src/ui/moduleDialog.ts lines 98-104: every log_in <select> (required or optional) is filled only via fillSelect(select, logChoiceNames(arg.default), arg.default) — logChoiceNames (line 96) returns catalog curve names plus the default mnemonic if absent from the catalog. No blank/(none) option is ever injected here.\n2. Same file, lines 129-146 (rebuildMaskOptions): the Mask select explicitly builds a value=\"\" / \"(none)\" option and prepends it — confirming the described asymmetry is real, not a misreading.\n3. Line 299: logInputs[name] = select.value always yields a real mnemonic (never empty) because the select has no blank option — so a user cannot request 'no curve' for CURVE2-5 through the dialog when a curve of that name exists in the catalog.\n4. src-tauri/src/facies.rs: electrofacies_spec/gmm_facies_spec (lines 43-47 / 181-185) do declare CURVE2-CURVE5 as log_in(..., false) (optional); electrofacies_spec's doc string (line 33) literally says 'Any curve slot with no data is dropped'; prep_samples (lines 64-74) drops a slot only when its whole column is NaN.\n5. src-tauri/src/workflow.rs lines 140 and 198-204 confirm the backend already supports the fix's premise: an empty-string mnemonic falls through to an all-NaN column (matching modules.rs's comment 'missing optional inputs become all-NaN'), but the frontend never emits an empty string because no blank option exists in the select.\n6. Cross-checked the alternate UI surface, src/ui/workflowDialog.ts's logInControl (lines 287-303): same gap — no blank option for log_in, only maskControl gets '(none)'. This reinforces rather than undermines the finding (systemic, not a one-file oversight).\n\nOnly a trivial nit: ROADMAP.md lines 396/398 don't contain the literal phrase quoted in the finding ('leave a slot blank/absent and it's dropped') — the actual text is a close paraphrase ('RHOB/NPHI/DT/SP optional ... Missing any present curve → MISSING'). This doesn't change the substance. All load-bearing code citations (moduleDialog.ts 96/98-104/129-146, facies.rs 44-47/182-185) are accurate and the causal chain described (no way to blank an existing-in-catalog optional curve slot, contradicting the 'dropped' behavior the modules advertise) is verified true against the real source.

</details>

---

## Legacy multimin (multimin.rs)

### 1. Workflow Builder's step picker exposes the deprecated multimin module unfiltered/unlabeled, while SandiMin has no path into chains at all

**Area:** D. Frontend wiring / F. Cross-function (Workflow Builder chaining)

**Effort:** small (UI-only filter/label change) for the picker fix; separately deciding whether SandiMin should become chain-composable is a larger design question worth a ticket, not a quick fix

**Where:** src/ui/workflowDialog.ts:51,154-171,231 (unfiltered listModules()-driven step picker); src/ui/ribbon.ts:358-362,385-386,462-471 (the ribbon-only filter that hides multimin); src-tauri/src/workflow.rs:107,265-266 (chain dispatch goes solely through modules::list_modules()/run_module(), confirmed zero references to multimin2/run_multimin/SandiMin anywhere in workflow.rs)

**Evidence:** ribbon.ts deliberately filters 'multimin' out of both the Saturation dropdown (ADVANCED_MODULE_IDS, lines 358-362, applied at 385-386) and the Advance tab (its META group caption is literally '(hidden)', line 469, and that caption is excluded from groupOrder at 471) because SandiMin supersedes it (REVIEW.md:914-918 confirms this was a deliberate rename/removal decision). But workflowDialog.ts builds its '+ Add module' dropdown directly from the unfiltered listModules() array (line 51) grouped only by raw category (lines 156-171) -- no equivalent exclusion set -- so 'Multimin -- Mineral Inversion' appears as a normal, unlabeled <option> under the 'Saturation' <optgroup>, indistinguishable from sw_arch/sw_indo/sw_sim/sw_rtc/sw_imts. Step rendering (line 231) shows only spec.title verbatim, no legacy/deprecated marker anywhere in the step list either. Meanwhile SandiMin (multimin2.rs's run_multimin/multimin_library commands) is not a generic module and is not dispatchable from workflow.rs at all (grep confirms zero hits for multimin2/run_multimin/SandiMin in workflow.rs; chain steps go exclusively through modules::list_modules()/run_module(), workflow.rs:107,265-266). Net effect: a user building a brand-new chain today can only silently add the deprecated fixed 4-component solver -- SandiMin cannot be chained in any form -- with no UI signal that this differs from the SandiMin they'd run standalone. This goes beyond what REVIEW.md documents ('it still runs inside saved workflow chains' reads as passive backward-compatibility for existing docs, not 'freely addable, unlabeled, to brand-new chains'). Compounding the confusion: the two solvers' default output mnemonics are near-mirror-images of each other -- legacy PHIT_MM/VSH_MM/SWT_MM/RECON_ERR (multimin.rs:69-72) vs SandiMin's default MM_-prefixed MM_PHIT/MM_VSH/MM_SWT/MM_RECON (multimin2.rs:644-812) -- easy to mis-pick later in the Curve Catalog or a downstream step's input dropdown.

**Suggested fix:** Exclude 'multimin' from workflowDialog.ts's category listing too (mirroring ribbon.ts's filter), leaving it reachable only via loading a pre-existing saved workflow document; or at minimum append a '(legacy)' suffix to its title specifically in the step-picker and step-list rendering so it can't be mistaken for a current option.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every cited location in D:\XX. SandiBumi against the real source and could not refute any part of the finding: (1) workflowDialog.ts:51 calls listModules() with no filtering, and its category grouping (154-171) and step-title rendering (231) have zero references to 'multimin', 'legacy', or 'deprecated' anywhere in the file (grep confirmed). (2) ribbon.ts's ADVANCED_MODULE_IDS (358-362) explicitly includes 'multimin' with a comment describing it as intentionally hidden/superseded, applied only to the ribbon's category renderer (385-386), and the Advance tab's META entry captions it '(hidden)' while groupOrder (471) omits that caption entirely (462-471) — this filtering machinery lives only in ribbon.ts. (3) workflow.rs's chain dispatch goes solely through modules::list_modules() (107) and modules::run_module() (265-266), and grep confirms zero references to multimin2/run_multimin/SandiMin anywhere in workflow.rs. (4) modules.rs registers only 'multimin' (not 'multimin2'/'sandimin') in list_modules() and run_module()'s match statement — SandiMin (multimin2.rs) is reachable from the frontend only via dedicated commands (run_multimin, multimin_library, etc. in lib.rs) consumed by a standalone multiminDialog.ts, entirely outside the generic module-chain system. (5) The output-mnemonic contrast (PHIT_MM/VSH_MM/SWT_MM/RECON_ERR at multimin.rs:69-72 vs default MM_PHIT/MM_VSH/MM_SWT/MM_RECON via multimin2.rs's 'MM' default prefix and format!() curve names) is accurate. (6) REVIEW.md:914-918 quote matches verbatim and the finding's interpretation of it is defensible. (7) sw_arch/sw_indo/sw_sim/sw_rtc/sw_imts all share category 'Saturation' with multimin, confirming it would appear indistinguishably among them in the step picker. Every load-bearing claim and line citation checked out; no evidence was fabricated, exaggerated, or mischaracterized.

</details>

### 2. RECON_ERR is a near-guaranteed ~0 (uninformative) QC signal whenever exactly 3 of the 4 tools are live -- the common one-log-missing case (e.g. no PEF)

**Area:** B. Backend correctness at singularities / C. Domain correctness (QC-curve semantics)

**Effort:** small-to-medium (logic change to gate/NaN the curve plus one new test; a full fix that makes RECON_ERR degrees-of-freedom-aware would be medium)

**Where:** src-tauri/src/multimin.rs:123-155 (gate + RECON_ERR computation), doc claim at line 34, test gap (only multimin_skips_when_too_few_tools and multimin_skips_two_tools_for_four_unknowns cover edges; neither exercises n_tools==3's RECON_ERR value)

**Evidence:** With n_tools == 3, the assembled system is 3 weighted tool rows + 1 weighted unity row (lines 134-143) -- exactly 4 equations for the 4 unknowns, i.e. square, not over-determined. Whenever the implied volumes come out non-negative, the NNLS solve is provably equal to the exact solution of that square system (residual 0 on every row it was built from), so RECON_ERR -- computed only over the n_tools rows, lines 149-155 -- reads ~0 regardless of whether the chosen endpoints are physically right for the rock. This is the exact 'solver-path artifact ... RECON_ERR reads ~0, so the QC curve would flag nothing' failure mode the code's own comment (lines 123-128) names as the reason to skip when n_tools < 3; raising that gate from 2 to 3 (already fixed per AUDIT-2026-07-20.md and confirmed current in this code) fixed the arbitrary-vertex volumes problem but left this adjacent QC-blindness problem at the new boundary untouched, since n_tools==3 is 'determined' in the linear-algebra sense but still square/exact-fit in the diagnostic sense. RECON_ERR only becomes genuinely informative once all 4 tools are present (4 tool rows + unity = 5 equations for 4 unknowns, truly over-determined) -- the only configuration the existing test multimin_recovers_known_clean_wet_sand exercises (uses RHOB+NPHI+DT+PEF). The module's own doc string (line 34) makes the unqualified claim 'RECON_ERR (RMS log-reconstruction error in sigma units -- high where the model fails)', which silently does not hold whenever exactly one of the four logs is missing -- not a rare corner case, since PEF in particular is commonly absent from older/budget logging strings in this user's own Mahakam Delta context.

**Suggested fix:** Either NaN/flag RECON_ERR (or emit a distinct 'under-constrained' indicator) whenever n_tools == 3 so it cannot be read as a genuine misfit statistic, or explicitly caveat the module doc that RECON_ERR is only meaningful at the full 4-tool count; add a regression test asserting RECON_ERR stays near-zero at n_tools==3 even when endpoints are deliberately wrong, to keep the limitation visible in the suite.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read src-tauri/src/multimin.rs in full (D:\XX. SandiBumi\src-tauri\src\multimin.rs) and confirmed every code-level claim verbatim: the gate is `if n_tools < 3 { continue; }` (line 130), so n_tools==3 proceeds to solve; the assembled system at n_tools==3 is exactly 3 weighted tool rows + 1 weighted unity row (lines 134-143) against N=4 unknowns, i.e. square; RECON_ERR (lines 149-155) sums squared residuals only over `0..n_tools`, excluding the unity row; the doc string's unqualified RECON_ERR claim is at line 34 verbatim; and the only three tests (lines 335-398) cover n_tools=1, n_tools=2, and n_tools=4 (multimin_recovers_known_clean_wet_sand uses RHOB+NPHI+DT+PEF) — none exercise n_tools==3, confirming the stated test gap. Cross-checked AUDIT-2026-07-20.md and multimin2.rs and confirmed the gate-raise (2→3) history and the analogous min_tools rule cited as context are accurate.

Beyond re-deriving the algebra, I built an independent Python replica of the exact weighted-row/unity/NNLS construction (using real scipy.optimize.nnls, not just an unconstrained solve) and forward-modeled a known rock through RHOB+NPHI+DT (PEF missing, the cited real-world case) then inverted with deliberately wrong SAND endpoints. Result: across a wide, physically plausible range of endpoint error, RECON_ERR stayed at machine epsilon (~1e-14) while recovered CLAY volume was off by up to 100% relative (0.15 true vs ~0 recovered) — a completely silent QC failure. Only when the endpoint error became large enough to force NNLS's non-negativity constraint to actually bind (breaking the square system's exact-fit property) did RECON_ERR rise above ~0.1. Repeating the same wrong-endpoint test with all 4 tools present (genuinely over-determined, 5 eq/4 unknowns) gave RECON_ERR≈0.51, a real detectable signal — confirming the finding's core distinction that RECON_ERR is only diagnostic at the full 4-tool count and is a near-guaranteed uninformative ~0 at n_tools==3 for the common (non-extreme) range of wrong-endpoint cases. I found no flaw in the finding's reasoning, citations, or characterization; it survives independent verification against the real file and independent numerical experiment, so I was unable to refute it.

</details>

---

## Pay summary & cutoff sweep

### 1. Cutoff-sweep NET/HPV/NTG isn't clamped to the zone/DST overlap — it re-introduces the exact 'step bleed past boundary' bug already fixed in run_pay_summary

**Area:** Backend domain correctness (workflow.rs) — run_cutoff_sweep / compute_sweep

**Where:** src-tauri/src/workflow.rs: compute_sweep's accumulation loop (lines 757-773, esp. line 767 `let h = step[i] as f64;`) fed by the membership mask built in run_cutoff_sweep (lines 980-988, `if d >= ztop && d < zbot && in_dst(d)`). Contrast with the correct geometric clamp in run_pay_summary (lines 592-623, esp. 598-609: `lo = s_top.max(zone.top_depth); hi = s_bot.min(zone.bottom_depth); h = hi-lo`).

**Evidence:** run_pay_summary tests a sample's *clamped overlap* with the zone (`h = hi - lo`, `if h <= 0.0 { continue }`) so net can never exceed gross — this was an explicit, deliberate fix (ROADMAP: 'sample thickness clamped to zone overlap... no step bleed past base', test `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`, workflow.rs lines 1300-1350). compute_sweep does not carry this fix: it decides membership from the sample's TOP depth only (`depth[i] >= ztop && depth[i] < zbot`, and separately `in_dst(d)`) and then adds the sample's FULL, unclamped `step[i]` to net/hpv. Reproducing the exact fixture from the pay_summary test — depths [1000,1001,1002,1003], step 1.0, zone Z1=[1000,1001.5) — a permissive cutoff makes samples at depth 1000 and 1001 both 'included' (1001 < 1001.5), so compute_sweep sums their full 1.0 m steps each → net = 2.0 against a geometric gross of 1.5 → NTG = 2.0/1.5 ≈ 1.33, a value greater than 1, which is geometrically impossible and disagrees with what run_pay_summary reports for the identical well/zone/cutoff. The same unclamped test applies to the DST-interval filter (`in_dst`), which is the primary real use case documented in docs/research_2026-07/ref_kkt_onwj_wave_e.md item 21 ('filtered on DST intervals only') — DST/perforation picks are independent of log sampling and routinely don't land on exact sample depths, so a DST interval boundary mid-sample causes that sample's full step to be either fully counted or fully dropped instead of its actual overlap fraction. The existing cutoff-sweep tests (`cutoff_sweep_ntg_and_dst_mask`, lines 1090-1110) only use fixtures where zone/DST boundaries are exactly sample-aligned, so they don't exercise this case. The code's own comment claims the sweep engine mirrors run_pay_summary ('Single-sourced through classify_sample so the sweep engine below applies identical cutoff logic', line ~523) — true for the per-sample pass/fail classification, false for the geometric integration that turns classified samples into net/hpv/ntg.

**Suggested fix:** In run_cutoff_sweep, replace the boolean `included: Vec<bool>` mask with a per-sample clamped-overlap thickness (same `lo.max/hi.min` computation as run_pay_summary, further intersected with any DST interval overlap when `dst` is Some), and pass that `Vec<f64>` into compute_sweep so `net += h_clamped` instead of the raw `step[i]`.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently read D:\XX. SandiBumi\src-tauri\src\workflow.rs in full for the relevant regions and confirmed every specific claim:\n\n- run_pay_summary (lines 592-623) computes h = hi.min - lo.max (geometric overlap with the zone), skips h<=0, so net is clamped and can never exceed gross.\n- compute_sweep's accumulation loop (line 767: `let h = step[i] as f64;`) uses the raw, unclamped sample step with no geometric intersection against zone/DST boundaries.\n- run_cutoff_sweep's membership mask (line 984: `if d >= ztop && d < zbot && in_dst(d)`) tests only the sample's point depth, not overlap fraction.\n- Reproduced the exact fixture cited (depths 1000/1001/1002/1003, step 1.0, zone [1000,1001.5)): run_cutoff_sweep includes samples at 1000 and 1001 in full (1 m each) giving net=2.0 vs geometric gross 1.5 → NTG≈1.33 (impossible, >1), while run_pay_summary on the same fixture correctly gives net=1.5, NTG=1.0. The two engines disagree on identical well/zone/cutoff.\n- Confirmed existing test cutoff_sweep_ntg_and_dst_mask (lines 1090-1110) only uses sample-aligned fixtures (step=1.0, gross exact multiples), so it doesn't exercise the misalignment case.\n- Confirmed the code comment (~line 523) and ROADMAP.md line 581 claim byte-identical logic reuse from pay-summary, which is true for classify_sample's per-sample pass/fail decision but false for the geometric integration into net/hpv/ntg — exactly the distinction the finding makes.\n- Confirmed via docs/research_2026-07/ref_kkt_onwj_wave_e.md that DST-interval filtering (independent of log sample depths) is the documented real use case, so this is not merely a hypothetical edge case.\n\nAll line numbers, code quotes, and the reproduction scenario in the finding match the actual file. The suggested fix (per-sample clamped-overlap thickness feeding compute_sweep, intersected with DST overlap) is a sound direction consistent with run_pay_summary's existing pattern. No mitigating logic (depth-grid snapping, hidden clamp) was found elsewhere that would neutralize the bug.

</details>

### 2. Compute Summary (the FLAG_*-writing pay-summary run) never calls recordProcess — the one persisting write in these two tools leaves no trace in Processing History

**Area:** Frontend wiring — summaryDialog.ts

**Where:** src/ui/summaryDialog.ts, runBtn click handler (lines 75-100) and file-level imports (lines 1-3): no import of `recordProcess`, no call to it anywhere in the file — only `bumpDataVersion()` (line 94) is called after a successful run.

**Evidence:** Every comparable persisting action in this codebase calls recordProcess right after the write: src/ui/mlDialog.ts:392, src/ui/monteCarloDialog.ts:310, and src/ui/cutoffDialog.ts:769 (which logs its own persisting action, 'Saved default cutoffs', via the same file that hosts the sibling Cutoff Sensitivity pane). The generic 'module' dock pane gets this for free via workspace.ts's `onRunComplete` hook (workspace.ts lines 355-369: `recordProcess("Module", ...); this.notifyDataChanged();`), but the 'paysummary' and 'cutoff' cases (workspace.ts lines 322-333) are wired with no such hook, so each dialog is on its own to log — and summaryDialog.ts simply never does. This isn't a borderline case: the explicit Cutoffs & Pay Summary run (non-skip_version, non-stats_only path in run_pay_summary) writes and *versions* FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY into a new PAYFLAG log-set version with the cutoffs recorded in provenance — a write the project's own commentary says was upgraded specifically 'exactly like every other module output... so a re-run keeps history' (REVIEW.md lines 244-255). 'Every other module output' does get a History-panel entry; this one doesn't. Confirmed against REVIEW.md's own audit of Processing-History coverage (lines 224-233): the explicit list of what got logging added (imports, equation runs, ML, Monte Carlo, workflow chains, log-set restore/delete, zone edits, manual tops, 'cutoff-default saves', map-polygon assignment) never mentions pay-summary runs.

**Suggested fix:** Import recordProcess in summaryDialog.ts and call it after a successful, non-stats_only run, e.g. `recordProcess("Pay Summary", \`VSH≤${vsh} PHIE≥${phie} SWE≤${swe}: ${rows.length} rows across ${wellIds.length} well(s)\`)`, mirroring mlDialog.ts/monteCarloDialog.ts.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every specific claim against the actual source; could not refute any part of it.

1. D:/XX. SandiBumi/src/ui/summaryDialog.ts (140 lines, read in full): imports at lines 1-3 are only from "../ipc", "../state", "./modal" — no recordProcess import. The runBtn click handler (lines 75-100) calls runPaySummary, renderTable, setStatus, and bumpDataVersion() at line 94 — recordProcess is never called anywhere in the file.

2. Contrast confirmed exactly as cited:
   - src/ui/mlDialog.ts:392 calls recordProcess("ML", ...) after a successful run.
   - src/ui/monteCarloDialog.ts:310 calls recordProcess("Monte Carlo", ...).
   - src/ui/cutoffDialog.ts:769 (inside saveDefaultBtn handler) calls recordProcess("Cutoffs", "Saved default cutoffs...") right after saveDocument — this is cutoffDialog's own persisting action (cutoff defaults), separate from the pay-summary write.
   - src/ui/workspace.ts lines 322-333: the "paysummary" and "cutoff" dock-pane cases are wired with no onRunComplete/logging hook, while the "module" case (lines 355-369) gets recordProcess("Module", ...) + this.notifyDataChanged() for free via an onRunComplete callback.

3. Backend confirmed in src-tauri/src/workflow.rs: run_pay_summary (line 473) — when req.stats_only is false and req.skip_version is false (the else branch, lines 546-573) — creates a new "PAYFLAG" log_set via equations::create_log_set with provenance (module, cutoffs in params_json, inputs), then writes FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY via write_computed_curves_versioned. This is a genuine persisting/versioned write, not a transient computation.

4. Confirmed summaryDialog.ts's runPaySummary call (lines 85-91) omits both skip_version and stats_only; PaySummaryRequest in workflow.rs marks both #[serde(default)] (false), so this UI button — "Compute Summary" — is exactly the explicit non-skip_version/non-stats_only path that performs the versioned write. run_cutoff_sweep (workflow.rs:873) is confirmed read-only (no writes anywhere in its body), consistent with the finding's framing that the one persisting write across these two backend functions lives in run_pay_summary.

5. bumpDataVersion (src/state.ts:93) and notifyDataChanged (workspace.ts:1066) only touch a UI refresh counter — no connection to recordProcess/Processing History, so there's no hidden side-channel logging that would make the missing call harmless.

6. REVIEW.md quotes check out verbatim: lines 224-233 list exactly what got Processing-History coverage (imports, equation/ML/Monte Carlo/workflow runs, log-set restore/delete, zone edits, manual tops, cutoff-default saves, map-polygon assignment) with no mention of pay-summary runs; lines 244-255 describe the PAYFLAG versioning upgrade with the "exactly like every other module output... so a re-run keeps history" language quoted in the finding.

Every line number, file path, and quoted string in the finding matches the real code. The suggested fix (import recordProcess in summaryDialog.ts, call it after a successful non-stats_only run) is consistent with the sibling patterns in mlDialog.ts/monteCarloDialog.ts. I found no inaccuracy or overstatement to refute — the finding stands as CONFIRMED.

</details>

### 3. run_pay_summary and run_cutoff_sweep abort the entire multi-well batch on one well's fetch/zone error, unlike run_workflow_module's per-well isolation

**Area:** Backend (Rust) — error handling / batch semantics

**Where:** src-tauri/src/workflow.rs: run_pay_summary lines 487-492 (`equations::fetch_curve_frame(...).map_err(|e| e.to_string())?;` then `db::list_zones(...).map_err(|e| e.to_string())?;`); run_cutoff_sweep lines 907-924 (same `fetch_curve_frame(...)... ?` then `db::list_zones(...)... ?`). Contrast with run_workflow_module_into (lines 153-301), which wraps each well's compute in a closure and converts any `Err` into `Outcome::Failed(e)` scoped to that well only (lines 283-286), letting every other well's result stand.

**Evidence:** Both PaySummaryRequest/CutoffSweepRequest handlers iterate `for well_id in &req.well_ids { ... ?; ... }` inside a function returning `Result<Vec<_>, String>`. The `?` on a per-well DB call (fetch_curve_frame or list_zones) propagates out of the whole function, discarding every row already pushed for prior wells and skipping all remaining wells — a single bad well zeroes the entire response. This is inconsistent with the module runner in the same file, which explicitly isolates per-well failures so a batch of hundreds of wells degrades gracefully (one Failed entry) rather than failing wholesale. Both audited tools are explicitly used at field scale — the Field Dashboard (~540 wells, per ROADMAP) drives run_pay_summary across every well, and the Cutoff Sensitivity pane runs run_cutoff_sweep across whatever wells are checked — so this all-or-nothing behavior is reachable at exactly the scale where a single well's transient/edge-case DB read failure is least tolerable: the frontend then shows one generic 'Summary failed: {err}' / 'Sweep failed: {err}' for every well instead of surfacing the N-1 good wells and flagging just the bad one.

**Suggested fix:** Mirror run_workflow_module_into's per-well isolation: wrap each well's fetch/zone/compute steps in a closure whose `Err` becomes a skipped well (optionally collected into a separate error list) via `continue`, rather than propagating through `?` and aborting the whole function.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the actual source in D:\XX. SandiBumi\src-tauri\src\workflow.rs (the SandiBumi repo).

1. run_pay_summary (fn signature at line 473: `pub fn run_pay_summary(...) -> Result<Vec<PaySummaryRow>, String>`) loops `for well_id in &req.well_ids` (line 477) and at line 487-488 does:
   `let (depth, columns) = equations::fetch_curve_frame(&conn, well_id, &curve_names).map_err(|e| e.to_string())?;`
   and at line 492:
   `let mut zones = db::list_zones(&conn, well_id).map_err(|e| e.to_string())?;`
   Both `?` operators are inside the per-well loop but propagate out of the whole function (which returns a single `Result`, not per-well results). A failure on well N discards `all_rows` already pushed for wells 1..N-1 and skips all remaining wells — exactly as claimed. Line numbers match the finding precisely (487-492).

2. run_cutoff_sweep (signature at line 873: `pub fn run_cutoff_sweep(...) -> Result<CutoffSweepResult, String>`) has the identical pattern: line 907-908 `fetch_curve_frame(...).map_err(|e| e.to_string())?;` and line 924 `db::list_zones(...).map_err(|e| e.to_string())?;`, both inside `for well_id in &req.well_ids` (line 898), both able to abort the whole function and discard the `series` Vec built for prior wells. Line numbers match the finding precisely (907-924).

3. Contrast confirmed: run_workflow_module_into (lines 100-303) defines an `Outcome` enum (Skipped/Failed/Computed) at line 153, wraps each well's fetch/compute steps in a `compute()` closure (line 175) returning `Result<...>`, and explicitly converts `Err` into `Outcome::Failed(e)` scoped to that one well only via `match compute() { Ok(...) => Outcome::Computed{...}, Err(e) => Outcome::Failed(e) }` at lines 283-286 inside a `par_iter().map()` — so one well's failure never aborts the batch; every other well's outcome stands. This is a materially different (per-well-isolated) error-handling pattern from the two audited functions.

4. Frontend evidence also checks out: src/ui/summaryDialog.ts line 96 shows `resultBox.textContent = \`Summary failed: ${err}\`;` inside the catch block wrapping the `runPaySummary` invoke call, and src/ui/cutoffDialog.ts line 562 shows `readout.textContent = \`Sweep failed: ${err}\`;` in the analogous catch block — confirming a single well's DB error would surface as one generic failure message for the whole request, not per-well granularity.

Every specific claim in the finding — file, function names, exact line numbers, the `?`-propagation mechanism, the return-type mismatch (single `Result` vs per-well `Vec<Outcome>`), and the contrasting isolation pattern in run_workflow_module_into — is verified true against the current source. I could not find any surrounding try/catch, per-well isolation wrapper, or continue-on-error logic in either audited function that the finding might have missed. The finding stands as accurate.

</details>

---

## Monte Carlo

### 1. PERM cutoff is silently ignored whenever PERM is produced by the Monte Carlo chain itself (not read from the DB)

**Area:** Backend correctness (dimension B/C)

**Effort:** small

**Where:** src-tauri/src/montecarlo.rs: build_plans() lines 330-422 (esp. the produced/external split, lines 337-356), and run_monte_carlo() lines 494-495 (`has_perm_cut`)

**Evidence:** `has_perm_cut` is computed as `req.perm_min.is_some() && raw_pool.get("PERM").map(|c| c.iter().any(|v| !v.is_nan())).unwrap_or(false)`. `raw_pool` is populated only from `external` curves — LogIn mnemonics not produced by any step in the chain (build_plans lines 345-356) — fetched once via `equations::fetch_curve_frame` before any realization runs. If the requested chain includes a permeability module (perm_coates, perm_wyllie_rose, or perm_transform — all register `log_out("PERM", ...)` in modules.rs around lines 2032/2055/2073/2091/2111/2128), then "PERM" is in the `produced` set and is deliberately excluded from `external`/`raw_pool`; it only ever exists in the per-realization in-memory `pool` built later by run_realization. So `raw_pool.get("PERM")` is `None`, `has_perm_cut` evaluates to `false` regardless of `req.perm_min`, and the PERM branch inside `zone_metrics` (`if has_perm_cut { pay = pay && ... }`) never executes for any realization — every sample passes the PERM test unconditionally. Net pay/NTG/avg_phie/avg_swe/HPV P10/P50/P90 come back inflated with the user's PERM cutoff silently dropped, no error or warning surfaced (monteCarloDialog.ts just shows "Done in N ms"). The sibling deterministic tool `run_pay_summary` (workflow.rs line 474, 508) does not have this bug because it always re-reads PERM straight from the DB (`fetch_curve_frame(&conn, well_id, &["VSH","PHIE","SWE","PERM"])`) after a prior deterministic run has already persisted PERM to computed_curves — an option Monte Carlo's design (everything in-memory, nothing written to computed_curves, per its own module doc comment lines 6-8) does not have, and never compensated for.

**Suggested fix:** Determine has_perm_cut from the per-realization `pool` (post chain-execution) rather than from the pre-execution `raw_pool`, e.g. check after running at least one realization's outputs, or simply check both raw_pool and the chain's declared LogOut set for "PERM".

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently traced the full code path in the real files (D:\XX. SandiBumi\src-tauri\src\montecarlo.rs, modules.rs, workflow.rs, src\ui\monteCarloDialog.ts) and could not refute the finding — it holds exactly as described, with line numbers matching the current source almost precisely.\n\nKey confirmations:\n1. montecarlo.rs build_plans() (330-422): `produced` is collected from every step's LogOut args across the whole chain (337-343); `external`/`raw_pool` explicitly exclude anything in `produced` (346-370). So a chain-produced PERM never enters raw_pool, regardless of step order or per-realization success.\n2. modules.rs confirms perm_wyllie_rose, perm_coates, perm_transform all declare log_out(\"PERM\", ...) (lines 2032, 2073, 2111), so including any of them in a saved workflow chain puts PERM into `produced`.\n3. run_monte_carlo() (494-495): `has_perm_cut = req.perm_min.is_some() && raw_pool.get(\"PERM\")...` is computed once, pre-loop, strictly from raw_pool — so it's unconditionally false whenever PERM is chain-produced, irrespective of req.perm_min.\n4. zone_metrics() (245-251): when has_perm_cut is false the PERM branch is skipped entirely, even though run_realization()'s per-realization `pool` (432+) does contain real computed PERM values that are simply never consulted, because has_perm_cut is frozen before the parallel realization loop and reused verbatim (512) rather than being recomputed against `pool`.\n5. The claimed asymmetry with run_pay_summary checks out: workflow.rs unconditionally fetches VSH/PHIE/SWE/PERM straight from the DB (474/488/508) and derives has_perm_cut from that live-fetched data, so it has no equivalent gap.\n6. UI: monteCarloDialog.ts only surfaces \"Done in N ms\" (line 307) with no warning; the existing hint text (\"PERM cutoff applies only where a PERM curve exists\", line 265) doesn't distinguish genuinely-absent PERM from chain-produced-but-silently-ignored PERM, so nothing signals the dropped cutoff to the user.\n\nNo mitigating logic exists anywhere in the file (no forced re-fetch of cutoff-relevant curves regardless of production status, no recomputation of has_perm_cut against the post-chain pool). The scenario is realistic and reachable through normal UI usage (any saved workflow chain containing a PERM module, combined with setting PERM ≥ cutoff in the Monte Carlo dialog), not a contrived edge case. The finding is accurate; I was unable to refute it.

</details>

### 2. montecarlo.rs's own from-scratch chain executor misses two correctness behaviors the real chain runner enforces: MASK blanking and computed_only provenance resolution

**Area:** Backend correctness (dimension B)

**Effort:** medium

**Where:** src-tauri/src/montecarlo.rs: build_plans() lines 330-422 and run_realization() lines 424-457, contrasted with src-tauri/src/workflow.rs lines 227-278 (MASK) and lines 205-224 (computed_only re-resolution), and src-tauri/src/chain.rs line 186/192

**Evidence:** (1) MASK: a workflow step's bad-hole/washout mask is stored as `step.opts["MASK"]` (set via workflowDialog.ts lines 361-364/489-597) and, in the real chain runner, is resolved and used to NaN-blank flagged samples in the module's INPUTS before the run and in its OUTPUTS after the run (workflow.rs lines 227-278, invoked per chain step from chain.rs line 186 `opts: step.opts.clone()` / line 192 `run_workflow_module_into`). Monte Carlo's chainSelect handler (monteCarloDialog.ts lines 88-95) copies a saved chain's `opts` verbatim into its own `steps`, so a MASK setting from a saved workflow rides along into McRequest.steps unchanged — but montecarlo.rs's build_plans/run_realization simply merges `step.opts` into a generic `opts` map passed to `modules::run_module` (which never reads a "MASK" key at all — confirmed by grep, MASK is handled purely as caller-side logic in workflow.rs, never inside modules.rs). So loading an existing masked chain (e.g. gr_normalize/log_predict with MASK=BADHOLE, the exact scenario documented in ROADMAP.md/REVIEW.md as the reason MASK exists) into Monte Carlo silently runs every realization over flagged bad-hole/washout samples with no indication to the user. (2) computed_only: gascorr's FTEMP/FPRESS args are declared `log_in_computed(...)` (modules.rs lines 1372-1375) specifically because a raw-imported curve of the same mnemonic can carry the wrong unit (degF/kPa vs the required degC/psi) — the real chain runner re-resolves these through `fetch_computed_only_aligned` (workflow.rs lines 205-224) instead of the general-precedence `fetch_curve_frame`. montecarlo.rs's build_plans treats every LogIn arg identically (lines 345-356, 402-408) and fetches all external inputs through the single generic `equations::fetch_curve_frame` call (line 359), with no check of `a.computed_only` anywhere in the file. A Monte Carlo chain that includes gascorr on a well carrying both a raw-imported FTEMP/FPRESS and an already-computed one under the same mnemonic risks silently mixing in the wrong-unit raw curve.

**Suggested fix:** Port (or share via a common helper) the MASK pre/post-blanking block and the computed_only re-resolution branch from workflow.rs into montecarlo.rs's build_plans/run_realization so a Monte Carlo run stays behaviorally identical to running the same chain through the Workflow Builder.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I could NOT refute this finding — every piece of evidence checks out against the actual source in D:\XX. SandiBumi (the SandiBumi working tree; the folder is still named Arshilla per prior renaming, per user memory).

Verified independently:

1. montecarlo.rs (330-422 build_plans, 424-457 run_realization) — read in full. Confirmed: opts are merged as `spec defaults -> step.opts` (lines 392-400) with zero special-casing of any key; log_args are all fetched through one generic path (`equations::fetch_curve_frame` at line 359, then `pool.get(mnem)` at line 437) regardless of `a.computed_only`; a grep for "MASK|mask|computed_only" across the entire file returns zero matches. There is no mask-blanking step anywhere (pre- or post-module) and no branch that treats a `computed_only` LogIn arg differently from a normal one.

2. workflow.rs — read lines 1-300. Confirmed real chain runner does two things montecarlo.rs lacks:
   - Lines 205-224: for `a.computed_only` LogIn args, re-resolves via `equations::fetch_computed_only_aligned` instead of trusting the generic frame fetch — exactly the gascorr FTEMP/FPRESS unit-contract case.
   - Lines 227-278: resolves `req.opts.get("MASK")` (line 234), blanks flagged samples in the module's `logs` inputs before the run (253-263) and in `outputs` after the run (270-278).

3. chain.rs lines 181-192: confirmed `RunModuleRequest.opts: step.opts.clone()` (line 186) feeds into `workflow::run_workflow_module_into` (line 192) — i.e., MASK set on a saved chain step really does reach and get enforced by the real workflow/chain path.

4. modules.rs: confirmed gascorr's FTEMP/FPRESS args are declared via `log_in_computed(...)` at lines 1374-1375 (computed_only:true, per the `log_in_computed` helper at line 106). Confirmed zero occurrences of "MASK" anywhere in modules.rs — MASK truly is caller-side-only logic, never module-side.

5. monteCarloDialog.ts lines 81-104 (chainSelect change handler): confirmed a selected saved chain's steps are mapped with `opts: s.opts ?? {}` (line 94) — a verbatim copy, no filtering of MASK or anything else. Grep for "MASK" in this file: zero matches — nothing strips or special-handles it client-side either.

6. workflowDialog.ts: confirmed MASK is a first-class per-step option — `step.opts.MASK` set/cleared at lines 361-364 and again at lines 489/596-597, matching the cited evidence precisely.

7. lib.rs: confirmed the `run_monte_carlo` Tauri command (lines 551-554) is a direct passthrough to `montecarlo::run_monte_carlo(&db.0, &req)` with no intervening MASK/computed_only handling at the command boundary either.

All cited line numbers were accurate or off by at most a line or two (function boundaries matched exactly: build_plans@330, run_realization@424, fetch_computed_only_aligned block@205ff, mask block@227ff, chain.rs opts.clone@186/run_workflow_module_into@192). The described exploit scenario (loading a saved MASK-bearing chain into Monte Carlo silently drops the blanking behavior; a chain with gascorr risks mixing in a wrong-unit raw FTEMP/FPRESS curve) is a real, verified behavioral divergence between montecarlo.rs's from-scratch executor and workflow.rs's real chain runner — not a paraphrase error or misreading. Default-refute posture does not apply here; the finding survives independent verification and should be confirmed=true.

</details>

### 3. Monte Carlo's HPV histogram canvas never repaints on a live theme swap or panel resize, unlike every sibling Canvas-2D dock pane

**Area:** Frontend wiring / UI-UX (dimension D/E)

**Effort:** small

**Where:** src/ui/monteCarloDialog.ts (drawHistogram(), lines 403-474; renderResults(), lines 340-397; dispose returns a no-op at line 322), contrasted with src/ui/cutoffDialog.ts lines 824-829, src/ui/histogramPanel.ts lines 698/705, src/ui/pickettPanel.ts lines 356/362

**Evidence:** monteCarloDialog.ts's canvas reads theme colors live via `cssVar()` (getComputedStyle) inside `drawHistogram`, but that function is only invoked from `renderResults` (after a run) and from the row-click handler (line 377) — there is no `appState.themeVersion.subscribe(...)` anywhere in the file, and no `attachResizeRedraw` (the shared canvas-resize helper from plotCanvas.ts). Every other Canvas-2D dock pane in this codebase wires both: cutoffDialog.ts line 825 `attachResizeRedraw(canvas, redraw)` and line 828 `appState.themeVersion.subscribe(() => redraw())`, with the accompanying comment "Repaint on live theme swaps like every other Canvas-2D pane, else the plot keeps the old palette until an unrelated interaction; the returned unsub is released in dispose" — the same failure mode this pane exhibits. histogramPanel.ts and pickettPanel.ts follow the identical pattern. Since Monte Carlo is hosted as a persistent dock pane (not a transient modal), a user who runs it, leaves it open, and then toggles the app theme is left with a histogram frozen in the old palette (and not reflowed if the pane is resized) until they happen to click a different result row or rerun.

**Suggested fix:** Wire `appState.themeVersion.subscribe(() => drawHistogram(canvas, currentZone))` and `attachResizeRedraw(canvas, ...)` the same way cutoffDialog.ts does, disposing both in the pane's `dispose()` (currently a no-op).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified directly against D:\XX. SandiBumi\src\ui\monteCarloDialog.ts: renderResults spans exactly lines 340-397, drawHistogram exactly lines 403-474, dispose at line 322 is a literal no-op `() => {}`. grep confirms drawHistogram has exactly two call sites (line 377 row-click, line 396 end of renderResults) and zero occurrences of `themeVersion` or `attachResizeRedraw`/`plotCanvas` import anywhere in the file; the file's only `../state` import is `{defaultRunWellIds, filterByActiveGroup}` (appState is never referenced). Cross-checked cutoffDialog.ts lines 825/828 which match the finding's quoted comment verbatim, and histogramPanel.ts/pickettPanel.ts which do carry the identical attachResizeRedraw+themeVersion.subscribe pattern (at lines 698-699 and 356-357 respectively — the finding's secondary line numbers 705/362 are each off by ~5-6 lines, a minor citation slip that doesn't affect the substance). Confirmed appState.themeVersion is a real live-swap signal shared by correlationPanel.ts, logViewPanel.ts, and mapPanel.ts, and that workspace.ts hosts Monte Carlo as a persistent singleton dock pane whose dispose() only fires on pane close, never on theme swap, with no compensating global resize/theme sweep. No evidence contradicts the finding; it is confirmed.

</details>

### 4. summarize() collapses an all-missing metric to Pctl::default() (all zeros) instead of NaN, unlike its own percentile() helper

**Area:** Backend correctness / NaN-as-missing discipline (dimension B)

**Effort:** small

**Where:** src-tauri/src/montecarlo.rs: summarize() lines 286-302 (esp. the early return at line 288-290), contrasted with percentile() lines 271-284 which explicitly returns `f32::NAN` for an empty input

**Evidence:** `zone_metrics` deliberately reports `avg_phie`/`avg_swe` as `f32::NAN` for a realization where that zone had zero net pay (lines 265-266: `if net > 0.0 { ... } else { f32::NAN }`), so as not to report a bogus average over an empty interval. But when EVERY realization for a well/zone has zero net pay (a genuinely dry zone, or a well missing the VSH/PHIE/SWE curves the chain needs), `summarize()`'s input vector is all-NaN; its early-return path (`if finite.is_empty() { return Pctl::default(); }`) yields a Pctl with p10=p50=p90=mean=sd=0.0 (`#[derive(Default)]` on f32 fields), rather than propagating NaN the way `percentile()` itself does for an empty slice. The frontend's `fmt()` (monteCarloDialog.ts line 333-335: `Number.isFinite(v) ? v.toFixed(dp) : "—"`) then renders this as "0.00" — a real-looking measured value — instead of the "—" no-data marker it shows for a true NaN, in exactly the kind of all-missing-input/singularity case this codebase's own convention treats as the highest-risk defect class. Neither existing unit test (hpv_distribution_is_ordered_and_reproducible, zero_variance_param_collapses_distribution) exercises a dry/no-data zone, so this isn't caught by the test suite.

**Suggested fix:** Have summarize()'s empty-input branch return a Pctl with NaN fields (matching percentile()'s own convention) instead of Pctl::default(), so a genuinely no-data metric renders as "—" in the UI instead of a misleading "0.00".

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual files at D:\XX. SandiBumi\src-tauri\src\montecarlo.rs and D:\XX. SandiBumi\src\ui\monteCarloDialog.ts. Every line reference in the finding checks out exactly: summarize() (lines 286-302) early-returns Pctl::default() (all-zero, since Pctl derives Default) when its finite-filtered vector is empty (lines 288-290), while its sibling percentile() (lines 271-284) explicitly returns f32::NAN for the same empty-input case (line 274) — a genuine internal inconsistency. zone_metrics() (lines 262-268, esp. 265-266) deliberately emits f32::NAN for avg_phie/avg_swe when net<=0 / sum_phie_w<=0, so a zone with zero net pay in every realization (dry zone, or a well missing VSH/PHIE/SWE) produces an all-NaN vector for exactly those two metrics, which is what reaches summarize()'s buggy branch — net/ntg/hpv are unaffected since those legitimately stay 0.0 rather than NaN. The frontend fmt() (lines 333-335) is exactly Number.isFinite(v) ? v.toFixed(dp) : "—", and I confirmed renderResults() has no guard that would intercept a dry-zone avg_phie/avg_swe before display — so the UI would show a misleading "0.000" instead of the "—" no-data marker it shows for a true NaN. I also read both existing unit tests and confirmed neither exercises a dry/no-data zone (both use a seed_well() that is deliberately always-productive, with assertions requiring p50 > 0 for net/avg_phie/avg_swe/hpv). I found no mitigating logic, alternate check, or test coverage anywhere that would refute the claim. This is a real, verified bug matching the finding's description precisely.

</details>

---

## ML bridge

### 1. run_ml has no bad-hole/flag MASK support at all, unlike every module wired through workflow.rs

**Area:** Backend correctness (dimension B/F) — MASK consumption

**Effort:** medium

**Where:** src-tauri/src/ml.rs (MlRequest struct lines 226-240, run_ml lines 271-416); src/ipc.ts (MlRequest lines 627-636); src/ui/mlDialog.ts (whole file, no MASK picker); compare src-tauri/src/workflow.rs lines 227-263 and src/ui/moduleDialog.ts line 297 / src/ui/workflowDialog.ts lines 348-597

**Evidence:** workflow.rs's own comment (lines 227-232) states exactly why the MASK exists: 'Modules that compute run-level statistics — gr_normalize's P3/P97 percentiles, log_predict's KNN training set — would otherwise be anchored by casing/washout samples, and that mis-anchoring contaminates every output sample, flagged or not.' run_ml is precisely this class of module — it fits a StandardScaler, K-Means/GMM/hierarchical/DBSCAN, RF/SVM/ANN/logreg, or PCA over POOLED samples from the selected wells (its own docstring: 'unsupervised tasks fit directly on the pooled APPLY samples'). MlRequest carries no `opts`/mask field whatsoever, ml.rs never reads any mask curve, and mlDialog.ts never renders a mask selector (moduleDialog.ts and workflowDialog.ts both do, reusing the same opts.MASK convention). A single washed-out/casing interval left in a training or apply well will silently bias the scaler, the cluster centers, the trained model, and the PCA components for every well in the run — with no way for the user to exclude it via the project's standard bad-hole mechanism.

**Suggested fix:** Add an opts.MASK-style field to MlRequest, resolve it the same way workflow.rs does (fetch_curve_frame_from_set → NaN out flagged rows before pooling), and add a mask <select> to mlDialog.ts mirroring moduleDialog.ts's mask picker.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently verified every specific claim in the finding against the actual source files at D:\XX. SandiBumi (the SandiBumi/Arshilla repo):

1. **`src-tauri/src/ml.rs` — `MlRequest` struct (lines 226-240) and `run_ml` (lines 271-416):** Confirmed. `MlRequest` has exactly the fields `task, algorithm, params, feature_curves, target_curve, train_well_ids, apply_well_ids, output_curve` — no `opts` field and no mask-related field of any kind. `run_ml` fetches training data via `fetch_curve_frame(&conn, well_id, &fetch_names)` (line 310) and apply data via `fetch_curve_frame(&conn, well_id, &features)` (line 324) — plain curve fetches with zero mask resolution, zero reference to any flag/BADHOLE curve, and zero NaN-blanking of flagged samples anywhere in the function body. `fetch_curve_frame` itself (equations.rs line 277) is confirmed to be a bare SQL fetch of standard_curves plus computed-curve resolution — it has no masking concept built in either.

2. **`src/ipc.ts` `MlRequest` interface (lines 627-636):** Confirmed verbatim match — `task, algorithm, params, feature_curves, target_curve, train_well_ids, apply_well_ids, output_curve`. No mask/opts field.

3. **`src/ui/mlDialog.ts` (457 lines, entire file):** Confirmed zero occurrences of "mask" (case-insensitive grep returned nothing). The form-building code (formRow calls for Task, Algorithm, Input curves, Target, Train wells, Apply wells, Parameters, Output curve, Common) has no mask `<select>` of any kind.

4. **`src-tauri/src/workflow.rs` lines 227-278:** Confirmed the exact comment quoted in the finding is real, word-for-word matching: "Modules that compute run-level statistics — gr_normalize's P3/P97 percentiles, log_predict's KNN training set — would otherwise be anchored by casing/washout samples, and that mis-anchoring contaminates every output sample, flagged or not." The code resolves `req.opts.get("MASK")` via `fetch_curve_frame_from_set`, then NaNs out flagged samples in both the module's inputs (before compute) and outputs (after compute).

5. **`src/ui/moduleDialog.ts` (mask picker):** Confirmed a full mask `<select>` (`maskSelect`) at lines ~125-149 wired to `opts.MASK = maskSelect.value` (line 297), plus `MASK_CURVE_SUGGESTIONS`/`maskCurveNames` helpers.

6. **`src/ui/workflowDialog.ts` (lines 348-597 range):** Confirmed extensive per-step MASK support — `maskControl()` function, `step.opts.MASK` read/write, a dedicated "mask" GridKind column in the bulk-edit grid, etc.

Every specific claim — the MlRequest field list, the absence of any mask logic in run_ml, the absence of a mask picker in mlDialog.ts, and the workflow.rs precedent (comment text, opts.MASK resolution via fetch_curve_frame_from_set, dual input/output blanking) — checks out exactly as stated. The finding's characterization of run_ml as "precisely this class of module" (pooling samples for run-level statistics: StandardScaler fit, K-Means/GMM/etc. cluster centers, PCA components) is also textually supported by the module's own docstring (lines 1-14) and the Python runner code (StandardScaler.fit on X or A, KMeans/GMM/Agglomerative/DBSCAN.fit_predict on pooled A, PCA.fit_transform on pooled A). I found no counter-evidence — no alternate mask mechanism, no implicit masking inside fetch_curve_frame, no mask handling hidden elsewhere in ml.rs or its call sites. The finding is confirmed as accurate.

</details>

### 2. mlDialog.ts never bumps dataVersion after a successful run, unlike every sibling curve-writing dialog

**Area:** Frontend wiring (dimension D) — dataVersion propagation

**Effort:** small

**Where:** src/ui/mlDialog.ts lines 387-392 (the res.error-else success branch)

**Evidence:** run_ml writes new computed_curves via write_computed_curves_versioned (ml.rs line 398), exactly the kind of change dataVersion exists to announce. Every comparable batch-run dialog in the same folder calls bumpDataVersion() right after a successful write — multiminDialog.ts:719, cutoffDialog.ts:768, summaryDialog.ts:94, autoCorrDialog.ts:198/205, curveEditDialog.ts:115/119/123, topsEditor.ts (multiple), workflowDialog.ts:805. mlDialog.ts's success branch (lines 389-392) calls setStatus(...) and recordProcess(...) but never bumpDataVersion(). Any already-open log view, crossplot, histogram, Curve Catalog, or Database Inspector panel will not show the newly-written ML curves (FACIES_ML, ML_PRED, PC1, etc.) until something unrelated happens to bump dataVersion.

**Suggested fix:** Import bumpDataVersion from ../state and call it in the success branch alongside recordProcess(...), matching multiminDialog.ts's pattern.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read D:\XX. SandiBumi\src\ui\mlDialog.ts in full (458 lines). Confirmed: (1) imports at top of file pull only appState, defaultRunWellIds, filterByActiveGroup from ../state — bumpDataVersion is never imported; (2) the success branch of the runBtn click handler (lines 389-393, matching the cited 387-392 range) calls only setStatus(...) and recordProcess(...), with no bumpDataVersion() call anywhere in the file; (3) grep for bumpDataVersion across src/ui confirms mlDialog.ts has zero occurrences while every sibling dialog named in the finding (multiminDialog.ts:720, cutoffDialog.ts:768, summaryDialog.ts:94, autoCorrDialog.ts:198/205, curveEditDialog.ts:115/119/123, topsEditor.ts multiple lines, workflowDialog.ts:805) does call it right after a successful write; (4) src-tauri/src/ml.rs lines 397-398 confirm run_ml's success path calls create_log_set then write_computed_curves_versioned, genuinely writing new curves per apply well to the DB — a real data change that other open views (log view, crossplot, Curve Catalog, DB Inspector) rely on dataVersion to learn about. No alternate propagation mechanism (event dispatch, shared refresh call, appState mutation) exists in mlDialog.ts to substitute for the missing bumpDataVersion() call. The finding's file, line range, and described mechanism all check out against the real code; I could not refute it.

</details>

### 3. mlDialog.ts never subscribes to dataVersion, so its own wells/curve-catalog lists go stale while the pane stays open

**Area:** Frontend wiring (dimension D) — no live reload / race-guard

**Effort:** small

**Where:** src/ui/mlDialog.ts lines 138-148 (one-shot Promise.all fetch) and line 403 (dispose: () => {}); compare src/ui/moduleDialog.ts lines 235-263

**Evidence:** The ML pane is a persistent dock singleton (workspace.ts case "ml" / openSingleton("ml", ...)), not a modal reopened each use, so it can stay mounted across a whole session. Its closest sibling, moduleDialog.ts (the other auto-built well-checklist + curve-picker + run-button pane), explicitly guards against staleness: it subscribes to appState.dataVersion (lines 256-263) and re-fetches listWells()/listCurveCatalog() on every bump (via refreshData(), lines 235-254), with a dataPrimed flag to skip the redundant first fire. mlDialog.ts has zero `.subscribe(` calls anywhere in the file — wells and the feature/target curve lists are fetched exactly once in buildMlContent and never refreshed. A user who imports a new well, computes a new curve via Equations/Workflow, or runs another module while the ML pane sits open in the background will not see the new well or curve as a selectable train/apply well or input/target curve without closing and reopening the pane.

**Suggested fix:** Add a dataVersion.subscribe (with the same dataPrimed-style guard moduleDialog.ts uses) that re-fetches wells/catalog and rebuilds the checklists/selects, preserving existing checked state the way moduleDialog.ts's rebuildWellChecklist(checkedIds) does; wire its unsubscribe into the currently-empty dispose().

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the real files at D:\XX. SandiBumi\src\ui\mlDialog.ts and moduleDialog.ts, plus workspace.ts. Confirmed: mlDialog.ts's buildMlContent (line 138) does a one-shot Promise.all fetch of listWells/listCurveCatalog at lines 141-144, has zero `.subscribe(` calls anywhere in the file (grep-verified), and returns `dispose: () => {}` verbatim at line 403. moduleDialog.ts by contrast genuinely subscribes to appState.dataVersion (lines 258-264) with a dataPrimed guard, calls refreshData()/rebuildWellChecklist(checkedIds) (lines 235-256), and wires real unsubscribe calls into its dispose (lines 336-339) — matching the finding's comparison. Confirmed the ML pane is a persistent dock singleton via openSingleton(\"ml\",...) at workspace.ts:986, routed through asyncPane (a one-shot loader with no follow/rebuild logic, lines 421-443), and that re-invoking openSingleton on an already-open panel only calls setActive()/moveTo() rather than rebuilding content (lines 919-932) — so there is no outer mechanism that would refresh mlDialog's stale lists. appState.dataVersion is a real Observable<number> (state.ts:54) bumped on data changes (state.ts:94). Every specific claim (file, line numbers, code behavior, and the sibling comparison) held up under direct inspection; I found no counter-evidence and could not refute the finding.

</details>

### 4. run_ml (the DB-integration function) has zero direct unit tests — only the pure Python bridge exec_ml is tested

**Area:** Backend testing (dimension B) — edge-case coverage

**Effort:** medium

**Where:** src-tauri/src/ml.rs tests module (lines 480-605); compare src-tauri/src/multimin2.rs tests around lines 1451-1550 (e.g. rejects_underdetermined_request, rejects_all_zero_conductivity_row) which call run_multimin(&conn, &req) against an in-memory duckdb::Connection

**Evidence:** All four ml.rs tests (regression_linear_recovers_line, classification_knn_labels_blobs_confidently, clustering_kmeans_orders_by_first_feature, pca_returns_numbered_components) call exec_ml directly — the pure numpy round-trip — never run_ml. None of run_ml's own Rust-side guards are exercised by any test: the <10-labelled-samples refusal, the 'no complete samples in the apply wells' refusal, the per-well error isolation when a well is missing an input curve (fcols.len() != d branch, line 327-335), or that log_sets/computed_curves actually get written with correct versioning. The sibling module multimin2.rs proves this is achievable in this codebase — it directly unit-tests its own top-level run_multimin against Connection::open_in_memory() with exactly this kind of request-shape and per-row refusal coverage — so ml.rs's gap isn't an inherent limitation of the pattern, just untested code.

**Suggested fix:** Add tests that call run_ml(&Mutex::new(Connection::open_in_memory()...), &req) with seeded standard_curves/computed_curves rows, covering: <10 training samples, an apply well missing a feature curve (verify that well's MlWellResult.error is set while other wells still succeed), and n_apply==0.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual source files at D:\XX. SandiBumi\src-tauri\src\ml.rs (full file, including the tests module lines 480-605) and D:\XX. SandiBumi\src-tauri\src\multimin2.rs (run_multimin signature at line 451, tests rejects_underdetermined_request at line 1480 and rejects_all_zero_conductivity_row at line 1513). Also grepped the whole src-tauri tree for run_ml( and MlRequest{ construction sites, and inspected pipeline_blso_test.rs and src-tauri/tests/ (fixtures only).

Every specific claim in the finding checks out against the real code:

1. All four ml.rs tests (regression_linear_recovers_line L497, classification_knn_labels_blobs_confidently L522, clustering_kmeans_orders_by_first_feature L552, pca_returns_numbered_components L581) call `exec_ml(...)` directly (confirmed at lines 506, 539, 565, 593) — none call `run_ml`.

2. `run_ml` (defined at ml.rs:271) is never called from any test anywhere in the repo. Grep for `run_ml(` across src-tauri turns up only: the definition (ml.rs:271), and its single production caller in lib.rs:560 (the Tauri command wrapper) — no test call sites. `MlRequest { ... }` is likewise never constructed outside its own struct definition (ml.rs:227) — i.e. no test seeds a request to drive run_ml. pipeline_blso_test.rs (the repo's one #[ignore] integration-style test file) exercises modules/workflow, not ml at all. src-tauri/tests/ contains only two duckdb fixture files, no test code.

3. run_ml's own Rust-side guards are confirmed present and unexercised by any test:
   - <10 labelled-samples refusal at lines 358-362 ("only {n_train} labelled training samples...")
   - "no complete samples in the apply wells" refusal at lines 364-366
   - the per-well fcols.len() != d branch producing "missing input curve data" isolation at lines 326-335 (matches the cited 327-335 range closely)
   - log_sets/computed_curves versioned-write path at lines 391-411 (create_log_set + write_computed_curves_versioned)
   None of these are touched by exec_ml-only tests, since exec_ml is the pure-Python-bridge helper called after all of run_ml's DB/guard logic.

4. The multimin2.rs comparison is accurate: run_multimin has the identical signature shape `pub fn run_multimin(db: &Mutex<Connection>, req: &MultiminRequest) -> MultiminResult` (line 451), and its tests rejects_underdetermined_request (line 1480) and rejects_all_zero_conductivity_row (line 1513) do exactly what's claimed — construct a request, wrap `Connection::open_in_memory().unwrap()` in a Mutex, and call `run_multimin(&conn, &req)` directly, asserting on refusal-message content and res.wells being empty. This proves the DB-integration-test pattern is already established practice in this codebase for a structurally identical sibling function, so ml.rs's gap is a real, avoidable coverage hole rather than a pattern limitation.

I found no evidence to refute any part of the finding — no hidden test file, no alternate test harness, no misquoted line numbers or fabricated guard behavior. The finding is accurate and the suggested fix (seed standard_curves/computed_curves, call run_ml with a Mutex<Connection::open_in_memory()>, cover <10 samples / missing-feature-curve well isolation / n_apply==0) is a faithful, actionable mirror of the existing multimin2.rs test pattern.

</details>

---

## Equations engine (Rhai + Python)

### 1. Equation runs never bump dataVersion — every other open panel goes stale after Run

**Area:** Frontend wiring (D) / cross-function shared state (F)

**Where:** src/ui/inspectorPanel.ts handleRun() lines 286-326 (esp. 318-321); src/ipc.ts runEquation() lines 36-38 (pure invoke passthrough); contrast with src/ui/inspectorPanel.ts line 513 (the Restore-version handler, which does call bumpDataVersion()); contract stated in src/state.ts lines 52-54

**Evidence:** state.ts's own doc comment on `dataVersion` reads: "Monotonic counter bumped whenever computed curves change (module run, equation run, pay summary) so open panels can refresh their data" — explicitly naming equation runs. But `handleRun()` only calls `recordProcess(...)` and `this.refreshCatalog()` (a local, direct re-render of the Inspector's own Catalog tab) — it never imports/calls `bumpDataVersion()`. Grepping the whole src tree confirms `runEquation(` has exactly one call site (inspectorPanel.ts:318) and `ipc.ts`'s wrapper is a bare `invoke()` with no side effect. Meanwhile the sibling Restore-log-set-version handler (line 506-519) explicitly calls `bumpDataVersion()` with the comment "every open panel (log views, plots, this catalog) refreshes", showing the developers know and use this pattern elsewhere — it just wasn't wired for the Run button. REVIEW.md's own field-review checklist (lines 166-173, still unchecked `[ ]`) instructs Jauhar to test exactly this: "Open a Histogram of PHIE (zoom in), then run a module/equation that recomputes PHIE ... The plot now re-reads the new curve in place." Reading the code, a Histogram/Crossplot/Pickett/log view left open in another dock will NOT refresh after an equation run — only the Inspector's own Catalog tab appears to update, masking the gap during a quick self-test.

**Suggested fix:** Call `bumpDataVersion()` in `handleRun()` right after a run reports at least one success (mirroring the Restore handler).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual files at D:\XX. SandiBumi (the real SandiBumi/Arshilla repo) and every claim checks out exactly:\n\n1. inspectorPanel.ts handleRun() (lines 286-326, confirmed via Read): calls runEquation() at line 318, recordProcess() at 320, this.refreshCatalog() at 321 -- and never calls bumpDataVersion(), despite bumpDataVersion being imported into this very file (line 3: `import { bumpDataVersion, filterByActiveGroup, setStatus as globalStatus } from \"../state\";`).\n\n2. refreshCatalog() (lines 104-134) is confirmed to be a purely local operation: it calls listGenericCurveCatalog/listComputedCatalog/listLogSets (or legacy listCurveCatalog) and re-renders only this.catalogTab -- no state bump, no notification to other panels.\n\n3. ipc.ts runEquation() (lines 36-38) is confirmed a bare `invoke<EquationRunResult[]>(\"run_equation\", { equationId, wellIds })` passthrough with zero side effects.\n\n4. state.ts dataVersion doc comment (lines 52-54) confirmed verbatim: \"Monotonic counter bumped whenever computed curves change (module run, equation run, pay summary) so open panels can refresh their data\" -- explicitly naming equation runs, which contradicts the actual wiring.\n\n5. The Restore-log-set-version handler (inspectorPanel.ts lines 506-519, comment at 513) confirmed to call bumpDataVersion() with comment \"every open panel (log views, plots, this catalog) refreshes\", proving the pattern is known and used elsewhere in the same file.\n\n6. Whole-src-tree grep confirms runEquation( has exactly one call site (inspectorPanel.ts:318) and ipc.ts's wrapper has no side effects.\n\n7. Additional corroboration found beyond the original claim: grepping other run-style dialogs shows workflowDialog.ts:805, multiminDialog.ts:720, curveEditDialog.ts:115/119/123, and cutoffDialog.ts:768 ALL call bumpDataVersion() after their operations -- confirming this is the established convention across the codebase and the equation-run path is the sole omission.\n\n8. Confirmed no alternate cross-panel refresh mechanism exists (no Tauri event `listen()` wiring anywhere in src/) that could substitute for the missing call -- dataVersion is the sole refresh mechanism, and 11 files (histogramPanel, crossplotPanel, logViewPanel, pickettPanel, correlationPanel, mapPanel, topsEditor, dbInspectorPanel, moduleDialog, ribbon, workspace) subscribe to it.\n\nI attempted to find any mitigating factor (alternate refresh path, indirect bump via refreshCatalog, event-based mechanism) and found none. The finding is fully confirmed as stated, including its suggested fix location (handleRun, after a successful run, mirroring the Restore handler at line 513) being directly actionable since the import is already present.

</details>

### 2. Equation 'Apply to all wells' still does one DB transaction per well and blocks the IPC thread — the exact freeze pattern just fixed for Workflow chains, left unfixed here

**Area:** Backend (B) — batching & UI-thread responsiveness

**Where:** src-tauri/src/equations.rs run_equation() lines 966-1024 and write_equation_output() lines 1028-1043; src-tauri/src/python_engine.rs run_python_equation() lines 243-293; compare to the new create_log_sets_batch() (equations.rs:626) / write_computed_curves_versioned_batch() (equations.rs:662), and to run_workflow_chain in src-tauri/src/lib.rs (~lines 811-885) which was moved to std::thread::spawn + the jobs.rs registry in this same working-tree diff

**Evidence:** `git diff --stat src-tauri/src/equations.rs` shows only two new functions added in this session's uncommitted perf-fix batch: `create_log_sets_batch` and `write_computed_curves_versioned_batch`, both introduced specifically to collapse the "~2 fsync-bound DB transactions per well per step (≈1,000 commits on 500 wells)" freeze that REVIEW.md documents for workflow chains — `grep` confirms they are called only from chain.rs/workflow.rs, never from equations.rs's own run_equation/run_python_equation. Those two functions still call singular `create_log_set` (one INSERT, effectively its own commit) plus single-well `write_computed_curves_versioned` (its own `with_txn`) once per well, inside the same rayon `par_iter`/`for well_id in well_ids` loop that fans out the compute — i.e. reads are parallel but writes are still N separate commits, not the 'compute all wells in parallel, then ONE batched write' shape the ROADMAP calls the fix for this bug class. Additionally, `#[tauri::command] fn run_equation` in lib.rs (lines 301-315) is still a plain synchronous function on the IPC/UI thread — `git diff src-tauri/src/lib.rs` shows `run_workflow_chain` was moved to `std::thread::spawn` with jobs.rs progress/cancel wiring in this very diff, but `run_equation` was not touched at all. The Inspector Panel's own 'Apply to all wells' checkbox (inspectorPanel.ts line 184-187) is precisely the entry point that invites a 500+ well run.

**Suggested fix:** Route run_equation/run_python_equation through create_log_sets_batch + write_computed_curves_versioned_batch (compute-then-one-write, matching chain.rs), and move the run_equation command off the IPC thread the same way run_workflow_chain was.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I independently read every file cited and could not refute the finding — every specific claim checks out against the actual working tree in D:\XX. SandiBumi (the SandiBumi repo):

1. `src-tauri/src/equations.rs::run_equation` (lines 966-1024, verified exact) does a `rayon` `par_iter` over `well_ids`. Reads (`fetch_curve_frame`) happen under `db.lock()` per well, but the write path at the end of the loop calls `write_equation_output(&conn, well_id, ...)` once per well (line 1018), each call still inside the same lock but its own DB commit.

2. `write_equation_output` (lines 1028-1043, verified exact) calls `create_log_set` (a single autocommitted INSERT, line 1041) then `write_computed_curves_versioned` (its own `crate::db::with_txn` transaction, line 1042) — i.e. ~2 commits per well, exactly the pattern REVIEW.md (lines 40-42, present in this same working tree) describes as the frozen-chain bug: "~2 fsync-bound DB transactions per well per step (≈1,000 commits on 500 wells)."

3. `src-tauri/src/python_engine.rs::run_python_equation` (lines 243-293, verified exact) is explicitly sequential (comment at line 239-242 says so) and calls the same single-well `write_equation_output` per well (line 284) — same non-batched write pattern, and here without even the parallel-read mitigation.

4. The two new batching primitives, `create_log_sets_batch` (equations.rs:626, verified) and `write_computed_curves_versioned_batch` (equations.rs:662, verified), exist in this exact session's diff (`git diff --stat` shows only equations.rs +105/lib.rs +95/-5, matching the finding's evidence) but are called ONLY from `chain.rs:155` and `workflow.rs:339,378` — never from `run_equation`/`run_python_equation`. Confirmed via grep across all of equations.rs and python_engine.rs.

5. `lib.rs`'s `#[tauri::command] fn run_equation` (lines 300-315, verified) is a plain synchronous function with no `std::thread::spawn` — it runs straight on the IPC/UI thread and calls `equations::run_equation`/`python_engine::run_python_equation` directly. Meanwhile `run_workflow_chain` (lib.rs lines 812-882, matches the "~811-885" citation) was rewritten in this same diff to register a job via `jobs::register` and dispatch via `std::thread::spawn` (line 868), with an explicit code comment (lines 853-867) describing exactly the freeze this fixes ("Run OFF the IPC/main thread so the window stays responsive... As a sync command this blocked the event loop for the whole multi-minute chain").

6. `src/ui/inspectorPanel.ts` lines 184-187 do contain the "Apply to all wells" checkbox (`#eq-all-wells`), and `handleRun()` (lines 286-314) shows checking it fetches every well in the active group and calls `runEquation(...)` on all of them — confirming this UI path is the entry point that can fan out to hundreds of wells through the still-synchronous, still-per-well-commit `run_equation` command.

7. Strongest corroboration: REVIEW.md (this session's own uncommitted notes, lines 30-36) explicitly states the async/batching conversion was done for workflow chains first and says verbatim "import, dashboard, multimin, Monte Carlo and equations will follow the *same* pattern" — i.e., the authors themselves acknowledge equations.rs was deliberately left for a follow-up, not yet fixed.

Every line number, function name, and behavioral claim in the finding matches the real code exactly. I found no counter-evidence (no hidden batching call, no async wrapper, no evidence the "Apply to all wells" path routes through a different, already-fixed function). The finding survives independent verification.

</details>

### 3. An equation with an unresolvable input or output curve name 'succeeds' silently as all-NaN, indistinguishable from a legitimate result

**Area:** Backend (B) — singularity/missing-input handling

**Where:** src-tauri/src/equations.rs fetch_curve_frame() lines 277-359 (every requested name is guaranteed a column, NaN-filled if unresolved — see its own test comment at line 1082 'Every requested name is present as a column'); run_equation()'s has_nan short-circuit lines 996-1015; python_engine.rs run_python_equation() lines 243-293; EquationRunResult::success() always returns error:None regardless of output content

**Evidence:** Because fetch_curve_frame always synthesizes an all-NaN column for a curve name that matches nothing (standard/computed/generic), a typo'd input_curve (or output_curve saved under the wrong case per the already-known DELETE/read case mismatch) produces a run that reports `{error: None, rows_written: n}` for every well — summarizeRun() in inspectorPanel.ts (line 547-556) then shows a clean 'N/N well(s) succeeded, n rows written' with no hint the curve is empty. This is inconsistent with the established convention elsewhere in this same codebase: modules.rs's gascorr explicitly checks for exactly this shape of problem — 'A FLAGGED run whose flag curve resolved to nothing would silently correct zero samples while reporting success — indistinguishable from no gas anywhere' (modules.rs lines 1435-1443) — and returns a loud Err instead, with a unit test asserting it (`gascorr_guards_stay_missing_or_error`, line 3577: 'all-NaN flag under FLAGGED must be loud'). The equation engine has no analogous check for any of its input_curves or its output_curve.

**Suggested fix:** Before running, check that every requested input_curve resolved to at least one non-NaN sample (as fetch_curve_frame already can report via fetch_computed_curves_batch/fetch_generic_curve_aligned's underlying data), and surface a distinct error/warning per well instead of a plain success when it didn't.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every cited line against the actual source in D:\XX. SandiBumi (SandiBumi):

1. src-tauri/src/equations.rs `fetch_curve_frame()` (lines 277-359): for each requested name it checks the 6 standard columns; if none match (or the standard column is all-NaN), the name is deferred to `fetch_computed_curves_batch` then `fetch_generic_curve_aligned` (lines 368-401, 428-460). `fetch_generic_curve_aligned` explicitly returns `Ok(vec![f32::NAN; depth_grid.len()])` (line 447) when no curve_id matches in `curve_meta`. So a typo'd/unresolvable name is guaranteed to land in the `columns` HashMap as an all-NaN vector — never an error, never a missing key. The unit test at lines 1082-1094 literally asserts this ("Every requested name is present as a column (callers rely on this)" / "absent curve should be all-NaN").

2. `run_equation()` (lines 966-1024): the has_nan short-circuit (per-sample loop, lines 996-1015) sets `has_nan = true` whenever any bound column value is NaN at that depth, pushing NaN into `output` and skipping Rhai eval. If an entire input curve is unresolved (all-NaN), every sample trips this, so `output` ends up entirely NaN. Execution still proceeds to `write_equation_output` and then unconditionally returns `EquationRunResult::success(well_id.clone(), n)` (line 1021) — success is reported purely on "did the DB write succeed," not on output content.

3. `EquationRunResult::success()` (lines 37-40): `Self { well_id, rows_written, error: None }` — hard-codes `error: None`, no content check possible even if someone wanted to add one later without changing the signature.

4. python_engine.rs `run_python_equation()` (lines 243-293): identical shape — same `fetch_curve_frame` call, same "empty depth → error" guard but no all-NaN-columns guard, `exec_script` runs numpy over whatever arrays it got (NaN-filled or not) and on success returns `EquationRunResult { ..., error: None }` (line 285) regardless of whether the result array is all-NaN.

5. lib.rs `run_equation` Tauri command (lines 300-315) is a thin dispatcher with no intermediate validation — it passes the raw `Vec<EquationRunResult>` straight through.

6. modules.rs `gascorr()` (lines 1428-1443) does exactly the analogous check the finding describes: gated on `OPT_GATE=FLAGGED`, if the flag curve resolved to nothing (`!flag.iter().any(|v| !v.is_nan())`) it returns a hard `Err` instead of silently succeeding — confirmed at the cited lines, with the explanatory comment matching verbatim, and the test `gascorr_guards_stay_missing_or_error` exists (found starting at line 3554) with the "all-NaN flag under FLAGGED must be loud" framing. No equivalent guard exists anywhere in equations.rs or python_engine.rs for input_curves or output_curve.

7. inspectorPanel.ts `summarizeRun()` (lines 547-556) confirmed verbatim: it filters only on `r.error`, sums `rows_written` for the "ok" bucket, and renders `"${ok.length}/${results.length} well(s) succeeded, ${totalRows} rows written."` with zero inspection of the actual curve values — exactly the "clean success" UI text described in the finding.

Every specific line range, function name, quoted comment, and test name in the finding matches the real file contents exactly. The described failure mode (typo'd input_curve or wrong-case output_curve → all-NaN result reported as full success, unlike gascorr's analogous guard) is real and reproducible by inspection. I found no hidden validation layer (in equations.rs, python_engine.rs, or the lib.rs command wrapper) that would catch this before it reaches the UI. Confirmed.

</details>

### 4. Python engine's 'script never assigned the output curve' check is defeated when output_curve matches an input curve name (or "depth") — common in-place curve-cleanup scripts silently no-op

**Area:** Backend (B) — Python vectorized path correctness

**Where:** src-tauri/src/python_engine.rs RUNNER_LOOP, lines 76-87 (embedded Python worker script)

**Evidence:** The worker pre-populates `ns` with every input name (including "depth") BEFORE exec: `for i, name in enumerate(names): ns[name] = np.frombuffer(...)`. After exec it validates the output only via `if out_name not in ns: send({"ok": False, ...})`. Whenever a user names the output curve the same as one of its declared input_curves (a very ordinary 'despike/clean this curve in place' pattern, e.g. input=["GR"], output="GR") or the output curve is literally "depth", that name is ALREADY in `ns` from the pre-population step, so the presence check is trivially true regardless of whether the script actually reassigned it. A buggy or no-op script (a forgotten `gr = ` before `np.clip(gr, 0, 250)`, a dead `if` branch, etc.) then silently writes the untouched, original input array back into computed_curves as though it were the computed result — reported as a normal success with real (unchanged) numbers, not caught as 'script never assigned the output'. None of python_engine.rs's three tests (python_vectorized_roundtrip, python_reports_script_errors, worker_survives_a_script_error) use an output name that collides with an input name, so this gap is untested.

**Suggested fix:** Snapshot which names came from the pre-populated input bindings before exec, and require the output name either be new or have its bound object identity/values actually change (or simplest: pop the output name out of ns before exec and only treat it as 'assigned' if the script put it back).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read src-tauri/src/python_engine.rs directly. RUNNER_LOOP pre-populates the exec namespace `ns` with every declared input curve name plus \"depth\" (lines 76-78: `ns = {\"np\": np, \"numpy\": np}` then `for i, name in enumerate(names): ns[name] = np.frombuffer(...).copy()`), where `names` always starts with \"depth\" (injected in run_python_equation at python_engine.rs:265, `vec![\"depth\".into()]`) followed by the lowercased input_curves. The sole post-exec validation is `if out_name not in ns: send({\"ok\": False, ...})` (lines 81-82). Because `ns` is pre-seeded with input/depth names before `exec` runs, if a user's output_curve equals one of its own input_curves (e.g. input=[\"GR\"], output=\"GR\", a normal in-place despike/clean pattern) or equals \"depth\", that key is already present in `ns` regardless of whether the script actually reassigned it. A no-op or buggy script (dead branch, forgotten assignment) then has `ns[out_name]` still equal to the original untouched input array, `out_name not in ns` evaluates False, and the original values are asarray'd, shape-checked (passes, same length), and sent back with {\"ok\": true} as if genuinely computed — silent no-op reported as success.

I additionally checked equations.rs (EquationDef, save_equation) and the frontend src/ui/inspectorPanel.ts (handleSave/readFormIntoCurrent) — there is no validation anywhere in the save or run path preventing output_curve from colliding with an input_curves entry or with \"depth\", so this is a reachable, ordinary user configuration, not a contrived edge case.

I verified the three existing tests in python_engine.rs (python_vectorized_roundtrip: output \"vsh\", inputs depth/gr; python_reports_script_errors: output \"vsh\", input depth; worker_survives_a_script_error: outputs \"boom\"/\"out\", input depth) — none uses an output name colliding with an input/depth name, confirming the stated test gap.

Only nitpick: the finding cites \"lines 76-87\"; the precise check is at 81-82 with the vulnerable read-back at line 83, and the try block extends to line 90 — a minor line-range looseness, not a substantive inaccuracy. The core claim is fully verified against the real code, so I could not refute it.

</details>

---

## Importers A (LAS, Core CSV, Tops CSV)

### 1. LAS import always mints a brand-new well — no name-based dedup against existing wells

**Area:** Database / domain correctness (Dimension A/C)

**Effort:** medium

**Where:** src-tauri/src/ingest.rs insert_parsed_well (lines 44-45: `let well_id = Uuid::new_v4();`; lines 86-100 wraps db::insert_well unconditionally); src-tauri/src/db.rs insert_well (lines 652-665, plain INSERT, no ON CONFLICT) and the `wells` schema (lines 60-66, well_name has no UNIQUE/PK — only well_id UUID is the primary key)

**Evidence:** Every OTHER importer in this codebase attaches to an EXISTING well chosen by the caller: import_core_csv, import_scal_csv, import_deviation_csv, import_aux_file all take `well_id: &str` and first check `SELECT 1 FROM wells WHERE well_id = ?1` (ingest.rs ~206-211, 239-244, 488-493); import_tops_file/import_locations_file resolve an existing well by case/whitespace-normalized name via a `name_to_id` HashMap built from `SELECT well_name, well_id FROM wells` (ingest.rs 300-318, 391-406); dlis.rs's `import_dlis_file(conn, well_id: &str, path: &str)` (line 116) likewise requires an existing well and even reports 'replaced N existing curve(s)' on a mnemonic collision (per REVIEW.md line 129-134). LAS import is the sole outlier: `insert_parsed_well` calls `Uuid::new_v4()` unconditionally with no lookup against `wells.well_name` at all, and the frontend (ribbon.ts ~940-964) just calls `importLasFiles(paths)` with no pre-check or confirmation dialog either. Re-importing a corrected/revised delivery of the same well (a routine petrophysics workflow — e.g. a corrected final LAS after a QC pass, or the same file picked twice in the multi-file dialog) silently creates a second `wells` row with an identical `well_name` but a different `well_id`, fragmenting that well's curves/tops/core/etc. across two disconnected records with no warning to the user.

**Suggested fix:** Before calling Uuid::new_v4(), look up an existing well by the same normalized-name convention already used in import_tops_file/import_locations_file, and either reuse that well_id (prompting the user to confirm replace/merge) or at minimum surface a clear warning ('a well named X already exists') instead of silently duplicating.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified every claim in the finding against the actual source at D:\XX. SandiBumi (the active SandiBumi codebase; the sibling "XX. SandiBumi Pre" folder has no src-tauri, so it is not the live tree being reviewed):

1. src-tauri/src/ingest.rs `insert_parsed_well` (lines 44-45): `fn insert_parsed_well(...) { let well_id = Uuid::new_v4(); ...` — confirmed verbatim: unconditional fresh UUID, zero lookup against `well_name` beforehand.
2. Lines 86-100: `let result: db::DbResult<()> = db::with_txn(conn, |conn| { db::insert_well(conn, well_id, &well_name, None, None, None)?; db::insert_standard_curves(...)?; Ok(()) });` — confirmed, insert_well is invoked unconditionally inside the txn closure with no prior existence/name check.
3. src-tauri/src/db.rs `insert_well` (lines 652-665): plain `INSERT INTO wells (well_id, well_name, field_name, td, kb) VALUES (?, ?, ?, ?, ?)` — confirmed, no ON CONFLICT / UPSERT clause of any kind.
4. `wells` schema (lines 60-66): `well_id UUID PRIMARY KEY, well_name VARCHAR NOT NULL, field_name VARCHAR, td FLOAT, kb FLOAT` — confirmed, well_name carries no UNIQUE constraint; only well_id (the freshly minted UUID) is the primary key, so nothing in the schema prevents duplicate well_name rows.
5. Cross-checked every other importer exactly as cited:
   - import_core_csv (ingest.rs 205-211), import_scal_csv (238-244), import_deviation_csv (167-172), import_aux_file (488-493): all take `well_id: &str` and run `SELECT 1 FROM wells WHERE well_id = ?1` before proceeding, returning an 'unknown well' error if absent. Confirmed at each cited line range.
   - import_tops_file (287-318, exposed to the frontend as the `import_tops_csv` Tauri command — see lib.rs 153-159 which calls `ingest::import_tops_file`) and import_locations_file (373-406): both build a `name_to_id` HashMap from `SELECT well_name, well_id FROM wells`, normalized via `.trim().to_uppercase()`, and resolve an existing well by name rather than minting a new one. Confirmed verbatim, including the exact normalization convention.
   - dlis.rs `import_dlis_file` (line 116) likewise requires an existing well via `SELECT 1 FROM wells WHERE well_id = ?1` (lines 119-124) before doing anything else.
6. Frontend src/ui/ribbon.ts (~940-964): the LAS import handler opens a file picker and calls `importLasFiles(paths)` directly with no name pre-check or confirmation dialog anywhere in that code path — confirmed.

Every cited line number, function name, and behavioral description matches the real code exactly. LAS import is genuinely the sole importer in the group that mints a brand-new UUID unconditionally instead of resolving against an existing well by ID or normalized name. Re-importing a same-named LAS (e.g., a corrected/revised delivery, or the same file picked twice in a multi-file dialog) will silently create a second `wells` row sharing the well_name but with a distinct well_id, fragmenting that well's curves/tops/core data across two disconnected records with no warning surfaced to the user. I found nothing in the code that refutes or weakens this finding — it holds up under independent verification.

</details>

### 2. Tops CSV: a blank WELL cell in a multi-well file silently misroutes that top to whatever well happens to be selected in the UI

**Area:** Domain correctness / data integrity (Dimension A/C)

**Effort:** low

**Where:** src-tauri/src/parsers.rs TopsRecord (lines 855-860) and parse_tops_file (lines 926-980, esp. 970-974); src-tauri/src/ingest.rs import_tops_file (lines 287-357, esp. 323-341); frontend caller src/ui/ribbon.ts handleImportTops (~1181-1214, `importTopsCsv(well?.well_id ?? null, path)`)

**Evidence:** parse_tops_file collapses two distinct situations into the same `TopsRecord.well == None`: a file with NO WELL column at all, and a file that HAS a WELL column but whose cell is blank for this particular row (`.filter(|s| !s.is_empty())` at line 973, with no `has_well_column` flag returned). import_tops_file then treats `None` uniformly: `None => match default_well_id { Some(id) => id.to_string(), None => return fail(...) }` (ingest.rs 335-341) — i.e. it routes the row to whatever well is currently selected in the UI. Since ribbon.ts always passes `well?.well_id ?? null` (the globally selected well, almost always Some in normal use), a blank WELL cell in an otherwise multi-well tops file gets silently attributed to the wrong (currently-selected) well instead of being skipped. This is the EXACT bug class already identified and fixed one function below in the same file: parse_locations_file explicitly returns `(bool /*has_well_column*/, Vec<LocationRecord>)` (parsers.rs line 1010, 1044) and import_locations_file uses it to distinguish and SKIP a blank-WELL row in a multi-well file rather than routing it to `default_well_id` (ingest.rs 428-434, with the comment 'misrouting it to the selected well would silently overwrite an unrelated well's real surface location', and an explicit regression test `locations_import_skips_blank_well_cell_not_default`). The identical fix and test were never applied to import_tops_file/parse_tops_file.

**Suggested fix:** Give parse_tops_file the same `(bool, Vec<TopsRecord>)` signature as parse_locations_file, and in import_tops_file treat a blank WELL cell in a has-WELL-column file as a dropped/reported row (like `blank_rows` in import_locations_file), only falling back to `default_well_id` for a genuinely column-less file.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the actual files at D:\XX. SandiBumi\src-tauri\src\parsers.rs, D:\XX. SandiBumi\src-tauri\src\ingest.rs, and D:\XX. SandiBumi\src\ui\ribbon.ts and could not refute any part of the claim -- every cited line matches.

1. parsers.rs TopsRecord (855-860): struct exactly as described, `well: Option<String>` with a doc comment saying None covers 'no well column' -- no has_well_column flag.
2. parsers.rs parse_tops_file (926-980): confirmed at 970-974 the well is derived via `.filter(|s| !s.is_empty())` and pushed straight into `TopsRecord{ well, ... }` with no signal distinguishing 'file has no WELL column' from 'file has WELL column but this cell is blank'. Only a single `Vec<TopsRecord>` is returned (not a tuple with a bool), unlike the sibling function.
3. parsers.rs parse_locations_file (1004-1010+): by contrast explicitly returns `(bool /*has_well_column*/, Vec<LocationRecord>)`, with doc comments explaining exactly why the flag exists (to let the importer tell a column-less file apart from a blank cell in a multi-well file).
4. ingest.rs import_tops_file (287-357): the well-id resolution at lines 323-341 has only two arms -- `Some(name) => ...` (name lookup) and `None => match default_well_id { Some(id) => id.to_string(), None => return fail(...) }` -- so any row with a blank/absent WELL cell (regardless of whether the file has a WELL column at all) falls back to `default_well_id`, i.e. the currently selected well. No blank-row counting/skipping exists in this function at all (confirmed via grep -- 'blank'/'has_well_column' appear nowhere in import_tops_file).
5. ingest.rs import_locations_file (373-462): explicitly takes `has_well_column` from parse_locations_file and has a dedicated `None if has_well_column => { blank_rows += 1; continue; }` arm (428-434) with the comment 'misrouting it to the selected well would silently overwrite an unrelated well's real surface location' (matches the finding's quoted comment verbatim), plus surfacing of blank rows (455-457) and a dedicated regression test `locations_import_skips_blank_well_cell_not_default` (781-825) that specifically asserts the selected well is untouched and the blank row is reported, not routed.
6. The existing tops test `tops_import_multiwell_and_default` (872-912) only covers fully-populated WELL cells (including a no-column single-well fallback) -- it never exercises a blank WELL cell inside a multi-well file, so no test currently guards against this misrouting for tops.
7. Frontend ribbon.ts handleImportTops (1181-1214) confirmed to call `importTopsCsv(well?.well_id ?? null, path)` at line 1196, passing the globally selected well as the fallback default for every row, exactly as described.

I verified db::upsert_top (src-tauri/src/db.rs ~731-739) has no additional guard against this -- it's a plain upsert keyed on (well_id, top_name), so a misrouted blank-WELL row will silently insert or overwrite a top on whatever well is currently selected, which is a real data-integrity defect matching the finding's description. All line numbers and quoted code/comments in the finding matched the real files; I found no mitigating code path anywhere in the tops import chain. The finding stands confirmed.

</details>

### 3. Core CSV import aborts the entire well's import with a raw DuckDB constraint error on any duplicate-depth row

**Area:** Backend (Rust) / importer robustness (Dimension A/B)

**Effort:** low

**Where:** src-tauri/src/parsers.rs parse_core_csv (lines 606-642, esp. 629-637 — only a NaN/unparseable depth is skipped; no dedup of a depth value repeated across rows); src-tauri/src/db.rs insert_core_data (lines 512-534, DELETE + Appender wrapped in with_txn against `core_data` whose PK is (well_id, depth), schema lines 213-221)

**Evidence:** Verified empirically: a core CSV with two plug rows at the same depth (2001.0/2001.0/2002.0 — a realistic repeat/sidewall-core scenario) makes `insert_core_data`'s Appender fail with `Failed to append: PRIMARY KEY or UNIQUE constraint violation: duplicate key "<well_id>, 2001.0"`, which with_txn then rolls back entirely, so `import_core_csv` returns `rows: 0` and that raw internal DuckDB error string as the user-facing message — the well's ENTIRE core dataset fails to import over one repeated depth. This contrasts with LAS import, where `depth_keep_indices`/`sanitize_curve_columns` (parsers.rs 285-332) explicitly dedupe (first occurrence kept) and the import still succeeds with a friendly 'dropped N row(s) with duplicate depth' warning (ingest.rs 72-82, tested by `duplicate_depth_las_imports_standard_and_generic_curves`). Core CSV has no equivalent handling or test.

**Suggested fix:** Apply the same first-occurrence-wins depth dedup used for LAS (reuse `depth_keep_indices` generically) inside parse_core_csv, and surface a warning (like the LAS path) instead of letting a duplicate-depth row propagate to a raw DuckDB constraint-violation string.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently verified against the actual source at D:\XX. SandiBumi\src-tauri\ (the SandiBumi/Arshilla repo) — every claim in the finding checks out, and I additionally reproduced it empirically by compiling and running a real test against the codebase.

1. parse_core_csv (parsers.rs lines 606-642): confirmed. The only depth handling is line 629-632: `if depth.is_nan() { continue; }` — a comment even says "a row with no depth can't be stored (PRIMARY KEY includes depth)", but there is zero logic to detect or drop a depth value that repeats across rows. No call to any dedup helper.

2. core_data schema (db.rs lines 213-221): confirmed PRIMARY KEY (well_id, depth).

3. insert_core_data (db.rs lines 512-534): confirmed — DELETE for the well, then a DuckDB Appender loop (`appender.append_row` per row) wrapped in `with_txn` (BEGIN/COMMIT/ROLLBACK at db.rs ~494-508). Any append failure returns Err, `with_txn` rolls back, and the error is DbError::DuckDb (db.rs line 8-9: `#[error("duckdb error: {0}")] DuckDb(#[from] duckdb::Error)`), i.e. a thinly-wrapped raw DuckDB message, not a friendly one.

4. import_core_csv (ingest.rs lines 205-222): confirmed — on `insert_core_data` Err(e), returns `CoreImportResult { rows: 0, error: Some(e.to_string()) }`, propagating the raw DuckDB string to the caller.

5. LAS contrast (parsers.rs 285-332, ingest.rs 44-82): confirmed. `depth_keep_indices` explicitly does first-occurrence-wins dedup (with a comment: "Without this, a single such file aborts the whole import with a cryptic PK-constraint error"), used by `sanitize_curve_columns`/`sanitize_las_frame`, called from `insert_parsed_well` (ingest.rs line 48), producing a friendly "dropped N row(s) with duplicate depth" warning (ingest.rs lines 73-77) instead of aborting. Grepping the whole src tree confirms `depth_keep_indices`/`sanitize_curve_columns` is used ONLY on the LAS path — never referenced from parse_core_csv, insert_core_data, or import_core_csv.

6. No existing test covers a duplicate-depth-within-one-file core CSV scenario (only a re-import-replaces test exists at ingest.rs ~537-589).

Empirical reproduction: I added a temporary #[cfg(test)] test to ingest.rs (BALAM-1 well, CSV with two rows at depth 2001.0 plus one at 2002.0), compiled with `cargo test --lib` in the real project, and ran it. Actual output:
`QC RESULT: rows=0, error=Some("duckdb error: Failed to append: PRIMARY KEY or UNIQUE constraint violation: duplicate key \"<well_id>, 2001.0\"")`
`QC ROWS IN TABLE AFTER IMPORT: 0`
This exactly matches the finding's claimed behavior (rows:0, raw DuckDB constraint string surfaced, whole import — including the two otherwise-valid unique-depth rows — rolled back to zero). I then reverted the temporary test via `git checkout -- src-tauri/src/ingest.rs` and confirmed via `git status`/`git diff --stat` that the file is back to its original tracked state with no residual diff.

Conclusion: the finding is accurate in every particular — code locations, mechanism, contrast with LAS handling, and real-world reproduced behavior. No refutation found; confirmed=true.

</details>

### 4. Tops CSV import writes row-by-row with no transaction wrap, unlike the near-identical Locations importer in the same file

**Area:** Database (Dimension A) — atomicity

**Effort:** low

**Where:** src-tauri/src/ingest.rs import_tops_file loop (lines 320-349, calling db::upsert_top per record with no BEGIN/COMMIT) vs import_locations_file (lines 408-454, which explicitly wraps its analogous per-record loop in `conn.execute_batch("BEGIN")` ... COMMIT/ROLLBACK, with the comment 'a mid-file DB error must not leave some wells relocated and others not')

**Evidence:** Each `db::upsert_top` call in import_tops_file auto-commits independently (no surrounding transaction), so a DB error partway through a multi-row file leaves the already-processed rows permanently committed while `import_tops_file` returns `fail(...)` reporting `tops_written: 0` — an inconsistent/misleading result versus what's actually persisted. This is a lower-probability trigger than the other findings (upsert_top's inputs are already validated f32/String, so a mid-loop failure needs something like a disk/lock error), but it's a direct, demonstrable inconsistency against the atomicity discipline the project itself applies one function later for the same file-import shape.

**Suggested fix:** Wrap the upsert_top loop in the same BEGIN/COMMIT/ROLLBACK (or db::with_txn) pattern already used by import_locations_file directly below it.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified directly against D:\XX. SandiBumi\src-tauri\src\ingest.rs (the actual SandiBumi/Arshilla repo).

import_tops_file (lines 287-357): the per-record loop at lines 323-349 calls `db::upsert_top(conn, &well_id, &rec.top_name, rec.depth, None)` (line 342) with no BEGIN/COMMIT/ROLLBACK anywhere around it or in the enclosing function. On error it does `return fail(e.to_string())` (line 347), and `fail` (defined line 288-294) always sets `tops_written: 0`, regardless of how many rows were already upserted before the error.

import_locations_file (lines 373-465): the analogous per-record loop is explicitly wrapped — `conn.execute_batch("BEGIN")` at line 410, `COMMIT` at line 451, with `ROLLBACK` on every early-return error path (lines 439, 446, 452) — and carries the comment at lines 408-409: "All-or-nothing: a mid-file DB error must not leave some wells relocated and others not (which would otherwise report wells_located = 0 while rows are already persisted)." This is essentially the exact rationale quoted in the finding.

Checked db.rs: `db::upsert_top` (line 731) is a single bare `conn.execute(...)` — no internal transaction. `db::with_txn` (line 492) exists and is the pattern used elsewhere (e.g., insert_parsed_well in the LAS import path) for exactly this atomicity purpose, confirming the project's own convention that connection execute() calls outside an explicit BEGIN commit immediately (autocommit), so a mid-loop error would strand already-executed upserts.

Checked callers in lib.rs (Tauri commands import_tops_csv line 152-160 and import_locations_csv line ~675-682): each just locks the mutex and calls straight into the respective ingest:: function — no outer transaction wraps import_tops_file either, so there is no hidden mitigation upstream.

Every specific claim in the finding — the missing transaction in import_tops_file, the exact line ranges, the presence and wording of the BEGIN/COMMIT/ROLLBACK plus comment in import_locations_file, and the tops_written:0-on-partial-failure behavior — checks out against the real file. I could not find any countervailing fact (no outer transaction, no idempotent-retry design, no test covering/mitigating this) that would refute it. The finding's own caveat about it being a lower-probability trigger (needs a mid-loop DB-level failure, not a data-validation failure) is also accurate and appropriately hedged. Confirmed as valid.

</details>

---

## Importers B (Aux data, Deviation, SCAL, Well locations)

### 1. Deviation-survey TVD/TVDSS is computed and stored, but no code path ever exposes it as a fetchable curve — sw_height's "TVD" input (the P0 fix's whole point) is permanently unreachable for any well relying on Import Deviation, and the module dialog silently pre-selects it as if it worked

**Area:** cross-function integration (deviation.rs / satheight.rs / ingest.rs) — dimensions B, C, F

**Where:** src-tauri/src/ingest.rs:161-194 (import_deviation_csv only writes db::insert_well_path, never a curve); src-tauri/src/deviation.rs:75-100 (tvd_at is #[allow(dead_code)], called nowhere outside its own tests); src-tauri/src/equations.rs:277-359 (fetch_curve_frame — the sole resolver ModuleContext.logs is built from — checks standard_curves → computed_curves → generic curve_meta/curve_samples store; grep for well_path/tvd_at/deviation across equations.rs, workflow.rs, chain.rs, modules.rs and curves.rs::family_for returns nothing); src-tauri/src/satheight.rs:127-144 (sw_height silently falls back `let dv = { let t = tvd[i] as f64; if t.is_nan() { depth[i] as f64 } else { t } }`); src/ui/moduleDialog.ts:84-104 (logChoiceNames/fillSelect always prepend and pre-select the module's declared default_curve — "TVD" for sw_height — into the dropdown even when zero curves of that name exist anywhere)

**Evidence:** REVIEW.md's P0 backlog (lines 105-110) touts "SW-height uses TVD and allows a subsea FWL" as a fixed correctness bug and tells Jauhar to "confirm SWH rises vs the old MD-based result" on a real deviated well "with the TVD curve mapped". But there is no code anywhere that turns a well's imported well_path stations into a curve a module can read: fetch_curve_frame (the only function that populates ModuleContext.logs for any module run) resolves a name from standard_curves, then computed_curves, then the generic curve_meta/curve_samples store — none of which import_deviation_csv ever writes to. deviation.rs's own tvd_at (the per-depth interpolation function that would be needed to resample station TVD onto a curve's depth grid) is explicitly tagged #[allow(dead_code)] with a comment saying it's "kept and tested now so the resampling math lands with the rest of the deviation work" for a *future* log/correlation-view depth scale — never wired to feed modules. ROADMAP.md line 253-255 only defers the display use ("optional TVD depth scale in the log/correlation views"); it never flags that sw_height's own advertised TVD input is equally starved. Meanwhile moduleDialog.ts's `logChoiceNames(arg.default)` always prepends the module's declared default (`log_in("TVD", ..., "TVD", false)` in satheight.rs:119) and `fillSelect(select, ..., arg.default)` pre-selects it — so a user opening SW — Saturation-Height sees "TVD" already chosen, with no indicator that zero samples back that name. At run time `ctx.log("TVD")` (modules.rs:135-137) returns an all-NaN vector for any name absent from ModuleContext.logs, so sw_height's per-sample fallback (`if t.is_nan() { depth[i] as f64 } else { t }`) silently takes the MD branch for every sample of every well — reproducing exactly the pre-fix, over-optimistic (~1/cos(inc)) SWH the P0 item claims was eliminated, with no error, warning, or visual cue anywhere in the run result. The only way to actually satisfy sw_height's TVD input is for the well's original LAS/DLIS delivery to already carry its own literal "TVD" mnemonic curve — a case the Import Deviation feature was built to NOT require.

**Suggested fix:** After computing `stations` in import_deviation_csv (or on well_path read), resample TVD/TVDSS onto the well's real curve depth grid (reusing tvd_at) and write it into curve_meta/curve_samples (e.g. mnemonic "TVD"/"TVDSS", family-tagged, set RAW or a dedicated "DERIVED" set) so fetch_curve_frame can find it like any other curve — this both fixes sw_height and unlocks the same for phimax/precalc's TVDSS input. Short of that, at minimum stop pre-selecting a default_curve in moduleDialog.ts when the catalog shows zero samples for it, or have the module runner flag/warn when a selected log_in resolves to an all-NaN curve.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read every file/line cited in the finding directly from D:\\XX. SandiBumi\\src-tauri and src\\ui:\n\n1. ingest.rs:161-194 (import_deviation_csv) — verified it computes `stations` via `crate::deviation::minimum_curvature` and writes them ONLY through `db::insert_well_path(conn, well_id, &stations)`. No call to db::upsert_curve_meta/insert_curve_samples anywhere in this function, unlike import_all_curves_into_generic_store which does write curves for LAS.\n\n2. deviation.rs:75-100 (tvd_at) — verified the `#[allow(dead_code)]` attribute and the exact comment \"Consumed by the Phase 6c TVD-depth-scale option in the log/correlation views; kept and tested now so the resampling math lands with the rest of the deviation work.\" It is only called from `#[cfg(test)] mod tests` in the same file.\n\n3. equations.rs:277-359 (fetch_curve_frame) — verified the resolution order standard_curves -> computed_curves (fetch_computed_curves_batch) -> generic store (fetch_generic_curve_aligned, querying curve_meta/curve_samples WHERE set_name='RAW'). Grepped well_path|tvd_at|deviation across the whole src-tauri/src tree: matches only in ingest.rs, db.rs, modules.rs (TVDSS param docs, unrelated to well_path), lib.rs, parsers.rs, satheight.rs, dlis.rs (comment only), deviation.rs itself. Zero matches in equations.rs, workflow.rs, chain.rs, curves.rs — exactly as the finding states.\n\n4. modules.rs:135-137 (ModuleContext::log) — verified `self.logs.get(name).cloned().unwrap_or_else(|| vec![f32::NAN; self.n])`, i.e., an unresolved curve name returns an all-NaN vector silently.\n\n5. satheight.rs:119 (log_in(\"TVD\",...,\"TVD\",false)) and satheight.rs:141-144 — verified the exact quoted fallback: `let t = tvd[i] as f64; if t.is_nan() { depth[i] as f64 } else { t }`.\n\n6. src/ui/moduleDialog.ts:84-104 — verified `logChoiceNames(keep)` prepends `keep` when `!curveNames.includes(keep)`, and `fillSelect` marks `option.selected = true` when `name === selected`. `curveNames` comes from `listCurveCatalog()` and the file's own doc comment states \"The catalog only lists curves that have actually been computed,\" confirming a never-written \"TVD\" name would still be prepended and pre-selected.\n\n7. REVIEW.md (~105-109) and ROADMAP.md (~253-255) — verified the quoted text matches: REVIEW.md's P0 item claims the TVD/subsea-FWL fix and instructs confirming on a real deviated well with TVD mapped; ROADMAP.md only defers the log/correlation-view TVD depth-scale *display*, never flagging sw_height's own input starvation.\n\n8. Additionally checked db.rs::get_well_path / lib.rs get_well_path Tauri command / ipc.ts getWellPath — defined but never called anywhere in src/, confirming no alternate UI path surfaces well_path-derived TVD either.\n\nNo counter-evidence found in jobs.rs, chain.rs, workflow.rs, dlis.rs, pipeline_blso_test.rs, or curves.rs::family_for that would resample well_path into the generic curve store. Every cited line number, quoted comment, and code snippet in the finding matches the real source exactly. The finding is accurate and reproducible: sw_height's (and phimax/precalc's) TVD/TVDSS input is unreachable for any well relying on Import Deviation, and the dialog pre-selects it with no indication it's unbacked.

</details>

### 2. Deviation-survey import has no duplicate-MD handling before the well_path (well_id, md) PK insert — unlike the already-fixed, analogous LAS duplicate-depth case, a survey with one repeated station MD aborts the whole import instead of surviving it

**Area:** importer robustness (parsers.rs / db.rs / ingest.rs) — dimension A/B

**Where:** src-tauri/src/parsers.rs:790-826 (parse_deviation_csv sorts stations by MD but never dedups equal MD, unlike sanitize_curve_columns/sanitize_las_frame used by the LAS path); src-tauri/src/db.rs:286-294 (well_path PRIMARY KEY (well_id, md)) and :1756-1766 (insert_well_path — DELETE then raw appender.append_row per station, no pre-dedup, wrapped in with_txn so any flush failure rolls back the whole write); src-tauri/src/ingest.rs:161-194 (import_deviation_csv calls parse_deviation_csv → minimum_curvature → insert_well_path directly, with no sanitize step analogous to sanitize_curve_columns at line 48 / sanitize_las_frame at line 132)

**Evidence:** This project's own house pattern (REVIEW.md P0 item "LAS import survives duplicate/odd-depth files on BOTH stores", and ingest.rs's own duplicate_depth_las_imports_standard_and_generic_curves test at lines 681-719) establishes that inserting into a table with a (well_id, depth)-style PRIMARY KEY via the DuckDB Appender aborts on a duplicate key — the ingest.rs comment at lines 127-129 says exactly this: duplicate depths "would otherwise abort each curve's insert here — silently, since this whole import is best-effort", which is why sanitize_las_frame is run before every curve_samples/standard_curves write. well_path carries the identical PRIMARY KEY (well_id, md) shape, but parse_deviation_csv only sorts stations by MD — it never removes an exact-duplicate MD row (a realistic artifact: re-entry surveys, a merged gyro + MWD file, or a repeated kickoff-point station). When that happens, insert_well_path's appender.flush() raises a primary-key constraint violation inside the with_txn closure, which rolls back the entire transaction (undoing even the DELETE of the well's prior survey) and returns a raw DB error string as CoreImportResult.error — a total import failure — instead of dropping the duplicate station and importing the rest with a warning, the behavior every other depth-keyed importer in this codebase now has. No test in deviation.rs or ingest.rs exercises a duplicate-MD deviation file.

**Suggested fix:** Add a dedup pass to parse_deviation_csv (or right before insert_well_path) that drops a station whose MD exactly repeats an earlier kept one — first-occurrence-wins, mirroring sanitize_curve_columns — and surface the drop count in CoreImportResult the same way ImportResult.warning does for LAS.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read every file/line cited (in D:\XX. SandiBumi\src-tauri\src\{parsers.rs, db.rs, ingest.rs, deviation.rs} plus the frontend caller in src\ui\ribbon.ts) and the finding's substance holds up completely.

Verified independently, point by point:

1. parsers.rs:790-826 `parse_deviation_csv` — confirmed exact: it builds `rows: Vec<(f32,f32,f32)>`, sorts by MD at line 817 (`rows.sort_by(...)`), and pushes every row into `DeviationSurvey` with zero dedup logic. No `.dedup()`, no equal-MD filtering — unlike `sanitize_curve_columns` (parsers.rs:318) and `sanitize_las_frame` (parsers.rs:372), which explicitly track and drop `duplicate` depths (see `DepthSanitizeReport.duplicate`, parsers.rs:272-273, and their own tests e.g. `sanitize_dedups_signed_zero_depths`).

2. db.rs:286-294 `CREATE TABLE well_path (... PRIMARY KEY (well_id, md))` — confirmed exact match, including the comment about vertical wells.

3. `insert_well_path` — content confirmed exactly as described (DELETE FROM well_path WHERE well_id, then a fresh Appender, `append_row` per station in a loop, `flush()`, all inside `with_txn`, no pre-dedup) — BUT its real location in the current file is db.rs:1830-1841, not :1756-1766 as the finding states. Lines 1756-1766 in the current file are actually part of `upsert_curve_meta` (an unrelated curve_meta upsert). This is a line-number citation error in the finding — the function and behavior it describes are real and correctly characterized, just mis-cited by ~75 lines, likely drift from a different revision of db.rs.

4. ingest.rs:161-194 `import_deviation_csv` — confirmed exact: it goes straight from `parsers::parse_deviation_csv` → `crate::deviation::minimum_curvature` → `db::insert_well_path`, with no sanitize/dedup call anywhere in between, unlike `insert_all_curves_into_generic_store` which explicitly calls `parsers::sanitize_las_frame` (ingest.rs:132) with a comment (ingest.rs:127-129, exact quote verified) explaining why: duplicate depths would otherwise abort the PK insert silently.

5. `db::with_txn` (db.rs:492-508) confirmed: on `Err`, it issues `ROLLBACK` and propagates the error — so any PK violation raised while appending duplicate-MD stations rolls back the preceding `DELETE FROM well_path` too, and the error string flows up as `CoreImportResult.error` (ingest.rs:192) with no partial-success/warning path (`CoreImportResult` only has `path/rows/error`, no `warning` field like `ImportResult` does).

6. `minimum_curvature` (deviation.rs) itself does not crash on a duplicate MD — a zero `d_md` just yields `d_tvd = 0`, producing two `Station`s with an identical `md` — so the failure is deferred exactly to the DB insert, matching the finding's causal chain.

7. Test coverage check: grepped `ingest.rs` and `deviation.rs` for "dedup"/"duplicate" — the only duplicate-depth test is `duplicate_depth_las_imports_standard_and_generic_curves` (ingest.rs:684-717), which covers LAS/curve_samples, not deviation/well_path. The one existing deviation-import test (`generic_las_import_keeps_all_curves_and_converts_units`, ingest.rs:619-677) uses strictly increasing, unique MDs (0, 1000, 2000) and would not exercise this path. No frontend guard either — `src\ui\ribbon.ts` `handleImportDeviation` passes the file straight to `importDeviationCsv` and on error just surfaces the raw `result.error` string in the status bar, confirming the "total import failure surfaced as a raw DB error" consequence.

Net: every substantive claim (missing dedup in the parser, PK shape, fragile DELETE+Appender+with_txn insert pattern, missing sanitize step in the ingest glue, absence of any covering test, and the established in-codebase precedent that this exact pattern aborts on duplicate keys) is verified true against the real code. I could not find any hidden dedup, frontend validation, or DuckDB-Appender leniency that would refute it. The only defect I found in the finding itself is a citation error: db.rs:1756-1766 does not contain `insert_well_path` in the current file (that function is actually at approximately db.rs:1830-1841); the cited range is `upsert_curve_meta` instead. This is worth correcting in the finding's evidence line-refs, but it does not undermine the finding's technical claim, which I independently reproduced by locating the real function elsewhere in the same file.

</details>

---

## DLIS import

### 1. DLIS import has no depth sanitization — a single duplicate/non-finite depth sample aborts the entire import (not just that curve), unlike the equivalent LAS fix

**Area:** A/B — importer robustness, per-item error isolation (audit checklist item explicitly asks: "survives duplicate/non-monotonic/all-null depth")

**Effort:** small — reuse parsers::depth_keep_indices (already public within the crate) per-frame in dlis.rs before the write, plus continue-on-error instead of return-on-error in the per-curve loop with a warning collected in DlisImportResult.

**Where:** src-tauri/src/dlis.rs lines 166 (depth read straight from the Python payload), 175-179 (null-sentinel screening applied only to `values`, never to `depth`), 207-218 (per-curve closure: any Err from insert_curve_samples propagates straight to `return fail(...)`, aborting the whole file). Compare src-tauri/src/ingest.rs lines 122-135 (`import_all_curves_into_generic_store` calls `parsers::sanitize_las_frame(&mut frame)` before writing) and src-tauri/src/parsers.rs lines 289-332 (`depth_keep_indices`/`sanitize_curve_columns`, whose doc comment says verbatim: "Without this, a single such file aborts the whole import with a cryptic PK-constraint error"), plus the regression test `duplicate_depth_las_imports_standard_and_generic_curves` (ingest.rs ~line 660, comment: "the generic path re-parses the file and writes curve_samples (curve_id, depth) PK, so without the same depth dedup it aborts silently").

**Evidence:** curve_samples has PRIMARY KEY (curve_id, depth) (db.rs line ~281). dlis.rs writes each curve's raw depth array straight into this table via db::insert_curve_samples with zero dedup/finite-check — no call to sanitize_curve_columns, sanitize_las_frame, or any equivalent. DLIS frames commonly contain duplicate or non-finite index samples (repeat sections, tool-sticking backup passes, memory-dump overlaps) — exactly the LAS failure mode the codebase already diagnosed and fixed elsewhere with an explicit test. Because dlis.rs's per-curve loop has no error isolation, one bad frame's PK violation returns `fail(...)` immediately: curves already committed in earlier loop iterations stay in the DB (each upsert_curve_meta/insert_curve_samples call commits independently), but the function reports a hard failure (`DLIS import failed: storing curve '<name>': ...`), so the user sees an unambiguous failure message while the well is actually left with a partial, silently-committed set of curves.

**Suggested fix:** Sanitize each frame's depth array (drop non-finite / duplicate depths, first-occurrence-wins, matching parsers::depth_keep_indices semantics) before calling insert_curve_samples, and/or wrap the per-curve write in the same best-effort isolation pattern already used for LAS's generic-store path so one bad curve is skipped and reported rather than aborting the whole file.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual source and could NOT refute the finding — every specific claim verifies against the real code:

1. **dlis.rs:166** — `let depth = read_f32(&payload[offset..offset + bytes]);` reads depth straight from the Python payload with zero processing. Confirmed.

2. **dlis.rs:175-179** — the sentinel/finite screening loop (`for v in &mut values { if !v.is_finite() || v.abs() > 1e30 || crate::parsers::is_las_null(*v) { *v = f32::NAN; } }`) iterates only over `values`. `depth` is never touched — no finite check, no dedup. Confirmed.

3. **dlis.rs:207-218** — the per-curve closure calls `db::upsert_curve_meta` then `db::insert_curve_samples`; on `Err(e)` it does `return fail(...)`, immediately terminating the whole function/loop rather than skipping just that curve. Confirmed verbatim.

4. **db.rs:281** — `curve_samples` table declared with `PRIMARY KEY (curve_id, depth)`. Confirmed.

5. **db.rs:1782-1798** — `insert_curve_samples` wraps its delete+append in its own `with_txn` (a single `BEGIN`/`COMMIT` for that one curve only, db.rs:492-507), while `upsert_curve_meta`'s `INSERT INTO curve_meta` (db.rs ~1772-1776) is a bare autocommitting statement outside any transaction. So each curve's writes commit independently — exactly as the finding claims: earlier-succeeding curves stay durably committed even though the function later returns a hard failure. I also checked the Tauri command wrapper (lib.rs:409-413) and `import_dlis_file`'s test caller — neither wraps the whole import in an outer transaction, so nothing rolls back the partial writes.

6. **ingest.rs:122-135 / parsers.rs:289-332,368-385** — `import_all_curves_into_generic_store` calls `parsers::sanitize_las_frame(&mut frame)` before writing, built on `depth_keep_indices` (drops non-finite depths and dedups depths, first-occurrence-wins). The doc comment at parsers.rs:313-314 reads, near-verbatim as quoted: "Without this, a single such file aborts the whole import with a cryptic PK-constraint error." Confirmed.

7. **ingest.rs:655-693** — regression test `duplicate_depth_las_imports_standard_and_generic_curves` exists exactly as described, with the comment (ingest.rs:657-658) "the generic path re-parses the file and writes curve_samples (curve_id, depth) PK, so without the same depth dedup it aborts silently" — near-verbatim match.

8. `depth_keep_indices` in parsers.rs is a private (non-`pub`) module function tied to `LasFrame`/`CurveColumns`, so DLIS's `Vec<f32>` depth/values pairs can't call it directly without refactoring — this doesn't change the finding's correctness, just confirms the suggested fix needs a small factoring step.

Every line reference, quoted comment, and causal mechanism (partial-commit-then-hard-fail) checks out against the real files at D:\XX. SandiBumi\src-tauri\src\{dlis.rs, db.rs, ingest.rs, parsers.rs, lib.rs}. I found no inaccuracy to refute.

</details>

### 2. The "no silent overwrite" collision check only catches a re-import of the identical DLIS file — the far more common case (a DLIS curve reusing a mnemonic already present in the well from LAS/standard_curves at run_no NULL) is never flagged, and the shadowed DLIS curve becomes permanently invisible to every module/equation with zero indication to the user

**Area:** A/F — provenance correctness, cross-function curve resolution (audit checklist item explicitly asks to re-check this fix, and item F asks to trace curve provenance both directions)

**Effort:** medium — the warning-only fix is small (one more query + a new result field surfaced in ribbon.ts); the delete/promote UI action is a larger addition (new backend command + Curve Catalog row action).

**Where:** src-tauri/src/dlis.rs lines 189-205 (collision check queries `mnemonic = ?2 AND run_no IS NOT DISTINCT FROM ?3`, i.e. only matches when the SAME run_no already exists); src-tauri/src/equations.rs lines 425-427 and 440 (`fetch_generic_curve_aligned`, doc comment: "among equals the base run (lowest run_no, NULL first) wins"); src-tauri/src/curve_edit.rs lines 109-144 (`locate_curve`, identical NULLS-FIRST precedence, used for every curve read/edit in the app, not just modules); src-tauri/src/db.rs lines 333-390 (`migrate_standard_curves_to_generic_store` — runs on every app launch, backfills GR/RES_DEEP/NPHI/RHOB/DT/SP into the generic store with run_no NULL for any well that has real standard_curves data, which is essentially every LAS-imported well); src/ui/inspectorPanel.ts lines 340-462 (Curve Catalog's generic-store rows are read-only — no delete/promote action; confirmed no `delete_generic_curve`/equivalent command exists anywhere in src-tauri).

**Evidence:** Concrete scenario: a well already has a LAS-imported GR (curve_meta row, run_no NULL — either from LAS import directly, or backfilled by the always-on migration). The user then imports a DLIS whose frame 3 also contains a GR channel. dlis.rs assigns run_no=Some(3) to the new row (by design, to avoid clobbering the LAS row's samples — the fix's stated goal). The collision check compares run_no=3 against the well's existing GR row (run_no=NULL) — no match, so `replaced` stays 0 and the status line reports a clean, unqualified success (`Imported N curve(s)...` with no "replaced" note). But every downstream consumer that resolves "GR" by name — fetch_generic_curve_aligned (equations/modules), locate_curve (curve viewing/editing) — always prefers the NULL-run row per the documented "base run wins" rule, so the freshly-imported DLIS GR is permanently unreachable by any module, equation, or curve-edit operation. It is visible only as a "GR · run 3" row in the read-only Curve Catalog table, with no tooltip explaining the precedence rule and no UI action anywhere to delete the old row or promote the new one. This directly undercuts the fix's stated purpose ("a frame-0 channel no longer silently replaces a same-mnemonic LAS curve... the status line reports 'replaced N existing curve(s)' when a collision does happen") for precisely the collision case a user is most likely to hit — re-running an open-hole DLIS suite (GR/resistivity/neutron/density/sonic/SP) over a well that already has a LAS with the same standard mnemonics.

**Suggested fix:** At minimum, have the collision check also test for an existing row with the same mnemonic under any OTHER run_no (not just the exact same one) and surface a distinct warning (e.g. "N curve(s) imported alongside an existing same-name curve that will take precedence — see Curve Catalog") rather than silence. Longer term, add a Curve Catalog action to delete or promote a specific generic-store run so the user has some in-app way to resolve the shadow once warned.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read every file/line cited and could not refute the finding — every specific claim checks out verbatim against the real source at D:\XX. SandiBumi\src-tauri\src\.

1. dlis.rs lines 189-205 (exact match to cited range): `run_no = Some(meta.run)` (per-frame numbering), and the collision check is `WHERE well_id=?1 AND set_name='RAW' AND mnemonic=?2 AND run_no IS NOT DISTINCT FROM ?3` — i.e. it only flags a collision when the SAME run_no already exists. A DLIS GR landing at run_no=Some(3) against an existing LAS/migrated GR at run_no=NULL produces `collides=false`, so `replaced` stays 0.

2. equations.rs: `fetch_generic_curve_aligned` (fn at line 428, doc comment lines 425-427 reads verbatim "Exact mnemonic matches win over family matches; among equals the base run (lowest `run_no`, NULL first) wins.") and its query at line 440 (`ORDER BY (upper(mnemonic) = ?2) DESC, run_no NULLS FIRST LIMIT 1`) confirm NULL-run rows always win over any higher run_no for the same mnemonic.

3. curve_edit.rs `locate_curve` (lines 109-146, matching the cited 109-144) has the identical `run_no NULLS FIRST` generic-store query, used for all curve view/edit operations. It's actually worse than the finding states in one respect: for the six STANDARD_COLUMNS mnemonics (GR, RES_DEEP, NPHI, RHOB, DT, SP — exactly the ones the migration backfills), `locate_curve` resolves to `CurveStore::Standard` first if the well's `standard_curves` table has any rows at all, so a DLIS-imported GR is shadowed even before the generic run_no ordering is consulted. `fetch_curve_frame` in equations.rs shows the same standard_curves-first precedence for module/equation inputs. This reinforces rather than undercuts the finding.

4. db.rs `migrate_standard_curves_to_generic_store` (lines 333-401, matching cited 333-390) backfills GR/RES_DEEP/NPHI/RHOB/DT/SP into `curve_meta` with `run_no NULL` for any well with real standard_curves data, and the function is invoked/checked on every launch (though a no-op post-migration via `curve_migration_done`).

5. `upsert_curve_meta` keys strictly on (well_id, set_name, mnemonic, run_no) — a different run_no always inserts a new row rather than colliding, exactly as the finding describes.

6. ribbon.ts confirms the exact user-facing consequence: `replacedNote` is only appended when `result.replaced > 0`; with replaced=0 the status line is the plain "Imported N curve(s), M samples into WELL." with no warning.

7. Grepped all of src-tauri for `delete_generic_curve`/`promote`/`delete_curve` — no such command exists anywhere. inspectorPanel.ts's `renderGenericCatalog` renders `genericEntries` (RAW/DLIS rows) as plain read-only `<tr>`s with no action buttons/tooltip; the only Restore/Delete buttons in the file belong to `computed_curves` log-set version history, a separate mechanism, confirmed via grep.

Every code citation, line number, and quoted comment in the finding matched the actual file contents exactly; I found no mitigating logic elsewhere (no additional collision check, no UI affordance, no alternate resolution path) that would prevent the described shadowing. I was unable to refute any part of it.

</details>

---

## Viz / reporting (Composite, Report, LAS export)

### 1. LAS export silently writes all-NULL columns for any computed curve whose stored name isn't already all-uppercase

**Area:** LAS export — cross-function curve resolution (dimension F/B)

**Effort:** small

**Where:** src-tauri/src/export.rs (export_las, lines 34-54 build curve_names, line 93 columns.get(name)) interacting with src-tauri/src/equations.rs fetch_curve_frame (line 318 `let upper = name.trim().to_uppercase();`, inserts under that uppercased key at lines 331 and 355)

**Evidence:** export_las pulls every computed-curve name for the well verbatim from the DB (`SELECT DISTINCT cc.curve_name ... FROM computed_curves cc`, lines 38-44) and pushes it into curve_names as-is (line 51). It then calls `equations::fetch_curve_frame(conn, well_id, &curve_names)` (line 56). Inside fetch_curve_frame, every name is normalized with `.trim().to_uppercase()` before being used as the HashMap key that `columns` is built with (lines 318, 331, 355) — this is a deliberate, documented convention (see the comment at equations.rs:409-410 about equation outputs sometimes being saved lowercase). But back in export.rs's write loop (line 93), the lookup is `columns.get(name)` using the ORIGINAL, non-uppercased `name` from curve_names — not `name.trim().to_uppercase()`. For any user-authored equation whose output_curve contains a lowercase character (fully permitted: EquationDef.output_curve is stored verbatim and the frontend field only `.trim()`s it, per the already-confirmed db-write-versioning-discipline finding), `columns.get(name)` misses the uppercase-keyed entry, falls through to `.unwrap_or(f32::NAN)`, and the ENTIRE column is written to the LAS file as -999.25 at every depth — with the correct-looking curve mnemonic still in the header, so the file looks complete but silently carries no data for that curve. composite.rs demonstrates the correct pattern one file over: `columns.get(&cs.curve_name.trim().to_uppercase())` at composite.rs:403. export.rs has zero unit tests (no #[cfg(test)] block at all), so nothing catches this.

**Suggested fix:** In export.rs's data-row loop, look up with the same normalization fetch_curve_frame uses: `columns.get(&name.trim().to_uppercase())` (keep `name` as-is for the printed header mnemonic/units). Add a unit test exporting a well with a mixed-case computed curve (e.g. "Vsh_final") and asserting the ASCII section is not all NULL_VALUE for that column.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual source at D:\XX. SandiBumi\src-tauri\src\export.rs and D:\XX. SandiBumi\src-tauri\src\equations.rs (the project referenced by the finding; confirmed this is the SandiBumi/Arshilla repo per user memory) and could not refute the claim — every detail checks out against the real code.

Verified facts:
1. export.rs lines 34-53: curve_names is built from the six standard uppercase mnemonics plus a `SELECT DISTINCT cc.curve_name ... FROM computed_curves` query (lines 38-44) that pushes the curve_name verbatim, no case normalization (line 51).
2. export.rs line 56 calls `fetch_curve_frame(conn, well_id, &curve_names)`.
3. equations.rs fetch_curve_frame (starts line 277): line 318 computes `let upper = name.trim().to_uppercase();`. All insertions into the `columns: HashMap<String, Vec<f32>>` use this uppercased key — line 331 (`columns.insert(upper, col.clone())` for standard curves) and line 355 (`columns.insert(upper, values)` for resolved/computed curves). There is no code path that inserts under the original, non-uppercased name.
4. Back in export.rs, the data-row loop at line 93 does `columns.get(name)` using the original `name` from `curve_names` — NOT `name.trim().to_uppercase()`. For any curve name containing a lowercase character, this lookup misses, falls through `.unwrap_or(f32::NAN)`, and every value for that curve is written as NULL (-999.25) via `fmt()` (lines 64-70), while the header still lists the correct-looking mnemonic (lines 86-88).
5. Verified the premise that mixed-case output_curve names are actually reachable: src/ui/inspectorPanel.ts line 253 only `.trim()`s the equation output_curve field (`output_curve: val("#eq-output").trim()`) with no case normalization; equations.rs stores it verbatim in the `equations` table (line 204) and writes it verbatim into `computed_curves` via write_computed_curves_versioned (line 1068 passes `equation.output_curve.as_str()`, and the appender at line 599 writes `name` as-is). db.rs confirms `curve_name VARCHAR NOT NULL` (line 129) with no case-normalizing constraint/trigger/generated column anywhere in the schema.
6. Confirmed the cited correct-pattern comparison: composite.rs line 403 does `columns.get(&cs.curve_name.trim().to_uppercase())` — the normalization export.rs is missing.
7. Confirmed export.rs has zero `#[cfg(test)]` blocks (grep found none), matching the "no unit tests" claim.

Net effect matches the finding exactly: a user-authored equation with a mixed-case output_curve (e.g. "Vsh_final") will have its computed values silently exported as an all-NULL (-999.25) column in the LAS file, with a correct-looking header mnemonic, because export.rs's lookup key doesn't match the uppercased key that fetch_curve_frame actually stores under. I found no mitigating logic (no schema constraint, no other normalization step, no case-insensitive HashMap wrapper) that would prevent this. The suggested fix (`columns.get(&name.trim().to_uppercase())` in the export.rs write loop) is correct and directly analogous to the existing composite.rs pattern.

</details>

### 2. Report generator's Render/Save/Batch actions persist FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY to the database but never bump dataVersion or record to History

**Area:** Report generator — frontend wiring (dimension D) / cross-function shared state (dimension F)

**Effort:** small

**Where:** src-tauri/src/report.rs report_pages (lines 368-380, run_pay_summary called with stats_only:false, skip_version:true) invoked from render_report/render_report_pdf/export_report_batch; src/ui/reportDialog.ts (renderBtn/pdfBtn/pngBtn/batchBtn handlers, lines 270-400) and src/ui/compositeDialog.ts (renderBtn/saveBtn/pdfBtn handlers, lines 182-266) — neither file imports or calls recordProcess or appState-data-change notification anywhere

**Evidence:** report.rs deliberately writes the pay-summary flag curves in place every time a report is rendered (comment at line 376-377: "report render side-effect" / "report persists FLAG_* in place (unchanged behavior)"), and this runs on every Render click in the dialog, every Save PDF, and once per well in Batch — a real computed_curves write, not a pure preview. Neither reportDialog.ts nor compositeDialog.ts calls recordProcess anywhere in the file (grepped, zero matches), even though every other export/import/edit path in the codebase does — ribbon.ts:932 records LAS export (`recordProcess("Export", ...)`), plotExport.ts:89/97 records plot image exports, and REVIEW.md's own 2026-07-20 "exhaustive recordProcess() sweep" enumerates DLIS/SCAL/core/tops imports, equation/ML/Monte Carlo/workflow runs, zone/top edits, cutoff saves, map-group assignment — composite/report exports are absent from that list. Separately, ipc.ts's renderReport/exportReportPdf/exportReportBatch (lines 243-255) are bare `invoke()` calls with no bumpDataVersion/notifyDataChanged wrapper, so nothing tells other open panes (a Log View showing a FLAG_PAY track, the Curve Catalog, another module's input picker) that new curves now exist for that well. Net effect: generate a report, and the FLAG_* curves it silently wrote are invisible everywhere else in the UI until some unrelated action bumps dataVersion, and there is no History entry showing the report (or its DB write) ever happened.

**Suggested fix:** After a successful render/export in reportDialog.ts, call recordProcess("Export", ...) (matching ribbon.ts's LAS-export convention) and bump appState.dataVersion (or call the same notifyDataChanged() hook workspace.ts uses for module runs) so the newly written FLAG_* curves show up elsewhere without an unrelated refresh. Add recordProcess to compositeDialog.ts's Save SVG/PDF handlers too for History-panel consistency (composite itself has no DB side effect, so no dataVersion bump is needed there).

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read every file the finding cites and could not refute any part of it.

1. src-tauri/src/report.rs, report_pages() (lines 367-380): calls run_pay_summary(db, PaySummaryRequest{..., skip_version: true, stats_only: false, ...}), with the exact comments quoted in the evidence ("report render side-effect — don't version the pay flags" / "report persists FLAG_* in place (unchanged behavior)"). This function runs inside render_report, render_report_pdf, and export_report_batch (all three call report_pages via the same code path), so it fires on every Render click, every Save PDF, and once per well in Batch — confirmed by reading render_report/render_report_pdf/export_report_batch directly below it.

2. src-tauri/src/workflow.rs run_pay_summary (lines 473-574): with stats_only:false and skip_version:true, it takes the `if !req.stats_only { if req.skip_version { ... equations::write_computed_curve(...) for FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY } }` branch — a genuine, unconditional DB write into the same computed-curve store used everywhere else (confirmed by reading equations::write_computed_curve, which calls write_computed_curves_batch, the same primitive backing normal module outputs). This is not a "preview" — it always executes when reportDialog.ts calls render/export.

3. src/ui/reportDialog.ts: grepped and read the full file — renderBtn (270), pdfBtn (296), pngBtn (327), batchBtn (374) handlers contain zero calls to recordProcess and zero calls to bumpDataVersion; the file doesn't even import bumpDataVersion from ../state (only appState, filterByActiveGroup are imported) nor recordProcess from ../processLog.

4. src/ui/compositeDialog.ts: same check — zero recordProcess calls, and confirmed separately that composite.rs has zero DB write calls (INSERT/UPDATE/write_computed_curve/create_log_set all absent), so composite itself is a pure read/render with no persistence — matching the finding's own caveat that composite needs no dataVersion bump, just recordProcess for History-panel parity.

5. src/ipc.ts lines 243-255: renderReport/exportReportPdf/exportReportBatch (and renderComposite/exportCompositeSvg/exportCompositePdf at 203-216) are bare `invoke<...>("command", {...})` calls with no wrapper — confirmed `invoke` is imported directly from @tauri-apps/api/core with no local wrapper that bumps dataVersion or logs history anywhere in the codebase.

6. Cross-checked the established convention: state.ts defines bumpDataVersion(); cutoffDialog.ts and summaryDialog.ts (the "explicit Cutoffs & Summary run" mentioned in workflow.rs's own comments) both call bumpDataVersion() after a pay-summary/cutoff write, and cutoffDialog.ts also calls recordProcess. ribbon.ts:932 and plotExport.ts:89/97 confirm recordProcess is the standing convention for export actions. REVIEW.md's 2026-07-20 "Processing history now covers every operation" entry (line 224-233) lists exactly the import/edit/run kinds it swept into recordProcess and does not mention composite/report — and a separate REVIEW.md entry (line 244-253) explicitly confirms, in the developer's own words, that "the report passes set skip_version ... so they keep overwriting in place — no version churn per refresh," verifying the write is intentional but says nothing about wiring recordProcess/dataVersion for it.

7. Confirmed the "invisible elsewhere" consequence is real, not exaggerated: logViewPanel.ts subscribes to appState.dataVersion (line 287) as its only non-well-switch reload trigger; several other panels (mapPanel, moduleDialog, pickettPanel, histogramPanel, correlationPanel, dbInspectorPanel, workspace.ts) do the same. Field Dashboard, by contrast, was fixed to use stats_only:true (write-nothing) specifically to avoid this exact side-effect/notify problem — so the Report path is the one place in the current codebase that still writes FLAG_* curves with no notification and no audit trail.

Every specific file/line/mechanism claim in the finding matches what's actually in the repository at D:\XX. SandiBumi (the "SandiBumi" project's real folder name per user memory). I found no contradicting code path (no hidden event emission, no global invoke wrapper, no alternate history/version mechanism) that would refute it.

</details>

### 3. Report generator's "Tables only" mode still does the full composite computation — it only skips appending the result

**Area:** Report generator — backend efficiency vs. documented behavior (dimension B/E)

**Effort:** medium

**Where:** src-tauri/src/report.rs report_pages, lines 291-300 (composite::render_pages called unconditionally) vs. lines 427-430 (`if !spec.tables_only { pages.extend(...) }`)

**Evidence:** report_pages always calls `composite::render_pages(&conn, &spec.composite)` (line 298) before ever checking spec.tables_only — this fetches every requested curve across the well's full depth range via fetch_curve_frame and builds the complete per-page DrawOp geometry (curve polylines, zone bands, depth grids, track headers) for every composite page, exactly as the full (non-tables-only) report does. `spec.tables_only` is only consulted afterward, at lines 428-430, to decide whether to append those already-built pages to the final output — the expensive data-fetch and DrawOp-construction work already happened either way and is simply discarded when tables_only is set. This contradicts ROADMAP.md line 1280's checked-off claim, "Tables only checkbox skips the composite pages (fast parameter/pay-summary handout)": the composite pages are computed, not skipped — only their inclusion in the output is skipped. This directly undercuts the Batch button's main use case for this checkbox (generating quick tables-only handouts across many wells): export_report_batch calls render_report_pdf per well, so a tables-only batch run over a field of wells pays the same fetch_curve_frame + per-page geometry cost per well as a full composite+report batch would, for zero benefit beyond a smaller output file.

**Suggested fix:** Guard the `composite::render_pages` call (or at least its expensive per-page DrawOp construction) behind `!spec.tables_only`, and derive the cover's interval directly from the well's curve depth extents (already available via a cheap query) instead of from the full page list when tables_only is set.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Read the real source and independently verified essentially the entire evidence chain.

1. src-tauri/src/report.rs, fn report_pages (lines 287-434): line 298 calls `composite::render_pages(&conn, &spec.composite)?` unconditionally, inside the initial `{ let conn = db.lock()...; ... }` block, before `spec.tables_only` is ever inspected. `spec.tables_only` is first read at line 428 (`if !spec.tables_only { pages.extend(composite_pages.into_iter().map(|p| p.ops)); }`), purely to decide whether to append the already-built `composite_pages` to the output `pages` Vec. This exactly matches the claimed lines/behavior.

2. src-tauri/src/composite.rs, fn render_pages (lines 178-229) — the function called at line 298 — does the full expensive work regardless of any tables-only concept (it has no such parameter): `fetch_header`, a full `equations::fetch_curve_frame` scan of `standard_curves` for every requested curve across the well's complete depth range (lines 277-308 in equations.rs, a plain `SELECT ... FROM standard_curves WHERE well_id = ?1 ORDER BY depth` with no LIMIT), `list_tops`/`list_zones`, and then per-page `build_page` calls that construct the full DrawOp geometry (headers, track headers, depth grids, curve polylines via `value_frac`, edge fills, zone bands — build_page runs ~300+ lines of geometry code). This is the identical function used by the non-tables-only `render_composite`/`render_composite_pdf` (lines 232-251), confirming the claim that tables-only pays "exactly as the full report does."

3. `export_report_batch` (line 467) loops over well_ids and calls `render_report_pdf` per well (line 478), which itself calls `report_pages` (line 460) — so the same unconditional `composite::render_pages` cost is paid once per well in a tables-only batch run, exactly as claimed.

4. The documented claim exists verbatim — but in REVIEW.md, not ROADMAP.md as the finding states. REVIEW.md lines 1280-1281 read exactly: "- [x] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary handout)." ROADMAP.md is only 924 lines long and contains no such text at all, and no occurrence of "tables_only"/"tables only". This is a citation error in the finding (wrong filename) — the substantive claim (a checked-off doc entry asserting the composite pages are "skipped" when they are actually computed and merely not appended) is verified true, just in REVIEW.md rather than ROADMAP.md.

5. Confirmed there is no test that exercises `report_pages`/`render_report_pdf` end-to-end with `tables_only: true` against a live connection — the only `tables_only: true` test in report.rs (line 530) calls `cover_page` directly with a hand-supplied interval tuple, never touching `composite::render_pages`, so the gap between documented and actual behavior was never caught by tests. This supports the finding's premise that the inefficiency has gone unnoticed.

Net: the core technical claim (composite pages are always fully computed via composite::render_pages; tables_only only gates whether they're appended to the output; batch pays the same per-well cost either way) is confirmed by direct code inspection at the exact cited line numbers. The only inaccuracy is the finding's citation of "ROADMAP.md line 1280" where it should be "REVIEW.md line 1280" — the quoted text and line number are otherwise exactly correct. This is a minor sourcing slip, not a refutation of the underlying defect, so I confirm the finding (with the file-name correction worth flagging back).

</details>

---

## Curve edit / undo

### 1. "Set constant" (and other numeric fields) silently coerce invalid/empty input to 0.0, not an error — a mistyped or cleared field overwrites real curve data with zero and reports full success

**Area:** Frontend wiring / UI-UX (D, E) — curveEditDialog.ts

**Effort:** small

**Where:** src/ui/curveEditDialog.ts lines 86-97 (request construction) interacting with src-tauri/src/curve_edit.rs lines 405-410 (apply_op "set" branch) and lines 358-364 (apply_in_range)

**Evidence:** curveEditDialog.ts builds the request as `delta: parseFloat(deltaInput.value) || 0`, `top: parseFloat(topInput.value) || 0`, `bottom: parseFloat(bottomInput.value) || 0`, `value: parseFloat(valueInput.value) || 0`, `add: parseFloat(addInput.value) || 0` — `parseFloat("")` and any unparsable content yield NaN, and `NaN || 0` silently becomes 0. Only `mul` gets a real guard afterward (`if (!Number.isFinite(req.mul!)) req.mul = 1;`), showing the author knew parseFloat can fail but didn't apply the same care to the other fields. On the backend, `apply_op`'s "set" branch only rejects when `!req.value.is_finite()` (curve_edit.rs:406) — but 0.0 IS finite, so a blanked/mistyped Value field is indistinguishable from an intentional 0 and sails through `apply_in_range(depth, value, top, bottom, |_| req.value)`, unconditionally overwriting every sample in the interval (including ones that weren't NaN) with 0.0. `edit_curve` then reports a normal success (`affected: N`, no error), and curveEditDialog.ts shows "N samples changed (Ctrl+Z undoes)" — nothing distinguishes this from a deliberate edit. The Value field is pre-filled with "0" by `num(0)` (line 41), so a user who clears it to retype and is interrupted, or double-clicks Apply too fast, gets a silent zero-write instead of an error.

**Suggested fix:** Parse with `Number(...)`/`parseFloat` and check `Number.isFinite` explicitly for every field before building the request (as already done for `mul`); when a field fails to parse, block Apply and show an inline "enter a value" hint instead of silently substituting 0. At minimum, the "set" op's Value field needs this since 0 is an active, data-altering value with no safe default.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Verified against the real files at D:\XX. SandiBumi\. In src/ui/curveEditDialog.ts (lines 90-97, inside the Apply click handler), the request is built with `parseFloat(x.value) || 0` for delta/top/bottom/value/add — parseFloat of an empty or unparsable type=number input yields NaN, and `NaN || 0` silently becomes 0 in JS. Only `mul` receives a subsequent `Number.isFinite` guard (line 97); the others do not. There is no <form> element anywhere in the dialog (confirmed by reading src/ui/modal.ts's openModal/formRow, which build plain divs), so no native browser validation intercepts this, and the Tauri command wrapper in lib.rs's `edit_curve` is a bare pass-through with no extra validation.

In src-tauri/src/curve_edit.rs, the "set" branch of apply_op (lines 405-410, verbatim match) only checks `!req.value.is_finite()` — and 0.0 is finite in Rust, so a coerced zero is indistinguishable from an intentional 0 and passes straight through. apply_in_range (lines 358-364, verbatim match) then unconditionally overwrites every sample in [top,bottom] with the constant via `|_| req.value`, ignoring the previous value entirely. edit_curve reports `Ok(CurveEditResult{affected: N, ...})` with no error, and curveEditDialog.ts's success handler (~lines 100-124) treats this as a normal successful edit, pushing an undo entry and showing "N samples changed (Ctrl+Z undoes)" — with no indication anything was wrong.

Every cited line number matches the actual file content exactly, and the full causal chain (mistyped/cleared field → parseFloat NaN → `||0` coercion → passes the is_finite() check → unconditional range overwrite → silent full-success report) is real and unmitigated by any other validation layer in the codebase. I could not find any refuting detail.

</details>

### 2. restore_curve_values (the undo path) has no staleness/version check — an old edit's Ctrl+Z can silently overwrite a curve that's been legitimately recomputed since, and the frontend never checks how many samples actually got restored

**Area:** Backend correctness / Frontend wiring (A, B, D) — curve_edit.rs + curveEditDialog.ts

**Effort:** medium

**Where:** src-tauri/src/curve_edit.rs lines 457-486 (restore_curve_values) and lines 274-299 (write_curve_inner's Computed branch, which only passively carries set_id forward); src/ui/curveEditDialog.ts lines 111-121 (pushUndo's undo/redo callbacks)

**Evidence:** `edit_curve` snapshots (depth, old-value) pairs at apply time and hands the raw bytes to the frontend, which can replay them via `restoreCurveValues` arbitrarily later (any time before the undo stack rolls off, LIMIT 100). `restore_curve_values` re-resolves the store fresh via `locate_curve` and matches purely by `depth.to_bits()` against whatever is CURRENTLY stored (lines 469-480) — it never checks the current `set_id`/log_sets version against anything, so it cannot tell the difference between "nothing has touched this curve since" and "this curve was re-run/re-imported/re-edited since, and now belongs to an entirely different, newer log_sets version." Since the project's own design explicitly keeps module/equation runs OUT of the undo stack (`src/undo.ts` header: "Module runs are intentionally NOT undoable"), a rerun never invalidates a pending curve-edit undo entry — so an old, stale Ctrl+Z can splice pre-run values back on top of a fresh legitimate recompute with zero warning. Compounding this: curveEditDialog.ts's undo callback (`await restoreCurveValues(...); bumpDataVersion();`) discards the `Promise<number>` restored-count entirely (ipc.ts:1062 returns the real count `n`), so even a 0-row restore (e.g. after the curve was deleted/renamed, or after a store-precedence change) is reported to the user as an unqualified "Undo: <label>" success via undo.ts's generic `.then((label) => setStatus(...))`.

**Suggested fix:** Have edit_curve record the well/curve's current set_id (or a simple write-generation counter) alongside the snapshot bytes, and have restore_curve_values verify it still matches before writing — refusing (or warning) instead of silently merging stale values into newer data. Separately, curveEditDialog.ts's undo callback should check the returned restored-count against `pointCount` and surface a distinct status/error when it doesn't match (e.g. "undo only restored 3 of 21 samples — curve changed since this edit") rather than always reporting success.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

I read the actual files (D:\XX. SandiBumi\src-tauri\src\curve_edit.rs, D:\XX. SandiBumi\src\ui\curveEditDialog.ts, D:\XX. SandiBumi\src\undo.ts, D:\XX. SandiBumi\src\ipc.ts, D:\XX. SandiBumi\src-tauri\src\lib.rs, D:\XX. SandiBumi\src-tauri\src\equations.rs, D:\XX. SandiBumi\src-tauri\src\db.rs) and could not refute any part of the finding — every cited line matches and the mechanism is real, not hypothetical.

Verified point by point:

1. `edit_curve` (curve_edit.rs:427-452) does snapshot pre-edit (depth, old-value) pairs and hand raw bytes to the frontend (`pack_pairs`, line 450) with no session/version tag attached to the payload — just point_count + bytes.

2. `restore_curve_values` (curve_edit.rs:457-486) re-resolves the store via `locate_curve` (line 467), reads current samples, and builds a `HashMap<u32,f32>` keyed purely on `depth.to_bits()` (lines 469-473), then splices matched values back (lines 474-480). There is no read or comparison of `set_id`, `log_sets.version`, or any generation counter anywhere in this function or in its Tauri wrapper (lib.rs:732-745, which is a bare pass-through). Confirmed exactly as claimed.

3. `write_curve_inner`'s `Computed` branch (curve_edit.rs:274-299) reads each row's existing `set_id` and re-inserts it unchanged ("Keep each row's set_id ... across the rewrite") — it only passively carries the value forward, never validates it against anything. Confirmed exactly as claimed.

4. `db.rs` (lines 133-141) and `equations.rs` (`write_computed_curves_versioned_batch`, lines 675-724) confirm `computed_curves` has a genuine versioning scheme: every module rerun gets a fresh `set_id`/incremented `version` in `log_sets`, and reruns normally reuse the well's existing depth grid — meaning depth bits between a stale pre-edit snapshot and a freshly recomputed curve will typically still match, so the bit-exact-depth-only match in `restore_curve_values` really can splice stale values onto newer, differently-versioned data.

5. `src/undo.ts` header (lines 1-3) literally states "Module runs are intentionally NOT undoable — they're deterministic and re-runnable," and `clearUndoStacks()` is only invoked from ribbon.ts (project switch), never on a module rerun — so a pending curve-edit undo entry survives an intervening rerun untouched, exactly as the finding describes.

6. Git history confirms the ordering that makes this a real gap rather than a designed tradeoff: the curve-edit/undo feature (commit ca45c07, "P2-d ... right-click curve editing") predates the log-set versioning feature (commit d5df3cd, "P1-c log-set versioning") — the undo mechanism was never revisited after versioning was introduced.

7. On the frontend side, `curveEditDialog.ts`'s undo callback (lines 111-121) does `await restoreCurveValues(wellId, curveName, pointCount, prevBytes); bumpDataVersion();` — the `Promise<number>` return (the real restored count `n`, per `ipc.ts:1062` and the Rust command's `Result<usize,String>`) is discarded entirely, never compared to `pointCount`. `undo.ts:46-53`'s `undo()` wrapper likewise just awaits `action.undo()` and unconditionally returns `action.label`; `installUndoHotkeys` (undo.ts:96) then does `setStatus(label ? \`Undo: ${label}\` : ...)`. So even a 0-of-N restore reports an unqualified "Undo: <label>" success, exactly as claimed. `bumpDataVersion()` (state.ts:93-95) is just a counter increment with no validation that could catch this.

I found no mitigating check anywhere in the call chain (Rust command layer, store-write layer, or frontend undo plumbing) that would prevent or surface the staleness scenario described. The finding's code citations, line ranges, and causal narrative all hold up against the real source.

</details>

### 3. locate_curve's computed-curve lookup picks an arbitrary case variant (LIMIT 1, no ORDER BY) — when the already-confirmed case-duplicate shadow-row bug has occurred, the edit tool can silently operate on the row set the log view isn't even showing

**Area:** Cross-function / substrate interaction (F) — curve_edit.rs, consequence of the known db-write-versioning-discipline finding

**Effort:** small (local mitigation) / tied to substrate fix (root cause)

**Where:** src-tauri/src/curve_edit.rs lines 128-137 (locate_curve's Computed resolution: `SELECT curve_name FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = ?2 LIMIT 1`, no ORDER BY) vs. src-tauri/src/equations.rs lines 376-395 (fetch_computed_curves_batch's upper()-merged, "last row wins per depth" read used for display)

**Evidence:** This is not a new root cause — it's the already-confirmed [db-write-versioning-discipline] substrate issue (computed_curves writers DELETE by exact curve_name while reads resolve case-insensitively, leaving stale shadow rows) surfacing specifically in this tool: `locate_curve` picks whichever cased `curve_name` row happens to scan first (no ORDER BY, so effectively arbitrary/insertion-order-dependent) and then `read_curve`/`write_curve_inner` operate exclusively on that exact cased name (curve_edit.rs lines 164-170, 274-299). The display path (equations.rs) instead merges ALL case variants together and lets the last-scanned row win per depth. If a curve has a stale shadow row from a re-cased equation output, the edit dialog can therefore silently patch the row set that the log view is NOT currently displaying — reporting "N samples changed" with zero visible effect after `bumpDataVersion()`, or vice versa affecting depths the viewer isn't currently preferring.

**Suggested fix:** The durable fix is the already-flagged substrate one (normalize curve_name casing on write). As a local mitigation, locate_curve could at least resolve to the same row set the display logic would show (e.g. prefer the name with the most recent/complete rows, or the name matching after normalization) rather than an unordered LIMIT 1.

<details><summary>Verifier evidence (independently re-checked, could not refute)</summary>

Independently read src-tauri/src/curve_edit.rs (lines 90-320) and src-tauri/src/equations.rs (lines 349-423), plus db.rs schema comments and the equation-save path, in D:\XX. SandiBumi. All specific claims check out verbatim:\n\n- locate_curve's Computed-store lookup (curve_edit.rs ~line 130) really is `SELECT curve_name FROM computed_curves WHERE well_id = ?1 AND upper(curve_name) = ?2 LIMIT 1` via query_row, with no ORDER BY — an arbitrary/scan-order-dependent pick when duplicate case-variant rows exist.\n- read_curve (line 166) and write_curve_inner's Computed arm (lines 277-278, 288-289) then operate exclusively on that one exact cased curve_name (SQL literal / `curve_name = ?2`), never the case-insensitive set.\n- The display path, fetch_computed_curves_batch (equations.rs, query at ~374-378, merge at ~388-397), queries `upper(curve_name) IN (...)` and explicitly comments "last row wins per depth" — merging ALL case variants via a separately scan-order-dependent resolution.\n- These are two independent, uncoordinated resolutions of the same duplicate rows, so the edit tool can genuinely operate on a different row set than what the viewer currently shows/prefers.\n- Checked the precondition (case-duplicate shadow rows are structurally possible): computed_curves has no PK by design (db.rs ~118-125), uniqueness is asserted only via exact-string DELETE-then-append discipline, and equation output_curve names (equations.rs line 19/1068) are raw user strings never case-normalized before being written — so a re-cased equation output can strand an old-case row exactly as the cited substrate bug describes.\n- The described frontend symptom ("N samples changed" + bumpDataVersion) matches curveEditDialog.ts lines 101-124 verbatim.\n\nI found no ORDER BY, uniqueness constraint, or case-normalization anywhere that would refute the mechanism. The finding is accurate and should stand as CONFIRMED.

</details>

---

## Appendix — refuted findings (not actionable, listed for transparency)

3 findings were raised by a review agent and then independently refuted by a separate verifier that re-read the actual code and found the claim didn't hold. Titles only:

- [workflow-chain-engine] MASK output-blanking in the workflow runner defeats log_predict's entire purpose (fill-the-masked-gap), contradicting REVIEW.md's untested claim that this was fixed
- [prep-correction-group] moduleDialog.ts never calls recordProcess — every run of the 7 audited Prep tools (and all other VSH/Porosity/Saturation/Permeability modules run from their own pane) is invisible to the History panel
- [prep-correction-group] REVIEW.md overstates nphimat's chart-digitization fidelity by ~15x (claims 0.04 pu, code/tests say 0.6 pu)
