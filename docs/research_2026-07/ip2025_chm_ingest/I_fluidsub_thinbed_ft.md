# Agent I — Fluid Substitution, Laminated / Thin-Bed, PVT, Formation Testing

**Source:** Interactive Petrophysics 2025 vendor manual (decompiled CHM), clean-text + raw HTML + equation rasters.
**Diff baseline:** IP 2018 CHM (`c18`).
**Provenance convention used throughout:**

- `(pagename.htm)` — fact taken from page prose / ASCII text.
- `[img-read: file.png]` — fact transcribed by me from the raster image with vision.
- `[img-read: file.png; pixel-verified]` — an ambiguous operator or exponent resolved by a pixel-level
  stroke detector (short-horizontal-run test + centre-column vertical-stroke test), not by eye.
- `[ip2018: file.gif]` — the same raster confirmed identical in the 2018 manual.

Vendor prose is copyrighted and is **paraphrased**; equations, constants, defaults and units are facts and
are captured in full. No vendor file was copied; nothing was written outside this file.

---

## 1. Scope & page inventory

All nine assigned pages were read in full. Every content image on each page is accounted for below —
either transcribed, or listed with the reason it was not transcribed.

| # | Page (`_text.txt` / `.htm`) | Title | Chars | Content imgs | Status |
|---|---|---|---|---|---|
| 1 | `laminatedfluidsubs` | Laminated Reservoir Fluid Substitution | 42,188 | 74 | Read; 38 equation rasters + 20 dialogs/figures transcribed |
| 2 | `fluidsubstitution` | Fluid Substitution | 22,108 | 79 | Read; 47 equation rasters + 12 dialogs transcribed |
| 3 | `ft_create_and_analyse_a_formation` | Create and Analyse a Formation Test Project | 11,851 | 34 | Read; 6 dialogs transcribed |
| 4 | `pvtfluidproperties` | PVT Fluid Properties | 11,668 | 6 | Read; 2 dialogs transcribed; 2 formulas in ASCII |
| 5 | `ft_define_the_formation_testing_p` | Define the Formation Testing Parameters | 10,802 | 12 | Read; 3 dialogs transcribed |
| 6 | `formation_test_analysis` | Formation Test Analysis | 8,373 | 7 | Read; 4 result-pane images transcribed |
| 7 | `ft_equations_and_methodology` | Formation Testing Equations and Methodology | 4,595 | 18 | Read; **all 13 equation rasters** + derivative transcribed |
| 8 | `laminated_sands_workflow` | Laminated Sands Workflow | 3,756 | 10 | Read; 6 dialogs/plots transcribed |
| 9 | `pressuredifferentiation` | Pressure Differentiation | 2,435 | 1 | Read; dialog transcribed |

**Images deliberately not transcribed** (accounted for, no extractable numeric/method content):

- `fluidsubstitution.htm`: ~24 single-glyph "Where…" symbol images (`embim510, 514, 516, 518, 519, 526,
  528, 531, 532, 534, 535, 547–552, 555, 556`) — each is one italic symbol already defined in the adjacent
  ASCII text; `_rpclip0019` (Discriminators dialog, curve-name pickers only); `_rpclip0022` (example log
  plot); `_rpclip0072` (25×25 toolbar icon); plus chrome (`contents_off`, `menu`, `arrow_*`, `home`,
  `printer`, `eye*`, `application_get`, `ipexpandarrow`).
- `laminatedfluidsubs.htm`: `_rpclip0037/0038` (Make-Plot and Print menus, three template names, all named
  in the prose); `_rpclip0059–0067`, `_rpclip0070` (QC log plots and cross-plots whose numeric content is
  restated in the prose or in the dialogs already transcribed).
- Other pages: navigation chrome and repeated `_ftaclip00104.png` (a "Δ" glyph used inline as the delta
  prefix on ΔP / Δt).

**Modules and where they live**

- Fluid Substitution: **GeoEng → Rock Physics → Fluid Substitution** (`fluidsubstitution.htm`).
  *(IP2018 placed it under Advanced Interpretation — see §6.)*
- Laminated Fluid Subs: **Advanced Interpretation → Rock Physics → Laminated Fluid Subs**
  (`laminatedfluidsubs.htm`).
- Laminated tensor-resistivity workflow: inside **Porosity & Water Saturation**
  (`laminated_sands_workflow.htm`).
- PVT: the **PL (Production Logging)** workflow (`pvtfluidproperties.htm`).
- Formation Testing: its own project type, `*.fta` (`ft_*.htm`, `formation_test_analysis.htm`).
- Pressure Differentiation: PL utility (`pressuredifferentiation.htm`).

---

## 2. Equations & methods per module

### 2.1 Fluid Substitution — Average Gassmann tab (`fluidsubstitution.htm`)

Fluid properties are computed from **Batzle & Wang (1992), *Seismic Properties of Pore Fluids***, cited
by name on the page (`fluidsubstitution.htm`). The page states the general identity
**Bulk Modulus = Density × Velocity²** and the dialogs repeat it as `K = rho x V²`
[img-read: _rpclip0015.png].

| ID | Equation as printed | Provenance |
|---|---|---|
| embim509 | μ = Vs² × ρ_b | [img-read: embim509.png] |
| embim511 | K = Vp² × ρ_b − 4/3 × μ | [img-read: embim511.png] |
| embim512 | K_ma = ½ { ( Σᵢ₌₁ⁿ Volᵢ·K_min,ᵢ ) + ( Σᵢ₌₁ⁿ Volᵢ / K_min,ᵢ )⁻¹ } | [img-read: embim512.png] |
| embim513 | ρ_ma = Σᵢ₌₁ⁿ Volᵢ × ρ_ma,min,ᵢ | [img-read: embim513.png] |
| embim515 | ρ_f = Sxo·ρ_brine + (1 − Sxo)·ρ_hydrocarbon | [img-read: embim515.png] |
| embim517 | K_f = Sxo^Exp × (K_brine − K_hydrocarbon) + K_hydrocarbon | [img-read: embim517.png] |
| embim520 | *(Reuss option)* K_f = 1 / ( Sxo/K_brine + (1 − Sxo)/K_hydrocarbon ) | [img-read: embim520.png] |
| embim521 | φ = (ρ_ma − ρ_b) / (ρ_ma − ρ_f) | [img-read: embim521.png] |
| embim522 | K_d = [ K·(φ·K_ma/K_f + 1 − φ) − K_ma ] / [ φ·K_ma/K_f + K/K_ma − 1 − φ ] | [img-read: embim522.png] |
| embim523 | ν_d = (3·K_d − 2μ) / (2μ + 6·K_d) | [img-read: embim523.png] |
| embim524 | R = K_d / μ  *(Modulus Ratio)* | [img-read: embim524.png] |

**embim512 is a Voigt–Reuss–Hill average** — the arithmetic (Voigt) and harmonic (Reuss) mixes of the
mineral moduli, averaged with a fixed ½ weighting. Unlike the laminated module, the weight is **not**
user-adjustable here.

**Mixing-law semantics** (`fluidsubstitution.htm`):

- embim517 is the **Brie et al. (1995)** empirical fluid-mixing law, cited on the page as SPE 30595,
  pp. 701–710. The page states exponent **3** simulates patchy saturation for gas/water, and **1.5** is
  approximately a Voigt–Reuss average for oil/water.
- The **Reuss (harmonic) average is OFF by default** — a checkbox on the tab
  [img-read: _rpclip0017.png]. When on, embim520 replaces embim517.
- The page distinguishes frequency regimes: the **homogeneous (Reuss) mix is what the Log Fluid
  Substitution tab uses** because it targets seismic frequencies, whereas the **empirical (Brie) mix is
  used for the dry-frame inversion** at logging frequencies (`fluidsubstitution.htm`).

### 2.2 Fluid Substitution — Fluid Substitution Crossplot

| ID | Equation as printed | Provenance |
|---|---|---|
| embim525 | K_p = φ_ag / ( 1/K_d **−** 1/K_ma ) | [img-read: embim525.png; pixel-verified] |
| embim527 | K_d = 1 / ( φ_xp/K_p **−** 1/K_ma ) | [img-read: embim527.png; pixel-verified] |
| embim529 | μ = ¾ × K_d · ( 3(1 − ν_d)/(1 + ν_d) − 1 ) | [img-read: embim529.png] |
| embim530 | ρ_f = Sw·ρ_brine + (1 − Sw)·ρ_hydrocarbon | [img-read: embim530.png] |
| embim533 | K_f = 1 / ( Sw/K_brine + (1 − Sw)/K_hydrocarbon ) | [img-read: embim533.png] |
| embim536 | ρ_b = ρ_f·φ_xp + (1 − φ_xp)·ρ_ma | [img-read: embim536.png] |
| embim537 | Vs = √(μ / ρ_b) | [img-read: embim537.png] |
| embim538 | K = K_d + (1 − K_d/K_ma)² / [ φ_xp/K_fl + (1 − φ_xp)/K_ma − K_d/K_ma² ] | [img-read: embim538.png] |
| embim539 | Vp = √( (K + 4/3 μ) / ρ_b ) | [img-read: embim539.png] |
| embim540 | AI = ρ_b × Vp | [img-read: embim540.png] |
| embim541 | ν = (Vp² − 2Vs²) / ( 2(Vp² − Vs²) ) | [img-read: embim541.png] |

embim538 is the standard Gassmann forward form; embim525/527 are the pore-modulus pair. **The signs in
525 and 527 are mutually inconsistent** — see §5(i).

### 2.3 Fluid Substitution — Log Fluid Substitution tab

Saturation of the flushed zone:

| ID | Equation as printed | Provenance |
|---|---|---|
| embim542 | **WBM:** Sxo = (Sw + Inv) / (1 + Inv) | [img-read: embim542.png], [img-read: _rpclip0016.png] |
| — | **OBM:** Sxo = min(Sw, SwMax) | [img-read: _rpclip0016.png] |

`Inv` is shown as `IV` in the dialog, default **1.0**; `SwMax` default **0.5**
[img-read: _rpclip0016.png].

Density bookkeeping — three output states (100 % brine, reservoir, substituted):

| ID | Equation as printed | Provenance |
|---|---|---|
| embim543 | ρ_f = Sxo·ρ_brine + (1 − Sxo)·ρ_hydrocarbon | [img-read: embim543.png] |
| embim544 | ρ_b100 = ρ_blog + (ρ_fBrine − ρ_fSxo)·φ | [img-read: embim544.png] |
| embim545 | ρ_bRes = ρ_blog + (ρ_fRes − ρ_fSxo)·φ | [img-read: embim545.png] |
| embim546 | ρ_bSub = ρ_blog + (ρ_fSub − ρ_fSxo)·φ | [img-read: embim546.png] |
| embim553 | K_ma — same VRH form as embim512 | [img-read: embim553.png] |
| embim554 | K_fSxo = Sxo^Exp × (K_brineSxo − K_hydrocarbonSxo) + K_hydrocarbonSxo | [img-read: embim554.png] |

**Dry-frame inversion when no shear log exists** — the page attributes the Vs prediction to
**Gregory (1977)** and notes the quadratic solution follows Hampson & Russell
(`fluidsubstitution.htm`). A dry-rock Poisson ratio is supplied by the user; the page gives the rule of
thumb **0.1–0.2 for consolidated rock (average 0.15)** and **0.1–0.25 for unconsolidated**
(`fluidsubstitution.htm`).

| ID | Equation as printed | Provenance |
|---|---|---|
| embim557 | S = 3(1 − ν_d) / (1 + ν_d) | [img-read: embim557.png] |
| embim558 | c = −φ × ( S − ρ_b·Vp²/K_ma ) × ( K_ma/K_fSxo − 1 ) | [img-read: embim558.png] |
| embim559 | a = S − 1 | [img-read: embim559.png] |
| embim560 | b = φ·S·(K_ma/K_fSxo − 1) − S + ρ_b·Vp²/K_ma | [img-read: embim560.png] |
| embim561 | K_d = K_ma · ( 1 − ( −b + √(b² − 4ac) ) / (2a) ) | [img-read: embim561.png] |
| embim562 | μ = K_d × 1.5 · (1 − 2ν_d) / (1 + ν_d) | [img-read: embim562.png] |

Note embim561 takes the **`−b + √…` root only**; no branch selection or discriminant guard is documented.

Shear and compressional outputs:

| ID | Equation as printed | Provenance |
|---|---|---|
| embim563 / 564 / 565 | Vs_100 = √(μ/ρ_b100); Vs_Res = √(μ/ρ_bRes); Vs_Sub = √(μ/ρ_bSub) | [img-read: embim563–565.png] |
| embim566 | K_Sxo = K_ma · [ K_fSxo/(φ(K_ma − K_fSxo)) + K_d/(K_ma − K_d) ] / [ 1 + K_fSxo/(φ(K_ma − K_fSxo)) + K_d/(K_ma − K_d) ] | [img-read: embim566.png] |
| embim567 | K_100 — same form, K_Brine substituted | [img-read: embim567.png] |
| embim568 | Vp_100 = √( (Vp²·ρ_b − K_Sxo + K_100) / ρ_b100 ) | [img-read: embim568.png] |
| embim569 | K_fRes = 1 / ( Sw/K_brineRes + (1 − Sw)/K_hydrocarbonRes ) | [img-read: embim569.png] |
| _rpclip0073 | K_Res — embim566 form with K_fRes | [img-read: _rpclip0073.png] |
| _rpclip0074 | Vp_Res = √( (Vp²·ρ_b − K_Sxo + K_Res) / ρ_bRes ) | [img-read: _rpclip0074.png] |
| embim570 | K_fSub = 1 / ( Sw/K_brineSub + (1 − Sw)/K_hydrocarbonSub ) | [img-read: embim570.png] |
| embim571 / 572 | K_Sub; Vp_Sub — same forms | [img-read: embim571.png, embim572.png] |

embim566/567 is Gassmann written in **modulus-ratio ("Biot–Gassmann quotient") form**, algebraically the
same as embim538 but numerically better behaved near K_d → K_ma. **Vp is updated by a modulus *increment*
(embim568: subtract K at Sxo, add K at the target state) rather than by recomputing Vp from scratch** —
so any error in the mineral modulus partly cancels.

Note embim569/570 use the **Reuss harmonic** fluid mix, while embim554 (the Sxo state, used for the dry
inversion) uses the **Brie exponent** mix. That is the deliberate frequency split described above, not an
inconsistency.

### 2.4 Laminated Reservoir Fluid Substitution (`laminatedfluidsubs.htm`) — HIGH PRIORITY

**Attribution.** The page states the module was developed by **Chris Skelt, Chevron**, and cites
Skelt, "Fluid substitution in Laminated Sands", *The Leading Edge*, May 2004, and Skelt, SPWLA 45th Annual
Logging Symposium, 2004 (`laminatedfluidsubs.htm`).

**The model is NOT Backus averaging and contains no tensor/anisotropic elastic formulation.** It is a
*volumetric partition* model: effective porosity is re-scaled into the sand laminations, the full Gassmann
fluid substitution is performed **in the sand fraction only**, and the resulting elastic *effects* are then
scaled back down by the net-to-gross of sand laminations.

| ID | Equation as printed | Provenance |
|---|---|---|
| embim600 | φ_sandlams = φ_effective / (1 − V_shalelams) | [img-read: embim600.png] |
| embim601 | fluid effects are attenuated by multiplying by (1 − V_shalelams) | [img-read: embim601.png] |

The prose states the same relation as **Phi_Slam = Phie / (1 − VshLam)** (`laminatedfluidsubs.htm`). The
figure states the partition explicitly: *effective porosity = POROSITY/(POROSITY+SAND+SHALE)*; *sandy
lamination porosity = POROSITY/(POROSITY+SAND)*; fluid effects are determined in the sand fraction and then
scaled by the N/G ratio indicated by the GR log [img-read: _rpclip0050.png].

**VshLam is not VShale.** The page is explicit that VshLam is the volume fraction of *shale laminations*,
derived from core-calibrated sand fraction or from a Thomas–Stieber density–neutron analysis
(`laminatedfluidsubs.htm`, l.543), and that a **Vshale (not Vclay) interpretation is recommended** as the
input. A maximum of **five rocks** is allowed, and lumping quartz/feldspar/mica (QFM) is permitted
(`laminatedfluidsubs.htm`).

**Elastic property formulae.** The section header states the unit contract: **moduli in GPa, velocities in
ft/s, impedances in g/cm³·km/s, density in g/cm³, slowness in µs/ft** (`laminatedfluidsubs.htm`).

| ID | Equation as printed | Provenance |
|---|---|---|
| embim573 | AI_c = 304.8 × ρ_b / Δt_c | [img-read: embim573.png] |
| embim574 | AI_s = 304.8 × ρ_b / Δt_s | [img-read: embim574.png] |
| embim575 | VpVs = Δt_s / Δt_c | [img-read: embim575.png] |
| embim576 | K_b = ρ_b × ( 304.8/Δt_c² − 4×304.8/(3·Δt_s²) ) | [img-read: embim576.png; pixel-verified], [ip2018: embim389.gif] |
| embim577 | v_p = 304.8 / Δt_c  (km/s) | [img-read: embim577.png] |
| embim578 | v_s = 304.8 / Δt_s  (km/s) | [img-read: embim578.png] |
| embim579 | PR = 0.5 × (Δt_s² − 2Δt_c²) / (Δt_s² − Δt_c²) | [img-read: embim579.png] |
| embim580 | P5AI (Hilterman) = 0.5 × ln( 304.8 × ρ_b / Δt_c ) | [img-read: embim580.png] |
| embim581 | λρ = (ρ_b·v_p·0.3048×10⁻³)² − 2(ρ_b·v_s·0.3048×10⁻³)² | [img-read: embim581.png] |
| embim582 | μρ = (ρ_b·v_s·0.3048×10⁻³)² | [img-read: embim582.png] |
| embim583 | G = ρ_b × (v_s·0.3048×10⁻³)² | [img-read: embim583.png] |
| embim584 | E = ρ_b/Δt_s² × (3Δt_s² − 4Δt_c²) / (Δt_s² − Δt_c²) | [img-read: embim584.png; pixel-verified] |
| embim585 | M = ρ_b × (V_ρ·0.3048×10⁻³)² | [img-read: embim585.png] |
| embim586 | Vsand = 1 − Vshale − φ_effective | [img-read: embim586.png] |

**304.8 is the ft/s ← µs/ft conversion (10⁶ × 0.3048 / 10³ = km/s).** embim576 and embim584 print it
un-squared where dimensional analysis requires 304.8²; see §5(ii).

**"Observed → Wet" program (the dry-frame inversion core):**

| ID | Equation as printed | Provenance |
|---|---|---|
| embim587 | ρ_wet = ρ_obs + φ × (1 − S_xo) × (ρ_wtr − ρ_HC) | [img-read: embim587.png] |
| embim588 | Δt_swet = Δt_sobs × √(ρ_wet / ρ_obs) | [img-read: embim588.png] |
| embim589 | 1/K_fWoods = S_W/K_W + S_filtrate/K_filtrate + S_HC/K_HC | [img-read: embim589.png] |
| embim590 | K_fVoigt = S_W×K_W + S_filtrate×K_filtrate + S_HC **+** K_HC | [img-read: embim590.png; pixel-verified], [ip2018: embim403.gif] |
| embim591 | K_f = Woodfac × K_fWoods + (1 − Woodfac) × K_fVoigt | [img-read: embim591.png] |
| embim592 | 1/K_rReuss = ( V_sand/K_r,sand + V_silt/K_r,silt + V_shale/K_r,shale ) / (1 − φ) | [img-read: embim592.png] |
| embim593 | K_rVoigt = ( V_sand·K_r,sand + V_silt·K_r,silt + V_shale·K_r,shale ) / (1 − φ) | [img-read: embim593.png] |
| embim594 | K_r = Voigtfac × K_rVoigt + (1 − Voigtfac) × K_rReuss | [img-read: embim594.png] |
| embim595 | G = ρ_wet/Δt_swet² = ρ_obs/Δt_sobs²  *(shear modulus invariant to fluid)* | [img-read: embim595.png] |
| embim596 | K_b = ρ × ( 1/Δt_c² − 4/(3·Δt_s²) )  *(no 304.8 — consistent-unit form)* | [img-read: embim596.png] |
| embim597 | K_df = [ K_b·(φ/K_flxo + (1 − φ)/K_r) − 1 ] / [ φ/K_flxo + (1 − φ)/K_r − 2/K_r + K_b/K_r² ] = tmp1_OtoW / tmp2_OtoW | [img-read: embim597.png] |
| embim598 | K_bWet = K_df + (1 − K_df/K_r)² / [ φ/K_wtr + (1 − φ)/K_r − K_df/K_r² ] = K_df + tmp3_OtoW/tmp4_OtoW | [img-read: embim598.png] |
| embim599 | Δt_cWet = √( ρ_bWet / (K_bWet + 4G/3) ) | [img-read: embim599.png] |

embim597 is algebraically the standard Gassmann dry-frame inversion; I re-derived it and confirmed
equivalence. **embim591 (Woodfac) and embim594 (Voigtfac) are the two user-tunable mixing dials** —
Woodfac **default 1** (pure Wood's/Reuss fluid mix), Voigtfac **default 0.5** (Hill average of the solid
moduli) [img-read: _rpclip0025.png, _rpclip0028.png]. The prose states Woodfac = 1 honours Wood's Law,
0 mixes by Voigt average of bulk moduli, intermediate values are weighted averages
(`laminatedfluidsubs.htm`, l.104), and that Voigt Mix Fact = 0.5 is synonymous with the Hill average and is
recommended absent counter-indications (l.135).

**Convergence.** The workflow block diagram states: *iterate until sand-lamination DT shear changes by
less than* **0.001 µs/ft** [img-read: _rpclip0057.png]. A diagnostic curve `Count_OtoW` records the
iteration count [img-read: _rpclip0036.png].

**Alternative dry-frame (K_df) models.** Model is a zoned parameter; options are
**Gassmann | CPM | Krief | Soft Sand | Stiff Sand | Intermediate | Mixed | External**, default **Gassmann**
[img-read: _rpclip0032.png].

| ID | Model | Equation as printed | Provenance |
|---|---|---|---|
| embim602/603 | Gassmann | K_dryframe = f(K_b, K_fluid, K_solid, φ) and its inverse | [img-read: embim602.png, embim603.png] |
| embim604 | Gassmann (ratio form) | K_b/(K_solid − K_b) = K_dryframe/(K_solid − K_dryframe) + K_fluid/[φ(K_solid − K_fluid)] | [img-read: embim604.png] |
| embim605 | Critical Porosity (CPM) | K_df = K_solid × (1 − φ/φ_crit); figure states K_dry = K_Solid × (φ_Crit − φ_Obs)/φ_Crit | [img-read: embim605.png] |
| embim606 | Krief | K_df = K_solid × (1 − φ)^( m/(1 − φ) ); figure states exponent 3/(1 − φ_Obs) | [img-read: embim606.png] |
| embim607 | Hertz–Mindlin | K_HM = [ C²(1 − φ₀)²·G_R²·P / (18π²(1 − ν)²) ]^(1/3)   and   G_HM = [ (2 + 3f − ν(1 + 3f)) / (5(2 − ν)) ] × [ 3C²(1 − φ₀)²·G_R²·P / (2π²(1 − ν)²) ] | [img-read: embim607.png], [ip2018: embim420.gif] |
| embim608 | Soft sand (lower Hashin–Shtrikman) | K_eff = [ (φ/φ_D)/(K_HM + 4/3·G_HM) + (1 − φ/φ_D)/(K + 4/3·G_HM) ]⁻¹ − 4/3·G_HM | [img-read: embim608.png] |
| embim609 | Stiff sand (upper Hashin–Shtrikman) | same as embim608 with G_R substituted for G_HM throughout | [img-read: embim609.png] |

Model semantics from the prose (`laminatedfluidsubs.htm`):

- CPM: dry bulk **and** shear moduli are both zero at critical porosity; the page calls the model purely
  empirical with no claimed physical basis, and cautions that the apparent analogy between critical porosity
  and Gassmann's f_R is illusory (l.577–580). **IP's implementation does not honour zero shear at φ_crit** —
  it estimates the shear modulus from the bulk modulus and the Greenberg–Castagna coefficients instead
  (l.583). *This is a documented deviation of the code from the model.*
- Krief: differs from CPM at very high porosity and, unlike CPM, permits shear propagation there;
  m default 3 (l.591).
- Hertz–Mindlin family: C = coordination number; f = fraction of grain contacts with perfect adhesion
  (l.603–606).
- **Intermediate** = the soft-sand model with **K_HM defined by a coordination number of 15** (l.611).
- **Stiff sand** follows the upper HS bound and **assumes adhesion fraction of one** (friction prevents
  slippage) (l.614).
- **Mixed** = weighted average of soft and stiff, favouring soft at high porosity and stiff at low (l.616).

**Modelled-saturation ("fill with hydrocarbon") correlation:**

| ID | Equation as printed | Provenance |
|---|---|---|
| embim610 | S_wHC = a + b·φ_e + c·φ_e² | [img-read: embim610.png] |
| — | worked example printed on the crossplot: **Sw = 1.219 − 5.923·Phie + 8.531·Phie²** | [img-read: _rpclip0066.png] |

The page is explicit that these coefficients are *field-specific and picked off a crossplot* — it says the
example "worked in one of our fields" and that the correlation was clipped at one
(`laminatedfluidsubs.htm`, l.672). **They are not defaults and must not be reused as such.**

**Special cases** (`laminatedfluidsubs.htm`):

- **Dry rock:** set Sw = 0, Sg = 1, and both gas density and gas bulk modulus to 0.
- **Fizz gas:** clip the modelled Sw curve between **0.85 and 1.0** (l.675).
- **Gas reservoir:** set oil saturation to zero; in oil-and-gas wells compute one of oil/gas saturation as
  zero and the other as 1 − Sw (l.671).
- Low hydrocarbon saturations in water zones must be removed first, to prevent spurious modelled effects
  (l.671).

**Output curve naming** — three letters: model (**L**aminated / **S**haley) + hydrocarbon type
(**G**as / **O**il / **W**ater / **I** in-situ) + saturation state (**H**ydrocarbon / **I**n-situ /
**F**luid): e.g. LII, SII, LGI, LGH, SOH (`laminatedfluidsubs.htm`).

**Failure flag `No_Com_Flg`:** 0 = computed; 1 = porosity or Vsh outside the cutoffs;
**2 = Gassmann inapplicable because K_formation > K_solid** (`laminatedfluidsubs.htm`).

### 2.5 Laminated Sands Workflow — tensor Rh/Rv (`laminated_sands_workflow.htm`)

A 12-step operator workflow inside Porosity & Water Saturation. Substantive rules:

1. Horizontal and vertical tensor resistivity curves are entered on the Input Curves tab.
2. "Laminated Sand Analysis" offers **Clay Model** or **Shale Model** — this selects whether the
   Thomas–Stieber laminated volumes come from a **Vshale/Phie** plot or a **Vclay** plot (l.90).
3. **"Do not use the Poupon equation."** (l.98) — a direct constraint on the saturation model.
4. Sat Model = **Laminated**; Rt Lam model = **Tensor Rsh** (or **Tensor Vlam** in the modified workflow)
   (l.101, l.144).
5. Wet and dry clay points are set on the neutron/density crossplot; **PhiMax** (clean-sand porosity) is set
   on the Thomas–Stieber crossplot (l.110, l.118).
6. Coupling warning: if **PhiT clay is left blank** it is derived from Rho Dry Clay and Rho Wet Clay, so
   dragging the PhiT-clay point on the crossplot **back-solves and changes Rho Dry Clay**, and the plotted
   data move underneath you (l.119).
7. The shale point is set on the **ResVert/ResHori "butterfly" crossplot** (l.127).
8. QC: **VlamTensor should read similar to Vlam from Thomas–Stieber** (l.135) — an independent
   cross-check of the resistivity-derived lamination fraction against the porosity-derived one.
9. Modified "Tensor Vlam" workflow: shale anisotropy is set by dragging an interactive line in the
   Anisotropy track to match the input Rv/Rh in pure shale (l.145).
10. **"Res Lam Shale" selects which root of the tensor model is used** — high value → sand resistivity
    solves below shale resistivity, low value → above. The page states **the absolute value is not
    important**; it is a branch selector, not a physical input (l.153).

No equations are printed on this page; the tensor mathematics is cross-referenced to *Porosity and Sw
Equations and Methodology* (another agent's page).

### 2.6 PVT Fluid Properties (`pvtfluidproperties.htm`)

Correlations offered **by name only** — prose list: *Standing, Vasquez Beggs, Glaso, Lasater* (l.94);
dialog list: *Standing, Vasquez Beggs, **Glasso**, Lasater* [img-read: _plclip00099.png]. **No coefficients
for any of the four are printed anywhere on this page.** User `*.PVT` lookup tables may be selected instead,
from `\Program Files\IntPetro\PL` (or the folder set in Preferences) (l.92).

Only two formulas are printed, both stated to be in **oilfield units** (l.122–125, repeated l.195–199):

```
Dens_oil_at_surface = 141.5 / (131.5 + Oil gravity in API)
Dens_oil_at_res     = (Dens_oil_at_surface + 0.0002178 * Gas gravity * Rs) / Bo
```

The first is the standard API→SG relation. The second is a mass-balance: 0.0002178 converts scf of solution
gas per stock-tank barrel, times gas relative density, into g/cc of dissolved mass.

Method facts (`pvtfluidproperties.htm`):

- The correlations **first compute bubble-point pressure**, then automatically branch to above- or
  below-bubble-point code based on the measured pressure (l.127).
- Gas and water PVT are computed the same way regardless of the lookup table or correlation chosen —
  the correlation choice affects **oil only** (l.84).
- N₂, CO₂ and H₂S percentages, if significant and known, **modify the Z-factor calculation** (l.139, l.163).
  No Z-factor equation is printed.
- Tabulated against pressure in a `.PVT` table: **Bo, RSO/GOR, Uo, DENo** (l.191, l.201–206). If DENo is
  absent, the two formulas above are used.
- Water salinity is entered in **ppm NaCl equivalent (total salinity)**. **If chloride (Cl⁻) values are
  supplied, multiply by ~1.65** to get total salinity (l.171–173).
- Gas-saturated vs gas-free water correction is described as minor (l.179).
- **Americium-source (old Sondex) density tool:** measures electron density and the page states analysis of
  many datasets showed it **doubles the salinity density response** — worked example: fresh water 1.00 g/cc,
  normal saline 1.04 g/cc, Americium tool reads **1.08 g/cc** (l.181). Multiphase Flow Calculations uses the
  Americium-derived density and prints the fact on the report (l.183).
- Condensate/wet-gas: the gas-condensate ratio is held **constant for all reservoir zones** as a consequence
  of computing wet gas from stock-tank condensate and gas ratio (l.149). Condensate gravity is used to
  compute mole weight (l.145).
- Zonal P and T are taken **at the top of each zone** (l.214). Absent data (−999) at a zone top raises
  "Entry for pressure/temperature is out of range" (l.216). Edited zonal P/T values are **used but not
  stored** — print the PVT report to record them (l.218).

### 2.7 Formation Testing — equations (`ft_equations_and_methodology.htm`)

**All 13 equation rasters on this page are byte-identical to their IP 2018 counterparts** (see §6).

**Drawdown (SI form):**

| ID | Equation as printed | Provenance |
|---|---|---|
| _ftaclip0042 | k_d = C·q·μ / ( r_pe · ΔP_dd ) | [img-read: _ftaclip0042.png] |
| _ftaclip0043 | r_pe = 0.5·r_p  *(from Muskat)* | [img-read: _ftaclip0043.png] |
| _ftaclip0044 | r_pe = 2·r_p / π  *(from "Carlson and Jaeger")* | [img-read: _ftaclip0044.png] |
| _ftaclip0045 | M_dd = (k/μ) = C · ( 921·Q / ( r_s · ΔP_dd ) )  *(field units, mobility)* | [img-read: _ftaclip0045.png] |

Nomenclature exactly as printed (l.92–97): r_s = snorkel radius (**inches**); ΔP_dd = drawdown pressure
(**psi**); V = pre-test chamber volume (**cc**); T = pre-test drawdown time (**sec**);
Q = flow rate of pretest = V/T (**cc/sec**); **C = flow coefficient, 0.5 to 1.0, dimensionless**.

I verified the constant against the worked example: Q = 0.1764 cm³/s, r = 6.000 in, ΔP_dd = 4827.225 psi,
C = 0.668, μ = 2.500 cP → M = 0.668 × 921 × 0.1764 / (6.000 × 4827.225) = **0.00375 mD/cP**, printed as
**0.004 mD/cP**, and k = M × μ = 0.0094 mD, printed **0.009 mD**
[img-read: _ftaclip00099.png]. **The radius used is r_pe (probe radius × multiplier), not r_s** — see §5(v).

**Spherical buildup** (infinite-acting homogeneous medium):

| ID | Equation as printed | Provenance |
|---|---|---|
| _ftaclip0046 | p_i − p_s = [ 8×10⁴ · q₁ · μ · (φ·μ·c_t)^0.5 / k_s^(3/2) ] × f_s(Δt) | [img-read: _ftaclip0046.png] |
| _ftaclip0047 | f_s(Δt) = 1/√Δt **−** 1/√(t + Δt)  *(single rate)* | [img-read: _ftaclip0047.png; pixel-verified] |
| _ftaclip0048 | f_s(Δt) = (q₂/q₁)/√Δt **−** (q₂/q₁ − 1)/√(T₂ + Δt) **−** 1/√(T₁ + T₂ + Δt)  *(2-rate, RFT only)* | [img-read: _ftaclip0048.png; pixel-verified] |
| _ftaclip0049 | m_s = 8×10⁴ · q · μ · (φ·μ·c)^0.5 / k_s^(3/2) | [img-read: _ftaclip0049.png] |
| _ftaclip0050 | k_s = 1856 · μ · (q₁/m)^(2/3) · (φ·c_t)^(1/3) | [img-read: _ftaclip0050.png] |

Two self-consistency checks I ran and passed:
- **(8×10⁴)^(2/3) = 1856.6**, so _ftaclip0050 is the exact algebraic rearrangement of _ftaclip0049 — the
  1856 constant is internally consistent with the 8×10⁴ constant. ✔
- _ftaclip0048 collapses to _ftaclip0047 at q₂ = q₁ (the middle term vanishes and T₁+T₂ → t). ✔
- Worked example: q = 0.8988 cc/s, gradient −0.963, μ = 1.000 cP, c = 10×10⁻⁶, φ = 0.100
  → k_s = 1856 × 1.0 × (0.8988/0.963)^(2/3) × (0.1 × 1e−5)^(1/3) = **17.726 mD** vs printed **17.722 mD** ✔
  [img-read: _ftaclip00089.png].

**Radial buildup:**

| ID | Equation as printed | Provenance |
|---|---|---|
| _ftaclip0051 | p_i − p_c = ( 88.4·q₁·μ / (k_r·h) ) × f_c(Δt) | [img-read: _ftaclip0051.png] |
| _ftaclip0052 | f_c(Δt) = log( (t+Δt)/(0.5t+Δt) ) **+** log( (0.5t+Δt)/Δt )  *(single rate)* | [img-read: _ftaclip0052.png; pixel-verified] |
| _ftaclip0053 | f_c(Δt) = log( (T₁+T₂+Δt)/(T₂+Δt) ) **+** (q₂/q₁)·log( (T₂+Δt)/Δt )  *(2-rate, RFT only)* | [img-read: _ftaclip0053.png; pixel-verified] |
| _ftaclip0054 | k_r = 88.4·q₁·μ / (m·h) | [img-read: _ftaclip0054.png] |

_ftaclip0052 telescopes exactly to the Horner function log((t+Δt)/Δt) — the split form is presented but is
mathematically identical. Worked example: q = 0.8988, gradient −0.065, μ = 1.000, h = 2.5 ft →
k_r = 88.4 × 0.8988 / (0.065 × 2.5) = **488.9**, printed **101.619 mD** — the printed example does **not**
reproduce with h = 2.5 ft; see OPEN ITEMS (h for that example is not printed on the pane)
[img-read: _ftaclip00093.png].

**Derivative plot** (`ft_equations_and_methodology.htm`, l.163–167):

- ΔP = P(t) − P_flowing, with the buildup referenced to Time = 0 and its corresponding flowing pressure.
- embim392: **Derivative = d(ΔP) / d(Int)** [img-read: embim392.png].
- Plotted log–log.

**Flow-regime diagnosis** (`ft_create_and_analyse_a_formation.htm`): derivative-plot gradient of
**−1/2 indicates spherical flow**, **0 indicates radial flow**. The page carries an explicit caveat that
this technique is borrowed from well-test analysis and **may not be appropriate given the small fluid
volumes involved with a formation tester**.

**Which permeability is which** (`formation_test_analysis.htm`): **spherical buildup → vertical
permeability; radial buildup → horizontal permeability**; both yield P* by extrapolation to the
pressure axis. For radial analysis, viscosity μ and bed thickness h are required (h estimable from open-hole
logs) and the flow rate q is generally the average rate from the drawdown
(`ft_equations_and_methodology.htm`, l.134).

**RFT dual-rate equations apply only to Schlumberger RFT data** (l.80, l.109, l.139). Every other tool uses
the single-rate forms.

### 2.8 Formation testing — gradients, contacts, filters (`formation_test_analysis.htm`, `ft_create_and_analyse_a_formation.htm`)

**Pressure gradient crossplot** — regression printed on the plot as
**Pressure = Depth × slope + intercept**, X = Pfbu (psia), Y = TVDSS (FT); example slopes 0.120, 0.434,
0.370, 0.434 psi/ft; contact intersections annotated as (6502.7 psia, 7920.0 FT) and
(6688.0 psia, 8459.6 FT) [img-read: _ftaclip00102.png, _cpclip0164.png]. **The gradient-to-fluid-density
conversion constant is not printed on any of my pages** — see OPEN ITEMS.

**Excess Pressure plot** may be output as a logplot (`formation_test_analysis.htm`, new in 2025) — but
**no excess-pressure or supercharging equation appears on my pages**; the page cross-references the
Crossplot documentation.

**Filters** (`ft_create_and_analyse_a_formation.htm`): Square (box), Bell (sine-shaped), Median, and User
(weights are normalised automatically). **Maximum filter length 121 database sample increments if the filter
table is used, otherwise 2001.**

**Depth collision rule:** two formation tests at the same depth are written **0.01 ft (or m) apart** unless
Overwrite is selected.

**Project storage:** `*.fta` files in
`C:\users\<username>\appdata\local\IntPetro36\Formation Testing\`.

### 2.9 Pressure Differentiation (`pressuredifferentiation.htm`)

**What it computes:** a density curve derived by differentiating a pressure curve with respect to depth,
for use when no good density curve exists (l.80). Also used to QC measured density curves after deviation
correction (l.82). The page states it works even in an unstable well, in shut-in (l.83).

**Method and constraints** (all from `pressuredifferentiation.htm`):

- Input pressure curve must be high-resolution, very smooth after averaging, and not temperature-sensitive.
  Older crystal-type pressure tools are called out as too temperature-sensitive to be useful (l.80–81).
- **Units of the pressure curve must match those defined in PL Set-Up** (l.87) — explicit unit-contract
  warning.
- Optional filter: **odd number of samples, 3 to 21 inclusive, all samples equally weighted** (l.88).
  Dialog example shows 11 [img-read: _plclip00171.png].
- **A deviation curve or a deviation value in degrees is required** (l.89) — i.e. the differentiation is
  along TVD, and dP/dMD must be deviation-corrected.
- **Differentiation length default 15 ft or 5 m**; with a stable pressure curve 5 ft may give good results.
  Longer length = smoother curve but density changes get stretched (l.90).
- **The processed interval must be greater than the differentiation length or nothing is output**, and
  **half the differentiation length is lost at the top and at the bottom** of the result (l.91).
- Naming convention suggested for comparing lengths: DE40P1R1, DE10P1R1, etc.; a composite `DEND..` curve is
  assembled from the best intervals using the IP Formula module (l.93–94).
- To be consumed by Multiphase Flow Calculations the curve must be declared **primary density sensor,
  type 20**, in PL Set-Up (l.95).

**No equation is printed for the differentiation itself** — neither the difference stencil nor the
pressure-gradient-to-density constant.

---

## 3. Parameters, defaults & constraints

### 3.1 Fluid Substitution — mineral endpoints

Default mineral list in the dropdown: **Dolomite, Feldspar, Limestone, Mica, Quartz, Wet Clay**
[img-read: _rpclip0016.png]. **No smectite or montmorillonite endpoint appears anywhere on any of my nine
pages** (see §4).

| Mineral | ρ (g/cc) | K (GPa) | V (ft/s) | Source |
|---|---|---|---|---|
| Quartz | 2.65 | 37 | 19849 | [img-read: _rpclip0016.png] |
| Wet Clay | 2.60 | 21 | 11188 | [img-read: _rpclip0016.png] |

Same endpoints appear in the Average Gassmann results listing in m/s: Quartz ρ 2.650, K 37.000, V 6050 m/s;
Wet Clay ρ 2.600, K 21.000, V 3410 m/s (`fluidsubstitution.htm`). The page states the default mineral values
are taken from **Mavko, Mukerji and Dvorkin, *The Rock Physics Handbook*, cited as 1999** on this page
(`fluidsubstitution.htm`) — the laminated page cites the same work as 1998; see §5(vii).

### 3.2 Fluid Substitution — fluid property calculator (Batzle & Wang) inputs

| Parameter | Default / example | Units | Source |
|---|---|---|---|
| Temperature | 200 | Deg F (or Deg C) | [img-read: _rpclip0011.png] |
| Pressure | 6000 | PSI (or MPa / Bars) | [img-read: _rpclip0011.png] |
| Water salinity | 87775 | ppm NaCl (alternatively Rw 0.1 at Rw Temp 60) | [img-read: _rpclip0012.png] |
| Gas–water ratio (GWR) | — | m³/m³, with Gas Free / Gas Saturated toggle | [img-read: _rpclip0013.png] |
| Oil gravity | 45 | API | [img-read: _rpclip0014.png] |
| GOR | 2520.1 | SCuFt/bbl or m³/m³ | [img-read: _rpclip0014.png] |
| Gas density (rel. air) | 0.8 | — | [img-read: _rpclip0014.png, _rpclip0015.png] |

Computed outputs from that input set: Brine ρ 1.0421 g/cc, K 3.0059 GPa, V 5572.2; Oil 0.5468 / 0.3504 /
2626.3; Gas 0.2931 / 0.1345 / 222.5 [img-read: _rpclip0015.png].

A second calculator instance labelled **"Batzle & Wang 1992 Calculator"** shows Temperature 200,
Pressure 60, salinity 10 ppm NaCl, GWR 0.21 m³/m³, Oil API 45, GOR 36.8, Gas Density 0.3, Gas Saturated
checked [img-read: _rpclip0159.png]. These are dialog example values, **not** recommended defaults.

### 3.3 Fluid Substitution — Average Gassmann worked results (for regression testing)

Printed example A (`fluidsubstitution.htm`): Flushed/Reservoir zone — Brine ρ 1.008, K 2.375, V 5035;
Oil 0.675 / 0.589 / 3064; Gas 0.293 / 0.134 / 2223. Results: Vp 3563, Vs 1982, Vp/Vs 1.797, PR 0.276,
ρ_b 2.307, φ 0.179 / 0.184, Sw 0.332, Sxo 0.503, mixing exponent 3.000, fluid ρ 0.843 / K 0.8162 / V 984;
**Dry K 16.131, μ 9.066, PR 0.263, Modulus Ratio 1.779.**

Printed example B [img-read: _rpclip0017.png]: Fluid ρ 0.983, K 2.1586 GPa, V 4863 ft/s; φ 0.23 / 0.252;
Sw 0.832; Sxo 0.88; **Fluid Mixing Law Exponent 3**; **Reuss harmonic average checkbox OFF**;
**Dry K 5.885, μ 3.614, PR 0.245, Modulus Ratio 1.628.**

| Parameter | Default | Source |
|---|---|---|
| Fluid Mixing Law Exponent (Brie) | **3** | [img-read: _rpclip0017.png], (`fluidsubstitution.htm`) |
| Reuss harmonic fluid average | **OFF** | [img-read: _rpclip0017.png] |
| Invasion factor `IV`/`Inv` (WBM Sxo) | **1.0** | [img-read: _rpclip0016.png] |
| `SwMax` (OBM Sxo) | **0.5** | [img-read: _rpclip0016.png] |
| Dry-rock Poisson ratio guidance | 0.1–0.2 consolidated (avg 0.15); 0.1–0.25 unconsolidated | (`fluidsubstitution.htm`) |

### 3.4 Laminated Fluid Subs — rocks and Greenberg–Castagna coefficients

From `FluidSub_Default_Rocks.par`, Rocks and default parameters dialog [img-read: _rpclip0026.png]:

| | Dolomite | Feldspar | Limestone | Quartz | Wet Clay |
|---|---|---|---|---|---|
| Bulk K (GPa) | 69.4 | 37.5 | 65 | 37 | 21 |
| G/C `a` | 0 | 0 | −0.055 | 0 | 0 |
| G/C `b` | 0.583 | 0.804 | 1.017 | 0.804 | 0.77 |
| G/C `c` | −0.078 | −0.856 | −1.03 | −0.856 | −0.867 |

Higher-precision (5 dp) values from the G/C Coeffs sub-tab [img-read: _rpclip0029.png]:

| | `a` | `b` | `c` |
|---|---|---|---|
| Dolomite | 0 | 0.58321 | −0.07775 |
| Feldspar | 0 | 0.80416 | −0.85588 |
| Limestone | −0.05508 | 1.01677 | −1.03049 |
| Quartz | 0 | *(off-screen)* | *(off-screen)* |

**Greenberg–Castagna form** (quadratic in Vp, km/s): Vs = a·Vp² + b·Vp + c. Feldspar and Quartz carry
identical coefficients in this table — consistent with the page's statement that QFM may be lumped.

**Laminated Shale sub-tab** [img-read: _rpclip0027.png]:

| Parameter | Default | Units |
|---|---|---|
| Rho Lam Sh | **2.65** | g/cc |
| DTc Lam Sh | **110** | µs/ft |
| Lam Shale `a` | **0** | — |
| Lam Shale `b` | **0.76969** | — |
| Lam Shale `c` | **−0.86735** | — |

⚠ **Rho Lam Sh = 2.65 g/cc is the *grain* density of quartz and is a surprising default for a shale
lamination.** It is what the dialog prints; I have not substituted a textbook shale density. Flagged in §5.

**Rock Properties sub-tab** [img-read: _rpclip0028.png]: **Voigt Mix Fact 0.5** (Hill); Bulk Mod Dolomite
69.4, Feldspar 37.5, Limestone 65, Quartz 37 GPa.

**Fluid Properties tab** [img-read: _rpclip0025.png]: **Woods Mix Fact default 1**; the dialog states
**"Fluid Bulk Modulus must be entered in GPa"**. Modelled-reservoir example: Water ρ 0.9654 / K 2.2839;
Oil 0.7506 / 0.6913; Gas 0.0518 / 0.0002.

### 3.5 Laminated Fluid Subs — cutoffs, K_df model parameters, units

**Cutoffs / Limits tab** [img-read: _rpclip0030.png]:

| Parameter | Default | Meaning |
|---|---|---|
| **Phi Max** | **0.39** | maximum sand-lamination porosity |
| **Vsh Max** | **0.95** | maximum shale volume |
| **Phi Min** | **0.02** | minimum porosity |
| Kill Logic | Activate blank; Val/Crv1 0; Operator 0; Val/Crv2 0 | external suppression flag |

The prose confirms Kill Logic is additional to the cutoffs and uses the same logic as the Phi/Sw module
(`laminatedfluidsubs.htm`, l.141). Levels failing the cutoffs get **No_Com_Flg = 1**.

**K_df Model tab** [img-read: _rpclip0032.png] — zoned parameter:

| Parameter | Default | Units / note |
|---|---|---|
| Model | **Gassmann** | of {Gassmann, CPM, Krief, Soft Sand, Stiff Sand, Intermediate, Mixed, External} |
| Critical Porosity (φ_crit) | **0.39** | v/v — CPM |
| Krief X in X/(1−Phi) | **3** | the `m` of embim606 |
| Original Porosity (φ₀ / φ_D) | **0.39** | v/v — depositional sand-pack porosity, HM family |
| Coord. Number (C) | **9** | HM family (Intermediate model forces 15) |
| Effective Press (P) | **5000** | **PSI** |
| Adhesion Fraction (f) | **0.5** | HM family (Stiff Sand forces 1) |
| Dolomite Shear Mod | **51.6** | GPa |

**Unit conventions stated on the page** (`laminatedfluidsubs.htm`):

- Moduli in **GPa**; densities in project units, **g/cm³ or kg/m³**.
- **1 Mpsi = 0.145038 GPa**  *(strictly this is the GPa-per-ksi factor: 1 Mpsi = 6.895 GPa; see §5)*
- 1 foot = exactly **0.3048 m**; 1 m ≈ **3.2808 ft**.
- Elastic-formula block: moduli GPa, velocities ft/s, impedances g/cm³·km/s, density g/cm³, slowness µs/ft.
- **Convergence tolerance: 0.001 µs/ft** on sand-lamination DT shear [img-read: _rpclip0057.png].

**Diagnostic curves written** [img-read: _rpclip0036.png]: `tmp1_OtoW`…`tmp4_OtoW`, `KdfFlgOtoW`,
`KwFlgOtoW`, **`Count_OtoW`** (iteration count), `tmp3_WtoR`, `tmp4_WtoR`, `KwFlgWtoR`; laminated set
`Rho_osl, DTc_osl, DTs_osl, Kb_osl, Rho_wsl, DTc_wsl, DTs_wsl, Kb_wsl, G_wsl, Rho_swl, DTc_swl, DTs_swl,
Kb_swl`. Suffixes decode as o/w/s (observed / wet / substituted) + sl (sand lamination).

**Woods Factor sensitivity figure** [img-read: _rpclip0049.png]: legend K_Woods, K_Serial, Woodfac
0.25 / 0.5 / 0.75, Brie Exp 3; K axis 0–3 GPa vs Sw 0–1.

### 3.6 Laminated Sands (tensor) parameters

Per-zone values from the Laminated Sand tab [img-read: _pawsclip0121.png] (four zones):

| Parameter | Zone values | Units |
|---|---|---|
| Sat Model | Laminated | — |
| Rt Lam model | Tensor Rsh | (alt: Tensor Vlam) |
| Res Lam Shale | 0.686 / 0.809 / 0.745 / 0.809 | ohm.m (branch selector only) |
| Res Shale Hori | 0.619 / 0.598 / 0.555 / 0.543 | ohm.m |
| Res Shale Vert | 1.23 / 1.23 / 1.24 / 1.04 | ohm.m |
| RshVert / RshHori | 2.39 / 2.39 / 2.53 / 1.83 | — (shale anisotropy ratio) |

Butterfly crossplot example: Res Shale Hori **0.77**, Res Shale Vert **1.15** → anisotropy 1.49
[img-read: _pawsclip0098.png]. Thomas–Stieber crossplot example: **Phi max 0.3**, PhiTCl Calc **0.1239**,
PhiTShale Calc **0.08**, with Laminated / Dispersed / Structural shale-volume families plotted
[img-read: _pawsclip0094.png]. Log tracks include an **Anisotropy track (Rv_Rh, RshV_RshH) scaled 1–10**
and Sand Lam Res / Shale Lam Res tracks carrying ResHz, ResVert, ConvRv:RtLam, ConvRv:RxoLam, RshHori,
RshVert [img-read: _pawsclip0126.png].

**Observed shale anisotropy in the vendor example is Rv/Rh ≈ 1.8–2.5** — useful as a sanity band.

### 3.7 PVT parameters

Field-default example (field PLDEMO3) [img-read: _plclip00103.png]:

| Parameter | Value | Units |
|---|---|---|
| Gas gravity | **0.820** | Air = 1.0 |
| % N₂ | 0.00 | % |
| % CO₂ | 0.00 | % |
| H₂S | 0 | ppm |
| Surface rate basis | Stock Tank (60 °F) **or** Normal (0 °C) | — |
| Oil gas gravity | 0.820 | Air = 1.0 |
| Oil gravity | **29.00** | API |
| Solution GOR at bubble point | **135.0** | scf/stb |
| Bubble point temperature | **174.0** | deg F |
| Bubble point pressure | **638.0** | psia |
| Water salinity | **83000** | ppm NaCl equivalent |
| Gas free / gas saturated | toggle | — |
| "Old model Americium Sondex tool" | checkbox | — |

The page notes GOR-at-bubble-point, bubble-point T and bubble-point P **may or may not be used in the
equations** but are always printed on the Fluid Properties Report (l.112), and that the field defaults are
probably adequate for those three (l.114).

### 3.8 Formation Testing parameters

**FT Default Models** (Tool = Baker Atlas FMT, "Main Model") [img-read: _ftaclip00072.png]:

| Parameter | Default | Units |
|---|---|---|
| Volume | **10** | cm³ |
| **Shape Factor** | **0.668** | dimensionless (this is `C`, the flow coefficient) |
| Drawdown Viscosity | **2.5** | cP |
| Probe Radius | **2** | in |
| **Probe Radius Multiplier** | **3** | — |
| Spherical Viscosity | **2.7** | cP |
| Compressibility | **7E-08** | psi⁻¹ |
| Porosity | **0.2** | v/v |
| Radial Viscosity | **2.5** | cP |
| Thickness | **2.5** | ft |

**0.668 sits inside the documented 0.5–1.0 range for C** (`ft_equations_and_methodology.htm`) — the dialog
label "Shape Factor" and the nomenclature label "flow coefficient" refer to the same quantity.

**Output unit options** [img-read: _ftaclip00065.png and `ft_define_the_formation_testing_p.htm`]:

| Quantity | Options |
|---|---|
| Pressure | psi, psia, bar, Pa, KPa, MPa |
| Volume | L, mL, m³, cm³, bbl, gal (US), gal (imp), fl oz |
| Flowrate | cm³/s, l/s, l/min, US G/min, m³/s |
| Permeability | D, mD, m² |
| Viscosity | P, cP, Pa.S |
| Radius | in, mm |

**Drawdown options** (`ft_define_the_formation_testing_p.htm`):

- Multiple Drawdown Test as Single (toggle)
- **Flowrate Option for Permeability = Average or Maximum**
- Volume Value Source = Volume Curve / Flowrate Curve / Model Value
- Flowrate Value Source = Volume Curve / Flowrate Curve

**Output curves and units** [img-read: _ftaclip00065.png]: `Phyd_before` / `Phyd_after` (psi),
`Tdds` / `Tdde` / `Tdd` (sec), `Pfbu` / `Pflow` / `Pdd` (psi), `Vdd` (cm³), `Qdd` (cm³/s), `Kdd` (mD),
`Mdd` (units cell blank — see OPEN ITEMS), `Psph` / `Prad` (psi), `Ksph` / `Krad` (mD), `Msph` / `Mrad`,
`Pfinal` (psi), `FT_XplotFlag`, `FT_RegFlag`.

**Worked drawdown example** [img-read: _ftaclip00099.png]: Vol 10.000 cm³, Start 108.2 s, End 164.9 s,
Time 56.7 s, Rate 0.1764 cm³/s, Flowing P 33.889 psi, Final BU P 4861.114 psi, ΔP_dd 4827.225 psi,
Shape Factor 0.668, μ 2.500 cP, Probe Radius 2.00 in, Mult 3.00, **Effective Probe Radius 6.000 in**,
Drawdown Perm 0.009 mD, Mobility 0.004 mD/cP.

**Worked spherical example** [img-read: _ftaclip00089.png]: Rate 0.8988 cm³/s, Gradient −0.963,
μ 1.000 cP, c 10×10⁻⁶, φ 0.100, P* 3447.157 psi, k 17.722 mD, M 17.722 mD/cP.

**Worked radial example** [img-read: _ftaclip00093.png]: Rate 0.8988 cm³/s, Gradient −0.065, μ 1.000 cP,
P* 3447.158 psi, k 101.619 mD, M 101.619 mD/cP.

### 3.9 Pressure Differentiation parameters

[img-read: _plclip00171.png] and (`pressuredifferentiation.htm`):

| Parameter | Default / constraint | Units |
|---|---|---|
| Pressure curve | e.g. PPREP1R1 | psia (must match PL Set-Up) |
| Filter — number of intervals | **odd, 3 to 21 inclusive**; example 11; equal weights | samples |
| Deviation | curve (e.g. DEVI) **or** constant value — required | degrees |
| **Differentiation length** | **15.0 ft** (or **5 m**); 5 ft possible if pressure is stable | ft / m |
| Density output curve | e.g. DENDP1R1 | g/cc |
| Top / Bottom | example 9100.0 / 9320.0; full well range by default | ft |
| Interval constraint | must exceed the differentiation length | — |
| Edge loss | half the differentiation length at each end | — |
| PL Set-Up sensor type | **type 20** (primary density sensor) | — |

---

## 4. Assumptions & validity limits

**Fluid substitution (both modules)**

- Gassmann assumes a homogeneous, isotropic, monomineralic-equivalent frame with a fully connected pore
  space at low (seismic) frequency. **IP applies the frequency distinction explicitly**: seismic-frequency
  work uses the homogeneous Reuss fluid mix; log-frequency dry-frame inversion uses the empirical Brie mix
  (`fluidsubstitution.htm`).
- **Gassmann is declared inapplicable when K_formation > K_solid**, and the module records this as
  `No_Com_Flg = 2` rather than producing a number (`laminatedfluidsubs.htm`). This is a real guard SandiBumi
  should replicate.
- The dry-frame inversion (embim557–561) takes one quadratic root with no documented discriminant guard.
- Shear modulus is treated as fluid-independent (embim595), the standard Gassmann assumption.
- **The CPM implementation deliberately deviates from the CPM model**: shear should be zero at φ_crit but IP
  instead estimates shear from bulk modulus and G/C coefficients (`laminatedfluidsubs.htm`, l.583).
- The page describes CPM as **purely empirical with no claimed physical basis** (l.580).

**Laminated fluid substitution (Skelt) — validity envelope**

- **Not validated in carbonates; clastics only** (`laminatedfluidsubs.htm`). This is stated by the page,
  not inferred.
- The page states modelled results are often **more sensitive to the choice of shale distribution than to
  the fluid and rock property uncertainties** — a strong statement about where the error budget lives.
- The model rests on **Gassmann's contention that electrostatically bound water belongs to the rock frame
  while capillary-bound water belongs to the pore system** (`laminatedfluidsubs.htm`). This is the physical
  justification for the laminated partition and is a genuine modelling choice, not a universal truth.
- **Maximum five rocks.**
- Cutoff envelope: φ must lie between **0.02 and 0.39**, Vsh ≤ **0.95**; outside → No_Com_Flg = 1.
- Input Sw must be the petrophysicist's best estimate of true formation saturation, with **spurious low
  hydrocarbon saturations in water zones removed first** (l.671).
- Coordination-number guidance from the notes: published data (**Murphy 1982**) suggests C varies from about
  **8 to 14 as porosity reduces from 0.4 to 0.2**, and increases with clay content
  (`laminatedfluidsubs.htm`, l.721). A personal-communication attribution (**R. Beardsley, 2007**) also
  appears in the notes.
- Notes cite **Mavko, Mukerji & Dvorkin, *The Rock Physics Handbook*, 1998** (vs 1999 on the fluid
  substitution page and 1995 in a figure — §5(vii)).

**Laminated tensor-resistivity workflow**

- **The Poupon equation is explicitly excluded** (`laminated_sands_workflow.htm`, l.98).
- The tensor solution is **multi-valued**; the operator selects the branch via "Res Lam Shale". This means
  results are **not deterministic from the curves alone** — an operator judgement is baked in.
- The recommended QC is the agreement between VlamTensor and Thomas–Stieber Vlam.

**PVT**

- Correlation choice affects **oil only**; gas and water are computed identically regardless (l.84).
- Wet-gas/condensate model assumes gas and condensate flow as a **mixture at reservoir conditions**, and the
  condensate–gas ratio is held **constant across all reservoir zones** (l.133, l.149).
- Salinity must be **total NaCl-equivalent**, not chloride.
- Zonal P/T are sampled at zone tops only.

**Formation testing**

- Spherical and radial analyses both assume **infinite-acting homogeneous medium** and require that
  wellbore/system storage effects have died out before the straight-line fit
  (`ft_equations_and_methodology.htm`).
- The **derivative-plot flow-regime diagnosis is explicitly caveated** as borrowed from well-test analysis
  and possibly inappropriate at formation-tester volumes (`ft_create_and_analyse_a_formation.htm`).
- **Dual-rate equations are valid only for Schlumberger RFT**; every other tool must use single-rate.
- Radial permeability needs h, which the page concedes is *estimated* from open-hole logs — so k_r inherits
  that uncertainty linearly.
- Flow coefficient C is a **dimensionless tuning factor in 0.5–1.0** — not a derived constant.

**Pressure differentiation**

- Requires a smooth, high-resolution, temperature-insensitive pressure curve; older crystal gauges excluded.
- Requires deviation correction (curve or constant).
- Loses half the differentiation length at each end of the interval.

**Smectite / montmorillonite: NOT PRESENT.** I searched all nine pages, both the clean text and the raw
HTML, and read every mineral-selection dialog. The complete mineral vocabulary on my pages is
**Dolomite, Feldspar, Limestone, Mica, Quartz, Wet Clay** (fluid substitution) and
**Dolomite, Feldspar, Limestone, Quartz, Wet Clay + a single lumped "shale"** (laminated). There is **no
smectite or montmorillonite endpoint of any kind** — no modulus, no density, no G/C coefficients. Clay is
handled as one undifferentiated "Wet Clay" with K = 21 GPa, ρ = 2.60 g/cc. **For a Mahakam-delta smectitic
system this is a material gap in IP's rock-physics coverage, and a concrete differentiation opportunity for
SandiBumi.**

---

## 5. Internal discrepancies

Ten confirmed inconsistencies. Where the same raster exists in IP 2018 I checked it, so that "vendor
long-standing error" can be distinguished from "2025 regression". **All of the equation-level defects below
are present identically in IP 2018 — none is a new regression.**

**(i) embim525 / embim527 sign inconsistency.**
embim525 gives K_p = φ_ag / (1/K_d − 1/K_ma). Inverting for K_d should give
K_d = 1 / (φ_xp/K_p **+** 1/K_ma). embim527 prints a **minus**. Both operators pixel-verified.
One of the two is wrong; the pair cannot both be correct.
[img-read: embim525.png, embim527.png; pixel-verified]

**(ii) 304.8 printed where 304.8² is dimensionally required.**
embim576 prints K_b = ρ_b(304.8/Δt_c² − 4·304.8/(3Δt_s²)). Since v = 304.8/Δt, K_b = ρ(v_p² − 4v_s²/3)
requires **304.8²**. embim584 (Young's modulus) drops the factor entirely. By contrast **embim596 on the
same page prints the consistent-unit form with no 304.8 at all**. So the page contains two mutually
incompatible unit conventions for the same quantity, plus a factor-of-304.8 error in one of them.
[img-read: embim576.png, embim584.png, embim596.png; pixel-verified], [ip2018: embim389.gif — identical]

**(iii) embim590 K_fVoigt prints "+" where "×" is required.**
Printed: K_fVoigt = S_W×K_W + S_filtrate×K_filtrate + S_HC **+** K_HC. The first two terms are products;
the third must be S_HC × K_HC for a Voigt average. I ran the operator detector on all three operators:
the first two show the crossing-diagonal ×, the third shows a horizontal bar with a vertical stroke, i.e.
a genuine "+". This is a **typesetting error in the vendor manual**, present in 2018 as well.
[img-read: embim590.png; pixel-verified], [ip2018: embim403.gif — identical]

**(iv) embim607 G_HM printed without the ^(1/3) that K_HM carries.**
Hertz–Mindlin gives both K_HM and G_HM as cube roots of the bracketed pressure term. IP prints the
exponent on K_HM but not on G_HM. Identical in 2018.
[img-read: embim607.png], [ip2018: embim420.gif — identical]

**(v) The implemented effective probe radius matches neither documented method.**
The page documents two ways to get r_pe: Muskat r_pe = 0.5·r_p, and "Carlson and Jaeger" r_pe = 2r_p/π
(≈ 0.6366 r_p). Both **reduce** the probe radius. The dialog instead exposes a **Probe Radius Multiplier
with default 3**, and the worked example prints **Probe Radius 2.00 in × Mult 3.00 = Effective Probe Radius
6.000 in** — a 3× *increase*. Neither documented formula produces 6 in from 2 in. Furthermore the
nomenclature calls the divisor in _ftaclip0045 "r_s, snorkel radius", but the computation demonstrably uses
r_pe (my numeric reproduction of the printed mobility only works with 6.000 in, not 2 in).
**Consequence: mobility computed by IP with defaults is 3× smaller than the Muskat form would give and
~4.7× smaller than Carslaw–Jaeger — a factor-of-several difference in reported permeability.**
[img-read: _ftaclip0043.png, _ftaclip0044.png, _ftaclip0045.png, _ftaclip00072.png, _ftaclip00099.png]

**(vi) Parameter file name conflict.**
The prose names the parameter file **`FluidSub_Default_Parameters.par`**; the dialog callout names
**`FluidSub_Default_Rocks.par`**. (`laminatedfluidsubs.htm`) vs [img-read: _rpclip0026.png].

**(vii) Rock Physics Handbook cited with three different years.**
1995 (a figure caption), **1998** (laminated-page notes), **1999** (fluid-substitution page). The Handbook's
first edition is 1998. The 1995 and 1999 citations are inconsistent with each other and with 1998.

**(viii) Author-name misspellings that break literature traceability.**
- **"Carlson and Jaeger"** → Carslaw & Jaeger (*Conduction of Heat in Solids*). The r_pe = 2r_p/π result is
  theirs; the misspelling makes the source hard to trace.
- **"Krieff"** appears alongside **"Krief"**.
- **"Glasso"** (dialog) vs **"Glaso"** (prose) — Glasø.
- **"Vasquez Beggs"** → Vazquez & Beggs.

**(ix) `_rpclip0057` and `_rpclip0058` are the same image**, presented on the page as if they were two
different block diagrams.

**(x) v_p / v_s unit conflict within one equation block.**
embim577/578 define v_p and v_s in **km/s**. embim581/582/583/585 then multiply those same symbols by
**0.3048×10⁻³**, i.e. treat them as **ft/s**. Both cannot hold. The block header says "velocities ft/s",
which contradicts embim577/578.
[img-read: embim577.png, embim578.png, embim581–583.png, embim585.png]

**(xi) `1 Mpsi = 0.145038 GPa` is inverted/mis-scaled.**
0.145038 is psi-per-kPa (the *reciprocal* direction). Correctly, 1 GPa = 0.145038 Mpsi, i.e.
**1 Mpsi = 6.8948 GPa**. As printed the conversion is wrong by a factor of ~47.5. Reported as printed;
not corrected. (`laminatedfluidsubs.htm`)

**(xii) Rho Lam Sh default = 2.65 g/cc.**
The Laminated Shale sub-tab defaults the shale-lamination density to 2.65 g/cc — the quartz grain density.
Combined with DTc Lam Sh = 110 µs/ft this is an internally odd pairing for a shale. Reported as printed.
[img-read: _rpclip0027.png]

**(xiii) Radial worked example does not reproduce.**
k_r = 88.4·q·μ/(m·h) with q = 0.8988, μ = 1.000, m = 0.065, h = 2.5 gives **488.9 mD**, not the printed
**101.619 mD**. Solving for h gives h ≈ 12.0 ft. The h used in that example is not printed on the pane, so
either a different h was used or the printed gradient is rounded. Recorded as an OPEN ITEM rather than a
proven defect. [img-read: _ftaclip00093.png]

---

## 6. IP 2018 numeric diff

Method: compared `c25\<stem>_text.txt` against `c18\<stem>.htm` for all nine pages, and byte-compared every
equation raster that exists in both.

**Container change (cosmetic, affects nothing numerically):** equation images moved from **`.gif` at 1×** in
2018 to **`.png` at 2×** in 2025, and the `embim` numbering shifted by a constant offset of **187** on the
fluid-substitution/laminated pages (2025 `embim576` ↔ 2018 `embim389`; `embim590` ↔ `embim403`;
`embim607` ↔ `embim420`). The rendered equation text is unchanged in every case I checked.

**Formation testing:** all **13 equation rasters** on `ft_equations_and_methodology` are byte-identical
between 2018 and 2025 — the drawdown, spherical, radial and time-function equations, and every constant in
them (**C 0.5–1.0, 921, 8×10⁴, 1856, 88.4**), are **unchanged since at least 2018**. Only
`_ftaclip00101` and `_ftaclip00102` (screenshot images, not equations) differ.

**Real content changes 2018 → 2025:**

1. **Fluid Substitution moved in the menu tree**: 2018 documented it under *Advanced Interpretation*;
   2025 documents it under **GeoEng → Rock Physics**. The Laminated Fluid Subs page still says
   *Advanced Interpretation* in 2025 — so the two pages now disagree about the menu location of sibling
   modules in the same Rock Physics group.
2. **`formation_test_analysis.htm` gained a paragraph** in 2025 describing the **Pressure Gradients
   crossplot and the Excess Pressure plot output as a logplot**. This is genuinely new documentation.

Everything else differs only in breadcrumb/navigation boilerplate.

**Numeric verdict: no constant, coefficient, default or equation on my nine pages changed between IP 2018
and IP 2025.** Consequently, all four equation-level defects in §5 (ii)(iii)(iv) and the r_pe behaviour in
§5(v) are **long-standing vendor documentation errors, not 2025 regressions** — which raises the likelihood
that the *code* is right and the *manual* is wrong for (ii)(iii)(iv), and that the *manual* is right and the
*default* is questionable for (v).

---

## 7. SandiBumi notes

**Things to copy**

1. **The `No_Com_Flg` discipline.** IP writes a per-level integer flag distinguishing "ran" (0), "outside
   cutoffs" (1), and **"Gassmann inapplicable, K_formation > K_solid" (2)**, rather than emitting a number
   or an absent value. SandiBumi should adopt exactly this: a physics-violation flag is not the same as a
   missing input, and collapsing them loses the diagnosis.
2. **Iteration-count diagnostic curves.** `Count_OtoW` records how many iterations each level took to reach
   the 0.001 µs/ft shear-slowness tolerance. Shipping the convergence count as a curve makes non-convergence
   visible on the log instead of silent.
3. **The modulus-increment update (embim568).** Updating Vp by adding the *change* in bulk modulus rather
   than recomputing from the absolute moduli cancels part of the mineral-modulus error. Worth replicating.
4. **The frequency split.** IP consciously uses the Reuss/Wood homogeneous fluid mix for
   seismic-frequency outputs and the Brie empirical mix for the log-frequency dry-frame inversion.
   A single global mixing law would be a step backwards.
5. **The Woodfac / Voigtfac dials.** Two scalars (fluid-mix stiffening 0→1, solid-mix Voigt weight 0→1) give
   the user the entire Reuss↔Voigt continuum with sensible defaults (1 = Wood, 0.5 = Hill). Clean design.
6. **Explicit unit contracts in the dialog** ("Fluid Bulk Modulus must be entered in GPa"; "the units of the
   pressure curve need to be as defined in PL Set-Up"). Given SandiBumi's unit-canonicalisation discipline,
   these are the right places to assert.

**Things to beat**

1. **No smectite/montmorillonite anywhere.** IP's entire clay vocabulary on these pages is a single
   "Wet Clay" at K = 21 GPa, ρ = 2.60 g/cc, with one G/C triplet (0, 0.77, −0.867). For Mahakam-delta
   smectitic shales this is not adequate. SandiBumi carrying real, cited smectite/illite/kaolinite/chlorite
   elastic endpoints would be a genuine capability gap closed — but **the endpoints must come from a cited
   source (Jauhar's chartbook, a reference-suite export, or a named study), not from me.** Flagged as a
   research gap, not filled.
2. **The laminated model is a scalar N/G partition, not an anisotropic formulation.** Skelt's method
   computes fluid effects in the sand fraction and scales them by (1 − V_shalelams). It never forms a
   stiffness tensor, never uses Backus averaging, and produces no anisotropy parameters. Meanwhile the
   *resistivity* side of IP (`laminated_sands_workflow`) is fully tensorised with Rv/Rh. **The two halves of
   IP's thin-bed treatment are inconsistent in sophistication** — the resistivity side knows about
   anisotropy and the elastic side does not. A SandiBumi thin-bed module that used a consistent anisotropic
   framework on both sides (Backus for elastics, tensor for resistivity, sharing one lamination fraction)
   would be a real advance. This connects directly to the existing Rh/Rv route work
   (`method_thinbed_rhrv_routes`).
3. **The tensor branch selection is an operator judgement.** "Res Lam Shale" picks which root of the tensor
   solution is used, and the page says its absolute value does not matter. That is an unreproducible,
   un-auditable step in a deliverable workflow. SandiBumi should either select the branch deterministically
   (e.g. by continuity with the adjacent level, or by a physical constraint such as Rsand ≥ Rshale in a
   hydrocarbon-bearing sand) or record the branch choice as a stored, versioned parameter.
4. **The r_pe question (§5 v) is a live numerical hazard.** If SandiBumi ever cross-checks mobility against
   IP output, expect a factor of 3 (vs Muskat) or ~4.7 (vs Carslaw–Jaeger). Whichever convention SandiBumi
   picks must be *stated on the output*, not buried.
5. **No excess-pressure/supercharging model is documented at all.** IP offers an Excess Pressure *plot* but
   no correction. Supercharging correction is a genuine unmet need in low-mobility formation testing.
6. **PVT correlations are name-only.** Standing / Vazquez-Beggs / Glasø / Lasater are offered with **zero
   printed coefficients**. If SandiBumi implements these, the coefficients must come from the original
   papers with citations — IP's manual provides no traceable source, so IP cannot be used as the provenance.

**Regression fixtures available.** The worked examples in §3.3, §3.8 give five independently reproducible
number sets (two Average Gassmann result panes, one drawdown, one spherical, one radial). I have already
verified the drawdown mobility (0.00375 vs printed 0.004) and spherical permeability (17.726 vs printed
17.722) reproduce from the printed equations. These make good acceptance tests for a SandiBumi
implementation.

---

## 8. OPEN ITEMS

Facts I could not establish with certainty. **None of these has been filled from textbook knowledge.**

1. **Quartz Greenberg–Castagna `b` and `c` at 5-dp precision** were off-screen in the G/C Coeffs sub-tab
   [img-read: _rpclip0029.png]. Only the 3-dp values 0.804 / −0.856 from the Rocks table
   [img-read: _rpclip0026.png] are captured. (They match Feldspar exactly at 3 dp; whether they match at
   5 dp is unverified.)
2. **Wet Clay G/C `a`** is partly obscured by a dialog callout in [img-read: _rpclip0026.png]. I read it as
   0, consistent with Dolomite/Feldspar/Quartz, but the glyph is not fully legible.
3. **Shear moduli for minerals other than Dolomite (51.6 GPa)** were scrolled off the K_df Model tab
   [img-read: _rpclip0032.png]. The Hertz–Mindlin family needs G_R per mineral; only Dolomite's is visible.
4. **`Mdd` output-curve units cell is blank** in [img-read: _ftaclip00065.png], although the analysis pane
   prints mD/cP [img-read: _ftaclip00099.png]. Cannot confirm whether the blank is a documentation omission
   or a genuinely unit-less internal curve.
5. **`d(Int)` in the derivative definition (embim392) is not defined anywhere on the page.** Almost
   certainly the derivative is taken with respect to the relevant time function (superposition/Horner), but
   the manual does not say so. Not guessed.
6. **No printed coefficients for Standing, Vazquez-Beggs, Glasø or Lasater.** Only the names and the two
   oil-density formulas. Also **no Z-factor equation** despite N₂/CO₂/H₂S being stated to modify it, and
   **no water-property or Bw correlation** is named at all.
7. **No excess-pressure / supercharging equation on my pages.** The 2025 addition to
   `formation_test_analysis.htm` says the Excess Pressure plot may be output as a logplot and
   cross-references the **Crossplot** documentation. → **Hand-off: whichever agent holds the Crossplot /
   cross-plot pages should look for the excess-pressure definition there.**
8. **No printed gradient→fluid-density conversion.** The pressure-gradient crossplot regresses
   `Pressure = Depth × slope + intercept` in psi/ft, and the example slopes (0.120, 0.370, 0.434 psi/ft) are
   consistent with gas/oil/water, but **the psi/ft → g/cc constant is never printed** on my pages. Likewise
   `pressuredifferentiation.htm` gives no conversion constant for dP/dz → density. → **Hand-off: may live
   on a PL Set-Up or Multiphase Flow Calculations page held by another agent.**
9. **`Rt Lam Sand` / `Rxo Lam Sand` columns were cut off** in the Laminated Sand parameter dialog
   [img-read: _pawsclip0121.png].
10. **The radial worked example does not reproduce** with h = 2.5 ft (§5 xiii); h is not printed on that
    pane. Either h ≈ 12 ft was used, or the printed gradient is rounded, or the equation is applied
    differently. Unresolved.
11. **Which of embim525 / embim527 is the correct sign** cannot be determined from the manual (§5 i). Both
    are pixel-verified as printed; the manual never states the intent.
12. **Whether IP's *code* squares the 304.8 in K_b** (§5 ii) cannot be determined from documentation alone.
    The 2018/2025 identity says the manual has been wrong for at least seven years; a numeric test against
    IP output would settle it.
13. **"Original Porosity" (φ₀) and "Critical Porosity" (φ_crit) both default to 0.39** and both appear on
    the K_df Model tab. Whether they are the same underlying parameter surfaced twice or two independent
    ones with a coincident default is not stated.
14. **The Skelt papers themselves (TLE May 2004; SPWLA 45th 2004) were not consulted** — they are cited by
    IP but not included in the CHM. Several of the laminated model's assumptions (particularly the
    bound-water partition) are stated without derivation and would need the source papers to verify.
