# SB-CUT Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 61 live `SB-CUT` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 44 acceptance-test intentions, and preserve every unresolved source, unit, product, and vendor-behaviour question without changing cutoff, summation, Monte Carlo, reporting, or import behavior.

**Architecture:** This is a documentation-only evidence pass over the shared cutoff custody, deterministic pay summary, cutoff sweep, Monte Carlo engine, result records, report and office-deliverable surfaces, IPC contracts, Results QC, manual matrix, and reachable Git history. The immutable PRD supplies the intended contract, original priority, chapter status, 44 parameter rows, open questions, escalations, refusals, and named test intentions. Current source, independently derived tests, manual evidence, and reachable history supply the separate live verdict recorded in the takeover ledger and one SB-CUT evidence receipt. A hard-coded constant, an internal `Result`, a self-referential computation, or a UI field that merely displays a value does not prove parameter authority or a client-visible result contract.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Parameter custody, petrophysical interpretation, data-integrity judgment, and final sign-off remain with the primary session.
- Work only in `D:\XX. SandiBumi`. The sole registered Git worktree MUST remain that path. `D:\XX. SandiBumi-check` is not a repository or evidence source.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `8ed522b744a8949e60c643a2ebcfc4bd45c747af`; fetched `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution and stop to reconcile if any moves.
- The local branch is `codex/g1-sb-cut-adjudication`. The serial Gate 1 chain remains intentionally local and unpushed; do not switch, merge, rebase, rewrite history, push, or open a pull request.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, exact tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/14_cutoffs-summation-mc.md`, applicable sections of `docs/record_fixes.md`, `docs/record_data_tools.md`, `docs/record_calibration.md`, and `docs/record_parallel_lanes.md`, the current manual matrix, and existing takeover receipts/status.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 61 ordered rows is `40d21a70e16657bda4779deffa66c7d52467458f1aca09eead6623de19e0a3fe`. The immutable chapter SHA-256 is `e4972351ae548204a92300f3ab75c0b9415af6f6a2e72709be6119e553e41359`.
- The chapter and ledger agree on 61 contiguous requirements: `P0=9`, `P1=23`, `P2=22`, `P3=7`. Chapter states are `ABSENT=29`, `PARTIAL=12`, `PRESENT-DIVERGENT=6`, `PRESENT-OK=13`, and `PRESENT-UNVERIFIED=1`. Reverify live behavior independently; do not copy these states into the live verdict.
- The chapter defines exactly 44 acceptance-test IDs: numeric `T01` through `T39`, with `T02b`, `T03b`, `T03c`, `T37b`, and `T37c`, and no `T38` omission despite its ordering after `T39` in the chapter table. Route every intention exactly once as a primary proof target.
- Section 5 contains exactly 44 parameter rows. Eight are `ABSENT - ships with no default`; eleven are `NON-ADOPTABLE - cited for verification`. Every parameter is CITED or ABSENT. Never adopt a source-code literal, an incumbent value, a neighboring-vendor value, an average, a midpoint, a remembered textbook value, or a plausible engineering value as authority.
- Preserve all twelve open questions `O-1` through `O-12`, all six escalations `E-1` through `E-6`, and all thirteen refusals `R-1` through `R-13`. `E-1` is resolved only as a prospective exposure decision; the required `SD_MULT=2` implementation and regression test are still open.
- `O-2` is the largest evidence gap and blocks the bed-amalgamation algorithm in `SB-CUT-013`. Do not infer tie-breaking from one worked example or invent it.
- `E-3` leaves the `B fact` unit as `null` with `unit_source: "not stated"`; never infer Qv's unit from the help page. `E-4` forbids a house `Rw` prior without a source/product decision. `E-5` keeps cutoff selection guidance absent while this domain implements only machinery. `E-6` preserves the unresolved CONTRACT section 2.1 transcription boundary.
- No cutoff default is defensible. The live quartet `VSH=0.5`, `PHIE=0.1`, `SWE=0.6`, `PERM=null` is implementation evidence of divergence, not parameter authority. Do not retain it merely because it is centralized.
- No generic uncertainty width is defensible. The UI fallback `max(abs(value)*0.1, 0.01)` and the IP-badged width table are implementation evidence only. The IP widths are verification data, not SandiBumi defaults; where used, the chapter requires explicit `SD_MULT=2`, so the current direct use is a two-times sigma defect.
- `as_built_status` answers what the accepted tree currently ships. `release_disposition` answers whether the observable contract is required for the paid offline Windows open-hole-petrophysics pilot. Priority and chapter status inform but do not mechanically decide either field.
- A compound requirement is not `PRESENT-OK` because one engine, pane, row, output field, or test is correct. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, a list, a provenance/export clause, or a refusal/control pair.
- A test is qualifying owned acceptance proof only if it exercises the observable contract and derives the expected value independently from a cited source or explicit arithmetic. An internal `Result`, source-text grep, compile success, same-code round trip, current-library snapshot, or distribution generated and judged by the same implementation is supporting evidence only.
- Classify each requirement's proof as `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING` under CONTRACT sections 3 and 6. A test pinning current divergent behavior is divergence evidence, never proof of the specified behavior.
- The fresh candidate suite passes: workflow `36/0/1 ignored`, Monte Carlo `23/0/0`, report `13/0/0`, core reporting `1/0/0`, net flag `8/0/0`, and frontend acceptance `13/0/0`. These 94 passing tests are candidates, not 61 automatic contract closures.
- The three recovered degraded-result defects have observable regression locks: rendered uninterpreted versus classified-zero rows, emitted PDF degradation plus batch record, and failed Monte Carlo job item. Do not replace those with weaker internal-result assertions.
- Manual evidence is separate and currently remains: `cutoffs-pay=0/23`, `workflow=0/23`, `field-dashboard=0/10`, `results-qc=0/1`, `monte-carlo=2/14`, and `report=6/53`. Automated or desktop-harness evidence cannot close an unchecked manual scenario.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record all of the following:

1. branch `codex/g1-sb-cut-adjudication`;
2. a clean worktree and the sole registered worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, fetched `origin/master`, merge base, and accepted-anchor reachability;
4. exactly 61 ledger rows, covering `SB-CUT-001` through `SB-CUT-061` once with no gap or duplicate;
5. priority counts `P0=9`, `P1=23`, `P2=22`, `P3=7`;
6. source-status counts `ABSENT=29`, `PARTIAL=12`, `PRESENT-DIVERGENT=6`, `PRESENT-OK=13`, `PRESENT-UNVERIFIED=1`;
7. all 61 source-owned `owned_tests` populated and all 61 live `as_built_status=UNADJUDICATED`;
8. exactly 44 defined test IDs and the same 44 unique IDs referenced by the ledger, with no defined-but-unowned or owned-but-undefined ID;
9. exactly 44 parameter rows, eight `ABSENT`, eleven `NON-ADOPTABLE`, and no source-free value promoted to authority;
10. exactly twelve opens, six escalations, and thirteen refusals;
11. takeover summary `438` adjudicated, `493` unadjudicated, and `326` pilot blockers before any CUT edit;
12. the manual capability counts listed in Global Constraints; and
13. the fresh candidate-test receipt listed in Global Constraints.

The only mechanically predictable post-adjudication ledger count is `499` adjudicated and `432` unadjudicated. Do not predict as-built, pilot-blocker, test-class, or manual-evidence totals before row-by-row classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-cut.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/workflow.rs`, `src-tauri/src/montecarlo.rs`, `src-tauri/src/netflag.rs`, `src-tauri/src/report.rs`, `src-tauri/src/office.rs`, `src-tauri/src/core_reporting_tests.rs`, `src/ui/cutoffs.ts`, `src/ui/cutoffDialog.ts`, `src/ui/summaryDialog.ts`, `src/ui/dashboardPanel.ts`, `src/ui/monteCarloDialog.ts`, `src/ui/reportDialog.ts`, `src/ui/resultsQcPanel.ts`, `src/ui/workbookDialog.ts`, `src/ui/deckDialog.ts`, `src/ipc.ts`, current tests, reachable Git history, manual evidence, and the immutable source chapter
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts

## Evidence Receipt Schema

Create one `### SB-CUT-NNN` section per requirement in numeric order. Every section MUST include:

- **Specified contract:** every observable limb, separated when compound.
- **Current implementation:** exact symbols and paths, plus explicit negative-inventory scope where absent.
- **As-built status:** one legal ledger state and why a stricter state fails.
- **Release disposition and risk:** pilot relevance independent of implementation status.
- **Automated evidence:** exact test names and commands classified `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`, with the independent expected-value source named.
- **Manual evidence:** exact checked capability/scenario or `NONE`; never inferred from automation.
- **Source/parameter boundary:** cited, absent, withdrawn, vendor-derived, non-adoptable, derived, engineering guard, conflicting, or not applicable, with open/escalation/refusal IDs.
- **UI/IPC/provenance surface:** request field, editor control, result object, log-set record, Results QC, workflow, report, office export, and batch/job surface as applicable.
- **History/reachability:** accepted commit evidence or confirmed negative history search where consequential.
- **Blocking decision/dependency:** exact source, legal, product, UI, manual, or implementation dependency.
- **Next action:** the smallest bounded follow-up, or `NONE` only when the whole contract is proved and no field evidence is required.

## Requirement Evidence Map

### Group A - Discretisation, footage, averaging, and volumetrics (`001`-`015`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Explicit CENTRED/TOPS/BOTTOMS parameter, shared interval ownership and clipping across summary, sweep, and MC. | `run_pay_summary`; `compute_sweep`; `zone_metrics`; thin-zone tests. | Three similar forward-interval implementations are not one implementation and do not expose the model. |
| `002` | Every thickness-bearing result records discretisation model and sample interval. | `PaySummaryRow`; `McZoneResult`; reports and office exports. | A comment or implicit algorithm is not result custody. |
| `003` | Gross, Net, NotNet, Unknown exact partition. | `PaySummaryRow`; flag NaN handling; report tables. | Current gross and net alone cannot prove the four-way partition. |
| `004` | Both `N:G` and `N:(G-Unknown)` labelled. | `PaySummaryRow.ntg`; report/dashboard/office fields. | One ratio is absence, even if Unknown is currently zero in a fixture. |
| `005` | Named relative reconciliation tolerance, recorded residual/absorption, structured refusal beyond tolerance. | Negative search in summary/sweep/MC result fields. | Arithmetic closure by construction is not an explicit reconciliation contract. |
| `006` | General power mean with explicit per-curve exponent. | Summary averaging; module manifests; result metadata. | Arithmetic-only special cases do not satisfy the general family. |
| `007` | Weight-normalised geometric mean plus non-positive exclusions. | Negative search for geometric aggregation and exclusion count. | Never adopt IP's unit-dependent form; R-2 remains binding. |
| `008` | Harmonic mean skips non-positive samples without aborting interval. | Negative search for harmonic aggregation. | Scope vendor defect claims to the shipped script under O-1. |
| `009` | Porosity weighting is an explicit per-curve flag, never mnemonic inference. | PHIE-weighted Sw in summary/MC; metadata types. | Correct PHIE arithmetic hard-coded by name is partial at most. |
| `010` | Direct HCPV equals reconstructed Net x PhiAvg x (1-SwAvg) under phi weighting. | Summary and MC HPV arithmetic; report consistency test. | Current shared arithmetic needs independent identity proof, including the thickness-weighted negative control. |
| `011` | Samples outside every zone excluded from cumulative results. | Zone-overlap loops and `cutoff_sweep_ntg_and_dst_mask`. | Confirm summary, sweep, and MC, not one path. |
| `012` | Reference frame is part of result identity. | Request/result types and reports; O-3. | A selected TVD curve or current MD loop without persisted identity is absent. |
| `013` | Three independent bed-amalgamation thresholds with defined tie-breaking. | Negative source/history inventory. | O-2 blocks the algorithm; do not infer from the worked example. |
| `014` | Bed statistics emitted before and after amalgamation. | Negative result/report inventory. | A net total alone is not bed statistics. |
| `015` | Bed-thickness convention explicitly stated. | Result/report metadata. | Sample interval alone is not the convention. |

### Group B - Cutoff custody, typing, expressions, and tiers (`016`-`030`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `016` | Fresh project ships no cutoff value. | `DEFAULT_CUTOFFS`; all nine current cutoff-bearing UI surfaces. | Centralizing an uncited quartet makes the defect consistent, not correct. |
| `017` | Every actual default carries a source string. | `CutoffDefaults`; document schema; UI help/provenance. | No source-free default may be grandfathered. |
| `018` | Every cutoff entry/display surface resolves one authority. | Eight `loadCutoffDefaults` consumers plus dashboard literals; source inventory test target. | The chapter's older six-pane snapshot is not current inventory; enumerate live surfaces. |
| `019` | Cutoff entry requires units and canonical conversion. | Bare numeric fields and Rust float request fields. | A label containing PHIE or PERM is not typed unit custody. |
| `020` | Two-sided range with explicit bounds operator. | `PaySummaryRequest`; `Cutoffs`; classifier comparisons. | Fixed one-sided `<=`/`>=` is absent. |
| `021` | Cutoff may be supplied as a curve. | Request and curve-resolution inventory. | A scalar curve name elsewhere does not satisfy per-sample cutoff custody. |
| `022` | Explicit enabled flag; one value shared across reservoir/pay tiers. | Nullable `perm_min`; mandatory VSH/PHIE/SWE; tier classifier. | Null-as-off for one field does not prove explicit activation for all. |
| `023` | Boolean cutoff expression with AND/OR/parentheses. | Fixed classifier and free-form net-flag polygon. | A separate polygon tool is not the pay-summary expression engine. |
| `024` | Arbitrary named flag tiers over arbitrary cutoff sets. | `SUMMARY_FLAGS`; result/report tier fields. | Three fixed names are partial at most. |
| `025` | Lumps are a many-to-one reporting transform over flags. | Negative source/history inventory. | Do not modify flags or infer bed merging. |
| `026` | Saturation disabled at reservoir tier by default. | `classify_sample` cascade and results. | Verify reservoir stays independent of SWE and pay still uses it. |
| `027` | No arbitrary cap on curves, cutoffs, report tiers, or flags. | Dynamic collections and UI controls. | Fixed three tiers/cutoffs may make the compound contract divergent despite no numeric vector cap. |
| `028` | SWE and SWT are explicit; no bare SW output. | Module manifests, output inventory, reports. | Confirm all emitting paths, not only saturation modules. |
| `029` | Null markers are typed sibling fields. | `n_classified`, `perm_cutoff_no_data`, row/result schemas. | One absence discriminator does not prove typed null custody for all relevant outputs. |
| `030` | Accumulate, flag-test, and present clamps are separate and declared. | Pay PHIE floor, module limited/unlimited curves, MC plausibility, UI rendering. | Current clamps are scattered and partly silent; never treat implementation comments as an observable stage contract. |

### Group C - Monte Carlo prior, sampling, accumulation, and results (`031`-`053`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `031` | Explicit mandatory sigma multiple; IP widths use `SD_MULT=2`. | `IP_MC_SEEDS`; `seedWidth`; `Distribution::Normal`. | Current direct width-to-sigma use is exactly 2x wide; E-1 removes delivered exposure, not the defect. |
| `032` | Store shift type with width; refuse Rec-to-Linear coercion. | `McSeed.pct`; `Distribution`; import inventory. | Percent-vs-absolute boolean cannot express reciprocal shift. |
| `033` | Import measurement priors as well as parameter priors. | UI parameter list and absent importer. | A manually added module parameter is not an imported measurement prior. |
| `034` | Mandatory seed and seed on result record. | `McRequest.seed`; persisted `params_json`; `McResult`; job record. | Request custody alone may be partial if the returned/reportable result omits seed. |
| `035` | Log-domain distributions. | `Distribution` enum. | Normal/uniform/triangular on linear value are absent. |
| `036` | Every prior is value/basis/sigma-multiple triple with units. | `McParam`; UI `McSeed`; distribution editor. | Central value plus width is insufficient; non-adoptable rows never become defaults. |
| `037` | Store centring rule per prior. | `Distribution.central`; request/result persistence. | Implicit mean/midpoint/mode selection is not stored custody; O-4 remains open for IP import. |
| `038` | Gaussian truncation plus realised variance-deficit report. | normal sampler and result fields. | No silent clipping and no invented variance correction. |
| `039` | Cited 2000 default and auto-stop on the reported percentile. | UI 1000 default; `converge`; checks and tolerance. | Existing optional convergence with uncited settings is divergent; preserve O-7. |
| `040` | One offset per section per iteration. | draw matrix and zone-span fills; T27 target. | Do not confuse one offset per parameter with proof of section-level invariance across depth. |
| `041` | Never clamp before accumulation. | `zone_metrics`; module limited/unlimited outputs; plausibility path. | Accumulating clamped PHIE/SWE is a silent reserves bias even if out-of-range companions are reported. |
| `042` | Perturb cutoffs per zone under MC. | `Cutoffs` scalar struct and draw schedule. | Parameter-zone scoping does not perturb cutoff values. |
| `043` | Derived ratios computed inside each iteration. | `zone_metrics.ntg` then per-zone `summarize`. | Confirm actual varying-gross negative control; code shape alone is supporting. |
| `044` | Per-iteration joint records and iteration-consistent percentile cases. | in-memory `per_real`; `McResult` fields. | Transient vectors are not an output record or labelled case. |
| `045` | Withhold unsupported statistic with machine-readable reason. | `summarize`; five-iteration P10 behavior; result errors/notes. | NaN, minimum, or unconditional output is not refusal. |
| `046` | Percentile interpolation method named on output. | type-7 helper; `McResult`; persisted params. | Implemented arithmetic without a result field is partial at most. |
| `047` | Reserves categories carry actual probabilities and per-quantity direction. | `low_pctl/high_pctl`; result labels; O-5. | Generic low/mid/high is not reserves-category custody. |
| `048` | Cross-zone roll-up merges cases, not marginal statistics. | Per-zone outputs and absent field roll-up. | O-11 remains open for incumbent comparison; SandiBumi still needs its own explicit behavior. |
| `049` | Requested and realised correlation both reported. | `McCorrelation`; Iman-Conover notes; sensitivity fields. | Parameter-output Spearman is not realised parameter-pair correlation. |
| `050` | Data-picked parameters re-derived each iteration. | draw and module execution plan. | Correlation support does not satisfy re-derivation. |
| `051` | Tornado bars in absolute output units. | `MetricSet` OAT endpoints; renderer. | Plot pixels or percentage labels are not an explicit units field; T34 requires run-count control. |
| `052` | Newly added prior starts perturbation off. | UI row creation and default check state. | A nonzero seeded/generic width on creation is divergent. |
| `053` | Impossible realizations reported, never excluded. | `McPlausibility`; per-real accumulation. | Verify all realizations stay in percentiles and the diagnostic covers relevant unlimited curves. |

### Group D - Degraded results, IPC, sensitivity, import, and formatting (`054`-`061`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `054` | All-realization chain failure is a failed job item with source error, never a zero uncertainty result. | `core_reporting_tests::a_monte_carlo_chain_failure_is_reported_in_the_job_and_never_as_a_zero_uncertainty_result`. | Must remain a job/result-surface test, not only `McResult.errors`. |
| `055` | Uninterpreted well renders absent while a classified zero remains numeric. | Frontend acceptance rendered-row test; report/office paths. | Backend `n_classified=0` alone is supporting, not the client-visible contract. |
| `056` | Failed pay computation leaves named section in every emitted PDF and batch record. | `report::tests::a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record`. | A helper note page or internal Result is insufficient. |
| `057` | Nested IPC is snake_case and unknown fields fail. | `netflag` serde structs and three wire tests. | Confirm both positive exact payload and negative unknown/case-drift payload. |
| `058` | Sweep more than one cutoff at a time. | `CutoffSweepRequest.property` and one-dimensional result. | A UI with several fixed fields remains a one-axis sweep. |
| `059` | Inverse solve from target. | Negative source/UI/history inventory. | A manually inspected elbow is not an inverse solver. |
| `060` | Imported parameters addressed by block, ordinal, and semantic key. | Negative importer inventory; E-6 boundary. | Never transcribe the protected 220-row map or silently remap an ordinal. |
| `061` | Display precision validated against field width. | report/office formatting types and validators. | `toFixed`/format strings alone are not pair validation. |

## Acceptance-Test Ownership Map

Every ID below is a primary proof target exactly once. During execution, record the exact existing test or `MISSING`; do not invent a substitute oracle.

| Test | Primary contract and expected-value source |
|---|---|
| `T01` | CENTRED fixture gives exact 3.25 ft; published worked fixture plus independent hand trace. |
| `T02` | TOPS-with-clip gives Net 3.0, Gross 4.0, Unknown 0.0; independent interval arithmetic. |
| `T02b` | Gross stays 4.0 under all three models; exact partition invariant. |
| `T03` | Interior single sample gives 0.1524 m under CENTRED and TOPS; dossier consequence. |
| `T03b` | Bottom-boundary sample differs: centred half-step, TOPS zero booked Unknown; dossier and cited source. |
| `T03c` | Between-sample boundary includes clipped partial interval; exact zone partition. |
| `T04` | Geometric mean is unit invariant and differs from IP's unit-dependent form; independent formula. |
| `T05` | Power means at `p=1,-1,0,1/3`; independently derived closed forms. |
| `T06` | Two phi-weighted Sw forms agree; algebraic identity. |
| `T07` | Direct and reconstructed HCPV agree only with phi-weighted Sw; independent identity plus negative control. |
| `T08` | Cited two-well roll-up gives Phi 0.187, Sw 0.263, PhiH 13.05, PhiSoH 9.623, Net 70 ft. |
| `T09` | Zero and negative values are excluded from geometric/harmonic means and counted; cited defect case. |
| `T10` | Passing sample outside every zone contributes nowhere; cited zone-membership rule. |
| `T11` | Thirty-percent null fixture satisfies four-way partition and distinguishes both N:G ratios. |
| `T12` | Same seed bit-identical, different seed different; cited determinism guarantee. |
| `T13` | IP `m` width 0.2 with `SD_MULT=2` yields sigma 0.10, not 0.20; cited IP convention. |
| `T14` | Gaussian draws stay within +/-2.5 sigma and variance deficit is reported; cited truncation rule. |
| `T15` | Percent shift at zero falls back rather than point mass and reaches accumulator unclamped; cited guard plus R-4. |
| `T16` | Marginal-product P50 differs from joint P50 case; method and direction are recorded; dossier and vendor precedent. |
| `T17` | Ordinal/semantic mismatch is a load error; CONTRACT addressing rule. |
| `T18` | Parameter draws per section, cutoff draws per zone, realised correlation reported; cited draw regimes. |
| `T19` | `P10(Net)/P10(Gross)` differs from `P10(Net/Gross)` on varying gross; independent arithmetic. |
| `T20` | Reciprocal shift asymmetry at Rt 1 and 100; independently derived `1/(1/R +/- s)`. |
| `T21` | Same ordinal resolves differently by block and bare ordinal refuses; cited per-block namespace. |
| `T22` | Reconciliation absorbs <=1e-7 relative residual and records it; larger residual refuses. |
| `T23` | Uniform relative shift at zero avoids point mass and accumulator clamp; cited missing guard plus R-4. |
| `T24` | Endpoint inclusion follows each documented bounds operator; SandiBumi specification is the oracle. |
| `T25` | Unclamped Sw mean remains 1.0 and pre-accumulation clamp biases by `-sigma/sqrt(2*pi)`; independent derivation. |
| `T26` | `35` refuses without unit, `35 pu` becomes 0.35 v/v, `35 v/v` refuses; cited unit trap. |
| `T27` | Vertical spread independent of N; horizontal shrinks by `1/sqrt(N)` and ratio is about 10 for N=100. |
| `T28` | Triangular `(1,2,6)` reports mode 2, mean 3, median 2.84 and P50 differs from base; cited values. |
| `T29` | P10 from five iterations is withheld with machine-readable reason; cited refusal precedent, not value adoption. |
| `T30` | Rec import into unsupported sampler is a load error, never Linear coercion. |
| `T31` | Bed amalgamation preserves total net, reduces interval count, raises thinnest bed, records both blocks/thresholds/step. |
| `T32` | Worked AND/OR expression evaluates and AND-only net is strictly smaller; cited worked rule. |
| `T33` | Clear fixture converges by 750, marginal fixture does not; auto-stop watches reported percentile and records count. |
| `T34` | Absolute tornado min/max stable at 750 versus 5000; percentage ranges differ and carry iteration count. |
| `T35` | CHARACTERIZATION inventory: every current cutoff-bearing pane resolves one authority; live pane list is re-enumerated. |
| `T36` | Fresh project has no cutoff defaults; enabled-unset refuses; intentional unfiltered run says so. |
| `T37` | Rendered uninterpreted row is absent while classified zero is numeric; existing observable regression target. |
| `T37b` | Emitted PDFs retain failed Pay Summary section and batch reports every degradation; existing observable regression target. |
| `T37c` | Failed MC chain produces failed job item and no clean uncertainty result; existing observable regression target. |
| `T38` | Exact snake_case nested payload works; unknown/case-drift fields refuse; existing positive and negative wire targets. |
| `T39` | Precision exceeding width refuses before render; requirement-defined formatting contract. |

## Parameter, Open-Question, Escalation, and Refusal Custody

### Parameter rows

- Rows 1-8 remain ABSENT: porosity cutoff, Sw cutoff, Vsh cutoff, permeability cutoff, new-prior distribution, minimum iterations for a tail percentile, `Rw` prior, and general per-parameter prior.
- Rows 9-33 are specification/decision/engineering rows: bounds operator; three bed thresholds; depth model; power exponent; mandatory seed; 2000 iterations; four auto-stop controls; `SD_MULT=2`; truncation 2.5; tornado size/units/cap; VERTICAL regime; no pre-accumulation clamp; explicit centring; listing percentiles; per-quantity direction; type-7 interpolation; and newly-added perturbation off. Verify live custody field by field; do not treat a source comment as a runtime field.
- Rows 34-44 remain NON-ADOPTABLE verification data: IP `m`, `n`, `a`, `Rw`, matrix density, neutron clay, GR endpoints, unknown-unit `B`, seven input-curve priors, Geolog percentage-error priors, and IP cutoff-block widths. These rows may verify an importer or conversion but MUST NOT become SandiBumi defaults.

### Opens and escalations

- Preserve O-1's narrow Techlog-script claim, O-2's bed-algorithm block, O-3's MD/TVD question, O-4's asymmetric Gaussian centring, O-5's Sw direction, O-6's dual Net reconciliation, O-7's iteration ceiling, O-8's default distribution conflict, O-9's measurement-correlation question, O-10's Rec behavior near zero, O-11's cross-well roll-up, and O-12's raster-only HCPF equation.
- Preserve E-1 as a resolved exposure ruling but open implementation/test obligation; E-2 as already adjudicated by the core regression-lock increment; E-3 as unknown `B` unit; E-4 as no house `Rw` prior; E-5 as no cutoff-selection guidance until sources are ingested; and E-6 as an unresolved CONTRACT boundary.

### Refusals

- Record R-1 through R-13 on every affected row: no cutoff defaults, no IP geometric formula, no arbitrary caps, no pre-accumulation clamp, no type-string bounds, no per-depth draws for summation inputs, no unsupported statistics, no ratio-of-percentiles, no Rec coercion, no non-PD correlation approximation, no invented B unit, no non-reconciling fixture, and no HCPV-to-volume conversion in CUT.

## Execution Tasks

### Task 1: Refreeze the baseline

- [ ] Re-run branch, worktree, hash, anchor, ledger, source-column, test-ID, parameter, open/escalation/refusal, manual-evidence, and candidate-test checks.
- [ ] Stop and reconcile if the branch, anchor reachability, source hashes, row counts, or source-owned fields differ from this plan.

### Task 2: Reverify deterministic summation and cutoff custody (`001`-`030`)

- [ ] Trace each contract through request type, algorithm, returned row, UI rendering, report/office export, history, and manual matrix.
- [ ] Enumerate the current cutoff surfaces rather than copying the chapter's historical six-pane list. At plan freeze there are eight shared-loader consumers (`cutoff`, `summary`, `Monte Carlo`, `report`, `Results QC`, `workbook`, `deck`, plus the loader itself as authority) and one direct-literal dashboard bypass; confirm exact production surfaces during execution.
- [ ] Keep all cutoff values absent as authority and classify the live constants only as as-built evidence.

### Task 3: Reverify Monte Carlo contracts (`031`-`053`)

- [ ] Trace prior creation, distribution representation, draw construction, correlation, convergence, accumulation, percentile calculation, sensitivity/tornado, persistence, returned result, job status, and rendered UI.
- [ ] Quantify the direct-width versus `SD_MULT=2` mismatch from the cited IP `m` row without inventing any new prior.
- [ ] Check MC and deterministic pay separately where missing permeability, PHIE flooring, zone clipping, and limited/unlimited curves can diverge.
- [ ] Treat type-7 arithmetic, correlation induction, and convergence as supporting until the required result fields and independent fixtures are demonstrated.

### Task 4: Reverify degraded-result, IPC, import, and formatting contracts (`054`-`061`)

- [ ] Confirm the three observable SB-CORE-002 locks on the current tree and record their exact command/test names.
- [ ] Confirm both sides of the snake_case/unknown-field wire contract.
- [ ] Confirm the multi-cutoff, inverse-solve, structured-import, and precision/width inventories negatively across expected source, UI, tests, and history.

### Task 5: Write the evidence receipt and update all 61 ledger rows

- [ ] Create `docs/takeover/evidence/sb-cut.md` with one complete section per requirement in numeric order.
- [ ] Update only live adjudication columns in `docs/takeover/requirements.csv`.
- [ ] Recompute the ledger and update `docs/takeover/STATUS.md` with exact as-built, disposition, risk, proof, manual, and remaining-row counts.
- [ ] Verify the six source-owned columns remain byte-identical to the frozen hash.

### Task 6: Run structural and evidence checks

- [ ] Assert 61 receipt sections and 61 matching ledger rows, each legal and non-placeholder.
- [ ] Assert all 44 test intentions are routed exactly once and all 44 parameters, 12 opens, 6 escalations, and 13 refusals are accounted for.
- [ ] Search new receipt/ledger text for prohibited identifying names, invented defaults, unresolved-unit fills, and accidental claims of field evidence.
- [ ] Run `node --test tools/takeover-ledger.test.mjs`, `node tools/takeover-ledger.mjs --summary-json`, and `git diff --check`.

### Task 7: Run the repository gates and commit the execution increment

- [ ] Run `npx tsc --noEmit` from the repository root.
- [ ] Run `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from the repository root. If the live Tauri application owns the default debug executable, use the already-established isolated `CARGO_TARGET_DIR`; do not stop the live application.
- [ ] Record fresh passed/failed/ignored totals; do not reuse the prior `946/0/36` result.
- [ ] Stage exactly `docs/takeover/evidence/sb-cut.md`, `docs/takeover/requirements.csv`, and `docs/takeover/STATUS.md`; inspect the cached diff and source-column hash.
- [ ] Commit locally with message `G1-DOM-CUT adjudicate 61 SB-CUT requirements`. Do not push, merge, or open a pull request.

### Task 8: Continue Gate 1 serially

- [ ] Recompute the remaining domain inventory from the post-CUT ledger, keep all 52 SB-GEO rows deferred to the next product version, and choose the next dependency-relevant open-hole-petrophysics domain.
- [ ] Create the next serial planning branch in the same `D:\XX. SandiBumi` worktree without touching any other folder.
- [ ] Do not begin Gate 2 production remediation until Gate 1 evidence reconciliation is complete or Jauhar explicitly changes the sequence.

## Plan Self-Review Receipt

- **Spec coverage:** all 61 requirements appear exactly once in the Requirement Evidence Map; all 44 tests appear exactly once in the Acceptance-Test Ownership Map; all 44 parameters, 12 opens, 6 escalations, and 13 refusals have explicit custody.
- **No placeholders:** this plan contains no `TBD`, `TODO`, guessed source, unspecific error-handling instruction, or cross-task shorthand.
- **Type/interface consistency:** the plan uses the live names `PaySummaryRequest`, `PaySummaryRow`, `CutoffSweepRequest`, `Cutoffs`, `McParam`, `McRequest`, `McResult`, `McZoneResult`, `McConvergence`, `McPlausibility`, `MetricSet`, `NetFlagSpec`, and the exact current Rust/TypeScript paths. It invents no production API.
- **Source discipline:** no cutoff, prior, unit, threshold, tie-break, distribution, or incumbent behavior is adopted beyond the immutable chapter. The eight ABSENT rows, eleven NON-ADOPTABLE rows, twelve opens, and unresolved escalations remain fenced.
- **Evidence discipline:** passing tests are candidates only until observable scope and independent expected values are checked. Manual cutoff/pay evidence remains `0/23`; Monte Carlo remains `2/14`.
- **Scope discipline:** planning changes only this plan and STATUS; execution changes only the CUT receipt, ledger, and STATUS. Production remediation remains a later Gate 2 activity.

## Execution Handoff

Execute inline in the current session with `superpowers:executing-plans`. Jauhar's persistent instruction is to continue Gate 1 serially, so no additional approval is required unless execution reaches an unresolved product-owner decision or source gap that would otherwise be silently guessed.
