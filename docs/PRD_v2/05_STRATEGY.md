# PRD v2 · Part III — Strategy

**Sections §16–§22.**

*Absorbed from `docs/FUTURE_PLAN.md` (2026-07-31) in full. Section numbering continues this
document's; the original's own numbering is noted where it aids cross-reference. Competitive claims
are dated — re-verify before quoting them to a client.*

---

## 16. Where the two products stand — 2026-07-31, updated 2026-08-07

### SandiBumi

Phases 1–10 code-complete. Shipped: dockview workspace, WebGPU log views, the full deterministic
module library, SandiMin, the Jauhar method suite (SSC/SSPW, LRLC RtC/IMTS), saturation-height, rock
typing, Monte Carlo, workflow chains, field dashboard, electrofacies and the ML suite, composite
plots and PDF reports, delivery sets across every store, open-path hardening, and the chartbook
overlay library. Open work is in `ROADMAP.md` Part B (hardening and the interpretation-workflow
queue) and Part C (method-suite waves, new data models, trust and reproducibility, platform and
extensibility).

### SegaraBumi

Much further along than an outside reading of the roadmap suggests. **P6 gate closed 2026-07-29:**
23,976 files force-migrated across three roots, 9,096 datasets classified on both axes, 4,092
attributes harvested, desktop GUI shipped, the semantic tier shipped as a separate model-stamped
store that degrades cleanly to exact-only, and the one-way pull contract into SandiBumi live.

Remaining to v1: GUI fuse toggle, the bilingual fixture set, installer, own icons. Then **v1.5 =
physics validator + XLSX capture** — the piece that completes the moat in §18.2.

### What this means

The two products are much closer to the strategic position than the feature lists imply. The work
ahead is **finishing and framing**, not starting.

---

## 17. The competitive picture — researched 2026-07-31

| Tool | Recent capability |
|---|---|
| **Techlog** (SLB) | **AI Import** — LLM extraction from unstructured legacy archives (TIF, PDF, scans, handwritten notes). **Log foundation model** trained on 18,000+ public wells, locally tunable, no GPU required. **3D Petrophysics** for high-angle and horizontal wells — wellbore-centric grid, azimuthal petrophysics. ExpertDip → local layer geometry |
| **Geolog** (AspenTech) | ML-enhanced evaluation with uncertainty quantification. **Aspen Epos** — multi-user simultaneous collaboration, 10K+ well projects. **OSDU connector**. Modular hooks for proprietary algorithms |
| **Interactive Petrophysics** (Geoactive) | **2025**: Mapping module (seismic surfaces, faults, GIS shapefiles, spatial query, polygons), **OSDU connector**, parent-well grouping for sidetracks, pinned plots. **2026**: new modules — Sonic Saturation (patented; on our Tier-C register), Multi-Well Experienced Eye, Pyrolysis; multi-regressions on crossplots; streamlined Python installer; upgraded Casing Inspection, Formation Testing, Image Analysis, Mapping |

### Three signals

1. **The archive→data problem is the live frontier.** Techlog's flagship new capability is ingesting
   legacy unstructured archives. It is LLM-based and non-deterministic.
2. **OSDU has become table stakes.** Two of three shipped connectors inside a year. See §20.
3. **Everyone is bolting Python on.** IP ships a "streamlined Python installer"; Geolog advertises
   hooks for proprietary algorithms. SandiBumi was born with Python/numpy as the scripting surface —
   this is a lead, not a gap.

### The honest constraint

We cannot out-feature Techlog. It has three decades and hundreds of engineers. Every plan below is
chosen because it exploits a **structural** weakness the incumbents cannot fix without abandoning
their own architecture — not because it is a feature they merely haven't got to.

**v2 adds a fourth signal, and it sharpens the constraint into an opportunity.** Scale is not the only
axis. The corpus proves that all three ship real, quantified defects in their core physics
(`03_EVIDENCE_BASE.md` §14.1) — a sum where a product belongs, a missing cube root, an inverted unit
constant. Three decades of engineering did not prevent those. Care does, and care does not require
headcount.

---

## 18. The positioning — three asymmetric axes

### 18.1 Axis 1 — Reproducibility as a product, not a feature

**The weakness:** none of the three can answer, end to end, *how was this curve made — which inputs,
which parameter values, from which source, by whom, when — and does re-running produce an identical
number?* Their audit trails are activity logs, not lineage graphs. The market has noticed: newer
entrants now advertise "full audit trail" as a differentiator, which only works as a pitch because
the incumbents are weak there.

**What we already have:** a primary-key-less computed-curve store with a strict write discipline;
versioned log sets with provenance; undo everywhere a cell or property changes; a deterministic
manifest-driven module framework; Monte Carlo seeded from `(seed, index)`; and SegaraBumi's proven
doctrine of rule IDs and byte-identical re-indexing.

**What ships it:** `SB-CORE-010`, `-011`, `-012` — the `runs` table, ancestry for any computed curve,
named scenarios with A/B diff.

**Why it is supra, not parity:** re-run a whole field from raw LAS to pay summary and get
byte-identical numbers, with a diffable record of every parameter and its source. That is what a
reserve audit, a PSC submission, or a partner data room actually needs, and no incumbent can produce
it. It also converts our strictest engineering constraint — the write discipline — from an internal
rule into a **sellable property**.

**Extension worth designing now:** the parameter-source rule should become *machine-recorded*, not
just a convention. A curve whose lineage names the paper its `m` came from is a claim no competitor
can make. **v2 makes this `SB-CORE-004` and `SB-CORE-005`, and it is no longer optional** — the
convention is currently ungated and has already decayed in one visible place.

### 18.2 Axis 2 — The deterministic archive bridge (SegaraBumi)

**The weakness:** Techlog AI Import is the most-promoted new capability in the market, and it is
LLM-based, non-deterministic, and in practice cloud-adjacent. It cannot tell a client *why* it
decided a file belonged to a well, and it cannot promise the same answer twice.

**What we already have:** essentially the whole thing. SegaraBumi indexes a real 24k-file corpus with
deterministic well-alias resolution, a mnemonic dictionary out-populated from the vendor catalogs, a
document-type registry with rule-traced decisions, full-text plus metadata-prefilter search, log
groups on two axes, best-curve ranking, completeness matrices, and a one-way pull contract into
SandiBumi.

**What completes it:** SegaraBumi **v1.5 — the physics validator** plus XLSX capture. One competitor
tabulates; another extracts; *neither validates the extracted numbers against physics before they
enter an interpretation.* That is the piece nobody else has, and it is the natural bridge between the
two products: SegaraBumi finds and validates, SandiBumi computes.

**Why it is supra:** for national-operator-scale Indonesian assets, *"point it at the archive, nothing leaves
the building, every automatic decision carries a rule ID, re-indexing is bit-reproducible"* beats *"an
LLM read your scans"* on the two axes that decide the sale — **data sovereignty and auditability**.
Techlog cannot follow us there without abandoning its own architecture.

### 18.3 Axis 3 — Own the thin-bed / LRLC problem completely

**The weakness:** all three handle laminated shaly sands generically. None ships a complete decision
suite for low-resistivity low-contrast pay.

**What we have:** the richest reference grounding available to anyone — Worthington,
Madjid-Worthington, Thomas-Stieber, Bateman, Klein, Hagiwara/Fanini, Elhadidy, Passey, Mollison —
already read and specified, with the Elhadidy multi-well dip-fit as the no-triaxial fallback that *is*
the common case, since triaxial induction is absent from most legacy well stock. The SSC/SSPW and RtC/IMTS half already ships.

**What ships it:** chapter `17_thinbed-laminated.md`, in the build order the existing thin-bed
specification defines.

**Why it is supra:** it is the exact domain the buyers lose money in, and the one place where being a
working petrophysicist rather than a software vendor is a structural advantage that cannot be bought.

**The claim is rewritten — 2026-08-07, and this is the most commercially important correction in the
document.** This axis previously read *"the most complete low-contrast-pay suite in existence, built
for deltaic Indonesian reservoirs."* `17_thinbed-laminated.md` was asked to substantiate that and
**could not.** Its capability audit of the domain returns **1 of 27 capabilities `PRESENT-OK`**, 6
`PRESENT-DIVERGENT`, 1 `PARTIAL`, and **19 `ABSENT`**. "Most complete" is a breadth claim and breadth
is precisely what is missing.

**A second problem is worse than the count, because a demo would expose it.** The two halves of the
old sentence are not connected in the code. `sw_rtc` and `sw_imts` have no equivalent in any of the
three incumbents — that part is real — but both consume **total porosity**: `lrlc.rs:123` and
`lrlc.rs:228` read `PHIT` (defaulting to `PHIT_SSC`, falling back to `PHIT_SSPW`), not a
laminar-sand porosity. Verified at source. So SandiBumi can produce a Thomas-Stieber decomposition,
and can produce an excess-conductivity saturation, but **it does not compute the saturation on the
sand laminae.** A buyer who asks "so the Sw is on the sand fraction?" gets a no.

**The honest claim, which is still a strong one:**

> **the only tool shipping excess-conductivity low-contrast saturation models alongside a
> Thomas-Stieber decomposition.**

"Alongside" is doing deliberate work in that sentence — it claims co-residence, not integration, and
it stays true today. `01_PRODUCT.md` §6 is the governing rule: an admitted gap costs a feature, a
discovered overclaim costs the deal. This axis was the single most likely place to be tested first,
because it is what the positioning invites a buyer to probe.

**Keep the axis. It is not gap-limited.** Only three items in the whole domain are Tier-C; two of
those are buildable now under the amended `CONTRACT.md` §2.2, and the third is the C-1 patent.
**Nothing in this domain is barred** — the 19 absent capabilities are absent because they are unbuilt,
not because they are unavailable, and `17_thinbed-laminated.md` specifies every one of them with
tests. Connecting the laminar and excess-conductivity halves is the highest-value item on the axis,
and it converts the honest claim back into the ambitious one.

---

## 19. The credibility floor

None of these are leapfrogs. They are the difference between "impressive personal project" and
"product". **Nothing in §18 lands if the app still reads as junior.**

| Gap | Why it matters | Where |
|---|---|---|
| **Installer + auto-update, no dev tools** | An app a colleague cannot install never feels supra regardless of what is inside | `27_ip-install-blockers.md` |
| **In-app method help (F1) — equation *and* citation** | A competitor's help is a PDF. A per-module panel showing the equation and the paper it came from is a genuine feel win, and we already keep the citations | `27_ip-install-blockers.md`; depends on `SB-CORE-004` |
| **Command palette (Ctrl+K)** | Days of work; changes perceived polish disproportionately | Roadmap C3 |
| **2D map window** | One incumbent shipped Mapping in 2025; another has it. A petrophysics tool with no map view reads as junior. The location and deviation data already exist. *Note: a map view was declined for SegaraBumi (2026-07-29); that decision does not bind SandiBumi* | Roadmap C5 |
| **OSDU connector** | Two of three incumbents shipped one within a year. See §20 | Not currently roadmapped |
| **User-authored Python modules in a project `modules/` folder** | One incumbent has a Python installer; another has "incorporate proprietary algorithms". Ours would *exceed* both — manifest-driven with an auto-generated dialog, zero UI code | Roadmap C4 |
| **NMR, then image logs** | The two data-model gaps that let a buyer say "it cannot read my modern suite". **NMR first** — the array-log store already exists and it is a fraction of the image-log lift | `16_nmr.md`, then roadmap C2 |

**Localisation is an underrated asset.** Bahasa Indonesia and Basa Sunda in the UI is something no
incumbent will ever do. For an Indonesian client that is a "built for us" signal money cannot buy.
Protect it; do not let technical terms drift into translation.

---

## 20. OSDU — what it is, and why it is powerful

*This section exists because OSDU is the one credibility-floor item with no entry in either roadmap,
and because it is the least understood of them.*

### 20.1 What it is

**OSDU** — the Open Subsurface Data Universe — is an **open-standard, vendor-neutral, cloud-native
data platform** for the energy industry, governed by The Open Group's OSDU Forum. It is not a product
you buy from a vendor; it is a *specification plus reference implementation* that an operator deploys
to hold its subsurface data: well logs, trajectories, seismic, horizons, reservoirs, production
records. Its purpose is to kill data silos — to make the data independent of whichever application
happens to be reading it this year.

**Timing matters:** The Open Group is releasing the **OSDU Data Platform Standard version 1.0** in
2026 — a stable, frozen subset of APIs with defined behaviour. Until now, "OSDU support" meant chasing
a moving target. From 1.0 onward it is a stable contract worth building against.

### 20.2 The record model — the part that actually matters

OSDU stores everything as **records**, and every record declares a `kind` bound to a **schema**
registered with the platform. Three layers:

| Layer | What it holds | Examples |
|---|---|---|
| **Master data** | The durable real-world entities | `Well`, `Wellbore` — referencing field, basin, prospect, operator |
| **Work Product Components (WPCs)** | The things *about* an entity | a well log, a trajectory, a marker set |
| **Reference data** | Controlled vocabularies | unit systems, curve types, facility types |

Two consequences follow, and they are the whole story:

1. **A wellbore has one canonical identity with a stable ID.** Every WPC hangs off it.
2. **Only fields the schema knows about are indexed by the Search service.** Schema-first is enforced,
   not advisory — you register the schema, then you may ingest.

Ingestion additionally requires a **legal tag** — a record of the legal constraints attached to the
data (ownership, export restriction, residency). Data cannot enter without one. Applications talk to
it over standard REST APIs: Storage, Search, Schema, Workflow, Entitlements.

### 20.3 What a connector actually does

An OSDU connector is a **boundary adapter**, not an architecture. It does four things: **query** the
Search service for wells/wellbores/logs matching a filter; **pull** the selected WPCs and their
master-data parents into the local project, mapping OSDU's schema onto ours; **push** results back as
new WPCs — computed curves, markers, interpretations — attached to the same canonical wellbore with
legal tags carried through; and **reconcile** identity so the same well is not duplicated on either
side. That is it. It adds no petrophysics.

### 20.4 Why it is powerful — five reasons, in order of weight

**1. It solves well identity centrally — the single hardest problem in multi-vintage data.** This is
the exact problem SegaraBumi's alias resolver exists to solve heuristically. In an OSDU estate the
client has *already* done that reconciliation, and every record carries the canonical wellbore ID.
Reading from OSDU means inheriting a solved identity graph instead of re-deriving one. For a
2,000-well field this is the difference between weeks of curation and an afternoon.

**2. It is the procurement gate.** Once an operator standardises on OSDU, software that cannot speak
it is *functionally excluded* — not because it is worse, but because it cannot participate in the data
estate. It stops being a feature comparison and becomes a yes/no question on a tender checklist. Two
incumbents shipping connectors within a year is the signal that this transition is underway, not
theoretical.

**3. Results become assets of record instead of files on a laptop.** Today an interpretation ends as a
project file and a PDF. Pushed back as WPCs against the canonical wellbore, the same interpretation
becomes part of the client's permanent data estate. **This is where Axis 1 compounds**: we would be
the only tool pushing back results that carry full computational lineage. Everyone else pushes
numbers; we would push numbers *with their derivation*.

**4. Legal and entitlement metadata travels with the data.** Legal tags, ownership and residency
constraints are first-class in the record model. For PSC work, joint ventures and NOC data governance
this is not bureaucratic overhead — it is the mechanism that makes sharing legal at all.

**5. It de-risks the format treadmill.** Instead of chasing LAS 3.0, WITSML and every vendor's export
dialect one at a time, one adapter reaches everything an OSDU-adopting client holds.

### 20.5 The honest limits

- **Only large operators run one.** For a small consultancy client, an OSDU connector is worth
  nothing. It is a capability for the national-operator-scale end of the market — which is precisely the end
  this product is aimed at, but the distinction must stay clear.
- **Being OSDU-capable is not the same as needing OSDU.** The one-file offline project remains the
  right answer for every client who does not have a platform. The connector is an *additional* route,
  never a replacement for the local-first architecture.
- **It adds no petrophysics.** It moves data. All the differentiation still has to come from §18.
- **Schema-first cuts both ways.** Pushing our lineage records back means either mapping them onto
  existing kinds or registering custom schemas with the client's platform. That is a real design task,
  not a serialisation detail — and it is exactly where reason 3 is won or lost.

### 20.6 Our design position

**Recommendation: build it as an import/export route in SandiBumi, symmetric with LAS and DLIS — not
as a core architectural change.** The single-file project *is* the product's spine and must not become
optional. OSDU becomes one more way data arrives and results leave, honouring the same delivery-set
model — an OSDU pull is a named set like any other import, and never overwrites.

**Open design tension, for Jauhar's decision:** the connector arguably belongs in SegaraBumi, where
well identity, mnemonic normalisation and curve-selection logic already live. Against that:
SegaraBumi's entire pitch is *nothing leaves the building*, and adding a cloud REST client muddies a
doctrine that is currently clean and provable.

| Option | For | Against |
|---|---|---|
| **A — SandiBumi import/export route** *(recommended)* | Symmetric with LAS/DLIS; keeps SegaraBumi's offline doctrine unblemished; smallest blast radius | Re-implements identity mapping SegaraBumi already does well |
| **B — SegaraBumi, feature-gated** | Reuses the alias resolver and dictionary; an OSDU estate becomes just another indexed root | Breaks the offline-only story, which is a *sales asset*, not just an engineering choice |
| **C — Separate adapter crate, writes to either** | Doctrine stays clean in both; one implementation | A third moving part before either product has shipped v1 |

**Prerequisite before any of this:** confirm a real client actually runs, or is committed to running,
an OSDU platform. This is demand-driven. Build it when a named opportunity requires it — but design
the seam now so it is a fortnight, not a quarter.

---

## 21. Deliberate non-goals from the competitive scan

Stated so they are decisions rather than omissions. `01_PRODUCT.md` §5 and §6.2 hold the formal list;
these are the ones the 2026 scan specifically raises.

- **Log foundation models.** One incumbent's is trained on 18,000+ wells. We cannot compete on
  training data, and a globally-tuned statistical predictor contradicts the determinism doctrine that
  is our differentiator. The existing ML suite — trained on the client's own wells, in-app,
  reproducible — is the right scope. Chapter `24_ml-advanced.md` is where this boundary is specified
  in detail.
- **Multi-user cloud collaboration.** Contradicts the offline single-binary posture that is a sales
  asset in this market.
- **3D petrophysics for high-angle and horizontal wells.** A genuine competitor moat. Given how much
  deltaic development drilling is now high-angle, this deserves a deliberate decision rather than a default skip — but it is a very large
  lift and is not currently justified.
- **Corpus-scale build-time LLM summaries** (SegaraBumi). Killed and the kill stands: model prose in
  the runtime database at scale breaks the audit story.

---

## 22. Capacity — the solo constraint, deliberately held

**Stated position (Jauhar, 2026-07-31):** the product capabilities are known and sufficient to ship.
Software-engineering capacity is deferred — the first engineering hire happens *after* contract revenue
exists. There is no capital to hire before that.

This is a legitimate bootstrap and the plan assumes it. Three consequences follow, and two cost
nothing to address now.

### 22.1 Do now, costs nothing but time — onboarding documentation

Hiring an engineer *later* into an undocumented codebase is expensive at the worst possible moment:
the onboarding cost is paid in Jauhar's review hours, precisely while a contract is being delivered.
The mitigation is to write the onboarding material **while there is slack**, not when there isn't. The
prompts that exist for this have still not been run. Output would be `ARCHITECTURE.md` plus decision
records — the same artefacts that answer a procurement continuity question. One job, two payoffs. This
is `SB-CORE-043`.

### 22.2 Do now, costs little — the continuity answer

State-sector procurement asks about business continuity, and the single-person bus factor is a real
objection. *"I will hire when I have a contract"* is a credible answer **if it is a written plan**, and
a weak one if it is a hope. Three cheap components: a written continuity plan with a named hire
trigger, role and timeline; a **source-code escrow** arrangement, which directly answers the objection
state buyers actually have and costs far less than a hire; and a channel partner as part of an honest
answer.

**v2 note:** escrow has a technical precondition nobody has noticed. `SB-CORE-041` — the tree
currently cannot build and test from a fresh clone. An escrow deposit that a third party cannot build
is worth nothing, so the one-line fixture fix is on the critical path of a commercial commitment.

### 22.3 Decide now — the hire trigger

Decide the threshold *in advance*, in writing: at what contract count or annual revenue does the first
hire happen, and is that hire an engineer or a support/deployment person? Support hours, not
engineering, are the measured ceiling on how many clients one person can serve. Deciding the trigger
before the money arrives prevents the standard failure of never quite feeling ready.

### 22.4 What the constraint does *not* excuse

Capability-complete and verification-complete are different claims, and only the second survives
procurement. R5 now stands at **6.7 %** field-verified. This is not a gap in what the product can do;
it is a gap in what can be *proven*. The real-field pipeline test is the precedent: running against
one real field surfaced two genuine bugs that no amount of synthetic testing had found.

Closing R5 needs no capital — it needs real wells, which is exactly what a named private pilot
supplies. **It is the highest-value use of the pre-revenue period**, and v2's measurement makes that
more true than v1's did, not less.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
