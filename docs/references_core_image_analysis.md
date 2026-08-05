# Core image / photo quantitative analysis & image conditioning — literature survey

Compiled 2026-08-05 for SandiBumi (`coreimage.rs`, `petrography.rs`, `registration.rs`, image tracks)
and the LRLC research line. Four verification-focused web sweeps (core-photo quantitative analysis;
image conditioning / colour calibration; thin-bed quantification + LRLC; UV fluorescence + digital
petrography); every citation below was checked against a publisher, society, DOI-registry or archive
record during the search. Items that could NOT be fully verified are flagged inline and gathered in
§7 — do not cite a flagged detail in a deliverable without one confirmation pass through a library
proxy. Entries found independently by two sweeps (Perarnau 2011; Nederbragt 2005/2006; Martin et
al. 2021; Abdlmutalib et al. 2025; Anselmetti et al. 1998) agreed on venue and volume in every case
except one page-range discrepancy on Ehrlich et al. 1984, flagged in §7.

Reading priority (if only a few): **Passey et al. 2006** (the thin-bed framework the CPHOTO work
feeds), **Perarnau 2011** (the direct published antecedent of the CPHOTO traces), **Yadav et al.
2009** (what goes wrong when you threshold an image for net sand), **Selmaoui et al. 2004** (strata
detection on core imagery — the next step past CPHOTO_TEX), **Grove & Jerram 2011** (the published
blue-epoxy analogue), **Haines et al. 2015** (why VPORE_TS reads below helium porosity), and
**Audinno et al. 2016** (the Mahakam LRLC case).

---

## 1. Quantitative traces and pseudo-logs from core photographs

Backs: `coreimage.rs` `extract_core_log` (CPHOTO_DARK / CPHOTO_RED / CPHOTO_TEX / CPHOTO_FLUOR),
the depth strips, the packed-plate lane reader.

- **Perarnau, A., 2011, "Use of Core Photo Data in Petrophysical Analysis": SPWLA 52nd Annual
  Logging Symposium, paper SPWLA-2011-Z.** Extracts average R, G, B along a band down a digital
  core photograph, subdividing the band into rectangles so each channel becomes a depth-indexed
  curve — explicitly for pay counting in a thinly laminated sequence below wireline resolution,
  and explicitly under both natural and UV light. *The most direct published antecedent of the
  whole CPHOTO family, including the read-a-band-and-average-across-it geometry.*
- **Martin, T., Meyer, R., and Jobe, Z., 2021, "Centimeter-Scale Lithology and Facies Prediction
  in Cored Wells Using Machine Learning": Frontiers in Earth Science, v. 9, 659611.** Introduces
  the colour-channel log: per-depth mean AND variance of R, G, B and grey down a depth-registered
  core image; the compressed statistical summary outperforms raw image input (69% vs 57% lithology
  accuracy) at <2 cm resolution, with a facies scheme that includes thin-bedded turbidites.
  *Independent support for darkness + per-channel statistics as the right reduction, and the
  across-core variance term is exactly CPHOTO_TEX.*
- **Meyer, R.G., Martin, T.P., and Jobe, Z.R., 2020, "CoreBreakout: Subsurface Core Images to
  Depth-Registered Datasets": Journal of Open Source Software, v. 5(50), 1969.** Open-source
  Mask-RCNN pipeline segmenting core-tray photographs into individual columns assembled into a
  depth-registered core column. *The nearest prior art to the packed-plate lane problem — but it
  handles trays, not multi-barrel display plates (see §6).*
- **Nederbragt, A.J., and Thurow, J.W., 2005, "Digital Sediment Colour Analysis as a Method to
  Obtain High Resolution Climate Proxy Records," in Francus, P., ed., Image Analysis, Sediments
  and Paleoenvironments: Developments in Paleoenvironmental Research, v. 7, Springer, DOI
  10.1007/1-4020-2122-4_6.** Line scans along the stratigraphic axis of split-core images,
  corrected for uneven light distribution before use; resolves sub-millimetre lamination — finer
  than any spectrophotometer track. *The line-scan-down-the-core method, and the necessity of
  illumination correction before reading a trace.*
- **Nederbragt, A.J., Dunbar, R.B., Osborn, A.T., Palmer, A., Thurow, J.W., and Wagner, T., 2006,
  "Sediment colour analysis from digital images and correlation with sediment composition," in
  Rothwell, R.G., ed., New Techniques in Sediment Core Analysis: Geological Society, London,
  Special Publications, v. 267, p. 113–128.** Camera RGB → CIE L\*a\*b\*, line scans corrected for
  uneven lighting, then calibrated against measured sediment composition. *Treating an RGB core
  trace as a measurement to be calibrated against an independent lab number.*
- **Li, C., Clementi, V.J., Bova, S.C., et al., 2022, "The Sediment Green-Blue Color Ratio as a
  Proxy for Biogenic Silica Productivity Along the Chilean Margin": Geochemistry, Geophysics,
  Geosystems, v. 23, e2022GC010350.** A G/B channel ratio from mm-resolution core reflectance,
  calibrated against measured opal; the ratio form chosen for robustness to overall brightness.
  *CPHOTO_RED's normalised (R−G)/(R+G) ratio is an established, calibrated proxy form.*
- **IODP Expedition methods chapters (e.g. Parnell-Turner, R.E., Briais, A., LeVay, L.J., et al.,
  2025, "Expedition 395 methods": Proceedings of the IODP, 395).** Standard-practice description
  of the Section Half Imaging Logger (calibrated line-scan imaging under fixed LED lighting with
  greyscale card in frame, shading-corrected) and SHMSL colorimetry in L\*a\*b\* at 1–2 cm
  spacing. *The convention that a core colour log is a legitimate depth-indexed measurement, and a
  reference sampling interval for the trace.*
- **Mezghani, M., 2024, "Machine Learning on Core Photographs for Conventional Core Analysis":
  EAGE conference proceedings, DOI 10.3997/2214-4609.2024101274.** Random-forest regression from
  core photographs to routine core-analysis properties: R² 0.497 (porosity), 0.578 (permeability).
  *An honest calibration ceiling — a photograph carries real but limited pore-scale information;
  supports the standing rule that CPHOTO_DARK is never VSH.*

### Lithology / facies classification from core photos

- **Thomas, A., Rider, M., Curtis, A., and MacArthur, A., 2011, "Automated lithology extraction
  from core photographs": First Break, v. 29(6), p. 103–109.** Object-based image analysis with
  interpreter-defined classes and training samples — the foundational pre-deep-learning paper.
  *The supervised, interpreter-in-the-loop posture — same as the mineral classifier's
  clicks-are-the-training-data design.*
- **Baraboshkin, E.E., et al., 2020, "Deep convolutions for in-depth automated rock typing":
  Computers & Geosciences, v. 135, 104330.** CNN rock typing over ~2000 m of core (~20,000
  images, six lithology classes), framed as producing a depth-continuous rock-type log.
  *Directly relevant to the planned CPHOTO_LITH discrete curve.*
- **Alzubaidi, F., Mostaghimi, P., Swietojanski, P., Clark, S.R., and Armstrong, R.T., 2021,
  "Automated lithology classification from drill core images using convolutional neural
  networks": Journal of Petroleum Science and Engineering, v. 197, 107933.** End-to-end core-tray
  photograph → lithology log (93% on sandstone/limestone/shale), including tray-to-column
  extraction. *Published precedent for the whole pipeline shape.*
- **Fu, D., Su, C., Wang, W., and Yuan, R., 2022, "Deep learning based lithology classification
  of drill core images": PLOS ONE, v. 17(7), e0270826.** Ten lithologies via ResNeSt-50; notable
  for admitting that brightness/contrast/saturation had to be randomised 30–130% in training
  because lighting differs between departments. *Measured evidence that unconditioned core
  photography varies enough between sessions to change the answer — the conditioning argument.*
- **Abdlmutalib, A.J., Ayranci, K., Waheed, U.B., et al., 2025, "Automated identification of
  sedimentary structures in core images using object detection algorithms": PLOS ONE, v. 20(7),
  e0327738.** YOLOv4 / Faster R-CNN on 15 sedimentary structure types (parallel lamination,
  cross-lamination, wavy bedding, fissile shale, mud drapes…) in 506 box-core photographs;
  mAP 92.8% — but performance degrades markedly on unseen datasets, reported rather than buried.
  *Lamination-class detection state of the art, deltaic training set, and the generalisation
  warning that mirrors the measured colour-cast finding on the first real delivery.*

### Core-to-log depth matching with core-derived curves

Backs: `registration.rs` (constant + per-barrel shifts, correlogram reporting, CPHOTO-vs-GR).

- **Hoppie, B.W., Blum, P., and Shipboard Scientific Party, 1994, "Natural gamma-ray measurements
  on ODP cores: introduction to procedures with examples from Leg 150": Proc. ODP, Initial
  Reports, 150, p. 51–59.** Core NGR's stated first purpose is core-log correlation; derives the
  empirical calibration bringing core counts onto API units so the two curves correlate directly.
  *The core premise of Register Depth — a curve measured ON the core correlated against wireline
  GR is the established method, units reconciled first.*
- **Fontana, E., Iturrino, G.J., and Tartarotti, P., 2010, "Depth-shifting and orientation of
  core data using a core–log integration approach: A case study from ODP–IODP Hole 1256D":
  Tectonophysics, v. 494(1–2), p. 85–100.** Incomplete recovery means core pieces sit at wrong
  depths; relocates them piece by piece against downhole logs and images. *The published argument
  for per-barrel / per-piece shifts (`RunShift`) rather than one constant per well.*
- **Torres Cáceres, V.A., Duffaut, K., Yazidi, A., Westad, F.O., and Johansen, Y.B., 2022,
  "Automated Well-Log Depth Matching — 1D Convolutional Neural Networks Vs. Classic Cross
  Correlation": Petrophysics, v. 63(1), p. 12–34.** Head-to-head; characterises where plain
  cross-correlation succeeds and fails (repetitive section, low contrast). *Documented failure
  modes for the correlogram engine — the comb-of-rival-peaks note has literature behind it.*
- **Ezenkwu, C.P., Guntoro, J., Starkey, A., Vaziri, V., and Addario, M., 2023, "Automated
  Well-Log Pattern Alignment and Depth-Matching Techniques: An Empirical Review and
  Recommendations": Petrophysics, v. 64(1), p. 115–129.** Systematic empirical review of
  cross-correlation, DTW and ML alignment with method-selection guidance. *Best single entry
  point to the depth-matching literature; supports reporting the whole correlogram.*

### Texture / lamination measures from images

- **Honeycutt, C.E., and Plotnick, R.E., 2008, "Image analysis techniques and gray-level
  co-occurrence matrices (GLCM) for calculating bioturbation indices and characterizing biogenic
  sedimentary structures": Computers & Geosciences, v. 34(11), p. 1461–1472.** GLCM texture on
  rock-slab images, with an explicit noise-sensitivity control on artificial images. *A texture
  measure down a core image with the noise discipline the touches_detail warning reaches for.*
- **Wang, Y., and Sun, S., 2022, "A rock fabric classification method based on the grey level
  co-occurrence matrix and the Gaussian mixture model": Journal of Natural Gas Science and
  Engineering, v. 104, 104627.** GLCM features + unsupervised GMM separating bedding fabrics with
  no training set. *A route to a lamination-vs-massive class curve without hand-labelling.*
- *(Flagged, see §7)* **Singh, A., et al., 2019, GLCM rock characterization: Water Resources
  Research, v. 55(3), DOI 10.1029/2018WR023342.** GLCM as an objective rock descriptor (micro-CT).

---

## 2. Image conditioning and colour calibration

Backs: `coreimage.rs` conditioning (white balance, CLAHE, perspective, detail warnings) and the
thin-section reference-plate correction in `petrography.rs`.

### Colour constancy / white balance foundations

- **Land, E.H., and McCann, J.J., 1971, "Lightness and Retinex Theory": Journal of the Optical
  Society of America, v. 61(1), p. 1–11.** Origin of the white-patch/max-RGB estimator family.
  *The click-a-grey-patch white balance is the manual white-patch estimator.*
- **Buchsbaum, G., 1980, "A spatial processor model for object colour perception": Journal of the
  Franklin Institute, v. 310(1), p. 1–26.** The primary source of the grey-world assumption.
  *The paper stating the assumption the rock violates — the citation for deliberately rejecting
  grey-world.*
- **Finlayson, G.D., Drew, M.S., and Funt, B.V., 1994, "Spectral sharpening: sensor
  transformations for improved color constancy": JOSA A, v. 11(5), p. 1553–1563;** and
  **Finlayson, Drew, and Funt, 1994, "Color constancy: generalized diagonal transforms suffice":
  JOSA A, v. 11(11), p. 3011–3019.** Together: when and why a diagonal (per-channel gain / von
  Kries) correction suffices in place of a full 3×3 matrix. *The formal defence of the three-gain
  model used in both the core white balance and the reference-plate correction, and of the
  argument that a hue rotation is not an equivalent substitute.*
- **Barnard, K., Cardei, V., and Funt, B., 2002, "A comparison of computational color constancy
  algorithms — Part I": IEEE Transactions on Image Processing, v. 11(9), p. 972–984 (Part II
  p. 985–996).** Benchmarks documenting grey-world's failure modes: it degrades as scene surface
  diversity falls; a scene dominated by one chromatic surface is the worst case. *Measured
  evidence for "the rock's colour is signal, not cast".*
- **Gijsenij, A., Gevers, T., and van de Weijer, J., 2011, "Computational color constancy: survey
  and experiments": IEEE Transactions on Image Processing, v. 20(9), p. 2475–2489.** The standard
  survey. *White-balance choice is an assumption choice, not a universally correct algorithm.*
- **McCamy, C.S., Marcus, H., and Davidson, J.G., 1976, "A color-rendition chart": Journal of
  Applied Photographic Engineering, v. 2(3), p. 95–99.** The 24-patch Macbeth/X-Rite
  ColorChecker. *Chart-based session calibration, and the neutral patch as anchor.*

### Colour calibration in geoscience imaging practice

- *(Flagged, see §7)* **ODP Technical Notes 26 (ch. 7) and 37 (ch. 15), "Reflectance
  spectrophotometry and colorimetry": Ocean Drilling Program, Texas A&M.** Zero + certified-white
  calibration before every run; L\*a\*b\* as the successor to Munsell. *A colour measurement is
  only comparable if tied to a declared standard.*
- *(Flagged, see §7)* **Rock-Color Chart Committee (Goddard, E.N., chair), Rock-Color Chart:
  Geological Society of America (1948; now the Munsell Geological Rock-Color Chart).** *Geologists
  already work against declared colour references.*
- **Kemp, D.B., 2014, "Colorimetric characterisation of flatbed scanners for rock/sediment
  imaging": Computers & Geosciences, v. 67, p. 69–74.** Device RGB → device-independent colour
  for rock imaging. *A delivered photograph carries the capture device's response, not the rock's
  colour.*
- **Troscianko, J., and Stevens, M., 2015, "Image calibration and analysis toolbox…": Methods in
  Ecology and Evolution, v. 6(11), p. 1320–1331.** Linear-radiance images calibrated against grey
  standards in frame, so photographs from different sessions become comparable measurements.
  *The standardise-across-lighting-sessions protocol, generic across fields.*
- *(Flagged, see §7)* **Boiger, R., et al., 2024, "Direct mineral content prediction from drill
  core images via transfer learning": Swiss Journal of Geosciences (arXiv:2403.18495).**
  ColorChecker ICC profile + white-patch normalisation applied to every core image BEFORE
  segmentation, validated against XRD. *Current practice for the ordering: colour first, then
  segmentation, then measurement.*

### Illumination correction and stain normalization (microscopy / histopathology analogues)

- **Model, M.A., and Burkhardt, J.K., 2001, "A standard for calibration and shading correction of
  a fluorescence microscope": Cytometry, v. 44(4), p. 309–316.** Prospective flat-field: shading
  correction as a calibration, not an enhancement. *Correct uneven illumination before any area
  or intensity measurement off a plate.*
- **Smith, K., et al., 2015, "CIDRE: an illumination-correction method for optical microscopy":
  Nature Methods, v. 12(5), p. 404–406;** and **Peng, T., et al., 2017, "A BaSiC tool for
  background and shading correction of optical microscopy images": Nature Communications, v. 8,
  14836.** Retrospective corrections estimated from the image collection itself — no calibration
  frames needed. *The realistic case for a delivered petrography set that arrives with no
  reference frames.*
- **Reinhard, E., Ashikhmin, M., Gooch, B., and Shirley, P., 2001, "Color transfer between
  images": IEEE Computer Graphics and Applications, v. 21(5), p. 34–41.** Mean/std matching in a
  decorrelated colour space. *The simplest reference-image normalisation — and its failure mode
  (matching whole-image statistics normalises away genuine content differences) is exactly the
  whole-plate-median trap the matrix-anchor correction avoids.*
- **Macenko, M., et al., 2009, "A method for normalizing histology slides for quantitative
  analysis": IEEE ISBI 2009, p. 1107–1110.** Separates the stains (SVD in optical-density space)
  before renormalising — far less prone to distorting a slide whose stain proportions genuinely
  differ. *The direct analogue of anchoring on a physically meaningful component (the matrix)
  rather than whole-image statistics.*
- **Vahadane, A., et al., 2016, "Structure-preserving color normalization and sparse stain
  separation for histological images": IEEE Transactions on Medical Imaging, v. 35(8),
  p. 1962–1971.** Colour changes, structure does not; designed to fix normalisers that alter
  apparent stain AMOUNT along with colour. *A colour correction must not change how much of a
  phase is measured — the invariant pinned by
  `a_plate_corrected_onto_one_lit_the_same_way_is_left_alone`.*
- **Tellez, D., et al., 2019, "Quantifying the effects of data augmentation and stain color
  normalization in convolutional neural networks for computational pathology": Medical Image
  Analysis, v. 58, 101544.** Across four organs and nine laboratories, measures how much
  between-site colour variation costs and how much normalisation recovers. *The best-documented
  quantification anywhere of "colour differences between capture sessions change the measured
  answer" — the histopathology twin of the 289° hue spread across one petrography delivery.*

### CLAHE and perspective rectification (canonical citations)

- **Pizer, S.M., et al., 1987, "Adaptive histogram equalization and its variations": Computer
  Vision, Graphics, and Image Processing, v. 39(3), p. 355–368.** States the over-enhancement
  problem the clip limit exists to solve.
- **Zuiderveld, K., 1994, "Contrast limited adaptive histogram equalization," in Graphics Gems
  IV: Academic Press, p. 474–485.** The CLAHE implementation reference (tiles, clip +
  redistribute, bilinear interpolation between tile mappings).
- **Hartley, R., and Zisserman, A., 2004, Multiple View Geometry in Computer Vision, 2nd ed.:
  Cambridge University Press.** Chapters 2 and 4: the plane homography, the four-point case, and
  metric rectification — including why the rectified rectangle's aspect ratio must come from the
  scene, not the frame. *The four-dragged-corners rectification and the
  proportions-from-the-quad's-own-sides rule.*

---

## 3. Thin-bed quantification and the LRLC link

Backs: the LRLC research line (`lrlc.rs` context), CPHOTO_TEX / planned CPHOTO_LITH as an
image-scale N/G source, and any future reconciliation against Thomas-Stieber.

### The canonical thin-bed framework

- **Passey, Q.R., Dahlberg, K.E., Sullivan, K.B., Yin, H., Brackett, R.A., Xiao, Y.H., and
  Guzmán-Garcia, A.G., 2006, Petrophysical Evaluation of Hydrocarbon Pore-Thickness in Thinly
  Bedded Clastic Reservoirs: AAPG Archie Series No. 1, 210 p.** THE reference monograph.
  Reframes the deliverable as hydrocarbon pore-thickness (HPT); covers bed-thickness effects on
  every measurement, Thomas-Stieber, resistivity anisotropy, and volumetric laminated-sand
  analysis (VLSA) — where bed-type properties from core combine with bed-frequency statistics
  from high-resolution logs or CORE IMAGE ANALYSIS. *VLSA is essentially the workflow a CPHOTO
  N/G feeds; HPT is the correct volumetric output when bed-scale N/G comes from an image.*
- **Thomas, E.C., and Stieber, S.J., 1975, "The distribution of shale in sandstones and its
  effect upon porosity": SPWLA 16th Annual Logging Symposium, Paper T.** Laminated / dispersed /
  structural shale each trace a distinct trajectory on the shale-indicator vs total-porosity
  crossplot. *The log-scale volumetric model an image-scale N/G is reconciled against — and
  already implemented in SandiBumi's interactive Thomas-Stieber crossplot.*
- **Klein, J.D., Martin, P.R., and Allen, D.F., 1997, "The petrophysics of electrically
  anisotropic reservoirs": The Log Analyst, v. 38(3), p. 25–36.** Thin beds behave as a bulk
  anisotropic medium; Rv/Rh carries saturation information a scalar Rt cannot. *The physical
  statement of WHY a laminated sequence reads as LRLC pay.*
- **Schoen, J.H., Mollison, R.A., and Georgi, D.T., 1999, "Macroscopic electrical anisotropy of
  laminated reservoirs: a tensor resistivity saturation model": SPE-56509-MS;** and **Mollison,
  R.A., et al., 1999, "A model for hydrocarbon saturation determination from an orthogonal tensor
  relationship in thinly laminated anisotropic reservoirs": SPWLA 40th, Paper OO.** The tensor
  laminated-sand model and its explicit inversion for laminar shale volume and sand saturation.
  *The theoretical bridge between a laminar sand fraction (which images estimate) and a
  defensible Sw; laminar shale volume is the exact quantity an image N/G supplies or checks.*
- **Mezzatesta, A.G., Mollison, R.A., and Frost, E., 2002, "Laminated shaly sand reservoirs — an
  interpretation model incorporating new measurements": SPWLA 43rd, Paper TT.** Multi-component
  induction + NMR + borehole images in one model. *The multi-measurement reconciliation problem
  stated explicitly.*
- **van Popta, J., Hofstra, P., and van Houwelingen, S., 2004, "An advanced evaluation method for
  laminated shaly sands including uncertainty and sensitivity": SPWLA 45th, Paper RRR.**
  Uncertainty/sensitivity framework ranking each input's contribution to Sw and net-pay
  uncertainty. *Template for propagating an image-derived N/G's uncertainty through
  `montecarlo.rs`.*
- **Minh, C.C., Joao, I., Clavaud, J.-B., and Sundararaman, P., 2007, "Formation evaluation in
  thin sand/shale laminations": SPE-109848-MS.** The simultaneous three-unknown problem (sand
  resistivity, sand fraction, sand porosity). *Fixing N/G from images collapses it to two.*
- **Majid, A.A., and Worthington, P.F., 2012, "Definitive petrophysical evaluation of thin
  hydrocarbon reservoir sequences": SPE Reservoir Evaluation & Engineering, v. 15(5),
  p. 584–595, SPE-163071-PA.** Systematic thin-bed geometry management. *Peer-reviewed journal
  anchor for the methodology.*

### Sand counting / net-to-gross from borehole image logs

- **Yadav, L., Dutta, T., Kundu, A., and Sinha, N., 2009, "A new approach for the realistic
  evaluation of net sand pay on image log in very thin reservoirs of Krishna Godavari Basin":
  SPWLA 50th, Paper J.** The key cautionary paper: a simple static-image threshold OVERESTIMATES
  net sand by up to 100%, because apparently homogeneous sand still contains shale below the
  imager's own resolution. *Directly transferable to a darkness/colour threshold on a core
  photograph — the same failure mode one order of magnitude finer.*
- **Yadav, L., Dutta, T., Kundu, A., and Sinha, N., 2010, SPE-132970-MS.** The explicit recipe:
  binary sand/shale lithology from the image-derived Vsh curve, dispersed shale + laminar sand
  porosity via Thomas-Stieber, Sw from anisotropy. *The published combination of an image-derived
  binary lithology with Thomas-Stieber — the reconciliation SandiBumi would aim at.*
- **Yadav, L., Dutta, T., and Sinha, N., 2012, "Reconciliation of core and log data analysis in
  very thin reservoirs…": SPE-149017-MS.** Plug-scale porosity/Sw agreement does NOT validate a
  thin-bed evaluation — non-uniform plugging biases the core toward the better sand. *The
  sharpest statement that plug agreement is not evidence; bears directly on how an image N/G is
  validated against core.*
- **Nooh, A.Z., and Moustafa, El Abbas A.A., 2017, "Comparison of quantitative analysis of image
  logs for shale volume and net-to-gross calculation of a thinly laminated reservoir…": Egyptian
  Journal of Petroleum, v. 26(3), p. 619–625.** Calibrates one FMI trace against GR, then
  computes Vclay and N/G from the corrected high-resolution trace. *An open-literature worked
  example of exactly the CPHOTO_DARK-calibrated-against-GR operation.*
- **Kherroubi, J., Maeso, C., Wang, Y., and Gamero-Diaz, H., 2016, "Lamination analysis from
  electrical borehole images: a quantitative workflow": SPWLA 57th, Paper BBBB.** Automatic
  extraction of lamination statistics via frequency-domain decomposition into thickness scales.
  *The closest published methodology to bed-thickness DISTRIBUTIONS (not just a net count) from
  an image.*
- **Feng, Z., et al., 2024, "A new method for quantitative evaluation of shale laminae using
  electrical image logging": Energy Geoscience, v. 5(3).** Lamina counting by peak/trough
  identification, validated against lamina counts observed in core. *The validation design to
  copy: image-derived lamination index vs core count.*
- **Claverie, M., et al., 2007, "Applications of NMR logs and borehole images to the evaluation
  of laminated deepwater reservoirs": SPE-110223-MS.** T2-partition sand/silt/clay volumes
  cross-checked against image sand counts (SE Asia deepwater). *A second independent N/G
  estimator to cross-check an image count.*
- **Bastia, R., et al., 2007, "Evaluation of low-resistivity-pay deepwater turbidites using
  constrained thin-bed petrophysical analysis": SPE-110752-MS.** High-resolution bed geometry as
  a CONSTRAINT stabilising the low-resolution inversion. *The architecture argument for feeding
  a core-photo N/G into the solver rather than reporting it alongside.*
- **Hayden, R., et al., 2009, "Thin bed interpretation techniques for northwestern Gulf of Mexico
  coastal and offshore clastics": SPWLA 50th (SPWLA-2009-54527).** Practitioner comparison of
  Thomas-Stieber vs anisotropy vs image-based approaches and where each breaks down. *Guidance
  text material for which technique to trust in which bedding regime.*
- **Hathon, L.A., Myers, M.T., and Horvath, W., 2024, "Validating the extended Thomas-Stieber
  model using a variety of imaging modalities": SPWLA 65th.** Thin-section and SEM image analysis
  quantify dispersed/structural shale independently, then test the extended Thomas-Stieber
  partition against them. *Precedent for validating Thomas-Stieber with image measurements — at
  the thin-section scale SandiBumi already handles.*

### Lamination from core photographs specifically

- **Selmaoui, N., Repetti, B., Laporte-Magoni, C., et al., 2004, "Coupled strata and granulometry
  detection on indurated cores by gray-level image analysis": Geo-Marine Letters, v. 24(4),
  p. 241–251.** Strata + grain size from core imagery at two resolutions; wavelet transform beats
  Fourier for strata detection, chiefly because it separates genuine strata from drilling-induced
  fractures. *The most directly transferable method — a wavelet decomposition of a grey-level
  core trace is the next step past CPHOTO_TEX, and the fracture-discrimination result is a
  warning CPHOTO_TEX will need.*
- **Xu, C., 2022, "Log evaluation, borehole image interpretation, and core calibration of
  deep-water reservoir rocks: a case study in the Gulf of Mexico": AAPG Bulletin, v. 106(6),
  p. 1197–1212.** Three-way core ↔ core-image ↔ borehole-image ↔ log calibration chain in thinly
  bedded rock, tested for regional transferability. *The published template for the full
  calibration chain.*
- *(Prior art only, not citable as a method source)* **Phillips, C. (Philliec459), "Utilize
  continuous core images to calibrate borehole image logs": GitHub repository.** Open-source
  Python sand/shale from continuous core images calibrating image-log sand count. Verified to
  exist; not peer-reviewed.

### LRLC pay — causes and regional cases

- **Worthington, P.F., 2000, "Recognition and evaluation of low-resistivity pay": Petroleum
  Geoscience, v. 6(1), p. 77–92** *(start page verified; end page commonly cited but
  unconfirmed — §7)*. THE standard review: laminated sand-shale, microporosity, clay
  conductivity, conductive minerals, fresh water, organised into a decision structure that
  selects the saturation model. *Thin beds first among the causes.*
- **Boyd, A., Darling, H., Tabanou, J., et al., 1995, "The lowdown on low-resistivity pay":
  Oilfield Review, v. 7(3), p. 4–18.** The accessible introduction, freely available, with a
  log/core integration case (Gandhar, India). *The reference to hand a non-specialist.*
- **Darling, H.L., and Sneider, R.M., 1992, "Production of low resistivity, low contrast
  reservoirs, offshore Gulf of Mexico Basin": GCAGS Transactions, v. 42, p. 73–88.** Established
  LRLC intervals as economically material; explicitly names Indonesia as an LRLC province.
  *The historical anchor.*
- **Claverie, M., Allen, D.F., Heaton, N., and Bordakov, G., 2010, "A new look at low-resistivity
  and low-contrast (LRLC) pay in clastic reservoirs": SPE-134402-MS.** Which LRLC causes modern
  measurements can separate from logs alone, and which still require core. *The 15-years-on
  update to Worthington.*
- **Belevich, A., and Bal, A.A., 2018, "The problem with silt in low-resistivity low-contrast
  (LRLC) pay reservoirs": Petrophysics, v. 59(2), p. 118–135.** Silt as a distinct, commonly
  mishandled third phase that defeats sand/shale binaries. *The sharpest warning for any
  image-based N/G: a darkness threshold sees a two-phase world, and silt is assigned to
  whichever side the threshold falls on.*
- **Tolioe, A., et al., 2016, "Low resistivity pay evaluation, case study: thin bed sand-shale
  lamination reservoirs, Peninsula, Malay Basin": IPTC-18724-MS.** Nearest-neighbour SE Asian
  deltaic case with the thin-bed cause explicit in the title.
- **Audinno, R.T., Pratama, I.P., Halim, A., and Kusuma, D.P., 2016, "Integrated analysis of the
  low-resistivity hydrocarbon reservoir in the 'S' field": Proceedings, Indonesian Petroleum
  Association 40th Annual Convention, IPA16-436-SE.** A MAHAKAM DELTA LRLC field (fluvio-deltaic,
  onshore East Kalimantan), using petrography, XRD and SEM to establish which clay/conductive
  minerals act alongside the thin-bed effect. *The Mahakam-specific reference — and it shows the
  same rock can be LRLC for two reasons at once, which is the petrography-beside-logs argument.*

---

## 4. UV fluorescence of core

Backs: `CPHOTO_FLUOR` / `CPHOTO_FLUOR_I` and the "inferred show, never a saturation" framing.

- **Swanson, R.G., 1981, Sample Examination Manual: AAPG Methods in Exploration Series No. 1,
  35 p.** The industry-standard wellsite logging system; fluorescence recorded as colour,
  intensity, distribution, and PERCENTAGE OF SAMPLE FLUORESCING, plus cut and residue. *That
  percentage is the manual analogue of the CPHOTO_FLUOR area fraction — the vocabulary the show
  reports use.*
- **Morton-Thompson, D., and Woods, A.M., eds., 1993, Development Geology Reference Manual: AAPG
  Methods in Exploration Series No. 10, 565 p.** Fluorescence colour/intensity/percentage are
  DESCRIPTIVE observations to integrate with cut, odour and log response — not a saturation.
  *The citation for the standing "inferred show" rule.*
- **McPhee, C., Reed, J., and Zubizarreta, I., 2015, Core Analysis: A Best Practice Guide:
  Developments in Petroleum Science, v. 64, Elsevier, 852 p.** White + UV core photography as
  routine core handling, and the conditions (freshness, invasion, drying, OBM contamination)
  under which UV appearance is and is not diagnostic. *The conditioning/lighting discipline and
  the caveats the fluorescence run must print.*
- **Reyes, M.V., 1994, "Application of fluorescence techniques for mud-logging analysis of oil
  drilled with oil-based muds": SPE Formation Evaluation, v. 9(4), p. 300–305.** QFT and Total
  Scanning Fluorescence; motivation stated outright: visual UV description is "highly subjective
  and inconsistent". *The case for a numeric fluorescence trace, and the precedent of a per-depth
  fluorescence intensity curve.*
- **Liu, K., and Eadington, P., 2005, "Quantitative fluorescence techniques for detecting
  residual oils and reconstructing hydrocarbon charge history": Organic Geochemistry, v. 36(7),
  p. 1023–1036.** QGF / QGF-E distinguish palaeo-oil zones from present residual oil. *The
  warning that fluorescence may record FORMER oil — another reason the curve is an inferred
  show.*
- **Downare, T.D., and Mullins, O.C., 1995, "Visible and near-infrared fluorescence of crude
  oils": Applied Spectroscopy, v. 49(6), p. 754–764.** Emission spectra and quantum yields for
  ten crudes; red shifts strongest for heavy oils under short-wavelength excitation. *Physical
  basis for reading hue as an oil-type proxy, and its limits at 365 nm.*
- **Ryder, A.G., 2007, "Analysis of crude petroleum oils using fluorescence spectroscopy," in
  Reviews in Fluorescence 2005: Springer, p. 169–198.** Emission red-shifts and quantum yield
  falls as aromatic/NSO content rises (API falls); quenching, not concentration, governs
  intensity at high loading. *The light-blue-white vs heavy-gold-brown rule with the
  concentration-quenching caveat that stops intensity being read as saturation.*
- **Stasiuk, L.D., and Snowdon, L.R., 1997, "Fluorescence micro-spectrometry of synthetic and
  natural hydrocarbon fluid inclusions…": Applied Geochemistry, v. 12(3), p. 229–241.**
  Systematic λmax red shift ~440 → ~595 nm with increasing aromatic/NSO content. *The strongest
  quantitative support for any hue-based split of the fluorescence band — relevant if a second
  FluorClass is ever added.*
- **Linton, P., Kosanke, T., Greene, J., and Porter, B., 2023, "The application of hyperspectral
  core imaging for oil and gas": Geological Society, London, Special Publications, v. 527(1),
  p. 95–119.** Modern instrumented core imaging: acquisition, calibration, segmentation, and
  UPSCALING core-scale image measurements to log scale. *The resample-onto-the-log-frame problem
  treated explicitly.*

---

## 5. Quantitative digital petrography

Backs: `petrography.rs` — pore area (Delesse), pore geometry, grain size/sorting, Wicksell,
staining schemes, the mineral classifier — and `plugqc.rs`.

### Stereology foundations

- **Delesse, A., 1848, "Procédé mécanique pour déterminer la composition des roches": Annales des
  Mines, 4th ser., v. 13, p. 379–388.** Areal fraction on a random section estimates volume
  fraction (AA = VV). *The entire justification for VPORE_TS.*
- **Rosiwal, A., 1898, "Über geometrische Gesteinsanalysen…": Verh. K.-K. Geol. Reichsanstalt,
  no. 5/6, p. 143–175.** Linear intercepts: LL = AA = VV. *The lineage behind point counting.*
- **Chayes, F., 1956, Petrographic Modal Analysis: An Elementary Statistical Appraisal: Wiley,
  113 p.** Binomial sampling variance of a modal estimate. *Why classifier clicks are a sample
  with quantifiable error, and why overall accuracy hides a rare class.*
- **van der Plas, L., and Tobi, A.C., 1965, "A chart for judging the reliability of point
  counting results": American Journal of Science, v. 263(1), p. 87–90.** The 2σ nomogram every
  petrographer uses. *Stating uncertainty on any point-count or classifier fraction. Caveat:
  Vermeesch (2018, EPSL) shows the normal approximation fails at extreme proportions/low counts —
  use exact binomial there.*
- **Wicksell, S.D., 1925, "The corpuscle problem": Biometrika, v. 17(1–2), p. 84–99.** Recovering
  sphere diameters from circular profiles — why a section under-reports size and over-reports
  spread. *The justification for the correction AND for the separate `_APP` / `_W` names.*
- **Saltykov, S.A., 1958, Stereometric Metallography, 2nd ed.: Metallurgizdat, Moscow.** The
  finite-histogram (Schwartz-Saltykov) inversion: logarithmic classes, coarsest-down subtraction;
  its ill-conditioning and negative populations are well documented. *The unfolding
  implementation and the clamp-and-count (`w_clamped`) decision.*
- **Underwood, E.E., 1970, Quantitative Stereology: Addison-Wesley, 274 p.** VV = AA = LL = PP;
  perimeter/surface estimation from INTERCEPT COUNTING rather than boundary tracing. *The Crofton
  perimeter's textbook home.*
- **Weibel, E.R., 1979, Stereological Methods, Vol. 1: Academic Press, 415 p.** Sampling design —
  how many fields, how many sections. *Why one plate per plug is a sample and field count matters
  more than pixel count.*
- **Sahagian, D.L., and Proussevitch, A.A., 1998, "3D particle size distributions from 2D
  observations: stereology for natural applications": Journal of Volcanology and Geothermal
  Research, v. 84(3–4), p. 173–196.** Quantifies the error of spherical coefficients on
  non-spherical particles. *The honest limit on Saltykov applied to sand grains.*

### Thin-section image analysis — porosity and pore geometry

- **Ehrlich, R., Crabtree, S.J., Kennedy, S.K., and Cannon, R.L., 1984, "Petrographic image
  analysis, I. Analysis of reservoir pore complexes": Journal of Sedimentary Petrology,
  v. 54(4), p. 1365–1378** *(page range flagged — §7)*. The founding PIA paper: computer
  measurement of pore size/shape/type from blue-epoxy sections. *The lineage of the whole
  per-pore geometry family.*
- **Ehrlich, R., Crabtree, S.J., Horkowitz, K.O., and Horkowitz, J.P., 1991, "Petrography and
  reservoir physics I: Objective classification of reservoir porosity": AAPG Bulletin, v. 75(10),
  p. 1547–1562;** **McCreesh, C.A., Ehrlich, R., and Crabtree, S.J., 1991, "…II: Relating thin
  section porosity to capillary pressure…": p. 1563–1578;** **Ehrlich, R., Etris, E.L.,
  Brumfield, D., Yuan, L.P., and Crabtree, S.J., 1991, "…III: Physical models for permeability
  and formation factor": p. 1579–1592.** The trilogy: pore TYPES with characteristic size/shape
  distributions; which pore BODIES associate with which THROATS (from Hg-Pc regression); and
  permeability/FF models from image-derived abundances. *Part II is the published basis for Plug
  QC's design decision — bodies vs throats compared on RANK correlation with no 1:1 line.*
- **Anselmetti, F.S., Luthi, S.M., and Eberli, G.P., 1998, "Quantitative characterization of
  carbonate pore systems by digital image analysis": AAPG Bulletin, v. 82(10), p. 1815–1836.**
  The carbonate counterpart: pore size, roundness, aspect ratio, perimeter-over-area on
  blue-epoxy sections, correlated with velocity and permeability; explicitly addresses
  unresolvable microporosity. *The carbonate case for every pore-geometry item and the choice of
  circularity + aspect as descriptors.*
- **Haines, T.J., Neilson, J.E., Healy, D., Michie, E.A.H., and Aplin, A.C., 2015, "The impact of
  carbonate texture on the quantification of total porosity by image analysis": Computers &
  Geosciences, v. 85B, p. 112–125.** Image porosity vs helium/MICP on the same samples: accurate
  in grainstones (≤5% low), badly low in wackestone (>10% absolute), due to unresolved
  microporosity + unrepresentative fields. *The published, TEXTURE-DEPENDENT expectation that
  VPORE_TS reads below helium — matching the microporosity finding from the first real delivery,
  and why it is not a constant offset to calibrate out.*
- **Grove, C., and Jerram, D.A., 2011, "jPOR: An ImageJ macro to quantify total optical porosity
  from blue-stained thin sections": Computers & Geosciences, v. 37(11), p. 1850–1859.** The
  closest published analogue of the blue-epoxy colour band: threshold-based, benchmarked against
  point counting, lower counting error and inter-operator variability — and it inherits exactly
  the lighting sensitivity the reference-plate correction exists for. *Prior art for the pore
  rule itself.*
- **Roduit, N., JMicroVision: image analysis toolbox for petrographic images (v. 1.3.5),
  https://jmicrovision.github.io.** The de facto reference implementation cited across the
  petrographic image-analysis literature. *Feature parity and a cross-check target.*
- **Schneider, C.A., Rasband, W.S., and Eliceiri, K.W., 2012, "NIH Image to ImageJ: 25 years of
  image analysis": Nature Methods, v. 9(7), p. 671–675;** and **Schindelin, J., et al., 2012,
  "Fiji: an open-source platform for biological-image analysis": p. 676–682.** ImageJ/Fiji's
  Analyze Particles conventions (circularity 4πA/P², fitted-ellipse aspect) are the de facto
  definitions users will compare against. *Where the Crofton perimeter differs from ImageJ's
  boundary trace should be stated — the systematic-bias argument already in the module doc.*
- **Rubo, R.A., Carneiro, C. de C., Michelon, M.F., and Gioria, R. dos S., 2019, "Digital
  petrography: Mineralogy and porosity identification using machine learning algorithms in
  petrographic thin section images": Journal of Petroleum Science and Engineering, v. 183,
  106382.** Pixel-wise supervised classification; random forests competitive with neural nets at
  lower cost, labels from an independent source (SEM). *Algorithm-choice precedent for the
  colour+texture mineral classifier.*

### Grain size and shape

- **Folk, R.L., and Ward, W.C., 1957, "Brazos River bar [Texas]: a study in the significance of
  grain size parameters": Journal of Sedimentary Petrology, v. 27(1), p. 3–26.** σI = (φ84−φ16)/4
  + (φ95−φ5)/6 and its verbal sorting scale. *GRAIN_SORT_APP/_W — the exact formula and classes.*
- **Krumbein, W.C., 1934, "Size frequency distributions of sediments": Journal of Sedimentary
  Petrology, v. 4(2), p. 65–77.** The phi transform, φ = −log2(d mm). *The phi convention
  including the sign — phi rises as grains get finer, pinned by
  `phi_rises_as_grains_get_finer`.*
- **Wadell, H., 1932, "Volume, shape, and roundness of rock particles": Journal of Geology,
  v. 40(5), p. 443–451; and 1933, "Sphericity and roundness of rock particles": v. 41(3),
  p. 310–331.** The authoritative sphericity/roundness definitions. *The distinction between
  roundness (corner sharpness) and circularity (overall form) that a perimeter-based circularity
  does NOT measure — worth stating in any grain-shape caption.*
- **Johnson, M.R., 1994, "Thin section grain size analysis revisited": Sedimentology, v. 41(5),
  p. 985–999.** Corrections from thin-section apparent size to sieve-equivalent size. *A
  section-derived D50 is not a sieve D50 even after unfolding — the `_APP`/`_W` naming caution.*
- **Schäfer, A., and Teyssen, T., 1987, "Size, shape and orientation of grains in sands and
  sandstones — image analysis applied to rock thin-sections": Sedimentary Geology, v. 52(3–4),
  p. 251–271.** Early automated grain analysis with a section-vs-sieve comparison: the section
  curve is displaced toward the coarse side but the discrepancy is small; Wicksell handled by
  Monte Carlo. *The empirical number to expect when a user compares GRAIN_D50_APP with a sieve.*

### Carbonate staining

- **Friedman, G.M., 1959, "Identification of carbonate minerals by staining methods": Journal of
  Sedimentary Petrology, v. 29(1), p. 87–97.** (Page range verified as 87–97, not the sometimes
  quoted 87–90.) Alizarin red S stains calcite red; dolomite stays UNSTAINED. *The colourless
  identification is why StainBand carries a saturation ceiling rather than a floor.*
- **Dickson, J.A.D., 1965, "A modified staining technique for carbonates in thin section":
  Nature, v. 205(4971), p. 587.** The priority publication for the combined ARS + potassium
  ferricyanide stain; ferricyanide reveals FERROUS IRON in any carbonate, not dolomite
  specifically.
- **Dickson, J.A.D., 1966, "Carbonate identification and genesis as revealed by staining":
  Journal of Sedimentary Petrology, v. 36(2), p. 491–505.** The full method: etch, combined
  stain, ARS alone — the four-class scheme (non-ferroan/ferroan calcite/dolomite) that became
  standard. *Note for the codebase: the repo's "Dickson (1966)" attribution is correct; a fuller
  attribution cites 1965 (Nature, priority) and 1966 (JSP, full method) together.*

---

## 6. Where the literature is thin — white space the CPHOTO work occupies

Four genuine gaps, each searched for and not found (absence of evidence after a directed search,
not proof of absence — but consistent across independent sweeps):

1. **No published paper cross-correlates a PHOTOGRAPH-derived trace against gamma ray for
   core-to-log depth registration.** Both halves exist separately (core gamma vs downhole gamma —
   Hoppie et al. 1994; colour/darkness logs from photos — Perarnau 2011, Martin et al. 2021).
   The joint is SandiBumi's.
2. **No peer-reviewed paper computes a fluorescing-AREA fraction from UV core photographs.**
   Perarnau 2011 is the closest (pixel colour under UV); a granted US patent (US 12,307,642,
   UV fluorescence imaging of drill cuttings) shows commercial interest but is not a method
   paper. The conservative "inferred show, never a saturation" framing is the right posture
   precisely because there is no published calibration to lean on.
3. **Nothing published on reading packed core-display plates** (multiple barrel columns per page
   with labelled intervals). CoreBreakout handles core trays, not multi-barrel plate layouts.
4. **Little on core-slab-photo lamination → petrophysical N/G specifically.** Selmaoui et al.
   2004 is the closest methodological match; the ML work (Martin 2021, Abdlmutalib 2025) labels
   facies/structures rather than producing an N/G that feeds a volumetric. This is the gap the
   LRLC line would step into if CPHOTO_TEX / CPHOTO_LITH is developed toward a laminar sand
   fraction feeding Thomas-Stieber/VLSA.

These matter for the licensed-product posture: they are claims of novelty a marketing or method
document could make, and they mark where SandiBumi cannot lean on a published calibration and must
carry its own validation (the Feng et al. 2024 design — image-derived index vs counted core — is
the template).

---

## 7. Flagged / partially verified — confirm before citing in a deliverable

- **Ehrlich et al. 1984** — two independent sweeps returned p. 1365–1376 and p. 1365–1378;
  resolve against the DOI record before quoting pages.
- **Worthington 2000** — start page 77 verified; end page 92 commonly cited but unconfirmed.
- **Nederbragt & Thurow 2005** — chapter verified via DOI; page range unconfirmed (paywalled).
- **ODP Technical Notes 26/37** — chapter content verified; chapter authorship not confirmed.
  Cite as an ODP Technical Note chapter.
- **GSA Rock-Color Chart** — the chart and its 115 Munsell chips verified; the exact 1948 Goddard
  citation and reprint years unconfirmed.
- **Boiger et al. 2024** (Swiss J. Geosciences) — authors/title verified; volume/article number
  unconfirmed. arXiv:2403.18495 is safe to cite.
- **Günther et al. 2025, "Machine learning for drill core image analysis: A review"** (Ore
  Geology Reviews, 106974) — one sweep verified authors + venue, the other dropped it over
  inconsistent journal identification; mining-focused either way. Verify before citing.
- **Singh et al. 2019** (WRR, DOI 10.1029/2018WR023342) — paper real; full author list
  unconfirmed against the publisher page.
- **Glagolev (point-counting priority, ~1931–34)** — referenced only second-hand in stereology
  histories; no reliable primary record found. Do NOT cite; Rosiwal 1898 + Chayes 1956 cover the
  ground with verified citations.
- **"Integrated Petrophysical Analysis to Evaluate LRLC Pays … SE Asia" (Academia.edu)** — no
  author list, venue or year confirmable. Do NOT cite without obtaining the original.

Springer and ScienceDirect blocked some direct fetches during the sweeps, so several records rest
on indexer metadata (Crossref, Semantic Scholar, ADS) rather than the publisher page — sound for
working use, but run one library-proxy confirmation pass before any of these appears in a client
deliverable or a method document.

---

## 8. PDF library — download manifest (2026-08-05)

PDFs live in `literature/` (gitignored — copyrighted files never ship in the repo, the chartbook
rule). Every file below was verified on disk: `%PDF-` header, plausible size, and a first-page
content spot-check on the doubtful ones. All copies came from lawful sources only — publisher
open access, official society/program archives, PMC/Europe PMC, arXiv, university repositories,
and authors' own academic pages. No Sci-Hub / LibGen / ResearchGate.

### On disk (30 files)

| File | Reference | Source |
|---|---|---|
| Abdlmutalib_2025_sedimentary_structures.pdf | Abdlmutalib et al. 2025, PLOS ONE | PLOS (OA) |
| Alzubaidi_2021_lithology_CNN_thesis.pdf | **SUBSTITUTE**: Al-Zubaidi 2022 UNSW PhD thesis, NOT the 2021 JPSE article (paywalled) — same work, different document | UNSWorks (CC-BY) |
| Baraboshkin_2020_rocktyping_CNN.pdf | Baraboshkin et al. 2020 — arXiv preprint 1909.10227, may differ slightly from the C&G published version | arXiv |
| Barnard_Cardei_Funt_2002_PartI.pdf / _PartII.pdf | Barnard et al. 2002, IEEE TIP I+II | author page (kobus.ca) |
| Boiger_2024_mineral_transferlearning.pdf | Boiger et al. 2024 | arXiv |
| Boyd_1995_LowResistivityPay.pdf | Boyd et al. 1995, Oilfield Review | SLB free archive |
| Chayes_1956_PetrographicModalAnalysis.pdf | Chayes 1956 (book) | archive.org (free) |
| Ezenkwu_2023_welllog_depthmatching.pdf | Ezenkwu et al. 2023, Petrophysics (accepted ms.) | Aberdeen AURA |
| Finlayson_Drew_Funt_1993_DiagonalTransforms_ICCV.pdf | **SUBSTITUTE**: 1993 ICCV precursor of the paywalled JOSA-A 1994 "generalized diagonal transforms suffice" — do not cite as the 1994 paper | SFU author page |
| Finlayson_Drew_Funt_1994_SpectralSharpening.pdf | Finlayson et al. 1994 (spectral sharpening) | SFU author page |
| Fu_2022_lithology_CNN.pdf | Fu et al. 2022, PLOS ONE | PLOS (OA) |
| Gijsenij_Gevers_vandeWeijer_2011_survey.pdf | Gijsenij et al. 2011 survey (author preprint) | author page |
| Gunther_2025_ML_drillcore_review.pdf | Günther et al. 2025, Ore Geology Reviews | DiVA portal |
| Hoppie_1994_ODP150_naturalgamma.pdf | Hoppie, Blum et al. 1994 | ODP/TAMU (free) |
| IODP_Exp395_methods.pdf | IODP Expedition 395 methods | publications.iodp.org |
| Li — NOT retrieved, see manual list | | |
| Macenko_2009_NormalizingHistologySlides.pdf | Macenko et al. 2009, ISBI | UNC author page |
| Martin_2021_lithofacies_ML.pdf | Martin, Meyer & Jobe 2021 | Frontiers (OA) |
| McCamy_Marcus_Davidson_1976_ColorRenditionChart.pdf | McCamy et al. 1976 | RIT (.edu) |
| Meyer_2020_CoreBreakout.pdf | Meyer et al. 2020, JOSS (with repo docs) | JOSS (CC-BY) |
| ODP_TN26_chap7_reflectance.pdf | ODP Technical Note 26 ch. 7 | ODP/TAMU (free) |
| ODP_TN37_full.pdf | ODP Technical Note 37 (full, incl. ch. 15) | ODP/TAMU (free) |
| Peng_2017_BaSiC.pdf | Peng et al. 2017, Nature Communications | Nature (OA) |
| Pizer_etal_1987_AHE_variations_citeseerx.pdf | **PREPRINT**: 1986 UNC TR 86-013, the preprint of the 1987 CVGIP paper | CiteSeerX |
| Reinhard_Ashikhmin_Gooch_Shirley_2001_ColorTransfer.pdf | Reinhard et al. 2001 | RIT (.edu) |
| Rosiwal_1898_Verhandlungen_fullvolume.pdf | Rosiwal 1898 — full public-domain journal volume; the article is at printed p. 143–175 | Austrian Geol. Survey scan |
| Tellez_2019_stain_augmentation.pdf | Tellez et al. 2019 (arXiv v2) | arXiv |
| Thomas_2011_lithology_corephotos.pdf | Thomas et al. 2011, First Break | author copy, geos.ed.ac.uk |
| Troscianko_Stevens_2015_ImageCalibrationToolbox.pdf | Troscianko & Stevens 2015 | Europe PMC |
| vanderPlas_Tobi_1965_PointCountingReliability.pdf | van der Plas & Tobi 1965 | ajsonline.org (free) |

### Free but needs a MANUAL browser download (bot-gated, two clicks each)

These are legitimately free to read; automated retrieval was blocked by bot-detection, which was
not bypassed. Open in a normal browser and save:

- Nooh & Moustafa 2017 (gold OA): https://www.sciencedirect.com/science/article/pii/S1110062116300344
- Feng et al. 2024, Energy Geoscience (gold OA): https://www.sciencedirect.com/science/article/pii/S2666759223001208
- Li et al. 2022, G3 (gold OA): https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2022GC010350
- Schneider et al. 2012, ImageJ (free on PMC, not in the redistributable OA subset): https://pmc.ncbi.nlm.nih.gov/articles/PMC5554542/
- Schindelin et al. 2012, Fiji (same): https://pmc.ncbi.nlm.nih.gov/articles/PMC3855844/
- Torres Cáceres et al. 2022 (accepted ms. on NTNU Open, download needs a browser session): https://hdl.handle.net/11250/2988262
- Grove & Jerram 2011 jPOR (Durham repository, post-Worktribe-migration page 403'd scripted access): search "jPOR Grove Jerram" at durham-repository.worktribe.com
- Delesse 1848 (public domain; scan exists via annales.org / Gallica — exact ark not located)

### Paywalled — no lawful free copy found

Petrophysics-society material (OnePetro/Datapages; SPWLA membership or per-paper purchase is the
route — the SPWLA papers alone justify a membership for this topic): Perarnau 2011; Thomas &
Stieber 1975; Klein et al. 1997; Schoen et al. 1999 + Mollison et al. 1999; Mezzatesta et al.
2002; van Popta et al. 2004; Minh et al. 2007; Majid & Worthington 2012; Yadav et al. 2009/2010/
2012; Kherroubi et al. 2016; Claverie et al. 2007 & 2010; Bastia et al. 2007; Hayden et al. 2009;
Hathon et al. 2024; Belevich & Bal 2018; Reyes 1994; Tolioe et al. 2016 (IPTC); Audinno et al.
2016 (IPA members download free — worth checking an IPA membership); Darling & Sneider 1992
(GCAGS/Datapages).

GSW/SEPM/AAPG journals: Ehrlich et al. 1984; the 1991 trilogy (Ehrlich I, McCreesh II, Ehrlich
III); Anselmetti et al. 1998; Folk & Ward 1957; Krumbein 1934; Friedman 1959; Dickson 1966; Xu
2022; Worthington 2000 (Lyell).

Commercial publishers: Selmaoui et al. 2004 (Springer); Nederbragt & Thurow 2005 (Springer);
Nederbragt et al. 2006 (Lyell); Fontana et al. 2010, Liu & Eadington 2005, Stasiuk & Snowdon
1997, Sahagian & Proussevitch 1998, Schäfer & Teyssen 1987, Wang & Sun 2022, Honeycutt & Plotnick
2008, Kemp 2014, Rubo et al. 2019 (Elsevier); Johnson 1994, Model & Burkhardt 2001 (Wiley);
Dickson 1965, Smith et al. 2015 CIDRE (Nature); Wadell 1932/1933 (UChicago); Downare & Mullins
1995 (SAGE); Wicksell 1925 (OUP); Buchsbaum 1980; Land & McCann 1971 (Optica); Vahadane et al.
2016 (IEEE); Ryder 2007 (Springer chapter); Linton et al. 2023 (Lyell); Mezghani 2024 (EarthDoc).

Books (buy or library): Passey et al. 2006 (AAPG Archie Series 1 — the one most worth owning for
the LRLC line); McPhee et al. 2015; Swanson 1981; Morton-Thompson & Woods 1993; Underwood 1970;
Weibel 1979; Saltykov 1958; Hartley & Zisserman 2004; Zuiderveld 1994 (Graphics Gems IV chapter).
