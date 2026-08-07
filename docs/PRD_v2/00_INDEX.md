# SandiBumi — Product Requirements Document v2

**Version 2.0 of this document · 2026-08-07 · describes product version `0.1.0` and specifies its expansion**

This document supersedes `docs/PRD.md` (v0.1, 2026-07-29) and `docs/FUTURE_PLAN.md` (2026-07-31)
as the authoritative product statement. **Both are absorbed here in full.** Neither is deleted:
they remain on disk as the dated historical record, and every section of both appears somewhere in
this directory, carried forward with its content intact and marked where 2026-08 evidence changes
it.

PRD v2 is a **directory, not a file** — one document per topic. The reason is arithmetic:
"maintain every particular thing" applied to 42,936 lines of cross-tool evidence cannot land in a
single readable document, and the four different readers this PRD has (Jauhar, an evaluating
petrophysicist, a client's IT reviewer, a future engineering hire) each need a different third of
it.

---

## Part 0 — How to read this document

### 0.1 What PRD v2 is

PRD v1 answered *what SandiBumi is*. It was deliberately not a plan, and it named four documents
that were supposed to follow it: `RELEASE.md` (the quality bar), `V1_SCOPE.md` (what 1.0 contains),
`TARGET_ARCHITECTURE.md`, and `ARCHITECTURE.md` + ADRs for the bus-factor risk.

**None of the four was ever written.** Verified 2026-08-07: no `docs/V1_SCOPE.md`, no
`docs/RELEASE.md`, no `docs/TARGET_ARCHITECTURE.md`, no `ARCHITECTURE.md`. The sequence PRD v1 set
up did not run.

PRD v2 therefore does the job of all of them. It is simultaneously:

- **the product statement** — `01_PRODUCT.md`, absorbed and corrected,
- **the evidence base and the expansion thesis** — `03_EVIDENCE_BASE.md` plus eighteen domain
  chapters,
- **the cross-cutting specification** — `04_CORE_REQUIREMENTS.md`,
- **the strategy** — `05_STRATEGY.md`, absorbed from FUTURE_PLAN,
- **the scope gate** — `06_SEQUENCING_AND_GATES.md`: what 1.0 contains, what blocks a sale, how each
  is verified.

### 0.2 Document map

**The spine — read in this order for the whole argument:**

| File | Holds | Primary reader |
|---|---|---|
| `00_INDEX.md` | **This file.** How to read, document map, rules, what changed v1→v2 | everyone |
| `01_PRODUCT.md` | §1–§8 · product statement, problem, users, capabilities as shipped, non-goals, differentiation, NFRs, commercial surface | evaluator, Jauhar |
| `02_RISKS_AND_CONTRADICTIONS.md` | §9–§11 · risk register R1–R15, open questions, where the documents disagree with the code | Jauhar, IT reviewer |
| `03_EVIDENCE_BASE.md` | §12–§14 · the cross-tool corpus, the chapter map, the four sources of genuine advantage | Jauhar, future hire |
| `04_CORE_REQUIREMENTS.md` | §15 · the `SB-CORE-nnn` requirements no single domain owns | engineering |
| `05_STRATEGY.md` | §16–§22 · competitive picture, three asymmetric axes, credibility floor, OSDU, non-goals from the scan, capacity | Jauhar |
| `06_SEQUENCING_AND_GATES.md` | §23–§26 · the release ladder, the 1.0 gate, verification strategy, open decisions for Jauhar | Jauhar |

**The authoring contract:**

| File | Holds |
|---|---|
| `CONTRACT.md` | The binding contract every chapter obeys — ID scheme, evidence tiers, priority scale, chapter skeleton, parameter discipline, Tier-C prohibition, traceability gate |

**The eighteen domain chapters** — one per evidence dossier, listed in `03_EVIDENCE_BASE.md` §13:

`10_clay-volume.md` · `11_porosity.md` · `12_saturation.md` · `13_mineral-solver.md` ·
`14_cutoffs-summation-mc.md` · `15_sat-height-rocktyping.md` · `16_nmr.md` ·
`17_thinbed-laminated.md` · `18_geomech-ppfg.md` · `19_toc-unconventional.md` · `20_envcorr-qc.md` ·
`21_data-io.md` · `22_database-model.md` · `23_plotting-interactivity.md` · `24_ml-advanced.md` ·
`25_fluidsub-rockphysics.md` · `26_production-logging.md` · `27_ip-install-blockers.md`

**The derived roll-ups**, written last because they are computed from the chapters:

| File | Holds |
|---|---|
| `90_GAP_ANALYSIS.md` | Rolled-up capability gap against IP 2025, Techlog 2018.2, Geolog V14 |
| `91_REQUIREMENTS_INDEX.md` | Every requirement ID, priority, status and source, in one table |

The evidence base itself sits outside this directory and is **not duplicated into it**:
`docs/research_2026-08/cross_tool/` — eighteen dossiers, **42,936 lines**, each cross-validating one
domain across IP 2025/2018, Techlog 2018.2 and Geolog V14 down to shipped parameter files and
executable source. Chapters cite dossier sections; they do not restate them.

### 0.3 Reading routes

- **"What is this product and should we buy it?"** → `01_PRODUCT.md`, then `02_RISKS…` §9.
- **"What is wrong with it right now?"** → `02_RISKS_AND_CONTRADICTIONS.md`, then
  `04_CORE_REQUIREMENTS.md` §15.1.
- **"What do I build next and why that?"** → `06_SEQUENCING_AND_GATES.md`, then the chapter for
  your domain.
- **"Why is this better than Techlog?"** → `03_EVIDENCE_BASE.md` §14, then `05_STRATEGY.md` §18.
- **"I am joining this project."** → `00` → `01` → `03` → `CONTRACT.md`, then one chapter end to end.
- **"What must Jauhar decide?"** → `06_SEQUENCING_AND_GATES.md` §26.

### 0.4 The rules this document obeys

1. **The code is the fact; the document is the bug.** Where this document and the source disagree,
   the source is right and this document is defective. Every as-built claim carries `file.rs:line`.
2. **A petrophysical parameter is cited or it is absent.** Never inferred, never rounded to
   something tidier, never carried across from a neighbouring vendor. Where the vendors disagree and
   no adjudication is defensible, the parameter ships with **no default** and the competing values
   are shown with their sources. See `CONTRACT.md` §2 — this is the rule that outranks everything
   else, because a wrong endpoint computes, plots and ships into a client deliverable without ever
   failing loudly.
3. **Tier C is never implemented, approximated or reverse-engineered.** The register is in
   `CONTRACT.md` §2.2. Capability-level description is competitive intelligence and is allowed;
   algorithm reconstruction is not, under any framing.
4. **No overclaim.** PRD v1 §6's rule stands and is not negotiable: *an admitted gap costs a
   feature; a discovered overclaim costs the deal.* The asymmetry is severe, and it is the reason
   several numbers in `01_PRODUCT.md` are less flattering than the ones they replace.
5. **Nothing is compacted.** Where an absorbed section is superseded, the supersession is marked and
   the original claim stays visible. A finding dropped without a recorded reason is a defect in this
   document.

### 0.5 What changed between v1 and v2 — the four-line version

| Measure | PRD v1 (2026-07-29) | Verified 2026-08-07 | Direction |
|---|---|---|---|
| Backend command surface | 118 Tauri commands | **209** | +77 % |
| Registered petrophysics modules | 42 | **51** registered, 1 retired-but-resolvable | +21 % |
| Backend regression net | 426 `#[test]`/`#[cfg(test)]` sites | **775 `#[test]` functions across 54 files** | +82 % |
| Rust / TypeScript source | 44 Rust files | **60 Rust files, 81,579 lines · 102 TS files, 52,754 lines** | grew |
| **Field-verified share of the review checklist** | **72 of 370 — 19.5 %** | **75 of 1,125 — 6.7 %** | **fell by two-thirds** |

The last row is not a regression in verification. It is the backlog tripling in nine days while
three items were retired. PRD v1 said the *trend*, not the absolute number, was the finding. The
trend now has a measurement, and `01_PRODUCT.md` §4.0 / §7.7 and risk R5 are rewritten around it.

### 0.6 Status of this document

**Not yet reviewed.** The spine (`00`–`06`) is complete; the eighteen domain chapters are the
substance and are in progress; `90_GAP_ANALYSIS.md` and `91_REQUIREMENTS_INDEX.md` are derived from
them and are written last.

What this document does *not* do, stated so it is not assumed: it renders no legal conclusion, it
sets no price, it commits to no date, and it does not decide any of the ten items in
`06_SEQUENCING_AND_GATES.md` §26.

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
