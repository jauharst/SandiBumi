# SB-PLG Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task by task. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 48 live `SB-PLG` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition per row, route all 68 chapter acceptance-test intentions, and preserve every unit, geometry, calibration, phase, null, cutoff, casing, correction, provenance, array-validity, parameter, refusal, and manual-evidence boundary without changing production behavior.

**Architecture:** This is a documentation-only evidence pass. Requirements 001-015 cover domain registration, production units, spinner/Vapp, geometry, phase schemas, time/station import, and null reduction. Requirements 016-031 cover cement indices, coverage, collar masking, impedance, probability/confidence, channeling, statistics, classifications, interval reporting, and waveform extraction. Requirements 032-044 cover casing-loss quantities, prior-survey merge, ovality, Barlow, nominal geometry, grades, despiking, correction recipes, calibration, environmental correction, weight/tension units, and collar detection. Requirements 045-048 cover complete run provenance, interpreted identity, machine-readable reports, and end-to-end array shape/validity. The immutable PRD supplies the intended contract, 132 parameter rows, 10 open items, 10 escalations, 18 refusals, 68 named test intentions, and 239 section-8 traceability rows. Current source, independent tests, manual evidence, and reachable Git history supply the separate live verdict. Generic scalar/array carriage is supporting infrastructure, not evidence that a production-logging calculation, definition gate, report, or domain unit exists.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js, TypeScript, Rust `cargo test`, the takeover-ledger validator, and the existing SandiBumi full gate.

## Global Constraints

- This planning increment may create this plan and update `docs/takeover/STATUS.md` only. It MUST NOT modify a ledger verdict, evidence receipt, Rust, TypeScript, CSS, tests, `REVIEW.md`, generated verification output, any file under `docs/PRD_v2/**`, or any file under `docs/research_2026-08/**`.
- Execute the later adjudication on GPT-5.6 Sol at xhigh with `superpowers:executing-plans`. Do not delegate or spawn subagents unless Jauhar explicitly authorizes it. Production-log method interpretation, parameter custody, data-integrity classification, and final sign-off stay with the primary session. Reserve Sol max for the final all-931-row Gate 1 audit.
- Work only in `D:\XX. SandiBumi`. It MUST remain the sole registered Git worktree. The retired `D:\XX. SandiBumi-check` path remains untouched.
- The accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. At plan freeze, `HEAD` is `6f3a52e9caf82dccfc203e17b1d1263a2e4cef68`; `origin/master` and the merge base are both `29833735816d9e5be954afafd9ceb71fd856e3f0`; the accepted anchor is reachable. Reverify all four before execution.
- The local planning branch is `codex/g1-sb-plg-plan`. The serial Gate 1 chain remains local and unpushed; do not merge, rebase, rewrite history, push, or open a pull request. After the planning commit, create `codex/g1-sb-plg-adjudication` in the same worktree.
- The codebase-index MCP server is not callable in this task. Targeted filesystem search is the explicit fallback. A consequential Rust/TypeScript absence MUST be confirmed across module registration/dispatch, unit typing, text/array intake, scalar/array persistence, IPC/display, curve/log-set provenance, reports/exports, tests, and reachable history.
- Before adjudicating, read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, all of `docs/PRD_v2/26_production-logging.md`, `docs/record_data_tools.md`, the current verification matrix, takeover receipts/status, and the exact source/tests about to be cited. No dedicated production-logging implementation record exists.
- Preserve the ledger's source-owned fields byte-for-byte: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status`, and `owned_tests`. Their frozen SHA-256 over the 48 ordered rows is `437fc033b9ff179a4a85f8aeaa91677b6934d4ec9df86d95d35e7b8a4f352b31`. The immutable chapter SHA-256 is `4ff1d534977621784494f444bc1a49d7d10ca2b425e4d38637d72e7e6fa0b6b9`.
- The chapter and ledger agree on 48 contiguous requirements: `P0=24`, `P1=17`, `P2=7`. Historical states are `ABSENT=43` and `PARTIAL=5`; all 48 live verdicts remain `UNADJUDICATED`. Reverify live behavior independently rather than copying those labels.
- The chapter defines 68 contiguous test IDs, `SB-PLG-T01` through `SB-PLG-T68`. The ledger has 69 ownership references covering all 68 unique IDs; T31 is shared by 019/048 and no ID is missing or unknown. Route each unique test intention once and preserve shared ownership without duplicating evidence.
- Section 5 contains 132 parameter rows, including 32 deliberate `ABSENT` values. Preserve every absence and source fence. In particular, do not invent a mixture-velocity method, slip/holdup law, phase-rate route, geometry table, cement threshold, expected-CBL coefficient authority, casing strength, correction recipe, or array-axis substitute.
- Preserve O-1 through O-10, E-1 through E-10, R-1 through R-18, all 239 traceability rows, and the independent-derivation boundary. A vendor screenshot, generic implementation, common industry practice, current code literal, or neighboring domain never becomes parameter authority.
- `f32::NAN`, raw bytemuck array bytes over IPC, `parsers::read_text_file`, subprocess Python with `sys.stdin.buffer`, undoable edits, and the existing DuckDB write discipline remain mandatory. This documentation lane must not change or reinterpret them.
- Current source has no production-specific module, command, importer, computation, report, or UI. Absence correctly avoids fabricated results, but absence does not implement the required Vapp output plus an explicit downstream refusal.
- The generic array store writes an optional axis blob, but `db::ArrayRow` contains only `depth` and `samples`; `read_array_log` selects only those two columns; `get_array_log` IPC sends depth, uniform width, and padded values only. Therefore `SB-PLG-048` is a live `PRESENT-DIVERGENT` candidate: “axis preserved end to end” is false even though row width and NaN samples can survive part of the path.
- The existing array round-trip test proves realization sample order only. It does not assert the stored axis, IPC axis, display axis, export axis, per-row valid count, or the full T68 contract. Generic intake/null/unit/provenance tests are supporting evidence only and MUST NOT be promoted into whole-contract SB-PLG proofs.
- Generic log-set provenance records module, parameter JSON, input JSON, version, and time, but no PLG-specific method/unit/source/mask/correction-order/warning completeness invariant. Generic imported/computed catalog labels likewise do not provide typed interpreted identities for ambiguous `BPI`, `OVALITY`, `MLOSS`, or unitless velocity.
- Manual evidence remains separate. Production logging, cement evaluation, and casing integrity are not listed as verification-matrix capabilities and therefore remain `0/0`, not “complete.” Supporting areas remain array-logs `0/16`, data-conventions `0/45`, generic-curve-store `0/18`, conditioning `0/27`, workflow `0/23`, report `6/53`, office-deliverables `0/39`, LAS-export `0/2`, processing-history `0/7`, portfolio-performance `0/50`, security-integrity `0/63`, and verification-stewardship `0/24`. Automated evidence closes none of these scenarios.
- New receipt and ledger text MUST contain no client, field, block, basin, operator, asset, well, or project name. Refer only to physical conditions, generic records, and source classes.

## Baseline and Count Contract

Before any adjudication edit, re-measure and record:

1. branch `codex/g1-sb-plg-adjudication`, created serially from the committed plan;
2. one clean worktree at `D:\XX. SandiBumi`;
3. current `HEAD`, accepted anchor, `origin/master`, merge base, and anchor reachability;
4. exactly 48 ledger rows, SB-PLG-001 through SB-PLG-048, with no gap or duplicate;
5. priorities `P0=24`, `P1=17`, `P2=7`;
6. historical source states `ABSENT=43`, `PARTIAL=5`;
7. all 48 live mutable evidence fields still unadjudicated or placeholder-only;
8. exactly 68 defined chapter test IDs, 69 ledger ownership references, and all 68 unique IDs owned;
9. exactly 132 parameter rows, including 32 deliberate absent values;
10. exactly 10 open items, 10 escalations, 18 refusals, and 239 section-8 traceability rows;
11. takeover summary `805` adjudicated, `126` unadjudicated, and `521` pilot blockers before SB-PLG;
12. no production-specific implementation candidate in current source or reachable history;
13. the manual evidence counts listed above; and
14. fresh focused supporting evidence for array sample order, wide-table axis parsing, unknown-unit preservation, null recognition, and generic log-set version/provenance behavior.

The only mechanically predictable post-adjudication ledger counts are `853` adjudicated and `78` unadjudicated. The preliminary release map is 41 P0/P1 rows as `PILOT-BLOCKER`, six P2 rows as `UNDECIDED`, and optional expected-CBL row 022 as `DEFERRED`; reverify each row before writing it. Do not freeze as-built or risk totals merely to make the receipt match this plan.

## File Structure for the Execution Increment

- Create: `docs/takeover/evidence/sb-plg.md`
- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/STATUS.md`
- Read only: `src-tauri/src/db.rs`, `src-tauri/src/intake.rs`, `src-tauri/src/ingest.rs`, `src-tauri/src/parsers.rs`, `src-tauri/src/curves.rs`, `src-tauri/src/units.rs`, `src-tauri/src/modules.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/composite.rs`, `src-tauri/src/report.rs`, `src-tauri/src/export.rs`, `src/ipc.ts`, applicable UI code, current tests, manual evidence, immutable source chapter, historical implementation records, and reachable Git history
- Never modify during adjudication: production code, tests, `REVIEW.md`, generated verification artifacts, PRD files, research dossiers, protected files, or unrelated takeover receipts.

## Evidence Receipt Schema

Create one `### SB-PLG-NNN` section per requirement in numeric order. Every section MUST include the specified contract, current implementation, as-built status, release disposition/risk, exact automated evidence class, manual evidence, source/parameter boundary, UI/IPC/provenance surface, history/reachability, blocking decision/dependency, and next action. Separate every observable limb of a compound requirement. Name whether evidence is correctness, characterization, optional-package, structural, manual, or missing; never inflate a helper-level test, generic carriage test, internal `Result`, renderer output, or storage-only round trip into a whole-contract proof.

## Requirement Evidence Map

| ID | Tests | Exact contract focus | Primary live candidates and adjudication guard |
|---|---|---|---|
| `001` | T01-T02 | Three separately gated `cement_eval`, `casing_integrity`, and `prodlog` units | Module registry/dispatch contains none of the three. A generic module list is not a domain capability gate. Candidate `ABSENT`. |
| `002` | T03-T05 | Production quantity typing, conversion, and refusal before computation | Generic import preserves unknown units and flags them unconverted, but the registry lacks velocity, slope, rate, tension, casing-weight, and acoustic-impedance quantity families/conversions. Candidate `PARTIAL`; do not credit verbatim carriage as typed computation. |
| `003` | T06-T08 | Zonal-average spinner calibration, branch inheritance, and flat/interpolated slope | No spinner calibration route or artifacts exist. Candidate `ABSENT`; do not infer algorithms from the acceptance arithmetic. |
| `004` | T09-T10 | Exact Vapp equation and sign-rule refusal | No Vapp route exists. Candidate `ABSENT`; the cited equation is authority, not current behavior. |
| `005` | T11 | Normalized multi-pass weighting | No production pass merger exists. Candidate `ABSENT`. |
| `006` | T12 | Emit Vapp while explicitly refusing unsupported Vmix/slippage/rates | No Vapp route and no downstream refusal exist. Candidate `ABSENT`; “nothing computes” is not the compound specified behavior. |
| `007` | T13-T14 | Per-family/per-tool sensor identities, angular geometry, and unit transforms | Generic arrays carry ordered samples and an optional common axis only; there is no tool/family sensor schema or geometry transform. Candidate `PARTIAL`. |
| `008` | T15-T16 | Explicit 24-output RST three-phase schema and mandatory inputs | No RST route or output manifest exists. Candidate `ABSENT`; P2 remains undecided. |
| `009` | T17 | Visible and persisted temperature-flow assumptions | No temperature-flow production method exists. Candidate `ABSENT`; P2 remains undecided. |
| `010` | T18 | Selective-inflow data sufficiency | No SIP route exists. Candidate `ABSENT`. |
| `011` | T19 | Cumulative-flow differentiation with declared window length | No cumulative-flow differentiation route exists. Candidate `ABSENT`; P2 remains undecided. |
| `012` | T20-T22 | Explicit Chronolog epochs and discriminate-before-estimate order | Generic text intake exists but no Chronolog/station reducer, epoch identity, or operation-order record exists. Candidate `ABSENT`. |
| `013` | T23 | Evidence-bounded station ASCII grammar and dot-decimal refusal | Generic text intake is not a station importer and does not establish the documented station grammar. Candidate `ABSENT`. |
| `014` | T24 | Normalize declared/suspected nulls before station reduction with audit | Generic import has declared null handling and `f32::NAN`, but no station suspected-null identification, exclusion audit, or reducer. Candidate `PARTIAL`. |
| `015` | T25 | Phase meaning, colors, units, and [0,1] validation | No production phase schema or phase UI exists. Candidate `ABSENT`. |
| `016` | T26-T27 | Measurement-family-specific cutoff polarity and endpoint refusal | No cement cutoff engine or typed family gate exists. Candidate `ABSENT`; no threshold may be borrowed. |
| `017` | T28-T29 | Exact logarithmic attenuation bond index | No bond-index computation exists. Candidate `ABSENT`. |
| `018` | T30 | Required named attenuation-versus-amplitude interpolation method | No bond interpolation method selector/provenance exists. Candidate `ABSENT`. |
| `019` | T31-T32 | Coverage denominator equals finite valid row width | Generic array rows preserve sample vectors and NaNs through storage, but no coverage computation or validity-denominator record exists. Candidate `PARTIAL`; T31 is also owned by 048. |
| `020` | T33 | Exclude collar samples from statistics without deleting export data | No cement collar mask/statistic/export chain exists. Candidate `ABSENT`. |
| `021` | T34-T35 | Slurry acoustic impedance in declared units | No slurry impedance route or required production units exist. Candidate `ABSENT`. |
| `022` | T36 | Expected-CBL correlation remains optional, attributed, and warning-bearing | No correlation route exists and coefficient authority is deliberately not adopted. Candidate `ABSENT`, release `DEFERRED`; preserve the absent authority. |
| `023` | T37-T38 | Probability terms and confidence weights remain distinct | No cement probability/confidence engine exists. Candidate `ABSENT`. |
| `024` | T39 | Validate service-term switches before probability calculation | No probability service selector or dependency validator exists. Candidate `ABSENT`. |
| `025` | T40 | Explain the reachable ceiling of a single-service score | No cement-index UI or ceiling explanation exists. Candidate `ABSENT`. |
| `026` | T41 | Channel detection with the adopted direction and visible warning | No channel detector or direction warning exists. Candidate `ABSENT`. |
| `027` | T42-T43 | Separate array derivative, element smoothing, and vertical statistics | Generic conditioning does not implement these three typed casing/cement operations or their incompatible windows. Candidate `ABSENT`. |
| `028` | T44 | Four-direction microdebond evidence and named neighborhood | No image microdebond route exists. Candidate `ABSENT`; P2 remains undecided. |
| `029` | T45 | Distinct cement classification identities | No cement classification schemes exist. Candidate `ABSENT`. |
| `030` | T46 | Isolation report minimum-interval semantics | No isolation report engine exists. Candidate `ABSENT`. |
| `031` | T47 | Reproducible waveform window, peak pick, gain, and delay | Generic arrays can carry waveform-like samples but no waveform extraction method/provenance exists. Candidate `ABSENT`; P2 remains undecided. |
| `032` | T48-T49 | Four canonical, dimensionally distinct casing-loss quantities | No casing-loss engine or identity registry exists. Candidate `ABSENT`; ambiguous `MLOSS` remains raw rather than becoming a result. |
| `033` | T50 | Retain signed apparent loss with flags, never clamp | No casing-loss computation exists. Candidate `ABSENT`. |
| `034` | T51 | Explicit prior-survey merge modes and current-absent behavior | No prior-survey casing merge exists. Candidate `ABSENT`. |
| `035` | T52-T53 | Named ovality definition and raw-ambiguous refusal | Generic import can retain raw `OVALITY`, but no typed ovality definition or threshold gate exists. Candidate `ABSENT`; viewability is not computation eligibility. |
| `036` | T54-T55 | Barlow computation only from sourced strength | No Barlow route exists. Candidate `ABSENT`; do not add a grade-strength table. |
| `037` | T56 | Sourced or measured nominal casing geometry | No casing-geometry source registry/refusal path exists. Candidate `ABSENT`. |
| `038` | T57 | Separate grades bound to their actual measurement quantities | No casing grade/classification route exists. Candidate `ABSENT`. |
| `039` | T58 | Three independently auditable, default-off despike stages | Generic conditioner is not the specified array/scalar/bad-azimuth three-stage contract. Candidate `ABSENT`. |
| `040` | T59 | Preserve named correction recipes and stage order | No production correction recipes or order validator exist. Candidate `ABSENT`. |
| `041` | T60 | One-depth offset-only versus two-depth gain-and-offset calibration | No pipe/casing calibration route exists. Candidate `ABSENT`. |
| `042` | T61 | Refuse double environmental correction | No production environmental-correction state/refusal exists. Candidate `ABSENT`. |
| `043` | T62-T63 | Canonicalize casing mass-per-length and tension separately | Unknown units are retained verbatim and flagged; the typed registry contains none of `lbf/ft`, `lb/ft`, `lbm/ft`, `KLBF`, or `LBF`. Candidate `ABSENT`, not partial. |
| `044` | T64 | Collar detection cutoff and search-window semantics without smoothing | No CCL collar picker exists. Candidate `ABSENT`. |
| `045` | T65 | Full PLG method/version/units/sources/inputs/masks/order/warnings provenance | Generic log sets record module, parameters, inputs, version, and time, but no PLG method or complete required payload exists. Candidate `PARTIAL`. |
| `046` | T66 | Separate raw imported, computed, and interpreted identities with use gates | Catalogs distinguish generic imported/computed data and retain unknown names, but no typed interpreted identity/definition gate exists for ambiguous PLG mnemonics. Candidate `PARTIAL`. |
| `047` | T67 | Machine-readable cement/casing interval report with numeric truth and masks | No production report/export path exists. Candidate `ABSENT`; P2 remains undecided. |
| `048` | T31,T68 | Preserve width, NaN slots, valid count, axis, display, and export end to end | Storage writes axis, but read/IPC discard it; IPC pads ragged rows to a maximum width; no array export carries the axis/validity contract. Candidate `PRESENT-DIVERGENT`, not partial. |

## Harsh-Truth Review Before Execution

1. Forty-eight requirements are not forty-eight small functions. The chapter describes three independently gated products—production flow, cement evaluation, and casing integrity—plus shared import/report infrastructure.
2. There is no production-specific implementation or executable SB-PLG acceptance test in the current tree. A green generic array or import test cannot be renamed into domain proof.
3. The array-axis defect is a real data-integrity divergence: the system accepts and stores semantic axis values, then silently makes them unavailable to the normal reader and frontend transport. A renderer can therefore display values without the physical coordinate that says what each bin means.
4. The absence of Vmix/slippage/rates avoids fabricated phase rates, but SB-PLG-006 is still absent because the required safe product behavior is “produce Vapp, then refuse unsupported downstream quantities by name.”
5. Thirty-two parameter values are deliberately absent. Gate 1 must preserve those absences even if the corresponding equations are familiar or easy to find elsewhere.
6. Production logging/cement/casing has zero listed manual scenarios, not verified coverage. The verification matrix must eventually gain real capability rows; this documentation-only adjudication cannot manufacture them.

## Task 1: Reverify Immutable Inputs and Live Evidence

1. Re-run the baseline/count checks above and stop on any branch, worktree, source-hash, row-count, test-ID, parameter-count, or traceability drift.
2. Confirm the absence search across module registration/dispatch, commands, UI, reports/exports, tests, and reachable history.
3. Run the five focused generic evidence tests named in the baseline contract. Record them only as supporting seams.
4. Inspect `db::write_array_log`, `db::read_array_log`, `db::ArrayRow`, the `get_array_log` IPC layout, frontend decoding, display, and exports. Record the first surface at which axis/validity semantics are lost.

## Task 2: Write the 48-Section Receipt

1. Create `docs/takeover/evidence/sb-plg.md` with one section per requirement and no omissions.
2. Route all 68 unique test intentions exactly once; preserve the shared T31 ownership.
3. Classify every current whole-contract proof honestly. Supporting generic tests must remain narrower evidence; absent exact acceptance bodies remain `MISSING`.
4. Route all 132 parameters, O-1…O-10, E-1…E-10, R-1…R-18, and all 239 traceability rows.
5. Record production/cement/casing manual evidence as not listed `0/0`, then list applicable supporting-area counts without converting them into field proof.

## Task 3: Update Only the 48 Ledger Rows

1. Replace only mutable evidence fields for `SB-PLG-001` through `SB-PLG-048`.
2. Preserve every non-PLG byte and all six source-owned PLG fields.
3. Use `PILOT-BLOCKER` for all 41 P0/P1 rows unless live evidence proves a stronger release disposition; use `UNDECIDED` for six P2 rows and `DEFERRED` for optional expected-CBL row 022.
4. Run `node tools/takeover-ledger.mjs --check`; verify `853/931` adjudicated and `78` unadjudicated.

## Task 4: Update the Dashboard and Verify the Increment

1. Update `docs/takeover/STATUS.md` with exact measured verdict, risk, release, and test-class totals; do not estimate.
2. Verify the receipt has 48 unique requirement sections and every immutable routing count.
3. Run, in order, `npx tsc --noEmit`, `cargo check`, and `powershell -ExecutionPolicy Bypass -File tools\check.ps1` from the authoritative repo.
4. Inspect `git diff --check`, the exact file set, source-owned hash, non-PLG byte identity, and worktree status.
5. Commit exactly the receipt, ledger, and dashboard with message `G1-DOM-PLG adjudicate 48 SB-PLG requirements`. Do not push.

## Completion Contract

The increment is complete only when all 48 rows have one live verdict, release disposition, risk class, evidence class, source boundary, dependency, and next action; all 68 tests, 132 parameters, 10 opens, 10 escalations, 18 refusals, and 239 traceability rows are routed; the source-owned hash and all non-PLG bytes are preserved; the receipt and dashboard agree with the validator; the full gate is green; and the exact documentation-only file set is committed locally. No production capability, parameter, test, manual scenario, PRD statement, or unrelated receipt changes in this increment.
