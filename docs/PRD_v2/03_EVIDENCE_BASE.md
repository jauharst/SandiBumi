# PRD v2 · Part II — The evidence base and the expansion thesis

**Sections §12–§14.** The cross-cutting requirements this evidence generates are in
`04_CORE_REQUIREMENTS.md` (§15); the domain requirements are in the eighteen chapters.

---

## 12. What was done, and what it found

Between 2026-07-31 and 2026-08-07 the three incumbent suites were cross-validated domain by domain,
working from **T1 executable and declarative source where held** — Geolog `.lls` Loglan sources,
Techlog `.py` modules, shipped `.par` and `.info` parameter files — and from ingested manuals (T2),
equation rasters (T3) and locally-held secondary material (T4) elsewhere.

**Output: eighteen dossiers, 42,936 lines** at `docs/research_2026-08/cross_tool/`.

Every one was then reviewed by an adversarial critic whose brief was to find defects, not to approve.
**All eighteen returned material issues; none passed clean.** Roughly 430 findings were fixed and
about 15 were rebutted with evidence — the critics were wrong often enough that deferring to them
would have introduced errors. Each dossier ends in a per-finding disposition section recording which.

The evidence tiers are carried into every chapter and every requirement, so a reader can see how far
any claim is from executable source:

| Tier | Meaning |
|---|---|
| **T1** | Executable or declarative source read directly |
| **T2** | Vendor manual or help text ingested as text |
| **T3** | Vendor raster — equation image or scanned chart — read visually |
| **T4** | Course notes, project records, secondary literature held locally |

The tier is not decoration. A T1 claim can be re-checked by opening a file; a T3 claim depends on
someone having read a raster correctly, and §12.1 contains a case where exactly that went wrong and
was caught.

### 12.1 Findings that changed conclusions

These are the reason the critic gate was worth its cost, and each one falsified something an earlier
pass had asserted:

- **The Hertz-Mindlin adhesion fraction was silently hard-coded to 1.** IP ships **0.5** (its Stiff
  Sand model forces 1) applied *inside* `G_HM` via the prefactor; Techlog ships a
  `ShearReductionFactor` of **0.4** applied *outside*, as a final multiply on finished shear; Geolog
  exposes no such parameter at all. **These are not the same knob.** Porting either default into the
  other's equation form yields a plausible, wrong shear modulus — about 42 % error, silent.
- **A "triple-corroborated at 5 decimal places" Greenberg-Castagna coefficient set was fabricated.**
  The Geolog page holds none of the digits — it says they are stored inside the code. The real ledger
  records 3 dp from two rasters and one vendor. An earlier pass had asserted a precision that does
  not exist anywhere on this machine.
- **Techlog ships an automatic hydraulic-flow-unit partitioner** — a Lorenz-curve inflection routine,
  225 lines of readable Python — that four earlier passes never opened. It falsifies the "only IP
  ships clustering" headline outright.
- **Geolog ships 126 environmental-correction modules across 10 vendor families**, found only by
  enumerating `bin\*.info` manifests; earlier passes had read only the `loglan\*.lls` sources. Its
  "fail-loud" reputation turns out to be a property of **the manifests, not the code** — a port
  lifting the algorithm without the manifest's VALIDATION columns inherits a fail-**silent** version.
  This is the single most transferable finding in the corpus, and it is why `SB-CORE-003` exists.
- **The robust-statistic masking threshold has a closed form**, `f* = 1/(k²+1)` — exactly 20 % at
  k = 2, independent of spike amplitude and set entirely by k. So *tightening* the cutoff makes
  masking worse, which is the opposite of the intuition a user brings to it.

### 12.2 The standing decision on disputed parameters

**Where vendors disagree and no adjudication is defensible, the parameter ships absent, not
defaulted.** This was tested during the review: a critic instructed a reviser to canonicalise one
vendor's normal-pressure gradient over five other shipped values, and the reviser correctly refused
it as silent adjudication disguised as a default.

The reasoning is a product argument, not a purity one. A default hides a choice the interpreter is
accountable for. Absence forces the choice into the open, where the provenance machinery can record
it, and where it appears in the deliverable as a decision with a name attached rather than as a
number nobody remembers picking.

### 12.3 One recorded transcription exception

One vendor parameter file — plain text, self-documenting, user-extensible by its own header, and
holding a digitization of a **published 1967 paper** — had its coefficient rows transcribed into a
dossier, in contrast to the vendor lookup tables that were deliberately not transcribed. It was
escalated as an open rule-boundary call and **ruled on by Jauhar directly on 2026-08-07: keep them.**
The values are marked non-adoptable; any implementation re-derives from the original paper.

**It is scoped to those rows and it is not a precedent.** No chapter may reason from it to any other
file. `CONTRACT.md` §2.1 states the rule; a chapter that believes it has a second case escalates
rather than decides.

---

## 13. The eighteen domain chapters

Each chapter specifies one domain: what the incumbents do that generates an obligation, what
SandiBumi's code does today at `file.rs:line`, the requirements that follow, every parameter with its
source or its recorded absence, the acceptance tests, and a traceability table accounting for
**every** item in its dossier.

| Chapter | Domain | ID code | Dossier lines |
|---|---|---|---|
| `10_clay-volume.md` | Clay and shale volume | `CLY` | 2,728 |
| `11_porosity.md` | Porosity | `POR` | 1,980 |
| `12_saturation.md` | Water saturation | `SAT` | 1,856 |
| `13_mineral-solver.md` | Multi-mineral solver | `MIN` | 2,205 |
| `14_cutoffs-summation-mc.md` | Cutoffs, summation, Monte Carlo | `CUT` | 3,627 |
| `15_sat-height-rocktyping.md` | Saturation-height and rock typing | `SHR` | 2,480 |
| `16_nmr.md` | Nuclear magnetic resonance | `NMR` | 2,406 |
| `17_thinbed-laminated.md` | Thin-bed and laminated analysis | `TBD` | 1,880 |
| `18_geomech-ppfg.md` | Geomechanics, pore pressure, fracture gradient | `GEO` | 2,244 |
| `19_toc-unconventional.md` | TOC and unconventional | `TOC` | 2,314 |
| `20_envcorr-qc.md` | Environmental corrections and log QC | `ENV` | 2,841 |
| `21_data-io.md` | Data import, export, formats | `DIO` | 1,865 |
| `22_database-model.md` | Database and project data model | `DBM` | 1,549 |
| `23_plotting-interactivity.md` | Plotting, display, interactivity | `PLT` | 2,416 |
| `24_ml-advanced.md` | Machine learning and advanced analysis | `MLA` | 2,222 |
| `25_fluidsub-rockphysics.md` | Fluid substitution and rock physics | `RPH` | 1,879 |
| `26_production-logging.md` | Production logging | `PLG` | 2,965 |
| `27_ip-install-blockers.md` | Install, deployment, packaging blockers | `INS` | 3,478 |

Chapters cite dossier sections; they do not restate them. A chapter that reads like a summary of its
dossier has failed the contract — see `CONTRACT.md` §0.

### 13.1 A known coverage gap in the eighteen — escalated, not decided

**No chapter owns the user-programming and formula layer.** The eighteen domains were derived from the
eighteen evidence dossiers, and the dossiers were organised by petrophysical domain — so Rhai and
Python user equations, the formula parser and the expression evaluator fall between all of them.
`24_ml-advanced.md` surfaced this while dispositioning, along with two defects that are real, testable
and orphaned: an operator-precedence error in `MIN()`, and a **silent 57.2958× error from a
degrees/radians mismatch** in the trigonometric functions. Both were dispositioned `ESCALATED` rather
than absorbed into a domain they do not belong to.

The second one is the exact failure shape this document treats as most serious — a unit error that
computes, plots and ships. It is also cross-cutting: any user equation in any domain inherits it.

**This is Jauhar's call and no agent may take it.** Three options, none obviously right: fold it into
`22_database-model.md` (wrong domain, but that chapter owns the persistence and evaluation seam); give
it a nineteenth chapter with no dossier behind it (honest, but breaks the one-chapter-per-dossier
contract); or raise the two defects as `SB-CORE` requirements and leave the layer unspecified until a
dossier exists for it. Recorded here so the gap is visible rather than discovered later as an omission.

---

## 14. The four sources of genuine advantage

The commission is to make SandiBumi *far* better than what the three incumbents ship, not to reach
parity. Four things generate that. Every chapter is instructed to raise an opportunity in any of them
as a requirement rather than leave it as an observation.

### 14.1 The vendors' own defects are the opportunity

The corpus found, in shipped products from three major vendors: an equation raster printing a **sum
where a product is required**; a **missing cube root** worth eighteen orders of magnitude; an
**inverted unit constant** (a 47.54× error); a coefficient whose shipped **range floor is three
orders of magnitude below its documented floor**, so a mistyped constant the manual would reject is
silently accepted; a shipped constant **192× off its own manual**; a **doubled negation** in a
denominator; a documented equation contradicting the same vendor's own equation image; and a
sandstone coefficient triplet **labelled as feldspar in a different module of the same product**.

Doing these correctly costs nothing but care, and it is a differentiator that cannot be
out-resourced — a vendor with three decades and hundreds of engineers still shipped them.

**The discipline this imposes:** SandiBumi does not copy a vendor's number to be compatible with a
vendor's error. Where a vendor's shipped behaviour is wrong, SandiBumi is right and says so, with the
primary source.

### 14.2 Where the vendors disagree, the disagreement is the product

Three packages routinely ship three different values for one constant, and none of them tells the
interpreter that the others exist. The corpus holds this comparison for eighteen domains.

Surfacing it — "this parameter has three shipped vendor values; here they are with sources; pick one
and your choice is recorded" — is a capability no incumbent can offer, because none of them can
credibly publish their competitors' defaults. It converts the standing absent-not-defaulted rule
(§12.2) from a limitation into the reason to buy.

Specified as `SB-CORE-013`.

### 14.3 Fail loud where they fail silent

The single most repeated pattern across all three tools: a computation proceeds on inputs outside its
stated validity and returns a plausible number. Sometimes the validity condition is in the manual;
sometimes it is a machine-readable column in a manifest the algorithm itself never reads.

Carrying validity conditions as **enforced preconditions in the data model** — not as documentation,
not as a comment — is cheap, unmatched, and directly serves the user in `01_PRODUCT.md` §3.1 whose
failure mode is a number they cannot defend.

**This applies inward as hard as outward.** The product currently violates its own cardinal rule in
at least seven named places (R15). Fixing those is a precondition for making the claim at all.

Specified as `SB-CORE-003` (outward) and `SB-CORE-002` (inward).

### 14.4 Provenance is structural here and conventional there

None of the three can answer, end to end: *how was this curve made — which inputs, which parameter
values, from which source, by whom, when — and does re-running produce an identical number?* Their
audit trails are activity logs, not lineage graphs.

A parameter that carries the paper it came from, through the computation, into the deliverable, is a
claim no incumbent can make. It is also the property that converts this project's strictest internal
engineering constraint into a sellable one.

Specified as `SB-CORE-010`, `-011`, `-012`, and it is Axis 1 of the strategy in `05_STRATEGY.md`
§18.1.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
