# SB-SAT Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 51 live `SB-SAT` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing saturation equations, selecting a petrophysical parameter, resolving an open source conflict, changing the PRD, or implementing a missing capability.

**Architecture:** This is a documentation-only evidence pass across both independent saturation engines and every surface that can make their answers observable. The immutable PRD supplies the intended contract, original priority, chapter status, parameter custody, and owned-test intentions. Current source, qualifying observable tests, manual evidence, and reachable Git history supply a separate live verdict in `docs/takeover/requirements.csv` and one SB-SAT receipt. The pass follows method identity, typed units, Archie/Simandoux/Indonesia/Juhasz/Waxman-Smits/dual-water behavior, conversions and root finding, no-default parameters, LRLC calibration custody, flushed-zone behavior, run provenance, export, guidance, and downstream curve selection. A correct helper or one correct engine is supporting evidence only until the complete cross-engine and reporting-surface contract is proved.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the existing takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Petrophysical math, parameter custody, data-integrity judgments, and final sign-off remain with the primary session.
- Work only in `D:\XX. SandiBumi`. The sole registered Git worktree MUST remain that path. `D:\XX. SandiBumi-check` is not a repository or evidence source.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. This plan is written on `7baa228ca1b48839fa7767d873d5e45293d61032`; fetched `origin/master` and the current merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`. Reverify every reference before execution. If any moves, stop and reconcile before classifying.
- The local branch is `codex/g1-sb-sat-adjudication`. It was renamed from the misleading GEO name without changing `HEAD`; `origin/master` was already an ancestor, so no merge, rebase, cherry-pick, or history rewrite occurred. The existing 31-commit local takeover chain remains intentionally unpushed under the serial Gate 1 protocol.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, exact tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete saturation chapter, applicable `docs/record_calibration.md`, `docs/record_fixes.md`, and `docs/record_data_tools.md`, the relevant `_SPINE_PENDING.md` entries, and the current takeover status/evidence.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. All 51 chapter statuses and all 51 owned-test mappings are populated; they are source evidence, not fields for this lane to reinterpret or rewrite.
- The chapter and live ledger agree on 51 contiguous requirements: `P0=13`, `P1=18`, `P2=12`, `P3=6`, `P4=2`. Chapter states are `PRESENT-OK=6`, `PRESENT-UNVERIFIED=1`, `PRESENT-DIVERGENT=16`, `PARTIAL=9`, and `ABSENT=19`. Reverify live behavior independently; do not copy these counts into the live verdict.
- The chapter defines 63 contiguous named tests, `SB-SAT-T01` through `SB-SAT-T63`. Route each intention exactly once as a primary proof target and list cross-support separately. Never count a test merely because its ID appears in an owned-test cell.
- Section 5 mechanically contains exactly 71 parameter rows, 20 rows whose value contains `ABSENT`, and 8 tierless rows. Inspect all 71. Every absent parameter remains absent; every tierless shipping literal remains uncited, withdrawn, explicitly calibrated, or blocked exactly as the chapter states.
- Section 7.1 contains ten escalations, `ESC-1` through `ESC-10`, while section 8.16 says the escalated mentions resolve to nine. Record that immutable-PRD structural mismatch in the receipt; do not edit the PRD or silently normalize the count.
- `ESC-1` disputed authorship, `ESC-2` the variable-`m` coefficient, `ESC-3` the Juhasz exponent form, `ESC-4` the `vQ0` candidates, `ESC-5` the Nigeria exponent, `ESC-6` brine density, `ESC-7` the fourth digit of `F`, `ESC-8` unknown incumbent solver behavior, `ESC-9` ledger sign-off, and `ESC-10` patent/design-around review remain distinct. None authorizes a guessed value.
- `SP-003` remains open for the Tier-C sonic-saturation route because the chapter records no owning requirement or test for it. Do not invent one in this lane.
- Every petrophysical parameter is CITED or ABSENT. A source-code literal, neighboring vendor value, remembered textbook value, result from another method, current fixture, average, range midpoint, rounded number, or plausible interpolation is not a citation.
- `as_built_status` answers what the accepted tree currently ships. `release_disposition` answers whether the contract is required for the Windows-first paid open-hole-petrophysics pilot. Priority and chapter status inform the decision but do not decide it automatically.
- A compound requirement is not `PRESENT-OK` because one limb, helper, engine, or output is correct. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, a list, a cross-engine phrase, or an export/provenance clause.
- A test is qualifying owned acceptance proof only when it exercises the observable contract and derives its expected value independently from a cited source or explicit arithmetic. Internal `Result`, source-text grep, compile success, helper round-trip through the same implementation, or self-generated synthetic truth is supporting evidence only.
- Classify tests under `CONTRACT.md` sections 3 and 6 as `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`. A test pinning current divergent behavior is divergence evidence, never proof of the specified behavior.
- The two saturation engines share no single implementation: deterministic modules live primarily in `modules.rs`/`lrlc.rs`, while solver and Results QC paths reuse `multimin2.rs`. Inventory both and compare public method identity, parameters, units, guards, back-out, outputs, flags, and provenance independently.
- Current `sw_sim` and solver `SwModel::Simandoux` use different forms under overlapping naming. Do not collapse Bardon-Pied and modified-Schlumberger variants into one method or accept matching UI labels without equation-level proof.
- Current `sw_indonesia` exposes three variants while the solver hard-codes one. Cross-engine parity requires the same named model with the same parameterization, not merely an Indonesia result in each engine.
- Current Waxman `B`, `Qv`, CEC, and temperature values are bare numeric types. Documentation comments are supporting evidence, not unit enforcement. Test both accepted canonical units and rejected wrong-scale/wrong-temperature controls.
- Current dual-water `Swb*(Cwb-Cw)` algebra is a positive seam, but alpha activity, temperature-branch `vQ`, beta dilution, validity flags, and effective back-out are separate obligations. Never let one correct coefficient close the whole chain.
- Current Juhasz arithmetic can return an answer when the excess-conductivity coefficient is negative. A numerically stable result is not a valid result unless the required observable flag/refusal exists.
- Current deterministic guards and `SWE_IRR` transform have positive evidence; current `sw_imts` retains the final iterate on non-convergence. Adjudicate solver guard, failure reporting, and output persistence separately.
- Current Rw correlations contain strong equation-level evidence, but the shipped defaults, branch provenance, exact boundary tests, source comments, and UI/run custody are separate requirements.
- Current LRLC coefficients are explicitly one calibration/placeholder route with tested fit paths. Preserve `docs/record_calibration.md`: a wet interval must be declared, the fit is the algebraic inverse of the exact module, held-fixed values travel with the fit, Apply is explicit and atomic, and calibration never silently auto-adjusts interpretation.
- Current generic run/export provenance carries module, parameter JSON, input JSON, and ancestry for computed curves. That is supporting evidence for `SB-SAT-043`; it is not complete until parameter source strings, papers, model identity, calibration state, flags, and export survival are proved for saturation results.
- Current Results QC displays method spread and deliberately does not fabricate Qv or Swb. Guidance and diagnostic comparison are not automatic model selection, but existing labels/formulas must still be checked against the stable equation identities.
- Current downstream aliases and selectors still admit bare `SW` in some generic surfaces. Absence of a newly emitted bare curve is not enough; trace registration, saturation-height selection, plotting, pay, export, and run history.
- No production equation, default, validation range, clamp, mnemonic, method label, unit, refusal, output, or provenance behavior changes during adjudication. Every repair remains a later one-requirement increment with its own sourced expected value and named test.
- The manual baseline remains separate: `saturation=2/97`, `saturation-height=0/6`, `workflow=0/23`, `crossplot=6/13`, `pickett=0/8`, `las-export=0/2`, `processing-history=0/7`, and `verification-stewardship=0/24`. Automated or desktop-harness evidence does not close an unchecked manual scenario.
- New receipt/ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record all of the following:

1. branch `codex/g1-sb-sat-adjudication`;
2. a clean worktree and the sole registered worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, fetched `origin/master`, merge base, and accepted-anchor reachability;
4. exactly 51 ledger rows, covering `SB-SAT-001` through `SB-SAT-051` once with no gap or duplicate;
5. priority counts `P0=13`, `P1=18`, `P2=12`, `P3=6`, `P4=2`;
6. source-status counts `PRESENT-OK=6`, `PRESENT-UNVERIFIED=1`, `PRESENT-DIVERGENT=16`, `PARTIAL=9`, `ABSENT=19`;
7. all 51 source-owned `owned_tests` populated and all 51 live `as_built_status=UNADJUDICATED`;
8. exactly 63 chapter test IDs, `T01` through `T63`, each once;
9. exactly 71 parameter rows, 20 ABSENT-bearing rows, and 8 tierless rows;
10. section 7.1 has ten escalations while section 8.16 claims nine;
11. takeover summary `341` adjudicated, `590` unadjudicated, and `251` pilot blockers before any SAT edit; and
12. the current manual capability counts listed in Global Constraints.

The only mechanically predictable post-adjudication ledger count is `392` adjudicated and `539` unadjudicated. Do not predict live as-built, pilot-blocker, test-class, or manual-evidence totals before classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-sat.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: current Rust, TypeScript, source, tests, history, manual-evidence, and export/report surfaces required below
- Never modify: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts

## Evidence Receipt Schema

Create one `### SB-SAT-NNN` section per requirement in numeric order. Every section MUST include:

- **Specified contract:** every observable limb, separated when compound.
- **Current implementation:** exact symbols and paths, plus explicit negative-inventory scope where absent.
- **As-built status:** one legal ledger state and why a stricter state fails.
- **Release disposition and risk:** pilot relevance independent of implementation status.
- **Automated evidence:** exact test names and commands classified `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`, with the independent expected-value source named.
- **Manual evidence:** exact checked capability/scenario or `NONE`; never inferred from automation.
- **Source/parameter boundary:** cited, absent, withdrawn, calibration-only, placeholder, conflicting, or not applicable, with escalation IDs.
- **Cross-engine/downstream surface:** deterministic, solver, Results QC, workflow, selection, plot, provenance, report, and export impact as applicable.
- **History/reachability:** accepted commit evidence or confirmed negative history search where consequential.
- **Blocking decision/dependency:** exact source, legal, product, UI, manual, or implementation dependency.
- **Next action:** the smallest bounded follow-up, or `NONE` only when the whole contract is proved and no field evidence is required.

## Requirement Evidence Map

### Group A - Model identity and equations (`001`-`011`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Stable equation identities; no bare vendor adjective as the model name. | `modules.rs` saturation manifests; `multimin2.rs::SwModel`; `resultsqc.rs`; `multiminDialog.ts`; saved-run labels. | A familiar label is not identity; match equation, options, persisted ID, UI, output, and export. |
| `002` | Effective and total Archie are separate named methods. | `sw_arch`; `sw_archie`; PHIE/PHIT use; module/solver outputs and selectors. | Two output curves from one total-porosity method do not prove two methods. |
| `003` | Vendor alias table resolves only mapped imports and refuses unknown names. | model registries/import/config readers, option IDs, saved workflows, UI labels, tests/history. | Generic case folding or substring matching is not a governed alias table. |
| `004` | Two Simandoux variants; `C` belongs only to modified-Schlumberger and validates `1:2`. | `sw_sim_spec`; `sw_sim`; `calc_sw`; `sw_simandoux`; Results QC and solver labels. | Separate equation identity, `C` scope, default, and range; current `0.5:2` is not source authority. |
| `005` | Simandoux `a` has no default. | `sw_sim_spec`, generic parameter resolution, solver fluid/default structures, dialog serialization. | A required field with fallback `1` or `0.8` still violates no-default custody. |
| `006` | Indonesia exposes parameterized `k=0/1/2` in deterministic and solver paths. | `sw_indo_spec`; `sw_indo`; `sw_indonesia`; `SwModel::Indonesia`; Results QC. | Three options in one engine plus one hard-coded form in another is partial, not parity. |
| `007` | Woodhouse Tar is a cited alias of Indonesia `k=2`. | `OPT_INDO=TAR_SAND`; source topics; registry/UI/run records. | Equation equivalence without alias, citation, and recorded selection is incomplete. |
| `008` | Total Shale is a preset of modified-Schlumberger with `C=1`, `n=2` fixed. | module/solver catalogs, Simandoux helpers, option persistence, root finder. | A pure-shale test or ordinary Simandoux run is not the named preset. |
| `009` | Juhasz uses shale-derived coefficient, shale normalization, and its own `m*`. | `sw_juhasz`; solver post-processing; `FluidProps`; Results QC; CEC/Qv routes. | Check coefficient, normalization, exponent, `a`, back-out, and both engine availability separately. |
| `010` | Negative Juhasz excess-conductivity coefficient is flagged. | `sw_juhasz`; `sw_cond_root`; solver result/flag paths; Results QC/UI. | Returning a finite or NaN value without the named observable flag fails the contract. |
| `011` | Waxman-Smits exposes `a`. | `sw_waxman_smits`; `SwModel`; solver configuration; Results QC parameters/UI. | An internal fixed value or unrelated Archie `a` does not satisfy model-specific custody. |

### Group B - Typed conductivity and dual water (`012`-`022`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `012` | `B` is unit-typed in canonical `L*S/(eq*m)` with the exact x100 converter. | `waxman_b`; `juhasz_b`; `sw_waxman_smits`; `lrlc.rs`; manifests and IPC. | A doc comment beside `f64` is not a type; pin canonical and wrong-scale controls. |
| `013` | `Qv` and CEC inputs are unit-typed in their declared canonical units. | solver mineral CEC, `qv_at`, LRLC manifests, Results QC request, curve metadata. | Two correct formulas using meq/g and meq/100g without types still diverge at the seam. |
| `014` | `B(T,Rw)` consumes typed degC, clamps `B>=0`, allows override, and records source. | `waxman_b`; `juhasz_b`; `fluid_calc`; temperature inputs; run/export provenance. | Existing anchors prove arithmetic only; type, override, source, and run record remain separate. |
| `015` | `B` method has no default and exposes four named formula options plus user-defined. | saturation manifests, solver config, source topics, dialog/persistence, history. | One hard-coded closed form is not a selectable no-default method family. |
| `016` | Dual water ships named CEC and simple forms; foreign `dual` resolves to Juhasz. | `DualWaterNonlinear`; `fluid_calc`; model registry/alias paths; Results QC. | One solver variant does not prove two forms or the import mapping. |
| `017` | Excess coefficient is exactly `Swb*(Cwb-Cw)`. | `sw_dual_nonlinear`; `sw_cond_root`; hand-computed tests. | This positive helper is one limb only and cannot close the broader dual-water chain. |
| `018` | `vQ` switches temperature expression when the diffuse layer expands. | `alpha_expansion`; `bndwat_multiplier`; `fluid_calc_at`; solver post-processing. | Multiplying the saline expression by alpha is not the specified expanded-temperature branch. |
| `019` | Alpha includes the Debye-Huckel activity ratio. | `alpha_expansion`; salinity conversion; fluid tests. | The square-root molarity approximation and an arbitrary ceiling do not satisfy the equation. |
| `020` | Beta carries the salinity-dilution factor. | bound-water conductivity chain in `fluid_calc_at`; parameter registry; tests/history. | A collapsed temperature coefficient without the dilution term is absence, not approximation. |
| `021` | `Qv>1/vQ` flags and `Swb<=1-PHIE/PHIT` clamps independently. | solver Qv/Swb construction; `sw_dual_nonlinear`; result flags/provenance. | Structural equality in one solver is not an explicit bound diagnostic or standalone protection. |
| `022` | `vQ0` ships absent until the cited candidates are adjudicated. | fluid/default structures, dialog serialization, source registry, `ESC-4`. | Do not choose 0.30, 0.28, or a current derived coefficient. |

### Group C - Back-out, outputs, and solver behavior (`023`-`030`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `023` | Effective back-out is model-specific, with named inverses, never blanket. | solver post-solve redistribution; deterministic SWE/SWT; Juhasz and dual-water back-out. | A shared `(SWT*PHIT-Vbw)/PHIE` path fails methods whose bound-water definition differs. |
| `024` | `SWE_IRR` is transformed as an effective quantity per model. | deterministic `SWT_IRR` handling; workflow overrides; method outputs. | Protect the specified transform and reject the broken blanket volume ratio. |
| `025` | Every method emits clipped and unclipped forms of the same quantity. | deterministic output manifests/functions; solver curve list; LRLC; export/catalog. | A clipped SWE paired with an unclipped SWT is not a same-quantity diagnostic pair. |
| `026` | No bare `SW`; every method emits a method-flag curve and water volumes. | module outputs; solver outputs; registration; downstream selectors; export. | Check positive suffixed outputs and negative bare-SW reachability; flags and volumes are separate. |
| `027` | One shared root finder carries the specified seed, budget, tolerance, and guards. | `calc_sw`; `sw_cond_root`; `sw_imts`; solver/model helpers. | Two individually stable solvers do not satisfy one shared guarded implementation. |
| `028` | Non-convergence returns missing/null, never a partial iterate. | `calc_sw`; `sw_imts`; solver post-processing; output persistence and UI status. | A max-iteration loop without explicit convergence state fails even when sample cases converge. |
| `029` | Low-porosity, total-zero, non-positive-Rt, variable-m, and coal guards preserve the specified volume detail. | deterministic modules; solver guards; LRLC; workflow output tests. | Check both saturation and `VOL_UWAT/VOL_XWAT`; value-only proof is incomplete. |
| `030` | `Vsh` approaching one flags before the singularity. | modified-Schlumberger branch; solver form; Results QC; method flags. | Silently returning all water is numerically bounded but observably wrong. |

### Group D - Parameter custody and correlations (`031`-`038`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `031` | `Rw` ships with no default. | `rw_args`; LRLC manifests; `FluidProps`; Results QC request/UI; saved parameters. | Current 0.1/0.3 literals remain withdrawn, not adopted precedents. |
| `032` | Measured, Kennedy, and Bateman-Konen Rw branches bind the correct temperature conversion. | `resolve_rw`; formation-temperature inputs; workflow override tests. | Test branch selection, boundary, and distinct degC/degF conversions independently. |
| `033` | Kennedy floor is exactly 0.0412 and the contradictory vendor prose cannot be used as a fix. | `resolve_rw`; source comment; boundary tests/history. | The code literal alone is supporting evidence; cite the transcribed algorithm and anti-fix rationale. |
| `034` | `a`, `m`, `n`, `m*`, and `n*` ship without defaults. | all saturation manifests, solver fluid/model structs, UI serialization, workflow resolution. | A generic UI placeholder or serde default is still a shipped default. |
| `035` | `Rsh` and shale total porosity ship without defaults; withdrawn values stay withdrawn. | `default_rsh`; `default_phit_sh`; `multiminDialog.ts`; Results QC parameters. | Do not preserve 4.0/0.10 merely because current tests depend on them. |
| `036` | Named core-derived and Qv-derived `m*`/`n*` routes exist, with core preferred. | model/preprocessing inventory; calibration/CEC paths; source registry/history. | An editable exponent or a single Qv formula is not either named derivation route. |
| `037` | Variable-`m` is a parameterized route whose coefficient has no default. | saturation manifests/helpers; solver config; `ESC-2`; tests/history. | Do not choose 0.018, 0.019, or 0.0; absence is the required parameter state. |
| `038` | Every parameter carries source string/tier and build fails without one. | `ArgSpec.sources_topic`; topic registry; dialogs; manifests; run persistence/build tests. | A generic optional source topic cannot prove universal compile-time enforcement. |

### Group E - Cross-module seams, provenance, and capability gaps (`039`-`051`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `039` | `MUDBASE` is available only on the documented models. | manifests/options, saved workflows, Sxo paths, history. | A global mud-base option or total absence both fail the scoped positive/negative contract. |
| `040` | Clay-bound-water `F` supports both unit forms; brine density stays open; `Swb=1-F` is opt-in. | bound-water/CEC utilities, solver inputs, source topics, `ESC-6/7`. | Do not assume density 1.0 or auto-promote an interpretive conversion. |
| `041` | Poupon-Aguilera and Poupon-Tixier exist with the laminated interlock. | saturation method registry, SSC/lamination paths, tests/history. | Generic thin-bed capability or method prose does not prove either saturation method. |
| `042` | SSM bound-water cap fires, resets total porosity, and emits a flag. | POR/CLY/solver bound-water paths; output flags; provenance. | The cap belongs to POR ownership for firing; a cited constant in the chapter is not implementation. |
| `043` | Every saturation result records parameters, source strings, papers, method, inputs, calibration state, and flags through export. | `log_sets`; workflow; Inspector; `export.rs::provenance_lines`; report methodology. | Generic JSON ancestry is partial until the scientific source and method contract survive. |
| `044` | Cross-tool equation/parameter disagreements are shown to the interpreter. | Results QC; source panel; dialogs; report/provenance. | A hidden note, chapter paragraph, or method-spread number without cause/custody is insufficient. |
| `045` | Selection guidance is visible but never switches the method automatically. | Results QC guidance, module UI, workflow defaults/selection, history. | Prove both guidance presence and absence of an automatic selector. |
| `046` | Flushed-zone Sxo methods, mud-base limits, and derived volumes are complete. | solver `SXOT`; standalone modules; workflow/export/downstream selection. | One solver curve without standalone methods, limits, flags, or volumes is only partial. |
| `047` | The same named model returns the same number in every engine. | deterministic modules, solver helpers, Results QC, shared reference cases. | Compare exact parameterization and quantity; matching after different clamps/back-outs is not parity. |
| `048` | LRLC coefficients are visibly calibration/placeholder values and flagged until fitted. | `sw_rtc_spec`; `sw_imts_spec`; fit dialogs/results; Apply/provenance/export. | Source comments and fit availability are insufficient if an unfitted run is not observably flagged. |
| `049` | Worthington type is carried per model in machine-readable metadata. | module/model registry, manifests, source/provenance records, export. | A chapter table or UI prose is not runtime metadata. |
| `050` | Apparent-Rw inversion exists per saturation model. | module/helper/Tauri/TS inventory; Pickett/Results QC; tests/history. | One Archie Rwa or algebra sketched in prose is not per-model inversion. |
| `051` | Mineral-conductivity/pyrite limitation is recorded on affected results. | solver mineral model; source/guidance/provenance/report/export. | A known gap in the PRD does not satisfy run-level disclosure. |

## Acceptance-Test Intention Routing

Every chapter test has one primary requirement below. Parenthetical IDs are mandatory cross-support, not duplicate primary ownership.

| Test | Primary requirement | Contract pinned |
|---|---|---|
| `T01` | `003` | All catalogued external names resolve through the alias table (`001`). |
| `T02` | `001` | No user-facing method is named only by a vendor adjective. |
| `T03` | `002` | Effective and total Archie differ on the sourced reference case. |
| `T04` | `002` | Archie matches its independently evaluated equation. |
| `T05` | `003` | An unknown external method name is refused rather than guessed. |
| `T06` | `004` | Bardon-Pied and modified-Schlumberger forms remain distinct. |
| `T07` | `004` | `C` changes only the modified-Schlumberger branch and validates `1:2`. |
| `T08` | `005` | Simandoux `a=0.8` and `a=1.0` remain 4.6 saturation units apart; no default is inferred. |
| `T09` | `006` | Indonesia `k=0/1/2` reproduces all three cited forms. |
| `T10` | `007` | Woodhouse Tar is the recorded `k=2` alias (`006`). |
| `T11` | `008` | Total Shale preset fixes `C=1` and `n=2`. |
| `T12` | `008` | The `n=2` closed form equals the general guarded solver (`027`). |
| `T13` | `009` | Juhasz coefficient uses the shale point and selected `m*`. |
| `T14` | `009` | The `Bn=1` placeholder differs by the sourced 14 saturation units. |
| `T15` | `010` | Negative Juhasz excess conductivity is flagged. |
| `T16` | `011` | Waxman-Smits responds to its exposed `a`. |
| `T17` | `012` | The wrong `B` unit scale is unrepresentable and its 27.2-su consequence is pinned. |
| `T18` | `012` | The exact x100 `B` converter round-trips. |
| `T19` | `013` | Qv canonical units agree across declared routes. |
| `T20` | `013` | A 1000x Qv unit error is rejected. |
| `T21` | `014` | Juhasz `B` matches both cited anchors and typed degF is rejected. |
| `T22` | `014` | Typed degC input and non-negative clamp survive edge cases. |
| `T23` | `015` | Four named B formulas plus user-defined are explicit and no default is elected. |
| `T24` | `016` | CEC and simple dual-water forms differ generally and agree only at the sourced fallback. |
| `T25` | `016` | External `dual` resolves to Juhasz rather than guessing (`003`). |
| `T26` | `017` | Dual-water excess coefficient is `Swb*(Cwb-Cw)` and retains the sourced alpha dependence. |
| `T27` | `018` | Expanded and saline `vQ` temperature branches diverge as sourced. |
| `T28` | `019` | Alpha includes the activity ratio at the cited salinities. |
| `T29` | `020` | Beta dilution changes the fresh-water branch by the sourced factor. |
| `T30` | `047` | Same named method has cross-engine numeric parity (`001`,`006`,`009`). |
| `T31` | `038` | Parameter registry refuses missing source/no-default metadata (`005`,`015`,`022`,`031`,`034`,`035`,`037`). |
| `T32` | `021` | Qv validity flag and Swb clamp are distinct and observable. |
| `T33` | `022` | `vQ0` has no default and both conflicting cited candidates are offered. |
| `T34` | `023` | Porosity-split effective back-out is algebraically equivalent and handles its degeneracy. |
| `T35` | `023` | Every model declares its citation and its own `Swb` rule (`007`). |
| `T36` | `023` | Dual-water back-out follows its model-specific inverse. |
| `T37` | `024` | `SWE_IRR` uses the sourced effective transform. |
| `T38` | `025` | Every method emits clipped and unclipped versions of one quantity. |
| `T39` | `026` | No method emits bare `SW`/`SXO`; suffixed identities and both water volumes accompany it. |
| `T40` | `026` | Every saturation run emits its method-flag curve through the alias registry. |
| `T41` | `028` | A root-finder cap returns missing, not the last iterate (`027`). |
| `T42` | `029` | Low porosity returns all water with porosity-sized volumes, while total-zero returns zero volumes. |
| `T43` | `029` | Missing/non-positive resistivity and missing variable-`m` input null every saturation and volume output. |
| `T44` | `030` | Near-pure-shale singularity is flagged before evaluation. |
| `T45` | `031` | Rw has no default and the measured branch uses the sourced degC Arps form (`032`). |
| `T46` | `032` | Kennedy/Bateman-Konen switch and continuity are sourced. |
| `T47` | `033` | Kennedy floor stays 0.0412 and rejects the factor-ten anti-fix. |
| `T48` | `032` | DegC and degF Arps conversions remain bound to the correct branches. |
| `T49` | `034` | All five Archie-family exponents/factors refuse absent values. |
| `T50` | `035` | Withdrawn shale values are absent and shale porosity follows the sourced accept/warn/reject ranges. |
| `T51` | `036` | Core and Qv exponent routes are named and core-preferred. |
| `T52` | `037` | Variable-`m` coefficient remains explicit and absent by default. |
| `T53` | `039` | MUDBASE is accepted only by the documented method subset and drives its OBM branch. |
| `T54` | `040` | Both F unit forms agree under an explicit density bridge. |
| `T55` | `040` | `Swb=1-F` requires explicit opt-in and absent brine density stays absent. |
| `T56` | `041` | Poupon-Aguilera and Poupon-Tixier remain distinct and cited. |
| `T57` | `041` | Poupon-Tixier refuses violation of the laminated interlock. |
| `T58` | `042` | SSM cap resets PHIT and raises its flag. |
| `T59` | `043` | Saturation provenance survives export (`048`,`049`). |
| `T60` | `044` | Interpreter sees disagreements and recorded gaps without auto-selection (`045`,`051`). |
| `T61` | `046` | Sxo behavior and cross-engine parity obey mud-base limits (`047`). |
| `T62` | `048` | A declared calibration replaces the shipped LRLC coefficients and travels with the run. |
| `T63` | `050` | Each implemented saturation method has its own independently checked Rwa inversion. |

Before adjudication, mechanically confirm that this table contains all 63 IDs once, that every one resolves to a chapter intention, and that requirements `049` and `051` retain their shared `T59`/`T60` coverage rather than receiving invented tests.

## Task 1: Freeze the Execution Tree and Create the Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-sat.md`
- Read: governing inputs and current Git state

- [ ] Confirm branch, clean worktree, exact anchors, reachability, merge base, and sole-worktree boundary.
- [ ] Run every guard in the Baseline and Count Contract before changing a verdict.
- [ ] Confirm `node tools/takeover-ledger.mjs --summary-json` still reports 341 adjudicated and 590 unadjudicated.
- [ ] Read every governing input listed above; treat chapter line numbers as reconnaissance until live source confirms them.
- [ ] Create all 51 receipt headings in numeric order plus test, parameter, escalation, manual-evidence, history, blocker, and follow-up summaries before classifying any row.
- [ ] Record the ten-versus-nine escalation mismatch, SP-003, 71/20/8 parameter counts, two-engine boundary, manual-evidence boundary, and no-default discipline at the receipt top.

## Task 2: Adjudicate Model Identity and Dual-Water Rows `001`-`022`

**Files:**

- Modify: `docs/takeover/evidence/sb-sat.md`
- Read: `modules.rs`, `multimin2.rs`, `lrlc.rs`, `resultsqc.rs`, model/dialog/source registries, workflow persistence, exact tests/history

- [ ] Inventory every saturation method from manifest/enum through dispatch, option resolution, equation helper, outputs, stored run ID, UI label, Results QC label, and export identity.
- [ ] Re-derive the reference equations from the chapter/cited arithmetic; never use one engine as the other engine's oracle.
- [ ] Separate effective/total Archie, both Simandoux forms, every Indonesia `k`, Total Shale, Juhasz, Waxman-Smits, and both dual-water forms.
- [ ] Trace `B`, `Qv`, CEC, temperature, Rw, Rsh, shale porosity, alpha, beta, `vQ`, and `Swb` units from UI/IPC through every call site. Check wrong-unit refusals from both sides.
- [ ] Run exact candidate tests individually and classify their oracle. Hand-computed tests using the same formula can be correctness only when the arithmetic is independent and the source is named.
- [ ] Preserve every absent/default-conflicted value. Do not choose a `B` method, `vQ0`, alpha ceiling, Rsh, shale porosity, or exponent.

## Task 3: Adjudicate Back-Out, Guards, Outputs, and Parameter Rows `023`-`038`

**Files:**

- Modify: `docs/takeover/evidence/sb-sat.md`
- Read: saturation functions, workflow/output naming, registration, curve metadata, source panels, tests/history

- [ ] Trace each method's total-to-effective back-out independently and prove every inverse against sourced arithmetic.
- [ ] Inventory clipped/unclipped pairs, method flags, `VOL_UWAT`, `VOL_XWAT`, SWE/SWT/Sxo identities, and bare-SW reachability from creation through downstream selection.
- [ ] Inventory each iterative solver's seed, tolerance, iteration cap, convergence state, invalid-domain guards, and observable failure output. Explicitly test a cap-hit path.
- [ ] Verify low-porosity, total-zero, non-positive-Rt, coal, variable-m, and pure-shale behavior including volume outputs and flags.
- [ ] Trace all 71 parameter rows across manifests, structs, UI serialization, source topics, saved runs, Results QC, and export. A serde/default helper is a shipped default even when the UI looks blank.
- [ ] Keep the Rw correlation arithmetic, default custody, branch provenance, source comment, and regression tests as separate proof surfaces.

## Task 4: Adjudicate Seams, Provenance, and Gaps `039`-`051`

**Files:**

- Modify: `docs/takeover/evidence/sb-sat.md`
- Read: model registries, CLY/POR seams, Results QC, workflow, Inspector/history, report, export, calibration UI/backend, downstream selectors, tests/history

- [ ] Search positive and negative MUDBASE scope, clay-bound-water F routes, Poupon methods, SSM cap, Sxo methods/volumes, Worthington metadata, Rwa inversions, and mineral-conductivity disclosure.
- [ ] Trace run provenance through `log_sets`, Inspector, report methodology, LAS provenance lines, and saved/reloaded workflows. Check every scientific source, paper, unit, option, calibration state, input identity, and flag separately.
- [ ] Inspect Results QC and source/help surfaces for explicit equation disagreements and guidance; prove that guidance never changes the selected model automatically.
- [ ] Compare same-name deterministic, solver, and Results QC outputs on identical typed parameters and quantities. Explain every mismatch rather than normalizing it.
- [ ] Audit RtC/IMTS manifests, fit results, Apply path, run flags, provenance, and export against the binding calibration record. A fit test generated by the same forward model is supporting round-trip evidence unless an independent expected value is named.
- [ ] Confirm consequential absences in public Rust/TypeScript surfaces, tests, and `git log --all -S/-G`; no result from a narrow grep alone closes a gap.

## Task 5: Classify All 63 Test Intentions, Manual Evidence, and History

**Files:**

- Modify: `docs/takeover/evidence/sb-sat.md`
- Read: all discovered tests, `REVIEW.md`, `docs/VERIFICATION_MATRIX.md`, accepted/reachable history

- [ ] Route all 63 tests exactly once through the primary table and record every cross-supported requirement separately.
- [ ] For each candidate correctness proof, name the chapter/public source and independently reproduce the expected arithmetic. If source or independence is uncertain, classify it as characterization/supporting-only.
- [ ] Run every discovered candidate test by exact name. Record command, result, expected-value source, and assertion surface; compilation or source grep is not a behavioral pass.
- [ ] Check both sides wherever a lazy implementation could pass: mapped/unmapped aliases, canonical/wrong units, each method variant, convergence/non-convergence, binding/non-binding flags, suffixed/bare outputs, guidance/no auto-switch, and fitted/unfitted calibration.
- [ ] Record manual evidence only from checked review scenarios and the generated capability matrix. Do not add or check a scenario during this lane.
- [ ] Confirm consequential negative findings across expected code, UI, tests, and reachable history, and preserve every source/legal/product dependency explicitly.

## Task 6: Update the Ledger Atomically and Self-Review All 51 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-sat.md`

- [ ] Prepare all 51 RFC 4180-safe row changes as one `apply_patch`; preserve every non-SAT row and every source-owned field.
- [ ] Require no SB-SAT row to remain `UNADJUDICATED`, and populate every adjudication-owned mandatory field.
- [ ] Run the takeover-ledger and PRD-audit checks to prove immutable fields and structural findings remain unchanged.
- [ ] Cross-check receipt and CSV row by row: full contract, exact status, independent test class, manual evidence, source state, blocker, dependency, and smallest next action.
- [ ] Generate the measured summary. The expected mechanical count is 392 adjudicated and 539 remaining; all other totals must come from the resulting ledger.

## Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 51-row SAT adjudication result and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, accepted implementation anchor unchanged, worktree protection unchanged, and the security branch follow-up open until separately adjudicated.
- [ ] Add one recent-increment row with actual as-built/disposition/test totals, parameter/source/legal/manual blocks, and `539/931` rows remaining.
- [ ] Name the next serial petrophysics domain planning increment from the live dependency order; do not prepare or execute it automatically.
- [ ] Keep the banked POR Gate 2 remediation plan dormant unless separately authorized.

## Task 8: Verify, Commit the Domain Adjudication, and Stop

- [ ] Run `npm run test:takeover-ledger`, `npm run check:takeover-ledger`, the PRD-audit check, and verification-matrix check.
- [ ] Run `npx tsc --noEmit`, then `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1`; require zero failures and record exact ledger/frontend/Rust passed and ignored counts.
- [ ] Run `git diff --check`, inspect the full diff, and stage only `docs/takeover/evidence/sb-sat.md`, `docs/takeover/requirements.csv`, and `docs/takeover/STATUS.md`.
- [ ] Commit once as `G1-DOM-SAT adjudicate 51 SB-SAT requirements`; do not push, merge, begin a production fix, or start the next domain.

## Plan Self-Review Before Execution

- [ ] Exactly 51 live SB-SAT IDs are covered once; no requirement can be silently skipped.
- [ ] All 13 P0, 18 P1, 12 P2, 6 P3, and 2 P4 rows are covered without treating priority as automatic pilot policy.
- [ ] All source-owned priorities, chapter statuses, and owned-test mappings remain byte-identical.
- [ ] All 63 test intentions are routed exactly once and `049`/`051` retain shared support without invented IDs.
- [ ] All 71 parameter rows are inspected; 20 ABSENT-bearing and 8 tierless rows remain explicitly fenced.
- [ ] Ten live escalations and the section 8.16 nine-escalation claim remain a documented PRD mismatch, not a silent correction.
- [ ] SP-003 remains open and no Tier-C sonic-saturation capability is invented.
- [ ] Deterministic modules, solver post-processing, LRLC, and Results QC remain separate implementations until exact parity is proved.
- [ ] Bardon-Pied versus modified-Schlumberger, effective versus total, SWE versus SWT, CEC versus simple dual water, clipped versus unclipped, and method versus guidance remain distinct.
- [ ] A correct `Swb*(Cwb-Cw)` coefficient cannot hide missing alpha, beta, `vQ`, bounds, back-out, units, flags, or provenance.
- [ ] Every default and validation range is checked against a named source; current literals never become authority by repetition.
- [ ] Every correctness test names an independent expected-value source; round trips and internal helpers remain supporting evidence unless independently derived.
- [ ] Generic run/export provenance cannot substitute for parameter sources, papers, calibration state, method identity, and flags surviving the deliverable.
- [ ] Generic bare-SW aliases and selectors are checked even if no saturation module currently emits bare `SW`.
- [ ] Calibration remains declared, explicit, atomic, reversible, source-bounded, and never auto-applied.
- [ ] Automated evidence never closes the saturation, export, workflow, history, or stewardship manual gate.
- [ ] The adjudication changes no Rust, TypeScript, test, PRD, research, `REVIEW.md`, generated artifact, equation, parameter, unit, label, default, guard, output, or field-evidence record.
- [ ] Execution ends after one local commit; no push, merge, production remediation, or next-domain work occurs automatically.
