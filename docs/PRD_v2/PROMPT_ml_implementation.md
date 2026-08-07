# Session prompt — implement the ML domain in SandiBumi

**Written 2026-08-07.** Paste the block below into a fresh Claude Code session whose working
directory is `D:\XX. SandiBumi`. It is self-contained: it assumes no memory of the PRD-authoring
session that produced the specification.

**Scope for this week: machine learning only.** No other domain.

---

## The prompt

```
Implement the machine-learning domain of SandiBumi against its written specification. ML only
this week — do not touch other domains.

READ FIRST, IN THIS ORDER

1. docs/PRD_v2/CONTRACT.md — §2 parameter discipline, §2.1 what is never transcribed,
   §2.2 the Tier-C rule, §2.3 client identifiers and asset names.
2. docs/PRD_v2/04_CORE_REQUIREMENTS.md — specifically SB-CORE-002 (a degraded or failed
   result is never presented as clean), SB-CORE-004 (no parameter without a source),
   SB-CORE-010 (every computed curve answers "how was I made?", and its scope reaches
   into the deliverable), SB-CORE-011 (byte-identical re-run), SB-CORE-014 (a learned
   model carries its training provenance).
3. docs/PRD_v2/24_ml-advanced.md — the specification. 65 requirements (SB-MLA-001 …
   SB-MLA-065), 10 P0, 105 parameters, 61 acceptance tests. §4 holds the requirements,
   §5 the parameters, §6 the tests, §8 the traceability.

Requirements are already written with rationale, as-built status and named tests. Do not
re-derive them and do not re-litigate them. Implement them.

ORDER OF WORK — correctness, then provenance, then visibility

Do SB-MLA-028 FIRST, before anything else.

  SB-MLA-028 — Every fitted transform is fitted inside the fold.  [P0]
  ml.rs:1130 fits StandardScaler on the FULL matrix, then ml.rs:1175-1176 builds the
  GroupKFold / KFold splits over the already-standardised data. Every blind-well score
  the product reports is therefore optimistic by construction — the scaler has seen the
  held-out well. The comment at ml.rs:1129 reasons about column subsetting and reads as
  reassurance, but it does not address row subsetting at all, which makes the leak look
  considered. Verify this at the source before changing it.

  It comes first because SB-MLA-008 (byte-identical re-run) and SB-MLA-009 (blind-well
  performance travels with the curve) both PROPAGATE this number. Recording a leaked
  score faithfully, and re-running it byte-identically, makes a wrong number more
  trusted rather than less. Fix the number before you make it durable.

  Note: this supersedes escalation E-12 in §7.2 of the chapter, which offers a choice
  between "001+003+006+008 (make it true)" and "001+006+009+010 (make it visible)".
  Neither is right until 028 lands. Do 028, then the "make it true" set, then the
  "make it visible" set.

Then the remaining P0s. Current status from the chapter:

  SB-MLA-001  PARTIAL             Record the effective parameter set, not the supplied one
  SB-MLA-006  PARTIAL             A curve produced by a fitted model names that model
  SB-MLA-008  PARTIAL             A recorded ML run re-runs to byte-identical curves
  SB-MLA-013  PRESENT-DIVERGENT   An unclusterable well fails; it never emits a clean empty curve
  SB-MLA-023  PRESENT-DIVERGENT   One k-means, one definition
  SB-MLA-026  PRESENT-DIVERGENT   The leaderboard evaluates the model the run will fit
  SB-MLA-035  ABSENT              A transformed quantity is a distinct quantity with its own name and unit
  SB-MLA-055  ABSENT              A class label is never interpolated
  SB-MLA-060  PRESENT-OK          No vendor model or weight file is read, converted or imported

SB-MLA-060 is already satisfied — it needs a REGRESSION TEST that locks it, not a fix.

Then the P1 provenance set: SB-MLA-003 (training-row hash), SB-MLA-009 (blind-well
performance travels with the curve), SB-MLA-010 (the deliverable ML provenance block).

SB-MLA-010 has a verified starting fact: report.rs and export.rs currently contain ZERO
occurrences of ml, facies, cluster, hfu or leaderboard. Every ML result the product
produces is invisible in every deliverable it generates. Re-verify that before building.

VERIFICATION — this is not optional and it is not delegable

- After every change, run the project's own gate and read the output yourself:
    cargo check   and   cargo test
  Run these via PowerShell, never Git Bash — Git Bash mangles the vcvars/cargo .bat
  chain and leaves a stale baseline.log that reads as a false pass.
- A delegated edit is NOT done until that gate passes. Never report a subagent's result
  as verified on the subagent's own say-so. Run the check.
- Every requirement you close must close its named acceptance tests from §6. A
  requirement marked done with no passing test is not done.

CONSTRAINTS — these outrank completeness

- A petrophysical parameter is CITED or it is ABSENT. Never invent one, never round one
  to something tidier, never carry one over from a neighbouring vendor. §5 of the chapter
  already records each parameter's source or its recorded absence — use those.
- Do not read, convert, import or inspect any vendor model or weight file, in any format
  (SB-MLA-060, CONTRACT §2.2).
- Do not name any field, block, basin, operator or client — including "Mahakam" — in code,
  comments, docs, UI strings or test names (CONTRACT §2.3). Name the physical condition
  instead: salinity, lithology, bed thickness, contrast.
- The chapter's §7.3 "Refusals" section PREDATES a 2026-08-07 amendment to CONTRACT §2.2
  and has not yet been retro-fitted. Do not build any new capability out of that section
  this week. The P0 and provenance work above is unaffected by the amendment.

DELIVERABLE

Working code with passing tests, plus a short report stating: which requirements closed,
which tests now pass that did not before, what you found that the specification got wrong
(the spine has been stale twice already — if the code disagrees with the document, verify
at source and say so), and anything you could not close and why.
```

---

## Why this order — the reasoning behind the recommendation

`24_ml-advanced.md` §7.2 escalation **E-12** frames the first increment as a binary: `001+003+006+008`
("make it true") or `001+006+009+010` ("make it visible"). **Both are premature.**

`SB-MLA-028` is the leakage requirement. Until it lands, the blind-well score is optimistic by
construction. `SB-MLA-008` makes that score reproducible byte-for-byte, and `SB-MLA-009` carries it
onto the curve where a user reads it — so both of the offered increments take a wrong number and
make it more durable and more visible. Provenance applied to a wrong number is worse than no
provenance, because it converts an unexamined figure into an auditable one that survives review.

So: **correctness (028) → provenance (001, 003, 006, 008) → visibility (009, 010)**.

## The four open decisions this work does not depend on

None of these block the week. Recorded so they are not lost.

1. **`units.rs:180`** — an undeclared depth unit defaults to metres rather than refusing, against
   `SB-CORE-001`. Amend the requirement to permit a declared, surfaced default, or make the refusal
   real? Not an ML question.
2. **US 12,242,011 B2** — three questions for a patent attorney before any C-1 capability is
   specified. See `REF_patent_US12242011.md`. Not an ML question.
3. **`manual_test_plan.md`** — roughly 25 remaining asset-name references, used as test-data
   descriptions rather than positioning. Sweep or leave?
4. **Asset-name rule breadth** — the §2.3 rule was extended beyond Mahakam to other operator assets.
   Confirm or narrow.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
