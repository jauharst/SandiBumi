# 23. Plotting, display and interactivity — requirements

**Status:** PRD v2 domain chapter
**Evidence dossier:** `docs/research_2026-08/cross_tool/plotting-interactivity.md`
**Adversarial companion:** `docs/research_2026-08/cross_tool/plotting-interactivity_critique.md`
**Requirement prefix:** `SB-PLT`
**Authoring contract:** `docs/PRD_v2/CONTRACT.md`

This chapter is written from both local evidence files named above. The revised dossier incorporates
the critique's blocker, eleven major findings and all reproducible minor findings; where the first
critique and the revised dossier differ, the revised dossier governs. Vendor chart payloads are not
reproduced here. Only chart purpose, schema defects and provenance obligations are stated.

---

## 1. Scope and boundary

This chapter owns the visual-analysis shell: plot binding, axes, binning, statistics, overlay
compatibility, linked selection, faceting, rendering, interaction, plot templates and plot export.
It also owns the integrity contract between the displayed sample and any value written from a plot.

It does **not** own the petrophysical equation behind a plotted curve. Pickett/Hingle rendering is
specified here; formation-factor and saturation parameters remain with `12_saturation.md`. Pressure-
gradient visualization is specified here; the pressure and fracture-gradient methods remain with
`18_geomech-ppfg.md`. Curve normalization math and environmental validity ranges remain with
`20_envcorr-qc.md`. Depth-grid construction and resampling remain with `21_data-io.md`. Persistent
plot objects and their foreign keys remain with `22_database-model.md`.

The boundary rule is simple: if the obligation changes a scientific value, the scientific chapter
owns it; if it changes how that value is bound, selected, displayed, exported or written back, this
chapter owns it.

### Named seams

- `SB-CORE-001`: unit identity and conversion precede axis binding.
- `SB-CORE-002`: a displayed clamp or exclusion never replaces the underlying value.
- `SB-CORE-006`: labels, equations, flags and plot semantics agree.
- `SB-CORE-010`: every plot-derived parameter and export carries provenance.
- `20_envcorr-qc.md`: normalization endpoints and valid/display ranges.
- `21_data-io.md`: depth alignment, resampling and null transport.
- `22_database-model.md`: persistent plot, selection and provenance records.
- `18_geomech-ppfg.md`: pressure-gradient computations and sign conventions.

### Explicitly not owned

No vendor lookup-table row, chart vertex, polygon coordinate, `.neu`, `.ovl`, `.itt`, `.itp`,
`.att`, `.bor`, `.eli` or CHM payload is part of this specification. The T1 inventory proves that
such artifacts exist and exposes their metadata/provenance defects; it does not license or justify
transcription (`CONTRACT.md` §2.1; dossier §§1.3, 2.6, 3.8, T1).

---

## 2. What the incumbents do — the requirement-bearing findings

### 2.1 Plot objects bind at different semantic strengths

Interactive Petrophysics binds single-well plots to concrete curve UIDs but multi-well plots to a
family or mnemonic. Techlog exposes four scoping levels and accepts expression-valued channels.
Geolog persists layouts as declarative objects and ships separate crossplot, ternary, histogram and
report families. The useful synthesis is a two-stage binding: persist semantic intent, then resolve
it to concrete curve identity separately for every well, recording the resolution (`plotting-
interactivity.md` §§1.1–1.3, 2.1, T1/T2).

### 2.2 Axis limits are data semantics, not decoration

The strongest incumbent resolution chain is user override → variable/header range → unit-family
range. The Geolog unit-limit mechanism demonstrates why the mechanism and its content must be
audited independently: of 83 checkable entries, 50 diverge by more than 15%, and 34 converted-unit
pairs carry identical numbers. One acoustic attenuation pair differs by a factor of 6.56, not by
rounding (`plotting-interactivity.md` §§2.2, 3.3–3.3a, T1/T2; critique A-4).

A valid range and a display range are different objects. A display range may clip glyphs; a valid
range controls QC and statistics. Conflating them silently changes counts and fitted values
(`plotting-interactivity.md` §§2.2, 5.3, T1/T2).

### 2.3 Binning and percent units contain silent-wrongness traps

The incumbents expose histogram count, two-dimensional grid size, smoothing, contours and minimum-
bin display thresholds. Their bin inclusion semantics and threshold comparators differ. The dossier
settles a canonical half-open rule and proves the exact conversion between the two documented
overplot thresholds: Geolog `T=0` (draw count `>0`) equals IP `D=1` (draw count `>=1`), generally
`T=D-1` (`plotting-interactivity.md` §§2.4, 3.9, 5.1–5.2, T1/T2).

Percentile probability and range position are not interchangeable. Percentile probability is
bounded `[0,100]`; range position may legitimately exceed 100 or be negative. A single `percent`
type turns an extrapolated endpoint into a silently clamped quantile (`plotting-interactivity.md`
§§2.4–2.5, 3.2, 5.1, T1/T2).

### 2.4 Overlay libraries are large and weakly provenanced

The Geolog install census contains 577 layout objects and 2,736 whole-tree plotting objects,
including a 333-object crossplot-polygon library. Four polygon-library defects were found by static
inspection. Chart records can carry contractor, data and revision fields, but the vendor directories
either omit them or ship them effectively empty. No incumbent supplies the complete chart
provenance needed for a defensible result (`plotting-interactivity.md` §§1.3, 2.6, 3.8, T1; critique
A-2).

Therefore chart compatibility cannot be inferred from mnemonic alone. It requires quantity,
canonical unit, unit conversion, chart identity, source, revision and a checksum of the payload
actually rendered (`plotting-interactivity.md` §§2.2, 2.6, 4.2, 5.4, T1/T2).

### 2.5 Interactivity ranges from local handles to a linked application state

The evidence contains six plot-to-parameter mechanisms, linked windows, selection propagation,
on-plot anchors, faceting, expression-valued channels and event subscriptions. It also contains
stale-view defects caused by subscribing to interval change but not theme or data change. The
requirement-bearing choice is named, persisted selections with one application event model and a
separate, provenance-bearing promotion from a selection to a scientific edit (`plotting-
interactivity.md` §§2.3, 2.8–2.8a, 3.5–3.7, 5.4, T1/T2; critique A-5, A-11).

### 2.6 Pickett, Hingle and regression are computational plots

The Pickett evidence includes ordinary least squares in both directions, reduced-major-axis fits,
robust regression, polynomial orders 2–5, exponential and power fits. The dossier prints the R²
definition and requires fit type, transformed space, valid-pair count and coefficients to be stored.
A Pickett intercept constrains the product `a·Rw`; the plot cannot identify `a` and `Rw` separately
without an independently supplied value (`plotting-interactivity.md` §§2.10, 3.1, 5.1; critique
A-3, A-8).

The correct Hingle transform is `Rt^(-1/m)`. The reciprocal-sign form found in vendor documentation
inverts the axis and creates a factor-of-ten error for a decade change in resistivity. It must be
refused, not emulated (`plotting-interactivity.md` §§2.10, 3.1, 5.1, T2).

### 2.7 Depth-step and capacity policy must preserve identity

One incumbent silently resamples to the first selected input; another averages arrays. The dossier
adopts: equal step proceeds; exact integer multiples decimate to the coarsest grid and report the
factor; non-integer ratios refuse. Windows are half-open `[lo, hi)`. For multi-well plots, a total
budget is acceptable only if allocation happens after valid-pair screening, every represented well
is reported, and the last eligible sample is retained (`plotting-interactivity.md` §§2.9, 2.11,
2.12, 5.3–5.5, T1/T2; critique A-10).

### 2.8 Rendering and export are one scientific surface

The dossier's target separates static and interaction layers, memoizes ranges and transformed
arrays, uses generation tokens for asynchronous loads, facets before decimation, and refetches when
zoom leaves the loaded interval. Paper-space export must preserve axes, legends and annotations as
vectors and must not crop them (`plotting-interactivity.md` §§2.8, 2.9, 5.4–5.5, T1/T2).

---

## 3. SandiBumi as-built

All findings in this section were re-verified against current source rather than inherited from the
dossier's earlier snapshot. Status vocabulary is the contract vocabulary.

### 3.1 Capability inventory

| Surface | Status | Direct source evidence |
|---|---|---|
| Histogram, crossplot, Pickett, correlation and declarative Vega panels | PRESENT-OK | `src/ui/workspace.ts:1156-1229`; `src/ui/vegaPanel.ts:1-32` (T1) |
| Persisted per-panel properties and named templates | PRESENT-OK | `src/ui/plotCommon.ts:55-194` (T1) |
| Canvas DPR sizing, mouse/keyboard pan-zoom, resize redraw and accessibility label | PRESENT-OK | `src/ui/plotCanvas.ts:80-131`, `:421-618`, `:620-635` (T1) |
| Theme, data and linked-brush subscriptions | PRESENT-OK | `src/ui/crossplotPanel.ts:2182-2255`; `src/ui/histogramPanel.ts:958-995`; `src/ui/pickettPanel.ts:604-614`; `src/ui/vegaPanel.ts:1131-1159` (T1) |
| Vector SVG/PDF plus PNG, clipboard and print | PRESENT-OK | `src/ui/plotExport.ts:8-190` (T1) |
| Regression: linear, power, log-X, exponential; Y-on-X, X-on-Y and RMA | PARTIAL | `src/ui/crossplotPanel.ts:117-179`, `:210-293` (T1); robust and polynomial 2–5 are absent |
| Pickett fit and saturation-line ladder | PARTIAL | `src/ui/pickettPanel.ts:100-172`, `:374-375` (T1); the `a·Rw` identifiability disclosure is absent |
| Hingle plot | ABSENT | No source symbol or panel found by an index-unavailable, repository-wide `rg` confirmation (T1 negative search, 2026-08-08) |

### 3.2 Binding and axis semantics

Crossplot axis defaults are hard-coded by mnemonic, and chart-overlay matching compares mnemonic
aliases only (`src/ui/crossplotPanel.ts:415-459`, T1). The chart definition has no per-axis unit,
source, revision or checksum field (`src/ui/chartOverlays.ts:12-26`, T1). The current surface is
therefore **PRESENT-DIVERGENT** from the typed-unit and provenance contract even though the overlays
render.

Plot builders do persist semantic curve names and well scope, but there is no persisted record of
the concrete curve ID, unit conversion and resolution decision for each well. Multi-well context
fetches return depth and values, yet `loadMultiWellCurves` retains only the value arrays
(`src/ipc.ts:646-679`; `src/ui/plotCommon.ts:482-497`, T1). Binding is **PARTIAL**.

### 3.3 Histogram and statistics

The histogram ships 60 bins, accepts 5–400, includes both endpoints of the selected range, and maps
the maximum into the final bin (`src/ui/histogramPanel.ts:69-94`, `:131-149`, T1). The dossier's
adoption default/range is 50 and 1–200 with half-open internal bins and a closed final bin
(`plotting-interactivity.md` §§5.1–5.2, T1/T2). Status: **PRESENT-DIVERGENT**.

The shared statistics skip all non-finite values, use sample standard deviation and linearly
interpolated percentiles (`src/ui/plotCanvas.ts:883-981`, T1). The histogram draws P25–P75 with a P50
line and P5/P95 whiskers (`src/ui/histogramPanel.ts:321-331`, T1). Status: **PRESENT-OK** for those
defined semantics. There is no distinct range-position-percent type. Status: **ABSENT**.

### 3.4 Multi-well integrity and depth identity

The loader uses a default concurrency of 8, divides a point budget equally among requested wells,
and stride-decimates each returned array (`src/ui/plotCommon.ts:463-510`, T1). Two defects remain:

1. Presence is tested by array existence/length, not by finite paired values. The backend represents
   a missing requested curve as a full-length NaN vector (`src-tauri/src/equations.rs:640`, T1), so
   an all-missing well can consume quota and appear represented.
2. The stride loop does not force the final eligible sample, so it can discard up to `stride−1`
   trailing levels (`src/ui/plotCommon.ts:488-501`, T1).

The backend interval is correctly half-open (`src-tauri/src/equations.rs:131-135`, T1). The frontend
does not retain per-series depths, does not check grid equality and does not enforce integer-multiple
decimation. Overall status: **PRESENT-DIVERGENT**.

### 3.5 Linked brushing, edits and provenance

Crossplot, histogram, log, Pickett and Vega surfaces share an ephemeral depth selection through
application state. The crossplot publishes exact depths after a shift-drag rectangle
(`src/state.ts:45-69`, `:144-151`; `src/ui/crossplotPanel.ts:1194-1215`, `:2211-2255`, T1). Status:
**PARTIAL** — the selection is one unnamed, unpersisted set, not a named multi-class selection.

Plot picks write zone parameters with a null source note (`src/ui/plotCommon.ts:526-570`;
`src/ui/crossplotPanel.ts:2132-2152`, T1). The write may succeed and be undoable through the common
parameter path, but it cannot prove which plot, axes, viewport, selection or data revision produced
the value. Status: **PRESENT-DIVERGENT** against `SB-CORE-010`.

### 3.6 Rendering and performance

Current panels use a shared DPR-capped backing-store helper, accessible canvas contract,
requestAnimationFrame resize coalescing and generation counters that discard stale asynchronous
builds (`src/ui/plotCanvas.ts:80-131`, `:620-635`; `src/ui/workspace.ts:1168-1207`, T1). Z-range
sorting is memoized in the current crossplot, correcting an older dossier finding
(`src/ui/crossplotPanel.ts:1218-1234`, T1). Theme/data subscriptions likewise correct the critique-
era stale-view findings.

The auto-axis path still invokes the same range function twice in one expression
(`src/ui/crossplotPanel.ts:724`, T1), and no committed device benchmark proves the 2,000-well target.
Status: **PARTIAL**.

---

## 4. Requirements

### 4.1 Binding, axes and units

#### SB-PLT-001 — Persist semantic intent and concrete resolution separately [P0] [status: PARTIAL]

Every plotted channel MUST persist its semantic request and, per well, the resolved curve ID,
mnemonic, quantity, source unit, display unit, conversion, sample count and resolution reason. A
plot MUST refuse an unresolved required channel rather than substitute a same-named curve silently.

**Evidence:** dossier §§2.1–2.2, 4.2, 5.4 (T1/T2); as-built §3.2 (T1).

#### SB-PLT-002 — Resolve axes through one explicit precedence chain [P0] [status: PRESENT-DIVERGENT]

Axis limits MUST resolve user override → variable/header display range → audited unit-family display
range → finite-data range. The UI and export MUST show which tier won. Validity ranges MUST NOT be
used as display ranges.

**Evidence:** dossier §§2.2, 3.3–3.3a, 5.3 (T1/T2).

#### SB-PLT-003 — Overlay compatibility is quantity-and-unit typed [P0] [status: ABSENT]

An overlay MUST declare X/Y quantity, canonical unit, orientation and admissible transform. The
renderer MUST convert compatible units before matching and MUST refuse incompatible axes. Mnemonic-
only matching MUST NOT authorize rendering.

**Evidence:** dossier §§2.2, 2.6, 3.3a, 5.4 (T1/T2); `crossplotPanel.ts:415-459` (T1).

#### SB-PLT-004 — Valid and display ranges remain distinct [P0] [status: ABSENT]

Every channel MAY carry both ranges. Display clipping MUST count and annotate hidden points; valid-
range exclusion MUST be an explicit analyst option and MUST report its effect on `n`, statistics and
fits.

**Evidence:** dossier §§2.2, 2.7, 5.3 (T1/T2).

#### SB-PLT-005 — Unit-limit content is audited before activation [P0] [status: ABSENT]

No imported unit-limit table MUST become a default merely because its schema is valid. Converted-
unit pairs MUST be dimensionally tested; suspect rows MUST ship disabled with a reason.

**Evidence:** dossier §3.3a (T1); critique A-4.

### 4.2 Binning, statistics and numerical plots

#### SB-PLT-006 — One canonical histogram-bin contract [P0] [status: PRESENT-DIVERGENT]

Bins MUST be `[edge_i, edge_{i+1})`, except that the final bin includes the upper range endpoint.
NaN and infinity MUST be excluded and counted separately. The displayed total MUST equal the sum of
bin counts.

**Evidence:** dossier §§2.4, 5.1–5.3 (T1/T2); `histogramPanel.ts:131-149` (T1).

#### SB-PLT-007 — Overplot thresholds expose the comparator [P1] [status: ABSENT]

A density layer MUST persist both its threshold value and comparator. Import MAY translate between
`draw count >= D` and `draw count > T` only by the exact integer relation `T=D−1`; it MUST NOT label
the raw numbers equivalent.

**Evidence:** dossier §§3.9, 5.1 (T1/T2).

#### SB-PLT-008 — Percentile probability and range position are different types [P0] [status: ABSENT]

`PercentileP` MUST accept only `[0,100]`. `RangePositionPct` MUST remain unbounded and MUST retain
negative and greater-than-100 values. APIs, templates and exports MUST name the type.

**Evidence:** dossier §§2.4–2.5, 3.2, 5.1 (T1/T2).

#### SB-PLT-009 — Statistics disclose population, estimator and exclusions [P1] [status: PARTIAL]

Every statistic MUST record active-well versus pooled population, interval, selection, finite-pair
count, exclusion counts, percentile interpolation and sample/population standard-deviation choice.

**Evidence:** dossier §§2.4, 2.9, 5.3 (T1/T2); `plotCanvas.ts:883-981` (T1).

#### SB-PLT-010 — Regression is a versioned scientific result [P1] [status: PARTIAL]

The fit record MUST contain model, method, transformed space, coefficients, R², valid-pair count,
excluded-pair counts, interval, wells and source curve revisions. v1 MUST support Y-on-X, X-on-Y and
RMA for linear/power/log-X/exponential fits; v1.5 SHOULD add robust and polynomial orders 2–5. R²
MUST be computed and displayed without clamping a valid negative non-OLS goodness metric into range.

**Evidence:** dossier §§2.10–2.11, 5.1, 5.4 (T1/T2); critique A-3, A-8, A-9.

#### SB-PLT-011 — Pickett states what is and is not identifiable [P0] [status: PRESENT-DIVERGENT]

A Pickett fit MUST state that the intercept identifies `a·Rw`, not `a` and `Rw` separately. It MUST
refuse to emit either separately unless the other is supplied with provenance. Saturation guide
lines MUST record `a`, `m`, `n`, `Rw` and their sources.

**Evidence:** dossier §§2.10, 5.1 (T1/T2); `pickettPanel.ts:100-172` (T1).

#### SB-PLT-012 — Hingle uses the negative reciprocal exponent [P1] [status: ABSENT]

The Hingle X transform MUST be `Rt^(-1/m)`. The reciprocal-sign form MUST be rejected by test and
MUST NOT be offered as a compatibility mode.

**Evidence:** dossier §§2.10, 3.1, 5.1 (T2).

### 4.3 Missing data, depth and capacity

#### SB-PLT-013 — Missing and out-of-range policy is channel-specific [P0] [status: PARTIAL]

The engine MUST implement and report: null/non-finite exclude; log-axis values `<=0` exclude from
plot and plot statistics; X/Y display overflow clip and count; Z overflow clamp to the endpoint
colour and edge-mark; array waveform overflow clamp and count. None MAY mutate the source curve.

**Evidence:** dossier §§2.7, 5.3 (T1/T2).

#### SB-PLT-014 — Multi-well allocation follows finite-pair screening [P0] [status: PRESENT-DIVERGENT]

The total point budget MUST be allocated only after required channels are aligned and finite pairs
are counted. A well with zero valid pairs MUST be reported as absent and MUST consume no quota.
Every represented well MUST retain its first and final eligible sample.

**Evidence:** dossier §§2.9, 5.3–5.5 (T1/T2); `plotCommon.ts:463-501` and
`equations.rs:640` (T1).

#### SB-PLT-015 — Decimation preserves pairing, endpoints and provenance [P0] [status: PARTIAL]

All channels in a mark MUST be decimated by one shared index vector. The plot MUST record original
count, displayed count, algorithm, stride and whether endpoints were forced. It MUST NOT label a
decimated view as complete.

**Evidence:** dossier §§2.9, 4.2, 5.3–5.5 (T1/T2).

#### SB-PLT-016 — Depth-step reconciliation is explicit and conservative [P0] [status: ABSENT]

Equal steps MUST proceed unchanged. Exact integer multiples MAY decimate to the coarsest step and
MUST report the factor. Non-integer ratios MUST refuse and route to the DIO resampling workflow.
Plot intervals MUST remain half-open `[lo,hi)`.

**Evidence:** dossier §§2.12, 5.1, 5.3 (T2); critique A-10;
`equations.rs:131-135` (T1).

#### SB-PLT-017 — Zoom beyond loaded data triggers an identified refetch [P1] [status: ABSENT]

When a viewport crosses the loaded interval, the panel MUST request the new half-open interval,
attach a generation token and discard stale responses. It MUST NOT stretch the old sample and imply
new data were loaded.

**Evidence:** dossier §§2.8, 5.4 (T1/T2).

### 4.4 Interactivity and parameter feedback

#### SB-PLT-018 — Linked selections are named, typed and persistable [P1] [status: PARTIAL]

A selection MUST have ID, label, colour, well set, channel predicates or exact depth membership,
creation source and data revision. Multiple selections MUST coexist. Ephemeral hover MAY remain
unpersisted; a selection used by a computation MUST be persisted.

**Evidence:** dossier §§2.8–2.8a, 3.5–3.7, 5.4 (T1/T2); `state.ts:45-69` (T1).

#### SB-PLT-019 — Every plot subscribes to the same invalidation contract [P1] [status: PRESENT-OK]

Plots MUST redraw on theme, data-revision, interval, selection and size changes; disposal MUST remove
every subscription and cancel pending work.

**Evidence:** dossier §§2.8, 5.4 (T1/T2); direct sources in §3.1 (T1).

#### SB-PLT-020 — Plot-derived parameter writes carry full provenance [P0] [status: PRESENT-DIVERGENT]

A write from a handle, marker, polygon or fit MUST be undoable and MUST carry plot ID, plot type,
axis bindings and units, viewport, selection, source-curve revisions, data interval, method, fit
record where applicable, user and timestamp. A null source note MUST be rejected.

**Evidence:** dossier §§2.8, 3.7, 5.4 (T1/T2); `plotCommon.ts:526-570` and
`crossplotPanel.ts:2132-2152` (T1); `SB-CORE-010`.

#### SB-PLT-021 — Expression-valued channels are sandboxed and reproducible [P2] [status: PARTIAL]

Plots MAY accept expressions, but MUST use the governed equation runtime, persist the expression and
dependencies, unit-check the output and record the data revision. Arbitrary panel JavaScript MUST
NOT become a scientific calculation path.

**Evidence:** dossier §§2.3, 3.6, 4.2 (T1/T2); Vega specification surface in
`vegaPanel.ts:1-32` (T1).

#### SB-PLT-022 — Faceting precedes decimation [P2] [status: ABSENT]

The engine MUST partition by facet before allocating a point budget, so small groups are not erased
by a global sampler. Each facet MUST report original and displayed counts.

**Evidence:** dossier §§3.5, 5.4 (T1/T2).

### 4.5 Chart provenance, templates and export

#### SB-PLT-023 — Every rendered chart is provenance-complete [P0] [status: ABSENT]

The plot MUST persist chart ID, title, chart type, X/Y quantity and unit, citation, publisher,
revision/date, digitizer if any, approved derivation path, payload checksum and transform applied.
Missing mandatory fields MUST block rendering in a deliverable.

**Evidence:** dossier §§2.6, 3.8, 5.4 (T1/T2).

#### SB-PLT-024 — Vendor chart payloads are never transcribed [P0] [status: PRESENT-OK]

The repository MUST NOT contain vendor chart tables, vertices or lookup artifacts. A product need
MUST be met by a licensed source or an independently digitized published primary source with its own
provenance. Metadata-only inventories MAY be retained.

**Evidence:** dossier §§1.3, 2.6, 3.8 (T1); `CONTRACT.md` §2.1.

#### SB-PLT-025 — Plot templates are schema-versioned and scope-aware [P1] [status: PARTIAL]

Templates MUST declare application scope, schema version, migration path, semantic bindings,
parameters and provenance dependencies. Applying a template MUST produce a diff and MUST NOT discard
an unknown field silently.

**Evidence:** dossier §§2.1, 2.8, 5.4 (T1/T2); `plotCommon.ts:73-194` (T1).

#### SB-PLT-026 — Export reruns the scientific draw at paper scale [P1] [status: PARTIAL]

SVG/PDF export MUST use the same plot state and vector draw path, retain all axes, legends,
annotations, exclusion counts and provenance footer, and prove that no element is cropped. Raster
print MUST be labelled raster.

**Evidence:** dossier §§2.8, 5.4–5.5 (T1/T2); `plotExport.ts:8-190` (T1).

#### SB-PLT-027 — Plot state is portable without embedding restricted payloads [P1] [status: PARTIAL]

Saved project state MUST reference approved chart IDs and checksums, never serialize a restricted
vendor chart payload into the project or template. Missing referenced content MUST fail visibly.

**Evidence:** dossier §§2.6, 3.8, 5.4 (T1/T2); `CONTRACT.md` §2.1.

### 4.6 Rendering, accessibility and performance

#### SB-PLT-028 — Static and interaction layers have separate invalidation [P1] [status: PARTIAL]

Axes, grids and invariant overlays SHOULD be cached separately from hover, brush and drag feedback.
Ranges, sorted quantiles and transformed arrays MUST be memoized by data revision and plot options.

**Evidence:** dossier §§2.8, 5.4 (T1/T2); §3.6 current source (T1).

#### SB-PLT-029 — Asynchronous plot loads are generation-safe [P0] [status: PRESENT-OK]

Every asynchronous build/refetch MUST carry a generation token; a superseded result MUST be disposed
without touching the active panel.

**Evidence:** dossier §5.4 (T1/T2); `workspace.ts:1168-1207` (T1).

#### SB-PLT-030 — Interactive canvases remain keyboard and assistive-technology reachable [P1] [status: PRESENT-OK]

Every canvas MUST expose a current accessible label, keyboard focus, keyboard pan/zoom and a non-
pointer route to properties and export.

**Evidence:** `plotCanvas.ts:527-618` (T1); dossier §2.8 (T2).

#### SB-PLT-031 — No silent record truncation [P0] [status: PARTIAL]

Any load, point, well, facet, legend or visual limit MUST be reported before and after reduction.
The user MUST be able to export the reduction manifest. A hard maximum MUST refuse rather than
silently return a prefix.

**Evidence:** dossier §§2.9, 3.10, 5.3–5.5 (T1/T2).

#### SB-PLT-032 — Plot performance is gated on declared hardware [P0] [status: ABSENT]

The release gate MUST include cold load, first useful paint, pan/zoom latency, selection latency,
memory and export time for single-well and multi-well fixtures. Results MUST name hardware, dataset,
curve count, point count, well count and software revision.

**Evidence:** dossier §§2.9, 5.4–5.5 (T1/T2); `01_PRODUCT.md` §6.3 records the benchmark gap.

### 4.7 Domain-specific plot shells

#### SB-PLT-033 — Pressure-gradient crossplots preserve the geomechanics sign convention [P1] [status: ABSENT]

The plotting shell MUST accept typed pressure/depth channels, show the selected datum and sign
convention, and persist picks with provenance. It MUST NOT compute pressure or fracture-gradient
methods; `18_geomech-ppfg.md` owns those equations and fixtures.

**Evidence:** dossier §§2.11, 5.1 (T1/T2); critique A-9.

#### SB-PLT-034 — Ternary plots normalize visibly [P2] [status: ABSENT]

A ternary plot MUST declare component units and normalization rule, flag non-finite or negative
components, show the pre-normalization sum and preserve original values. It MUST NOT silently
renormalize invalid mineral volumes into a plausible point.

**Evidence:** dossier §§1.2–1.3, 2.11, 4.2 (T1/T2).

#### SB-PLT-035 — Clay-volume interactive plots use the governed equation [P2] [status: PARTIAL]

An interactive clay endpoint plot MUST call the same versioned equation and parameter schema as the
batch module, and every endpoint write MUST satisfy `SB-PLT-020`. It MUST NOT duplicate a hidden
formula in the UI.

**Evidence:** dossier §§2.8, 3.7; critique A-11 (T1/T2); seam `10_clay-volume.md`.

---

## 5. Parameters

Values are byte-exact from dossier §5.2 or current source. No neighbouring-vendor default is chosen
to settle a disagreement. `ABSENT` rows are intentional release behaviour under `CONTRACT.md` §2.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Histogram bin default | `HIST_BINS` | 50 | bins | `plotting-interactivity.md` §5.2, adoption row “Histogram bins” | T1/T2 |
| Histogram minimum bins | `HIST_BINS_MIN` | 1 | bins | `plotting-interactivity.md` §5.2 | T1/T2 |
| Histogram maximum bins | `HIST_BINS_MAX` | 200 | bins | `plotting-interactivity.md` §5.2 | T1/T2 |
| Current histogram default to retire | `HIST_BINS_CURRENT` | 60 | bins | `src/ui/histogramPanel.ts:69-80` | T1 |
| Current histogram accepted range to retire | `HIST_BINS_CURRENT_RANGE` | 5–400 | bins | `src/ui/histogramPanel.ts:93-94` | T1 |
| Two-dimensional density grid | `DENSITY_GRID` | 50 × 50 | bins | `plotting-interactivity.md` §5.2 | T1/T2 |
| Density smoothing default | `DENSITY_SMOOTHING` | off | boolean | `plotting-interactivity.md` §5.2 | T1/T2 |
| Contour count | `CONTOUR_COUNT` | 20 | contours | `plotting-interactivity.md` §5.2 | T1/T2 |
| Contour minimum level | `CONTOUR_MIN` | 1 | count | `plotting-interactivity.md` §5.2 | T1/T2 |
| Minimum displayed bin count | `MIN_BIN_COUNT` | 1 | count | `plotting-interactivity.md` §§3.9, 5.2 | T1/T2 |
| Box lower hinge | `BOX_P_LO` | 25 | percentile probability | `plotting-interactivity.md` §§2.4, 5.2 | T1/T2 |
| Box median | `BOX_P_MED` | 50 | percentile probability | `plotting-interactivity.md` §§2.4, 5.2 | T1/T2 |
| Box upper hinge | `BOX_P_HI` | 75 | percentile probability | `plotting-interactivity.md` §§2.4, 5.2 | T1/T2 |
| Box lower whisker | `BOX_W_LO` | 5 | percentile probability | `plotting-interactivity.md` §§2.4, 5.2 | T1/T2 |
| Box upper whisker | `BOX_W_HI` | 95 | percentile probability | `plotting-interactivity.md` §§2.4, 5.2 | T1/T2 |
| Current multi-well total budget | `POINT_BUDGET` | 60000 | points | `src/ui/crossplotPanel.ts:1700-1702` | T1 |
| Current context-fetch concurrency | `FETCH_CONCURRENCY` | 8 | wells in flight | `src/ui/plotCommon.ts:510` | T1 |
| Crossplot point-size default | `POINT_SIZE` | 1.6 | CSS px | `src/ui/crossplotPanel.ts:117-121` | T1 |
| Crossplot point-size accepted range | `POINT_SIZE_RANGE` | 0.5–8 | CSS px | `src/ui/crossplotPanel.ts:176` | T1 |
| Crossplot fixed width default | `PLOT_W` | 640 | CSS px | `src/ui/crossplotPanel.ts:132` | T1 |
| Crossplot fixed height default | `PLOT_H` | 480 | CSS px | `src/ui/crossplotPanel.ts:133` | T1 |
| Crossplot fixed-size accepted range | `PLOT_SIZE_RANGE` | 200–2000 | CSS px | `src/ui/crossplotPanel.ts:170-171` | T1 |
| Backing-store device-pixel-ratio cap | `DPR_CAP` | 2.5 | ratio | `src/ui/plotCanvas.ts:84` | T1 |
| Mouse zoom-in factor | `ZOOM_IN` | 0.83 | ratio | `src/ui/plotCanvas.ts:448` | T1 |
| Mouse zoom-out factor | `ZOOM_OUT` | 1.2 | ratio | `src/ui/plotCanvas.ts:448` | T1 |
| Minimum regression valid pairs | `REG_N_MIN` | 3 | pairs | `src/ui/crossplotPanel.ts:259` | T1 |
| Regression variance epsilon | `REG_VAR_EPS` | 1e-12 | transformed-unit² | `src/ui/crossplotPanel.ts:263` | T1 |
| Pickett saturation guides | `SW_GUIDES` | 1 / 0.5 / 0.25 | v/v | `src/ui/pickettPanel.ts:151-164` | T1 |
| Pickett cementation exponent | `m` | **ABSENT — ships with no default** | dimensionless | Dossier §2.10: incumbent values are not adjudicated; `12_saturation.md` owns the parameter | T1/T2 |
| Pickett saturation exponent | `n` | **ABSENT — ships with no default** | dimensionless | Dossier §2.10: incumbent values are not adjudicated; `12_saturation.md` owns the parameter | T1/T2 |
| Pickett tortuosity factor | `a` | **ABSENT — ships with no default** | dimensionless | Dossier §2.10: Pickett intercept identifies `a·Rw`, not `a` separately | T1/T2 |
| Pickett formation-water resistivity | `Rw` | **ABSENT — ships with no default** | ohm·m | Dossier §2.10: Pickett intercept identifies `a·Rw`, not `Rw` separately | T1/T2 |
| Ternary invalid-sum tolerance | `TERNARY_SUM_TOL` | **ABSENT — ships with no default** | v/v | No cited value in dossier §§2.11 or 5.2 | — |
| Device interaction-latency gate | `INTERACTION_GATE` | **ABSENT — ships with no default** | ms | Dossier §6 `OPEN-X8`: no measured baseline on target hardware | — |
| Device first-useful-paint gate | `FIRST_PAINT_GATE` | **ABSENT — ships with no default** | ms | Dossier §6 `OPEN-X8`: no measured baseline on target hardware | — |

Thirty-five rows. **Seven** read `ABSENT — ships with no default`. Those seven are the four
scientific Pickett inputs, one ternary tolerance and two performance gates. The first four belong to
the saturation seam and cannot be manufactured here; the final three require a product decision or
measurement. The current 60-bin and 5–400 histogram values are cited as as-built values to retire,
not competing defaults silently adopted.

---

## 6. Acceptance tests

Every test states input, operation, expected result and the source of that expected result.

| Test | Input and operation | Expected value | Source |
|---|---|---|---|
| SB-PLT-T01 | Resolve an axis with user, header, family and data ranges present | User range wins; provenance tier=`user` | Dossier §§2.2, 5.3 (T1/T2) |
| SB-PLT-T02 | Remove the user range from T01 | Header/variable display range wins | Dossier §§2.2, 5.3 (T1/T2) |
| SB-PLT-T03 | Bind an overlay whose mnemonic matches but quantity/unit is incompatible | Rendering is refused | Dossier §§2.2, 2.6, 3.3a (T1/T2) |
| SB-PLT-T04 | Bind compatible source/display units with a registered conversion | Values are converted; both units and transform persist | Dossier §§2.2, 5.4 (T1/T2) |
| SB-PLT-T05 | Audit the documented attenuation converted-unit pair | Divergence is reported as 6.56×, not rounding | Critique A-4; dossier §3.3a (T1) |
| SB-PLT-T06 | Histogram `[0,1,2,3]` over `[0,3]` with 3 bins | Counts are `[1,1,2]`; final upper endpoint is included | Dossier §§2.4, 5.1 (T1/T2) |
| SB-PLT-T07 | Histogram `[0,NaN,+∞,1]` over `[0,1]` | Bin total=2; non-finite excluded count=2 | Dossier §§2.7, 5.3 (T1/T2) |
| SB-PLT-T08 | Compare IP draw threshold `D=1` with Geolog `T=0` | Both draw bins with count ≥1 | Dossier §3.9 (T1/T2) |
| SB-PLT-T09 | Convert arbitrary integer `D=4` | `T=3`; comparator remains explicit | Dossier §§3.9, 5.1 (T1/T2) |
| SB-PLT-T10 | Parse `PercentileP=130` | Rejected | Dossier §§2.5, 3.2 (T1/T2) |
| SB-PLT-T11 | Parse `RangePositionPct=130` and `-5` | Both retained byte-exact | Dossier §§2.5, 3.2 (T1/T2) |
| SB-PLT-T12 | Statistics on `[1,2,3,NaN,+∞]` | count=3, mean=2, P50=2; exclusions=2 | `plotCanvas.ts:883-981` (T1) |
| SB-PLT-T13 | Box plot on ordered values 0…100 | Hinges P25/P75, median P50, whiskers P5/P95 | Dossier §§2.4, 5.2 (T1/T2) |
| SB-PLT-T14 | Linear Y-on-X fixture `y=2+3x`, x=1…5 | intercept=2, slope=3, R²=1, n=5 | Dossier §5.1 formula; arithmetic fixture |
| SB-PLT-T15 | Same fixture fitted X-on-Y and RMA | Method field differs; stored source points and n remain 5 | Dossier §§2.10, 5.1 (T1/T2) |
| SB-PLT-T16 | Power fit with one X value `0` | That pair is excluded and counted; remaining fit is finite | Dossier §§2.7, 5.1 (T1/T2) |
| SB-PLT-T17 | Pickett fit without sourced `a` or `Rw` | UI reports identifiable product `a·Rw`; separate write is blocked | Dossier §2.10 (T1/T2) |
| SB-PLT-T18 | Hingle transform at `m=2`, `Rt=100` | X=`100^(-1/2)=0.1` | Dossier §§3.1, 5.1 (T2); shown arithmetic |
| SB-PLT-T19 | Hingle reciprocal-sign variant at same input | Variant is rejected; `10` is never rendered as equivalent | Dossier §3.1 (T2) |
| SB-PLT-T20 | Multi-well request where one well has full-length all-NaN Y | Missing well gets zero quota and an explicit absent reason | Dossier §§2.9, 5.3; `equations.rs:640` (T1/T2) |
| SB-PLT-T21 | Decimate eligible indices 0…10 at stride 4 | Displayed indices include 0 and 10; manifest reports forced endpoint | Dossier §§2.9, 5.3–5.5 (T1/T2) |
| SB-PLT-T22 | Decimate X/Y/Z/depth together | Every displayed mark uses one shared source index | Dossier §§2.9, 5.3 (T1/T2) |
| SB-PLT-T23 | Inputs at equal depth step | Proceed with factor 1 | Dossier §2.12 (T2) |
| SB-PLT-T24 | Inputs at steps 0.5 and 1.0 | Decimate to 1.0; report factor 2 | Dossier §§2.12, 5.1 (T2) |
| SB-PLT-T25 | Inputs at steps 0.5 and 0.8 | Refuse; route to DIO resampling | Dossier §§2.12, 5.1 (T2) |
| SB-PLT-T26 | Query interval `[100,101)` containing samples 100,100.5,101 | Return 100 and 100.5 only | `equations.rs:131-135` (T1) |
| SB-PLT-T27 | Pan a view beyond its loaded high bound | One generation-tagged refetch is issued | Dossier §§2.8, 5.4 (T1/T2) |
| SB-PLT-T28 | Resolve two async refetches in reverse order | Only newest generation renders | Dossier §5.4; `workspace.ts:1168-1207` (T1/T2) |
| SB-PLT-T29 | Create two named selections and activate both | Both persist with IDs, colours and membership | Dossier §§2.8–2.8a, 5.4 (T1/T2) |
| SB-PLT-T30 | Promote a plot pick to a zone parameter | Write contains plot/axis/unit/interval/revision/user/time provenance | Dossier §§3.7, 5.4; `SB-CORE-010` (T1/T2) |
| SB-PLT-T31 | Attempt the same write with null source metadata | Write is rejected | `SB-CORE-010`; as-built defect `plotCommon.ts:564` (T1) |
| SB-PLT-T32 | Change theme with crossplot, histogram, Pickett and Vega open | Every panel redraws once and retains data/viewport | Direct sources listed in §3.1 (T1) |
| SB-PLT-T33 | Change data revision during an in-flight build | Stale build disposes and never replaces current content | `workspace.ts:1168-1207` (T1) |
| SB-PLT-T34 | Facet a dataset with one small and one large group, then budget | Small facet remains represented; counts shown per facet | Dossier §§3.5, 5.4 (T1/T2) |
| SB-PLT-T35 | Load a chart record missing source revision | Deliverable rendering is blocked | Dossier §§2.6, 3.8, 5.4 (T1/T2) |
| SB-PLT-T36 | Save/reload a template containing an unknown future field | Field is preserved or migration refuses; never silently dropped | Dossier §§2.1, 5.4 (T1/T2) |
| SB-PLT-T37 | Export a plot with long axis labels and outside legend to SVG/PDF | Labels/legend are present, uncropped and vector | Dossier §5.5; `plotExport.ts:109-190` (T1/T2) |
| SB-PLT-T38 | Print the same plot through raster path | Output is labelled raster and retains provenance footer | Dossier §5.5; `plotExport.ts:37-74` (T1/T2) |
| SB-PLT-T39 | Focus canvas and use keyboard pan/zoom | View changes; accessible label remains current | `plotCanvas.ts:527-618` (T1) |
| SB-PLT-T40 | Request an export after budget reduction | Export includes original/displayed counts and algorithm | Dossier §§2.9, 5.3–5.5 (T1/T2) |
| SB-PLT-T41 | Normalize ternary components `[0.2,0.3,0.5]` | Sum shown as 1; plotted values unchanged | Dossier §2.11 (T1/T2) |
| SB-PLT-T42 | Normalize ternary components `[0.2,-0.1,0.9]` | Invalid negative component is flagged; no silent plausible point | Dossier §§2.7, 2.11 (T1/T2) |
| SB-PLT-T43 | Run the declared performance fixture twice on named hardware | Report contains all §4.6 metrics and software/data revisions | Dossier §§2.9, 5.5, 6 `OPEN-X8` (T1/T2) |

Forty-three acceptance tests cover all thirty-five requirements. Numerical expected values are from
the dossier/current source or show their arithmetic in the row; no scientific default is invented.

---

## 7. Open items, escalations and refusals

### 7.1 Open items

**O-1 — Performance gates are unmeasured.** The dossier has no device baseline for the 2,000-well
target (`OPEN-X8`). **Settled by:** running `SB-PLT-T43` on declared release hardware and adopting
the measured gates; no number is invented meanwhile.

**O-2 — Unit-limit content needs a row-level screen.** The audit establishes widespread divergence
but does not adjudicate every row (`OPEN-X4`). **Settled by:** dimensional re-derivation against
primary unit definitions. Bulk import remains disabled.

**O-3 — Robust-regression algorithm and tuning are not sourced.** The incumbent inventory names the
capability but not a reproducible method/default (`OPEN-X5`). **Settled by:** a primary statistical
source and deterministic fixtures; the feature remains absent meanwhile.

**O-4 — Ternary invalid-sum tolerance is absent.** The evidence requires explicit normalization but
supplies no tolerance (`OPEN-X9`). **Settled by:** a house exact-sum policy or a cited numerical-
analysis tolerance.

**O-5 — Chart rights and provenance are unresolved for every payload currently represented only by
code constants or metadata.** **Settled by:** a licensed source or independently digitized published
primary source plus the fields in `SB-PLT-023`. Rendering capability does not settle content rights.

**O-6 — The one-selection application state has no persistence owner yet.** `22_database-model.md`
must define records before `SB-PLT-018` can be implemented without an ad-hoc JSON island.

**O-7 — Pressure-gradient plot datum/sign ownership must close with chapter 18.** The visual shell is
`SB-PLT-033`; all numeric conventions remain deliberately absent here.

**O-8 — The dossier's `OPEN-X1`…`OPEN-X12` register is fully dispositioned in §8.** None is silently
dropped; unresolved items above preserve the specific acquisition or decision needed.

### 7.2 Escalations

**E-1 — Decide whether 50 bins replaces both current defaults.** The adoption dossier chooses 50,
while current histogram and crossplot defaults are 60 and 40. The chapter specifies one canonical
default, but product approval is needed because saved templates migrate visibly.

**E-2 — Approve the chart-provenance blocking rule for deliverables.** The evidence shows incumbent
fields are empty, and the current registry has no equivalent. Blocking is the defensible default;
the product decision is whether exploratory rendering may show an explicit unverified watermark.

**E-3 — Assign persistent named selections to the database chapter.** Plotting owns behaviour;
database-model owns storage and referential integrity.

**E-4 — Decide whether the current 60,000-point budget is retained as a product constant.** It is
cited as-built, not benchmark-validated. `SB-PLT-032` must settle it before it becomes contractual.

**E-5 — Acquire primary references for robust regression and any polynomial-fit safeguards.** The
feature inventory alone is not a derivation source.

**E-6 — Confirm chart-library disposition before release.** No vendor payload may be committed or
embedded; every surviving overlay needs an allowed source and checksum.

### 7.3 Refusals

**R-1 — SandiBumi will not bind overlays by mnemonic alone.** *Instead:* quantity/unit matching and
explicit conversion (`SB-PLT-003`). *Why:* a familiar name does not establish measurement identity.

**R-2 — SandiBumi will not bulk-adopt a syntactically valid unit-limit table.** *Instead:* audit each
dimension/conversion and disable suspect rows (`SB-PLT-005`). *Why:* the dossier found 50 of 83
checkable entries beyond the stated divergence threshold and a 6.56× converted-unit defect.

**R-3 — SandiBumi will not conflate percentile probability with range position.** *Instead:* two
types (`SB-PLT-008`). *Why:* clamping a valid 130% extrapolation changes an endpoint silently.

**R-4 — SandiBumi will not reproduce the wrong-sign Hingle axis.** *Instead:* use `Rt^(-1/m)` and
reject the reciprocal form (`SB-PLT-012`). *Why:* it inverts the axis and creates a decade error.

**R-5 — SandiBumi will not treat excluded display cells as excluded statistics without saying so.**
*Instead:* the plot records population and exclusions (`SB-PLT-009`, `SB-PLT-013`).

**R-6 — SandiBumi will not let an all-NaN curve consume a well quota.** *Instead:* screen finite
pairs before allocation (`SB-PLT-014`). *Why:* current full-length NaN transport makes length an
invalid presence test.

**R-7 — SandiBumi will not silently lose the tail of a decimated interval.** *Instead:* force the
last eligible source index and disclose the reduction (`SB-PLT-014`, `SB-PLT-015`).

**R-8 — SandiBumi will not resample to the first selected input.** *Instead:* equal step, exact-
multiple decimation or explicit DIO refusal (`SB-PLT-016`). *Why:* selection order is not a
scientific resampling policy.

**R-9 — SandiBumi will not write a picked value with null provenance.** *Instead:* reject the write
until `SB-PLT-020` metadata are complete. *Why:* an undoable but untraceable edit still violates
`SB-CORE-010`.

**R-10 — SandiBumi will not silently truncate to a maximum record count.** *Instead:* reduce with a
manifest or refuse (`SB-PLT-031`).

**R-11 — SandiBumi will not transcribe vendor chart payloads.** *Instead:* licensed or independently
digitized public primary sources with provenance (`SB-PLT-024`).

**R-12 — SandiBumi will not renormalize invalid ternary components into a plausible point.**
*Instead:* preserve originals, show the sum and flag/refuse (`SB-PLT-034`).

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

---

## 8. Traceability — dossier disposition

### 8.1 Requirement-to-evidence map

| Requirements | Dossier evidence | Disposition |
|---|---|---|
| SB-PLT-001–005 | §§2.1–2.2, 2.6, 3.3–3.3a, 5.3–5.4 | Binding, axes, units and overlay compatibility specified |
| SB-PLT-006–012 | §§2.4–2.5, 2.10–2.11, 3.1–3.2, 3.9, 5.1–5.2 | Binning, percent types, statistics, regression, Pickett and Hingle specified |
| SB-PLT-013–017 | §§2.7, 2.9, 2.12, 5.3–5.5 | Missing data, capacity, decimation, depth and refetch specified |
| SB-PLT-018–022 | §§2.3, 2.8–2.8a, 3.5–3.7, 5.4 | Selections, subscriptions, writeback, expressions and faceting specified |
| SB-PLT-023–027 | §§1.3, 2.6, 3.8, 5.4–5.5 | Chart provenance, prohibited payloads, templates and export specified |
| SB-PLT-028–032 | §§2.8–2.9, 3.10, 5.3–5.5 | Rendering, async safety, accessibility, truncation and benchmarks specified |
| SB-PLT-033–035 | §§2.8, 2.11, 5.1; critique A-9/A-11 | Domain plot shells specified with seams preserved |

All thirty-five `SB-PLT` IDs are unique and all are traced above.

### 8.2 Dossier method inventory

| Inventory block | Coverage |
|---|---|
| §1.1 IP plot, crossplot, histogram, Pickett/Hingle, chart and interactivity surfaces | §§2.1–2.8; SB-PLT-001–018, -023–026, -031 |
| §1.2 Techlog plot object/scopes, Plot.py, regressions, expressions and subscriptions | §§2.1, 2.5–2.6; SB-PLT-001, -010, -018–022, -028–029 |
| §1.3 Geolog layouts, 577-object census and 2,736-object whole-tree census | §§2.1, 2.4–2.5; SB-PLT-023–027 |
| §1.3 333-object polygon library and four static defects | §2.4; SB-PLT-023–024; O-5/E-6 |
| §1.4 SandiBumi as-built | §3 re-verified at current source; requirements carry current statuses |

No chart coordinate or lookup-table payload was copied.

### 8.3 Difference and decision ledgers

| Dossier register | Disposition |
|---|---|
| `D02` | Axis-limit/unit split → SB-PLT-002–005 |
| `L-D-L01`…`L-D-L03` | Plot binding and axis resolution → SB-PLT-001–005 |
| `L-D-L04`…`L-D-L06` | Binning, percent semantics and reduction → SB-PLT-006–009, -014–015 |
| `L-D-L07`…`L-D-L09` | Interactivity, expressions and faceting → SB-PLT-018–022 |
| `L-D-L10` | Explicit out-of-range/null policy → SB-PLT-013 |
| `L-D-L11` | Chart identity/provenance → SB-PLT-003, -023–024, -027 |
| §4.2 item-by-item optimal choices | Adopted or routed across SB-PLT-001–035; scientific equations remain at named seams |

### 8.4 Open and escalation registers

| Dossier item | Disposition |
|---|---|
| `OPEN-L01`…`OPEN-L04` | Binding/unit/chart questions → O-2, O-5; SB-PLT-001–005, -023 |
| `OPEN-L05`…`OPEN-L08` | Binning/statistics/interactivity questions → O-3; SB-PLT-006–010, -018–022 |
| `OPEN-L09`…`OPEN-L11` | Capacity/depth/export questions → O-1; SB-PLT-014–017, -026, -031–032 |
| `OPEN-L12`…`OPEN-L13` | Domain-plot and provenance questions → O-4/O-7; SB-PLT-020, -033–035 |
| `OPEN-X1` | Axis source precedence → SB-PLT-002 |
| `OPEN-X2` | Unit compatibility → SB-PLT-003–005, O-2 |
| `OPEN-X3` | Chart provenance → SB-PLT-023–024, O-5 |
| `OPEN-X4` | Unit-limit row audit → O-2 |
| `OPEN-X5` | Robust-fit method/default → O-3/E-5 |
| `OPEN-X6` | Selection persistence → O-6/E-3 |
| `OPEN-X7` | Depth reconciliation → SB-PLT-016 |
| `OPEN-X8` | Device benchmark → O-1, SB-PLT-032 |
| `OPEN-X9` | Ternary policy → O-4, SB-PLT-034 |
| `OPEN-X10` | Chart rights → O-5/E-6 |
| `OPEN-X11` | Plot-derived provenance → SB-PLT-020 |
| `OPEN-X12` | Cross-domain pressure plot → O-7, SB-PLT-033 |
| `E1`–`E10` and `E2b` | Consolidated without loss into §7.1–7.2 and the named seams; no numeric answer invented |

### 8.5 Adoption-spec and test disposition

Dossier §5.1 canonical equations are covered by SB-PLT-006–012 and tests T06–T19. Every dossier
§5.2 parameter row is either present in §5, represented as an explicit behaviour rather than a
parameter, or left `ABSENT`; no vendor disagreement is adjudicated silently. Dossier §5.3 policies
are covered by SB-PLT-013–017 and tests T20–T28. Dossier §5.4 architecture is covered by
SB-PLT-018–029 and tests T29–T38. Dossier tests `T1`–`T22`, including `T6b`–`T6i`, are represented by
chapter tests T01–T43; identifiers are chapter-local rather than copied.

### 8.6 Critique disposition

| Critique item | Chapter disposition |
|---|---|
| A-1 blocker | Corrected pressure-gradient constant remains owned by chapter 18; no value copied here; SB-PLT-033 preserves the seam |
| A-2 | Full Geolog censuses and polygon library recorded in §§2.4/8.2; no payload copied |
| A-3 | Pickett and regression restored in §§2.6, 4.2 and tests T14–T19 |
| A-4 | 6.56× unit defect recorded; SB-PLT-005/T05 |
| A-5 | Fifth chart parameter/on-plot anchor preserved at capability level; governed by SB-PLT-020/-023 |
| A-6 | No invented units; every §5 row uses the dossier or direct source string |
| A-7 | Threshold equivalence stated only with sourced comparator and exact conversion; SB-PLT-007/T08–T09 |
| A-8 | R² and fit record made normative in SB-PLT-010/T14–T16 |
| A-9 | Regression and pressure-gradient plot restored; SB-PLT-010/-033 |
| A-10 | Depth-step rule restored; SB-PLT-016/T23–T26 |
| A-11 | Interactive clay plot restored with governed-equation seam; SB-PLT-035 |
| A-12 | Client/asset identifier removed; this chapter contains none |
| B-1–B-11, B-13 | Incorporated in revised dossier and discharged by the corresponding binding, chart, rendering, parameter and traceability requirements above |
| B-12 | Revised dossier marks the finding non-reproducible; no obligation asserted from it |

### 8.7 As-built corrections to the dossier snapshot

The chapter does not perpetuate defects that current source has already closed. Theme/data/brush
subscriptions, DPR backing-store setup, keyboard interaction, vector export, generation-token panel
builds and memoized Z range are PRESENT-OK or PARTIAL as stated in §3. Those are current T1 findings,
not contradictions to the historical critique. Remaining requirements are grounded in current gaps.

### 8.8 Completeness statement

The dossier's inventories, equations, eleven ledger rows, thirteen `OPEN-L` items, twelve `OPEN-X`
items, adoption parameters, acceptance tests, applicable defect rules, escalation register and the
critique's A/B findings are all dispositioned above. Surplus requirements SB-PLT-019, -024, -029 and
-030 capture current source strengths that the release must preserve. No Tier-C item falls in this
domain, and §7.4 states that explicitly.
