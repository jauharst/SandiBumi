# Stewardship prompts — keeping SandiBumi and SegaraBumi hireable

Prompts for the layer **above** day-to-day work: the structural health of the codebase, the
decisions nobody wrote down, and the question that decides whether this stays a one-person
project forever — *could a software engineer I hire be productive here in week one?*

**This file does not overlap with the others.** Keep the boundary sharp:

| File | Scope | Cadence |
|---|---|---|
| `maintenance_scaling_prompt.md` | one increment: expand / debug / maintain | every session |
| `engineering_review_prompt.md` | whole-app *behaviour* sweep, tiers F1–F5 | occasional, produces findings |
| `qc_audit_prompt_template.md` | one tool, end to end, physics correctness | per tool |
| **this file** | whole-repo *structure*, decisions, onboarding | quarterly, plus once before any hire |

The distinction that matters: the other three ask **"is this correct?"** This one asks
**"is this survivable by someone who is not me?"** A codebase can be 100% correct and still be
unhireable, and that is the failure mode that kills vibe-coded products — not bugs.

---

## 0. Measured baseline — 2026-07-25

Recorded so future audits measure *drift* rather than re-arguing from scratch. Re-measure with
Prompt 3; do not trust these numbers after a few months of work.

**SandiBumi** (`D:\XX. SandiBumi`)

| Metric | Value | Reading |
|---|---|---|
| Rust source files | 44 | |
| Rust `#[test]` / `#[cfg(test)]` sites | 426 | the physics has a real regression net |
| TS / Svelte files | 67 | |
| Frontend tests | **0** | no vitest, no harness in-tree |
| Lint config (eslint / clippy / rustfmt) | **none** | conventions live in prose, unenforced |
| CI | **none** | every gate is manual and machine-local |
| `ARCHITECTURE.md` | **absent** | `CLAUDE.md` is doing two jobs at once |
| `docs/adr/` | **absent** | rationale is scattered across `REVIEW.md` rounds |
| Largest files | `modules.rs` 3,847 · `multimin2.rs` 3,829 · `montecarlo.rs` 2,254 · `db.rs` 2,048 · `crossplotPanel.ts` 2,016 · `workflow.rs` 1,912 · `parsers.rs` 1,838 · `ipc.ts` 1,840 | |

**What that table actually says.** The tested half and the buggy half are different halves. Every
fault class in `REVIEW.md` that cost real time — wrong well's data on screen, stale rows, editor
lifecycle leaks — sits in the 67 files with no tests, while the 426 tests defend the code that was
already the most carefully written. That is not a criticism of the past; it is the highest-value
target for the next year.

**SegaraBumi** — design complete (`docs/research_2026-07/sonar_ingest/`), **no code yet**.
Everything in Prompt 1 applies to it and to nothing else. Once it has a source tree, add its own
baseline row here.

---

## Prompt 1 — CHARTER (greenfield: SegaraBumi, before the first line of code)

The cheapest prompt in this file by a wide margin. Run it *once*, at the start.

```
I am about to start building SegaraBumi as a standalone Rust workspace. The design is complete
and lives in D:\XX. SandiBumi\docs\research_2026-07\sonar_ingest\ (B_gap_matrix, C_data_foundation,
D_type_registry, E_indexer_search, F_digitization_design, FINDINGS). Read those first, plus
docs/sonar_ingest_adopt_prompt.md.

Do NOT write feature code in this session. Write the charter that every later session obeys.

## Step 1 — Draw the boundaries before anything can cross them

Propose the crate/module split and, for each boundary, state what is FORBIDDEN to cross it.
The design already fixes three of these; hold them:

  - core library crate (indexer, type registry, dictionaries, physics validator) --- the CLI and
    the UI both depend on it; it depends on neither. If core ever needs to know a UI concept,
    the boundary was drawn wrong.
  - CLI and thin UI as separate crates over that core.
  - Tier 1 (deterministic, mandatory, air-gap-capable) and Tier 2 (optional local-LLM
    enrichment). Tier 2 may READ Tier-1 facts and may WRITE ONLY to its own columns. Tier 1 must
    compile, run and pass its whole test suite with the Tier-2 crate absent entirely.

Make that last one structural, not a rule in a document. Tier 2 behind a cargo feature flag, or
its own crate that Tier 1 does not depend on, so violating it is a compile error rather than a
code-review catch. A rule a machine enforces survives; a rule in a README does not.

## Step 2 — Freeze the two contracts SandiBumi will later depend on

SandiBumi integrates later via the core crate API and/or the shared .db file. Write both down
now, in docs/CONTRACT.md, with a version number:
  (a) the SQLite schema, including which columns are Tier-1 facts and which are Tier-2 output;
  (b) the public crate API surface SandiBumi is allowed to call.
State the rule for changing them: additive changes bump minor, anything else needs an ADR.
These are the only two things that are expensive to change later. Everything else is cheap.

## Step 3 — Write the enforcement before the features

In this order, and it must be running before feature work starts:
  1. tools/check.ps1 -- fmt, clippy (warnings as errors), test, in one command, exit non-zero on
     any failure. This IS the quality gate; there will be no CI at first and that is deliberate.
  2. rustfmt.toml and clippy.toml, committed.
  3. One end-to-end test that indexes a fixture directory and asserts the search result -- even
     if it indexes three files. The first test is hard; the hundredth is free.
  4. Test fixtures drawn from the real corpora (the 6,668 pooled LAS in project-kb, the 36
     Techlog training LAS). Small, committed, real -- not synthetic.

## Step 4 — Start ARCHITECTURE.md and docs/adr/ on day one

ARCHITECTURE.md: the 30-minute map for a new engineer -- what each crate does, how a file gets
from disk to a search result, where the data lives.
docs/adr/0001-*.md onward: one file per decision that a competent newcomer would otherwise
"fix". Start with the ones already made: standalone-first rather than a SandiBumi module; two
tiers rather than zero-AI; rule IDs rather than embedding thresholds; XLSX/CSV as the only v1
entry surface with no bespoke per-report editors.

Each ADR: Context / Decision / Consequences / Status. Half a page. Never rewrite one -- if the
decision changes, write a new ADR that supersedes it, and link both. The superseded ones are
the point: they record what was already tried.

## Step 5 — Propose, wait

Show me the boundary map, the contract sketch and the ADR list before creating files.
```

**Why this is the highest-leverage prompt here.** Every structural problem in the baseline table
above — no lint, no frontend tests, no CI, no ARCHITECTURE.md, no ADRs — costs perhaps two days to
add to an empty repository and perhaps two months to retrofit into a working one, because
retrofitting means changing code that already works, which means risking correctness you already
paid for. SegaraBumi is currently free. It will not be free again.

**The Tier-1/Tier-2 point is worth the whole prompt.** "Tier 2 never overwrites Tier-1 facts" is
currently a sentence in a design document. Sentences in design documents get violated by helpful
future contributors — including future Claude sessions, including you at 1 a.m. A cargo feature
flag cannot be violated by accident.

---

## Prompt 2 — STEWARDSHIP PLAN (once per project, produces documents not code)

Run once per project. Re-run only after a large change of direction.

```
Produce the stewardship documents for {{SandiBumi at D:\XX. SandiBumi | SegaraBumi at <path>}}.
Write documents only -- no behaviour changes in this session.

Read first: CLAUDE.md, AGENTS.md, CONTRIBUTING.md, README.md, ROADMAP.md, REVIEW.md,
docs/maintenance_scaling_prompt.md, docs/playbook_build_progress.md, and enough of the source to
be accurate rather than plausible. Where a document and the code disagree, the CODE is the fact
and the document is the bug -- note every such disagreement, do not silently follow either.

## Deliverable 1 -- ARCHITECTURE.md (new, repo root)

The map a new engineer reads in 30 minutes, before touching anything. Sections:
  - What this software is, in one paragraph, in domain terms (petrophysics, not "a Tauri app").
  - The layers and what each owns: Svelte/TS UI, the Tauri IPC boundary, the Rust module system,
    the solvers, DuckDB persistence. For each, one sentence of "this layer must not ...".
  - The two or three journeys that explain the whole system. At minimum: a LAS file from import
    to a computed curve on screen; a module run from parameter dialog to persisted result.
    Name the actual files and functions each journey passes through.
  - The module manifest explained -- what modules.rs is, why adding a method is a manifest entry
    and not UI code. This is THE mechanism that makes the app growable; a newcomer who does not
    understand it will build a bespoke panel and permanently widen the maintenance surface.
  - Where the data lives, and the write-discipline rules that are not enforced by the schema.

Aim for accuracy over completeness. A short true document beats a long half-true one.

## Deliverable 2 -- docs/adr/ (new)

Mine REVIEW.md, CLAUDE.md, ROADMAP.md, the AUDIT files and the code comments for decisions that
look like bugs to someone who was not there, and write one ADR each. The known set to start:
  - computed_curves is deliberately PK-less; uniqueness is upheld by write discipline; no
    upsert/ON CONFLICT path may assume a primary key.
  - DuckDB is single-writer by design (Mutex<Connection>) -- fundamental, not a defect; scaling
    comes from fewer bigger writes, not more threads.
  - Module runs are deliberately not undoable.
  - The GPU is render-only and is not a compute resource.
  - The backend does not enforce well-group scoping (and what upholds it instead).
  - The MSVC 14.29 toolchain pin (14.50 is broken -- missing clui.dll).
Search for more; those are the ones already known. Each ADR: Context / Decision / Consequences /
Status, half a page, dated, never rewritten -- superseded by a new one if it changes.

## Deliverable 3 -- docs/STEWARDSHIP.md (new)

The operating manual for keeping this healthy:
  - The measured baseline (re-measure it, do not copy mine).
  - The budgets and thresholds of Prompt 3, with today's values and the agreed limits.
  - The cadence: what gets run, how often, and what triggers it early.
  - The escalation rule: which findings are fix-now (data honesty, wrong numbers) and which
    queue (cosmetic, ergonomic).

## Deliverable 4 -- the split of CLAUDE.md

CLAUDE.md currently serves two audiences: instructions for an AI agent, and a description of the
system for a human. Once ARCHITECTURE.md exists, CLAUDE.md keeps only what an agent needs to act
correctly (conventions, commands, prohibitions, gotchas) and links to ARCHITECTURE.md for what
the system IS. Propose the split, show me the two outlines, do not perform it unprompted -- I
depend on CLAUDE.md every session and a bad split costs me immediately.

## Finally

List every place where an existing document contradicts the code, with file and line, as a
separate section. Do not fix them in this session -- that is Mode C work, one at a time.
```

**Why ADRs and not just more prose.** Your `REVIEW.md` rounds already contain most of this
rationale, but they are ordered by *time*, so answering "why is this table PK-less?" means reading
thirty rounds. An ADR is indexed by *decision*, which is how a newcomer's question actually
arrives. The content already exists; the retrieval path does not.

**The clause that earns its keep is "the code is the fact."** Documentation drift is the specific
way careful projects go bad: the docs stay trusted long after they stop being true, and then
someone acts on them. Making every stewardship pass *report* drift, without licence to fix it in
the same breath, keeps the two activities separable and reviewable.

---

## Prompt 3 — STRUCTURAL HEALTH AUDIT (quarterly, or when something feels heavy)

The anti-rot sweep. This is what catches "vibe-coded and no longer scalable" **before** it is
true, which is the only time the diagnosis is useful.

```
Structural health audit of {{PROJECT}} at {{PATH}}. Measure, judge, propose. Change nothing.

Compare against the baseline in docs/stewardship_prompt.md section 0 and report the DRIFT, not
just the absolute numbers. A file that grew 400 lines this quarter matters more than a file that
has been large and stable for a year.

## 1. Size and shape
Line counts of the 20 largest source files. For each above the budget, decide and state which it
is:
  (a) legitimately large and cohesive -- a single solver, or a manifest that is supposed to grow
      (multimin2.rs, modules.rs). Leave it alone and say why.
  (b) a god file: several unrelated responsibilities that arrived by accretion. Propose a split,
      with the seam named, and add it to ROADMAP rather than doing it now.
Budgets: UI/panel files 800 lines; plumbing (ipc.ts, workspace.ts, db.rs) 1,200; solvers exempt
but must stay single-purpose; modules.rs exempt AS A MANIFEST -- if solver logic has started
living inline in it, that is a finding regardless of length.

## 2. Duplication and drift
Find logic that exists in more than one place and has started to diverge: unit conversions,
NaN/null handling, depth-index alignment, curve lookup, parameter defaults, error formatting.
Divergent duplicates are worse than duplicates -- rank by whether the copies still agree.

## 3. Coupling
Which files import the most, and which are imported by the most. Flag any UI file reaching past
the IPC boundary into backend concepts, and any backend file that knows a UI concept. Name the
specific import.

## 4. Test cover where the bugs actually are
Cross-reference the fault classes in REVIEW.md against what has tests. State plainly which
recurring fault classes currently have NO instrument that could catch a regression. Propose the
smallest test surface that would cover the top one -- for the frontend that likely means a
headless harness rather than a component-test framework; recommend which, and why, given this is
a single-developer desktop app and not a web team.

## 5. Dead and orphaned
Exported functions with no callers, panels never registered, config keys never read, files not
reachable from any entry point. Cheap to find, and every one of them costs a newcomer a
half-hour of "is this important?".

## 6. Dependency and toolchain posture
npm audit and cargo tree, assessed as a DESKTOP TAURI threat model -- not as a public web server.
An advisory in a build-time dev dependency is not a vulnerability in a shipped desktop app; say
so rather than inflating the count. Flag unmaintained direct dependencies, and any dependency
carrying a capability the app does not need (network, spawn, filesystem beyond its job).

## 7. Documentation truth
Spot-check ten specific claims in CLAUDE.md / ARCHITECTURE.md / CONTRIBUTING.md against the code.
Report the pass rate. That single number is the best available proxy for whether the documents
can still be trusted by someone who cannot check them.

## Output
A ranked table: finding, evidence (file:line or a number), cost of leaving it, cost of fixing it.
Ranked by (cost of leaving) / (cost of fixing), highest first. Then ONE recommended next
increment. These are hypotheses -- they enter the normal Mode C flow one at a time, where the
first step is proving they are still open.
```

**Why ranked by ratio and not severity.** Severity ranking produces a list of large scary items
that never get done, and the codebase rots anyway. The ratio surfaces the cheap high-value fixes
first, which is what actually gets executed by one person with a day job. Three of those beat one
heroic refactor that stalls half-finished — and a half-finished refactor is strictly worse than
either state.

**The threat-model clause in §6 is there for a reason.** `npm audit` on a Tauri desktop app
reports a large number of findings that are irrelevant to your actual attack surface. An audit that
cries wolf gets ignored, and then the one real finding is ignored too.

---

## Prompt 4 — HANDOVER / HIRE-READINESS (run before you hire, not after)

```
Hire-readiness assessment for {{PROJECT}} at {{PATH}}.

Assume a competent software engineer joins on Monday. Strong general skills, Rust and TypeScript
fine, and NO petrophysics background -- they do not know what Vsh is, why a cementation exponent
matters, or that a wrong constant silently produces wrong reserves for years.

Answer these six questions with evidence from the repository, not with reassurance. Where the
answer is "no", say no plainly -- an optimistic assessment here costs me a salary.

1. LAPTOP TO RUNNING APP. Trace the exact steps from a clean Windows machine to the app running
   with real data loaded. Every prerequisite, version, environment quirk (the MSVC 14.29 pin, the
   Python subprocess dependency, DuckDB files, sample data). Then say how much of that is written
   down anywhere, and produce tools/setup.ps1 that automates what can be automated and prints
   clear instructions for what cannot. Target: under one hour, unassisted.

2. GREEN GATE. Is there ONE command that proves the tree is healthy? If not, write tools/check.ps1
   -- typecheck, build, cargo test through the pinned toolchain, exit non-zero on any failure. A
   newcomer with no green-gate command cannot tell whether they broke something, so they will
   either not commit or commit blind.

3. THE 30-MINUTE MAP. Does ARCHITECTURE.md exist and is it true? If it exists, verify it by
   tracing two journeys through the real code and reporting where the map is wrong.

4. THE BLAST RADIUS MAP. Produce docs/OWNERSHIP.md classifying every top-level area into:
     GREEN  -- a newcomer may change this alone (UI layout, panel ergonomics, formatting, docs)
     AMBER  -- change with review (IPC surface, persistence paths, module manifest)
     RED    -- do not change without me (physics constants and defaults, saturation/porosity
               equations, the write-discipline paths on computed_curves, cutoff logic)
   RED exists because these have a property normal code review cannot check: a wrong constant
   compiles, passes tests, looks reasonable and produces wrong reserves silently. State that
   reason in the document, or the classification will read as territorial and be ignored.

5. FIRST TICKETS. Propose 3 real tasks from ROADMAP/REVIEW that are genuinely useful, entirely in
   GREEN or AMBER, self-contained, and verifiable by the newcomer without domain knowledge. For
   each: the files involved, what "done" looks like, and how they would prove it.

6. THE DOMAIN PRIMER. A short docs/DOMAIN.md -- the 20 terms a newcomer must know to read this
   code without guessing (Vsh, PHIE/PHIT, Sw, Rt/Rw, m/n/a, cutoff, zone, curve, well group,
   LAS/DLIS). One line each, written for an engineer, not a petrophysicist. This is the document
   that stops a newcomer pattern-matching on variable names and getting the physics subtly wrong.

Deliver 1, 2, 4, 5, 6 as files. Report 3 as findings. Be specific everywhere -- "the code is
fairly clear" is not an answer to any of these questions.
```

**Why the blast-radius map matters more than it looks.** Ordinary code review catches ordinary
mistakes: a null deref, a bad loop bound, a leaked handle. It does not catch `RHOB_MATRIX = 2.68`
where the chartbook says `2.85`, because that line is syntactically perfect, semantically
plausible, and wrong. There is no reviewer for that except someone who knows the domain. Naming
those regions explicitly is what lets you delegate *everything else* with confidence — the map
exists to enlarge the newcomer's freedom, not to fence it.

**Why a domain primer beats more code comments.** A newcomer who does not know what `PHIE` means
will infer it from context and be right about 80% of the time, which is the worst possible number:
often enough to feel confident, rarely enough to ship silent errors.

---

## Prompt 5 — INTEGRATION SEAM (when SandiBumi first depends on SegaraBumi)

```
SandiBumi is about to consume SegaraBumi. Before writing integration code, verify the seam.

Read SegaraBumi's docs/CONTRACT.md (the schema and crate API frozen at charter time) and check:

1. Does SandiBumi need anything the contract does not offer? List each gap. For each, decide:
   extend the contract (additive, minor bump), or work around it on the SandiBumi side. Prefer
   the second while the gap is small -- a contract that grows a special case per consumer is no
   longer a contract.
2. Does SandiBumi want to WRITE anything into SegaraBumi's database? Default answer is no.
   Two writers to one SQLite file, from two applications with different lifecycles, is the kind
   of coupling that is invisible for a year and then permanent. Argue it explicitly if yes.
3. Which direction does the dependency point, and does it stay acyclic? SegaraBumi must never
   learn about SandiBumi. If it needs to, the contract is wrong.
4. What happens when they are at different versions -- user upgrades one and not the other?
   Specify the behaviour: refuse, degrade, or migrate. Silently misreading an older schema is
   the one unacceptable option.
5. Tier-2 fields must arrive at SandiBumi still labelled as model output. Verify nothing in the
   integration path launders a Tier-2 value into a Tier-1 fact. This is the same cardinal rule as
   SandiBumi's own: a degraded or uncertain result must never be presented as a clean one.

Report the seam assessment before any code.
```

**Why this gets its own prompt.** The point where two products start depending on each other is
where architecture is actually decided, and it is usually decided by accident, in a hurry, by
whoever needed one field. Ten minutes of seam review here is worth more than any later refactor,
because after the first integration ships, the wrong seam becomes load-bearing.

---

## Cadence

| When | Run |
|---|---|
| Before SegaraBumi's first commit | Prompt 1 — once, non-negotiable |
| Now, once per project | Prompt 2 |
| Quarterly, or when the app starts feeling heavy | Prompt 3 |
| Before hiring, and again on the new engineer's first day | Prompt 4 |
| At first SandiBumi ↔ SegaraBumi dependency | Prompt 5 |
| Every session | `maintenance_scaling_prompt.md` — unchanged, still the daily discipline |

---

## The three things that actually separate a maintainable product from a vibe-coded one

Stated plainly, because they are easy to lose among the checklists above.

**1. Enforcement beats intention.** Every rule that lives only in prose is eventually violated —
by a contributor, by an AI session, by you when tired. Rules that live in a compile error, a
failing test, or a `check.ps1` exit code survive contact with reality. When you have a choice
between documenting a constraint and making it structural, make it structural. This is the single
biggest difference between the projects that scale and the ones that quietly stop being editable.

**2. Write down decisions, not just code.** A codebase without recorded rationale forces every
future contributor to re-derive why things are the way they are — and re-derivation usually ends
in "this looks wrong, let me fix it". Your PK-less table, your single-writer database and your
non-undoable module runs are all deliberate, all correct, and all *look* like defects. Undocumented
deliberate choices are the most expensive kind of technical debt because the interest is paid by
someone who thinks they are helping.

**3. Keep the increment small enough that one person can verify it.** This is the constraint that
holds everything else together. It is what lets one petrophysicist field-check a 2,000-well
application; it is what makes a commit history readable to a newcomer; it is what stops a fix from
becoming a refactor. Everything in `maintenance_scaling_prompt.md` — one item per increment, one
increment per commit, the `Try:` line — exists to protect this property. Do not trade it away for
speed, because it is the property that *is* speed, measured over a year.
