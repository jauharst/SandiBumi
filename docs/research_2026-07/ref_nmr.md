# NMR log analysis suite (T2 distribution model, CBW/BVI/FFI partition, MPHI/MSIG porosity, Timur-Coates & SDR permeability, T2 log-mean, hydrocarbon typing, pseudo-Pc, Swirr) for SandiBumi compute module

## Files
- D:\01. Work\00. Guidebook\01. Reference\1999 - Coates et al - NMR Logging Principles and Applications.pdf
  PRIMARY SOURCE. Coates, Xiao & Prammer, Halliburton/NUMAR 1999 (publication H02308), ~250 pp. The canonical NMR logging text the task named. Read in detail: Ch.1 (printed pp.6-16, PDF 24-34: raw data, NMR porosity, T2 distribution & bin displays, FFI/BVI, permeability overview, Coates-C calibration crossplot, hydrocarbon typing, MRIAN intro); Ch.3 (printed 45-67, PDF 63-85: relaxation mechanisms eqs 3.1-3.16, multi-exponential decay 3.17-3.22, echo-fit inversion 3.23, pore-size/MICP correlation, cutoff BVI + core calibration workflow, spectral BVI eqs 3.24-3.27, Coates permeability eq 3.28, SDR eq 3.29, MRIL porosity model MPHI/MCBW/MSIG); Ch.6 (printed 122-133, PDF 140-151: diffusion analysis, DIFAN eqs 6.1-6.4, EDM T2DW eqs 6.5-6.9, TDA full math appendix 6-A.1 to 6-A.13); Ch.7 (printed 135-140, PDF 153-158: MRIAN dual-water eqs 7.1-7.17, total-porosity activation). PDF page = printed page + 18.
- D:\01. Work\00. Guidebook\01. Reference\1982 - Coates etal - Volan, An Advanced Computational Log Analysis.pdf
  Same first author but NOT NMR - Volan is a conventional (Rxo/Rt, EPT) computational analysis. Listed only because it matched *Coates* glob; not read for this task.
- D:\01. Work\00. Guidebook\01. Reference\1941 - Leverett, M.C. - Capillary Behavior in Porous Solids.pdf
  Leverett J-function original paper; relevant supporting reference for the pseudo-Pc-from-T2 module (J-function normalization of core Pc used in calibration). Not NMR-specific; not read in detail.

## Methods
### T2 distribution data model (array-curve storage)
The T2 distribution is the porosity-binned inversion product of the CPMG echo train: incremental porosity phi_i assigned to m pre-selected, log-spaced relaxation times T2_i. Coates book (Ch.3, eq 3.23 + Ch.1 Fig 1.7): m ranges 2-50; MRIL wellsite product uses 12 bins at 0.5,1,2,4,8,16,32,64,128,256,512,1024 ms when MSIG (total porosity) is shown, and 9 bins 4-1024 ms when MPHI (effective) is shown; a bin labeled '8 ms' covers measurements 6-12 ms (bin = interval, label = nominal center). Service companies deliver the distribution as an array log: Halliburton MRIL/MRIL-Prime 12 bins (BIN1..BIN12 or T2DIST), Schlumberger CMR/CMR-Plus typically 30 log-spaced bins 0.3-3000 ms (DLIS array channel T2_DIST plus T2_BIN_TIMES), Baker Hughes MREX 27-64 bins - so SandiBumi must NOT hard-code bin count. Recommended DuckDB model: one array column (LIST<DOUBLE> or fixed FLOAT[] per depth row) holding incremental porosity per bin in v/v, plus a per-curve metadata table row storing the bin-time vector (ms), bin count, spacing type (log10), units, tool, and activation (TE, TW, NE). Store companion scalar curves delivered alongside (MPHI/TCMR, MSIG/CMRP, MCBW, BVI, FFI/CMFF, MPERM/KTIM, T2LM). Derived partitions must always be recomputed from the array + cutoffs rather than trusted from vendor scalars, since T2cutoff is re-calibrated locally.

Equations:
Echo train forward model (eq 3.17/3.23): M(t) = sum_i M_i(0) exp(-t/T2_i); echo(k) = sum_{i=1..m} phi_i exp(-t(k)/T2_i) + noise, t(k)=k*TE. Inversion is regularized NNLS (phi_i >= 0); area under distribution = total porosity (eq 3.21: phi = sum phi_i = M(0)/M_100%(0)). Cumulative porosity at T2c: PHI_cum(T2c) = sum_{T2_i<=T2c} phi_i. Interpolation within a bin (needed when a calibrated cutoff falls between bin centers): log-linear split of the bin's phi_i is the standard practical choice.

Inputs: Array: T2 bin amplitudes (porosity units, per depth). Scalars/metadata: bin-time vector (ms), TE (ms), TW (s), number of echoes NE, tool ID, HI assumption. Raw echo trains are normally NOT delivered to the interpreter; SandiBumi should consume the inverted distribution, not perform echo-fit.
Outputs: Stored array curve + bin-time metadata; cumulative-porosity curve; all downstream NMR products.
Calibration: Vendor inversion is calibrated to a 100% water tank (M_100%(0)). QC per book Ch.9: MPHI vs MSIG consistency, MPHI vs neutron-density crossplot porosity agreement, HI/TW polarization effects on MPHI.

### T2 cutoffs -> CBW / BVI / FFI partition (cutoff-BVI method)
Two fixed cutoffs partition the distribution. (1) Clay-bound cutoff: T2 < ~4 ms = MCBW (clay-bound water); book states MCBW is the area for T2 < 4 ms and MPHI the area for T2 >= 4 ms (Ch.1 p.8). Some sandstone practice uses 3 ms - make it a parameter T2cutoff_clay, default 4 ms (MRIL convention). (2) Free-fluid cutoff T2cutoff divides MPHI into BVI (capillary-bound, T2cutoff_clay <= T2 < T2cutoff) and FFI (free fluid, T2 >= T2cutoff). Defaults in absence of core (book Ch.3 p.58): 33 ms sandstone, 92 ms carbonate ('work very well in Gulf of Mexico'); task/industry range for carbonates 92-100 ms. Cutoff is affected by lithology, pore-wall chemistry, paramagnetic/ferromagnetic minerals, texture, pore-throat/pore-body ratio, and varies sample-to-sample even in one lithology (Fig 3.9) - core calibration essential per field/facies.

Equations:
MCBW = sum(phi_i for T2_i < T2cutoff_clay); BVI = sum(phi_i for T2cutoff_clay <= T2_i < T2cutoff); FFI = MPHI - BVI = sum(phi_i for T2_i >= T2cutoff); MSIG (phiT) = MCBW + MPHI. All in v/v.

Inputs: Array: T2 distribution. Scalars: T2cutoff_clay (default 4 ms), T2cutoff (default 33 ms ss / 92 ms carb; zone-parameter, core-calibrated).
Outputs: MCBW, BVI, FFI (v/v) continuous curves.
Calibration: Book Fig 3.8 workflow: lab NMR on core plug at Sw=100% and again after desaturation to Swirr (centrifuge or porous plate at a specified air-brine capillary pressure - pressure choice must match intended use: producible-water estimate vs permeability, and height above free water). Plot both cumulative-porosity curves vs T2; enter at the irreducible-state total (plateau) porosity value on the cumulative axis, project horizontally to the 100%-saturated cumulative curve, drop to the T2 axis: that T2 is T2cutoff. Repeat over multiple plugs, take field average (Fig 3.9).

### Spectral BVI (SBVI) - alternative bound-water model
Addresses failure of fixed cutoff when large pores hold surface-film/micro-irregularity bound water (coarse sands, North Sea chalks). Every bin contributes a bound fraction via weighting function W_i (0<=W_i<=1). Halliburton generic coefficients from 340 sandstone + 71 carbonate samples: b=1, m=0.0618 per ms (sandstone), m=0.0113 per ms (limestone). Book best practice: compute both CBVI and SBVI and take the LARGER as BVI (valid for b=1 functions). Worth implementing as an option in SandiBumi; default remains cutoff BVI.

Equations:
1/W_i = m*T2_i + b (eq 3.24); W_i = 1 for T2_i <= T2_k where W would exceed 1; SBVI = sum_{i=1..n} W_i * phi_i (eq 3.25). Motivation: 1/Swirr_core = m*T2gm + b linear on core data (Fig 3.13).

Inputs: Array: T2 distribution. Scalars: m (1/ms), b (dimensionless, default 1).
Outputs: SBVI (v/v); BVI_final = max(CBVI, SBVI) if option enabled.
Calibration: Solve least squares system eq 3.27 across s core samples: Swirr_j * phi_j = sum_i W_i * phi_{j,i}, with Swirr_j from centrifuge/porous-plate desaturation at chosen Pc and phi_{j,i} the sample's binned NMR porosities. Yields field-specific m,b.

### Total & effective NMR porosity (MSIG, MPHI, MCBW) and gas effect
MPHI (effective, TE=1.2 or 0.9 ms, long TW full polarization) and MCBW (clay-bound, TE=0.6 ms, short TW=20 ms partial polarization) come from the dual-TE/TW total-porosity activation; MSIG = MPHI + MCBW is total porosity. MPHI is calibrated to HI=1 water; matrix/dry-clay OH hydrogen is invisible. Error sources (Ch.3 p.66-67): HI<1 (gas, light HC) and incomplete polarization (long-T1 fluids, TW too short) make MPHI read LOW; TE too long loses fast-T2 components (MPHI and MCBW low). In gas zones density porosity reads HIGH (low rho_fluid) while MPHI reads LOW - the divergence DPHI>MPHI is a gas flag, and the combination yields gas-corrected porosity. The book prescribes correcting phiT and phie via TDA before use (Ch.7 p.137); the standard closed-form alternative is the Density-NMR (DMR) combination - spec from standard literature (Freedman et al., SPE 49097, 1998), NOT in the reference file: PHIT_DMR ~= 0.6*DPHI + 0.4*TCMR for typical gas HI/polarization (exact weights lambda = f(HI_g, polarization factor)).

Equations:
MSIG = MPHI + MCBW. Apparent porosity: MPHI_apparent = sum over fluids of S_f * HI_f * (1 - exp(-TW/T1_f)) * phiT (polarization/HI response). DMR (standard lit): PHIT = (DPHI*(1 - HI_g*P_g*(rho_ma-rho_g)/(rho_ma-rho_f)) ... practical form PHIT_DMR = lambda*DPHI + (1-lambda)*MPHI with lambda ~0.6 default; gas flag: DPHI - MPHI > threshold.

Inputs: Scalars per depth: MPHI, MCBW, MSIG (or computed from array), DPHI (density porosity, from density module), TW, TE. Zone scalars: HI_gas, T1_gas, rho_g.
Outputs: PHIT_NMR, PHIE_NMR, CBW volume, gas-flag curve, gas-corrected PHIT_DMR (optional).
Calibration: Core He-porosimetry vs NMR core porosity (book: agreement better than 1 p.u. when TE short, TW long, HI=1). QC: MPHI<=MSIG always; MPHI vs ND-crossplot porosity overlay.

### Timur-Coates (free-fluid) permeability KTIM
Primary NMR permeability estimator; works in hydrocarbon-bearing rock because FFI/BVI is insensitive to non-wetting phase (as long as BVI excludes HC contribution). Book eq 3.28: k = [ (phi/C)^2 * (FFI/BVI) ]^2 , equivalently k = (phi/C)^4 * (FFI/BVI)^2 - the industry KTIM form with exponents a=4 (porosity) and b=2 (ratio), both exposable as parameters. phi = MPHI (use corrected porosity in gas). Default C=10 (book Ch.6 p.124 states wellsite defaults T2cutoff=33 ms and C=10); book example calibration yields C=6.2 (Fig 1.11). Caveats: unflushed gas -> MPHI low and BVI high -> k underestimated; heavy oil counted as BVI -> k underestimated; fractures not represented (matrix perm only).

Equations:
KTIM [mD] = ( (MPHI/C)^2 * (FFI/BVI) )^2, MPHI/FFI/BVI in p.u. or consistently in v/v with C rescaled; generalized: KTIM = (MPHI/C)^a * (FFI/BVI)^b with defaults a=4, b=2, C=10.

Inputs: Scalars per depth: MPHI, FFI, BVI (from partition module). Zone scalars: C, a, b.
Outputs: KTIM (mD) continuous curve.
Calibration: Book Fig 1.11: crossplot (Coreperm/MPHI)^(1/4) on x vs sqrt(FFI/BVI) on y from core-NMR or log-vs-core-perm pairs; slope of line through origin gives 1/C... practically regress log10(k_core) = 4*log10(MPHI) + 2*log10(FFI/BVI) - 4*log10(C) to solve C (and optionally a,b). Use Klinkenberg-corrected core perm at overburden stress.

### SDR (Mean-T2) permeability KSDR
Kenyon/Schlumberger-Doll-Research model, book eq 3.29: k = a * T2gm^2 * phi^4 with NMR effective porosity. Valid ONLY in water zones: any oil/gas skews T2gm toward bulk-fluid values (light HC raise, unflushed gas lowers vs liquid) and error is NOT correctable - book explicitly says the model fails in hydrocarbon-bearing formations. Also matrix-only (fails in fractures). The book gives the functional form but no default a; standard literature default (Kenyon et al. 1988): a = 4.0 for sandstone (k in mD, T2gm in ms... strictly a=4 with T2 in ms and phi in fraction gives mD-scale results; carbonates a ~ 0.1-0.4) - flag a as zone parameter with sandstone default 4.0, carbonate default 0.1, always core-calibrated.

Equations:
KSDR [mD] = a * (T2LM)^2 * (PHIE_NMR)^4; generalized KSDR = a * T2LM^b * phi^c, defaults b=2, c=4.

Inputs: Scalars per depth: T2LM (ms), PHIE_NMR (v/v). Zone scalars: a, b, c.
Outputs: KSDR (mD).
Calibration: Regress log10(k_core) vs log10(T2LM) and log10(phi) on 100%-water-saturated core NMR + perm data; or fix b=2,c=4 and solve a as geometric-mean ratio k_core/(T2LM^2 phi^4).

### T2 log-mean (T2LM / T2gm)
Porosity-amplitude-weighted geometric mean of the distribution; the 'size parameter' of the SDR model and a general texture curve. Book uses T2gm throughout (eq 3.29, Fig 3.13, DIFAN free-fluid-window means) without printing the formula; formula is standard literature. SandiBumi should compute T2LM of the full distribution and optionally of a T2 window (e.g. free-fluid window only, as DIFAN does, or above the clay cutoff).

Equations:
T2LM = exp( sum_i phi_i * ln(T2_i) / sum_i phi_i ) = 10^( sum_i phi_i*log10(T2_i)/sum_i phi_i ). Windowed variant restricts i to T2_i within [T2min_win, T2max_win].

Inputs: Array: T2 distribution + bin times. Optional scalars: window bounds.
Outputs: T2LM (ms) curve; optional T2LM_FFI (free-fluid window).
Calibration: None (deterministic); QC against vendor-delivered T2LM curve.

### Swirr from BVI
Irreducible water saturation directly from the bound-fluid volume. Effective-porosity system: Swirr = BVI/MPHI. Total-porosity system (book eq 7.12, used inside MRIAN): Swirr = (phiT*Swb + BVI)/phiT = (MCBW + BVI)/MSIG, i.e. clay-bound water counts as immobile in the total system. Valid where the zone is at/above transition (water not displaced below capillary-bound level); in transition zones BVI-based Swirr underestimates actual Sw. Also gives BVW-type quick-look: zone produces water-free if Sw(from resistivity) ~= Swirr(NMR).

Equations:
Swirr_e = BVI / MPHI; Swirr_t = (MCBW + BVI) / MSIG; bulk-volume-irreducible check BVW_irr = Swirr_e * MPHI = BVI.

Inputs: Scalars per depth: BVI, MPHI, MCBW, MSIG.
Outputs: SWIRR curve (both systems, selectable).
Calibration: Centrifuge or porous-plate core desaturation Swirr at field-appropriate Pc (height above free water) vs log Swirr; adjust T2cutoff (or SBVI m,b) until log Swirr matches core - i.e. Swirr calibration IS the cutoff calibration.

### Pseudo capillary pressure (pseudo-Pc) from T2 distribution
NOT explicitly formulated in the reference file - the book gives the physical basis only (Ch.3 Figs 3.4-3.6: T2 distribution overlays MICP pore-throat distribution when shifted by effective relaxivity rho_e; its ref 17 = Marschall, Gardner, Mardon & Coates 1995, SCA-9511, is the source method). Spec from standard literature (Marschall et al. 1995; Volokitin et al. 2001, Petrophysics 42(4)): since T2 (surface relaxation) is proportional to pore size and Pc is inversely proportional to throat radius, Pc maps to 1/T2. Build the pseudo-Pc curve by (1) converting each bin: Pc_i = Kappa / T2_i, where Kappa [psi*ms] is the T2-to-Pc scaling constant lumping surface relaxivity, throat/body ratio and interfacial tension; (2) forming the saturation axis from the cumulative distribution sorted large-T2-first: Sw(Pc_i) = 1 - (cumulative phi from largest T2 down to bin i)/phiT. Fluid-pair conversion for reservoir use: Pc_res = Pc_lab * (sigma cos theta)_res/(sigma cos theta)_lab. Output per depth: a pseudo-Pc curve (array), plus derived Swirr at a reference Pc and a per-depth J-function if rock density of k,phi available.

Equations:
Pc_i = Kappa / T2_i (air-brine lab equivalent); SHg or Sw_i = 1 - SUM_{j: T2_j >= T2_i} phi_j / phiT; Kappa calibration: minimize misfit between pseudo-Pc(Sw) and core MICP Pc(SHg) (after closure correction and Hg->air-brine conversion: Pc_ab = Pc_Hg * (sigma cos theta)_ab/(sigma cos theta)_Hg ~ Pc_Hg * 72*cos0/(480*cos140) ~ Pc_Hg/5.1) for plugs with both NMR and MICP. Typical sandstone Kappa order 1000-3000 psi*ms lab air-brine (field-specific).

Inputs: Array: T2 distribution + bin times, phiT. Zone scalars: Kappa, sigma-cos-theta pairs (lab and reservoir), optional closure-correction handled at calibration time.
Outputs: Pseudo-Pc(Sw) array per depth (or at user depths), Sw at reference height/Pc, saturation-height inputs.
Calibration: Plug-by-plug: overlay NMR T2 cumulative (converted) on same plug's MICP after closure correction; solve Kappa by least squares in log(Pc); validate against centrifuge air-brine Pc and centrifuge Swirr. Note book Fig 3.4-3.5: effective relaxivity rho_e is found at max correlation of shifted distributions - equivalent single-parameter shift, Kappa = 2*sigma*cos(theta)*rho_e*(body/throat ratio) conceptually.

### Hydrocarbon typing: dual-TW (TDA/DSM), dual-TE (SSM/DIFAN/EDM) - ADVANCED/LATER
Flagged advanced/deferred for SandiBumi phase-later; equations captured for completeness. (a) T1-weighted dual-TW: two echo trains, TW_L and TW_S; water fully polarized at TW_S so the train difference isolates light HC. DSM differences the two T2 distributions; TDA differences echo trains then fits (more robust). TDA appendix math: dM(t) = M_oil(0)exp(-t/T2oil)*dAlpha_o + M_gas(0)exp(-t/T2gas)*dAlpha_g, with polarization functions dAlpha_f = exp(-TW_S/T1f) - exp(-TW_L/T1f) (eqs 6-A.6..8,10); apparent-to-true porosity: phi*_f = phi_f * HI_f * dAlpha_f (6-A.12/13). Procedure: acquire dual-TW; estimate T1,T2,HI of oil/gas at reservoir P,T; subtract trains; fit for phi*_oil, phi*_gas; correct to phi_oil, phi_gas; then water porosity and corrected phie. Requires DPhi >= ~1.5 p.u. and strong T1 contrast; best in high-porosity water-wet rock with light HC. (b) Diffusion-weighted dual-TE: 1/T2 = 1/T2int + C*Da*(gamma*G*TE)^2/12, C=1.08 for MRIL (eq 6.1). SSM = qualitative shift comparison of TE_S vs TE_L distributions. DIFAN (quantitative, oils 0.5-35 cp): compute T2 geometric means of free-fluid windows of both distributions -> solve eqs 6.2-6.3 simultaneously for T2int and Da; enter 1/T2int vs Da/Dw crossplot bounded by Swa=100% line (through bulk-water point, intercept 1/T2int = 0.04 /ms) and Swa=0% line (through (Doil/Dw, 1/T2bulk,oil)); read Swa; then Sw = (Swa*FFI + BVI)/(FFI+BVI) (eq 6.4). EDM: choose long TE so T2DW = 12/(C*Dw*(gamma*G*TE)^2) (eq 6.6) sits below minimum expected T2oil (design rule 2*T2DW << min T2oil, eq 6.9); any signal beyond T2DW is unambiguously oil. Bulk-fluid property correlations needed as inputs (Ch.3): T1bulk_water ~= 3*(T_K/(298*eta)) s; T1bulk_gas ~= 2.5e4*rho_g/T_K^1.17 s; T1bulk_deadoil ~= 0.00713*T_K/eta s; Dw ~= 1.2*(T_K/(298*eta))e-5 cm2/s; Do ~= 1.3*(T_K/(298*eta))e-5; Dg ~= 8.5e-2*(T_K^0.9/rho_g)e-5.

Equations:
See summary (eqs 6.1-6.9, 6-A.1..13, 3.3-3.14 of the book).

Inputs: Arrays: two T2 distributions (dual-TE) or two echo trains/distributions (dual-TW). Scalars: TE_S, TE_L, TW_S, TW_L, G (gradient, tool chart f(tool,temperature)), gamma, C=1.08, fluid properties (eta, rho_g, T, P -> T1/T2/HI/D per fluid), FFI, BVI.
Outputs: phi_oil, phi_gas, phi_water (TDA); Swa and Sw (DIFAN); oil flag beyond T2DW (EDM).
Calibration: Fluid-property correlations vs PVT samples; job planning of TE/TW is acquisition-side, so SandiBumi only needs the interpretation given delivered dual arrays.

### MRIAN - NMR-enhanced water saturation (dual-water with resistivity)
Bonus method fully specified in the file, directly relevant to Jauhar's LRLC work: combines deep resistivity with MRIL MCBW/MPHI in a dual-water model for virgin-zone Sw. Swb (clay-bound water saturation) = (phiT - phie)/phiT = MCBW/MSIG (eq 7.5; the MRIL primary estimate is compared with conventional Swb estimates and the minimum taken). Cw from Rw; clay-water conductivity Ccw = 0.000216*(T-16.7)*(T+504.4), T in degF (eq 7.2). Coates W-exponent replaces m,n: Ct = (phiT*SwT)^W * [Cw*(1 - Swb/SwT) + Ccw*Swb/SwT] (eq 7.4), solved for SwT. W estimated empirically: W_Q = 1.65 + 0.4*(BVI/MPHI) (eq 7.14), clamped between W_i (irreducible, eq 7.11 with Swirr=(phiT*Swb+BVI)/phiT eq 7.12) and W_w (100% wet, eq 7.10); if W_Q>W_w zone flagged wet, if W_Q<W_i flagged at irreducible. QC crossplot Cwa = 1/(Rt*phiT^W) vs Swb bounded by Cwa=Cw+Swb(Ccw-Cw) (SwT=100%) and Cwa=Swb^W*Ccw (irreducible). Outputs: phiwT = SwT*phiT, CBVWE = phiwT - MCBW, phih = phie - CBVWE (eqs 7.15-7.17).

Equations:
See summary (eqs 7.1-7.17).

Inputs: Scalars per depth: Rt (deep resistivity), MSIG (phiT), MPHI (phie), MCBW, BVI. Zone scalars: Rw (Cw), formation T, optional fixed W.
Outputs: SwT, Sw_effective, CBVWE, hydrocarbon pore volume phih, wet/irreducible zone flags.
Calibration: Rw from water samples/SP/Archie transforms/BVI; phiT & phie must be TDA/DMR-corrected in gas zones before use; QC crossplot per Fig 7.2.


## Notes
SEARCH COVERAGE: Globbed "D:\01. Work\00. Guidebook\01. Reference" for *NMR*, *CMR*, *MREX*, *T2*, *Coates*, *magnetic resonance*, *Dunn*, *Prammer* and listed all PDFs in the folder plus a one-level listing of "D:\01. Work\00. Guidebook". Exactly ONE NMR reference exists: the 1999 Coates/Xiao/Prammer Halliburton book (which is also the canonical text the task named). No CMR/MREX service-company brochures, no Dunn et al., no NMR-specific core reports found. All equations above with book eq numbers are read directly from the file (Ch.1, 3, 6, 7; PDF page = printed + 18).

FROM STANDARD LITERATURE, NOT FROM FILES (as instructed, no invented citations - these are standard published sources): (1) SDR coefficient default a=4 sandstone / ~0.1 carbonate (Kenyon et al. 1988, SPE Formation Evaluation - the book cites Kenyon as its ref 14 but prints no default a); (2) T2LM formula (standard geometric mean; book uses T2gm without printing it); (3) DMR gas-corrected porosity weights (Freedman et al. 1998, SPE 49097); (4) pseudo-Pc Pc=Kappa/T2 workflow (Marschall et al. 1995 SCA-9511 - which IS the book's own reference 17, and Volokitin et al. 2001); (5) modern tool bin counts 27-64 (CMR-Plus/MREX delivery practice; the book documents only MRIL's 12 bins).

FILE-CONFIRMED DEFAULTS for SandiBumi zone parameters: T2cutoff_clay=4 ms; T2cutoff=33 ms sandstone, 92 ms carbonate; Coates C=10 (book wellsite default, p.124), a=4, b=2; SBVI m=0.0618/ms ss, 0.0113/ms ls, b=1; DIFAN C=1.08; Ccw(T) formula; W_Q=1.65+0.4*BVI/MPHI; total-porosity activation TE pairs 0.6/1.2 ms, TW 20 ms partial.

IMPLEMENTATION ORDER SUGGESTION: (1) array-curve storage + partition + T2LM (pure array math, no calibration); (2) KTIM/KSDR + Swirr (zone parameters); (3) pseudo-Pc (needs core MICP calibration UI); (4) MRIAN (needs resistivity + Rw plumbing); (5) hydrocarbon typing dual-TW/TE (advanced, needs dual-activation array pairs + fluid-property correlations - defer as the task instructed). Mahakam Delta context: sandstone defaults apply (33 ms), but Jauhar's silty LRLC sands often need lower cutoffs (silt-bound water) - expose per-zone T2cutoff and consider the SBVI max(CBVI,SBVI) practice which handles silt microporosity better; DIFAN example in the book (Fig 6.15) is an Indonesian well with TE 1.2/4.8 ms.

DATA-MODEL NOTE FOR DUCKDB: incremental-porosity arrays are dimensionless v/v and bin times logarithmic - store bin_times as its own metadata array once per curve-set, never per row; DLIS import must map FRAME array channels (one channel, element count = bins) and LAS 2.0 vendor exports typically explode bins into BIN01..BINnn columns which the importer should re-pack into the array column.
