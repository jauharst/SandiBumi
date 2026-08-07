# 14. Cutoffs, summation and Monte Carlo — requirements

**Dossier.** `docs/research_2026-08/cross_tool/cutoffs-summation-mc.md` — 3,627 lines, read in
full including the correction log (C-1…C-23), the ledger dispositions (§4.1), the corrections
forced on prior records (§4.2), the eighteen gaps and escalations (§6) and the authoritative
`## Critique disposition`. The disposition is treated as authoritative over any body text it
corrects, per CONTRACT §4 rule 2. `cutoffs-summation-mc_critique.md` was **not** read, per
CONTRACT §4 rule 3.

**Evidence tiers held.** The dossier refines CONTRACT §1.2 T1 into **T1a** (vendor *executable*
source read directly — Techlog `.py`, Geolog `.lls`) and **T1b** (shipped program *data* files the
executable reads — `MonteCarloDefaults.par`, `Cutoff.hlp`, `*.paysum`, `*.info`). Both are T1
under the contract and the refinement is carried because it is load-bearing: several findings turn
on whether a number is in code or in a data file the code reads. **T2** (vendor manual text),
**T3** (vendor raster / help page read visually) and **T4** (literature and project records) are
used as the contract defines them. The dossier's **P** label (delivered-project precedent from
project-kb decision records) is carried as a labelled subclass of **T4**.

**Author date.** 2026-08-07.

**Requirements.** 61 (`SB-CUT-001` … `SB-CUT-061`). **P0: 9.**

**Parameters.** 44 rows in §5. Of those, **8 ship `ABSENT — ships with no default`** and 11 are
`NON-ADOPTABLE — cited for verification`.

**Acceptance tests.** 44 (`SB-CUT-T01` … `SB-CUT-T39`, with the dossier's `b`/`c` suffixes kept).

**Cross-cutting requirements this chapter carries.** `SB-CORE-002` (a degraded or failed result is
never presented as a clean one — **P0**, and this chapter owns the largest share of it: the Monte
Carlo path, the pay summary and the report's pay section) and `SB-CORE-004` (no parameter ships
without a source — **P0**, whose named worked example is this domain's cut-off defaults). Both are
defined in `04_CORE_REQUIREMENTS.md` §15.1; no new `SB-CORE` identifiers are minted here.

---

## 1. Scope and boundary

This chapter owns everything between a finished interpretation and a reported number: the cut-off
comparison that turns continuous curves into net flags, the discretisation rule that turns flagged
samples into footage, the averaging that turns footage into zone statistics, the volumetric
identities that tie those statistics together, and the Monte Carlo machinery that puts an
uncertainty band on all of it. Concretely: cut-off records and their operators, flag tiers,
lumping and bed amalgamation, the Gross / Net / NotNet / Unknown accounting, arithmetic /
geometric / harmonic / power-mean averaging, φ-weighted saturation, `HCPV`, net-to-gross, the
cut-off sensitivity sweep, and the sampler, correlation, percentile and tornado machinery of a
volumetric uncertainty run.

**Seam — `SHR` (saturation-height and rock typing).** Rock typing supplies the flow-unit basis
that several cut-off schemes are defined *against*: a permeability cut-off derived from an
economic mobility ratio `(k/µ)_c`, and a porosity cut-off read from a φ-vs-`k/µ` crossplot, are
both rock-type-scoped quantities, not global constants. This chapter owns the *machinery* — a
cut-off may be a curve, cut-offs may be scoped per zone, per flag tier and per rock type — and
`SHR` owns the *derivation* of the value. The dossier's escalation 8 (Worthington & Cosentino
2005 for the `(φc, kc)` pair; Qassamipour et al. 2020 for the histogram-maximisation method) is
guidance, not machinery, and is escalated to `SHR` in §7. Ledger item **D-05** (the 0.433 psi/ft
Pc↔height factor) belongs wholly to `SHR`: it appears **zero** times across all four cut-off /
summation / MC page texts, and the summation module MUST NOT convert an `HCPV` thickness to a
volume using a fluid gradient.

**Seam — `PLT` (plotting, display, interactivity).** The cut-off *sensitivity plot* — the curve of
net, `HCPV` or N:G against a swept cut-off, and Geolog's live crossplot editing where dragging a
cut-off line re-runs the summation under `autorun` — is a `PLT` surface driven by a `CUT` engine.
This chapter owns the sweep computation, the permutation grid, the inverse solve and the tornado
*values*; `PLT` owns their rendering, the interaction model, and the histogram/CDF display of the
Monte Carlo output. The tornado-units requirement (`SB-CUT-051`) is stated here because it is a
computation contract, not a rendering choice: a bar measured as a percentage of the Monte Carlo
range moves with iteration count while the underlying sensitivity does not.

**Seam — `DBM` (database and project data model).** Several requirements here are storage
contracts: per-iteration joint records (`SB-CUT-044`), the seed and discretisation model on the
result record (`SB-CUT-002`, `SB-CUT-034`), the reference frame as part of a result's *identity*
(`SB-CUT-012`), and block-scoped parameter addressing on import (`SB-CUT-060`). They are specified
here because the obligation arises from this domain's evidence; `DBM` owns the schema realisation.

**Seam — `DIO` (data import, export, formats).** Requirements `SB-CUT-032` (shift type stored, a
`Rec` shift never coerced), `SB-CUT-060` (block-scoped ordinals) and `SB-CUT-061` (precision
validated against field width) constrain an IP import path that `DIO` builds.

**Not in scope.** The derivation of φ, Vsh, Sw or k themselves (`POR`, `CLY`, `SAT`, `SHR`); the
propagation of a single parameter's uncertainty through a *deterministic* equation, which is the
same machinery but is specified by the owning method chapter; and volumetrics above the well —
areal extent, GRV, recovery factor — which no chapter in this corpus owns and which this chapter
deliberately stops short of.

---

## 2. What the incumbents do — the requirement-bearing findings

Twenty-five findings, each generating at least one requirement. Findings that inform without
obliging are dispositioned `EVIDENCE-ONLY` in §8 and are not repeated here.

### F-1 — Four shipped cut-off default sets, no two identical, two of them from one vendor

**Tier T1b + T2, all three tools, plus T4/P delivered precedent.** Shipped values:

| Source | Vsh | φ | Sw | k |
|---|---|---|---|---|
| IP 2025.3 manual, Reports 1–4 configured, Report 5 unconfigured | 0.5 | 0.1 | 0.5 | — |
| Techlog `SummariesMonteCarlo.py` (`VSH_max` / `POR_min` / `SW_max`) | 0.5 | 0.15 | 0.85 | — |
| Geolog `default_*.paysum` | 0.3 | 0.08 (PHIE) | 0.5 (SWE) | — |
| Geolog `determin_mc.info` | 0.5 | 0.08 (PHIE), 0.08 (PHIT) | 0.5 (SWE), 0.5 (SWT) | 0 |
| Geolog `tp_pay_sensitivity.info` (permissive) | 1 | 0 | 1 | 0.01 mD |

**Consequence, quantified.** On one shipped set the porosity cut-off is 0.08 v/v and on another
0.15 v/v — a 1.875× ratio on the single parameter that most directly sets net footage. Geolog
disagrees with **itself** on Vsh: `vshale-only_*.paysum` ships 0.3 while `determin_mc.info` ships
0.5. Against that, delivered work on this machine spans **Vsh 0.20–0.85, PHIE 0.05–0.27, Sw
0.50–0.90**, and a *single* project record spans **Vsh 0.55–0.85 across intervals of one area**.
No shipped default is inside the delivered range often enough to be a defensible starting point,
and the intra-area spread proves the quantity is not even constant within one field.

**Obligation.** `SB-CUT-016`, `SB-CUT-017`, `SB-CUT-018`, `SB-CUT-022`. The dossier's §3.7 verdict
— *ship no cut-off value at all* — is adopted, and CONTRACT §2's `ABSENT — ships with no default`
is the mechanism. This is the single most important requirement in the chapter.

### F-2 — IP's geometric average is unit-dependent, and therefore wrong

**Tier T3 (IP equation raster `embim163.png`, re-read at synthesis) vs T1a (Techlog `average()`
averageType 1 + `image2081.gif`) and T1b (Geolog power-mean family).** IP's form is
`(C₁·C₂·…·Cₙ)^(1/Σhᵢ)` — the product of sample values raised to the reciprocal of *total
thickness*. The correct weight-normalised form is `exp(Σhᵢ·lnCᵢ / Σhᵢ)`.

**Consequence, quantified.** The dossier's worked case: one identical permeability log returns
**10 mD at a 1.0 ft step, 100 mD at 0.5 ft, and 3.6 × 10⁶ mD at 0.1524 m** under IP's exponent.
Changing the *units of the depth index* changes a reported permeability by five orders of
magnitude. Two independent correct implementations settle it (ledger D-5.2, RESOLVED).

**Obligation.** `SB-CUT-007` (correct form, mandatory) and `SB-CUT-T04` (unit-invariance
regression, which must also assert the result differs from IP's form on a 0.5-step log, so the
divergence is documented rather than discovered by a client).

### F-3 — Depth discretisation diverges by at most half a step per zone contact, with opposite signs

**Tier T1a (Techlog `computeGross()`, hand-traced line by line) + T2 (IP `cutoffsandsummation.htm`
+ raster `_candsclip0030.png`) + T1b (Geolog `tp_paysummary.info` L63 `FRAME_REP`).** Three
interval-ownership models exist — CENTRED (sample owns half the step either side), TOPS (sample
owns the forward interval), BOTTOMS — and all three are zone-clipped by
`hᵢ = max(0, min(Z_bot, bᵢ) − max(Z_top, aᵢ))`. The invariant `Σhᵢ = Z_bot − Z_top` holds
**exactly** in all three, so the models differ only in *apportionment*, never in total.

**Consequence, quantified.** On IP's own published fixture — zone 100.0 → 104.0 ft, step 0.5,
flags `0,0,1,1,1,1,1,1,1` — **IP reports Net = 3.25 ft and Techlog reports Net = 3.0 ft**, with
Gross = 4.0 ft exactly (set, not summed) and Unknown = 0.0 in both. The 0.25 ft gap is exactly one
half-step at one zone contact. The envelope is bounded at ½ step per zone-boundary contact and the
two contacts carry **opposite** signs, so the errors partly cancel over a zone and cannot
accumulate over many zones the way a per-bed error would.

The earlier framing that this was a thin-bed catastrophe is **withdrawn by the dossier itself**
(critique blocker B-1). The real thin-bed hazard is elsewhere — see F-4.

**Obligation.** `SB-CUT-001` (explicit model, default CENTRED by four independent vendor votes,
implemented by zone clipping), `SB-CUT-003`, and tests `SB-CUT-T01`/`T02`/`T02b`/`T03`/`T03b`/`T03c`.

### F-4 — One product ships two different definitions of "Net" and labels neither

**Tier T2, IP alone.** IP's Cut-off & Summation report computes net by the half-weight
(CENTRED) rule. IP's **Curve Statistics** report computes `Net = count × step`. These are
different discretisation models, in the same product, under the same column heading, with no
statement anywhere that they differ.

**The thin-bed trap lives here, not in F-3.** The *reported bed thickness* is defined as
`Bottom − Top + step` in IP and `Depth[Bottom] − Depth[Top]` in Techlog. For a bed one sample
thick, Techlog reports **0.0** and IP reports one full step. A Detail Interval Breakdown listing
forty laminae is the difference between "one clean 30 ft sand" and "forty 0.75 ft laminae", and it
is the number that decides whether a net figure is usable for thin-bed work at all.

**Consequence.** A summation number without its discretisation model is not reproducible, and a
bed-thickness statistic without its convention is not comparable between tools.

**Obligation.** `SB-CUT-002` (the model is named on every thickness-bearing result), `SB-CUT-015`
(reported bed thickness convention is explicit), `SB-CUT-014` (bed statistics emitted twice).

### F-5 — The σ convention: the same tabulated digit means three different things

**Tier T2 (IP `define_monte_carlo_parameters.htm` L162 + `D_cutoffs_montecarlo.md` §2.7) + T1a
(Techlog) + T1b/T3 (Geolog `montecarlo.montecarlo` `DEFAULT_PDF_SD = 3` + Configuration Files
Editor help).**

| Tool | Stated convention | σ from a symmetric tabulated width `w` |
|---|---|---|
| IP | *"Low Value Shift + High Value Shift represents four standard deviations"* | **σ = w/2** |
| Geolog | `σ = Shift / SD`, `DEFAULT_PDF_SD = 3` | **σ = w/3** |
| Techlog | σ supplied directly (`StandardDeviation`) | **σ = w** |

**Consequence, quantified.** A 3× spread on σ from the same typed digit. IP's convention is
independently corroborated twice: its tornado runs at *"± 2 standard deviations for Gaussian
distributions"*, and under `Lo+Hi = 4σ` with `Lo = Hi = w`, ±2σ is **exactly** the tabulated
Low/High edge — a second statement of `w = 2σ` that is inconsistent with `σ = w`. Applied to `Rw`,
whose tabulated widths are Geolog 5 % and IP 20 %, the σ values are **1.67 % and 10.0 % — a 6.0×
gap, not the 4× the printed numbers suggest**. Propagated through Archie at `n = 2`
(`∂lnSw/∂lnRw = 1/n = 0.5`) that is a P10–P90 half-width on Sw of ≈ ±1.1 % versus ≈ ±6.4 % for
the same well from two shipped vendor defaults.

**This finding has a live consequence inside SandiBumi** — see §3.9 and `SB-CUT-031`.

**Obligation.** `SB-CUT-031` (P0), `SB-CUT-036`, `SB-CUT-T13`.

### F-6 — Iteration defaults span two orders of magnitude, and convergence depends on the marginality of the pay

**Tier T2 (IP) + T1a (Techlog) + T3 (Geolog `determin_unc_ref_hc`).** IP defaults to **2000**
with an auto-stop architecture (burn-in 200, check every 100, minimum 300, tolerance 0.1 % on P10,
P50, P90 *and* mean simultaneously). Geolog defaults to **250** while its own documentation
recommends 1,000–5,000. Techlog defaults to **20**. Geolog additionally ships `determin_mc` with
`OPT_MC = 1` — Monte Carlo effectively off.

**Consequence, quantified.** A P10 from 20 iterations is the second-smallest of twenty samples;
its sampling error is of the same order as the spread it is reporting. Geolog's published
convergence experiment ran 10 / 750 / 5,000 / 10,000 iterations on a *marginal* interval and found
that the clear case converges by 750 while the marginal case does not — convergence is driven by
whether the data sit **on** the cut-off, not by parameter count. Thin-bed work lives in that regime
permanently.

**Obligation.** `SB-CUT-039`, `SB-CUT-045`, `SB-CUT-T33`.

### F-7 — Clamping perturbed data before accumulation manufactures hydrocarbon

**Tier T1a (Techlog clamps unconditionally at four call sites: L679, L681, L1281, L1651) + T2 (IP
clips by declared Curve Type) + T3 (Geolog states the opposite policy and its reason:
*"the data used to compute zone averages are the unlimited versions … to ensure that there is no
bias at the edges of the scales"*).** A genuine three-way split with one vendor arguing the case.

**Consequence, quantified.** For a truly wet interval (`Sw = 1.0`) perturbed with Gaussian noise
of standard deviation σ, clamping the draws at `Sw = 1` truncates the upper half of the
distribution and shifts the zonal mean by the half-normal mean offset:

```
E[min(1, 1 + σZ)] − 1 = −σ · E[max(0, −Z)] = −σ / √(2π) = −0.3989 σ
```

At Techlog's shipped σ = 0.1 v/v that is **−0.0399 v/v ≈ 4 saturation units of hydrocarbon
created out of noise**, in a zone that contains none. The bias is **independent of iteration
count**, so it cannot be found by running longer, and its sign is always toward more hydrocarbon.
IP's variant is worse in a different way: the clip policy is keyed on the *declared curve type*, so
mis-typing a curve silently changes its numerics.

**Obligation.** `SB-CUT-041` (P0), `SB-CUT-030`, `SB-CUT-T25`, and the extensions to
`SB-CUT-T15`/`T23`.

### F-8 — Techlog's shipped summation script has four independent defects

**Tier T1a, `SummariesMonteCarlo.py`, read to source depth.**

1. `average()` for the **harmonic** case early-returns `MissingValue` unless every sample in the
   interval is flagged — so a harmonic average over a partially-flagged zone silently returns null
   rather than the average of the flagged samples.
2. The **geometric** case has no non-positive guard: a single zero or negative sample takes
   `ln(C)` to `−inf` or `NaN`.
3. `limitType` modes **4, 5 and 6 raise `NameError`**; mode **7 is a silent always-pass**, i.e. a
   cut-off that filters nothing while appearing configured.
4. The cut-off contingency path at **L1635 omits `int()`**, so a float list subscript raises.

**Caveat, carried deliberately.** These are established for the shipped *Python* MC script. The
GUI Quanti Summaries module is C++ and was not read. Per the dossier's escalation 5, this MUST NOT
be reported as "Techlog is broken" in any positioning material without a live Techlog session.

**Obligation.** `SB-CUT-008` (harmonic skips and counts, never early-returns), `SB-CUT-007`
(non-positive guard on log-domain averages), `SB-CUT-020` (bounds-operator semantics tested
against SandiBumi's own spec, since Techlog's docstring and code disagree for modes 2 and 3),
`SB-CUT-T09`, `SB-CUT-T24`.

### F-9 — Techlog reports net-to-gross as a ratio of percentiles

**Tier T1a, `statDictNTG[p] = ni[p] / gi[p]`.** The reported P10 net-to-gross is the P10 of net
divided by the P10 of gross, not the P10 of the per-iteration net-to-gross.

**Consequence.** These are equal only when gross is constant. The moment gross carries any
uncertainty — a zone boundary, a deviation survey, a TVD conversion — they diverge, and the
divergence is in the tail statistic that a reserves case is quoted from. The error is latent in
Techlog today only because its gross is near-constant in the common configuration. Same C++
caveat as F-8.

**Obligation.** `SB-CUT-043`, `SB-CUT-T19` (the test must use a *varying* gross or it passes
vacuously).

### F-10 — Neither IP nor Techlog delivers the correlation coefficient it accepts

**Tier T1a + T2.** Both expose a "correlation" input and both implement it as a *blending weight*
between an independent draw and a shared draw, which is not a rank correlation and does not
produce the requested coefficient in the realised sample. Techlog's cut-off variant is
additionally broken by the float-subscript defect of F-8.

**Geolog takes a third position the first pass missed.** It ships **no** correlation coefficient
at all, as a stated design decision: combining independently-perturbed derived quantities
*"leads to situations where combinations of porosity and saturation which cannot exist in reality
are included in the Monte Carlo results"*. Its alternative is **Automatic Parameter Adjustment** —
parameters that were *picked from the data* (shale points, `m` and `Rw` from Pickett plots) are
re-derived from the perturbed logs each iteration rather than correlated to them.

**Consequence.** Both mechanisms are needed and neither substitutes for the other: a copula
handles correlated *measurement* error, and re-derivation handles the fact that a data-picked
parameter is not an independent input at all.

**Obligation.** `SB-CUT-049` (statistical half — already largely built, see §3.7) and
`SB-CUT-050` (causal half — new).

### F-11 — Percentile interpolation is unstated by all three vendors

**Tier T2 (IP), T1a (Techlog — `tlStat.percentile` is inside a compiled library, not the shipped
`.py`), T3 (Geolog — silent).** The choice changes P10 and P90 by up to one rank spacing.

**Consequence, quantified.** At IP's 2000 iterations one rank spacing is 0.05 % of the
distribution and negligible. At Techlog's 20 iterations the P10 is bracketed by ranks 2 and 3 and
the interpolation rule *is* the answer. This is a decision, not a lookup — no further reading will
settle it.

**Obligation.** `SB-CUT-046`. SandiBumi already implements type-7 (§3.8); the requirement is that
it is **stated on the output record**, which closes the dossier's escalation 1.

### F-12 — A single global percentile-direction flag cannot express the vendor's own carve-out

**Tier T1b + T2.** IP's `MonteCarloDefaults.par` `Results` line is `Yes 10 50 90 -999 -999`, and
its header defines *"'Yes' as the first parameter means the 10% is the 10 percentile lowest
value"* — one global flag. IP's manual separately describes a carve-out for Sw, where the
optimistic case is the *low* end. The shipped file cannot express it.

Geolog answers the same problem properly: it names the ambiguity (*"some people consider P10
optimistic, others pessimistic … certainly not universal usage"*), adopts SPE Reserves Booking
Guidelines (2001) terminology, and makes the report self-describing — *"The actual probabilities
used are quoted in the summary reports."*

**Obligation.** `SB-CUT-047` — reserves-category naming with analyst-settable probabilities, the
actual probability printed on every quoted case, and a per-quantity `direction` in metadata rather
than one global flag.

### F-13 — Two shipped vendor prior sets, one exact agreement in the whole table

**Tier T1b, IP `MonteCarloDefaults.par` vs Geolog `determin_mc.info`.** Geolog's priors are
percentage errors named in the identifier (`ERR_PC_RW 5`, `ERR_PC_RT 2`, `ERR_PC_RHO 2`,
`ERR_PC_NPHI 5`, `ERR_PC_DT 5`, `ERR_PC_GR 5`, `ERR_PC_M 10`, and thirteen more at 10 %).

**Consequence, quantified.** Row for row against IP: **exactly one row agrees exactly** — neutron
at 5 %. `Rw` is 4× apart on the tabulated width and 6.0× apart on σ (F-5). `Rho Matrix` is ~2×
apart. `Rt`/`Rxo` are **not comparable at all**, because IP's shift is reciprocal and Geolog's is
percentage. Neither vendor documents the provenance of any prior.

**Obligation.** `SB-CUT-028b`-class discipline is expressed as `SB-CUT-036` (explicit
`(value, basis, sigma_multiple)` triple, never a bare number) and the §5 table, where every one of
these is `NON-ADOPTABLE — cited for verification`.

### F-14 — IP's reciprocal shift is not expressible in either competitor

**Tier T1b.** IP's `InCurves` block applies `Rec` (reciprocal) shifts to `Rt` and `Rxo`:
`Result = 1 / (1/Input + Shift)` at `Shift = 0.005`. Techlog offers `absolute` and `relative`;
Geolog offers `Linear` and `Linear (%)`. Neither has a reciprocal sampler.

**Consequence, quantified.** A `s = 0.005` (ohm-m)⁻¹ shift is ±0.5 % at `Rt = 100` and
**−33 % / +100 % at `Rt = 1`**. Importing it as a Linear ±0.005 ohm-m shift is negligible at high
resistivity and catastrophic at low — a defect that hides perfectly in high-resistivity test data
and appears in a water leg. It is not a units conversion; it changes the prior's *shape*.

**Obligation.** `SB-CUT-032` — the shift **type** is stored with the value, and an import meeting
a `Rec` shift without a reciprocal sampler is a **load error**, never a coercion.
`SB-CUT-T20`/`T30`.

### F-15 — Null footage is tracked by one vendor and marked in-band by another

**Tier T1a (Techlog books a non-positive clipped interval as **UNKNOWN**, distinct from NOT-NET)
+ T2 (IP prints `$$` inside a numeric report column to mean "nulls present").**

**Consequence.** `Gross = Net + NotNet + Unknown` is an auditable identity; `Gross = Net + NotNet`
with nulls silently in one of them is not. `N:G` and `N:(G−Unknown)` differ by exactly the null
fraction, and over a badly-logged interval that is the difference between a defensible and an
indefensible net-to-gross. An in-band `$$` in a numeric column is a parse failure waiting to
happen and cannot be carried through a calculation.

**Obligation.** `SB-CUT-003`, `SB-CUT-004`, `SB-CUT-029`, `SB-CUT-T11`.

### F-16 — Only one vendor models bed amalgamation, and net-to-gross is not scale-invariant

**Tier T1b (Geolog `LUMP_SATHK` / `LUMP_INCTHK` / `LUMP_MAXSEP`) + T4 (Bentley & Ringrose).** IP
has a minimum height only; Techlog has `thinBedInterval` and nothing else; Geolog models three
thresholds — minimum bed thickness, maximum separation between beds to be merged, and maximum
non-net thickness that may be included inside a merged bed.

**Consequence, quantified.** The literature result the dossier treats as authoritative: net-to-
gross for the *same* sand drifts **0.55 → 0.75 → 1.0** across three successive blocking steps, and
a published field case over-estimates N/G_res **0.36 → 0.83 → 0.88**. A net-to-gross number is
therefore **not scale-invariant**, and a summation result that does not record its sample interval
and its amalgamation thresholds cannot legitimately be compared against a model N/G at all.

**Obligation.** `SB-CUT-013`, `SB-CUT-014`, `SB-CUT-T31`.

### F-17 — One vendor changed its cut-off activation trigger between two modules of one product

**Tier T3, Geolog.** `Determin` activates a cut-off on the presence of the *curve*;
`determin_mc` activates it on the presence of the *cut-off value*. Same feature name, same
product, different trigger.

**Consequence.** An inferred activation rule cannot be audited: given a result, there is no way to
determine which cut-offs were live without re-deriving the inference, and the inference is not
stable across the vendor's own modules.

**Obligation.** `SB-CUT-022` — an explicit per-cut-off enable flag, never inferred from the
presence of a curve or of a value.

### F-18 — Two vendors print statistics their own documentation says are unsupported

**Tier T3 (Geolog) + T2 (IP) + T1a (Techlog).** Geolog **withholds** `EHC_CDF` below
`OPT_MC > 10`, stating *"Below this number the results are meaningless."* IP prints a 50-cell
modal statistic while its own page states it needs ≥3 values per cell. Techlog prints a P10 from
20 iterations.

**Consequence.** This is CONTRACT §5 point 3 exactly: a computation proceeding outside its
validity and producing a plausible number. One vendor demonstrates that the refusal is
implementable and shippable.

**Obligation.** `SB-CUT-045` — a statistic whose preconditions fail is **withheld with a
machine-readable reason**, not printed, not `NaN`, not silently the minimum. `SB-CUT-T29`.

### F-19 — A vendor's own manual mixes `pu` and `v/v` for one quantity

**Tier T2, IP.** The cut-off sensitivity sweep example is expressed in porosity units; the cut-off
default is expressed in v/v. Same quantity, same product, two units, no unit tag on the field.

**Consequence, quantified.** `35` typed into a field expecting `0.1 v/v` is a **350× error**. Its
symptom is not a crash or a null — it is an all-net result, which looks like a good well.

**Obligation.** `SB-CUT-019` — cut-off entry is unit-tagged and a bare number is **rejected**.
`SB-CUT-T26`.

### F-20 — Two vendors ship uncertainty off and cut-offs on

**Tier T1a (all six of Techlog's `_Uncertainty` flags ship `no`) + T1b (Geolog `determin_mc` ships
`OPT_MC = 1`, one iteration, against a validation range of `1:50000`).**

**Consequence.** Two vendors independently chose the safe failure for perturbation: a Monte Carlo
run left at defaults produces **zero spread rather than invented spread**. The same two vendors
chose the *unsafe* failure for cut-offs, shipping active values. The asymmetry is instructive and
both halves are adopted — perturbation off by default (`SB-CUT-052`), cut-offs absent by default
(`SB-CUT-016`).

### F-21 — One vendor implemented, tested and dropped the per-depth draw regime

**Tier T3 (Geolog) + T1a (Techlog).** Geolog names both regimes. *Vertical* processing applies one
offset per section per iteration and models **systematic** uncertainty (accuracy). *Horizontal*
processing draws independently per depth sample and models **precision**. Geolog implemented
horizontal, tested it, and dropped it: it *"does not effectively model systematic uncertainties …
does not allow for sensitivity studies … does not allow for auto adjustment … does not allow
results to be defined on a percentile basis."*

Techlog draws **per depth sample** for inputs (L1216–1218) and **per zone** for cut-offs
(L1599–1601) — it mixes the two regimes inside one module.

**Consequence, quantified.** Independent per-depth draws average out as **1/√N** over a summation
interval. Over a 100-sample zone the reported zone-average spread is **10× too narrow**, and it
narrows further the more finely the well is sampled — an uncertainty band that shrinks when you
increase the logging rate is measuring the wrong thing.

**Obligation.** `SB-CUT-040`, `SB-CUT-T27`.

### F-22 — A shipped vendor spec set carries a name collision

**Tier T1b, Geolog.** `vshale-only_{metric,imperial,mixed}.paysum` all three declare
`NAME = default_imperial`, which is also the name of a *different* shipped spec file. The metric
and mixed variants are mislabelled in their own headers, and `CUTOFF_NDP` differs between them
(3 metric vs 2 imperial/mixed).

**Consequence.** A spec-import keyed on the declared `NAME` silently loads the wrong unit system.
The metric/imperial pair differs *only* in the thickness thresholds — the dimensionless cut-offs
are identical — so the failure is invisible on every quantity except footage.

**Obligation.** `SB-CUT-060` (identity on import is `(block, ordinal, semantic key)`, never a
name), and `SB-CUT-019`'s unit discipline (dimensionless cut-offs are unit-invariant; thickness
thresholds are not).

### F-23 — A numeric formatter whose precision can exceed its field width

**Tier T2, IP.** `Result Precision` defaults to 3 with a maximum of 6, against an **8-character**
report field. A precision-6 value in a wide unit overflows the field. Report Title is capped at 25
characters and Short Name at 4.

**Consequence.** Silent truncation in a delivered report — the number is wrong and nothing says so.

**Obligation.** `SB-CUT-061` — precision and field width are validated **against each other** at
configuration time, not set independently.

### F-24 — A TVD summation is not a rescaling of an MD summation

**Tier T2, IP: zonal averages are weighted by vertical thickness per depth increment, so TVD
zonal averages *"could be considerably different"* from MD averages.** The per-sample weight is
`Δz` in MD and `Δz·cos θ` in TVD, so the *weights* differ, not merely the totals.

**Consequence.** In a 60° hold section the weights differ by a factor of two between frames, so
the thickness-weighted average of any curve differs — the frame is part of a result's **identity**,
not a display option. Techlog offers four frames (MD, TVD, TVDSS, TST); IP offers two.

**Obligation.** `SB-CUT-012`.

### F-25 — Reservoir and Pay share one cut-off value and two independent use flags

**Tier T1b, IP `Cutoff.hlp` ordinals 1–3: `Phi Net Use`, `Phi Pay Use`, `Phi Cutoff`, the last
described as *"Porosity cutoff value for Pay and Reservoir report"*.** One value, two flags.
Reports 3/4/5 each get their own values (ordinals 47/48/49 for φ, 61/62/63 for Sw, 75/76/77 for
Vcl). Separately, `Sw Net Use` and `Sw Pay Use` are independent ordinals and Net Reservoir is
described as porosity- and clay-driven — **Sw is off by default for the reservoir tier**.

This also settles a prior open item the record said needed a live IP session (ledger D-5.3 /
D-OPEN-2): it did not.

**Obligation.** `SB-CUT-024` (flag tiers over arbitrary cut-off sets), `SB-CUT-026` (Sw MUST NOT
default on for the reservoir tier), and the cut-off record shape in `SB-CUT-022`.

---

## 3. SandiBumi as-built

Read from source on 2026-08-07. Files: `src-tauri/src/workflow.rs` (3,931 lines),
`src-tauri/src/montecarlo.rs` (2,732), `src-tauri/src/netflag.rs` (498),
`src-tauri/src/report.rs` (1,442), `src-tauri/src/modules.rs`, and the frontend
`src/ui/{cutoffs,cutoffDialog,summaryDialog,reportDialog,dashboardPanel,monteCarloDialog,resultsQcPanel}.ts`.

**Two pointers in the assignment brief were stale and are corrected here.** The cutoff/lumping
engine and the `FLAG_SAND` / `FLAG_RESERVOIR` / `FLAG_PAY` curves are **not** in `modules.rs`;
they are `run_pay_summary` in `workflow.rs:922`, which writes the three flag curves at
`workflow.rs:1005` and `workflow.rs:1030-1032`. And `workflow.rs:651` is
`Err(e) => Outcome::Failed(e)` inside `run_workflow_module_into` — not a pay-summary fabrication
site. The pay-summary row is emitted at `workflow.rs:1098-1117`.

### 3.1 The four assigned "live findings" — three are already closed in the source

The brief named four defects to verify at the code and, *if confirmed*, carry as requirements.
Three of the four are **fixed in the shipped source**, each with an in-code comment naming the
previous defect. Per CONTRACT §5's closing rule they are reported here as `PRESENT-OK` and carried
into §4 as **regression locks** — a requirement whose obligation is that the fix stays fixed —
rather than asserted as live defects. Reporting a closed defect as open is an overclaim in the
other direction and costs the same credibility.

| Brief's finding | Verified status |
|---|---|
| 1. Six panes, two different default sets | **PARTIAL — partly live.** Five of six panes unified; one still hard-codes. The *provenance* problem is fully live. §3.2 |
| 2. `run_net_flag` can never deserialize (camelCase/snake_case) | **PRESENT-OK — fixed.** `netflag.rs` header + `#[serde(deny_unknown_fields)]`. §3.3 |
| 3. Monte Carlo silently swallows module errors | **PRESENT-OK — fixed.** `montecarlo.rs:1148` `run_realization` records the first failure; surfaced at `:1825-1828` and `:1834-1838`. §3.3 |
| 4. Pay summary fabricates Net 0.0 / N:G 0.00; report omits the section on error | **PRESENT-OK — fixed.** `workflow.rs:865-866` + `report.rs:454-458`, `:507-512`, `:533-549`. §3.3 |

### 3.2 Cut-off defaults — `PARTIAL`, and the provenance gap is total

`src/ui/cutoffs.ts` is the unification point introduced to close the six-pane drift:

```ts
export const DEFAULT_CUTOFFS: CutoffDefaults = { vsh_max: 0.5, phie_min: 0.1, swe_max: 0.6, perm_min: null };
```

Its own doc comment records the drift it fixed — Monte Carlo had hard-coded PHIE ≥ 0.08 / SWE ≤
0.5 against the summary's 0.1 / 0.6 while the MC settings tooltip claimed *"Cutoffs match the pay
summary"*. `cutoffDialog.ts`, `summaryDialog.ts`, `reportDialog.ts`, `monteCarloDialog.ts` and
`resultsQcPanel.ts` all now import `loadCutoffDefaults()`.

**Status `PARTIAL`, two distinct residuals.**

1. **`src/ui/dashboardPanel.ts:56-59` is a sixth pane that does not import `./cutoffs`** and
   hard-codes the same three numbers as literals:

   ```ts
   const vshIn = num("0.5");
   const phieIn = num("0.1");
   const sweIn = num("0.6");
   const permIn = num("", "(off)");
   ```

   It therefore neither honours a saved project cut-off set nor participates in the drift
   protection the other five now share. It agrees with `DEFAULT_CUTOFFS` **today**, by coincidence
   of transcription, and nothing prevents the next edit from re-opening the divergence.

2. **`PRESENT-DIVERGENT` — no document anywhere states where 0.5 / 0.1 / 0.6 came from.** A
   repository-wide search for the provenance of these three values returns four hits
   (`docs/playbook_build_progress.md:351`, `REVIEW.md:1794`, `docs/review_sweep/F3.md:74`,
   `docs/manual_test_plan.md:4511`) and every one of them records only what the values were
   unified *to*. None cites a source. Against F-1's evidence — four shipped vendor sets, no two
   identical, two from the same vendor, and delivered work spanning Vsh 0.20–0.85 / PHIE
   0.05–0.27 / Sw 0.50–0.90 — these are three unsourced numbers presented to an interpreter as
   defaults. That is a direct CONTRACT §2 violation and the highest-priority requirement in this
   chapter (`SB-CUT-016`, `SB-CUT-017`).

**Cut-off entry widgets — `PARTIAL`.** `src/ui/monteCarloDialog.ts:464-467` bounds the three
fields to `[0, 1]`:

```ts
const vshMax  = numField("VSH ≤", cuts.vsh_max, 0, 1);
const phieMin = numField("PHIE ≥", cuts.phie_min, 0, 1);
const sweMax  = numField("SWE ≤", cuts.swe_max, 0, 1);
const permMin = numField("PERM ≥ (blank=off)", cuts.perm_min ?? NaN, 0, 1e6);
```

`src/ui/cutoffDialog.ts:104-111` does not:

```ts
const numInput = (value: string, cls = "form-control"): HTMLInputElement => {
  const i = document.createElement("input");
  i.className = cls;
  i.type = "number";
  i.step = "any";
  i.value = value;
  return i;
};
```

No `min`, no `max`, no unit. `35` typed into the PHIE field is accepted and stored as `35 v/v` —
F-19's 350× error, reachable today in the primary cut-off dialog, with an all-net result as its
symptom. Neither widget carries a unit tag anywhere; the labels read `PHIE ≥` and nothing else.

### 3.3 What is already right, and must be locked

**`PRESENT-OK` — the Tauri DTO break is closed and documented.** `netflag.rs` carries the rule in
its header and enforces it structurally, with `#[serde(deny_unknown_fields)]` on `NetFlagSpec` and
a comment naming *"the silent direction of the camelCase break that made this whole feature a
no-op"*. The convention it states: **struct DTOs cross the wire in snake_case, because Tauri
camel-cases only the top-level command argument key, not nested fields.** `src/ipc.ts:542-571`
documents the same rule from the TypeScript side and notes that `netflag.rs` has a test that reads
the interface and fails on drift. The polygon test itself (`point_in_polygon`, even-odd ray cast)
runs in the **transformed** plane, so "inside the drawn polygon" is exact on log axes.

**`PRESENT-OK` — Monte Carlo module failure is loud.** `montecarlo.rs:1148` `run_realization`
records the first failure into a `OnceLock` rather than dropping it, with the comment explaining
the previous behaviour: *"Swallowing it left the pool unchanged, so every downstream step read NaN
and the study came back as a P10=P50=P90 table of zeros with nothing to explain it."* It surfaces
at `:1825-1828` as `"{well_id}: chain step failed on every realization"` and marks the job item
`Failed` at `:1834-1838`.

**`PRESENT-OK` — the pay summary does not fabricate.** `PaySummaryRow.n_classified`
(`workflow.rs:866`) carries the count of samples that were actually classified, with the contract
stated at `workflow.rs:865`: *"Consumers must render '—' rather than 0.00 when this is 0."* The
comment at `:880` records the failure it prevents — two wells of identical rock reporting 0 and
full net pay. `perm_cutoff_no_data` distinguishes "PERM cut-off inactive" from "PERM cut-off
active and no PERM curve present".

**`PRESENT-OK` — the report never silently drops the pay section.** `report.rs:454-458` emits the
section header unconditionally, with the comment naming the previous `unwrap_or_default()`
collapse as *"exactly the cardinal-rule failure the report path must not allow"*. `:507-512`
renders `-` for Net / NTG / HPV when `n_classified == 0`; `:533-549` pushes a `note_page` for both
the `Ok(empty)` and the `Err(e)` branch; a `pay_caveat` explains the PERM-no-data case on the page
itself. The 540-PDF silent-omission scenario in the brief is not reachable in this source.

**`PRESENT-OK` — plausibility is reported, not excluded.** `montecarlo.rs:341-366` counts
realizations whose sampled parameter combination drove the petrophysics out of physical bounds on
the chain's **unlimited** companion curves (`PHIE_DN`, `SWT_ARCH`, `SWE_INDO`), and carries
`checked: bool` so a well with no finite porosity/saturation samples shows *"not checked"* rather
than a fabricated clean pass. This is ahead of all three incumbents and must be claimed as such.

**`PRESENT-OK` — derived ratios are computed inside the iteration.** `montecarlo.rs:812`
`ntg: if gross > 0.0 { (net / gross) as f32 } else { 0.0 }` sits **inside** `zone_metrics`, i.e.
inside the realization, so the reported P10 net-to-gross is the P10 of the per-iteration ratio.
This is the correct side of F-9 and beats Techlog's `statDictNTG[p] = ni[p]/gi[p]`.

**`PRESENT-OK` — correlation refuses rather than approximates.** `iman_conover`
(`montecarlo.rs:480`) implements rank-correlation induction with van der Waerden scores, the
Spearman→Pearson pre-adjustment at `:525` (`let rho = 2.0 * (PI * rho_s / 6.0).sin();`) and
Cholesky re-colouring (`cholesky`, `:595`). Both gates refuse rather than approximate: a
non-positive-definite target matrix pushes the note *"correlation targets are jointly inconsistent
(matrix not positive-definite); correlations skipped"* and returns, and `:469` refuses below 2
parameters or 10 iterations. This is beyond both vendors' blending weights (F-10, statistical
half).

**`PRESENT-OK` — the seed is mandatory and lands in the result record.**
`src/ui/monteCarloDialog.ts:461` (`numField("Seed", 42, 0, 1e9)`) is a required field, and
`montecarlo.rs:1730` writes it into the persisted `MONTECARLO` log set's `params_json` alongside
`iterations`, `sampling`, `low_pctl`, `high_pctl`, `kept_realizations` and the parameter list. Per
the dossier's §4.2 correction 2 this **matches Geolog and beats IP and Techlog** — it must never
be positioned as beating all three.

**`PRESENT-OK` — the flag hierarchy has the right shape.** `workflow.rs:1005` / `:1030-1032` write
`FLAG_SAND` (Vsh only), `FLAG_RESERVOIR` (Vsh + PHIE) and `FLAG_PAY` (Vsh + PHIE + SWE, plus PERM
when active). Saturation is **not** applied at the reservoir tier, which is the correct side of
F-25's `Sw Res Use` finding.

### 3.4 Depth discretisation — `PRESENT-DIVERGENT`, hard-coded TOPS, implemented three times

`workflow.rs:973-983`:

```rust
// Sample thickness: forward depth difference, last sample reuses the previous step.
let mut step = vec![0.0f32; n];
for i in 0..n {
    step[i] = if i + 1 < n { depth[i + 1] - depth[i] }
              else if i > 0 { step[i - 1] } else { 0.0 };
}
```

with the zone clip at `workflow.rs:1067-1074`:

```rust
let s_top = depth[i] as f64;
let s_bot = (depth[i] + step[i]) as f64;
let lo = s_top.max(zone.top_depth as f64);
let hi = s_bot.min(zone.bottom_depth as f64);
let h = hi - lo;
if h <= 0.0 { continue; }
```

and gross **set, not summed**, at `workflow.rs:1097`: `let gross = zone.bottom_depth - zone.top_depth;`.

This is **TOPS with zone clipping** — structurally identical to Techlog's shipped path, and
therefore on the side of F-3 that yields 3.0 ft on IP's fixture where IP yields 3.25 ft. Three
divergences from the requirement:

1. **The model is not selectable.** There is no CENTRED and no BOTTOMS. Four independent vendor
   votes favour CENTRED as the default (IP hard-codes it; Geolog `tp_paysummary.info` L63
   `FRAME_REP` defaults to `CENTRALISED`; `determin_mc` pins it with no alternative; Techlog's
   unreachable `"centred"` branch implements it).
2. **The model is not named on the result.** `PaySummaryRow` carries no discretisation field, and
   nothing in the report names one. F-4's obligation is unmet.
3. **The rule is implemented three times independently** — `workflow.rs:973-983` (pay summary),
   `workflow.rs:1476-1483` (cut-off sweep) and `montecarlo.rs:1046-1052` (Monte Carlo). Three
   copies of a numeric contract with no shared authority is exactly the structure in which two of
   them drift and the third is used to argue that they agree. The Monte Carlo copy carries a
   comment asserting that it *"mirrors `run_pay_summary`"* — an assertion enforced by nothing.

### 3.5 Averaging — `PARTIAL`, arithmetic only

`workflow.rs:1113` computes the φ-weighted saturation correctly:

```rust
avg_swe: if sum_phie_w > 0.0 { (sum_phie_swe / sum_phie_w) as f32 } else { f32::NAN },
```

and per-curve average denominators (`net_vsh`, `net_phie`) are tracked separately, so a missing
PHIE over part of a zone does not drag `avg_phie` toward zero — a subtle correctness the vendors
do not all get right.

`ABSENT` in this area: the generalised power mean, the geometric average, the harmonic average,
any per-curve choice of average, the non-positive guard those require, and any extra-curve
averaging at all. φ-weighting is hard-wired to the saturation slot rather than being an explicit
per-curve flag, so it cannot be requested for a different curve and cannot be switched off for a
curve that should not carry it.

### 3.6 Footage accounting — `PARTIAL`

Gross is set from the zone and net is accumulated (`workflow.rs:1097`, `:1098-1117`), so the
`Σhᵢ = Z_bot − Z_top` invariant holds by construction rather than by reconciliation. What is
`ABSENT`: a `NotNet` category, an `Unknown` category, the `Gross = Net + NotNet + Unknown`
identity, `N:(G−Unknown)`, and any reconciliation tolerance or recorded residual. `n_classified`
partly covers the "was this interpreted at all" question but does not partition the footage.

Bed amalgamation is `ABSENT` from the summation path. `modules.rs:1397-1428` implements a
`MIN_THICK` spike-removal for the conditioning flags (coal / tight / crossover / bad-hole), with
run bridging across nulls and a correct *"thickness counts one sample spacing beyond the run's
depth extent"* rule — but it is a *conditioning* filter on a different set of flags, not the
`SATHK` / `INCTHK` / `MAXSEP` lumping model of F-16, and it does not touch `FLAG_PAY`. Bed
statistics (interval count, total net, mean, thinnest, thickest) are `ABSENT` entirely.

### 3.7 Monte Carlo engine — `PRESENT-OK` core, `PARTIAL` surface

`Rng` (`montecarlo.rs:393-419`) is SplitMix64 with Box–Muller; `build_draws` (`:429`) is Latin
Hypercube by default with jittered strata `(i + rng.unit())/n` and a Fisher–Yates stratum
permutation, with a documented `Sampling::Random` path that is byte-identical to the pre-LHS
sequence for a given seed (`:154`). `percentile` (`:820`) is **type-7 linear interpolation**,
named in the code. `summarize` (`:834`) returns all-NaN for a no-data metric rather than a zero.
`spearman` (`:937`) uses average ranks for ties. `CONV_MIN_ITER = 200` (`:672`) floors the
auto-stop, and the request is clamped to `1..=100_000` at `:1241`.

Divergences and gaps:

- **`PRESENT-DIVERGENT` — the σ reading.** See §3.9. This is the P0.
- **`PRESENT-DIVERGENT` — iteration default 1,000.** `src/ui/monteCarloDialog.ts:460`
  (`numField("Iterations", 1000, 1, 100000)`). Inside Geolog's recommended 1,000–5,000 band, below
  IP's 2,000. The number is defensible; its *absence of a source* is not.
- **`ABSENT` — distributions.** `montecarlo.rs:46-50` offers Normal, Uniform and Triangular only.
  No log-normal, no log-triangular, no log-uniform, no reciprocal (`Rec`) shift. The log variants
  are the ones permeability and Rw actually need.
- **`ABSENT` — cut-offs are never perturbed.** `montecarlo.rs:1248` builds `Cutoffs` once from the
  request, outside the realization loop. Both IP and Techlog perturb cut-offs; delivered work uses
  LOW/BASE/HIGH cut-off cases.
- **`ABSENT` — no Gaussian truncation**, no `TRUNC_SD`, and therefore no reported variance deficit.
- **`ABSENT` — no per-iteration joint record**, so no iteration-consistent percentile *case*. The
  percentile arrays are marginal only. `kept_realizations` is capped at 1,024 when persisting
  (`:1266`), which is a retention policy, not a joint-record contract.
- **`ABSENT` — no reserves-category naming**, no printed probabilities, no per-quantity direction.
- **`PARTIAL` — the refuse-to-report gate.** `CONV_MIN_ITER = 200` gates the *auto-stop*, not the
  *reporting*. A 5-iteration run still returns a P10.
- **`PARTIAL` — percentile interpolation.** Type-7 is implemented and named in the code, but is
  not echoed onto the output record, so a consumer of the result cannot tell.
- **`PARTIAL` — tornado.** One-at-a-time at each distribution's P10 / median / P90, plus Spearman
  rank correlation. The absolute-units and iteration-count-labelling contract of F-21 is not
  enforced on the emitted values.
- **`PRESENT-OK` — the draw regime is vertical.** Parameters are drawn once per realization and
  applied across the section, never per depth sample. This is the correct side of F-21, though it
  is a consequence of the architecture rather than a stated and tested choice.

### 3.8 Clamping — `PARTIAL`, with an in-code rationale the cross-tool evidence overturns

`zone_metrics` (`montecarlo.rs:758-816`) accumulates from the chain's **limited** curves. The
unlimited companions exist and are used, but only for the plausibility diagnostic
(`:1276`, `:1385`). `montecarlo.rs:344-350` states the rationale for keeping clamped realizations
in the pool:

> the clamp already gives an impossible draw the physically-correct volumetric answer (an
> over-dense matrix → zero effective porosity, a supersaturated combo → fully wet), so they remain
> valid low/high tails of the distribution — dropping them would bias P10/P90.

**That argument is correct for a single deterministic evaluation and incorrect in expectation over
an ensemble**, which is exactly F-7. For a truly wet interval, the unclamped hydrocarbon
contribution `φ(1−Sw)` has expectation zero under symmetric noise; the clamped contribution
`φ·max(0, 1−Sw)` has expectation `φ·σ/√(2π) = 0.3989 φσ > 0`. Clamping does not merely relocate a
tail — it moves the **mean**, always toward more hydrocarbon, by an amount independent of
iteration count.

**Two things bound the exposure honestly, and both are stated because neither removes it.**
Accumulation runs over pay-flagged samples only (`montecarlo.rs:800`, `if !pay { continue; }`), and
the flag test itself runs on the same clamped curves. So with a restrictive saturation cut-off
(`swe_max < 1`) the Sw clamp at 1 is inert — the samples it would have altered fail the cut-off
anyway. The bias is fully live in exactly two configurations: a permissive cut-off set
(`swe_max = 1`, `phie_min = 0` — the whole-zone-average configuration, and precisely what Geolog's
`tp_pay_sensitivity` ships), and any future reporting of a zone average that is not pay-restricted.

The structural gap is unconditional: there is **no** `accumulate` / `flag_test` / `present` stage
separation, no configuration in which the accumulator sees unclamped data, no `out_of_range` flag
on an emitted zone average, and bounds are attached to module output rather than to the quantity.

### 3.9 `PRESENT-DIVERGENT` — every IP-seeded Gaussian prior is exactly 2× too wide

`docs/ref_monte_carlo_seeds.md` imports twelve per-parameter widths from IP's
`MonteCarloDefaults.par` and adopts an explicit reading at its line 50:

| Distribution kind | Interpretation of the tabulated shift `w` |
|---|---|
| Normal | **one standard deviation** — σ = `w`, so ≈68% of draws land within ±`w` of the value |

The document's stated premise (line 45) is that *"The `.par` header states the shift's units but
**not** which percentile of the Gaussian the tabulated shift corresponds to."* **That premise is
factually wrong.** IP states the convention on its Monte Carlo manual page: *"Low Value Shift +
High Value Shift represents four standard deviations."* With `Lo = Hi = w` that gives `2w = 4σ`,
i.e. **σ = w/2**. It is corroborated independently by IP's tornado, which runs at ±2σ — exactly
the tabulated Low/High edge under the same mapping.

The reading is live in the code. `src/ui/monteCarloDialog.ts:236-240`:

```ts
const seeded = seedWidth(param, d);
const has = Number.isFinite(seeded);
const spread = has ? seeded : Math.max(Math.abs(d) * 0.1, 0.01);
const wide = has ? seeded : Math.max(Math.abs(d) * 0.2, 0.02);
if (kind === "normal") return [d, spread, d + spread];
```

`spread` is passed straight through as σ. **Every one of the twelve seeded Gaussian priors is
therefore exactly 2× too wide**, and each row carries a muted `IP` badge
(`monteCarloDialog.ts:95-102`) asserting IP provenance for a width IP does not mean. Worked:
`M` is seeded `w = 0.2` (`monteCarloDialog.ts:70`) and realises σ = 0.20 where IP's own convention
gives **σ = 0.10**. Since `Sw` depends on `m` through `φ^(−m/n)`, `∂lnSw/∂m = −lnφ / n`, so
doubling σ on `m` doubles the reported P10–P90 half-width on saturation. Every IP-badged
uncertainty band SandiBumi has printed to date is twice its intended width.

Two further gaps in the same document, both from the dossier's §4.2:

- **The shift *type* is not stored.** `IP_MC_SEEDS` (`monteCarloDialog.ts:68-81`) carries
  `{ w, pct }` only. IP's `Rec` type has no representation, so an `Rt` prior cannot be imported
  without silently changing its shape (F-14).
- **The `InCurves` block was never imported.** `ref_monte_carlo_seeds.md:37-39` excludes it
  deliberately. Those seven rows are the only ones in the file describing *measurement*
  uncertainty, and they are what a Monte Carlo over a **cut-off** module actually needs, because
  cut-offs act on curves rather than on parameters.

### 3.10 Sensitivity sweep — `PARTIAL`

`run_cutoff_sweep` (`workflow.rs:1401`) sweeps **one** property (`VSH` | `PHIE` | `SWE`) over
`[sweep_min, sweep_max]` at `steps` points, reporting `NET` | `HPV` | `NTG`, optionally masked to
a zone and to DST or perforation intervals via `aux_intervals` and `sample_incl_thickness`. The
interval masking is a genuine capability none of the three incumbents documents.

`ABSENT`: the multi-cut-off permutation sweep (Geolog sweeps four simultaneously), the inverse
solve (IP alone can solve backwards from a target net or HCPV), and live crossplot-driven cut-off
editing. The `PLT` seam owns the last of these.

### 3.11 Import surface — `ABSENT`

There is no IP `.par` / `.hlp` import path, so none of the block-scoped-ordinal, shift-type or
precision/field-width contracts exists yet. This is recorded as `ABSENT` rather than omitted
because the requirements are cheap to honour while the importer is being written and expensive to
retrofit afterwards.

---

## 4. Requirements

Sixty-one requirements. Nine are `P0`. Four of the nine (`SB-CUT-054` … `SB-CUT-057`) are
**regression locks** on defects already fixed in the shipped source: their obligation is that the
fix cannot be undone silently, and their `As-built` line is `PRESENT-OK`. They are P0 because each
one, if reopened, presents a failed result as a clean one — CONTRACT §5 point 3, and the
application's own cardinal rule.

**`SB-CORE-002` — this chapter's share.** `04_CORE_REQUIREMENTS.md` §15.1 names three of its seven
shipped violations in this domain: Monte Carlo swallowing module errors and reporting all-NaN
volumetrics as success; the pay summary fabricating `Net 0.0 / N:G 0.00 / HPV 0.00` for wells whose
inputs were never computed; and a batch report emitting 540 PDFs all missing their pay tables while
reporting zero errors. **All three were verified at the source on 2026-08-07 and all three are
already closed** — §3.1 and §3.3 carry the `file.rs:line` evidence and the in-code comments naming
each previous defect, and two of the three line pointers carried in the core chapter
(`montecarlo.rs:1125`, `workflow.rs:651`, `report.rs:380`) no longer resolve to the cited behaviour.
They are therefore carried here as `SB-CUT-054`, `SB-CUT-055` and `SB-CUT-056`, at **P0**, with
`PRESENT-OK` as-built and a regression lock as the obligation. **`SB-CORE-002`'s status for this
domain should be revised from `PRESENT-DIVERGENT` to `PRESENT-OK, regression-locked` once
`SB-CUT-T37`, `SB-CUT-T37b` and `SB-CUT-T37c` are green** — the spine holds that call, and it is
raised in §7 as an escalation rather than decided here. Per `SB-CORE-002`'s own verification clause,
each of those three tests asserts on the **reported artefact** — the summary row, the PDF page, the
job result — never only on the internal `Result`.

**`SB-CORE-004` — this chapter's worked example.** `04_CORE_REQUIREMENTS.md` §15.1 cites this
domain's cut-off defaults as the thinnest visible place in the parameter-source discipline.
`SB-CUT-016`, `SB-CUT-017` and `SB-CUT-018` are the domain-level discharge of it, and §5 records
both disagreeing sets, records that neither carries a source, and specifies the refusal rather than
adjudicating between them.

### 4.1 Discretisation and summation

#### SB-CUT-001 — Make the depth discretisation model an explicit parameter [P1] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST expose the depth discretisation model as a named parameter with
the values `CENTRED`, `TOPS` and `BOTTOMS`, MUST default it to `CENTRED`, and MUST implement all
three by the single interval-ownership rule plus the shared zone clip
`hᵢ = max(0, min(Z_bot, bᵢ) − max(Z_top, aᵢ))`. The invariant `Σhᵢ = Z_bot − Z_top` MUST hold
exactly under every model. The rule MUST have exactly one implementation, shared by the pay
summary, the cut-off sweep and the Monte Carlo path.

**Rationale.** F-3. Four independent vendor votes for CENTRED (T2 IP hard-codes it; T1b Geolog
`tp_paysummary.info` L63 `FRAME_REP` defaults `CENTRALISED`; T1b `determin_mc` pins it; T1a
Techlog implements an unreachable `"centred"` branch). Techlog's `min`/`max` clip is the correct
*implementation* because it is exact when a zone boundary falls between samples and reduces to
IP's half-weight rule when it falls on one. On IP's own fixture the models differ by 0.25 ft
(3.25 vs 3.0), bounded at ½ step per zone contact with opposite signs at the two contacts.

**As-built.** `PRESENT-DIVERGENT` — TOPS-with-clip only, hard-coded, and implemented three times
independently: `workflow.rs:973-983` and `workflow.rs:1067-1074` (pay summary),
`workflow.rs:1476-1483` (sweep), `montecarlo.rs:1046-1052` (Monte Carlo). Magnitude of the
divergence: one half-step per zone-boundary contact, i.e. 0.25 ft on a 0.5 ft grid.

**Verified by.** SB-CUT-T01, SB-CUT-T02, SB-CUT-T02b, SB-CUT-T03, SB-CUT-T03b, SB-CUT-T03c

#### SB-CUT-002 — Name the discretisation model on every thickness-bearing result [P1] [status: ABSENT]

**Requirement.** Every result record carrying a thickness, a net, a net-to-gross or a
thickness-weighted average MUST carry the discretisation model that produced it and the sample
interval it was computed on. A consumer MUST NOT have to infer either.

**Rationale.** F-4. IP ships **two** different definitions of "Net" in one product — Cut-off &
Summation's half-weight rule and Curve Statistics' `count × step` — under the same column heading,
and labels neither (T2). A summation number without its discretisation model is not reproducible.
The sample interval is required separately because net-to-gross is **not scale-invariant** (T4:
0.55 → 0.75 → 1.0 across three blocking steps).

**As-built.** `ABSENT` — `PaySummaryRow` (`workflow.rs:855-895`) has no such field and the report
names no model.

**Verified by.** SB-CUT-T02b, SB-CUT-T31

#### SB-CUT-003 — Partition gross footage four ways [P1] [status: PARTIAL]

**Requirement.** A summation MUST report `Gross`, `Net`, `NotNet` and `Unknown` as four separate
quantities satisfying `Gross = Net + NotNet + Unknown` exactly. `Unknown` MUST be the footage
whose flag could not be evaluated (null input, non-positive clipped interval); it MUST NOT be
folded into `NotNet`.

**Rationale.** F-15. Techlog books a non-positive clipped interval as UNKNOWN, distinct from
NOT-NET (T1a L1050–1052); IP marks nulls in-band with `$$` inside a numeric column (T2). Only the
four-way partition is auditable.

**As-built.** `PARTIAL` — gross is set from the zone (`workflow.rs:1097`) and net is accumulated
(`:1098-1117`), so the invariant holds by construction, but there is no `NotNet` and no `Unknown`.

**Verified by.** SB-CUT-T11, SB-CUT-T22

#### SB-CUT-004 — Report net-to-gross both with and without the unknown footage [P2] [status: ABSENT]

**Requirement.** A summation MUST report both `N:G = Net/Gross` and `N:(G−Unknown)`, each labelled.

**Rationale.** F-15 (T1a). The two differ by exactly the null fraction. Over a washed-out or
partially-logged interval the difference is the whole argument about whether a net-to-gross is
defensible, and no incumbent surfaces both.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T11

#### SB-CUT-005 — Reconcile the footage partition with a named tolerance and a recorded residual [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST check `Gross − (Net + NotNet + Unknown)` against a named relative
tolerance. Within tolerance the residual MUST be absorbed into the largest component **and the
absorbed amount MUST appear in the result record**. Outside tolerance the summation MUST fail with
a structured error.

**Rationale.** F-15 and the dossier's §3.13: Techlog's `adjustFinal` has the right shape but
reports the residual with a `print` statement, so it is lost from the result. A reconciliation
whose correction is not recorded is indistinguishable from no reconciliation.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T22

#### SB-CUT-006 — Implement averaging as a generalised power mean with an explicit exponent [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST implement zone averaging as `M_p = (Σw·C^p / Σw)^(1/p)` with `p`
an explicit per-curve parameter, MUST implement the `p → 0` limit as the weight-normalised
geometric mean, and MUST record `p` on the result. `p = 1` MUST be the default.

**Rationale.** T1b Geolog's `POWER_MEAN_EXP`. One code path subsumes arithmetic (`p = 1`),
harmonic (`p = −1`), geometric (`p → 0`) and Techlog's third-power (`p = 1/3`), so there is one
implementation to test rather than four. All three vendors default extra-curve averaging to
arithmetic, so `p = 1` is the defensible default; Geolog's shipped `POWER_MEAN_EXP = 3` is **not**
adopted.

**As-built.** `ABSENT` — arithmetic only (`workflow.rs:1098-1117`).

**Verified by.** SB-CUT-T05

#### SB-CUT-007 — Compute the geometric average in weight-normalised form, with a non-positive guard [P2] [status: ABSENT]

**Requirement.** The geometric average MUST be `exp(Σhᵢ·lnCᵢ / Σhᵢ)`. SandiBumi MUST NOT implement
IP's `(ΠCᵢ)^(1/Σhᵢ)`. Samples with `Cᵢ ≤ 0` MUST be excluded from any log-domain average, the
excluded count MUST be reported, and the average MUST NOT early-return null because of them.

**Rationale.** F-2 (ledger D-5.2, RESOLVED). IP's form is unit-dependent: the same permeability log
returns 10 mD / 100 mD / 3.6 × 10⁶ mD at 1.0 ft / 0.5 ft / 0.1524 m steps (T3 vs T1a + T1b, two
independent correct implementations). Techlog's geometric path has no non-positive guard (T1a),
so one zero sample takes the result to `−inf`.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T04, SB-CUT-T09

#### SB-CUT-008 — Make the harmonic average skip non-positive samples rather than refuse the interval [P2] [status: ABSENT]

**Requirement.** The harmonic average MUST be `Σhᵢ / Σ(hᵢ/Cᵢ)` over flagged samples with `Cᵢ > 0`,
MUST report the count of skipped samples, and MUST NOT return null merely because some samples in
the interval are unflagged or non-positive.

**Rationale.** F-8 (T1a). Techlog's shipped `average()` early-returns `MissingValue` for the
harmonic case unless *every* sample in the interval is flagged — so a harmonic permeability average
over a partially-flagged zone silently returns nothing. Carried with the dossier's escalation-5
caveat: established for the shipped Python script, not the C++ GUI module.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T09

#### SB-CUT-009 — Key porosity-weighting off an explicit per-curve flag, never off the mnemonic [P1] [status: PARTIAL]

**Requirement.** Porosity weighting of an averaged curve MUST be controlled by an explicit
per-curve flag stored with the curve's averaging configuration. SandiBumi MUST NOT infer it from
the curve's name or family.

**Rationale.** T3, Techlog: *"the SW curve is weighted by POR but the SWE is not weighted"* — a
curve named `SWE` silently loses its φ-weighting because of its mnemonic. The φ-weighted form
`Σ(Sw·φ·h)/Σ(φ·h)` is agreed by all three vendors and is required for the volumetric identity of
`SB-CUT-010` to hold at all.

**As-built.** `PARTIAL` — the φ-weighted form is implemented correctly at `workflow.rs:1113`, but
it is hard-wired to the saturation slot: it cannot be requested for another curve and cannot be
switched off.

**Verified by.** SB-CUT-T06, SB-CUT-T07

#### SB-CUT-010 — Hold the volumetric identity between summed and reconstructed hydrocarbon pore volume [P1] [status: PRESENT-UNVERIFIED]

**Requirement.** `HCPV` computed by direct summation `Σφ(1−Sw)h` MUST equal `Net × φ̄ × (1 − S̄w)`
computed from the reported averages, to floating-point tolerance, for every emitted zone.

**Rationale.** T1a + T2 + T1b — the identity is the reason φ-weighted saturation is not optional.
It holds only with φ-weighted Sw and fails with thickness-weighted Sw, so testing it locks both
design choices together and makes a future regression in either one visible.

**As-built.** `PRESENT-UNVERIFIED` — both sides are computed (`workflow.rs:1098-1117`;
`montecarlo.rs:800-816`) and the arithmetic is consistent by inspection, but no test asserts the
identity.

**Verified by.** SB-CUT-T07

#### SB-CUT-011 — Exclude samples outside every zone from cumulative results [P1] [status: PRESENT-OK]

**Requirement.** A sample that passes every cut-off but lies outside every defined zone MUST NOT
contribute to any cumulative curve or summary statistic.

**Rationale.** T2, IP's stated rule. It is easy to violate in a single-pass implementation that
applies cut-offs before zone membership.

**As-built.** `PRESENT-OK` — the zone clip at `workflow.rs:1067-1074` runs inside the per-zone
loop and drops `h ≤ 0`; wells without zones get one whole-well `ALL` zone (`workflow.rs:921`).

**Verified by.** SB-CUT-T10

#### SB-CUT-012 — Treat the reference frame as part of a result's identity [P2] [status: ABSENT]

**Requirement.** A summation result MUST carry `{frame, weights_source}` where `frame` is one of
MD, TVD, TVDSS or TST. MD and TVD summations MUST be **separate records**. SandiBumi MUST NOT
present a TVD result as a rescaling of an MD result.

**Rationale.** F-24 (T2, IP: zonal averages are weighted by vertical thickness per depth
increment, so TVD averages *"could be considerably different"*). The per-sample weight is `Δz` in
MD and `Δz·cos θ` in TVD — in a 60° hold section the weights differ by a factor of two, so the
thickness-weighted average of *any* curve differs. Techlog offers four frames, IP two.

**As-built.** `ABSENT` — the summation is MD-only and no frame is recorded.

**Verified by.** SB-CUT-T08

#### SB-CUT-013 — Model bed amalgamation with three independent thresholds [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST support minimum bed thickness, maximum separation between beds to
be merged, and maximum non-net thickness that may be included inside a merged bed, as three
independent parameters, and MUST record all three on the result.

**Rationale.** F-16 (T1b, Geolog `LUMP_SATHK` / `LUMP_MAXSEP` / `LUMP_INCTHK` — the only vendor
that models more than a minimum height). It is the only model that survives laminated sand, which
is the regime this domain's delivered work lives in.

**As-built.** `ABSENT` from the summation path. `modules.rs:1397-1428` has a `MIN_THICK`
spike-removal for conditioning flags only; it does not touch `FLAG_PAY`.

**Verified by.** SB-CUT-T31

#### SB-CUT-014 — Emit bed statistics twice, pre- and post-amalgamation [P2] [status: ABSENT]

**Requirement.** A summation MUST emit interval count, total net, mean / thinnest / thickest bed
thickness **both before and after** amalgamation, in the same result, with the amalgamation
thresholds recorded alongside.

**Rationale.** F-16 + F-4 (T2 IP's Detail Interval Breakdown, extended). Amalgamation changes the
interval count and both thickness extremes while leaving total net unchanged, so a single copy of
the block cannot be interpreted. `Total Net / Total Intervals` is the number that says whether a
net figure is one clean sand or forty laminae. No vendor emits both.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T31

#### SB-CUT-015 — State the reported bed-thickness convention explicitly [P2] [status: ABSENT]

**Requirement.** The definition of a reported bed thickness MUST be an explicit, recorded
convention. A bed one sample thick MUST NOT report zero thickness.

**Rationale.** F-4 (T2 IP `Bottom − Top + step` vs T1a Techlog `Depth[Bottom] − Depth[Top]`). The
Techlog convention returns **0.0** for a single-sample bed. This is the real thin-bed hazard in
this domain — not the summation divergence of F-3, which is bounded and partly self-cancelling.

**As-built.** `ABSENT` — no bed-level reporting exists.

**Verified by.** SB-CUT-T31

### 4.2 Cut-off semantics

#### SB-CUT-016 — Ship no cut-off value [P0] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST NOT ship a numeric default for any cut-off. Every cut-off field
MUST ship in the first-class state `no default — user must set`, an unfiltered summation MUST be
reported **as unfiltered** on the result and in the report, and a summation MUST NOT run against
an unset cut-off that has been enabled.

**Rationale.** F-1, CONTRACT §2, `03_EVIDENCE_BASE.md` §12.2 (*"Where vendors disagree and no
adjudication is defensible, the parameter ships absent, not defaulted"*), and `SB-CORE-004`, which
names this domain's cut-off defaults as its worked example. Four shipped vendor sets, no two identical,
**two of them from one vendor** (T1b/T2): IP φ 0.1 / Sw 0.5 / Vsh 0.5; Techlog 0.15 / 0.85 / 0.5;
Geolog `default_*.paysum` 0.08 / 0.5 / 0.3; Geolog `determin_mc.info` 0.08 / 0.5 / **0.5**; Geolog
`tp_pay_sensitivity` permissive. Delivered work (T4/P) spans Vsh 0.20–0.85, PHIE 0.05–0.27, Sw
0.50–0.90, and one record spans Vsh 0.55–0.85 **across intervals of a single area** — so the
quantity is not constant even within one field. Picking one vendor's number over four others is
adjudication disguised as a default.

**As-built.** `PRESENT-DIVERGENT` — six panes present 0.5 / 0.1 / 0.6 as defaults
(`src/ui/cutoffs.ts` for five of them, `src/ui/dashboardPanel.ts:56-59` for the sixth) with no
source. §3.2.

**Verified by.** SB-CUT-T36

#### SB-CUT-017 — Carry a source string on every default [P0] [status: ABSENT]

**Requirement.** Every default value SandiBumi ships in this domain MUST carry a machine-readable
source string identifying the file, section or citation it came from. A default with no source
MUST fail the build, and a module requiring a source-less parameter MUST refuse at run time with an
actionable message.

**Rationale.** `SB-CORE-004` — this is its domain-level discharge, and the build gate is the
requirement rather than an implementation detail: the difference between a convention and a
contract is whether a machine enforces it. Also CONTRACT §2 and the dossier's §5.3 item 4
(FINDINGS §6 rule 9: vendor dialogs show
demo values indistinguishable from defaults — IP's Report 5 column is unconfigured while Reports
1–4 carry values, the signature of a shipped set partially overwritten by demo content). A number
whose provenance is not stored is a number nobody can defend in a client review.

**As-built.** `ABSENT` — no default in this domain carries a source. The four repository hits for
0.5 / 0.1 / 0.6 record only what the values were unified *to*. §3.2.

**Verified by.** SB-CUT-T36

#### SB-CUT-018 — Resolve every cut-off entry point from one authority [P0] [status: PARTIAL]

**Requirement.** Every user-facing surface that accepts or displays a cut-off MUST resolve it from
a single shared module. No pane may hard-code a cut-off literal. A test MUST enumerate the panes
and fail when one bypasses the authority.

**Rationale.** `SB-CORE-004` names this as the thinnest visible place in the parameter-source
discipline, and the drift is documented in SandiBumi's own source. **Two disagreeing sets were
copy-pasted across six panes:** VSH ≤ 0.5 / PHIE ≥ **0.08** / SWE ≤ **0.5** in
`monteCarloDialog.ts` and `resultsQcPanel.ts`, against VSH ≤ 0.5 / PHIE ≥ **0.1** / SWE ≤ **0.6**
in `cutoffDialog.ts`, `summaryDialog.ts`, `reportDialog.ts` and `dashboardPanel.ts` — **and the
Monte Carlo settings tooltip claimed "Cutoffs match the pay summary" while they did not**
(`src/ui/cutoffs.ts` doc comment). **Neither set has a documented source**, and this chapter does
not adjudicate between them: §5 records both and specifies the refusal, per `03_EVIDENCE_BASE.md`
§12.2. A cut-off divergence between the number a user sets and the number a study runs at produces
a plausible wrong answer, not an error.

**As-built.** `PARTIAL` — five of six panes import `loadCutoffDefaults()`;
`src/ui/dashboardPanel.ts:56-59` hard-codes literals and does not import `./cutoffs`. It agrees
with the authority today by coincidence of transcription and is protected by nothing.

**Verified by.** SB-CUT-T35

#### SB-CUT-019 — Require a unit on cut-off entry [P1] [status: ABSENT]

**Requirement.** A cut-off value MUST be entered with a unit and stored with it. A bare number
MUST be rejected. `35 pu` MUST be accepted and stored as `0.35 v/v`; `35 v/v` MUST be rejected as
out of bounds. Dimensionless cut-offs MUST be bounded to their quantity's physical range;
thickness thresholds MUST carry the project depth unit.

**Rationale.** F-19 (T2): IP's own manual expresses the sensitivity-sweep example in porosity units
and the cut-off default in v/v, for the same quantity, with no unit tag on the field. `35` where
`0.1` is meant is a **350×** error whose symptom is an all-net result — a good-looking well, not a
visible failure. F-22 adds the converse: Geolog's metric and imperial spec pairs differ *only* in
the thickness units, so a unit error there is invisible on every quantity except footage.

**As-built.** `ABSENT` — `src/ui/cutoffDialog.ts:104-111` builds a bare `type="number"
step="any"` input with no `min`, no `max` and no unit; the label reads `PHIE ≥` and nothing else.
`monteCarloDialog.ts:464-467` bounds to `[0,1]` but is equally unit-free.

**Verified by.** SB-CUT-T26

#### SB-CUT-020 — Express a cut-off as a two-sided range with an explicit bounds operator [P2] [status: ABSENT]

**Requirement.** A cut-off MUST be expressible as a two-sided range with an explicit operator
selecting the inclusivity of each bound. The single-sided `≥` / `≤` forms MUST be the degenerate
case with an open far bound. Every operator's boundary behaviour MUST be tested against
SandiBumi's own written specification.

**Rationale.** T1a Techlog `limitType`, which is strictly more general than IP's single-sided form
— but whose shipped implementation is also the warning: modes 4/5/6 raise `NameError`, mode 7 is a
silent always-pass (a configured cut-off that filters nothing), and modes 2/3 are **documented as
outside tests and implemented as inside tests**. A boundary convention that is not tested against
its own spec is a coin flip at every sample sitting exactly on the cut-off — which, per F-6, is
precisely the population that decides a marginal-pay result.

**As-built.** `ABSENT` — the three cut-offs are fixed single-sided comparisons
(`workflow.rs:1059-1096`).

**Verified by.** SB-CUT-T24

#### SB-CUT-021 — Allow a cut-off to be supplied as a curve [P3] [status: ABSENT]

**Requirement.** A cut-off value MUST be able to reference a curve as well as a scalar, so a
cut-off may vary with depth.

**Rationale.** T1b Geolog `CUTOFF_IS_LOG`. It expresses a facies- or rock-type-varying cut-off,
which neither competitor can state at all. Seam: `SHR` supplies the rock-type curve.

**As-built.** `ABSENT` — `PaySummaryRequest` takes scalars only.

**Verified by.** SB-CUT-T24

#### SB-CUT-022 — Make cut-off activation an explicit flag, and share one value across the reservoir and pay tiers [P1] [status: PARTIAL]

**Requirement.** Each cut-off MUST carry an explicit enable flag per report tier. Activation MUST
NOT be inferred from the presence of a curve or of a value. The reservoir and pay tiers MUST share
**one value** with **two independent use flags**; additional report tiers MUST each carry their
own value.

**Rationale.** F-17 (T3): Geolog changed the activation trigger between two modules of one product
— `Determin` activates on the presence of the *curve*, `determin_mc` on the presence of the
*value*. An inferred rule cannot be audited from a result. F-25 (T1b `Cutoff.hlp` ordinals 1–3:
`Phi Net Use`, `Phi Pay Use`, `Phi Cutoff`, the last described as *"Porosity cutoff value for Pay
and Reservoir report"*) settles the shared-value question that a prior record said needed a live
IP session.

**As-built.** `PARTIAL` — the PERM cut-off has an explicit inactive state and a distinct
`perm_cutoff_no_data` flag (`workflow.rs`), but VSH / PHIE / SWE are always active and there are no
per-tier use flags.

**Verified by.** SB-CUT-T24, SB-CUT-T36

#### SB-CUT-023 — Evaluate cut-off criteria as a boolean expression [P3] [status: ABSENT]

**Requirement.** Cut-off criteria MUST be evaluable as a boolean expression over cut-off
predicates supporting AND, OR, NOT and parentheses, defaulting to AND-of-all-enabled.

**Rationale.** T4 Bentley & Ringrose's own worked net-reservoir rule is
`IF Gamma < 40 API AND (Poro > 0.05 OR Perm > 0.1 mD)` — an **OR** inside the criterion. All three
vendors implement AND-of-all and nothing else, so the one literature rule this dossier treats as
authoritative on net definitions is not expressible in any of them and a petrophysicist wanting it
must build a flag curve outside the module. The default preserves the common case and lets vendor
imports round-trip exactly.

**As-built.** `ABSENT` — the conjunction is hard-coded.

**Verified by.** SB-CUT-T32

#### SB-CUT-024 — Support arbitrary named flag tiers over arbitrary cut-off sets [P2] [status: PARTIAL]

**Requirement.** SandiBumi MUST support an arbitrary number of named flag tiers, each over its own
set of cut-offs, and MUST ship net sand / net reservoir / net pay as the default three.

**Rationale.** T1a + T1b (Techlog and Geolog both allow arbitrary named flags; IP caps at five
reports) backed by T4 (Bentley & Ringrose's net sand → net reservoir → net pay is the literature
model). IP's cap has no physical basis.

**As-built.** `PARTIAL` — exactly three tiers, hard-coded (`workflow.rs:1005`, `:1030-1032`), with
the correct tier definitions.

**Verified by.** SB-CUT-T36

#### SB-CUT-025 — Treat lumps as a many-to-one reporting transform over flags [P3] [status: ABSENT]

**Requirement.** A lump MUST be a reporting transform applied over a flag curve, and one flag MUST
be able to feed more than one lump under different rule sets, each with its own curve-reporting
configuration.

**Rationale.** T1b Geolog `SANDA` / `SANDB` lump one flag under two rule sets. It separates "what
is net" from "how net is aggregated for reporting", which is what makes a sensitivity over
amalgamation thresholds possible without re-running the cut-off pass.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T31

#### SB-CUT-026 — Leave saturation off at the reservoir tier by default [P1] [status: PRESENT-OK]

**Requirement.** The net reservoir tier MUST NOT apply a saturation cut-off by default. Net
reservoir MUST be porosity- and clay-driven; saturation MUST enter at the pay tier.

**Rationale.** F-25 (T1b `Cutoff.hlp`: `Sw Net Use` and `Sw Pay Use` are separate ordinals and Net
Reservoir is described as porosity- and clay-driven). Ledger D-5.10 — a default the vendor never
states plainly, and getting it wrong reclassifies wet reservoir as non-reservoir.

**As-built.** `PRESENT-OK` — `FLAG_RESERVOIR` applies VSH + PHIE only (`workflow.rs:1030-1032`).

**Verified by.** SB-CUT-T36

#### SB-CUT-027 — Impose no cap on curves, cut-offs, report tiers or flags [P2] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST NOT impose a fixed maximum on the number of input curves,
cut-offs, report tiers or flag curves.

**Rationale.** Ledger D-5.4 (T1b): IP's parameter model stops at Curve 10, its 2025 prose claims
50, and IP2018's "up to 10 input curves … the additional 7" was correct — the 2025 edit introduced
the error. All of these are vendor implementation limits with no physical basis, and SandiBumi
should inherit neither the caps nor the confusion.

**As-built.** `PRESENT-OK` — no cap exists, because the surface is not yet general enough to have
one. This becomes a live constraint when `SB-CUT-024` is built.

**Verified by.** SB-CUT-T36

#### SB-CUT-028 — Emit `SWE` and `SWT`, never a bare `SW` [P1] [status: PRESENT-OK]

**Requirement.** Saturation quantities MUST be named `SWE` or `SWT` explicitly wherever a cut-off,
an average or a result field refers to one. A bare `SW` MUST NOT appear in a cut-off record or a
result field.

**Rationale.** FINDINGS §6 rule 8, sharpened by T3: in Techlog the mnemonic silently changes the
weighting (*"the SW curve is weighted by POR but the SWE is not"*), so a bare name is both
ambiguous about the flavour and load-bearing on the arithmetic.

**As-built.** `PRESENT-OK` — `swe_max`, `avg_swe`, `SWE` throughout
(`workflow.rs:855-895`, `montecarlo.rs:722-728`).

**Verified by.** SB-CUT-T06

#### SB-CUT-029 — Carry null markers as typed sibling fields [P1] [status: PRESENT-OK]

**Requirement.** A null or not-computed condition MUST be carried in a typed sibling field
(`has_nulls: bool`, `null_count: int`, or an explicit classified-sample count), never as an in-band
marker inside a numeric field. Consumers MUST render an em-dash rather than a zero when the count
is zero.

**Rationale.** F-15 (T2): IP prints `$$` inside a numeric report column to mean "nulls present" —
unparseable, uncarryable through a calculation, and invisible to a downstream consumer that reads
the column as a number.

**As-built.** `PRESENT-OK` — `n_classified` with the rendering contract stated at
`workflow.rs:865`, `perm_cutoff_no_data` alongside it, and `report.rs:507-512` honouring both. The
remaining work is the footage partition of `SB-CUT-003`, not the marker discipline.

**Verified by.** SB-CUT-T37

#### SB-CUT-030 — Separate the accumulate, flag-test and present stages of clamping [P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST implement three named pipeline stages: `accumulate` (never
clamped), `flag_test` (clamped to the quantity's bounds) and `present` (clamped). Bounds MUST
attach to the **quantity**, never to a curve-type string. An unbounded quantity MUST NOT be clamped
to `[0,1]`. A zonal average falling outside its bounds MUST be emitted with `out_of_range: true`,
not corrected. Percent-to-fraction conversion and the bound check MUST be separate operations, and
an over-bound value after conversion MUST raise.

**Rationale.** F-7 (T3 Geolog states the policy and its reason; T1a Techlog clamps unconditionally
at four sites; T2 IP clips by *declared curve type*, so mis-typing a curve silently changes its
numerics). Binding bounds to a type string is the specific failure that makes IP's version worse
than Techlog's: it is invisible in the data.

**As-built.** `PARTIAL` — limited and unlimited companion curves both exist and the plausibility
diagnostic reads the unlimited ones (`montecarlo.rs:1276`, `:1385`), but the accumulator reads the
limited ones (`montecarlo.rs:758-816`), there is no stage separation, no `out_of_range` flag on an
emitted average, and bounds are attached at module output. §3.8.

**Verified by.** SB-CUT-T15, SB-CUT-T23, SB-CUT-T25

### 4.3 Monte Carlo

#### SB-CUT-031 — Make the shift-to-σ multiple explicit and mandatory, and set it to 2 for IP-sourced widths [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Every Gaussian prior MUST carry an explicit `SD_MULT` field giving the number of
standard deviations the tabulated width spans, and the realised σ MUST be `w / SD_MULT`. The field
MUST be mandatory — a prior with no `SD_MULT` MUST be rejected, never defaulted. An IP-sourced
width MUST use `SD_MULT = 2`. `docs/ref_monte_carlo_seeds.md` MUST be corrected, and its stated
premise that the convention is undocumented MUST be withdrawn.

**Rationale.** F-5. Three vendors, three conventions from one typed digit: IP `Lo+Hi = 4σ` ⟹
σ = w/2 (T2, stated on its Monte Carlo page and corroborated by its ±2σ tornado landing exactly on
the tabulated Low/High edge); Geolog `σ = Shift/SD`, `SD = 3` ⟹ σ = w/3 (T1b + T3); Techlog σ
supplied directly ⟹ σ = w (T1a). Applied to `Rw` the σ values are 1.67 % and 10.0 % — a **6.0×**
gap on the parameter with the largest single influence on Archie saturation.

**As-built.** `PRESENT-DIVERGENT` — `docs/ref_monte_carlo_seeds.md:50` adopts σ = w and
`src/ui/monteCarloDialog.ts:236-240` passes the seeded width straight through as σ. **All twelve
seeded Gaussian priors are exactly 2× too wide.** `M` seeded at `w = 0.2`
(`monteCarloDialog.ts:70`) realises σ = 0.20 against IP's σ = 0.10, and each row carries a muted
`IP` badge (`:95-102`) asserting a provenance the number does not have. §3.9. **One-line fix; the
regression test is the deliverable.**

**Verified by.** SB-CUT-T13

#### SB-CUT-032 — Store the shift type with the width, and refuse to coerce a reciprocal shift [P1] [status: ABSENT]

**Requirement.** A prior MUST store its shift type (`Linear`, `%`, `Rec`) alongside its width. An
import carrying a `Rec` shift into a target with no reciprocal sampler MUST be a **load error**.
SandiBumi MUST NOT silently coerce a `Rec` shift to `Linear`.

**Rationale.** F-14 (T1b). `Result = 1/(1/Input + Shift)` at `Shift = 0.005` is ±0.5 % at
`Rt = 100` and **−33 % / +100 % at `Rt = 1`**. Coercing it to a linear ±0.005 ohm-m shift is
negligible in high-resistivity test data and catastrophic in a water leg — the shape of the prior
changes, not its units. Neither Techlog nor Geolog implements a reciprocal shift, so the coercion
is the only path an importer would otherwise take.

**As-built.** `ABSENT` — `IP_MC_SEEDS` (`src/ui/monteCarloDialog.ts:68-81`) carries `{ w, pct }`
only; `Distribution` (`montecarlo.rs:46-50`) has no reciprocal variant.

**Verified by.** SB-CUT-T20, SB-CUT-T30

#### SB-CUT-033 — Import measurement priors, not only parameter priors [P2] [status: ABSENT]

**Requirement.** The Monte Carlo import MUST cover input-curve (measurement) priors as well as
model-parameter priors, each with its unit and shift type.

**Rationale.** Dossier §4.2 correction 4 (T1b). The `InCurves` block is the only part of IP's
shipped defaults file that describes *log* uncertainty, and a Monte Carlo over a **cut-off** module
needs exactly that, because cut-offs act on curves. The block also refutes IP's own manual: its
"±10 %" sentence is wrong for **all seven** input curves and for all but two parameter rows.

**As-built.** `ABSENT` — `docs/ref_monte_carlo_seeds.md:37-39` excludes the block deliberately.

**Verified by.** SB-CUT-T20

#### SB-CUT-034 — Make the seed mandatory and part of the result record [P1] [status: PRESENT-OK]

**Requirement.** The Monte Carlo seed MUST be a required input and MUST be stored on the result
record, not held as a UI setting. Re-running with the same seed MUST reproduce a bit-identical
result.

**Rationale.** T3 Geolog's Module Launcher `Seed` field — *"Entering a Seed value ensures that the
results from run to run are deterministic"*. IP and Techlog have none. Per the dossier's §4.2
correction 2 the correct claim is **"matches Geolog, beats IP and Techlog"** and any positioning
material MUST use that wording.

**As-built.** `PRESENT-OK` — `src/ui/monteCarloDialog.ts:461` is a required field;
`montecarlo.rs:1730` persists it in the `MONTECARLO` log set's `params_json` with the iteration
count, sampling mode and percentile pair.

**Verified by.** SB-CUT-T12

#### SB-CUT-035 — Provide log-domain distributions [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide log-normal, log-triangular and log-uniform distributions
alongside normal, triangular and uniform.

**Rationale.** T3 Geolog's Module Launcher offers six; IP and Techlog offer three. Permeability and
`Rw` are the parameters that need them — a symmetric prior on a quantity spanning orders of
magnitude puts probability mass on negative values and is truncated rather than modelled.

**As-built.** `ABSENT` — `montecarlo.rs:46-50` is Normal / Uniform / Triangular.

**Verified by.** SB-CUT-T14

#### SB-CUT-036 — Ship every prior as a (value, basis, sigma-multiple) triple with units [P1] [status: PARTIAL]

**Requirement.** An uncertainty prior MUST be stored as an explicit `(value, basis, sigma_multiple,
unit)` record. `basis` MUST distinguish absolute from relative. SandiBumi MUST NOT store a prior as
a bare number.

**Rationale.** F-13 (T1b). Geolog names its basis in the identifier (`ERR_PC_RW 5` = 5 %
*percentage* error) and IP names its shift type in a separate column; comparing the two shows
**exactly one row agreeing exactly** across the whole table (neutron, 5 %), with `Rw` 4× apart on
width and 6.0× apart on σ, `Rho Matrix` ~2× apart, and `Rt`/`Rxo` **not comparable at all**. A
bare number cannot express any of those distinctions and is the mechanism by which they get
silently conflated.

**As-built.** `PARTIAL` — `McSeed { w, pct }` (`src/ui/monteCarloDialog.ts:58-61`) carries the
relative/absolute basis but neither the σ multiple (`SB-CUT-031`) nor the unit nor the shift type
(`SB-CUT-032`).

**Verified by.** SB-CUT-T13, SB-CUT-T21

#### SB-CUT-037 — Store the centring rule per prior [P2] [status: ABSENT]

**Requirement.** Each prior MUST store its centring rule explicitly. SandiBumi MUST NOT infer the
centring from the distribution's name, and where a prior is asymmetric the reported P50 and the
base case MUST both be surfaced when they differ.

**Rationale.** Under an asymmetric shift the three vendors do three different things (T2/T1a/T3):
IP centres the Gaussian mean on the Start Value; Techlog's `np.random.triangular(lower, val,
upper)` anchors the **mode** at the unperturbed value; Geolog anchors the **mean at the range
mid-point** — *"The mid-point between min and max values is used as the mean"* — so the perturbed
mean is not the analyst's value. Geolog publishes the consequence: for `a=1, c=2, b=6`, mode 2.0,
mean 3.0, median **2.84**, with a named warning that P50 ≠ base case.

**As-built.** `ABSENT` — `Distribution::Triangular` takes `(lo, mode, hi)` with no recorded
centring semantics.

**Verified by.** SB-CUT-T28

#### SB-CUT-038 — Truncate Gaussian draws and report the resulting variance deficit [P2] [status: ABSENT]

**Requirement.** Gaussian sampling MUST support truncation at a configurable number of standard
deviations, MUST resample rather than clip when a draw falls outside, and MUST report the realised
variance deficit against the nominal σ².

**Rationale.** T2 IP: draws are *"limited to 2.5 standard deviations either side of the Mean
value… another random number will be chosen"*, identical 2018→2025, and IP's own 2025 material
warns the realised variance is biased low. Truncating without reporting the deficit understates
the spread the run claims to have sampled. IP's stated purpose for the truncation is also what
keeps a `Rec`-shifted resistivity positive (`SB-CUT-032`).

**As-built.** `ABSENT` — no truncation in `montecarlo.rs:393-427`.

**Verified by.** SB-CUT-T14

#### SB-CUT-039 — Set the iteration default from a cited source and auto-stop on the reported percentile [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The iteration default MUST carry a source. Auto-stop, where enabled, MUST evaluate
its tolerance on the **percentile that will actually be reported**, not on the mean alone, and the
achieved iteration count MUST be recorded on the result.

**Rationale.** F-6. IP 2000 with a documented auto-stop architecture (burn-in 200, check every 100,
minimum 300, tolerance 0.1 % on P10, P50, P90 *and* mean simultaneously — T2); Geolog 250 against
its own 1,000–5,000 recommendation; Techlog 20 (T1a). Geolog's published convergence experiment
(10 / 750 / 5,000 / 10,000) shows convergence is set by the **marginality of the pay**, not by
parameter count — a well sitting on the porosity cut-off does not converge where a clear well does.
That is the permanent regime for thin-bed work.

**As-built.** `PRESENT-DIVERGENT` — the default is 1,000 (`src/ui/monteCarloDialog.ts:460`) with no
recorded source. `CONV_MIN_ITER = 200` (`montecarlo.rs:672`) floors the auto-stop and
`used_iterations` / `requested_iterations` are both on the result (`:335-336`), so the recording
half is already met.

**Verified by.** SB-CUT-T33

#### SB-CUT-040 — Draw one offset per section per iteration [P1] [status: PRESENT-OK]

**Requirement.** Monte Carlo perturbation MUST apply one offset per section per iteration
(`VERTICAL`). SandiBumi MUST NOT draw independently per depth sample for any quantity that feeds a
summation.

**Rationale.** F-21 (T3 + T1a). Geolog implemented, tested and **dropped** horizontal processing
because it *"does not effectively model systematic uncertainties"*. Independent per-depth draws
average out as **1/√N** over a summation interval: over a 100-sample zone the reported spread is
10× too narrow, and it narrows further the more finely the well is logged. Techlog mixes both
regimes inside one module — per-depth for inputs (L1216–1218), per-zone for cut-offs
(L1599–1601).

**As-built.** `PRESENT-OK` — `build_draws` (`montecarlo.rs:429`) produces one value per parameter
per realization and `resolve_zone_param` (`:731`) broadcasts it across the section. The regime is
correct but is a consequence of the architecture rather than a named, tested choice — the test
makes it explicit.

**Verified by.** SB-CUT-T27

#### SB-CUT-041 — Never clamp before accumulation [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Zone averages and volumetric accumulations MUST be computed on unclamped values.
A zone average outside its physical bounds MUST be emitted with `out_of_range: true` and MUST NOT
be corrected.

**Rationale.** F-7. Clamping a wet interval at `Sw = 1` shifts the zonal mean by
`−σ/√(2π) = −0.3989σ`, **independent of iteration count**, always toward more hydrocarbon —
≈ **4 saturation units** of manufactured hydrocarbon at Techlog's shipped σ = 0.1 v/v. Geolog is
the only vendor that states the policy and its reason: *"the data used to compute zone averages are
the unlimited versions … to ensure that there is no bias at the edges of the scales"* (T3). The
bias cannot be found by running longer, which is why it needs a requirement rather than a QC step.

**As-built.** `PRESENT-DIVERGENT` — `zone_metrics` (`montecarlo.rs:758-816`) accumulates from the
limited curves. The in-code rationale at `:344-350` is correct for a single deterministic
evaluation and wrong in expectation over an ensemble. Exposure is masked (not removed) by the
pay-flag restriction at `:800` whenever `swe_max < 1`, and is fully live under a permissive
cut-off set. §3.8.

**Verified by.** SB-CUT-T25, SB-CUT-T15, SB-CUT-T23

#### SB-CUT-042 — Perturb cut-offs under Monte Carlo, per zone [P2] [status: ABSENT]

**Requirement.** Cut-off values MUST be perturbable under Monte Carlo, with the draw taken **per
zone per iteration**, and the realised cut-off set MUST be recorded per iteration.

**Rationale.** T1b IP's `Cutoff` block ships 40 Monte Carlo shift rows (φ 0.05, Sw 0.2, Vcl 0.3 —
the only three carrying meaningful widths; every unnamed extra curve gets a placeholder `1`), and
T1a Techlog draws cut-offs per zone. T4/P: delivered work uses LOW/BASE/HIGH cut-off cases, so the
uncertainty is already being carried by hand.

**As-built.** `ABSENT` — `montecarlo.rs:1248` builds `Cutoffs` once from the request, outside the
realization loop.

**Verified by.** SB-CUT-T18

#### SB-CUT-043 — Compute derived ratios inside the iteration [P1] [status: PRESENT-OK]

**Requirement.** A derived ratio such as net-to-gross MUST be computed within each realization and
the percentile taken over the resulting distribution. SandiBumi MUST NOT divide one percentile by
another.

**Rationale.** F-9 (T1a): Techlog reports `statDictNTG[p] = ni[p]/gi[p]`. The two agree only when
gross is constant; the moment a zone boundary, a deviation survey or a TVD conversion carries
uncertainty they diverge, precisely in the tail statistic a reserves case is quoted from.

**As-built.** `PRESENT-OK` — `montecarlo.rs:812` computes `ntg` inside `zone_metrics`, inside the
realization.

**Verified by.** SB-CUT-T19

#### SB-CUT-044 — Store per-iteration joint records and report iteration-consistent percentile cases [P2] [status: ABSENT]

**Requirement.** Monte Carlo results MUST store per-iteration joint records, MUST report
iteration-consistent percentile **cases** (a percentile output is the value from the actual
iteration that produced the result at that percentile) with marginal percentiles available and
labelled as such, and MUST ship the iteration-consistent form **on** by default. The base case MUST
be a named iteration, not a synthetic average.

**Rationale.** T3 Geolog's `CDF_OP` already does exactly this — *"Each array element represents the
value for that output curve from the actual Monte Carlo iteration which gave the result at the
corresponding percentile"* — and `EHC_1P/2P/3P` are whole iterations. **The dossier explicitly
withdraws the earlier "beyond all three" framing** (§4.2 correction 8): SandiBumi's only additions
are shipping it on where Geolog ships `CDF_OP = FALSE`, and naming the base-case iteration. IP
documents the inconsistency and works around it by hand; Techlog commits it silently.

**As-built.** `ABSENT` — percentile arrays are marginal only; `kept_realizations` is a retention
cap (`montecarlo.rs:1266`), not a joint-record contract.

**Verified by.** SB-CUT-T16

#### SB-CUT-045 — Withhold a statistic whose preconditions fail [P1] [status: PARTIAL]

**Requirement.** A statistic whose sample size or binning cannot support it MUST be withheld with a
machine-readable reason. It MUST NOT be printed, MUST NOT be returned as `NaN`, and MUST NOT
silently degrade to the nearest order statistic. The minimum iteration count for a tail percentile
MUST be configurable and recorded.

**Rationale.** F-18 — CONTRACT §5 point 3 with a vendor precedent proving it shippable. Geolog
refuses `EHC_CDF` below `OPT_MC > 10`: *"Below this number the results are meaningless"* (T3). IP
prints a 50-cell modal statistic while its own page requires ≥3 values per cell (T2). Techlog
prints a P10 from 20 iterations (T1a).

**As-built.** `PARTIAL` — `summarize` (`montecarlo.rs:834`) returns all-NaN for a no-data metric
rather than a fabricated zero, which is the right instinct, and `CONV_MIN_ITER = 200` (`:672`)
gates the auto-stop. Neither gates *reporting*: a 5-iteration run still returns a P10.

**Verified by.** SB-CUT-T29

#### SB-CUT-046 — Name the percentile interpolation method on the output record [P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST choose one percentile interpolation method, MUST document it, and
MUST state it on every output record carrying a percentile.

**Rationale.** F-11 — unstated by all three vendors (T2 IP; T1a Techlog's `tlStat.percentile` is
inside a compiled library; T3 Geolog silent). The choice moves P10/P90 by up to one rank spacing:
negligible at IP's 2000 iterations, decisive at Techlog's 20, where the P10 sits between ranks 2
and 3. This closes the dossier's escalation 1, which is a decision rather than a lookup.

**As-built.** `PARTIAL` — type-7 linear interpolation is implemented and named in the source
(`montecarlo.rs:819-820`), but is not echoed onto `McResult`, so a consumer of the result cannot
tell which convention produced it.

**Verified by.** SB-CUT-T16

#### SB-CUT-047 — Report percentile cases as reserves categories with their actual probabilities [P2] [status: ABSENT]

**Requirement.** Percentile cases MUST be reported using reserves-category naming (1P / 2P / 3P)
with analyst-settable underlying probabilities, and the **actual probability MUST be printed on
every quoted case**. Each quantity MUST carry a `direction` (`higher_is_better` /
`lower_is_better`) in its metadata. SandiBumi MUST NOT use one global ascending/descending flag.

**Rationale.** F-12. Geolog names the ambiguity (*"some people consider P10 optimistic, others
pessimistic … certainly not universal usage"*), adopts SPE Reserves Booking Guidelines (2001)
terminology, and makes the report self-describing — *"The actual probabilities used are quoted in
the summary reports"* (T3). IP's convention lives in a `.par` file the report never echoes
(T1b: `Results = Yes 10 50 90 -999 -999`), and its single global flag **cannot express the Sw
carve-out IP's own manual describes**.

**As-built.** `ABSENT` — `low_pctl` / `high_pctl` are echoed numerically with no category naming
and no per-quantity direction.

**Verified by.** SB-CUT-T16

#### SB-CUT-048 — Merge cases, not statistics, when rolling percentiles up across zones [P3] [status: ABSENT]

**Requirement.** A multi-zone percentile roll-up MUST merge the per-zone iteration **cases**, not
the per-zone statistics, and MUST retain the unmerged per-zone iteration arrays so the merge can be
re-derived or audited.

**Rationale.** T3 Geolog's `OPT_ZONEMERGE` merges *"each 1P iteration for each zone"* — so a merged
1P curve may be iteration 25 in one zone spliced to iteration 10 in another — and `EHC_ALL`
deliberately stores the unmerged arrays. Summing P90s across zones is the classic
conservative-bias error and this is the documented alternative.

**As-built.** `ABSENT` — zones are summarised independently (`montecarlo.rs:1382`).

**Verified by.** SB-CUT-T16

#### SB-CUT-049 — Report the realised correlation alongside the requested one [P1] [status: PARTIAL]

**Requirement.** Correlation between priors MUST be induced in rank space and the **realised**
rank correlation MUST be reported alongside the requested one for every pair. Where the requested
correlation matrix is not attainable, SandiBumi MUST refuse and say why, and MUST NOT approximate.

**Rationale.** F-10 (T1a + T2): both IP and Techlog accept a "correlation" and implement it as a
blending weight, which does not deliver the requested coefficient in the realised sample — the
number the user asked for is not the number the run used, and neither tool says so. Techlog's
cut-off variant additionally crashes on a float list subscript.

**As-built.** `PARTIAL` — the induction half is `PRESENT-OK` and ahead of both vendors:
`iman_conover` (`montecarlo.rs:480`) with van der Waerden scores, the Spearman→Pearson
pre-adjustment `2·sin(πρ/6)` at `:525`, Cholesky re-colouring (`:595`), and two refusal gates
(non-positive-definite matrix, and fewer than 2 parameters or 10 iterations at `:469`). The
**realised**-correlation report is `ABSENT`.

**Verified by.** SB-CUT-T18

#### SB-CUT-050 — Re-derive data-picked parameters each iteration rather than correlating them [P3] [status: ABSENT]

**Requirement.** A parameter that was picked *from* the data — a shale point, an `m` or `Rw` from a
Pickett plot — MUST be re-derivable from the perturbed logs on each iteration, as an alternative to
assigning it an independent prior.

**Rationale.** F-10, second half (T3). Geolog's Automatic Parameter Adjustment exists because
independently perturbing a derived quantity *"leads to situations where combinations of porosity
and saturation which cannot exist in reality are included in the Monte Carlo results"*, and the
vendor states the naive alternative *"will lead to greater uncertainty in the results"* — i.e. it
over-states the spread. A copula handles correlated measurement error; only re-derivation handles
the fact that a data-picked parameter is not an independent input at all. Neither substitutes for
the other.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T18

#### SB-CUT-051 — Emit tornado bars in absolute output units [P2] [status: PARTIAL]

**Requirement.** Tornado bars MUST be emitted in the output quantity's absolute units. Any
percentage form MUST carry the iteration count it was normalised against. A tornado MUST NOT be
computed across merged zones.

**Rationale.** T3 Geolog measures bars *"in actual units of EHC or EPC"* and states why:
*"The computations of min/max EHC from the sensitivity analysis should not change from one run to
the next"* — only the Monte Carlo range in a percentage denominator moves with iteration count, so
a percentage tornado appears unstable when the underlying sensitivity is not. The no-merged-zones
rule follows because models and parameters can differ between intervals.

**As-built.** `PARTIAL` — the tornado runs one-at-a-time at each distribution's P10 / median / P90
with a Spearman rank correlation alongside, but neither the units contract nor the
iteration-count labelling is enforced on the emitted values.

**Verified by.** SB-CUT-T34

#### SB-CUT-052 — Ship perturbation off [P1] [status: PRESENT-DIVERGENT]

**Requirement.** A newly added uncertain parameter MUST ship with perturbation **disabled**. A
Monte Carlo run left entirely at defaults MUST produce zero spread, not invented spread.

**Rationale.** F-20. All six of Techlog's `_Uncertainty` flags ship `no` (T1a) and Geolog's
`determin_mc` ships `OPT_MC = 1` — a single iteration, Monte Carlo off — against a validation range
of `1:50000` (T1b). Two vendors independently chose the safe failure. An invented spread is worse
than no spread because it looks like an answer.

**As-built.** `PRESENT-DIVERGENT` — `defaultRow` (`src/ui/monteCarloDialog.ts:245-248`) constructs
every added parameter with `kind: "normal"` and a non-zero width from `distDefaults`, so a
parameter becomes uncertain the moment it is added. The width is either an IP seed (2× too wide,
`SB-CUT-031`) or a generic 10 % of value.

**Verified by.** SB-CUT-T15, SB-CUT-T23

#### SB-CUT-053 — Report physically impossible realizations, never exclude them [P1] [status: PRESENT-OK]

**Requirement.** Realizations whose sampled parameter combination drove the petrophysics outside
physical bounds MUST be counted and reported, MUST NOT be excluded from the percentile pool, and
MUST be distinguishable from "not checked" when the well produced no data to judge.

**Rationale.** Excluding them biases P10/P90 by truncating the pool asymmetrically; hiding them
removes the only signal that the input distributions are straining physics. A large impossible
fraction is a finding about the priors, not about the well. No incumbent surfaces it.

**As-built.** `PRESENT-OK` — `McPlausibility` (`montecarlo.rs:341-366`) counts on the unlimited
companion curves, carries `checked: bool` so an unjudgeable well shows "not checked" rather than a
fabricated clean pass, and emits a human-readable breakdown. **Regression lock.**

**Verified by.** SB-CUT-T25

#### SB-CUT-054 — Fail a Monte Carlo study whose chain step failed on every realization [P0] [status: PRESENT-OK]

**Requirement.** A module error inside a realization MUST be recorded and surfaced. A study whose
chain step failed on every realization MUST be reported as **failed**, with the underlying error
text, and MUST NOT be returned as a successful result.

**Rationale.** `SB-CORE-002`, first named violation, plus CONTRACT §5 point 3 and the application's
cardinal rule. A swallowed module error leaves the curve pool unchanged, so every downstream step
reads NaN and the study returns a P10 = P50 = P90 table of zeros with nothing to explain it — a
failed run presented as a converged one.

**As-built.** `PRESENT-OK` — `run_realization` (`montecarlo.rs:1148`) records the first failure in
a `OnceLock` with the defect named in the comment; surfaced at `:1825-1828` and marked `Failed` at
`:1834-1838`. The `montecarlo.rs:1125` pointer carried in `04_CORE_REQUIREMENTS.md` no longer
resolves to the cited behaviour. **Regression lock: this must not be undone.**

**Verified by.** SB-CUT-T37c

#### SB-CUT-055 — Never report an uninterpreted well as a zero result [P0] [status: PRESENT-OK]

**Requirement.** A pay summary row MUST carry the count of samples actually classified, and a
consumer MUST render an em-dash rather than `0.00` for Net, N:G and HCPV when that count is zero.
A well that was never interpreted MUST NOT be presented with the same shape of number as a well
that was interpreted and found barren.

**Rationale.** `SB-CORE-002`, second named violation, plus CONTRACT §5 point 3. Two wells of
identical rock reporting `Net 0.0 / N:G 0.00 / HCPV 0.00` and full net pay respectively is a
deliverable-grade error, and its symptom is a clean table.

**As-built.** `PRESENT-OK` — `n_classified` with the contract at `workflow.rs:865-866` and the
failure it prevents documented at `:880`; honoured at `report.rs:507-512`. The `workflow.rs:651`
pointer carried in `04_CORE_REQUIREMENTS.md` is `Err(e) => Outcome::Failed(e)` inside
`run_workflow_module_into`, not a fabrication site; the pay-summary row is emitted at `:1098-1117`.
**Regression lock.**

**Verified by.** SB-CUT-T37

### 4.4 Reporting, sensitivity and import

#### SB-CUT-056 — Never omit a report section because its computation failed [P0] [status: PRESENT-OK]

**Requirement.** The report MUST emit every configured section's header unconditionally. Where the
section's computation returned empty or errored, the report MUST render an explicit note stating
which, on the page. A batch export MUST NOT produce documents that are silently missing a section
while reporting zero errors.

**Rationale.** `SB-CORE-002`, third named violation, plus CONTRACT §5 point 3. A 540-well batch
producing 540 PDFs all missing their pay tables, with no error raised, is the worst failure mode
available to this product: it is undetectable by the person running it and obvious to the client
receiving it.

**As-built.** `PRESENT-OK` — `report.rs:454-458` emits the header unconditionally with the previous
`unwrap_or_default()` collapse named in the comment as *"exactly the cardinal-rule failure the
report path must not allow"*; `:533-549` pushes a `note_page` for both the empty and the error
branch; a `pay_caveat` explains the PERM-no-data case on the page. The `report.rs:380` pointer
carried in `04_CORE_REQUIREMENTS.md` no longer resolves to the cited behaviour. **Regression lock.**

**Verified by.** SB-CUT-T37b

#### SB-CUT-057 — Cross the IPC boundary in snake_case with unknown fields rejected [P0] [status: PRESENT-OK]

**Requirement.** Struct DTOs crossing the Tauri IPC boundary MUST use snake_case field names, MUST
carry `#[serde(deny_unknown_fields)]`, and MUST have a test that fails when the TypeScript
interface drifts from the Rust struct.

**Rationale.** Tauri camel-cases only the top-level command *argument* key, never nested struct
fields. A camelCase TypeScript DTO against a snake_case Rust struct deserialises to defaults with
no error — the failure mode that made an entire polygon-to-curve feature a no-op in a shipped
build while every call returned success. This is `SB-CUT-054`'s pattern one layer lower: a silent
drop where a loud failure belongs.

**As-built.** `PRESENT-OK` — `netflag.rs` carries the rule in its header and
`#[serde(deny_unknown_fields)]` on `NetFlagSpec`; `src/ipc.ts:542-571` documents the same rule and
records that a Rust test reads the interface and fails on drift. **Regression lock.**

**Verified by.** SB-CUT-T38

#### SB-CUT-058 — Sweep more than one cut-off at a time [P2] [status: PARTIAL]

**Requirement.** The cut-off sensitivity sweep MUST support sweeping more than one cut-off
simultaneously over a permutation grid, and MUST report the resulting surface.

**Rationale.** T1b Geolog `tp_pay_sensitivity` sweeps four cut-offs simultaneously. A
one-at-a-time sweep cannot show the interaction between a porosity and a saturation cut-off, which
is where the sensitivity of a marginal net actually lives.

**As-built.** `PARTIAL` — `run_cutoff_sweep` (`workflow.rs:1401`) sweeps exactly one of
`VSH | PHIE | SWE` over `[sweep_min, sweep_max]` at `steps` points, reporting `NET | HPV | NTG`.
Its masking of the sweep to DST or perforation intervals (`aux_intervals`,
`sample_incl_thickness`) is a capability none of the three incumbents documents and should be
kept.

**Verified by.** SB-CUT-T32

#### SB-CUT-059 — Solve backwards from a target [P3] [status: ABSENT]

**Requirement.** SandiBumi MUST be able to solve for the cut-off value that produces a stated
target net, net-to-gross or hydrocarbon pore volume.

**Rationale.** T2 IP is the only one of the three that can invert the sweep. Combined with
`SB-CUT-058`'s multi-cut-off grid the pair exists in no tool — Geolog has the sweep and the
interaction, IP has the inversion, neither is a superset.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T32

#### SB-CUT-060 — Address imported parameters by block, ordinal and semantic key [P2] [status: ABSENT]

**Requirement.** An imported parameter MUST be addressed by a `(block, ordinal, semantic key)`
triple. Any mismatch between the three MUST be a **load error**, never a silent remap. A bare
ordinal without a block MUST be a load error. Identity MUST NOT be taken from a declared name.

**Rationale.** T1b, with a worked failure. The ordinal namespace in IP's `MonteCarloDefaults.par`
is **per-block**: `#9` is `Vcl Cutoff` in `Cutoff`, `Rho mud_filt` in `PhiSw`, `Res Clay` in
`MinSolve`, `SP Clean` in `ClayVol` and `Rho Fluid` in `BLA`. The *name* is not a tiebreaker
either — `Res Clay`, `PhiT Clay` and `Qv 'b' Const` each appear in two blocks at different
ordinals — and the dossier's own first draft mis-filed four `MinSolve` rows as `PhiSw` for exactly
this reason. F-22 supplies the name-based failure independently: three Geolog spec files declare
the same `NAME = default_imperial`, so a name-keyed import silently loads the wrong unit system,
and the metric/imperial pair differs *only* in the thickness thresholds, making the error invisible
on every quantity except footage.

**As-built.** `ABSENT` — no import path exists.

**Verified by.** SB-CUT-T17, SB-CUT-T21

#### SB-CUT-061 — Validate display precision against field width [P3] [status: ABSENT]

**Requirement.** Display precision and field width MUST be validated against each other at
configuration time. A configuration whose precision can overflow its field MUST be rejected.

**Rationale.** F-23 (T2): IP ships `Result Precision` default 3 / maximum 6 against an
**8-character** report field, so a precision-6 value in a wide unit truncates silently in a
delivered report. A numeric formatter whose precision can exceed its field width is a silent
truncation waiting to happen, and the truncation is indistinguishable from a smaller number.

**As-built.** `ABSENT`.

**Verified by.** SB-CUT-T39

---

## 5. Parameters

**Forty-four rows. Eight ship `ABSENT — ships with no default`; eleven are `NON-ADOPTABLE — cited
for verification`.**

**Two transcription boundaries, stated so the call can be checked.** (a) IP's
`MonteCarloDefaults.par` is transcribed here because it is a self-documenting per-parameter
*defaults* file whose header names its own conventions — not a lookup table whose rows *are* its
content in the sense of CONTRACT §2.1, and not a chart digitization. Every row from it is marked
`NON-ADOPTABLE`, so nothing from it ships as a SandiBumi value; the rows exist so that an IP
import can be verified and so that the cross-vendor disagreement of F-5 and F-13 is checkable.
(b) The 220-entry `Cutoff.hlp` ordinal map is **not** transcribed. It is a parameter-name index
whose rows are its content; it is described by row count, structure and purpose in the source
string for `SB-CUT-060`, which is what CONTRACT §2.1 requires. No vendor chart lookup-table data
appears anywhere in this chapter.

Where a row's value is `ABSENT`, the competing vendor values are carried in the Source column with
their files, exactly as CONTRACT §2 requires, so the absence is auditable rather than merely
asserted.

**The two SandiBumi cut-off sets, recorded and not adjudicated.** `SB-CORE-004` names this domain's
cut-off defaults as the worked example of the parameter-source gap, so both shipped sets are
recorded here in full rather than only the survivor:

| Set | VSH ≤ | PHIE ≥ | SWE ≤ | Panes | Documented source |
|---|---|---|---|---|---|
| A | 0.5 | **0.08** | **0.5** | `monteCarloDialog.ts`, `resultsQcPanel.ts` | **none found** |
| B | 0.5 | **0.1** | **0.6** | `cutoffDialog.ts`, `summaryDialog.ts`, `reportDialog.ts`, `dashboardPanel.ts` | **none found** |

Set A was unified into Set B by an earlier fix (`src/ui/cutoffs.ts`), so Set B is what ships today
and `dashboardPanel.ts:56-59` is the one pane still hard-coding it rather than resolving it. **That
unification did not create a source.** A repository-wide search returns four hits
(`docs/playbook_build_progress.md:351`, `REVIEW.md:1794`, `docs/review_sweep/F3.md:74`,
`docs/manual_test_plan.md:4511`), every one of which records what the values were unified *to* and
none of which cites where any of them came from.

**This chapter does not adjudicate between Set A and Set B, and does not propose a third set.**
Under `03_EVIDENCE_BASE.md` §12.2 a cut-off with no defensible source ships **absent, not
defaulted**. The refusal is specified as `SB-CUT-016` (ship no value), `SB-CUT-017` (a default
without a source fails the build; a module needing a source-less parameter refuses at run time with
an actionable message) and `SB-CUT-018` (one authority, no pane hard-coding a literal), and is
verified by `SB-CUT-T35` and `SB-CUT-T36`. Choosing between 0.08 and 0.1 here would be exactly the
silent adjudication §12.2 was written to prevent — and F-1's evidence shows the choice is not even
well-posed, since delivered work spans PHIE 0.05–0.27 and one project record spans Vsh 0.55–0.85
across intervals of a single area.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Porosity cut-off | φ_c | **ABSENT — ships with no default** | v/v | Deliberate, per §3.7 of the dossier. Competing shipped values: IP 2025.3 manual Reports 1–4 `0.1`; Techlog `SummariesMonteCarlo.py` `POR_min = 0.15`; Geolog `default_*.paysum` PHIE `0.08`; Geolog `determin_mc.info` `PHIE_CUT 0.08` / `PHIT_CUT 0.08`; Geolog `tp_pay_sensitivity.info` `PHI_CUT 0` (permissive). Delivered work (project-kb `astrea-phr`, `bunga-block-phe-posco`, `duri-area09-phr` incl. its 2026 per-interval table) spans **0.05–0.27** | T1b / T2 / T4 |
| Water-saturation cut-off | Sw_c | **ABSENT — ships with no default** | v/v | As above. Competing: IP `0.5`; Techlog `SW_max = 0.85`; Geolog `determin_mc.info` `SWE_CUT 0.5` / `SWT_CUT 0.5`; Geolog `tp_pay_sensitivity.info` `SW_CUT 1`. Delivered work spans **0.50–0.90** | T1b / T2 / T4 |
| Clay/shale-volume cut-off | Vsh_c | **ABSENT — ships with no default** | v/v | As above. Competing: IP `0.5`; Techlog `VSH_max = 0.5`; Geolog `vshale-only_*.paysum` `VSH ≤ 0.3`; Geolog `determin_mc.info` `VSH_CUT 0.5` — **Geolog disagrees with itself**. Delivered work spans **0.20–0.85**, upper bound from `duri-area09-phr`'s Jaga interval; one record spans 0.55–0.85 across intervals of a single area | T1b / T2 / T4 |
| Permeability cut-off | k_c | **ABSENT — ships with no default** | mD | Geolog `tp_pay_sensitivity.info` line 50 default `0.01` mD, validation range `0<`; Geolog `advanced_metric.paysum` `LOPERM 0.1` mD / `HIPERM 1` mD; Geolog `determin_mc.info` `PERM_CUT 0`. Literature basis is the economic mobility ratio `(k/µ)_c` — Worthington & Cosentino (2005) via petro-kb Bentley & Ringrose §3.5; the paper itself is **not** in the corpus (§7 escalation) | T1b / T4 |
| Default distribution for a new prior | — | **ABSENT — ships with no default** | — | One vendor states three answers: Geolog `montecarlo.montecarlo` `DEFAULT_PDF_CURVE = Normal`; Geolog Configuration Files Editor help *"Normal, **Triangular (default)**, and Uniform"*; all **45** `*_DIST` rows in `determin_mc.info` default `Triangular`. IP's observed panel default is Gaussian; Techlog `*_Distribution = 'normal'`. Not adjudicated (dossier C-19) | T1b / T3 / T2 / T1a |
| Minimum iterations for a tail percentile | — | **ABSENT — ships with no default** | — | Geolog withholds `EHC_CDF` below `OPT_MC > 10` (*"Below this number the results are meaningless"*) — a **precedent for the refusal, not a value to adopt**, since it governs a different statistic on a different quantity. IP and Techlog set no threshold and print unsupported tail statistics | T3 |
| `Rw` uncertainty prior | σ_Rw | **ABSENT — ships with no default** | % of value | Geolog `determin_mc.info` `ERR_PC_RW 5` (⟹ σ 1.67 % at `SD = 3`); IP `MonteCarloDefaults.par` `Rw` 20.0 % (⟹ σ 10.0 % at `SD_MULT = 2`) — **6.0× apart on σ**, and neither vendor documents its provenance. `Rw` has the largest single influence on Archie `Sw` | T1b |
| Per-parameter uncertainty prior (general) | — | **ABSENT — ships with no default** | varies | Row-for-row across the two shipped vendor sets, **exactly one row agrees exactly** (neutron, 5 %); every other row disagrees in value, in basis, or in shift algebra. No cross-vendor consensus prior exists to adopt | T1b |
| Bounds operator | — | `min <= v < max` | — | Techlog `SummariesMonteCarlo.py` `limitType` 0, the shipped default; matches the Techlog doc string *"minimum cutoff <= range of values < maximum cutoff"* | T1a |
| Minimum bed thickness | SATHK | 0 | project depth unit | IP `Min Res/Pay Height` default 0 (`cutoffsandsummation.htm`); Techlog `thinBedInterval = 0.0` (`SummariesMonteCarlo.py` L636); Geolog `LUMP_SATHK 0` in `default_metric.paysum`. All three agree | T1a / T1b / T2 |
| Maximum bed separation | MAXSEP | 0 | project depth unit | Geolog `LUMP_MAXSEP 0` in `default_metric.paysum`; `0.5` m in `advanced_metric.paysum`. No IP or Techlog analogue | T1b |
| Maximum non-net inclusion | INCTHK | 0 | project depth unit | Geolog `LUMP_INCTHK 0` in `default_metric.paysum`; `0.25` m in `advanced_metric.paysum`. No IP or Techlog analogue | T1b |
| Depth discretisation model | — | `CENTRED` | — | Geolog `tp_paysummary.info` L63 `FRAME_REP` default `CENTRALISED`, options `CENTRALISED,TOPS`; matches IP's hard-coded rule (`cutoffsandsummation.htm` + raster `_candsclip0030.png`); Techlog implements an unreachable `"centred"` branch pinned at `SummariesMonteCarlo.py` L639 | T1b / T2 / T3 / T1a |
| Power-mean exponent, per curve | p | 1 (arithmetic) | — | IP raster `_candsclip0010.png` row 4; Techlog *"By default, an arithmetic average is used"*; Geolog `CURVE_ARITH = Yes` in `default_metric.paysum`. Geolog `tp_paysummary.info` L66 `POWER_MEAN_EXP` default **3** is explicitly **not** adopted | T1b / T2 / T3 |
| Monte Carlo seed | — | required; recorded on the result | int | Geolog Module Launcher `Seed` field — *"Entering a Seed value ensures that the results from run to run are deterministic"* (`mod_launcher_hc.1.07.html`). IP and Techlog have none | T3 |
| Iterations | N | 2000 | — | IP shipped dialog default (rasters `_mceaclip0038.png`, `_mceaclip0039.png`; confirmed as a product default by the IP2018 ADDENDUM item 4). Geolog 250 and Techlog 20 rejected as too low to resolve P10/P90 | T3 |
| Auto-stop enabled | — | off | — | IP raster `_mceaclip0038.png`, checkbox clear | T3 |
| Auto-stop burn-in | — | 200 | — | IP `define_monte_carlo_parameters.htm`, identical 2018→2025 | T2 |
| Auto-stop check interval | — | 100 | — | IP `define_monte_carlo_parameters.htm` | T2 |
| Auto-stop minimum total | — | 300 | — | IP `define_monte_carlo_parameters.htm` | T2 |
| Auto-stop tolerance | — | 0.1 | % | IP `define_monte_carlo_parameters.htm`; the criterion is P10, P50, P90 **and** mean all within tolerance simultaneously | T2 |
| Shift-to-σ multiple | SD_MULT | **2.0** for IP-sourced widths; field mandatory and explicit | — | IP `define_monte_carlo_parameters.htm`: *"Low Value Shift + High Value Shift represents four standard deviations"* ⟹ symmetric `w = 2σ`. Corroborated by the same page's ±2σ tornado landing exactly on the tabulated Low/High edge. Geolog `montecarlo.montecarlo` `DEFAULT_PDF_SD = 3`; Techlog supplies σ directly ⟹ 1 | T2 / T1b / T1a |
| Sampler truncation | TRUNC_SD | 2.5 | σ | IP `define_monte_carlo_parameters.htm`: *"limited to 2.5 standard deviations either side of the Mean value… another random number will be chosen"*, identical 2018→2025 | T2 |
| Tornado perturbation size | tornado_sd | 2.0 | σ | IP `define_monte_carlo_parameters.htm`: tornado runs at *"± 2 standard deviations for Gaussian distributions"* with all other parameters held at default. Governs a different mechanism from `TRUNC_SD` and is not in conflict with it | T2 |
| Tornado bar units | — | absolute output units | the quantity's own unit | Geolog `determin_unc_ref_hc.1.23.html`: bars *"measured in actual units of EHC or EPC"*, because *"The computations of min/max EHC from the sensitivity analysis should not change from one run to the next"* | T3 |
| Tornado parameter cap | tornado_top_n | 15 | — | Geolog `determin_mc`: *"the details of the 15 most significant parameters and logs … are presented in a table in the report file"*. IP states no cap | T3 |
| Perturbation regime | mc_regime | `VERTICAL` | — | Geolog `determin_unc_ref_hc.1.07.html` — horizontal processing implemented, tested and dropped: *"it does not allow for sensitivity studies … auto adjustment … results to be defined on a percentile basis"*. Techlog mixes both (inputs per depth L1216–1218, cut-offs per zone L1599–1601) | T3 / T1a |
| Clamp before accumulate | — | **false** | — | Geolog `determin_unc_ref_hc.1.23.html` "Results Averaging": *"the data used to compute zone averages are the unlimited versions … to ensure that there is no bias at the edges of the scales"*. Techlog clamps unconditionally at `SummariesMonteCarlo.py` L679, L681, L1281, L1651; IP clips by declared Curve Type | T3 / T1a / T2 |
| Prior centring rule | — | unperturbed value; stored explicitly per prior | — | Three vendors disagree under an asymmetric shift: IP centres the Gaussian mean on the Start Value; Techlog `np.random.triangular(lower, val, upper)` anchors the **mode**; Geolog `determin_unc_ref_hc.1.09.html` — *"The mid-point between min and max values is used as the mean"* | T2 / T1a / T3 |
| Listing percentiles | — | 10, 50, 90 | % | IP `MonteCarloDefaults.par` `Results` line `Yes  10  50  90  -999  -999`; Techlog `LowQuantile` / `MidQuantile` / `UpQuantile` = 10 / 50 / 90. Two independent vendors agree | T1b / T1a |
| Percentile direction | — | per-quantity, labelled on every output | — | IP `MonteCarloDefaults.par` header: *"'Yes' as the first parameter means the 10% is the 10 percentile lowest value"*, shipped `Yes` — one global flag, which cannot express the Sw carve-out IP's own manual describes | T1b / T2 |
| Percentile interpolation | — | type 7 (linear, `h = (n−1)p`) | — | **SandiBumi decision, not a vendor value** — no vendor states its method (IP unstated; Techlog `tlStat.percentile` inside a compiled library; Geolog silent). Implemented at `montecarlo.rs:819-820`; must be echoed on the output record per `SB-CUT-046` | decision |
| Perturbation enabled on a newly added prior | — | off | — | All six Techlog `_Uncertainty` flags ship `no`; Geolog `determin_mc.info` ships `OPT_MC = 1` (one iteration) against a validation range `1:50000` | T1a / T1b |
| IP `m exponent` MC width | w_m | 0.2 | Linear, dimensionless | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `PhiSw` #65 / `MinSolve` #18 / `BLA` #5. Realises σ = 0.10 at `SD_MULT = 2` | T1b |
| IP `n exponent` MC width | w_n | 0.2 | Linear, dimensionless | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `PhiSw` #66 / `MinSolve` #19 / `BLA` #6 | T1b |
| IP `a factor` MC width | w_a | 0.1 | Linear, dimensionless | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `PhiSw` #64 / `MinSolve` #17 / `BLA` #4 | T1b |
| IP `Rw` MC width | w_Rw | 20.0 | % of value | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `PhiSw` #1 / `MinSolve` #1 / `BLA` #3. Also `Rw bound` / `Rmf bound` at 30.0 % (`PhiSw` #5 / #7) | T1b |
| IP `Rho Matrix` MC width | — | 0.03 | g/cc, Linear | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `BLA` #8. Companion rows: `Rho Fluid` 0.02, `Rho Clay` 0.05, `Rho Dry Clay` 0.1 (`PhiSw` #12) | T1b |
| IP `Neu Clay` MC width | — | 0.05 | v/v, Linear | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `ClayVol` #7 / `BLA` #18 | T1b |
| IP `Gr Clean` / `Gr Clay` MC width | — | 10 | gAPI, Linear | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `ClayVol` #2 / #3 and `BLA` #1 / #2 | T1b |
| IP `B fact Juhasz` / `B fact W&S` MC width | w_B | 0.1 | **not stated — carry `unit: null, unit_source: "not stated"`** | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par`, `PhiSw` #94 and #132; the file has **no unit column**. IP's help prints `meq/ml`, which is **Qv's** unit and cannot be `B`'s, since the product `B·Qv` must carry conductivity units. **MUST NEVER be inferred from the help page.** Escalated in §7 | T1b |
| IP input-curve priors (`InCurves`, 7 rows) | — | Density 0.02 (g/cc, Linear); Sonic 2 (µs/ft, Linear); Neutron 5 (%); Rt 0.005 and Rxo 0.005 (**`Rec`**); GammaRay 5 (gAPI, Linear); Sigma 2 (c.u., Linear) | as stated per row | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par` `InCurves` block. **Not one row is ±10 %**, refuting IP's own manual sentence for all seven curves. The `Rec` type on Rt/Rxo has no analogue in Techlog or Geolog | T1b |
| Geolog `determin_mc` percentage-error priors | ERR_PC_* | `RW 5`, `RT 2`, `RXO 5`, `RHO 2`, `NPHI 5`, `DT 5`, `GR 5`, `M 10`, `SW_EXP 10`; `VSH`, `QV`, `PERM`, `PHI`, `SW`, `SXO`, `U`, `SP`, `RHOMA`, `FTEMP` 10 each; `ERR_RWS_MN`/`_MX`, `ERR_RT_SH_MN`/`_MX` 10 each | % of value | **NON-ADOPTABLE — cited for verification.** Geolog `bin/determin_mc.info`; the `ERR_PC_` prefix and the row comments (*"Rw percentage error"*) both state the basis. Realised σ is `value / 3` at `DEFAULT_PDF_SD = 3` | T1b |
| IP `Cutoff`-block MC shift widths | — | φ 0.05 (v/v); Sw 0.2 (v/v); Vcl 0.3 (v/v); every unnamed extra curve 1 (curve unit) | as stated per row | **NON-ADOPTABLE — cited for verification.** IP `MonteCarloDefaults.par` `Cutoff` block, 40 rows: shared Res/Pay at ordinals 3 / 6 / 9 and per-report at 47–49 / 61–63 / 75–77, all Linear Gaussian. The per-report widths are identical to the shared ones, and the flat `1` on unnamed curves is a placeholder rather than a considered prior | T1b |

---

## 6. Acceptance tests

Forty-four tests. IDs `SB-CUT-T01` … `SB-CUT-T39` follow the dossier's own `T-nn` numbering wherever
one exists, so §8 traceability is one-to-one; `T02b`, `T03b` and `T03c` keep the dossier's suffixes.
`SB-CUT-T35` … `SB-CUT-T39` are new and specific to SandiBumi's as-built state.

A test whose expected value has no source is labelled `CHARACTERIZATION`.

**`SB-CORE-002`'s verification clause is binding on `SB-CUT-T37`, `T37b` and `T37c`:** each asserts
on the **reported artefact** — the rendered summary row, the emitted PDF page, the returned job
result — and never only on the internal `Result`. A test that inspects a `Result::Err` and stops
there does not discharge the requirement, because every one of the three historical defects had a
correct internal error that never reached a surface a user reads.

| ID | Input | Operation | Expected (with tolerance) | Source of the expected value |
|---|---|---|---|---|
| **SB-CUT-T01** | Zone 100.0 → 104.0 ft, step 0.5 ft, flags `0,0,1,1,1,1,1,1,1` | Summation under `CENTRED` | `Net = 3.25 ft` exactly (0 ulp) | IP's own published worked fixture; arithmetic re-verified at dossier synthesis |
| **SB-CUT-T02** | Same fixture | Summation under `TOPS` with zone clipping | `Net = 3.0 ft`, `Gross = 4.0 ft`, `Unknown = 0.0`, all exact | Hand-trace of Techlog `computeGross()` (T1a). The bottom-most flagged sample sits **on** `zone.bottom`, so `min(104.0, 104.5) − max(100.0, 104.0) = 0` and it drops. The 0.25 ft gap to T01 is exactly one half-step at one zone contact |
| **SB-CUT-T02b** | Same fixture | `Gross` under all three models | `Gross = 4.0 ft` in **every** model | The `Σhᵢ = Z_bot − Z_top` invariant: the models differ in apportionment, never in total |
| **SB-CUT-T03** | One flagged sample **interior** to a zone, step 0.1524 m | Summation under `CENTRED` and under `TOPS` | Both return `0.1524 m`; assert **equality** | Dossier §2.1 consequence 1. A test asserting `TOPS ⟹ 0.0` would lock in a defect: no sample contributes zero for structural reasons, because the last sample owns a full trailing interval |
| **SB-CUT-T03b** | One flagged sample sitting exactly **on** `zone.bottom` | Summation under both models | `CENTRED ⟹ 0.5 × step`; `TOPS` with clip `⟹ 0.0`, and the sample is booked **UNKNOWN**, not NOT-NET | Dossier §2.1 consequence 2 + Techlog T1a L1050–1052. The only configuration in which a flagged sample yields zero net |
| **SB-CUT-T03c** | Zone bottom falling **between** samples (`Z_bot = 103.75` on a 0.5 ft grid) | Summation | Net includes the clipped partial interval; `Σhᵢ = Z_bot − Z_top` exactly | The property zone clipping exists to provide |
| **SB-CUT-T04** | One identical log expressed in ft and in m | Geometric average | Results agree **to machine precision**; additionally assert the result **differs** from IP's `(ΠC)^(1/Σh)` on a 0.5-step log | F-2 / ledger D-5.2. IP's form returns 10 / 100 / 3.6 × 10⁶ mD at 1.0 ft / 0.5 ft / 0.1524 m |
| **SB-CUT-T05** | Synthetic curve, known closed forms | Power mean at `p = 1`, `p = −1`, `p → 0`, `p = 1/3` | Each agrees with its closed form to 1e-12 relative | Definitional: `M_p = (Σw·C^p/Σw)^(1/p)`; `p → 0` ⟹ `exp(Σw·lnC/Σw)`. `p = 1/3` matches Techlog's third-power average |
| **SB-CUT-T06** | Flagged interval with varying φ and Sw | `1 − Σφh(1−Sw)/Σφh` vs `Σφh·Sw/Σφh` | Identical to 1e-12 relative | Algebraic identity; the IP↔Techlog form equivalence (dossier §2.2) |
| **SB-CUT-T07** | Flagged interval with varying φ and Sw | `HCPV` by direct summation vs `Net × φ̄ × (1 − S̄w)` | Identical to 1e-6 relative **with φ-weighted Sw**; assert it **fails** with thickness-weighted Sw | Dossier §2.9. Locks `SB-CUT-009` and `SB-CUT-010` together |
| **SB-CUT-T08** | IP's published two-well roll-up fixture | Multi-well summation | `Av Phi = 0.187`, `Av Sw = 0.263`, `PhiH = 13.05`, `PhiSoH = 9.623`, `Net = 70 ft`, each to the printed precision | IP's worked report (dossier §2.8), which reconciles. IP's *multi-well* worked report (D-5.11) does **not** reconcile and MUST NOT be used as a fixture |
| **SB-CUT-T09** | Interval containing one `0` and one `−5` sample, `p ≤ 0` | Geometric and harmonic average | Those two samples excluded, `excluded_nonpositive = 2` reported, **no early return**, average computed over the rest | F-8 (T1a): Techlog's harmonic early-returns `MissingValue` and its geometric has no guard |
| **SB-CUT-T10** | A sample outside every zone that passes every cut-off | Summation | Excluded from every cumulative curve and summary statistic | IP's stated zone-membership rule (T2) |
| **SB-CUT-T11** | Zone with 30 % null input samples | Summation | `Gross = Net + NotNet + Unknown` exactly; `N:(G−Unknown) > N:G` | Definitional partition (F-15, T1a Techlog's UNKNOWN booking) |
| **SB-CUT-T12** | Any fixture | Two runs at the same seed; two runs at different seeds | Same seed ⟹ **bit-identical**; different seed ⟹ different | Geolog's determinism guarantee (T3). Already passing at `montecarlo.rs:2130` |
| **SB-CUT-T13** | IP row `m` with tabulated width `0.2`, imported with `SD_MULT = 2` | Realise the Gaussian prior | Realised `σ = 0.10`; assert **not** `0.20` | IP `define_monte_carlo_parameters.htm`: *"Low Value Shift + High Value Shift represents four standard deviations"*, `Lo = Hi = w` ⟹ `σ = w/2 = 0.2/2 = 0.10`. **This is the live defect of §3.9** |
| **SB-CUT-T14** | 10⁶ Gaussian draws at `TRUNC_SD = 2.5` | Sample and summarise | No draw beyond ±2.5σ; realised variance **< σ²**, and the deficit is reported | IP `define_monte_carlo_parameters.htm` + IP2025's own warning that the realised variance is biased low |
| **SB-CUT-T15** | A `%` shift on a zero-valued parameter, normal distribution | Build the prior and draw | Falls back to the generic width; **never** a point mass. Additionally assert the returned value is **not** clamped to `[0,1]` before reaching the accumulator | Techlog's `val == 0` degenerate branch (T1a); the clamp extension guards F-7 |
| **SB-CUT-T16** | 2000 iterations with varying gross, φ and Sw | `P50 Gross × P50 N:G × P50 φ̄ × P50 (1−S̄w)` vs `P50 HCPV` | Assert they **differ**; assert the iteration-consistent P50 case is reported separately and labelled; assert the interpolation method and the percentile direction are present on the output record | Dossier §4 item 22; IP documents the inconsistency explicitly. Geolog's `CDF_OP` is the vendor precedent for the consistent form |
| **SB-CUT-T17** | An import row whose ordinal and semantic key disagree | Import | **Load error**, not a silent remap | FINDINGS §6 rule 7, driven by IP's `#41` keeping its number and changing its referent |
| **SB-CUT-T18** | A Monte Carlo run with both a perturbed parameter and a perturbed cut-off | Inspect the draw schedule | Parameter offsets are drawn once per section per iteration; cut-off offsets are drawn once **per zone** per iteration; the realised rank correlation of every requested pair is reported alongside the requested value | Dossier §2.13 (Techlog's documented per-zone cut-off draw) + F-10 (the realised-correlation gap) |
| **SB-CUT-T19** | 2000 iterations in which `Gross` **varies** | `P10(Net)/P10(Gross)` vs `P10(Net/Gross)` | Assert they **differ**; assert SandiBumi reports the latter | F-9 (T1a `statDictNTG[p] = ni[p]/gi[p]`). The test **must** use a varying gross or it passes vacuously |
| **SB-CUT-T20** | `s = 0.005` (ohm-m)⁻¹ applied as a `Rec` shift to `Rt = 1` and `Rt = 100` | Apply the shift | `Rt = 100` ⟹ ≈ 99.5–100.5 (±0.5 %); `Rt = 1` ⟹ ≈ 0.667–2.0 (−33 % / +100 %) | Arithmetic of `1/(1/R ± s)`: `1/(1/1 + 0.005) = 0.99502`, `1/(1/100 − 0.005) = 200.0`, `1/(1/100 + 0.005) = 66.67`. Locks the reciprocal shift's physical intent and catches a units mislabel |
| **SB-CUT-T21** | A `.par` row `#9` under `PhiSw`, under `MinSolve` and under `Cutoff` | Import | Resolves to `Rho mud_filt` / `Res Clay` / `Vcl Cutoff` respectively; a bare ordinal with no block is a **load error** | IP's per-block ordinal namespace (T1b), and the exact failure the dossier's own first draft made |
| **SB-CUT-T22** | A forced floating-point residual in `Gross − (Net+NotNet+Unknown)` | Reconcile | Within 1e-7 relative ⟹ absorbed into the largest component **and the absorbed amount appears in the result record**; beyond ⟹ structured error | Techlog's `adjustFinal` shape (T1a) with the `print` → result-field refinement |
| **SB-CUT-T23** | A relative shift on a zero-valued parameter under a **uniform** distribution | Build the prior and draw | Must **not** return a hard zero on every draw; falls back to the generic width. Additionally assert no `[0,1]` clamp is applied to the draw | The guard Techlog has in `triangular()` and omits in `uniform()` (T1a); the clamp extension guards F-7 |
| **SB-CUT-T24** | A value exactly equal to `min`, and one exactly equal to `max`, under every supported operator | Cut-off test | Included or excluded per the operator's own **documented** definition, for every operator | SandiBumi's own written specification. Techlog's docstring and code disagree for modes 2 and 3, so the vendor cannot be the oracle here — **the spec is** |
| **SB-CUT-T25** | 100,000 iterations on a synthetic interval with true `Sw = 1.0`, `σ = 0.10`, permissive cut-offs | Zone-average `Sw` | Unclamped mean `= 1.000 ±` Monte Carlo error; assert `out_of_range: true` is emitted; assert the clamped variant reproduces `−0.3989σ = −0.0399` to within Monte Carlo error | `E[min(1, 1+σZ)] − 1 = −σ·E[max(0,−Z)] = −σ/√(2π) = −0.3989σ`; at σ = 0.10 that is −0.0399 v/v ≈ 4 s.u. The bias is independent of iteration count, so it cannot be found by running longer |
| **SB-CUT-T26** | `35` entered into a porosity cut-off field | Validate | **Rejected** unless a unit accompanies it; `35 pu` accepted and stored as `0.35 v/v`; `35 v/v` rejected as out of bounds | F-19: IP's own manual mixes `pu` and `v/v` for one quantity — a 350× error whose symptom is a plausible all-net result |
| **SB-CUT-T27** | The same prior and seed under `VERTICAL` and `HORIZONTAL`, on a zone of `N = 100` samples | Zone-average spread | The vertical run's spread is **independent of N**; the horizontal run's shrinks as `1/√N`; assert the ratio ≈ `√100 = 10` | F-21. Geolog's stated reason for dropping horizontal processing |
| **SB-CUT-T28** | Triangular prior with `a = 1`, `c = 2`, `b = 6` | Sample and summarise | Mode `2.0`, mean `3.0`, median `2.84` (±0.01); assert the reported P50 **≠** the base case and that both are surfaced | Geolog's published values for this exact asymmetric case, with its named warning that P50 ≠ base case |
| **SB-CUT-T29** | A P10 requested from a 5-iteration run | Report | **Withheld** with a machine-readable reason; not printed, not `NaN`, not silently the minimum | F-18. Geolog's `EHC_CDF` requires `OPT_MC > 10`; IP and Techlog both print unsupported tail statistics |
| **SB-CUT-T30** | An IP row with `Shift = Rec` imported into a target with no reciprocal sampler | Import | **Load error**; never silently coerced to `Linear` | F-14, quantified by SB-CUT-T20: the coercion is negligible at `Rt = 100` and −33 % / +100 % at `Rt = 1`, so it hides in high-resistivity test data |
| **SB-CUT-T31** | Laminated fixture with `INCTHK` and `MAXSEP` non-zero | Amalgamate and report bed statistics | `Total Net Thickness` identical pre and post; `Total Number of Intervals` strictly lower and `Thinnest` strictly higher post-amalgamation; **both** blocks present in the result, with the thresholds and the sample interval recorded | F-16 + F-4. Geolog ships a complete worked amalgamation example (`examples/data/paysummary_edit.unl` + `examples/specs/paysummary_edit.paysum`) — the only vendor-supplied end-to-end fixture found in any of the three trees |
| **SB-CUT-T32** | `Gamma < 40 API AND (Poro > 0.05 OR Perm > 0.1 mD)` | Evaluate as a cut-off expression | Evaluates correctly; the same fixture under AND-of-all gives a **strictly smaller** net | T4 Bentley & Ringrose's own worked net-reservoir rule, which is not expressible in any of the three tools |
| **SB-CUT-T33** | Two fixtures — one clear of the cut-offs, one with data sitting **on** the porosity cut-off — at 10 / 750 / 5,000 iterations | Converge | The clear fixture converges by 750; the marginal fixture does **not**. Assert the auto-stop evaluates its tolerance on the **percentile that will be reported** and that the achieved iteration count is on the result | Geolog's published convergence experiment (F-6): convergence is set by the marginality of the pay, not by parameter count |
| **SB-CUT-T34** | The same fixture at 750 and 5,000 iterations | Tornado | Absolute min/max bars **identical** between runs; percentage-of-range bars **differ**. Assert bars are emitted in absolute units and any percentage carries its iteration count | Geolog: *"The computations of min/max EHC from the sensitivity analysis should not change from one run to the next"* — only the denominator moves |
| **SB-CUT-T35** | Every user-facing pane that accepts or displays a cut-off | Enumerate and resolve | Every pane resolves from the single cut-off authority; the test fails when a pane hard-codes a literal or bypasses the loader | `CHARACTERIZATION` on the pane list; the assertion itself is structural. Written against `SB-CUT-018` and the live `src/ui/dashboardPanel.ts:56-59` bypass |
| **SB-CUT-T36** | A freshly created project | Open every cut-off pane | Every cut-off field is in the `no default — user must set` state; a summation requested against an enabled-but-unset cut-off fails with a structured error; an unfiltered summation is reported **as unfiltered** | `SB-CUT-016` / `SB-CUT-017`. Expected state follows from the requirement, whose source is F-1's four disagreeing vendor sets and the delivered ranges |
| **SB-CUT-T37** | A well whose inputs were never computed, run through the pay summary | Read the **rendered summary row**, not the internal `Result` | `n_classified = 0`, and Net / N:G / HCPV render `—` — never `0.00`. A second well of identical rock that *was* interpreted renders numbers, so the two are distinguishable in the artefact | `SB-CORE-002` violation 2 (the pay summary fabricating `Net 0.0 / N:G 0.00 / HPV 0.00`) + CONTRACT §5 point 3. **Regression lock** on `SB-CUT-055`; feeds `SB-CORE-T03`…`T09` |
| **SB-CUT-T37b** | A batch report export over N wells in which the pay summary errors for every well | Read the **emitted PDF pages** | Every PDF carries the Pay Summary section header and an explicit note page stating the computation failed, with the error text; the batch reports N failures, not zero | `SB-CORE-002` violation 3 (540 PDFs missing their pay tables while reporting zero errors). **Regression lock** on `SB-CUT-056` |
| **SB-CUT-T37c** | A Monte Carlo whose chain step fails on every realization | Read the **job result**, not the internal `Result` | The job item is `Failed` and carries the underlying error text; no `McResult` is returned with a P10 = P50 = P90 table of zeros or NaNs presented as success | `SB-CORE-002` violation 1 (Monte Carlo swallowing module errors and reporting all-NaN volumetrics as success). **Regression lock** on `SB-CUT-054` |
| **SB-CUT-T39** | A numeric format whose display precision exceeds its field width | Validate the configuration | Rejected at configuration time, with the precision and the width both named in the message | `SB-CUT-061`; IP ships `Result Precision` default 3 / max 6 against an 8-character field |
| **SB-CUT-T38** | A TypeScript DTO carrying a field the Rust struct does not know, and one whose casing drifts | Deserialise across the IPC | Both **fail loudly**; neither silently drops the field or falls back to a default | `SB-CUT-057`. The failure it locks is a shipped feature that was a no-op while every call returned success |

---

## 7. Open items, escalations and refusals

### 7.1 Open — needed, not yet answerable

| # | Open item | What would settle it |
|---|---|---|
| O-1 | **Whether Techlog's C++ Summaries module shares the shipped Python script's defects** — the harmonic early-return, the missing geometric guard, the `limitType` 4–7 failures, the `int()` omission at L1635, and the `ni[p]/gi[p]` net-to-gross. Established for `SummariesMonteCarlo.py` only. | One live Techlog session settles all of them at once, plus the `limitType` 2/3 doc-vs-code question. **Until then, no SandiBumi material may say "Techlog is broken"** — the claim is scoped to the shipped script and must be stated that way |
| O-2 | **Geolog's lumping algorithm is specified only by a worked example.** The `.info` manifest and `.paysum` specs give the parameter surface and the defaults, but the interaction of `Min Thk` / `Include Min Thk` / `Include Max Sep`, and its tie-breaking when a lump could merge two ways, live in `tp_paysummary.exe`. | Run the shipped `paysummary_edit` example in Geolog and reverse the rules from the layout. Bounded, but needs a live session. **This is the largest remaining evidence gap in the domain and it blocks `SB-CUT-013`** |
| O-3 | **`Min Res Height` / `Min Pay Height` in MD or TVT when a TVD curve is selected.** Unstated by IP. Matters for deviated wells and interacts with `SB-CUT-012`. | Live IP session |
| O-4 | **IP's asymmetric-Gaussian centring.** Techlog anchors the mode, Geolog anchors the range mid-point, IP is the only one unstated. Sharper question: does IP keep the mean at the Start Value (two-piece) or move it (symmetric about a shifted mean)? | Live IP session — set Low 0.1 / High 0.5 on one parameter, run 10⁵ iterations, read the realised mean |
| O-5 | **The Sw percentile flip (D-5.12).** IP's manual says the convention is per-parameter and editable in the `Results` section; the shipped `Results` section has one global flag. | Live IP 2025 session — run an MC, read the P10 Sw and the P10 PhiSoH from one listing, and see whether they come from opposite ends of their arrays. Determines whether an IP-imported Sw percentile needs flipping |
| O-6 | **Whether IP's Curve Statistics "Net" and Cut-off & Summation "Net" are reconciled anywhere in the product.** They use different discretisation models and nothing found says so. | Live IP session — run both on one curve over one zone and compare. Determines whether F-4 is a documentation gap or a genuine internal inconsistency |
| O-7 | **`determin_mc`'s iteration ceiling contradicts its own validation by 10×.** `determin_mc.info` L269 gives `OPT_MC` the range `1:50000`; the Module Limitations page states 5,000 iterations over 2,000 depth units per zone. `EHC_ALL` is declared as a **5,000**-element array, which suggests 5,000 is real and the validation range is the defect. | One live Geolog run at `OPT_MC = 20000` |
| O-8 | **Geolog's default distribution is stated three ways** (`Normal` in `montecarlo.montecarlo`; *"Triangular (default)"* in the Configuration Files Editor help; `Triangular` in all 45 `*_DIST` rows). Low numeric stakes, high stakes for an importer that must fill an absent field. | Live Geolog session — open a Module Launcher MC panel with no prior configuration and read the pre-selected distribution |
| O-9 | **Whether Geolog can express a correlation between two genuinely independent *measurements*** (e.g. two resistivity tools sharing a calibration error). Its lack of a correlation coefficient is a documented design decision with Automatic Parameter Adjustment as the alternative, but that alternative does not cover this case. | Live Geolog session. Low priority — `SB-CUT-049` and `SB-CUT-050` take both mechanisms regardless |
| O-10 | **`Rec` shift behaviour as `R → 0`.** `1/(1/R + s)` is well-behaved for positive `R`, but IP documents no floor, and a perturbation driving `1/R + s ≤ 0` yields a negative or infinite resistivity. IP's ±2.5σ truncation is stated to exist for this, but the interaction of truncation with the `Rec` transform is not described. | Live IP run with a large `Rec` shift on a high-resistivity interval — or a decision to clamp explicitly in SandiBumi and document the divergence |
| O-11 | **Geolog's cross-well roll-up, and Techlog's at both levels.** IP's rule is fully documented; Geolog's *zone*-level roll-up is documented in full (`OPT_ZONEMERGE`); the rest is not. | Techlog `Doc/concept` multi-well pages (not found by name search) or a live session |
| O-12 | **The `HCPF` equation is raster-only.** Only the identity *"(EHC \* Area) would give the same value as (HCPF \* GRV)"* is in text. From it `HCPF` must reduce to `EHC / gross thickness`, but that is an inference, not a quotation, and is not carried as fact anywhere in this chapter. | Read the image, or a live Geolog run comparing `EHC/Gross` against a reported `HCPF` |

### 7.2 Escalation — needs Jauhar or a source not on this machine

| # | Escalation | The exact question |
|---|---|---|
| **E-1** ✅ **RESOLVED 2026-08-07** | **The `SD_MULT` fix is a one-line change with a numeric blast radius.** Correcting `docs/ref_monte_carlo_seeds.md` from σ = w to σ = w/2 halves every IP-seeded Gaussian prior, and therefore halves the reported P10–P90 spread on every study that used one. | **Jauhar ruled 2026-08-07: no delivered or in-flight study has quoted an uncertainty band from an IP-badged prior.** The 2× exposure is therefore **prospective only** — it never reached a deliverable, and no client-facing correction is required. The fix proceeds as specified: `SB-CUT-031` **[P0]** makes `SD_MULT` explicit and mandatory, `SB-CUT-T13` locks σ = 0.10 (not 0.20) for the IP `m` row, and `docs/ref_monte_carlo_seeds.md:50` is corrected from σ = w. **The regression lock is the whole point of the ruling:** the reason this stayed harmless is that nobody had used the path yet, so the test is what keeps it harmless once they do |
| **E-2** | **`SB-CORE-002`'s status for this domain.** All three of its named violations in this chapter were verified closed at the source on 2026-08-07, and two of the three `file.rs:line` pointers in `04_CORE_REQUIREMENTS.md` no longer resolve to the cited behaviour. | Should `SB-CORE-002` be revised from `PRESENT-DIVERGENT` to `PRESENT-OK, regression-locked` for this domain once `SB-CUT-T37`/`T37b`/`T37c` are green — and should the stale pointers be corrected in the core chapter? **The spine holds this call; this chapter does not edit `04_CORE_REQUIREMENTS.md`.** The other four of the seven violations are outside this domain and are untouched by this finding |
| **E-3** | **The `B fact` (Waxman-Smits / Juhász) unit is genuinely unknown.** `MonteCarloDefaults.par` has no unit column; IP's help prints `meq/ml`, which is **Qv's** unit and cannot be `B`'s, since `B·Qv` must carry conductivity units. An earlier draft filled the gap with `(mho·cm²)/(meq·m)`, which appears in **no** source and was deleted. | What is `B`'s unit in the Juhász formulation? Closure route is the Juhász source paper — **not on this machine** — or a live IP session showing the unit in the parameter dialog. Until then an import carries `unit: null, unit_source: "not stated"`. **It MUST NEVER be inferred from the help page.** Cross-domain: `SAT` owns Waxman-Smits and should be consulted before anyone fills this in |
| **E-4** | **Neither IP nor Geolog documents the provenance of its `Rw` uncertainty prior**, and they are **6.0× apart on σ** once each vendor's own convention is applied (Geolog 1.67 % vs IP 10.0 % of value). `Rw` has the largest single influence on Archie `Sw`. Neither is adoptable. | Should a house `Rw` prior exist at all, and if so should it be derived from the RFT/DST-calibrated `Rw` workflows in the delivered `astrea-phr` and `bunga-block-phe-posco` records? **A prior is a petrophysical parameter and is not invented here.** Cross-domain: `SAT` owns `Rw` |
| **E-5** | **The named-paper closure for cut-off *selection* is not on this machine.** Worthington & Cosentino (2005) — crossplot φ against k/µ for a consistent `(φc, kc)` pair — is identified via petro-kb's Bentley & Ringrose note §3.5 but the paper itself is not in the corpus. Qassamipour et al. (2020) for the histogram-maximisation method is cited in a project record and is likewise absent. | Should both be ingested before SandiBumi ships any cut-off **guidance**? This chapter specifies cut-off **machinery** only and deliberately ships no guidance. Cross-domain: `SHR` owns the derivation seam (§1) |
| **E-6** | **CONTRACT §2.1 boundary call, raised rather than decided.** This chapter transcribes IP's `MonteCarloDefaults.par` rows into §5 as `NON-ADOPTABLE — cited for verification`, on the reasoning that it is a self-documenting per-parameter *defaults* file rather than a lookup table whose rows are its content. The 220-entry `Cutoff.hlp` ordinal map is **not** transcribed, on the same reasoning applied the other way. | Is that the right line? CONTRACT §2.1 records exactly one exception (the Matthews & Kelly rows) and states it is **not a precedent**, and that a chapter believing it has a second case **stops and escalates rather than deciding**. This chapter is not claiming a second exception — it is claiming the file falls outside the prohibited class entirely — but the distinction is fine enough to be worth a ruling |

### 7.3 Refusal — things a vendor does that SandiBumi will not do

| # | Refusal | Reason |
|---|---|---|
| **R-1** | **Ship a cut-off default value.** | Four shipped vendor sets, no two identical, two of them from one vendor, against delivered work spanning Vsh 0.20–0.85 / PHIE 0.05–0.27 / Sw 0.50–0.90 with one record spanning Vsh 0.55–0.85 inside a single area. `03_EVIDENCE_BASE.md` §12.2 |
| **R-2** | **Implement IP's geometric average `(ΠC)^(1/Σh)`.** | It is unit-dependent — 10 / 100 / 3.6 × 10⁶ mD for one log at three depth steps. The divergence from IP on geometrically-averaged curves is **expected and documented**, not a bug, and `SB-CUT-T04` asserts it |
| **R-3** | **Adopt IP's 10 input-curve / 7 additional / 5 report caps, or its prose "50".** | Vendor implementation limits with no physical basis; the "50" is not backed by IP's own parameter model |
| **R-4** | **Clamp perturbed data before accumulation.** | It manufactures ≈ 4 saturation units of hydrocarbon at σ = 0.1 v/v, independently of iteration count, always in the same direction |
| **R-5** | **Bind value bounds to a declared curve-type string.** | IP's clip policy is keyed on curve type, so mis-typing a curve silently changes its numerics. Bounds attach to the quantity |
| **R-6** | **Draw independently per depth sample for anything feeding a summation.** | The zone-average spread then shrinks as 1/√N and gets *narrower* the more finely the well is logged. Geolog implemented this, tested it and dropped it |
| **R-7** | **Print a tail percentile, a CDF or a modal statistic whose preconditions fail.** | `SB-CORE-002`. Geolog demonstrates the refusal is shippable; IP and Techlog both print unsupported statistics |
| **R-8** | **Report a derived ratio as a ratio of percentiles.** | It is only correct when the denominator is constant, and it is wrong exactly in the tail a reserves case is quoted from |
| **R-9** | **Coerce an IP `Rec` shift to `Linear` on import.** | It changes the prior's shape, not its units: negligible at `Rt = 100`, −33 % / +100 % at `Rt = 1`. A load error is the correct outcome |
| **R-10** | **Approximate a correlation matrix that is not positive-definite.** | Already implemented as a refusal (`montecarlo.rs:595` + the note at the call site) and kept. Both incumbents accept the input and silently deliver a different coefficient |
| **R-11** | **Infer the `B fact` unit from IP's help page.** | The printed `meq/ml` is `Qv`'s unit and is dimensionally impossible for `B`. See E-3 |
| **R-12** | **Use IP's multi-well worked report as a test fixture (D-5.11).** | Its arithmetic does not reconcile. `SB-CUT-T08` uses the two-well roll-up, which does |
| **R-13** | **Convert an `HCPV` thickness to a volume inside the summation module.** | The fluid-gradient conversion belongs to `SHR`; doing it here would silently bind a summation to a saturation-height assumption |

---

## 8. Traceability — dossier disposition

**199 rows.** Every numbered item in the source dossier appears exactly once. The count
reconciliation is at the end of the section.

Disposition vocabulary per CONTRACT §8: `ADOPTED` (→ requirement ID) · `DEFERRED` (→ priority +
trigger) · `REJECTED` (→ reason) · `EVIDENCE-ONLY` (informs the chapter, generates no obligation) ·
`ESCALATED` (→ §7).

### 8.1 §2 — Definitions, equations and assumptions compared (17 rows)

These are the dossier's evidence inventory rather than its findings; each is `EVIDENCE-ONLY` unless
a §3 or §4 item lifts it into an obligation, and the lifting item is named.

| Dossier item | Disposition | Where it went |
|---|---|---|
| §2.1 Depth discretisation — three models | EVIDENCE-ONLY | Lifted by §3.2 / §4 item 1 → F-3, `SB-CUT-001` |
| §2.2 Averaging formulas, all three tools | EVIDENCE-ONLY | Lifted by §4 items 2–6 → `SB-CUT-006`…`008` |
| §2.3 Which average applies to which quantity | EVIDENCE-ONLY | Source for the `power_mean_exp = 1` default row in §5 |
| §2.4 Cut-off semantics and operators | EVIDENCE-ONLY | Lifted by §4 item 7 → `SB-CUT-020` |
| §2.5 Flag / report hierarchy | EVIDENCE-ONLY | Lifted by §4 item 9 → `SB-CUT-024` |
| §2.6 Cut-off default values as shipped | EVIDENCE-ONLY | Lifted by §3.7 / §4 item 10 → F-1, `SB-CUT-016` |
| §2.7 Minimum-thickness / lumping model | EVIDENCE-ONLY | Lifted by §4 items 11, 29 → `SB-CUT-013`, `SB-CUT-025` |
| §2.8 Multi-well / field roll-up | EVIDENCE-ONLY | Supplies the `SB-CUT-T08` fixture |
| §2.9 Derived volumetric outputs | EVIDENCE-ONLY | Supplies the `SB-CUT-010` identity and `SB-CUT-T07` |
| §2.10 Monte Carlo architecture | EVIDENCE-ONLY | Frames §4.3 as a whole |
| §2.11 MC distributions and parameterisation | EVIDENCE-ONLY | Lifted by §4 items 16, 17 → `SB-CUT-035`, `SB-CUT-036` |
| §2.12 MC correlation between parameters | EVIDENCE-ONLY | Lifted by §3.11 / §4 item 23 → F-10, `SB-CUT-049`/`050` |
| §2.13 Sampling mechanics, iteration counts, seeds | EVIDENCE-ONLY | Lifted by §4 items 15, 20, 24c, 28 → `SB-CUT-034`, `039`, `040`, `052` |
| §2.14 Percentiles and outputs | EVIDENCE-ONLY | Lifted by §4 items 21, 22, 22c → `SB-CUT-047`, `044`, `048` |
| §2.15 Cut-off sensitivity sweep | EVIDENCE-ONLY | Lifted by §4 item 24 → `SB-CUT-058`/`059`; the `pu`-vs-`v/v` flag → F-19, `SB-CUT-019`. Also carries the `EHC` / `EPC` definitions |
| §2.16 Zone-membership and other gating rules | EVIDENCE-ONLY | Lifted into `SB-CUT-011` and F-24 / `SB-CUT-012` |
| §2.17 Curve Statistics and the Detail Interval Breakdown | EVIDENCE-ONLY | Lifted by §4 items 1b, 31 → F-4, `SB-CUT-002`, `014`, `015` |

### 8.2 §3 — Differences that matter (14 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §3.1 IP's geometric average is wrong and unit-dependent | ADOPTED | F-2 → `SB-CUT-007`; `SB-CUT-T04`; refusal R-2 |
| §3.2 Discretisation bounded at ½ step per zone contact | ADOPTED | F-3 → `SB-CUT-001`; `SB-CUT-T01`/`T02`/`T02b` |
| §3.3 The σ convention — a 3× spread on one digit | ADOPTED | F-5 → `SB-CUT-031` **[P0]**; `SB-CUT-T13`; escalation E-1 |
| §3.4 Iteration defaults differ by two orders of magnitude | ADOPTED | F-6 → `SB-CUT-039`; `SB-CUT-T33` |
| §3.5 The Sw percentile flip | ADOPTED + ESCALATED | F-12 → `SB-CUT-047`; open item O-5 |
| §3.6 Techlog's harmonic average is broken in shipped source | ADOPTED + ESCALATED | F-8 → `SB-CUT-008`; `SB-CUT-T09`; open item O-1 bounds the claim |
| §3.7 Cut-off defaults — no two vendors agree, reality agrees with none | ADOPTED | F-1 → `SB-CUT-016` **[P0]**, `017`, `018`; §5 both-sets table; refusal R-1 |
| §3.8 Reproducibility — Geolog wins, correcting a standing claim | ADOPTED | `SB-CUT-034`; the "matches Geolog, beats IP and Techlog" wording is binding in §3.3 and §4.3 |
| §3.9 Bed amalgamation — only Geolog models it | ADOPTED | F-16 → `SB-CUT-013`; `SB-CUT-T31`; open item O-2 blocks it |
| §3.10 Null / "Unknown" accounting | ADOPTED | F-15 → `SB-CUT-003`, `004`, `029`; `SB-CUT-T11` |
| §3.11 Neither IP nor Techlog delivers the requested coefficient | ADOPTED | F-10 → `SB-CUT-049`; refusal R-10 |
| §3.12 Techlog reports N:G as a ratio of percentiles | ADOPTED + ESCALATED | F-9 → `SB-CUT-043`; `SB-CUT-T19`; open item O-1 bounds the claim |
| §3.13 Techlog's `adjustFinal` reconciliation guard | ADOPTED | `SB-CUT-005`; `SB-CUT-T22`, with the `print` → result-field refinement |
| §3.14 Clamping the perturbed data — three-way split | ADOPTED | F-7 → `SB-CUT-041` **[P0]**, `SB-CUT-030`; `SB-CUT-T25`; refusals R-4, R-5 |

### 8.3 §4 — Optimal choice per item (39 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 Depth discretisation | ADOPTED | `SB-CUT-001` |
| 1b Which model produced a given "Net" | ADOPTED | `SB-CUT-002` |
| 2 Arithmetic average | ADOPTED | `SB-CUT-006` at `p = 1` (the default) |
| 3 Sw average, φ-weighted | ADOPTED | `SB-CUT-009`, `SB-CUT-010` |
| 4 Geometric average | ADOPTED | `SB-CUT-007` |
| 5 Harmonic average | ADOPTED | `SB-CUT-008` |
| 6 Power mean | ADOPTED | `SB-CUT-006` |
| 7 Cut-off shape — two-sided with explicit operator | ADOPTED | `SB-CUT-020` |
| 8 Cut-off may be a curve | ADOPTED | `SB-CUT-021` [P3] |
| 9 Flag hierarchy — arbitrary named flags | ADOPTED | `SB-CUT-024`, with the OR revision in `SB-CUT-023` |
| 10 Cut-off default values — none ship | ADOPTED | `SB-CUT-016` **[P0]** |
| 10b Cut-off activation rule | ADOPTED | `SB-CUT-022` |
| 11 Bed amalgamation — three thresholds | ADOPTED | `SB-CUT-013` |
| 12 Null accounting — `Unknown` + `N:(G−Unknown)` | ADOPTED | `SB-CUT-003`, `SB-CUT-004` |
| 13 Reference frames — MD + TVD + TVDSS + TST | ADOPTED | `SB-CUT-012` |
| 14 Rising-hole handling — named modes | DEFERRED | **P4.** Trigger: the first dataset in scope with a genuine multi-pass rising-hole section. IP's `*`-prefix is a report annotation, not a computation choice, so there is no partial form worth shipping early |
| 15 MC seed — mandatory, recorded | ADOPTED | `SB-CUT-034` |
| 16 MC distributions + log variants | ADOPTED | `SB-CUT-035` |
| 17 Normal parameterisation — explicit σ + "shift is N σ" | ADOPTED | `SB-CUT-031`, `SB-CUT-036` |
| 18 Shift units — explicit units column | ADOPTED | `SB-CUT-036` |
| 19 Reciprocal shift for resistivity — keep it | ADOPTED | `SB-CUT-032`; `SB-CUT-T20`; open item O-10 on its `R → 0` behaviour |
| 20 Iteration default + convergence | ADOPTED | `SB-CUT-039` |
| 21 Percentile convention — 1P/2P/3P | ADOPTED | `SB-CUT-047` |
| 22 Percentile consistency — iteration-consistent cases | ADOPTED | `SB-CUT-044`, carrying the dossier's own withdrawal of the "beyond all three" claim |
| 22b Refusing to report an unsupported statistic | ADOPTED | `SB-CUT-045`; refusal R-7 |
| 22c Multi-zone roll-up of percentiles | ADOPTED | `SB-CUT-048` [P3] |
| 23 Correlation — rank copula + causal re-derivation | ADOPTED | `SB-CUT-049` (statistical), `SB-CUT-050` (causal) |
| 24 Sensitivity sweep UX | ADOPTED | `SB-CUT-058`, `SB-CUT-059`; live crossplot editing handed to the `PLT` seam (§1) |
| 24b Tornado units | ADOPTED | `SB-CUT-051` |
| 24c MC perturbation regime — vertical | ADOPTED | `SB-CUT-040`; `SB-CUT-T27`; refusal R-6 |
| 25 Cut-off uncertainty in MC | ADOPTED | `SB-CUT-042`; `SB-CUT-T18` |
| 26 Derived ratios (N:G) under MC | ADOPTED | `SB-CUT-043`; refusal R-8 |
| 27 Gross/Net/NotNet/Unknown reconciliation | ADOPTED | `SB-CUT-005` |
| 28 Perturbation on/off default — off | ADOPTED | `SB-CUT-052` |
| 28b Per-parameter priors as an explicit triple | ADOPTED | `SB-CUT-036`; the vendor priors themselves are `NON-ADOPTABLE` rows in §5 |
| 28c Distribution default stated once | ADOPTED + ESCALATED | §5 `ABSENT` row for `default_pdf`; `SB-CUT-037` for the centring half; open item O-8 |
| 29 Lumping vs flagging — many-to-one | ADOPTED | `SB-CUT-025` [P3] |
| 30 Clamping perturbed data | ADOPTED | `SB-CUT-041` **[P0]**, `SB-CUT-030` |
| 31 Bed-thickness statistics, emitted twice | ADOPTED | `SB-CUT-014`, `SB-CUT-015` |

### 8.4 §4.1 — Ledger dispositions (14 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| D-5.2 Geometric exponent `1/Σhᵢ` not `1/n` — RESOLVED, IP wrong | ADOPTED | F-2 → `SB-CUT-007`; `SB-CUT-T04` asserts the divergence from IP explicitly, so it is documented rather than discovered |
| D-OPEN-4 / R-3 MC shift defaults for m, n, a, Rw — RESOLVED from `MonteCarloDefaults.par` | ADOPTED | §5 `NON-ADOPTABLE` rows (a 0.1, m 0.2, n 0.2 Linear; Rw 20 %). The earlier inferred *"~0.25 from crossplot axis extents"* stays unadopted |
| D-05 Pc↔height with/without 0.433 psi/ft — NOT APPLICABLE | REJECTED (out of domain) | Verified absent from all four cut-off/summation/MC page texts. Ownership stays with `SHR`; §1 states the boundary and R-13 states the refusal |
| O-OPEN-3 / R-10 Cutoff-module parameter ordinals — RESOLVED, all 220 | ADOPTED (structure only) | `SB-CUT-060`; `SB-CUT-T21`. **The 220-row map itself is deliberately not transcribed** — CONTRACT §2.1; see escalation E-6 |
| D-5.3 / D-OPEN-2 Can Reservoir and Pay hold different cut values — RESOLVED | ADOPTED | F-25 → `SB-CUT-022` (one shared value, two independent use flags; Reports 3–5 carry their own) |
| D-5.4 Input-curve capacity 50 vs 7 — RESOLVED, binding cap is 10 total / 7 additional | ADOPTED as a refusal | `SB-CUT-027`; refusal R-3. SandiBumi inherits neither cap |
| D-5.1 `Sw` without the `i` subscript — vendor typesetting defect | EVIDENCE-ONLY | Confirms the per-sample `Swᵢ` term behind `SB-CUT-009`; no ambiguity remains |
| D-5.5 MC statistic curve naming, `XXX MN` vs `_mn` — cosmetic | DEFERRED | **P2**, trigger: building the IP import mapper (`SB-CUT-060`). SandiBumi uses one documented suffix scheme; the mapper must handle both spellings |
| D-5.6 Output Percentiles 10/50/90 vs Result-Curve P5/P50/P95 | ADOPTED | §5 `percentiles_listing` = 10, 50, 90, verified in shipped data. The P5/P50/P95 curve default remains screenshot-only; SandiBumi uses one set |
| D-5.7 REP3 titled "SW<0.45" ships with Sw cut 0.5 unticked | EVIDENCE-ONLY | Demo-content defect. Reinforces `SB-CUT-017` (vendor dialogs show demo values indistinguishable from defaults) but generates no separate obligation |
| D-5.8 Dependency correlation prose 0.5 vs grid 0.8 | EVIDENCE-ONLY | Neither is a value SandiBumi would adopt; §5 ships no correlation default |
| D-5.9 "±10 % of the valid value" copy-paste defect — CONFIRMED and quantified | ADOPTED | `SB-CUT-033`; §5 `InCurves` row records that **not one of the seven** input curves is ±10 % |
| D-5.10 `Sw Res Use` off by default, never stated plainly — CONFIRMED | ADOPTED | `SB-CUT-026` |
| D-5.11 Multi-well worked report arithmetic does not reconcile | ADOPTED as a refusal | Refusal R-12; `SB-CUT-T08` uses the §2.8 two-well roll-up, which does reconcile |

### 8.5 §4.2 — Corrections this dossier forces on prior records (10 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 The "stride of 3 ⟹ (value, use, ?)" inference is WRONG — the triplet is (Net Use, Pay Use, Cutoff value) | ADOPTED | `SB-CUT-022`'s record shape; `SB-CUT-060`'s addressing rule |
| 2 The "settable seed beats IP" framing oversells — Geolog already has one | ADOPTED | Binding wording in §3.3 and `SB-CUT-034`: **"matches Geolog, beats IP and Techlog"** |
| 3 `ref_monte_carlo_seeds.md`'s `σ = w` is inconsistent with its own source | ADOPTED | `SB-CUT-031` **[P0]**; §3.9 quantifies it at exactly 2×; `SB-CUT-T13`; escalation E-1. **The single highest-priority action arising from the dossier** |
| 4 `ref_monte_carlo_seeds.md` omits the `InCurves` block | ADOPTED | `SB-CUT-033` |
| 5 The dossier's own first draft mis-filed four `MinSolve` rows as `PhiSw` (C-1) | ADOPTED | `SB-CUT-060`; `SB-CUT-T21`. The worked failure is why only `(block, ordinal, name)` triples are safe |
| 6 The IP2018 → IP2025 reversal on "are 0.1/0.5/0.5 defaults?" was followed but never logged | ADOPTED | F-1 records both readings and notes that `SB-CUT-016` is strengthened under **either** — on the 2018 reading IP has no defaults, on the 2025 reading it has dangerous ones |
| 7 `ref_monte_carlo_seeds.md` must store the shift *type*, not just the value | ADOPTED | `SB-CUT-032`; `SB-CUT-T30`; refusal R-9 |
| 8 The "beyond all three" claim on percentile consistency is withdrawn | ADOPTED | `SB-CUT-044` carries the withdrawal in its own Rationale, so positioning material cannot re-derive the overclaim from this chapter |
| 9 Bentley & Ringrose quoted selectively — the **OR** rule and the N:G upscaling bias | ADOPTED | `SB-CUT-023` + `SB-CUT-T32` (the OR); `SB-CUT-002` + `SB-CUT-014` + F-16 (scale-dependence of N:G) |
| 10 Assorted IP detail cited but not carried | ADOPTED (in part) + DEFERRED (in part) | The Result-Precision-3/max-6 against an 8-character field → F-23, `SB-CUT-061`, `SB-CUT-T39`. The remainder (`Update Graphics every` 20; MC curve set named `MC (Monte Carlo)`; histogram overlay default Gaussian; Fluid/Solution Efficiency 1; Report Title 25 chars / Short Name 4 chars; X/Y zone coordinates requiring a deviation survey **and** surface location; the CPU and array-size cost of percentile output curves) → **DEFERRED P3**, trigger: the IP import mapper and the report-layout work. Recorded here so the chapter is not a lossy summary of its own source |

### 8.6 §5.1, §5.2, §5.5, §5.6 — Adoption-spec sections (4 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §5.1 Canonical equation forms (the single discretisation definition + the three-stage clamp contract) | ADOPTED | `SB-CUT-001` (the one definition, all three models, one shared clip), `SB-CUT-030` (the clamp contract), `SB-CUT-006`…`010` (the averaging family) |
| §5.2 Parameter table — every value with its source string | ADOPTED | §5 in full: 44 rows, 8 `ABSENT`, 11 `NON-ADOPTABLE`, every value byte-exact with its source string |
| §5.5 IP Cutoff-module parameter ordinal map, 220 entries | ADOPTED as *structure*, REJECTED as *content* | `SB-CUT-060` and `SB-CUT-T21` take the per-block addressing rule and the 40-row `Cutoff` block count (6 + 18 + 16). The 220 rows themselves are **not transcribed** — CONTRACT §2.1; the boundary call is escalated as E-6 |
| §5.6 FINDINGS §6 defect-catalogue rules applying to this domain | ADOPTED | Rule 3 → `SB-CUT-019`/`036`; rule 4 → `SB-CUT-T04`; rule 6 → `SB-CUT-029`; rule 7 → `SB-CUT-060`; rule 8 → `SB-CUT-028`; rule 9 → `SB-CUT-016`/`017`; rule 10 → F-13, `SB-CUT-033`; rule 14 → `SB-CUT-045` |

### 8.7 §5.3 — Data-model requirements (20 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 Cut-off record shape; Res/Pay shares one value, two flags | ADOPTED | `SB-CUT-022` |
| 2 `(block, ordinal, semantic key)` addressing; per-block namespace | ADOPTED | `SB-CUT-060`; `SB-CUT-T17`, `SB-CUT-T21` |
| 3 No cap on input curves, cut-offs, reports or flags | ADOPTED | `SB-CUT-027`; refusal R-3 |
| 4 Every default carries a source string; `no default` is first-class | ADOPTED | `SB-CUT-017` **[P0]**, discharging `SB-CORE-004` for this domain |
| 5 Units on every quantity in the type system | ADOPTED | `SB-CUT-019`; `SB-CUT-T26`. F-22's metric/imperial pair is the worked case |
| 6 No bare `SW`; φ-weighting off an explicit flag, never the mnemonic | ADOPTED | `SB-CUT-028`, `SB-CUT-009` |
| 7 Null discipline — `Unknown` separate from `Not Net`; both ratios | ADOPTED | `SB-CUT-003`, `SB-CUT-004`, `SB-CUT-029` |
| 8 Per-iteration joint records; `CDF_OP` semantics, shipped on; base case is a named iteration | ADOPTED | `SB-CUT-044` |
| 9 The seed is part of the result record, not a UI setting | ADOPTED | `SB-CUT-034` — already `PRESENT-OK` |
| 10 The discretisation model is named on every thickness-bearing result | ADOPTED | `SB-CUT-002` |
| 11 The reference frame is part of a result's identity | ADOPTED | `SB-CUT-012` |
| 12 Clip/clamp contract — three named stages, bounds on the quantity | ADOPTED | `SB-CUT-030`, `SB-CUT-041` **[P0]**; refusals R-4, R-5 |
| 13 Cut-off activation is an explicit per-cut-off flag | ADOPTED | `SB-CUT-022` |
| 14 A statistic whose preconditions fail is withheld with a machine-readable reason | ADOPTED | `SB-CUT-045`; `SB-CUT-T29`; refusal R-7 |
| 15 Null markers are typed sibling fields, never in-band sentinels | ADOPTED | `SB-CUT-029` — already `PRESENT-OK` via `n_classified` |
| 16 Cut-off criteria are a boolean expression, not a fixed conjunction | ADOPTED | `SB-CUT-023`; `SB-CUT-T32` |
| 17 Numeric formatting validated as a pair — precision against field width | ADOPTED | `SB-CUT-061`; `SB-CUT-T39` |
| 18 Bed statistics emitted twice, thresholds recorded, N:G not scale-invariant | ADOPTED | `SB-CUT-014`; `SB-CUT-T31`; the sample-interval half also lands in `SB-CUT-002` |
| 19 Percentile cases as reserves categories; per-quantity `direction` | ADOPTED | `SB-CUT-047` |
| 20 Multi-zone roll-up merges cases, not statistics; unmerged arrays retained | ADOPTED | `SB-CUT-048` |

### 8.8 §5.4 — Validation and regression tests (37 rows)

The dossier's test numbering is preserved one-for-one in §6, so this table is a
correspondence, not a re-derivation. Five tests in §6 (`SB-CUT-T35` … `T39`,
plus the `T37b`/`T37c` split) have **no** dossier antecedent — they arise from
the as-built reading in §3 and from `SB-CORE-002`, and are listed in §8.13.

| Dossier item | Disposition | Where it went |
|---|---|---|
| T-01 IP half-interval fixture, `Net = 3.25 ft` | ADOPTED | `SB-CUT-T01` |
| T-02 TOPS with zone clipping, `Net = 3.0` / `Gross = 4.0` / `Unknown = 0.0` | ADOPTED | `SB-CUT-T02` |
| T-02b `Gross` invariant under both models | ADOPTED, **strengthened** | `SB-CUT-T02b` — asserted under **all three** models, not two, since `SB-CUT-001` ships BOTTOMS as well |
| T-03 Interior single sample, CENTRED ≡ TOPS | ADOPTED | `SB-CUT-T03` |
| T-03b Sample exactly on `zone.bottom` | ADOPTED | `SB-CUT-T03b` |
| T-03c Zone bottom between samples | ADOPTED | `SB-CUT-T03c` |
| T-04 Unit invariance of the geometric mean | ADOPTED | `SB-CUT-T04` |
| T-05 Power-mean identities at `p = 1, −1, →0, 1/3` | ADOPTED | `SB-CUT-T05` |
| T-06 Sw average form equivalence | ADOPTED | `SB-CUT-T06` |
| T-07 Volumetric identity, and its failure under thickness weighting | ADOPTED | `SB-CUT-T07` |
| T-08 IP two-well roll-up fixture | ADOPTED | `SB-CUT-T08`, with D-5.11's warning attached: the *multi*-well report MUST NOT be a fixture |
| T-09 Non-positive guard, no early return | ADOPTED | `SB-CUT-T09` |
| T-10 Zone-membership gate | ADOPTED | `SB-CUT-T10` |
| T-11 `Unknown` accounting and the two ratios | ADOPTED | `SB-CUT-T11` |
| T-12 MC determinism on the seed | ADOPTED | `SB-CUT-T12` — already passing; carried as a regression lock |
| T-13 `SD_MULT` round-trip, `σ = 0.10` not `0.20` | ADOPTED | `SB-CUT-T13` **[P0]** |
| T-14 Gaussian truncation and the reported variance deficit | ADOPTED | `SB-CUT-T14` |
| T-15 Relative-shift degeneracy, plus the no-clamp extension | ADOPTED | `SB-CUT-T15` |
| T-16 Percentile consistency | ADOPTED, **extended** | `SB-CUT-T16` — additionally asserts the interpolation method and the percentile *direction* are on the output record (`SB-CUT-046`, `SB-CUT-047`) |
| T-17 Ordinal/semantic mismatch is a load error | ADOPTED | `SB-CUT-T17` |
| T-18 Per-sample input draws vs per-zone cut-off draws | ADOPTED, **extended** | `SB-CUT-T18` — additionally asserts the *realised* rank correlation is reported (`SB-CUT-049`) |
| T-19 Derived-ratio percentiles, with a varying gross | ADOPTED | `SB-CUT-T19` |
| T-20 `Rec` shift asymmetry at `Rt = 1` and `Rt = 100` | ADOPTED | `SB-CUT-T20` |
| T-21 Block-scoped ordinal import | ADOPTED | `SB-CUT-T21` |
| T-22 `Gross` reconciliation with the absorbed residual on the record | ADOPTED | `SB-CUT-T22` |
| T-23 Uniform-sampler zero degeneracy, plus the no-clamp extension | ADOPTED | `SB-CUT-T23` |
| T-24 Bounds-operator semantics at the endpoints | ADOPTED, with the oracle changed | `SB-CUT-T24` — the oracle is **SandiBumi's own specification**, not the vendor, because Techlog's docstring and code disagree for modes 2/3 |
| T-25 Clamping bias, `−0.3989σ` | ADOPTED | `SB-CUT-T25` |
| T-26 Unit-tagged cut-off entry, the 350× trap | ADOPTED | `SB-CUT-T26` |
| T-27 Draw-regime distinguishability, ratio ≈ `√N` | ADOPTED | `SB-CUT-T27` |
| T-28 Asymmetric triangular centring, mode 2.0 / mean 3.0 / median 2.84 | ADOPTED | `SB-CUT-T28` |
| T-29 Refuse-to-report gate on a 5-iteration P10 | ADOPTED | `SB-CUT-T29` |
| T-30 `Rec` not importable as `Linear` | ADOPTED | `SB-CUT-T30` |
| T-31 Bed statistics pre- and post-amalgamation | ADOPTED, **extended** | `SB-CUT-T31` — additionally asserts the thresholds **and the sample interval** are recorded (`SB-CUT-002`, `SB-CUT-015`) |
| T-32 OR-criterion support | ADOPTED | `SB-CUT-T32` |
| T-33 Convergence on marginal pay | ADOPTED | `SB-CUT-T33` |
| T-34 Tornado stability in absolute units | ADOPTED | `SB-CUT-T34` |

### 8.9 §6 — Gaps and escalations (19 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| 1 Percentile interpolation method unstated by all three vendors | ADOPTED as a requirement | `SB-CUT-046` — SandiBumi closes the gap by **declaring** type-7 on the output record rather than waiting on a vendor statement. `SB-CUT-T16` asserts the declaration is present |
| 2 D-5.12 the Sw percentile flip | CARRIED OPEN | Open item **O-5**; `SB-CUT-047` is written so the answer is a per-quantity `direction` field rather than a code change |
| 3 Geolog MC correlation support — re-framed, mostly closed | CARRIED OPEN (residue only) | Open item **O-9**, explicitly low-priority: `SB-CUT-049` and `SB-CUT-050` adopt **both** mechanisms (reported realised correlation *and* per-iteration re-derivation), so the answer changes nothing structural |
| 4 Geolog `tp_paysummary` averaging source is compiled | CARRIED OPEN | Folded into open item **O-2**, the `tp_paysummary.exe` evidence gap, which the chapter names as **the largest remaining evidence gap in the domain** and which blocks `SB-CUT-013` |
| 5 Whether Techlog's GUI Summaries shares the shipped script's code | CARRIED OPEN | Open item **O-1**, with the binding scoping rule: **no SandiBumi material may say "Techlog is broken"** — every such claim is scoped to `SummariesMonteCarlo.py` and must say so |
| 6 `Min Res Height` / `Min Pay Height` in MD or TVT | CARRIED OPEN | Open item **O-3**; interacts with `SB-CUT-012` |
| 7 Asymmetric Low/High under a Gaussian — revised | CARRIED OPEN | Open item **O-4**, sharpened to the answerable form (two-piece vs shifted-mean). `SB-CUT-037` stores the centring rule per prior, so either answer is expressible without a schema change |
| 8 Named-paper closure for the cut-off *selection* method | ESCALATED | **E-5**. This chapter specifies cut-off **machinery** and deliberately ships no selection **guidance**; the seam is declared to `SHR` in §1 |
| 9 Multi-well / multi-zone roll-up in Techlog and Geolog — partly closed | CARRIED OPEN | Open item **O-11**. IP's rule is fully documented and is what `SB-CUT-T08` tests; Geolog's zone-level roll-up is documented in full and is in `SB-CUT-048` |
| 10 Geolog's `EHC` never defined | CLOSED AT SOURCE | Closed inside the dossier from two independent shipped files (`*.loginfo` type declarations + `determin_unc_ref_hc.1.08.html`): `EHC = Σ φT·(1−SwT)·Δz`, a **length**, on **total** φ and Sw. Recorded in §8.10's register row and used in §3; no obligation of its own. The related `HCPF` equation stays open as item 17 |
| 11 Whether Techlog's C++ Summaries shares the N:G ratio-of-percentiles defect | CARRIED OPEN | Open item **O-1** (same scope). `SB-CUT-043` is `PRESENT-OK` regardless — SandiBumi computes ratios inside the iteration whatever Techlog does |
| 12 `Rec` shift behaviour as `R → 0` | CARRIED OPEN | Open item **O-10**, with the alternative closure route named: decide to clamp explicitly and **document the divergence** rather than wait |
| 13 The `B fact` unit is genuinely unknown | ESCALATED + REFUSED | **E-3** and refusal **R-11**. An importer carries `unit: null, unit_source: "not stated"`; the unit MUST NEVER be inferred from IP's help page, which prints `Qv`'s unit. Cross-domain to `SAT` |
| 14 `determin_mc`'s iteration ceiling contradicts its own validation by 10× | CARRIED OPEN | Open item **O-7**. `SB-CUT-039` takes its iteration default from a cited source and an auto-stop, so neither ceiling is adopted |
| 15 Geolog's default distribution is stated three ways | CARRIED OPEN | Open item **O-8**. Low numeric stakes; matters only to an importer filling an absent field, which `SB-CUT-060` requires to fail loudly rather than guess |
| 16 Neither IP nor Geolog documents its `Rw` prior's provenance — 6.0× apart on σ | ESCALATED | **E-4**. Both rows ship `NON-ADOPTABLE — cited for verification` in §5. `Rw` has the largest single influence on Archie `Sw`, and **a prior is a petrophysical parameter and is not invented here**. Cross-domain to `SAT` |
| 17 The `HCPF` equation is raster-only | CARRIED OPEN | Open item **O-12**. The inferred reduction `HCPF = EHC / gross thickness` is recorded as an **inference, not a quotation**, and is not carried as fact anywhere in this chapter |
| 18 Whether IP's two "Net" columns are reconciled anywhere in the product | CARRIED OPEN | Open item **O-6**; the finding itself is F-4 and is already a requirement (`SB-CUT-002`) whichever way the open question resolves |
| **Collective — the escalations the dossier closed during its own verification passes** (§0.1, §0.2), including `EHC`, the `determin_mc` reference section, the Geolog `.paysum` net-sand tier, the `DEFAULT_PDF_UNITS` decode and the `MonteCarloDefaults.par` shift-defaults question | ACCEPTED AS CLOSED, not re-adjudicated | Their *results* are carried throughout §2–§7; the closure decisions themselves are the dossier's and are not re-opened here. Listed as one row deliberately — see the reconciliation note at the end of this section |

### 8.10 §7 — Source register (1 row)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §7.1–§7.4 source register: IP, Techlog 2018.2 (r22885), Geolog V14, literature and project precedent | ADOPTED WHOLESALE | Every tier label in §5 and every `file:line` citation in §2–§3 resolves against this register. Tier assignments follow the dossier's T1a / T1b / P refinement of the CONTRACT §2 scheme, which is declared in this chapter's front matter. One row, not four: the register is a single apparatus and splitting it would imply four independent decisions where there was one |

### 8.11 `## Critique disposition` — blockers, majors, minors (23 rows)

Read from the dossier's own `## Critique disposition` section, which is
authoritative over the critique file. `cutoffs-summation-mc_critique.md` was
**not** opened, per CONTRACT §4 rule 3.

| Dossier item | Disposition | Where it went |
|---|---|---|
| **B-1** Techlog discretisation mischaracterised; §3.2's thin-bed table wrong | ADOPTED in the corrected form | The rebuilt §2.1 is what §3.1 and `SB-CUT-001` are written from. The **half-step-per-contact bound with opposite signs at the two contacts** replaces the withdrawn "bed disappears / −33 %" claim, and F-4 relocates the real thin-bed hazard to the *reported* bed thickness (`SB-CUT-015`). The partial rebuttal is carried precisely: `intervalType` **is** a parameter with a `"centred"` branch, but the shipped script pins it outside the user-reachable block — which is why `SB-CUT-001` demands an *explicit* model, not merely a configurable one |
| **B-2** Invented, dimensionally wrong unit on the `B` factor | ADOPTED as a deletion + escalation | E-3, refusal R-11, and the `unit: null, unit_source: "not stated"` import rule. The worked lesson — a plausible unit invented at synthesis and carried through four table rows — is the concrete argument behind `SB-CUT-017` and `SB-CORE-004` |
| **B-3** `EHC` is defined in the tree; escalation 10 was a false gap | ADOPTED | §8.9 row 10. The methodological lesson is carried as well: a *"no file states X"* claim must be backed by a name search of the spec catalogues before it is escalated |
| **B-4** `determin_mc` and its 26-page reference section never opened | ADOPTED — the largest single evidence addition | Geolog's positions in §2 and §3 rest on it: unclamped Results Averaging (F-7, `SB-CUT-041`), the vertical/horizontal decision (F-21, `SB-CUT-040`, refusal R-6), `CDF_OP` (`SB-CUT-044`), `OPT_MC` default 1 (`SB-CUT-052`), and the 45 `*_DIST` / 23 `ERR_PC_*` rows in §5. It is also why the "beyond all three" percentile-consistency claim is **withdrawn** rather than defended |
| **M-1** Stale "47 `Cutoff`-block rows" against C-2's corrected 40 | ADOPTED | `SB-CUT-060`'s block-count arithmetic (6 + 18 + 16 = 40) and `SB-CUT-T21` |
| **M-2** Techlog hard-clamps every perturbed value to `[0,1]` | ADOPTED — became a P0 requirement | F-7, `SB-CUT-041` **[P0]**, `SB-CUT-030`, `SB-CUT-T25`, refusals R-4 and R-5. Confirmed at four call sites in the vendor source |
| **M-3** IP's Curve-Type pre-processing clips were dropped in compaction | ADOPTED | §3's clamp comparison, and the binding rule that bounds attach to the **quantity**, never to a declared curve-type string (`SB-CUT-030`, refusal R-5) |
| **M-4** Curve Statistics and the Detail Interval Breakdown cited but never carried | ADOPTED | F-4 (**two "Net" columns in one product, computed two ways** — a finding the critique did not claim), `SB-CUT-002`, `SB-CUT-014`, `SB-CUT-015`, `SB-CUT-045`, `SB-CUT-029`, `SB-CUT-T31`, open item O-6 |
| **M-5** Stale `tornado_sd` source string; missed corroboration of the headline finding | ADOPTED | §5's `tornado_*` rows, and — load-bearing — the **second independent confirmation of `σ = w/2`**: IP's ±2σ tornado positioning is exactly the tabulated Low/High edge under its own `Lo+Hi = 4σ`. This is why `SB-CUT-031` is stated as a defect and not as a convention preference |
| **M-6** Delivered-work cut-off ranges understated | ADOPTED in the source's form, not the review's | The corrected range **Vsh 0.20–0.85 / PHIE 0.05–0.27** is what §5's evidence-against-a-default row carries, with the observation that made it decisive: one delivered record spans **Vsh 0.55–0.85 across intervals of a single area**. That is stronger evidence against a shipped default than the cross-client spread, and it is the empirical half of `SB-CUT-016` **[P0]** |
| **M-7** Three mutually incompatible definitions of the TOPS rule | ADOPTED | `SB-CUT-001`'s single interval-ownership definition with one shared zone clip, and the rebuilt `SB-CUT-T02` / `T03` / `T03b` / `T03c`. The draft's `TOPS ⟹ 0.0` expectation would have **locked a defect into the test suite** — recorded in `SB-CUT-T03`'s source column so it cannot be re-introduced |
| **M-8** Techlog equation-image coverage overstated; one equation stated wrongly | ADOPTED | `HCPOR−TH = net × POR_AVG × (1 − SW_AVG)` replaces an invented term; it is the identity `SB-CUT-010` and `SB-CUT-T07` assert |
| **M-9** Deliberately-unread Geolog specs hold a shipped name-collision defect and a net-sand tier | ADOPTED | Geolog's **third** lumping default set and its one-cut-off net-sand tier appear in §5; the tier structure is `SB-CUT-024`. The shipped `NAME = default_imperial` collision across three files is cited in §3 as vendor evidence that a spec's identity must not be inferred from its filename |
| **m1** §3.3's propagated table arithmetically inconsistent | ADOPTED | The corrected 17.8 % linearised figure and the exact asymmetric bounds (+19.4 % / −16.3 %) are in §3.9. The asymmetry is itself an argument for ranked-iteration percentiles over `mean ± kσ` (`SB-CUT-044`) |
| **m2** A snippet labelled "verbatim" that is not | ADOPTED as a discipline | Every code quotation in §2–§3 of this chapter is labelled either *transcribed from source* with a `file:line`, or as this chapter's own gloss. No quotation is labelled verbatim without a line reference |
| **m3** §2.13's evidence cell mis-cites the per-depth / per-zone split | ADOPTED | The corrected line references stand behind F-21, `SB-CUT-040` and `SB-CUT-042` |
| **m4** The `InCurves` "new evidence" claim overstates novelty | ADOPTED | `SB-CUT-033`'s scope is narrowed to what is genuinely new — the shift **types** (notably `Rec` on `Rt`/`Rxo`) plus the `GammaRay` and `Sigma` rows — and the `Rec` non-expressibility becomes `SB-CUT-032`, `SB-CUT-T30` and refusal R-9 |
| **m5** The 0.1/0.5/0.5 "IP defaults" reading adopted without reconciling two ingest records | ADOPTED | F-1 records both readings and states that `SB-CUT-016` **[P0]** is strengthened under either: on the 2018 reading IP ships no defaults, on the 2025 reading it ships dangerous ones |
| **m6** An in-IP percent/decimal inconsistency passing unflagged | ADOPTED | F-19, `SB-CUT-019`, `SB-CUT-T26`. The **350×** magnitude is stated because the symptom is a plausible all-net result rather than a visible failure |
| **m7** Assorted dropped IP detail | ADOPTED (in part) + DEFERRED (in part) | The substantive item — thickness-weighted zonal averages, so a TVD report is **not** a rescaling of an MD report — is F-24 and `SB-CUT-012`. The Result-Precision/field-width pair is `SB-CUT-061`. The remainder is deferred with its trigger recorded in §8.5 row 10 |
| **m8** Bentley & Ringrose quoted selectively | ADOPTED | The **OR** rule → `SB-CUT-023`, `SB-CUT-T32`; the N:G upscaling bias (0.55 → 0.75 → 1.0) → `SB-CUT-002` and `SB-CUT-014`. N:G is **not scale-invariant**, which is why the sample interval is part of a result's identity |
| **m9** `DEFAULT_PDF_UNITS = Linear (%)` quoted but never decoded | ADOPTED | Geolog's σ ≈ 1.67 % of value enters §3.9's three-vendor comparison only because of this decode. It is what makes the finding *"the same typed digit `5` spans two orders of magnitude across three vendors"* checkable rather than rhetorical |
| **m10** An unrecorded Geolog intra-vendor inconsistency (99.7 % vs 99 %) | EVIDENCE-ONLY | Doc drift on the coverage figure only; both pages agree on the load-bearing ±3σ mapping, so `σ = w/3` is unaffected and no requirement moves |

### 8.12 §0 — Correction log (1 row)

| Dossier item | Disposition | Where it went |
|---|---|---|
| C-1 … C-23, the dossier's own two verification passes | ACCEPTED AS THE CORRECTED TEXT | Every C-entry is a correction the dossier applied **to itself** before this chapter read it, so the corrected reading is the only one this chapter ever saw. Three are load-bearing enough to be named individually elsewhere and are not double-counted here: **C-1** (four `MinSolve` rows mis-filed as `PhiSw`) → §8.5 row 5 and `SB-CUT-060`; **C-8** (the rebuilt T-02/T-03 expectations) → §8.11 M-7; **C-18** (the ±2σ tornado corroboration of `σ = w/2`) → §8.11 M-5. One collective row, deliberately — see the reconciliation note below |

### 8.13 Requirements and tests with no dossier antecedent

The dossier is the *evidence* source, not the only source of obligations. The
following arise from §3's reading of the shipped SandiBumi source and from
`04_CORE_REQUIREMENTS.md`, and are listed here so §8 is a complete map in both
directions rather than only dossier → chapter.

| Item | Origin | Note |
|---|---|---|
| `SB-CUT-018`, `SB-CUT-T35`, `SB-CUT-T36` | §3's as-built reading | The six-pane cut-off drift and the surviving `src/ui/dashboardPanel.ts:56-59` bypass. The dossier establishes that **no defensible default exists**; the source establishes that SandiBumi currently **ships one anyway**, from two disagreeing sets, in a pane that does not consult the authority |
| `SB-CUT-054`, `055`, `056`, and `SB-CUT-T37`, `T37b`, `T37c` | `SB-CORE-002` | All three named violations were verified **closed at the source** on 2026-08-07. They are carried at **P0** as regression locks, not as live defects — see E-2 and CONTRACT §5's overclaim rule |
| `SB-CUT-057`, `SB-CUT-T38` | §3's as-built reading | The camelCase/snake_case IPC break that made a shipped feature a silent no-op while every call returned success. Now fixed with `#[serde(deny_unknown_fields)]`; the test locks it |
| `SB-CUT-061`, `SB-CUT-T39` | §4.2 item 10 (deferred residue), promoted | The one item in the deferred remainder with teeth: display precision validated against field width |

### 8.14 Count reconciliation

**199 rows.** The arithmetic, by sub-table:

| Sub-table | Source | Rows |
|---|---|---|
| §8.1 | §2 definitions, equations and assumptions compared | 17 |
| §8.2 | §3 differences that matter | 14 |
| §8.3 | §4 optimal choice per item (1…31, including 1b, 10b, 22b, 22c, 24b, 24c, 28b, 28c) | 39 |
| §8.4 | §4.1 ledger dispositions | 14 |
| §8.5 | §4.2 corrections forced on prior records | 10 |
| §8.6 | §5.1, §5.2, §5.5, §5.6 adoption-spec sections | 4 |
| §8.7 | §5.3 data-model requirements | 20 |
| §8.8 | §5.4 validation and regression tests (T-01…T-34 + T-02b, T-03b, T-03c) | 37 |
| §8.9 | §6 gaps and escalations (18) + 1 collective | 19 |
| §8.10 | §7 source register | 1 |
| §8.11 | `## Critique disposition` (B-1…B-4, M-1…M-9, m1…m10) | 23 |
| §8.12 | §0 correction log C-1…C-23, collective | 1 |
| | **Total** | **199** |

`17 + 14 + 39 + 14 + 10 + 4 + 20 + 37 + 19 + 1 + 23 + 1 = 199`.

**The count matches the dossier's own inventory, with two deliberate
collectives and one deliberate merge, each declared here rather than left to be
discovered:**

1. **§8.12 collapses C-1…C-23 into one row.** These are corrections the dossier
   applied **to itself** across two verification passes before this chapter
   opened it. Dispositioning them individually would record 23 decisions this
   chapter did not make: the corrected text is the only text it ever read.
   The three that carry independent weight — C-1, C-8, C-18 — are named
   individually in §8.5, §8.11 and §8.12 respectively, so nothing load-bearing
   hides inside the collective.
2. **§8.9's final row collapses the escalations the dossier closed during its
   own passes.** Same reasoning: their *results* are carried throughout §2–§7
   and are individually cited there, but re-adjudicating a closure this chapter
   did not perform would be a fabricated audit trail. Item 10 (`EHC`) is listed
   **separately and by number** because §6 still carries it as a numbered gap,
   and dropping it into the collective would have made the §6 row count 17.
3. **§8.10 records §7's four sub-registers as one row.** The source register is
   a single apparatus underpinning every citation in the chapter; four rows
   would imply four independent adoption decisions where there was one.

**Two counts differ from a naive reading of the dossier and are stated here so
the discrepancy is not mistaken for an omission:**

- **§8.3 is 39 rows against a §4 that numbers to 31.** The eight lettered
  sub-items (1b, 10b, 22b, 22c, 24b, 24c, 28b, 28c) are separately numbered
  findings with their own evidence and their own dispositions, not elaborations
  of their parents. Each has its own row.
- **§8.8 is 37 rows against a test list that numbers to T-34.** T-02b, T-03b and
  T-03c are the three tests M-7 forced when the TOPS rule was rebuilt. They are
  the tests that replaced an expectation which would have locked a defect in,
  so they are the last three that should be folded into a parent row.

**One direction of the map is deliberately not one-to-one.** Four requirement
clusters and six tests in this chapter have **no** dossier antecedent — they come
from §3's reading of the shipped source and from `SB-CORE-002`. They are
enumerated in §8.13 rather than silently added, so a reader reconciling §4 and
§6 against §8 finds the surplus accounted for.

**Nothing in the dossier is dispositioned `DEFERRED` without a stated trigger,
and nothing is dispositioned `REJECTED` without the evidence for the rejection.**
There are three `DEFERRED` items (§4 item 14, D-5.5, and the §4.2 item 10
residue) and two `REJECTED` items (D-05, out of domain and owned by `SHR`; and
§5.5's 220 rows as *content*, under CONTRACT §2.1 with the boundary call
escalated as E-6). No dossier item is unaccounted for.
