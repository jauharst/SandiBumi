# 16. Nuclear magnetic resonance — requirements

> **Dossier:** `docs/research_2026-08/cross_tool/nmr.md` — 2,406 lines — read in full 2026-08-08
>
> **Critique:** `docs/research_2026-08/cross_tool/nmr_critique.md` — 662 lines — read in full 2026-08-08
>
> **Evidence tiers held:** T1 executable source/manifests; T1p primary published text; T2 complete
> manual ingest; T3 shipped documentation; T4 held research notes
>
> **Requirements:** 38 · **P0:** 0 · **Parameters:** 42 (16 `ABSENT`) · **Acceptance tests:** 57

The revised dossier governs: it incorporates two blockers, seventeen major findings, seventeen minor
findings and the additional source discoveries made while repairing them. No vendor chart payload,
proprietary lookup table or compiled algorithm is transcribed.

---

## 1. Scope and boundary

This chapter owns the NMR T2-distribution data contract and the interpretations derived from an
already-inverted distribution: porosity partition, cutoff and spectral bound-fluid volumes, T2 log
mean, Timur–Coates and SDR permeability, DMR gas correction, T2-to-capillary-pressure conversion,
MRIAN saturation, fluid-substitution preconditions, provenance and NMR-specific QC.

The first release MUST consume a delivered T2 distribution. It does not own raw echo-train inversion.
That deliberate boundary follows the evidence that raw echoes are normally unavailable to the
interpreter and that the two evidenced inversion implementations are compiled while their complete
regularisation mathematics is not held (`nmr.md` §§2.11, 4.10, T1/T3/T4).

### Named seams

- `SB-CORE-001`: every T2, pressure, density and porosity unit is canonicalized before use.
- `SB-CORE-002`: clamps and exclusions preserve the unclipped value and emit a flag.
- `SB-CORE-006`: method names, equations, curve labels and flags agree.
- `SB-CORE-010`: every parameter and derived curve carries source provenance.
- `11_porosity.md`: density porosity, matrix/fluid density and total/effective porosity semantics.
- `12_saturation.md`: general water-saturation and resistivity equations outside MRIAN.
- `15_sat-height-rocktyping.md`: capillary-pressure scaling, saturation height and rock typing.
- `20_envcorr-qc.md`: acquisition/environmental QC outside NMR-specific provenance.
- `21_data-io.md`: DLIS/LAS/array import, depth reference and unit conversion.
- `22_database-model.md`: array-log identity, axis persistence and result provenance.
- `23_plotting-interactivity.md`: T2-distribution displays and interactive picks.
- `25_fluidsub-rockphysics.md`: non-NMR fluid substitution and elastic properties.

### Explicitly not owned

This chapter does not implement or specify a vendor echo-inversion algorithm, fast-relaxation
correction, proprietary chart, compiled diffusion correlation or malformed undocumented formula.
It records their presence as input provenance or an acquisition gap. It does not turn a fluid
saturation into a clay/shale bulk volume by naming convention.

---

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 The array and partition are the foundation

The three tools consume or create a T2-amplitude vector at every depth. Their first interpretation
is a mass-conserving split into clay-bound water, capillary-bound water and free fluid. One tool can
silently collapse total and effective NMR porosity when a clay-bound option is unticked; another
emits the constituent volumes and saturations separately. The requirement-bearing choice is an
explicit partition whose three volumes always reconcile to total NMR porosity (`nmr.md` §§2.1,
5.1A, T1/T2/T3).

The bin label is not necessarily the bin boundary. The primary text documents nominal centres and
finite intervals, while shipped recognition presets carry a bin-centre convention that differs by
tool family. A cutoff inside a bin therefore requires a declared split rule, not whole-bin rounding
(`nmr.md` §§2.11, 4.10, 5.2, T1-equivalent/T1p/T2).

### 2.2 Cutoffs are cited seeds, never defaults

The evidence contains clay-bound candidates of 3 and 4 ms, sandstone free-fluid evidence of 33 ms
with a documented 12 to greater-than-80 ms spread, and carbonate candidates of 92 and 100 ms. The
partition module with the deepest manifest evidence ships both cutoff fields blank. A downstream
statistics module ships 33 ms, but that does not make it a partition default (`nmr.md` §§2.2, 4.1,
T1/T1p/T3; critique MAJ-4).

Accordingly SandiBumi ships no cutoff preselected. It may present the cited candidates and their
context, but an analyst must accept or calibrate a value and that decision must travel with the
result.

### 2.3 “Tapered” names two equations

The Coates thin-film family used by two tools is `W_i = 1/(m·T2_i+b)`. The third tool uses a
piecewise quadratic taper. Under its own `T_cutoff/4` guidance the quadratic has nearly the same
long-T2 asymptote as the sandstone thin-film constants, but it clamps the short-T2 side to one. The
canonical choice is the primary-literature thin-film family with `min(1, …)` and simultaneous
emission of cutoff and spectral volumes. The final bound volume may take their maximum only when the
documented `b=1` condition holds (`nmr.md` §§2.3, 3.3, 4.4, 5.1B, T1/T1p/T2/T3).

### 2.4 Permeability is dominated by unit and semantic identity

The Timur–Coates equation is algebraically consistent across the primary text and three vendor
implementations when porosity units and coefficient placement are reconciled. The corroborated
canonical v/v form is:

`KTIM = 10000 · PHIE_NMR^4 · (FFI/BVI)^2`.

The evidence also exposes four incompatible exponent-letter conventions and two different vendor
modules whose constants produce a factor 2.04 difference. One manifest describes porosity in
porosity units while declaring its input in v/v, creating a potential `10^8` error in the fourth-
power term. Parameters must therefore be keyed by semantic role and porosity-unit system, never by
single letters (`nmr.md` §§2.4, 3.1–3.2, 5.1C, T1/T1p/T3; critique MAJ-16, m5–m6).

SDR uses T2 log mean and porosity. Its exponents are corroborated, but the only shipped vendor
multiplier and the held literature value differ by a factor 2.5. More importantly, the primary text
states that SDR fails in hydrocarbon-bearing intervals. No numeric multiplier may ship, and the
hydrocarbon gate is mandatory (`nmr.md` §§2.5, 3.5, 4.3, T1/T1p/T3/T4).

### 2.5 T2 log mean is addressed by time, not ordinal

The portable definition is the amplitude-weighted geometric mean, windowed by T2 time in
milliseconds. A bin ordinal is not portable across 12-, 30-, 54-, 64- or 200-component
distributions. Per-bin means and a free-fluid-window mean remain useful outputs, but every window
must retain its physical T2 bounds (`nmr.md` §§2.6, 4.6, 5.1E, T1/T1p/T2/T3).

### 2.6 DMR must use the full equation and measured fluid inputs

The full Freedman density–magnetic-resonance solve and its simplified fixed-weight form are distinct
equations. The simplified form assumes fluid hydrogen index equals one and one vendor makes a 60/40
shortcut its default while withholding several outputs. The full solve is the correct target, with
uncertainty propagation, but the coefficient `lambda` is used in three equations and is undefined in
the held page. That missing primary source blocks the method (`nmr.md` §§2.7, 4.7, 5.1G, E3,
T1/T3/T4; critique m7).

The gas hydrogen index should be computed from the cited density relation, not selected between two
inconsistent fixed constants. The relation `HI_GAS = 2.25·RHO_GAS` is cited by the vendor to the
primary NMR text; fixed 0.5 and 0.4 values remain visible seeds only (`nmr.md` §§2.7, 4.7, T2;
critique MAJ-11).

### 2.7 T2-to-Pc constants and offsets are unit-bearing

The three tools express `Pc ∝ 1/T2` as a pure inverse, a two-segment inverse and a log-linear power
law. Their evidence also contains all three live unit traps: psi·s versus psi·ms (1000×), a manifest
whose factor is tagged per second while its T2 is declared in milliseconds, and one module whose Pc
array is in bar while entry pressure is in psi (14.5038×) (`nmr.md` §§2.8, 3.6, 4.8, T1/T2/T3;
critique BLK-2, MAJ-7, MAJ-9).

`PC_OFFSET` is therefore the typed triple `(value, T2_argument_unit, pressure_output_unit)`. Changing
the T2 argument from ms to s requires `offset_s = offset_ms − 3·gain`; it is not a relabel. No gain,
offset or kappa ships by default.

### 2.8 MRIAN is the canonical NMR saturation path

The revised evidence proves that IP's Z equations reproduce primary-text MRIAN equations 7.3 and
7.9–7.11. The difference is operational: the vendor uses the computed wet/irreducible curves as
picking aids for two constants, whereas MRIAN uses per-level bounds and a measured BVI/effective-
porosity interpolation. MRIAN therefore removes two picked constants from the operative path
(`nmr.md` §§2.9, 3.7a–3.8, 4.9, 5.1I, T1p/T2; critique MAJ-1–MAJ-2).

The clay-water conductivity coefficient remains disputed: the primary text prints 0.000216 and the
vendor raster prints 0.000126, a factor 1.714. Both were re-read at source. Neither may be selected
silently (`nmr.md` §§3.7, 4.9, E1, T1p/T2).

### 2.9 Naming can turn a saturation into a shale volume

One deterministic implementation emits `CBW/PHIT` and `BFV/PHIT` under clay/shale-volume names with
v/v unit tags. These ratios are pore-volume saturations and are already emitted elsewhere under
correct saturation names. Feeding them directly to a bulk-volume solver produces an in-range but
wrong result. SandiBumi must emit `SWB_NMR` and `SWIRR_T_NMR` and must never publish those ratios as
`VCL_*` or `VSH_*` (`nmr.md` §§2.9, 4.11 GL-D-7, T1; critique MAJ-15).

### 2.10 Fluid typing and echo inversion remain source-bound

Time-domain and diffusion-analysis capabilities are described in the primary text, but the complete
vendor algorithms are compiled and several shipped correlations are malformed or unit-contradictory.
These capabilities may be derived later from the published primary literature, never from binary
behaviour. First release records inversion provenance and consumes the delivered distribution
(`nmr.md` §§2.10–2.11, 4.10, E5/E7/E8/E8c, T1/T1p/T3/T4).

---

## 3. SandiBumi as-built

This section was re-verified against current source. The codebase-index server is unavailable in
this session, so each negative source result was confirmed with a targeted repository-wide `rg`, as
required by the repository instructions for a regex-parsed Rust/TypeScript tree.

### 3.1 The array-log foundation

The database has a keyed array-log table with one little-endian f32 vector per depth and an optional
axis blob. The comments explicitly name T2 time as an axis use case (`src-tauri/src/db.rs:259-287`,
T1). The writer validates depth/vector cardinality, rejects duplicate storable depths, writes in one
transaction and stores the axis on every row (`src-tauri/src/db.rs:1153-1217`, T1). Status:
**PRESENT-OK** as a storage primitive.

Wide/block intake commits its numeric header axis with the array (`src-tauri/src/intake.rs:1124-1236`,
T1). The UI can select array curves and render band, spaghetti and value-histogram heatmap modes
(`src/ui/layoutPropsDialog.ts:614-673`; `src/ui/logViewPanel.ts:1054-1221`, T1). Status:
**PRESENT-OK** for generic distributions and Monte Carlo realizations.

### 3.2 The NMR axis is lost on read

`ArrayRow` contains only depth and samples (`src-tauri/src/db.rs:1099-1103`, T1). The read query
selects only `depth, samples`, omitting the stored axis (`src-tauri/src/db.rs:1220-1256`, T1). The IPC
response consequently contains depth, width and values but no axis (`src-tauri/src/lib.rs:695-735`,
T1), and the frontend `ArrayLog` likewise has no axis (`src/ipc.ts:2870-2900`, T1).

The current heatmap bins the **amplitude values** across a user min/max range. An NMR distribution
instead needs amplitude or density displayed at the stored T2-axis position. The current modes
cannot make that scientific plot (`src/ui/logViewPanel.ts:1185-1221`, T1). Status:
**PRESENT-DIVERGENT**.

### 3.3 No NMR interpretation module exists

No NMR-specific compute symbol for partition, spectral BVI, T2LM, DMR, MRIAN or T2-to-Pc is present
in the module/equation sources (targeted `rg`, 2026-08-08, T1 negative search). The existing
`perm_coates` is a generic Swirr-based transform with `CONST_COATES=100` and `SWE_IRR=0.15`; it does
not consume an NMR distribution and is not the dossier's canonical NMR Timur–Coates form
(`src-tauri/src/modules.rs:2444-2476`, T1). Status: **ABSENT** for NMR interpretation.

### 3.4 Spine alignment

The spine accurately says the array-log store exists and NMR is demand-driven after the first-sale
gate (`05_STRATEGY.md` §21; `06_SEQUENCING_AND_GATES.md` §§23–24). It also accurately says full NMR
inversion is absent (`01_PRODUCT.md` §6.1). No NMR-specific spine correction is required.

---

## 4. Requirements

The spine places NMR after the first-sale gate. Consequently this chapter allocates no P0 item; its
P1/P2/P3 priorities apply when the NMR increment is sequenced, and the spine remains authoritative.

### 4.1 Array identity, geometry and provenance

#### SB-NMR-001 — Carry the physical T2 axis through storage, IPC and UI [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Every NMR distribution MUST contain depth, amplitude vector, T2-axis vector, T2 unit, amplitude unit,
bin convention and curve-set identity. Read/IPC paths MUST return the stored axis byte-exact. A
missing axis MUST block NMR interpretation.

**Rationale.** dossier §§2.11, 4.10 (T1/T1p/T2); as-built §§3.1–3.2 (T1).

**As-built.** PRESENT-DIVERGENT — storage writes an axis, but read, IPC and UI omit it (§3.2).

**Verified by.** SB-NMR-T01, SB-NMR-T45

#### SB-NMR-002 — Validate array geometry before accepting a distribution [P1] [status: PARTIAL]

**Requirement.** At each depth, amplitude and T2 axis lengths MUST match. Axis values MUST be finite, positive and
strictly increasing. A bin-count change inside one curve set MUST split the set or refuse; it MUST
NOT be padded and treated as one geometry.

**Rationale.** dossier §§2.11, 5.2 `IP-D-NMR-1` (T1-equivalent/T1p/T2); array-store ragged warning
at `db.rs:1106-1115` (T1).

**As-built.** PARTIAL — the writer checks cardinality and duplicate depth, but not the complete NMR-axis contract (§3.1).

**Verified by.** SB-NMR-T02–T04

#### SB-NMR-003 — Record acquisition and processing provenance [P1] [status: ABSENT]

**Requirement.** The distribution MUST record tool family, acquisition mode, echo spacing, wait time, delivered bin
geometry, inversion software/version, inversion method, regularisation settings, polarization
correction and whether a fast-relaxation-corrected companion was present. Unknown fields remain
unknown; they MUST NOT be inferred from mnemonic alone.

**Rationale.** dossier §§2.11, 4.10, 5.2, E5/E12/E16 (T1/T1p/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T48

#### SB-NMR-004 — Reject defective recognition presets [P1] [status: ABSENT]

**Requirement.** A recognition preset MUST be internally consistent: named bin count equals inclusive bin range,
start/stop T2 exist and mnemonics required for auto-mapping are present. A defective vendor row MUST
be rejected and reported, never repaired with a house number.

**Rationale.** dossier §5.2 `IP-D-NMR-1` and §8 “Beyond the critique” (T1-equivalent).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T05

#### SB-NMR-005 — Log every normalization or rebinning decision [P1] [status: ABSENT]

**Requirement.** Normalization MUST preserve the source distribution, axis and total amplitude; record source and
target axes, interpolation, bin-centre convention and residual. Visual picks MUST be blocked on an
incomplete or unrecognized support until the analyst acknowledges it.

**Rationale.** dossier §§2.11, 4.10, 5.3 rule 15 (T1-equivalent/T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T49

### 4.2 Partition and bound fluid

#### SB-NMR-006 — Cutoffs ship absent and require explicit acceptance [P1] [status: ABSENT]

**Requirement.** `T2C_CBW` and `T2C_FF` MUST have no preselected value. The UI MAY show cited lithology-specific
seeds, their conflicts and documented spread, but a run MUST require analyst acceptance or
calibration and record the source chosen.

**Rationale.** dossier §§2.2, 4.1, 5.2 (T1/T1p/T3); critique MAJ-4.

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T06

#### SB-NMR-007 — Partition is conservative across a cutoff inside a bin [P1] [status: ABSENT]

**Requirement.** The engine MUST compute `CBW`, `BVI_CUT` and `FFI` with a declared log-T2 bin-split rule and MUST
assert `CBW+BVI_CUT+FFI = PHIT_NMR`. It MUST emit all three plus `PHIE_NMR=PHIT_NMR−CBW`.

**Rationale.** dossier §§2.1, 5.1A, test T1 (T1/T1p/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T07–T08

#### SB-NMR-008 — Support values, saddle-point and spectral methods without hiding the branch [P2] [status: ABSENT]

**Requirement.** The method selector MUST distinguish user values, saddle-point search and spectral weighting. A
saddle pick MUST preserve search bounds, guide curve and a unit-typed guide window. An untagged
vendor half-window MUST NOT be imported as a number.

**Rationale.** dossier §§2.1, 4.5, 5.2; critique m14 (T1).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T06, SB-NMR-T10–T11

#### SB-NMR-009 — Implement the cited thin-film spectral weighting [P1] [status: ABSENT]

**Requirement.** The canonical weight MUST be `W_i=min(1,1/(M_SPEC·T2_i+B_SPEC))`, with `M_SPEC` unit `1/ms`.
Sandstone and carbonate parameter sets MUST remain distinct and cited.

**Rationale.** dossier §§2.3, 4.4, 5.1B (T1/T1p/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T09

#### SB-NMR-010 — Emit both cutoff and spectral volumes in one run [P1] [status: ABSENT]

**Requirement.** `BVI_CUT`, `SBVI` and the selected `BVI` MUST all be emitted. `MAXIMUM` MUST report which branch
bound and MUST warn/refuse when `B_SPEC≠1`.

**Rationale.** dossier §§2.1, 2.3, 4.4; critique MAJ-3 (T1/T1p/T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T10

#### SB-NMR-011 — T2 log mean is a time-windowed geometric mean [P1] [status: ABSENT]

**Requirement.** The engine MUST compute `exp(sum(phi_i·ln(T2_i))/sum(phi_i))` on physical T2 bounds in ms. It MUST
emit whole-distribution, free-fluid-window and requested per-bin means without using portable-looking
ordinal defaults.

**Rationale.** dossier §§2.6, 4.6, 5.1E (T1/T1p/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T12–T13

### 4.3 NMR permeability

#### SB-NMR-012 — Timur–Coates parameters are semantic and unit-typed [P1] [status: ABSENT]

**Requirement.** The engine MUST store `C_COATES`, porosity exponent and FFI/BVI exponent by semantic name. Its
internal coefficient MUST carry `{PHI_VV|PHI_PU}` and conversion MUST be derived from
`(100/C_COATES)^exp_phi`. Letter-only parameter files MUST be rejected.

**Rationale.** dossier §§2.4, 3.1–3.2, 5.1C; critique MAJ-16/m5/m6 (T1/T1p/T3/T4).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T14–T18

#### SB-NMR-013 — Guard the BVI denominator without disguising it [P1] [status: ABSENT]

**Requirement.** When `BVI<BVI_MIN`, the calculation MUST use the cited guard, retain original BVI and raise a
binding flag. Output floors or ceilings MUST preserve the unbounded permeability as a companion.

**Rationale.** dossier §§2.4, 3.9, 4.2, 5.1C (T1/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T19

#### SB-NMR-014 — Modified Coates is optional and calibrated [P2] [status: ABSENT]

**Requirement.** The connectivity form MAY be offered. `D_CONN=1` is only the structural identity that reduces it to
standard Timur–Coates; any active `D_CONN≠1` MUST be supplied or calibrated with provenance and MUST
NOT inherit a dialog value as a default.

**Rationale.** dossier §§2.4, 4.2, 5.2; critique MAJ-13 (T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T20–T21

#### SB-NMR-015 — SDR is null and flagged in hydrocarbon-bearing intervals [P1] [status: ABSENT]

**Requirement.** The SDR method MUST require a water-zone validity state. Where hydrocarbon is present or unknown, it
MUST return null plus a reason. The T2 and porosity exponents MAY use their cited values, but the
multiplier MUST remain absent until calibrated.

**Rationale.** dossier §§2.5, 3.5, 4.3, 5.1D (T1/T1p/T3/T4).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T22–T23

#### SB-NMR-016 — Carbonate SDR requires a sourced surface relaxivity [P2] [status: ABSENT]

**Requirement.** The relaxivity form MUST carry `RHO_SURF` in µm/s and the explicit ms-to-s conversion. It MUST
refuse without a measured or cited relaxivity and MUST store the multiplier with the exponent pair
against which it was calibrated.

**Rationale.** dossier §§2.5, 4.3, 5.1D; critique m4 (T1/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T50

#### SB-NMR-017 — Swanson remains a sourced, disabled extension [P3] [status: ABSENT]

**Requirement.** The product MAY add the cited `SBPC_MAX` transform only after the input quantity and pressure/volume
units are pinned from its primary source. The two manifest constants MAY be retained as cited seeds;
the method MUST remain disabled meanwhile.

**Rationale.** dossier §§2.5a, 4.11 GL-D-4, E15 (T1).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T51

### 4.4 DMR gas correction

#### SB-NMR-018 — Full DMR is blocked until lambda is sourced [P2] [status: ABSENT]

**Requirement.** The full Freedman form MUST be implemented from its named primary source. Until `LAMBDA_DMR` is
defined and cited, the module MUST refuse. A fixed 0.6 weight MAY be shown only as an explicitly
selected approximation and MUST NOT suppress gas-saturation or fluid-volume outputs.

**Rationale.** dossier §§2.7, 4.7, 5.1G, E3 (T1/T3/T4).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T24

#### SB-NMR-019 — Gas hydrogen index is computed from gas density [P2] [status: ABSENT]

**Requirement.** Default mode MUST compute `HI_GAS=2.25·RHO_GAS`, echo the implied value and preserve density source.
Fixed-HI mode MAY expose the two cited seeds but MUST preselect neither. Lab-measured HI MUST override
both when provided.

**Rationale.** dossier §§2.7, 4.7, 5.2; critique BLK-1/MAJ-11 (T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T25

#### SB-NMR-020 — DMR propagates input uncertainty and flags clamps [P2] [status: ABSENT]

**Requirement.** DMR MUST emit central, lower, upper and uncertainty curves for corrected porosity, flushed-zone gas
saturation and gas/water volumes. Any `[0,1]` clamp MUST preserve the unclipped result and flag it.

**Rationale.** dossier §§2.7, 3.9, 4.7, 5.2 (T1/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T27

#### SB-NMR-021 — Fluid properties remain measured or explicitly sourced [P1] [status: ABSENT]

**Requirement.** Gas density, fluid hydrogen index, gas T1, matrix density and fluid density MUST have units and
sources. Competing vendor seeds MUST NOT become defaults. Temperature conventions in the gas-T1 and
clay-water relations MUST remain distinct.

**Rationale.** dossier §§2.7, 2.10, 5.2 (T1/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T52

### 4.5 T2 to capillary pressure

#### SB-NMR-022 — Pc gain and offset carry both unit ends [P1] [status: ABSENT]

**Requirement.** `PC_OFFSET` MUST be stored as `(value, PC_T2_UNIT, PC_OUT_UNIT)`. An absent unit tag MUST reject the
parameter. Converting ms→s MUST apply `offset_s = offset_ms − 3·PC_GAIN`; the T2 array MUST NOT be
silently relabelled.

**Rationale.** dossier §§2.8, 4.8, 5.1H; critique BLK-2 (T1/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T30–T31

#### SB-NMR-023 — Kappa is unit-typed and never defaulted [P1] [status: ABSENT]

**Requirement.** `KAPPA` MUST distinguish pressure·s from pressure·ms and MUST convert by 1000 exactly. The conflicting
vendor values and the internally contradictory low-segment values MAY be shown as seeds only.

**Rationale.** dossier §§2.8, 3.6, 4.8, 5.2, E4/E8b (T1/T3/T4).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T28–T29

#### SB-NMR-024 — Every Pc output carries pressure unit, datum and saturation convention [P1] [status: ABSENT]

**Requirement.** Pc arrays, entry pressure and irreducible pressure MUST each carry pressure unit, depth datum,
wetting-phase convention and conversion provenance. The product MUST NOT infer one output's unit
from another output of the same imported module.

**Rationale.** dossier §§2.8, 3.6, 4.8, 5.3 rule 13; critique MAJ-9 (T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T32, SB-NMR-T53

#### SB-NMR-025 — T2-to-Pc requires a water-saturated distribution [P1] [status: ABSENT]

**Requirement.** The method MUST require measured 100%-water data or a provenance-complete pseudo-water result. An
uncorrected hydrocarbon-bearing distribution MUST refuse. The cumulative direction MUST be stored as
long-T2 to short-T2.

**Rationale.** dossier §§2.8, 4.8, 5.1H; critique m12 (T1/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T33

### 4.6 MRIAN saturation

#### SB-NMR-026 — Implement primary-source MRIAN as the canonical NMR saturation method [P2] [status: ABSENT]

**Requirement.** The engine MUST implement primary equations 7.4, 7.5 and 7.9–7.17, compute per-level `WI`/`WW`,
clamp `WQ` between them with wet/irreducible flags and emit `W_EST` as QC only. `W_EST` MUST NOT feed
back into the solve.

**Rationale.** dossier §§2.9, 3.7a, 4.9, 5.1I (T1p/T2); critique MAJ-2.

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T37–T38

#### SB-NMR-027 — Clay-water conductivity coefficient ships absent [P1] [status: ABSENT]

**Requirement.** `CCW_A` MUST have no value. The UI MUST show both verified candidates, their factor-1.714 conflict
and sources. A run MUST require a selected, cited value until the named primary source closes E1.

**Rationale.** dossier §§3.7, 4.9, 5.2, E1 (T1p/T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T35–T36

#### SB-NMR-028 — Keep effective and total irreducible saturation distinct [P1] [status: ABSENT]

**Requirement.** The engine MUST emit `SWIRR_E=BVI/PHIE_NMR` and `SWIRR_T=(CBW+BVI)/PHIT_NMR`. No output, parameter
or alias MAY use bare `SWIRR` where the reference system is not encoded.

**Rationale.** dossier §§2.9, 4.9, 5.1F (T1/T1p/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T54

#### SB-NMR-029 — IP compatibility uses the resolved positive Z slope [P2] [status: ABSENT]

**Requirement.** If an IP-compatible path is offered, `Z(SwT=0)=Z_IRR`, `Z(SwT=1)=Z_WET` and therefore
`Z_SLOPE=Z_WET−Z_IRR`. Imported opposite-sign prose MUST be corrected with a warning. The endpoints
remain explicit sourced inputs, not operative MRIAN defaults.

**Rationale.** dossier §§3.8, 4.9, 5.2; critique MAJ-1 (T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T39–T40

#### SB-NMR-030 — Tortuosity is not silently dropped between saturation paths [P1] [status: ABSENT]

**Requirement.** The MRIAN path MUST reject `A_TORT≠1` because the primary equations contain no tortuosity divisor.
An IP-compatible path MUST expose `A_TORT` with no default and preserve it in provenance.

**Rationale.** dossier §§2.9, 4.9, 5.2; critique MAJ-14 (T1p/T2).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T41

#### SB-NMR-031 — NMR pore-volume ratios never masquerade as shale volumes [P1] [status: ABSENT]

**Requirement.** `CBW/PHIT` and `BFV/PHIT` MUST be named as saturations. Converting either to a bulk-rock volume MUST
be a separate named operation that multiplies by porosity and records the physical interpretation.

**Rationale.** dossier §§2.9, 4.11 GL-D-7; critique MAJ-15 (T1).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T42

### 4.7 Fluid substitution, inversion and presentation

#### SB-NMR-032 — Pseudo-water substitution enforces ordering and water-leg calibration [P2] [status: ABSENT]

**Requirement.** Fluid substitution MUST require a prior spectral-weight result and MUST calibrate against a water
interval before hydrocarbon intervals. The delivered malformed correlation MUST NOT be implemented;
the feature remains absent until its primary method is recovered.

**Rationale.** dossier §§2.10, 4.8, 5.2, E8c; critique MAJ-6 (T1).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T43–T44

#### SB-NMR-033 — Hydrocarbon typing is independently derived from published NMR literature [P3] [status: ABSENT]

**Requirement.** Time-domain and diffusion-analysis capability MUST be derived from the held/published primary NMR
literature with its own equations, parameters and tests. SandiBumi MUST NOT infer compiled vendor
behaviour or consume vendor binaries.

**Rationale.** dossier §§2.10, 4.10, E7 (T1p/T4).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T55

#### SB-NMR-034 — Echo inversion is excluded from first-release NMR [P2] [status: ABSENT]

**Requirement.** The initial NMR increment MUST consume delivered distributions and record inversion provenance. It
MUST NOT expose known vendor regularisation knobs as if SandiBumi implemented their undocumented
algorithms.

**Rationale.** dossier §§2.11, 4.10, E5 (T1/T3/T4); spine `01_PRODUCT.md` §6.1.

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T56

#### SB-NMR-035 — Detect but do not reproduce undocumented fast-relaxation correction [P1] [status: ABSENT]

**Requirement.** Import MUST detect and record the correction companion/status where available and MUST flag
comparisons between corrected and uncorrected distributions. The named compiled algorithm MUST NOT
be reconstructed.

**Rationale.** dossier §§1.2, 2.11, 5.2, E16 (T3); critique MAJ-10.

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T46

#### SB-NMR-036 — NMR heatmaps use the physical T2 axis [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The distribution view MUST plot amplitude or density at each stored T2 position, normally on a log
T2 axis, and MUST show cutoff/spectral overlays without changing data. It MUST NOT reinterpret T2
bins as Monte Carlo realizations or histogram the amplitudes as the horizontal coordinate.

**Rationale.** dossier §§2.1, 2.11 (T1/T1p/T2/T3); as-built §3.2 (T1).

**As-built.** PRESENT-DIVERGENT — the heatmap bins amplitudes as values instead of positioning them on T2 (§3.2).

**Verified by.** SB-NMR-T45

#### SB-NMR-037 — Every output carries method and parameter provenance [P1] [status: ABSENT]

**Requirement.** Each output MUST record input curve-set/axis revision, interval, method, parameter values and sources,
unit conversions, cutoff/spectral branch, guards/clamps and software revision. Derived plots and
exports MUST retain that record.

**Rationale.** dossier §5.3 rules 3/7/8/9/14/15 (T1/T2/T3); `SB-CORE-010`.

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T57

#### SB-NMR-038 — QC flags are explicit curves and run-summary counts [P1] [status: ABSENT]

**Requirement.** Geometry failure, missing cutoff, split bin, denominator guard, output clamp, hydrocarbon invalidity,
solver non-convergence, MRIAN bound, unit conversion and provenance insufficiency MUST have stable
codes, per-level curves where applicable and run-summary counts.

**Rationale.** dossier §§2.9, 3.9, 5.3 rules 6/14 (T1/T2/T3).

**As-built.** ABSENT — no NMR-specific implementation satisfies this obligation (§3.3).

**Verified by.** SB-NMR-T47

---

## 5. Parameters

Every row is byte-exact from dossier §5.2 or a named primary equation. Where sources disagree or the
required source is missing, the value is `ABSENT — ships with no default`. Cited seeds are described
after the table; they are not silently promoted to defaults.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Clay-bound T2 cutoff | `T2C_CBW` | **ABSENT — ships with no default** | ms | `nmr.md` §§2.2/4.1: 3 ms (Geolog/Techlog) conflicts with 4 ms (Coates 1999 Ch.1/Ch.7) | T1/T1p/T3 |
| Free-fluid T2 cutoff | `T2C_FF` | **ABSENT — ships with no default** | ms | `nmr.md` §§2.2/4.1: 33 ms seed with 12…>80 ms spread; carbonate 92 vs 100 ms conflict | T1/T1p |
| Saddle guide window | `SADDLE_WINDOW` | **ABSENT — ships with no default** | unknown (bins or ms) | Geolog `tp_nmr_t2_bfv.info` L87 ships 2 with blank unit; critique m14 | T1 |
| Spectral slope, sandstone | `M_SPEC_SST` | 0.0618 | 1/ms | Geolog `M_SST` default; IP prose; Coates 1999 Eq. 3.24, dossier §§2.3/5.2 | T1/T1p/T2 |
| Spectral intercept, sandstone | `B_SPEC_SST` | 1 | dimensionless | Same sources, Geolog `B_SST` | T1/T1p/T2 |
| Spectral slope, carbonate | `M_SPEC_CARB` | 0.0113 | 1/ms | Geolog `M_CARB` default; IP prose; Coates 1999 Eq. 3.25 | T1/T1p/T2 |
| Spectral intercept, carbonate | `B_SPEC_CARB` | 1 | dimensionless | Same sources, Geolog `B_CARB` | T1/T1p/T2 |
| Bound-volume selection | `BVI_METHOD` | MAXIMUM | enum | Coates 1999 larger-of rule (`b=1`); Geolog suggested workflow; IP method selector, dossier §§2.3/4.4 | T1/T1p/T2 |
| Coates constant | `C_COATES` | 10 | dimensionless | Coates 1999 Ch.6 p.124; algebraically corroborated by Techlog and Geolog, dossier §§2.4/4.2 | T1/T1p/T3 |
| Timur–Coates v/v coefficient | `KTIM_A_VV` | 10000 | coefficient for v/v porosity | Derived exactly from `C_COATES=10`; corroborated by Techlog/Geolog, dossier §2.4 | derived/T1/T3 |
| Timur–Coates FFI/BVI exponent | `KTIM_EXP_RATIO` | 2 | dimensionless | Techlog default; Geolog `COATES_A`; Coates 1999 Eq. 3.28 | T1/T1p/T3 |
| Timur–Coates porosity exponent | `KTIM_EXP_PHI` | 4 | dimensionless | Techlog default; Geolog `COATES_B`; Coates 1999 Eq. 3.28 | T1/T1p/T3 |
| BVI denominator guard | `BVI_MIN` | 0.02 | v/v | Techlog `BFV_MIN` default, dossier §§2.4/5.2 | T3 |
| Timur–Coates output floor | `KTIM_FLOOR` | 1×10^-6 | mD | Techlog documented floor, dossier §§2.4/5.2 | T3 |
| Modified-Coates identity setting | `D_CONN` | 1 | dimensionless | Algebraic identity `D_CONN=1 ⇒ KTIM`; not a vendor default, dossier §5.2/critique MAJ-13 | structural/T2 |
| Active Modified-Coates connectivity | `D_CONN_ACTIVE` | **ABSENT — ships with no default** | dimensionless | No corroborated default; dossier §§2.4/4.2 | — |
| SDR T2 exponent | `KSDR_EXP_T2` | 2 | dimensionless | Geolog `SDR_A`; Coates 1999 Eq. 3.29 | T1/T1p |
| SDR porosity exponent | `KSDR_EXP_PHI` | 4 | dimensionless | Geolog `SDR_B`; Coates 1999 Eq. 3.29 | T1/T1p |
| SDR multiplier | `KSDR_C` | **ABSENT — ships with no default** | mD·ms^-2 at exponent 2 | Geolog ships 10; held literature gives 4.0; factor 2.5 conflict, dossier §3.5 | T1/T4 |
| Carbonate surface relaxivity | `RHO_SURF` | **ABSENT — ships with no default** | µm/s | Techlog carbonate SDR declares unit but no default, dossier §2.5 | T3 |
| Swanson constant | `SWAN_CONST` | 355 | source-unit dependent | Geolog `tp_nmr_permeability.info` L103; disabled until `SBPC_MAX` units close | T1 |
| Swanson exponent | `SWAN_EXP` | 2.005 | dimensionless | Geolog `tp_nmr_permeability.info` L104 | T1 |
| Gas-HI relation coefficient | `HI_GAS_RHO_COEFF` | 2.25 | cm³/g | IP Basic Log Analysis, attributed there to Coates et al. 1999; dossier §§2.7/4.7 | T2 |
| Gas density | `RHO_GAS` | **ABSENT — ships with no default** | g/cm³ | Vendor seeds 0.15 and 0.2 conflict and are not mutually consistent with their HI constants | T1/T3 |
| Flushed-fluid hydrogen index | `HI_FL` | 1.0 | dimensionless | Geolog `HI_FL` default, dossier §§2.7/5.2 | T1 |
| Gas longitudinal relaxation time | `T1_GAS` | **ABSENT — ships with no default** | s | 4 s seed vs computed relation whose result unit is unstated; dossier §5.2 | T1/T2/T3 |
| DMR coupling coefficient | `LAMBDA_DMR` | **ABSENT — ships with no default** | dimensionless | Used but undefined in Techlog page; Freedman primary source required, dossier E3 | — |
| DMR shortcut weight | `W_DMR_APPROX` | 0.6 | dimensionless | Techlog SixtyForty approximation, dossier §§2.7/5.2 | T3 |
| Pc gain | `PC_GAIN` | **ABSENT — ships with no default** | dimensionless | IP prints form but no default, dossier §§2.8/5.2 | T2 |
| Pc offset | `PC_OFFSET` | **ABSENT — ships with no default** | `(value,T2 unit,pressure unit)` | IP prints form but neither argument/output unit nor default; dossier §§4.8/5.1H | T2 |
| Inverse Pc coefficient | `KAPPA` | **ABSENT — ships with no default** | pressure·time | Geolog 3 and Techlog 4 psi·s conflict; Geolog manifest has 1000× ambiguity, dossier §§3.6/E8b | T1/T3 |
| Pc entry convention | `SWET_ENTRY` | 85 | percent wetting-phase saturation | Techlog `PC_ENT` definition, dossier §§4.8/5.2 | T3 |
| MRIAN interpolation intercept | `W_A` | 1.65 | dimensionless | Coates 1999 Eq. 7.14, read at source | T1p |
| MRIAN interpolation slope | `W_B` | 0.4 | dimensionless | Coates 1999 Eq. 7.14, read at source | T1p |
| Clay-water conductivity coefficient | `CCW_A` | **ABSENT — ships with no default** | relation coefficient with T in °F | 0.000216 Coates Eq. 7.2 vs 0.000126 IP raster; factor 1.714, dossier NMR-X1/E1 | T1p/T2 |
| IP-path Z endpoints | `Z_WET`, `Z_IRR` | **ABSENT — ships with no default** | dimensionless | IP prose seeds 2.0/1.6 and worked 2.0/1.7; compatibility inputs only | T2 |
| IP-path Z slope sign | `Z_SLOPE_SIGN` | `+(Z_WET−Z_IRR)` | structural | IP endpoint definition at `nmrinterpretation.htm` L1830–1831; dossier NMR-X2 | T2 |
| IP-path tortuosity | `A_TORT` | **ABSENT — ships with no default** | dimensionless | IP evidence is dialog-only; MRIAN has no `a`, dossier §5.2/critique MAJ-14 | —/T2 |
| Pseudo-water iteration cap seed | `PW_ITERATIONS` | 30 | iterations | Geolog `tp_nmr_t2_fluid_subs.info`, disabled with malformed method | T1 |
| Pseudo-water flatness seed | `PW_FLATNESS` | 3 | source-defined | Same manifest; disabled pending primary method | T1 |
| Pseudo-water scalar | `PC_SCALAR` | **ABSENT — ships with no default** | unknown | Manifest value 1.5 has unrecoverable role because its printed equation is malformed, dossier GL-D-8/E8c | T1 |
| NMR axis canonical unit | `T2_AXIS_UNIT` | ms | milliseconds | Dossier §5.1 canonical unit; all sanctioned internal equations declare ms | spec/T1/T2/T3 |

Forty-two rows. **Sixteen** read `ABSENT — ships with no default`. They cover both cutoffs, the
untagged saddle window, active Modified-Coates connectivity, SDR multiplier, carbonate relaxivity,
gas density/T1, DMR lambda, Pc gain/offset/kappa, disputed MRIAN conductivity, IP compatibility
endpoints/tortuosity and the malformed pseudo-water scalar (the combined rows yield sixteen absent
table entries). Values recorded for disabled extensions are seeds with an explicit implementation
gate; they are not active defaults.

The cutoff seed set is retained outside the Value column exactly as the contract requires:
`T2C_CBW` candidates 3 and 4 ms; sandstone `T2C_FF` seed 33 ms with documented 12…>80 ms spread;
carbonate candidates 92 and 100 ms. None is preselected. Fixed gas-HI seeds 0.5 and 0.4 likewise
remain visible but unselected. No location-specific project value is carried into this chapter.

---

## 6. Acceptance tests

| Test | Input and operation | Expected value | Source |
|---|---|---|---|
| SB-NMR-T01 | Persist/read axis `[0.3,3,30,300]` with one amplitude row | Returned axis is byte-identical and unit=`ms` | As-built storage contract `db.rs:259-287`; SB-NMR-001 (T1) |
| SB-NMR-T02 | Import amplitude length 3 with axis length 4 | Refuse before write; zero rows committed | Dossier §2.11; SB-NMR-002 (T1/T2) |
| SB-NMR-T03 | Import axis `[0.3,3,3,30]` | Refuse non-increasing axis | Dossier §2.11; SB-NMR-002 (T1/T2) |
| SB-NMR-T04 | Change bin count inside one curve set | Split/refuse; never pad as one geometry | Dossier §2.11; `db.rs:1106-1115` (T1/T2) |
| SB-NMR-T05 | Load a recognition row whose named bin count disagrees with its inclusive range | Reject and name the contradiction | Dossier `IP-D-NMR-1` (T1-equivalent) |
| SB-NMR-T06 | Run partition with either cutoff absent | Run refuses and lists cited seeds without selecting one | Dossier §§2.2/4.1 (T1/T1p/T3) |
| SB-NMR-T07 | Partition any positive distribution, including cutoff inside a bin | `CBW+BVI_CUT+FFI=PHIT_NMR` within 1×10^-12 | Dossier test T1/§5.1A (T1/T1p/T2) |
| SB-NMR-T08 | Place cutoff exactly on a bin edge | No double count; conservation still holds | Dossier test T1 (T1/T1p/T2) |
| SB-NMR-T09 | Spectral sandstone at `T2=100 ms` | `W=min(1,1/(0.0618×100+1))=0.139275…` | Dossier §§2.3/3.3; shown arithmetic (T1/T1p/T2) |
| SB-NMR-T10 | `BVI_METHOD=MAXIMUM` with `B_SPEC=1` | Emit cutoff, spectral and max values plus binding branch | Dossier §§2.3/4.4 (T1/T1p/T2) |
| SB-NMR-T11 | Same with `B_SPEC≠1` | Warn/refuse maximum rule | Dossier §4.4/test T16 (T1p) |
| SB-NMR-T12 | Geometric distribution rebinned to 12, 30 and 64 log bins | T2LM agrees within 0.5% | Dossier test T8 (T1/T1p/T3) |
| SB-NMR-T13 | Compute T2LM using equivalent time bounds but different ordinal grids | Physical-time results agree; ordinal-only import refuses | Dossier §§2.6/4.6 (T1/T2/T3) |
| SB-NMR-T14 | `PHIE=0.25`, `FFI/BVI=2`, canonical KTIM | `156.25 mD` | Dossier test T5/§3.1 arithmetic (T1/T1p/T3) |
| SB-NMR-T15 | Same with alternative coefficient 4900 | `76.5625 mD`; ratio to canonical `2.0408` | Dossier test T6/§3.1 (T1) |
| SB-NMR-T16 | Compute KTIM in v/v and porosity-unit forms | Results agree to machine precision | Dossier test T2/§5.1C (T1/T1p/T3) |
| SB-NMR-T17 | Load IP/Techlog/Geolog letter-shaped exponent files | Semantic mappings all agree; letter-only file errors | Dossier test T3/§3.2 (T1/T2/T3/T4) |
| SB-NMR-T18 | Compare FFI/BVI and `(1−SWIRR_E)/SWIRR_E` forms | KTIM values are identical | Dossier test T4 (T1p) |
| SB-NMR-T19 | Run KTIM with `BVI=0` | Apply 0.02 guard, retain source zero, emit flag; no infinity | Dossier test T11/§3.9 (T3) |
| SB-NMR-T20 | Set `D_CONN=1` in Modified Coates | Result equals standard KTIM exactly | Dossier §5.1C2 (T2/derived) |
| SB-NMR-T21 | Request active Modified Coates without calibrated `D_CONN` | Refuse | Dossier §4.2/critique MAJ-13 (T2) |
| SB-NMR-T22 | Run SDR where `HC_FLAG=true` | Null plus invalid-method flag | Dossier test T10/§4.3 (T1p) |
| SB-NMR-T23 | Run SDR without multiplier | Refuse; show 10 and 4.0 conflict without preselection | Dossier §3.5 (T1/T4) |
| SB-NMR-T24 | Run DMR without `LAMBDA_DMR` | Refuse | Dossier test T15/E3 (T3) |
| SB-NMR-T25 | `RHO_GAS=0.2 g/cm³` in computed-HI mode | `HI_GAS=0.45` and source relation is recorded | Dossier §2.7/§4.7 arithmetic (T2) |
| SB-NMR-T26 | Use fixed 0.6 DMR approximation | Label approximation and still emit gas-saturation output | Dossier §§2.7/4.7 (T3) |
| SB-NMR-T27 | DMR produces value outside `[0,1]` | Preserve raw value, clamp display/output companion, emit flag | Dossier §§2.7/3.9 (T1) |
| SB-NMR-T28 | `KAPPA=3 psi·s`, `T2=100 ms` | `Pc=30.0 psi` | Dossier test T9 (T1) |
| SB-NMR-T29 | `KAPPA=3000 psi·ms`, same T2 | Same `30.0 psi`; bare `3000` errors | Dossier test T9 (T1/T4) |
| SB-NMR-T30 | `gain=1`, `offset_ms=3.4771`, convert ms→s and evaluate at 100 ms/0.1 s | `offset_s=0.4771`; both forms return 30.0 psi to the source's shown precision | Dossier test T9b/BLK-2 (T2) |
| SB-NMR-T31 | `gain=0.5`, convert any offset ms→s | `offset_s=offset_ms−1.5`, not `offset_ms−3` | Dossier test T9b (T2) |
| SB-NMR-T32 | Convert identical Pc bar→psi→bar | Ratio is 14.5038 and round trip is within float tolerance | Dossier test T9c/critique MAJ-9 (T3) |
| SB-NMR-T33 | T2-to-Pc on uncorrected HC distribution | Refuse with water-saturation precondition | Dossier §§2.8/4.8 (T1/T3) |
| SB-NMR-T34 | Cumulative saturation on axis `[long…short]` | Highest T2 starts at Sw=1; direction persists | Dossier §2.8/critique m12 (T1/T2) |
| SB-NMR-T35 | Evaluate both `CCW_A` candidates at 200°F | 0.000216→27.89 mho/m; 0.000126→16.27 mho/m | Dossier test T12/NMR-X1 (T1p/T2) |
| SB-NMR-T36 | Build with a literal hard-coded CCW candidate | Characterization test fails | Dossier test T12 (T1p/T2) |
| SB-NMR-T37 | MRIAN with `WQ>WW` then `WQ<WI` | Clamp to WW/WI and flag WET/IRREDUCIBLE respectively | Coates Eq. 7.13; dossier test T14 (T1p) |
| SB-NMR-T38 | Evaluate `W_EST` after convergence | Reproduces operative W unless a clamp/solver diagnostic is present | Coates Eq. 7.9; dossier §5.1I (T1p) |
| SB-NMR-T39 | IP compatibility with `Z_IRR=1.6`, `Z_WET=2.0`, `SwT=0.5` | `Z=1.8`; `(0.25×0.5)^Z=0.02366` | Dossier test T13/NMR-X2 (T2) |
| SB-NMR-T40 | Import opposite-sign Z sentence | Correct to positive endpoint slope with warning | Dossier §3.8 (T2) |
| SB-NMR-T41 | Set `A_TORT≠1` on MRIAN path | Refuse; parameter is not silently ignored | Coates Eqs. 7.4/7.9–7.11; critique MAJ-14 (T1p/T2) |
| SB-NMR-T42 | Import a curve named as clay volume whose formula is CBW/PHIT | Map only as saturation or require manual mapping; never Vsh | Dossier GL-D-7/critique MAJ-15 (T1) |
| SB-NMR-T43 | Run pseudo-water without spectral weights or prior water calibration | Refuse both missing preconditions | Dossier §2.10/E8c (T1) |
| SB-NMR-T44 | Attempt to parse the malformed pseudo-water expression | Method remains disabled; no guessed parenthesis | Dossier GL-D-8/E8c (T1) |
| SB-NMR-T45 | Open NMR distribution view with physical axis | Horizontal positions follow stored log-T2 axis, not amplitude histogram | Dossier §§2.11/4.10; as-built §3.2 (T1/T2) |
| SB-NMR-T46 | Compare corrected and uncorrected imported distributions | UI flags not-like-for-like using processing provenance | Dossier E16/critique MAJ-10 (T3) |
| SB-NMR-T47 | Run output with any guard/clamp/non-convergence | Stable per-level flag and run-summary count both exist | Dossier §§3.9/5.3 rule 14 (T1/T2/T3) |
| SB-NMR-T48 | Round-trip an imported distribution with known acquisition/inversion metadata | Every supplied field returns unchanged; omitted fields remain unknown | Dossier §§2.11/4.10 and E5/E12/E16 (T1/T1p/T2/T3) |
| SB-NMR-T49 | Rebin one distribution onto a second declared axis | Preserve the source array and record both axes, interpolation, convention and conservation residual | Dossier §5.3 rule 15 (T1/T2) |
| SB-NMR-T50 | Request carbonate SDR without `RHO_SURF` | Refuse; do not borrow the sandstone SDR multiplier | Dossier §§2.5/4.3 and critique m4 (T1/T3) |
| SB-NMR-T51 | Request Swanson permeability while `SBPC_MAX` units are unresolved | Keep method disabled and name the unresolved source/unit gate | Dossier §§2.5a/4.11 GL-D-4 and E15 (T1) |
| SB-NMR-T52 | Supply measured and vendor-seed fluid properties together | Use measured values and retain, but do not select, the cited seeds | Dossier §§2.7/2.10/5.2 (T1/T2/T3) |
| SB-NMR-T53 | Emit Pc after a bar→psi conversion | Curve metadata states psi, source bar, conversion 14.5038, datum and wetting-phase convention | Dossier §§2.8/4.8 and critique MAJ-9 (T2/T3) |
| SB-NMR-T54 | Compute `SWIRR_E=BVI/PHIE` and `SWIRR_T=BFV/PHIT` on unequal PHIE/PHIT | Emit distinct values under distinct semantic keys; no bare `SWIRR` alias | Dossier §§2.9/4.9/5.1F (T1/T1p/T3) |
| SB-NMR-T55 | Attempt to load a vendor binary as a hydrocarbon-typing implementation | Refuse; only a separately documented published-equation implementation can register | Dossier §§2.10/4.10 and E7 (T1p/T4) |
| SB-NMR-T56 | Submit a raw echo train, then a delivered T2 distribution | Echo inversion reports unsupported scope; delivered distribution enters geometry validation | Dossier §§2.11/4.10 and E5 (T1/T3/T4); spine `01_PRODUCT.md` §6.1 |
| SB-NMR-T57 | Export any computed NMR output from a known fixture | Provenance contains input/axis revision, interval, method, all parameter values/sources, conversions, branch, guards and software revision | Dossier §5.3 rules 3/7/8/9/14/15 (T1/T2/T3); `SB-CORE-010` |

Fifty-seven tests cover all thirty-eight requirements. Every numeric expectation is cited or shows
its arithmetic; no cutoff, fluid property or disputed coefficient is invented.

---

## 7. Open items, escalations and refusals

### 7.1 Open items

**O-1 — `CCW_A` is unresolved.** The two source-read constants differ by 1.714. **Settled by:** the
named dual-water primary source and a third independently documented implementation. Blocks MRIAN.

**O-2 — `LAMBDA_DMR` is undefined.** It appears in three DMR equations. **Settled by:** the named
Freedman primary paper. Blocks full DMR.

**O-3 — Pc factor units and low-segment value remain contradictory.** **Settled by:** the named
Volokitin papers or a controlled vendor run; no value ships.

**O-4 — Echo-inversion control names/defaults are known; their complete mathematics is not.** This
does not block the distribution-consumer release. It blocks any later inversion implementation.

**O-5 — Hydrocarbon-typing math needs a primary-source transcription.** The held primary text is the
allowed route; compiled vendor behaviour is not.

**O-6 — Three shipped fluid-property correlations are malformed or unit-contradictory.** They remain
disabled until a primary source or controlled run settles the form and units.

**O-7 — The pseudo-water correlation has an unclosed parenthesis.** Both possible readings exist in
the dossier; neither is adopted. `PC_SCALAR` remains absent.

**O-8 — The Swanson `SBPC_MAX` unit basis is unknown.** The method stays disabled pending its named
primary paper.

**O-9 — A referenced T2-P20 carbonate permeability model is not printed.** It is not specified here
and needs its published source.

**O-10 — One acquisition family remains absent from the shipped recognition table.** A format
manifest or sample delivery closes geometry recognition; no geometry is invented.

**O-11 — The device/performance gate for NMR arrays is unmeasured.** It is sequenced with the NMR
increment, not promoted to the first-sale gate.

### 7.2 Escalations

The dossier's E-register is preserved without location/client identifiers:

- **E1:** acquire the named dual-water primary to settle `CCW_A`.
- **E3:** acquire the named DMR primary to define `LAMBDA_DMR`.
- **E4:** acquire the Pc-from-T2 primary papers to settle kappa conflicts.
- **E5:** read the held inversion-theory help before any inversion work.
- **E6:** inspect only the printed closed-form chart relationships; transcribe no chart data.
- **E7:** derive TDA/DIFAN from the held primary book, not compiled code.
- **E8/E8a/E8b/E8c:** controlled source/manual/live checks for malformed diffusion, Coates units,
  Pc time units and pseudo-water parentheses.
- **E9/E10:** obtain stated multiplier/fit sources; absence remains the default.
- **E11:** local core NMR plus Pc-to-Swirr calibration is a data acquisition need, not a reading task.
- **E12:** close the remaining acquisition-geometry recognition gap.
- **E13:** read the remaining Pc/integrated-permeability pages.
- **E14/E15:** obtain the T2-P20 and Swanson primary sources.
- **E16:** identify the fast-relaxation paper if capability is ever considered; detection already
  mitigates imported-data risk.

The dossier closes E2 from the vendor page itself: `Z_SLOPE=Z_WET−Z_IRR`. It is not escalated here.

### 7.3 Refusals

**R-1 — SandiBumi will not ship a T2 cutoff by borrowing one location/lithology example.** *Instead:*
cited seeds plus explicit acceptance (`SB-NMR-006`).

**R-2 — SandiBumi will not map a cutoff to a whole bin silently.** *Instead:* declared log-T2 split
and conservation (`SB-NMR-007`).

**R-3 — SandiBumi will not let spectral and cutoff volumes become mutually invisible.** *Instead:*
emit both and the resolved branch (`SB-NMR-010`).

**R-4 — SandiBumi will not key permeability exponents by `a`, `b` or `c`.** *Instead:* semantic keys
and unit tags (`SB-NMR-012`).

**R-5 — SandiBumi will not apply an output clamp to hide a divide-by-zero or unit error.** *Instead:*
guard inputs, preserve raw results and flag (`SB-NMR-013`, `SB-NMR-038`).

**R-6 — SandiBumi will not emit SDR permeability in a hydrocarbon-bearing interval.** *Instead:* null
and flag (`SB-NMR-015`).

**R-7 — SandiBumi will not make the 60/40 DMR shortcut the default or suppress its missing outputs.**
*Instead:* full sourced solve, approximation opt-in (`SB-NMR-018`).

**R-8 — SandiBumi will not treat a Pc offset as unitless.** *Instead:* typed triple and exact
offset conversion (`SB-NMR-022`).

**R-9 — SandiBumi will not inherit implicit bar/psi or s/ms mixing.** *Instead:* explicit units on
every parameter and output (`SB-NMR-023`, `SB-NMR-024`).

**R-10 — SandiBumi will not choose either disputed clay-water conductivity coefficient.** *Instead:*
absence until the primary source closes it (`SB-NMR-027`).

**R-11 — SandiBumi will not inherit the wrong-sign Z-slope sentence.** *Instead:* use the same page's
two-point definition (`SB-NMR-029`).

**R-12 — SandiBumi will not publish a pore-volume saturation as a clay/shale bulk volume.** *Instead:*
correct saturation names and explicit conversion (`SB-NMR-031`).

**R-13 — SandiBumi will not repair an unbalanced vendor equation by guessing parentheses.** *Instead:*
keep the capability absent pending primary evidence (`SB-NMR-032`).

**R-14 — SandiBumi will not treat an NMR amplitude vector as Monte Carlo realizations.** *Instead:*
render against its stored physical T2 axis (`SB-NMR-036`).

**R-15 — SandiBumi will not reproduce an undocumented fast-relaxation algorithm.** *Instead:* detect
and preserve processing provenance (`SB-NMR-035`).

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

---

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirements | Evidence | Disposition |
|---|---|---|
| SB-NMR-001–005 | Dossier §§2.11, 4.10, 5.2–5.3 | **ADOPTED** — array geometry, provenance, preset validation and normalization |
| SB-NMR-006–011 | §§2.1–2.3, 2.6, 4.1, 4.4–4.6, 5.1A/B/E | **ADOPTED** — partition, cutoffs, spectral BVI and T2LM |
| SB-NMR-012–017 | §§2.4–2.5a, 3.1–3.5, 4.2–4.3, 5.1C/D | **ADOPTED/DEFERRED** — permeability families, guards and source-gated extensions |
| SB-NMR-018–021 | §§2.7, 4.7, 5.1G | **ADOPTED/ESCALATED** — DMR contract, gas HI and missing primary coefficient |
| SB-NMR-022–025 | §§2.8, 3.6, 4.8, 5.1H | **ADOPTED/ESCALATED** — Pc forms, units and unresolved constants |
| SB-NMR-026–031 | §§2.9, 3.7–3.8, 4.9, 5.1F/I | **ADOPTED/ESCALATED** — MRIAN, disputed coefficient, naming and compatibility |
| SB-NMR-032–038 | §§2.10–2.11, 4.10, 5.3 | **ADOPTED/DEFERRED** — substitution, typing, inversion boundary, display, provenance and QC |

All thirty-eight requirement IDs are unique and traced.

### 8.2 Method inventory and equation blocks

| Dossier block | Disposition | Where it went |
|---|---|---|
| §1.1 IP partition, spectral, permeability, Z/MRIAN, LHC, T2-wet, Pc and normalization | **EVIDENCE-ONLY / ADOPTED** | §§2.1–2.10; SB-NMR-005–016, -019, -022–035 |
| §1.2 Techlog inversion/QC, partition, bin porosity, DMR and Pc | **EVIDENCE-ONLY / ADOPTED** | §§2.1–2.10; provenance, DMR, Pc and refusal requirements |
| §1.3 Geolog 45-module inventory and deterministic sources | **EVIDENCE-ONLY / ADOPTED / REJECTED** | §§2–3; manifest obligations, source-gated extensions and refusals |
| §1.4 primary literature | **ADOPTED** | Canonical spectral, KTIM, MRIAN and acquisition obligations |
| §§2.1–2.11 | **ADOPTED / DEFERRED / REJECTED** | Each equation/assumption block maps in §8.1; no method block omitted |
| §5.1 canonical forms | **ADOPTED / ESCALATED** | §§2, 4, 7.1–7.2 and tests T07–T44 |
| §5.2 parameter table | **ADOPTED / ESCALATED** | §5; every unresolved value ships absent |
| §5.3 defect-catalogue rules | **ADOPTED** | SB-NMR-001–038 and T47/T49/T57 |
| §5.4 dossier tests T1–T18 | **ADOPTED** | §6 and §8.5 |

### 8.3 Difference, ledger and open-item disposition

| Named item | Disposition |
|---|---|
| H-D-4 / H-OPEN-7 | **ESCALATED** — cutoffs absent, cited seeds only → SB-NMR-006/§7.2 |
| H-D-5 / H-OPEN-3 | **ADOPTED/ESCALATED** — KTIM roles corroborated; connectivity remains calibrated → SB-NMR-012/-014 |
| H-D-6 | **REJECTED** — wrong citation year not inherited; primary/book citation used |
| H-D-7 | **REJECTED** — malformed polarization raster refused; correct sourced form required |
| NMR-X1 | **ESCALATED** — `CCW_A` conflict → SB-NMR-027/O-1 |
| NMR-X2 | **ADOPTED** — positive slope resolved → SB-NMR-029/T39–T40 |
| NMR-X3 | **ADOPTED** — MRIAN provenance/equation correction → SB-NMR-026 |
| GL-D-1 | **ADOPTED** — two Coates constants surfaced → SB-NMR-012/T14–T17/O-3 |
| GL-D-2 / GL-D-6 | **REJECTED/ESCALATED** — malformed fluid correlations and pressure-unit contradiction → O-6/R-13 |
| GL-D-3 | **ADOPTED** — porosity-unit 10^8 trap → SB-NMR-012/T16 |
| GL-D-4 | **DEFERRED** — unprinted T2-P20 model → O-9 |
| GL-D-5 | **ESCALATED** — Pc factor s/ms contradiction → SB-NMR-022/-023/O-3 |
| GL-D-7 | **REJECTED** — saturation-as-volume naming → SB-NMR-031/T42 |
| GL-D-8 | **REJECTED/ESCALATED** — malformed pseudo-water correlation → SB-NMR-032/T43–T44 |
| TL-D-1 / REF-D-1 | **ADOPTED** — four exponent conventions → SB-NMR-012/T17 |
| TL-D-2 | **REJECTED** — SDR metadata exponent swap → SB-NMR-015/-016 |
| TL-D-3 / TL-D-4 / TL-D-5 | **ADOPTED/REJECTED** — typed units and corrected mnemonic semantics → SB-NMR-023/-024/-031 |
| IP-D-NMR-1 | **REJECTED** — defective acquisition presets → SB-NMR-004/T05 |

| Dossier gaps | Disposition |
|---|---|
| E1 | **ESCALATED** → §7.2; blocks `CCW_A` |
| E2 | **ADOPTED** — closed positive Z slope → SB-NMR-029 |
| E3 | **ESCALATED** → §7.2; blocks `LAMBDA_DMR` |
| E4 | **ESCALATED** → §7.2; Pc primary papers |
| E5 | **DEFERRED** → SB-NMR-034/O-4; does not block distribution consumption |
| E6 | **DEFERRED** — closed-form equations only; chart nodes remain prohibited |
| E7 | **DEFERRED** → SB-NMR-033/O-5; independent published derivation only |
| E8 | **ESCALATED** → §7.2; malformed diffusion relationship |
| E8a | **ESCALATED** → §7.2; Coates porosity units |
| E8b | **ESCALATED** → §7.2; Pc time units |
| E8c | **ESCALATED** → §7.2; pseudo-water parentheses |
| E9 | **ESCALATED** → §7.2; multiplier source |
| E10 | **ESCALATED** → §7.2; fit source |
| E11 | **DEFERRED** — local NMR/Pc calibration data acquisition |
| E12 | **ESCALATED** → §7.2/O-10; remaining acquisition geometry |
| E13 | **ESCALATED** → §7.2; remaining source pages |
| E14 | **ESCALATED** → §7.2/O-9; T2-P20 primary source |
| E15 | **ESCALATED** → §7.2/O-8; Swanson primary source |
| E16 | **DEFERRED** → SB-NMR-035/T46; detection mitigates imported-data risk |

### 8.4 Optimal-choice and parameter disposition

Dossier choices §§4.1–4.11 are adopted as follows: cutoff absence (`SB-NMR-006`), canonical
Timur–Coates with semantic/unit keys (`-012`), water-gated SDR (`-015`), thin-film spectral BVI and
one-pass maximum (`-009`/`-010`), three-way cutoff method (`-008`), physical-time T2LM (`-011`),
full-source-blocked DMR (`-018`), typed general Pc form (`-022`–`-025`), canonical MRIAN (`-026`–
`-031`) and delivered-distribution-first scope (`-034`). Every dossier §5.2 parameter is either in
§5, represented as provenance-only metadata, routed to its owning seam or deliberately omitted when
it is a vendor chart-derived correlation outside the NMR compute contract. Nothing is silently
adjudicated.

### 8.5 Acceptance-test disposition

Dossier tests T1–T18, including T9b and T9c, map to chapter tests T07–T47. Additional tests T01–T06
cover the current axis-loss and recognition-store seam; T48–T57 close direct verification of
metadata, source-gated extensions, semantic outputs, scope and provenance. Every chapter test names
its input, operation, expected value and source.

### 8.6 Critique disposition

| Critique block | Chapter disposition |
|---|---|
| BLK-1 | Uses the corrected revised dossier; gas-HI primary attribution preserved, fabricated negative absent |
| BLK-2 | Unit-bearing Pc offset → SB-NMR-022/T30–T31; the dossier's worked T9b conversion (`offset_s=offset_ms−3·gain`) governs its stray plus-sign prose |
| MAJ-1–MAJ-2 | Z sign closed and Eq. 7.9 restored → SB-NMR-026/-029 |
| MAJ-3–MAJ-4 | Mutually exclusive volumes and blank partition defaults corrected → SB-NMR-006/-010 |
| MAJ-5–MAJ-10 | Inversion metadata, malformed correlations, unit contradictions and fast-relaxation provenance all dispositioned |
| MAJ-11–MAJ-17 | Computed gas HI, permeability scope, structural `d`, tortuosity, saturation naming and canonical equation repair all carried |
| m1–m17 | Revised counts, citations, units, output sets, direction, preset defects and echo-train wording govern; none is reverted |

### 8.7 Completeness statement

The inventories, eleven numbered comparison sections plus §2.5a, ten numbered difference sections
plus §§3.1a/3.7a, eleven optimal-choice blocks, parameter register, eighteen dossier tests, E1–E16
register, ledger items, two critique blockers, seventeen majors, seventeen minors and four
beyond-critique discoveries are all dispositioned. No
vendor chart data or proprietary file content is transcribed. No operator, client, field, block,
basin, well or project name appears. No Tier-C item falls in this domain.
