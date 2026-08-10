# SB-DIO Live Adjudication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.

**Goal:** Reverify every one of the 63 live `SB-DIO` requirements against the accepted Gate 1 tree, record one evidence-backed as-built classification and pilot disposition for each row, and produce bounded follow-up increments without changing product behavior.

**Architecture:** This is a documentation-only evidence pass. The PRD remains an immutable statement of intent and historical chapter status; current source, qualifying acceptance tests, manual evidence and reachable Git history establish the separate as-built classification. One domain receipt explains every verdict, while `docs/takeover/requirements.csv` carries the machine-validated summary. The pass follows the data path from bytes and container identity through null/index/unit handling, native samples, curve identity, import commit, and export verification so a locally correct parser cannot conceal a downstream fidelity failure.

**Tech Stack:** Markdown, RFC 4180 CSV, PowerShell 5.1, Git, `rg`, Node.js `node:test`, Rust `cargo test`, the existing takeover-ledger validator and the existing SandiBumi full gate.

## Global Constraints

- This increment MUST NOT modify Rust, TypeScript, CSS, package behavior, database schema or any generated product artifact.
- Execute this plan on the session model with `superpowers:executing-plans`; do not delegate or spawn subagents unless Jauhar explicitly authorizes that in the execution turn.
- This increment MUST NOT edit `docs/PRD_v2/**`, `REVIEW.md`, `verification/capabilities.json`, `docs/VERIFICATION_MATRIX.md`, `ROADMAP.md`, `CLAUDE.md` or `AGENTS.md`.
- The exact accepted implementation evidence anchor is `b332026cb498c105f36eade0bf7899bc0c1309f0`. Reverify it before execution. The planning branch is stacked on the reviewed SB-CORE adjudication commit, while `origin/master` was frozen at `29833735816d9e5be954afafd9ceb71fd856e3f0` when this plan was written. If either accepted reference moves, stop and reconcile the base before classifying a row.
- Work only in `D:\XX. SandiBumi`. Leave the empty, locked `D:\XX. SandiBumi-check` folder untouched; it is not evidence and is not a Git worktree.
- The codebase-index MCP server is not callable in the current task. Targeted filesystem search is therefore the explicit fallback. A consequential negative result MUST be confirmed against exact source files, tests and reachable history.
- Read `AGENTS.md`, all of `CLAUDE.md`, `docs/PRD_v2/CONTRACT.md`, the complete data-I/O chapter, the applicable `docs/record_*.md` and the takeover design before adjudicating.
- Preserve the ledger's source-owned fields exactly: `requirement_id`, `chapter`, `title`, `original_priority`, `chapter_status` and `owned_tests`. Chapter statuses are historical evidence, not current verdicts. The many post-chapter SB-DIO commits make copying those statuses especially unsafe.
- `as_built_status` answers only what the accepted tree currently ships. `release_disposition` answers only whether the contract belongs in the Windows-first paid pilot. Original P0/P1/P2/P3 is evidence, not automatic pilot scope.
- A positive code path does not close a requirement by itself. Check every obligation joined by `and`, `every`, `all`, `never`, `must`, or an enumerated list. One missing obligation makes the result `PARTIAL` or `PRESENT-DIVERGENT`, not `PRESENT-OK`.
- A test counts as an owned acceptance test only when it exercises the requirement's observable contract, maps to the chapter's test sentence, and uses an independently sourced expected value. A helper test, internal `Result`, or passing parser call is supporting evidence only.
- Classify test evidence exactly under `CONTRACT.md` sections 3 and 6. An unsourced expected value is `CHARACTERIZATION`, never correctness. Optional-package ignores and specified-divergence ignores remain separately named; do not use `#[ignore]` as evidence of a passing contract.
- Every text import must still route through `parsers::read_text_file`; no direct `read_to_string` or `BufReader<File>` path may be treated as conforming. Every Python runner must still read `sys.stdin.buffer`; a successful ASCII fixture cannot close the non-ASCII sidecar contract.
- Preserve native source sampling and `(set_name, mnemonic)` identity. Viewing is not Reframe. Resampling is conforming only when it is explicit, named, recorded, and performed through the explicit Reframe/write option described by the requirement.
- Missing data is `f32::NAN`, never `Option<f32>` in the numeric array contract. Arrays cross IPC as bytemuck bytes, never JSON number arrays. A container test that drops array channels cannot close multidimensional DLIS.
- The frontend never sends SQL for writes. Do not propose changing `db.rs`, relaxing the one-writer/one-transaction discipline, or adding `ON CONFLICT`/upsert to deliberately PK-less `computed_curves`.
- `SB-DIO-023` is a hard evidence block. Physical-family bounds are deliberately absent in `21_data-io.md` section 5.6 and named as open item O-4 in section 7.1. Do not supply bounds, do not infer them from labels, and do not execute T36-T38 as correctness tests until SB-ENV supplies cited bounds. Record the requirement as blocked unless current evidence changes the stated dependency without inventing a number.
- `SB-DIO-057` likewise has no sourced log-scale family membership under section 7.1 O-5. Record the missing family registry; do not invent one.
- The RP66 multidimensional-channel, CWLS LAS 3.0 associated-section and MS-XLS mechanisms must be verified from the chapter's acquired normative sources or remain gaps. A library's current behavior is not a substitute for the specified published mechanism.
- Manual and field evidence comes only from `REVIEW.md` and `docs/VERIFICATION_MATRIX.md`. An unchecked scenario stays unchecked. Automated, desktop-harness and build evidence do not become field evidence.
- Git reachability proves that a change is in the accepted tree, not that the behavior is correct. Commit messages are locators only; open the accepted source and the test body.
- No branch switch, rebase, merge, push, PR or worktree cleanup occurs during execution. Every repository write is made with `apply_patch`; stage exact paths only.
- The approved plan authorizes only the evidence receipt, the 63 ledger-row adjudications and the dashboard handoff. It does not authorize a parser/export fix, a new test, a new parameter, or a product-owner decision.

---

## Baseline and Count Contract

The consolidated ledger contains exactly these 63 SB-DIO rows:

```text
SB-DIO-001 SB-DIO-002 SB-DIO-003 SB-DIO-004 SB-DIO-005 SB-DIO-006 SB-DIO-007
SB-DIO-008 SB-DIO-009 SB-DIO-010 SB-DIO-011 SB-DIO-012 SB-DIO-013 SB-DIO-014
SB-DIO-015 SB-DIO-016 SB-DIO-017 SB-DIO-018 SB-DIO-019 SB-DIO-020 SB-DIO-021
SB-DIO-022 SB-DIO-023 SB-DIO-024 SB-DIO-025 SB-DIO-026 SB-DIO-027 SB-DIO-028
SB-DIO-029 SB-DIO-030 SB-DIO-031 SB-DIO-032 SB-DIO-033 SB-DIO-034 SB-DIO-035
SB-DIO-036 SB-DIO-037 SB-DIO-038 SB-DIO-039 SB-DIO-040 SB-DIO-041 SB-DIO-042
SB-DIO-043 SB-DIO-044 SB-DIO-045 SB-DIO-046 SB-DIO-047 SB-DIO-048 SB-DIO-049
SB-DIO-050 SB-DIO-051 SB-DIO-052 SB-DIO-053 SB-DIO-054 SB-DIO-055 SB-DIO-056
SB-DIO-057 SB-DIO-058 SB-DIO-059 SB-DIO-060 SB-DIO-061 SB-DIO-062 SB-DIO-063
```

At plan time all 63 are `UNADJUDICATED`, `UNDECIDED`, `UNCLASSIFIED`, `MISSING-OR-UNCLASSIFIED`, `UNVERIFIED`, with `next_action=LIVE-ADJUDICATION`. The original priority mix is ten P0, forty-one P1, eleven P2 and one P3. Gate 1 adjudicates all 63 because a lower historical priority can still affect paid-pilot fidelity and release disposition is deliberately independent of old priority.

Run this guard before and after editing:

```powershell
$dio = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-DIO-*' }
$expected = 1..63 | ForEach-Object { 'SB-DIO-{0:D3}' -f $_ }
if ($dio.Count -ne 63) { throw "Expected 63 SB-DIO rows, found $($dio.Count)" }
if (@(Compare-Object $expected @($dio.requirement_id)).Count -ne 0) {
    throw 'The live SB-DIO ID set differs from the approved plan'
}
```

The mechanical post-execution count is exactly 88 adjudicated and 843 unadjudicated out of 931. This plan predicts no as-built or release-disposition totals.

---

## File Structure

### Create during adjudication

- `docs/takeover/evidence/sb-dio.md` - complete 63-row evidence receipt, including obligation-by-obligation source findings, tests, manual evidence, history, verdict and next action.

### Modify during adjudication

- `docs/takeover/requirements.csv` - only adjudication-owned fields for the 63 SB-DIO rows.
- `docs/takeover/STATUS.md` - measured row counts, blocker summary and next serial domain handoff.

### Read-only governing inputs

- `AGENTS.md`
- `CLAUDE.md`
- `docs/superpowers/specs/2026-08-10-sandibumi-takeover-design.md`
- `docs/PRD_v2/CONTRACT.md`
- `docs/PRD_v2/03_EVIDENCE_BASE.md`
- `docs/PRD_v2/21_data-io.md`
- `docs/PRD_v2/06_SEQUENCING_AND_GATES.md`
- `docs/PRD_v2/91_REQUIREMENTS_INDEX.md`
- `docs/record_data_tools.md`
- relevant sections of `docs/record_parallel_lanes.md`, `docs/record_petrography.md`, `docs/record_core_depth.md` and `docs/record_fixes.md`
- `docs/takeover/DECISIONS.md`, `CLAIMS.md` and existing evidence receipts
- `REVIEW.md`, `verification/capabilities.json` and `docs/VERIFICATION_MATRIX.md`
- current source, current tests and reachable Git history

### Files this adjudication MUST NOT change

- every read-only governing input above;
- every path under `src/`, `src-tauri/`, `tools/` and `verification/`;
- any file under `docs/PRD_v2/`;
- `REVIEW.md` and its generated matrix.

---

## Evidence Receipt Schema

`docs/takeover/evidence/sb-dio.md` MUST begin with the branch, exact HEAD, accepted anchor, `origin/master`, merge base, date, worktree cleanliness and 63-row guard result. Then give every requirement one heading in numeric order with these fields:

```markdown
## SB-DIO-NNN - exact title

- Chapter evidence: priority, verbatim chapter status, owned test IDs and cited sections.
- Atomic obligations: every independently falsifiable clause in the requirement.
- Current source: exact paths/symbols and what each proves; explicitly name unsatisfied clauses.
- Qualifying acceptance tests: exact path and test sentence, expected-value source and class.
- Supporting tests: tests that help but cannot close the owned contract, with the reason.
- Manual evidence: exact capability ID, checked/total count and state from the generated matrix.
- Git evidence: accepted/reachable commit or `UNIMPLEMENTED`; reachability command and result.
- Verdict: as-built status, release disposition, risk class, test class and commit state.
- Blocker or decision: exact missing evidence, source, dependency or `none`.
- Next action: one bounded production, test, field, decision or no-action increment.
```

Copy each receipt verdict into only these adjudication-owned ledger fields:

```text
as_built_status
release_disposition
risk_class
implementation_paths
test_class
expected_value_source
manual_evidence
dependencies
commit_state
blocking_decision
next_action
last_reverified
```

Use this exact implementation verification stamp:

```text
2026-08-10 @ b332026cb498c105f36eade0bf7899bc0c1309f0
```

The docs-only planning or adjudication commit is not implementation evidence.

---

## Requirement Evidence Map

These are inspection maps, not pre-decided verdicts. Each row must be expanded until every atomic obligation is answered.

### Null state, sentinel and alias contracts (`001`-`009`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `001`, `002` | One project sentinel reaches every registered writer, and the default writer can honor it. | Writer registry and signatures in `export.rs`; default-format selection; T01-T03 bodies; every registered writer and self-read path. | A compile-time signature test does not prove every runtime writer uses the value. |
| `003`, `005`, `006` | `NoNull` is distinct from unset; nulls are plural and channel-specific; exception rules are many-to-many. | `parsers.rs` null types, merge/resolution functions and T04-T05/T09-T10; IPC request shape in `src/ipc.ts`; import-result visibility. | Current source includes `NoNull`, plural lists and six-pattern coverage, but each observable distinction still needs its exact owned test. |
| `004` | One relative-tolerance recognizer is used everywhere and recognition never rewrites a finite sample. | `matches_null`, `is_null_value*`, static search for competing comparisons, T06-T08. | Numeric recognition and preservation are separate sides. |
| `007` | Blank/absent and explicit sentinel-nulled values remain distinguishable through import and export. | Parser representation, stored audit/provenance, writer output and T11. | Both may become NaN for arithmetic; that alone cannot satisfy export distinction. |
| `008`, `009` | Coverage-aware deterministic alias selection is preserved and the chosen/passed-over candidates plus coverage are reported. | Alias tables and scoring in `parsers.rs`, ingest result structs/IPC, T12-T14. | T12 is characterization; deterministic selection and observable reporting are separate obligations. |

Manual capability candidates: `las-import`, `delimited-intake`, `data-conventions`, `generic-curve-store`, `las-export`. Copy only committed capability IDs and actual check counts.

### Index identity and depth-reference contracts (`010`-`014`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `010`, `011` | Structural declaration wins, documented alias namespaces remain separate, and the fired mechanism is recorded. | `resolve_index_column`, all alias constants and source comments, ingest result/IPC, T15-T17. | LAS positional guarantee is characterization; it does not justify positional guessing for delimited tables. |
| `012`, `013` | Non-increasing indexes block with a located row; unresolved delimited indexes require designation and commit nothing. | Index validation/preflight/commit boundary in `ingest.rs`, T18-T19 and clean-control fixtures. | Duplicate depth and non-increasing order are separate decisions; do not let one fixture accidentally satisfy both. |
| `014` | TVD is not consumed as MD; TVD-referenced tops remain identified and MD join/plot refuses without a survey. | MD/TVD alias namespaces, tops import metadata, deviation lookup and user-facing join/plot path, T20-T21. | A static alias test closes only the first half. Any vertical-well fallback must be reconciled with the explicit missing-survey refusal sentence. |

Manual capability candidates: `las-import`, `delimited-intake`, `data-conventions`, `delivery-sets`, `log-view`, `crossplot`.

### Depth-unit and native-sampling contracts (`015`-`023`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `015`, `016` | Undeclared index units refuse; DLIS reads, reconciles and reports its index unit. | LAS/DLIS header extraction, unit-designation preflight, project-unit conversion and T22-T26. | Project unit is not permission to assume the file unit. |
| `017`, `018`, `019` | LAS declares the unit actually written; one canonical-unit registry serves all writers; project-unit changes never silently rescale stored curves. | `export.rs`, `curves.rs`, `units.rs`, project settings command and T27-T31. | Round trip must cover feet and metres. Refusal and explicit migration are alternative compliant behaviors for `019`; record which one ships. |
| `020` | Duplicate depths require a declared policy and the count/result are reported. | Parser/ingest preflight and commit boundary, every policy branch, T32-T33 and malformed exemplar expectation. | Keep-first correctness and no-policy refusal must both be present. |
| `021`, `022` | Read-time resampling is explicit/named/off; write-time Re-grid is accurately named and default off. | Native `curve_samples` ingest path, Reframe boundary, export options and T34-T35. | Native LAS samples and `(set_name, mnemonic)` identity are binding; viewer alignment is not authorization to resample. |
| `023` | Numeric family validation uses sourced physical bounds rather than labels. | Chapter sections 5.6 and 7.1 O-4, any current ENV-owned bounds registry, import preflight and T36-T38 only if cited bounds now exist. | **Hard block:** bounds deliberately ship absent. Never run or write tests against invented limits. |

Manual capability candidates: `las-import`, `dlis-import`, `data-conventions`, `reframe`, `las-export`, `project-lifecycle`.

### Conversion and curve-identity contracts (`024`-`034`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `024`, `025` | Automatic conversion is reported; coverage is declared; unknown units remain verbatim and flagged. | Conversion report fields in `ingest.rs`, canonical/convertible family registry in `curves.rs`, IPC/UI result and T39-T41. | Silent numeric correctness is insufficient; source and destination units plus factor must be visible. |
| `026`, `027`, `028`, `029` | Affine conversion works; ambiguous vendor aliases do not bind; every factor has independent derivation; irreducible ambiguity has no default. | Conversion transform types/registry, quantity typing, source/derivation strings, designation refusal and T42-T45. | Do not promote a neighboring vendor mapping or choose the meaning of `MS/FT`. |
| `030` | Alias rename records raw name, canonical name and firing rule. | Parser alias decision, curve metadata/provenance, ingest result and T46. | A renamed in-memory key without recorded provenance is not enough. |
| `031`, `032` | Missing requested curves stay unavailable; an accepted substitute is explicit and recorded. | All generic fetch/read operations, Reframe substitution validation, ancestry/provenance and T47-T48. | Proving the Reframe path does not prove the universal phrase "any operation, any configuration" in T47. |
| `033`, `034` | Saved selections are named inspectable objects; reads never auto-select by curve type. | `reframe.rs`, saved selection IPC/UI, generic curve-resolution helpers and T49-T50. | A UI chooser does not prove backend readers avoid a type-based fallback. |

Manual capability candidates: `data-conventions`, `generic-curve-store`, `reframe`, `workflow`, `processing-history`.

### DLIS and LAS structure contracts (`035`-`044`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `035`, `036`, `037` | Interval extension and duplicate names require explicit decisions; partial loads name unreadable channels and never report full success. | DLIS preflights, mapping/commit transaction, result schema and T51-T53. | A readable+unreadable synthetic header is supporting evidence unless it traverses the observable import result. |
| `038` | Multidimensional channels import through the published RP66 container with shape, axis labels and units. | `dlis.rs`, array-log store/IPC, RP66 source and T54-T55. | Current source explicitly describes skipping multidimensional channels. Do not reinterpret one-dimensional flattening as conformance. |
| `039` | Per-channel sentinel exceptions preserve genuine values; default screening reports deleted count per channel. | DLIS null rule input, channel screening, result report and T56-T57. | Preservation and counted default screening must both pass. |
| `040` | Wrapped LAS reads correctly; writer emits unwrapped. | LAS parser row assembly, export `WRAP` declaration and T58 plus writer check. | T58 is characterization and tests only the read side unless the writer is separately inspected. |
| `041`, `042` | LAS 3.0 is recognized, every unread section is named, and associated sections are parsed. | Version/section parser, result schema, core/tops stores, CWLS source and T59-T60. | Recognition plus named omission can close `041` while `042` remains absent. Do not merge their verdicts. |
| `043`, `044` | LAS 1.2 reads but is not offered for writing; unknown/out-of-order section strictness is declared, consistent and reported. | Parser version handling, writer format list, section policy and T61-T62. | A tolerant parser without an explicit reported policy cannot close `044`. |

Manual capability candidates: `dlis-import`, `array-logs`, `las-import`, `core-point-import`, `delivery-sets`, `generic-curve-store`.

### Container identity, writers and provenance contracts (`045`-`053`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `045`, `048` | Multi-logical-file containers preview and create separate wells; identity comes from the container, not filename. | DLIS logical-file map, pre-commit preview, well creation and T63-T64/T67. | Multiple sets in one well and multiple wells are different cases. Filename may be a confirmable default but never silently override container identity. |
| `046`, `063` | Missing sidecar dependencies refuse per format with an actionable fix; non-ASCII paths and payloads survive every sidecar boundary. | Python discovery and per-format errors in `python_engine.rs`, `dlis.rs`, `office.rs`, `images.rs`; `sys.stdin.buffer`; T65/T96. | Optional-package tests may be ignored, but ASCII-only success cannot close encoding. |
| `047` | Precision reduction is declared and observable. | Core import numeric types, LAS writer precision, result metadata and T66. | A fixed four-decimal output is not acceptable if truncation is silent. |
| `049`, `050` | Every writer self-reads and rejects invalid output; declared STEP disagreement is detected at import. | Writer registry/self-check, LAS import spacing audit and T68-T70. | One LAS self-read does not prove every registered writer. STEP validation must inspect the full index, not endpoints only. |
| `051`, `052` | Deliverables carry measured/computed/model provenance and mark final versus working curves. | `export.rs` `~O` construction, curve metadata, model lookup/refusal, output file and T71-T74. | Do not weaken the saved-model requirement. Model-derived provenance must be real, not fabricated in the exporter. |
| `053` | Well headers are explicitly mapped, unknown fields preserved verbatim, and missing identity is not synthesized. | LAS header parse/store/preview, filename fallback UI, T75-T76. | Retaining an unknown field and refusing invented UWI are two independent sides. |

Manual capability candidates: `dlis-import`, `las-import`, `las-export`, `office-deliverables`, `image-data`, `processing-history`, `project-lifecycle`.

### Robustness, legacy workbook and encoding contracts (`054`-`063`)

| IDs | Contract boundary | Required evidence | Known caveat |
|---|---|---|---|
| `054`, `055` | Every skipped input item and every omitted export curve is counted and named; all-skipped is an error. | DLIS/LAS result structures, export omission record and T77-T81. | Count-only or name-only reporting is partial. File and user-visible result must agree for export omissions. |
| `056` | Declared STEP is verified against the complete index; irregular sampling emits STEP zero. | Full-index spacing audit in import/export and T82-T83. | Endpoint arithmetic can pass a mixed-step fixture incorrectly. |
| `057` | Exact zeros in a log-scale family are surfaced for confirmation and the recorded user decision governs commit. | Family registry, import preview/decision/commit provenance and T84-T85 only after a cited family-membership source exists. | **Source block O-5:** do not invent which curves are logarithmic. Declining conversion explicitly permits zero values while recording the decision. |
| `058`, `059` | BIFF `.xls` plates use the published drawing-anchor mechanism; cell-only `.xls` reads without drawings. | `images.rs`, `intake.rs`, Python/office boundaries, MS-XLS source and T86-T88. | Modern OOXML support and package-based behavior do not prove old BIFF support from the published specification. |
| `060` | Signature selects the format and structurally disambiguates collisions; disagreement is reported. | Intake signature probe/router, BIFF/ZIP/text cases and T89-T90. | Extension fallback may be supporting behavior but cannot override a recognized signature. |
| `061` | Every reader runs the malformed corpus; failures are bounded, located, counted and named; registration drift fails the build. | `example_data_test.rs`, `tests/fixtures/dio-malformed/**`, registered reader inventory and T91-T94. | Reverify `read_text_file_with_encoding`, `parse_core_csv_with_depth_column`, LAS variants and Intake readers. A test that omits a new reader fails the universal contract. |
| `062` | Text encoding is detected and reported for UTF-8, UTF-16 both byte orders/BOM states and Windows-1252. | `parsers::read_text_file*`, every text reader call site, report field and T95 plus both-side fixtures. | One stray byte must not reject a delivery; direct text reads are nonconforming even if fixtures pass. |
| `063` | Non-ASCII path and payload survive DLIS, Office and image sidecars unchanged. | All three Python runner scripts, byte stdin, JSON serialization and T96. | One sidecar cannot stand in for all three. Keep optional-package dependency state distinct from Unicode correctness. |

Manual capability candidates: `dlis-import`, `las-import`, `delimited-intake`, `image-data`, `petrography`, `office-deliverables`, `las-export`, `verification-stewardship`.

---

### Task 1: Freeze the Evidence Base and Create the 63-Row Receipt Skeleton

**Files:**

- Create: `docs/takeover/evidence/sb-dio.md`
- Read only: all governing and evidence paths listed above

- [ ] **Step 1: Reverify branch, base, origin and cleanliness**

```powershell
git fetch origin --prune
git branch --show-current
git rev-parse HEAD
git rev-parse origin/master
git merge-base HEAD origin/master
git merge-base --is-ancestor b332026cb498c105f36eade0bf7899bc0c1309f0 HEAD
git status --short
git worktree list --porcelain
```

Expected branch: `codex/g1-sb-dio-adjudication`. The accepted anchor remains an ancestor; only `D:\XX. SandiBumi` is a registered worktree; no unexplained change is present. If `origin/master` moved, stop for baseline reconciliation.

- [ ] **Step 2: Re-read governing documents completely**

Read in repository-prescribed order. The plan is not a substitute for live `CLAUDE.md`, the PRD chapter or build record.

- [ ] **Step 3: Run the 63-row baseline guard**

Run the exact guard above, then assert every row is still `UNADJUDICATED`. If another increment has touched a DIO verdict, reconcile rather than overwrite it.

- [ ] **Step 4: Create the receipt header and 63 headings with `apply_patch`**

Include IDs `001` through `063` exactly once, in order. Headings are evidence slots, not verdicts. Do not commit an empty skeleton.

- [ ] **Step 5: Machine-check heading coverage**

```powershell
$headings = Select-String -LiteralPath 'docs\takeover\evidence\sb-dio.md' -Pattern '^## SB-DIO-[0-9]{3} - '
$ids = @($headings | ForEach-Object { [regex]::Match($_.Line, 'SB-DIO-[0-9]{3}').Value })
$expected = 1..63 | ForEach-Object { 'SB-DIO-{0:D3}' -f $_ }
if ($ids.Count -ne 63 -or @($ids | Sort-Object -Unique).Count -ne 63) {
    throw 'The SB-DIO receipt does not contain exactly 63 unique headings'
}
if (@(Compare-Object $expected $ids).Count -ne 0) {
    throw 'The SB-DIO receipt ID set does not match the ledger'
}
```

---

### Task 2: Adjudicate Null, Alias and Index Contracts

**Rows:** `SB-DIO-001` through `SB-DIO-014`

- [ ] Trace the complete writer registry and the complete text/LAS/DLIS import path before classifying universal claims.
- [ ] Open every T01-T21 test body and record whether the expected value is independently sourced, characterization, ignored for a permitted reason, or missing.
- [ ] Run focused test filters for every executable test sentence found. A filter matching zero tests is a missing mapping, not a pass.
- [ ] Pin both sides where required: explicit `NoNull` versus unset; plural per-channel nulls versus another channel; coverage winner versus equal-coverage priority; structural/positional index versus unresolved designation; TVD identity versus MD join refusal.
- [ ] Search for competing null recognizers, alias lists and direct text readers. Confirm negative results in exact files and history.
- [ ] Record actual manual evidence under committed capability IDs without changing the matrix.
- [ ] Write fourteen complete receipt verdicts; do not implement a missing distinction.

---

### Task 3: Adjudicate Depth, Sampling, Unit and Curve-Identity Contracts

**Rows:** `SB-DIO-015` through `SB-DIO-034`

- [ ] Trace declared/undeclared LAS and DLIS index units through preflight, conversion, commit metadata and result reporting.
- [ ] Verify native sample preservation, explicit Reframe, duplicate-depth decisions and export default behavior separately.
- [ ] For T22-T50, inspect exact bodies and expected-value sources. Re-derive only sourced arithmetic such as exact unit conversions; never select a petrophysical bound or ambiguous quantity.
- [ ] For `SB-DIO-023`, record sections 5.6 and 7.1 O-4 as the blocking dependency. Do not execute T36-T38 as correctness evidence unless a current, cited SB-ENV registry exists and is explicitly authorized for this contract.
- [ ] Inventory every conversion family/factor/derivation and every generic curve-resolution path. One Reframe substitution path cannot prove `SB-DIO-031` universally.
- [ ] Verify relevant implementation commits are ancestors of the accepted anchor, then open the accepted source instead of trusting commit messages.
- [ ] Write twenty complete receipt verdicts.

---

### Task 4: Adjudicate DLIS, LAS Structure and Version Contracts

**Rows:** `SB-DIO-035` through `SB-DIO-044`

- [ ] Trace DLIS preview, identity, interval/duplicate decisions, partial-load reporting and atomic commit boundaries.
- [ ] Inspect array-channel storage and IPC. If multidimensional channels are skipped or flattened, record the exact divergence under `SB-DIO-038`; do not treat scalar-channel success as closure.
- [ ] Verify both per-channel sentinel-exception sides and deleted-count reporting.
- [ ] Open T51-T62 and run qualifying focused tests. Separate LAS 3 recognition/named omissions (`041`) from associated-section parsing (`042`).
- [ ] Verify WRAP read and unwrapped write separately, LAS 1.2 read/write-format behavior separately, and declared strictness/reporting for unknown/out-of-order sections.
- [ ] Record normative-source gaps exactly; do not substitute package documentation for RP66 or CWLS requirements.
- [ ] Write ten complete receipt verdicts.

---

### Task 5: Adjudicate Containers, Writers, Provenance and Robustness

**Rows:** `SB-DIO-045` through `SB-DIO-063`

- [ ] Trace multi-logical-file identity and preview into separate wells; distinguish container identity from filename suggestion.
- [ ] Inventory every Python sidecar and optional dependency refusal. Verify `sys.stdin.buffer` and non-ASCII path/payload behavior for DLIS, Office and image paths independently.
- [ ] Inventory every writer and self-reader. Verify precision declarations, STEP audit over the full index, provenance, final/working state and omission reporting in both file and user-visible result.
- [ ] For `SB-DIO-057`, preserve O-5 as a source block unless a cited family-membership registry now exists. Do not invent logarithmic families.
- [ ] For `.xls`, distinguish modern `.xlsx`, BIFF cell tables and BIFF drawing anchors. Verify the specified MS-XLS mechanism or record the gap.
- [ ] Rebuild the malformed-reader inventory from public readers and compare it to `REGISTERED_FILE_READERS`; explicitly include `read_text_file_with_encoding`, `parse_core_csv_with_depth_column`, LAS variants and Intake readers.
- [ ] Open and classify T63-T96, run executable focused tests, and record optional-package ignores honestly.
- [ ] Write nineteen complete receipt verdicts.

---

### Task 6: Update the Ledger Atomically and Self-Review All 63 Rows

**Files:**

- Modify: `docs/takeover/requirements.csv`
- Modify: `docs/takeover/evidence/sb-dio.md`

- [ ] **Step 1: Prepare all 63 changes as one RFC 4180-safe patch**

Use PowerShell only to calculate/review values; make the repository edit with `apply_patch`. Preserve column order, quoting, source-owned fields and every non-SB-DIO row byte-for-byte where practical.

- [ ] **Step 2: Enforce row completeness**

```powershell
$dio = Import-Csv -LiteralPath 'docs\takeover\requirements.csv' |
    Where-Object { $_.requirement_id -like 'SB-DIO-*' }
if (@($dio | Where-Object { $_.as_built_status -eq 'UNADJUDICATED' }).Count -ne 0) {
    throw 'At least one SB-DIO row was silently skipped'
}
foreach ($row in $dio) {
    foreach ($field in @('release_disposition','risk_class','implementation_paths','test_class','commit_state','next_action','last_reverified')) {
        if ([string]::IsNullOrWhiteSpace($row.$field)) {
            throw "$($row.requirement_id) is missing $field"
        }
    }
}
```

Use literal `none` when a nullable evidence field deliberately does not apply.

- [ ] **Step 3: Enforce source-owned-field immutability**

```powershell
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
```

- [ ] **Step 4: Cross-check every receipt verdict against the ledger**

Verify all 63 IDs exactly once; every correctness test names an independent source; every characterization says so; all universal clauses have inventory evidence; all blocks name a dependency; no manual checkbox is promoted; no commit message substitutes for source; and no physical bound, default, conversion meaning or log-family membership was invented.

- [ ] **Step 5: Generate the measured summary**

```powershell
node tools/takeover-ledger.mjs --summary-json
```

Expected mechanical counts: total 931, adjudicated 88, unadjudicated 843. Report actual as-built and release-disposition counts rather than predicting them.

---

### Task 7: Update the One-Minute Handoff

**Files:**

- Modify: `docs/takeover/STATUS.md`

- [ ] Replace planning state with the measured 63-row adjudication state and exact gate counts.
- [ ] Keep Gate 1 `IN PROGRESS`, pilot field evidence `OPEN`, and the worktree-protection statement unchanged.
- [ ] Add one recent-increment row with exact as-built/disposition totals, hard source blocks and `843/931` rows remaining.
- [ ] Name the next serial domain plan only after Jauhar reviews the DIO dispositions. Do not start it.

---

### Task 8: Verify, Commit the Domain Adjudication, and Stop

**Files committed during execution:**

- `docs/takeover/evidence/sb-dio.md`
- `docs/takeover/requirements.csv`
- `docs/takeover/STATUS.md`

- [ ] **Step 1: Run focused tracker and evidence checks**

```powershell
npm run test:takeover-ledger
npm run check:takeover-ledger
node tools/takeover-ledger.mjs --check-prd-audit
node tools/generate-verification-matrix.mjs --check
```

- [ ] **Step 2: Run compile checks in order**

```powershell
npx tsc --noEmit
Push-Location src-tauri
cargo check
Pop-Location
```

- [ ] **Step 3: Run the full gate**

```powershell
powershell -ExecutionPolicy Bypass -File tools\check.ps1
```

Expected: zero failed. Record actual tracker, frontend and Rust pass/ignore counts. A failure outside the three authorized documentation files stops execution; do not edit production code to make an adjudication commit green.

- [ ] **Step 4: Review and stage the exact diff**

```powershell
git diff --check
git diff --stat
git status --short
git add -- docs/takeover/evidence/sb-dio.md docs/takeover/requirements.csv docs/takeover/STATUS.md
git diff --cached --check
git diff --cached --name-only
```

Assert no production, PRD, REVIEW, matrix or unrelated file changed.

- [ ] **Step 5: Commit once and stop**

```powershell
git commit -m "G1-DOM-DIO adjudicate 63 SB-DIO requirements"
git status --short
```

Do not push. Report every row's status/disposition/next action, exact counts, hard evidence blocks, gate counts, files and commit. Do not begin a production fix or the next domain.

---

## Plan Self-Review Before Approval

- [ ] Exactly 63 live SB-DIO IDs are covered once; no ledger row is silently skipped.
- [ ] All ten original P0s and all fifty-three lower-priority rows are adjudicated without treating old priority as pilot policy.
- [ ] The plan changes no production behavior and does not amend PRD v2.
- [ ] Chapter statuses are treated as historical evidence because reachable later commits materially changed the live tree.
- [ ] `SB-DIO-023` remains blocked on section 7.1 O-4; T36-T38 cannot pin absent physical bounds.
- [ ] `SB-DIO-057` does not acquire an invented log-scale family registry.
- [ ] RP66, CWLS and MS-XLS normative-source gaps remain gaps until the exact acquired source supports the specified mechanism.
- [ ] Native sampling, explicit Reframe and `(set_name, mnemonic)` identity are preserved.
- [ ] No direct text reader or `sys.stdin` sidecar is accepted as conforming.
- [ ] No helper/internal-only test closes a user-observable reporting contract.
- [ ] Every expected value is sourced or explicitly characterized; no petrophysical value is selected.
- [ ] Universal writer, reader, sidecar and curve-resolution claims require full inventories, not one example.
- [ ] Git reachability, test evidence, manual evidence and pilot field evidence stay separate.
- [ ] `release_disposition` and `as_built_status` cannot be read as synonyms.
- [ ] The plan predicts only the mechanical 88 adjudicated / 843 remaining result, not verdict totals.
- [ ] The plan commit changes zero ledger verdict rows; execution starts only after Jauhar approves it.
