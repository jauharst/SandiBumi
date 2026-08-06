# IP 2025 CHM ingest — Agent F: Log QC, curve editing, environmental corrections, depth/TVD

Source: decompiled Interactive Petrophysics 2025 help CHM.
Clean text `…\c25\<stem>_text.txt`; raster images `…\c25\<name>.png`; IP2018 counterpart `…\c18\`.
Provenance convention used throughout: `(stem.htm)` = vendor prose on that page; `[img-read: file.png]` = transcribed by reading the raster directly.
Vendor prose is copyrighted — this document paraphrases prose and captures only equations, constants, defaults and constraints, which are facts. No digitized chart lookup tables have been transcribed.

---

## 1. Scope & page inventory

36 pages, all read in full.

**Environmental corrections (2)**
`environmentalcorrections`, `easteuroperescorrections`

**Log QC / automatic editing (5)**
`log_qc`, `curve_autoedit`, `qc_edit`, `caliper-qc`, `navigation_qc`

**Curve editing & conditioning (14)**
`dataediting`, `interactive_curves_edit`, `interactivecurveedit`, `interactive_curves_splice`, `curve_filter`, `interactivefilter`, `curve_despike`, `curve_average`, `curve_rescale`, `curve-masks`, `cleandata`, `fill_data_gaps`, `restorebackupcurves`, `transformcurveunits`

**Arithmetic / integration (3)**
`differentiate`, `normalizearray`, `curveintegration`

**Depth shifting (7)**
`depthshifting`, `interactivedepthshift`, `interactive_bulk_depth_shift`, `interactiveblockdepthshift`, `sensorblockdepthshift`, `depth_shift_other_curves`, `stretchdepthshift`

**TVD / TVT / TST (4)**
`tvdcalculations`, `trueverticalstratigraphic`, `batch_loading_tvd_survey`, `multiple_well_batch_tvd`

**Shear sonic (1)**
`shearsonic`

Four of these pages (`curve_autoedit`, `log_qc`, `curve-masks`, `caliper-qc`) **do not exist in the IP2018 manual** — see §7.

**Rule 9 statement:** no smectite or montmorillonite endpoint, and no clay-mineral endpoint of any kind, appears anywhere on these 36 pages. Nothing to capture.

---

## 2. Equations & methods (with provenance)

### 2.1 Shear sonic creation — Greenberg-Castagna

Method: Greenberg-Castagna (1992) empirical relationships per mineral; defaults are the Greenberg-Castagna constants for 100 % brine-saturated rock; user-modifiable with a *Reset default coefficients* button (`shearsonic.htm`).

Functional form `Vs = a·Vp² + b·Vp + c`, with **all constants in km/s regardless of the input sonic unit** — the page states explicitly that the input curve may be metric or imperial but the equation constants are always km/s (`shearsonic.htm`).

Default coefficients [img-read: _rpclip0004.png] (verified at 5× crop):

| Mineral | a | b | c |
|---|---|---|---|
| Sandstone | 0 | 0.80416 | −0.85588 |
| Limestone | −0.05508 | 1.01677 | −1.03049 |
| Dolomite | 0 | 0.58321 | −0.07775 |
| Shale | 0 | 0.76969 | −0.86735 |

Mineral mixing — Voigt-Reuss-Hill (arithmetic/harmonic average pair, then their mean) [img-read: embim507.png]:

```
Vs = ½ { ( Σ_{i=1..4} X_i · Vs_i )  +  ( Σ_{i=1..4} X_i / Vs_i )^(−1) }
```
where `X_i` = volume of the i-th mineral, `Vs_i` = shear velocity of the i-th mineral (`shearsonic.htm`). The sum is over exactly four minerals — sandstone, limestone, dolomite, shale.

Poisson's ratio [img-read: embim508.png]:
```
ν = ( Vp² − 2·Vs² ) / ( 2 · ( Vp² − Vs² ) )
```

Young's modulus, output curve default name `E`, output in **GPa** (`shearsonic.htm`, ASCII):
```
E = 2.0 × G × (1 + Pr)
G = Rho × Vs²
```
`G` = shear modulus, `Pr` = Poisson ratio, `Rho` = bulk density, `Vs` = shear velocity.

Also output by the QC tab: `Vp`, `Vs`, `PoisRatio`, `Vp/Vs Ratio`, bulk modulus `KB`, shear modulus `Mu` (`shearsonic.htm`).

The QC crossplot's stated purpose is to confirm a recorded shear curve is genuinely shear and not a mud-wave or Stoneley arrival from mis-processed waveforms (`shearsonic.htm`). Crossplot Z-axis defaults to the gamma-ray curve.

**Default-mineral rule:** volumes that do not sum to 100 % are made up by the nominated default mineral (`shearsonic.htm`). Volumes accepted in percent or **decimals (decimals is the default)**.

### 2.2 TVT / TST (`trueverticalstratigraphic.htm`, ASCII — no raster)

```
TST = H ( cos(Hdev)·cos(Fdev) − sin(Hdev)·sin(Fdev)·cos(Hazi − Fazi) )
TVT = H ( cos(Hdev)         − sin(Hdev)·tan(Fdev)·cos(Hazi − Fazi) )
```
`H` = measured-depth thickness along the borehole; `Hdev` = borehole deviation from vertical; `Hazi` = borehole azimuth; `Fdev` = formation dip from horizontal; `Fazi` = formation dip azimuth.

Requires `NDIST`, `EDIST`, `TVD` curves to exist first (TVD module or loaded survey). Dips above the shallowest entered depth inherit that shallowest dip; dips below the deepest entered depth inherit that deepest dip. The **Pole Depth** is the MD anchor from which the calculation runs and to which TVT/TST are aligned to TVD; if blank it defaults to the well top depth, and the calculation differs above vs below it (`trueverticalstratigraphic.htm`).

### 2.3 TVD survey calculation (`tvdcalculations.htm`)

Four survey-calculation methods are offered: **Average Angle, Triangular Tangential, Radius of Curvature, Minimum Curvature**. **Minimum Curvature is the default** [img-read: _ccclip0201.png].

### 2.4 Filters (`curve_filter.htm`, `interactivefilter.htm`, `navigation_qc.htm`)

Filter types: square (box, unweighted), **bell (sine-shaped)**, median, user-defined weights.

Bell weights [img-read: embim4.png] — **cross-checked against the second, independent raster [img-read: _plclip00072.png]; both are identical (rule 4 satisfied)**:
```
Wt_j = [ 1 − cos( 2 · π · j / (FiltLen + 1) ) ] / (FiltLen + 1)
```

Filter length is specified in **samples**, and **even lengths are rounded up to make them odd** (`navigation_qc.htm`, `curve_filter.htm`).

### 2.5 Despike (`curve_despike.htm`, ASCII)

```
Input_psd = Input_mean + STD × SpikeCutoff
Input_msd = Input_mean − STD × SpikeCutoff
```
Mean and STD are computed over the moving *Data Filter width*; samples outside the ± band are treated as spikes.

### 2.6 Curve average — "Lateral" (pairwise-median / Hodges-Lehmann style) (`curve_average.htm`)

The page's own worked example, reproduced because it fixes the divisor convention: inputs `5, 8, 10, 3, 6` → all 10 pairwise sums → sorted `8, 9, 11, 11, 13, 13, 14, 15, 16, 18` → result `(13 + 13) / 4 = 6.5`. Note the divisor is **4**, i.e. the median pair-sum is halved twice (mean of the two central pair-sums, then halved to recover a level). Cited to J. L. Hodges and E. L. Lehmann, *Basic Concepts of Probability and Statistics*, Holden-Day, 1970 (`curve_average.htm`).

### 2.7 Curve AutoEdit (`curve_autoedit.htm`) — new module

A bank of multi-linear regressions predicts each target curve from every available combination of the other curves; the prediction from the **largest number of inputs (highest M)** that can be solved is used.

Base form and system (`curve_autoedit.htm`, ASCII):
```
y = a + b·x
A·x = b
```
solved as a least-squares SVD via **`rmatrixsolvels` from alglib.net** (`curve_autoedit.htm`).

Regression equation 5 [img-read: _declip0101.png]:
```
y_{j,i} = U·w_{0,i} + C_{j,k,i}·w_{1,i} + C_{j,k+1,i}·w_{2,i} + … + C_{j=N,k=m,i}·w_{N,i}
```

Combination count [img-read: _declip0098.png]:
```
m! / ( k! (m − k)! )
```

The module ships **32 predefined regression sets** [img-read: _declip0099.png, verified at 3×], arranged in blocks of 4 / 12 / 12 / 4; in each set the **last letter is the prediction target**.

Reference: Banas, R., McDonald, A., and Perkins, T. J., 2021, SPWLA 62nd Annual Logging Symposium (`curve_autoedit.htm`).

### 2.8 Log QC (`log_qc.htm`) — new module

Per-sample flag curve values:

| Value | Meaning |
|---|---|
| 1 | Good |
| 2 | Null / missing |
| 3 | Outside user limits |
| 4 | Outside extreme limits |
| 5 | Constant with depth |
| 6 | Nulled by badhole |
| 7 | Flagged badhole |

**Precedence (highest first):** Null/Missing → Nulled by Badhole → Flagged Badhole → Extreme → Outside User → Constant → Good (`log_qc.htm`). This ordering matters: a sample that is both constant-with-depth and in badhole reports as badhole, not constant.

Crossplot-consistency flags `XPND`, `XPSD`, `XPNS`, `XPGRPE` use a **different** convention: `1` = good, `0` = bad, `−999` = null (`log_qc.htm`). Two flag conventions coexist in one module — see §6.

Reference: McDonald, A. (2021), SPWLA 62nd Annual Logging Symposium (`log_qc.htm`).

### 2.9 Eastern European Resistivity Corrections (EERC) (`easteuroperescorrections.htm`)

The largest page in the assignment (≈46 kB text, 118 rasters). Corrects Normal and Lateral electrode-device logs and inverts for `Rt`, `Ri`, `Rxo`, `Di` by best-fitting measured apparent resistivities to Alpin two- and three-layer theoretical curve families. Origin: AGH University of Science and Technology, Kraków.

**Tool length convention** (`easteuroperescorrections.htm`):
- Lateral: `L = AB/2 + AM`
- Normal: `L = AM`

Apparent-resistivity forward model, Eq 1 [img-read: embim292.png]:
```
Ra = Rm [ 1 + (4L² / (π d)) ∫₀^∞ m · C_m(m) · sin(m · 2L/d) dm ]
```
Evaluated by **64-point Gauss quadrature to 0.5 % accuracy** (`easteuroperescorrections.htm`).

Bed temperature, Eq 3 [img-read: embim304.png]:
```
t_bed = t₀ + ( (H − 20) / 100 ) · G_g
```

Curve-family weighting, Eq 4 [img-read: embim335.png]:
```
w_i = 1 / ( 1 + sqrt( (Rs^theor/Rm − Rs^meas/Rm)² + (h^theor/d − h^meas/d)² ) )
```
Normalisation, Eq 5 [img-read: embim343.png]:
```
W_i = w_i / Σ_{i=1..N} w_i
```
Weighted true resistivity, Eq 6 [img-read: embim345.png]:
```
Rt/Rm = Σ_{i=1..N} W_i · [ R_a(5)^corr / Rm ]_i
```
Thin-bed misfit functional, Eq 7 [img-read: embim368.png]:
```
F = sqrt( Σ_{i=1..3} { (X_meas(i) − X_theor(i))² + (Y_meas(i) − Y_theor(i))² } )
```
with the X/Y measured pairs at [img-read: embim369.png … embim376.png] and the theoretical pairs at [img-read: embim377.png … embim384.png].

Invaded-zone resistivity, selected by which devices agree [img-read: embim351.png / embim353.png / embim352.png / embim354.png / embim356.png]:
```
Ri/Rm = ½ Σ_{i=2..3} R_a(i)^corr / Rm        (two-device agreement)
Ri/Rm = R_a(2)^corr / Rm                     (single device)
Ri/Rm = ¼ Σ_{i=2..5} R_a(i)^corr / Rm        (four-device agreement)
```

Invaded-vs-virgin discrimination test [img-read: embim357.png]:
```
( Rt/Rm − Ri/Rm ) / [ ( Rt/Rm + Ri/Rm ) / 2 ]  <  0.35
```

Annulus condition at [img-read: embim323.png]; lateral/normal functional forms at [img-read: embim360.png, embim361.png]. Supporting rasters read and consistent: embim289–291, embim297, embim302–303, embim310, embim312, embim347, embim365–366, embim390–391.

**EERC reference list, fully recovered** [img-read: _eercclip0007.png] — 10 entries, numbering matching the page's inline superscripts 1–10:
1. Abramowitz & Stegun (1970)
2. Alpin (1964)
3. Bała, Jarzyna & Cichy (1999)
4. Chapellier (1992)
5. Jarzyna, Bała & Cichy (1999), EAGE 61st Conference, paper P056, Helsinki
6. Dakhnov (1967)
7. Jarzyna et al. (2002), GeoWin®, EEGS, Aveiro
8. Ossowski (1990), CPBP 03.01.GF-1-3
9. Pierkov (1964)
10. Pirson (1963)

The IP2018 ingest recorded this list as unrecoverable. It is recoverable: the raster is byte-identical in both manuals (MD5 `fdf2247109cd6810c0d5f668c54a9fc7`) and simply had not been read.

### 2.10 Curve rescale (`curve_rescale.htm`)

Linear or logarithmic rescale of a curve between an input (left, right) pair and an output (left, right) pair. **The logarithmic option uses the natural logarithm** of input/output values — stated explicitly. Documented uses: fixing wrongly-scaled digitized logs, and converting count-rate neutron to neutron porosity.

### 2.11 Fill data gaps (`fill_data_gaps.htm`)

Gaps are runs of `−999`. Filling is a **linear extrapolation using the first sample either side of the gap** (i.e. linear interpolation across the gap). Maximum **100 curves** per run.

### 2.12 Curve integration (`curveintegration.htm`)

Restructured in 2025 into three tabs with **three calculation types**. Borehole-volume integration takes an X caliper and optionally a Y caliper. Sonic integration accepts µs/m or µs/ft input, outputs seconds or milliseconds. Each sample is divided by the well depth step before accumulation. Integration direction reverses by entering the deepest depth as the top depth. Optional **Pip curve** carrying values 1 or 2 according to Small Pip / Large Pip.

---

## 3. Correction-chart inventory (identity, applicability, ranges — no chart data)

`environmentalcorrections.htm` implements published vendor chart books. Chart *identities* and applicability are recorded below; **no digitized chart lookup values have been transcribed** (rule 5).

### 3.1 Vendor families implemented

| Vendor / family | Measurements corrected | Basis |
|---|---|---|
| Schlumberger | GR, density, neutron, resistivity (DLL / MSFL / MLL) | Schlumberger "Green Book" log interpretation chart book |
| Anadrill (LWD) | LWD tool suite | Anadrill chart book |
| Baker Atlas | wireline suite | Baker Atlas chart book |
| Baker Hughes INTEQ (LWD) | LWD suite | INTEQ chart book |
| Halliburton | wireline suite | Halliburton chart book |
| Sperry-Sun (LWD) | LWD suite | Sperry-Sun chart book |
| Weatherford / Reeves | wireline suite | Weatherford/Reeves chart book |
| PathFinder (LWD) | LWD suite | PathFinder chart book |
| **GE — new in IP2025** | **neutron only** | **"Allied Horizontal Formation Log Interpretation Charts", 1 October 2012, revised 15 October 2013** |

Each vendor appears as a tab; each tab lists the specific chart numbers applied per tool type. The per-vendor chart-number lists are **unchanged between IP2018 and IP2025** for all eight legacy vendors (verified by full prose diff — §7).

### 3.2 GE corrections — identity and applicability

- Input is **NPHI**; the module is **neutron-only** (`environmentalcorrections.htm`).
- Implemented as a **vendor-supplied software module**, not an IP re-digitisation.
- Correction procedure follows chart-book page **CNL-2**.
- Step 1 applies a partial correction only; the remaining correction is completed downstream.
- Relative Water Density is derived from pressure and temperature.
- Output limestone / sandstone / dolomite results are stated to be **equivalent to the vendor's NPHL / NPHS / NPHD**.
- A second tab, **"CNL Xplot"**, uses crossplot chart **XPL-6** together with the apparent-matrix-density **equation on chart-book page POR-7**.

### 3.3 Eastern European corrections — applicability envelope

Not a chart book; a theoretical-curve inversion. Applicability constraints (`easteuroperescorrections.htm`):
- Bed-thickness regime split at **h/d = 32**: `h/d ≥ 32` = thick bed, `h/d < 32` = thin bed. Different algorithms apply either side.
- Device lengths supported: **0.55 m, 1.05 m, 2.625 m**.
- `Dx0` is fixed at **90 % of 2·L_N1**.
- Theoretical-curve boundary values for `Rt/Rm`: **1000** above, **0.5** below.
- Shoulder resistivity `R_s` averaged over **0.5 h above the bed top**; `R_aav` averaged over the window `H_base − (H_top − 0.2 h)`.
- A theoretical family whose normalised weight is **< 0.25** is excluded from the solution.

---

## 4. Parameters, defaults & constraints

All values below are IP's shipped defaults as shown in the manual's own screenshots.

### 4.1 Log QC (`log_qc.htm`)

Curve Parameters — user limits [img-read: _declip0109.png]:

| Curve | Min | Max |
|---|---|---|
| GR | 59 | 168 |
| Density | 1.8 | 3 |
| Neutron porosity | −0.1 | 0.6 |
| DTC | 40 | 240 |

Extreme Limits [img-read: _declip0110.png]:

| Curve | Low | High |
|---|---|---|
| GR | 117 | 256 |
| Density | 1.5 | 3.5 |
| Neutron porosity | −0.2 | 1 |
| DTC | 40 | 240 |

Badhole [img-read: _declip0111.png]: Caliper **6 – 16**; **DRHO −0.1 – +0.1**; all three badhole flags enabled by default.

Nulling Data tab, zone 1 [img-read: _declip0112.png]:
- Null Data Outside Limits — **on**
- Badhole Null **Gamma** — **off**
- Badhole Null **Density** — **on**
- Badhole Null **Neutron** — **on**
- Badhole Null **PE** — **on**

That asymmetry is deliberate and worth preserving: GR survives badhole, the nuclear porosity/lithology measurements do not.

### 4.2 Curve AutoEdit (`curve_autoedit.htm`)

- Fix window: **1 – 100 ft**
- Smooth window: **1 – 100 ft**, must be **odd**; an even entry is **rounded up by 1**
- Empirical guidance printed by the vendor: "the value of 5 ft for this parameter seems to work the best in practice"
- Logic flags: `0` / `1`
- **Bad-hole flag input format: null (`−999`) = invalid, `1` = valid** — the inverse of the Log QC flag scheme (see §6)

### 4.3 Curve Despike [img-read: _declip0095.png]

- Data Filter width: **10 (FT)**
- Spike cutoff: **2 (standard deviations)**

### 4.4 Auto Depth Shift [img-read: _declip0092.png]

- Correlation window height: **30 ft**
- Maximum shift: **20 ft**
- Maximum difference between shifts: **5 ft**
- Remove shifts with correlation **R² < 0.2**
- Delete all shifts in track: **checked**
- Minimum distance from original shifts: **5 ft**
- Correlation Type: **Both** (vs Positive / Negative)

### 4.5 Depth-shift unit convention (`interactive_bulk_depth_shift.htm`)

**Shift Increment is expressed in well steps, not depth units.** The page's own example: a Shift Increment of 5 moves the data 2.5 ft when the well step is 0.5. **Positive shifts move the curve down; negative moves it up.** Shifts are stored in the Curve Header `Shift Inc` column and are reverted by zeroing that column. Only one bulk shift per curve.

### 4.6 Fill Data Gaps [img-read: _declip0042.png] and (`fill_data_gaps.htm`)

- Fill gap maximum width: **5**, unit radio default **In Samples** (alternative: In FT)
- Output mode default: **Overwrite Curve** (alternatives: Create backup → `bu` suffix; New curve → `1` suffix)
- Max **100** curves per run

### 4.7 Clean Curve Data [img-read: _plclip00173.png] and (`cleandata.htm`)

- Curve selection mask: `*`
- Set values **above 1.0E08 → −999.00** (on)
- Set values **below −1.0E08 → −999.00** (on)
- Set values **equal to 0.0 → −999.00** (off)
- Set values **equal to −999.25 → −999.00** (on)
- Fill data gaps, maximum gap length **3 ft** (on)

Null values recognised by Clean Data: **−999, −999.25, −9999, −99**; **IP's canonical null is −999** (`cleandata.htm`).

### 4.8 TVD calculation [img-read: _ccclip0201.png, _ccclip0110.png]

- Survey method: **Minimum Curvature** (default)
- Magnetic deviation: **0**
- Reference depth / TVD / East / North: **0 / 0 / 0 / 0**
- Reference mode: **"Offset distances from well surface position"** (default)
- **TVDSS positive below MSL** by default
- New in 2025: `EDIST` / `NDIST` may be **Absolute Positions in the well's Grid System** or **Relative Positions** to the surface location; the East/North Distance Type populates the Curve Reference field on the Well Header Position tab (`tvdcalculations.htm`)

### 4.9 Environmental corrections — Schlumberger defaults

GR [img-read: _encclip0001.png]: **Eccentered**, **Non Barite**, **Open Hole**; Tool Diameter **3.625 in**; Casing Material Density **66.81 lbs/gal**; Cement Density **16. lbs/gal**; No. Levels **3**.

Density [img-read: _encclip0002.png]: tool **LDT**; No. Levels **3**.

Neutron (CNL) [img-read: _encclip0003.png]: temperature unit **Deg C**; borehole salinity **2.8E-4 Kppm**; formation salinity **2.8E-4 Kppm**; compressibility multiplier **4**; standoff **0 in**; input and output matrix **Limestone**; tool **CNT-A**; *Calculate pressure from depth* **checked**.

Resistivity DLL / MSFL / MLL [img-read: _encclip0004.png]: Filter Levels **1 / 1 / 5**; tool **DLT-B**; **"Eccentered (1.5" SO)"**; **MSFL (Regular)**; temperature unit **Deg C**.

### 4.10 Environmental corrections — GE defaults [img-read: _encclip0034.png]

- Tool temperature: **68 deg F**
- **Decentralized** checked, Stand off **0 in**
- Borehole salinity **0 kppm**, formation salinity **0 kppm**
- Casing OD **5.5 in**, casing thickness **0.3 in**, cement thickness **1.1875 in**

### 4.11 Filter length limits

- Interactive Filter: **1 – 121**
- Curve Filter: **3 – 121**, and **2001** appears as a limit on `dataediting.htm`

(These do not agree — see §6.)

### 4.12 Backup-curve naming conventions (`restorebackupcurves.htm`)

| Operation | Backup suffix / type code |
|---|---|
| Interactive edit | `bu`, `b1`, … |
| Filter | `df` |
| Interactive filter | `bu`, `b1`, … |
| Non-linear depth shift | `v1`, `v2` |

Restore masks: `*` = all backups; `********BF` = all filter backups.

### 4.13 Navigation QC (`navigation_qc.htm`)

- Output curves: `GXn/GYn/GZn`, `HXn/HYn/HZn`, `GQ`, `HQ`, `DEVIC`, `HAZIC`, `P1AZC`, `RBC`, `P1NOC`
- **`GQ` and `HQ` are always 1** because the inputs are normalised — so they are a normalisation check, not a field-strength measurement
- `P1NOC` is output **for Schlumberger tools only**
- Fitting method: **circle fit** or **3D fit**
- Magnetic declination and inclination are read from Well Header → Position; declination is applied, inclination is used only as a QC overlay (`MINC` vs `MINCTMP`)
- Frame Time and Magnetic Field Intensity are **read but not used by any current model**
- Output filters are simple symmetric filters, lengths in samples, even rounded up to odd

---

## 5. Assumptions & validity limits

1. **TVT/TST breaks down at high dip.** The vendor warns that unrealistic TVT/TST results occur when borehole deviation and formation dip are both high (**> 45°**), such that the well penetrates the base of the formation before the top (`trueverticalstratigraphic.htm`). SandiBumi should refuse or flag rather than silently emit these.
2. **Shear-sonic constants are unit-locked to km/s.** Any implementation that carries the user's display unit into the correlation is silently wrong (`shearsonic.htm`).
3. **Greenberg-Castagna defaults assume 100 % brine saturation** (`shearsonic.htm`). Using them on hydrocarbon-bearing intervals without fluid substitution is outside the stated basis.
4. **Voigt-Reuss-Hill is a four-mineral mixture only** (sandstone, limestone, dolomite, shale) with a nominated default mineral absorbing the closure error (`shearsonic.htm`).
5. **EERC assumes electrode-device geometry** (Normal/Lateral) and the tabulated device lengths; the thick/thin split at `h/d = 32` selects fundamentally different solution paths (`easteuroperescorrections.htm`).
6. **EERC forward model is a numerical integral** — 64-point Gauss quadrature at 0.5 % stated accuracy. That 0.5 % is a floor on achievable inversion accuracy.
7. **GE corrections are neutron-only and vendor-supplied**; there is no GE density, GR or resistivity path (`environmentalcorrections.htm`).
8. **Environmental correction requires the correct tool string.** Every vendor tab is keyed to a specific tool (CNT-A, LDT, DLT-B …); applying the wrong tool's chart silently produces a plausible wrong number.
9. **Log QC user limits shipped as defaults are not physical limits** — the shipped GR range 59–168 is a data-specific example, not a validity envelope. Shipping them as SandiBumi defaults would flag most real logs.
10. **Fill Data Gaps interpolates linearly from the two bounding samples only** — no trend, no shape preservation (`fill_data_gaps.htm`).
11. **Curve Rescale logarithmic mode uses natural log**, not log10 (`curve_rescale.htm`).
12. **Depth shifts are metadata, not resampling.** They live in the Curve Header as an integer step count and are reversible by zeroing (`interactive_bulk_depth_shift.htm`).

---

## 6. Internal discrepancies

1. **Log QC extreme-low GR (117) exceeds user-min GR (59)** [img-read: _declip0109.png vs _declip0110.png]. By the module's stated semantics the extreme band should bracket the user band. The page's own narrative example also discusses flagging GR *below 0*, which 117 cannot express. One of the two shipped panels is wrong; the extreme table is the likelier culprit.
2. **Two contradictory flag polarities inside the same QC/edit family.** Log QC flag curves use `1 = good … 7 = flagged badhole` and crossplot flags use `1 = good, 0 = bad, −999 = null`, while Curve AutoEdit consumes a bad-hole flag as `−999 = invalid, 1 = valid` (`log_qc.htm`, `curve_autoedit.htm`). Three conventions, one workflow.
3. **Fill Data Gaps contradicts itself in a single paragraph**: gaps are filled when they are "less than the maximum data gap", and then "There is no limit to the size of the gap that can be filled" (`fill_data_gaps.htm`). The dialog does expose a maximum-width control with a default of 5 samples [img-read: _declip0042.png], so the second sentence appears to be stale text.
4. **Filter length limits disagree**: 1–121 (`interactivefilter.htm`) vs 3–121 (`curve_filter.htm`) vs 2001 (`dataediting.htm`).
5. **EERC Eq 3 divides by 100** [img-read: embim304.png] while the accompanying ASCII defines the gradient `G_g` in **°C/m** (`easteuroperescorrections.htm`). Those are inconsistent unless `G_g` is really °C/100 m. **Unresolved — see §9.**
6. **EERC X/Y measured pair (4) is identical to pair (2)** [img-read: embim372.png vs embim370.png], and the "theoretical" rasters embim377–384 are visually identical to the "measured" rasters embim369–376. Either the manual reuses the wrong images or the distinction is carried only by context. **Unresolved — see §9.**
7. **embim361 uses `/` where embim360 uses `;`** in otherwise parallel lateral/normal functional forms.
8. **Differentiate is language-dependent** (`differentiate.htm`): the Fortran, C++, VB and C# variants compute a *ratio* while the MATLAB, IronPython and Python variants compute a true finite difference. Two different operators under one module name.
9. **C# example contains an assignment-for-comparison bug**: `if (Curve(index-1) = 0.0)` (`differentiate.htm`).
10. **Normalize Array C++ example increments the wrong index** — the `IY` loop increments `IX` (`normalizearray.htm`).
11. **Restore Backup filter type is `df`, but the filter-backup mask is `********BF`** (`restorebackupcurves.htm`) — the mask does not match the documented suffix.
12. **Sperry-Sun chart-book year cited as 1998 in one place and 1996 in another** (`environmentalcorrections.htm`).
13. **Baker Atlas is listed with a 1984 chart book while the page also states that book was never received as a chart book** (`environmentalcorrections.htm`).
14. **Salinity default inconsistency across vendors**: Schlumberger CNL ships **2.8E-4 Kppm** [img-read: _encclip0003.png] while GE ships **0 kppm** [img-read: _encclip0034.png]. Both are effectively fresh water, but 2.8E-4 Kppm = 0.28 ppm is a nonsense number that looks like a unit-conversion artefact.
15. **`_encclip0022.png` is a stale "IP 2018" screenshot** still shipped in the 2025 manual.

---

## 7. IP2018 → IP2025 diff

Method: full cleaned prose diff (`difflib.SequenceMatcher` opcode walk, image markers/bullets/pipes/U+FFFD stripped) over all pages present in both manuals, plus MD5 comparison of equation rasters.

### 7.1 Pages new in IP2025 (absent from IP2018)

- **`curve_autoedit`** — entirely new module (2021 SPWLA reference)
- **`log_qc`** — entirely new module (2021 SPWLA reference)
- **`curve-masks`** — new
- **`caliper-qc`** — new

### 7.2 Correction charts added / removed

- **Added: GE Corrections** — the only correction family added 2018→2025. Confirmed absent from IP2018 (`grep -c` on the 2018 page returns 0).
- **Removed: none.**
- **Chart numbers for all eight legacy vendors are unchanged**, tab by tab.

### 7.3 Numeric diffs

Systematic numeric-token diff across all 32 shared pages. Result: **no petrophysical constant, default or limit changed anywhere.** The only new numeric tokens are:

| Token | Origin |
|---|---|
| 2012, 2013 | GE chart-book dates |
| 3, 4 | "three tabs", "three calculation types" in restructured `curveintegration` |
| 360 | new modulo-360 extrapolation note |

Three apparent 2018-only numbers (`0`, `2`, `105`) were **false positives** — they come from the c18 extractor's `# equation_images: N` header line, not page content. Verified and excluded.

### 7.4 Equation stability

**Greenberg-Castagna coefficients are unchanged.** Proven by MD5-matching `_rpclip0004.png` across c18 and c25 (`c4dbb89270a69948f94eff88ad1f753e`). This is stronger evidence than a text diff, because those coefficients exist only inside the raster.

Bell filter, TVT/TST, despike, Poisson, Young's modulus, Voigt-Reuss-Hill and all EERC equations: unchanged.

### 7.5 Non-numeric changes worth recording

- Vendor rename throughout: **Senergy Ltd. → Lloyd's Register Digital Products Limited** (including the disclaimer sentence).
- **Shear Sonic QC/Create moved menus**: Advanced Interpretation → Rock Physics (2018) becomes **GeoEng → Rock Physics** (2025).
- **`tvdcalculations` gained the Absolute vs Relative EDIST/NDIST distinction** and the Well Header Position Curve Reference linkage.
- **`curveintegration` restructured** from a single dialog into three tabs / three calculation types.
- **`interactive_curves_edit` and `curve_filter` clarified backup semantics** (Create Backup off → new `…1` curve, original untouched; Create Backup on → `…bu` copy, edits applied to the original). Behaviour is the same as 2018; only the wording is clearer.
- Typo fix `nomogram's` → `nomograms`.

### 7.6 Correction to the IP2018 ingest record

`ip2018_chm_ingest/F_envcorr_tierc_citations.md` §F1.6 states the EERC reference list "is not recoverable — do not complete these from memory." **It is recoverable** — see §2.9. The 2018 record should be amended; the list was never missing, only never read.

---

## 8. SandiBumi notes

**Adopt directly (method is public, constants are published literature):**
- Greenberg-Castagna Vs correlation and its four default coefficient sets — a published 1992 correlation, and the manual's values match the published form. Store the km/s unit lock as a hard invariant, not a comment.
- Voigt-Reuss-Hill mixing, Poisson's ratio, Young's modulus — standard elasticity.
- TVT/TST equations — standard geometry; adopt with the >45° guard.
- Minimum Curvature (and the other three survey methods) — standard directional-survey mathematics.
- Bell/box/median filter forms; despike as mean ± k·σ; Hodges-Lehmann pairwise-median averaging (cite the 1970 text).
- Log QC flag scheme and its precedence order — the *scheme* is a design, not vendor data, and it is a good one. Reimplement the precedence exactly; the ordering is the part that is easy to get subtly wrong.

**Do not copy:**
- Any vendor chart-book digitisation (Schlumberger, Halliburton, Baker, Weatherford, Sperry-Sun, PathFinder, Anadrill, GE). These are licensed vendor data. SandiBumi must either license them, obtain vendor-supplied modules as IP did for GE, or expose the correction as a user-supplied table.
- The EERC theoretical curve families (Alpin) — the equations are citable literature, the digitized curve families are AGH work product.

**Design cautions drawn from IP's own inconsistencies:**
- Pick **one** flag polarity across the entire QC/edit chain. IP carries three and it is a standing source of user error (§6.2).
- Do not ship data-specific limits as defaults. IP's GR 59–168 is a screenshot artefact that became a default (§6.1).
- Canonicalise nulls at ingest. IP recognises −999, −999.25, −9999, −99 and normalises to −999; SandiBumi's LAS ingest must do the equivalent explicitly, because −999.25 is the LAS-standard null and will otherwise survive as a real value.
- Store depth shifts as reversible header metadata (step counts), not by resampling the curve. IP's `Shift Inc` design is right and cheap to reproduce.
- **Shift-increment units are steps, not depth.** If SandiBumi ever imports an IP shift value, this is a silent-wrongness trap: a shift of 5 is 2.5 ft at 0.5 ft sampling and 0.5 ft at 0.1 ft sampling.

**Cross-agent notes:**
- The Greenberg-Castagna coefficients here are the same constants any rock-physics or geomechanics agent will encounter; this table is the verified copy (raster-verified, and MD5-proven unchanged 2018→2025).
- The Log QC badhole gates (Caliper 6–16, **DRHO ±0.1**) overlap the project's existing log-QC gate memory. DRHO ±0.1 g/cc agrees with the established house gate.
- The `−999` / `−999.25` null handling belongs to whichever agent covers data I/O and loaders.

---

## 9. OPEN ITEMS

1. **EERC Eq 3 gradient units.** The raster [img-read: embim304.png] divides by 100 while the ASCII labels `G_g` as °C/m (`easteuroperescorrections.htm`). Cannot be resolved from the manual. Do not implement a geothermal gradient from this page without independent confirmation.
2. **EERC X/Y measured vs theoretical rasters.** embim377–384 ("theoretical") are visually identical to embim369–376 ("measured"), and pair (4) duplicates pair (2). Either the manual reuses images or the pairs are context-dependent. The Eq 7 misfit functional itself is unambiguous; the individual X/Y definitions are not. **Not transcribed as definitive.**
3. **Which of the two Log QC GR limit panels is authoritative** (extreme-low 117 vs user-min 59). Not determinable from the manual.
4. **Schlumberger CNL salinity default `2.8E-4 Kppm`.** Transcribed exactly as displayed [img-read: _encclip0003.png]; verified at 5× upscale. The value is physically odd (0.28 ppm). Reported as-is per rule 3 — not "corrected" to a plausible textbook salinity.
5. **Filter maximum length** — 121 vs 2001. Which applies to which module is not stated consistently.
6. **AutoEdit's 32 regression sets**: the block structure (4/12/12/4) and the last-letter-is-target rule are confirmed [img-read: _declip0099.png at 3×], but the full curve-mnemonic membership of each set was not transcribed — it is a vendor-designed lookup, and reading it in full risks transcription error at that resolution.
7. **`differentiate` intended semantics.** Two different operators ship under one name (ratio vs finite difference) depending on scripting language. Which is the intended module behaviour is not stated.
8. **GE step-1 "partial correction".** The page says step 1 applies only part of the correction, with the remainder completed downstream, but does not specify the split. Vendor-module internals; not documented.

---

*Rule 9 restated for the record: no smectite or montmorillonite endpoints appear on any of the 36 assigned pages.*
