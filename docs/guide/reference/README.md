<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Module reference

One page per petrophysics module, generated from the same manifests the application builds its parameter panes from — descriptions, defaults, sources, ranges and pre-run checks here are exactly what the running application enforces. For the workflow these modules live in, start with the [first hour guide](../first-hour.md).

## Condition

| Module | Title | What it does |
|---|---|---|
| [`despike`](despike.md) | Despike | Replaces samples that stand off their neighbours with the local median. |
| [`smooth`](smooth.md) | Smooth | Averages a curve over a WINDOW stated as a THICKNESS. |
| [`clip`](clip.md) | Clip | Holds a curve inside a range. |
| [`fill_gaps`](fill_gaps.md) | Fill Gaps | Fills holes in a curve that are no wider than MAX_GAP, and marks every sample it invented in <OUT>_FILL. |
| [`flip`](flip.md) | Flip Polarity | Mirrors a curve about a pivot: OUT = 2 x pivot - CURVE. |
| [`normalize`](normalize.md) | Normalize | Maps a curve onto a common reference frame so wells can be compared and pooled. |

## Facies

| Module | Title | What it does |
|---|---|---|
| [`electrofacies`](electrofacies.md) | Electrofacies (K-means) | Unsupervised electrofacies: k-means clusters the samples of THIS well in the space of the supplied curves (each feature z-scored by default, so mixed units are comparable) into K facies. |
| [`gmm_facies`](gmm_facies.md) | Electrofacies (GMM, soft) | Soft electrofacies: a Gaussian mixture model (diagonal covariance, EM, initialized from k-means) clusters this well's samples in the space of the supplied curves. |

## Frame

| Module | Title | What it does |
|---|---|---|
| [`block`](block.md) | Block (Upscale) | Replaces a curve with one value per bed, held across the bed. |
| [`bed_detect`](bed_detect.md) | Bed Detect | Writes the bed number each sample falls in, found from the curve's own steps — the same segmentation Block's AUTO mode uses, exposed on its own so the beds can be LOOKED AT on a log before anything is averaged over them. |

## Lithology

| Module | Title | What it does |
|---|---|---|
| [`midplot`](midplot.md) | Apparent Matrix (MID plot: UMAA / RHOMAA) | Apparent matrix density RHOMAA and apparent matrix volumetric photoelectric factor UMAA — the two axes of the Schlumberger Lith-6 MID plot (crossplot X = UMAA, Y = RHOMAA, then switch on the 'Lith-6 Umaa-Rhomaa MID plot' chart overlay). |

## Permeability

| Module | Title | What it does |
|---|---|---|
| [`perm_wyllie_rose`](perm_wyllie_rose.md) | Permeability — Wyllie-Rose | PERM = (C * PHIE^D / SWE_IRR^E)^2, mD. |
| [`perm_coates`](perm_coates.md) | Permeability — Coates | PERM = (C * PHIE^2 * (1 - SWE_IRR)/SWE_IRR)^2, mD. |
| [`perm_transform`](perm_transform.md) | Permeability — Por-Perm Transform | log10(PERM) = PT_A * PHIE + PT_B — the classic core-derived porosity-permeability regression. |

## Porosity

| Module | Title | What it does |
|---|---|---|
| [`phi_den`](phi_den.md) | Porosity from Density | PHIE = (RHO_MA - RHOB)/(RHO_MA - RHO_FL) - VSH*(RHO_MA - RHO_SH)/(RHO_MA - RHO_FL). |
| [`phi_dn`](phi_dn.md) | Porosity from Density-Neutron | Shale-corrects RHOB and NPHI to 'shale reduced' values, then combines density porosity and neutron porosity: AVERAGE = (PHID+PHIN)/2, GAS_RMS = sqrt((PHID²+PHIN²)/2) for gas-bearing zones. |
| [`phi_dnbk`](phi_dnbk.md) | Porosity from Bateman-Konen N-D Crossplot | The chart-free ANALYTIC neutron-density crossplot (Bateman & Konen 1977, Appendix B), solved as a two-pseudo-mineral system rather than looked up in a transcribed chart. |
| [`phi_son`](phi_son.md) | Porosity from Sonic | Sonic porosity, three transforms each named for what it computes (SB-POR-014). |
| [`phimax`](phimax.md) | Porosity Ceiling (φmax) | Caps an input porosity at a maximum ceiling — the field's compaction-controlled upper limit (the crossplot 'max core porosity' line). |
| [`ssc`](ssc.md) | SSC — Sand-Silt-Clay (Kuttan) | Sand-Silt-Clay model on the N-D crossplot (Kuttan Malay Basin, SandiBumi edit). |
| [`sspw`](sspw.md) | SSPW — Sandstone Petrophysical Workflow | Three-component sandstone workflow (quartz + shale + water). |

## Prep

| Module | Title | What it does |
|---|---|---|
| [`ftemp_grad`](ftemp_grad.md) | Formation Temperature | GRADIENT: FTEMP = TSURF + TGRAD*depth. |
| [`precalc`](precalc.md) | Pre-Calculation (P / T / Rmf / Ct / Cxo) | Reservoir-condition inputs for saturation and SandiMin work, from trend fits: formation temperature = SURF_TEMP + TEMP_GRAD*TVDSS and FPRESS = PSURF + PGRAD*TVDSS, both linear in true vertical depth. |
| [`badhole`](badhole.md) | Bad-Hole QC Flag | BADHOLE = 1 where the borehole departs from gauge or the density correction is large enough to distrust the porosity logs: \|DRHO\| > DRHO_MAX, or \|CALI - bit size\| > DCAL_MAX. |
| [`condflag`](condflag.md) | Data Conditioning Flags | Flags samples whose density/neutron readings should not feed porosity or mineral solving. |
| [`nphimat`](nphimat.md) | Neutron Matrix Conversion | Converts a neutron porosity log recorded in one matrix convention into all three (NPHI_LS / NPHI_SS / NPHI_DOL), using the chartbook porosity-equivalence curves: Por-5 for the CNL thermal tools (NPHI ratio method; TNPH environmentally corrected, with 0 and 250,000 ppm salinity variants) and Por-4 for the epithermal tools — APLC and FPLC (APS) plus the legacy sidewall SNP. |
| [`gascorr`](gascorr.md) | Gas Correction (density, iterated) | Removes the gas effect from RHOB (iterated density-neutron loop): density porosity and Archie SWT are solved from the current density, then RHOB_GC = RHOB + PHIT*(1-SWT)*(RHO_FL - GASDEN) replaces the gas volume with liquid, iterated until PHIT moves less than 1e-4 (max 20 passes; non-converging samples stay MISSING). |
| [`gr_hole_corr`](gr_hole_corr.md) | GR Hole-Size Correction | GR_EC = GR * (1 + K_GR*(CALI - BS)): linear borehole-enlargement correction — gamma rays attenuated by the extra mud annulus are restored. |
| [`nphi_env_corr`](nphi_env_corr.md) | Neutron Environmental Correction | NPHI_EC = NPHI + K_TEMP*(FTEMP - T_REF) + K_SAL*(SALW/100000): linearized formation-temperature and formation-salinity terms. |
| [`rhob_hole_corr`](rhob_hole_corr.md) | Density Hole-Size Correction | RHOB_EC = RHOB + K_RHO*(CALI - HD_REF) for CALI beyond HD_REF: in oversize holes the pad reads too much mud, so density is restored upward using supplied, tool-specific chart values. |
| [`gr_normalize`](gr_normalize.md) | GR Normalization (Two-Point Percentile) | GRN = (GR − Plow_well)·(Phigh_ref − Plow_ref)/(Phigh_well − Plow_well) + Plow_ref. |
| [`log_predict`](log_predict.md) | Synthetic Log (KNN Predict) | Facimage-style synthetic log: trains on the samples of THIS run where TARGET and every supplied predictor are present, then predicts TARGET everywhere the predictors exist by distance-weighted K-nearest-neighbour regression (predictors z-scored; training set decimated to ≤4000 points). |
| [`depth_shift`](depth_shift.md) | Depth Shift | Shifts CURVE by SHIFT metres (+ = the feature moves DEEPER) and resamples it back onto the well's depth grid by linear interpolation. |
| [`splice`](splice.md) | Splice Curves | SPLICED = TOP_CURVE above SPLICE_DEPTH, BOT_CURVE at and below it — the classic run-to-run splice. |

## Rock Typing

| Module | Title | What it does |
|---|---|---|
| [`rocktyping`](rocktyping.md) | Rock Typing (FZI / R35 / PGS) | Per-sample rock-typing indicators from porosity and permeability. |
| [`lucia_rfn`](lucia_rfn.md) | Lucia Rock-Fabric Number (carbonate) | Carbonate rock typing by Lucia rock-fabric number (Jennings & Lucia 2003). |
| [`pittman_rx`](pittman_rx.md) | Pittman Pore-Throat Radii (r10–r75) | Pittman (1992) pore-throat aperture family: writes PR10..PR75 = pore-throat radius (µm) at mercury saturation 10..75 %, each log10 rX = C0 + C1·log10 k + C2·log10 φ% (k mD, φ in PERCENT). |
| [`rt_cutoff`](rt_cutoff.md) | Rock Type from Cutoffs (electrofacies) | Log-domain rock-type class from a Vsh + PHIE cutoff ladder — the electrofacies half of the rock-typing tie-in. |

## Saturation

| Module | Title | What it does |
|---|---|---|
| [`sw_arch`](sw_arch.md) | SW — Archie | Archie (1942) as two separately named methods (SB-SAT-002). |
| [`sw_indo`](sw_indo.md) | SW — Indonesia (Poupon-Leveaux) | 1/RT = (v/RT_SH + PHIE^M/(A*Rw) + 2*sqrt(v*PHIE^M/(A*Rw*RT_SH))) * SW^N, v = VSH^(2-VSH) (FULL), VSH^2 (SIMPLE), VSH^(2-2*VSH) (TAR_SAND). |
| [`sw_sim`](sw_sim.md) | SW — typed Simandoux equations | Each persisted id names one equation. |
| [`sw_rtc`](sw_rtc.md) | SW — RtC (Clay + Capillary Correction) | LRLC RtC method: excess conductivity from clay chemistry and capillary (micropore) water is regressed as Cex = (A_CAP·CAPBW + B_QV·Qv + C0)·PHIT·RSF and removed from the measured conductivity before Archie: Sw = [Rw·(1/Rt − Cex)/PHIT^M]^(1/N). |
| [`sw_imts`](sw_imts.md) | SW — IMTS (Mineral-Textural Scaling) | LRLC IMTS model: Waxman-Smits-family conductivity with the clay charge referenced to the ACTIVE water — Qv_eff = Qv_bulk/(1−Swirr), where Qv_bulk is the clay MASS per unit dry-rock mass times literature CEC constants (kaolinite 8 / illite 25 meq/100g of DRY ROCK), i.e. |
| [`multimin`](multimin.md) | Multimin — Mineral Inversion (retired · use SandiMin) | RETIRED — superseded by SandiMin (Advance ▸ Mineral Solver); running this step now returns a message directing you to SandiMin rather than executing the old fixed 4-component solver. |
| [`sw_height`](sw_height.md) | SW — Saturation-Height | SWH from height above the free-water level. |

## ThinBeds

| Module | Title | What it does |
|---|---|---|
| [`thin_bed_ts`](thin_bed_ts.md) | Thin Beds — Thomas-Stieber | Decomposes bulk VSH into laminar and dispersed shale by comparing the measured (VSH, PHIT) point against the pure-laminated line PHIT = PHI_SD_MAX*(1-VSH) + PHI_SH*VSH and the pure-dispersed line PHIT = PHI_SD_MAX - VSH*(1-PHI_SH). |

## Unconventional

| Module | Title | What it does |
|---|---|---|
| [`toc_passey`](toc_passey.md) | TOC — Passey ΔlogR + Schmoker | Total organic carbon from the Passey (1990) ΔlogR overlay — the separation between deep resistivity and a baselined porosity curve — converted to TOC with the maturity term 10^(2.297−0.1688·LOM). |
| [`kerogen`](kerogen.md) | Kerogen volume + OM-corrected porosity | Converts TOC (weight %) to kerogen VOLUME and corrects total porosity for the organic matter that low-density kerogen inflates on the density log. |
| [`gip`](gip.md) | Gas-in-place (free + Langmuir adsorbed) | Per-sample gas-in-place as gas CONTENT (scf per ton of rock), so it composites like any curve. |
| [`brittleness`](brittleness.md) | Brittleness index (elastic / mineralogical) | Brittleness index (0 ductile .. |

## VSH

| Module | Title | What it does |
|---|---|---|
| [`vsh_gr`](vsh_gr.md) | VSH from Gamma Ray | VSH_GR = (GR - GR_MA) / (GR_SH - GR_MA), with optional non-linear corrections (Stieber, Larionov, Clavier). |
| [`vsh_dn`](vsh_dn.md) | VSH from Density-Neutron | Two-log crossplot VSH: the (RHOB, NPHI) point's position between the clean matrix line and the shale point. |
