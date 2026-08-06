# M — Production Logging, Cased Hole, Cement Evaluation

**Source:** Interactive Petrophysics 2025 vendor manual (decompiled CHM), agent M of 14.
**Extraction date:** 2026-08-06.
**Provenance convention:** `(pagename.htm)` = prose on that help page; `[img-read: file.png]` = transcribed by reading the PNG directly (vision). `[derived]` = arithmetic I performed on vendor-printed numbers, shown with the check.
**Non-negotiable:** nothing in this file is a textbook value. Where the manual is silent, it says so in §8 OPEN ITEMS rather than being filled in.

---

## 1. Scope & page inventory (29 pages, all accounted for)

| # | Page stem (`_text.txt` / `.htm`) | Title | Chars | Content imgs | Status |
|---|---|---|---|---|---|
| 1 | `cementeval` | Cement Evaluation | 71,012 | 74 | Read in full (817 lines, 2 passes) |
| 2 | `multiphaseflowcalculations` | Multiphase Flow Calculations | 37,166 | 16 | Read in full |
| 3 | `casinginspection` | Casing Inspection | 24,341 | 30 | Read in full |
| 4 | `plset` | Production Log Interpretation Set-Up | 22,641 | 15 | Read in full |
| 5 | `chronolog_help` | Output Time Curves to Depth Well (Chronolog) | 22,202 | 55 | Read in full |
| 6 | `spinnercalibrationapparentvelocity` | Spinner Calibration and Apparent Velocity | 18,738 | 14 | Read in full |
| 7 | `multiphase_array_flow_calculat` | Multiphase Array Flow Calculations | 14,809 | 11 | Read in full |
| 8 | `flow_from_temperature` | Flow Estimation from Temperature (oil-water) | 14,342 | 10 | Read in full |
| 9 | `selectiveinflowperformance` | Selective Inflow Performance (SIP) | 11,469 | 12 | Read in full |
| 10 | `production_log_analysis_module` | Production Log Analysis Module | 8,468 | 25 | Read in full |
| 11 | `depth-based_pl_passes_-_ascii_` | Depth-Based PL Passes — ASCII Format | 7,390 | 5 | Read in full |
| 12 | `multiphaseinflowcurves` | Multiphase Inflow Curves | 4,570 | 2 | Read in full |
| 13 | `time-based_pl_station_data_-_l` | Time-Based PL Station Data — LAS | 4,675 | 6 | Read in full |
| 14 | `depth-based_pl_passes_-_las_fo` | Depth-Based PL Passes — LAS Format | 4,386 | 5 | Read in full |
| 15 | `time-based_pl_station_data_-_a` | Time-Based PL Station Data — ASCII | 3,533 | 2 | Read in full |
| 16 | `pllateralaverage` | PL Lateral Average | 3,398 | 1 | Read in full |
| 17 | `import_pl` | Import PL | 2,252 | 0 | Read in full |
| 18 | `pltavailablereports` | PLT — Available Reports | 1,981 | 1 | Read in full |
| 19 | `plt-availablereports` | PLT — Available Reports (duplicate topic) | 1,899 | 1 | Read in full |
| 20 | `import_pl_for_sondex_maps_tool` | Import PL for Sondex MAPS tools | 1,740 | 4 | Read in full |
| 21 | `pl_array_workflow` | PL Array Workflow | 1,199 | 2 | Read in full |
| 22 | `cased_hole` | Cased Hole (hub) | 1,039 | 1 | Read in full |
| 23 | `spinnerthresholds` | Spinner Thresholds (PLWin) | 1,034 | 0 | Read in full |
| 24 | `importing_plwin_wells` | Importing PLWin Wells | 1,017 | 1 | Read in full |
| 25 | `production_logging` | Production Logging (hub) | 952 | 0 | Read in full |
| 26 | `sensorplot` | Sensor Plot | 600 | 2 | Read in full |
| 27 | `rebuildsensorplot` | Rebuild Sensor Plot | 385 | 0 | Read in full |
| 28 | `bubble-analysis` | Bubble Analysis | 124 | 0 | **STUB** — body is literally the authoring placeholder "Enter topic text here." No content. |
| 29 | `terminal-events` | Terminal Events | 124 | 0 | **STUB** — same placeholder. No content. |

**Images read (vision), 30 total** — selected as those where a marker sits where a number, chart, or equation belongs:
`_cemclip0004, 0019, 0021, 0031, 0044, 0054, 0055, 0056, 0061, 0070, 0071, 0073, 0075, 0076, 0077`;
`_ciclip0004, 0005, 0018, 0019`;
`_plclip00082, 00092, 00094, 00096, 00098, 00117, 00118, 00119, 00120, 00121, 00122, 00123, 00126, 00132, 00145, 00160, 00163, 00164, 00318, 00322, 00323`.
The remaining ~180 images on these pages are UI chrome (`contents_off`, `menu`, `arrow_*`, `printer`, `eye`, `ipexpandarrow`), toolbar icons (`_plclip00004`–`00022`, 0.9–2.3 kB each), cursor glyphs (`_cemclip0024/0025/0026/0038/0039/0040`, 244–885 bytes), or workflow/menu/log-plot screenshots carrying no numeric content beyond what the prose states.
Zoom-suffix handling: `_cemclip0058_zoom50` and `_cemclip0074_zoom50` both have suffix-free siblings — neither was needed. `_cemclip0016_zoom70` / `_cemclip0017_zoom70` have **no** suffix-free sibling; both are curve-selection dialogs with no numeric content, so no loss.

**Two pages carry no extractable content at all** (28, 29). They are shipped placeholders, not truncation artefacts — flagged so no other agent re-hunts them.

---

## 2. Equations & correlations per module

### 2.1 Spinner Calibration and Apparent Velocity — tasking (a)

The manual prints these two equations as ASCII, not as rasters:

**Individual apparent velocity, per spinner/pass**
```
Vapp(i) = Spinner speed / Slope − Cable Speed + Threshold
```
(spinnercalibrationapparentvelocity.htm, line 243)

**Weighted apparent velocity (the output curve)**
```
            W(1)·Vapp(1) + W(2)·Vapp(2) + … + W(n)·Vapp(n)
Vapp  =  ───────────────────────────────────────────────────
                     W(1) + W(2) + … + W(n)
```
(spinnercalibrationapparentvelocity.htm, lines 245–246). The manual states explicitly: *"It is not necessary that the weights add up to 1.0, the program normalises the weights."* — so the denominator is real, not decorative.

**Calibration-line fitting math (as documented):**
- The module computes, per calibration zone and per selected spin curve, the **average** spin and the **average** cable speed over that zone (one point per zone per pass). Those zone-averaged pairs are the regression input — *not* the raw samples. (spinnercalibrationapparentvelocity.htm)
- Two separate best-fit lines per zone: one through the **positive**-spin points, one through the **negative**-spin points. Slope and velocity intercept are reported for each. (same)
- **Degenerate case rule:** *"If there is only one point for a positive or negative part of a zone then the slope will be set to the corresponding opposite slope."* (same) — a hard fallback SandiBumi must replicate.
- A **mean** positive slope and mean negative slope are computed as the plain average of all zonal slopes; the user may apply either mean to all zones, or edit slopes individually.
- Points can be disabled by clicking; the affected zone's slope is then recomputed. Changing a zone's depth **re-enables** all its previously disabled points ("as the depth changes could have been done to find better points").
- Stationary spinner readings are entered in **rps** directly into the zone table and join the regression; blanking the box removes them.

**Slope-to-depth mapping for the continuous curve** (this is the slippage-free part of the velocity computation and is stated twice):
> use the slope **at the middle of each specified zone**; extrapolate the top-most zone's slope to the top of the well; extrapolate the bottom-most zone's slope to the bottom of the well; **interpolate** slopes between the mid-points of adjacent zones.
(spinnercalibrationapparentvelocity.htm, lines 161 and 203)

Consequence the manual draws out: where casing ID or fluid type changes, calibration zones must be created **at** the change depth even if spinner data there is poor, so the interpolation happens over a short interval; the zone's slope is then edited by hand to the value seen in a good zone of the same ID/fluid.

**Threshold handling:**
- Thresholds represent the spinner's resistance to starting to spin (friction, tool design, wellbore fluids). Positive and negative thresholds are independent.
- They must be **picked in situ from the spinner/cable-speed regression plot, not from tool specification tables**. (spinnerthresholds.htm and spinnercalibrationapparentvelocity.htm — identical wording on both pages)
- Picking rule: in a no-flow zone, the velocity **intercept** of the fitted line is the threshold. The program only offers the "update threshold" prompt when it determines the zone *could* be a no-flow zone. Confirmed by the dialog text: *"Update positive spinner speed threshold with zonal intercept value of 1.52"* `[img-read: _plclip00092.png]`.
- **Sign constraint (enforced):** the intercept of a positive slope in a no-flow zone must be positive; of a negative slope, negative — "as it represents the force needed to overcome spinner friction." Wrong sign raises an error dialog. Negative thresholds may be typed as either positive or negative numbers.
- The threshold is *a function of the assigned slope* — slope must be settled first, then the threshold defined. First-defined thresholds are copied to all calibration zones and may then be edited per zone.
- Scale of effect: **"a velocity of 1 ft/min represents 100 rb/d in a 9 5/8" casing and 50 rb/d in 7""** (both pages).

**Spinner reversal (Schlumberger GMS/PTS only):** these tools do not record spin direction, so data is always positive. Spin is made negative **below** each entered reversal depth, inside the program (do not pre-multiply by −1). Multiple reversal depths allowed for crossflow, entered in increasing depth left→right; a double reversal (×−1 ×−1) leaves the data positive. Even an always-negative pass needs the top of the well entered as a reversal depth. Sign convention: **upward flow relative to the tool = positive spin**. Does **not** apply to FSI and MAPS (directional), nor to most Schlumberger CPLT spinners.

**Quicklook (field/memory-tool shortcut, not the main path):**
```
Velocity_fluid = C × Velocity_apparent            (C is a single fixed factor)
Q_total        = Velocity_fluid × Area_inside_casing      (reservoir conditions)
```
(spinnercalibrationapparentvelocity.htm, lines 258 and 262). The manual is explicit that this differs from the main path: *"All velocity values will be corrected by the same amount … and not, as in Multiphase Flow Calculations, by a Reynold's number dependent correcting factor, which becomes larger for higher velocity values."*

### 2.2 Multiphase Flow Calculations — tasking (b)

**Mixture velocity from apparent velocity.** The manual names the mechanism but *does not print the formula*. What it does state:
- The conversion is a **Reynolds-number-dependent** correction, a function `f(Re, D_casing, D_spinner)` (multiphaseflowcalculations.htm, line 210 lists exactly this signature).
- *"In oilfield flow we have always turbulent flow with a conversion factor based on the Reynold's number usually between .8 and .95 (Vmix/Vapp)."*
- *"when the viscosity is high, the Reynold's number is linearly reduced and we seem to have laminar flow with a low conversion factor between .5 and .6. This is clearly wrong."* → hence the viscosity clamp.
- Spinner diameter enters here (`Spinner Diameter: Is used in the mixture velocity calculation from the apparent velocity`); tool diameter enters the **Fanning** friction correction; relative roughness enters the **Colebrooke** friction-factor correction.
- The Reynolds correction factor multiplier *"causes a direct multiplication of the mixture velocity by this factor"*.

**Named correlations, exactly as the manual names them** (multiphaseflowcalculations.htm + `[img-read: _plclip00122.png]`, the "Slippage correction for bubble flow" dialog, which is the authoritative enumeration):

*Slippage model (global):*
- `Standard slippage model`
- `Standard slippage model with reversal above 90 degrees deviation` — with an editable ramp: **"slippage is reduced to zero at 90 degrees over [3] degrees"** `[img-read: _plclip00122.png]`.

*Oil–Water:*
- `Standard 1 (chart for vertical well)` — "represents this chart" (the chart in the Slippage Section of the PL User Manual; the chart itself is **not** reproduced in the CHM).
- `Deviated (chart with modified Ew dependency for deviated well)` — annotated in the dialog: *"VSLIP corrected for deviation / VSLIP multiplier always about 1.0"*. Documented behaviour of the modification: *"reduces the Vslip dependency on the water hold-up by keeping the value for Ew = 0.6 and reducing the chart's difference for the actual Ew value — boosting the slippage for lower Ew values and reducing it for higher ones."*
- Checkbox `Do not allow water fallback` (ticked in the shipped screenshot).
- Dialog also displays a computed diagnostic: **"Average deviation over top 25% = 30.0 degrees"** — i.e. the auto-selection of the Deviated option is driven by the mean deviation over the **top 25 %** of the interval.

*Gas–Liquid:*
- `Pressure dependent correlation` (correlation 1) — "computes a slippage velocity dependent on pressure".
- `Correlation similar to oil-water standard` (correlation 2) — "based on a similar slippage velocity dependence on heavy phase hold-up and inter-facial tension as for oil/water".
- `Apply slippage velocity deviation multiplier`, with **two printed coefficient forms** `[img-read: _plclip00122.png]`:
  ```
  gas-liquid BUBBLE flow:          Vslip × (1 + dev/25)
  gas-liquid SLUG or CHURN flow:   Vslip × (1 + (dev/25) × 0.5)
  ```
  `dev` = deviation in degrees. The **0.5** is an editable field (default 0.5 as shipped). The prose (line 173) states only the bubble-flow form; **the slug/churn form and its 0.5 factor exist only in the raster.**
- `Force Bubble Flow` — "Slug and churn flow will be forced to act as bubble flow." Provided from **version 5.0**, for data near the flowmap's bubble↔slug/froth border where "the phase flow results may become jumpy … due to the higher slippage assigned to slug/froth flow."

*Gas–Oil–Water (three phase):*
- `Slippage between phases`: `Between individual phases` (default per prose) / `Between major and minor phases (original)` / `None`.
- *"The program module combines 2 phases (oil + gas or water + oil) when one of them is small."*
- `Minimum percentage gas of oil holdup` — value in %, "if the gas indicator is suspect and the user has reason to believe that gas is present".

*Multipliers and dependency caps* `[img-read: _plclip00122.png]`:
- `Gas-liquid slippage multiplier` 1.00; `Oil-water slippage multiplier` 1.00; `Slippage multiplier in gas-liquid downflow` 1.00; `Annular Mist slippage correction multiplier` 1.0.
- `Maximum Vslip = 2.0 × mixture velocity`
- `True Downflow: Max. Vslip = 1.0 × mixture velocity`
- `Use legacy method for fluid split` (ticked as shipped) — prose: *"Also for true downflow the complicated flow computations relating Vmix to Vslip have been updated. To match earlier computed results, use the legacy method."*

**Apparent-downflow (a.d.f.) flow equation — printed in ASCII:**
```
Qoil = Eoil × 0.5 × Vslip × Area
```
(multiphaseflowcalculations.htm, line 199). Physical justification given: the pressure-based density measures the average fluid gradient, so light-phase hold-up is calculable; the light-phase velocity in stratified flow relates to Vslip but "the upflow is slowed by the turbulence, which creates this apparent downflow." **For gas or condensate in a.d.f., the slippage is forced to the pressure-dependent correlation** regardless of the user's Gas–Liquid selection.

**Flow-regime logic, as documented** (multiphaseflowcalculations.htm §Gas):
- *"Whenever gas is one phase of the multiphase flow computation, the flowmap determines the flow regime."* The flowmap itself is **not reproduced** in the CHM (see OPEN ITEMS).
- Regime definitions given: **Bubble** (discrete bubbles ~uniformly distributed in continuous liquid); **Slug** (Taylor bubbles ~pipe diameter, moving up uniformly, separated by liquid slugs, thin falling film at the wall); **Churn** ("similar to slug flow … much more chaotic, frothy and disordered"); **Mist** (continuous gas core, liquid as entrained drops or a waxy wall film).
- **Slug vs churn is not decided by the flowmap.** It is decided by a user-entered **entry length LE**: *"the distance that such churning can be observed from the entry before stable slug flow takes place, depends on the flow rates and pipe size … The entry length is the distance between a major fluid inflow into the well and the top of the reservoir zone being worked on."*
- Critically: **"all computations, especially the slippage velocity, are the same for slug and churn flow."** The slug/churn distinction is therefore *reporting only* for the base Vslip — but note it **does** change the deviation multiplier branch is shared (both use the ×0.5 form), so the split matters only versus bubble flow.

**Zonal contribution arithmetic:**
- Default (`With wellbore expansion` **not** ticked): zonal flow of each phase = flow above the current zone − flow above the zone below, **at reservoir conditions**, representing wellbore flow.
- `With wellbore expansion` ticked (for gas wells with long unperforated intervals): the difference is taken **at surface conditions**, then converted back to reservoir conditions with that zone's PVT phase volume factors — "in this way spinner derived flow between the zones (due to PVT parameter changes) is excluded." Manual's own caveat: *"The zonal contributions do not necessarily add up to the total flow at reservoir conditions any more."* The zonal **surface** flow is identical under both options.
- The reported zonal results at zone **tops** are cumulative; true zonal results (the differences) appear with the zone **bottom** on the last page of the flow report only.

**Fluid-property mapping to depth:** within a reservoir zone, properties are constant at that zone's values; **between** zones, interpolated; **above** the top zone, top-zone values; **below** the bottom zone, bottom-zone values.

**Automatic surface-rate matching** is available for **monophasic and oil/water only**. It adjusts the Reynolds correction factor multiplier "to the nearest surface production rate value within the defined 4 digit precision", and for oil/water also the density shift, iteratively (a new density shift needs a changed Reynolds multiplier). It leaves slippage multipliers unchanged. Also available when hold-up comes from a DEFT or a hold-up-based capacitance curve (0–1). **Not** provided when capacitance uses a raw-cps conversion chart, "as the matching usually involves changes of the charts water and hydrocarbon point values and the capacitance shift."

**Absent-data rules:** absent (−999) pressure/temperature → zonal extrapolation values used. Absent VAPP → the DENM curve is computed **without friction correction**.

### 2.3 Multiphase Array Flow Calculations (MAPS)

- Two-phase downhole only in this release: Oil/Water or Gas/Water. *"if Oil/Water model is selected a Gas rate will also be calculated as surface conditions using PVT conversions."*
- The borehole is divided into **N horizontal segments**; flow is computed per segment from that segment's velocity and hold-up, then summed. Slippage is applied **only to segments containing mixed fluid**.
- Default slippage: *"assumes a vertical borehole and uses the fluid holdup in each borehole segment and density difference between fluids to calculate a 'Vertical Slippage' from a conventional slippage model."* The model is not named further.
- User-defined slippage: a fixed value = "the velocity difference between the lighter hydrocarbon phase and the heavier water phase", entered in f/mn `[img-read: _plclip00322.png]`.
- **Hole-deviation weighting of slippage.** The manual admits there is no industry model: *"In the absence of any industry accepted models for correcting slippage in highly deviated wells the following approach has been taken."* Assumptions stated: the model/user value is the **maximum** slippage, occurring when vertical; slippage is **zero in a perfectly horizontal borehole**. The weighting is given **only as a chart** `[img-read: _plclip00323.png]`, titled *"Weighting on Vertical Slippage based on Hole Angle"*, Y = "Slippage Deviation correction", X = Hole Angle 0–180°. Read points: **(0°, 1.0), (30°, ≈0.87), (60°, ≈0.5), (90°, 0.0), (120°, ≈−0.5), (150°, ≈−0.87), (180°, −1.0)**. That is numerically `cos θ` to plotting accuracy — but **the manual never prints the formula**, so `cos θ` is my inference, not a vendor statement (see OPEN ITEMS).
- Sensitivity direction, stated: ↑Velocity multiplier → ↑total production from all zones; ↑Holdup multiplier → ↑water, ↓hydrocarbon; ↑Slippage multiplier → ↑hydrocarbon, ↓water. Default multipliers all 1.0.
- `Surface Match` solves for the optimum **velocity and holdup** multipliers to best match measured surface rates and re-runs immediately (slippage multiplier is not solved for).
- Segment-count constraint: *"if the number of segments chosen results in upper or lower segments which do not contain any valid data then the user will be warned to reduce this number."*
- Array image orientation convention: *"the left side of the images representing the low side of the borehole and the right side … the high side."*

### 2.4 Flow Estimation from Temperature — tasking (e)

**The model, as stated:** the program assumes the **friction heat is part of the temperature curve at the depth of first flow**, and draws a straight line through the temperature value at that depth with the **vertical geothermal gradient**. That line is the output curve `GEOTH_FH` — "the geothermal temperature shifted by the friction heat." (flow_from_temperature.htm)

Validity gate the manual imposes: *"The values of the friction heat shifted geothermal temperature curve must be lower than the measured temperature curve everywhere above the depth of first flow."*

**The three constants of the model** (flow_from_temperature.htm, confirmed on the dialog `[img-read: _plclip00164.png]`):

| Constant | Value | Units (as displayed) | Note |
|---|---|---|---|
| **Friction heat** | 2.000 (example) | deg F | "the absolute temperature shift between the geothermal temperature and the inflowing temperature", observed at first inflow |
| **Conductivity loss** | **0.250** (default) | `deg/deg /ft @ 1 bl/d` (dialog string) — prose says **0.25 degF/ft @ 1 bbl/d** | heat loss to the surrounding cooler formation |
| **Oil expansion factor** | **1.000** (default) | `deg F/(1000 vertical ft)` | heat loss from oil expansion as pressure drops going up |
| Vertical geothermal gradient | 0.020 (example) | deg F per ft | *"characteristic for a field … should be provided by the Oil Company"* |

**Friction-heat magnitude rule (the calibration heuristic):**
> *"From extensive work with (North Sea) datasets it can be concluded that the friction heat mainly depends on the drawdown pressure. A first approximation of the drawdown pressure is the pressure difference between shut-in and the particular flowing well condition. As a rough guidance, a friction heat of 1 degree F can be expected per 1000 psi drawdown."*

**The load-bearing assumption, stated as a warning:** *"the module only works if we assume that the friction heat is more or less rate independent, which we found to be true. However, differential depletion between different reservoir sections would change the drawdown and therefore the friction heat and would add an error to the flow rate computation."*

**No friction correction on density is possible here** ("no fluid velocity curve is usually available"). Magnitude given for when it matters: *"under 10,000 rb/d in 7" casing the correction is less than 1%."*

No zonal contributions are output by this module; it outputs an estimated velocity curve (`VATE…`) that can be fed to Multiphase Flow Calculations.

### 2.5 Selective Inflow Performance — tasking (f)

**The fitting math, as documented** (selectiveinflowperformance.htm):
- Per reservoir **layer**, plot **zonal pressure (y) versus zonal total flow rate at reservoir conditions (x)** across all selected well conditions, and fit a regression line.
- **Slope of the line = Productivity Index (or Injectivity Index).**
- **Intercept at Q = 0 = the Sand Face Pressure** at measured depth; when all layers are referred to a datum depth, the intercept is called the **Potential**.
- Data requirement: *"at least 3 rates and a shut-in survey should have been logged."* Confirmed by the shipped condition list P1 High / P2 Medium / P3 Low / S1 Shut-in / P4 Very high `[img-read: _plclip00145.png]`.
- "Total" = the sum of all fluid-phase contributions, read as `QTOT…` at the **top** of the zone (i.e. the cumulative flow), from the array the Multiphase Flow Calculations module wrote.
- Only **one** shut-in condition may enter the regression; the user picks which.
- **Sump is excluded** — "All defined reservoir zones, with the exception of the sump, may be selected."
- Conditions computed **with wellbore expansion** are displayed in blue.
- Values may be edited in the SIP table; edits feed the regression but are **not** written back to the Multiphase Flow Calculations results.

**Shut-in admissibility rules (validity gates, not math):** shut-in data is valid for SIP *"only when real crossflow is observed"*. If a strong downflow is actually an apparent downflow caused by light-phase upflow, either edit in the estimated upflow as the zonal flow, or exclude shut-in. If the crossflow is a well-stabilisation artefact, exclude shut-in.

**QC rule stated as a hard expectation:** *"For a zone the zonal flow contributions cannot decrease when the total (cumulative) flow of the specified well condition is increasing and the data points of all well conditions have to fall on a similar slope for a reservoir zone."*

**Datum variant:** pressures are re-read from the averaged primary pressure curves at the datum depth; if the datum lies outside the well's depth range the user must supply extrapolated pressures "based on depth, deviation and corrected densities" — and **"the curve names on the screen will not be checked."**

**Gas-at-surface variant (`Measured (S)` / `Datum (S)`):** exists specifically because, in gas wells with long inter-zone distances, reservoir-condition flow changes with PVT so the spinner reads different velocities at the top of the lower zone than at the bottom of the higher zone; *"the zonal surface results are correct as they have been computed with the appropriate PVT parameters."* SIP is then run on **surface** gas rates.

### 2.6 Multiphase Inflow Curves

Incremental (inflow) curves are produced by **differentiating the cumulative flow curves** over a user-set differentiation length; there is no measurement of inflow. Rules:
- Input mnemonics carry `E` in the 4th position (`QTOE`, `QOIE`, `QWAE`, `QGAE`) — these are the *editable* copies created in Multiphase Flow Calculations. Output mnemonics: `TINF`, `OINF`, `WINF`, `GINF`, plus water cut `WCUI`.
- *"The shorter this length is, the more mathematically correct are the result curves. 5 or 7 feet seems to give good results."* Shipped dialog value **7.0 ft** `[img-read: _plclip00132.png]`.
- Units transform: input `rb/d` → output `rb/d/ft` `[img-read: _plclip00132.png]`.
- **Light-phase closure rule:** for 2-phase, only total + heavy phase `Q..E` curves are created; for 3-phase, total + oil + water. *"The Multiphase Inflow Curves module will compute the corresponding inflow curves, but also the inflow curve of the light phase by subtracting the heavy phase from the total inflow curve."* → light phase is a **residual**, never independently differentiated.
- Curves are **cut to zero outside the perforated/reservoir zones by default**; the option `Compute values within half length of reservoir zones` recovers the half-differentiation-length of inflow otherwise lost at each zone boundary.
- `Threshold to compute water cut` (rb/d) suppresses water cut below a total-inflow floor, "For very small values of total inflow, the water cut is likely to be spiky and meaningless."
- Manual's own accuracy disclaimer: *"The incremental curves give a good graphical representation … but are not absolutely correct. … The error depends on the differentiation length and the optional filter."*

### 2.7 PL Lateral Average

Averaging of multiple passes uses the **Hodges–Lehmann pair averaging technique** (pllateralaverage.htm) — named explicitly, no formula given. Null (−999) samples are excluded from the average at that depth. Output name = first 6 characters of the input curves + run number.

Caliper special case (printed): `CALIR3 (inchMy98) = (CALIS1R3 + CALIP1R3) / 2` — combining the best shut-in and producing caliper averages in the IP Formula module.

### 2.8 Cement Evaluation — tasking (c)

> **CRITICAL NEGATIVE FINDING.** The IP2025 Cement Evaluation page contains **no bond-index equation, no attenuation equation, and no compressive-strength correlation with coefficients.** "Bond Index" appears twice on the page and only as the *name of a workflow option* ("an alternative to the Bond Index option for determining bond from Casing to Cement"). Verified by exhaustive grep of `cementeval_text.txt` and by reading all 74 image markers' host paragraphs. The strength↔log-response conversion is delegated wholesale to **vendor look-up charts**, which are named but **not reproduced**. Do not expect to reverse-engineer CBL/SBT compressive strength from this manual.

**The strength → log-response chart table (this is the whole of the conversion math)** (cementeval.htm):

| Tool | Chart cited | Inputs | Output |
|---|---|---|---|
| Baker Hughes SBT | Baker Atlas Log Interpretation Charts (Rev. 1 12-95) **Chart 9-1** | Casing Thickness (inch); Cement Strength (psi) | Expected Log Response (**dB/ft**) |
| Baker Hughes CBL | Baker Atlas Log Interpretation Charts (Rev. 1 12-95) **Chart 9-4**, Tool series **1415, 1417** | Casing Outer Diameter (inch); Cement Strength (psi) | Expected Log Response (**mV**) |
| Schlumberger CBL | Schlumberger Log Interpretation Charts 2010 — **Chart CEM 1** | Casing OD (inch); Cement Strength (psi) | Expected Log Response (**mV**) |
| Halliburton CBL | Halliburton Log Interpretation Charts — **Chart CBL 1** | Casing OD (inch); Cement Strength (psi) | Expected Log Response (**mV**) |

Note the **input differs by tool family**: SBT keys on casing **thickness**, all CBLs key on casing **OD**. Ultrasonic and Radial tools use **no chart at all**.

**The one real equation on the page: acoustic impedance from slurry density and transit time.**
The `Calculate Impedance from Density and Transit Time` option converts a cement slurry's density (lb/gal) and travel time (µs/in) to Acoustic Impedance (MRayl). The manual prints no formula, but the shipped grid gives two worked pairs `[img-read: _cemclip0021.png]`:

| Slurry | Density (lb/gal) | Travel Time (µs/in) | Expected Strength (MRayl) |
|---|---|---|---|
| Lead | 10.00 | 9.00 | **3.38** |
| Tail | 13.00 | 7.00 | **5.66** |

`[derived]` The standard product `Z = ρ·v` reproduces row 1 exactly:
```
v [m/s]      = 25400 / Δt[µs/in]
ρ [g/cc]     = ρ[lb/gal] × 0.1198264      (1 lb/US gal = 0.1198264 g/cm³)
Z [MRayl]    = ρ[g/cc] × v[m/s] / 1000
```
Check row 1: v = 25400/9 = 2822.22 m/s; ρ = 10 × 0.1198264 = 1.19826 g/cc; Z = 1.19826 × 2822.22 / 1000 = **3.3817 → 3.38** ✔ (exact to 3 s.f.).
Check row 2: v = 25400/7 = 3628.57 m/s; ρ = 13 × 0.1198264 = 1.55774 g/cc; Z = **5.6523**, but the grid shows **5.66** — a **+0.14 % mismatch**. See §5 Discrepancy D-3.

**Derivative method (lightweight-cement alternative to Bond).** Documented mechanism, no formula:
- Builds a new array from the **rate of change between depth samples** of the bond data.
- **Window Size is a half-length in samples, integer, range 1–25** (cementeval.htm; shipped default **1** `[img-read: _cemclip0071.png]`). "A Window Size of 2 will calculate the derivative based on the data values two samples above and two samples below the current depth."
- Physical basis stated: *"a homogeneous material such as drilling fluid should have a much lower Derivative than a heterogeneous material such as cement."*
- Decision rule: value **below** `Derivative Solidcut` → fluid (blue); **above** → solid (green). Then a coverage % is formed and compared to `Derivative Coverage`.
- Availability constraint: the Derivative tab **does not appear** for Advanced tools (Isolation Scanner, INTex) — "which already have Secondary measurements to deal with lightweight cements."

**Bond coverage → pass logic (the core CBL/VDL interpretation rule as stated):**
1. Colour the bond map per depth-and-azimuth sample against `PrimaryBond Acceptable` and `PrimaryBond Good`: above Good → **green** (good bond); between Acceptable and Good → **orange** (acceptable bond); below Acceptable → **blue** (fluid behind pipe); for Ultrasonic tools only, below `PrimaryBond Gas` → **red** (gas behind pipe).
2. Per depth, compute the **circumferential coverage %** of cement using the chosen cutoff.
3. `Bond Pass` = coverage % ≥ `PrimaryBond Coverage` (default **85 %**).
4. Which cutoff feeds step 2 is set by `Use as Pass` — **default is the Acceptable flag**, not Good.

**Free-pipe calibration (multi-sensor banding removal), new in 2025:**
- Compute each sensor's **vertical average over the free-pipe calibration interval**; display min / max / overall average across sensors.
- Compute a per-sensor **normalisation factor to the overall average**, applied as either **Linear Shift** or **Multiplier** `[img-read: _cemclip0073.png]`.
- The factor is then applied **to the whole well**.
- **Collar data is excluded from the statistics by default** (`Include collar zones in statistics` unticked `[img-read: _cemclip0073.png]`).
- Interval-length guidance: *"ideally be quite long, perhaps a few hundred feet, so that the tool will have made a number of rotations … each sensor will have 'seen' all sides of the well."* (Shipped example: 10686.50–12026.50, i.e. 1340 ft.)
- For Advanced tools, **Primary and Secondary are calibrated independently** (Impedance/Flexural for Isolation Scanner; Shear/Lamb for INTex).

**Radial-tool normalisation to 3-ft CBL amplitude (the "Normalised" workflow), new in 2025:**
- At each depth level, the radial sensors' average (`Bond_avg`) is compared to the input 3-ft Amplitude curve; **a normalisation factor is derived and applied to every radial sensor curve.**
- Outputs get an `_N` suffix (`AMPS1_N`…`AMPS8_N`, `Bond_Avg_N`, `Bond_Max_N`, `Bond_Min_N`).
- Closure property stated: **"The normalised average curve will be equal to the input 3ft amplitude curve."**
- Explicit assumption: *"After normalisation has been performed, we make the assumption that the CBL chart can be applied to the normalised responses, to relate mV to Compressive Strength for a given pipe size / weight."*
- Only then do the 100 %-bonded and 100 %-free-pipe guide lines appear in the Radial track, and Strength(psi)↔Bond(mV) become linked in the Parameters tab.
- In the **Classic** (non-normalised) radial workflow the manual disclaims the CBL chart: *"these charts were not intended for use with radial tools but they maybe an interesting comparison"*; `CBLpipe` / `CBLcem` are **reference only, used in no calculation**.

**'Combined' Solid–Fluid–Gas crossplot workflow (Advanced tools only), new in 2025:**
1. Crossplot **Primary (x) vs Secondary (y)** final bond values.
2. Four interactive cutoff lines partition the plane: `PrimaryBond Acceptable` and `PrimaryBond Gas` on x; `SecBond Acceptable` and `SecBond Gas` on y → regions **Solid**, **Liquid**, **Gas**. INTex adds a fourth region **Micro-Annulus**, bounded on the secondary axis by `SecBond MA`.
3. Build a Combined S-F-G image (3 colours; 4 for INTex).
4. At each depth, count the fraction of points around the borehole that are Solid / Fluid / Gas → curves **`CovSolid`, `CovFluid`, `CovGas`**, plotted cumulatively.
5. `Bond Pass` = `CovSolid` ≥ **`FinalCoverage %`** (default 85). For INTex, Micro-Annulus points may optionally be counted as Solid.

Axis units observed on the crossplots: Isolation Scanner — Primary `Bond_Final` in **MRayl** (0–20), Secondary `Secondary_Final` in **dB/m** (0–250) `[img-read: _cemclip0075.png]`. INTex — Primary `Bond_Final` in **dB/ft** (0–10), Secondary `Secondary_Final` in **dB/ft** (0–40) `[img-read: _cemclip0077.png]`. **The Isolation Scanner secondary is dB/m and the INTex secondary is dB/ft — different length units on the same parameter name.**

**Hydraulic-isolation criteria (the final traffic light).** All selected criteria must pass:
- **Casing/Cement Bond** — one of: `Bond` | `Derivative` | `Either` | `Both` | `Impedance` | `Flexural` | `Shear` | `Lamb` | `Either` (primary or secondary) | `Combined`. Options shown depend on tool type.
- **Formation Arrival** — formation arrivals visible on the VDL, marked by the user. Rationale stated: arrivals appear near the centre of the VDL and their character tracks lithology; their presence indicates the outer cement sheath is bonded to the borehole wall.
- **No Channel Risk** — interval not in the Bond tab's Channel Risk list (user-picked from the bond image).
- Optional: `Include Micro-Annulus as Solid`, `Formation Tops` (labels rows with the zone name).

**`Omit Multiple Casings` rule:** through double/multiple casing, good cement between inner and outer casing produces **no** formation arrival on the VDL. With the option on, the Formation criterion is marked **not applicable** (grey dash / grey flag) through multi-casing intervals and **does not influence** the traffic light there; it re-activates in single casing.

**Traffic light semantics:** Red = Fail ("Did not meet any of the criteria, very unlikely to provide annular hydraulic isolation"); Amber = Partial Pass ("Met some of the criteria, hydraulic isolation may or may not exist"); Green = Pass ("Passed all criteria … hydraulic isolation likely to exist in annulus").

**Completion-tab validation rules (enforced before zones/parameters are created):** at least one casing entry; **no gaps between casing sizes**; **no overlapping cement intervals**. On Apply: re-order largest→smallest top→bottom, identify innermost casing at each depth, save parameter set, build well diagram, create interpretation-zone depths, compute Expected Log Response per zone. New interpretation zones are created wherever **inner casing size changes or cement slurry strength changes**.

### 2.9 Casing Inspection — tasking (d)

**Metal-loss equation.** The manual states the mechanism but prints no formula:
> *"A metal thickness value is calculated for every radius at every depth. This can be calculated from Internal Radius, or in the case of Ultrasonic tools which provide a direct thickness measurement, it can be taken directly from the thickness. … The thickness is compared to the nominal thickness taken from the Completion tab, and each data point is then assigned a colour-coded grade."* (casinginspection.htm)

`[derived]` I recovered the exact relation from the shipped Joint-by-Joint Results table `[img-read: _ciclip0019.png]`, with `Calculate Loss by: Internal Radius`:

```
Loss % = ( IR_measured − IR_nominal ) / t_nominal × 100
```

Verification against nine joints in that table (Max Loss column vs Max Radius, Nominal Radius, Nominal Thickness):

| Joint | Nom R | Max R | Nom t | (MaxR−NomR)/t ×100 | Table "Maximum Loss" |
|---|---|---|---|---|---|
| 47 | 3.05 | 3.50 | 0.45 | 100.0 | **99.00** |
| 48 | 2.45 | 2.64 | 0.30 | 63.3 | **64.03** |
| 45 | 3.05 | 3.28 | 0.45 | 51.1 | **51.68** |
| 56 | 2.45 | 2.58 | 0.30 | 43.3 | **45.13** |
| 44 | 3.05 | 3.25 | 0.45 | 44.4 | **44.13** |
| 46 | 3.05 | 3.23 | 0.45 | 40.0 | **40.54** |
| 50 | 2.45 | 2.57 | 0.30 | 40.0 | **39.42** |
| 59 | 2.45 | 2.53 | 0.30 | 26.7 | **28.53** |
| 58 | 2.45 | 2.51 | 0.30 | 20.0 | **21.33** |

All nine agree within the rounding of the 2-dp displayed Max Radius (the loss column uses the true per-finger maximum, the radius column is rounded/averaged). Joint 47 landing on 99.0 vs 100.0 pins the denominator as **nominal wall thickness**, not nominal radius or OD. The relation is therefore confirmed to the precision the vendor screenshot permits.

**Grading schema** `[img-read: _ciclip0019.png]`, confirmed against prose:
- `Number of Grades` is user-settable; the grades *"will always be equally spaced between 0 and 100 % loss"*.
- Shipped configuration `Number of Grades = 4` produces **five** displayed grades (prose: *"By default, there are five grades: a grade of zero indicates no loss (or a build-up); and then four levels of loss"*):

| Grade | Condition | Colour |
|---|---|---|
| 0 | Nominal or Buildup | blue |
| 1 | 0 – <25 % Loss | green |
| 2 | 25 – <50 % Loss | yellow |
| 3 | 50 – <75 % Loss | orange |
| 4 | 75 – 100 % Loss | red |

So the general rule is: grade *k* (1..N) spans `[(k−1)·100/N , k·100/N)` % loss, with grade 0 reserved for zero loss or build-up.
- **Joint grade = worst-case of all data in that joint** → output curve `Condition`. Per-point grades → array `ConditionArray`.
- Loss source options: `Internal Radius` | direct `Thickness` (ultrasonic) | **both**, taking either best or worst case. The both-option only becomes available once a Thickness curve is mapped on the Setup tab.

**Nominal-casing table handling (schema, not a dump).** The Completion tab has two grids — **Casing** and **Tubing** — with identical schema `[img-read: _ciclip0005.png]`:

| Column | Units | Entry |
|---|---|---|
| Top Depth | ft (well units) | user |
| Bottom Depth | ft | user |
| Outer Diameter | inch | **dropdown**, selected first |
| Weight | lb/ft | **dropdown**, filtered by the chosen OD |
| Inner Diameter | inch | **auto-populated from an internal casing database** |
| Thickness | inch | **auto-populated** |

`[derived]` The auto-populated pair satisfies `Thickness = (OD − ID) / 2` on every shipped row:

| OD | Weight | ID | Thickness shown | (OD−ID)/2 |
|---|---|---|---|---|
| 20.000 | 106.500 | 19.000 | 0.50 | 0.500 ✔ |
| 13.375 | 72.000 | 12.347 | 0.51 | 0.514 ✔ |
| 13.375 | 68.000 | 12.415 | 0.48 | 0.480 ✔ |
| 9.625 | 53.500 | 8.535 | 0.55 | 0.545 ✔ |
| 7.000 | 32.000 | 6.094 | 0.45 | 0.453 ✔ |
| 5.500 (tbg) | 17.000 | 4.892 | 0.30 | 0.304 ✔ |
| 5.500 (tbg) | 20.000 | 4.778 | 0.36 | 0.361 ✔ |

→ SandiBumi needs an **(OD, weight) → ID** lookup table; thickness is derivable, and the two nominal-radius curves are emitted as `IR_Nominal` and `OR_Nominal` "calculated from the well diagram" at each depth. The manual does **not** print the casing database itself.

Validation rules (fewer than Cement — no cement grid): at least one casing entry; **no gaps between casing sizes**. On Apply: re-order largest→smallest, identify innermost tubular at each depth, save parameter set, build well diagram, create interpretation-zone depths.

**Correction equations (all stated in prose, casinginspection.htm):**
- **De-spiking:** *"Every data point is compared to the average of its four neighbours (left, right, above, below) and if it differs by more than the percentage entered as the de-spike threshold, it is replaced with the neighbour average."* → a 4-connected neighbourhood mean on the (azimuth × depth) array, percentage-relative test. Emits `DeSpikeCount`.
- **Normalisation:** *"The average value of each arm over the entire interval is compared to the overall average of all arms. If it differs by more than the percentage entered as the normalisation threshold, that arm is marked as bad. A new overall average is then calculated, excluding the bad arms. The bad arms are then normalised to make their average equal to the overall average."* → two-pass: flag, recompute clean mean, then shift/scale the flagged arms onto it.
- **Drift correction:** two anchor depths; the module displays the measured average radius at each; the user enters the **Expected Radius** at each; a shift is computed at top and bottom and **linearly interpolated** for all depths between, then applied.
- **Centralisation:** "centre re-projection". Emits `ToolXOffset`, `ToolYOffset`, `ToolEccentricity` ("the distance between the tool axis and the borehole axis"), plus:
  ```
  ToolEllipticity  = Long Axis Diameter / Short Axis Diameter
  ToolOvality      = (Long Axis Diameter − Short Axis Diameter) / Long Axis Diameter
  ```
  (casinginspection.htm, Output Curves table — printed verbatim, these are the only two closed-form expressions on the page.)
- **De-Rotation** is applied **to the corrected radii**, last, if enabled on Setup — "will remove apparent spiralling of the data caused by tool rotation during recording."
- Correction order is fixed by the tab's own list: De-spike → Centralisation → Normalisation → Drift → De-Rotation. Output array: `IR_Processed`.

### 2.10 Chronolog (time↔depth) — tasking (g)

**Time representation (the load-bearing contract):**
- Date-Time reference curves store **double-precision floating point = the number of days since 1 Jan 1900**; the fractional part is the part-day. *"Hence if you wanted to shift a Date Time curve by 2 days you would just add or subtract 2.0 from its value."* (chronolog_help.htm)
- **Unix-time LAS files** encode Date-Time as **seconds since 1 Jan 1970** ("generally over 10^9 seconds"). To load these, the input name for the time reference curve must be set to the literal string **`UnixTime`**. The loader attempts auto-recognition and errors if it cannot determine the format.
- A **Time** well references seconds or milliseconds; a **Date Time** well references Y/M/D/H/M/S.

**Time → depth mapping rules** (`Output Time Curves to Depth Well`):
- The user nominates a **Primary Y-axis Depth curve** in the time well — *"the curve which is used to align the Time data well to the Depth data well and hence must be the drilling depth associated with all the curves that are to be copied."*
- Because a time well normally holds **many samples at one depth**, a `Data averaging method` must be chosen:
  | Method | Rule | Manual's stated use |
  |---|---|---|
  | **Mean** | average of all data points at the depth | — |
  | **Median** | middle value | *"useful for eliminating spikes"* |
  | **Earliest Time** | first value | *"drilling type data like Rate of Penetration or RPM where you are interested in the value when the drill bit drills the formation and not any value when reaming or tripping"* |
  | **Latest Time** | last value | — |
- **Order of operations is stated:** *"When discriminators are used the data at a depth will first have to pass the discriminators and what is left will be averaged using the selected method."* → **discriminate, then average.**
- **Sensor Length** offset: *"In order to put the different LWD sensors on depth you must specify the distance from the sensor measure point to the bottom of the drill string, where the depth measurement is taken."* Stored per curve, editable from the Curve Header module; values come from the drilling contractor's reports.
- **Multiple Depth-Time intervals** may be defined, "allowing LWD data from different dates but the same interval to be copied" — the documented use is monitoring an invasion profile over time as sensors pass a zone while drilling and again on a trip.
- `Fill data gaps` extrapolates depth gaps with a user-set **maximum gap width**.
- Reverse direction (`Copy Curves From Well`, depth→time) requires a reference depth curve in the TOD well; *"the same depth data could be copied to many different Date Times in the TOD well, all having the same depth."*

**Loader constraints:** LIS and DLIS loaders **do not work** for Date-Time wells (stated as of IP3.6). ASCII and LAS (v1.2, v2.0, LBS) do. Column-naming is mandatory and case-specific in effect: the time column must be labelled **`Time`**, the date column **`Date`**, or if combined, **`Date Time`** — otherwise an error dialog. IP assumes **decimal point** as the decimal delimiter; comma-decimal ASCII will load incorrectly.

### 2.11 PL Set-Up — naming/identity contracts (needed to consume everything above)

**8-digit curve name grammar** (plset.htm):
```
[1-4] sensor/quantity mnemonic   PRES TEMP SPIN GR__ PRH_ VAPP QOIL QWAT …  (no spaces)
[5]   condition type             S | P | I
[6]   condition number           1-9   (max 9 conditions)
[7-8] pass number + direction, OR run number
```
Examples given: `PTEMP2D1` (PTEM, production 2, down pass 1); `PRESP1R2` (averaged, run 2); `VAPPS115` (shut-in 1, **run 15** — note the run number consumes both digits); `QOILP3R8`.
- **Flow curves may use 10 digits**, digits 9–10 identifying the originating sensor combination (`CP` capacitance, `DT` DEFT, `FD` fullbore+density, `AD` apparent downflow, `SE` secondary tool set, `_D`/`_C`/`_F` for flow-from-temperature origin).
- Editable copies for inflow differentiation put **`E` in position 4**: `QTOE`, `QOIE`, `QWAE`, `QGAE`.
- Backup curves append `bu` (or `b1`,`b2`…), filter backups append `bf`.
- Stationary data: condition code + `ST` (`TEMPP1ST`); time-based stationary: + station number (`TEMPP1ST2`); time-based continuous carry the station depth (`TEMPP1ST_9125.0`).
- Run date is optionally embedded **in the curve units** (`degF May98`, `psiaFeb99`) as the provenance marker — 4 chars of unit + 2-letter month + 2-digit year.

**Reservoir-zone rules:** must be perforated; must not overlap — **except a 1-sample overlap is permitted** to allow round-number reporting (`Z1: 9220'–9240'`, `Z2: 9240'–9270'`). Auto-creation from a `PERFS` curve names zones `R1, R2, …` top-down. Max 12 characters. **The sump is the documented exception to "must be perforated"** (tick `not perforated`). *"For nearly all calculations, the top of the reservoir zone will be used"*; top **and** bottom are used only for listing zonal contributions.

**Calculation-zone default:** *"If space between the zone and the next higher one is available, the default calculation zone extends directly above the reservoir zone for **15 ft or 5 m**, less if there is less space available."* (multiphaseflowcalculations.htm)

---

## 3. Parameters, defaults & thresholds

### 3.1 Multiphase Flow Calculations

| Parameter | Default / shipped | Units | Source |
|---|---|---|---|
| Relative roughness (Colebrooke friction factor) | **0.0006** | dimensionless | (multiphaseflowcalculations.htm); confirmed `[img-read: _plclip00118.png]` |
| Relative roughness, scaled casing | **0.002** (proposed) | dimensionless | (multiphaseflowcalculations.htm) |
| Reynolds correction factor multiplier | prose default **1.0**; shipped screenshot **1.0300** | dimensionless | prose (…htm); `[img-read: _plclip00119.png]` |
| Limit maximum input viscosity | field value **5.00**; prose says "values between **1 and 10** have so far given reasonable results"; checkbox **off** by default | cp | prose; `[img-read: _plclip00119.png]` |
| Density shift (addition) | **−0.0035** (example) | g/cc | `[img-read: _plclip00119.png]` |
| Make deviation corrections on density | **ticked** in shipped example | — | `[img-read: _plclip00119.png]` |
| Make friction corrections on density | **unticked** in shipped example | — | `[img-read: _plclip00119.png]` |
| Mean cable speed addition | **0.0** | ft/min (positive if against flow) | `[img-read: _plclip00119.png]` |
| PTS friction factor multiplier | **1.00** = no adjustment (smooth housing, mainly since 1994); reasonable value for grooved older tools **1.4 – 1.5** | dimensionless | prose; `[img-read: _plclip00119.png]` |
| Tool diameter (Fanning friction) | 1.68750 (example) | inch | `[img-read: _plclip00118.png]` |
| Spinner diameter (mixture velocity) | 1.25 (in-line example) / 3.50 (fullbore example) | inch | `[img-read: _plclip00118.png]` / `[img-read: _plclip00082.png]` |
| Vmix/Vapp conversion, turbulent oilfield flow | **0.8 – 0.95** | ratio | (multiphaseflowcalculations.htm) |
| Vmix/Vapp conversion, spurious laminar (high viscosity) | 0.5 – 0.6 — *"clearly wrong"* | ratio | (multiphaseflowcalculations.htm) |
| Deviated-well slippage auto-select trigger | deviation **> 5 degrees** over the reservoir section | deg | (multiphaseflowcalculations.htm) |
| Slippage reversal ramp | reduced to zero at 90° over **3** degrees | deg | `[img-read: _plclip00122.png]` |
| Gas-liquid deviation multiplier, bubble | **1 + dev/25** | — | prose; `[img-read: _plclip00122.png]` |
| Gas-liquid deviation multiplier, slug/churn | **1 + (dev/25) × 0.5** | — | `[img-read: _plclip00122.png]` **only** |
| Gas-liquid slippage multiplier | **1.00** | — | `[img-read: _plclip00122.png]` |
| Oil-water slippage multiplier | **1.00** | — | `[img-read: _plclip00122.png]` |
| Slippage multiplier in gas-liquid downflow | **1.00** | — | `[img-read: _plclip00122.png]` |
| Annular Mist slippage correction multiplier | **1.0** | — | `[img-read: _plclip00122.png]` |
| Maximum Vslip | **2.0** × mixture velocity | — | `[img-read: _plclip00122.png]` |
| Max Vslip, true downflow | **1.0** × mixture velocity | — | `[img-read: _plclip00122.png]` |
| Vslip multiplier target (deviated / deviation-corrected) | *"should be always about 1.0"* | — | prose ×2 |
| Ew anchor for the deviated oil-water correlation | **Ew = 0.6** (value held; chart's difference reduced around it) | fraction | prose ×2 |
| Average-deviation window for auto-select | **top 25 %** of interval | — | `[img-read: _plclip00122.png]` |
| Three-phase slippage default | `Calculate slippage between individual phases` **ticked** | — | prose |
| Default calculation-zone length above reservoir zone | **15 ft or 5 m** | ft / m | prose |
| Capacitance meter calibration (chart type) | water point **1000.00**, hydrocarbon point **6000.00**; addition **0.00** | cps | `[img-read: _plclip00120.png]` |
| Hold-up meter calibration (direct type) | water point **1.0000**, hydrocarbon point **0.0000**; addition **0.0000** | unity/decimal | `[img-read: _plclip00121.png]` |
| Sondex CAT oil point (exception to the above) | **0.2** | hold-up | prose (multiphaseflowcalculations.htm) |
| Shipped LUT filenames | `HUMOW.LUT`, `HUMCOW.LUT` (SLB HUM oil/water), `FCAPGW.LUT` (Atlas FCAP gas/water), `FCAPOW.LUT` (Atlas FCAP oil/water), `PL1_0.LUT` | — | prose + `[img-read: _plclip00120/121.png]` |
| LUT directory | `\Program Files\IntPetro\PL` | — | (plset.htm) |
| Flow shading — Standard | Gas **red**, Oil **green**, Water **blue** | — | `[img-read: _plclip00117.png]` |
| Flow shading — Alternate | Gas **green**, Oil **red**, Water **blue** ("at least some units of Shell") | — | `[img-read: _plclip00117.png]` + prose |
| Absent-data sentinels recognised | **−999.0** and **−999.25** | — | multiple pages |
| Sentinel to force a re-read of a zonal box | blank, `-999`, or a leading `/` | — | prose |

### 3.2 Spinner Calibration

| Parameter | Default / typical | Units | Source |
|---|---|---|---|
| Spinner discriminator | prose: *"Values of **−0.1 and +0.1** rps are usual"*; shipped screenshot **−0.50 / +0.50** | rps | prose vs `[img-read: _plclip00094.png]` — see D-1 |
| Spinner thresholds, typical | **+3, −4** (tool dependent) | rps intercept in velocity units | prose |
| Spinner thresholds, shipped example | negative **−5.96**, positive **+1.57** | — | `[img-read: _plclip00094.png]` |
| Zonal slopes, shipped example | negative **0.22033**, positive **0.22050** (identical across all 4 zones after applying the mean) | rps per ft/min | `[img-read: _plclip00094.png]` |
| Threshold pick prompt | "zonal intercept value of **1.52**" | — | `[img-read: _plclip00092.png]` |
| Interactive new-zone default length | **5 %** of the data displayed on screen | — | prose |
| Interactive zone auto-name | `Cn` (n = consecutive zone number) | — | prose |
| Pass weighting, shipped example | D1 **5.0**, D2 **4.0**, D3 **3.0**, D4 **2.0**, U1–U4 **1.0** | — | `[img-read: _plclip00096.png]` |
| Weight normalisation | automatic — weights need not sum to 1.0 | — | prose |
| Quicklook correction factor C | **0.900** (typical value 0.9) | — | prose; `[img-read: _plclip00098.png]` |
| Threshold sensitivity guide | 1 ft/min ≈ **100 rb/d** in 9 5/8" casing, **50 rb/d** in 7" | — | prose ×2 pages |
| Default Vapp curve names | `VAPP` / `VAP2` (or `VAFB` / `VAIL`) fullbore / in-line | — | prose |
| Recommended spinner plot scaling | choose end-points so 0 rps falls on one of the plot's **10 grid lines** (worked example: data −20…50 → scale −21…49) | rps | prose |

### 3.3 Flow from Temperature

| Parameter | Default | Units | Source |
|---|---|---|---|
| Conductivity loss | **0.25** | degF/ft @ 1 bbl/d | prose; `[img-read: _plclip00164.png]` (dialog shows `0.250`) |
| Oil expansion factor | **1** | degF / 1000 vertical ft | prose; `[img-read: _plclip00164.png]` (dialog shows `1.000`) |
| Friction heat (rule of thumb) | **1 degF per 1000 psi drawdown** | degF/psi | prose |
| Friction heat, shipped example | 2.000 | degF | `[img-read: _plclip00164.png]` |
| Vertical geothermal gradient | user/company supplied; example **0.020** | degF/ft | `[img-read: _plclip00163.png]` |
| Depth of first flow | usually the **bottom of the lowermost perforated interval** | ft | prose |
| Friction correction negligibility threshold | < 1 % below **10,000 rb/d in 7" casing** | — | prose |
| Deviated-well slippage trigger | > **5 degrees** | deg | prose |
| Oil-water slippage multiplier | **1.00** | — | `[img-read: _plclip00160.png]` |
| Ew curve names by source | `EW_D…` density, `EW_C…` capacitance/hold-up meter, `EW_F…` DEFT | — | prose |
| Flow curve origin suffix | `_D` density, `_C` capacitance, `_F` DEFT | — | prose |

### 3.4 Multiphase Inflow Curves

| Parameter | Default / guidance | Units | Source |
|---|---|---|---|
| Differentiation length | **7.0** shipped; *"5 or 7 feet seems to give good results"*; *"about 5 ft should work well"* for smooth inputs | ft | prose; `[img-read: _plclip00132.png]` |
| Apply filter | **off** | — | `[img-read: _plclip00132.png]` |
| Filter number of intervals | **7** | samples | `[img-read: _plclip00132.png]` |
| Compute values within half-length of reservoir zones | **off** | — | `[img-read: _plclip00132.png]` |
| Threshold to compute water cut | blank (no threshold) | rb/d | `[img-read: _plclip00132.png]` |
| Output units | rb/d/ft (from rb/d) | — | `[img-read: _plclip00132.png]` |

### 3.5 Multiphase Array Flow (Sondex MAPS) — shipped tool geometry

All dimensions in **inches** (stated on the tab) `[img-read: _plclip00318.png]`:

| Parameter | Shipped value |
|---|---|
| Number of Segments | **7** |
| Casing ID | 3.9 |
| Tool OD | 1.6875 |
| Mini-Spinner OD | 0.4 |
| Mini-Spinner standoff | 0.5 |
| RAT Sensor OD | 0.25 |
| RAT Sensor standoff | 0.5 |
| CAT Sensor OD | 0.25 |
| CAT Sensor standoff | 0.5 |
| Apply Slippage corrections on Run | **ticked**, `Use default calculations for slippage` |
| Make Hole deviation corrections | **ticked** |
| Velocity / Holdup / Slippage multipliers | **1.0** each |
| Make Holdup Cross Section Display | **ON** by default |
| Make Velocity Cross Section Display | **OFF** by default |

Manual's caveat: *"these values could change depending on which generation of MAPS tool is being used"* — the logging contractor supplies them. Recommended MAPS import set: RAT mean ×12, CAT normalised or un-normalised ×12, Spinner ×6, plus all three rotation curves; Standard Deviations and Number-of-Scan curves **not** required.

### 3.6 Cement Evaluation

| Parameter | Default / shipped | Units | Source |
|---|---|---|---|
| Minimum Circumferential coverage (`PrimaryBond Coverage`) | **85 %** | % | prose ("Default value is 85%"); grids show 85 `[img-read: _cemclip0054/0055/0056/0076.png]` |
| `Use as Pass` | **Acceptable** (not Good) | — | prose |
| `PrimaryBond Gas` (Ultrasonic only) | **0.3 MRayl** | MRayl | prose; grid shows 0.3 `[img-read: _cemclip0055.png]` |
| `DerivCover %` | 85 | % | `[img-read: _cemclip0054/0055/0056/0076.png]` |
| `Derivative Solidcut` | 0 in all four shipped grids | tool units/sample | `[img-read: _cemclip0054/0055/0056/0076.png]` |
| Derivative Window Size | **1**; permitted range **1–25**, integer, half-length in samples | samples | prose; `[img-read: _cemclip0071.png]` |
| `FinalCoverage %` (Combined workflow) | **85** | % | `[img-read: _cemclip0076.png]` |
| Free-pipe calibration method | **Linear Shift** (alternative: Multiplier) | — | `[img-read: _cemclip0073.png]` |
| `Include collar zones in statistics` | **off** (collars excluded) | — | prose; `[img-read: _cemclip0073.png]` |
| VDL shading scale | Min **−106**, Max **+106** | amplitude | `[img-read: _cemclip0061.png]` |
| VDL palette | `Blue.pal` | — | `[img-read: _cemclip0061.png]` |
| `Omit Multiple Casings` | **off** | — | prose; `[img-read: _cemclip0061.png]` |
| `Include Micro-Annulus as Solid` | **off** | — | `[img-read: _cemclip0044.png]` |
| Rotation alignment default | **Highside** | — | prose |
| `Is sensor data rotated?` / `Rotate cement maps?` | **No / No** | — | `[img-read: _cemclip0004.png]` |
| `Invert Rotation (360-rot)` | **off** — "In most cases this checkbox will be left off as default" | — | prose |
| Unrotated image scale label | letter **`N`** | — | prose |
| Calibrate to free-pipe | **off** | — | `[img-read: _cemclip0004.png]` |
| Derivative Index (lightweight cements) | **off** | — | `[img-read: _cemclip0004.png]` |

**Shipped zonal parameter grids** (each is one worked example, not a default set — reproduced because they pin the units and the relative ordering of the cutoffs):

*CBL/SBT-family grid* `[img-read: _cemclip0054.png]` — CCL Cutoff 10 · Strength Expected 2500 · Logvalue Expected 11.12177 · Strength Acceptable 500 · PrimaryBond Acceptable 6.633263 · Strength Good 2000 · PrimaryBond Good 10.36511 · Gas n/a · Logvalue FreePipe 1.643141 · Coverage 85 · Solidcut 0 · DerivCover 85 · CBLpipe n/a · CBLcem n/a.
*Ultrasonic grid* `[img-read: _cemclip0055.png]` — CCL 10 · Strengths all **n/a** · Logvalue Expected 3.59894 · PrimaryBond Acceptable 3 · PrimaryBond Good 3.59894 · **Gas 0.3** · FreePipe 0 · Coverage 84.4 · Solidcut 0 · DerivCover 85.
*Radial grid* `[img-read: _cemclip0056.png]` — CCL 0.75 · Strength Expected 1000 · Logvalue Expected n/a · PrimaryBond Acceptable 10.4 · PrimaryBond Good 2.631986 · **Gas n/a** · Coverage 85 · Solidcut 0 · DerivCover 85.
*Isolation Scanner (Advanced) grid* `[img-read: _cemclip0076.png]` — Zone0 6580–12337.5 · CCL 10 · Strength Expected 5 · PrimaryBond Acceptable 8.044982 · PrimaryBond Good 6.21 · PrimaryBond Gas 3.460207 · FreePipe 0 · Coverage 85 · Solidcut 0 · DerivCover 85 · **SecBond Acceptable 115.5989 · SecBond Good 125 · SecBond Gas 59.1922 · SecBond Coverage 80.8 · FinalCoverage 85 · SecBond MA 8**.

**Free-pipe calibration statistics, shipped INTex example** `[img-read: _cemclip0073.png]`: interval 10686.50–12026.50; **Shear** min −0.0900 / max 16.9148 / avg 3.2268; **Lamb** min −0.4692 / max 35.3216 / avg 10.3775.

**Crossplot cutoffs, shipped examples:** Isolation Scanner `SecBond Gas : 57.9` (dB/m) `[img-read: _cemclip0075.png]`; INTex `SecBond Gas : 3.86`, `SecBond MA : 13.3` (dB/ft) `[img-read: _cemclip0077.png]`.

**Cement slurry grid schema** (Ultrasonic, calculate-impedance mode) `[img-read: _cemclip0021.png]`: Expected Top Depth · Expected Bot Depth · Slurry Name · Density (lb/gal) · Travel Time (µs/in) · Expected Strength (MRayls). Direct-input mode replaces the density/travel-time pair with Expected Strength (MRayls) alone `[img-read: _cemclip0019.png]`.

**Tool catalogue and unit contract** (cementeval.htm table + `[img-read: _cemclip0004.png]` dropdown):

| Tool | Company | Tool units | Chart strength unit | Category |
|---|---|---|---|---|
| Ultrasonic Imaging Tool (USIT) | Schlumberger | Acoustic Impedance (MRayl) | Acoustic Impedance (MRayl) | Ultrasonic |
| Circumferential Acoustic Scanning Tool (CAST) | Halliburton | Acoustic Impedance (MRayl) | Acoustic Impedance (MRayl) | Ultrasonic |
| Ultrasonic Radial Scanner (URS) | Weatherford | Acoustic Impedance (MRayl) | Acoustic Impedance (MRayl) | Ultrasonic |
| Ultrasonic Explorer (ULTex) | Baker Hughes | Acoustic Impedance (MRayl) | Acoustic Impedance (MRayl) | Ultrasonic |
| Radial Bond Tool (RBT) | G.E. (Sondex) | Millivolts (mV) | **No chart or conversion** | Radial |
| Slim Cement Mapping Tool (SCMT) | Schlumberger | Millivolts (mV) | **No chart or conversion** | Radial |
| Segmented Bond Tool (SBT) | Baker Hughes | Acoustic Attenuation (dB/ft) | Compressive Strength (psi) | Segmented |
| Cement Bond Logs (CBLs) | Various | Amplitude (mV) | Compressive Strength (psi) | Cement Bond Log |
| Titan RIB | Hunting | Amplitude (mV) | Compressive Strength (psi) | Radial |
| Isolation Scanner | Schlumberger | Acoustic Impedance (MRayl) | Acoustic Impedance (MRayl) | **Advanced** |
| Integrity Explorer (INTex) | Baker Hughes | Acoustic Attenuation (dB/ft) | Acoustic Attenuation (dB/ft) | **Advanced** |

*Footnote as printed:* "the workflow for each CBL tool is identical however the log response chart cross referenced by the module is unique to service companies specific tool."
Note the **Titan RIB category conflict** — see D-4.

### 3.7 Casing Inspection

| Parameter | Default / shipped | Units | Source |
|---|---|---|---|
| De-spike Threshold | **5.00** | % deviation from 4-neighbour mean | `[img-read: _ciclip0018.png]` |
| Normalization Threshold | **5.0000** | % | `[img-read: _ciclip0018.png]` |
| De-spiking / Centralization / Normalization / Drift Correction | **all unticked** by default | — | `[img-read: _ciclip0018.png]` |
| Number of Grades | **4** loss grades (+ grade 0) = 5 displayed | — | prose; `[img-read: _ciclip0019.png]` |
| Grade spacing | always **equally spaced 0–100 % loss** | % | prose |
| Calculate Loss by | `Internal Radius` (shipped); alternatives `Thickness`, or both best/worst-case | — | `[img-read: _ciclip0019.png]` |
| Drift correction anchors (example) | Depth 1 = 4645 (avg radius 2.4377); Depth 2 = 12230 (avg radius 3.2492) | ft / inch | `[img-read: _ciclip0018.png]` |
| Spikes Removed (example) | 7330 | count | `[img-read: _ciclip0018.png]` |
| Only zonal interpretation parameter | **`CCL Cutoff`** per zone | — | prose |
| Parameter Set name | `CasingInspection` | — | `[img-read: _ciclip0018.png]` |
| Supported tool families (dropdown) | Halliburton CAST (Ultrasonic, MRayl); **MultiFinger Imaging Tool (Caliper, units n/a)**; Schlumberger USI (Ultrasonic, MRayl) | — | `[img-read: _ciclip0004.png]` |

**Output curve inventory** (casinginspection.htm): `Collars`, `IR_Processed`, `IRAV_Processed`, `IRMN_Processed`, `IRMX_Processed`, `DeSpikeCount`, `IR_Nominal`, `OR_Nominal`, `IR_RawDiff`, `IR_ProcDiff`, `ConditionArray`, `Condition`, `ToolEllipticity`, `ToolOvality`, `ToolXOffset`, `ToolYOffset`, `ToolEccentricity`.

**Joint-by-Joint Results table schema** `[img-read: _ciclip0019.png]`: ID · Top Depth · Bottom Depth · Length · Nominal Radius · Minimum Radius · Minimum Diameter · Min Dia Depth · Average Radius · Max Radius · Nominal Thickness · *[column header read as "Minimum Thickness" — see OPEN-4]* · Maximum Loss · Max Loss Depth · Grade.

### 3.8 Collar picking (shared by Cement Evaluation and Casing Inspection)

Identical logic on both pages:
- `Collar Curve` — normally CCL, but any curve responding strongly enough (Casing Inspection explicitly allows a Finger curve).
- `Default Collar Cut-Off` with a **Greater Than / Less Than** direction selector.
- `Default Collar Influence` — a vertical interval marked invalid **above and below** each detected collar, **in addition to** the depths that actually exceed the cut-off.
- The Default value applies to all zones on the first run only; per-zone cut-offs are then set interactively and the Default becomes redundant.
- `Disable Interactive Plot Cut-Off` — auto-ticked as soon as the user manually edits any collar pick, so edits survive re-runs. Flags turn **yellow → red** when the interactive cut-off is inactive.
- **Collar data is not deleted.** It is flagged and *"effectively ignored and interpolated over in the final results."*
- Joint listing: `Top of Joint` from **mid-point / highest reading / lowest reading** within each collar interval; numbering **Bottom-up (casing-tally convention) or Top-down**; `Start Number` to match an existing tally. Casing Inspection adds `Include Collar Length` in the Length column.

### 3.9 Data import / station data

| Parameter | Value | Source |
|---|---|---|
| Recognised null values | −999.00 and −999.25 (auto); user may specify another | (depth-based_pl_passes_-_ascii_.htm) |
| Stationary import — omit leading garbage | option **"Omit the first 5% of lines"** | (time-based_pl_station_data_-_a/l.htm) |
| Absent data in station averaging | −999.0 / −999.25 excluded from the average | same |
| Mandatory depth column label | `DEPTH` (need not be first column; not written to the database) | (depth-based …) |
| Mandatory time column label | `Time` | (time-based …) |
| Curve-name length limit on import | **4 characters** (except DEPTH) | (depth-based …) |
| Extension auto-build | tick "use last four characters of file name" — recognises the PL convention (`A3R2P1D1`, `D5S1U2`, `P1D3`) | (depth-based …) |
| ASCII fixed-format grammar | column widths comma-separated, with repeat brackets: `4(8),10,3(12)`; a trailing single width repeats (`10` ≡ `7(10)`) | (depth-based_pl_passes_-_ascii_.htm) |
| Adjacent-delimiter rule | two adjoining delimiters imply a null between them (`1000.0,55.2,,100` → `1000.0 55.2 −999.0 100.0`) | same |
| Max well conditions | **9** | (plset.htm) |
| Autoplot curve limit | data from **11 sensors** (+ computed velocities and rates) | (plset.htm) |
| Sensor Plot tracks | up to **15**, two sensor plots available | (plset.htm / sensorplot.htm) |
| Zone annotation default | top of zone, 0.05 inch from left of depth track, blue, **Arial 8** | (plset.htm) |
| Averaging exclusions | Gamma Ray, spinners and cable velocity excluded from the averaging routine | (plset.htm) |
| Curves not needed for station averaging | cable velocity, gamma ray, CCL | (time-based …) |

**Density-sensor → correction routing table** (plset.htm) — this determines which deviation/friction correction Multiphase Flow Calculations applies:

| Density sensor | Sensor type | Default mnemonic |
|---|---|---|
| Schlumberger GMS | `GMS` | `GRHO` |
| Schlumberger PTS | `PTS` | `PRH_` |
| Schlumberger CPLT | `CPTS` | `WFDE` |
| Schlumberger PSP | `PSP` | `WFDE` |
| Sondex Gradio | `So-Grad` | `DPRE` |
| Sondex Differential Pressure | `So-Diff` | `FDD_` |
| Sondex Mass Balance | `MassBal` | `DEMB` |
| Sondex MAPS | *(blank)* | `FDEN` |
| Baker-Atlas Differential Pressure Fluid Density | `BA-DPFD` | `DPDF` |
| Scientific Drilling Differential Pressure | `SD-Diff` | `GRD_` |
| Nan Gall tuned fork | `TunFork` | `TFD_` |

Correction rules attached to it: pressure-differentiating sensors (gradio or differential-pressure) need **deviation and friction** correction, but for newer sensors deviation is applied by the acquisition software and only friction is offered in IP. The pressure-differentiated `DEND` curve gets **casing friction only, automatically** (tool friction cancels between the two depth points). **Nuclear density gets neither correction.** GHT (Sondex, Co57 source) and GHOST (Schlumberger, mnemonic `GHHM`) data must already be in hold-up units.

---

## 4. Assumptions & validity limits

1. **Turbulent flow is assumed for oilfield PL.** The Vmix/Vapp factor is expected in 0.8–0.95; the appearance of 0.5–0.6 (laminar) is declared "clearly wrong" and is the reason the viscosity clamp exists. High-viscosity fields (Chinese fields named) break the Reynolds path. (multiphaseflowcalculations.htm)
2. **The slippage charts were established for vertical wells.** Everything for deviated/horizontal wells is a documented modification to those charts, not an independent model.
3. **Slippage direction reverses at ~90° deviation** — "It has been observed that the slippage direction changes at about 90 degrees deviation."
4. **Deviated correlation anchors on Ew = 0.6** and compresses the chart's Ew dependence around that point.
5. **Three-phase interpretation is explicitly incomplete.** *"The major problem with triphasic interpretations using density and capacitance sensors is the capacitance/Ew relationship, as the different gas flow regimes affect it but also there is no clear hydrocarbon point, but the point depends on the gas/oil mixture itself."* And: *"Implementing the other slippage correlations and further changes for the three phase interpretation are planned."*
6. **Only one fluid identifier per phase pair enters the computation** — one value for 2-phase, two for 3-phase — even though up to two density and up to four hold-up sensors may be defined. Alternative sensor combinations are computed separately and *"the user has to decide which one gives the best results."*
7. **Zonal input data should be read from stabilised flow intervals above the zone**, not at the zone top, "as the flow there can be associated with the flow from these zones."
8. **Shut-in zonal flow rates are usually not computed or reported.** Observed shut-in flows are often apparent flows from stabilisation movement. If apparent crossflow changes phase, "only the total flow curve (QTOT) should be displayed and commented on the plot as 'apparent flow'."
9. **Zonal values are sticky by design.** Once entered they persist and are redisplayed; the user must explicitly blank/`-999`/`/` a box to force a re-read from updated curves. The manual calls this out as *"Important, as confusing to new users"* — it is deliberate, "so that no edited zonal value will be lost."
10. **Wellbore-expansion contributions do not sum to the total** at reservoir conditions; they are "wellbore barrels". Surface-condition zonal flow is correct under both options.
11. **Flow-from-temperature requires rate-independent friction heat.** Differential depletion between reservoir sections invalidates it (see §2.4).
12. **Flow-from-temperature applies no friction correction to density** (no velocity curve available); acceptable below ~10,000 rb/d in 7".
13. **Multiphase inflow curves are differentiated, not measured** — "No logging tools are currently measuring this inflow." Inputs must be edited free of vortex/turbulence dips first, or the differentiation produces noisy or partially wrong curves. Results "are not absolutely correct."
14. **Array slippage has no industry model above vertical** — the deviation weighting is an in-house assumption (max at vertical, zero at horizontal). Stated plainly by the vendor.
15. **MAPS array release is two-phase downhole only**; a gas rate at surface conditions is a PVT back-conversion, not a downhole measurement.
16. **SIP requires ≥3 flowing rates plus a shut-in**, and the shut-in is admissible only under real crossflow.
17. **Cement Evaluation: cured downhole strength ≠ lab strength.** The manual builds this in: `Strength Expected` is the lab/surface value; `Strength Acceptable` is a separate, client-policy-dependent minimum; the report's `Diff` column exists specifically to expose the gap and the comment box to justify it.
18. **Radial tools have no strength↔log-value chart at all** — "the Expected Strengths and Expected Log Values are not linked … they are completely independent of each other." Any coupling requires the new Normalised workflow, which itself rests on the stated assumption that the CBL chart may be applied to normalised radial responses.
19. **Formation arrivals cannot be expected through multiple casing strings** — the basis of `Omit Multiple Casings`.
20. **Free-pipe calibration presumes a genuinely homogeneous interval** and enough tool rotations that every sensor has seen every side of the well.
21. **Casing Inspection grade is worst-case per joint** — a single bad point downgrades the whole joint.
22. **Chronolog: most interpretation modules make no sense on a time well** — *"most of the Interpretation modules will make no sense to run since most sensors are not on depth with each other."*
23. **Chronolog LIS/DLIS loaders do not work** for Date-Time wells (as of the version the text was written for, IP3.6).
24. **Spinner thresholds must never come from tool spec sheets** — in-situ picking only, "as many variables are involved as well as the imposition of the zonal regression slopes."
25. **Spinner calibration zones must not overlap** within a well condition.
26. **Cement/Casing collar data is interpolated over, never deleted.**

---

## 5. Internal discrepancies

**D-1 — Spinner discriminator: prose ±0.1 rps vs shipped dialog ±0.50 rps.**
Prose (spinnercalibrationapparentvelocity.htm): *"Values of −0.1 and +0.1 rps are usual."* The shipped calibration dialog reads `Discriminators: -0.50   0.50` `[img-read: _plclip00094.png]`. A 5× difference in the value that decides which points enter the regression. The prose also notes discriminators are configurable under Preferences and *"The new values will be effective for the current PC"* — so the screenshot may be a customised machine. **Do not adopt either as canonical without a live IP check.**

**D-2 — Reynolds correction factor multiplier: prose default 1.0 vs shipped 1.0300.**
Prose: *"Default is 1.0."* Dialog: `1.0300` `[img-read: _plclip00119.png]`. The dialog is a worked example (it also carries a −0.0035 g/cc density shift), so 1.0 is almost certainly the true default — but the pairing is worth stating because the auto-surface-match adjusts exactly this parameter.

**D-3 — Impedance-from-density worked example is internally inconsistent at 0.14 %.**
`[img-read: _cemclip0021.png]` Row 1 (10.00 lb/gal, 9.00 µs/in → 3.38 MRayl) reproduces exactly under `Z = ρ·v`. Row 2 (13.00 lb/gal, 7.00 µs/in) computes to **5.6523** but displays **5.66**. Candidate explanations: the density cell displays a rounded 13.00 for ~13.02; a different lb/gal→g/cc constant; or a display rounding artefact. Either way, a SandiBumi implementation reproducing row 1 exactly will differ from the vendor screenshot on row 2 by 0.14 %.

**D-4 — Titan RIB tool category / unit inconsistency.**
The prose table lists **Titan RIB — Hunting — Amplitude (mV) — Compressive Strength (psi) — Radial**. But the same page states for the Radial category: *"Due to the fact there are no cement strength vs log value charts for Radial type tools, the Expected Strengths and Expected Log Values are not linked."* Meanwhile the tool-selection dropdown shows **`Hunting Titan RIB — Tool Type: Radial — Units: Cement Strength (PSI)`** `[img-read: _cemclip0004.png]`, and the prose table's own Radial entries (RBT, SCMT) say **"No chart or conversion"** while Titan RIB says "Compressive Strength (psi)". Titan RIB is therefore simultaneously (a) Radial, (b) chart-less by category rule, and (c) tagged with a psi chart unit. **Unresolved in the manual.**

**D-5 — Cutoff ordering for mV-amplitude (Radial) tools is inconsistent between two shipped grids, and conflicts with the stated colouring rule.**
The Bond Image rule is directional and unqualified: *"Any values **above** the 'Good' cut-off value will be coloured green … any value **below** the 'Acceptable' cut-off will be coloured blue."* That is correct for attenuation (dB/ft) and impedance (MRayl), where higher = better cement. For CBL **amplitude in mV**, lower = better cement. The two shipped radial grids disagree with each other:
- `[img-read: _cemclip0056.png]` (Radial, classic): PrimaryBond **Acceptable 10.4 mV**, **Good 2.631986 mV** → Good < Acceptable (physically correct for mV).
- `[img-read: _cemclip0070.png]` (Radial, after normalisation): Strength Acceptable 500 psi → **Bond Acceptable 2.15557 mV**; Strength Good 900 psi → **Bond Good 10 mV** → Good > Acceptable, i.e. *higher* strength maps to *higher* amplitude, which inverts CBL physics.
The manual never states how the colouring rule flips for decreasing-with-quality measurements. **This is the single most implementation-dangerous ambiguity on the Cement page** — see SandiBumi note S-4.

**D-6 — Micro-Annulus is described as carved out of Solids, but drawn adjacent to Liquid.**
Prose (twice): *"This crossplot further subdivides the **Solid** zone into an additional Micro-Annulus zone based on the Secondary (Lamb) measurement"* and *"Data falling in this zone is regarded as Solid, but with the presence of a micro-annulus."* But also: *"The interactive boundary between **Liquid** and Micro-Annulus on the Secondary axis represents the SecBond MA parameter."* The INTex crossplot `[img-read: _cemclip0077.png]` places **Micro-Annulus upper-left** — i.e. **low** primary (Shear), **high** secondary (Lamb) — on the *opposite* side of the primary Acceptable line from Solid. Geometrically it is carved from Liquid, not Solid; semantically it is counted as Solid. The two prose statements cannot both be geometric descriptions.

**D-7 — Advanced-tool secondary axis uses different length units per tool, under one parameter name.**
`SecBond Acceptable/Good/Gas/MA` are **dB/m** for Isolation Scanner `[img-read: _cemclip0075.png]` and **dB/ft** for INTex `[img-read: _cemclip0077.png]`. The parameter grid `[img-read: _cemclip0076.png]` carries no unit column. A value of 115.6 means something ~3.28× different depending on tool. The manual never flags this.

**D-8 — Spinner diameter differs 3× between two shipped screens of the same workflow.**
`[img-read: _plclip00118.png]` (Multiphase Flow physical inputs) shows 1.25 in; `[img-read: _plclip00082.png]` and `[img-read: _plclip00098.png]` show 3.50 in. Almost certainly in-line vs fullbore examples (the module supports both, run twice), not a contradiction — recorded so it is not mistaken for one later. The manual does warn: *"The correct spinner diameter has to be edited."*

**D-9 — Slug/churn deviation multiplier exists only in the raster.**
Prose gives only `1 + dev/25`. The dialog gives **two** branches, the second with an editable 0.5 factor `[img-read: _plclip00122.png]`. An implementation built from the prose alone would silently over-correct slug/churn slippage by a factor of ~2 in the deviation term.

**D-10 — Two near-identical "PLT — Available Reports" pages ship in the same CHM.**
`pltavailablereports.htm` (1,981 chars) and `plt-availablereports.htm` (1,899 chars) are the same topic; the shorter one adds the Acrobat-PDFWriter purchase note, the longer adds the suggested report ordering. IP2018 ships only `pltavailablereports.htm`. Cosmetic, but relevant to any page-count reconciliation across agents.

**D-11 — Conductivity-loss unit string is malformed in the dialog.**
`[img-read: _plclip00164.png]` displays `deg/deg /ft @ 1 bl/d`; the prose says `0.25degF/ft @ 1bl/d`. The prose form is the sensible one. Recorded so the dialog string is not treated as a second, different quantity.

---

## 6. IP2018 numeric diff

Method: probed `c18/*.htm` for every constant and named feature captured above.

### 6.1 Constants — no drift whatsoever in the PL engine

Every one of the following is present and **numerically identical** in IP2018:

| Constant | IP2018 | IP2025 |
|---|---|---|
| Relative roughness default | 0.0006 | 0.0006 |
| Relative roughness, scaled casing | 0.002 | 0.002 |
| PTS friction factor multiplier range | 1.4 – 1.5 | 1.4 – 1.5 |
| Default calculation-zone length | 15 ft or 5 m | 15 ft or 5 m |
| Deviated-slippage trigger | 5 degrees | 5 degrees |
| Slippage reversal deviation | 90 degrees | 90 degrees |
| Gas-liquid deviation multiplier | dev/25 | dev/25 |
| Vmix/Vapp turbulent range | .8 – .95 | .8 – .95 |
| Viscosity-clamp guidance | "values between 1 and 10" | same |
| Force Bubble Flow option | present | present |
| Spinner discriminator | −0.1 and +0.1 | −0.1 and +0.1 |
| Threshold rate guide | 100 rb/d @ 9 5/8", 50 rb/d @ 7" | same |
| Typical thresholds | +3, −4 | +3, −4 |
| Quicklook C | typical value: 0.9 | 0.9 |
| Conductivity loss | 0.25 degF/ft | 0.25 degF/ft |
| Oil expansion factor | 1 degF/1000 ft | 1 degF/1000 ft |
| Friction heat guidance | 1 degF per 1000 psi | same |
| Friction-correction negligibility | 10,000 rb/d in 7" | same |
| Differentiation length | 5 or 7 feet; ~5 ft | same |
| Cement `PrimaryBond Gas` | 0.3 MRayls | 0.3 MRayls |
| Cement coverage default | 85% | 85% |
| Cement look-up charts | Chart 9-1, Chart 9-4, CEM 1, CBL 1 | identical four |
| Apparent-downflow / Vslip-dependency / legacy-method / max-slippage-velocity text | present (same occurrence counts: 12 / 3 / 1 / 1) | identical |
| Condensate model | present (3 occurrences) | identical |
| Chronolog: 1st Jan 1900 epoch, UnixTime, Median/Earliest/Latest, Sensor Length, Fill data gaps | all present | all present |
| PL Set-Up: MAPS, Nan Gall / TunFork, Lookup Tables, Fluid Shading, Secondary Tool Set, Scientific Drilling | all present | all present |

**Conclusion: the production-logging computational engine did not change numerically between IP2018 and IP2025.** Anything SandiBumi validates against IP2018 output remains valid.

### 6.2 Module-level deltas (IP2025 additions)

| Change | Evidence |
|---|---|
| **Casing Inspection is entirely new in IP2025.** | `c18/casinginspection.htm` **does not exist**. The whole module — MFC/ultrasonic support, de-spike/centralisation/normalisation/drift corrections, condition grading, joint-by-joint results, ellipticity/ovality/eccentricity outputs — has no IP2018 counterpart. |
| **Cement: Advanced-tool support added.** IP2018's tool table has only 6 tools (USIT, CAST, URS, RBT, SCMT, SBT). Absent from 2018: **Ultrasonic Explorer (ULTex), Titan RIB, Isolation Scanner, Integrity Explorer (INTex)**, and generic "Cement Bond Logs" is present but the Advanced category is not. | grep of `c18/cementeval.htm` returns zero hits for `Isolation Scanner`, `Integrity Explorer`, `INTex`, `Ultrasonic Explorer`, `Titan RIB` |
| **Cement: `Combined` Solid-Fluid-Gas crossplot workflow is new.** Zero hits in 2018 for `Combined`, `Micro-Annulus`, `Lamb`, `Flexural`, `CovSolid`. | same grep |
| **Cement: `Calibrate to Free Pipe` (per-sensor banding normalisation) is new.** Zero hits in 2018. | same grep |
| **Cement: Radial `Normalise to 3ft Amplitude` workflow is new.** Zero hits in 2018 for `Normalis*`. | same grep |
| **Cement: Derivative Window Size control is new.** 2018 has the Derivative concept (53 hits on "Derivative") but no `Window Size` and no "between 1 and 25". | grep of `c18/cementeval.htm` |
| **Cement: `Import Well Diagram` is new.** Zero hits in 2018. | same grep |
| **`plt-availablereports.htm` is a new duplicate topic** (2018 has only `pltavailablereports.htm`). | file listing |
| **`bubble-analysis.htm` and `terminal-events.htm` are new but empty** — no 2018 counterpart, and no 2025 content. | file listing + read |
| Unchanged in 2018: `Omit Multiple Casings` (7 hits), `Joint Listing` (3 hits), the four look-up charts. | grep |

---

## 7. SandiBumi notes

**S-1 — The cement strength↔log-response conversion is not implementable from this manual.**
IP delegates it entirely to four named vendor charts (Baker Atlas 9-1 / 9-4, SLB CEM 1, HAL CBL 1). SandiBumi has two honest routes: (a) digitise those charts — Jauhar's `tools/chartdig` dash-tip vector extraction method already exists for exactly this class of problem (see `reference_chartbook_digitization`), and Chart CEM 1 lives in the 2010 SLB chartbook; or (b) ship Ultrasonic/Advanced workflows first, where **no chart is needed** because both the parameter and the log are in MRayl. Route (b) exceeds IP on the tools that matter most for modern jobs and defers the chart-digitisation debt. **Do not synthesise a strength↔mV correlation from textbook sources — the whole point of IP's design is that these curves are per-service-company and per-tool-series (note the "Tool series 1415, 1417" qualifier on Chart 9-4).**

**S-2 — The impedance-from-density path is fully implementable today.** `Z[MRayl] = ρ[g/cc] × (25400/Δt[µs/in]) / 1000`, verified against the vendor's own worked example to 3 s.f. This gives SandiBumi a chart-free entry into ultrasonic cement evaluation with a documented provenance. Carry D-3 as a known 0.14 % divergence from the vendor's second example.

**S-3 — Casing Inspection is the cheapest high-value module to match-and-exceed.** It is new in IP2025 (so competitors' users have low incumbency), and every piece of its math is either printed (ellipticity, ovality) or recoverable (loss %, thickness). The only external dependency is the **(OD, weight) → ID** casing table, which is a public API 5CT-style lookup, not a proprietary chart. Note the module's **only** interpretation parameter is the per-zone CCL cutoff — everything else is QC. That is a small surface to beat.

**S-4 — Fix D-5 before writing any cement bond code.** IP's colour rule ("above Good = green") is written for increasing-with-quality measurements and IP's own radial screenshots contradict each other. SandiBumi should carry an explicit **per-tool polarity flag** (`quality_increases_with_value`: true for dB/ft attenuation and MRayl impedance, false for mV amplitude) and drive all cutoff comparisons and colouring from it, rather than inheriting IP's implicit assumption. This is exactly the class of silent-wrongness failure the project cares about: a cement job would be graded green where it is free pipe.

**S-5 — Carry units on the Advanced secondary parameters (D-7).** `SecBond*` in dB/m for Isolation Scanner and dB/ft for INTex, with no unit column in IP's own grid, is a ready-made unit trap. SandiBumi's parameter store should make the unit non-optional.

**S-6 — Implement the slug/churn deviation branch from the raster, not the prose (D-9).** `Vslip × (1 + (dev/25) × 0.5)` with 0.5 exposed as a parameter.

**S-7 — The spinner calibration math is fully specified and worth implementing verbatim.** `Vapp(i) = rps/slope − CableSpeed + Threshold`, weighted-mean combination with auto-normalised weights, per-zone positive/negative slopes from zone-*averaged* pairs, mid-zone slope with extrapolate-outside / interpolate-between. The two non-obvious rules to preserve: (i) single-point zone inherits the opposite-sign slope; (ii) changing a zone's depth re-enables its disabled points. Both are cheap to get wrong and invisible when wrong.

**S-8 — Adopt IP's curve-naming grammar as an interop surface, not as an internal schema.** The 8/10-digit convention with date-in-units is genuinely useful provenance (it is how IP users identify a curve's origin across runs), but it caps mnemonics at 4 characters and conditions at 9. SandiBumi should store structured provenance and *render* IP-style names on export.

**S-9 — Two IP behaviours are worth deliberately not copying:** the sticky-zonal-value re-read protocol (blank / `-999` / leading `/`), which the manual itself flags as confusing; and the requirement to run Spinner Calibration twice for simultaneous in-line + fullbore spinners. Both are 1990s-workflow artefacts.

**S-10 — Chronolog's contracts are simple and should be matched exactly:** days-since-1900-01-01 as a double, `UnixTime` as the magic input name for seconds-since-1970, discriminate-then-average ordering, and per-curve Sensor Length as the LWD depth offset. The averaging-method choice (Median for spike rejection, Earliest Time for ROP/RPM) is good domain judgement worth reproducing.

**S-11 — No smectite or montmorillonite endpoints appear anywhere in these 29 pages.** Searched; zero occurrences. These are production-logging and cased-hole pages with no clay-mineral content. Other agents own that.

---

## 8. OPEN ITEMS

**OPEN-1 — The gas flowmap is not reproduced in the CHM.**
multiphaseflowcalculations.htm states *"the flowmap determines the flow regime"* and defines bubble/slug/churn/mist qualitatively, but no flowmap image, axes, or boundary equations appear on the page. `_plclip00126.png`, which sits nearest the flowmap discussion, is the *Edit Input Values* dialog, not a flowmap. **The flow-regime boundaries are therefore unobtainable from this manual.** They are referred to "the PL User Manual", a separate document not in this CHM. Flagging for whichever agent holds the PL User Manual, if any is in scope.

**OPEN-2 — The oil/water and gas/liquid slippage charts themselves are not reproduced.**
"Standard 1 represents this chart" — the chart is in the PL User Manual's Slippage Section, not here. So while every *option name*, *modifier*, and *multiplier* is captured, the underlying Vslip(Ew) and Vslip(P) relations are not. SandiBumi cannot reproduce IP's slippage numerically from this source.

**OPEN-3 — Array slippage deviation weighting: `cos θ` is my inference, not a vendor statement.**
`[img-read: _plclip00323.png]` The plotted curve passes through (0°,1.0), (30°,≈0.87), (60°,≈0.5), (90°,0.0), (120°,≈−0.5), (150°,≈−0.87), (180°,−1.0). That is `cos θ` to reading accuracy, and the vendor's stated boundary conditions (1 at vertical, 0 at horizontal) are satisfied. **But the manual prints no formula and the chart is a low-resolution Excel export**, so a half-cosine of a different power, or `cos` of a rescaled angle, cannot be excluded from the read points alone. Do not hard-code `cos θ` as vendor-sourced.

**OPEN-4 — Casing Inspection results-table column 12 header is uncertain.**
`[img-read: _ciclip0019.png]` I read the two-line header as "Minimum / Thickness". The **values contradict that reading**: with Nominal Thickness 0.30 the column carries 1.02, 1.61, 0.13, 2.14, 2.95; with Nominal Thickness 0.45 it carries 0.47, 1.63, 5.44, 0.34. `[derived]` Treating the column instead as **Minimum Loss %** reproduces sensible values in every row (e.g. joint 59: 1.02 % loss ⇒ measured IR 2.453 vs nominal 2.45 — a finger reading essentially nominal, which is exactly what a joint's *minimum* loss should look like). The screenshot resolution does not let me settle "Thickness" vs "Loss" in the header text. **Resolve against a live IP run before relying on that column.** The `Maximum Loss` column and its formula are *not* affected — those are verified in §2.9.

**OPEN-5 — Gas FVF units in the zonal PVT table are unreadable.**
`[img-read: _plclip00126.png]` The Gas FVF row shows a value of 0.008508 with a unit string that renders as `( f/mn )` — the same string as the Apparent velocity row (ft/min), which cannot be right for a formation volume factor. Oil FVF is clearly `rb/stb` and Water FVF `rbw/stbw`. **The gas FVF unit is not legible at this raster resolution; do not assume rb/scf.**

**OPEN-6 — The Reynolds-dependent Vmix/Vapp function itself is not printed.**
Only its signature `f(Re, D_casing, D_spinner)` and its expected output range (0.8–0.95 turbulent). This is the single most load-bearing undocumented relation in the whole PL module — every flow rate scales through it. Not recoverable from this CHM.

**OPEN-7 — The internal casing database (OD, weight → ID) is not printed** in either Cement Evaluation or Casing Inspection. Only the derived relation `thickness = (OD − ID)/2` is recoverable. SandiBumi must source the table independently.

**OPEN-8 — The `Derivative` computation is described but not defined.**
"rate of change between depth samples" over a ±window-half-length — but whether it is a simple two-point difference across the window, a least-squares gradient, a mean absolute difference, or an RMS of successive differences is not stated. Given the method's whole premise is that heterogeneous cement has a *higher* derivative than homogeneous fluid, an absolute-value or RMS form is implied (a signed gradient would average toward zero) — **but the manual does not say so.** Do not guess.

**OPEN-9 — `Bond Index` is named twice as an option but never defined.**
The classic industry bond index (attenuation ratio) may or may not be what IP means by "the Bond Index option"; on this page "Bond" appears to simply denote the amplitude/impedance cutoff workflow on the Bond tab. **The manual never gives a bond-index equation.** If SandiBumi ships a "bond index", it must not claim IP compatibility for it.

**OPEN-10 — Free-pipe calibration: `Linear Shift` vs `Multiplier` arithmetic is not written out.**
"a normalisation factor for each sensor which will normalise each to the overall average" — for Multiplier this is presumably `overall_avg / sensor_avg`, for Linear Shift `overall_avg − sensor_avg`. Both are the obvious readings, neither is stated. Low risk, but unstated.

**OPEN-11 — Radial normalisation factor form is not written out.**
"the Average value of the radial sensors (Bond_avg) is compared to the input 3ft Amplitude curve. A normalisation factor is derived, and then this factor is applied to each of the radial sensor curves." The stated closure property (`Bond_Avg_N` ≡ the 3-ft amplitude curve) is satisfied by a multiplicative factor `Amp3ft / Bond_avg` **and** by an additive shift `Amp3ft − Bond_avg`. The manual does not say which. Given `_N` curves are produced per-sensor and the closure holds for both, this is genuinely ambiguous.

**OPEN-12 — `bubble-analysis.htm` and `terminal-events.htm` carry no content in IP2025.**
Both are 124-char stubs whose body is the authoring placeholder "Enter topic text here." Neither exists in IP2018. Whatever these features are, they are undocumented. If another agent encounters "Bubble Analysis" or "Terminal Events" referenced from a hub page, this is why the link is dead.

**OPEN-13 — Whether the Cement `Strength Acceptable`→`PrimaryBond Acceptable` conversion re-enters the chart or interpolates.**
The prose says the log-value parameters are "derived from the Strength Acceptable parameter and using the relevant log response chart" — but whether the chart is re-queried live when the user edits a strength, or a local linearisation is used, is not stated. The Normalised-radial section implies live coupling ("The user can now manually enter strength values here, and the Bond mV values will update"), but does not generalise it.

---

*End of report — agent M.*
