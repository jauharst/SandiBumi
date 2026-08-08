# 26. Production logging and cased-hole integrity — requirements

| | |
|---|---|
| Evidence dossier | `docs/research_2026-08/cross_tool/production-logging.md` — 2,965 lines |
| Critique applied | `docs/research_2026-08/cross_tool/production-logging_critique.md` — 601 lines; 26/26 findings dispositioned |
| Evidence held | T1 readable manifests/specifications, T2 extracted manuals, T3 directly read help/catalogues, T4 capability claims |
| Source audit | `src-tauri/src/modules.rs`, `db.rs`, `ingest.rs`, `lib.rs`; `src/ipc.ts` |
| Authored | 2026-08-08 |
| Requirements | 48 (`SB-PLG-001`…`SB-PLG-048`) |
| P0 requirements | 24 |

## 1. Scope and boundary

This chapter owns three independently shippable units: `cement_eval`, `casing_integrity` and
`prodlog`. `prodlog` owns spinner calibration through apparent fluid velocity, station/time-depth
reduction, array-tool geometry, temperature-flow estimates, inflow differentiation, selective
inflow performance and production holdup. It stops before multiphase rate computation until the
missing mixture-velocity, slippage and gas-regime equations are sourced. `cement_eval` owns scalar
and array cement measurements, probability/confidence, channel and microdebond products, collar
exclusion and isolation reporting. `casing_integrity` owns typed wall-loss quantities, ovality,
burst pressure, survey merging, preprocessing and per-joint reporting.

**`21_data-io.md` (`DIO`) — ingest and storage.** DIO owns LAS/DLIS/ASCII readers, array-log storage,
log sets, null handling and export. PLG defines the production-specific time references, units,
array geometry, curve identities and validation required after generic ingest.

**`20_envcorr-qc.md` (`ENV`) — generic conditioning.** ENV owns generic despiking and correction
infrastructure. PLG owns the three non-equivalent casing-inspection despike definitions, their
objects, order, default posture and modification flags.

**`22_database-model.md` (`DBM`) — durable identity.** DBM owns the physical schema. PLG requires
definition-bearing names, per-tool array axes and provenance records; it does not introduce a
second ungoverned curve store.

**`23_plotting-interactivity.md` (`PLT`) — presentation.** PLT owns rendering and linked interaction.
PLG owns gas/oil/water semantics, cement/casing classifications, collar masks and the persisted
method identity behind every plotted result.

**`12_saturation.md` (`SAT`) — behind-casing saturation.** Pulsed-neutron saturation is SAT work.
The RST three-phase method remains here only where it produces production holdup; a saturation
input is a typed optional input, not permission to duplicate saturation interpretation.

**`27_ip-install-blockers.md` (`INS`) — executable distribution.** INS owns packaging. PLG owns no
installer or licensed-vendor runtime dependency; all adopted calculations must remain local and
reproducible.

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 Flow evidence stops at apparent velocity

One manual prints the spinner-to-apparent-velocity equation, zoned positive/negative regressions,
midpoint interpolation and pass weighting [T2]. Another confirms that calibration works on
zone-averaged spinner/cable-speed pairs but exposes no equations [T3]. The held corpus does not
contain the mixture-velocity transform, slippage correlations or gas-regime map needed to reach
phase rates. A plausible continuation would be silent fabrication, so PLG ships the evidenced
`Vapp` chain and refuses rate computation.

The apparent-velocity catalogue itself is internally inconsistent: `ft/s` ranges conflict with
their `m/min` alternatives by about 91×, while another sibling declares `ft/min`. Spinner speed is
`rps` in one tool and `c/s` in another with no counts-per-revolution factor. Units therefore decide
whether a run is legal; no fixed 60× vendor conversion is inferred.

### 2.2 Array identity is geometric, not ordinal

Two readable array-tool specifications use different diameter units, different spinner/probe
counts, different clockwise conventions and different gas/water offsets from their spinners [T1].
Index-aligning sensor families can join measurements from different places in the pipe. Geometry is
stored per sensor family and per tool, then transformed into one declared reference frame.

### 2.3 Time-to-depth reduction has a two-tool invariant

The full Chronolog workflow uses days since 1900-01-01 or a literal `UnixTime` curve in seconds
since 1970-01-01, discriminates before averaging, exposes Mean/Median/Earliest/Latest and carries a
sensor length per curve [T2]. A second tool independently specifies median station reduction [T3].
Median is therefore the sourced default, but the epoch, estimator, nulls and sensor offset remain
explicit run metadata.

### 2.4 Cement methods differ enough to flip isolation

The strongest scalar-CBL equation held is the logarithmic attenuation ratio [T3]. At bonded/free/
measured amplitudes of 3/60/17 mV it gives 0.42098, matching the printed 0.42; a linear-amplitude
rescale gives 0.75439. On 70/10/30 mV endpoints the alternatives give 0.43543 and 0.66667, crossing
a 60% pass line. Interpolation identity cannot be hidden behind one `BPI` name.

Array coverage is `100 × passing valid elements / valid elements`; one suite uses both 72- and
360-element arrays [T1]. A fixed denominator causes a 5× error. Amplitude improves downward while
attenuation and impedance improve upward. A parameter set whose endpoint order contradicts its
measurement family is rejected at load.

### 2.5 Probability and confidence remain separate

The readable probability architecture multiplies casing-bonding, no-channeling and formation
probabilities, averages their confidences, and multiplies the two results for a cement index [T3].
Isolation applies the accepted probability and accepted confidence thresholds separately over an
interval. Its worked examples reproduce 0.21 and 0.66667. A single service can therefore cap at
about 0.667 while the accepted-probability preset is 0.8; the UI must not compare unlike scalars or
hide why a result cannot cross a line.

### 2.6 Cement cutoffs are scoped contracts

The effective prepared-workflow ultrasonic adequate threshold is 2.7 MRayl; 1.5 is the fallback
seen before preparation [T1/T3]. The gas boundary of 0.3 MRayl is supported only for the ultrasonic
tool class, while a similarly named 3.460207 value serves a different crossplot partition. A
derivative cutoff of 1 MRayl sits above the same vendor suite's 0–0.6 display scale, and the channel
manifest labels its 50% threshold with a direction contrary to its algorithm description. These
values carry scope and warnings, never universal status.

### 2.7 Collar samples are excluded, not erased

All three tools exclude collar-affected samples from cement/casing statistics while retaining or
interpolating through them [T1–T3]. The 10 m collar window is a jump-ahead search stride, not a
smoothing width; the 0.5 m influence is hardware-dependent collar length. Deleting flagged samples
destroys the audit trail and changes later depth alignment.

### 2.8 “Metal loss” hides three different measurements

The evidence distinguishes radius-over-thickness penetration, normalized cross-sectional area
loss, absolute area loss and direct thickness loss [T1–T3]. Under fixed OD,
`area_loss/penetration = (IRmeas + IRnom)/(ORnom + IRnom) ≤ 1`; the normalized forms converge at
full penetration and have their largest absolute gap at 50% penetration. A generic `MLOSS` output
is therefore prohibited.

Negative apparent loss is meaningful QC evidence for scale, buildup or sub-tolerance noise. One
reporting scheme explicitly spans −100% to 100%. Values remain signed and flagged, never clamped.

### 2.9 Ovality, grading and preprocessing need named definitions

One ovality convention is zero for a round pipe, another ellipticity convention is one, and a
third tool's indexed-radius expression cannot be interpreted because its radii are undefined
[T1–T3]. Condition grades also apply to different quantities: 12.5% grades thickness loss while
12% grades penetration. Their numerical proximity is not definition-level agreement.

The tools also ship three despike operations on different objects: four-neighbour array detection,
minimum-thickness scalar detection and bad-azimuth patching. Default posture differs. They remain
separate stages, default off in PLG's conservative recipe, with every changed sample flagged.

### 2.10 Burst pressure is only as sound as nominal geometry

Barlow's equation is printed, but one tool's 25,000 psi default and another's N-80 grade imply
2,142.9 and 6,857.1 psi on the same geometry—a 3.2× difference [T1/T3]. Yield strength and nominal
ID must come from a cited casing-grade/geometry source; neither ships as a house scalar. A KLBF
input and LBF output in one module add a separate 1,000× unit trap.

## 3. SandiBumi as-built

### 3.1 Domain computation

`ABSENT` — the deterministic registry lists 51 manifests but no spinner, production-rate, cement,
casing, collar or Chronolog module (`src-tauri/src/modules.rs:434-486`). The dispatcher contains no
branch for any of those capabilities (`src-tauri/src/modules.rs:507-574`). The source-wide targeted
search found no alternate domain implementation.

### 3.2 Generic scalar and array carriage

`PARTIAL` — generic LAS import preserves arbitrary curves, canonicalizes recognized families and
writes curve metadata (`src-tauri/src/ingest.rs:350-405`). The database has an `array_logs` store
with one optional per-row axis blob (`src-tauri/src/db.rs:260-289`), and the IPC decoder exposes
depth, values and width (`src/ipc.ts:2886-2921`). It does not carry the per-tool/per-family geometry,
clockwise convention, epoch, cement-family polarity or casing-definition metadata required here.

### 3.3 Production-specific import and reporting

`ABSENT` — the IPC command surface exposes generic curve/array reads but no station importer,
Chronolog transform, PL tool schema, cement/casing report or production-specific validation
(`src-tauri/src/lib.rs:689-716`; `src/ipc.ts:2886-2921`). Generic storage is not an interpretation.

## 4. Requirements

#### SB-PLG-001 — Ship three independently gated domain units [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST gate and version `cement_eval`, `casing_integrity` and `prodlog`
independently; absence of one MUST NOT weaken validation in another.

**Rationale.** The dossier §5 adoption boundary reflects radically different evidence completeness.

**As-built.** ABSENT — no domain entry exists in `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T01, SB-PLG-T02

#### SB-PLG-002 — Type every production unit at ingest [P0] [status: ABSENT]

**Requirement.** Every spinner, velocity, rate, pressure, temperature, casing and cement curve MUST
carry a declared unit; inconsistent or absent required units MUST refuse computation.

**Rationale.** The corpus contains live 91×, 1,000× and 3.280839895× traps.

**As-built.** ABSENT — generic metadata exists, but no PLG unit gate (`src-tauri/src/ingest.rs:350-405`).

**Verified by.** SB-PLG-T03, SB-PLG-T04, SB-PLG-T05

#### SB-PLG-003 — Calibrate spinner slopes from zonal averages [P0] [status: ABSENT]

**Requirement.** The calibration MUST regress zone-averaged spinner/cable pairs separately by spin
sign, support opposite-slope inheritance for a one-point sign, place slopes at zone midpoints, hold
them flat outside the calibrated range and interpolate between midpoints.

**Rationale.** This is the complete two-tool-confirmed calibration contract in dossier §2.1.

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T06, SB-PLG-T07, SB-PLG-T08

#### SB-PLG-004 — Compute apparent fluid velocity exactly [P0] [status: ABSENT]

**Requirement.** For each pass SandiBumi MUST compute
`Vapp = spinner_rps/slope_rps_per_ft_min − cable_speed_ft_min + threshold_ft_min`, using an in-situ
threshold whose sign is consistent with the calibrated branch.

**Rationale.** This is the last complete flow equation in the held corpus [T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:507-574`.

**Verified by.** SB-PLG-T09, SB-PLG-T10

#### SB-PLG-005 — Normalize multi-pass weights [P1] [status: ABSENT]

**Requirement.** Combined apparent velocity MUST equal `Σ(wᵢVappᵢ)/Σwᵢ`; weights MUST be explicit,
non-negative and normalized rather than assumed to sum to one.

**Rationale.** The manual prints weighted combination but no universal pass-weight policy.

**As-built.** ABSENT — `src-tauri/src/modules.rs:507-574`.

**Verified by.** SB-PLG-T11

#### SB-PLG-006 — Stop before unsupported phase rates [P0] [status: ABSENT]

**Requirement.** `prodlog` MUST refuse mixture velocity, slippage, gas-regime and phase-rate output
until a selected published method and complete equations are held. It MUST NOT claim incumbent
compatibility from option names or charts.

**Rationale.** Dossier E-3/E-4 and R-12 mark the numerical chain after `Vapp` unavailable.

**As-built.** ABSENT — there is no PLG path to guard (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T12

#### SB-PLG-007 — Store sensor geometry per family and tool [P0] [status: PARTIAL]

**Requirement.** Every array-tool definition MUST carry diameter unit, clockwise convention,
reference azimuth and per-family sensor angle/radius offsets. Spinner, gas and water arrays MUST NOT
be index-aligned without geometric transformation.

**Rationale.** Readable specifications place these sensor families at different coordinates [T1].

**As-built.** PARTIAL — arrays store values and an axis, not this geometry (`src-tauri/src/db.rs:272-289`).

**Verified by.** SB-PLG-T13, SB-PLG-T14

#### SB-PLG-008 — Use an explicit three-phase holdup schema [P2] [status: ABSENT]

**Requirement.** If RST three-phase holdup is implemented, its 24 outputs MUST be defined by an
explicit schema, including asymmetric uncertainty names, optional water-saturation mode and
mandatory carbon-density input; suffix inference is prohibited.

**Rationale.** The vendor table is internally asymmetric and includes outputs omitted by summaries.

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T15, SB-PLG-T16

#### SB-PLG-009 — Keep temperature-flow assumptions visible [P2] [status: ABSENT]

**Requirement.** Temperature-derived flow MUST expose conductivity loss, oil expansion and friction
heating separately and stamp the geometry/rate validity guidance on the result.

**Rationale.** One malformed dialog joins quantities that the prose defines separately [T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T17

#### SB-PLG-010 — Enforce selective-inflow data sufficiency [P1] [status: ABSENT]

**Requirement.** SIP MUST require at least three flowing rates plus shut-in; a shut-in crossflow
point MUST be accepted only when the input represents observed crossflow.

**Rationale.** The manual states these minimums and admissibility conditions [T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T18

#### SB-PLG-011 — Differentiate cumulative inflow with a declared length [P2] [status: ABSENT]

**Requirement.** Inflow differentiation MUST report `rb/d/ft` and persist the selected depth
length; 7 ft is the sourced preset and 5 ft remains an explicit alternative.

**Rationale.** Output magnitude scales directly with the window [T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T19

#### SB-PLG-012 — Make Chronolog epochs and operation order explicit [P0] [status: ABSENT]

**Requirement.** Chronolog MUST support fractional days since 1900-01-01 and seconds since
1970-01-01 only with the literal `UnixTime` reference, MUST discriminate before averaging and MUST
persist estimator and per-curve sensor length.

**Rationale.** Epoch or operation-order ambiguity shifts every result while preserving a smooth log.

**As-built.** ABSENT — no time/station command exists (`src/ipc.ts:2886-2921`).

**Verified by.** SB-PLG-T20, SB-PLG-T21, SB-PLG-T22

#### SB-PLG-013 — Restrict station import to evidenced grammars [P1] [status: ABSENT]

**Requirement.** The production station importer MUST accept only the evidenced ASCII/LAS forms,
exact required labels and dot-decimal numeric syntax; unsupported proprietary formats MUST refuse.

**Rationale.** The held loader contract is complete only for those forms [T2].

**As-built.** ABSENT — generic LAS import has no station grammar (`src-tauri/src/ingest.rs:350-405`).

**Verified by.** SB-PLG-T23

#### SB-PLG-014 — Normalize nulls before station reduction [P0] [status: PARTIAL]

**Requirement.** Declared nulls MUST be honored first; −999.00, −999.25 and −9999 families MUST be
flagged as suspected undeclared nulls and excluded before station aggregation.

**Rationale.** The three sentinels occur in the held PL/cased-hole workflows [T2/T3].

**As-built.** PARTIAL — generic LAS sanitization exists, but not the domain sentinel audit
(`src-tauri/src/ingest.rs:365-382`).

**Verified by.** SB-PLG-T24

#### SB-PLG-015 — Preserve phase semantics [P1] [status: ABSENT]

**Requirement.** Production holdup and shading MUST map gas to red, oil to green and water to blue;
each holdup MUST remain bounded to `[0,1] V/V` with missing values distinct from zero.

**Rationale.** This is a three-tool invariant [T1–T3].

**As-built.** ABSENT — no production family schema exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T25

#### SB-PLG-016 — Bind cutoff polarity to measurement family [P0] [status: ABSENT]

**Requirement.** Amplitude MUST pass downward (`value ≤ cutoff`); attenuation and impedance MUST
pass upward (`value ≥ cutoff`). Endpoint ordering contrary to the family MUST be a load error.

**Rationale.** Reversing this rule can grade free pipe as good cement.

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T26, SB-PLG-T27

#### SB-PLG-017 — Implement logarithmic attenuation bond index [P0] [status: ABSENT]

**Requirement.** The default attenuation method MUST compute
`ln(free/measured)/ln(free/full)` and clamp the result to `[0,1]` after validating positive,
distinct endpoints.

**Rationale.** It reproduces the held worked example and follows attenuation physics [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T28, SB-PLG-T29

#### SB-PLG-018 — Name and require the bond interpolation method [P0] [status: ABSENT]

**Requirement.** A run MUST identify `attenuation` or `amplitude` interpolation; imported ambiguous
`BPI` values MUST remain uninterpreted until their definition is declared.

**Rationale.** The two methods differ by 23 points and flip a 60% classification.

**As-built.** ABSENT — no cement schema exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T30

#### SB-PLG-019 — Derive coverage from valid array width [P0] [status: PARTIAL]

**Requirement.** Coverage MUST equal `100 × pass_count/valid_count` at each depth. Width MUST be
read from the data; invalid samples MUST leave the denominator and fixed-width assumptions MUST be
rejected.

**Rationale.** The same suite contains 72- and 360-element arrays [T1].

**As-built.** PARTIAL — width is decoded, but no coverage computation exists (`src/ipc.ts:2902-2921`).

**Verified by.** SB-PLG-T31, SB-PLG-T32

#### SB-PLG-020 — Exclude collars without deleting data [P0] [status: ABSENT]

**Requirement.** Collar-flagged samples MUST be excluded from statistics and retained in every
source/output curve with a retrievable mask.

**Rationale.** This is the second three-tool invariant and preserves auditability.

**As-built.** ABSENT — no collar mask exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T33

#### SB-PLG-021 — Compute slurry acoustic impedance in declared units [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST use `rho_gcc=rho_lbgal×0.1198264`, `v_ms=25400/dt_usin` and
`Z_MRayl=rho_gcc×v_ms/1000`, retaining full precision.

**Rationale.** Two printed rows expose a real +0.135% vendor divergence [T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T34, SB-PLG-T35

#### SB-PLG-022 — Keep expected-CBL correlation optional and attributed [P2] [status: ABSENT]

**Requirement.** The cited expected-CBL correlation MAY initialize a threshold only when its full
coefficient set, input units and provenance caveat travel with the result; it MUST NOT be a house
default.

**Rationale.** The formula is readable, but its primary provenance is not stated [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T36

#### SB-PLG-023 — Keep probability and confidence separate [P1] [status: ABSENT]

**Requirement.** Cement placement MUST compute `Pgood=Pb×Pno_channel×Pformation`, confidence as the
arithmetic mean of term confidences, and `cement_index=Pgood×confidence`; isolation MUST test the
probability and confidence thresholds separately over the selected interval.

**Rationale.** Both equations and worked examples are held [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T37, SB-PLG-T38

#### SB-PLG-024 — Validate probability-term switches [P0] [status: ABSENT]

**Requirement.** `use_bond_index` and `use_channeling_index` MUST be explicit. Channeling for
impedance, attenuation or SLG MUST refuse when bonding is disabled. Disabled-factor and confidence-
denominator policies MUST remain absent until sourced.

**Rationale.** The dependency is printed; disabled-term arithmetic is not [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T39

#### SB-PLG-025 — Explain the single-service ceiling [P1] [status: ABSENT]

**Requirement.** A single-service result MUST display its reachable maximum and MUST NOT compare
`cement_index` directly with a probability-only threshold.

**Rationale.** The worked full-bond case caps at 0.66667 against a 0.8 probability preset.

**As-built.** ABSENT — no cement UI exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T40

#### SB-PLG-026 — Implement channel detection with an explicit direction warning [P1] [status: ABSENT]

**Requirement.** Channel detection MUST apply a 2D circumference/depth window, exclude collars,
require mean coverage `≤ threshold` and channel fraction `≥ channel_threshold`, and record that the
vendor field label contradicts the latter direction.

**Rationale.** The algorithm description and parameter comment disagree [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T41

#### SB-PLG-027 — Separate derivative, smoothing and vertical statistics [P1] [status: ABSENT]

**Requirement.** Foamed-cement processing MUST use an explicitly selected depth-difference
estimator, absolute value, optional x-direction smoothing and a separately sized vertical-standard-
deviation statistic. It MUST NOT use the vertical window as a smoothing dimension.

**Rationale.** The prior conflation reproduces parameter names but not the method [T1/T2].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T42, SB-PLG-T43

#### SB-PLG-028 — Preserve four-direction microdebond evidence [P2] [status: ABSENT]

**Requirement.** Microdebond MUST compute standard deviations over vertical, horizontal and two
diagonal neighborhoods of width `2×half_window+1`, then average the four; its interpretation MUST
retain the all-directions/any-direction distinction.

**Rationale.** This is the complete readable vendor contract [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T44

#### SB-PLG-029 — Keep cement classifications distinct [P1] [status: ABSENT]

**Requirement.** Bond score, cement-bond quality, ultrasonic coverage and radial-bond coverage MUST
remain separate named classifications; no colormap MAY be reused as an equation.

**Rationale.** The suite ships different three- and four-band schemes for different quantities [T1].

**As-built.** ABSENT — no cement classification exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T45

#### SB-PLG-030 — Enforce isolation-report interval length [P1] [status: ABSENT]

**Requirement.** Bond-quality reporting MUST declare its minimum sustained interval and MUST NOT
promote isolated samples to an interval classification.

**Rationale.** A readable report manifest ships a 5 m minimum [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T46

#### SB-PLG-031 — Make waveform extraction reproducible [P2] [status: ABSENT]

**Requirement.** CBL waveform extraction MUST persist peak window, expected transit time, gain and
delay; the transit-time pick MUST be the highest peak within the selected window.

**Rationale.** These settings are printed and materially affect amplitude [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T47

#### SB-PLG-032 — Emit four named casing-loss quantities [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST emit `penetration_pct`, `area_loss_pct`, `area_loss_abs_in2` and
`thickness_loss_pct` by their canonical equations; it MUST NOT emit `metal_loss` or `MLOSS`.

**Rationale.** One bare name covers non-equivalent definitions [T1–T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T48, SB-PLG-T49

#### SB-PLG-033 — Retain signed apparent loss [P0] [status: ABSENT]

**Requirement.** Loss quantities MUST remain signed on `[-100,100]%`; negative values MUST be
flagged for scale/buildup/noise and MUST NOT be clamped to zero.

**Rationale.** The reporting specification explicitly includes negative loss [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T50

#### SB-PLG-034 — Make prior-survey merge explicit [P1] [status: ABSENT]

**Requirement.** A prior penetration survey MUST combine only through one recorded method:
`merge_input_only`, `maximum`, `minimum` or `average`; no method ships by default.

**Rationale.** The four modes are printed, but the vendor does not identify a default [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T51

#### SB-PLG-035 — Require an ovality definition [P0] [status: ABSENT]

**Requirement.** PLG MUST expose zero-based ovality and one-based ellipticity as separate outputs.
An imported `OVALITY` curve without a declared definition MUST remain uninterpreted.

**Rationale.** A round pipe is 0 under one convention and 1 under another.

**As-built.** ABSENT — no casing family schema exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T52, SB-PLG-T53

#### SB-PLG-036 — Compute Barlow only from sourced strength [P0] [status: ABSENT]

**Requirement.** Burst pressure MUST equal `2St/(Df)` with declared units. Yield strength MUST be
provided or derived from a versioned grade table; no scalar strength default may ship.

**Rationale.** Competing presets change burst pressure by 3.2×.

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T54, SB-PLG-T55

#### SB-PLG-037 — Source nominal casing geometry [P0] [status: ABSENT]

**Requirement.** Nominal OD, ID and wall thickness MUST come from declared measurements or a cited
public-standard dataset; OD/weight lookup MUST refuse when that dataset is unavailable.

**Rationale.** No held vendor source provides the required database.

**As-built.** ABSENT — no casing database exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T56

#### SB-PLG-038 — Bind grades to their measurement quantity [P1] [status: ABSENT]

**Requirement.** Condition and report bands MUST declare whether they grade thickness loss,
penetration or another quantity. The 12% and 12.5% schemes MUST NOT be merged as equivalent.

**Rationale.** Similar numbers apply to different definitions [T1/T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T57

#### SB-PLG-039 — Keep three despike stages distinct and auditable [P1] [status: ABSENT]

**Requirement.** Array-neighbor detection, minimum-thickness scalar detection and bad-azimuth
patching MUST be separately selectable, default off in the house recipe, and flag every modified
sample/frame.

**Rationale.** They operate on different objects and have opposing vendor defaults [T1–T3].

**As-built.** ABSENT — the generic conditioner is not casing-aware (`src-tauri/src/modules.rs:434-486`).

**Verified by.** SB-PLG-T58

#### SB-PLG-040 — Preserve named correction recipes [P1] [status: ABSENT]

**Requirement.** PLG MUST persist correction order. It MUST support the evidenced recipes without
silently reordering their gain/offset, calibration, rotation, patching, centralization,
normalization, drift or derotation stages.

**Rationale.** Non-commuting corrections produce different radii [T2/T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T59

#### SB-PLG-041 — Distinguish one- and two-depth calibration [P1] [status: ABSENT]

**Requirement.** One-depth calibration MUST solve offset only; two-depth calibration MUST solve gain
and offset. Applied coefficients and depths MUST be stored per finger.

**Rationale.** The pipe-evaluation help prints this distinction [T3].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T60

#### SB-PLG-042 — Refuse untracked environmental correction [P0] [status: ABSENT]

**Requirement.** Each radius/depth environmental gain or offset MUST carry its source; inputs
declared already corrected MUST not be corrected again.

**Rationale.** Double correction is a silent geometry error [T3].

**As-built.** ABSENT — no casing correction path exists (`src-tauri/src/modules.rs:434-574`).

**Verified by.** SB-PLG-T61

#### SB-PLG-043 — Canonicalize casing weight and tension [P0] [status: ABSENT]

**Requirement.** `lbf/ft`, `lb/ft` and `lbm/ft` casing-weight labels MUST map to one declared
mass-per-length quantity at import. KLBF and LBF tension MUST never pool without an exact 1,000×
conversion.

**Rationale.** Both traps occur in held vendor schemas [T1/T3].

**As-built.** ABSENT — no production/casing family map exists (`src-tauri/src/ingest.rs:384-400`).

**Verified by.** SB-PLG-T62, SB-PLG-T63

#### SB-PLG-044 — Detect collars with correct window semantics [P1] [status: ABSENT]

**Requirement.** CCL detection MUST normalize to `[-1,1]`, apply the selected cutoff, jump ahead by
the collar-separation window after a pick, and derive influence length from casing hardware.

**Rationale.** Treating the 10 m value as smoothing changes the algorithm [T1].

**As-built.** ABSENT — `src-tauri/src/modules.rs:434-574`.

**Verified by.** SB-PLG-T64

#### SB-PLG-045 — Stamp full run provenance [P0] [status: PARTIAL]

**Requirement.** Every PLG output MUST carry module version, equation/method identity, parameter
sources, units, inputs, masks, correction order and warnings.

**Rationale.** Most domain failures remain numerically plausible.

**As-built.** PARTIAL — log sets support module/parameter/input provenance, but no PLG payload exists
(`src-tauri/src/db.rs:312-328`).

**Verified by.** SB-PLG-T65

#### SB-PLG-046 — Separate computed, imported and interpreted identities [P0] [status: ABSENT]

**Requirement.** Imported ambiguous `BPI`, `OVALITY`, `MLOSS` or velocity curves MUST remain raw;
only definition-bearing, unit-validated products MAY feed thresholds or reports.

**Rationale.** Bare vendor family names are not portable definitions.

**As-built.** ABSENT — generic curve metadata does not enforce these identities
(`src-tauri/src/ingest.rs:384-400`).

**Verified by.** SB-PLG-T66

#### SB-PLG-047 — Export machine-readable reports with masks [P2] [status: ABSENT]

**Requirement.** Cement and casing reports MUST export the underlying numeric curves, units,
classification definition, interval aggregation, collar/QC masks and provenance—not a raster-only
traffic light.

**Rationale.** The dossier's defect catalogue prohibits raster-only truth.

**As-built.** ABSENT — no domain report command exists (`src-tauri/src/lib.rs:3251-3429`).

**Verified by.** SB-PLG-T67

#### SB-PLG-048 — Preserve array width and per-row validity end to end [P0] [status: PARTIAL]

**Requirement.** Array-log import, compute, display and export MUST preserve width, axis and `NaN`
validity without padding invalid values into coverage statistics.

**Rationale.** Coverage and sectorization depend on the actual valid population.

**As-built.** PARTIAL — IPC preserves width and `NaN`, but PLG compute/export is absent
(`src/ipc.ts:2886-2921`).

**Verified by.** SB-PLG-T31, SB-PLG-T68

## 5. Parameters

`ABSENT — ships with no default` is a deliberate product value: the run must receive a sourced
value or explicit choice. Vendor chart/lookup contents are not transcribed. A preset is available
only when its scope and source travel with the run; it is not a universal physical constant.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Apparent-velocity unit | `U_VAPP` | `ft/min` | enum | Dossier §§2.1,5.1 C-13 | T2 |
| Spinner-speed unit | `U_SPIN` | `rps` | enum | Same | T2 |
| Spinner slope unit | `U_SLOPE` | `rps/(ft/min)` | enum | Same | T2 |
| Spinner counts per revolution | `SPIN_CPR` | **ABSENT — ships with no default** | counts/rev | Dossier OPEN-T6; `c/s`↔`rps` factor unstated | T3 |
| Spinner discriminator | `SPIN_DISC` | **ABSENT — ships with no default** | rps | Dossier M-D-1: ±0.1 prose versus ±0.50 dialog | T2 |
| Spinner threshold | `SPIN_THRESHOLD` | **ABSENT — in-situ pick; ships with no default** | ft/min | Dossier §§2.1,5.2 | T2 |
| Typical threshold evidence | `SPIN_THRESHOLD_HINT` | `+3 / −4` | ft/min | Dossier §5.2; tool-dependent evidence only | T2 |
| Quick-look coefficient | `QUICKLOOK_C` | `0.900` | dimensionless | Dossier §5.2; two-version agreement | T2 |
| Pass weights | `PASS_WEIGHTS` | **ABSENT — ships with no default** | non-negative vector | Dossier C-13 requires explicit normalized weights | T2 |
| Array deviation weighting | `DEV_WEIGHT_TABLE` | **ABSENT — user table; ships with no default** | angle→weight | Dossier M-OPEN-3; endpoints 1 vertical, 0 horizontal | T2 |
| Mixture-velocity method | `VMIX_METHOD` | **ABSENT — not implementable from held corpus** | method id | Dossier R-12 / M-OPEN-6 / E-4 | T2 |
| Slippage correlation | `SLIP_METHOD` | **ABSENT — not implementable from held corpus** | method id | Dossier R-12 / M-OPEN-2 / E-4 | T2 |
| Gas-regime map | `GAS_REGIME` | **ABSENT — not implementable from held corpus** | method id | Dossier R-12 / M-OPEN-1 / E-4 | T2 |
| Calculation zone above reservoir | `CALC_ZONE_ABOVE` | `15 ft` or `5 m` | depth | Dossier §5.2; two-version agreement | T2 |
| Rate sensitivity guidance | `RATE_SENS` | `100` at 9 5/8 in; `50` at 7 in | rb/d per ft/min | Dossier §5.2; guidance, not conversion | T2 |
| Inflow differentiation length | `DIFF_LEN` | `7` (`5` alternative) | ft | Dossier §5.2 | T2 |
| Temperature conductivity loss | `TEMP_COND_LOSS` | `0.25` | °F/ft at 1 bl/d | Dossier §5.2 | T2 |
| Oil expansion | `OIL_EXPANSION` | `1` | °F/1000 ft | Same | T2 |
| Friction heating guidance | `FRIC_HEAT` | `1` | °F/1000 psi | Same | T2 |
| Friction-negligible rate | `FRIC_NEGLIGIBLE` | `10000` in 7 in casing | rb/d | Same | T2 |
| Slug/churn deviation factor | `SLUG_CHURN_DEV` | `0.5` | dimensionless | Dossier M-D-9; raster-sourced, exposed | T2 |
| Deviation trigger | `DEV_TRIGGER` | `5`, mean over top 25% of interval | degrees | Dossier §5.2 | T2 |
| Slippage reversal | `SLIP_REVERSAL` | `90`, ramp length `3` | degrees | Same | T2 |
| Relative roughness | `REL_ROUGH` | `0.0006`; scaled-casing evidence `0.002` | dimensionless | Same | T2 |
| Tool-housing friction multiplier | `PTS_FRIC_MULT` | `1.00`; older grooved evidence `1.4–1.5` | dimensionless | Same | T2 |
| Maximum slippage ratio | `MAX_VSLIP_RATIO` | `2.0×Vmix`; true downflow `1.0×Vmix` | ratio | Same; deferred with slippage method | T2 |
| Deviated oil/water anchor | `EW_ANCHOR` | `0.6` | fraction | Same; deferred with slippage method | T2 |
| SIP flowing-rate minimum | `SIP_N_FLOW` | `3` plus one shut-in | count | Dossier §2.2 | T2 |
| SIP shut-in admissibility | `SIP_SHUTIN` | observed crossflow only | enum | Same | T2 |
| Holdup mnemonic set | `HOLDUP_NAMES` | `WFYG / WFYO / WFYW` | schema | Dossier §5.2 | T1 |
| Holdup range | `HOLDUP_RANGE` | `[0,1]` | V/V | Same | T1 |
| Phase colours | `PHASE_COLOURS` | gas red / oil green / water blue | semantic enum | Dossier D-O; three-tool agreement | T1–T3 |
| Array diameter unit | `ARRAY_DIAM_UNIT` | **ABSENT — required per tool** | `in` or `mm` | Dossier §2.2; readable tool specs disagree | T1 |
| Array clockwise convention | `ARRAY_CLOCKWISE` | **ABSENT — required per tool** | boolean | Same | T1 |
| Sensor angle offsets | `SENSOR_AZ` | **ABSENT — required per family/tool** | degrees | Same | T1 |
| Sensor radius offsets | `SENSOR_RADIUS` | **ABSENT — required per family/tool** | declared tool unit | Same | T1 |
| Chronolog day epoch | `TIME_EPOCH_DAY` | `1900-01-01` | epoch | Dossier §2.7 | T2 |
| Chronolog Unix epoch | `TIME_EPOCH_UNIX` | `1970-01-01` | epoch | Same | T2 |
| Unix reference mnemonic | `UNIX_MNEMONIC` | `UnixTime` exactly | string | Same | T2 |
| Station estimator | `STATION_EST` | `Median` (`Mean/Earliest/Latest` alternatives) | enum | Dossier §2.7; two-tool agreement | T2/T3 |
| Operation order | `STATION_ORDER` | discriminate, then average | enum | Dossier §2.7 | T2 |
| Sensor length | `SENSOR_LENGTH` | **ABSENT — ships with no default per curve** | depth | Same | T2 |
| Station null candidates | `STATION_NULLS` | `−999.00`, `−999.25` | numeric sentinel | Dossier §5.2 | T2 |
| Dot-decimal syntax | `DECIMAL_MARK` | `.` | character | Dossier §2.7 | T2 |
| Gas-behind-pipe impedance | `Z_GAS_ULTRA` | `0.3` | MRayl | Dossier D-R/§5.2; ultrasonic tool class only | T1/T2 |
| House coverage cutoff | `COV_CUTOFF` | **ABSENT — ships with no default** | % | Dossier D-H: vendor presets 85 and 60 disagree | T1/T2 |
| Incumbent coverage presets | `COV_PRESETS` | `85` and `60` | % | Dossier §5.2; selectable provenance-bearing presets | T1/T2 |
| Passing cutoff class | `USE_AS_PASS` | `adequate` | enum | Dossier C-2; two-tool agreement | T1/T2 |
| Scalar full-bond preset A | `CBL_FULL_A` | `10` | mV | Dossier §5.2 | T1 |
| Scalar free-pipe preset A | `CBL_FREE_A` | `70` | mV | Same | T1 |
| Scalar full-bond preset B | `CBL_FULL_B` | `0` | mV | Dossier §5.2 | T3 |
| Scalar free-pipe preset B | `CBL_FREE_B` | `100` | mV | Same | T3 |
| Bond interpolation | `BOND_INTERP` | **ABSENT — ships with no default** (`attenuation` / `amplitude`) | enum | Dossier D-G / OPEN-G2 | T2/T3 |
| Ultrasonic good impedance | `ZAI_GOOD` | `5` | MRayl | Dossier §5.2; two-tool agreement | T1/T3 |
| Ultrasonic adequate impedance | `ZAI_ADEQUATE` | `2.7` | MRayl | Dossier G-D-1 resolution; 1.5 is unprepared fallback | T1/T3 |
| Ultrasonic free-pipe impedance | `ZAI_FREE` | `0` | MRayl | Dossier §5.2 | T1 |
| Liquid acoustic impedance | `AI_LIQUID` | **ABSENT — ships with no default** | MRayl | Dossier T-D-5 / OPEN-T3; printed 70 rejected | T3 |
| Attenuation bonded/free endpoints | `ATT_ENDPOINTS` | **ABSENT — ships with no default** | dB/m | Dossier T-D-7 / OPEN-T3; printed 5/70 reverses polarity | T3 |
| Radial amplitude good | `RAM_GOOD` | `10`, or attributed expected CBL | mV | Dossier §5.2; unit from readable qualify spec | T1 |
| Radial amplitude adequate | `RAM_ADEQUATE` | `20` | mV | Same | T1 |
| Accepted probability | `P_ACCEPT` | `0.8` | dimensionless | Dossier §2.3.1 | T3 |
| Accepted confidence | `C_ACCEPT` | `0.8` | dimensionless | Same | T3 |
| Isolation interval | `ISO_INTERVAL` | `1` | m | Same | T3 |
| Formation term | `FORMATION_TERM` | probability `1`, confidence `0.5` | dimensionless | Same | T3 |
| Use bond index | `USE_BOND` | **ABSENT — ships with no default** | boolean | Dossier C-6; vendor default unstated | T3 |
| Use channeling index | `USE_CHANNEL` | **ABSENT — ships with no default** | boolean | Same | T3 |
| Disabled-factor policy | `DISABLED_FACTOR` | **ABSENT — ships with no default** | policy | Dossier OPEN-T8 | T3 |
| Confidence denominator policy | `CONF_DENOM` | **ABSENT — ships with no default** | policy | Dossier OPEN-T8 | T3 |
| Waveform peak window | `PEAK_WINDOW` | `80` | µs | Dossier §5.2 | T3 |
| Expected transit time | `TT_EXPECTED` | `200` | µs | Same | T3 |
| CBL gain | `CBL_GAIN` | `1` | dimensionless | Same | T3 |
| Waveform delay | `WF_DELAY` | `0` | µs | Same | T3 |
| Expected-CBL coefficients | `CBL_EXPECTED_COEFFS` | `a=0.369106050328276; b=0.297251565437175; c=−1.00578212129878; d=363.278958833953; e=0.712895639650241; f=−0.81649474492953; free=201.54; free exponent=−0.6044` | typed set | Dossier C-5; readable manifest, primary provenance unstated | T1 |
| Slurry-density conversion | `LB_GAL_TO_G_CC` | `0.1198264` | g/cm³ per lb/US gal | Dossier C-4 | T2 |
| Transit-time velocity numerator | `US_IN_TO_M_S` | `25400` | m·µs/(s·in) | Same | T2 |
| Derivative differencing scheme | `DERIV_D` | **ABSENT — ships with no default** | enum | Dossier M-OPEN-8 | T1/T2 |
| Derivative absolute form | `DERIV_ABS` | `true` | boolean | Dossier C-8 | T1 |
| Derivative smoothing | `DERIV_SMOOTH` | `true` | boolean | Same | T1 |
| Derivative smoothing width | `DERIV_WIDTH` | `5` | x-direction array elements | Same | T1 |
| Derivative smoothing method | `DERIV_METHOD` | `MEAN` | enum | Same | T1 |
| Vertical-statistic window | `DERIV_VERT_WINDOW` | `5` | depth frames | Same; not smoothing | T1 |
| Derivative cutoff | `DERIV_CUTOFF` | `1` | MRayl | Dossier §5.2; warning: display scale ends 0.6 | T1 |
| Derivative BPI cutoff | `DERIV_BPI_CUTOFF` | `60` | % | Same | T1 |
| Derivative array width | `DERIV_ARRAY_WIDTH` | **ABSENT — read from log** | elements | Dossier C-2/C-8; 72 and 360 both occur | T1 |
| Microdebond half-window | `MICRO_HALF_WINDOW` | `2` | samples | Dossier C-9 | T1 |
| Composite-debond ZAI threshold | `DEBOND_ZAI` | `0.3` | context-inferred MRayl | Dossier §5.2; manifest unit blank, composite-map scope | T1 |
| Composite-debond sigma threshold | `DEBOND_SIGMA` | `1` | context only | Same; manifest unit blank | T1 |
| Channel minimum width | `CHANNEL_WIDTH` | `5` | % circumference | Dossier C-7 | T1 |
| Channel minimum depth | `CHANNEL_DEPTH` | `2` | m | Same | T1 |
| Channel mean-coverage limit | `CHANNEL_MEAN_MAX` | `0.5` | fraction | Same | T1 |
| Channel-fraction threshold | `CHANNEL_FRAC_MIN` | `50` | % | Same; `≥` adopted with vendor-label warning | T1 |
| Bond-score bands | `BONDSCORE_BANDS` | `0–25 / 25–50 / 50–100` | % | Dossier §5.2 | T1 |
| Cement-bond bands | `CEMENT_BOND_BANDS` | `0–25 / 25–50 / 50–100` | % | Same; semantically distinct from bond score | T1 |
| Bond-quality minimum thickness | `BOND_REPORT_MIN` | `5` | m | Same | T1 |
| CCL cutoff | `CCL_CUTOFF` | `0.25` on `[-1,1]` | dimensionless | Dossier §5.2; starting point, interactively adjustable | T1 |
| Collar separation window | `COLLAR_WINDOW` | `10` | m | Same; jump-ahead stride | T1 |
| Collar influence | `COLLAR_INFLUENCE` | **ABSENT — derive from collar length** | m | Dossier §5.2; 0.5 is hardware-specific | T1 |
| Include collars in statistics | `INCLUDE_COLLARS` | `false` | boolean invariant | Dossier D-O; three-tool agreement | T1–T3 |
| Loss domain | `LOSS_RANGE` | `[-100,100]` | % | Dossier C-10/D-K | T1 |
| Condition bands | `CONDITION_BANDS` | `0–12.5 good / 12.5–20 moderate / 20–100 poor` | % thickness loss | Dossier §5.2; claimed standard not identified | T1 |
| Penetration corroboration | `PENE_NORMAL` | `12` | % penetration | Dossier D-K; not a thickness-loss band | T3 |
| Measurement accuracy | `MEAS_ACCURACY` | `2` | % | Dossier §5.2 | T1 |
| Reporting bands | `REPORT_BANDS` | `−100–2 / 2–5 / 5–15 / 15–30 / 30–100` | % signed thickness loss | Dossier D-K | T1 |
| Display bands | `DISPLAY_BANDS` | `0–10 / 10–20 / 20–30 / 30–40 / 40–50` | % | Dossier §5.2; display only | T1 |
| Casing grade | `CASING_GRADE` | **ABSENT — ships with no house default** | grade id | Dossier D-J; vendor N-80 preset is not adopted | T1 |
| Yield strength | `YIELD_STRENGTH` | **ABSENT — ships with no default** | psi | Dossier OPEN-T4 | T1/T3 |
| Nominal casing geometry | `CASING_GEOMETRY` | **ABSENT — ships with no default** | in | Dossier M-OPEN-7 / E-5 | primary needed |
| Burst safety factor | `BURST_SF` | `1.5` | dimensionless | Dossier §5.2 | T3 |
| Burst acceptable | `BURST_ACCEPT` | `0` | psi | Same | T3 |
| Eccentering limit | `ECC_LIMIT` | `10` | % | Same | T3 |
| Ovality limit | `OVALITY_LIMIT` | `10` | source family declares % | Same; source help omits unit | T3 |
| Eccentering method | `ECC_METHOD` | `Standard` | enum | Same; imported preset only | T3 |
| Eccenter smoothing | `ECC_FRAMES` | `3` | frames | Dossier §5.2 | T1 |
| Array despike threshold | `DESPIKE_ARRAY` | `5`, house default off | % from four-neighbor mean | Dossier D-N | T2 |
| Scalar despike threshold | `DESPIKE_SCALAR` | `1/6` of range within 3 frames, house default off | fraction | Dossier D-N; vendor ships on | T1 |
| Bad-azimuth patch rule | `PATCH_AZIMUTH` | **ABSENT — readable algorithm unavailable** | method id | Dossier E-10; compiled helper not decompiled | T1/T3 |
| Normalization threshold | `NORMALIZE_THRESHOLD` | `5` | % | Dossier §5.2 | T2 |
| Finger wear | `FINGER_WEAR` | `0` | µm/m | Dossier §5.2 | T3 |
| Log direction | `LOG_DIRECTION` | `Up` | enum | Same | T3 |
| Radius orientation | `RADIUS_VIEW` | `Inside view` | enum | Same | T3 |
| Penetration merge input | `PENE_MERGE` | **ABSENT — only when prior survey loaded** | % | Dossier C-10b | T3 |
| Merge method | `MERGE_METHOD` | **ABSENT — ships with no default** | enum | Same; four modes printed | T3 |
| Environmental correction | `ENV_CORRECTION` | **ABSENT — required source or already-corrected declaration** | typed surface | Dossier §5.2 | T3 |
| Histogram null | `HIST_NULL` | `−9999` | sentinel | Same | T3 |
| Per-finger identity gain/offset | `FINGER_GAIN_OFFSET` | `1 / 0` | dimensionless / radius | Same | T3 |
| RST porosity preset | `RST_PHI` | `0.2` | v/v | Dossier §2.5 | T3 |
| RST carbonate preset | `RST_CARB` | `0` | v/v | Same | T3 |
| RST borehole status | `RST_STATUS` | `Cased` | enum | Same | T3 |
| RST casing size | `RST_CASING_SIZE` | `7` | in | Same | T3 |
| RST casing weight | `RST_CASING_WEIGHT` | `32` | source declares lbf/ft; canonical mass/length | Same | T3 |
| RST bit size | `RST_BIT_SIZE` | `9` | in | Same | T3 |
| RST carbon density value | `RST_CDV` | **ABSENT — mandatory, ships with no default** | source method unit | Dossier §2.5 | T3 |

**Parameter count: 132. ABSENT count: 32.** Enumerations and measured inputs whose absence blocks a
run are counted as ABSENT. Deferred method identities and required per-tool geometry are included;
evidence-only presets are not counted as house defaults.

## 6. Acceptance tests

Every expected value below is either printed by the cited source or derived in the row from the
cited equation. Refusal tests have no numeric tolerance; numeric tolerances are explicit.

| ID | Input and operation | Expected value | Source of expected value |
|---|---|---|---|
| `SB-PLG-T01` | Register `cement_eval`, `casing_integrity`, `prodlog`; query capability gates | Three distinct versioned gates; exact-name match | Dossier §5 module boundary |
| `SB-PLG-T02` | Disable `prodlog`; run a valid cement fixture | Cement run remains available; no weakened checks | Dossier §5 independently shippable units |
| `SB-PLG-T03` | Load apparent velocity with no unit and a slope in `rps/(ft/min)` | Refusal before computation | Dossier D-C and §5.3 T2.4 |
| `SB-PLG-T04` | Convert `1 ft/min` to `ft/s`, and `1 m/min` to `ft/min` | `1/60 ft/s` and `3.280839895 ft/min`, tolerance `1e-9` | Dossier §5.3 T2.4; exact unit definitions |
| `SB-PLG-T05` | Pool `1 KLBF` with a curve declared `LBF` | Converted value `1000 LBF`, tolerance `1e-9`; unconverted pool refuses | Dossier D-M / §5.3 T2.9 |
| `SB-PLG-T06` | One zone with spinner/cable pairs `(2,10),(4,20)`; form calibration point | Zone average is `(3,15)`, tolerance `1e-12` | Dossier §§2.1,5.1 C-13 |
| `SB-PLG-T07` | A zone has two positive-spin points and one negative-spin point | Negative branch inherits the fitted positive slope | Dossier §2.1 / C-13 |
| `SB-PLG-T08` | Slope `0.2` at midpoint 1000 and `0.4` at 1100; evaluate 950,1050,1150 | `0.2`, `0.3`, `0.4`, tolerance `1e-12` | Dossier C-13 flat/interpolated contract |
| `SB-PLG-T09` | `spinner=10 rps`, `slope=0.2 rps/(ft/min)`, cable `20 ft/min`, threshold `3 ft/min` | `10/0.2−20+3 = 33 ft/min`, tolerance `1e-9` | Dossier §5.1 C-13 |
| `SB-PLG-T10` | Positive-slope branch with a negative in-situ intercept | Load refusal | Dossier §2.1 threshold sign rule |
| `SB-PLG-T11` | `Vapp=[10,20]`, weights `[1,3]` | `(1×10+3×20)/4 = 17.5`, tolerance `1e-12` | Dossier C-13 |
| `SB-PLG-T12` | Valid `Vapp` but no selected published `VMIX_METHOD` | `Vapp` emitted; Vmix/slippage/rates refused | Dossier R-12, M-OPEN-1/2/6, E-4 |
| `SB-PLG-T13` | Tool schema with four spinner positions and four gas probes rotated 20° | Geometry transform retains the 20° separation; ordinal join refuses | Dossier §2.2 readable array specifications |
| `SB-PLG-T14` | Radius offset `1 in` transformed to an `mm` reference frame | `25.4 mm`, tolerance `1e-12` | Exact inch definition; dossier §2.2 unit contract |
| `SB-PLG-T15` | Load the held RST output manifest | Exactly 24 declared outputs, including asymmetric uncertainty names; no suffix inference | Dossier §2.5 / critique m7 |
| `SB-PLG-T16` | Valid RST curves with carbon-density value absent | Run refuses and names the missing mandatory input | Dossier §2.5 / R-P9 |
| `SB-PLG-T17` | Temperature-flow setup with sourced presets | Persist `0.25 °F/ft @ 1 bl/d`, `1 °F/1000 ft`, `1 °F/1000 psi` as three fields | Dossier §5.2; two-version agreement |
| `SB-PLG-T18` | SIP with two flowing rates plus shut-in, then three flowing rates plus real crossflow shut-in | First refuses; second is admissible | Dossier §2.2 SIP contract |
| `SB-PLG-T19` | Cumulative flow rises `70 rb/d` over `7 ft` | Differentiated inflow `10 rb/d/ft`, tolerance `1e-12`; window stored as `7 ft` | Dossier §5.2 differentiation preset and definition |
| `SB-PLG-T20` | Chronolog day value `1.5` | `1900-01-02T12:00:00`, exact to one second | Dossier §2.7 epoch/fraction contract |
| `SB-PLG-T21` | Curve named exactly `UnixTime` with value `86400` | `1970-01-02T00:00:00`, exact to one second | Dossier §2.7 |
| `SB-PLG-T22` | Station values `[0,100]`, discriminator retaining values `≤10`, estimator median | Result `0`, not pre-discrimination mean `50` | Dossier §2.7 operation order |
| `SB-PLG-T23` | Station ASCII numeric token `1,25` | Refusal identifying dot-decimal requirement | Dossier §2.7 loader grammar |
| `SB-PLG-T24` | Station values `[1,−999.00,3,−9999]`, no declared null | Suspected nulls flagged/excluded; median `2`, tolerance `1e-12` | Dossier §5.3 T2.12 and station-null rows |
| `SB-PLG-T25` | Holdups gas/oil/water `[1,1,1]` | Terminal semantics red/green/blue and units `V/V`; values outside `[0,1]` refuse | Dossier D-O; three-tool agreement |
| `SB-PLG-T26` | Amplitude endpoints full `10`, free `70`, measured `60 mV` | Log index `ln(70/60)/ln(70/10)=0.07920`, tolerance `1e-5`; does not grade good | Dossier C-1 and §5.3 T2.1 |
| `SB-PLG-T27` | Amplitude family with full `70`, free `10`; impedance family with good `2`, adequate `5` | Both parameter sets refuse at load | Dossier C-3 / §5.3 T2.2 |
| `SB-PLG-T28` | Full `3`, free `60`, measured `17 mV` | Bond index `0.42098`, tolerance `1e-5`; printed comparator `0.42` | Dossier C-1; vendor worked example |
| `SB-PLG-T29` | Full `3`, free `60`, measured `3 mV` | Bond index `1.0` exactly | Same worked-example source |
| `SB-PLG-T30` | Full `10`, free `70`, measured `30 mV`; run both interpolation methods | Attenuation `0.43543`, amplitude `0.66667`, tolerance `1e-5`; 60% classification flips | Dossier D-G / §5.3 T3.3 |
| `SB-PLG-T31` | One array row `[6,4,3,NaN]`, upward cutoff `4` | `2/3×100 = 66.6667%`, tolerance `1e-4` | Dossier C-2 |
| `SB-PLG-T32` | Equivalent 50%-passing rows of width 72 and 360 | Both return `50%`, tolerance `1e-12` | Dossier C-2; readable 72/360 declarations |
| `SB-PLG-T33` | Five depths, centre depth collar-flagged, compute mean and export | Mean uses four unflagged values; all five samples and mask remain exported | Dossier D-O / §5.3 T2.11 |
| `SB-PLG-T34` | Density `10 lb/gal`, transit time `9 µs/in` | `3.3817673 MRayl`, tolerance `1e-7`; printed comparator `3.38` | Dossier C-4 / §5.3 T1.5 |
| `SB-PLG-T35` | Density `13 lb/gal`, transit time `7 µs/in` | `5.65238247 MRayl`, tolerance `1e-8`; record vendor `5.66` gap `+0.135%` | Dossier C-4 / §5.3 T1.6 |
| `SB-PLG-T36` | Fixed valid OD/ID; evaluate attributed expected-CBL formula at strengths `500` and `900 psi` | Higher strength produces lower amplitude; coefficient provenance warning retained | Dossier C-5; `e=0.712895639650241<1` |
| `SB-PLG-T37` | BI `0.42`, channel and formation probabilities `1`; confidences `0.5,0.5,0.5` | Cement index `0.21`, tolerance `1e-12` | Dossier C-6 vendor partial-bond example |
| `SB-PLG-T38` | Probabilities all `1`; confidences `1,0.5,0.5` | Cement index `0.6666667`, tolerance `1e-6`; printed comparator `0.66` | Dossier C-6 vendor full-bond example |
| `SB-PLG-T39` | Impedance service, channeling on, bonding off | Validation refusal before probability calculation | Dossier C-6 printed dependency |
| `SB-PLG-T40` | Single-service full-bond fixture with accepted probability `0.8` | UI shows reachable cement-index ceiling `0.667` and does not label `0.8` as its own threshold | Dossier T-D-4 / C-6 |
| `SB-PLG-T41` | Collar-free window mean coverage `0.4`, channel fraction `60%`, thresholds `0.5/50%`; repeat mean `0.6` | First is channel; second is not | Dossier C-7; description-direction adoption |
| `SB-PLG-T42` | Derivative smoothing width `5`, vertical-statistic window `9` | Smoothing spans five array elements only; vertical statistic spans nine depth frames | Dossier C-8 correction / critique MAJ-2 |
| `SB-PLG-T43` | Load derivative cutoff `1 MRayl` with display maximum `0.6` | Run may proceed with a persisted inconsistency warning; warning cannot be dismissed silently | Dossier OPEN-G4 |
| `SB-PLG-T44` | `half_window=2` on a valid image | Each directional statistic uses `2×2+1=5` neighbors; output is mean of four σ values | Dossier C-9 |
| `SB-PLG-T45` | Value `40%` classified by bond-score and cement-bond schemes | Outputs retain distinct identities even though both land in their middle band | Dossier readable qualify specifications |
| `SB-PLG-T46` | Passing isolation spans `4.9 m`, then `5.0 m` under the 5 m report preset | First not promoted; second qualifies, depth tolerance `1e-6 m` | Dossier bond-quality report manifest |
| `SB-PLG-T47` | Waveform window containing peaks `2,5,3 mV` | Transit-time pick is the location of `5 mV`; gain `1`, delay `0` preserve amplitude/time | Dossier §2.3 waveform help and presets |
| `SB-PLG-T48` | `IRnom=3.05`, `IRmeas=3.28`, `ORnom=3.50`, `tnom=0.45 in` | Penetration `51.11%`, area loss `49.39%`, absolute area loss `4.5738 in²`, tolerances `0.01%/1e-4 in²` | Dossier C-10 / §5.3 T1.7–T1.8 |
| `SB-PLG-T49` | Attempt to write the T48 result as `MLOSS` | Writer refuses; four canonical identities remain distinct | Dossier C-10 / §5.3 T2.7 |
| `SB-PLG-T50` | Direct thickness exceeds nominal enough to yield `−5%` loss | Output remains `−5%` and carries scale/buildup/noise flag; no clamp | Dossier D-K signed `−100…100%` evidence |
| `SB-PLG-T51` | Current penetration `20%`, prior `40%`; apply four merge modes, and input-only with current absent | Results `20,40,20,30,40%` for input-only/current, maximum, minimum, average, input-only/absent | Dossier C-10b printed mode definitions |
| `SB-PLG-T52` | `Dmax=6.2`, `Dmin=6.0`, `Dmean=6.1 in` | Zero-based ovality `3.2787%`; ellipticity `1.033333`, tolerance `1e-5` | Dossier C-11 / §5.3 T3.2 |
| `SB-PLG-T53` | Import a raw curve named `OVALITY` with no definition | Loads as uninterpreted; threshold application refuses | Dossier C-11 / §5.3 T2.6 |
| `SB-PLG-T54` | Barlow: `S=80000 psi`, `t=0.45 in`, `D=7 in`, `f=1.5` | `6857.1 psi`, tolerance `0.1 psi` | Dossier C-12 / §5.3 T1.9 |
| `SB-PLG-T55` | Repeat T54 with `S=25000 psi` | `2142.9 psi`, tolerance `0.1 psi`; ratio to T54 `3.2` | Dossier D-J / §5.3 T3.5 |
| `SB-PLG-T56` | Provide casing OD and weight but no cited geometry table or measured ID | Geometry-dependent loss/burst run refuses | Dossier M-OPEN-7 / E-5 |
| `SB-PLG-T57` | Apply 12% penetration and 12.5% thickness-loss bands to a direct-thickness case | Two separately named classifications; no merged “agreement” result | Dossier D-K / critique MAJ-5 |
| `SB-PLG-T58` | Enable array, scalar and bad-azimuth stages on fixtures with one defect of each kind | Each stage changes only its declared object and emits its own mask; default-off run changes none | Dossier D-N / critique MAJ-6 |
| `SB-PLG-T59` | Load the two evidenced correction recipes | Stored stage orders exactly match their selected recipe; an automatic reorder refuses | Dossier §2.4.4 correction-order evidence |
| `SB-PLG-T60` | One calibration depth, then two distinct depths | First fits offset only; second fits gain and offset, with both depths persisted | Dossier pipe-evaluation help |
| `SB-PLG-T61` | Input declares environmental correction already applied; request the same correction again | Refusal naming double-correction risk | Dossier critique MAJ-9 / pipe-evaluation help |
| `SB-PLG-T62` | Import equal casing-weight values labelled `lbf/ft`, `lb/ft`, `lbm/ft` | All map to one mass-per-length quantity with original spelling retained | Dossier D-L / §5.3 T2.8 |
| `SB-PLG-T63` | Import `2 KLBF` and `2000 LBF` tensions | Canonical values equal, tolerance `1e-12`; raw numeric pooling fails | Dossier D-M |
| `SB-PLG-T64` | Normalized CCL spikes at depths `0`, `1`, `11 m`; cutoff `0.25`, window `10 m` | Pick at `0`, skip search through `<10`, next eligible pick `11`; no smoothing occurs | Dossier collar manifest / critique m5 |
| `SB-PLG-T65` | Execute any valid PLG fixture and inspect provenance | Exact module/method/version, units, sources, inputs, masks, order and warnings present | CONTRACT §2 and dossier §5 defect rules |
| `SB-PLG-T66` | Import ambiguous raw `BPI`, `OVALITY`, `MLOSS`, unitless velocity; request a report | All remain viewable raw; none can feed computation until typed/defined | Dossier D-C/D-E/D-F/D-G |
| `SB-PLG-T67` | Export a cement/casing interval report | Numeric curves, units, classification definition, aggregation, masks and provenance present; raster is optional | Dossier FINDINGS §6.1 no raster-only truth |
| `SB-PLG-T68` | Round-trip a width-4 array row `[1,NaN,3,4]` through storage/display/export | Width remains `4`, second slot remains `NaN`, valid count remains `3`, exact bitwise `f32` round trip | `src-tauri/src/db.rs:260-289`; dossier C-2 validity contract |

## 7. Open items, escalations and refusals

### 7.1 Open items

- **O-1 — Disabled probability terms.** Whether a disabled factor becomes one and whether its
  confidence remains in the arithmetic-mean denominator are unstated. Settlement: readable method
  source or vendor confirmation (dossier OPEN-T8).
- **O-2 — Derivative estimator.** The absolute form and depth direction are evidenced, but two-point,
  least-squares, mean-absolute and RMS estimators remain live. Settlement: primary paper or readable
  implementation (M-OPEN-8).
- **O-3 — Channel comparison label.** The algorithm says `≥50%`; its parameter comment says
  “Maximum.” PLG adopts the physically consistent algorithm direction with a warning. Settlement:
  run the reference module or obtain vendor confirmation (OPEN-G5).
- **O-4 — Derivative cutoff/display mismatch.** A 1 MRayl cutoff lies above a 0–0.6 display scale.
  Settlement: determine whether the scale, unit or cutoff serves a different stage (OPEN-G4).
- **O-5 — Scalar-CBL interior form.** Log attenuation is the default; the other suite's stated
  endpoint scaling does not define its interior. Settlement: readable source or live probe (OPEN-G2).
- **O-6 — Casing grading attribution.** The 12.5/20 thickness-loss bands claim a standard but cite
  no document. Settlement: exact public standard and edition (OPEN-G3).
- **O-7 — Indexed-radius definitions.** The four radii in one “Standard ovality” expression are not
  defined. Settlement: diagram or live UI help; until then the definition is import-only (OPEN-T7).
- **O-8 — Strength-to-threshold coupling.** Whether edited cement strength re-queries the full
  response correlation or locally interpolates is not stated. Settlement: live reference run
  (M-OPEN-13).
- **O-9 — Radial normalization.** Additive versus multiplicative normalization is unresolved.
  Settlement: live reference run (M-OPEN-11).
- **O-10 — Casing report column identity.** “Minimum Thickness” versus “Minimum Loss %” is unresolved.
  Settlement: live reference run (M-OPEN-4).

### 7.2 Escalations

- **E-1 — Production-logging comparator corpus.** Ingest the market-leading PL interpretation
  manual/install tree before claiming competitive flow coverage.
- **E-2 — Licensed Geolog PL coverage.** Confirm whether a separate licence supplies computations;
  the held base tree contains geometry/readers but no flow source.
- **E-3 — Techlog flow equations.** Obtain the Production Logging user manual or live parameter help
  for spinner, holdup, slippage, inversion rates and SIP.
- **E-4 — IP PL User Manual.** Obtain the separate source for Vmix, slippage and gas-regime methods;
  until then phase-rate work remains a refusal.
- **E-5 — Nominal casing database.** Source an exact public-standard OD/weight/grade/ID dataset with
  edition and provenance.
- **E-6 — Three bounded live checks.** Resolve spinner discriminator, casing report column identity
  and radial normalization in a live reference session.
- **E-7 — Cement papers.** Obtain the three cited primary papers for microdebond and derivative
  methods before promoting vendor descriptions to independently sourced algorithms.
- **E-8 — Expected-CBL coefficient provenance.** Identify the paper or calibration population behind
  the readable coefficient set; until then it remains optional and attributed.
- **E-9 — Independent literature corpus.** Add primary PL, cement-evaluation and casing-integrity
  references; the local literature corpus has no substantive coverage.
- **E-10 — Bad-azimuth detector.** Obtain readable documentation or an independent published
  detector. The compiled helper is not to be decompiled.

### 7.3 Refusals

- Refuse a unitless velocity, spinner-speed, cement, casing or array-geometry input when a unit is
  required; never infer the catalogue's intended velocity unit.
- Refuse a `c/s` spinner curve without a sourced counts-per-revolution factor.
- Refuse Vmix, slippage, gas-regime and phase-rate output from the held corpus; option names and
  chart references are not equations.
- Refuse hard-coded cosine deviation weighting; only an explicit, validated user table is legal.
- Refuse vendor chart and lookup-table transcription, including cement strength-response charts and
  proprietary tool dimensions. Capability, axes, attribution and purpose may be cited.
- Refuse an endpoint set whose order contradicts its measurement family and any rule that grades
  free pipe as good cement.
- Refuse a fixed array denominator or sectorization incompatible with actual valid width.
- Refuse a bare `BPI`, `OVALITY`, `MLOSS` or `metal_loss` as a computed-input identity.
- Refuse deletion of collar samples; exclusion must use a reversible mask.
- Refuse silent adoption of the 1.5 MRayl unprepared fallback, the 70 MRayl liquid preset, the
  polarity-inverted attenuation pair or a universal 0.3 MRayl gas threshold.
- Refuse use of the optional expected-CBL correlation as an unattributed house method.
- Refuse channeling-on/bonding-off for impedance, attenuation or SLG services.
- Refuse clamping negative apparent casing loss to zero.
- Refuse a default penetration-merge mode, scalar yield strength or inferred nominal casing ID.
- Refuse reuse of one despike algorithm on a different object or any unflagged destructive edit.
- Refuse double environmental correction and unrecorded correction-order changes.
- Refuse decompilation or behavioural reconstruction of compiled vendor helpers.
- Refuse raster-only traffic lights or reports without their numeric curves, masks and provenance.

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirements | Dossier evidence |
|---|---|
| SB-PLG-001…006 | §§0,1.1–1.2,2.1–2.2,3 D-C/D-D/D-P/D-Q,4.3,5.1 C-13 |
| SB-PLG-007…015 | §§1.2/1.5,2.2/2.5/2.7/2.8,3 D-L/D-O,4.3 |
| SB-PLG-016…031 | §§1.1.1/1.3/1.4,2.3/2.6/2.9,3 D-A/D-B/D-G/D-H/D-I/D-R,4.1,5.1 C-1…C-9 |
| SB-PLG-032…044 | §§1.1/1.3/1.4,2.4/2.6,3 D-E/D-F/D-J/D-K/D-M/D-N,4.2,5.1 C-10…C-12 |
| SB-PLG-045…048 | §§2.8/2.10,4.0–4.3,5.4 and source audit of generic curve/array stores |

### 8.2 Inventory, canonical-form and optimal-choice disposition

| Dossier item | Disposition | Where it went |
|---|---|---|
| IP-1 — spinner calibration/apparent velocity | `ADOPTED` | SB-PLG-002…005 |
| IP-2 — multiphase flow calculation | `DEFERRED` after Vapp; missing equations | SB-PLG-006; E-4 |
| IP-3 — multiphase array flow | `ADOPTED` for geometry; flow math deferred | SB-PLG-007; E-4 |
| IP-4 — temperature-flow estimate | `ADOPTED` P2 | SB-PLG-009 |
| IP-5 — selective inflow performance | `ADOPTED` P1 | SB-PLG-010 |
| IP-6 — inflow curves | `ADOPTED` P2 | SB-PLG-011 |
| IP-7 — lateral average | `DEFERRED`; named estimator has no equation | O-2 class; dossier E-9 |
| IP-8 — cement evaluation | `ADOPTED` only where equations/typed contracts are held | SB-PLG-016…031 |
| IP-9 — casing inspection | `ADOPTED` with names and units corrected | SB-PLG-032…044 |
| IP-10 — Chronolog | `ADOPTED` | SB-PLG-012…014 |
| IP-11 — PL setup/curve grammar | `ADOPTED` as semantic schema, not internal positional identity | SB-PLG-013/046 |
| IP-12 — production loaders | `ADOPTED` only for evidenced ASCII/LAS forms | SB-PLG-013/014 |
| IP-13 — sensor plots/workflows/reports | `ADOPTED` at output-contract level | SB-PLG-047/048 |
| IP-14 — two empty shipped pages | `EVIDENCE-ONLY`; capability absence, no product obligation | §2.1 honesty boundary |
| Techlog PL processing/stacking/fluids/rates/SIP/report/toolbox/plot inventory | `EVIDENCE-ONLY` except station median and RST schema; names are not equations | SB-PLG-008/012; E-3 |
| Techlog 98-family production taxonomy | `EVIDENCE-ONLY`; unit defects drive typed refusal | SB-PLG-002/015/043 |
| Techlog cement probability architecture | `ADOPTED` | SB-PLG-023…025 |
| Techlog pipe-evaluation architecture | `ADOPTED` where equations are printed | SB-PLG-032…043 |
| Geolog cement suite | `ADOPTED` where manifests define equations/parameters; contradictory labels carried | SB-PLG-016…030 |
| Geolog casing suite | `ADOPTED` where definitions are readable | SB-PLG-032…044 |
| Geolog production geometry and holdup colormaps | `ADOPTED` as geometry/semantic evidence; no flow capability inferred | SB-PLG-007/015; E-2 |
| C-1 — logarithmic bond index | `ADOPTED` | SB-PLG-017 |
| C-1a — amplitude bond index | `ADOPTED` named alternative, no default | SB-PLG-018 |
| C-2 — circumferential coverage | `ADOPTED` | SB-PLG-019/020/048 |
| C-3 — measurement-family polarity | `ADOPTED` | SB-PLG-016 |
| C-4 — slurry impedance | `ADOPTED` | SB-PLG-021 |
| C-5 — expected CBL | `DEFERRED` P2 optional initializer; attribution mandatory | SB-PLG-022; E-8 |
| C-6 — cement probability/confidence | `ADOPTED`; disabled-term arithmetic escalated | SB-PLG-023…025; O-1 |
| C-7 — channel detection | `ADOPTED` with contradiction warning | SB-PLG-026; O-3 |
| C-8 — foamed-cement derivative | `ADOPTED` except estimator, which ships absent | SB-PLG-027; O-2/O-4 |
| C-9 — microdebond | `ADOPTED` P2; primary papers escalated | SB-PLG-028; E-7 |
| C-10 — four wall-loss quantities | `ADOPTED` | SB-PLG-032/033 |
| C-10b — penetration merge | `ADOPTED` with no default | SB-PLG-034 |
| C-11 — ovality/ellipticity | `ADOPTED`; undefined third convention import-only | SB-PLG-035; O-7 |
| C-12 — Barlow | `ADOPTED`; source-governed strength/geometry | SB-PLG-036/037 |
| C-13 — spinner Vapp | `ADOPTED` | SB-PLG-003…005 |
| C-14 — Chronolog | `ADOPTED` | SB-PLG-012…014 |
| §4.0 strategic sequence | `ADOPTED` | SB-PLG-001 |
| §4.1 cement choices | `ADOPTED` or explicitly escalated above | SB-PLG-016…031 |
| §4.2 casing choices | `ADOPTED` or explicitly escalated above | SB-PLG-032…044 |
| §4.3 flow choices | `ADOPTED` through Vapp; post-Vapp work refused | SB-PLG-002…015 |
| §5.3 dossier tests T1.1…T3.5 | `ADOPTED` and expanded | SB-PLG-T03…T68 |
| §5.4 defect rules | `ADOPTED` where applicable | Requirements, parameters and refusals throughout |

### 8.3 Discrepancy and open-item disposition

| Dossier item | Disposition | Where it went |
|---|---|---|
| M-D-1 — discriminator 5× conflict | `ESCALATED`; ships absent | §5 `SPIN_DISC`; E-6 |
| M-D-2 — threshold prose/dialog mismatch | `ADOPTED` in-situ-only rule | SB-PLG-004 |
| M-D-3 — slurry row divergence | `ADOPTED` corrected arithmetic; vendor gap recorded | SB-PLG-021; T34/T35 |
| M-D-4 — casing result column mismatch | `ESCALATED` | O-10; E-6 |
| M-D-5 — cutoff direction failure | `REJECTED` defect; family polarity adopted | SB-PLG-016 |
| M-D-5b — crossed scanner endpoints | `REJECTED` as invalid parameter set | SB-PLG-016; T27 |
| M-D-6 — cement rules not equations | `ADOPTED` as evidence boundary | SB-PLG-016…031 |
| M-D-7 — dB/m versus dB/ft | `ADOPTED` typed-unit refusal | SB-PLG-002 |
| M-D-8 — stale/inconsistent grids | `EVIDENCE-ONLY`; no grid copied | §5 scoped presets |
| M-D-9 — raster-only 0.5 factor | `ADOPTED` exposed, deferred with slippage | §5 `SLUG_CHURN_DEV` |
| M-D-10 — casing derived labels | `ADOPTED` definition-bearing names | SB-PLG-032/035 |
| M-D-11 — malformed temperature label | `REJECTED` label; prose quantities adopted | SB-PLG-009 |
| M-OPEN-1 — gas flowmap | `ESCALATED`; implementation refused | SB-PLG-006; E-4 |
| M-OPEN-2 — slippage charts | `ESCALATED`; implementation refused | SB-PLG-006; E-4 |
| M-OPEN-3 — deviation weighting | `ADOPTED` as required user table | SB-PLG-007; §5 |
| M-OPEN-4 — casing column identity | `ESCALATED` | O-10; E-6 |
| M-OPEN-5 — cement chart axes/values | `REJECTED` for transcription | RF list; CONTRACT §2.1 |
| M-OPEN-6 — Vmix/Vapp function | `ESCALATED`; implementation refused | SB-PLG-006; E-4 |
| M-OPEN-7 — casing database | `ESCALATED` | SB-PLG-037; E-5 |
| M-OPEN-8 — derivative estimator | `OPEN`; not closed by absolute polarity evidence | SB-PLG-027; O-2 |
| M-OPEN-9 — compatibility claim | `REJECTED`; only named published methods permitted | SB-PLG-006 |
| M-OPEN-10 — chart digitization | `REJECTED` until licensed/sourced route exists | §7.3 |
| M-OPEN-11 — radial normalization | `OPEN` | O-9; E-6 |
| M-OPEN-12 — empty pages | `EVIDENCE-ONLY` | IP-14 |
| M-OPEN-13 — strength coupling | `OPEN` | O-8 |
| G-D-1 — 1.5 versus 2.7 | `RESOLVED`; 2.7 prepared value, 1.5 warned fallback | §5 `ZAI_ADEQUATE`; T43 context |
| OPEN-G2 — scalar-BPI interior | `OPEN` | O-5 |
| OPEN-G3 — grading standard | `OPEN` | O-6 |
| OPEN-G4 — derivative display/cutoff | `OPEN` | O-4 |
| OPEN-G5 — channel direction label | `OPEN`; physical `≥` adopted with warning | O-3; SB-PLG-026 |
| OPEN-P8 — expected-CBL provenance | `ESCALATED` | SB-PLG-022; E-8 |
| OPEN-T3 — liquid/attenuation defaults | `REJECTED` as defaults | §5 `AI_LIQUID/ATT_ENDPOINTS` |
| OPEN-T4 — yield strength | `ESCALATED`; ships absent | SB-PLG-036; §5 |
| OPEN-T6 — counts per revolution | `ESCALATED`; ships absent | SB-PLG-002; §5 |
| OPEN-T7 — radius indices | `OPEN`; import-only | O-7; SB-PLG-035 |
| OPEN-T8 — disabled probability terms | `OPEN` | O-1; SB-PLG-024 |
| T-D-1 — bond-index reading trap | `RESOLVED`; logarithmic form adopted | SB-PLG-017 |
| T-D-2 — negative MLOSS sign | `RESOLVED` into positive-corrosion signed convention | SB-PLG-032/033 |
| T-D-3 — burst-pressure unit defect | `REJECTED` vendor unit; psi canonical | SB-PLG-036 |
| T-D-4 — 0.667 cap versus 0.8 | `ADOPTED` as explicit UI constraint | SB-PLG-025 |
| T-D-5 — 70 MRayl liquid | `REJECTED` as default | §5 `AI_LIQUID` |
| T-D-6 — velocity declared mV | `REJECTED` documentation defect | SB-PLG-002 |
| T-D-7 — attenuation endpoint polarity | `REJECTED` as default | §5 `ATT_ENDPOINTS` |
| T-D-8 — numeric default on object reference | `REJECTED` documentation defect | SB-PLG-002/046 |

### 8.4 Gap and escalation disposition

| Dossier gap | Disposition | Where it went |
|---|---|---|
| E-1 — missing independent PL comparator | `ESCALATED` | §7.2 E-1 |
| E-2 — no Geolog base flow implementation | `ESCALATED` without claiming capability absence | §7.2 E-2 |
| E-3 — Techlog flow equations absent | `ESCALATED` | §7.2 E-3 |
| E-4 — IP PL manual absent | `ESCALATED`; hard implementation boundary | §7.2 E-4; SB-PLG-006 |
| E-5 — casing geometry database absent | `ESCALATED` | §7.2 E-5; SB-PLG-037 |
| E-6 — three live UI checks | `ESCALATED` | §7.2 E-6 |
| E-7 — three primary cement papers | `ESCALATED` | §7.2 E-7; SB-PLG-028 |
| E-8 — expected-CBL provenance | `ESCALATED` | §7.2 E-8; SB-PLG-022 |
| E-9 — independent domain literature absent | `ESCALATED` | §7.2 E-9 |
| E-10 — compiled bad-azimuth helper | `REJECTED` for decompilation; independent source escalated | §7.2 E-10; SB-PLG-039 |

### 8.5 Dossier parameter-row disposition

The dossier contains 111 parameter rows. Each is accounted for below; combined vendor rows remain
combined so the count matches the source. Chapter §5 expands them into 132 typed product rows.

| Dossier parameter row | Disposition | Where it went |
|---|---|---|
| Dossier §5.2 — `gas_behind_pipe_impedance` (ultrasonic tool class only) | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `coverage_cutoff_pct` (IP preset) | `EVIDENCE-ONLY / selectable sourced preset` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `coverage_cutoff_pct` (Geolog preset) | `EVIDENCE-ONLY / selectable sourced preset` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `coverage_cutoff_pct` (house) | `ADOPTED as ABSENT or required input` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `use_as_pass` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `cbl_fullbond_mV` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `cbl_freepipe_mV` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `cbl_fullbond_mV` (Techlog preset) | `EVIDENCE-ONLY / selectable sourced preset` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `cbl_freepipe_mV` (Techlog preset) | `EVIDENCE-ONLY / selectable sourced preset` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `zai_good_MRayl` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `zai_adequate_MRayl` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `zai_freepipe_MRayl` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `ai_liquid_MRayl` | `ADOPTED as ABSENT or required input` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `att_bonded_dB_m` / `att_freepipe_dB_m` | `ADOPTED as ABSENT or required input` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `ram_good_mV` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `ram_adequate_mV` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `bond_index.interpolation` | `ADOPTED as ABSENT or required input` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `accepted_probability` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `accepted_confidence` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `isolation_interval_m` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `formation_bonding_probability` / `_confidence` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `peak_window_size_us` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `tt_expected_us` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `cbl_gain` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `wf_delay_us` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `casing_id_in` / `outer_casing_id_in` / `open_hole_size_in` | `ADOPTED` | §5 cement rows; SB-PLG-016…031 |
| Dossier §5.2 — `deriv_opt_smooth` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_smooth_width` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_method` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_differencing_scheme` | `ADOPTED as ABSENT or required input` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_vertical_stddev_window_frames` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_statistics_enabled` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_pass_through` / `deriv_prefix_out` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `zai_derivabs_cutoff_MRayl` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `bpi_zaideriv_cutoff_pct` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `zai_derivabs_array_width` | `ADOPTED as ABSENT or required input` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_window_size` (IP preset) | `EVIDENCE-ONLY / selectable sourced preset` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `deriv_solidcut` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `microdebond_half_window` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `debond_zai_threshold` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `debond_stddev_threshold` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `channel_pct_min_width` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `channel_depth_min_length_m` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `channel_threshold` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `channel_coverage_threshold_pct` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `bondscore_bands` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `cement_bond_bands` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `bond_quality_min_thickness_m` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `ccl_cutoff` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `collar_window_m` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `collar_influence_m` | `ADOPTED as ABSENT or required input` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `include_collars_in_statistics` | `ADOPTED` | §5 cement/collar rows; SB-PLG-020/026…031/044 |
| Dossier §5.2 — `condition_bands` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `threshold_nominal_pct` | `EVIDENCE-ONLY / selectable sourced preset` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `meas_accuracy_pct` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `reporting_bands` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `display_bands` | `EVIDENCE-ONLY / selectable sourced preset` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `casing_grade` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `yield_strength_psi` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `burst_safety_factor` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `burst_acceptable_psi` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `eccentering_limit_pct` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `ovality_limit` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `eccentering_method` | `EVIDENCE-ONLY / selectable sourced preset` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `eccenter_frames_smooth` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `despike_threshold_pct` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `despike_range_fraction` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `normalisation_threshold_pct` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `finger_wear_um_per_m` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `log_direction` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `radius_orientation` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `collar_length_m` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `grading_scheme` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `penetration_merge_pct` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `merge_method` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `tool_environmental_correction` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `hist_samples_null` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `gain_finger` / `offset_finger` | `ADOPTED` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `spinner_discriminator_rps` | `ADOPTED as ABSENT or required input` | §5 casing rows; SB-PLG-032…043 |
| Dossier §5.2 — `spinner_threshold` | `ADOPTED as ABSENT or required input` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `typical_thresholds` | `EVIDENCE-ONLY / selectable sourced preset` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `quicklook_C` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `rate_sensitivity` | `EVIDENCE-ONLY / selectable sourced preset` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `differentiation_length_ft` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `calc_zone_above_reservoir` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `conductivity_loss` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `oil_expansion` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `friction_heat_guidance` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `friction_negligible_above` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `slug_churn_dev_factor` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `deviation_trigger_deg` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `slippage_reversal_deg` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `relative_roughness` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `pts_friction_multiplier` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `max_vslip_ratio` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `deviated_oil_water_anchor_Ew` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `array_deviation_weighting` | `ADOPTED as ABSENT or required input` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `slippage_correlation` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `vmix_from_vapp` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `gas_flow_regime_map` | `DEFERRED with post-Vapp method` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `holdup_curves` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `flow_shading` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `array_tool_diam_units` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `spinner_counts_per_rev` | `ADOPTED as ABSENT or required input` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `station_null_values` | `ADOPTED` | §5 production rows; SB-PLG-002…015 |
| Dossier §5.2 — `porosity` | `ADOPTED` | §5 RST rows; SB-PLG-008 |
| Dossier §5.2 — `carbonate_fraction` | `ADOPTED` | §5 RST rows; SB-PLG-008 |
| Dossier §5.2 — `borehole_status` | `ADOPTED` | §5 RST rows; SB-PLG-008 |
| Dossier §5.2 — `casing_size` | `ADOPTED` | §5 RST rows; SB-PLG-008 |
| Dossier §5.2 — `casing_weight` | `ADOPTED` | §5 RST rows; SB-PLG-008 |
| Dossier §5.2 — `bit_size` | `ADOPTED` | §5 RST rows; SB-PLG-008 |

### 8.6 Critique disposition

| Critique finding | Disposition | Where it went |
|---|---|---|
| BLK-1 — 1.5/2.7 workflow state misread | Applied: 2.7 adopted, 1.5 warned as unprepared fallback | §2.6; §5 `ZAI_ADEQUATE` |
| BLK-2 — station median falsely withdrawn | Applied: two-tool median default restored | SB-PLG-012; §5 `STATION_EST` |
| BLK-3 — missing M-OPEN-13 | Applied individually | O-8; §8.3 |
| MAJ-1 — derivative estimator falsely closed | Applied: estimator ships absent | SB-PLG-027; O-2 |
| MAJ-2 — vertical window mis-scoped as smoothing | Applied: separate parameters and test | SB-PLG-027; T42 |
| MAJ-3 — 0.3 MRayl over-promoted | Applied: ultrasonic-only scope, composite threshold separated | §2.6; §5 |
| MAJ-4 — wall-loss forms converge, not diverge | Applied with closed-form relationship and swept fixture | §2.8; T48/T50 |
| MAJ-5 — penetration and thickness-loss grades conflated | Applied: separate quantities | SB-PLG-038; T57 |
| MAJ-6 — scalar despike scope/default omitted | Applied: three stages and explicit house posture | SB-PLG-039; §5 |
| MAJ-7 — three cement colormaps omitted | Applied without merging them into equations | SB-PLG-029; O-4 |
| MAJ-8 — −100 lower reporting bound suppressed | Applied: signed loss preserved | SB-PLG-033; §5 `REPORT_BANDS` |
| MAJ-9 — pipe merge/QC/correction parameters dropped | Applied | SB-PLG-034/039/042; §5 |
| MAJ-10 — velocity catalogue called internally consistent | Applied: no fixed vendor factor asserted | §2.1; SB-PLG-002; T03/T04 |
| m1 — slurry row 2 truncated | Applied: `5.65238247`, not `5.6523` | SB-PLG-T35 |
| m2 — array width misattributed/uniformity missed | Applied: actual width required; 72 and 360 tested | SB-PLG-019/048; T32 |
| m3 — 85% source overclaimed | Applied: three grids plus prose only; preset remains scoped | §5 `COV_PRESETS` |
| m4 — channel direction contradiction omitted | Applied and kept open | SB-PLG-026; O-3 |
| m5 — collar search semantics dropped | Applied: jump-ahead and hardware length | SB-PLG-044; T64 |
| m6 — gas/water probe offsets omitted | Applied at structural level without proprietary lookup rows | SB-PLG-007; T13 |
| m7 — RST inputs/outputs truncated | Applied: 24-output schema, CDV and optional-Sw mode | SB-PLG-008; T15/T16 |
| m8 — cement toggles and object-default defect omitted | Applied: dependency enforced; invalid object default rejected | SB-PLG-024/046 |
| m9 — confidence meaning omitted | Applied: confidence remains distinct and visible | SB-PLG-023/025 |
| m10 — quote attribution slip | Applied in evidence handling; no verbatim vendor quote needed here | §2/§8 |
| m11 — crossed endpoints called empty rather than double-classified | Applied: contradictory ordering is a load error | SB-PLG-016; T27 |
| m12 — radial/debond units over-sourced | Applied: units attributed to correct readable source or marked contextual | §5 `RAM_*` / `DEBOND_*` |
| m13 — local asset identifiers propagated | Applied: no local identifier or asset detail appears in this chapter | Self-check; §8.7 |

### 8.7 Completeness statement

This chapter accounts for all 14 IP inventory items, the Techlog and Geolog inventory groups, all
16 canonical forms (`C-1`, `C-1a`, `C-2`…`C-10`, `C-10b`, `C-11`…`C-14`), every `M-D`, `M-OPEN`,
`G-D`, `OPEN-G`, `OPEN-P`, `OPEN-T` and `T-D` item touching the domain, all ten dossier gaps, all
111 dossier parameter rows, all 26 critique findings and every adoption-choice/test group. The
chapter expands the dossier into 48 unique requirements, 132 parameter rows and 68 acceptance
tests. No Tier-C item falls in this domain. No spine correction was required.
