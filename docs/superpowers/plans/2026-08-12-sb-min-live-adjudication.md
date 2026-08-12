# SB-MIN Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 46 live `SB-MIN` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 44 acceptance-test intentions, and preserve every unresolved source or product decision without changing mineral-solver behavior.

**Architecture:** This is a documentation-only evidence pass over SandiMin's bounded solver, endpoint library, fluid and clay helpers, constraints, output persistence, retired predecessor, UI/IPC custody, Results QC, workflow refusal, export provenance, manual matrix, and reachable Git history. The immutable PRD supplies the intended contract, original priority, chapter status, parameter custody, and named test intentions; current code, independently derived tests, manual evidence, and reachable history supply the separate live verdict recorded in the takeover ledger and one SB-MIN evidence receipt. A passing helper test, a self-forward-modelled fixture, generic log-set provenance, or a correctly retired compute path counts only for the observable limb it actually proves.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Petrophysical math, parameter custody, data-integrity judgments, and final sign-off remain with the primary session.
- Work only in `D:\XX. SandiBumi`. The sole registered Git worktree MUST remain that path. `D:\XX. SandiBumi-check` is not a repository or evidence source.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `5129b4ea42669469796c7f4763741aaba6d32769`; fetched `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution and stop to reconcile if any moves.
- The local branch is `codex/g1-sb-min-adjudication`. The serial Gate 1 chain remains intentionally local and unpushed; do not switch, merge, rebase, rewrite history, push, or open a pull request.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, exact tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/13_mineral-solver.md`, applicable `docs/record_parallel_lanes.md`, `docs/record_data_tools.md`, and `docs/record_fixes.md`, the current manual matrix, and the existing takeover receipts/status.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. All 46 rows have populated chapter status and owned-test mappings; this lane may change only live adjudication fields.
- The chapter and ledger agree on 46 contiguous requirements: `P0=10`, `P1=11`, `P2=16`, `P3=9`. Chapter states are `PRESENT-OK=5`, `PRESENT-UNVERIFIED=1`, `PRESENT-DIVERGENT=7`, `PARTIAL=6`, and `ABSENT=27`. Reverify live behavior independently; do not copy these states into the live verdict.
- The chapter defines 44 contiguous acceptance tests, `SB-MIN-T01` through `SB-MIN-T44`. Route every intention exactly once as a primary proof target. Shared ownership is allowed only where the chapter maps one test to multiple requirements; list cross-support separately and never count a test merely because its ID appears in an `owned_tests` cell.
- Section 5 contains exactly 78 parameter rows. Ten are `ABSENT — ships with no default`; five more are `NON-ADOPTABLE — cited for verification`; 16 values are vendor-derived and remain subject to `SB-CORE-005` resourcing. Inspect every row and preserve each status literally.
- Four values that currently ship are nevertheless source-absent: generic `Clay` CEC `0.00`, generic `Clay` WCLP `0.120`, function-local bound-water density `1.000`, and universal `VP/VS = 1.7`. Treat them as live silent-wrongness evidence; never convert code presence into parameter authority.
- `ESC-1` through `ESC-8` remain distinct: conductivity root convention, default matched CEC/WCLP library, WBM iteration/cap behavior, Shell constant, two source-less fluid constants, clay-density-triple handling, per-clay `Rsh` parity, and universal `VP/VS`. No current literal or test may settle one.
- `ESC-2` is a product-owner/source adjudication: no agent may choose Geolog, Techlog, a hybrid, an average, or a generated default library. Chlorite and Glauconite Geolog-pair WCLP values remain absent.
- `ESC-3` has no cited iteration cap. Record the shipped one-pass equality behavior and the specified inequality behavior separately; do not invent a cap or completion policy.
- `ESC-4` preserves the three incompatible Shell constants `0.018`, `0.019`, and `0.19` with sources while shipping no default. Do not round, average, or select one.
- `OPEN-8` is an immutable-PRD count defect: the front matter says 34 tests, 63 parameters, and 9 P0 while the chapter actually contains 44 tests, 78 parameters, and 10 P0. Record the mismatch; do not edit the PRD.
- `OPEN-9` leaves `SB-MIN-T32`'s cross-toolchain bit-identical replay tolerance undefined. A same-build replay cannot prove the broad guarantee.
- `OPEN-10` deliberately has no current acceptance-test surface because the relevant vendor constant-equation bound-water forms are absent. Do not invent a requirement or test.
- Every petrophysical parameter is CITED or ABSENT. A source-code literal, neighboring-vendor value, remembered textbook value, existing fixture, average, midpoint, rounded number, or plausible interpolation is not a citation.
- `as_built_status` answers what the accepted tree currently ships. `release_disposition` answers whether the observable contract is required for the paid offline Windows open-hole-petrophysics pilot. Priority and chapter status inform but do not mechanically decide either field.
- A compound requirement is not `PRESENT-OK` because one helper, output, engine, or UI control is correct. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, a list, a provenance/export clause, or a refusal/control pair.
- A test is qualifying owned acceptance proof only if it exercises the observable contract and derives the expected value independently from a cited source or explicit arithmetic. An internal `Result`, source-text grep, compile success, same-code round trip, self-forward-modelled synthetic truth, or current-library snapshot is supporting evidence only.
- Classify each requirement's proof as `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING` under `CONTRACT.md` sections 3 and 6. A test pinning current divergent behavior is divergence evidence, never proof of the specified behavior.
- The current SandiMin test module contains 42 tests: 41 pass and one optional real-data test is ignored. The exact retirement, workflow-refusal, and LAS-provenance tests also pass. These counts are candidate evidence, not 44 automatic requirement closures.
- `pef_converts_to_u_before_mixing` uses current library endpoints to construct its own target. It proves the implementation follows a U route and differs from linear Pe, but it is not the independent quartz/water numeric correctness fixture required by `SB-MIN-T12`.
- The current bounded-solver tests verify non-negativity and a preserved vector dimension only indirectly; `SB-MIN-T01` also requires the returned component names and list length to survive the case where unconstrained LSQ would drive a component to `-0.04`.
- Current unity tests prove a hard sum and bounds but do not prove that X-only fluids have coefficient zero in the unity row. Do not close `SB-MIN-003` from the sum alone.
- Current fluid ceiling is typed correctly in the library and classifier but can revert to `1.0` after a rename through the UI name map and serde default. Classify the end-to-end contract, not the positive library row.
- Current CEC and WCLP routes are both implemented, but absent CEC silently becomes zero, WCLP at or above `0.5` silently switches route, and the shipped clay columns are a cross-vendor chimera. These are three separate observable failures.
- Current endpoint rows have no per-value source, unit, or wet/dry-convention fields. A comment, table column, or component kind is not runtime custody.
- Current `RECON` arithmetic and per-tool residuals exist, but there is no independent long-form Eq 79/80 equivalence test and no IP-comparable `TOTERR_IP`, Geolog `QUALITY`, or `CONDNUM` output.
- Current DOF count and zero-DOF warning are positive evidence. They do not prove conditioning, conflict-row diagnostics, Tool-row DOF behavior, or a trusted/untrusted sample flag.
- A tool disappears when the frontend omits it, but `ToolSpec` has no explicit `active` field, no separate weight multiplier, and no MIN/MAX/printed-default uncertainty record. The hard-coded `BASE_TOOLS` sigmas are shipped literals, not the chapter's sourced uncertainty library.
- Current conductivity exponent `w = 0.75m + 0.25n` is implicit and partially recorded through fluid properties. It is not an explicit named convention and must not be labelled IP parity while `ESC-1` is open.
- Current `dry_clay_calc` hard-codes bound-water density `1.0` and transit time `189.0`; the density is source-absent while the transit time has a source. Preserve the distinction.
- Current constraints implement POROSITY and BNDWAT as soft rows, and WBM as a once-detected weighted equality to zero. There is no iterated inequality, Tool hard-plus-pseudo row, OBM pair, PHIMAX, BVIRR, or IRRWAT solver contract.
- Current outputs include volume, porosity, saturation, moved-hydrocarbon, VSH, RECON, and optional REC/DIF curves. They omit `SXOE`, `PHIE_X`, and `PHIT_X`; curve metadata does not declare wet/dry clay convention; no bare `SW` emission is a positive but partial limb.
- Current log-set persistence records only component names, prefix, Sw model, and input curve names. Generic log-set and export ancestry is supporting evidence, not the fully resolved parameter set required by `SB-MIN-032`.
- The legacy `multimin` module remains resolvable and refuses through the real runner, which is positive. Its retired spec still exposes unshared `RHOB_CLAY 2.55` and `PEF_CLAY 3.10`, so the compound `SB-MIN-041` contract remains divergent.
- Formation-temperature samples outside `32..600 °F` fall back to the scalar, but the number of substitutions is not returned or persisted. The fallback behavior and reporting behavior must be adjudicated separately.
- The manual baseline is separate: `sandimin=0/28`, `results-qc=0/1`, `workflow=0/23`, `delivery-sets=0/33`, `las-export=0/2`, `processing-history=0/7`, `report=6/53`, `security-integrity=0/63`, and `verification-stewardship=0/24`. Automated or desktop-harness evidence cannot close an unchecked manual scenario.
- New receipt/ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record all of the following:

1. branch `codex/g1-sb-min-adjudication`;
2. a clean worktree and the sole registered worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, fetched `origin/master`, merge base, and accepted-anchor reachability;
4. exactly 46 ledger rows, covering `SB-MIN-001` through `SB-MIN-046` once with no gap or duplicate;
5. priority counts `P0=10`, `P1=11`, `P2=16`, `P3=9`;
6. source-status counts `PRESENT-OK=5`, `PRESENT-UNVERIFIED=1`, `PRESENT-DIVERGENT=7`, `PARTIAL=6`, `ABSENT=27`;
7. all 46 source-owned `owned_tests` populated and all 46 live `as_built_status=UNADJUDICATED`;
8. exactly 44 chapter test IDs, `T01` through `T44`, each defined once;
9. exactly 78 parameter rows, 10 `ABSENT — ships with no default`, five `NON-ADOPTABLE`, and the four source-absent values that currently ship;
10. exactly eight escalations, ten acquisition gaps, and ten open items, preserving their distinct triggers;
11. takeover summary `392` adjudicated, `539` unadjudicated, and `292` pilot blockers before any MIN edit;
12. the current manual capability counts listed in Global Constraints; and
13. candidate-test receipt: 41 passing and one ignored `multimin2::tests`, plus the passing retirement, workflow-refusal, and LAS-provenance tests.

The only mechanically predictable post-adjudication ledger count is `438` adjudicated and `493` unadjudicated. Do not predict as-built, pilot-blocker, test-class, or manual-evidence totals before row-by-row classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-min.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/multimin2.rs`, `src-tauri/src/multimin.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/workflow.rs`, `src-tauri/src/resultsqc.rs`, `src-tauri/src/export.rs`, `src-tauri/src/equations.rs`, `src/ui/multiminDialog.ts`, `src/ui/resultsQcPanel.ts`, `src/ipc.ts`, current tests, reachable Git history, manual evidence, and the immutable source chapter
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts

## Evidence Receipt Schema

Create one `### SB-MIN-NNN` section per requirement in numeric order. Every section MUST include:

- **Specified contract:** every observable limb, separated when compound.
- **Current implementation:** exact symbols and paths, plus explicit negative-inventory scope where absent.
- **As-built status:** one legal ledger state and why a stricter state fails.
- **Release disposition and risk:** pilot relevance independent of implementation status.
- **Automated evidence:** exact test names and commands classified `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`, with the independent expected-value source named.
- **Manual evidence:** exact checked capability/scenario or `NONE`; never inferred from automation.
- **Source/parameter boundary:** cited, absent, withdrawn, vendor-derived, non-adoptable, derived, engineering guard, conflicting, or not applicable, with escalation IDs.
- **UI/IPC/provenance surface:** request field, editor control, result object, log-set record, Results QC, workflow, report, and export impact as applicable.
- **History/reachability:** accepted commit evidence or confirmed negative history search where consequential.
- **Blocking decision/dependency:** exact source, legal, product, UI, manual, or implementation dependency.
- **Next action:** the smallest bounded follow-up, or `NONE` only when the whole contract is proved and no field evidence is required.

## Requirement Evidence Map

### Group A — Solver core and structural bounds (`001`–`005`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Bounded non-negative LSQ preserves every named component; no deletion heuristic. | `solve_bounded_lsq`; `run_multimin`; `nonneg_holds_when_truth_is_a_boundary`; `unity_is_exact_and_bounds_hold`; SandiMin history. | Vector bounds alone do not prove the specified `-0.04` case or returned names/list preservation. |
| `002` | Run record discloses bounded constraints versus IP mineral deletion. | `MultiminResult`; log-set `params_json`; UI result copy; history. | An implementation comment or chapter statement is not an observable run record. `OPEN-7(e)` still gates the factual IP deletion claim. |
| `003` | Hard unity over non-X components; X-only coefficient exactly zero. | `unity_of`; `ZoneSets`; `solve_bounded_lsq`; unity tests. | Sum closure does not prove X-only exclusion. Check both sides of the boundary. |
| `004` | Any misfit statistic states whether unity was hard or a Tool row. | `RECON`; `dof_note`; log-set params; Results QC. | A hard-unity solver without the convention beside the statistic is absence. |
| `005` | Every fluid keeps the 0.5 upper bound structurally after rename. | `Component.kind`; `classify`; `max_vol`; library; `maxMap`; serde/UI fallbacks. | Library water at 0.5 is insufficient; the renamed-fluid control must survive request construction. |

### Group B — Bound water, library custody, and U mixing (`006`–`012`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `006` | CEC route uses `α·96·CEC·ρ/(T+298)` and the cited salinity expansion with cap. | `fluid_calc_at`; `bound_water_multiplier`; `bound_water_tracks_clay_volume`; `fluid_calc_matches_reference`. | Inspect arithmetic, threshold, cap, and route use separately; broad approximate fixture tolerances do not replace T05/T06. |
| `007` | Missing clay CEC refuses under CEC route; valid WCLP control still computes. | generic Clay library row; `bound_water_multiplier`; `bndwat_soft_rows`; WCP tests. | Silent `k=0` is a live defect. Do not credit the alternate route as satisfying the refusal. |
| `008` | CEC and WCLP ship only as a matched same-library pair within 2% relative. | `LIB`; `library_has_expected_shape`; both bound-water routes; ESC-2. | A snapshot of the current chimera is divergence evidence, not correctness. Run the Geolog, Techlog, and Chlorite controls independently. |
| `009` | Every endpoint value carries a source string and vendor-derived flag where applicable. | `Component.endpoints`; `LibRow`; IPC/library command; editor; log-set/export provenance. | Column-level comments and generic method provenance do not provide per-value custody. |
| `010` | Every clay row and emitted clay curve declares wet or dry; incompatible mixtures refuse. | `Component.kind`; library; editor; output names/metadata; `dry_clay_calc`. | Kind=`clay` is not a wet/dry convention; inspect rows, request, curve metadata, and refusal. |
| `011` | CEC declares meq/g and refuses outside `[0.01, 2.0]` with a unit hint. | `Component.cec`; IPC number; editor input; validation search. | A tooltip without runtime validation is partial at most; preserve `OPEN-7(a)` unit-source escalation. |
| `012` | PEF forward response mixes volumetric U and rejects the linear-Pe answer. | `pef_to_u`; response-row construction; `pef_converts_to_u_before_mixing`; retired R17 history. | Current-library self-construction is supporting only. The independent 50/50 quartz-water `1.3821` versus `1.085` fixture is the qualifying target. |

### Group C — Misfit, conditioning, and DOF (`013`–`016`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `013` | `RECON` follows the stated long-form Eq 79/80 incoherence identity. | RECON calculation; REC/DIF emission; decomposition tests; `recon_qc_emits_per_tool_curves_and_flags_endpoint_error`. | A residual decomposition using the shipped result is not the independent LargestWeight cancellation proof required by T13. |
| `014` | Emit labelled `RECON`, `TOTERR_IP`, and Geolog `QUALITY`, each with unity convention. | `MultiminResult`; outputs; Results QC; export. | One statistic with heuristic UI thresholds is not three comparable statistics. |
| `015` | Emit condition number; `>8` suspect and `>10` unstable; refuse trusted presentation of collinear split. | bounded solver/KKT path; result/output/flag searches. | A singular internal `None` or skipped sample is not an observable condition number or trusted/untrusted result. |
| `016` | Count DOF, flag zero-DOF validation failure, and account Tool rows correctly. | `dof_note_set_when_exactly_determined`; zero-DOF endpoint test; run result. | Positive zero-DOF evidence does not prove Tool-row accounting; adjudicate the compound contract. |

### Group D — Tool activation, weighting, and uncertainty (`017`–`020`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `017` | Tool-off removes a row and one DOF; huge sigma retains row and DOF. | `ToolSpec`; frontend `tools.filter(t.on)`; weighting; DOF. | UI omission approximates one limb but there is no explicit persisted active state or two-sided test. |
| `018` | Per-tool weight multiplier is separate from sigma and no-op at 1.0. | `ToolSpec`; `weighted`; UI tool grid; IPC. | `1/sigma` weighting is not a separate multiplier. Do not infer the field from sigma editing. |
| `019` | Store MIN, MAX, and printed default uncertainty independently. | `BASE_TOOLS`; endpoint/tool library; request/run record. | Hard-coded default sigma without its MIN/MAX/source fails custody. Never derive the printed default. |
| `020` | Sourced default uncertainty library preserves printed rows and labels nine derived 1.5%-range rows. | `BASE_TOOLS`; tool keys; chapter tables; run record. | Current literals have no per-value source and the per-tool default row is source-absent. Do not bless familiar values. |

### Group E — Conductivity and clay-density inputs (`021`–`024`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `021` | Conductivity root convention is explicit, selectable, and recorded; all three conventions stay distinct. | `FluidCalc.w`; `fluid_calc_at`; `FluidProps`; UI/IPC/log-set params; ESC-1. | Shipping Geolog `0.75m+0.25n` internally is partial. Never claim IP parity while `1/2` versus `1/m` is unresolved. |
| `022` | Shell porosity-dependent-m request refuses without constant and displays all three sourced candidates. | model enum/UI/search; §5 absent row; ESC-4. | The capability is absent; do not implement or choose a constant during adjudication. |
| `023` | Variable `m*` reproduces corroborated Dual-Water coefficients and labels single-sourced Waxman coefficients. | model/function/UI searches; §5 rows. | Ordinary fixed `m` and post-solve Sw models are not variable `m*`. Base `m` remains absent. |
| `024` | Wet↔dry conversion accepts explicit sourced bound-water density and responds numerically. | `dry_clay_calc`; hard-coded `RHO_W`; dry-clay tests/editor. | Current function-local 1.0 is source-absent. Existing KKT snapshots cannot prove the explicit-parameter contract. |

### Group F — Model inputs, units, and persisted custody (`025`–`032`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `025` | Per-equation invasion factor controls X/U fluid response and is persisted. | zone classifier; response-row construction; request/UI/log-set params. | One global categorical zone assignment is not a continuous per-equation factor. Preserve `OPEN-6`. |
| `026` | Named neutron response set is an explicit model input and never inherited from header. | component endpoints; UI/IPC/header searches; run record. | Fixed NPHI endpoints are not a selected named response model. Preserve unread-table gaps. |
| `027` | WCLP is v/v; p.u. input refuses instead of switching to CEC; valid control computes. | `bound_water_multiplier`; WCP editor/IPC; route fallback tests. | The current ceiling branch silently changes method. Pin both 10.4 refusal and 0.104 control. |
| `028` | Named selectable endpoint libraries plus project override surface >5% disagreements with sources. | one hard-coded `LIB`; library command/editor; run params. | A single 27-row table and documentation comparison do not satisfy runtime selection. ESC-2 remains separate from capability. |
| `029` | Fluid sonic endpoint source appears at point of use and alternatives surface. | fluid rows; endpoint editor; `Component.endpoints`; run/export provenance. | `DT=250` in code without source is divergent even when the value matches one vendor. |
| `030` | Silt is first-class and Elan Eq 78 is not mislabeled as compact IP Simandoux. | component library; `SwModel`; saturation helpers; UI/Results QC labels. | A generic mineral named Silt or a different Simandoux helper does not prove Eq 78 or label separation. |
| `031` | Per-clay `Rsh` is consumed where supplied, with zonal fallback. | single `FluidProps.rsh`; clay rows; saturation post-process. | Do not adopt Techlog's non-adoptable Rsh values; the requirement is capability, not defaults. |
| `032` | Persist fully resolved endpoints, sources, options, weights, constraints, seed, units, and inputs so replay is possible. | log-set `params_json`/`inputs_json`; export provenance; request/result. | Component names, prefix, Sw model, and input curve names are materially incomplete. `OPEN-9` limits bit-identical claims. |

### Group G — Constraint feasibility and Tool semantics (`033`–`035`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `033` | Infeasible constraints name every conflicting row and depth. | solver `None`/error paths; per-sample skip; result object/UI. | Generic singular or underdetermined errors do not identify conflict rows. |
| `034` | WBM uses `Sxo≥Sw` inequality, iterates to feasibility, leaves already-valid solves unchanged, and reports cap outcome. | once-only WBM re-solve; `MOVEDHC`; `request_defaults_keep_every_constraint_on`; ESC-3. | Current weighted equality perturbs valid solves. Do not invent iteration cap or policy. |
| `035` | Tool constraint is hard equality plus pseudo-measurement, contributes DOF/misfit, and emits tie residual. | POROSITY/BNDWAT soft rows; sigma constraint; outputs. | A high-weight soft row alone is the documented divergence. |

### Group H — Outputs, uncertainty, routing, and retirement (`036`–`041`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `036` | Complete convention-labelled output set includes `SXOE`, `PHIE_X`, `PHIT_X` and never bare `SW`. | output construction; metadata; UI/Results QC/export. | No bare `SW` is one positive limb; missing X porosity/effective saturation and clay convention keep the whole contract open. |
| `037` | Endpoint Monte Carlo requires explicit integer seed, persists it, and reproduces bands. | SandiMin request/result; generic Monte Carlo is read-only cross-support. | A separate module's Monte Carlo does not implement SandiMin endpoint uncertainty. No clock seed. |
| `038` | Per-volume linearized uncertainty is emitted, nonnegative, scales with misfit, and states endpoint exclusion. | result/output searches; solver matrix internals. | Internal matrices or core RMS do not equal posterior volume uncertainty. |
| `039` | Balanced pre-solve uncertainties are available and labelled. | tool uncertainty handling; solver setup. | Current fixed sigma and post-solve residuals are not the Geolog pre-solve calculation. |
| `040` | CEC and WCLP routes are mutually exclusive and the selected route is persisted. | `PorositySource`; request default; UI radios; log-set params. | Runtime enum is positive; fallback from invalid WCLP to CEC violates exclusivity, and incomplete persistence may violate custody. |
| `041` | Retired module resolves, refuses through real runner with replacement, and exposes no orphan default. | `multimin_spec`; `retired_module`; `run_module`; workflow test; retirement test. | T40 passes, T41 fails on `RHOB_CLAY` and `PEF_CLAY`; classify the compound requirement as divergent. |

### Group I — Additional constraints, units, temperature, and density gate (`042`–`046`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `042` | OBM inequality pair constrains X versus U hydrocarbon volumes. | mud-type branch; WBM-only constraint code; outputs. | A mud-type string or WATER-only branch is not OBM behavior. |
| `043` | Opt-in PHIMAX, BVIRR, and IRRWAT ceilings/floors are explicit and recorded. | request/constraint/UI searches; generic `phimax` module only as cross-support. | A separate porosity module or component `max_vol` is not a SandiMin constraint row. |
| `044` | Canonicalize request units and prove metric/imperial invariance to `1e-6 v/v`. | raw numeric IPC types; endpoint units; conversions; tests/history. | Comments and conventional units do not enforce the boundary; test all four named traps. |
| `045` | Formation temperature is bounded, invalid samples fall back, and substitution count is returned/persisted. | `FTEMP_MIN_F/MAX_F`; curve fallback tests; `MultiminResult`; log-set params. | Current fallback passes but count/report limb is absent. Do not infer count from fixture construction. |
| `046` | Clay density triple validates Eq 11 within 1% and surfaces vendor inconsistencies without adopting them. | library fields; `dry_clay_calc`; editor/validation. | Current library lacks a triple gate. Techlog values are non-adoptable fixtures only; preserve ESC-6 handling decision. |

## Acceptance-Test Ownership Map

Route each chapter test exactly once as a primary intention during execution:

| Test | Primary requirement(s) | Current candidate evidence | Minimum qualifying check |
|---|---|---|---|
| `T01` | `001` | nonneg/bounds tests | specified negative unconstrained component, preserved names/count, bounds, unity |
| `T02` | `002` | none | observable persisted solver-class field |
| `T03` | `003` | unity tests | non-X sum and exact zero X-only unity coefficient |
| `T04` | `005` | library ceiling only | renamed fluid with omitted max remains 0.5 end to end |
| `T05` | `006` | bound-water/fluid tests | independent `0.184106 ±1e-6` arithmetic fixture |
| `T06` | `006` | no exact three-point candidate | 5000/500/35000 ppm expansion and cap controls |
| `T07` | `007` | silent-zero behavior only | named CEC refusal plus valid WCLP `0.040909` control |
| `T08` | `008`, `040` | library snapshot | shipped-pair failure plus two vendor-pair and Chlorite controls; route custody |
| `T09` | `009` | generic export provenance only | enumerate every endpoint value with source and vendor-derived flag |
| `T10` | `010` | none | mixed wet/dry rows refuse naming both |
| `T11` | `011` | none | 16.0 and 0.001 both refuse with meq/g window |
| `T12` | `012` | `pef_converts_to_u_before_mixing` supporting | independent quartz/water `1.3821`, wrong-law `1.085`, separation >0.25 |
| `T13` | `013` | RECON decomposition supporting | independent long-form Eq 79/80 equality with LargestWeight ≠1 |
| `T14` | `004`, `014` | none | all three labelled misfits and unity convention |
| `T15` | `015` | none | collinear model emits `CONDNUM>10` and untrusted result |
| `T16` | `016` | two passing zero-DOF tests | both 2-DOF/no-note and zero-DOF/note cases; Tool-row accounting cross-check |
| `T17` | `017` | frontend omission supporting | active=false changes DOF; sigma=1e6 does not |
| `T18` | `018` | none | 1.0 bit-no-op, 0.25 changes residual, sigma unchanged in record |
| `T19` | `019`, `020` | hard-coded defaults only | five printed deviations stored independently from range rule |
| `T20` | `020`, `039` | none | nine rule-governed rows and derived labels; balanced pre-solve output |
| `T21` | `021` | implicit Geolog exponent only | three independent exponents and persisted convention |
| `T22` | `022` | absence search | missing constant refuses; all three sourced candidates displayed without default |
| `T23` | `023` | none | independent Dual-Water `m*=2.206503`; single-source label control |
| `T24` | `010`, `024` | hard-coded-density dry-clay snapshots | explicit rho_bw changes result from `2.831325` to `2.821084` |
| `T25` | `025` | categorical X/U rows only | one equation at IF=0.5, others 1.0, record persists factors |
| `T26` | `026` | none | selected response set changes volumes, both records name it, no header read |
| `T27` | `027`, `040` | silent route switch | 10.4 refuses; 0.104 returns `0.116071`; selected route persists |
| `T28` | `028`, `029` | one library only | two selectable libraries, >5% disagreement shown with values/sources |
| `T29` | `029` | code literal only | gas DT source and alternatives shown at editor |
| `T30` | `030` | different Simandoux helpers only | Eq 78 no-silt reduction and distinct displayed identity |
| `T31` | `031` | one zonal Rsh only | two clay-specific values consumed; one missing uses zonal fallback |
| `T32` | `032` | incomplete generic log-set replay support | persisted resolved set alone replays; preserve OPEN-9 toolchain limit |
| `T33` | `033` | generic solver errors only | PHIMAX/BVIRR conflict names both rows and depth |
| `T34` | `034` | divergent equality path | violating case corrected; already-valid case bit-unchanged |
| `T35` | `035` | soft tie only | hard tie 1e-9, residual curve, DOF and misfit contribution |
| `T36` | `042`, `043` | none | OBM inequality and opt-in ceiling behavior with recorded values |
| `T37` | `036` | partial output-name inventory | X outputs present, no bare SW, clay/shale curves declare convention |
| `T38` | `037` | separate MC module only | same seed identical, different seed differs, seed persisted |
| `T39` | `038` | none | nonnegative per-volume values, linear misfit scaling, caveat label |
| `T40` | `041` | retirement and workflow tests pass | catalogue resolution plus real-runner refusal and replacement message |
| `T41` | `041` | current mismatch evidence | every retired endpoint matches live or is labelled historical |
| `T42` | `044` | no boundary-invariance candidate | metric/imperial solve equivalence and four unit-trap controls |
| `T43` | `045` | fallback test passes | exactly 30/100 substitutions returned and persisted |
| `T44` | `046` | no triple-gate candidate | three vendor inconsistencies independently computed and surfaced |

## Execution Tasks

### Task 1: Freeze the live receipt baseline

**Files:**
- Read: `docs/takeover/requirements.csv`
- Read: `docs/takeover/STATUS.md`
- Read: `docs/VERIFICATION_MATRIX.md`
- Create later: `docs/takeover/evidence/sb-min.md`

**Interfaces:**
- Consumes: accepted anchor, branch, worktree, immutable chapter counts, current ledger summary
- Produces: a baseline receipt whose counts every later row must reconcile against

- [ ] **Step 1: Reverify Git identity and reachability**

Run:

```powershell
git branch --show-current
git status --short
git rev-parse HEAD
git rev-parse origin/master
git merge-base HEAD origin/master
git merge-base --is-ancestor b332026cb498c105f36eade0bf7899bc0c1309f0 HEAD
git worktree list --porcelain
```

Expected: branch `codex/g1-sb-min-adjudication`, clean tree, sole worktree `D:/XX. SandiBumi`, accepted anchor reachable. Stop and reconcile if not.

- [ ] **Step 2: Recompute ledger and chapter counts**

Run:

```powershell
$rows = Import-Csv docs/takeover/requirements.csv | Where-Object requirement_id -like 'SB-MIN-*'
$rows.Count
$rows | Group-Object original_priority | Sort-Object Name | Format-Table Name,Count
$rows | Group-Object chapter_status | Sort-Object Name | Format-Table Name,Count
node tools/takeover-ledger.mjs --summary-json
```

Expected: the Baseline and Count Contract exactly. Do not edit if IDs, source fields, or baseline totals differ.

- [ ] **Step 3: Start the evidence receipt with the measured baseline**

Create `docs/takeover/evidence/sb-min.md` with the branch/commit/anchor receipt, 46-row/44-test/78-parameter counts, current manual matrix, exact candidate-test commands/results, and a prominent statement that this is adjudication only—not production remediation or source adoption.

### Task 2: Adjudicate solver core and endpoint custody (`001`–`012`)

**Files:**
- Read: `src-tauri/src/multimin2.rs`
- Read: `src/ui/multiminDialog.ts`
- Read: `src/ipc.ts`
- Modify: `docs/takeover/evidence/sb-min.md`
- Modify: `docs/takeover/requirements.csv`

**Interfaces:**
- Consumes: Group A/B map, T01–T12, chapter parameter rows, ESC-2
- Produces: 12 complete receipt sections and 12 live ledger verdicts

- [ ] **Step 1: Trace request-to-solver structure**

Record exact paths for component names/kinds/zones/bounds, ToolSpec, request defaults, unity row, classifier, bounded solver, UI maps, and IPC. Confirm every negative in both Rust and TypeScript.

- [ ] **Step 2: Classify current tests by proof quality**

Run:

```powershell
cd src-tauri
cargo test multimin2::tests::nonneg_holds_when_truth_is_a_boundary
cargo test multimin2::tests::unity_is_exact_and_bounds_hold
cargo test multimin2::tests::bound_water_tracks_clay_volume
cargo test multimin2::tests::wet_clay_porosity_bound_water_tie
cargo test multimin2::tests::fluid_calc_matches_reference
cargo test multimin2::tests::library_has_expected_shape
cargo test multimin2::tests::pef_converts_to_u_before_mixing
cd ..
```

For every pass, identify whether the expected value is independent. Do not promote library snapshots or same-code forward models to correctness.

- [ ] **Step 3: Write and cross-check rows `001`–`012`**

For each row, fill every Evidence Receipt Schema field, then update only live ledger fields. Preserve `ESC-2`; name the four source-absent shipped literals where relevant; never choose a library or default.

### Task 3: Adjudicate diagnostics and weighting (`013`–`020`)

**Files:**
- Read: `src-tauri/src/multimin2.rs`
- Read: `src-tauri/src/resultsqc.rs`
- Read: `src/ui/resultsQcPanel.ts`
- Read: `src/ui/multiminDialog.ts`
- Modify: `docs/takeover/evidence/sb-min.md`
- Modify: `docs/takeover/requirements.csv`

**Interfaces:**
- Consumes: Group C/D map, T13–T20, Geolog/Elan statistic and uncertainty definitions
- Produces: eight complete receipt sections and ledger verdicts

- [ ] **Step 1: Trace every diagnostic surface**

Inventory RECON arithmetic, per-tool residual curves, DOF/note, solver failure paths, result object, UI text/thresholds, and exports. Search explicitly for `TOTERR`, `QUALITY`, `CONDNUM`, `active`, weight multiplier, uncertainty MIN/MAX/source, and trusted/untrusted flags.

- [ ] **Step 2: Run exact positive candidates**

Run:

```powershell
cd src-tauri
cargo test multimin2::tests::recon_qc_emits_per_tool_curves_and_flags_endpoint_error
cargo test multimin2::tests::dof_note_set_when_exactly_determined
cargo test multimin2::tests::an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so
cd ..
```

Classify the zero-DOF tests against T16 only; do not let them close conditioning, conflict detection, or missing statistics.

- [ ] **Step 3: Write and cross-check rows `013`–`020`**

Record absence or partial implementation explicitly. Treat hard-coded `BASE_TOOLS` values without per-value sources as custody failures, not correctness evidence.

### Task 4: Adjudicate conductivity, model inputs, and persistence (`021`–`032`)

**Files:**
- Read: `src-tauri/src/multimin2.rs`
- Read: `src/ui/multiminDialog.ts`
- Read: `src/ipc.ts`
- Read: `src-tauri/src/equations.rs`
- Read: `src-tauri/src/export.rs`
- Modify: `docs/takeover/evidence/sb-min.md`
- Modify: `docs/takeover/requirements.csv`

**Interfaces:**
- Consumes: Group E/F map, T21–T32, ESC-1/2/4/7, OPEN-2/3/5/6/7/9
- Produces: 12 complete receipt sections and ledger verdicts

- [ ] **Step 1: Trace named input and unit custody**

Inventory fluid exponent, dry-clay constants, zone/invasion behavior, neutron endpoints, WCLP validation, library selection, sonic source display, silt/Simandoux identities, per-clay Rsh, request serialization, log-set params, and export provenance.

- [ ] **Step 2: Run relevant current helpers and generic provenance support**

Run:

```powershell
cd src-tauri
cargo test multimin2::tests::dry_clay_matches_the_kkt_example
cargo test multimin2::tests::dry_clay_cec_reproduces_the_bndwat_tie
cargo test multimin2::tests::dry_clay_rejects_degenerate_densities
cargo test multimin2::tests::dry_clay_rejects_unphysical_picks
cargo test export::tests::every_las_export_carries_measured_computed_and_model_provenance_in_the_file
cd ..
```

Generic export provenance remains supporting only until the full resolved SandiMin parameter set exists.

- [ ] **Step 3: Write and cross-check rows `021`–`032`**

Keep every unresolved convention/default absent. Do not interpret hard-coded density, endpoint, exponent, or run JSON as a sourced resolved parameter set.

### Task 5: Adjudicate constraints, outputs, uncertainty, retirement, and guards (`033`–`046`)

**Files:**
- Read: `src-tauri/src/multimin2.rs`
- Read: `src-tauri/src/multimin.rs`
- Read: `src-tauri/src/modules.rs`
- Read: `src-tauri/src/workflow.rs`
- Read: `src-tauri/src/resultsqc.rs`
- Read: `src/ui/multiminDialog.ts`
- Read: `src/ui/resultsQcPanel.ts`
- Read: `src/ipc.ts`
- Modify: `docs/takeover/evidence/sb-min.md`
- Modify: `docs/takeover/requirements.csv`

**Interfaces:**
- Consumes: Group G/H/I map, T33–T44, ESC-3/6/8, OPEN-9
- Produces: 14 complete receipt sections and ledger verdicts

- [ ] **Step 1: Trace constraints and outputs end to end**

Inventory feasibility errors, WBM re-solve, soft rows, output names/metadata, missing uncertainty paths, retired spec/catalog/runner, unit boundaries, FTEMP fallback/reporting, and clay triple validation.

- [ ] **Step 2: Run exact constraint, temperature, retirement, and workflow candidates**

Run:

```powershell
cd src-tauri
cargo test multimin2::tests::request_defaults_keep_every_constraint_on
cargo test multimin2::tests::ftemp_curve_constant_equals_fixed_temperature
cargo test multimin2::tests::ftemp_curve_overrides_scalar_temperature
cargo test multimin2::tests::ftemp_curve_out_of_range_falls_back
cargo test multimin2::tests::ftemp_curve_recon_qc_decomposition_holds
cargo test modules::tests::multimin_is_retired_but_still_cataloged
cargo test workflow::tests::phase7_generic_store_feeds_modules_and_mask
cd ..
```

Credit T40 narrowly. Record T41's orphan defaults as a current divergence. Credit FTEMP fallback narrowly and leave the substitution-count contract open.

- [ ] **Step 3: Write and cross-check rows `033`–`046`**

Keep ESC-3 cap/policy and ESC-6 warn/refuse adoption unresolved. Keep `VP/VS=1.7` source-absent and outside any claim of validated endpoint construction.

### Task 6: Complete parameter, escalation, history, and manual-evidence receipts

**Files:**
- Read: `docs/PRD_v2/13_mineral-solver.md`
- Read: `docs/VERIFICATION_MATRIX.md`
- Read: relevant reachable Git commits
- Modify: `docs/takeover/evidence/sb-min.md`

**Interfaces:**
- Consumes: all 46 row sections and immutable chapter inventories
- Produces: domain-level proof that no source, escalation, test, history seam, or manual scenario was silently skipped

- [ ] **Step 1: Reconcile all 78 parameters**

Add a parameter-custody appendix listing each row's disposition. Explicitly enumerate the ten absent rows, five non-adoptable entries, four currently shipping source-absent values, and all vendor-derived rows. Confirm no receipt text promotes one.

- [ ] **Step 2: Reconcile every escalation/open item**

Map `ESC-1`–`ESC-8`, acquisition gaps `ACQ-1`–`ACQ-10`, and `OPEN-1`–`OPEN-10` to affected requirements and next actions. Record `OPEN-8`'s stale counts, `OPEN-9`'s replay ambiguity, and `OPEN-10`'s deliberately absent surface.

- [ ] **Step 3: Record reachable history without treating it as current proof**

At minimum, inspect and record reachability for `73f952d`, `8fc873b`, `b375d7e`, `a3cd716`, `a5739c2`, `627e859`, `1e8a837`, `2b4a30c`, `b0a1bb8`, `857581f`, and `254714e`. History explains intent; live source/tests decide current state.

- [ ] **Step 4: Record the manual-evidence boundary**

Copy only checked counts from `docs/VERIFICATION_MATRIX.md`. State `NONE` for each MIN requirement without a checked scenario. Do not infer SandiMin exercise from generic workflow, report, export, or automated tests.

### Task 7: Validate ledger completeness and update program status

**Files:**
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read: `docs/takeover/evidence/sb-min.md`

**Interfaces:**
- Consumes: 46 complete evidence sections and live verdicts
- Produces: mechanically valid Gate 1 ledger/status with SB-MIN summarized exactly

- [ ] **Step 1: Validate row coverage and allowed fields**

Run a PowerShell comparison that confirms IDs `001..046` appear exactly once in both CSV and receipt, source-owned columns are byte-identical to `HEAD^` where applicable, and no live row remains `UNADJUDICATED`.

- [ ] **Step 2: Run the ledger validators**

Run:

```powershell
node --test tools/takeover-ledger.test.mjs
node tools/takeover-ledger.mjs --summary-json
```

Expected: all tracker tests pass; summary reports `438` adjudicated and `493` unadjudicated. Use measured, not predicted, MIN as-built/disposition/test counts in STATUS.

- [ ] **Step 3: Update `docs/takeover/STATUS.md`**

Replace the active increment, automated-gate receipt, open blockers, next increment, dashboard counts, ledger roll-up, and recent-increment table with measured SB-MIN results. Keep POR/RHG remediation banked for Gate 2 and keep SB-GEO deferred.

### Task 8: Self-review the execution against the immutable spec

**Files:**
- Read: all three execution files
- Read: `docs/PRD_v2/13_mineral-solver.md`

**Interfaces:**
- Consumes: completed receipt, ledger, status
- Produces: corrected adjudication with no skipped requirement/test/parameter or unsupported claim

- [ ] **Step 1: Requirement coverage review**

Check all 46 requirements in §4 against receipt sections and ledger rows. For every compound contract, verify the stricter status fails for an explicitly named limb rather than vague absence.

- [ ] **Step 2: Test-intention coverage review**

Check all 44 tests in §6 against the Acceptance-Test Ownership Map. Confirm each appears exactly once as primary, each expected value names its source, and self-generated fixtures are labelled characterization/supporting rather than correctness.

- [ ] **Step 3: Parameter and escalation review**

Recount 78 parameters, ten absent, five non-adoptable, eight escalations, ten acquisition gaps, and ten open items. Search the execution diff for an adopted value or resolved decision not authorized by the chapter/Jauhar.

- [ ] **Step 4: Placeholder and confidentiality scan**

Run:

```powershell
rg -n 'TBD|TODO|fill in|implement later|appropriate error handling|similar to Task|client|field|block|basin|operator|well name|project name' docs/takeover/evidence/sb-min.md docs/takeover/requirements.csv docs/takeover/STATUS.md
```

Review every match in context. Generic words in established schema/headings are allowed; identifying names, placeholders, or unspecific next actions are not.

### Task 9: Verify and commit the adjudication increment

**Files:**
- Commit only: `docs/takeover/evidence/sb-min.md`, `docs/takeover/requirements.csv`, `docs/takeover/STATUS.md`

**Interfaces:**
- Consumes: self-reviewed execution files
- Produces: one local, unpushed Gate 1 SB-MIN adjudication commit with a green full gate

- [ ] **Step 1: Run TypeScript and Rust compile checks**

Run from repository root:

```powershell
npx tsc --noEmit
cd src-tauri
cargo check
cd ..
```

Expected: both exit 0.

- [ ] **Step 2: Run the complete full gate**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: green with zero failures. Record exact passed/failed/ignored totals; do not reuse the prior 946/0/36 receipt.

- [ ] **Step 3: Inspect and stage only the three execution files**

Run:

```powershell
git status --short
git diff --check
git diff -- docs/takeover/evidence/sb-min.md docs/takeover/requirements.csv docs/takeover/STATUS.md
git add docs/takeover/evidence/sb-min.md docs/takeover/requirements.csv docs/takeover/STATUS.md
git diff --cached --check
git diff --cached --name-only
```

Expected cached paths: exactly the three listed files.

- [ ] **Step 4: Commit locally**

Run:

```powershell
git commit -m "G1-DOM-MIN adjudicate 46 SB-MIN requirements"
```

Do not push, merge, or open a pull request.

- [ ] **Step 5: Continue Gate 1 serially**

Recompute the domain inventory from the remaining 493 unadjudicated rows, exclude the 52 deferred SB-GEO rows from the current version, select the next dependency-relevant open-hole-petrophysics domain, create/rename the serial topic branch without changing worktree, and begin its planning increment. Do not start POR/RHG production remediation before Gate 1 evidence reconciliation is complete.

## Plan Self-Review Receipt

- **Spec coverage:** all 46 requirements appear exactly once in the Requirement Evidence Map; all 44 tests appear exactly once in the Acceptance-Test Ownership Map; all 78 parameters and all eight escalations/ten acquisition gaps/ten open items have explicit execution tasks.
- **No placeholders:** this plan contains no `TBD`, `TODO`, `implement later`, unspecific error-handling instruction, or cross-task shorthand. Every command, file, expected count, and adjudication boundary is concrete.
- **Type/interface consistency:** the plan uses the live names `Component`, `ToolSpec`, `FluidProps`, `MultiminRequest`, `MultiminResult`, `PorositySource`, `SwModel`, log-set `params_json`/`inputs_json`, and the exact current source/UI paths. It does not invent a production type or implementation API.
- **Source discipline:** no unresolved parameter or vendor choice is adopted. ESC-1 through ESC-8, the ten absent parameter rows, five non-adoptable entries, and four source-absent shipping literals remain fenced.
- **Evidence discipline:** passing tests are candidates only until their expected values and observable scope are independently checked. Manual `0/28` SandiMin evidence remains open.
- **Scope discipline:** planning changes only this plan and STATUS; execution changes only the MIN receipt, ledger, and STATUS. Production remediation remains a later Gate 2 activity.

## Execution Handoff

Execute inline in the current session with `superpowers:executing-plans`. The user's persistent instruction is to continue Gate 1 serially; no additional approval is required unless execution reaches an unresolved product-owner decision that would otherwise be silently guessed.
