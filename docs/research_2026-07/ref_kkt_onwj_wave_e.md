# Wave E sources — KKT ONWJ full field evaluation (Jauhar, LAPI-ITB / PHE ONWJ)

Sources (NOT in repo — client material, do not commit):
- `C:\Users\ARUNIKA\OneDrive\01. Work\42. KKT ONWJ\Full FE KKT ONWJ.pptx` (95 slides;
  the PDF beside it has the same content at page = slide − 1)
- `C:\Users\ARUNIKA\OneDrive\01. Work\42. KKT ONWJ\Multimin Parameters.xlsx`
  (17 structure sheets, per-well/per-zone SandiMin endpoint tables + conversion formulas)

These specs back ROADMAP §4c Wave E items (17)–(22). Slide numbers below are pptx
numbering.

## (17) Pre-calculation module — FTEMP / FPRESS / RMF / CT / CXO (slide 31, 60)

Multimin needs RMF, formation temperature and formation pressure at reservoir
conditions; mud data are incomplete across wells, so they are estimated from trends:

- **FTEMP** [degF]: linear in TVDSS, calibrated to header TLT + DST BHT points
  (BLT-derived temperatures can still reflect circulating mud — DST BHT preferred).
  Example fit (KK structures): `FTEMP = 77 + 0.0260292·TVDSS(ft)`.
- **FPRESS** [psi]: linear in TVDSS from RFT/pressure-gradient points.
  Example: `FPRESS = 44.2823 + 0.539812·TVDSS(ft)` (CC 0.956).
- **RMF** [ohmm at FTEMP]: from mud-property inputs (Rm/Rmf measured @ surface temp →
  Arps to FTEMP; `multimin2::arps_f` already exists) or, when mud data are missing, a
  field regression vs depth. Example: `RMF = 0.517068 − 0.116517·log10(TVDSS)`.
- **CT / CXO** [mmho/m]: conductivity transforms of the deep / flushed resistivities,
  `CT = 1000/RT`, `CXO = 1000/RXO` — the linearly-additive inputs SandiMin's CT/CXO
  rows consume.

Module shape: Prep-category manifest module, per-well; params = surface temp + temp
gradient, pressure intercept + gradient, Rmf@meas-temp + meas-temp (or regression
a/b), inputs TVDSS (fallback DEPTH), RT, RXO; outputs FTEMP, FPRESS, RMF, CT, CXO.
Fit helpers (BHT/RFT point fits) can come later — start with user-entered gradients.
Existing hooks: `multimin2::FluidProps { rmf, rmf_temp_f, ftemp_f, .. }` takes scalar
values today; the SandiMin dialog should be able to read them from precalc output
curves (zone-averaged) once this module exists.

## (18) Wet-clay → dry-clay endpoint conversion (Multimin Parameters.xlsx)

The xlsx picks WET clay log readings per zone (from shale intervals: RHOB, NPHI, GR,
DT) plus an assumed DRY clay density, and derives the dry-clay endpoints for the
PHIT-basis (dry-clay framework) model. Formulas (verbatim from the sheet, water at
1.00 g/cc and 189 µs/ft):

```
φ_clay    = (ρ_dryclay − ρ_wetclay) / (ρ_dryclay − 1.0)     // clay-bound porosity
NPHI_dry  = (NPHI_wet − φ_clay) / (1 − φ_clay)
GR_dry    =  GR_wet / (1 − φ_clay)
DT_dry    = (DT_wet − 189·φ_clay) / (1 − φ_clay)
RHOB_dry  =  given (2.70–2.78 by zone; deeper zones higher)
```

Example row (KK-1 Post Main): quartz GR 40 / ρ 2.65; dry clay ρ 2.70 → wet ρ 2.18333,
wet NPHI 0.489583, wet GR 110 gives φ_clay 0.3039, NPHI_dry 0.2667, GR_dry 158.0.

Use in SandiBumi: a converter in the SandiMin dialog (enter wet-clay readings →
derived dry-clay component endpoints + φ_clay), so the volume model solves dry clay +
clay-bound water explicitly and PHIT includes CBW (dual-water consistent). CBW =
φ_clay · V_dryclay/(1−φ_clay) bookkeeping to be confirmed against the deck's CWB
slide (59) at implementation.

## (19) Gas correction, iterated to convergence (slide 65; QC slides 66–67)

Slide 65 derivation ends at:

```
ρb_corrected = RHOB + Φt·(1−Sw)·(1.00 − ρg_res)
```

with `Φt·(1−Sw)` = gas volume and `ρg_res` = gas density from SG converted to
reservoir P & T (needs FPRESS/FTEMP from item 17; real-gas ρg = M·P/(z·R·T) with a
z-factor correlation — choice of correlation to confirm with Jauhar, Papay is the
simple default). Φt and Sw are outputs of the solve that consumes RHOB, so per
Jauhar this must be an **outer iteration until convergence (IP workflow style)**:
solve → Φt, Sw → correct RHOB (NPHI analog: excavation/HI deficit — confirm whether
the study corrected NPHI too) → re-solve, until |ΔΦt| below tol (~1e-4) or ~10
iterations. QC: PHIE vs VOL_WETCLAY crossplot before/after — the detached
high-porosity gas cloud (slide 66 red circles) collapses onto the trend after
correction (slide 67).

## (20) Porosity φmax option from compaction trend (slide 64)

The porosity-validation crossplots draw a hard "max core porosity: 0.35" line — the
field's compaction-controlled ceiling. Item: porosity modules (and SandiMin) get an
optional **PHI_MAX** cap — constant per zone, or a compaction-trend function of
TVDSS. Exact trend form (linear vs exponential, per formation) to confirm with
Jauhar before building; parameter plumbing (zone-overridable PHI_MAX) is the same
either way.

## (21) Cutoff sensitivity crossplots (slides 84–87)

Two methods, both to reproduce:

- **Method 1 — pay-sensitivity sweep** ("Pay Volume of Shale / Pay Porosity / Pay
  Water Saturation Sensitivity Crossplot"): X = candidate cutoff value swept across
  its range (VSH 0→1, PHIE descending, SW 0→1); Y = normalized cumulative pay metric
  (EHCNM_*: HC column/HCPV captured, normalized 0–1), one curve per well per
  interval, computed with the OTHER two cutoffs fixed; **filtered on DST intervals
  only**. Cutoff picked at the curve elbow/plateau. Implementation reuses the
  existing paysummary engine in a sweep loop (~50–100 evaluations per property).
- **Method 2 — DST-calibrated crossplots**: PHIE vs VOL_WETCLAY and PHIE vs SW, all
  samples black, DST-tested intervals colored per well, red crosshair lines at the
  candidate cutoffs. Needs: interval-highlight overlay on the existing crossplot
  (perforation/DST interval sets are already importable) + draggable cutoff
  crosshairs.
- Result of the study (for scale): per-zone cutoffs Post-Main 0.70/0.100/0.85,
  Main 0.65/0.085/0.85, Massive 0.65/0.075/0.80, Talang Akar 0.50/0.050/0.75
  (Vwetclay/PhiE/Sw). Picked cutoffs should write into the pay-summary defaults
  (per zone).

## (22) Map pane with editable polygons → well groups

No deck reference — workflow need. Current code has **no well surface coordinates
and no map view** (checked: wells table/header lack X/Y; no map panel). Needs:
(a) surface X/Y (+ CRS/units note) on the well header (editor exists in Data ▾
Tools) + LAS/DLIS header harvest where present + CSV import;
(b) a Map pane (plotCanvas: posted wells by X/Y, name labels, zoom/pan, active-group
awareness);
(c) polygon draw/edit (click to add vertices, drag to move, insert/delete, close),
persisted as `polygon` documents;
(d) "assign wells in polygon → well group" (point-in-polygon → existing well-groups
CRUD), the actual goal: polygon-based well grouping for per-area parameter work.

## Dependencies / order

17 → 19 (gas density needs P & T). 18 standalone (feeds SandiMin params). 20 small,
standalone. 21 standalone (paysummary + crossplot). 22 standalone (UI + schema).
Suggested build order: 17, 18, 19, 21, 20, 22 (or 22 anytime as a UI change of pace).
