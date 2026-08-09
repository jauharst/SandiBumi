# P0 tracker — every open requirement, in order, until gate item 1 closes

**This is the page to work from.** It holds the sequence, the exact lines to paste, and the
progress ledger. The prompt body and the four-command check live in `docs/lane_prompt.md` §2–§3 —
one home each, so neither can drift from the other.

**Target:** 1.0 gate item 1, *"every `P0` requirement closed and verified"*. **235 open** as of
`91_REQUIREMENTS_INDEX.md`, minus the 8 `SB-DIO` P0s closed in PR #29 → **227 actually open**,
grouped into **8 batches**.

---

## The ledger — tick as you go

| # | Batch | Reqs | branch | prompt run | 4 checks | pushed | merged | synced | field-verified |
|---|---|---:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| 1 | `feat/p0-core` | 8 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 2 | `feat/p0-env-cut` | 23 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 3 | `feat/p0-por-cly` | 31 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 4 | `feat/p0-sat-shr` | 26 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| — | **re-mark + reindex** | — | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | n/a |
| 5 | `feat/p0-min-tbd-toc` | 27 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 6 | `feat/p0-plt-ins` | 23 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 7 | `feat/p0-geo` | 33 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| 8 | `feat/p0-plg-rph` | 36 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |

**Overdue from before this tracker:** the 46 `SB-DIO` requirements merged in #29 and #37 have
never been opened against a real delivery. Field-verify those before batch 2.

---

## The loop — eight steps, identical every batch

```sh
# 1  BRANCH
cd "/d/XX. SandiBumi-check"
git fetch origin
git checkout -b <BRANCH> origin/master
git branch --show-current          # must print <BRANCH>, not master
```

**2 · PROMPT** — paste `docs/lane_prompt.md` §2, with this batch's two lines below substituted
for `SCOPE:`/`SPEC:`, plus the three add-on paragraphs at the end of this file.

**3 · WAIT** — do not touch the folder. Codex commits as it goes.

```sh
# 4  THE FOUR CHECKS  (docs/lane_prompt.md §3 has the pass/fail table)
git status --short                            # only ?? claude-usage-widget/
git log --oneline origin/master..HEAD         # one commit per requirement
git diff --stat origin/master...HEAD          # no db.rs, no chartOverlays.ts
# and Codex's report: 0 failed, passed total HIGHER than last batch
```

```sh
# 5  PUSH
git push -u origin <BRANCH>
```

**6 · PR** — open the link git prints → *Create pull request* → read *Files changed* →
*Merge pull request* → **Create a merge commit** → *Delete branch*.
**For a physics batch, put `Physics — needs Opus review before release` in the body.**

```sh
# 7  SYNC
cd "/d/XX. SandiBumi"
git checkout master
git pull origin master
```

**8 · FIELD-VERIFY** — `npm run tauri dev`, open a real well, check what changed, mark
`REVIEW.md`. **The only step no gate and no agent can do.** Then tick the row and go to step 1.

---

## Batch 1 · `feat/p0-core` · 8 · **alone, first**

```
SCOPE: the open P0 requirements of SB-CORE in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/04_CORE_REQUIREMENTS.md — implement what it specifies.
```

First and alone because `SB-CORE-007` consolidates the definition sites for shared constants. Any
domain batch running before it adds new sites to the 22.2 % `VSH` spread it exists to remove.
`SB-CORE-006` (7.3 saturation units under one word) and `SB-CORE-001` (the silent 3.28× Pc error)
are equally cross-cutting and sit in the same batch.

**You can check this one yourself without reading code:** the same `VSH` from the same inputs
should come out as one number instead of four.

## Batch 2 · `feat/p0-env-cut` · 23 · physics

```
SCOPE: the open P0 requirements of SB-ENV and SB-CUT in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/20_envcorr-qc.md for SB-ENV, docs/PRD_v2/14_cutoffs-summation-mc.md for
       SB-CUT — implement what each specifies.
```

Holds the highest-value single fix in the queue — **`SB-CUT-031`**: twelve IP-seeded Gaussian
priors passed as σ at **twice** the cited convention's width, which has already moved P10/P90 in
delivered studies.

## Batch 3 · `feat/p0-por-cly` · 31 · physics

```
SCOPE: the open P0 requirements of SB-POR and SB-CLY in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/11_porosity.md for SB-POR, docs/PRD_v2/10_clay-volume.md for SB-CLY —
       implement what each specifies.
```

All 62 `SB-POR` requirements omit a status mark (`_SPINE_PENDING.md` SP-009), so the index cannot
say which already ship. Expect several to come back reported as already satisfied — that is the
lane being honest, not idle.

## Batch 4 · `feat/p0-sat-shr` · 26 · physics

```
SCOPE: the open P0 requirements of SB-SAT and SB-SHR in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/12_saturation.md for SB-SAT, docs/PRD_v2/15_sat-height-rocktyping.md for
       SB-SHR — implement what each specifies.
```

## Re-mark + reindex · after batch 4

```sh
git checkout -b docs/remark-and-reindex origin/master
```

Four batches in, `91_REQUIREMENTS_INDEX.md` will be badly stale — it reads status out of the
chapters, and lanes change code, not chapters. Without this the queue starts telling Codex to
redo closed requirements. The prompt for it is in `docs/lane_prompt.md` §6's neighbour; the rule
that matters is that a status change must cite `file:line`, and `PRESENT-OK` needs **both** code
and a test pinning it.

## Batch 5 · `feat/p0-min-tbd-toc` · 27 · physics

```
SCOPE: the open P0 requirements of SB-MIN, SB-TBD and SB-TOC in
       docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/13_mineral-solver.md for SB-MIN, docs/PRD_v2/17_thinbed-laminated.md for
       SB-TBD, docs/PRD_v2/19_toc-unconventional.md for SB-TOC — implement what each specifies.
```

`17_thinbed-laminated.md` carries axis 3's whole differentiator, recorded in `RESUME.md` §5:
`lrlc.rs:123` and `:228` both read `PHIT`, so the Thomas-Stieber decomposition and the
excess-conductivity saturation **are not connected**. Connecting them is the single most valuable
thing in this batch.

## Batch 6 · `feat/p0-plt-ins` · 23 · frontend, **no physics paragraph**

```
SCOPE: the open P0 requirements of SB-PLT and SB-INS in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/23_plotting-interactivity.md for SB-PLT,
       docs/PRD_v2/27_ip-install-blockers.md for SB-INS — implement what each specifies.
```

Three `SB-INS` requirements are blocked on decisions only Jauhar can make: the installer package
type and scope (§7.1 O-INS-1), the offline runtime strategy (§7.1 O-INS-2 — this is
`06_SEQUENCING_AND_GATES.md` §26 **decision #2 and it blocks 1.0**), and the supported-Windows
matrix (§7.1 O-INS-4). **The other seven depend on none of them.** A lane reporting all ten
blocked has misread the HALT rule as stopping the lane rather than the requirement.

`CLAUDE.md`'s Organic design system is binding for anything visual here: read values from
`docs/design_organic/organic-tokens.css`, never eyeball them; chrome goes Organic, data stays dense.

## Batch 7 · `feat/p0-geo` · 33 · new domain, **alone**

```
SCOPE: the open P0 requirements of SB-GEO in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/18_geomech-ppfg.md — implement what it specifies.
```

Nothing ships in this domain, so most requirements are a build rather than a fix — a Rust function
plus a `modules.rs` manifest entry each, per `CLAUDE.md` rule 9. Alone because 33 new requirements
in an unbuilt domain is already a large branch.

**Expect blockers, and they are correct.** The chapter records one Traugott parameter as `1000` in
printed help and `5600` in the shipped manifest — *"it has no defensible default"* — and that
raster-only Sayers/Wendt equations and closed Traugott coefficients cannot be derived from the held
corpus. Named gaps, not failures.

## Batch 8 · `feat/p0-plg-rph` · 36 · new domains

```
SCOPE: the open P0 requirements of SB-PLG and SB-RPH in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/26_production-logging.md for SB-PLG,
       docs/PRD_v2/25_fluidsub-rockphysics.md for SB-RPH — implement what each specifies.
```

---

## The three paragraphs to append to §2's prompt

```
BATCH ORDER: finish the first domain completely — implemented, tested, gate green — before
starting the next. Do not interleave. One commit per requirement, the message naming the id.

This lane touches physics. Every value you write must ALREADY APPEAR in its SPEC chapter, and
each commit message names the chapter section it came from. If a chapter does not carry a
number, record a named gap and move to the next requirement. Never supply one.

An uncited value blocks THAT REQUIREMENT ONLY. Record it and continue the lane. Only a
boundary violation, a branch switch, or a gate failure you cannot fix inside your own files
stops the lane.
```

Omit the middle paragraph for batch 6.

---

## Not in the eight batches, and why

| What | Why it is out |
|---|---|
| **`SB-DBM`** · 16 P0s · `22_database-model.md` | Touches `db.rs`. The DuckDB write discipline — `computed_curves` is deliberately PK-less, uniqueness upheld by the write path — **has no chapter to cite**, so the citation rule that licenses physics work cannot reach it. Needs Jauhar in the loop, not a prompt |
| **3 of `SB-INS`'s 10** | Blocked on decisions above. `SB-INS-008` is a 1.0 gate blocker already |
| **`SB-DIO-023`** | Needs a family range table owed by `20_envcorr-qc.md` — the first cross-chapter dependency the index surfaced. May close during batch 2 |
| **`SP-003`** | US 12,242,011 B2 claims. For a lawyer, not an agent |
| **`SB-NMR`, `SB-MLA`** | Zero open `P0`s. NMR's 38 requirements are all P1 or below; ML's ten P0s closed in the round-3 work |

## When all eight are ticked

Gate item 1 is closed — *every `P0` requirement closed and verified* — and 1.0's remaining
engineering is items 2, 5 and 7 (`06_SEQUENCING_AND_GATES.md` §24): the 2000-well claim
demonstrated or removed, the verification matrix with a ratio that is not 6.7 %, and
`ARCHITECTURE.md` with decision records.

Items **3, 4 and 6 are not engineering at all** — a lawyer's answer on R1/R2/R3/R13, the six
commercial decisions, and a support boundary in writing. None of them moves because an agent ran.

Then the `P1` set: 401 requirements, same eight-step loop, `-p1` on each branch name.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
