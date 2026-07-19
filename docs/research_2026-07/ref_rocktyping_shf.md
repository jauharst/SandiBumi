# Rock typing (FZI, Winland/Pittman, Lucia, PGS, perm binning, electrofacies tie-in) and Saturation Height Function building (Leverett-J, Brooks-Corey, Thomeer, Skelt-Harrison, Cuddy FOIL/lambda, log-derived SHF, FWL determination) — fitting/calibration side for SandiBumi, complementing the existing forward sw_height apply module

## Files
- D:\01. Work\00. Guidebook\01. Reference\1941 - Leverett, M.C. - Capillary Behavior in Porous Solids.pdf
  Original Leverett J-function paper (AIME 142:152-169). In Jauhar's library; canonical citation for J-function SHF.
- D:\01. Work\00. Guidebook\01. Reference\1993 - Cuddy etal - A Simple, Convincing Model for Calculating Water in Southern North Sea Gas Fields.pdf
  READ IN FULL. Cuddy/Allinson/Steele SPWLA 1993 — the original FOIL function paper. Contains: survey of 5 legacy SHF forms, J-function derivation, BVW=A*H^B fit (example A=0.01619, B=-0.85771), data-QC rules (net reservoir only, >1 m from bed boundaries), FWL-scan algorithm (Eq 19: minimize sum of squared BVW residuals over candidate FWLs in 0.5-ft steps), Southern North Sea defaults (sigma=74 dyn/cm, theta=0, alpha=0.06, beta=2.31, rho_w=1.02, rho_g=0.2 g/cc, logK=16.931*phi-2.022), threshold-height concept, FWL-vs-GWC discussion.
- D:\01. Work\00. Guidebook\01. Reference\2017 - Cuddy - Using Fractals to Determine a Reservoirs Hydrocarbon Distribution.pdf
  READ pp1-11. Cuddy SPWLA 2017 fractal justification of FOIL: BVW=a*H^b with b=D+3 (D=fractal dimension of pore space); FWL/HWC/transition-zone definitions; net-reservoir cutoff varies with height (net where phi > BVW(H)); FWL picking from BVW-vs-TVDss trend across wells; TVD normalization between wells. Duplicate copy: '2017 - Cuddy, S - Using Fractals to Determine Reservoir Distribution.pdf' in same folder.
- D:\01. Work\00. Guidebook\01. Reference\2014 - Paiaman etal - A new framework for selection of representative samples for special core analysis.pdf
  SCAL sample-selection framework (not read in depth; relevant to which plugs feed SHF fitting).
- D:\01. Work\00. Guidebook\06. PETRO SKILLS - General PE\02_RockProperties\2_4_CapillaryPressure.pdf
  PetroSkills course chapter on capillary pressure (standard Pc theory, lab methods, J-function averaging). Not deep-read; standard content.
- D:\01. Work\00. Guidebook\06. PETRO SKILLS - General PE\02_RockProperties\2_2_Permeability.pdf
  PetroSkills permeability chapter (standard).
- D:\01. Work\00. Guidebook\06. PETRO SKILLS - General PE\02_RockProperties\2_6_Reservoir_Description.pdf
  PetroSkills reservoir description chapter (rock typing context, standard).
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\20221101 Manpatu Core Data\Mahakam Phi-k Laws_Simplification_RP_nobackup.pdf
  READ IN FULL (20 slides, PHM 2022). THE production perm-binning reference for Mahakam: burial-depth zonation (Shallow <1250mSS unconsolidated / Intermediate 1250-2200mSS friable / Deep >2200mSS consolidated), per-zone PhiT_core@NCS->PHIE piecewise-linear conversions, semilog phi-k laws K=10^(a+b*PHIE) with separate K_KL (monophasic) and Kg@Swi (biphasic gas) laws, 3-segment piecewise laws for Tunu-Peciko SU1-SU6, Sisi-Nubi laws, mobility cutoff Mob=1 mD/cp (K=0.026 mD at mu_gas=0.026 cp -> PhiE~0.07). All constants transcribed in methods[].)
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\20221101 Manpatu Core Data\Phi-K Law South Mahakam.pptx
  pptx (cannot read here): South Mahakam phi-k law deck, companion to the PDF above.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\AAA Mahakam Capillary Pressure - all data.xlsx
  Mahakam-wide Pc compilation workbook. Sheets: 'All PC Data', 'Without 0 micp', 'Standard PC' — i.e. mixed MICP + air-brine data with a standardized-Pc sheet. Primary SCAL input example for the SHF-fitting module.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\SGR_Pc_Tunu Sisi Nubi.xlsx
  Tunu/Sisi-Nubi Pc data (SGR = shaly sand context).
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\2016_10_EPS_TW_laws Swt_Kr drainage and imbibition_UD - Sisi Nubi.xlsx
  TotalEnergies EPS Sisi-Nubi Swt(height) and Kr laws, drainage + imbibition — an actual field SHF 'law' deliverable format SandiBumi should be able to reproduce.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\Core\W-MND-1\Appendix - III Special Core Analysis\6. POROUS PLATE\PCP_Core 4-tabel-1.pdf
  READ. Actual porous-plate Pc lab table format (Corelab-style, TOTAL Indonesia W. Mandu-1): header block (company/well/formation), OB stress (4915 psi), pressure columns 1,2,4,8,10,20,30,45,60,80,100,150 psi; rows = Sample, Depth(m), Perm(mD), Poro(%); cell values = brine saturation %PV. Defines the wide-format import the SCAL loader must parse.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\Core\W-MND-1\Appendix - III Special Core Analysis
  Full SCAL report folder tree: 1 PRO_COND, 2 CEC, 3 BASIC PARAM, 4 FORMATION FACTOR (FF-OB tables+graphs), 5 RESISTIVITY INDEX, 6 POROUS PLATE, 7 CENTRIFUGE CAPILLARY PRESS, 8 RESID IMBIBITION, 9 ROCK COMPRESS — the canonical SCAL deliverable structure Jauhar receives.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\Core\W-MND-1\W-MND-1_Pc Data for asset
  Digitized centrifuge Pc workbooks per plug ('Tabel Sat Ph2-12A(ok).xlsx', 'S-16A.xlsx', ... plus 'PcPP and Curve.xlsx' merge files) — second import format (per-sample centrifuge tables).
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\20221101 Manpatu Core Data\Core_data_SCS.CSV
  READ header. Geolog-export CCAL/RCA core table format: row1 mnemonics DEPTH,K_KL_CORE_NCS_1,PHIE_CORE_NCS_1,PHIT_CORE_NCS_1,RHOG_CORE_1,SAMPLE_NUMBER_1,WELL_NAME_1; row2 units METRES,%,%,%,G/C3; then data. Companion Core_data_SDS.CSV same format. This is the phi-k input table shape for rock typing.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\Core\phi-K\131009_SMK_PhiK_STP-2_W-MND-1 FINAL.xlsx
  Study phi-k crossplot workbook (STP-2 + W-MND-1); with 131021_KH_DST_vs_log.xlsx (KH from DST vs log-derived KH validation) — the calibration/QC artifacts a perm-binning module must emit.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\20221101 Manpatu Core Data\20230116 Kr Pc Adjustment.xlsx
  Kr/Pc adjustment workbook (Manpatu) — example of post-fit Pc curve adjustment workflow.
- D:\01. Work\2026\43. SCS - PHM\01. Data\05_Core and Petrophysical Data\OTHER REFERENCE\Bekapai Reference\Pc-Sw_BL-9_Final-SCAL-Report_Corelab_120509.xls
  Bekapai Corelab Pc-Sw SCAL export; folder also has 'BL SCAL & CCAL.xls', 'calcul_KR_tout corey_BEKAPAI.xls' (Corey fitting sheet), 'Pc-Sw_depth conversion_HWA 031020.xlsx' (Pc-to-height conversion example).
- D:\01. Work\00. Guidebook\01. Reference\IPA19-G-545.pdf
  Skimmed p1. PHE ONWJ 'MAINE' method — includes 2PST (Porosity-Permeability-Irreducible Sw Transformation) and Type Curve Analysis with rock-type-dependent m,n; tangential rock-typing reference for LRP.
- D:\01. Work\00. Guidebook\01. Reference\1999 - Coates et al - NMR Logging Principles and Applications.pdf
  NMR book (Coates/Timur perm, T2-based pore size) — supporting reference for log-driven rock typing, not read.

## Methods
### FZI / RQI Hydraulic Flow Units (Amaefule et al. 1993, SPE 26436) — SPECCED FROM LITERATURE (no local file)
Cluster core phi-k data into Hydraulic Flow Units using the Flow Zone Indicator; per-HFU perm transform; extendable to logs by predicting FZI.

Equations:
RQI[um] = 0.0314*sqrt(k[mD]/phi_e); phi_z = phi_e/(1-phi_e); FZI[um] = RQI/phi_z. On log-log RQI vs phi_z, samples of one HFU fall on a unit-slope line with intercept FZI at phi_z=1. Inverse perm predictor per HFU: k[mD] = 1014.24 * FZI_mean^2 * phi_e^3/(1-phi_e)^2. GHE option (Corbett & Potter 2004): fixed global FZI bin boundaries 0.0938, 0.1875, 0.375, 0.75, 1.5, 2.5, 4, 6, 8 defining GHE1-GHE10 (verify against paper at implementation).

Inputs: Core table: WELL, SAMPLE, DEPTH, k (Klinkenberg @ NCS preferred, mD), phi_e (@ NCS, frac). Exact shape of Core_data_SCS.CSV (mnemonic row + unit row + data).
Outputs: Per-sample RQI, phi_z, FZI, HFU id; per-HFU FZI_mean (geometric), k(phi) transform, sample count, R2; depth-track HFU flag curve.
Calibration: 1) Compute FZI per sample. 2) Cluster log10(FZI): options = (a) histogram/probability-plot break picking (interactive), (b) Ward hierarchical clustering on log FZI (agglomerative, minimum-variance linkage; user chooses n clusters from dendrogram/silhouette), (c) fixed GHE bins. 3) Per HFU take geometric-mean FZI, back-compute k(phi) curve, overlay on phi-k crossplot. 4) Validate: predicted-vs-core k cross plot, KH vs DST. 5) Log tie: regress FZI (or HFU class) on log curves (GR, RHOB-NPHI, Vsh, PHIE) or ML classifier to propagate to uncored wells.

### Winland R35 + Pittman pore-throat radius — SPECCED FROM LITERATURE (no local file)
Assign rock types by pore-throat radius at reference mercury saturation computed from k-phi regression; bin into port-size classes.

Equations:
Winland (Kolodzie 1980): log10 R35 = 0.732 + 0.588*log10 k_air[mD] - 0.864*log10 phi[%], R35 in um. Pittman (1992) r35: log10 r35 = 0.255 + 0.565*log10 k - 0.523*log10 phi (same units); Pittman's Table 1 gives coefficients for r10...r75 (r25 variant commonly used for finer rocks) — store the whole coefficient table as data, transcribe from Pittman 1992 at implementation. Port classes (Hartmann-Beaumont/Martin): mega >10 um, macro 2.5-10, meso 0.5-2.5, micro 0.1-0.5, nano <0.1.

Inputs: Core k_air (mD) and phi (%) per sample; optionally MICP curves to derive true r35 (apex/percentile of throat-size distribution via Washburn r[um]=2*sigma*cos(theta)/Pc -> for Hg-air: r = 107.6/Pc[psi] approx using sigma=480, theta=140).
Outputs: R35/rX per sample, port-class rock type, per-class k-phi transform; iso-R35 curve overlays for the phi-k crossplot (R35 fixed, solve k as function of phi).
Calibration: If MICP exists: compute measured r35 from each curve, regress local coefficients log r35 = a + b*log k + c*log phi (multi-linear least squares) instead of the global Winland constants; else use Winland/Pittman defaults. Choose class boundaries on R35 histogram or fixed port classes; QC against facies descriptions.

### Lucia Rock-Fabric Number (carbonates; Lucia 1995/2007, Jennings & Lucia 2003) — SPECCED FROM LITERATURE (no local file)
Carbonate rock typing via rock-fabric number (RFN ~0.5-4) from interparticle porosity and permeability; classes 1 (grainstone), 2 (grain-dominated packstone), 3 (mud-dominated).

Equations:
Global transform (Jennings & Lucia 2003): log10 k[mD] = (A - B*log10 RFN) + (C - D*log10 RFN)*log10 phi_ip, with A=9.7982, B=12.0838, C=8.6711, D=8.2965 (phi_ip = interparticle porosity fraction, k in mD) — VERIFY constants against the paper before release. Invert numerically (1-D root find in log10 RFN, monotonic) to get RFN from each (phi_ip, k) pair. Class bins: Class1 RFN 0.5-1.5, Class2 1.5-2.5, Class3 2.5-4.

Inputs: Core phi_total, separated vuggy porosity (phi_sv from thin section/NMR/sonic-vs-total method) so phi_ip = phi_total - phi_sv; core k.
Outputs: RFN per sample, Lucia class, per-class k(phi_ip) curves; log-domain RFN if phi_ip derivable from logs.
Calibration: Compute RFN per plug, map RFN vs depositional fabric from core description, choose class boundaries; forward k prediction from mapped RFN + log phi_ip. Mahakam is clastic-dominated so this is secondary, but Jauhar requested it for carbonate stringers.

### PGS Pore Geometry-Structure rock typing (Permadi & Susilo, ITB) — SPECCED FROM LITERATURE, FLAG FOR VERIFICATION (no local file found)
Indonesian (ITB) rock-typing method: log-log crossplot of pore-geometry variable (k/phi) vs pore-structure variable (k/phi^3.5); samples with similar pore architecture fall on straight power-law lines; rock types are bands between fitted lines.

Equations:
From Kozeny-Carman k = phi^3/(c*tau*Sgv^2*(1-phi)^2): define pore geometry PG = k/phi (proportional to r_mh^2, mean hydraulic radius squared) and pore structure PS = k/phi^3.5 (empirically modified exponent 3.5 replacing Kozeny's 3 — per Permadi & Susilo 2009). Rock-type line: (k/phi) = a*(k/phi^3.5)^b fitted log-log per group; with b typically near 1. Per-RT permeability prediction by inversion: k^(1-b) = a*phi^(1-3.5b) => k = [a*phi^(1-3.5b)]^(1/(1-b)) (guard b->1: then line defines constant phi, use direct k-phi regression fallback). Workflow references: Permadi & Susilo 2009 (SPE 125350, 'Permeability Prediction and Characteristics of Pore Structure and Geometry as Inferred from Core Data') and Wibowo & Permadi 2013 (IPA) — MUST verify exact exponent (3 vs 3.5), whether PG uses sqrt(k/phi), and published a,b ranges against the papers; no copy exists in Jauhar's library.

Inputs: Same core phi-k table as FZI (k mD, phi frac; consistent stress state).
Outputs: PG, PS per sample; RT id per sample from band assignment; per-RT (a,b) and k(phi) inversion curve; crossplot with iso-RT lines.
Calibration: 1) Plot log PG vs log PS colored by facies. 2) Fit family of parallel lines: either free (a,b) per user-drawn group, or fix common slope b (regress pooled) and cluster intercepts log a (1-D clustering like FZI). 3) Define RT bands midway between adjacent lines. 4) Validate k prediction per RT; tie RT to electrofacies for log propagation. Include a method-comparison view (PGS vs FZI vs R35 class per sample) since Jauhar will run several.

### Permeability binning / per-rock-type phi-k transforms — FROM FILE: Mahakam Phi-k Laws (PHM 2022 deck)
The Mahakam production standard: zone data by burial depth and delta axis, convert core PhiT@NCS to PHIE with piecewise-linear laws, then fit semilog (log10 k linear in PHIE) or piecewise-3-segment laws, separately for monophasic K_KL and gas-at-Swi Kg; apply mobility-based lower cutoff.

Equations:
PhiT->PhiE conversions: Inner/Median Shallow: PhiT>=0.31: PHIE=max(0,min(PhiT,1.21605*PhiT-0.0810932)); PhiT<0.31: PHIE=max(0,min(PhiT,3.29839*PhiT-0.7298850)). Intermediate: PHIE=max(0,min(PhiT,1.74905*PhiT-0.241863)). Handil deep: PHIE=max(0,min(PhiT,1.4578*PhiT-0.0632)). Tunu-Peciko: PHIE=max(0,min(PHIcore_ovbd,1.28*PHIcore_ovbd-0.051)). Sisi-Nubi: PHIE=max(1.313*PhiT-0.095, 1.021*PhiT-0.031). K laws: Shallow (surface-1250mSS, PhiE>=0.14): K_KL_NCS=10^(-0.108295+9.62881*PHIE); Kg@Swi=10^(-0.223328+9.75957*PHIE) [gas-bearing only]. Intermediate (1250-2200mSS, PhiE>=0.04): K_KL_NCS=10^(0.816306+8.85422*PHIE); Kg@Swi=10^(0.749543+8.95089*PHIE). Deep/Tunu-Peciko SU1-SU6 3-segment monophasic: K=min(10^(0.892+8.17*PHIE), min(10^(-2.81+31.07*PHIE), 10^(-4.084+50.68*PHIE))) i.e. PHIE>0.16 / 0.065-0.16 / <0.065 segments; Kg@Swi: PHIE>0.16: 10^(8.70*PHIE+0.718); 0.095-0.16: 10^(35.78*PHIE-3.607); <0.095: 10^(50.68*PHIE-5.020). Sisi-Nubi (PhiT-based): K_high=10^(0.620413+9.35851*PhiT); K_low=10^(9.41756+10.6268*log10(PhiT)). Flow cutoff: mobility 1 mD/cp rule => with mu_gas=0.026cp, K=0.026 mD => PhiT=0.09 / PhiE=0.07.

Inputs: Core CCAL at NCS (per-well NCS psi recorded, e.g. Sisi-Nubi 2600-5000 psi list on slide 20), PhiT_core, K_KL (and Kg@Swi from SCAL rel-perm), zone attribute (burial depth mSS, delta axis, facies SF1-SF8); Geolog-CSV core table format (Core_data_SCS.CSV).
Outputs: Per-bin (zone x facies-group x fluid) law coefficients (a,b) of log10 K = a + b*PHIE, optional piecewise segments with break phis; predicted K and Kg@Swi curves; KH integration for DST comparison.
Calibration: 1) QC/convert core phi to NCS and to PHIE (fit the piecewise linear conversion vs upscaled e-log PHIE, max/min clamped). 2) Group by burial zone (breaks e.g. 1250/2200 mSS) — deck shows 'no obvious burial effect' test should be run per dataset. 3) Least-squares of log10 K on PHIE per group; allow user-forced breakpoints for piecewise fits; draw p10/p90 envelope lines. 4) Derive biphasic law from Kg@Swi data or Kg/K ratio curve. 5) Validate KH(log) vs KH(DST) (131021_KH_DST_vs_log.xlsx pattern). SandiBumi module should store laws per rock type/zone and expose them to the perm-curve compute.

### Cutoff-based electrofacies tie-in — from Jauhar workflow standards + STP-2 cutoff study (no formal paper)
Propagate core rock types to uncored intervals via log-domain classes built from cutoffs (Vsh/GRN, PHIE, Sw), then attach per-class phi-k and SHF laws.

Equations:
Electrofacies class = f(cutoffs): e.g. RT1 if Vsh<v1 and PHIE>=p1; RT2 if v1<=Vsh<v2 or p2<=PHIE<p1; else non-net. GRN normalization per Jauhar standard: GRN scaled to P3/P97 percentiles. Confusion-matrix tie: for cored intervals, cross-tabulate electrofacies vs core RT (FZI/PGS/R35 class); accept mapping if dominant-class purity above threshold (user-set, e.g. 70%).

Inputs: Computed log curves (VSH, PHIE, SW, GRN), core-derived RT flags at matched depths (after depth-shift), cutoff values.
Outputs: RT_LOG curve per well; per-RT net flags; purity/confusion matrix QC table.
Calibration: Depth-match core to log (existing core-shift), classify, iterate cutoffs to maximize agreement; alternative supervised classifiers optional. This bridges rock typing to the SHF-apply module (per-RT SHF selection).

### Leverett J-function SHF build — FROM FILES (Leverett 1941; Cuddy 1993 for form and defaults) + literature unit constants
Normalize all Pc curves to dimensionless J, fit J-Sw power/lambda law per rock type, convert to Sw(h) with unit-consistent constants. SandiBumi already has the forward apply; this module does the fitting.

Equations:
J(Sw) = C * Pc/(sigma*cos(theta)) * sqrt(k/phi); unit-consistent C=0.21645 for Pc[psi], k[mD], sigma[dyn/cm] (C=1 if Pc dyn/cm2, k cm2). Fit form (Cuddy Eq 8): J = alpha * Sw^-beta (log-log linear least squares), or with irreducible: Sw = Swirr + (1-Swirr)*(J_entry/J)^lambda. Height link: Pc_res[psi] = 0.433*(rho_w - rho_hc)[g/cc]*h[ft] = 1.422e-1*(rho_w-rho_hc)[g/cc]*h[m]*... implement as Pc[psi]=(rho_w-rho_hc)[kg/m3]*9.80665*h[m]/6894.76. Lab->reservoir system conversion: Pc_res = Pc_lab*(sigma cos theta)_res/(sigma cos theta)_lab. Default sigma*cos(theta) table (Amyx/Core Labs standard): lab air-brine 72*cos0=72; lab oil-brine 48*cos30~41.6; lab Hg-air 480*cos140~367.7 (magnitude); reservoir gas-brine ~50 (theta 0); reservoir oil-brine ~30*cos30~26. Cuddy 1993 SNS example: sigma=74, theta=0, alpha=0.06, beta=2.31, rho_w=1.02, rho_g=0.2 g/cc.

Inputs: SCAL/MICP long table: (well, sample, depth, k_plug mD, phi_plug frac, system [air-brine|Hg-air|oil-brine|centrifuge], stress psi, rows of (Pc, Sw)); rock-type id per sample; fluid densities and sigma-cos-theta lab & reservoir.
Outputs: Per-RT (alpha, beta) or (Swirr, J_entry, lambda); pooled J-Sw crossplot with fit and scatter stats; Sw(h) curves per RT at chosen phi-k pairs; exportable law table for the forward sw_height module.
Calibration: 1) Standardize Pc: convert Hg-air and lab air-brine to common reservoir system. 2) Compute J point-by-point per sample using plug k, phi. 3) Pool by rock type; robust log-log regression (optionally exclude Sw>0.9 plateau and lowest-Pc points below entry). 4) Report R2 and per-sample residuals to catch samples that belong in another RT (rock-typing feedback loop). 5) Optionally fit Swirr by nonlinear least squares (Levenberg-Marquardt) on Sw = Swirr+(1-Swirr)*a*J^-b.

### Brooks-Corey fit to Pc/MICP — SPECCED FROM LITERATURE
Per-sample (or per-RT) drainage Pc parameterization with entry pressure and pore-size distribution index.

Equations:
Se = (Sw - Swirr)/(1 - Swirr) = (Pe/Pc)^lambda for Pc > Pe; Sw=1 for Pc<=Pe. Equivalent log-log line: log Se = lambda*(log Pe - log Pc). Height form: Sw(h) = Swirr + (1-Swirr)*(he/h)^lambda for h>he, he = Pe/(gradient) with gradient 0.433*(rho_w-rho_hc)[g/cc] psi/ft.

Inputs: Same standardized Pc long table; initial Swirr guess (min Sw of curve).
Outputs: Per-sample (Pe, lambda, Swirr) + fit stats; per-RT representative parameters (geometric mean Pe, arithmetic lambda) and correlations Pe vs sqrt(k/phi) (log-log) to let Pe scale with rock quality inside a RT.
Calibration: Nonlinear least squares over (Pe, lambda, Swirr) with bounds (Pe>0, 0.2<lambda<10, 0<=Swirr<min Sw); good initialization: Pe = Pc at Sw=0.995 extrapolated, lambda from mid-curve slope. Fit each sample, then regress parameters vs rock quality within RT; or fit pooled per-RT after J-normalization.

### Thomeer hyperbola fit to MICP — SPECCED FROM LITERATURE
MICP-native parameterization (bulk-volume domain) suited to multi-pore-system rocks; basis of Thomeer/Clerke workflows.

Equations:
Bv(Pc)/Bv_inf = exp(-G / ln(Pc/Pd)) for Pc > Pd, where Bv = phi*S_Hg (bulk volume fraction invaded), Bv_inf = asymptotic invaded volume, Pd = displacement (entry) pressure, G = pore geometrical factor (typ. 0.1-1, lower=better sorted). Multi-modal rocks: Bv_total = sum_i Bv_inf,i * exp(-G_i/ln(Pc/Pd,i)) (2-3 systems). Sw conversion: Sw = 1 - Bv/(phi) (for Hg as nonwetting analog). Optional Swanson perm: k_air = 399*(Bv/Pc)_apex^1.691 (Swanson 1981 — verify constants before release).

Inputs: MICP table per sample: rows (Pc_Hg psi, S_Hg or Bv), plug phi, k; conformance-corrected (remove closure/surface conformance at low Pc — the Mahakam workbook sheet 'Without 0 micp' suggests zero/closure points already filtered).
Outputs: Per-sample (Pd, G, Bv_inf) per pore system; apex point (Bv/Pc)max; derived Sw(h) after system conversion; Thomeer-class rock types (cluster on Pd-G plane).
Calibration: Nonlinear fit in log Pc - log Bv space; detect multimodality from dBv/dlogPc; initialize Pd at first inflection. Rock-type by clustering (Pd, G, Bv_inf); convert type-average curve to reservoir system and height for the SHF library.

### Skelt-Harrison SHF — SPECCED FROM LITERATURE (Skelt & Harrison SPWLA 1995)
Direct height-domain 4-parameter fit with good behavior in transition zone and at asymptote; commonly fit to log Sw or Pc data per rock type.

Equations:
Sw(h) = 1 - A * exp(-(B/(h + D))^C). A ~ (1 - Swirr) asymptote amplitude, B height-scale, C shape (curvature), D vertical shift (entry-height offset; D<0 delays desaturation above FWL). h = height above FWL in consistent units.

Inputs: Either (a) standardized Pc curves converted to h, or (b) log-derived Sw vs h point cloud per rock type.
Outputs: Per-RT (A,B,C,D) + fit stats; Sw(h) family plots.
Calibration: Nonlinear least squares with bounds (0<A<=1, B>0, C>0, D free); initialize A=1-min(Sw), C=1; robust loss (Huber) recommended because log Sw clouds are noisy; constrain monotonic decrease of Sw with h.

### Cuddy FOIL / fractal BVW function — FROM FILES (Cuddy 1993 + Cuddy 2017, both read)
Field-wide porosity- and permeability-independent SHF: bulk volume water is a power law of height above FWL; simplest robust SHF for shaly deltaic sands; doubles as FWL finder and height-varying net cutoff.

Equations:
BVW = Sw*phi = a*H^b (b negative; fractal interpretation b = D+3... i.e., dimensionless, from pore-space fractal dimension). Fit domain: log10 BVW = log10 a + b*log10 H by least squares (1993 example: a=0.01619, |b|=0.85771). Applied Sw: Sw = min(1, a*H^b/phi); rock is water-saturated (non-net for HC) where phi <= a*H^b — i.e., the FOIL curve IS the height-varying net-reservoir porosity cutoff (2017 paper Fig 24). Lambda-function variant (per-RT instead of field-wide): Sw = min(1, max(Swirr, c*H^d/phi)) fit per rock type.

Inputs: Per-well computed PHIE and SW (from the petrophysical workflow), TVDSS, FWL depth per well/compartment; QC flags. Data-selection rules from 1993 paper: net reservoir only, exclude points within 1 m of bed boundaries (tool-resolution contamination), exclude non-net porosity cutoff intervals, use near-vertical wells with good poroperm near the contact to derive; apply everywhere.
Outputs: (a,b) per field/zone (optionally per facies — 1993 Hyde used slight per-facies variants); BVW-vs-H log-log plot with fit; back-calculated Sw curves per well vs log Sw (QC overlay); HCPH = (1-Sw)*phi*H ranking of intervals.
Calibration: 1) Assemble (H, BVW) from all wells with known FWL. 2) Log-log least squares (BVW independent variable per paper). 3) QC: back-calculate zonal Sw vs log-derived Sw per well; check porosity-independence by coloring residuals by phi, k, facies. 4) Iterate FWL if scatter minimized at different contact (see FWL method). 5) Export as SHF law for the reservoir model and for wells without resistivity.

### Log-derived SHF per rock type (direct Sw-vs-height fit) — GENERIC + Cuddy QC rules
Skip SCAL: fit Sw(h) directly to computed log Sw grouped by rock type and height above FWL; the pragmatic Mahakam route when Pc data are sparse; mirrors the Sisi-Nubi 'Swt laws' workbook.

Equations:
Per RT choose functional form: power/lambda Sw = max(Swirr, min(1, c*h^d)); or Skelt-Harrison; or BVW-based FOIL per RT. Recommended default: fit log10 Sw vs log10 h (robust line) with floor Swirr = P5(Sw) high in column and cap Sw=1 below entry height h_e solved from fit crossing Sw=1: h_e = (1/c)^(1/d).

Inputs: Computed SW, PHIE, RT_LOG per 0.1524 m sample; FWL per compartment; filters: exclude thin beds (<1 m), bad hole, transition-invaded/OBM flushing issues, wells far off-depth (TVD error +-30 ft per Cuddy 2017 -> allow per-well depth shift to common FWL).
Outputs: Per-RT curve parameters + envelope (P10/P50/P90 fits) for uncertainty; comparison overlay against any SCAL-derived SHF (J or BC) for consistency (Cuddy's three-source consistency: core Pc, log Sw, formation pressure).
Calibration: Group points by RT; robust regression; iterate jointly with FWL scan; QC: HPV computed with fitted SHF vs HPV from log Sw per well must agree within tolerance (report %diff).

### FWL determination — FROM FILE (Cuddy 1993 Eq 19) + standard pressure-gradient method
Locate the free water level: pressure-gradient intersection when RFT/MDT data exist; Cuddy correlation scan from log BVW when they do not; explain apparent GWC variation via threshold height.

Equations:
(1) Gradients: fit hydrostatic line Pw = mw*TVD + cw (water zone) and hydrocarbon line Phc = mh*TVD + ch (HC zone) from formation pressure points; FWL at intersection TVD* = (ch-cw)/(mw-mh). Gradients give in-situ densities: rho = m/0.433 g/cc (psi/ft). (2) Cuddy scan (no clear contact): for candidate FWL_i stepped 0.5 ft over search window, Quality(i) = sum_j (BVW_log(j) - BVW_FOIL(j))^2 / N_levels; FWL = argmin Quality; also usable to depth-normalize wells to a common FWL. (3) Threshold height / HWC: hydrocarbon-water contact sits above FWL by entry height h_e = Pe/(0.433*delta_rho) per local rock quality — explains well-to-well 'varying GWC' (Cuddy 1993). FWL preferred over GWC because: GWC unclear in thick transition zones, FWL common datum for all rock types, pressures and Pc reference FWL.

Inputs: RFT/MDT (TVD, pressure, fluid), or computed BVW logs + FOIL fit; densities; search window.
Outputs: FWL depth + uncertainty (correlation-coefficient-vs-depth curve, sharpness of minimum), per-well contact table, per-compartment grouping suggestion (wells whose scans disagree beyond deviation-survey error).
Calibration: Derive FOIL constants from best wells with clear contacts first; then scan poor wells; group wells by common FWL allowing larger error for high-deviation wells (Cuddy: TVD error up to 30 ft).


## Notes
SOURCING: SHF side is strongly grounded in Jauhar's own library — Cuddy 1993 (FOIL, read fully), Cuddy 2017 (fractal update, read pp1-11), Leverett 1941 present. Perm binning is grounded in the actual Pertamina Hulu Mahakam 'Phi-k Laws' deck (read fully; all law constants transcribed into methods[]) — SandiBumi should treat these Mahakam laws as a shippable preset. Rock typing proper (FZI, Winland/Pittman, Lucia, PGS) has NO reference files anywhere under 'D:\01. Work\00. Guidebook' (globbed for FZI/Winland/Lucia/rock-typ/Thomeer/Permadi/pore-geometry — zero hits), so those four methods are specced from standard published literature: Amaefule et al. 1993 SPE 26436; Kolodzie 1980 SPE 9382; Pittman 1992 AAPG; Corbett & Potter 2004 (GHE); Lucia 1995 AAPG / Jennings & Lucia 2003 SPE 78740; Permadi & Susilo 2009 (SPE 125350) and Wibowo & Permadi 2013 (IPA) for PGS. VERIFY BEFORE RELEASE (stated from memory, no local copy): (1) PGS exact axes/exponent — task brief and my recall say k/phi vs k/phi^3.5 log-log, but confirm 3.5 vs 3.0 and whether pore geometry uses sqrt(k/phi), plus published a,b ranges; (2) GHE FZI bin boundary list; (3) Jennings-Lucia RFN transform constants; (4) Pittman full rX coefficient table; (5) Swanson perm constants. Do not hardcode until checked against the papers. DATA FORMATS captured from real project files (SCS-PHM study): core phi-k table = Geolog CSV export with mnemonic row + unit row (DEPTH/K_KL_CORE_NCS/PHIE_CORE_NCS/PHIT_CORE_NCS/RHOG/SAMPLE_NUMBER/WELL_NAME); porous-plate Pc = wide table, rows (sample, depth m, k mD, phi %) x pressure columns psi with Sw %PV cells and an OB-stress header; centrifuge Pc = per-plug xlsx workbooks; Mahakam Pc compilation xlsx has sheets All PC Data / Without 0 micp / Standard PC (mixed MICP + air-brine, pre-standardized). Recommend a DuckDB long-format scal_pc table (well, sample, depth, k, phi, system, stress, pc_psi, sw_frac, rt_id) plus importers for the wide porous-plate and per-plug centrifuge shapes. INTEGRATION: SandiBumi already has a forward sw_height (Leverett-J apply) module — this backlog item is the fitting/building side; every SHF method above must export its fitted law in whatever parameter table the forward module consumes, and rock-typing outputs (RT id per depth sample via electrofacies tie-in) select which law applies. Cuddy's three-source consistency check (core Pc vs log Sw vs formation pressures) is the natural top-level QC screen for the module. Nothing was modified; research only.
