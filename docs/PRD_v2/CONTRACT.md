# PRD v2 — chapter authoring contract

**Binding on every domain chapter in `docs/PRD_v2/`.** Written 2026-08-07. A chapter that
violates this contract is rejected and rewritten, not patched.

This file exists because PRD v2 is written by eighteen independent authors against eighteen
independent evidence dossiers. Without a fixed identifier scheme, a fixed parameter discipline
and a fixed traceability gate, the result is eighteen documents that cannot be indexed, compared
or gated — which is the failure this whole exercise exists to avoid.

---

## 0. What a chapter is, and is not

A chapter is a **requirements specification**: what SandiBumi must do, why, with what parameters,
proven by what test, and how far the current code is from it.

A chapter is **not** a restatement of its dossier. The dossier is *evidence*. The chapter is
*obligation*. Where the dossier says "IP ships 0.5, Techlog ships 0.4, and they are applied at
different points in the equation", the chapter says "SandiBumi MUST expose the adhesion fraction,
MUST apply it inside `G_HM` as a prefactor, MUST NOT default it, and MUST fail a run that
requests Hertz-Mindlin without one" — and cites the dossier for why.

The dossier stays on disk and is cited by section. Nothing is deleted by being restated.

---

## 1. Identifier scheme

### 1.1 Requirement IDs

`SB-<DOM>-<nnn>` — zero-padded three digits, allocated in document order, never renumbered after
first publication. A withdrawn requirement keeps its number and is marked `WITHDRAWN` with a
reason; numbers are never reused.

| Code | Domain | Dossier |
|---|---|---|
| `CLY` | Clay and shale volume | `clay-volume.md` |
| `POR` | Porosity | `porosity.md` |
| `SAT` | Water saturation | `saturation.md` |
| `MIN` | Multi-mineral solver | `mineral-solver.md` |
| `CUT` | Cutoffs, summation, Monte Carlo | `cutoffs-summation-mc.md` |
| `SHR` | Saturation-height and rock typing | `sat-height-rocktyping.md` |
| `NMR` | Nuclear magnetic resonance | `nmr.md` |
| `TBD` | Thin-bed and laminated analysis | `thinbed-laminated.md` |
| `GEO` | Geomechanics, pore pressure, fracture gradient | `geomech-ppfg.md` |
| `TOC` | TOC and unconventional | `toc-unconventional.md` |
| `ENV` | Environmental corrections and log QC | `envcorr-qc.md` |
| `DIO` | Data import, export, formats | `data-io.md` |
| `DBM` | Database and project data model | `database-model.md` |
| `PLT` | Plotting, display, interactivity | `plotting-interactivity.md` |
| `MLA` | Machine learning and advanced analysis | `ml-advanced.md` |
| `RPH` | Fluid substitution and rock physics | `fluidsub-rockphysics.md` |
| `PLG` | Production logging | `production-logging.md` |
| `INS` | Install, deployment, packaging blockers | `ip-install-blockers.md` |
| `CORE` | Cross-cutting — **spine only**, not allocated by chapters | `04_CORE_REQUIREMENTS.md` |

If a requirement genuinely belongs to another domain, state it and cite the other domain's code
without allocating a number there — the spine reconciles cross-domain requirements at index time.

**The spine is a set of topic documents, not one file.** `00_INDEX.md` (how to read + document map),
`01_PRODUCT.md` (§1–§8), `02_RISKS_AND_CONTRADICTIONS.md` (§9–§11), `03_EVIDENCE_BASE.md` (§12–§14),
`04_CORE_REQUIREMENTS.md` (§15, the `SB-CORE` allocations), `05_STRATEGY.md` (§16–§22),
`06_SEQUENCING_AND_GATES.md` (§23–§26). Chapters cite these by filename and section.

### 1.2 Evidence tiers — carried unchanged from the dossiers

| Tier | Meaning |
|---|---|
| **T1** | Executable or declarative source read directly — Geolog `.lls`, Techlog `.py`, a shipped `.par`/`.info` parameter file |
| **T2** | Vendor manual or help text ingested as text |
| **T3** | Vendor raster (equation image, scanned chart) read visually |
| **T4** | Course notes, project records, secondary literature held locally |

Every factual claim in a chapter carries its tier. A claim with no tier is a defect.

### 1.3 Priority

| Code | Meaning | Gate |
|---|---|---|
| `P0` | Blocks first sale | Must be closed before any paid release |
| `P1` | v1.0 scope | In the 1.0 gate |
| `P2` | v1.5 — the differentiating depth | After 1.0 ships |
| `P3` | v2 — where SandiBumi exceeds the incumbents | Sequenced by the spine |
| `P4` | Horizon — design the seam now, build on demand | Not scheduled |

Priority is a **proposal** from the chapter. The spine holds the authoritative sequencing and may
overrule any chapter's priority; a chapter that disagrees says so rather than silently conforming.

### 1.4 Normative verbs — RFC 2119, used strictly

`MUST` / `MUST NOT` — a violation is a defect. `SHOULD` / `SHOULD NOT` — a violation needs a
recorded reason. `MAY` — genuinely optional. Do not use "will", "should ideally", "aims to", or
any other softener; if the obligation is not one of the five words above, the requirement is not
yet written.

---

## 2. Parameter discipline — the rule that outranks everything else

**A petrophysical parameter is cited or it is absent. It is never inferred, never rounded to
something tidier, never carried over from a neighbouring vendor, and never filled in from general
knowledge.** This is the standing rule for all work on this machine and it is the single most
expensive thing to get wrong, because a plausible-but-wrong endpoint computes, plots, and ships
into a client deliverable without ever failing loudly.

Every parameter appears in a table with exactly these columns:

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|

- **Value** is transcribed byte-exact from the dossier. Not re-derived, not reformatted, not
  unit-converted in the table. If a unit conversion is required, it is a separate row with its own
  source, or a stated derivation in the body with the arithmetic shown.
- **Source** is a specific, checkable string — file and section, page, module and parameter name,
  or a full literature citation. `"IP default"` is not a source. `"industry standard"` is not a
  source. `"per the dossier"` is not a source; name the dossier section.
- Where the vendors disagree and no adjudication is defensible, the value column reads
  **`ABSENT — ships with no default`** and the row carries the competing values in the body with
  their sources. This is a **standing decision already made on this project**: silently picking one
  vendor's number over five others is adjudication disguised as a default. Shipping absent forces
  the interpreter to choose and to record the choice; shipping a default hides it.
- Where a value exists but SandiBumi must not adopt it (licensing, provenance, or it is a vendor
  digitization rather than the primary source), mark it **`NON-ADOPTABLE — cited for verification`**
  and state what the implementation must be derived from instead.

### 2.1 What must not be transcribed at all

- **Vendor chart lookup-table data** — Schlumberger, Halliburton, Baker, Weatherford, Sperry-Sun,
  PathFinder, Anadrill, GE. Cite the chart by existence, attribution and purpose. Do not carry its
  tabulated values.
- **Vendor lookup-table parameter files** whose values *are* their content — the `.obg` overburden
  tables, `Poisson_Ratio_Lithologies.par`, `.neu`/`.ovl` chart tables. Describe by row count,
  column meaning and purpose.
- **No vendor file** (`.itt`, `.itp`, `.att`, `.bor`, `.eli`, `.neu`, `.ovl`, CHM content) is copied
  into this repository in any form.

There is exactly one recorded exception in the whole corpus — the Matthews & Kelly rows in
`Fract_Grad_Coeff.par`, retained because that file is plain text, self-documenting and
user-extensible by its own header, its rows are a digitization of a **published 1967 paper**, and a
High-rated quantification is uncheckable without them. It was escalated as an open rule-boundary
call and **ruled on by Jauhar directly on 2026-08-07**. It is scoped to those rows and **is not a
precedent**. Do not reason from it to any other file. If a chapter believes it has a second case,
it stops and escalates rather than deciding.

### 2.2 Tier C — reconstruction is prohibited, independent derivation is required

**Amended 2026-08-07 by Jauhar's direction.** The earlier rule read "never implemented" and chapters
took it as a blanket bar on the capability. That was the wrong reading of the risk and it was
costing the product real features. **What is prohibited is the derivation path, not the
capability.**

**The prohibited path — reconstruction.** SandiBumi MUST NOT derive a method by reading a vendor's
internals: decompiling or inspecting binaries and weight files, parsing proprietary key files,
transcribing an undisclosed encoding, or inferring an algorithm from observed input/output
behaviour. A method obtained this way stays prohibited **when renamed** — a reconstruction with a
new label is still a reconstruction, and renaming buys nothing.

It also buys nothing *technically*, which is the more important half. A method derived from a
vendor's implementation inherits that implementation's defects, and `03_EVIDENCE_BASE.md` §14.1 —
the vendors' own defects are the opportunity — is the product's primary competitive claim. Copying
their internals forfeits it.

**The required path — independent derivation.** Where a Tier-C item serves a real user need, the
chapter that owns that need **MUST specify a SandiBumi capability derived independently**: from
published literature, primary sources, and first principles, with its own name, its own method, its
own defaults under §2's citation discipline, and its own tests. This is not a permitted option a
chapter may decline. A chapter that leaves such a need unspecified records why in §7.4 and escalates
it; it does not simply refuse.

**Three classes, with different terms. The register is classified, not uniform:**

| Class | Items | Terms |
|---|---|---|
| **C-1 — Patent-claimed** | Omovie Sonic Saturation (US 12,242,011 B2) | A granted patent claims the *method*, so renaming and re-derivation do not clear it. A design-around MUST be checked against the granted claims. **Patent claims are a published public document and reading them is the correct way to design around** — it is not reconstruction. Until the claims are read, no requirement is specified. **Jauhar's decision: read the claims, license, or drop.** |
| **C-2 — Proprietary implementation, publicly described** | Experienced Eye / EEFS (**SPWLA-2021-0091**, Brackenridge et al.), Domain Transfer Analysis, Textural Facies | The *capability* is not protected; the vendor's *implementation* is. The public paper is a legitimate primary source. **Independent derivation is required.** |
| **C-3 — Opaque artifact** | Shipped neural-network weight files, `Freq_Tiles` tile encoding, entropy image speed-correction, frequency-domain dispersion fits | There is nothing to derive *from* — the internals are not visible, and inferring them from behaviour is the prohibited path. The capability is built **natively** from the primary literature. A vendor-trained model is never consumed in any format. |

**A design-around MUST beat what it replaces, and MUST say how.** Parity is not the goal
(`00_INDEX.md`, §5 of this contract). Every independently-derived requirement carries a **`Betters:`**
line naming the incumbent's documented limitation it removes, with that limitation cited. If a
chapter cannot write that line, it has specified a clone and MUST re-derive.

The worked case: Experienced Eye is documented in this corpus as **a brute-force cross-product
harness, not an algorithm** — established by three exact cross-product reproductions — and its
shipped form is **capped at 475 depth levels and samples 100 randomly where the same vendor's
standalone tool uses 200 sorted**. An independently-derived equivalent that is uncapped,
deterministic rather than random-sampled, and provenance-recording under `SB-CORE-010` is better on
three stated axes and needs none of their code to build.

**Capability-level description of any Tier-C item remains permitted** — what it is for, what it
consumes, where it sits in a workflow — because that is competitive intelligence published in the
vendor's own marketing.

### 2.2.1 Two kinds of refusal, never in one list

Chapters MUST NOT file these under one heading. They are opposite in meaning and mixing them makes
strengths read as gaps.

**§7's subsections are not uniformly numbered across chapters** — some run 7.1–7.3, others 7.1–7.4 —
so these two are identified **by name and position, not by number**. They are the **last two
subsections of §7**, in this order:

- **`Refusals` — defect refusals only.** SandiBumi declining to reproduce a vendor's broken equation,
  silent option-drop, or unit-incorrect form. **These are competitive wins**, they discharge
  `03_EVIDENCE_BASE.md` §14.1, and each states what SandiBumi does instead and why it is correct.
- **`Independent-derivation requirements` — the final subsection.** The C-1/C-2/C-3 items in this
  domain, each with its class, its primary sources, its `Betters:` line, and its owning requirement
  id. An item that genuinely cannot be derived yet records the **specific missing source** and
  escalates — an acquisition gap, not a refusal.

A chapter with no Tier-C items in its domain writes the second subsection with the single line
**"No Tier-C item falls in this domain."** It is never omitted, because its absence must be
distinguishable from an oversight.

### 2.3 Client identifiers and asset names

No individual well name enters a chapter.

**Tightened 2026-08-07 by Jauhar's direction.** Field, block, basin and operator names are **not**
used to characterise SandiBumi's methods, scope, or target case — and **"Mahakam" specifically must
not appear anywhere in the product's documentation.** His words: *"i never use Mahakam to define my
own method, mahakam is just another existing alternative from pertamina hulu mahakam, so never talk
about it in sandibumi explicitly anymore."* Mahakam is an operator's asset (Pertamina Hulu Mahakam);
naming it as the defining case misattributes the work.

**Name the physical condition instead.** Wherever a rock, fluid or data context is load-bearing,
state the condition that is actually doing the work:

| Instead of | Write |
|---|---|
| "the Mahakam-delta case" | "fresh formation water (3–13 kppm)" — or whatever the real driver is |
| "Mahakam sands" | "deltaic clastic sands", "thinly interbedded sand-shale" |
| "Mahakam horizontals" | "high-angle and horizontal completions" |
| "at Mahakam spacing" | "at typical development-field sample density" |

**Never substitute a different asset name** — not another field, block, basin, operator or client.
The replacement is the physics or nothing.

This is a positioning rule before it is a confidentiality rule, and it makes the document stronger:
a method defined by its physical conditions generalises to any basin, while a method defined by an
asset is a claim about one customer's acreage. A chapter that needs a concrete case cites the
**conditions** — salinity, age, lithology, bed thickness, contrast — never the place.

Client and asset names inherited from project-kb record citations remain permissible **inside the
project-kb register itself**, which is where that provenance legitimately lives. They do not cross
into a chapter, a requirement, a parameter source string, or any product-facing text.

---

## 3. Chapter skeleton

Every chapter has exactly these sections, in this order, with these numbers.

```
# <N>. <Domain title> — requirements

<Front matter block: dossier, dossier line count, evidence tiers held, author date,
 requirement count, P0 count.>

## 1. Scope and boundary
## 2. What the incumbents do — the requirement-bearing findings
## 3. SandiBumi as-built
## 4. Requirements
## 5. Parameters
## 6. Acceptance tests
## 7. Open items, escalations and refusals
## 8. Traceability — dossier disposition
```

### §1 Scope and boundary

What this chapter owns, and the named seams where it hands off to another chapter. One paragraph
per seam, naming the other domain code. Overlaps are declared here, not discovered at index time.

### §2 What the incumbents do — the requirement-bearing findings

Not a survey. Only the findings that **generate an obligation**. Each carries its tier, the tools
compared, and the consequence of getting it wrong — preferably quantified, because a divergence
with a number attached survives review and one without it does not. Findings from the dossier that
generate no obligation are accounted for in §8, not padded into here.

### §3 SandiBumi as-built

**This section MUST be written from the source code, not from a summary.** Read the relevant files
under `D:\XX. SandiBumi\src-tauri\src\` and `D:\XX. SandiBumi\src\`. Every status claim carries
`file.rs:line`. The repository is **read-only for this task** — read it, do not edit it.

Status vocabulary, used exactly:

| Status | Meaning |
|---|---|
| `ABSENT` | No implementation |
| `PARTIAL` | Implemented for a subset of cases; state which subset |
| `PRESENT-OK` | Implemented and consistent with the requirement |
| `PRESENT-DIVERGENT` | Implemented, but differs from the requirement in a way that changes a number; state the divergence and its magnitude |
| `PRESENT-UNVERIFIED` | Implemented, no test and no field verification; state what would verify it |

`PRESENT-DIVERGENT` is the most valuable status in the document. Look for it specifically: a
capability that exists and is quietly wrong costs more than one that is missing, because the
missing one is visible.

### §4 Requirements

One block per requirement:

```
#### SB-XXX-001 — <short imperative title>          [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST …

**Rationale.** <why — with the cross-tool evidence and its tier>

**As-built.** <status> — `file.rs:123` …

**Verified by.** SB-XXX-T01, SB-XXX-T04
```

Requirements are atomic. If a block contains two obligations that could be independently satisfied,
it is two requirements. If a requirement cannot name an acceptance test, it is not yet a
requirement — either write the test or downgrade it to an open item in §7.

### §5 Parameters

The table from §2 above, complete for the domain. Every parameter any requirement in §4 refers to
appears here exactly once. Group by method where that aids reading, but do not split the table
across sections — one place, so a reviewer can scan every number the domain ships.

### §6 Acceptance tests

`SB-<DOM>-T<nn>`. Each test states: the input, the operation, the expected output **with tolerance**,
and the source of the expected value. A test whose expected value has no source is not a test —
it is a snapshot of current behaviour, and it MUST be labelled `CHARACTERIZATION` if it is kept.

Where a numeric expectation is derived rather than cited, show the arithmetic in the test so a
reviewer can check it without re-deriving it.

### §7 Open items, escalations and refusals

Three labelled lists. **Open** — needed, not yet answerable, with what would settle it. **Escalation**
— needs Jauhar or a primary source that is not on this machine, with the exact question. **Refusal**
— something a vendor does that SandiBumi deliberately will not do, with the reason.

An escalation is a real question with a checkable answer, not a note that more work exists.

### §8 Traceability — dossier disposition

A table with one row for **every** numbered finding, discrepancy-ledger entry, OPEN item and
adoption-spec line in the source dossier. Columns: dossier item, disposition, where it went.

Disposition vocabulary: `ADOPTED` (→ requirement ID) · `DEFERRED` (→ priority + trigger) ·
`REJECTED` (→ reason) · `EVIDENCE-ONLY` (informs the chapter, generates no obligation) ·
`ESCALATED` (→ §7).

**This table is the completeness gate.** "Maintain every particular thing" is enforced here and
nowhere else. A dossier item that appears in no row is a defect in the chapter, not an omission in
the dossier. If the count of rows does not match the dossier's own count of findings, say so and
explain the difference.

---

## 4. Rules for the author

1. **Read the whole dossier.** Not its adoption spec alone. The discrepancy ledger and the critique
   disposition carry findings the adoption spec assumes rather than restates.
2. **The dossier's `## Critique disposition` section is authoritative over the body it corrects.**
   Where they conflict, the disposition wins and you note the conflict.
3. **Do not read `*_critique.md` files as statements of fact.** They assert defects as live; the
   dossier records which were fixed and which were rebutted with evidence. A critique read alone
   will reintroduce a corrected error.
4. **Edit incrementally.** Write §1 through §8 as you resolve them; do not hold the whole chapter in
   memory and write it at the end. Partial progress must survive an interruption.
5. **Do not invent to fill a shape.** An empty §5 with "no parameters in this domain ship values" is
   a correct chapter. A §5 padded with plausible values is a broken product.
6. **Quantify.** "Materially different" is not a finding. "1.382 against 1.085 b/e — a 0.30 b/e
   systematic residual, exactly 1.0× the default `SIG_PEF`" is a finding.
7. **Say when you are unsure.** A requirement marked as resting on a T3 raster you could not read
   cleanly is worth more than a confident one that is wrong.

---

## 5. What "far better" means here — the standard chapters are held to

The commission is to make SandiBumi *far* better than what the three incumbents ship, not to reach
parity with them. Four things generate that, and a chapter that finds an opportunity in any of them
should raise it as a requirement rather than leave it as an observation:

1. **The vendors' own defects are the opportunity.** This corpus found equation rasters printing a
   sum where a product is required, a missing cube root worth eighteen orders of magnitude, an
   inverted unit constant, a coefficient range three orders of magnitude below its documented floor,
   and a shipped constant 192× off its manual. Every one of those is a place where doing it
   correctly is a differentiator that costs nothing but care.
2. **Where the vendors disagree, the disagreement itself is the product.** Three tools shipping
   three different values for one constant is a fact the interpreter needs and none of them
   surfaces. SandiBumi can.
3. **Fail loud where they fail silent.** The recurring pattern across all three is a computation
   that proceeds on inputs outside its validity, producing a plausible number. Validity conditions
   are shipped data in at least one vendor's manifests; carrying them as enforced preconditions is
   both cheap and unmatched.
4. **Provenance is structural here and conventional there.** A parameter that carries the paper it
   came from, through the computation, into the deliverable, is a claim no incumbent can make.

None of this licenses an overclaim. The rule from PRD v1 §6 stands and is not negotiable: an
admitted gap costs a feature; a discovered overclaim costs the deal.
