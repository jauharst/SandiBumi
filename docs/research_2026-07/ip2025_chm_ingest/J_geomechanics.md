# Agent J — Geomechanics / Rock Mechanics / Pore Pressure & Fracture Gradient

**Source:** Interactive Petrophysics 2025 vendor manual, decompiled CHM
(`c25\*_text.txt` clean text, `c25\*.htm` raw, `c25\*.png` equation rasters).
**IP2018 counterpart:** `c18\` (only 6 of my 22 pages have one).
**Provenance convention used throughout:**

- `(pagename.htm)` — fact taken from page prose / parameter text.
- `[img-read: file.png]` — equation transcribed by reading the raster directly (vision).
- `[img-read: c18\file.png]` — same, but from the 2018 manual, for version diffs.

Nothing in this report is filled in from textbook knowledge. Where a symbol or a
coefficient could not be resolved unambiguously it is in **§8 OPEN ITEMS**, not in
the equation tables.

---

## 1. Scope & page inventory

All 22 assigned pages were opened. 6 have an IP2018 counterpart; **16 are new in IP 2025**
(the whole PPFG Toolbox, the anisotropic-elastic module, rock-strength anisotropy, the
vertical-stress module, poro-/thermo-elastic stress, and fracture/fault stress).

| Page (`c25\<stem>`) | Chars | Imgs | Read | IP2018 counterpart | Content class |
|---|---:|---:|---|---|---|
| `rock_strength` | 52,369 | 83 | full | yes (DIFFERS) | UCS/TWC/friction-angle correlations, dyn↔sta moduli, compressibility, Biot |
| `wellbore_stability` | 50,672 | 58 | full | yes (DIFFERS) | 3 shear-failure criteria, MW window, breakout, stress polygon |
| `porepressurecalculations2` | 48,102 | 72 | full | yes (DIFFERS) | Single-well PP + 5 fracture-gradient models, Eaton Poisson polynomials |
| `multi_well_pore_pressure` | 37,088 | 42 | full | **none** | Eaton 4 variants, Bowers loading/unloading, NCT fitting |
| `rock_stress` | 30,209 | 28 | full | yes (DIFFERS) | ShMin 6 models, ShMax 3 models, tectonic strain |
| `overburden_tools` | 27,744 | 34 | full | **none** | Overburden RHOB models incl. **smectite/illite** |
| `anisotropic-elastic` | 20,125 | 102 | full | **none** | VTI-vertical / VTI-horizontal / TTI stiffness tensor, Thomsen, ANNIE |
| `resistivity_to_pressure` | 18,824 | 21 | full | **none** | Klein anisotropy, Arps, Rt shale picker, semi-log & Archie NCT |
| `acoustic_to_pressure` | 15,316 | 18 | full | **none** | Alberty/McLean, Bowers, Chapman, Eberhart-Phillips, Miller NCTs |
| `fractures_and_fault_stress` | 10,386 | 15 | full | **none** | Fracture-plane projection, critically-stressed test |
| `dxc_to_pressure` | 8,614 | 16 | full | **none** | Dxc from ROP, linear & power-law NCT |
| `poro-_and_thermo-elastic_stres` | 7,659 | 10 | full | **none** | Depletion/injection & thermal stress change |
| `rock_mechanics` | 7,325 | 6 | full | yes (DIFFERS) | Suite overview, calibration workflow |
| `rock_strength_anisotropy` | 5,590 | 6 | full | **none** | Planes-of-Weakness workflow |
| `vertical_stress` | 4,357 | 8 | full | **none** | Overburden integration, Amoco, Barker & Wood, lookup tables |
| `fracture_gradient` | 3,509 | 3 | full | **none** | Generalized Kf fracture-gradient form, 7 relationships |
| `geoengineering` | 2,598 | 1 | full | **none** | Menu map of the suite |
| `specify_rock_mechanics_options` | 2,381 | 2 | full | yes (DIFFERS) | Depth-reference options, LOT table |
| `stress` | 2,055 | 0 | full | **none** | Stress-workflow overview |
| `ppfg_toolkit` | 857 | 0 | full | **none** | PPFG Toolbox table of contents |
| `wellbore_stability2` | 745 | 1 | full | **none** | Iso/aniso workflow flowchart pointer |
| `rock-properties` | 702 | 0 | full | **none** | Rock-properties overview (2 modules: isotropic, anisotropic) |

**Tallies:** ~205 distinct equations/relations transcribed; ~92 named correlations;
~95 numeric defaults / valid ranges captured; 17 internal discrepancies; 13 OPEN ITEMS.

---

## 2. Equations & correlations per module

### 2.1 Isotropic Elastic and Strength (`rock_strength.htm`)

#### 2.1.1 UCS correlations — sandstone

All sandstone/shale/carbonate/dolomite UCS models below end in `* 145.038` unless
noted. **`145.038` is the MPa→psi conversion factor** — i.e. the published correlations
are natively in MPa and IP converts to psi. Any reimplementation that omits this is
wrong by 145×.

| Model | Equation (as printed) | Source |
|---|---|---|
| Sarda Phi | `(111.5 * EXP(-11.6 * PHI)) * 145.038` | `[img-read: rmnew_clip0045.png]` |
| Sarda NPHI | `(111.5 * EXP(-11.6 * NPHI)) * 145.038` | `[img-read: rmnew_clip0046.png]` |
| Formel Phi | `(43 - (140*PHI) + (63*PHI^2)) * 145.038` | `[img-read: rmnew_clip0047.png]` |
| Formel NPHI | `(43 - (140*NPHI) + (63*NPHI^2)) * 145.038` | `[img-read: rmnew_clip0048.png]` |
| Formel DT | `(140 - (2.1*DT) + (0.0083*DT^2)) * 145.038` | `[img-read: rmnew_clip0049.png]` |
| Vernik | `(277 * EXP(-10*PHI)) * 145.038` | `[img-read: rmnew_clip0050.png]` |
| Modified Vernik | `((254 - 204*VCL) * (1 - 2.7*PHI)^2) * 145.038` | `[img-read: rmnew_clip0051.png]` |
| Coates & Denoo | `ESTA * (0.008*VCL + 0.0045*(1-VCL))` — **no 145.038** | `[img-read: rmnew_clip0052.png]` |

#### 2.1.2 UCS correlations — shale

| Model | Equation | Source |
|---|---|---|
| Chang GOM | `(0.43 * (304.8/DT)^3.2) * 145.038` | `[img-read: rmnew_clip0053.png]` |
| Chang Global | `(1.35 * (304.8/DT)^2.6) * 145.038` | `[img-read: rmnew_clip0054.png]` |
| Chang High Porosity | `(0.286 * PHI^-1.762) * 145.038` | `[img-read: rmnew_clip0055.png]` |
| Lashkaripour & Dusseault | `(1.001 * PHI^-1.143) * 145.038` | `[img-read: rmnew_clip0056.png]` |
| Lal | `10*((304.8/DT) - 1) * 145.038` | `[img-read: rmnew_clip0057.png]` |
| Horsrud DT | `(0.77 * (304.8/DT)^2.93) * 145.038` | `[img-read: rmnew_clip0058.png]` |
| Horsrud Phi | `(2.922 * PHI^-0.96) * 145.038` | `[img-read: rmnew_clip0059.png]` |
| Horsrud Youngs | `0.0232 * ESTA^0.91` — **no 145.038** | `[img-read: rmnew_clip0060.png]` |

`304.8/DT` with DT in µs/ft is **Vp in km/s** (exact: 304.8 mm/ft ÷ µs → km/s).

#### 2.1.3 UCS correlations — carbonate & dolomite

| Model | Equation | Source |
|---|---|---|
| Golurev & Rabinovich | `10^(2.44 + (109.14/DT))` — **no 145.038** | `[img-read: rmnew_clip0061.png]` |
| Chang Limestone 1 | `143 * EXP(-6.95*PHI) * 145.038` | `[img-read: rmnew_clip0062.png]` |
| Chang Limestone 2 | `135.9 * EXP(-4.8*PHI) * 145.038` | `[img-read: rmnew_clip0063.png]` |
| Militzer & Stoll | `(7682/DT)^1.82` — **no 145.038** | `[img-read: rmnew_clip0064.png]` |
| Rzhewski | `40020 * (1 - 3*PHI)^2` — **no 145.038** | `[img-read: rmnew_clip0065.png]` |
| Chang Dolomite | `64 * ESTA^0.34` — **no 145.038** | `[img-read: rmnew_clip0066.png]` |

**Composite curve mechanics:** each zone carries a `Lithology Type` parameter plus four
model selectors (`UCS Sand Model`, `UCS Shale Model`, `UCS Carb Model`, `UCS Dolom Model`).
IP splices the selected per-lithology result into one `UCS` output curve (`rock_strength.htm`).

#### 2.1.4 Tensile strength and thick-walled cylinder

| Quantity | Equation | Source |
|---|---|---|
| Tensile strength T0 | `A * UCS^B` | `[img-read: rmnew_clip0176.png]` |
| TWC Veeken | `(1.18*(9.29e5*(RHOB/DT^2)) + 0.00226*(9.29e5*(RHOB/DT^2))^2) * 14.5` | `[img-read: rmnew_clip0067.png]` |

The Veeken trailing constant is **14.5, not 145.038** — verified twice (glyph
segmentation of the raster, and a physical-plausibility check: 14.5 gives TWC
2,752–14,570 psi over realistic RHOB/DT, 145.038 gives absurd values). The c18 and c25
copies of this raster are byte-identical (md5 `eb3b90cb01e8…`), so it is unchanged
since IP2018.

#### 2.1.5 Friction angle

| Model | Equation | Source |
|---|---|---|
| Sonic (Lal) | `θ = arcsin( ((304878/DT) - 1000) / ((304878/DT) + 1000) )` | `[img-read: rmnew_clip0068.png]` |
| Plumb | `θ = 26.5 - 37.4*(1 - φ - Vclay) + 62.1*(1 - φ - Vclay)^2` | `[img-read: rmnew_clip0069.png]` |
| Horsrud | `θ = 11*(304.8/DT) - 10.2` | `[img-read: rmnew_clip0070.png]` |
| Weingarten & Perkins | `57.75 - 105 * Φ` (ASCII in prose, no raster) | `rock_strength.htm` |

Note `304878` in the Lal friction-angle model = m/s using 3.28 ft/m, versus the exact
`304.8` km/s convention used everywhere else (0.026 % offset). See §5.

#### 2.1.6 Dynamic elastic moduli — IP 2025 outputs **Mpsi**

| Output | Equation | Source |
|---|---|---|
| `EC_DYN` (compressional modulus) | `1.34747e4 * RHOB / DTc^2` | `[img-read: rmnew_clip0072.png]` |
| `GDYN` (shear modulus) | `1.34747e4 * RHOB / DTs^2` | `[img-read: rmnew_clip0073.png]` |
| `KDYN` (bulk modulus) | `1.34747e4 * RHOB * (1/DTc^2 - 4/(3*DTs^2))` | `[img-read: rmnew_clip0074.png]` |
| `PRDYN` | `[0.5*(DTs/DTc)^2 - 1] / [(DTs/DTc)^2 - 1]` | `[img-read: rmnew_clip0075.png]` |
| `EDYN` | `2*GDYN*(1 + PRDYN)` | `[img-read: rmnew_clip0076.png]` |

RHOB g/cc, DT µs/ft → moduli in Mpsi.

#### 2.1.7 Static moduli

| Output | Equation | Source |
|---|---|---|
| `KSTA` | `ESTA / (3*(1 - 2*PRSTA))` | `[img-read: rmnew_clip0169.png]` |
| `GSTA` | `ESTA / (2*(1 + PRSTA))` | `[img-read: rmnew_clip0170.png]` |
| Lacy (Sand) | `ESTA = 0.0293*EDYN^2 + 0.4533*EDYN` | `[img-read: rmnew_clip0077.png]` |
| Lacy (Shale) | `ESTA = 0.0428*EDYN^2 + 0.2334*EDYN` | `[img-read: rmnew_clip0171.png]` |
| Lacy (Mixed) | `ESTA = 0.018*EDYN^2 + 0.422*EDYN` | `[img-read: rmnew_clip0172.png]` |
| Morales & Marcinew | `ESTA = 0.956*EDYN^0.69` | `[img-read: rmnew_clip0078.png]` |
| Generic | `ESTA = A * EDYN^B + C` | `[img-read: rmnew_clip0079.png]` |
| Static Poisson's ratio | `PRSTA = PRDYN * 'Poisson Factor'` | `[img-read: embim472.png]` |

The Lacy polynomials are **only dimensionally correct with E in Mpsi** (the quadratic
term has units of 1/modulus). This is a hard constraint on reimplementation.

#### 2.1.8 Compressibility & Biot

| Output | Equation | Source |
|---|---|---|
| `CB_DYN` | `3*(1 - 2*PRDYN)/EDYN` → microsips | `[img-read: rmnew_clip0080.png]` |
| `CP_DYN` isostatic | `(CB_DYN - (1+PHI)*Cg)/PHI` | `[img-read: rmnew_clip0081.png]` |
| `CP_DYN` uniaxial (Geertsma) | `CP_DYN_ISO **+** [BIOT_DYN*(1+PRDYN) / (3*(1-PRDYN))]` | `[img-read: rmnew_clip0082.png]` |
| `CP_DYN` uniaxial (Raaen) | `CP_DYN_ISO - (2(1-2PRDYN)BIOT_DYN)/(3(1-PRDYN)) * (CP_DYN_ISO + Cg)` | `[img-read: rmnew_clip0173.png]` |
| `BIOT_DYN` | `1 - Cg/CB_DYN` | `[img-read: rmnew_clip0083.png]` |
| `CB_STA` | `3*(1 - 2*PRSTA)/ESTA` | `[img-read: rmnew_clip0084.png]` |
| `CP_STA` isostatic | `(CB_STA - (1+PHI)*Cg)/PHI` | `[img-read: rmnew_clip0085.png]` |
| `CP_STA` uniaxial (Geertsma) | `CP_STA_ISO **×** [BIOT_STA*(1+PRSTA) / (3(1-PRSTA))]` | `[img-read: rmnew_clip0086.png]` |
| `CP_STA` uniaxial (Raaen) | `CP_STA_ISO - (2(1-2PRSTA)BIOT_STA)/(3(1-PRSTA)) * (CP_STA_ISO + Cg)` | `[img-read: rmnew_clip0174.png]` |
| `BIOT_STA` | `1 - Cg/CB_STA` | `[img-read: rmnew_clip0175.png]` |

The `+` vs `×` mismatch between the dynamic and static Geertsma forms is real and
**new in IP 2025** — see §5 D1 and §6.

Dependency flowchart `[img-read: rmnew_clip0071.png]`: Dynamic Moduli → Static Moduli →
Strength; Dynamic/Static Compressibility branch separately; Friction Angle is an
independent branch (not fed by moduli).

---

### 2.2 Anisotropic Elastic (`anisotropic-elastic.htm`) — NEW in IP 2025

Three model types, selected by the `Anisotropy Model Type` parameter:
**VTI Vertical Well** (TI parallel), **VTI Horizontal Well** (TI perpendicular),
**TTI Tilted**.

Slowness→velocity is uniformly `V = 1/DT`
(`[img-read: embim409/410/411/412/427/428/429/444/445/446.png]`).

#### 2.2.1 VTI Vertical Well — stiffnesses

| Stiffness | Equation | Source |
|---|---|---|
| `C33` | `Vp^2 * RHOB` | `[img-read: embim413.png]` |
| `C44` | `Vs^2 * RHOB` | `[img-read: embim414.png]` |
| `C66` | `RHOF * (Vst^2 * Vf^2)/(Vf^2 - Vst^2)` | `[img-read: embim415.png]` |
| `C11` | `C33 + 2*C66 - 2*C44` | `[img-read: embim416.png]` |
| `C12` | `C11 - 2*C66` | `[img-read: embim417.png]` |
| `C13` | `C12` | `[img-read: embim418.png]` |

`C66` comes from the **Stoneley** mode coupled to the drilling fluid — which is why
`Drilling Fluid Density` and `Drilling Fluid Slowness` are required *parameters*, not
optional. `C13 = C12` means the vertical-well VTI solution is itself an ANNIE-type
closure.

#### 2.2.2 VTI Horizontal Well — stiffnesses

| Stiffness | Equation | Source |
|---|---|---|
| `C11` | `Vp^2 * RHOB` | `[img-read: embim430.png]` |
| `C44` | `Vss^2 * RHOB` (slow shear) | `[img-read: embim431.png]` |
| `C66` | `Vsf^2 * RHOB` (fast shear) | `[img-read: embim432.png]` |
| `C12` | `C11 - 2*C66` | `[img-read: embim433.png]` |
| `C13` | `C12` | `[img-read: embim434.png]` |
| `C33` | `C11 + 2*C44 - 2*C66` | `[img-read: embim435.png]` |

#### 2.2.3 ANNIE approximation (Schoenberg, Muir & Sayers 1996)

Checkbox option; only `C33`, `C44`, `C66` are entered.

```
C12 = mult13 * C13                       [img-read: embim405.png]
C13 = mult33 * C33 - 2 * C44             [img-read: embim406.png]
where mult13 = mult33 = 1                [img-read: embim407.png]
C11 = C33 + 2*C66 - 2*C44                [img-read: embim408.png]
```

The `mult13` / `mult33` multipliers are exposed in the equation but **fixed at 1** in the
documented form — a generalized-ANNIE hook.

#### 2.2.4 Thomsen parameters (identical in all three models)

| Parameter | Equation | Source |
|---|---|---|
| `EPS_TH` (ε), VTI-vertical | `(C11 - C33) / (2*C33)` | `[img-read: embim419.png]` |
| `GAM_TH` (γ) | `(C66 - C44) / (2*C44)` | `[img-read: embim420.png]`, `[img-read: embim437.png]` |
| `DEL_TH` (δ) | `[(C13 + C44)^2 - (C33 - C44)^2] / [2*C33*(C33 - C44)]` | `[img-read: embim421.png]`, `[438]`, `[463]` |
| `EPS_TH` (ε), VTI-horizontal / TTI | `(C11 - C33) / (2*C33)` | `[img-read: embim436.png]`, `[462]` |

#### 2.2.5 Dynamic moduli and Poisson's ratios from the stiffness tensor

Identical in all three model types (`embim422–426`, `439–443`, `464–468`):

| Output | Equation | Source |
|---|---|---|
| `EDYN_VER` | `C33 - 2*C13^2/(C11 + C12)` | `[img-read: embim422.png]` |
| `EDYN_HOR` | `(C11 - C12)*(C11*C33 - 2*C13^2 + C12*C33) / (C11*C33 - C13^2)` | `[img-read: embim423.png]` |
| `PRDYN_VERVH` | `C13 / (C11 + C12)` | `[img-read: embim424.png]` |
| `PRDYN_HORHH` | `(C12*C33 - C13^2) / (C11*C33 - C13^2)` | `[img-read: embim425.png]` |
| `PRDYN_HV` | `(EDYN_HOR / EDYN_VER) * PRDYN_VER` | `[img-read: embim426.png]` |

#### 2.2.6 TTI model

`TTI_THETA` = angle between the well-axis vector (from `Well Deviation` / `Well Azimuth`
input curves) and the TI symmetry axis (from `Formation Dip Angle` / `Formation Dip Azimuth`
parameters, **assumed normal to the dip plane**), computed by dot product
(`anisotropic-elastic.htm`).

Shear polarisation branching (parameter `Polarisation of Fast Shear Wave`):

| Setting | VSV | VSH | Source |
|---|---|---|---|
| Horizontal | `1/DTSS` | `1/DTSF` | `[img-read: embim447/448.png]` |
| Vertical | `1/DTSF` | `1/DTSS` | `[img-read: embim449/450.png]` |
| Automatic, `AZI_DTSF` = 90 or 270 ±30° | `1/DTSS` | `1/DTSF` | `[img-read: embim451/452.png]` |
| Automatic, `AZI_DTSF` = 0 or 180 ±30° | `1/DTSF` | `1/DTSS` | `[img-read: embim453/454.png]` |
| Automatic, outside those ranges | *no solution — nulls output* | | `anisotropic-elastic.htm` |

Four elastic moduli then feed a trigonometric solve:

```
mu_qP  = RHOB * VP^2                                    [img-read: embim455.png]
mu_qSV = RHOB * VSV^2                                   [img-read: embim456.png]
mu_SH  = RHOB * VSH^2                                   [img-read: embim457.png]
mu_ST  = RHOF * (VST^2 * VF^2)/(VF^2 - VST^2)           [img-read: embim458.png]
```

With a Stoneley curve the system is solved directly (γ from `TTI_THETA`, then `C44`,
`C66`, `C33`); without Stoneley the same three are solved **iteratively** from
`mu_qP`, `mu_qSV`, `mu_SH` alone (`[img-read: embim469/470/471.png]`, prose in
`anisotropic-elastic.htm`). Remaining stiffnesses:

```
C11 = 2*(C66 - C44) + C33                               [img-read: embim459.png]
C12 = C11 - 2*C66                                       [img-read: embim460.png]
C13 = C12                                               [img-read: embim461.png]
```

The manual does **not** print the trigonometric system itself — only its inputs and
outputs. See §8 O-1.

#### 2.2.7 ANISO_FLAG QC criteria (Helbig & Schoenberg 1987)

`ANISO_FLAG = 1` only if all five hold, plus dynamic and static Poisson's ratios in
`[0.0, 0.5]` inclusive (`anisotropic-elastic.htm`):

1. `C44 > 0` `[img-read: embim478.png]`
2. `C66 > 0` `[img-read: embim479.png]`
3. `C11 - C66 > 0` `[img-read: embim480.png]`
4. `C33 * (C11 - C66) > C13^2` `[img-read: embim481.png]`
5. `C11 * C33 > C13^2` `[img-read: embim482.png]`

Static conversions reuse the isotropic transforms verbatim
(`[img-read: embim473/474/475/476/477.png]` = Lacy sand / Lacy shale / Lacy mixed /
Morales & Marcinew / Generic, matching §2.1.7 coefficient-for-coefficient).

---

### 2.3 Rock Strength Anisotropy — Planes of Weakness (`rock_strength_anisotropy.htm`)

- Failure on the weakness plane uses a **linear Mohr-Coulomb** criterion defined by
  Cohesion + Friction Angle.
- Cohesion is entered either directly as a pressure or derived via a
  `PoW UCS Reduction Factor` in [0, 1].
- Plane orientation from `PoW Dip Azimuth` and `PoW Dip Angle` (both **True**).
- Outputs `SF_MP_0_RSA`, `SF_EMW_0_RSA` (anisotropic path) and `SF_MP_0_COMBI`,
  `SF_EMW_0_COMBI` (combined with the isotropic result). The anisotropic branch runs
  **in parallel with**, not instead of, the isotropic one (`wellbore_stability2.htm`,
  flowchart `rmnew_clip0198.png`).

---

### 2.4 Wellbore Stability (`wellbore_stability.htm`)

Three shear-failure criteria only — **Mohr-Coulomb, Modified Lade, Hoek-Brown**.
There is **no Mogi-Coulomb** in IP 2025 (a named gap versus the special tasking).
Common reference cited: Zoback, *Reservoir Geomechanics*, Cambridge 2007
(ISBN 978-0-521-14619-7).

Solution method: **Mohr-Coulomb and Hoek-Brown are solved iteratively; Modified Lade has
an analytical solution and uses it.**

Stress preparation (`wellbore_stability.htm`): far-field Sv/SHmax/Shmin → local normal +
shear stresses at the wellbore wall → effective stresses → principal stresses σ1, σ2, σ3.

| Item | Equation | Source |
|---|---|---|
| Effective stress | `σ' = σt - α*P` | `[img-read: rmnew_clip0135.png]` |
| Mohr-Coulomb surface | `σ1 = C0 + q*σ3`, `q = tan²(π/4 + φ/2)` | `[img-read: rmnew_clip0136.png]` |
| MC failure test | `σ1 > C0 + q*σ3` | `[img-read: rmnew_clip0137.png]` |
| Modified Lade | `(I1')³/I3' = 27 + η` where `I1' = (σ1+S)+(σ2+S)+(σ3+S)`, `I3' = (σ1+S)(σ2+S)(σ3+S)`, `S = S0/tan φ`, `S0 = C0/(2√q)`, `q = tan²(π/4 + φ/2)`, `η = 4(tan φ)²(9 - 7 sin φ)/(1 - sin φ)` | `[img-read: rmnew_clip0138.png]` |
| Hoek-Brown | `σ1 = σ3 + C0*(mb*σ3/C0 + s)^a` | `[img-read: rmnew_clip0139.png]` |
| Hoek-Brown alternate parameterisation | `mb = mi*e^((GSI-100)/(28-14D))`; `s = e^((GSI-100)/(9-3D))`; `a = 1/2 + (1/6)*(e^(-GSI/15) - e^(-20/3))` | `[img-read: rmnew_clip0140.png]` |

MC is explicitly stated to be independent of σ2; Modified Lade depends on all three.

**Hydraulic fracture outputs:** `HFP = S_TT_EFF_MIN + MWactual + T0`; `HFG = HFP / TVD`
(`wellbore_stability.htm`).

**Stress polygon / breakout inversion** (analytical forms, **vertical wells only** —
IP warns if deviation > 30°):

| Line | Equation | Source |
|---|---|---|
| Shear failure (breakout, Zoback) | `σHmax = [UCS + Pm + αPp - σhmin(1 + 2cos2θb)] / (1 - 2cos2θb)` | `[img-read: rmnew_clip0180.png]` |
| Shear failure (breakout, MC frictional) | `σHmax = [UCS + Pm + αPp - σhmin(1 + 2cos2θb) + (Pm - αPp)tan²β] / (1 - 2cos2θb)` | `[img-read: rmnew_clip0181.png]` |
| Tensile failure | `σHmax = 3σhmin - Pm - αPp + T0` | `[img-read: rmnew_clip0182.png]` |

with `2θb = 180° - Breakout angle` and `β = 45° - Φ/2` (`wellbore_stability.htm`).
Iterative variants of the same two lines exist (given a mud weight and a target breakout
angle, solve SHmax for each Shmin).

---

### 2.5 Vertical Stress / overburden (`vertical_stress.htm`)

| Model | Equation | Source |
|---|---|---|
| Amoco Avg Sediment Density | `OBG = [(8.5 * W) + (ρavg * (D - W - A))] / D` | `[img-read: denew_clip0014.png]` |
| Amoco Compaction | `ρavg = 16.3 + [(D - W - A)/3125]^0.6` | `[img-read: denew_clip0015.png]` |
| Barker & Wood | `Cum.Av.Formation Bulk Density (lb/gal) = 5.3 * (TVDBML)^0.1356` | `[img-read: denew_clip0016.png]` |

Legend (`vertical_stress.htm`): OBG lb/gal; `W` water depth ft; `8.5` = assumed sea-water
density lb/gal; `D` = TVD below KB ft; `A` = air gap ft. The `0.6` exponent is stated to
be empirical, and the compaction relationship is declared suitable **only for sand/shale
sequences that have not experienced unloading/uplift**.

Three bundled overburden lookup tables: Offshore Texas/Louisiana (Unocal, March 2003);
Average GOM (Amoco, from Fig. 2 of Eaton & Eaton 1997); Deepwater GOM (Amoco, from
Barker & Wood). Registered in `OBG_Files.obg`, stored as space-delimited `.obg` text
(`porepressurecalculations2.htm`).

---

### 2.6 Overburden Tools (`overburden_tools.htm`) — NEW in IP 2025

#### 2.6.1 SMECTITE / ILLITE (priority capture)

**Alberty/McLean acoustic→density** (OTC 15290, 2003; SPE 108787, 2005), `Dt` in µs/ft:

```
RHOB_smectite = 2.918 - 0.00517 * Dt
RHOB_illite   = 3.044 - 0.00505 * Dt        [img-read: ppfg_035.png]
```

Calibrated to **Miocene and Pliocene smectite- and illite-dominated shale in the GoM,
North Sea, Nile Delta and Caspian Sea** (`overburden_tools.htm`, `acoustic_to_pressure.htm`).

**Katahara kinetic smectite→illite transition** (personal communication 1 May 2013,
`Smectite-Illite_ArrheniusIntegral.xlm`):

```
Frac_sm = 0.5 * (1 - tanh( 3*(Tf - 0.5*(Tbeg + Tend)) / (Tend - Tbeg) ))    [img-read: ppfg_036.png]
```

`Tf` formation temperature °F; `Tbeg` default **160 °F**, `Tend` default **220 °F**
(each allowed 0–500). Physical basis stated in the manual: the conversion is driven by
potassium released by diagenetic breakdown of K-feldspar, which normally begins at 160 °F
and completes at 220 °F. **Stated failure mode: if the formation is devoid of K-feldspar
the conversion is potassium-starved and the model overestimates the illite fraction**
(`overburden_tools.htm`).

The manual also notes the Alberty (2005) and Katahara (2013) acoustic-to-density
relationships are very close, and treats the spread between them as a proxy for
uncertainty. The Katahara smectite/illite acoustic-to-density relationship is
**referenced but not implemented** in the PPFG Toolkit.

Downstream use: the shale-salinity module falls back to a pseudo-density from acoustic
via the Alberty smectite/illite path when density porosity is missing, and its default
matrix density is **RHOma = 2.59 g/cc**, described in the manual as a reasonable *dry*
matrix density for **smectite and kaolinite** (`resistivity_to_pressure.htm`).

#### 2.6.2 Overburden RHOB models

| Model | Equation | Source |
|---|---|---|
| Alberty NSC | `RHOB = RHO_ma + (RHO0_ma - RHO) * Exp(-SIGMAMEAN*(OC/6000000))` — **subscripts ambiguous, see O-2** | `[img-read: ppfg_021.png]` |
| Miller porosity | `Porosity = porA + porB * Exp(-Kdecl * P(Dbml)^(1/Pcur))` | `[img-read: ppfg_024.png]` |
| Miller density | `RHOB = RHOMA*(1 - Porosity) + Porosity*RHOW` | `[img-read: ppfg_024.png]` |
| Traugott Power Law | `RHOB = RHOMA + (RHO0 - RHOMA) * Exp(-SIGMAMEAN/Cpg)` | `[img-read: ppfg_027.png]` |
| Traugott K0 | `K0 = 0.039 * (TVD - (0.25*WTRDEPTH - DFElev))^0.33` | `[img-read: ppfg_028.png]` |
| Traugott World Oil | `[img-read: ppfg_030.png]` (average-density correlation vs depth) | `overburden_tools.htm` |
| Katahara avg→actual | `RHOB = 16.3 + 1.6 * ((DBML/3125)^0.6)/8.345` — **parenthesisation ambiguous, see O-3** | `[img-read: ppfg_031.png]` |
| Gardner generalized | `RHOB = 7.27 * Dt^(-0.25)` (≡ power law with A = 0.23, B = 0.25) | `[img-read: ppfg_040.png]` |

**2D Overburden Builder assumptions**, stated explicitly: no sea-floor erosion; no
erosional unconformities along the well path; a single salt body with no inclusions;
uniform average salt density; important density variations roughly parallel to the seabed
(`overburden_tools.htm`).

---

### 2.7 Multi-Well Pore Pressure (`multi_well_pore_pressure.htm`) — NEW in IP 2025

#### 2.7.1 Eaton — exact exponents by input type

| Variant | Equation | Exponent | Source |
|---|---|---|---|
| Resistivity | `P/D = S/D - (S/D - P/Dn) * (Rsh_obs / Rsh_norm)^1.2` | **1.2** | `[img-read: embim483.png]` |
| Sonic | `P/D = S/D - (S/D - P/Dn) * (ΔTsh_norm / ΔTsh_obs)^3.0` | **3.0** | `[img-read: embim484.png]` |
| Velocity | `P/D = S/D - (S/D - P/Dn) * (Vsh_obs / Vsh_norm)^3.0` | **3.0** | `[img-read: embim485.png]` |
| D-exponent | `P/D = S/D - (S/D - P/Dn) * (Dxc_obs / Dxc_norm)^1.2` | **1.2** | `[img-read: embim486.png]` |

**The sonic ratio is inverted relative to the other three** (normal over observed).
This is correct — Dt is inversely related to velocity — but it is the single easiest
sign error to make when reimplementing.

#### 2.7.2 Bowers

| Item | Equation | Source |
|---|---|---|
| NCT, sonic | `NCT = 1e6 / (1e6/DT_mudline + A*(OBPress - Phydrostatic)^B)` | `[img-read: ppnew_clip0015.png]` |
| NCT, velocity | `NCT = V_mudline + A*(OBPress - Phydrostatic)^B` | `[img-read: ppnew_clip0016.png]` |
| PP sonic, loading | `OBPress - [((1e6/DT) - (1e6/DT_mudline))/A]^(1/B)` | `[img-read: ppnew_clip0017.png]` |
| PP sonic, unloading | `OBPress - [ ((1e6/DT_min - 1e6/DT_ml)/A)^((1-U)/B) * ((1e6/DT - 1e6/DT_ml)/A)^(U/B) ]` | `[img-read: ppnew_clip0019.png]` |
| PP velocity, loading | `OBPress - [(V - V_mudline)/A]^(1/B)` | `[img-read: ppnew_clip0018.png]` |
| PP velocity, unloading | `OBPress - [ ((Vmax - Vml)/A)^((1-U)/B) * ((V - Vml)/A)^(U/B) ]` | `[img-read: ppnew_clip0020.png]` |
| Effective stress | `σe = Vertical Stress - Phydrostatic` | `[img-read: ppnew_clip0029.png]` |
| Loading / virgin curve | `V = Vml + A*σe^B` | `[img-read: ppnew_clip0030.png]` |
| Unloading curve | `V = Vml + A*[σe,max * (σe/σe,max)^(1/U)]^B` | `[img-read: ppnew_clip0031.png]` |

I verified algebraically that `ppnew_clip0031` (crossplot form) and `ppnew_clip0020`
(pressure form) are mutually consistent — no discrepancy in the Bowers chain.

Bell-taper smoothing filter: `Wt_j = (1 - cos(2πj/(FilterLength+1))) / (FilterLength+1)`
`[img-read: ppnew_clip0022.png]`. Filter length must be **odd, 3–2001**
(`porepressurecalculations2.htm`).

---

### 2.8 Resistivity to Pressure (`resistivity_to_pressure.htm`) — NEW in IP 2025

| Item | Equation | Source |
|---|---|---|
| Klein (1991) anisotropy correction | `RtCor = Rt_in * ( (sin²(RelAng)/RvRh) + cos²(RelAng) )^0.5` | `[img-read: ppfg_054.png]` |
| Arps (1953) temperature correction | `Rt_corr = Rt * (Temp_form + 6.77)/(Temp_target + 6.77)` | `[img-read: ppfg_057.png]` |
| Eaton on resistivity | `VES = Nves * (Rt_shale / R_nct)^EatonExp` | `[img-read: ppfg_061.png]` |
| Athy-like porosity (Archie NCT) | `Phi = Phi0 * exp(-Nves/OC)` | `resistivity_to_pressure.htm` (ASCII) |
| Archie NCT (Sw = 100 %) | `Rnct = Rw100 * a / Phi^m` | `resistivity_to_pressure.htm` (ASCII) |
| Density porosity (shale salinity) | `Porosity = (RHO_ma - RHOB)/(RHO_ma - RHO_fl)` | `[img-read: ppfg_067.png]` |
| Dusenbery acoustic porosity | `Porosity = 0.0048*DT - 0.234` | `[img-read: ppfg_068.png]` |

Pore pressure = overburden − VES throughout (`resistivity_to_pressure.htm`).
The Rt shale picker uses an HP (pentadiagonal linear-inversion) smoother; the manual
credits Kurt Annen's pentadiagonal solver, reworked to 1-D arrays because the IP API does
not permit 2-D matrices.

---

### 2.9 Acoustic to Pressure (`acoustic_to_pressure.htm`) — NEW in IP 2025

Five velocity→effective-stress NCT relationships, each then run through Eaton.

| Model | Equation | Source |
|---|---|---|
| Alberty/McLean smectite/illite | `Vp = A + B * e^x`, `x = C * ES^D` | `[img-read: ppfg_077.png]` |
| Bowers | `Vp = MLV + A * VES^B` | `[img-read: ppfg_080.png]` |
| Chapman | `DT_NCT = DTma + (DTml - DTma) * exp(-TVDbml / c)` | `[img-read: ppfg_082.png]` |
| Eberhart-Phillips, original (kbar, km/s) | `Vp = 5.77 - 6.94*Phi - 1.73*Vcl^0.5 + 0.446*(ES - e^(-16.7*ES))` | `[img-read: ppfg_084.png]` |
| Eberhart-Phillips, IP-modified (psi, ft/s) | `Vp = Vma - B*Phi - 5676*Vcl^0.5 + 0.101*(ES - 1460*e^(-ES/868))` | `[img-read: ppfg_085.png]` |
| Traugott modified-Athy porosity | `Phi = Phi0 * e^(-ES/C)` | `[img-read: ppfg_086.png]` |
| Miller | `Vp = Vml + (Vma - Vml)*(1 - e^(-lambda*(Ovbd - Hydro)))` | `[img-read: ppfg_088.png]` |

Manual-stated unit conversions for Eberhart-Phillips: 5.77 → 18931; 6.94 → 22770;
1.73 → 5676; 0.446 → 0.101; 16.7 → 1/868. **The `1460` inside the bracket does not
reconcile with those conversions — see §5 D14.**

Alberty A/B/C/D constants for smectite-rich and illite-rich shale are **not printed** in
the manual — see §8 O-4. This is the most consequential gap in the whole PPFG section.

---

### 2.10 Dxc to Pressure (`dxc_to_pressure.htm`) — NEW in IP 2025

Basis: Jorden & Shirley (1966) drilling exponent, with the Rehm & McClendon (1971) mud
weight correction.

```
Dxc = Log10(rop/(60*rpm)) / (Log10((12*wob)/(1000000*bs))) * hyd/mudwt   [img-read: ppfg_091.png]
```

Legend (`dxc_to_pressure.htm`): `rop` ft/hr; `rpm` rev/min; `wob` **pounds**; `bs`
**inches**; `hyd` formation-fluid hydrostatic pressure; `mudwt` mud weight in the same
units as `hyd`.

| NCT method | Equation | Source |
|---|---|---|
| Linear NCT + Eaton | `VES = Nves * (Dxc_SHALE / Dxc_NCT)^Eaton_exp` | `[img-read: ppfg_097.png]` |
| Power-law NCT (Gyllenhammar, 2000 pers. comm.) | `Dxc_NCT = Dxc_Incpt * 10^(TVDbml * Dxc_Grad / 10000)` | `[img-read: ppfg_101.png]` |
| Power-law NCT + Eaton | `VES = Nves * (Dxc_SHALE / Dxc_NCT)^Eaton_exp` | `[img-read: ppfg_102.png]` |

The manual states the power-law method is **unpublished** and has been used successfully
in unconsolidated shale with a single zone; the linear method typically needs several
zones to fit calibration points.

---

### 2.11 Horizontal Stress (`rock_stress.htm`)

#### Minimum horizontal stress — 6 models

| Model | Equation | Source |
|---|---|---|
| Eaton | `SHMING_EAT = (Svg - Pg)*(u/(1-u)) + Pg` [psi/ft] | `rock_stress.htm` |
| Matthews & Kelly | `SHMING_MK = Ki*(Sg - Pg) + Pg` | `rock_stress.htm` |
| Anderson et al. 1973 (SPE-4135-PA) | `SHMING_AND = (Svg - αPg)*(u/(1-u)) + αPg` | `rock_stress.htm` |
| Daines 1982 (SPE-9254-PA) | `SHMING_DAI = Stg + (Svg - Pg)*(ν/(1-ν)) + Pg` | `rock_stress.htm` |
| Breckels & Van Eekelen 1982, `Z ≤ 11,500 ft` | `σh = 0.197*Z^1.145 + 0.46*(pp - Ppn)` | `rock_stress.htm` |
| Breckels & Van Eekelen 1982, `Z > 11,500 ft` | `σh = 1.167*Z - 4596 + 0.46*(pp - Ppn)` | `rock_stress.htm` |
| Elastic (poro-elastic) | `ν/(1-ν)*(σv - αPp) + αPp` | `[img-read: rmnew_clip0165.png]` |
| Tectonic term | `νE/(1-ν²)*dεH + E/(1-ν²)*dεh` | `[img-read: rmnew_clip0166.png]` |
| Tectonic Strain, full ShMin | `Shmin = ν/(1-ν)(σv - αPp) + αPp + νE/(1-ν²)*dεH + E/(1-ν²)*dεh` | `[img-read: rmnew_clip0167.png]` |

Breckels & Van Eekelen: σh in **psi**, Z in **ft TVDSS**; `Ppn` = normal pore pressure,
default **0.45 psi/ft**.

#### Maximum horizontal stress — 3 models

| Model | Equation | Source |
|---|---|---|
| ShMin (linear upscale) | `SHMAXP = SHMINP * Factor` | `rock_stress.htm` |
| Sv–ShMin | `SHMAXP = SHMINP + k*(Sv - SHMINP)` | `rock_stress.htm` |
| Tectonic Strain | `S_H = ν/(1-ν)(SV - αPP) + αPP + E*εx/(1-ν²) + νE*εy/(1-ν²)` | `[img-read: rmnew_clip0168.png]` |

Further references cited on the page: Anderson 1951, *The Dynamics of Faulting and Dyke
Formation* (Oliver & Boyd); Addis, Last & Yassir, March 1996, SPE-28140-PA.

---

### 2.12 Single-well Pore & Fracture Pressure (`porepressurecalculations2.htm`)

Five fracture-gradient models: **Eaton, Matthews & Kelly, Modified Eaton, Barker & Wood,
Daines**.

| Model | Equation | Source |
|---|---|---|
| Eaton FG | `F/d = (S/d - P/d)*(μ/(1-μ)) + P/d` | `[img-read: clip1003.png]` |
| Matthews & Kelly | `F = Ki * σ + P` where `σ = S - P` | `[img-read: clip1009.png]` |
| Modified Eaton, composite vertical stress | `σ_vc = (0.442 * WD) + (OBGrad * D_sed)` | `[img-read: embim503.png]` |
| Modified Eaton, matrix stress ratio | `Ke = 0.05329427 * 0.99996^Deff * Deff^0.3006479` | `[img-read: embim505.png]` |
| Modified Eaton, effective depth | `Deff = (Water Depth / 2) + Dsed` | `porepressurecalculations2.htm` (Eq. 3) |
| Modified Eaton, fracture pressure | `FP = PP + Ke*[σ_vc - PP]` | `[img-read: embim506.png]` |
| Daines FG | `FG = σt + σ1' * (μ/(1-μ)) + P` | `[img-read: clip1013.png]` |
| Daines stress ratio | `β = σt / σ1'` | `[img-read: clip1019.png]` |

**Eaton Gulf Coast Poisson's Ratio** (Eaton & Eaton 1997), Depth = ft below mudline:

```
0–4999 ft:  μ = 0.2007142857 - 7.5e-9 * Depth^2 + 8.0214286e-5 * Depth    [img-read: clip1005.png]
>5000 ft:   μ = 0.3724340861 - 1.77258e-10 * Depth^2 + 9.4748424e-6 * Depth [img-read: clip1006.png]
```

**Eaton Deep Water GoM Poisson's Ratio:**

```
0–4999 ft:  μ = 0.3124642857 - 6.089286e-9 * Depth^2 + 5.7875e-5 * Depth   [img-read: clip1007.png]
>5000 ft:   μ = 0.4260341387 - 1.882e-10 * Depth^2 + 7.2947129e-6 * Depth  [img-read: clip1008.png]
```

I verified these transcriptions by testing continuity at the 5,000 ft breakpoint:
Gulf Coast 0.41429 vs 0.41538; Deep Water 0.44961 vs 0.45780 — near-continuous, which
would not survive a mistyped coefficient.

Presets live in `Fract_Grad_Coeff.par` (Poisson's ratio presets, Matthews & Kelly `Ki`
presets) and `Poisson_Ratio_Lithologies.par` (Daines per-lithology Poisson's ratios,
corresponding to the table in Daines' original paper), both in the IP install directory.

**Barker & Wood limits:** derived from >50 LOTs in 20 GoM wells, water depths 2,000–7,000 ft,
valid to roughly **8,000 ft BML**, assumes **8.55 lb/gal** water density; the model sets
fracture gradient = overburden gradient in the shallow section.

**Explicit caution reproduced by the vendor:** the manual cites Mouchet et al. (1989) as
critical of *all* fracture-gradient methodologies — over-simplified geologic/tectonic
models on one side, extra unknowns (e.g. the Daines tectonic component) on the other —
and tells the user to weigh trajectory, tested formation and local/regional in-situ
stress knowledge.

---

### 2.13 Fracture Gradient — PPFG Toolkit (`fracture_gradient.htm`) — NEW in IP 2025

```
FG = Kf * (Ovbd - PP) + PP + Tectonic + Tensile        [img-read: ppfg_105.png]
```

All terms in common pressure units; **psi in this implementation**. `Kf` is defined as
`(FP - PP)/(Sv - PP)`.

Seven relationships offered: Fixed Kf (Matthews 1967), Eaton World Oil (Eaton 1997),
Matthews & Kelly LA GC (1967), Matthews & Kelly S. Tx (1967), GeoFluids (Casey 2015),
Hess Malaysia Low Side (2016), Hess Malaysia High Side (2016). Their internal Kf
coefficients are **not printed** — see §8 O-5.

**GoM salt guidance printed in the manual:** for salt, use Fixed Kf with `Kf = 1.0` and
tensile strength **0 psi low / 1000 psi most-likely / 1200 psi high**; for inclusions
within salt, `Kf = 1.0` with **0 tensile for all cases**. Zone the salt intervals separately.

The manual also warns that tectonic stress is normally *measured* in the SHmax direction
while fracture gradient is normally oriented along Shmin, so only the reoriented
component should be entered — and defers to a geomechanics SME for the amount.

---

### 2.14 Poro- & Thermo-Elastic Stress (`poro-_and_thermo-elastic_stres.htm`) — NEW in IP 2025

| Model | Equation | Source |
|---|---|---|
| Thermo-elastic | `Δσ_T = E*β*ΔT / (1 - ν)` | `[img-read: denew_clip0022.png]` |
| Poro-elastic, Stress Path Factor | `Δσ_P = SPF * ΔP` | `[img-read: denew_clip0023.png]` |
| Poro-elastic, Linear Elastic Equation | `Δσ_P = ((1 - 2ν)/(1 - ν)) * α * ΔP` | `[img-read: denew_clip0024.png]` |

`ΔT = T_target - T_initial`; `ΔP = P_target - P_initial`. `E` static Young's modulus,
`β` linear thermal expansion coefficient, `ν` static Poisson's ratio, `α` static Biot alpha.
**Poro-elastic affects the two horizontal stresses only; thermo-elastic affects horizontal
*and* vertical stress.** `Biot Alpha Factor`, `Poisson's Ratio Model`, `Poisson's Ratio`
and `Young's Modulus` are shared state with the Horizontal Stress module — changing them
in one changes both.

---

### 2.15 Fractures and Fault Stress (`fractures_and_fault_stress.htm`) — NEW in IP 2025

In-situ stresses are projected into a North-East-Z(down) frame and resolved onto each
picked fracture plane (true dip and true azimuth from an Image Analysis pick set), then
tested against a linear Mohr-Coulomb line:

```
Shear Stress = Cohesion + Normal Effective Stress * tan(Friction Angle)   [img-read: rmnew_clip0190.png]
StressRatio_Fracture = TAUP_Fracture / SNP_EFF_Fracture                   [img-read: rmnew_clip0202.png]
ShearRatio_Fracture  = Shear_Stress / Shear_Stress_At_Coulomb_Failure     [img-read: rmnew_clip0203.png]
```

Failure when the projected shear stress ≥ the failure-line shear stress at that normal
effective stress. Outputs are **sparse** curves — values only at pick depths.
`SNN_`, `TAUN_`, `PPCN_`, `DPCN_` variants are the same quantities normalised to vertical
stress.

---

## 3. Parameters, defaults & constraints

### 3.1 Rock strength — validity ranges as the manual states them

| Correlation | Stated calibration / validity |
|---|---|
| Sarda | φ > 30 %; Germiny-sous-Coulombs, France; max φ 35 % |
| Formel | φ 20–35 %, DT 90–140 µs/ft, low confining stress 0–5 MPa, 200 samples, Norwegian North Sea |
| Vernik | φ 0.2–33 % |
| Modified Vernik | shaly sandstone, φ < 30 % |
| Chang GOM | Pliocene-and-younger Gulf of Mexico |
| Lashkaripour & Dusseault | 13 data points, mean UCS ≈ 79 MPa, nine points ≤ 10 % φ |
| Horsrud | high-porosity North Sea Tertiary shales |
| Chang Limestone 1 | 0.05 < φ < 0.2, 30 < UCS < 150 MPa, Middle East |
| Chang Limestone 2 | 0 < φ < 0.2, 10 < UCS < 300 MPa |
| Chang Dolomite | UCS 8,700–14,500 psi |

### 3.2 Rock strength — numeric defaults

| Parameter | Default | Notes | Source |
|---|---|---|---|
| `T0 Coeff A` | 0.1 | tensile-strength `A*UCS^B` | `rock_strength.htm` |
| `T0 Coeff B` | 1 | | `rock_strength.htm` |
| TWC-UCS Upscaler A | 80.884 | vendor calls these "globally applicable values for sandstones" | `rock_strength.htm` |
| TWC-UCS Upscaler B | 0.57 | | `rock_strength.htm` |
| `Poisson Factor` | 1 | i.e. PRSTA = PRDYN by default | `rock_strength.htm`, `anisotropic-elastic.htm` |
| `Generic Youngs Coefficient A` | 1 | | both pages |
| `Generic Youngs Coefficient B` | 1 | | both pages |
| `Generic Youngs Coefficient C` | 0 | **new in IP 2025**, see §6 | both pages |

### 3.3 Wellbore stability — QC ranges and parameters

Input QC ranges (standard / extreme) (`wellbore_stability.htm`):

| Input | Standard | Extreme |
|---|---|---|
| Pore Pressure | 2–16 ppg | 0–24 ppg |
| Min horizontal stress | 4–28 ppg | 2–40 ppg |
| Max horizontal stress | 4–28 ppg | 4–40 ppg |
| Vertical stress | 10–24 ppg | 8–28 ppg |
| UCS | 100–10,000 psi | 0–40,000 psi |
| Friction angle | 10–45 deg | 5–60 deg |
| Tensile strength | 0 – UCS/10 psi | 0 – UCS/5 psi |
| Biot | 0.4–1 | 0–1 |
| Poisson's Ratio | 0.1–0.5 | 0–0.5 |

Other parameters: `Allow Stress Substitution` (result flag **Blue** = σ2 was substituted
for σ3); `Perform Damage Angle Calculation` — **off by default** because it is slow;
`Allowable Breakout Model` ∈ {`60 Deg`, `90 Deg – Deviation`, `User Param`};
`UCS Calibration Multiplier`; `Poisson's Ratio Model` presets (`Gulf Coast`, `Deep Water`,
…) read from `Fract_Grad_Coeff.par`; `Hoek Brown Parameter Model` ∈ {`Default` (mb, a, s),
`Alternate` (GSI, D, mi)}.

**Fault Friction Coefficient** default **0.6** (= 31° fault friction angle); the manual
cites Zoback et al. 2003 for a crustal range of **0.6–1.0**. Stress-polygon analytical
lines are valid for **vertical wells only**; IP warns above 30° deviation.

Hoek-Brown numeric defaults are described as "based on an analysis for weak rock" and
explicitly *not* representative of all rock types, but **the numbers themselves are not
printed** — see §8 O-6. The manual points users to `rocscience.com/education/hoeks_corner`.

### 3.4 Overburden RHOB model defaults

**Alberty NSC** (`overburden_tools.htm`):

| Parameter | Default | Range | Provenance given |
|---|---|---|---|
| Sea water density | 1.038 g/cc | 0.96–1.3 | average at Mars and Ursa, GoM, from seafloor pressure + LWD quartz gauges |
| Formation water density | 1.073 g/cc | 0.96–1.5 | ≈ 140,000 ppm NaCl at typical P/T |
| Mudline porosity PHI0 | 0.4 (v/v total) | 0.1–0.8 | |
| Matrix density | 2.65 g/cc | 2.2–3.6 | |
| Overburden calibration constant OC | 1000 | 500–5000 | controls porosity decline rate vs mean effective stress |
| Lump layer top / bottom / density | 0 / 0 / 0 | | |

**Miller** (`overburden_tools.htm`):

| Parameter | Default | Range | Notes |
|---|---|---|---|
| Matrix density | 2.68 g/cc | 2.2–3.3 | |
| Water density | 1.03 g/cc | 0.96–1.5 | ≈ 30,000 ppm NaCl |
| porA | 0.35 | 0.01–0.6 | porosity at "essentially infinite" depth |
| porB | **0.30** | 0.2–0.8 | **Miller's original default was 0.35**; changed after comparison with Ursa shallow density and IODP shallow cores in S. Mississippi Canyon |
| Kdecl | 0.0035 | 0.001–0.01 | porosity decline factor |
| Curvature | 1.09 | 0.1–100 | |

Miller is stated to be intended for the first 1,000–2,000 ft BML and not beyond ~2,000 ft BML.

**Traugott Power Law** (`overburden_tools.htm`): sea water 1.038 g/cc (0.96–1.3);
formation water 1.073 g/cc (0.96–1.5); PHI0 0.4 (0.1–0.8); RHOMA 2.65 g/cc (2.2–3.6);
Cpg 1000 (100–10,000); lump layer 0/0/0.

**Default overburden gradient:** 1 psi/ft (19.25 lb/gal). The manual says this "may give
acceptable results for onshore wells" and explicitly does **not** recommend it for deep
water (`vertical_stress.htm`).

**Gardner:** the manual states the default constants were fit to a *group* of lithologies
(sandstone, shale, limestone, dolomite), that using the defaults is **not recommended as
general practice**, and that only the power-law form has adjustable constants
(A = 0.23, B = 0.25 reproduce the generalized solution). The polynomial form runs ~0.03 g/cc
denser (`overburden_tools.htm`).

### 3.5 Pore-pressure method defaults

| Parameter | Default | Range | Source |
|---|---|---|---|
| Eaton exponent, resistivity | **1.2** | 0.9–2.00 | `multi_well_pore_pressure.htm` |
| Eaton exponent, sonic | **3.0** | 2.0–4.0 | `multi_well_pore_pressure.htm` |
| Eaton exponent, velocity | **3.0** | 2.0–4.0 | `multi_well_pore_pressure.htm` |
| Eaton exponent, D-exponent | **1.2** | 0.9–2.00 | `multi_well_pore_pressure.htm` |
| Eaton exponent (PPFG Toolbox acoustic modules) | **3** | 1–6 | `acoustic_to_pressure.htm` |
| Eaton exponent (PPFG Toolbox Rt / Dxc modules) | **1.2** | 0.1–5 | `resistivity_to_pressure.htm`, `dxc_to_pressure.htm` |
| Resistivity temperature constant (Arps) | **6.77** | — | Amoco alternative **−6** also noted |
| Normal hydrostatic gradient | **1.0 g/cc (fresh water)** | — | `multi_well_pore_pressure.htm` |
| Rt semi-log NCT top / base | 1.0 / 2.0 ohm-m | 0.02–2000 | `resistivity_to_pressure.htm` |
| Archie NCT: mudline shale porosity PHI0 | 0.4 | 0–1 | `resistivity_to_pressure.htm` |
| Archie NCT: Rw100 | **0.056 ohm-m** (≈ 100,000 ppm NaCl) | 0.00005–2 | `resistivity_to_pressure.htm` |
| Archie NCT: overburden calibration const | 1000 | 1–10,000 | `resistivity_to_pressure.htm` |
| Archie NCT: cementation exponent m | **1.87** | 1–5 | `resistivity_to_pressure.htm` |
| Archie NCT: tortuosity a | **0.81** | 0.1–3 | `resistivity_to_pressure.htm` |
| Shale-salinity RHOma | **2.59 g/cc** (smectite + kaolinite dry matrix) | — | `resistivity_to_pressure.htm` |
| Alberty/McLean Tbeg | 160 °F | 0–500 | `acoustic_to_pressure.htm` |
| Alberty/McLean Tend | 220 °F | 0–500 | `acoustic_to_pressure.htm` |
| Bowers mudline velocity | 5000 ft/s | 2000–7000 | `acoustic_to_pressure.htm` |
| Bowers A (gain) | **14.3** (published GoM value) | 2–100 | `acoustic_to_pressure.htm` |
| Bowers B (exponent) | **0.724** (published GoM value) | 0.2–2 | `acoustic_to_pressure.htm` |
| Chapman DTml | 195 µs/ft | 100–300 | `acoustic_to_pressure.htm` |
| Chapman DTma | 59 µs/ft | 40–100 | `acoustic_to_pressure.htm` |
| Chapman exponent c | **5480** | 1000–20,000 | defaults credited to Katahara's GoM notes |
| Eberhart-Phillips K0 | 0.724 | — | `acoustic_to_pressure.htm` |
| Eberhart-Phillips PHI0 | 0.40 | 0.20–0.85 | `acoustic_to_pressure.htm` |
| Eberhart-Phillips VCL | 0.40 | 0.20–0.9 | `acoustic_to_pressure.htm` |
| Eberhart-Phillips C_ | 5200 | 1–10,000 | `acoustic_to_pressure.htm` |
| Eberhart-Phillips B_ | 22,770 | 10,000–30,000 | `acoustic_to_pressure.htm` |
| Eberhart-Phillips VMA | 18,931 ft/s | 4800–30,000 | `acoustic_to_pressure.htm` |
| Miller mudline velocity | 5000 ft/s | 180–7000 | `acoustic_to_pressure.htm` |
| Miller matrix velocity (at infinite ES) | 14,300 | 40–20,000 | `acoustic_to_pressure.htm` |
| Miller lambda | 0.00025 | 0.0001–0.0005 | `acoustic_to_pressure.htm` |
| Dxc linear NCT, top / base | 1.0 / 1.4 | 0.01–10 | `dxc_to_pressure.htm` |
| Dxc power-law intercept | 0.65 | 0.01–10 | `dxc_to_pressure.htm` |
| Dxc power-law gradient | 1.7 | 0.01–10 | `dxc_to_pressure.htm` |
| Shale-picker reduction window | — | **max 100 ft** | all three picker modules |
| Shale-picker bias | — | max ±2 standard deviations; negative is the recommended polarity | all three picker modules |
| Pore-pressure `% Tolerance` | 5 | — | `porepressurecalculations2.htm` |
| Filter length | — | odd, 3–2001 | `porepressurecalculations2.htm` |

### 3.6 Unit-conversion constants printed by IP

| Conversion | Value | Source |
|---|---|---|
| MPa → psi | **145.038** | every UCS correlation raster |
| bar → psi | 14.5038 | `rock_strength.htm` |
| lb/gal → psi/ft | 0.051948 | `porepressurecalculations2.htm` |
| psi/ft → lb/gal | 19.25 | `porepressurecalculations2.htm` |
| g/cc → lb/gal | 8.3454 | `porepressurecalculations2.htm` |
| psi/ft → g/cc | 2.30666 | `porepressurecalculations2.htm` |
| fresh water | 0.434 psi/ft, 8.345 lb/gal | `porepressurecalculations2.htm` |
| saturated brine | 0.519 psi/ft, 9.991 lb/gal | `porepressurecalculations2.htm` |
| IP default water density | 8.5 lb/gal = 0.441 psi/ft = 1.018 g/cc | `porepressurecalculations2.htm` |
| Vp from DT | `304.8/DT` = km/s | UCS correlation rasters |
| Vp from DT (Lal friction angle only) | `304878/DT` = m/s using 3.28 ft/m | `[img-read: rmnew_clip0068.png]` |

### 3.7 Unit discipline per module (special tasking f)

| Module | Pressures | Gradients | Moduli | Slowness |
|---|---|---|---|---|
| Rock strength | psi (UCS, T0, TWC) | — | **Mpsi** (EDYN/ESTA/K/G); compressibility in **microsips** | µs/ft |
| Anisotropic elastic | — | — | stiffnesses from `V²·ρ`, moduli follow isotropic Mpsi convention | µs/ft |
| Wellbore stability | psi (UCS, T0, stresses); **ppg** for QC ranges and EMW outputs | ppg | — | — |
| Horizontal stress | psi (SHMINP/SHMAXP) | psi/ft (SHMING/SHMAXG) | Young's modulus parameter in **Mpsi** | — |
| Poro-/thermo-elastic | internally always **pressure**, gradients converted on input | user-selectable | Young's modulus **Mpsi** if a fixed value; curve units converted internally | — |
| Fracture/fault stress | internally always **pressure** | output separately | — | — |
| PPFG Toolbox (Rt/Dt/Dxc/FG) | **psi** | lb/gal or psi/ft | — | µs/ft or ft/s, auto-detected |
| Vertical stress | psi or MPa | lb/gal or SG | — | — |

Rule that repeats across the newer modules: **if both a pressure and a gradient curve are
supplied, the pressure wins.**

---

## 4. Assumptions & validity limits

1. **UCS correlations are lithology- and basin-specific.** Every one carries a stated
   calibration set (§3.1). The composite `UCS` curve is a splice, so a single well can
   silently mix four different empirical bases across zones.
2. **Static-modulus transforms are dimension-locked to Mpsi.** The Lacy quadratics and the
   Morales & Marcinew power law are not scale-invariant.
3. **Matthews & Kelly assumes a constant 1 psi/ft overburden** and is derived from Gulf
   Coast Texas/Louisiana sandstones — "of little use outside the GOM coast region"
   (`porepressurecalculations2.htm`).
4. **Barker & Wood** is a shallow-BML deep-water GoM method: 2,000–7,000 ft water depth,
   valid to ≈ 8,000 ft BML, assumes FG = OBG in the shallow section.
5. **Modified Eaton (Simmons & Rau 1988)** requires the input overburden gradient curve to
   be referenced to **TVDSS**, and its outputs are TVDSS-referenced — they must be
   re-datumed for a rig-floor/KB deliverable. The manual says so explicitly.
6. **Daines must be calibrated to the first good LOT in compacted formation**, and in a
   zoned well every row of the LOT table must be populated or it will not run.
7. **Stress-polygon analytical breakout equations are vertical-well-only**; IP warns above
   30° deviation but still computes.
8. **TTI model degenerates** when the well axis is within 10° of the TI symmetry axis —
   IP warns and computes anyway.
9. **TTI automatic polarisation has a genuine no-solution region**: if `AZI_DTSF` falls
   outside 0/180 ± 30° and 90/270 ± 30°, nulls are output. There is no fallback.
10. **VTI Vertical Well requires a Stoneley curve** to obtain C66; VTI Horizontal requires
    fast *and* slow shear. Neither is optional.
11. **Miller RHOB is a shallow model** — intended for the first 1,000–2,000 ft BML.
12. **Amoco compaction relationship** is only for sand/shale sequences with no
    unloading/uplift.
13. **Alberty/Katahara smectite-illite kinetics assume K-feldspar is the potassium source**;
    without it the model overestimates illite.
14. **Resistivity pore-pressure limitations printed by the manual:** CEC effects, temperature
    and salinity sensitivity, applicability only to low-TOC shales, and reduced accuracy at
    low porosity / deep burial.
15. **2D Overburden Builder** carries five explicit geological assumptions (§2.6.2) — any
    salt inclusion or erosional unconformity violates it.
16. **Bellotti** is used but its supporting publications are stated to be no longer
    available and its basis unverifiable; **Lindseth** is not calibrated against lab
    measurements but derived from a general impedance-velocity relationship.
17. **Mouchet et al. (1989)** criticism of all fracture-gradient methods is reproduced by
    the vendor as a standing caution.

---

## 5. Internal discrepancies

Reported as found. No value has been "corrected" to match textbook expectation.

| # | Discrepancy | Consequence |
|---|---|---|
| **D1** | `CP_DYN` uniaxial Geertsma uses **`+`** (`rmnew_clip0082`) while `CP_STA` uniaxial Geertsma uses **`×`** (`rmnew_clip0086`) for the same named model. IP2018 used **`×`** for both. | One of the two is wrong in IP 2025. Order-of-magnitude error in pore compressibility. **Highest-severity finding.** |
| **D2** | `ESTA` composite curve is declared "Unit: Mpsi" but the Coates & Denoo, Horsrud Youngs and Chang Dolomite legends say "(psi)". | 10⁶ risk on any UCS model that consumes ESTA. |
| **D3** | Tectonic Strain `Youngs Modulus (Mpsi)` parameter vs equation legend "E = Young's modulus (psi)". | 10⁶ risk on horizontal stress. |
| **D4** | `SHMAXG_TS` gradient defined self-referentially as "SHMAXG_TS / TVDSS" (should be SHMAXP_TS / TVDSS). | Documentation only. |
| **D5** | `SHMING` composite block header says "Unit: psi" while every sub-model equation is psi/ft. | Gradient-vs-pressure confusion. |
| **D6** | Three different sea-water constants coexist in one suite: **0.442 psi/ft** (Modified Eaton), **8.5 lb/gal ≈ 0.441 psi/ft** (IP default), **8.55 lb/gal** (Barker & Wood). | Small but systematic offsets between FG models in the same well. |
| **D7** | Veeken TWC ends in **14.5** while every other strength correlation uses **145.038**. Verified twice and byte-identical to IP2018, so it is deliberate, not a typo — but it means Veeken is bar→psi, not MPa→psi. | Reimplement exactly as printed. |
| **D8** | Friction-angle Lal uses **304878** (m/s at 3.28 ft/m) while all UCS models use **304.8** (km/s exact). | 0.026 % — negligible numerically, but a tell that the two came from different sources. |
| **D9** | Alberty NSC decay is `Exp(-SIGMAMEAN*(OC/6000000))` — OC in the **numerator**, so a larger OC decays faster — whereas Traugott's `Exp(-SIGMAMEAN/Cpg)` puts it in the denominator, yet both parameters carry the identical prose description ("controls the rate at which porosity declines"). | Opposite parameter sensitivity behind identical help text. |
| **D10** | `ppfg_105` is labelled "FG" but computes a **pressure** in psi. | Naming only; the module outputs both. |
| **D11** | The Zoback and MC-frictional breakout equations carry **`+ αPp`** where a consistent effective-stress convention would subtract it. | Reported as printed; flagged for the SandiBumi implementer to derive independently. |
| **D12** | Fresh-water gradient given as **0.434 psi/ft**, but the Breckels & Van Eekelen `Ppn` default is **0.45 psi/ft**. | Different normal-pressure baselines inside one suite. |
| **D13** | ShMin tectonic strain uses symbols `dεH` / `dεh` (`rmnew_clip0167`) while ShMax uses `εx` / `εy` (`rmnew_clip0168`), and `εx` / `εy` are never defined on the page. | Cannot be certain the two equations use the same strain convention. |
| **D14** | **Eberhart-Phillips modified form does not reconcile with its own stated conversions.** The manual lists 0.446 → 0.101 and 16.7 → 1/868. Converting the original exponential term gives 0.446 × 3280.84 = **1463 ft/s**, but the printed modified equation is `0.101*(ES − 1460*e^(−ES/868))`, which distributes to **147.5 ft/s** — a factor of 10 low. The internally consistent reading is `… + 0.101*ES − 1460*e^(−ES/868)` (the 1460 sitting outside the bracket). | A ~1,300 ft/s bias in the NCT at low effective stress. Do not implement the bracketed form without independent derivation. |
| **D15** | VTI Horizontal Well prose labels **both** `embim428` and `embim429` as "DTSS" ("where DTSS is the input Slow Shear Slowness" then "where DTSS is the input Fast Shear Slowness"), while the rasters show `Vss = 1/DTSS` and `Vsf = 1/DTSF`. | Prose typo; the rasters are authoritative. |
| **D16** | Poro-elastic section legends copy-paste the thermo-elastic wording: "Δσp is the **thermo**-elastic stress change" and "ΔP is the change in **temperature**". | Prose typo; equations are unambiguous. |
| **D17** | `PRDYN_HV` is written `= (EDYN_HOR/EDYN_VER) × PRDYN_VER` but the curve produced two lines earlier is named `PRDYN_VERVH`, and the output-curve list calls it `PRDYN_VER_VH`. | Three spellings of one curve; assume they are the same quantity. |

---

## 6. IP2018 numeric diff

Only 6 of my 22 pages have an IP2018 counterpart. I diffed the raw HTML, then compared
every shared equation raster byte-for-byte and read every changed one in both versions.

| Page | 2018 imgs | 2025 imgs | Shared | Shared with changed bytes | Genuinely changed equations |
|---|---:|---:|---:|---:|---|
| `rock_strength` | 76 | 92 | 56 | 14 | **9** (below) |
| `wellbore_stability` | 52 | 67 | 24 | 11 | **0** — all changed images are UI screenshots |
| `porepressurecalculations2` | 60 | 81 | 59 | 2 | **0** — both are navigation chrome |
| `rock_stress` | 28 | 37 | 16 | 11 | **0** — all changed images are UI screenshots |
| `rock_mechanics` | 6 | 14 | 4 | 2 | 0 (workflow diagrams) |
| `specify_rock_mechanics_options` | 4 | 10 | 1 | 1 | 0 (screenshot) |

### 6.1 The one substantive numeric change: dynamic moduli units

```
IP2018:  EC_DYN = 1.34747 × 10^10 * RHOB / DTc^2      [img-read: c18\rmnew_clip0072.png]
IP2025:  EC_DYN = 1.34747 × 10^4  * RHOB / DTc^2      [img-read: rmnew_clip0072.png]
```

Same for `GDYN` (`c18\rmnew_clip0073` vs `rmnew_clip0073`) and `KDYN`
(`c18\rmnew_clip0074` vs `rmnew_clip0074`). **The constant changed by exactly 10⁶ — IP2018
emitted dynamic moduli in psi, IP 2025 emits them in Mpsi.** This retroactively resolves
D2/D3 in favour of the Mpsi reading for the 2025 code path, and it means:

- Any Lacy / Morales & Marcinew coefficient set carried over from an IP2018-era workflow
  is now being fed a number 10⁶ smaller.
- `CB_DYN = 3(1−2ν)/EDYN` consequently changed from 1/psi to **microsips** with no change
  to the printed formula (`rmnew_clip0080` is byte-identical across versions).

### 6.2 Geertsma uniaxial pore compressibility: operator changed

```
IP2018 dynamic:  CPISO × [BIOT × (1 + PRDYN) / (3 × (1 − PRDYN))]   [img-read: c18\rmnew_clip0082.png]
IP2025 dynamic:  CP_DYN_ISO + [BIOT_DYN × (1 + PRDYN) / (3 × (1 − PRDYN))]
IP2018 static:   CPSTA × [BIOT × (1 + PRSTA) / (3 × (1 − PRSTA))]   [img-read: c18\rmnew_clip0086.png]
IP2025 static:   CP_STA_ISO × [BIOT_STA × (1 + PRSTA) / (3 × (1 − PRSTA))]  (unchanged)
```

The **dynamic** branch changed `×` → `+` in IP 2025; the **static** branch did not. This is
the origin of D1 and it is unambiguously a version-to-version edit, not a rendering artefact.

### 6.3 Generic Young's transform gained a constant

```
IP2018:  ESTA = A × EDYN^B          [img-read: c18\rmnew_clip0079.png]
IP2025:  ESTA = A × EDYN^B + C      [img-read: rmnew_clip0079.png]
```

Matches the new `Generic Youngs Coefficient C` parameter (default 0), so IP2018 workflows
reproduce exactly at C = 0.

### 6.4 Renames only (no numeric change)

`CBISO` → `CB_DYN`, `CPISO` → `CP_DYN_ISO`, `CBSTA` → `CB_STA`, `CPSTA` → `CP_STA_ISO`,
`BIOT` → `BIOT_DYN` / `BIOT_STA` (`c18\rmnew_clip0081/0083/0085` vs c25 equivalents).
The new `_DYN` / `_STA` suffixing is what made the whole dynamic/static split explicit.

### 6.5 Unchanged across versions

**All 22 UCS correlations, the Veeken TWC, all four friction-angle models, `PRDYN`,
`EDYN`, `CB_DYN`, `CB_STA`, and all three wellbore-stability failure criteria
(Mohr-Coulomb, Modified Lade, Hoek-Brown) plus the Hoek-Brown alternate parameterisation,
and the rock-stress elastic/tectonic-strain equations** are byte-identical between the
2018 and 2025 rasters. IP 2025's geomechanics additions are almost entirely *new modules*
rather than *revised math* — the exceptions are §6.1–6.3.

New-in-2025 equation rasters on shared pages: `rmnew_clip0169–0176` (KSTA, GSTA, Lacy
shale, Lacy mixed, Raaen dynamic, Raaen static, BIOT_STA, tensile T0) on `rock_strength`;
`rmnew_clip0177–0182` (stress-polygon block) on `wellbore_stability`; `rmnew_clip0168`
(ShMax tectonic strain) on `rock_stress`; `embim494–506` (Modified Eaton chain) on
`porepressurecalculations2`.

---

## 7. SandiBumi notes

1. **Encode `145.038` as a declared MPa→psi conversion, not as a magic literal.** Six of
   the 22 UCS correlations *omit* it (Coates & Denoo, Horsrud Youngs, Golurev & Rabinovich,
   Militzer & Stoll, Rzhewski, Chang Dolomite) because they are natively in psi. A blanket
   "multiply by 145.038" is wrong for exactly those six; a blanket omission is wrong for the
   other sixteen. This must be a per-correlation flag in the model registry.
2. **Make the modulus unit an explicit type, not a convention.** IP shipped a 10⁶ change in
   this constant between releases (§6.1) and still has two pages that disagree with
   themselves about psi vs Mpsi (D2, D3). SandiBumi should carry `Mpsi` in the type, so the
   Lacy quadratics cannot be fed a psi value.
3. **Do not copy IP's Geertsma uniaxial pore compressibility.** D1/§6.2 shows the two
   branches disagree in IP 2025 and both agreed in IP2018. Derive it once, cite the source,
   and note the IP divergence in the docs. This is precisely the "silently wrong" class.
4. **Eaton's sonic ratio is inverted relative to resistivity/velocity/Dxc.** Implement the
   four variants as four explicit expressions, not one parameterised ratio with a sign flag.
5. **Named-correlation coverage gaps vs IP 2025 worth deciding on deliberately:**
   IP has *no* Mogi-Coulomb (only MC, Modified Lade, Hoek-Brown); *no* implemented Katahara
   acoustic-to-density (referenced only); and the Experienced-Eye-style Kf coefficient tables
   for five of the seven PPFG fracture-gradient relationships are closed.
6. **Smectite/illite is a first-class citizen in IP 2025's PPFG toolbox** and Mahakam-delta
   work will hit it. The two published acoustic→density lines (2.918 − 0.00517·Dt and
   3.044 − 0.00505·Dt) and the Katahara tanh blend with 160/220 °F endpoints are fully
   documented and reimplementable. The Alberty *velocity* NCT constants are not (O-4).
7. **Anisotropy feeds strength/stress only through the moduli.** IP does not anisotropise
   UCS directly — anisotropic strength is handled separately by the Planes-of-Weakness
   module with its own Mohr-Coulomb line, running *in parallel* with the isotropic path.
   That architectural split (anisotropic *elastic* module → moduli; PoW module → strength)
   is worth copying; it keeps the strength correlations single-valued.
8. **The `.par` / `.obg` file pattern is a good precedent**: `Fract_Grad_Coeff.par`,
   `Poisson_Ratio_Lithologies.par`, `OBG_Files.obg` are plain text in the install directory,
   editable by the user. SandiBumi's parameter provenance discipline maps onto this well —
   with the addition that each row should carry its citation.
9. **Every default in §3 traces to a named source in the vendor text.** Where it does not
   (Hoek-Brown weak-rock defaults, Alberty A/B/C/D, five of seven Kf sets), that is recorded
   in §8 rather than filled in. None of those may be defaulted in SandiBumi without an
   independent citation.

---

## 8. OPEN ITEMS

| # | Item | Why it is open |
|---|---|---|
| **O-1** | **TTI trigonometric solve.** The manual names the inputs (`mu_qP`, `mu_qSV`, `mu_SH`, `mu_ST`, `TTI_THETA`) and the outputs (`C44`, `C66`, `C33`) but prints neither the direct system nor the iterative scheme — only "a series of trigonometric equations are solved". Not reconstructable from the manual. |
| **O-2** | **Alberty NSC subscripts** (`ppfg_021`): cannot resolve whether the bracket is `(RHO0_ma − RHO)` or `(RHO0 − RHO_ma)`. Structural scan of the raster found no fraction bar and no tall parentheses to disambiguate. The parallel Traugott form is unambiguously `RHOMA + (RHO0 − RHOMA)·Exp(…)`, which suggests the second reading, but the manual does not say so. |
| **O-3** | **Katahara avg→actual parenthesisation** (`ppfg_031`): literally `RHOB = 16.3 + 1.6*((DBML/3125)^0.6)/8.345`, where `/8.345` binds only to the last term and mixes units. The physically consistent reading divides the whole right-hand side (ppg → g/cc). At DBML = 5,000 ft the two readings give **16.55** vs **2.206** — a 7.5× difference. Structural scan confirmed no fraction bar. |
| **O-4** | **Alberty/McLean velocity-NCT constants A, B, C, D** for smectite-rich and illite-rich shale are never printed. The manual gives only the form `Vp = A + B·e^(C·ES^D)` and says one set exists per clay type. **This blocks reimplementation of the flagship smectite/illite pore-pressure method.** |
| **O-5** | **Kf coefficient sets** for Eaton World Oil, Matthews & Kelly LA GC, Matthews & Kelly S. Tx, GeoFluids (Casey 2015), Hess Malaysia Low Side and Hess Malaysia High Side. Only the generalized form and the Fixed-Kf salt guidance are published. |
| **O-6** | **Hoek-Brown default values** for `mb`, `s`, `a` (or `GSI`, `D`, `mi`). Described as "based on an analysis for weak rock"; numbers not printed. |
| **O-7** | **Matthews & Kelly `Ki` lookup tables** (LA Gulf Coast, S. Texas Gulf Coast). Stated to be digitized from Reference 9 and stored in `Fract_Grad_Coeff.par`; values not in the manual. |
| **O-8** | **Daines per-lithology Poisson's ratios** — in `Poisson_Ratio_Lithologies.par`, not printed. |
| **O-9** | **Bellotti (consolidated & unconsolidated), Lindseth, Gardner power-law and Gardner polynomial coefficients.** These live in the Density Estimation module, which is not one of my 22 pages. Only the Gardner generalized form (`7.27·Dt^−0.25`, ≡ A = 0.23 / B = 0.25) is given here. **Cross-agent: whoever holds `density_estimation` should have these.** |
| **O-10** | **Traugott World Oil average-density correlation** (`ppfg_030`) — I have the raster identified but the transcription of its coefficients is not recorded here; it is the input to the ambiguous `ppfg_031` (O-3) and should be resolved together with it. |
| **O-11** | **Eberhart-Phillips `1460` placement** (D14). Both readings are printed-plausible; the arithmetic favours the un-bracketed form by a factor of 10. Needs a look at the original Eberhart-Phillips, Han & Zoback 1989 paper, not the manual. |
| **O-12** | **Effective-stress sign in the breakout inversions** (D11). `+ αPp` is what the rasters show. Independent derivation needed before use. |
| **O-13** | **Miller RHOB porosity exponent structure** (`ppfg_024`): transcribed as `porA + porB*Exp(−Kdecl * P(Dbml)^(1/Pcur))`, but the grouping of `P(Dbml)` and the curvature exponent could not be confirmed from the raster alone. The parameter list (porA, porB, Kdecl, curvature) is unambiguous; the exact nesting is not. |

---

**Montmorillonite:** no montmorillonite endpoint appears anywhere in these 22 pages.
The clay-mineral content of this suite is entirely **smectite / illite / kaolinite**, and
is concentrated in `overburden_tools`, `acoustic_to_pressure` and the shale-salinity module
of `resistivity_to_pressure` (§2.6.1, §3.5).

**Delegation:** none. All 22 pages, all equation rasters and both IP2018 diffs were read on
the session model.
