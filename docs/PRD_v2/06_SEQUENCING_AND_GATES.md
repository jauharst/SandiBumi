# PRD v2 · Part IV — Sequencing and gates

**Sections §23–§26.** This is the file that decides what gets built next, what 1.0 contains, and what
Jauhar owes.

---

## 23. The release ladder

`05_STRATEGY.md`'s strategic tiers and this document's requirement priorities are two different
things and must not be conflated. **The tiers say what claim we can make; the priorities say what must
be true before we make it.**

| Tier | The claim it buys | What it needs |
|---|---|---|
| **Tier −1 — Truth** | *(no claim — the precondition for all of them)* | The five `P0` requirements, §23.1 |
| **Tier 0 — Feel** | "It is a product." | Installer, in-app method help, command palette (`27_ip-install-blockers.md`) |
| **Tier 1 — The claim** | "It is the only one that can prove how a number was made." | `SB-CORE-010`, `-011`, `-012` |
| **Tier 2 — The moat** | "It reads your archive without your data leaving the building — and checks the physics." | SegaraBumi v1.5 |
| **Tier 3 — The depth** | "It is the best low-contrast-pay tool in existence." | `17_thinbed-laminated.md` |
| **Tier 4 — Floor** | "It fits your estate." | 2D map → NMR → OSDU *(demand-driven)* → image logs |

Ordering principle, unchanged from FUTURE_PLAN: **finish what is nearly done before starting what is
merely valuable.**

### 23.1 Tier −1 — Truth

**v2 inserts this tier below Tier 0, and it is not optional.** Nothing above can be claimed while the
product violates its own cardinal rule, computes a silently wrong number on a foot-based project, or
cannot state what has been verified.

| ID | What | Risk it closes |
|---|---|---|
| `SB-CORE-001` | Depth unit carried and enforced | R14 — the silent 3.28× Pc error |
| `SB-CORE-002` | No degraded result presented as clean | R15 — the seven named violations |
| `SB-CORE-004` | No parameter ships without a source — machine-gated | R2, and the foundation of Tier 1 |
| `SB-CORE-006` | One name, one equation — the two engines agree | R16 — 7.3 su under one word |
| `SB-CORE-007` | One definition per constant and transform | R17 — 22.2 % Vsh spread across four definition sites |
| `SB-CORE-040` | Verification indexed by capability | R5 — makes 6.7 % answerable |
| `SB-CORE-041` | Fresh clone builds and tests | R6, R9, and escrow |

**The argument for putting these first is not tidiness.** Several are *cheap*, all are *visible to a
buyer*, and every one of them is a place where the product currently contradicts the thing it is sold
on. Axis 1 says we can prove how a number was made; `SB-CORE-002` is the requirement that stops us
proving how a *wrong* one was made.

**`SB-CORE-006` and `SB-CORE-007` were added after the first two chapters landed**, and both were
found by cross-reading the product against itself rather than by testing it. That is a signal about
the remaining sixteen chapters: this tier should be expected to grow, and the 1.0 gate's item 1
("every `P0` requirement closed") is therefore a moving target until the chapter set is complete.
Treat the Tier −1 list as open until `91_REQUIREMENTS_INDEX.md` is built.

There is also a sequencing dependency that is easy to miss: `SB-CORE-004` is a Tier −1 item **and** the
precondition for Tier 1. Lineage that records a parameter value without its source is an activity log,
which is what the incumbents already ship. Building Tier 1 before `SB-CORE-004` produces a
differentiator that is not differentiated.

---

## 24. The 1.0 gate

**A paid release requires all of the following. Each is a fact, not a judgement.**

| # | Gate item | Owner |
|---|---|---|
| 1 | Every `P0` requirement closed and verified | Engineering |
| 2 | The 2000-well claim either **demonstrated on a fixture** or **removed from all customer-facing text** (`01_PRODUCT.md` §7.1). Ambiguity is the only unacceptable outcome | Engineering + Jauhar |
| 3 | R1, R2, R3 and R13 routed to and answered by a lawyer. This document renders no legal conclusions | **Lawyer** |
| 4 | The commercial surface decided — `01_PRODUCT.md` §8's six decisions, with the Python decision first | **Jauhar** |
| 5 | A verification matrix a buyer can be shown, and a stated ratio that is not 6.7 % | Engineering |
| 6 | A support boundary in writing (R10) | **Jauhar** |
| 7 | `ARCHITECTURE.md` and decision records in the tree (R9 / `SB-CORE-043`) | Engineering |

Items 3, 4 and 6 are Jauhar's or a lawyer's and cannot be done by any amount of engineering. Items 1,
2, 5 and 7 are engineering and are all in scope here.

**What is deliberately *not* on this gate:** feature parity with any incumbent, NMR, image logs, OSDU,
the 2D map, and automation. Every one of them is a real gap and none of them blocks a first sale. A
1.0 gate that includes everything valuable is a gate that never opens — which is precisely what
happened to PRD v1's four named follow-on documents.

---

## 25. Verification strategy

The corpus makes a stronger verification approach possible than the product currently uses, and it
costs little to adopt.

1. **Cross-tool numeric agreement as a test oracle.** Where two independent implementations agree,
   their agreed value is a legitimate expected value with a real source. Where they disagree, that
   disagreement is itself the test — SandiBumi must reproduce whichever it claims to implement, and
   say which.

2. **Characterization tests are labelled as such.** A test whose expected value is "what the code does
   today" is a snapshot, not a proof, and `CONTRACT.md` §6 requires it to say so.

   *A snapshot test wearing the costume of a correctness test is worse than no test*, because it
   converts a bug into a defended invariant. This is not hypothetical here: the retired legacy solver
   carried a test that forward-models the same wrong PEF mixing law the solver used, so it passed by
   construction and made a 0.30 b/e systematic error look verified.

3. **Real-field regression over synthetic.** The precedent is on file: one real field surfaced two
   genuine bugs that synthetic testing had not. This is also the cheapest route to closing R5 —
   `05_STRATEGY.md` §22.4.

4. **The gate is machine-run** (`SB-CORE-042`), or it is not a gate.

5. **Every chapter's §6 tests are executable specifications, not intentions.** `CONTRACT.md` §3
   requires each acceptance test to name its input, its expected value, and the source of that
   expected value. A test whose expected value has no source is `SB-CORE-004`'s failure mode wearing a
   different hat.

---

## 26. Open decisions for Jauhar

Consolidated from every part of this PRD so they sit in one place. Each names what would settle it.
**Nothing in this list has been decided by any agent.**

| # | Decision | What settles it | Blocking? |
|---|---|---|---|
| 1 | **The 2000-well claim** — build the fixture or delete the number | Either action; not a third option | Blocks 1.0 |
| 2 | **Python** — prerequisite, bundled, or optional add-on | A decision plus one experiment: attempt an install on a genuinely locked-down machine | Blocks 1.0 |
| 3 | **The six commercial decisions** (`01_PRODUCT.md` §8) | Jauhar, informed by one conversation with a target account | Blocks 1.0 |
| 4 | **Which document is authoritative for `ROADMAP.md` §B1** (`02_RISKS…` §10.8) | Jauhar, or re-deriving each of the ~15 items at the code | **Blocks chapters** — no chapter may quote either source until settled |
| 5 | **Portfolio-scale target** — the exact number and the operations that must hold at it | Jauhar | **Blocks `SB-CORE-030`** — it cannot be written as a test without the number |
| 6 | **Whether to reopen the parked compute DAG and cache** (`SB-CORE-033`) | Follows from decision 5 | No |
| 7 | **OSDU** — is any named client running or committed to a platform, and does the connector live in SandiBumi, SegaraBumi, or a shared crate | A named opportunity. Until then, design the seam only | No |
| 8 | **Lineage granularity** — per-run, or per-sample provenance for edited curves | The second is much more expensive and may not be needed for an audit to pass | Blocks `SB-CORE-010` design |
| 9 | **The hire trigger** (`05_STRATEGY.md` §22.3) | Jauhar, in writing, in advance | No |
| 10 | **Theme ids** carrying client brand names (R3) | Lawyer, on marks | Blocks 1.0 |

### Decisions already made, recorded so they are not reopened by accident

| Decision | Date | Where recorded |
|---|---|---|
| The two-agent automation vision is **SegaraBumi's**, not SandiBumi's | 2026-07-29 | `01_PRODUCT.md` §4.9, non-goal §5.7 |
| SandiBumi ships first | 2026-07-31 | FUTURE_PLAN Q0 |
| Vendor libraries are treated as authoritative for SegaraBumi's dictionary | 2026-07-29 | SegaraBumi P1 closeout |
| Disputed parameters ship **absent, not defaulted** | 2026-08-06 | `03_EVIDENCE_BASE.md` §12.2 |
| The Matthews & Kelly coefficient rows stay in the geomech dossier — scoped, no precedent | 2026-08-07 | `03_EVIDENCE_BASE.md` §12.3 |
| PRD v2 absorbs PRD v1 and FUTURE_PLAN **uncompacted** | 2026-08-06 | `00_INDEX.md` §0.4 rule 5 |
| Target scale is a **portfolio**, thousands of wells across many fields | 2026-08-06 | `01_PRODUCT.md` §1 |
| Positioning is a **commercial product**, not an internal tool | 2026-08-06 | `01_PRODUCT.md` §8 |
| The compute DAG and content-hash cache is **parked** | 2026-07-29 | `SB-CORE-033` |
| QC fixes go serially in the main working tree — no branches, no worktree isolation | earlier | project convention |

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
