# PRD v2 · Part I (continued) — Risks, open questions, and where the documents lie

**Sections §9–§11.** Continues `01_PRODUCT.md`.

*Absorbed from `docs/PRD.md` v0.1 (2026-07-29) in full, with 2026-08-07 verification and the 2026-08
as-built audit folded in. Section numbers preserved.*

This is the document to hand a sceptic. Everything in it is a fact or an explicitly labelled open
question, and nothing in it is a claim about the product's merits.

---

## 9. Risk register

This identifies risks and routes them. **It renders no legal conclusions, and neither Jauhar nor
Claude is qualified to.**

| # | Risk | Status 2026-08-07 | Who must answer | Urgency |
|---|---|---|---|---|
| R1 | **Chart data provenance.** `chartOverlays.ts` and `neutron_charts.rs` declare in their own headers that they are digitized from a 2013 vendor chartbook. The values ship inside the product; the source PDF does not | **DOCUMENTED** — `IP_PROVENANCE.md` §2.1 records the asset, derivation path, the precise legal question, and three costed fallbacks. *"The single most exposed item in the product"* | Lawyer | Before first sale |
| R2 | **Vendor-derived defaults.** SandiMin's 27-entry `LIB` merges endpoint defaults from two vendor installs, in one vendor's dropdown order | **DOCUMENTED, and v2 raises it to a requirement** — `IP_PROVENANCE.md` §2.2. The action worth doing regardless of the legal answer is to cite primary literature per row, converting "merged from vendor installs" into "sourced from the literature, cross-checked". Chapter `MIN` carries it as `SB-CORE-005` | Jauhar (citations), lawyer (status) | Before first sale |
| R3 | **Third-party names in shipped code and copy** | **PARTLY FIXED.** Copy is fixed. The **theme ids are unchanged and escalated** — they are *client*-branded palettes, and renaming them deletes the feature's purpose rather than answering the question | Lawyer (marks) | Before first sale |
| R4 | **Python prerequisite** — now five packages, not three | **OPEN** — product decision. Note that *not* distributing the packages is a materially lighter licensing obligation than bundling them | Jauhar | Before first enterprise sale |
| R5 | **6.7 % field-verified** — 1,050 of 1,125 checklist items never exercised against real data, and the backlog grew 3× in nine days | **OPEN and worse than at v1.** Cannot be closed by editing; it is verification effort. **The strongest candidate for the v1.0 gate**, and the one number most likely to decide an evaluation | Jauhar | **Before first sale** |
| R6 | **No CI, no lint, no frontend test gate, and a tree that cannot `cargo test` from a fresh clone** | **OPEN** — see `01_PRODUCT.md` §7.7. The fresh-clone break is new since v1 and is cheap to fix | Jauhar | Before first enterprise sale |
| R7 | **Unencrypted project database** | **OPEN, deliberately.** A feature, not a hardening tweak: the hard part is key management, and a lost key destroys months of interpretation | Jauhar / client security review | Before first enterprise sale |
| R8 | **Null CSP** with untrusted text arriving from every imported file | **FIXED** — a real policy is set; `'unsafe-inline'` is absent from `script-src`, which is what defeats the class. Build-only verification caveat in `01_PRODUCT.md` §7.5 | — | Closed |
| R9 | **Single-person bus factor.** No CI, no second maintainer, no `ARCHITECTURE.md`, no ADRs | **OPEN and unmoved** — the stewardship prompts that exist to close this have still not been run, and `ARCHITECTURE.md` still does not exist | Jauhar | Before first enterprise sale |
| R10 | **Support obligation with no defined boundary** | **OPEN** — a commercial decision, not an engineering one | Jauhar | Before first sale |
| R11 | **Granted-but-unused OS capability** (`opener`) | **FIXED** — removed at all four layers. Recorded because the same check should run on every new plugin | — | Closed |
| **R12** | **`save_png` is an unrestricted arbitrary-path write callable from page JS**, with a doc comment asserting a whitelist that does not exist | **OPEN — new at v2.** A security questionnaire will find this. The doc comment makes it worse, not better | Jauhar | Before first enterprise sale |
| **R13** | **Dependency licence obligations are inventoried but not adjudicated** | **PARTLY CLOSED — new at v2.** `THIRD-PARTY-LICENSES.md` now lists distributed dependencies and their declared licences, and states it is not legal advice. What remains is someone competent reading it against the distribution model | Lawyer | Before first sale |
| **R14** | **A silent unit error in saturation-height on foot-declared projects.** **Revised 2026-08-07 after source re-verification — the original wording was wrong.** Depth-unit handling *does* exist and *is* wired into the production module path (`units.rs:38`/`:116`; `workflow.rs:420-422` → `:595`), and the **Leverett** Pc law is unit-correct at `satheight.rs:189` and regression-tested at `satheight.rs:246`. The live defect is the **Skelt-Harrison** branch at `satheight.rs:175`, which compares `SH_B`/`SH_D` — both declared `"m"` at `satheight.rs:117`/`:119` — against a height in the project's own unit with **no conversion at all** | **OPEN and P0, narrowed but not reduced.** Two aggravating facts: the existing regression test pins `("OPT_SWH", "LEVERETT")` at `satheight.rs:251`, so **it covers only the fixed branch and reads as false assurance**; and `units.rs:180` defaults an undeclared unit to metres rather than refusing. `15_sat-height-rocktyping.md` puts the divergence at up to **47.7 saturation units** in the transition zone. Still a wrong number that computes, plots and ships — on **foot-declared projects**, which are real and declared | Jauhar | **Before first sale** |
| **R16** | **One product ships two answers under one method name.** `modules.rs:2279-2283` (`sw_sim`, default `OPT_SIM=MODIFIED`) computes the Bardon-Pied form with no `(1−Vsh)` divisor — faithfully matching Geolog's own `MODIFIED` label. `multimin2.rs:174` computes the `(1−Vsh)` Schlumberger form while its doc comment at `:164` and enum comment at `:115` both call it *"Modified Simandoux (Bardon-Pied)"*. Running the module and the solver on the same well under the same word gives results **7.3 saturation units and ~19 % HCPV apart** | **OPEN — new at v2, and it is P0.** Not a naming nit: the deliverable records one method name for two equations, which defeats `SB-CORE-010` lineage at the point it matters most | Jauhar | **Before first sale** |
| **R17** | **One constant, four definitions.** The clean/shale gamma-ray endpoints are defined at four sites with three different clean values and two different shale values — `ssc.rs:95` (10/150), `modules.rs:521` (20/120), `modules.rs:597` (15/120), `modules.rs:2631` (20/120) — a **22.2 % spread in `Vsh` at GR 70 gAPI** decided by which code path the user entered through. The eight-transform GR ladder is duplicated verbatim at `ssc.rs:57-68` with no test asserting the copies agree. Sandstone matrix density is split 2.645 (five sites) / 2.65 (nine sites), a split `lithology.rs:201` *documents* rather than resolves. **Worst case: a gas-conditioning defect was fixed in `ssc()` on 2026-07-29 and its twin at `ssc.rs:433` (`sspw`) still runs the broken 1.6 weight — 4.72 p.u. low in gas — with the correct form written eleven lines above it and the module header claiming the fix landed** | **OPEN — new at v2, and it is P0.** Verified at source 2026-08-07. The same shape recurs in the cutoff defaults (six panes, two disagreeing sets) | Jauhar | **Before first sale** |
| **R18** | **Every reported blind-well ML score is optimistic, and no ML number is reconstructable.** `ml.rs:1130` fits `StandardScaler` on the full feature matrix; `ml.rs:1175-1176` then builds the `GroupKFold` splits over that already-scaled matrix, so each held-out well's statistics are inside the scaler that standardised it. The blind-well score — the one number a customer relies on — is inflated by construction. Separately, **`report.rs` and `export.rs` contain zero references to `ml`, `facies`, `cluster`, `hfu` or `leaderboard`**, so no trained-model result carries any provenance into a deliverable | **OPEN — new at v2.** Both verified at source 2026-08-07. The leakage is P0-shaped (a wrong number presented as a validation result); the provenance gap is `SB-CORE-010` + `SB-CORE-014` at P1. The comment at `ml.rs:1129` reasons about *column* subsetting and reads as reassurance while saying nothing about *row* subsetting — it makes the leak look considered | Jauhar | **Before first sale** |
| **R15** | **Result-honesty violations in shipped paths.** The 2026-08 audit found seven named places presenting a degraded or failed result as a clean one. **Three are now verified closed at source (2026-08-07)** — Monte Carlo reports per-well failures and guards against *"a confident P10=P50=P90 table of zeros"* (`montecarlo.rs:1300`, `:1824-1828`); the report emits *"Pay Summary unavailable"* rather than omitting the section (`report.rs:546`). **Four remain open** | **OPEN on four of seven, and it is P0.** These violate the product's own stated cardinal rule and attack the exact property the product is sold on. The three closures need regression locks (`SB-CUT-T37`/`T37b`/`T37c`) or they will recur | Jauhar | **Before first sale** |

**Reading the register.** Four risks are closed (R8, R11, and half of R3 and R13). **Four of the five
new ones are P0** and are *correctness* risks rather than commercial ones — which is the more serious
kind, because a commercial risk costs a negotiation and a correctness risk costs the reference
customer. `04_CORE_REQUIREMENTS.md` §15.1 turns R14, R15, R16 and R17 into requirements with tests.

**R16 and R17 arrived from the saturation and clay-volume chapters after this register was first
written, and how they arrived matters as much as what they say.** Both were found by cross-reading
the product against itself — two engines against each other, four definition sites against each
other — which is something no vendor does to its own product and which this PRD's method makes
routine. Neither would have been found by testing, because every individual path is
self-consistent and passes.

They also land on the uncomfortable side of `03_EVIDENCE_BASE.md` §14.1. That section sells SandiBumi
on the incumbents' internal inconsistency — a constant 192× off its own manual, a coefficient
labelled as one mineral in one module and another elsewhere. R16 and R17 are the same class of defect
in this product. The correct response is not to soften §14.1; it is to fix these before the claim is
made, which is why both are P0 and both sit in Tier −1.

Expect more of this shape as the remaining chapters land. `SB-CORE-006` and `SB-CORE-007` are written
to catch the class, not the two instances.

---

## 10. Open questions

Each carries the specific thing that would settle it.

1. **Are `01_PRODUCT.md` §2's four problems the *buyer's* problems, or the builder's?** — Settled by
   structured conversations with 3–5 petrophysicists at target operators. No customer research exists
   in this repository. **Unchanged since v1 and still the largest unvalidated assumption.**
2. **Who signs the purchase order, and what is the budget line?** — Settled by one conversation with
   a target account. Determines whether `01_PRODUCT.md` §8's licence unit is per-seat or
   per-asset-team.
3. **What is the 2000-well answer?** — Settled by building the stress fixture *or* deleting the claim
   from customer-facing text. Cannot remain open past 1.0.
4. **Python: prerequisite, bundled, or optional add-on?** — Settled by a decision from Jauhar plus
   one experiment: attempt an install on a genuinely locked-down machine.
5. **What must a per-capability verification matrix look like, and what does it cost to build?** —
   **v2 promotes this from a question to requirement `SB-CORE-040`**, because R5's trajectory makes
   it unavoidable.
6. **Does the Python subprocess leave client data in temp files?** — Settled by reading the
   subprocess protocol in `python_engine.rs`, `dlis.rs` and `ml.rs` for temp-file use.
7. **Is the automation vision part of the product being sold, or the next product?** — **ANSWERED
   2026-07-29:** the next product. Recorded in `01_PRODUCT.md` §4.9 and non-goal §5.7.
8. **Which document is authoritative for `ROADMAP.md` §B1?** — **New at v2.** ROADMAP §B1 is headed
   *"Correctness — OPEN, awaiting Jauhar's method decision"* and carries roughly fifteen unticked
   items; `review_triage.md` marks the same numbered findings as FIXED on 2026-08-01. ROADMAP is
   demonstrably maintained in that section, so this is not wholesale staleness. Settled by Jauhar or
   by re-deriving each item at the code. **Blocking: no chapter may quote either source until this is
   settled.**

---

## 11. Where the documents disagree with the code

The code is the fact and the document is the bug. **Reported, not fixed.**

1. **`CLAUDE.md` misstated `REVIEW.md`'s own convention — FIXED 2026-07-29.** `[x]` means accepted;
   `CLAUDE.md` had preserved a superseded legend under which 72 accepted items looked like 72 broken
   ones. The verification ratio counts `[x]` as accepted, which is what it means.
2. **`docs/qc_audit_prompt_template.md` §3 carries stale counts.** The numbers should be re-derived,
   never quoted.
3. **`README.md` and `CLAUDE.md` state "2000+ wells"** as a present capability. The measured figures
   are 100 wells for a chain and 540 wells for a 15-minute project open.
4. **Competitor-referential product descriptions** were removed from `README.md` on 2026-07-29; the
   same phrasing persists in `CLAUDE.md` and `ROADMAP.md`, acceptable while internal, needing the
   same treatment before publication.
5. **The two-agent product thesis existed only inside a prompt file** and is not shipped. **RESOLVED
   2026-07-29** by allocation to SegaraBumi. The prompt file's ROLE section still describes it as
   SandiBumi's and is stale.

**v2 adds the contradictions the 2026-08 audit surfaced. These are unreconciled, and every one of
them is a place where a document will mislead a reader who trusts it:**

6. **Test count.** Four documents give four numbers (247 / 279+354 / 366 / 589) across three weeks.
   **Settled here: 775 `#[test]` functions across 54 files, counted 2026-08-07.** All four earlier
   figures may have been true at their own dates; none is current.
7. **Command count.** Documents give 115 and 117. **Settled here: 209, counted 2026-08-07.**
8. **Monte Carlo sensitivity and the ML leaderboard** are listed as gaps in one research document and
   as shipped-and-committed in two others.
9. **Project switching** is recorded as absent in one research document and as shipped in ROADMAP and
   quoted from code in a sweep. The research file is a pre-state snapshot not labelled as one.
10. **Theme violations** appear in three documents with three different phantom-token sets and no
    reconciliation.
11. **Manual test count** is 250 in two documents and 243 in two others; 243 is probably the pre-SHIP
    figure but no document says so.
12. **Cancellation** has two incompatible framings of the same defect — "three workers ignore the
    flag" and "only 5 of ~27 job kinds read it" — both dated the same day. `SB-CORE-036` specifies
    the behaviour regardless of which framing is arithmetically right.
13. **The brain-pointed changelog is stale on the repository path**, still stating the project lives
    at `D:\XX. Arshilla`. That path still exists as an empty shell, so anything acting on the
    changelog writes into a directory where the work is silently lost. The file carries a MOVED
    banner; its contents do not.

14. **An overclaim inside production code.** `modules.rs:770`, the shipped `doc` string for Porosity
    from Density-Neutron, reads: *"(Commercial suites use service-company chart lookups here; this is
    the standard analytic equivalent.)"* — over an implementation whose two options are the
    arithmetic average `(PHID+PHIN)/2` and the Gaymard RMS. **None of IP, Techlog or Geolog ships the
    arithmetic average as a porosity method at all**, so it is not the analytic equivalent of
    anything they do; the chart-free Bateman-Konen route that would be differs from it by a measured
    1.64–1.79 p.u. (`11_porosity.md`). This is `01_PRODUCT.md` §6's overclaim rule broken in the one
    place nobody reviews for marketing copy, and it reaches the user through the module's own help
    text. Verified verbatim 2026-08-07.

15. **A label that names a method the code does not implement.** `phi_son`'s second branch is
    labelled *"RHG (Raymer-Hunt-Gardner)"* and computes none of the three published Raymer
    renderings — it is a field-observed transform on raw `Dt` with a Wyllie shale term, 2.44 p.u.
    off Techlog's convention. Its opt-in compaction correction **adds** 2.30 p.u. at its own shipped
    `DT_SH = 90`, the opposite of the module's documented purpose, with every value inside every
    declared range (`11_porosity.md`). This is an `SB-CORE-006` instance found independently of the
    Simandoux one, in a different chapter, by a different agent.

**The pattern worth naming.** Ten of the first thirteen are the same failure: a document stated a
*measurement* rather than the *method that produces it*, and then the code moved. A test count
written as "775" is stale the next day; a test count written as "`rg -c '#\[test\]' src-tauri/src`"
is never stale. `CONTRACT.md` §4 forbids chapters from quoting counts that are not either
freshly measured and dated, or expressed as the command that measures them.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
