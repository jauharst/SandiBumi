# Jauhar's standard petrophysics workflow — normalization, mnemonics, multimin, reports

Distilled from three of Jauhar's own studies held in his study archive (not in this
repository): a clastic field final report, a carbonate field report, and a multi-well multimin
model set. These are the conventions SandiBumi's defaults should follow.

## Standard workflow order (his every study)

1. Database management + mnemonic standardization → 2. TVDSS calc → 3. Log QC →
4. Pre-calculation (FTEMP from geothermal gradient vs tester temps, FPRESS, RMF) →
5. Core depth shifting → 6. GR normalization → 7. Coal + badhole flagging →
8. Synthetic logs & badhole correction → 9. Parameter zoning → 10. VSH → porosity
(SSPW/SSC/multimin) → SWIRR → perm → Sw → 11. Rock typing (FZI; PERM_COATES first, then
PERM_FZI per RT, calibrate to CPERM+RFT mobility) → 12. Cutoffs & lumping (poro/Vsh/Sw +
HCPV contribution) → 13. Uncertainty analysis → 14. Saturation-height function.

## GR normalization (two-point percentile)

GRN = (GR_well − P3_well)·(P97_ref − P3_ref)/(P97_well − P3_well) + P3_ref, percentiles
taken over a reference interval. Rokan reference values: P3 = 53.68 GAPI, P97 = 133.93
GAPI (from 562 wells). QC via average maps (bullseye check).

## Mnemonic standardization (Bunga table — merged into `curves.rs` FAMILIES)

GR: CGR preferred, else GR. RDEEP: ILD, PSR##(deepest), ATR, BDAV, RING, phase-shift res
as last resort. RSHAL: SN, AHT##(shallowest), SFLU, R25P, BSAV. RMIC: MSFL.
RHOB: RHOB/ROBB/RHOZ/SBD2. DRHO: HDRA/DRHO. NPHI: NPHI/TNPH/FSTP. PEF: PEFZ/PE/PEB/PEF.
DTCO: DTC/DT. DTS: DT_S/DTSM. CALI: CALI/HORD/HCAL.

## Synthetic logs (Facimage MRGC in the reference suite; SandiBumi mirror = `log_predict` KNN module)

- Synthetic RHOB from GRN + clean-RHOB association; **keep raw RHOB where synthetic <
  raw** (washout only pushes RHOB down — the MAX_RAW rule).
- Synthetic NPHI from RHOB_ED + GRN + N-D separation; train ~32 wells, blind-test
  correlation, then generate for wells missing NPHI. Also synthetic DT and U for multimin
  coverage.

## Multimin standards (KKT = the reference suite Multimin, Herron-Matteson response defaults)

- Per-zone per-well models (e.g. kk2_main / massive / postmain / taf / taf_gas).
  Components: QUARTZ+KAOLIN (+CALCITE in massive) + flushed & unflushed fluids
  (XOIL/XBNDWAT/XFREWAT, UOIL/UBNDWAT/UFREWAT); gas zones swap OIL→GAS.
- Active tools: RHOB_ED (unc 0.0264), NPHI_ED (0.042), GRN_F (unc 30), CT unflushed
  conductivity + CXO flushed, both DUAL-WATER NONLINEAR (interval unc). DT/PEF/U
  available but off by default. NPHI converted to limestone units (NPHI_LS) for multimin.
- M=1.78 N=1.76 W=1.775; Waxman-Smits m*=n*=1.6; Indonesia a=1 m=n=1.76 full Vsh
  exponent; Simandoux "Bardon & Pied"; CWBS=20 CWBT=170; RTFU 35, RTFX 1000; bound water
  from "Wet Cl/Sh Porosity Fraction"; clay porosity from "Wet Clay Porosity"; WCS
  porosity from grain density.
- BLSO multimin: RHOB+NPHI_LS+DT+U with quartz/calcite/shale/glauconite, endpoint values
  from the Schlumberger Chartbook, trial-and-error tuned until predicted logs match;
  uncertainty section per multimin.

## Carbonate exploration style (Bunga, Beicip-Franlab)

Variable cementation exponent m for secondary porosity (SPI — secondary porosity index
from sonic vs density porosity), m* 2.20 from SCAL (carbonate), n* 2.00 assumed; clastics
m 1.83 n 1.85. Indonesia-Simandoux for shaly, Archie clean carbonate. ρg carbonate 2.71,
clastic 2.655 (core-based). Rw: Pickett + water-sample salinity. FZI rock typing.

## Report structure (template for the SandiBumi report generator, `report.rs`)

Executive Summary → Introduction/Methodology (parameter–method–remarks TABLE) → Data
Management & Conditioning (mnemonics, TVDSS, LQC, precalc, core shift, GRN, badhole,
synthetics) → Per-formation evaluation (VSH, porosity, SWIRR, perm, Sw, cutoffs+lumping,
uncertainty) → Reservoir quality / payzone classification → Saturation Height Function →
Discussion & recommendations → References. Bilingual (Indonesian body, English terms).

Related: `method_ssc_sspw.md`, `method_lrlc_rtc_imts.md`.
