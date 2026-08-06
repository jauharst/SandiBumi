# K — Geophysics, Acoustics & Borehole-Image Analysis

Ingest of the Interactive Petrophysics 2025 vendor manual (decompiled CHM), agent K.
Consumer: SandiBumi. Provenance discipline: every fact below carries its source.

- `(pagename.htm)` — fact read from page prose (clean-text extract of that page).
- `[img-read: file.png]` — equation/symbol transcribed by reading the raster directly (vision).
- Nothing here is filled in from textbook knowledge. Where a symbol or value could not be
  resolved from the page it is in **§9 OPEN ITEMS**, not in the body.

Vendor prose is copyrighted and is paraphrased throughout; equations, constants, curve names
and numeric limits are facts and are captured in full.

---

## 1. Scope & page inventory

All 20 assigned pages read to end-of-file. Sources are the clean-text extracts in `c25`,
with the raw `.htm` consulted where a byte-level question arose.

| # | Page | Chars | Content imgs | Status | Yield |
|---|---|---:|---:|---|---|
| 1 | `plotting_image_analysis_data.htm` | 90,250 | 201 | read 1–2417 (complete) | Mean-dip vector math (11 rasters), Terzaghi weighting, fracture aperture, Net2Gross, profile/tadpole/stereonet conventions |
| 2 | `acoustic_waveform_processing.htm` | 82,888 | 68 | read 1–1386 (complete, 2 passes) | Semblance equation, Nth-root, dispersion, Alford anisotropy, full curve dictionary |
| 3 | `imageanalysisandpicks.htm` | 51,693 | 123 | read complete | TST/TVT geometry, sinusoid picking, dip uncertainty, auto-dip |
| 4 | `synthetic_seismorgram.htm` | 48,909 | 53 | read complete | Ricker, convolution, Aki–Richards R(θ), Backus (8 rasters), time-curve rules, full validity envelope |
| 5 | `imageanalysisoverview.htm` | 39,430 | 75 | read complete | Normalization/histogram methods, 17 enhancements, orientation options, pick-export curve names |
| 6 | `manageimagetools.htm` | 26,262 | 29 | read complete | Tool-state requirements, nav-curve precedence, magnetic declination, dipmeter table |
| 7 | `image-corrections-and-processi.htm` | 23,026 | 17 | read complete | Speed-correction parameters, deconvolution, calibrate/transform equations |
| 8 | `geosteering.htm` | 17,771 | 64 | read complete | Segment-dip definition, grid defaults, RSI |
| 9 | `velocity_for_array_tools.htm` | 16,044 | 9 | read complete | Fluid-velocity equation, holdup cutoffs |
| 10 | `image_analysis.htm` | 13,722 | 2 | read complete | Licensing/module routing |
| 11 | `3d_viewer.htm` | 11,158 | 22 | read complete | 3D display conventions |
| 12 | `navigation_qc.htm` | 8,438 | 3 | read complete | Accelerometer/magnetometer re-computation, output curve set |
| 13 | `cat_rat_normalise.htm` | 6,602 | 2 | read complete | 1-point/2-point normalization, curve naming |
| 14 | `image_toolkit.htm` | 5,946 | 9 | read complete | 8 utility modules incl. Inclinometry Calculator |
| 15 | `cat_rat_image.htm` | 5,605 | 2 | read complete | Holdup endpoints, array geometry convention, default colours |
| 16 | `velocity_images.htm` | 4,813 | 2 | read complete | Array velocity images, top/middle/bottom weighting |
| 17 | `elastic_impedance.htm` | 3,413 | 13 | read complete | EI equation (5 rasters), normalization to AI |
| 18 | `rockphysics.htm` | 2,735 | 0 | read complete | Geophysics hub; module roster + attributions |
| 19 | `3d-viewer.htm` | 112 | 0 | read complete | **Empty placeholder stub** — content is the authoring-tool default text only |
| 20 | `analysis-sticks.htm` | 124 | 0 | read complete | **Empty placeholder stub** — same |

Two pages (#19, #20) carry no technical content whatsoever; they are unwritten topics left in
the shipped help. Recorded here so the inventory is closed, not skipped.

**Attribution roster** (`rockphysics.htm`): the section is titled *Geophysics* in IP2025 and
contains Shear Sonic QC/Create, Density Estimation (Gardner / Bellotti et al / Lindseth),
Fluid Substitution, Laminated Fluid Substitution (Gassmann-based), Elastic Impedance
(Connolly, TLE 1999), Synthetic Seismogram (Aki & Richards 1980 linearised Zoeppritz), and
Acoustic Waveform Processing.

---

## 2. Equations & methods per module

### 2.1 Elastic Impedance (`elastic_impedance.htm`, `rockphysics.htm`)

Attribution: P. Connolly, *The Leading Edge* (1999); the **high-angle inversion form,
equation 4.1** of that paper (`rockphysics.htm`, `elastic_impedance.htm`).

```
[img-read: embim611.png]
EI = Vp^(1 + Sin²θ) · Vs^(−8 × K × Sin²θ) · ρ_b^(1 − 4 × K × Sin²θ)
```

Symbol definitions, each read from its own raster:

| Symbol | Definition | Source |
|---|---|---|
| Vp | compressional velocity | `elastic_impedance.htm` |
| Vs | shear velocity | `elastic_impedance.htm` |
| ρ_b | bulk density | `[img-read: embim613.png]` |
| θ | incidence angle | `[img-read: embim614.png]` |
| K | `K = Vs² / Vp²` | `[img-read: embim615.png]`, and `[img-read: embim612.png]` shows the same ratio |

Prose additionally describes K as a constant representing the average over the interval
(`elastic_impedance.htm`) — i.e. K is entered as a **single scalar** for the run, even though it
is *defined* as a pointwise velocity ratio. See §6.1.

**Normalization to AI.** Optional. The ratio AI/EI is formed at user-specified normalization
depths and applied to the EI curve away from those depths; with more than one depth, the
normalization value between them is a linear extrapolation of the values at the entered depths
(`elastic_impedance.htm`). At the normalization depths the normalized EI equals AI exactly.

**Unit sensitivity — load-bearing.** The page states explicitly that because of the form of the
equation the result curve *shape* differs with the calculation units chosen for velocity and
density, and that unit conversion **cannot** be applied afterwards to the output curve — it must
be applied to the inputs (`elastic_impedance.htm`). This is a hard correctness constraint, not a
cosmetic one: EI is not dimensionally homogeneous across unit systems.

### 2.2 Synthetic Seismogram (`synthetic_seismorgram.htm`)

Three stages: time-curve creation → reflectivity + convolution → rock-physics plot.
1D modelling only; no ray tracing is involved other than Backus averaging.

**Ricker wavelet**

```
[img-read: embim616.png]
W(t) = (1 − 2π² f_dom² t²) · e^(−π² f_dom² t²)
```

**Convolution (trace synthesis)**

```
[img-read: embim617.png]
s_t = Σ_{k=0}^{N−1} R_k · W_{t−k}
```

**Reflectivity — Aki & Richards (1980) first-order linearisation of Zoeppritz (1912)**

```
[img-read: embim618.png]
R(θ) = [ 1/2 − 2 (V̄_S² / V̄_P²) sin²θ ] · (Δρ / ρ̄)
     + [ 1/2 sec²θ ]                    · (ΔV_P / V̄_P)
     − [ 4 (V̄_S² / V̄_P²) sin²θ ]       · (ΔV_S / V̄_S)
```

(Overbars denote the interface averages; Δ the contrasts across the interface.)

**Backus averaging (Backus, 1962)** — optional upscaling to a seismic frequency. The page
describes it as replacing a heterogeneous medium with an equivalent homogeneous transversely
isotropic one, with filter aperture set by wavelength/frequency.

Step 1 — compute the coefficients:

```
[img-read: embim619.png]   C = 1 / ⟨ 1 / (ρ V_P²) ⟩
[img-read: embim620.png]   D = 1 / ⟨ 1 / (ρ V_S²) ⟩
[img-read: embim621.png]   ⟨ · ⟩  — the bracket operator
```
Brackets denote averages of the properties **weighted by volumetric proportion**
(`synthetic_seismorgram.htm`).

Step 2 — reconstruct the upscaled properties:

```
[img-read: embim622.png]   V_{S,V}      = √( D / ρ )
[img-read: embim623.png]   V_P          = √( C / ρ )
[img-read: embim624.png]   ρ_Backus     = ⟨ ρ_b ⟩
[img-read: embim625.png]   I_{P-backus} = ρ_Backus · V_{P,V}
[img-read: embim626.png]   I_{S-backus} = ρ_Backus · V_{SH,V}
```

Transcribed exactly as printed. Two symbols in the impedance lines (`V_{P,V}`, `V_{SH,V}`) are
**never defined on the page** — see §6.2. The `ρ` inside the two square roots is likewise
ambiguous — see §6.3.

**Time-curve construction** (`synthetic_seismorgram.htm`):
- Generated by integrating the sonic over **TVD** (trapezoid rule), then converted back to MD.
  Gaps in the integration are linearly interpolated.
- A zero-time anchor point is inserted: for TVDSS indexing, (Depth = 0, Time = 0); for TVD or MD
  indexing, (Depth = Log Elevation, Time = 0).
- **Replacement-velocity calibration**: the calibration point is `One-way time = input depth /
  input velocity`; the curve is extrapolated to the top at the defined reference velocity. Default
  reference depth = depth of the first data point in the selected sonic log.
- **Checkshot calibration**: builds a correction curve from the depth-time pairs; the correction is
  linearly interpolated between calibration points, **held constant (repeated) past the last
  calibration point**, and linearly extrapolated past the end of the sonic. The sonic is
  stretched/squeezed onto the checkshot reference depths.
- Vertical-well quick path: `TVD = MD`, `TVDSS = MD − (height of reference above sea level)`,
  the latter defaulting from the well header (KB elevation above the seismic reference datum).

### 2.3 Acoustic Waveform Processing (`acoustic_waveform_processing.htm`)

Purpose: process raw sonic waveform arrays for compressional, shear and Stoneley slowness, plus
classic crossed-dipole anisotropy. Engine is written in Matlab and needs the Matlab Runtime;
IP 4.4 update 2 and later use **Matlab 2015a (V8.5)**.

**Semblance (STC).** Attribution printed on the page: *"Semblance processing of borehole acoustic
array data"* by **Kimball and Marzeta**, *Geophysics* **Vol.49 Mo.3, March 1984**. Both the author
spelling and "Mo.3" are as printed by the vendor — see §6.9.

Setup as stated: an array of M receivers at distances z₁…z_M from the transmitter; a correlation
window of length Tw; comparison over a range of slownesses s and times τ.

```
[img-read: _awpclip0025.png]

              (1/M) ∫_{t=0}^{Tw} [ Σ_{m=1}^{M} r_m( t + s(z_m − z₁) + τ ) ]² dt
ρ²(s, τ) = ──────────────────────────────────────────────────────────────────────
                Σ_{m=1}^{M} ∫_{t=0}^{Tw} { r_m[ t + s(z_m − z₁) + τ ] }² dt
```

Search-window anchoring:

```
Nominal Expected Arrival Time = Slowness × ( z_middle-Rx )
```
`Early` and `Late` are offsets **relative to that nominal arrival time, per slowness** — not
absolute times. The slowness axis is set by `Min Slow`, `Slow Step`, `Max Slow`; the time axis by
`Early`, `Late`, `Time Step`.

**Nth-Root correlation.** The alternative to semblance. **N is hard-coded to 4** — the user cannot
change it.

**Derived petro-elastic quantities**, as printed:

```
Poissons Ratio = ((0.5*VPVS²) − 1) / ((VPVS²) − 1))          [parentheses unbalanced as printed]
Aniso (%)      = 100 * ( (Slow − Fast) / 0.5*(Slow + Fast) )
Energy Ratio   = Inline / (Inline + Crossline)
```
Poisson's ratio is computed from the "final" VPVS whenever a VPVS has been calculated.

**Gain actions** (applied to waves before semblance):
```
ExpDivide:   out = in / (10^(Gain/10))
ExpMultiply: out = in * (10^(Gain/10))
```

**Anisotropy (Alford rotation).** Two methods, both building a 2-D anisotropy map:
- *Alford (Time)*: XX/YY/XY/YX waves filtered → Alford rotation equations → rotated waves → normal
  semblance, with Min Slowness constrained to min(DTXX, DTYY) and Max Slowness to max(DTXX, DTYY),
  Early/Late from the Anisotropy tab. The constrained 2-D STC is collapsed to a VDL by Sum or Max;
  this becomes one vertical stripe. The rotation is incremented by the Angle Resolution parameter
  and repeated. **Only 180° is computed — the second 180° is a copy of the first (symmetry).**
  The map is then collapsed to one slowness per angle at maximum semblance; if that maximum is
  below the Semblance Cutoff the output slowness for that angle is **nulled**.
- *Alford (Frequency)*: same rotation, then FFT; a frequency/slowness map is built by the Phase
  Slowness method, constrained to the Anisotropy tab's `Process In` frequency range, then collapsed
  to a single slowness per azimuth by **a fit that prefers the low-frequency end**. Same 180°
  symmetry.

Fast-angle determination from the map: **Minimum Energy** or **Minimum Slowness** (user choice).

**Dispersion — two distinct mechanisms, do not conflate:**
1. *Simple Dispersion Correction Factor* — a flat percentage correction applied to the flexural
   curves throughout the interval; **default 0.98**; applied **only** to `DTS_*` curves derived
   from the Dipole modes, and only to the `_smooth` versions.
2. *Frequency-domain dispersion correction* — **TIER C / capability-only.** The page states that a
   **proprietary fit function** is applied to the 2D Frequency Semblance map and the minimum value
   of the fit function is output as `DTmodename_disp`. The functional form is not disclosed and is
   deliberately **not** reconstructed here. Recorded as a capability, not a spec.

**Smoothing.** If the Smoothing Window > 1, data is smoothed over that many depth samples using a
**zero-phase forward-and-reverse digital IIR filter**. Applies only to `_Smooth` outputs.

**Model Missing Data.** A curve is fitted to a VPVS-vs-DTC crossplot over the entire interval;
where compressional exists but shear does not, a modelled shear is generated from the fit, and vice
versa. If neither exists the gap remains. Affects only `_smooth` outputs.

**Receiver stacking modes**: Single Acquisition, Common Transmitter, Common Receiver, Sub Stack,
Combined Wave, Reflection ID. For Reflection ID the running-average length is
`(Num of Acq to Stack × 4) + 1`.

**Output curve dictionary** (all `acoustic_waveform_processing.htm`):
`STC_mn` (STC correlogram arrays), `FVDL`, `FVDL2`, `FSVDL`, `FSVDL2` (frequency VDLs),
`DTC`, `DTS_MP`, `DTXX`, `DTYY`, `DTXY`, `DTYX`, `DTST`, `DTS_modename` (simple-dispersion-corrected
dipole shear), `_psv`, `_Smooth`, `_VPVS`, `TT` / `TT_pip` (integrated travel time),
`DTmodename_disp` (frequency dispersion pick), `AnisoMap`, `AnisoDeltaMap`, `AnisoInlineMap`,
`AnisoCrosslineMap`, `AnisoEnergyRatioMap`, `AnisoEnergyMax`, `AnisoEnergyMin`, `DTFast`, `DTSlow`,
`AnisoAngle`, `AnisoStart`, `AnisoStop`, `STC_mn_RFL`, `Poissons Ratio`, `Poissons Ratio_Smooth`.

Auto-picked Delta-T curves exist for every mode; the page warns the peak-tracking algorithm can
latch onto an incorrect peak when signal is weak, and that results must be inspected.

Imaging-style **LWD anisotropy is explicitly unsupported**.

### 2.4 Dip geometry — the vector-mean transform (`plotting_image_analysis_data.htm`)

This is the core orientation math and it is entirely image-borne on the page; all eleven rasters
were read directly.

Step 1 — each dip is transformed from (Dip Magnitude, Dip Direction) to a unit vector:

```
[img-read: embim393.png]   V_i = ( x_i , y_i , z_i )
[img-read: embim394.png]   x_i = sin(DipMagnitude) × sin(DipDirection)
[img-read: embim395.png]   y_i = sin(DipMagnitude) × cos(DipDirection)
[img-read: embim396.png]   z_i = cos(DipMagnitude)
```

Step 2 — component means (N = number of unit vectors under consideration):

```
[img-read: embim397.png]   x̄ = ( Σ_{i=1}^{N} x_i ) / N
```
(the y and z means follow the same form per the page prose)

Step 3 — renormalise to a unit vector:

```
[img-read: embim398.png]   x̄' = x̄ / √( x̄² + ȳ² + z̄² )
[img-read: embim399.png]   ȳ' = ȳ / √( x̄² + ȳ² + z̄² )
[img-read: embim400.png]   z̄' = z̄ / √( x̄² + ȳ² + z̄² )
[img-read: embim401.png]   ( x̄' , ȳ' , z̄' )
```

Step 4 — transform back:

```
[img-read: embim402.png]   DipMagnitude = cos⁻¹( z̄' )
[img-read: embim403.png]   DipDirection = cos⁻¹[ ȳ' / sin(DipMagnitude) ]
```

**This is a vector (not tensor/axial) mean**, and the printed back-transform is
quadrant-degenerate — see §4.1 and §6.4. It is the single most important geometry result in this
ingest.

### 2.5 Stratigraphic thickness (`imageanalysisandpicks.htm`)

```
TST = MT * ( (Cos WD * Cos DIP) − (Sin WD * Sin DIP * Cos (HAZ − AZM)) )
TVT = TST / Cos DIP
```

| Symbol | Meaning (as printed) |
|---|---|
| TST | true stratigraphic thickness (feet or meters) |
| TVT | true vertical thickness (feet or meters) |
| MT | measured thickness |
| WD | well deviation angle |
| DIP | true dip angle |
| AZM | true dip azimuth |
| HAZ | azimuth of hole direction relative to true north |

Available as spreadsheet columns on the pick set.

### 2.6 Pick geometry & dip picking (`imageanalysisandpicks.htm`, `imageanalysisoverview.htm`)

- Pick tools: Simple, Point, Trace, Auto Dip. Planes are picked as **best-fit sinusoids** through
  the unrolled image; the Trace tool fits a best-fit sinusoid through a traced feature.
- **Dip uncertainty fan**: with the simple sinewave picking method, the displayed uncertainty fan
  is the sensitivity of the resulting dip to varying the pick up/down/left/right by **2 pixels**.
  Works during initial picking only, not during subsequent editing.
- Auto Dip Curves parameters: Amplitude, Azimuth, Quality, Downsampling Factor.
- Pick export curve names: `DPAA`, `DPAZ`, `DPRZ`, `DPAP`, `DPTR`, `DPRP`.
- Exported **Dip Grade** = pick quality, `1.0 = good`, `0.0 = poor`.
- Exported **Dip Symbol** = data-type tadpole shape: `Circle = 1`, `Triangle = 2`, `Square = 3`,
  `Diamond = 4`.
- Exported **Dip Color** = the ARGB colour of the pick type.
- Dip orientation states available throughout: **True / Apparent / De-rotated**.

**Dip de-rotation** (removal of regional dip) is parameterised per zone in the Image Analysis
parameter set: Structural Dip-Angle and Structural Dip-Azimuth (value or curve), plus an
Interpolation Method of **Block** (whole zone de-rotated by the same values) or **Linear** (the
zone values define structural dip at the zone **mid-point**, and picks are de-rotated by linear
interpolation to and from surrounding zones).

### 2.7 Image corrections & processing (`image-corrections-and-processi.htm`)

**Correction order is a hard constraint.** Corrections must be applied in the order listed on the
screen, earliest at the top. Applying a correction means earlier corrections can no longer be run
on the output data. Unavailable corrections state their reason, e.g. a subsequent correction has
already been applied, or the correction cannot be applied to a raw tool where speed correction must
come first.

**Corrections**: Gain; Comb Effect Removal (removes comb effect, depth-aligns pads and buttons);
Button Repair (Fix Button — interpolates between adjacent good buttons, row-aware for two-row
layouts; Clip Data; Null Data); Center Re-projection; Image Normalization (equalises pad and button
values, fixes dead/misbehaving buttons); Equalise Image; Dynamic Image.
Legacy corrections moved to **Image Toolkit → Image Corrections NonStandard**.

**Speed correction / depth-shift curve from accelerometer data.** Z Acceleration is **required**;
at least one of Interval Time, Cumulative Time, Surface Velocity must also be supplied (more is
better). Parameters:

| Parameter | Meaning |
|---|---|
| Logging Direction | up-hole / down-hole; auto-detected if Cumulative Time supplied |
| Calibration Method | **Shift** (double-integrate, calibrate over the defined period) or **Velocity and Shift** (calibrate against velocity after the first integration) |
| Velocity Calibration Window | rolling window size, Velocity-and-Shift method only |
| Automatic Shift Calibration | calibrate over a specified number of Cycles |
| Shift Calibration Window | rolling window, used when Automatic Shift Calibration is off |
| Gravity Filter Window | rolling window used to remove gravity from Z acceleration |
| Acceleration Zero Snap | Z acceleration magnitudes below this (after gravity removal and smoothing) snap to zero — noise suppression when the tool is stuck |
| Acceleration Smoothing Window | 1 = no smoothing |
| Time Interval Smoothing Window | 1 = no smoothing |
| Accelerometer Offset | vertical offset between tool reference depth and accelerometer reference depth; **required for some tools, notably FMI**. Does not affect generation of the depth-shift curve, only its application to image data |
| Output Smoothing Window | 1 = no smoothing |

**All rolling window sizes are defined as an odd number of high-resolution vertical samples.**

This method is *documented*, not proprietary — it is a conventional double-integration with
calibration, so it is specified above rather than capability-only.

**Image Deconvolution.** Two modes. *Average*: mean of all buttons at one depth level, subtracted
from each button. *Running Average*: same, but the mean is taken over several depth levels with a
user-specified window length. Net effect is a **high-pass filter** producing a High Frequency Image;
Low Frequency Image curves are also written with an **`_LFI`** suffix.

**Image Flatten.** Removes the effects of borehole inclination and dipping beds. Requires a Pick Set
with at least one pick (typically regional/structural dip) and preferably a speed-corrected image.
Also outputs average-value curves by tool class — average conductivity and resistivity for pad
conductivity tools (FMI, XRMI, STAR), average amplitude for acoustic tools (UBI, CAST, CBIL).
Named as a prerequisite stage for Image Calibration and secondary-porosity work.

**Image Calibrate** (raw image → calibrated, typically against a shallow resistivity curve).
Three fitting equations selectable: **Linear, Exponential, Power Law**, each with fitting
coefficients `a` and `b` obtained by regression of a Hi-Res image button-array curve against a
low-resolution calibration curve. Regression Calculation Method:
- **Entire Well** — one regression, constant a and b;
- **Use Zones** — separate regression per zone in the Image Analysis zone set; a and b step at zone
  boundaries;
- **User Defined Window** — fixed Window Length, advancing by Window Step; a and b step at each step.

Outputs the calibrated image curves plus **`ACOFF`** and **`BCOFF`** holding the a and b actually
used at each depth step.

**Image Transform.** Same three equation families with a and b. Two flavours:
- *Enhancement* — for low-dynamic-range images (the page's example is an OBM image whose histogram
  piles up at one end). Equation as printed:
  ```
  y = a * log10( x + b )
  ```
  with an inverse option where the equation becomes a divide, stretching the data downward from the
  top of the histogram instead. Warning printed on the page: `b` exists to offset raw data so no
  negative or zero values reach the logarithm; the raw data must be inspected to choose it. A
  warning is issued if a log of a non-positive number is attempted.
- *Porosity* — same as the resistivity calibration except the regression input curve is relabelled
  Porosity, and a porosity histogram array can be output as **`PORMAP`** with dimension equal to the
  specified Number of Bins (Min, Max, Number of Bins user-specified). The page is explicit that
  **despite the name, PORMAP is a histogram, not a map**, and it is written at the low resolution of
  the porosity curve.

**Rotation correction** (e.g. core images): pick pairs of original/destination points on the image
(blue = original, green = destination); at least one pair defines a rotation, several pairs handle
non-constant rotation. Saved as a rotation curve, then applied via Image Toolkit → Image Corrections
NonStandard → Image Rotate.

### 2.8 Fracture analysis (`plotting_image_analysis_data.htm`)

**Terzaghi weighting** corrects the fracture sampling bias arising from fracture orientation
relative to the borehole. Rolling-window statistics:

| Statistic | Definition (as printed) |
|---|---|
| Window height | height of the rolling window; fixed except where truncated at the ends of the analysis depth range |
| Fracture count | number of fractures in the window |
| Fracture density | fracture count / window height |
| Fracture spacing | inverse of the density; **undefined if no fractures in window** |
| Weighted fracture count | sum of the Terzaghi weighting values of all fractures in the window |
| Weighted fracture density | weighted count / window height |
| Corrected fracture spacing | inverse of the weighted density; undefined if no fractures in window |
| Cumulative fracture count | number of fractures between the bottom of the analysed range and the current depth |

The page warns explicitly that density and weighted density **depend on the chosen window height**
— the same picks give different densities at a given depth under a different window.

The Terzaghi factor can become very large under certain conditions, so it must be limited. Two
limiting methods are offered: **limit the factor directly**, or **limit the maximum angle used in
computing the factor**. The limiting value is user-supplied. The factor's own formula is not printed
on the page — see §9.

Curve naming for fracture statistics is `type_stat`, e.g. `AllFr_WDen`.

**Fracture aperture.** Attribution as printed: *"Fracture apertures from electrical borehole scans"*
by **S. M. Luthi and P. Souhaite**, *Geophysics*, **Vo1.55, No. 7, July 1990** ("Vo1" is the vendor's
typo for Vol). Inputs include `Con1` and `Con2` (mud-filtrate-dependent constants), `Rm`, `Rxo`.
Outputs: Aperture (mm), Diff Con (mmho), Permeability (mD), Porosity (v/v). The aperture equation
itself is not printed on this page — see §9.

### 2.9 Net-to-gross from images (`image_toolkit.htm`, `plotting_image_analysis_data.htm`)

**Image Average** produces a single average value for both resistivity and conductivity around the
wellbore at each high-resolution depth step: `AVGCOND`, `AVGRES`.

**Curve Net2Gross** takes a high-resolution bedding-indicator curve (e.g. `AVGCOND`) and thresholds
it. The user declares whether sand is the low-value or high-value side (i.e. conductivity vs
resistivity image). Cutoff **defaults to the curve Mean taken from the curve header** and is
interactively adjustable on the log plot. Outputs into the results set:

| Curve | Definition |
|---|---|
| `SAND` | flag = 1 at every high-res level below the threshold |
| `SHALE` | flag = 1 at every high-res level above the threshold |
| `FACIES` | 1 = Sand, 2 = Shale |
| `BEDTHICK` | bed thickness computed between facies changes |
| `THRESHOLD` | the threshold value applied at each depth |

**Image Net2Gross** is the sibling that takes the whole image array directly without averaging
first, launched by right-clicking the image track. Its outputs are `SAND`, `SHALE`, `FACIES`,
`CutOff1`, `CutOff2`, `CumCutOff1`, `CumCutOff2`.

### 2.10 Navigation QC / inclinometry (`navigation_qc.htm`, `image_toolkit.htm`)

Recomputes Hole Deviation, Hole Azimuth, Pad 1 Azimuth and Relative Bearing from accelerometer and
magnetometer curves.

Outputs: `GXn`, `GYn`, `GZn` (accelerometers corrected for normalization and re-centring);
`HXn`, `HYn`, `HZn` (magnetometers likewise); `GQ` (accelerometer field, vector sum of the three
accelerometer curves); `HQ` (magnetometer field, vector sum of the three magnetometer curves);
`DEVIC`, `HAZIC`, `P1AZC`, `RBC`, `P1NOC` (recalculated nav curves; **P1NOC is Schlumberger tools
only**); `MINC` (magnetic inclination).

**`GQ` and `HQ` are always exactly 1** because the input curves are normalised — the page states
this outright. They are therefore *not* usable as field-strength QC in IP's implementation; the real
QC is `MINC` plotted against the header magnetic inclination (as `MINCTMP`).

Fitting method: **circle fit** or **3D fit** (user choice).

Other parameters: Depth Offset (for service companies whose inclinometry is depth-aligned to the
inclinometry instrument rather than the image tool); RB Offset (where the orientation device is
physically separate from the image tool, a field-calibrated offset curve is supplied and must be
re-applied after recomputation to convert RB-Orientation-Device to RB-Image-Tool); Deviation
Discriminator (include only data within a tolerance of a specified deviation, so the crossplot
shows a single clean arc); Use Hole Survey (recompute azimuths from an external directional survey
when geomagnetic anomalies render magnetometers unusable — deviation and RB still come from the
good accelerometers, and Pad 1 Azimuth is then derived from the three available nav curves).

**Not currently used by any model**: Frame Time, Magnetic Field Intensity. Stated explicitly.

Filters on the outputs are simple symmetric filters; filter lengths are in samples and **even
numbers are rounded up to make them odd**.

The crossplot geometry: data plots as circles or partial circles whose **diameter is a function of
deviation** and whose **arc length depends on tool rotation**. Arcs should be centred on the origin;
off-centre arcs indicate an offset requiring re-centring, either by dragging the target or via
Compute Centroid. The page is explicit that **if the tool is not rotating there are no arcs and
re-centring cannot be assessed at all**.

**Inclinometry Calculator** (`image_toolkit.htm`): given any three of Hole Deviation, Hole Azimuth,
Pad 1 Azimuth, Relative Bearing, computes the fourth. This confirms the four are related by exactly
one constraint equation (the equation itself is not printed — see §9).

### 2.11 Image tool definition & mapping (`manageimagetools.htm`)

Tool state is tri-level: **Can Display / Can Pick / Can Display 3D**, each gated on a specific set of
mapped curves.

**Navigation-curve precedence (load-bearing for orientation):**
- **P1AZ takes precedence over RB + HAZI.** If Pad 1 Azimuth is mapped it is used directly;
  otherwise it is derived from Relative Bearing and Hole Azimuth.
- **Radius takes precedence over Caliper.**

**Acoustic tools** require Transit Time + Fluid Slowness + Effective Head Radius, or alternatively
Excluder Time / Excluder Thickness / Excluder Slowness.

**Multi-stage tools** carry rotational and vertical offsets per stage, plus a nominated Picking
Caliper.

**Magnetic declination.** Stored in both wells and image tools. The well's value lives in the Well
Header → Position tab, in degrees, and is initialised from the input file's **`MDEC`** parameter when
loading DLIS, LIS or LAS. An image tool's declination is initialised from the well's value and is
**independent thereafter**. The tool option *"Magnetic Declination already applied?"* is **selected
by default**, meaning IP assumes declination is already accounted for in the navigation curves;
de-selecting it makes IP apply declination during rendering. If the value changes after picks exist
and the "already applied" option was not selected, IP offers to apply the change to existing pick
sets.

**Dipmeter mnemonic families** documented: HDT, SHDT, OBDT, HDIP, SED. Geometry noted for SHDT:
**3 cm between buttons on a pad**.

Templates are XML: `.itt` (tool) and `.itp` (parameter). Export value range defaults **0–255**.

### 2.12 CAT / RAT production-logging arrays (`cat_rat_normalise.htm`, `cat_rat_image.htm`)

CAT and RAT are the two array holdup tool families handled by the PL image workflow.

**Normalized curve naming**: `CCnnXXXX` for CAT and `RCnnxxxx` for RAT, where `nn` is the sensor
number **00–12** and the trailing four characters are the logging-pass description.

Normalization options:
- **No normalization (copy input curves)** — still must be run, because downstream modules expect
  curves under the calibrated naming convention; outputs are identical values under new names.
- **One-point** — a shift only.
- **Two-point** — a shift plus a span multiplier.
- **Fix at mean** option available.

Holdup endpoints (100% water value and 100% hydrocarbon value) are **entered by the user per tool
type**, and the page warns the value will most likely differ between CAT and RAT sensors so it must
be updated when switching tool types. A **linear relation between sensor response and holdup is
assumed unless a look-up table is supplied.**

Hydrocarbon type affects labelling and colouring only. Default colours: **Oil = Green, Gas = Red,
Condensate = Red, Hydrocarbon = Grey**.

**Array geometry convention (see §4.3)**: after successful rotation correction, the **left and right
edges of the image represent the highside** of the borehole and the **middle represents the
lowside**.

### 2.13 Array-tool velocity (`velocity_for_array_tools.htm`, `velocity_images.htm`)

```
Fluid Velocity = Spinner speed / Slope − Cable Speed + Threshold
```

Provenance note on the minus sign: the clean text renders the operator as a mojibake `?`. The raw
`velocity_for_array_tools.htm` was inspected byte-wise (`od -c`) and the character is octal 226 =
`0x96` = CP1252 EN DASH, used here as a **minus**. Resolved from the bytes, not guessed.

Two-phase handling: slopes are weighted by holdup. Calibration filters, with the page's own examples:
water calibration used only where water holdup `>= 0.9` (at least 90% water); gas calibration only
where water holdup `<= 0.1`; oil calibration only where water holdup `<= 0.1`. Also: **Max Allowed
Vertical Distance** (example 30 — only spinner data within 30% of the borehole's vertical height may
enter the same regression, because segregated fluids can travel at different velocities across the
borehole height), **Slope Discriminator**, **Minimum Number of Regressions**.

**Velocity Images**: builds a velocity array per logging pass plus an averaged array across selected
passes. A **rotation curve is essential** for accurate velocities, with an `Add` offset in degrees
applied throughout the logged interval. De-selecting a bad sensor curve causes it to be **replaced
by an interpolation of the two curves either side of it** — a silent data-fabrication path worth
knowing about. Top/Middle/Bottom curves treat the borehole as three equal horizontal sectors; the
`All` curve weights the three by their **individual sector areas** (the middle sector being the
largest). The page states these three-sector curves are **for reference only** — only the final
averaged velocity array feeds the downstream flow calculation.

### 2.14 Geosteering (`geosteering.htm`)

The page is unusually candid about its own geometry, and the statement is load-bearing:

> the measured dip is neither true dip nor apparent dip — the page calls it a special case of
> apparent dip, close to what LWD terminology calls **relative dip**.

The reasoning given: apparent dip is the bed as viewed from the borehole looking "down dip";
geosteering instead looks "down hole", i.e. at an angle. The page states that in theory one should be
able to convert geosteering dip → apparent dip → true dip, but **does not print that conversion**.
See §5 and §9.

Inputs: TVD, HX (MD) and a correlation curve. The loaded TVD and HX drive the trajectory window,
plus `EDist` and `NDist` curves for the **RSI (Reference Surface Intersection)** when plotted.

Display parameters: New Segment Length; Well Thickness 1–10; Track Width 1–20; Log-Well offset;
**Grid Spacing MD/TVD — auto default `100 x 50`**; light grid lines; zone-name and annotation
toggles. Correlation-curve normalisation in this module is **display-only — no data is changed**.

### 2.15 Image display & 3-D (`imageanalysisoverview.htm`, `image_analysis.htm`, `3d_viewer.htm`)

**Normalization**: Static (fixed customisable scale) or Dynamic (incremental step size). The page
notes the trade-off explicitly — dynamic normalization makes similar lithologies look different in
different parts of the well; static prevents that but restricts visible detail.

**Histogram Method** (how raw values map onto the palette):
- **Normalize** — data linearly normalized, values spread equally across the palette;
- **Equal Area (Default)** — equal-area histogram; high-frequency parts of the histogram are
  emphasised and low-frequency parts neglected; the higher the data frequency, the thinner the bin;
- **Equal Bins** — fixed-width bins.

**17 named image enhancements**: Greyscale, Invert, Contrast Normalize (linear contrast stretch),
Horizontal Sobel, Vertical Sobel, Emboss, Sharpen, Gaussian Sharpen, Conservative Smooth, Mean,
Median, Gaussian Blur, Adaptive Smooth, Blur, Simple Posterization, Increase Brightness,
Decrease Brightness.

**Orientation options for display**: North, Highside, Lowside, Custom.

**Image Tool Profile settings**: View Inclination — the "tip" applied to the profile, **default
20 degrees**; View Declination — rotation of the profile, i.e. which side of the borehole faces out,
**default 0**; Reverse Mapping — caliper radius curves are **by default plotted starting from the
first radius curve in an anti-clockwise direction**, and this option flips them to clockwise.

**Tadpole plotting methods**: Regular, Inverted, Strike.

**Stereonet**: projections available are **Equal-area (Schmidt)**, **Equal-angle (Wulff)** and
**Equal-interval (Kavraiskii)**. Grid styles Polar and Equatorial; points are always plotted on the
equatorial plane, projected from the selected hemisphere. **Hemisphere default = Upper.** Points may
be drawn as the pole of the dip plane or as great circles. Rose plots may be plotted as **Azimuth**
(summarises dip-azimuth values) or **Strike** (summarises the horizontal-plane intersection,
perpendicular to azimuth); azimuth-rose bins may be generated **From Dip** or **From Pole to Plane**.
Statistics reported include **Vector Mean Azimuth** and **Vector Mean Angle** (computed by the §2.4
transform). Merged areas report **the mean of individual group means**, not a pooled mean — §6.7.

**Image Histogram Plot**: number of bins selectable **2 to 256**; number of thresholds limited to
**five**; threshold test is a Less-than / Greater-than logic switch; summation direction Top Down or
Bottom Up; each threshold writes a CutOff curve and a Cumulative CutOff curve.

**Walkout plots**: four types — Dip-Azimuth, Dip-Angle, Directional Dip-Angle (vectors drawn
left/right based on look-azimuth difference), and 3D Dip-Angle/Dip-Azimuth. The 3D plot offers
**Use Down Plunge View**, which the page describes as calculating the **minimum eigen values** to
give the best viewing angle of the data. Vectors may be plotted as Pole or Dip-Vector.

**Cumulative plots**: Cumulative (sum of dip orientation) or Difference (difference between
consecutive points), against sample number, depth, or first derivative.

---

## 3. Parameters, defaults & constraints

### 3.1 Explicit defaults (value stated by the page as the default)

| Parameter | Default | Module | Source |
|---|---|---|---|
| Nth Root order N | **4, hard-coded** (not user-settable) | Acoustic Waveform | `acoustic_waveform_processing.htm` |
| Simple Dispersion Correction Factor | **0.98** | Acoustic Waveform | `acoustic_waveform_processing.htm` |
| Histogram Method | **Equal Area** | Image display | `imageanalysisoverview.htm` |
| Stereonet Hemisphere | **Upper** | Stereonet | `plotting_image_analysis_data.htm` |
| Magnetic Declination already applied? | **selected (on)** | Manage Image Tools | `manageimagetools.htm` |
| View Inclination | **20°** | Image Tool Profile | `plotting_image_analysis_data.htm` |
| View Declination | **0** | Image Tool Profile | `plotting_image_analysis_data.htm` |
| Radius-curve plotting direction | **anti-clockwise from first radius curve** | Image Tool Profile | `plotting_image_analysis_data.htm` |
| Geosteering grid spacing (auto) | **100 × 50** (MD × TVD) | Geosteering | `geosteering.htm` |
| Net2Gross cutoff | **curve Mean from the curve header** | Curve Net2Gross | `image_toolkit.htm` |
| Image export value range | **0–255** | Manage Image Tools | `manageimagetools.htm` |
| Holdup colours | Oil = Green, Gas = Red, Condensate = Red, Hydrocarbon = Grey | CAT/RAT | `cat_rat_image.htm` |
| Single Active Zone mode | **on** (Multiple Active Zones cleared) | Zone list | `plotting_image_analysis_data.htm` |
| Select When Active | **selected** | Zone list | `plotting_image_analysis_data.htm` |
| Replacement-velocity reference depth | first data point of the selected sonic | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Array velocity curves | all input curves **on** by default | Velocity Images | `velocity_images.htm` |
| Synthetic time-curve depth range | top and bottom of the current PL analysis | Velocity Images | `velocity_images.htm` |

### 3.2 Hard numeric limits and validity gates

| Constraint | Value | Module | Source |
|---|---|---|---|
| Sonic auto-null threshold | values **≤ 25 µs/ft (80 µs/m)** are automatically nulled | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Expected P-velocity range | **600 – 9000 m/s** (warning outside) | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Expected S-velocity range | **300 – 6000 m/s** | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Replacement velocity range | **1500 – 4000 m/s**, and must be positive | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Incidence angle | `0 ≤ min < max`, **max ≤ 60°** — "the method used is not valid beyond 60 degrees" | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Incidence step | **≥ 0.01°**, integer or decimal | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Max angles per gather | **61** | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Ricker peak frequency | **≥ 5 Hz**, and must be **≤ Nyquist = 0.4 / sample rate** else aliasing | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Sample rate | **2 – 8 ms** | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Wavelet normalisation | max amplitude **1**; truncated to points with amplitude **> 1% of maximum** | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Stack curve name length | **≤ 12 characters** (reflectivity variants append `_R`) | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| Well inclination for valid synthetics | **≤ 30°** — module only produces valid synthetics below this | Synthetic Seismogram | `synthetic_seismorgram.htm` |
| EI angles per run | **up to 10** | Elastic Impedance | `elastic_impedance.htm` |
| EI low-angle vs high-angle | little difference **below ~20°**; low-angle form becomes unstable **above 30°** | Elastic Impedance | `rockphysics.htm` |
| Dip uncertainty fan | **± 2 pixels** | Image picking | `imageanalysisandpicks.htm` |
| Sensor number range (CAT/RAT) | **00 – 12** | CAT/RAT | `cat_rat_normalise.htm` |
| Image histogram bins | **2 – 256** | Image Histogram | `plotting_image_analysis_data.htm` |
| Image histogram thresholds | **max 5** | Image Histogram | `plotting_image_analysis_data.htm` |
| Anisotropy map angular coverage | **180° computed; second 180° is a copy (symmetry)** | Acoustic Waveform | `acoustic_waveform_processing.htm` |
| Anisotropy semblance cutoff | below cutoff → slowness **nulled** for that azimuth (page example value 0.25) | Acoustic Waveform | `acoustic_waveform_processing.htm` |
| Nav-QC filter length | samples; **even lengths rounded up to odd** | Navigation QC | `navigation_qc.htm` |
| Speed-correction window sizes | **odd number of high-resolution vertical samples**, all of them | Image Corrections | `image-corrections-and-processi.htm` |
| Reflection ID running average | `(Num of Acq to Stack × 4) + 1` | Acoustic Waveform | `acoustic_waveform_processing.htm` |
| Geosteering well thickness / track width | 1–10 / 1–20 | Geosteering | `geosteering.htm` |

### 3.3 Page-supplied example values (NOT defaults — do not adopt as such)

These are printed as illustrations. They are recorded so they are never mistaken for defaults.

| Value | Context | Source |
|---|---|---|
| Semblance Cutoff 0.25 | shown alongside "Semblance Cutoff = 0" as a comparison figure | `acoustic_waveform_processing.htm` |
| Water-calibration holdup `>= 0.9` | *"for example, if the parameter is set to…"* | `velocity_for_array_tools.htm` |
| Gas/oil-calibration holdup `<= 0.1` | same phrasing | `velocity_for_array_tools.htm` |
| Max Allowed Vertical Distance 30 (%) | *"For example if the parameter value is set to 30…"* | `velocity_for_array_tools.htm` |
| Correlation window 500 µs at 40 µs/ft slowness, start ~1000 µs | figure caption illustrating the semblance window | `acoustic_waveform_processing.htm` |
| 50 µs/ft × 8 ft ≈ 400 µs nominal arrival | worked illustration of Early/Late anchoring | `acoustic_waveform_processing.htm` |
| Angle sweep 0–35° step 1° → 36 series | worked illustration | `synthetic_seismorgram.htm` |

---

## 4. Geometry & orientation conventions (explicit)

This section is the answer to special tasking (a) and is the part most likely to be silently
mis-implemented. Every convention below is stated as the page states it, with the gaps named.

### 4.1 The dip unit-vector convention

From §2.4, with `M` = Dip Magnitude and `A` = Dip Direction:

```
x = sin(M) · sin(A)
y = sin(M) · cos(A)
z = cos(M)
```

What this fixes, unambiguously:

- **The vector is the dip vector expressed in a geographic (North-referenced) frame, not a
  borehole frame.** `y` carries `cos(A)` and `x` carries `sin(A)`, so **y is the North component and
  x is the East component** — the standard compass convention where azimuth is measured clockwise
  from North. It is *not* the mathematical convention (x = cos, y = sin measured anticlockwise from
  East).
- **z = cos(M), so z is the borehole-axis-independent vertical component**, positive for a
  horizontal bed (M = 0 → z = 1). The vector points along the **pole/normal sense for magnitude but
  along the dip azimuth sense for direction** — i.e. it is a unit vector whose plunge from vertical
  equals the dip magnitude and whose horizontal bearing is the dip direction.
- Because `z = cos(M)` and dips are 0–90°, **z is always ≥ 0**: all vectors sit in one hemisphere by
  construction. This is why the arithmetic mean does not immediately suffer the antipodal
  cancellation that a true axial dataset would. It is a **vector mean, not an axial (eigenvector)
  mean** — for data spanning near-vertical dips or conjugate sets, this is a real methodological
  choice with real consequences (§5).

**What the page does NOT state**: whether the frame is true-North or magnetic-North at this point in
the pipeline. It is resolvable only indirectly, via the declination handling in §4.4 — the
declination is applied (or asserted already applied) at the *tool/rendering* level, so by the time a
dip has a Dip Direction it should already be true-North referenced. The page does not say so
explicitly. Recorded in §9.

### 4.2 Apparent → true dip conversion

**The explicit apparent-to-true conversion formula is not printed on any of my 20 pages.** What is
present:

- The three dip states — **True, Apparent, De-rotated** — are first-class and switchable on every
  dip plot (walkout, cumulative, scatter, stereonet).
- The page definitions: *true dip* is the bed's orientation in 3D space; *apparent dip* is the same
  bed as measured when viewed from the borehole (`geosteering.htm`).
- De-rotated dip = true dip after structural dip has been removed (`imageanalysisoverview.htm`),
  with the Block/Linear interpolation rule in §2.6.
- The TST/TVT equations in §2.5 are the only printed formula that explicitly couples well deviation
  (WD), hole azimuth (HAZ), dip angle (DIP) and dip azimuth (AZM) — and they are thickness
  equations, not dip-conversion equations.

Do not infer the conversion from the TST equation. Flagged in §9.

### 4.3 Image unrolling and pad-to-image mapping

- The **Inclinometry Calculator** confirms Hole Deviation, Hole Azimuth, Pad 1 Azimuth and Relative
  Bearing are related by exactly one equation — any three give the fourth (`image_toolkit.htm`).
  The equation is not printed.
- **P1AZ takes precedence over RB + HAZI** when both are mapped; otherwise P1AZ is derived
  (`manageimagetools.htm`). Any reimplementation must reproduce this precedence or images will
  rotate differently from IP's for tools that carry both.
- **Radius takes precedence over Caliper** (`manageimagetools.htm`).
- Multi-stage tools carry **rotational and vertical offsets per stage** plus a nominated **Picking
  Caliper** (`manageimagetools.htm`).
- **PL array convention (CAT/RAT and velocity images)**: after successful rotation correction, the
  **left and right edges of the array image are highside** and the **middle is lowside**
  (`cat_rat_image.htm`, `velocity_images.htm`). This is the opposite edge/middle assignment from the
  common "image starts at highside and wraps" convention, and it applies to the PL array images
  specifically. Getting this backwards inverts every gas/water interpretation on the image.
- Display orientation choices are **North / Highside / Lowside / Custom**
  (`imageanalysisoverview.htm`).
- Caliper radius curves are plotted around the borehole **anti-clockwise from the first radius
  curve** by default (`plotting_image_analysis_data.htm`).
- **Create Array from Quadrants** for LWD takes the four quadrant curves in the order
  **Up, Right, Down, Left** (`image_toolkit.htm`) — that ordering is the array element order.

### 4.4 Magnetic declination handling

Fully specified (`manageimagetools.htm`), and this is the cleanest orientation contract in the
manual:

1. Declination lives in **two independent places**: the well header (Position tab, degrees,
   initialised from the file's **`MDEC`** parameter on DLIS/LIS/LAS load) and the image tool
   (initialised from the well, independent thereafter).
2. The image tool carries the flag **"Magnetic Declination already applied?"**, **default ON**.
   ON ⇒ IP assumes the navigation curves are already true-North referenced and applies nothing.
   OFF ⇒ IP applies declination to the nav-curve calculations during rendering.
3. Changing the value after picks exist, with the flag OFF, prompts to propagate the change to
   existing pick sets.

In **Navigation QC**, declination is read from the Well Header Position tab and **is applied to the
re-computed curves** (`navigation_qc.htm`). Magnetic Inclination from the same header is used only as
a QC overlay against the computed `MINC`. Magnetic Field Intensity is read but **not used**.

**The double-application hazard is real and is created by the default.** The tool flag defaults to
"already applied", but Navigation QC applies declination when it recomputes. A tool whose nav curves
were replaced by Nav-QC outputs has had declination applied once by Nav-QC; the tool flag's default
then correctly asserts "already applied". But if a user flips the tool flag OFF while using Nav-QC
outputs, declination is applied twice. The manual does not warn about this.

### 4.5 Acoustic array geometry

- Receivers at distances `z₁ … z_M` from the transmitter; the moveout term in the semblance
  numerator/denominator is `s(z_m − z₁)`, i.e. **slowness times offset measured from the first
  receiver** (`[img-read: _awpclip0025.png]`).
- The search-time anchor uses `z_middle-Rx`, i.e. the **middle receiver** offset, not the first
  (`acoustic_waveform_processing.htm`). The two references are deliberately different: moveout is
  relative to R1, absolute expected arrival is relative to the array mid-point.
- Anisotropy azimuth: the Azimuth curve and Azimuth offset must be specified in Configure Tool to
  correct results for tool position; **if they are not specified, the anisotropy results are in the
  orientation of the tool, not the earth** (`acoustic_waveform_processing.htm`). This is stated
  plainly and is an easy silent error.

### 4.6 Stratigraphic-thickness frame

TST/TVT (§2.5) use `HAZ` = azimuth of hole direction **relative to true north**, and `AZM` = true dip
azimuth. The coupling term is `Cos(HAZ − AZM)` — the azimuthal difference between hole direction and
dip direction. `WD` is the well deviation angle from vertical. This is a bedding-normal thickness
computation in the geographic frame.

---

## 5. Assumptions & validity limits

1. **Synthetic seismograms are 1D.** No ray tracing except Backus averaging
   (`synthetic_seismorgram.htm`). Combined with the hard 30° inclination gate, this module is a
   near-vertical-well tool.
2. **The Aki & Richards linearisation is a first-order approximation** and IP enforces its own
   validity envelope at 60° incidence, stating the method is not valid beyond that
   (`synthetic_seismorgram.htm`).
3. **Backus averaging assumes a transversely isotropic equivalent medium** with an aperture set by
   wavelength/frequency (`synthetic_seismorgram.htm`).
4. **EI is unit-system dependent** — see §2.1. The output is not a physical impedance in any
   consistent unit set; it is a curve whose shape depends on the units chosen.
5. **The dip mean is a vector mean, not an axial mean** (§4.1). For steep or conjugate dip
   populations this differs from the eigenvector/Bingham approach. IP does use eigen-analysis
   elsewhere (walkout "Use Down Plunge View" is described as using minimum eigen values), so the
   choice in the mean-dip routine is deliberate, not an oversight.
6. **Terzaghi weighting is unbounded** and must be limited; the page says so outright and offers
   two limiting strategies. Any density/spacing number is meaningless without its window height and
   its limiting rule (`plotting_image_analysis_data.htm`).
7. **Fracture spacing is undefined in windows with zero fractures** — an explicit null, not a zero.
8. **CAT/RAT holdup assumes a linear sensor-response-to-holdup relation** unless a look-up table is
   supplied (`cat_rat_normalise.htm`).
9. **Deselected velocity sensors are silently replaced by interpolation of their neighbours**
   (`velocity_images.htm`) — the output array will look complete regardless.
10. **Nav-QC `GQ`/`HQ` are always 1** by construction and carry no QC information
    (`navigation_qc.htm`).
11. **Nav-QC re-centring is impossible without tool rotation** — no arcs, no assessment
    (`navigation_qc.htm`).
12. **Geosteering dip is neither true nor apparent dip**, and the conversion to true dip is
    described as theoretically possible but is not provided (`geosteering.htm`). Any number taken
    out of geosteering is in its own frame.
13. **LWD imaging-style anisotropy is unsupported** in Acoustic Waveform Processing.
14. **Auto-picked Delta-T can latch onto the wrong peak** in low-signal zones — the manual says the
    user must inspect (`acoustic_waveform_processing.htm`).
15. **Correction order is irreversible**: once a later correction is applied, earlier ones can no
    longer be run on that output (`image-corrections-and-processi.htm`).
16. **Merged stereonet areas report a mean of means**, not a pooled mean (§6.7).
17. **Image histogram/density statistics depend on the chosen window or bin count** and are not
    comparable across different settings.

---

## 6. Internal discrepancies

Each of these is a place where the manual contradicts itself, prints an unresolvable symbol, or is
ambiguous in a way that would produce a silently wrong number.

**6.1 — EI: K is both a pointwise ratio and a scalar constant.** `[img-read: embim615.png]` defines
`K = Vs²/Vp²` (a curve-valued quantity), while the prose describes K as a constant representing the
average over the interval and provides a *Calculate K from interval* button plus an optional
`EI_Kconst` output curve holding "the instantaneous K value over the whole well"
(`elastic_impedance.htm`). So: the equation is evaluated with a **scalar** K, and `EI_Kconst` exists
purely as a diagnostic of what K would have been pointwise. The manual never says this in one place.
Severity: **medium** — using pointwise K in the EI exponent gives a different (and non-Connolly)
curve.

**6.2 — Backus: two undefined symbols in the impedance equations.** Step 2 defines `V_{S,V}`
(`embim622`) and `V_P` (`embim623`), but the impedance lines use `V_{P,V}` (`embim625`) and
`V_{SH,V}` (`embim626`). Neither `V_{P,V}` nor `V_{SH,V}` is defined anywhere on the page. The
obvious reading is that `V_{P,V}` is a typo for the `V_P` of `embim623` and `V_{SH,V}` a typo for the
`V_{S,V}` of `embim622` — but that is an inference, not something the page states, and the `SH`
subscript hints at a horizontally-polarised shear distinct from the `S,V` (vertically-polarised)
one, which in a TI medium is a genuinely different velocity. Severity: **high** for anyone
implementing Backus impedances. Escalated to §9.

**6.3 — Backus: which ρ is inside the square roots?** `V_{S,V} = √(D/ρ)` and `V_P = √(C/ρ)` are
printed **before** `ρ_Backus = ⟨ρ_b⟩` is defined (equation order is 622, 623, 624). The `ρ` in the
radicals is therefore either the raw `ρ_b` or the Backus-averaged `ρ_Backus`. Physically the
upscaled velocity should use the upscaled density, but the page does not say so. Severity:
**medium-high**.

**6.4 — Mean dip: the back-transform is quadrant-degenerate.**
`DipDirection = cos⁻¹[ ȳ' / sin(DipMagnitude) ]` (`[img-read: embim403.png]`) returns a value in
[0°, 180°] only. Since `ȳ'` is the North component, the arc-cosine cannot distinguish an easterly
mean direction from its westerly mirror — the sign of `x̄'` is required to place the result in the
correct half of the compass, and **the page never uses `x̄'` in the back-transform at all**. As
printed, every mean dip direction in the western half (180°–360°) would be reported as its eastern
mirror. IP's actual implementation must use a two-argument arctangent or an explicit sign test; the
documented equation is incomplete. Severity: **high** — this is exactly the kind of documented-but-
wrong formula that gets copied into a reimplementation and produces plausible, wrong azimuths.
Also note `sin(DipMagnitude)` → 0 for horizontal beds, making the expression singular there.

**6.5 — Poisson's ratio has unbalanced parentheses.**
`Poissons Ratio = ((0.5*VPVS²) − 1) / ((VPVS²) − 1))` as printed
(`acoustic_waveform_processing.htm`) has one more closing than opening parenthesis. The intended
grouping is unambiguous from the numerator/denominator layout, but the printed string will not parse.
Severity: **low** (typographic), recorded for completeness.

**6.6 — Net2Gross sand-side flag contradiction.** The configuration text says sand can be
represented by values above **or** below the threshold (user's choice, depending on whether the input
is a conductivity or resistivity image), but the output-curve definitions hard-code
`SAND` = below threshold and `SHALE` = above threshold (`image_toolkit.htm`). One of the two is
wrong; most likely the output description assumes the conductivity case. Severity: **medium** — a
flipped facies flag is silent.

**6.7 — Merged-area statistics are a mean of means.** The page states that for merged (grouped)
areas, the reported means are calculated as the mean of the individual group means
(`plotting_image_analysis_data.htm`). This is **not** the same as the vector mean over the pooled
points unless the groups are equal-sized, and it contradicts the natural reading of "merge these
areas and give me the statistics". Severity: **medium**, and it is at least stated openly.

**6.8 — Acoustic inputs documented as present but unused.** `Frame Time` and `Magnetic Field
Intensity` in Navigation QC are both described as "not currently used with any of the models"
(`navigation_qc.htm`), yet they are exposed as input parameters. Harmless, but it means a user can
set them and see no effect.

**6.9 — Citation errors in the vendor text.** Recorded because §7 shows they have persisted across
two major releases and because getting the reference right matters for independent verification:
- *"Kimball and Marzeta"* — the actual authors of the 1984 *Geophysics* semblance paper are
  **Kimball and Marzetta**.
- *"Geophysics Vol.49 Mo.3"* — "Mo.3" is a typo for **No.3**.
- *"Geophysics, Vo1.55, No. 7, July 1990"* (Luthi & Souhaite) — "Vo1" is a typo for **Vol**.
These are reported as the page prints them, per rule 3, with the correction noted separately.

**6.10 — `PORMAP` is named a map but is a histogram.** The page says so itself
(`image-corrections-and-processi.htm`). Recorded so nobody treats it as an azimuthal array.

---

## 7. IP2018 numeric diff

Method: for every one of my 20 pages, the counterpart `c18/<same-name>.htm` was checked for
existence, tag-stripped, and searched for each equation, constant, default and limit recorded above.
Equation rasters that differed in filename were opened and compared visually.

### 7.1 Pages present in both — numeric drift

| Item | IP2018 | IP2025 | Drift |
|---|---|---|---|
| EI attribution, eq 4.1, 20°/30° stability notes | identical | identical | **none** |
| EI: up to 10 angles, `EI_10/20/30`, `EI_Kconst`, AI/EI ratio normalization, linear extrapolation, unit-conversion-on-inputs warning | identical | identical | **none** |
| Sonic auto-null 25 µs/ft (80 µs/m) | identical | identical | **none** |
| P 600–9000 m/s, S 300–6000 m/s, replacement velocity 1500–4000 m/s | identical | identical | **none** |
| Incidence ≤ 60°, step ≥ 0.01°, max 61 angles | identical | identical | **none** |
| Ricker ≥ 5 Hz, Nyquist 0.4/sample rate, sample rate 2–8 ms, amplitude norm to 1, 1% truncation | identical | identical | **none** |
| Stack name ≤ 12 chars + `_R` | identical | identical | **none** |
| Max well inclination 30° | identical | identical | **none** |
| Nth Root N hard-coded to 4 | identical | identical | **none** |
| Simple Dispersion Correction default 0.98 | identical | identical | **none** |
| Semblance Cutoff illustration 0.25 | identical | identical | **none** |
| Proprietary dispersion fit function statement | identical wording | identical wording | **none** |
| Poisson's ratio equation (incl. the unbalanced paren) | identical | identical | **none** |
| Kimball/Marzeta citation incl. both typos | identical | identical | **none** |
| `Nominal Expected Arrival Time = Slowness * (zmiddle-Rx)` | identical | identical | **none** |
| Matlab 2015a (V8.5) from IP 4.4 update 2 | identical | identical | **none** |
| TST / TVT equations and symbol list | identical | identical | **none** |
| Dip uncertainty ± 2 pixels | identical | identical | **none** |
| Mean-dip vector transform (11 equations) | rasters `embim576–586.gif` | rasters `embim393–403.png` | **none** — `embim577/578` and `embim585/586` opened and compared: byte-different files, mathematically identical, including the quadrant-degenerate `cos⁻¹` back-transform of §6.4 |
| Terzaghi weighting + limiting-method choice | identical | identical | **none** |
| Luthi & Souhaite 1990 attribution incl. "Vo1" typo | identical | identical | **none** |
| Image Deconvolution (Average / Running Average, `_LFI`) | identical | identical | **none** |
| Histogram methods (Normalize / Equal Area default / Equal Bins) | identical | identical | **none** |
| 17 image enhancements | identical list, identical order | identical | **none** |
| Dip Grade 1.0/0.0; Dip Symbol Circle 1/Triangle 2/Square 3/Diamond 4; ARGB Dip Color | identical | identical | **none** |
| SHDT 3 cm between buttons on pad | identical | identical | **none** |
| Magnetic declination model (well ↔ tool, `MDEC`, "already applied?" default on, pick-set propagation) | identical | identical | **none** |
| Fluid Velocity = Spinner speed / Slope − Cable Speed + Threshold | identical | identical | **none** |
| Holdup cutoff examples 0.9 / 0.1, Max Allowed Vertical Distance 30% | identical | identical | **none** |
| CAT/RAT naming `CCnnXXXX` / `RCnnxxxx`, sensors 00–12, "no normalization still must be run" | identical | identical | **none** |
| Holdup default colours (Oil Green / Gas Red / Condensate Red / HC Grey) | identical | identical | **none** |
| PL array highside=edges, lowside=middle | identical | identical | **none** |
| Geosteering "neither true dip nor apparent dip… relative dip" | identical | identical | **none** |
| Geosteering grid auto 100×50, well thickness 1–10, track width 1–20 | identical | identical | **none** |
| RSI / EDist / NDist | identical | identical | **none** |
| `GQ`/`HQ` always 1 | identical | identical | **none** |

**Result: zero numeric drift.** Not one equation, constant, default, threshold or range in my
20-page scope changed between IP2018 and IP2025. Every difference found is additive or structural.

### 7.2 Structural / behavioural changes (IP2018 → IP2025)

1. **Section renamed.** `rockphysics.htm` is titled *Rock Physics* in IP2018 and **Geophysics** in
   IP2025, and the IP2025 roster **adds Acoustic Waveform Processing** to the listed modules (the
   module itself existed in IP2018 but was not listed in the section hub).
2. **`image-corrections-and-processi.htm` is new in IP2025.** In IP2018 the image-corrections
   material lived inside `imageanalysisoverview.htm` as a short list. The IP2025 page is a full
   module page and adds material that has **no IP2018 counterpart anywhere in the CHM** (verified by
   whole-corpus grep of `c18`):
   - **Image Calibrate** and **Image Transform** with the Linear / Exponential / **Power Law**
     equation choice — `"Power Law"` appears in **0** IP2018 files;
   - **`ACOFF` / `BCOFF`** coefficient outputs — **0** IP2018 files;
   - **`PORMAP`** porosity histogram output — **0** IP2018 files;
   - Enhancement `y = a * log10(x + b)` and its inverse variant;
   - **Image Flatten**;
   - **Button Repair** (Fix Button / Clip Data / Null Data);
   - Rotation Corrections (pick-pair rotation curve → Image Rotate).
   The correction roster also changed name and shape: IP2018's *Depth Align*, *Streak*, *Fill Gaps*,
   *Negative Tapering*, *Accelerometer* became IP2025's *Comb Effect Removal*, *Image Normalization*,
   *Equalise Image*, *Dynamic Image*, with the legacy ones relocated to **Image Toolkit → Image
   Corrections NonStandard**.
3. **`image_toolkit.htm` is new in IP2025.** No IP2018 counterpart. Its contents — Image Average
   (`AVGCOND`/`AVGRES`), **Curve Net2Gross** and **Image Net2Gross** (`"Net2Gross"` appears in **0**
   IP2018 files), Create Array from Quadrants, **Inclinometry Calculator**, **Stuck Tool
   Calculator**, **Travel Time To Radius**, Image Corrections NonStandard, User App from Image Tool
   — are all new capability.
4. **Navigation QC gained substantial capability and changed a declination contract.** This is the
   only behavioural change in my scope that alters a computed result:
   - IP2018: *"Magnetic Declination / Inclination: These are view-only and are read from the **Image
     Tool Template**."*
   - IP2025: Magnetic Declination is read from the **Well Header → Position tab** and **is applied to
     the re-computed curves**; Magnetic Inclination is used as a QC overlay against the computed
     `MINC`; Magnetic Field Intensity is read but unused.

     So the **source of the declination changed (tool template → well header) and its role changed
     (view-only → applied)**. A well processed through Nav-QC in IP2018 and IP2025 with divergent
     tool-template and well-header declinations will produce **different azimuths**. This is the
     single highest-consequence difference found.
   - **New output curves in IP2025**: `P1NOC` (Schlumberger tools only) and `MINC`. IP2018 wrote only
     `DEVIC`, `HAZIC`, `P1AZC`, `RBC`.
   - **New parameters in IP2025**: Frame Time, Magnetic Field Intensity, Depth Offset, Deviation
     Discriminator (with an interactive line on the log plot), fitting **Method (circle fit / 3D
     fit)**, Additional Corrections → Curve Edit, and a Reset button.
   - Input renamed: IP2018 *Survey Hole Azimuth* → IP2025 *Hole Survey Azimuth*.
5. **Image Tool Profile settings are new in IP2025.** `"View Inclination"`, `"View Declination"` and
   the anti-clockwise radius-plotting note appear in **0** IP2018 files. The 20° / 0° defaults and
   the anti-clockwise convention therefore have no IP2018 baseline to diff against.
6. **Pages absent from IP2018 entirely**: `image-corrections-and-processi.htm`, `image_toolkit.htm`,
   `3d-viewer.htm`, `analysis-sticks.htm` (the last two being IP2025's empty stubs).

---

## 8. SandiBumi notes

Ordered by what would cost the most if got wrong.

**8.1 — Implement the mean-dip back-transform correctly, and document the deviation.**
IP's printed `DipDirection = cos⁻¹(ȳ'/sin M)` cannot resolve east from west (§6.4). SandiBumi should
use `atan2(x̄', ȳ')` normalised to [0°, 360°), which reduces to IP's expression on the eastern half
and gives the correct answer on the western half. This must be an explicit, documented divergence
from IP with a test case in the west (e.g. a set of dips clustered at azimuth 250°) so the difference
is visible and intentional rather than an unexplained mismatch during any cross-validation against
IP. Also handle `M → 0` (singular) and decide the near-vertical case.

**8.2 — Decide vector mean vs axial mean deliberately, and label the output.**
IP computes an arithmetic vector mean of unit dip vectors (§4.1, §5.5). That is defensible for
bedding but is the wrong statistic for fracture sets and any bimodal/conjugate population, where the
eigenvector (Bingham/Woodcock) approach is standard. SandiBumi should either match IP exactly and
label the curve "vector mean" without qualification, or offer both and make the choice explicit in
the output metadata. Silently substituting an eigen-mean would make SandiBumi disagree with IP on
real datasets with no visible cause.

**8.3 — Treat the frequency-domain dispersion correction as Tier C, capability-only.**
The fit function is proprietary and undisclosed (§2.3). SandiBumi may offer *a* dispersion
correction, but it must be its own documented method with its own citation — never a reconstruction
of IP's, and never named or positioned as equivalent. The **simple 0.98 flat factor is a different
thing entirely** and is fully documented, so it can be implemented as-is with attribution to the
manual.

**8.4 — Adopt the validity gates, do not soften them.**
The synthetic-seismogram envelope (60° incidence ceiling, 30° well-inclination gate, 25 µs/ft null,
velocity ranges, 2–8 ms sample rate, Nyquist check) is a coherent, defensible set that IP has held
stable across two releases. Any SandiBumi module that claims to exceed IP should *widen* these only
with a stated method change, not by omitting the check. The 30° inclination gate in particular is a
real limitation of 1D modelling and is a candidate area to genuinely exceed IP — but only with ray
tracing or an explicit deviated-well treatment, not by removing the warning.

**8.5 — Carry the EI unit-dependence forward as a hard interface rule.**
EI's exponents make it unit-system dependent (§2.1, §5.4). SandiBumi should record the unit system in
the EI curve's own metadata and refuse post-hoc unit conversion on the output, mirroring IP's rule.
Better: emit the unit system in the curve description automatically, which IP does not do.

**8.6 — The Nav-QC declination change is a cross-version reproducibility trap.**
Because IP2018 sourced declination from the tool template and IP2025 sources it from the well header
and applies it (§7.2.4), historical IP-processed azimuths are not necessarily reproducible from the
same inputs under IP2025. If SandiBumi ever ingests or validates against IP-derived nav curves, the
IP version must be recorded alongside them. This also argues for SandiBumi storing declination once,
in one place, with the applied/not-applied state as an explicit boolean on the data — which is
essentially IP's `"already applied?"` flag, but promoted from a per-tool UI checkbox to a data
property.

**8.7 — Reproduce the nav-curve precedence rules exactly.**
P1AZ over RB+HAZI, and Radius over Caliper (§4.3). These are cheap to implement and expensive to
discover: an image that is rotationally offset from IP's by a constant is a nightmare to debug.

**8.8 — The PL array highside/lowside convention is inverted relative to intuition.**
Edges = highside, middle = lowside (§4.3). Hard-code it with a named constant and a test, and put it
in the schema documentation, not just the renderer.

**8.9 — Fracture statistics must carry their window height and limiting rule as metadata.**
Terzaghi density is meaningless without both (§5.6). SandiBumi should refuse to emit a weighted
density curve without recording the window height and the factor/angle limit that produced it. This
is a place SandiBumi can straightforwardly exceed IP, which documents the dependence but does not
enforce anything.

**8.10 — Do not copy the "interpolate over deselected sensors" behaviour silently.**
Velocity Images replaces a deselected curve with an interpolation of its neighbours with no marker in
the output (§5.9). If SandiBumi does this, it must flag the affected samples in a quality curve.

**8.11 — The `GQ`/`HQ` = 1 result is a wasted QC opportunity.**
IP normalises the accelerometer and magnetometer inputs before computing the field magnitude, so the
magnitude carries no information (§5.10). SandiBumi should compute the field magnitude from
**un-normalised** raw counts, where deviation from the expected local gravity / field intensity is a
genuine tool-health indicator — and the well header already carries Magnetic Field Intensity, which
IP reads and then does not use (§6.8). This is a concrete, low-cost way to exceed IP.

**8.12 — Backus impedances are blocked on §6.2 / §6.3.** Do not implement `I_P-backus` /
`I_S-backus` from this manual alone; the symbol definitions are missing. Go to Backus (1962) directly.

**8.13 — Smectite / montmorillonite endpoints: none found.** No smectite or montmorillonite
parameter, endpoint or mention appears anywhere in my 20 pages. These are geophysics/imaging modules
and carry no clay-mineral endpoints. Reported explicitly per the standing instruction.

---

## 9. OPEN ITEMS

Ranked by consequence. Each is something a page implies, references or requires but does not print
in a form I could resolve without guessing.

**9.1 (HIGH) — Apparent ↔ true dip conversion formula.** Not printed on any of my 20 pages, despite
True / Apparent / De-rotated being first-class switchable states throughout the product. The TST/TVT
equations couple the same variables but are thickness equations and must not be repurposed. *Where to
look next*: an IP module page outside my scope (dip computation / image picking internals), the
IP2018 `.hlp` module-parameter register, or the IP2025 Tier-C register. Assign to whichever agent
holds the remaining image-analysis or general-methods pages.

**9.2 (HIGH) — Backus `V_{P,V}` and `V_{SH,V}` definitions.** §6.2. The two symbols appearing in
`embim625` / `embim626` are undefined on the page. The `SH` subscript may indicate a
horizontally-polarised shear velocity distinct from the `S,V` of `embim622`, which in a TI medium is
physically different — so this is not safely dismissable as a typo. Resolve from Backus (1962) or
from an IP module reference, not by inference.

**9.3 (HIGH) — Terzaghi weighting factor formula.** The page names the correction, describes what it
does, states it can become very large, and offers two ways to limit it — but never prints the factor
itself. Without it, weighted fracture density cannot be reproduced.

**9.4 (HIGH) — Luthi & Souhaite fracture-aperture equation.** Attribution, inputs (`Con1`, `Con2`,
`Rm`, `Rxo`) and outputs (Aperture mm, Diff Con mmho, Permeability mD, Porosity v/v) are all given;
the equation is not. `Con1`/`Con2` are described as mud-filtrate dependent but no values, ranges or
defaults are printed. Source is the 1990 *Geophysics* paper.

**9.5 (MEDIUM-HIGH) — Backus radical density.** §6.3. Whether `ρ` in `√(C/ρ)` and `√(D/ρ)` is `ρ_b`
or `ρ_Backus`. Affects the upscaled velocities directly.

**9.6 (MEDIUM-HIGH) — The inclinometry constraint equation.** The Inclinometry Calculator proves
DEVI, HAZI, P1AZ and RB are related by one equation, and Nav-QC computes all four from accelerometers
and magnetometers, but neither page prints the relation. Needed for §8.7 and for any independent
image orientation.

**9.7 (MEDIUM) — Is Dip Direction true-North or magnetic-North referenced at the point of the
mean-dip transform?** §4.1. Inferable from the declination handling but never stated. Worth pinning
down before implementing.

**9.8 (MEDIUM) — Alford rotation equations.** Named and central to both anisotropy methods; the
rotation equations themselves are not printed. Also unprinted: the exact form of the low-frequency-
preferring fit used to collapse the frequency-domain anisotropy map (distinct from the Tier-C
dispersion fit, though possibly related — if it is the same proprietary function, it is Tier C too
and should stay capability-only).

**9.9 (MEDIUM) — `EI_Kconst` semantics.** Described as "the instantaneous K value over the whole
well". Given §6.1 this is presumably the pointwise `Vs²/Vp²` curve, but "instantaneous … over the
whole well" is self-contradictory phrasing and is not confirmed.

**9.10 (MEDIUM) — Net2Gross sand-side convention.** §6.6. Which of the two contradictory statements
governs the `SAND`/`SHALE` flags.

**9.11 (LOW-MEDIUM) — Speed-correction parameter defaults.** All the depth-shift-from-accelerometer
window parameters are described qualitatively (§2.7) but **no default values are printed** for
Gravity Filter Window, Acceleration Zero Snap, either smoothing window, the calibration windows, or
the number of Cycles. Only the odd-sample constraint is given. Any implementation needs starting
values from elsewhere.

**9.12 (LOW-MEDIUM) — Image filter catalogue.** The Image Filter option is repeatedly described as
"a variety of filters" with the actual list carried only in screenshot `_iaclip00477.png`, which is a
dialog capture rather than an equation raster. Filter names and any kernel sizes are unrecorded.

**9.13 (LOW-MEDIUM) — Stuck Tool Calculator and Travel Time To Radius algorithms.** Both are named
in `image_toolkit.htm` with a one-line purpose and a screenshot; neither has any documented method,
parameter or equation. Travel Time To Radius in particular is a geometry conversion whose method
matters (fluid slowness, head radius, excluder handling all appear in `manageimagetools.htm` but are
not tied to this tool on the page).

**9.14 (LOW) — Semblance vs Nth-root output equivalence.** Nth root is offered as an alternative
correlation technique with N fixed at 4, but the Nth-root correlation expression itself is not
printed — only the semblance equation is. Cannot be reimplemented from this manual.

**9.15 (LOW) — Image Deconvolution `_LFI` reconstruction.** The high-frequency image is defined
(subtract the mean); the `_LFI` low-frequency curves are said to be "also written out" but their
definition (presumably the subtracted mean itself) is not stated.

**9.16 (LOW) — `AnisoEnergyMax` / `AnisoEnergyMin` / `AnisoDeltaMap` definitions.** Curve names are
listed; the Delta map is said to be "calculated as follows" with the detail beyond my read window
into the anisotropy output section. Worth a targeted re-read if anisotropy is implemented.

**9.17 (LOW) — `3d-viewer.htm` and `analysis-sticks.htm` are empty.** Both contain only the authoring
tool's placeholder text. If "analysis sticks" is a real IP feature, its documentation is elsewhere or
does not exist. `3d_viewer.htm` (with underscore) is the real, populated page; `3d-viewer.htm` (with
hyphen) is the stub — the near-identical names are a trap for anyone building a page index.
