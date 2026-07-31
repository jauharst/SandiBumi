# SandiBumi + SegaraBumi — Future Plan

**Written 2026-07-31.** Cross-product strategic plan: where the two products stand, what the
incumbents shipped in 2024–2026, and the sequenced path to being *superior relatives* of
Geolog, Techlog, and Interactive Petrophysics rather than imitations of them.

## 0. How to read this

This document sits **above** the roadmaps, not inside them.

| Document | Answers |
|---|---|
| `docs/PRD.md` | What SandiBumi **is** today |
| `ROADMAP.md` | What SandiBumi will **build**, in order |
| `D:\XX. SegaraBumi\docs\PRD.md` | What SegaraBumi is and its phase plan |
| **This file** | **Why** those items, in **what order across both products**, and what we are betting on |

Nothing here overrides a decision already recorded in those files. Where this plan touches an
item they already own, it cites the item rather than restating it. Competitive claims are dated
— re-verify before quoting them to a client.

---

## 1. Where we actually stand — 2026-07-31

### SandiBumi

Phases 1–10 code-complete. Shipped: dockview workspace, WebGPU log views, the full
deterministic module library, SandiMin (multimin2), the Jauhar method suite (SSC/SSPW,
LRLC RtC/IMTS), saturation-height, rock typing, Monte Carlo, workflow chains, field dashboard,
electrofacies + ML suite, composite plots + PDF reports, delivery sets across every store,
open-path hardening, and the chartbook overlay library.

Open work is in `ROADMAP.md` Part B (hardening + interpretation-workflow queue) and Part C
(method-suite waves, new data models, trust & reproducibility, platform & extensibility).

### SegaraBumi

Much further along than an outside reading of the roadmap suggests. **P6 gate closed
2026-07-29**: 23,976 files force-migrated across three roots, 9,096 datasets classified on both
axes, 4,092 attributes harvested, desktop GUI shipped, P4 semantic tier shipped (separate
model-stamped store, degrades cleanly to exact-only), P5 SEGARA pull contract live
(`docs/SEGARA-CONTRACT.md`).

Remaining to v1: GUI fuse toggle, the §7.4 bilingual fixture set, installer, own icons.
Then **v1.5 = physics validator + XLSX capture** — which is the piece that completes the moat
described in §3.2 below.

### What this means

The two products are much closer to the strategic position than the feature lists imply. The
work ahead is **finishing and framing**, not starting.

---

## 2. The competitive picture — researched 2026-07-31

### What the incumbents shipped, 2024–2026

| Tool | Recent capability |
|---|---|
| **Techlog** (SLB) | **AI Import** — LLM extraction from unstructured legacy archives (TIF, PDF, scans, handwritten notes). **Log foundation model** trained on 18,000+ public wells, locally tunable, no GPU required. **3D Petrophysics** for high-angle/horizontal wells — wellbore-centric grid, azimuthal petrophysics. ExpertDip → local layer geometry. |
| **Geolog** (AspenTech) | ML-enhanced evaluation with uncertainty quantification. **Aspen Epos** — multi-user simultaneous collaboration, 10K+ well projects. **OSDU connector** via Subsurface Connector. Modular hooks for proprietary algorithms. |
| **Interactive Petrophysics** (Geoactive) | **2025**: Mapping module (seismic surfaces, faults, GIS shapefiles, spatial query, polygons), **OSDU connector**, parent-well grouping for sidetracks, pinned plots. **2026**: new modules — Sonic Saturation (patented; in our Tier-C register), Multi-Well Experienced Eye, Pyrolysis; multi-regressions on crossplots; streamlined Python installer; upgraded Casing Inspection, Formation Testing, Image Analysis, Mapping. |

### Three signals

1. **The archive→data problem is the live frontier.** Techlog's flagship new capability is
   ingesting legacy unstructured archives. It is LLM-based and non-deterministic.
2. **OSDU has become table stakes.** Two of three shipped connectors inside a year. See §5.
3. **Everyone is bolting Python on.** IP ships a "streamlined Python installer"; Geolog
   advertises hooks for proprietary algorithms. SandiBumi was born with Python/numpy as the
   scripting surface — this is a lead, not a gap.

### The honest constraint

We cannot out-feature Techlog. It has three decades and hundreds of engineers. Every plan
below is chosen because it exploits a **structural** weakness the incumbents cannot fix without
abandoning their own architecture — not because it is a feature they merely haven't got to.

---

## 3. The positioning — three asymmetric axes

### 3.1 Axis 1 — Reproducibility as a product, not a feature

**The weakness:** none of the three can answer, end to end, *how was this curve made — which
inputs, which parameter values, from which source, by whom, when — and does re-running produce
an identical number?* Their audit trails are activity logs, not lineage graphs. The market has
noticed: newer entrants now advertise "full audit trail" as a differentiator, which only works
as a pitch because the incumbents are weak there.

**What we already have:** PK-less `computed_curves` with a strict write discipline; versioned
log sets with provenance; undo everywhere a cell or property changes; a deterministic
manifest-driven module framework; Monte Carlo seeded from `(seed, index)`; and SegaraBumi's
proven doctrine of rule IDs and byte-identical re-indexing.

**What ships it:** `ROADMAP.md` **C3 / Phase 11** — the `runs` table, "how was I made?"
ancestry for any computed curve, named interpretation scenarios with A/B diff.

**Why it is supra, not parity:** re-run a whole field from raw LAS to pay summary and get
byte-identical numbers, with a diffable record of every parameter and its source. That is what
a reserve audit, a PSC submission, or a partner data room actually needs, and no incumbent can
produce it. It also converts our strictest engineering constraint — the write discipline — from
an internal rule into a **sellable property**.

**Extension worth designing now:** the parameter-source rule (defaults trace to `docs/`, a
reference-suite export, the chartbook, or a past study) should become *machine-recorded*, not
just a convention. A curve whose lineage names the paper its `m` came from is a claim no
competitor can make.

### 3.2 Axis 2 — The deterministic archive bridge (SegaraBumi)

**The weakness:** Techlog AI Import is the most-promoted new capability in the market, and it
is LLM-based, non-deterministic, and — in practice — cloud-adjacent. It cannot tell a client
*why* it decided a file belonged to a well, and it cannot promise the same answer twice.

**What we already have:** essentially the whole thing. SegaraBumi P6 indexes a real 24k-file
corpus with deterministic well-alias resolution, a mnemonic dictionary out-populated from the
Techlog/IP catalogs, a document-type registry with rule-traced decisions, FTS5 + metadata
pre-filter search, log groups on two axes, a best-curve ranking, completeness matrices, and a
one-way pull contract into SandiBumi.

**What completes it:** SegaraBumi **v1.5 — the physics validator** (+ XLSX capture). SONAR
tabulates; Techlog extracts; *neither validates the extracted numbers against physics before
they enter an interpretation.* That is the piece nobody else has, and it is the natural bridge
between the two products: SegaraBumi finds and validates, SandiBumi computes.

**Why it is supra:** for Pertamina-scale Indonesian assets, *"point it at the archive, nothing
leaves the building, every automatic decision carries a rule ID, re-indexing is
bit-reproducible"* beats *"an LLM read your scans"* on the two axes that decide the sale —
**data sovereignty and auditability**. Techlog cannot follow us there without abandoning its
own architecture.

### 3.3 Axis 3 — Own the thin-bed / LRLC problem completely

**The weakness:** all three handle laminated shaly sands generically. None ships a complete
decision suite for low-resistivity low-contrast pay.

**What we have:** the richest reference grounding available to anyone — Worthington,
Madjid-Worthington, Thomas-Stieber, Bateman, Klein, Hagiwara/Fanini, Elhadidy, Passey,
Mollison — already read and specified in `docs/research_2026-07/ref_thin_bed_lrlc.md`, with the
Elhadidy multi-well dip-fit as the no-triaxial fallback that *is* the Mahakam case. `ssc.rs`
and `lrlc.rs` already ship the SSC/SSPW and RtC/IMTS half.

**What ships it:** `ROADMAP.md` **C1 item (10)**, in the build order that spec already defines.

**Why it is supra:** "the most complete low-contrast-pay suite in existence, built for deltaic
Indonesian reservoirs" is a defensible claim in the exact domain the buyers lose money in. It
is also the one place where being a working petrophysicist rather than a software vendor is a
structural advantage that cannot be bought.

---

## 4. The credibility floor

None of these are leapfrogs. They are the difference between "impressive personal project" and
"product." Nothing in §3 lands if the app still reads as junior.

| Gap | Why it matters | Roadmap |
|---|---|---|
| **Installer + auto-update, no dev tools** | An app a colleague cannot install never feels supra regardless of what is inside. SegaraBumi has NSIS configured but unbuilt; SandiBumi has none. | SandiBumi C4; SegaraBumi v1 |
| **In-app method help (F1) — equation *and* citation** | Techlog's help is a PDF. A per-module panel showing the equation and the paper it came from is a genuine feel win, and we already keep the citations. | SandiBumi C4 |
| **Command palette (Ctrl+K)** | Days of work; changes perceived polish disproportionately. | SandiBumi C3 |
| **2D map window** | IP shipped Mapping in 2025; Geolog has it. A petrophysics tool with no map view reads as junior. `geo.rs`, `deviation.rs` and imported locations already hold the data. **Note: map view was declined for *SegaraBumi* (2026-07-29) — that decision does not bind SandiBumi, where it is already roadmapped.** | SandiBumi C5 |
| **OSDU connector** | Two of three incumbents shipped one within a year. See §5. | Not currently roadmapped |
| **User-authored Python modules in a project `modules/` folder** | IP has a "Python installer"; Geolog has "incorporate proprietary algorithms." Ours would *exceed* both — manifest-driven with an auto-generated dialog, zero UI code. | SandiBumi C4 |
| **NMR, then image logs** | The two data-model gaps that let a buyer say "it cannot read my modern suite." **NMR first** — `array_logs` already exists and it is a fraction of the image-log lift. | SandiBumi C2 |

**Localisation is an underrated asset.** Bahasa Indonesia and Basa Sunda in the UI is something
no incumbent will ever do. For an Indonesian client that is a "built for us" signal money
cannot buy. Protect it; do not let technical terms drift into translation (already the rule).

---

## 5. OSDU — what it is, and why it is powerful

*This section exists because OSDU is the one credibility-floor item with no entry in either
roadmap, and because it is the least understood of them.*

### 5.1 What it is

**OSDU** — the Open Subsurface Data Universe — is an **open-standard, vendor-neutral,
cloud-native data platform** for the energy industry, governed by The Open Group's OSDU Forum.
It is not a product you buy from a vendor; it is a *specification plus reference
implementation* that an operator deploys (on Azure, AWS, or Google) to hold its subsurface
data: well logs, trajectories, seismic, horizons, reservoirs, production records.

Its purpose is to kill data silos — to make the data independent of whichever application
happens to be reading it this year.

**Timing matters:** The Open Group is releasing the **OSDU Data Platform Standard version 1.0**
in 2026 — a stable, frozen subset of APIs with defined behaviour. Until now, "OSDU support"
meant chasing a moving target. From 1.0 onward it is a stable contract worth building against.

### 5.2 The record model — the part that actually matters

OSDU stores everything as **records**, and every record declares a `kind`. A `kind` is bound to
a **schema** registered with the platform. Three layers:

| Layer | What it holds | Examples |
|---|---|---|
| **Master data** | The durable real-world entities | `Well`, `Wellbore` — referencing field, basin, prospect, operator |
| **Work Product Components (WPCs)** | The things *about* an entity | a well log, a trajectory, a marker set |
| **Reference data** | Controlled vocabularies | unit systems, curve types, facility types |

Two consequences follow, and they are the whole story:

1. **A wellbore has one canonical identity with a stable ID.** Every WPC hangs off it.
2. **Only fields the schema knows about are indexed by the Search service.** Schema-first is
   enforced, not advisory — you register the schema, then you may ingest.

Ingestion additionally requires a **legal tag** — a record of the legal constraints attached to
the data (ownership, export restriction, residency). Data cannot enter without one.

Applications talk to it over standard REST APIs: **Storage**, **Search**, **Schema**,
**Workflow**, **Entitlements**.

### 5.3 What a connector actually does

An OSDU connector is a **boundary adapter**, not an architecture. It does four things:

1. **Query** the Search service for wells/wellbores/logs matching a filter.
2. **Pull** the selected WPCs and their master-data parents into the local project, mapping
   OSDU's schema onto ours (curve names, units, depth reference).
3. **Push** results back as new WPCs — computed curves, markers, interpretations — attached to
   the same canonical wellbore, with legal tags carried through.
4. **Reconcile** identity so the same well is not duplicated on either side.

That is it. It adds no petrophysics. It changes where data comes from and where results go.

### 5.4 Why it is powerful — five reasons, in order of weight

**1. It solves well identity centrally — the single hardest problem in multi-vintage data.**
This is the exact problem SegaraBumi's alias resolver exists to solve heuristically:
`RAMD-14` ↔ `RAMOS DELTA-14`. In an OSDU estate the client has *already* done that
reconciliation, and every record carries the canonical wellbore ID. Reading from OSDU means
inheriting a solved identity graph instead of re-deriving one. For a 2,000-well field this is
the difference between weeks of curation and an afternoon.

**2. It is the procurement gate.** Once an operator standardises on OSDU, software that cannot
speak it is *functionally excluded* — not because it is worse, but because it cannot
participate in the data estate. It stops being a feature comparison and becomes a yes/no
question on a tender checklist. Geolog and IP both shipping connectors within a year is the
signal that this transition is underway, not theoretical.

**3. Results become assets of record instead of files on a laptop.** Today a SandiBumi
interpretation ends as a `.duckdb` file and a PDF. Pushed back as WPCs against the canonical
wellbore, the same interpretation becomes part of the client's permanent data estate,
discoverable by every other application they run. **This is where Axis 1 compounds**: we would
be the only tool pushing back results that carry full computational lineage. Everyone else
pushes numbers; we would push numbers *with their derivation*.

**4. Legal and entitlement metadata travels with the data.** Legal tags, ownership, residency
constraints are first-class in the record model. For PSC work, joint ventures, and NOC data
governance, this is not bureaucratic overhead — it is the mechanism that makes sharing legal
at all. A connector that honours legal tags is doing compliance work the client would otherwise
do by email.

**5. It de-risks the format treadmill.** Instead of chasing LAS 3.0, WITSML, and every vendor's
export dialect one at a time, one adapter reaches everything an OSDU-adopting client holds.

### 5.5 The honest limits

- **Only large operators run one.** Standing up an OSDU platform takes the capacity of a major
  or a national oil company. For a small consultancy client, an OSDU connector is worth
  nothing. It is a capability for the Pertamina-scale end of the market — which is precisely
  the end this product is aimed at, but the distinction must stay clear.
- **Being OSDU-capable is not the same as needing OSDU.** The one-file offline project remains
  the right answer for every client who does not have a platform. The connector is an
  *additional* route, never a replacement for the local-first architecture.
- **It adds no petrophysics.** It moves data. All the differentiation still has to come from
  §3.
- **Schema-first cuts both ways.** Pushing our lineage records back means either mapping them
  onto existing kinds or registering custom schemas with the client's platform. That is a real
  design task, not a serialisation detail — and it is exactly where reason 3 above is won or
  lost.

### 5.6 Our design position

**Recommendation: build it as an import/export route in SandiBumi, symmetric with LAS and
DLIS — not as a core architectural change.**

Rationale: the DuckDB single-file project *is* the product's spine and must not become
optional. OSDU becomes one more way data arrives and results leave, sitting beside
`ingest.rs`'s existing routes and honouring the same delivery-set model (an OSDU pull is a
named set like any other import — it never overwrites).

**Open design tension, for Jauhar's decision:** the connector arguably belongs in SegaraBumi,
because that is where well identity, mnemonic normalisation, and the curve-selection logic
already live — an OSDU estate is just another corpus to catalog. Against that: SegaraBumi's
entire pitch is *nothing leaves the building*, and adding a cloud REST client muddies a
doctrine that is currently clean and provable. Options:

| Option | For | Against |
|---|---|---|
| **A — SandiBumi import/export route** *(recommended)* | Symmetric with LAS/DLIS; keeps SegaraBumi's offline doctrine unblemished; smallest blast radius | Re-implements identity mapping SegaraBumi already does well |
| **B — SegaraBumi, feature-gated** | Reuses the alias resolver and dictionary; OSDU estate becomes just another indexed root | Breaks the offline-only story, which is a *sales asset*, not just an engineering choice |
| **C — Separate adapter crate, writes to either** | Doctrine stays clean in both; one implementation | A third moving part before either product has shipped v1 |

**Prerequisite before any of this**: confirm a real client actually runs, or is committed to
running, an OSDU platform. This is a demand-driven item. Build it when a named opportunity
requires it — but design the seam now so it is a fortnight, not a quarter.

---

## 6. The sequenced plan

Ordering principle: **finish what is nearly done before starting what is merely valuable.**

### Tier 0 — Feel (weeks, both products)

The highest perception change per unit effort. Nothing in §3 reads as credible without it.

- SandiBumi: installer + auto-update; in-app method help (F1) with equation + citation;
  command palette.
- SegaraBumi: close the remaining v1 items — GUI fuse toggle, §7.4 bilingual fixture set,
  installer build, own icons (currently SandiBumi placeholders).

### Tier 1 — The claim (SandiBumi C3 / Phase 11)

Lineage + named scenarios + A/B diff + byte-reproducible re-run. Our strongest differentiator,
and the foundations are further along than the roadmap admits — the write discipline, log-set
provenance, and seeded Monte Carlo are already down-payments.

### Tier 2 — The moat (SegaraBumi v1.5)

Physics validator + XLSX capture. Completes the deterministic answer to Techlog AI Import and
creates the bridge no competitor has: find → **validate** → compute.

### Tier 3 — The depth (SandiBumi C1 item 10)

The thin-bed / LRLC suite, in the build order `ref_thin_bed_lrlc.md` already specifies.

### Tier 4 — Floor, in value order

2D map window → NMR (`array_logs` exists) → OSDU connector *(demand-driven, see §5.6)* →
image logs.

### What each tier lets us say

| After | The claim |
|---|---|
| Tier 0 | "It is a product." |
| Tier 1 | "It is the only one that can prove how a number was made." |
| Tier 2 | "It reads your archive without your data leaving the building — and checks the physics." |
| Tier 3 | "It is the best low-contrast-pay tool in existence." |
| Tier 4 | "It fits your estate." |

---

## 7. Deliberate non-goals

Stated so they are decisions rather than omissions. `docs/PRD.md` §5 and §6.2 hold the formal
list; these are the ones the 2026 competitive scan specifically raises.

- **Log foundation models.** Techlog's is trained on 18,000+ wells. We cannot compete on
  training data, and a globally-tuned statistical predictor contradicts the determinism
  doctrine that is our differentiator. The existing ML suite (`ml.rs`, `facies.rs`) — trained
  on the client's own wells, in-app, reproducible — is the right scope.
- **Multi-user cloud collaboration.** Contradicts the offline single-binary posture that is a
  sales asset in this market.
- **3D petrophysics for high-angle/horizontal wells.** A genuine Techlog moat. Given Mahakam
  horizontals this deserves a deliberate decision rather than a default skip — but it is a very
  large lift and is not currently justified.
- **Corpus-scale build-time LLM summaries** (SegaraBumi). Killed in FINDINGS §4 and the kill
  stands: model prose in the runtime DB at scale breaks the audit story.

---

## 8. Open questions

> **Commercial strategy** — market access, pricing, and route to market — is held in
> `docs/commercial/`, which is **gitignored by decision (2026-07-31)**: it covers partner
> structures and IP positions that are not shared-repo material. The product questions below are
> the ones that belong here. Three of them have commercial answers recorded there.

0. ~~Which product leads to market?~~ **DECIDED 2026-07-31 (Jauhar): SandiBumi ships first.**
   SegaraBumi remains the archive bridge (Axis 2) and continues to v1/v1.5 on its own track, but
   the commercial motion leads with the interpretation engine. Tier 0 is therefore
   SandiBumi-first: installer, in-app method help, command palette.
   *Engineering capacity is deliberately deferred — first engineering hire happens after contract
   revenue exists, not before. See §9.*
1. **OSDU trigger.** Is any named client running or committed to an OSDU platform? Until yes,
   §5 stays a design-the-seam item, not a build item.
2. **OSDU connector home** — option A, B, or C in §5.6.
3. **3D petrophysics / HAHz** — deliberate skip, or a Wave item once Tier 1–3 land?
4. **Lineage granularity** (Tier 1). Per-run, or per-sample-provenance for edited curves? The
   second is much more expensive and may not be needed for an audit to pass.
5. Whether the parameter-source rule (§3.1) becomes a machine-enforced field on `zone_params`
   and the per-well parameter table, or stays a documentation convention.

---

## 9. Capacity — the solo constraint, deliberately held

**Stated position (Jauhar, 2026-07-31):** the product capabilities are known and sufficient to
ship. Software-engineering capacity is deferred — the first engineering hire happens *after*
contract revenue exists. There is no capital to hire before that.

This is a legitimate bootstrap and the plan assumes it. Three consequences follow, and two of them
cost nothing to address now.

### 9.1 Do now, costs nothing but time — onboarding documentation

Hiring an engineer *later* into an undocumented codebase is expensive at the worst possible
moment: the onboarding cost is paid in Jauhar's review hours, precisely while a contract is being
delivered. The mitigation is to write the onboarding material **while there is slack**, not when
there isn't.

`stewardship_prompt.md` Prompts 2 and 4 exist for exactly this and have not been run
(`PRD.md` R9). Output would be `ARCHITECTURE.md` + ADRs — the same artifacts that answer a
procurement continuity question. One job, two payoffs.

### 9.2 Do now, costs little — the continuity answer

State-sector procurement asks about business continuity, and `PRD.md` R9 (single-person bus
factor) is a real objection. *"I will hire when I have a contract"* is a credible answer **if it is
a written plan**, and a weak one if it is a hope. Three cheap components:

1. A written continuity plan — named hire trigger, role, timeline.
2. ⚖ A **source-code escrow** arrangement. Directly answers the objection state buyers actually
   have, and costs far less than a hire.
3. FERG as channel partner is part of an honest answer (see `commercial/PLAN.md` §5).

### 9.3 Decide now — the hire trigger

Decide the threshold *in advance*, in writing: at what contract count or annual revenue does the
first hire happen, and is that hire an engineer or a support/deployment person? Support hours, not
engineering, are the measured ceiling on how many clients one person can serve
(`commercial/PLAN.md` §3). Deciding the trigger before the money arrives prevents the standard
failure of never quite feeling ready.

### 9.4 What the constraint does *not* excuse

Capability-complete and verification-complete are different claims, and only the second survives
procurement. `PRD.md` R5 stands at 19.5% field-verified — 298 of 370 checklist items never
exercised against real data. This is not a gap in what the product can do; it is a gap in what can
be *proven*. The BLSO real-data pipeline test is the precedent: running against one real field
surfaced two genuine bugs (LAS alias coverage, `sw_rtc`/`sw_imts` default RT input) that no amount
of synthetic testing had found.

Closing R5 needs no capital — it needs real wells, which is exactly what a named private pilot
supplies. It is the highest-value use of the pre-revenue period.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
