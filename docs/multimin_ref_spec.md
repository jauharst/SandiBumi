# the reference suite Multimin — extracted specification (from the reference install helpset, 2026-07-19)

Research basis for SandiBumi generalized multimin (multimin2) the reference suite-parity work.
Sources: `C:\Program Files\AspenTech\the reference install\doc\helpset\{MultiminModel, RF04_Multimin, PT07_Multimin, MultiminAnalysis, RF03_LogsMods}`.

## A. Flushed (X) / Unflushed (U) zone model

- One unknown vector v per depth frame: mineral volumes (single set, common to both zones) + TWO parallel fluid sets:
  X-zone (flushed): X Oil, X Gas, X OBM Filtrate, X BndW, X Irred Water, X FreeW, X Special Fluid, X Isolated, X Parallel.
  U-zone (unflushed): U Oil, U Gas, U BndW, U Irred Water, U FreeW, U Special Fluid, U Isolated, U Parallel.
- Limits: max 30 volumes, max 50 equations. nvol <= nequations + ntool_constraints.
- Zone assignment: only deep resistivity (CT) sees the U zone; ALL other tools respond to X-zone fluids. CXO sees X-zone fluids.
- Default response example (Quartz/Illite/Kaolinite + X Oil/X BndW/X FreeW + U Oil/U BndW/U FreeW):
  - RHO_COR: 2.65, 2.78, 2.62, 0.6341, 1.049, 1.049, 0,0,0 (G/C3)
  - TNPH_COR: -0.05, 0.247, 0.451, 0.9807, 0.9529, 0.9529, 0,0,0 (V/V)
  - DT: 50.4, 85.34, 85.34, 189, 189, 189, 0,0,0 (US/F)
  - U (PEF*rhoe): 5.04, 11.12, 5.38, 0.09142, 0.7806, 0.7806, 0,0,0 (B/C3)
  - GR_COR: 1, 160, 104, 0, 0, 9.6(KCl mud), 0,0,0 (GAPI)
  - CT: 0,0,0, 0,0,0, 0, 18.51, 4.298 (MH/M)
  - CXO: 0,0,0, 0, 18.51, 22.52, 0,0,0 (MH/M)
- General linear form: t_k = sum_i P[k][i] * v_i (X components; deep conductivity uses U fluids).
- X-zone Geometric Factor (optional): PRED = sum(min responses) + X*sum(X-fluid resp) + (1-X)*sum(U-fluid resp); X in [0,1]; default 1 for all logs, 0 for CT (fixed), 1 for CXO (fixed).

### Derived outputs (exact)
```
VOL_UWAT  = U irred + U free (+special/iso/par)
VOL_XWAT  = X irred + X free (+special/iso/par)
PHIE_X = VOL_XWAT + VOL_XOIL + VOL_XGAS + VOL_OBMFILT
PHIE   = VOL_UWAT + VOL_UOIL + VOL_UGAS
PHIT_X = PHIE_X + VOL_XBNDWAT ; PHIT = PHIE + VOL_UBNDWAT
SWE  = VOL_UWAT/PHIE ; SWT = (VOL_UWAT+VOL_UBNDWAT)/PHIT
SXOE = VOL_XWAT/PHIE_X ; SXOT = (VOL_XWAT+VOL_XBNDWAT)/PHIT_X
VOL_MOVEDHC = (VOL_UOIL+VOL_UGAS) - (VOL_XOIL+VOL_XGAS)
VOL_WETCLAY = sum(dry clay) + VOL_UBNDWAT
```
Predicted logs: CT_PRED/CXO_PRED, RT_REC = 1000/CT_PRED etc., R0 (Sw=1 deep resistivity).

## B. Volume constraints table

Row semantics: sum_i(coef_i * v_i) <Type> Result.
Types:
- `==` hard equality, no DOF added.
- `Tool` = BOTH a hard equality AND a pseudo-measurement with uncertainty 0.01 (enters incoherence). Only Tool rows add a degree of freedom.
- `>=` / `<=` hard inequalities, no uncertainty, no DOF.

Program constraints (auto):
| Row | Meaning |
|---|---|
| UNITY | sum(minerals) + sum(U fluids) = 1 (X fluids excluded). Always. Tool. U=0.01 |
| POROSITY | sum(X fluids) - sum(U fluids) = 0. Always. Tool. U=0.01 |
| IRR WATER | X irred = U irred (when present). Tool. |
| X BNDWAT | sum_clays(k_clay * v_dryclay) - VOL_XBNDW = 0. Tool. (DW model + clays) |
| U BNDWAT | same with VOL_UBNDW. Tool. |
| WATER MUD | sum(X waters) - sum(U waters) >= 0 (WBM with non-water X phase). Inequality. |
| OIL MUD | mirror <= (OBM). OIL MUD GAS: X gas <= U gas (OBM). |

Bound-water clay coefficient (exact):
```
k_clay = alpha * 0.096 * CEC_clay[meq/g] * rho_clay[g/cc scaled] / (T_degC + 298)
alpha = sqrt(0.35 mol/L / n) if salinity n < 20455 ppm NaCl else 1   (expansion ON)
```
Verified: Illite CEC=0.25, rho=2.78, T=64.4C -> 0.1841; Kaolinite CEC=0.1, rho=2.62 -> 0.0694.
Also k = WCLP/(1-WCLP) (Illite WCLP 0.1555, Kaolinite 0.06489).
X row uses alpha from RMF salinity, U row from RW salinity.

User constraints: up to 5, any linear combination, Type any, coefficients V or L.

## C. Volume properties

- Grain Density (always, G/C3, dry) — synced bidirectionally with Core Grain Density response.
- CEC [meq/g] (Dual Water / Waxman-Smits; only clays + special minerals).
- Wet Clay Porosity [V/V] — alternative parameterization.
- Porosity Source radio: CEC entered -> WCLP computed, or vice versa.
- WCLP tie: WCLP = (RHOG - RHOB)/(RHOG - RHOB_UFREW); RHOG = (RHOB - WCLP*RHOB_UFREW)/(1-WCLP).
- Non-DW models (Juhasz/Indonesia/Simandoux/NormDW): CEC row -> "Resistivity" (wet clay resistivity -> CT/CXO response of that clay), WCLP -> Wet Cl/Sh Porosity + Bound Water Fraction (applied AFTER minimization).

## D. Volume bounds

Hard box constraints, always honored exactly. Defaults: all 0..1; every fluid (X and U) upper bound 0.5.

## E. Methods tab

Columns: Method | Log | Uncertainty | M (V/L; CT/CXO also I) | Units | Active Log | Output Wet.
- Neutron methods: LINEAR MATRIX (default), Classic Matrix, per-vendor nonlinear fits phi_T = a + b*phiN + 10^(c + d*phiN).
- Sonic: WYLLIE LINEAR / RHG NONLINEAR.
- GR: LINEAR / MASSIC (GR = sum(rho_i*v_i*GR_i)/RHO_pred — density weighted).
- CT & CXO (same method both): DUAL WATER LINEAR/NONLINEAR, ARCHIE LINEAR/NONLINEAR, WAXMAN-SMITS, NORMALIZED DW, JUHASZ, INDONESIA, SIMANDOUX (all NONLINEAR variants).
- Uncertainty M: V constant (default ~1.5% of normal range), L log, I (CT/CXO only) computed: U_Cxo = 0.03*Cmf^(1/w), U_Ct = 0.03*Cfw^(1/w).
- Weight in incoherence = 1/U^2.
- Active Log = No -> excluded from minimization, predicted anyway -> synthetic log <LOG>_SYNTH.
- Output Wet -> <log>_WET synthetic with HC replaced by free water.
- Neutron excavation-effect correction checkbox (Segesman & Liu 1971), default ON with neutron.

Default uncertainties (RF04 6.9): Unity 0.01, EquivFluids 0.01, RHOB 26.4 kg/m3 (0.0264 g/cc), NPHI 0.014 v/v, DT 6.4 us/m (1.951 us/ft), U 0.32 b/cc, GR 6 API, Th 0.5 ppm, K 0.2%, Ur 1.0 ppm, Vp/Vs 0.11 km/s, CT/CXO calculated, EPT 0.6 ns/m, EATT 50 dB/m, Sigma 1.10 cu, user-defined 0.015, core poro 0.005, core grain density 0.010 g/cc, XRD 0.050 w/w.

### Dual-water saturation parameters
- m (cementation), n (saturation), w = 0.75*m + 0.25*n (internal, linearized pass).
- CBW conductivity Method Model: Cbw = 0.0007*(T_C + 8.5)*(T_C + 298); Cxbw = Cbw/alpha_x, Cubw = Cbw/alpha_u.
- DW response equations (exact):
```
Nonlinear: Ct = phiT^(m-n) * (vubw+vufw)^(n-1) * (vubw*Cubw + vufw*Cufw)
Linear:    Ct^(1/w) = vubw*Cubw^(1/w) + vufw*Cufw^(1/w)
```
(phiT = sum U-zone fluids; same form for CXO with X volumes.)
- Nonlinear runs: solve w-linearized model first, then full m,n nonlinear engine.
- Pyrite correction option (freq-dependent, U zone 35 Hz, X zone 1000 Hz, valid to ~7% pyrite).

## F. Fluid properties tab

Inputs: Mud Type WATER/OIL, KCl% of mud, Formation Temp (148 F default ex.), Formation Pressure (3600 psi),
RW sample + temp (0.43 ohmm @ 77F -> 13048 ppm NaCl, 0.9948 g/cc), RMF sample + temp (0.1 ohmm @ 62F -> 84646 ppm, 1.0410 g/cc),
Oil API (36.8 -> 0.8408 g/cc stock tank), Gas SG (0.685 -> 0.2022 g/cc reservoir), Condition Number Cutoff (linear, default 10).
Calculate Fluid Responses = Yes: computes bound-water constraint multipliers, CT/CXO I-uncertainties, all fluid response cells
(RMF -> Cmf -> X FreeW/X BndW conductivity + nuclear responses; RW -> Cwf -> U fluids; API/GasSG/T/P -> HC density/HI/Pe; KCl% -> GR of filtrate).
= No: cells become user-editable (orange).

## G. Extra output logs

1. Linear Logs: user equations, result = sum(mult_i * VOL_i) (Mode Volume) or over weights (Mode Weight); output name + units.
2. Synthetic Logs: from Active=No rows (<LOG>_SYNTH) and Output Wet (<log>_WET).
3. Normalized Mineral Volumes: VOL_<MIN>_NORM = VOL_MIN / sum(non-fluid volumes); per-mineral Yes/No, default No.

## H. Solver

Objective (incoherence): Delta^2 = sum_k [ (t_k - f_k(v))/U_k ]^2, including Tool-constraint pseudo-measurements.
Linear: convex QP  min (t-Pv)' U (t-Pv)  s.t. hard box bounds + hard linear eq/ineq constraints.
Engine: dual QP of Goldfarb & Idnani (1983) / Powell (1985). Nonlinear: linearize -> linear solve for start, then SQP (Powell VF02 + watchdog).
Verification: DOF check + conflict check; condition number = log10(SVD norm ratio) of A = P'UP; >8 suspect, >10 unstable (linear cutoff default 10).
Per-frame quality logs: CONDNUM; QUALITY = sqrt(Delta^2 / chi2_95%(ntool-3)) (<1 good); NFUN (iteration count).
Predicted per-volume errors = sqrt(diag(A^-1)) * QUALITY.

## I. Default wireline response endpoints (RF04 6.2) — the reference suite values

Internal units metric: RHOB kg/m3, DT us/m, NPHI frac, U barns/cc, GR API, Th/U ppm, K %, Sigma c.u., EPT ns/m.
Clays are DRY-clay responses. `*` = computed at runtime from fluid properties. U-zone fluids: all responses 0 except DT.

| Component | RHOB | NPHI | DT us/m | U(PEF) | GR | Th | K% | Ur | Sigma | EPT | CEC meq/g | CoreRhoG |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Quartz | 2650 | -0.050 | 165.35 | 5.04 | 1.0 | 0 | 0 | 0.10 | 4.71 | 7.2 | 0 | 2650 |
| Silt | 2650 | -0.050 | 165.35 | 5.04 | 1.0 | 0 | 0 | 0.10 | 4.71 | 7.2 | 0 | 2650 |
| Calcite | 2710 | 0.000 | 156.8 | 14.13 | 11.0 | 0 | 0 | 1.40 | 7.44 | 9.7 | 0 | 2710 |
| Dolomite | 2847 | 0.025 | 142.7 | 9.6 | 8.0 | 0 | 0 | 0.90 | 6.92 | 8.7 | 0 | 2870 |
| Halite | 2037 | -0.018 | 219.8 | 9.63 | 5.0 | 0.2 | 0 | 0 | 750 | 8.2 | 0 | 2170 |
| Orthoclase K | 2570 | -0.006 | 175.5 | 8.71 | 171 | 1.10 | 10.21 | 0.40 | 15.34 | 7.6 | 0 | 2570 |
| Anorthite Ca | 2760 | -0.01 | 147.6 | 8.57 | 8 | 0 | 0 | 0.1 | 9.36 | 7.6 | 0 | 2760 |
| Albite Na | 2610 | -0.005 | 180.8 | 5.57 | 8.0 | 0 | 0.50 | 0 | 11.40 | 7.7 | 0 | 2620 |
| Anhydrite | 2977 | -0.02 | 164.0 | 14.95 | 5.0 | 0.2 | 0 | 0.4 | 16.0 | 8.4 | 0 | 2960 |
| Ankerite | 3080 | 0.050 | 150.0 | 25.80 | 8.0 | 0 | 0 | 0 | 22 | 0 | 0 | 3080 |
| Gypsum | 2350 | 0.576 | 172.3 | 9.46 | 5.0 | 0 | 0 | 0 | 20.0 | 6.8 | 0 | 2320 |
| Pyrite | 4987 | -0.019 | 123.4 | 82.22 | 5.0 | 0 | 0 | 0 | 90.0 | 0 | 0 | 5000 |
| Galena | 6390 | -0.03 | 128.6 | 10000 | 10 | 0 | 0 | 0 | 13.13 | 7.2 | 0 | 6390 |
| Rutile | 4120 | 0.09 | 108.6 | 40.60 | 10 | 0 | 0 | 0 | 194.2 | 7.2 | 0 | 4120 |
| Siderite | 3960 | 0.184 | 143.7 | 72.2 | 6.0 | 0.40 | 0 | 0.50 | 54.21 | 9.0 | 0 | 3960 |
| Muscovite | 2840 | 0.21 | 160.76 | 11.43 | 130 | 0 | 7.80 | 0.70 | 95.27 | 8.9 | 0 | 2810 |
| Biotite | 3220 | 0.11 | 162.1 | 22.42 | 127 | 1.50 | 7.20 | 0.70 | 54.10 | 7.7 | 0 | 3020 |
| Glauconite (dry) | 2850 | 0.51 | 162.1 | 19.10 | 150 | 3.0 | 5.94 | 5.40 | 89.60 | 8.0 | 0.20 | 2960 |
| Illite (dry) | 2780 | 0.247 | 280.0 | 11.12 | 160 | 12.3 | 4.48 | 4.80 | 40.56 | 8.0 | 0.25 | 2780 |
| Kaolinite (dry) | 2620 | 0.451 | 280.0 | 5.38 | 104 | 19.3 | 0.08 | 3.20 | 20.12 | 8.0 | 0.10 | 2620 |
| Chlorite Mg (dry) | 2670 | 0.44 | 280.0 | 16.82 | 56 | 6.90 | 0.42 | 2.90 | 11.42 | 8.0 | 0.15 | 2670 |
| Chlorite Fe (dry) | 3420 | 0.5 | 280.0 | 21.55 | 56 | 6.9 | 0.42 | 2.9 | 43.72 | 8.0 | 0.15 | 3420 |
| Smectite (dry) | 2630 | 0.218 | 280.0 | 7.61 | 168 | 26.0 | 0.58 | 7.10 | 20.22 | 8.0 | 1.00 | 2630 |
| Anthracite | 1520 | 0.38 | 344.5 | 0.26 | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 1520 |
| Lignite | 1220 | 0.52 | 524.9 | 0.26 | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 1220 |
| Kerogen | 1200 | 0.5 | 420 | 0.26 | 100 | 0 | 0 | 10 | 0 | 0 | 0 | 1200 |
| Heavy Mineral | 4510 | 0.01 | 314.3 | 1070 | 40000 | 9365 | 0 | 1338 | 0 | 7.2 | 0 | 4510 |
| X Oil | * | * | 620.1 | * | 0 | 0 | 0 | 0 | 21.0 | 5.0 | | |
| X Gas | * | * | 820.2 | * | 0 | 0 | 0 | 0 | 5.0 | 3.3 | | |
| X Bnd/Irr/Free Water | * | * | 620.1 | * | 0 (FreeW GR/K from KCl mud) | | | | 50.0 | * | | |
| X Special Fluid (barite) | 4084 | -0.002 | 620.1 | 1065 | 0 | 0 | 0 | 0 | 19.9 | 0 | | |
| U fluids (all) | 0 | 0 | 620.1 (gas 820.2) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | | |

DT us/ft conversion: /3.2808. E.g. Quartz 50.4, Calcite 47.8, Dolomite 43.5, Illite 85.34, water 189, oil 189, gas 250.

## J. Fluid property correlations (RF04 5.09-5.16)

- Density chain per fluid: reservoir rho_r -> electron rho_e -> tool-apparent rho_a = 1.0704*rho_e - 188.3 (kg/m3).
- Oil (Vasquez-Beggs): Rs = C1*gg*P^C2*exp(C3*go/(T+C4)); Bo; rho_roil = (C1*go + C2*gg*Rs)/Bo; rho_eoil = 1.1406*rho_roil.
- Gas (Dranchuk/Standing-Katz Z): rho_rgas = C1*gg*P/(Z*(C2+T)); rho_egas = f(gg)*rho_rgas.
- Water: S = 10^((3.562 - log10(R75 - 0.0123))/0.955) ppm; rho_rwat = 1000*(1+7e-7*S)/Bw; rho_ewat = 1.11*rho_rwat.
- HI: oil 0.009*rho_roil*(4-0.0025*rho_roil)/(16-0.0025*rho_roil); gas 2.2e-3*rho_rgas; water rho_rwat*(1-1e-6*S)/1000.
- U: oil 0.000119*rho_eoil.
- CBW conductivity: Cbw = 0.0007*(T_C+8.5)*(T_C+298).
- KCl mud: GR_xfw = 4.8*KCL_MUD wt%; K_xfw = 0.524*KCL_MUD.
- Arps: conductivity temperature conversion via (T+21.5) ratios (the reference suite standard).

## K. Output-log suite (tp_multimin)

VOL_<comp> per component; aggregates PHIE/PHIT/PHIE_X/PHIT_X, SWE/SWT/SXOE/SXOT/SWT_BND/SXOT_BND,
VOL_MOVEDHC, VOL_WETCLAY, VOL_UWAT/VOL_XWAT; predicted log per equation + CT_PRED/CXO_PRED/RT_REC/RXO_REC/R0;
QUALITY, CONDNUM, NFUN; lithology (dominant mineral); optional per-volume uncertainty logs; synthetic <LOG>_SYNTH,
wet <log>_WET; normalized VOL_<min>_NORM; extra saturations SO_XT/SG_XT/SO_UT/SG_UT etc.

## L. Solver notes for reimplementation (from PT07/RF04 deep pass)

- The DUAL WATER LINEAR trick: with w = 0.75m + 0.25n, the CT row becomes LINEAR in volumes:
  `Ct^(1/w) = v_ubw*Cubw^(1/w) + v_ufw*Cufw^(1/w)` — transform the measured conductivity and the fluid
  conductivity endpoints by ^(1/w) before assembling the row; minerals/HC get 0. Same for CXO with X volumes/Cmf.
  the reference suite's NONLINEAR mode = linear solve first, then SQP refinement (Powell VF02 + watchdog) — linear is a
  supported production method on its own.
- CT/CXO row uncertainty (already in transformed units): U_Cxo = 0.03*Cmf^(1/w), U_Ct = 0.03*Cfw^(1/w).
- QUALITY = sqrt(Delta^2 / chi2_95(ntool-3)); < 1 good. CONDNUM = log10(cond(A)) via SVD, A = P'UP.
- Model switching: primary model + up to 10 secondary (expression, model) pairs; reserved SKIP/NONE/IGNORE
  (IGNORE = bad-hole abandon). Probabilistic mode: weights W_i = P_i * prod(1-W_j), normalized.
- Archie linear = same w-transform with Cbw := Cfw. Waxman-Smits/Indonesia/Simandoux/Juhasz: nonlinear only.
- Uncertainty philosophy: default = ~1.5% of the tool's normal logged range; weight = 1/U^2.

## UI color semantics (for cloning)
gray = derived/not editable; orange = editable override; yellow = CBW-sourced CT/CXO cells; red = missing mandatory response; white = editable.
"-" in a cell restores the default. V/L method columns hidden behind a "Volume Method Columns" checkbox.
