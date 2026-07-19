# Geomechanics: pore pressure prediction and borehole stability (1D MEM) for SandiBumi compute modules

## Files
- D:\01. Work\00. Guidebook\221102 Borehole Failure Criteria (final).pptx
  37-slide deck 'Borehole Failure Criteria' (Eril S. Lanin, Geomechanics Study PHR-LAPI ITB, Nov 2022). Full equations (extracted from slide XML/OMML math) for Mohr-Coulomb, Hoek-Brown, inscribed/circumscribed Drucker-Prager, Modified Wiebols-Cook, Modified Lade, Mogi-Coulomb failure criteria; Kirsch borehole stresses for a vertical well; closed-form collapse (Pwb) and fracturing (Pwf) mud pressures for 3 stress-ordering cases per criterion. Primary source for method 9.
- D:\01. Work\00. Guidebook\05. Geomechanics\GEOMECHANICS BEKASAP SOUTH FIELD FINAL REPORT.pptx
  77-slide LAPI ITB final report, Bekasap South field (Rokan PSC). Complete 1D MEM template: data conditioning (GRN mean-SD normalization, RHOB-from-GRN regressions per formation, RHOB shallow extrapolation vs TVD, Faust DT synthetic DT=a(ZR)^(1/6) with a=490/513/600, DTSM=10^(A+B*DT) regressions), Sv from RHOB integration + power-law fit, hydrostatic 0.45 psi/ft, Eaton PP exponent=1 on DTCO NCT with RFT calibration and manual ppg shifts, Thiercelin-Plumb vs Eaton Shmin with LOT calibration (eps_Hmax=0.000577, eps_hmin=0, alpha=1), McNally UCS + Lal friction angle + dyn-to-static 0.74, SHmax=1.05*Shmin, collapse MW via MC/Mogi/Drucker/Lade, mud window, max injection pressure (caprock + CFF fault criterion mu=0.3). Per-formation UCS/phi/E/nu min-mean-max tables (Petani...Lower Red Bed). Appendix repeats all failure-criteria equations with effective-stress (P0) forms.
- D:\01. Work\00. Guidebook\05. Geomechanics\GEOMECHANICS SERUNI FIELD FINAL REPORT.pptx
  85-slide LAPI ITB final report, Seruni field. Same 1D MEM template as Bekasap: Eaton exponent factor=1, DTCO NCT, Biot=1, Thiercelin-Plumb/Eaton FG split at top of underpressure, SHMAX=1.05*Shmin, McNally/Lal strength, collapse-MW workflow slide listing inputs per sub-module.
- D:\01. Work\00. Guidebook\05. Geomechanics\GEOMECHANICS PEMBURU FIELD FINAL REPORT.pptx
  84-slide LAPI ITB final report, Pemburu field. Same template; Kirsch appendix at ~L150 of extracted text.
- D:\01. Work\00. Guidebook\05. Geomechanics\GEOMECHANICS TILAN FIELD FINAL REPORT.pptx
  90-slide LAPI ITB final report, Tilan field. Same template; notes abundant RFT at Pematang, FIT available/no LOT, World Stress Map SHmax azimuth 30-45 deg in Central Sumatra Basin, Kotabatak caliper-washout calibration of SHmax/Shmin=1.05.
- D:\01. Work\00. Guidebook\05. Geomechanics\Gulamo_Field_Geomechanics.pptx
  85-slide LAPI ITB report, Gulamo field. Same template; explicit fitted trends: Sv=0.355378*TVD^1.11174 (psi), Shmin power laws per litho-interval (e.g. shale Petani-Telisa Shmin=0.754374*TVD^0.9980028), VSH>0.6 sand/shale cutoff for Shmin trend separation.
- D:\01. Work\00. Guidebook\05. Geomechanics\Jorang_Field_Geomechanics.pptx
  84-slide LAPI ITB report, Jorang field. Same template; RFT-trend PP calibration (Telisa uses Ubi field 0.416 psi/ft trend), underpressure 0.204-0.436 psi/ft, per-well UCS/phi tables.
- D:\01. Work\00. Guidebook\05. Geomechanics\Kopar_Field_Geomechanics.pptx
  78-slide LAPI ITB report, Kopar field. Same template; Eaton(1969) PP with Biot=1, McNally UCS, Lal friction angle, CFF fault injection limit.
- D:\01. Work\00. Guidebook\05. Geomechanics\jorang referensi laporan lamaw.pptx
  40-slide older reference study (Prianto Setiawan, ARM Team Rumbai, Aug 2017): 'Jorang Field Pore pressure & fracture Gradient prediction'. Adds: Sv polynomial Sv(psi)=0.00003*TVD^2+0.7798*TVD-3.0015 with warning never to assume 1 psi/ft; Matthews & Kelly method for shale FG calibrated to FIT trend (FIT = lower bound of FG); Eaton FG for sand with Poisson ratio adjusted until shale-interval Eaton FG matches M&K trend; stress-regression handling below Petani (two FG trend lines); sand FG calibrated against drilling MW with no-loss records; depletion/compartmentalization handling of RFT (min-max PP scenarios); centroid/gas-buoyancy overpressure in Petani gas sand.

## Methods
### 0. Log conditioning / synthetic inputs for MEM
Pre-processing chain used by all LAPI-ITB CSB studies before any geomechanics calc: GR normalization, RHOB repair + surface extrapolation, synthetic DT (Faust) and synthetic DTS (regression), despiking.

Equations:
GRN: mean-SD normalization to reference well (Bekasap example: mean=79, SD=25.9 at Top Bekasap). RHOB repair (badhole/missing): per-formation linear regression RHOB=a+b*GRN (examples: Petani RHOB=1.07472+0.0124274*GRN; Telisa RHOB=1.13393+0.0124485*GRN; Duri-LRB RHOB=2.17049+0.00243209*GRN; Basement RHOB=2.50577+0.00220229*GRN). Shallow RHOB to surface: linear fit vs TVD on first 500-1500 ft of good data (example BESO00014: RHOB=1.73976+0.000164715*TVD). Faust synthetic sonic: DT = a*(Z*R)^(1/6) with a=Faust constant (CSB values used: 513 default, 490 Petani, 600 other intervals), Z depth (ft), R resistivity (ohm.ft); then bias-correct via linear fit to measured DT (example: DT=-15.7747+1.13314*DT_FAUST, CC=0.92). DTS synthetic: DTSM=10^(1.80336+0.00557315*DT) for Petani; DTSM=10^(1.58342+0.00695659*DT) all other intervals (regression from Pager/Rangau/Seruni fields).

Inputs: GR, RHOB, CALI/badhole flag, DT (DTCO), DTSM (if any), deep resistivity, TVD, formation tops, VSH.
Outputs: GRN, RHOB_edited + RHOB_extrapolated (composite to surface), DT_synthetic, DTSM_synthetic, despiked elastic-property inputs.
Calibration: Regression coefficients fitted per field/formation; DT_Faust crossplotted vs wired DT (target CC~0.9); DTSM regression validated against wells with measured DTSM and vendor (PHR) equation; despiking required so per-formation property tables have reasonable ranges (explicit action item in reports).

### 1. Overburden (vertical) stress from RHOB integration
Sv by trapezoidal integration of composite bulk density from surface; fitted to a power-law or polynomial trend per field.

Equations:
Sv(z) = integral 0..z of rho_b*g dz. Field units: Sv[psi] = SUM(0.433891 * rho_b[g/cc] * dTVD[ft]) (1 g/cc = 0.4335 psi/ft). Use RHOB_extrapolated where log absent (surface gap), RHOB_edited where present. OBG[psi/ft]=Sv/TVD; ppg=psi/ft / 0.052. Fitted trends from files: Bekasap Sv=10^(-0.292866+1.07234*log10(TVD)) psi; Gulamo Sv=0.355378*TVD^1.11174 psi; Jorang Sv=0.00003*TVD^2+0.7798*TVD-3.0015 psi. Offshore variant (standard lit.): Sv = 0.4335*rho_w*Zw + integral below mudline; extrapolation alternative (Miller/Traugott): rho(z)=rho_mudline+A0*(z-Zml)^alpha.

Inputs: Composite RHOB (edited + extrapolated), TVD, KB elevation/air gap, water depth (offshore).
Outputs: Sv (psi), OBG (psi/ft and ppg) vs depth; fitted Sv(TVD) trend function for wells without RHOB.
Calibration: No direct measurement; QC by cross-well consistency of OBG profile. File warning: never assume 1 psi/ft (heavily overestimates at CSB depths); plot 1 psi/ft line for reference only.

### 2. Normal compaction trend (NCT) fitting
Fit compaction trend of shale-point DT (preferred) or resistivity vs depth in the known-hydrostatic section; deviation from NCT marks top of over/underpressure and feeds Eaton.

Equations:
Shale discrimination: VSH cutoff (files use VSH>0.6 = shale). Trend forms (standard lit., files fit DTCO but do not print the functional form): (a) log-linear: log10(DT_n)=A-B*z or ln form; (b) asymptotic exponential: DT_n = DT_matrix + (DT_mudline - DT_matrix)*exp(-c*z); resistivity: log10(R_n)=A+B*z. Top of abnormal pressure picked where shale DT departs from NCT (CSB: departure to FASTER DT = underpressure top, e.g. Top Telisa at Bekasap South, Top Petani-B at Jorang, Top Duri at Gulamo).

Inputs: DT (DTCO) or deep resistivity, VSH (shale flag), TVD, known hydrostatic interval for anchoring.
Outputs: DT_n(z) / R_n(z) NCT curves; top-of-abnormal-pressure depth pick.
Calibration: Anchor through shale points in interval confirmed hydrostatic by RFT/MW; interactive fit (SandiBumi should allow manual A/B or graphical drag); best CSB indicator is DTCO (explicit file statement, 'also works for other CSB fields').

### 3. Eaton pore pressure (sonic + resistivity)
Eaton (1975) ratio method: pore pressure from deviation of measured log from NCT, scaled between overburden and hydrostatic. The CSB studies use exponent 1 on DTCO with Biot=1; standard literature default exponent 3 (sonic) / 1.2 (resistivity).

Equations:
Sonic: Pp = Sv - (Sv - Pn) * (DT_n/DT)^n, n=3 standard (Eaton 1975), n=1 used for all CSB fields (explicit: 'exponent factor = 1'). Resistivity: Pp = Sv - (Sv - Pn) * (R/R_n)^m, m=1.2 standard. Pn = hydrostatic = 0.45 psi/ft * TVD (CSB; 8.6625 ppg; range 0.433 fresh - 0.465 saline). Gradient output: Pp_grad=Pp/TVD; ppg=Pp_grad/0.052.

Inputs: Sv, hydrostatic gradient, DT or Rt with matching NCT, TVD; optional manual shift table (formation, delta-ppg).
Outputs: Shale pore pressure curve (psi, psi/ft, ppg); merged PP profile (shale from Eaton, sand from RFT/trend).
Calibration: RFT/MDT points in sands are authoritative (sand PP is dynamic with production; shale PP stays on Eaton curve); mud weight + drilling events (kick/loss/connection gas) bound the profile; where no RFT in a sand, use Eaton trend calibrated to nearby RFT or analog-field RFT gradient (Jorang: Telisa given Ubi-field 0.416 psi/ft); manual block shifts applied per formation (files: +1.5 ppg below Brown Shale overpressure; -2 ppg correction in BESO00014); gas buoyancy/centroid can push sand above hydrostatic (Petani gas: 0.48 psi/ft, up to 9.2 ppg).

### 4. Bowers effective-stress method (loading + unloading)
Bowers (1995) velocity-effective-stress transform; handles both undercompaction (loading) and fluid-expansion/unloading overpressure. NOT in Jauhar's reference files - spec from Bowers 1995 (SPE Drilling & Completion) / Zoback 2007. Relevant for Mahakam Delta overpressure where unloading occurs.

Equations:
Vp[ft/s] = 1e6/DT[us/ft]. Loading (virgin) curve: V = V0 + A*sigma^B with V0~5000 ft/s (mudline), sigma = vertical effective stress (psi) => sigma = ((V-V0)/A)^(1/B); Pp = Sv - sigma. Unloading curve: V = V0 + A*[sigma_max*(sigma/sigma_max)^(1/U)]^B, with sigma_max = ((V_max-V0)/A)^(1/B) (V_max = velocity at onset of unloading, usually velocity-reversal maximum), U = unloading exponent (U>=1; U=1 collapses to loading curve; typical 3-8). Invert: sigma = sigma_max*(((V-V0)/A)^(1/B)/sigma_max)^U; Pp = Sv - sigma.

Inputs: DT (or Vp), Sv, hydrostatic, A, B (loading constants), U, V_max/sigma_max (unloading), depth of unloading onset.
Outputs: Pp (psi, ppg) valid through both loading- and unloading-generated overpressure.
Calibration: Fit A,B on crossplot of Vp vs effective stress (sigma = Sv - Pp_measured) using RFT/MDT and offset-well data; A,B are basin-specific (Bowers published Gulf of Mexico values; must be refit locally - do not hardcode); detect unloading from velocity reversal with density staying high (velocity-density crossplot); U from data inside the reversal zone.

### 5. Equivalent depth method
Classic vertical effective-stress method: an overpressured shale with log value X carries the same effective stress as the shallower NCT depth with the same X. NOT in reference files - standard literature (Foster & Whalen 1966; Zoback 2007).

Equations:
For depth z with measured DT(z): find z_e on NCT such that DT_n(z_e)=DT(z). Then sigma_e(z) = sigma_e(z_e) = Sv(z_e) - Pn(z_e); Pp(z) = Sv(z) - sigma_e(z_e). Works with any compaction-responsive log (DT, R, RHOB).

Inputs: Measured shale log, NCT function (invertible), Sv(z), Pn(z).
Outputs: Pp in overpressured shale.
Calibration: Same RFT/MW calibration as Eaton. Limitation to encode: valid only for loading/undercompaction; underestimates Pp where unloading occurred (use Bowers there). For CSB underpressure the method is rarely used (Eaton preferred); include as cross-check.

### 6. Fracture gradient / minimum horizontal stress
Four-method family: Eaton-Poisson effective-stress ratio, Thiercelin & Plumb poroelastic-with-strain, Matthews & Kelly, Hubbert & Willis, plus direct Shmin-vs-depth power-law fits. CSB practice: Eaton for hydrostatic (shallow/Petani) intervals, Thiercelin-Plumb for underpressured intervals, calibrated to LOT/FIT.

Equations:
Eaton (1969): Shmin = nu/(1-nu)*(Sv - alpha*Pp) + alpha*Pp, alpha(Biot)=1 in CSB studies, nu from DTCO/DTSM. Thiercelin & Plumb (1994) poroelastic: Shmin = nu/(1-nu)*(Sv-alpha*Pp) + alpha*Pp + E*eps_h/(1-nu^2) + E*nu*eps_H/(1-nu^2); SHmax = same with eps_H and eps_h swapped. CSB calibration: eps_hmin=0, eps_Hmax=0.000577 (fitted so Shmin=LOT at Duri Fm, PAGE00016, alpha=1). Matthews & Kelly (1967): FG = Pp/z + K(z)*(Sv-Pp)/z, K = matrix stress coefficient from local LOT/FIT-vs-depth fit (used for shale FG in 2017 Jorang study; shale interval is where Eaton and M&K are forced to agree). Hubbert & Willis (1957): lower bound Pfrac=(Sv+2Pp)/3, upper bound (Sv+Pp)/2. Direct trend: fit Shmin=10^(A+B*log10(TVD)) per litho-interval, sand vs shale split at VSH 0.6 (Bekasap examples: shale Petani-Telisa 10^(-0.132006+1.00397*log10 TVD); Duri-Menggala 10^(0.59247+0.797578*log10 TVD); Pematang under/over-pressure variants; Gulamo: 0.754374*TVD^0.9980028). Depletion coupling (standard lit.): dShmin/dPp = alpha*(1-2nu)/(1-nu) - sand FG is dynamic with reservoir pressure.

Inputs: Sv, Pp (calibrated), nu, E (static), alpha, eps_h/eps_H (or K(z) table), VSH, TVD.
Outputs: Shmin/FG curve (psi, psi/ft, ppg); SHmax if strains given; per-interval min-max FG table (min from sand/current PP, max from shale/initial PP - explicit report convention).
Calibration: LOT/XLOT (shale) is best; FIT = lower bound (FG must plot above FIT trend); minifrac/step-rate/injection tests for sand; drilling losses/ballooning events; MW-with-no-loss records as lower bound for sand FG; adjust nu (or strains) until log-based FG matches the calibrated shale trend (2017 Jorang procedure); handle stress regression by separate trend lines above/below the regression marker (Base Petani); check Shmin never exceeds Sv, Eaton used where Thiercelin-Plumb over-predicts (near/above Sv).

### 7. Dynamic elastic moduli from DTC/DTS/RHOB + dynamic-to-static
Standard isotropic dynamic moduli from sonic slownesses and density, converted to static with a field-calibrated factor (CSB lab factor 0.74).

Equations:
Vp=1e6/DTC, Vs=1e6/DTS (ft/s from us/ft). R=DTS/DTC: nu_dyn=(R^2-2)/(2*(R^2-1)). G_dyn=rho*Vs^2 (field units: G[psi]=1.34e10*rho_b[g/cc]/DTS[us/ft]^2). E_dyn=2*G*(1+nu). K_dyn=rho*(Vp^2-(4/3)Vs^2)=E/(3*(1-2nu)). M=rho*Vp^2. Dynamic-to-static: E_static=0.74*E_dyn (CSB crossplot vs lab at RANG00003, single-depth calibration 7277 ft); alternates from literature if lab data exist (e.g. per-lithology regressions); static nu commonly taken = dynamic nu.

Inputs: RHOB, DTC, DTS (measured or synthetic from method 0).
Outputs: E_dyn, E_static, nu, G, K, M vs depth; per-formation sand/shale min-mean-max tables (report deliverable format). CSB magnitudes for QC: E 0.2-3.3 Mpsi mean, nu 0.2-0.5 - 'weak' rock justifying SHmax~Shmin.
Calibration: Lab triaxial static moduli on core (only RANG00003 in CSB study); despike inputs first; validate DTS synthetic before use; report which intervals used synthetic logs.

### 8. Rock strength correlations (UCS, friction angle, cohesion, T0)
Log-based strength: CSB standard is McNally (1987) UCS from DT + Lal (1999) friction angle from Vp, validated against lab UCS. Supplement with per-lithology library from Chang, Zoback & Khaksar (2006) for other basins (Mahakam).

Equations:
McNally 1987 (sandstone, from DT): UCS[MPa]=1200*exp(-0.036*DT[us/ft]) (form as compiled in Chang-Zoback-Khaksar 2006; the CSB decks name the method, equation rendered as image). Lal 1999 (shale, Vp in km/s): sin(phi)=(Vp-1)/(Vp+1) => phi=asin((Vp-1)/(Vp+1)); cohesion S0[MPa]=5*(Vp-1)/sqrt(Vp); UCS=2*S0*cos(phi)/(1-sin(phi)). Additional standard options to ship (Chang-Zoback-Khaksar 2006 compilation): sandstone UCS=254*(1-2.7*phi_por)^2 MPa (Vernik, phi_por<0.3), UCS=277*exp(-10*phi_por); shale UCS=0.77*(304.8/DT)^2.93 (Horsrud 2001), UCS=1.35*(304.8/DT)^2.6, UCS=10*(304.8/DT-1) (Lal); carbonate UCS=(7682/DT)^1.82/145 MPa (Militzer-Stoll), UCS=143.8*exp(-6.95*phi_por) limestone. Friction angle alternatives: Plumb (1994) sandstone from Vsh/porosity; default phi=30 deg. Tensile strength T0~0 (explicit CSB assumption; optionally UCS/10-UCS/12). Derived: q=tan^2(45+phi/2), mu_i=tan(phi), S0=UCS/(2*sqrt(q)).

Inputs: DT (us/ft) or Vp, porosity, VSH/lithology flag; lithology-based correlation selector.
Outputs: UCS, phi (deg), mu_i, S0, T0 curves + per-formation sand/shale statistics tables. CSB ranges for QC: UCS 33-19013 psi, phi 6.8-42.1 deg across Petani-Lower Red Bed.
Calibration: Core UCS/triaxial where available (CSB: single well RANG00003; McNally matched lab points); despike DT first; take mu_i for stability from AVERAGE shale friction angle per interval (explicit report action item); flag anomalously high phi (Menggala note).

### 9. Borehole stability: Kirsch stresses + Mohr-Coulomb / Mogi-Coulomb / Drucker-Prager / Modified Lade / (Hoek-Brown, Wiebols-Cook) -> collapse mud weight & mud window
Vertical-well analytical stability: Kirsch wall stresses, six failure criteria with closed-form minimum (collapse) and maximum (fracturing) wellbore pressures over 3 principal-stress-ordering cases; mud window = collapse MW to Shmin/breakdown. Fully specified in the 2022 PHR-LAPI ITB deck.

Equations:
Kirsch at wall (vertical well, effective stresses): sigma_rr=dP=Pw-Pp; sigma_theta=SHmax+Shmin-2Pp-2(SHmax-Shmin)cos2theta-dP-sigma_dT; sigma_zz=Sv-2nu(SHmax-Shmin)cos2theta-Pp; max hoop (theta=90 from SHmax): 3SHmax-Shmin-2Pp-dP; min hoop (theta=0): 3Shmin-SHmax-2Pp-dP. Define A(D)=3SHmax-Shmin-2Pp, B(E)=Sv+2nu(SHmax-Shmin)-2Pp (collapse side), F=3Shmin-SHmax, G=Sv-2nu(SHmax-Shmin) (fracture side). Mohr-Coulomb (q=tan^2(45+phi/2), C=UCS-Pp(q-1)): collapse cases Pwb1=(B-C)/q [sz>=st>=sr], Pwb2=(A-UCS)/(1+q), Pwb3=A-UCS-q*B; fracturing Pwf1=UCS+qG, Pwf2=(UCS+qF)/(1+q), Pwf3=(UCS-G)/q+F; Pmud=Pwb+Pp; take max of collapse roots / min of frac roots over valid cases. Mogi-Coulomb (a'=2*S0*cos phi, b'=sin phi; invariants I1, I2): collapse closed form Pwb1=(3D+2b'M-sqrt(L+12M^2+b'DM))/(6-2b'^2) with L=D^2(4b'^2-3)+E^2-DE(4b'^2-12), M=a'+b'(E-2Pp) (deck also gives N=M+b'D variant); analogous roots cases 2-3 and tensile cases. Drucker-Prager (J2^0.5=k+alpha_dp*J1): inscribed k=3*UCS*cos phi/(2*sqrt(q)*sqrt(9+3sin^2 phi)), alpha_dp=(3 sin phi)/sqrt(9+3 sin^2 phi); circumscribed k=sqrt(3)*UCS*cos phi/(q*(3-sin phi))... (=6*S0*cos phi/(sqrt(3)(3-sin phi))), alpha_dp=(3*sqrt(2)... ) 6 sin phi/(sqrt(3)(3-sin phi)); collapse from quadratic in Pw (deck form Pwb=(3A-sqrt(36k+l(A+B-3Pp)^2-3A-2B^2 ...))/6 - implement by solving J2=k+alpha*J1 numerically; safer than transcribed closed form). Modified Lade ((I1')^3/I3'=27+eta; S=S0/tan phi; eta=4 tan^2 phi (9-7 sin phi)/(1-sin phi)): with F_l=Pp-B-S, G_l=A(Pp-S)-(Pp-S)^2, H_l=A+B+3S-3Pp: Pwb=(A-sqrt(A^2-4*(G_l-H_l^3/((27+eta)*F_l^2))))/2. Hoek-Brown: sigma1=sigma3+UCS*sqrt(m*sigma3/UCS+s); collapse/frac = smaller/larger quadratic roots per case. Mud window: lower bound = max(Pp, collapse Pw of chosen criterion); upper bound = min(Shmin, breakdown Pwf). SHmax model: SHmax=1.05*Shmin (CSB assumption, Anderson normal-faulting regime Sv>>SHmax>Shmin). Convert Pw to ppg: MW=Pw/(0.052*TVD).

Inputs: Sv, Shmin, SHmax (or ratio), Pp, nu, UCS, phi (S0, mu_i), T0, TVD; criterion selector; optional thermal term sigma_dT.
Outputs: Collapse mud weight per criterion (Mohr-Coulomb, Mogi-Coulomb, Drucker inner/outer, Modified Lade compared side-by-side in reports), breakdown pressure, safe mud-weight window track (min-mean-max per interval), max injection pressure (see method 11).
Calibration: Compare predicted collapse MW vs actual MW per well against caliper washouts/breakouts (wells drilled below predicted collapse MW should show enlargement, above should be in-gauge); tune SHmax/Shmin ratio to make this consistent (Kotabatak: 1.05); Mogi-Coulomb found most applicable for CSB Petani-Lower Red Bed; report requires MC vs Mogi vs Lade comparison column plus collapse-vs-actual-MW column. Note mu (fault friction, ~0.6 Byerlee) vs mu_i (intact internal friction) distinction is called out explicitly in the deck.

### 10. Breakout / tensile (drilling-induced fracture) analysis
Interpret caliper/image breakouts and DITFs to constrain SHmax magnitude and orientation. Files use caliper washout vs collapse-MW comparison only (no image logs in CSB); full breakout-width inversion spec'd from Zoback 2007.

Equations:
Breakout half-width: breakout occurs where sigma_theta(theta) >= rock strength; wall hoop stress sigma_theta(theta)=SHmax+Shmin-2Pp-2(SHmax-Shmin)cos2theta-dP. Breakout edges at theta_b: SHmax constraint (Zoback 2007): SHmax=(UCS_eff+2Pp+dP+sigma_dT-Shmin(1+2cos2theta_b))/(1-2cos2theta_b), with 2theta_b=180deg-wBO (wBO=breakout width from image/6-arm caliper); breakout azimuth = Shmin azimuth (SHmax azimuth = breakout azimuth+90). DITF condition: sigma_theta_min<=-T0 => breakdown Pw=3Shmin-SHmax-2Pp-T0(+thermal); DITF presence gives lower bound on SHmax: SHmax>=3Shmin-2Pp-dP-T0. Tensile check in reports: fracturing if sigma_theta_min < sigma_rr. Orientation prior: CSB SHmax azimuth 30-45 deg (World Stress Map, breakout-derived).

Inputs: Oriented 4/6-arm caliper or image log (breakout azimuth + width), MW at logging time, Pp, Shmin, UCS, T0, nu.
Outputs: SHmax magnitude bounds and azimuth; breakout flag/width prediction vs depth; DITF prediction; QC overlay of predicted collapse MW vs caliper enlargement.
Calibration: Image-log breakout picks are primary; where absent (all CSB fields) fall back to caliper-washout consistency method of method 9; use mud weight actually in hole when the enlargement formed; validate azimuth against regional World Stress Map.

### 11. Maximum injection pressure (caprock integrity + fault reactivation, CFF)
Waterflood/dumpflood/frac screening from the CSB reports: intact caprock bounded by Shmin (upper) and collapse MW (lower); critically-stressed fault limit from Coulomb Failure Function on cohesionless fault planes.

Equations:
Intact caprock: P_inj_max = Shmin (upper bound); collapse mud weight = lower operational bound (Lang et al. 2011 referenced). Fault/fracture reactivation: CFF = tau - mu*(sigma_N - Pp); fault slips when CFF=0; P_inj_max = the Pp that drives CFF to zero on the most critically oriented plane, resolving tau and sigma_N from (Sv, SHmax, Shmin) on the fault plane (3D Mohr circle). CSB assumptions: fault cohesion C0=0, fault friction coefficient mu=0.3 (conservative; Byerlee default 0.6), with per-interval mu variant from log-derived friction angle.

Inputs: Sv, SHmax, Shmin (magnitudes + azimuth), Pp, fault plane orientation (strike/dip) or scan over all orientations, mu_fault, cohesion (0).
Outputs: Maximum allowable injection pressure per interval/fault; critically-stressed-fracture flag.
Calibration: mu=0.3 chosen conservatively for CSB; calibrate against observed SSI (subsurface-injection) events, step-rate tests, and injection history without losses; Shmin bound from LOT-calibrated FG (method 6).


## Notes
SOURCE SITUATION: Jauhar's geomechanics references live in "D:\01. Work\00. Guidebook\05. Geomechanics" (8 pptx field reports) plus "D:\01. Work\00. Guidebook\221102 Borehole Failure Criteria (final).pptx" - nothing under "01. Reference" itself. All are pptx; per task notes they can't be Read directly, but since they are the ONLY geomechanics sources I extracted their text programmatically (unzip + a:t/m:t XML runs, incl. OMML equations) to scratchpad - the equations above marked as file-derived are verbatim from that extraction. Some equations rendered as images in the field decks (Eaton PP formula, McNally, Lal, elastic moduli) are only NAMED in files; their canonical forms are supplied from standard literature. NOT in the library at all (spec'd purely from standard published literature, flagged per method): Bowers loading/unloading (Bowers 1995, SPE Drilling & Completion), equivalent depth (Foster & Whalen 1966 / Zoback 2007), Hubbert & Willis 1957, Matthews & Kelly 1967 K(z) formulation, breakout-width SHmax inversion (Zoback 2007 Reservoir Geomechanics), UCS correlation library (Chang, Zoback & Khaksar 2006, J. Pet. Sci. Eng.), Horsrud 2001. No Zoback book, no Eaton/Bowers papers on disk. CONTEXT WARNING FOR SPEC: all file calibrations are Central Sumatra Basin (Rokan/PHR-LAPI ITB waterflood context) - UNDERpressured reservoirs, normal-faulting regime, Eaton exponent 1, hydrostatic 0.45 psi/ft, SHmax=1.05*Shmin, dyn-to-static 0.74, eps_Hmax=0.000577, fault mu=0.3, Faust a=490/513/600, VSH cutoff 0.6. Jauhar's own Mahakam Delta work is the opposite regime (compaction-disequilibrium + unloading OVERpressure), so SandiBumi should ship these CSB values as a named preset ("CSB/Rokan") with literature defaults (Eaton n=3 sonic /1.2 res, Bowers enabled, alpha adjustable, SHmax ratio free) as the general default. IMPLEMENTATION HINTS: every method chain is depth-vector math over TVD with per-formation parameter tables + point-data calibration overlays (RFT/MDT, LOT/FIT, MW, core UCS) - fits DuckDB well/curve model; the report deliverable format seen in files is min-mean-max per formation split sand/shale (VSH 0.6) which should be an output table; mud window track = Pp / collapse MW / Shmin / Sv with criterion selector; implement Drucker-Prager collapse by numeric root-find rather than the deck's transcribed closed form (extraction shows garbling); Mohr-Coulomb/Mogi/Lade closed forms are complete in the 221102 deck (slides 18-36, 51-55, 69-74). Extracted text files kept in scratchpad pptx\\*.txt this session (bfc_math.txt, bekasap.txt, seruni.txt, jorang.txt, jorang_ref.txt, gulamo.txt, kopar.txt, pemburu.txt, tilan.txt) - they are temp files and will not persist.
