# 18. Geomechanics, pore pressure and fracture gradient — requirements

This chapter compiles the binding product requirements for vertical stress, pore pressure,
fracture pressure, horizontal stress, elastic properties and wellbore stability. Its evidence base
is the paired local dossier `geomech-ppfg.md` and `geomech-ppfg_critique.md`, read in full. Evidence
tiers follow `CONTRACT.md` §1.2; parameter handling follows §§2–2.1 exactly.

## 1. Scope and boundary

This domain owns the calculation chain from a typed depth/density input through vertical stress,
normal pressure, pore pressure, fracture pressure, elastic/static-property calibration, horizontal
stress and wellbore-stability outputs. It owns datum alignment, correlation-validity gates, stress
frames and pressure/gradient naming. The fluid-substitution chapter owns elastic-wave modelling;
the unconventional chapter may emit dynamic elastic properties, but this chapter owns their static
calibration and use in stress. Generic storage and plotting remain cross-cutting capabilities.

The chapter does not copy vendor charts, vendor lookup tables, binary constants or local delivered-
study presets. Published method names are inventory; adoption requires readable equations, sourced
parameters and an enforced applicability domain. Every regional correlation is a selectable,
provenance-bearing method, never a house default.

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 Vertical stress starts at the physical anchor

All three incumbent families integrate bulk density, but only one executable implementation makes
the first interval explicit: integration starts at ground level or mudline, not at the first density
sample. A density combiner must distinguish measured from synthetic samples and must expose every
fill. The exact dimensional conversion is `0.4335275 psi/ft per g/cc`; the dossier carries
`0.433528` as the product constant (`geomech-ppfg.md` §§2.1,5.1–5.3, T1–T2).

### 2.2 Synthetic density is a calibrated choice

The evidence contains exponential mudline-density, Gardner, Miller, Traugott, Alberty and Sayers families plus an
incumbent auto-best-fit interaction. Their constants and validity domains differ. One Traugott
parameter is `1000` in printed help and `5600` in the shipped manifest, so it has no defensible
default. Raster-only Sayers/Wendt equations and closed Traugott coefficients cannot be implemented
from the held corpus (`geomech-ppfg.md` §§2.1,3.1,4,6, T1–T3).

### 2.3 Pore-pressure ratios are method-specific

Eaton resistivity, sonic, velocity and corrected-drilling-exponent forms do not share one ratio:
the sonic ratio is inverted relative to the other three. Bowers needs a calibration crossplot and
its unloading exponent satisfies `U >= 1`, with `U = 1` reducing to loading. Normal pressure is
sea-floor anchored. Clamp behaviour differs silently between incumbent paths, including absence on
one input family (`geomech-ppfg.md` §§2.2,3.3,3.6, T1–T3).

### 2.4 Fracture pressure is not fracture gradient

The generalized form is `FP = K(Sv - alpha*Pp) + alpha*Pp + sigma_tec + T0`. `FP` is pressure;
`FG = FP/TVD/0.051948` is an equivalent mud weight only after the datum and depth are declared.
Coefficient families carry depth, overburden-premise and source-geography restrictions. The held
vendor coefficient and lithology tables are evidence of structure only and are not transcribed
(`geomech-ppfg.md` §§2.3,3.2,3.4,5.1, T1–T3).

### 2.5 Stress calculations are unit- and frame-sensitive

Dynamic modulus appears as Mpsi in one tool, GPa in an executable stress path, and psi in its
outputs. Tectonic strains mean minimum- and maximum-horizontal directions, never arbitrary x/y.
Inclined-well calculation needs all six total-stress components, a right-handed north/east/down
frame and an explicit compression sign. Dynamic-to-static identity is an incumbent convenience,
not a physical default (`geomech-ppfg.md` §§2.4–2.5,4, T1–T3).

### 2.6 Wellbore stability needs published forms, not raster truth

The union of incumbent criteria includes Mohr-Coulomb, Mogi-Coulomb, Modified Lade, two
Drucker-Prager forms, Hoek-Brown and Stassi d'Alia; Modified Wiebols-Cook is present only in the
held public teaching source. No one incumbent supplies the complete union with readable equations,
parameters and validity ranges. Raster-only identity is inventory, not implementation evidence
(`geomech-ppfg.md` §§3.8,4,6; `geomech-ppfg_critique.md` B-1, T2–T4).

## 3. SandiBumi as-built

### 3.1 Registered domain computation

`ABSENT` — the deterministic module registry and dispatcher contain no overburden, pore-pressure,
fracture-gradient, horizontal-stress or stability module (`src-tauri/src/modules.rs:434-574`). A
targeted source search found no alternate implementation.

### 3.2 Dynamic elastic-property seam

`PARTIAL` — the unconventional brittleness module computes dynamic Young's modulus and Poisson's
ratio from `DT`, `DTS` and `RHOB`, emits `YME` in Mpsi and `PR`, and rejects negative Poisson's
ratio (`src-tauri/src/unconventional.rs:489-606`). It has no static calibration, stress use,
anisotropic frame or PPFG provenance.

### 3.3 Generic carriage

`PARTIAL` — arbitrary scalar curves can be imported and stored, but generic curve carriage is not
a geomechanics interpretation. No domain-specific datum contract, validity gate, calibration
record, pressure/gradient pair or stress tensor schema is registered.

## 4. Requirements

#### SB-GEO-001 — Gate six independently versioned domain units [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST gate `vertical_stress`, `pore_pressure`, `fracture_pressure`,
`elastic_static`, `horizontal_stress` and `wellbore_stability` independently.

**Rationale.** Evidence completeness and release risk differ across the six units (dossier §4).

**As-built.** ABSENT — no domain unit is registered (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-GEO-T01, SB-GEO-T02

#### SB-GEO-002 — Type every depth and reference datum [P0] [status: ABSENT]

**Requirement.** Every input and output MUST declare depth unit, vertical-depth basis, reference
elevation and ground/mudline anchor; incompatible references MUST refuse composition.

**Rationale.** A datum mismatch silently changes both integration length and gradient (dossier §5.3 V-20, T2).

**As-built.** ABSENT — generic curves do not carry the complete domain datum contract.

**Verified by.** SB-GEO-T03, SB-GEO-T04

#### SB-GEO-003 — Integrate vertical stress from the physical anchor [P0] [status: ABSENT]

**Requirement.** `Sv` MUST equal water-column pressure plus the interval sum of bulk density times
`0.433528 psi/ft per g/cc`; the first interval MUST begin at ground or mudline.

**Rationale.** Starting at the first sample under-integrates every curve with a shallow gap (dossier §§2.1,5.1, T1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T05, SB-GEO-T06, SB-GEO-T07

#### SB-GEO-004 — Preserve measured and synthetic density provenance [P0] [status: ABSENT]

**Requirement.** A merged density curve MUST retain a per-sample measured/synthetic/missing mask,
the selected synthesis method and every filled interval; an unexplained constant fill MUST refuse.

**Rationale.** Incumbents otherwise hide both missing density and the stress contribution (dossier §§2.1,4, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T08, SB-GEO-T09

#### SB-GEO-005 — Select or fit synthetic density explicitly [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST offer only sourced synthetic-density methods and MAY offer an
auto-best-fit selector over an interval with measured density; it MUST store the candidates,
objective, winning method and residuals.

**Rationale.** No correlation is universally preferred; the auto interaction is a genuine incumbent advantage (dossier §4, T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T10, SB-GEO-T11

#### SB-GEO-006 — Enforce every correlation's applicability contract [P0] [status: ABSENT]

**Requirement.** Each correlation MUST declare depth range, input range, source geography,
lithology/compaction assumptions and excluded histories; out-of-domain samples MUST return null and
log the breached condition unless the method's source explicitly defines another policy.

**Rationale.** A depth-only gate misses binding geography and burial-history limits (dossier §§3.4,6.21, T2–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T12, SB-GEO-T13

#### SB-GEO-007 — Never make a vendor table implementation truth [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST NOT transcribe vendor overburden, Poisson-ratio or coefficient
lookup rows; adopted values MUST be derived from the named primary publication and versioned.

**Rationale.** `CONTRACT.md` §2.1 and dossier §5.4 prohibit vendor table truth.

**As-built.** ABSENT.

**Verified by.** SB-GEO-T14

#### SB-GEO-008 — Resolve one shared water density [P0] [status: ABSENT]

**Requirement.** A run MUST resolve one project-level water density for all compatible modules;
private correlation constants are permitted only when the publication embeds and labels them.

**Rationale.** The corpus contains eight carried values and six distinct equivalents (dossier §3.7, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T15, SB-GEO-T16

#### SB-GEO-009 — Anchor normal pressure at the water/formation boundary [P0] [status: ABSENT]

**Requirement.** Normal pressure MUST be a sea-floor/ground-anchored linear gradient with one
explicit sourced gradient; SandiBumi MUST NOT silently substitute a fresh- or saline-water value.

**Rationale.** Held sources disagree from `0.434` through `0.465 psi/ft` (dossier §§3.6,5.2, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T17, SB-GEO-T18

#### SB-GEO-010 — Apply Terzaghi effective stress with explicit Biot alpha [P0] [status: ABSENT]

**Requirement.** The engine MUST use `sigma_eff = Sv - alpha*Pp`, MUST store `alpha`, and MUST
reject values outside the selected method's sourced range.

**Rationale.** Omitting alpha changes every downstream pressure and stress (dossier §§2.2,5.1, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T19

#### SB-GEO-011 — Implement four distinct Eaton forms [P0] [status: ABSENT]

**Requirement.** Resistivity, sonic, velocity and corrected-drilling-exponent Eaton calculations
MUST be separate typed methods with factor and exponent exposed; sonic MUST use `Dtnct/Dtobs`, while
the other three use observed/trend orientation.

**Rationale.** Sharing one ratio creates the easiest silent sign error in this domain (dossier §§2.2,5.1, T2–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T20, SB-GEO-T21

#### SB-GEO-012 — Require readable trend inputs and output them [P0] [status: ABSENT]

**Requirement.** Every pore-pressure method MUST consume an explicit normal-compaction trend and
MUST output that trend, the raw estimate and its post-processing flags.

**Rationale.** A trend hidden inside a module cannot be audited or recalibrated (dossier critique m-15, T2–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T22

#### SB-GEO-013 — Calibrate Bowers before emission [P0] [status: ABSENT]

**Requirement.** Bowers loading MUST require a stored calibration crossplot and sourced `Vml`, `A`
and `B`; absence of any selected coefficient MUST block output.

**Rationale.** Incumbent coefficient sets differ by as much as `3.6 ppg` (dossier §3.3, T1–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T23, SB-GEO-T24

#### SB-GEO-014 — Make Bowers unloading algebraically consistent [P0] [status: ABSENT]

**Requirement.** Bowers unloading MUST enforce `U >= 1`; `U = 1` MUST reproduce the loading curve
exactly, and the maximum effective stress MUST be explicit.

**Rationale.** An incumbent prose statement contradicts its own printed equation (dossier §3.3 and critique M-10, T2–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T25, SB-GEO-T26

#### SB-GEO-015 — Block methods whose primary equation is missing [P0] [status: ABSENT]

**Requirement.** Katahara, Eberhart-Phillips, Alberty/McLean NCT, Sayers, Wendt and closed Traugott
variants MUST remain unavailable until their named primary equations and parameters are acquired;
vendor rasters or binaries MUST NOT fill the gap.

**Rationale.** The held corpus proves identity or partial parameters, not an implementable whole (dossier §§4,6, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T27

#### SB-GEO-016 — Apply one uniform pressure-limit policy [P0] [status: ABSENT]

**Requirement.** The hydrostatic floor and overburden ceiling MUST be a single default-Off policy
available to every pore-pressure method; when enabled, each clamped sample and bound MUST be logged.

**Rationale.** Incumbents apply different defaults and omit the option from some paths (dossier §3.6, critique M-4, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T28, SB-GEO-T29

#### SB-GEO-017 — Emit pressure and gradient as separate typed curves [P0] [status: ABSENT]

**Requirement.** Pore and fracture calculations MUST emit pressure in psi and equivalent gradient
in ppg under distinct names; a pressure curve MUST NOT be relabelled as a gradient.

**Rationale.** The corpus contains this exact documentation defect (dossier J-D5/J-D10, T2).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T30

#### SB-GEO-018 — Implement the alpha-aware generalized fracture equation [P0] [status: ABSENT]

**Requirement.** Fracture pressure MUST use `K*(Sv-alpha*Pp)+alpha*Pp+sigma_tec+T0`, with every
term typed and persisted.

**Rationale.** This is the evidence-backed superset of the readable incumbent forms (dossier §§2.3,5.1, T2–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T31, SB-GEO-T32

#### SB-GEO-019 — Keep K relationships explicit [P1] [status: ABSENT]

**Requirement.** User-Poisson, published Poisson-polynomial, Daines, constant-K, Matthews–Kelly and
Zamora relationships MUST be separate methods with separate provenance and gates; one MUST NOT
silently fall back to another.

**Rationale.** Their premises and coefficient sources are not interchangeable (dossier §§2.3,4, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T33

#### SB-GEO-020 — Source Matthews–Kelly coefficients from the paper [P1] [status: ABSENT]

**Requirement.** The method MUST use coefficients re-derived from Matthews and Kelly (1967), MUST
store the coefficient-set version and validity range, and MUST NOT use vendor-file rows as truth.

**Rationale.** The contract's one retained evidence exception is not an implementation source (dossier §6.4; CONTRACT §2.1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T14, SB-GEO-T34

#### SB-GEO-021 — Enforce the Matthews–Kelly overburden premise [P0] [status: ABSENT]

**Requirement.** A run MUST compare `Sv/TVD` with the method's `1 psi/ft` premise, require a sourced
tolerance, and warn, record the departure and report implied bias when breached.

**Rationale.** This premise can dominate the coefficient-set difference (dossier §§3.4,5.3 V-9b, T2).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T35

#### SB-GEO-022 — Enforce declared source geography [P0] [status: ABSENT]

**Requirement.** A correlation MUST refuse or explicitly warn outside the geography declared by
its primary source; the run record MUST identify the breached scope without creating a house preset.

**Rationale.** Several held correlations carry explicit regional invalidity (dossier §§3.4,6.21, T2–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T12, SB-GEO-T36

#### SB-GEO-023 — Expose every Poisson-polynomial breakpoint [P1] [status: ABSENT]

**Requirement.** Each piecewise published Poisson relationship MUST expose its breakpoint and
coefficient provenance; conflicting shipped and documented breakpoints MUST remain named choices
pending the primary paper.

**Rationale.** One discrepancy changes fracture gradient by `0.173 ppg` (dossier §3.2, critique M-1, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T37, SB-GEO-T38

#### SB-GEO-024 — Rebuild the Daines table from primary literature [P1] [status: ABSENT]

**Requirement.** Daines lithology values MUST be sourced from Daines (1982), retain the published
family/variant semantics, and MUST NOT be transcribed from the vendor lookup file.

**Rationale.** The vendor file exposes structure but its values are prohibited evidence (dossier J-O-8; critique m-4/m-13, T1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T14, SB-GEO-T39

#### SB-GEO-025 — Plot published fracture-pressure bounds as an envelope [P1] [status: ABSENT]

**Requirement.** SandiBumi SHOULD calculate the Hubbert–Willis lower and upper pressure bounds and
MUST label them as a sanity envelope, not as a calibrated K relationship.

**Rationale.** The pair is absent from the incumbents and is useful without pretending to predict (dossier §6.21b, T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T40

#### SB-GEO-026 — Limit fracture pressure only by explicit policy [P0] [status: ABSENT]

**Requirement.** An overburden ceiling MUST default Off, MUST apply uniformly to all fracture
methods when enabled, and MUST count and provenance every limited sample.

**Rationale.** A shipped incumbent default is undocumented (dossier X-6, T1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T41

#### SB-GEO-027 — Compute minimum and maximum horizontal stress explicitly [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the poroelastic-plus-tectonic-strain forms for `Shmin`
and `SHmax`, with Biot coefficient, static elastic properties and two named strains explicit.

**Rationale.** The readable incumbent forms converge, but their default calibrations do not (dossier §§2.4,4, T1–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T42, SB-GEO-T43

#### SB-GEO-028 — Name strains by stress direction [P0] [status: ABSENT]

**Requirement.** Strains MUST be named `eps_hmin` and `eps_Hmax`; ambiguous `x`/`y` names MUST be
rejected unless a declared frame maps them unambiguously.

**Rationale.** One executable source resolves the intended directions and exposes the naming trap (dossier J-D13, T1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T44

#### SB-GEO-029 — Reuse dynamic properties without broadening their meaning [P1] [status: PARTIAL]

**Requirement.** Geomechanics MAY consume the existing `YME` and `PR` outputs only when their
dynamic type, units, inputs and run provenance are present; it MUST NOT treat them as static.

**Rationale.** Dynamic properties already exist but identity conversion is physically unjustified (dossier §2.4, T1–T3).

**As-built.** PARTIAL — dynamic outputs exist (`src-tauri/src/unconventional.rs:489-606`).

**Verified by.** SB-GEO-T45, SB-GEO-T46

#### SB-GEO-030 — Require sourced dynamic-to-static transforms [P0] [status: ABSENT]

**Requirement.** Young's-modulus and Poisson transformations MUST be separately versioned,
calibrated and cited; identity MUST NOT be the default, and absence MUST block static stress.

**Rationale.** Both incumbent identity defaults are convenience settings (dossier §4, T1–T2).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T47

#### SB-GEO-031 — Make modulus conversions dimensional [P0] [status: PARTIAL]

**Requirement.** Mpsi, psi and GPa MUST be distinct types with explicit conversions; correlation
code MUST declare whether its native output is psi, MPa or bar before any multiplier is applied.

**Rationale.** The corpus contains `10^6`, `145.038` and `14.5038` traps (dossier §§2.5,5.4, T1–T2).

**As-built.** PARTIAL — `YME` is labelled Mpsi, but no domain-wide type gate exists.

**Verified by.** SB-GEO-T48, SB-GEO-T49, SB-GEO-T50

#### SB-GEO-032 — Keep stress and stress gradient distinct [P0] [status: ABSENT]

**Requirement.** Every horizontal and vertical stress output MUST declare pressure or pressure-per-
depth and MUST use different curve identities for the two quantities.

**Rationale.** One incumbent source mixes GPa input, psi output and psi/ft output in one path (dossier critique m-8, T1).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T51

#### SB-GEO-033 — Transform inclined stresses in a declared frame [P1] [status: ABSENT]

**Requirement.** The inclined-stress unit MUST accept six total-stress components, use a right-
handed X-north/Y-east/Z-down frame, declare positive- or negative-in-compression, and output ordered
principal stresses plus orientations.

**Rationale.** An undeclared frame can produce plausible but rotated results (dossier §§1.2,4, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T52, SB-GEO-T53

#### SB-GEO-034 — Assert total versus effective input state [P0] [status: ABSENT]

**Requirement.** Stress-tensor inputs MUST declare total or effective state; the inclined transform
MUST consume total stress and MUST apply pore-pressure correction exactly once downstream.

**Rationale.** Double correction is a silent stress error (dossier §4 inclined-well adoption, T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T54

#### SB-GEO-035 — Preserve omitted physical terms as explicit inputs [P2] [status: ABSENT]

**Requirement.** Thermal stress, depletion and depth-of-damage MUST be explicit optional terms with
zero contribution only by an attributed user choice; inclined calculation MUST NOT silently drop them.

**Rationale.** A readable incumbent inclined path omits all three (dossier §§1.2,4, T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T55

#### SB-GEO-036 — Implement failure criteria from public equations [P1] [status: ABSENT]

**Requirement.** Mohr-Coulomb, Mogi-Coulomb, Modified Lade, inscribed and circumscribed Drucker-
Prager, Hoek-Brown and Modified Wiebols-Cook MUST each have a public equation source, validity
contract, parameter schema and independent test before activation.

**Rationale.** Criterion names in vendor rasters do not provide implementable truth (dossier §§3.8,4,6, T2–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T56, SB-GEO-T57

#### SB-GEO-037 — Solve Drucker–Prager numerically from invariants [P1] [status: ABSENT]

**Requirement.** Drucker–Prager MUST solve `sqrt(J2) = k + alpha_dp*J1` with a bounded numeric root
and MUST report convergence, bracket and residual.

**Rationale.** The held extracted closed form is garbled; the invariant equation is readable (dossier §4, T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T58

#### SB-GEO-038 — Classify shear-failure modes separately from failure [P2] [status: ABSENT]

**Requirement.** SandiBumi SHOULD retain four theoretical shear-failure modes, MAY merge them into
two operational classes, and MUST expose the sourced `45°` orientation threshold.

**Rationale.** Mode reconciles a stress model with borehole-image evidence (dossier §§1.2,4, T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T59

#### SB-GEO-039 — Validate every stability input before solve [P0] [status: ABSENT]

**Requirement.** Stability calculation MUST validate pressure, stresses, UCS, friction angle,
tensile strength, Biot alpha and Poisson's ratio against a selected source range; it MUST NOT clamp
an invalid input silently.

**Rationale.** The held manual supplies a coherent input-QC surface (dossier §5.2, T2).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T60

#### SB-GEO-040 — Bind strength correlations to native units [P0] [status: ABSENT]

**Requirement.** Every UCS correlation MUST declare its native output unit; the engine MUST apply
`145.038` only to MPa-native forms and `14.5038` only to bar-native forms.

**Rationale.** A universal multiplier changes some correlations by two orders of magnitude (dossier §5.3 V-13/V-14, T2).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T49, SB-GEO-T50, SB-GEO-T61

#### SB-GEO-041 — Use atan2 for every angle back-transform [P0] [status: ABSENT]

**Requirement.** Every azimuth or orientation recovered from vector components MUST use `atan2`
and MUST round-trip western and southern quadrants.

**Rationale.** One-argument arctangent loses quadrant information (dossier §5.4 rule 2, T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T62

#### SB-GEO-042 — Make unset sourced parameters block execution [P0] [status: ABSENT]

**Requirement.** Every parameter MUST resolve a non-empty source; every value marked `ABSENT` in
§5 MUST block the dependent method until a sourced run value is supplied.

**Rationale.** A plausible default is silent wrongness (CONTRACT §2; dossier §5.4 rule 9).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T63

#### SB-GEO-043 — Address parameter files semantically and ordinally [P1] [status: ABSENT]

**Requirement.** Imported parameter sets MUST match both semantic key and declared ordinal; a
mismatch MUST be a load error, not a shifted assignment.

**Rationale.** Ordinal-only vendor manifests are fragile (dossier §5.4 rule 7, T1–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T64

#### SB-GEO-044 — Version local calibration without promoting it to default [P0] [status: ABSENT]

**Requirement.** Every local trend, strain, transform or correlation fit MUST carry calibration
data identity, interval, objective, residuals, author and version; it MUST remain a named run preset
and MUST NOT become a general default.

**Rationale.** Every held delivered-study parameter is asset-specific evidence (dossier §6.22, T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T65

#### SB-GEO-045 — Refuse extrapolation outside a declared range [P0] [status: ABSENT]

**Requirement.** A correlation MUST emit null plus a structured reason outside its declared range;
an override MAY exist only as an explicit, provenance-recorded user action.

**Rationale.** Several polynomial and shallow-density methods become unphysical when extrapolated (dossier §§3.4,5.3 V-10, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T66

#### SB-GEO-046 — Prohibit raster- and binary-only implementation truth [P0] [status: ABSENT]

**Requirement.** An equation or coefficient available only as a vendor raster, binary or opaque
artifact MUST remain unavailable until independently supported by readable public literature.

**Rationale.** `CONTRACT.md` §§2.1–2.2 and dossier §5.4 bind both provenance and correctness.

**As-built.** ABSENT.

**Verified by.** SB-GEO-T27, SB-GEO-T67

#### SB-GEO-047 — Separate imported, computed and interpreted identities [P1] [status: ABSENT]

**Requirement.** Imported pressure/stress curves, deterministic outputs and interpreter-edited
curves MUST have distinct identities and provenance; one MUST NOT overwrite another.

**Rationale.** Comparison and calibration require the original evidence to remain recoverable (dossier §5 adoption spec, T1–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T68

#### SB-GEO-048 — Export a complete geomechanics run record [P1] [status: ABSENT]

**Requirement.** Export MUST include curves, units, datum, methods, parameters, sources, calibration
records, masks, clamp counts, validity breaches and software version in machine-readable form.

**Rationale.** A pressure curve without its assumptions cannot be reproduced (dossier §5.4 rules 9–14, T1–T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T69

#### SB-GEO-049 — Keep shared parameters single-valued within a run [P0] [status: ABSENT]

**Requirement.** Water density, normal gradient, Biot alpha and depth reference MUST each resolve to
one typed run value; modules MUST NOT retain private alternatives.

**Rationale.** The incumbent corpus contains contradictory constants within one workflow (dossier §§3.6–3.7, T1–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T15, SB-GEO-T70

#### SB-GEO-050 — Execute every worked example [P1] [status: ABSENT]

**Requirement.** Every numeric example in product documentation MUST be generated by the same
released calculation path and MUST fail the documentation gate if its expected value changes.

**Rationale.** Static examples otherwise drift from code (dossier §5.4 rule 11, T4).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T71

#### SB-GEO-051 — Keep post-processing visible [P1] [status: ABSENT]

**Requirement.** Smoothing, output filtering, limiting and shale-selection logic MUST be separate,
ordered operations with parameters and masks; dual indicators MUST expose their agreement rule.

**Rationale.** The critique recovered incumbent outputs and filters omitted from the first dossier draft (critique m-15, T2–T3).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T72

#### SB-GEO-052 — Gate acquisition-dependent methods individually [P1] [status: ABSENT]

**Requirement.** A missing paper or equation MUST disable only its dependent method, identify the
named acquisition gap and leave independently sourced methods available.

**Rationale.** Evidence gaps are method-specific, not a reason to weaken the whole domain (dossier §6).

**As-built.** ABSENT.

**Verified by.** SB-GEO-T02, SB-GEO-T73

## 5. Parameters

`ABSENT — ships with no default` is a deliberate product value: the dependent calculation blocks
until a sourced value is supplied. Vendor chart and lookup-table contents are not transcribed.
Reference presets are selectable only with their source and applicability metadata; they are never
universal physical constants.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Density integration conversion | `RHO_TO_PSI_FT` | `0.433528` | psi/ft per g/cc | `geomech-ppfg.md` §5.2; derived there as `1000*9.80665*0.3048/6894.757` | T2 |
| Pressure-gradient to mud-weight conversion | `PSIFT_TO_PPG` | `19.25` | ppg per psi/ft | `geomech-ppfg.md` §5.2; printed vendor form | T2 |
| Mud-weight to pressure-gradient conversion | `PPG_TO_PSIFT` | `0.051948` | psi/ft per ppg | `geomech-ppfg.md` §5.2; printed vendor form | T2 |
| MPa to psi | `MPA_TO_PSI` | `145.038` | psi/MPa | `geomech-ppfg.md` §§2.5,5.2 | T2 |
| bar to psi | `BAR_TO_PSI` | `14.5038` | psi/bar | `geomech-ppfg.md` §§2.5,5.2 | T2 |
| GPa to psi | `GPA_TO_PSI` | `145038` | psi/GPa | `geomech-ppfg.md` §2.5; executable input/output comparison | T1 |
| Water density | `RHO_WATER` | **ABSENT — ships with no default** | g/cc | `geomech-ppfg.md` §§3.7,5.2; eight carried values conflict | T1–T3 |
| Normal pressure gradient | `GRAD_NORMAL` | **ABSENT — ships with no default** | psi/ft | `geomech-ppfg.md` §§3.6,5.2; `0.434`, `0.45`, `0.455`, `0.465` evidence conflicts | T1–T3 |
| Normal-gradient reference preset | `GRAD_NORMAL_REF` | `8.66` | ppg | `geomech-ppfg.md` §5.2; documented vendor default equivalent to `0.44987 psi/ft` | T3 |
| Pore-pressure lower limit | `LIMIT_PP_HYDRO` | `Off` | boolean | `geomech-ppfg.md` §§3.6,5.2 adoption decision | T1–T4 |
| Pore-pressure upper limit | `LIMIT_PP_OBG` | `Off` | boolean | `geomech-ppfg.md` §§3.6,5.2 uniform-policy decision | T1–T4 |
| Fracture-pressure upper limit | `LIMIT_FP_OBG` | `Off` | boolean | `geomech-ppfg.md` §§3.6,5.2 | T1 |
| Air density | `RHO_AIR` | `0.0` | g/cc | `geomech-ppfg.md` §1.2 shared overburden inputs | T3 |
| Exponential mudline density | `EXP_RHO0` | `1.9533` | g/cc | `geomech-ppfg.md` §§2.1,5.2; printed `16.3 ppg` and cross-tool identity | T2–T3 |
| Exponential density exponent | `EXP_RHO_N` | `0.6` | dimensionless | `geomech-ppfg.md` §§2.1,5.2; two-source agreement | T2–T3 |
| Exponential density depth constant | `EXP_RHO_D0` | `3125` | ft | `geomech-ppfg.md` §5.2 | T2 |
| Gardner coefficient | `GARDNER_A` | `0.23` | native equation coefficient | `geomech-ppfg.md` §§2.1,5.2; two-source agreement | T2–T3 |
| Gardner exponent | `GARDNER_B` | `0.25` | dimensionless | `geomech-ppfg.md` §§2.1,5.2; two-source agreement | T2–T3 |
| Miller matrix density | `MILLER_RHOMA` | `2.68` | g/cc | `geomech-ppfg.md` §§2.1,5.2; manual/manifest agreement | T1–T2 |
| Miller water density | `MILLER_RHOW` | `1.03` | g/cc | `geomech-ppfg.md` §§2.1,5.2; correlation-embedded value | T1–T3 |
| Miller initial porosity | `MILLER_PORA` | `0.35` | v/v | `geomech-ppfg.md` §§2.1,5.2 | T1–T3 |
| Miller transition porosity | `MILLER_PORB` | `0.30` | v/v | `geomech-ppfg.md` §§2.1,5.2; revised printed/manifest value | T1–T2 |
| Miller decline coefficient | `MILLER_KDECL` | `0.0035` | dimensionless | `geomech-ppfg.md` §§2.1,5.2 | T1–T3 |
| Miller curvature | `MILLER_PCUR` | `1.09` | dimensionless | `geomech-ppfg.md` §§2.1,5.2 | T1–T3 |
| Miller validity ceiling | `MILLER_ZMAX` | `2000` | ft below mudline | `geomech-ppfg.md` §5.2 | T2 |
| Barker–Wood coefficient | `BW_A` | `5.3` | native equation coefficient | `geomech-ppfg.md` §5.2 | T2 |
| Barker–Wood exponent | `BW_B` | `0.1356` | dimensionless | `geomech-ppfg.md` §5.2 | T2 |
| Barker–Wood water density | `BW_WATER` | `8.55` | lb/gal | `geomech-ppfg.md` §5.2; correlation-embedded assumption | T2 |
| Barker–Wood water-depth range | `BW_WD_RANGE` | `2000–7000` | ft | `geomech-ppfg.md` §5.2 | T2 |
| Barker–Wood depth-below-mudline ceiling | `BW_ZMAX` | `8000` | ft | `geomech-ppfg.md` §5.2; approximate source limit | T2 |
| Alberty NSC seawater density | `ANSC_RHOSW` | `1.038` | g/cc | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty NSC formation-fluid density | `ANSC_RHOF` | `1.073` | g/cc | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty NSC initial porosity | `ANSC_PHI0` | `0.4` | v/v | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty NSC matrix density | `ANSC_RHOMA` | `2.65` | g/cc | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty NSC compaction constant | `ANSC_OC` | `1000` | native equation unit | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty NSC lateral/vertical ratio | `ANSC_K0` | `0.8` | dimensionless | `geomech-ppfg.md` §§3.1,5.2; manifest-only exposed parameter | T1 |
| Traugott power-law constant | `TRAUGOTT_CPG` | **ABSENT — ships with no default** | native equation unit | `geomech-ppfg.md` §§3.1,5.2; printed `1000` versus manifest `5600` | T1–T2 |
| Smectite density intercept | `SI_SMECTITE_A` | `2.918` | g/cc | `geomech-ppfg.md` §5.2 printed equation | T2 |
| Smectite sonic coefficient | `SI_SMECTITE_B` | `-0.00517` | g/cc per µs/ft | `geomech-ppfg.md` §5.2 printed equation | T2 |
| Illite density intercept | `SI_ILLITE_A` | `3.044` | g/cc | `geomech-ppfg.md` §5.2 printed equation | T2 |
| Illite sonic coefficient | `SI_ILLITE_B` | `-0.00505` | g/cc per µs/ft | `geomech-ppfg.md` §5.2 printed equation | T2 |
| Transformation start temperature | `SI_TBEG` | `160` | °F | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Transformation end temperature | `SI_TEND` | `220` | °F | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Alberty/McLean NCT coefficients | `SI_NCT_ABCD` | **ABSENT — ships with no default** | coefficient set | `geomech-ppfg.md` §§4.1 J-O-4,6.1; primary papers missing | T1–T2 |
| Resistivity Eaton exponent | `EATON_N_R` | `1.2` | dimensionless | `geomech-ppfg.md` §5.2; cross-tool agreement | T2–T3 |
| Sonic Eaton exponent | `EATON_N_DT` | `3.0` | dimensionless | `geomech-ppfg.md` §5.2; cross-tool agreement | T2–T3 |
| Velocity Eaton exponent | `EATON_N_V` | `3.0` | dimensionless | `geomech-ppfg.md` §5.2; cross-tool agreement | T2–T3 |
| Corrected-drilling-exponent Eaton exponent | `EATON_N_DXC` | `1.2` | dimensionless | `geomech-ppfg.md` §5.2; cross-tool agreement | T2–T3 |
| Smectite/illite Eaton exponent | `EATON_N_SI` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §§3.5,5.2; printed `3` versus manifest `4.8` | T1–T2 |
| Eaton factor | `EATON_A` | `1.0` | dimensionless | `geomech-ppfg.md` §5.2; printed cross-tool value | T2–T3 |
| Bowers mudline velocity | `BOWERS_VML` | `5000` | ft/s | `geomech-ppfg.md` §§3.3,5.2; agreement after unit correction | T1–T3 |
| Bowers loading coefficient | `BOWERS_A` | **ABSENT — ships with no default** | calibrated coefficient | `geomech-ppfg.md` §§3.3,5.2; vendor sets disagree | T1–T4 |
| Bowers loading exponent | `BOWERS_B` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §§3.3,5.2; vendor sets disagree | T1–T4 |
| Bowers unloading exponent | `BOWERS_U` | **ABSENT — ships with no default** | dimensionless, `>=1` | `geomech-ppfg.md` §§2.2,5.2; calibration required | T2–T4 |
| Bowers maximum effective stress | `BOWERS_SIGMA_MAX` | **ABSENT — ships with no default** | psi | `geomech-ppfg.md` §2.2 unloading equation; run-specific history | T2–T3 |
| Chapman mudline slowness | `CHAPMAN_DTML` | `195` | µs/ft | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Chapman matrix slowness | `CHAPMAN_DTMA` | `59` | µs/ft | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Chapman coefficient | `CHAPMAN_C` | `5480` | native equation unit | `geomech-ppfg.md` §§4.2 X-20,5.2; range conflict retained | T1–T2 |
| Miller PP mudline velocity | `MPP_VML` | `5000` | ft/s | `geomech-ppfg.md` §5.2 | T1–T2 |
| Miller PP matrix velocity | `MPP_VMA` | `14300` | ft/s | `geomech-ppfg.md` §5.2 | T1–T2 |
| Miller PP compaction coefficient | `MPP_LAMBDA` | `0.00025` | native equation unit | `geomech-ppfg.md` §5.2 | T1–T2 |
| Eberhart-Phillips constants | `EHP_CONSTANTS` | **ABSENT — ships with no default** | coefficient set | `geomech-ppfg.md` §§4.1 J-D14,6.3; primary paper required | T1–T2 |
| Dxc linear trend top | `DXC_TOP` | `1.0` | dimensionless | `geomech-ppfg.md` §5.2 | T2 |
| Dxc linear trend base | `DXC_BASE` | `1.4` | dimensionless | `geomech-ppfg.md` §5.2 | T2 |
| Dxc power-law intercept | `DXC_A` | `0.65` | dimensionless | `geomech-ppfg.md` §5.2; unpublished vendor attribution | T2 |
| Dxc power-law gradient | `DXC_B` | `1.7` | dimensionless | `geomech-ppfg.md` §5.2; unpublished vendor attribution | T2 |
| Arps temperature constant | `ARPS_C` | `6.77` | °F | `geomech-ppfg.md` §5.2 | T2 |
| Archie reference-water resistivity | `ARCHIE_RW100` | **ABSENT — ships with no default** | ohm-m at 100 °F | `geomech-ppfg.md` §5.2 X-17b; manifest blank, printed reference `0.056` | T1–T3 |
| Archie cementation exponent | `ARCHIE_M` | `1.87` | dimensionless | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Archie coefficient | `ARCHIE_A` | `0.81` | dimensionless | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Archie initial porosity | `ARCHIE_PHI0` | `0.4` | v/v | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Archie compaction constant | `ARCHIE_OC` | **ABSENT — ships with no default** | psi | `geomech-ppfg.md` §§3.9,5.2; printed `1000` versus manifest `5.2` | T1–T2 |
| Archie Eaton exponent | `ARCHIE_EATON_N` | `1.2` | dimensionless | `geomech-ppfg.md` §5.2; manual/manifest agreement | T1–T2 |
| Semi-log resistivity trend top | `RT_TOP` | `1.0` | ohm-m | `geomech-ppfg.md` §5.2 | T2 |
| Semi-log resistivity trend base | `RT_BASE` | `2.0` | ohm-m | `geomech-ppfg.md` §5.2 | T2 |
| Poisson polynomial set A breakpoint | `PR_A_BREAK` | `5000` | ft below mudline | `geomech-ppfg.md` §§3.2,5.2; three-source agreement | T1–T3 |
| Poisson polynomial set B breakpoint | `PR_B_BREAK` | **ABSENT — ships with no default** | ft below mudline | `geomech-ppfg.md` §§3.2,5.2; manifest `4300`, printed sources `5000` | T1–T3 |
| Poisson polynomial coefficients | `PR_POLY_COEFFS` | **NON-ADOPTABLE — cited for verification** | piecewise coefficient sets | `geomech-ppfg.md` §§2.3,5.2; primary 1997 paper required | T1–T3 |
| Matthews–Kelly coefficients | `MK_COEFFS` | **NON-ADOPTABLE — cited for verification** | cubic coefficient set | `geomech-ppfg.md` §§2.3,6.4; re-derive from 1967 paper | T1–T3 |
| Matthews–Kelly depth argument | `MK_Z` | `TVD - water_depth - air_gap` | ft below mudline | `geomech-ppfg.md` §§2.3,5.2 | T1–T3 |
| Matthews–Kelly cubic depth range | `MK_Z_RANGE` | `0–40000` | ft below mudline | `geomech-ppfg.md` §5.2 | T1 |
| Matthews–Kelly overburden premise | `MK_OBG_PREMISE` | `1` | psi/ft | `geomech-ppfg.md` §§3.4,5.2 | T2 |
| Matthews–Kelly premise tolerance | `MK_OBG_TOL` | **ABSENT — ships with no default** | psi/ft | `geomech-ppfg.md` §5.3 V-9b requires a declared tolerance but supplies none | T2 |
| Matthews–Kelly source geography | `MK_SCOPE` | declared source region only | enum | `geomech-ppfg.md` §§3.4,5.2; vendor-declared invalidity elsewhere | T2 |
| Daines lithology values | `DAINES_NU_TABLE` | **NON-ADOPTABLE — cited for verification** | lithology-to-Poisson table | `geomech-ppfg.md` §§2.3,5.2; source from Daines (1982), not vendor file | T1 |
| Constant K | `K_FIXED` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §5.2; shipped reference `0.724` lacks primary method basis | T1 |
| Tensile strength | `T0` | **ABSENT — ships with no default** | psi | `geomech-ppfg.md` §5.2; vendor reference values are scope-specific | T1–T4 |
| Tectonic stress contribution | `SIGMA_TEC` | **ABSENT — ships with no default** | psi | `geomech-ppfg.md` §5.2; direction and calibration required | T1–T2 |
| Biot coefficient | `BIOT_ALPHA` | `1.0` | dimensionless | `geomech-ppfg.md` §5.2; executable, manual and public-study agreement | T1–T4 |
| Minimum-horizontal tectonic strain | `EPS_HMIN` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §§2.4,5.2; calibration required | T1–T4 |
| Maximum-horizontal tectonic strain | `EPS_HMAX` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §§2.4,5.2; calibration required | T1–T4 |
| Dynamic-to-static Young transform | `YME_D2S` | **ABSENT — ships with no default** | versioned function | `geomech-ppfg.md` §§2.4,5.2; identity rejected | T1–T4 |
| Dynamic-to-static Poisson transform | `PR_D2S` | **ABSENT — ships with no default** | versioned function | `geomech-ppfg.md` §§2.4,5.2; identity rejected | T1–T4 |
| Horizontal-stress ratio | `SHMAX_SHMIN` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §5.2; local calibration required | T4 |
| Fault friction coefficient | `FAULT_MU` | **ABSENT — ships with no default** | dimensionless | `geomech-ppfg.md` §5.2; competing references and local calibration | T2–T4 |
| Dynamic modulus constant | `DYN_MOD_C` | `1.34747e4` | Mpsi form for g/cc and µs/ft | `geomech-ppfg.md` §§2.5,5.2 | T2 |
| Dynamic modulus unit | `YME_DYN_UNIT` | `Mpsi` | enum | `geomech-ppfg.md` §§2.5,5.2; J-D2/J-D3 resolution | T1–T3 |
| Compression sign | `STRESS_SIGN` | positive in compression | enum | `geomech-ppfg.md` §4 inclined-stress adoption | T3 |
| Stress frame | `STRESS_FRAME` | right-handed X north, Y east, Z down | enum | `geomech-ppfg.md` §4 inclined-stress adoption | T3 |
| Shear-mode angle | `SHEAR_MODE_ANGLE` | `45` | degrees | `geomech-ppfg.md` §§1.2,4 | T3 |
| Stability pore-pressure range | `WBS_PP_RANGE` | `2–16` | ppg | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability minimum-stress range | `WBS_SHMIN_RANGE` | `4–28` | ppg | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability maximum-stress range | `WBS_SHMAX_RANGE` | `4–28` | ppg | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability vertical-stress range | `WBS_SV_RANGE` | `10–24` | ppg | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability UCS range | `WBS_UCS_RANGE` | `100–10000` | psi | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability friction-angle range | `WBS_PHI_RANGE` | `10–45` | degrees | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability tensile-strength range | `WBS_T0_RANGE` | `0–UCS/10` | psi | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability Biot range | `WBS_ALPHA_RANGE` | `0.4–1` | dimensionless | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Stability Poisson range | `WBS_NU_RANGE` | `0.1–0.5` | dimensionless | `geomech-ppfg.md` §5.2 manual QC range | T2 |
| Output-filter enable | `OUTPUT_FILTER` | `No` | boolean | `geomech-ppfg_critique.md` m-15; printed vendor default | T3 |
| Output-filter window | `OUTPUT_FILTER_N` | `3`, odd | samples | `geomech-ppfg_critique.md` m-15; printed vendor default | T3 |

## 6. Acceptance tests

Every expected value is printed by the cited source or derived in the row from its cited equation.
Refusal expectations are exact; numeric tolerances are explicit.

| ID | Input and operation | Expected value | Source of expected value |
|---|---|---|---|
| `SB-GEO-T01` | Query the domain registry | Six separately versioned gates with the exact names in SB-GEO-001 | `geomech-ppfg.md` §4 adoption boundary |
| `SB-GEO-T02` | Disable one of the six gates and run a valid fixture for another | Enabled unit remains available with unchanged validation | Dossier §4 independent capability inventory |
| `SB-GEO-T03` | Compose a vertical-stress curve referenced to sea level with fracture depth referenced to rig floor | Refusal naming both incompatible references | Dossier §5.3 V-20 |
| `SB-GEO-T04` | Convert equivalent `1000 ft` and `304.8 m` TVD inputs under one datum | Identical canonical depth, tolerance `1e-12 ft` | Exact foot definition; dossier §2.5 unit contract |
| `SB-GEO-T05` | Water pressure `450 psi`, then `100 ft` of `2.0 g/cc` density | `Sv = 450 + 2*0.433528*100 = 536.7056 psi`, tolerance `1e-7` | Dossier §§2.1,5.1 integration equation and constant |
| `SB-GEO-T06` | First density sample at `100 ft` below the anchor, constant `2.0 g/cc` | First `Sv` contribution is `86.7056 psi`, not zero, tolerance `1e-7` | Dossier §5.3 V-4; executable anchor semantics |
| `SB-GEO-T07` | Integrate the same constant-density column in ft/g/cc and m/kg/m³ | Pressure curves agree to `1e-9 psi` | Dossier §5.3 V-3; exact conversions |
| `SB-GEO-T08` | Merge measured values at samples 1 and 3 with synthetic value at sample 2 | Mask is `[measured,synthetic,measured]`; all source identities retained | Dossier §§2.1,4 density-combiner requirement |
| `SB-GEO-T09` | Density gap with neither measured values nor selected synthesis method | Null stress across the unresolved interval plus structured refusal | Dossier §2.1 missing-density rule |
| `SB-GEO-T10` | Candidate predictions `[2,2,2]` and `[2,3,2]` against measured `[2,2,2]`; least-squares auto fit | First candidate selected; SSE `0` versus `1` | Dossier §4 auto-best-fit adoption; direct SSE arithmetic |
| `SB-GEO-T11` | Request auto fit over an interval containing no measured density | Refusal; no method selected | Dossier §4 requires measured-density fit evidence |
| `SB-GEO-T12` | Run a scoped correlation with `scope=OUTSIDE_DECLARED_SCOPE` | Null outputs and a breached-geography record | Dossier §§3.4,5.3 V-9c/V-10 |
| `SB-GEO-T13` | Register a regional correlation without depth or applicability metadata | Registration refusal listing missing fields | Dossier §6.21 and `04_CORE_REQUIREMENTS.md` SB-CORE-004 rationale |
| `SB-GEO-T14` | Scan the packaged module resources for vendor `.obg`, Poisson-table or coefficient-row content | Zero embedded vendor lookup rows/files | `CONTRACT.md` §2.1; dossier §5.4 |
| `SB-GEO-T15` | Two modules request water density in one run | Both resolve the exact same typed value and source id | Dossier §5.3 V-21 |
| `SB-GEO-T16` | A published correlation declares embedded `8.55 lb/gal` while run water density differs | Correlation uses and labels `8.55`; other modules retain shared run value | Dossier §5.2 Barker–Wood row and V-21 exception |
| `SB-GEO-T17` | Ground/mudline pressure `450 psi`, gradient `0.45 psi/ft`, depth `1000 ft` below anchor | Normal pressure `900 psi`, tolerance `1e-12` | Dossier §2.2 sea-floor-anchored linear form |
| `SB-GEO-T18` | Request normal pressure with `GRAD_NORMAL` unset | Refusal naming the absent parameter | Dossier §§3.6,5.2 conflicting gradients |
| `SB-GEO-T19` | `Sv=10000`, `Pp=5000`, `alpha=0.8` | Effective stress `6000 psi`, tolerance `1e-12` | Dossier §2.2 Terzaghi equation |
| `SB-GEO-T20` | Eaton sonic: `Sv=10000`, `Pn=4500`, `a=1`, `Dtnct/Dtobs=0.8`, `n=3` | `Pp=7184 psi`, tolerance `1e-9` | Dossier §§2.2,5.1 sonic equation; direct substitution |
| `SB-GEO-T21` | Increase observed sonic slowness while holding its trend fixed; separately increase observed R, V and Dxc | Sonic Pp increases; each other Pp decreases | Dossier §5.3 V-5 |
| `SB-GEO-T22` | Run any valid Eaton fixture | Output contains selected NCT, raw Pp, final Pp and filter/clamp masks | Dossier critique m-15 and §5.3 V-8c |
| `SB-GEO-T23` | Bowers request with `A` unset | Refusal before curve allocation | Dossier §§3.3,5.2 |
| `SB-GEO-T24` | Bowers calibration record with coefficients but no interval or residuals | Refusal naming missing calibration evidence | Dossier §4 requires calibration crossplot |
| `SB-GEO-T25` | Bowers loading `V=Vml+A*sigma^B`; invert through unloading with `U=1` | Recovered `sigma` equals input to `1e-9` over `100–10000 psi` | Dossier §5.3 V-6/V-7 |
| `SB-GEO-T26` | Bowers unloading with `U=0` | Refusal; no constant effective-stress curve emitted | Dossier §2.2 equation and critique M-10 |
| `SB-GEO-T27` | Select a method whose readable primary equation is missing | Method-specific unavailable result naming the missing publication; no binary/raster fallback | Dossier §6 named-paper gaps; CONTRACT §§2.1–2.2 |
| `SB-GEO-T28` | Raw Pp `3000 psi`, normal pressure `4500 psi`, both limits Off | Final Pp remains `3000 psi`; clamp count `0` | Dossier §§3.6,5.2 default-Off decision |
| `SB-GEO-T29` | Same input with hydrostatic floor On | Final Pp `4500 psi`; clamp count `1`, lower-bound mask true | Dossier §5.3 V-11; direct max operation |
| `SB-GEO-T30` | Pressure `5194.8 psi` at `10000 ft` | Gradient `10 ppg`, tolerance `1e-12`; pressure remains a separate curve | Dossier §5.1 and `0.051948` conversion |
| `SB-GEO-T31` | `Sv=10000`, `Pp=5000`, `alpha=1`, `K=0.5`, `sigma_tec=T0=0` | Fracture pressure `7500 psi`, tolerance `1e-12` | Dossier §§2.3,5.1 generalized equation |
| `SB-GEO-T32` | Same except `alpha=0.8` | Fracture pressure `7000 psi`, tolerance `1e-12` | `geomech-ppfg.md` §§2.3,5.1 generalized equation; direct substitution |
| `SB-GEO-T33` | Select constant-K, then switch to Daines without required lithology values | First uses only constant-K; second refuses, with no fallback | Dossier §4 K-relationship separation |
| `SB-GEO-T34` | Load a candidate Matthews–Kelly primary fit that decreases inside its declared range | Coefficient-set load refusal | Dossier §5.3 V-9 monotonicity gate |
| `SB-GEO-T35` | Matthews–Kelly run with mean `Sv/TVD=0.93 psi/ft` and a sourced tolerance smaller than `0.07` | Premise warning, recorded departure `-0.07 psi/ft`, bias field present | Dossier §§3.4,5.3 V-9b |
| `SB-GEO-T36` | Correlation source scope does not contain the run's declared scope | Refusal/warning exactly per registered policy; breached scope persisted | Dossier §5.3 V-9c |
| `SB-GEO-T37` | Piecewise fixture `f1(z)=z`, `f2(z)=z+1`, breakpoint `5000`; evaluate `4999` and `5000` | `4999` and `5001` under the declared half-open branch rule | Dossier §2.3 piecewise-breakpoint structure; direct substitution |
| `SB-GEO-T38` | Load conflicting breakpoint choices without selecting one | Refusal naming both sourced alternatives | Dossier §§3.2,5.2 unresolved breakpoint |
| `SB-GEO-T39` | Load the primary-derived Daines schema | Exactly `30` value rows across `10` families; every value cites Daines (1982) | Dossier §2.3 and critique m-4 |
| `SB-GEO-T40` | `Sv=10000 psi`, `Pp=5000 psi`; calculate published bounds | Lower `6666.6667 psi`, upper `7500 psi`, tolerance `1e-4`; labelled envelope | Dossier §6.21b equations; direct substitution |
| `SB-GEO-T41` | Raw FP `11000 psi`, Sv `10000 psi`; ceiling Off then On | Off: `11000`; On: `10000` and clamp count `1` | Dossier §§3.6,5.2 |
| `SB-GEO-T42` | `Sv=10000`, `Pp=5000`, `alpha=1`, `nu=0.25`, both strains zero | `Shmin=6666.6667 psi`, tolerance `1e-4` | Dossier §2.4 poroelastic equation; direct substitution |
| `SB-GEO-T43` | Swap distinct `eps_hmin` and `eps_Hmax` in an otherwise symmetric fixture | The two tectonic increments exchange according to the printed directional equations | Dossier §2.4 horizontal-stress forms |
| `SB-GEO-T44` | Supply only `eps_x` and `eps_y` without a declared mapping frame | Refusal naming both required directional strains | Dossier J-D13 resolution |
| `SB-GEO-T45` | Consume existing `YME=2.880`, `PR=0.2354` from the documented dynamic fixture | Values retain `dynamic`, `Mpsi`, and source-run tags | `src-tauri/src/unconventional.rs:641-646`; dossier §2.4 seam |
| `SB-GEO-T46` | Existing dynamic fixture `DT=100`, `DTS=130`, `RHOB=2.5` | `YME`, `PR` and downstream static use are null | `src-tauri/src/unconventional.rs:659-664` |
| `SB-GEO-T47` | Valid dynamic moduli with `YME_D2S` unset | Static property and stress calculation refuse | Dossier §§2.4,5.2 no-identity decision |
| `SB-GEO-T48` | Convert `1 Mpsi` to psi and GPa | `1000000 psi` and `6.894757 GPa`, tolerance `1e-6` | Exact SI conversions; dossier §2.5 typed-unit requirement |
| `SB-GEO-T49` | Correlation declares `1 MPa` output | `145.038 psi`, tolerance `1e-12` | Dossier §5.2 `MPa_to_psi` |
| `SB-GEO-T50` | Correlation declares `1 bar` output | `14.5038 psi`, tolerance `1e-12` | Dossier §5.2 `bar_to_psi` |
| `SB-GEO-T51` | Stress `5194.8 psi` at `10000 ft` | Separate gradient `0.51948 psi/ft` and `10 ppg`, tolerance `1e-12` | Dossier §§2.5,5.1 conversions |
| `SB-GEO-T52` | Diagonal total-stress tensor `[100,80,60] psi`, zero shear, identity orientation | Principal stresses `[100,80,60] psi` with original axes | Dossier §4 six-component principal transform; linear algebra identity |
| `SB-GEO-T53` | Same tensor expressed under negative-in-compression input convention | Canonical principal stresses still `[100,80,60] psi`; sign conversion recorded | Dossier §4 explicit sign-convention requirement |
| `SB-GEO-T54` | Mark tensor inputs `effective` and request the total-stress transform | Refusal before pore-pressure correction | Dossier §4 total-not-effective assertion |
| `SB-GEO-T55` | Inclined solve omits thermal/depletion/damage choices | Refusal naming three unset optional-term policies | Dossier §4 incumbent inclined-path omissions |
| `SB-GEO-T56` | Enable a criterion with only a vendor raster citation | Registration refusal naming missing public equation and parameter source | Dossier §§3.8,6; CONTRACT §2.1 |
| `SB-GEO-T57` | Query enabled failure-criterion registry after all public sources are supplied | Seven distinct criterion ids, including two Drucker–Prager forms | Dossier §4 optimal union |
| `SB-GEO-T58` | Drucker–Prager fixture `J1=0`, `k=2`, `alpha_dp=0`, target `sqrt(J2)=2` | Converged root residual `0`, tolerance `1e-12`; bracket and iteration count present | Dossier §4 invariant equation; direct substitution |
| `SB-GEO-T59` | Principal-stress direction at `44°`, `45°`, `46°` to borehole axis | Classification changes only at the exposed `45°` boundary under documented inclusivity | Dossier §§1.2,4 shear-mode rule |
| `SB-GEO-T60` | Stability inputs with `UCS=99 psi`, all others inside cited ranges | Refusal naming `WBS_UCS_RANGE 100–10000 psi`; value is not clamped | Dossier §5.2 QC ranges |
| `SB-GEO-T61` | A psi-native UCS correlation emits `100 psi` | Final value `100 psi`, not `14503.8 psi` | Dossier §5.3 V-13 native-unit flag |
| `SB-GEO-T62` | Back-transform vector `(x=0,y=-1)` to azimuth | `270°` under the declared frame, tolerance `1e-12`, and round-trip recovers vector | Dossier §5.3 V-16 / rule 2 |
| `SB-GEO-T63` | Registry contains one empty source and one required ABSENT parameter | Registration refuses the first; run refuses the second | Dossier §5.3 V-17; CONTRACT §2 |
| `SB-GEO-T64` | Parameter ordinal points to `B` while semantic key says `A` | Load error naming ordinal and key | Dossier §5.3 V-18 |
| `SB-GEO-T65` | Attempt to mark a locally calibrated fit as the global default | Refusal; fit remains versioned named preset | Dossier §6.22 local-calibration boundary |
| `SB-GEO-T66` | Miller synthetic density at `2001 ft` below mudline | Null plus range-breach record | Dossier §5.2 `MILLER_ZMAX=2000 ft` and V-10 |
| `SB-GEO-T67` | Supply an opaque vendor coefficient artifact to a disabled method | Artifact rejected; method remains disabled | CONTRACT §2.2 prohibited reconstruction path; dossier §6.17–20 |
| `SB-GEO-T68` | Import `PP`, compute `PP`, then edit interpreted `PP` | Three identities remain retrievable; no overwrite | Dossier §5 adoption provenance discipline |
| `SB-GEO-T69` | Export a run with one clamp and one synthetic-density sample | Export contains both masks/counts plus all §48 provenance fields | Dossier §§5.3–5.4 rules 9–14 |
| `SB-GEO-T70` | Two modules attempt private Biot values `1.0` and `0.8` in one run | Composition refusal naming the shared-value conflict | Dossier §§2.2,3.7 shared-parameter defect class |
| `SB-GEO-T71` | Execute every numeric example in this chapter through the released calculators | Stored outputs match each expected value at its stated tolerance | Dossier §5.3 V-19 and §5.4 rule 11 |
| `SB-GEO-T72` | Raw `[1,100,1]`, filter Off then `window=3`; inspect operation log | Off preserves raw values; On records an odd 3-sample filter as a separate ordered step | Dossier critique m-15; no filter equation is inferred |
| `SB-GEO-T73` | One acquisition-dependent method lacks its paper while Eaton inputs are complete | Missing method refuses with named gap; Eaton remains runnable | Dossier §6 method-specific acquisition list |

## 7. Open items, escalations and refusals

### 7.1 Open items

1. Confirm the published breakpoint and coefficients for the two piecewise effective-Poisson
   relationships; current manifest and printed sources disagree.
2. Determine whether a fresh-install UI resolves Traugott `Cpg` to `1000` or `5600`.
3. Determine whether a fresh-install UI resolves Archie `OC` to `5.2` or `1000`, and confirm that
   `Rw100` is intentionally blank.
4. Resolve the Chapman lower-range limit (`1` versus `1000`) from a primary method source.
5. Resolve the Miller pore-pressure nesting and the Alberty NSC bracket/subscripts from primary
   publications.
6. Obtain primary provenance and validity ranges for every adopted wellbore-failure criterion.
7. Decide whether Stassi d'Alia serves a distinct product need after a readable primary source is
   acquired; it is inventory only today.
8. Source the Hubbert–Willis bounds from the 1957 paper before moving the T4 envelope to P0.
9. Define a product policy and cited tolerance for the Matthews–Kelly `1 psi/ft` premise gate.
10. Read the remaining named-model and sensitivity pages identified in dossier §6.11.

### 7.2 Escalations

1. Acquire Alberty and McLean (2003/2005) for the velocity-NCT coefficients and exponent.
2. Acquire Eberhart-Phillips, Han and Zoback (1989) for the `1460`/`868` placement.
3. Acquire Matthews and Kelly (1967) for provenance-clean polynomial coefficients.
4. Acquire Daines (1982), SPE-9254-PA, for the lithology Poisson values.
5. Acquire Bowers (1995) for independent coefficient provenance and unloading calibration range.
6. Acquire Katahara (2003) before implementing the modified-Eaton form.
7. Acquire the Traugott primary publications for both density families and `Cpg`.
8. Acquire Eaton and Eaton (1997) to settle the breakpoint/coefficient findings.
9. Acquire public primary equations for the complete failure-criterion set, including Modified
   Wiebols-Cook and Hoek-Brown parameterization.
10. Perform the three live-UI checks in §7.1 without inferring algorithms from outputs.

### 7.3 Refusals

- SandiBumi refuses the claim that Bowers unloading collapses at `U=0`; it enforces `U=1` because
  the incumbent's own equation reduces to loading there (`geomech-ppfg.md` §2.2, T3).
- SandiBumi refuses method-dependent hidden pressure clamps; it uses one default-Off, counted policy
  because the shipped asymmetry is an incumbent defect (dossier §3.6, T1–T3).
- SandiBumi refuses the declining Matthews–Kelly quadratic; it loads only a primary-derived fit
  that remains non-decreasing over its declared range (dossier §§3.4,5.3 V-9, T1–T3).
- SandiBumi refuses to label pressure as gradient; it emits separately typed `FP`/`FG` and `Pp`/
  `PPG` pairs (dossier J-D5/J-D10, T2).
- SandiBumi refuses identity dynamic-to-static conversion; missing calibration blocks static stress
  because both incumbent identity defaults are physically unsupported (dossier §2.4, T1–T3).
- SandiBumi refuses universal MPa/bar multipliers; each correlation declares its native unit because
  the incumbent families include psi-, MPa- and bar-native outputs (dossier §5.3 V-13/V-14, T2).
- SandiBumi refuses unbounded regional extrapolation; it returns null with a breached-contract record
  because several incumbent correlations explicitly disallow transfer (dossier §§3.4,6.21, T2–T4).
- SandiBumi refuses hardcoded legacy trend interpolation with fixed pressure offsets; it uses sourced
  published methods and explicit trends because the executable legacy path embeds inconsistent
  constants (`geomech-ppfg.md` §§2.2,4, T1).
- SandiBumi refuses the vendor breakout effective-stress sign until it is independently derived from
  Kirsch plus a public failure criterion; the held raster is unresolved (dossier J-D11/J-O-12, T2–T4).
- SandiBumi refuses silently dropping thermal, depletion or damage terms in an inclined solve; every
  zero term is an explicit attributed choice (dossier §4, T3).

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirement range | Evidence owned |
|---|---|
| `SB-GEO-001–008` | Dossier §§1,2.1,3.1,3.7,4; vertical-stress and density inventory |
| `SB-GEO-009–017` | Dossier §§2.2,3.3,3.5–3.6,5.1; normal pressure, Eaton, Bowers and limits |
| `SB-GEO-018–026` | Dossier §§2.3,3.2,3.4,4,5.1; fracture equation, K methods, bounds and gates |
| `SB-GEO-027–035` | Dossier §§2.4–2.5,4; elastic/static properties, stress and inclined transform |
| `SB-GEO-036–041` | Dossier §§3.8,4,5.2–5.4; stability criteria, units, modes and angles |
| `SB-GEO-042–052` | Dossier §§5–6 and CONTRACT §§2–2.2; provenance, refusal and acquisition gates |

### 8.2 Inventory, canonical-form and optimal-choice disposition

| Dossier item | Disposition in this chapter |
|---|---|
| §1.1 first incumbent inventory | Capability union represented; binary/raster-only methods gated by SB-GEO-015/046 |
| §1.2 second incumbent inventory | Auto density, alpha-aware pressure/stress and inclined-frame strengths adopted with explicit units |
| §1.3 executable legacy inventory | Vertical-stress anchor adopted; hardcoded pressure interpolation refused |
| §1.4 delivered-study evidence | Retained as evidence that calibration is necessary; all asset-specific presets excluded |
| §2.1 overburden equations | Canonical integration, density masks and sourced correlation parameters in SB-GEO-003–008 |
| §2.2 pore-pressure equations | Terzaghi, four Eaton forms and Bowers loading/unloading in SB-GEO-009–016 |
| §2.3 fracture equations | Generalized FP, separate FG, K-method boundary and bounds in SB-GEO-017–026 |
| §2.4 horizontal stress | Directional poroelastic/strain requirements in SB-GEO-027–030 |
| §2.5 units | Typed depth, pressure, gradient, modulus and strength in SB-GEO-002/031/032/040 |
| §3 differences | Every material divergence maps to an ABSENT value, selectable source, gate or defect refusal |
| §4 optimal choices | Adopted except where a primary source is missing; omissions are named in §7.2 |
| §5 adoption spec | Canonical forms, parameters and all V-series tests compiled into §§4–6 |
| §5.4 findings rules | `atan2`, typed units, dual addressing, source strings, executable examples and fail-loud policy adopted |

### 8.3 Prior discrepancy-ledger disposition

| Item | Chapter disposition |
|---|---|
| `J-D1` | No Geertsma form copied; primary derivation remains open under SB-GEO-052 |
| `J-D2` | Mpsi made a type in SB-GEO-031; T48 guards the `10^6` boundary |
| `J-D3` | Tectonic-strain Young's modulus typed; GPa input conversion explicit |
| `J-D4` | Self-referential output documentation rejected by SB-GEO-032/050 |
| `J-D5` | Stress and stress-gradient identities separated by SB-GEO-032 |
| `J-D6` | Conflicting water constants yield `RHO_WATER` ABSENT and SB-GEO-008/049 |
| `J-D7` | Bar-native lineage retained through SB-GEO-040 and T50 |
| `J-D8` | Correlation-native conversion required; no universal replacement constant |
| `J-D9` | Opposite-sensitivity forms remain separately sourced; unresolved form blocked |
| `J-D10` | Fracture pressure and gradient separated by SB-GEO-017 |
| `J-D11` | Breakout sign remains refused pending independent public derivation |
| `J-D12` | Four-way normal-gradient spread yields `GRAD_NORMAL` ABSENT |
| `J-D13` | Directional strain names fixed by SB-GEO-028 |
| `J-D14` | Eberhart-Phillips constants ABSENT pending the named 1989 paper |
| `J-D15` | Anisotropic slowness names require canonical semantic identity under SB-GEO-043 |
| `J-D16` | Copied thermo/poroelastic wording prevented by executable documentation SB-GEO-050 |
| `J-D17` | One canonical dynamic-Poisson identity required by SB-GEO-029/043 |
| `J-O-1` | TTI solve unavailable pending readable public derivation |
| `J-O-2` | `K0=0.8` recorded; unresolved bracket remains acquisition-gated |
| `J-O-3` | Parenthesization remains open; no implementation from corroboration alone |
| `J-O-4` | NCT coefficient set ABSENT; named papers escalated |
| `J-O-5` | Readable and closed K families are both non-adoption sources; only public derivation may ship |
| `J-O-6` | Hoek-Brown defaults remain ABSENT pending public source/live confirmation |
| `J-O-7` | Matthews–Kelly form inventoried; coefficient rows not transcribed into the chapter |
| `J-O-8` | 30-row/10-family structure retained; values must come from Daines (1982) |
| `J-O-9` | Gardner `0.23/0.25` carried with two-source agreement |
| `J-O-10` | Closed Traugott coefficients unavailable; paper is the only route |
| `J-O-11` | Same Eberhart-Phillips block as `J-D14` |
| `J-O-12` | Same breakout sign block as `J-D11` |
| `J-O-13` | Miller parameter semantics partly retained; unresolved nesting remains open |

### 8.4 New discrepancy and gap disposition

| Item | Chapter disposition |
|---|---|
| `X-1` | `TRAUGOTT_CPG` ABSENT; live check and primary paper escalated |
| `X-2` | Second Poisson breakpoint ABSENT; primary 1997 paper escalated; `0.173 ppg` stake tested by choice discipline |
| `X-3` | Conflicting shallow coefficient not adopted; entire set primary-sourced |
| `X-4` | Smectite/illite Eaton exponent ABSENT |
| `X-5` | Default-On lower clamp refused; uniform default-Off policy adopted |
| `X-5b` | Missing clamp on the drilling-exponent path closed by the same uniform policy |
| `X-6` | Default-On fracture ceiling refused; counted default-Off policy adopted |
| `X-7` | `ANSC_K0=0.8` carried with manifest-only provenance and explicit visibility |
| `X-8` | Correlation-specific density disagreement remains visible; no cross-method fill |
| `X-9` | Eberhart-Phillips range divergence remains acquisition-gated |
| `X-10` | Undocumented K/tensile/tectonic values not promoted to house defaults |
| `X-11` | Bowers A/B ABSENT; calibration required regardless of vendor path |
| `X-12` | Turning quadratic refused by monotonicity load gate |
| `X-13` | Closed in favour of `U=1` from the incumbent equation, guarded by T25/T26 |
| `X-14` | Metric/imperial sibling conflict closed by typed units and T07/T48–T51 |
| `X-15` | Hardcoded legacy constants not adopted |
| `X-16` | Mudline-density unit ambiguity neutralized by typed input and identity fixture |
| `X-17` | Archie `OC` ABSENT |
| `X-17b` | Archie `Rw100` ABSENT; printed value remains reference evidence only |
| `X-18` | Two mudline-density representations retain separate provenance; product value uses dossier-adopted identity |
| `X-19` | Matthews–Kelly independent variable defined as depth below anchor; contradictory prose refused |
| `X-20` | Chapman range conflict remains an open primary-source question |
| `X-21` | Traugott fluid selection remains unresolved; no inferred mudline density path |
| Gap `1–8b` | Named primary-publication acquisitions listed individually in §7.2 |
| Gap `9–13` | Retired/deeper incumbent-document reads dispositioned in §§7.1–7.2 without raster transcription |
| Gap `14–16c` | Live checks narrowed to parameter display/mapping; no behavioural inference authorized |
| Gap `17–20` | Permanent reverse-engineering boundaries enforced by SB-GEO-007/015/046 |
| Gap `21` | Regional recalibration and declared-scope gates enforced by SB-GEO-006/022/044 |
| Gap `21b` | Published bounds adopted as an envelope; raster-only extra criterion remains open |
| Gap `22` | All asset-specific preset values excluded; local calibration remains versioned and non-default |

### 8.5 Dossier parameter-row disposition

| Dossier §5.2 row group | Disposition |
|---|---|
| Integration and unit constants | Carried as typed constants; differences tested rather than averaged |
| Water density and normal gradient | Both ABSENT; one documented reference preset retained with source |
| Limit switches | Replaced by one uniform default-Off, counted policy for pore pressure and fracture pressure |
| Exponential/Gardner density | Readable constants carried; conversion identity and applicability gate required |
| Miller synthetic density | Six parameters and `2000 ft` ceiling carried; alternate vendor values not promoted |
| Barker–Wood | Formula constants, water assumption and all three validity conditions carried |
| Alberty NSC | Readable values including manifest-only `K0` carried and exposed |
| Traugott power law | `Cpg` ABSENT; other unresolved closed coefficients not copied |
| Alberty/McLean and transformation temperatures | Readable density forms/endpoints carried; NCT coefficient set ABSENT |
| Eaton exponents/factor | Four corroborated method values carried; special disputed exponent ABSENT |
| Bowers | `Vml` carried; `A`, `B`, `U` and maximum effective stress require calibration and are ABSENT |
| Chapman/Miller pressure trends | Readable point parameters carried; Chapman range conflict remains open |
| Eberhart-Phillips | Entire dependent constant set ABSENT pending primary paper |
| Dxc | Readable trend parameters carried with unpublished-provenance warning |
| Archie/semi-log | `m`, `a`, `phi0`, exponent and trend endpoints carried; `Rw100` and `OC` ABSENT |
| Piecewise Poisson relationships | Breakpoint A carried; breakpoint B ABSENT; coefficients non-adoptable pending primary paper |
| Matthews–Kelly | Depth/range/premise carried; coefficients non-adoptable and premise tolerance ABSENT |
| Daines | Table shape retained; vendor values non-adoptable pending 1982 paper |
| Fixed K, tensile and tectonic stress | All physical run values ABSENT; vendor references not made defaults |
| Biot and tectonic strains | `alpha=1.0` carried; both strains ABSENT pending calibration |
| Static transforms, stress ratio and friction | All ABSENT; asset-specific precedents excluded |
| Dynamic modulus | `1.34747e4` Mpsi form and Mpsi type carried |
| Frame, sign and shear-mode angle | Explicit frame, positive-compression convention and `45°` threshold carried |
| Stability QC ranges | All nine printed range fields carried as validation, not clamping defaults |
| Output filter | Default `No` and odd window `3` carried; no filter equation inferred |

### 8.6 Critique disposition

| Finding | Chapter disposition |
|---|---|
| `B-1` | Criterion inventory corrected to include both Drucker–Prager forms and Mogi-Coulomb; chapter claims union/provenance advantage, not nonexistence |
| `B-2` | Invented Traugott mudline density excluded; fluid choice and `Cpg` remain unresolved/ABSENT |
| `M-1` | Piecewise polynomials are not described as crossing; breakpoint remains a source choice, not a root inference |
| `M-2` | Closed/readable K-family distinction retained, but no vendor coefficient set is an adoption source |
| `M-3` | Archie `OC` and blank `Rw100` both represented as ABSENT and separately tested |
| `M-4` | Two-sided incumbent limiting captured; product policy is uniform and default Off |
| `M-5` | Documented `8.66 ppg` reference retained without becoming the house normal-gradient default |
| `M-6` | Exact `8.345404` conversion used by the mudline-density identity; second representation retained as a discrepancy |
| `M-7` | `1 psi/ft` premise and declared source geography are both load-bearing gates |
| `M-8` | Eaton and Eaton (1997) appears explicitly in the acquisition list |
| `M-9` | All local identifiers and delivered-study names are excluded from the chapter; only generic calibration obligations remain |
| `M-10` | False source-independence claim not repeated; `U=1` rests on the incumbent's own algebra and a regression test |
| `m-1` | `0.173 ppg` stake used consistently |
| `m-2` | Eight carried water constants/six distinct equivalents described correctly |
| `m-3` | No mixed-provenance polynomial fixture is transcribed; each coefficient set is source-atomic |
| `m-4` | Daines structure stated as 30 rows across 10 families |
| `m-5` | Matthews–Kelly depth argument explicitly defined; contradictory prose rejected |
| `m-6` | Sayers/Traugott/density-combiner findings retained in inventory; raster-only equation blocks implementation |
| `m-7` | Eaton factor and exponents sourced to the corrected cross-tool evidence |
| `m-8` | In-file GPa-to-psi transition made an explicit typed conversion requirement |
| `m-9` | Closed Traugott parameter route recorded; named paper is the only allowed path |
| `m-10` | No single-asset pressure range is generalized into a house method or preset |
| `m-11` | Hubbert–Willis envelope and Stassi d'Alia inventory status both represented |
| `m-12` | Barker–Wood shallow `FG=OBG` assumption carried with numeric validity limits |
| `m-13` | Vendor table contents not transcribed; the recorded exception remains evidence-only and sets no implementation precedent |
| `m-14` | Chapman range divergence remains explicit and open |
| `m-15` | Trend outputs, flags, filter default/window and dual-indicator agreement are required by SB-GEO-012/051 |

### 8.7 Completeness statement

The chapter dispositions cover the full §1 inventory, §§2.1–2.5 canonical forms, every J-D/J-O
item, `X-1` through `X-21` including lettered sub-items, all §6 gap groups, every §5.2 parameter
row and all 27 critique findings. No value from a prohibited vendor lookup table is transcribed.
No Tier-C item was found. Asset-specific delivered-study values were deliberately excluded rather
than converted into product defaults.
