# 19. TOC and unconventional resources — requirements

> **Dossier:** `docs/research_2026-08/cross_tool/toc-unconventional.md` — 2,314 lines — read in full 2026-08-08
>
> **Critique:** `docs/research_2026-08/cross_tool/toc-unconventional_critique.md` — 600 lines — read in full 2026-08-08
>
> **Evidence tiers held:** T1 executable source/manifests; T2 complete manual ingest; T3 shipped
> documentation and install manifests; T4 held research notes
>
> **Requirements:** 43 · **P0:** 18 · **Parameters:** 76 (23 `ABSENT`) · **Acceptance tests:** 58

The revised dossier governs. It incorporates both critique blockers, all seven major findings, all
eleven minor findings and the subsequent source discoveries. No proprietary regression coefficient,
chart payload, compiled algorithm or location-specific example is transcribed.

---

## 1. Scope and boundary

This chapter owns log-derived total organic carbon (TOC), maturity handling, kerogen volume,
organic-matter-corrected porosity, single-component Langmuir adsorption, free-gas content, areal
gas-in-place, Ambrose pore-volume correction, TOC-adjacent brittleness and mud-gas ratio analysis.
It owns the unit and naming boundary between intensive gas content and extensive gas volume.

Named seams:

- `SB-CORE-001`: TOC, density, pressure, temperature, gas content and gas-volume units.
- `SB-CORE-002`: clamps preserve raw values and emit stable flags.
- `SB-CORE-006`: curve labels, equations and physical quantities agree.
- `SB-CORE-010`: every method, parameter, pick and output carries provenance.
- `11_porosity.md`: source density/neutron/sonic porosity and organic-matter correction handoff.
- `12_saturation.md`: `Sw`/`So` inputs and their total/effective reference systems.
- `16_nmr.md`: NMR porosity-deficit TOC is deferred until the NMR distribution path exists.
- `18_geomech-ppfg.md`: static/dynamic elastic semantics and pressure/depth datum.
- `21_data-io.md`: unit-tagged TOC and mud-gas channel import.
- `23_plotting-interactivity.md`: overlay picks, calibration crossplots and parameter persistence.
- `25_fluidsub-rockphysics.md`: gas properties and elastic-property ownership.

This chapter does not own proprietary density-regression coefficients, vendor-compiled mud-gas
equations, raw NMR inversion, isotope-template interpretation, cut-off-based resource ranking or
CO2 storage. Those items are rejected, source-gated or routed to their owning domain below.

---

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 TOC is a unit-bound quantity, not a bare number

Two tools calculate Passey TOC as a weight fraction, while another tool and SandiBumi expose wt%.
One incumbent mixes fraction and percent across—and even within—modules. The resulting failure is
exactly 100× and remains numerically plausible. SandiBumi's shipped `toc_passey → kerogen` chain is
internally consistent in wt%; changing canonical unit would be a breaking migration that buys no
correctness. The obligation is therefore a typed wt% representation with named conversion to/from
fraction, never an untagged scalar (`toc-unconventional.md` §§3.1, 3.14, 5.1; T1/T2/T3).

The critique found the same defect inside the dossier's first adoption spec: `TOC_LANG` had been
converted to fraction while the numerator stayed in wt%, planting a 100× adsorbed-gas error. The
revised source correctly pairs wt%/wt% and v/v/v/v paths and adds a direct identity test (dossier
§§2.7, 5.1–5.4; critique BLOCKER-1; T1).

### 2.2 Delta-log-R is settled; baselines and maturity are not defaults

Two independent T1 implementations agree on all three overlay coefficients: sonic `+0.02`, density
`−2.50`, neutron `+4.00`. The negative neutron sign in a legacy SandiBumi reference is a latent
documentation defect, not shipped math, because current code exposes sonic and density only. A
0.10 v/v neutron excursion would move separation by 0.80 decades between the two signs and by about
2.58 wt% at LOM 10.6 (`toc-unconventional.md` §§2.1, 3.2; T1/T3).

Baseline choices dominate the answer. Substituting documented incumbent seeds for SandiBumi's
current seeds can shift TOC by 1.28–2.82 wt%, comparable to the whole source-rock grading interval.
Baselines must therefore be interval picks, not global defaults. One incumbent's single-depth pick
is the strongest design because all baseline curves come from the same rock (dossier §§2.3, 3.4;
T1/T2/T3).

The Passey factor and VR-to-LOM polynomial are printed in T1 sources. A 10.5 LOM cap is cited and
shipped by one tool; another contains the same clamp commented out. At VR 2.0 the uncapped path
under-calls TOC by about 3.4×. SandiBumi already enforces a 6.0 lower range, but its 12.0 ceiling
rejects rather than applies the cited cap with a flag (`toc-unconventional.md` §§2.2, 3.3; T1/T3).

### 2.3 Method diversity is valuable only when provenance and calibration survive

Incumbents disagree on final overlay selection and none defends its default. Computing sonic,
density and neutron together, preserving each output, and emitting minimum/average/maximum plus
spread exposes uncertainty instead of hiding it. Proprietary weighted combinations and anonymous
density-regression banks are not reproducible evidence (`toc-unconventional.md` §§3.11, 3.13,
4.1; T1/T2/T3).

Only one tool fits per-overlay and final slope/intercept against lab TOC. That capability matches
the evidenced interpretation workflow and matters more than another universal coefficient. The fit,
zone, data revision and statistics must persist with the result (dossier §§3.9, 4.1; T1/T4).

### 2.4 Kerogen conversion is corroborated, but similarly named constants differ

Three independent implementations corroborate
`V_KER=(TOC/100)·KCF·RHOB/RHO_KER` for wt%-canonical TOC, and the inverse is algebraic. A separate
incumbent factor converts wt% directly to volume and folds the density ratio into its number; it is
not interchangeable with `KCF`. Cross-substitution is about a 2× error (`toc-unconventional.md`
§2.6; T1/T2).

The critique proved that SandiBumi's shipped `RHO_KER=1.10 g/cc` is correctly valued but wrongly
attributed in its legacy reference. Like-for-like incumbent evidence carries the same 1.10 endpoint,
including matching neutron and sonic responses. Moving it because of the old critique premise would
shift kerogen volume 27% and corrected porosity 40% in the dossier fixture. The value stays; its
source and maturity caveat change (dossier §§3.5, 5.2; critique BLOCKER-2; T1/T2).

### 2.5 RockEval must not be compacted to six indices

The T1 manifest prints six indices plus both pyrolysed-carbon branches, both residual-carbon
branches and `TOC=SRAPC+SRARC`. The `12/280` and `12/440` constants include the ppk-to-percent
step; simplifying them to `12/28` and `12/44` creates a 10× error. The full branch is also the only
evidenced method that reconstructs TOC when lab TOC is missing or incomplete
(`toc-unconventional.md` §§2.5, 5.1; critique MAJOR-6; T1).

### 2.6 Gas content and gas-in-place are different physical quantities

SandiBumi currently emits `GIP_*` in scf/ton. That is intensive gas content, while an incumbent
uses the same names for extensive Bcf volumes. Both are positive curves, so a mnemonic-only merge
fails silently. The domain must reserve `GC_*` for scf/ton and `GIP_*` for a declared area/thickness
volume (`toc-unconventional.md` §3.15; T1).

The current Langmuir volume is flat with depth. Two independent tools instead scale capacity with
TOC, and one also supplies laboratory-isotherm fitting and a through-origin TOC calibration. Missing
TOC coupling is the largest functional gap in the shipped module. `V_L`, `P_L`, pressure,
temperature and gas properties are measured inputs and must not be hidden placeholders (dossier
§§2.7–2.7.1, 3.15; T1/T2).

Free-gas content needs oil saturation, non-combustible fraction and a named standard condition.
The incumbent standard-condition chain differs from SandiBumi by 1.31% when its extra factor is
included. The Ambrose coefficient is independently corroborated across two tools to 0.03%, but
sorbed density and molar-mass choices change the correction by 42%; both therefore ship absent.
One incumbent corrects intensive free gas but not extensive free GIP. SandiBumi must correct both
and assert their identity (`toc-unconventional.md` §§2.7, 3.7–3.8; T1/T2).

### 2.7 Brittleness needs an explicit scale and modulus basis

The Rickman elastic form is common, but one tool emits 0–100 while SandiBumi emits 0–1. Another
uses static moduli only with user-picked endpoints. SandiBumi's current dynamic-modulus calculation
and [0,1] output are coherent; the remaining obligation is to preserve the basis and scale in
metadata and refuse static moduli paired silently with the dynamic endpoints
(`toc-unconventional.md` §2.8; T1/T2).

### 2.8 Mud-gas thresholds converge; equations and class boundaries do not

Two tools agree on wetness thresholds 0.5/17.5/40, balance 100 and character 0.5. The printed GWR
denominator is unresolved: a full C1–C5 denominator is physically bounded, while the compatibility
form omits C3/C4. The dossier recommends a canonical/compatibility split but also lists a canonical
default despite stating that mode selection must be explicit. Because the primary paper is absent,
this chapter resolves that internal contradiction conservatively: both modes may exist, but
`GWR_MODE` ships absent (`toc-unconventional.md` §§3.12, 4.2, 5.2–5.4; T1/T2/T4).

One incumbent's eight-class table has two overlaps and five uncovered regions. Half-open,
total-and-disjoint rules plus an explicit missing-input state are therefore product requirements,
not stylistic choices. Pixler, oil-indicator and normalization formulas remain source-gated because
their executable bodies or reference constant are not held (dossier §§2.9–2.10, 4.2; T1/T2/T3).

---

## 3. SandiBumi as-built

The codebase-index server is unavailable in this session, so source claims were re-verified with
targeted `rg` plus direct reads of the exact Rust/TypeScript files. Negative results were searched
across the module registry and UI sources rather than inferred from one file.

### 3.1 Registered shipped modules

`toc_passey`, `kerogen`, `gip` and `brittleness` are registered and executable
(`src-tauri/src/modules.rs:484-487,565-568`, T1). The spine's inventory of those four module names
is accurate. Status: **PRESENT-OK** as an inventory claim.

### 3.2 TOC and kerogen

`toc_passey` ships sonic/density overlays, wt% output, the cited Passey factor, clamp-then-add
background order and Schmoker cross-check (`src-tauri/src/unconventional.rs:18-111`, T1). It has no
neutron input or branch. `R_BASE=2.0`, `DT_BASE=70`, `RHOB_BASE=2.65`, `LOM=10.6` with range 6–12,
and `TOC_BG=0` are preselected (`:36-42`). Status: **PARTIAL** for method coverage and
**PRESENT-DIVERGENT** for the no-default/cap contract.

The `kerogen` path correctly divides wt% by 100 and computes/caps bulk kerogen volume, with
`RHO_KERO=1.10` and `K_TOC2OM=1.2` (`unconventional.rs:117-179`, T1). Its documentation incorrectly
compares the 1.10 endpoint to a different rock-mechanics density (`:122-134`, T1). The value and
math are **PRESENT-OK**; source semantics are **PRESENT-DIVERGENT**.

No source symbol implements generalized density TOC, uranium TOC, masks, core calibration,
S2/HI, RockEval or inverse kerogen-to-TOC (targeted registry/source `rg`, 2026-08-08, T1 negative
search). Status: **ABSENT**.

### 3.3 Gas storage

The `gip` module explicitly describes its output as per-sample gas content in scf/ton, but names
the curves `GIP_ADS`, `GIP_FREE` and `GIP_TOTAL` (`unconventional.rs:247-281`, T1). It preselects
reservoir pressure, temperature, z, `VL`, `PL`, ash, moisture and measured gas content (`:262-275`).
Its Langmuir term is not TOC-scaled; free gas omits oil saturation and non-combustible fraction;
the standard condition is implicit in `0.02827`; Ambrose and areal GIP are absent (`:286-364`, T1).
Status: **PRESENT-DIVERGENT**.

This source finding contradicts the spine's unqualified phrase “gas-in-place”: the shipped curves
are intensive content and take no thickness or area. The correction is recorded as SP-002 in
`_SPINE_PENDING.md`; the spine itself remains untouched.

### 3.4 Brittleness

The elastic path calculates dynamic `E`/`nu`, normalizes the cited endpoints and clamps BI to [0,1].
Mineralogical Jarvie and Wang-Gale paths are also present (`unconventional.rs:489-609`, T1). The
output and equations are **PRESENT-OK**. The API does not carry a static/dynamic provenance type,
so the cross-domain guard is **PARTIAL**.

### 3.5 Visual companion

The UI presents sonic/density overlays and a Langmuir curve. Its baseline and `VL/PL` controls are
client-side values initialized to the same unsourced numbers; they do not write module parameters
or pick provenance (`src/ui/unconventionalPanel.ts:454-639`, T1). Status:
**PRESENT-DIVERGENT** — the visual can disagree with a computation while looking authoritative.

No RockEval, mud-gas analysis or TOC-calibration implementation was found in the registry or UI
(targeted `rg`, 2026-08-08, T1 negative search). Status: **ABSENT**.

---

## 4. Requirements

### 4.1 TOC units, overlays, baselines and calibration

#### SB-TOC-001 — Make TOC a unit-tagged wt% quantity [P0] [status: PARTIAL]

**Requirement.** Every TOC input, parameter and output MUST carry `wt%` or `w/w` identity. The
canonical compute representation MUST remain wt%; conversions MUST be named and round-trip tested.

**Rationale.** Bare TOC scalars create the dossier's 100× H-D-8/GL-D-2 failure class (§§3.1, 3.14;
T1/T2/T3).

**As-built.** PARTIAL — the current chain is numerically consistent in wt% but stores ordinary
numeric arrays and strings (`unconventional.rs:42,47,123,153-161`).

**Verified by.** SB-TOC-T01–T04

#### SB-TOC-002 — Ship no numeric delta-log-R baseline [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `RT_bl`, `DT_bl`, `RHOB_bl` and `NPHI_bl` MUST ship absent. One interval/depth pick
MUST resolve all available baseline channels together and persist the interval and curve revisions.

**Rationale.** Evidenced baseline substitutions shift TOC 1.28–2.82 wt% (dossier §§2.3, 3.4;
T1/T2/T3).

**As-built.** PRESENT-DIVERGENT — three unsourced values are preselected and independently editable
(`unconventional.rs:38-40`; `unconventionalPanel.ts:483-572`).

**Verified by.** SB-TOC-T14–T15

#### SB-TOC-003 — Compute all three overlay separations [P1] [status: PARTIAL]

**Requirement.** Sonic, density and neutron separations MUST use `+0.02`, `−2.50` and `+4.00` in
their native units, emit separate curves and accept metric sonic only through an explicit conversion.

**Rationale.** Two T1 implementations agree exactly; the old negative neutron sign changes the
fixture by 0.80 decades (§§2.1, 3.2; T1/T3).

**As-built.** PARTIAL — sonic and density exist; neutron is absent (`unconventional.rs:36,73-83`).

**Verified by.** SB-TOC-T05, SB-TOC-T08, SB-TOC-T34

#### SB-TOC-004 — Preserve the native Passey wt% and clamp order [P0] [status: PRESENT-OK]

**Requirement.** TOC MUST equal `max(0,DLR·10^(2.297−0.1688·LOM))+TOC_BG` in wt%, with the
background added after the Passey term is floored.

**Rationale.** T1 equations corroborate the constants; the missing `/100` is correct only because
SandiBumi remains wt%-canonical (dossier §5.1; T1).

**As-built.** PRESENT-OK — the source implements that exact order (`unconventional.rs:85-95`).

**Verified by.** SB-TOC-T03, SB-TOC-T09–T10

#### SB-TOC-005 — Apply the cited LOM cap with a flag [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Direct LOM or the cited VR polynomial MUST produce `LOM_raw`; 10.5 MUST be an
enabled, user-defeatable cap that emits `LOM_CAPPED`. Values below 6 MUST warn without a house clamp.

**Rationale.** The capped/uncapped VR=2 fixture differs by 3.416× (§§2.2, 3.3; T1/T3).

**As-built.** PRESENT-DIVERGENT — LOM is directly entered with a 6–12 validation range and no cap
flag or VR path (`unconventional.rs:41,67-94`).

**Verified by.** SB-TOC-T11–T13

#### SB-TOC-006 — Treat background TOC as part of the baseline pick [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `TOC_BG` MUST ship absent, MAY show the cited 0.8 wt% seed, and MUST be persisted
with the baseline interval rather than silently applied globally.

**Rationale.** The quantity is the organic content of the lean baseline rock, not a universal
constant (dossier §§2.2, 5.2; T1/T2/T3).

**As-built.** PRESENT-DIVERGENT — 0.0 wt% is silently preselected (`unconventional.rs:42,69,91-95`).

**Verified by.** SB-TOC-T09, SB-TOC-T14–T15

#### SB-TOC-007 — Preserve overlay alternatives and spread [P1] [status: ABSENT]

**Requirement.** `TOC_SONIC`, `TOC_DENSITY`, `TOC_NEUTRON`, minimum, average, maximum and
`TOC_SPREAD` MUST be emitted together. Final selection MUST ship absent because incumbent defaults
conflict without a defensible adjudication.

**Rationale.** Three tools choose three unexplained defaults; the spread is a free uncertainty
measure (§3.11; T1/T2/T3).

**As-built.** ABSENT — one chosen overlay writes one generic `TOC` (`unconventional.rs:36,47,57-108`).

**Verified by.** SB-TOC-T24

#### SB-TOC-008 — Mask invalid geology and borehole conditions before TOC [P0] [status: ABSENT]

**Requirement.** Coal, reservoir/sand and badhole masks MUST execute before every TOC method, use
one exclusion polarity, return null rather than a large TOC and identify the reason.

**Rationale.** Only one incumbent enforces the documented invalidity conditions (§§2.11, 3.10;
T1/T3).

**As-built.** ABSENT — `toc_passey` consumes no mask inputs (`unconventional.rs:36-49,53-111`).

**Verified by.** SB-TOC-T22–T23

#### SB-TOC-009 — Calibrate each overlay and final TOC against lab data [P1] [status: ABSENT]

**Requirement.** Per-zone slope/intercept fits MUST preserve raw prediction, calibrated output,
sample pairing, R-squared, RMSE, data revision and provenance. Identity settings MUST be explicit.

**Rationale.** Only one tool fits calibration; the evidenced workflow selects method by lab
back-test (dossier §3.9; T1/T4).

**As-built.** ABSENT — no calibration inputs, outputs or fit UI are registered.

**Verified by.** SB-TOC-T25

#### SB-TOC-010 — Carry TOC method and pick provenance [P0] [status: ABSENT]

**Requirement.** Every output MUST record input revisions, overlay, baseline interval, maturity
source/cap, background, masks, calibration, unit conversions and software revision.

**Rationale.** These choices move the result by factors larger than method precision (§5.3 rules
3/9/12/14; T1/T2/T3).

**As-built.** ABSENT — module outputs are bare arrays and UI picks are client-local
(`unconventional.rs:105-109`; `unconventionalPanel.ts:630-639`).

**Verified by.** SB-TOC-T57

### 4.2 Density TOC, kerogen and RockEval

#### SB-TOC-011 — Retain the cited Schmoker cross-check [P1] [status: PRESENT-OK]

**Requirement.** The density cross-check MUST compute `max(0,154.497/RHOB−57.261)` in wt% and
retain its method identity.

**Rationale.** The constants and zero crossing are corroborated in held sources (dossier §2.4;
T1/T3).

**As-built.** PRESENT-OK — exact source form at `unconventional.rs:97-103`.

**Verified by.** SB-TOC-T16

#### SB-TOC-012 — Offer the generalized density-deficit form with harmonic grain density [P2] [status: ABSENT]

**Requirement.** The generalized form MUST use semantic density parameters, harmonic mixing of
weight fractions, explicit normalization below unity, and refusal when weights exceed unity.

**Rationale.** T1 source prints the form and its `sum(w)>1` warning (§2.4; T1/T3).

**As-built.** ABSENT — only the fixed Schmoker form exists.

**Verified by.** SB-TOC-T17–T18

#### SB-TOC-013 — Implement uranium TOC only with its environmental warning [P2] [status: ABSENT]

**Requirement.** Uranium TOC MUST use the cited wt%-converted gain, clamp negative output, preserve
raw value and warn where uranium enrichment is not an admissible organic proxy.

**Rationale.** T1 supplies the regression; held literature supplies the validity warning
(dossier §§2.5, 4.1; T1/T4).

**As-built.** ABSENT — no uranium TOC module is registered.

**Verified by.** SB-TOC-T21

#### SB-TOC-014 — Make kerogen conversion bidirectional and guarded [P1] [status: PARTIAL]

**Requirement.** TOC→kerogen and kerogen→TOC MUST be inverse operations under the same `KCF`,
`RHO_KER` and `RHOB`; negative or out-of-range results MUST flag rather than propagate.

**Rationale.** Three independent implementations corroborate the pair (§2.6; T1/T2).

**As-built.** PARTIAL — forward conversion and clamps exist; the inverse path is absent
(`unconventional.rs:145-179`).

**Verified by.** SB-TOC-T19

#### SB-TOC-015 — Keep the 1.10 kerogen endpoint and correct its provenance [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `RHO_KER=1.10 g/cc` MUST remain the shipped cited value, stay aligned with the
mineral endpoint, expose sourced alternatives with maturity context and MUST NOT cite the unrelated
rock-mechanics density.

**Rationale.** The critique's proposed change rested on a wrong comparand; like-for-like T2 evidence
matches 1.10 exactly (§3.5; critique BLOCKER-2).

**As-built.** PRESENT-DIVERGENT — code value is correct but its doc names the wrong comparison
(`unconventional.rs:122-134`; `multimin2.rs:2100`).

**Verified by.** SB-TOC-T19–T20

#### SB-TOC-016 — Keep pyrite coupling source-gated [P2] [status: ABSENT]

**Requirement.** The evidenced `V_PYR=slope·V_KER+intercept` MAY be offered as a named option, but
the compiled four-component TOC body MUST remain absent until its primary equations are held.

**Rationale.** The regression is printed; the complete solver is not (§§2.4, 6 OPEN-U-4; T1/T3).

**As-built.** ABSENT — no pyrite-coupled TOC path exists.

**Verified by.** SB-TOC-T26

#### SB-TOC-017 — Compute S2 from typed TOC and selected kerogen type [P2] [status: ABSENT]

**Requirement.** Type-II and Type-III HI polynomials MUST retain their distinct coefficients and
compute `S2=HI·TOC_wt%/100`; type selection and LOM source MUST travel with S2.

**Rationale.** A T1 implementation prints both polynomials and proves the unit boundary (§2.5; T1).

**As-built.** ABSENT — no HI/S2 path exists.

**Verified by.** SB-TOC-T02, SB-TOC-T27

#### SB-TOC-018 — Implement the complete RockEval carbon balance [P2] [status: ABSENT]

**Requirement.** Six indices, both `SRAPC` branches, both `SRARC` branches and
`TOC_FROM_PEAKS=SRAPC+SRARC` MUST ship together with correct ppk, wt% and mass-ratio labels.

**Rationale.** Omitting the carbon branches removes the only TOC-producing function (§2.5;
critique MAJOR-6; T1).

**As-built.** ABSENT — no RockEval symbols are registered.

**Verified by.** SB-TOC-T04, SB-TOC-T28–T29

### 4.3 Adsorbed gas, free gas and areal GIP

#### SB-TOC-019 — Make measured gas inputs required and sourced [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Reservoir pressure, temperature, `Z`, `V_L`, `P_L`, ash, moisture and measured gas
content MUST ship absent unless a cited or measured value is supplied. Each MUST carry unit and
sample/lab provenance.

**Rationale.** The dossier marks the gas inputs absent; dialog examples are not defaults (§5.2;
T1/T2/T3).

**As-built.** PRESENT-DIVERGENT — all are preselected numerical parameters
(`unconventional.rs:262-275`).

**Verified by.** SB-TOC-T30, SB-TOC-T32

#### SB-TOC-020 — Couple Langmuir capacity to the matching organic input [P0] [status: ABSENT]

**Requirement.** The wt% route MUST pair `TOC/TOC_LANG`; the volume route MUST pair
`V_KER/VOL_KER_LANG`. The pairs MUST be mutually exclusive and the two operands MUST share a unit.

**Rationale.** Two tools corroborate proportional TOC scaling; crossed pairs produce 100×
(§§2.7–2.7.1; T1/T2).

**As-built.** ABSENT — `VL` is flat and the module consumes neither TOC nor kerogen volume
(`unconventional.rs:262-281,315-328`).

**Verified by.** SB-TOC-T06–T07, SB-TOC-T31

#### SB-TOC-021 — Keep Langmuir temperature correction opt-in and provenance-bound [P2] [status: ABSENT]

**Requirement.** The cited log-space temperature correction MUST default OFF, retain its native
degrees-Celsius coefficients and display that it came from a single regional coal calibration.

**Rationale.** One T1 source prints the constants and warns against universal use (§2.7; T1).

**As-built.** ABSENT — no temperature correction exists.

**Verified by.** SB-TOC-T33

#### SB-TOC-022 — Include oil and non-combustible gas in free-gas content [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Free gas MUST use `PHI·(1−Sw−So)` and derate that gas-filled porosity by
`1−NCGF`; invalid phase sums MUST refuse and flag.

**Rationale.** One T1 source prints both terms; current two-phase form over-calls gas where either
is nonzero (§§2.7, 3.15; T1/T2).

**As-built.** PRESENT-DIVERGENT — source uses `PHI·(1−Sw)` only (`unconventional.rs:330-346`).

**Verified by.** SB-TOC-T35–T36

#### SB-TOC-023 — Name the Bg standard condition [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Bg MUST carry one z-factor and explicit standard pressure/temperature metadata.
Alternative standard conditions MUST be converted, not silently mixed; the contradictory extra
vendor factor MUST NOT be adopted.

**Rationale.** Held standard-condition chains differ by 1.31% and one manifest defines its extra
factor inconsistently (§§2.7, 3.7; T1/T4).

**As-built.** PRESENT-DIVERGENT — `0.02827` embeds the condition without metadata
(`unconventional.rs:307-313`).

**Verified by.** SB-TOC-T34

#### SB-TOC-024 — Parameterize and flag the Ambrose correction [P1] [status: ABSENT]

**Requirement.** Ambrose correction MUST take sourced `M_GAS` and `RHO_ADS`, ship them absent,
apply the correction outside the non-combustible derate, and flag any pore-volume overshoot.

**Rationale.** The constant is cross-tool corroborated while parameter choices move the term 42%
(§§2.7, 3.8; T1/T2/T4).

**As-built.** ABSENT — source explicitly defers Ambrose (`unconventional.rs:247-260`).

**Verified by.** SB-TOC-T37–T39

#### SB-TOC-025 — Reserve gas-content and gas-in-place names by quantity [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Intensive scf/ton outputs MUST use `GC_ADS`, `GC_FREE`, `GC_TOTAL`; `GIP_*` MUST
mean extensive Bcf with declared area and thickness. Mnemonic aliases MUST NOT override units.

**Rationale.** Industry tools use the same GIP name for both quantities (§3.15; T1/T3).

**As-built.** PRESENT-DIVERGENT — intensive outputs use `GIP_*` (`unconventional.rs:278-281,361-363`).

**Verified by.** SB-TOC-T40

#### SB-TOC-026 — Add an internally consistent areal GIP layer [P1] [status: ABSENT]

**Requirement.** Areal adsorbed/free/total GIP MUST use the cited rock-mass and volume constants,
declared thickness/area units and the same Ambrose-corrected free pore volume as `GC_FREE`.

**Rationale.** One incumbent's intensive and extensive curves disagree because only one is
corrected; the dossier derives the identity (§2.7 GL-D-7; T1).

**As-built.** ABSENT — no thickness, area or Bcf output exists.

**Verified by.** SB-TOC-T41

#### SB-TOC-027 — Preserve guarded critical-desorption and in-situ derate behavior [P1] [status: PARTIAL]

**Requirement.** Critical desorption MUST require `0<GC<V_L`; ash and moisture derates MUST require
measured fractions, preserve the dry-basis value and reject a negative remaining fraction.

**Rationale.** The equation and dry-basis correction are evidenced; missing sulfur is recorded but
not invented (§2.7; T1/T2).

**As-built.** PARTIAL — equations and bounds exist, but missing ash/moisture become zero defaults
(`unconventional.rs:315-328,348-356`).

**Verified by.** SB-TOC-T44–T45

#### SB-TOC-028 — Make the isotherm-fit estimator explicit [P2] [status: ABSENT]

**Requirement.** Lab isotherm fitting MUST expose linearized and direct nonlinear estimators as
different methods, output fit residuals and MUST NOT inherit a linearized fit silently.

**Rationale.** T1 prints the Hanes-Woolf algebra; estimator weighting remains an explicit open
choice (§2.7.1, OPEN-U-12; T1).

**As-built.** ABSENT — the module accepts constants only.

**Verified by.** SB-TOC-T42

#### SB-TOC-029 — Force zero intercept in TOC-to-Langmuir calibration [P1] [status: ABSENT]

**Requirement.** `V_L=TOCLV_GRAD·TOC` MUST force zero intercept by default; any nonzero experimental
intercept MUST be visibly nonphysical-at-zero and require explicit acknowledgement.

**Rationale.** One incumbent's calibration forces zero while its application accepts nonzero
(§2.7.1 GL-D-5; T1).

**As-built.** ABSENT — no capacity-versus-TOC calibration exists.

**Verified by.** SB-TOC-T43

### 4.4 Brittleness

#### SB-TOC-030 — Keep dynamic Rickman brittleness on a declared [0,1] scale [P0] [status: PRESENT-OK]

**Requirement.** Dynamic `E`/`nu` with cited endpoints MUST emit BI on [0,1], and curve metadata MUST
state both modulus basis and scale.

**Rationale.** Incumbent output scales differ by 100× even where equations agree (§2.8; T1/T2).

**As-built.** PRESENT-OK — dynamic calculation, endpoints and [0,1] clamp are explicit
(`unconventional.rs:489-583`).

**Verified by.** SB-TOC-T46

#### SB-TOC-031 — Refuse static moduli with dynamic endpoints [P0] [status: PARTIAL]

**Requirement.** A static modulus input MUST require user-calibrated static endpoints; the cited
dynamic endpoints MUST NOT be applied after an unrecorded static correction.

**Rationale.** Incumbents use coherent but different basis/endpoint pairs (§2.8; T1/T2).

**As-built.** PARTIAL — computation is dynamic, but no typed basis travels with incoming elastic
curves (`unconventional.rs:504-515,546-583`).

**Verified by.** SB-TOC-T47

#### SB-TOC-032 — Preserve mineralogical brittleness method identity [P1] [status: PRESENT-OK]

**Requirement.** Jarvie and Wang-Gale numerators/denominators MUST remain separately named; missing
minerals MAY be zero only when absence is explicit rather than unresolved mapping.

**Rationale.** The methods assign carbonate, dolomite and organic volume differently (§2.8; T1/T4).

**As-built.** PRESENT-OK for equations, with current missing-as-zero behavior documented
(`unconventional.rs:584-606`).

**Verified by.** SB-TOC-T48

### 4.5 Mud-gas ratios and classification

#### SB-TOC-033 — Resolve C1-C5 channel identity before ratios [P1] [status: ABSENT]

**Requirement.** Iso/normal C4 and C5 channels MUST be combined once; supplying combined and
component channels together MUST refuse unless an explicit mapping excludes duplicates.

**Rationale.** The vendor documentation identifies double counting as a live trap (§2.9; T2/T3).

**As-built.** ABSENT — no mud-gas module exists.

**Verified by.** SB-TOC-T49

#### SB-TOC-034 — Require an explicit GWR denominator mode [P1] [status: ABSENT]

**Requirement.** Full-C1–C5 and compatibility denominators MUST be separately named and
`GWR_MODE` MUST ship absent until selected. No result may use a silent default.

**Rationale.** Thresholds converge but the primary denominator is unresolved; the dossier's own
default/mode-required statements conflict (§3.12, H-D-11; T1/T2/T4).

**As-built.** ABSENT — no ratio implementation exists.

**Verified by.** SB-TOC-T50

#### SB-TOC-035 — Make classification rules total and disjoint [P1] [status: ABSENT]

**Requirement.** Every finite ratio triple MUST match exactly one half-open class. Missing input
MUST map to `UNCLASSIFIED`; a rule gap or overlap MUST fail registration.

**Rationale.** The evidenced commercial table has five holes and two overlaps (§2.9 GL-D-4; T1/T2).

**As-built.** ABSENT — no classifier exists.

**Verified by.** SB-TOC-T51–T52

#### SB-TOC-036 — Use only cross-tool-corroborated Haworth thresholds [P1] [status: ABSENT]

**Requirement.** Wetness 0.5/17.5/40, balance 100 and character 0.5 MAY seed a named Haworth
classifier, but every boundary MUST follow SandiBumi's explicit half-open rule.

**Rationale.** Two tools independently print identical constants (§3.12; T1/T2).

**As-built.** ABSENT — no classifier exists.

**Verified by.** SB-TOC-T51

#### SB-TOC-037 — Gate compiled mud-gas extensions on primary equations [P3] [status: ABSENT]

**Requirement.** Pixler, oil-indicator and gas-quality capabilities MUST remain unavailable until
their public primary equations are independently documented and tested; compiled behavior MUST NOT
be inferred.

**Rationale.** Names and thresholds are visible but formula bodies are compiled (§6 OPEN-U-5;
T1/T3).

**As-built.** ABSENT — no extension exists.

**Verified by.** SB-TOC-T53

#### SB-TOC-038 — Keep mud-gas normalization absent until its constant is sourced [P3] [status: ABSENT]

**Requirement.** The known input units MAY be stored, but normalization MUST refuse while the
reference-condition meaning of 5.0028 is unresolved.

**Rationale.** A geometric derivation in the declared units gives 1470.6, proving hidden scaling
(§2.10, OPEN-9; T2/T3).

**As-built.** ABSENT — no normalizer exists.

**Verified by.** SB-TOC-T54

#### SB-TOC-039 — Add component-sum versus total-gas QC [P2] [status: ABSENT]

**Requirement.** Where measured total gas exists, the module MUST emit `sum(C1…C5)−TOTAL_GAS`, its
relative residual and a missing-channel flag before interpretation.

**Rationale.** Held workflow evidence identifies this free QC check (§2.9; T4).

**As-built.** ABSENT — no mud-gas QC exists.

**Verified by.** SB-TOC-T55

### 4.6 UI, flags and migration

#### SB-TOC-040 — Persist visual picks as computation parameters [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Overlay baseline and Langmuir picks MUST write the same typed, undoable parameter
records consumed by compute modules, including zone, data revision, viewport and source note.

**Rationale.** A client-only overlay can look authoritative while disagreeing with computation
(dossier §5.5; T1).

**As-built.** PRESENT-DIVERGENT — controls redraw local canvases only
(`unconventionalPanel.ts:483-639`).

**Verified by.** SB-TOC-T15, SB-TOC-T56

#### SB-TOC-041 — Keep visualization and compute equations identical [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Visual overlays and isotherms MUST consume the registered method/parameter payload,
not duplicated numeric fallbacks; a source revision MUST invalidate the plot.

**Rationale.** The present UI duplicates baseline and Langmuir values (§3.5; T1).

**As-built.** PRESENT-DIVERGENT — UI fallbacks are hard-coded separately
(`unconventionalPanel.ts:483-530,573-596`).

**Verified by.** SB-TOC-T06, SB-TOC-T56

#### SB-TOC-042 — Emit stable unconventional QC flags [P0] [status: ABSENT]

**Requirement.** Unit mismatch, unset baseline, LOM cap, TOC clamp, masks, weight-sum error, phase
sum, Ambrose overshoot, missing gas property, classification gap and missing provenance MUST have
stable codes, per-sample curves where applicable and run-summary counts.

**Rationale.** Silent failures are the dominant risk and current NaN/clamp behavior loses reason
(dossier §5.3 rules 5/14; T1/T2/T3).

**As-built.** ABSENT — outputs carry values without reason codes (`unconventional.rs:53-179,286-364`).

**Verified by.** SB-TOC-T57

#### SB-TOC-043 — Migrate existing projects without changing meanings silently [P0] [status: ABSENT]

**Requirement.** Renaming `GIP_*` content curves, removing numeric defaults and adding typed TOC
MUST use an explicit schema/version migration that preserves old values, units and provenance and
requires acknowledgement before recompute.

**Rationale.** These are changes to shipped curves and saved parameters, not greenfield features
(dossier §5.5; T1).

**As-built.** ABSENT — no domain-specific migration exists for these semantic changes.

**Verified by.** SB-TOC-T58

---

## 5. Parameters

Every number below is byte-exact from the revised dossier or shown as a named derivation. Measured,
conflicted or unjustified values read `ABSENT — ships with no default`.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Sonic overlay coefficient | `K_DT` | +0.02 | 1/(µs/ft) | Techlog `TOC_Computation.py`; Geolog `shale_toc_deltalogr.info`; dossier §5.2 | T1/T3 |
| Density overlay coefficient | `K_RHOB` | −2.50 | 1/(g/cc) | Techlog `TOC_Computation.py`; Geolog `shale_toc_deltalogr.info`; dossier §5.2 | T1 |
| Neutron overlay coefficient | `K_NPHI` | +4.00 | 1/(v/v) | Techlog `TOC_Computation.py`; Geolog `shale_toc_deltalogr.info`; dossier §§2.1/5.2 | T1/T3 |
| Passey intercept | `PASSEY_A` | 2.297 | — | Techlog `TOC_Computation.py`; Geolog `shale_toc_deltalogr.info`; dossier §§2.2/5.2 | T1/T4 |
| Passey maturity slope magnitude | `PASSEY_B` | 0.1688 | 1/LOM | Techlog `TOC_Computation.py`; Geolog `shale_toc_deltalogr.info`; dossier §§2.2/5.2 | T1/T4 |
| Level of maturity | `LOM` | **ABSENT — ships with no default** | dimensionless | Seeds 10.5/10.6/10.403 differ by method; dossier §5.2 | T1/T3 |
| LOM cap | `LOM_CAP` | 10.5 | dimensionless | Geolog manifest with cited justification; dossier §5.2 | T1 |
| Low-LOM warning | `LOM_WARN_LOW` | 6.0 | dimensionless | Techlog commented clamp; dossier §5.2 | T1 |
| VR-to-LOM coefficients, descending | `VR_LOM_COEFFS` | 0.0989, −2.1587, 12.392, −29.032, 32.53, −3.0338 | — | Techlog executable source; dossier §5.2 | T1/T3 |
| Vitrinite reflectance | `VR` | **ABSENT — ships with no default** | — | Techlog seed 0.9 is method-specific; dossier §5.2 | T1 |
| Background TOC | `TOC_BG` | **ABSENT — ships with no default** | wt% | Cited seed 0.8; current 0.0; baseline-specific, dossier §5.2 | T1/T2/T3 |
| Resistivity baseline | `RT_bl` | **ABSENT — ships with no default** | ohm·m | Conflicting picked/seeds; dossier §§2.3/5.2 | T1/T2/T3 |
| Sonic baseline | `DT_bl` | **ABSENT — ships with no default** | µs/ft | Conflicting picked/seeds; dossier §§2.3/5.2 | T1/T2/T3 |
| Density baseline | `RHOB_bl` | **ABSENT — ships with no default** | g/cc | Conflicting picked/seeds; dossier §§2.3/5.2 | T1/T2/T3 |
| Neutron baseline | `NPHI_bl` | **ABSENT — ships with no default** | v/v | Conflicting picked/seeds; dossier §§2.3/5.2 | T1/T2/T3 |
| Final overlay selection | `OVERLAY_FINAL` | **ABSENT — ships with no default** | enum | Sonic/average/density defaults conflict and none is defended; dossier §3.11 | T1/T2/T3 |
| Schmoker numerator | `SCHM_A` | 154.497 | — | Techlog `TOC_Computation.py`; Rider via dossier §5.2 | T1/T4 |
| Schmoker offset | `SCHM_B` | 57.261 | — | Techlog `TOC_Computation.py`; Rider via dossier §5.2 | T1/T4 |
| Clay grain density | `RHO_CLAY` | 2.71 | g/cc | Techlog executable default; dossier §5.2 | T1 |
| QFM grain density | `RHO_QFM` | 2.65 | g/cc | Techlog executable literal; dossier §5.2 | T1 |
| Carbonate grain density | `RHO_CARB` | 2.71 | g/cc | Techlog `TOC_Computation.py`; dossier §5.2 | T1 |
| Pyrite grain density | `RHO_PYR` | 5.00 | g/cc | Techlog executable; Geolog corroboration; dossier §5.2 | T1/T3 |
| Grain-mix default weights | `W_CLAY,W_CARB,W_QFM,W_PYR` | 0.15, 0.40, 0.40, 0.05 | kg/kg | Techlog executable defaults; dossier §5.2 | T1 |
| Pyrite/kerogen slope | `PYR_KER_SLOPE` | 0.135 | dimensionless | Geolog manifest; dossier §5.2 | T1 |
| Pyrite/kerogen intercept | `PYR_KER_INTERCEPT` | 0.0078 | v/v | Geolog manifest; dossier §5.2 | T1 |
| No-organic density | `RHO_NO_KER` | **ABSENT — ships with no default** | g/cc | Input-log seed 2.69 and range 2.6–2.9; dossier §5.2 | T1 |
| Uranium TOC gain | `U_GAIN` | 0.5 | wt%/ppm | Exact `100×0.005` conversion from Techlog fraction gain; dossier §5.2 | derived/T1 |
| Uranium TOC offset | `U_OFFSET` | 0 | wt% | Techlog executable default; dossier §5.2 | T1 |
| Carbon-to-organic-matter factor | `KCF` | 1.2 | dimensionless | Techlog and two Geolog modules; dossier §§2.6/5.2 | T1 |
| Kerogen grain density | `RHO_KER` | 1.10 | g/cc | Like-for-like IP endpoint and matching shipped mineral endpoint; dossier §§3.5/5.2 | T1/T2 |
| TOC-to-Langmuir slope | `TOCLV_GRAD` | **ABSENT — ships with no default** | (scf/ton)/wt% | Lab regression output; dossier §§2.7.1/5.2 | T1 |
| TOC-to-Langmuir intercept | `TOCLV_INT` | 0 | scf/ton | Through-origin calibration constraint; dossier §5.2 | T1 |
| TOC calibration slope | `TOC_GRAD` | 1 | dimensionless | Identity/not-calibrated setting in Geolog manifests; dossier §5.2 | T1 |
| TOC calibration intercept | `TOC_INT` | 0 | wt% | Geolog `SCHM_INTERCEPT`/`TOC_CALIB_INT`/`DLR_*_INT`; dossier §5.2 | T1 |
| GR reservoir-mask cutoff | `SGR_CUT` | 30 | GAPI | Geolog manifest; dossier §5.2 | T1 |
| Neutron-density mask cutoff | `DIFFND_CUT` | 0.4 | — | Geolog `shale_toc_deltalogr.info`; dossier §5.2 | T1 |
| Type-II HI coefficients | `HI_II_COEFFS` | 0.1028, −3.94, 50.4, −290, 960 | mg HC/g TOC | Techlog `TOC_Computation.py`; dossier §5.2 | T1 |
| Type-III HI coefficients | `HI_III_COEFFS` | 0.2914, −11.64, 169.57, −1099, 2863.2 | mg HC/g TOC | Techlog `TOC_Computation.py`; dossier §5.2 | T1 |
| Kerogen type | `TOC_TYPE` | II | enum | Techlog executable default; dossier §5.2 | T1 |
| RockEval carbon coefficient | `SRA_K` | 83 | — | Geolog `geochem_sra_indexes.info` line 100; dossier §5.2 | T1 |
| CO carbon stoichiometry | `C_CO_PPK_TO_PCT` | 12/280 | ppk→wt% | Geolog printed equation; dossier §§2.5/5.2 | T1 |
| CO2 carbon stoichiometry | `C_CO2_PPK_TO_PCT` | 12/440 | ppk→wt% | Geolog `geochem_sra_indexes.info` lines 54/59/62; dossier §5.2 | T1 |
| Pyrolysed-carbon method | `PYRC_METHOD` | `S1+S2` | enum | Geolog manifest default; dossier §5.2 | T1 |
| Residual-carbon method | `REMC_METHOD` | `S4` | enum | Geolog manifest default; dossier §5.2 | T1 |
| Reservoir pressure | `RES_P` | **ABSENT — ships with no default** | psia | Measured-input discipline, dossier §5.2; current uncited seed at `unconventional.rs:267` | T1 |
| Reservoir temperature | `TEMP_F` | **ABSENT — ships with no default** | °F | Measured-input discipline, dossier §5.2; current uncited seed at `unconventional.rs:268` | T1 |
| Langmuir volume | `V_L` | **ABSENT — ships with no default** | scf/ton | Vendor scalar fallback/dialog seeds are not transferable; dossier §5.2 | T1/T2 |
| Langmuir pressure | `P_L` | **ABSENT — ships with no default** | psia | Geolog `LP` fallback and IP dialog seed; dossier §5.2 | T1/T2 |
| Reference TOC for Langmuir volume | `TOC_LANG` | **ABSENT — ships with no default** | wt% | Cited seed 4, paired wt% path; dossier §5.2 | T1 |
| Reference kerogen volume | `VOL_KER_LANG` | **ABSENT — ships with no default** | v/v | Cited seed 0.04, paired volume path; dossier §5.2 | T1 |
| Isotherm measurement temperature | `T_LANG` | 60 | °F | Geolog manifest `TEMP_LANG`; dossier §5.2 | T1 |
| Langmuir-volume temperature coefficient | `K_T_VL` | −0.0027 | 1/°C in log10 | Geolog manifest; dossier §5.2 | T1 |
| Langmuir-pressure temperature coefficient | `K_T_PL` | +0.005 | 1/°C in log10 | Geolog `shale_gas.info`; dossier §5.2 | T1 |
| Temperature-correction default | `OPT_TEMP_COR` | OFF | boolean | Geolog manifest default; dossier §5.2 | T1 |
| Non-combustible gas fraction | `NCGF` | 0.03 | v/v | Geolog manifest, range 0–0.1; dossier §5.2 | T1 |
| Oil saturation seed | `SO` | 0 | v/v | Geolog manifest default; dossier §5.2 | T1 |
| Gas z-factor | `Z` | **ABSENT — ships with no default** | dimensionless | Cited seed 1.1; measured/PVT property, dossier §5.2 | T1 |
| Standard pressure | `P_SC` | 14.696 | psia | Held SandiBumi reference; alternative documented in Geolog, dossier §5.2 | T1/T4 |
| Standard temperature | `T_SC_F` | 60 | °F | Held SandiBumi reference and Geolog `SHT`; dossier §5.2 | T1/T4 |
| Free-gas content constant | `C_GC_FREE` | 32.0368 | scf/ton per (v/v)/(g/cc)/(res-ft³/scf) | Geolog in-file derivation; dossier §§2.7/5.2 | T1 |
| Adsorbed GIP constant | `C_GIP_ADS` | 1.3597e-6 | Bcf per scf/ton·(g/cc)·ft·acre | Geolog in-file derivation; dossier §5.2 | T1 |
| Free GIP constant | `C_GIP_FREE` | 4.3560e-5 | Bcf per (v/v)·ft·acre/(res-ft³/scf) | Geolog in-file derivation; dossier §5.2 | T1 |
| Ambrose conversion constant | `C_AMBROSE` | 1.318e-6 | (v/v)·(g/cc)⁻¹ per (scf/ton)·(g/mol)/(g/cc) | Geolog equation, cross-checked to IP form; dossier §5.2 | T1/T2 |
| Gas molar mass | `M_GAS` | **ABSENT — ships with no default** | g/mol | Seeds 16.043 and 20 represent different fluids; dossier §5.2 | T1/T2 |
| Sorbed-gas density | `RHO_ADS` | **ABSENT — ships with no default** | g/cc | Seeds 0.34/0.37/0.4223 conflict by 24%; dossier §5.2 | T1/T2/T4 |
| Ash fraction | `F_ASH` | **ABSENT — ships with no default** | w/w | Measured-input discipline, dossier §5.2; current uncited seed at `unconventional.rs:273` | T1 |
| Moisture fraction | `F_MOIST` | **ABSENT — ships with no default** | w/w | Measured-input discipline, dossier §5.2; current uncited seed at `unconventional.rs:274` | T1 |
| Measured in-situ gas content | `GC_MEAS` | **ABSENT — ships with no default** | scf/ton | Measured-input discipline, dossier §5.2; current uncited seed at `unconventional.rs:275` | T1 |
| Rickman ductile Young's endpoint | `E_LO` | 1 | Mpsi | IP equation and cited publication; dossier §5.2 | T2/T4 |
| Rickman brittle Young's endpoint | `E_HI` | 8 | Mpsi | IP equation and Rickman et al. SPE 115258; dossier §5.2 | T2/T4 |
| Rickman ductile Poisson endpoint | `NU_LO` | 0.4 | dimensionless | IP equation and Rickman et al. SPE 115258; dossier §5.2 | T2/T4 |
| Rickman brittle Poisson endpoint | `NU_HI` | 0.15 | dimensionless | IP equation and Rickman et al. SPE 115258; dossier §5.2 | T2/T4 |
| Haworth wetness thresholds | `WH_BOUNDS` | 0.5, 17.5, 40 | ratio scale | IP and Geolog agree; dossier §§2.9/5.2 | T1/T2 |
| Haworth balance threshold | `BH_BOUND` | 100 | ratio | IP and Geolog agree; dossier §§2.9/5.2 | T1/T2 |
| Haworth character threshold | `CH_BOUND` | 0.5 | ratio | IP and Geolog agree; dossier §§2.9/5.2 | T1/T2 |
| Wetness denominator mode | `GWR_MODE` | **ABSENT — ships with no default** | enum | Primary denominator unresolved; dossier §3.12/H-D-11; conservative resolution of §5.2/§5.4 conflict | T1/T2/T4 |

Seventy-six rows. **Twenty-three** ship `ABSENT — ships with no default`: maturity/VR, background,
four baselines, final-overlay selection, no-organic density, Langmuir calibration slope, measured
pressure/temperature, four Langmuir reference inputs, z, two Ambrose fluid properties, ash,
moisture, measured gas content and the GWR mode.

---

## 6. Acceptance tests

| Test | Input and operation | Expected value | Source |
|---|---|---|---|
| SB-TOC-T01 | Convert `4 wt%→w/w→wt%` through the typed TOC API | `0.04` then exactly `4`; an untagged scalar refuses | Dossier test 1/§3.14 (T1/T2/T3) |
| SB-TOC-T02 | `HI=200`, `TOC=4 wt%`; compute S2 in percent and fraction forms | Both equal `8 mg/g rock` exactly | Dossier test 2 and §5.1 arithmetic (T1) |
| SB-TOC-T03 | Synthetic DLR/LOM fixture evaluated in wt%, then converted to fraction | Equals the literal Techlog `/100` form to machine precision | Dossier test 3 (T1) |
| SB-TOC-T04 | Pass fraction-tagged `0.04` into RockEval's percent TOC slot | Type refusal; never a 100× index | Dossier test 4/GL-D-2 (T1) |
| SB-TOC-T05 | Same sonic rock supplied in µs/ft and converted µs/m | DLR values agree to machine precision | Dossier test 5 (T1) |
| SB-TOC-T06 | `V_L=100`, `TOC=TOC_LANG=4 wt%` | `V_L_eff=100 scf/ton` exactly; mismatched unit tags refuse | Dossier test 7/BLOCKER-1 (T1) |
| SB-TOC-T07 | Attempt wt% numerator with `VOL_KER_LANG`, then valid v/v pair | Crossed pair refuses; valid pair computes | Dossier test 8 (T1) |
| SB-TOC-T08 | `RT/RT_bl=3`, `DT−DT_bl=20`, `RHOB−RHOB_bl=−0.15`, `NPHI−NPHI_bl=0.10` | All three DLR values are positive; neutron term is `+0.4` | Dossier test 9/§3.2 (T1/T3) |
| SB-TOC-T09 | All logs at baseline, any finite LOM, `TOC_BG=0.8 wt%` | DLR `0`; every overlay TOC `0.8 wt%` exactly | Dossier test 10/§5.1 (T1) |
| SB-TOC-T10 | Increase RT/DT/NPHI separately and decrease RHOB | TOC is monotone in the documented direction | Dossier test 11 (T1) |
| SB-TOC-T11 | LOM `[6,8,9,10,10.5,10.6,11,12]` | Factor `[19.240,8.843,5.995,4.064,3.347,3.219,2.756,1.868]` within `1e-3` | Dossier test 12/§3.3 (T1) |
| SB-TOC-T12 | VR `[0.5,0.6,0.9,1.0,1.2,1.5,2.0]` | LOM `[7.390,8.437,10.403,10.796,11.379,12.085,13.660]` within `1e-3` | Dossier test 13 (T1/T3) |
| SB-TOC-T13 | VR `2.0`, unit DLR, cap on versus off | Capped/uncapped TOC ratio `3.416` to shown precision and `LOM_CAPPED=1` | Dossier test 14/§3.3 (T1) |
| SB-TOC-T14 | Run TOC with any required baseline absent | Refuse with `BASELINE_UNSET`; no house seed substituted | Dossier tests 27/§3.4 (T1/T2/T3) |
| SB-TOC-T15 | Pick one interval with all four curves, persist and reopen | All baseline values, interval and curve revisions round-trip unchanged | Geolog single-depth design; dossier §§2.3/4.1 (T1) |
| SB-TOC-T16 | Evaluate Schmoker at `RHOB=154.497/57.261` and just above | Zero within `1e-4 g/cc`; above is clamped to zero with raw retained | Dossier test 15/§2.4 (T1/T4) |
| SB-TOC-T17 | Compare generalized `kA/kB` and density-deficit forms at `RHO_G=2.70` | Equal to machine precision | Dossier test 16/§2.4 (T1) |
| SB-TOC-T18 | Default four weights in harmonic mix; then set sum `>1` | First equals `1/(0.15/2.71+0.40/2.65+0.40/2.71+0.05/5)` to machine precision; second case refuses | Dossier test 17 (T1) |
| SB-TOC-T19 | `TOC=3 wt%`, `RHOB=2.4`, `KCF=1.2`, `RHO_KER=1.10`, forward then inverse | `V_KER=0.078545…`; recovered TOC `3 wt%` to machine precision | Dossier test 18/§3.5 arithmetic (T1/T2) |
| SB-TOC-T20 | Load default kerogen response metadata | Density `1.10`, neutron `0.6`, sonic `150`; no unrelated density citation | Dossier §3.5/BLOCKER-2 (T1/T2) |
| SB-TOC-T21 | `U=6 ppm`, cited gain/offset | `TOC_U=3 wt% ±1e-6`; non-admissible environment emits warning | Dossier §§2.5/5.1 arithmetic (T1/T4) |
| SB-TOC-T22 | High-separation low-density coal-like fixture with coal mask set | Null TOC plus `MASKED_COAL`, not a large value | Dossier test 25/§2.11 (T1/T3) |
| SB-TOC-T23 | Set badhole mask on otherwise valid fixture | Null TOC plus `MASKED_BADHOLE` | Dossier test 26 (T1) |
| SB-TOC-T24 | Three arbitrary finite overlay TOCs | `MIN≤AVERAGE≤MAX` and `SPREAD=MAX−MIN≥0` exactly | Dossier test 28 (T1) |
| SB-TOC-T25 | Calibration `GRAD=1`, `INT=0` | Output bit-identical to raw prediction | Dossier test 29 (T1) |
| SB-TOC-T26 | `V_KER=0.10`, cited pyrite slope/intercept | `V_PYR=0.135×0.10+0.0078=0.0213 v/v ±1e-6` | Geolog printed parameters; dossier §5.1 (T1) |
| SB-TOC-T27 | Evaluate both HI polynomials at `LOM=10` | Type II `188`; Type III `104.2 mg HC/g TOC` exactly from shown polynomials | Dossier §5.1 (T1) |
| SB-TOC-T28 | Exercise both PYRC and both REMC methods with fixed peaks | Matches `12/280` and `12/440`; replacing with `/28` or `/44` is detected as 10× | Dossier test 23/§2.5 (T1) |
| SB-TOC-T29 | Emit all six RockEval indices | OSI/HI/OI are ppk; GP/PI/HTI dimensionless mass ratios, never v/v | Dossier test 24/GL-D-3 (T1) |
| SB-TOC-T30 | `V_L=100`, `P=P_L=1000` and `P=0` | Adsorbed content `50` and `0 scf/ton` exactly | Langmuir equation; dossier §5.1 (T1/T2) |
| SB-TOC-T31 | `V_L=100`, `TOC=2 wt%`, `TOC_LANG=4 wt%` | `V_L_eff=50 scf/ton` exactly | Dossier §§2.7/5.1 (T1/T2) |
| SB-TOC-T32 | Run gas content without measured pressure, temperature, z, `V_L` or `P_L` | Refuse and list each missing input; no current placeholder appears | Dossier §5.2 parameter discipline (T1/T2/T3) |
| SB-TOC-T33 | `V_L=100`, `P_L=300`, `T=100°F`, `T_LANG=60°F`, correction on | `V_L_eff=87.09636`, `P_L_eff=387.46490` to `1e-5` | Dossier §5.1 equations and shown arithmetic (T1) |
| SB-TOC-T34 | Compare `14.696 psia/60°F` coefficient to documented `14.7/68°F` plus `CF=0.998` | `0.02827949` vs `0.02791415` within `1e-8`; difference `1.3088% ±1e-4%` | Dossier §§2.7/3.7 arithmetic (T1/T4) |
| SB-TOC-T35 | `PHI=.10`, `Sw=.30`, `So=0`, `NCGF=0`, `RHOB=2.4`, `P=3000`, `T=200°F`, `z=.9` | `Bg=0.0055947` within `1e-5`; `GC_FREE=167.0 scf/ton` within `0.15` | Dossier free-gas form; current source fixture `unconventional.rs:419-427` (T1) |
| SB-TOC-T36 | `PHI=.10`, `Sw=.30`, `So=.10`, `NCGF=.03` | `PHI_GFP=.06`; derated gas-filled porosity `.0582` exactly | Geolog printed form; dossier §5.1 (T1) |
| SB-TOC-T37 | Convert Ambrose constants across scf/ton and cm3/g | Agreement better than `0.1%`; implied molar mass `16.03±0.05 g/mol` | Dossier test 19/§2.7 (T1/T2) |
| SB-TOC-T38 | Same fixture with `NCGF=.03`, compare both correction bracketings | Shipped form derates gas-filled porosity, not sorbed correction; difference equals 3% of correction | Dossier test 22/Geolog line 114 (T1) |
| SB-TOC-T39 | Set `DPHI_ADS>PHI_GFP` | Raw negative retained, output guarded at zero, `AMBROSE_OVERSHOOT=1` | Dossier test 22/rule 14 (T1) |
| SB-TOC-T40 | Register scf/ton and Bcf curves with a shared proposed mnemonic | Refuse collision; content is `GC_*`, extensive volume is `GIP_*` | Dossier §3.15/SB-D-5 (T1/T3) |
| SB-TOC-T41 | Compute corrected `GC_FREE`, then areal free GIP | `GIP_FREE=4.356e-5·GC_FREE·RHOB·H·AREA/32.0368` to machine precision | Dossier test 21/GL-D-7 (T1) |
| SB-TOC-T42 | Synthetic isotherm from `V_L=100`, `P_L=50` at `(P,G)=(50,50),(100,66.666…)` | Linearized slope `0.01`, intercept `0.5`; recovered `V_L=100`, `P_L=50`, all within `1e-6` | Dossier §2.7.1 printed algebra (T1) |
| SB-TOC-T43 | Fit/apply TOC-LV calibration at `TOC=0` | `V_L=0` exactly; nonzero intercept requires explicit acknowledgement | Dossier §2.7.1/GL-D-5 (T1) |
| SB-TOC-T44 | `V_L=100`, `P_L=1000`, `GC=50` then `GC≥V_L` | First `PCD=1000 psia`; second refuses | Dossier §5.1; current source fixture `unconventional.rs:459-470` (T1) |
| SB-TOC-T45 | Dry-basis adsorbed content `100`, ash `.10`, moisture `.05` | In-situ content `85 scf/ton ±1e-6`; missing fractions refuse | Dossier §5.1; current source fixture `unconventional.rs:450-457` (T1/T2) |
| SB-TOC-T46 | Evaluate BI at `(E,nu)=(1,.4)` and `(8,.15)` Mpsi | `0` and `1` exactly; metadata says dynamic and [0,1] | Dossier §2.8/Rickman form (T2/T4) |
| SB-TOC-T47 | Supply static moduli with cited dynamic endpoints | Refuse until static endpoints/calibration are supplied | Dossier §2.8 comparison (T1/T2) |
| SB-TOC-T48 | Volumes `Q=.4,Dol=.1,Calc=.2,Clay=.2,Org=.1` | Jarvie `0.4/0.9=0.44444… ±1e-5`; Wang-Gale `0.5/1.0=0.5 ±1e-6` | Dossier §2.8; current equations `unconventional.rs:584-606` (T1/T4) |
| SB-TOC-T49 | Supply combined C4 plus iC4/nC4 as active mappings | Refuse double count | Dossier test 33/vendor warning (T2/T3) |
| SB-TOC-T50 | C1/C2/C3/C4/C5=`5000/1500/1200/700/200 ppm`, run both modes | Compatibility `53.73`; full denominator `41.86` to shown precision; absent mode refuses | Dossier test 30/§3.12 arithmetic (T2/T4) |
| SB-TOC-T51 | Sweep finite WH/BH/CH including exact `0.5/17.5/40/100` boundaries | Exactly one half-open class at every point; boundary goes to upper class | Dossier test 31/H-D-12 (T1/T2) |
| SB-TOC-T52 | Encode the evidenced incumbent eight-class table as `CHARACTERIZATION` | Reproduces five known holes and two overlaps exactly | Dossier test 32/§2.9 (T1) |
| SB-TOC-T53 | Request Pixler/OI/GQR without primary-equation package | Refuse and name the source gate; no compiled inference | Dossier OPEN-U-5/E-3 (T1/T3) |
| SB-TOC-T54 | Request mud-gas normalization with known units but unsourced 5.0028 | Refuse; display geometric 1470.6 mismatch only as evidence | Dossier §2.10/OPEN-9 (T2/T3) |
| SB-TOC-T55 | C1…C5 sum `1000`, measured total gas `1100` | Residual `−100 ±1e-6`, relative residual `−0.090909… ±1e-6`; missing component flags | Dossier §2.9 QC precedent (T4), shown arithmetic |
| SB-TOC-T56 | Change a persisted baseline or `V_L/P_L` pick and reopen compute plus visual | Both consume one identical typed parameter revision; no UI fallback | As-built §3.5; SB-CORE-010 (T1) |
| SB-TOC-T57 | Trigger cap, clamp, mask, missing parameter and overshoot in one run | Stable per-sample codes plus matching run-summary counts and full provenance | Dossier §5.3 rules 5/14 (T1/T2/T3) |
| SB-TOC-T58 | Open a pre-migration project with intensive `GIP_*` scf/ton curves | Preserve old curves/version; create `GC_*` only through acknowledged migration; never reinterpret as Bcf | Dossier §5.5/SB-D-5 (T1) |

Fifty-eight tests cover all forty-three requirements. Every expected number is sourced or includes
the arithmetic that derives it; no baseline, fluid property or unresolved method choice is invented.

---

## 7. Open items, escalations and refusals

### 7.1 Open items

**O-1 — GWR denominator.** The full-denominator and compatibility modes remain distinct until the
named primary paper is held. This blocks only a single-mode implementation.

**O-2 — Tmax to VR.** Coefficients are absent; only direct LOM and the printed VR polynomial may ship.

**O-3 — Low-LOM warning.** The shipped lower bound is supported by a commented source clamp, but the
primary calibration range is not held.

**O-4 — Generalized four-component density TOC.** Pyrite regression parameters are held; the full
compiled equation body is not.

**O-5 — Sorbed density and molar mass.** Three sourced density seeds and two fluid compositions
remain unadjudicated. Both parameters ship absent.

**O-6 — Isotherm estimator.** Linearized and nonlinear fits weight error differently; the product
must compare them on measured desorption data before choosing a default.

**O-7 — Runtime vendor unit conversion.** A manifest cannot establish whether a vendor converts
fraction/percent at module boundaries; this bounds the severity wording, not SandiBumi's design.

**O-8 — Unit-switch UI behavior in one incumbent.** Install metadata proves one range spans two
unit systems but not whether the dialog rescales or rejects.

**O-9 — Final overlay choice.** No incumbent defends its default; SandiBumi ships no preselection.

### 7.2 Escalations

- Acquire the Haworth primary paper to settle the wetness denominator and probable threshold typo.
- Acquire the cited Passey maturity paper to confirm the 10.5 cap, low-end range and overlay guidance.
- Acquire the Ambrose primary paper to settle sorbed density and molar-mass treatment.
- Acquire the cited Tmax/VR publication before enabling Tmax input.
- Acquire the cited Pixler/OI/GQR papers before those extensions ship.
- Acquire the mud-gas normalization paper before using 5.0028.
- A controlled vendor run may settle runtime unit conversion and unit-switch UI behavior; neither is
  required for SandiBumi correctness.

### 7.3 Refusals

**R-1 — SandiBumi will not pass an untagged TOC scalar between modules.** *Instead:* typed wt% with
named fraction conversion (`SB-TOC-001`).

**R-2 — SandiBumi will not ship universal baseline numbers.** *Instead:* a provenance-bearing
interval pick (`SB-TOC-002`, `SB-TOC-006`).

**R-3 — SandiBumi will not implement anonymous density regressions or proprietary polynomial
coefficients.** *Instead:* cited physical forms and user-fitted calibration (`SB-TOC-009`,
`SB-TOC-011`, `SB-TOC-012`).

**R-4 — SandiBumi will not substitute a direct wt%-to-volume factor for `KCF`.** *Instead:* keep
their dimensions and equations distinct (`SB-TOC-014`).

**R-5 — SandiBumi will not simplify `12/280` or `12/440`.** *Instead:* retain the ppk-to-percent
stoichiometry (`SB-TOC-018`).

**R-6 — SandiBumi will not call scf/ton content an areal gas-in-place volume.** *Instead:* reserve
`GC_*` and `GIP_*` by physical quantity (`SB-TOC-025`).

**R-7 — SandiBumi will not cross the TOC and kerogen-volume Langmuir reference pairs.** *Instead:*
one unit-matched path per run (`SB-TOC-020`).

**R-8 — SandiBumi will not adopt the contradictory gas compaction/compressibility factor.**
*Instead:* one z-factor and named standard conditions (`SB-TOC-023`).

**R-9 — SandiBumi will not copy a mud-gas class table with overlaps or holes.** *Instead:* a proved
half-open partition (`SB-TOC-035`).

**R-10 — SandiBumi will not infer equations from compiled vendor behavior.** *Instead:* source-gated
published derivation (`SB-TOC-037`, `SB-TOC-038`).

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

---

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirements | Dossier evidence | Disposition |
|---|---|---|
| SB-TOC-001–010 | §§2.1–2.3, 3.1–3.4, 3.9–3.14, 5.1–5.5 | **ADOPTED/ESCALATED** — typed TOC, overlays, picks, cap, masks, calibration and provenance |
| SB-TOC-011–018 | §§2.4–2.6, 3.5, 4.1, 5.1–5.4 | **ADOPTED/DEFERRED** — density TOC, kerogen, HI/S2 and complete RockEval |
| SB-TOC-019–029 | §§2.7–2.7.1, 3.7–3.8, 3.15, 5.1–5.5 | **ADOPTED/ESCALATED** — typed gas inputs, TOC scaling, free gas, Ambrose, naming and GIP |
| SB-TOC-030–032 | §2.8, §4.1, §5.2 | **ADOPTED** — brittleness basis, scale and method identity |
| SB-TOC-033–039 | §§2.9–2.10, 3.12, 4.1–4.3, 5.4 | **ADOPTED/DEFERRED/ESCALATED** — gas ratios, total/disjoint classification and source gates |
| SB-TOC-040–043 | §§3.14–3.15, 5.3–5.5 | **ADOPTED** — parameter/UI identity, flags and migration |

All forty-three requirement IDs are unique and traced.

### 8.2 Inventory, equation and optimal-choice disposition

| Dossier item | Disposition | Where it went |
|---|---|---|
| §§1.1–1.4 tool inventory and capability matrix | **EVIDENCE-ONLY / ADOPTED** | §§2–3 and requirement statuses; negative cells do not become product claims |
| §§2.1–2.3 overlay, maturity and baseline blocks | **ADOPTED** | SB-TOC-001–010 |
| §§2.4–2.6 density, RockEval and kerogen blocks | **ADOPTED / DEFERRED** | SB-TOC-011–018; compiled four-component body deferred |
| §§2.7–2.7.1 gas-storage and calibration blocks | **ADOPTED / ESCALATED** | SB-TOC-019–029 |
| §2.8 brittleness | **ADOPTED** | SB-TOC-030–032 |
| §§2.9–2.10 mud-gas analysis/normalization | **ADOPTED / DEFERRED / ESCALATED** | SB-TOC-033–039 |
| §2.11 validity limits | **ADOPTED** | SB-TOC-008, SB-TOC-042 |
| §§3.1–3.15 quantified differences | **ADOPTED / REJECTED / ESCALATED** | §§2–7 and discrepancy table below |
| §4.1 method choices | **ADOPTED with explicit exceptions** | Requirements; proprietary fits and unsourced choices refused |
| §4.2 H-D-8…H-D-17 and OPEN-4/-5/-9/-11 | **ADOPTED / REJECTED / ESCALATED** | §7 and ledger below |
| §4.3 GL/TL/IP/SB discrepancy ledger | **ADOPTED / REJECTED / EVIDENCE-ONLY** | §8.3 |
| §5.1 equation forms | **ADOPTED** | §§2,4–6 |
| §5.2 parameter register | **ADOPTED with stricter absence** | §5; `OVERLAY_FINAL` and `GWR_MODE` absent because source does not adjudicate |
| §5.3 defect rules | **ADOPTED** | SB-TOC-001, -010, -042 and test suite |
| §5.4 tests 1–34 | **ADOPTED** | §6, expanded to 58 chapter tests |
| §5.5 shipped-code changes | **ADOPTED / MIGRATION REQUIRED** | SB-TOC-002, -005–006, -015, -019–026, -040–043 |

### 8.3 Discrepancy-ledger disposition

| Dossier item | Disposition |
|---|---|
| H-D-8 / GL-D-2 | **ADOPTED** — typed wt% boundary → SB-TOC-001/T01–T04 |
| H-D-9/H-D-10 | **EVIDENCE-ONLY** — datum seam and cosmetic typo; oil-storage path outside scope |
| H-D-11 | **ESCALATED** — dual denominator modes, no default → SB-TOC-034/O-1 |
| H-D-12 / GL-D-4 | **REJECTED** — overlapping/incomplete vendor tables → SB-TOC-035/T51–T52 |
| H-D-13 / OPEN-6 | **REJECTED** — malformed Langmuir bound; measured inputs absent |
| H-D-14 | **ADOPTED** — harmonic mixing precedent → SB-TOC-012 |
| H-D-15/H-D-16/H-D-17 | **EVIDENCE-ONLY / ADOPTED** — cosmetic subscripts; manifest overrides demo values |
| OPEN-4 / OPEN-5 | **REJECTED** — undocumented weighted overlay and proprietary regressions |
| OPEN-9 / OPEN-11 | **ESCALATED / SUPERSEDED** — normalization constant gated; total classes remove no-case color ambiguity |
| GL-D-1 | **ESCALATED** — probable threshold typo; never copied |
| GL-D-3 | **ADOPTED** — correct ppk labels, correct dimensionless mass-ratio labels → SB-TOC-018 |
| GL-D-5 | **REJECTED** — nonzero capacity intercept → SB-TOC-029 |
| GL-D-6 | **REJECTED** — contradictory extra gas factor → SB-TOC-023 |
| GL-D-7 | **REJECTED/ADOPTED** — inconsistent free-GIP correction rejected; shared identity adopted → SB-TOC-026 |
| TL-D-1 | **EVIDENCE-ONLY** — implicit density inconsistency informs semantic parameter names |
| TL-D-2 | **REJECTED** — unbounded LOM behavior → SB-TOC-005 |
| TL-D-3 | **REJECTED** — unclamped paths → SB-TOC-013/-014/-042 |
| TL-D-4 | **EVIDENCE-ONLY** — suspicious screening cutoff is outside this chapter's cutoff scope |
| IP-D-H18 | **EVIDENCE-ONLY / ESCALATED** — proof/inference distinction preserved in O-8 |
| SB-D-1 | **ADOPTED** — latent neutron-sign fix → SB-TOC-003 |
| SB-D-2 | **ADOPTED** — value retained, provenance corrected → SB-TOC-015 |
| SB-D-3 | **ADOPTED** — baseline defaults removed → SB-TOC-002 |
| SB-D-4 | **ADOPTED** — TOC-scaled Langmuir → SB-TOC-020 |
| SB-D-5 | **ADOPTED** — content/GIP naming migration → SB-TOC-025/-043 |
| SB-D-6 | **ADOPTED** — background TOC absent → SB-TOC-006 |

### 8.4 Gap and escalation disposition

| Dossier gap | Disposition |
|---|---|
| OPEN-U-1 | **ADOPTED** — documentation correction only; 1.10 value retained |
| OPEN-U-2 | **EVIDENCE-ONLY/ESCALATED** — current lower guard kept; primary range requested |
| OPEN-U-3 | **ADOPTED** — no baseline defaults |
| OPEN-U-4 | **DEFERRED** — compiled density-TOC body |
| OPEN-U-5 | **DEFERRED/ESCALATED** — published mud-gas equations required |
| OPEN-U-6 | **DEFERRED** — second VR-to-LOM implementation unavailable; T1 polynomial labeled single-source |
| OPEN-U-7 | **DEFERRED/ESCALATED** — Tmax path blocked |
| OPEN-U-8 | **ESCALATED** — sorbed density ships absent |
| OPEN-U-9 | **EVIDENCE-ONLY** — local negative search closed; compiled capability not reconstructed |
| OPEN-U-10 | **ESCALATED** — final overlay ships absent |
| OPEN-U-11 | **ADOPTED** — wt% retained because it already ships; typed boundary is the correctness control |
| OPEN-U-12 | **ESCALATED** — isotherm estimator explicit |
| OPEN-U-13 | **EVIDENCE-ONLY/ESCALATED** — vendor runtime unit conversion does not affect product design |
| OPEN-U-14 | **EVIDENCE-ONLY/ESCALATED** — UI unit-switch behavior not asserted as fact |
| E-1…E-8 | **ESCALATED by source need / EVIDENCE-ONLY** — mapped in §7.2; E-8's closed negative local-code sweep retained as evidence |

### 8.5 Critique disposition

| Critique block | Chapter disposition |
|---|---|
| BLOCKER-1 | Revised percent/percent Langmuir pairing governs → SB-TOC-001/-020/T06–T07 |
| BLOCKER-2 | 1.10 endpoint retained and correctly compared → SB-TOC-015/T20 |
| MAJOR-1–MAJOR-2 | Restored IP kerogen routes and full-corpus negative discipline are reflected in §2/§8.2 |
| MAJOR-3–MAJOR-4 | Three unclamped branches and corrected 5:4 unit near-tie govern; no old count inherited |
| MAJOR-5 | Unit pairing, not ratio form, governs SB-TOC-020 |
| MAJOR-6 | Full carbon-partition branch governs SB-TOC-018/T28 |
| MAJOR-7 | Contradictory extra gas factor refused by SB-TOC-023 |
| MINOR-1–MINOR-11 | Corrected citations, counts, boundaries, labels, source gates, brittleness scale and Ambrose bracketing all retained |
| Revision-only findings | Unique test IDs, TL-D-4 evidence and non-adopted screening cutoffs accounted for |

### 8.6 Completeness statement

The four-tool inventory, capability matrix, eleven numbered comparison blocks plus §2.7.1, fifteen
difference blocks, every optimal-choice row, H-D/GL-D/TL-D/IP-D/SB-D ledger item, parameter register,
all 34 dossier tests, §5.5 migration list, OPEN-U-1…U-14, critique's two blockers/seven majors/eleven
minors and revision-only discoveries are dispositioned. No vendor chart data, proprietary algorithm,
operator/client asset name, field/block/basin/well/project name or Tier-C material is transcribed.
The Tier-C disposition is recorded in §7.4.
