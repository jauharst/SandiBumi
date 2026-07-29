# SandiBumi — v1.0 Scope Gate

**Version 0.1 of this document · 2026-07-29 · applies to the product described in `docs/PRD.md`**

Produced by Prompt 1 Step 2 of `docs/product_definition_prompt.md`, against the PRD as reviewed
2026-07-29. One sequencing deviation, recorded per that prompt's own rules: `docs/RELEASE.md`
(Prompt 3) has not been written yet, so the quality bar is derived **inline in §5** from Prompt 3's
candidate list; `RELEASE.md` will formalise versioning, project-file compatibility and the
changelog policy, and must adopt §5 rather than re-derive it.

**What this document is:** the answer to *"what is enough?"* — which capabilities a first paid
release must contain, which shipped capabilities are deliberately not part of the promise, what in
the roadmap should never be built under this product's non-goals, and the measurable bar that
separates "code-complete" from "sellable". `ROADMAP.md` answers *what next* and cannot answer this;
that is the whole reason this file exists.

**What it is not:** a feature plan. Nothing here schedules work. It draws a line through work that
already exists or is already planned.

---

## 1. The scope principle

One sentence, and every inclusion decision below traces to it:

> **v1.0 is the complete field-study workflow of PRD §3.1's petrophysicist — mixed-vintage logs in,
> defensible deliverables out, at multi-well scale — where every step of that workflow has been
> field-verified by a human against real well data.**

Two consequences, stated up front:

- **Depth beats breadth.** A capability outside the core workflow adds sales surface but also adds
  verification surface, and §6 shows verification — not construction — is the binding constraint.
  The cheapest true statement about SandiBumi at 1.0 is a shorter capability list where every line
  survives a client's evaluation.
- **"Excluded from 1.0" does not mean removed.** Per PRD §0.2 the legitimate move is to ship the
  code and not sell the promise: excluded capabilities stay in the build, marked as preview, and
  carry no support commitment. They are upside during an evaluation, not claims that can fail one.

---

## 2. REQUIRED for 1.0

Grouped by the job, traced to the user in PRD §3 whose failure mode it answers. Evidence of
existence is in PRD §4; this table adds only the *inclusion reason* and the *verification surface*
(what a human must click through before the bar in §5 is met).

### 2.1 Data in — the workflow cannot start without it

| Capability | Traces to | Verification surface |
|---|---|---|
| LAS 2.0 import (alias resolution, declared-NULL, malformed-file family) | §3.1 — multi-vintage heterogeneity is problem #1 in PRD §2 | import a real mixed-vintage batch incl. known-bad files; every failure is per-file and named |
| DLIS import | §3.1 — deliveries arrive as DLIS in this market | one real DLIS per available vintage; missing-Python failure must be clear and non-fatal |
| Tops / core / deviation-TVD / SCAL CSV imports | §3.1 — calibration and zonation inputs | one real file each; malformed rows skipped loudly |
| Generic curve store, unit canonicalization, family aliasing, versioned log sets with provenance | §3.1 — "where did this number come from?" | catalog shows set/provenance for a module output; restore of a prior version works |
| LAS export | §3.1/§3.2 — results must leave the tool in the industry's own format | round-trip a computed curve; NULLs and units survive |

### 2.2 Conditioning — the difference between a demo and a field study

| Capability | Traces to | Verification surface |
|---|---|---|
| Bad-hole QC + universal MASK | §3.1 — the mask defends every downstream number | MASK verified on both a module's **inputs and outputs** (a real recurred bug class, ROADMAP §4b) |
| Environmental corrections (GR/NPHI/RHOB), condflag, ftemp | §3.1 | one well against hand calculation |
| GR normalization (two-point percentile) | §3.1 — multi-vintage GR is the normalization case | field histogram before/after |
| Depth shift / splice; deviation → TVD/TVDSS | §3.1 — depth reference errors are silent and fatal | TVD spot-checked vs an external computation for one deviated well |

### 2.3 Evaluation — the deterministic core

| Capability | Traces to | Verification surface |
|---|---|---|
| VSH (GR, D-N), porosity (den/DN/sonic/phimax), Sw (Archie/Indonesia/Simandoux), permeability (Wyllie-Rose/Coates/transform) | §3.1 — the industry-standard backbone; a buyer tests these *first* because they can check them against their incumbent | one well per method vs a prior interpretation from the incumbent suite (the ground-truth corpus in PRD §2 exists precisely for this) |
| **SSC/SSPW + LRLC RtC/IMTS** | §6.1 differentiator — the reason to buy SandiBumi over parity products | validated against Jauhar's reference-suite LAS exports (`CLAUDE.md` still flags "SSPW needs validation" — that flag must be resolved, not shipped) |
| **SandiMin** (multi-mineral optimizer) | §6.1 differentiator | reconstruction-QC + RMS-vs-core on a cored well (the Round 57 machinery exists for exactly this) |
| Zones, zone parameters, interactive picks (Pickett/crossplot/Thomas-Stieber writing zone params) | §3.1 — parameters must be *pickable and auditable*, not typed from memory | pick → zone param → module rerun chain on one well |
| Rhai/Python equations | §3.1 — the escape hatch every field study needs | one equation per engine; Python-absent degradation is clear |

### 2.4 Field scale — the product statement's own claim

| Capability | Traces to | Verification surface |
|---|---|---|
| Workflow chains (batch, progress, cancel, per-step overrides) | §1 — "the unit of work is a field" | the real 100-well chain, plus cancel mid-run |
| Pay summary + cutoffs | §3.2 — the number management consumes | PERM-missing-fails-cutoff semantics re-verified (this changed numbers once already, AUDIT-2026-07-20) |
| Field dashboard | §3.2 | totals cross-checked against per-well pay summaries |
| Monte Carlo uncertainty (LHS, correlations, P10/50/90) | §6.1 differentiator — uncertainty is a headline, so it is inside the promise, not preview | one zone's P50 vs deterministic run; seeds reproduce |
| Well groups as global filter | §3.1 — the backend does not enforce scoping; the frontend contract is the defence | every batch dialog honours the active group (the standing QC sweep in `qc_audit_prompt_template.md` §3.3) |

### 2.5 Deliverables — what the buyer's management actually sees

| Capability | Traces to | Verification surface |
|---|---|---|
| Composite log plots at true print scale (SVG/PDF) | §3.2 | printed depth scale measured with a ruler (the true-ratio fix exists; verify it held) |
| Report generator (methodology/zone/pay tables + composites, batch per well) | §3.2 — the deliverable that disagrees with the last one is §3.2's failure mode | one full report opened in Acrobat; batch across ≥10 wells |
| Log views, histogram/crossplot/Pickett/correlation, chart overlays | §3.1 — the defence layer for every number | overlay registration under zoom; theme repaint; the R28 wrong-well interval class stays closed |
| Plot image/SVG/PDF export | §3.2 | one of each format placed in a slide/report |

### 2.6 Shell — the conditions for trusting all of the above

| Capability | Traces to | Verification surface |
|---|---|---|
| Undo on data/UI edits; crash safe-mode; autosave; WAL recovery | §3.1 — months of work in one file | kill-and-recover drill on a copy of a real project |
| Processing history | §3.1 — auditability | history reflects a day's real session |
| Sessions, docking, themes, i18n | §3.1 — daily-driver ergonomics | smoke pass; **theme names pending the R3 decision (PRD §9) — the *feature* is in scope, the *names* are a lawyer question** |
| Installer on a clean machine | §3.3 — the IT department's first test | §5 bar item Q4, on hardware that is not the dev machine |

**The Python boundary, stated once for the whole table:** three required capabilities (equations,
DLIS import, ML-none — see §3) depend on a client-side Python. The PRD's open question §10.4
(prerequisite / bundled / add-on) **must be decided before 1.0**, because it changes the §2.1 and
§2.3 install stories. This document does not decide it; it records that 1.0 cannot ship with the
question open.

---

## 3. SHIPPED, and deliberately NOT part of the 1.0 promise

These exist, work, and stay in the build — labelled preview, excluded from customer-facing claims,
carrying no support commitment. For each: why exclusion is the right call *for this release*.

| Capability | Why not in the promise |
|---|---|
| **ML suite** (regression/classification/clustering/reduction leaderboards) | Doubles down on the Python prerequisite (scikit-learn on every seat) for a capability the core workflow does not need; and its value claim ("ML-assisted interpretation") invites exactly the automation expectations §5.7 of the PRD routes to SegaraBumi. Preview label keeps it as evaluation upside. |
| **Electrofacies (k-means/GMM) + facies block track** | Works, demoed, useful — but classification quality claims need per-field validation that no client evaluation window allows. Preview. |
| **Unconventional suite** (Passey TOC, kerogen, GIP/CBM, brittleness) | The launch market (Mahakam deltaic, PRD §2) is conventional; the modules are sound but their verification would consume field-check budget the core workflow needs. Preview, promoted when an unconventional buyer materialises. |
| **Vega-Lite panel + SQL console** | PRD non-goal §5.6 already caps these as escape hatches. An escape hatch in the *promise* becomes a support surface for arbitrary user specs/queries. Stay in build, out of copy. |
| **Auto-correlation (elastic depth warp)** | Assisted-automation adjacent; strong demo, hard verification. Preview. |
| **Assisted contact picking** | Same class. Preview. |
| **KNN synthetic logs (log_predict)** | A synthetic curve presented near real curves is the cardinal-rule's hardest edge case; the labelling discipline exists, but selling it requires verification effort that outranks its buyer value at launch. Preview. |
| **Saturation-height MODELLING suite** (Thomeer/log-J per-RT fits, FWL scan) | The *forward* `sw_height` module is in §2.3; the fitting laboratory (Round 14/15's five families) is specialist SCAL work with a five-item unverified checklist. Preview until a SCAL-rich project verifies it. |
| **Rock typing beyond cutoff-classifier** (FZI/GHE, Winland/Pittman, Lucia, Lorenz plots) | Same shape: methods sound, constants verified on paper (Round 5 open), field verification pending. Preview. |
| **Monte Carlo sensitivity extensions** (tornado, per-param sweep) | The MC *engine* is §2.4; the Wave-B sensitivity layer rides the preview label until the engine's own verification is done. |

The list is long on purpose. **Eleven exclusions is what makes the §2 promise finishable** — every
one of these would otherwise add its own row to the §6 verification debt.

---

## 4. In the ROADMAP, and should NOT be built under this product's non-goals

Per the prompt: what `ROADMAP.md` contains that PRD §5 says never gets built. Listed for
**Jauhar to disposition** — this document proposes, he deletes/moves.

| Roadmap item | Location | Colliding non-goal | Proposed disposition |
|---|---|---|---|
| **Auto-picks: per-zone GR_MA/GR_SH percentile suggestions, change-point auto-zonation, field-wide spike QC** | Part B §B4 | **§5.7 — automation is SegaraBumi's scope** (Jauhar's 2026-07-29 decision) | Move to the SegaraBumi charter inputs; delete from SandiBumi's backlog. This is the clearest collision in the file. |
| **Missing-curve synthesis (per-field regressors for absent DT/NPHI)** | Part B §B4 | §5.7-adjacent — machine-generated curves entering the same store as measured ones | **Borderline, flag not delete:** if kept, the output must carry the same synthetic-labelling discipline as log_predict, and it belongs behind the preview label of §3, not in the core promise. Jauhar's call. |
| **Tauri auto-update** | Part C §C4 ("Distribution") | Collides with **§7.5's zero-egress posture and §8's offline-activation reasoning**, not a §5 non-goal — but the PRD explicitly weighs an in-app updater against losing the clean "nothing leaves the machine" answer | Do not build by default; revisit only as a deliberate §8 commercial decision with the egress trade-off written down. |
| **2D Window: thickness/property maps, contours, volumetrics** | Part C §C5 | Approaches §5.4 (not a corporate data product) and edges toward geomodelling territory §5.2 protects | Keep the *per-marker interval maps* (interpreter QC, in-scope); flag "simple volumetrics" as the line not to cross — volumetrics is the simulator-handoff boundary. |
| **Image log suite** (C2-6), **NMR suite** (C2-5), **core photo digitization** (C2-7) | Part C §C2 | No §5 collision — these are legitimately petrophysics | No deletion. Note only: each is a Wave-D-scale lift that must not precede the §5 bar being met, or 1.0 recedes forever. |

**One roadmap-hygiene finding, not a non-goal collision:** `ROADMAP.md` §C4's "done when" line — *a
colleague installs SandiBumi from an installer… zero developer tools involved* — **is not Phase 12's
finish line; it is §5's bar Q4, needed at 1.0.** It should move from Part C (future) to the release
gate.

---

## 5. The quality bar

Derived from Prompt 3's candidate list against this repo's measured state. Each item is
binary-checkable by one person. **1.0 means all seven, not most of seven.**

| # | Bar | State today (measured 2026-07-29) |
|---|---|---|
| **Q1** | **Every §2 capability field-verified** — a `[x]`-marked checklist item (or equivalent matrix row) exercised by a human against real well data | **NOT MET.** 72 accepted items exist, and §6 shows they cover only part of §2. |
| **Q2** | **Zero open Critical-severity items** in ROADMAP §4b | **MET** as of the R-chain (`ROADMAP.md` A9: Critical & Reliability tiers ✅) — must be re-checked at release, not assumed from today. |
| **Q3** | **One green-gate command** (`tools/check.ps1`: tsc + vite build + cargo test through the pinned toolchain, non-zero on any failure) | **NOT MET.** The three gates exist separately and are run by hand. Cheapest item on this list. |
| **Q4** | **Clean-machine install** — installer built, installed and exercised on hardware that is not the dev machine, by someone following `CONTRIBUTING.md` alone | **NOT MET / never attempted.** Also the §3.3 IT-department test and the Python-decision forcing function. |
| **Q5** | **The 2000-well claim resolved** — stress fixture demonstrated, or the number removed from all customer-facing copy | **NOT MET.** PRD §7.1; still the open acceptance item. Measured today: 100 wells. |
| **Q6** | **Numbers-that-changed ledger exists** — every release note names any module whose output changed for unchanged inputs (to be formalised in `RELEASE.md`) | **NOT MET** as policy (no releases yet); precedent exists (pay-summary PERM change was documented). |
| **Q7** | **The §9 before-first-sale risk register items answered** — R1/R2 (IP provenance) and R3 (theme names) have a lawyer's answer; R10 (support boundary) is written down | **NOT MET.** `IP_PROVENANCE.md` has made them askable; none is answered. |

Deliberately **not** on the bar: frontend test harness and CI (R6). They are before-first-*enterprise*-sale
work (PRD §9) and genuinely valuable, but putting them on the 1.0 bar would gate the release on
retrofitting infrastructure rather than on verifying the product — and Q1 is the honest version of
what that infrastructure would eventually protect.

---

## 6. The distance to Q1 — measured, and worse than the PRD's headline number

The PRD reported 72/370 = 19.5% verified. Preparing this gate required locating those 72, and the
distribution matters more than the ratio:

- **All 72 accepted marks sit in a single round** — Round 2, the 2026-07-21/22 field-review session
  (255 checkbox items; 183 of them still open).
- **Every round since — Rounds 3 through 89, ~87 rounds, everything shipped after 2026-07-22 — has
  zero accepted items.** That includes the whole R-chain, SandiMin's Sw models, the chart-overlay
  library verification rounds, rock typing, SHF, Monte Carlo upgrades and the Vega panel.
- **Worse: roughly Rounds 14–57 mostly have no checkboxes at all.** They record their `Try:`
  instructions as prose/blockquotes, so they are not merely unverified — they are *not countable as
  verifiable items* by any grep. The 370-item denominator undercounts the true verification surface
  by an unknown amount.

Three consequences for the gate:

1. **Q1 is not "finish the checklist"; it is first "build the checklist".** The instrument itself
   is inconsistent: a verification matrix (capability × round × status, the PRD §10.5 open
   question) has to be constructed before progress toward Q1 is even measurable. The §2 tables
   above are the row skeleton for that matrix.
2. **Prioritise by §2, not by round order.** Working REVIEW.md top-to-bottom spends field time on
   R-chain hygiene confirmations; working §2.1→§2.6 spends it on the promise. The R-chain items
   matter, but a 1.0 gate weights the pay summary above a memory-leak confirmation.
3. **The one-round distribution also carries good news:** Round 2 proves the verification *method*
   works at scale — one person accepted 72 items in one sustained session. The debt is large but
   the throughput is demonstrated. At Round-2 throughput, the §2 surface is plausibly a handful of
   dedicated field-review sessions, not months.

---

## 7. What this document leaves for others

| Decision | Owner | Where |
|---|---|---|
| Python: prerequisite / bundled / add-on | Jauhar | PRD §10.4 — blocks §2's install story |
| §4 dispositions (auto-picks → SegaraBumi, etc.) | Jauhar | edits `ROADMAP.md` himself, per the sequencing table |
| Versioning, project-file compatibility, changelog policy, release checklist | `docs/RELEASE.md` (Prompt 3) | must adopt §5, not re-derive it |
| Licence unit, activation, support boundary | Jauhar | PRD §8 |
| The verification matrix build | next increment candidate | §6.1 — the enabling work for Q1 |
| Lawyer's answers to R1/R2/R3 | external | `IP_PROVENANCE.md` |

---

## Acceptance

Accepted when Jauhar has confirmed three things: the §2/§3 split (in particular that **Monte Carlo
is inside the promise** and the **ML suite is outside it**), the §4 dispositions he intends to
apply, and the §5 bar as the definition of 1.0. Anything else redirects, per the collaboration
protocol.
