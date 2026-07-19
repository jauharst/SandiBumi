# IP2018 Mineral Solver — extracted specification (2026-07-19)

Research basis for SandiBumi generalized multimin. Sources: decompiled `C:\Program Files\IP2018\Interact.chm`
(mineralsolver / minsolveeqandmeth / minsolvecalibrate chapters) + live default files
`C:\Program Files\IP2018\MINDEF.PAR` (mineral endpoints) and `MINEQDEF.PAR` (equation defaults).
Jauhar's mineral-list screenshot = IP's MINDEF.PAR mineral dropdown, in file order.

## A. Default endpoints (MINDEF.PAR, verbatim)

Units: Density g/cc, Sonic us/ft, U barns/cc, EPT_TPL ns/m, Sigma c.u., Pota %, Thor/Uran ppm.
"Auto" = computed at runtime (neutron matrix lookup; fluid props from Rw/Rmf/T/P).

### Matrix minerals
| Mineral | Density | Neutron | Sonic | U | EPT | SIGMA | K% | Th | Ur |
|---|---|---|---|---|---|---|---|---|---|
| Calcite | 2.71 | Auto | 47 | 13.8 | 9.1 | 7.4 | 0 | 0 | 1.4 |
| Quartz | 2.65 | Auto | 55 | 4.8 | 7.2 | 4.7 | 0 | 0 | 0.1 |
| Dolomite | 2.85 | Auto | 42 | 9.0 | 8.7 | 6.92 | 0 | 0.1 | 0.9 |
| Orthoclase | 2.57 | -0.01 | 69 | 8.7 | 7.6 | 15.3 | 10.2 | 1.1 | 0.4 |
| Albite | 2.60 | -0.001 | 49 | 5.6 | 7.6 | 11.4 | 0.5 | 0 | 0 |
| Anhydrite | 2.98 | -0.02 | 50 | 14.95 | 8.4 | 11.1 | 0 | 0 | 0.4 |
| Halite | 2.04 | -0.03 | 67 | 9.7 | 8.2 | 750 | 0 | 0 | 0 |
| Gypsum | 2.35 | 0.54 | 52 | 9.46 | 6.8 | 20.0 | 0 | 0 | 0.3 |
| Pyrite | 4.99 | 0.01 | 39.2 | 82 | - | 90.0 | 0 | 0 | 0 |
| Siderite | 3.88 | 0.18 | 47 | 72 | 8.9 | 54.2 | 0 | 0.4 | 0.5 |
| Muscovite | 2.85 | 0.24 | 49 | 11.5 | 8.9 | 95.3 | 7.8 | 0 | 0.7 |
| Biotite | 3.04 | 0.13 | 50.8 | 21.6 | 7.8 | 54.1 | 7.2 | 1.5 | 0.7 |
| Glauconite | 2.96 | 0.41 | - | 16.5 | 12.0 | 90.0 | 5.6 | 2.8 | 5.1 | (Qv 0.57, PhiTClay 0.156, BW 0.185) |
| Coal | 1.19 | 0.52 | 160 | 0.24 | - | - | 0 | 0 | 0 |
| Kerogen | 1.10 | 0.60 | 150 | 0.264 | - | - | - | - | - |

### Clays (Wet_Clay type) — with Qv (meq/cm3 wet clay), PhiTClay, BoundWater
| Clay | Density | Neutron | Sonic | U | EPT | SIGMA | K% | Th | Ur | Qv | PhiTClay | BW |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Kaolinite | 2.55 | 0.51 | - | 5.1 | 11.0 | 21.9 | 0.1 | 18.9 | 3.1 | 0.22 | 0.058 | 0.062 |
| Chlorite | 2.81 | 0.58 | - | 21.7 | 11.0 | 43.7 | 0.67 | 11.0 | 3.5 | 0.38 | 0.101 | 0.112 |
| Illite | 2.61 | 0.35 | - | 9.9 | 14.0 | 41.0 | 4.32 | 11.8 | 4.6 | 0.59 | 0.156 | 0.185 |
| Montmorill | 2.02 | 0.65 | - | 4.4 | 16.0 | 22.0 | 0.5 | 20.6 | 5.6 | 1.60 | 0.425 | 0.739 |
| Clay (generic) | 2.65 | 0.35 | 100 | 10.0 | 8.0 | - | 2.0 | 6.0 | 12 | 0 | 0 | 0.15 |

Qv_param = CEC(meq/g) * DryClayDensity * (1 - WetClayTotalPorosity).

### Fluids (zone-typed: _Sxo = flushed, _Sw = unflushed)
| Fluid | Type | Density | Neutron | Sonic | U | EPT |
|---|---|---|---|---|---|---|
| Water_Sxo | Water_Sxo | Auto | Auto | 189 | Auto | Auto |
| Water_Sw | Water_Sw | Auto | Auto | 189 | Auto | Auto |
| BoundWater | Bound_Water | 1.0 | 1.0 | 189 | 0.39 | 30 |
| Oil_Sxo / Oil_Sw | Hyd._Sxo/_Sw | 0.8 | 0.8 | 200 | 0.8 | 5.0 |
| Gas_Sxo / Gas_Sw | Hyd._Sxo/_Sw | 0.2 | 0.2 | 220 | 0.8 | 3.3 |

HC density/neutron = true downhole values; IP converts to electron density / HI internally:
DenHcApp = 2*rho_h*(10-2.5*rho_h)/(16-2.5*rho_h); NeuHyHI = 9*rho_h*(4-2.5*rho_h)/(16-2.5*rho_h).
Uwat = 0.00481*Sal_ppm/1000?*... (Uwat = 0.00481*Sal + 0.3883, Sal kppm); gas U = 0.119*rho_h, oil U = 0.133*rho_h.
Water density: Den = 1.0 + 7e-7*Sal_ppm - 1e-6*(T_F-80)^2. Salinity from Rmf75: Sal = alog((3.562-log(Rmf75-0.0123))/0.955).

### Default equation confidences (MINEQDEF.PAR)
Unity 0.01 | Density 0.02 g/cc | Neutron 0.02 v/v | Sonic 3.0 us/ft | Cond Cxo/Ct 10 mmho | Res Rxo/Rt 100 ohmm |
EPT 0.2 | U 0.2 | SIGMA 0.2 | GR 5 API | K 1.0 | Th/Ur 0.1 | ECS/elements 0.02 | TOC 0.02 | Linear 1.0 |
BoundWater 0.01 | Constant 0.01 | PhiLimit 0.01 (>Limit, 0.3) | OBM_Limit (<) / WBM_Limit (>) 0.01, IF 0.5.
Invasion factor: 1.0 for all flushed-reading tools, 0.0 for Ct/Rt.

## B. Solver

- Equation rows: Y = sum(Vol_i * Endpoint_i); Unity always (conf 0.01). Modes: Model / >Limit / <Limit / Output.
- Stage 1 LINEAR: normalize rows by confidence -> SVD solve -> iteratively zero the most-negative volume and re-solve until all >= 0 -> renormalize to sum 1 -> reconstruct + error.
- Stage 2 NONLINEAR: DNOPT (Stanford dense nonlinear optimizer) from linear start, full nonlinear equations; keep whichever solution has lower total error.
- TotalErr = sqrt(sum(((Crv - Crv_rec)/Tol)^2)); resistivities -> conductivity then sqrt before use. TotErr > 1 = red.
- Invasion factor: fluid endpoint * IF for Sxo fluids, * (1-IF) for Sw fluids; porosity-equalization constraint auto-added when both families present: 0 = -sum(V_U) + sum(V_X).
- Saturation coupling (recommended): keep Rt/Rxo OUT of linear solve; auto-add Sxo equation `0 = (Sxo-1)*Vwater + Sxo*Vhyd...` with Sxo from Rxo via chosen sat equation each outer iteration (conf 0.01); analogous Sw equation with Rt. Outer loop until dPhi < 0.001, dSxo < 0.002 (max 20/30/10 iters, PhiFlag codes).
- Direct conductivity rows (linear solver only): linearized Archie n=m: Cxo^(1/m) = sum(Vwat_i*(Cwat_i/a)^(1/m)) + ...
- Sat equations available: Archie, Archie PhiT, Simandoux, Mod Simandoux, Indonesian, Dual Water, Juhasz, Waxman-Smits, Poupon-Aguilera, Poupon-Tixier, Woodhouse Tar. W-S B = (-1.28+0.225T-0.0004059T^2)/(1+Rw^1.23*(0.045T-0.27)), T degC.
- Sxo logic WBM: Sw^0.2 >= Sxo >= Sw; OBM: Sxo <= Sw; no Rxo: Sxo = (Sw+IF)/(1+IF).
- Sonic Wyllie with Cp; Hunt-Raymer optional; nonlinear neutron via tool lookup tables (contractor/tool .neu files) with excavation factor Exfact = (rhoma/2.65)^2*(2*Swx*phix^2+0.04*phix)*(1-Swx).
- Porosity bookkeeping (wet clay): Phie = sum(Vwat+Vhyd); Vcl = sum(Vwetclay); PhiT = Phie + Vcl*PhiTClay.
- Final: BVW, BVWsxo, Rwapp = Rt*PhiT^m/a, Qvn, secondary porosity vs sonic, etc.

## C. UI (4 tabs)
Curves (inputs Temp/Rt/Rxo + output curve names) | Zonal Parameters (5 sub-tabs: zones/mixings, waters-clays Rw/Rmf/RwB + salinity sync, Sw logic/limits, Sw params m n a B Qv, sonic-neutron-density options) | Models (up to 20/well; grid equations x minerals; per-row Curve/Val, Equation type, Eq Mode, Use, Confidence, Inv Factor; per-column Mineral, Type, Shading, Use, Result curve; endpoint cells can be curves; blue=Auto, green=true HC density) | Mixings (rule-based model selection per zone, evaluated top-down; model merge distance box-filters Model_Num fractions).
Calibrate: multi-linear regression of endpoints vs core/XRD volume curves (no intercept), per-mineral Fixed/Var, R^2, copy back.

## D. User-defined inputs
- MINEQDEF.PAR: add row `Label CurveType/Value Confidence EqType InvFactor` -> new equation appears in dropdown. Generic `Linear` type exists.
- MINDEF.PAR: add matching column with per-mineral defaults; `Auto` / `-` allowed. New mineral rows addable.
- Any endpoint/confidence/IF accepts a curve instead of a constant.
- Labels ending in (Wt%) get dry-weight -> volume conversion automatically.
