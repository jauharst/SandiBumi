# B — Core Petrophysics (IP 2025 CHM ingest)

Agent B. Source: decompiled Interactive Petrophysics 2025 help (`c25`), read-only.
Diff baseline: IP 2018 decompiled help (`c18`) + `../ip2018_chm_ingest/` reports.
Extraction date 2026-08-06.

**Provenance convention used throughout**

- `(pagename.htm)` — fact stated in the page's prose/ASCII text.
- `[img-read: file.png]` — transcribed by reading the rasterised image directly. Every
  such transcription was unambiguous at native resolution unless listed in §8.
- `[hlp: File.hlp]` — from the IP module-parameter definition files as captured in
  `../ip2018_chm_ingest/H_module_parameter_reference.json` (the note there records these
  files are identical in IP2018 and IP2025).
- Nothing in this report is filled from textbook knowledge. Where the manual is silent,
  §8 says so.

**Delegation statement.** This ingest ran entirely on the session model. No subagent was
used at any point, in line with the standing rule that petrophysical parameters and method
math are never delegated.

---

## 1. Scope & page inventory

19 assigned pages. All 19 opened and accounted for.

| # | Page | chars | In IP2018? | Yield |
|---|---|---:|---|---|
| 1 | `swequationsandmethodology.htm` | 46,463 | yes | Primary equation source. ~55 equations recovered from images (embim38–122, `_pawsclip*`). |
| 2 | `swparameters.htm` | 60,873 | yes | ASCII restatement of #1 + all PhiSw defaults. 4 mismatches vs #1 (§5). |
| 3 | `sand_silt_malay_model.htm` | 22,280 | yes | High-priority. Complete SSC model, §2.4. |
| 4 | `clayequationsandmethodology.htm` | 4,848 | yes | **Major recovery** — all 13 clay indicators, lost in the 2018 ingest. |
| 5 | `clayparameters.htm` | 20,275 | yes | Clay defaults (1)–(72). Authoritative Clip Low/High. |
| 6 | `basicloganalysis.htm` | 22,240 | yes | BLA equations + defaults. D-01 site. |
| 7 | `basiclogcalculations.htm` | 16,862 | yes | Basic-function equations; **permeability coefficients recovered**. |
| 8 | `porosityandwatersaturation.htm` | 28,745 | yes | Module setup/logic; no new constants. |
| 9 | `swplot.htm` | 21,417 | yes | Interactive-plot semantics; parameter coupling rules (§4). D-02 site. |
| 10 | `clayplot.htm` | 7,308 | yes | Bad-hole indicator logic; organic-shale plot. |
| 11 | `clayvolume.htm` | 10,162 | yes | Module setup; auto-default caveat (§4). |
| 12 | `interpretation.htm` | 6,645 | yes | Navigation page. No constants. |
| 13 | `density_estimation.htm` | 2,972 | **NEW** | Density-from-sonic incl. Alberty Smectite/Illite. |
| 14 | `densityestimation2.htm` | 2,174 | yes | Older version of #13. |
| 15 | `co_sw_analysis.htm` | 32,657 | **NEW** | C/O + Inelastic-Ratio cased-hole Sw. No closed form published (§8). |
| 16 | `references_and_appencices.htm` | 11,723 | **NEW** | Bibliography (PPFG-scoped) + curve nomenclature. §9. |
| 17 | `temperaturegradient.htm` | 1,847 | yes | Gradient in deg/100 ft or /100 m; F/C output flag. |
| 18 | `rwfromsp.htm` | 1,575 | yes | Schlumberger chart SP-2; NaCl assumption. |
| 19 | `bla-load.htm` | 162 | **NEW** | Stub (162 chars). No content. |

Counts: **≈129 equations** transcribed (≈96 from images), **≈150 defaults / constants /
coefficients**, **11 internal discrepancies** (3 reopened from 2018 + 8 new), **13 OPEN
items** (one more was opened and closed within this pass).

---

## 2. Equations & methods

### 2.1 Clay volume — single indicators

All from `clayequationsandmethodology.htm`, images read at native resolution.

Linear gamma ray — `[img-read: embim22.png]`

```
VclGr = (Gr − GrClean) / (GrClay − GRClean)
```

Everything below uses `Z` = this linear GR index.

Curved (three-branch) — `[img-read: embim23.png]`, `[img-read: embim24.png]`

```
Z < 0.55          : VclGr = 0.0006078 × (100.0 × Z)^1.58527
0.55 < Z < 0.73   : VclGr = 2.1212 × Z − 0.81667
0.73 < Z < 1.0    : VclGr = Z
```

Clavier — `[img-read: embim25.png]`

```
VclGr = 1.7 − sqrt( 3.38 − (Z + 0.7)^2 )
```

Stieber — `[img-read: embim26.png]`, confirmed verbatim by `[hlp: ClayVol.hlp]`

```
VclGr = Z / (1 + STB (1 − Z))          STB default 2.0
```

Larionov, older rocks (Mesozoic) — `[img-read: embim27.png]`

```
VclGr = 0.333 × (2^(2 × Z) − 1)
```

Larionov, younger rocks (Tertiary clastics) — `[img-read: embim28.png]`

```
VclGr = 0.08336 × (2^(3.7 × Z) − 1)
```

SP — `[img-read: embim29.png]`

```
VclSP = (SP − SPClean) / (SPClay − SPClean)
```

Neutron — `[img-read: embim30.png]`

```
VclNeu = sqrt( (PhiNeu / PhiNeuClay) × ((PhiNeu − PhiNeuClean) / (PhiNeuClay − PhiNeuClean)) )
```

Resistivity — `[img-read: embim31.png]`, `[img-read: embim32.png]`

```
Z = (Rclay / Rt) × ((Rclean − Rt) / (Rclean − Rclay))
Rt > 2 × Rclay : VclRes = 0.5 × (2 × Z)^(0.67 × (Z + 1))
otherwise      : VclRes = Z
```

Other (generic linear) — `[img-read: embim33.png]`

```
Vcl = (LogCurve − LogClean) / (LogClay − LogClean)
```

### 2.2 Clay volume — double indicators

Neutron/Density — `[img-read: embim34.png]`

```
VclND = [ (DenCl2 − DenCl1)(Neu − NeuCl1) − (Den − DenCl1)(NeuCl2 − NeuCl1) ]
      / [ (DenCl2 − DenCl1)(NeuClay − NeuCl1) − (DenClay − DenCl1)(NeuCl2 − NeuCl1) ]
```

`Cl1`/`Cl2` are the two points defining the clean line; `Clay` is the clay point.
Sonic/Density `[img-read: embim35.png]`, Neutron/Sonic `[img-read: embim36.png]` and
Other/Double `[img-read: embim37.png]` are the identical bilinear form with the
corresponding curves substituted. This is a 2-point clean **line** plus a single clay
**point**, not a triangle — a real geometric difference from a Thomas-Stieber layout.

### 2.3 Organic-shale corrections (clay module)

Gamma ray, neutron, density — `(clayequationsandmethodology.htm)`, confirmed by
`[hlp: ClayVol.hlp]` for the GR form:

```
GrCorr  = Gr_in  − TOCvol × Gr_Kerogen
NeuCorr = Neu_in − TOCvol × Neu_Kerogen − HvyMinVol × Neu_HeavyMin
DenCorr = Den_in − TOCvol × Rhob_Kerogen − HvyMinVol × Rhob_HeavyMin
TOCvol     = TOC_in     × Kerogen_Wt%Con
HvyMinVol  = HeavyMin_in × HeavyMin_Wt%Con
```

Sonic — `[img-read: _cvclip0061.png]`

```
Dt_corr = ( Dt − Vol_TOC × TOC_corrfac × DT_kerr − Vol_hvy × Hvy_corrfac × DT_hvy )
        / ( 1  − Vol_TOC × TOC_corrfac          − Vol_hvy × Hvy_corrfac )
```

Note the sonic correction renormalises by the removed volume; the GR/neutron/density
corrections do not. That asymmetry is in the manual as written.

### 2.4 Sand / Silt / Malay model (high priority)

Source `sand_silt_malay_model.htm`. Stated purpose: very fine-grained sediments with
fresh-to-brackish formation water, where 1980s shaly-sand models over-estimate clay volume
and therefore under-estimate porosity. Curve shapes on the Lithology Conversion Chart were
derived from **Malay Basin core data**; the "Clay at Silt Point" parameter is point B on
that chart and point A = 1.0 − B.

Coal logic (applied first, hard override):

```
IF Density < DenCoal AND Neutron > NeuCoal THEN
    PhiT = Phie = Vcl = Vshale = Vquartz = Vsilt = Vdclay = 0
    SwT = SxoT = Sw = Sxo = 1.0
    Vcoal = 1.0
```

Shale volume from GR — `[img-read: _ssmclip0070.png]`

```
VshGr = (Gr − GrClean) / (GrShale − GrClean)
```

Hydrocarbon properties — `[img-read: _ssmclip0056.png]`, `[img-read: _ssmclip0057.png]`,
`[img-read: _ssmclip0058.png]`

```
NeuHydHI   = 9 × Rho_hden × (4 − 2.5 × Rho_hden) / (16 − 2.5 × Rho_hden)
DenHydApp  = (5.5 × Rho_hden × (4 − Rho_hden) − 3) / (16 − 2.5 × Rho_hden)
Rho_hden   = input hydrocarbon density
```

These are the **Modified** hydrocarbon-density branch. Three-way agreement with
`[img-read: _pawsclip0110.png]` / `[img-read: _pawsclip0112.png]` on the PhiSw page and the
`swparameters.htm` prose.

Hydrocarbon corrections — `(sand_silt_malay_model.htm)`

```
DenCorr = Rhob + Phie × (1.0 − Sxo) × (Rhofl − DenHydApp)
NeuCorr = PhiNeu + exfact + Phie × (1.0 − Sxo) × (1.0 − NeuHydHI)
exfact  = sqrt(Rhoma / 2.65) × (2.0 × SwH × Phie^2 + 0.04 × Phie) × (1.0 − SwH)
SwH     = Sxo + (1 − Sxo) × NeuHydHI
```

Shale volume from neutron-density — `[img-read: _ssmclip0059.png]`

```
VshND = [ (Den_fl − Den_mat)(Neu_Corr − Neu_mat) − (Den_corr − Den_mat)(Neu_fl − Neu_mat) ]
      / [ (Den_fl − Den_mat)(Neu_WetClay − Neu_mat) − (Den_WetClay − Den_mat)(Neu_fl − Neu_mat) ]
```

Matrix, porosity, volumes — `(sand_silt_malay_model.htm)`

```
Rhoma     = Fsn × Denmat + Fsi × Densilt + Fdc × Dendcl
PhiT      = (Rhoma − DenCorr) / (Rhoma − Denfl)
Vsand     = Fsn × (1 − PhiT)
Vsilt     = Fsi × (1 − PhiT)
Vdcl      = Fdc × (1 − PhiT)
PhiTclay  = (Dendcl − Denwcl) / (Dendcl − Denfl)
Vcl       = Vdcl / (1 − PhiTclay)
Vshale    = Vcl + Vsilt
```

`Fsn`, `Fsi`, `Fdc` are the sand / silt / dry-clay fractions read off the Lithology
Conversion Chart. **The chart's numeric node table is not published in the help** — see §8.

Effective porosity — three forms are given, and the module uses the **combined** one:

```
(a) Phie = PhiT − Vcl × PhiTclay
(b) Phie = PhiT − Vcl × PhiT
(combined) Phie = (1 − PhiT) × (PhiT − Vcl × PhiTclay) + Vcl × (PhiT − Vcl × PhiT)
```

Limits and bound water:

```
Phie <= MaxPhie × (1.0 − Vcl)
IF Vshale > Vshale_Cutoff THEN Phie = 0
Vbw = PhiT − Phie
IF Vbw > Vcl × PhiTclay × 1.5 THEN
    Vbw  = Vcl × PhiTclay × 1.5
    PhiT = Phie + Vbw
```

The 1.5 factor is an explicit cap on bound-water volume relative to the clay's own total
porosity. It is a hard-coded constant in the model, not a parameter.

Saturation — `[img-read: _ssmclip0062.png]` … `[img-read: _ssmclip0068.png]`, plus
`[img-read: _ssmclip0071.png]`

```
Archie          : 1/Rt = PhiT^m × Sw^n / (a × Rw)
Dual Water      : 1/Rt = (PhiT^m × SwT^n / a) × ( 1/Rw + (Swb/SwT)(1/Rwb − 1/Rw) )
                  Swb  = 1.0 − Phie/PhiT
Juhasz          : 1/Rt = (PhiT^m × SwT^n)/(a × Rw) × (1 + Bn × Qvn × Rw / SwT)
                  Qvn  = (Vcl × PhiTclay) / PhiT
Waxman-Smits    : 1/Rt = (PhiT^m × SwT^n)/(a × Rw) × (1 + B × Qv × Rw / SwT)
                  Qv   = a / PhiT + b
B (Juhasz form) : B = (−1.28 + 0.225 × T − 0.0004059 × T^2)
                      / (1 + Rw^1.23 × (0.045 × T − 0.27))
Sw  = (SwT − Swb) / (1 − Swb)
Sxo = (Sw + InvasionFactor) / (1 + InvasionFactor)
```

The `B` image on this page is clean and **brackets balance** — this is the authoritative
rendering (see §5, D-08).

OBM branch and iteration:

```
OBM: Sxo  = Invasion Factor
     SxoT = Sxo × (1 − Swb) + Swb
Convergence: |ΔPhiT| < 0.0001 ; |ΔSxoT| < 0.001
```

Diagnostic outputs:

```
Rwapp   = PhiT^m × Rt / a
CwApp   = 1.0 / Rwapp
QvNorm  = Vcl × PhiTclay / PhiT
QvApp   = a / (B × Rt × PhiT^m) − 1.0 / (B × Rw)
RecPhiT = 1.0 / PhiT
```

Logic Flag (SSM) — `(sand_silt_malay_model.htm)`

| Flag | Meaning |
|---|---|
| 0 | OK |
| 1 | VshGr ≠ VshND |
| 2 | main hydrocarbon loop hit max 40 iterations |
| 3 | Sxo loop hit max 100 |
| 4 | Sw loop failed to converge after 10 |
| 5 | bad-hole loop hit max 100 |
| 6 | bad-hole logic used |
| 7 | PhieMax limit used |
| 8 | Vshale cutoff used |
| 9 | null inputs |

### 2.5 Porosity models (PhiSw module)

Density porosity, with clay and dual-fluid terms — `[img-read: embim43.png]`
(= `[img-read: embim55.png]`)

```
Phi = ( Rho_ma − Rho_b − Vcl × (Rho_ma − Rho_cl) )
    / ( Rho_ma − Rho_fl × Sxo − Rho_HyAp × (1 − Sxo) )
```

Bulk-density back-calculation — `[img-read: embim6.png]`

```
Rhob = Rho_ma × (1 − Phi) + Rho_Fluid × Phi
```

Neutron porosity — `[img-read: embim49.png]` (= `embim54`)

```
Phi = ( PhiNeu − Vcl × NeuCl + NeuMatrix + Exfact + NeuSal )
    / ( Sxo + (1 − Sxo) × NeuHyHI )
```

Excavation-effect term — `[img-read: embim50.png]`

```
Exfact = (Rho_ma / 2.65)^2 × (2 × Swx × Phi_x^2 + 0.04 × Phi_x) × (1 − Swx)
Phi_x  = Phi + Vcl × NeuCl
Swx    = ( Phi × (Sxo + (1 − Sxo) × NeuHyHI) + Vcl × NeuCl ) / Phi_x
```

Note the exponent is **2** on `(Rho_ma/2.65)` here, whereas the SSM page's `exfact` uses
`sqrt(Rhoma/2.65)`. Both were read at native resolution; see §5, D-09.

Sonic porosity, Wyllie — `[img-read: embim52.png]`

```
Phi = ( Dt − Dtma − Vcl × (Dtcl − Dtma) )
    / ( ( Dtfl × Sxo + Dthy × (1 − Sxo) − Dtma ) × Cp )
```

Sonic porosity, Raymer — `[img-read: embim53.png]`

```
Phi_clay = [ (2 Vma − Vf) − sqrt( (2 Vma − Vf)^2 − 4 Vma (Vma − Vclay) ) ] / (2 Vma)
Vfc      = 1 / ( Dtfl × Sxo + Dthy × (1 − Sxo) )
Phi_son  = [ (2 Vma − Vfc) − sqrt( (2 Vma − Vfc)^2 − 4 Vma (Vma − Vlog) ) ] / (2 Vma)
Phi      = Phi_son − Phi_clay × Vcl
```

Neutron-density crossplot porosity, variable matrix — `[img-read: embim58.png]`

```
Phi = Phi_D1 + (Phi_N1 − Phi_D1) / ( 1 − (Phi_N1 − Phi_N2)/(Phi_D1 − Phi_D2) )
```

`N1/N2` = neutron porosity for matrix 1/2, `D1/D2` = density porosity for matrix 1/2. For
the Sand/Limestone/Dolomite model IP first decides from the matrix density whether to use
the Sand-Limestone or Limestone-Dolomite pair, then applies this. Matrix density is
back-solved from the density equation afterwards.

Variable-Sxo logic — `[img-read: _pawsclip0143_zoom80.png]`: the neutron and density
porosity equations above are solved **simultaneously** for `Phi` and `Sxo`.
Variable-hydrocarbon-density logic solves the same pair for `Phi` and `Rho_hden` using
`[img-read: embim56.png]` / `[img-read: embim57.png]`:

```
NeuHyHI  = 9 × Rho_hden × (4 − 2.5 × Rho_hden) / (16 − 2.5 × Rho_hden)
Rho_HyAp = 2 × Rho_hden × (10 − 2.5 × Rho_hden) / (16 − 2.5 × Rho_hden)   [Conventional]
```

Variable-clay-volume logic solves the identical pair for `Phi` and `Vcl` with hydrocarbon
density known.

Pass-through porosity — `[img-read: embim66.png]`, `[img-read: embim63.png]`,
`[img-read: embim64.png]`, `[img-read: embim65.png]`

```
Phie = Phi_input − Vcl × PhiTClay      (when input is PhiT)
PhiT = Phi_input                        (input already total)
PhiT = Phie + Vcl × PhiTClay            (when input is Phie)
```

Total porosity and bound water — `[img-read: embim73.png]`, `[img-read: embim78.png]`,
`[img-read: embim79.png]`

```
PhiTclay = (Rho_dryClay − Rho_wetClay) / (Rho_dryClay − Rho_fl)
PhiT     = Phie + Vcl × PhiTclay
Swb      = 1 − Phie / PhiT
```

Porosity limits / bad hole — `[img-read: embim69.png]`, `[img-read: embim71.png]`,
`[img-read: embim72.png]`

```
Phi <= Phi_sonic                                    (bad-hole branch)
Phi_limit = (PhiMax + DeltaPhiMax) × (1 − Vcl) × 10^( −10 × (Vcl − VclCutoff)^1.6 )
Phi <= Phi_limit
```

Above the shale cutoff the shale-zone limit collapses to
`Phi_limit = (PhiMax + DeltaPhiMax) × (1.0 − Vcl)` `(swequationsandmethodology.htm)`.
The rasterised form settles the malformed ASCII version on `swparameters.htm` (§5, D-11).

Organic-shale porosity (PhiSw module) — `[img-read: _pawsclip0134.png]` …
`[img-read: _pawsclip0138.png]`, `[img-read: _pawsclip0149.png]`

```
Rho_dry_rock = [ (1 − Vcl_dry − PhiT − Vker − Vhvy) Rho_ma
                 + Vcl_dry Rho_clay + Vker Rho_ker + Vhvy Rho_hvy ] / (1 − PhiT)
Vker = TOCin  × (Rho_dry_rock / Rho_ker) × (1 − PhiT)
Vhvy = VhvyIn × (Rho_dry_rock / Rho_hvy) × (1 − PhiT)

Phi (density) = ( Rho_ma − Rho_b − Vcl(Rho_ma − Rho_cl)
                  − Vker(Rho_ma − Rho_ker) − Vhvy(Rho_ma − Rho_hvy) )
              / ( Rho_ma − Rho_fl × Sxo − Rho_HyAp × (1 − Sxo) )

Phi (neutron) = ( PhiNeu − Vcl·NeuCl − Vker·NeuKer − Vhvy·NeuHvy
                  + NeuMatrix + Exfact + NeuSal ) / ( Sxo + (1 − Sxo) NeuHyHI )

Phie (sonic)  = ( Dt − DT_ma − Vcl(DT_clay − DT_ma) − Vol_toc(DT_kerr − DT_ma)
                  − Vol_hvy(DT_hvy − DT_ma) )
              / ( (DT_fl × Sxo + DT_hy × (1 − Sxo)) × Cp )
```

`Vker`/`Vhvy` are self-referential through `Rho_dry_rock`; the manual does not state the
iteration order for this sub-loop (§8).

### 2.6 Saturation equations (PhiSw module)

All from `swequationsandmethodology.htm`, images read at native resolution; the
higher-risk ones (`embim114`, `embim118`, `embim119`, `embim120`) were re-read at 3× and 6×
magnification.

```
Archie             [img-read: embim103.png]
  1/Rt = Phi^m × Sw^n / (a × Rw)
                   [img-read: embim104.png]
  Sw   = ( a × Rw / (Rt × Phie^m) )^(1/n)
                   [img-read: embim105.png]
  SwT  = Sw (1 − Swb) + Swb

Archie PhiT        [img-read: embim106.png / embim107.png / embim108.png]
  same with PhiT ; Sw = (SwT − Swb)/(1 − Swb)

Simandoux          [img-read: embim109.png]
  1/Rt = Phi^m × Sw^n / (a × Rw) + Vcl × Sw / Rcl

Modified Simandoux [img-read: embim110.png]
  1/Rt = Phi^m × Sw^n / (a × Rw × (1 − Vcl)) + Vcl × Sw / Rcl

Indonesian         [img-read: embim111.png]
  1/sqrt(Rt) = ( sqrt(Phi^m/(a × Rw)) + Vcl^(1 − Vcl/2) / sqrt(Rcl) ) × Sw^(n/2)

Woodhouse Tar      [img-read: embim112.png]
  1/Rt = Sw^n × ( Phi^(m/2)/sqrt(a·Rw) + Vcl^(1 − Vcl)/sqrt(Rcl) )^2

Dual Water         [img-read: embim113.png]
  1/Rt = (PhiT^(m*) × SwT^n / a) × ( 1/Rw + (Swb/SwT)(1/Rwb − 1/Rw) )

Juhasz             [img-read: embim115.png]
  1/Rt = (PhiT^m × SwT^n)/(a × Rw) × (1 + Bn × Qvn × Rw / SwT)
  Qvn  = Vcl × PhiTclay / PhiT

Waxman-Smits       [img-read: embim116.png]
  1/Rt = (PhiT^(m*) × SwT^n)/(a × Rw) × (1 + B × Qv × Rw / SwT)
  Qv   = a / PhiT + b                              [img-read: embim117.png]
```

Note the exponent difference: **Juhasz uses `m`, Waxman-Smits and Dual Water use `m*`.**

Variable `m*` — `[img-read: embim114.png]` (Dual Water), `[img-read: embim119.png]` (W&S),
both verified at 6×:

```
Dual Water   : m* = m_input + Cm ( 0.258 × Y + 0.20 (1 − e^(−16.4 × Y)) )
Waxman-Smits : m* = m_input + Cm ( 1.128 × Y + 0.22 (1 − e^(−17.3 × Y)) )
Y            = Qv × PhiT / (1 − PhiT)
```

`B` coefficient (Juhász form) — `[img-read: embim118.png]`

```
B = (−1.28 + 0.225 × T − 0.0004059 × T^2) / (1 + Rw^1.23 (0.045 × T − 0.27))
```

This image's brackets do **not** balance (one `(`, two `)`); the SSM page's clean rendering
resolves it — §5, D-08. `T` is in degF (implied by the SSM page's temperature handling; not
stated on this page — §8).

Shell variable-`m` — `[img-read: embim120.png]`, verified at 6×

```
m = 1.87 + 0.018 / Phie
```

The prose everywhere says `0.019`. This is D-10 and is **NOT resolved**.

`m` varying with clay — `[img-read: embim121.png]`

```
m = m × 10^(Vcl − VclCutoff)
```

Bound-water back-out — `[img-read: embim122.png]`

```
Sw = (SwT − Swb) / (1 − Swb)
```

### 2.7 Laminated / thin-bed / tensor resistivity

`Rt Lam` model "Normal" — `(swequationsandmethodology.htm)`

```
RtLam = (1.0 − Vlam) × Rt × Rshale / (Rshale − Rt × Vlam)      clamped at 2000 ohmm
SwT   = 1.0 − PhiTLam × (1.0 − SwTLam) × (1 − Vlam) / PhiT
```

`Rt Lam` model "Tensor" — `[img-read: _pawsclip0114.png]`, `[img-read: _pawsclip0115.png]`

```
Rv   = (1 − Vlam) × Rsd + Vlam × RshVert
1/Rh = (1 − Vlam)/Rsd + Vlam/RshHori
```

Three solution methods:

- **Tensor Vlam** — solve for `Rsand`, `RshVert`, `RshHori`. Inputs: `Vlam` from
  Thomas-Stieber and the anisotropy ratio `RshVert/RshHori`. Two roots exist
  (`Rsand < Rshale` and `Rsand > Rshale`); the `Res Lam Shale` parameter selects which.
  **The user must pick the root manually — IP does not.** If the anisotropy ratio is too
  high for a solution, IP reduces it until one exists and sets `PhiFlag = 15`.
- **Tensor Rsh** — solve for `Rsand` and `Vlam` from input `RshVert`, `RshHori`. Outputs
  `VlamTensor`.
- **Tensor Rsh Mod** — as above, then override Thomas-Stieber shale typing:

```
Vlam   = VlamTensor,  subject to  Vlam <= Vwcl
Vdisp  = PhiMax × (1 − Vlam) − Phie
Vstruc = Vwclay − Vlam − Vdisp
```

Other shale-distribution outputs `(swequationsandmethodology.htm)`:

```
Vshale = VWCL / CSR
Vfines = VWCL + Vsilt
VSILT  = 1 − VolwetClay − (Phie / PhiMax)          (swparameters.htm)
```

Stated `Vlam` limits: 0 – 0.99.

### 2.8 Multi-mineral (U-based) analysis

`(swequationsandmethodology.htm)`, with `[img-read: embim82.png]` = `Rho_b`:

```
U     = Pef × (Rho_b + 0.1883) × 0.93423
Uwat  = 0.00481 × Sal + 0.3883
Uhyd  = 0.119 × Rho_hyd        (gas, Rho < 0.4)
Uhyd  = 0.133 × Rho_hyd        (oil)

U_matrixApp = ( U − Phi_NDxp × U_water ) / (1.0 − Phi_NDxp)   [img-read: embim7.png]
Grain Den   = (Vmatrix·Rhoma + Vdcl·RhoDclay + VExtraMins·RhoExtraMins)
              / (Vmatrix + Vdcl + VExtraMins)
```

The identical `U = Pef × (RHOB + 0.1883) × 0.93423` appears independently on
`(basiclogcalculations.htm)`.

### 2.9 EPT / TPL water propagation time

`[img-read: embim40.png]`, `[img-read: embim41.png]`, `[img-read: embim42.png]`

```
E'  = (79.4 − 202.69 × Sal) × ( 1 − 0.385 × (T − 75) × (3230.0 − T) × 10^−6 )^2
E'' = 4558 / T^1.568 + 16.34 / Rmf
TPW = 2.3586 × sqrt( sqrt(E'^2 + E''^2) + E' )
```

`Sal` in ppm and `Rho` in gm/cc per `[img-read: embim38.png]` / `[img-read: embim39.png]`.
The `(T − 75)` reference and the constant 3230.0 imply degF; not stated (§8).

### 2.10 Basic Log Analysis (BLA)

`basicloganalysis.htm`.

```
Vcl  [img-read: embim11.png] = (Gr_Log − Gr_Clean)/(Gr_Clay − Gr_Clean)
Phi (density)  [img-read: embim12.png]
     = ( Rho_matrix − Rho_b − Vcl × (Rho_matrix − Rho_clay) ) / (Rho_matrix − Rho_fluid)
Phi (Wyllie)   [img-read: embim13.png]
     = ( Dt − Dt_matrix − Vcl × (Dt_Clay − Dt_matrix) ) / ( (Dt_Fluid − Dt_matrix) × Cp )
Phi (Raymer)   [img-read: embim15.png]  quadratic in Phi_clay / Phi_son, then
     Phi = Phi_son − Phi_clay × Vcl
Rwa  [img-read: embim16.png] = Phi^m × Rt / a
SwU  [img-read: embim17.png] = ( a × Rw / (Phi^m × Rt) )^(1/n)
Simandoux  [img-read: embim18.png] : 1/Rt = Phi^m·SwU^n/(a·Rw) + Vcl·SwU/Rcl
Indonesian [img-read: embim19.png] :
     1/sqrt(Rt) = ( sqrt(Phi^m/(a·Rw)) + Vcl^(1 − Vcl/2)/sqrt(Rcl) ) × SwU^(n/2)
Rt_Hingle  = Rt^(−1/m)     ; Rxo_Hingle = Rxo^(−1/m)
```

`embim14.png` is a **blank spacer image**, not a dropped equation — verified by reading it.

### 2.11 Basic Log Calculations (utility functions)

`basiclogcalculations.htm`.

```
Phi (density)  [img-read: embim5.png]  = (Rho_ma − Rhob) / (Rho_ma − Rho_Fluid)
Rhob           [img-read: embim6.png]  = Rho_ma (1 − Phi) + Rho_Fluid × Phi
Phi (Wyllie)   [img-read: _ccclip1818.png] = (DT − DT_matrix) / ((DT_fluid − DT_matrix) × Cp)
Phi (Raymer)   [img-read: _ccclip1819.png]
   = [ (2 Vma − Vf) − sqrt( (2 Vma − Vf)^2 − 4 Vma (Vma − Vlog) ) ] / (2 Vma)

M / N lithology ratios  [img-read: embim8.png]
   'M' = ( (DT_fluid − DT) / (Rhob − Rho_fluid) ) × 0.01
   'N' = (1 − Phi_neu)   / (Rhob − Rho_fluid)

Formation factor / saturation  [img-read: embim9.png]
   FF     = a / Phi^m
   Rwapp  = Rt  / FF
   Rmfapp = Rxo / FF
   Sw     = (FF × Rw  / Rt )^(1/n)
   Sxo    = (FF × Rmf / Rxo)^(1/n)

NMR gas T1  [img-read: embim10.png]
   T1_gas = 25 000 000.0 × MethaneDensity / (Temp(C) + 273.15)^1.17

Misc  (basiclogcalculations.htm)
   U        = Pef × (RHOB + 0.1883) × 0.93423
   Velocity = 1 / DT
   Caliper  = DCal + Bit Size
   Ct       = 1000 / Rt      ;  Cxo = 1000 / Rxo
   HI_Gas   = 2.25 × GasDensity
```

Permeability — `[img-read: _ccclip1769.png]`

```
K = a × Phi^b / Swi^c
```

with **`Phi` and `Swi` both in decimal fractions** (the input panel labels both "(dec)").
`Swi` is irreducible water saturation. Default coefficient sets `[img-read: _ccclip1770.png]`:

| Model | a | b | c | Chart source |
|---|---:|---:|---:|---|
| Timur | 8581 | 4.4 | 2 | Western Atlas chartbook |
| Morris Biggs oil | 62500 | 6 | 2 | Western Atlas chartbook |
| Morris Biggs gas | 6241 | 6 | 2 | Western Atlas chartbook |
| Schlumberger Chart K3 | 10000 | 4.5 | 2 | Schlumberger chartbook |

These coefficients were **not recovered in the 2018 ingest** (they live only in these two
screenshots). Sanity check on the decimal-fraction convention: Morris Biggs oil 62500 = 250²
and gas 6241 = 79², i.e. exactly the squares of the coefficients in the familiar
`sqrt(K) = C·Phi³/Swi` forms under decimal `Phi`; Timur 8581 likewise squares back to ~92.6.
The unit convention on the panel is therefore self-consistent.

### 2.12 Density estimation from sonic

`density_estimation.htm` (new page) and `densityestimation2.htm`. Models offered:
Gardner, Gardner Generalised, AGIP (Bellotti), Lindseth, and — **new in IP2025** —
Alberty Smectite / Illite. Defaults `[img-read: denew_clip0003.png]`:

| Parameter | Default |
|---|---|
| Gardner PowerLaw `a` | 0.23 |
| Gardner PowerLaw `b` | 0.25 |
| Alberty S/I `T begin` | 160 degF |
| Alberty S/I `T end` | 220 degF |

The Alberty smectite→illite transition window (160–220 degF) is the model's diagenetic
control. Bibliography entries: Alberty 2003 (OTC 15290), Alberty 2005 (SPE 108787),
Alberty & Reilly 2018, Gardner et al. 1974, Bellotti et al. 1979, Lindseth 1979.

### 2.13 C/O and Inelastic-Ratio cased-hole saturation

`co_sw_analysis.htm` (new page). Method: build two model lines against porosity — `COO`
(100 % oil) and `COW` (100 % water) — for the tool and lithology, then read `Sw` by
**linear interpolation between them at the level's porosity**. Default model lines are
supplied for both C/O and Inelastic-Ratio tools in Sandstone, Limestone and Dolomite.
The model lines are meant to be adjusted until the oil leg reads a realistic `Sw`.

**No closed-form Sw equation and no default `COO`/`COW` coefficients are published** — see
§8. Do not reconstruct one.

### 2.14 Temperature gradient and Rw from SP

`temperaturegradient.htm`: gradient entered in degrees per 100 ft or per 100 m depending on
well units; requires a reference depth + temperature; alternatively interpolates between
fixed temperature points. Output curve carries an explicit F/C unit flag which downstream
interpretation modules read to do the conversions.

`rwfromsp.htm`: requires a baseline-shifted SP with the **shale baseline set to 0.0 mV** and
a formation-temperature curve. Result is corrected to a user-specified output temperature.
Optional salinity output in Kppm NaCl equivalent. Two water models: NaCl Formation Waters,
or Average Fresh Formation Waters (which uses the **dashed line on Schlumberger chart SP-2**
to go Rweq → Rw). Stated assumption: predominantly sodium-chloride waters.

---

## 3. Parameters, defaults & constraints

### 3.1 Clay Volume module (`clayparameters.htm`, numbering (1)–(72))

| # | Parameter | Default | Units / range |
|---|---|---|---|
| (55) | Use Percentiles | — | flag; unavailable when organic-shale corrections are on `[hlp]` |
| (56) | Percentile Clean | **not stated** | %; negatives allowed, linear-extrapolated below 0 % `[hlp]` |
| (57) | Percentile Clay | 130 % | %; >100 % normal, linear-extrapolated above 100 % |
| (58) | Percentile Group | — | integer group id |
| (59) | Clip Low % | **0 %** | enter 0–100 % |
| (60) | Clip High % | **98 %** | enter 0–100 % |
| (61) | Stieber Constant | 2.0 | shape control in `Z/(1+STB(1−Z))` |
| (62) | Organic Shale Corr. | off | flag |
| (63) | Gr Kerogen | not stated | API of 100 % kerogen; set 0 if input is U-corrected (CGR) |
| (64) | Rhob Kerogen | 1.1 | gm/cc |
| (65) | Nphi Kerogen | 0.6 | v/v |
| (71) | Sonic Kerogen | 150 | uS/ft — **new in IP2025** |
| (66) | Rhob Heavy_Min. | 4.3 | gm/cc (mainly pyrite) |
| (67) | Nphi Heavy_Min. | −0.03 | v/v |
| (72) | Sonic Heavy_Min. | 40 | uS/ft — **new in IP2025** |
| (68) | Kerogen Wt%_Con. | 2.5 | wt% → volume; set 1.0 if input already volume |
| (69) | Heavy_Min. Wt%_Con. | 1.0 | wt% → volume |
| (70) | Link Clean Paras | — | flag `[hlp: ClayVol.hlp]` |
| (45)–(50) | BadH1/BadH2 Use / Min / Max | blank | blank = discriminator ignored |

Bad-hole semantics `(clayparameters.htm, clayplot.htm)`: double clay indicators are
switched **off** when the indicator curve is **greater than** `BadHn Min` **or less than**
`BadHn Max`. Using one caliper in both slots catches washout and closed-tool intervals.

### 3.2 PhiSw module (`swparameters.htm`, cross-checked `[hlp: PhiSw.hlp]`)

| Parameter | Default | Units | Cross-check |
|---|---|---|---|
| Rho Dry Clay | 2.78 | gm/cc | 2018 htm agrees |
| Hc Den | 1.0 if blank | gm/cc | `[hlp]` agrees |
| Hc Den Min / Rho GD min / Rho GD max | — | clip limits on variable outputs | `swplot.htm` |
| PhiShr limit | 0.02 | v/v | `[hlp]` agrees |
| Rw fallback | 0.1 @ 60 degF | ohmm | |
| B fact Juhasz | 1.0 | meq/ml | `[hlp]` agrees |
| B fact W-S | — | meq/ml | |
| Cm* | 1.0 | — | `[hlp]` present |
| Rho / Neu / Sonic / U Kerogen | 1.1 / 0.6 / 150 / 0.264 | gm/cc, v/v, uS/ft, b/cc | |
| Rho / Neu / Sonic / U Heavy Min | 4.3 / −0.03 / 40 / 77 | gm/cc, v/v, uS/ft, b/cc | |
| TP Lime / Sand / Dol / Clay | 9.1 / 7.2 / 8.7 / 8.0 | nsec/m; Clay range 7–16 | `[hlp]` agrees on integer part |
| TP Hc | 3.3 gas; 4.7–5.2 oil | nsec/m | |
| Sonic Lime | 49 (160 uS/m) | uS/ft | `[hlp]` agrees |
| **Sonic Sand** | **56** (page says "180 uS/m") | uS/ft | `[hlp]` agrees on 56 → see D-06 |
| Sonic Dol | 44 (145 uS/m) | uS/ft | `[hlp]` agrees |
| Sonic water | 189 (620 uS/m) | uS/ft | `[hlp]` agrees |
| Sonic Cp | 1.0 | — | `[hlp]` agrees |
| Vlam | 0 – 0.99 | v/v | limit |
| Neutron look-up node grid | 0, 2, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60 | p.u. | table format |
| Neutron salinity columns | 50 / 100 / 150 / 200 / 250 | Kppm | order: Sandstone, Limestone, Dolomite |

Derived-parameter relations `(swparameters.htm)`:

```
Rho Wet Clay = Rho Matrix + ( (Rho Shale − Rho Matrix) / CSR )
   (and the identical form for Neutron and Sonic wet-clay values)
Sxo < Sw^SxoLimit
m   = m × 10^(Vcl − VclCutoff)
```

Iteration tolerances `(swequationsandmethodology.htm)`: main porosity loop `|ΔPhi| < 0.001`;
Sxo loop `|ΔSxo| < 0.002`. (The SSM module uses tighter values — 0.0001 / 0.001.)

### 3.3 Basic Log Analysis defaults (`basicloganalysis.htm`)

| Parameter | Default |
|---|---|
| Rho Matrix | 2.65 (2.71 for limestone) |
| Rho Fluid | 1.0 (1.1 for salt mud) |
| Rho Clay | picked, no default |
| DT Matrix | 55 uS/ft sandstone; 49 limestone |
| DT Fluid | 189 uS/ft (~174 salt-saturated) |
| Sonic Cp | 1.0 |
| Rw | 0.1 ohmm |
| a / m / n | 1.0 / 2.0 / 2.0 |
| Sat Equation | Archie |

Note BLA independently defaults sandstone `DT Matrix` to **55**, not 56 — relevant to D-06.

### 3.4 PHIFLAG (PhiSw) — selected codes

`(swequationsandmethodology.htm)` publishes codes 0–16. Captured here are the ones that
change the answer rather than merely annotate it: bad-hole logic invoked, porosity-limit
applied, shale-cutoff applied, iteration non-convergence, and **15 = anisotropy ratio was
automatically reduced in the Tensor Vlam solve**. A PHIFLAG > 0 interval must be reviewed;
the manual says so explicitly.

---

## 4. Assumptions & validity limits

Stated in the manual, in the manual's own terms:

1. **Permeability transforms** (Timur, Morris Biggs, Schlumberger K3) are applicable only
   where the zone is at **irreducible water saturation** — hydrocarbon zones above the
   transition zone. `(basiclogcalculations.htm)`
2. **Mud-resistivity conversions** (Lowe & Dunlap 1986; Overton & Lipson 1958) are valid
   only **below 70 Kppm** salinity. Lowe & Dunlap does not compute Rmc.
   `(basiclogcalculations.htm)`
3. **Rw from SP** presumes predominantly NaCl waters, and requires the SP shale baseline
   shifted to 0.0 mV. `(rwfromsp.htm)`
4. **NMR gas T1** assumes 100 % methane, density relative to air 0.554
   (Coates/Xiao/Prammer 1999). `(basiclogcalculations.htm)`
5. **Neutron input must be in limestone porosity units** for the neutron and
   neutron-density clay indicators; `clayparameters.htm` repeats this warning twice, and
   warns that setting `Neu Clean` above zero easily under-estimates clay volume.
6. **Tensor model** assumes sand-shale lamination with **anisotropic shale**, and sums
   resistivities in series (vertical) and parallel (horizontal). Root selection between the
   two mathematical solutions is left to the user.
   `(swequationsandmethodology.htm)`
7. **Auto-picked clay defaults are not interpretation picks.** `clayvolume.htm` states the
   auto-generated parameters are chosen only to put values in range — generally the maximum
   and minimum readings of the indicator, with the ND clean line defaulting to the sandstone
   line. They must be re-picked.
8. **Wet Clay / Dry Clay / CSR are a coupled triple.** `(swplot.htm)` In *Clay mode* the wet
   clay pick is honoured and the shale point is recomputed from CSR; in *Shale mode* the
   shale pick is honoured and the wet clay point is recomputed. Editing CSR moves whichever
   point is not the honoured one. Moving the dry clay point changes `PhiTclay` (when the
   parameter is left blank), which changes `PhiT`.
9. **`PhiTclay` is auto-derived from the wet/dry clay density difference** whenever the
   parameter is blank. `(swplot.htm)`
10. **SSM bound water is hard-capped** at `1.5 × Vcl × PhiTclay`.
    `(sand_silt_malay_model.htm)`
11. **SSM is scoped** to very fine-grained sediments with fresh-to-brackish formation water;
    the lithology-conversion curve shapes come from Malay Basin core.
12. **Sigma / environmental corrections** trace to Schlumberger Tcor-1/2a/3b and Western
    Atlas charts 8-4/8-5/8-6; downhole fluid densities from Batzle & Wang (1992); CO2
    properties from AGA 8 NIST (Detail and GERG). `(basiclogcalculations.htm)`
13. **Horner plot** extrapolates to Horner time 1.0. `(basiclogcalculations.htm)`

---

## 5. Internal discrepancies

### Reopened from IP2018

**D-01 — Clip Low % default (doc bug). STILL BROKEN in IP2025. Adoption unchanged.**

- `basicloganalysis.htm` (IP2025) — Clip Low %: "…low clip values between 0-100%. Default
  is (98%)." followed by the Clip-High text block about the 130 % percentile. Clip High %
  carries no default.
- `basicloganalysis.htm` (IP2018) — **character-identical** text.
- `clayparameters.htm` (IP2025) — (59) Clip Low % "Default is (0%)"; (60) Clip High %
  "Default is (98%)". Internally consistent.
- `clayparameters.htm` (IP2018) — also carries exactly one "(0%)" and one "(98%)".
- `[hlp: ClayVol.hlp]` — states **no default** for either parameter.

Conclusion: the defect is a duplicated text block on `basicloganalysis.htm` only, present
identically in both editions. The correct values are **Clip Low 0 %, Clip High 98 %**,
corroborated by two pages across two editions. The 2018 adoption stands unchanged.

**D-02 — Hingle plot Y-axis definition. RESOLVED.**

The computed output curve is defined five separate times as

```
Rt_Hingle  = Rt^(−1/m)          Rxo_Hingle = Rxo^(−1/m)
```

including in the Equations-and-Methodology sections of PhiSw, Mineral Solver and NMR. The
contradicting form `(1/Rt)^(−1/m)` appears **only** inside one boilerplate
interactive-plot paragraph that has been copy-pasted onto three pages (`basicloganalysis`,
`swplot`, and one further plot page) — and that same paragraph corrects itself two sentences
later: it states that when the Format is shown, the Y-scale values are the values actually in
the Y curve, i.e. `Rt^(−1/m)`, not `Rt`. `crossplottypes_text.txt` was cross-read for this
question and contains **no** Hingle specification.

**Adopt `Y = Rt^(−1/m)`.** The `(1/Rt)^(−1/m)` phrasing is a vendor typo, not an alternate
convention. (Note it is also the reciprocal: `(1/Rt)^(−1/m) = Rt^(+1/m)`, which would invert
the plot — so this is not a harmless wording difference.)

**D-07 — Clay-bound-water factor `F`, unbalanced brackets. Provenance SETTLED, equation
still OPEN.**

Both editions carry, byte-for-byte in the raw HTML:

```
F = 1 - [0.6425 * (Salinity ^ (-0.5) + 0.22 ] * Qv]
```

(one `[`, two `]`; two `(`, one `)`). Context: `PcCorr = Pc × F^(−0.5)`,
`SwPcCorr = 1 − (1 − SwPc) × F`, Salinity in Kppm NaCl equivalent, Qv in meq/ml, attributed
to Hill, Shirley & Klein 1979, SPWLA 20th Annual Symposium, Paper AA.

This **eliminates the "the CHM decompiler dropped a glyph" hypothesis** raised in the 2018
report: the malformation is in the vendor source and has survived two editions unchanged.
The intended bracketing cannot be inferred without either the rendered help page or SPWLA
Paper AA. **Do not repair it by pattern-matching.** Remains OPEN — see §8.

### New in this ingest

**D-08 — Waxman-Smits `B` coefficient, bracket defect. RESOLVED internally.**

`[img-read: embim118.png]` (`swequationsandmethodology.htm`) renders

```
B = (−1.28 + 0.225 × T − 0.0004059 × T^2) / (1 + Rw^1.23 (0.045 × T − 0.27))
```

with an unmatched `)` (confirmed at 3× and 6×). `[img-read: _ssmclip0071.png]`
(`sand_silt_malay_model.htm`) states the same formula with **balanced brackets and an
explicit `×`**. Numeric content is identical. **Adopt the SSM rendering.** Note the 2018
ingest never captured this formula at all — it is a net recovery.

**D-09 — Excavation-effect exponent: `(Rho_ma/2.65)^2` vs `sqrt(Rho_ma/2.65)`.**

- PhiSw: `[img-read: embim50.png]` → `Exfact = (Rho_ma/2.65)^2 × (2·Swx·Phi_x^2 + 0.04·Phi_x) × (1 − Swx)`
- SSM: `(sand_silt_malay_model.htm)` prose → `exfact = sqrt(Rhoma/2.65) × (2·SwH·Phie^2 + 0.04·Phie) × (1 − SwH)`

Both were read carefully; the PhiSw superscript `2` is unambiguous at native resolution.
These are two different modules and may legitimately differ, but the modules are otherwise
near-identical in their hydrocarbon handling. **Reported, not reconciled.** SandiBumi must
not assume one form covers both.

**D-10 — Shell variable-`m`: 0.018 (image) vs 0.019 (prose). NOT RESOLVED.**

- `[img-read: embim120.png]`, verified at 6× : `m = 1.87 + 0.018 / Phie`
- Prose `0.019 / PHIE` on IP2025 `swparameters.htm`, `mineral_solver`, seven occurrences in
  the interp-demo worked-example code, **and** IP2018 `swparameters.htm`.

Nine-plus prose statements across two editions against one raster image. The prose is the
weight of evidence, but the image is the higher-fidelity rendering of the theory section.
**Reported, not adopted.** See §8.

**D-11 — Shale-zone porosity limit: malformed in ASCII, clean in the image. RESOLVED.**

`swparameters.htm` prose:
`Phie <= (PhiMax+DeltaPhiMax)*(1.0-Vcl)*10**(-10.0*(Vcl-Vclcut-off)**1.6))` — one `)` too
many. `[img-read: embim71.png]` renders it cleanly:

```
Phi_limit = (PhiMax + DeltaPhiMax) × (1 − Vcl) × 10^( −10 × (Vcl − VclCutoff)^1.6 )
```

**Adopt the image form.**

**D-12 — Juhasz and Waxman-Smits prose omit the `Rw` factor.**

`swparameters.htm` prose gives `(1 + Bn·Qvn / SwT)` and `(1 + B·Qv / SwT)`, while
`[img-read: embim115.png]` and `[img-read: embim116.png]` give `(1 + Bn × Qvn × Rw / SwT)`
and `(1 + B × Qv × Rw / SwT)`. The SSM page's images
(`[img-read: _ssmclip0064.png]`, `[img-read: _ssmclip0065.png]`) independently carry the
`Rw` factor. This is numerically large. **Adopt the image form (with `Rw`).** The prose is
wrong.

**D-13 — Sonic Sand default: 56 uS/ft vs the stated "180 uS/m". RESOLVED — the metric
value is the error.**

`swparameters.htm` states "Default 56 (180 uSec/m)". 56 uS/ft = 183.7 uS/m; 55 uS/ft =
180.5. Every neighbouring entry converts consistently (49→160, 44→145, 189→620). Decisive
evidence: **`[hlp: PhiSw.hlp]` independently states 56** — a different vendor file from the
CHM. **Adopt 56 uS/ft**; the "(180 uS/m)" parenthetical is the defect (should read ~184).
Separately, `basicloganalysis.htm` defaults sandstone `DT Matrix` to **55**, so IP itself is
not internally uniform across modules — that is a module difference, not an error.

**D-14 — Clay parameter (56) "Percentile Clean" has no numbered entry in the CHM.
RESOLVED as a CHM-only authoring defect.**

Both IP2018 and IP2025 `clayparameters.htm` jump (55) → (57); the text describing Percentile
Clean is merged into the (55) block. `[hlp: ClayVol.hlp]` carries it correctly at n = 56
with a full description ("Numbers can be less than 0 %; negative percentiles are calculated
by linear scaling from the 0 % to the 100 % percentiles and extrapolating"). The parameter
is real and its semantics are known. **No default is stated in either source** (§8).

**D-15 — SW / SWE nomenclature conflict.**

Appendix 1 of `references_and_appencices.htm` defines `SW` as total water saturation on
total porosity and `SWE` as effective water saturation on effective porosity. The PhiSw
module uses `SW` = effective and `SWT` = total. **Two incompatible conventions ship in the
same manual.** SandiBumi must pin one and never emit a bare `SW`.

---

## 6. IP2018 numeric diff

Method: 15 of the 19 assigned pages exist in `c18`; 4 are new. `c18` contains only 6
`embim*.png` files out of 4328 PNGs, so **equation images cannot be image-diffed** — the
2018 side of every equation comparison is prose/raw-HTML or the prior ingest reports.

**Pages new in IP2025:** `references_and_appencices`, `density_estimation`,
`co_sw_analysis`, `bla-load`.

**Parameters new in IP2025:** clay module (71) Sonic Kerogen 150 uS/ft and (72) Sonic
Heavy_Min. 40 uS/ft — absent from `ClayVol.hlp` (2018), whose organic-shale block stops at
`Heavy_Min. Wt%_Con.`. The sonic organic-shale correction is therefore a genuine IP2025
capability addition, not a doc change.

**Method new in IP2025:** Alberty Smectite/Illite density-from-sonic. `densityestimation2.htm`
(2018) contains Gardner, Bellotti/AGIP and Lindseth only — zero occurrences of "Alberty".

**Unchanged across editions (verified byte-level or value-level):**

| Item | IP2018 | IP2025 |
|---|---|---|
| Clip Low/High doc bug (`basicloganalysis`) | broken | broken, identical text |
| Clip Low/High (`clayparameters`) | 0 % / 98 % | 0 % / 98 % |
| Missing (56) Percentile Clean numbering | absent | absent |
| Clay-bound-water `F` bracket defect | malformed | malformed, byte-identical |
| Shell `m` prose constant | 0.019 | 0.019 |
| Rho Dry Clay | 2.78 | 2.78 |
| Stieber constant | 2.0 | 2.0 |
| Kerogen 1.1 / 0.6, Heavy Min 4.3 / −0.03 | same | same |
| Wt% conversion 2.5 / 1.0 | same | same |
| PhiShr limit 0.02 | same | same |
| TP Lime/Sand/Dol/Clay 9.1/7.2/8.7/8.0 | same | same |
| Sonic Lime/Sand/Dol/water 49/56/44/189 | same | same |
| B fact Juhasz 1.0 meq/ml | same | same |
| Cm* | present | present, 1.0 |
| Permeability model names (Timur, Morris Biggs ×2, Chart K3) | same | same |

**Net recoveries from IP2025 that IP2018 could not yield** (all because 2018's equations
were rasterised GIFs that the 2018 ingest could not read):

1. All 13 clay-volume indicator equations with exact coefficients (§2.1–2.2).
2. The Waxman-Smits / Juhász `B(T, Rw)` formula.
3. The permeability coefficient table (Timur 8581/4.4/2 etc.).
4. The complete porosity-model equation set including the excavation-effect term.
5. The EPT/TPL water propagation-time equations.
6. `M`/`N` lithology ratios and the NMR gas-T1 expression.
7. The organic-shale porosity equations with the `Rho_dry_rock` coupling.

---

## 7. SandiBumi notes

1. **The clay-indicator recovery is the headline.** SandiBumi's Vcl module can now be
   checked coefficient-for-coefficient against IP for all 13 indicators. The Curved
   indicator's three-branch break at Z = 0.55 / 0.73 and the constants 0.0006078 / 1.58527 /
   2.1212 / 0.81667 are the kind of thing that silently diverges; pin them in a test.
2. **`m` vs `m*` is a real branch.** IP uses `m*` for Dual Water and Waxman-Smits but plain
   `m` for Juhász. Getting this wrong produces plausible-but-wrong Sw. The two `m*`
   coefficient sets (0.258/0.20/16.4 vs 1.128/0.22/17.3) are *not* interchangeable.
3. **The `Rw` factor in the Juhász / W&S conductivity term (D-12) must come from the
   equation images, not the parameter page.** If SandiBumi was built from an ASCII
   transcription of `swparameters.htm`, this is a live bug.
4. **`SW` is ambiguous in IP's own documentation (D-15).** SandiBumi should emit `SWE`/`SWT`
   explicitly and never a bare `SW`, and should refuse to auto-alias an imported `SW`
   without asking which porosity basis it is on.
5. **Sand/Silt/Clay:** the model is fully specified *except* the Lithology Conversion Chart
   node table (§8). SandiBumi cannot reproduce IP's `Fsn`/`Fsi`/`Fdc` split from the help
   text alone — it needs either the chart itself or Kuttan et al. Flag this before claiming
   SSM parity.
6. **Bound-water cap.** The SSM `Vbw <= 1.5 × Vcl × PhiTclay` clamp (and the `PhiT` re-set
   that follows it) is easy to omit and changes `Swb`, hence Sw, in shaly intervals.
7. **Tensor root selection is a user decision in IP.** If SandiBumi automates it, that is a
   deliberate improvement over IP and should be documented as such — not silently.
8. **Two independent vendor sources agree on Sonic Sand = 56** (CHM + `.hlp`), while BLA
   uses 55. If SandiBumi ships a single sandstone `DTma` default it will disagree with one
   IP module whichever it picks; better to make it module-scoped like IP does.
9. **PHIFLAG-equivalent output is mandatory.** IP's flag distinguishes "limit applied",
   "cutoff applied", "did not converge" and "anisotropy auto-reduced". A result with a limit
   silently applied is the same failure class Jauhar's own rules call out.

---

## 8. OPEN ITEMS

1. **D-07 — clay-bound-water `F`.** Correct bracketing unknown. Both editions carry
   `F = 1 - [0.6425 * (Salinity ^ (-0.5) + 0.22 ] * Qv]`. Resolve against the rendered help
   page or Hill, Shirley & Klein 1979 (SPWLA 20th Annual Symposium, Paper AA). Do not guess.
2. **D-10 — Shell `m` constant: 0.018 or 0.019?** Image says 0.018, nine-plus prose
   statements across two editions say 0.019. Unresolved; needs the rendered help page or the
   Shell source.
3. **D-09 — excavation-effect exponent** differs between PhiSw (`^2`) and SSM (`sqrt`).
   Unresolved; may be a genuine module difference.
4. **SSM Lithology Conversion Chart node table** is not published — only the "Clay at Silt
   Point" parameter B and A = 1 − B. `Fsn`/`Fsi`/`Fdc` cannot be reproduced from the help.
   Needs the chart image or Kuttan et al. (Malay Basin).
5. **C/O and Inelastic-Ratio saturation:** no closed-form Sw and no default `COO`/`COW`
   model-line coefficients are published. Genuine documentation gap.
6. ~~Permeability `a`/`b`/`c` role assignment.~~ **CLOSED** during this pass by reading
   `[img-read: _ccclip1769.png]`: `K = a × Phi^b / Swi^c`, both `Phi` and `Swi` in decimal.
   See §2.11.
7. **`(56) Percentile Clean` default** is not stated in the CHM or the `.hlp`. The prose uses
   "a 10th percentile" as an *illustration*, not a default — do not adopt 10 %.
8. **`Gr Kerogen` default** is not stated anywhere; the manual gives only a procedure for
   deriving it (crossplot Gr − CGR against volume TOC, regress, extrapolate to 100 % TOC).
9. **Temperature units for `B(T,Rw)`** are not stated on `swequationsandmethodology.htm`.
   The coefficient magnitudes and the SSM page's handling imply degF but the manual does not
   say so on the equation page.
10. **Temperature units for the TPL `E'` equation** (the `(T − 75)` and `3230.0` constants)
    are likewise unstated.
11. **`Rw^1.23` in `B`** — the exponent is legible at 6×, but the base is `Rw` at reservoir
    temperature vs at 75 degF is not specified.
12. **Organic-shale `Vker`/`Vhvy` iteration.** `Rho_dry_rock` depends on `Vker` and `Vhvy`,
    which depend on `Rho_dry_rock`. The manual gives no starting value, iteration order, or
    convergence tolerance for this sub-loop.
13. **`U Kerogen 0.264` and `U Heavy Min 77`** are stated on `swparameters.htm` without
    units. By context (multi-mineral `U`) these should be barns/cc, but the page does not
    say.
14. **Images not read** (judged low equation-yield from their text context, listed for
    completeness so the inventory is honest): `swequationsandmethodology` symbol-label
    images `embim44`–`embim48`, `embim51`, `embim59`–`embim62`, `embim67`, `embim68`,
    `embim74`–`embim77`, `embim80`, `embim81`, `embim83`–`embim102`, `embim123`–`embim158`;
    `_pawsclip0070`/`0071` (chart screenshots, described fully in the adjacent prose);
    `_ssmclip0049`–`0054_zoom70` (interactive-plot screenshots). The text labels each of
    these as a single symbol, a UI panel or a chart, not as an equation. If a later pass
    needs certainty on the `embim123`–`embim158` tail, that is the highest-value remaining
    block.

---

## 9. Bibliography (task c)

`references_and_appencices.htm` is **scoped to the PPFG (pore pressure / fracture gradient)
toolbox**, not the whole manual — relevant to whichever agent owns the PPFG pages. 29
entries:

Alberty, M. (2003) OTC 15290 · Alberty, M. (2005) SPE 108787 · Alberty & Reilly (2018) ·
Annen (2014) · Archie, G.E. (1942) SPE 942054-G · Arps (1953) SPE 953327-G · Athy (1930) ·
Ball (2015) HREI03 · Bellotti et al. (1979) · Bowers (1994) SPE 27488 · Casey (2015) ·
Daines (1982) SPE 9254, doi:10.2118/9254-PA · Dusenbery & Osoba (1986) SPE 15030 ·
Eaton & Eaton (1997) · Eberhart-Phillips et al. (1989) · Fertl et al. (1994) ·
Gardner et al. (1974) · Hodrick & Prescott (1997) · Jorden & Shirley (1966) ·
Katahara (2013) · Klein (1991) · Lindseth (1979) · Matthews & Kelly (1967) · Miller (2003) ·
Nagihara et al. (2013) · Rehm & McClendon (1971) SPE 3601 · Reilly (2018) · Teca (2014) ·
Traugott (1997) · Vernik (2016) doi:10.1190/1.9781560803256.fm · Zhang et al. (2008).

Appendix 1 is a curve-nomenclature table (depth, directional, pressure, petrophysical,
mudlog and formation-tester mnemonics) — the source of the SW/SWE conflict in D-15.
Appendix 2 documents curve-aliasing setup.

Citations found elsewhere in my page set: Hill, Shirley & Klein (1979) SPWLA 20th Annual
Symposium Paper AA (clay-bound water); Coates, Xiao & Prammer (1999) (NMR gas T1);
Batzle & Wang (1992) (downhole fluid densities); Lowe & Dunlap (1986) and Overton & Lipson
(1958) (mud resistivity); Kuttan et al. (Sand/Silt/Malay model); Western Atlas and
Schlumberger chartbooks (permeability, Sigma, SP-2, K3); AGA 8 NIST Detail and GERG (CO2).
