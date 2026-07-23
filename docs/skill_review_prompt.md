# SandiBumi A-to-Z skill-driven review prompt

A reusable prompt for reviewing SandiBumi against **the 45 installed Claude skills** — Jauhar's
own delivered-project playbooks (`proj-*`), the petrophysics reference library (`petro-*`),
the vendor-operation skills (`sw-*`, `geolog-loglan*`), and the reservoir-engineering
downstream skills (`pe-*`) — plus **Track F**, an engineering-craft track that exists because
those 45 skills are all geoscience and none of them covers code quality (see §2, Lens E).

**This is not a second copy of `docs/qc_audit_prompt_template.md`.** That prompt audits one
tool at a time along *engineering* dimensions (DB → backend → frontend → UX → integration) and
checks the code against **SandiBumi's own `docs/`**. It has no authority when a tool has no
spec doc, and by construction it cannot catch a spec doc that is itself wrong. This prompt
reviews along *domain* dimensions and checks the code against **independent external
knowledge**, so it catches a different defect class:

| Defect class | Caught by the QC prompt | Caught by this prompt |
|---|---|---|
| Wiring, NaN leaks, race guards, well-group scoping | ✅ | — |
| Code departs from its own spec doc | ✅ | ✅ |
| **The spec doc itself is wrong** | ❌ (doc is the authority) | ✅ |
| **Module has no spec doc at all** | ❌ (explicitly out of scope) | ✅ |
| **A constant with no source anywhere** | partly | ✅ (highest severity) |
| **A step in a real delivered study the app can't do** | ❌ | ✅ (`proj-*` lens) |

Run both. They are complementary, not alternatives.

## Why this is staged, not one pass

"A to Z in one go" is not achievable in one context window — SandiBumi is ~40 backend modules,
~30 panels, and 45 skills whose reference notes run to hundreds of pages. A single pass would
produce generic findings and miss everything specific. So the sweep is **sharded by domain**,
one pass per session, each pass loading only the 1–4 skills that are the authority for it.

**How to use:** pick one pass from §3, fill the `{{...}}` placeholders in §1 from that pass's
row, and run it as a fresh Claude Code session. Findings accumulate in
`docs/review_sweep/REGISTER.md`. Fixes are **not** part of a review pass — they land afterward
as normal serial increments under the playbook acceptance bar (`docs/playbook_build_progress.md`).

---

## 1. The master prompt (copy, fill in `{{...}}`, run)

```
Run review pass "{{PASS_ID}} — {{PASS_NAME}}" over SandiBumi at D:\XX. Arshilla.

Authority skills for this pass: {{SKILLS}}
SandiBumi surfaces in scope: {{SURFACES}}
SandiBumi spec docs for these surfaces: {{DOCS}}   (or "none" — if none, say so; do not invent one)

## Step 0 — Load the authority BEFORE reading any SandiBumi code

This ordering is mandatory and is the point of the whole exercise. If you read the
implementation first you will rationalise whatever it does. So:

1. Invoke each skill in {{SKILLS}} with the Skill tool.
2. Read its `references/*.md` files, not just SKILL.md — SKILL.md is a router; the equations,
   constants, cutoffs and step-by-step methods live in `references/_overview.md` and the
   per-topic notes under C:\Users\ARUNIKA\.claude\skills\<skill>\references\.
3. Write down, before opening the repo, a numbered EXPECTATION LIST: what a correct
   implementation of this domain must do — the equations, the constant values with their
   ranges, the required inputs, the edge cases, the QC artifacts, the workflow order. Cite the
   reference file and section for each line. 10-30 lines is the right size.

Only then read the SandiBumi code.

## Step 1 — Don't re-report what is already known

Grep REVIEW.md, ROADMAP.md, docs/playbook_build_progress.md and AUDIT-*.md for this pass's
module and method names. REVIEW.md is at Round 57+; the playbook tracks deliberate deferrals.
A recorded deferral is an explicit decision by Jauhar, not an oversight — do not report it as a
bug. If you think a past decision is now wrong, say so as a decision to revisit, and say what
changed.

## Step 2 — Three-way comparison, per expectation-list line

For each line, compare: what the SKILL says / what the SandiBumi DOC says / what the CODE does.
Classify into exactly one class. The classes are the deliverable — a finding without a class is
not a finding:

- MATCH — all three agree. Report nothing.
- CODE-BUG — code departs from both the skill and the doc. Standard defect.
- DOC-BUG — code faithfully implements the doc, but the DOC departs from the skill. The defect
  is in docs/, and it is more dangerous than a code bug because every future audit against that
  doc will bless the wrong behaviour. Say which is right and why.
- UNSOURCED — a constant, threshold, coefficient or cutoff in the code that appears in neither
  the doc nor any skill reference. This project's standing rule is that fabricated or
  unverified physics constants must never ship, so this is the highest-severity class.
  Cross-check docs/constants_verification_2026-07-22.md before raising one.
- HOUSE-DEPARTURE — code and doc agree with each other and deliberately depart from the
  textbook for a real reason (Mahakam fresh-water muds, LRLC, thin beds, Jauhar's field
  standards: GRN P3/P97, LRLC IMTS/RtC, SSC/SSPW). This is legitimate. The finding is only
  whether the rationale is WRITTEN DOWN somewhere; if it is not, the finding is a documentation
  task, not a physics bug.
- WORKFLOW-GAP — (proj-* passes) a step in a study Jauhar actually delivered that SandiBumi
  cannot perform, or can only perform by leaving the app.
- METHOD-GAP — the skill covers a method SandiBumi does not implement at all. This is a
  ROADMAP candidate, NOT a bug. Do not let gaps dominate the report; cap them and rank them.
- NO-AUTHORITY — the skills are silent on something the code decides. Record it explicitly.
  Skill silence is not approval, and naming these tells Jauhar where his reference library has
  a hole.

## Step 3 — Who wins when they disagree

The skill is an authority, not an oracle. Precedence when sources conflict:

1. `proj-*` skills (Jauhar's own delivered studies) — highest authority on what the WORKFLOW
   and the practical parameter choices must be. If a textbook skill and a proj-* skill
   disagree, the proj-* skill describes what actually shipped to a client.
2. SandiBumi `docs/` house method notes + Jauhar's field standards — authority on deliberate
   house choices.
3. `petro-*` / `pe-*` / `sw-*` skills — authority on general physics and vendor behaviour.

A conflict is a QUESTION for Jauhar, not automatically a bug. Frame it as: "skill X says A,
doc Y says B, code does B; here is why I think A/B is right; your call."

## Step 4 — Refute yourself before reporting

Follow the project's house QC method: adversarially try to REFUTE every finding before it goes
in the report. A finding survives only if you could not talk yourself out of it. For each
survivor, state how you tried to refute it and why that failed.

Specifically try to refute by: re-reading the code path (is the check done upstream?); checking
whether a caller already guarantees the precondition; checking git log / REVIEW.md for whether
this was a deliberate change; and re-reading the skill reference to confirm you are not quoting
it out of its stated applicability range.

## Step 5 — Report

Write docs/review_sweep/{{PASS_ID}}.md with:

(a) The expectation list from Step 0, each line marked with its class.
(b) One block per surviving finding:

    Title / Class / Severity (High/Med/Low) / Effort (S/M/L) /
    Where (file:line) / Skill authority (exact reference file + section) /
    SandiBumi doc position (file + section, or "no doc") /
    Evidence (what the code actually does — quote it) /
    Suggested fix / Refutation attempted and why it failed

(c) A one-paragraph verdict: is this domain in good shape or not, plainly stated.
(d) Append one row per finding to docs/review_sweep/REGISTER.md.

## Rules

- Report only findings that survived refutation. If the domain is clean, say so plainly —
  do not manufacture low-value findings to fill the report.
- Never propose a constant, threshold or coefficient without citing the specific skill
  reference file and section it came from. "Typical values are..." with no citation is exactly
  the failure mode this project forbids.
- Review only. Do not edit code in this pass. Fixes land afterward as separate serial
  increments in the main working tree, each with tsc + cargo test + a REVIEW.md entry.
- Distinguish "SandiBumi should do this" from "SandiBumi v1 should do this". v1 is openhole
  petrophysics; pore pressure, geomechanics, seismic, simulation, NMR and image logs are Later
  unless the roadmap says otherwise. Do not let a skill's breadth inflate v1 scope.
- If context runs short, stop at a clean boundary, write what is complete, and state the
  resume point.
```

---

## 2. Skill inventory — which of the 45 actually get used, and as what

Using all 45 as review authorities would be wrong: nine `sw-petrel-*` skills cover 3D
structural modelling, prestack seismic and simulation, which SandiBumi does not do. They are
listed here so a future run does not waste a pass rediscovering that.

### Lens P — Practice fit (8 `proj-*`) · **highest authority, run these first**
Distilled from Jauhar's ingested delivered projects. They answer *"can SandiBumi run the study
that was actually shipped to the client?"* — a capability question the other lenses cannot ask.

`proj-shaly-sand-sw` (10 projects) · `proj-sand-silt-clay-thinbed` (6) · `proj-lqr-low-quality-reservoir` (5) ·
`proj-lrlc-low-contrast-pay` (4) · `proj-carbonate-eval` (8) · `proj-ggr-regional-exploration` (4) ·
`proj-clastic-conventional-and-gas` (4) · `proj-specialized-geomechanics-geothermal` (2)

### Lens D — Domain physics (11 `petro-*`) · correctness authority
`petro-well-log-fundamentals` · `petro-log-qc-normalization` · `petro-porosity-lithology` ·
`petro-shaly-sand-saturation` · `petro-sand-silt-clay` · `petro-lrlc-lrp-screening` ·
`petro-thin-bed-anisotropy` · `petro-saturation-height` · `petro-core-scal-integration` ·
`petro-formation-eval-general` · `petro-reservoir-geology-modeling`

### Lens V — Vendor parity (8 used of 17 `sw-*`/`geolog-*`/`mudlog-*`) · behaviour + naming authority
SandiBumi's users are Geolog/Techlog/IP-trained. These define what those users will expect.

`sw-geolog-conditioning-multimin` · `sw-geolog-porosity-perm` · `sw-geolog-facies-synthetic` ·
`sw-geolog-fluid-id-mifi` · `sw-geolog-core-python` · `sw-techlog` ·
`geolog-loglan` + `geolog-loglan-workspace` (scripting-surface authority) · `mudlog-litho-to-las`

### Lens C — Downstream handoff (4 used of 8 `pe-*`, + 4 `sw-petrel-*` as consumers)
These do **not** review SandiBumi's internals. They define what SandiBumi's *exports* must
carry to be usable by whoever receives them.

Used: `pe-volumetrics-reserves` · `pe-rock-fluid-properties` · `pe-pvt-fluid-properties` ·
`pe-cbm-unconventional` (only for the shipped TOC/GIP/CBM modules).
Consumer view only: `sw-petrel-volumetrics-contacts` · `sw-petrel-property-modeling` ·
`sw-petrel-wells-correlation-design` · `sw-petrel-static-model-qc`.

### Not review authorities — consult on demand only (9)
`pe-reservoir-simulation` · `pe-well-test-analysis` · `pe-production-completion` ·
`pe-reservoir-performance-recovery` · `sw-petrel-reservoir-simulation` ·
`sw-petrel-structural-modeling` · `sw-petrel-prestack-psi-processing` ·
`sw-petrel-qi-avo-rockphysics` · `sw-petrel-data-import-visualization`

Out of SandiBumi v1 scope. Open one only if a specific finding needs it.

### Lens E — Engineering craft · **no installed skill authority (verified 2026-07-24)**

All 48 skills on this machine are geoscience: the 45 in `~/.claude/skills` plus three in the
plugin marketplace cache (a duplicate `geolog-loglan`, `petropy`, `well-log-evaluation`). There
is **no Anthropic-authored frontend / UX / code-optimization skill installed**, and none in the
marketplace cache. So Lens E has no skill to load, and Track F below substitutes three other
authorities:

1. **`/code-review ultra`** — the multi-agent branch review. **Jauhar triggers it; Claude cannot.**
   This is the right instrument for diff-scoped code craft (bugs, idiom, naming, structure of
   what changed). Run it BEFORE F1/F2 so a pass does not re-find what it already found.
2. **The `code-simplifier` subagent** — clarity/consistency/maintainability preserving behaviour.
   Useful inside F1/F2 on a single named file, not as a whole-repo sweep.
3. **The checklists written into each F pass below** — these encode SandiBumi's own accumulated
   conventions (the 15-var CSS contract, lazy-chunk discipline, dispose symmetry, race guards).

**Why Track F exists at all, given `/code-review ultra`:** ultra reviews a *diff*. It cannot see
app-wide invariants — that all ~30 panels honour the theme contract, that every `DomPanel`
unsubscribes what it subscribed, that the main bundle has not grown. Those are whole-app
properties and need a whole-app pass. F1/F2 are ultra's territory and are cheap follow-ups;
**F3/F4/F5 are the ones ultra structurally cannot do.**

`artifact-design` / `artifact-capabilities` are the only Anthropic skills present. They build
published web pages and are **not** review authorities — relevant only if a sweep report is to
be published as a shareable artifact.

---

## 3. Pass schedule (fill `{{...}}` from here)

Surfaces are `src-tauri/src/*.rs` unless noted. The per-tool file/command inventory is in
`docs/qc_audit_prompt_template.md` §2 — do not duplicate it here, and re-derive it with a grep
if it has gone stale.

### Track A — Practice fit (run first; these reframe everything after)
| Pass | Skills | Surfaces | Docs |
|---|---|---|---|
| A1 Workhorse chain | proj-shaly-sand-sw, proj-clastic-conventional-and-gas | end-to-end walk: gr_normalize → vsh_gr → ssc/sspw → sw_sim → cutoffs → sw_height (modules.rs, ssc.rs, workflow.rs, satheight.rs) | workflow_standards.md, method_ssc_sspw.md |
| A2 Thin-bed & LRLC | proj-sand-silt-clay-thinbed, proj-lrlc-low-contrast-pay | ssc.rs, lrlc.rs, thin_bed_ts | method_ssc_sspw.md, method_lrlc_rtc_imts.md, research_2026-07/ref_thin_bed_lrlc.md |
| A3 LQR cutoffs & tiering | proj-lqr-low-quality-reservoir | workflow.rs (pay summary, cutoff sweep), rocktyping.rs, resultsqc.rs | workflow_standards.md, ref_rock_typing.md |
| A4 Carbonate | proj-carbonate-eval | multimin2.rs, variable-m paths, rocktyping.rs, satheight.rs | multimin_ref_spec.md |
| A5 Regional / multi-well | proj-ggr-regional-exploration | gr_normalize reference-subset, log_predict, well_groups, chain.rs, report.rs batch | workflow_standards.md |
| A6 Specialized (scope check only) | proj-specialized-geomechanics-geothermal | brittleness/TOC/GIP modules; confirm nothing is half-wired or implied-but-absent | ref_unconventional.md |

### Track B — Domain physics
| Pass | Skills | Surfaces | Docs |
|---|---|---|---|
| B1 Log physics & conditioning | petro-well-log-fundamentals, petro-log-qc-normalization | parsers.rs, ingest.rs, dlis.rs, curve_edit.rs, Prep modules (badhole, condflag, *_hole_corr, nphi_env_corr, gr_normalize, depth_shift, splice, ftemp_grad, precalc) | workflow_standards.md, research_2026-07/ref_kkt_onwj_wave_e.md |
| B2 Porosity & lithology | petro-porosity-lithology, sw-geolog-porosity-perm | phi_den/phi_dn/phi_son/phimax, nphimat, gascorr, neutron_charts.rs, chartOverlays.ts, perm_* | none (flag it) |
| B3 Shaly-sand saturation | petro-shaly-sand-saturation | sw_arch, sw_indo, sw_sim, dual-water, Waxman-Smits, Rw/temperature, pickettPanel | none (flag it) |
| B4 Multimineral | petro-porosity-lithology, sw-geolog-conditioning-multimin, sw-techlog | multimin2.rs: endpoints, X/U zoning, RECON, dof guard, dry clay, fluid calc, core RMS | multimin_ref_spec.md, multimin_ip_spec.md |
| B5 Core, SCAL & SwH | petro-core-scal-integration, petro-saturation-height, pe-rock-fluid-properties, sw-geolog-core-python | rocktyping.rs, lorenz.rs, satheight.rs, facies_tie.rs, core_data/scal_pc | ref_rock_typing.md, ref_shf.md |
| B6 Facies, zonation & FE integration | petro-reservoir-geology-modeling, petro-formation-eval-general, sw-geolog-facies-synthetic | facies.rs, ml.rs, log_predict, tops/zones, correlationPanel, netflag.rs | none (flag it) |

### Track C — Vendor parity
| Pass | Skills | Surfaces | Docs |
|---|---|---|---|
| C1 Geolog parity | the 5 sw-geolog-*, geolog-loglan, geolog-loglan-workspace | naming/behaviour across Prep+multimin+facies; equations.rs, python_engine.rs scripting semantics | research_2026-07/ip_ingest/* |
| C2 Techlog / IP parity | sw-techlog | families/aliases in parsers.rs, Quanti/Elan correspondence, montecarlo.rs (+ ref_monte_carlo_seeds.md) | ref_monte_carlo_seeds.md, techlog_ingest/FINDINGS.md |
| C3 Lithology & mudlog ingest | mudlog-litho-to-las | aux_data ingest, composite.rs lithology patterns (the open #9A hatch residual) | none (flag it) |

### Track D — Downstream handoff
| Pass | Skills | Surfaces | Docs |
|---|---|---|---|
| D1 Volumetrics & reserves | pe-volumetrics-reserves, pe-rock-fluid-properties, pe-pvt-fluid-properties | workflow.rs pay summary, montecarlo.rs P10/P50/P90, report.rs, dashboardPanel | workflow_standards.md |
| D2 Export & model handoff | the 4 sw-petrel-* consumer skills | export.rs (LAS), deviation.rs, geo.rs, tops export, NTG/upscaling readiness of outputs | none (flag it) |

### Track F — Engineering craft (Lens E; no skill to load — use the checklist in each row)

Substitute Step 0 for these passes: instead of loading a skill, **restate the row's checklist as
the expectation list**, then read the code. The finding classes still apply, with three of them
re-read for a craft context: `CODE-BUG` = a defect; `HOUSE-DEPARTURE` = a deliberate convention
(e.g. `linear_dw` stays default, module runs are deliberately not undoable) that must be written
down; `NO-AUTHORITY` = a judgement call with no convention behind it — name it so a convention
can be decided.

| Pass | Checklist | Surfaces | Instrument |
|---|---|---|---|
| F1 Frontend architecture | file size vs responsibility (which dialogs have outgrown one module); duplication across dialogs that should be a shared helper; `ipc.ts` types actually matching the Rust structs they mirror; `any`/non-null-assertion leakage; dead exports; consistent async/error shape | `src/ui/*.ts`, `src/ipc.ts`, `src/state.ts`, `src/workspace.ts` | run `/code-review ultra` first; then `code-simplifier` per named file |
| F2 Rust idiom & hot paths | needless clone/alloc inside per-sample loops; `unwrap()`/`expect()` on any user-data path; error-type consistency (`Result<T,String>` at the IPC edge vs typed inside); rayon compute / single batched DB-write separation; `Mutex<Connection>` hold time; confirmed dead code (`petrophysics.rs` orphan, `inversion.rs` stub) quarantined or deleted | `src-tauri/src/*.rs` | run `/code-review ultra` first; then `code-simplifier` per named file |
| F3 UX & theming sweep | **app-wide**: every panel/dialog against the 15-var CSS contract (no raw hex, no hardcoded font); render under dark + light + Pertamina/Halliburton/SLB/LAPI-ITB; `themeVersion` subscription on every canvas panel; dialog layout/label consistency; error-message specificity; empty + loading + no-well states; keyboard/aria beyond the 9D base | all `src/ui/*`, `src/styles.css` | this prompt only — ultra cannot see it |
| F4 Build & bundle health | main `index` bundle has not grown (baseline **1,125.01 kB**); every heavy panel still a lazy chunk (vega, codemirror, dialogs); the 7 open `npm audit` high advisories in vega deps; unused deps/assets; tsc strictness settings actually on | `vite.config.ts`, `package.json`, `tsconfig.json`, build output | this prompt only — ultra cannot see it |
| F5 Lifecycle & leaks | **app-wide**: every `DomPanel` unsubscribes what it subscribed (dispose symmetry); `dataVersion`/`themeVersion` subscription pairs; a generation counter on every async reload; listener accumulation across close→reopen; `filterByActiveGroup` present in every batch dialog (the backend enforces no group scoping at all) | all `src/ui/*`, `src/workspace.ts` | this prompt only — ultra cannot see it |

### Track Z — Synthesis (run last)
| Pass | Input | Output |
|---|---|---|
| Z1 Consolidate | every `docs/review_sweep/*.md` | dedupe across passes, rank by severity ÷ effort, split into: fix-now increments / ROADMAP additions / doc corrections / questions for Jauhar. Propose diffs against ROADMAP.md and the playbook — never a rewrite. |

---

## 4. Sequencing

1. **A1 first, always.** It walks a real delivered study end to end and will reframe every
   later pass — a formula that is right in isolation but unreachable in the actual workflow is
   a bigger problem than a coefficient in the third decimal.
2. **A2 → B4 → B3.** The densest physics and the most recently rebuilt code (SSC/LRLC,
   SandiMin, saturation), and the areas with real spec docs to check against.
3. **B1, B5, B6, A3.** Broad surface, moderate depth.
4. **A4, A5, C1, C2.** Parity and coverage.
5. **C3, D1, D2, A6.** Narrow scope, cheap.
6. **Track F any time** — it is independent of the domain tracks and does not need their
   findings. Order within it: run `/code-review ultra` first, then **F3 → F5 → F4** (the three
   ultra cannot do), then F1/F2 only for what ultra's diff scope missed.
7. **Z1 last.**

**Minimum viable sweep** if the full 22 passes is too much: run **A1, A2, B3, B4** on the domain
side and **F3 + F5** on the craft side, then **Z1**. That covers the workhorse chain, the two
hardest physics modules, Jauhar's two signature methods, and the two app-wide invariants that no
other instrument checks — which is where a real defect is both most likely and most costly.

---

## Notes (outside the prompt)

- **The `proj-*` lens is the genuinely new capability here.** Every prior review instrument —
  the QC audit prompt, REVIEW.md, the AUDIT files — asks "is this code correct?". None of them
  asks "could Jauhar have delivered the Duri study in this app?". Those 8 skills encode ~43
  studies that actually shipped, so they are the only available ground truth for workflow
  completeness. That is why Track A runs first and outranks the textbook skills on conflict.

- **DOC-BUG is the class that justifies this prompt existing.** `qc_audit_prompt_template.md`
  states plainly that `docs/` wins over code — correct for an engineering audit, but it means a
  wrong spec doc is unfalsifiable under that instrument. An external authority is the only
  thing that can find one.

- **Deliberately not included:** a fix loop. Review passes produce findings; fixes go through
  the existing per-increment discipline (small steps → tsc + cargo check + cargo test →
  REVIEW.md entry with a real-data "Try:" line → commit). Merging the two would produce
  half-verified fixes inside a report, which is how a review becomes a liability.

- **Track F was added because the skill library has no engineering half.** 45 domain
  authorities and zero craft authorities means the sweep could confirm every equation and still
  miss that a dialog has outgrown its module or that a panel leaks listeners. The honest fix is
  not to pretend a petrophysics skill covers it — it is to name the gap, point F1/F2 at
  `/code-review ultra` (which is built for exactly that and which Jauhar must trigger), and keep
  F3/F4/F5 here because they are app-wide invariants a diff review cannot see.

- **Writing house engineering skills was considered and deferred.** The pipeline that built the
  petro skills exists, but its input was a petrophysics PDF library; there is no engineering
  source material on disk to distil. That makes it a sourcing project, not a build step. Revisit
  if Track F keeps finding the same class of thing.

- **Skill count is 45 as of 2026-07-24** (8 proj-*, 11 petro-*, 8 pe-*, 5 sw-geolog-*,
  9 sw-petrel-*, sw-techlog, geolog-loglan, geolog-loglan-workspace, mudlog-litho-to-las), plus
  3 in the plugin marketplace cache. Re-derive §2 if that changes; the tier assignment is a
  judgement about SandiBumi v1 scope, not a property of the skills, so it needs revisiting if v1
  scope moves. **Re-check the Lens E claim too** — if an Anthropic engineering skill bundle is
  ever installed, Track F should load it instead of relying on its inline checklists.
