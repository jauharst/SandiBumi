# 25. Fluid substitution, rock physics and image/core analysis — requirements

| | |
|---|---|
| Evidence dossier | `docs/research_2026-08/cross_tool/fluidsub-rockphysics.md` — 1,879 lines |
| Critique applied | `docs/research_2026-08/cross_tool/fluidsub-rockphysics_critique.md` — 707 lines; 36/36 findings dispositioned |
| Evidence held | T1 readable source/API, T2 extracted help, T3 directly read help, T4 training/SOP evidence |
| Source audit | `src-tauri/src/modules.rs`, `coreimage.rs`, `images.rs`, `lib.rs`; `src/ipc.ts`; `src/ui/coreConditionDialog.ts`, `coreTraceDialog.ts`, `imageImportDialog.ts` |
| Authored | 2026-08-08 |
| Requirements | 52 (`SB-RPH-001`…`SB-RPH-052`) |
| P0 requirements | 15 |

## 1. Scope and boundary

This chapter owns isotropic and anisotropic fluid substitution, fluid and mineral elastic mixing,
dry-frame models, shear prediction, Backus upscaling, elastic attributes, AVO and synthetic-seismic
products, borehole-image conditioning and image-derived quantities, and the processing of imported
core photographs. It owns the physics and processing after a curve, array log or photograph exists.

**`21_data-io.md` (`DIO`) — ingest and storage.** DIO owns LAS/DLIS/array/image import, depth and
unit canonicalization, delivery sets and round trips. RPH consumes DIO's typed curves, array logs and
depth-registered images; it does not define file readers.

**`20_envcorr-qc.md` (`ENV`) — generic conditioning.** ENV owns generic curve QC and the shared run
mask. RPH owns borehole-image-specific speed correction, button/pad conditioning, orientation,
declination provenance and the image residuals those operations create.

**`17_thinbed-laminated.md` (`TBD`) — electrical versus elastic anisotropy.** TBD owns laminated
electrical models and electrical anisotropy. RPH owns Backus/TI elastic tensors and anisotropic fluid
substitution. An electrical-anisotropy result is never silently treated as an elastic tensor.

**`13_mineralogy-multimineral.md` (`MIN`) — mineral volumes and endpoint governance.** MIN owns
mineral-volume solutions and the product-wide endpoint registry. RPH consumes a versioned elastic
endpoint set and owns elastic mixing and endpoint completeness checks; it does not copy a vendor
lookup table into the product.

**`16_nmr.md` (`NMR`) — NMR fluid substitution.** NMR owns spectral-weight fluid substitution and
its water-salinity calibration. RPH owns non-NMR elastic substitution. A shared fluid-property object
may be consumed, but neither chapter silently changes the other's saturation basis.

**`18_geomech-ppfg.md` (`GEO`) — mechanical interpretation.** RPH produces dynamic elastic moduli and
anisotropy products. GEO owns static calibration, stress, pore pressure and fracture gradient.

**`23_plotting-interactivity.md` (`PLT`) — presentation.** PLT owns track rendering, crossplots and
interactive picking mechanics. RPH owns the equations, units, method identity and persisted picks
behind an elastic, AVO, image or core-photo view.

**`24_ml-advanced.md` (`MLA`) — advanced learning.** MLA owns Experienced Eye/EEFS, Domain Transfer
Analysis, Textural Facies, `Freq_Tiles` and vendor neural-weight needs. This chapter owns only the
in-domain C-3 needs: an independently derived image speed-correction alternative and a published-
literature dispersion workflow. Section 7.4 states their acquisition gates.

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 One Gassmann equation still needs explicit guards and one unit system

Three tools corroborate the same isotropic forward equation and two readable implementations give
an algebraically equivalent inverse [T1/T2]. The dangerous differences are around it: one help page
omits a squared `304.8`, another prints an inverted Mpsi/GPa relation, and an inverse can return a
plausible number across a singular denominator. The printed unit defects are 304.8× and 47.54×.
RPH therefore uses SI-with-GPa internally, validates the inverse before division, and emits no
physical result when `Kdry` is outside `(0,K0)` or `Ksat >= K0`.

### 2.2 Three-phase fluid mixing is not a cosmetic selector

Readable implementations agree that Brie first Reuss-mixes water and oil into a liquid, then mixes
that liquid with gas [T1]. A worked comparison produces 0.574, 0.116 and 0.846 GPa for three
different plausible interpretations—a factor of about 7 between two of them. `e = 1` is exactly
Voigt over `{liquid,gas}`, not three-phase Voigt; increasing `e` moves monotonically toward the gas
modulus. The method, saturation basis and exponent must therefore be explicit and provenance-bound.

### 2.3 Frame-model knobs that look alike are physically different

Critical porosity and depositional porosity are separate quantities even where a vendor happens to
ship the same number for both. Hertz–Mindlin adhesion fraction `f` changes the shear prefactor;
Techlog's `s_fact` is a later empirical multiplier on completed modified-HS shear. Fixing `f = 1`
instead of the evidenced `f = 0.5` variant changes `G_HM` by about 42% and `Vs` by about 19% over the
tested Poisson-ratio range [T1/T2]. RPH therefore keeps four separate registry entries:
`PHI_CRIT`, `PHI_DEPOSITIONAL`, `HM_ADHESION` and `SHEAR_SCALE`.

### 2.4 Bounds and endpoint provenance are part of the result

Two tools expose Voigt, Reuss, VRH and Hashin–Shtrikman bounds [T1/T3]. One vendor's solid-mixing
and shear-prediction modules ship different elastic endpoints—up to 5.8% on dolomite `Vs`—and seven
listed minerals have no shear endpoint at all. Five named clay rows share identical `Vp` and `Vs`
and differ only in density. RPH must compute bounds, refuse an incomplete required endpoint, and
stamp endpoint-set identity/version; it must not transcribe vendor lookup-table content.

### 2.5 Elastic anisotropy can remain plausible while being wrong

The complete readable Backus route produces all six TI stiffness terms and distinguishes the Voigt
`C66` from the Reuss `C44` [T1]. Swapping SH and SV changes shear velocity by 43% and flips the sign
of Thomsen gamma while leaving a positive-definite, plausible tensor. A vendor documents a 40–50°
relative-dip caution band but ships `FAST`; that conditional default is not adoptable. SH/SV
assignment, relative dip, fast-azimuth reference frame and the `C13` assumption must travel with
every result.

### 2.6 Seismic products need exact method identity and unit discipline

The held implementations span six AVO methods, exact Zoeppritz, linearized Shuey/Aki–Richards,
Ricker convolution, Elastic Impedance, LMR and Hilterman P5AI [T2/T3]. Elastic Impedance changes by
2.4× when a velocity unit is changed without re-evaluating the formula, so it is unit-system-
dependent and cannot be converted after calculation. Greenberg–Castagna is a km/s correlation; a
m/s call can be 21% wrong while still looking geologically reasonable.

### 2.7 Image processing needs reversible geometry and explicit conventions

The tools collectively cover accelerometer speed correction, per-channel offsets, image residual
correction, pad/button harmonization, equalization, true-dip rotation and navigation QC [T2–T4].
Mean dip must use `atan2` for direction; a cosine-only recovery mirrors western quadrants. Magnetic
declination can be applied twice if a user-controlled checkbox overrides already-corrected
navigation. RPH therefore persists correction curves, reference frames and a derived
`DECL_APPLIED` stamp, and makes every geometric correction reversible.

### 2.8 Fracture corrections have three valid but non-equivalent policies

Terzaghi correction is exposed as `EXCLUDE`, `CAP_ANGLE` and `CAP_WEIGHT` across the tools [T2–T4].
At an 80° pick the three cited configurations respectively drop it, keep weight 5.759, or keep
weight 5.0. Fracture density also depends on window height. A result without window, step, policy,
limit, correction state and angle convention is not reproducible and must not be written.

### 2.9 Image porosity and fracture aperture are calibration workflows

Image porosity has Archie-per-pixel, calibrated conductivity and Newberry-style scaling routes
[T3/T4]. Vendor `a=1,m=n=2` values are not transferable interval defaults. Luthi–Souhaite evidence
conflicts on `b` (0.8 versus approximately 0.863) and on `Rm` versus `Rmf`; the latter can change an
aperture by 1.4–2.4×. RPH ships those parameters absent and requires the resistivity convention by
name.

### 2.10 Core photographs are measurements, not decoration

The vendor evidence provides photo-to-array ingestion but little photometric conditioning [T3/T4].
The current product already goes further: it preserves a source copy, proposes rather than silently
applies conditioning, keeps fractional lane geometry, distinguishes white-light from ultraviolet
traces, and refuses interval products for point photographs. The obligation is to preserve and
test that work while keeping image-derived proxies distinct from petrophysical `VSH`, lithology or
porosity.

## 3. SandiBumi as-built

### 3.1 Fluid substitution, rock physics and seismic

`ABSENT` — the deterministic module registry contains 51 manifests but no Gassmann, fluid-mixing,
dry-frame, Backus, anisotropic-substitution, AVO or rock-physics-inversion module
(`src-tauri/src/modules.rs:434-486`). Its dispatcher likewise has no such branch
(`src-tauri/src/modules.rs:507-574`). The only nearby calculation is a dynamic-moduli helper inside
the unconventional workflow (`src-tauri/src/unconventional.rs:546`); it is not a rock-physics
workflow.

### 3.2 Borehole-image processing

`ABSENT` — the generic image importer offers `FMI` as a delivery label
(`src/ui/imageImportDialog.ts:137`) and the database can store depth-registered raster deliveries,
but there is no borehole-image array processor, speed-correction route, pad/button harmonization,
dip picker, Terzaghi correction or fracture-aperture implementation in the module registry or IPC
surface (`src-tauri/src/modules.rs:434-574`; `src/ipc.ts:3182-3461`). A raster labelled as an image
delivery is not an interpreted borehole image.

### 3.3 Core-photo import and conditioning

`PRESENT-OK` for the non-destructive source/conditioned split and explicit application. The recipe
contains rotation, perspective, crop, colour and detail controls (`src-tauri/src/coreimage.rs:77-194`);
`bake_core_images` writes a conditioned derivative (`coreimage.rs:351-475`). The recommendation path
reads the delivered source, proposes settings, never applies them, and declines white-light
correction on likely ultraviolet frames (`coreimage.rs:2217-2303`). The controls are exposed through
`src/ui/coreConditionDialog.ts:921-1057` and `src/ipc.ts:3205-3284`.

### 3.4 Core-photo depth geometry and proxy logs

`PRESENT-OK` for interval refusal, fractional lane geometry, versioned outputs and proxy naming.
`CoreLogSpec` stores lane/depth declarations, illumination, lithology-cut and unfold controls
(`src-tauri/src/coreimage.rs:928-1033`). A photograph without a finite base deeper than its top is
refused as a point sample (`coreimage.rs:1247-1283`). White-light/ultraviolet semantics and
`CPHOTO_LITH` refusal on ultraviolet input are explicit (`coreimage.rs:1504-1572`), and generated
curves remain readable by the ordinary curve path (`coreimage.rs:4430-4461`).

### 3.5 Core-photo lane detection and strips

`PRESENT-OK` for proposal-only lane detection and inspectable strip geometry. Lane detection is
implemented at `src-tauri/src/coreimage.rs:1974-2015`; the strip builder rotates and restacks lanes
once into an ordinary depth-registered image and refuses to overwrite its source delivery
(`coreimage.rs:2516-2676`). The UI calls both routes (`src/ui/coreTraceDialog.ts:617-1043`).

### 3.6 Remaining as-built limits

`PARTIAL` — current core-photo processing is extensive, but it is not a borehole-image workflow and
does not implement the dossier's calibrated image porosity, fracture aperture/intensity,
declination or Terzaghi outputs. Its recommendation heuristics are source-visible and tested, but
their product-level calibration against counted core remains an open validation item.

## 4. Requirements

### 4.1 Canonical elastic state and substitution

#### SB-RPH-001 — Use one typed SI-with-GPa elastic state [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST store rock-physics moduli in GPa, velocities in m/s, slowness in
µs/m, density in kg/m³, pressure in MPa, temperature in °C, salinity in ppm and fractions in v/v;
conversion MUST occur only at typed I/O boundaries.

**Rationale.** Unit defects in the held help produce 304.8× and 47.54× errors (§2.1; T1/T2).

**As-built.** `ABSENT` — no rock-physics state exists (`modules.rs:434-574`).

**Verified by.** SB-RPH-T01, SB-RPH-T02, SB-RPH-T03

#### SB-RPH-002 — Derive the complete isotropic elastic suite [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST derive `K`, `G`, `E`, Poisson ratio, `M`, lambda, `AIp`, `AIs`,
`Vp/Vs`, lambda-rho, mu-rho and P5AI from typed velocity and density inputs, with missing or
non-physical inputs producing null plus a named flag.

**Rationale.** The 14-equation incumbent suite is the minimum useful bridge from logs to seismic;
clamping Poisson ratio would conceal bad input (§2.6; T1/T2).

**As-built.** `PARTIAL` — four dynamic quantities exist only inside the unconventional module
(`unconventional.rs:546`); there is no reusable suite.

**Verified by.** SB-RPH-T04, SB-RPH-T05

#### SB-RPH-003 — Implement guarded Gassmann forward and inverse [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the canonical forward and closed-form inverse in §5,
evaluate denominator and modulus guards before use, keep shear unchanged, and emit no physical
value when a guard fails.

**Rationale.** The algebra is three-way corroborated; silent singularity handling is not (§2.1;
T1/T2).

**As-built.** `ABSENT` — no dispatcher branch (`modules.rs:507-574`).

**Verified by.** SB-RPH-T06, SB-RPH-T07, SB-RPH-T08

#### SB-RPH-004 — Re-synthesize density and velocities from the substituted state [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST calculate new fluid and bulk density before `Vp`/`Vs`, MUST use the
selected mineral/fluid mixtures, and MUST not reuse observed density after saturation changes.

**Rationale.** All three substitution routes make density part of the new state (§5.2 dossier;
T1/T2).

**As-built.** `ABSENT` — no substitution state exists.

**Verified by.** SB-RPH-T06, SB-RPH-T09

#### SB-RPH-005 — Persist method, state and failure provenance [P0] [status: ABSENT]

**Requirement.** Every substitution output MUST record input set/version, initial and final
saturations, fluid and solid models, endpoint-set version, frame model, effective parameters,
canonical units and a semantic flag; failed samples MUST remain null.

**Rationale.** A plausible curve without its saturation and model basis is not reproducible
(dossier §§5.4–5.6; T1–T3).

**As-built.** `ABSENT` — no domain output exists.

**Verified by.** SB-RPH-T10, SB-RPH-T11

#### SB-RPH-006 — Derive fluid properties from published physics [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide brine, oil and gas density/modulus calculations derived
from Batzle and Wang (1992), with their published pressure, temperature, salinity and composition
domains; until the paper is held, it MUST accept sourced fluid properties and ship no empirical
coefficient default.

**Rationale.** The only readable implementation carries a useful unit contract but is vendor code;
the product method must trace to the paper (T1; dossier E-7).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T12

### 4.2 Fluid, mineral and dry-frame mixing

#### SB-RPH-007 — Select a named fluid-mixing law [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST expose `REUSS`, `VOIGT` and `BRIE` as named laws and MUST NOT blend
them through an unlabeled continuous factor.

**Rationale.** Named laws are independently auditable; incumbent `Woodfac`/`Voigtfac` dials obscure
the physical assumption (§2.2; T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T13, SB-RPH-T14

#### SB-RPH-008 — Preserve Brie's liquid-lumping semantics [P1] [status: ABSENT]

**Requirement.** `BRIE` MUST Reuss-mix water and oil within the liquid, then mix liquid with gas;
the exponent and saturation basis MUST be explicit.

**Rationale.** Alternative lumping changes the fixture by as much as about 7× (§2.2; T1).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T13, SB-RPH-T14, SB-RPH-T15

#### SB-RPH-009 — Compute mineral bounds beside every mixture [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST expose Voigt, Reuss and VRH and compute Hashin–Shtrikman lower and
upper bounds beside every supported mixture; an out-of-bound selected result MUST be flagged.

**Rationale.** Bounds are a low-cost validity gate present in two incumbents (§2.4; T1/T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T16

#### SB-RPH-010 — Govern elastic endpoints without copying vendor tables [P0] [status: ABSENT]

**Requirement.** Each endpoint MUST be sourced independently, versioned and complete for the
selected operation. SandiBumi MUST NOT transcribe a vendor lookup table; it MUST refuse a shear-
dependent calculation when a selected phase has no shear endpoint.

**Rationale.** Same-vendor endpoint sets differ by up to 5.8%, and missing cells are not zero
(§2.4; T3; `CONTRACT.md` §2.1).

**As-built.** `ABSENT` — no elastic endpoint registry is present.

**Verified by.** SB-RPH-T17, SB-RPH-T18

#### SB-RPH-011 — Keep critical and depositional porosity distinct [P0] [status: ABSENT]

**Requirement.** `PHI_CRIT` and `PHI_DEPOSITIONAL` MUST be separate required parameters with
separate provenance and MUST never alias, even when their values happen to match.

**Rationale.** They control different equations and two vendors expose them separately (§2.3;
T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T19

#### SB-RPH-012 — Implement critical-porosity and suspension domains [P1] [status: ABSENT]

**Requirement.** The critical-porosity model MUST reduce both dry moduli to zero at `phi_c` and use
the Reuss suspension domain above it; `PHI_CRIT` ships without a default.

**Rationale.** One incumbent incorrectly preserves shear at critical porosity (§2.3; T2/T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T20

#### SB-RPH-013 — Implement Krief with the cited exponent [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the corroborated Krief dry/saturated form and expose its
exponent as a sourced parameter.

**Rationale.** Two tools print the same formulation and exponent (§5.2 dossier; T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T21

#### SB-RPH-014 — Require Hertz–Mindlin adhesion explicitly [P0] [status: ABSENT]

**Requirement.** The Hertz–Mindlin route MUST require `HM_ADHESION`, MUST implement the full
cube-root shear expression, and MUST ship no adhesion default. A stiff-sand branch that forces
`f=1` MUST say so in the output.

**Rationale.** Silent `f=1` changes shear by about 42%; one printed equation omits a cube root and
would be wrong by about 20 orders (§2.3; T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T22, SB-RPH-T23

#### SB-RPH-015 — Keep empirical shear scaling separate [P1] [status: ABSENT]

**Requirement.** `SHEAR_SCALE` MUST be applied only after modified-HS interpolation, default to the
identity `1.0`, and be stamped whenever not equal to identity; it MUST NOT be labeled or used as
Hertz–Mindlin adhesion.

**Rationale.** Readable source proves the two knobs occupy different equations (§2.3; T1).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T24

#### SB-RPH-016 — Distinguish soft, stiff and external dry frames [P2] [status: ABSENT]

**Requirement.** Soft/friable and stiff modified-HS branches MUST be named, independently tested
and recorded per interval; an external dry-frame curve MUST carry its own provenance.

**Rationale.** Incumbent model selectors are zoned and materially different (§2.4 dossier; T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T25

#### SB-RPH-017 — Gate effective-medium models by their validity domains [P2] [status: ABSENT]

**Requirement.** Kuster–Toksöz, SCA and DEM MUST be independently derived from their cited
literature and MUST reject or flag invalid pore concentration/aspect-ratio states; no compiled
vendor algebra may be reconstructed.

**Rationale.** Help pages name methods and limits but hide algebra (T3; dossier E-6).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T26

#### SB-RPH-018 — Support finite-shear pore fillers only from primary equations [P2] [status: ABSENT]

**Requirement.** Generalized substitution for finite-shear pore fillers MUST be derived from Ciz
and Shapiro (2007), emit HS validity bounds, and remain unavailable until that source is held.

**Rationale.** The capability is documented but its vendor implementation is compiled (T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T27

#### SB-RPH-019 — Specify Bayesian inversion without cloning its solver [P3] [status: ABSENT]

**Requirement.** A future inversion MUST expose priors and standard deviations, L1/L2 choice,
optional density/resistivity channels and effective-fluid-modulus mode, but its objective and solver
MUST be independently sourced before implementation.

**Rationale.** Only the API contract is readable; the mathematics is opaque (T1; dossier E-14).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T28

### 4.3 Shear prediction and anisotropy

#### SB-RPH-020 — Lock empirical shear correlations to their native units [P1] [status: ABSENT]

**Requirement.** Every shear correlation MUST carry a lithology identity and native-unit type.
Greenberg–Castagna MUST accept km/s only and MUST ship no coefficients until the primary paper is
held; analyst coefficients require source provenance.

**Rationale.** The held coefficients are single-vendor raster readings and a unit slip changes the
answer by 21% (§2.6; T2/T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T29, SB-RPH-T30

#### SB-RPH-021 — Keep alternative shear methods semantically addressed [P2] [status: ABSENT]

**Requirement.** Mudrock, Han, modified-GC, modulus, Poisson and fixed-`Vp/Vs` methods MUST be named
by physics, not ordinal position, and their branch cutoffs/coefficients MUST be sourced or absent.

**Rationale.** One incumbent exposes seven distinct methods and inconsistent mineral labels
(dossier §§2.5–2.6; T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T31

#### SB-RPH-022 — Produce a complete Backus TI tensor [P1] [status: ABSENT]

**Requirement.** Backus upscaling MUST produce `C11,C12,C13,C33,C44,C66`, thickness-weighted
density, four directional velocities and Thomsen `epsilon,delta,gamma`; `C66` MUST come from the
Voigt shear average and `C44` from the Reuss shear average.

**Rationale.** A partial incumbent equation set cannot compute its own SH impedance (§2.5; T1/T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T32, SB-RPH-T33

#### SB-RPH-023 — Make SH/SV assignment an explicit measured decision [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST require `SH_ASSIGNMENT` or derive it from `AZIM_FAST` with a declared
reference frame; it MUST record relative dip and flag, not auto-choose within the 40–50° caution
band.

**Rationale.** A silent swap changes shear velocity 43% and flips gamma without making the tensor
obviously invalid (§2.5; T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T34, SB-RPH-T35

#### SB-RPH-024 — Separate TIV, tilted-TIV and orthotropic input contracts [P2] [status: ABSENT]

**Requirement.** Plain TIV MUST run from `DTCO,DTSM,DTST,RHOB`; tilted-TIV and orthotropic routes
MUST declare their additional cross-dipole and geometry inputs. Unused inputs MUST NOT be exposed.

**Rationale.** One help page wrongly encourages a universal input contract; plain TIV does not need
cross-dipole or relative dip (T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T36

#### SB-RPH-025 — Reject non-positive-definite stiffness states [P0] [status: ABSENT]

**Requirement.** Every isotropic or anisotropic substitution MUST validate mineral and output
stiffness positive-definiteness before writing curves; weak-anisotropy substitution MUST flag
`delta > 0.3`.

**Rationale.** A readable API validates positive definiteness and another tool states the weak-
anisotropy limit (§2.5; T1/T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T37, SB-RPH-T38

### 4.4 Elastic attributes, AVO and synthetics

#### SB-RPH-026 — Emit the full named elastic-attribute suite [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST emit the §4.1 suite with equation identity, unit and source; lambda-
rho and mu-rho MUST remain distinct named products and P5AI MUST identify its Hilterman basis.

**Rationale.** Compacting the suite hides attributes analysts actually deliver (§2.6; T2).

**As-built.** `ABSENT` as a reusable suite.

**Verified by.** SB-RPH-T04, SB-RPH-T39

#### SB-RPH-027 — Treat Elastic Impedance as unit-system-dependent [P1] [status: ABSENT]

**Requirement.** EI MUST be computed pointwise, stamped with its input unit system and angle, and
MUST refuse post-hoc unit conversion.

**Rationale.** EI changes 2.4× under a velocity-unit change (§2.6; T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T40

#### SB-RPH-028 — Provide exact and declared approximate reflectivity [P2] [status: ABSENT]

**Requirement.** The reflectivity surface MUST offer exact Zoeppritz and separately named Shuey and
Aki–Richards approximations. Bortfeld and Hilterman branches MUST remain unavailable until numeric
validation fixtures are sourced.

**Rationale.** One tool ships six branches; only exact Zoeppritz remains valid at all offsets
(§2.6; T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T41, SB-RPH-T42

#### SB-RPH-029 — Build synthetics from explicit wavelet and sampling state [P2] [status: ABSENT]

**Requirement.** A convolutional synthetic MUST record wavelet family, frequency, phase, sample
rate, reflectivity method and time/depth transform; frequency outside the cited envelope MUST be
refused.

**Rationale.** The evidenced Ricker workflow supplies explicit frequency limits (§5.3 dossier; T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T43

#### SB-RPH-030 — Keep simple dispersion distinct from fitted dispersion [P3] [status: ABSENT]

**Requirement.** The cited scalar correction MAY be offered as `SIMPLE_FACTOR` with its factor
recorded. A frequency-dependent fit MUST use the independently derived route in §7.4 and MUST never
be inferred from a vendor implementation.

**Rationale.** The two capabilities have different evidence and Tier-C status (T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T44

### 4.5 Borehole-image processing and interpretation

#### SB-RPH-031 — Consume array logs without flattening their geometry [P1] [status: ABSENT]

**Requirement.** Borehole-image processing MUST consume DIO array logs with channel/pad/azimuth,
sample depth, orientation and missingness intact and MUST write a new versioned set.

**Rationale.** A raster import is insufficient for tool-space corrections (§2.7; T2–T4).

**As-built.** `ABSENT` — only raster delivery labels exist (`imageImportDialog.ts:137`).

**Verified by.** SB-RPH-T45

#### SB-RPH-032 — Make image geometry corrections reversible [P0] [status: ABSENT]

**Requirement.** Speed, channel-offset, residual-image and orientation corrections MUST emit the
applied displacement/orientation curves and MUST be reversible from persisted provenance.

**Rationale.** Geometry edits otherwise cannot be audited or undone (§2.7; T2–T4).

**As-built.** `ABSENT` — no borehole-image IPC route (`ipc.ts:3182-3461`).

**Verified by.** SB-RPH-T46, SB-RPH-T47

#### SB-RPH-033 — Condition buttons and pads after speed correction [P1] [status: ABSENT]

**Requirement.** Button/pad harmonization, bad-button repair, inpainting and static/dynamic
equalization MUST occur after geometric correction; each operation MUST preserve a residual or
mask and MUST record its window.

**Rationale.** Tool manuals agree that residual alignment precedes harmonization (§2.7; T3/T4).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T48

#### SB-RPH-034 — Recover dip direction with full quadrants [P1] [status: ABSENT]

**Requirement.** Dip direction MUST use `atan2` in a declared East/North frame; planar features
MUST use an axial/eigenvector mean rather than a vector mean.

**Rationale.** The incumbent cosine form mirrors two quadrants and vector averaging cancels
opposite poles (§2.7; T2/T4).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T49, SB-RPH-T50

#### SB-RPH-035 — Prevent magnetic-declination double application [P0] [status: ABSENT]

**Requirement.** Navigation output MUST carry `DECL_APPLIED`, `DECL_VALUE` and `DECL_SOURCE`; any
second correction MUST be refused from the stamp rather than allowed by a checkbox.

**Rationale.** The documented workflow can double-apply declination without warning (§2.7; T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T51, SB-RPH-T52

#### SB-RPH-036 — Calibrate image porosity to interval electrical parameters [P2] [status: ABSENT]

**Requirement.** Archie-per-pixel, calibrated conductivity and Newberry scaling MUST be separately
named routes. Archie parameters and fitted coefficients MUST come from the matching interval study
and ship absent.

**Rationale.** The vendor's textbook triple and example regression are not transferable (§2.9;
T3/T4).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T53

#### SB-RPH-037 — Require calibrated fracture-aperture constants and convention [P1] [status: ABSENT]

**Requirement.** Luthi–Souhaite aperture MUST require `b`, `c` and a named `RM` or `RMF` convention,
record integration radius and image sampling, and mark vendor starting values `QUALITATIVE`.

**Rationale.** The held exponent/convention disagreements move aperture by about 16% and 1.4–2.4×
respectively (§2.9; T3/T4).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T54, SB-RPH-T55

#### SB-RPH-038 — Expose all three Terzaghi policies [P1] [status: ABSENT]

**Requirement.** Terzaghi correction MUST be opt-in and require `EXCLUDE`, `CAP_ANGLE` or
`CAP_WEIGHT`; the angle MUST be from borehole axis to feature-plane normal, with weight floored at
1.0.

**Rationale.** The policies differ in numerator as well as weight (§2.8; T2–T4).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T56, SB-RPH-T57

#### SB-RPH-039 — Compute fracture intensity with explicit geometry [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST support counts plus P21/P22/P32/P33 where required geometry exists;
P22/P33 MUST refuse without aperture/width input and partial picks MUST declare their covered arc.

**Rationale.** These are the documented measures needed by a fracture-model handoff (T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T58

#### SB-RPH-040 — Name pooled and area-weight statistics separately [P2] [status: ABSENT]

**Requirement.** Image/fracture statistics MUST emit `MEAN_POOLED` and `MEAN_OF_AREAS` separately;
neither may be named bare `mean`.

**Rationale.** A 3-frame and 300-frame example gives 0.200 versus 0.1020 (§2.8; T2).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T59

#### SB-RPH-041 — Refuse metadata-free fracture outputs [P0] [status: ABSENT]

**Requirement.** Density, spacing or weighted-density writers MUST require window, step, policy,
limit, correction state and angle convention. Empty-window spacing MUST be null, never zero or
infinity.

**Rationale.** Density is window-dependent and spacing is undefined without picks (§2.8; T2/T3).

**As-built.** `ABSENT`.

**Verified by.** SB-RPH-T60, SB-RPH-T61

### 4.6 Core-photo processing

#### SB-RPH-042 — Preserve non-destructive core-photo conditioning [P0] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST retain the delivered source, store an explicit recipe, preview
before apply, and make reset/restore possible without cumulative resampling.

**Rationale.** Core photographs are calibration evidence; irreversible display edits destroy the
measurement basis (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:77-194,351-475`; UI at
`coreConditionDialog.ts:921-1057`.

**Verified by.** SB-RPH-T62, SB-RPH-T63

#### SB-RPH-043 — Separate colour correction from detail-changing operations [P1] [status: PRESENT-OK]

**Requirement.** Colour/exposure transforms and denoise/clarity/sharpen MUST be separately
provenanced; quantitative traces MUST state which conditioned derivative they consume.

**Rationale.** Detail operations directly alter darkness and texture proxies (§2.10).

**As-built.** `PRESENT-OK` — recipe `touches_detail`/`touches_light` separation and documented trace
effects (`coreimage.rs:124-194`).

**Verified by.** SB-RPH-T64

#### SB-RPH-044 — Require interval geometry before core-log extraction [P0] [status: PRESENT-OK]

**Requirement.** Every core-photo interval trace MUST require finite top/base and explicit lane
order/direction; point photographs MUST be refused for interval extraction.

**Rationale.** Pixels do not contain depth semantics (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:1247-1283`.

**Verified by.** SB-RPH-T65

#### SB-RPH-045 — Keep white-light and ultraviolet meanings distinct [P0] [status: PRESENT-OK]

**Requirement.** Illumination MUST be declared. White-light darkness/colour/texture and ultraviolet
fluorescence MUST use different mnemonics, thresholds and conditioning rules.

**Rationale.** A dark ultraviolet background is signal context, not underexposure (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:957-989,1677-1704,2286-2303`.

**Verified by.** SB-RPH-T66, SB-RPH-T67

#### SB-RPH-046 — Keep image-derived lithology a labeled proxy [P1] [status: PRESENT-OK]

**Requirement.** The two-class white-light curve MUST remain `CPHOTO_LITH`, record its cut source,
and MUST NOT be named `VSH` or a definitive lithology; it MUST be refused on ultraviolet input.

**Rationale.** Darkness covaries with lithology but is not a mineral solution (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:657-661,1504-1572`.

**Verified by.** SB-RPH-T68, SB-RPH-T69

#### SB-RPH-047 — Preserve fractional lane geometry and inspectable strips [P1] [status: PRESENT-OK]

**Requirement.** Lane edges and unfolding MUST be stored as fractional geometry; strip building
MUST preserve source depth intervals, use a separate delivery and create a renderer-independent,
inspectable image.

**Rationale.** Baking layout once prevents screen/export geometry drift (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:1974-2015,2516-2676`.

**Verified by.** SB-RPH-T70, SB-RPH-T71

#### SB-RPH-048 — Keep automatic core advice proposal-only [P1] [status: PRESENT-OK]

**Requirement.** Automatic lane, unfold and conditioning advice MUST report its measurements and
reasons and MUST require user application; weak or flat evidence MUST produce no proposal.

**Rationale.** A hidden auto-correction is an undocumented parameter (§2.10).

**As-built.** `PRESENT-OK` — `coreimage.rs:1974-2015,2217-2422,4870-5091`.

**Verified by.** SB-RPH-T72, SB-RPH-T73

### 4.7 Execution, validity and audit

#### SB-RPH-049 — Make every method batch-safe and versioned [P1] [status: ABSENT]

**Requirement.** RPH methods MUST run through the shared multi-well runner, preserve per-zone
parameters, write new log sets, honor cancellation and never overwrite imported curves.

**Rationale.** A desktop method is not product-complete if it bypasses the field-scale execution
contract.

**As-built.** `ABSENT` for RPH modules; core-photo writes are versioned but are separate IPC routes.

**Verified by.** SB-RPH-T74

#### SB-RPH-050 — Validate method-specific inputs before calculation [P1] [status: ABSENT]

**Requirement.** Every selector MUST have a total, named input contract and every parameter domain
MUST be validated before any sample is calculated; an unknown method or option MUST hard-fail.

**Rationale.** Silent option fallback is indistinguishable from a successful different method
(dossier §5.6 rule 14).

**As-built.** `ABSENT` for this domain.

**Verified by.** SB-RPH-T75

#### SB-RPH-051 — Persist enough provenance to reproduce every number [P0] [status: PARTIAL]

**Requirement.** Every RPH output MUST record equation/method version, effective parameters with
sources, input set/version, unit conversions, flags, window/resolution and user-confirmed picks or
conventions.

**Rationale.** Method identity without effective values cannot reproduce a result.

**As-built.** `PARTIAL` — core-photo sets and recipes carry much of this state
(`coreimage.rs:928-1033,1400-1704`); no common RPH provenance contract exists.

**Verified by.** SB-RPH-T10, SB-RPH-T76

#### SB-RPH-052 — Never accept and ignore a parameter [P1] [status: ABSENT]

**Requirement.** A parameter that is unused in the selected branch MUST be hidden or refused; a
QC-only input MUST be labeled QC-only and MUST NOT appear in the calculation provenance.

**Rationale.** One navigation workflow exposes unused inputs, a documented design defect
(dossier K-6.8; T2).

**As-built.** `ABSENT` for this domain.

**Verified by.** SB-RPH-T77

## 5. Parameters

`ABSENT — ships with no default` is a deliberate product value. It means the run must obtain a
sourced value from the analyst or a versioned study. Vendor endpoint-table contents are not
transcribed here (`CONTRACT.md` §2.1).

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Canonical modulus unit | `U_MOD` | `GPa` | enum | Dossier §5.1 canonical unit contract | T1/T2 |
| Canonical velocity unit | `U_VEL` | `m/s` | enum | Dossier §5.1 | T1/T2 |
| Canonical slowness unit | `U_DT` | `µs/m` | enum | Dossier §5.1 | T1/T2 |
| Canonical density unit | `U_RHO` | `kg/m³` | enum | Dossier §5.1 | T1/T2 |
| Canonical pressure unit | `U_P` | `MPa` | enum | Dossier §5.1 | T1/T2 |
| Canonical temperature unit | `U_T` | `°C` | enum | Dossier §5.1 | T1/T2 |
| Canonical salinity unit | `U_SAL` | `ppm` | enum | Dossier §5.1 | T1/T2 |
| Mpsi conversion check | `MPSI_TO_GPA` | `6.894757` | GPa/Mpsi | Dossier §5.5 T-9; corrected cross-tool conversion | T1/T2 |
| Gassmann denominator tolerance | `EPS_GASSMANN` | **ABSENT — ships with no default** | dimensionless | Dossier §5.2 names `eps` but supplies no value | T1/T2 |
| Fluid-property pressure | `P_FL` | **ABSENT — ships with no default** | MPa | Batzle & Wang (1992) input contract; dossier §§4.1,5.1 | primary needed |
| Fluid-property temperature | `T_FL` | **ABSENT — ships with no default** | °C | Same | primary needed |
| Brine salinity | `SAL` | **ABSENT — ships with no default** | ppm | Same | primary needed |
| Oil/gas composition inputs | `FL_COMP` | **ABSENT — ships with no default** | typed set | Same | primary needed |
| Fluid mixing law | `FLUID_MIX` | **ABSENT — ships with no default** (`REUSS` / `VOIGT` / `BRIE`) | enum | Dossier §§2.3,4.1 | T1/T2 |
| Brie exponent | `E_BRIE` | `3` | dimensionless | IP help default and Geolog `fluid_mix.lls`, dossier §5.3 | T1/T2 |
| Frequency policy | `FREQ_POLICY` | `REUSS_SEISMIC_BRIE_LOG` | enum | IP documented split, dossier §4.1 | T2 |
| Mineral mixing law | `MINERAL_MIX` | `VRH` | enum | Dossier §4.1; HS bounds computed alongside | T1/T3 |
| Elastic endpoint set | `ENDPOINT_SET` | **ABSENT — ships with no default** | version id | Dossier §§2.5,5.3; vendor tables non-transcribable under CONTRACT §2.1 | T3 |
| Critical porosity | `PHI_CRIT` | **ABSENT — ships with no default** | v/v | Dossier §5.3: vendors ship 0.39 / 0.36 / 0.4; no adjudication | T1/T2/T3 |
| Depositional porosity | `PHI_DEPOSITIONAL` | **ABSENT — ships with no default** | v/v | Dossier §5.3: vendors ship 0.39 / 0.4 and one conflates it; no adjudication | T1/T2/T3 |
| Krief exponent | `M_KRIEF` | `3` | dimensionless | IP help + Geolog `mod_krief.lls`, dossier §5.3 | T1/T2 |
| Hertz–Mindlin adhesion | `HM_ADHESION` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3; cited variants 0.5 and forced 1 conflict with implicit implementations | T1/T2 |
| Empirical shear scale | `SHEAR_SCALE` | `1.0` | dimensionless | Techlog function signature identity; dossier §5.3 | T1 |
| Coordination number | `COORDINATION` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3: cited values 9, 9, 6 and forced 15 disagree | T1/T2/T3 |
| Effective pressure | `P_EFFECTIVE` | **ABSENT — ships with no default** | MPa | Dossier §5.3 cites 5000 psi only as field-specific vendor value | T2 |
| Kuster–Toksöz aspect ratio | `KT_ASPECT` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3 carries conflicting model-specific ratios; primary equations pending | T1/T3 |
| Effective-medium pore fractions | `PORE_FRACTIONS` | **ABSENT — ships with no default** | v/v | Dossier §5.3; must match selected pore system | T1/T3 |
| Greenberg–Castagna coefficients | `GC_A_B_C` | **NON-ADOPTABLE — cited for verification** | km/s coefficients | Single-vendor rasters; Greenberg & Castagna (1992) is missing, dossier E-12 | T2; primary needed |
| Greenberg–Castagna input unit | `GC_UNIT` | `km/s` | enum | IP help and dossier §§2.6,5.1 | T2 |
| Mudrock coefficients | `MUDROCK_A_B` | **ABSENT — ships with no default** | km/s coefficients | Dossier §5.3 records one vendor line; primary source not held | T3 |
| Han coefficients and cutoffs | `HAN_CFG` | **ABSENT — ships with no default** | typed set | Dossier §5.3 records vendor branches; primary source not held | T3 |
| Backus averaging window | `BACKUS_WINDOW` | **ABSENT — ships with no default** | depth | Dossier §5.6 rule 15 requires logging but supplies no value | T1 |
| Anisotropy model | `ANISO_MODEL` | **ABSENT — ships with no default** (`TIV` / `TIV_TILTED` / `ORTHOTROPIC`) | enum | Dossier §5.3 records a vendor `TIV-tilted` default but says select from data | T3 |
| SH/SV assignment | `SH_ASSIGNMENT` | **ABSENT — ships with no default** (`FAST` / `SLOW` / derived) | enum | Dossier §5.3; vendor `FAST` is conditional | T3 |
| SH/SV caution band | `SHSV_CAUTION` | `40–50` | ° relative dip | `tp_geom_anisotropic.html`, dossier §5.3; range only, never branch threshold | T3 |
| Fast-azimuth reference | `AZIM_FAST_FRAME` | **ABSENT — ships with no default** (`TOH` / `NORTH`) | enum | Same | T3 |
| Drilling-fluid density | `DFD` | **ABSENT — ships with no default** | g/cc | Same; vendor default cell empty | T3 |
| Borehole-fluid slowness | `DTF` | **ABSENT — ships with no default** | µs/ft | Same; vendor default cell empty | T3 |
| Weak-anisotropy delta limit | `DELTA_MAX` | `0.3` | dimensionless | `tp_fluid_substitution_anisotropy.html`, dossier §5.3 | T3 |
| AVO method | `AVO_METHOD` | **ABSENT — ships with no default** | enum | Dossier §§1.2,4.1; exact/approximate identity must be selected | T3 |
| Incidence-angle maximum | `ANGLE_MAX` | `60` | ° | IP help validity envelope, dossier §5.3 | T2 |
| Ricker frequency minimum | `F_RICKER_MIN` | `5` | Hz | IP help, dossier §5.3 | T2 |
| Ricker frequency maximum | `F_RICKER_MAX` | `0.4/sample-rate` | Hz | IP help, dossier §5.3 | T2 |
| Nth-root stack exponent | `N_STACK` | `4` | dimensionless | IP help hard-coded value, dossier §5.3 | T2 |
| Simple dispersion factor | `DISP_SIMPLE` | `0.98` | dimensionless | IP help, dossier §5.3 | T2 |
| Declination value | `DECL_VALUE` | **ABSENT — ships with no default** | ° | Dossier §§2.9,5.4; source measurement required | T2 |
| Declination source | `DECL_SOURCE` | **ABSENT — ships with no default** | provenance id | Same | T2 |
| Image speed-correction model | `IMG_SPEED_MODEL` | **ABSENT — ships with no default** | enum | Dossier §§2.9,4.1; independent derivation gate in §7.4 | T2/T4 + primary needed |
| Image correction window | `IMG_WINDOW` | **ABSENT — ships with no default** | frames | Dossier §5.6 rule 15 | T2–T4 |
| Image sand-side convention | `SAND_IS` | **ABSENT — ships with no default** (`ABOVE` / `BELOW`) | enum | Dossier K-6.6 | T2 |
| Image porosity Archie `a` | `A_IMG` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3; vendor value 1 explicitly not adopted | T3 |
| Image porosity cementation exponent | `M_IMG` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3; vendor value 2 explicitly not adopted | T3 |
| Image porosity saturation exponent | `N_IMG` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3; vendor value 2 explicitly not adopted | T3 |
| Image-porosity fitted coefficients | `POR_FIT` | **ABSENT — ships with no default** | typed set | Dossier §5.3; SOP example is interval-specific | T4 |
| Image binarization window | `BIN_WINDOW` | `0.6` | m | Image SOP via dossier §5.3 | T4 |
| Histogram window | `HIST_WINDOW` | `101` | frames | Image SOP via dossier §5.3 | T4 |
| Histogram step | `HIST_STEP` | `1` | frame | Same | T4 |
| Histogram bins | `HIST_BINS` | `100` | bins | Same | T4 |
| Terzaghi policy | `TERZ_POLICY` | **ABSENT — ships with no default** (`EXCLUDE` / `CAP_ANGLE` / `CAP_WEIGHT`) | enum | Dossier §5.3; policies produce different curves | T2/T3/T4 |
| Terzaghi correction state | `TERZ_ON` | `FALSE` | boolean | `tp_fracture_density.html`, dossier §5.3 | T3 |
| Terzaghi blind zone | `TERZ_BLIND` | `20` | ° | Same; applies only to `EXCLUDE` | T3 |
| Terzaghi cap angle | `TERZ_CAP_ANGLE` | `85` | ° | Image training manual via dossier §5.3 | T4 |
| Terzaghi cap weight | `TERZ_CAP_WEIGHT` | **ABSENT — ships with no default** | dimensionless | IP help states user-supplied, dossier §5.3 | T2 |
| Terzaghi weight floor | `TERZ_FLOOR` | `1.0` | dimensionless | Image training manual via dossier §5.3 | T4 |
| Fracture window length | `FRAC_WINDOW` | `1.0` | m | `tp_fracture_density.html` and training manual, dossier §5.3 | T3/T4 |
| Fracture step length | `FRAC_STEP` | `1.0` | m | Same | T3/T4 |
| Luthi–Souhaite exponent | `LS_B` | **ABSENT — ships with no default** | dimensionless | Dossier §5.3: 0.8 vs approximately 0.863; primary paper missing | T3/T4 |
| Luthi–Souhaite coefficient | `LS_C` | **ABSENT — ships with no default** | tool-specific | Dossier §5.3: vendor 4 requires core calibration | T3 |
| Aperture resistivity convention | `LS_R_TERM` | **ABSENT — ships with no default** (`RM` / `RMF`) | enum | Dossier §5.3; sources conflict | T2/T3/T4 |
| Aperture integration radius | `LS_RADIUS` | `5` | frames | `tp_fracture_aperture.html`, dossier §5.3; vendor starting value | T3 |
| Aperture standard-deviation factor | `LS_STDDEV` | **ABSENT — ships with no default** | dimensionless | Vendor ships 1 but purpose is undocumented; dossier §5.3 says do not implement blind | T3 |
| Core-photo illumination | `CP_ILLUMINATION` | **ABSENT — ships with no default** (`WHITE` / `UV`) | enum | `coreimage.rs:957-989` requires declaration | source |
| Core-photo lane geometry | `CP_LANES` | **ABSENT — ships with no default** | fractional intervals | `coreimage.rs:928-1033` | source |
| Core-photo unfold | `CP_UNFOLD` | **ABSENT — ships with no default** | project depth unit | `coreimage.rs:1008-1033`; proposal never auto-applied | source |
| Core-photo lithology cut | `CP_LITH_CUT` | **ABSENT — Otsu proposes from this trace** | normalized darkness | `coreimage.rs:985-989,1504-1541` | source |
| Core-photo minimum bed | `CP_MIN_BED` | **ABSENT — ships with no default** | project depth unit | `coreimage.rs:1012` | source |

**Parameter count: 76. ABSENT count: 43. NON-ADOPTABLE count: 1.** Enumerations whose absence
forces an explicit method choice are counted as ABSENT; canonical units and cited identities are
not.

## 6. Acceptance tests

| Test | Input | Operation | Expected value / behavior | Source of expected value |
|---|---|---|---|---|
| **SB-RPH-T01** | Same elastic input expressed once in canonical SI and once in supported display units | Convert at input, compute, convert for display | All physical outputs agree to `1e-6` relative | Dossier §5.5 T-5; canonical equations §5.2 |
| **SB-RPH-T02** | `rho=2.4 g/cc`, `DTCO=70 µs/ft`, `DTS=130 µs/ft` | Convert and derive `K` | `20 <= K <= 50 GPa`; never `0.1–0.2 GPa` | Dossier §5.5 T-8 |
| **SB-RPH-T03** | `1 Mpsi` | Convert to GPa and back | `6.894757 GPa`, round trip within `1e-9` relative | Dossier §5.5 T-9 |
| **SB-RPH-T04** | Finite `Vp,Vs,rho` fixture | Derive named elastic suite | Each value matches the dossier §5.2 equations within `1e-9` relative; all names/units present | Dossier §5.2 canonical equations |
| **SB-RPH-T05** | `Vp<=0`, `Vs<=0`, `rho<=0`, and a state yielding an invalid denominator | Derive attributes | Physical outputs null; named input/physics flag, never clamped | Dossier §§5.2,5.4 |
| **SB-RPH-T06** | Any valid `K0,Kdry,Kfl,phi` fixture | Forward Gassmann, inverse with same fluid, forward again | Recovered `Ksat` within `1e-9` relative | Dossier §5.5 T-1 |
| **SB-RPH-T07** | Same valid fixture evaluated by increment, modulus-ratio and closed inverse forms | Compare | All agree within `1e-10` relative | Dossier §5.5 T-2 |
| **SB-RPH-T08** | Cases with `b=0`, `Kdry<=0`, `Kdry>=K0`, or `Ksat>=K0` | Invert | No physical result; `GASSMANN_FLAG=2` | Dossier §5.2 guards and §5.4 flag table |
| **SB-RPH-T09** | Valid observed state with changed fluid density and unchanged dry shear | Re-synthesize | `rho_new=(1-phi)rho_min+phi rho_fl`; `Vs=Vs_obs*sqrt(rho_obs/rho_new)` within `1e-9` | Dossier §5.2 |
| **SB-RPH-T10** | One successful and one failed substitution | Inspect output set | Successful header contains all SB-RPH-005 fields; failed physical values null and flags non-null | Dossier §§5.4–5.6 |
| **SB-RPH-T11** | A flag value numerically equal to a plausible physical value | Static/output-schema check | No physical curve contains flag-domain values | Dossier §5.5 T-20 |
| **SB-RPH-T12** | Missing primary fluid coefficients, but valid sourced `Kw,Ko,Kg,rho` inputs | Open fluid-property and direct-property routes | Derived-property route unavailable with missing-source message; direct-property route accepts sourced values | CONTRACT §2; dossier E-7 |
| **SB-RPH-T13** | `Sw,So,Sg>0`, distinct `Kw,Ko,Kg`, `e=1` | Compute Brie and Voigt variants | Brie equals two-phase `{liquid,gas}` Voigt exactly and differs from three-phase Voigt | Dossier §5.5 T-3 |
| **SB-RPH-T14** | Dossier §3.1 three-phase fixture | Compute specified mixing variants | `0.574`, `0.116`, `0.846 GPa` to `0.001 GPa` | Dossier §5.5 T-4 |
| **SB-RPH-T15** | Same gas-bearing fixture, `e=1,3,5,40` | Compute Brie | `Kfl` strictly decreases and approaches `Kg` | Dossier §5.5 T-3b |
| **SB-RPH-T16** | Two-mineral fractions swept 0…1 | Compute Voigt/Reuss/VRH/HS | VRH remains inside HS lower/upper within `1e-12` | Dossier §5.5 T-18 |
| **SB-RPH-T17** | Endpoint-set A versus B identity with no numeric content copied | Resolve endpoint registry | Different version ids; output records chosen set; no fallback between sets | Dossier §§2.5,5.3; CONTRACT §2.1 |
| **SB-RPH-T18** | Selected phase missing `Vs` in a shear-dependent mix | Run | Hard refusal naming phase and endpoint field; zero is not substituted | Dossier §2.5 and critique M-10 |
| **SB-RPH-T19** | `PHI_CRIT=x`, `PHI_DEPOSITIONAL=y`, then swap only `y` | Resolve parameters | Critical-porosity output unchanged; HM output changes; both provenance values retained | Dossier critique M-17 and §5.2 |
| **SB-RPH-T20** | `phi=phi_c` then `phi>phi_c` | Critical-porosity model | At equality `Kdry=Gdry=0`; above, suspension branch selected | Dossier §5.5 T-11 |
| **SB-RPH-T21** | Valid `K0,G0,Kfl,phi`, `m=3` | Krief | Output matches dossier §5.2 equations within `1e-9` relative | Dossier §5.2 and parameter §5.3 |
| **SB-RPH-T22** | Dossier §3.4 HM fixture | Compute HM with cube roots | `1 <= Ghm <= 50 GPa`; never `1e18 GPa` | Dossier §5.5 T-10 |
| **SB-RPH-T23** | Same `nu0` state at `f=0` and `f=1` | Evaluate shear prefactor | `f=0` gives exactly `1/5`; `f=1` gives `(5-4nu0)/(5(2-nu0))` within `1e-12` | Dossier §5.2 HM identities |
| **SB-RPH-T24** | Same modified-HS result at `SHEAR_SCALE=1.0` and `0.4` | Apply scale | Identity run unchanged; second is exactly `0.4*Gdry` and header records 0.4 | Techlog readable source summarized in dossier §5.3 |
| **SB-RPH-T25** | Same valid frame inputs through soft and stiff branches | Run | Distinct named outputs; stiff header states forced `f=1`; external path refuses missing provenance | Dossier §5.2 |
| **SB-RPH-T26** | `VOL_POAR/POAR=1` and `>1` | Kuster–Toksöz validity gate | Boundary accepted; exceedance refused/flagged before calculation | Dossier §5.3 validity row |
| **SB-RPH-T27** | Finite-shear filler request before Ciz–Shapiro source acquisition | Dispatch | Unavailable with escalation/source message, no fallback to fluid Gassmann | Dossier E-6; CONTRACT §2.2 |
| **SB-RPH-T28** | Bayesian inversion request before solver literature is held | Dispatch | Interface can be inspected, solver cannot run; E-14 named | Dossier E-14 |
| **SB-RPH-T29** | Greenberg–Castagna called with m/s-typed `Vp` | Dispatch | Hard unit error; no numeric output | Dossier §5.5 T-6 |
| **SB-RPH-T30** | GC request without primary-sourced/user-sourced coefficients | Dispatch | Refusal naming missing coefficient source | CONTRACT §2; dossier E-12 |
| **SB-RPH-T31** | Each shear selector plus an unknown string | Resolve method | Six named alternatives resolve to their own contracts; unknown hard-fails | Dossier GL-13 and §5.6 rule 7 |
| **SB-RPH-T32** | Dossier §3.8 two-layer fixture | Backus | `C66=17.5 GPa`, `C44=8.571 GPa`, `gamma=0.521`, `Vsh=2.70 km/s`, `Vsv=1.89 km/s`, tolerance `0.001` | Dossier §5.5 T-12 |
| **SB-RPH-T33** | Identical layers | Backus | `epsilon=delta=gamma=0` within `1e-12`; all four velocities equal layer values | Dossier §5.5 T-13 |
| **SB-RPH-T34** | Same tilted-TIV inputs, once `FAST`, once `SLOW` | Solve | `C44/C66` exchange, gamma changes sign, both tensors remain positive-definite | Dossier §5.5 T-12b |
| **SB-RPH-T35** | Relative dip 40°, 45°, 50° with either assignment | Solve | Every sample raises caution; no automatic reassignment | Dossier §5.5 T-12b and §5.3 range |
| **SB-RPH-T36** | Plain TIV with `DTCO,DTSM,DTST,RHOB` only | Solve | Full stiffness set returned; no cross-dipole/RDIP requirement | Dossier §5.5 T-12c |
| **SB-RPH-T37** | Non-positive-definite mineral matrix | Anisotropic substitution | Named hard refusal before output | Dossier §5.5 T-19 |
| **SB-RPH-T38** | Weak-anisotropy run at `delta=0.3` then `0.300001` | Validate | Boundary accepted; exceedance flagged | Dossier §5.3 validity row |
| **SB-RPH-T39** | One valid elastic state | Emit attributes | All §4.1 names emitted; lambda-rho/mu-rho/P5AI identities and units present | Dossier IP-10b, §2.8 |
| **SB-RPH-T40** | Existing EI curve plus post-hoc unit conversion request | Convert | Hard refusal | Dossier §5.5 T-7 |
| **SB-RPH-T41** | Normal-incidence two-layer interface with finite `rho1,Vp1,rho2,Vp2` | Exact and approximate reflectivity | All methods equal the derived normal-incidence value `R=(rho2*Vp2-rho1*Vp1)/(rho2*Vp2+rho1*Vp1)` within `1e-9` | Exact Zoeppritz and approximation identities at zero angle; primary references printed by `tp_avo.html`, dossier §2.8/GL-15 |
| **SB-RPH-T42** | Unknown AVO method and unvalidated Hilterman branch | Dispatch | Unknown hard-fails; unvalidated branch unavailable with reason | Dossier §4.1 reflectivity choice |
| **SB-RPH-T43** | Ricker request at `4.999 Hz`, `5 Hz`, and just above `0.4/sample-rate` | Validate | First and third refused; 5 Hz accepted when below upper bound | Dossier §5.3 frequency limits |
| **SB-RPH-T44** | `SIMPLE_FACTOR` at 0.98 and a frequency-fit request | Dispatch | Scalar output exactly `0.98*input`; fit route blocked on §7.4 source gate | Dossier §5.3 and Tier-C register |
| **SB-RPH-T45** | Array log with two pads, distinct azimuths and one missing button | Load RPH image frame | Pad/azimuth/missingness unchanged exactly | DIO array contract; dossier §2.9 capability contract |
| **SB-RPH-T46** | Synthetic image with known speed anomaly | Correct and invert correction | Applied displacement curve reconstructs original pixel-depth mapping exactly | Dossier speed-correction adoption; CONTRACT test in `20_envcorr-qc.md` SB-ENV-T68 |
| **SB-RPH-T47** | Two channels with distinct physical offsets | Apply same accelerometer motion at each offset | Each receives its offset-time correction; provenance records both offsets | Image training manual summarized in dossier §2.9 |
| **SB-RPH-T48** | Misaligned image with button bias | Run residual correction then harmonization | Residual alignment completes first; masks/residuals and window recorded | Dossier §4.1 speed correction; T4 workflow order |
| **SB-RPH-T49** | Dips at 45°, 135°, 225°, 315° | Recover azimuth | All four exact within `1e-9°` | Dossier §5.5 T-14 |
| **SB-RPH-T50** | Two planar poles 180° apart | Mean orientation | Axial mean stable; vector mean not used | Dossier §5.5 T-15 |
| **SB-RPH-T51** | Navigation correction sets `DECL_APPLIED=1`, then rendering requests correction | Apply | Second application refused by stamp | Dossier §5.5 T-17d |
| **SB-RPH-T52** | Declination `+2.5°` with matching and mismatching sources | Correct | Azimuth shifts exactly once by `2.5°`; mismatch warns; source/value retained | Dossier §5.5 T-17e |
| **SB-RPH-T53** | Image-porosity run without interval-sourced `a,m,n` or calibration coefficients | Dispatch | Refused; no vendor textbook defaults inserted | Dossier §5.3 image-porosity rows |
| **SB-RPH-T54** | Aperture request missing any of `b,c,LS_R_TERM` | Dispatch | Refusal names every missing item | Dossier §5.2 aperture contract |
| **SB-RPH-T55** | Same excess-current/Rxo input under explicit `RM` and `RMF` conventions | Compute | Distinct outputs and headers; no bare-resistivity overload exists | Dossier M-13/E-15; expected direction follows printed formula |
| **SB-RPH-T56** | 100 m; ten picks each at 0°,45°,87°; cap angle 85° | Terzaghi `CAP_ANGLE` | Weights `1.0,1.4142,11.4737`; raw `0.3/m`; corrected `1.389/m` within `0.001/m` | Dossier §5.5 T-16 |
| **SB-RPH-T57** | One pick at 80° | Run three policies | `CAP_ANGLE@85°=5.759` kept; `CAP_WEIGHT@5=5.0` kept; `EXCLUDE@20°` dropped | Dossier §5.5 T-17 |
| **SB-RPH-T58** | P22/P33 request without `FRACWIDTH`, then with it and complete geometry | Run | First refused by name; second returns both measures at window midpoint | `tp_fracture_density.html`, dossier GL-19 |
| **SB-RPH-T59** | Areas: 3 samples mean 0.30; 300 samples mean 0.10 | Aggregate | `MEAN_OF_AREAS=0.200`; `MEAN_POOLED=0.1020` | Dossier §5.5 T-22 |
| **SB-RPH-T60** | Same picks at 1 m and 5 m windows | Compute density | Values differ; each header carries window; writer refuses missing window metadata | Dossier §5.5 T-17b |
| **SB-RPH-T61** | Window with zero fractures | Compute density and spacing | Density `0`; spacing null, never `0` or infinity | Dossier §5.5 T-17c |
| **SB-RPH-T62** | Existing 400×200 BMP fixture and fractional crop `(x=0,y=0,w=0.5,h=1.0)` | Bake conditioned derivative | Live derivative is 200×200 JPEG; stored source remains byte-for-byte equal to the BMP input | Existing executable fixture `coreimage.rs:3704-3752` |
| **SB-RPH-T63** | The T62 fixture after conditioning | Apply the identity recipe/reset | Live bytes equal the original BMP byte-for-byte; width/height/mime return exactly to `400/200/image/bmp` | Existing executable fixture `coreimage.rs:3754-3766` |
| **SB-RPH-T64** | Same photo, colour-only recipe then detail recipe | Extract proxy provenance | Colour-only/detail flags differ and consumed derivative id is recorded | `coreimage.rs:124-194` |
| **SB-RPH-T65** | Point photo (`depth_base=null`) and interval photo (`base>top`) | Extract trace | Point refused by name; interval accepted | `coreimage.rs:1247-1283` |
| **SB-RPH-T66** | Dark, near-neutral-free ultraviolet fixture | Recommend recipe | Identity recipe; note states ultraviolet reasoning | Existing test contract `coreimage.rs:3457-3479` |
| **SB-RPH-T67** | Paired white-light and ultraviolet fixtures over same depth | Extract | White outputs `CPHOTO_DARK/RED/TEX`; UV outputs `CPHOTO_FLUOR*`; no mnemonic collision | `coreimage.rs:4498-4569` |
| **SB-RPH-T68** | White-light darkness trace with two separable populations | Build lith proxy | `CPHOTO_LITH` values only `0/1`; Otsu cut and darker/live counts recorded | `coreimage.rs:1504-1541` |
| **SB-RPH-T69** | Ultraviolet trace | Request lith proxy | Refused with fluorescence-not-lithology reason | `coreimage.rs:1510-1517,4809-4821` |
| **SB-RPH-T70** | Known dipping contact, with/without declared unfold | Extract | Unfolded transition narrower than original and run records unfold | Existing test `coreimage.rs:4932-5048` |
| **SB-RPH-T71** | Box photo with known top/base and lanes | Build strip then re-read | Strip top/base exactly preserved and its trace matches box trace within existing sample tolerance | Strip contract `coreimage.rs:2516-2676`; existing strip tests around `:4046-4171` |
| **SB-RPH-T72** | Strong lane profile and weak/uneven profile | Detect | Proposal returned with reasons; weak case reports caution and applies nothing | `coreimage.rs:1974-2015` |
| **SB-RPH-T73** | Peaked and flat unfold scans | Recommend | Peaked returns proposal; flat returns none with reason | Existing test `coreimage.rs:4870-4927` |
| **SB-RPH-T74** | Two wells, two zones, cancellation after first item | Run future RPH module | New versioned sets only; effective zone parameters retained; second item canceled without partial physical curves | Shared runner contract; `modules.rs:434-574` is current absence source |
| **SB-RPH-T75** | Unknown model/option and one out-of-domain parameter | Validate | Hard error names value and allowed domain before any output | Dossier §5.6 rules 9/14 |
| **SB-RPH-T76** | Re-run one result from recorded provenance only | Reproduce | All physical curves match within `1e-6` relative and flags exactly | Dossier §5.6 reproducibility rules |
| **SB-RPH-T77** | Parameter supplied to a branch that does not consume it | Validate | Parameter refused as unused or omitted from surface; never accepted silently | Dossier K-6.8 |

## 7. Open items, escalations and refusals

### 7.1 Open items

- **O-1 — Core-photo field calibration.** Validate `CPHOTO_DARK`, `CPHOTO_TEX`, fluorescence and
  `CPHOTO_LITH` against counted/described core across illumination and camera changes. Settle with a
  blinded core-photo/count comparison; no new default is implied.
- **O-2 — Gassmann tolerance.** `EPS_GASSMANN` remains absent because the dossier names the guard but
  supplies no tolerance. Settle from numerical-conditioning analysis across the supported modulus
  range, then cite that derivation.
- **O-3 — Endpoint library.** Obtain independently sourced elastic endpoints and uncertainty for the
  mineral set actually supported. Vendor tables remain evidence-only and untranscribed.
- **O-4 — Image tool geometry.** Define supported pad/button/azimuth schemas after DIO's array-log
  contract is finalized; method physics must not be coupled to vendor reader names.
- **O-5 — AVO fixtures.** Acquire primary numeric fixtures for exact Zoeppritz and every retained
  approximation before enabling Bortfeld or either Hilterman branch.

### 7.2 Escalations

- **E-1:** live-product test for the printed `304.8²` and sign contradictions; confirmation only,
  not a source for SandiBumi's method.
- **E-2:** primary Muskat/Carslaw–Jaeger sources or a live numeric test for effective probe radius;
  outside this chapter's scheduled scope unless formation-tester mobility is added.
- **E-3:** Backus (1962) paper to confirm the T1-readable complete tensor.
- **E-4/E-15:** Luthi and Souhaite (1990) to settle `b`, `Rm` versus `Rmf`, and excess-current
  integral; core/reservoir calibration still required for `c`.
- **E-6:** primary papers for KT/SCA/DEM, Ciz–Shapiro, Brown–Korringa and Mavko–Bandyopadhyay algebra.
- **E-7:** Batzle and Wang (1992) plus the missing fluid-property documentation.
- **E-9:** source for the printed `C13=C12` assumption; until held it must remain a named assumption.
- **E-11:** delivered-study precedent for parameter-selection practice, queried without copying any
  asset identifier into this chapter.
- **E-12:** Greenberg and Castagna (1992), *Geophysical Prospecting* 40, 195–209, before adopting
  coefficients now held only in vendor rasters.
- **E-13:** live discrimination of the adhesion fraction compiled into one contact-model branch;
  confirmation only, never reconstruction.
- **E-14:** published mathematics behind the Bayesian inversion API before specifying its objective.
- **E-16:** cross-dipole data or primary anisotropy literature for low-relative-dip SH/SV assignment.

### 7.3 Refusals

- **RF-1 — Broken unit algebra. SandiBumi does instead:** typed SI/GPa state and tested boundary
  conversions; it does not reproduce the missing `304.8²` or inverted Mpsi print (§2.1).
- **RF-2 — Unguarded inversion. SandiBumi does instead:** pre-division domain guards and null plus a
  semantic flag (§4.1).
- **RF-3 — Missing Hertz–Mindlin cube root. SandiBumi does instead:** the dimensionally correct
  cube-root expression corroborated by readable source (§4.2).
- **RF-4 — Conflated `f` and `s_fact`. SandiBumi does instead:** two named parameters in their actual
  equation positions (§4.2).
- **RF-5 — Silent endpoint fallback. SandiBumi does instead:** versioned independently sourced
  endpoints and refusal on incomplete shear state (§4.2).
- **RF-6 — Cosine-only dip direction and vector mean for planes. SandiBumi does instead:** `atan2`
  and axial/eigenvector means (§4.5).
- **RF-7 — Pointwise EI treated as a constant or convertible quantity. SandiBumi does instead:**
  pointwise computation and a non-convertible unit stamp (§4.4).
- **RF-8 — Mean of means called mean. SandiBumi does instead:** separately named pooled and
  area-weight results (§4.5).
- **RF-9 — User checkbox overrides declination provenance. SandiBumi does instead:** derives the
  interlock from `DECL_APPLIED` (§4.5).
- **RF-10 — Accepted-but-unused inputs. SandiBumi does instead:** hides or refuses them and labels
  QC-only inputs (§4.7).
- **RF-11 — Image proxy named as petrophysical truth. SandiBumi does instead:** keeps `CPHOTO_*`
  namespaces and refuses ultraviolet lithology classification (§4.6).

### 7.4 Independent-derivation requirements

- **C-3 — Entropy image speed-correction user need. Owning requirement: SB-RPH-032.** The vendor
  artifact is not read, inferred or reconstructed. The independently derived capability is a
  reversible physical motion model using accelerometer kinematics, declared channel offsets and a
  separate image-residual alignment pass. **Primary sources:** specific peer-reviewed borehole-image
  motion-correction literature is not held; acquiring it is an escalation before the algorithm is
  specified. Vendor help/training evidence may define the user need and test seams, not the method.
  **Betters:** unlike the documented incumbent split, the SandiBumi result must emit one reversible
  displacement field with per-channel offsets, residuals and provenance rather than a correction
  whose geometry cannot be reconstructed (dossier §§2.9,4.1). Until the primary source is held,
  SB-RPH-032 remains `ABSENT`.
- **C-3 — Frequency-domain dispersion fitting. Owning requirement: SB-RPH-030.** No vendor fit,
  binary or observed-input/output reconstruction may be consumed. **Primary sources:** the specific
  peer-reviewed dispersion-model and inversion papers are not held; acquisition is required before
  selecting a model, defaults or acceptance fixture. **Betters:** the native route must expose the
  dispersion model, frequency band, uncertainty, residual and failure domain, improving on the
  incumbent capability-level description that supplies none of those auditable outputs (dossier
  IP-14 and Tier-C register). The sourced scalar `0.98` correction is a different, non-Tier-C method.
- Experienced Eye/EEFS, Domain Transfer Analysis, Textural Facies, `Freq_Tiles` and neural-weight
  needs are in MLA's scope and are not duplicated here.

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirements | Dossier evidence |
|---|---|
| SB-RPH-001, SB-RPH-002, SB-RPH-003, SB-RPH-004, SB-RPH-005, SB-RPH-006 | §§2.1,2.10,4.1,5.1–5.6; IP-1…4; GL-1…5; TL-1…4 |
| SB-RPH-007, SB-RPH-008, SB-RPH-009, SB-RPH-010, SB-RPH-011, SB-RPH-012, SB-RPH-013, SB-RPH-014, SB-RPH-015, SB-RPH-016, SB-RPH-017, SB-RPH-018, SB-RPH-019 | §§2.3–2.5,4.1,5.2–5.5; IP-5/6; GL-2/4/9–12; TL-3/5/6/9/13/15/17/18 |
| SB-RPH-020, SB-RPH-021, SB-RPH-022, SB-RPH-023, SB-RPH-024, SB-RPH-025 | §§2.6–2.7,3.8,4.1,5.2–5.5; IP-7/8/11; GL-7/8/10/13/14; TL-7/11/12 |
| SB-RPH-026, SB-RPH-027, SB-RPH-028, SB-RPH-029, SB-RPH-030 | §§2.8,3.9–3.10,4.1,5.3–5.5; IP-9/10/10b/14; GL-15; TL-8 |
| SB-RPH-031, SB-RPH-032, SB-RPH-033, SB-RPH-034, SB-RPH-035, SB-RPH-036, SB-RPH-037, SB-RPH-038, SB-RPH-039, SB-RPH-040, SB-RPH-041 | §§2.9,3.7,4.1,5.2–5.5; IP-12…15; GL-16…20; TL-14 |
| SB-RPH-042, SB-RPH-043, SB-RPH-044, SB-RPH-045, SB-RPH-046, SB-RPH-047, SB-RPH-048 | §§2.9,4.1 and E-10, updated by source audit of `coreimage.rs`/`images.rs` |
| SB-RPH-049, SB-RPH-050, SB-RPH-051, SB-RPH-052 | §§3.11,5.4–5.6 and failure discipline §3.12 |

### 8.2 Inventory, equation and optimal-choice disposition

| Dossier item | Disposition | Where it went |
|---|---|---|
| IP-1 — Average Gassmann | `ADOPTED` in guarded canonical form | SB-RPH-003…005 |
| IP-2 — Crossplot Gassmann | `ADOPTED` as forward-form corroboration | SB-RPH-003; T06/T07 |
| IP-3 — Log Fluid Substitution | `ADOPTED` at capability level; quadratic inversion is secondary and guarded | SB-RPH-003…005 |
| IP-4 — observed-to-wet conditioning | `DEFERRED` P2 until the base substitution state exists | SB-RPH-003; dossier convergence row retained in evidence |
| IP-5 — dry-frame selector | `ADOPTED` by named models; external provenance tightened | SB-RPH-012…016 |
| IP-6 — fluid mixing and blend factors | `ADOPTED` named laws; continuous blends `REJECTED` | SB-RPH-007…009; RF-4 |
| IP-7 — Greenberg–Castagna | `ESCALATED` for primary coefficients; unit contract adopted | SB-RPH-020; E-12 |
| IP-8 — Skelt volumetric partition | `DEFERRED` P3, trigger: elastic laminated workflow | Seam TBD/RPH; O-3 |
| IP-9 — Elastic Impedance | `ADOPTED` with pointwise/unit corrections | SB-RPH-027 |
| IP-10 — Ricker, convolution, Aki–Richards/AVO | `ADOPTED` in part | SB-RPH-028/029 |
| IP-10b — 14 elastic attributes | `ADOPTED` | SB-RPH-002/026 |
| IP-11 — partial Backus | `REJECTED` as incomplete; complete T1 route adopted | SB-RPH-022; RF-5 |
| IP-12 — mean dip/TST/TVT | `ADOPTED` with `atan2`/axial correction | SB-RPH-034 |
| IP-13 — image transforms/N:G/PORMAP | `ADOPTED` in part; naming and threshold defaults rejected | SB-RPH-033/036/040; RF-8 |
| IP-14 — semblance/nth-root/simple and fitted dispersion | `DEFERRED` P3; Tier-C fit acquisition-gated | SB-RPH-030; §7.4 |
| IP-15 — formation-tester mobility | `DEFERRED` outside scheduled RPH scope | E-2 |
| GL-1 — shaly Gassmann | `ADOPTED` at method level | SB-RPH-003…005 |
| GL-2 — generic fluid mixing | `ADOPTED` | SB-RPH-007/008 |
| GL-3 — fluid-property wrappers | `ESCALATED` to primary equations | SB-RPH-006; E-7 |
| GL-4 — Krief/critical porosity | `ADOPTED` | SB-RPH-012/013 |
| GL-5 — clean-sand substitution family | `ADOPTED` as corroboration | SB-RPH-003…005 |
| GL-6 — finite-shear pore-filler substitution | `ESCALATED` to primary equations | SB-RPH-018; E-6 |
| GL-7 — anisotropic fluid substitution | `ESCALATED` to primary equations | SB-RPH-025; E-6 |
| GL-8 — measured anisotropy | `ADOPTED` with SH/SV default removed | SB-RPH-023…025 |
| GL-9 — rock bounds | `ADOPTED` | SB-RPH-009 |
| GL-10 — KT/SCA/DEM | `ADOPTED` capability and gates; algebra source-gated | SB-RPH-017; E-6 |
| GL-11 — contact model | `EVIDENCE-ONLY`; conflicting defaults drive absence | SB-RPH-011…016; §5 |
| GL-12 — mineral library | `EVIDENCE-ONLY`; table not transcribed | SB-RPH-010; O-3 |
| GL-13 — seven shear models | `DEFERRED` pending primary parameters | SB-RPH-020/021; O-3 |
| GL-14 — Xu–White/Xu–Payne | `DEFERRED` pending primary equations/parameters | SB-RPH-017; E-6 |
| GL-15 — six AVO methods | `ADOPTED` exact + two approximations; others validation-gated | SB-RPH-028; O-5 |
| GL-16 — image-module family | `ADOPTED` as capability/input inventory | SB-RPH-031…035 |
| GL-17 — image porosity | `ADOPTED` without vendor Archie defaults | SB-RPH-036 |
| GL-18 — fracture aperture | `ADOPTED` functional form; constants absent | SB-RPH-037; E-4/E-15 |
| GL-19 — fracture intensity/density | `ADOPTED` with all policies and metadata | SB-RPH-038…041 |
| GL-20 — dip count | `ADOPTED` as corroborating Terzaghi route | SB-RPH-038/041 |
| GL-21 — photo-to-array | `ADOPTED` and surpassed by as-built core workflow | SB-RPH-042…048 |
| TL-1 — basic elastic/mixing equations | `ADOPTED` as T1 source | SB-RPH-002/007/009 |
| TL-2 — Batzle–Wang implementation | `EVIDENCE-ONLY` pending primary derivation | SB-RPH-006; E-7 |
| TL-3 — Brie | `ADOPTED` | SB-RPH-008 |
| TL-4 — Gassmann forward | `ADOPTED` | SB-RPH-003 |
| TL-5 — bounds | `ADOPTED` | SB-RPH-009 |
| TL-6 — contact/dry-frame models | `ADOPTED` in canonical, source-governed form | SB-RPH-012…017 |
| TL-7 — full Backus | `ADOPTED` | SB-RPH-022 |
| TL-8 — substitution-line/AVO helpers | `ADOPTED` as corroboration | SB-RPH-003/028 |
| TL-9 — vendor endpoint dataset | `REJECTED` for transcription | SB-RPH-010; CONTRACT §2.1 |
| TL-10 — compiled isotropic substitution | `EVIDENCE-ONLY`; no binary reconstruction | SB-RPH-018; E-6 |
| TL-11 — compiled anisotropic substitution | `EVIDENCE-ONLY`; no binary reconstruction | SB-RPH-025; E-6 |
| TL-12 — compiled shear prediction | `EVIDENCE-ONLY`; no binary reconstruction | SB-RPH-021; E-14 |
| TL-13 — model/estimation APIs | `EVIDENCE-ONLY`; no binary reconstruction | SB-RPH-019; E-14 |
| TL-14 — image training workflow | `ADOPTED` as user-need/workflow evidence | SB-RPH-032…041 |
| TL-15 — Bayesian inversion API | `DEFERRED` P3 pending solver literature | SB-RPH-019; E-14 |
| TL-16 — anisotropic utility layer | `EVIDENCE-ONLY`; no binary reconstruction | SB-RPH-019/025 |
| TL-17 — forward multi-mineral defaults | `EVIDENCE-ONLY`; conflicts drive absence | §5; SB-RPH-011…017 |
| TL-18 — remaining function inventory | `EVIDENCE-ONLY`; prevents false absence claims | §2 and SB-RPH-002…030 |
| §5.1 canonical units | `ADOPTED` | SB-RPH-001; parameters U_* |
| §5.2 canonical equations | `ADOPTED` except source-gated advanced models | SB-RPH-002…025 |
| §5.3 parameter table | `ADOPTED` under CONTRACT discipline; vendor tables omitted | §5; 44 ABSENT, 1 NON-ADOPTABLE |
| §5.4 flags/nulls | `ADOPTED` | SB-RPH-005,025,041,051 |
| §5.5 tests T-1…T-22 | `ADOPTED` and expanded | SB-RPH-T01…T77 |
| §5.6 rules 1…15 | `ADOPTED` where in domain | Requirements and refusals throughout |

The dossier's 82 parameter rows receive the following one-for-one disposition. A vendor number can
be useful evidence while still being absent or non-adoptable under `CONTRACT.md` §2.

| Dossier §5.3 parameter row | Disposition | Where it went |
|---|---|---|
| 1 — G–C sandstone coefficients | `NON-ADOPTABLE` until primary paper | §5 `GC_A_B_C`; E-12 |
| 2 — G–C limestone coefficients | `NON-ADOPTABLE` until primary paper | §5 `GC_A_B_C`; E-12 |
| 3 — G–C dolomite coefficients | `NON-ADOPTABLE` until primary paper | §5 `GC_A_B_C`; E-12 |
| 4 — G–C shale coefficients | `NON-ADOPTABLE` until primary paper | §5 `GC_A_B_C`; E-12 |
| 5 — G–C quartz mineral row | `REJECTED` as unnecessary mineral-addressed variant | SB-RPH-020; IP-OPEN-1 |
| 6 — Mudrock line | `DEFERRED` pending primary source; ships absent | §5 `MUDROCK_A_B`; SB-RPH-021 |
| 7 — Han high-clay branch | `DEFERRED` pending primary source; ships absent | §5 `HAN_CFG`; SB-RPH-021 |
| 8 — Han low-clay branch | `DEFERRED` pending primary source; ships absent | §5 `HAN_CFG`; SB-RPH-021 |
| 9 — Han high-porosity branch | `DEFERRED` pending primary source; ships absent | §5 `HAN_CFG`; SB-RPH-021 |
| 10 — Han low-porosity branch | `DEFERRED` pending primary source; ships absent | §5 `HAN_CFG`; SB-RPH-021 |
| 11 — GC brine-velocity seed range | `EVIDENCE-ONLY`; no single default exists | SB-RPH-021 |
| 12 — mineral endpoint set A | `EVIDENCE-ONLY`; vendor table not transcribed | SB-RPH-010; O-3 |
| 13 — mineral endpoint set B | `EVIDENCE-ONLY`; vendor table not transcribed | SB-RPH-010; O-3 |
| 14 — clay endpoint rows | `EVIDENCE-ONLY`; vendor table not transcribed | SB-RPH-010; O-3 |
| 15 — minerals with no shear endpoint | `ADOPTED` as refusal behavior, not values | SB-RPH-010; T18 |
| 16 — empty generic-shale slot | `ADOPTED` as missing-input behavior | SB-RPH-010; T18 |
| 17 — vendor quartz bulk modulus | `EVIDENCE-ONLY`; not adopted | §5 `ENDPOINT_SET`; O-3 |
| 18 — alternate quartz bulk moduli | `EVIDENCE-ONLY`; conflict drives absence | §5 `ENDPOINT_SET`; O-3 |
| 19 — vendor dolomite shear modulus | `EVIDENCE-ONLY`; not adopted | §5 `ENDPOINT_SET`; O-3 |
| 20 — laminated mineral bulk-modulus set | `EVIDENCE-ONLY`; vendor table not transcribed | §5 `ENDPOINT_SET`; O-3 |
| 21 — wet-clay endpoint | `EVIDENCE-ONLY`; not adopted | §5 `ENDPOINT_SET`; O-3 |
| 22 — absent smectite elastic endpoint in one tool | `EVIDENCE-ONLY`; absence claim correctly narrowed | SB-RPH-010 |
| 23 — critical porosity variants | `ADOPTED` as an absent default | §5 `PHI_CRIT`; SB-RPH-011/012 |
| 24 — depositional porosity variants | `ADOPTED` as an absent default | §5 `PHI_DEPOSITIONAL`; SB-RPH-011/014 |
| 25 — HM adhesion variants | `ADOPTED` as an absent default | §5 `HM_ADHESION`; SB-RPH-014 |
| 26 — empirical shear scale | `ADOPTED` at identity only | §5 `SHEAR_SCALE`; SB-RPH-015 |
| 27 — coordination number, tool A | `EVIDENCE-ONLY`; conflict drives absence | §5 `COORDINATION` |
| 28 — coordination number, tool B | `EVIDENCE-ONLY`; conflict drives absence | §5 `COORDINATION` |
| 29 — coordination number, tool C | `EVIDENCE-ONLY`; conflict drives absence | §5 `COORDINATION` |
| 30 — forced intermediate coordination | `EVIDENCE-ONLY`; branch-specific value not generalized | §5 `COORDINATION`; SB-RPH-016 |
| 31 — Krief exponent | `ADOPTED` | §5 `M_KRIEF`; SB-RPH-013 |
| 32 — HM pressure | `ADOPTED` as absent because field-specific | §5 `P_EFFECTIVE`; SB-RPH-014 |
| 33 — KT aspect ratio | `DEFERRED`; source-gated and absent | §5 `KT_ASPECT`; SB-RPH-017 |
| 34 — multi-pore aspect ratios | `EVIDENCE-ONLY`; conflicts drive absent pore configuration | §5 `PORE_FRACTIONS`; SB-RPH-017 |
| 35 — sand/shale mixing weight | `EVIDENCE-ONLY`; no generalized default | SB-RPH-017 |
| 36 — Brie exponent | `ADOPTED` | §5 `E_BRIE`; SB-RPH-008 |
| 37 — Wood/Voigt blend factors | `REJECTED` in favor of named laws | SB-RPH-007/009 |
| 38 — porosity/shale QC triplet | `DEFERRED`; interval QC belongs to selected workflow | SB-RPH-050 |
| 39 — shaly-substitution cutoff | `EVIDENCE-ONLY`; no universal default adopted | SB-RPH-050 |
| 40 — irreducible-water default | `EVIDENCE-ONLY`; no universal default adopted | SB-RPH-050 |
| 41 — clean-sand substitution cutoffs | `EVIDENCE-ONLY`; no universal default adopted | SB-RPH-050 |
| 42 — water-based invasion ratio | `DEFERRED` until invasion workflow is scheduled | SB-RPH-004 |
| 43 — oil-based maximum water saturation | `DEFERRED` until invasion workflow is scheduled | SB-RPH-004 |
| 44 — laminated-shale density | `REJECTED` vendor default; ships absent if route is built | O-3; I-xii |
| 45 — laminated-shale compressional slowness | `DEFERRED`; field-specific and absent | O-3 |
| 46 — observed-to-wet convergence | `DEFERRED` with IP-4 until base substitution exists | SB-RPH-003 |
| 47 — Terzaghi policy | `ADOPTED` as required selection | §5 `TERZ_POLICY`; SB-RPH-038 |
| 48 — Terzaghi blind zone | `ADOPTED` | §5 `TERZ_BLIND`; SB-RPH-038 |
| 49 — Terzaghi cap angle | `ADOPTED` | §5 `TERZ_CAP_ANGLE`; SB-RPH-038 |
| 50 — Terzaghi cap weight | `ADOPTED` as absent | §5 `TERZ_CAP_WEIGHT`; SB-RPH-038 |
| 51 — Terzaghi correction state | `ADOPTED` opt-in | §5 `TERZ_ON`; SB-RPH-038 |
| 52 — Terzaghi weight floor | `ADOPTED` | §5 `TERZ_FLOOR`; SB-RPH-038 |
| 53 — fracture window/step | `ADOPTED` | §5 `FRAC_WINDOW/FRAC_STEP`; SB-RPH-041 |
| 54 — Luthi–Souhaite `b` | `ADOPTED` as absent | §5 `LS_B`; SB-RPH-037; E-4 |
| 55 — Luthi–Souhaite `c` | `ADOPTED` as absent | §5 `LS_C`; SB-RPH-037; E-4 |
| 56 — aperture resistivity convention | `ADOPTED` as required selection | §5 `LS_R_TERM`; SB-RPH-037; E-15 |
| 57 — aperture integration radius | `ADOPTED` as labeled vendor start | §5 `LS_RADIUS`; SB-RPH-037 |
| 58 — aperture standard-deviation factor | `DEFERRED` as absent because meaning is undocumented | §5 `LS_STDDEV`; SB-RPH-037 |
| 59 — pad/category defaults | `EVIDENCE-ONLY`; tool-specific labels are not generalized | SB-RPH-037/039 |
| 60 — image-porosity Archie parameters | `ADOPTED` as absent | §5 `A_IMG/M_IMG/N_IMG`; SB-RPH-036 |
| 61 — image-porosity regression | `ADOPTED` as absent per-interval fit | §5 `POR_FIT`; SB-RPH-036 |
| 62 — Newberry form | `ADOPTED` as coefficient-free route | SB-RPH-036 |
| 63 — binarization window | `ADOPTED` | §5 `BIN_WINDOW`; SB-RPH-036 |
| 64 — histogram window/step/bins | `ADOPTED` | §5 `HIST_WINDOW/HIST_STEP/HIST_BINS`; SB-RPH-040 |
| 65 — Xu–White aspect ratios | `DEFERRED` pending primary model source | §5 `KT_ASPECT`; SB-RPH-017; E-6 |
| 66 — Xu–Payne aspect ratios | `DEFERRED` pending primary model source | §5 `PORE_FRACTIONS`; SB-RPH-017; E-6 |
| 67 — example saturation quadratic | `REJECTED` as field-specific | SB-RPH-050 |
| 68 — nth-root exponent | `ADOPTED` as exposed parameter | §5 `N_STACK`; SB-RPH-030 |
| 69 — simple dispersion factor | `ADOPTED` only for named scalar method | §5 `DISP_SIMPLE`; SB-RPH-030 |
| 70 — Ricker limits | `ADOPTED` | §5 `F_RICKER_MIN/MAX`; SB-RPH-029 |
| 71 — Vp/Vs validity envelopes | `DEFERRED` until primary validation; not silently adopted | SB-RPH-050 |
| 72 — replacement-velocity envelope | `DEFERRED` to time/depth workflow | SB-RPH-029 |
| 73 — sonic auto-null threshold | `DEFERRED` to ENV conditioning; not duplicated | Seam ENV/RPH |
| 74 — maximum incidence angle | `ADOPTED` | §5 `ANGLE_MAX`; SB-RPH-028 |
| 75 — SH/SV assignment | `ADOPTED` as absent | §5 `SH_ASSIGNMENT`; SB-RPH-023 |
| 76 — SH/SV caution band | `ADOPTED` as a range only | §5 `SHSV_CAUTION`; SB-RPH-023 |
| 77 — fast-azimuth frame | `ADOPTED` as required selection | §5 `AZIM_FAST_FRAME`; SB-RPH-023 |
| 78 — anisotropy-model selector | `ADOPTED` as required selection | §5 `ANISO_MODEL`; SB-RPH-024 |
| 79 — drilling/borehole fluid inputs | `ADOPTED` as absent | §5 `DFD/DTF`; SB-RPH-024 |
| 80 — weak-anisotropy validity | `ADOPTED` | §5 `DELTA_MAX`; SB-RPH-025 |
| 81 — KT validity | `ADOPTED` | SB-RPH-017; T26 |
| 82 — finite-shear substitution validity | `ADOPTED` as future gate; equations escalated | SB-RPH-018; E-6 |

### 8.3 Discrepancy-ledger disposition

| Dossier item | Disposition | Where it went |
|---|---|---|
| I-i — sign contradiction | Canonical T1 form `ADOPTED`; live behavior `ESCALATED` | SB-RPH-003; E-1 |
| I-ii — missing `304.8²` | `REJECTED` printed defect; live behavior `ESCALATED` | SB-RPH-001; RF-1; E-1 |
| I-iii — Voigt `+` versus product | `REJECTED` defect | SB-RPH-007; T13 |
| I-iv — missing HM cube root | `REJECTED` defect | SB-RPH-014; RF-3; T22 |
| I-v — probe radius | `DEFERRED` and `ESCALATED` | E-2 |
| I-vi — parameter-file name mismatch | `EVIDENCE-ONLY`; no vendor file copied | SB-RPH-051; CONTRACT §2.1 |
| I-vii — citation/year defects | `EVIDENCE-ONLY`; publication metadata rule adopted | SB-RPH-051 |
| I-viii — attribution spelling defects | `EVIDENCE-ONLY`; publication metadata rule adopted | SB-RPH-051 |
| I-ix — duplicated diagram | `EVIDENCE-ONLY`; no method obligation | §8 completeness record |
| I-x — contradictory velocity units | `REJECTED` ambiguity; canonical units adopted | SB-RPH-001; RF-1 |
| I-xi — inverted Mpsi conversion | `REJECTED` defect | §5; T03 |
| I-xii — laminated-shale density default | `REJECTED`; ships absent if route added | Parameter discipline; O-3 |
| I-xiii — non-reproducing FT example | `DEFERRED` and `ESCALATED` | E-2 |
| I-xiv — GC semantic labels | `REJECTED`; lithology addressing adopted | SB-RPH-020/021 |
| K-6.1 — pointwise EI factor | `ADOPTED` corrected | SB-RPH-027 |
| K-6.2 — missing Backus velocity definitions | `ADOPTED` from complete T1 route | SB-RPH-022; E-3 |
| K-6.3 — Backus density ambiguity | `ADOPTED` thickness-weighted density | SB-RPH-022; T32 |
| K-6.4 — quadrant-degenerate dip | `REJECTED` defect | SB-RPH-034; RF-6 |
| K-6.5 — extra parenthesis | `EVIDENCE-ONLY` typographic | SB-RPH-002 source choice |
| K-6.6 — image sand-side contradiction | `ADOPTED` explicit convention | §5 `SAND_IS` |
| K-6.7 — mean of means | `REJECTED` as pooled statistic | SB-RPH-040; T59 |
| K-6.8 — unused inputs | `REJECTED` | SB-RPH-052; RF-10 |
| K-6.9 — citation/naming defects | `EVIDENCE-ONLY`; publication metadata rule adopted | SB-RPH-051 |
| K-6.10 — PORMAP naming | `REJECTED`; histogram named honestly | SB-RPH-036/040 |
| K-9.1 — apparent/true dip equations absent | `ADOPTED` from T4, primary validation still required | SB-RPH-034 |
| K-9.3 — Terzaghi equation/policies | `ADOPTED` all three | SB-RPH-038…041 |
| K-9.4 — Luthi–Souhaite | `ADOPTED` form, constants/convention `ESCALATED` | SB-RPH-037; E-4/E-15 |
| R-11 — GC coefficient evidence | `ESCALATED`; raster digits not adopted | SB-RPH-020; E-12 |
| IP-OPEN-1 — off-screen quartz digits | `REJECTED` as unnecessary mineral-addressed row | SB-RPH-020 |
| IP-OPEN-2 — obscured wet-clay digit | `EVIDENCE-ONLY`; shale-lithology row supersedes it | SB-RPH-020 |

### 8.4 Gap and escalation disposition

| Dossier gap | Disposition | Where it went |
|---|---|---|
| E-1 — IP numeric behavior | `ESCALATED` as confirmation only | §7.2; SB-RPH-001/003 |
| E-2 — probe radius/FT example | `DEFERRED` outside scheduled scope; `ESCALATED` | §7.2 |
| E-3 — Backus primary paper | `ESCALATED` confirmatory | §7.2; SB-RPH-022 |
| E-4 — Luthi–Souhaite paper/constants | `ESCALATED` | §7.2; SB-RPH-037 |
| E-5 — mineral table | `CLOSED` in dossier, but values `EVIDENCE-ONLY` under no-transcription rule | SB-RPH-010; O-3 |
| E-6 — compiled advanced-model algebra | `ESCALATED` to named primary papers | §7.2; SB-RPH-017/018/025 |
| E-7 — fluid-property route | `ESCALATED` to primary paper/documentation | §7.2; SB-RPH-006 |
| E-8 — off-screen GC mineral rows | `REJECTED` as unnecessary to lithology-addressed design | SB-RPH-020 |
| E-9 — `C13` assumption | `ESCALATED` | §7.2; SB-RPH-024/025 |
| E-10 — core-photo conditioning white space | `CLOSED/PRESENT-OK` by source audit after dossier date | SB-RPH-042…048; §3.3–3.5 |
| E-11 — delivered-study precedent | `ESCALATED` without carrying identifiers into this chapter | §7.2 |
| E-12 — GC primary paper | `ESCALATED` | §7.2; SB-RPH-020 |
| E-13 — compiled adhesion behavior | `ESCALATED` as confirmation only | §7.2; SB-RPH-014 |
| E-14 — Bayesian inversion mathematics | `ESCALATED` | §7.2; SB-RPH-019 |
| E-15 — aperture resistivity convention | `ESCALATED` | §7.2; SB-RPH-037 |
| E-16 — low-dip SH/SV rule | `ESCALATED` | §7.2; SB-RPH-023 |

### 8.5 Critique disposition

| Critique findings | Disposition | Where it went |
|---|---|---|
| B-1 — GC corroboration overstated | Applied: single-vendor digits non-adoptable, primary source escalated | §5; SB-RPH-020; E-12 |
| B-2 — adhesion silently fixed/conflated | Applied: `f` absent and separate from identity `s_fact` | SB-RPH-014/015; T23/T24 |
| B-3 — aperture defaults misreported | Applied: vendor starts acknowledged but not adopted quantitatively | SB-RPH-037; §5; E-4/E-15 |
| M-1 — Brie scale direction | Applied: monotone direction corrected | §2.2; SB-RPH-008; T15 |
| M-2 — `e=1` scope | Applied: two-phase identity only | §2.2; SB-RPH-008; T13 |
| M-3 — final-state two-phase module | Applied: not conflated with generic three-phase mixer | §2.2; SB-RPH-007/008 |
| M-4 — wrong mixing-file citation | Applied: claims separated by module/evidence tier | §2.2; SB-RPH-007/008 |
| M-5 — `Voigtfac` is a solid dial | Applied; continuous blends rejected | SB-RPH-007/009 |
| M-6 — contradictory IP velocity units | Applied: canonical SI boundary | SB-RPH-001; RF-1 |
| M-7 — ledger ID collision | Applied: real I-xiii and new I-xiv separate | §8.3 |
| M-8 — six AVO methods | Applied | SB-RPH-028 |
| M-9 — two same-vendor endpoint sets | Applied without transcribing either table | SB-RPH-010; O-3 |
| M-10 — clay/missing endpoint facts | Applied: missing is not zero; table not transcribed | SB-RPH-010; T18 |
| M-11 — three Terzaghi policies | Applied | SB-RPH-038; T56/T57 |
| M-12 — image-porosity attribution/method split | Applied | SB-RPH-036 |
| M-13 — `Rm` versus `Rmf` | Applied as required convention | SB-RPH-037; T55 |
| M-14 — conventional versus entropy speed correction | Applied | SB-RPH-032; §7.4 |
| M-15 — drift source attribution | Applied in source strings; no false addendum citation | §8 throughout |
| M-16 — inversion/default inventory | Applied; API retained, defaults not silently adopted | SB-RPH-019; §5 |
| M-17 — two porosities | Applied | SB-RPH-011; T19 |
| M-18 — SH/SV rule | Applied with no default | SB-RPH-023; T34/T35 |
| M-19 — full elastic suite | Applied | SB-RPH-002/026 |
| M-20 — declination interlock | Applied | SB-RPH-035; T51/T52 |
| n-1 — source-file line count | Applied in evidence interpretation; no product obligation | §8 completeness record |
| n-2 — velocity-conversion factor | Applied through typed boundary rather than copied inline constant | SB-RPH-001; T01/T02 |
| n-3 — unit-rule citation | Applied | §2.1; SB-RPH-001 |
| n-4 — HM error magnitude | Applied with corrected ~20-order consequence | §2.3; SB-RPH-014; T22 |
| n-5 — image-porosity shipped constants | Applied as cited but non-adopted defaults | §5 `A_IMG/M_IMG/N_IMG` |
| n-6 — Brie exponent validation differences | Applied through explicit selected law/domain | SB-RPH-008/050 |
| n-7 — seven shear models and Han branches | Applied | SB-RPH-021; §5 `HAN_CFG` |
| n-8 — fracture-density defaults/midpoint | Applied | §5; SB-RPH-038…041; T58 |
| n-9 — conflicting Zoeppritz dates | Applied without normalization; primary fixture remains open | SB-RPH-028; O-5 |
| n-10 — previously undispositioned ledger items | Applied individually | §8.3; SB-RPH-040/052 |
| n-11 — full dry-frame selector and zoning | Applied | SB-RPH-016/049 |
| n-12 — special-case substitution recipes | `DEFERRED` until base substitution exists; no silent saturation rewrite | SB-RPH-003…005 |
| n-13 — two worked-example answer keys | `ADOPTED` as future regression evidence | Dossier §5.5 retained; SB-RPH-003 |

### 8.6 Completeness statement

All 55 inventory identifiers (IP 16 including IP-10b, GL 21, TL 18), every optimal-choice group,
all discrepancy items named in §4.2, all 16 escalations, every §5 adoption-spec subsection and all
36 critique findings are dispositioned above. Core-photo claims were re-verified against current
source rather than inherited from the dossier's earlier white-space assessment. No vendor chart or
lookup-table content is transcribed. No client, operator, field, block, basin, well or project name
appears in this chapter.
