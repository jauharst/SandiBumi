# Target E — IP2025 mineral endpoints + three-way cross-check

Tier A (reference catalog data, extracted verbatim). Sources, all READ-ONLY:
`C:\Program Files\IP2025\MINDEF.PAR`, `MINEQDEF.PAR`, `ElanToIPMapping.par`, `Mineral Solver Models\*.mdl`.

## Outputs
- `E_ip_endpoints.json` — MINDEF.PAR fully parsed: 30 minerals/fluids x 51 columns.
- `E_mdl_models.json` — best-effort binary parse of the 2 shipped .mdl solver models.
- `E_threeway_endpoint_compare.json` — IP vs Techlog vs SandiMin reconciliation, per mineral-property verdict.

## IP unit conventions (vs Techlog / SandiMin)
| Property | IP MINDEF native | Techlog | SandiMin sec-I |
|---|---|---|---|
| Density | g/cc | g/cc | kg/m3 |
| Sonic | uSec/ft | uSec/ft | uSec/m |
| Neutron | fraction (v/v) | fraction | fraction |
| U (=PEF*rhoe) | barns/cc | barns/cc (separate from PEF b/e) | barns/cc |
| Sigma | c.u. | c.u. | c.u. |
| GR | **none — derived from K_Conc/Th_Conc/U_Conc** | API (direct) | API (direct) |
| CEC | **Qv in meq/cm3 of WET clay** | meq/g | meq/g |

Two IP conventions matter most for SandiBumi:
1. **IP carries no direct GR endpoint.** MINDEF has K_Conc/Th_Conc/U_Conc (weight fractions) and the MinSolve GammaRay equation composes GR from them. So GR is only a 2-way (Techlog vs SandiMin) check. If SandiMin wants an IP GR cross-check it must reconstruct GR from IP's K/Th/U columns.
2. **IP CEC is a wet-clay volumetric Qv, not meq/g.** MINDEF header gives the exact reverse: `CEC[meq/g] = Qv / (GrainDensity_dry * (1 - PhiTClay))`. Applied here. IP Illite Qv 0.59 -> 0.268 meq/g (Techlog/SandiMin 0.25). IP Kaolinite 0.22 -> 0.092 (both 0.10). IP Chlorite 0.38 -> 0.152 (SandiMin 0.15). Illite/Kaolinite CEC agree in substance once converted.

IP `Auto` = computed at runtime (Neutron for pure matrix; Density/Neutron/U for fluids). `-` = undefined / restores default.

## Three-way verdict (18 minerals shared across all three libraries)
Name alignment via ElanToIPMapping.par (QUAR/DOLO/CALC/ILLI/KAOL/CHLO/ALBI/MONT...). IP `Montmorill`=Techlog/SandiMin `Smectite`; IP `Orthoclase`=Techlog `K-Feldspar`; IP `Albite`=Techlog `N-Feldspar`.

Agreement criterion: relative spread <=5% across the libraries present.

| Property | Agree | Diverge |
|---|---|---|
| RHOB (g/cc) | 12 | 6 |
| NPHI (frac) | 5 | 13 |
| DT (us/ft) | 9 | 9 |
| U (barns/cc) | 6 | 12 |
| SIGMA (c.u.) | 6 | 12 |
| CEC (clays, meq/g) | 1 | 3 |

**Validates SandiMin core:** RHOB agrees 3-way for every clean non-clay matrix mineral (12/18). Calcite and Halite agree on *every* property 3-way. U agrees across the diagnostic evaporite/sulphide set (Quartz, Calcite, Anhydrite, Halite, Gypsum, Pyrite). SandiMin's matrix endpoints are corroborated by a third independent vendor library — not just Techlog.

**Expected divergence (library provenance, not bugs):**
- All clays (Illite, Kaolinite, Chlorite, Montmorillonite/Smectite) diverge widely — different clay tool-response vintages, and IP stores a *wet-clay-model* density (e.g. Illite 2.61 g/cc) where Techlog/SandiMin store dry-clay density (2.7 / 2.78). This is a modelling-convention difference.
- NPHI, U, SIGMA diverge for ~2/3 of minerals — three independent endpoint tables built at different times from different chartbooks.
- Notable specific splits: IP Quartz **Sonic = 55 us/ft** (Wyllie sandstone) vs SandiMin 50.4 / Techlog 53; IP Dolomite **U = 9.0** vs SandiMin 9.6; Siderite density IP 3.88 / Techlog 3.70 / SandiMin 3.96 (all "real" siderite endpoints in the literature range 3.7–3.96).

## .mdl solver models (best-effort, secondary)
Two shipped MinSolve models. Component names, curve names, shading, and equation (tool) labels are ASCII and reliable; endpoint values are interspersed little-endian IEEE doubles (sentinels: -999 unset, -99 flag, -2 Auto, 131072 bitfield). Recovered density leaders correctly identify each component (2.71/2.85/2.98/2.65 = Calcite/Dolomite/Anhydrite/Quartz), confirming the models simply reuse MINDEF defaults — MINDEF.PAR remains authoritative.
- `Calcite Dolomite Anhy WetClay Oil - Den Neu Son U Gr.mdl`: Calcite+Dolomite+Anhydrite+Clay(wet)+Oil_Sxo+Water_Sxo; eqns Unity/Density/Neutron/Sonic/GammaRay(+U).
- `Quatz Calcite DryClay Oil - Den Neu Son.mdl`: Calcite+Quartz+Clay(dry, CEC .25/WCLP)+Water_Sxo+Oil_Sxo+BoundWater; eqns Unity/Density/Neutron/Sonic.

## Tier tags for roadmap merge
All of Target E is **Tier A** (reference catalog data) and **v1-core** (openhole petrophysics endpoint library). No Tier C content touched. The MinSolve engine itself (UserProgram.dll, compiled) was not decompiled; only the readable .PAR/.mdl reference data was extracted.
