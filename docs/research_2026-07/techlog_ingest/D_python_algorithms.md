# Techlog 2018.2 (r22885) — PythonScripts Algorithm Ingest (D)

**Source (read-only):** `D:\01. Work\00. Guidebook\03. Guidebooks Techlog\Techlog 2018.2 (r22885)\PythonScripts\`
**Purpose:** IP-careful *understanding* extraction — identify WHICH published equation each shipped method implements, its exact constants, inputs/outputs, and vendor branching, so SandiBumi can re-implement independently from the published source. No code was copied; formulas below are transcribed as algebra with citations.
**Date:** 2026-07-22

> **Legal note.** Everything here is method identification + numeric constants read from Schlumberger-shipped Python. The high-value physics kernels (EnvCorr chart engine, RockPhysics_EquationsLibrary, Raformula, RST) are shipped **compiled (.pyc / .dll)** and were NOT decompiled. Where a method is compiled, it is identified from the readable wrapper, its call signature, docstring citation, and default parameters — which is exactly what is needed to reimplement from the published literature.

---

## 1. Tree map — packages and file counts

Root contains ~370 loose top-level `.py` (utilities, importers, geomechanics, one-off petrophysics) plus 7 sub-packages. `.pyc`/`__pycache__` skipped throughout.

| Package | `.py` (src) | What it is | Math readable? |
|---|---|---|---|
| **EnvCorr/** | 71 (5 top + 65 in `Tools/` + `__init__`) | Environmental/borehole correction **data-prep descriptors** + wizard front-end. | **No** — correction math in compiled `EnvCorrPreProcessingPrivate.pyc` + C++ wizard. Descriptors readable. |
| **RockPhy/** (folder) + ~40 top-level `RockPhy_*.py` | 20 in `TechlogRockPhysics/` + ~40 loose | Rock-physics model wrappers + shared equation library. | **Partly** — `RockPhyEquations.py` fully readable; `RockPhysics_EquationsLibrary.pyc`, `Raformula.pyc`, `Library_*.pyc`, `RPI_*.pyc` compiled. Wrappers + docstrings readable. |
| **Acoustics/** | 54 (top + 40 in `Tools/`) | Sonic tool data-prep descriptors + waveform utilities + anisotropy. | See §4. |
| **Preprocessing/** | 4 (`Preprocessing.py`, `Tool.py`, `Tools/Example.py`, `__init__`) | Base **framework** for the working-set prep pattern (EnvCorr & Acoustics inherit it). | Readable — it's a data-marshalling framework, no petro math. |
| **Techlog/** | 5 (`Data`, `Engine`, `Plot`, `Utils`, `__init__`) | The scripting-API Python package. | Readable — API surface, §6. Backed by compiled C-ext modules `TechlogDatabase`, `TechlogMath`, `TechlogStat`, `TechlogPlot`, `TechlogPlatform`. |
| **PPP/** | 5 (`Formation_Temperature_Models.py` + `Anisomecpro/` 4) | Pore-pressure / geomech helpers. | Readable. |
| **RST/** | 0 `.py` here | Reservoir Saturation Tool. | **Dead end** — `RST/lib_x64/RSTCOT_x64.dll` + top-level `RSTUtilFunctions.pyc` (150 KB, compiled). Top-level `RST*.py` are thin DLL callers. |
| **pyExcelerator/** | 20 | 3rd-party xls writer (public OSS). | N/A — not petrophysics. |

**Also notable at top level (readable, real petro math, outside the 5 priority packages — flagged for SandiBumi):** `Archie.py`, `VSH_GRequation.py`/`VSH_GRlibrary.py`, `PorosityAndLithologyComputation.py` (120 KB), `ComputeLithology.py` (54 KB), `TempCorr_Resistivity.py`, `TOC_Computation.py` (34 KB, Passey ΔlogR / Schmoker), `D-Exponent.py`, `QVn.py`, `BoreholeComputation.py`, plus the geomech `UCS_*.py`, `YME_*.py`, `FG_*.py`, `GME_*.py` families.

---

## 2. EnvCorr (environmental / borehole corrections) — HIGHEST PRIORITY

### 2.1 Architecture — how EnvCorr actually works (important for SandiBumi)

EnvCorr is **two-stage** and the second stage is compiled:

1. **Data-prep stage (readable).** Each `EnvCorr/Tools/<TOOL>.py` is a tiny class subclassing `EnvCorrTool` (which subclasses `Preprocessing.Tool`). It declares only:
   - `getName()` — service label;
   - `getVariableToKeepList()` — regex list of **input mnemonics** the tool needs;
   - `getOptionalVariableList()`;
   - `samplingRate` — target resample step (e.g. GR 0.5 ft, AIT 0.25 ft, density STANDARD 0.5 ft) and interpolation = linear;
   - `varToRename` — mnemonic normalization (e.g. `{'A.?IFC':'AIFC'}`);
   - `toolRestriction` — matches `TOOL-NAME` property (e.g. `['ADN']`);
   - `getModule()` — `Wireline` / `LWD` / `PathFinder`;
   - `getAdditionalPropertyToCopy()` — calibration properties to carry (e.g. `ADN/SSW1_BG_MAS/CALI_MEAS`).
   Running one builds a working-set dataset `WFW_<name>_<ServiceRun>_<Resolution>`. **No correction arithmetic here.**

2. **Correction stage (compiled).** The wizard imports `EnvCorr.EnvCorrPreProcessingPrivate` (`EnvCorrPreProcessingPrivate.pyc`) — `class EnvCorrPreProcessing` + `toolMap` dict mapping the ~60 UI method names to tool classes. The actual **chart lookups / correction polynomials live in this .pyc and in the Techlog C++ engine.** Not decompiled.

**Takeaway for SandiBumi:** the Python gives you the *taxonomy* (which tools, which curves, which resolution, WL vs LWD) but NOT the chart coefficients. To reimplement corrections you go to the published chartbooks (Schlumberger *Log Interpretation Charts*), which is the intended independent path.

### 2.2 Borehole-parameter front-end — `BoreholeEnvCorr.py` (top level, readable)

Collects/derives the borehole environment variables consumed by the correction wizard. Outputs family-tagged curves: `WTEM` (borehole temp), `WPRE` (pressure), `RM/RMF/RMC` (mud/filtrate/cake resistivity), `RW`, `BSAL/SALMF/SALW/FSAL` (salinities), `BS` (bit size), `IHV` (hole volume), casing geometry.

Real sub-computations it wires in (from `BoreholeComputationLib.pyc`, compiled, but **identified by import + docstring**):
- **`RwGen6(TempC, PressKPa, SalinityPPK)`** → Rw of a NaCl brine. Docstring: *"algorithm used in ELAN, developed by Jack LaVigne, presented in the Schlumberger Charts Book (2009) as **Chart Gen-6**."* Valid 0–260 °C.
- **`SalinityGen6(Rw, TempC, PressKPa)`** → inverse (salinity from Rw). Solves the Chart Gen-6 function.
- **`APSmudHI(...)`** → APS mud hydrogen-index helper.
- Temperature option: linear geotherm `T(TVD) = BTEMP_surface + gradient·(TVD − refTVD)`, default gradient 3 °C/100 m.
- Pressure option: from mud weight × TVD, default MW 1.1 g/cm³, air gap 2 m.

Parameter defaults captured: Rm 0.1 Ω·m @20 °C, Rmf 0.08, Rmc 0.16, Rw 0.1 @100 °C, bit size 8.5 in, casing OD 7 in / 29 lbm/ft. Salinity units ppk, resistivities Ω·m.

### 2.3 Resistivity temperature correction — `TempCorr_Resistivity.py` (top level, **fully readable**)

Two published forms, temperatures in **°F**, `tref` default 200 °F. Both are the standard "resistivity varies inversely with (T + constant)" law:

- **Arps** (`Method='Arps'`, default): `Rt(tref) = R · (T − 6) / (tref − 6)`
- **Exxon** (`Method='Exxon'`): `Rt(tref) = R · (T + 6.77) / (tref + 6.77)`

(Arps 1953; the "Exxon" variant is the same rational form with a different additive constant — note in °C the classic Arps constant is 21.5, which is 6.77 in a different convention. As-shipped uses the °F constants above.) Guards `T>0 and R>0` else MissingValue.
Inputs: `RESISTIVITY` (deep, Ω·m), `TEMPERATURE` (°F). Output: `RT_COMP`.

### 2.4 EnvCorr tool taxonomy (65 `Tools/*.py`) — the reusable checklist

Full vendor/tool coverage present (each = a curve-prep descriptor):

- **SLB Wireline density/PEF:** `HLDS_STANDARD/HIRES`, `HRDD`, `LDS_STANDARD/HIRES`, `SLDT_STANDARD/HIRES` (Litho-Density & Hostile-Environment LDS), `HGNS_STANDARD/HIRES`.
- **SLB Wireline neutron:** `CNT_STANDARD/HIRES` (CNL), `APS_STANDARD/HIRES` (Accelerator Porosity Sonde).
- **SLB Wireline GR/spectroscopy:** `GR` (SGT; keeps `RGR|GR`, opt `STOF`), `HNGS`, `NGD`.
- **SLB Wireline resistivity:** `AIT`, `AITRT` (resistivity transform), `ZAIT`, `HRLT` (HRLA), `HALS`, `HALS_IP_DIP`, `HALS_IP_HIGH`, `ALAT`, `MCFL` (micro-resistivity).
- **LWD density-neutron (adnVISION):** `ADN_DENSITY_STANDARD/HIRES`, `ADN_DENCAL`, `ADN_BIPNCAL`, `ADN_IDD` (image-derived density), `ADN_UCAL` (ultrasonic caliper), `ADN_NEUTRON8/46`, `ADN_AZNEUTRON46`.
- **LWD EcoScope/NeoScope:** `ECOSCOPE_DENSITY_STANDARD/HIRES`, `ECOSCOPE_DENCAL`, `ECOSCOPE_IDD`, `ECOSCOPE_UCAL`, `ECOSCOPE_NEUTRON`, `ECOSCOPE_AZNEUTRON`, `ECOSCOPE_GR`/`ECOSCOPE_GR_API`, `ECOSCOPE_SIGMA`, `ECOSCOPE_SPL`, `ECOSCOPE_SPST`, `NGD` (SNGD neutron-gamma density).
- **LWD resistivity/GR (arc/CDR/geoVISION/PeriScope/ImPulse/PathFinder):** `ARC_WIZARD`, `ARC_WIZARD_IMPULSE`, `ARC_WIZARD_LITE`, `CDR_GR/RES/DIE`, `GVR_GR/INCL/INV`, `AVR_GR_API/CPS`, `IMP_GR_API/CPS`, `PER_GR_API/CPS`, `DSGR_GR_API`, `PF_GR`, `PF_NEUTRON`.
- **Cased-hole:** `RSTIC`, `RSTSIG` (RST inelastic-capture / sigma prep; correction in RST DLL).

Each maps to a UI method string in the `UserTool` dropdown (full list preserved in `EnvCorrTool.py` / `BoreholeEnvCorr.py`). For SandiBumi this is the authoritative "which tools need env-correction and at what resolution" table.

---

## 3. RockPhy (rock physics) — PRIORITY 2

### 3.1 Architecture

Two library tiers under `RockPhy/TechlogRockPhysics/`:
- **`RockPhyEquations.py`** — **fully readable** (the "geophysics" tier: fluid mixing, Gardner, Faust, Batzle-Wang, elastic conversions). See 3.2.
- **`RockPhysics_EquationsLibrary.pyc`, `Raformula.pyc`, `Library_*.pyc`, `RPI_*.pyc`** — **compiled** kernels (Hertz-Mindlin, Gassmann, HS bounds, contact cement, DEM, Ciz-Shapiro, Walton, Berryman SC, Xu-White, patchy sat, carbonate, CO2). Identified from wrappers + docstrings (3.3).

Each `RockPhy_<Model>.py` at top level is a thin per-sample loop driver that calls the library. Wrappers expose the **published method name, call signature, default constants, and citation** — enough to reimplement from Mavko et al. *The Rock Physics Handbook*.

### 3.2 `RockPhyEquations.py` — fully readable formulas + constants

**`elastic(K,G,ρ)` / `elastic2(Vp,Vs,ρ)`** — isotropic moduli↔velocity:
- ν = (3K−2G)/(2(3K+G)); M = K+4G/3; AI=√(Mρ); SI=√(Gρ); Vp=√(M/ρ); Vs=√(G/ρ); E=9KG/(3K+G).
- Inverse: G=ρVs², K=ρ(Vp²−4/3 Vs²).

**Fluid / modulus mixing:**
- **Reuss** (isostress, homogeneous sat): `1/Kf = Sw/Kw + Sg/Kg + So/Ko`.
- **Voigt** (isostrain, patchy): `Kf = Sw·Kw + Sg·Kg + So·Ko`.
- **Brie** (Brie et al. 1995), default exponent e=3: `Kbrie = (1−Sg)^e·(Kliq − Kg) + Kg`, with liquid `Kliq` a Reuss mix of water+oil.
- **Hill average**: `(Kvoigt + Kreuss)/2` over a mineral/volume list.

**Gardner (1974) density↔velocity**, per-lithology `ρ = a·Vp^b` then volumetrically blended. **Default coefficients (a,b):** sandstone (1.66, 0.261), calcite/limestone (1.359, 0.386), dolomite (1.74, 0.252), anhydrite (2.19, 0.160), shale (1.75, 0.265). Inverse `gardnerinv` provided. (Vp in the tool's working velocity units; classic Gardner ρ=0.23V^0.25 is the ft/s form — these are the per-mineral recalibrated set.)

**Faust (1953) velocity from resistivity+depth:** `Vp = 1e6 / [ factor·((depth_ft)·R)^exponent ]`, **defaults factor=1945, exponent=0.1667 (1/6)**. depth converted m→ft internally (/0.3048). Used by `RockPhy_DTC_Reconstruction_FAUST.py` (auto-calibrates factor/exponent if a reference DT is supplied).

**Batzle-Wang (1992) fluid properties — `batzle(method,sal,og,gg,gor,giib,giio,p,t)`** — COMPLETE readable implementation. Returns Vp/ρ/K for brine, oil, gas. Constants exactly as published:
- Gas: pseudo-reduced `Pr=P/(4.892−0.4048·gg)`, `Tr=(T+273.15)/(94.72+170.75·gg)`; density `ρg=28.8·gg·P/(z·R·(T+273.15))`, R=8.31441; z from the E/gamma polynomial; `Kg=P·γ/(1−Pr/z·f)/1000`.
- Oil: dead-oil `ρ0=141.5/(og+131.5)`; live-oil `B0=0.972+0.00038·(2.4·gor·√(gg/ρ0)+T+17.8)^1.175`; velocity `Vpo=2096·√(ρ'/(2.6−ρ'))−3.7T+4.64P+0.0115·(4.12·√(1.08/ρ'−1)−1)·T·P`; `Ko=Vpo²·ρo`.
- Brine: `ρw=1+1e-6·(−80T−3.3T²+0.00175T³+489P−2TP+0.016T²P−1.3e-5T³P−0.333P²−0.002TP²)`; `ρb=ρw+sal·(0.668+0.44·sal+…)`; velocity from the 4×5 Wilson coefficient matrix `matrixw` (all 20 constants present) + salinity/pressure correction; gas-water ratio `gwr` correction. Method flag: `1`=gas-index-in-oil, else GOR (l/l). Salinity input ppm (÷1e6 internally), P in MPa, T in °C, oil gravity °API, gas gravity SG.

### 3.3 RockPhy model wrappers — method identification (kernels compiled)

| Wrapper (`RockPhy_*.py`) | Published model | Key params / library call | Notes |
|---|---|---|---|
| `MineralMix` | **Reuss / Voigt / Hill / Hashin-Shtrikman** solid mixing | `rkphy.multiReuss1/multiVoigt1/multiHill1/hashinShtrikman` | HS mixes all `shale`-type then all `matrix`-type, then the two. |
| `FluidMix` | **Brie / Reuss** fluid mix | `rkphy.brie`, `rkphy.multiReuss`, `mineralPhaseDensity` | Sw/Sg/So; Brie default e=3. |
| `HertzMindlin_Models` | **Dvorkin-Nur** Soft(friable) / Stiff sand / Contact-cement + **Gassmann** | `hertzMindlinFriableSandModel`, `hertzMindlinStiffSandModel`, `contactCementModelQSI`, **`gassmanKGdry(Kmin,Kdry,Gdry,φ,Kfl)`** | Defaults: critical porosity **0.4**, coordination number **9**, shear-reduction factor **1**. K_sat clipped to [Kfl, 2Kmin]. |
| `Modified_Bound_Models` | **Modified Hashin-Shtrikman** (critical-porosity soft/stiff) | `modifiedHashinShtrikman`, `criticalPorosityMethod`, `Reuss/Voigt/Hill` | |
| `Inclusion_Models` | **Kuster-Toksöz** | `rkphy.toksoz` | Inclusion aspect-ratio spectra. |
| `KriefNur` | **Krief (1990) / Nur critical-porosity** + Gassmann | `RAF.KriefEquation` (`Raformula.pyc`) | Krief: `K_dry = K0·(1−φ)^(α/(1−φ))`, **α default 2.7** (docstring m(φ)=3/(1−φ)); Nur uses critical porosity (default 0.15 in this wrapper). |
| `Hudson` | **Hudson (1980/1981)** cracked-rock weak-inclusion anisotropy | returns C 6×6 (TIH, aligned crack normals) | Dry cracks → Kfl=0. Small crack density/aspect ratio. |
| `XuWhite` | **Xu-White (1995)** sand-clay inclusion | `Library_XuWhite.pyc` | Outputs K, Kdry, G, reconstructed DTC/DTS, sand & shale porosity. |
| `RPI_DEM` | **Differential Effective Medium** (+critical-porosity + HS-lower-bound fill) | `dem.` (`RPI_dem.pyc`) | φc=1 → conventional DEM. |
| `RPI_Ciz_Shapiro` | **Ciz & Shapiro (2007)** generalized Gassmann / Brown-Korringa (solid pore-fill) | `cs.Ciz_Shapiro` (`RPI_Ciz_Shapiro.pyc`) | Explicit citation in docstring. |
| `RPI_BAM` | **Bounding Average Method** (Marion 1990) | `bam.BAM1` (`RPI_BAM.pyc`) | Two-phase w/ critical porosity. |
| `RPI_Walton` | **Walton (1987)** sphere-pack contact | `RPI_Walton.pyc` | |
| `RPI_berrysc` | **Berryman self-consistent** (SC/CPA) | `RPI_berrysc.pyc` | |
| `CarbonateModel` | Carbonate dry-frame (vug porosity) | `Library_CarbonateRockModel.pyc` | |
| `PatchSaturationFluidSubstitution` | **Patchy saturation** | `Library_PatchySaturationModel.pyc` | |
| `ComputeCO2_Properties` | CO2 EOS ρ/K/heat-capacity from P,T | `Library_CO2Properties.pyc` | Likely Span-Wagner-class; kernel compiled. |
| `IRP_BulkModulus/ShearModulus (+_Optimization)` | **ISIS Rock Physics (IRP)** proprietary regression estimation | `isisrpm/` package | Regression var ∈ {None, Porosity, Vmin1, PorosityAndVmin1}. Proprietary — treat as black box. |
| `Han_EmpiricalRelations_Parametric_Vp/Vs_Phi` (+ `NonLinCalibration`) | **Han (1986) / Eberhart-Phillips (1989)** Vp,Vs = f(φ, Cclay, Pe) | **Fully readable** — see 3.4 | |
| `EI_EEI` | **Connolly (1999) EI / Whitcombe (2002) EEI** | **Fully readable** — see 3.5 | |
| `DTC_Reconstruction_Gardner`, `RHOB_Reconstruction_Gardner` | Gardner forward/inverse w/ auto-calibration | uses `gardner`/`gardnerinv` | writes fitted a,b to AWI table. |
| `BackusAverageZone`, `BackusAverageMovingWindow` | **Backus (1962)** upscaling average | readable driver | window = N samples (half-length). |
| `ElasticCurves`, `SyntheticVelocities`, `Compressional/Shear_*_from_*` | moduli↔velocity↔slowness conversions | uses `elastic`/`elastic2` | trivial unit/algebra. |
| `LambdaRho_MuRho` | **Goodway (1997)** LMR | **Fully readable** — see 3.6 | |

### 3.4 Han / Eberhart-Phillips (readable) — velocities in **km/s**

Multivariate form (Rock Physics Handbook / Eberhart-Phillips, Han & Zoback 1989), effective pressure `Pe` term with the 16.7 decay constant:
- **Vp** = `5.77 − 6.94·φ − 1.73·√Cclay + 0.446·(Pe − e^(−16.7·Pe))`
- **Vs** = `3.70 − 4.94·φ − 1.57·√Cclay + 0.361·(Pe − e^(−16.7·Pe))`

(φ = porosity v/v, Cclay = clay volume v/v, Pe in the script scaled by 0.01 → kbar.) The `NonLinCalibration_Vp/Vs` variants fit the coefficients to well data. Basis: Han (1986) samples φ 3–30 %, clay 0–55 %.

### 3.5 Connolly EI / Whitcombe EEI (readable), K=(Vs/Vp)²

- **Elastic Impedance (Connolly 1999):** `EI(θ) = Vp^a · Vs^b · ρ^c`, `a=1+tan²θ`, `b=−8K·sin²θ`, `c=1−4K·sin²θ`. Normalized variant `EI_N = Vp0·(Vp/Vp0)^a·(Vs/Vs0)^b·(ρ/ρ0)^c`.
- **Extended EI (Whitcombe 2002):** `EEI(χ) = Vp0·(Vp/Vp0)^p·(Vs/Vs0)^q·(ρ/ρ0)^r`, `p=cosχ+sinχ`, `q=−8K·sinχ`, `r=cosχ−4K·sinχ`. K taken as mean (Vs/Vp)² when normalization = Automatic.

### 3.6 Goodway Lambda-Rho / Mu-Rho (readable)

`MuRho = (ρ·Vs)² = SI²`;  `LambdaRho = (ρ·Vp)² − 2·MuRho = AI² − 2·MuRho`. Inputs Vp, Vs (km/s), ρ (g/cc).

---

## 4. Acoustics

**Headline:** like EnvCorr, this package is a **data-preparation / descriptor layer** — it contains **no independent, reimplementable geophysics** (no Wyllie/Raymer sonic-porosity, no Gassmann, no Thomsen ε/γ/δ, no Alford rotation, no semblance/STC in Python). All real DSP and slowness math is in the **compiled `TechlogAcoustic` module** (imported as `ac`) and in C++ workflow wizards (`deltaTWizard`, `anisotropy_wizard_16`). Only elementary Python arithmetic exists: azimuthal-sensor waveform combination, zero-padding/depth-shift bookkeeping, and fast-shear-azimuth (FSA) ±90° swap editing.

### 4.1 Architecture (dominant pattern = dead end for algorithms)

Same EnvCorr-style descriptor pattern. `AcousticTool` (subclass of `Preprocessing.Tool`) does **not** compute slowness — it reads DLIS/waveform axis metadata and writes Techlog waveform **properties**: `TL_WFLEN`, `TL_NUMRCV`, `TL_WFSTART`, `TL_WFSAMRATE`, `TL_RRSPACING`, `TL_TRSPACING`, `TL_DEPTHREF_OFFSET`, `TL_SLL`/`TL_SUL` (slowness search limits), `TL_PKSL`/`TL_PKTT`/`TL_PKCH` (peak slowness/time/coherence). TR spacing computed by array arithmetic only. Vendor mid-classes `AcousticSLBTool` / `AcousticHALTool` (Halliburton) / `AcousticBHITool` (Baker Hughes) / `AcousticWFTTool` (Weatherford) / `AcousticINATool` — the non-SLB ones are near-empty. ~40 leaf `Tools/*.py` each declare `getName`, `isMatching` (mnemonic fingerprint), `getWaveformToKeepList`, hard-coded TR-spacing/sample-rate constants, and rename maps.

Math delegated to compiled `ac` (`TechlogAcoustic`): `ac.waveformNormalization(...,gainType,...)` (gain apply; `product` for SLB, `dB` for Baker XMAC), `ac.removeDCoffset(...)` (baseline removal before STC; modes `OptimizedWindow` default / `SingleWindow`), `ac.waveformPack`/`waveformUnpack`, `ac.checkToolName`. Wizard launchers (`AcousticsWizardLauncher.py`, `AnisotropyWizardLauncher.py`) are pure UI/XML plumbing over `.cfg` files — the STC/semblance and Alford/Thomsen solvers live behind the compiled wizards.

### 4.2 REAL algorithm — azimuthal waveform mode synthesis (`WaveformUtility.py`)

Synthesizes monopole/dipole/quadrupole borehole modes from azimuthal-sensor (segmented-receiver) waveforms by sample-by-sample sum/difference (numpy, full array). With A,B,C,D = azimuthally-spaced sensor waveforms:
- Monopole (2 sensors): `MP = (A+B)/2`;  Monopole (4): `MP = (A+B+C+D)/4`
- Dipole (opposing pair): `DP = (A−C)/2`
- Quadrupole: `QP = (A−B+C−D)/4`

Standard **azimuthal modal decomposition** of a segmented cross-dipole/array tool (modal slowness itself still computed downstream in `ac`). Vendor call sites: Halliburton XBAT (`XBM*/XBQ40*/XBD50*`), Halliburton WaveSonic sensor mode (`WMA..WMD`, `WXA/WXC…`), Halliburton BAT; Weatherford CXD/MDX/MDA/MSS instead use `ac.waveformPack` to bundle 8 receivers per mode.

### 4.3 REAL algorithm — waveform zero-padding / acquisition-delay alignment (`WaveformUtility.py`, `waveformPadZero.py`)

Prepends N zero samples per receiver to align to a common time origin: `nZeros = int(startTime / SamplingRate)` (SamplingRate from `TL_WFSAMRATE`/`AXIS_SPACING`). Modes: **Variable** (per-depth start-time channel, e.g. LWD SonicVISION `TFST`) / **Constant** (default `StartTimeValue = 300 µs`). Not a published petrophysics equation — a first-motion time-base alignment utility feeding STC. Output `<wf>_ZEROPAD`.

### 4.4 REAL algorithm — Fast-Shear-Azimuth (FSA) swap & anisotropy edit (`AnisotropyDirectionSwapping.py`, `AnisotropyWaveformSwapping.py`)

The manual **fast/slow-shear 90°-ambiguity editing** step for cross-dipole anisotropy (NOT the rotation itself — that is compiled). Flips FSA by 90° and swaps dependent channels at edited depths:
- Swap decision: `mid = AUL − 90`; if `FSA < mid → FSA+90`; if `FSA > mid → FSA−90`; else `FSA = mid` (AUL = azimuth upper limit 90/180).
- North-90 error bounds: `err_max = FSA + FSA_ERR` (wrap −180 if >90); `err_min = FSA − FSA_ERR` (wrap +180 if <−90).
- North-180: `FSA_180 = FSA + 180 if FSA < 0`.
- Time-difference channels (`TDIF`, `TDIF_B`) sign-flipped on swap.
- Waveform swap frame shift: `shiftamount = trspacing[0]/2 + (trspacing[r−1] − trspacing[0])`, `r=(total_r+1)//2`; `frameshift = int(shiftamount // samplingRate)`. Swap depths where `|FSA_ref − FSA_swap| > 1°`.

Underlying **Alford 4-component rotation and Thomsen parameters are absent from Python** (produced by compiled `anisotropy_wizard_16`). SLB Sonic Scanner workflow. Inputs FSA/FSA_ERR, TDIF/ADIF/CCOR, fast/slow waveforms, `TL_AUL/TL_ALL`, `TL_TRSPACING`; outputs edited `FSA_NORTH90/180` + swapped `ADIF/CCOR/TDIF/WF_FAST/WF_SLOW`.

### 4.5 Minor real utilities (`WaveformUtility.py`)

`receiverRenumbering` (reverse receiver order, index-only); `ExtractSubarray_SPR_numpy` (trim zero columns of a slowness-time projection, set `TL_SLL = idx1·SSTE`, `TL_SUL = idx2·SSTE`); `shiftWaveforms_withUnit` (block-shift to array mid by `TL_DEPTHREF_OFFSET` via `db.shiftArrayByConst`). Bookkeeping only.

### 4.6 Tool inventory (~40 `Tools/` descriptors — all descriptor-only, no math)

- **SLB:** DSI, Sonic Scanner (WFA1..12; MaxWell+GeoFrame branches), SonicScope (LWD quadrupole), sonicVISION (LWD), DSLT, ASLT/ISLT/QSLT/SSLT (array; renumber + SPR subarray), SDT, GeoFrame, SonicPacer variants (incl. PathFinder), ThruBitDP.
- **ThruBit / PathFinder (base AcousticTool):** ThruBit, eSONIC.
- **Baker Hughes:** XMAC / XMAC_Unpacked (gainType `dB`; TR/sample 12/8/36/32 µs), SoundTrak (LWD), XDA (INA base).
- **Halliburton:** BAT, BSAT, XBAT (mono stack + dipole/quadrupole combine §4.2), WaveSonic / _pack / _Sensor / _Packed_v2 (sensor-mode azimuthal combine), Xaminer.
- **ATL:** ATL_FWS50.
- **Weatherford:** CXD / WFT_CXD_sensor, WFT_CrossWave, WFT_HBC, WFT_MDA/MDX/MDX_ABCD/MSS/ShockWave (receiver packing to `WF_MPS/XX/YY/...`).

### 4.7 Dead ends (explicit)

Both wizard launchers (UI/XML only); the four near-empty vendor base classes; all `Tools/*.py` (descriptors over `ac.*`). **The STC/semblance slowness estimator, gain normalization, DC-offset filter, and Alford-rotation/Thomsen anisotropy solver are compiled (`TechlogAcoustic` + C++ wizards) and NOT recoverable from Python.** Independent reimplementation must reference published sources directly: semblance/STC (Kimball & Marzetta 1984), Alford rotation (Alford 1986), cross-dipole anisotropy processing.

**Acoustics completeness:** 19 of 54 files read in full (all 14 top-level + 5 representative `Tools/`); the other 35 `Tools/` characterized via math-signature + call-site grep (structurally identical descriptors). No reimplementable algorithm missed.

---

## 5. Preprocessing

`Preprocessing/` is the **base framework** that EnvCorr and Acoustics build on, not a set of DSP algorithms.

- **`Tool.py`** — abstract `Tool` base: `getName`, `getVariableToKeepList`, `getOptionalVariableList`, `getVariableToRename`, `getOutputDatasetBaseName`, regex `strMatches`, property-copy helpers. This is the contract every EnvCorr/Acoustics tool implements.
- **`Preprocessing.py`** — `class Preprocessing`: `loadTools()` (dynamic import of every `Tools/*.py` across project/user/company/install dirs), `findMatchingTool()` (auto-detects applicable tool per well/dataset by mnemonic regex, or forced by name), `createDataset` / `copyVariables` / `renameVariables` / `process`. **Resampling/depth-alignment is delegated to `db.variableCopy(..., interpolationMethod)`** (default `'automatic'`, else `'linear'`) — the actual resample kernel is in compiled `TechlogDatabase`. Variable provenance saved via `TL_ORIGINAL_NAME` / `TL_ORIGINAL_DATASET`.
- **`Tools/Example.py`** — template only.

**No despiking, normalization, or edit-detection math ships in this package** — those live in the Techlog C++ engine (and separate top-level scripts like `QC_OneLog.py`, `RescaleVariables_*.py`, `DatasetHarmonisation.py`, `InstantFFT.py`). SandiBumi verdict: framework pattern is worth borrowing (mnemonic-driven tool auto-detect), but there's no reusable algorithm here.

---

## 6. Techlog package — public API surface (feature checklist)

`import Techlog` re-exports from four readable sub-modules; the heavy lifting is in compiled C-extension modules `TechlogDatabase` (aliased `db`), `TechlogMath`, `TechlogStat`, `TechlogPlot`, `TechlogPlatform`.

**`Techlog.Data`** — data model
- `Variable` — a log curve (load/value/setValue/save, family, unit, storage type, zone-aware indexing).
- `Array` — 2-D array curve (waveforms/image rows).
- `CacheVarData` — cached curve buffer.
- `HistoryItem` — processing-history record.
- `DataProperty`, `DataPropertyManager`, `AbstractDataClass` — property plumbing.
- `MissingValue = −9999` (global null convention).
- `getDatasetZoneIndiceTop(well,dataset,zonation,zone)`, `getDatasetZoneIndiceBottom(...)`.

**`Techlog.Engine`** — execution context
- `Context` — run/well/dataset/zonation context.
- `Parameter`, `ParameterProxy` — the `parameterDict` typed-parameter system (name/type/family/unit/mode/min/max/list/enable…) used by every shipped script header.
- `Zone`, `Zonation` — zone iteration.
- Loop tokens: `LOOP`, `LOOP_MVTEST`, `LOOP_ARRAY`, `LOOP_ARRAY_MVTEST`, `LOOP_INV`, `LOOP_INV_MVTEST`, `LOOP_INV_ARRAY`, `LOOP_INV_ARRAY_MVTEST` (the "Automatic Generation Loop" markers — MVTEST = skip-missing, INV = inversion/multi-input, ARRAY = 2-D).

**`Techlog.Plot`** — plotting
- `CrossPlot`, `Histogram`, `LogView`.

**`Techlog.Utils`** — utilities
- `ProgressBar`, `output`, `exit`, `printError`, `printErrorCount`, `printTest`, `ExitExpectedException`, `PrintErrorLimit=100`.

This is a **feature checklist**, not an algorithm source — the numeric engine is compiled.

---

## 7. PPP (pore-pressure / geomech helpers)

- **`PPP/Formation_Temperature_Models.py`** — "BP Temperature Models", regional formation-temperature vs depth (readable). `Dbml = TVD − WaterDepth`, T in °F:
  - Deepwater GOM: `T = 39 + 0.014·(Dbml+KB)`
  - Caspian: `T = 58 + 0.0285·TVD^0.893`
  - Gulf Coast: `T = 72 + (0.002·Dbml)^1.55`
  - North Sea: `T = 50 + 0.0178·Dbml`
  - Trinidad: `T = 72 + 0.01154·Dbml`
  - Malaysia: `T = 115 + 0.026·TVD`  ← relevant to Mahakam / SE-Asia.
- **`PPP/Anisomecpro/`** — `anisomec_moduli_ratio.py`, `anisomec_poisson_ratio.py`, `anisomec_stress_modeling.py`, `anisomec_youngs_modulus.py` — anisotropic (TIV) mechanical-property / horizontal-stress modeling helpers (readable drivers; some Biot-coefficient math). Secondary priority.

---

## 8. "Worth reimplementing in SandiBumi" shortlist

Effort: **S** = a few hrs (closed-form, constants in hand), **M** = 1–2 days, **L** = multi-day (needs literature + validation data).

| Item | Source | Why | Effort |
|---|---|---|---|
| Resistivity T-correction (Arps/Exxon) | `TempCorr_Resistivity.py` | Exact formula+constants; ubiquitous QC step. | **S** |
| Rw/Salinity Gen-6 (NaCl brine) | ident. only (compiled) — reimplement from Charts Book Gen-6 / Bateman-Konen | Core Rw↔salinity↔T,P; Mahakam fresh-water pay. | **M** |
| Batzle-Wang 1992 fluids | `RockPhyEquations.py` (full) | Complete constants transcribed; feeds any Gassmann sub. | **M** |
| Gardner (per-lithology) fwd/inv | `RockPhyEquations.py` (full) | DTC/RHOB reconstruction, defaults in hand. | **S** |
| Faust DTC-from-Rt | `RockPhyEquations.py` (full) | Synthetic sonic where DT missing; factor 1945/exp 1/6. | **S** |
| Reuss/Voigt/Brie/Hill mixing | `RockPhyEquations.py` (full) | Fluid & mineral mixing primitives. | **S** |
| Elastic conversions + LMR + EI/EEI | `RockPhyEquations.py`, `LambdaRho_MuRho.py`, `RockPhy_EI_EEI.py` (full) | AVO attributes; all closed-form. | **S** |
| Han/Eberhart-Phillips Vp,Vs | `RockPhy_Han_*` (full) | Vp/Vs from φ,Cclay,Pe; coefficients in hand. | **S** |
| Gassmann + Hertz-Mindlin (soft/stiff/cement) | ident. only (compiled) — reimplement from Rock Physics Handbook | Fluid substitution & unconsolidated-sand modeling; defaults φc 0.4, Cn 9, SRF 1. | **M** |
| Krief dry-frame | ident. (compiled) — formula in docstring `K_dry=K0(1−φ)^(3/(1−φ))` | Simple, α=2.7 default. | **S** |
| BP formation-temperature models | `PPP/Formation_Temperature_Models.py` | Includes a Malaysia model for SE-Asia. | **S** |
| EnvCorr tool→curve→resolution taxonomy | `EnvCorr/Tools/*` | Drive a "which correction applies" advisor; NOT the chart math. | **M** (data-entry) |
| DEM / Ciz-Shapiro / Kuster-Toksöz / HS bounds / Backus | ident. only (compiled) — Rock Physics Handbook | Advanced RPT; reimplement per literature. | **L** each |

**Dead ends (compiled/opaque, do not chase in Python):** EnvCorr chart engine (`EnvCorrPreProcessingPrivate.pyc`), `RockPhysics_EquationsLibrary.pyc` + `Raformula.pyc` + all `RPI_*.pyc`/`Library_*.pyc` kernels, **Acoustics `TechlogAcoustic` (`ac`) STC/semblance + Alford/Thomsen wizards**, RST (`RSTCOT_x64.dll` + `RSTUtilFunctions.pyc`), `CPI_utility.pyc`, `TechlogDatabase/Math/Stat/Plot` C-extensions. For every one, the readable wrapper still yields method name + citation + defaults, which is the intended reimplementation path.

**Acoustics verdict:** no reimplementable geophysics in the Python layer — the entire package is descriptor/prep. SandiBumi should NOT invest here; get slowness (Kimball-Marzetta STC) and anisotropy (Alford) from the literature if needed.

---

## 9. Extraction completeness

- **Files opened in full:** ~16 (EnvCorrTool, EnvCorrPreProcessing, EnvCorrPreProcessingSLB, GR, AIT, ADN_DENSITY_STANDARD, TempCorr_Resistivity, RockPhyEquations, RockPhysics_TechlogUtilities, RockPhy_KriefNur, RockPhy_HertzMindlin_Models, RockPhy_EI_EEI (partial), Preprocessing, Tool, PPP/Formation_Temperature_Models, Techlog/__init__).
- **Files sampled via grep/scripted docstring+signature extraction:** all ~40 `RockPhy_*.py` (docstrings + library calls + defaults), all 65 `EnvCorr/Tools/*` (names/mnemonics/resolutions), `BoreholeEnvCorr.py`, `Han_*` (equation lines), `Techlog/{Data,Engine,Plot,Utils}.py` (class/def listing).
- **Deliberately skipped:** the ~120 top-level example/import/IO/UI/geomech-utility scripts not in the 5 priority packages (Example*_*.py tutorials, DLIS/LAS importers, dip/image tools), `pyExcelerator/` (OSS xls writer), and all `.pyc`/`.dll` compiled kernels (identified, not decompiled — IP-careful).
- **Priority-1 EnvCorr:** completeness limited by design — the correction arithmetic is compiled; taxonomy + borehole-parameter + T-correction fully captured.
- **Priority-2 RockPhy:** high — readable geophysics tier fully transcribed; model kernels identified with citations + defaults.
- **Priority-3 Acoustics:** 19/54 read in full + 35 characterized by grep; confirmed descriptor-only, real DSP compiled in `TechlogAcoustic`. Complete for the question asked (no reimplementable algorithm exists in Python).
- **Priority-4/5 Preprocessing & Techlog:** fully covered (framework + API surface; numeric engine compiled).
