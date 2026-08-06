# Target E — Capillary Pressure / Saturation-Height & Rock Typing (IP 2018 help ingest)

Source: decompiled IP 2018 CHM text at `C:\Users\ARUNIKA\AppData\Local\Temp\c18\_text\`.
Original install `C:\Program Files\IP2018` treated as read-only; nothing written there.
Vendor: PGL / Lloyd's Register / Geoactive ("Interactive Petrophysics").

**Pages read (assigned):** `cappressuresetup.txt`, `cappressurefunctions.txt`,
`logswversusheightfunctions.txt`, `saturation_versus_height_curve.txt`,
`saturation_height_modelling.txt`, `capillarypressuredataloader.txt`,
`cluster_analysis.txt`, `multiplelinearregression.txt`.

**Page read in addition (justified):** `hfu.txt` ("Hydraulic Flow Units"). The
Cap-Pressure Set-Up page explicitly hands off to it ("the Pittman equation in the
Hydraulic Flow Unit module", HFU column feeding the Rock Class column), and it is
where IP's Winland / Pittman / Lucia / FZI rock-typing actually lives. It carries the
densest set of primary citations in the whole rock-typing area, so omitting it would
have left the Tier-B citation harvest half-empty.

**Cross-reference only (not extracted here):** `som.txt` (Self-Organising Maps) — a
second, neural clustering route to electrofacies that shares the same five hierarchical
consolidation methods as Cluster Analysis and cites *"Self-Organizing Maps, 3rd Edition"
(pub. Springer) by T. Kohenen* [sic]. Belongs to an ML/statistics target, not this one.

### THE NUMBERS RULE — how it was applied here
Every number below is one the manual states, quoted as written (including IP's own
typos and its `?` mojibake for curly quotes). Nothing rounded, converted, or supplied
from outside knowledge. Where IP says a default exists but the value lives only in a
screenshot, the entry reads **`not stated in manual`** — see §3.1, which is the single
most important gap in this target.

---

## 1. Module architecture (Tier A — free to adopt)

IP splits saturation-height work across **four interlocked modules** plus a loader,
all writing to **one shared parameter file** (`.cap` in the project root, also mirrored
into `IPDBProj.dat`). The split is worth copying because it separates *data
conditioning* from *function fitting* from *field application*:

| Module | Role | Consumes | Produces |
|---|---|---|---|
| Capillary Pressure Data Loader | Spreadsheet → array curves | pasted columns | Pc / PcSat array curves + Phi/Perm |
| Capillary Pressure Set-Up & Corrections | QC + correct + convert lab→reservoir | raw Pc/Sw arrays | `PcCorr`, `SwPcCorr`, `PcUse`, `Pc Height`, closure/entry-pressure, pore-size, Rock Class |
| Capillary Pressure Functions | Fit Sw = f(Pc, φ, K) models | `PcCorr`, `SwPcCorr`, `PcUse` | fitted Model + `SwPcReg` / `SwCpIndReg` / `SwCpReg` |
| Log Sw Vs. Height Functions | Fit Sw = f(h) from **logs only, no core** | PHIE, SW, BVW, (PERM), TVDSS, FWL | fitted Function + `SwLogHt`, `Ht_FWL` |
| Saturation Versus Height Curves | Apply functions field-wide; solve FWL / IFT | any of the above functions | `Sw_Ht`, `Sw_PcHt`, `HtAbCont`, `PcAbCont` |

Key structural ideas an independent implementer should steal:

- **Plug status is a first-class tri-state**: `Good` / `Part Good` / `Bad`. "Part Good"
  means the plug survived but individual points were edited out. Only Good + Part Good
  plugs with `Select Plug` ticked flow into the fitting module.
- **A per-point use flag travels with the array**: `PcUse` is an array curve of the same
  dimensions as Pc and PcSat, holding 1 = valid, 0 = bad. Editing a point on the
  crossplot writes into that array; the flag is then a default discriminator in the
  fitting module. This is a clean, auditable QC contract.
- **Models are versioned containers, not global state.** "Each Model can have a different
  set of plugs", each with its own discriminator logic; output curves get `1,2,3...`
  appended so re-runs never overwrite (`SwPcReg1`, `SwPcReg2`, ...). IP explicitly calls
  out that this "removes the old problem of over-writing the output curve every time".
- **Function Mixing** is a separate table that decides which fitted function applies
  where (by porosity range, facies code, etc.), decoupled from how the function was fitted.
  Grid defaults to **15 lines** per mixing, extensible "up to a maximum of 1000 lines".

---

## 2. Saturation-height / capillary-pressure functions

### 2.1 Master table

`Family` codes: **T** = normalising *transform* applied before fitting (One-Equation-for-all
route); **R** = *regression form* fitted to the data; **M** = *method* (log-height route).
"Rasterized?" = whether the equation is only available as an image / screenshot.

| # | Function | Family | Parameters | Stated defaults | Citation given by manual | Rasterized? | Source page |
|---|---|---|---|---|---|---|---|
| 1 | Leverett J Function | T (Pc route) | Pc, σ, θ, K, ϕ | constant `0.2166`; σ/θ via Set Gas/Water = `0 degrees` / `50 dynes/cm`, Set Oil/Water = `30 degrees` / `30 dynes/cm`; output curve name `Lev_J` | **none** | No — text: `J = 0.2166 * Pc / ( σ * COS θ) * √(Κ/ϕ)` | cappressurefunctions |
| 2 | Leverett J Function, porosity modifier | T (Pc route) | Pc, σ, θ, K, ϕ, m | "The default m value is set at a value of **2.8**"; σ/θ as above; output `Lev_J` | **none** | No — text: `J = 0.2166 * Pc / ( σ * COS θ) * √(Κ/ϕ ^m )` | cappressurefunctions |
| 3 | Porosity & Pc Function 1 | T (Pc route) | Pc, ϕ, fitting constants `a, b, λ, c` | none stated | **none** | **YES** — equation only in dialog screenshot | cappressurefunctions |
| 4 | Porosity & Pc function 2 | T (Pc route) | Pc, ϕ, `a, b, λ, c` | none stated | **none** | **YES** — screenshot only | cappressurefunctions |
| 5 | Porosity & Pc function 3 | T (Pc route) | Pc, ϕ, `a, b, λ, c` | none stated | **none** | **YES** — screenshot only | cappressurefunctions |
| 6 | Porosity & Pc Lambda Function | T (Pc route) | Pc, ϕ, `a, b, λ, c` | none stated | **none** | **YES** — screenshot only | cappressurefunctions |
| 7 | Thomeer Function | T and R | Pc, BVnw, ϕ, G, Pd | none stated | **none** | No (but glyph loss) — text: `Sw = 1.0 - (BVnw / ) e^( -G / Log10(Pc/Pd) )` | cappressurefunctions |
| 8 | Lambda (on J) | R | J, `a, λ, b` | none stated | **none** | No — `Sw = a * J^(- λ ) + b` | cappressurefunctions |
| 9 | Hyperbola (on J) | R | J, `a, b, c` | none stated | **none** | No — `Sw = a / (J-b) + c` | cappressurefunctions |
| 10 | Exponential (on J) | R | J, `a, b, c` | none stated | **none** | No — `Sw = a * e^(b*J) + c` | cappressurefunctions |
| 11 | Lambda (per-plug, on Pc) | R | Pc, `a, λ, b` | none stated | **none** | No — `Sw = a * Pc^(- λ ) + b` | cappressurefunctions |
| 12 | Hyperbola (per-plug) | R | Pc, `a, b, c` | none stated | **none** | No — `Sw = a / (Pc-b) + c` | cappressurefunctions |
| 13 | Exponential (per-plug) | R | Pc, `a, b, c` | none stated | **none** | No — `Sw = a * e^(b * Pc) + c` | cappressurefunctions |
| 14 | **Brooks Corey** | R | Pc, Pd, λ, Swi | none stated | **none** | Dual: text `Sw = (Pc / Pd)^(- λ ) * (1-Swi)+Swi`; also `[[EQUATION_IMAGE: embim460.gif]]` | cappressurefunctions |
| 15 | **Skelt Harrison** | R | Pc, `a, b, c, d` | none stated | **none** | No — `Sw = a*e^( -(b/(Pc+d))^c)` | cappressurefunctions |
| 16 | User-Defined (Pc route) | R | any; up to 7 curves + 7 coefficients | keywords `Pc, Phi, Perm` fixed + 4 optional curve slots; per-plug route allows "up to 4 coefficients which can be regressed" | n/a | No | cappressurefunctions |
| 17 | Sw function of height | M (log route) | Sw, h | none stated | **none** | No — `Sw = f(h)` | logswversusheightfunctions |
| 18 | BVW function of height | M (log route) | BVW, h | none stated | **none** | No — `BVW = f(h)` | logswversusheightfunctions |
| 19 | Rock Quality Index (log route) | M (log route) | K, ϕ, h | none stated | **none** | No — `Sw = f(RQI.h) & RQI = √ ( Κ / ϕ )` | logswversusheightfunctions |
| 20 | Rock Quality Index, porosity modifier | M (log route) | K, ϕ, h, m | none stated (no default m given on this page) | **none** | No — `Sw = f(RQI.h) & RQI = √ ( Κ / ϕ ^m)` | logswversusheightfunctions |
| 21 | Porosity & Height function 1 | M (log route) | h, ϕ, `a, b, c` | none stated | **none** | No — `Sw = 1.0 / [(a +h^b). ϕ ^c]` | logswversusheightfunctions |
| 22 | Porosity & Height function 2 | M (log route) | h, ϕ, `a, b, c, d` | none stated | **none** | No — `Sw = 1.0 / [(a +b.h^c)). ϕ ^d]` *(stray `)` is verbatim)* | logswversusheightfunctions |
| 23 | Porosity & Height function 3 | M (log route) | h, ϕ, `a, b, c, d, f` | none stated | **none** | No — `Sw = [a + b.Log(h) + c.Log(h)^2 + d. Log(h)^3 ] / [ ϕ ^f ]` | logswversusheightfunctions |
| 24 | Porosity & Height Lambda function | M (log route) | h, ϕ, `a, b, c, d` | none stated | **none** | No — `Sw = a.h^(b. ϕ + c) + d` | logswversusheightfunctions |
| 25 | Linear (log route) | R | h, `a, b` | none stated | **none** | No — `Sw = a + b.h` | logswversusheightfunctions |
| 26 | Linear / Log | R | h, `a, b` | none stated | **none** | No — `Sw = a + b.Log(h)` | logswversusheightfunctions |
| 27 | Log / Linear | R | h, `a, b` | none stated | **none** | No — `Log(Sw) = a + b.h` | logswversusheightfunctions |
| 28 | Log / Log | R | h, `a, b` | none stated | **none** | No — `Log(Sw) = a + b.Log(h)` | logswversusheightfunctions |
| 29 | Lambda (log route) | R | h, `a, λ, c` | none stated | **none** | No — `Sw = a.(h)^(- λ ) + c` | logswversusheightfunctions |
| 30 | Hyperbola (log route) | R | h, `a, b, c` | none stated | **none** | No — `Sw = a / (h - b) + c` | logswversusheightfunctions |
| 31 | Exponential (log route) | R | h, `a, b, c` | none stated | **none** | No — `Sw = a.e^(b.h) + c` | logswversusheightfunctions |
| 32 | User Defined Equation (log route) | R | any; up to 7 curves + 7 coefficients | keywords `Ht, Phi, Perm` fixed + 4 optional | n/a | No | logswversusheightfunctions |

**Headline finding: not one saturation-height function in IP 2018 carries a primary
citation.** Leverett J, Thomeer, Brooks-Corey, Skelt-Harrison, Lambda — all are presented
as bare algebra. Every citation IP does give in this subject area is in the *conditioning*
and *rock-typing* code, never in the SHF code (see §3.4 and §4.2). For an independent
implementer this cuts both ways: the algebra is disclosed and freely reimplementable, but
IP is no help at all in tracing the science, so the primary papers must be sourced
elsewhere. **Cuddy (FOIL / BVW = a·H^b) is never named** — IP's `BVW function of height`
method is the same idea, presented uncredited and unparameterised.

### 2.2 The three model **Types** (the real design decision)

`Type` chooses how curves are aggregated, and it changes the whole Equations tab:

1. **One Equation for all Pc curves** — normalise every plug with a transform (J-function
   or a porosity/Pc function), then fit **one** regression through the whole cloud.
   *This is the per-rock-type route*: "a standard scenario would be to create separate
   models that can be applied over different porosity ranges. i.e. a model that works in
   the 10-15% porosity range, a second model for the 15-20% porosity range and a third
   for the 20-25% porosity range." Rock typing here is expressed via **Discriminators +
   one Model per rock type**, not via a rock-type argument inside the function.
2. **Separate equation for each Pc curve** — fit each plug independently, then build a
   **Combined Equation** by correlating each fitted coefficient against a curve
   (typically core φ or K). This is the coefficient-regression route and is where IP is
   genuinely strong (see §2.4).
3. **User Defined Equation** — arbitrary formula in `User Formula` syntax, up to 7 curve
   keywords and 7 coefficients, savable/loadable as a `.UDE` for transfer between projects.

Note in §2.2's Thomeer entry that Thomeer appears in **both** Type 1 and Type 2 lists.
Also note: "When analyzing the results of applying one of these four functions, using the
crossplot tool, **no Regression line function is created**" — i.e. the Porosity & Pc
functions 1/2/3 + Lambda are fitted directly, not via the Lambda/Hyperbola/Exponential
secondary fit that J-function uses.

### 2.3 Fitting mechanics (all Tier A, all worth copying)

- **Regression weighting.** "the points that go into the regression are weighted by the
  reciprocal of the Sw value. The Sw value will be in decimal units and the **maximum
  weighting factor will be 10**. Therefore an Sw of **1.0** will have a weight of **1**
  and an Sw of **0.1** will have a weight of **10**." Effect: weight toward low Sw.
  Identical wording on the log-height page. This is a stated, reproducible spec —
  adopt it verbatim, including the clamp at 10.
- **Non-linear regression is iterative and start-value sensitive.** "It is possible that
  several regression results, equally good, could be found for the same function. The
  starting coefficients will be what decide which solution is found." IP exposes start
  values, a cancel button, and advises "Some equations will work with the starting
  coefficients all set to zero. Others will just fail."
- **Fixed coefficients** — any coefficient can be pinned (`Fix Coeff`) and excluded from
  the regression; a fixed coefficient may itself be *an equation* (a correlation against
  another curve), not just a constant. IP's recommended workflow is explicitly staged:
  run all free → find the coefficient with the best correlation → fix it → re-run → repeat.
  Caveat stated: "fixing a coefficient to a value which is a long way away from the free
  regression results can result in a very poor fit."
- **Regression Function Comparator** — brute-forces every model and ranks by R². On the
  log-height page the manual states the search space size: "**there are 32 models**".
  That is exactly (4 base Methods × 7 Regression Equations) + 4 Porosity-&-Height
  functions = 32, which independently confirms the method/regression inventory in §2.1
  is complete for the log route.
- **View Function / Function XPlot** — plots the fitted function as families of constant-φ
  or constant-K lines over a *user-chosen* range, explicitly so the user can see
  extrapolation behaviour: "if the function was generated using capillary pressure curves
  which came from plugs with porosities between 0.15 and 0.2 but the reservoir has
  porosities between 0.05 and 0.25 then the crossplot will allow the user to see how the
  function works at the low and high porosities." 3D surface mode available. **This
  extrapolation-preview is the single best UX idea in the whole target** and is cheap
  to build.

### 2.4 Coefficient-correlation types (Type-2 route)

Correlation Type options for mapping a fitted coefficient (Y) onto a curve (X):

| Option | Meaning as stated |
|---|---|
| `y = Av. value` | coefficient = Average Value across plugs — **"This is the default setting when the module is first opened."** |
| `y = Median value` | coefficient = Median Value across plugs |
| `y = f(x)` | "linear regression where Coefficient (Y) is a function of the selected Correlation Curve (X). The squared Y-error distances are minimized." |
| `x = f(y)` | "Linear regression where Correlation Curve (X) is a function of the selected Coefficient (Y). The squared X-errors are minimized." |
| `RMA` | "The RMA (reduced major axis) line-fit gives an equation that is midway between the above methods." |
| `2nd Order Poly` | Second Order Polynomial regression equation |
| `3rd Order Poly` | Third Order Polynomial regression equation |

Plus `Log(X)` / `Log(Y)` toggles, an interactive polygon (`Create Area` →
`Re-run Regression using Area`) to exclude outliers, and a draggable regression line
whose new equation is written back into the model. Combined-equation stats reported per
coefficient: Average, Median, Min, Max, Standard Deviation.

**Worked example from the manual's own report (example output, NOT a default):**

```
Model (1) : Lev J Phi < .15      Regression Type : Lambda
Regression Equation : Sw = a.Pc^(-?) + b        Number of Pc curves : 3
Sw Weighted regression used : No
W(8) Depth:4400.19  Yes 0.2885 -1.9828 0.1932 0.998 41
W(8) Depth:4403.23  Yes 0.6839 -2.3015 0.1945 0.995 31
W(8) Depth:4403.39  Yes 0.3272 -1.067  0.1733 0.981 47
Average 0.43323 -1.78377 0.18698 0.99119 | Median 0.32724 -1.9828 0.19318 0.99451
Min 0.28852 -2.30148 0.17326 0.98145   | Max 0.68392 -1.06704 0.19449 0.9976
Std Deviation 0.218 0.6408 0.0119 0.0086
Result equation: Sw = (-999. - 999. * RawC:Phi) * Pc^(-1.78377) + 0.18698
Discriminators: RawC:Phi < 0.15 and SwPcCorr < 0.9
```

Two things to note: the discriminator `SwPcCorr < 0.9` (see §5), and that the printed
Result equation carries **`-999.`** null values where the RMA fit failed to populate —
a real IP defect visible in its own documentation, and a reminder to gate on
"coefficients are finite" before shipping a function.

---

## 3. The lab-to-reservoir conversion chain

### 3.1 IFT and contact angle — **the numbers that get mis-transcribed**

IP holds a **Reservoir and Laboratory fluid / rock properties table** with four columns
(Contact Angle Laboratory, Interfacial Ten. Laboratory, Contact Angle Reservoir,
Interfacial Ten. Reservoir) × three rows (**Mercury Injection**, **Centrifuge**,
**Porous Plate**), plus a `Method ID` column for mapping a numeric measurement-method
flag curve onto those three rows.

**Reservoir-side defaults — stated verbatim in the manual:**

| Trigger | Contact Angle | Interfacial Tension | Verbatim source sentence |
|---|---|---|---|
| `Set Gas / Water` button | `0 degrees` | `50 dynes/cm` | "Clicking Set Gas / Water returns Contact Angle Reservoir 0 degrees and Interfacial Tension Reservoir 50 dynes/cm." |
| `Set Oil / Water` button | `30 degrees` | `30 dynes/cm` | "Clicking Set Oil / Water returns Contact Angle Reservoir 30 degrees and Interfacial Tension Reservoir 30 dynes/cm." |
| `Use Dry Rock Reservoir Properties` tickbox | `0 deg` | `1 dynes/cm` | "This sets the reservoir Contact Angle to 0 deg and the Interfacial Tension to 1 dynes/cm." |

The same Set Gas/Water and Set Oil/Water buttons reappear inside the Leverett-J and
Leverett-J-porosity-modifier dialogs, and again in Saturation-Versus-Height as
`Set Pc Fluid Oil/Water` / `Set Pc fluid Gas/Water`. **Same two default pairs, three
places.** Units are stated explicitly: "Contact angles are entered in units of degrees
and IFT in dynes/cm."

**Laboratory-side defaults: `not stated in manual`.** The manual says only *"Default
values for Interfacial Tension and Contact Angle parameters are provided. However, the
table can be modified."* and documents a `Restore Defaults` button that "will reset the
values in the first 3 columns of the table". The actual per-method numbers
(air/mercury, air/brine, oil/brine) exist **only inside a screenshot** — Tier D, not
recoverable from text. **Do not fill these from memory.** They are precisely the
values that get silently mis-transcribed, and IP 2018's help does not state them.

### 3.2 Lab → reservoir Pc conversion

Governed by the `Convert to Reservoir` flag per well row. The manual describes the
operation in words but the formula itself is an image:

> "The conversion uses the following equation, multiplying the known Pc values by the
> **absolute ratio of the Laboratory and Reservoir σ \* Cos θ values**"

followed by definitions of `Pc, Res`, `Pc, Lab`, `σ Lab`, `θ Lab`, `σ Res`, `θ Res` —
the equation between them is **rasterized / not recoverable**. The *worked example*
elsewhere in the suite discloses the same algebra numerically and IS quotable:

> "if the function was developed for oil with an IFT value of **30** and a contact angle
> of **30 degrees**, then used in gas with an IFT of **50** and a contact angle of
> **0 degrees**. The IFTCorrFactorGas would be equal to **50 x Cos(0.0) / (30 x Cos(30.0) = 1.924**"

(Note the unbalanced parenthesis is verbatim.) That worked example is the reliable
statement of the σcosθ ratio convention, and it also confirms the Set-Gas/Water and
Set-Oil/Water defaults are the intended pairs.

### 3.3 Mercury / non-wetting-phase handling

`Pc Sat Type in` is a per-well two-state switch:

- `Water Wet` — "should be set when the measurement technique reports saturation as the
  wetting phase saturation."
- `Non Wet` — mercury-style; IP converts with `SwPc = 1 - (Snw)`.

Accepted input units — Pc: `psi, Bar, Kg/M 2, Mpa` (IP "converts Pc pressure values to
psi based on the input units"); saturation and porosity: `%` or `dec` (converted to
decimals); permeability: `mD` or `m 2` (converted to mD). **Default output units: "psi
for corrected Capillary Pressure curves and decimals for Corrected Pc Saturation curves."**

### 3.4 Stress correction and clay-bound-water correction

**Stress correction** (flag `Stress Correct`, driven by `PhiRes/PhiLab Factor`, decimal):

```
PcCorr   = Pc * ( PhiRes/PhiLab Factor ) ^ (-0.5)
SwPcCorr = 1- [(1- SwPc ) * ( PhiRes/PhiLab Factor )]
```

Example value used in the manual's screenshot narrative: **`0.9562`** (example, not a default).

**Clay-bound-water correction** (flag `Clay Correct`, requires a `Qv meq/ml` curve and a
`Salinity (Kppm NaCl)` value). **This is the one correction IP actually cites:**

> "based on the method described by **Hill, Shirley and Klein 1979 ( SPWLA 20th annual
> Symposium Paper AA - "The Central Role of Qv and Formation Water Salinity in the
> Evaluation of Shaley Formations")**"

```
PcCorr   = Pc * ( F ) ^ (-0.5)
SwPcCorr = 1- (1 - SwPc ) * F
F = 1 - [0.6425 * ( Salinity ^ (-0.5) + 0.22 ] * Qv ]
```

where `Salinity = formation water salinity (Kppm NaCl equivalent)` and
`Qv = Cation exchange Capacity per total pore volume (meq/ml)`.

⚠ **The `F` line is quoted exactly as the decompiled text gives it and its brackets are
unbalanced** (three `]`, one `[`, one unmatched `(`). Do not "fix" it by pattern-matching
to a remembered form — verify against the rendered help page or the Hill-Shirley-Klein
paper before implementing. Note also the units trap already on record for this project:
Qv here is **meq/ml**, and salinity is **Kppm**, not ppm. Example in the manual's
narrative: "Well 6 is to be Clay Corrected using an input Qv curve and a Salinity of
**30Kppm NaCl**" (example, not a default).

**Closure correction** (flag `Closure Correct`): `SwCorr = Sw (input) + SwClosure`, with
the corrected Sw "limited to be between 0 and 100% Sw". Paired with an **Entry Pressure
(Displacement Pressure)** pick: the Pc at the 100% Sw point is forced to the entered entry
pressure, and "If after corrections there are multiple Sw values of 100% then all but the
last one will be set to a Null Value, this insures that the curve fitting function used in
the Cap Pressure Function module are not affected by multiple 100% Sw values with
different Pc value." Manual's worked table (`Closure Correction = 5 %`,
`Entry Pressure = 3.5 psi`):

| Pc Raw (psi) | Sw Raw (%) | Pc Corrected (psi) | Sw Corrected (%) |
|---|---|---|---|
| 0 | 100 | Null Value | Null Value |
| 1 | 96 | 3.5 | 100 |
| 4 | 90 | 4 | 95 |
| 8 | 85 | 8 | 90 |
| 15 | 70 | 15 | 75 |
| 25 | 60 | 25 | 65 |

Closure picking has **Automatic** mode (click between two raw points; IP extrapolates
lines through the two points left and the two points right, intersection = closure point)
and **Manual** mode (drag the red star).

### 3.5 Correction execution order (binding — copy this exactly)

> 1. Convert input Pc Saturation data to Non-wetting phase saturations.
> 2. Closure correct the input saturation values.
> 3. Perform the Stress Correction, if selected.
> 4. Apply the Clay-bound-water correction, if selected.
> 5. Convert saturation data to Wetting phase saturations.
> 6. Perform the Laboratory to reservoir Pc Corrections, if selected.

Note that stress and clay corrections are applied **in non-wetting-phase saturation
space**, with the flip back to wetting phase happening at step 5 — before, not after,
the lab→reservoir conversion. Getting this order wrong silently changes every fitted
coefficient.

### 3.6 Pc ↔ height, FWL and GOC

**In Cap-Pressure Set-Up** (`Pc Height Curve Out`):

```
Height = Pc / 0.433 (ρWater - ρHc)
```
> "h = height above FWL (in ft); Pc = is the Hc / Brine capillary pressure (in psi);
> ρWater and ρHc are the specific gravities of brine and hydrocarbon at reservoir
> conditions; **0.433 psi/ft is the gradient of pure water at ambient conditions**: takes
> care of the g term in the normal P=hρg equations."

The manual carries a full derivation of why `g` is absent (gradient form
`G = 0.433 * SG`), stating "The gradient of fresh water is 0.433 psi/ft for a density of
1 g/cc and standard gravity g." Output "Height will be in the same units as the database
depth curve."

**In Saturation Versus Height** (forward application), the same relation with an IFT
correction factor:

```
Oil:            Pc = h * 0.433 (ρWater - ρHc) * IFTCorrFactor
Gas over oil, above the GOC:
   Pc = (FWL-GOC) * 0.433 (ρWater - ρHc) * IFTCorrFactor
      + (h - (FWL-GOC)) * 0.433 (ρWater - ρGas) * IFTCorrFactorGas
```

⚠ **Internal inconsistency to be aware of:** in the *Function XPlot* format panels of
both function modules the same conversion is written **without** the 0.433 constant —
"Height above FWL = Pc / (Water Density ? Hydrocarbon Density)". Two different statements
of the same conversion in one product. Quoted here verbatim from both places; treat the
0.433 form as the computational one (it is the one with the derivation and the one used
in the Run All Wells path).

**FWL vs contacts — how IP frames it:**
- FWL is entered as **`TVD Free Water Level` — "True Vertical Subsea Depth value"** per well.
  There is no OWC input at all; IP is FWL-only, and OWC is never modelled as a separate
  parameter. The transition zone is whatever the function produces above the FWL.
- `Oil Wet` tickbox: "When checked, allows Sw_Ht values to be calculated below the FWL.
  If un-checked, everything below the FWL is set to wet." — that is the only below-FWL
  behaviour switch.
- `Well Group` — an integer group id linking FWLs across wells so one drag updates a whole
  fault block: "eg group s 1,2,3 may represent different fault blocks with different FWLs."
- `TVD Gas Oil Contact` — "only required for calculations using Pc models and for
  Gas-over-Oil reservoirs, otherwise leave blank."
- `Hyd Density` / `Gas Density` "Can be entered as a fixed value or as a curve. This allows
  compositional hydrocarbon densities to be used. If a curve is entered the capillary
  pressure at a level is calculated by calculating the **thickness weighted average of the
  input density curve from this level to the contact** (either FWL or GOC). This allows for
  high angle wells where the hole angle may be varying over the interval of calculation."
- Default `MD Calc. Bot Depth`: "If not entered will be the bottom of the well **or the
  FWL, which ever is shallower**."
- `PcAbCont` output "does not include any IFT correction factor" — the raw Pc-above-contact
  curve is deliberately un-corrected, while the Pc actually fed to the functions is.

**Solvers.** Two brute-force search panels, both scoring on cumulative difference and
both defaulting to porosity-weighted comparison:
- **Fluid Contact Fitting** — scan FWL over a depth range; "The Cumulative Difference value
  is the sum of the differences of the hydrocarbon volumes (or Sws) for each depth step
  above the current FWL, for all selected wells. The lowest values will be the Most Likely
  FWL." `Use Hydrocarbon volumes` button "is selected (default)".
- **IFT Fitting** — scan the IFT correction factor: **"The range is logarithmic and the
  defaults of `0.1` to `5` are normally adequate."** Same cumulative-difference scoring.
  Rationale stated: "one of the uncertainties is the IFT value and contact angle for the
  fluids in the reservoir. Sometimes it is felt that the log interpretation Sw values give
  better absolute values of water saturation due to these uncertainties." The IFT Corr
  Factor is also draggable as a live line in track 4 of the interactive plot.

### 3.7 Pore-throat / derived array outputs

- `Pore Size Curve Out` — "an array curve of X dimension of **80**. Each X value represents
  a pore size. The first X value is a pore size of **0.01 microns** and the 80th X value is
  **100 microns**. The scale is logarithmic with **20 X values per decade**."
- `Throat Size` — "an array curve of **80** elements always containing the same values. The
  first value is **0.1** (microns) last value **100** (microns) the other values are
  logarithmically spaced between the two end points."
  ⚠ **These two contradict each other** on the low end (0.01 µm vs 0.1 µm) while both claim
  80 elements to 100 µm. 20/decade × 80 elements spans 4 decades = 0.01→100, so the
  `Throat Size` 0.1 is likely the typo — but the manual states both, so both are recorded
  and neither is corrected here.
- `Pc Normalized Curve Out` — "X cell dimension of **51**. Each cell represents **2.0
  saturation units**. The value in the cell is the pressure value. The transformation is
  approximate due to the coarseness of the cells."
- `Pc Sat / Pc` — "used to make an ?Apex? xplot. It is the slope of the individual PC v. Sw
  points."
- Pore radius in the interactive plot (track 7): **`R = 2* IFT * Cos (CA) /Pc`**, where
  "IFT = Interfacial tension **at lab conditions**" and "CA = Contact angle **at lab
  conditions**". Note the lab-conditions qualifier — the pore-size track is deliberately
  computed pre-conversion.

---

## 4. Rock typing

### 4.1 Cluster Analysis for Rock Typing (`cluster_analysis.txt`) — **Tier A, entirely standard**

**No branded or proprietary clustering method.** IP uses textbook K-means + textbook
agglomerative hierarchical clustering, with one vendor-original selection heuristic
(§4.1.5). The only literature pointer given is a reading suggestion, not a method
citation: *"For more information on Cluster Analysis techniques a good starting point is
Multivariate Pattern Recognition and Classification Methods, Geological Log Analysis Using
Computer Methods, **J.H Doveton**."*

**4.1.1 Two-stage architecture** (this is the design to copy):
> "Firstly, the data is divided up into manageable data clusters... **15 to 20 clusters
> would appear to be a reasonable number for most data sets**. The second step, which is
> more manual, is to take these 15 to 20 clusters and group them into a manageable number
> of geological facies. This may involve reducing the data to **4 to 5 clusters**."

**4.1.2 Input handling.**
- "You can use up to **eight input curves** per well."
- Per-curve `Log` flag: "if selected will take the base 10 logarithm of the curve before
  using it in the cluster analysis. For curve data which is in logarithmic form (such as
  core permeability) this can greatly improve the accuracy of the predicted results."
- `Default Name` row decouples curve *type* from per-well curve *name*: "Curve names can
  vary from well to well", and a newly added well auto-selects curves matching the defaults.
- Separate `Use Well for Model Build` and `Use Well for Model Run` flags — build on one
  well set, apply to another.
- Two discriminator curve slots (`Discriminator Crv1`, `Discriminator Crv2`).
- Optional `Calibration Curve` row (the "yellow line") — a facies curve, **may be a Text
  curve**, used only to label clusters; "This curve is only used for the calibration and
  does not affect the K-Mean results." Constraint stated: "the calibration curve **cannot
  be a continuously variable curve** like core permeability."

**4.1.3 Normalisation — stated exactly:**
> "All input log data is normalized (standardized) before starting so that each input log
> has the same dynamic range. The normalization is done by calculating the mean and
> standard deviation of the log and then normalizing the data by subtracting the mean and
> dividing by the standard deviation. Hence a normalized log data value of 1.0 or -1.0
> will be one standard deviation."

Plain z-score standardisation, applied to every input, unconditionally. Reported cluster
statistics are back-transformed: "`Mean`: This is the mean value of the log for the
cluster **in units of the input log (i.e. un-normalized)**", while "`Cluster Spread`:
...(units are standard deviation of the original data)".

**4.1.4 Seeding — PCA, not random:**
> "The `Seed Clusters` button seeds the grid by performing a **principal component
> analysis** on the input data. The results are sorted and the data is divided up equally
> into the required number of clusters using the `Number of Clusters` box. The input data
> in each cluster is then averaged to give the mean seed points."

Rationale given: "the first principal component log... will normally contain most of the
variation in the data and hence is an ideal way to seed the data to give maximum coverage."
K-means itself is described conventionally (minimise within-cluster sum of squares,
iterate until means stop changing). Known failure mode documented: *"One or more of the
clusters had zero data points! Try re-running the clustering or changing the default seed
points"* — with the advice to simply re-run.

A **sort index** control lets the user pick an input curve whose cluster means order the
output cluster numbers (with a `Reverse sort`), "so that the output cluster numbers have
some sort of geological sense i.e. the high cluster values can be set to be shales and low
cluster number to be sands."

**4.1.5 Consolidation — five hierarchical linkages, verbatim:**

| Method | Manual's definition (Z = merge of A and B; distance to C) |
|---|---|
| Minimum distance between all objects in clusters | "the minimum of the distances (A to C, B to C)" |
| Maximum distance between all objects in clusters | "the maximum of the distances (A to C, B to C)" |
| Average distance between merged clusters | "the average distance of all objects that would be within the cluster formed by merging clusters and C" |
| Average distance between all objects in clusters | "the average distance of objects within cluster Z to objects within cluster C" |
| **Minimize the within-cluster sum of squares distance** | "clusters are formed so as to minimize the increase in the within-cluster sums of squares. The distance between two clusters is the increase in these sums of squares if the two clusters were merged." |

> **"The default method Minimize the within-cluster sum of squares distance gives good
> results for separating out the different log lithologies into different clusters."**

Behavioural guidance given: "Minimum distance... will yield **long thin clusters** while
Maximum distance... will yield clusters that are **more spherical**. Average distance
between merged clusters and Minimize the within-cluster sum of squares distance tend to
yield clusters that are similar to those obtained with Average distance between all
objects in clusters."

**Cluster-count selection.** Two aids: a **dendrogram** (cut at N groups; groups coloured,
above-cutoff merges in black; branch numbers give merge order) and a **Cluster Randomness
Plot**, IP's own heuristic, disclosed in full:

```
Av. Thickness     = Number of depth levels / Number of cluster layers
Random Thickness  = Σ p i / (1 ? p i )            [ ? is the manual's mojibake for a minus sign ]
Randomness index  = Av. Thickness / Random Thickness
```
> "Where p i is the proportion of depth levels assigned to the i th cluster... A value of
> **1** would be totally random, higher values less random... The plot is interpreted by
> picking the number of clusters that are **least random (highest peaks)**. In the above
> example, a cluster grouping of **6** or perhaps **10** would seem to give the most likely
> information."

This metric carries **no citation**. It is a vertical-persistence / bed-thickness measure —
clusters that produce thick coherent layers rather than depth-by-depth noise score high.
It is fully disclosed and cheap to implement, but an independent implementer should treat
it as a vendor heuristic and validate it, not assume a published pedigree.

**4.1.6 Calibration to external facies** — distance-weighted voting, disclosed exactly:
> "For each input calibration data point the multi-dimensional distance from the
> calibration point to each K-Mean cluster point is calculated. The value of the
> calibration curve is then stored at each cluster with a **weighting factor which is the
> inverse of the square of the distance** of the calibration point to this cluster point.
> Once all the calibration data has been processed the weighted average for each
> calibration facies for each K-Mean cluster is calculated. For each K-Mean cluster the
> facies with the **highest weighting** is considered the most likely result."

QC reading rule given: "Clusters which show mostly one facies can be considered well
calibrated. Clusters which show several facies with the same percentages would indicate a
poorly calibrated facies." The chosen facies can be manually overridden per cluster.

**4.1.7 Outputs and limits.**
- "there is a **maximum number of User sets which is seven**".
- Optional parallel **text curve** per facies curve; default name extension **`_T`**;
  default text values "?Facies ? followed by the facies number".
- `Cluster Distance` output curve.
- `Fit` flag curve when a calibration set exists: "value **1.0** when the input calibration
  curve is the same as the output facies curve and **0.0** when different", named from
  `Calibration Fit Base Curve Name` + `_1`, `_2`, `_3`...
- **Contingency table** (counts or percentages, rows/columns swappable, with
  Calibration count / Result count / Match count histograms, and a choice of whether
  calibration or result is the percentage reference: "Changing this option can affect the
  table and graphics by a considerable amount").
- Manual's example run: "four wells... K-mean clustering has been performed with an initial
  **15** clusters - the `L_FaciesAll`. The `L_Facies` used the same Clustering Method but
  was restricted to **11** cluster groups. The `L_Facies2` used **8** cluster groups and
  the `L_Facies3` used **5** cluster groups." (example, not a default)
- Colour caveat: "users should not use colors with shading, for example bitmaps as the
  crossplot module will not be able to recognize the bitmap and will display an
  alternative color."

### 4.2 Hydraulic Flow Units (`hfu.txt`) — **where all the Tier-B citations live**

Four rock-typing methods, all φ–K based, all independent of each other and comparable:

**(a) Rock Quality Index / FZI**
```
RQI  = 0.0314 x Sqrt( K / Phi )
PhiZ = Phi / (1 ? Phi)                     [ "Pore-Grain volume ratio" ]
FZI  = RQI / PhiZ
```
> **Citation given:** "This methodology is described well in **SPE paper 26436 ?Enhanced
> Reservoir Description: Using Core and Log Data to Identify Hydraulic (Flow) units and
> Predict Permeability in Uncored Intervals/Wells?**"

(IP gives the SPE number and title only — **no author names**; it does not print
"Amaefule". Recorded as stated.) Theory statement given: "FZI will be a constant for a
flow unit. A log/log plot of RQI against Phiz will show data in the same flow unit as
values on a straight line with **unit slope**. The FZI of the flow unit will be the point
on the line when Phiz equal **1**." The intermediate Kozeny-Carmen rearrangement steps
("The basic Kozeny-Carmen relationship is given as", "Where K = permeability in µm 2",
"Where K is in md") are **rasterized — the equations are missing from the text stream.**

**(b) Winland R35**
```
WinR35 = ALog( 0.732 + 0.588 Log(K) ? 0.864 Log(Phi) )      where Phi is in percent
Log (R35) = 0.732 + 0.588 Log(Kair) ? 0.864 Log(Phicore)
```
> **Attribution given (no formal paper):** "The Winland equation was created by **Dale
> Winland of Amoco**. It is an empirical equation where R35 is the pore aperture radius
> corresponding to the **35th percentile** of mercury saturation in a mercury porosimetry
> test, Kair is the **uncorrected air permeability (in md)** and Phicore is porosity **in %**."
> "originally defined from mercury porosimetry measurements on some **300 samples** from
> the **Spindle Field in Colorado**."
> Net-pay convention: "Winland used an R35 value of **0.5μm** as the definition of net pay
> for the Spindle Field due to evidence he had seen of dry wells having an R35 of
> **<0.5μm** and producing wells with **R35>0.5μm**. The value of 0.5μm has since been used
> in other reservoirs to define pay."

**(c) Pittman — all 14 equations, verbatim** (K in mD, Phi **in percent**):
```
Log(R10) = 0.459 + 0.500 Log(K) - 0.385 Log(Phi)
Log(R15) = 0.333 + 0.509 Log(K) - 0.344 Log(Phi)
Log(R20) = 0.218 + 0.519 Log(K) - 0.303 Log(Phi)
Log(R25) = 0.204 + 0.531 Log(K) - 0.350 Log(Phi)
Log(R30) = 0.215 + 0.547 Log(K) - 0.420 Log(Phi)
Log(R35) = 0.255 + 0.565 Log(K) - 0.523 Log(Phi)
Log(R40) = 0.360 + 0.582 Log(K) - 0.680 Log(Phi)
Log(R45) = 0.609 + 0.608 Log(K) - 0.974 Log(Phi)
Log(R50) = 0.778 + 0.626 Log(K) - 1.205 Log(Phi)
Log(R55) = 0.948 + 0.632 Log(K) - 1.426 Log(Phi)
Log(R60) = 1.096 + 0.648 Log(K) - 1.666 Log(Phi)
Log(R65) = 1.372 + 0.643 Log(K) - 1.979 Log(Phi)
Log(R70) = 1.664 + 0.627 Log(K) - 2.314 Log(Phi)
Log(R75) = 1.880 + 0.609 Log(K) - 2.626 Log(Phi)
```
> **Citation given:** "**Pittman, E.D.: "Relationship of Porosity and Permeability to
> Various Parameters Derived from Mercury Injection-Capillary Pressure Curves for
> Sandstone," AAPG Bull., v. 76, no. 2 (1992b) 191-198.**"
> Provenance: "based on correlation made on around **200** mercury injection PC curves from
> **sandstone** plugs... In the Pittman study a saturation of **36%** was found to be the
> average for these sandstones."
> **How to choose which R to use:** "If a plot of mercury saturation (HgSat) versus
> HgSat/PC is made (?Apex? plot), then the HgSat at the maximum value on the Y axis
> (threshold pressure) is the saturation where the Hg become continuous." The Cap-Pressure
> Set-Up page states the same rule with a worked case: "a **60% Sw** seen on the ?Apex?
> plot, which corresponds to a **40% mercury saturation**, means the **Pittman R40**
> equation should be used." — **This closes the loop between the Pc module and the rock-typing
> module and is the most directly implementable rock-typing rule in the whole target.**

**(d) Lucia Carbonate Rock Fabric Number (RFN)**
> Classes (identical boundaries stated twice on the page):
> - "Class 1 : Grain-dominated Fabrics ? Grainstones, dolograinstones, and large
>   crystalline dolostones. RFN?s of **0.5 ? 1.5**"
> - "Class 2 : Grain-dominated Fabrics ? Packstones. RFN?s of **1.5 ? 2.5**"
> - "Class 3 : Mud-dominated Fabrics ? Packstone, Wackestone, Mudstone. RFN?s of **2.5 ? 4.0**"
>
> **Citations given:** "**Jerry Lucia book Carbonate Reservoir Characterization, 2007
> published by Springer**. Also **SPE paper 84942 Predicting Permeability from Well Logs in
> Carbonates With a Link to Geology for Interwell Permeability Mapping August 2003 by
> James W. Jennings Jr and F. Jerry Lucia**."

The **RFN equation itself is rasterized** ("The Rock Fabric number is calculated from :"
followed by an image, then "Where porosity is in decimals and permeability is in
milli-darcies"), and so are the **Lucia Swi-vs-height equations** ("Lucia took the core
Capillary Pressure data to establish average equations, by rock class, for calculating
irreducible water saturation." → image → "H : height above the FWL **in feet**;
φ : interparticle porosity **in v/v**"). Both are **not recoverable** from the text and
must be sourced from Lucia 2007 / SPE 84942. Method note given: "He plotted the core data
Porosity v. Permeability for each class and fit the data using a **RMA regression**. He
combined the equations into a global transform using the boundaries of the petrophysical
rock classes."

Note that Lucia is the **only** rock-typing method in IP that also emits a saturation-height
product (`Output Rock Class Swi Curve`), bridging §2 and §4.

**(e) Boundary-picking machinery, shared by all HFU methods:**
- Two auto-init options: (i) "boundaries are selected from the maximum and minimum FZI
  values. They are equally spaced **in logarithmic space**"; (ii) cluster analysis —
  **"The FZI data is clustered into `25` nodes using K mean clustering. Then the cluster
  nodes are re-clustered into the number of selected flow units using Hierarchical
  clustering."** (identical wording and the same `25` for the Winland and Pittman variants.)
- "The low values of the first flow unit and the high value of the last flow units are
  **extended to cover data beyond the dataset currently selected**." — extrapolation guard.
- **Lorenz plot** for boundary picking: data reverse-sorted on FZI/WinR35/Pittman, then
  "linearly accumulated and normalized to a give a maximum value of **1.0**"; X =
  Cumulative Storage Capacity (cumulated porosity), Y = Cumulative Flow Capacity
  (cumulative permeability), Z = the indicator. "used for picking the flow unit boundaries
  at **inflection points**." Axis expansion options `-0.1 to 1.0` (X) and `0.0 to 1.1` (Y).
- Default HFU names "?HFU 1?, ?HFU 2? ...."; Lucia defaults "?RC 1?, ?RC 2? and ?RC 3?".
- Unit traps stated explicitly: "the **Winland equations use porosity in percent** so the
  input porosity curve may need multiplying by 100"; "the **Pittman equations use porosity
  in percent**"; "the **Lucia equations use porosity in decimals** so the input porosity
  curve may need dividing by 100". Three methods, two conventions, in one module.
- Data-mixing caveat: "It is recommended **not to mix log and core data** due to up scaling
  resolution problems."

### 4.3 How rock typing feeds the Pc modules

The Cap-Pressure Set-Up module carries a **`Rock Class`** column that is "either an input
or output curve", editable in the plug grid, pickable interactively on any crossplot with
a Rock Class Z axis (a `Rock Class Picker` dialog + circle cursor), and seedable from HFU
via a **`Copy HFU number to Rock Class`** button with "4 options to copy the HFU to the
Rock Class". `Select Plugs` then filters plugs "using their Rock Class or HFU number...
for use in the ?Capillary Pressure Function? module."

⚠ Stated gotcha: "**Note it is necessary to unselect all plugs before making a new
selection.** For example if you want to select all plugs with Rock Class 2 and 3 you first
need to unselect all, then select Rock Class 2 and finally select Rock Class 3." And: if
Rock Class values are absent, "the PC curves will **not show up** on the crossplot" — the
user must initialise them "to some default value (ie 0)".

**So IP's rock-typed SHF workflow is: HFU/cluster → Rock Class per plug → Discriminator →
one Model per rock type → Function Mixing to re-apply by rock type in the wells.** There
is no rock-type term inside any function; rock typing is entirely a partitioning concern.
That is a clean, implementable separation and SandiBumi should adopt it.

---

## 5. Log-Sw-versus-height functions — how they differ from core Pc functions

| | Core Pc route | Log Sw vs Height route |
|---|---|---|
| Requires core? | Yes (Pc arrays + core φ, K) | **"This module does not require core data."** |
| Independent variable | Pc (psi, converted) | h = height above FWL, in well depth units |
| FWL | needed only at apply time | **required up front**, per well, as TVDSS |
| IFT / contact angle | central | **not used at all** ("If working with log functions, capillary pressure is not required... The Oil / Gas density and IFT correction factors are not used.") |
| Inputs | `PcCorr`, `SwPcCorr`, `PcUse`, core φ/K | PHIE, SW, BVW (all **decimals**), optional PERM (mD or m²), TVDSS |
| Fitted per rock type via | Models + Discriminators | Functions + Discriminators (same pattern) |
| Outputs | `SwPcReg` / `SwCpIndReg` / `SwCpReg` | `SwLogHt`, `Ht_FWL` (+ an RQI curve when an RQI method is chosen) |
| Search space | Regression Function Comparator over models | Comparator over **"32 models"** |

Additional stated requirements and defaults:
- "**A Free Water Level (FWL) must be known in each well**", TVDSS, and the TVDSS curve's
  "Curve Type... must be set to `depth` in the Manage Curve Headers module."
- Unit-mixing prohibition: "**DO NOT mix wells with imperial (Feet) and metric units
  (meters) in the same function**, unless the TVDSS input curves... have been converted."
- "This module can only be run **after** an interpretation to evaluate Porosity and Water
  Saturation has been made in each well."
- FWL scoping: "The Use and the FWL TVDSS is now setup and saved **by the model Function**.
  Hence, in a reservoir with multiple stacked reservoirs different functions can be
  developed for each reservoir." A blank FWL on the Functions tab means "use different
  FWLs for different wells" from the Input Curves tab.
- Same 1/Sw weighting, same max factor 10, same fix-coefficient mechanism, same UDE
  save/load — the two modules are deliberately symmetric. UDE keywords differ only in the
  first: **`Ht, Phi, Perm`** here vs **`Pc, Phi, Perm`** on the Pc side, and "the ?Ht?
  keyword must be included in the equation for the equation to have any meaning."

⚠ **Naming collision worth flagging:** `RQI` means two different things in IP 2018 —
`√(K/ϕ)` **without a constant** in the Log-Sw-vs-Height methods, versus
`0.0314 × √(K/Phi)` in the Hydraulic Flow Units module. Same acronym, different scaling,
same product. Any implementation must namespace these.

---

## 6. Multiple Linear Regression (`multiplelinearregression.txt`)

**Role in this workflow:** it is *not* part of the SHF fitting chain. It is IP's generic
curve-prediction engine — "allows you to predict a result curve from a number of input
curves, using a **least squares regression routine**". In the rock-typing/SHF context its
use is upstream: predicting a continuous log-scale property (the manual's own example
predicts **core permeability**, `Perm`, plotted as points) so that a continuous K curve
exists in every well for the J-function / RQI / HFU routes, which all require log-derived
φ and K rather than sparse core (see §7).

Options and defaults:
- `Curve to Predict` row; **"If a well is not to be used in the creation of a model, but
  only for prediction, then leave the `Curve to Predict` cell blank."**
- `Default Name` doubles as an on/off switch: "**If the `Default Name` is blank then the
  row will not be used.**" — the documented way to try simpler models without retyping.
- `Log` column: "will normalize the curve by taking the **base 10 logarithm** of the curve
  before using it. All other displays in the module will then reflect this."
- Separate **Model Build** and **Model Run** depth intervals; "The defaults are the total
  well depths" for both. Build-interval limits are "not used for running a created model" —
  so you can fit on a subset and verify on everything.
- Two discriminator curve slots (`Discriminator Curve 1`, `Discriminator Curve 2`), applied
  at both build and run.
- **`Norm Coefficients`** reported alongside raw coefficients: "The closer the normalized
  coefficient value is to zero for an input curve, the lower its effect on the model build.
  Conversely, the closer the value is to one, the more important is the curves effect."
  Plus total point count and **R²**.
- `Clip resultant curve` — "when selected, restrict the output curve to have values within
  the minimum and maximum values entered." (bounds are user-entered; **no default values
  stated**).
- `Null All Output Curves` before re-running.
- **`Copy as Formula`** — right-click the coefficient grid to export the fit as a formula
  string usable in the User Formula module. IP does this everywhere (also in Cap-Pressure
  Functions and the Sat-vs-Height report); it is a cheap, high-value interoperability
  feature: *every fitted model in the system is exportable as text.*

No default number of curves, no regularisation, no collinearity diagnostics, no
train/test split are described.

---

## 7. Stated QC rules and caveats

**What invalidates a Pc fit (verbatim):**
- **Double / composite curves must be excluded.** "If for example, you think a Pc Curve is
  a **Double curve, i.e. a composite curve composed of two curves, each representing one
  pore network, it should be excluded** from your dataset. Obviously invalid Pc versus
  Saturation curves should also be discarded."
- **Multiple 100% Sw points break the fit** — hence the null-out rule in §3.4: "this
  insures that the curve fitting function used in the Cap Pressure Function module are not
  affected by multiple 100% Sw values with different Pc value."
- **You must QC one curve at a time.** "if multiple points are on top of each other then
  the Remove / Restore data point will only change the top point. Hence, it is best when
  QCing Pc curves to work on one Pc curve at a time."
- **Only certain crossplots can edit.** "For quality checking plug data the `Pc V Sw. Z
  axis Use flag` crossplot **must** be selected. The Phi and Perm crossplots **cannot** be
  used for quality checking the plugs interactively... these crossplots ignore the the
  PcUse curves and therefore are unable to change their values."
- **Destructive-reset warnings.** Flipping a plug back to `Plug Good` "will reset all the
  plugs that you have Qcd and possibly modified... Your data is returned to its un-modified
  state. **Any modifications are discarded.**" `Clear All` likewise: "any Pc curve edits
  that you have performed up until the point when you click Clear All will be lost."

**Core-data conditioning rules:**
- "**the core porosity should be overburden corrected** as the function generated will be
  used with log porosity."
- "the core permeability should be overburden corrected **only if** used with log data
  which has generated an overburden corrected permeability. I.e. log and core data should
  be the same with respect to overburden correction."
- Closure: "Closure corrections are normally made to the Pc curves by the core laboratory...
  **There is no definitive way of calculating the closure correction and it has to be done
  visually** by looking closely at each Pc curve."
- Clay correction is scoped: "**Where an Air/Mercury measurement system has been used**,
  applying the Clay Correct Flag provides a correction for the missing clay-bound water."

**Application-side rules:**
- "**Sparse data Core Porosity and / or Core Permeability curves are not suitable for use
  in this module**" — the apply module needs continuous log curves "calibrated to core data
  (where available)". This is the reason MLR (§6) matters.
- "The Functions Mixing Setup relies on a **consistent curve naming convention in all
  selected wells**. So, for example, if a Function discriminator setting uses a curve
  called `PHIE`, then this curve must be available in all wells." Same constraint on UDE
  optional curves: "these curves must exists in all wells used to create this function."
- `Null output Sw Curves` warning: "if you are using multiple hydrocarbon models, selecting
  this check box will **delete the results from other models**."
- Fit-quality reading: for the per-plug route "The output shows a near-perfect 1:1
  regression curve fit for each separate plug, **as we are regressing only one curve at a
  time**" — i.e. the individual-equation R² is not evidence of anything; only the Combined
  Equation crossplot is ("This plot will show much wider data scatter").
- Function-shape choice: "**Both Hyperbola and Exponential regression fits are better
  suited to use in poor quality rock, because they are not very good at representing sharp
  transition zones seen in high porosity / permeability reservoir.**" And the four
  Porosity & Pc functions "are suitable for poor quality (low porosity & permeability)
  reservoir rocks which have significant transition zones."
- Sanity check before trusting a UDE regression: "It is a good idea to use the '`View
  Function`' option to check the resulting functions are at least plotting on-screen. If
  they are not, then this usually indicates that the starting coefficients are so far off
  that the regression has not converged on a sensible solution."
- Loader collision rule: `Depth Tolerance` — "If, for example, 2 Pc core plugs are closer
  to one another than the ?Depth Tolerance? value then, on loading, the Pc / Sw data for
  the second plug will be considered to be at the same depth as that for the first plug and
  will **overwrite** the data for plug 1." Loader limit: **"A maximum of 100 plugs"** per
  session; defaults `No. of Curves` = **five**, `No. of Text Curves` = **two**.

---

## 8. Tier-C flags

**None found in this target.** Every method in these pages is either standard published
science (Leverett/Thomeer/Brooks-Corey/Skelt-Harrison/Winland/Pittman/Lucia/FZI), standard
statistics (K-means, PCA seeding, five classic agglomerative linkages, least-squares MLR,
RMA), or plain UI/workflow. No patent numbers, no trademarks, no "proprietary" or
"patented" language, no shipped model weights appear anywhere in the eight assigned pages
or in `hfu.txt`.

Two items sit just below the Tier-C line and are logged for completeness, **not** as
restrictions:
1. **Cluster Randomness Plot / Randomness index** (§4.1.5) — vendor-original heuristic with
   no citation, but the formula is fully disclosed in the manual. Reimplementable; validate
   independently rather than assuming published pedigree.
2. **Regression Function Comparator** (Pc and log routes) — brute-force-and-rank-by-R² over
   the model grid. Trivially reimplementable UI logic, no method claim.

Adjacent (out of scope, flagged for whoever takes the ML target): `som.txt`
(Self-Organising Maps) ships a trained-map concept and Kohonen citation, and
`statisticalcurveprediction.txt` implements Cuddy's fuzzy-logic prediction — check those
against the Tier-C register separately.

---

## 9. Gaps — what this target could NOT recover

1. **Laboratory Contact Angle and Interfacial Tension defaults for Mercury Injection /
   Centrifuge / Porous Plate.** The single highest-value number set in this target.
   Manual confirms defaults exist and ship with a `Restore Defaults` button, but the values
   are in a screenshot only. `not stated in manual`. **Must be read off the live IP UI or
   sourced from literature — do not guess.**
2. **The lab→reservoir Pc conversion equation itself** — rasterized. Only the prose
   ("absolute ratio of the Laboratory and Reservoir σ * Cos θ values") plus the 1.924
   worked example survive.
3. **Porosity & Pc Function 1, 2, 3 and Porosity & Pc Lambda Function** — four SHF
   functions whose equations exist only as dialog screenshots. Parameters known
   (`a, b, λ, c`, φ in decimals, Pc in output-curve units); forms unknown. Their
   log-domain siblings (Porosity & Height 1/2/3/Lambda) **are** given in text and are the
   obvious place to look for the pattern, but they are not the same functions and must not
   be substituted.
4. **Kozeny-Carmen derivation steps** in the RQI methodology — rasterized.
5. **Lucia RFN equation** and **Lucia Swi-by-rock-class equations** — rasterized. Units are
   stated (φ decimals / K mD; H in feet, φ in v/v) but the algebra is not.
6. **`[[EQUATION_IMAGE: embim460.gif]]`** — Brooks Corey, the only tagged equation image in
   the assigned set. Mitigated: the same equation is given in text elsewhere on the page.
7. **The clay-correction `F` expression has unbalanced brackets** in the decompiled text
   (§3.4). Needs verification against the rendered page before implementation.
8. **No citation for any saturation-height function.** Leverett, Thomeer, Brooks-Corey,
   Skelt-Harrison, Lambda, and the BVW-vs-height (Cuddy-shaped) method are all uncited.
   The papers must be sourced independently — the existing project note
   `docs/research_2026-07/ref_rocktyping_shf.md` already holds Leverett 1941 and Cuddy
   1993/2017 locally.
9. **No default numeric starting values** are stated for any non-linear regression, and no
   default coefficient values for any SHF function. IP ships forms, not parameters.
10. **No stated convergence tolerance, iteration cap, or R² acceptance threshold** anywhere
    in the fitting chain — only a user-operated Cancel button.

---

## 10. The three findings that matter most for an independent implementer

1. **The Apex-plot → Pittman-R selection rule is the one fully-specified, end-to-end
   rock-typing recipe in the product.** Plot HgSat vs HgSat/Pc from the MICP data; the
   HgSat at the Y-maximum is where mercury becomes continuous; pick the Pittman R-equation
   at that saturation (worked example: 60% Sw ⇒ 40% Hg ⇒ use R40). All 14 Pittman
   coefficient sets are printed verbatim, the citation is complete
   (Pittman 1992b, AAPG Bull. v.76 no.2, 191-198), the derived quantity (`Pc Sat / Pc`)
   is already an output of the Pc module, and the porosity-in-percent trap is documented.
   This is buildable today with zero further research.

2. **Rock typing is a partitioning concern, never a function argument.** Not one of IP's
   32+ saturation-height forms takes a rock type. Rock class arrives via HFU/cluster
   assignment on the plug, is filtered by Discriminators into one Model per rock type, and
   is re-applied in the wells through a separate Function Mixing table that may legitimately
   differ from the fitting discriminators ("it is possible to create a function using
   discriminators for porosity > 0.1 but apply the function to all porosity ranges").
   Copying this three-way split — *classify → fit per class → mix on apply* — buys SandiBumi
   the whole rock-typed-SHF capability without any function-signature changes, and it means
   the SHF and rock-typing build increments can ship independently and still compose.

3. **The correction order (§3.5) and the 1/Sw-clamped-at-10 weighting (§2.3) are the two
   silent-wrongness landmines.** Both are fully specified in the manual and both change
   every fitted coefficient if done differently: corrections run in *non-wetting-phase*
   space with the flip to wetting phase at step 5, immediately before the lab→reservoir
   conversion; and the regression weight is `min(1/Sw, 10)` with Sw in decimals. Neither
   produces an error if implemented wrong — they just produce a different, plausible
   function. Pin both with unit tests reproducing the manual's own closure-correction table
   (§3.4) as a fixture.

Runner-up, because it is cheap and disproportionately useful: **the View-Function /
extrapolation-preview crossplot** (§2.3) — render the fitted function as constant-φ and
constant-K families across the *reservoir's* property range, not the *core plugs'* range,
so the user sees what the function does where it was never calibrated. Optional 3D surface.
