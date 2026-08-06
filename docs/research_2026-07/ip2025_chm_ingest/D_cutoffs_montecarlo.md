# D — Cut-offs, Summation & Monte Carlo (IP 2025 CHM ingest)

Agent D of the 14-agent Interactive Petrophysics 2025 vendor-manual ingest.
Consumer: SandiBumi. All facts carry provenance: `(page.htm)` for prose,
`[img-read: file.png]` for image transcription. Vendor prose is restated in my
own words; equations, constants and parameter values are reproduced as facts.

**Convention used throughout:** values labelled *IP DEFAULT* are what Interactive
Petrophysics ships or displays in its own dialogs. They are **not** universal
petrophysical truths and must never be adopted by SandiBumi as physics without an
independent, cited source.

---

## 1. Scope & page inventory

| Page (stem) | Title | Chars | Imgs | Status |
|---|---|---:|---:|---|
| `cutoffsandsummation` | Cut-off and Summations | 45,235 | 43 | **Fully read** + 6 equation rasters transcribed + 5 parameter panels read |
| `define_monte_carlo_parameters` | Monte Carlo Uncertainty Analysis | 37,792 | 41 | **Fully read** + 8 parameter panels read |
| `multi-wellcutoffsandsummati` | Multiple Well Cutoff and Summation | 19,750 | 13 | **Fully read** (text carries a full worked report listing; images are `_zoom50` GUI shots, no new numerics) |
| `cut-off_sensitivity_results` | Cutoff Sensitivity and Results | 12,055 | 9 | **Fully read** (worked multi-well averaging example is in prose) |
| `batchformula` | Multiple Well Batch Operation | 12,283 | 11 | **Fully read** — hub/workflow page, no petrophysical numerics |
| `multiwellcurvestats` | Multiple Well Curve Statistics | 4,218 | 4 | **Fully read** |
| `cm_curve_statistics` | Curve Statistics | 4,735 | 3 | **Fully read** |
| `batchmontecarlo` | Batch Monte Carlo | 4,335 | 8 | **Fully read** — hub page over the MC module |

Images read directly (vision), 20 total:
`embim159`–`embim164` (the six averaging equations), `_candsclip0010`
(Input Curves grid), `_candsclip0012` (Default Cut-offs grid), `_candsclip0020`
(Reservoir Cutoffs), `_candsclip0021` (Pay Cutoffs), `_candsclip0030`
(net-thickness worked diagram), `_mceaclip0005` (distribution shapes),
`_mceaclip0020` (results listing), `_mceaclip0024` (dependency crossplots),
`_mceaclip0038` / `_mceaclip0039` (Output tab defaults), `_mceaclip0041`
(Result Curves tab), `_mceaclip0049` (Model tab), `_mceaclip0057` (Input Curves
MC shifts), `_mceaclip0058` (Dependencies), `_mceaclip0070` (Clay Volume MC
shifts, zonal), `_mceaclip0075` / `_mceaclip0076` (Wellbore Stability MC shifts).

**No smectite or montmorillonite endpoint appears anywhere on these 8 pages**
(verified by case-insensitive grep of all 8 raw `.htm`). Nothing to add to the
standing SandiMin clay review from this agent.

---

## 2. Equations & methods

### 2.1 Discretisation — the half-interval rule

Each depth sample is treated as a discrete interval whose **recorded depth is the
centre** of that interval; consequently only **half** of the top and bottom depth
increments of a zone are counted when averaging or summing
(`cutoffsandsummation.htm`).

Worked example transcribed from the vendor diagram `[img-read: _candsclip0030.png]`
— zone 100.0 → 104.0 ft, depth step 0.5 ft, flag = 0 at 100.0 and 100.5, flag = 1
from 101.0 to 104.0:

```
Net = ( (0.5 x 0) + (1 x 0) + (6 x 1) + (0.5 x 1) ) x 0.5
    = ( 0 + 0 + 6 + 0.5 ) x 0.5
    = 6.5 depth levels x 0.5
    = 3.25            [ft]
```

Weight per level: **0.5 for the zone's first and last level, 1.0 for every interior
level**; the weighted level count is then multiplied by the depth step to give
thickness. `[img-read: _candsclip0030.png]`

### 2.2 Averaging formulas (all six rasters transcribed)

Symbol key as given on the page: `i` = ith input value, `hᵢ` = ith input interval,
`n` = number of samples (`cutoffsandsummation.htm`).

**Average porosity** — thickness-weighted arithmetic. `[img-read: embim159.png]`

```
              Σ(i=1..n) φᵢ × hᵢ
   φ_av  =  ─────────────────────
                Σ(i=1..n) hᵢ
```

**Average water saturation** — computed via hydrocarbon pore volume, i.e.
**porosity-×-thickness weighted**, not thickness weighted.
`[img-read: embim160.png]`

```
                   Σ(i=1..n) φᵢ × hᵢ × (1 − Sw)
   S_av  =  1  −  ──────────────────────────────
                        Σ(i=1..n) φᵢ × hᵢ
```

> Transcription note: in the raster the `Sw` inside the numerator carries **no `i`
> subscript**, while `φᵢ` and `hᵢ` both do. See §5.1 — treated as a vendor
> typesetting defect, not transcribed as a different quantity.

**Average clay volume** — thickness-weighted arithmetic. `[img-read: embim161.png]`

```
                Σ(i=1..n) Vclᵢ × hᵢ
   Vcl_av  =  ───────────────────────
                  Σ(i=1..n) hᵢ
```

**Extra curves — Arithmetic** (thickness-weighted). `[img-read: embim162.png]`

```
                    Σ(i=1..n) Curveᵢ × hᵢ
   Curve_av  =  ────────────────────────────
                       Σ(i=1..n) hᵢ
```

**Extra curves — Geometric.** `[img-read: embim163.png]` (verified at 4× upscale)

```
   Curve_av  =  ( C₁ . C₂ . C₃ . C₄ ..... Cₙ ) ^ ( 1 / Σ(i=1..n) hᵢ )
```

> **The root order is the summed thickness Σhᵢ, not the sample count n.** The
> product term is the plain product of the n curve values, with no thickness
> weighting inside. This is reproduced exactly as drawn; see §5.2 for why it
> matters and §7.

**Extra curves — Harmonic** (thickness-weighted). `[img-read: embim164.png]`

```
                     Σ(i=1..n) hᵢ
   Curve_av  =  ──────────────────────────
                Σ(i=1..n) hᵢ / Curveᵢ
```

**Guard rule (all three extra-curve methods, stated for geometric and harmonic):**
any input value **≤ 0 is ignored** and excluded from the final average
(`cutoffsandsummation.htm`).

### 2.3 Which average applies to what

- Porosity and clay volume: thickness-weighted arithmetic (§2.2).
- Water saturation: **porosity-weighted**. Stated twice in prose — the `Sw` *Curve
  Type* "initiates a computation of a porosity-weighted average for the input curve
  when computing zonal averages", and both result tabs describe `Av Sw Res` /
  `Av Sw Pay` as "a porosity-weighted average" (`cutoffsandsummation.htm`).
- Extra curves 1–7: user-selectable Arithmetic / Geometric / Harmonic per curve,
  set in the *Average Method* column of the Input Curves tab
  (`cutoffsandsummation.htm`, `multi-wellcutoffsandsummati.htm`).
- **Zonal averages on the main report are thickness-weighted, explicitly not
  arithmetic**; interval-breakdown averages are plain arithmetic over intervals, so
  "interval average × number of intervals" does not reconcile to the zonal average
  (`cutoffsandsummation.htm`).

### 2.4 Multi-well / field aggregation

Field ("All Wells") averages are **net-thickness-weighted**
(`multi-wellcutoffsandsummati.htm`). The method is: sum the value-thickness product
across wells, divide by summed thickness (`cut-off_sensitivity_results.htm`).

Vendor worked example, transcribed (`cut-off_sensitivity_results.htm`):

```
Well 1 (Zone 1)   Av Phi 0.195   Net 50 ft   PhiH = 50 x 0.195 = 9.75
                  Av Sw  0.25                PhiSoH = 9.75 x (1 - 0.25) = 7.313
Well 2 (Zone 1)   Av Phi 0.165   Net 20 ft   PhiH = 20 x 0.165 = 3.3
                  Av Sw  0.30                PhiSoH = 3.3 x (1 - 0.3) = 2.31

Combined:  PhiH   = 9.75 + 3.3   = 13.05
           Net    = 50 + 20      = 70 ft
           PhiSoH = 7.313 + 2.31 = 9.623
           Av Phi = 13.05 / 70          = 0.187
           Av Sw  = 1.0 - 9.623 / 13.05 = 0.263
```

Definitions given on the same page: `So = 1 − Sw`; `PhiH` = net porosity thickness;
`PhiSoH` = net hydrocarbon pore thickness. Note the combined `Av Sw` is recovered
from the **PhiSoH/PhiH ratio**, i.e. the multi-well roll-up uses the same
porosity-weighted route as §2.2.

### 2.5 Interval-breakdown thickness convention

Net thickness of a flagged interval = `Bottom − Top + depth step`, where Top is the
first depth at which the flag turns on and Bottom the last depth at which it is on.
A single-sample interval therefore has thickness = one depth step, not zero
(`cutoffsandsummation.htm`).

### 2.6 Curve Statistics definitions

- **Net** = (count of non-NULL samples) × well step. For an all-good curve this
  equals `Bottom − Top + one database sample increment` (`cm_curve_statistics.htm`).
- **Mode** = data sorted into a **50-cell histogram**, highest cell taken; the page
  notes it needs on average **at least 3 values per cell** to be meaningful
  (`cm_curve_statistics.htm`).
- **Fail Disc** = count of depth steps inside the range failing the discriminator
  logic (`cm_curve_statistics.htm`).

### 2.7 Monte Carlo shift algebra

Three *Type Shift* modes, formulas given verbatim in prose
(`define_monte_carlo_parameters.htm`):

| Mode | Formula |
|---|---|
| Linear | `Result = Input + Shift` |
| Percent | `Result = Input × (1 + Shift / 100)` |
| Reciprocal | `Result = 1 / ( 1 / Input + Shift )` |

Gaussian width mapping: **Low Value Shift + High Value Shift = four standard
deviations**, confirmed by the annotated figure which labels the full Low+High span
"4 Standard deviations" with the Start Value at the centre line
(`define_monte_carlo_parameters.htm`; `[img-read: _mceaclip0005.png]`). Hence
`σ = (Low + High) / 4`.

Distribution shapes as drawn `[img-read: _mceaclip0005.png]`:
- **Square** — flat-topped box spanning Start−Low to Start+High.
- **Triangular** — apex at Start Value, falling linearly to Start−Low and Start+High.
- **Gaussian** — bell centred on Start Value, Low+High span = 4σ.

Truncation: the Gaussian is **limited to ±2.5 σ about the mean**; a draw outside
that range is discarded and re-drawn. Stated purpose is to stop parameters such as
Rw going negative and breaking the interpretation module
(`define_monte_carlo_parameters.htm`).

Dependency / correlation algebra: the randomly-selected shift for Parameter 1 is
applied to dependent Parameter 2; negative correlation inverts the shift; for
|correlation| < 1 a randomness is superimposed — "a coefficient of 0.5 will apply a
randomness of half what would have been selected if the coefficient was 0.0"
(`define_monte_carlo_parameters.htm`). Correlation range: 0 = none, 1 = 100 %,
−1 = inverse 100 %.

Tornado plot construction: for each parameter, **two** workflow runs — one at its
low value, one at its high value, **±2 standard deviations for Gaussian
distributions** — with all other parameters held at default
(`define_monte_carlo_parameters.htm`). This is internally consistent with §2.7's
4σ = Low+High mapping (±2σ = exactly the Low/High bounds).

---

## 3. Parameters, defaults & constraints

### 3.1 Cut-off logic semantics (tasking a)

**Combination rule — AND.** Every enabled cut-off must be satisfied simultaneously.
The prose repeats the conjunction for each parameter: a level "can be considered for
Pay or Reservoir **if the level also meets all the other cut-offs**"
(`cutoffsandsummation.htm`, stated for Phi, Sw, Vcl and for Other cut-offs 1–3).
There is no OR mode and no per-report boolean operator anywhere on the page.

**Direction of each inequality** (`cutoffsandsummation.htm`, confirmed against the
Input Curves grid `[img-read: _candsclip0010.png]`):

| Curve | Test | Cut-off Type shown |
|---|---|---|
| Porosity | `φ ≥ cut` (greater than **or equal**) | `>=` |
| Water saturation | `Sw ≤ cut` (less than **or equal**) | `<=` |
| Clay volume | `Vcl ≤ cut` (less than **or equal**) | `<=` |
| Extra curves 1–3 (and 1–7) | `≥` **or** `≤`, user-set per curve in the *Cut-off Type* column | new row defaults to `>=` |

**Reservoir vs Pay vs additional reports.**

- *Net Reservoir* — "determined by the application of the **Porosity and optional
  Clay Volume** cut-off criteria" (`cutoffsandsummation.htm`).
- *Net Pay* — "determined by the application of the **Vclay, Porosity and Water
  Saturation** cut-off criteria" (`cutoffsandsummation.htm`).
- The distinction is therefore **not a separate cut-off value** but which *Use*
  flags are enabled: Sw participates in Pay and not in Reservoir by default.
- Up to **3 additional reports** (Reports 3–5) beyond Reservoir and Pay, i.e. **up
  to 5 summation reports total**. Additional reports are fully free-form — "they do
  not necessarily have to be set up to report Net and/or Pay"
  (`cutoffsandsummation.htm`).
- With **no** cut-off criteria applied at all, the module reports zonal averages for
  every defined zone (`cutoffsandsummation.htm`).

**Zone-membership gate (overrides cut-offs).** Cumulative output curves are computed
per zone and include only data **inside a defined zone**: "if a level is not defined
as being in a zone, it will not be included in the cumulative curves, **regardless of
whether the level meets the cut-off criteria**" (`cutoffsandsummation.htm`).

**Minimum-height gate.** `Min Res Height` / `Min Pay Height` / `Min XXXX Height` set a
minimum thickness for an interval to count. **IP DEFAULT = 0**, documented meaning
"all depth intervals will count towards net if they meet the cut-off criteria"
(`cutoffsandsummation.htm`; same default restated in
`multi-wellcutoffsandsummati.htm`).

**Sub Total zones** carry **no cut-offs of their own**; they re-use the interpretation
results of the normal zones they span (`cutoffsandsummation.htm`,
`multi-wellcutoffsandsummati.htm`).

**Sign constraint (sensitivity module):** "Cutoffs cannot be negative."
(`cut-off_sensitivity_results.htm`).

### 3.2 IP DEFAULT cut-off values

Read from the shipped Reports Set-Up / Default Cut-offs grid
`[img-read: _candsclip0012.png]`. **Report 5 is the unconfigured column** (its *Use
report* box is clear) and is therefore the cleanest evidence of the factory default
state for a fresh report.

| Row | Curve | Res (R1) | Use | Pay (R2) | Use | REP3 | Use | REP4 | Use | R5 | Use |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Porosity | 0.1 | ✓ | 0.1 | ✓ | 0.1 | ✓ | 0.1 | ✓ | **0.1** | **✓** |
| 2 | Water Saturation | 0.5 | — | 0.5 | ✓ | 0.5 | — | 0.5 | — | **0.5** | **—** |
| 3 | Clay Volume | 0.5 | ✓ | 0.5 | ✓ | 0.5 | ✓ | 0.5 | ✓ | **0.5** | **✓** |
| 4 | Gama | 25 | — | 25 | — | 0 | — | 0 | — | 0 | — |
| 5 | Density | 2.6 | — | 2.6 | — | 0 | — | 0 | — | 0 | — |
| 6 | Deep Resistivity | 200 | — | 200 | — | 0 | — | 0 | — | 0 | — |
| 7 | Porosity (2nd curve) | 0.13 | — | 0.13 | — | 0 | — | 0.15 | ✓ | 0 | — |
| 8–10 | (empty) | 0 | — | 0 | — | 0 | — | 0 | — | 0 | — |

**IP DEFAULTS, stated plainly:**

| Parameter | IP DEFAULT | Used by default? | Units | Source |
|---|---|---|---|---|
| Porosity cut | **0.1** | Reservoir ✓, Pay ✓ | v/v decimal | `[img-read: _candsclip0012.png]`, `_candsclip0020.png` |
| Water saturation cut | **0.5** | Reservoir ✗, Pay ✓ | v/v decimal | same |
| Clay volume cut | **0.5** | Reservoir ✓, Pay ✓ | v/v decimal | same |
| Min Res Height | **0** | — | depth units | `cutoffsandsummation.htm`; `[img-read: _candsclip0020.png]` |
| Min Pay Height | **0** | — | depth units | `cutoffsandsummation.htm`; `[img-read: _candsclip0021.png]` |
| Extra-curve cut (unset row) | **0** | ✗ | — | `[img-read: _candsclip0012.png]` |
| Fluid Efficiency (CO2) | **1** | — | multiplier | `[img-read: _candsclip0021.png]` |
| Solution Efficiency (CO2) | **1** | — | multiplier | `[img-read: _candsclip0021.png]` |

Prose corroboration of the two default reports
(`cutoffsandsummation.htm`): *Reservoir* = "Porosity >= 0.1, Clay Volume <= 0.5";
*Pay* = "Porosity >= 0.1, Clay Volume <= 0.5, Water Saturation <= 0.5".

Rows 4–7 of the grid (Gamma 25, Density 2.6, Deep Resistivity 200, 2nd Porosity 0.13)
are values in a **vendor example screenshot with every *Use* box clear** — they are
carried but not applied. Do not treat them as IP defaults for those curve types.

The multi-well worked report echoes the same numbers in its "cut-offs USED" footer:
`Phi > 0.100`, `Sw < 0.500` (Pay flagged `Y`), `Vcl < 0.500`, for all four zones
(`multi-wellcutoffsandsummati.htm`).

### 3.3 Reservoir / Pay parameter-panel defaults

`[img-read: _candsclip0020.png]` — **Reservoir Cutoffs** tab, all zones 1–6 identical:
`Min Res Height 0` · `Phi Cut Res/Pay 0.1` · `Phi Res Use ✓` · `Sw Cut Res/Pay 0.5` ·
`Sw Res Use (clear)` · `Vcl Cut Res/Pay 0.5` · `Vcl Res Use ✓`.

`[img-read: _candsclip0021.png]` — **Pay Cutoffs** tab, all zones 1–6 identical:
`Min Pay Height 0` · `Fluid Efficiency 1` · `Solution Efficiency 1` ·
`Phi Pay Use ✓` · `Sw Pay Use ✓` · `Vcl Pay Use ✓`.

Note the Pay tab carries **only *Use* flags, no cut *values*** — the value columns on
the Reservoir tab are headed `Res/Pay`, i.e. shared. See §5.3.

### 3.4 Input Curves tab defaults

`[img-read: _candsclip0010.png]` (verified at 3× upscale):

| Row | Cutoff Name | Use | Short | Curve Type | Input Curve | Cut-off Type | Average Method | Result Precision | Cum CrvH |
|---|---|---|---|---|---|---|---|---|---|
| 1 | Porosity | ✓ | Phi | Phi | `PhiSw:PHIE` | `>=` | Arithmetic | 3 | ✓ |
| 2 | Water Saturation | ✓ | Sw | Sw | `PhiSw:SW` | `<=` | Arithmetic | 3 | ✓ |
| 3 | Clay Volume | ✓ | Vcl | Vcl | `PhiSw:VWCL` | `<=` | Arithmetic | 3 | — |
| 4 | *(empty new row)* | — | — | — | — | **`>=`** | **Arithmetic** | **3** | — |

Row 4 gives the defaults applied to a freshly-added curve row: `>=`, Arithmetic,
precision 3.

**Curve Type pre-processing** (`cutoffsandsummation.htm`) — setting a Curve Type
applies a clip before summation:

| Curve Type | Clip applied | Extra behaviour |
|---|---|---|
| `Phi` | values **> 0** only | — |
| `Vcl` | clipped to **[0, 1]** | lets a GR-derived Vcl be used directly |
| `Sw` | clipped to **[0, 1]** | **triggers porosity-weighted zonal averaging** |

**Other Input Curves constraints:**

| Parameter | Value | Source |
|---|---|---|
| Max input curves | **50** | `cutoffsandsummation.htm` (but see §5.4) |
| Pre-defined curves | 3 (Porosity, Water Saturation, Clay Volume) | `cutoffsandsummation.htm` |
| Optional additional curves (Parameters tabs, Results) | **7** | `cutoffsandsummation.htm` |
| Optional additional curves (multi-well module) | **7** | `multi-wellcutoffsandsummati.htm` |
| Max summation reports | **5** (2 default + 3 optional) | `cutoffsandsummation.htm` |
| Result Precision default | **3** decimal places | `cutoffsandsummation.htm`; `[img-read: _candsclip0010.png]` |
| Result Precision maximum | **6** decimal places | `cutoffsandsummation.htm` |
| Report field width to file/printer | **8 characters** | `cutoffsandsummation.htm` |
| Report Title max length | **25** alphanumeric characters | `cutoffsandsummation.htm` |
| Report Short Name max length | **4** characters | `cutoffsandsummation.htm` |

The 6-place / 8-character interaction is flagged by the vendor itself: precision 6 is
only useful for very small numbers because the output string is capped at 8 chars
(`cutoffsandsummation.htm`).

### 3.5 Output curve names (defaults)

`cutoffsandsummation.htm`. `PayFlag` and `ResFlag` "should always be calculated"
(and in the multi-well module the Pay Flag and Reservoir Flag curves **must** be
output — `multi-wellcutoffsandsummati.htm`).

| Quantity | Default name |
|---|---|
| Cumulative Reservoir Porosity Thickness | `ResPhiH` |
| Cumulative Reservoir Thickness (net) | `ResPhiSoH` |
| Cumulative Reservoir Clay Volume Thickness | `ResVclH` |
| Cumulative Pay Porosity Thickness | `PayPhiH` |
| Cumulative Pay Thickness (pay) | `PayPhiSoH` |
| Cumulative Pay Clay Volume Thickness | `PayVclH` |
| Cumulative Reservoir CO2 capacities | `ResCO2fluid`, `ResCO2soln`, `ResCO2stor` |
| Cumulative Storage Porosity Thickness | `StorPhiH` |
| Cumulative Storage Thickness | `StorPhiSH` |
| Cumulative Storage Clay Volume Thickness | `StorVclH` |
| Cumulative Storage CO2 capacities | `StorCO2fluid`, `StorCO2soln`, `StorCO2stor` |
| Optional-report cumulative curve | auto-named from report short name, e.g. `REP4RHOBH` |

Two distinct cumulative modes (`cutoffsandsummation.htm`,
`multi-wellcutoffsandsummati.htm`):
- **Output Cum CrvH** — accumulates `curve × thickness` (e.g. permeability-footage).
- **Output Cum Crv** — accumulates the curve by **simple summation, no height
  factor**, for inputs that already embed thickness (CO2 storage capacity curves).

`$$` in a results column means **null data exists in that zone interval**
(`cutoffsandsummation.htm`).

### 3.6 Result parameters produced

Reservoir tab (`cutoffsandsummation.htm`): `Gross Interval`, `Net Res`,
`Net/Gross Res`, `Av Phi Res`, `Av Sw Res` (porosity-weighted), `Av Vcl Res`,
`Other Curve Av (Name) Res`, `PhiH Res`, `PhiSoH Res`, `Other Curve (Name)H Res`;
plus `TVD/TVT Gross`, `TVD/TVT Net Res`, `TVD/TVT N/G Res` when a TVD curve is set.
Pay tab is the same set with `Pay` substituted. All non-editable.

### 3.7 Monte Carlo — defaults & constraints (tasking c)

**Distribution types offered — exactly three:** `Gaussian`, `Triangular`, `Square`
(`define_monte_carlo_parameters.htm`; shapes at `[img-read: _mceaclip0005.png]`).
The same three are offered for interpretation parameters, for input curves, and for
Mineral Solver model parameters. Observed default in every panel read: **Gaussian**.

**Sampling method as stated:** "IP uses a random number generator, **seeded through
the CPU clock time**, to calculate the shifts for each parameter for each
simulation. At the start of each simulation, each parameter is changed using a
different random number." (`define_monte_carlo_parameters.htm`).

> **Reproducibility:** there is **no user-settable seed** documented anywhere on the
> page. Clock-seeding means runs are **not bit-reproducible**. IP's substitute for
> reproducibility is the *Show/Output individual iteration result* workflow — rank
> iterations by a chosen Result Parameter, pick a percentile, and IP reloads the
> internally-saved shifted parameter sets and shifted input curves for that exact
> iteration so the run can be recreated and exported
> (`define_monte_carlo_parameters.htm`).

**Iteration / convergence defaults:**

| Parameter | IP DEFAULT | Min / constraint | Source |
|---|---|---|---|
| Stop simulation at | **2000** iterations | — | `[img-read: _mceaclip0038.png]`, `_mceaclip0039.png` |
| Auto-stop enabled | **off** (checkbox clear) | — | `[img-read: _mceaclip0038.png]` |
| Convergence tolerance | **0.1 %** change between checks | — | `define_monte_carlo_parameters.htm`; `[img-read: _mceaclip0038.png]` |
| Auto-stop burn-in | — | **minimum 200** iterations before first check | `define_monte_carlo_parameters.htm` |
| Auto-stop check interval | — | every **100** iterations after burn-in | `define_monte_carlo_parameters.htm` |
| Auto-stop minimum total | — | **300** iterations | `define_monte_carlo_parameters.htm` |
| Convergence Result Parameter | **`PhiSoH Res`** (hydrocarbon pore volume, reservoir) | — | `define_monte_carlo_parameters.htm`; `[img-read: _mceaclip0038.png]` |
| Convergence Result Zone | **`All`** | — | same |
| Convergence criterion | P10, P50, P90 **and** mean must all be within tolerance | — | `define_monte_carlo_parameters.htm` |
| Update Graphics every | **20** iterations | — | `[img-read: _mceaclip0038.png]` |
| Save all output curve simulations | **on** ("Default is to save all results") | — | `define_monte_carlo_parameters.htm`; `[img-read: _mceaclip0039.png]` |
| Max output curves | **10** | — | `define_monte_carlo_parameters.htm` |
| Max histograms displayed | **9** | — | `define_monte_carlo_parameters.htm` |
| Max crossplots displayed | **9** | — | `define_monte_carlo_parameters.htm` |
| Output Percentiles (listing) | **10, 50, 90** (5 slots, 2 blank) | — | `[img-read: _mceaclip0038.png]` |
| Output curve set | `MC (Monte Carlo)` | — | `[img-read: _mceaclip0038.png]` |
| Gaussian truncation | ±**2.5 σ**, redraw outside | — | `define_monte_carlo_parameters.htm` |
| Gaussian width | Low + High = **4 σ** | — | `define_monte_carlo_parameters.htm`; `[img-read: _mceaclip0005.png]` |
| Tornado low/high runs | ±**2 σ** for Gaussian | — | `define_monte_carlo_parameters.htm` |
| Shift values | **must be positive** | hard constraint | `define_monte_carlo_parameters.htm` |
| Correlation range | 0 = none, 1 = 100 %, −1 = inverse 100 % | — | `define_monte_carlo_parameters.htm` |
| Histogram overlay default | **Gaussian** | — | `define_monte_carlo_parameters.htm` |

**Result Curves tab defaults** `[img-read: _mceaclip0041.png]`:

| Output | Enabled by default | Extension |
|---|---|---|
| Output mean result | ✗ | `_mn` |
| Output plus one standard deviation | ✗ | `_psd` |
| Output minus one standard deviation | ✗ | `_msd` |
| Output P: **5** | **✓** | `_P5` |
| Output P: **50** | **✓** | `_P50` |
| Output P: **95** | **✓** | `_P95` |
| Output curve array results | auto when "save all" on | `_mc` |

Auto Log Plot curve mnemonics quoted in prose: `XXX MN` (mean), `XXX PSD` (+1 σ),
`XXX MSD` (−1 σ), where XXX is the original curve name
(`define_monte_carlo_parameters.htm`). See §5.5.

**Percentile sign convention (important, and configurable):** "By default, the 10th
percentile will be the 10th percent **lowest** value of all the simulation results,
**except for Sw where it will be the 10th percent highest value**." The convention
lives in the Results section of `MonteCarloDefaults.par` and can be edited
(`define_monte_carlo_parameters.htm`). For the *output curves*, the plain rule is
stated: "P5 will be the 5th percentile lowest value. P50 will be the middle value."

**Per-parameter default uncertainty ranges actually observed in shipped panels.**
These are IP's populated defaults, transcribed as-is:

| Panel | Parameter / Curve | Type Shift | Distribution | Low Shift | High Shift | Initial value shown | Source |
|---|---|---|---|---|---|---|---|
| Input Curves | `RHOB` | Linear | Gaussian | **0.02** | **0.02** | — | `[img-read: _mceaclip0057.png]` |
| Input Curves | `DTLN` | Linear | Gaussian | **2.000** | **2.000** | — | same |
| Input Curves | `TNPH` | **Percent** | Gaussian | **5.000** | **5.000** | — | same |
| Input Curves | `LLD` | **Reciprocal** | Gaussian | **0.005** | **0.005** | — | same |
| Input Curves | `Dens:RhoGard` | Linear | Gaussian | **0.020** | **0.020** | — | `[img-read: _mceaclip0076.png]` |
| Clay Volume | `Gr Clean` | Linear | Gaussian | **10** | **10** | 13.9 | `[img-read: _mceaclip0070.png]` |
| Clay Volume | `Gr Clay` | Linear | Gaussian | **10.000** | **10.000** | 134.9 | same |
| Wellbore Stability | **every** parameter | **Percent** | Gaussian | **20.000** | **20.000** | see below | `[img-read: _mceaclip0075.png]` |

Wellbore Stability initial values shown alongside the uniform ±20 % shift
`[img-read: _mceaclip0075.png]`: Max Hori Stress A **45**; UCS *Input Curve*;
UCS Cali Multiplier **1 – 2**; Rock Strength *Input Curve*; Friction Angle **30**;
Biot Alpha Factor **1**; Poisson's Ratio *Input Curve*; Hoek B Granularity **6.7**;
Hoek B Exponent **0.5**; Hoek B Disturbance **0.06**; Hoek B Strength **75**;
Hoek B Dist Factor **0**; Hoek B Mat Const **16**.

Zonal shift grids: uncertainties may be entered **per zone** (Zonal Values checkbox →
Zonal button). In both zonal grids read, all zones were populated with the same
value as the global entry (`Gr Clean` 10.000 × 6 zones; `RHOB` 0.020 × 6 zones)
`[img-read: _mceaclip0070.png]`, `[img-read: _mceaclip0057.png]`.

Mineral Solver model parameters: "default values are filled in which are equal to the
**Mineral end-point value plus or minus 10 % of the valid value**"
(`define_monte_carlo_parameters.htm`).

Dependency example values `[img-read: _mceaclip0058.png]`: `m exponent` ↔
`n exponent` correlation **0.8**; `Rho Wet Clay` ↔ `Neu Wet Clay` correlation
**−0.8**. Resulting shift spans observed on the companion crossplot
`[img-read: _mceaclip0024.png]`: m ≈ −0.289…0.29, n ≈ −0.282…0.277, Rho Wet Clay
≈ −0.0714…0.0711, Neu Wet Clay ≈ −0.0709…0.0705, at 518 simulations. (Axis extents
only — the underlying Low/High shift entries are not shown; see §8.)

**Which parameters can be varied.** Modules selectable into the MC workflow
(`define_monte_carlo_parameters.htm`): Basic Log Analysis, Formula, Multi-Line
Formula, Curve from Zones, Clay Volume, Porosity SW, Cutoff, NMR, Mineral Solver,
Sigma Sw, Fuzzy Logic, Multi-Linear Regression, Neural Networks, Cluster Analysis,
SOM, DTA; Geomechanics suite (Mechanical Properties, Horizontal Stress, Wellbore
Stability, Pore Pressure, Density Estimation, Vertical Stress, Poro- and Thermo-
Elastic Stress, Fractures and Fault Stress, Rock Strength Anisotropy); Saturation vs
Height Curves; Curve Auto Edit; PPFG Toolbox modules; Unconventional Resources
Toolbox modules that have a Parameter Set; Sand Silt Malay; TOC; any Zonal User App.

Within a selected module, **all parameters are selected by default** ("All parameters
by default are selected"), with unused ones harmlessly varied
(`define_monte_carlo_parameters.htm`). Special cases: **DTA has no parameters** —
uncertainty applies to input curves only; **Saturation vs Height Curves has no zonal
parameter set** — the zonal parameters are the per-Hydrocarbon-Model values. The
**Cutoff module is optional** in the workflow, but omitting it restricts results to
foot-by-foot curve errors only (no zonal summation statistics).

Constants inside a Formula cannot be varied directly — the documented workaround is
to convert the constant to a Constant **curve** and vary that curve via the Input
Curves tab (`define_monte_carlo_parameters.htm`).

**`MonteCarloDefaults.par` index map** (`cutoffsandsummation.htm`) — the parenthesised
numbers prefixing Cut-off parameter names are IP's own indices into that file:

| Index | Parameter |
|---|---|
| (3) | Phi Cut Res/Pay |
| (6) | Sw Cut Res/Pay |
| (9) | Vcl Cut Res/Pay |
| (12) | Other cut-off 1 |
| (15) | Other cut-off 2 |
| (18) | Other cut-off 3 |

Stride of 3 — consistent with a (value, use, ?) triplet per cut-off, though the file's
actual layout is never shown.

### 3.8 Batch Monte Carlo

`batchmontecarlo.htm`. A parameter model file must first be built and run in the
single-well MC module (only a few simulations needed) and saved. The batch
`Stop simulation at` **overrides** the count in the per-well MC parameter file.
Recommended practice quoted: test all wells at **1 or 2** simulations first, then set
the final count (**2000**). Missing input curve → that curve is dropped from the MC
inputs; missing output curve → no statistics for it.

### 3.9 Cutoff sensitivity

`cut-off_sensitivity_results.htm`. Sweeps **one** cut-off parameter from
`Cutoff Start Value` to `Cutoff Stop Value` by `Cutoff Step`, re-running Cut-off and
Summation at each step. Vendor example: PHI cut-off **0.0 → 35 pu, step 1.0 pu**.
Displayed percentiles default to the **10th, 50th and 90th**. Files: `.cos` (setup
format), `.cosr` (results). Prerequisite: Cut-off and Summation (single- or
multi-well) must already have been run on every well. Multi-well mode requires
consistent curve names across wells and a common zone set; missing zones warn and
continue.

### 3.10 Multi-well summation reporting

`multi-wellcutoffsandsummati.htm`. Seven tabs: Curve Set-Up, Input Curves, Output
Curves, Zones, Reports, Cutoffs, Results. Porosity and Water Saturation curves are
**required**; Clay Volume optional unless used as a cut-off. Field averages
(`All Wells` row) are **net-thickness weighted**. Output-file naming defaults:
`Output files to well directories` selected (default) and `Use Well Name in file
name` selected (default) → filename `Summation<WellName>`; if cleared → `Cutoff`.
CSV delimiter is switchable to semicolon via
`Tools → Options → Miscellaneous Options → CSV Delimiter`.

### 3.11 Batch operation file extensions

`batchformula.htm` — Set-type extensions IP uses on disk: Formula `.frm`;
Multi-Line Formula `.mlf`; Temperature `.tem`; Zone Curves `.ztc`; Filter `.flt`;
Basic Log Func `.blc`; Fill Data Gaps `.fdg`; Basic Log Anal / Clay Volume /
Porosity Sw / Mineral Solver / Cutoff `.set`; batch job itself `.fbt`.
Cut-off and Summation parameter-set listing prints to `.TXT`
(`cutoffsandsummation.htm`).

---

## 4. Assumptions & validity limits

1. **Centre-of-interval discretisation.** Every sample owns a symmetric interval
   about its recorded depth; zone-boundary samples contribute half weight. Any
   engine that instead treats samples as top-of-interval will disagree with IP on
   net thickness by up to one depth step per zone (`cutoffsandsummation.htm`;
   `[img-read: _candsclip0030.png]`).
2. **Cut-offs are ANDed only.** No OR, no nested logic. Complex criteria must be
   expressed by spending one of the 3 optional report columns
   (`cutoffsandsummation.htm`).
3. **Zone membership dominates.** Levels outside any defined zone are excluded from
   cumulative curves even when they pass every cut-off
   (`cutoffsandsummation.htm`).
4. **Sw averaging is not thickness-weighted.** It is hydrocarbon-pore-volume
   weighted. Substituting a thickness-weighted Sw changes reported HCPV
   (`cutoffsandsummation.htm`, `embim160.png`).
5. **Geometric/harmonic averages silently drop non-positive samples**, so the sample
   population differs from the arithmetic case and n is not comparable across
   methods (`cutoffsandsummation.htm`).
6. **MC percentiles are computed parameter-by-parameter, independently.** The manual
   warns explicitly that P50 values do not come from a common iteration, so
   `P50 Gross × P50 N/G × P50 AvPhi × P50 (1 − AvSw)` does **not** reproduce the P50
   `PhiSoH` — "though it should be close"
   (`define_monte_carlo_parameters.htm`).
7. **MC is not reproducible by seed.** Clock-seeded RNG, no seed field
   (`define_monte_carlo_parameters.htm`).
8. **Gaussian is truncated at ±2.5 σ**, so the sampled distribution is not a true
   normal and its realised variance is below the nominal σ²
   (`define_monte_carlo_parameters.htm`).
9. **MC requires a prior deterministic interpretation.** Every module in the workflow
   "must have been set up and run on the well data before using the Monte Carlo
   module", including User Programs (`define_monte_carlo_parameters.htm`).
10. **Resistivity-type parameters are the documented failure mode.** Large shifts
    drive them negative and the interpretation module refuses to run; the manual
    tells the user to narrow that parameter's shift
    (`define_monte_carlo_parameters.htm`).
11. **Horizontal / rising-hole wells.** For a TVD summation where the well reverses
    upward, IP reports the actual vertical thickness *cut by the well* (so a zone can
    report more gross TVD than its top-to-bottom TVD difference) and prefixes such
    net and gross TVD thicknesses with `*`. Zonal averages are weighted by vertical
    thickness per depth increment, so TVD zonal averages can differ considerably from
    MD averages. Vendor example: TVD top −6077.38, bottom −6136.13 → difference
    58.75, but reported gross TVD **\*79.5** (`cutoffsandsummation.htm`).
12. **X/Y zone coordinates require a valid deviation survey and surface location**
    loaded in IP, otherwise they cannot be computed (`cutoffsandsummation.htm`).
13. **Curve-statistics Mode is unreliable on sparse data** — 50 bins, needs ≥3 values
    per bin on average (`cm_curve_statistics.htm`).
14. **Percentile-curve computation is CPU-intensive**; the manual advises keeping the
    graphics-update interval high (`define_monte_carlo_parameters.htm`).
15. **Array output curves scale with iteration count** — one array element per
    iteration per output curve; the manual warns the IP well can become very large
    (`define_monte_carlo_parameters.htm`).

---

## 5. Internal discrepancies

**5.1 `Sw` without index in the Average-Sw raster.** `embim160.png` draws
`Σ φᵢ × hᵢ × (1 − Sw)` — `φ` and `h` carry subscript `i`, `Sw` does not, while the
denominator's `φᵢ × hᵢ` both do. Verified at 4× upscale; the subscript is genuinely
absent, not a resolution artefact. Read literally the numerator would factor a single
constant `Sw` out of the sum and collapse to `S_av = Sw`, which is nonsense and
contradicts the surrounding prose ("porosity-weighted average"). Treated as a vendor
typesetting defect; the intended term is `Swᵢ`.
`[img-read: embim160.png]` vs `cutoffsandsummation.htm`.

**5.2 Geometric average root order is Σhᵢ, not n.** `embim163.png` draws
`(C₁·C₂·…·Cₙ)^(1/Σhᵢ)`. Every other formula on the page pairs a thickness-weighted
numerator with a Σhᵢ denominator; here an **unweighted product** is raised to
`1/Σhᵢ`. The two are only equivalent when `Σhᵢ = n`, i.e. when the depth step is
exactly 1 unit. At a 0.5 ft step, `Σhᵢ = n/2` and the expression returns the
**square** of the conventional geometric mean; at 0.1524 m it returns roughly the
6.6th power. Reported exactly as drawn — I did not "correct" it to `1/n`.
`[img-read: embim163.png]`

**5.3 Cut values: per-report in the setup grid, shared Res/Pay in the parameters
grid.** The Reports Set-Up tab shows **independent** `Cut Value` columns for Report 1
(Reservoir) and Report 2 (Pay) `[img-read: _candsclip0012.png]`, and the prose says
Reservoir-column entries "automatically populate the appropriate cells in the Pay
report column" — implying they are separately editable after population. But the
Cutoff Parameters window presents a single column headed **`Phi Cut Res/Pay`** (and
likewise `Sw Cut Res/Pay`, `Vcl Cut Res/Pay`), with no Pay-side value columns at all
on the Pay Cutoffs tab `[img-read: _candsclip0020.png]`, `[img-read: _candsclip0021.png]`.
Since all post-Run editing must happen in the Parameters window
(`cutoffsandsummation.htm`), Reservoir and Pay appear to be **forced to share one cut
value per curve** in practice, differing only by their `Use` flags. Unresolved —
see §8.

**5.4 Input-curve capacity: 50 vs 7.** The Input Curves section states "Up to **50**
input curves can be entered" (1 occurrence), while the Parameters-tab section states
the tabs "expand to accommodate up to **7** optional additional curves" (1
occurrence) and the Results sections refer to "the additional input curves **1-7**"
(4 occurrences) (`cutoffsandsummation.htm`). The multi-well page independently says
"up to 7 additional curves" (`multi-wellcutoffsandsummati.htm`). IP2018 said "Up to
10 input curves … The additional 7 curves (rows 4 - 10)" — internally consistent. The
2025 edit raised the Input Curves figure to 50 but **left the downstream 7-curve text
and every screenshot (10-row grids) unchanged**. The capacity of the *parameter and
results tabs* is the binding constraint for SandiBumi, and the manual only ever
documents 7 there.

**5.5 MC statistic curve naming: prose vs dialog.** Prose gives `XXX MN`, `XXX PSD`,
`XXX MSD` (space-separated suffix, upper case)
(`define_monte_carlo_parameters.htm`); the Result Curves dialog shows the extensions
as `_mn`, `_psd`, `_msd` (underscore, lower case) `[img-read: _mceaclip0041.png]`.

**5.6 Two different percentile defaults in one module.** The Output tab's
`Output Percentiles` ship as **10 / 50 / 90** `[img-read: _mceaclip0038.png]`, while
the Result Curves tab's percentile **curves** ship as **P5 / P50 / P95**
`[img-read: _mceaclip0041.png]`. These are different features (listing vs output
curve) but the manual never notes the asymmetry, and the prose discussion of the
percentile convention uses P10/P50/P90 language throughout.

**5.7 REP3 titled "SW<0.45" does not implement it.** Prose describes the optional
report as "Sw < 0.45 (optional) - Porosity >=0.1, Clay Volume <= 0.5, **Water
Saturation <= 0.45**" (`cutoffsandsummation.htm`). The corresponding screenshot shows
Report 3 with Water Saturation `Cut Value` = **0.5** and its `Use` box **clear**
`[img-read: _candsclip0012.png]`. The report title string is the only thing carrying
0.45. Likewise REP4 is described as including "Water Saturation <= 0.5" but its Sw
`Use` box is also clear in the screenshot. The screenshot and the prose disagree on
both the value and whether Sw is applied at all.

**5.8 Dependency correlation: prose 0.5 vs screenshot 0.8.** Prose says the
illustration shows "an m and n dependency correlation of **0.5** and a Neu Wet Clay
and Rho Wet Clay dependency correlation of -0.8"
(`define_monte_carlo_parameters.htm`); the Dependencies grid shows m↔n = **0.8**
(and Rho Wet Clay ↔ Neu Wet Clay = −0.8, which agrees)
`[img-read: _mceaclip0058.png]`.

**5.9 Input-curve default shifts described as "±10 % of the valid value".** The Input
Curves section says "Low / High Values - default values are filled in which are equal
to the **Mineral end-point value** plus or minus 10 % of the valid value"
(`define_monte_carlo_parameters.htm`) — wording lifted verbatim from the Mineral
Solver section later on the same page. Input *curves* have no mineral end-points, and
the observed shipped defaults are not ±10 %: RHOB ±0.02 (≈0.8 % of ~2.5 g/cc), DTLN
±2.0 (≈3 % of ~70 µs/ft), LLD ±0.005 reciprocal `[img-read: _mceaclip0057.png]`. Only
TNPH is actually a percent shift, and it is 5 %, not 10 %. The sentence is a
copy-paste defect.

**5.10 `Sw Res Use` clear by default vs "Set to On to use".** The parameter
description reads "Sw Res Use: Flag - Set to On (selected) to use the water
saturation input for a Net Reservoir cut-off" (`cutoffsandsummation.htm`), which is
neutral, but the *Reservoir Results* description says net reservoir is determined by
"the Porosity and optional Clay Volume cut-off criteria" — Sw is not mentioned at
all. The shipped panel confirms `Sw Res Use` **clear** `[img-read: _candsclip0020.png]`.
Not a contradiction, but the manual never states plainly that Sw is off for Reservoir
by default; it must be inferred from the panel. Recorded so SandiBumi does not
default it on.

**5.11 Multi-well example report arithmetic does not reconcile.** In the worked
listing (`multi-wellcutoffsandsummati.htm`), the `All Wells` Net Reservoir row for
`Mid Shale` gives Gross 19.50 / Net 6.50, but the two contributing wells give Gross
13.50 + 25.50 = 39.00 and Net 2.75 + 10.2 = 12.95. Several `All Wells` rows in the
listing are inconsistent with their member wells (`A Sand`, `All Zones`), and the
`XYZ 1` / `XYZ 3` `All Zones` rows carry identical `Phi*H` / `Phi*So*H` values
(9.832 / 8.252) despite different inputs. The listing appears to be a hand-assembled
or stale example. Do not use it as a numerical conformance fixture.

---

## 6. IP2018 → IP2025 numeric diff (tasking d)

Method: deterministic. All 8 pages exist in both trees (two renamed:
`multi_wellcutoffsandsummati` → `multi-wellcutoffsandsummati`,
`cut_off_sensitivity_results` → `cut-off_sensitivity_results`). Tags stripped, text
normalised, numeric-token multisets compared in both directions, then a sentence-level
diff filtered to sentences containing digits.

**Numeric defaults changed: exactly one.**

| Page | Quantity | IP2018 | IP2025 |
|---|---|---|---|
| `cutoffsandsummation` | Max input curves on the Input Curves tab | **10** | **50** |
| `cutoffsandsummation` | Wording for extra curves | "The additional 7 curves (rows 4 - 10)" | sentence deleted; lead-in changed from "up to 7 input curves" to "multiple input curves" |

Everything else that changed on that page is **example-listing data**, not defaults:
the 1998/2002-dated sample summation reports were removed and replaced with image
links (`Example listing 1 - Standard Report`, `Example listing 2`). All the removed
numerics (0.01823, 0.02743, 7790.00, 0.132, …) belong to those deleted listings.

**Unchanged 2018 → 2025 (zero numeric tokens differ in either direction):**

- `define_monte_carlo_parameters` — **every Monte Carlo default is identical**: 4
  standard deviations, ±2.5 σ truncation, ±2 σ tornado runs, 200 burn-in, 100 check
  interval, 300 minimum, 0.1 % tolerance, 10 output curves, 9 histograms, 9
  crossplots, correlation 0/1/−1 semantics, the three distribution types, the three
  shift types and their formulas.
- `cut-off_sensitivity_results` — 0.0–35 pu / 1.0 pu example, 10/50/90 percentiles.
- `batchformula` — all extensions and behaviour.
- `cm_curve_statistics` — 50-cell mode histogram, 3-per-cell guidance.
- `batchmontecarlo` — 1–2 test simulations, 2000 final.
- `multi-wellcutoffsandsummati` — no numeric change; the sample report listing is
  byte-identical.

**Non-numeric additions in 2025** (`cutoffsandsummation.htm`,
`multi-wellcutoffsandsummati.htm`): the whole **CO2 storage workflow** —
CO2 Fluid / CO2 Solution / CO2 Storage input curves; `Fluid Efficiency` and
`Solution Efficiency` multipliers on the Pay Cutoffs tab (default **1**, with a
double-application warning); the **`Output Curve Cum Crv`** column (simple summation,
no height factor) alongside the existing `Cum CrvH`; and the `ResCO2*` / `Stor*`
output-curve family. `multi-wellcutoffsandsummati` also drops "Working with Curves in
Multi Wells" from Related Topics.

**Cross-check against the prior IP2018 ingest**
(`ip2018_chm_ingest/C_cutoffs_defaults_mc.md`) — two of its recorded gaps are now
closed:

1. That ingest recorded the six averaging equations as **"rasterized - not
   recoverable"** (its line 128, for `embim164.gif` = geometric). **All six are now
   transcribed** (§2.2). Note the raster numbering shifted between editions: in 2025,
   `embim163` = geometric and `embim164` = harmonic.
2. That ingest concluded "The 2000 figure is a **recommendation in the batch page's
   prose, not a stated program default**" (its lines 508–511, 827). The 2025 Output
   tab screenshot shows `Stop simulation at` populated with **2000 Iterations** in the
   single-well MC dialog `[img-read: _mceaclip0038.png]`, `[img-read: _mceaclip0039.png]`.
   **2000 is the shipped dialog default**, not merely batch advice.

Also newly captured versus that ingest: `Update Graphics every` = 20; Output
Percentiles 10/50/90; Result Curves P5/P50/P95 with `_mn`/`_psd`/`_msd`/`_P*`/`_mc`
extensions; auto-stop `Result Parameter` = `PhiSoH Res` and `Result Zone` = `All`;
`Save all output curve simulations` = on; the per-curve MC shift defaults (§3.7); the
Wellbore Stability uniform ±20 % Gaussian; the Reservoir/Pay `Use`-flag defaults; the
`Fluid`/`Solution Efficiency` = 1; and the worked net-thickness diagram (§2.1).

---

## 7. SandiBumi notes

1. **Adopt the half-interval rule as a hard contract.** §2.1 is the single most
   testable behaviour here and the worked diagram gives a ready-made unit-test
   fixture (100.0–104.0 ft, step 0.5, flag pattern `0,0,1,1,1,1,1,1,1` → Net = 3.25).
   Any engine that gets this wrong is off by up to a depth step per zone on every
   net, and the error compounds through N/G, PhiH and PhiSoH.
2. **Do not copy IP's geometric mean.** §5.2 — `(ΠCᵢ)^(1/Σhᵢ)` is step-size dependent
   and returns the true geometric mean only when the sample interval is exactly 1.
   SandiBumi should implement the thickness-weighted geometric mean
   `exp( Σ hᵢ ln Cᵢ / Σ hᵢ )`, which is the natural companion to the arithmetic and
   harmonic forms IP actually uses, and **document the divergence** so IP-comparison
   runs are expected to differ on geometrically-averaged extra curves. Flag this to
   whoever owns numerical conformance testing.
3. **Sw averaging is porosity-weighted, everywhere.** Single-well (§2.2), multi-well
   roll-up (§2.4) and field averages all recover Sw from PhiSoH/PhiH. Implement once,
   share.
4. **Cut-off defaults ship as Phi ≥ 0.1 / Sw ≤ 0.5 / Vcl ≤ 0.5, with Sw OFF for
   Reservoir and ON for Pay** (§3.2, §3.3). Record these as *IP defaults*, never as
   petrophysical recommendations. For Mahakam delta work in particular, a 0.5 Vcl
   cut-off and a 0.5 Sw cut-off are inherited vendor placeholders, not values traceable
   to any of Jauhar's studies — SandiBumi must require the user to set them, or cite
   a project source, rather than silently seeding them.
5. **Min Height default 0 means "no minimum".** Worth surfacing explicitly in any UI:
   users frequently assume a bed-resolution floor is applied and it is not (§3.1).
6. **Cut-offs combine with AND only.** If SandiBumi wants richer logic it is a genuine
   differentiator, but the migration path from IP must map IP's 5-report structure
   onto it losslessly (§3.1).
7. **Zone membership gates cumulation ahead of cut-offs** (§3.1) — an easy behaviour
   to get wrong when cumulative curves are computed in a single pass.
8. **Monte Carlo: implement a settable seed.** IP's clock-seeded RNG with no seed
   field (§3.7) makes runs unreproducible; a seed field plus stored seed in the
   results is a cheap, real improvement. Keep IP's ±2.5 σ truncation available as an
   option for conformance, but note it biases realised variance low (§4.8).
9. **Percentiles are computed per-parameter, so P50 quantities are not
   self-consistent** (§4.6). If SandiBumi instead reports a *joint* iteration (the
   iteration whose PhiSoH is the P50), say so loudly — the numbers will not match IP
   and the difference is a feature, not a bug. IP's own
   *Show/Output individual iteration result* workflow is effectively an admission of
   this and is worth reimplementing properly.
10. **The Sw percentile flip is real and configurable in IP** — P10 means 10th-lowest
    for everything except Sw, where it means 10th-highest, driven by the Results
    section of `MonteCarloDefaults.par` (§3.7). Two IP installations can therefore
    report different P10 Sw from identical data. SandiBumi should fix one convention
    and label it on every output.
11. **MC shift defaults worth carrying as a starting uncertainty template** (§3.7):
    RHOB ±0.02 g/cc linear, DT ±2 µs/ft linear, NPHI ±5 % percent, deep resistivity
    ±0.005 in **reciprocal** (conductivity) space, GR clean/clay endpoints ±10 GAPI
    linear — all Gaussian. The reciprocal treatment of resistivity is the sound part
    of IP's design and directly addresses the "resistivity goes negative" failure it
    warns about. These are IP defaults; cite them as such, do not present them as
    measurement accuracy.
12. **Result Precision 3 with an 8-character output field** (§3.4) is a formatting
    trap IP documents against itself — SandiBumi should not inherit the 8-char cap.
13. **The `MonteCarloDefaults.par` index stride of 3** (§3.7) is the only visible clue
    to that file's layout. If a real `.par` file is ever available, it would resolve
    several open items at once.

---

## 8. OPEN ITEMS

1. **`MonteCarloDefaults.par` contents are never shown** — only the filename, its IP
   directory location, the (3)/(6)/(9)/(12)/(15)/(18) index stride, and the fact that
   the Results section controls the percentile convention and the default
   crossplots/histograms. Carried forward unresolved from the IP2018 ingest.
   Resolvable only from a real installation.
2. **§5.3 — whether Reservoir and Pay can hold different cut values after Run.** The
   setup grid implies yes, the Parameters window's shared `Res/Pay` column implies no.
   Needs a live IP session to settle. This materially affects how SandiBumi models
   the cut-off record (one value + two flags, vs two independent values).
3. **§5.4 — true input-curve capacity: 50 or 7.** The prose contradicts itself and
   every screenshot still shows a 10-row grid. Needs a live session.
4. **Default Low/High shift values for the Clay Volume, Porosity Sw and Cutoff MC
   parameter tabs are not fully shown.** Only `Gr Clean` / `Gr Clay` (±10) are
   visible `[img-read: _mceaclip0070.png]`. The defaults for `m`, `n`, `a`, `Rw`,
   `Rt`, `Phi cut`, `Sw cut`, `Vcl cut` etc. are **not** on any image on these pages.
   The `_mceaclip0024.png` crossplot axis extents suggest m and n shifts on the order
   of ±0.29 at the ±2.5 σ truncation limit, which would imply a Low/High entry near
   0.25 — **this is an inference from an axis label, not a documented value, and is
   deliberately not recorded as a default.** Another agent covering the Porosity/Sw
   or Clay Volume pages may find the populated panels.
5. **Whether the "Gaussian" shape supports asymmetric σ.** Low Shift and High Shift
   are independently settable and jointly define 4 σ, but the manual never states
   whether an asymmetric pair produces a two-piece Gaussian (different σ each side) or
   a symmetric one about a shifted mean. `_mceaclip0005.png` draws unequal Low/High
   arrows under a visually symmetric bell without resolving it.
6. **Triangular and Square distributions with asymmetric Low/High** — the figure shows
   the Start Value as the triangular apex with unequal legs, implying the mean is not
   the apex, but no formula is given.
7. **Exact definition of `ResPhiSoH` as "Cumulative Reservoir Thickness (net)".** The
   output-curve list labels `ResPhiSoH` as "Cumulative Reservoir Thickness ( ResPhiSoH
   - net)" while the Results tab defines `PhiSoH Res` as hydrocarbon pore thickness
   (`cutoffsandsummation.htm`). The parenthetical "- net" gloss is unexplained;
   probably an editing slip in the bullet list, but not certain enough to assert.
8. **§5.7 — whether IP's shipped REP3 example actually applies a 0.45 Sw cut.** The
   title says 0.45, the grid says 0.5 and unticked.
9. **Whether `Min Res Height` / `Min Pay Height` are in measured or true vertical
   thickness** when a TVD curve is selected. Never stated.
10. **Units of the Height (`h`) term in the averaging equations.** The formulas are
    unit-agnostic; the worked diagram uses feet with a 0.5 ft step. Since the
    geometric-average exponent is `1/Σhᵢ` (§5.2), the geometric result is **not**
    unit-invariant — a metric and an imperial run over the same interval give
    different answers. Not acknowledged anywhere in the manual; recorded here as a
    consequence of the transcribed formula, not as a vendor statement.
