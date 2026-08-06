# C — Mineral Solver (IP 2025 help ingest)

**Agent:** C (mineral solver) of the 14-agent IP 2025 CHM ingest.
**Source:** Interactive Petrophysics 2025 compiled help, decompiled to
`C:\Users\ARUNIKA\AppData\Local\Temp\c25\`. Clean text `<stem>_text.txt`, images `<name>.png`.
**Counterpart for diffing:** IP 2018 decompile at `C:\Users\ARUNIKA\AppData\Local\Temp\c18\`.
**Prior anchor:** `D:\XX. SandiBumi\docs\research_2026-07\ip2018_chm_ingest\D_mineral_solver_ssc.md`.

**Numeric discipline.** Every number below is transcribed exactly as printed or as read from the
named image. Nothing is rounded, converted, inferred, or supplied from textbook knowledge. Where a
value is ambiguous in the raster it is in §8 OPEN ITEMS, not in the tables. Transcription source is
tagged on every line: `(page.htm)` = prose/ASCII on the page; `[img-read: file.png]` = read from the
image by this agent.

**Headline vs the 2018 ingest.** The 2018 ingest recorded ~99 equation images on
`minsolveeqandmeth.htm` as *rasterized — not recoverable*. **This ingest read them.** Roughly 70
equation rasters and ~30 model-grid screenshots on the 2025 pages have been transcribed, including
every previously-lost item: the Total Model Error objective, both density hydrocarbon models, the
neutron excavation chain, the EPT chain, the U hydrocarbon branches, the CEC→Qv end-point
conversion, the wet/dry clay porosity algebra, all 12 Sw equations, the B(T,Rw) formula, and the
secondary-porosity Wyllie/Raymer forms. The 2025 pages also, for the first time, print **real
mineral end-point grids as screenshots** — the 2018 report's headline finding was that *no*
end-point table is published anywhere. That is no longer true (§3).

---

## 1. Scope & page inventory

| Page (`_text.txt`) | Lines | Images | Status | What it carries |
|---|---|---|---|---|
| `minsolveeqandmeth` | 1312 | 122 | **fully read** — narrative + ~70 equation rasters read individually + ~30 model-grid screenshots read + ~20 UI chrome (nav arrows, expander glyphs) ignored as non-content | The whole method: solver algorithm, all tool response functions, all Sw equations, output equations, clay models, final calcs, PHIFLAG table, neutron `.neu` table |
| `mineral_solver` | 885 | 31 | **fully read** — 6 content screenshots read (parameter tabs, endpoint grids, model options) | **The ASCII counterpart page.** All 12 Sw equations, both m* forms, Shell m, HC density models, Sxo/Sw auto-equations, the complete zonal-parameter dictionary. This is the cross-check surface for the rasters on `minsolveeqandmeth` (rule 4) |
| `plot_the_mineral_solver_result` | 370 | 15 | **fully read** — images are logplots/dialogs, no equations | Track-by-track plot semantics, `TotErr`/`PhiFlg` QC reading, Mixings rule engine, Combined Model |
| `minsolvecalibrate` | 192 | 4 | **fully read** — 4 dialog-grid screenshots, no equations | End-point calibration against core XRD by multiple linear regression; one ASCII equation |
| `mineralsolver` | 194 | 5 | **fully read** — 3 content screenshots read | Module overview; **`U = Pef × (RHOB + 0.1883) × 0.93423`**; dry-weight→wet-volume; MINDEF.PAR mineral density grid |
| `3dparview` | 253 | 21 | **fully read — NO petrophysical content.** Pure UI/visualisation page (well selection, Z-axis/contour setup, spectrum, labels). All 21 images are dialog screenshots | Only method-relevant fact: parameters entered **as curves** are **averaged over the zone** before mapping, and such values are annotated `av`. Curve averages may be Averaged / Summed / Minimum / Maximum / Percentile |

No page was unreadable. No raw HTML fallback was needed — the text conversion was clean on all six.

**Image-read totals:** 78 distinct images read directly by this agent (63 equation rasters, 15
grid/dialog screenshots), 8 of them re-read at 1.9×–6× upscale to settle digit-level ambiguity.

---

## 2. Equations & response functions

Symbols follow the vendor's own naming. `EPP` = End Point Parameter (the mineral's response if the
rock were 100 % that mineral).

### 2.1 The universal model form

`Y = Vol1 × Min1 + Vol2 × Min2 + Vol3 × Min3 …` — printed twice, identically
(`minsolveeqandmeth.htm`, `mineral_solver.htm`). `Y` = input tool curve or fixed value (for Output
equations, the result curve); `Vol_i` = solved volumes; `Min_i` = mineral end-points.

Non-linearity is handled by **relinearisation inside an outer loop**: end-points are recomputed each
pass from the current porosity and Sw, so a linear solver can carry non-linear responses
(`minsolveeqandmeth.htm`).

### 2.2 Objective and error metrics

| # | Relation | Source |
|---|---|---|
| E1 | `Total_err = sqrt( Σ_{i=1..NumCrvs} ( (Crv_i − Crv_Rec_i) / Crv_Tol_i )² )` | `[img-read: _imsclip0122.png]`, verified at 4× |
| E2 | `TotalLinearModelError = Σ Abs(InputLog_i − ReconstructedLog_i) / ConfidenceWeight_i` | `[img-read: embim217.png]` |

`Crv_i` = ith input curve value; `Crv_Rec_i` = ith reconstructed curve; `Crv_Tol_i` = ith input
curve tolerance; `NumCrvs` = number of input curves (`minsolveeqandmeth.htm`).

**Critical structural note (verified at 4×):** there is **no `/NumCrvs` term and no `1/N`
factor**. The only normalisation is per-curve division by that curve's tolerance. E2 is emitted only
for backward compatibility with IP versions before IP4.4 (`minsolveeqandmeth.htm`).

**Resistivity special-case (prose):** for resistivity and conductivity equations all resistivities
are first converted to conductivities, then `Crv_i`, `Crv_Rec_i` and `Crv_Tol_i` each have their
square root taken before entering E1 (`minsolveeqandmeth.htm`).

### 2.3 Density

| # | Relation | Source |
|---|---|---|
| E3 | `ρb = ρmin1 × Vol1 + ρmin2 × Vol2 + ρmin3 × Vol3 …` | `[img-read: embim218.png]` |
| E4 | `Salinity = Alog( (3.562 − Log(Rmf75 − 0.0123)) / 0.955 )` [ppm] | `[img-read: embim219.png]` |
| E5 | `Den_water = 1.0 + 7 × Salinity × 10⁻⁷ − (Temp − 80)² × 10⁻⁶` [gm/cc] | `[img-read: embim220.png]` |
| E6 | Conventional HC model: `Den_hydrocarbon = 2 × ρhden × (10 − 2.5 × ρhden) / (16 − 2.5 × ρhden)` | `[img-read: _imsclip0088.png]` |
| E7 | Modified HC model: `Den_hydrocarbon = (5.5 × ρhden × (4 − ρhden) − 3) / (16 − 2.5 × ρhden)` | `[img-read: _imsclip0089.png]` |
| E8 | `ρhden = input hydrocarbon density` | `[img-read: _imsclip0090.png]` |

`Temp` = entered formation temperature °F; `Rmf75` = Rmf converted to 75 °F. For oil-based mud, Rw
and Rw temperature substitute into the same equations (`minsolveeqandmeth.htm`). Model choice is the
`Den Hy Model` parameter on the Sonic/Neutron/Density tab; the Modified model "corrects better for
the calibration of the density tool from electron density to apparent density"
(`minsolveeqandmeth.htm`).

**E4 resolved, not guessed.** The 2025 raster renders the antilog operator with a space (`A log`).
The IP 2018 raster of the same equation renders it closed-up as **`Alog`**
`[img-read: c18/embim216.gif]`, i.e. base-10 antilog. Independently confirmed arithmetically against
the vendor's own parameter screenshot: `Rmf = 0.1 ohm-m @ 60 °F` → Arps to 75 °F gives
`Rmf75 = 0.08166` → E4 with `Alog = 10^x` returns **87 700 ppm**, and the screenshot's
`Rmf Salinity` cell reads **87.8 Kppm** `[img-read: _imsclip0009.png]`. Same check on
`Rw = 0.081 @ 60 °F` returns 114 500 ppm against a printed 114 Kppm. Two independent confirmations.

**ASCII cross-check (rule 4) — both HC density models.** `mineral_solver.htm` prints them in ASCII
as `DenHcApp = RhoH * 2 (10 - 2.5 RhoH) / (16 - 2.5 RhoH)` and
`DenHcApp = (5.5 * Rhohy (4-Rhohy) -3) / (16 - 2.5 RhoH)`. The 2018 ingest flagged these as
"almost certainly corrupted — `* 2` is likely an exponent. Do NOT implement." **The raster settles
it: `2` is a multiplier, not an exponent.** The ASCII is correct, merely awkwardly spaced. This
resolves a standing 2018 open item.

### 2.4 Neutron

| # | Relation | Source |
|---|---|---|
| E9 | `φ = (φneu − Vcl × NeuCl + NeuMatrix + Exfact + NeuSal) / (Sxo + (1 − Sxo) × NeuHyHI)` | `[img-read: embim222.png]` |
| E10 | `NeuHyHI = 9 ρhc (4 − 2.5 ρhc) / (16 − 2.5 ρhc)` | `[img-read: _imsclip0111.png]` |
| E11 | `Exfact = (ρma / 2.65)² (2 Swx φx² + 0.04 φx) (1 − Swx)` | `[img-read: _imsclip0112.png]` |
| E12 | `φx = φ + (Vcl · NeuCl)` | `[img-read: _imsclip0113.png]` |
| E13 | `Swx = [ φ (Sx + (1 − Sx) NeuHyHI) + (Vcl · NeuCl) ] / φx` | `[img-read: _imsclip0114.png]` |
| E14 | `Neu_hc = NeuHyHI − Exfact / (φ (1 − Sx))` | `[img-read: _imsclip0115.png]` |

`Vcl` = wet clay volume; `NeuCl` = neutron wet clay end-point; `Exfact` = neutron excavation factor;
`NeuSal` = neutron formation salinity correction; `Sx` = water saturation as seen by the neutron
tool; `NeuHyHI` = neutron hydrocarbon apparent hydrogen index (`minsolveeqandmeth.htm`).

Stated behaviour (`minsolveeqandmeth.htm`):
- Sand, limestone and dolomite neutron end-points are **non-linear** and tool-dependent; set them to
  `Auto` and they are looked up from the `.neu` tables as a function of the depth-level porosity.
- Water-HI comes from the neutron **salinity look-up tables**, accounting for matrix type and the
  water salinity the tool sees. If `Neu Form Sal` is Off, **water-HI = 1.0**.
- Hydrocarbon HI, when computed from an entered hydrocarbon density (green cell), is adjusted for
  **excavation** via E10–E14.
- The zone the neutron reads can be switched from flushed (`Sxo`) to un-invaded (`Sw`) via the
  per-equation Invasion Factor.

**Neutron tool look-up file format** (`minsolveeqandmeth.htm`, ASCII, transcribed verbatim in the
2018 report §2.5 — **byte-identical in 2025**, re-verified line by line). Conventions:
- Entered neutron porosity is assumed to be in **limestone units**.
- Columns: `True Phi, Sandstone Matrix, Dolomite Matrix`, then salinity corrections at
  **50/100/150/200/250 Kppm** for sand, then lime, then dolomite, in that order.
- Porosity rows run to **0.60 (60 pu)** and the table must be completed to that value even where no
  published data exists; extrapolate as accurately as possible.
- Register new tools in `Neu_Parm_Files.neu` in the IP executable directory. Spacing is free; the
  **number of parameters per line and the number of porosity lines are not**. "Porosity values must
  not be changed."
- **The `-.1960` outlier at φ = .20 in the Dolomite/50 kppm column persists unchanged in 2025.**
  Neighbours are `-.0120` (φ=.15) and `-.0180` (φ=.25). Still almost certainly `-.0196` mis-keyed.
  Not corrected here. Verify against a shipped `Sch_CNL.neu` before use.

### 2.5 Sonic

| # | Relation | Source |
|---|---|---|
| E15 | Wyllie, solved for φ: `φ = [ Dt − Dtma − Vcl × (Dtcl − Dtma) ] / [ (Dtfl × Sxo + Dthy × (1 − Sxo) − Dtma) × Cp ]` | `[img-read: embim223.png]` |
| E16 | Wyllie, forward: `Dt = Dtma(1 − φCp − Vcl) + Dtfl × Sxo × Cp + Dthy(1 − Sxo)Cp + Dtcl × Vcl` | `[img-read: embim224.png]` |
| E17 | In volume terms: `Dt = Dtma × VolMat + Dtfl × VolWater + Dthy × VolHyd + Dtcl × VolClay` | `[img-read: embim225.png]` |
| E18 | Hunt-Raymer as used in IP: `Vfc = 1/(Dtfl × Sxo + Dthy × (1 − Sxo))`; `φ = [ (2Vma − Vfc) − sqrt( (2Vma − Vfc)² − 4 × Vma × (Vma − Vlog) ) ] / (2 × Vma)` | `[img-read: embim226.png]` |
| E19 | Wyllie↔Hunt-Raymer bridge: `Cp = 0.65156 + 0.8109 × Phi + 0.01322 × Dtma − 0.003261 × Dtfl` | `(minsolveeqandmeth.htm)` ASCII |

`Vma = 1/Dtma`, `Vf = 1/Dtfl`, `Vlog = 1/Dt` (`minsolveeqandmeth.htm`). E19 units: Phi in decimals,
Dtma and Dtfl in µsec/ft.

Stated compaction-factor compromise (`minsolveeqandmeth.htm`): the Wyllie equation is linear only
when `Cp = 1.0`. To carry `Cp ≠ 1` into the **linear** solver, IP multiplies **all fluid parameters
(water plus hydrocarbon) by Cp and ignores the Cp term in the VolMat term**. The vendor calls this
an acceptable compromise since Cp is only an unconsolidated-formation adjustment. Hunt-Raymer is
hard-coded as non-linear; E19 is the fitted bridge used inside the iteration loop, with `Dtma` and
`Phi` taken from the current model solution and the resulting `Cp` fed to Wyllie. **The non-linear
optimizer uses the full sonic equations and needs no Cp approximation.**

> The 2018 ingest classified E19 as **Tier C** (vendor-fitted, coefficients deliberately not
> transcribed). It is printed in plain ASCII on both the 2018 and 2025 pages and is transcribed here
> because the mission requires every equation captured. **Treat as Tier C for adoption purposes.**

### 2.6 EPT

| # | Relation | Source |
|---|---|---|
| E20 | `E' = (79.4 − 202.69 × Sal) × (1 − 0.385 × (T − 75) × (3230.0 − T) × 10⁻⁶)²` | `[img-read: embim227.png]` |
| E21 | `E" = 4558 / T^1.568 + 16.34 / Rmf` | `[img-read: embim228.png]` |
| E22 | `TPW = 2.3586 × sqrt( sqrt(E'² + E"²) + E' )` | `[img-read: embim229.png]` |

`Sal` = salinity of filtrate in ppm × 10⁻⁶; `T` = formation temperature °F; `Rmf` = filtrate
resistivity at formation temperature. For OBM, Rw substitutes for Rmf (`minsolveeqandmeth.htm`).

Usage rule (`minsolveeqandmeth.htm`): if the EPT/TPL equation is in the model, **remove the Sxo
equation** and set `Sxo Method = Min Model` so Sxo comes from the EPT result.

### 2.7 Volumetric photoelectric cross-section (U) — SPECIAL TASKING (a)

**The solver mixes U volumetrically, never Pe.** This is explicit and unambiguous.

| # | Relation | Source |
|---|---|---|
| E23 | `U wat = 0.00481 × Sal + 0.3883` | `(minsolveeqandmeth.htm)` ASCII |
| E24 | Gas branch (Input Hyd Den **< 0.4**): `U_hyd = 0.119 × ρhden` | `[img-read: embim230.png]` |
| E25 | Oil branch: `U_hyd = 0.133 × ρhden` | `[img-read: embim231.png]` |
| E26 | `U = Pef × (ρb + 0.1883) × 0.93423` | `[img-read: embim232.png]` **and** ASCII on `(mineralsolver.htm)` — identical |

The conversion equation is stated twice, once as a raster inside the methodology section and once as
plain text on the module-overview page, and the two agree exactly. Vendor rule, verbatim in
substance: **U must be computed from the Pef and Rhob curves *outside* the Mineral Solver module**
(`minsolveeqandmeth.htm`). The Preprocessor's General tab is the supplied route — it takes a density
curve and a PEF curve and writes a U curve, alongside a Rt/Rxo → Ct/Cxo conductivity conversion
`[img-read: _imsclip0001.png]`.

There is **no Pe equation type anywhere in the Mineral Solver equation list.** Pe enters the system
only as an input to E26. Both hydrocarbon and water U end-points can be set to `Auto`, in which case
they are computed from the true hydrocarbon density and the water salinity by E23–E25
(`minsolveeqandmeth.htm`). The `Auto`/true-density convention for U is separately switchable — the
Model Options dialog carries a dedicated checkbox, *"Oil and Gas parameters for U log will be
entered in downhole true densities"* `[img-read: _imsclip0077.png]`.

**Observed U end-points in the vendor's own worked models** (see §3.3): Quartz 4.8, Calcite 13.8,
Dolomite 9, wet Clay 7.454 / 6.429 / 10, Kerogen 0.264, Pyrite 82.

> **SandiMin implication.** This is the same U-not-Pe volumetric-mixing rule already recorded in
> Jauhar's `reference_tool_response_constants` memory note. IP 2025 confirms it independently and
> supplies the exact conversion constants and the fluid branches.

### 2.8 Conductivity / resistivity — linearised Archie

| # | Relation | Source |
|---|---|---|
| E27 | `1/Rxo = φ^m × Sxo^n / (a × Rmf)` | `[img-read: embim233.png]` |
| E28 | with n = m: `Cxo = (φ × Sxo)^m × (Crmf / a)` | `[img-read: embim234.png]` |
| E29 | `Cxo^(1/m) = (φ × Sxo) × (Crmf / a)^(1/m)` | `[img-read: embim235.png]` |
| E30 | generalised: `Cxo^(1/m) = Σ( Vwat_i × (Cwat_i / a)^(1/m) ) + Σ( Vmin_i × (Cmin_i)^(1/m) )` | `[img-read: embim236.png]`, verified at 4× |

`Cwat_i` = ith input water end-point; `Cmin_i` = ith conductive-mineral end-point; `Vwat_i`, `Vmin_i`
= result volumes (`minsolveeqandmeth.htm`).

**Structural note, verified at 4×:** in E30 the **water** term carries `/a` inside the 1/m-th root
and the **conductive-mineral** term does **not**. The asymmetry is in the vendor raster, not a
mis-read. Do not "tidy" it when implementing.

Constraints, stated plainly (`minsolveeqandmeth.htm`, repeated on `mineral_solver.htm`):
- The **linear** solver supports only a **modified Archie with n = m** for in-model
  resistivity/conductivity. The vendor calls this "not the recommended method".
- The **non-linear** solver uses the actual Sw equation from Zonal/Model Parameters. "This is one of
  the main advantages of the non-linear optimizer option."
- Four explicit linearised types exist: `Cxo Archie Lin`, `Rxo Archie Lin`, `Ct Archie Lin`,
  `Rt Archie Lin` `[img-read: _imsclip0106.png]`. Their unique capability: the linearised Archie is
  the **only** Sw equation that lets you assign a conductivity/resistivity to *every* mineral volume,
  so a conductive mineral such as pyrite can carry a resistivity (`mineral_solver.htm`).
- Un-invaded (`Cond. Ct` / `Res. Rt`) equations require **Invasion Factor = 0.0**, the auto-Sw
  equation **off**, and `Sw Method = Min Model`.

**Confidence transform, worked example as printed** (`minsolveeqandmeth.htm`). The confidence is
converted alongside the curve — for conductivity it has the 1/m-th root taken; for resistivity it is
first converted to conductivity, then rooted:

| | Conductivity example | Resistivity example |
|---|---|---|
| Input | 500 mmho | 2 ohmm = 500 mmho |
| Input confidence | 5 mmho | 200 ohmm = 5 mmho |
| m | 2 | 2 |
| Solver input value | 22.3 | 22.3 |
| Solver confidence | 2.24 | 2.24 |
| + error | (22.3 + 2.24)² = 602.2 mmho | 602.2 mmho = 1.661 ohmm |
| − error | (22.3 − 2.24)² = 402.4 mmho | 402.4 mmho = 2.485 ohmm |

Transcribed as printed. The resistivity row's `+error`/`−error` labels are inverted in the source
(1.661 < 2.485). **Unchanged from 2018 — the vendor has not fixed it.** Not corrected here.

### 2.9 Constraint equations

| # | Relation | Source |
|---|---|---|
| E31 | Unity: `1 = Vol1 + Vol2 + Vol3 …` — always included | `(minsolveeqandmeth.htm)` |
| E32 | Porosity equalisation: `VwatU + VhydU1 + VhydU2 + … = Vwat + Vhyd1 + Vhyd2 + …`, entered as `0 = −VwatU − VhydU1 − VhydU2 − … + Vwat + Vhyd1 + Vhyd2 + …` | `(minsolveeqandmeth.htm)` |
| E33 | Bound water, single clay: `0.15 = VboundWater / (VdryClay + VboundWater)` ⟺ `0.0 = 0.15 × VdryClay − 0.85 × VboundWater` | `[img-read: embim237.png]` |
| E34 | Bound water, multi-clay: `V_BoundWater = Σ [ V_dryClay_i × φTclay_i / (1 − φTclay_i) ]`, entered as `0 = Σ [ φTclay_i/(1 − φTclay_i) ] × V_dryClay_i + (−1) × V_BoundWater` | `[img-read: embim238.png]` |
| E35 | Sxo (auto-added), ratio form: `Sxo = ΣVwater_i / (ΣVwater_i + ΣVhydrocarbon_i)` | `[img-read: embim239.png]` |
| E36 | Sxo (auto-added), solver form: `0 = Sxo × ΣVhydrocarbon_i − (1 − Sxo) × ΣVwater_i` | `[img-read: embim240.png]` |
| E37 | Sw (auto-added), ratio form: `Sw = ΣVwaterU_i / (ΣVwaterU_i + ΣVhydrocarbonU_i)` | `[img-read: embim241.png]` |
| E38 | Sw (auto-added), solver form: `0 = Sw × ΣVhydrocarbonU_i − (1 − Sw) × ΣVwaterU_i` | `[img-read: embim242.png]` |
| E39 | Porosity limit example: `0.3 > Vwater + Vhyrocarbon` *[vendor typo "Vhyrocarbon" — transcribed as printed]* | `[img-read: embim243.png]`, verified at 4× |

`mineral_solver.htm` prints E36/E38 in an algebraically equivalent expanded ASCII form:
`0 = (Sxo−1) Vwater + Sxo × Vhydrocarbon1 + Sxo × Vhydrocarbon2 + …` and
`0 = (Sw−1) VwaterU + Sw × VhydrocarbonU1 + …`. Consistent with the rasters.

**E34 is the load-bearing one.** The constant end-point for a dry-clay bound-water equation is
**φTclay / (1 − φTclay)**, not φTclay, "to take into account that the output volumes are dry clay
volumes" (`minsolveeqandmeth.htm`). Vendor's own worked screenshot confirms the arithmetic exactly:
Illite `0.185`, Chlorite `0.112`, BoundWater `−1` `[img-read: _imsclip0060.png]`, against the
PhiTClay end-points Illite `0.156`, Chlorite `0.101` `[img-read: _imsclip0070.png]` —
0.156/0.844 = 0.1848 ✓ and 0.101/0.899 = 0.11235 ✓.

Constant-equation idioms printed as text (`minsolveeqandmeth.htm`):
- fixed volume: `0.02 = VPyrite` (2 % pyrite)
- mineral ratio, Orthoclase = 20 % of Quartz: `0 = (0.2 × Quartz) − Orthoclase`; entered as
  Orthoclase `−1.`, Quartz `0.2` `[img-read: _imsclip0061.png]`
- linear assemblage `VGlau = VQtz × a + b` → entered as `b = VGlau − a × VQtz`, with `b` in the
  left-hand Curve/Val column (zero for an intercept at the origin), VGlau parameter `1.0`, VQtz
  parameter `a`, everything else zero `[img-read: _imsclip0062.png]`
- **recommended constant-equation confidence ≈ 0.01**, "a 1 % volume error in the result, since the
  constant equations have units of volume"

### 2.10 Output equations

| # | Relation | Source |
|---|---|---|
| E40 | generic linear: `OutputCurve = Σ( Vol_i × EndPointParameter_i )` | `[img-read: embim244.png]` |
| E41 | `GrainDensity = Σ(Vol_i × EPP_i) / Σ(Vol_i)`, for all minerals that do **not** have a 0.0 EPP | `[img-read: embim245.png]` |
| E42 | `SonicMatrix = Σ(Vol_i × EPP_i) / Σ(Vol_i)`, same proviso | `[img-read: embim246.png]` |
| E43 | `Qv = Σ(Vol_i × EPP_i) / φT`; `φT = total porosity of the rock` | `[img-read: embim247.png]` |
| E44 | `Qv = ρdcl × Vdcl × CECdcl / φT` | `[img-read: embim248.png]` |
| E45 | Qv EPP, **wet** clay parameters: `EPP = ρdcl × CECdcl × (1 − φTclay)` | `[img-read: embim249.png]` |
| E46 | Qv EPP, **dry** clay parameters: `EPP = ρdcl × CECdcl` | `[img-read: embim250.png]` |
| E47 | `φTclay = Σ(Vol_i × EPP_i) / Σ(Vol_i)`, same proviso | `[img-read: embim251.png]` |
| E48 | `φT = φe + Vclay × φTclay` | `[img-read: embim252.png]` |
| E49 | `Saturation_eff = Σ(Vol_i × EPP) / Phie` | `[img-read: embim253.png]` |
| E50 | `Saturation_Tot = Σ(Vol_i × EPP) / PhiT` | `[img-read: embim254.png]` |
| E51 | `Rescl = Σ(Vol_i × EPP_i) / Σ(Vol_i)` | `[img-read: embim255.png]` |
| E52 | `OutPara = Σ(Vol_i × EPP_i) / Σ(Vol_i)` | `[img-read: embim256.png]` |
| E53 | `VolMin1Wt% = VolMin1 × Min1Den / ((1.0 − PhiT) × GrainDen)` | `(minsolveeqandmeth.htm)` ASCII |
| E54 | `DenHyCorr = DenInput − VolHyd (DenHyd − DenWater)` | `(minsolveeqandmeth.htm)` ASCII |
| E55 | Wt% → volume: `Wet Vol % = (Dry Weight %) × (1 − PhiT) × (Rock Grain Density) / (Mineral Grain Density)` | `(minsolveeqandmeth.htm)` and `(mineralsolver.htm)`, identical |

Behaviour, stated (`minsolveeqandmeth.htm`):
- Output curves are computed **after each loop of the mineral solver and before the water-saturation
  calculations**, so `Qv`, `PhiTClay`, `ResClay`, `Output Para` can be fed straight back into the Sw
  parameter grid within the same run.
- **Wiring rule:** reference the output curve with **no set name, or a set name that does not
  exist** (e.g. `Model:Qv_ms` where set `Model` is absent) so IP resolves it to the *current* model's
  output set. If the set or a same-named curve in the default set exists, that one is used for **all**
  models — a real cross-model contamination trap in multi-model interpretations.
- In zones of zero clay, `PhiTClay` / `ResClay` / `Output Para` output the value of the **first**
  mineral listed.
- For **Sonic, Density, Neutron, Conductivity, Ept and U** output equations, the same end-point
  handling and special processing used inside the model is reapplied (e.g. a Sonic output honours the
  Wyllie/Hunt-Raymer choice and Cp).
- `Output Dry Wt%` requires a grain-density equation in the model; the EPP is `1` for the volume you
  want and `0` elsewhere; for a **wet** clay model the clay EPP must be adjusted for its bound water
  (0.85 for 0.15 clay water). Not needed for a dry-clay model.
- `Den / Neu / Son Hydrocarbon Corrected` outputs are the input curve minus the hydrocarbon
  correction; EPPs are unused and should be 0. Multiple hydrocarbons sum; invaded and un-invaded
  volumes both count; with multiple waters, **the first water entered** is used for the correction.

### 2.11 Clay models and porosity

| # | Model | Relations | Source |
|---|---|---|---|
| E56 | **Wet clay** | `Phie = ΣVwater_i + ΣVhydrocarbon_i` ; `Vcl = ΣVwetclay_i` ; `Phit = Phie + Vcl × PhitClay` ; `Vdcl = Vcl − (Phit − Phie)` | `[img-read: embim257.png]` |
| E57 | **Dry clay, bound water in model** | `Phie = ΣVwater_i + ΣVhydrocarbon_i` ; `Vdcl = ΣVdryclay_i` ; `Phit = Phie + ΣVboundwater_i` ; `Vcl = Vdcl + (PhiT − Phie)` | `[img-read: embim258.png]` |
| E58 | **Dry clay, no bound water** (water = total porosity) | `Phit = ΣVwater_i + ΣVhydrocarbon_i` ; `Vdcl = ΣVdryclay_i` ; `Vcl = Vdcl / (1 − PhitClay)` ; `Phie = PhiT − PhitClay × Vcl` | `[img-read: embim259.png]` |
| E59 | Flushed-zone Sw, wet clay **or** dry clay **with** bound water | `Sxo = ΣVwater_i / (ΣVwater_i + ΣVhydrocarbon_i)` | `[img-read: embim260.png]` |
| E60 | Flushed-zone Sw, dry clay **without** bound water (water = total porosity) | `Sxot = ΣVwater_i / (ΣVwater_i + ΣVhydrocarbon_i)` | `[img-read: embim261.png]` |

`Vwater_i` = ith mineral of type `Water Sxo`; `Vhydrocarbon_i` = ith of type `Hyd. Sxo`;
`Vwetclay_i` = ith of type `Wet Clay`; `PhitClay` = the `PhiT Clay` zonal parameter, which may itself
be a curve produced by a `PhiTClay` output equation (`minsolveeqandmeth.htm`).

E59 and E60 are the **same algebraic expression** — the difference is only which saturation it *is*
(effective vs total). Verified: both rasters read identically apart from the `Sxo` / `Sxot` label.

**Hard constraints (prose):** wet and dry clays may **not** be mixed in one model; **Bound Water can
only be added to a Dry Clay model**; wet clay + bound water is rejected. Regardless of clay model, IP
computes both effective and total porosity and any Sw equation may be used
(`minsolveeqandmeth.htm`, `mineral_solver.htm`). For dry-clay models the `PhiTClay` output equation
is not used — bound water goes in as a mineral/fluid and its volume is set by a constant equation
(E33/E34).

### 2.12 Water-saturation equations (12)

Raster and ASCII exist for all of these — every one cross-checked (rule 4). Discrepancies in §5.

| # | Equation | Raster transcription | Source |
|---|---|---|---|
| E61 | Archie | `1/Rt = φ^m × Sw^n / (a × Rw)` | `[img-read: embim262.png]` |
| E62 | Archie PhiT | `1/Rt = φT^m × SwT^n / (a × Rw)` | `[img-read: embim263.png]` |
| E63 | Simandoux | `1/Rt = φ^m Sw^n / (a Rw) + Vcl × Sw / Rcl` | `[img-read: embim264.png]` |
| E64 | Modified Simandoux | `1/Rt = φ^m Sw^n / (a × Rw × (1 − Vcl)) + Vcl × Sw / Rcl` | `[img-read: embim265.png]` |
| E65 | Indonesian (Poupon-Leveaux) | `1/√Rt = ( √(φ^m /(a × Rw)) + Vcl^(1−Vcl/2) / √Rcl ) × Sw^(n/2)` | `[img-read: embim266.png]` |
| E66 | Woodhouse Tar | `1/Rt = Sw^n · ( φ^(m/2) / √(a·Rw) + Vcl^(1−Vcl) / √Rcl )²` | `[img-read: embim267.png]` |
| E67 | Dual Water | `1/Rt = ( φT^(m*) × SwT^n / a ) × ( 1/Rw + (Swb/SwT)(1/Rwb − 1/Rw) )` | `[img-read: embim268.png]` |
| E68 | Dual Water `m*` | `m* = m_input + Cm( 0.258 × Y + 0.2 (1 − e^(−16.4 × Y)) )` ; `Y = Qv × φT / (1 − φT)` | `[img-read: embim269.png]` |
| E69 | Juhasz (Waxman-Smits) | `1/Rt = ( φT^m × SwT^n / (a × Rw) ) × ( 1 + Bn × Qvn × Rw / SwT )` ; `Qvn = Vcl × φTclay / φT` | `[img-read: embim270.png]` |
| E70 | Waxman-Smits | `1/Rt = ( φT^(m*) × SwT^n / (a × Rw) ) × ( 1 + B × Qv × Rw / SwT )` | `[img-read: embim271.png]` |
| E71 | Qv from PhiT | `Qv = a / PhiT + b` | `[img-read: embim272.png]` |
| E72 | B(T, Rw) | `B = ( −1.28 + 0.225 × T − 0.0004059 × T² ) / ( 1 + Rw^1.23 (0.045 × T − 0.27) )` | `[img-read: embim273.png]`, verified at 4× |
| E73 | Waxman-Smits `m*` | `m* = m_input + Cm( 1.128 × Y + 0.22 (1 − e^(−17.3 × Y)) )` ; `Y = Qv × φT / (1 − φT)` | `[img-read: embim274.png]` |
| E74 | Poupon-Aguilera | `1/Rt = φ^m × Sw^n / (a × Rw × (1 − Vcl)) + Vcl / Rcl` | `[img-read: _imsclip0091.png]` |
| E75 | Poupon-Tixier | `1/Rt = (1 − Vcl) × φ^m × Sw^n / (a × Rw) + Vcl / Rcl` | `[img-read: _imsclip0110.png]` |

Symbol dictionary as printed (`minsolveeqandmeth.htm`): `m` cementation factor; `m*` cementation
factor for Dual Water / W&S; `n` saturation exponent; `a` tortuosity factor; `Vcl` **wet** clay
volume; `Sw` effective water saturation; `SwT` total; `Swb` bound-water saturation; `Rw` formation
water resistivity; `Rwb` bound-water resistivity; `Rt` input resistivity curve; `Rcl` clay
resistivity; `Qvn` "normalized" CEC per unit **total pore volume**; `Bn` normalized equivalent
conductance of clay cations; `Qv` CEC per unit total pore volume; `B` equivalent conductance of clay
cations; **`T` formation temperature in degrees centigrade**.

**E72 note.** The trailing parenthesis in the vendor raster is unbalanced (`… (0.045 × T − 0.27))`).
Confirmed at 4×; **identical in IP 2018** `[img-read: c18/embim282.gif]`. Structure is unambiguous
despite the stray bracket. This matches the Juhász B formula already verified in Jauhar's
`reference_waxman_smits_b` memory note.

Citations the manual itself gives:
- **Woodhouse Tar** — R. Woodhouse, *"Athabasca Tar Sands Reservoir Properties Derived from Core and
  Logs"*, 1976 17th annual SPWLA Logging Symposium. Described as a modified Indonesian
  (Poupon-Leveaux).
- **Poupon-Aguilera** — Roberto Aguilera, *"Extensions of Pickett Plots for the Analysis of Shaly
  Formations by Well Logs"*, The Log Analyst, Sept–Oct 1990. Was simply called "Poupon" in earlier
  IP. `n` and `m` exponents added by IP.
- **Poupon-Tixier** — Poupon A, Loy ME, Tixier MP (1954), *"A contribution to electric log
  interpretation in shaly sands"*, Trans AIME 6(06):138–145. `m` and `n` exponents added by IP.
- Both Poupon variants: **"assumes a formation of laminated sands and shales with the sands being
  clean."**
- Vendor's Vcl-vs-Vsh position, stated for both Poupon forms: in the original equations `Vcl` is
  `Vshale` and `Rcl` is `Rshale`; IP computes clay volumes and treats shale as a rock type, but the
  equation can still be used if the interpreter picks shale-representative parameters.

**Qvn / Bn coupling, worth carrying into SandiMin** (`minsolveeqandmeth.htm`): `Qvn` and apparent
water conductivity are output as curves when Juhasz is the default Sw equation; `Bn` is picked on the
interactive Qvn/Cwapp crossplot by making the 100 % wet line pass through the wet shaley points. The
manual states explicitly that **there is a strong correlation between bound-water volume
(PhiT − Phie) and Bn** — change `PhiTclay` and `Bn` must be re-picked.

### 2.13 m, n and Sxo logic

| # | Relation | Source |
|---|---|---|
| E76 | Shell formula: `m = 1.87 + 0.018 / φe` | `[img-read: embim275.png]`, verified at 4× — **see §5.1, ASCII says 0.019** |
| E77 | m variable with Vcl: `m = m × 10^(Vcl − VclCutoff)`, applied when `Vcl > Vcl cut-off`, **after** any m* calculation | `[img-read: embim276.png]` |
| E78 | Effective Sw from total: `Sw = (SwT − Swb)/(1 − Swb)` ; `Swb = 1 − Phie/Phit` | `[img-read: embim277.png]` |
| E79 | Sxo from invasion factor, WBM: `Sxo = (Sw + InvasionFactor)/(1 + InvasionFactor)` | `[img-read: embim278.png]` |
| E80 | Sxo limits, WBM: `Sw^SxoLimit ≥ Sxo ≥ Sw` | `[img-read: embim279.png]` |
| E81 | Sxo limits, OBM: `Sw ≥ Sxo` | `[img-read: embim280.png]` |
| E82 | `n` from EPT/Rxo: `n = m + mPlus` | `(mineral_solver.htm)` ASCII |

E78 applies whenever **Archie PhiT, Dual Water, Juhasz or Waxman-Smits** is the chosen equation
(`minsolveeqandmeth.htm`).

`m` sources (`mineral_solver.htm`): parameter, input curve, **Shell formula**, **EPT/Rxo**, or
**Qv → variable m\*** (Dual Water and W&S only). For the EPT/Rxo route the EPT tool must be in the
mineral model, `Sxo Method` must be `Min Model`, and the Sxo equation must **not** be in the model.
The resulting m is clipped by `min m value` / `max m value`.

Flushed-zone substitution rule (`minsolveeqandmeth.htm`): Sxo uses **the same equation** as Sw with
`Rmf` for `Rw`, `Rxo` for `Rt`, `Rmfb` for `Rwb`, `RxoCl` for `Rcl`.

OBM behaviour: with OBM the invasion-factor parameter **is** the flushed-zone Sxo value — an
invasion factor of 0.5 gives a maximum Sxo of 0.5 (`minsolveeqandmeth.htm`). The published chart
`[img-read: _imsclip0074.png]` plots Sxo vs Sw for invasion factors **0.5, 1, 2, 4**, all converging
at Sw = 1, Sxo = 1 — consistent with E79.

### 2.14 Secondary porosity

Computed automatically whenever the Sonic tool is **present on the model grid** — the `Use` box may
be off and the sonic need not participate in the solve (`minsolveeqandmeth.htm`,
`[img-read: _imsclip0079.png]`).

| # | Relation | Source |
|---|---|---|
| E83 | Wyllie sonic porosity: `φ = [Dt − Dtma − Vcl × (Dtcl − Dtma)] / [(Dtfl × Sxo + Dthy × (1 − Sxo) − Dtma) × Cp]` | `[img-read: _imsclip0123.png]` |
| E84 | Raymer: `φclay = [(2Vma − Vf) − sqrt((2Vma − Vf)² − 4 × Vma × (Vma − Vclay))]/(2 × Vma)` ; `Vfc = 1/(Dtfl × Sxo + Dthy × (1 − Sxo))` ; `φson = [(2Vma − Vfc) − sqrt((2Vma − Vfc)² − 4 × Vma × (Vma − Vlog))]/(2 × Vma)` ; `φ = φson − φclay × Vcl` | `[img-read: _imsclip0124.png]`, verified at 4× |
| E85 | `PhiSecU = Phie − PhiSonic` (wet clay model) ; `PhiSecU = PhiT − PhiSonic` (dry clay model) ; `PhiSec = PhiSecU` clipped at zero | `(minsolveeqandmeth.htm)` ASCII |

`Vma = 1/Dtma`, `Vf = 1/Dtfl`, `Vclay = 1/Dtclay`, `Vlog = 1/Dt` (`minsolveeqandmeth.htm`).

Volume-weighted `Dtmatrix`, `Dtwater`, `Dthydrocarbon` and `Dtclay` are built from the solved volumes
when multiple mineral/fluid inputs exist; the model's own `Vclay` and `Sxo` then feed E83/E84.
Stated calibration intent: adjust parameters so **average secondary porosity in non-vuggy rock is
zero** — `PhiSecU` is deliberately allowed negative for exactly this QC.

### 2.15 Final calculations

| # | Relation | Source |
|---|---|---|
| E86 | `BVW = φe × Sw` | `[img-read: embim281.png]` |
| E87 | `BVWsxo = φe × Sx` | `[img-read: embim282.png]` |
| E88 | `Rwapp = Rt × φt^m / a` | `[img-read: embim283.png]` |
| E89 | `Rmfapp = Rxo × φt^m / a` | `[img-read: embim284.png]` |
| E90 | `Cwapp = 1 / Rwapp` | `(minsolveeqandmeth.htm)` ASCII |
| E91 | `Qvn = Vcl × φTclay / φT` (normalized Qv, Juhasz W&S plot) | `[img-read: embim285.png]` |
| E92 | `QvApp = a / (B × Rt × φ^m) − 1.0 / (B × Rw)` | `[img-read: embim286.png]`, verified at 4× |
| E93 | `PhiT_recp = 1 / PhiT` | `(minsolveeqandmeth.htm)` ASCII |
| E94 | `BVWIRR = φe × Sxo × (Rw / Rmf) × (Rmf − Rmfeq) / (Rmf − Rw)` | `[img-read: embim287.png]` |
| E95 | `Rt_Hingle = Rt^(−1/m)` | `(minsolveeqandmeth.htm)` ASCII |

`Rwapp`, `Rmfapp` and `Cwapp` are converted back to the temperature entered for `Rw` and `Rmf`
respectively (`minsolveeqandmeth.htm`).

E94 is only computed when the Rxo curve is entered, Sxo comes from the EPT mineral model, and m is
**not** calculated from EPT/Rxo. The `RMFEQ` curve is produced first by solving the flushed-zone
saturation equation for Rmf using the model's Sxo. Limit applied: **`BVW > BVWIRR > 0`**.

Hingle-plot construction (`plot_the_mineral_solver_result.htm`): the Y axis is built from
`(1/Rt)^(−1/m)` but **labelled** in resistivity; the Y-scale lines move when `m` changes; if the
format is shown, the Y values are the actual curve values `Rt^(−1/m)`, not `Rt`.

### 2.16 Calibration

`Input Curve = Min1 × Vol1 + Min2 × Vol2 + Min3 × Vol3 …` — multiple linear regression per equation
row, `Min_i` solved for, `Vol_i` the input core mineral-volume curves (`minsolvecalibrate.htm`).
Stated distinction from ordinary multi-linear regression: **there is no constant term.** Fixed
end-points are subtracted from the input curve before the regression runs. Reports `Corr Coeff` (R²)
and `Num Points` per row. One input volume curve may be left blank — IP sums the others and assigns
the remainder to it; **only one** may be blank. Input volume units must be declared `Dec (V/V)` or
`Percent %`.

---

## 3. Endpoints, parameters, defaults & constraints

### 3.1 Zonal parameter defaults — as shipped in the vendor's worked "Carbonate" example

Read from the parameter tabs. These are the values the vendor ships in its own example; where the
manual separately names something "Default", that is noted.

**Waters/Clays tab** `[img-read: _imsclip0009.png]`

| Parameter | Value | Units |
|---|---|---|
| `Rw` | 0.081 | ohm-m |
| `Rw Temp` | 60 | °F |
| `Rw Salinity` | 114 | Kppm |
| `Rmf` | 0.1 | ohm-m |
| `Rmf Temp` | 60 | °F |
| `Rmf Salinity` | 87.8 | Kppm |
| `Rw bound` | 0.1 | ohm-m |
| `Rwb Temp` | 60 | °F |
| `Rwb Salinity` | 87.8 | Kppm |
| `Rmf bound` | 0.1 | ohm-m |
| `Rmfb Temp` | 60 | °F |
| `Rmfb Salinity` | 87.8 | Kppm |
| `Res Clay` | 1 | ohm-m |
| `Rxo Clay` | 1 | ohm-m |
| `PhiT Clay` | 0.15 | v/v |

Stated fallback (`mineral_solver.htm`): IP first tries to read `Rw` / `Rw Temp` from
Well Header → Default Parameters; if empty it uses **Rw 0.1 at 60 degrees**.

**Sw Logic / Limits tab** `[img-read: _imsclip0010.png]`

| Parameter | Value |
|---|---|
| `Sat Equation` | Archie |
| `Sw Method` | Rt |
| `Sxo Method` | Inv Fac |
| `OBM ?` | on |
| `Sw Sxo Inv Logic` | on |
| `m vari wth Vcl` | off |
| `Vcl cutoff` | 0.6 |
| `Sxo Limit ?` | on |
| `Sxo Limit` | 0.2 |
| `Invasion factor` | 0.5 |
| `Phi Sw Limit` | 0 |
| `Vcl Sw Limit` | 1 |

**Sw Params tab** `[img-read: _imsclip0011.png]`, plus `[img-read: _imsclip0080.png]`

| Parameter | Value |
|---|---|
| `m source` / `n source` | Param / Param |
| `a factor` | 1 |
| `m exponent` | 2 |
| `n exponent` | 2 |
| `min m value` | 1.5 |
| `max m value` | 3 |
| `m plus value` | 0 |
| `B fact Juhasz` | 1 |
| `B fact W&S` | blank (→ calculated from T and Rw) |
| `Qv` | blank, or a curve (`Qv_ms` in the worked example) |
| `Qv 'a' Const` | 0.5 |
| `Qv 'b' Const` | −3 |
| `Cm*` | 1 |

**Sonic / Neutron / Density tab** `[img-read: _imsclip0087.png]`

| Parameter | Value |
|---|---|
| `Sonic Equ` | Wyllie |
| `Sonic Cp` | 1 |
| `Neu Form Sal` | on |
| `Neu Log Cont` | Schlumb |
| `Neu Tool Type` | CNL |
| `Den Hy Model` | **Modified** |

Temp input units: **Fahrenheit**.

### 3.2 Named defaults stated in prose

| Control | Value | Source |
|---|---|---|
| Unity equation tolerance | 0.01 | `minsolveeqandmeth.htm` |
| Sxo auto-equation confidence | 0.01 ("1 saturation unit") | `mineral_solver.htm`, `[img-read: _imsclip0077.png]` |
| Sw auto-equation confidence | 0.01 | same |
| Constant-equation confidence (recommended) | ≈ 0.01 | `minsolveeqandmeth.htm` |
| Outer-loop φe convergence | difference **< 0.001** | `minsolveeqandmeth.htm` |
| Outer-loop Sxo convergence | difference **< 0.002** | `minsolveeqandmeth.htm` |
| Main linearization loop cap | 20 iterations (PHIFLAG 4) | `minsolveeqandmeth.htm` |
| Solver iteration cap | 30 iterations (PHIFLAG 5) | `minsolveeqandmeth.htm` |
| Sw equation loop cap | 10 iterations (PHIFLAG 8) | `minsolveeqandmeth.htm` |
| `Sonic Cp` | Default 1.0 | `mineral_solver.htm` |
| `Cm*` | Default 1.0 | `mineral_solver.htm` |
| `B fact Juhasz` | Default 1.0 **meq/ml** | `mineral_solver.htm` |
| `Sxo Limit` exponent | Default 0.2 | `mineral_solver.htm` |
| Invasion factor, OBM branch | Default 0.5 | `mineral_solver.htm` |
| Invasion Factor, WBM Sxo(Sw) relation | Default **2.0** | `minsolveeqandmeth.htm` |
| No-Calculation flag null | −999 | `mineral_solver.htm` |
| Max models per well | 20 | `mineral_solver.htm` |
| Max mixing rule sets | 5 | `plot_the_mineral_solver_result.htm` |
| Max end-points per crossplot | 8 | `mineral_solver.htm` |

> **Invasion-factor name collision persists in 2025**, exactly as flagged by the 2018 ingest: default
> 0.5 in the OBM context (`mineral_solver.htm`) and 2.0 in the WBM empirical `Sxo(Sw, IF)` relation
> (`minsolveeqandmeth.htm`). Two different parameters sharing a name. Do not merge.

### 3.3 Mineral end-point grids — the big 2025 gain

**IP 2018 published no end-point table at all.** IP 2025 publishes several complete worked model
grids as screenshots. Every value below was read from the image and the three densest grids were
re-read at 1.9×–2× to confirm digits.

#### 3.3.1 Carbonate model — the flagship example `[img-read: _imsclip0021.png / _imsclip0020.png]`

Confidences: Unity 0.01, Density 0.02, Neutron 0.02, Sonic 3, U 0.4, GammaRay 5. Invasion factor 1.0
on every row.

| Equation | Quartz (Matrix) | Calcite (Matrix) | Dolomite (Matrix) | Clay (**Wet Clay**) | Water Sxo | Oil Sxo (Hyd. Sxo) |
|---|---|---|---|---|---|---|
| Unity | 1 | 1 | 1 | 1 | 1 | 1 |
| Density (g/cc) | 2.65 | 2.71 | 2.85 | **2.429** | Auto | 0.8 *(true HC density)* |
| Neutron (v/v) | Auto | Auto | Auto | **0.373** | Auto | 0.8 *(true HC density)* |
| Sonic (µs/ft) | 55 | 47 | 42 | **100** | 189 | 200 |
| **U (b/cm³)** | **4.8** | **13.8** | **9** | **7.454** | Auto | 0.8 *(true HC density)* |
| GammaRay (API) | 10 | 10 | 10 | **238.4** | 0 | 0 |

#### 3.3.2 Clay-mineral discrimination model — Kaolinite vs Illite `[img-read: _imsclip0082.png, _imsclip0084.png]`

Both wet clays. Confidences: Unity 0.01, Density 0.02, Neutron 0.02, Sonic 3, GR 5.

| Equation | Calcite | Quartz | **Kaolinite** | **Illite** | Water Sxo | Oil Sxo |
|---|---|---|---|---|---|---|
| Density (g/cc) | 2.71 | 2.65 | **2.55** | **2.61** | Auto | 0.8 |
| Neutron (v/v) | Auto | Auto | **0.51** | **0.35** | Auto | 0.8 |
| Sonic (µs/ft) | 47 | 55 | **120** | **100** | 189 | 200 |
| GammaRay (API) | 20 | 20 | **170** | **150** | 20 | 20 |
| `Res Clay` output (ohm-m) | 0 | 0 | **2.5** | **1.2** | 0 | 0 |
| `Output Para` → m | **1.8** | **2.05** | 0 | 0 | 0 | 0 |

#### 3.3.3 Illite / Chlorite clay-porosity model `[img-read: _imsclip0070.png, _imsclip0060.png]`

| Quantity | Illite | Chlorite |
|---|---|---|
| `PhiTClay` output end-point (wet-clay total porosity, v/v) | **0.156** | **0.101** |
| `BoundWater` constant-equation coefficient (dry-clay model) | **0.185** | **0.112** |

The BoundWater row carries `−1.` on the BoundWater column. Arithmetic self-consistency confirmed
against E34: 0.156/(1−0.156) = 0.1848; 0.101/(1−0.101) = 0.11235.

#### 3.3.4 ECS wet-clay model `[img-read: _imsclip0050.png, verified at 1.9×]`

Confidences: Unity 0.01, all four ECS Wt% rows 0.02, Neutron 0.04, Density 0.02. IF 1.0 throughout.

| Equation | Quartz | Clay (**Wet**) | Water Sxo | Oil Sxo | Siderite | Dolomite | Calcite |
|---|---|---|---|---|---|---|---|
| Unity | 1. | 1. | 1. | 1. | 1. | 1. | 1. |
| ECS QFM (Wt%) | 1. | 0. | 0. | 0. | 0. | 0. | 0. |
| ECS LsDol (Wt%) | 0. | 0. | 0. | 0. | 0. | 1. | 1. |
| ECS Clay (Wt%) | 0. | **0.85** | 0. | 0. | 0. | 0. | 0. |
| ECS Sid (Wt%) | 0. | 0. | 0. | 0. | 1. | 0. | 0. |
| Neutron | Auto | **0.4** | Auto | 0.8 | **0.18** | Auto | Auto |
| GrainDensity (output) | 2.65 | **2.78** | 0. | 0. | 3.88 | 2.85 | 2.71 |
| Density | 2.65 | **2.4** | Auto | 0.8 | 3.88 | 2.85 | 2.71 |

**The wet/dry clay density split is explicit here and it matters:** the clay's *density-equation*
end-point is **2.4** (wet clay, includes its bound water) while its *grain-density-equation*
end-point is **2.78** (dry grain density). The prose confirms: the ECS measures dry clay, so the wet
clay `ECS_Clay (Wt%)` end-point is **0.85**, not 1.0 — a 100 % wet-clay zone with 0.15 clay porosity
reads 0.85 after weight→volume conversion. For a **dry** clay model that parameter is **1.0**
(`minsolveeqandmeth.htm`).

#### 3.3.5 ECS dry-clay model `[img-read: _imsclip0092.png, verified at 1.9×]`

Same minerals plus a `BoundWater` (Bound Water) column. Confidences: Unity 0.01, ECS rows 0.02,
Neutron 0.02, Density 0.02, BoundWater 0.01.

| Equation | Quartz | Clay (**Dry**) | Water Sxo | Oil Sxo | Siderite | Dolomite | Calcite | BoundWater |
|---|---|---|---|---|---|---|---|---|
| ECS Clay (Wt%) | 0 | **1** | 0 | 0 | 0 | 0 | 0 | 0 |
| Neutron | Auto | **0.35** | Auto | 0.8 | 0.18 | Auto | Auto | **1** |
| GrainDensity (output) | 2.65 | **2.78** | 0 | 0 | 3.88 | 2.85 | 2.71 | 0 |
| Density | 2.65 | **2.78** | Auto | 0.8 | 3.88 | 2.85 | 2.71 | **1** |
| BoundWater (constant) | 0 | **0.15** | 0 | 0 | 0 | 0 | 0 | **−1** |

In the dry-clay model the density end-point for clay is **2.78** (the dry grain density) and bound
water carries density **1** and neutron **1** as separate fluid.

#### 3.3.6 Shale-gas / TOC model `[img-read: _imsclip0093.png, verified at 1.9×]`

Confidences: Unity 0.01, Density 0.02, Neutron 0.02, Sonic 3, GammaRay 5, TOC(Wt%) 0.02, **U 0.2**.

| Equation | Quartz | Clay (**Wet**) | Water Sxo | **Gas Sxo** | **Kerogen** | **Pyrite** |
|---|---|---|---|---|---|---|
| Density (g/cc) | 2.65 | **2.65** | Auto | 0.2 *(true)* | **1.1** | **4.99** |
| Neutron (v/v) | Auto | **0.35** | Auto | 0.2 *(true)* | **0.6** | **0.01** |
| Sonic (µs/ft) | 55 | **100** | 189 | 220 | **150** | **39.2** |
| GammaRay (API) | 20 | **150** | 20 | 20 | **120** | 20 |
| TOC (Wt%) | 0 | 0 | 0 | 0 | **1** | 0 |
| GrainDensity (output) | 2.65 | **2.78** | 0 | 0 | **1.1** | **4.99** |
| **U (b/cm³)** | **4.8** | **10** | Auto | 0.2 *(true)* | **0.264** | **82** |

Stated setup rule for shale gas (`minsolveeqandmeth.htm`): kerogen grain density goes in the
grain-density output equation, and **the clay grain density must be the *dry* grain density** because
TOC is measured on the dry rock.

#### 3.3.7 Other model grids read

| Grid | Values | Source |
|---|---|---|
| Conductivity model | Clay density 2.446, neutron 0.3671. `Cxo Archie Lin` conf **10**, IF 1: Quartz 0, Calcite 0, **Clay 250 mmho**, **Water Sxo 16000 mmho**, Oil 0 | `[img-read: _imsclip0052.png]` |
| Resistivity model | Same densities. `Res. Rxo` conf **200**, IF 1: **Clay 4 ohm-m**, **Water Sxo 0.063 ohm-m** | `[img-read: _imsclip0055.png]` |
| Pyrite constant-equation model | Calcite 2.71 / Quartz 2.65 / Clay 2.64 density; Clay neutron 0.35; Sonic 47 / 55 / **90** / 189 / 200; **Pyrite density 4.99, neutron 0.01, sonic 39.2**. Constant `0.02 = VPyrite` at conf 0.01 | `[img-read: _imsclip0058.png]` |
| Dry-clay bound-water model | Calcite 2.71 / Quartz 2.65 / **Clay 2.87 (dry)** / BoundWater 1. Neutron: **Clay 0.15**, BoundWater 1. Sonic 47 / 55 / **65** / 189 / 200 / BoundWater 189. Constant `0.0`: **Clay 0.15, BoundWater −0.85** | `[img-read: _imsclip0059.png]` |
| HC-corrected-output model | Calcite 2.71 / Quartz 2.65 / Dolomite 2.85 / **Clay 2.414**; Neutron Clay **0.3929**; Sonic 47 / 55 / 42 / **94.83**; U 13.8 / 4.8 / 9 / **6.429**; GR 10 / 10 / 10 / **217.5**; Oil true density **0.7** | `[img-read: _imsclip0121.png]` |
| Saturation-output model (oil + gas) | Density 2.71 / 2.65 / Clay 2.65 / Oil 0.8 / Gas 0.2; Neutron Clay 0.35; Sonic 47 / 55 / 100 / 200 / 189 / 220; `Res. Rxo` conf 100: **Clay 3.**, Water **0.09**; GR 20 / 20 / **180** / 20; `Cond. Cxo` conf 10: **Clay 250**, Water **18000**; GrainDensity 2.71 / 2.65 / **2.75** | `[img-read: _imsclip0063.png]` |
| Output Dry Wt% model | Density Calcite 2.71 / Quartz 2.65 / **Clay 2.65**; GrainDensity 2.71 / 2.65 / **2.73**; `VclayWt%` row: **Clay 0.85** | `[img-read: _imsclip0119.png]` |
| Auto-endpoint resistivity model (invaded + un-invaded) | Density 2.65 / 2.71 / **Clay 2.73 (dry)** / BoundWater 1; Neutron Clay **0.25**; Sonic conf 10: 55 / 47 / **70** / 220 / 189 / 189 / 200 / 189; GR conf 10: 25 / 25 / **129.8**; `Cond. Cxo` IF 1 and `Cond. Ct` IF **0**, water end-points `Auto` | `[img-read: _imsclip0107.png]` |
| Grain-density output example | Calcite 2.71, Quartz 2.65, Clay/fluids 0. — "clean" grain density | `[img-read: _imsclip0067.png]` |
| Sonic-matrix output example | Calcite 47, Quartz 55, Clay/fluids 0 | `[img-read: _imsclip0118.png]` |
| Qv output example | Illite **0.59**, Chlorite **0.38** (Qv end-point parameters) | `[img-read: _imsclip0068.png]` |
| PhiLimit example | `0.3` as `> Limit`, conf 0.01: Quartz 0., Clay 0., Water 1., Oil 1. | `[img-read: _imsclip0065.png]` |
| Calcite max-limit example | `0.2` as `> Limit`, conf 0.01, Calcite 1. | `[img-read: _imsclip0066.png]` |

#### 3.3.8 MINDEF.PAR mineral grain densities exposed by the Preprocessor `[img-read: _imsclip0003.png]`

The Dry Weight to Volume Conversion tab shows the mineral-density defaults it pulls from
`MINDEF.PAR`, with the vendor's own parenthetical mineral labels:

| Weight curve | Mineral Density (gm/cc) | Vendor label |
|---|---|---|
| XSIO | 2.65 | (Quartz) |
| XKF | 2.57 | (Orthoclase) |
| XPL | 2.59 | *(no label printed)* |
| XCA | 2.71 | (Calcite) |
| XSI | 3.88 | (Siderite) |
| XAN | 2.74 | *(no label printed)* |
| XFE | 4.99 | (Pyrite) |
| XKA | 2.55 | (Kaolinite) |

XPL and XAN are **not labelled in the source**; their mineral identity is an OPEN ITEM (§8), not
guessed here.

### 3.4 SPECIAL TASKING (d) — every clay-mineral endpoint the pages state

Consolidated. Every value carries its model context, because IP's clay end-points are
**model-convention-dependent** (wet clay vs dry clay changes the number).

| Clay | Property | Value | Units | Model convention | Source |
|---|---|---|---|---|---|
| Generic Clay | Density | **2.429** | g/cc | Wet Clay, carbonate model | `[img-read: _imsclip0021.png]` |
| Generic Clay | Density | **2.446** | g/cc | Wet Clay, cond./res. models | `[img-read: _imsclip0052/0055.png]` |
| Generic Clay | Density | **2.414** | g/cc | Wet Clay, HC-corr model | `[img-read: _imsclip0121.png]` |
| Generic Clay | Density | **2.4** | g/cc | Wet Clay, ECS model | `[img-read: _imsclip0050.png]` |
| Generic Clay | Density | **2.64** | g/cc | Wet Clay, pyrite model | `[img-read: _imsclip0058.png]` |
| Generic Clay | Density | **2.65** | g/cc | Wet Clay, shale-gas + Wt% + sat-output models | `[img-read: _imsclip0093/0119/0063.png]` |
| Generic Clay | Density | **2.78** | g/cc | **Dry** Clay (also the dry grain density used in every GrainDensity output row) | `[img-read: _imsclip0092/0050/0093.png]`, prose |
| Generic Clay | Density | **2.87** | g/cc | **Dry** Clay, bound-water model | `[img-read: _imsclip0059.png]` |
| Generic Clay | Density | **2.73** | g/cc | **Dry** Clay, auto-endpoint model; also GrainDensity 2.73 in the Wt% model | `[img-read: _imsclip0107/0119.png]` |
| Generic Clay | Density | **2.75** | g/cc | GrainDensity output, sat-output model | `[img-read: _imsclip0063.png]` |
| Generic Clay | Neutron | **0.373 / 0.3671 / 0.3929 / 0.4 / 0.35** | v/v | Wet Clay (varies by example) | `[img-read: _imsclip0021/0052/0121/0050/0093.png]` |
| Generic Clay | Neutron | **0.35 / 0.25 / 0.15** | v/v | **Dry** Clay | `[img-read: _imsclip0092/0107/0059.png]` |
| Generic Clay | Sonic | **100 / 94.83 / 90** | µs/ft | Wet Clay | `[img-read: _imsclip0021/0121/0058.png]` |
| Generic Clay | Sonic | **70 / 65** | µs/ft | **Dry** Clay | `[img-read: _imsclip0107/0059.png]` |
| Generic Clay | **U** | **7.454 / 6.429 / 10** | b/cm³ | Wet Clay | `[img-read: _imsclip0021/0121/0093.png]` |
| Generic Clay | GammaRay | **238.4 / 217.5 / 180 / 150** | API | Wet Clay | `[img-read: _imsclip0021/0121/0063/0093.png]` |
| Generic Clay | GammaRay | **129.8** | API | **Dry** Clay | `[img-read: _imsclip0107.png]` |
| Generic Clay | Conductivity | **250** | mmho | wet clay conductivity example (= 4.0 ohm-m per prose) | `[img-read: _imsclip0052/0063.png]`, prose |
| Generic Clay | Resistivity | **4 / 3** | ohm-m | wet clay res. example | `[img-read: _imsclip0055/0063.png]`, prose |
| Generic Clay | Total clay porosity `PhiT Clay` | **0.15** | v/v | zonal parameter default in the carbonate example | `[img-read: _imsclip0009.png]` |
| Generic Clay | `ECS_Clay (Wt%)` end-point | **0.85** wet / **1.0** dry | v/v | prose + grid | `(minsolveeqandmeth.htm)`, `[img-read: _imsclip0050/0092.png]` |
| **Kaolinite** | Density | **2.55** | g/cc | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Kaolinite** | Density (MINDEF.PAR) | **2.55** | gm/cc | dry mineral grain density | `[img-read: _imsclip0003.png]` |
| **Kaolinite** | Neutron | **0.51** | v/v | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Kaolinite** | Sonic | **120** | µs/ft | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Kaolinite** | GammaRay | **170** | API | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Kaolinite** | `ResClay` | **2.5** | ohm-m | Wet Clay | `[img-read: _imsclip0082.png]`, prose |
| **Illite** | Density | **2.61** | g/cc | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Illite** | Neutron | **0.35** | v/v | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Illite** | Sonic | **100** | µs/ft | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Illite** | GammaRay | **150** | API | Wet Clay | `[img-read: _imsclip0082.png]` |
| **Illite** | `ResClay` | **1.2** | ohm-m | Wet Clay | `[img-read: _imsclip0082.png]`, prose |
| **Illite** | `PhiTClay` | **0.156** | v/v | Wet Clay | `[img-read: _imsclip0070.png]`, prose |
| **Illite** | `Qv` output end-point | **0.59** | *unit not stated* | Wet Clay | `[img-read: _imsclip0068.png]` |
| **Illite** | BoundWater constant coefficient | **0.185** | — | Dry Clay | `[img-read: _imsclip0060.png]` |
| **Chlorite** | `PhiTClay` | **0.101** | v/v | Wet Clay | `[img-read: _imsclip0070.png]`, prose |
| **Chlorite** | `Qv` output end-point | **0.38** | *unit not stated* | Wet Clay | `[img-read: _imsclip0068.png]` |
| **Chlorite** | BoundWater constant coefficient | **0.112** | — | Dry Clay | `[img-read: _imsclip0060.png]` |

> **SMECTITE / MONTMORILLONITE: NOT PRESENT.** Neither word appears anywhere on the six assigned
> pages, in any prose, table or readable grid. The 2018 corpus sweep found the same. **The standing
> SandiMin smectite review gets nothing from IP's Mineral Solver help.** Its clay roster in worked
> examples is Kaolinite, Illite, Chlorite, plus a generic "Clay". Any smectite end-point must come
> from `MINDEF.PAR` on disk or from another source entirely.
>
> **No CEC value for any clay is stated anywhere.** The Qv end-points (0.59 Illite, 0.38 Chlorite)
> are *composites* — by E45 they equal `ρdcl × CECdcl × (1 − φTclay)` — so a CEC cannot be
> back-solved without independently knowing ρdcl. Not attempted here.

### 3.5 Non-clay mineral end-points observed

| Mineral | Density (g/cc) | Neutron (v/v) | Sonic (µs/ft) | U (b/cm³) | GR (API) | Sources |
|---|---|---|---|---|---|---|
| Quartz | 2.65 | Auto | 55 | 4.8 | 10 / 20 / 25 | `_imsclip0021/0093/0050/0107` |
| Calcite | 2.71 | Auto | 47 | 13.8 | 10 / 20 / 25 | `_imsclip0021/0082/0107` |
| Dolomite | 2.85 | Auto | 42 | 9 | 10 | `_imsclip0021/0050` |
| Siderite | 3.88 | 0.18 | — | — | — | `_imsclip0050/0092` |
| Pyrite | 4.99 | 0.01 | 39.2 | 82 | 20 | `_imsclip0093/0058` |
| Kerogen | 1.1 | 0.6 | 150 | 0.264 | 120 | `_imsclip0093` |
| Orthoclase | 2.57 (MINDEF) | — | — | — | — | `_imsclip0003` |
| Water (fresh) | 1.0 | — | 189 | — | 0 / 20 | prose + grids |
| Oil (true downhole) | 0.8 / 0.7 | 0.8 / 0.7 | 200 | 0.8 / 0.7 | 0 / 20 | grids |
| Gas (true downhole) | 0.2 | 0.2 | 220 | 0.2 | 20 | `_imsclip0093/0063` |
| Bound Water | 1 | 1 | 189 | — | — | `_imsclip0092/0059/0107` |

Water sonic 189 µs/ft and oil 200 / gas 220 µs/ft recur in every grid that carries a sonic row.

### 3.6 End-point entry conventions (unchanged from 2018, re-confirmed)

| Convention | Statement |
|---|---|
| **Blue cell = Auto-calculable.** Enter `Auto` and IP computes it. | For the **Neutron** equation the **Calcite, Quartz and Dolomite parameters must be left at `Auto`** for IP to use the non-linear neutron equations (`mineral_solver.htm`). |
| **Water `Auto`** | Density and HI computed from the `Rmf` parameter corrected to depth and temperature; for OBM or the un-invaded zone, from `Rw`. For conductivity/resistivity equations, `Auto` on a `Water Sxo` end-point is computed from `Rmf` and `Rmf Temp` (`mineral_solver.htm`, `[img-read: _imsclip0107.png]`). |
| **Green cell = true downhole hydrocarbon density.** | IP converts internally to the tool response (electron density for RHOB, hydrogen index for NPHI). Enabled by three independent Model Options checkboxes — Density, Neutron and **U** each separately (`[img-read: _imsclip0077.png]`). |
| **Any end-point may be a curve.** | Trend curves vary a parameter level by level (`mineral_solver.htm`). |
| **Confidence may be a curve.** | Explicitly recommended for washed-out hole where density accuracy degrades (`mineral_solver.htm`). |
| **Invasion factor may be a curve**, values 0–1. | `mineral_solver.htm` |
| **Mineral Types, exhaustive** | `Water Sxo`, `Bound Water`, `Hyd. Sxo`, `Matrix`, `Wet Clay`, `Dry Clay`, `Water Sw`, `Hyd. Sw` — read directly from the drop-down `[img-read: _imsclip0098.png]`, matching the prose list. |
| **Defaults live in two ASCII files** | `MINDEF.PAR` (minerals + properties) and `MINEQDEF.PAR` (equation defaults), both in the IP directory, both user-editable via Tools → Defaults. "Names that are not in the drop-down list will not have any default end-point values defined." Selecting the **Equation type first** then picking minerals auto-populates end-points (`mineral_solver.htm`). |

---

## 4. Solver behaviour & assumptions — SPECIAL TASKING (b)

### 4.1 Two solvers in series

| | Linear solver | Non-linear solver |
|---|---|---|
| Technique | **Singular Value Decomposition** | **DNOPT Dense Nonlinear OPTimizer** |
| Attribution given | "Numerical Recipes (Cambridge University Press)" | "from Stanford University" |
| Role | solves the normalised linear system; "very fast and stable" | seeded by the linear solution, searches for lower total error |
| Resistivity/conductivity | **linearised Archie with n = m only** | the **actual** Sw equation from Zonal/Model Parameters |
| Sonic | Wyllie with the Cp approximation (E19) | full sonic equations, no Cp approximation |
| Cost | — | "slower to run … but it generally finds a marginally better solution" |

Sources: `minsolveeqandmeth.htm`, `mineral_solver.htm`. Default selection in the Model Options
dialog is **"Linear equation Solver with non-linear equation end point iteration"**
`[img-read: _imsclip0077.png, _imsclip0097.png]`.

**Selection rule, stated three separate times:** the non-linear optimizer's solution is always
checked against the linear solver's and **the best (lowest total error) solution is always
selected.**

### 4.2 The objective

Minimise **Total Model Error** (E1) — the square root of the summed squared per-curve misfits, each
normalised by that curve's own tolerance. Resistivities are converted to conductivities and
square-rooted before entering. See §2.2 for the "no 1/N" structural note.

### 4.3 The linear-solver algorithm, exactly as stated (`minsolveeqandmeth.htm`)

1. Each equation is normalised by dividing all its terms by its confidence weighting.
2. Equations are solved by Singular Value Decomposition.
3. **Negative-volume handling:** if any volume is negative, the **largest negative term is set to
   zero and removed from the model**, the solver re-runs, and this repeats until all volumes are
   positive. *(This is IP's entire non-negativity mechanism — active-set removal, not a bounded
   optimisation.)*
4. Result volumes are adjusted to sum to 1.0. Stated caveat: "Due to the way the equation solver
   works, the unity equations will not necessarily force the results to absolutely 1.0. (The
   tolerance of the unity equation is set at **0.01** by default)."
5. Input logs are reconstructed from the volume results.
6. Total normalised error is computed (E1).

Then the non-linear optimiser runs from the linear solution as its starting point.

### 4.4 Weighting / uncertainty scheme

- Per-equation, named **Confidence**. **"The smaller the confidence number, the more the weight that
  will be attached to this equation (exception to this is the resistivity equation)"**
  (`mineral_solver.htm`).
- Confidence is expressed **in the physical units of the equation** — density in gm/cc, gamma ray in
  API. It is a tool-accuracy figure, not an abstract weight.
- Confidence may be a **curve**.
- For conductivity/resistivity the confidence is transformed with the curve (1/m-th root; see §2.8
  worked table). The result-track shading width therefore reflects the rooted confidence, not the raw
  one.
- Per-input-curve uncertainty outputs: `***_re` (reconstructed), `***_me` (minus-error), `***_pe`
  (plus-error), where `***` is the input curve name (`mineral_solver.htm`).

### 4.5 Constraint handling

| Constraint | Form | Handling |
|---|---|---|
| **Unity** | `1 = ΣVol_i` | always included; **soft**, tolerance 0.01, results renormalised in step 4 |
| **Non-negativity** | `Vol_i ≥ 0` | **not a solver constraint** — enforced by the iterative removal in step 3 |
| **Porosity equalisation** | E32 | auto-added when both invaded and un-invaded fluids are present |
| **Sxo equation** | E36 | auto-added by default; confidence 0.01 |
| **Sw equation** | E38 | auto-added optionally (for un-invaded fluids); confidence 0.01 |
| **Constant** | user linear relation | confidence ≈ 0.01 recommended |
| **`<Limit` / `>Limit`** | inequality on a mineral/fluid or a group | see below |
| **Invasion Factor** | per-equation, 0.0–1.0 | end-points × IF for invaded fluids, × (1−IF) for un-invaded; may be a curve |

**Limit-equation algorithm (stated explicitly, `minsolveeqandmeth.htm`):** IP first solves the model
**without any limit equations**, validates each limit, and if a result falls outside a limit **adds
that one equation and completely re-solves**, then repeats. Limits are added **one at a time**, and
**the order they are added is the order they are entered in the grid.** Once added they are treated
as constant equations weighted by their Confidence — so **the result can still fall outside the
limit**. The manual warns limits should make **minor adjustments only**: its own worked failure case
shows a 0.05 porosity limit forced onto a washed-out interval producing badly reconstructed curves
and a large total error, and prescribes building a bad-hole model that drops the density curve
instead.

Note the sense: `> Limit` means the mineralogy result must be **less than** the entered Curve/Val
(the entered value sits on the left of the inequality — E39 reads `0.3 > Vwater + Vhydrocarbon`).
`< Limit` means the result must be **greater than** it (`minsolveeqandmeth.htm`, confirmed against
`[img-read: _imsclip0065.png]`).

### 4.6 Convergence and iteration control

Outer loop repeats until **both**: `φe difference < 0.001` and `Sxo difference < 0.002`
(`minsolveeqandmeth.htm`). Caps: main linearization loop 20, solver 30, Sw equation loop 10.

Documented flow `[img-read: _imsclip0049.png]` — the Mineral Solver Logic Flow Diagram, fully
legible. Sequence: Initialize Phi/Sxo/Sw → Set automatic endpoints based on Phi/Sxo/Sw → optionally
add Sxo equation → optionally add Sw equation → if un-invaded fluids, add the flushed/un-invaded
equal-porosity equation → **Calculate mineral/fluid volumes (Linear Solver)** → calculate porosities
from fluid volumes → Sxo either from mineral fluid volumes or from the Rxo curve via the selected Sxo
equation → Sw likewise from fluid volumes or from Rt → **check convergence of Phi and Sw**; if not
converged, loop back to the endpoint step; if converged, either refine with the non-linear optimizer
or go straight to final results (BVW, BVWSXO etc).

### 4.7 Quality / incoherence indicators

- **`TotErr`** — Model Normalized Total Error, plotted in its own track; **values greater than 1.0
  are displayed in red** (`plot_the_mineral_solver_result.htm`).
- **`PHIFLAG`** — logic/error flag. "For a normal execution of IP at any depth level, the PHIFLAG
  should be zero." Complete table as printed (`minsolveeqandmeth.htm`) — **identical to 2018**:

| PHIFLAG | Meaning |
|---|---|
| 2 | A Limit equation was used in the model results |
| 4 | Main linearization loop did not converge after 20 iterations |
| 5 | Solver did not converge after 30 iterations |
| 6 | Sw curve set to 1.0 due to Phi Sw Limit or Vcl Sw Limit parameter limits being reached |
| 7 | Sw limited to Sw irreducible parameter |
| 8 | Sw equation loop did not converge after 10 iterations |
| 9 | Fatal Error with non-linear optimizer. See error log |
| 10 | Non-linear optimizer. Unbounded Objective |
| 11 | Non-linear optimizer. Iteration limit reached |
| 12 | Non-linear optimizer. Major iteration limit reached |
| 14 | Non-linear optimizer. Terminated during objective evaluation |

(0, 1, 3 and 13 are not listed.) Interpretation guidance as printed: error 9 is serious and the level
falls back to the linear solver's answer; errors 10–14 mean the optimizer struggled but the results
"are probably still ok and will be better than the linear solution", and usually indicate the model
does not fit the data.

- **Visual QC:** one reconstruction track per input equation — original curve, yellow **Confidence
  Band**, reconstructed curve in red. Reconstructed Rt/Rxo tracks are optional and carry **no**
  confidence shading because they are not part of the mineral model; they indicate how well the
  **Sw equation** works, especially in water zones (`plot_the_mineral_solver_result.htm`,
  `mineral_solver.htm`).
- Complete output-curve roster read from the Curves tab `[img-read: _imsclip0078.png]`: `Sxou`,
  `SxoT`, `SxoTu`, `BVW`, `BVWsxo`, `Swb`, `PhiFlg`, `mVar`, `BVWirr`, `RmfEq`, `Rwapp`, `Rmfapp`,
  `NormQv`, `Cwapp`, `QvApp`, `PhiT_recp`, `TotErr`, `PhiSec`, `PhiSecU`.

### 4.8 Multi-model combination (Mixings)

Up to **20 models** per well, combined zone by zone by a **Mixing** (up to **5** rule sets). Rules
are evaluated **top-down and stop at the first true statement**; if none is true the Mixing's
Default Model is used. `and`/`or` join adjacent lines and are resolved pairwise before the next line
is read; the manual's own worked precedence examples: `Line1 or Line2 and Line3` is true if Line3 is
true **and** either Line1 or Line2 is true; `Line1 and Line2 or Line3` is true if Line3 is true
regardless (`plot_the_mineral_solver_result.htm`).

**`Mdl Merge Dist`** smooths transitions. Worked example as printed: 2.0 in a 0.5-step set gives a
4-sample transition at 20/80, 40/60, 60/40, 80/20. Implementation stated: set each `Model_Num` array
column to 1 or 0, box-filter each column with filter length = `Mdl Merge Dist`, then normalise
columns to sum 1.0 per level. **Transitioned:** porosity, BVW, Sw, Sxo, Vcl, mineral volumes.
**Not transitioned:** the combined reconstructed curves (`mineral_solver.htm`).

### 4.9 Stated assumptions worth carrying into SandiMin

1. **Flushed-zone-only fluids is the normal setup.** Lithology tools read shallow and are assumed to
   read the flushed zone; Sw is computed *after* the model from porosity and the deep resistivity
   (`mineralsolver.htm`).
2. Un-invaded fluids are optional and only needed when an equation's Invasion Factor < 1.0. The
   neutron is called out by name as the tool that often reads deeper than the density and may need an
   IF closer to 0.0 (`mineralsolver.htm`, `minsolveeqandmeth.htm`).
3. **Un-invaded-zone porosity is forced equal to invaded-zone porosity** by E32.
4. **You should input at least as many tools/equations as there are minerals and fluids**
   (`mineralsolver.htm`).
5. A single mineral model rarely covers a whole well — the vendor expects separate models for
   carbonate, clastic and bad-hole intervals, combined by Mixings (`mineralsolver.htm`).
6. Neutron porosity input is assumed to be in **limestone units**.
7. **Cased-hole recipe** (`mineral_solver.htm`): Rt and Rxo curves absent; `Sw Method = No Calc`;
   `Sxo Method = Min Model`; saturation from the input sigma curve inside the model; **Rmf must still
   be entered** because it drives the automatically-calculated neutron parameters; both auto-Sw and
   auto-Sxo equations turned off. Outputs are `Sxo` and `SxoT` (differing by clay bound water); `Sw`
   and `SwT` are absent. **No sigma equation type, sigma end-point convention or sigma unit is named
   anywhere on these six pages.**
8. `No Calculation` flag curve: any value other than 0 or null (−999) suppresses calculation at that
   level, setting Sw = 1.0, porosities = 0.0 and mineral volumes = 0.0 (`mineral_solver.htm`).
9. Elan import (`mineral_solver.htm`): `.elp` files load **minerals, fluids, input equations and
   end-point values only**. IP will **not** load model constraints or mixing rules — "These are not
   directly compatible with IP." Sw is calculated differently in the two products, and the wet/dry
   clay model must be sorted out manually. Mapping in `ElanToIPMapping.par`.

---

## 5. Internal discrepancies

### 5.1 Shell cementation formula — image says 0.018, ASCII says 0.019 (BOTH VERSIONS)

| Source | Value |
|---|---|
| IP 2025 raster `[img-read: embim275.png]`, verified at 4× | `m = 1.87 + **0.018** / φe` |
| IP 2025 ASCII (`mineral_solver.htm`, Sw Params → `m source` → Shell) | `m = 1.87 + **0.019** / Phie` |
| IP 2018 raster `[img-read: c18/embim284.gif]`, verified at 6× | `m = 1.87 + **0.018** / φe` |
| IP 2018 ASCII (`mineral_solver.htm`) | `m = 1.87 + **0.019** / Phie` |

**This is a longstanding vendor self-contradiction, not a 2025 regression.** Both renderings have
disagreed with each other across at least two releases. The published Shell formula uses **0.019**,
so the ASCII agrees with the literature and the raster does not — but the manual states both and this
report does not pick a winner. **SandiMin must not implement either silently.**

Practical magnitude: at φe = 0.10, m = 2.05 (0.018) vs 2.06 (0.019) — small. At φe = 0.02,
m = 2.77 vs 2.82 — a ~5 % difference in the Archie exponent in tight rock, which propagates as
roughly a 5–10 % Sw error there. Not negligible in low-porosity pay.

### 5.2 Juhasz and Waxman-Smits — the raster carries an `Rw` factor the ASCII drops

| Equation | Raster (this ingest) | ASCII (`mineral_solver.htm`) |
|---|---|---|
| Juhasz | `1/Rt = (φT^m SwT^n / (a Rw)) × (1 + Bn · Qvn · **Rw** / SwT)` | `1/Rt = PhiT**m.SwT**n.(1+Bn.Qvn/SwT)/(a.Rw)` |
| Waxman-Smits | `1/Rt = (φT^(**m***) SwT^n / (a Rw)) × (1 + B · Qv · **Rw** / SwT)` | `1/Rt = PhiT**m.SwT**n.(1+B.Qv/SwT)/(a.Rw)` |

Two differences, both real:
1. The ASCII omits the `× Rw` multiplying the `Bn·Qvn` / `B·Qv` term. The raster form is the correct
   factorisation of `Ct = φ^m Sw^n (Cw + B·Qv/Sw)/a` after pulling out `Cw = 1/Rw`; **the ASCII form
   is dimensionally wrong** (it adds a conductivity to a dimensionless 1).
2. The ASCII prints `PhiT**m` for Waxman-Smits where the raster prints `φT^(m*)`. The `m*` is
   correct — `mineral_solver.htm` itself elsewhere states that a variable `m*` is used for W&S and
   Dual Water.

**Prefer the raster.** Recorded here because the 2018 ingest transcribed only the ASCII (it could not
read the rasters), so the 2018 report currently carries the dimensionally-wrong form.

### 5.3 Bound-water constant coefficient — two conventions shown in the same manual

| Source | Dry clay coefficient | Bound water coefficient | Implied relation |
|---|---|---|---|
| `[img-read: embim237.png]` (equation) | 0.15 | **−0.85** | `0.15 = VBW/(VdryClay + VBW)` |
| `[img-read: _imsclip0059.png]` (screenshot) | 0.15 | **−0.85** | consistent with E33 |
| `[img-read: embim238.png]` (multi-clay equation) | `φTclay/(1−φTclay)` | **−1** | `VBW = ΣVdryClay·φ/(1−φ)` |
| `[img-read: _imsclip0060.png]` (screenshot, Illite/Chlorite) | 0.185 / 0.112 | **−1** | consistent with E34 |
| `[img-read: _imsclip0092.png]` (ECS dry-clay screenshot) | **0.15** | **−1** | **inconsistent with both** |

The first four are two internally-consistent formulations of the same physics. **The fifth is not:**
with a `−1` bound-water coefficient the clay coefficient must be `φ/(1−φ)`, so `0.15` there implies
φTclay = 0.1304, not 0.15. Either the example is loose or it is a typo. Flagged, not resolved.

### 5.4 Conductivity generalisation — `/a` on the water term only

E30 `[img-read: embim236.png]`, verified at 4×: the water summation carries `(Cwat_i / a)^(1/m)` and
the conductive-mineral summation carries `(Cmin_i)^(1/m)` with **no** `/a`. Asymmetric. Confirmed as
printed; no ASCII counterpart exists to cross-check.

### 5.5 Resistivity confidence worked example — ± labels inverted

`minsolveeqandmeth.htm` labels `+error = 1.661 ohmm` and `−error = 2.485 ohmm`. The larger
conductivity (602.2 mmho) is the *lower* resistivity, so the labels are the wrong way round.
**Unchanged since 2018.** Transcribed as printed, not corrected.

### 5.6 Neutron look-up table outlier persists

The `-.1960` entry at φ = .20 in the Dolomite/50 kppm column, flagged by the 2018 ingest as almost
certainly a mis-keyed `-.0196`, is **byte-identical in 2025**. So is the milder φ = .25 sand/100 kppm
non-monotonicity. The vendor has not corrected either.

### 5.7 Invasion factor default name collision persists

0.5 (OBM branch, `mineral_solver.htm`) vs 2.0 (WBM empirical `Sxo(Sw)`, `minsolveeqandmeth.htm`).
Two different parameters, one name. Unchanged from 2018.

### 5.8 Duplicated paragraphs in the 2025 source

Two blocks are printed twice verbatim on `minsolveeqandmeth.htm`: the Limit-equation description
(lines 607–608 of the text extract) and the Iteration-Loops preamble (lines 1169–1170). Cosmetic
authoring artefact; no numeric consequence. Noted so a downstream diff does not read it as new
content.

### 5.9 Vendor typo in a limit equation raster

`[img-read: embim243.png]` reads `0.3 > Vwater + Vhyrocarbon` — "Vhyrocarbon" missing the `d`.
Confirmed at 4×. Cosmetic.

---

## 6. IP2018 → IP2025 numeric diff

Method: every numeric relation transcribed here was checked against the IP 2018 decompile at
`C:\Users\ARUNIKA\AppData\Local\Temp\c18\` — ASCII by direct string match, rasters by reading the
2018 GIF at the matching position in `minsolveeqandmeth.htm`.

### 6.1 Verified IDENTICAL (no numeric drift)

| Item | 2018 evidence | 2025 evidence |
|---|---|---|
| `U_hyd` gas branch `0.119 × ρhden` | `[img-read: c18/embim232.gif]` | `[img-read: embim230.png]` |
| `U_hyd` oil branch `0.133 × ρhden` | `[img-read: c18/embim234.gif]` | `[img-read: embim231.png]` |
| `Uwat = 0.00481 × Sal + 0.3883` | ASCII string match | ASCII |
| `U = Pef × (RHOB + 0.1883) × 0.93423` | ASCII string match, `mineralsolver.htm` | ASCII + raster |
| Salinity from Rmf75 (E4) | `[img-read: c18/embim216.gif]` — renders `Alog` closed-up | `[img-read: embim219.png]` — renders `A log` |
| `B(T, Rw)` formula (E72) incl. the unbalanced paren | `[img-read: c18/embim282.gif]` | `[img-read: embim273.png]` |
| Neutron porosity equation (E9) | `[img-read: c18/embim219.gif]` | `[img-read: embim222.png]` |
| Shell `m` **raster** `0.018` | `[img-read: c18/embim284.gif]` | `[img-read: embim275.png]` |
| Shell `m` **ASCII** `0.019` | ASCII string match | ASCII |
| Wyllie↔Hunt-Raymer bridge `Cp = 0.65156 + 0.8109·Phi + 0.01322·Dtma − 0.003261·Dtfl` | ASCII string match | ASCII |
| Neutron `.neu` look-up table (all 15 rows, 17 columns) incl. the `-.1960` outlier | ASCII | ASCII |
| PHIFLAG table (11 listed values, same wording) | 2018 report §1.8 | `minsolveeqandmeth.htm` |
| Convergence: φe < 0.001, Sxo < 0.002; caps 20/30/10; unity tolerance 0.01 | 2018 report §1.7 | `minsolveeqandmeth.htm` |
| Sw/Sxo auto-equation confidence 0.01; `Sonic Cp` 1.0; `Cm*` 1.0; `B fact Juhasz` 1.0 meq/ml; `Sxo Limit` 0.2; invasion factor 0.5 / 2.0 | 2018 report §1.7 | `mineral_solver.htm` |
| Solvers: SVD (Numerical Recipes) + DNOPT (Stanford); "best (lowest total error) always selected" | ASCII match on `DNOPT` | ASCII |
| Resistivity ± label inversion in the confidence worked example | 2018 report §1.5 | `minsolveeqandmeth.htm` |
| Max 20 models, 5 mixings, 8 crossplot end-points | 2018 report §1.7 | as cited |
| All 12 Sw equations, ASCII forms | 2018 report §3.3 | `mineral_solver.htm` |

**Conclusion: no petrophysical constant changed between IP 2018 and IP 2025 in the Mineral Solver.**
Every coefficient checked is bit-identical. This is a strong result for SandiMin — the IP response
model is stable across a seven-year release gap.

### 6.2 NEW in 2025 — content absent from the IP 2018 mineral-solver pages

Verified by grepping the whole IP 2018 decompile, not just the six counterpart pages.

| New item | Nature | Evidence |
|---|---|---|
| **`SonicMatrix` output equation** (E42) | new output equation type | strings `SonicMatrix` / `Sonic Matrix` absent from **every** `.htm` in c18 |
| **`Output Dry Wt%` output equation** (E53) | new output equation type, with its own conversion equation | string `Dry Wt` absent from every `.htm` in c18 |
| **`Den / Neu / Son Hydrocarbon Corrected` output equations** (E54) | three new output equation types | `Hyd Corr` absent; `Hydrocarbon Corrected` appears in c18 **only** on `sand_silt_malay_model.htm`, a different module |
| **`Model_Use` flag curve** + **"Discriminate Model to Interval Used"** plot option | new QC/plotting capability — discriminates a model's logplot to the intervals the mixings actually use it | `Model_Use`, `Discriminate Model` absent from all c18 `.htm` |
| **Explicit `Arps equation` naming + "Salinity Curves" / "Rw Curves" depth-by-depth handling** | documentation of Rw/salinity-as-curve behaviour, including the zonal-average rule for salinity curves | `Salinity Curves`, `Arps equation` absent from c18 |
| **Published mineral end-point grids** (§3.3) | ~14 worked model grids as screenshots, carrying real density/neutron/sonic/U/GR/Res end-points for Quartz, Calcite, Dolomite, Siderite, Pyrite, Kerogen, Kaolinite, Illite, Chlorite and generic Clay | 2018 report §2.1: "The IP 2018 help does not publish a mineral end-point table anywhere" |
| **MINDEF.PAR density values surfaced** (§3.3.8) | eight mineral grain densities visible in the Preprocessor screenshot | not present in 2018 |
| Image count on `minsolveeqandmeth` | 99 → **122** (+23) | 2018 report §0 vs 2025 census |

### 6.3 Resolved 2018 open items

| 2018 open item | Resolution here |
|---|---|
| "Density HC Conventional/Modified ASCII almost certainly corrupted — `* 2` is likely an exponent. **Do NOT implement**" | **Resolved.** The rasters show `2` is a plain multiplier. E6/E7 are safe to implement. |
| "Total Model Error formula — rasterized, not recoverable" | **Resolved.** E1, verified at 4×, including the absence of any `1/N`. |
| "CEC→Qv end-point conversion rasterized; the manual never states the CEC unit" | **Half-resolved.** The algebra is now known (E44–E46). The **unit is still not stated** — remains open (§8). |
| "BoundWater end-point = clay φT/(1−clay φT)" (text only) | **Confirmed by raster** (E34) and by two independent worked screenshots. |
| "Neutron excavation chain rasterized" | **Resolved.** E10–E14. |
| "EPT chain rasterized" | **Resolved.** E20–E22. |
| "U gas/oil branches rasterized" | **Resolved.** E24/E25, and confirmed identical in 2018. |
| "Secondary-porosity Wyllie/Raymer forms rasterized" | **Resolved.** E83/E84. |
| "Final calcs BVW … QvApp rasterized" | **Resolved.** E86–E94. |
| "IP 2018 `MINDEF.PAR` roster/values unknown from help text" | Still true for the file itself. But eight IP 2025 mineral densities are now visible in the help (§3.3.8), narrowing the diff surface. |

---

## 7. SandiBumi / SandiMin notes

1. **U-not-Pe is confirmed and now fully specified.** SandiMin should mix `U` volumetrically and
   compute it once, up front, from `U = Pef × (ρb + 0.1883) × 0.93423`. The fluid branches
   (`Uwat = 0.00481·Sal + 0.3883`; gas `0.119·ρh` below ρh 0.4; oil `0.133·ρh`) are now available and
   verified stable across two IP releases. IP has **no** Pe equation type at all — matching Pe
   directly would be a departure from the reference implementation, not a parity feature.

2. **The Total Model Error is not an RMS.** E1 has no `1/N`. A six-curve model in which every curve
   sits exactly at its tolerance scores 2.45, not 1.0 — yet the plot colours red above 1.0. If
   SandiMin wants a comparable "TotErr" it must replicate this exactly, or its numbers will not line
   up with an IP run on the same well. Decide deliberately and document the choice.

3. **Non-negativity is an active-set hack, not a constraint.** Step 3 zeroes the *largest* negative
   volume, drops that mineral, and re-solves — repeatedly. Unity is soft (tolerance 0.01) with a
   post-hoc renormalisation. If SandiMin uses a genuine bounded/NNLS solver it will *not* reproduce
   IP volume-for-volume even with identical end-points. This is the single most likely source of
   "SandiMin disagrees with IP" reports, and it is a design choice worth stating in the docs rather
   than chasing as a bug.

4. **Clay end-points are model-convention-dependent — this is the trap.** The same "Clay" column
   takes density 2.4 in a wet-clay model and 2.78 in a dry-clay model in the *same* worked example.
   Any SandiMin end-point library must carry a wet/dry flag on every clay row; a bare "clay density"
   field is a silent-wrongness generator. Same for the ECS/Wt% end-point (0.85 wet, 1.0 dry) and the
   bound-water coefficient (φ vs φ/(1−φ)).

5. **The smectite gap is real and now documented.** IP's Mineral Solver help names only Kaolinite,
   Illite and Chlorite. The standing SandiMin smectite review gains nothing from this manual — it
   must be sourced from `MINDEF.PAR` on disk, from the chartbook, or from one of Jauhar's studies.
   For Mahakam delta work, where smectite matters, IP is simply not a source.

6. **Shell `m`: do not implement from this manual.** §5.1. If SandiMin offers a Shell-m option it
   should cite the published source and state which constant it uses, with a note that IP's own help
   disagrees with itself.

7. **The Juhász/W&S raster form is the one to implement** (§5.2). Anyone reading only the ASCII —
   including anyone reading the 2018 report — would build a dimensionally-wrong equation.

8. **Design patterns worth adopting wholesale** (Tier A, conventions not expression): plain-ASCII
   user-editable defaults files (`MINDEF.PAR`, `MINEQDEF.PAR`, per-tool `.neu`,
   `ElanToIPMapping.par`); models saved as individual reusable files; `Print Parameters to File`
   producing a complete `.txt` audit trail of every model, parameter and mixing; the
   `***_re` / `***_me` / `***_pe` reconstruction+uncertainty output triple per input curve; the
   `Model_Use` discriminator flag; and the "output curves computed after each solver loop, before Sw"
   ordering that lets `Qv` / `PhiTClay` / `ResClay` feed the saturation step in the same run.

9. **The curve-set resolution trap is worth copying *and* fixing.** IP requires you to reference a
   model output curve with a **non-existent** set name so it resolves per-model; if a real set or a
   default-set curve of the same name exists, that one silently wins for *all* models. SandiMin
   should scope model outputs properly rather than reproduce this.

10. **Secondary porosity is free.** Add a sonic row to the grid with `Use` off and IP still emits
    `PhiSec` / `PhiSecU`. The deliberate allowance of negative `PhiSecU` as a calibration target
    ("average secondary porosity in non-vuggy rock should be zero") is a good, cheap QC idea.

11. **Cased-hole sigma is undocumented.** The workflow is described but no sigma equation type,
    end-point convention or unit appears anywhere in scope C. Do not assume parity is achievable from
    the manual alone.

---

## 8. OPEN ITEMS

Ambiguities, refusals to guess, and unresolved contradictions. Nothing below was filled in.

1. **Shell formula constant — 0.018 (raster) vs 0.019 (ASCII), in both IP 2018 and IP 2025.**
   §5.1. Not resolved. Both transcribed. Requires a decision by Jauhar against the published Shell
   source, not by this agent.

2. **CEC units are never stated.** E44–E46 give `Qv = ρdcl · Vdcl · CECdcl / φT` and
   `EPP = ρdcl · CECdcl · (1 − φTclay)`, but no page states whether `CECdcl` is meq/100 g or meq/mL,
   nor the units of `ρdcl` in that product. The observed Qv end-points (Illite 0.59, Chlorite 0.38)
   are composites and cannot be decomposed without an independent ρdcl. **This is exactly the
   meq/mL-vs-meq/L trap already recorded in `reference_waxman_smits_b`** — do not assume.
   Note: `B fact Juhasz` *is* given a unit (meq/ml), but that is `Bn`, not CEC.

3. **`Qv` output end-point units not stated.** The 0.59 / 0.38 values in `[img-read: _imsclip0068.png]`
   carry no unit anywhere on the page.

4. **MINDEF.PAR entries `XPL` (2.59) and `XAN` (2.74) have no mineral label** in the screenshot
   `[img-read: _imsclip0003.png]`, unlike the six neighbours. Not guessed. Resolvable only by opening
   `MINDEF.PAR` on disk.

5. **The full mineral drop-down roster is not printed.** `_imsclip0020.png` shows the Models tab with
   annotation callouts, not an expanded mineral list; `_imsclip0098.png` shows the **Type** drop-down
   (8 entries, captured in §3.6), not the mineral list. The mineral roster lives in `MINDEF.PAR`.

6. **Bound-water coefficient inconsistency in `_imsclip0092.png`** — `0.15` clay with `−1` bound
   water, which fits neither E33 nor E34 (§5.3). Could be a loose example or a typo. Not resolved.

7. **`E'` in the EPT chain (E20) — the trailing `²`.** `[img-read: embim227.png]` renders a
   superscript 2 immediately after the closing bracket of the second factor. The reading
   `E' = (79.4 − 202.69·Sal) × (1 − 0.385·(T−75)·(3230.0−T)·10⁻⁶)²` is the only parse consistent with
   the bracket nesting shown, and the `²` is clearly present and clearly outside the bracket — but
   the glyph sits very close to the bracket at native resolution. **Confidence: high, not absolute.**
   Verify against a second source before implementing the EPT branch.

8. **`Uwat` slope units.** E23 uses `Sal`, but the page does not state whether `Sal` here is in ppm,
   kppm, or the `ppm × 10⁻⁶` convention used in the EPT section (E20's `Sal`). The two sections use
   the same symbol for possibly different scalings. **Not resolved — do not implement E23 without
   settling the unit.**

9. **`NeuSal` and `NeuMatrix` in E9 are named but not given closed forms.** They come from the `.neu`
   look-up tables; no interpolation scheme between the tabulated porosity rows is stated.

10. **Sigma / cased-hole:** no equation type, end-point convention or unit anywhere in scope C (§4.9).

11. **`Neu Log Cont` / `Neu Tool Type` rosters not enumerated.** Only `Schlumb` / `CNL` appear, in a
    screenshot. The full contractor and tool-type lists are not printed.

12. **The `-.1960` neutron look-up outlier** is transcribed unchanged and remains unverified against
    a shipped `Sch_CNL.neu`. Do not silently repair.

13. **Poupon-Aguilera / Poupon-Tixier rasters** `[img-read: _imsclip0091.png, _imsclip0110.png]` were
    read at native resolution and cross-check exactly against the ASCII on `mineral_solver.htm`. No
    ambiguity — recorded here only to note that the agreement is the basis for confidence, since
    these two are the smallest rasters in the set.

14. **`_imsclip0050.png` GrainDensity row, Water/Oil cells.** Read as `0.` at 1.9× upscale. At native
    resolution they could have been read as blank. `0.` is the reading at the higher resolution and
    is consistent with the parallel `_imsclip0092.png` and `_imsclip0093.png` grids, which both show
    explicit `0`. Recorded as `0.` with that caveat.

---

*End of scope C. No vendor file was copied, modified, or moved. `c25`, `c18` and the
`ip2018_chm_ingest` folder were read only. This report is the sole file written.*
