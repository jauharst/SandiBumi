# E — Saturation-Height, Capillary Pressure & Hydraulic Flow Units (IP 2025 CHM ingest)

Agent E of the 14-agent Interactive Petrophysics 2025 vendor-manual ingest.
Consumer: SandiBumi. Provenance discipline: every fact below carries
`(pagename.htm)` for prose or `[img-read: file.png]` for an image transcription.
Nothing is filled in from textbook knowledge — where the manual is ambiguous it
is reported as ambiguous in §8.

Source root (read-only): `C:\Users\ARUNIKA\AppData\Local\Temp\c25\`
IP2018 counterpart (numeric diffing only): `C:\Users\ARUNIKA\AppData\Local\Temp\c18\`
Prior-ingest cross-check anchor: `D:\XX. SandiBumi\docs\research_2026-07\ip2018_chm_ingest\E_shf_rocktyping.md`

---

## 1. Scope & page inventory

All 7 assigned pages read in full (text stream + raw-HTML fallback where needed),
plus 34 content images read directly (vision) at the points where an equation,
a defaults table, or a dropdown enumeration sits behind an `[[IMG:…]]` marker.

| # | Page (stem) | Title | Chars | Content imgs | Status | Images read |
|---|---|---|---|---|---|---|
| 1 | `cappressuresetup` | Capillary Pressure Set-Up & Corrections | 63,470 | 56 | full | 10 |
| 2 | `cappressurefunctions` | Capillary Pressure Functions | 52,618 | 84 | full | 11 |
| 3 | `hfu` | Hydraulic Flow Units | 35,656 | 55 | full | 15 |
| 4 | `logswversusheightfunctions` | Log Sw Vs Height Functions | 20,276 | 35 | full | 2 |
| 5 | `saturation_versus_height_curve` | Saturation Versus Height Curves | 20,039 | 16 | full | 1 |
| 6 | `capillarypressuredataloader` | Capillary Pressure Data Loader | 14,582 | 16 | full | 0 (UI-only, no equations) |
| 7 | `saturation_height_modelling` | Saturation Height Modeling hub | 3,772 | 0 | full | 0 (nav hub) |

### Module architecture (saturation_height_modelling.htm)

Four interrelated modules under **Advanced Interpretation → Saturation Height
Modeling (Capillary Pressure)**:

1. **Capillary Pressure Set-Up & Corrections** — QC + correct raw Pc; outputs
   `PcCorr`, `SwPcCorr`, `PcUse`.
2. **Capillary Pressure Functions** — fits curve-fitting models to the corrected
   Pc/Sw data.
3. **Log Sw versus Height Functions** — fits Sw-vs-height functions from *log*
   curves only; requires no core data.
4. **Saturation Versus Height Curves** — applies (1–3)'s functions to multiple
   wells/zones; can also solve for an unknown FWL.

Data ingress is via the **Capillary Pressure Data Loader** (max 100 plugs per
session), or ASCII Load / Interval Loader (saturation_height_modelling.htm,
capillarypressuredataloader.htm).

All four modules share one external parameter file, a **`.cap` Parameter Set**,
by default in the IP project root, plus an automatic copy into the project
database `IPDBProj.dat` (per logged-in user) (saturation_height_modelling.htm).

**Plug gating rule** (repeated on 3 pages): only plugs with `Plug Status` =
`Good` or `Part Good` **and** the `Select Plug` column ticked in the Set-Up
module's *Data View / Edit* tab are passed to the Capillary Pressure Functions
module (cappressuresetup.htm, cappressurefunctions.htm).

---

## 2. Equations & methods (with provenance)

**86 distinct equations / functional forms** captured. Image-sourced equations
are tagged `[img-read: …]`; the remainder are literal from the text stream.

### 2.1 Saturation-height functional forms offered (SPECIAL TASKING (a))

IP 2025 offers three *model types* in the Capillary Pressure Functions module —
`One Equation for all Pc curves`, `Separate equation for each Pc curve`,
`User Defined Equation` — plus a parallel, core-free set in the Log Sw vs Height
module. Complete roster:

#### A. Pc-route, "One Equation for all Pc curves" — `Method` dropdown

| Method | Equation | Fitted params | Source |
|---|---|---|---|
| Leverett J Function | `J = 0.2166 * Pc / (σ * COS θ) * √(K/ϕ)` | none in J itself; J then regressed | cappressurefunctions.htm; **confirmed** `J = 0.2166 * (Pc / (σ Cosθ)) √(K / Φ)` [img-read: _shmclip0028.png] |
| Leverett J, porosity modifier | `J = 0.2166 * Pc / (σ * COS θ) * √(K/ϕ^m)` | `m` (user input, default 2.8) | cappressurefunctions.htm; **confirmed** [img-read: _shmclip0029.png] |
| Porosity & Pc function 1 | `Sw = 1.0 / [ (a + Pc^b) Φ^c ]` | `a, b, c` | **RECOVERED** [img-read: _shmclip0030.png] |
| Porosity & Pc function 2 | `Sw = 1.0 / [ (a + b.Pc^c) Φ^d ]` | `a, b, c, d` | **RECOVERED** [img-read: _shmclip0031.png] |
| Porosity & Pc function 3 | `Sw = [ a + b.Log(Pc) + c.Log(Pc)^2 + d.Log(Pc)^3 ] / [ Φ^f ]` | `a, b, c, d, f` | **RECOVERED** [img-read: _shmclip0032.png] |
| Porosity & Pc Lambda function | `Sw = a.Pc^(b.Φ + c) + d` | `a, b, c, d` | **RECOVERED** [img-read: _shmclip0033.png] |
| Thomeer function | `Sw = 1.0 - (BVnw∞ / Φ) e^( -G / Log10(Pc/Pd) )` | `BVnw∞`, `G`, `Pd` | cappressurefunctions.htm (glyphs `_shmclip0004/0005` = `∞`, `Φ`); **confirmed inline** [img-read: _shmclip0034.png] |

Thomeer parameter definitions (cappressurefunctions.htm):
`BVnw∞` = bulk volume of the non-wetting phase at infinite Pc; `Φ` = input
porosity; `G` = curve-shape factor; `Pd` = displacement pressure (Pc needed to
start reducing saturations).

The four "Porosity &" methods are described as suited to **poor-quality (low
φ/K) reservoir rock with significant transition zones** (cappressurefunctions.htm).
Pc units in these equations = the *output* Pc curve units set in the Set-Up
module; porosity is **always decimal** (cappressurefunctions.htm).

#### B. Regression equations applied to `J` (One-Equation type)

`Regression Equation` dropdown enumerates exactly seven entries
[img-read: _shmclip0180.png]:

| Entry | Equation | Fitted params |
|---|---|---|
| Lambda | `Sw = a * J^(-λ) + b` | `a, λ, b` |
| Hyperbola | `Sw = a / (J - b) + c` | `a, b, c` |
| Exponential | `Sw = a * e^(b*J) + c` | `a, b, c` |
| Normalized J (RQI) | see §2.2 | `a, b, c, d` |
| Normalized J (FZI) | see §2.2 | `a, b, c, d` |
| Normalized J (Phi) | see §2.2 | `a, b, c, d` |
| Normalized J (Perm) | see §2.2 | `a, b, c, d` |

(cappressurefunctions.htm for equations; dropdown membership from the image.)

Note the ASCII prose calls this option "Normalised J" (British spelling) while
the UI dropdown reads "Normalized J" — see §5.

Vendor guidance: *"Both Hyperbola and Exponential regression fits are better
suited to use in poor quality rock, because they are not very good at
representing sharp transition zones"* (cappressurefunctions.htm).

#### C. Normalised-J workflow (SPECIAL TASKING (a) + (c) interface)

Full chain (cappressurefunctions.htm):

```
J     = 0.2166 * Pc / (σ * COS θ) * √(K/ϕ)          (output curve: Lev_J)
SwN   = (Sw − Swirr) / (1 − Swirr)                   (output array curve: SwN_cp)
SwN   = a . J^b                                       (regression 1 → a, b)
Swirr = c . RQI^d                                     (regression 2 → c, d)
Sw    = Swirr + ((1 − Swirr) × SwN)                   (re-arranged)
Sw    = c.RQI^d + (1 − c.RQI^d) × a.J^b               (final substituted form)
```

The rock-quality indicator in regression 2 is user-selected from **RQI, FZI,
Phi, Perm**. When the model is later run in the well, continuous φ and/or K
curves are required for that indicator (cappressurefunctions.htm).

`Swirr` source is either **the lowest Sw at the highest Pc**, or an external
`SWirr Curve In` set on the Set-Up module's Curves Set-up tab — the manual notes
the highest Pc may not represent true irreducible (cappressurefunctions.htm).

RQI as used *inside the Capillary Pressure Functions module*:

```
RQI = 0.0314 √(K / Φ)          [img-read: _shmclip0181.png]   output curve: RQI_cp
```

Worked example carried by the manual [img-read: _shmclip0182.png]:

```
Sw = 0.01222 * RQI^-0.71201 + (1 - 0.01222 * RQI^-0.71201) * 0.16108 * J^-1.36921
R2 = 0.9083 | Num of Pc Crvs used = 16 | Num of Points used = 2869
a = 0.16108   b = -1.36921   c = 0.01222   d = -0.71201
```

**Weight Regression** has **no effect** in the Normalised-J workflow
(cappressurefunctions.htm) — unlike every other regression type.

#### D. Pc-route, "Separate equation for each Pc curve"

Each plug fitted individually, then coefficients combined into a *Combined
Equation*. `Regression Equation` options (cappressurefunctions.htm):

| Entry | Equation | Fitted params |
|---|---|---|
| Lambda | `Sw = a * Pc^(-λ) + b` | `a, λ, b` |
| Hyperbola | `Sw = a / (Pc - b) + c` | `a, b, c` |
| Exponential | `Sw = a * e^(b * Pc) + c` | `a, b, c` |
| Thomeer | `Sw = 1.0 - (BVnw∞ / Φ) e^( -G / Log10(Pc/Pd) )` | `BVnw∞, G, Pd` |
| **Brooks Corey** | `Sw = (Pc / Pd)^(-λ) * (1 - Swi) + Swi` | `Pd, λ, Swi` |
| **Skelt Harrison** | `Sw = a*e^( -(b/(Pc+d))^c )` | `a, b, c, d` |
| User-Defined | free-form, ≤4 regressable coefficients | ≤4 |

Brooks-Corey is confirmed independently as a raster: `Sw = (Pc/Pd)^(−λ) × (1 −
Swi) + Swi` [img-read: embim288.png]. Its fixed-coefficient meanings
(cappressurefunctions.htm): `Pd` = displacement pressure (value when Sw = 1);
`λ` = Pc-curve shape factor; `Swi` = irreducible water saturation (value when Pc
is very large).

User-Defined example dialog [img-read: _shmclip0159.png]:
`Sw = Acoeff * Pc^(- Bcoeff) + Ccoeff`, `Num. of coefficients = 3`,
initial values `Acoeff 1`, `Bcoeff 0.2`, `Ccoeff 0.03`, `Dcoeff 0` (greyed).

**Combined-Equation correlation types** (each coefficient may be replaced by a
function of a chosen curve): `y = Av. value`, `y = Median value`, `y = f(x)`
(minimises squared Y-errors), `x = f(y)` (minimises squared X-errors), `RMA`
(reduced major axis, midway between the two), `2nd Order Poly`, `3rd Order Poly`
(cappressurefunctions.htm).

#### E. Log Sw vs Height route (no core data) — `Method` dropdown

Eight entries [img-read: _shmclip0073.png], equations from
logswversusheightfunctions.htm:

| Method | Equation |
|---|---|
| Sw function of height | `Sw = f(h)` |
| BVW function of height | `BVW = f(h)` |
| Rock Quality Index | `Sw = f(RQI.h)` where `RQI = √(K/ϕ)` |
| Rock Quality Index, porosity modifier | `Sw = f(RQI.h)` where `RQI = √(K/ϕ^m)` |
| Porosity & Height function 1 | `Sw = 1.0 / [(a + h^b) . ϕ^c]` |
| Porosity & Height function 2 | `Sw = 1.0 / [(a + b.h^c) . ϕ^d]` |
| Porosity & Height function 3 | `Sw = [a + b.Log(h) + c.Log(h)^2 + d.Log(h)^3] / [ϕ^f]` |
| Porosity & Height Lambda function | `Sw = a.h^(b.ϕ + c) + d` |

> **The `RQI` here carries NO `0.0314` constant.** See §5, discrepancy D1 — this
> is the single most dangerous item on these pages for a reimplementation.

`Regression Equation` dropdown — seven entries [img-read: _shmclip0074.png],
equations from logswversusheightfunctions.htm:

| Entry | Equation |
|---|---|
| Linear | `Sw = a + b.h` |
| Linear / Log | `Sw = a + b.Log(h)` |
| Log / Linear | `Log(Sw) = a + b.h` |
| Log / Log | `Log(Sw) = a + b.Log(h)` |
| Lambda | `Sw = a.(h)^(-λ) + c` |
| Hyperbola | `Sw = a / (h - b) + c` |
| Exponential | `Sw = a.e^(b.h) + c` |

8 methods × 7 regressions ⇒ the manual states the **Regression Function
Comparator rates 32 models** (logswversusheightfunctions.htm) — see §5, D8.

### 2.2 Lab-to-reservoir Pc conversion chain (SPECIAL TASKING (b) — RECOVERY)

The IP2018 ingest recorded this conversion as **lost to rasterisation**
(`E_shf_rocktyping.md` §"OPEN ITEMS" items 1 and 2). **Both are now recovered.**

**Step 1 — the conversion equation** [img-read: clip1181.png]:

```
                    ⎡ σ_Res * Cos θ_Res ⎤
Pc,Res = Pc,Lab * ABS⎢ ───────────────── ⎥
                    ⎣ σ_Lab * Cos θ_Lab ⎦
```

Symbols (cappressuresetup.htm): `Pc,Res` = capillary pressure at reservoir
conditions; `Pc,Lab` = at laboratory conditions; `σ` = interfacial tension;
`θ` = contact angle. Note the **ABS** — the ratio is taken in absolute value, so
a lab contact angle > 90° (mercury, 140°, `Cos θ < 0`) does not flip the sign.

**Step 2 — the σ / θ defaults table.** Recovered in full from the *Reservoir and
Laboratory fluid / rock properties* table [img-read: _shmclip0009.png],
independently corroborated by [img-read: _shmclip0084.png] and
[img-read: _shmclip0167.png]:

| Measurement Type | Method ID | Contact Angle **Laboratory** (deg) | Interfacial Ten. **Laboratory** (dynes/cm) | Contact Angle **Reservoir** (deg) | Interfacial Ten. **Reservoir** (dynes/cm) |
|---|---|---|---|---|---|
| **Mercury Inj** | 1 | **140** | **480** | 30 | 30 |
| **Centrifuge** | 2 | **0** | **72** | 30 | 30 |
| **Porous Plate** | 3 | **0** | **72** | 30 | 30 |

Derived `σ·cos θ` per system (computed here from the table's own values, not
printed by the manual — flagged as derived):

| System | σ·cos θ (dynes/cm) | |σ·cos θ| |
|---|---|---|
| Mercury injection (lab) | 480 × cos 140° = −367.7 | 367.7 |
| Centrifuge (lab) | 72 × cos 0° = 72.0 | 72.0 |
| Porous plate (lab) | 72 × cos 0° = 72.0 | 72.0 |
| Reservoir default (table) | 30 × cos 30° = 25.98 | 25.98 |
| Reservoir, Set Gas/Water | 50 × cos 0° = 50.0 | 50.0 |
| Reservoir, Set Oil/Water | 30 × cos 30° = 25.98 | 25.98 |
| Reservoir, Dry Rock | 1 × cos 0° = 1.0 | 1.0 |

**Step 3 — the reservoir-side preset buttons** (cappressuresetup.htm,
[img-read: _shmclip0010.png]):

- `Set Gas / Water` → Contact Angle Reservoir **0 degrees**, Interfacial Tension
  Reservoir **50 dynes/cm** (applied to all three method rows).
- `Set Oil / Water` → Contact Angle Reservoir **30 degrees**, Interfacial Tension
  Reservoir **30 dynes/cm**.
- `Restore Defaults` / `Restore Lab Defaults` button resets **the first 3 columns
  only** (Measurement Type, Method ID, Contact Angle Laboratory) per the prose —
  see §5, D6.

**Step 4 — Dry Rock mode.** Ticking `Use Dry Rock Reservoir Properties` sets
**Contact Angle Reservoir = 0 deg** and **Interfacial Tension Reservoir = 1
dynes/cm** for every method row (cappressuresetup.htm; visually confirmed
[img-read: _shmclip0084.png]). Consequences (cappressuresetup.htm):

- σ/θ are then supplied later, per-well, in the Saturation Versus Height module.
- The `Pc Height` output curve is **no longer produced**, and the Height-v-Sw QC
  crossplots are removed from the *Make Crossplot* menu.
- *"The dry rock option greatly reduces the values of the corrected Pc pressure
  curves"* — crossplot scales must be re-set manually.
- Functions built this way are not tied to one hydrocarbon type (oil vs gas).

**Step 5 — order of operations.** The `Run Corrections` button executes in this
fixed order (cappressuresetup.htm):

1. Convert input Pc Saturation data to **non-wetting** phase saturations (per the
   `Pc Sat Type In` setting).
2. Closure-correct the input saturation values.
3. If multiple Sw = 100% values remain, **all but the last are set to Null** — to
   stop the Functions module's curve fits being skewed by several 100%-Sw points
   at different Pc.
4. Stress correction (if selected), on Pc and on non-wetting-phase saturation.
5. Clay-bound-water correction (if selected).
6. Convert saturation data back to **wetting** phase saturations.
7. Laboratory → reservoir Pc conversion (if selected).

> Note the corrections at steps 4–5 are applied to the **non-wetting** phase
> saturation, but the correction formulae in §2.3 are written in terms of
> `SwPc` (wetting phase). See §5, D5.

### 2.3 Pc corrections (cappressuresetup.htm)

**Wetting/non-wetting conversion:**

```
SwPc = 1 - (Snw)
```
where `Snw` = input non-wetting-phase saturation, `SwPc` = wetting-phase water
saturation. Set by the `Pc Sat Type In` parameter (`Water Wet` | `Non Wet`).

**Stress correction** (activated by the `Stress Correct` flag):

```
PcCorr   = Pc * (PhiRes/PhiLab Factor) ^ (-0.5)
SwPcCorr = 1 - [ (1 - SwPc) * (PhiRes/PhiLab Factor) ]
```
`PhiRes/PhiLab Factor` = laboratory→reservoir porosity correction factor, decimal.

**Clay-bound-water correction** (activated by the `Clay Correct` flag). Cited to
**Hill, Shirley and Klein 1979, SPWLA 20th Annual Symposium Paper AA — "The
Central Role of Qv and Formation Water Salinity in the Evaluation of Shaley
Formations"** (cappressuresetup.htm):

```
PcCorr   = Pc * (F) ^ (-0.5)
SwPcCorr = 1 - (1 - SwPc) * F
```
with the correction factor, **transcribed literally from the page**:

```
F = 1 - [0.6425 * ( Salinity ^ (-0.5) + 0.22 ] * Qv ]
```

`Salinity` = formation water salinity in **Kppm NaCl equivalent**;
`Qv` = cation exchange capacity per total pore volume in **meq/ml**.

> This bracket sequence is **unbalanced** (`[ ( ] ]`) and cannot be evaluated as
> written. It is byte-identical in IP2018 and IP2025, so it is a long-standing
> vendor typo, not an ingest artefact. Two readings are possible — see §8, OPEN-1.
> The physical rationale given: *during cleaning and drying of shaley core plugs,
> clay-bound water could be lost*; an **Air/Mercury** measurement system is named
> as the case where this matters. A `Qv` input curve is mandatory when the flag
> is on.

**Closure correction — four methods** (cappressuresetup.htm). Default method is
set on the Correction Parameters tab and may be overridden per plug; the
per-plug dropdown value `Default` defers to the module default. **Module default
= `Shift`** [img-read: _shmclip0167.png].

| Method | Behaviour | Equation / rule |
|---|---|---|
| **Shift** | shifts every Sw by a fixed amount until the closure point meets Sw = 1; points originally right of the closure point "pile up" on the Sw = 1 line | `SwCorr = Sw(input) + (1 - SwClosure)` |
| **Proportional** | divides Sw by the closure Sw value; shifts high-Sw values more than low | prose only: *"divides the plug Sw values by the Sw Closure value"* |
| **Crop** | removes Pc/Sw points above the closure Sw value; no shift applied | procedural |
| **Extrapolate** | takes the two next-lower plug points below the closure Sw point and extrapolates all values above it | Entry pressure = where the extrapolated line crosses Sw = 1 |

Entry-pressure rules (cappressuresetup.htm):
- Entry pressure is normally the Y-axis value of the picked closure point, but is
  **clipped to realistic values**: it *cannot be higher than the plug Pc values
  at Sw lower than the closure Sw*.
- For **Extrapolate**, entry pressure is **fixed** — the user may only adjust the
  closure Sw value.
- In **all four methods**, if multiple Sw = 100% values survive the correction,
  all but the last are set to Null.
- Closure picking has two modes: **Automatic** (user clicks between two raw points;
  IP fits lines through the two points either side and takes their intersection)
  and **Manual** (drag the red closure star). If either axis is set logarithmic,
  the auto-pick lines are fitted **in logarithmic space** — and the picked closure
  correction and entry pressure will differ from the linear pick.

Closure corrections picked on the crossplot are stored in the `Closure
Correction` and `Entry Pressure` output curves **regardless** of whether the
`Closure Correct` flag is on; the flag only controls whether they are *applied*
(cappressuresetup.htm).

### 2.4 Pore-size inversion (cappressuresetup.htm)

Pore-throat radius from Pc [img-read: _shmclip0175.png]:

```
Pc = 2σ Cosθ / r
```
`r` = radius; `σ` = interfacial tension **at lab conditions**; `θ` = contact
angle **at lab conditions**; `Pc` = capillary pressure.

Algorithm as documented:
1. Convert each corrected **lab-condition** Pc point to a radius via the equation
   above.
2. For each of **80 fixed radii from 0.01 to 100 microns**, look up the
   corresponding saturation on the Radius/Sw curve. Because no function has yet
   been fitted at this stage, the lookup is a **polynomial fit through the
   closest 3 data points**.
3. Difference each Sw element from its neighbour → volume accessed through that
   pore-throat radius range.
4. Normalise all 80 elements so their **sum = 1**.

Output curves:
- **`Pore Size`** — 80-element array; logarithmic scale, **20 values per decade**;
  unitless (a fraction of total pore volume).
- **`Throat Size`** — 80-element scale/partner array, always the same values,
  first = **0.01 microns**, last = **100 microns**, logarithmically spaced.
  Exists only because the Crossplot module needs a curve to plot against.
- **`Pc Normalized`** — 51-cell array, each cell = **2.0 saturation units**, cell
  value = the pressure; explicitly described as *approximate due to the coarseness
  of the cells*.

### 2.5 Swanson permeability (cappressuresetup.htm) — NEW IN IP 2025

`Swanson Point` = the inflection point where the ratio of **non-wetting-phase
saturation over capillary pressure**, `(So/Pc)`, becomes a **maximum**. Output as
its own curve so the user can build alternative correlations in a user formula.

Four correlation models [dropdown confirmed: img-read: _shmclip0176.png]:

| Model | Equation |
|---|---|
| Gas All Rocks | `K = 399 x (So/Pc)^1.691` |
| Liquid Carbonates | `K = 431 x (So/Pc)^2.109` |
| Liquid Sands | `K = 290 x (So/Pc)^1.901` |
| Liquid All rocks | `K = 355 x (So/Pc)^2.005` |

Swanson output curves are created when `Run Corrections` is clicked.
The dropdown labels are `Gas`, `Liquid Carbonates`, `Liquid Sands`, `Liquid All`
— slightly abbreviated versus the prose names.

**No citation for Swanson is given anywhere on the page.**

### 2.6 Height ↔ Pc conversion and datum conventions (SPECIAL TASKING (d))

**Set-Up module — `Pc Height` panel** (cappressuresetup.htm):

```
Height = Pc / [ 0.433 (ρWater - ρHc) ]
```
- `h` = height above **FWL**, in ft
- `Pc` = the Hc/brine capillary pressure, in psi
- `ρWater`, `ρHc` = **specific gravities** of brine and hydrocarbon at **reservoir
  conditions**
- `0.433 psi/ft` = *"the gradient of pure water at ambient conditions: takes care
  of the g term in the normal P = hρg equations"*

The page carries a full derivation of why `g` is absent
[figure: img-read not required — text stream complete; capillary-tube schematic
is `_shmclip0161.png`]:

```
P1 = hρ1 g ;  P2 = hρ2 g
Pc = hρ1 g − hρ2 g = h g (ρ1 − ρ2)          "standard" form
G1 = ρ1 g ;  G2 = ρ2 g                       gradients (psi/ft)
P1 = h G1 ;  P2 = h G2
Pc = h (G1 − G2)
G1 = 0.433 * SG1 ;  G2 = 0.433 * SG2        "The gradient of fresh water is
                                             0.433 psi/ft for a density of
                                             1 g/cc and standard gravity g"
Pc = h * 0.433 * (SG1 − SG2)                 the form IP uses
```

The corrected `PcCorr` value is used; **Height is emitted in the same units as
the database depth curve**, and densities are converted internally
(cappressuresetup.htm).

`Pc Height` panel defaults [img-read: _shmclip0011.png]:
**Water Density `1`, Hyd. Density `0.7`, Fluid Units `gm/cc`.**

**Saturation Versus Height module — forward Pc from height**
(saturation_versus_height_curve.htm):

*Oil situation:*
```
Pc = h * 0.433 (ρWater - ρHc) * IFTCorrFactor
```

*Gas-over-oil situation:*
- Between FWL and GOC: identical to the oil equation.
- Above the GOC, Pc is the sum of two components — the full oil column of height
  `(FWL−GOC)`, plus the height into the gas cap `(h − (FWL−GOC))`:

```
Pc = (FWL-GOC) * 0.433 (ρWater - ρHc)  * IFTCorrFactor
   + (h - (FWL-GOC)) * 0.433 (ρWater - ρGas) * IFTCorrFactorGas
```

`FWL` and `GOC` are **TVD depths**; `h` = height above FWL in ft; `ρWater, ρHc,
ρGas` are **specific gravities at reservoir conditions**.

*IFT correction factor for the gas leg* — worked example given verbatim:
> "if the function was developed for oil with an IFT value of 30 and a contact
> angle of 30 degrees, then used in gas with an IFT of 50 and a contact angle of
> 0 degrees. The IFTCorrFactorGas would be equal to
> `50 x Cos(0.0) / (30 x Cos(30.0)) = 1.924`"

i.e. `IFTCorrFactorGas = (σ_gas · cos θ_gas) / (σ_oil · cos θ_oil)`
(verified arithmetically: 50 / (30 × 0.866025) = 1.9245 ✓).

**Datum definitions** (saturation_versus_height_curve.htm):
- **TVD Free Water Level** — True Vertical *Subsea* Depth, entered per well;
  mandatory.
- **TVD Gas Oil Contact** — TVDSS; required only for Pc models in gas-over-oil
  reservoirs; leave blank otherwise.
- **TVD Gas Free Water Level** — an *alternative* way to specify the GOC. The two
  are **bidirectionally linked**: editing one updates the other. **It is the GOC
  value that enters the calculation.** The Gas FWL is defined as *the projection
  of the gas Pc line back to the zero-capillary-pressure axis*, so the link is
  through the slope of that line, which depends on `(ρWater − ρGas)` — **the
  density values must be populated for the linking to work**.
  Geometry confirmed [img-read: _shmclip0185.png]: the **Gas FWL lies below the
  GOC and above the (oil) FWL**; `H` in the figure is the GOC→GasFWL interval.
- **`Oil Wet` flag** — when checked, `Sw_Ht` values are calculated **below** the
  FWL; unchecked, everything below the FWL is set to wet.
- **`Well Group`** — an integer that links the FWL across wells (e.g. fault
  blocks 1, 2, 3); changing the FWL in one well changes it in all group members.
- **`Hyd Density` / `Gas Density`** may be a **fixed value or a curve**. If a
  curve, the Pc at a level is computed from the **thickness-weighted average of
  the density curve from that level to the contact** (FWL or GOC) — explicitly to
  handle **high-angle wells** where hole angle varies over the calculation
  interval.
- A zone set named **`CapPress FWL`** is auto-created in the well; the **bottom of
  the zone is the FWL** and can be dragged interactively to re-run the model.
  One zone per hydrocarbon model.

**Output curves of the Saturation Versus Height module:**

| Curve | Meaning |
|---|---|
| `Sw_Ht` | Sw as a function of height above FWL |
| `Sw_PcHt` | Sw from the Pc curves, looked up at the calculated Pc for each plug depth above FWL (uses the **corrected** Pc data); empty if no Pc data was entered |
| `HtAbCont` | height above contact, in **well units (ft/m)**, in vertical depth |
| `PcAbCont` | Pc pressure above contact, from vertical depth + hydrocarbon/water densities. **Explicitly does NOT include the IFT correction factor.** |

Pc output units = the corrected-Pc output-curve units set in the Set-Up module,
*"These are the units expected by the functions."*

**Crossplot-format Pc↔Height converter** — a *different* expression appears in
the Function-Xplot format panels of **both** function modules:
```
Height above FWL = Pc / (Water Density - Hydrocarbon Density)
```
(cappressurefunctions.htm and logswversusheightfunctions.htm, identical wording).
**The 0.433 is absent.** See §5, D2.

### 2.7 Hydraulic Flow Units (SPECIAL TASKING (c)) — hfu.htm

Four typing methods: **RQI**, **Winland R35**, **Pittman (14 equations)**,
**Lucia Rock Fabric Number**. Lucia *"does not produce Hydraulic Flow units"* but
is bundled because it uses the same φ–K plots.

Module-level guidance: *"works equally well with core or log porosity,
permeability data. It is recommended not to mix log and core data due to up
scaling resolution problems."*

#### RQI / FZI chain — full Kozeny-Carman derivation RECOVERED

Cited to **SPE 26436**, *"Enhanced Reservoir Description: Using Core and Log Data
to Identify Hydraulic (Flow) units and Predict Permeability in Uncored
Intervals/Wells"* (hfu.htm). Six raster equations, all recovered:

```
        Øe³      ⎡    1     ⎤
k = ───────── × ⎢ ───────── ⎥                     [img-read: _hfuclip0014.png]
    (1 − Øe)²   ⎣ Fs τ² Sgv² ⎦

    ⎧ K      ⎡   Øe    ⎤ ⎡     1      ⎤
   √⎨ ── ⎬ = ⎢ ─────── ⎥ ⎢ ────────── ⎥            [img-read: _hfuclip0015.png]
    ⎩ Øe     ⎣ 1 − Øe  ⎦ ⎣ √Fs τ Sgv  ⎦
                                                   ("Where K = permeability in µm²")

RQI = 0.0314 √( K / Øe )                           [img-read: _hfuclip0016.png]
                                                   ("Where K is in md")

Øz  = [ Øe / (1 − Øe) ]                            [img-read: _hfuclip0017.png]

FZI = [ 1 / (√Fs · τ · Sgv) ]                      [img-read: _hfuclip0018.png]

Log(RQI) = Log(Øz) + Log(FZI)                      [img-read: _hfuclip0019.png]
```

Symbol set: `Øe` = effective porosity; `Fs` = shape factor; `τ` = tortuosity;
`Sgv` = surface area per unit grain volume. The `0.0314` is the µm² → mD
scaling constant introduced between eq. 2 and eq. 3.

ASCII restatement in the same page (hfu.htm), which the rasters corroborate
exactly:
```
Rock quality index      RQI  = 0.0314 x Sqrt( K / Phi )
Pore-Grain volume ratio PhiZ = Phi / (1 - Phi)
Flow Zone Indicator     FZI  = RQI / PhiZ
```
UI panel confirms all three plus default output-curve names
[img-read: _hfuclip0004.png]: `RQI`, `PhiZ`, `FZI`.

Interpretation rule (hfu.htm): *"FZI will be a constant for a flow unit. A
log/log plot of RQI against Phiz will show data in the same flow unit as values
on a straight line with unit slope. The FZI of the flow unit will be the point on
the line when Phiz equal 1."* Confirmed by the crossplot description: *"The FZI
values appear as parallel stripes across the data with unit slope of 1 on a
Log/Log plot."*

#### Permeability-prediction equations emitted per flow unit

The `Output Equations` button writes an IP multi-line-formula parameter set or a
text file. Recovered verbatim from the sample outputs:

**RQI method** [img-read: _hfuclip0010.png] — file `RQI_HFU_Equations.txt`,
header *"'Phi' in decimals"*:
```
HFU n : Perm = Phi^3 * ( FZI_n / (0.0314 * (1.0-Phi)) )^2
```
with the sample's per-unit FZI values:
`HFU 1 → 0.458 | HFU 2 → 1.327 | HFU 3 → 2.382 | HFU 4 → 3.801 | HFU 5 → 6.711`
(these are example mean-FZI values for the demo dataset, **not** defaults).

**Winland method** [img-read: _hfuclip0027.png] — `WinR35_HFU_Equations.txt`,
header *"'Phi' in percent"*:
```
RFU n : Perm = 10^((Log(R35_n) - 0.732 + 0.864*Log(Phi)) / 0.588)
```
sample R35 values `0.462, 2.446, 8.205, 21.635, 41.373`.

**Pittman method** [img-read: _hfuclip0038.png] — `PittR50_HFU_Equations.txt`,
header *"'Phi' in percent"*:
```
HFU n : Perm = 10^((Log(R50_n) - 0.778 + 1.205*Log(Phi)) / 0.626)
```
sample R50 values `0.189, 0.600, 2.125, 5.108, 15.835`. The constants `0.778`,
`1.205`, `0.626` are exactly the R50 coefficients from the Pittman table, so
this is the algebraic inversion of `Log(R50) = 0.778 + 0.626 Log(K) − 1.205
Log(Phi)` — i.e. each of the 14 Pittman equations has its own inversion.

**Lucia method** [img-read: _hfuclip0049.png] — `Lucia_RC_Equations.txt`,
header **`'Phi' in percent`** (see §5, D3 — this contradicts the page prose):
```
RC 1 : Perm = 10^(9.7982 + 8.6711*Log(Phi) - Log(1)   *(12.0838 + 8.2965*Log(Phi)))
RC 2 : Perm = 10^(9.7982 + 8.6711*Log(Phi) - Log(2)   *(12.0838 + 8.2965*Log(Phi)))
RC 3 : Perm = 10^(9.7982 + 8.6711*Log(Phi) - Log(3.25)*(12.0838 + 8.2965*Log(Phi)))
```
The `rfn` substitutions **1, 2, 3.25** are the **midpoints** of the three Lucia
class ranges (0.5–1.5, 1.5–2.5, 2.5–4.0), consistent with the prose: *"These
lines will be positioned at the midpoint of each class not the average class
value as is done for the HFU methods."*

#### Winland R35 (hfu.htm)

```
WinR35    = ALog( 0.732 + 0.588 Log(K) - 0.864 Log(Phi) )      where Phi is in percent
Log (R35) = 0.732 + 0.588 Log(Kair) - 0.864 Log(Phicore)
```
Confirmed on the UI panel [img-read: _hfuclip0021.png]:
`Log(R35) = .732 + .588 Log(K) - .864 Log(Phi)`, default output curve `WinR35`.

Method notes (hfu.htm, own words): created by **Dale Winland of Amoco**; empirical;
`R35` = pore-aperture radius at the **35th percentile of mercury saturation**;
`Kair` = uncorrected air permeability in **md**; `Phicore` in **%**. Derived from
mercury porosimetry on **~300 samples from the Spindle Field, Colorado**. The 35th
percentile was found to give the best correlation and was taken to approximate the
modal pore-throat class at which the pore network becomes interconnected — the
manual immediately qualifies this, noting that strictly this is only true at the
**point of inflexion of the pore-throat-size vs mercury-saturation plot (the "Apex"
plot)**. Winland used **R35 = 0.5 µm as the net-pay cut-off** for Spindle Field
(dry wells R35 < 0.5 µm, producers R35 > 0.5 µm); *"The value of 0.5μm has since
been used in other reservoirs to define pay."*

#### Pittman — all 14 equations (hfu.htm)

Printed twice on the page (selection panel and methodology section); **both
listings are numerically identical** and match IP2018 exactly. `K` in mD,
`Phi` in **percent**.

| Eq | Intercept | Log(K) coeff | Log(Phi) coeff |
|---|---|---|---|
| `Log(R10)` | 0.459 | +0.500 | −0.385 |
| `Log(R15)` | 0.333 | +0.509 | −0.344 |
| `Log(R20)` | 0.218 | +0.519 | −0.303 |
| `Log(R25)` | 0.204 | +0.531 | −0.350 |
| `Log(R30)` | 0.215 | +0.547 | −0.420 |
| `Log(R35)` | 0.255 | +0.565 | −0.523 |
| `Log(R40)` | 0.360 | +0.582 | −0.680 |
| `Log(R45)` | 0.609 | +0.608 | −0.974 |
| `Log(R50)` | 0.778 | +0.626 | −1.205 |
| `Log(R55)` | 0.948 | +0.632 | −1.426 |
| `Log(R60)` | 1.096 | +0.648 | −1.666 |
| `Log(R65)` | 1.372 | +0.643 | −1.979 |
| `Log(R70)` | 1.664 | +0.627 | −2.314 |
| `Log(R75)` | 1.880 | +0.609 | −2.626 |

UI panel confirms the selector and default output-curve naming
[img-read: _hfuclip0032.png]: `Pittman Log(R50) = 0.778 + 0.626 Log(K) - 1.205
Log(Phi)`, output curve `PittR50`.

Citation given: **Pittman, E.D.: "Relationship of Porosity and Permeability to
Various Parameters Derived from Mercury Injection-Capillary Pressure Curves for
Sandstone," AAPG Bull., v. 76, no. 2 (1992b) 191-198.**

Basis: *"around 200 mercury injection PC curves from **sandstone** plugs"*; `Rnn`
= aperture size at nn% mercury saturation. **The Pittman study found an average
of 36% for these sandstones** as the saturation at which the mercury path becomes
continuous.

**Apex-plot → R-equation selection rule** (stated on both hfu.htm and
cappressuresetup.htm): plot HgSat versus HgSat/Pc; the HgSat at the **maximum on
the Y-axis (threshold pressure)** is the saturation at which mercury becomes
continuous, and that selects the Pittman equation. Worked example
(cappressuresetup.htm): *"a 60% Sw seen on the 'Apex' plot, which corresponds to
a 40% mercury saturation, means the Pittman R40 equation should be used."*
Apex plots are produced from the Saturation Height Set-up module.

#### Lucia Carbonate Rock Fabric Number (hfu.htm) — equations RECOVERED

Rock Fabric Number [img-read: _hfuclip0043.png], confirmed on the UI panel
[img-read: _hfuclip0044.png]:
```
            ⎛ 9.7982 + 8.6711 Log(Ø) − Log(K) ⎞
RFN = Alog ⎜ ─────────────────────────────── ⎟
            ⎝ 12.0838 + 8.2965 Log(Ø)        ⎠
```
Prose immediately below the raster: *"Where porosity is in **decimals** and
permeability is in **milli-darcies**."*

Global transform [img-read: _hfuclip0054.png]:
```
Log(K) = 9.7982 − 12.0838 Log(rfn) + ( 8.6711 − 8.2965 Log(rfn) ) Log(Øip)
```
`rfn` = Rock Fabric Number; `Øip` = **interparticle** porosity.

Rock classes (identical wording in both places on the page):
- **Class 1** — Grain-dominated fabrics, Grainstone *(methodology section adds:
  "Grainstones, dolograinstones, and large crystalline dolostones")*. **RFN 0.5 – 1.5**
- **Class 2** — Grain-dominated fabrics, Packstone. **RFN 1.5 – 2.5**
- **Class 3** — Mud-dominated fabrics: Packstone, Wackestone, Mudstone. **RFN 2.5 – 4.0**

Irreducible-saturation equations by class [img-read: _hfuclip0055.png] — the only
saturation-height model in the HFU module:
```
Class 1 :  Swi = 0.02219 × H^(-0.316) × Ø^(-1.745)
Class 2 :  Swi = 0.1404  × H^(-0407)  × Ø^(-1.44)     ← decimal point MISSING in the raster
Class 3 :  Swi = 0.611   × H^(-0.505) × Ø^(-1.21)
```
`H` = **height above the FWL in feet**; `Ø` = **interparticle porosity in v/v**
(hfu.htm). The Class 2 exponent is reported exactly as printed — see §8, OPEN-2.

Swi calculation controls (hfu.htm): `Output Rock Class Swi Curve` checkbox; the
user enters the **free water level depth in well depth units (feet or meters)**;
IP looks up the **TVDSS depth at each level from the well header (Position tab)**
and errors if that information is missing. The height above FWL is also output as
a curve, in the depth units of the well. UI defaults
[img-read: _hfuclip0051.png]: `Lucia_RC`, `Lucia_RC_txt`, `Lucia_Swi`,
`Ht_above_FWL`, and the FWL box is labelled **`ft`**.

Citations given: **Jerry Lucia, *Carbonate Reservoir Characterization*, 2007,
Springer**; and **SPE 84942, "Predicting Permeability from Well Logs in Carbonates
With a Link to Geology for Interwell Permeability Mapping", August 2003, James W.
Jennings Jr and F. Jerry Lucia.** Method note: *"He plotted the core data Porosity
v. Permeability for each class and fit the data using a **RMA regression**. He
combined the equations into a global transform using the boundaries of the
petrophysical rock classes."*

#### Clustering / binning methods

Identical wording for RQI, Winland and Pittman (hfu.htm). Two options for the
initial flow-unit boundary selection:

1. **Equal logarithmic spacing** between the minimum and maximum of the indicator
   curve (FZI / WinR35 / Pittman).
2. **Cluster analysis** — *"The data is clustered into **25 nodes** using **K-mean
   clustering**. Then the cluster nodes are re-clustered into the number of
   selected flow units using **Hierarchical clustering**."*

The `25` is fixed and identical for all three methods. In both cases *"The low
values of the first flow unit and the high value of the last flow units are
**extended to cover data beyond the dataset currently selected**"* — i.e. the
outer bins are open-ended.

Two interactive boundary editors:
- **Histogram** — vertical red boundary lines, draggable.
- **Lorenz plot** — data **reverse-sorted on the indicator value** (FZI / WinR35 /
  Pittman), then **linearly accumulated and normalised to a maximum of 1.0**.
  X = cumulative porosity ("Cumulative Storage Capacity"), Y = cumulative
  permeability ("Cumulative Flow Capacity"), Z = the indicator as a colour scale.
  Boundaries are picked at **inflection points**; `Draw Linear Segments` assists.
  Axis-expansion options plot X from **−0.1 to 1.0** and Y from **0.0 to 1.1**.

Lucia has **no** clustering step — its class boundaries default to Lucia's own
RFN ranges, restorable via `Set Defaults`, and only a **histogram** editor (no
Lorenz plot).

`Re-initialize HFU's` is required to regenerate default boundaries once they have
been set — after the first pass, `Make … Curves` no longer moves the boundaries,
so extra wells can be added without disturbing an agreed scheme.

---

## 3. Parameters, defaults & constraints

### 3.1 Numeric defaults, with units and source

| Parameter | Default | Units | Module / source |
|---|---|---|---|
| Contact Angle Laboratory — Mercury Injection | **140** | deg | [img-read: _shmclip0009.png] |
| Interfacial Tension Laboratory — Mercury Injection | **480** | dynes/cm | [img-read: _shmclip0009.png] |
| Contact Angle Laboratory — Centrifuge | **0** | deg | [img-read: _shmclip0009.png] |
| Interfacial Tension Laboratory — Centrifuge | **72** | dynes/cm | [img-read: _shmclip0009.png] |
| Contact Angle Laboratory — Porous Plate | **0** | deg | [img-read: _shmclip0009.png] |
| Interfacial Tension Laboratory — Porous Plate | **72** | dynes/cm | [img-read: _shmclip0009.png] |
| Contact Angle Reservoir (table default, all methods) | **30** | deg | [img-read: _shmclip0009.png] |
| Interfacial Tension Reservoir (table default, all methods) | **30** | dynes/cm | [img-read: _shmclip0009.png] |
| Method ID — Mercury Inj / Centrifuge / Porous Plate | **1 / 2 / 3** | — | [img-read: _shmclip0009.png] |
| `Set Gas / Water` → θ_Res, σ_Res | **0 deg**, **50 dynes/cm** | | cappressuresetup.htm; [img-read: _shmclip0010.png] |
| `Set Oil / Water` → θ_Res, σ_Res | **30 deg**, **30 dynes/cm** | | cappressuresetup.htm; [img-read: _shmclip0010.png] |
| `Use Dry Rock Reservoir Properties` → θ_Res, σ_Res | **0 deg**, **1 dynes/cm** | | cappressuresetup.htm; [img-read: _shmclip0084.png] |
| Pc Height — Water Density | **1** | gm/cc | [img-read: _shmclip0011.png] |
| Pc Height — Hyd. Density | **0.7** | gm/cc | [img-read: _shmclip0011.png] |
| Pc Height — Fluid Units | **gm/cc** | — | [img-read: _shmclip0011.png] |
| Default Closure Correction Method | **Shift** | — | [img-read: _shmclip0167.png] |
| Fresh-water gradient constant | **0.433** | psi/ft | cappressuresetup.htm, saturation_versus_height_curve.htm (see §5 D9) |
| Leverett-J constant | **0.2166** | — | cappressurefunctions.htm; [img-read: _shmclip0028.png] |
| Leverett-J porosity modifier `m` | **2.8** | — | cappressurefunctions.htm; [img-read: _shmclip0029.png] |
| RQI constant (HFU + Cap-Pressure-Functions modules) | **0.0314** | md→µm² scaling | [img-read: _hfuclip0016.png], [img-read: _hfuclip0004.png], [img-read: _shmclip0181.png] |
| RQI constant (Log Sw vs Height module) | **absent (1.0)** | — | logswversusheightfunctions.htm — see §5 D1 |
| Clay correction constants | **0.6425**, **0.22** | — | cappressuresetup.htm |
| Swanson — Gas All Rocks | **399**, exp **1.691** | — | cappressuresetup.htm |
| Swanson — Liquid Carbonates | **431**, exp **2.109** | — | cappressuresetup.htm |
| Swanson — Liquid Sands | **290**, exp **1.901** | — | cappressuresetup.htm |
| Swanson — Liquid All rocks | **355**, exp **2.005** | — | cappressuresetup.htm |
| Winland R35 coefficients | **0.732 / 0.588 / −0.864** | — | hfu.htm; [img-read: _hfuclip0021.png] |
| Winland net-pay R35 cut-off | **0.5** | µm | hfu.htm (Spindle Field; noted as reused elsewhere) |
| Pittman — 14 equations | see §2.7 table | — | hfu.htm (printed twice, identical) |
| Pittman study mean continuity saturation | **36 %** | Hg sat | hfu.htm |
| Lucia RFN numerator constants | **9.7982**, **8.6711** | — | [img-read: _hfuclip0043.png] |
| Lucia RFN denominator constants | **12.0838**, **8.2965** | — | [img-read: _hfuclip0043.png] |
| Lucia class boundaries (RFN) | **0.5–1.5 / 1.5–2.5 / 2.5–4.0** | — | hfu.htm |
| Lucia class midpoints used for perm lines | **1 / 2 / 3.25** | RFN | [img-read: _hfuclip0049.png] |
| Lucia Swi Class 1 | **0.02219**, H exp **−0.316**, Ø exp **−1.745** | — | [img-read: _hfuclip0055.png] |
| Lucia Swi Class 2 | **0.1404**, H exp **−0407** (sic), Ø exp **−1.44** | — | [img-read: _hfuclip0055.png] |
| Lucia Swi Class 3 | **0.611**, H exp **−0.505**, Ø exp **−1.21** | — | [img-read: _hfuclip0055.png] |
| K-mean cluster nodes (all 3 HFU methods) | **25** | nodes | hfu.htm |
| `Number of flow units` | **5** | — | [img-read: _hfuclip0004.png], [img-read: _hfuclip0021.png], [img-read: _hfuclip0032.png] |
| Pore Size / Throat Size array length | **80** | elements | cappressuresetup.htm |
| Pore-throat radius range | **0.01 → 100** | microns | cappressuresetup.htm |
| Pore Size log resolution | **20** | values per decade | cappressuresetup.htm |
| `Pc Normalized` array length | **51** | cells | cappressuresetup.htm |
| `Pc Normalized` cell width | **2.0** | saturation units | cappressuresetup.htm |
| Regression weighting | **1/Sw**, capped at **10** | — | cappressurefunctions.htm, logswversusheightfunctions.htm |
| Function-mixing discriminator grid | **15** lines default, **1000** max | rows | saturation_versus_height_curve.htm |
| IFT-fitting search range | **0.1 to 5** (logarithmic) | factor | saturation_versus_height_curve.htm |
| Log Sw vs Height — models rated by comparator | **32** | models | logswversusheightfunctions.htm |
| User-Defined Equation capacity (Pc route) | **7** curves, **7** coefficients; **≤4** regressable in the "Separate equation" dropdown variant | — | cappressurefunctions.htm |
| User-Defined Equation capacity (Log route) | **7** curves, **7** coefficients | — | logswversusheightfunctions.htm |
| Cap Pressure Data Loader — max plugs | **100** | plugs/session | capillarypressuredataloader.htm |
| Data Loader — `No. of Curves` | **5** | — | capillarypressuredataloader.htm |
| Data Loader — `No. of Text Curves` | **2** | — | capillarypressuredataloader.htm |

Screenshot-only example values (**not defaults** — do not adopt):
`PhiRes/PhiLab Factor 0.9562`, `Salinity 30 Kppm NaCl` [img-read: _shmclip0106.png];
`0.95` in the truncated variant [img-read: _shmclip0009.png]; the demo FZI/R35/R50
flow-unit values in §2.7; the demo report's `Water Density 1.12 gm/cc`,
`Hyd. Density 0.4 gm/cc`, `IFT Corr. Factor 2.27`
(saturation_versus_height_curve.htm).

### 3.2 Unit conventions (hard constraints)

| Quantity | Accepted input | Internal working unit | Source |
|---|---|---|---|
| Capillary pressure | **psi, Bar, Kg/m², MPa** | **psi** — *"IP converts Pc pressure values to psi based on the input units"* | cappressuresetup.htm |
| Pc saturation | **% or dec** | **decimal** | cappressuresetup.htm |
| Core porosity | **% or dec** | **decimal** | cappressuresetup.htm |
| Core permeability | **mD or m²** | **mD** — *"IP will convert the values to milliDarcies for all computations"* | cappressuresetup.htm |
| Default output units | Pc: **psi**; Pc Sat: **decimal** | | cappressuresetup.htm |
| Contact angle | **degrees** everywhere | | cappressuresetup.htm, saturation_versus_height_curve.htm |
| Interfacial tension | **dynes/cm** everywhere | | as above |
| Qv | **meq/ml** | | cappressuresetup.htm |
| Salinity | **Kppm NaCl equivalent** | | cappressuresetup.htm |
| Log Sw / Porosity / BVW inputs (Log route) | **decimals only** — *"this is the standard output for all IP interpretation modules"* | | logswversusheightfunctions.htm |
| Log permeability (Log route, RQI) | **mD or m²** → converted to **mD** | | logswversusheightfunctions.htm |
| Sat-vs-Height module φ and Sw inputs | **decimal** | | saturation_versus_height_curve.htm |
| Winland `Phi` | **percent** | | hfu.htm (×3 statements) |
| Pittman `Phi` | **percent** | | hfu.htm |
| Lucia `Phi` | **decimals** (prose) / **percent** (equation file header) | ⚠ conflict — §5 D3 | hfu.htm vs [img-read: _hfuclip0049.png] |
| Lucia `H` | **feet** | | hfu.htm |
| Height / `HtAbCont` output | **same units as the database depth curve** (ft or m) | | cappressuresetup.htm, saturation_versus_height_curve.htm |
| FWL / GOC entry | **TVDSS** | | saturation_versus_height_curve.htm |
| Pore Size curve | **unitless fraction**, Σ(80 elements) = **1** | | cappressuresetup.htm |

---

## 4. Assumptions & validity limits

1. **Two-phase, two-variable model.** *"Capillary pressure curves can be defined
   for any 2 phase fluid system in a given rock. The only variables are the
   Contact Angle (θ) and the Interfacial tension (σ)."* This is the entire
   theoretical basis for normalising across measurement methods and converting
   lab→reservoir. Grain size, shape, sorting and cementation are acknowledged as
   controls but are **not** in the conversion (cappressuresetup.htm).
2. **The lab→reservoir conversion corrects Pc only, never porosity** —
   *"These values will be used to convert laboratory measured capillary pressures
   to a Reservoir fluid system and not the core porosity"* (cappressuresetup.htm,
   stated twice).
3. **Core porosity must already be overburden-corrected**, *"as the function
   generated will be used with log porosity"* (cappressuresetup.htm).
4. **Core permeability overburden correction must MATCH the log side** —
   *"should be overburden corrected only if used with log data which has generated
   an overburden corrected permeability. I.e. log and core data should be the same
   with respect to overburden correction"* (cappressuresetup.htm).
5. **Multiple Sw = 100% points are destructive to curve fitting** — hence the
   automatic nulling of all but the last (cappressuresetup.htm, stated twice).
6. **Closure correction is not deterministic** — *"There is no definitive way of
   calculating the closure correction and it has to be done visually by looking
   closely at each Pc curve"* (cappressuresetup.htm).
7. **Double (bimodal) Pc curves must be excluded** — *"If for example, you think a
   Pc Curve is a Double curve, i.e. a composite curve composed of two curves, each
   representing one pore network, it should be excluded from your dataset"*
   (cappressuresetup.htm).
8. **QC one plug at a time** — *"if multiple points are on top of each other then
   the Remove / Restore data point will only change the top point"*
   (cappressuresetup.htm).
9. **Only the `Pc v Sw` / Z-axis-Use-Flag crossplot can edit plug data.** The Phi
   and Perm crossplots *"ignore the PcUse curves and therefore are unable to change
   their values"* (cappressuresetup.htm). On a Use-Flag Z-axis plot, *"the point
   will appear as bad if **any** of the points in the PC curve are bad."*
10. **Do not mix log and core φ/K in the HFU module** — up-scaling resolution
    mismatch (hfu.htm).
11. **Do not mix feet and metre wells in one Log-Sw-vs-Height function** unless the
    TVDSS inputs have been pre-converted to consistent units
    (logswversusheightfunctions.htm).
12. **Sparse core curves are invalid input to the Saturation Versus Height module**
    — *"continuous porosity / permeability input log curves, calibrated to core data
    (where available), are the required input types. Sparse data Core Porosity and /
    or Core Permeability curves are not suitable for use in this module"*
    (saturation_versus_height_curve.htm).
13. **Function mixing requires a consistent curve-naming convention across all
    selected wells**; a discriminator curve must exist in every well
    (saturation_versus_height_curve.htm, and the same for HFU discriminators and
    User-Defined-Equation key-word curves).
14. **Non-linear regression is seed-dependent.** *"It is possible that several
    regression results, equally good, could be found for the same function. The
    starting coefficients will be what decide which solution is found."* Guidance:
    check with `View Function` — if the function does not plot on screen, the
    regression has not converged sensibly (cappressurefunctions.htm,
    logswversusheightfunctions.htm).
15. **Fixing a coefficient far from its free value gives a very poor fit**; the
    recommended workflow is free → inspect coefficient-vs-curve correlations →
    fix one → re-run → repeat (cappressurefunctions.htm).
16. **The `Pc Normalized` transform is explicitly approximate** — *"approximate due
    to the coarseness of the cells"* (51 cells × 2 saturation units)
    (cappressuresetup.htm).
17. **`PcAbCont` excludes the IFT correction factor** while the Pc actually used
    inside the functions includes it (saturation_versus_height_curve.htm). Two
    different Pc quantities coexist in the outputs.
18. **Below-FWL behaviour is a switch, not physics** — the `Oil Wet` flag decides
    whether Sw is modelled below the FWL or forced to 1
    (saturation_versus_height_curve.htm).
19. **Rock Class values must be initialised before a Rock-Class-Z-axis crossplot**,
    otherwise *"the PC curves will not show up on the crossplot"*
    (cappressuresetup.htm).
20. **Depth Tolerance is a silent overwrite risk** — two Pc plugs closer than the
    tolerance are treated as the same depth and *"the Pc / Sw data for the second
    plug will be considered to be at the same depth as that for the first plug and
    **will overwrite** the data for plug 1"* (capillarypressuredataloader.htm).
21. **Excel loader silently prefers a pre-converted file** — if an `.xls` was
    previously auto-converted, the sibling `.xlsx` is loaded instead of the file
    the user selected (a warning is shown) (capillarypressuredataloader.htm).
22. **`Save` does not save curves** — the Set-Up module's `Save` button writes only
    parameters; output curves require an explicit `Save All Wells to Database`
    (cappressuresetup.htm, emphasised in capitals).
23. **Destructive UI actions**: clicking the `Plug Good` column *header* resets every
    QC edit for the well; `Clear All` discards all Pc curve edits; `Null output Sw
    Curves` in the Sat-vs-Height module *"will delete the results from other
    models"* (cappressuresetup.htm, saturation_versus_height_curve.htm).
24. **A generic-input-curve discriminator is required for multi-Set work** — because
    each input Pc curve Set has a different curve name, *"discriminating multiple
    input Sets is now impossible"* with named curves (cappressurefunctions.htm).
25. **IP v3.4 back-compatibility switch**: `Output Sets same as PC curve Input sets`
    defaults ON for new projects, but is OFF for projects previously run under IP
    v3.4, in which case output curves get **no numeric suffix** — the mechanism that
    otherwise prevents same-depth plugs overwriting each other
    (cappressuresetup.htm).
26. **`Use Zones` requires the selected zones to exist in every well** (hfu.htm,
    cappressurefunctions.htm).

---

## 5. Internal discrepancies

**D1 — `RQI` means two different things in two modules. (HIGHEST SEVERITY)**
- Hydraulic Flow Units module: `RQI = 0.0314 x Sqrt(K/Phi)`
  (hfu.htm; [img-read: _hfuclip0016.png], [img-read: _hfuclip0004.png])
- Capillary Pressure Functions module (Normalised-J): `RQI = 0.0314 √(K/Φ)`
  ([img-read: _shmclip0181.png])
- **Log Sw versus Height module: `RQI = √(K/ϕ)` and `RQI = √(K/ϕ^m)` — no 0.0314**
  (logswversusheightfunctions.htm)

Same acronym, same inputs, scaling differs by a factor of **0.0314** (≈31.8×).
The IP2018 ingest flagged this identically (`E_shf_rocktyping.md` line ~749), so
it has survived at least one release. Any Swirr-vs-RQI or Sw = f(RQI·h) coefficient
is **not portable between the two modules**.

**D2 — Two incompatible Pc↔Height conversions.**
- Set-Up module and Saturation-versus-Height module:
  `Height = Pc / [0.433 (ρWater − ρHc)]` / `Pc = h * 0.433 (ρWater − ρHc) * IFTCorr`
- Function-Xplot format panel, **both** function modules:
  `Height above FWL = Pc / (Water Density − Hydrocarbon Density)` — **no 0.433**
  (cappressurefunctions.htm, logswversusheightfunctions.htm)

A crossplot height axis built with the second form is **wrong by a factor of
1/0.433 ≈ 2.31** relative to the computed `HtAbCont`. The 0.433 form is the one
with the full derivation and the one used in the compute path; the crossplot form
appears to be an abbreviation. Identical in IP2018.

**D3 — Lucia porosity units contradict themselves.**
- hfu.htm prose, immediately under the RFN raster: *"Where porosity is in
  **decimals** and permeability is in milli-darcies."*
- hfu.htm prose, Output Equations section: *"the Lucia equations use porosity in
  **decimals** so the input porosity curve may need **dividing by 100**."*
- **The emitted equation file's own header reads `'Phi' in percent`**
  [img-read: _hfuclip0049.png], and the file body uses the same
  `9.7982 / 8.6711 / 12.0838 / 8.2965` constants.

Both cannot be right: with those constants, swapping decimal↔percent moves
predicted permeability by many orders of magnitude. The two prose statements
agree with each other and with the Lucia source convention (φip in v/v), so the
**text-file header is the likely error** — but this is *not* certain, and it is
exactly the class of silent-wrongness failure that must not be guessed. Carried
to §8, OPEN-3.

**D4 — Screenshot row labels shifted in the σ/θ table.**
[img-read: _shmclip0167.png] labels the three rows **`Default` / `Mercury Inj` /
`Centrifuge`** while carrying the *same* numbers (140/480, 0/72, 0/72) that
[img-read: _shmclip0009.png] and [img-read: _shmclip0084.png] label
**`Mercury Inj` / `Centrifuge` / `Porous Plate`**. Two of three screenshots and
all prose agree on three methods; `_shmclip0167` is the outlier. Treat
`_shmclip0009` as authoritative. If `_shmclip0167` reflects a real newer UI with
a leading `Default` row, then in that build the Mercury row would carry 0°/72
dynes/cm — physically wrong for mercury — which supports reading it as a
screenshot artefact.

**D5 — Correction order vs. correction formulae.** `Run Corrections` step 1
converts saturations to **non-wetting** phase; steps 4–5 apply the stress and clay
corrections; step 6 converts **back** to wetting phase. But the stress and clay
formulae are both written in terms of `SwPc` = "input Pc saturation (**wetting
phase**) curve" and produce `SwPcCorr` = "Stress/Clay Corrected Pc Saturation
(**wetting phase**)" (cappressuresetup.htm). The formulae's stated phase
contradicts the pipeline's stated phase. Note both formulae have the structural
form `1 − (1 − Sw)·F`, i.e. they scale the **non-wetting** saturation `(1 − Sw)`
by `F` — consistent with the pipeline and inconsistent with the labels. The labels
are the likely error, but this is reported, not resolved.

**D6 — `Restore Defaults` scope.** Prose: *"clicking the Restore Defaults button
will reset the values in the **first 3 columns** of the table."* The first three
columns are Measurement Type, Method ID and Contact Angle Laboratory — which would
leave Interfacial Tension Laboratory un-restored. The button is captioned
`Restore Defaults` in [img-read: _shmclip0009.png]/[img-read: _shmclip0010.png]
but **`Restore Lab Defaults`** in [img-read: _shmclip0084.png] and
[img-read: _shmclip0167.png], suggesting the intent is "all four laboratory
columns". Unresolved.

**D7 — Clay-correction bracket imbalance.** `F = 1 - [0.6425 * ( Salinity ^ (-0.5)
+ 0.22 ] * Qv ]` — three closing delimiters, two opening. Not evaluable as
written. Byte-identical in IP2018 and IP2025 (verified). See §8, OPEN-1.

**D8 — "32 models" arithmetic.** logswversusheightfunctions.htm states the
comparator rates **32 models**. The dropdowns enumerate **8 methods × 7 regression
equations = 56** combinations [img-read: _shmclip0073.png, _shmclip0074.png]. The
four "Porosity & Height" methods carry their own built-in fitted forms and
plausibly do not cross with the 7 regressions (4 × 7 = 28, leaving 4 methods ×
7 = 28 + 4 = 32 ✓ if the four Porosity & Height methods each count once). That
reading is arithmetically consistent but **is an inference, not a statement of the
manual**.

**D9 — The `0.433` constant is not the constant IP actually uses.** The worked
report in saturation_versus_height_curve.htm gives
`Water Density 1.12 gm/cc`, `Hyd. Density 0.4 gm/cc`, `IFT Corr. Factor 2.27`
and the resulting compiled expression `Pc = (7622 - TVDSS) * 0.70856 psi`.
Back-solving: `0.70856 / (0.72 × 2.27) = 0.43352`. Using the documented `0.433`
gives `0.70770` — a **0.12 % discrepancy**. `0.43352` is the exact fresh-water
gradient (62.428 lb/ft³ ÷ 144 in²/ft² = 0.43353 psi/ft). **Inference, clearly
labelled as such**: IP's internal constant appears to be the unrounded
≈`0.43353 psi/ft`, and `0.433` is a display rounding in the documentation. Small,
but systematic and one-directional in every height calculation.

**D10 — Spelling split.** Prose uses British "**Normalised J**"; the UI dropdown
reads "**Normalized J (RQI)**" etc. [img-read: _shmclip0180.png]. Cosmetic, but it
matters for any string-matching parameter reader.

**D11 — `Entry Correction` / `Closure Corr.` curve naming.** The Data View/Edit
grid column `Entry Correction` is described as *"picked up from the input closure
curve **PcClosure**"* while the Curves Set-up tab names the output
`Closure Correction Curve Out` (cappressuresetup.htm). Two names, one quantity.

**D12 — Thomeer listed under both model types.** Thomeer appears both in the
"One Equation for all Pc curves" `Method` list and in the "Separate equation for
each Pc curve" `Regression Equation` list (cappressurefunctions.htm). Not an
error, but the manual also says that when using the four "One Equation" functions
*"no Regression line function is created"* when analysing with the crossplot tool —
so the same equation behaves differently depending on where it was selected. Also
flagged by the IP2018 ingest.

---

## 6. IP2018 numeric diff

Method: HTML stripped to text for both `c18` and `c25`, all decimal literals
extracted and set-differenced per page; then keyword-count structural diff.

### 6.1 Pages with ZERO numeric change

| Page | c18 decimals | c25 decimals | Diff |
|---|---|---|---|
| `cappressurefunctions` | 44 | 44 | **none** |
| `hfu` | 53 | 53 | **none** |
| `logswversusheightfunctions` | 5 | 5 | **none** |
| `capillarypressuredataloader` | 0 | 0 | none |
| `saturation_height_modelling` | 0 | 0 | none |

**Every constant in the HFU module is bit-identical between IP2018 and IP2025**:
all 14 Pittman coefficients, Winland `0.732 / 0.588 / 0.864`, RQI `0.0314`, Lucia
`9.7982 / 8.6711 / 12.0838 / 8.2965`, the 25-node K-means, the Lucia class
boundaries. Likewise Leverett `0.2166`, `m = 2.8`, and every regression form on
the Functions page. Structural keyword counts for `hfu` are identical across all
9 probes (Lucia 21/21, Pittman 23/23, Winland 14/14, Lorenz 25/25, cluster 22/22,
Swi 5/5, R10 3/3, R75 3/3).

### 6.2 `cappressuresetup` — real changes

| Literal | IP2018 | IP2025 | Meaning |
|---|---|---|---|
| `1.691`, `1.901`, `2.005`, `2.109` | absent | present | **Swanson permeability is entirely NEW.** Keyword count `Swanson` c18 = **0**, c25 = **12**. All four correlation models (399/431/290/355 with these exponents) are new content. |
| `0.1` | present | absent | IP2018's `Throat Size Curve Out` said *"The first value is **0.1** (microns)"* while its own `Pore Size Curve Out` said `0.01` with 20 values/decade over 80 elements. **IP2025 corrects the Throat Size text to `0.01`**, making it self-consistent (0.01→100 = 4 decades × 20 = 80 elements ✓). |
| `3.5` | present | absent | IP2018 carried a worked closure-correction table (`Closure Correction = 5 %`, `Entry Pressure = 3.5 psi`). **Removed in IP2025** and replaced by the 4-method scheme. |
| `0.3`, `1.0` | absent | present | New `Output defined PC curves` feature (example curve name `PC_0.3`; output clipped `> 0` and `< 1.0` for Sw). Keyword `Output defined` c18 = 0, c25 = 2. |

**Closure-correction formula CHANGED — semantics of the stored parameter moved:**

| | IP2018 | IP2025 |
|---|---|---|
| Formula | `SwCorr = Sw(input) + SwClosure` | `SwCorr = Sw(input) + (1 − SwClosure)` (Shift method) |
| Meaning of `SwClosure` | the **correction magnitude** (worked example: 5 %) | the **Sw at the picked closure point** (so 1 − 0.95 = 0.05) |
| Methods offered | one | **four**: Shift / Proportional / Crop / Extrapolate (default **Shift**) |
| Clipping | *"limited to be between 0 and 100 % Sw"* | not restated; instead entry pressure is clipped to plug Pc values |
| Keyword counts | `Closure` 38, `Extrapolate` 0, `Proportional` 0 | `Closure` 57, `Extrapolate` 5, `Proportional` 3 |

**This is the single most consequential version change on these pages.** A
`Closure Correction` curve carried forward from an IP2018 project stores a
*different quantity* than IP2025 expects.

Unchanged on this page: the clay-correction string (byte-identical, including the
bracket typo), `0.6425`, `0.22`, the Hill/Shirley/Klein citation, `Dry Rock`
(3/3), `Rock Class` (24/24), `Apex` (4/4), `Pc Normalized` (1/1).

### 6.3 `cappressurefunctions` — structural additions (no numeric change)

| Keyword | c18 | c25 |
|---|---|---|
| `Normalised J` | 0 | 4 |
| `RQI` | 0 | 8 |
| `FZI` | 0 | 4 |
| `Swirr` | 0 | 22 |

**The entire Normalised-J workflow is NEW in IP2025** — including the
`RQI = 0.0314 √(K/Φ)` raster, the `SwN`/`Swirr` split, the `SWirr Curve In` input,
and the four `Normalized J (RQI|FZI|Phi|Perm)` dropdown entries. It introduced no
new numeric literal into the text stream because all of its constants live in
rasters (which the decimal-diff cannot see) — worth noting as a limitation of the
diff method. Unchanged: `User Defined` 4/4, `Skelt` 1/1, `Brooks Corey` 3/3,
`Thomeer` 4/4, `Regression Comparison` 1/1, `3D Plot` 2/2.

Brooks-Corey equation image renamed `embim460.gif` (IP2018) → `embim288.png`
(IP2025); the equation itself is unchanged.

### 6.4 `saturation_versus_height_curve` — structural additions

All 12 IP2018-only and 9 IP2025-only decimals are **worked-example report numbers
from a different demo well**, not method constants. IP2018's demo:
`0.1174, 0.2, 0.28661, 0.3406, 0.43881, 0.5, 0.74781, 1.0, 1.14, 1.29659,
3.59475, 3128.8`. IP2025's demo (well "Experior"): `0.01, 0.02016, 0.31459, 0.4,
0.70856, 1.11468, 1.12, 2.27, 2.27063`. **No method constant changed.**

| Keyword | c18 | c25 | Change |
|---|---|---|---|
| `Gas Oil Contact` | 2 | 7 | expanded gas-over-oil treatment |
| `Gas Free Water` | 0 | 3 | **NEW** `TVD Gas Free Water Level` parameter + GFWL↔GOC linking |
| `Petrel` | 0 | 5 | **NEW** Petrel-friendly equation export (`pow(Pc, …)` form) |
| `Pore Size` | 0 | 1 | **NEW** pore-size-distribution track (track 7) on the interactive plot |
| `Oil Wet`, `Well Group`, `IFT Corr` | 1/1/5 | 1/1/5 | unchanged |

### 6.5 Recovery scorecard against the IP2018 ingest's OPEN ITEMS

`E_shf_rocktyping.md` listed these as unrecoverable. Status now:

| IP2018 OPEN item | Status |
|---|---|
| 1. Lab Contact Angle / IFT defaults for Mercury Inj / Centrifuge / Porous Plate | ✅ **RECOVERED** — 140°/480, 0°/72, 0°/72 [img-read: _shmclip0009.png] |
| 2. The lab→reservoir Pc conversion equation | ✅ **RECOVERED** — `Pc,Res = Pc,Lab × ABS[(σRes·cosθRes)/(σLab·cosθLab)]` [img-read: clip1181.png] |
| 3. Porosity & Pc functions 1 / 2 / 3 (screenshot only) | ✅ **RECOVERED** — all three, plus Porosity & Pc Lambda [img-read: _shmclip0030–0033.png] |
| 4. Kozeny-Carman derivation steps | ✅ **RECOVERED** — all six [img-read: _hfuclip0014–0019.png] |
| 5. Lucia RFN equation | ✅ **RECOVERED** [img-read: _hfuclip0043.png, _hfuclip0054.png] |
| 5b. Lucia Swi-by-rock-class equations | ✅ **RECOVERED** (Class 2 exponent has a vendor typo — §8 OPEN-2) [img-read: _hfuclip0055.png] |
| 6. Brooks-Corey `embim460.gif` | ✅ **RECOVERED** as `embim288.png` — matches the ASCII form exactly |
| — | ➕ **BONUS**: pore-throat radius equation, RQI_cp equation, Swanson panel, all four HFU permeability-inversion equation files |

---

## 7. SandiBumi notes

1. **Namespace `RQI` from day one.** D1 is a live trap: `RQI_hfu = 0.0314√(K/φ)`
   vs `RQI_logsh = √(K/φ)`. If SandiBumi ever imports an IP-fitted
   `Swirr = c·RQI^d` or `Sw = f(RQI·h)`, the coefficients silently mean different
   things depending on which IP module produced them. Store the scaling constant
   *alongside* the coefficients, never implicitly.

2. **The σ/θ defaults table is now citable** and is the single highest-value item
   recovered here. Mercury Inj **140° / 480 dynes/cm** (lab), Centrifuge and
   Porous Plate **0° / 72 dynes/cm** (lab), reservoir default **30° / 30
   dynes/cm**. These trace to `cappressuresetup.htm` (IP 2025) and can be cited as
   a documented vendor source under the project's no-invented-parameters rule.
   Note `|σcosθ|_Hg = 367.7` vs `72.0` for the aqueous methods — a factor of 5.1
   in the normalisation, so getting the method flag right per plug is not optional.

3. **Take the `ABS` in the lab→reservoir conversion literally.** With θ_lab = 140°,
   `cos θ` is negative; without `ABS` every mercury-derived Pc flips sign.

4. **Use 0.43353 psi/ft, not 0.433, and say so.** D9 shows IP's own worked example
   is consistent with the unrounded gradient. The 0.12 % bias is small but
   systematic and free to avoid. Flag this as an inference from the worked example,
   not a stated constant, wherever it is recorded.

5. **Closure-correction parameter semantics are version-dependent** (§6.2). Any
   SandiBumi importer that reads an IP `Closure Correction` curve must know which
   IP release wrote it, or it will apply a 0.95 shift where a 0.05 was meant.
   Recommend refusing to import the curve without a version tag.

6. **Do not implement the Lucia permeability transform until D3 is resolved.**
   Decimal-vs-percent on `Log(Phi)` with those constants is an orders-of-magnitude
   error, and it is exactly the silent-wrongness class the project rules name.
   Resolve against Lucia 2007 / SPE 84942 before shipping.

7. **The clay correction cannot be implemented from this manual alone** (D7 /
   OPEN-1). Get the form from Hill, Shirley & Klein 1979 (SPWLA Paper AA) — the
   manual cites it explicitly, so the primary source is identified.

8. **Two Pc quantities must be kept distinct in the data model**: `PcAbCont`
   (no IFT correction) and the Pc actually fed to the functions (with
   `IFTCorrFactor`). Conflating them changes every modelled Sw.

9. **The FWL search and IFT search are the same algorithm with different axes** —
   brute-force sweep, scoring on Σ|difference| of porosity-weighted hydrocarbon
   volume (default) or raw Sw, minimum wins. IFT range defaults `0.1–5`
   logarithmic. Cheap to replicate and genuinely useful; note that both fit a
   *single* scalar against log Sw, so they inherit whatever bias the log Sw carries.

10. **Permeability-prediction inversions are free wins.** All four are recovered
    verbatim in §2.7 and are pure algebra over the forward equations. The unit
    conventions differ per method (RQI decimals, Winland percent, Pittman percent,
    Lucia disputed) — carry the unit as data, not as a comment.

11. **No citation exists for Swanson, Leverett-J, Thomeer, Brooks-Corey,
    Skelt-Harrison, Lambda, or the four Porosity & Pc functions.** Winland,
    Pittman, Lucia, Hill/Shirley/Klein and SPE 26436 *are* cited. If SandiBumi's
    method notes require a traceable source per method, the uncited group needs
    primary-literature sourcing before it can ship as documented.

12. **Nothing on these seven pages mentions smectite, montmorillonite, illite,
    kaolinite or chlorite** (verified by scan across all 7 pages). No clay
    endpoints to capture from this slice.

13. **Interoperability detail worth copying**: IP emits a "Petrel-friendly"
    variant of every model equation alongside its own (`a * pow(Pc, b)` instead of
    `a * Pc^b`), except where the two are identical. Cheap, and it is the kind of
    thing that makes a tool adopted rather than admired.

---

## 8. OPEN ITEMS

**OPEN-1 — Clay-correction factor `F` is not evaluable as printed.**
Literal string (identical in IP2018 and IP2025):
`F = 1 - [0.6425 * ( Salinity ^ (-0.5) + 0.22 ] * Qv ]`
Bracket sequence `[ ( ] ]` is unbalanced. Two readings are grammatically
reachable:
(a) `F = 1 − [ (0.6425 · S^(−0.5) + 0.22) · Qv ]`
(b) `F = 1 − [ 0.6425 · (S^(−0.5) + 0.22) · Qv ]`
These differ materially — (a) puts `0.22` outside the `0.6425` scaling, (b) inside.
**Not resolved here; do not guess.** Resolve against Hill, Shirley & Klein 1979,
SPWLA 20th Annual Symposium Paper AA — cited by name on the page.
Source: cappressuresetup.htm.

**OPEN-2 — Lucia Class 2 height exponent has a missing decimal point.**
The raster literally reads `Sw_i = 0.1404 × H^(-0407) × Ø^(-1.44)`
[img-read: _hfuclip0055.png, verified at 4× upscale]. Classes 1 and 3 use
`-0.316` and `-0.505`, so `-0.407` is the arithmetically natural reading and
`-407` is physically impossible. **Reported as printed; the correction is not
made here.** Resolve against Lucia 2007 / SPE 84942.

**OPEN-3 — Lucia porosity units: decimals or percent?** (= D3). Page prose says
decimals **twice**; the emitted `Lucia_RC_Equations.txt` header says
**`'Phi' in percent`** [img-read: _hfuclip0049.png]. Same constants in both.
Unresolvable from the manual. Blocking for implementation.

**OPEN-4 — `Proportional` closure correction has no printed equation.**
Only prose: *"divides the plug Sw values by the Sw Closure value"*, giving
`SwCorr = Sw / SwClosure` — but whether the result is clipped at 1.0, and whether
`SwClosure` is the Sw at the closure point (as in Shift) or the correction
magnitude, is not stated. The accompanying figure `_shmclip0164.png` is a
crossplot illustration, not an equation. Source: cappressuresetup.htm.

**OPEN-5 — `Crop` and `Extrapolate` closure methods are procedural only.**
No closed-form expression is printed for either. Extrapolate's stated rule
("takes the two next lower plug point below the closure Sw point and uses them to
extrapolate") does not specify linear vs log-linear when the axes are linear —
the page only says the extrapolation *"will be using the logarithmic grid"* when
the axis Log checkbox is on. Source: cappressuresetup.htm.

**OPEN-6 — `Restore Defaults` scope: 3 columns or 4?** (= D6). Prose says "first 3
columns"; button caption varies between `Restore Defaults` and `Restore Lab
Defaults` across screenshots. Whether Interfacial Ten. Laboratory is restored is
undetermined. Source: cappressuresetup.htm, [img-read: _shmclip0009.png] vs
[img-read: _shmclip0084.png].

**OPEN-7 — Which phase do the stress/clay corrections actually operate on?** (= D5).
Formula labels say wetting phase; the `Run Corrections` order says non-wetting;
the formulae's algebraic structure `1 − (1 − Sw)·F` agrees with non-wetting.
Reported, not resolved. Source: cappressuresetup.htm.

**OPEN-8 — `_shmclip0167.png` row labels vs `_shmclip0009.png`** (= D4). Whether
IP 2025 has a 4-row table with a leading `Default` row, or the screenshot is a
scroll artefact, cannot be settled from these pages. All prose describes exactly
three methods.

**OPEN-9 — "32 models" is not derivable without an assumption** (= D8). 8 methods ×
7 regressions = 56; 32 is reachable only if the four Porosity & Height methods do
not cross with the regression list. Not stated. Source:
logswversusheightfunctions.htm.

**OPEN-10 — Swanson `So` is never defined dimensionally.** `So/Pc` uses
"Non Wetting Phase Saturation over Capillary Pressure", but whether `So` is a
fraction or a percentage, and whether `Pc` is psi at lab or reservoir conditions,
is not stated. With exponents near 2, a fraction-vs-percent error is a factor of
~10⁴ in K. No citation is given for the Swanson correlations, so there is no
primary source named on the page to resolve it against.
Source: cappressuresetup.htm.

**OPEN-11 — `0.43353` vs `0.433`** (= D9). The unrounded value is an **inference**
back-solved from one worked example (`0.70856` from `1.12`, `0.4`, `2.27`), not a
stated constant. Confirm against a second IP-generated report before adopting.

**OPEN-12 — Winland `ALog` base.** `WinR35 = ALog(...)` — `ALog` is used without
definition. The companion form `Log(R35) = 0.732 + ...` and the inversion
`Perm = 10^(...)` [img-read: _hfuclip0027.png] both imply base 10, but the manual
never says so. Same question applies to Lucia's `RFN = Alog(...)`.
Source: hfu.htm.

**OPEN-13 — `Fs`, `τ`, `Sgv` are never given values or typical ranges.** They
appear only inside the Kozeny-Carman derivation and are absorbed into the fitted
FZI. Not a defect, but there is no IP-sourced default to cite if SandiBumi ever
wants to decompose FZI.

---

*Ingest agent E. 7/7 pages read in full; 34 images read; no vendor file copied,
moved or modified; nothing written outside this file.*
