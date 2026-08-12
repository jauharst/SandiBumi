# SB-SHR Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 42 live `SB-SHR` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 44 acceptance-test intentions, and preserve every unresolved source, unit, convention, fluid-property, fitted-object, and model-selection question without changing saturation-height, capillary-pressure, rock-typing, reporting, import, or cutoff behavior.

**Architecture:** This is a documentation-only evidence pass over the saturation-height module, pooled and per-rock-type fitting paths, SCAL and Thomeer fits, flow-unit clustering and Lorenz segmentation, rock-typing modules, unit carrier, report/export surfaces, IPC types, manual matrix, and reachable Git history. The immutable PRD supplies the intended contract, original priority, chapter status, 61 parameter rows, open observations, escalations, refusals, acquisition gaps, and named test intentions. Current source, independently sourced or derived tests, manual evidence, and reachable history supply the separate live verdict recorded in the takeover ledger and one SB-SHR evidence receipt. A dialog-local fit, an internal numeric helper, a self-generated round trip, a source comment, or a correct branch beside an incorrect branch does not prove the whole observable contract.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, test, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn. Parameter custody, petrophysical interpretation, data-integrity judgment, and final sign-off remain with the primary session.
- Work only in `D:\XX. SandiBumi`. The sole registered Git worktree MUST remain that path. `D:\XX. SandiBumi-check` is not a repository or evidence source and MUST remain untouched.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `f4e328f0bbe45d0f73a985591071f0d816a0adeb`; fetched `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution and stop to reconcile if any moves.
- The local branch is `codex/g1-sb-shr-adjudication`. The serial Gate 1 chain remains intentionally local and unpushed; do not switch, merge, rebase, rewrite history, push, or open a pull request.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential negative result MUST be confirmed in the expected Rust and TypeScript files, exact tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/15_sat-height-rocktyping.md`, applicable sections of `docs/record_calibration.md`, `docs/record_fixes.md`, `docs/record_data_tools.md`, `docs/record_petrography.md`, and `docs/record_parallel_lanes.md`, the current manual matrix, and existing takeover receipts/status.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 42 ordered rows is `f19ac95bc0e09df63e7c3a1f617b2d0bd418d0d4cef4d444a3e6938003728433`, calculated as PowerShell `Import-Csv`, ordered `Select-Object` of those six fields, `ConvertTo-Csv -NoTypeInformation`, LF join, and UTF-8 without BOM. The immutable chapter SHA-256 is `4567929c51a96d63222eb5c59aa4e1beb3de2fcc27a98ceecf4b094056bba29b`.
- The chapter and ledger agree on 42 contiguous requirements: `P0=13`, `P1=23`, `P2=6`. Chapter states are `ABSENT=25`, `PARTIAL=6`, and `PRESENT-DIVERGENT=11`. Reverify live behavior independently; do not copy these historical states into the live verdict.
- The chapter defines exactly 44 contiguous acceptance-test IDs, `SB-SHR-T01` through `SB-SHR-T44`, with no suffixes and no omissions. Route every intention exactly once as a primary proof target. `T17` jointly serves requirements `014` and `024` but remains one routed test intention.
- Section 5 contains exactly 61 parameter rows: 14 height/pressure rows, 9 interfacial-tension/contact-angle rows, 12 model-coefficient rows, 14 rock-typing rows, 5 capillary-pressure-permeability rows, and 7 partitioning rows. Nine rows contain `ABSENT`, six contain `NON-ADOPTABLE`, fourteen contain `UNSOURCED`, and one contains `PRESENT-UNVERIFIED`. These are source-custody findings, not mutually exclusive as-built states.
- A petrophysical value is cited or absent. Never adopt the live `J_CONST=0.21645`, gradient `0.433`, forward `RHO_HC=0.8`, `FWL=2000`, `IFT_RES=26`, fused `HG_AIR_IFT=367`, the seven forward model placeholders, the four cutoff boundaries, the Swanson coefficient pair, a remembered textbook value, or a plausible engineering number as authority.
- Preserve the three section 7.1 contract/core observations, all five `ESC-SHR-1` through `ESC-SHR-5` escalations, all twelve defect refusals `(a)` through `(l)`, both independent-derivation requirements, all eight ranked acquisition gaps, all six coverage gaps, and all four uncertainty notes. A source gap is an evidence finding, not permission to select a number.
- `ESC-SHR-1` keeps the four fluid-system defaults absent until a primary source or explicit contract ruling exists. `ESC-SHR-2` keeps Swanson blocked without a cited apex basis. `ESC-SHR-3` leaves the 0.7/0.8 hydrocarbon-density house choice unresolved. `ESC-SHR-4` keeps Lucia flagged until the original paper is acquired. `ESC-SHR-5` preserves the P0 FWL-uncertainty requirement while recording that its priority remains a product-owner call.
- The current direct-domain candidate suite passes `58/0/0`: satheight `7`, SHF fit `10`, Thomeer `4`, HFU `8`, Lorenz `10`, rock typing `13`, and units `6`. These are candidates, not 42 automatic contract closures. There is no executable source/test use of an `SB-SHR-Tnn` identifier outside the immutable PRD, ledger tooling fixture, backlog/tracker text, and this plan.
- The historical PRD statement that Skelt branch parity “must fail today” is stale against the current branch: `satheight::tests::skelt_harrison_is_identical_whichever_unit_the_project_declares` and `saturation_height_is_identical_whichever_unit_the_project_declares` both pass, with reachable history at `bb807ca`. Record the live closure if the observable family/unit contract and regression oracle qualify; do not edit the PRD during Gate 1.
- `as_built_status` answers what the accepted tree currently ships. `release_disposition` answers whether the observable contract is required for the paid offline Windows open-hole-petrophysics pilot. Priority and chapter status inform but do not mechanically decide either field.
- A compound requirement is not `PRESENT-OK` because one model, fitted family, dialog, result field, import path, report, or regression test is correct. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, a list, a provenance/export clause, or a refusal/control pair.
- A test is qualifying owned acceptance proof only if it exercises the observable contract and derives the expected value independently from a cited source or explicit arithmetic. An internal `Result`, source-text grep, compile success, fit-and-forward path sharing the same wrong literal, synthetic data produced by the same equation under test, or current-library snapshot is supporting evidence only.
- Classify each requirement's proof as `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING` under CONTRACT sections 3 and 6. A test pinning current divergent behavior is divergence evidence, never proof of the specified behavior.
- Manual evidence remains separate: `saturation-height=0/6`, `rock-typing=0/26`, `saturation=2/97`, `report=6/53`, `workflow=0/23`, and `processing-history=0/7`. Automated or desktop-harness evidence cannot close an unchecked manual scenario.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record all of the following:

1. branch `codex/g1-sb-shr-adjudication`;
2. a clean worktree and the sole registered worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, fetched `origin/master`, merge base, and accepted-anchor reachability;
4. exactly 42 ledger rows, covering `SB-SHR-001` through `SB-SHR-042` once with no gap or duplicate;
5. priority counts `P0=13`, `P1=23`, `P2=6`;
6. source-status counts `ABSENT=25`, `PARTIAL=6`, `PRESENT-DIVERGENT=11`;
7. all 42 live `as_built_status=UNADJUDICATED`, `release_disposition=UNDECIDED`, `risk_class=UNCLASSIFIED`, and `commit_state=UNVERIFIED`;
8. exactly 44 defined test IDs and the same 44 unique IDs referenced by the ledger, with no defined-but-unowned or owned-but-undefined ID;
9. exactly 61 parameter rows, including the nine ABSENT, six NON-ADOPTABLE, fourteen UNSOURCED, and one PRESENT-UNVERIFIED rows described above;
10. exactly three contract/core observations, five escalations, twelve refusals, two independent-derivation requirements, eight acquisition gaps, six coverage gaps, and four uncertainty notes;
11. takeover summary `499` adjudicated, `432` unadjudicated, and `368` pilot blockers before any SHR edit;
12. the manual capability counts listed in Global Constraints; and
13. the fresh `58/0/0` direct-domain candidate-test receipt listed in Global Constraints.

The only mechanically predictable post-adjudication ledger count is `541` adjudicated and `390` unadjudicated. Do not predict as-built, pilot-blocker, test-class, or manual-evidence totals before row-by-row classification.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-shr.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/satheight.rs`, `src-tauri/src/shf_fit.rs`, `src-tauri/src/thomeer.rs`, `src-tauri/src/hfu.rs`, `src-tauri/src/lorenz.rs`, `src-tauri/src/rocktyping.rs`, `src-tauri/src/units.rs`, `src-tauri/src/ingest.rs`, `src-tauri/src/parsers.rs`, `src-tauri/src/contacts.rs`, `src-tauri/src/distribution.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/workflow.rs`, `src-tauri/src/report.rs`, `src-tauri/src/office.rs`, `src-tauri/src/export.rs`, `src-tauri/src/lib.rs`, `src/ipc.ts`, `src/ui/shfDialog.ts`, `src/ui/thomeerDialog.ts`, `src/ui/hfuDialog.ts`, `src/ui/lorenzDialog.ts`, `src/ui/crossplotPanel.ts`, `src/ui/reportDialog.ts`, current tests, reachable Git history, manual evidence, and the immutable source chapter
- Read-only negative-store inventory may inspect `src-tauri/src/db.rs`, but neither planning nor adjudication may modify it or infer a persistence design from its absence.
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected vendor material, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-SHR-NNN` section per requirement in numeric order. Every section MUST include:

- **Specified contract:** every observable limb, separated when compound.
- **Current implementation:** exact symbols and paths, plus explicit negative-inventory scope where absent.
- **As-built status:** one legal ledger state and why a stricter state fails.
- **Release disposition and risk:** pilot relevance independent of implementation status.
- **Automated evidence:** exact test names and commands classified `CORRECTNESS`, `CHARACTERIZATION`, `SUPPORTING-ONLY`, or `MISSING`, with the independent expected-value source named.
- **Manual evidence:** exact checked capability/scenario or `NONE`; never inferred from automation.
- **Source/parameter boundary:** cited, absent, unsourced, non-adoptable, present-unverified, derived, engineering guard, conflicting, or not applicable, with escalation/refusal/acquisition IDs.
- **UI/IPC/provenance surface:** request field, dialog control, fitted object, result record, computed curve, log-set record, Results QC, report, office export, generic export, and batch/job surface as applicable.
- **History/reachability:** accepted commit evidence or confirmed negative history search where consequential.
- **Blocking decision/dependency:** exact source, legal, product, UI, manual, or implementation dependency.
- **Next action:** the smallest bounded follow-up, or `NONE` only when the whole contract is proved and no field evidence is required.

## Requirement Evidence Map

### Group A - Units, constants, and fluid-property custody (`001`-`008`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `001` | Every shipped height family converts project-unit height into its coefficient unit; a new family cannot omit the declaration. | `satheight::sw_height`; `units::to_metres`; family selector; the two passing parity tests; `shf_fit` family routes. | Passing Leverett alone is insufficient; confirm Skelt, every fit family, and the new-family control rather than copying the chapter's stale defect claim. |
| `002` | `B`, `D`, `Hd`, `He`, and FWL carry explicit units and re-express when project depth unit changes. | `ModuleSpec` parameter units; `units::set_project_depth_unit`; SHF request types and dialogs. | Refusing a project-unit change while curves exist is not coefficient re-expression; a hard-coded `m` label is evidence of absence. |
| `003` | Every domain entry point refuses an undeclared project depth unit by name and computes nothing. | `project_depth_unit_or_default`; module runner; fit commands; Thomeer/HFU/Lorenz entry points. | A parse-time unit default or fallback is not a result-level domain refusal. |
| `004` | All height outputs, prompts, plot axes, and export headers use the declared project unit. | `sw_height_spec`; `HAFWL`; `shfDialog`; IPC result labels; LAS family-unit table and export. | A numerically converted value under a stale label remains divergent; inspect all four surfaces. |
| `005` | One shared default object for water density, hydrocarbon density, and reservoir fluid system across fit/apply; disagreement refuses. | `satheight` parameter spec; `shf_fit::d_rho_w`, `d_rho_hc`, `d_ift_res`; forward round trip. | Do not select 0.7 or 0.8 during Gate 1; the shared-object and refusal limbs both matter. |
| `006` | Leverett J constant and gradient are first-principles evaluable derivations, single-sited, source-bearing. | `satheight::J_CONST`; `PSI_PER_FT_PER_SG`; module docs and tests. | Numerically close literals and copied source comments do not satisfy evaluable derivation or machine-readable source custody. |
| `007` | `J_C`, gradient, `RQI_C`, and `PERM_C` each have one literal site across code, UI, and fixtures. | `satheight`, `hfu`, `rocktyping`, `shfDialog`, `ingest` fixture, repository search. | The fixture must derive its oracle independently; a duplicate with the same value is still divergent. |
| `008` | Interfacial tension and contact angle remain separate for every lab/reservoir system; no fused constant ships. | `thomeer::HG_AIR_IFT`; `scal_pc` system/IFT fields; import dialog; pore-radius standardisation. | Correct use of an explicit ratio cannot erase fused storage or absent contact-angle custody. |

### Group B - Fitted objects, conventions, validity, and honesty (`009`-`025`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `009` | Pooled/per-rock-type SHF, Thomeer, HFU, and Lorenz fits persist as named versioned objects with complete training provenance. | Four result structs and dialogs; document/store tables; exclusion ledgers; project reopen path. | Returned JSON or a canvas-local result is not persistence; verify all four fitted-object classes. |
| `010` | Forward apply consumes a stored fit; overrides record old and new values. | `sw_height` parameter path; SHF fit result; module dialogs; zone-parameter provenance. | Copying coefficients by hand or from UI text is the prohibited workflow, even when the arithmetic matches. |
| `011` | FWL is uncertain first-class custody: mandatory scan, interval, per-zone confidence, and unconstrained refusal. | `foil_fwl_scan`; `CuddyFoilResult`; `shfDialog`; fit statistics. | An argmin and residual curve without an interval/confidence is partial at most. |
| `012` | Brooks-Corey emits labelled lambda and reciprocal N; undeclared convention import/export refuses. | `fit_brooks_corey`; `ShfFitResult.params`; IPC/dialog/import/export inventory. | Correct published lambda arithmetic alone does not prove dual-labelled custody or refusal. |
| `013` | Thomeer emits labelled base-10 and natural-log G; undeclared-base import refuses. | `thomeer_bv`; SHF Thomeer fit; result structs/dialog/import inventory. | Base-10 arithmetic is a competitive win but does not satisfy both forms and import refusal. |
| `014` | Swanson has no apex-basis default; selected basis and sourced pair travel with every result; unselected is missing. | `thomeer::run_thomeer_fit`; `swanson_k`; Thomeer dialog and generic export. | Never legitimise the live 399/1.691 pair or infer its basis; `ESC-SHR-2` remains blocking. |
| `015` | Both published port-size schemes selectable, no silent default, active scheme recorded. | `rocktyping::PORT_BOUNDS`; `port_class`; module spec/results. | One fixed array and a correct class are not a selectable recorded scheme. |
| `016` | Amaefule RQI and saturation-family RQI are quantity-disambiguated and cross-consumption refuses. | `rocktyping` outputs; curve metadata/family resolution; module inputs. | Different formulas behind the same mnemonic are not safe because current fixtures happen to use the intended one. |
| `017` | Every `log phi` correlation enforces declared unit/range, fails the run, and names the curve. | Pittman/Winland/Lucia/other correlations; workflow runner; module notes/errors. | `continue` to `f32::NAN`, comments, or one correlation's guard is not the named whole-run refusal. |
| `018` | Conflicting incumbent gradients are shown as height consequences at the choice surface. | Negative chooser/UI inventory; fluid-gradient arithmetic. | A list of gradient values or a PRD table is not a product choice surface. |
| `019` | Sigma-scoped coefficients remain NON-ADOPTABLE, unreachable as defaults/seeds, and display their scope if shown. | parameter sources; module defaults; fit seeds; dialogs. | Absence from one module is not repository-wide unreachability. |
| `020` | Lucia RFN enforces both limits; RFN value and class carry the same out-of-band state. | `lucia_rfn`; `lucia_class`; `RFN` and `RFN_CLASS` outputs. | Preserve the fraction-porosity correctness control while checking the missing lower bound and dual-output agreement. |
| `021` | Published extrapolation regimes carry per-sample flags and no source-unpublished clamp. | Pittman table, radius outputs, crossover regressions, module metadata. | The non-monotone values are correct; missing flags are a separate obligation and must not be “fixed” by clamping. |
| `022` | Every registered module returns and persists named exclusion reasons/counts; reduced coverage explains why. | `shf_fit` exclusion vector; module output/result runner; computed-curve provenance. | The working fit ledger is the control, not proof that module runs have the same observable custody. |
| `023` | `rt_cutoff` distinguishes unclassified from lowest class; every boundary cited or absent/user-entered. | `rt_cutoff_spec`; `rt_cutoff`; four live cutoff literals; output classes. | A class number cannot encode both states; do not adopt the four current boundaries. |
| `024` | Unverified constants produce a result-level flag through dialog, plot, export, and report. | Lucia/Swanson source comments; result types; all consumer surfaces. | A comment or dialog tooltip is not transitive result custody. |
| `025` | Deliverable report names actual SHF/rock-typing methods, parameters, and FWL and omits unused fixed methodology. | `report::default_methodology`; editable report rows; PDF and Word paths. | Generic curve export and user-editable text do not prove truthful automatic methodology. |

### Group C - SCAL corrections, pore geometry, and flow-unit completeness (`026`-`033`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `026` | Lab-to-reservoir Pc conversion uses separately declared sigma and absolute-cosine angle for both systems and refuses undeclared systems. | SCAL row fields; Thomeer standardisation; import UI; negative service inventory. | The current fused IFT ratio is structurally insufficient and its defaults remain absent. |
| `027` | Shift, proportional, crop, and extrapolate corrections are explicit no-default choices carried into curve and derived parameters. | SCAL import/conditioning inventory; Thomeer fit/result. | Do not borrow a vendor default or infer a treatment from changed numbers. |
| `028` | Net-stress and clay-bound-water corrections operate on non-wetting phase and record identity. | SCAL correction inventory; phase representation. | An absent correction cannot be credited as a refusal; preserve the open salinity-unit acquisition gap. |
| `029` | Pore-throat modality is reported and paired with the log-side Buckles diagnostic. | `shf_fit::buckles_note`; throat-distribution/SCAL inventory. | A Buckles note alone is partial; no mode count may be invented from one plotted curve. |
| `030` | Lorenz-gradient inflection partitioning selectable beside exact Ward; method recorded. | `lorenz::compute_lorenz`; shared Ward DP; result/dialog fields; history `bb1e4d9`. | Ward over depth order is not gradient-inflection partitioning even when it recovers the same synthetic boundaries. |
| `031` | Dykstra-Parsons VDP is returned alongside Lorenz coefficient. | `LorenzResult`; dialog/report; negative symbol/history inventory. | Do not infer percentiles or interpolation details beyond the published definition. |
| `032` | Purcell and Katz-Thompson routes available alongside Swanson with separate sources and side-by-side results. | Thomeer result/dialog; parameter source inventory. | Primary sources are acquisition gaps; no formula or constant is supplied from memory. |
| `033` | Winland R35, Aguilera R35, and PGS are separately named indicators with no substitution. | `rocktyping` outputs `R35`, `PGEOM`, `PSTRUC`; module docs/crossplot overlay. | Existing Winland and PGS outputs do not prove Aguilera exists or that quantity metadata prevents substitution. |

### Group D - Cross-domain seams, reference conditions, model selection, and reproducibility (`034`-`042`)

| ID | Exact contract focus | Primary live candidates | Adjudication guard |
|---|---|---|---|
| `034` | One fluid-gradient service supplies chapter 14, uses derived constant and stored-fit densities, and refuses undeclared unit. | CUT HCPV inventory; `satheight` constants; stored-fit inventory. | Local arithmetic in either domain is not the shared service. |
| `035` | Candidate cutoffs derived from rock partition, flow boundaries, and Pc evidence retain that evidence and are never selected here. | rock-type/flow/Pc outputs; cutoff source types; CUT handoff inventory. | Existing indicators are inputs, not evidence-bearing derived cutoff objects. |
| `036` | Lambda family remains absent and is never approximated until sources exist. | SHF method enum and dialog options; source/history inventory. | “Not implemented” may satisfy the refusal limb but not the future capability; classify the compound contract precisely. |
| `037` | Swirr reference condition is typed as pressure plus lab/reservoir state; undeclared refuses. | Swirr parameters/results; SCAL pressure metadata; import and fit types. | Free text, a naked pressure, or implicit lab context is insufficient. |
| `038` | Corrected Pc import requires correction provenance and resolves declared module identity. | `import_scal_files`; parser records; SCAL schema/UI. | Fluid system and IFT are not correction provenance; preserve accepted uncorrected imports as the control. |
| `039` | Model-selection sweep is exhaustive, uncapped in depth, deterministic, provenance-bearing, and retains full ranking. | negative runner/result/UI/history inventory; independent-derivation section 7.4. | A family dropdown or one fit at a time is not a sweep; do not reconstruct any vendor implementation. |
| `040` | Closure, entry pressure, and rock class are bit-identical regardless of plot state or axis scale. | numerical routines versus dialog/canvas state; negative closure inventory. | Absence of closure capability is not evidence that its future result is display-independent. |
| `041` | Published prediction direction is preserved and recorded; no algebraic inversion is presented as a regression. | Pittman forward radius functions, permeability outputs, result metadata. | Correct forward `R=f(k,phi)` code is only partial if inverse prediction and direction custody are absent. |
| `042` | One contact-angle range and representation; every Pc expression uses absolute cosine; stored angle/cosine cannot disagree. | fused-fluid inputs; SCAL/Thomeer calculations; negative angle-type inventory. | No angle field means no single-valued convention; do not infer a default range. |

## Acceptance-Test Ownership Map

| Test | Independent oracle and required observable scope |
|---|---|
| `T01` | Metre/foot descriptions of one physical column agree for every family; exact conversion follows the declared project unit and includes a new-family control. |
| `T02` | Static family inventory proves no raw height reaches model arithmetic without unit conversion. |
| `T03` | `B=30 m` re-expresses as `98.4252 ft` and SWH is invariant; exact international-foot conversion. |
| `T04` | Every domain entry point returns a named undeclared-unit refusal and no result. |
| `T05` | Foot-declared HAFWL curve, FWL prompt, plot axis, and LAS header all say feet. |
| `T06` | Shared fluid object identity plus fit/apply round trip within 0.5 saturation units; disagreement must refuse. |
| `T07` | J and gradient equal the chapter's independently shown first-principles expressions and two gradient routes agree below `1e-6`. |
| `T08` | Repository-wide literal inventory plus fixture derived independently rather than from the guarded literal. |
| `T09` | No fused sigma-cosine constant; changing stored angle changes every derived radius. |
| `T10` | Fit survives dialog close and project reopen with identical coefficients. |
| `T11` | Every persisted fit carries the complete enumerated training provenance and incomplete objects cannot apply. |
| `T12` | Forward apply accepts only stored fit; override records replaced and replacement values. |
| `T13` | Flat residual produces wide FWL interval and sharp residual produces narrow interval; both state confidence. |
| `T14` | Brooks-Corey emits labelled lambda and reciprocal N, refuses undeclared convention, and pins the cited 26-saturation-unit stake. |
| `T15` | Thomeer emits both labelled G forms with `G_ln=ln(10)*G_log10` and refuses undeclared base. |
| `T16` | No apex basis yields missing Swanson permeability with named reason. |
| `T17` | Selected basis and unverified state reach result, dialog, export, and report; cited three-basis values are `0.00864`, `0.000193`, and `1.97 mD`. |
| `T18` | Both port schemes selectable; `2.2 um` changes class; chosen scheme travels. |
| `T19` | RQI quantity collision refuses independent of mnemonic and fixture pins cited `11.8x` Swirr stake. |
| `T20` | Every log-porosity correlation fails by curve name on unit mismatch and pins `676.08x`, `234.42x`, and `1.3183e6x` traps. |
| `T21` | Choice surface reports `27.9`, `54.6`, and `77.0 ft` consequences at the chapter's fixed Pc. |
| `T22` | No NON-ADOPTABLE coefficient is reachable as default or seed. |
| `T23` | RFN `0.2` is out-of-band on both outputs; `phi=0.05 v/v` retains the independent RFN `2.07`, not `7.41`, control. |
| `T24` | Published tight-rock values remain `0.77`, `0.86`, `1.11 um`, carry extrapolation flags, and are not clamped. |
| `T25` | Every registered module reconciles named exclusion counts to missing outputs; five working SHF-fit reasons remain populated. |
| `T26` | A sample meeting no cutoff is distinct from class 3 and each boundary is cited or requires user entry. |
| `T27` | Rendered report names actual SWH family, coefficients, and FWL and omits an unused electrical-method row. |
| `T28` | Lab/reservoir conversion round-trips the published ratio and refuses either undeclared system. |
| `T29` | Four correction treatments give distinct entry pressures, no default applies, and choice travels into Thomeer results. |
| `T30` | Stress and clay-bound-water corrections structurally target non-wetting phase and record identity. |
| `T31` | Synthetic bimodal throat distribution reports two modes and more than one law, alongside Buckles diagnostic when available. |
| `T32` | Three known Lorenz-gradient regimes recover true boundaries; inflection and Ward are selectable, recorded, and may disagree. |
| `T33` | Published VDP is returned beside Lorenz coefficient on a layered synthetic. |
| `T34` | Purcell and Katz-Thompson appear beside Swanson, each separately sourced and presented. |
| `T35` | Winland R35, Aguilera R35, and PGS are distinct typed outputs with no substitution route. |
| `T36` | Chapter 14 calls the shared derived-gradient service, uses stored-fit densities, and refuses undeclared unit. |
| `T37` | Candidate cutoff carries rock partition and Pc evidence and this domain does not select it. |
| `T38` | Lambda is unavailable and no adjacent family is offered as an approximation. |
| `T39` | Swirr cannot exist without pressure plus lab/reservoir flag; identical numeric pressures select order-of-magnitude-separated points under the two states. |
| `T40` | Untagged corrected Pc refuses, tagged control accepts, and declared module identity wins over mismatched specification text. |
| `T41` | Independent cross-product enumeration matches exhaustive sweep; every depth participates; repeated runs rank identically; full ranking retained. |
| `T42` | Closure pick, entry pressure, and rock class are bit-identical with plot closed, linear, and logarithmic. |
| `T43` | Forward versus inverted Pittman routes diverge by cited opposite-direction factors at 5% and 30% porosity; result records direction. |
| `T44` | One angle range and absolute-cosine representation; 40 and 140 degrees agree to the chapter's three-decimal tolerance without treating rounding as sign error. |

## Parameter, Observation, Escalation, Refusal, and Acquisition Custody

### Parameter rows

- Section 5.1 has 14 rows. Derived candidates are the J expression, two gradient routes, exact metre/foot conversion, and `ln(10)`. The current J literal, current gradient literal, apply-path hydrocarbon density, and FWL default are UNSOURCED. The hydrocarbon-gradient default remains ABSENT. Do not resolve `ESC-SHR-3` by choosing between the two density paths.
- Section 5.2 has 9 rows. The four lab/reservoir fluid-system values remain ABSENT under `ESC-SHR-1`; live `IFT_RES=26` and fused `HG_AIR_IFT=367` remain UNSOURCED implementation evidence. Derived pressure/radius conversion rows may serve as test oracles; the reference-bank mercury constant remains NON-ADOPTABLE.
- Section 5.3 has 12 rows. `SWH_A`, `SWH_B`, `SH_A`, `SH_B`, `SH_C`, `SH_D`, and `SWT_IRR` remain UNSOURCED placeholders. Preserve the cited Brooks-Corey and Thomeer conventions, the existing algorithmic porosity screen, and the declared SandiBumi Buckles heuristic without promoting fit coefficients to defaults.
- Section 5.4 has 14 rows. Preserve the cited Amaefule, Corbett-Potter, Winland, Pittman, port-scheme, Permadi-Susilo, and FZI-basis rows. Lucia remains PRESENT-UNVERIFIED under `ESC-SHR-4`; its unsupported high-side null threshold remains ABSENT. The four `rt_cutoff` values remain one UNSOURCED row and may not become a house cutoff set.
- Section 5.5 has 5 rows. Swanson apex basis and alternative permeability-route constants remain ABSENT; the live Swanson pair, Oklahoma exponent, and BETA unit mismatch remain NON-ADOPTABLE verification evidence.
- Section 5.6 has 7 rows. Preserve the current algorithmic Ward/HFU controls and the cited Lorenz/VDP definitions. The Lorenz-inflection tolerance remains ABSENT; do not derive a numeric tolerance from one synthetic fixture.

### Contract/core observations

- Preserve the local fixture-independence clause under `SB-SHR-007`; do not mint a new `SB-CORE` id during adjudication.
- Preserve the missing primary-literature evidence-tier observation; do not invent a tier or rewrite the contract.
- Record that the chapter's stale `SB-CORE-001` Skelt description has been superseded in live source/history, but do not amend either PRD chapter during Gate 1.

### Escalations

- Keep `ESC-SHR-1` open for the fluid-property source/contract boundary, `ESC-SHR-2` blocking for Swanson basis, `ESC-SHR-3` open for the hydrocarbon-density house choice, `ESC-SHR-4` open for original Lucia evidence, and `ESC-SHR-5` open only as the priority/product ruling on an already-specified FWL-uncertainty contract.
- An escalation can coexist with an as-built verdict. It does not authorize Gate 1 to remove a live default, select a basis, choose a density, adopt a paper transcription, or reprioritize a requirement.

### Refusals and independent derivation

- Preserve implemented refusals `(a)` through `(i)`: no Pittman monotonic clamp, no printed-wrong Pc corrections, no silent natural-log Thomeer or reciprocal Brooks-Corey convention, no percent-porosity Lucia form, no sigma-scoped adoption, no forced equality of two published constants, no substituted Lorenz input curve, no dropped failed per-rock-type fit, and no barred fluid-property transcription.
- Preserve prospective refusals `(j)` through `(l)` without crediting unbuilt behavior: no display-dependent numerical result, no correction-reporting fall-through, and no gradient-free height relation.
- Preserve `SB-SHR-030` and `SB-SHR-039` as independent-derivation requirements. Do not reconstruct vendor internals, copy sampling caps, or import a vendor algorithm merely because a capability is commercially familiar.

### Acquisition and coverage gaps

- Preserve all eight acquisition gaps in ranked order: primary fluid-property reference; original Lucia paper; Swanson primary; alternative MICP permeability primaries; cutoff-selection papers owned by chapter 14; full Amaefule text; mixed-method SCAL dataset; and one licensed correction-module run.
- Preserve all six coverage gaps as unknowns, not negative evidence. Only the rendered clay-bound-water help gap currently gates an open salinity-unit escalation.
- Preserve the four chapter uncertainty notes: unmeasured Skelt magnitude in the historical source state, possibly incomplete log-porosity correlation inventory, negative evidence on fit persistence, and the judgement involved in applying C-2 discipline to the deterministic sweep.

## Execution Tasks

### Task 1: Refreeze the baseline

- [ ] Re-run branch, worktree, hash, anchor, ledger, source-column, test-ID, parameter, observation/escalation/refusal/acquisition, manual-evidence, and candidate-test checks.
- [ ] Stop and reconcile if the branch, anchor reachability, source hashes, row counts, source-owned fields, or chapter structure differ from this plan.

### Task 2: Reverify units, constants, and fitted-object custody (`001`-`011`)

- [ ] Trace every height family from project-unit declaration through coefficient-unit conversion, result value, unit label, IPC, dialog, generic export, and reachable regression history.
- [ ] Reconcile the chapter's historical Skelt finding with `bb807ca` and the two current parity tests; classify the observable closure and any remaining new-family/static gap separately.
- [ ] Inventory each physical literal, source string, duplicate, fixture oracle, fluid-property default, and fused quantity without adopting any value.
- [ ] Confirm persistence negatively across fit result types, document/store inventory, project reopen behavior, and all four fit dialogs before classifying `009`/`010`.

### Task 3: Reverify conventions, validity, and honesty (`012`-`025`)

- [ ] Trace Brooks-Corey, Thomeer, Swanson, port-class, RQI, log-porosity, Lucia, Pittman, exclusion-ledger, classifier, and unverified-state obligations through result/UI/export/report surfaces.
- [ ] Keep each implemented defect refusal separate from the missing declaration, flag, provenance, or choice surface it does not yet satisfy.
- [ ] Treat the Pittman paper-backed table/crossover tests as independent candidates, while treating self-generated synthetic fits and helper round trips as supporting unless their oracle is independent.
- [ ] Confirm report methodology from the emitted PDF and Word paths, not only `default_methodology` or the editable text box.

### Task 4: Reverify corrections and capability completeness (`026`-`033`)

- [ ] Trace SCAL parse/store fields, fluid-system standardisation, correction inventory, non-wetting representation, throat-distribution diagnostics, Lorenz/HFU partitioning, and result/dialog/report surfaces.
- [ ] Keep all fluid-property and alternative-permeability constants absent; record missing primaries as dependencies rather than implementation suggestions.
- [ ] Distinguish shared exact Ward logic from the absent Lorenz-inflection method, and distinguish existing Winland/PGS outputs from absent Aguilera identity.

### Task 5: Reverify seams, model selection, and reproducibility (`034`-`042`)

- [ ] Trace chapter-14 gradient use, derived-cutoff custody, Lambda refusal, Swirr reference type, corrected-Pc import tags, sweep inventory, display/numeric separation, regression direction, and contact-angle representation.
- [ ] Confirm consequential absence across expected Rust, TypeScript, tests, and reachable history.
- [ ] Do not credit an unbuilt numerical operation with a prospective refusal and do not infer a model-selection grid, depth cap, angle range, or correction tolerance.

### Task 6: Write the evidence receipt and update all 42 ledger rows

- [ ] Create `docs/takeover/evidence/sb-shr.md` with one complete section per requirement in numeric order.
- [ ] Update only live adjudication columns in `docs/takeover/requirements.csv`.
- [ ] Recompute the ledger and update `docs/takeover/STATUS.md` with exact as-built, disposition, risk, proof, manual, and remaining-row counts.
- [ ] Verify the six source-owned columns remain byte-identical to the frozen hash.

### Task 7: Run structural and evidence checks

- [ ] Assert 42 receipt sections and 42 matching ledger rows, each legal and non-placeholder.
- [ ] Assert all 44 test intentions are routed exactly once and all 61 parameters, three observations, five escalations, twelve refusals, two independent-derivation requirements, eight acquisition gaps, six coverage gaps, and four uncertainty notes are accounted for.
- [ ] Search new receipt/ledger text for prohibited identifying names, invented defaults, source-gap fills, protected data, and accidental claims of field evidence.
- [ ] Run `node --test tools/takeover-ledger.test.mjs`, `node tools/takeover-ledger.mjs --summary-json`, and `git diff --check`.

### Task 8: Run the repository gates and commit the execution increment

- [ ] Run `npx tsc --noEmit` from the repository root.
- [ ] Run `cargo check` from `src-tauri`.
- [ ] Run `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from the repository root. If the live Tauri application owns the default debug executable, use the established isolated `CARGO_TARGET_DIR`; do not stop the live application.
- [ ] Record fresh passed/failed/ignored totals; do not reuse the prior `946/0/36` result.
- [ ] Stage exactly `docs/takeover/evidence/sb-shr.md`, `docs/takeover/requirements.csv`, and `docs/takeover/STATUS.md`; inspect the cached diff and source-column hash.
- [ ] Commit locally with message `G1-DOM-SHR adjudicate 42 SB-SHR requirements`. Do not push, merge, or open a pull request.

### Task 9: Continue Gate 1 serially

- [ ] Recompute the remaining domain inventory from the post-SHR ledger, keep all 52 SB-GEO rows deferred to the next product version, and choose the next dependency-relevant open-hole-petrophysics domain.
- [ ] Create the next serial planning branch in the same `D:\XX. SandiBumi` worktree without touching any other folder.
- [ ] Do not begin Gate 2 production remediation until Gate 1 evidence reconciliation is complete or Jauhar explicitly changes the sequence.

## Plan Self-Review Receipt

- **Spec coverage:** all 42 requirements appear exactly once in the Requirement Evidence Map; all 44 tests appear exactly once in the Acceptance-Test Ownership Map; all 61 parameters, three observations, five escalations, twelve refusals, two independent-derivation requirements, eight acquisition gaps, six coverage gaps, and four uncertainty notes have explicit custody.
- **No placeholders:** this plan contains no guessed source, unspecific error-handling instruction, invented numeric control, or cross-task shorthand.
- **Type/interface consistency:** the plan uses the live names `LeverettFit`, `ScalPoint`, `CuddyFoilRequest`, `CuddyFoilResult`, `ShfFitRequest`, `ShfFitResult`, `ThomeerRequest`, `ThomeerResult`, `HfuRequest`, `HfuResult`, `LorenzRequest`, `LorenzResult`, `ModuleSpec`, `ModuleRunResult`, and the exact current Rust/TypeScript paths. It invents no production API.
- **Source discipline:** no fluid property, FWL, model coefficient, cutoff, apex basis, port scheme, correction, permeability route, reference pressure, angle range, boundary tolerance, or vendor behavior is adopted beyond the immutable chapter. Every ABSENT, NON-ADOPTABLE, UNSOURCED, and PRESENT-UNVERIFIED exposure remains fenced.
- **Evidence discipline:** 58 passing direct-domain tests are candidates only until observable scope and independent expected values are checked. Manual saturation-height evidence remains `0/6` and rock-typing evidence remains `0/26`.
- **Scope discipline:** planning changes only this plan and STATUS; execution changes only the SHR receipt, ledger, and STATUS. Production remediation remains a later Gate 2 activity.

## Execution Handoff

Execute inline in the current session with `superpowers:executing-plans`. Jauhar's persistent instruction is to continue Gate 1 serially, so no additional approval is required unless execution reaches an unresolved product-owner decision or source gap that would otherwise be silently guessed.
