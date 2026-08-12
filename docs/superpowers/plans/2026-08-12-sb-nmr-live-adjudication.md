# SB-NMR Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 38 live `SB-NMR` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 57 chapter acceptance-test intentions, and preserve every absent cutoff, fluid property, coefficient, unit, source, inversion, legal, provenance, performance, and manual-evidence boundary without changing array storage, intake, plotting, interpretation, report, or export behavior.

**Architecture:** This is a documentation-only evidence pass over the generic array-log store and intake path, raw-byte IPC, array-track renderer and print renderer, module registry, generic permeability/saturation/Monte-Carlo helpers, unit registry, report/export surfaces, current tests, manual matrix, and reachable Git history. The immutable PRD supplies the intended contract, 42 parameter rows, 11 open items, 18 active escalation identifiers, 15 refusals, no Tier-C item, and 57 named test intentions. Current source, independently sourced tests, manual evidence, and reachable history supply the separate live verdict. A stored axis, a generic array display, a generic permeability equation, or a clean internal `Result` does not prove the compound observable NMR contract.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, tests, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Parameter custody, NMR method interpretation, legal/source boundaries, and final sign-off remain with the primary session.
- Work only in `D:\XX. SandiBumi`. The sole registered Git worktree MUST remain that path. `D:\XX. SandiBumi-check` remains untouched.
- The accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `4082926d6b5776e380baac1059c46186c180a64a`; fetched `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution.
- The local planning branch is `codex/g1-sb-nmr-plan`. The serial Gate 1 chain remains local and unpushed; do not merge, rebase, rewrite history, push, or open a pull request. After the planning commit, create `codex/g1-sb-nmr-adjudication` in the same worktree.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential Rust/TypeScript absence MUST be confirmed across expected source, tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/16_nmr.md`, the array sections of `docs/record_data_tools.md`, the current verification matrix, current takeover receipts/status, and the cited local dossier only where the chapter routes a consequential source dispute.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 38 ordered rows is `a6bdd774779db3db04e0f32706a2023caf97bc22cf03126e7f6f037350021670`. The immutable chapter SHA-256 is `ae471dee92a65e1ba89714c0686ad2edd13f3765af037d1e198e193b623b2617`.
- The chapter and ledger agree on 38 contiguous requirements: `P1=26`, `P2=10`, `P3=2`; there is no P0 item because the spine sequences NMR after first sale. Historical states are `ABSENT=35`, `PARTIAL=1`, and `PRESENT-DIVERGENT=2`. Reverify live behavior independently.
- The chapter defines 57 contiguous test IDs, `SB-NMR-T01` through `SB-NMR-T57`. The consolidated index references 54 test occurrences covering only 51 unique IDs; T03, T15, T16, T17, T26 and T34 are defined by the chapter but absent from the source-owned `owned_tests` fields. Route all 57 in the evidence receipt. Do not alter the immutable ledger field to repair this PRD/index discrepancy.
- Section 5 contains exactly 42 parameter rows, 16 reading `ABSENT - ships with no default`: `T2C_CBW`, `T2C_FF`, `SADDLE_WINDOW`, `D_CONN_ACTIVE`, `KSDR_C`, `RHO_SURF`, `RHO_GAS`, `T1_GAS`, `LAMBDA_DMR`, `PC_GAIN`, `PC_OFFSET`, `KAPPA`, `CCW_A`, the combined `Z_WET/Z_IRR` row, `A_TORT`, and `PC_SCALAR`. Preserve absence.
- Values for disabled Swanson, DMR-approximation, and pseudo-water extensions remain cited seeds behind explicit implementation gates. They are not active defaults and must not become one because a generic neighboring module happens to carry a number.
- A petrophysical value is cited or absent. Never select a cutoff candidate, disputed `CCW_A`, DMR lambda, SDR multiplier, gas property, Pc gain/offset/kappa, tortuosity, pseudo-water scalar, relaxation value, or performance threshold from training, a neighboring method, or one vendor seed.
- Preserve O-1 through O-11. Preserve the active escalation identifiers E1, E3, E4, E5, E6, E7, E8, E8a, E8b, E8c, E9, E10, E11, E12, E13, E14, E15 and E16; E2 is closed by the positive Z-slope evidence and must not be reopened. Preserve R-1 through R-15. No Tier-C item exists in this domain.
- First-release NMR consumes delivered distributions and excludes raw echo inversion. That is a scope contract, not authorization to import an echo train as an undifferentiated distribution or to reconstruct vendor regularization, hydrocarbon-typing binaries, fast-relaxation correction, chart payloads, or malformed correlations.
- Current storage writes one optional axis blob on every array row, but `ArrayRow`, `read_array_log`, `get_array_log`, `ArrayLog`, and `decodeArrayLog` omit it. The current heatmap histograms amplitude values over the track range. Reverify the whole write-to-view chain; do not call stored-but-unread bytes a physical-axis implementation.
- `write_array_log` validates depth/vector list cardinality and duplicate storable depths, but does not validate amplitude/axis length, finite positive strictly increasing axis, or one geometry across a curve set. `read_wide` parses numeric header positions and `commit_arrays` writes them; parsing a number is not the complete NMR geometry contract.
- No NMR-specific partition, spectral BVI, T2LM, Timur-Coates, SDR, DMR, T2-to-Pc, MRIAN, pseudo-water, typing, correction, provenance, or QC module symbol is present. Generic `perm_coates`, saturation, array-display, Monte Carlo, and processing-history features are supporting only.
- Fresh focused candidates pass `10/0/0`: five array-filtered Rust tests plus five exact intake/heatmap tests. They prove generic axis parsing/storage, replacement/refusal, array ordering, and value-heatmap behavior; none uses an executable `SB-NMR-Tnn` identifier or proves the full NMR contract.
- Manual evidence remains separate: array-logs `0/16`, delimited intake `3/27`, permeability `0/16`, saturation `2/97`, log-view `5/37`, report `6/53`, workflow `0/23`, processing-history `0/7`, and portfolio-performance `0/50`. Automated or desktop-harness evidence cannot close an unchecked manual scenario.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record:

1. branch `codex/g1-sb-nmr-adjudication`, created serially from the committed plan;
2. one clean worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, `origin/master`, merge base, and anchor reachability;
4. exactly 38 ledger rows, `SB-NMR-001` through `SB-NMR-038`, with no gap or duplicate;
5. priorities `P1=26`, `P2=10`, `P3=2`, and zero P0;
6. historical source states `ABSENT=35`, `PARTIAL=1`, `PRESENT-DIVERGENT=2`;
7. all 38 live status/disposition/risk/commit fields still unadjudicated;
8. exactly 57 defined chapter test IDs, 54 ledger references, 51 unique ledger IDs, and the six defined-but-unowned IDs named above;
9. exactly 42 parameters and the same 16 absent rows;
10. exactly 11 open items, 18 active escalation identifiers, 15 refusals, zero Tier-C items, and all section 8 traceability blocks;
11. takeover summary `607` adjudicated, `324` unadjudicated, and `432` pilot blockers before NMR;
12. manual capability counts listed above; and
13. the fresh `10/0/0` focused candidate-test receipt.

The only mechanically predictable post-adjudication ledger count is `645` adjudicated and `286` unadjudicated. Do not predict as-built, blocker, test-class, risk, or manual-evidence totals before row-by-row classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-nmr.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/db.rs`, `src-tauri/src/intake.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/workflow.rs`, `src-tauri/src/montecarlo.rs`, `src-tauri/src/units.rs`, `src-tauri/src/composite.rs`, `src-tauri/src/report.rs`, `src-tauri/src/export.rs`, `src/ipc.ts`, `src/ui/intakePanel.ts`, `src/ui/logViewPanel.ts`, `src/ui/layoutPropsDialog.ts`, current tests, manual evidence, immutable source chapter, cited local evidence, and reachable Git history
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-NMR-NNN` section per requirement in numeric order. Every section MUST include the specified contract, current implementation, as-built status, release disposition/risk, exact automated evidence class, manual evidence, source/parameter boundary, UI/IPC/provenance surface, history/reachability, blocking decision/dependency, and next action. Compound requirements must have each observable limb separated.

## Requirement Evidence Map

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Carry the physical T2 axis through storage, IPC and UI | `db::ArrayRow/read_array_log`; `lib::get_array_log`; `ipc::ArrayLog/decodeArrayLog`; log view. | Stored axis bytes are not proof when read/IPC/UI omit them. |
| `002` | Validate array geometry before accepting a distribution | `db::write_array_log`; `intake::read_wide/commit_arrays`; duplicate/cardinality tests. | Depth/vector cardinality and duplicate-depth refusal do not validate axis length, positivity, monotonicity or intra-set geometry. |
| `003` | Record acquisition and processing provenance | array schema/catalog/import request; curve/log metadata; report/export; negative NMR provenance search. | Generic set identity and filename text do not supply acquisition/inversion facts. |
| `004` | Reject defective recognition presets | intake mapping/templates and recognition inventories; negative NMR preset validator search. | Do not repair a defective preset with a guessed bin count or T2 endpoint. |
| `005` | Log every normalization or rebinning decision | intake/reframe/rebin/history/provenance inventories; source-array preservation. | A preview or resample without source/target axes, method and residual is not governed normalization. |
| `006` | Cutoffs ship absent and require explicit acceptance | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | Cited cutoff candidates are seeds, never defaults. |
| `007` | Partition is conservative across a cutoff inside a bin | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | Whole-bin assignment is not the declared inside-bin conservation rule. |
| `008` | Support values, saddle-point and spectral methods without hiding the branch | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | Do not hide whether values, saddle or spectral method supplied the answer. |
| `009` | Implement the cited thin-film spectral weighting | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | Generic weighting or a quadratic vendor taper is not the cited thin-film equation. |
| `010` | Emit both cutoff and spectral volumes in one run | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | One selected BVI without both source volumes and binding branch is incomplete. |
| `011` | T2 log mean is a time-windowed geometric mean | module registry/body; array input plumbing; negative partition/spectral/T2LM symbols; generic distribution helpers. | An ordinal-bin mean is not a physical-time T2LM. |
| `012` | Timur–Coates parameters are semantic and unit-typed | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | Generic Coates/Timur transforms do not consume the NMR distribution or prove semantic/unit parameter identity. |
| `013` | Guard the BVI denominator without disguising it | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | An output floor is not the cited BVI denominator guard and raw companion. |
| `014` | Modified Coates is optional and calibrated | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | Structural D_CONN=1 is not an active calibrated default. |
| `015` | SDR is null and flagged in hydrocarbon-bearing intervals | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | No multiplier or water-state may be inferred; hydrocarbon/unknown must remain invalid. |
| `016` | Carbonate SDR requires a sourced surface relaxivity | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | Never borrow a sandstone multiplier for carbonate relaxivity. |
| `017` | Swanson remains a sourced, disabled extension | `modules::perm_coates/perm_wyllie_rose`; module specs; negative NMR permeability symbols and guards. | Disabled cited seeds remain disabled until SBPC_MAX units and primary source close. |
| `018` | Full DMR is blocked until lambda is sourced | module registry; precalc/fluid-property types; Monte Carlo uncertainty; negative DMR symbols. | The fixed 0.6 approximation is not the full source-blocked DMR solve. |
| `019` | Gas hydrogen index is computed from gas density | module registry; precalc/fluid-property types; Monte Carlo uncertainty; negative DMR symbols. | Fixed HI seeds are not a computed density relation and none is preselected. |
| `020` | DMR propagates input uncertainty and flags clamps | module registry; precalc/fluid-property types; Monte Carlo uncertainty; negative DMR symbols. | Generic Monte Carlo does not prove DMR input uncertainty, raw companions or clamp flags. |
| `021` | Fluid properties remain measured or explicitly sourced | module registry; precalc/fluid-property types; Monte Carlo uncertainty; negative DMR symbols. | No vendor fluid seed becomes a default; temperature conventions stay distinct. |
| `022` | Pc gain and offset carry both unit ends | SCAL/Pc import and saturation-height types; unit registry; negative T2-to-Pc symbols. | A Pc offset without both unit ends is invalid; ms-to-s changes the intercept. |
| `023` | Kappa is unit-typed and never defaulted | SCAL/Pc import and saturation-height types; unit registry; negative T2-to-Pc symbols. | Bare kappa is invalid even when its number gives a plausible Pc. |
| `024` | Every Pc output carries pressure unit, datum and saturation convention | SCAL/Pc import and saturation-height types; unit registry; negative T2-to-Pc symbols. | Do not infer array/entry-pressure units, datum or wetting convention from a sibling output. |
| `025` | T2-to-Pc requires a water-saturated distribution | SCAL/Pc import and saturation-height types; unit registry; negative T2-to-Pc symbols. | A generic imported array is not proven water-saturated; cumulative direction must persist. |
| `026` | Implement primary-source MRIAN as the canonical NMR saturation method | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | Generic saturation or IP prose is not primary-source MRIAN. |
| `027` | Clay-water conductivity coefficient ships absent | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | The 0.000216/0.000126 conflict remains visible and unselected. |
| `028` | Keep effective and total irreducible saturation distinct | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | Bare SWIRR or SSC irreducible curves cannot substitute for NMR effective/total identities. |
| `029` | IP compatibility uses the resolved positive Z slope | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | Positive Z slope is resolved structurally; endpoints remain explicit compatibility inputs. |
| `030` | Tortuosity is not silently dropped between saturation paths | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | MRIAN must refuse A_TORT; compatibility must preserve it. |
| `031` | NMR pore-volume ratios never masquerade as shale volumes | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | Mnemonic shape cannot turn a pore-volume saturation into bulk shale volume. |
| `032` | Pseudo-water substitution enforces ordering and water-leg calibration | saturation/LRLC module registry; curve families; negative MRIAN/pseudo-water symbols. | Do not guess the malformed pseudo-water parenthesis or bypass ordering/calibration. |
| `033` | Hydrocarbon typing is independently derived from published NMR literature | model/binary loader inventory; negative NMR typing implementation; legal/source boundary. | A loader refusal alone does not implement independently derived hydrocarbon typing. |
| `034` | Echo inversion is excluded from first-release NMR | intake/UI command inventory; raw-array acceptance; negative echo-inversion command; first-release scope. | Feature absence is not enough if raw echoes can enter as an undifferentiated generic array; prove scope refusal and delivered-distribution handoff. |
| `035` | Detect but do not reproduce undocumented fast-relaxation correction | intake metadata/curve comparison inventory; negative fast-relaxation provenance detector. | Detection/provenance is required; do not reconstruct the compiled correction. |
| `036` | NMR heatmaps use the physical T2 axis | `logViewPanel::drawArrayHeatmap`; `ArrayLog`; axis-loss chain; composite heatmap characterization. | A value histogram is the wrong coordinate system even if it renders cleanly. |
| `037` | Every output carries method and parameter provenance | array/import/module provenance types; reports/exports; negative NMR result-provenance record. | Generic processing history does not prove output-level NMR method/parameter/export provenance. |
| `038` | QC flags are explicit curves and run-summary counts | module results/run summary/flag curves; array geometry path; negative NMR QC-code inventory. | Internal errors or NaNs are not stable flag curves plus observable run-summary counts. |

## Acceptance-Test Ownership Map

| Test | Primary receipt owner | Exact proof intention |
|---|---|---|
| `T01` | `001` | stored axis returns byte-identical with ms unit. |
| `T02` | `002` | axis/amplitude length mismatch refuses before write. |
| `T03` | `002` | non-increasing physical axis refuses. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T04` | `002` | intra-set bin-count change splits or refuses. |
| `T05` | `004` | defective recognition row names contradiction. |
| `T06` | `006` | absent cutoff refuses while unselected seeds remain visible. |
| `T07` | `007` | inside-bin partition conserves total amplitude. |
| `T08` | `007` | cutoff on edge has no double count. |
| `T09` | `009` | cited sandstone spectral weight arithmetic. |
| `T10` | `010` | cutoff, spectral and selected volumes plus branch all emit. |
| `T11` | `008` | maximum rule refuses when B_SPEC differs from one. |
| `T12` | `011` | T2LM remains stable under sourced rebinning. |
| `T13` | `011` | physical-time bounds agree while ordinal-only input refuses. |
| `T14` | `012` | canonical NMR KTIM arithmetic. |
| `T15` | `012` | alternative sourced coefficient and ratio remain distinct. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T16` | `012` | v/v and porosity-unit forms agree. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T17` | `012` | semantic mappings agree and letter-only input refuses. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T18` | `012` | ratio and SWIRR_E forms agree. |
| `T19` | `013` | zero BVI uses cited guard and preserves zero. |
| `T20` | `014` | D_CONN identity equals standard KTIM. |
| `T21` | `014` | uncalibrated active D_CONN refuses. |
| `T22` | `015` | SDR in hydrocarbon is null with reason. |
| `T23` | `015` | missing SDR multiplier exposes conflict without selection. |
| `T24` | `018` | full DMR refuses without lambda. |
| `T25` | `019` | gas HI derives from supplied density. |
| `T26` | `018` | 0.6 approximation remains labelled and retains outputs. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T27` | `020` | DMR raw/clamped companions and flag. |
| `T28` | `023` | psi-s kappa arithmetic. |
| `T29` | `023` | psi-ms equivalent plus bare-value refusal. |
| `T30` | `022` | Pc offset ms-to-s conversion at unit gain. |
| `T31` | `022` | offset conversion scales with arbitrary gain. |
| `T32` | `024` | bar/psi round trip carries exact cited factor. |
| `T33` | `025` | hydrocarbon-bearing T2-to-Pc refuses. |
| `T34` | `025` | long-to-short cumulative direction persists. **Index-owned-test gap:** route in receipt; do not alter source-owned ledger field. |
| `T35` | `027` | both disputed CCW values reproduce sourced arithmetic. |
| `T36` | `027` | literal hard-coded CCW candidate is prohibited. |
| `T37` | `026` | MRIAN wet/irreducible clamps and flags. |
| `T38` | `026` | W_EST remains QC-only. |
| `T39` | `029` | positive Z-slope compatibility arithmetic. |
| `T40` | `029` | opposite-sign prose corrects with warning. |
| `T41` | `030` | MRIAN refuses non-unity tortuosity. |
| `T42` | `031` | pore-volume ratio cannot map to VSH. |
| `T43` | `032` | pseudo-water ordering/calibration preconditions. |
| `T44` | `032` | malformed correlation stays disabled. |
| `T45` | `036` | physical T2 axis drives heatmap positions. |
| `T46` | `035` | corrected-versus-uncorrected comparison flags. |
| `T47` | `038` | stable flag curve and run-summary count. |
| `T48` | `003` | acquisition/inversion metadata round-trips with unknowns preserved. |
| `T49` | `005` | rebin retains source, both axes, method and residual. |
| `T50` | `016` | carbonate SDR refuses missing relaxivity. |
| `T51` | `017` | Swanson stays disabled on unresolved unit basis. |
| `T52` | `021` | measured fluid properties win while seeds remain unselected. |
| `T53` | `024` | Pc output metadata carries conversion, datum and convention. |
| `T54` | `028` | effective and total irreducible saturation stay distinct. |
| `T55` | `033` | vendor binary refuses as typing implementation. |
| `T56` | `034` | raw echo unsupported while delivered distribution enters validation. |
| `T57` | `037` | computed output/export carries complete provenance. |

## Parameter, Open-Item, Escalation, Refusal, and Traceability Custody

- Preserve all 42 parameter rows and the 16 absent entries exactly. Cited cutoff, gas-HI, Swanson, approximation, and pseudo-water values remain contextual seeds unless the chapter explicitly makes them active.
- Keep O-1 through O-11 explicit. O-1 blocks MRIAN's disputed coefficient, O-2 blocks full DMR, O-3 keeps Pc constants absent, O-4/O-5 protect inversion and typing boundaries, O-6/O-7 keep malformed correlations disabled, O-8/O-9 keep permeability extensions disabled, O-10 keeps recognition geometry absent, and O-11 keeps the device/performance gate unmeasured.
- Preserve all active escalation identifiers listed above and record E2 as closed without reopening it. A local data-acquisition need is not permission to synthesize a parameter.
- Preserve R-1 through R-15 separately: no cutoff borrowing, whole-bin rounding, hidden branch, letter-keyed coefficients, output-clamp disguise, SDR in hydrocarbons, default 60/40 DMR, unitless Pc offset, implicit unit mixing, disputed CCW choice, wrong-sign Z, saturation-as-shale naming, guessed parenthesis, amplitude-as-realizations, or undocumented correction reproduction.
- Preserve every section 8 evidence, method, difference, gap, optimal-choice, test, critique, and completeness disposition. Do not edit the dossier or PRD in Gate 1.

## Execution Tasks

### Task 1: Refreeze the baseline

- [ ] Create `codex/g1-sb-nmr-adjudication` from the committed plan and verify one clean worktree at `D:\XX. SandiBumi`.
- [ ] Re-run branch, anchor, hashes, row, priority, source-state, test-ID, parameter, open/escalation/refusal, traceability, manual-evidence, and focused-candidate checks.
- [ ] Stop and reconcile any changed hash, count, branch relationship, or source-owned field.

### Task 2: Reverify array identity, geometry, provenance, scope, and display (`001`-`005`, `034`-`036`)

- [ ] Trace axis bytes from intake header through write, storage, read, raw-byte IPC, frontend decode, interactive view, composite view, and export.
- [ ] Test the distinctions between depth/vector cardinality, amplitude/axis cardinality, finite-positive-monotonic axis, and intra-set geometry changes.
- [ ] Inventory acquisition/inversion metadata, recognition presets, normalization/rebin records, corrected companions, raw-echo entry points, and delivered-distribution scope.
- [ ] Prove the current heatmap coordinate semantics from implementation, not its label.

### Task 3: Reverify partition, spectral, T2LM, and permeability (`006`-`017`)

- [ ] Confirm all missing NMR-specific module/result/curve symbols across source, tests, UI, IPC, reports, exports, and history.
- [ ] Separate generic distribution/permeability helpers from the chapter's axis-aware input, semantic parameters, water validity, guards, raw companions, and source-gated extensions.
- [ ] Preserve all absent cutoffs/multipliers/relaxivity/active-connectivity values and disabled extension gates.

### Task 4: Reverify DMR and T2-to-Pc (`018`-`025`)

- [ ] Inventory DMR inputs, density/HI/T1 types, uncertainty outputs, raw/clamped companions, and approximation labels.
- [ ] Inventory Pc unit carriers, offset/kappa conversions, output metadata, water-saturation precondition, and cumulative direction.
- [ ] Keep lambda, fluid properties, Pc constants, and malformed correlations absent.

### Task 5: Reverify MRIAN and pseudo-water contracts (`026`-`032`)

- [ ] Confirm MRIAN, compatibility, irreducible-saturation, tortuosity, saturation-naming, and pseudo-water symbols are absent or identify exact live seams.
- [ ] Keep both CCW candidates visible and unselected; do not reopen the positive Z-slope decision.
- [ ] Keep malformed pseudo-water and compiled behavior disabled; generic saturation methods do not substitute.

### Task 6: Reverify typing, provenance, and QC (`033`, `037`, `038`)

- [ ] Inventory model/binary loader refusals, published-method registration, result provenance, plot/export custody, stable QC codes, flag curves, and run-summary counts.
- [ ] Separate correct absence/refusal boundaries from unimplemented positive capabilities.
- [ ] Preserve all manual evidence as user-owned and unchanged.

### Task 7: Write the evidence receipt and update all 38 ledger rows

- [ ] Create `docs/takeover/evidence/sb-nmr.md` with one complete section per requirement.
- [ ] Route all 57 test intentions exactly once, including the six defined-but-unowned IDs, while preserving the ledger's source-owned `owned_tests` field.
- [ ] Update only live adjudication columns in `docs/takeover/requirements.csv`.
- [ ] Recompute and update `docs/takeover/STATUS.md` with exact live counts.
- [ ] Reverify the frozen six-column hash.

### Task 8: Run structural checks, gates, and commit

- [ ] Assert 38 receipt sections, all legal/non-placeholder ledger fields, all 57 test routes, all 42 parameters, 11 opens, 18 active escalation identifiers, 15 refusals, zero Tier-C items, and section 8 custody.
- [ ] Search new receipt/ledger text for identifying names, invented values, guessed algorithms, vendor binaries/chart payloads, and false field-evidence claims.
- [ ] Run `node --test tools/takeover-ledger.test.mjs`, `node tools/takeover-ledger.mjs --summary-json`, and `git diff --check`.
- [ ] Run `npx tsc --noEmit`, `cargo check`, and `powershell -ExecutionPolicy Bypass -File tools\check.ps1`, using the established isolated Cargo target if needed.
- [ ] Stage exactly the NMR receipt, ledger, and STATUS; inspect the cached diff and source hash.
- [ ] Commit locally with message `G1-DOM-NMR adjudicate 38 SB-NMR requirements`. Do not push, merge, or open a pull request.

### Task 9: Continue Gate 1 serially

- [ ] Recompute remaining domains after NMR while SB-GEO remains deferred to the next version.
- [ ] Select the next dependency-relevant petrophysics domain from live evidence.
- [ ] Create the next serial planning branch in `D:\XX. SandiBumi`.
- [ ] Do not begin Gate 2 production remediation until Gate 1 evidence reconciliation completes or Jauhar explicitly changes sequence.

## Plan Self-Review Receipt

- **Coverage:** all 38 requirements appear once; all 57 chapter tests appear once; the six source-index ownership gaps are routed without changing the immutable source fields; all 42 parameters, 11 opens, 18 active escalation identifiers, 15 refusals, zero Tier-C items, and section 8 blocks have custody.
- **Live drift:** storage-axis existence is separated from read/IPC/UI loss; generic geometry checks are separated from the NMR-axis contract; generic amplitude heatmap behavior is recorded as divergent rather than credited.
- **Source discipline:** all 16 absent parameters remain absent; disputed values and malformed methods stay unselected/disabled; E2 remains closed; no vendor algorithm, chart, coefficient, cutoff, fluid property, or threshold is invented.
- **Evidence discipline:** the 10 passing candidates prove generic array seams only. None is an executable `SB-NMR-Tnn` whole-contract proof, and manual array-log evidence remains `0/16`.
- **Scope discipline:** planning changes only this plan and STATUS; execution changes only the NMR receipt, ledger, and STATUS. Production remediation remains Gate 2.

## Execution Handoff

Commit this planning increment locally, create `codex/g1-sb-nmr-adjudication` serially from it, then execute inline with `superpowers:executing-plans`. Jauhar's standing direction is to continue Gate 1 serially, so no additional approval is required unless execution reaches an unresolved product-owner, source, legal, or performance decision that would otherwise be guessed.

