# 17. Thin-bed and laminated analysis — requirements

> **Dossier** `docs/research_2026-08/cross_tool/thinbed-laminated.md` (1,880 lines), read in full
> including the discrepancy ledger, the OPEN register, the derivation register D-1…D-27 and the
> critique disposition. The `*_critique.md` companions were **not** read, per `CONTRACT.md` §4.3.
> **Supporting primary register** `docs/research_2026-07/ref_thin_bed_lrlc.md` (158 lines).
> **Evidence tiers held** T1 (shipped vendor manifest `lssa.info`, shipped module parameter
> registers, SandiBumi source), T2 (vendor technical references and manuals — Geolog LSSA 57-page
> technical reference, IP 2025 CHM, Techlog help), T3 (rasterized equation images read visually),
> T4 (primary literature and this machine's project records).
> **Author date** 2026-08-07.
> **Requirements** 66 · **P0** 4 (`SB-TBD-006`, `-007`, `-009`, `-066`) · **Parameters** 52 rows, of
> which **9** ship `ABSENT — ships with no default`, **2** are `WITHDRAWN` and **0** are
> `NON-ADOPTABLE` · **Acceptance tests** 66, of which 4 are labelled `CHARACTERIZATION` ·
> **Dossier items dispositioned** 327 of 327 (§8).
>
> **Standing note on parameter discipline.** Not one endpoint, exponent, cutoff, resistivity or
> shale-porosity value in this chapter was inferred, rounded or carried across from a neighbouring
> vendor. Where the three incumbents disagree and no adjudication is defensible, §5 reads
> `ABSENT — ships with no default` and carries the competing values with their sources. Two values
> SandiBumi ships **today** are withdrawn by this chapter for having no source at all
> (`SB-TBD-066`), which is the correct outcome under `CONTRACT.md` §2 and not a regression.

---

## 1. Scope and boundary

This chapter owns the analysis of reservoirs whose productive sand is **thinner than the measurement
that is trying to see it**, and the low-resistivity-low-contrast (LRLC) pay that results. Concretely
it owns nine things:

1. **Shale-distribution decomposition** — the Thomas-Stieber family in all its shipped
   parameterizations: the laminar/dispersed/structural split, the solution-space constraint
   behaviour, the branch selection, the endpoint picking interaction, and the sand-referenced
   porosity bookkeeping (`PHIT_SS`, `PHIE_SS`, `VOL_SS`) that follows from it.
2. **The resistivity-tensor route** — horizontal/vertical resistivity mixing (parallel and series),
   the two-component laminated solve for sand resistivity, the anisotropic-shale quadratic, root
   selection, and every validity condition that bounds them.
3. **Relative dip and apparent resistivity** — the Moran-Gianzero relation, the bedding-normal angle
   convention, the √(Rh·Rv) ceiling, and the multi-well dip-fit route used where no triaxial tool
   was run.
4. **The anisotropy validity threshold** — the machine-readable conditions under which a
   weak-anisotropy or isotropic-shale substitution is invalid. `04_CORE_REQUIREMENTS.md`
   `SB-CORE-003` names this chapter as owner; `SB-TBD-034` and §5 discharge it, and §7.2
   escalates the one part of it the corpus does not document.
5. **The Vlam reconciliation diagnostic** — laminar shale from the Thomas-Stieber route against
   laminar shale from the tensor route, and what their disagreement means.
6. **Sand-referenced net and pay bookkeeping** — laminar net sand and net pay summed on the sand
   fraction rather than on the bulk volume, and the sand-fraction permeability transform.
7. **LRLC recognition and routing** — the Worthington taxonomy screen, the Madjid-Worthington
   bed-thickness scenario router, and the dispatch from a recognized cause to a shipped method.
8. **Resolution enhancement and dispersion correction** — binary-lithology deconvolution, the
   VLSA interval Monte-Carlo, the clay-mineral (`Vcm`) correction.
9. **The thin-bed visual surfaces** — the Thomas-Stieber triangle plot and its interactive
   endpoint handles, the Klein / butterfly crossplot, the anisotropy track, the reconciliation track.

Everything in this chapter is a **volumetric or geometric correction that changes what a saturation
equation is handed**. It does not own the saturation equation itself. That is the first and most
important seam.

### Named seams

**`12_saturation.md` — saturation equations, `Rw`, and the Archie parameters.** That chapter owns
`Rw`, `m`, `n`, `a`, the Waxman-Smits `B(T,Rw)` and `Qv` formulations, and the LRLC
excess-conductivity models `sw_rtc` and `sw_imts` **as saturation equations**. This chapter owns
only *what porosity and what resistivity those equations are handed*: the requirement that a
laminated well dispatches them on `PHIT_SS` and `RSS` rather than on bulk `PHIT` and `RT`
(`SB-TBD-044`, `SB-TBD-046`), and the renormalization back to a bulk saturation for reporting
(`SB-TBD-063`). The wrong Fahrenheit-to-Celsius conversion recorded as `D-TB-07` in the dossier is a
defect inside the `B` coefficient and therefore sits on the far side of this seam; it is recorded in
§7.3 as a refusal SandiBumi inherits, and its requirement belongs to `12_saturation.md`. The
`SWE_MIN` clipping guard (`SB-TBD-048`) is claimed here because it is a *laminated-sand* output
guard on `lssa.info`, not a property of any saturation equation — but the value it clips is that
chapter's.

**`15_sat-height-rocktyping.md` — saturation-height and rock typing.** Bound-water and irreducible
saturation as a *function of height above free water*, capillary-pressure-derived saturation, and
any rock-type-conditioned parameterization belong there. This chapter's `SWIRR` appearances are
inputs it consumes, never values it defines. The textural-facies capability appears in the Tier-C
register (`CONTRACT.md` §2.2, C-2) and belongs to that chapter, not this one.

**`25_fluidsub-rockphysics.md` — anisotropic rock physics.** Where transverse isotropy is a *rock
physics model* — Backus averaging, Thomsen parameters, anisotropic elastic moduli, velocity
anisotropy and its fluid substitution — it belongs there. This chapter owns anisotropy only as an
**electrical** phenomenon acting on resistivity, and only where it is a thin-bed correction. The
seam is sharp: if the quantity being made anisotropic is a resistivity, it is here; if it is a
modulus or a velocity, it is there.

**`10_clay-volume.md` — the shale volume this chapter consumes.** `VSH` is an input to every method
here and is defined there. Two consequences. First, the GR-to-`VSH` transform ladder duplicated
verbatim at `ssc.rs:57-68` against the original in `modules.rs:490-560` is an `SB-CORE-007`
violation that chapter already carries at `10_clay-volume.md:697` as `PRESENT-DIVERGENT`; this
chapter does not re-allocate it. Second, the `GR_MA 10.0 / GR_SH 150.0` pair at `ssc.rs:95-96` is
withdrawn by that chapter at `10_clay-volume.md:1824`; this chapter's §5 does not restate those
rows. What this chapter *does* own is the **clay-mineral** correction `Vcm` (Madjid & Worthington),
because that is a thin-bed-specific correction to a clay volume, not a clay-volume method.

**`11_porosity.md` — the total porosity this chapter consumes.** `PHIT` is an input. The `sspw`
gas-conditioning weight still shipping at `ssc.rs:433` — superseded on 2026-07-29 by the RMS-midpoint
form at `ssc.rs:171-185` but never propagated to the twin — is owned there as `SB-POR-059` [P0] with
tests `SB-POR-T33`/`T34`, and quantified there as **4.72 p.u.** of porosity in gas
(φD 0.25, NPHI 0.10 → `ssc` 0.1903943, `sspw` 0.1431782; arithmetic re-checked here and confirmed
to seven figures). This chapter does not re-allocate it either, but it does carry one obligation
that follows from it: the LRLC route table must not silently dispatch a gas-bearing laminated
interval into a module carrying an open P0 porosity defect (`SB-TBD-003`).

**`14_cutoffs-summation-mc.md` — cutoffs, summation and Monte-Carlo machinery.** That chapter owns
the cutoff panes, the summation engine and the Monte-Carlo infrastructure. This chapter owns the
**sand-referenced variant** of summation only: that in a laminated mode the cutoffs apply to
sand-fraction curves, that a shale-volume cutoff is *not* applied at all in that mode, and that
bulk-referenced and sand-referenced cutoffs are never silently interchanged (`SB-TBD-051`,
`SB-TBD-053`). The VLSA interval Monte-Carlo (`SB-TBD-059`) is specified here because its sampling
unit is a *bed-thickness distribution*, not a curve; its execution machinery is that chapter's.

**`13_permeability.md` — permeability transforms.** The transform is theirs. This chapter owns only
the sand-referencing of it: that a laminated well computes permeability on the sand fraction and
converts back by `PERM_FM = PERM_SS·(1 − VSH_LAM)` (`SB-TBD-064`), and that the Timur coefficient
carries a unit type so the two published forms cannot be interchanged (`SB-TBD-055`).

**`19_multi-mineral.md` — simultaneous inversion.** Where a laminated system is solved as one
optimizer problem with the shale as a mineral, that is a multi-mineral formulation and belongs
there. This chapter owns the closed-form route. §2 finding **F-18** records why the closed form is
kept as the primary route in this domain and is not an oversight.

**`20_plots-crossplots.md` and `21_reporting-deliverables.md`.** Plot infrastructure and deliverable
layout are theirs. The four thin-bed plots specified here (`SB-TBD-021`, `SB-TBD-050`, `SB-TBD-054`,
`SB-TBD-060`) are specified as *content and constraint geometry*, not as rendering.

### What this chapter deliberately does not claim

It does not own NMR bound-fluid partitioning (`23_nmr.md`), dielectric measurement, image-log bed
counting (`24_image-logs.md`, which supplies bed-thickness statistics this chapter's scenario router
consumes), or borehole environmental correction of the input curves (`08_envcorr-logqc.md`). Each is
consumed, none is defined here.

---

## 2. What the incumbents do — the requirement-bearing findings

Thirty-five findings. Each carries its evidence tier, the tools compared, and the consequence of
getting it wrong in petrophysical units. Dossier findings that generate no obligation are accounted
for in §8 rather than restated here.

Two conventions used throughout. `RSS` is the resistivity of the sand fraction; `PHIT_SS` and
`PHIE_SS` are total and effective porosity **referenced to the sand fraction** rather than to the
bulk volume; `VSH_LAM_TS` and `VSH_LAM_TN` are laminar shale volume from the Thomas-Stieber and from
the tensor route respectively. All angles are **degrees from the bedding normal**: θ = 0 is a
vertical well through flat beds, θ = 90° puts the tool in the bedding plane.

### 2.1 The Thomas-Stieber family

**F-1 — "Thomas-Stieber" names three different methods, and the name does not disambiguate them.**
*T1+T2+T3 · Techlog vs IP vs Geolog.* Techlog implements the 1975 GR-index/zeta original, exposing
`ζ = Rb/(Rb − Ra)` as the user handle. IP implements a Juhász development parameterized on `Vcl` and
`Phie`. Geolog implements `Thomas-Stieber-Juhasz` on PHIT-vs-VSH, with the laminar-dispersed branch
(Eq 81–90) as its shipped default. **Consequence:** a curve labelled `VLAM` imported from one tool
and compared against `VLAM` from another is not comparing like with like, and the difference is not
a QC signal. Any SandiBumi import, alias table or cross-tool comparison must carry the
parameterization, not just the name. Generates `SB-TBD-018`, `SB-TBD-020`.

**F-2 — SandiBumi's shipped `thin_bed_ts` is Geolog Eq 86 exactly.** *T2 + source.* Re-derived here
rather than taken from the dossier: eliminating `f_disp` from `modules.rs:2474-2479` gives
`VLAM = [PHIT + VSH·(1 − PHI_SH) − PHI_SD_MAX] / (1 − PHI_SD_MAX)`, which is Geolog Eq 86 with
`PHIT_MAX ↔ PHI_SD_MAX` and `PHIT_SH ↔ PHI_SH`. **Consequence:** the algebra is right and does not
need replacing. What follows in F-3 to F-11 is everything built around it that is not.

**F-3 — Beyond the pore-filling limit the 1975 original and the Geolog development are different
lines, and SandiBumi ships both — one in the picker, one in the module.** *T4 (Thomas & Stieber 1975
Eq 22, via `docs/research_2026-07/ref_thin_bed_lrlc.md`) vs T2 (Geolog Eq 86) · both live in
SandiBumi.* In the 1975 construction the dispersed trend descends to a vertex at
`VSH = PHI_SD_MAX`, where shale exactly fills the pores, then **rises** to the shale point along
`PHIT = PHI_SH·VSH`. Geolog's Eq 86 extends the pore-filling line linearly and has no vertex.
`crossplotPanel.ts:301-322` draws the 1975 kinked limb; `modules.rs:2475` computes the Geolog line.
**Consequence, quantified at the shipped defaults** (`PHI_SD_MAX` 0.30, `PHI_SH` 0.15): on the
picker's own drawn limb at `VSH` = 0.60, `PHIT` = 0.090, the module returns `VLAM` = 0.4286 and
`VSAND` = 0.5714 where the drawn construction places all shale in the dispersed phase
(`VLAM` = 0, `VSAND` = 1.000). **That is a 42.9 p.u. divergence in net-to-gross between the picture
the interpreter is looking at and the number the module returns from the same two endpoints they
just dragged.** It is not a SandiBumi-vs-Geolog divergence — Geolog agrees with the module, and the
same arithmetic run through Eq 86 returns 0.4286 — it is a SandiBumi-vs-SandiBumi divergence, and it
is `SB-CORE-006` ("one name, one equation") inside one product. Generates `SB-TBD-006`.

**F-4 — `PHIE_SS ≡ PHIE/(1 − VSH_LAM)` is a free correctness oracle and nobody asserts it.**
*T2, Geolog Eq 122 (p.136-51) against Eq 89.* Two independently-printed vendor equations must agree
identically on the laminar-dispersed branch for arbitrary in-range inputs. **Consequence:** a
one-line property test catches the entire class of sand-referencing algebra errors — including F-5 —
at zero interpretive cost. Generates `SB-TBD-011`.

**F-5 — Geolog distinguishes `PHIT_SS` (Eq 88) from `PHIE_SS` (Eq 89), and SandiBumi ships the first
under the second's name.** *T2 + source.* `modules.rs:2485-2486` computes
`(PHIT − VLAM·PHI_SH)/VSAND` and labels the output `PHIE_LAM`, "Laminar-shale-corrected sand
porosity" (`modules.rs:2451`). That expression is Eq 88 — the sand **total** porosity. The effective
value subtracts the dispersed shale as well. **Consequence, quantified:** at `PHIT` = 0.16,
`VSH` = 0.40 and the shipped defaults, `PHIE_LAM` = 0.164000 while `PHIE_SS` = 0.140000 —
**2.40 p.u.** too high; at `PHIT` = 0.12, `VSH` = 0.60 the gap is **3.65 p.u.** Feed the mislabelled
value into Archie at a = 1, m = n = 2, `Rw` = 0.10 ohm·m, `Rt` = 5 ohm·m and Sw reads **0.862
instead of 1.010** — a wet interval presented as 14 saturation units of movable hydrocarbon.
Generates `SB-TBD-009` [P0].

**F-6 — Geolog constrains out-of-model data in the total-porosity direction only and flags it;
SandiBumi clamps the derived fraction and says nothing.** *T2 (p.136-38/40) vs source.* Geolog's
stated rationale is that shale-volume indicators are the more robust of the two, so `PHIT` is the
quantity moved onto the boundary, and a coded `TSFLG` records the constraint direction and the
amount moved. `modules.rs:2477` instead applies `limit(…, 0.0, 1.0)` to the derived dispersed
fraction, and `modules.rs:2486` applies a second, undocumented clamp to `[0, PHI_SD_MAX]` on the
porosity output. **Consequence:** a point outside the solution space returns an ordinary in-range
`VLAM` with no flag, no record of how far outside it was, and no way for a reviewer to find it
afterwards. This is the exact failure mode `SB-CORE-002` exists to prevent, and it is the
highest-value divergence in this chapter. Generates `SB-TBD-007` [P0], `SB-TBD-008`.

**F-7 — Geolog does not constrain the below-left region at all; it back-solves a diagnostic
instead.** *T2, p.136-39.* For points below-left of the dispersed-pore-filling boundary Geolog emits
`PMAXNU` — the hypothetical clean-sand porosity that would place the point on the boundary — and
`PORFIL = PHIT_MAX − PMAXNU`. **Consequence:** this converts "your endpoint pick is wrong" from a
guess into a number the analyst reads directly off the log. Neither IP nor Techlog nor SandiBumi has
an equivalent. Generates `SB-TBD-010`.

**F-8 — Geolog's branch selection is analyst-controlled; IP's is automatic on an unvalidated rule.**
*T1 (`lssa.info` l.152) + T2 (p.136-40 Table 10) vs T2/T3.* Geolog gates the laminar-structural
branch on `STCT`, shipped at 0 so the default path is laminar-dispersed only. IP switches branch
automatically per depth level. **Consequence:** an automatic per-level rule can flip the
shale-distribution model between adjacent samples inside one geological unit, producing a `VLAM`
curve whose model discontinuities look like geology. Generates `SB-TBD-014`, `SB-TBD-015`.

**F-9 — The three Geolog shale cutoffs have different *kinds* of action, and the vendor admits one of
them is cosmetic.** *T1 (`lssa.info` l.152-154) + T2 (p.136-40 Table 10).* `STCT` and `LMCT` position
constraint *lines*; `DPCT` alone sets values (`PHIE := 0`, `PHIT := VSH·PHIT_SH`). The vendor
describes the `LMCT`→`DPCT` segment as a *"cosmetic ramping down"* of total porosity in the
high-`VSH` region, with a stated physical reason: above ~80 % shale a slight `PHIT` change swings
laminar sand porosity wildly, and washout makes both `VSH` and `PHIT` suspect. **Consequence:** an
interpreter who cannot tell which of the three moved a number cannot defend the result, and a port
that adopts all three uniformly ships a cosmetic operation as physics. Generates `SB-TBD-016`.

**F-10 — The endpoint pick dominates every downstream number, and the interactive surface that sets
it records nothing.** *T2 + T4 + source.* Geolog p.136-11 makes depth-coloured points on the
crossplot the mechanism for discovering that zoning is needed. SandiBumi ships a draggable-handle
picker (`crossplotPanel.ts:2023-2147`) that writes `PHI_SD_MAX` and `PHI_SH` into the zone-parameter
store on mouse-up (`crossplotPanel.ts:2146-2147`). **Consequence:** the most consequential parameter
choice in the domain is made by dragging, persisted with no source string, no analyst, no date and
no "picked interactively on this well's crossplot" provenance — invisible to `SB-CORE-010`.
Generates `SB-TBD-012`, `SB-TBD-021`.

**F-11 — The picker and the module disagree about the admissible range of the same parameter, and
the disagreement surfaces one run later.** *Source only.* `modules.rs:2445-2446` declares
`PHI_SD_MAX` valid on [0.05, 0.45] and `PHI_SH` on [0.0, 0.45]. `crossplotPanel.ts:2119` clamps the
drag to a hard-coded [0, 0.5] for **both** handles, and `db.rs:6991-6995` (`set_zone_param`) persists
whatever it is given with no validation against any `ModuleSpec`.
**Verified consequence — and it is narrower than it looks, which matters.** The value is *not*
silently computed on: `workflow.rs:60-96` (`resolve_param_arrays`) is a deliberate choke point that
**rejects** any user-supplied or zone-supplied parameter outside its declared `ArgSpec` range,
naming the parameter, the offending value and the valid range, with the reasoning recorded in the
source comment — *"Out-of-spec parameter values are REJECTED here, not clamped… would hand back a
plausible-but-wrong answer"* — and locked by the test at `workflow.rs:1976`. So SandiBumi already
holds the correct house pattern, and this chapter's clamp findings (F-6) are a **local** violation of
a rule the rest of the product keeps. What remains is still a real defect and it is
`SB-CORE-007`-shaped: **one constant, two definition sites.** A handle dragged to 0.48 is accepted by
the plot, persisted, and then kills the *next* run with an error the interpreter has to trace back to
a drag they made on a different screen. This is the **third** `SB-CORE-007` instance in this
codebase, distinct from the two already owned by `10_clay-volume.md` and `11_porosity.md`, and it is
the only one where the second definition site is in the frontend. Generates `SB-TBD-013` [P1].

### 2.2 The resistivity tensor

**F-12 — The two mixing laws, and a printed left-hand side that is the wrong quantity.** *T2 + T4.*
Parallel: `CH = (1 − VLAM)·CSS + VLAM·CSH`. Series: `RV = (1 − VLAM)·RSS + VLAM·RSH_V`. Geolog Eq 92
prints `CV =` on a right-hand side that evaluates to a resistivity (dossier `D-TB-04`); Eq 93 and
Eq 96 then consume `CV` as a conductivity, so the intent is unambiguous and is verified numerically.
**Consequence:** an implementer transcribing Eq 92 literally inverts the series branch. Generates
`SB-TBD-022`.

**F-13 — The anisotropic-shale form appears nowhere in the corpus correct and in one piece.**
*T2/T3, `D-TB-02`.* Geolog p.136-43 Eq 98 and p.136-5 Eq 6 differ in **three** places — the sign
inside the radical's leading term, the placement of the ½ (on the first term only in one printing, a
factor-of-2 defect on the radical), and the `±`. Neither printing is correct as printed. The working
form is the ordinary quadratic `a·CSS² + b·CSS + c = 0` with `a = CV − CV_SH`,
`b = CV_SH·CH_SH − CV·CH`, `c = CV·CV_SH·(CH − CH_SH)`. **Consequence:** any transcription must be
labelled as repaired, and SandiBumi's canonical form must be the machine-readable quadratic rather
than a copied radical. Generates `SB-TBD-023`, `SB-TBD-024`.

**F-14 — A fixed sign in the closed form silently returns the wrong root, and it starts doing so well
before the guard anyone would think to write.** *Derived, `D-TB-06` — the dossier's most consequential
finding.* The `±` does not track a physical branch: it swaps roots when `CSSAP` crosses `CH_SH`, i.e.
at `RV_SH_flip = RV/(2 − RH_SH/RH)`. On the truth case (`RH` = 1.904762, `RV` = 11.0, `RH_SH` = 1.0)
that is **7.4576 ohm·m, not `RV` = 11**. A fixed-sign implementation agrees with the quadratic at
`RV_SH` = 7.40 (both give `RSS` = 14.455) and returns **0.518** at 7.46 — a **96 % collapse across a
0.06 ohm·m step in a picked parameter**, with no discontinuity in the true solution there at all. The
threshold is level-dependent through `RH_SH/RH`: below `RV` only in the hydrocarbon/fresh-water-sand
quadrant (`RH_SH < RH`), above `RV` in the water-sand quadrant (`RH < RH_SH < 2·RH`), and
non-existent for `RH_SH ≥ 2·RH`. **Consequence:** solving the quadratic and selecting by quadrant is
immune by construction; guarding a closed form on `RV_SH ≥ RV` leaves the whole 7.458–11 window
silent. Generates `SB-TBD-025`, `SB-TBD-026`, `SB-TBD-033`.

**F-15 — The impossible quadrant is cheap to detect on the inputs, and none of the three tools
detects it.** *T2 claim + derived correction.* `RH < RH_SH` **and** `RV > RV_SH` is non-physical: a
water-sand condition in the horizontal component with a pay-sand condition in the vertical. Geolog's
manual describes the symptom as *"negative laminar shale volume and infinitely large RSS"*; a grid
sweep of the entire quadrant reproduces neither together — the isotropic form returns `RSS` = −180
with `VSH_LAM_TN` = +1.0495, and the anisotropic quadratic returns `RSS` = 2.545 with `Vlam` = 18.59
alongside `RSS` = −25.15 with `Vlam` = 1.284. **Consequence:** the printed signature must **not** be
tested for, but the quadrant test on the *inputs* is a complete, one-line pre-flight validator that
returns no number at all. Generates `SB-TBD-027`.

**F-16 — The solve is genuinely singular at `RV_SH = RV`, and over-picking above it destroys `RSS`
without bound.** *T2 (p.136-45) + derived.* At equality `a = CV − CV_SH = 0` and the quadratic
degenerates. Above it, measured on one baseline (truth `RV_SH` = 2.0 ⇒ `RSS` = 20.000): 3.0 → −5.2 %,
4.0 → −10.4 %, 12.0 → −49.5 %, 15.0 → **−61.6 %**. Under-picking is bounded — 1.0 (−50 %) costs only
**+5.3 %**. **Consequence:** the error is one-sided and unbounded on the side an analyst is most
likely to drift toward, because a resistive shale reads "safer". Generates `SB-TBD-028`,
`SB-TBD-030`.

**F-17 — The horizontal shale pick, not the anisotropy ratio, is the dominant sensitivity — and it is
the one rule none of the three tools enforces in code.** *T2 qualitative + derived quantitative.* A
**+10 %** error in `RH_SH` returns `RSS` **+11.2 %** and net sand **−11.1 %** simultaneously. Getting
the anisotropy *ratio* wrong by 2× in either direction costs only **∓5.2/5.3 %** on `RSS` and
**∓0.3 %** on `Vlam`. Geolog's own guidance — *"RH_SH should usually be no more than approximately a
tenth of an ohmm less than RH"* (p.136-45) — is therefore the operationally binding constraint, and
it is documentation everywhere and code nowhere. `RH_SH/RH` also appears inside `RV_SH_flip`, so the
dominant sensitivity and the branch-flip failure are **the same parameter seen twice**. Generates
`SB-TBD-029`, `SB-TBD-034`.

**F-17b — On the parallel-only route the same pick decides pay outright.** *T2/T3 + derived,
`D-TB-01`.* At a = 1, m = n = 2, `Rw` = 0.10 ohm·m, `PHIT_SS` = 0.25, IP's own worked example gives
`Rsand` = 50.00 → **Sw 17.9 %** at `Rshale` = 1.0; 4.13 → **62.3 %** at 1.5; 2.83 → **75.2 %** at
2.0. **A ±1 ohm·m uncertainty on one analyst-picked number moves Sw from 18 % to 75 %.** IP's manual
prints 5.75 for the middle case where the correct value is **4.13** — its own example fails its own
arithmetic. This is not a synthetic regime: project records put real shale picks from low-contrast
clastic sections inside the same 1–2 ohm·m band, with DST-confirmed pay at `Rt` ≈ 1.8 ohm·m — that
is, the shale pick and the pay resistivity are the same size, which is the definition of the
ill-conditioned case. Generates `SB-TBD-035`
and test `SB-TBD-T40`.

**F-18 — A shipped vendor bound exists on the tensor sand resistivity whose *action* is documented
nowhere.** *T1 beats T2 — `D-TB-09`.* `lssa.info` l.157-158 ships `RT_SS_MAX` = 100 ohm·m
(valid 2:2000) and `RT_SS_MIN` = 0.2 ohm·m (valid 0.02:20) in the "Tensor Model" group,
`VISIBLE = TRUE` unconditionally. A full-text search of the 57-page technical reference returns
**0 hits** for `RT_SS`, "Maximum Sand Resistivity" and "Minimum Sand Resistivity".
**Consequence, two-sided:** (a) the *bound* is first-class T1 evidence and must be carried; (b) the
*action* is not evidence at all, and inferring it by running `lssa.exe` and watching the output is
precisely the reconstruction path `CONTRACT.md` §2.2 prohibits. A gas-bearing laminated sand
legitimately reads above 100 ohm·m, and the same manual states `RSS` legitimately exceeds `RV`.
SandiBumi flags with the computed value preserved and names Geolog as the source of the bound, so a
cross-tool mismatch is explained rather than mysterious. Generates `SB-TBD-031`, and `SB-TBD-032`
keeps IP's parallel-route 2000 ohm·m bound off this path — the two differ by 20×.

**F-19 — The parallel-only route has a pole, and IP hides it behind a silent clamp.** *T2/T3 +
derived.* `RtLam` diverges at `VLAM_CRIT = RSH/RT`; past the pole `RSS` computes **negative**. IP
limits `RtLam`/`RxoLam` to 2000 ohm·m. **Consequence:** the value written into the log at the pole is
2000 ohm·m — a number, in range, plotted, with nothing to indicate the solve failed. A negative `RSS`
and a saturated `RSS` are different diagnoses and must not collapse into one clamped value. Generates
`SB-TBD-035`, `SB-TBD-036`.

**F-20 — IP silently reduces an anisotropy ratio it cannot solve.** *T2, `PhiFlag = 15`.*
**Consequence:** the run proceeds on a parameter the user did not set and is not told about — the
purest form of the fail-silent pattern. Recorded as a refusal in §7.3; generates `SB-TBD-037`.

**F-21 — Techlog prints the Klein-plot horizontal-mixing equation with a multiplication where a
division belongs.** *T3 raster vs prose, `D-TB-05`.* The prose on
`petrophysics-low-resistivity-pay-awi-response-equations.html` reads
`1/R_H = F_SAND/R_SAND + F_SH·R_SH_H`; the shipped raster on the same page has `VLAM/RSH_H`. The
raster is correct. **Consequence:** a prose-only transcription inverts the shale term, and the error
is **invisible at `R_SH_H` = 1 ohm·m** — the shale resistivity typical of the fresh-water,
low-contrast clastic sections where the method matters most. Generates `SB-TBD-054`.

### 2.3 Relative dip

**F-22 — Only Techlog has a dip term; IP and Geolog assume the supplied pair is already in the
bedding frame, and neither says so at the point of use.** *T3 raster + T4.* The shipped form
`Rt = RH/√(cos²θ + (RH/RV)·sin²θ)` is exactly Moran-Gianzero. Limits: θ = 0 → `Rt` = `Rh`;
θ = 90° → `Rt` = **√(Rh·Rv)**, the anisotropy ceiling. **Consequence:** in a deviated well through
dipping beds, running IP or Geolog on raw `Rh`/`Rv` silently mixes an apparent resistivity into a
bedding-frame solve. A tool that defaults θ to zero without saying so cannot be told apart from one
that has no dip term at all. Generates `SB-TBD-038`, `SB-TBD-039`, `SB-TBD-040`.

**F-23 — The angle convention is bedding-normal under all four independent sources, and a 90° swap is
the easiest error in the domain to make.** *T3 + T4 (memory `method_thinbed_rhrv_routes.md`,
`ref_thin_bed_lrlc.md` Elhadidy entry, the Moran-Gianzero standard form, the Techlog raster).*
**Consequence:** passing 90 − θ returns a different, entirely plausible answer. Carrying the
convention in the parameter name costs nothing. Generates `SB-TBD-041`.

**F-24 — Below roughly 40° relative dip the `Rv` sensitivity collapses, which bounds the only route
available without a triaxial tool.** *T4, memory `method_thinbed_rhrv_routes.md`.* The
Elhadidy/Aldred multi-well dip-fit route (route A2) recovers `Rh`/`Rv` by fitting apparent
resistivity across wells of differing relative dip; it needs a well stock **spanning more than 40°**,
and a near-vertical stock cannot feed it at all. **Consequence:** this is the one *numeric*
anisotropy-recovery threshold the corpus documents, and it binds hardest exactly where this suite is
meant to earn its keep: thinly interbedded deltaic clastic sequences drilled with near-vertical wells
and logged without a triaxial tool. It must be a machine-checked
precondition on the module, not a caveat in a manual. Generates `SB-TBD-042`, `SB-TBD-043`, and is
the load-bearing half of the `SB-CORE-003` discharge in `SB-TBD-034`.

### 2.4 Sand-referenced saturation and bookkeeping

**F-25 — The point of the whole exercise is the dispatch: `PHIT_SS` and `RSS`, never bulk.** *T2 +
T4 deck.* Geolog's rule is no dispersed shale → Archie; dispersed shale → Waxman-Smits or Dual Water;
**always on the sand fraction**. **Consequence:** SandiBumi's `sw_rtc` and `sw_imts` today consume
bulk `PHIT` (`lrlc.rs:123`, `lrlc.rs:228` — default curve `PHIT` with a per-sample fallback to
`PHIT_SSPW`) with no sand-fraction path at all. A laminated well therefore runs the LRLC models on a
porosity that still contains the laminar shale, so the correction this chapter exists to make is not
applied to the two modules `05_STRATEGY.md` §18.3 names as the differentiator. Generates
`SB-TBD-044`, `SB-TBD-045`, `SB-TBD-046`.

**F-26 — IP blocks three saturation equations under the laminated model, not two.** *T2/T3,
`laminated_sands_workflow.htm` l.98 and `A_porosity_sw.md` §8 l.697-704.* Poupon, Poupon-Aguilera and
Poupon-Tixier each already carry their own `(1 − Vcl)` / `Vcl/Rcl` laminated-shale term.
**Consequence:** running any of them on top of a laminated Sw model corrects for lamination twice.
The block must key on **equation identity**, not on a string match — though here a `Poupon` prefix
happens to catch exactly the right three, a useful secondary check. Generates `SB-TBD-047`.

**F-27 — Laminated net and pay are summed on the sand fraction with no `Vsh` cutoff, and the vendor
warns in its own manual that this changes reserves.** *T2, p.136-9 + T1 (`lssa.info` l.191-195).*
*"In LSSA no Vsh cutoff is used… in a laminated shaly sand formation which is 10 m thick and contains
50% laminar shale and 50% porous HC bearing sand layers, the net pay would be 5 m."* Outputs
`NET_LAM_SS`, `NET_LAM_PAY`, `CUM_HC`, and `PERM_FM = PERM_SS·(1 − VSH_LAM)`. The scheme ships
**off** by default (`OPT_NET` False, l.191). **Consequence:** `PHIE_SS_CUT` = 0.08 v/v *looks* like a
conventional net-pay porosity cutoff and is not numerically comparable to one, because it applies to
a sand-referenced porosity. Silently interchanging the two changes a reserves number without changing
anything visible in the parameter panel. Generates `SB-TBD-051`, `SB-TBD-052`, `SB-TBD-053`.

**F-28 — Two vendors ship a minimum-Sw clip for the low-contrast case, and one publishes the value.**
*T1 (`lssa.info` l.180-181) + T3.* Geolog: `OPT_SWLIM` default False, `SWE_MIN` default 0.08 v/v.
Techlog ships the same guard with no published default, stating the purpose: *"When the resistivity
of the sand and the shale layers are about the same, you can compute unrealistic small values of
water saturation."* **Consequence:** the clip is a real and useful guard, but a clipped `SWE`
presented without its unclipped twin is exactly the degraded-result-as-clean failure `SB-CORE-002`
prohibits. Generates `SB-TBD-048`.

### 2.5 Recognition, resolution and reporting

**F-29 — The two Vlam estimates disagreeing is a *measurement of bedding continuity*, not an error,
and only one vendor says so.** *T2, p.136-7/8.* `Vlam_TS ≈ Vlam_TN` → laminae laterally extensive,
trust the answer. `Vlam_TS` high / `Vlam_TN` low → bedding **disturbed** (bioturbation, slumping):
the shale genuinely is grain- and porosity-replacing, but the laminae are no longer laterally
extensive over the tool's depth of investigation, and the prescribed response is to re-assign the
majority of shale to dispersed and switch to a dispersed Sw equation. `Vlam_TS` low / `Vlam_TN` high
→ *"an additional factor causing electrical anisotropy, such as thin highly resistive layers"*.
**Consequence:** this machine's own KB note (`lssa-geolog-laminated-shaly-sand-analysis.md`) carries
a **wrong-signed** inference for the third case — it guessed *more conductive* where the vendor
states *highly resistive*. A wrong-signed diagnostic is worse than an absent one. Generates
`SB-TBD-049`, `SB-TBD-050`, and a KB-correction escalation in §7.2.

**F-30 — LRLC has a published cause taxonomy and a published bed-thickness scenario framework, and no
tool ships either as a routing decision.** *T4, Worthington 2000 (four manifestations × six causes)
and Madjid & Worthington SPE 163071 (scenarios A–E at 60–10 / 10–3 / 3–1 / <1 cm / mixed).*
**Consequence:** the cause determines the method. Thin-bed lamination needs the tensor or T-S route;
microporosity and fine-grain conductivity need an excess-conductivity model such as the shipped
`sw_rtc`/`sw_imts`. Choosing the wrong route is not a small error — applying a laminar correction to
a fine-grained low-contrast sand removes pay that is really there. Generates `SB-TBD-001`,
`SB-TBD-002`, `SB-TBD-003`.

**F-31 — `Vcm` is a *systematic* over-estimation of clay content in thin beds, and it propagates
straight into under-estimated pay.** *T4, Madjid & Worthington Eq 8–10, with φ_tsh (11), φ_tsd (12),
φ_esd (13).* The calibration chain is core porosity → `ρ_bc` → a `Vcm`-vs-`Vsh` regression fitted on
**thick** beds in the same depositional system and exported to the thin beds. **Consequence:** the
bias is one-directional — too much clay, too little sand porosity, too little pay — the opposite
direction from what a conservative estimate is supposed to err in. Generates `SB-TBD-061`.

**F-32 — No closed-form implementation in any of the three tools propagates uncertainty, and one
vendor's own deck admits it.** *T4: the deck's summary is stamped "Uncertainty!!" and carries
`Rsand = 10 ± 1 ohm-m` with no stated derivation.* Given F-17b, a ±1 ohm·m band on a shale pick is
the difference between 18 % and 75 % Sw. **Consequence:** an interval-average Monte-Carlo over the
picked endpoints and the bed-thickness distribution is the only honest way to book a laminated
reserve, and Jauhar's own `thinbed_vlsa` engine (Passey HPT, validated at True/Conventional/VLSA =
2.41 / 1.41 / 2.40 ft) already exists to do it. Generates `SB-TBD-059`.

**F-33 — Validity conditions live in one vendor's shipped *manifest*, not in anyone's code — and that
is the corpus's most transferable finding.** *T1, `lssa.info` VALIDATION columns.* Every parameter in
the manifest carries its validation range as data. The failure mode this exposes is precise: **a port
that lifts the algorithm out of the technical reference and leaves the manifest behind inherits a
fail-silent version of a fail-loud tool.** **Consequence:** validity conditions must be
machine-readable data attached to the `ModuleSpec` and evaluated before the solve, not prose in a
`doc` string. `04_CORE_REQUIREMENTS.md` `SB-CORE-003` names this chapter owner of the anisotropy
instance; §4.5 and §5 discharge it. Generates `SB-TBD-034`.

**F-34 — Unit traps that change an answer by orders of magnitude, all documented, none enforced.**
*T2 + T4.* Four in this domain. The Timur coefficient is **8581** for fractional φ and Sw and
**0.136** only for percent (p.136-51) — a factor of 63,000 between them. `CEC` is meq/100 g while
`Qv` is meq/mL, and Techlog labels its `Qv` "(1/L)" against a `B Value` in "L·S/m" — internally
consistent in a per-litre convention, but whether that convention is offset from meq/mL by 1000×
turns on Techlog's *compiled* definition of `B` and is **not established**. Temperature enters the
Juhász `B` correlation in °C while one Geolog printing computes `1.8·(FMT − 32)` and labels the
result Celsius. Grain size is Krumbein PHI. **Consequence:** any `B`/`Qv` import must convert **as a
pair or not at all**. Generates `SB-TBD-055`, `SB-TBD-056`, `SB-TBD-057`.

**F-35 — Techlog emits two sand-fraction curves with no stated difference between them.** *T3,
`petrophysics-outputs-curves-thomasstieber.html` — `VSS_TS` "Thomas-Stieber sand fraction" and
`GAMMA_TS` "Sand fraction", 21 curves total; no read page distinguishes them.* **Consequence:**
mapping either onto a SandiBumi curve before the difference is known risks silently swapping a
modelled fraction for a raw one. Generates `SB-TBD-058` — a refusal to map, which is the correct
requirement here.

---

## 3. SandiBumi as-built

Written from the source, not from the dossier. Every line pointer below was re-opened and re-read
during authoring; where the dossier and the source disagreed, the source won and the difference is
noted.

### 3.1 Capability inventory

| # | Capability | Status | Evidence |
|---|---|---|---|
| 1 | Thomas-Stieber laminar/dispersed decomposition | `PRESENT-OK` (algebra) | `modules.rs:2457-2487`; ≡ Geolog Eq 86, re-derived |
| 2 | `PHIE_LAM` output identity | `PRESENT-DIVERGENT` | `modules.rs:2485-2486` computes Eq 88 (`PHIT_SS`), labelled as effective at `:2451` |
| 3 | Out-of-model constraint behaviour | `PRESENT-DIVERGENT` | `modules.rs:2477`, `:2486` — two silent clamps, no flag |
| 4 | Laminar-structural branch | `ABSENT` | `modules.rs:2442` — *"Structural shale is not modeled."* |
| 5 | `STCT` / `LMCT` / `DPCT` cutoffs | `ABSENT` | no analogue in `modules.rs:2432-2455` |
| 6 | `TSFLG` / constraint flags | `ABSENT` | `TSFLG` — 0 hits across `src-tauri/src/` and `src/` |
| 7 | `PMAXNU` / `PORFIL` diagnostics | `ABSENT` | 0 hits |
| 8 | T-S triangle plot | `PARTIAL` | `crossplotPanel.ts:288-323` — laminated + dispersed limbs and two handles only; no structural line, no cutoff lines, no depth colouring |
| 9 | Interactive endpoint picking | `PRESENT-DIVERGENT` | `crossplotPanel.ts:2045-2147` — writes with no provenance; drawn model ≠ computed model (F-3) |
| 10 | Endpoint range as one definition | `PRESENT-DIVERGENT` | `modules.rs:2445-2446` [0.05,0.45] vs `crossplotPanel.ts:2119` [0,0.5] |
| 11 | Endpoint defaults cited | `PRESENT-DIVERGENT` | `modules.rs:2445-2446` — `0.30` and `0.15`, neither traceable to any source |
| 12 | Resistivity-tensor solve (any form) | `ABSENT` | `RV_SH`, `RH_SH`, `RT_SS`, `anisotrop*` — **0 hits** |
| 13 | Root selection / quadrant classifier | `ABSENT` | 0 hits |
| 14 | Parallel-only sand resistivity | `ABSENT` | 0 hits |
| 15 | Relative-dip correction | `ABSENT` | `Moran`, `Gianzero`, `reldip`, `relative dip` — **0 hits** |
| 16 | `VSH_LAM_TS` / `VSH_LAM_TN` reconciliation | `ABSENT` | `VSH_LAM` — 0 hits |
| 17 | Laminar net sand / net pay summation | `ABSENT` | `NET_LAM` — 0 hits |
| 18 | Sand-referenced Sw dispatch | `ABSENT` | no `PHIT_SS` or `RSS` curve exists anywhere |
| 19 | `sw_rtc` / `sw_imts` | `PRESENT-DIVERGENT` for this domain | `lrlc.rs:73`, `:118`, `:179`, `:225` — bulk-porosity only |
| 20 | Klein / butterfly crossplot | `ABSENT` | `Klein`, `butterfly` — 0 hits |
| 21 | Anisotropy track | `ABSENT` | 0 hits |
| 22 | LRLC recognition screen | `ABSENT` | `Worthington`, `Madjid` — 0 hits |
| 23 | `Vcm` clay-mineral correction | `ABSENT` | 0 hits |
| 24 | Resolution enhancement (binary litho) | `ABSENT` | the three `Bateman` hits are Bateman-Konen salinity (`modules.rs:1937`, `:1966`, `multimin2.rs:547`) — unrelated |
| 25 | VLSA interval Monte-Carlo | `ABSENT` | `VLSA` 0 hits; the 39 `Passey` hits are `toc_passey`, the ΔlogR TOC module (`unconventional.rs:18`) — unrelated |
| 26 | Uncertainty on any thin-bed output | `ABSENT` | — |
| 27 | Test coverage of the above | `PRESENT-UNVERIFIED` | exactly one test, `modules.rs:4862` |

**Read this table as a whole before reading §4.** Of twenty-seven capabilities the domain requires,
**one** is present and correct, **six** are present and divergent, **one** is partial, and
**nineteen** are absent. The entire resistivity-tensor half of the domain — the half that makes
low-contrast pay tractable when a triaxial tool exists — has no implementation of any kind.

### 3.2 `thin_bed_ts` — the one shipped module

Registered at `modules.rs:371`, dispatched at `modules.rs:457`, specified at `modules.rs:2432-2455`,
executed at `modules.rs:2457-2496`.

Two parameters and two log inputs:

- `PHI_SD_MAX` — "Clean sand porosity (endpoint)", v/v, **default 0.30**, range [0.05, 0.45]
  (`modules.rs:2445`)
- `PHI_SH` — "Shale porosity (endpoint)", v/v, **default 0.15**, range [0.0, 0.45]
  (`modules.rs:2446`)
- `PHIT` (default curve `PHIT`) and `VSH` (default curve `VSH`), both required
  (`modules.rs:2447-2448`)

Four outputs: `VLAM`, `VDISP`, `VSAND`, `PHIE_LAM` (`modules.rs:2449-2452`).

**The algebra is correct.** `modules.rs:2474-2479` builds the laminated line
`lam_line = PHI_SD_MAX·(1 − VSH) + PHI_SH·VSH` and the dispersed line
`disp_line = PHI_SD_MAX − VSH·(1 − PHI_SH)`, interpolates the dispersed fraction between them, and
splits `VSH`. Eliminating `f_disp` by hand gives
`VLAM = [PHIT + VSH·(1 − PHI_SH) − PHI_SD_MAX] / (1 − PHI_SD_MAX)`, which is Geolog Eq 86 exactly.
Independently checked numerically against Eq 86 at four `(PHIT, VSH)` pairs; agreement to all
printed figures. **Status `PRESENT-OK` for the decomposition itself.**

**Three defects sit on top of it.**

**(a) `PHIE_LAM` is `PHIT_SS`.** `modules.rs:2485-2486` computes `(PHIT − VLAM·PHI_SH)/VSAND`. That
is Geolog Eq 88 — the sand-fraction **total** porosity. The declared meaning at `modules.rs:2451` is
"Laminar-shale-corrected sand porosity", and the module doc at `modules.rs:2441-2442` calls it
"the laminar-shale-corrected porosity of the net sand" — both read as effective. The true effective
value additionally removes the dispersed shale. Quantified in F-5: **2.40 p.u.** at
(0.16, 0.40) and **3.65 p.u.** at (0.12, 0.60), worth **14.8 saturation units** through Archie at
m = n = 2. **`PRESENT-DIVERGENT`.**

**(b) Two silent clamps.** `modules.rs:2477` wraps the derived dispersed fraction in
`limit(…, 0.0, 1.0)`; `modules.rs:2486` wraps the porosity output in `limit(…, 0.0, phi_sd)`. The
first is the construction the dossier's adoption spec explicitly prohibits spec-wide; the second is
**not named in the dossier at all** and was found by reading the source. Neither sets a flag, neither
records how far outside the model the point was, and both return values indistinguishable from an
ordinary in-range answer. There is also a third, benign clamp at `modules.rs:2473` (`VSH` to [0,1])
which is an input sanitization rather than a derived quantity and is not a defect.
**`PRESENT-DIVERGENT`.**

**(c) `SB-CORE-004` — neither endpoint default is cited.** `0.30` and `0.15` appear at
`modules.rs:2445-2446` with no source in the code, no source in the doc string, and no agreeing
number in the corpus. Geolog ships `PHIT_MAX` = 0.35 (`lssa.info`, valid 0:0.5) and ships
`PHIT_SH` with **no default at all**, deriving it on the default path from wet-shale endpoints that
are themselves undefaulted. IP's 0.08 is a demo value, not a default. So SandiBumi's 0.30 agrees
with nobody and its 0.15 agrees with nobody. **`PRESENT-DIVERGENT`.**

**Structural shale is explicitly out of scope,** stated in the module's own doc string at
`modules.rs:2442`: *"Structural shale is not modeled."* That is an honest `ABSENT`, correctly
declared, and it is the model's best-documented boundary.

### 3.3 The interactive Thomas-Stieber picker

`drawTsOverlay` at `crossplotPanel.ts:291-323`, wired at `crossplotPanel.ts:1263`, toggled by the
"T-S triangle" checkbox at `crossplotPanel.ts:1897`, with drag handles registered at
`crossplotPanel.ts:2045-2048` and the write-back at `crossplotPanel.ts:2144-2148`:

```ts
if (mode === "ts-sand") write("PHI_SD_MAX", opts.tsPhiSd, (v) => v.toFixed(3));
else write("PHI_SH", opts.tsPhiSh, (v) => v.toFixed(3));
```

This is a genuinely good piece of product — an analyst drags the two endpoints on the crossplot and
the zone parameters follow. It is also the source of three findings.

**The drawn model is not the computed model.** `crossplotPanel.ts:304-322` draws the dispersed trend
as two segments: `(0, φsd) → (vMin, φsd·φsh)` with `vMin = min(1, φsd)`, then
`(vMin, φsd·φsh) → (1, φsh)`. The second segment is the Thomas & Stieber 1975 original — shale beyond
the pore-filling limit displacing matrix — and the code comment at `crossplotPanel.ts:301-302` says
so. The module has no such segment; `modules.rs:2475` extends the first line linearly forever.
Quantified in F-3: **42.9 p.u. of net-to-gross** at `VSH` = 0.60 on the shipped defaults.
**`PRESENT-DIVERGENT`.**

**The pick carries no provenance.** `write()` persists a number. Nothing records that it came from a
drag, on which crossplot, of which well, by whom, on what date, or against which depth interval.
`SB-CORE-010`'s provenance chain therefore terminates at "someone set 0.283". **`ABSENT`.**

**The drag clamp is a second definition of the parameter's range.** `crossplotPanel.ts:2119` clamps
both handles to a hard-coded `[0, 0.5]`, against `[0.05, 0.45]` and `[0.0, 0.45]` in the spec.
Per F-11 the run *does* reject the resulting override at `workflow.rs:60-96` rather than computing on
it, so this fails loud — but it fails one run later and on a different screen.
**`PRESENT-DIVERGENT`.**

### 3.4 The parameter write path

`db.rs:6976-6997` (`set_zone_param`) is an `INSERT … ON CONFLICT DO UPDATE` with no reference to any
`ModuleSpec` and no range check. The enforcement lives downstream at `workflow.rs:60-96`, which
rejects out-of-spec **user- and zone-supplied** values (spec defaults are trusted and not
re-validated, per the comment at `workflow.rs:66-67`) and is locked by
`workflow.rs:1976 out_of_range_zone_param_is_rejected_not_clamped`. The source comment there states
the design intent in the same terms this chapter uses — *"Silently clamping a percent-entered
`SWT_IRR` of 25 down to 0.6 would hand back a plausible-but-wrong answer"*.

**This matters for how §4 should be read.** SandiBumi already holds the reject-don't-clamp discipline
as a product-wide rule with a test behind it. The clamps at `modules.rs:2477` and `:2486` are not a
missing policy — they are a **local violation of a policy the product already has**, inside its own
module body where the choke point cannot see them. That makes them cheap to fix and expensive to
leave.

### 3.5 The LRLC saturation modules

`lrlc.rs` is 2,129 lines and ships four things: `sw_rtc` (spec `lrlc.rs:73`, body `lrlc.rs:118`),
`sw_imts` (spec `lrlc.rs:179`, body `lrlc.rs:225`), the Juhász `B` helper (`lrlc.rs:64`) and two
calibration fitters (`run_rtc_fit` at `lrlc.rs:478`, `run_s_factor_fit` at `lrlc.rs:911`).

**Both consume bulk porosity.** `lrlc.rs:103` and `lrlc.rs:211` declare the `PHIT` input with default
curve `PHIT_SSC`; `lrlc.rs:123` and `lrlc.rs:228` resolve it as
`prefer(ctx.log("PHIT"), ctx.log("PHIT_SSPW"))`. There is no sand-fraction input and no
sand-fraction path. In a laminated well these two modules therefore run the excess-conductivity
correction on a porosity that still includes the laminar shale. **`PRESENT-DIVERGENT` for this
domain** — they are correct as bulk saturation models, which is `12_saturation.md`'s business; they
are simply not connected to the laminated route.

**A naming hazard follows.** The dossier's `_SS` suffix means *sand fraction*. SandiBumi's existing
`_SSC` suffix (`ssc.rs:118-119` — `PHIT_SSC`, `PHIE_SSC`) means *sand-silt-clay model output*, and
`_SSPW` means *SSPW model output*. `PHIT_SS` and `PHIT_SSC` differ by one character and mean
different things. Introducing the sand-fraction curves without settling this is an `SB-CORE-006`
accident waiting to happen. Generates `SB-TBD-062`.

**Both carry candidly-labelled uncited constants** — `sw_rtc`'s doc string records its calibration as
*"one study's calibration … from one field"*, and `sw_imts`'s `S_FACTOR_GW` is described in its own
source as *"a property of the rock and of the clay curves it is paired with … not a value measured anywhere"*. Those are `12_saturation.md`'s
parameters and are not restated in §5 here; they are noted because §7's claim assessment depends on
them.

### 3.6 Test coverage

One test exists for the whole domain: `modules.rs:4862 thin_bed_ts_pure_laminated_and_dispersed`. It
asserts that a point placed on the laminated line returns `VLAM == VSH`, and a point placed on the
dispersed line returns `VDISP == VSH`. Both are the two exact corner cases where the clamps at
`modules.rs:2477` and `:2486` cannot fire and where the picker/module divergence of F-3 is zero. The
test is correct and passes; it is also, by construction, blind to every defect in §3.2 and §3.3.
**`PRESENT-UNVERIFIED`** for the module as a whole.

### 3.7 The two `SB-CORE-007` instances that are not this chapter's

Stated here so §4 is not read as ignoring them.

`ssc.rs:57-68` holds a verbatim second copy of the eight-transform GR→`VSH` ladder whose original is
`modules.rs:490-560`, with no test asserting the copies agree. Owned by `10_clay-volume.md:697`.

`ssc.rs:430-434` still runs the superseded gas-conditioning weight
`phid = √(φD² − 1.6·|φD² − NPHI²|/2)` where the corrected RMS-midpoint form at `ssc.rs:171-185`
(recorded as fixed on 2026-07-29 in the file header at `ssc.rs:22`) was never propagated to the
`sspw` twin. Verified numerically here: at φD 0.25 / NPHI 0.10, `ssc` returns 0.1903943 and `sspw`
returns 0.1431782 — **4.72 p.u.**, `sspw` biased low in gas. Owned by `11_porosity.md` as
`SB-POR-059` [P0].

Both are inputs to this domain and neither is re-allocated here. The consequence this chapter does
carry is `SB-TBD-003`: an LRLC route table must not dispatch a gas-bearing interval into a module
with an open P0 porosity defect without saying so.

---

## 4. Requirements

Sixty-six requirements, four of them P0. Normative verbs are RFC 2119 per `CONTRACT.md` §1.4.
Requirements marked with a `Betters:` line are independently-derived capabilities under
`CONTRACT.md` §2.2 and are cross-listed in §7.4.

### 4.1 LRLC recognition and routing

#### SB-TBD-001 — Ship the LRLC recognition screen as a decision, not a chapter of prose   [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a `thinbed_screen` module that classifies a low-contrast
interval against the Worthington (2000) taxonomy — four manifestations × six causes — from the
curves already available, and emits the classification as a coded curve with a confidence.

**Rationale.** F-30 (T4). The cause determines the method; no incumbent ships the taxonomy as a
routing decision. Applying a laminar correction to a fine-grained low-contrast sand removes pay
that is really there, and applying an excess-conductivity model to a laminated sand leaves pay on
the table. The classification is the cheapest high-value thing in this chapter because it needs no
`Rv`.

**As-built.** `ABSENT` — `Worthington` returns 0 hits across `src-tauri/src/` and `src/`.

**Verified by.** SB-TBD-T01, SB-TBD-T02

#### SB-TBD-002 — Route by bed thickness against tool resolution   [P2] [status: ABSENT]

**Requirement.** The screen MUST carry the Madjid & Worthington (SPE 163071) scenario framework
A–E, keyed on bed thickness relative to the resolution of the resistivity measurement, and MUST
name the bed-thickness source (image log, core, outcrop analogue, assumed) on the output.

**Rationale.** F-30 (T4). Scenario A (60–10 cm) and scenario D (<1 cm) require different methods,
and the difference is not visible in any log curve — it comes from an external measurement. A
scenario assignment whose thickness source is unrecorded is an assumption presented as a
classification.

**As-built.** `ABSENT` — `Madjid` returns 0 hits.

**Verified by.** SB-TBD-T02

#### SB-TBD-003 — The cause→method route table is data, and it discloses open defects on a route   [P2] [status: ABSENT]

**Requirement.** The mapping from a recognized LRLC cause to the module that treats it MUST be
machine-readable data on the screen module, not documentation. Where a route's target module carries
an open P0 defect recorded in another chapter, the screen MUST surface that on the route rather than
dispatch silently.

**Rationale.** F-30, and §3.7. The fine-grain/microporosity causes route to the SSC/SSPW porosity
suite, and `11_porosity.md` `SB-POR-059` [P0] records that the `sspw` twin returns porosity 4.72 p.u.
low in gas (`ssc.rs:433`). Dispatching a gas-bearing laminated interval down that route without
saying so is `SB-CORE-002` at the workflow level rather than the curve level.

**As-built.** `ABSENT` — no screen exists, therefore no route table exists.

**Verified by.** SB-TBD-T03

#### SB-TBD-004 — Declare and enforce the two-component limit   [P1] [status: ABSENT]

**Requirement.** Every module in the tensor and Thomas-Stieber suite MUST declare that it models a
**two-component** system (sand + shale) and MUST flag intervals where a third high-resistivity
component (coal, tight streak, carbonate cement, volcanic clast) is indicated by the available
curves.

**Rationale.** Dossier §2.6, all three tools. Both mixing laws and the whole T-S construction assume
two components. A coal lamina inside the interval loads the series resistivity as if it were
resistive sand, inflating `RSS` and deflating `VSH_LAM_TN`, with no symptom in either output.

**As-built.** `ABSENT` — `modules.rs:2437-2442` declares the shale-distribution assumption but not
the component count.

**Verified by.** SB-TBD-T04

#### SB-TBD-005 — Declare the excluded lithologies   [P2] [status: ABSENT]

**Requirement.** The suite MUST declare in machine-readable form that it is specified for clastic
laminated sand-shale systems and MUST warn when run on an interval flagged as carbonate or
cemented sandstone.

**Rationale.** Dossier §2.6. The declaration is free and the misuse is plausible — a low-contrast
carbonate is a real diagnosis and reaches for the same menu.

**As-built.** `ABSENT`.

**Verified by.** SB-TBD-T04

### 4.2 Thomas-Stieber

#### SB-TBD-006 — One name, one equation: the picker and the module MUST implement the same construction   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** The Thomas-Stieber construction drawn on the crossplot and the Thomas-Stieber
algebra executed by the module MUST be the same construction. SandiBumi MUST select one
parameterization, name it on both surfaces, and — where a second parameterization is offered — draw
and compute it as a separate, separately-named method.

**Rationale.** F-3 (T4 vs T2, both live in the product). `crossplotPanel.ts:301-322` draws the
Thomas & Stieber 1975 kinked dispersed limb; `modules.rs:2475` computes Geolog's linearly-extended
Eq 86 line. At the shipped defaults, on the picker's own drawn limb at `VSH` = 0.60 / `PHIT` = 0.090,
the module returns `VLAM` = 0.4286 and `VSAND` = 0.5714 against a drawn construction that places all
shale in the dispersed phase — **42.9 p.u. of net-to-gross** between the picture and the number, from
the same two endpoints the analyst just dragged. This is `SB-CORE-006` inside one product, and it is
P0 because a buyer evaluating this domain will drag those handles in the first ten minutes.

**As-built.** `PRESENT-DIVERGENT` — `crossplotPanel.ts:301-322` against `modules.rs:2475`.

**Verified by.** SB-TBD-T05, SB-TBD-T06

#### SB-TBD-007 — Never clamp a derived volume fraction or a derived sand porosity   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** No module in this suite may apply a clamp to a derived volume fraction, a derived
sand porosity, or any other quantity computed from the inputs. Out-of-model data is handled by
`SB-TBD-008`, not by clipping the answer into range.

**Rationale.** F-6, `SB-CORE-002`, and the dossier's own spec-wide prohibition. `modules.rs:2477`
clamps the derived dispersed fraction to [0,1]; `modules.rs:2486` clamps the porosity output to
[0, `PHI_SD_MAX`] — the second is not recorded in the dossier and was found by reading the source.
A clamped value is indistinguishable from a computed one at every downstream consumer. SandiBumi
already holds the correct discipline elsewhere: `workflow.rs:60-96` rejects rather than clamps, with
the reasoning in the source comment and a test at `workflow.rs:1976`. These two lines are a local
violation of the product's own rule.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2477`, `modules.rs:2486`. The input sanitization at
`modules.rs:2473` (`VSH` to [0,1]) is not in scope: it constrains an input, not a result.

**Verified by.** SB-TBD-T07, SB-TBD-T08

#### SB-TBD-008 — Constrain in the total-porosity direction, and record the shift   [P1] [status: ABSENT]

**Requirement.** For a point outside the Thomas-Stieber solution space the module MUST move `PHIT`
onto the boundary — never `VSH` — recompute, and emit a coded flag naming the constraint direction
together with the **signed amount `PHIT` was moved**, in porosity units.

**Rationale.** F-6 (T2, p.136-38/40). Geolog's stated rationale is that shale-volume indicators are
the more robust of the two inputs. The amount moved is the diagnostic: a 0.002 v/v nudge is noise, a
0.06 v/v shove means the endpoints are wrong, and only the second is worth an analyst's afternoon.
No incumbent publishes the shift; Geolog publishes only the flag.

**Betters:** Geolog emits `TSFLG` as a bare code (p.136-40 Table 10); the magnitude of the
constraint is not an output of any of the three tools. Emitting the signed shift converts a boolean
"this was constrained" into a quantity an interpreter can threshold and log-plot.

**As-built.** `ABSENT` — `TSFLG` and every equivalent return 0 hits.

**Verified by.** SB-TBD-T07, SB-TBD-T09

#### SB-TBD-009 — Retire `PHIE_LAM`; emit `PHIT_SS` and `PHIE_SS` as distinct curves   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** The module MUST emit the sand-fraction **total** porosity and the sand-fraction
**effective** porosity as two separately-named curves, and MUST NOT emit a curve whose name says
effective and whose value is total. `PHIE_LAM` MUST be withdrawn rather than redefined.

**Rationale.** F-5 (T2 + source). `modules.rs:2485-2486` computes Geolog Eq 88 and labels it
"Laminar-shale-corrected sand porosity" at `modules.rs:2451`. Quantified: **2.40 p.u.** too high at
(`PHIT` 0.16, `VSH` 0.40) and **3.65 p.u.** at (0.12, 0.60) on the shipped defaults; through Archie
at a = 1, m = n = 2, `Rw` = 0.10 ohm·m, `Rt` = 5 ohm·m that is Sw **0.862 instead of 1.010** — a wet
interval reported as 14 saturation units of movable hydrocarbon. `SB-CORE-006`. Withdrawal rather
than redefinition is required because a curve of that name already exists in delivered LAS files,
and silently changing what it means is worse than removing it.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2451`, `modules.rs:2485-2486`.

**Verified by.** SB-TBD-T10, SB-TBD-T11

#### SB-TBD-010 — Back-solve the below-left diagnostic instead of constraining it   [P2] [status: ABSENT]

**Requirement.** For points below-left of the dispersed-pore-filling boundary the module MUST NOT
constrain. It MUST emit the back-solved hypothetical clean-sand porosity and its difference from the
picked endpoint as analyst diagnostics.

**Rationale.** F-7 (T2, p.136-39). This converts "your endpoint pick is wrong" from a judgement into
a number read directly off the log. Neither IP nor Techlog has an equivalent, and SandiBumi's current
behaviour is to clamp the point into range and say nothing.

**As-built.** `ABSENT` — `PMAXNU` / `PORFIL` return 0 hits.

**Verified by.** SB-TBD-T12

#### SB-TBD-011 — Assert the `PHIE_SS ≡ PHIE/(1 − VSH_LAM)` identity as a property test   [P1] [status: ABSENT]

**Requirement.** The suite MUST carry a property test asserting that the sand-fraction effective
porosity computed from Geolog Eq 89 equals `PHIE/(1 − VSH_LAM)` from Eq 122 to machine precision, for
arbitrary in-range `(PHIT, VSH, PHIT_MAX, PHIT_SH)`.

**Rationale.** F-4 (T2, two independently-printed vendor equations). This is a free oracle over the
whole sand-referencing algebra and would have caught `SB-TBD-009` on the day it was written.

**As-built.** `ABSENT` — the only test in the domain is `modules.rs:4862`.

**Verified by.** SB-TBD-T11

#### SB-TBD-012 — Every interactive endpoint pick carries its provenance   [P1] [status: ABSENT]

**Requirement.** A parameter written by a crossplot drag MUST persist, alongside the value, the fact
that it was picked interactively, the plot and axes it was picked on, the well and depth interval
displayed, and the date. That provenance MUST survive into the deliverable under `SB-CORE-010`.

**Rationale.** F-10, F-17b. The endpoint pick is the dominant sensitivity in the domain — a ±1 ohm·m
band on a shale pick spans Sw 18 % to 75 %, and the porosity endpoints drive `VLAM` directly.
`crossplotPanel.ts:2146-2147` writes a bare number. The chain that makes SandiBumi's provenance claim
true terminates at "someone set 0.283".

**Betters:** none of the three incumbents records the interactive origin of a picked endpoint;
IP's crossplot handle additionally *back-solves* a second parameter and moves the plotted data
underneath the analyst (dossier `OPEN-TB-1`, ip2025 `I` §2.5 item 6). Recording the pick makes the
most consequential number in the domain auditable, which is the one thing no incumbent offers.

**As-built.** `ABSENT` — `crossplotPanel.ts:2144-2148`; `db.rs:6976-6997` stores value only.

**Verified by.** SB-TBD-T13

#### SB-TBD-013 — One admissible range per parameter, in one place   [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The admissible range of a module parameter MUST be defined once, on the
`ModuleSpec`, and every surface that can set it — typed dialog, zone dialog, DB inspector, crossplot
drag handle — MUST read that range rather than carry its own.

**Rationale.** F-11, `SB-CORE-007`. `modules.rs:2445-2446` declares [0.05, 0.45] and [0.0, 0.45];
`crossplotPanel.ts:2119` hard-codes [0, 0.5] for both handles. The run correctly rejects the result
at `workflow.rs:60-96`, so nothing wrong is computed — but the analyst learns about it one run later,
on a different screen, from an error naming a parameter they set by dragging. This is the third
`SB-CORE-007` instance in the codebase and the only one whose second definition site is in the
frontend.

**As-built.** `PRESENT-DIVERGENT` — `crossplotPanel.ts:2119` against `modules.rs:2445-2446`.

**Verified by.** SB-TBD-T14

#### SB-TBD-014 — Ship the laminar-structural branch, analyst-selected   [P1] [status: ABSENT]

**Requirement.** The Thomas-Stieber module MUST offer the laminar-structural branch in addition to
laminar-dispersed, under an explicit analyst-set control, and MUST return the same sand-fraction
total porosity from both branches for the same `VSH_LAM`.

**Rationale.** F-8 (T1 `lssa.info` l.152 + T2 p.136-40). Structural shale is a real distribution mode
and `modules.rs:2442` currently declares it out of scope. Geolog Eq 83 ≡ Eq 88 is the cross-branch
identity that keeps the two consistent.

**As-built.** `ABSENT` — `modules.rs:2442`: *"Structural shale is not modeled."*

**Verified by.** SB-TBD-T15, SB-TBD-T16

#### SB-TBD-015 — No automatic per-level branch switching   [P1] [status: ABSENT]

**Requirement.** The shale-distribution branch MUST NOT change between adjacent depth samples on an
automatic rule. Branch selection is a zone-level analyst decision.

**Rationale.** F-8. IP switches automatically per level; a model discontinuity inside one geological
unit is indistinguishable from geology on a log plot and propagates into every downstream curve.

**As-built.** `ABSENT` — only one branch ships, so the defect is not present; the requirement
constrains the implementation of `SB-TBD-014`.

**Verified by.** SB-TBD-T15

#### SB-TBD-016 — Shale cutoffs carry their action class, and the cosmetic one says so   [P2] [status: ABSENT]

**Requirement.** Where the suite implements the structural/laminar/dispersed shale cutoffs, each MUST
declare whether it positions a constraint line or sets values, and the segment the vendor documents
as cosmetic MUST be labelled display-quality rather than physics in both the UI and the deliverable.

**Rationale.** F-9 (T1 l.152-154 + T2 p.136-40 Table 10). Only the dispersed cutoff has a hard
numerical action (`PHIE := 0`, `PHIT := VSH·PHIT_SH`); the vendor itself calls the laminar→dispersed
segment a *"cosmetic ramping down"*. Adopting all three uniformly ships a cosmetic operation as
physics.

**Betters:** Geolog's own manual carries the distinction in a table an interpreter has to find;
carrying it as a per-cutoff attribute puts it in front of the person setting the number.

**As-built.** `ABSENT`.

**Verified by.** SB-TBD-T17

#### SB-TBD-017 — Keep the two laminar-shale estimates separately named   [P1] [status: ABSENT]

**Requirement.** Laminar shale from the Thomas-Stieber route and from the tensor route MUST be
emitted under distinct names, with a third curve carrying whichever the analyst selected as
authoritative, and the selection MUST be recorded.

**Rationale.** F-29, `SB-CORE-006`. The whole reconciliation diagnostic (`SB-TBD-049`) requires the
two to be separable; collapsing them into one `VLAM` destroys the single most transferable QC in the
domain.

**As-built.** `ABSENT` — `VSH_LAM` returns 0 hits; `modules.rs:2449` emits an undifferentiated
`VLAM`.

**Verified by.** SB-TBD-T18

#### SB-TBD-018 — An imported Thomas-Stieber curve carries its parameterization, not just its name   [P1] [status: ABSENT]

**Requirement.** Importing a `VLAM`-family curve from a vendor project MUST record which
Thomas-Stieber parameterization produced it, and a cross-tool comparison MUST refuse to difference
two curves produced by different parameterizations without declaring it.

**Rationale.** F-1 (T1+T2+T3). Techlog's zeta original, IP's Juhász `Vcl`/`Phie` development and
Geolog's PHIT-vs-VSH form share a name and are three different methods. A difference between them is
not a QC signal, and presenting it as one manufactures a discrepancy.

**As-built.** `ABSENT`.

**Verified by.** SB-TBD-T19

#### SB-TBD-019 — One flag convention across the suite, counted in the run summary   [P1] [status: ABSENT]

**Requirement.** Every guard in this chapter MUST emit into a single documented flag scheme, and the
count of flagged levels per condition MUST appear in the run summary. Levels suppressed as
unusable MUST be counted and reported, never silently skipped.

**Rationale.** F-33, `SB-CORE-002`, Geolog p.136-10. The failure this prevents is a run that
completes with 40 % of its levels flagged and a summary that looks identical to a clean one. The
codebase already has a per-module flag idiom (`condition.rs:203`, `:212`, `:390-391`), so this is
adoption of an existing house pattern rather than new infrastructure.

**As-built.** `ABSENT` for this domain — `run_summary` returns 0 hits anywhere;
`condition.rs:1174` counts flags inside a test only.

**Verified by.** SB-TBD-T20

#### SB-TBD-020 — Accept both spellings of Stieber on import   [P2] [status: PARTIAL]

**Requirement.** The mnemonic and method matcher MUST map `Steiber` → `Stieber` on import.

**Rationale.** Dossier §4.10 (`D-03`, ip2025 `DISCREPANCIES.md` l.53, status NOTED). Both spellings
appear in vendor documentation and in field-delivered curve names.

**As-built.** `PARTIAL` — `modules.rs:490-560` ships `STIEBER1/2/3` spelled correctly;
`Steiber` returns 0 hits, so the alias does not exist. Note this is the GR-ladder naming owned by
`10_clay-volume.md`; the requirement here is the **import alias**, which is this chapter's because
it also governs `VLAM`-family curve names.

**Verified by.** SB-TBD-T21

#### SB-TBD-021 — Ship the complete Thomas-Stieber triangle   [P1] [status: PARTIAL]

**Requirement.** The Thomas-Stieber crossplot MUST draw every boundary of the construction in use —
laminated line, dispersed limb, structural line where that branch is active — plus the active cutoff
lines, and MUST colour the plotted points by depth.

**Rationale.** F-10 (T2 p.136-11): depth colouring is the mechanism by which an interpreter discovers
that one set of endpoints cannot serve the whole interval and zoning is needed. Without it the
crossplot shows a cloud that a single pick always appears to fit.

**As-built.** `PARTIAL` — `crossplotPanel.ts:293-323` draws the laminated line and a two-segment
dispersed limb with two handles. No structural line, no cutoff lines, no depth colouring.

**Verified by.** SB-TBD-T22

#### SB-TBD-066 — Withdraw the two uncited endpoint defaults   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `PHI_SD_MAX` = 0.30 and `PHI_SH` = 0.15 MUST be withdrawn as shipped defaults.
The clean-sand endpoint MUST ship the cited Geolog value or ship absent; the shale porosity MUST be
**derived** from picked wet- and dry-shale endpoints on the default path, with direct entry
available only where porosity is supplied externally.

**Rationale.** `SB-CORE-004` [P0], and dossier `OPEN-TB-1`. Neither number at `modules.rs:2445-2446`
traces to any source in the corpus. Geolog ships `PHIT_MAX` = 0.35 and ships `PHIT_SH` with **no
default**, deriving it from wet-shale `RHO_SH`/`NPHI_SH` (themselves undefaulted) on the shipped
porosity path; IP's 0.08 is a demo value, not a default. SandiBumi's 0.30 and 0.15 therefore agree
with nobody, and per F-10 they drive `VLAM` directly. The right answer is Geolog's **method**, not
Geolog's number.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2445-2446`.

**Verified by.** SB-TBD-T23, SB-TBD-T24

### 4.3 The resistivity tensor

Every requirement in this group is `ABSENT`: `RV_SH`, `RH_SH`, `RT_SS` and `anisotrop*` return zero
hits across the entire codebase. The as-built line is not repeated on each block.

#### SB-TBD-022 — Implement the series branch as a resistivity mix   [P1] [status: ABSENT]

**Requirement.** The vertical (series) mixing law MUST be implemented as
`RV = (1 − VLAM)·RSS + VLAM·RSH_V`, and any transcription of the vendor equation that prints a
conductivity on the left-hand side MUST be labelled as repaired in the source.

**Rationale.** F-12, `D-TB-04` (T2). Geolog Eq 92 prints `CV =` over a right-hand side that evaluates
to a resistivity; Eq 93 and Eq 96 then consume `CV` as a conductivity. A literal transcription
inverts the branch.

**Verified by.** SB-TBD-T25

#### SB-TBD-023 — The canonical anisotropic form is the quadratic, and every repair is labelled   [P1] [status: ABSENT]

**Requirement.** The anisotropic-shale solve MUST be implemented as the quadratic
`a·CSS² + b·CSS + c = 0` with `a = CV − CV_SH`, `b = CV_SH·CH_SH − CV·CH`,
`c = CV·CV_SH·(CH − CH_SH)`, using a sign-stable root form. It MUST NOT be implemented by
transcribing a printed radical, and the source MUST record that no vendor printing of this equation
in the corpus is correct as printed.

**Rationale.** F-13, `D-TB-02` (T2/T3). The two Geolog printings differ in three places — the sign
inside the radical's leading term, the placement of the ½, and the `±` — and neither is correct.
`CONTRACT.md` §5.1: the vendors' own defects are the opportunity.

**Betters:** the implemented form exists nowhere in the vendor documentation in one piece; a
machine-readable quadratic with a labelled derivation is checkable by a reviewer, which the radical
is not.

**Verified by.** SB-TBD-T26, SB-TBD-T27

#### SB-TBD-024 — Retain and report both roots   [P1] [status: ABSENT]

**Requirement.** The solve MUST compute both roots of the quadratic and MUST make the non-selected
root available as a diagnostic output.

**Rationale.** F-14. The two-root structure is the mechanism behind the domain's most consequential
failure; discarding one root inside the solve makes the failure undiagnosable after the fact. IP
exposes the ambiguity through a magic-number interface, Geolog hides it entirely.

**Verified by.** SB-TBD-T28

#### SB-TBD-025 — Select the root by a quadrant classifier, and record the branch   [P1] [status: ABSENT]

**Requirement.** Root selection MUST be made by classifying the input quadrant from
`(RH, RV, RH_SH, RV_SH)`, MUST emit the chosen branch as a coded curve, and MUST fall back to a
per-zone analyst override — flagging the level as analyst-resolved — where the classification is
ambiguous. The override enum MUST be semantic (`AUTO` / sand-resistive / sand-conductive) and MUST
NOT reproduce a magic-number interface.

**Rationale.** F-14, `D-TB-06`, `OPEN-TB-8`. Geolog's `lssa.info` ships **no root-selection
parameter**, and there is no `.lls` source — the vendor's mechanism is inside a compiled binary and
is not documented. Inferring it by running the binary and observing outputs is precisely the
reconstruction path `CONTRACT.md` §2.2 prohibits. The quadrant test is derived independently from
the algebra and the printed physical constraints.

**Betters:** the incumbent's branch choice is *undocumented and uninspectable* (`OPEN-TB-8`,
`lssa.exe` compiled, no `.lls`, no manifest parameter) and, per `D-TB-06`, a fixed sign silently
returns the wrong root above `RV_SH_flip`. A quadrant classifier that records its decision as a curve
is both correct where the fixed sign is not, and auditable where the vendor's is not.
**Independent derivation, class C-3 — see §7.4.**

**Verified by.** SB-TBD-T29, SB-TBD-T30

#### SB-TBD-026 — Ship `RV_SH_flip` as an interoperability advisory   [P2] [status: ABSENT]

**Requirement.** The tensor module MUST compute `RV_SH_flip = RV/(2 − RH_SH/RH)` **per depth level**
and MUST flag levels where `RV_SH ≥ RV_SH_flip` with an advisory stating that a third-party
closed-form answer may be on the wrong root. The threshold MUST NOT be hard-coded as a constant or
as `RV`.

**Rationale.** F-14, `D-TB-06`. On the truth case the flip is at **7.4576 ohm·m against `RV` = 11** —
a fixed-sign implementation agrees with the quadratic at `RV_SH` = 7.40 (both 14.455) and returns
0.518 at 7.46. SandiBumi's quadratic-plus-quadrant implementation is immune by construction, so this
is not a correctness guard for SandiBumi; it is the curve that explains why a Geolog run, a
spreadsheet or an inherited script disagrees. The threshold is level-dependent through `RH_SH/RH` and
sits below `RV` only when `RH_SH < RH`.

**Betters:** no incumbent computes this threshold at all, and the vendor whose closed form exhibits
the flip does not document the window. Shipping the advisory turns an unexplained cross-tool
disagreement into a named, located one.

**Verified by.** SB-TBD-T29, SB-TBD-T31

#### SB-TBD-027 — Reject the impossible quadrant on the inputs, before the solve   [P1] [status: ABSENT]

**Requirement.** Where `RH < RH_SH` **and** `RV > RV_SH`, the module MUST reject the level and return
no number. The rejection MUST key on the **input quadrant test**, not on any signature of the output.

**Rationale.** F-15 (T2 claim + derived correction). The quadrant is non-physical, and every solve in
it returns out-of-range output. The vendor's printed symptom — *"negative laminar shale volume and
infinitely large RSS"* — does **not** reproduce: a grid sweep returns `RSS` = −180 with
`VSH_LAM_TN` = +1.0495 on the isotropic form, and `RSS` = 2.545 / `Vlam` = 18.59 alongside
`RSS` = −25.15 / `Vlam` = 1.284 on the anisotropic quadratic, with `|RSS| > 10⁴` never occurring.
Testing for the printed signature would ship a guard that does not fire.

**Betters:** the input quadrant test is a complete, one-line pre-flight validator that **none of the
three tools runs**, and it is strictly cheaper than the output-inspection heuristic the vendor
describes.

**Verified by.** SB-TBD-T32

#### SB-TBD-028 — Hard-flag the `RV_SH ≥ RV` singularity   [P1] [status: ABSENT]

**Requirement.** Where `RV_SH ≥ RV` the module MUST flag and return no number — it MUST NOT divide by
zero, return an infinity, or return a large finite value. The margin `RV/RV_SH` MUST be emitted as a
QC curve.

**Rationale.** F-16 (T2 p.136-45 + derived). At equality `a = CV − CV_SH = 0` and the quadratic is
genuinely singular. Above it the error is one-sided and unbounded: on one baseline (truth
`RV_SH` = 2.0 ⇒ `RSS` = 20.000), 3.0 costs −5.2 %, 12.0 costs −49.5 % and 15.0 costs **−61.6 %**,
while under-picking to 1.0 costs only +5.3 %.

**Verified by.** SB-TBD-T33, SB-TBD-T34

#### SB-TBD-029 — Enforce the horizontal shale-pick proximity guidance in code   [P1] [status: ABSENT]

**Requirement.** Where `RH_SH` is not within the vendor-documented proximity of `RH`, the module MUST
warn, naming the guidance and its source. Where `RH_SH` is within 0.1 ohm·m of `RH` the module MUST
additionally warn that `VSH_LAM_TN → 1` and `RSS` becomes undefined.

**Rationale.** F-17 (T2 p.136-45 + derived). A **+10 %** error in `RH_SH` returns `RSS` **+11.2 %**
and net sand **−11.1 %** simultaneously — an order of magnitude more damaging than a 2× error in the
anisotropy ratio (∓5.2/5.3 %). Geolog's guidance is the operationally binding constraint in the
domain and *"it is the one rule none of the three tools enforces in code"*.

**Betters:** all three tools carry this as documentation only. Enforcing it at the point of the pick
converts the domain's dominant sensitivity from a thing an experienced analyst remembers into a thing
the software checks.

**Verified by.** SB-TBD-T35

#### SB-TBD-030 — Validate `RV_SH ≥ RH_SH` at parameter-entry time   [P1] [status: ABSENT]

**Requirement.** `RV_SH < RH_SH` MUST be rejected when the parameter is set, not when the solve runs.

**Rationale.** F-16 (T2 p.136-45, stated as a physical constraint: vertical shale resistivity cannot
be less than horizontal). Catching it at entry means the analyst learns at the moment of the mistake.

**Verified by.** SB-TBD-T36

#### SB-TBD-031 — Flag the tensor sand-resistivity bounds; never clamp to them   [P1] [status: ABSENT]

**Requirement.** Where the solved sand resistivity falls outside the vendor-shipped tensor bounds the
module MUST preserve the computed value and raise an advisory naming Geolog as the source of the
bound and stating that a Geolog run may disagree in that range. It MUST NOT write the bound as the
value.

**Rationale.** F-18, `D-TB-09` (T1 `lssa.info` l.157-158; **0 hits** in the 57-page T2 reference).
The bound is first-class T1 evidence; the *action* is documented nowhere and inferring it by
observing the binary's behaviour is the prohibited derivation path. A gas-bearing laminated sand
legitimately exceeds 100 ohm·m, and the same manual states `RSS` legitimately exceeds `RV` — so a
clamp would be wrong on the physics as well as on the provenance.

**Betters:** the incumbent ships a bound whose behaviour its own 57-page technical reference never
mentions. Preserving the value and explaining the divergence converts an unexplainable cross-tool
mismatch into a documented one, which is the opposite of what the incumbent offers.
**Independent derivation, class C-3 — see §7.4.**

**Verified by.** SB-TBD-T37

#### SB-TBD-032 — The parallel-route saturation bound MUST NOT be applied to the tensor route   [P1] [status: ABSENT]

**Requirement.** IP's 2000 ohm·m `RtLam`/`RxoLam` bound is scoped to the parallel route and MUST NOT
be used as the tensor-route threshold.

**Rationale.** F-18. The two vendor bounds differ by **20×** (2000 against 100 ohm·m) and belong to
different solves. Using the looser one on the tensor path silently disables the guard.

**Verified by.** SB-TBD-T37

#### SB-TBD-033 — Never a hard-coded sign   [P1] [status: ABSENT]

**Requirement.** No implementation in this suite may select a quadratic branch by a fixed `+` or `−`.

**Rationale.** F-14, `D-TB-06`. This is stated as a separate requirement from `SB-TBD-025` because it
is independently testable and independently violable: an implementation can carry a quadrant
classifier and still short-circuit to a fixed sign in a fast path.

**Verified by.** SB-TBD-T29

#### SB-TBD-034 — Ship the anisotropy validity conditions as machine-readable data on the module spec   [P1] [status: ABSENT]

**Requirement.** Every validity condition bounding the tensor and dip solves MUST be attached to the
`ModuleSpec` as **data** — condition expression, severity (reject / flag / warn), message, and
source citation — and MUST be evaluated by the framework before the solve runs. It MUST NOT be
expressed as prose in a `doc` string, as an `if` inside a module body, or as documentation.

The conditions this chapter owns, each with its source:

| Condition | Severity | Source |
|---|---|---|
| `RV_SH < RH_SH` | reject at entry | Geolog `lssa_tech_reference.pdf` p.136-45 |
| `RV_SH ≥ RV` | reject at solve | Geolog p.136-45 + derived (`a = CV − CV_SH = 0`) |
| `RH < RH_SH` **and** `RV > RV_SH` | reject at solve | Geolog p.136-45 + derived §3.4 |
| `RH_SH ≥ 2·RH` | no positive `RV_SH_flip` exists | derived `D-TB-06` |
| `RV_SH ≥ RV/(2 − RH_SH/RH)` | advisory | derived `D-TB-06` |
| `\|RH_SH − RH\| < 0.1` ohm·m | warn | Geolog p.136-45 |
| `RSS` outside the shipped tensor bounds | advisory | Geolog `lssa.info` l.157-158 |
| `VLAM/VLAM_CRIT > 0.9` on the parallel route | warn | dossier §5.5 (reporting threshold, no vendor antecedent) |
| relative-dip span across the well stock `< 40°` | reject the multi-well fit route | memory `method_thinbed_rhrv_routes.md` |
| dip source absent | reject | dossier §5.5 |

**Rationale.** `04_CORE_REQUIREMENTS.md` `SB-CORE-003` names this chapter as owner of the anisotropy
threshold and requires it as machine-readable data. F-33 gives the mechanism and the reason: the
corpus's most transferable finding is that one vendor's fail-loud reputation lives in its manifest
`VALIDATION` columns rather than in its code, so **a port that lifts the algorithm and leaves the
manifest behind inherits a fail-silent version of a fail-loud tool.** F-24 supplies the one *numeric*
anisotropy-recovery threshold the corpus documents (40° of dip span).

**Scope limit, stated rather than filled.** `SB-CORE-003` speaks of "the documented anisotropy
threshold beyond which a weak-anisotropy substitution is invalid". The corpus documents the
**sensitivity** of the isotropic-shale substitution — ∓5.2 %/+5.3 % on `RSS` and ∓0.3 % on `Vlam` at
a 2× ratio error (F-17) — but **no numeric threshold on the shale-anisotropy ratio itself** beyond
which the substitution is declared invalid. That threshold is escalated in §7.2 and is **not
invented here**. What ships is the table above, which is complete for every condition the corpus does
document.

**Betters:** the vendor whose manifest carries validation ranges evaluates them only as UI entry
bounds; the derived conditions (`RV_SH_flip`, the quadrant test, the 40° span) are enforced by no
tool at all. Evaluating the whole set as pre-flight data, with each condition carrying its citation
into the flag message, is unmatched in the corpus.

**Verified by.** SB-TBD-T29, SB-TBD-T32, SB-TBD-T33, SB-TBD-T36, SB-TBD-T38

#### SB-TBD-035 — Detect the parallel-route pole; never clamp through it   [P1] [status: ABSENT]

**Requirement.** The parallel-only module MUST compute `VLAM_CRIT = RSH/RT`, MUST emit the margin as
a curve, MUST warn as the margin closes, and MUST NOT write a saturation bound as a value.

**Rationale.** F-19, F-17b. `RtLam` diverges at the pole; IP limits the output to 2000 ohm·m, so the
number written into the log at a failed solve is an in-range plottable value with nothing to mark it.
The stakes are set by F-17b: on IP's own example a ±1 ohm·m shale-pick uncertainty moves Sw from
17.9 % to 75.2 %, and IP's manual prints 5.75 for a case whose correct value is **4.13**.

**Verified by.** SB-TBD-T39, SB-TBD-T40

#### SB-TBD-036 — A negative sand resistivity is a distinct diagnosis   [P1] [status: ABSENT]

**Requirement.** Past the pole, where the parallel solve returns a negative sand resistivity, the
module MUST raise that as its own named condition — never as a large positive number and never as the
same flag as a saturated result.

**Rationale.** F-19. "The solve went past its pole" and "the sand is very resistive" have opposite
interpretations and opposite responses; collapsing them into one clamped 2000 ohm·m destroys both.

**Verified by.** SB-TBD-T39

#### SB-TBD-037 — Never silently reduce an anisotropy ratio   [P1] [status: ABSENT]

**Requirement.** Where the anisotropy ratio admits no root the module MUST flag and stop for that
level. It MUST NOT alter the ratio and continue.

**Rationale.** F-20 (T2, IP `PhiFlag = 15`). The run otherwise proceeds on a parameter the user did
not set and is not told about — the purest form of the fail-silent pattern this product is sold
against. Recorded as a refusal in §7.3.

**Verified by.** SB-TBD-T41

### 4.4 Relative dip

All `ABSENT`: `Moran`, `Gianzero`, `reldip` and `relative dip` return zero hits across the codebase.

#### SB-TBD-038 — Implement the Moran-Gianzero relation, forward and inverse   [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide `Rt = RH/√(cos²θ + (RH/RV)·sin²θ)` and its inverse, as a
module that converts between the measured apparent resistivity and the bedding-frame pair.

**Rationale.** F-22 (T3 raster + T4). Only Techlog has a dip term; IP and Geolog assume the supplied
`Rv`/`Rh` pair is already in the bedding frame and neither says so at the point of use. In a deviated
well through dipping beds — the normal case in a prograding deltaic sequence — running either on raw
curves mixes an apparent resistivity into a bedding-frame solve.

**Verified by.** SB-TBD-T42, SB-TBD-T43

#### SB-TBD-039 — Refuse to default the relative dip to zero   [P1] [status: ABSENT]

**Requirement.** The tensor route MUST require either a dip source or an explicit analyst declaration
that the inputs are already in the bedding frame. It MUST NOT silently assume θ = 0.

**Rationale.** F-22, `SB-CORE-002`. A tool that assumes zero dip without saying so is
indistinguishable from a tool that has no dip term. The declaration costs one control and converts an
invisible assumption into a recorded one.

**Betters:** two of the three incumbents make this assumption with no mechanism to state or override
it; the third has the term but does not force the choice. Requiring the declaration is the
difference between a documented model and an accident.

**Verified by.** SB-TBD-T44

#### SB-TBD-040 — Assert the √(Rh·Rv) ceiling   [P1] [status: ABSENT]

**Requirement.** The implementation MUST satisfy `Rt → Rh` as θ → 0 and `Rt → √(Rh·Rv)` as θ → 90°,
and MUST carry that as an executed test.

**Rationale.** F-22 (T3 + T4 memory `method_thinbed_rhrv_routes.md`, independently recorded). The
ceiling is the closed-form check that catches a transposed `Rh`/`Rv` or a degrees/radians error in
one assertion.

**Verified by.** SB-TBD-T42

#### SB-TBD-041 — Carry the bedding-normal convention in the parameter name   [P1] [status: ABSENT]

**Requirement.** The relative-dip parameter MUST be named so that the reference frame is unambiguous
at the point of entry, and the suite MUST carry a test asserting that passing 90 − θ returns a
different answer.

**Rationale.** F-23 (T3 + three independent T4 sources agreeing on bedding-normal). A 90° convention
swap returns a different, entirely plausible number and is the easiest error in the domain to make.
`SB-CORE-013`: state the reference convention.

**Verified by.** SB-TBD-T43

#### SB-TBD-042 — Ship the multi-well dip-fit route for wells without a triaxial tool   [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide the Elhadidy-family multi-well route that recovers `Rh` and
`Rv` by fitting apparent resistivity across wells of differing relative dip in one depositional
system, with the fit residual reported per well.

**Rationale.** F-24 (T4). Triaxial induction is rare in the deltaic clastic sections this suite
targets; without this route the entire tensor half of the chapter is unreachable on conventional
data, and half the suite's value goes unrealised on exactly the reservoirs it is specified for.

**Betters:** no incumbent offers any route to `Rv` without a triaxial measurement — the tensor
modules in all three tools take `Rh` and `Rv` as given inputs. Supplying a data route to those
inputs, with a stated precondition and a residual, is capability none of them has.

**Verified by.** SB-TBD-T45

#### SB-TBD-043 — Enforce the 40° dip-span precondition on the multi-well route   [P1] [status: ABSENT]

**Requirement.** The multi-well dip-fit route MUST reject a well stock whose relative-dip span is
below the documented threshold, and MUST report the actual span. The threshold MUST be declared as
machine-readable data under `SB-TBD-034`.

**Rationale.** F-24 (T4, memory `method_thinbed_rhrv_routes.md`: *Rv sensitivity collapses below
~40° relative dip*). This is the one **numeric** anisotropy-recovery threshold the corpus documents,
and it binds exactly where the product is positioned — a near-vertical well stock cannot feed the
route at all, however many wells it contains. A fit run below it returns a converged, plausible `Rv` that is unconstrained by the
data.

**Verified by.** SB-TBD-T46

### 4.5 Sand-referenced saturation and bookkeeping

#### SB-TBD-044 — Dispatch saturation on the sand fraction, never on bulk   [P1] [status: ABSENT]

**Requirement.** With the laminated model active, every saturation model MUST be evaluated on
`PHIT_SS` and `RSS`. The dispatch rule MUST be: no dispersed shale → Archie; dispersed shale →
Waxman-Smits or Dual Water. The saturation equations themselves belong to `12_saturation.md`; this
requirement governs only what they are handed.

**Rationale.** F-25 (T2 + T4 deck). This is the entire point of the chapter. Running a saturation
model on bulk porosity in a laminated well leaves the laminar shale inside the pore system the
equation is solving for.

**As-built.** `ABSENT` — no `PHIT_SS` or `RSS` curve exists.

**Verified by.** SB-TBD-T47

#### SB-TBD-045 — Emit the sand-referenced curve set explicitly   [P1] [status: ABSENT]

**Requirement.** The suite MUST emit the sand-fraction volume, the sand-fraction total and effective
porosity, and the sand resistivity as named curves. The sand fraction MUST be an explicit output, not
left to a downstream consumer to compute as `1 − VLAM`.

**Rationale.** F-25, `SB-CORE-006`. Every downstream consumer that recomputes the sand fraction is a
place the convention can drift.

**As-built.** `ABSENT` for the porosity and resistivity curves. `PARTIAL` for the volume:
`modules.rs:2451` emits `VSAND` = `1 − VLAM` — correct in value, but named against the bulk model
rather than the sand-referenced one and not accompanied by the rest of the set.

**Verified by.** SB-TBD-T47, SB-TBD-T48

#### SB-TBD-046 — Offer `sw_rtc` and `sw_imts` on the sand fraction   [P2] [status: PRESENT-DIVERGENT]

**Requirement.** The two LRLC excess-conductivity models MUST be selectable as sand-referenced
saturation options within the laminated dispatch, consuming `PHIT_SS` and `RSS`.

**Rationale.** F-25. `05_STRATEGY.md` §18.3 positions these two as the shipped half of Axis 3. They
are the excess-conductivity complement to the laminar correction and no commercial tool has them —
but today they consume bulk porosity (`lrlc.rs:123`, `lrlc.rs:228`), so the two halves of the
strategy's own claim do not connect.

**Betters:** neither Techlog, IP nor Geolog ships an excess-conductivity LRLC model of this family at
all; connecting them to the sand fraction makes the combination — laminar correction *and*
excess-conductivity correction, dispatched by a recognized cause — a capability with no incumbent
equivalent.

**As-built.** `PRESENT-DIVERGENT` — `lrlc.rs:73`, `:118`, `:179`, `:225`; bulk-porosity path only.

**Verified by.** SB-TBD-T49

#### SB-TBD-047 — Block all three Poupon-family equations under the laminated model   [P1] [status: ABSENT]

**Requirement.** With the laminated model active, Poupon, Poupon-Aguilera and Poupon-Tixier MUST be
refused with the stated reason. The block MUST key on **equation identity**, not on a name match.

**Rationale.** F-26 (T2/T3, `laminated_sands_workflow.htm` l.98 and `A_porosity_sw.md` §8
l.697-704). Each already carries its own laminated-shale term, so running one on top of a laminated
Sw model double-corrects for lamination. IP names two of the three in one place and the third in
another, which is how a two-item rule gets written; blocking on identity catches all three.

**Betters:** IP states the prohibition in prose across two documentation pages and does not enforce
it in code. Enforcing it at dispatch, with the vendor's own stated reason surfaced, converts a
documented trap into an impossible action.

**Verified by.** SB-TBD-T50

#### SB-TBD-048 — The minimum-Sw guard emits its unclipped twin   [P1] [status: ABSENT]

**Requirement.** Where the minimum clean-sand `Swe` guard is enabled, the module MUST emit both the
clipped and the unclipped saturation, and MUST flag the levels where the clip fired.

**Rationale.** F-28 (T1 `lssa.info` l.180-181 + T3), `SB-CORE-002`. The guard is real and useful —
its documented purpose is the case where sand and shale resistivities are about the same, which is
this domain's normal case — but a clipped `SWE` presented alone is a degraded result presented as a
clean one.

**Betters:** both vendors that ship this guard write the clipped value and nothing else. Emitting the
twin makes the guard's effect measurable instead of invisible.

**Verified by.** SB-TBD-T51

#### SB-TBD-063 — Renormalize the sand-referenced saturation back to bulk for reporting   [P1] [status: ABSENT]

**Requirement.** Where a saturation is computed on the sand fraction, the suite MUST also emit the
bulk-referenced saturation obtained by renormalizing it through the laminar shale volume and the
shale's own saturation, and the renormalization MUST conserve bulk hydrocarbon volume.

**Rationale.** F-25 (T2, Geolog's dispatch). A sand-referenced `Sw` is the correct number for the
sand and the wrong number for a summation over the interval; reporting only one of the pair forces
every downstream consumer to guess which reference frame it is holding. Conservation of bulk
hydrocarbon volume is the closed-form check that the renormalization is right. The saturation
equations remain `12_saturation.md`'s.

**Verified by.** SB-TBD-T66

#### SB-TBD-049 — Ship the Vlam reconciliation classifier   [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST compare laminar shale from the Thomas-Stieber route against laminar
shale from the tensor route and emit a three-case classification as a coded curve: agreement,
`TS` high / `TN` low, `TS` low / `TN` high — each carrying the vendor-documented interpretation and
its prescribed response.

**Rationale.** F-29 (T2, p.136-7/8). All three tools converge on the comparison; only Geolog's
technical reference explains what disagreement **means**, and the meaning is not "one of them is
wrong" — it is a measurement of lateral bedding continuity over the tool's depth of investigation.
The prescribed response to the disturbed-bedding case is to re-assign shale to dispersed and switch
the Sw equation, which is an action, not a caveat.

**Betters:** IP states only that the two *should* agree; Geolog builds its tuning loop around the
comparison but leaves the interpretation in a manual. Shipping the classification as a coded curve
with its response attached puts the diagnosis in the log rather than in the analyst's memory.

**Verified by.** SB-TBD-T52

#### SB-TBD-050 — Ship the reconciliation track   [P2] [status: ABSENT]

**Requirement.** The deliverable MUST offer a track overlaying the two laminar-shale estimates with
the mismatch shaded and the classifier curve alongside.

**Rationale.** F-29. The disagreement is a depth-varying signal; a single summary statistic hides
exactly the intervals where it matters.

**Verified by.** SB-TBD-T52

#### SB-TBD-051 — Ship laminar net sand and net pay, summed on the sand fraction   [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a laminar summation mode that applies **no shale-volume
cutoff** and sums net sand and net pay on the sand-fraction curves, emitting laminar net sand,
laminar net pay and cumulative hydrocarbon.

**Rationale.** F-27 (T2 p.136-9 + T1 `lssa.info` l.191-195). This is the bookkeeping that makes a
laminated evaluation mean anything, and it is the one that changes reserves.

**Verified by.** SB-TBD-T53

#### SB-TBD-052 — The laminar summation mode is labelled, opt-in, and off by default   [P1] [status: ABSENT]

**Requirement.** The laminar summation MUST be a distinct, labelled mode requiring explicit opt-in,
MUST default to off, and MUST carry the vendor's own warning into the deliverable header.

**Rationale.** F-27. Geolog ships it off (`OPT_NET` False, l.191) and warns in its own manual that
*"This difference in technique can lead to major changes in results for a zone, and this must be
taken into account when using the results for reserves determination."* A summation mode that changes
reserves and is silently on is the single most expensive default this chapter could ship.

**Verified by.** SB-TBD-T53

#### SB-TBD-053 — Sand-fraction and bulk cutoffs are never interchangeable   [P1] [status: ABSENT]

**Requirement.** Cutoffs applied to sand-fraction curves MUST be typed distinctly from cutoffs
applied to bulk curves. A cutoff value MUST NOT be transferable between the two modes without an
explicit conversion, and the deliverable MUST state which mode each reported cutoff belongs to.

**Rationale.** F-27. The sand-fraction porosity cutoff *looks* like a conventional net-pay porosity
cutoff and is not numerically comparable to one. Carrying a bulk-mode number into a sand-referenced
summation changes a reserves figure with nothing visible in the parameter panel. Seam:
`14_cutoffs-summation-mc.md` owns the cutoff machinery; this requirement owns the typing.

**Verified by.** SB-TBD-T54

### 4.6 Plots, permeability and unit discipline

#### SB-TBD-054 — Ship the Klein / butterfly crossplot with the mixing overlays   [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a log-log `Rv`-vs-`Rh` crossplot with iso-sand-resistivity
and iso-net-to-gross overlays forward-modelled from the two mixing laws, the shale point, and the
isotropic `Rv = Rh` line. The overlay generator MUST use the **division** form of the horizontal
mixing law.

**Rationale.** F-21, `D-TB-05` (T3 raster over prose). IP ships it as a standalone butterfly
crossplot and Techlog as the modified Klein plot; both are computationally specified. The division
guard matters because Techlog's prose prints a multiplication, and the resulting error is **invisible
at a shale resistivity of 1 ohm·m** — the value typical of fresh-water low-contrast clastics, where
this plot is the whole point. A reference fixture already exists in Jauhar's own
`Klein plot trial.xlsx` prototype.

**Verified by.** SB-TBD-T55, SB-TBD-T56

#### SB-TBD-055 — The Timur coefficient is unit-typed   [P1] [status: ABSENT]

**Requirement.** Where the suite ships a Timur-family permeability transform, the coefficient MUST
carry a unit type binding it to fractional or percentage porosity and saturation, and the two forms
MUST NOT be interchangeable without conversion.

**Rationale.** F-34 (T2, p.136-51: *"8581 is the appropriate coefficient if the input is in
fractions"*, with 0.136 applying only to percent). The two differ by a factor of **63,000**.
`SB-CORE-013`.

**Verified by.** SB-TBD-T57

#### SB-TBD-056 — `Qv` and `B` are converted as a pair or not at all   [P1] [status: ABSENT]

**Requirement.** Any import of a `Qv` curve or a `B` coefficient from a vendor project MUST convert
both together under one declared convention, or refuse the import. A `Qv` converted without its
paired `B` MUST be rejected.

**Rationale.** F-34, `OPEN-TB-14` (T3 + T4). Techlog labels `Qv` "(1/L)" and `B Value` "L·S/m" —
internally consistent in a per-litre convention — while the literature provenance on this machine is
meq/mL. Whether the two conventions are offset by 1000× turns on Techlog's **compiled** definition of
`B` and is **not established**. Converting one without the other silently changes a saturation by
three orders of magnitude. The `B` correlation itself is `12_saturation.md`'s.

**Verified by.** SB-TBD-T58

#### SB-TBD-057 — Angle and temperature conventions are typed, not assumed   [P1] [status: ABSENT]

**Requirement.** Relative dip MUST be unit-typed as degrees-from-bedding-normal and temperature as
°C or °F explicitly, with conversion by tested functions. `CEC` (meq/100 g), `Qv` (meq/mL) and grain
size (Krumbein PHI) MUST likewise carry their units as types.

**Rationale.** F-34, `SB-CORE-013`. Every one of these has a documented cross-tool conflict behind
it, including a Geolog printing that computes `1.8·(FMT − 32)` and labels the result Celsius — which
if implemented as printed yields 302–482 where a Celsius formation temperature is expected. That
defect sits inside the `B` coefficient and is `12_saturation.md`'s to fix; the **typing** that makes
it impossible to ship silently is this chapter's.

**Verified by.** SB-TBD-T43, SB-TBD-T59

#### SB-TBD-058 — Refuse to map the ambiguous vendor sand-fraction curves   [P1] [status: ABSENT]

**Requirement.** The Techlog import MUST NOT map either of the two Thomas-Stieber sand-fraction
curves onto a SandiBumi curve until their difference is established. Both MUST import under their
vendor names, unmapped, with the ambiguity recorded on the import.

**Rationale.** F-35, `OPEN-TB-15` (T3, 21 output curves, no read page distinguishing the two).
Importing the wrong one silently swaps a modelled fraction for a raw one, and the swap is invisible
because both are sand fractions in the right range. A refusal to map is the correct engineering here;
guessing is not.

**Verified by.** SB-TBD-T60

#### SB-TBD-064 — Compute permeability on the sand fraction and convert back explicitly   [P2] [status: ABSENT]

**Requirement.** In the laminated mode, permeability MUST be computed on the sand fraction and
converted to a formation permeability by `PERM_FM = PERM_SS·(1 − VSH_LAM)`, with both curves emitted.

**Rationale.** F-27 (T2, p.136-9). The transform itself is `13_permeability.md`'s; the sand
referencing and the conversion are this chapter's, and emitting both makes the laminar reduction
visible rather than folded into one number.

**Verified by.** SB-TBD-T61

#### SB-TBD-060 — Ship the anisotropy track   [P2] [status: ABSENT]

**Requirement.** The deliverable MUST offer a track carrying the formation and shale anisotropy
ratios on a common scale, for setting shale anisotropy interactively against a pure-shale interval.

**Rationale.** F-17 (T2/T3; IP ships the scaling). Shale anisotropy is picked, not measured, and the
pick is made by looking at the ratio in a thick shale. Without the track the parameter is entered
blind.

**Verified by.** SB-TBD-T62

### 4.7 Uncertainty, resolution and naming

#### SB-TBD-059 — Propagate uncertainty through an interval Monte-Carlo   [P3] [status: ABSENT]

**Requirement.** SandiBumi MUST provide an interval-average hydrocarbon-pore-thickness estimate with
uncertainty, sampling the picked endpoints and the bed-thickness distribution, and MUST report the
result as a distribution rather than a single value.

**Rationale.** F-32 (T4). No closed-form implementation in any of the three tools propagates
uncertainty, and one vendor's own deck stamps its summary *"Uncertainty!!"* while carrying a ±1 ohm·m
band with no stated derivation. Per F-17b that band spans Sw 18 % to 75 %. Jauhar's existing
`thinbed_vlsa` engine already implements the Passey route and validates against the published
True/Conventional/VLSA figures of 2.41 / 1.41 / 2.40 ft.

**Betters:** all three incumbents return a single deterministic number in a domain whose dominant
input is an analyst's pick on a crossplot. Reporting a distribution is the difference between a
reserves number that can be defended and one that cannot.

**Verified by.** SB-TBD-T63

#### SB-TBD-061 — Ship the clay-mineral correction   [P3] [status: ABSENT]

**Requirement.** SandiBumi MUST provide the Madjid & Worthington `Vcm` correction for the systematic
over-estimation of clay content in thin beds, with its calibration fitted on thick beds in the same
depositional system and the fit exported to the thin-bed interval. The calibration source and the
donor interval MUST be recorded on the output.

**Rationale.** F-31 (T4, SPE 163071 Eq 8–10 with φ_tsh/φ_tsd/φ_esd Eq 11–13). The bias is
one-directional — too much clay, too little sand porosity, too little pay — so it does not average
out and it errs in the unsafe direction. No incumbent ships it.

**Betters:** the correction exists in the literature and in no commercial tool in this corpus.
Recording the donor interval makes the transferability assumption — thick-bed calibration applied to
thin beds — visible instead of buried in the method.

**Verified by.** SB-TBD-T64

#### SB-TBD-065 — Resolution enhancement is derived from published literature, never from a vendor model   [P3] [status: ABSENT]

**Requirement.** Where SandiBumi ships bed-resolution enhancement or log deconvolution for thin beds,
it MUST be derived from published literature and its own training. It MUST NOT consume a
vendor-trained model, weight file or shipped inference artifact in any format.

**Rationale.** `CONTRACT.md` §2.2 class C-3 — a vendor-trained model is never consumed. The published
routes are available on this machine: the binary-lithology deconvolution family and the Hagiwara 2023
ML deconvolution paper, the latter already implemented in Jauhar's own `petro_deconv` engine.

**Betters:** a vendor's shipped model is opaque, unversioned against the data it saw, and cannot
carry provenance into a deliverable. A natively-trained model under `SB-CORE-010` can state what it
was trained on, which is a claim no incumbent makes.
**Independent derivation, class C-3 — see §7.4.**

**Verified by.** SB-TBD-T65

#### SB-TBD-062 — Disambiguate the sand-fraction suffix from the model-name suffix   [P1] [status: ABSENT]

**Requirement.** The sand-referenced curve suffix introduced by this chapter MUST be distinguishable
from the existing model-name suffixes at a glance and by the mnemonic matcher, and the chosen
convention MUST be registered before any sand-referenced curve ships.

**Rationale.** `SB-CORE-006`. The dossier's `_SS` means *sand fraction*; SandiBumi's existing `_SSC`
(`ssc.rs:118-119`) means *sand-silt-clay model output* and `_SSPW` means *SSPW model output*.
`PHIT_SS` and `PHIT_SSC` differ by one character and mean different things — total porosity of the
sand fraction versus total porosity from the SSC model. Shipping both without settling the convention
manufactures a collision in the exact place this chapter's correctness lives.

**As-built.** `ABSENT` — the collision does not exist yet because no sand-referenced curve ships.
This requirement exists to keep it that way.

**Verified by.** SB-TBD-T48

---

## 5. Parameters

Every value below is transcribed byte-exact from the dossier's parameter tables (§2.5 and §5.3) or,
where marked, from the SandiBumi source. Nothing is re-derived, rounded or unit-converted in the
table. Where a source string is long it is compressed to file, line and page; the dossier holds the
verbatim UI string.

**Read the Value column first.** Nine of the fifty-two rows read `ABSENT — ships with no default`.
That is not incompleteness — it is the standing project decision recorded in `CONTRACT.md` §2:
silently picking one vendor's number over the others is adjudication disguised as a default. Two
further rows read `WITHDRAWN`: they are values SandiBumi ships **today** that trace to no source at
all, and `SB-TBD-066` [P0] removes them.

### 5.1 The table

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Clean-sand porosity endpoint | `PHIT_MAX` | 0.35 | v/v | Geolog V14 `lssa.info` (T1), `IN_OUT PHIT_MAX` default .35, valid 0:0.5 | T1 |
| Clean-sand endpoint as SandiBumi ships it | `PHI_SD_MAX` | **WITHDRAWN — 0.30 today, uncited** | v/v | `modules.rs:2445`. No agreeing value anywhere in the corpus; Geolog ships 0.35, IP's 0.30 is a worked-example value not a default. `SB-TBD-066` | — |
| Shale porosity | `PHIT_SH` | **ABSENT — ships with no default** | v/v | Geolog `lssa.info` (T1) l.150: no default, valid 0:0.5. Visibility predicate is `OPT_PHI=='External'` — entered **only** when porosity is supplied externally; on the shipped path Geolog derives it from `RHO_SH`/`NPHI_SH`. IP demo value 0.08 is **not** a default (ip2025 `I` §3.6) | T1 |
| Shale porosity as SandiBumi ships it | `PHI_SH` | **WITHDRAWN — 0.15 today, uncited** | v/v | `modules.rs:2446`. Also duplicated as the picker default at `crossplotPanel.ts:130`. `SB-TBD-066` | — |
| Wet-shale bulk density | `RHO_SH` | **ABSENT — ships with no default** | g/cm³ | Geolog `lssa.info` (T1) l.142: no default, no validation range. Source of `PHIT_SH` on the default porosity path | T1 |
| Wet-shale neutron porosity | `NPHI_SH` | **ABSENT — ships with no default** | v/v | Geolog `lssa.info` (T1) l.146: no default, no validation range | T1 |
| Dry-shale bulk density | `RHO_DSH` | 2.7 | g/cm³ | Geolog `lssa.info` (T1) l.141 — shipped, in contrast to the wet-shale pair | T1 |
| Dry-shale neutron porosity | `NPHI_DSH` | 0.35 | v/v | Geolog `lssa.info` (T1) l.145 | T1 |
| Structural shale cutoff | `STCT` | 0 | v/v of total shale | Geolog `lssa.info` (T1) l.152, valid 0:0.98. Action: `lssa_tech_reference.pdf` p.136-40 Table 10 — constrains structural shale volume and total porosity in the laminar-structural solution. Default 0 ⇒ laminar-dispersed only | T1 + T2 |
| Laminar shale cutoff | `LMCT` | 0.7 | v/v of total shale | Geolog `lssa.info` (T1) l.153, valid .70:.999. Action p.136-40 Table 10; acts as the pivot for the two constraint lines. The `LMCT`→`DPCT` segment is the vendor's own *"cosmetic ramping down"* — display-quality, not physics | T1 + T2 |
| Dispersed shale cutoff | `DPCT` | 1.0 (= disabled) | v/v of total shale | Geolog `lssa.info` (T1) l.154, valid 0.8:1.0. Action p.136-40 Table 10: `PHIE := 0`, `PHIT := VSH·PHIT_SH`. The only one of the three with a hard numerical action | T1 + T2 |
| Constraint flag — no constraint | `TSFLG` = 0 | 0 | code | Geolog output-curve table (T2) | T2 |
| Constraint flag — no shale present | `TSFLG` = 9 | 9 | code | Geolog `lssa_tech_reference.pdf` p.136-40, text layer | T2 |
| Constraint flag — constrained cases | `TSFLG` 1…8 | **ABSENT — not recovered** | code | Raster-only: Figures 16 (p.136-39) and 18 (p.136-41). A full text-layer search of the 57-page PDF returns no other numeric value (`OPEN-TB-13`). SandiBumi defines its own codes and does not advertise compatibility | — |
| Horizontal shale resistivity | `RH_SH` | **ABSENT — ships with no default** | ohm·m | Geolog `lssa.info` (T1): no default. Pick in thick shale | T1 |
| `RH_SH` picking guidance | — | *"no more than approximately a tenth of an ohmm less than RH"* | ohm·m | Geolog `lssa_tech_reference.pdf` p.136-45. `SB-TBD-029` enforces it | T2 |
| Vertical shale resistivity | `RV_SH` | **ABSENT — ships with no default** | ohm·m | Geolog `lssa.info` (T1): no default. Hard constraint `RH_SH ≤ RV_SH < RV` — lower bound p.136-45, upper bound derived (singular at equality, `a = CV − CV_SH = 0`) | T1 + derived |
| Shale anisotropy sanity band | `RV_SH/RH_SH` | 1.8–2.5 — **advisory, not a default** | ratio | IP 2025 vendor example range, ip2025 `I_fluidsub_thinbed_ft.md` §3.6 | T2 |
| Tensor sand-resistivity ceiling | `RT_SS_MAX` | 100 — adopted as a **flag threshold, never a clamp** | ohm·m | Geolog `lssa.info` (T1) l.157, valid 2:2000, group "Tensor Model". **The action is undocumented**: 0 hits in the entire 57-page technical reference (`D-TB-09`) | T1 |
| Tensor sand-resistivity floor | `RT_SS_MIN` | 0.2 — **flag threshold, never a clamp** | ohm·m | Geolog `lssa.info` (T1) l.158, valid 0.02:20. Same T2 silence. No counterpart floor exists in IP or Techlog anywhere in the corpus | T1 |
| Parallel-route saturation bound | `RTLAM_MAX` | 2000 — **flag threshold, parallel route only** | ohm·m | IP limit on `RtLam`/`RxoLam`, ip2018 `A_porosity_sw.md` §8 l.652. **Scope:** not the tensor-route number — Geolog's tensor bound is 20× tighter (`SB-TBD-032`) | T2 |
| Laminar shale volume range | `VLAM` | [0, 0.99] | v/v | IP limit, ip2018 `A_porosity_sw.md` §8; Geolog states 0–0.99 equivalently (ip2025 `B` §2.7) | T2 |
| Branch-flip advisory threshold | `RV_SH_flip` | `RV/(2 − RH_SH/RH)` — **computed per depth level, not a constant** | ohm·m | Derived §2.2 (`D-TB-06`), verified numerically: 7.4576 on the truth case (`RH` = 1.904762, `RV` = 11.0, `RH_SH` = 1.0), **not** `RV` = 11. Below `RV` only when `RH_SH < RH`; does not exist for `RH_SH ≥ 2·RH` | derived |
| Parallel-route pole | `VLAM_CRIT` | `RSH/RT` | v/v | Derived §3.2 | derived |
| Pole-margin warning threshold | `VLAM/VLAM_CRIT` | 0.9 | ratio | Dossier §5.5. **A reporting threshold chosen by the adoption spec with no vendor antecedent.** It changes no computed value — only when a warning fires — and is recorded as a spec choice rather than a petrophysical parameter. Escalated in §7.1 for a decision | spec |
| Root selector | `s` | Not a numeric parameter — enum `{AUTO, SAND_RESISTIVE, SAND_CONDUCTIVE}` | — | Dossier §5.3. Explicitly **does not** reproduce IP's magic-number interface | spec |
| Relative dip | `θ` | **ABSENT — ships with no default** | degrees from **bedding normal** | Techlog shipped equation raster `low-resistivity-pay-awi-resistivity-equation-3.png` (T3); the bedding-normal convention is corroborated independently by the Moran-Gianzero standard form, memory `method_thinbed_rhrv_routes.md` and `ref_thin_bed_lrlc.md` (Elhadidy entry). `SB-TBD-039` refuses to default it | T3 + T4 |
| Multi-well dip-fit span precondition | — | 40 | degrees | Memory `method_thinbed_rhrv_routes.md` — *Rv sensitivity collapses below ~40° relative dip*, so the multi-well route needs a stock spanning more than that. **The one numeric anisotropy-recovery threshold the corpus documents** (`SB-TBD-043`) | T4 |
| Isotropic-shale substitution cost | — | ∓5.2 % / +5.3 % on `RSS`, ∓0.3 % on `Vlam` at a 2× ratio error | % | Derived §3.3. **A sensitivity, not a threshold** — the corpus documents no ratio beyond which the substitution is declared invalid. Escalated in §7.2 | derived |
| Sand-fraction pay Sw cutoff | `SWE_CUT` | 0.5 | v/v | Geolog `lssa.info` (T1) l.192, valid 0:1, visible when `OPT_NET==TRUE`. Applied to the **sand-fraction** `Swe` (p.136-9) | T1 |
| Sand-fraction pay porosity cutoff | `PHIE_SS_CUT` | 0.08 | v/v | Geolog `lssa.info` (T1) l.193, valid 0:0.4. **Not numerically comparable to a conventional bulk net-pay porosity cutoff** even though it looks like one (`SB-TBD-053`) | T1 |
| Sand-fraction pay permeability cutoff | `PERM_CUT` | 0 | mD | Geolog `lssa.info` (T1) l.194, visible when `OPT_NET==TRUE` and `OPT_PERM > 0`. Applied to the sand-fraction permeability | T1 |
| Laminar summation enable | `OPT_NET` | False | logical | Geolog `lssa.info` (T1) l.191, *"Reservoir Summary Calculation Option"*. **Off by default in the vendor and adopted off** (`SB-TBD-052`) | T1 |
| Zone-merge on summation | `OPT_ZONEMERGE` | False | logical | Geolog `lssa.info` (T1) l.195 | T1 |
| Minimum-Sw guard enable | `OPT_SWLIM` | False | logical | Geolog `lssa.info` (T1) l.180 | T1 |
| Minimum clean-sand `Swe` | `SWE_MIN` | 0.08 | v/v | Geolog `lssa.info` (T1) l.181, valid 0:1, *"Minimum Swe in clean sand"*. Techlog ships the same guard with **no published default** (T3 `-thomasstieber-options.html`) | T1 |
| Formation-permeability conversion | `PERM_FM` | `PERM_SS·(1 − VSH_LAM)` | mD | Geolog `lssa_tech_reference.pdf` p.136-9 | T2 |
| Timur coefficient, fractional inputs | — | 8581 | coefficient for fractional φ and Sw | Geolog `lssa_tech_reference.pdf` p.136-51: *"8581 is the appropriate coefficient if the input is in fractions"* | T2 |
| Timur coefficient, percentage inputs | — | 0.136 | coefficient for percentage φ and Sw | Geolog p.136-51 — applies **only** to percent. The two differ by a factor of 63,000 (`SB-TBD-055`) | T2 |
| `CEC` unit convention | — | meq/100 g | meq/100 g | Geolog `lssa_tech_reference.pdf` p.136-48 Eq 113 plus an explicit vendor units warning | T2 |
| `Qv` unit convention | — | meq/mL | meq/mL | Memory `reference_waxman_smits_b.md` — the meq/mL-not-meq/L trap; **this note is the whole provenance**. Techlog labels `Qv` "(1/L)" against `B Value` "L·S/m", internally consistent per-litre; whether that is offset by 1000× is **not established** (`OPEN-TB-14`) | T4 |
| `B` unit convention | — | S/m per (meq/mL) — **paired with the `Qv` convention above** | — | Geolog computes `B` from Eq 108 (p.136-47) and exposes no `B` parameter; Techlog ships "B Value" default 0, unit L·S/m (T3); IP's "B fact Juhasz" default is written "1.0 meq/ml" — `B` labelled with `Qv`'s unit. **Do not propagate IP's unit label.** Seam: `12_saturation.md` | T1/T2/T3 |
| `B` correlation leading constant | — | **ABSENT — ships with no default; competing values −1.25 and −1.28** | — | Geolog prints −1.25, IP prints −1.28 for the same named Juhász (1981) correlation with all other coefficients identical (`D-TB-07`). ≈ 0.2 % in `B` at 200 °F. **Not adjudicated.** Seam: `12_saturation.md` owns the choice; this chapter requires that whichever is used is named in the source string | T2 |
| Archie `a` | `A` | 1 | — | Geolog `lssa.info` (T1) default 1, valid 0.05:5.0. **Seam: `12_saturation.md` owns this**; carried here because the laminated dispatch consumes it | T1 |
| Archie `m` | `M` | 2 | — | Geolog `lssa.info` (T1) default 2, valid 0.5:5.0. Laminated convention ≈ 1.8: Techlog 2018.2 `-thomasstieber-saturation-tab.html` (T3) *"usually close to 1.8 in the sands"*; a project record lowered `m` = `n` to 1.8 for a laminated clastic section. **Seam: `12_saturation.md`** | T1 + T3 + T4 |
| Archie `n` | `N` | 2 | — | Geolog `lssa.info` (T1) default 2, valid 0.5:5.0. Same 1.8 laminated convention. **Seam: `12_saturation.md`** | T1 + T3 + T4 |
| `CEC` sanity band, laminated clastics | — | 0.75–3.21 — **advisory, not a default** | meq/100 g | Project record: SCAL wet-chemistry `CEC` across 8 plugs in a laminated sand-silt-clay section. Geolog's valid range is 0–300 and is not a recommendation | T4 |
| Bed-thickness scenario boundaries | — | 60–10 / 10–3 / 3–1 / <1 / mixed | cm | Madjid & Worthington SPE 163071 scenarios A–E, via `docs/research_2026-07/ref_thin_bed_lrlc.md` | T4 |
| `Vcm` regression coefficients | `K` factors | **ABSENT — fitted per depositional system, never defaulted** | — | Madjid & Worthington SPE 163071 Eq 8–10 with φ_tsh (11), φ_tsd (12), φ_esd (13). Calibration chain: core porosity → `ρ_bc` → a `Vcm`-vs-`Vsh` regression fitted on **thick** beds and exported to the thin beds | T4 |
| VLSA published validation figures | — | 2.41 / 1.41 / 2.40 | ft (HPT) | Memory `reference_thinbed_deconv_vlsa_tools.md` — Passey's own True / Conventional / VLSA validation. Used as the acceptance fixture for `SB-TBD-059`, not as a shipped parameter | T4 |
| LRLC recognition taxonomy dimensions | — | 4 manifestations × 6 causes | — | Worthington (2000), via `docs/research_2026-07/ref_thin_bed_lrlc.md` | T4 |
| `sw_rtc` / `sw_imts` calibration constants | — | **SEAM — `12_saturation.md` owns every value** | — | `lrlc.rs:73-117`, `lrlc.rs:179-224`. Carried here only to record that `sw_rtc`'s doc string labels its own calibration *"one study's calibration … from one field"* and `sw_imts`'s `S_FACTOR_GW` is labelled *"A PROPERTY OF THE ROCK AND OF THE CLAY CURVES IT IS PAIRED WITH"* and ships absent, the run refusing without it (DEC-094). Both bear on §7.5 | source |

### 5.2 Counts and what they mean

Fifty-two rows. **Nine** read `ABSENT — ships with no default`. **Two** read `WITHDRAWN`. **One**
reads `SEAM` — carried only to record that the values exist and that `12_saturation.md` owns them.
**Zero** read `NON-ADOPTABLE`: nothing in this domain is a vendor chart digitization or a
licensing-blocked value, so no row needed that disposition.

The remaining forty are cited values. **Two of those forty are advisory bands, not defaults** — the
1.8–2.5 shale-anisotropy sanity band and the 0.75–3.21 laminated-clastic `CEC` band — and both say so
in the Value column, because a sanity band silently promoted to a default is the failure this
discipline exists to prevent.

**Five rows are not petrophysical parameters** and carry `spec` or `derived` in place of a tier: the
root-selector enum and the pole-margin warning threshold (`spec`); the branch-flip formula, the
parallel-route pole and the isotropic-substitution sensitivity (`derived`, each reproducible from
§2). Four of the five are structural or reproducible. The fifth — the pole-margin threshold of 0.9 —
is the only number in this chapter chosen by the adoption spec rather than taken from a source, and
it is recorded as such rather than dressed up: it changes no computed value, only when a warning
fires. It is raised in §7.1 as O-1 for a decision rather than adopted silently.

**On the ABSENT count specifically.** Nine is the honest number and it is lower than a first reading
of §4 suggests, because most of what this chapter specifies is *behaviour* — guards, flags, dispatch,
naming, refusals — which needs no parameter at all. The nine that are absent are the ones that
matter: both shale-resistivity picks, the shale porosity and its two supporting endpoints, the
relative dip, the unrecovered flag codes, the disputed correlation constant and the clay-mineral
regression coefficients. **Every one of them is a value a competitor ships with a default, and every
one of them would change a saturation.**

**Two rows carry values SandiBumi ships today that trace to nothing.** `PHI_SD_MAX` = 0.30 and
`PHI_SH` = 0.15 at `modules.rs:2445-2446` are the whole of `SB-TBD-066` [P0]. The corpus does not
supply replacement numbers — it supplies a **method**: Geolog derives shale porosity from picked
wet- and dry-shale endpoints on its default path and only accepts a typed value when porosity comes
from outside. That is what should be adopted, and it is why `SB-TBD-066` specifies a derivation
rather than a number.

---

## 6. Acceptance tests

Sixty-six tests. Every expected value is either cited to a source or derived with the arithmetic
shown in the test body so a reviewer can check it without re-deriving it. Four tests are labelled
`CHARACTERIZATION` because they pin current behaviour rather than assert a sourced expectation; they
are kept deliberately and are marked so nobody mistakes them for correctness evidence.

Where a fixture uses the tensor truth case, it is: `RSS` = 20, `RSH_H` = 1.0, `RSH_V` = 2.0,
`VLAM` = 0.50 ⇒ `RH` = 1.904762, `RV` = 11.0.

### 6.1 Recognition and routing

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T01** | Synthetic interval with each of the six Worthington causes constructed in turn | Run the recognition screen | Each interval classifies to its constructed cause; the coded curve is populated at every level | Worthington (2000) taxonomy via `ref_thin_bed_lrlc.md` |
| **SB-TBD-T02** | Bed thicknesses of 40, 6, 2 and 0.5 cm against a fixed resistivity resolution | Run the scenario router | Assign scenarios A, B, C, D respectively; the thickness **source** field is populated and non-default on every output | Madjid & Worthington SPE 163071 scenario boundaries 60–10 / 10–3 / 3–1 / <1 cm |
| **SB-TBD-T03** | A gas-bearing interval classified to a fine-grain/microporosity cause, with the SSPW porosity route selected | Dispatch through the route table | The run surfaces the open porosity defect on that route before computing; the route table is readable as data, not parsed from prose | `11_porosity.md` `SB-POR-059`; `ssc.rs:433` |
| **SB-TBD-T04** | An interval carrying a coal lamina, and a carbonate interval | Run any tensor or T-S module | Both raise their declared-scope flags; the two-component and lithology declarations are machine-readable on the spec | dossier §2.6 |

### 6.2 Thomas-Stieber

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T05** | `VSH` = `VLAM`, endpoints `PHIT_MAX`, `PHIT_SH` | T-S laminar-dispersed | Reproduces the laminated line `PHIT = PHIT_MAX − VLAM·(PHIT_MAX − PHIT_SH)` exactly | derived, dossier §2.1 |
| **SB-TBD-T06** | `VLAM` = 0 | T-S laminar-dispersed | Reproduces the dispersed line `PHIT = PHIT_MAX − (1 − PHIT_SH)·VSH` exactly | derived, dossier §2.1 |
| **SB-TBD-T07** | A point outside the T-S solution space | Run the module | `VSH_LAM` is **not** a clipped value; `PHIT` was constrained instead; the constraint flag is non-zero; the reported `PHIT` shift is non-zero. A `clamp()` on any derived fraction fails this test | dossier §5.2/§5.5; `SB-CORE-002` |
| **SB-TBD-T08** | The four `(PHIT, VSH)` pairs used in F-3 and F-5 | Current `thin_bed_ts` against the new Eq 86 implementation | `VLAM` agrees to machine precision. `CHARACTERIZATION` for the current implementation's outputs; the Eq 86 side is sourced | Geolog Eq 86, p.136-42 |
| **SB-TBD-T09** | A point 0.04 v/v below the dispersed line | Run the module | The emitted `PHIT` shift equals the constructed offset to within 1e-6 v/v, and its sign points toward the boundary | Geolog p.136-38 rationale |
| **SB-TBD-T10** | `PHIT` = 0.16, `VSH` = 0.40, `PHIT_MAX` = 0.30, `PHIT_SH` = 0.15 | Compute the sand-fraction porosities | `PHIT_SS` = 0.164000 and `PHIE_SS` = 0.140000 (± 1e-6); the two are emitted as **different named curves**; no curve named `PHIE_LAM` is emitted. Arithmetic: `VLAM` = 0.285714, `PHIT_SS` = (0.16 − 0.15·0.285714)/0.714286, `PHIE_SS` = `PHIT_SS` − `VDISP`·`PHIT_SH`/(1 − `VLAM`) | Geolog Eq 88 / Eq 89, p.136-42 |
| **SB-TBD-T11** | Arbitrary in-range `(PHIT, VSH, PHIT_MAX, PHIT_SH)`, property-tested over the domain | Compare Eq 89 against Eq 122 | Equal to machine precision at every sample | Geolog Eq 122, p.136-51 |
| **SB-TBD-T12** | A point below-left of the dispersed-pore-filling boundary | Run the module | The point is **not** constrained; the back-solved clean-sand porosity and its difference from the picked endpoint are emitted | Geolog p.136-39 |
| **SB-TBD-T13** | Drag the clean-sand handle from 0.30 to 0.283 and release | Read the persisted parameter record | The record carries the value **and** the interactive origin, plot identity, well, depth interval and date; a `SB-CORE-010` provenance query on the resulting curve reaches the pick | `SB-CORE-010`; `crossplotPanel.ts:2146` |
| **SB-TBD-T14** | Attempt to drag the clean-sand handle to 0.48 | Read the drag clamp and the persisted value | The drag is bounded by the **`ModuleSpec`** range, not by a hard-coded frontend constant; the value 0.48 never reaches `zone_params` | `modules.rs:2445`; `crossplotPanel.ts:2119`; `SB-CORE-007` |
| **SB-TBD-T15** | One `VSH_LAM` value, both branches, branch set per zone | Run both T-S branches | Both return the same sand-fraction total porosity (Eq 83 ≡ Eq 88) exactly; the branch does not change between adjacent samples within a zone | Geolog p.136-42 |
| **SB-TBD-T16** | `VLAM` = 0, laminar-structural branch | Run the module | Reproduces the structural line `PHIT = PHIT_MAX + PHIT_SH·VSH` exactly | derived, dossier §2.1 |
| **SB-TBD-T17** | An interval spanning `VSH` from 0.5 to 0.95 with all three cutoffs active | Run the module and read the outputs | The dispersed cutoff sets values while the other two position lines; the laminar→dispersed segment is labelled display-quality in both the flag text and the deliverable | Geolog p.136-40 Table 10 |
| **SB-TBD-T18** | A well with both T-S and tensor routes run | Read the emitted curves | Three distinct laminar-shale curves exist — T-S, tensor, and the selected one — and the selection is recorded | dossier §5.5 |
| **SB-TBD-T19** | Two `VLAM` curves imported from two different vendor projects | Attempt a difference | The comparison refuses, or proceeds only after the differing parameterizations are declared | F-1 |
| **SB-TBD-T20** | A run in which 40 % of levels raise at least one guard | Read the run summary | Every condition's flagged-level count appears; suppressed levels are counted, not silently dropped; a clean run and this run are distinguishable from the summary alone | `SB-CORE-002`; Geolog p.136-10 |
| **SB-TBD-T21** | A curve or method named `Steiber` | Import | Maps to `Stieber` | dossier §4.10 (`D-03`) |
| **SB-TBD-T22** | An interval with 400 levels spanning two facies | Render the T-S crossplot | Every boundary of the active construction is drawn, the active cutoff lines are drawn, and points are coloured by depth | Geolog p.136-11 |
| **SB-TBD-T23** | A fresh install, no analyst input | Read the `thin_bed_ts` spec | Neither porosity endpoint ships an uncited numeric default; the shale-porosity path derives from wet/dry-shale endpoints unless porosity is supplied externally | `SB-CORE-004`; Geolog `lssa.info` l.150 |
| **SB-TBD-T24** | Picked wet- and dry-shale endpoints | Derive shale porosity | The derived value matches a hand computation from the picked endpoints to 1e-6 v/v, and its provenance names the two picks | Geolog `lssa.info` l.141-150 |

### 6.3 Resistivity tensor

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T25** | A synthetic laminated stack of known `RSS`, `RSH_V`, `VLAM` | Forward the series law and invert | `RV = (1 − VLAM)·RSS + VLAM·RSH_V` round-trips exactly. Asserts the implementation is a **resistivity** mix, guarding `D-TB-04` | Geolog Eq 92 as repaired; derived §2.2 |
| **SB-TBD-T26** | `RSS` = 20, `Rsh` = 1, `VLAM` = 0.5 ⇒ `RH` = 1.9048, `RV` = 10.5 | Isotropic tensor round-trip | Recovers `RSS` = 20.000 and `VLAM` = 0.5000 exactly | derived, dossier §2.2 |
| **SB-TBD-T27** | The truth case | Anisotropic tensor round-trip | Recovers `RSS` = 20.000 and `VLAM` = 0.5000 exactly. Additionally assert the `(1 − ΔC)` sign variant **fails**, returning `RSS` ≈ 5.92 — the wrong sign must not be in the code | derived, dossier §2.2 |
| **SB-TBD-T28** | `CH_SH = CV_SH` (ΔC = 0) | Anisotropic form | Degenerates exactly to the isotropic form; both roots are available as outputs | Geolog p.136-6 |
| **SB-TBD-T29** | The truth case, sweeping `RV_SH` ∈ {2, 5, 7.40, 7.46, 8, 10} | Solve and select | (a) the implementation computes `RV_SH_flip` = **7.4576 ± 0.001** and asserts it is strictly less than `RV`; (b) the physical root is monotone-decreasing 20.000 → 16.883 → 14.455 → 14.40⁻ → 13.860 → 11.929 with **no discontinuity at 7.4576**; (c) a fixed-sign closed form agrees below the flip (14.455 at 7.40) and diverges immediately above it (0.518 at 7.46, 0.456 at 8.0). The divergence must be shown to begin at 7.46, **not** at 11 | derived, `D-TB-06` |
| **SB-TBD-T30** | The truth case, baseline `RV_SH` = 2.0 ⇒ `RSS` = 20.000 | Sweep `RV_SH` | 1.0 → 21.053 (+5.3 %); 3.0 → 18.954 (−5.2 %); 4.0 → 17.914 (−10.4 %); 12.0 → 10.103 (−49.5 %); 15.0 → 7.676 (−61.6 %). Tolerance ± 0.5 % | derived, dossier §2.2/§3.3 |
| **SB-TBD-T31** | `RH_SH` swept across `RH` and `2·RH` | Compute `RV_SH_flip` | `< RV` when `RH_SH < RH`; `= RV` when `RH_SH = RH`; `> RV` when `RH < RH_SH < 2·RH` (e.g. `RH_SH` = 2.5 → 16.0); **no positive root** when `RH_SH ≥ 2·RH`. Guards against hard-coding the threshold as a constant or as `RV` | derived, `D-TB-06` |
| **SB-TBD-T32** | `RH` = 1.9048, `RV` = 11, `RH_SH` = `RV_SH` = 2.0 | Attempt the solve | The **input quadrant test** raises its named flag and **no number is returned**. Explicitly assert the test does *not* look for the vendor's printed "negative Vlam + infinite RSS" signature: the unguarded solve yields `RSS` = −180 / `VSH_LAM_TN` = +1.0495 on the isotropic form and `RSS` = 2.545 / `Vlam` = 18.59 alongside `RSS` = −25.15 / `Vlam` = 1.284 on the quadratic, none of which matches | derived, dossier §3.4 |
| **SB-TBD-T33** | `RV_SH` = `RV` exactly | Attempt the solve | The crossover flag is raised and no number is returned — no division by zero, no ±inf, no large finite value | derived §2.2; Geolog p.136-45 |
| **SB-TBD-T34** | The truth case, `RV_SH` = 10.99 with `RV` = 11.0 | Solve | The physical root satisfies `RSS` → `RV`: 11.009 ± 0.01 | derived §2.2; Geolog p.136-45 |
| **SB-TBD-T35** | The truth case with `RH_SH` = 1.1 (+10 %) | Solve | `RSS` = 22.25 (+11.2 %), `VLAM` = 0.5555, net sand 0.4445 (−11.1 %), all ± 0.1 %; the proximity warning is raised | derived, dossier §3.3 |
| **SB-TBD-T36** | `RV_SH` set below `RH_SH` | Set the parameter | Rejected **at entry**, not at solve time, with a message naming the constraint and its source | Geolog p.136-45 |
| **SB-TBD-T37** | A synthetic gas-sand case with true `RSS` = 250 ohm·m, and symmetrically `RSS` < 0.2 | Solve on the tensor route | Returns **250** exactly, plus the advisory naming the vendor bound — never 100. Additionally assert the 2000 ohm·m parallel-route bound is not applied on this path | `lssa.info` l.157-158; ip2018 §8 l.652 |
| **SB-TBD-T38** | Each condition in the `SB-TBD-034` table, one at a time | Inspect the `ModuleSpec` and run | Every condition is present as machine-readable data with its severity, message and **citation**; the framework evaluates it before the solve; the citation reaches the flag message | `SB-CORE-003` |
| **SB-TBD-T39** | `VLAM` = 0.55 against `RT` = 1.9048, `RSH` = 1.0 | Parallel solve | The pole flag is raised rather than the value clamped; past the pole a negative `RSS` raises its **own distinct** condition, never the saturated-value flag | derived, dossier §3.2 |
| **SB-TBD-T40** | `VLAM` = 0.4, `RT` = 2.42718, with `RSH` = 1.0, 1.5, 2.0 | Parallel solve | 50.00, **4.13** (± 0.01), 2.83. The test body carries a comment recording that IP's manual prints 5.75 for the middle case | ip2018 §8 + derived §3.1 (`D-TB-01`) |
| **SB-TBD-T41** | An anisotropy ratio admitting no root | Solve | Flag and stop for that level; the ratio is unchanged on the output. A silently reduced ratio fails this test | dossier §4.10 (IP `PhiFlag = 15`) |

### 6.4 Relative dip

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T42** | `Rh`, `Rv` known; θ = 0 and θ = 90° | Forward the dip relation | θ = 0 → `Rt` = `Rh` exactly; θ = 90° → `Rt` = √(`Rh`·`Rv`) exactly | Techlog raster; derived §2.3 |
| **SB-TBD-T43** | The same case, passing 90 − θ instead of θ | Forward the dip relation | The answer **differs**. Guards the bedding-plane/bedding-normal swap; the parameter name states the convention | dossier §2.0; `SB-CORE-013` |
| **SB-TBD-T44** | A tensor run with no dip source and no bedding-frame declaration | Run | Refused with a message naming both ways to satisfy it. θ is never silently 0 | dossier §5.5 |
| **SB-TBD-T45** | A synthetic multi-well stock spanning 15° to 75° relative dip, with known `Rh`/`Rv` | Run the multi-well fit | Recovers the constructed pair within the stated residual, and the per-well residual is reported | `ref_thin_bed_lrlc.md` (Elhadidy entry) |
| **SB-TBD-T46** | A well stock spanning 12° | Run the multi-well fit | Rejected, with the actual span reported against the threshold | memory `method_thinbed_rhrv_routes.md` |

### 6.5 Sand-referenced saturation and bookkeeping

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T47** | A laminated well with the model active | Run any saturation model | The model is evaluated on the sand-fraction porosity and sand resistivity; asserting the bulk-input variant gives a **different** answer, so the two can never be silently interchanged | Geolog dispatch rule, T2 + T4 deck |
| **SB-TBD-T48** | A completed laminated run | Read the emitted curve set | Sand-fraction volume, sand-fraction total and effective porosity, and sand resistivity are all present as named curves; the sand-fraction suffix is distinguishable from the SSC and SSPW model suffixes by the mnemonic matcher | `SB-CORE-006`; `ssc.rs:118-119` |
| **SB-TBD-T49** | The same well, `sw_rtc` and `sw_imts` selected | Run | Both consume the sand-fraction porosity; asserting the bulk-porosity path gives a different answer | `lrlc.rs:123`, `lrlc.rs:228` |
| **SB-TBD-T50** | The laminated model active; request Poupon, Poupon-Aguilera and Poupon-Tixier in turn | Dispatch | All three refused, each with the vendor's stated reason surfaced. Three separate assertions, keyed on **equation identity** not on the name string | ip2018 §8 l.697-704; `laminated_sands_workflow.htm` l.98 |
| **SB-TBD-T51** | An interval where the minimum-`Swe` guard fires | Run with the guard enabled | Both clipped and unclipped saturations are emitted, and the fired levels are flagged | `lssa.info` l.180-181; `SB-CORE-002` |
| **SB-TBD-T52** | Three synthetic intervals constructed for agreement, `TS` high / `TN` low, and `TS` low / `TN` high | Run the reconciliation | Each classifies to its constructed case; the vendor's stated interpretation and prescribed response are attached to the output; the track renders the shaded mismatch | Geolog p.136-7/8 |
| **SB-TBD-T53** | A 10 m interval containing 50 % laminar shale and 50 % porous hydrocarbon-bearing sand | Run the laminar summation with the mode enabled | Net pay = **5 m** exactly. Additionally assert the mode is **off** by default and that enabling it carries the vendor's reserves warning into the deliverable header | Geolog p.136-9; `lssa.info` l.191 |
| **SB-TBD-T54** | The same interval, cutoffs at the sand-fraction values | Run net sand and net pay | Net sand counts where the sand-fraction porosity and/or permeability cutoffs pass; net pay additionally requires the sand-fraction `Swe` cutoff — **all on sand-fraction curves**. Assert a bulk-curve implementation gives a different (larger) answer | Geolog p.136-9; `lssa.info` l.191-195 |
| **SB-TBD-T66** | A laminated interval with a computed sand-fraction saturation | Renormalize to bulk | Bulk hydrocarbon volume is conserved exactly between the two reference frames | derived, dossier §2.2 |

### 6.6 Plots, permeability and units

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T55** | The forward-modelled overlay grid | Render the Klein / butterfly plot | Reproduces the reference-fixture grid; the iso-sand-resistivity and iso-net-to-gross families, the shale point and the `Rv` = `Rh` line are all present | Jauhar's `Klein plot trial.xlsx` prototype as the reference fixture |
| **SB-TBD-T56** | The overlay generator at shale fraction = 1 | Evaluate the horizontal mixing law | Returns `R_SH_H`. The erroneous prose form would give `1/R_SH_H`; at `R_SH_H` = 1 ohm·m the two are indistinguishable, so the test is run at `R_SH_H` = 4 ohm·m where they differ by 16× | `D-TB-05`, Techlog raster |
| **SB-TBD-T57** | The same porosity and saturation expressed as fractions and as percentages | Timur transform | 8581 with fractional inputs equals 0.136 with percentage inputs, exactly. A coefficient used against the wrong unit type is rejected, not silently computed | Geolog p.136-51 |
| **SB-TBD-T58** | A vendor project carrying a `Qv` curve and a `B` coefficient | Import | Both convert under one declared convention, or the import is refused. Importing `Qv` alone is rejected | `OPEN-TB-14`; memory `reference_waxman_smits_b.md` |
| **SB-TBD-T59** | The same well run in metric and in imperial units | Compare interval statistics | Laminar net sand and cumulative hydrocarbon agree to machine precision. Additionally assert that any temperature entering a correlation is unit-typed and lies in a physical range — a °F value entering a °C slot is rejected, not computed | `SB-CORE-013`; derived §3.7 |
| **SB-TBD-T60** | A Techlog project carrying both Thomas-Stieber sand-fraction curves | Import | Both import under their vendor names, unmapped, with the ambiguity recorded. Neither is mapped onto a SandiBumi curve | `OPEN-TB-15` |
| **SB-TBD-T61** | A laminated interval with a sand-fraction permeability | Convert to formation permeability | `PERM_FM = PERM_SS·(1 − VSH_LAM)` exactly; both curves are emitted | Geolog p.136-9 |
| **SB-TBD-T62** | A well with a thick shale interval | Render the anisotropy track | Both anisotropy ratios render on the common scale over the shale interval | ip2025 `I` §3.6 |

### 6.7 Uncertainty, resolution and characterization

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **SB-TBD-T63** | Passey's published validation case | Run the interval Monte-Carlo | True / Conventional / VLSA hydrocarbon pore thickness = **2.41 / 1.41 / 2.40 ft** as published; the output is a distribution, not a scalar | memory `reference_thinbed_deconv_vlsa_tools.md` |
| **SB-TBD-T64** | A synthetic system with thick and thin beds of identical composition | Run the clay-mineral correction | The correction removes the constructed thin-bed clay over-estimate; the donor (thick-bed) interval and the fitted coefficients are recorded on the output | Madjid & Worthington SPE 163071 Eq 8–10 |
| **SB-TBD-T65** | Any resolution-enhancement model artifact | Load | A vendor-supplied model, weight file or inference artifact is refused in every format; a natively-trained model reports what it was trained on | `CONTRACT.md` §2.2 class C-3; `SB-CORE-010` |

### 6.8 Characterization tests, declared

Four tests pin current behaviour rather than a sourced expectation and are labelled
`CHARACTERIZATION` in the source. They exist so a refactor is detectable, **not** as evidence of
correctness, and each carries a comment saying so.

| ID | What it pins | Why it is not a correctness test |
|---|---|---|
| **SB-TBD-T08** (partial) | The current `thin_bed_ts` numeric outputs at four `(PHIT, VSH)` pairs | Half of it is sourced — the Eq 86 comparison. The "current output" half is a snapshot |
| **SB-TBD-T22** (partial) | The rendered crossplot's current axis auto-scaling | No source specifies the auto-scale; only the drawn geometry is sourced |
| **SB-TBD-T55** (partial) | The overlay grid spacing | The mixing laws are sourced; the grid density is a rendering choice |
| **SB-TBD-T62** | The anisotropy track's 1–10 scale | Taken from a vendor screenshot, not from a stated specification |

---

## 7. Open items, escalations and refusals

Per `CONTRACT.md` §2.2.1 the refusals are split into two lists that must never be merged: §7.3 holds
**defect refusals**, which are wins, and §7.4 holds **independent-derivation requirements**, which are
obligations. §7.5 answers the competitive question this chapter was commissioned to answer.

### 7.1 Open

Needed, not yet answerable, with what would settle each.

**O-1 — The pole-margin warning threshold of 0.9 has no source.** It is the only number in §5 chosen
by the adoption spec rather than taken from evidence. It changes no computed value, only when a
warning fires, so it is a reporting choice rather than a petrophysical parameter — but it sits in a
table of sourced numbers and must not sit there unmarked. **Settled by:** Jauhar setting it as a house
reporting convention, or by replacing the fixed threshold with a reported margin and no threshold at
all.

**O-2 — The `TSFLG` constraint codes 1…8 are unrecovered.** They live in two figures in the vendor
PDF with no text equivalent (`OPEN-TB-13`). §4 adopts only the two recovered codes and defines
SandiBumi's own for the rest. **Settled by:** a visual read of those two figures — cheap, not done
here, and explicitly **not to be guessed**. Until then SandiBumi must not advertise flag
compatibility.

**O-3 — Techlog's Thomas-Stieber solution algebra is not printed anywhere in the local tree.** Only
the zeta definition and the option semantics are (`OPEN-TB-2`). The compute is compiled, so no script
route exists. **Settled by:** a live session with the module open, or the published side-by-side study
of that vendor's low-resistivity inversion against its Thomas-Stieber module. **Not blocking** —
Geolog's equations are complete and are what §4 specifies.

**O-4 — The microscopic shale-anisotropy correction Techlog names is unexplained.** It prints only
the three macroscopic response equations (`OPEN-TB-3`). **Settled by:** the named paper series, not in
the local library. **Not blocking for v1** — the Moran-Gianzero dip term covers the macroscopic part.

**O-5 — "Floating m" for laminated shale is proposed in a project record and never specified**
(`OPEN-TB-4`). Is `m` a function of laminar shale volume, of sand-fraction porosity, or zone-stepped?
**Settled by:** that study's final report. **Not blocking** — ship constant `m` with the ≈ 1.8
laminated guidance surfaced. Seam: `12_saturation.md`.

**O-6 — Two IP parameter columns were cut off in the source raster** (`OPEN-TB-5`). **Settled by:** a
live IP session. **Not blocking** — Geolog's equations cover the same ground.

**O-7 — No uncertainty propagation exists in any of the three closed-form implementations**
(`OPEN-TB-12`). **Settled by:** `SB-TBD-059`. **Not blocking v1; required before these results book a
reserve.**

**O-8 — Whether Techlog's per-litre cation-exchange convention is genuinely offset from the meq/mL
literature** (`OPEN-TB-14`) turns on that vendor's compiled definition and is not established.
**Settled by:** a round-trip against a case with an independently known product. **Not blocking** —
`SB-TBD-056` converts the pair or refuses.

**O-9 — Techlog's two sand-fraction curves are not distinguished by any read page** (`OPEN-TB-15`).
**Settled by:** a live run comparing the two on one well. **Not blocking** — `SB-TBD-058` refuses to
map either.

**O-10 — The `_SS` sand-fraction suffix collides visually with the existing `_SSC` and `_SSPW` model
suffixes.** `SB-TBD-062` requires the convention be settled **before** any sand-referenced curve
ships. **Settled by:** a house naming decision. Cheap now, expensive after the first delivered LAS.

**O-11 — No new `SB-CORE` id is requested, and that is a considered finding rather than an
omission.** The strongest candidate — a core rule that an interactive picking surface must implement
the same model as the module it parameterizes — is already inside `SB-CORE-006`, which requires that
*"the emitted method flag, the UI label, the doc comment and the equation MUST agree"*. The drawn
construction is a UI representation of the method, so F-3 is an `SB-CORE-006` instance and is carried
as `SB-TBD-006`. Likewise the un-provenanced interactive pick is already inside `SB-CORE-010`, which
requires every parameter value **and that value's source string** travel into every deliverable; that
is `SB-TBD-012`. Recorded so the absence of a request is visibly considered.

### 7.2 Escalation

Each is a real question with a checkable answer, needing Jauhar or a source not on this machine.

**E-1 — `SB-CORE-003` asks this chapter for "the documented anisotropy threshold beyond which a
weak-anisotropy substitution is invalid". The corpus does not contain one, and this chapter has not
invented one.** What the corpus documents is a **sensitivity**, not a boundary: assuming isotropic
shale where the true anisotropy ratio is 2 costs +5.3 % on sand resistivity and +0.3 % on laminar
shale volume; the converse costs −5.2 % and −0.3 %. That is a magnitude. The one genuine numeric
threshold in the anisotropy space is the ~40° relative-dip-span floor below which `Rv` sensitivity
collapses, and `SB-TBD-043` ships it as machine-readable data. **The exact question:** does Klein
(1995/1997) or Mollison et al. (1999) state a shale-anisotropy ratio beyond which the isotropic
substitution is declared *invalid*, as opposed to merely inaccurate? Neither paper is on this machine.
**Until answered, `SB-CORE-003` is discharged by `SB-TBD-034` for every condition the corpus does
document, and recorded as incomplete for this one.** Inventing the number would be exactly the failure
`CONTRACT.md` §5.2 forbids.

**E-2 — Mollison et al. (1999) is Geolog's stated methodology reference and is not in the local
library.** The library holds the 2002 successor instead, whose printed equations match Geolog's
mixing laws (`OPEN-TB-9`). **The exact question:** does the 1999 paper contain the root-selection rule
that `lssa.info` does not expose? This is the single source most likely to close both E-1 and the
first C-3 item in §7.4. Named-paper acquisition.

**E-3 — The multi-well dip-fit route is blocked on a paper that does not close under its own
equations.** Aldred (2017) prints sand resistivities of 72.6 (series), 4.3 (parallel) and ~103
(simultaneous) at a laminar shale volume of ~0.49 against a stated "25 %" label; the disagreement
turns on total-`Vsh` versus laminar-`Vsh` and is never disambiguated (`OPEN-TB-6`). **A ~40 % swing in
sand resistivity propagates straight into Sw.** **The exact question:** which shale volume do the
printed equations take? **This is blocking for `SB-TBD-042`** and must be settled before any
multi-well dip-fit result reaches a client.

**E-4 — A KB note on this machine carries a wrong-signed diagnostic and should be corrected.** For the
case where tensor laminar shale exceeds Thomas-Stieber laminar shale, the note infers *"the laminae
are more conductive than the bounding shale"*; the vendor's technical reference states *"thin highly
**resistive** layers"* — opposite sign. The note also reaches the right conclusion by the wrong
mechanism for the converse case. **The exact action:** retract the sign-inverted inference and record
the vendor mechanism. A wrong-signed diagnostic is worse than an absent one, and this one is currently
retrievable as if it were fact. Outside this chapter's write scope.

**E-5 — The two withdrawn endpoint defaults need replacements, and the corpus supplies a method
rather than numbers.** `SB-TBD-066` specifies deriving shale porosity from picked wet/dry-shale
endpoints. **The exact question for Jauhar:** are there endpoint picks from a delivered study that
should ship as cited defaults, or does the pair ship ABSENT? Shipping ABSENT is the safe answer and is
what §5 currently records.

**E-6 — Ownership of the resistivity-independent saturation capability is unallocated, and this
chapter has not claimed it.** See §7.4 item C-1-1. **The exact question:** does the capability belong
here, framed as an LRLC route, or to `25_fluidsub-rockphysics.md`, framed as an acoustic/elastic
method? **This chapter's view:** it belongs to `25_fluidsub-rockphysics.md`. The method is an elastic
inversion; low-contrast pay is the *application*, not the mechanism, and §1's own boundary rule — if
the quantity being made anisotropic is a resistivity it is here, if it is a modulus or a velocity it
is there — points the same way. But that chapter is not yet written, and the strategic exposure is
Axis 3's, which is here. **The call is Jauhar's, not this chapter's to make unilaterally.**

### 7.3 Defect refusals — things SandiBumi deliberately will not do

These are competitive wins. Each states what SandiBumi does instead and why that is correct.

**R-1 — SandiBumi will not clamp a derived volume fraction or a derived sand porosity into range.**
*Instead:* constrain the input in the total-porosity direction, recompute, and emit a coded flag
carrying the signed shift (`SB-TBD-007`, `SB-TBD-008`). *Why:* a clamped value is indistinguishable
from a computed one at every downstream consumer (`SB-CORE-002`). This refusal applies to SandiBumi's
**own** three shipped clamps at `modules.rs:2473`, `:2477` and `:2486` as much as to any incumbent's,
which is the only honest way to hold it.

**R-2 — SandiBumi will not silently reduce an anisotropy ratio it cannot solve.**
*Instead:* flag and stop for that level (`SB-TBD-037`). *Why:* the incumbent's behaviour proceeds on a
parameter the user never set and is never told about, and the result is in-range and plottable.

**R-3 — SandiBumi will not write a saturation bound as if it were a computed value.**
*Instead:* preserve the computed value, raise the bound as an advisory, and treat a negative result
past the pole as its own distinct condition (`SB-TBD-031`, `SB-TBD-035`, `SB-TBD-036`).

**R-4 — SandiBumi will not clamp the tensor sand resistivity to the vendor's shipped bounds.**
*Instead:* flag, preserve, and name the vendor as the source of the bound so a cross-tool mismatch is
explainable (`SB-TBD-031`). *Why:* the bounds are real T1 manifest evidence but their **action**
appears nowhere in 57 pages of the vendor's own technical reference (`D-TB-09`); a gas-bearing
laminated sand legitimately exceeds them, and the same manual states the sand resistivity legitimately
exceeds `RV`.

**R-5 — SandiBumi will not transcribe the anisotropic-shale equation as printed.** Neither vendor
printing is correct, and they differ in three places. *Instead:* implement the quadratic, label the
transcription as repaired, and record that **no printing in the corpus is correct in one piece**
(`SB-TBD-023`). *Why:* `CONTRACT.md` §5.1 — the vendors' own defects are the opportunity.

**R-6 — SandiBumi will not implement the series mixing law with the left-hand side as printed.** The
vendor prints a conductivity equated to a resistivity expression. *Instead:* implement the resistivity
mix and guard it with a round-trip test (`SB-TBD-022`, `SB-TBD-T25`).

**R-7 — SandiBumi will not transcribe the Klein-plot horizontal mixing law from the vendor's prose.**
The prose prints a multiplication where the vendor's own raster on the same page prints a division.
*Instead:* use the division form and test it at a shale resistivity where the two differ
(`SB-TBD-054`, `SB-TBD-T56`). *Why:* the error is **invisible at 1 ohm·m**, which is the shale
resistivity typical of the fresh-water low-contrast clastics where the plot is the whole point.

**R-8 — SandiBumi will not test for the vendor's printed failure signature for the impossible
quadrant.** The printed *"negative laminar shale volume and infinitely large RSS"* does not reproduce:
a full grid sweep never produced both together. *Instead:* reject on the input quadrant test
(`SB-TBD-027`). *Why:* a guard written to the printed signature would not fire.

**R-9 — SandiBumi will not guard the branch flip on `RV_SH ≥ RV`.** That is the guard the evidence
first suggested and it is silent across the entire window where the failure actually occurs — on the
truth case the flip is at 7.4576 against an `RV` of 11. *Instead:* solve the quadratic and classify by
quadrant, which is immune by construction, and ship the level-dependent flip threshold as an
interoperability advisory (`SB-TBD-025`, `SB-TBD-026`).

**R-10 — SandiBumi will not adopt the vendor's constraint-flag coding scheme.** Six of its codes were
never recovered. *Instead:* adopt the two that were, define and document its own for the rest, and
**not advertise compatibility** (`SB-TBD-008`, O-2). *Why:* you cannot adopt a scheme you have not
read, and claiming a compatibility you cannot demonstrate is exactly the overclaim `01_PRODUCT.md` §6
prices at a deal.

**R-11 — SandiBumi will not switch the shale-distribution branch automatically per depth level.**
*Instead:* branch selection is a zone-level analyst decision (`SB-TBD-015`). *Why:* a model
discontinuity inside one geological unit is indistinguishable from geology on a log plot.

**R-12 — SandiBumi will not map either of Techlog's two ambiguous sand-fraction curves.**
*Instead:* import both unmapped with the ambiguity recorded (`SB-TBD-058`). *Why:* the two are
indistinguishable by value, and a wrong map silently substitutes a raw fraction for a modelled one.

**R-13 — SandiBumi will not present a clipped minimum saturation without its unclipped twin.**
*Instead:* emit both and flag the fired levels (`SB-TBD-048`). *Why:* `SB-CORE-002`.

**R-14 — SandiBumi will not enable laminar-referenced summation by default.** *Instead:* off, explicit
opt-in, with the vendor's own reserves warning carried into the deliverable (`SB-TBD-052`). *Why:* it
changes reserves, and the vendor that invented the scheme also ships it off.

**R-15 — SandiBumi will not reproduce the wrong Fahrenheit-to-Celsius conversion recorded in one
vendor printing of the ion-mobility correlation.** *Instead:* the temperature argument is unit-typed
and range-checked, so the defective form cannot be entered (`SB-TBD-057`). *Why:* implemented as
printed it yields 302–482 where a Celsius formation temperature belongs, with an accidental near-zero
error at one particular water-resistivity/temperature pair that would make the bug invisible in a
single-well check. **Seam:** the correlation is `12_saturation.md`'s; the typing that makes the defect
unshippable is this chapter's.

### 7.4 Independent-derivation requirements

`CONTRACT.md` §2.2 as amended 2026-08-07. What is prohibited is the derivation *path*, not the
capability. Four items in this domain — three specified, one deliberately not.

---

**C-3-1 — Anisotropic two-component root selection.**

*Class:* **C-3, opaque artifact.** The vendor's branch choice lives in a compiled binary. `lssa.info`
ships **no root-selection parameter**, no `.lls` source exists in the install, and the 57-page
technical reference prints a bare `±` (`OPEN-TB-8`). There is nothing to derive *from*, and inferring
the rule by running the binary and observing its outputs is precisely the prohibited path.

*Primary sources:* Klein (1995/1997) and the Mollison/Mezzatesta tensor-formulation literature; the
vendor's own **published** technical reference for the printed equations, which is documentation and
not internals; and first-principles algebra — the quadratic, its `a`/`b`/`c` coefficients, the
quadrant classification and the flip threshold, all derived independently and reproducibly in the
dossier's derivation register.

*Betters:* the incumbent's mechanism is **undocumented and uninspectable**, and per `D-TB-06` a fixed
sign silently returns the wrong root once the apparent sand conductivity crosses the horizontal shale
conductivity — a **96 % collapse in returned sand resistivity across a 0.06 ohm·m step in a picked
parameter**, with no flag. A quadrant classifier that retains both roots, chooses on a stated rule and
records that choice as a coded curve is correct where the fixed sign is not, and auditable where the
vendor's is not.

*Owning requirements:* `SB-TBD-024`, `SB-TBD-025`, `SB-TBD-026`, `SB-TBD-033`.

*Acquisition gap:* E-2. Specifying does not wait on it; the derivation stands on its own.

---

**C-3-2 — Behaviour at the tensor sand-resistivity bounds.**

*Class:* **C-3.** The bounds themselves are shipped manifest data and are first-class T1 evidence. The
**action** taken at them appears nowhere: zero hits across the entire technical reference
(`D-TB-09`). Determining it by feeding values through the binary is the prohibited path.

*Primary sources:* `SB-CORE-002`, `SB-CORE-003`, and the same manual's own statement that the sand
resistivity legitimately exceeds `RV` — which rules out a clamp on physical grounds independently of
any provenance argument.

*Betters:* the incumbent ships a bound whose behaviour its own reference never mentions, so a
cross-tool disagreement above it is unexplainable to the user of either tool. Preserving the computed
value, flagging the excursion and naming the vendor as the source of the bound converts an
unexplainable mismatch into a documented one.

*Owning requirements:* `SB-TBD-031`, `SB-TBD-032`.

*Acquisition gap:* a vendor release note or module document stating the action. None found.

---

**C-3-3 — Thin-bed resolution enhancement and log deconvolution.**

*Class:* **C-3.** Shipped inference artifacts and trained weight files are never consumed, in any
format.

*Primary sources:* the published binary-lithology deconvolution family and the published ML-based
deconvolution work — the latter already implemented natively in Jauhar's own `petro_deconv` engine,
from the paper, on this machine.

*Betters:* a vendor's shipped model is opaque, is not versioned against the data it saw, and cannot
carry provenance into a deliverable. A natively-trained model under `SB-CORE-010` states what it was
trained on — a claim no incumbent in this corpus makes.

*Owning requirement:* `SB-TBD-065`.

*Acquisition gap:* none. This one is buildable today.

---

**C-1-1 — Resistivity-independent saturation from acoustic response. NOT SPECIFIED, deliberately.**

*Class:* **C-1, patent-claimed.** **US 12,242,011 B2**, granted 2025-03-04, active. Full claim
analysis is at `docs/PRD_v2/REF_patent_US12242011.md` — read it there; it is not duplicated here.

*Why it lands in this chapter's field of view:* the patent's own abstract states the method addresses
*"a low resistivity low contrast shaly sand reservoir where previous methods would indicate the
reservoir was wet."* That is this domain's problem statement and Axis 3's exact positioning.

*Disposition:* **no requirement is allocated, no method is described, no formula is named, and no
`Betters:` line is written.** C-1 terms are unchanged by the 2026-08-07 amendment: independent
re-derivation does not clear a granted claim, and the analysis document records that the claims have
**not been cleared by counsel** and that the doctrine of equivalents is a real risk it cannot assess.
That a non-infringement position exists and is not weak is not a licence to specify. The analysis
document's own first recommendation is *"Do not implement anything yet."*

*Ownership:* unallocated — escalated as E-6. This chapter's view is `25_fluidsub-rockphysics.md` on
the mechanism; the call is Jauhar's.

*Bearing on the claim assessment:* §7.5.

---

**C-2 items in this domain: none.** The C-2 register entries — the cross-product screening harness,
the domain-transfer analyser and the textural-facies classifier — are multi-well and rock-typing
capabilities; none has a capability falling inside §1's boundary, and the textural one seams to
`15_sat-height-rocktyping.md`. Recorded so the empty list is visibly checked rather than overlooked.

### 7.5 Can Axis 3's claim be substantiated?

`05_STRATEGY.md` §18.3 makes this domain **Axis 3** and states the goal as *"the most complete
low-contrast-pay suite in existence, built for deltaic Indonesian reservoirs."* This chapter was
commissioned to be harder on SandiBumi here than anywhere else, on the grounds that an admitted gap
costs a feature and a discovered overclaim costs the deal. Accordingly:

**Today: no. The claim cannot be substantiated as written, and it should not be made in a sales
context until it can.**

What actually ships against it is: one Thomas-Stieber laminar/dispersed module whose core algebra is
correct and whose output labelling, clamping and endpoint defects are catalogued in §3.2; a crossplot
picker that draws a **different** Thomas-Stieber construction from the one the module computes; and
two genuinely differentiated excess-conductivity saturation models that run on **bulk** porosity — so
the laminar correction and the excess-conductivity correction, the two halves of the strategy's own
sentence, are not connected to each other. Of the twenty-seven capabilities this chapter identifies as
constituting the domain, **one is present and correct**. The entire resistivity-tensor half is absent,
as are relative dip, cross-method reconciliation, laminar net/pay summation, the recognition screen,
uncertainty, and all four plots.

**The two halves of the claim differ in strength, and conflating them is where the overclaim risk
sits.** "Most complete" is a **breadth** claim and it is currently false by a wide margin. But
`sw_rtc` and `sw_imts` are real, are shipped, and have **no equivalent in any of the three
incumbents** — the corpus found none. That is a defensible **depth** claim on a narrow front, and it
survives scrutiny where the breadth claim does not. The honest form of the sentence today is nearer to
*"the only tool that ships excess-conductivity low-contrast saturation models alongside a
Thomas-Stieber decomposition"* — narrower, true, and checkable by a buyer.

**Two caveats that cut in opposite directions.**

*Against the claim:* a granted, active US patent (C-1-1) covers a resistivity-independent saturation
route whose own abstract names low-resistivity low-contrast shaly sand. A competitor holding that
position is a real qualifier on "most complete in existence", it is unresolved, and the family is
still prosecuting a continuation. No version of the claim that survives should imply completeness
across routes SandiBumi has not cleared.

*For the claim:* that patent is also evidence the problem is commercially valuable enough to patent —
and, more usefully, **this chapter is not gap-limited**. Under `CONTRACT.md` §2.2 as amended, every
absent capability above is specifiable and is specified here: the tensor solve, root selection,
relative dip, the recognition screen, the summation, the uncertainty. Only three items in the entire
domain are Tier-C, two of them are buildable now under independent derivation, and the third is
deliberately unallocated. **There is no capability in this domain that SandiBumi is barred from
building** — which is the real finding, and a stronger position than the sentence in `05_STRATEGY.md`
currently claims.

**Recommendation.** Keep Axis 3; it is the right axis and the ceiling is genuinely high. Rewrite the
sentence to what ships, and let the roadmap carry the rest. An admitted gap costs a feature; a
discovered overclaim costs the deal — and this is the domain a serious buyer will test first.

---

## 8. Traceability — dossier disposition

Source: `docs/research_2026-08/cross_tool/thinbed-laminated.md`, 1,881 lines, third-pass revision of
2026-08-06. No `*_critique.md` file was read.

### 8.1 Row-count reconciliation

Every addressable item in the dossier is accounted for below. "Addressable" means a numbered item, a
table row in an inventory or adoption register, or an analysis subsection — not every sentence.

| Register | Where | Items | How dispositioned |
|---|---|---|---|
| Method inventory — IP | §1.1 | 16 | Individually, §8.2 |
| Method inventory — Geolog | §1.2 | 18 | Individually, §8.3 |
| Method inventory — Techlog | §1.3 | 19 | Individually, §8.4 |
| Method inventory — SandiBumi as-was | §1.4 | 4 | Individually, §8.5 |
| Parameter comparison rows | §2.5 | 44 | Block, §8.12 |
| Difference analyses | §3.1–§3.8 | 8 | Individually, §8.6 |
| Optimal-choice decisions | §4.1–§4.9 | 9 | Individually, §8.7 |
| Ledger and open-item dispositions | §4.10 | 19 | Individually, §8.8 |
| Proposed new ledger entries | §4.10, §6 | 9 | Individually, §8.9 |
| Adoption-spec modules | §5.1 | 10 | Individually, §8.11 |
| Adoption-spec parameter rows | §5.3 | 28 | Block, §8.12 |
| Plots to ship | §5.4 | 4 | Individually, §8.11 |
| Guards, flags and failure behaviour | §5.5 | 16 | Individually, §8.11 |
| Regression tests | §5.6 | 36 | Block, §8.12 |
| FINDINGS rules mapped | §5.7 | 9 | Individually, §8.11 |
| Open items | §6 | 15 | Individually, §8.10 |
| KB-update recommendations | §6 | 3 | Block, §8.12 |
| Derivations performed | §7 | 27 | Block, §8.12 |
| Critique dispositions | §8 | 33 | Block, §8.12 |

**Total addressable items: 327.** Individually dispositioned: **156**. Block-dispositioned with
exceptions enumerated: **171**. Nothing is dropped without a stated reason.

The `D-TB-nn` count (9) does not double-count §4.10: eight of the nine are proposed *inside* §4.10
rows and one (`D-TB-08`) is raised in §2.1. They are listed separately because the ledger entries are
the dossier's own deliverable to the defect register and each needs its own disposition here.

### 8.2 Method inventory — Interactive Petrophysics (§1.1, 16 items)

| Item | Subject | Disposition |
|---|---|---|
| IP-1 | `Sat Model = Normal/Laminated` gate | ADOPTED — `SB-TBD-044`, dispatch on the sand fraction |
| IP-2 | Parallel-conductivity solve for `Rsand` | ADOPTED-MODIFIED — `SB-TBD-035`; the pole is detected, not clamped through (R-3) |
| IP-3 | Tensor Vlam solve mode | ADOPTED — `SB-TBD-023`, `SB-TBD-024` |
| IP-4 | Tensor Rsh solve mode | ADOPTED — `SB-TBD-023`, `SB-TBD-025` |
| IP-5 | Tensor Rsh Mod — tensor `Vlam` overrides T-S typing | NOT ADOPTED as an override; ADOPTED as a *reconciliation input* — `SB-TBD-049`. An automatic override destroys the comparison that makes the pair diagnostic |
| IP-6 | Silent anisotropy-ratio reduction on failure | REFUSED — §7.3 R-2, `SB-TBD-037` |
| IP-7 | T-S attributed to Juhász, Vcl/Phie parameterization | ADOPTED as one of three named parameterizations — `SB-TBD-018` |
| IP-8 | Per-level automatic branch rule | REFUSED — §7.3 R-11, `SB-TBD-015` |
| IP-9 | Clay-model vs shale-model toggle | ADOPTED — `SB-TBD-018` carries the parameterization with the curve. Endpoint semantics seam to `10_clay-volume.md` |
| IP-10 | Rv/Rh butterfly crossplot | ADOPTED — `SB-TBD-054` |
| IP-11 | Interactive anisotropy-track picking | ADOPTED — `SB-TBD-060`, with `SB-TBD-012` provenance on the pick |
| IP-12 | Vendor QC rule: tensor and T-S `Vlam` should agree | ADOPTED and strengthened — `SB-TBD-049` makes it a classifier, not advice |
| IP-13 | "Do not use the Poupon equation" | ADOPTED and widened — `SB-TBD-047` blocks all three Poupon-family forms |
| IP-14 | Two shipped-but-undocumented crossplot types | EVIDENCE-ONLY — establishes both plots have a vendor precedent; the plots themselves are `SB-TBD-021`, `SB-TBD-054` |
| IP-15 | Laminated fluid substitution, elastic side | SEAM — `25_fluidsub-rockphysics.md`. §1 excludes elastic anisotropy |
| IP-15b | Scope correction: Backus averaging ships elsewhere in the tool | SEAM — `25_fluidsub-rockphysics.md` |

### 8.3 Method inventory — Geolog LSSA (§1.2, 18 items)

| Item | Subject | Disposition |
|---|---|---|
| GL-1 | 9-step deterministic workflow | ADOPTED as the module decomposition — §4.1–§4.7 requirement grouping follows it |
| GL-2 | T-S laminar-structural branch, Eq 81–85 | ADOPTED — `SB-TBD-014` |
| GL-3 | T-S laminar-dispersed branch, Eq 86–90 | ADOPTED — already shipped and verified equivalent (`SB-TBD-006` fixes the picker, not the algebra) |
| GL-4 | Isotropic-shale tensor closed form, Eq 93 | ADOPTED — `SB-TBD-023` |
| GL-5 | Anisotropic-shale closed form, Eq 96–100 | ADOPTED-MODIFIED — implemented as the quadratic, transcription labelled repaired (`SB-TBD-023`, §7.3 R-5) |
| GL-6 | `OPT_VSH_LAM` dispatch | ADOPTED — `SB-TBD-017` keeps the two estimates separately named |
| GL-7 | T-S constraints with `TSFLG` | ADOPTED-PARTIAL — `SB-TBD-008`; six of ten codes unrecovered, §7.3 R-10, O-2 |
| GL-8 | `STCT`/`LMCT`/`DPCT` analyst cutoffs | ADOPTED with their action class made explicit — `SB-TBD-016` |
| GL-9 | `PORFIL`/`PMAXNU` below-left diagnostic | ADOPTED — `SB-TBD-010` |
| GL-10 | Sand-referenced `OPT_SW` dispatch | ADOPTED — `SB-TBD-044`, `SB-TBD-045`. Equations seam to `12_saturation.md` |
| GL-11 | `OPT_QV` options | SEAM — `12_saturation.md`; the unit-pairing guard is `SB-TBD-056` |
| GL-12 | 10 permeability options | SEAM — `13_permeability.md`; the sand-fraction convention is `SB-TBD-064` |
| GL-13 | No-Vsh-cutoff net/pay scheme | ADOPTED — `SB-TBD-051`, `SB-TBD-052`, `SB-TBD-053` |
| GL-14 | `BAD_DATA` suppression with a counted summary | ADOPTED — `SB-TBD-019` |
| GL-14b | `SWE_MIN` minimum-Sw floor, default 0.08 | ADOPTED-MODIFIED — `SB-TBD-048` emits the unclipped twin (§7.3 R-13). The 0.08 default is a cited T1 value in §5 |
| GL-15 | Shale resistivity accepted as curves | ADOPTED — carried on the tensor module inputs (`SB-TBD-023` input contract) |
| GL-16 | Explicit quadrant validity rules | ADOPTED and made the root-selection mechanism — `SB-TBD-025`, `SB-TBD-027` |
| GL-17 | `RT_SS_MAX`/`RT_SS_MIN` shipped bounds, T1-only | ADOPTED as evidence, REFUSED as an action — `SB-TBD-031`, §7.3 R-4, §7.4 C-3-2 |

### 8.4 Method inventory — Techlog TBA (§1.3, 19 items)

| Item | Subject | Disposition |
|---|---|---|
| TL-1 | T-S in the original 1975 GR-index parameterization | ADOPTED as a *named third parameterization* — `SB-TBD-018`. Also the antecedent of the picker/module divergence (`SB-TBD-006`) |
| TL-2 | Hagiwara-Conser resistivity solution | PARTIAL — the macroscopic dip half is `SB-TBD-038`; the microscopic half is O-4 |
| TL-3 | Parallel-conductivity solution | ADOPTED — `SB-TBD-035` |
| TL-4 | Sw model list | SEAM — `12_saturation.md` |
| TL-4b | `Relative bed dip` parameter on the T-S module itself | ADOPTED — `SB-TBD-038`, `SB-TBD-039`, `SB-TBD-041` |
| TL-4c | T-S `Qv` model set differs from LowReP's | EVIDENCE-ONLY — supports `SB-TBD-018`'s premise that the name does not fix the method. Seam: `12_saturation.md` |
| TL-5 | Minimum-Sw guard for `Rsand ≈ Rshale` | ADOPTED-MODIFIED — `SB-TBD-048`, `SB-TBD-029` |
| TL-6 | Geostatistical porosity-limit model | NOT ADOPTED for v1 — a second sand-fraction porosity model with no local validation case. Recorded, not specified |
| TL-7 | Freeman grain-size permeability, Krumbein units | SEAM — `13_permeability.md`; the unit-typing rule is `SB-TBD-055` |
| TL-8 | Conser clay grain fraction | SEAM — `10_clay-volume.md` |
| TL-9 | Iterative Sxo correction with convergence status | ADOPTED as a pattern — `SB-TBD-019` (a non-converged result is flagged, never returned bare) |
| TL-10 | LowReP 6-volume forward-model inversion | NOT ADOPTED for v1 — an optimizer route; §1 scopes this chapter to closed forms. Recorded as the architectural alternative in §2 |
| TL-11 | LowReP T-S constraint set | ADOPTED in spirit — `SB-TBD-008` |
| TL-12 | Hagiwara macroscopic anisotropy with effective dip | ADOPTED — `SB-TBD-038`, `SB-TBD-040` |
| TL-13 | Invasion geometric factor on nuclear responses | SEAM — `19_multi-mineral.md` |
| TL-14 | Monte-Carlo sensitivity with Tornado output | ADOPTED-MODIFIED — `SB-TBD-059`, sourced from Jauhar's own engine rather than from the vendor (§7.4 rationale for independent sourcing) |
| TL-15 | Modified Klein plot after Minh et al. 2008 | ADOPTED — `SB-TBD-054`, with the printed multiplication refused (§7.3 R-7) |
| TL-16 | Out-of-model splice logic | NOT ADOPTED — splicing a different model into a depth interval without a curve saying so is the failure `SB-CORE-002` exists to prevent. Recorded and declined |
| TL-17 | `Qv = C·PHITsd^d` | SEAM — `12_saturation.md` |

### 8.5 Method inventory — SandiBumi as-was (§1.4, 4 items)

| Item | Subject | Disposition |
|---|---|---|
| SB-1 | The single `thin_bed_ts` module | Confirmed at source and re-verified: `modules.rs:371`, `:457`, `:2432`, `:2457`, `:4862` |
| SB-2 | Inputs, parameters and defaults | Confirmed; both endpoint defaults are uncited and are WITHDRAWN by `SB-TBD-066` |
| SB-3 | Laminar-dispersed only, no tensor, no dip, no constraints | Confirmed; drives the 19 ABSENT rows in §3 |
| SB-4 | `lrlc.rs` `sw_rtc` / `sw_imts` as separate modules | Confirmed at `lrlc.rs:73`, `:118`, `:179`, `:225`; the bulk-porosity defect at `:123`/`:228` is `SB-TBD-046` |

### 8.6 Difference analyses (§3.1–§3.8, 8 items)

| § | Subject | Disposition |
|---|---|---|
| 3.1 | The shale-resistivity pick dominates; IP's own worked example is arithmetically wrong | ADOPTED — `SB-TBD-029`, `SB-TBD-012`; the arithmetic error becomes `D-TB-01` and a passing-plus-recording test |
| 3.2 | The parallel-only model has a pole and IP clamps through it | ADOPTED — `SB-TBD-035`, `SB-TBD-036`, §7.3 R-3 |
| 3.3 | With `Rv`, the horizontal shale pick still dominates — quantified | ADOPTED — the ∓5.2 %/+5.3 % and ∓0.3 % figures are the sourced content of `SB-TBD-034`'s validity table, and the reason E-1 exists |
| 3.4 | The two-root problem | ADOPTED — `SB-TBD-024`, `SB-TBD-025`, `SB-TBD-027`, `SB-TBD-033`; §7.4 C-3-1 |
| 3.5 | Relative dip: Techlog has it, IP and Geolog do not | ADOPTED — `SB-TBD-038`–`SB-TBD-041` |
| 3.6 | Constraint handling and out-of-model data: Geolog is far ahead | ADOPTED — `SB-TBD-008`, `SB-TBD-009`, `SB-TBD-010`, `SB-TBD-019`; the out-of-model splice is declined (TL-16) |
| 3.7 | A Geolog unit conversion that is wrong as printed | REFUSED as printed — §7.3 R-15, `SB-TBD-057`; escalated as `D-TB-03` / OPEN-TB-7. The correlation itself seams to `12_saturation.md` |
| 3.8 | Closed-form vs optimizer architecture | EVIDENCE-ONLY — §1 scopes this chapter to closed forms; the optimizer route is recorded, not specified |

### 8.7 Optimal-choice decisions (§4.1–§4.9, 9 items)

| § | Dossier's recommendation | Disposition |
|---|---|---|
| 4.1 | Adopt Geolog Eq 81–90 verbatim | ADOPTED — with the qualification that "verbatim" cannot apply to Eq 82, which is repaired (`D-TB-08`) |
| 4.2 | Adopt the anisotropic closed form with the corrected sign | ADOPTED-MODIFIED — the canonical form is the **quadratic**, not the closed form; a sign correction alone still inherits the branch flip (`SB-TBD-023`, §7.3 R-5) |
| 4.3 | Implement the quadrant test as an automatic classifier with a manual override | ADOPTED — `SB-TBD-025`, `SB-TBD-033` |
| 4.4 | Adopt Moran-Gianzero; make it mandatory-explicit | ADOPTED — `SB-TBD-038`, `SB-TBD-039` |
| 4.5 | Geolog's dispatch plus the LRLC models as options | ADOPTED — `SB-TBD-044`, `SB-TBD-046`, `SB-TBD-063` |
| 4.6 | No defaults for resistivity endpoints; cite the porosity ones | ADOPTED and **widened** — the porosity endpoints have no citable source either, so `SB-TBD-066` withdraws both rather than citing them. This is the one place this chapter is stricter than the dossier |
| 4.7 | Adopt the no-Vsh-cutoff net/pay scheme | ADOPTED — `SB-TBD-051`, `SB-TBD-052`, `SB-TBD-053`, `SB-TBD-064` |
| 4.8 | Reconciliation as primary QC with Geolog's diagnostic interpretation | ADOPTED — `SB-TBD-049`, `SB-TBD-050`; the KB note's wrong-signed inference is escalated as E-4 |
| 4.9 | Ship the Monte-Carlo route from Jauhar's own engine | ADOPTED — `SB-TBD-059` |

### 8.8 Ledger and open-item dispositions (§4.10, 19 rows)

| Item | Disposition |
|---|---|
| `D-L-08` — crossplot type list omits two shipped types | CARRIED as evidence for `SB-TBD-021` / `SB-TBD-054`; the ledger item itself belongs to the ingest register, not here |
| `OPEN-L-13` — same, owner unknown | CLOSED upstream by R-5; no action here |
| `R-5` — the two crossplots, owner found | CARRIED as evidence; no action here |
| `D-03` — `Stieber` vs `Steiber` spelling | ADOPTED — `SB-TBD-020`, import-alias both spellings |
| `PhiFlag = 15` — silent anisotropy-ratio reduction | REFUSED — §7.3 R-2, `SB-TBD-037` |
| `Rt Lam Sand`/`Rxo Lam Sand` columns cut off | OPEN — O-6 |
| IP's inconsistent thin-bed halves (tensor resistivity, scalar elastic) | SEAM — `25_fluidsub-rockphysics.md` |
| `D-TB-01` proposed — IP's sensitivity example fails its own arithmetic | See §8.9 |
| `D-TB-02` proposed — Eq 98 sign conflict with Eq 6 | See §8.9 |
| `D-TB-03` proposed — `CFMTMP` labelled °C | See §8.9 |
| `D-TB-07` proposed — IP −1.28 vs Geolog −1.25 in the same named correlation | See §8.9 |
| `D-TB-09` proposed — `RT_SS_MAX`/`MIN` documented nowhere but the manifest | See §8.9 |
| `D-TB-08` proposed — Eq 82 prints an undefined variable | See §8.9 |
| `TSFLG` codes 1…8 raster-only | See §8.9 (`D-TB` scope) and O-2 |
| `D-TB-04` proposed — Eq 92 prints `CV =` with a resistivity RHS | See §8.9 |
| `D-TB-06` proposed — the `±` does not track a physical branch | See §8.9 |
| `D-TB-05` proposed — Eq 98's ½ on the first term only, and the Klein-plot multiplication | See §8.9 |
| Techlog prints the Klein-plot relation with a multiplication | REFUSED — §7.3 R-7, `SB-TBD-054` |
| SandiBumi `PHIE_LAM` is actually `PHIT_SS` | ADOPTED as a P0 defect — `SB-TBD-009` |

### 8.9 Proposed new ledger entries (`D-TB-01`…`D-TB-09`, 9 items)

These are the dossier's proposed additions to the defect register. This chapter does not own that
register and does not create entries in it; each is recorded here with the requirement that carries it.

| Entry | Subject | Carried by |
|---|---|---|
| `D-TB-01` | IP's shale-resistivity sensitivity example is internally inconsistent (5.75 should be 4.13) | `SB-TBD-029`; the corrected value is the expected result of the corresponding test |
| `D-TB-02` | Geolog Eq 98 conflicts with Eq 6 on the sign; resolved by derivation to `(1 + ΔC)` | `SB-TBD-023` |
| `D-TB-03` | `CFMTMP = 1.8·(FMT − 32)` labelled Celsius | `SB-TBD-057`; §7.3 R-15; escalation seam to `12_saturation.md` |
| `D-TB-04` | Eq 92 prints a conductivity equated to a resistivity expression | `SB-TBD-022`; §7.3 R-6 |
| `D-TB-05` | Eq 98's ½ applied to the first term only; and the Klein-plot multiplication-for-division | `SB-TBD-023`, `SB-TBD-054`; §7.3 R-5, R-7 |
| `D-TB-06` | The fixed `±` silently returns the wrong root past the branch-flip threshold | `SB-TBD-025`, `SB-TBD-026`, `SB-TBD-033`; §7.4 C-3-1 |
| `D-TB-07` | The same named correlation prints −1.28 in one tool and −1.25 in another | **SEAM — `12_saturation.md` owns the correlation.** Recorded here because it was found here; no requirement allocated |
| `D-TB-08` | Eq 82 prints an undefined variable; transcribed repaired, not silently normalized | `SB-TBD-014` (the structural branch), with the repair labelled per `SB-TBD-023`'s rule |
| `D-TB-09` | The tensor sand-resistivity bounds are manifest-only; the action is undocumented | `SB-TBD-031`; §7.3 R-4; §7.4 C-3-2 |

### 8.10 Open items (`OPEN-TB-1`…`OPEN-TB-15`, 15 items)

| Item | Disposition in this chapter |
|---|---|
| `OPEN-TB-1` | RESOLVED BY WITHDRAWAL — `SB-TBD-066` removes the uncited default rather than seeking a citation for it; replacement picks escalated as E-5 |
| `OPEN-TB-2` | OPEN — O-3. Not blocking |
| `OPEN-TB-3` | OPEN — O-4. Not blocking; the macroscopic half ships as `SB-TBD-038` |
| `OPEN-TB-4` | OPEN — O-5. Seam: `12_saturation.md` |
| `OPEN-TB-5` | OPEN — O-6. Not blocking |
| `OPEN-TB-6` | ESCALATED — E-3. **Blocking for `SB-TBD-042`** |
| `OPEN-TB-7` | REFUSED as printed — §7.3 R-15, `SB-TBD-057`. The residual cross-tool constant conflict is `D-TB-07`, seamed to `12_saturation.md` |
| `OPEN-TB-8` | ADDRESSED BY INDEPENDENT DERIVATION — §7.4 C-3-1. The vendor's mechanism stays unknown; SandiBumi's is specified and stated |
| `OPEN-TB-9` | ESCALATED — E-2, named-paper acquisition |
| `OPEN-TB-10` | CARRIED as a transcription hazard — the reversed vertical/horizontal averaging in the printed source is one reason `SB-TBD-023` requires every repair to be labelled and every mixing law round-trip tested (`SB-TBD-T25`). No permeability-anisotropy requirement is allocated; that is `13_permeability.md`'s |
| `OPEN-TB-11` | ADDRESSED — `SB-TBD-008` implements the constraint behaviour from Geolog's complete printing rather than from Techlog's partially-captured page. The second Techlog constraint remains uncaptured and is not relied on |
| `OPEN-TB-12` | ADDRESSED — `SB-TBD-059`; also O-7, which records that it is not blocking for v1 but is blocking before these results book a reserve |
| `OPEN-TB-13` | OPEN — O-2; scope-limited by `SB-TBD-008` and §7.3 R-10 |
| `OPEN-TB-14` | OPEN — O-8; made unshippable-if-wrong by `SB-TBD-056` |
| `OPEN-TB-15` | ADDRESSED BY REFUSAL — §7.3 R-12, `SB-TBD-058`; also O-9 |

### 8.11 Adoption-spec lines (§5.1, §5.4, §5.5, §5.7 — 39 items)

**Module decomposition (§5.1, 10 modules).** All ten are adopted as the shape of the suite. This
chapter specifies capabilities rather than module boundaries, so each maps to a requirement group
rather than to a named binary: recognition screen → `SB-TBD-001`–`SB-TBD-005`; Thomas-Stieber →
`SB-TBD-006`–`SB-TBD-021`; dip → `SB-TBD-038`–`SB-TBD-043`; tensor → `SB-TBD-022`–`SB-TBD-034`;
parallel → `SB-TBD-035`–`SB-TBD-037`; reconcile → `SB-TBD-049`, `SB-TBD-050`; sand-referenced Sw →
`SB-TBD-044`–`SB-TBD-048`, `SB-TBD-063`; summary → `SB-TBD-051`–`SB-TBD-053`, `SB-TBD-064`;
interval Monte-Carlo → `SB-TBD-059`; clay-mineral correction → `SB-TBD-061`.

**Plots (§5.4, 4 plots).** Thomas-Stieber triangle → `SB-TBD-021`. Klein / butterfly →
`SB-TBD-054`. Anisotropy track → `SB-TBD-060`. Reconciliation track → `SB-TBD-050`. All four ADOPTED;
all four currently ABSENT or PARTIAL.

**Guards and failure behaviour (§5.5, 16 rows).** All sixteen ADOPTED, distributed as: outside the
triangle → `SB-TBD-008`; below-left → `SB-TBD-010`; no shale present → `SB-TBD-008`; flag-coding
scope → `SB-TBD-008` + §7.3 R-10; pole margin → `SB-TBD-035` (with the 0.9 threshold flagged as
unsourced, O-1); tensor bounds → `SB-TBD-031`; impossible quadrant → `SB-TBD-027`; no admissible
anisotropy root → `SB-TBD-037`; `RV_SH < RH_SH` → `SB-TBD-030`; `RV_SH ≥ RV` singularity →
`SB-TBD-028`; branch-flip threshold → `SB-TBD-026`; root selection → `SB-TBD-025`, `SB-TBD-033`;
shale pick within 0.1 ohm·m → `SB-TBD-029`; bad-hole suppression → `SB-TBD-019`; dip not supplied →
`SB-TBD-039`; naming → `SB-TBD-009`, `SB-TBD-017`, `SB-TBD-062`.

**FINDINGS rules mapped (§5.7, 9 rules).** All nine ADOPTED. Rule 1 (no raster-only truth) is the
reason §2 carries an evidence tier on every finding and the reason O-2 refuses to guess. Rule 3
(unit-typed quantities) → `SB-TBD-041`, `SB-TBD-055`, `SB-TBD-056`, `SB-TBD-057`. Rule 4
(unit-invariant statistics) → `SB-TBD-051`. Rule 5 (one flag convention) → `SB-TBD-019`. Rule 8 (no
ambiguous symbols) → `SB-TBD-009`, `SB-TBD-017`, `SB-TBD-062`. Rule 9 (defaults cited or absent) →
`SB-TBD-066` and the 22 ABSENT rows in §5. Rule 11 (worked examples reproduce) → `SB-TBD-029` and its
test. Rule 13 (state the reference convention) → `SB-TBD-041`. Rule 14 (silent failures are bugs) →
`SB-TBD-007` and every refusal in §7.3.

### 8.12 Block-dispositioned registers, with exceptions enumerated

**Parameter comparison rows (§2.5, 44 rows) → §5 of this chapter.** Every row was carried into §5's
50-row table, re-checked against its cited source, and given a tier. **Three exceptions:** (a) the two
SandiBumi porosity endpoints are WITHDRAWN rather than carried, because neither is citable
(`SB-TBD-066`); (b) the `Qv`, `B`, `m`, `n` and `Rw` rows are cited but marked SEAM — `12_saturation.md`
owns them and §5 does not re-specify them; (c) `SWE_MIN` = 0.08 is carried as a cited T1 default but
its *action* is modified by `SB-TBD-048`. §5 has more rows than §2.5 because the dip, validity-table
and flag-scheme parameters have no §2.5 antecedent.

**Adoption-spec parameter rows (§5.3, 28 rows) → §5 of this chapter.** Carried in full. The dossier's
"no default, analyst-set" convention for the five endpoint parameters is preserved exactly; this
chapter adds two more ABSENT rows by withdrawing the porosity endpoints.

**Regression tests (§5.6, 36 tests) → §6 of this chapter.** All 36 are represented among the 66
`SB-TBD-Tnn` tests. §6 is larger because it adds tests for the requirements in §8.13 and one test per
refusal in §7.3, so that a refusal is enforced by code rather than asserted in prose. **One
exception:** the dossier's test that pins the vendor flag-coding scheme is narrowed to the two
recovered codes only, per §7.3 R-10.

**KB-update recommendations (§6, 3 bullets).** NOT PERFORMED — outside this chapter's write scope,
which is this file only. The first is ESCALATED as E-4 because it is a *retraction* of a wrong-signed
inference and is more urgent than the two additions. The other two are recorded for whoever next
touches those notes.

**Derivations (§7, `D-1`…`D-27`).** EVIDENCE, not allocatable items. Each is a reproducible check
that establishes a finding already dispositioned above. Fifteen were re-derived independently while
writing this chapter — the Eq 86 equivalence, the quadratic and its coefficients, the branch-flip
threshold and its quadrant scoping, the Eq 89 ≡ Eq 122 identity, and the sensitivity figures — and
all fifteen reproduced. **One correction arising:** the dossier's `D-1` states SandiBumi's
interpolation is identical to Eq 86, which is true of the algebra and **not** true of what the
crossplot picker draws; that gap is `SB-TBD-006` and is this chapter's, not the dossier's.

**Critique dispositions (§8, `BL-1`…`BL-3`, `MJ-1`…`MJ-12`, `m-1`…`m-18`).** EVIDENCE-QUALITY
RECORDS, not domain findings. They record what the third pass fixed and are the reason this chapter
treats the pass-3 figures as current where they supersede pass-2 ones — notably the ∓5.2 %/+5.3 %
sensitivity replacing ±5.3 %, and the corrected citation coordinates. No `*_critique.md` file was read;
these are the dossier's own record of its revisions. Two are load-bearing here: `BL-3`, which
establishes the cross-tool constant conflict now seamed as `D-TB-07`, and `m-1`, which is why every
line pointer in this chapter was re-verified at source rather than copied.

### 8.13 Surplus — requirements with no dossier antecedent

Nine requirements originate in this chapter's own verification of the shipped source and of
`04_CORE_REQUIREMENTS.md`, not in the dossier. They are listed so the delta is auditable.

| Requirement | Origin |
|---|---|
| `SB-TBD-003` | The route table must **disclose the open defects on a route it recommends**. The dossier catalogues the defects and the routes separately; connecting them so the user sees the caveat at the point of choice is `SB-CORE-013` applied here |
| `SB-TBD-006` | Verified this session: `crossplotPanel.ts:301-303` draws the **1975 dispersed limb** — down to a minimum, then back up to the shale point — while `modules.rs:2475` computes the Eq 86 line that never turns. Same name, two constructions, one of them the parameter picker for the other. `SB-CORE-006` |
| `SB-TBD-007` | The dossier names two clamps. Verification found a **third**, at `modules.rs:2486`, clamping the emitted sand porosity to the clean-sand endpoint |
| `SB-TBD-011` | Making the Eq 89 ≡ Eq 122 identity a **shipped property test** rather than a one-off derivation. The identity is the dossier's; the test is not |
| `SB-TBD-012` | An interactive pick carries its provenance. Straight from `SB-CORE-010`'s "every parameter value **and that value's source string**"; the dossier does not raise it |
| `SB-TBD-013` | Verified this session: the picker clamps the dragged total porosity to 0.5 (`crossplotPanel.ts:2119`) while the module's own declared range ends at 0.45 (`modules.rs:2445`). Two admissible ranges for one parameter — an `SB-CORE-007` instance the dossier does not contain |
| `SB-TBD-032` | The parallel-route saturation bound and the tensor-route resistivity bounds are different objects with different justifications. The dossier treats bounds generically; conflating them would apply a parallel-model artefact to a tensor result |
| `SB-TBD-062` | The `_SS` sand-fraction suffix collides with the shipped `_SSC`/`_SSPW` model suffixes (`ssc.rs:118-119`). Discovered while verifying that `PHIT_SS` was genuinely absent from the source — the grep for it matched the model-suffixed curves instead |
| `SB-TBD-063` | Renormalizing the sand-referenced saturation back to bulk **with bulk hydrocarbon volume conserved as the check**. The dossier specifies the dispatch; the conservation identity is the part that makes it testable |

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
