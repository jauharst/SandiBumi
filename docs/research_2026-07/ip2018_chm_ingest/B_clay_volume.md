# IP 2018 CHM ingest — Target B: Clay Volume, Lithology & Basic Log Analysis

Source: decompiled help text of Interactive Petrophysics 2018 (PGL / Lloyd's Register / Geoactive),
`C:\Users\ARUNIKA\AppData\Local\Temp\c18\_text\*.txt`. Install `C:\Program Files\IP2018` is read-only
and was not modified.

Pages read: `clayparameters.htm`, `clayequationsandmethodology.htm`, `clayvolume.htm`, `clayplot.htm`,
`createeditlithologycurves.htm`, `basicloganalysis.htm`, `basiclogcalculations.htm`.
Supporting pages consulted for cross-references: `swparameters.htm`, `curve_average.htm`, `tools.htm`.

**Rule applied throughout:** every number below is one the manual actually states, quoted as written.
Where the manual states no value the entry reads `not stated in manual`. Nothing has been rounded,
unit-converted, tidied, or supplied from petrophysical knowledge.

---

## 0. THE HEADLINE ANSWER — are the Larionov / Clavier / Stieber coefficients recoverable?

**No. They are lost to raster.** `clayequationsandmethodology.htm` carries `equation_images: 16`, and
every one of the six gamma-ray Vcl forms, plus SP, Neutron, Resistivity, Other-Linear and all four
double-indicator forms, is a rasterized GIF. The decompiled text preserves only the *method label* and
the surrounding prose.

| Method | Equation token in text | Recoverable as text? |
|---|---|---|
| GR Linear | `[[EQUATION_IMAGE: embim23.gif]]` | No — raster |
| GR Curved (composite) | `embim24.gif` (Z<0.55), `embim25.gif` (0.55<Z<0.73) | No — raster. **Branch thresholds ARE in text** (see §1) |
| GR Clavier | `embim26.gif` | No — raster |
| GR Stieber | `embim27.gif` | No — raster. **Shape constant default IS in text** (2.0) |
| GR Larionov older rocks (Mesozoic) | `embim28.gif` | No — raster |
| GR Larionov younger rocks (Tertiary clastics) | `embim29.gif` | No — raster |
| SP | `embim30.gif` | No — raster |
| Neutron | `embim31.gif` | No — raster |
| Resistivity | `embim32.gif`, `embim33.gif` | No — raster. **Rt > 2 × Rclay branch IS in text** |
| Other Linear | `embim34.gif` | No — raster |
| Neutron/Density | `embim35.gif` | No — raster |
| Sonic/Density | `embim36.gif` | No — raster |
| Neutron/Sonic | `embim37.gif` | No — raster |
| Other Double | `embim38.gif` | No — raster |

What survives as text and IS therefore usable: the **Curved-method branch thresholds (0.55, 0.73)**,
the **Stieber shape constant (default 2.0)**, the **resistivity branch condition (Rt > 2 × Rclay)**,
the **bad-hole null value (-999)**, and the **entire organic-shale correction set** (§6), which is
written out longhand.

**No reconstruction has been attempted.** The Larionov coefficients in particular are exactly the
silent-wrongness case the ingest brief warns about — a wrong exponent computes, plots, and ships.
SandiBumi must implement Larionov / Clavier / Stieber from the primary literature, not from here.

The GIF files themselves do exist on disk at
`C:\Users\ARUNIKA\AppData\Local\Temp\c18\embim23.gif` … `embim38.gif`
(and `embim14.gif`–`embim22.gif` for Basic Log Analysis, `embim5.gif`–`embim13.gif` for Basic Log
Functions). A human can open them directly. They were deliberately **not** OCR'd into this report —
a vision-transcribed exponent is indistinguishable from an invented one once it is in a text file.

---

## 1. Clay / shale volume methods IP 2018 offers

### 1.1 Single Clay Indicators

Each single indicator is a two-point (Clean, Clay) transform of one curve. Selected indicators are laid
out two per tab in the parameter grid: *"A maximum of two Single indicators will be stored on each tab."*
(page: clayparameters.htm)

| Indicator | Use flag | Clean param | Clay param | Method choice | Equation |
|---|---|---|---|---|---|
| Gamma Ray | (1) GR Use Flag | (2) Gr Clean | (3) Gr Clay | (4) Gr Method — 6 options | raster |
| Neutron | (5) Neu Use Flag | (6) Neu Clean | (7) Neu Clay | fixed | raster (`embim31.gif`) |
| SP | (8) SP Use Flag | (9) SP Clean | (10) SP Clay | fixed | raster (`embim30.gif`) |
| Resistivity | (11) Res Use Flag | (12) Res Clean | (13) Res Clay | fixed + branch | raster (`embim32/33.gif`) |
| Other (user-named) | (14) Oth Use Flag | (15) Oth Clean | (16) Oth Clay | linear | raster (`embim34.gif`) |

(page: clayparameters.htm for all parameter numbers and names)

**The six Gr Method options, quoted verbatim** (page: clayparameters.htm):

- *"Linear : Linear relationship between Gr and VclGr"*
- *"Curved : A composite curved relationship."*
- *"Clavier : As per Clavier et al."*
- *"Stieber : As per Stieber et al (South Louisiana Miocene and Pliocene)."*
- *"Old Rock : As per Larionov et al for older rocks (Mesozoic)."*
- *"Young Rock : As per Larionov for younger rocks (Tertiary clastics)."*

Note the spelling: IP writes **Stieber** (not "Steiber") on both `clayparameters.htm` and
`clayequationsandmethodology.htm`.

**Citations the manual gives:** author surnames and rock-age applicability only. There is **no
year, journal, volume or paper title anywhere on either page** for Clavier, Stieber or Larionov.
This is a citation gap, not a Tier-B harvest — SandiBumi must source the primary papers independently.

**Curved method — the one GR method with recoverable structure** (page: clayequationsandmethodology.htm):

> `Z = VclGr as above` (i.e. Z is the linear result)
> `for Z less than 0.55` → `[[EQUATION_IMAGE: embim24.gif]]`
> `for Z greater than 0.55 and less than 0.73` → `[[EQUATION_IMAGE: embim25.gif]]`
> `for Z greater than 0.73 and less than 1.0` → `VclGr = Z`

The two breakpoints **0.55** and **0.73** and the identity branch above 0.73 are text. The two curved
branch expressions are raster.

**Stieber** (page: clayequationsandmethodology.htm):
> `Z = VclGr linear`
> `STB = Stieber Constant shape parameter (default =2.0)`

**Resistivity indicator branch** (page: clayequationsandmethodology.htm):
> `for Rt greater than 2 x Rclay then [[EQUATION_IMAGE: embim33.gif]]`
> `otherwise VclRes = Z`

Applicability caveat, verbatim (page: clayparameters.htm):
> *"The resistivity indicator will generally only work in hydrocarbon-bearing zones where the surrounding shales have low resistivity."*

Res Clean picking convention (page: clayparameters.htm):
> *"Value of the resistivity in a clean, zero Vclay zone. Generally chosen as the highest resistivity in a hydrocarbon-bearing, clay-free zone."*

Neutron caveat, verbatim (page: clayparameters.htm):
> *"NOTE: When setting the Neu Clean Parameter, exercise caution if setting this parameter to any other value than zero. This indicator can easily under-estimate the clay volume if the parameter is set too high."*

### 1.2 Double Clay Indicators

Principle, verbatim (page: clayequationsandmethodology.htm):
> *"The Double Clay Indicators work on the principle of defining a clean line and a clay point . The clay volume is calculated as the distance the input data falls between the clay point and the clean line ."*

Each double indicator is parameterised by **one clay point (2 coords) + a two-point clean line (4 coords)** = 6 parameters.

| Indicator | Clay point | Clean line pt 1 | Clean line pt 2 |
|---|---|---|---|
| Neutron/Density (ND) | (18) ND Neu Clay, (19) ND Den Clay | (20) ND Den Clean1, (22) ND Neu Clean1 | (21) ND Den Clean2, (23) ND Neu Clean2 |
| Sonic/Density (SD) | (25) SD Son Clay, (26) SD Den Clay | (27) SD Den Clean1, (29) SD Son Clean1 | (28) SD Den Clean2, (30) SD Son Clean2 |
| Neutron/Sonic (NS) | (32) NS Neu Clay, (33) NS Son Clay | (34) NS Son Clean1, (36) NS Neu Clean1 | (35) NS Son Clean2, (37) NS Neu Clean2 |
| Other Double (OD) | (39) OD Curv1 Clay, (40) OD Curv2 Clay | (41) OD Curv1 Clean1, (43) OD Ot1 Clean1 | (42) OD Ot2 Clean2, (44) OD Ot1 Clean2 |

(page: clayparameters.htm — parameter names transcribed exactly, including the inconsistent
`Curv1/Curv2` vs `Ot1/Ot2` naming in the OD block, which is as-printed in the manual)

Use flags: (17) ND Use, (24) SD Use, (31) NS Use, (38) OD Use.
> *"The other double clay indicator is calculated just like the Neutron /Density clay indicators with a clay point and a clean line."* (page: clayparameters.htm)

**Hard input-unit contract**, stated twice (pages: clayvolume.htm, clayparameters.htm):
> *"IP makes the assumption that any neutron curve entered is in Limestone matrix units ."* (clayvolume.htm)
> *"NOTE: the Input Neutron Porosity curve should have been recorded in limestone porosity units."* (clayparameters.htm)

### 1.3 Bad Hole gating of double indicators

Four parameters: (45) BadH1 Use, (46) BadH1 Min, (47) BadH1 Max, (48) BadH2 Use, (49) BadH2 Min,
(50) BadH2 Max (page: clayparameters.htm).

Logic, verbatim (page: clayparameters.htm):
> *"When the Bad Hole Indicator 1 curve values are greater than this minimum value , any double clay indicators will be switched off."* (BadH1 Min)
> *"When the Bad Hole Indicator 1 curve values are less than this maximum value , any double clay indicators will be switched off."* (BadH1 Max)
> *"When the parameter is left blank, the discriminator curve is ignored."*

Note the deliberately counter-intuitive naming: **Min is an upper gate** (fires above it) and **Max is a
lower gate** (fires below it). Confirmed on `clayplot.htm`:
> *"Set the BadH1 Min parameter to the maximum allowable caliper value (in caliper curve units)."*

Two-sided gating (over-gauge + closed caliper) is done by selecting the same caliper curve into **both**
Bad Hole Indicator name boxes and setting `BadH1 Max` and `BadH2 Min` (page: clayplot.htm).

Output on a bad-hole flagged interval, verbatim (page: clayequationsandmethodology.htm):
> *"If the Bad Hole indicator logic is used, then over any interval which is flagged as Bad Hole all the Double Clay Indicators will be set to Null (-999)."*

Bad-hole gating applies to **double indicators only** — single indicators are not gated.

Example value seen in the manual (an illustration, **not** a stated default) (page: clayplot.htm):
> *"...turn off double clay indicators in 2 zones where the Bad Hole Indicator (caliper) curve values exceed 10.5 inches."*

---

## 2. Every stated default and endpoint (verbatim)

### 2.1 Clay Volume module

| Parameter | Stated default | Source |
|---|---|---|
| (61) Stieber Constant | *"Default is 2.0"* | clayparameters.htm |
| (57) Percentile Clay | *"(default is 130%)"* | clayparameters.htm; repeated on basicloganalysis.htm |
| (56) Percentile Clean | **not stated in manual** (only an example: *"a 10th percentile for all zones will give the same Gr Clean for all zones"*) | clayparameters.htm |
| (59) Clip Low % | *"Enter a low clip values between 0-100%. Default is (0%)."* | clayparameters.htm |
| (60) Clip High % | *"Enter a high clip values between 0-100%. Default is (98%)."* | clayparameters.htm |
| Gr Clean / Gr Clay (Clay Volume module) | **not stated as a number**; set on first Run from the data — see §2.3 | clayvolume.htm |
| All double-indicator clay/clean coordinates | **not stated as numbers**; set on first Run from the data — see §2.3 | clayvolume.htm |
| BadH1/2 Min/Max | **not stated in manual**; *"Initially, the bad hole indicator logic is not active nor are its cut-off parameters set."* | clayplot.htm |

> **CONFLICT — flag for reimplementation.** `basicloganalysis.htm` states, for its own Clay Volume
> tab: *"Clip Low % : ... Enter a low clip values between 0-100%. Default is (98%)."* and gives **no**
> default for Clip High %. `clayparameters.htm` states Clip Low = 0%, Clip High = 98%. The two pages
> disagree; the Basic-Log-Analysis page appears to have the Clip-High default text pasted onto Clip
> Low. Both are recorded verbatim above. **Do not silently pick one.** The physically sensible pairing
> (low clip 0%, high clip 98%) is the `clayparameters.htm` one, but that is an inference, not a
> documented value.

### 2.2 Organic Shale tab (Clay Volume module) — all from clayparameters.htm

| Parameter | Stated default (verbatim) |
|---|---|
| (62) Organic Shale Corr. | on/off flag; no numeric default stated |
| (63) Gr Kerogen | **not stated in manual** — *"The value is the Gamma Ray API reading of 100% Kerogen."* Method for deriving it is given (see §6.1) |
| (64) Rhob Kerogen | *"Density of Kerogen. Default is 1.1 gm/cc"* |
| (65) Nphi Kerogen | *"Neutron of Kerogen. Default is 0.6 v/v"* |
| (66) Rhob Heavy_Min. | *"Density of Heavy minerals associated with organic shale (mainly Pyrite). Default is 4.3 gm/cc"* |
| (67) Nphi Heavy_Min. | *"Neutron of Heavy minerals associated with organic shale (mainly Pyrite). Default is -0.03 v/v"* |
| (68) Kerogen Wt%_Con. | *"Conversion factor to convert input TOC curve, in weight %, to a volume curve. Default is 2.5"* |
| (69) Heavy_Min. Wt%_Con. | *"Conversion factor to convert input Heavy Mineral curve, in weight %, to a volume curve. Default is 1.0 (input is already in volumes)."* |

### 2.3 How the *initial* endpoints are chosen (the auto-default rule)

Verbatim (page: clayvolume.htm):
> *"If this is a new Run , then default parameters will be calculated for each clay indicator and these values will populate the Clay Volume Parameters Set. These defaults are not meant to be the optimum interpretation parameters, but are chosen simply to set the parameters within the correct range of values for a particular single curve or pair of curves."*
> *"For the Single Clay Indicators the defaults are generally the maximum and minimum readings for the indicator."*
> *"For the Double Clay Indicators the defaults will be the sandstone line for the clean line , and the shale point will be chosen towards the bottom right hand edge of the shale points , as seen on a standard crossplot."*

The Basic Log Analysis module states the same rule concretely for GR (page: basicloganalysis.htm):
> *"Gr Clean : ... Should be set to the minimum Gamma Ray value seen in clean (non shaley) intervals. This defaults to the minimum Gamma Ray value seen in the Gamma Ray input curve."*
> *"Gr Clay : ... Should be set to the maximum Gamma Ray value seen in shale intervals. This defaults to the maximum Gamma Ray value seen in the Gamma Ray input curve."*

### 2.4 Percentile-picking convention (the most reusable piece of Tier-A design here)

All verbatim from `clayparameters.htm` unless noted.

**(55) Use Percentiles — a two-way binding, not a one-way switch:**
> *"When on the percentiles entered under the 'Percentile Clean' and the 'Percentile Clay' parameters will be used to calculate the 'Gr Clean' and 'Gr Clay' parameters which are used for the calculation of Vcl GammaRay. When off the ?Gr-Clean? and ?Gr-Clay? parameters are used to calculate their percentiles which are displayed in the ?Percentile Clean? and ?Percentile Clay? parameters boxes."*

(The `?` characters are mojibake for typographic quotes in the decompiled text; left as-is.)

**Percentiles outside 0–100 are legal and expected:**
> *"Numbers greater than 100% are allowed and are normal for the calculation of clay volume (default is 130%). Percentiles over 100% are calculated by a linear scaling from the 0% to the 100% percentiles and extrapolating to the entered number."* (Percentile Clay)
> *"Numbers can be less than 0%. Negative percentiles are calculated by a linear scaling from the 0% to the 100% percentiles and extrapolating to the negative entered number."* (Percentile Clean — page: basicloganalysis.htm)

**(58) Percentile Group — the pooling key:**
> *"Select a number or a letter for each zone. GammRay data from Zones with the same group number are lumped together when calculating or looking up percentiles."*
> *"Hence if all zones use the same 'Percentile Group' number then a 10th percentile for all zones will give the same Gr Clean for all zones."*
> *"...a 90th percentile for all zones will give the same Gr Clay for all zones."*

(10th and 90th here are illustrative numbers in the manual's own example sentences, not stated defaults.)

**Clip-before-percentile ordering:**
> *"The GammaRay curve data for each 'Percentile Group' is clipped using the 'Clip Low %' and the 'Clip High %' parameters before the percentiles are calculated. This allows a percentile of say 130% to be calculated with the removal of spikes in the data before the 130% is calculated."*

So the pipeline is: **pool by Percentile Group → clip low/high → compute percentile → linear-extrapolate beyond 0–100 → that is Gr Clean / Gr Clay.**

### 2.5 Basic Log Analysis defaults (all from basicloganalysis.htm)

| Parameter | Verbatim |
|---|---|
| Rho Matrix | *"Defaults to 2.65 gm/cc for sandstone. Set to 2.71 gm/cc for limestone."* |
| Rho Fluid | *"Defaults to 1.0 gm/cc for fresh water. Set to 1.1 gm/cc for salt water."* |
| Rho Clay | *"The default value is chosen as the maximum density reading over the interval where the sonic is high."* |
| DT Matrix | *"Defaults to 55 uSec/ft for sandstone. Set to 49 uSec/ft for limestone."* |
| DT Fluid | *"Default value is189 uSec/ft. For salt-saturated formation water use about 174 usec/ft."* (the missing space after "is" is as-printed) |
| DT Clay | no default stated; *"Chosen from log plots as the sonic reading in shale."* |
| Sonic Cp | *"Defaults to 1.0. For unconsolidated sands this needs to be increased. A rule of thumb for estimating this parameter is to divide the sonic clay parameter by 100. The result should be greater than 1.0 for a valid Cp value."* |
| Neu Clay | no default stated; *"Can be picked from the interactive Neutron / Density crossplot."* |
| Sat Equation | *"Defaults to Archie."* |
| Rw | *"Defaults to 0.1 ohmm but must be adjusted to the correct value."* |
| a factor | *"Defaults to 1.0."* |
| m Exponent | *"Defaults to 2.0."* |
| n Exponent | *"Defaults to 2.0."* |
| Res Clay | no default stated; *"Can be selected from the Vcl / Resistivity interactive crossplot."* |
| Porosity method | *"The default porosity method is Density unless only the Sonic curve has been enter in which case the program will select the Sonic method for porosity."* |
| Sonic Equ | *"It is recommended that the Raymer equation is used by default since this equation automatically takes care of the problem of unconsolidated sands, whilst the Wyllie equation has an extra compaction factor parameter which has to be estimated."* |

---

## 3. Clay volume vs shale volume — IP's explicit terminology position

This is the cleanest statement of the distinction in the manual, and it is a **workflow decision made
before the module runs**, not a post-hoc relabel (page: clayvolume.htm):

> *"Before running this module a fundamental decision has to be made whether you which to calculate Clay volume or Shale volume. The module has been designed for clay volumes. But can be used just as easily for Shale volume calculations."*

> *"The user selects the type calculation in the above dialog. When selected the window labels, output curves and parameter names will adjust appropriately. Note changing this selection on a current parameter set will not change any of the parameter values currently used, it will be up to the user to re-pick the parameters for the type of analysis. However the default output curve names will change. This could mean the wrong input Vcl or Vsh curve could be used in the PhiSw module."*

**The physical basis IP gives** (page: clayvolume.htm):
> *"Shales normally contain 50-80% of clay. The actual clay contain could change at each depth level and is why shales are usually seen as a cloud of points on crossplot. Shale picks are normally made to the edge of the cloud of points while clay picks are made outside the cloud to give around 60-80% clay content."*

**The picking-convention consequence** — this is the operationally important sentence:
> shale point = **edge of the shale cloud**; clay point = **outside the cloud**, targeting *"around 60-80% clay content"*.

**Why it matters downstream** (page: clayvolume.htm):
> *"It is clearly easier to pick shale points than clay points, however it must be remembered that the porosity analysis modules work volumetrically and can only handle mineral volumes e.g. clay volume. If shale volumes are picked these have to be converted to clay volume before they can be used correctly. The Porosity and Water saturation module allows the input of shale volume (Vsh) and has an addition parameter to convert it into clay volume (Clay Shale Ratio)."*

**The conversion itself is stated in TEXT** on the PhiSw parameter page (page: swparameters.htm — outside
the target set but load-bearing for this terminology question):

> *"(178) Clay Shale Ratio ? the percentage of clay in 100% shale in decimals (v/v). Used to calculate the clay parameters from the shale parameters or vice versa. Also used to calculate the final Vshale output from the VWCL curve. Vshale = VWCL / CSR"*
> *"Rho Wet Clay = Rho Matrix + ((Rho Shale - Rho Matrix) / CSR)"*
> *"Neu Wet Clay = Neu Matrix + ((Neu Shale - Neu Matrix) / CSR)"*
> *"Son Wet Clay = Son Matrix + ((Son Shale - Son Matrix) / CSR)"*
> and from swequationsandmethodology.htm: *"CSR = input Clay Shale Ratio parameter ( The fraction of clay seen in 100% shale )."*

**No default value for CSR is stated on any page read.**

**Design takeaway for SandiBumi:** shale is treated as a *rock type* (a mixture), clay as a *mineral
volume*. The volumetric engine accepts only mineral volumes. The Vsh→Vcl bridge is a single scalar
(CSR) applied both to the endpoint parameters and to the output curve, in opposite directions:
endpoints are divided-into by CSR to move shale→clay, and the output Vshale is `VWCL / CSR`.

---

## 4. Combining indicators — the minimum / average / mixed logic

Four result curves (page: clayvolume.htm):

| Curve | Definition (verbatim) | Clipped 0–1? |
|---|---|---|
| per-indicator outputs | *"An Output curve will be produced for each clay indicator selected."* | **No** |
| **VCL** (minimum) | *"The Minimum Clay Volume ( VCL ) curve is calculated as the minimum clay volume response of all selected single and double clay indicators, and, by default is picked as the Wet Clay Volume ( VWCL ) curve used in the set up of the Porosity and Water Saturation Module."* | Yes |
| **VCLAV** (average) | *"Vclay minimum ( VCL )and Vclay average ( VCLAV ) curves will also be calculated."* | Yes |
| **VCLMIX** | *"The Output curve, VCLMIX utilizes both the VCL Minimum and the VCL Average curves. A parameter is set by zone that determines whether the selected curve is VCL Minimum or the VCL Average Curve."* | (via its source) |

**Clipping discipline — order matters, and IP states it:**
> *"The curve results for these indicators will not be limited to between 0 and 1.0. These curves are meant for quality control purposes, and should not be used in another module for Vclay, unless they are clipped first to between 0 and 1.0. This clipping could be done using the formula module."*
> *"Vclay minimum ( VCL )and Vclay average ( VCLAV ) curves will also be calculated. These curves are clipped to between 0 and 1.0. **For the VCLAV curve, the separate Vclay indicator curves will first be clipped before the average curve is created.**"*

So: **clip each indicator to [0,1] first, then average.** (Emphasis added.) The per-indicator QC curves
themselves stay unclipped on purpose.

**Three VCL Average methods** (page: clayvolume.htm):
> *"There are two new methods for calculating VCL Average. Currently the VCL Average is the mean value of all the VCL results at a level. The two new methods are ?Median? and ?Lateral?. The ?Lateral? average is the median value of all the individual pair products. This is almost identical to the Hodges-Lehmann method."*

VCLMIX selector: *"For VCL Average; Mean, Median or Lateral and for VCL Mix; Minimum or Average."*

The Lateral average is defined more fully, **with a citation**, on `curve_average.htm`:
> *"Lateral averaging is performed by creating pair products of the input data and then taking the median of the results."*
> *"This is almost identical to the Hodges-Lehmann method (J.L Hodges and E. L Lehmann, Basic Concepts of Probability and Statistics, Holden-Day 1970)."*
> *"The Lateral average can give better results since it gives a median-type average that will not be badly skewed by any outliers in the data."*

That is the **only full primary citation the manual gives anywhere in the clay chain** — Tier B, usable.

**Other combination-adjacent rules stated:**
- A per-zone Use flag nulls an indicator over that zone: *"Set to Off for Vclay from gamma ray to be set to Null values over this zone."* (page: clayparameters.htm)
- Nulled indicators drop out of the display and, by implication, the min/average: *"A clay indicator which is not used in a zone will not be displayed in the result track for that zone (its values having been set to Null over that zone)."* (page: clayplot.htm)
- Bad-hole gating nulls **doubles only**, to -999 (page: clayequationsandmethodology.htm).
- Initialisation discipline: *"NOTE: It is good practice to verify any indicator that might possibly be used, BEFORE running a Clay Volume Analysis interpretation, since this will allow IP to initialize the default clay parameters and create the Bad Hole tab parameters ."* (page: clayvolume.htm)

**Parameter-linking rules** (page: clayparameters.htm) — relevant if SandiBumi replicates the shared-endpoint UX:
- `Link Clay Paras`: changing a curve's clay value in one double indicator updates the same curve's clay value in every other double indicator (e.g. `ND Den Clay` ↔ `DS Den Clay`).
- *"NOTE : Setting this parameter to on DOES NOT update Single Clay Indicators"* — doubles and singles are deliberately decoupled.
- `Link Clean Paras`: links the **Clean 1** parameters across doubles (`ND Den Clean 1` ↔ `SD Den Clean 1`); Clean 2 is not mentioned as linked.
- `Link PhiSw Clay`: cross-module link of Density/Neutron/Sonic clay parameters to the PhiSw module, active only when the parameter sets are linked; *"When the Calculate Shale Volume option is on the shale parameters are linked."*

---

## 5. Lithology curve creation & coding scheme

Module: **Edit → Create/Edit Lithology Curve** (page: createeditlithologycurves.htm).

**What a lithology curve is, verbatim:**
> *"Lithology Curves in IP are "flag" type curves with values mapped to Lithology Bitmaps (stored in Lithology.opt)."*
> *"...generates a curve called LITH , which is a flag type curve, whose values are mapped to the lithology bitmaps stored in the IP Shading Bitmaps subdirectory."*

**Conventions stated:**

| Convention | Verbatim | Source |
|---|---|---|
| Default curve name | *"a curve called LITH"* | createeditlithologycurves.htm |
| Curve kind | flag-type (discrete integer index), not continuous | createeditlithologycurves.htm |
| Thin-bed resolution limit | *"Thin beds (down to the well step increment (0.5ft, 0.1524m) can be digitized over the top of a thick interval of a single dominant lithology and the thin bed lithology is inserted into the Flag curve."* | createeditlithologycurves.htm |
| Unknown lithology | *"The <Clear> fill can be used to enter an interval of unknown lithology."* | createeditlithologycurves.htm |
| Suggested working scales | *"e.g.1:500 / 1:200 Scale"* | createeditlithologycurves.htm |
| Optional parallel text curve | *"Output Text Curve Option - When ticked on a text curve will be created. This curve will contain the description of the lithology selected at each depth."* | createeditlithologycurves.htm |
| Header suppression for hardcopy | type the word `None` in the Shading Description column for the LITH track | createeditlithologycurves.htm |
| Legend assets | `Lithology Legend.emf` and `Lithology Legend.ppt` in the `\Shading Bitmaps` sub-directory | createeditlithologycurves.htm |
| Persistence | curve is in memory until *"Save All Wells to Database"* / *"Save Current Well to Database As"* | createeditlithologycurves.htm |

**The coding scheme itself** (page: tools.htm, "Default Lithology"):
> *"The Default Lithology File ( Lithology.opt ), shipped with IP contains 39 bitmap shading files, linked to lithology descriptions. These descriptions are listed in ascending alphabetic order so that the Create / Edit Lithology Curve module is easier to work with."*
> *"NOTE: there is currently an upper limit of 80 lithology shadings permitted in the Edit Default Lithology table."*
> *"NOTE: bitmaps should have dimensions of 16 x 16 pixels."*
> *"The Index numbers are used in the Create / Edit Lithology Curve module. The numbers are used to map a bitmap shading to a curve value in the Lithology curve. ... Index numbers must be unique for each row of the Default Lithology Table."*

Two-level override: IP-directory `Lithology.opt` is the shipped default; a project-directory
`Lithology.opt` overrides it. *"The Project Level Defaults will be used in preference to the IP Defaults, where they exist."* (page: default_settings.htm). Same pattern for `DefaultUnits.opt` and `ShadeTypes.opt`.

**The manual does not enumerate the 39 index→lithology mappings.** They are in the shipped config file,
which is a 3-column CSV (`index,BitmapName,Description`). Read read-only from
`C:\Program Files\IP2018\Lithology.opt` (39 rows, confirming the manual's count):

```
1,ANHYDRITE2,Anhydrite        14,DOL-Sndy,Dolomite - sandy    27,SANDSTONE,Sandstone
2,BASEMENT,Basement           15,GYPSUM2,Gypsum               28,SST-Arg.,Sandstone - arg.
3,BENT,Bentonite              16,HALITE,Halite                 29,SST-Ark.,Sandstone - ark.
4,BREC,Breccia                17,IGNEOUS,Igneous               30,SST-Calc.,Sandstone - calc.
5,CHALK,Chalk                 18,LST,Limestone                 31,SST-carb.,Sandstone - carb.
6,CHLKY_MARL,Chalky Marl      19,LST-Dol,Limestone - Dolomitic 32,SST-Fine,Sandstone - fine
7,CHERT,Chert                 20,LST-Mddy,Limestone - muddy    33,SST-Tuff.,Sandstone - tuff.
8,CLAY,Clay                   21,LST-Sndy,Limestone - sandy    34,SST-Slty,Sandstone - silty
9,CLAYSTONE,Claystone         22,MARL,Marl                     35,SST-Volc.,Sandstone - volc.
10,COAL2,Coal                 23,META,Metamorphic              36,SLTST,Siltstone
11,CONGLOM.,Conglomerate      24,MDST,Mudstone                 37,SHALE,Shale
12,DOLOMITE,Dolomite          25,MDST-Calc.,Mudstone - calc.    38,TUFF,Tuff
13,DOL-mdy,Dolomite - muddy   26,MDST-Tuff,Mudstone - tuff.     39,VOLCANICS,Volcanics
```

Structure worth noting for SandiBumi: **indices are arbitrary and user-editable, ordering is
alphabetical-by-description, and the scheme is a flat list with modifier suffixes** (`- sandy`,
`- muddy`, `- calc.`, `- arg.`, `- silty`) rather than a hierarchy. There is no numeric semantics — index
5 (Chalk) is not "between" 4 and 6 in any petrophysical sense. Any SandiBumi lithology enum must
therefore carry its own stable identity and treat an IP LITH curve as requiring an explicit
index→lithology map imported alongside the curve, or it will silently mis-map on any project that
edited its `Lithology.opt`. The manual warns about exactly this failure inside IP itself:
> *"...the Lithology curve shadings could lose their mappings to the Lithology curve values and the curve will not display correctly in the log plot."* (page: tools.htm)

---

## 6. Organic-shale corrections — fully recoverable as text

The only clay-chain equations written longhand. All from `clayequationsandmethodology.htm`.

### 6.1 Gamma Ray correction
```
GrCorr = Gr_in - TOC_in x Kerogen_Wt%Con x Gr_Kerogen
```
Where: `Gr_in` = input GR curve; `TOC_in` = input TOC curve **in weight percent**;
`Kerogen_Wt%Con` = wt%→volume conversion factor (input parameter, default 2.5);
`Gr_Kerogen` = *"Gr API reading in 100% Kerogen (input parameter)"*.

How to obtain `Gr_Kerogen`, verbatim (page: clayparameters.htm):
> *"This value can be obtained from wells where a spectral gamma ray has been run. Crossplot the difference between Normal Gr and Uranium corrected Gr against volume TOC (need to correct TOC from weight % to volume before crossplotting). Make a linear regression through the data and extrapolate the line to 100% TOC. This will be the Gr Kerogen value."*
> *"If the input Gr curve is a Uranium corrected curve (eg CGR) then set this value to zero."*

### 6.2 Neutron correction
```
TOCvol    = TOC_in x Kerogen_Wt%Con
HvyMinVol = HvyMin_in x HvyMin_Wt%Con
NeuCorr   = (Neu_in - TOCvol x NeuKer ? HvyMinVol x NeuHvy) / (1.0 - TOCvol - HvyMinVol)
```

### 6.3 Density correction
```
TOCvol    = TOC_in x Kerogen_Wt%Con
HvyMinVol = HvyMin_in x HvyMin_Wt%Con
DenCorr   = (Den_in - TOCvol x DenKer ? HvyMinVol x DenHvy) / (1.0 - TOCvol - HvyMinVol)
```

> **CHARACTER-MANGLING WARNING.** The `?` in the numerators of 6.2 and 6.3 is a character that did not
> survive decompilation. Structurally it must be an operator between `TOCvol x NeuKer` and
> `HvyMinVol x NeuHvy`; a minus is the only reading consistent with a two-component volume-stripping
> denominator `(1 - TOCvol - HvyMinVol)`. **It is recorded as `?` and not silently resolved.** Verify
> against `embim`-free source or a live IP run before implementing. The same `?` mojibake appears
> harmlessly elsewhere on these pages standing in for typographic quotes.

### 6.4 Application scope
> *"When the ?Organic Shale Corrections? are turned on from the clay volume setup window the Gamma Ray, Neutron and Density corrected curves will be output. The corrections are made regardless whether the ?Organic Shale Corr. ? parameter is turned on or off for a zone. The ?Organic Shale Corr.? parameter controls what input curves are used for the Gamma Ray, Neutron and Neutron/Density clay corrections."*

So: **correction curves are always produced when the feature is on; the per-zone flag only chooses
whether Vcl consumes the corrected or uncorrected curves.** Corrected Vcl applies to *"Vclay Gamma Ray,
Vclay Neutron and Vclay Neutron Density"* only (page: clayparameters.htm).

**Input-unit auto-detection rule** (page: clayvolume.htm):
> *"If the units are set at '%', 'pec' or 'wt%' then the input curve will be assumed to be in units of percent and will be divided by 100 to get it in units of ?v/v?. All other units will be assumed to be ?v/v?."*

Heavy Mineral input is optional: *"if left blank the volume of heavy mineral will be assume to be zero."*

---

## 7. Basic Log Analysis — scope, curves, equations

**Deliberate scope limits, verbatim** (page: basicloganalysis.htm):
> *"The functionality has been deliberately simplified to perform a type of analysis that could be easily duplicated using a calculator."*
> *"No hydrocarbon or bad-hole corrections are made. No flushed zone Sxo calculations are made."*

**Vclay Method options:** `GammaRay` | `InputCurve` | `None`.
> *"Set to None to make the clay volume zero. The output Vcl curve will be set to 0.0 for all levels."*

**Output curves:** `PHI`, `SwU` (unlimited), `SW` (clipped 0–1), `BVW`, `VCL` (clipped 0–1), `Rwapp`,
plus hidden `Rt_Hingle`.
> *"Sw is set to SwU but is limited to be less than or equal to 1.0."*
> *"For all methods the calculated Porosity is limited to being greater than 0.0."*

**Equations:** Vcl-from-GR `embim14.gif`, density φ `embim15.gif`, Wyllie `embim16/17.gif`, Raymer
`embim18.gif`, Rwapp `embim19.gif`, Archie `embim20.gif`, Simandoux `embim21.gif`, Indonesian
(Poupon-Leveaux) `embim22.gif` — **all raster**. The only text-form equation on the page:
```
Rt_Hingle = Rt^(-1/m)
Vma = 1/Dtmatrix    Vf = 1/Dtfluid    Vlog = 1/Dt
```
Hingle Y-axis, verbatim: *"The Hingle plot Y-axis is built from (1/Rt)^(-1/m) but scaled in resistivity."*
(Note this is stated differently from `Rt_Hingle = Rt^(-1/m)` on the same page — recorded as-is, both verbatim.)

**Neutron/Density crossplot porosity method, verbatim:**
> *"The neutron/density crossplot porosity uses the neutron tool type to generate the standard chart book neutron/density crossplots for sandstone, limestone and dolomite. The neutron and density logs are then corrected for clay and the porosity is calculated from the position the clay corrected data falls on the chart."*

Tool-type dependence is explicit: *"Each different Neutron tool will have a slightly different response to Sandstone and Dolomite compared to Limestone (all Neutron tools are calibrated to read correctly in limestone)."*

---

## 8. Basic Log Functions — text-form equations and citations (Tier A / Tier B)

All from `basiclogcalculations.htm`. Nine tabs; *"Each tab works as a stand-alone operation and the Run
Tab button must be clicked for each page."*

**Equations stated as text (fully recoverable):**
```
U = Pef x (RHOB + 0.1883) x 0.93423        (Volumetric Cross Section)
Ct = 1000 / Rt        Cxo = 1000 / Rxo     (Conductivity)
Caliper = DCal + Bit Size                  (Differential caliper)
Velocity = 1 / DT
HI_Gas = 2.25 * GasDensity                 (NMR gas hydrogen index)
Vma = 1/Dtma   Vf = 1/Dtfl   Vlog = 1/Dt
```
Raster on this page: `embim5`–`embim13` (density φ, φ→RHOB, Wyllie, Hunt-Raymer, U-matrix-apparent,
M & N lithology identifiers, Archie FF, gas T1).

**Primary citations the manual gives — Tier B, harvestable:**

| Topic | Citation verbatim |
|---|---|
| Mud filtrate resistivity (fresh muds) | *"The Lowe and Dunlap equations come from " Estimation of Mud Filtrate Resistivity in Fresh Water Drilling Muds " The Log Analyst (March-April 1986)."* |
| Mud resistivity vs solids | *"The Overton and Lipson equations come from "A Correlation of the Electrical Properties of Drilling Fluids with Solids Content" Transactions AIME (1958)"* |
| Downhole fluid densities | *"...come from the paper by Batzle and Wang " Seismic Properties of Pore Fluids", Geophysics (1992) ."* |
| NMR gas HI and T1 | *"The relationships for Gas HI and T1 are those published in NMR Logging: Principles and Applications by Coates, Xiao and Prammer (1999)."* |
| Lateral / median averaging | *"(J.L Hodges and E. L Lehmann, Basic Concepts of Probability and Statistics, Holden-Day 1970)"* (page: curve_average.htm) |

**Stated validity limits:**
> *"The equations are valid for salinities below 70 Kppm. The Lowe and Dunlap option does not calculate Rmc."*
> Permeability: *"These equations are applicable only over zones which are at irreducible water saturation i.e. hydrocarbon zones above the transition zone."*
> Gas T1: *"calculated assuming 100% Methane, assuming a density relative to air of 0.554. (The Gas Density Relative To Air parameter is not used.)"*

**Vendor chartbook provenance (see Tier-C-adjacent flag, §9):**
> *"The Timur , Morris Biggs oil and Morris Biggs gas defaults come from the Western Atlas chartbook, whilst the Schlumberger Chart K3 is from the Schlumberger chartbook."*
> *"Sigma Water is calculated using the charts in the Schlumberger chart book (Tcor-2a and Tcor-3b)."*
> *"Sigma Oil is calculated from GOR using charts in the Western Atlas chart book (8-6 Rev1 12-95 chart book)."*
> *"Sigma Gas is calculated using the Schlumberger chart for Methane (Tcor-1). If Wet Gas is selected then the methane sigma is corrected as per chart 8-4 in the Western Atlas chart book. If Condensate is selected then the methane sigma is corrected as per chart 8-5 in the Western Atlas chartbook."*

Implementation detail IP leaks about itself: *"?UnitsConversion.pas? contains all the default routines
and defaults needed for conversions."* — confirms a Delphi/Pascal codebase; Tier A market intelligence only.

---

## 9. Tier classification & flags

**Tier A (free to adopt)** — everything in §1 (structure), §2 (defaults), §3 (terminology), §4 (combination
logic), §5 (lithology conventions), §6 (organic-shale corrections, which are simple volume-stripping
algebra IP publishes openly), §7–8 scope and unit conventions.

**Tier B (name + citation only; reimplement from primary source)** — Larionov (older/younger rocks),
Clavier, Stieber, Wyllie, Hunt-Raymer, Archie, Simandoux, Indonesian (Poupon-Leveaux), Timur,
Morris-Biggs, Hodges-Lehmann, Lowe & Dunlap, Overton & Lipson, Batzle & Wang, Coates/Xiao/Prammer.
**Only five of these carry a full citation in the manual** (the table in §8). Larionov, Clavier and
Stieber carry surname-only attributions.

**Tier C (name + evidence only) — new flags from target B:**

1. **No new patented/branded method found in the clay-volume chain.** No trademark, patent number, or
   "proprietary" claim appears on any of the seven target pages. The clay-volume module is
   conventional published petrophysics with a proprietary *implementation*.
2. **Tier-C-adjacent (third-party copyright, not patent) — shipped vendor chartbook digitizations.**
   IP ships the Schlumberger and Western Atlas chartbooks as executable lookup tables: neutron
   tool-type response tables (*"The neutron tool lithology conversions are made using Look-up Tables"*),
   `Tcor-1/2a/3b`, Western Atlas `8-4`, `8-5`, `8-6 Rev1 12-95`, `Chart K3`, and the per-tool `.neu` /
   `.ovl` files visible in the install root. **SandiBumi must not lift these tables**, from IP or from
   scanned chartbooks, without its own licensed or independently digitized source. This is a
   copyright/licensing exposure rather than a patent one, but it is the single largest "looks like
   Tier A, is not" trap in this target area.
3. **`Neu Tool Type` / `Neu Log Cont` per-contractor response model** — the mechanism (select
   contractor → select tool → look up sandstone/dolomite offset from limestone) is Tier A and freely
   describable; the **coefficient tables behind it are item 2 above**.

---

## 10. Gaps (things a reimplementation still needs and this ingest could not supply)

1. **Larionov / Clavier / Stieber / Curved coefficients** — raster, `embim24`–`embim29`. Must come from
   primary literature. This is the top open item.
2. **Full citations for Larionov, Clavier, Stieber** — the manual gives surnames and rock-age scope only.
3. **`Percentile Clean` default** — not stated anywhere. `Percentile Clay` is 130%; its counterpart is
   documented only by an illustrative "10th percentile" example.
4. **`Clip Low %` default is contradicted between two pages** (0% on clayparameters.htm, 98% on
   basicloganalysis.htm). Unresolved — see §2.1.
5. **`Clay Shale Ratio` default** — not stated on any page read, despite CSR being the sole Vsh→Vcl bridge.
6. **`Gr Kerogen` default** — not stated (a derivation method is given instead).
7. **The `?` operator in NeuCorr / DenCorr numerators** — decompilation-mangled character, sign unverified.
8. **Bad-hole `Min`/`Max` initial values** — explicitly uninitialised by design.
9. **The exact null sentinel outside bad-hole gating** — `-999` is stated for bad-hole-nulled doubles;
   whether the same sentinel is used for zone-level Use-flag nulls is not stated.
10. **VCLMIX per-zone selector parameter number** — the parameter exists and is described, but is not in
    the numbered `(1)`–`(69)` MonteCarloDefaults.par list on clayparameters.htm; its Monte-Carlo index is
    unknown. (The list also skips `(51)`–`(54)` and `(56)` entirely in the decompiled text.)
11. **The 39 lithology index mappings are not in the manual** — recovered from the shipped
    `Lithology.opt` (§5), which is the authoritative source and is user-editable per project.

---

*Compiled 2026-08-06. No values inferred, rounded, or supplied from outside the cited pages.*
