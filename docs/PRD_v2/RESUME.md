# PRD v2 — resume point

**Paused 2026-08-07 at Jauhar's instruction, to resume the week of 2026-08-10.**
This file is the handoff. It assumes no memory of the authoring sessions.

The ML implementation week runs **independently of everything here** — see
`PROMPT_ml_implementation.md`, which is self-contained and needs nothing from this document.

---

## 1. What this is

A refined product requirements document for SandiBumi, built from a cross-tool evidence base
(Interactive Petrophysics 2025/2018, Techlog 2018.2, Geolog V14) plus a full audit of SandiBumi's own
source. The governing rules are in `CONTRACT.md` and they outrank convenience everywhere:

- **§2** — a petrophysical parameter is cited or it ships `ABSENT`. Never invented, never rounded,
  never carried over from a neighbouring vendor.
- **§2.1** — what is never transcribed: vendor chart lookup tables, `.itt`/`.itp`/`.att`/`.bor`/
  `.eli`, `.neu`/`.ovl`, CHM content. Interfaces may be specified; data may not be copied.
- **§2.2** — **amended 2026-08-07.** Tier C is no longer "never implemented". Reconstruction from
  vendor internals stays absolutely prohibited; **independent derivation from published literature is
  required** instead, with its own name, method, defaults, tests, and a mandatory `Betters:` line
  naming the incumbent limitation removed. Three classes: C-1 patent-claimed, C-2 proprietary but
  publicly described, C-3 opaque artifact.
- **§2.2.1** — defect refusals and capability refusals are never in one list. Identified by name and
  position (the last two subsections of §7), **not by number** — chapters number §7 inconsistently.
- **§2.3** — **no client, field, block, basin or operator name** anywhere. Name the physical
  condition instead (salinity, lithology, bed thickness, contrast). Never substitute a different
  asset name.

Vendor install trees are **read-only**. `D:\XX. SandiBumi` is writable for these docs; verify the
repo root has `src-tauri\` and `ROADMAP.md` before treating any path as the project — the old
`D:\XX. Arshilla` path still exists as an empty shell and writing to it silently succeeds.

---

## 2. Status

### Spine — complete

| File | Purpose |
|---|---|
| `00_INDEX.md` | Chapter map |
| `01_PRODUCT.md` | What the product is; §6 the overclaim rule |
| `02_RISKS_AND_CONTRADICTIONS.md` | Open risks; R14 is the depth-unit one |
| `03_EVIDENCE_BASE.md` | Tiers T1–T4; §12.1, §12.2, §14.1 are load-bearing |
| `04_CORE_REQUIREMENTS.md` | `SB-CORE-001…015` — the cross-domain contracts |
| `05_STRATEGY.md` | Positioning; §18 the three differentiation axes |
| `06_SEQUENCING_AND_GATES.md` | Build order |
| `CONTRACT.md` | The authoring rules above |
| `REF_patent_US12242011.md` | Claim analysis, Omovie sonic saturation (C-1) |

### Chapters — 11 of 18 written

**Batch 1** (written before the §2.2 amendment — see task 9 below):

| Chapter | Reqs | P0 | Params | Tests | Dossier items |
|---|---|---|---|---|---|
| `10_clay-volume.md` | — | — | — | — | full |
| `11_porosity.md` | — | — | — | — | full |
| `12_saturation.md` | — | — | — | — | full |
| `13_mineral-solver.md` | — | — | — | — | full |
| `14_cutoffs-summation-mc.md` | — | — | — | — | full |
| `24_ml-advanced.md` | 65 | 10 | 105 | 61 | full |

**Batch 2** (written under the amended rule, all five verified at source afterwards):

| Chapter | Reqs | P0 | Params | Tests | Dossier items |
|---|---|---|---|---|---|
| `15_sat-height-rocktyping.md` | 42 | 13 | 61 | 44 | 79 of 255 |
| `22_database-model.md` | 43 | 17 | 45 | 44 | 297 of 297 |
| `21_data-io.md` | 63 | 10 | 86 | 96 | 202 of 202 |
| `17_thinbed-laminated.md` | 66 | 4 | 52 (9 ABSENT, **0 invented**) | 66 | 327 of 327 |
| `20_envcorr-qc.md` | 58 | 23 | 83 | 70 | 178 of 178 |

All five report `mahakam_refs: 0`.

### Remaining — 7 chapters

`16_nmr` · `18_geomech-ppfg` · `19_toc-unconventional` · `23_plotting-interactivity` ·
`25_fluidsub-rockphysics` · `26_production-logging` · `27_ip-install-blockers`

Then `90_GAP_ANALYSIS.md`, `91_REQUIREMENTS_INDEX.md`, and a machine verification sweep.

---

## 3. How to run the next batch

The method that has worked, and why each part of it matters:

1. **Standalone background `Agent` calls, five at a time. Not `Workflow`.** Workflow's 180 s
   no-progress watchdog kills healthy long-thinking agents; one earlier run burned 8.7 M tokens to
   nothing. Sequential batches of five is Jauhar's directive.
2. **A killed agent has usually already written its file.** Check the output path before re-running
   anything. The envcorr agent died on a transport error with §1 already on disk and was resumed via
   `SendMessage`, not restarted.
3. **Verify every headline claim at source before folding it into the spine.** This is not optional
   and it is where the value is. Running tally across batch 2: the spine was stale **four** times and
   the chapter was right; the chapter was wrong **twice** and the source was right. Both directions
   happen. See §5.
4. **Never delegate parameters or method math.** Both `21_data-io` and `17_thinbed` ran entirely on
   the session model and correctly said so. Announce every delegation inline, `->` on dispatch and
   `<-` on return, with the tier reason.

---

## 4. Open decisions for Jauhar — five, none blocking

1. **`units.rs:179-180`** — an undeclared depth unit falls through to `.unwrap_or(DepthUnit::Metres)`
   rather than refusing, against `SB-CORE-001`. Amend the requirement to permit a declared, surfaced
   default, or make the refusal real?
2. **US 12,242,011 B2** — three questions for a patent attorney before any C-1 capability is
   specified. Listed at the end of `REF_patent_US12242011.md`. That file is explicitly **not** a
   freedom-to-operate opinion and not legal advice.
3. **`manual_test_plan.md`** — roughly 25 remaining asset-name references, used as test-data
   descriptions rather than positioning. Sweep or leave?
4. **Asset-name rule breadth** — §2.3 was extended past Mahakam to other operator assets. Confirm or
   narrow.
5. **`ESC-16` (new, 2026-08-07)** — SandiBumi's despike `K` ships at 3.0. The contamination ceiling
   is `f* = min(1/k, ½)`: flat at **50 %** for every `k ≤ 2`, falling above it. `K = 3.0` therefore
   buys **33.3 %** where 50 % was free on the robustness axis, and the value is `SHIPPED-UNCITED` —
   the code comment at `condition.rs:253-255` calls it "the ordinary three-deviation convention …
   NOT a field calibration". Lowering `K` toward 2 costs false positives. **Not changed** — a
   despike cutoff is a parameter, and parameters are cited or asked about. Does `K` stay at 3.0 with
   the ceiling displayed (`SB-ENV-031`), or move to 2.0?

---

## 5. Corrections made during verification — the record

Kept because a claim that moved is evidence about how the document was built.

**Spine was stale, chapter was right (4):** `SB-CORE-001` (three separate wrong claims — Leverett is
closed, Skelt-Harrison is not); `SB-CORE-002`; `SB-CORE-036` (`cancellable` already ships at
`jobs.rs:89` and six other sites); `SB-CORE-032` (the count had doubled to 109 of 130 sync, 17 of 79
async, 128 `db.0.lock()` sites). Every time the chapter correctly refused to edit the spine itself.

**Chapter was wrong, source was right (2), both in `20_envcorr-qc.md`:**

- **§2.5 stated the despike masking direction backwards** in two places while stating it correctly in
  two others, five lines apart. The closed form `f* = 1/(k²+1)` is *decreasing* in `k`, so a **looser**
  cutoff lowers the ceiling; the draft claimed tightening did. Worse, it attached IP's `mean ± kσ`
  formula to the **Hampel/MAD** estimator SandiBumi actually ships, where the correct ceiling is
  `min(1/k, ½)` on the zero-scatter fallback branch and 50 % on the true-MAD branch. Corrected, with
  the fallback derived from `condition.rs:154-172`, `ESC-16` raised, and tests `T69`/`T70` added.
- **`SB-ENV-047` rested on a misread of a two-branch `if`.** It claimed `ftemp_grad` declares `BHT`
  and `TD_BHT` and never consumes them, citing `modules.rs:1051` — which is the `else` arm. The BHT
  branch consumes both at `:1041`, `:1042`, `:1046`, `:1049`, guarded, documented and tested.
  Status corrected `PRESENT-DIVERGENT` → `PRESENT-OK`; `ESC-10`'s candidate `SB-CORE` id amended
  down to preventive, since its only instance evaporated.

**Overclaim removed from `05_STRATEGY.md` §18.3.** Axis 3 claimed "the most complete low-contrast-pay
suite in existence". The audit returns **1 of 27 `PRESENT-OK`**, 19 `ABSENT`, and `lrlc.rs:123` and
`:228` both read `PHIT` — total porosity — so the Thomas-Stieber decomposition and the
excess-conductivity saturation are not connected. Replaced with the supportable claim: *the only tool
shipping excess-conductivity low-contrast saturation models **alongside** a Thomas-Stieber
decomposition.* The axis stays; connecting the two halves is its highest-value item.

**Folded into the spine 2026-08-07.** `SB-CORE-007`'s requirement extended to cover **output-mnemonic
ownership**, with the dual-`FTEMP` case as its third verified instance and `SB-CORE-T23` as the gate:
`ftemp_grad` and `precalc` both write `FTEMP`, **33.1 °C apart at 2 000 m TVD on their own shipped
defaults** (86.7 vs 119.8), propagating through `Rw(T)` and `Sw ∝ √Rw` to **14.3 % relative on `Sw`**.
Deliberately **not** filed under `SB-CORE-006`: both modules compute the same linear trend, so
`SB-CORE-T17`'s shared fixture would pass. The divergence lives entirely in the defaults, which is
why `SB-CORE-T23` forbids the fixture from supplying parameters.

**Note.** `SB-CORE-T04` … `SB-CORE-T08` are unassigned gaps in the spine's test numbering. Not an
error, but worth closing during the `91_REQUIREMENTS_INDEX.md` sweep.

---

## 6. Queued, deliberately not started

**Task 9 — retro-fit the `CONTRACT.md` §2.2 amendment into the six batch-1 chapters** (`10`, `11`,
`12`, `13`, `14`, `24`). They were written when Tier C meant "never implemented" and now need the
derivation-path framing plus the §2.2.1 two-list split. Heaviest is `24_ml-advanced.md`. `11` and
`14` likely need only the line *"No Tier-C item falls in this domain"*, which §2.2.1 requires rather
than permitting omission.

**Do not build new capability out of `24_ml-advanced.md` §7.3 this week** — it predates the amendment
and has not been retro-fitted. The ML P0 and provenance work is unaffected.

Also queued: task 7 (harsh-critic review), task 3 (gold benchmark, deferred), task 8 (closeout).

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
