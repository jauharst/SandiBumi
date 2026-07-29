# Product-definition prompts — deciding what SandiBumi *is*

Prompts for the layer **below** everything else in `docs/`: not whether the software is correct,
maintainable or competitive, but **what it is, who buys it, and when it is finished.**

Every other prompt in this folder presupposes the answer. This one produces it.

| File | The question it asks | Cadence |
|---|---|---|
| `maintenance_scaling_prompt.md` | is this increment correct? | every session |
| `engineering_review_prompt.md` | does the whole app behave? | occasional |
| `qc_audit_prompt_template.md` | is this tool's physics right? | per tool |
| `stewardship_prompt.md` | is this survivable by someone who is not me? | quarterly |
| `sandibumi_maturation_prompt.md` | what should we build next? | per intelligence source |
| **this file** | **what IS this, for whom, and when is it 1.0?** | once, then before each major release |

**Settled inputs** (Jauhar, 2026-07-29 — these are product decisions, not findings; do not
re-derive or re-litigate them, build on them):

- **Commercial posture: a licensed product.** Sold or licensed to E&P operators and
  consultancies, Indonesian market first, Pertamina-scale assets. This is the fact that changes
  everything below — see §0.4.
- **Deployment: single-user Windows desktop today; a shared multi-user backend is a real future
  direction, not a fantasy.** Name it as a future state so today's decisions do not foreclose it.
  Do not build it now.
- **Order: describe what exists, then gate.** The forward-looking document is only trustworthy
  if the backward-looking one is accurate.

---

## 0. Rules that outrank every prompt in this file

**0.1 — The code is the fact.** Borrowed verbatim from `stewardship_prompt.md`, because it
matters more here than anywhere else. Where a document and the code disagree, the code is the
fact and the document is the bug. Report every disagreement; fix none of them in a
product-definition session.

**0.2 — Shipped is not planned, and the difference is not a nuance.** This is the cardinal
data-honesty rule wearing different clothes. In the app: *a degraded or failed result must never
be presented as a clean one.* In a product document: **an unshipped, partial, or never-field-verified
capability must never be presented as a shipped one.** Every capability sentence in the PRD
carries evidence — a file, a command, a `REVIEW.md` line marked `[o]` — or it goes under
*Planned*, in a different section, in different words. "The app supports X" and "X is
code-complete but unverified against field data" are different claims, and only one of them is
safe to put in front of a buyer.

**0.3 — Do not invent market facts.** No market sizes, no customer quotes, no win/loss claims, no
competitor capability assertions that are not traceable to something already on disk
(`docs/research_2026-07/`, the maturation findings, the vendor install ingests). Where a claim
needs evidence you do not have, write the claim as an **open question with the experiment that
would settle it**, not as a finding. A hallucinated PRD is worse than no PRD, because it gets
quoted in a proposal.

**0.4 — Selling changes the calculus; flag, do not resolve.** A pile of things that are free in a
personal tool become obligations the moment the software is licensed to a third party:

| Free when it's yours | An obligation when it's sold |
|---|---|
| Chart data digitized from a copyrighted chartbook | provenance and redistribution rights of the shipped `chartOverlays.ts` values |
| Endpoint defaults and mnemonic tables lifted from vendor installs | the four-tier IP discipline in `sandibumi_maturation_prompt.md` becomes a compliance record, not a habit |
| Theme names like Halliburton / Schlumberger / Pertamina | third-party trademarks used in a product you charge for |
| "the reference suite-class modules" as shorthand in the README | describing your product by reference to a competitor's product line, in customer-facing text |
| A Python interpreter you happen to have | a runtime dependency that a client's IT department may refuse to install |
| "it works on my machine" | an installer that must survive a locked-down corporate SOE |

Your job in these sessions is to produce a **risk register**: each item, what could go wrong, who
would have to answer it (Jauhar, a lawyer, a client's IT). **You are not a lawyer and neither is
Jauhar — do not render legal conclusions, and do not reassure.** Naming the risk precisely is the
whole deliverable.

**0.5 — Separate dev-machine constraints from product constraints.** They read alike in
`CLAUDE.md` and are completely different in a PRD. The MSVC 14.29 pin is a *developer* constraint
— no customer ever sees it, it belongs in `CONTRIBUTING.md` and nowhere near the PRD. The Python
subprocess is a *product* constraint — it gates equations, DLIS import and the whole ML suite on
a client machine, and it is the single most likely reason an install fails at a customer site.
Getting these two backwards produces a PRD that worries about the wrong things.

**0.6 — Non-goals are the deliverable.** A PRD with no non-goals section has not been written. In
a solo project the non-goals are the only mechanism that ever stops scope creep, because
`ROADMAP.md` is structurally incapable of it — a roadmap is ordered by time and grows forever;
a PRD is ordered by value and is allowed to say *no*.

---

## Prompt 1 — THE PRD (run once; two steps, one session each)

```
Write the Product Requirements Document for SandiBumi at D:\XX. SandiBumi.

Read docs/product_definition_prompt.md section 0 FIRST and obey it — especially "shipped is not
planned" and "do not invent market facts". Then read: README.md, CLAUDE.md, ROADMAP.md,
REVIEW.md, CONTRIBUTING.md, the AUDIT-* files, docs/stewardship_prompt.md section 0, and the
ROLE section of docs/sandibumi_maturation_prompt.md (it contains the only written product
thesis that currently exists anywhere in this repo — the two-agent design, the decision
playbook, the parameter knowledge base from 50+ past projects). Read enough source to be
accurate rather than plausible.

Settled product decisions, given, not to be re-argued: licensed product sold to Indonesian E&P
operators and consultancies; single-user Windows desktop now with a shared multi-user backend
as a named future state; describe-then-gate.

## STEP 1 — docs/PRD.md, describing the product AS IT EXISTS TODAY

Derive this from the code, not from the roadmap. Sections:

1. PRODUCT STATEMENT. One paragraph, in petrophysics terms, that a working petrophysicist
   would recognise as true. Not "a Tauri app". Not a feature list.

2. THE PROBLEM. What does a petrophysicist at an Indonesian operator actually do today, with
   which tools, and where does it hurt at 2000-well scale? Ground this in what the repo already
   knows (the vendor ingests, the workflow standards doc, the method docs) — where you don't
   know, write an open question, per rule 0.3.

3. USERS. Three at minimum, and be honest that they have different interests:
     - the interpreting petrophysicist (the buyer's user)
     - the asset team / management who consume the deliverables and never open the app
     - the client's IT department, who must install it and will ask about network access,
       admin rights, data residency and the Python dependency
   For each: their job-to-be-done, and what "this product failed me" looks like for them.

4. CAPABILITIES AS SHIPPED. Grouped by the job they serve, NOT by module name. Every entry
   cites its evidence (file, tauri command, REVIEW.md line). Three buckets, explicitly labelled
   and never blurred:
     SHIPPED AND FIELD-VERIFIED  -- REVIEW.md marks it [o]
     SHIPPED, NOT YET VERIFIED   -- code-complete, tsc/cargo clean, never checked against real
                                   well data by a human
     PLANNED                     -- in ROADMAP, not in the app
   Count each bucket. Re-measure the REVIEW.md verified/unverified ratio yourself; do not
   quote the stale number from qc_audit_prompt_template.md. That ratio is the most commercially
   important number in this document and it belongs in the executive summary.

5. NON-GOALS. Explicit, reasoned, and phrased so a future session cannot argue around them.
   Start from what the code deliberately is not and confirm each against the repo: not a seismic
   interpretation package; not a reservoir simulator; not a real-time / geosteering tool; not a
   corporate data-management or master-database product; not a core-laboratory workflow system.
   For each, one sentence of WHY, because a non-goal without a reason is just a to-do nobody
   got to yet.

6. DIFFERENTIATION, HONESTLY. Where SandiBumi genuinely does something the incumbent suites
   (IP / Techlog / Geolog) do not, sourced from the vendor-intel work already on disk. Then --
   and this section is mandatory -- WHERE IT DELIBERATELY DOES NOT COMPETE. A licensed product
   that claims parity it does not have gets found out in the first evaluation, and everything
   else you claimed becomes suspect.

7. NON-FUNCTIONAL REQUIREMENTS, with numbers, each marked measured / target / unmeasured:
     - scale: the stated 2000+ well ambition vs what has actually been run (note the 2000-well
       stress fixture is still an open ROADMAP item -- say so)
     - performance: cite the measured figures that exist (the 100-well x 4-module chain result)
       and mark everything else unmeasured rather than guessing
     - install footprint: single installer, no external database or runtime -- EXCEPT Python.
       State that exception in the plainest possible language; it is a sales objection.
     - offline / air-gap capability: what works with no network at all
     - data security posture: where does client well data live, and what leaves the machine
       (answer this from the code, including any telemetry or update check -- if the answer is
       "nothing", prove it, because you will be asked to)
     - localisation: EN / Bahasa Indonesia / Basa Sunda, and the deliberate rule that technical
       terms stay English

8. COMMERCIAL SURFACE. What a licence covers, what a seat is, activation and whether it can work
   offline, update and support expectations, and the version-support window (cross-reference
   Prompt 3). Where a decision has not been made, write it as a decision Jauhar owes, with the
   options and their consequences -- do not pick for him.

9. RISK REGISTER, per rule 0.4. Table: risk / what could go wrong / who must answer it / how
   urgent (before first sale, before first enterprise sale, or later). Include IP provenance of
   the digitized chart data and vendor-derived defaults, third-party trademarks in the theme
   names and shipped copy, the Python dependency, and the absence of CI and frontend tests as a
   quality-assurance claim you may be asked to make in writing.

10. OPEN QUESTIONS. Everything you could not settle from the repo, each with the specific
    experiment, document or person that would settle it.

## STEP 2 — the gate (only after Step 1 is reviewed and accepted)

Using the accepted PRD, produce docs/V1_SCOPE.md:
  - which capabilities are REQUIRED for a first paid release, traced to a user's job in PRD §3
  - which shipped capabilities are explicitly NOT part of the 1.0 promise (they exist, they are
    simply not sold or supported yet -- this is a legitimate and underused move)
  - what ROADMAP.md contains that PRD §5 says should never be built, listed for deletion
  - the quality bar, from Prompt 3

Do not touch ROADMAP.md in either step. Propose the reconciliation; Jauhar performs it.
```

**Why the three-bucket capability table is the single most valuable thing in this prompt.**
`README.md` today describes SandiBumi's capabilities in one undifferentiated list — chart
overlays, SandiMin, ML, Monte Carlo, composite plots — and every item in that list is *true* in
the sense that the code exists. But a large fraction of `REVIEW.md` has never been checked against
real well data by a human, and that fraction is invisible in the README. For a personal tool
that's harmless. In a licensed product it is the gap between what you sold and what you can
defend, and it is exactly the gap that surfaces during a client's evaluation, at the worst
possible moment. Forcing the count into the executive summary makes the number impossible to
avoid, which is the point.

**Why users §3 includes the IT department.** They are not users of the software and they have
absolute veto over the sale. The Python-subprocess design is genuinely correct engineering — rule
7 of `CLAUDE.md` exists so a missing interpreter can never stop the app launching, which is the
right call — but "correct engineering" and "survives a locked-down corporate standard operating
environment" are different tests, and only one of them is in the repo's current test suite.

**Why non-goals must carry reasons.** A bare non-goal reads as an omission and gets quietly
overturned by a future session (including a future Claude session) trying to be helpful. A
reasoned one — *"not a reservoir simulator, because the deliverable is the property model that
feeds one, and owning both would double the validation surface for no additional buyer"* —
survives, because the next session has to argue with the reason rather than just notice a gap.

---

## Prompt 2 — TARGET-STATE ARCHITECTURE (run after the PRD is accepted)

```
Produce docs/TARGET_ARCHITECTURE.md for SandiBumi at D:\XX. SandiBumi.

READ FIRST and respect the boundary: docs/stewardship_prompt.md Prompt 2 Deliverable 1
commissions ARCHITECTURE.md, which is a DESCRIPTIVE map of the system as built, for a new
engineer. This document is the opposite direction: given the accepted PRD, what must be TRUE
architecturally for the product to be sellable and to keep its options open. If
ARCHITECTURE.md already exists, this document must not restate it -- reference it.

Inputs: docs/PRD.md, docs/V1_SCOPE.md, CLAUDE.md, the stewardship baseline table, and the code.
Change no code in this session. Anything actionable becomes a ROADMAP entry, nothing more.

1. DEPLOYMENT MODEL, TODAY AND NAMED FUTURE. Today: single-user Windows desktop, one project
   file. Named future: a shared multi-user backend. State the future explicitly so it is a
   decision with a tripwire rather than a vibe.

2. WHAT IS SINGLE-USER-SHAPED, AND HOW DEEPLY. Inventory every place the design assumes exactly
   one user and one open project. Known starting points -- verify each and find the rest:
     - DuckDB behind Mutex<Connection>, single-writer by design
     - one project = one .duckdb file
     - appState globals: selectedWell, selectedInterval, pinnedWellId, activeWellGroup
     - the undo stack and session snapshots
   Classify each as FUNDAMENTAL (rewriting it means rewriting the product) or LOCAL (it is an
   assumption in one layer and could be lifted without touching the physics). That
   classification IS the deliverable -- it is the difference between "we could add a server
   tier in a quarter" and "we could not".

3. THE ONE RULE THAT KEEPS THE OPTION OPEN. Propose it, argue it, and make it checkable. The
   obvious candidate: the compute layer -- solvers, modules, the physics -- must never read
   "the currently open project" or any UI state; it takes inputs and returns outputs. If that
   holds, a server tier is a new caller. If it rots, it is a rewrite. Assess whether it holds
   TODAY, with named exceptions.

4. LICENSING AND ACTIVATION SURFACE. Where would it live, what must it be unable to touch, and
   what must still work when activation cannot reach a network. Do not design a scheme; define
   the boundary a scheme would have to respect.

5. THE DATA BOUNDARY, STATED AS A PROMISE. Client well data is the most sensitive thing this
   product touches. Write the promise in one sentence, then verify it against the code: what
   leaves the machine, ever, including update checks, crash reports and telemetry. If the
   honest answer today is "nothing", say so and name what would have to stay true. Any future
   telemetry must be opt-in and structurally separated, not a flag someone can flip.

6. EXTENSION POINTS THAT CARRY THE PRODUCT PROMISE. The module manifest (modules.rs), the
   generic curve store, plot templates and layouts. For each: what promise in the PRD depends
   on it, and what would break the promise. The manifest is the mechanism that makes new
   petrophysics cheap; a contributor who bypasses it with a bespoke panel widens the
   maintenance surface permanently, and that is now a commercial cost, not just an untidy one.

7. THE SegaraBumi SEAM. Cross-reference stewardship Prompt 5. Do not duplicate it; state only
   what the PRD changes about it -- principally that a licensed product cannot ship a
   dependency whose contract is still moving.

8. WHAT WE ARE DELIBERATELY NOT DOING, with reasons. Include the ones already decided and
   correct: the GPU stays render-only and is not a compute resource; module runs are not
   undoable; computed_curves stays PK-less and no upsert path may assume otherwise.

9. MIGRATION TRIGGERS. For the multi-user future: the observable signal that means "start
   building it" -- a number of concurrent users, a client requirement, a specific deal.
   Written now, while nobody is under pressure, it is a decision. Written later it is a panic.
```

**Why this is a separate document from `ARCHITECTURE.md` and not a section in it.** They have
different lifetimes and different readers. The descriptive map changes every time the code
changes and is read by whoever is about to edit a file. The target-state document changes only
when the product strategy changes and is read when deciding whether to accept a constraint. Merged
into one file, the strategic content gets buried under maintenance edits within a quarter — this
is precisely the failure `CLAUDE.md` is already showing, where the "Current state" narrative and
the agent instructions are fighting for the same file.

**Why §2's FUNDAMENTAL-vs-LOCAL split is the whole exercise.** "Could SandiBumi become
multi-user?" is unanswerable as posed and gets answered by feel, usually optimistically. Broken
into an inventory with a two-way classification, it becomes a finite question with a defensible
answer — and the answer is what you would have to give a client who asks whether their team of
six can share a project. Note that the honest answer today may well be *no, and here is what that
would cost*, which is a perfectly good thing to be able to say precisely.

**Why §3 gets its own section even though it sounds like a truism.** "Keep the compute layer pure"
is the kind of rule that is obviously right, universally agreed, and violated within a month by
one convenient shortcut. Per `stewardship_prompt.md`'s first closing principle — enforcement beats
intention — the useful output here is not the rule but an assessment of whether it currently
holds, with the exceptions named. An exception you know about is a decision; one you don't is a
surprise during a rewrite.

---

## Prompt 3 — v1.0 DEFINITION AND VERSIONING (run with, or just after, Prompt 1 Step 2)

```
Produce docs/RELEASE.md for SandiBumi at D:\XX. SandiBumi -- what 1.0 means, how versions work,
and what the product promises across versions.

Inputs: docs/PRD.md, docs/V1_SCOPE.md, REVIEW.md, ROADMAP.md, db.rs (schema + migrations),
CONTRIBUTING.md.

1. THE 1.0 BAR. Two halves, and both must be met:
   (a) SCOPE -- the capabilities in V1_SCOPE.md, each traced to a user job.
   (b) QUALITY -- measurable, re-checkable, and stated as numbers, not adjectives. Propose the
       bar; defensible candidates given this repo's actual history:
         - every capability inside the 1.0 scope marked [o] in REVIEW.md (field-verified by a
           human against real well data) -- this is THE gate, given the shipped-but-unverified
           count from the PRD
         - zero open Critical-severity items in ROADMAP section 4b
         - ONE green-gate command that proves the tree is healthy (stewardship Prompt 4 Q2);
           note honestly that none exists today
         - a verified clean-machine install, performed on a machine that is not the dev machine
         - the 2000-well scale claim either demonstrated on the stress fixture or REMOVED from
           customer-facing copy -- one or the other, not left ambiguous
   State plainly which of these are not met today.

2. VERSIONING. Adapt semantic versioning to a desktop product where the user never sees an API.
   The real compatibility axis here is NOT the code -- it is the PROJECT FILE. Define:
     MAJOR -- the project file format changes such that an older version cannot read a newer one
     MINOR -- new capability, project file still readable by the same major
     PATCH -- fixes only, no schema change
   Petrophysics-specific and easy to miss: a change to a DEFAULT PARAMETER or a physics
   constant changes the numbers a user gets from the same inputs. Decide now where that lands
   (it is at least MINOR, arguably MAJOR) and require it to be called out in the changelog by
   name, because a reserves number that silently moves between versions is the commercial
   version of the cardinal rule.

3. PROJECT-FILE COMPATIBILITY POLICY. The most important section for a licensed product.
     - forward: what happens when an OLDER app opens a NEWER project file. The only acceptable
       behaviours are refuse-with-a-clear-message or migrate. Silently misreading is
       unacceptable -- it is the same failure as presenting a degraded result as a clean one,
       with the user's whole project as the blast radius.
     - backward: migration on open (db.rs already does this pattern -- see the computed_curves
       PK migration), idempotent, and never destructive: never modify the file in place without
       a recoverable copy.
     - state which of this is TRUE TODAY versus which is a requirement being written down for
       the first time. Do not describe an aspiration in the present tense.

4. CHANGELOG POLICY. User-facing, in petrophysics terms, not commit subjects. Mandatory
   category: "numbers that changed" -- any release that alters a result from unchanged inputs
   says so at the top, names the module and the reason. Precedent: the pay-summary PERM
   semantics fix in the 2026-07-20 audit changed real numbers.

5. RELEASE CHECKLIST. The literal sequence for cutting a release, executable by one person in
   an afternoon, including what gets tagged, what gets built, what gets installed on a clean
   machine, and what gets written down.

6. SUPPORT WINDOW. Which versions receive fixes, for how long, and what a customer on an old
   version is entitled to. One paragraph is enough -- but the absence of this paragraph is what
   turns a single support request into an open-ended obligation.
```

**Why the project file, and not the code, is the versioning axis.** Semantic versioning is
written for libraries, where the contract is an API and the consumer is another program. This
product's contract with its user is the `.duckdb` project — months of interpretation work that
must still open next year, possibly on a different machine, possibly after an upgrade the user did
not choose. Versioning the code and ignoring the file format produces the specific failure where
`1.4.2` opens a project written by `1.5.0`, reads the columns it recognises, silently ignores the
rest, and shows the user a plausible interpretation missing half their work.

**Why "numbers that changed" is a mandatory changelog category.** Every other kind of software can
describe a release as "bug fixes and improvements". This one cannot. If a default changes, or a
constant is corrected, or a cutoff's semantics are fixed, then the same well, the same inputs and
the same button produce a different net pay than they did last month — and someone may have
already put last month's number in a reserves report. The fix is still correct and must still
ship; what is not optional is saying so loudly. This repo has already done it once, correctly, in
the audit; the policy just makes it the rule rather than the instinct.

---

## Sequencing

| Order | Run | Produces | Blocked by |
|---|---|---|---|
| 1 | Prompt 1 Step 1 | `docs/PRD.md` | nothing |
| 2 | *Jauhar reviews and accepts* | — | Step 1 |
| 3 | Prompt 3 | `docs/RELEASE.md` | the accepted PRD (needs the capability buckets) |
| 4 | Prompt 1 Step 2 | `docs/V1_SCOPE.md` | Prompt 3 (needs the quality bar) |
| 5 | Prompt 2 | `docs/TARGET_ARCHITECTURE.md` | the accepted PRD |
| 6 | Reconcile `ROADMAP.md` against `V1_SCOPE.md` | — | Jauhar performs, Claude proposes |

Steps 3 and 4 are circular on purpose and the loop terminates: Prompt 3 proposes the bar from the
PRD's measured buckets, and Prompt 1 Step 2 applies it. Run 3 before 4 and the circle closes on
the first pass.

After that, re-run **only** on a change of product direction, or before a major release. This is
not a recurring discipline like `maintenance_scaling_prompt.md` — a PRD that gets rewritten every
month is not a PRD, it is a diary.

---

## What this file deliberately does not do

- **It does not write a feature spec per increment.** That is `maintenance_scaling_prompt.md`
  Mode A, which already carries the per-increment discipline (implement → verify → `REVIEW.md`
  entry with a `Try:` line → commit). Adding a product one-pager on top would be ceremony, and
  ceremony is the thing that kills a solo developer's process first.
- **It does not decide what to build next.** That is `sandibumi_maturation_prompt.md`, which is
  grounded in real vendor intelligence. This file constrains that one — a maturation proposal
  that lands in a PRD non-goal should be rejected on those grounds alone, cheaply, without a
  design pass.
- **It does not map the code.** That is `stewardship_prompt.md` Deliverable 1.
- **It does not give legal advice.** §0.4 produces a register of questions for a professional.

---

## The one thing worth remembering from this file

The other prompts in this folder are all forms of *checking*. Checking has a comfortable property:
you can always do more of it, and doing more always feels responsible. A solo developer with a
strong checking discipline and no product definition can spend a year making a codebase steadily
more correct without ever deciding whether it is finished — and correctness, unlike scope, has no
natural stopping point.

`ROADMAP.md` cannot supply that stopping point, because a roadmap answers *what next?* and the
answer is never *nothing*. Only a scope document with real non-goals can say **enough**. That
sentence is the reason this file exists, and it is worth more than any individual section above.
