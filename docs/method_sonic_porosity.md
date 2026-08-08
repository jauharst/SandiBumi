# Sonic transit time to porosity — the cited primaries

**Extracted 2026-08-08 from the three primary papers, supplied by Jauharst.** This closes the
evidence gap `_SPINE_PENDING.md` **SP-004** recorded against `13_mineral-solver.md` §7.5 candidate
**MIN-C2-1** and acquisition item **ACQ-11**.

Sources, all read directly:

| Tag | Paper |
|---|---|
| **W56** | Wyllie, Gregory & Gardner, *Elastic wave velocities in heterogeneous and porous media*, 1956 |
| **W58** | Wyllie, Gregory & Gardner, *An experimental investigation of factors affecting elastic wave velocities in porous media*, 1958 |
| **RHG80** | Raymer, Hunt & Gardner, *An improved sonic transit time-to-porosity transform*, SPWLA Twenty-First Annual Logging Symposium, July 8–11 1980 |

**A caution on transcription.** The equations below were read out of scanned PDFs whose text layer
is imperfect — subscripts and exponents in particular. **The forms are stated here as understood;
verify the exact typography against the paper before writing code**, and cite the page. Nothing in
this note should reach `modules.rs` unverified. The *prose* quotations are reliable and are given
verbatim.

---

## 1. The time-average equation (W56, W58)

`1/V = φ/Vf + (1 − φ)/Vma`

**W56 states the theoretical condition under which it holds** — the sentence is the whole reason a
laminated-medium average applies to rock at all:

> *"For such a system the theoretical velocity is the time-average velocity if the wave travel is
> directly through the pile normal to the discs whether wave lengths are large or small compared
> with the thicknesses of the segments. This follows from the fact that wave velocities and
> transmission coefficients are independent of frequencies for elastic media when there is no
> slippage or separation of interfaces."*

**W58's abstract states the limits, and they are limits of applicability, not of accuracy:**

> *"It is shown that the time-average relationship cannot be applied to determine the total
> volumetric porosity of carbonate rocks which are vugular and fractured."*

W58 also records, from the differential-pressure cell: velocity **increases with increasing
differential pressure**, rapidly at first then flattening; behaviour at zero differential pressure
is materially different; and the effects of **oil and gas saturation are "comparatively minor."**

And the authors' own caveat on laboratory transfer, worth carrying into any test:

> *"Owing to instrumental limitations, it cannot necessarily be assumed that measurements made in
> the laboratory are directly applicable to the interpretation of velocity data obtained under
> field [conditions]."*

---

## 2. The lack-of-compaction factor `Cp` — cited range at last (RHG80)

`φc = φ_apparent / Cp`, where `φ_apparent` comes straight from the time-average equation.

**RHG80 gives the range verbatim:**

> *"Cp is always greater than unity. Values ranging from 1 to 1.3 are common, with values as high
> as 1.8 occasionally observed."*

**And the simplest estimator, also verbatim:**

> *"The simplest is to use the sonic transit time observed in nearby shales divided by 100
> (i.e., Cp = Δt_sh / 100)."*

A more accurate route it describes: compare a recorded transit time against a porosity known from
formation factor in a clean water-bearing sand, and let the ratio define `Cp`.

**So `Cp` is now a cited parameter with a cited range and two cited estimators.** It is not a
default: RHG80's own point is that `Cp` is *calibrated per well*, which is why it says the practice
*"detract[s] from the use of the time-average equation"* and is *"somewhat difficult to incorporate
into automatic computer programs."*

---

## 3. The RHG80 transform — three segments, cited breakpoints

RHG80 states plainly that **no single algorithm covers the range**:

> *"We could not, unfortunately, find a single algorithm which accurately described our transit
> time-to-porosity transform over the entire porosity range. The response curve was, therefore,
> divided into three segments."*

The breakpoints are **0.37** and **0.47** — cited constants, not fitted by us.

### φ < 37 %

Two algorithms, either adequate. The fluid-general form:

`V₁ = (1 − φ)² · Vma + φ · Vf`

RHG80's note on the pair: the first (a density form in `ρ` and `ρma`) applies *"when the saturating
fluid in the zone investigated by the sonic log is water"*; the second *"can be used regardless of
the nature of the saturating fluid; of course, the proper value for fluid velocity is required."*

### φ > 47 %

The suspension form, density-weighted on squared transit time:

`ρ / Δt² = φ · ρf / Δtf² + (1 − φ) · ρma / Δtma²`

RHG80 explains why the physics changes: above ~50 % porosity *"the suspended solid particles tend
to float within the fluid"*, so compressibilities add — `c = φ·cf + (1−φ)·cma`. Below ~35 % *"the
rock matrix also becomes continuous"* and the network is more parallel, `K = φ·Kf + (1−φ)·Kma`.
The intermediate band is where *"the rock matrix lattice … breaks down into individual solid
particles suspended in the fluid."*

**That is the physical reason for the three segments**, and it is the reason a single fitted bridge
across them is a curve fit rather than a model.

### 37 % < φ < 47 %

Linear interpolation between the two:

`Δt = [(0.47 − φ)/0.1] · Δt₁ + [(φ − 0.37)/0.1] · Δt₂`

And a simpler variant which RHG80 notes *"eliminates the need to ever actually calculate Δt₂"*, by
substituting `Δtf` for `Δt₂`.

---

## 4. The `Betters:` line, published rather than inferred

`CONTRACT.md` §2.2 requires an independently derived capability to name the incumbent limitation it
removes. **RHG80 states it in its own words:**

> *"This is the region in which present sonic interpretation would require a 'lack of compaction'
> correction. Use of the new transform eliminates the need for the 'lack of compaction' correction
> factor. Using the proposed transform, sonic transit time yields porosity directly."*

So the `Betters:` line for **MIN-C2-1** is sourced, not asserted: *the published three-segment
transform removes the per-well `Cp` calibration step that the time-average equation requires above
~35 % porosity.*

This matters for the Tier-C classification. `13_mineral-solver.md` §7.5 identified IP's
Wyllie ↔ Hunt-Raymer `Cp` bridge as a **proprietary four-term fit with no patent claim (draft C-2)**
and named the lawful route as solving the full nonlinear response directly. **RHG80 is that route,
published in 1980.** There is no need to reconstruct, approximate, or calibrate against any vendor's
fitted bridge — and per §7.5 the fit and its coefficients must not be used as a calibration target
either.

---

## 5. Other cited values in RHG80, and one departure point

| Quantity | Value | Note |
|---|---|---|
| Limestone matrix velocity | 20,500 ft/s → `Δtma` = 49 µs/ft | stated as a unique matrix velocity |
| Dolomite matrix velocity | 22,750 ft/s → `Δtma` = 44 µs/ft | as above |
| Sandstone, 5–25 % porosity | transform reads slightly higher φ than time-average at `Vma` 18,000 ft/s; at 15 % it matches `Vma` 19,500 ft/s | RHG80's reading of why high matrix velocities were historically chosen |
| Sandstone, ~30 % porosity | transform ≈ time-average at `Vma` 18,000 ft/s | |
| **Departure** | **above 35 % porosity** | *"sonic transit time increases much more rapidly than porosity, and its response quickly departs from that predicted by the time-average equation"* |
| Above 50 % porosity | transit time nearly independent of porosity, close to the fluid value | |
| Gas-saturated sandstone | a separate transform; normally needed only above 30 % porosity, *"occasionally observed in rocks of lower porosity"* | |

**RHG80's own honesty about its status, which any requirement built on it must carry:**

> *"The transform is totally empirical, being based entirely on comparisons…"*

Empirical is not a defect — it is a fact about the provenance, and `CONTRACT.md` §2 wants that fact
recorded beside the number.

---

## 6. What this unblocks, and what it does not

**Unblocked.** `SP-004` / `ACQ-11` can close: MIN-C2-1 now has a published method, cited defaults, a
cited applicability band, a sourced `Betters:` line, and enough stated structure to write the
analytic, boundary, continuity and regression tests §7.5 asks for. The continuity test is
obvious and worth naming here: **the interpolation must join the two segment forms at exactly 0.37
and 0.47**, and a test should pin both joins.

**Not unblocked, and do not conflate them.** This says nothing about `SP-003` — Omovie Sonic
Saturation, draft **C-1** under US 12,242,011 B2. That remains a patent question for a lawyer, and
these three papers are not evidence about it.

**Still Jauhar's call:** whether SandiBumi implements the RHG80 route at all, under what name, and
whether the existing sonic-porosity path is amended or joined by a second module. This note supplies
the evidence; it does not choose.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
