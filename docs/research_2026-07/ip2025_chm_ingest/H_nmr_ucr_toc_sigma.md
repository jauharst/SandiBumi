# Agent H — NMR, Unconventional Reservoirs, TOC, Sigma, CO2, Gas

Ingest of the Interactive Petrophysics **2025** vendor manual (decompiled CHM), agent-H scope.
Consumer: SandiBumi. Provenance discipline is absolute — every fact below carries either
`(pagename.htm)` for ASCII prose or `[img-read: file.png]` for a vision transcription of a raster.
Nothing here is filled in from textbook knowledge; gaps are gaps and live in §8 OPEN ITEMS.

---

## 1. Scope & page inventory

All 8 assigned pages read to completion. Every content image that sits where an equation,
constant table, or parameter dialog belongs was opened with vision.

| Page | Title | Chars | Content imgs | Status | 2018 counterpart |
|---|---|---|---|---|---|
| `nmrinterpretation.htm` | NMR Interpretation | 117,696 | 95 | read in full (1,972 lines) | yes |
| `ucr.htm` | Unconventional Reservoirs | 87,559 | 128 | read in full (2,276 lines) | yes |
| `sigma.htm` | Sigma Analysis | 22,351 | 34 | read in full | yes |
| `total_organic_carbon_content.htm` | Total Organic Carbon Content | 12,895 | 6 | read in full | yes |
| `co2storagecapacity.htm` | CO2 Storage Capacity | 7,362 | 8 | read in full | **NEW in 2025** |
| `gas_analysis.htm` | Gas Analysis | 5,368 | 5 | read in full | yes |
| `nmrnormalization.htm` | NMR Normalization | 4,875 | 3 | read in full | yes |
| `mud-gas-normalization.htm` | Mud Gas Normalization | 2,402 | 1 | read in full | **NEW in 2025** |

**Equation rasters transcribed by vision (90 total):**

- NMR: `embim165`–`embim195` (31) + `_nmriclip0040.png` (T2 Log Mean) = 32
- Sigma: `embim196`–`embim207` (12)
- UCR: `equation1_zoom75`–`equation43_zoom75` (43) + `_ucrclip0202.png` (brittleness, 3 sub-equations) = 44
- CO2: `embim20.png`, `embim21.png` (2)
- Gas / mud-gas / TOC: equations are printed as ASCII on the page, not rastered; the only
  numeric-bearing rasters are the HCF crossplot overlay `_ccclip1806_zoom85.png` and the
  Sigma mineral-library dropdown `_chclip0006.png` — both read.

**Parameter-dialog rasters read for defaults (24):** `_nmriclip0002/0007/0008/0009/0010/0011/0012/0042/0043/0044/0050`,
`_chclip0004/0005/0006`, `_intclip0041/0042/0043`, `_ucrclip0011/0019/0035/0045/0046/0047/0048/0054/0062`,
`_co2clip0004/0005`, `mudgasnormalization.png`.

**Headline counts:** 8/8 pages · 90 equation rasters transcribed by vision (plus ~30 ASCII
equations) · 230+ distinct parameter values, constants, cut-offs and validation bounds tabulated in
§3 · 17 internal discrepancies · 1 genuine 2018→2025 numeric documentation change · 11 OPEN ITEMS.

---

## 2. Equations & methods per module

### 2.1 Sigma Analysis (cased hole / TDT) — special tasking (c)

**This page answers agent C's gap: cased-hole capture-cross-section endpoints ARE documented here.**
Units throughout are **capture units (CU)**.

**Sigma matrix from mineral volumes** (ASCII, `sigma.htm`):

```
SigMat = VolMin1*SigmaMin1 + VolMin2*SigmaMin2 + VolMin3*SigmaMin3
       + VolMin4*SigmaMin4 + VolMin5*SigmaMin5
```

**Sigma matrix from matrix density — SS-DOL model** `[img-read: embim200.png]`:

```
SigMat = ((RhoMat - 2.65) / (2.85 - 2.65)) * (SigDol - SigSand) + SigSand
subject to   SigDol >= SigMat >= SigSand
```

**Sigma matrix from matrix density — SS-LS-DOL model** `[img-read: embim201.png]`:

```
if RhoMat <= 2.71:
    RhoMat = maximum of 2.65 or RhoMat
    SigMat = ((RhoMat - 2.65) / (2.71 - 2.65)) * (SigLime - SigSand) + SigSand
if RhoMat >= 2.71:
    RhoMat = minimum of 2.85 or RhoMat
    SigMat = ((RhoMat - 2.71) / (2.85 - 2.71)) * (SigDol - SigLime) + SigLime
```

The three density anchors 2.65 / 2.71 / 2.85 g/cc are hard-wired in the equation raster, not
parameters.

**Stand-alone Sw from sigma** `[img-read: embim202.png]`:

```
SwTDTU = [SigLog - SigMat - Phi*(SigHyd - SigMat)] / [Phi*(SigWat - SigHyd)]
       - [Vcl*(SigClay - SigMat)] / [Phi*(SigWat - SigHyd)]
```

Algebraically consistent with the reconstruction equation below (verified by re-deriving
`SigRec` for `SwOH = SwTDTU`).

**Time-lapse chain** `[img-read: embim196/197/198.png]`:

```
DeltaSw = (SigNew - SigBase) / (Phi * (SigWat - SigHyd))
if Phi < PhiCutoff  or  Vcl > VclCutoff:   DeltaSw = 0.0
SwTDTU  = SwBase + DeltaSw
```

**Bulk volume water** `[img-read: embim199.png, embim203.png]`: `BVWtdt = Phi * SwTDT`

**Apparent water sigma** `[img-read: embim204.png]`:

```
SigWatApp = [Sigma - SigMat*(1 - Vcl - Phi) - SigClay*Vcl] / Phi
```

**Reconstruction curves** `[img-read: embim205/206/207.png]`:

```
SigRec    = (1-Vcl-Phi)*SigMat + Vcl*SigClay + Phi*SwOH*SigWat + Phi*(1-SwOH)*SigHyd
SigRecWat = (1-Vcl-Phi)*SigMat + Vcl*SigClay + Phi*SigWat
SigRecHyd = (1-Vcl-Phi)*SigMat + Vcl*SigClay + Phi*SigHyd
```

Note the volumetric convention: matrix fraction is `(1 - Vcl - Phi)`, i.e. **clay is carried as a
bulk-volume fraction alongside porosity**, not as a fraction of the matrix.

Three `Sigma Ma Source` options are offered (`sigma.htm`): **Parameter** (fixed constant, the
interactive Sigma-Matrix point on the Phi–Sigma crossplot), **Matrix Den** (the two interpolation
branches above), **Mineral Vols** (`MinVol1..MinVol5` × `SigMin1..SigMin5`).

### 2.2 NMR Interpretation — special tasking (a)

#### Porosity partitioning (CBW / BVI / FFI)

Rules are stated as bin-summation logic, not equations (`nmrinterpretation.htm`):

- `PhiT` = sum of **all** bins.
- `Phie` = sum of all bins **above** the T2 clay-bound cut-off. If no clay-bound cut-off is set,
  `Phie == PhiT`.
- `nmrFF` = sum of all bins **above** the capillary-bound (free-fluid) cut-off — or, under the
  tapered method, the weighted sum below.
- Clay-bound cut-off is only applied when the **Clay Bound Fluid Cutoff** box is ticked (newer
  tools that actually measure clay fluid). Unticked ⇒ `PhiT == Phie`.
- `nmrBFEt = nmrBFTt - nmrCBF` (effective bound fluid, tapered method).

**Tapered (spectral) bound fluid** `[img-read: embim179.png, embim180.png]`:

```
nmrBFTt = SUM(i = 1..n) [ Wi * PHIi ]
1/Wi    = m * T2i + b
```

The ASCII parameter text prints the same weighting as `Wi = 1.0 / (m.T2 + b)`
(`nmrinterpretation.htm` line 663) — **ASCII and raster agree** (RULE 4 cross-check passes).

**Tapered coefficient defaults, from a global core study of 340 sandstone + 71 carbonate samples**
(`nmrinterpretation.htm`):

| Tapered Constants option | m | b |
|---|---|---|
| Sand | **0.0618** | **1** |
| Carbonate | **0.0113** | **1** |
| User | user-entered | user-entered |

Cited to *Coates et al., "A new characterization of bulk-volume irreducible using magnetic
resonance", paper QQ, 38th Annual SPWLA Symposium* — the manual prints the year as **1977**
(see §5 discrepancy D-6).

`BF Method` selects **T2 cutoff** / **Tapered** / **Maximum** (max of the two). The page argues
for Maximum in hydrocarbon zones at irreducible saturation, where a tapered weight < 1 would be
applied to water that is in fact all bound.

#### Permeability models

| Model | Equation | Source |
|---|---|---|
| Timur–Coates | `K = a * (FF/BF)^b * Phi^c` | `[img-read: embim165.png, embim176.png]` |
| Modified Coates | `K = a * ( d*FF / (BF + (1-d)*FF) )^b * Phi^c` | `[img-read: embim166.png, embim177.png]` |
| T2 Log Mean (SDR-form) | `K = a * T2LogMean^b * Phi^c` | `[img-read: embim167.png, embim178.png]` |

```
T2LogMean = ALog[ SUM(i=StartBin..StopBin) (Log(T2(i)) * Phi(i))
                / SUM(i=StartBin..StopBin) Phi(i) ]
```
`[img-read: _nmriclip0040.png]`

`d` is a **pore-connectivity** term for carbonates: `d = 1` ⇒ all pore space connected and the
Modified Coates collapses to standard Timur–Coates; `d = 0` ⇒ all free fluid isolated/immobile
(`nmrinterpretation.htm`). If *Remove Clay Bound fluid from Coates Permeability* is selected,
`Phie` is substituted for `PhiT` in the Coates equations.

**The manual prints no default a/b/c/d in prose.** The values below are read off the shipped
dialog and must be treated as an example set, not a documented default (see §8 OPEN-3).

#### Saturation — Dual Water and the Z reformulation

```
Ct  = (PhiT^m * SwT^n / a) * [ Cw*(1 - Swb/SwT) + Ccw*(Swb/SwT) ]     [img-read: embim183.png]
1/Rt = (PhiT^m * SwT^n / a) * [ (1/Rw)*(1 - Swb/SwT) + (1/Rwb)*(Swb/SwT) ]
                                                                      [img-read: embim184.png]
Ccw = 1/Rwb = 0.000126 * (T - 16.7) * (T + 504.4)      T in degF       [img-read: embim185.png]
Swe = (SwT - Swb) / (1 - Swb)                                          [img-read: embim186.png]
```

Z-function form:

```
Ct = ((PhiT * SwT)^Z / a) * [ Cw*(1 - Swb/SwT) + Ccw*(Swb/SwT) ]       [img-read: embim187.png]

Z  = log(PhiT^m * SwT^n) / log(PhiT * SwT)
   = [ m*log(PhiT) + n*log(SwT) ] / log(PhiT * SwT)                    [img-read: embim188.png]

Z  = Zoffset + Zslope * SwT                                            [img-read: embim189.png]
```
where `Zoffset = Z irreducible` and `Zslope = (Z irreducible − Z wet)` (`nmrinterpretation.htm`).

Theoretical Z end-point curves, output to help pick the two parameters:

```
Z       = log[ Ct / ( Cw*(1 - Swb/SwT) + Ccw*(Swb/SwT) ) ] / log(PhiT * SwT)   [embim190.png]
nmrZwet = log[ Ct / ( Cw + Swb*(Ccw - Cw) ) ] / log(PhiT)                      [embim191.png]
nmrZswi = log[ Ct / ( Cw + (Swb/nmrSwiT)*(Ccw - Cw) ) ] / log(PhiT * nmrSwiT)  [embim192.png]
```

`Z` is floored at `nmrZswi`, which limits `SwT` to `nmrSwiT`. **This limit is applied whether or not
the Z function is selected** (`nmrinterpretation.htm`).

Other:

```
Rwapp     = Rt * PhiT^m / a                                            [img-read: embim193.png]
Rt_Hingle = Rt^(-1/m)                                                  (nmrinterpretation.htm)
```
with `m = entered m parameter`, or `m = Zoffset + Zslope` when the Z function is active.

#### Light-hydrocarbon (LHC) correction

Detection (`nmrinterpretation.htm`, ASCII):

```
lhcPhidW = (RhoMa - RhoB - VCL*(RhoMa - RhoWcl)) / (RhoMa - RhoFluid)     with RhoFluid = 1.0
```
Compared against `nmrPhiE` when the density porosity is clay-corrected, against `nmrPhiT` when it
is not. `lhcPhidW > nmrPhi` ⇒ `lhcFlag = 1`.

Polarisation factors `[img-read: embim181.png, embim182.png]`:

```
Pg = HIg * 1 - e^(-Tpol / T1g)          <- parentheses omitted in the raster
Pw = HIw * 1 - e^(-Tpol / T1w)          <- parentheses omitted in the raster
```
The ASCII text elsewhere on the same page prints the intended grouping:
`PHC = HI_LHC * (1 - e^(-Tw / T1_LHC))` (`nmrinterpretation.htm` line 1846). See §5 D-7.

Simultaneous solution for `Sgff` and `FF` (ASCII):

```
nmrPHIE = (FF * Sgff * Pg) + (FF * (1 - Sgff) * Pw) + (HIw * BFE)
RhoB    = RhoMa*(1 - FF - BFE - Vcl) + (RhoWcl*Vcl) + (RhoG*Sgff*FF)
        + (RhoW*(1 - Sgff)*FF) + (RhoW*BFE)
```
Assumption stated explicitly: water-wet rock, **all gas contained in the free fluid**; bound water
**100 % polarised** so only HI (no polarisation factor) is applied to the bound-water term.

#### T2 Wet (fluid substitution) and T2→Pc

```
HCsum = (PhiT_f - BVWSxoT) * PHC,     PHC = HI_LHC * (1 - e^(-Tw / T1_LHC))
```
A synthetic hydrocarbon T2 distribution (normal, user std-dev, centred on the Hydrocarbon T2
parameter) is scaled to area `HCsum` and subtracted bin-by-bin; negative bins clamp to zero and the
un-subtracted residual is reported as QC curve `T2neg_adj`. A synthetic water signal of area
`WaterSum = PhiT_final − sum(T2NoHCsignal)` is then re-inserted, its centre-point varied by
successive approximation (within the T2 Water Low/High limits) until `T2LMwet` matches
`T2LMtarget = Gain * FF_BF + Offset`, with `FF_BF = FF_f / BFT_f`. Method cited to
Volokitin, Looyestijn, Slijkerman & Hofman, SPWLA 40th, 1999 (`nmrinterpretation.htm`).

T2→Pc (cited to SPE 81057, Glorioso):

```
Pci = 10^( Pcgain * Log(1/T2i) + Pcoffset )                            [img-read: embim194.png]
Pci = A log( Pcgain * Log(1/T2i) + Pcoffset )                          [img-read: embim174.png]
        ("A log" = antilog; same relationship, alternate rendering)
Pc  = (FWL - TVDss) * (DensityWater - DensityHydrocarbon)              [img-read: embim195.png]
PoreRadius = 100 / Pc          (Pc in psi)                             [img-read: embim175.png]
```
Sw axis construction: the **highest** T2 time is taken as `Sw = 100 %`; each bin porosity is
converted to `BinPhi / PhiT` and subtracted cumulatively; the lowest T2 time reaches `Sw = 0 %`.

### 2.3 NMR Normalization

No equations. The module re-samples an asymmetric T2 array, fits a polynomial through it, and
re-normalises the fitted curve to the porosity so the porosity is not over-called
(`nmrnormalization.htm`). Rationale: Baker Atlas and pre-2000 Halliburton distributions do not span
complete log decades (e.g. 4–512 ms, 0.5–1024 ms, 1–4096 ms), so a cut-off line cannot be plotted
against them correctly — the arithmetic is right, the visual pick is not.

Tool-specific end-point defaults (`nmrnormalization.htm` and `nmrinterpretation.htm` agree):
Schlumberger CMR **3 and 3000 ms**; CMR-200 and CMR-plus **0.3 and 3000 ms**. Schlumberger and
post-2000 Halliburton span 4 log decades (0.3–3000 or 0.5–5000 ms).

Special case: the **CMR-plus can encode the T2 distribution in a 90-sample array of which only the
last 30 samples are the distribution** — set Start sample = 61, Stop sample = 90.

Bin-edge convention: the *"T2 Start, Stop times are bin mid point times"* flag should be **off** for
Schlumberger tools and **on** for Numar (Halliburton / Baker Atlas) tools. Impact is small except
for low-bin-count tools.

Tool setup defaults live in an editable **`NMR_Tools.csv`** in the IP directory — i.e. the vendor
treats the per-tool defaults as data, not code.

### 2.4 Total Organic Carbon — special tasking (b), TOC part

**Fourteen density regressions, all ASCII** (`total_organic_carbon_content.htm`). `RHOB` in g/cc;
`RhoShale` is a parameter:

```
TOC_DEN1 = (-30.784*RHOB + 80.902) * .01
TOC_DEN2 = (-17.061*RHOB + 46.711) * .01
TOC_DEN3 = (55.82 * ((RhoShale/RHOB) - 1.0)) * .01
TOC_DEN4 = (-30.28*RHOB + 79.94) * .01
TOC_DEN5 = ((518.0 - 194.0*RHOB - 7.0) / 7.3) * .01
TOC_DEN6 = (-31.1*RHOB + 82.50) * .01
TOC_HEND = (-29.172*RHOB + 77.23) * .01
TOC_SCHM = ((156.956/RHOB - 58.272)) * .01
TOC_COL  = (-23.0*RHOB + 62.1) * .01
TOC_MODE = (RhoShale - RHOB) / (1.7*RHOB)
TOC_MDPY = (-26.738*RHOB + 70.741) * .01
TOC_JLN1 = (-42.97115 + 114.1864/RHOB) * .01
TOC_JLN2 = (-26.6694 + 75.36593/RHOB) * .01
TOC_SCHD = (-14.9*RHOB + 41.11) * .01
```

`TOC_DenAvg` = mean of whichever of the fourteen have their **Use** flag on (default: all fourteen).
`TOC_DenReg` = a **proprietary** 3rd-order (default) or 5th-order regression — coefficients are
**not printed** (§8 OPEN-5).

**ΔlogR:** the module implements Passey, Creaney, Kulla, Moretti & Stroud, *AAPG Bulletin* v.74
no.12, December 1990, with sonic / density / neutron overlays plus an arithmetic (`DlogRavg`) and a
weighted (`DlogRcombo`) average. **The manual prints only the citation — no ΔlogR equation, no
`TOC = ΔlogR × 10^(a − b·LOM)` coefficients, and no weighting scheme for `DlogRcombo`.** This is a
real documentation gap, not an extraction failure (§8 OPEN-4).

**Heavy mineral:** `VHVY_EMP = Xfactor * TOC_final`, rationale given as the geochemical link between
TOC and pyrite precipitation.

**Unit contract, stated verbatim as an "Important Note"** (`total_organic_carbon_content.htm`):
the density relationships and ΔlogR methods return TOC in **dry weight percent (wt%)**, while the
default `Xfactor` returns heavy-mineral volume in **wet volume (v/v)**. The PhiSw *Organic Shale*
option defaults are set to exactly this condition; recalibrating to TOC v/v or heavy-mineral wt%
requires changing the PhiSw parameters to match. (Tension with the `*.01` scaling — see §5 D-8.)

### 2.5 Unconventional Reservoirs — special tasking (b)

#### Rock Mechanics (eq 5-1 … 5-7)

```
v   = 10^6 / Dt                                                        [equation1_zoom75.png]
Ed  = 0.001 * rho_b * vs^2 * ( (3*vp^2 - 4*vs^2) / (vp^2 - vs^2) )     [equation2_zoom75.png]
K   = 0.001 * rho_b * ( vp^2 - (4/3)*vs^2 )                            [equation3_zoom75.png]
G   = 0.001 * rho_b * vs^2                                             [equation4_zoom75.png]
mu  = 0.5 * ( (vp^2 - 2*vs^2) / (vp^2 - vs^2) )                        [equation5_zoom75.png]
sigma_h / sigma_v = mu / (1 - mu)                                      [equation6_zoom75.png]
Es  = 10^( 6 + log(10^-9 * rho * Ed) - 0.55 )                          [equation7_zoom75.png]
```

Units (`ucr.htm`): `v` m/s, `Dt` µs/m, `rho_b` **kg/m3**, `Ed`/`Es`/`K`/`G` **kPa**, `mu`
dimensionless, stresses kPa. The `0.001` factors are the kg/m3·(m/s)² → kPa conversion.
`sigma_h/sigma_v` explicitly **ignores tectonic, thermal, and other stresses**.

`Es` (eq 5-7) is stated to come from *Barree, Gilbert & Conway, SPE 118701 (2009), Equation 15,
with changes for units*, matched to the "Modified E-K Log-Linear" points. The manual also records
that Barree et al. found the static-vs-dynamic Poisson's-ratio difference minor — which is why IP
applies no static correction to `mu`.

#### Brittleness Index (SPE 115258, Rickman et al.) `[img-read: _ucrclip0202.png]`

```
E_britt  = ( (E_Mpsi - 1) / (8 - 1) ) * 100
mu_britt = ( (mu - 0.4) / (0.15 - 0.4) ) * 100
Britt    = (E_britt + mu_britt) / 2
```
`E_Mpsi` = **dynamic** Young's modulus in **Mpsi**; `mu` = Poisson's ratio (`ucr.htm`).
The normalisation end-points are hard-wired: E over 1→8 Mpsi, mu over 0.4→0.15 (inverted, so low
Poisson's ratio ⇒ high brittleness). Verified at 5× upscale.

#### Gas Pressure (eq 8, 9) and Oil Pressure (eq 10, 11)

```
T     = T_datum - DeltaT * (d - d_datum)                    [equation8/10_zoom75.png]
p_i-1 = p_i - 9.80665 * rho_g * d_inc                       [equation9_zoom75.png]
p_i-1 = p_i - 9.80665 * rho_o * d_inc                       [equation11_zoom75.png]
```
`9.80665` = standard gravity, giving kPa from (g/cm3 × m). Densities in **g/cm3**, `d_inc` in m,
pressures kPaa. The pressure march is done at **one-metre intervals** between the datum and the top
of the interval, then "updated with one iteration with the average gradient between the two depth
intervals" (`ucr.htm`) — i.e. a two-pass trapezoidal correction. Note the **minus** sign in the
temperature equation (§5 D-9).

#### Fluid Properties (eq 12, 13, 14)

```
v_sg  = [ gamma / (c_g * rho_g) ]^0.5                       [equation12_zoom75.png]
K_a   = gamma * c_g^-1                                      [equation13_zoom75.png]
Dt_g  = 10^6 / v_sg                                         [equation14_zoom75.png]
```
`gamma` = Cp/Cv (dimensionless), `c_g` isothermal gas compressibility kPa⁻¹, `rho_g` g/cm3,
`v_sg` m/s, `Dt_g` µs/m. **Methane is excluded** from eq 12 — for pure methane, NIST properties are
used. z-factor from **Hall & Yarborough** (Standing–Katz). Algorithms attributed to Whitson & Brulé,
*Phase Behavior*, SPE Monograph 20 (2000).

#### Gas Storage Capacity — Langmuir chain (eq 13-1 … 13-22)

```
G_sgai = G_sLi * p / (p + p_Li)                             [equation15_zoom75.png]  (13-1)
G_sLoi = G_sLi / C_TOC-I                                    [equation16_zoom75.png]  (13-2)
G_sLi  = G_sLoi * C_TOC                                     [equation17_zoom75.png]  (13-3)

extended Langmuir (multi-component):
G_sgai = G_sLi * [ (p*y_i / p_Li) / (1 + p * SUM(j=1..n) (y_j / p_Lj)) ]
                                                            [equation18_zoom75.png]  (13-4)
G_sga  = SUM(i=1..n) G_sai                                  [equation19_zoom75.png]  (13-5)
x_i    = G_sgai / G_sga                                     [equation20_zoom75.png]  (13-6)
z_i    = (G_sga*x_i + G_sgf*y_i) / (G_sga + G_sgf)          [equation21_zoom75.png]  (13-7)
G_ga   = A * h * rho_b * G_sga                              [equation22_zoom75.png]  (13-8)
```

**Adsorbed-gas pore-volume correction** `[equation23_zoom75.png]` (13-9):

```
Phi * S_gc = Phi * S_g - 4.221e-5 * M_hat * (rho_b / rho_s) * G_sga
```
`4.221 × 10⁻⁵` is a hard constant. `M_hat` = gas molecular weight g/gmol, `rho_b` and `rho_s` g/cm3,
`G_sga` cm3/g.

**Adsorbed-gas mixture density** `[equation24_zoom75.png]` (13-10):

```
rho_s = [ SUM(i=1..n) (x_i / rho_si) ]^-1
```

Free / dissolved / totals:

```
G_sgf = Phi * S_gc / (rho_b * B_g)                          [equation25_zoom75.png] (13-11)
B_g   = [ p_sc / (z_sc * (T_sc + 273.15)) ] * [ z*(T + 273.15) / p ]
                                                            [equation26_zoom75.png] (13-12)
G_gf  = A * h * Phi * S_gc / B_g                            [equation27_zoom75.png] (13-13)
G_sgd = Phi * S_w * R_sw / (rho_b * B_w)                    [equation28_zoom75.png] (13-14)
G_gd  = A * h * Phi * S_w * R_sw / B_w                      [equation29_zoom75.png] (13-15)
G_sg  = G_sgf + G_sga + G_sgd                               [equation30_zoom75.png] (13-16)
G_g   = G_gf + G_ga + G_gd                                  [equation31_zoom75.png] (13-17)
```
The `+ 273.15` in eq 13-12 confirms `T`, `T_sc` are entered in **°C** and converted to K internally.

**Condensate recombination / gas-gravity corrections:**

```
gamma_wh = ( gamma_g + gamma_o*rho_w*r_p*V_g_hat )
         / ( 1 + gamma_o*rho_w*r_p*V_g_hat / M_o_hat )       [equation32_zoom75.png] (13-18)

gamma_hc = [ gamma_g - (gamma_H2S*x_H2S + gamma_CO2*x_CO2 + gamma_N2*x_N2) ]
         / [ 1 - (x_H2S + x_CO2 + x_N2) ]                    [equation33_zoom75.png] (13-19)

x_hc     = 1 - ( gamma_g + gamma_o*rho_w*r_p*V_g_hat )
             / ( M_o_hat + gamma_o*rho_w*r_p*V_g_hat )       [equation34_zoom75.png] (13-20)

x_i'     = x_hc * x_i                                        [equation35_zoom75.png] (13-21)

gamma_ws'= gamma_ws*x_hc + gamma_H2S*x_H2S' + gamma_CO2*x_CO2 + gamma_N2*x_N2
                                                             [equation36_zoom75.png] (13-22)
```
where `V_g_hat` = molar volume of gas m3/mol, `M_o_hat` = oil molecular weight g/mol,
`rho_w` kg/m3, `r_p` separator condensate–gas ratio m3/m3.

**Non-hydrocarbon specific gravities relative to air, printed as hard constants** (`ucr.htm`):
**H2S = 1.1764**, **CO2 = 1.5192**, **N2 = 0.96928**. Air molecular weight is taken as
**28.97 g/gmol** throughout.

**Table 31 — Isotherm Gas Properties** (`ucr.htm`). The last row is explicitly stated to be the
adsorbed-gas density values **built into the software**:

| Property | Units | Methane | Ethane | Propane | Helium | Nitrogen | Carbon Dioxide |
|---|---|---|---|---|---|---|---|
| Molar mass | g/gmol | 16.0428 | 30.070 | 44.0956 | 4.0026 | 28.0348 | 44.0098 |
| Critical pressure | kPaa | 4,599.2 | 4,871.8 | 4,247.7 | 227.45 | 3,395.8 | 7,377.4 |
| Critical temperature | °C | −82.58 | 32.18 | 96.7 | −267.96 | −146.96 | 30.978 |
| Critical density | g/cm3 | 0.1627 | 0.2066 | 0.2243 | 0.0696 | 0.3133 | 0.4676 |
| Triple-point temperature | °C | −182.46 | −182.80 | −187.67 | −270.97 | −210.00 | −56.408 |
| Atm.-pressure boiling point | °C | −161.48 | −88.60 | −42.7566 | −268.92 | −197.79 | −78.4 |
| **Liquid density at atm. boiling point ⇒ adsorbed density** | **g/cm3** | **0.4223** | **0.5440** | **0.5812** | **0.1247** | **0.8061** | **1.1780\*** |

\* CO2 value is the saturated-liquid density at the **triple-point** temperature, not the boiling
point (footnote, `ucr.htm`).

Note the asymmetry: **Helium appears in Table 31 but is not one of the five Langmuir gases**
(C1, C2, C3, N2, CO2) offered on the Adsorption tab.

#### Oil Storage Capacity (eq 14-1 … 14-6)

```
O_sv  = Phi * S_o / B_o                                     [equation38_zoom75.png] (14-1)
W_sv  = Phi * S_w / B_w                                     [equation39_zoom75.png] (14-2)
G_svg = O_vs * R_s + W_sv * R_sw                            [equation40_zoom75.png] (14-3)
O     = A * h * O_sv                                        [equation41_zoom75.png] (14-4)
G     = A * h * G_svg                                       [equation42_zoom75.png] (14-5)
W     = A * h * W_sv                                        [equation43_zoom75.png] (14-6)
```
`h` in all in-place equations is the **true-vertical-depth increment between adjacent log samples**;
the incremental volumes are summed only over net-pay intervals (`ucr.htm`).
(`O_vs` in the 14-3 raster is a typo for `O_sv` — §5 D-10.)

The whole UCR tool suite is stated to have been **developed by Apache Corporation and licensed for
use in IP** (`ucr.htm`) — relevant to any clean-room reimplementation.

### 2.6 CO2 Storage Capacity — special tasking (e), NEW in 2025

Only resource type supported: **Saline Aquifer** (`co2storagecapacity.htm`).

**Fluid displacement** `[img-read: embim20.png]`:

```
CO2fluid = ( ACF * A * H * theta * (1 - Swirr) * rho_CO2 * E_CO2 ) / 1000
```

**Soluble fraction** `[img-read: embim21.png]`:

```
CO2SolubleFraction = ( ACF * A * H * theta * Sw * rho_w * 44.01 * Sol * Es_CO2 ) / (1000 * 1000)
```

**Total storage capacity = CO2fluid + CO2SolubleFraction**, output in **Metric Tonnes**, computed
per depth step (`H` is the depth-step thickness) and cumulated by the Cutoff & Summation module.

Constants and unit conventions, all stated in prose (`co2storagecapacity.htm`):

- `ACF` = area conversion factor to m²: **4046.8564** for Acres, **10⁶** for km².
- `rho_CO2`, `rho_w` in **kg/m³**. If Tools > Default Units is set to g/cc, the entered value is
  multiplied by 1000 internally. Densities are at **downhole conditions**, so **no `Bg` is applied**.
- `44.01` = CO2 molar mass g/mol; combined with the `/1000`, converts `Sol` (mol/kg) to kg.
- Trailing `/1000` in each equation = kg → Metric Tonne.
- `E_CO2` / `Es_CO2` are efficiency multipliers. The page notes `Es_CO2` may also be used to absorb
  the reduction in available water volume caused by the fluid-displacement term — i.e. the two
  terms are **not** rigorously mass-balanced against each other.

**Inaccessible-pore method (branch, changes the equation):**
- **Swirr method** — usable pore space = `theta * (1 - Swirr)`, exactly as the raster prints it.
- **BVI method** — usable pore space = `theta - BVI` instead.

### 2.7 Gas Analysis — special tasking (d)

Haworth et al. (1985), "Interpretation of Hydrocarbon Shows Using Light (C1–C5) Hydrocarbon Gases
from Mud-Log Data". ASCII equations (`gas_analysis.htm`):

```
GWR = (C2 + C3 + C4 + C5) / (C1 + C2 + C5) * 100        (Haworth's Wh)
LHR = (C1 + C2) / (C3 + C4 + C5)                        (Haworth's Bh)
OCQ = (iC4 + nC4 + C5) / C3                             (Haworth's Ch)
where C4 = iC4 + nC4  and  C5 = iC5 + nC5
```

**The GWR denominator `(C1 + C2 + C5)` omits C3 and C4** — see §5 D-11.

**Hydrocarbon Flag (HCF) decision table**, verbatim conditions:

| HCF | Meaning | Condition |
|---|---|---|
| 1 | very dry gas; probably non-productive | LHR > 100.0 and GWR ≤ 0.5 |
| 2 | very dry gas; possibly productive | LHR > 100.0 and GWR > 0.5 |
| 3 | potential gas, density increasing as LHR falls | LHR < 100.0, LHR > GWR, 17.5 > GWR > 0.5 |
| 4 | gas/oil or gas/condensate — gas | LHR < 100.0, LHR < GWR, 17.5 > GWR > 0.5, OCQ < 0.5 |
| 5 | gas/oil or gas/condensate — light oil | LHR < 100.0, LHR < GWR, 17.5 > GWR > 0.5, OCQ > 0.5 |
| 6 | oil | LHR < 100.0, LHR < GWR, 40 > GWR > 17.5 |
| 7 | residual oil | LHR < 100.0, LHR < GWR, GWR > 40 |
| 8 | no solution | — |
| 9 | no case | LHR < 100.0, LHR > GWR, and (17.5 < GWR or GWR < 0.5) |

Discriminant constants: **LHR = 100**, **GWR = 0.5 / 17.5 / 40**, **OCQ = 0.5**.
Colour code (Z-axis on the Haworth crossplot): 1 blue, 2 aqua, 3 fuchsia, 4 green, 5 brown,
6 lime, 7 navy. The crossplot overlay `[img-read: _ccclip1806_zoom85.png]` confirms the region
geometry: LHR axis 0.1–1000 log, GWR axis 0.1–1000 log, with the diagonal `LHR = GWR` line and the
horizontal 0.5 / 17.5 / 40 boundaries and vertical LHR = 100 boundary exactly as tabulated.

`OCQ = 0.5` is a **strict** inequality on both sides (`< 0.5` → HCF 4, `> 0.5` → HCF 5) — the
manual defines no behaviour at exactly 0.5 (§5 D-12).

Inputs are C1–C8 plus δ13C isotopes, Total Gas, ROP, RPM and GR. **Total Gas, ROP, RPM and GR are
carried for plot correlation only and are not used in any calculation** (`gas_analysis.htm`).
Generic `C4`/`C5` go in the **normal** (nC) boxes; entering the same curve in both the normal and
iso boxes double-counts, because the module sums them. Available δ13C crossplots: Bernard, two
Bernard-as-modified-by-Milkov, Haworth, Lorant, two Schoel, two Whiticar.

Single parameter: **Value for No Calculation** — `ZERO (0.0)` or `NULL (-999)`. Null output curves
are deleted by default.

### 2.8 Mud Gas Normalization — special tasking (d), NEW in 2025

The "AGIP" relationship, printed as ASCII (`mud-gas-normalization.htm`):

```
Gas(normalized) = (Gas * Flowrate * 5.0028) / (ROP * Diameter^2)
```
**`5.0028` is a hard constant.** The page does **not** state the units of Flowrate, ROP or bit
Diameter that make `5.0028` correct (§8 OPEN-9). Required inputs: ROP, Flowrate, Bit Diameter and
at least one gas curve.

Normalized outputs (`[img-read: mudgasnormalization.png]`, module short name **AMGN — Advanced Mud
Gas**): C1N, C2N, C3N, iC4N, nC4N, iC5N, nC5N, nC6N, nC7N, nC8N, CHN, MCHN, BENEZENEN *(sic)*,
TOLUENEN, TGN — i.e. C1–C8 plus cyclohexane, methylcyclohexane, benzene, toluene and total gas.

Same `Value for No Calculation` parameter (`NULL (-999)` or `ZERO (0.0)`), no log plot, zones
optional. References: Kandel, Quagliaroli, Segalini & Barraud, SPE 65176 (2000); Alberty & Fink,
SPE 166188 (2013).

---

## 3. Parameters, defaults, cutoffs & constraints

Values marked **[dialog]** are read off the shipped example dialog and are *not* stated as defaults
in the prose. Values marked **[prose]** are explicitly called defaults by the manual. The
distinction matters: only **[prose]** values are safe to port as SandiBumi defaults.

### 3.1 Sigma / TDT

| Parameter | Value | Unit | Source |
|---|---|---|---|
| Sigma Water | 80 | CU | `[img-read: _chclip0004.png]` [dialog] |
| Sigma Hyd | 20 | CU | `[img-read: _chclip0004.png]` [dialog] |
| Sigma Clay | 25 | CU | `[img-read: _chclip0004.png]` [dialog] |
| Sigma Sand | **4.3** (at ρma 2.65) | CU | `sigma.htm` [prose] + `_chclip0005.png` |
| Sigma Lime | **7.1** (at ρma 2.71) | CU | `sigma.htm` [prose] + `_chclip0005.png` |
| Sigma Dol | **4.7** (at ρma 2.85) | CU | `sigma.htm` [prose] + `_chclip0005.png` |
| Sigma Ma Source | Matrix Den (of Parameter / Matrix Den / Mineral Vols) | — | `[img-read: _chclip0004.png]` [dialog] |
| Matrix Input | Parameter; Matrix Den 2.65 | g/cc | `[img-read: _chclip0005.png]` [dialog] |
| Model Input | SS-LS-DOL (of SS-DOL / SS-LS-DOL) | — | `[img-read: _chclip0005.png]` [dialog] |
| Phi Limit / Phi Cutoff | **0** (0 disables) | v/v | `sigma.htm` [prose] |
| Vcl Limit / Vcl Cutoff | **1** (1 disables) | v/v | `sigma.htm` [prose] |
| Sw Limit | **1** | v/v | `sigma.htm` [prose] |

Sourcing guidance stated in prose: *Sigma Water* from a chartbook by salinity (Schlumberger
**Tcor-2**); *Sigma Hyd* from **Tcor-1**.

**Sigma mineral library — 21 entries, from the Sig Mat Min dropdown** `[img-read: _chclip0006.png]`
(CU):

| Mineral | Sigma | Mineral | Sigma | Mineral | Sigma |
|---|---|---|---|---|---|
| Quartz | 4.26 | Ankerite | 22.18 | Anorthite | 7.24 |
| Opal | 5.03 | Siderite | 52.31 | Muscovite | 16.85 |
| Garnet | 44.9 | Orthoclase | 15.51 | Glauconite | 24.79 |
| Tourmaline | 7450 | Anorthoclase | 15.91 | Biotite | 29.83 |
| Zircon | 6.92 | Microcline | 15.58 | Kaolinite | 14.12 |
| Calcite | 7.08 | Albite | 7.47 | Chlorite | 24.87 |
| Dolomite | 4.7 | | | Illite | 17.58 |
| | | | | **Montmorillonite** | **?.12 — TRUNCATED** |

**RULE 8 — this is the only smectite/montmorillonite endpoint anywhere in agent-H scope, and its
leading digit(s) are cut off by the screenshot's ragged bottom edge.** See §8 OPEN-1.

### 3.2 NMR Interpretation

| Tab | Parameter | Value | Unit | Source |
|---|---|---|---|---|
| T2 Cutoffs | Free Fluid Cut | 90 | ms | `[img-read: _nmriclip0007.png]` [dialog] |
| T2 Cutoffs | Cly B Fluid Cut | 3 | ms | `[img-read: _nmriclip0007.png]` [dialog] |
| T2 Cutoffs | BF Method | T2 cutoff / Tapered / Maximum | — | `nmrinterpretation.htm` |
| T2 Cutoffs | Tapered Constants | Sand / Carbonate / User | — | `nmrinterpretation.htm` |
| T2 Cutoffs | Tapered m (Sand) | **0.0618** | — | `nmrinterpretation.htm` [prose] |
| T2 Cutoffs | Tapered b (Sand) | **1** | — | `nmrinterpretation.htm` [prose] |
| T2 Cutoffs | Tapered m (Carbonate) | **0.0113** | — | `nmrinterpretation.htm` [prose] |
| T2 Cutoffs | Tapered b (Carbonate) | **1** | — | `nmrinterpretation.htm` [prose] |
| Permeability | Perm Equation | Timur-Coates | — | `[img-read: _nmriclip0008.png]` [dialog] |
| Permeability | Perm Const `a` | 10000 | — | `[img-read: _nmriclip0008.png]` [dialog] |
| Permeability | Perm FF/BF Exp `b` | 2 | — | `[img-read: _nmriclip0008.png]` [dialog] |
| Permeability | Perm Phi Exp `c` | 4 | — | `[img-read: _nmriclip0008.png]` [dialog] |
| Permeability | Perm `d` Const | 1 | — | `[img-read: _nmriclip0008.png]` [dialog] |
| Sw Params | `a` | 1 | — | `[img-read: _nmriclip0009.png]` [dialog] |
| Sw Params | `m` | 2 | — | `[img-read: _nmriclip0009.png]` [dialog] |
| Sw Params | `n` | 2 | — | `[img-read: _nmriclip0009.png]` [dialog] |
| Sw Params | m/n Method | Z Function | — | `[img-read: _nmriclip0009.png]` [dialog] |
| Sw Params | **Z wet** | **2.0** | — | `nmrinterpretation.htm` [prose] |
| Sw Params | **Z irreducible** | **1.6** | — | `nmrinterpretation.htm` [prose] |
| Sw Fluids | Rw | 0.0811 @ 60 (≈114,000 ppm) | ohm-m | `[img-read: _nmriclip0010.png]` [dialog] |
| Sw Fluids | Rwb | 0.0993 @ 60 (≈88,500 ppm) | ohm-m | `[img-read: _nmriclip0010.png]` [dialog] |
| Sw Fluids | Rw / Rw Temp fallback | **0.1 at 60 deg** if Well Header blank | ohm-m | `nmrinterpretation.htm` [prose] |
| Sw Limits | **Swi Limit** | **0.0** | v/v | `nmrinterpretation.htm` [prose] |
| Sw Limits | **FF Sw Limit** | **0.005** | v/v | `nmrinterpretation.htm` [prose] |
| Sw Limits | **Phi Sw Limit** | **0.0** | v/v | `nmrinterpretation.htm` [prose] |
| Sw Limits | **CBW Sw Limit** | **1.0** | v/v | `nmrinterpretation.htm` [prose] |
| Phi Logic | Phi Method / CBF Method | NMR | — | `[img-read: _nmriclip0042.png]` [dialog] |
| Phi Logic | PhiT Clay | 0.1 | v/v | `[img-read: _nmriclip0042.png]` [dialog] |
| Phi Logic | HI Water | 1 | — | `[img-read: _nmriclip0042.png]` [dialog] |
| LHC Setup | Matrix Density | 2.65 | g/cc | `[img-read: _nmriclip0043.png]` [dialog] |
| LHC Setup | Wet Clay Density | 2.5 | g/cc | `[img-read: _nmriclip0043.png]` [dialog] |
| LHC Setup | Polarisation Time | 10.241 | s | `[img-read: _nmriclip0043.png]` [dialog] |
| LHC Fluids | Rho LHC | 0.2 | g/cc | `[img-read: _nmriclip0044.png]` [dialog] |
| LHC Fluids | HI LHC | 0.3 | — | `[img-read: _nmriclip0044.png]` [dialog] |
| LHC Fluids | T1 LHC | 3000 | ms | `[img-read: _nmriclip0044.png]` [dialog] |
| LHC Fluids | Rho Water / HI Water / T1 Water | 1 / 1 / 500 | g/cc, —, ms | `[img-read: _nmriclip0044.png]` [dialog] |
| T2 Wet | T2 Hydrocarbon | 800 | ms | `[img-read: _nmriclip0050.png]` [dialog] |
| T2 Wet | HC Std Dev / Water Std Dev | 0.3 / 0.5 | — | `[img-read: _nmriclip0050.png]` [dialog] |
| T2 Wet | T2 Water Low / High Limit | 33 / 3000 | ms | `[img-read: _nmriclip0050.png]` [dialog] |
| T2 Wet | Calib Log Mean Gain / Offset | 1 / 1.5 | — | `[img-read: _nmriclip0050.png]` [dialog] |
| Pc Curves | FWL TVDss | 2500 | (depth) | `[img-read: _nmriclip0012.png]` [dialog] |
| Pc Curves | Water Density / Hyd Density | 1 / 0.8 | g/cc | `[img-read: _nmriclip0012.png]` [dialog] |
| Pc Curves | Contact Angle | 30 | deg | `[img-read: _nmriclip0012.png]` [dialog] |
| Pc Curves | IFT | 30 | dynes/cm | `[img-read: _nmriclip0012.png]` [dialog] |
| Pc Curves | Peak Threshold | 0.02 | — | `[img-read: _nmriclip0012.png]` [dialog] |
| Pc Curves | Pore-size bins 1/2/3 | 0.01–100 / 0.5–2 / 2–10 | µm | `[img-read: _nmriclip0012.png]` [dialog] |
| Advanced Setup | Preset | "IP Normalized Default (30 bin)" | — | `[img-read: _nmriclip0002.png]` [dialog] |
| Advanced Setup | Start/Stop sample | 1 / 30 | — | `[img-read: _nmriclip0002.png]` [dialog] |
| Advanced Setup | Start/Stop T2 time | 0.3 / 3000 | ms | `[img-read: _nmriclip0002.png]` [dialog] |
| Advanced Setup | Bin mid-point times | ON | — | `[img-read: _nmriclip0002.png]` [dialog] |
| Tool defaults | CMR start/stop | 3 / 3000 | ms | `nmrinterpretation.htm`, `nmrnormalization.htm` [prose] |
| Tool defaults | CMR-200 / CMR-plus start/stop | 0.3 / 3000 | ms | as above [prose] |
| Tool defaults | CMR-plus 90-sample encoding | use samples 61–90 | — | `nmrnormalization.htm` [prose] |

**Convergence limits** (`nmrinterpretation.htm`): the Z equation loop is capped at **20 iterations**
(SWLOGICFLAG 1 on failure); the SwT equation loop at **10 iterations** (SWLOGICFLAG 8).

**SWLOGICFLAG dictionary:** 0 normal · 1 Z loop failed to converge in 20 · 2 SwT clipped to Swi
limit (SwU/SwTU unchanged) · 3 SwT forced to 1.0, FF below limit · 4 SwT forced to 1.0, Phie below
limit · 5 SwT forced to 1.0, CBW above limit · 8 SwT loop failed to converge in 10.

**Monte Carlo numeric-parameter index (new in 2025)** — 55 addressable parameters, numbers 1–84
with gaps: 1 FF Cutoff, 2 CBF Cutoff, 3–5 Perm a/b/c, 8–9 Tapered m/b, 10–12 a/m/n, 14–15 Z wet/Z
irreducible, 16–17 Rw/Rw Temp, 19–20 Rwb/Rwb Temp, 23–26 Swi/FF/Phi/CBW Sw Limits, 32 PhiT Clay,
38 Rw Form Temp, 39–40 nmr-to-Pc Offset/Gain, 41 FWL TVDss, 42–43 Water/Hyd Density, 44 Rwb @ Temp,
49–50 Matrix/Wet Clay Density, 52 Polarisation Time, 53–55 Rho/HI/T1 Hydrocarbon, 56–58 Rho/HI/T1
Water, 60 Perm d, 62–66 T2 HC / HC StdDev / Water StdDev / Water Low / Water High, 67–68 Calib Log
Mean Gain/Offset, 69–70 Contact Angle / IFT, 72 Sxo input value, 73–74 Rw/Rwb Salinity, 79–84 pore-
size bin 1/2/3 start & stop (`nmrinterpretation.htm`).

### 3.3 TOC

| Parameter | Default | Unit | Source |
|---|---|---|---|
| RhoShale | **2.71** | g/cc | `total_organic_carbon_content.htm` [prose] |
| RegMethod | **3rd order** (or 5th order) | — | `total_organic_carbon_content.htm` [prose] |
| Use flags (×14) | **all ON** | — | `total_organic_carbon_content.htm` [prose] |
| Level Of Maturity (LOM) | **10.6** | — | `total_organic_carbon_content.htm` [prose] |
| Passey Mod | **0.8** | — | `total_organic_carbon_content.htm` [prose] |
| DT offset | **0** | — | `total_organic_carbon_content.htm` [prose] |
| RHOB offset | **0** | — | `total_organic_carbon_content.htm` [prose] |
| NPHI offset | **0** | — | `total_organic_carbon_content.htm` [prose] |
| Final TOC | **DlogRavg** (of DenAvg / DenReg / DlogRavg / DlogRcombo) | — | `total_organic_carbon_content.htm` [prose] |
| Xfactor | **0.357** | v/v per wt% TOC | `total_organic_carbon_content.htm` [prose] |

The `Passey Mod` default of 0.8 is described as a later empirical modification: the original ΔlogR
calibration data covered **only low LOM** and was designed to return TOC = 0 where the curves
overlie, but an offset of 0.8 was sometimes found to fit better.
`Xfactor = 0.357` is stated to usually match pyrite volumes from core XRD.

### 3.4 Unconventional Reservoirs

**Rock Mechanics** — no numeric parameters at all; only Zones / Units / Options tabs
`[img-read: _ucrclip0011.png]`.

**Gas Pressure** `[img-read: _ucrclip0019.png]` [dialog]: Temp @ Datum 100, Temp Gradient 0.04,
Pressure @ Datum 40000, Datum SS **−2000**, Surface Elevation 500, Gas Gravity 0.556, H2S 0, N2 0,
CO2 0, Cond Liquid Loading 0, Condensate Gravity 0.8.
(The negative Datum SS is what makes the minus sign in eq 8/10 self-consistent — see §5 D-9.)

**Fluid Properties** `[img-read: _ucrclip0035.png]` [dialog]: Water Salinity 100000 ppm, Gas Gravity
0.556, H2S/N2/CO2 0, Cond Liquid Loading 0, Condensate Gravity 0.8, Oil Gravity 0.85, Oil SolGas 100.

**Gas Storage — Adsorption** `[img-read: _ucrclip0045.png]` [dialog]: `GsLo` C1/C2/C3/N2/CO2 all
**60**; `pL` C1/C2/C3/N2/CO2 all **7000**.
**Ads. Gas Comp** `[img-read: _ucrclip0046.png]`: C1 = 1, C2 = C3 = N2 = CO2 = 0.
**Free Gas Comp** `[img-read: _ucrclip0047.png]`: Gas Gravity 0.7, free H2S/N2/CO2 0, Liquid Loading
0, Condensate Gravity 0.85.
**Parameters** `[img-read: _ucrclip0048.png]`: TOC Density **1250** (kg/m3), Water Salinity 100000
ppm, Area 2.59, NetMin Clay 0, NetMax Clay 0.5, NetMin Por 0.001, NetMax Por 0.25, PayMin Sw 0,
PayMax Sw 0.75.

**Oil Storage** `[img-read: _ucrclip0054.png]` [dialog]: Oil Gravity 0.85, Oil Sol Gas 9, Water
Salinity 25000 ppm, Area 2.59, NetMin Clay 0, NetMax Clay 0.5, NetMin Por 0, NetMax Por 0.35,
PayMin Sw 0, PayMax Sw 0.75.

**Calculate Shale Limits** `[img-read: _ucrclip0062.png]` [dialog]: Calcite 0–1, Clay 0–0.5,
Quartz 0–1, Porosity 0.001–0.5, Water Sat 0–0.75, HC-Filled Porosity 0–1, Gamma Ray 0–1000,
Density 1000–5000 (kg/m3), Neutron Porosity 0–1, Resistivity 0.01–2000 ohm-m, H-V Stress 0–1.

**Valid input ranges as printed** (`ucr.htm`) — these are hard validation bounds, worth mirroring:

| Parameter | Range | Unit |
|---|---|---|
| Temp Gradient | 0–5 / 0–9 | °C/m / °F/ft |
| Gas Gravity (Fluid Props) | 0.5538 (methane) – 4 | rel. air |
| Gas Gravity (Free Gas Comp) | 0.556 (methane) – 2 | rel. air |
| H2S conc (Gas Press / Fluid Props) | 0–0.77 / 0–74 | frac / mole % |
| N2 conc | 0–0.25 / 0–25 | frac / mole % |
| CO2 conc | 0–0.255 / 0–25.5 | frac / mole % |
| Free H2S / N2 / CO2 conc (Free Gas Comp) | 0–0.7 / 0–70 | frac / mole % |
| Cond Liquid Loading | 0–1,000 / 0–178,100 | m3/m3 / bbl/MMscf |
| Condensate Gravity | 35–70.6 | API |
| Oil Gravity | 0.0702–1.037 / 5–70 | rel. water / API |
| Sol Gas Ratio | 0–1000 / 0–5614.6 | m3/m3 / scf/STB |
| Water Salinity | 10–250,000 | ppm |
| TOC Density | 1–3 | g/cm3 |
| Area | 0.001–100,000 / 0.001–2.471×10⁷ | km2 / acres |
| NetMin/Max Porosity | 0.001–0.5 | frac |
| Langmuir storage capacity `GsLo` | 0–10,000 / *"(between 320,369)"* | cm3/g / scf/ton |
| Langmuir pressure `pL` | 0.001–100,000 / 0.001–14,503.7 | kPaa / psia |
| Gamma Ray (Shale Limits) | 0–5000 | API |
| Density (Shale Limits) | 0–10 / 0–10000 | g/cm3 / kg/m3 |
| Resistivity (Shale Limits) | 0.001–20000 | ohm-m |
| H-V stress ratio (Shale Limits) | 0–10 | frac |

The `scf/ton` Langmuir range is malformed in the source and repeated identically five times —
see §5 D-13.

### 3.5 CO2 Storage

`[img-read: _co2clip0004.png, _co2clip0005.png]` [dialog]: Resource Type Saline Aquifer, Area 1
(km2), Rho CO2 0.5, Rho Water 1.05, Inaccessible Pores = **BVI**, Swirr 0.2, BVI 0.1, `E_CO2` 0.05,
`Es_CO2` 0.1, Solubility 0.6 mol/kg. **No values are called defaults in prose** — the page states
only unit conventions (§8 OPEN-8).

Density unit is *not* set on this dialog — it is inherited from **Tools > Default Units**
(g/cc or kg/m3) and converted internally (`co2storagecapacity.htm`).

---

## 4. Assumptions & validity limits

**Sigma / TDT**
- Sw-from-sigma is a **linear volumetric mix** of matrix, clay, water and hydrocarbon capture
  cross-sections. No shale-conductivity or excess-conductivity analogue exists in this chain.
- Matrix fraction is `(1 − Vcl − Phi)`; **Vcl is a bulk-volume fraction**, and `SigClay` is the
  wet-clay endpoint.
- The `SigMat`-from-`RhoMat` branches are **clamped** (`SigDol ≥ SigMat ≥ SigSand` for SS-DOL;
  ρma clamped to 2.65 / 2.71 / 2.85 for SS-LS-DOL), so out-of-range matrix densities silently
  saturate rather than extrapolate.
- Time-lapse ΔSw is **zeroed** wherever `Phi < PhiCutoff` or `Vcl > VclCutoff` — non-reservoir is
  forced to no-change, not to null.

**NMR**
- Clay-bound porosity only splits `PhiT` from `Phie` if the tool actually measures clay fluid and the
  option is enabled; otherwise the two are identical.
- The straight T2 cut-off underestimates bound water in well-sorted coarse high-porosity sand
  (narrow single-exponential spectrum), driving permeability erroneously high — this is the stated
  reason the tapered method exists.
- The tapered method under-calls bound water in hydrocarbon zones at irreducible saturation;
  `BF Method = Maximum` is the stated remedy.
- LHC: **water-wet rock assumed**, all gas in the free fluid, bound water 100 % polarised.
- Z function: valid as a **linear approximation** to the true `Z(Sw)` curve except at very low Sw;
  `Z = m` exactly at `SwT = 1`.
- The manual warns explicitly that users *"should exercise caution when adjusting these parameters
  from the defaults"* (Z wet 2.0, Z irreducible 1.6) — the Z-wet/Z-irreducible difference is the
  gain on Sw and is the single most optimism-prone knob in the module.
- T2→Pc: *"will only be as good as the calibration of the T2 to Pc curve. This calibration will
  change with rock type. Hence it is highly recommended to use these results with caution."*
  The Sw axis assumes the **highest T2 bin is 100 % Sw** and the lowest is 0 % — a normalisation
  assumption, not a measurement.
- Ccw / `Rwb` temperature relation is defined in **degF** only.

**TOC**
- ΔlogR was developed on **low-LOM data only**; the `Passey Mod` offset is an empirical patch.
- Core TOC and core heavy-mineral volumes are **plotted for comparison but never used in a
  regression fit** — the calibration is entirely by eye through LOM / Passey Mod / offsets.
- The TOC→PhiSw handoff assumes TOC in **wt%** and `VHVY_EMP` in **v/v**; changing either
  calibration silently invalidates the PhiSw Organic Shale defaults.

**UCR**
- `sigma_h/sigma_v = mu/(1−mu)` ignores tectonic, thermal and all other stresses.
- No static correction is applied to Poisson's ratio (justified from Barree et al.).
- Brittleness end-points (1–8 Mpsi, 0.4–0.15) are Barnett-calibrated by provenance; the SPE 115258
  title itself is *"All Shale Plays Are Not Clones of the Barnett Shale"*.
- Langmuir storage capacity is **temperature-dependent and IP does not correct for it**. The manual
  states there is no simple method; the recommended workarounds are (a) measure isotherms at an
  average temperature of interest, or (b) use multiple zones with different Langmuir parameters.
- Adsorbed gas occupies part of the gas-filled porosity; the `Compute Ads Sg` option (default ON)
  removes it, and the manual says this *"is the recommended option and should be used for correct
  computations"* — i.e. leaving it off double-counts.
- Gas and water saturations sum to one, i.e. **a liquid hydrocarbon phase is assumed absent** in the
  Gas Storage module (stated verbatim for the Water Saturation input).
- Condensate recombination assumes **no non-hydrocarbons in the condensate**.
- Coalbed-methane usage: set the TOC curve to coal organic content
  (`1 − ash − sulfur − equilibrium moisture`, weight fractions) and porosity to a low value such as
  **0.001**; free gas becomes negligible and the adsorbed capacity represents the coal.
- To exclude adsorption in a conventional reservoir: zero TOC curve, **or** set the organic-fraction
  Langmuir capacities to **0.001**.
- `Bg` is not applied in the CO2 module because densities are already at downhole conditions.

**Gas analysis**
- Total Gas, ROP, RPM and GR are correlation-only and enter no calculation.
- Generic C4/C5 must go in the **normal** boxes only; duplicating into the iso boxes double-counts.

---

## 5. Internal discrepancies

**D-1 — Sigma: five-mineral prose vs six-column dialog.** The prose equation and the `Mineral Vols`
option both describe `MinVol1..MinVol5` / `SigMin1..SigMin5` (`sigma.htm`), but the parameter dialog
`[img-read: _chclip0006.png]` exposes **six** columns, *Sig Mat Min 1* through *Sig Mat Min 6*.
Present identically in 2018 and 2025, so it is a persistent vendor inconsistency, not an artefact.

**D-2 — Sigma: no equation image for the Mineral Vols branch.** All other `SigMat` sources have a
rendered equation; the mineral-volume branch is ASCII only, with no statement of whether the volumes
are normalised or whether porosity is excluded.

**D-3 — Sigma: montmorillonite value truncated.** `[img-read: _chclip0006.png]` — see OPEN-1.

**D-4 — NMR: no stated default for either T2 cut-off.** The prose describes `Free Fluid Cut` and
`Cly B Fluid Cut` and never gives a value. The 90 / 3 ms pair appears only in a dialog screenshot
and in an example parameter printout (`nmrinterpretation.htm` line 969), both from the same
carbonate demonstration well.

**D-5 — NMR: no stated default for perm a/b/c/d.** Same pattern; only the dialog's
10000 / 2 / 4 / 1 exists.

**D-6 — NMR: Coates tapered-cutoff citation year.** Printed as *"paper QQ 38th Annual SPWLA
Symposium 1977"*. The 38th SPWLA Annual Logging Symposium was **1997**. Identical text in 2018.

**D-7 — NMR: missing parentheses in the polarisation rasters.** `[img-read: embim181.png,
embim182.png]` render `Pg = HIg * 1 − e^(−Tpol/T1g)` with no bracket around `(1 − e^…)`. The ASCII
form on the same page (`PHC = HI_LHC * (1 − e^(−Tw/T1_LHC))`) shows the intended grouping. The
raster as drawn is dimensionally wrong; the ASCII is correct.

**D-8 — TOC: wt% claimed, fractions computed.** Thirteen of the fourteen density regressions end in
`* .01`, and `TOC_MODE` is natively a fraction, so every relationship returns a **fraction**. The
prose nonetheless states outputs are *"in weight percentage units"* and *"dry weight percent
(wt%)"*. Either the prose means "wt% expressed as a fraction" or the units label is wrong; the
manual never resolves it. **This is a live unit trap for anyone porting these regressions.**

**D-9 — UCR: sign of the temperature equation.** `T = T_datum − ΔT(d − d_datum)` (eq 8 and 10,
verified at 5× upscale) with the symbol list defining `d` as *"true vertical depth at top of
interval"* and `d_datum` as *"true vertical depth relative to sea level at the datum"*. With
depth-positive-down this makes temperature **fall** with depth. The shipped dialog has
`Datum SS = −2000` `[img-read: _ucrclip0019.png]`, which implies both `d` and `d_datum` are
**subsea elevations (negative downward)**, making the minus correct. The manual never states the
sign convention.

**D-10 — UCR eq 14-3 subscript.** Raster reads `G_svg = O_vs·R_s + W_sv·R_sw`; the symbol list
defines `O_sv`. Typo in the raster.

**D-11 — Gas analysis: GWR denominator omits C3 and C4.** `GWR = (C2+C3+C4+C5)/(C1+C2+C5)*100`.
The denominator skips C3 and C4 while the numerator includes them. Identical in the 2018 manual, so
this is a **persistent vendor statement**, not an extraction error — but it should be checked
against Haworth (1985) before reimplementation.

**D-12 — Gas analysis: HCF 4/5 boundary undefined at OCQ = 0.5.** Both conditions are strict
(`OCQ < 0.5` and `OCQ > 0.5`). No branch covers exactly 0.5; presumably it falls to HCF 8
("no solution"), but the manual does not say.

**D-13 — UCR: malformed Langmuir range.** *"scf/ton (between 320,369)"* — a range with one number,
repeated verbatim for all five gases. Most plausibly "between 320 and 369", but 0–10,000 cm3/g maps
to roughly 0–320,369 scf/ton, so the intended text is almost certainly **"between 0 and 320,369"**.
Not resolvable from the page (§8 OPEN-6).

**D-14 — UCR eq 13-10 described as a mole-fraction weighted average, rendered as a harmonic sum.**
Prose: *"The adsorbed density is a mole-fraction weighted average of the adsorbed density of each
component."* Raster: `rho_s = [ Σ (x_i / rho_si) ]^-1`. That is a **reciprocal (harmonic) mixing
rule**, i.e. the correct form for mixing *specific volumes*, not an arithmetic weighted average.
The equation is the physically sensible one; the prose description is wrong.

**D-15 — UCR eq 13-5 subscript.** Raster reads `G_sga = Σ G_sai`; every other equation uses
`G_sgai`. Cosmetic.

**D-16 — UCR eq 5-7 symbol.** Raster uses bare `ρ`; the symbol list only defines `ρ_b`.

**D-17 — TOC dialogs show example values that contradict the stated defaults.**
`[img-read: _intclip0041.png, _intclip0042.png]` (parameter set "UsrProgTOC") show LOM **9.53**,
Passey Mod **0.2**, DT offset **0.571**, RHOB offset **0.679**, NPHI offset **0.5**, and only
**10 of 14** Use flags ticked — against stated defaults of 10.6 / 0.8 / 0 / 0 / 0 / all-14-on.
`[img-read: _intclip0043.png]` (Final tab) does agree with the stated defaults: DlogRavg,
Xfactor 0.357. Anyone reading defaults off the screenshots would get five wrong numbers.

---

## 6. IP2018 numeric diff

Counterparts exist for 6 of 8 pages. `co2storagecapacity.htm` and `mud-gas-normalization.htm` are
**new in 2025** with no 2018 baseline.

**Method.** ASCII text was extracted from both `.htm` trees with an identical parser, navigation
chrome stripped, and set-differenced. Equation rasters were compared by (a) direct PIL pixel-diff
where the 2018 and 2025 images are the same format and dimensions, and (b) direct vision reads of
the 2018 `.gif` where the 2025 `.png` had been re-rendered at higher DPI.

| Page | Text delta | Raster delta | Verdict |
|---|---|---|---|
| `sigma.htm` | nav chrome + menu relocation only (2018 "Cased Hole Interpretation Tools" under Interpretation > Cased Hole; 2025 launchable from **either** the Interpretation or the Cased Hole menu) | `embim166–177.gif` → `embim196–207.png`, 1:1 in order, re-rendered at ~2× DPI (e.g. 380×71 → 752×134) | **No numeric change.** The three numeric rasters (`embim170/171/172.gif`, the SS-DOL and SS-LS-DOL ρma interpolations and the SwTDTU inversion) were read directly by vision and are character-for-character identical to the 2025 versions, including the 2.65 / 2.71 / 2.85 anchors. |
| `nmrnormalization.htm` | nav chrome + menu relocation only (Advanced Interpretation → Interpretation) + image-naming convention `_nmrclip0006.zoom50.png` → `_nmrclip0006_zoom50.png` | none | **No numeric change.** |
| `nmrinterpretation.htm` | 1,177 → 1,972 extracted lines. Substantial new content: T2 Wet fluid substitution, Pc multi-plug calibration, the LHC section, and the Monte Carlo parameter index (`grep -c "Monte Carlo"` on the 2018 dump = **0**). | perm equations re-read from the 2018 `.gif`s: identical | **One genuine numeric documentation change** — see below. |
| `total_organic_carbon_content.htm` | none of substance | all 14 regressions ASCII in both | **No numeric change.** |
| `ucr.htm` | none of substance | all 43 `equationN_zoom75` rasters same size in both trees; mean per-channel difference 0.03–0.13 / 255 (JPEG/PNG recompression noise only) | **No numeric change.** |
| `gas_analysis.htm` | none of substance | — | **No numeric change**, including the GWR denominator (D-11). |

**The one genuine numeric change, and it matters:**

> NMR Z-function guidance, `nmrinterpretation.htm`
> **IP2018:** *"Z irreducible is typically between **0.2 and 0.5**."*
> **IP2025:** *"Z irreducible is typically between **1.5 and 1.8**."*

The 2018 statement was internally inconsistent with its own shipped default of `Z irreducible = 1.6`
and `Z wet = 2.0`, and with the worked example in the same section (Z wet 2, Z irreducible 1.7 from a
40 ft sand). 2025 corrects the guidance to match. **Anyone who calibrated an NMR Sw against the 2018
text's stated range was working from a wrong number.** The defaults themselves never changed.

---

## 7. SandiBumi notes

1. **Sigma closes agent C's gap.** `sigma.htm` is the only place in this manual that documents
   cased-hole capture cross-sections, and it does so fully: fluid endpoints (water 80, hyd 20, clay
   25 CU), the three lithology endpoints tied to their matrix densities (4.3 @ 2.65 / 7.1 @ 2.71 /
   4.7 @ 2.85), a 21-mineral sigma library, the ρma→SigMat interpolation with its clamps, and the
   full SwTDTU inversion. If SandiBumi wants a TDT module, this section is implementable as-is —
   with the caveat that the endpoints are *IP's* endpoints and still need tracing to Schlumberger
   Tcor-1/Tcor-2 before shipping as SandiBumi defaults.

2. **Do not port the NMR T2 cut-offs or permeability coefficients as defaults.** The manual states
   neither. 90 ms / 3 ms and a=10000, b=2, c=4 come from one carbonate demo well. The *only*
   NMR constants the manual actually calls defaults are the tapered coefficients
   (Sand 0.0618/1, Carbonate 0.0113/1 — traceable to a 340-sandstone + 71-carbonate core study),
   `Z wet 2.0` / `Z irreducible 1.6`, the four Sw limits (0 / 0.005 / 0 / 1), and the CMR start/stop
   times. Everything else must come from Jauhar's own sources or be asked about.

3. **The `*.01` vs "wt%" tension in TOC (D-8) is exactly the class of silent failure this project
   exists to avoid.** A SandiBumi TOC module must pick one convention, state it at the API boundary,
   and carry a unit tag on the curve — not inherit IP's ambiguity. Note also that IP's PhiSw
   Organic Shale defaults are tuned to *TOC in wt%, heavy mineral in v/v*, which cross-references
   the `Kerogen Wt%_Con. default 2.5` and `heavy mineral default −0.03 v/v` entries already recorded
   in the 2018 anchor JSON.

4. **The IP2018 anchor does not cover this scope.** `H_module_parameter_reference.json` contains
   only `ClayVol`, `PhiSw` and `Cutoff` modules; zero hits for sigma, TDT, NMR, Coates, Timur,
   Langmuir or Passey. Everything in this report is new ground relative to that file, so there is
   nothing here to reconcile against it — but equally, nothing here was corroborated by it.

5. **Mud-gas normalisation is a one-line win.** `Gas_norm = (Gas × Flowrate × 5.0028)/(ROP × D²)`
   applies directly to the SCS-PHM mudlog work. It cannot be shipped until the unit set behind
   `5.0028` is pinned (OPEN-9) — the constant is meaningless without it, and this is precisely the
   "validate on physical bounds, not names" trap already recorded for mudlog gas curves.

6. **Gas-analysis HCF is a clean, fully-specified decision table** (nine states, four discriminant
   constants) and is directly implementable — with D-11 (the GWR denominator) checked against
   Haworth (1985) first, and D-12 (the OCQ = 0.5 boundary) decided explicitly rather than inherited.

7. **CO2 storage is genuinely new vendor content** and its equation chain is complete and
   self-consistent: two terms, four hard constants (4046.8564, 10⁶, 44.01, 1000), one branch
   (Swirr vs BVI), Metric Tonne output. The one conceptual soft spot is that `Es_CO2` is asked to
   absorb the water volume already displaced by the fluid term — the two terms are not mass-balanced.

8. **The UCR suite is Apache Corporation IP licensed into Interactive Petrophysics.** Any SandiBumi
   equivalent should be built from the primary references (SPE 115258, SPE 118701, Whitson & Brulé,
   Passey 1990) rather than from this manual's rendering of them.

---

## 8. OPEN ITEMS

**OPEN-1 (RULE 8, highest priority) — Montmorillonite sigma value is truncated.**
`[img-read: _chclip0006.png]` The Sig Mat Min dropdown's last visible row reads
`…12  Montmorillonite` with the leading digit(s) clipped by the screenshot's ragged bottom edge.
A tight 10× NEAREST crop was attempted; the pixels are genuinely absent. The c18 copy of the same
image was pixel-diffed and is **byte-identical** (`ImageChops.difference(...).getbbox()` → `None`),
so no alternate rendering exists in either manual. Neighbouring clays are Kaolinite 14.12,
Chlorite 24.87, Illite 17.58, so the true value plausibly sits in the low tens — **but this is
inference, not evidence, and it is not recorded here as a value.** Resolution requires the live IP
2025 install's Sigma module dropdown, or `MINDEF`/mineral-table files on disk.
*No smectite or montmorillonite endpoint of any other kind appears in agent-H scope.*

**OPEN-2 (cross-agent, RULE 8) — smectite endpoints live outside my pages.** A corpus grep found
`montmorillonite`/`smectite` in ASCII on `density_estimation.htm`, `overburden_tools.htm`,
`acoustic_to_pressure.htm`, `resistivity_to_pressure.htm`, `references_and_appencices.htm` and
`intro_whats_new_in_ip.htm` — none of them mine. `density_estimation.htm` documents an **Alberty
Smectite/Illite** acoustic-to-density model that outputs separate smectite and illite densities plus
a smectite volume, interpolating on formation temperature (K-feldspar breakdown, **160 degF onset,
220 degF complete**); `overburden_tools.htm` adds the Katahara (2013) counterpart and states both
were calibrated to **GoM Miocene and younger**. Whichever agent owns those pages should capture the
endpoints for the SandiMin clay review.

**OPEN-3 — NMR permeability coefficients: no documented defaults.** a/b/c/d = 10000/2/4/1 exist only
in a dialog screenshot. Do not treat as vendor defaults.

**OPEN-4 — TOC ΔlogR equations are not printed.** The manual gives only the Passey et al. (1990)
citation. Missing: the ΔlogR separation equation itself, the `TOC = f(ΔlogR, LOM)` relationship and
its two constants, and the weighting scheme behind `TOC_DlogRcombo` (as distinct from the plain
average `TOC_DlogRavg`). Must come from the AAPG paper, not from this manual.

**OPEN-5 — TOC "proprietary" 3rd- and 5th-order density regressions.** `TOC_DenReg` coefficients are
withheld by the vendor. Unrecoverable from the manual.

**OPEN-6 — Langmuir `scf/ton` valid range is malformed.** *"(between 320,369)"*, repeated five times.
See D-13.

**OPEN-7 — no per-lithology T2 cut-off defaults anywhere in the manual.** Grepped for `33 ms`,
`92 ms`, `default cut-off`, `sandstone.*cut`, `carbonate.*cut` — one unrelated hit. Only the
**tapered** constants are lithology-keyed. **The conventional 33 ms sandstone / 92 ms carbonate
values are NOT in this manual and have deliberately not been written into this report.** If
SandiBumi needs lithology-keyed T2 cut-offs they must come from a cited source of Jauhar's.

**OPEN-8 — CO2 module: no stated defaults.** Every value in §3.5 is dialog-only. The page documents
units and conversions thoroughly but never says "the default is".

**OPEN-9 — mud-gas normalisation: units behind `5.0028` unstated.** The page gives the formula and
the required inputs but not whether Flowrate is gpm or L/min, ROP m/hr or ft/hr, or Diameter inches
or mm. The constant is unusable until this is pinned — most likely from the Kandel et al.
SPE 65176 (2000) reference the page cites.

**OPEN-10 — Sigma `Mineral Vols` branch has no rendered equation** and no statement of volume
normalisation or porosity handling (D-2), and the 5-vs-6 mineral-column mismatch (D-1) is unresolved.

**OPEN-11 — HCF colours 8 and 9 have no documented colour.** `[img-read: _ccclip1805.png]` is the
Haworth LHR/GWR crossplot with a **discrete nine-class HCF colour bar** along the bottom, ticked
1 → 9. The prose assigns colours only to HCF 1–7 (blue, aqua, fuchsia, green, brown, lime, navy);
the bar shows two further classes for **8 ("no solution") and 9 ("no case")** — rendered as a
dark steel-navy and an olive respectively — which the page never names. Minor, but a
reimplementation needs a defined colour for the two failure states rather than an inherited one.
*(The same image independently confirms the §2.7 discriminants: overlay lines sit at GWR 0.5, 17.5
and 40, LHR 100, and the diagonal LHR = GWR, on log axes spanning 0.1–1000 in both LHR and GWR.)*
