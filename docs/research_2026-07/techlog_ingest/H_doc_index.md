# Techlog 2018.2 Offline Documentation — Formation-Evaluation Index & Equation Extraction

Source (read-only): `D:\01. Work\00. Guidebook\03. Guidebooks Techlog\Techlog 2018.2 (r22885)\Doc\`
Publisher: Schlumberger, (C) 2018. DITA-generated XHTML help. ~3,808 HTML pages + raster images.
Purpose: let SandiBumi cite equation-level method documentation for FE workflows.
Prepared: 2026-07 ingest.

---

## 1. Doc Tree Structure

Top level of `Doc\`:

| Path | Role |
|---|---|
| `home.html`, `index.html` (402 KB), `tocnav.html` | Entry points / master TOC. `index.html` is the full keyword index. |
| `concept\` | **1,417 pages — the substantive content.** Concept/reference topics incl. all petrophysics method + equation pages. Flat folder, descriptive hyphenated filenames. |
| `task\` (25) | Step-by-step how-to task pages (mostly non-petrophysics: geology, import, printing). |
| `reference\` (1) | Almost empty (one geology drift-computation reference). |
| `topic\pythonlib\` | **2,248 pages — Python API function reference** (`TechlogQuanti` etc.): one page per callable, with argument names/units/defaults. |
| `modulesDescription\<Module>\` | One `index.html` per module (Quanti, QuantiMin, WBI, NMR, Acoustics, ShaleSuite, Techcore…). These are thin landing pages; the real docs live in `concept\`. |
| `image\` | All figures **and all equations** (GIF/PNG/JPG). Equations are rasterized images, not MathML/text. |
| `css\`, `js\`, `index\`, `ressource\`, `csh\` | Styling, search-index, context-sensitive-help plumbing. |
| `WelcomeTour\`, `WhatsNew\` | Onboarding / release notes. |

**Naming convention:** FE method pages are almost all `concept\petrophysics-<method>.html`. Deterministic Quanti methods use `petrophysics-<name>.html` / `quanti-<name>.html`; the multi-mineral solver uses `petrophysics-elan*` / `petrophysics-quantielan*` and `quanti-elan-theory.html`. Python equivalents are `topic\pythonlib\<funcname>.html`.

**How equations are stored (critical for citation):** Each method page has HTML **text tables** for *Inputs*, *Equation parameters* (with **default values**), and *Outputs*, followed by an *Equations* section where the formula itself is an **`<img>` (GIF/PNG)**. So symbols, units and defaults are machine-readable text; the math must be read from the image (transcribed below). Each page also carries DITA metadata (`contentId`, `DC.Title`, SVN rev/date) usable as a stable citation key.

---

## 2. Categorized Index of FE-Relevant Pages

All paths relative to `Doc\`. "E" = equation-level (formula images + parameter tables). "H" = how-to/UI only. "P" = parameter/definition tables.

### 2.1 Water saturation — Quanti deterministic

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `petrophysics-archie.html` | Archie | E+P | Clean-formation Sw (Archie), a/m/n/Rw defaults |
| `petrophysics-simandoux.html` | Flushed zone saturation Simandoux | E+P | Sxo via Simandoux (Rxo, Rmf) |
| `petrophysics-modified-simandoux.html` | Modified Simandoux | E+P | Effective-porosity Sw (modified Simandoux) |
| `petrophysics-indonesia.html` | Indonesia | E+P | Poupon-Leveaux Indonesia Sw |
| `petrophysics-waxmansmits.html` | Waxman-Smits | E+P | Waxman-Smits Sw (Qv, B, m*/n*) |
| `petrophysics-dual-water-method.html` | Dual water method | E+P | Dual-Water total-porosity Sw (Levenberg-Marquardt) |
| `petrophysics-total-shale.html` | Total shale | E+P | Total-shale Sw model |
| `petrophysics-juhasz-method.html` | Juhasz | E/P | Juhasz normalized-Qv (shaly-sand) |
| `petrophysics-flushed-zone-saturation-archie.html` / `-indonesia.html` / `-dual-water.html` / `-modified-simandoux.html` / `-waxmansmits.html` | Flushed-zone Sxo variants | E+P | Flushed-zone form of each Sw model |
| `quanti-saturation-methods.html` | Saturation methods | H | Index/landing for Sw methods |
| `petrophysics-quanti-totalporosity-saturation.html` / `-effectiveporositysaturation.html` / `-flushedzonesaturation.html` | Sw families | H | Landing pages grouping the above |
| `petrophysics-pnc-saturation-absolute.html` | PNC (cased-hole) saturation | E/P | Sigma/pulsed-neutron Sw |

### 2.2 Water saturation — Quanti.Elan (multi-mineral inversion)

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `quanti-elan-theory.html` | Quanti.Elan theory | H(index) | Master TOC of ELAN theory chapter |
| `petrophysics-elanplus-solution-method.html` | ELANPlus solution method | **E** | Incoherence function + standard deviation (Eq 79/80) |
| `petrophysics-elanplus-overdetermined-solution.html` | Overdetermined solution | E(concept) | Weighted least-squares, weight=1/uncertainty |
| `petrophysics-elan-theory-uncertainties.html` | ELAN uncertainties | H(index)+E | Uncertainty→weight, weight multipliers, sqrt-conductivity |
| `petrophysics-elan-theory-conductivity-models.html` | Conductivity models | E/P | Table 21 of Sw models; UWAT/UIWA→Sw function |
| `petrophysics-elanplus-simandoux-conductivity-equation.html` | ELAN Simandoux conductivity | **E+P** | Full derivation + Eq 78, ersh/swshe/mc2 defaults |
| `petrophysics-elanplus-dualwater-equation.html` | ELAN dual-water equation | E+P | Dual-Water conductivity, b(T), mdw=1.8 |
| `petrophysics-elanplus-indonesian-nigerian-conductivity-equations.html` | Indonesian/Nigerian | E+P | EVCL/MVCL = 1.0/0.5 (Indo), 1.4/0.0 (Nigeria) |
| `petrophysics-elanplus-linear-conductivity-equation.html` | Linear conductivity | E | Linear (simplified Dual-Water) conductivity |
| `petrophysics-elanplus-water-saturation-linear-conductivity.html` | Sw from linear conductivity | E | Deriving linear eq from classic Sw |
| `petrophysics-elanplus-wet-dry-clay.html` | Wet/dry clay | E/P | Wet-clay porosity, bound-water partitioning |
| `petrophysics-elanplus-conductivity-constraint-waterbased-mud.html` | WBM conductivity constraint | E | CXDC / EQHY constraints |
| `petrophysics-quantielan-equations.html` | Quanti.Elan equations | H(index) | Linear/resistivity/neutron/sonic/CRIM/spectroscopy eq sets |
| `petrophysics-quantielan.html` / `-workflow.html` / `-postprocessing.html` / `-utilities.html` | Quanti.Elan module | H | UI workflow, model build, post-processing |
| `petrophysics-inversion-constants.html` | Inversion constants | H/P | `QUANTI_INVERSION_CONSTANTS.xml` endpoints table |
| `petrophysics-elan-theory-glossary.html` | ELAN glossary | P | Mnemonic/term definitions |
| `petrophysics-assumptions-elanplus-application.html` | ELAN assumptions | H | Modeling assumptions |

> Note: `petrophysics-inversion.html` is **CMR/NMR** T2 inversion, NOT the ELAN mineral solver — do not confuse.

### 2.3 Shale volume (Vsh)

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `petrophysics-vsh-from-gamma-ray.html` | VSH from gamma ray | **E+P** | GR index + Linear/Clavier/Larionov×2/Stieber×3/Curved |
| `petrophysics-vsh-from-potassium.html` | VSH from potassium | E/P | Linear + Larionov (K curve) |
| `petrophysics-vsh-from-thorium.html` | VSH from thorium | E/P | Vsh from Th |
| `petrophysics-shale-volume.html` | Shale volume combined method | H+P | Merge stats (arith/geom/harmonic/median/min/max/first/product/sum); points to per-method eq pages |
| `quanti-shale-volume.html` | Shale volume | H(index) | Landing page for Vsh methods |
| `petrophysics-modified-total-shale.html` | Modified total shale | E/P | Modified total-shale Vsh |

### 2.4 Porosity (density / neutron / sonic)

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `petrophysics-effective-porosity-saturation-from-density.html` | PHIE & Sw from density | E+P | PHIE, SWE, SXOE from RHOB; HC+shale correction; Tixier/Pickett Sxo |
| `petrophysics-total-porosity-saturation-from-density.html` | PHIT & Sw from density | E+P | Total-porosity variant |
| `petrophysics-effective-porosity-from-neutrondensity.html` | PHIE from neutron-density | E+P | ND crossplot PHIE; RHOB_sh 2.4, NPHI_sh 0.4 defaults |
| `petrophysics-effective-porosity-saturation-from-neutrondensity-input-variables.html` / `-pefsonic.html` | ND(+PEF/sonic) PHIE & Sw | E+P | Compound ND / ND-Pe-sonic porosity+saturation |
| `petrophysics-total-porosity-from-sonic.html` | PHIT from sonic | E/P | Wyllie / sonic porosity |
| `petrophysics-effective-porosity-from-sonic.html` | PHIE from sonic | E/P | Sonic effective porosity |
| `petrophysics-neutron-porosity-equations.html` | Neutron porosity equations | E | Neutron porosity transform set |
| `petrophysics-compressional-slowness-equation.html` | Compressional slowness | E | DTC equation |
| `density-magnetic-resonance-porosity.html` | Density-MR porosity | E | DMRP |

### 2.5 Permeability

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `petrophysics-coates.html` | Coates | **E** | Coates (free-fluid) k, clean & shaly forms |
| `petrophysics-wyllierose.html` | Wyllie-Rose | **E+P** | Wyllie-Rose k; Morris-Biggs & Timur coefficients |
| `petrophysics-quanti-permeability.html` | Permeability | H(index) | Landing for k methods |
| `petrophysics-boundfree-fluid-permeability.html`, `petrophysics-integrated-permeability.html`, `petrophysics-kmod.html` | NMR/CMR permeability | E/H | Coates-FFI & SDR-type NMR k |

### 2.6 Thomas-Stieber / thin-bed / LRLC

| Path (concept\) | Title | Kind |
|---|---|---|
| `petrophysics-thomasstieber.html` (+ `-porosity-tab`, `-saturation-tab`, `-resolution-tab`, `-iterative-hydrocarbon-correction-tab`, `-options`, `-workflow`) | Thomas-Stieber | E/H |
| `thomasstieber-plot-neutron-vs-density.html`, `thomasstieber-plot-shale-volume-vs-porosity.html` | T-S crossplots | E |
| `petrophysics-low-resistivity-pay*.html`, `petrophysics-lowrep-method.html`, `lowrep-paytheoreticalworkflow.html`, `petrophysics-low-resistivity-pay-awi-response-equations.html` | Low-Resistivity Pay (LowReP/AWI) | E/H |
| `petrophysics-add-vertical-horizontal-resistivities.html` | Rv/Rh anisotropy | E |

### 2.7 Temperature / Rw / environmental corrections / cross-plots

| Path (concept\) | Title | Kind | Documents |
|---|---|---|---|
| `parameter-initialization-from-salinity-temperature.html` | Parameter init from salinity & T | E(img)+P | Computes Rw, Rmf from salinity+T; grad 0.03 degC/m default |
| `waxman-b-from-temperature-salinity.html`, `petrophysics-waxman-b.html` | Waxman B | E/P | B(T,salinity); 1978/1972 charts |
| `petrophysics-compressional-slowness-resistivity-temperature-calculation.html` | Rw/T from DTc | E | Temperature/Rw from compressional slowness |
| `petrophysics-environmental-corrections.html`, `petrophysics-schlumberger-environmental-corrections.html` | Environmental corrections | H+P | Per-tool borehole corrections (mostly charts/how-to) |
| `petrophysics-hydrocarbon-correction.html` | Hydrocarbon correction | E/P | HC density/HI porosity correction |
| `petrophysics-pickett-plot-porosity-function-resistivity.html`, `petrophysics-pickett-plot-resistivity-function-porosity.html`, `petrophysics-pick-parameters-from-pickett-plot.html` | Pickett plot | H | Graphical a/m/Rw picking |
| `plot-regressions-equations-crossplot.html`, `crossplot-array-vs-array.html`, `plot-charts.html` | Cross-plots | H/E | Regression equations on crossplots |

### 2.8 Python API (equation contracts) — `topic\pythonlib\`

One page per callable with argument names, units, defaults, output. Useful for citing the exact parameter contract SandiBumi should mirror. Key FE functions:

- **Sw:** `swarchie`, `swesimandoux`, `swemodifiedsimandoux`, `swemodifiedsimandouxvarn`, `sweindonesia`, `swetotalshale`, `swemodifiedtotalshale`, `swedispersedshale`, `swtdualwater`, `swtwaxmansmits`, `swtjuhasz`, `sweequivalent`, `swtequivalent`
- **Vsh:** `vshgammaray`, `vshpotassium`, `vshthorium`, `vshneutrondensity`, `vshsonicdensity`, `vshneutronsonic`, `vshresistivity`, `vshmn`, `vshlaminated`, `vshdispersed`, `vshstructured`
- **Porosity:** `pordensity`, `porneutron`, `porsonic`, `porneutrondensity`, `porneutronsonic`, `poreshearsonic`, `porvshcorrected`, `porcorecalibrated`, `pordeep`/`porshallow`, `pordielectric`
- **Rw / clay:** `rwfromftemp-salinity`, `rwfromftemp-sp`, `rwfromftemp-ionconc`, `bqv`, `bwaxmanfromrw`, `bwaxmanfromsalinity`
- Master: `python-module-techlogquanti.html`

---

## 3. Extracted Equations & Parameters (high-value methods)

Equations transcribed from the source equation images (path cited). Symbols/units/defaults are from each page's HTML parameter tables.

### 3.1 Archie — `concept\petrophysics-archie.html`
Image: `image\modules-quanti-saturation-archie.gif`

```
SW = ( a·Rw / (Rt · φt^m) )^(1/n)
SH = 1 − SW            (image: modules-quanti-saturation-sh.gif)
BVW = SW · φ
```
Inputs: Rt (ohm.m), φ (POR, v/v), a, m, n, Rw. Outputs: `SW_AR`, `SH_AR`, `SW_AR_UNCL`, `BVW_AR`.
**Defaults:** a = 1, m = 2, n = 2, Rw = 0.03.

### 3.2 Modified Simandoux — `concept\petrophysics-modified-simandoux.html`
Image: `image\modules-quanti-saturation-simand.gif` (solved for Sw)

```
(φe^m / (a·Rw·(1 − Vsh)))·Sw^n  +  (Vsh / Rsh)·Sw  −  1/Rt  =  0
```
Inputs: Rt, φe, Vsh, a, m, Rw. Outputs: `SWE_SIM`, `SHE_SIM`, `SWE_SIM_UNCL`, `BVWE_SIM`.
**Defaults:** a = 1, m = 2, n = 2, Rw = 0.03, Res_shale (Rsh) = 5.

**Flushed-zone Simandoux** — `concept\petrophysics-simandoux.html`, image `image\flushed-simandoux.png`: same algebraic form with Rt→Rxo, Rw→Rmf, φ→φe; outputs `SXO_SIM`, `SHXO_SIM`, `BVWXO_SIM`. Defaults a=1, m=2, n=2, **Rmf = 0.065**, Res_shale = 5.

### 3.3 Indonesia (Poupon-Leveaux) — `concept\petrophysics-indonesia.html`
Images: `modules-quanti-saturation-indo1`,`-indo4`,`-indo5`,`image1248`,`image1894`.gif

```
Ro = a·Rw / φ^m
B  = (Rt / Ro)^0.5
A  = Vsh^C · (Rt / Rsh)^0.5            [ = Vsh^C / (Rsh/Rt)^0.5 ]
Sw = (A + B)^(−2/n)
BVWE = SWe · φe
```
(Equivalent to the classic 1/√Rt = [Vsh^C/√Rsh + √(φ^m/(a·Rw))]·Sw^(n/2); C is the Vsh exponent.)
Inputs: Rt, φe, Vsh, a, m, n, Rw. Outputs: `SWE_INDO`, `SHE_INDO`, `SWE_INDO_UNCL`, `BVWE_INDO`.
**Defaults:** a = 1, m = 2, n = 2, Rw = 0.03, Rsh = 5.

### 3.4 Waxman-Smits — `concept\petrophysics-waxmansmits.html`
Image: `image\image1879.gif`

```
Rt = Rw / ( Sw^n* · φ^m* · (1 + Rw·B·Qv / Sw) )
```
Inputs: Rt, Qv (eq/L), POR (total, v/v), m*, n*, Rw, fTemp (degC), B (L·S/eq·m).
**Defaults:** m* = 2, n* = 2, Rw = 0.03, B = 4 (when user-defined). B-method default = **"1978 Waxman B chart"** (also "1972 original fit", "1972 revised", "user defined"). Qv/B via `bqv`, `bwaxmanfromrw`, `bwaxmanfromsalinity`.

### 3.5 Dual Water — `concept\petrophysics-dual-water-method.html`
Image: `image\modules-quanti-saturation-dualw.gif` — **solved for Sw by Levenberg-Marquardt iteration.**

```
a / (Rt · φt^m) = (1/Rw)·Sw^n  +  Qv·( 1/(φt_sh^2 · Rsh) − 1/Rw )·Sw^(n−1)
```
(`modules-quanti-saturation-qv.gif` gives the Qv term.)
Inputs: Rt, φt (total), Vsh, a, m, n, Rw. Outputs: `SWT_DUAL`, `SHT_DUAL`, `SWT_DUAL_UNCL`, `BVW_DUAL`.
**Defaults:** a = 1, m = 2, n = 2, Rw = 0.03, R_shale = 5, **Porosity_shale (φt_sh) = 0.4**.

### 3.6 Vsh from Gamma Ray — `concept\petrophysics-vsh-from-gamma-ray.html`
Reference cited on page: *Bassiouni Z., Theory, Measurement, and Interpretation of Well Logs, SPE Textbook Series vol. 4, 1994.*

```
GR_index = (GR − GR_matrix) / (GR_shale − GR_matrix)        [modules-quanti-volume-shale-gr0.gif]

Linear:               VSH = GR_index                          [gr6]
Clavier:              VSH = 1.7 − sqrt( 3.38 − (GR_index + 0.7)^2 )   [gr7]
Larionov (Tertiary):  VSH = 0.083 · ( 2^(3.7·GR_index) − 1 )   [image470]
Larionov (older):     VSH = 0.33  · ( 2^(2.0·GR_index) − 1 )   [image471]
Stieber variation I:  VSH = GR_index / (2 − GR_index)          [gr3]
Stieber Miocene/Plio: (gr4)   Stieber variation II: (gr5)   Curved method: (curved-method-equation.png)
```
**Defaults:** GR_matrix = 10 gAPI, GR_shale = 100 gAPI. Method default = Linear. (Histogram picker: matrix=quantile-5/min, shale=quantile-95/max.)

### 3.7 Permeability

**Coates** — `concept\petrophysics-coates.html`
```
Clean zones:  PERM = kc · PHLe^4 · ( (1 − Swirr) / Swirr )^2                 [coates-equation1.png]
Else:         PERM = kc · PHLe^4 · ( (PHLt − PHLe·Swirr) / (PHLe·Swirr) )^2  [coates-equation2.png]
```
Inputs: PHLe (effective φ), PHLt (total φ), Swirr; output PERM (mD).

**Wyllie-Rose** — `concept\petrophysics-wyllierose.html`, image `modules-quanti-quanti-images-permimage1.gif`
```
PERM = Kw · PHI^d / SW^e
```
| Mode | d | e | Kw (oil) | Kw (gas) |
|---|---|---|---|---|
| Morris-Biggs | 6.0 | 2 | 62500 | 6500 |
| Timur | 4.4 | 2 | 3400 | 340 |
User-input mode: d, e, Kw set manually.

### 3.8 Quanti.Elan solver methodology (multi-mineral inversion)

**Solution / incoherence function** — `concept\petrophysics-elanplus-solution-method.html`, image `image\image2969.gif` (Eqs 79–80):

```
incoherence      = ½ · Σ_tools [ (xxxx_REC − xxxx) · xxxx_UNC_WM / (xxxx_UNC · LargestWeight) ]^2          (79)
standard deviation = sqrt[ 2 · incoherence / (num tools) ] · LargestWeight                                  (80)
```
where `xxxx_REC` = curve reconstructed from output component volumes, `xxxx` = measured input curve (e.g. RHOB, NPHI), `xxxx_UNC` = tool uncertainty, `xxxx_UNC_WM` = weight multiplier, `LargestWeight` = max weight over all equations. **The program selects the component volumes that minimize the incoherence** (Eq 80 minimand). One summation term per response equation used in the Solve process.

Key principles (from `-overdetermined-solution.html`, `-uncertainties.html`, `-conductivity-models.html`):
- Overdetermined weighted least-squares: ≥ as many response equations as unknown component volumes; disagreements settled by weighted best fit.
- **weight = 1 / uncertainty** — only the *relative* uncertainties matter. A weight-multiplier of 1.0 ⇒ tool influences answer as strongly as the Volume-Summation equation.
- Volume uncertainties are **independent of the volumes** ⇒ can be computed for QC before volumes are solved ("balanced uncertainties").
- Internal equations (Summation-of-Volumes, Equal-Hydrocarbon-Ratio) also carry uncertainties/weights.
- The **square root of conductivity** is used internally by the nonlinear solver for numerical stability/speed.
- Sw is not solved directly; ELAN solves for fluid *volumes* (UWAT, UIWA, USFLw, UOIL, UGAS, USFLhc) then Sw = Σwaters / (Σwaters+Σhc) via a Function process (image `image2918.gif`).

**ELAN Simandoux conductivity equation (Eq 78)** — `concept\petrophysics-elanplus-simandoux-conductivity-equation.html`, image `image\image2967.gif`:

```
        C_ucl · V_cl^ersh · (V_uma/φe)^(n/2)          C_uwa      m+(mc2/φe)   (V_uwa/φe)^n
Ct  =  ------------------------------------------  +  -----  · φe          · ---------------------
        (V_cl + V_silt)^(ersh − swshe − 1)             a                      [1 − (V_cl+V_silt)]^(swshe+1)
```
**Defaults:** ersh = 1.0, swshe = 0.5, mc2 = 0.0 (use **0.19** for tight low-porosity limestone). These correspond to Worthington "x"=1.0 and "c"=1.5 (silt treated like clay by default). Expected ranges: ersh 1.4→2.4, swshe 0→1.0. Flushed zone: U/u→X/x, ERSH→ERSHO, N→EXPXO.

**ELAN Indonesian / Nigerian** — `-indonesian-nigerian-conductivity-equations.html`: identical form; **Indonesia (default): EVCL = 1.0, MVCL = 0.5; Nigeria: EVCL = 1.4, MVCL = 0.0.** Warning: equation is singular at Vcl=100% — add a constraint forcing Vwater ≳ 0.5 p.u.

**ELAN Dual Water** — `-dualwater-equation.html`: bound-water form; `b` is a function of **temperature only** ("volume of bound water per counterion charge", ≈ 0.28 cm³/meq at room T); `mdw` default = 1.8 (set Cdw=0 & mdw=2.0 to force fixed m=2). Uses clay-water conductivity + wet-clay porosity (WCLP); √conductivity used internally.

### 3.9 Rw / temperature initialization — `concept\parameter-initialization-from-salinity-temperature.html`
Computes salinity/temperature-dependent Quanti parameters (Rw, Rmf) from XWaterSalt/UWaterSalt (kppm), MFST, RWT, mud weight. Equations are images `quanti-elan-equations-initialization-1..8.png` (Arps-type Rw(T,salinity)). **Defaults:** Temperature gradient = 0.03 degC/m, reference Depth = 30.48 m, Water-Based-Mud = Yes. Dataset/well properties (RMFS, MFST, DFD, FSAL, GGRD) override defaults when present. Python: `rwfromftemp-salinity`, `rwfromftemp-sp`, `rwfromftemp-ionconc`.

### 3.10 Effective porosity from neutron-density — `concept\petrophysics-effective-porosity-from-neutrondensity.html`
ND-crossplot PHIE (Neutron must be limestone-calibrated). **Defaults:** RHOB_shale = 2.4, RHOB_fluid = 1.0, NPHI_shale = 0.4, HC density = 0.8, Mud salinity = 100000 ppm, m = 2, n = 2, invasion factor = 1. `-from-density` page adds Tixier Sxo = Sw^satfac (satfac default 0.2) and Pickett Sxo = (Sw+InvFac)/(1+InvFac) (InvFac default 2), water-phase salinity invaded 6.7 / virgin 33 ppk.

---

## 4. Equation-Level vs How-To — Documentation Quality Note

**Documented at equation level (formula image + parameter table + defaults — directly citable):**
- Sw deterministic: Archie, (Modified) Simandoux + flushed, Indonesia, Waxman-Smits, Dual Water, Total Shale — all with a/m/n/Rw/Rsh defaults.
- Vsh: from GR (7 named variants + curved), from K, from Th — with GR_matrix/GR_shale defaults and a literature reference (Bassiouni 1994).
- Porosity: density, neutron-density, sonic, neutron-porosity transforms — with shale/fluid endpoint defaults.
- Permeability: Coates (2 forms), Wyllie-Rose (Morris-Biggs & Timur coefficient tables).
- **Quanti.Elan: the strongest set** — a genuine theory chapter with *numbered, derived* equations (incoherence Eq 79/80, Simandoux Eq 78 with full historical derivation, Dual-Water, Indonesian/Nigerian, Linear), parameter tables (ersh/swshe/mc2, EVCL/MVCL, mdw), uncertainty/weight methodology, and literature references (Simandoux 1963, Schlumberger 1972, Worthington 1985, Poupon 1967).
- Rw/temperature initialization; hydrocarbon correction.

**How-to / UI only (no standalone equation — cite the parent method page for math):**
- `quanti-saturation-methods.html`, `quanti-shale-volume.html`, `petrophysics-quanti-permeability.html` and the other landing/index pages.
- `petrophysics-shale-volume.html` (combined method) — documents only the *merge statistics* (arithmetic/geometric/harmonic/median/min/max/first-present/product/sum) and defers per-method math to the individual Vsh pages.
- Pickett-plot pages — graphical parameter-picking workflow only.
- `petrophysics-environmental-corrections.html` / Schlumberger env-corr — mostly per-tool correction charts + how-to, not closed-form equations.
- Most `petrophysics-quantielan-workflow/-postprocessing/-utilities` — module operation, not theory.
- `petrophysics-inversion.html` is CMR/NMR T2 inversion (not the ELAN mineral solver).

**Practical citation guidance for SandiBumi:** For any deterministic method, cite `concept\petrophysics-<method>.html` + the specific equation image filename in `image\` (both listed above). Symbols/units/defaults are extractable as text from the page's parameter table; the formula must be read from the referenced image. For the multi-mineral solver, `concept\petrophysics-elanplus-simandoux-conductivity-equation.html` (Eq 78) and `-solution-method.html` (Eq 79/80) are the two most authoritative, equation-complete pages. The `topic\pythonlib\<func>.html` pages give the exact argument contract (order, units, defaults) if SandiBumi wants to mirror Techlog's function signatures.
