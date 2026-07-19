# Borehole image log suite (conditioning, processing, interpretation) + core photo digitization pipeline for SandiBumi

## Files
- D:\01. Work\00. Guidebook\03. Guidebooks Techlog\335269406-Techlog-2011-Training-Course.pdf
  MISLEADING FILENAME - this is actually the full 'Techlog Wellbore Imaging Workflow/Solutions Training v2011.1' manual (~150 pp, Schlumberger), annotated by Jauhar's own hand (tool family notes, static/dynamic sketches). THE primary Topic A reference. Modules: 1 Data Loading, 2 Data Processing (speed correction, pad image creation, image-based speed correction, button harmonization, pad concatenation, histogram equalization), 3 Display, 4 LWD images, 5 Dip Picking (5 modes), 6 Dip Interpretation Plots (walkout/cumulative/stereonet), 7 Automatic Dip Computation, 8 Structural Dip Removal, 9 Fracture Counting (Terzaghi correction with worked example), 10 3D display. Manual page N = PDF page N+6. Key pages read: PDF 17-44 (processing), 69-84 (dip picking + dip dataset structure), 129-143 (structural dip removal + fracture counting).
- D:\01. Work\00. Guidebook\02. Guidebook Geolog\SOP GEOLOG\SOP\Carbonate\10Image Porosity.pdf
  Jauhar's team SOP (Indonesian) for Geolog's Geomage borehole-image module chain: Speed Correction (TOOL_OFFSET 0.4064 m), Image Generation (FCA1-FCD4 arrays, EMEX flag, magnetic declination, DOI 0.2 in), Equalize and Normalize (window 16.4042 ft equalize / 0.984252 ft dynamic-normalize, max 5 bad buttons per window), Image Binarisation (cutoff window 0.6 m), Histogram Upscaling (101-frame window, 100 bins), and three image-porosity methods: crossplot regression PHIT=10^(-1.54672+0.356959*log10(CXO)), inverse-Archie per button, 'Newsberry' (Newberry). Also documents the full Geomage menu = a module inventory blueprint: Speed Correction, Image Generation, Image Extract Pad, Image Cull, Image Conversion, Correlate Pad Image, Equalize and Normalize, Image Binarisation, Image Filter, Eccentering Correction, Histogram Upscaling, Auto Dip, Auto Texture 2D-SOM, Auto Texture MRGC, Auto Fracture; speed correction 'Apply to' targets: FMS, FMI, OBMI, UBI, EMI, CAST, STAR Resistivity, CBIL, Earth Imager, HMI, Any.
- D:\01. Work\00. Guidebook\01. Reference\STAR-XR-extended-range-resistivity-imaging-service-slsh.pdf
  Baker Hughes STAR-XR brochure: 6 independently articulated arms, 24 sensors/pad = 144 buttons, 0.26 in (6.6 mm) vertical+azimuthal resolution, hole 6.25-21 in, mud resistivity 0.01-10 ohm-m, max 900 ft/hr, combinable with CBIL/UltrasonicXplorer acoustic imagers. Useful for tool-geometry table.
- D:\01. Work\00. Guidebook\02. Guidebook Geolog\FACIMAGE\2006 - Geolog 6.6.1 - FACIMAGE; Practical Illustrated Guide.pdf
  Rabiller's FACIMAGE guide. NOT borehole image processing - it is electrofacies: MRGC clustering, KNN core-data prediction, STM log similarity, histogram upscaling. Relevant only as analog for a future auto-texture/electrofacies module (Geomage Auto Texture MRGC uses same engine). A second copy (Geolog 6.6) sits in the same folder.
- D:\01. Work\00. Guidebook\02. Guidebook Geolog\coreanalysis.pdf
  Geolog 20 'Core Analysis' user guide (219 pp): SCAL import, capillary-pressure corrections (closure, clay-bound water, stress), saturation-height modeling (lambda/EQR, Leverett-J, FWL solving). NOT core photos - irrelevant to Topic B, relevant to a future SCAL module. Duplicate larger copy: 'Core Analysis User Guide.pdf' in same folder.
- D:\01. Work\00. Guidebook\03. Guidebooks Techlog\Techlog Fundamentals 2015.PDF
  General Techlog training (not read in detail; not image-specific).
- D:\01. Work\00. Guidebook\02. Guidebook Geolog\SOP GEOLOG\SOP\Carbonate\12Porosity Type.pdf
  Companion SOP in same carbonate chain (porosity typing downstream of image porosity); not read.

## Methods
### A1. Image data model (storage layer)
Canonical storage for pad-based electrical imagers, acoustic imagers and LWD images. Follow Techlog's convention: after 'pad image creation' every tool reduces to N pad arrays [frame x buttons] with pads and buttons ordered CLOCKWISE looking down-hole, all samples shifted to true borehole depth. Store raw vendor arrays untouched plus a normalized layer. Tool zoo (from Jauhar's handwritten cover notes + brochure): conductive-mud FMI/FMS (SLB), STAR/STAR-XR (Baker, 6 pads x 24 = 144 buttons, 0.26 in res), XRMI (Halliburton), CMI (Weatherford); OBM: OBMI, Dual OBMI, Earth Imager, OMRI; acoustic: UBI, CBIL, CAST-V (travel-time + amplitude arrays); LWD: GVR, StarTrak (azimuthally binned while rotating - no speed correction, different handling).

Equations:
MEASURE-POINT-OFFSET per channel (from DLIS or tool default table): z_channel = z_ref - offset; FMI accelerometer-to-measure-point offset = 0.4064 m (16 in). Positive offset = deeper than reference (Techlog sign convention).

Inputs: DLIS/LIS load. FMI channel set (exact mnemonics from both manuals): button arrays FCA1,FCA2 (Pad A rows 1,2), FCA3,FCA4 (Flap A), FCB1..FCB4, FCC1..FCC4, FCD1..FCD4 (raw = 16 arrays x 12 buttons; after interlacing = 8 arrays x 24 -> tool alias FMI_16 vs FMI_8). Auxiliary: C1,C2 (calipers), P1AZ (pad-1 azimuth), RB (relative bearing = pad-A orientation w.r.t. well high side), HAZI, DEVI, EV (EMEX voltage), FBGA (FMI electronic gain), AZ + FCAZ (z-accelerometer; MUST use fast channel FCAZ for SLB), FTIM (frame time), magnetometers. Inclinometry commonly 120 samples/ft while image is 60 or 30 samples/ft - store the high-rate set separately, never downsample (Techlog warning).
Outputs: DuckDB schema suggestion: dataset table (well, run, tool, source system OP/GEOFRAME/RECALL/EXPRESS/GEOLOAD, orientation_type North|TopOfHole, DOI, button vertical/angular geometry, associated caliper/HAZI/DEVI curve ids - Techlog stores exactly these as array properties and refuses dip picking without them); frames table keyed (dataset_id, depth) with FixedSizeList/BLOB per pad array; separate high-rate inclinometry table; versioned outputs per processing step (suffix chain like Techlog _S speed-corrected, _ISC image-based-SC, _H harmonized, _DYNAMIC).
Calibration: Source-system parameter presets (Techlog Pad image creation): Source {OP raw SLB, GEOFRAME, RECALL, EXPRESS raw Baker, GEOLOAD raw Halliburton, IDEAL} drives Interlaced?, Reverse arrays?, Reverse odd pads?, odd-pad and odd-button vertical offsets. GeoFrame stores buttons counter-clockwise (must reverse); Recall clockwise; raw STAR packs two button-row depths in one frame (needs de-multiplex + odd-pad vertical shift). Beware round-tripped files: PRODUCT property may lie (set source manually).

### A2. Inclinometry QC + accelerometer speed correction
First conditioning step. QC/repair magnetometer + accelerometer channels, recompute orientation curves, apply magnetic declination; then correct depth for tool stick-pull using z-accelerometer + frame time (preferred) or cable speed (legacy fallback when FTIM absent).

Equations:
Axial kinematics: a_tool(t) = FCAZ - g*cos(DEVI); v_tool = integral(a_tool dt) drift-locked to cable speed via complementary (high-pass accel / low-pass cable) filtering; z_tool(t) = integral(v_tool dt); shift dz(t) = z_tool - z_cable; each channel corrected at z_channel = z_ref(t) - channel_offset (an event in time hits different depths for different sensors - correction must be applied per channel at its own offset). Sticking: v_tool ~ 0 while cable moves -> flag; pull: v_tool >> v_cable. Aim: depth match to 0.1 in.

Inputs: FCAZ (z-accel), FTIM (frame time = elapsed time between successive frames, NOT cumulative) or cable speed, measured depth, per-channel relative offsets (Offsets tab: e.g. FCA1 -16.3 in, FCA2 -16 in, flaps -10.6 in relative to accelerometer), zonation interval.
Outputs: Corrected depth curve DEPTH_S, per-channel speed-corrected arrays (_S), QC curves: SHIFT_S (applied shift), TOOL_ACCELERATION_S, TOOL_VELOCITY_S, STICKING_DETECTOR_S (flag 0/1), STICKING_PERIOD.
Calibration: QC recipe from manual: run in 'compute only' mode first, display shift + tool velocity tracks, look for excessive shifts; the most common failure is wrong accelerometer channel (AZ instead of FCAZ). Only then 'compute and apply'. Correction outside the zonation interval = 0.

### A3. Pad image creation (geometry normalization + EMEX/gain correction)
Transforms vendor-specific raw arrays into the canonical per-pad arrays (A1 conventions): interlace the two button rows of each pad into one array (first button of row1, first of row2, ...), reverse arrays where source is counter-clockwise, apply odd-pad/odd-button vertical offsets, and apply the tool's electrical corrections: EMEX voltage + electronic gain for FMI, pad gain + bucker gain for STAR. After this step FMI_16 becomes FMI_8 (8 arrays x 24 buttons) and all later processing is tool-generic.

Equations:
EMEX normalization (standard FMI form): button conductivity c_ij proportional to I_ij / (V_emex * G_electronic); Geolog exposes it as EMEX_FLAG gain correction toggle. Interlacing: out[2k]=row1[k], out[2k+1]=row2[k].

Inputs: Speed-corrected button arrays (FCAx_S...), EV, FBGA (or STAR PadGain/BuckerGain), source-system preset (A1 calibration).
Outputs: PAD_1..PAD_4, FLAP_1..FLAP_4 arrays at true depth, gain-corrected (units: calibrated conductivity counts).
Calibration: Gain correction default = yes whenever EV+FBGA present. Zonation tip: exclude casing / closed-arm intervals so they do not skew later color scaling.

### A4. Image-based (residual) speed correction
After accelerometer correction, residual depth offsets remain between the two button rows (sawtooth look) and between pads/flaps (alternating up-down of successive pads). Cross-correlate button columns / pad strips over a sliding depth window and shift each to maximize correlation, minimizing residual offsets. Techlog exposes almost no parameters (advanced: correlation window + max allowable shift, hidden by default). Recommended order: run BEFORE button harmonization (optionally harmonize before AND after if responses differ wildly, to help correlation).

Equations:
For pad p, window w: shift_p = argmax_s corr(pad_p(z+s), reference(z)), s limited to +/- max shift; reference = mean image or adjacent pad; apply shift by resampling.

Inputs: PAD_x/FLAP_x arrays from A3.
Outputs: _ISC arrays; per-pad shift curves (QC).
Calibration: Defaults fine per manual; verify visually that sawtooth on bed boundaries disappears.

### A5. Button harmonization + dead-button detection/repair + cutoff
Corrects response differences between buttons/pads caused by electronics, poor pad contact, mudcake smear. Applies per-button shift + gain to match a reference response - no interpolation or filtering, so no resolution loss; recommended always-on. Bundled sub-algorithms (both default-off): (a) faulty (dead) button detection - flag buttons whose activity stays below threshold over a detection window, null them, repair by interpolation from neighbor buttons; (b) cutoff - clip values outside [min,max], replacing with nulls, neighbor interpolation, or boundary values. Very NOISY (not dead) buttons are repaired manually: null the column in a data editor, then run faulty-button repair to interpolate.

Equations:
Per button b (window or whole zone): x' = (x - mu_b) * (sigma_ref / sigma_b) + mu_ref, with (mu_ref, sigma_ref) from the global/array population. Dead if var(x_b) < tolerance (Techlog defaults: detection window 0.03048 m, tolerance 0.0001, repair = lateral interpolation).

Inputs: _ISC arrays; harmonization process type {Global = each button matched to response of all buttons; By array = matched within its pad; Both = By-array then Global}; optional window size; exclude-array list (a bad pad can be excluded from computing the global reference while still being corrected).
Outputs: _H arrays; dead-button flag map.
Calibration: CRITICAL warning from manual: default is NO window (whole-interval statistics). If windowed, window length must be MUCH longer than the sine-wave amplitude of dipping geology, else the algorithm erases real formation contrast around the borehole. Geolog equivalent defaults: equalization window 5 m (16.4042 ft), auto bad-button correction on, max 5 bad buttons per window, uniform target distribution.

### A6. Pad concatenation and orientation (oriented image array)
Places every pad/flap at its correct azimuthal position around the borehole and resamples to a single oriented array (default 360 columns = 1 deg/px) referenced to North or Top-of-Hole; gaps between pads sized from calipers. This single array is the interpretation product (dip picking, sharing, Python processing). Also derives ASSOC_CAL (average caliper) if none supplied - needed for dip picking.

Equations:
Column of button k on pad p: theta = P1AZ + delta_p + k*button_pitch_angle (mod 360), where delta_p from tool geometry; for TOH reference theta_TOH = theta_N - (HAZI-derived high-side azimuth). Gap width from caliper: arc = pad_width / (pi * D) * 360.

Inputs: P1AZ (North ref) or RB+DEVI+HAZI (TOH ref), C1, C2, HAZI, DEVI, _H pad arrays, orientation mode {North|TopOfHole}, angular resolution (default 360 px), perimeter mode {use calipers | bit size}, horizontal interpolation yes/no, exclude-pad flags.
Outputs: ARRAY_WBI_H (oriented, gaps as nulls or interpolated), ASSOC_CAL; array properties set: orientation type, DOI (FMI 0.394 in), reference caliper/deviation/azimuth names - prerequisite metadata for dip picking (A8).
Calibration: Excluding a pad with bad response is a display/processing option (deviated hole with floating pad). Magnetic declination must already be applied to P1AZ/HAZI (Geolog exposes MDEC_COR toggle).

### A7. Static and dynamic normalization (histogram equalization)
Contrast normalization by histogram equalization, output scale default 0-255. STATIC = one histogram for the whole interval; same measurement always maps to same color; preserves absolute resistivity character (best for inter-well comparison, OBMI extremes). DYNAMIC = sliding-window equalization; local contrast enhanced, same value maps differently along the hole; brings out fine internal detail (Jauhar's margin note: 'take small area from the log, distribute colors, for fine details'). Both are kept as separate arrays; interpretation uses them side by side.

Equations:
y = round((L-1) * CDF_w(x)) where CDF_w is the cumulative histogram of the window (or whole zone for static); cumulative frequency of equalized data tends to linear (equal-area bins).

Inputs: Oriented array or individual pad arrays; window size (Techlog default 1 m dynamic; Geolog default 0.3 m / 0.984252 ft); process type {Global | By array | By button}; output min/max (0-255 editable, reversible for high=resistive vs high=conductive conventions); equalization type {static|dynamic|both}; low/high percentile cutoffs (Geolog 0/100 defaults).
Outputs: IMAGE_STATIC, IMAGE_DYNAMIC (or _H_DYNAMIC pad arrays re-concatenated).
Calibration: Dynamic window sizing rule from manual: geology changes slowly -> too-short window amplifies noise; geology changes fast -> too-long window hides subtle contrast. Start 1 m, tune visually.

### A8. Manual dip picking (5 modes) + sinusoid-to-dip math
Interactive picking on the unrolled oriented image at scale ~1:20. Five modes (Techlog): (1) FULL sine wave - click >=3 points, sinusoid auto-fits, points draggable, right-click/Enter validates; (2) PARTIAL sine wave - for features not extending around the hole (truncated cross-beds, faulted bedding); first click = counter-clockwise end, wave drawn clockwise from it; stores which sector is valid; (3) STRETCHY sinusoid - cursor IS a sinusoid: mouse up/down = depth, left/right = trough azimuth, click-drag = amplitude; fastest for laminated sections; (4) INDUCED FRACTURE - two clicks on the ends of the near-vertical linear feature (not a sinusoid); (5) BREAKOUT - parallelogram fitted over the wide dark band; pick each side of a breakout pair independently (breakouts are often asymmetric; two picks give better statistics). Each pick carries a quality value (manual picks default 1.0, editable 0-1) and a classification type.

Equations:
On the unrolled image (x = azimuth theta from reference, y = depth), a plane cuts a cylinder as z(theta) = z0 - (R_eff * tan(delta_app)) * cos(theta - az_app), R_eff = ASSOC_CAL/2 + DOI (image formed at depth-of-investigation, FMI DOI = 0.394 in ~ 1 cm). Apparent (borehole-frame) dip: tan(delta_app) = h / d_eff where h = peak-to-trough height of the sinusoid, d_eff = 2*R_eff; az_app = azimuth of the trough (down-dip). True dip from borehole geometry: build unit normal n_bh = (sin delta_app * cos az_app, sin delta_app * sin az_app, cos delta_app) in borehole frame, rotate to earth frame n_e = Rz(HAZI) * Ry(DEVI) * n_bh; true dip = acos(n_e.z), true dip azimuth = atan2(n_e.y, n_e.x) (+180 if needed so azimuth points down-dip). If image referenced to TOH, first add high-side azimuth to az_app. Techlog stores the apparent dip PER IMAGE (different orientation reference and DOI give different apparent dips for the same feature) and one true dip.

Inputs: Oriented image with properties from A6 (orientation type, DOI, ASSOC_CAL, DEVI, HAZI); dip classification scheme; picking scale.
Outputs: Dip point dataset (see A9). Breakout pick attributes: Breakout_Azimuth_(N|TOH), Breakout_Height (along-hole length), Breakout_Omega_Angle (angle to borehole axis, counter-clockwise), Breakout_Width (angular width deg). Induced fracture: Induced_Fracture_Azimuth, _Height, _Omega_Angle. Do NOT represent breakouts/induced fractures as sine-wave dips (their azimuth is the feature azimuth, not a plane dip azimuth; sine picks would be 90 deg off and break symbol display).
Calibration: Manual warns 'apparent dip' here = dip relative to the borehole, not line-of-section apparent dip. Picking on over-compressed depth scale introduces serious dip errors.

### A9. Dip dataset structure + classification scheme
Dips saved as point datasets. Three load-bearing variables: Dip_TRU (true dip inclination), Azimuth (true dip azimuth), Type (classification); everything else supports display and reprocessing. Classification is a user-editable scheme scoped at User/Project/Company/App level; Techlog's default type list (a good SandiBumi default): Bedding, Cross bedding, Sandstone bedding, Shale bedding, Carbonate bedding, Sandstone lamination, Heterolithic bedding, Deformed bed, Erosional surface/boundary, Lithological boundary, MSD Low quality, MSD High quality, Conductive fracture, Resistive fracture, Mixed fracture, Fault, Detachment, Induced fracture, Breakout, Mean, NONE. Each type carries Colour + Shape (tadpole symbol) associations. Reclassification is done by brushing dips in any plot with a target class selected.

Equations:
undefined

Inputs: Picks from A8/A10 or imported dip ASCII (imported dips must have azimuth/type/quality associations re-established).
Outputs: Point dataset: MD, Dip_TRU (dega), Azimuth (dega), Type, Colour, Shape, Quality (0-1), HAzi + HDev sampled at pick depth (stored per dip - needed later for Terzaghi correction and true<->apparent conversion without reloading logs), per-image apparent groups: <ImageName>_Dip_APP, <ImageName>_Azimuth, <ImageName>_DipHeight, breakout group, induced-fracture group, InfoArray (opaque redraw support for partial waves/breakouts).
Calibration: Saving creates a NEW dataset each time (picks in the creation track are unsaved/live); overwriting a dip dataset destroys derived variables added to it (e.g. rotated dips) - version instead.

### A10. Automatic/assisted dip picking
Two engines to mirror: Techlog 'Automatic dip computation' (Module 7; runs on FMI oriented arrays, generates dense dips + quality for user validation/classification - Techlog also auto-adds the dip Type to classification, Module 5 Lesson 4) and Geolog Geomage 'Auto Dip'. Standard implementable algorithm: slide a depth window over the oriented image; within the window, either (a) cross-correlate azimuthal strips to estimate best-fit plane (dipmeter-style interval correlation), or (b) detect edges then least-squares/RANSAC fit z(theta)=z0 - A*cos(theta-phi) maximizing contrast coherence along the sinusoid; keep fits above a quality threshold; quality = normalized correlation/coherence. Output flows into the same dip dataset with quality<1 to distinguish from manual picks.

Equations:
undefined

Inputs: Oriented image (static or dynamic), window length, step, max dip, quality threshold.
Outputs: Dense dip set (typically bedding), quality curve.
Calibration: Validate against a manually picked interval; auto dips are downsampled/filtered (e.g. keep local quality maxima) before structural analysis.

### A11. Dip interpretation plots: stereonet, rose, walkout, cumulative dip
Four plot types (Techlog Module 6), all filterable by Type/zonation and supporting interactive selection -> reclassification or set flagging: (1) Stereonet: poles or dip vectors, Schmidt (equal-area) or Wulff, upper/lower hemisphere, density contours, strike histogram overlay, multi-well overlay; polygon/interactive selection creates flag variables (used to define fracture sets, A13). (2) Rose diagram: azimuth (or strike) frequency; azimuth-mirror option for breakout/induced-fracture pairs (180-deg symmetric data). (3) Walkout (vector) plot: cumulative unit-vector walk of dip azimuths vs depth; direction changes locate structural-domain boundaries; also used for paleotransport. (4) Cumulative dip plot: cumulative dip magnitude vs depth; slope breaks = structural zone boundaries / unconformities / faults. Stereonets can also be posted on a well map.

Equations:
Schmidt projection of pole (dip d, az a): r = R*sqrt(2)*sin((90-d)/2 in appropriate convention), theta = a; walkout: P_k = P_{k-1} + (sin az_k, cos az_k); cumulative dip: S_k = S_{k-1} + d_k.

Inputs: Dip dataset(s) (multi-well), type filter, zonation.
Outputs: Structural zone boundaries, mean structural dip/azimuth per zone (read from stereonet pole cluster mean), fracture-set flag variables.
Calibration: Workflow: identify structural zones on walkout/cumulative plots -> compute per-zone mean bedding dip on stereonet (low-energy bedding only, outliers rejected) -> feed A12.

### A12. Structural dip removal
Restores dips to depositional orientation by removing per-zone structural dip (Techlog Module 8, 'Structural dip removal (relative dip)'). User selects which input dip types get rotated (typically all sedimentary types; exclude induced fracture/breakout), and per structural zone specifies the structural dip to remove as: mean dip + mean azimuth (typical), top/bottom-interpolated dip pair, or a variable (dangerous - variable must be a smoothed representative structural dip with no outliers, e.g. downsampled mean of low-energy bedding merged into the dataset). Output = new rotated dip variables in the same dataset.

Equations:
Rotation: for each dip build unit normal n from (d, a); rotate by -d_s about the horizontal axis along strike of the structural plane (axis azimuth = a_s + 90): n' = R_axis(-d_s) * n; rotated dip/azimuth from n'. (Equivalently: subtract the structural plane by rotating the whole dataset so the mean bedding pole becomes vertical.)

Inputs: Dip_TRU, Azimuth, Type; zonation table with per-zone (mean dip to remove, mean azimuth to remove) e.g. Zone_1: 4 deg @ 12 deg, Zone_2: 8 deg @ 10 deg (manual example); dip-type include list.
Outputs: DPTR_ROT (rotated dip inclination), DPTRAZ_ROT (rotated dip azimuth); parameters savable to/restorable from the zonation dataset (Top/Bottom/Mean Dip+Azm to Remove, Method, Input Dip type).
Calibration: Rotated dips then re-examined in walkout plots for paleotransport interpretation; per-zone parameters live with the zonation so reruns/reclassifications restore them.

### A13. Fracture counting, Terzaghi-corrected density, fracture sets
Dip feature counting (multi-well, multi-dataset): counts selected feature types (e.g. Conductive fracture, Resistive fracture, Mixed fracture, Fault) in a sliding window, outputs count, density and borehole-bias-corrected density per type + per fracture-set flag + total. Fracture sets are defined by interactive polygon selection on the multi-well stereonet, which writes a flag variable (1,2,...) per dip. Bias correction is per-fracture Terzaghi weighting - NOT per-set mean orientation (per-set correction adds uncertainty).

Equations:
Weight per fracture: w_i = 1/cos(beta_i), beta_i = 3D angle between the borehole axis (from HDev,HAzi at that depth) and the pole to the fracture plane (beta=0 when fracture orthogonal to hole); beta capped at 85 deg. Corrected density = sum(w_i)/L over window. Manual worked example: 100 m, 10 fr @0 deg + 10 @45 + 10 @87(capped 85): count=30, raw density 0.3 /m, corrected = (10*1 + 10*1.4142 + 10*11.4737)/100 = 1.389 /m - shows correction blows up for near-axial fractures; expose the cap and warn on low counts.

Inputs: Dip dataset with Type, Dip_TRU, Azimuth, HDev, HAzi (per-dip hole geometry), optional set flag curve; window size (default 1 m), step size (default = window; smaller step -> running-average density; counts then overlap but density stays correctly normalized per depth unit), max bias angle (default 85 deg), 'correct for borehole orientation' toggle.
Outputs: Per type X: X_COUNT (unitless), X_DENS (1/m, = count/window, i.e. P10), X_DENS_C (1/m Terzaghi-corrected); Count_SUM, Dens_SUM, Dens_SUM_C; same set under Flag_1_/Flag_2_ groups; plus 'Terzaghi correction factor' variable written back onto each dip (min 1.0, max 1/cos(85) ~ 11.4737).
Calibration: Fracture set definition: multi-well Schmidt stereonet, filter to Conductive/Resistive/Mixed fracture + Fault (exclude induced), contours + strike histogram, polygon-select each strike cluster (e.g. NNW-SSE set=1, ENE-WSW set=2), save labeling variable.

### A14. Fracture aperture (Luthi-Souhaite) - literature spec, no library file
Hydraulic/electrical aperture of open conductive fractures from calibrated FMI conductivity: integrate the excess current (above matrix background) along scanlines crossing the fracture trace, convert to aperture. Requires the image calibrated to physical conductivity (Techlog 'Image calibration' method: fit button conductivity to a shallow resistivity curve, e.g. RXO/LLS) and mud + flushed-zone resistivity.

Equations:
Luthi & Souhaite (1990): W = c * A * Rm^b * Rxo^(1-b), with b ~= 0.863; A = integrated additional (excess) current caused by the fracture (area under the conductivity anomaly across the trace, background-subtracted); c is a constant depending on tool geometry, from forward-model or lab calibration.

Inputs: Calibrated conductivity image, fracture trace picks (A8), Rm (mud resistivity at depth), Rxo, tool constant c (tool/size specific).
Outputs: Aperture W (mm) sampled along each fracture trace; mean/max aperture per fracture; FVAH-style cumulative aperture-length products; feeds fracture porosity (aperture x trace length / image area).
Calibration: Two-step: (1) image calibration regression vs shallow resistivity log over homogeneous zones; (2) c from published FMI values or matching known (core-measured) apertures. Flag results as qualitative when calibration unavailable.

### A15. Image porosity + binarization + sand count (Geolog SOP methods)
Jauhar's carbonate SOP: convert each button conductivity into a porosity curve -> porosity image; then Histogram Upscaling produces a per-depth porosity histogram/spectrum whose spread quantifies secondary (vug/fracture) porosity; Image Binarisation with a cutoff produces a binary macro-pore/conductive-anomaly image. The same binarization on a sand/shale conductivity cutoff gives image-based sand count for thin beds: per depth row, net-sand fraction = fraction of azimuthal pixels flagged sand; integrate over interval for high-resolution N/G to tie against Thomas-Stieber/SSC thin-bed results from conventional logs.

Equations:
(1) X-plot method: regress PHIT vs shallow conductivity in log space over the study interval: PHIT = 10^(a + b*log10(CXO)); SOP example fit PHIT = 10^(-1.54672 + 0.356959*log10(CXO)), CC=0.915; apply per button (replace CXO with FCAx button value, 16x for FMI). (2) Inverse Archie per button: phi_i = sqrt(Rmf / (Sxo^2 * R_button_i)) (a=1, m=n=2; adjust to field m,n). (3) Newberry ('Newsberry' in SOP): PHIT_i = PHIT * (LLS * C_i)^(1/2) - scales external total porosity by the ratio of button conductivity to shallow laterolog. Secondary porosity = mean(phi_image) - matrix mode, or area of histogram above matrix-mode threshold.

Inputs: Conditioned button arrays or oriented image; PHIT from conventional evaluation; CXO/LLS shallow resistivity; Rmf, Sxo; binarization: cutoff direction {normal|reversed}, cutoff min/max (auto from window stats or manual), window 0.6 m default; histogram upscaling: window 101 frames, step 1, 100 bins, linear/log scale.
Outputs: Porosity image (per button), POROSITY_SPECTRUM (histogram array log), IMAGE_BINARISED + CUTOFF_VAL, secondary-porosity fraction, image sand-count curve (net sand per window) + sand flag image.
Calibration: Regression coefficients are per-well/per-interval (recalibrate against PHIT_DN); binarization cutoff picked on histogram between matrix mode and conductive-anomaly tail; color scale on Xplot uses caliper to exclude bad-contact points.

### B1. Core photo import + data model
NO reference file exists in the library - entire Topic B spec is from standard practice. Import box photos (JPEG/TIFF/PNG, keep 16-bit TIFF path), one photo = one core box holding N columns (typically 3-5 rows of ~1 m). Non-destructive model: store original file + an editable processing recipe (ordered ops with parameters) + derived products. White-light and UV photo pairs linked to the same box.

Equations:
undefined

Inputs: Image files; user metadata: well, core no., box no., box top/bottom depth, column count, per-column depth ranges, recovery, spacer/rubble annotations; optional EXIF.
Outputs: DuckDB tables: core_photo(box) -> photo_file(blob or path, type WL/UV) -> column_crop(polygon, depth_top, depth_bottom, orientation) -> registration(control points px->depth) -> recipe(json ops); derived: rectified column images, stitched strip pyramid.
Calibration: Everything user-editable later; recipe replay gives reproducibility (same philosophy as Techlog's versioned arrays).

### B2. Geometric conditioning: crop, rotate/deskew, perspective (pure image processing - Rust or Canvas/WebGPU)
(a) Column segmentation: manual rectangles (must-have) + assisted detection: luminance projection profile across the box finds dark gaps between columns; Hough lines find box-divider edges; snap user rectangles. (b) Deskew: rotate by small angle from dominant Hough line of the column edge or a user 2-point line; plus 90/180-deg orientation fixes and top-down direction flag. (c) Perspective: 4-point homography H (DLT from 4 user-clicked corners) to rectify oblique shots so the depth axis is linear in pixels.

Equations:
Affine rotate: p' = R(angle)p; homography: p' ~ H p, H from >=4 correspondences via DLT + bilinear/bicubic resampling. All GPU-trivial (single texture sample pass).

Inputs: Photo, user clicks / assist parameters.
Outputs: Rectified per-column images, constant px-per-mm along depth axis.
Calibration: None beyond user picks; keep sub-pixel resampling (bicubic) to preserve lamination detail.

### B3. Color correction: white balance + color-card matrix + exposure normalization (pure image processing)
Core photos across boxes/sessions differ in lighting; correction makes the stitched strip color-consistent so color = lithology signal. Priority: (1) color-card correction when a 24-patch card (X-Rite style) or gray card is in frame - user clicks card corners, patches auto-sampled; (2) fallback gray-world / white-patch per-channel gains; (3) inter-box exposure normalization by matching card gray-patch luminance (or overlap statistics) across boxes.

Equations:
Work in linear RGB (inverse sRGB gamma first). Card: solve M (3x4 affine incl. offset) minimizing sum ||M*[r,g,b,1]_measured - [r,g,b]_reference||^2 over patches (least squares); apply M then re-gamma. Gray card / white balance: gain_c = L_ref/mean_c over card. Gray-world fallback: gain_c = mean(all)/mean_c. Exposure: scalar k = L*_ref/L*_card applied in linear space.

Inputs: Rectified image, card patch samples + reference sRGB values, or fallback stats.
Outputs: Color-corrected image; stored 3x4 correction matrix per photo.
Calibration: Card reference values built-in (classic 24-patch table); QC = residual dE on patches; warn if >5.

### B4. Enhancement + noise removal (pure image processing)
Display-side enhancement kept separate from the calibrated image (two layers: calibrated 'archive' image and a view LUT/ops, like static vs dynamic images in Topic A). Ops: levels/curves/gamma, CLAHE for local contrast (the core-photo analog of dynamic normalization), saturation, sharpening (unsharp mask), denoise: median 3x3 (impulse noise), bilateral or non-local-means (sensor noise, edge-preserving).

Equations:
CLAHE: per-tile histogram clipped at limit, redistributed, equalized, bilinear-blend tiles. Bilateral: w = exp(-|dp|^2/2s_s^2)*exp(-|dI|^2/2s_r^2). Unsharp: I' = I + a(I - G_sigma*I).

Inputs: Corrected image; CLAHE clip limit ~2-3, tile ~8x8; NLM strength h~5-10; unsharp amount/radius.
Outputs: Enhanced view (recipe-stored), optional baked export.
Calibration: User-preset defaults; per-well preset reuse so all boxes get identical treatment.

### B5. Depth registration + stitching to continuous strip (interpretation-assisted)
Per column: map pixel row -> depth. Base linear map from column top/bottom depths; refine with user control points on ruler ticks visible in photo (piecewise-linear). Handle lost core: spacers/rubble marked as gap intervals (depth advances, pixels skipped or gap-filled). Stitch: resample every column to a common px-per-meter, concatenate by depth into one continuous strip; where photos overlap prefer higher-quality/later photo; store as tiled multi-resolution pyramid (e.g. 256-px tiles, factor-2 levels) for fast log-view rendering.

Equations:
depth(y) = piecewise-linear interp of control points; resample scale s = target_px_per_m / column_px_per_m.

Inputs: Column crops + depth ranges, control points, gap annotations, target resolution (e.g. 10 px/cm).
Outputs: strip(depth) image pyramid per well per photo-type (WL/UV); gap map.
Calibration: QC: column pixel length x scale vs (bottom-top) mismatch flagged >2%; recovered length vs driller's recovery check.

### B6. Core-to-log depth shift + log-view display (interpretation)
Display the stitched strip as a depth track beside wireline logs; apply core-shift so core depth ties log depth. Manual: drag whole cores (block shift per core barrel, constant within a core, stored in a shift table; optional stretch-squeeze between anchor points). Assisted: build a 1D proxy log from the photo - mean darkness/luminance or R/G ratio per depth sample (dark = shale/organic) - and cross-correlate against wireline GR (or core gamma if loaded) over each core interval; propose shift = argmax correlation within +/- a few meters; user accepts/edits. Rendering: tile-pyramid track in the SandiBumi log viewer (Canvas/WebGPU), level-of-detail by zoom, WL/UV toggle, gap hatching; all interpretation overlays (lithology flags, sample points) live in core-shifted depth.

Equations:
proxy(z) = 1 - mean(L*(x,z))/100 across strip width (gaps excluded); shift* = argmax_s corr(proxy(z+s), GR(z)) over core interval; stretch: depth' = piecewise-linear map through user anchors.

Inputs: Strip pyramid, wireline GR, shift search window, anchor points.
Outputs: shift_table(core_no, block_shift, optional stretch pairs), shifted-depth axis, proxy log curve (loggable/QC-able).
Calibration: Core gamma, if available, replaces photo proxy as the correlation curve (standard practice). Keep original and shifted depths both queryable.


## Notes
COVERAGE: Topic A is strongly covered by two in-library sources that mirror the two platforms Jauhar actually uses: the Techlog Wellbore Imaging 2011.1 training manual (misnamed '335269406-Techlog-2011-Training-Course.pdf', hand-annotated by Jauhar - treat its module list as the UX blueprint he expects) and his team's Geolog Geomage SOP (10Image Porosity.pdf, Indonesian). Numeric defaults quoted in methods (window sizes, offsets, 85-deg Terzaghi cap, regression example) are read directly from these files. Not read but available for deeper follow-up in the same manual: Module 1 data loading / WbiImport script (PDF pp 11-16), Module 3 display/palettes/filtering (PDF 47-58), Module 4 LWD orientation (PDF 59-68), Module 5 automatic dip picking exercises (PDF 85-95), Module 7 automatic dip computation detail (PDF 115-120), Module 8 structural-zone identification exercises (PDF 123-128 partially read). GAPS SPECCED FROM LITERATURE (flagged in each method): sinusoid->true-dip rotation math (standard, e.g. Rider 1999 ch. on dipmeter - in library; Serra dipmeter texts), Luthi & Souhaite 1990 aperture equation (no copy in library - constants b~0.863 from the published paper; do not treat c as known), auto-dip algorithm internals (manuals show UI only), and ALL of Topic B: zero core-photo files exist anywhere in the Guidebook tree (searched *image*, *core*, *photo*, *FMI*, *XRMI*, *dip*, *fracture*; the two 'Core Analysis' PDFs are SCAL capillary-pressure/saturation-height, and FACIMAGE is electrofacies clustering, not images). Topic B is therefore specced entirely from standard image-processing practice - no citations invented. ARCHITECTURE SPLIT: Topic A conditioning (A2-A7) and Topic B B2-B4 are deterministic array/image ops well suited to Rust (ndarray/image) with GPU display; A8/A9 picking and B5/B6 registration are interactive UI features; A10-A15 are compute modules over the dip/image datasets. Terzaghi example, EMEX handling, source-system presets and the dip dataset variable inventory are the highest-value implementation details captured verbatim from the manuals. Rose-diagram azimuth-mirror option is needed for breakout/induced-fracture (180-deg symmetric) data. Suggested module naming parity: keep Techlog's processing-chain suffixes (_S, _ISC, _H, _STATIC/_DYNAMIC) so Jauhar can map SandiBumi outputs to what he knows.
