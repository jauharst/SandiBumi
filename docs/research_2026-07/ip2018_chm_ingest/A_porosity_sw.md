# IP 2018 CHM ingest — Target A: Porosity & Water Saturation

Source: decompiled Interactive Petrophysics 2018 help (vendor PGL / Lloyd's Register / Geoactive).
Extraction date 2026-08-06. Pages read:
`swparameters.htm`, `swequationsandmethodology.htm`, `porosityandwatersaturation.htm`,
`swplot.htm`, `rwfromsp.htm`, `temperaturegradient.htm`, `densityestimation2.htm`,
plus two in-scope adjacents found during the sweep: `basicloganalysis.htm` and
`sand_silt_malay_model.htm` (both are porosity+Sw modules and both carry stated
numeric defaults the named pages do not).

## Reading rules used

- **Every number below is quoted from the manual.** Nothing is rounded, unit-converted,
  tidied, or supplied from petrophysical knowledge. Where the manual is silent the entry
  says `not stated in manual`.
- `swequationsandmethodology.htm` has **121 rasterized equation images**
  (`[[EQUATION_IMAGE: embimNN.gif]]`). Those formulas are **not recoverable** and are not
  reconstructed here. They are marked `rasterized — not recoverable`.
- **A large and unexpected win:** many equations that are rasterized on the methodology
  page are also written **in plain ASCII on `swparameters.htm`** (all eleven Sw equations,
  the Shell m formula, both m\* formulas, the hydrocarbon-response formulas, the silt
  index, the shale-porosity-suppression formula). Where that is true it is called out,
  because it means the method variant IP actually uses is knowable without the images.
- Tier B items (published science) record the **method name + the citation the manual
  gives**. SandiBumi reimplements from the primary paper. The IP-documented form is
  recorded only as evidence of *which variant* IP chose (e.g. whether Simandoux carries
  `Sw^n` or `Sw`, whether `(1-Vcl)` appears) — that distinction is the whole value.

---

## 1. Module architecture (Tier A — free to adopt)

IP splits porosity/Sw across **three separate modules**, not one:

| Module | Page | Scope |
|---|---|---|
| **Porosity and Water Saturation** (PhiSw) | `porosityandwatersaturation.htm` | The full engine. 5 porosity methods + pass-through, 11 Sw equations, multi-mineral, laminated sand, organic shale, EPT. |
| **Basic Log Analysis** | `basicloganalysis.htm` | Deliberately reduced: "a type of analysis that could be easily duplicated using a calculator". 3 porosity methods, 3 Sw equations, **no hydrocarbon correction, no bad-hole correction, no Sxo**. This is where IP states its bare `a`/`m`/`n`/`Rw` defaults. |
| **Sand/Silt Malay Model** | `sand_silt_malay_model.htm` | Separate 3-endpoint (quartz/silt/dry-clay) N-D model for fine-grained fresh-to-brackish sediments. 4 total-Sw equations. |

Parameters are **per-zone, in a tabbed grid** (Clay, Waters, Hydrocarbon, Matrix, Phi Logic,
Sw Logic, Laminated Sand, Limits/Badhole, Organic Shale, Density/Neutron, EPT, Sonic,
Coal/Salt/Anhy/Kill, MultiMin 1/2/3). Design details worth stealing:

- **Column edit** — click a column header (turns green) and changing one cell sets every
  cell in that column. Persists while the well is in memory; resets on reload. (page: swparameters.htm)
- **Lock Zone** — a locked zone is grey everywhere and immune to parameter edits, to
  interactive-line drags, and to the Multi-Well Change Parameters module. (page: swparameters.htm)
- **Greyed cells mean "not used by this zone's model"** — e.g. Waxman-Smits parameters
  grey out in zones whose Sw equation is Dual Water. (page: swparameters.htm)
- **Edits do not auto-recalculate.** "You must click the Run button in order to re-calculate
  all zones and update all interactive displays." (page: swparameters.htm)
- **Null-all-before-run** checkbox sets all module outputs to null (-999) before recomputing,
  for when the depth interval or zonation shrank. (page: swparameters.htm)
- **Per-parameter context help** — a floating window that describes whichever parameter the
  cursor is over, deliberately *not* linked to the main help document. (page: swparameters.htm)
- Parameters carry a **Monte Carlo index** in parentheses, e.g. `(64) a factor`,
  `(65) m exponent`, `(66) n exponent`, `(1) Rw`, `(132) B fact W-S`, `(134) Qv a Const`.
  These map into `MonteCarloDefaults.par`. (page: swparameters.htm)

---

## 2. Porosity methods

`Porosity Method` options: **Neutron/Density, Neutron/Sonic, Neutron, Density, Sonic**
(page: swparameters.htm), plus **Input Crv** (pass-through) when a curve is supplied in
the Pass through Porosity box (page: porosityandwatersaturation.htm).

### 2.1 Density porosity

- Equation rasterized (`embim44`) — not recoverable. Inputs named: matrix density
  (curve, parameter, or from mineral volumes), bulk density log, wet clay density,
  filtrate density, apparent hydrocarbon density, Vcl, Sxo. (page: swequationsandmethodology.htm)
- `Rho GD` — matrix grain density, gm/cc (kg/m3). Used when `GD source = param`.
  (page: swparameters.htm)
- **Stated verbatim:** "If a Rho GD is not entered then it is assumed to be 2.71."
  (page: swequationsandmethodology.htm — in the *Neutron* branch of the 2-mineral
  lithology section; this is the "Rho GD 2.71 assumed" statement)
- **Stated verbatim (2-mineral lithology from Density method):** "A fixed lithology for an
  input Rho GD is assumed. 2.65 100% sand, 2.71 100% lime, 2.85 100% Dolomite. Input Rho GD
  between these end points will result in a linear extrapolation between the end points.
  For example an input Rho GD of 2.68 will result in 50% Sandstone and 50% Limestone."
  (page: swequationsandmethodology.htm)
- `Rho Wet Clay` — "A value must be entered if a density tool is selected on the set-up
  window." No default. (page: swparameters.htm)
- `Rho Dry Clay` — **"( IP defaults to 2.78 gm/cc.)"** (page: swparameters.htm)
- Basic Log Analysis equivalents (**this module states its defaults explicitly**):
  - `Rho Matrix` — "Defaults to 2.65 gm/cc for sandstone. Set to 2.71 gm/cc for limestone."
  - `Rho Fluid` — "Defaults to 1.0 gm/cc for fresh water. Set to 1.1 gm/cc for salt water."
  - `Rho Clay` — "The default value is chosen as the maximum density reading over the
    interval where the sonic is high." (data-derived, no fixed number)
  (page: basicloganalysis.htm)

### 2.2 Neutron porosity

- Equations rasterized (`embim50`, `embim51`). Named terms: input neutron log, Vcl,
  NeuCl, NeuMatrix, **Exfact (neutron excavation factor)**, NeuSal, Sxo, NeuHyHI.
  (page: swequationsandmethodology.htm)
- **Input convention:** "IP assumes that any neutron curve entered is in Limestone matrix
  units." (page: porosityandwatersaturation.htm)
- `Neu Matrix` — neutron matrix in decimal porosity units, **linear** response. "Any value
  for the Rho GD parameter will totally override Neu Matrix, therefore if you want to use
  Neu Matrix, blank out the Rho GD parameter." (page: swparameters.htm)
- `Rho GD` in the Neutron model selects the **non-linear** transform:
  "if you wanted to calculate a neutron sandstone, enter 2.65 (2650), and the non-linear
  neutron sandstone transform will be used." (page: swparameters.htm)
- **Recipe to defeat all corrections** (verbatim set): "Neu Matrix = 0.0 /
  Matrix Density = Blank / NeuHyHI = 1.0". Also: "If the Neutron hydrocarbon apparent
  hydrogen index (NeuHyHI) is entered as a parameter then the Neutron excavation factor
  (Exfact) will be set at 0.0." (page: swequationsandmethodology.htm)
- `Neu Wet Clay` — "A value must be entered if a neutron tool is selected on the set-up
  window." No default. (page: swparameters.htm)

#### Neutron tool look-up tables — Tier A file format, high value

Per-tool ASCII response tables, selected by `Neu Log Cont` (contractor) + `Neu Tool Type`.
Registered in **`Neu_Parm_Files.neu`** in the IP directory; each tool has its own `.neu`
table (example shown in the manual: `Sch_CNL.neu`, "Schlumberger CNL TNPH").
(page: swequationsandmethodology.htm)

Column layout, verbatim from the file's own header comment:
"True Phi (limestone matrix), Sandstone Matrix correction, Dolomite Matrix correction,
Salinity correction Sand, Salinity correction Lime, Salinity correction Dol" —
"Formation Salinity corrections are for following values 50, 100, 150, 200, 250 Kppm and
in this order" — "Porosity values must not be changed".

Porosity node list (verbatim): "0%, 2%, 5%, 10%, 15%, 20%, 25%, 30%, 35%, 40%, 45%, 50%,
55% and 60% p.u." — "The values of porosity extend to 60 pu., and it is necessary to
complete the table up to this value even though it is unlikely that there are any
published results for these high porosities. It is necessary to extrapolate the data as
accurately as possible." "NOTE: all values in the table should be in decimal notation."

The manual gives the **full construction recipe** (Tier A, free): take the vendor's
Neutron Porosity Equivalence Curves chart, tabulate the correction between the limestone
porosity and the true sandstone/dolomite porosity at each node; then take the vendor's
environmental-correction nomograph, Formation Salinity panel, and read the correction at
50/100/150/200/250 kppm for the equivalent limestone porosity.

**Caution for SandiBumi:** the *values* in IP's shipped `.neu` files are digitizations of
third-party service-company chartbooks. Adopt the file format and the recipe; digitize the
numbers from the vendor chart directly (Jauhar's chartdig tooling already does this) rather
than copying IP's tables.

Salinity correction is applied only if `Neu Form Sal` is On; the correction is interpolated
in the table on **formation porosity, flushed-zone salinity, and matrix density**, with the
matrix mix taken from `Rho GD` + `Mineral Model`. Verbatim example: "if the mineral model
chosen was Sand/Dolomite and the input matrix density was 2.75 g/cc, then the correction
would be half way in-between the correction for Sand and Dolomite."
(page: swequationsandmethodology.htm)

### 2.3 Sonic porosity

`Sonic Equ` — **Wyllie time average** or **Raymer Hunt**. (page: swparameters.htm)
Both equations rasterized (`embim53` Wyllie, `embim54` Raymer). For Raymer the manual
states the substitutions in text: `Vma = 1/Dtma`, `Vf = 1/Dtfl`, `Vclay = 1/Dtclay`,
`Vlog = 1/Dt`. (page: swequationsandmethodology.htm)

PhiSw module defaults (all verbatim, page: swparameters.htm):

| Parameter | Default | Units |
|---|---|---|
| `Sonic Lime` | "Default 49 (160 μ Sec/m)" | μSec/ft (μSec/m) |
| `Sonic Sand` | "Default 56 (180 μ Sec/m)" | μSec/ft (μSec/m) |
| `Sonic Dol` | "Default 44 (145 μ Sec/m)" | μSec/ft (μSec/m) |
| `Sonic water` | "Default 189 (620 μ Sec/m)" | μSec/ft (μSec/m) |
| `Sonic Cp` | "(Default 1.0)" | — |
| `Sonic matrix` | not stated in manual | μSec/ft |
| `Sonic Hc` | "Must be entered if Sonic porosity model is chosen." No default. | μSec/ft (μSec/m) |
| `Sonic Wet Clay` | "A value must be entered if a sonic tool is selected". No default. | μSec/ft (μSec/m) |

Basic Log Analysis states a **different sonic sand matrix** (page: basicloganalysis.htm):

| Parameter | Verbatim |
|---|---|
| `DT Matrix` | "Defaults to 55 uSec/ft for sandstone. Set to 49 uSec/ft for limestone." |
| `DT Fluid` | "Default value is189 uSec/ft. For salt-saturated formation water use about 174 usec/ft." |
| `Sonic Cp` | "Defaults to 1.0. For unconsolidated sands this needs to be increased. A rule of thumb for estimating this parameter is to divide the sonic clay parameter by 100. The result should be greater than 1.0 for a valid Cp value." |

> **Note the internal inconsistency: PhiSw `Sonic Sand` = 56, Basic Log Analysis
> `DT Matrix` sandstone = 55.** Both are quoted verbatim above. Do not reconcile them —
> IP ships both.

**IP's own recommendation, verbatim:** "It is recommended that the Raymer equation is used
by default since this equation automatically takes care of the problem of unconsolidated
sands, whilst the Wyllie equation has an extra compaction factor parameter which has to be
estimated." (page: basicloganalysis.htm)

### 2.4 Neutron/Density porosity — the four "variable" solvers

The N/D model is a constrained solver with **four flags that operate in series and can all
be active at one depth**: Variable Sxo, Variable Hc Den, Variable GD, Variable Vcl.
(page: swequationsandmethodology.htm)

Resolution order, verbatim (page: swparameters.htm):
- `Variable GD`: "If the Variable Hc Den flag is also set, then the hydrocarbon density
  will be calculated firstly using the input matrix density. Only if the Hc Den is outside
  its limits will the matrix density be varied."
- `Variable Vcl`: "Result must be between 0 and 1. If the Variable Hc Den or Variable GD
  flag is also set, then these will be varied first. Only if both are at their limits will
  the input clay volume be varied. If Hc Den, Matrix Den and Clay Vol are all at their
  limits and no solution is possible, then the neutron and/or density input curve will be
  reduced in order to resolve the solution... the PHIFLAG curve will be set to 6 to
  indicate a reduction in density, and 7 to indicate a reduction in neutron."
- `Variable Sxo`: "The result is limited to be greater than Sw (WBM) or less than Sw (OBM).
  It is also limited to be less than the Sxo limit (WBM)." Plus a usage trap, verbatim:
  "when using Variable Sxo logic the Sxo limits can still be activated. Therefore, it is
  required to tick another of the 'Variable' calculations along with 'Variable Sxo' to use
  in the case of Sxo being clipped."

Crossplot porosity from the two-mineral pair is rasterized (`embim59`), with the four
inputs named as neutron-corrected porosity for matrix 1/2 and density-corrected porosity
for matrix 1/2. (page: swequationsandmethodology.htm)

`Mineral Model` options and the **switch rule, verbatim** (page: swparameters.htm):
"Sandstone/Limestone/Dolomite (ss/ls/dol). If matrix density is greater than 2.71g/cc
(2710 kg/m3), then the model will be Limestone/Dolomite; less than 2.71g/.cc (2710 Kg/m3),
Sandstone/Limestone." Other options: `ss/dol`, `ss/sp min`, `ls/sp min`, `dol/sp min`
(special-mineral endpoints must be entered).

The Neutron/Sonic branch uses the same rule on transit time instead:
"(Dtma > Dtls ss/ls, Dtma < Dtls ls/dol)". (page: swequationsandmethodology.htm)

Limits: `Rho GD max` / `Rho GD min` bound Variable GD; `Hc Den` (max) / `Hc Den Min`
bound Variable Hc Den. No numeric defaults stated for any of the four.
(page: swparameters.htm)

### 2.5 Neutron/Sonic porosity

"The model uses the same logic as the Variable matrix density logic in the Neutron/Density
model, except that the sonic log is substituted for the Density log, and Dt matrix is
calculated from the porosity." Sonic-model choice (Wyllie/Raymer) still applies.
(page: swequationsandmethodology.htm)

### 2.6 Pass-through porosity

Set `Porosity Method = Input Crv` and `Input Porosity = Total | Effective`.
"IP will calculate Total Porosity if Effective Porosity is entered and Effective Porosity
if Total Porosity is entered." The conversion uses PhiT-clay, "entered as an input
parameter or is calculated from the dry clay density". Conversion equations rasterized
(`embim64`–`embim69`). (pages: swparameters.htm, swequationsandmethodology.htm)

2-mineral lithology under pass-through: "The calculations assume the lithology can be taken
from the input Rho GD parameter." (page: swequationsandmethodology.htm)

### 2.7 Hydrocarbon response — formulas recoverable in ASCII

These are given in text on the parameters page (rasterized on the methodology page).
No citation is given for either density model. (page: swparameters.htm)

- `Neu Hc HI` default calculation, verbatim:
  `Neu Hc HI = RhoH * 9 (4 - 2.5 RhoH) / (16 - 2.5 RhoH)` where RhoH is `Hc Den` in gm/cc.
- `Den Hc app`, **Conventional** model, verbatim:
  `DenHcApp = RhoH * 2 (10 - 2.5 RhoH) / (16 - 2.5 RhoH)`
- `Den Hc app`, **Modified** model, verbatim:
  `DenHcApp = (5.5 * RhoH (4-RhoH) -3) / (16 - 2.5 RhoH)`
- `Den Hyd model` selects between them. Verbatim rationale: "'Conventional' option uses the
  technique that has historically been used in IP. 'Modified' option uses a method that
  better takes into account the correction from electron density to apparent density."

Related defaults (page: swparameters.htm):

| Parameter | Default (verbatim) |
|---|---|
| `Hc Den` | "If Hc Den is left blank, the value defaults to 1.0 gm/cc (1000 kg/m3)." |
| `Hc Den Min` | not stated in manual |
| `PhiShr limit` | "Volume of hydrocarbon limit seen in the flushed zone in decimals ( Phi*(1-Sxo ) default 0.02" |

Display-only cutoff: hydrocarbon shading is green above and red below **0.4 gm/cc**
output hydrocarbon density; "This cut-off value can be changed in the Plot Format."
(page: porosityandwatersaturation.htm)

### 2.8 Organic shale porosity

Only `Density Neutron`, `Density`, `Neutron` are allowed; other porosity models raise an
error. With N/D, "only the variable Matrix Density logic is usable".
(pages: porosityandwatersaturation.htm, swequationsandmethodology.htm)

| Parameter | Default (verbatim) | Units |
|---|---|---|
| `Rho Kerogen` | "Default is 1.1 gm/cc." | gm/cc |
| `Neu Kerogen` | "Default is 0.6 v/v." | v/v |
| `Rho Heavy Min.` | "Default is 4.3 gm/cc." | gm/cc |
| `Neu Heavy Min.` | "Default is -0.03 v/v." | v/v |
| `Kerogen Wt% Con.` | "If left blank then the conversion factor will be calculated by the program." Plus verbatim: "This conversion factor can be very significant with some interpreters recommending values as high as 2.5." | — |
| `Heavy Min Wt% Con.` | "If left blank then the conversion factor will be calculated by the program." | — |

Verbatim caveat: "The volumes of TOC and Heavy Mineral have a very significant effect of
the calculation of porosity due to their extreme densities compared to quartz and clay...
The conversion factor for TOC can more than double the TOC input while the factor for heavy
minerals can half the volume of heavy mineral in the model." (page: swparameters.htm)

Unit auto-detection rule, verbatim: "It the units found are '%', 'pec' or 'wt%' then the
input curve will be divided by 100 before use as a decimal volume. Any other units found
will be assumed to be decimal volume." (page: swequationsandmethodology.htm)

Output note: "The output Matrix density curve is the density of the rock matrix minus Clay,
Kerogen and Heavy minerals." (page: swequationsandmethodology.htm)

### 2.9 Total porosity, PhiT-clay, bound water

- `PhiT Clay` — "If a cell is left blank, PhiT Clay will be calculated from the Rho Dry Clay
  and Rho Wet Clay parameter entries." (page: swparameters.htm)
- The PhiT-clay, PhiT and Swb equations are rasterized (`embim74`, `embim79`, `embim80`).
  (page: swequationsandmethodology.htm)
- Interactive-picking coupling worth copying, verbatim: "If 'PhiT clay' is actually
  calculated from 'Rho Dry Clay' and 'Rho Wet Clay' ('PhiT clay' parameter is blank) then
  moving of this parameter will recalculate the 'Rho Dry Clay' parameter."
  (page: swplot.htm)

---

## 3. Water saturation models

`Saturation equation` — **eleven** options, used for both Sw and Sxo. All eleven are
rasterized on the methodology page (`embim104`–`embim117`) **but written in ASCII on the
parameters page** (page: swparameters.htm). Tier B: reimplement each from the primary
paper; the forms below are recorded to pin down *which variant* IP uses.

| # | Name | IP-documented form (swparameters.htm, verbatim) | Citation the manual gives | Rasterized on methodology page |
|---|---|---|---|---|
| 1 | Archie | `1/Rt = Phi**m.Sw**n / (a.Rw)` | none | yes (embim104–106) |
| 2 | Archie PhiT | `1/Rt = PhiT**m.Sw**n / (a.Rw)` — "Same as Archie except PhiT used instead of Phie." | none | yes (embim107–109) |
| 3 | Simandoux | `1/Rt = Phi**m.Sw**n / (a.Rw) + Vcl.Sw / Rcl` | none | yes (embim110) |
| 4 | Modified Simandoux | `1/Rt = Phi**m.Sw**n / (a.Rw.(1-Vcl)) + Vcl.Sw / Rcl` | none | yes (embim111) |
| 5 | Indonesian (Poupon-Leveaux) | `1/Rt**.5 = ((Phi**m/(a.Rw))**.5 + Vcl**(1-(Vcl/2))/Rcl**.5 ).Sw**(n/2)` | none | yes (embim112) |
| 6 | Woodhouse Tar | `1/Rt = Sw**n.((( Vcl**(1-Vcl))/( Rcl**0.5))+((Phi**(m/2))/((a.Rw)**0.5)))**2` | **"a SPWLA paper 'Athabasca Tar Sands Reservoir Properties Derived from Core and Logs' 1976 17th annual Logging symposium by R. Woodhouse."** Manual adds: "This is a modified version of the standard Indonesian (Poupon-Leveaux) equation." | yes (embim113) |
| 7 | Dual Water | `1/Rt = PhiT**m.SwT**n/a.(1/Rw + Swb/SwT(1/Rwb-1/Rw))` | none | yes (embim114–115) |
| 8 | Juhasz (Waxman-Smits) | `1/Rt = PhiT**m.SwT**n.(1+Bn.Qvn/SwT)/(a.Rw)`; `Qvn = Vcl*PhiClay / PhiT`; `Bn = B normalized, entered parameter` | none | yes (embim116) |
| 9 | Waxman-Smits | `1/Rt = PhiT**m.SwT**n.(1+B.Qv/SwT)/(a.Rw)`; `Qv = a / PhiT + b` | none | yes (embim117–120) |
| 10 | Poupon-Aguilera | `1/Rt = Phi**m.Sw**n / (a.Rw(1-Vcl)) + Vcl / Rcl` | **"'Extensions of Pickett Plots for the Analysis of Shaly Formations by Well Logs', Roberto Aguilera (The Log Analyist, Sept-Oct 1990), where the exponents of 'n' and 'm' have been added."** ("The Log Analyist" is the manual's spelling.) | no explicit image — prose only |
| 11 | Poupon-Tixier | `1/Rt = (1-Vcl).Phi**m.Sw**n/(a.Rw) + Vcl/Rcl` | **"'A contribution to electric log interpretation in shaly sands', Poupon A, Loy ME, Tixier MP (1954) Trans AIME 6(06):138?145 with the addition of 'm' and 'n' exponents."** | no explicit image — prose only |

Naming history worth recording: "In previous versions of IP, this [Poupon-Aguilera] was
simply called 'Poupon." (page: swequationsandmethodology.htm)

Vcl-vs-Vshale convention, verbatim for both Poupon variants: "In the original equation Vcl
is Vshale and Rcl is Rshale. In IP we calculate clay volumes and shale is considered a rock
type. However the equation can be used if the interpreter picks parameters to represent
shale rather than clay." (page: swequationsandmethodology.htm)

### 3.1 Archie parameters and their stated defaults

`swparameters.htm` describes `a factor`, `m exponent`, `n exponent` **without stating any
default value**. The only stated numeric defaults for these live in Basic Log Analysis:

| Parameter | Verbatim | Source |
|---|---|---|
| `Rw` | "Formation water resistivity at formation temperature. **Defaults to 0.1 ohmm** but must be adjusted to the correct value." | basicloganalysis.htm |
| `a factor` | "Archie equation a factor. **Defaults to 1.0.**" | basicloganalysis.htm |
| `m Exponent` | "Archie equation m (cementation) factor. **Defaults to 2.0.**" | basicloganalysis.htm |
| `n Exponent` | "Archie equation n (saturation exponent) factor. **Defaults to 2.0.**" | basicloganalysis.htm |
| `Sat Equation` | "Water Saturation equation. **Defaults to Archie.** In clean rock select Archie, in shaley sands the Simandoux or Indonesian equations will make corrections for shale in the rock and give better results." | basicloganalysis.htm |
| `a` / `m` / `n` in PhiSw module | **not stated in manual** | swparameters.htm |
| `a` / `m` / `n` in Sand/Silt Malay | **not stated in manual** | sand_silt_malay_model.htm |

### 3.2 `m source` — four ways to get the cementation exponent

Options, verbatim (page: swparameters.htm):
- **Parameter** — fixed value.
- **Curve** — from an input curve.
- **Shell** — "Calculate from the Shell formula **`m = 1.87 + 0.019 / PHIE`**"
  (**ASCII, recoverable**; rasterized as `embim121` on the methodology page. No citation
  given beyond the name "Shell".)
- **Rxo EPT** — "A variable m is calculated from Rxo, using the Sxo calculated from the EPT.
  The resultant m value is limited by the parameters `min m value` and `max m value`."
- **m \*** — variable m\*, W&S / Dual Water only (below).

`min m value` / `max m value` / `m plus value` — no numeric defaults stated.

**m boosted in shales**, verbatim: "If set to On (box is selected) and If Vcl > Vcl cut-off
then: **`m = m*10**(Vcl - Vcl cut-off)`**. This has the effect of removing any hydrocarbons
in zones of high clay content." (page: swparameters.htm; rasterized as `embim122`)

### 3.3 `n source`

Options: **Parameter**, **Curve**, **Rxo EPT** — the last sets `n = m + mPlus` using the
EPT-derived m. (page: swparameters.htm)

### 3.4 Variable m\* — coefficients recoverable in ASCII (high value)

Verbatim from `swparameters.htm` (`m *` entry):

```
m* = m + Cm(1.128 Y + 0.22 (1-e**(-17.3Y)))    W&S
m* = m + Cm(0.258 Y + 0.20 (1-e**(-16.4Y)))    Dual Water
Y = Qv PhiT / (1 - PhiT)
Cm* is an input parameter
Qv = a/PhiT + b where a and b are parameters that can be modified.
Qv can also be entered as an input curve.
```

Every coefficient here (1.128, 0.22, 17.3, 0.258, 0.20, 16.4) is stated by the manual and
is quoted exactly. **No citation is given for either form.** Both are rasterized on the
methodology page (`embim115` Dual Water, `embim120` W&S) but the ASCII above is the same
content.

`Cm*` default, verbatim two ways:
- "(140) Cm\* - Cm constant in the W&S and Dual Water, variable m\* equation. **Default value
  is 1.0.** If set to 0.0, m\* will be equal to equation input m. The variable m\* option is
  turned on by setting the m source parameter to m\*." (page: swparameters.htm)
- "The Cm\* parameter is used to adjust the weighting on the variable m\* and is entered as a
  parameter **(Default is 1.0)**." and, in the Waxman-Smits section, "The Cm parameter is
  used to adjust the weighting on the variable m\* and is entered as a parameter
  **(Default is 1.0)**." (page: swequationsandmethodology.htm)
- Fallback: "If the m source parameter is not set to m\*, the m\* used in the equation will be
  the input m parameter." (page: swequationsandmethodology.htm)

### 3.5 Qv and B

| Parameter | Verbatim | Source |
|---|---|---|
| `Qv W-S Source` | "Set to Param for calculation of Qv from the equation: `Qv = a / PhiT + b`... If set to Curve then Qv is taken from the input curve." | swparameters.htm |
| `Qv a Const` (134) | "The a Constant in the Qv equation: `Qv = a / PhiT + b`. Used in the Waxman-Smits Sw equation. a and b can be selected interactively from the 1/PhiT versus QvApp crossplot." **No numeric default stated.** | swparameters.htm |
| `Qv b Const` (135) | "The b Constant in the Qv equation: `Qv = a / PhiT + b`." **No numeric default stated.** | swparameters.htm |
| `B fact Juhasz` (94) | "B factor (equivalent conductance of clay cations) in the Juhasz Waxman-Smits equation. **Default 1.0 meq/ml**. Can be set interactively using the Cwa versus Qvn crossplot." | swparameters.htm |
| `B fact W-S` (132) | "B factor (equivalent conductance of clay cations) in the Waxman-Smits equation. **If left blank then this is calculated from formation temperature and Rw.**" No numeric default. | swparameters.htm |
| B-from-T,Rw formula | **rasterized (`embim119`) — not recoverable.** Manual states only that B "is calculated from the following equation" and that "T = Formation temperature in degrees centigrade". | swequationsandmethodology.htm |

> **Unit flag on `B fact Juhasz`.** The manual writes the default as "1.0 meq/ml", i.e. it
> labels **B** with **Qv's** unit. Record verbatim as above; do **not** normalize. This
> matches the meq/mL-vs-meq/L trap already in Jauhar's Waxman-Smits note — IP's help text
> is itself loose here, so an implementer copying the label would inherit the confusion.

Bn calibration guidance, verbatim: "The Bn factor is adjusted on the crossplot so that the
100% wet line goes through the wet shaley points. It should be noted that there is a strong
correlation between the bound water volume (PhiT minus Phie) and the Bn factor. If you
change the bound water by changing the dry clay density (or total porosity clay) then the
Qvn/Cwapp relationship will change and the Bn should be adjusted."
(page: swequationsandmethodology.htm)

### 3.6 Effective ↔ total Sw conversion

"If the Archie PhiT, Dual Water, Juhasz or Waxman Smits saturation equation has been used,
then the effective water saturation is calculated as follows:" — **rasterized (`embim123`)
— not recoverable.** (page: swequationsandmethodology.htm)

### 3.7 Sw comparison curves — a caveat worth copying verbatim

IP outputs parallel Sw/SwT curves for every equation, but warns:

"These comparison methods are designed to compare water saturation methods **they should
not be used for anything other than this**. The final Sw used for summations should be the
Sw and SwT curves using the method set with the Sat Equation parameter."

"There are two Archie equations; the Archie and the Archie Total. Archie outputs an
effective water saturation (SwArch). While Archie Total outputs a total water saturation
(SwTArchT). **One cannot compare Sw and SwT directly as they are different things.** Hence,
IP calculates a SwT from the Archie equation (SwTArch) and a Sw from the Archie Total
(SwArchT) equation." — "These curves are meant for rough comparison purposes only as they
have not been hydrocarbon corrected in the same way as water saturations calculated via
the Sat Equation parameter."

The same warning is repeated for the porosity comparison curves. (page: porosityandwatersaturation.htm)

---

## 4. Sxo (flushed zone)

`Sxo Method` options: **Rxo**, **TPL** (from EPT), **Invasion Factor**; plus the N/D
`Variable Sxo` route. (pages: swparameters.htm, swequationsandmethodology.htm)

**Rxo route** — same Sw equation, with the substitutions stated verbatim:
`Rmf for Rw`, `Rxo for Rt`, `Rmfb for Rwb`, `RxoCl for Rcl`. (page: swequationsandmethodology.htm)

**Invasion-factor route** — ASCII on the parameters page (rasterized `embim124` on the
methodology page):
- "For water based mud `Sxo = (Sw +InvasionFactor) / (1+ InvasionFactor)`."
- "For Oil based mud `Sxo = InvasionFactor`."
(page: swparameters.htm) — `Invasion factor` has **no stated numeric default**, but the OBM
example uses 0.5: "if the Invasion factor is set at 0.5 then Sxo will be the minimum of 0.5
or Sw."

**Mud-type limits** (both rasterized, `embim128` OBM / `embim129` WBM) but the intent is
stated in prose: OBM — "Sxo can not be greater than the water saturation in the undisturbed
zone (Sw)"; WBM — "Sxo will be greater than or equal to Sw". Consequences stated verbatim:
"Sxo is usually the same as Sw in hydrocarbon zones in OBM environments" and "Sxo is usually
the same as Sw in water bearing zones in WBM environments."
(page: swequationsandmethodology.htm)

**`Sxo Limit ?` flag** — "the calculation of Sxo is limited by the following equation -
`Sxo < Sw**SxoLimit`." `Sxo Limit` is the exponent; **no numeric default stated**.
Restriction, verbatim: "It is only available in water-based mud where the Sxo Method
parameter (Sw Logic tab) is set to Rxo or EPT TPL." Purpose, verbatim: "for the situation
where a micro resistivity (Rxo) tool loses pad contact and calculates too high an Sxo."
(pages: swparameters.htm, swequationsandmethodology.htm)

---

## 5. Waters, Rw, and temperature

### 5.1 Rw / Rmf parameter set (page: swparameters.htm)

Eight water parameters, each with a **temperature** and a **salinity** twin, all
bidirectionally linked: "Changing the Salinity parameter will update the corresponding
fluid resistivity parameter. Changing the fluid resistivity will update the Salinity
parameter."

| Parameter | Note (verbatim where numeric) |
|---|---|
| `Rw` (1) | Formation water resistivity. No default stated in PhiSw. |
| `Rw Temp` | "If Rw Temp is left blank then the Rw value will be assumed to be at formation temperature, and no further conversion will be made." Example given: "the Rw Temp is set to 60 degF. This means that the Rwapp curve computed by the module will be Rwapp at 60 DegF." |
| `Rw Salinity` | direct salinity entry alternative |
| `Rmf` (3) / `Rmf Temp` / `Rmf Salinity` | same pattern |
| `Rw bound` (5) / `Rwb Temp` / `Rwb Salinity` | Dual Water only. "The value can be estimated from the Rw apparent curve in the Shaley wet sections. Rw bound can be adjusted to give 100% Sw in these zones." |
| `Rmf bound` (7) / `Rmfb Temp` / `Rmfb Salinity` | Dual Water flushed-zone twin. "Normally the same as Rw bound but can vary, due to the different responses of the micro resistivity and the deep resistivity curve in Shaley zones." |
| `Rho Sxo zone` | "If Rho Sxo zone is blank the value is calculated from the Rmf parameter. In oil-based mud Rho Sxo is calculated from Rw if the cell/column is left blank." |
| `Salin Sxo zone` | "Flushed zone water salinity **in decimals (e.g. 25 KPPM entered as .025)**. Used for making formation salinity corrections to the neutron log and calculating TP water. If Salin Sxo zone is left blank, the value is calculated from the Rmf parameter." |

**A hard requirement, verbatim:** "The Temperature is an essential curve and must be
calculated and selected as an input for this module to operate."
(page: porosityandwatersaturation.htm)

Salinity↔resistivity and mud-filtrate-density conversions are **rasterized**
(`embim39`, `embim40`). Terms named: "Temp = Entered formation temperature °F",
"Rmf75 = Rmf value converted to 75°F". So IP's reference temperature for that conversion
is **75 °F**, stated verbatim. "For oil-based mud, if filtrate salinity or density are not
entered, they are calculated from Rw using the same equation as above."
(page: swequationsandmethodology.htm)

### 5.2 Rw from SP module (page: rwfromsp.htm)

Purpose: "create a continuous Rw curve". Menu: Calculation → Rw from SP.

Inputs and rules, verbatim:
- "Enter the baseline-shifted SP curve. **The shale baseline must have been set to 0.0 mv.**
  The baseline shifts can be made in the Interactive Baseline Shift module."
- "A Formation Temperature curve must be entered."
- "The result RwSP curve will be calculated and corrected to the output temperature entered.
  The output temperature can either be a curve or a fixed value."
- "The optional Salinity curve (leave box blank if not wanted) converts the RwSP results to
  a salinity in **units of Kppm NaCl equivalent**."
- "Formation Waters: Select either **NaCl Formation Waters** or **Average Fresh Formation
  Waters**. If Average Fresh Formation Waters are selected the module uses **the chart SP-2
  in the Schlumberger chart book using the dashed line on the chart** to convert from Rweq
  to Rw."
- "Top and Bottom boxes allow the specification of the calculation interval. If left blank
  the whole well will be used."
- **Caveat, verbatim:** "Note: Calculations are presuming predominantly Sodium Chloride
  waters."

**No equations, no coefficients, no numeric defaults are given on this page at all.**
The Rweq→Rw step is delegated to a third-party chart (Schlumberger SP-2). For SandiBumi
this is a gap: the SSP coefficient, the Rweq relation, and the fresh-water dashed-line
transform must come from the primary Schlumberger chartbook (Jauhar's 2013 chartbook
digitization already covers this family), not from IP.

### 5.3 Temperature Gradient module (page: temperaturegradient.htm)

- Two modes: "entering a temperature gradient, or by entering temperatures at fixed points
  and IP will interpolate between them."
- **Gradient units, verbatim:** "The temperature gradient is entered in **degrees per 100
  feet or meters, depending on the units of the well**. A reference depth and temperature
  also need to be entered to give a starting point for the temperature curve."
- Output curve units toggle: "**F for Fahrenheit, C for Centigrade**" — "The output curve
  units are important and are used in the interpretation modules to make the correct
  temperature conversions."
- Null-before-run option, and Save/Load of module parameters "particularly important if you
  want to use this module in the Multi-Well batch module."
- **No default gradient value, no default surface temperature, no default reference depth
  are stated.** Related module listed: Horner Plot.

---

## 6. Limits, bad hole, and silt

(page: swparameters.htm unless noted)

| Parameter | Stated behaviour / value (verbatim) |
|---|---|
| `Phi max` | "should be set to the Maximum likely porosity value in silt-free sand, in decimal porosity units." No default. |
| Silt index | **`VSILT = 1 - VolwetClay - (Phie / PhiMax)`** (ASCII, recoverable; rasterized `embim134` in Final Calculations). Verbatim caveat: "It is important to note that the Silt Index is simply that, an index. That Silt volumes are not designed to be accurate and that the VSILT curve is used only for the Lithology display". |
| `Delta Phi max` + `Vcl cut-off` | **`Phie <= (PhiMax+DeltaPhiMax)*(1.0-Vcl)*10**(-10.0*(Vcl-Vclcut-off)**1.6))`** applied when `Vcl > Vcl cut-off` (ASCII, recoverable; rasterized `embim71`–`embim73`). No default for either parameter. |
| `m vari wth Vcl` | `m = m*10**(Vcl - Vcl cut-off)` when `Vcl > Vcl cut-off`. |
| `Vcl cut-off` (clay-volume side-effect) | "If Vcl > Vcl cut-off and variable Vcl logic (on the Phi Logic tab) is chosen then: VolWetClay (Vcl out) >= Vcl input." |
| `Phie Sw Limit` | "when effective porosity (PHIE) < Phie Sw Limit value, Sw and SwT etc.. will be set to 1.0 (100%)." No default. |
| `Phie Limit` | "when effective porosity (PHIE) < Phie Limit value, effective porosity (PHIE) will be **set to 0.001**, Sw, SwT etc.. will be set to 1.0 (100%)." No default for the threshold. |
| `Vcl Limit` | "when Volume Wet clay (VWCL) > Vcl Limit value, effective porosity (PHIE) will be **set to 0.0001**, Sw, SwT etc... will be set to 1.0 (100%)." No default for the threshold. |
| `Swi Limit` | "when the saturation calculations give Sw, SwT etc. < Swi Limit, then the resultant Sw, SwT etc... will be set to the Swi Limit value before being used in other calculations (BVW, BVWSXO, PHIE etc.)" No default. |

> **Manual inconsistency, recorded not resolved.** The bullet summary at the top of the
> Limits/Badhole section says "set effective porosity to a very small number (**0.0001**)
> and Sw to 1.0 (100%) where (effective) porosity is below a Limit value", but the
> `Phie Limit` parameter entry says **0.001**, while `Vcl Limit` and PHIFLAG code 9 both say
> **0.0001**. Three statements, two numbers. All quoted verbatim above.
> (page: swparameters.htm; PHIFLAG on swequationsandmethodology.htm)

**Bad Hole Discriminator** — a curve plus `Disc Min` / `Disc Max`, output is the minimum of
sonic porosity and the model porosity. Two traps stated verbatim:
- "You should only set up the discriminator with either the Disc Min or Disc Max parameter
  for a single zone. NOTE: **the logic DOES NOT WORK if BOTH discriminators are set in a
  zone.**"
- "NOTE: If the BadHole Input Curve contains null values (-999) then the PhiSw calculation
  will also be nulled at those depths."
- "If both the Disc Min and Disc Max parameters are left blank and the Bad Hole Disc flag is
  turned on, then Phi Model is always limited to be less than Phi Sonic."

**Force 100% Wet** — per-zone: "Sw, Swu, SwT, SwTu, Sxo, Sxou, SxoT, SxoTu will all be set
to 1.0. Phiflag logic curve will be set to 16... **no hydrocarbon corrections will be made
to the porosity**. Comparison water saturation output curves will still be computed
normally."

**Kill Logic** — `Kill Val/Crv 1`, `Kill Val/Crv 2`, `Kill Operator`. "When the logic is
true the porosity and volume outputs will be set to 0.0 and the water saturations will be
set to 1.0. All other outputs will be set to null values." Intended for "cased intervals,
extreme bad hole or volcanic sections".

---

## 7. Coal / Salt / Anhydrite discrimination

Three three-log tests, all with the same "blank = skip that criterion" rule
(page: swparameters.htm; confirmed on swequationsandmethodology.htm):

| Lithology | Test |
|---|---|
| Coal | `Density < Rho Coal` AND `Neutron > Neu Coal` AND `Sonic > Dt Coal` |
| Salt | `Density < Rho salt` AND `Neutron < Neu salt` AND `Sonic < Dt salt` |
| Anhydrite | `Density > Rho Anhydrite` AND `Neutron < Neu Anhydrite` AND `Sonic < Dt Anhydrite` |

"If either ... is left blank, then they will not be used in the logic."
"It is not necessary to use all the porosity input tools to flag the Salt or Coal; only
those which work."
On a hit: "the output curve VCOAL or VSALT will be set to a value of 1.0 over those
intervals, and all porosity outputs will be set to 0.0 and all water saturation outputs
to 1.0." **No numeric default is stated for any of the nine thresholds.**

---

## 8. Laminated sand / thin-bed

(pages: swparameters.htm, swequationsandmethodology.htm, porosityandwatersaturation.htm)

`Sat Model` = **Normal** | **Laminated**. In Laminated mode, "saturations are calculated in
the sand lamination using the porosity, clay volume and resistivity of the sand lamination.
Once the sand laminated Sws are calculated the normal Sw and SwT will be calculated,
**assuming that there is no hydrocarbon in the shale laminations**."

Substitution set, verbatim: `PhiT = PhiTLam`, `Phie = PhiLam`, `Vcl = VclLam`,
`Rt = RtLam`, `Rxo = RxoLam`.

**Recoverable equations (`Rt Lam Model = "Normal"`):**
```
RtLam  = (1.0 - Vlam) * Rt  * Rshale / (Rshale ? Rt  * Vlam)
RxoLam = (1.0 - Vlam) * Rxo * Rshale / (Rshale ? Rxo * Vlam)
Sw  = SwLam
SwT = 1.0 - PhiTLam * (1.0 - SwTLam) * (1 - Vlam) / PhiT
```
(The `?` is a mangled minus/en-dash in the decompiled text; the raster original is
authoritative. Recorded as-is, not "corrected".)

**Numeric limits and the sensitivity warning — the most quotable passage in the whole target:**
- "the RtLam and RxoLam are **limited to 2000 ohmm**."
- "The three lines have been made to all go through **40% shale lamination**. The three
  lines represent a resistivity of shale of **1.0, 1.5 and 2.0 ohmm**. The resulting sand
  resistivities are respectively; **50.0, 5.75 and 2.83 ohmm**. Hence, a small change in
  the pick of shale resistivity can have a large effect on the calculated Sw results.
  Hence, **it is much better if an external, calculated sand resistivity can be used**."
- `Vlam` "must be between **0 and 0.99**"; `Vlam + Vstruc + Vdisp = Vcl`.
- `SwLam` "is limited to be between **1.0 and 0.001**".

**Tensor (Rv/Rh) routes** — three models, all with rasterized core equations:
- **`Tensor Vlam`** — "solved for Rsand, RshVert and RshHori. The inputs are the Vlam which
  is taken from the Thomas Stieber Vlam and the anisotropic ratio between RshVert and
  RshHori which is an input parameter ('RshVert / RshHori')." Two-root ambiguity, verbatim:
  "There are two solutions to this problem one when Rsand is < Rshale and the other Rsand >
  Rshale. We use the input parameter 'Res Lam Shale' to determine which of the two solutions
  is correct... **The user must select the correct solution manually.**"
- **`Tensor Rsh`** — "solved for Rsand and Vlam. The inputs are the parameters RshVert and
  RshHori. Vlam is output as the 'VlamTensor' curve."
- **`Tensor Rsh Mod`** — as above but VlamTensor overrides Thomas-Stieber:
  `VLam = VlamTensor`, `Vlam <= Vwcl`, then
  `Vdisp = PhiMax (1 ? Vlam) ? Phie` and `Vstruc = Vwclay ? Vlam - Vdisp`.
- Anisotropy failure handling, verbatim: "If the anisotropic shale ratio input is too high
  the calculations will not always find a possible result. In these cases the anisotropic
  ratio is reduced until a solution that is possible is found. The 'PhiFlag' output curve is
  then set to a value of **15**."
- The **"butterfly" crossplot** (Rv vs Rh with constant-Vlam and constant-Rt-sand overlays)
  is IP's shale-point picker for this. Name worth registering.

**Thomas-Stieber / clay-type distribution:**
- Verbatim attribution: "The method comes from **the Juhasz SPWLA paper 'Assessment of the
  distribution of shale, porosity and hydrocarbon saturation in shaley sands'**. The
  techniques described within IP are the same as in the Thomas-Stieber paper but developed
  further so that Phie and Vcl can be used directly rather than the raw Neutron / Density
  crossplot. This allows for hydrocarbon corrected results."
  (No year/volume given for either paper — a citation gap.)
- Branch rule, verbatim: "If Vlam < Vcl then the model Laminated/Structural is used" with
  `Vdisp = 0`, `Vstruc = Vcl - Vlam`; "Otherwise the Dispersed/Laminated model is used" with
  `Vstruc = 0`, `Vdisp = Vcl - Vlam`. "**There is no option to have a Dispersed / Structural
  model.**"
- Clay Model vs Shale Model toggle: Thomas-Stieber volumes computed from a Vshale/Phie plot
  instead of Vclay; "the lines on the crossplot are now in shale volume but are internally
  converted back to clay volume for use in the rest of the module."
- Twelve output curves: Vlam, Vdisp, Vstruc, PhieLam, PhiTLam, SwLam, SwTLam, BVWLam,
  BVWTLam, VclLam, RtLam, RxoLam.

**Restriction that matters most (repeated twice for two equations), verbatim:**
"Note this equation assumes a formation of laminated sands and shales with the sands being
clean. **Do not use this equation if the Laminated Sw model options are turned on since
this would be double correcting for laminations.**" — applies to **Poupon-Aguilera** and
**Poupon-Tixier**. (page: swequationsandmethodology.htm; also stated on swparameters.htm
for Poupon-Aguilera: "Use for Laminated clean Sand / Shale formations. Do not use if the
Laminated Sw model is turned on since this will double compensate for the effect of
laminated sands and shales.")

---

## 9. Multi-mineral (inside PhiSw)

Models: **U/Rho** (3 minerals), **Rho/Dt** (3 minerals), **U/Rh/Dt** (4 minerals), plus
**Mineral 1..4** forcing modes where "mineral volumes will be set to 0 except Mineral N
which will be `1-Phie-Vcl`". (page: swparameters.htm)

**Recoverable U equations (coefficients stated in text; only the ρ symbols are rasterized)**
(page: swequationsandmethodology.htm):
```
U     = Pef     x ( [ρ] + 0.1883) x 0.93423
UClay = PefClay x ( [ρ] + 0.1883) x 0.93423
Uwat  = 0.00481 x Sal + 0.3883
Gas ([ρ] less than 0.4):  Uhyd = 0.119 x [ρ]
Oil:                      Uhyd = 0.133 x [ρ]
```
The bracketed `[ρ]` marks a rasterized symbol (`embim83`–`embim87`); the numeric
coefficients 0.1883, 0.93423, 0.00481, 0.3883, 0.4, 0.119, 0.133 are literal text.
Note the **gas/oil switch at hydrocarbon density 0.4**, same number as the plot shading cutoff.

Negative-volume handling, verbatim: "If negative volumes are calculated they are set to zero
and the other volumes are recalculated keeping the same ratios so that the total of all
volumes is 1."

True-vs-apparent mineral density, verbatim (worth copying into SandiMin's docs):
"The true mineral density can be different to the apparent mineral density if the minerals
are not the standard Limestone / Dolomite / Quartz. For example, **Clay could have an
apparent matrix density of 3.0 but a true density of 2.5**. This is due to the apparent
crossplot porosity in the clay calculating an apparent matrix density which is much too
high."

`GD source` options: **Fixed value (param)** | **Variable (curve)** | **Multi-mineral**;
"This is the default setting if the multi-mineral options are activated."
(page: swparameters.htm)

`Clay Corr Input` on → Umatrix/Dtmatrix/RhoMatrix corrected with the input Vcl and the Sw
equation clay-corrected; off → "the input Vcl curve is ignored and clay can be calculated
as one of the minerals", with `Min1..4 Clay ?` flags deciding which mineral(s) become VWCL.

**No numeric defaults are stated for any Min1–Min4 Umat / RhoMat / DtMat / Rho True /
Dt True / TP parameter.** (Cross-reference: IP2025's `MINDEF.PAR` is the third endpoint
source already registered in Jauhar's reference tree; IP2018 exposes the same file via
Tools → Defaults → Edit Mineral Solver Mineral System Defaults — page: default_settings.htm.)

---

## 10. EPT / TPL parameters

(page: swparameters.htm — the EPT tab only appears if an EPT/TPL curve is entered)

| Parameter | Default (verbatim) | Units |
|---|---|---|
| `TP water` | "If not entered, TP water will be calculated from formation temperature and salinity in the flushed zone." | nsec/m |
| `TP Lime` | "**Default 9.1.**" | nsec/m |
| `TP Sand` | "**Default 7.2.**" | nsec/m |
| `TP Dol` | "**Default 8.7.**" | nsec/m |
| `TP Clay` | "**Default 8.0 normal range 7-16.**" | nsec/m |
| `TP Hc` | "**Normal values gas 3.3, oil 4.7-5.2.**" (given as normal values, not as a default) | nsec/m |
| `TP Sp mineral` | not stated in manual | nsec/m |

TPL-water calculation is rasterized (`embim41`–`embim43`); named terms verbatim:
"Sal = Salinity of filtrate in ppm10-6", "T = Formation temperature °F", "Rmf = Resistivity
of filtrate at formation temperature", "For oil-based mud, Rw is substituted for Rmf."
Sxo-from-TPL is rasterized (`embim125`). (page: swequationsandmethodology.htm)

---

## 11. Convergence, flags, and final calculations

**Iteration tolerances, verbatim** (page: swequationsandmethodology.htm):
- "[Porosity] difference **< 0.001**"
- "Sxo difference **< 0.002**"

**PHIFLAG codes** (complete, verbatim, page: swequationsandmethodology.htm):

| Value | Meaning |
|---|---|
| 0 | No limits were applied to the results. |
| 1 | Bad hole logic used. Porosity set to be equal to the sonic porosity. |
| 2 | Hydrocarbon iteration loop did not converge after **20** loops. |
| 3 | Porosity set to be equal to the maximum porosity limit. |
| 4 | Porosity was limited to be greater than 0. |
| 5 | Sxo Limit parameter is set. |
| 6 | Neutron/Density model. Porosity was calculated from only the neutron log. Density log was incompatible with selected logic options. |
| 7 | Neutron/Density model. Porosity was calculated from only the density log. Neutron log was incompatible with selected logic options. |
| 8 | Iterative solving of the saturation equations did not converge after **10** loops. |
| 9 | If the calculated VWCL curve is greater than the VCL Limit or the calculated Phie curve is less than the Phie limit then Phie = **0.0001** and all output saturation curves are set to 1.0. |
| 10 | If the calculated Phie curve is less than the Phie Sw Limit then all output saturation curves are set to 1.0. |
| 11 | The Sw or SwT curves have been clipped to the Swi Limit. The SwU or SwT curves have not be changed. |
| 12 | Tensor resistivity, Calculating Rsand from Rv, Rh, RshV, RshH resulted in Vlam >= 1. Rsand set to Rh. |
| 13 | Tensor resistivity ... result gave a determinate < 0. Determinate set to zero and calculation continued. Results could be in error. |
| 14 | Tensor resistivity, Calculating Rsand from Rv, Rh, Vlam result failed Rv = Rh. Rsand set to Rh. |
| 15 | Tensor resistivity ... result failed and shale anisotropy reduced so a solution was found. |
| 16 | Zone has been set 100% wet. All Sw curves, except the comparison Sw equation curves, are set to 1.0. |

**Final calculations recoverable in ASCII** (page: swequationsandmethodology.htm):
```
Vfines    = VWCL + Vsilt
Cwapp     = 1 / Rwapp
PhiT_recp = 1 / PhiT
Rt_Hingle  = Rt^(-1/m)
Rxo_Hingle = Rxo^(-1/m)
Vshale     = VWCL / CSR                       (clamped to a maximum of 1.0)
Grain Den  = (Vmatrix x Rhoma + Vdcl x RhoDclay) / (Vmatrix + Vdcl)
             Vmatrix = 1 ? Phie ? VWCL
BVW > BVWIRR > 0                              (BVWIRR limits)
```
Rasterized in the same section: Vdcl, BVW, BVWsxo, Vsilt, Rwapp, Rmfapp, Qvapp, BVWIRR,
the hydrocarbon-corrected density/neutron/sonic curves, secondary porosity.

From the clay tab, all ASCII (page: swparameters.htm):
```
Rho Wet Clay = Rho Matrix + ((Rho Shale - Rho Matrix) / CSR)
Neu Wet Clay = Neu Matrix + ((Neu Shale - Neu Matrix) / CSR)
Son Wet Clay = Son Matrix + ((Son Shale - Son Matrix) / CSR)
Vshale       = VWCL / CSR
```
`Clay Shale Ratio (CSR)` = "the percentage of clay in 100% shale in decimals (v/v)".
**No numeric default stated for CSR.**

From the output tab (page: porosityandwatersaturation.htm):
```
Den Fluid Rxo Zone = RhoFiltrate * Sxo + (1-Sxo) * RhoHyd
```
"where RhoHyd is the apparent hydrocarbon density, as seen by the density tool i.e.
electron density."

`Ro` output, verbatim: "Ro is calculated from the appropriate Sw equation ... with Sw set
to 1.0. This curve can be useful to judge the appropriateness of the water saturation
model. **Ro should equal Rt in the non-hydrocarbon zones.**"

---

## 12. Sand/Silt Malay Model — separate module, largely recoverable

(page: sand_silt_malay_model.htm — 1 equation image only, so most of the method is
readable text)

**Citation, verbatim:** "The Sand Silt Malay model is based loosely on the paper
'**Log Interpretation in the Malay Basin by K. Kuttan et al, 21st SPWLA symposium**'."
Purpose, verbatim: "designed for very fine grained sediments with fresh to brackish
formation water. It was built to overcome interpretation problems of standard shaley sand
log analysis of the type used in the 1980's, where an overestimate of clay volume leads to
too pessimistic porosities."

Sw equations offered: **Waxman Smt, Juhasz W&S, Dual Water, Archie PhiT** (total-Sw only).
Sxo logic: **ND Separation | Input Curve | Invasion Factor**.

**Recoverable equations, verbatim:**
```
DenCorr  = Rhob + Phie x (1.0 - Sxo) x (Rhofl ? DenHydApp)
NeuCorr  = PhiNeu + exfact + Phie x (1.0 -Sxo) * (1.0 ? NeuHydHI)
exfact   = √(Rhoma/2.65) x (2.0 x SwH x Phie x Phie + 0.04 x Phie) x (1.0 ? SwH)
SwH      = Sxo + (1-Sxo)* NeuHydHI)

Rhoma    = Fsn x Denmat + Fsi x Densilt + Fdc x Dendcl
PhiT     = (Rhoma ? DenCorr) / (Rhoma ? Denfl)

Vsand    = Fsn x (1 ? PhiT)
Vsilt    = Fsi x (1 ? PhiT)
Vdcl     = Fdc x (1 ? PhiT)
PhiTclay = (Dendcl ? Denwcl) / (Dendcl - Denfl)
Vcl      = Vdcl / (1 ? PhiTclay)
Vshale   = Vcl + Vsilt

Phie     = (1 - PhiT) x (PhiT ? Vcl x PhiTclay) + Vcl x (PhiT ? Vcl x PhiT)
Phie    <= MaxPhie x (1.0 ? Vcl)

Vbw      = PhiT ? Phie
if Vbw > Vcl x PhiTclay x 1.5  then  Vbw = Vcl x PhiTclay x 1.5 ; PhiT = Phie + Vbw

Rwapp    = PhiTm x Rt / a
CwApp    = 1.0 / Rwapp
QvNorm   = VCL x PhiTClay / PhiT
QvApp    = a / (B x Rt x PhiTm) - 1.0 / (B x Rw)
RecPhiT  = 1.0 / PhiT
```
The **excavation-factor constants 2.65, 2.0, 0.04** and the **bound-water inflation factor
1.5** are literal text — this is the most fully-specified excavation-factor treatment
anywhere in the target set.

The blended Phie derivation is explained verbatim: "When Vcl is closed to 0, then Phie turns
out to the first equation. When Vcl is closed to1, then Phie turns out to the second
equation." (first = `PhiT ? Vcl x PhiTclay`, second = `PhiT ? Vcl x PhiT`).

**Iteration tolerances (different from PhiSw), verbatim:**
"PhiT difference **< 0.0001**", "SxoT difference **< 0.001**".
Sxo-from-ND search step, verbatim: "SxoT is adjusted in **steps of 0.01** until VshND equals
VshGr."

**Logic Flag codes, verbatim:** 0 ok / 1 ND SxoT HC correction did not match VshGR and
VshND / 2 main iterative HC loop reached limit **max 40** / 3 Sxo correction loop reached
**max 100** / 4 Sw equation iterative loop did not converge after **10** iterations /
5 bad hole correction loop reached **max 100** / 6 bad hole logic used / 7 PhieMax logic
used / 8 Vshale cutoff logic used / 9 input found NullValues so no output.

**Stated parameter guidance:** "To remove the effect of this limit make MaxPhie a high
number (**0.6**)." "Setting Vshale Cutoff to **1.0** will completely remove this cutoff."
"By default this [Vshale cutoff interactive line] has a value of **1.0** and is on the
extreme right edge of the track." No other numeric defaults (`Den Matrix`, `Neu Matrix`,
`Den Silt`, `Neu Silt`, `Den Dry Clay`, `Neu Dry Clay`, `Clay at Silt Point`, `a`, `m`, `n`,
`Rw`, `Rwb`) are stated.

**Bad-hole warning, verbatim:** "Note this logic can have drastic effects on the results if
the VshGr clean and shale picks overestimate the VshGr. **It should only be turned on when
bad hole affects to the density have been identified.**"

**Waxman-Smits usability warning, verbatim:** "The user should plot the data over a wet
shaley interval and, if such a relationship exists, interactively set the line. **If no
relationship exists then the user must enter Qv into the model as a fixed value or curve
after establishing Qv at each depth level using a different methodology.**"

---

## 13. Density Estimation from sonic (Rock Physics)

(page: densityestimation2.htm — 0 equation images, but **no equations are given either**;
only method names, references, and outputs)

| Method | Output curve | Citation the manual gives |
|---|---|---|
| Gardner | `RhoGard` | "Gardner G. L. F., Gardner L.W., & Gregory A.R. ? (1974) Formation velocity and density - the diagnostic basics for stratigraphic traps Geophysics 39, 770-780." |
| AGIP Bellotti | `RhoAgip` | "Bellotti, P. Di Lorenzo, V. & Giacca, D. - Overburden gradient from sonic log Trans. SPWLA, London March 1979" |
| Lindseth | `RhoLind` | "Lindseth, R. O., (1979) ? Synthetic Sonic Logs ? a process for stratigraphic interpretation, Geophysics v.44 no.1 p.3-26" |

"Parameters such as the coefficients for the Gardner transform can be edited, saved and
re-called." **No coefficient values are stated.**

Seismic-ahead-of-bit note, verbatim: "it can be used ahead of drilling, by utilizing seismic
data, converting seismic interval velocities (V, in ft/sec) to sonic transit times
(1/V \*10 6 (usec/ft))."

Vendor note, verbatim: "Bellotti et al found more acceptable results comparing actual FDC
curves to their **Unconsolidated formations equation**. (illustrated by examples from the
Adriatic, Mauritania and **Indonesia**)."

---

## 14. Unit conventions (Tier A)

| Quantity | IP convention (verbatim where quoted) | Source |
|---|---|---|
| Density | "gm/cc (kg/m3)" throughout the parameter set | swparameters.htm |
| Sonic | "μSec/ft (μSec/m)" | swparameters.htm |
| Neutron | "decimals (v/v)" / "decimal porosity units"; **input assumed limestone matrix units** | swparameters.htm, porosityandwatersaturation.htm |
| Clay/shale volume, porosity, saturation | decimals (v/v), not percent | swparameters.htm |
| Salinity (parameters) | "in decimals (e.g. 25 KPPM entered as .025)" | swparameters.htm |
| Salinity (RwSP output) | "Kppm NaCl equivalent" | rwfromsp.htm |
| Salinity (EPT calc) | "ppm10-6" | swequationsandmethodology.htm |
| Neutron table salinity nodes | "50, 100, 150, 200, 250 Kppm" | swequationsandmethodology.htm |
| EPT propagation time | "nsec/m" | swparameters.htm |
| Temperature | °F or °C, toggled per curve; parameter temps follow the curve-input/output window setting | swparameters.htm, temperaturegradient.htm |
| Temperature gradient | "degrees per 100 feet or meters, depending on the units of the well" | temperaturegradient.htm |
| Rmf conversion reference | "Rmf75 = Rmf value converted to 75°F" | swequationsandmethodology.htm |
| B (Juhasz) | labelled "meq/ml" (see flag in §3.5) | swparameters.htm |
| SP baseline | "The shale baseline must have been set to 0.0 mv." | rwfromsp.htm |
| Null value | -999 | swparameters.htm |
| Internal working units | "IP works with Density, Sonic and Caliper log curves defined in units of grams per cubic centimeter, microseconds per foot and inches, respectively." Conversions live in `UnitConversion.par`. | default_settings.htm |

---

## 15. Configuration file inventory (Tier A market intel)

From `default_settings.htm` and `swequationsandmethodology.htm` — IP's whole extensibility
surface is plain-text files in the IP directory, overridable at project level and shareable
via "Corporate Search Folders":

| File | Contents |
|---|---|
| `Neu_Parm_Files.neu` | registry of neutron tool names → look-up table files |
| `<tool>.neu` (e.g. `Sch_CNL.neu`) | per-tool neutron matrix + salinity correction tables |
| `MonteCarloDefaults.par` | Monte Carlo distributions + high/low shifts; keyed by the `(nn)` parameter indices on the PhiSw tabs |
| `MINDEF.PAR` | Mineral Solver default minerals and properties |
| `MINEQDEF.PAR` | Mineral Solver equation defaults |
| `UnitConversion.par` | recognized unit abbreviations + factors for density/sonic/caliper |
| `CparmDef.xml` / `CPARMDEF_USER.PAR` | curve display defaults (colors, scales) |
| `CurveType.opt` / `UserCurveType.opt` | generic curve types for auto-selection |
| `CurveAlias.txt` | load-time mnemonic aliasing (also usable as a LAS batch mask) |
| `Lithology.opt`, `DefaultUnits.opt`, `ShadeTypes.opt` | project-level color/lithology/curve settings |

Parameter sets: saved in the database, plus a `.TXT` listing via Print Parameter Set;
default set name for this module is `PhiSw`. (page: porosityandwatersaturation.htm)

---

## 16. Explicit caveats and "do not use" statements (collected)

1. **Poupon-Aguilera / Poupon-Tixier vs laminated model** — "Do not use this equation if the
   Laminated Sw model options are turned on since this would be double correcting for
   laminations." (swequationsandmethodology.htm, swparameters.htm)
2. **Bad-hole discriminator** — "the logic DOES NOT WORK if BOTH discriminators are set in a
   zone." (swparameters.htm)
3. **Bad-hole null propagation** — nulls in the discriminator curve null the whole PhiSw
   calculation at those depths. (swparameters.htm)
4. **Comparison curves are not deliverables** — "they should not be used for anything other
   than this"; the summation Sw must be the `Sat Equation` result. Same for comparison
   porosities. (porosityandwatersaturation.htm)
5. **Silt index is an index** — "Silt volumes are not designed to be accurate ... used only
   for the Lithology display." (swparameters.htm)
6. **Laminated Rsh sensitivity** — a 1.0→2.0 ohmm shale pick moves sand resistivity
   50.0→2.83; "it is much better if an external, calculated sand resistivity can be used."
   (swequationsandmethodology.htm)
7. **Tensor two-root ambiguity** — "The user must select the correct solution manually."
   (swequationsandmethodology.htm)
8. **Organic shale model restriction** — only Density Neutron / Density / Neutron; "Other
   models will cause an error message to be displayed." With N/D, only variable-matrix-
   density logic works. (porosityandwatersaturation.htm, swequationsandmethodology.htm)
9. **Variable Sxo needs a partner flag** — must also tick another Variable option for when
   Sxo is clipped. (swparameters.htm)
10. **Neutron input must be limestone units** — otherwise convert first.
    (porosityandwatersaturation.htm)
11. **Duplicate output names** — Basic Log Analysis "will overwrite any current output curves
    with the same name, without warning", and shares many names with PhiSw.
    (basicloganalysis.htm)
12. **Rw from SP assumes NaCl** — "Calculations are presuming predominantly Sodium Chloride
    waters." (rwfromsp.htm)
13. **Malay bad-hole logic** — "should only be turned on when bad hole affects to the density
    have been identified." (sand_silt_malay_model.htm)
14. **Malay Waxman-Smits** — if no Qvapp/1-PhiT relationship exists, Qv must come from an
    independent methodology. (sand_silt_malay_model.htm)
15. **Close ≠ cancel** — closing the Parameters window keeps the edits but does not apply
    them until something else forces a recalculation. (swparameters.htm)

---

## 17. Tier-C flags (name + evidence only — nothing transcribed)

Nothing patented or branded in the IP2025 sense (SonicSaturation, Domain Transfer Analysis,
Experienced Eye, entropy-based image speed correction, shipped NN weights) appears in this
target. Two items are flagged as **proprietary calibration / third-party data**, not as
patents:

1. **Sand/Silt Malay "Lithology Conversion Chart"** (page: sand_silt_malay_model.htm).
   The dry-sand / dry-silt / dry-clay fractions are read off a five-curve chart, and the
   manual states verbatim: "The position of A and B points along Y axis, and the shape of
   those five curves are **determined based on the characteristics of core data acquired in
   Malay Basin**." The chart itself is a raster figure; the curve shapes are an undisclosed
   proprietary calibration. **Record the existence and the Kuttan et al. citation only.**
   Do not attempt to digitize IP's chart or fit its curves. If SandiBumi wants this model it
   must be calibrated from Jauhar's own core data, and the fitted shape documented as his.
2. **Shipped neutron tool look-up tables** (`Sch_CNL.neu` and siblings). The *format* and the
   *construction recipe* are Tier A and fully documented. The *values* are digitizations of
   service-company chartbooks redistributed by IP. **Adopt format + recipe; digitize values
   from the primary vendor chart.**

Also worth noting, not Tier C but unattributed: the **"Modified" density-hydrocarbon model**
and the **Shell m formula** and the **two m\* formulas** are given without any citation.
SandiBumi should trace each to a primary source before adopting, or mark them as
"IP-documented, source untraced".

---

## 18. Gaps — what was expected and not found

1. **No `a` / `m` / `n` defaults in the PhiSw module.** The only stated Archie defaults in
   the whole manual are Basic Log Analysis's `a=1.0`, `m=2.0`, `n=2.0`, `Rw=0.1 ohmm`.
   The main module states none.
2. **No Rw default, no formation-temperature default, no temperature-gradient default** in
   PhiSw or in the Temperature Gradient module.
3. **No `Qv a Const` / `Qv b Const` numeric defaults**, and no `B fact W-S` numeric default —
   only "calculated from formation temperature and Rw", with that equation rasterized.
4. **The B(T,Rw) equation is rasterized** (`embim119`) — IP's exact Juhász/W-S B form is not
   recoverable from this help file. (Jauhar's verified Juhász B formula in the brain remains
   the reference; IP's is unverifiable from here.)
5. **No `Rw from SP` equations at all** — no SSP coefficient, no Rweq relation, no
   fresh-water transform. Delegated to Schlumberger chart SP-2.
6. **No Gardner / Bellotti / Lindseth coefficients** — methods and citations only.
7. **No citations** for Archie, Simandoux, Modified Simandoux, Indonesian/Poupon-Leveaux,
   Dual Water, Waxman-Smits, Juhász, Wyllie, Raymer-Hunt, the Shell m formula, or the two
   m\* forms. Only Woodhouse, Aguilera, Poupon-Tixier, Thomas-Stieber/Juhász (untitled year),
   Kuttan, Gardner, Bellotti, Lindseth are cited.
8. **No thresholds for coal / salt / anhydrite** (nine parameters, zero defaults).
9. **No multi-mineral endpoint values** (Umat / RhoMat / DtMat / true densities / TP) —
   those live in `MINDEF.PAR`, not in the help.
10. **No Sand/Silt Malay endpoint defaults** (quartz / silt / dry-clay density and neutron,
    `Clay at Silt Point`).
11. **121 of the methodology page's formulas are rasterized.** Recovering them would require
    OCR of `embim*.gif` from the CHM — deliberately not attempted, and any reconstruction
    from general knowledge would violate the no-invented-numbers rule.
12. **`Neu Matrix`, `Sonic matrix`, `Rho Sp mineral`, `Neu Sp mineral`, `DT Sp mineral`,
    `Pef Clay`, `Res Clay`, `Rxo Clay`, `Clay Shale Ratio`** — described but no defaults.
