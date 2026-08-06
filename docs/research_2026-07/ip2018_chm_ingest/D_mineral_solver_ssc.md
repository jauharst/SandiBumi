# D — Mineral Solver & the Sand-Silt-Clay / Malay Model (IP 2018 help ingest)

**Source:** Interactive Petrophysics 2018 compiled help (vendor: PGL / Lloyd's Register / Geoactive),
decompiled to `C:\Users\ARUNIKA\AppData\Local\Temp\c18\_text\`. Install `C:\Program Files\IP2018` is
read-only and was not touched.

**Pages read (6):** `mineralsolver.htm`, `mineral_solver.htm`, `minsolveeqandmeth.htm`,
`minsolvecalibrate.htm`, `plot_the_mineral_solver_result.htm`, `sand_silt_malay_model.htm`.
Cross-checks in `default_settings.htm`, `tools.htm`, `interpretation.htm`,
`specialinterpretation.htm`, `intro_whats_new_in_ip.htm`.

**Numeric discipline:** every number below is transcribed exactly as printed on the cited page.
Nothing is rounded, converted, normalised, or supplied from outside the manual. Where the manual
does not state a value the entry reads `not stated in manual`.

---

## 0. Extraction status — what is recoverable and what is not

| Page | Rasterized equation images | Recoverable from text? |
|---|---|---|
| `mineralsolver.htm` | 0 | yes — fully |
| `mineral_solver.htm` | 0 | **yes — fully. This is the richest page: it prints all 11 Sw equations in plain ASCII** |
| `minsolveeqandmeth.htm` | 99 (`embim213`–`embim311`) | narrative only; every formula is a GIF |
| `minsolvecalibrate.htm` | 0 | yes — fully |
| `plot_the_mineral_solver_result.htm` | 0 | yes — fully |
| `sand_silt_malay_model.htm` | 1 `embim` **+ 24 `_intclip*.png`** | narrative only; see below |

**Important extraction finding.** The extractor's `# equation_images:` header counts only
`embimNN` files. The Sand/Silt Malay page carries its mathematics as *clipboard screenshots*
named `_intclipNNNN.png`, so its header reads `equation_images: 1` while **~13 of its 24 images are
in fact equations**. Verified by parsing `sand_silt_malay_model.htm` directly. Precise map:

| Image | What sits at that position | Kind |
|---|---|---|
| `_intclip0046.png` | Sand Silt Malay model Neutron/Density crossplot | figure |
| `_intclip0047/0048.png` | module dialog | screenshot |
| `_intclip0069.png` | Zones tab | screenshot |
| `_intclip0049–0054.zoom70.png` | interactive plot / crossplots | screenshots |
| `_intclip0055.png` | methodology **Flow Diagram** | figure |
| `_intclip0056.png` | step 2 — **Vsh from GR** | equation |
| `_intclip0057.png` | step 3 — **apparent HC density** | equation |
| `_intclip0058.png` | step 3 — **apparent HC neutron HI** | equation |
| `_intclip0059.png` | step 5 — **Vsh from Neutron-Density** | equation |
| `_intclip0060.zoom20.png` | step 8 — N/D matrix-line projection diagram | figure |
| `_intclip0061.png` | step 8 — **Lithology Conversion Chart** | figure — **Tier C, see §7** |
| `_intclip0062.png` | step 13 — **Archie PhiT** | equation |
| `_intclip0063.png` | step 13 — **Dual Water** | equation |
| `_intclip0064.png` | step 13 — **Juhasz (Waxman-Smits)** | equation |
| `_intclip0065.png` | step 13 — **Waxman-Smits** | equation |
| `_intclip0066.png` | step 13 — **Qv = a/PhiT + b** | equation |
| `embim178.gif` | step 13 — **B(T, Rw) formula** | equation |
| `_intclip0067.png` | step 13 — **effective Sw from total Sw** | equation |
| `_intclip0068.png` | step 14 — **Sxo from invasion factor, WBM** | equation |

All of the above are marked `rasterized - not recoverable`. None was reconstructed.

---

## 1. Solver structure

### 1.1 Two solvers, run in series

| | Linear solver | Non-linear solver |
|---|---|---|
| Technique | **Singular Value Decomposition** | **DNOPT Dense Nonlinear OPTimizer** |
| Attribution given | "Numerical Recipes (Cambridge University Press)" | "from Stanford University" |
| Role | solves normalised linear equations; fast and stable | seeded by the linear solution, searches for a lower total error |
| Sw handling | resistivity/conductivity limited to a **linearised Archie with n = m** | uses the **actual Sw equation** from Zonal/Model Parameters |
| Sonic handling | Wyllie with a compaction-factor approximation | "full sonic equations … without having to use the compaction factor approximation" |

The manual states: *"The non-linear optimizer's solution is always checked against the linear solvers
solution and the best (lowest total error) solution is always selected"* (stated three separate times,
`minsolveeqandmeth.htm`, `mineral_solver.htm`).

### 1.2 What is minimised

The objective is the **Total Model Error**, described in words as the normalised misfit between each
input curve and its reconstruction from the solved volumes:

> Terms named on the page: `Crv_i` (ith input curve value), `Crv_Rec_i` (ith reconstructed curve from
> volumes), `Crv_Tol_i` (ith input curve tolerance), `NumCrvs` (number of input curves in the model).

Formula: `[[EQUATION_IMAGE: embim213.png]]` — **rasterized, not recoverable**.

A second, legacy metric is also emitted: the **Total Linear Model Error**, "for backwards
compatibility with versions of IP prior to IP4.4", over `InputLog_i`, `Reconstructed_i`,
`Confidence Weight_i`. Formula `[[EQUATION_IMAGE: embim214.gif]]` — **rasterized, not recoverable**.

**Resistivity/conductivity special-casing (stated in text, important):** "all resistivities are first
converted to conductivities. Then the terms `Crv_i`, `Crv_Rec_i`, and `Crv_Tol_i` have the square root
taken of them before applying them in the above equation."

### 1.3 The linear-solver algorithm, as stated (6 steps)

1. Each equation is normalised by dividing all terms by its confidence weighting.
2. Equations are solved by Singular Value Decomposition.
3. **Negative-volume handling:** if any volume result is negative, the *largest negative term is set to
   zero and removed from the model*, the solver re-runs, and this repeats until all volumes are positive.
4. Result volumes are adjusted to sum to 1.0. "Due to the way the equation solver works, the unity
   equations will not necessarily force the results to absolutely 1.0. (The tolerance of the unity
   equation is set at **0.01** by default)."
5. Input logs are reconstructed from the volume results.
6. Total normalised error is computed.

### 1.4 How tool responses are combined

Every equation is the same linear form, printed twice in the manual:

```
Y = Vol1 x Min1 + Vol2 x Min2 + Vol3 x Min3 ...
```

where `Y` = input tool curve or fixed value, `Vol_i` = solved volumes, `Min_i` = mineral end-point
values (the response of 100 % of that mineral). Non-linearity is handled by **relinearisation inside an
outer iteration loop**: "The non-linear equations are made linear by adjusting their input end-point
mineral values knowing what the porosity and water saturation values are."

### 1.5 Weighting / uncertainty

- Weighting is per-equation and is called **Confidence**. "The smaller the confidence number, the more
  the weight that will be attached to this equation (**exception to this is the resistivity equation**)."
- Confidence is expressed **in the physical units of the equation** — "Density confidence would be in
  gm/cc, GammaRay in API units" — i.e. it is a tool-accuracy figure, not an abstract weight.
- Confidence may be a **curve**, not just a scalar, so it can vary level by level (explicitly recommended
  for washed-out hole where the density tool degrades).
- For conductivity/resistivity equations the confidence is transformed with the curve: it "has 1/m th
  root taken of it before using in the solver". Worked example printed on `minsolveeqandmeth.htm`:

  | Quantity | Conductivity example | Resistivity example |
  |---|---|---|
  | Input | 500 mmho | 2 ohmm = 500 mmho |
  | Input confidence | 5 mmho | 200 ohmm = 5 mmho |
  | m | 2 | 2 |
  | Solver input value | 22.3 (1/m th root) | 22.3 (1/m th root) |
  | Solver confidence | 2.24 | 2.24 |
  | + error | (22.3 + 2.24)^2 = 602.2 mmho | … = 602.2 mmho = 1.661 ohmm |
  | − error | (22.3 − 2.24)^2 = 402.4 mmho | … = 402.4 mmho = 2.485 ohmm |

  (Transcribed as printed. Note the manual's own resistivity row labels the +error as the *lower* ohmm
  value — an inversion in the source, not corrected here.)
- Output uncertainty curves are written per input curve: `***_re` (reconstructed), `***_me`
  (minus-error), `***_pe` (plus-error), where `***` is the input curve name.

### 1.6 Constraint handling

| Constraint | Form | Notes |
|---|---|---|
| **Unity** | `1 = Vol1 + Vol2 + Vol3 …` | always included; tolerance 0.01 |
| **Porosity equalisation** | `0 = -VwatU - VhydU1 - Vhyd2U - … + Vwat + Vhyd1 + Vhyd2 + …` | auto-added when both invaded and un-invaded fluids are present |
| **Sxo equation** | `0 = ( Sxo-1 ) Vwater + Sxo * Vhydrocarbon1 + Sxo * Vhydrocarbon2 + ...` | auto-added by default; default confidence **0.01** ("1 saturation unit") |
| **Sw equation** | `0 = ( Sw-1 ) VwaterU + Sw * VhydrocarbonU1 + Sw * VhydrocarbonU2 + ...` | auto-added optionally; default confidence **0.01** |
| **Constant** | user linear relation, e.g. `0.02 = VPyrite` | recommended confidence "about 0.01" = "a 1% volume error" |
| **`<Limit` / `>Limit`** | inequality on a mineral/fluid or a group | see below |
| **Invasion Factor** | per-equation, 0.0–1.0 | endpoint × IF for invaded fluids, × (1−IF) for un-invaded; may be a curve |

**Limit-equation algorithm (stated explicitly):** IP solves the model *without* limits, validates each
limit, and if a limit is violated **adds that one equation and completely re-solves**, then repeats —
"The limit equations are added one at a time and the model is completely resolved before adding another
equation. The order that equations are added is the order that they are entered into the grid."
Limits are then treated as constant equations weighted by their Confidence, so "the results could still
be outside the limits of the equation". The manual warns limits should make **minor** adjustments only.

Constant-equation idioms printed as text:
- fixed volume: `0.02 = VPyrite`
- mineral ratio (Orthoclase = 20 % of Quartz): `0 = (0.2 * Quartz) - Orthoclase`
- linear assemblage relation `VGlau = VQtz * a + b` → entered as `b = VGlau - a * VQtz`

### 1.7 Convergence controls and their defaults

| Control | Value as printed | Page |
|---|---|---|
| Outer loop: effective-porosity convergence | `e difference < 0.001` (printed "ϕ [image] e difference < 0.001") | `minsolveeqandmeth.htm` |
| Outer loop: Sxo convergence | `Sxo difference < 0.002` | `minsolveeqandmeth.htm` |
| Main linearization loop iteration cap | 20 iterations (PHIFLAG 4) | `minsolveeqandmeth.htm` |
| Solver iteration cap | 30 iterations (PHIFLAG 5) | `minsolveeqandmeth.htm` |
| Sw equation loop cap | 10 iterations (PHIFLAG 8) | `minsolveeqandmeth.htm` |
| Unity equation tolerance | 0.01 | `minsolveeqandmeth.htm` |
| Sw / Sxo auto-equation confidence | 0.01 | `mineral_solver.htm` |
| Constant-equation confidence (recommended) | about 0.01 | `minsolveeqandmeth.htm` |
| Sonic Cp (Wyllie compaction factor) | Default 1.0 | `mineral_solver.htm` |
| `Cm*` | Default is 1.0 | `mineral_solver.htm` |
| `B fact Juhasz` | Default 1.0 **meq/ml** | `mineral_solver.htm` |
| `Sxo Limit` exponent | The default value is 0.2 | `mineral_solver.htm` |
| Invasion factor (OBM branch) | The default value is 0.5 | `mineral_solver.htm` |
| Invasion Factor (Sxo-from-Sw empirical) | The default value of the Invasion Factor is 2.0 | `minsolveeqandmeth.htm` |
| Max models per well | Up to 20 | `mineral_solver.htm` |
| Max mixing rule sets | Up to 5 | `plot_the_mineral_solver_result.htm` |
| Max mineral end-points per crossplot | 8 | `mineral_solver.htm` |

> **Conflict recorded, not resolved:** the invasion factor default is printed as **0.5** on
> `mineral_solver.htm` (OBM context) and as **2.0** on `minsolveeqandmeth.htm` (the WBM empirical
> `Sxo(Sw, IF)` relation). Both are transcribed. They are different parameters in different contexts,
> but the manual reuses the name. Do not merge them.

### 1.8 Quality / incoherence reporting

Two output curves carry solution quality:

- **`TotErr`** — Model Normalized Total Error. "Error values greater than 1.0 are displayed in red."
- **`PHIFLAG`** — logic/incoherence flag. "For a normal execution of IP at any depth level, the PHIFLAG
  should be zero." Full table as printed:

| PHIFLAG | Logic |
|---|---|
| 2 | A Limit equation was used in the model results |
| 4 | Main linearization loop did not converge after 20 iterations |
| 5 | Solver did not converge after 30 iterations |
| 6 | Sw curve set to 1.0 due to Phi Sw Limit or Vcl Sw Limit parameter limits being reached |
| 7 | Sw limited to Sw irreducible parameter |
| 8 | Sw equation loop did not converge after 10 iterations |
| 9 | Fatal Error with non-linear optimizer. See error log |
| 10 | Non-linear optimizer. Unbounded Objective |
| 11 | Non-linear optimizer. Iteration limit reached |
| 12 | Non-linear optimizer. Major iteration limit reached |
| 14 | Non-linear optimizer. Terminated during objective evaluation |

(Values 0, 1, 3 and 13 are not listed in the table.) Interpretation guidance as printed: "Error 9 is
serious and the level results will be the linear solvers solution. Errors 10-14 indicate the optimizer
has had problems finding an optimal solution, however the results are probably still ok and will be
better than the linear solution."

Visual QC: one reconstruction track per input equation, original curve + **yellow Confidence Band** +
reconstructed curve in red. "The closer the reconstructed curve is to the original curve, the better the
Model and the results fit and the lower the Total Error will become."

### 1.9 Multi-model combination (Mixings)

- Up to 20 models per well, combined zone-by-zone by a **Mixing** (up to 5 rule sets), each with a
  Default Model plus sequential logic rules evaluated top-down, first true wins.
- **`Mdl Merge Dist`** smooths model transitions. Worked example as printed: "if the `Mdl Merge Dist` is
  set to 2.0 in a 0.5 step Set then the transition will occur over 4 samples … the first depth step of
  the transition will take 20% of the volumes from Mdl1 and add those to 80% of Mdl2. The second step
  will take 40% … 60 % … third 60%/40% … fourth 80%/20%."
- Implementation stated: set each `Model_Num` array column to 1/0, box-filter each column with filter
  length = `Mdl Merge Dist`, then normalise columns to 1.0 per level.
- Transitioned: Porosity, BVW, Sw, Sxo, Vcl, mineral volumes. **Not** transitioned: the combined
  reconstructed curves.

---

## 2. Mineral end-points

### 2.1 The headline finding — no end-point table is printed

**The IP 2018 help does not publish a mineral end-point table anywhere in the Mineral Solver chapter.**
Verified by reading all six pages and by sweeping the whole 279-page corpus for the usual anchors
(`Kaolinite`, `Illite`, `Chlorite`, `Smectite`, `Montmorillonite`, `Anhydrite`, `Siderite`, `Pyrite`,
`2.65`, `2.71`). The corpus hits are all prose parameters in other modules, not a solver end-point table.

Instead the manual points at two **external, user-editable ASCII files in the IP install directory**:

| File | Contents, as described | Reached via |
|---|---|---|
| **`MINDEF.PAR`** | "the default minerals and their properties for the Mineral Solver module … The user can add extra minerals and their matrix properties to the table, or edit the properties of existing minerals." Also supplies the mineral grain densities used by the Preprocessor's dry-weight→volume conversion. | Tools → Defaults → Edit Mineral Solver Mineral System Defaults |
| **`MINEQDEF.PAR`** | "the mineral Equation default settings … Any equation that is selected in the Model dialog is set up according to the parameters in this file. New equation types can be added to the list." | Tools → Defaults → Edit Mineral Solver Mineral Equation Defaults |

The two "work together and allow you to define new minerals with new default values for the input
equations." Behaviour stated: selecting the **Equation type first** then picking Minerals auto-populates
end-points from these files; "Names that are not in the drop-down list will not have any default
end-point values defined."

**→ Open cross-check for the caller (not guessed here):** IP 2025's `MINDEF.PAR` is already registered
in this reference suite as a third independent endpoint source that validates SandiMin. Whether IP
2018's `MINDEF.PAR` uses the same column convention, the same mineral roster, and the same values as IP
2025's is **a question, not a finding** — it cannot be answered from the help text, only by diffing the
two `.PAR` files. IP 2018's file is at `C:\Program Files\IP2018\MINDEF.PAR` (read-only).

### 2.2 Every numeric mineral/material response the manual does print

All of these are illustrative values inside worked examples, not a defaults table. Transcribed exactly.

| Mineral / material | Property | Value as printed | Units as printed | Context | source_page |
|---|---|---|---|---|---|
| Calcite | density end-point | 2.71 | g/cc | "100% Calcite density would be 2.71 g/cc" | `mineralsolver.htm` |
| Fresh water | density end-point | 1.0 | g/cc | "for 100% fresh water the density would be set to 1.0 g/cc" | `mineralsolver.htm` |
| Fresh water | density (fixed end-point in calibration) | 1.0 | gm/cc | "the density of fresh water at 1.0 gm/cc" | `minsolvecalibrate.htm` |
| Wet Clay | **dry** grain density (ECS example) | 2.78 | not stated in manual (grain-density context) | "The density of wet clay has been entered as 2.78 which is the dry grain density not the wet grain density" | `minsolveeqandmeth.htm` |
| Wet Clay | `ECS_Clay (Wt%)` end-point | 0.85 | v/v | wet-clay model with clay total porosity 0.15 | `minsolveeqandmeth.htm` |
| Wet Clay | clay total porosity in that example | 0.15 | v/v | `ECS_Clay (Vol) = ECS_Clay (Wt%) x (1 - 0.15) x (2.78 / 2.78)` | `minsolveeqandmeth.htm` |
| Dry Clay | `ECS_Clay` end-point | 1.0 | v/v | "For a dry clay model the ECS_Clay parameter would be 1.0" | `minsolveeqandmeth.htm` |
| Illite | wet-clay total porosity (`PhiTClay` end-point) | 0.156 | v/v | PhiTClay output-equation example | `minsolveeqandmeth.htm` |
| Chlorite | wet-clay total porosity (`PhiTClay` end-point) | 0.101 | v/v | same example | `minsolveeqandmeth.htm` |
| Kaolinite | wet-clay resistivity (`ResClay` end-point) | 2.5 | ohmm | ResClay output-equation example | `minsolveeqandmeth.htm` |
| Illite | wet-clay resistivity (`ResClay` end-point) | 1.2 | ohmm | same example | `minsolveeqandmeth.htm` |
| Calcite | `m` cementation end-point (`Output Para`) | 1.8 | dimensionless | variable-m output example | `minsolveeqandmeth.htm` |
| Quartz | `m` cementation end-point (`Output Para`) | 2.05 | dimensionless | same example | `minsolveeqandmeth.htm` |
| Clay | conductivity end-point | 250 | mmhos | conductivity-equation setup example | `minsolveeqandmeth.htm` |
| Clay | resistivity end-point | 4.0 / 4 | ohmm | same example ("250 mmhos or 4.0 ohmm"; "Clay resistivity has been set at 4 ohmm") | `minsolveeqandmeth.htm` |
| Mud filtrate (Rmf) | conductivity | 16,000 | mmhos | same example | `minsolveeqandmeth.htm` |
| Mud filtrate (Rmf) | resistivity | 0.063 | ohmm | same example | `minsolveeqandmeth.htm` |
| Resistivity equation | confidence used in example | 200 | ohmm | "The confidence has been set at 200 ohmm" | `minsolveeqandmeth.htm` |
| Pyrite | volume, constant-equation example | 0.02 | v/v | "2% of the rock contains Pyrite" | `minsolveeqandmeth.htm` |
| Bound water in wet clay | volume, constant-equation example | 0.15 | v/v | | `minsolveeqandmeth.htm` |
| (porosity) | `PhiLimit` max example | 0.3 | v/v | "This will set the maximum porosity to 0.3" | `minsolveeqandmeth.htm` |
| (porosity) | `PhiLimit` bad-hole example | 0.05 | v/v | washed-out-hole failure illustration | `minsolveeqandmeth.htm` |
| Calcite | max-volume limit example | 0.2 | v/v | "Setting a maximum limit of 0.2 Calcite" | `minsolveeqandmeth.htm` |
| Orthoclase / Quartz | ratio, constant-equation example | 0.2 | fraction | "If Orthoclase is 20% of Quartz" | `minsolveeqandmeth.htm` |
| (permeability) | calibration discriminator example | 0.1 | mD | "select only those data where the permeability is greater than 0.1mD" | `minsolvecalibrate.htm` |

Density end-point values for `Neu Matrix`, `Den Dry Clay`, `Den Silt`, `Den Wet Clay`, GR API
end-points, Sigma end-points, PEF end-points, U end-points, and CEC values are **not stated in manual**.

### 2.3 IP's stated end-point conventions

| Convention | What the manual states |
|---|---|
| **Auto end-points** | Cells with a **blue** background can be auto-calculated; entering `Auto` triggers it. "In order for IP to use the non-linear equations for the Neutron curve, the **Calcite, Quartz and Dolomite parameters must be left at Auto**." |
| **Water end-points** | With `Auto`, water density and HI are computed from the **Rmf** parameter corrected to depth/temperature; for OBM or the un-invaded zone, from **Rw**. For conductivity/resistivity equations, `Auto` on a `Water Sxo` end-point is computed from `Rmf` and `Rmf Temp`. |
| **Hydrocarbon end-points** | Cells with a **green** background take **down-hole *true* hydrocarbon density**; IP converts it internally to the tool response (electron density for RHOB, hydrogen index for NPHI). |
| **End-points may be curves** | "Any end-point value can be entered as a curve" — Trend curves vary a parameter level by level. |
| **GR** | There is **no dedicated GR equation type**. GR enters as an ordinary linear equation with a per-mineral end-point in API units ("GammaRay in API units" is used as the Confidence-unit example). Spectral GR is handled differently — see Wt% below. GR is therefore a **direct end-point**, not composed from K/Th/U. |
| **Spectral GR / ECS / XRD** | Handled by the **`Wt%` equation family**: "Spectral Gamma Ray tools output curves that measure dry weight percent results either in the form of minerals or actual elements. In order to use them inside the mineral solver these curves need to be converted into volumes." Conversion is automatic when the equation type has `Wt%` in the name; the curve is divided by 100 if its units are `%` or `pec`. |
| **CEC / Qv** | Qv is defined as "**Cation exchange capacity per unit total pore volume**" and Qvn as "'Normalized' cation exchange capacity per unit total pore volume" — i.e. **per unit volume, not per gram**. The `Qv` output-equation end-points "can be calculated from the CEC of the clay and its density and total porosity" — but that CEC→end-point conversion is `embim257`–`embim259`, **rasterized, not recoverable**, and the manual never states the CEC unit (meq/100 g vs meq/mL). The Juhász normalized B default is printed as **1.0 meq/ml**. |
| **Wet vs Dry clay** | "You cannot mix Wet Clay and Dry Clay types in the same model or Wet Clay and Bound Water." Bound Water can only be added to a **Dry Clay** model. Either way IP computes both Phie and PhiT, and any Sw equation may be used. |
| **Mineral Types** (drop-down, exhaustive as printed) | `Water Sxo`, `Bound Water`, `Hyd. Sxo`, `Matrix`, `Wet Clay`, `Dry Clay`, `Water Sw`, `Hyd. Sw` |

### 2.4 Dry-weight → wet-volume conversion (printed twice, identically)

```
Wet Vol % = (Dry Weight %) x (1 - Porosity) x (Rock Grain Density) / (Mineral Grain Density)
```
(`mineralsolver.htm`; the `minsolveeqandmeth.htm` Wt% section prints the same with `PhiT` in place of
`Porosity`.) Rock Grain Density and Porosity come from routine core analysis; Mineral Grain Density from
`MINDEF.PAR` or a chart book. The manual notes "The Porosity part of this correction is usually the most
important input."

### 2.5 The one real response table the manual prints — neutron tool look-up

This is the only tabulated numeric tool-response data in scope D. Printed as an example of the ASCII
`.neu` look-up files (`minsolveeqandmeth.htm`), transcribed **verbatim**:

```
$ IP
$
$ Sch_CNL.neu file
$ Contains lookup table for Schlumberger CNL TNPH
$
$
$ Data is as follows
$ True Phi, Sandstone Matrix, Dolomite Matrix, Salinity corr Sand, Salinity corr Lime, Salinity corr Dol
$ Salinity correction are for following values 50, 100, 150, 200, 250 Kppm and in this order
$ Porosity values must not be changed
$
$phi ss    Dol   50 SS  100    150    200    250    50 LS  100    150    200    250    50 Dol 100    150    200    250
.00 .020 -.006 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000 .0000
.02 .022 -.009 -.0012 -.0032 -.0040 -.0056 -.0060 -.0012 -.0024 -.0032 -.0036 -.0040 -.0020 -.0044 -.0064 -.0076 -.0092
.05 .028 -.012 -.0030 -.0080 -.0100 -.0140 -.0160 -.0030 -.0060 -.0080 -.0090 -.0100 -.0050 -.0110 -.0160 -.0190 -.0230
.10 .036 -.017 -.0070 -.0140 -.0190 -.0230 -.0280 -.0060 -.0120 -.0160 -.0200 -.0210 -.0100 -.0190 -.0270 -.0330 -.0380
.15 .045 -.022 -.0120 -.0220 -.0290 -.0340 -.0350 -.0100 -.0190 -.0260 -.0300 -.0340 -.0120 -.0240 -.0330 -.0400 -.0450
.20 .049 -.030 -.0160 -.0270 -.0340 -.0370 -.0380 -.0140 -.0260 -.0320 -.0350 -.0390 -.1960 -.0290 -.0370 -.0420 -.0440
.25 .051 -.037 -.0160 -.0370 -.0330 -.0340 -.0330 -.0160 -.0290 -.0350 -.0390 -.0390 -.0180 -.0300 -.0370 -.0400 -.0410
.30 .052 -.045 -.0150 -.0350 -.0290 -.0290 -.0270 -.0200 -.0310 -.0380 -.0380 -.0380 -.0180 -.0300 -.0340 -.0360 -.0330
.35 .049 -.052 -.0130 -.0180 -.0190 -.0190 -.0150 -.0200 -.0310 -.0350 -.0350 -.0330 -.0160 -.0260 -.0290 -.0280 -.0260
.40 .046 -.059 -.0100 -.0140 -.0140 -.0120 -.0070 -.0180 -.0280 -.0310 -.0300 -.0270 -.0150 -.0230 -.0240 -.0210 -.0180
.45 .043 -.066 -.0120 -.0140 -.0140 -.0100 -.0030 -.0170 -.0240 -.0280 -.0260 -.0220 -.0150 -.0220 -.0210 -.0170 -.0130
.50 .040 -.073 -.0110 -.0120 -.0110 -.0060 .0020 -.0160 -.0220 -.0250 -.0220 -.0170 -.0140 -.0200 -.0180 -.0130 -.0080
.55 .037 -.080 -.0100 -.0100 -.0080 -.0020 .0070 -.0150 -.0200 -.0220 -.0180 -.0120 -.0130 -.0180 -.0150 -.0090 -.0030
.60 .034 -.087 -.0090 -.0080 -.0050 .0020 .0120 -.0140 -.0180 -.0190 -.0140 -.0070 -.0120 -.0160 -.0120 -.0050 .0020
```

Stated conventions for these files:
- "It is assumed that the entered neutron porosity is in **Limestone units**."
- Columns 2–3 are the **sandstone and dolomite matrix offsets**; columns 4–18 are salinity corrections
  at 50/100/150/200/250 kppm for sand, lime, dolomite in that order.
- "The values of porosity go up to 60 pu., and it is necessary to complete the table up to this value,
  even though it is unlikely that there are any published results for these high porosities."
- New tools are added by registering them in `Neu_Parm_Files.neu` and writing a new `xxx.neu` table.
  "The spacing between the parameters is not important, but the number of parameters in each line and
  the number of porosity lines are." "Porosity values must not be changed."

> **Suspected typo, transcribed but flagged, NOT corrected:** at `phi = .20`, the *Dolomite / 50 kppm*
> entry reads `-.1960`, roughly 20× its neighbours (`.15` → `-.0120`, `.25` → `-.0180`) and 100× out of
> family with the equivalent sand/lime cells. This is almost certainly `-.0196` mis-keyed in the vendor
> manual. Do not silently repair it — verify against the shipped `Sch_CNL.neu` before use.
> A second, milder oddity: at `phi = .25` the sand 100 kppm cell (`-.0370`) is larger in magnitude than
> the 150/200/250 cells in the same row, breaking the monotonic trend of every other row.

---

## 3. Tool-response equation inventory

Legend — **Tier A** = reference data / method inventory, free to adopt. **Tier B** = published citable
science. **R** = formula is rasterized and was not reconstructed.

### 3.1 Model (input) equation types

| # | Equation name | Inputs | Rasterized? | Tier | Notes / citation |
|---|---|---|---|---|---|
| 1 | **Density** | RHOB curve; per-mineral density end-points | R (`embim215`–`embim218`) | A | Linear. HC and water end-points auto-calculable. Two HC-density models — see §3.4 |
| 2 | **Neutron** | NPHI curve (assumed limestone units); end-points | R (`embim219`–`embim223`) | A | **Non-linear** for sand/lime/dolomite via `.neu` look-up tables; water HI from salinity look-up; HC HI adjusted for **excavation effect**. Named terms: `Vcl`, `NeuCl`, `Exfact`, `NeuSal`, `Sx`, `NeuHyHI` |
| 3 | **Sonic** | DT curve; Dtma/Dtfl/Dthy/Dtcl end-points | R (`embim224`–`embim228`) | B | Wyllie time-average (linear when Cp = 1.0) or **Hunt-Raymer** (non-linear, hard-coded) |
| 4 | **EPT** | TPL curve | R (`embim229`–`embim231`) | A | Flushed-zone Sw directly in the model; water end-point `Auto` from filtrate salinity. Inputs named: `Sal` (ppm 10⁻⁶), `T` (°F), `Rmf` at formation temperature; Rw substitutes for Rmf in OBM |
| 5 | **Wt% family** (e.g. `ECS_Clay (Wt%)`, TOC) | dry weight-% curve + Grain Density output equation | no — printed in text | A | `Wet Vol % = (Dry Weight %) x (1 - PhiT) x (Rock Grain Density) / (Mineral Grain Density)` |
| 6 | **U** (volumetric photoelectric cross-section) | U curve (computed outside the module from PEF+RHOB) | partly (`embim232`–`embim236`); water branch printed | A | `U wat = 0.00481 x Sal + 0.3883`. Gas branch (Input Hyd Den < 0.4) and oil branch are rasterized. Pre-processor: `U = Pef x (RHOB + 0.1883) x 0.93423` |
| 7 | **Cond. Cxo** | flushed-zone conductivity curve | R | A | Non-linear solver: uses the *actual* Sw equation |
| 8 | **Cond. Ct** | un-invaded conductivity curve | R | A | Invasion Factor must be 0.0; `Sw Method` → `Min Model`; auto-Sw equation off |
| 9 | **Res. Rxo** | flushed-zone resistivity curve | R | A | as Cond. Cxo but in resistivity |
| 10 | **Res. Rt** | deep resistivity curve | R | A | as Cond. Ct but in resistivity |
| 11 | **Cxo Archie Lin** | Cxo + water/mineral conductivity end-points | R (`embim237`–`embim241`) | B | Linearised Archie with **n = m**; curve, end-points and confidence all raised to the 1/m th root. Uniquely allows **conductive minerals (e.g. Pyrite) to carry a resistivity** |
| 12 | **Rxo Archie Lin** | Rxo + resistivity end-points | R | B | as above |
| 13 | **Ct Archie Lin** | Ct | R | B | un-invaded variant |
| 14 | **Rt Archie Lin** | Rt | R | B | un-invaded variant |
| 15 | **Constant** | fixed value or curve = linear combination of volumes | text | A | recommended confidence ≈ 0.01 |
| 16 | **BoundWater** (constant preset) | clay total porosity per clay mineral | R (`embim242`–`embim244`) | A | End-point = clay total porosity / (1 − clay total porosity), "to take into account that the output volumes are dry clay volumes" |
| 17 | **Unity** | — | text | A | `1 = Vol1 + Vol2 + Vol3 ….`, always present |
| 18 | **Sxo** (auto-added) | Sxo from Rxo via the selected Sw equation | text + R (`embim245`–`embim247`) | A | `0 = ( Sxo-1 ) Vwater + Sxo * Vhydrocarbon1 + …` |
| 19 | **Sw** (auto-added) | Sw from Rt via the selected Sw equation | text + R (`embim248`–`embim250`) | A | `0 = ( Sw-1 ) VwaterU + Sw * VhydrocarbonU1 + …` |
| 20 | **PhiLimit** | limit value | R (`embim253`) | A | preset limit equation; auto-configures when selected |
| 21 | **Sigma** (cased hole) | sigma curve | **not described** | A | The cased-hole recipe is given (`Sw Method` = `No Calc`, `Sxo Method` = `Min Model`, Rt/Rxo absent, Rmf still entered for neutron auto-parameters, both auto-Sw/Sxo equations off) but **no sigma equation type, end-point convention or Sigma unit is named anywhere in scope D** |

### 3.2 Output-only equation types

| Equation | Purpose | Rasterized? | Tier |
|---|---|---|---|
| **Grain Density** | rock grain density from included minerals; excluded minerals/fluids get end-point 0.0 | R (`embim255`) | A |
| **Qv** | Qv curve for Waxman-Smits / Dual Water; end-points from clay CEC, density, total porosity | R (`embim256`–`embim259`) | A |
| **PhiTClay** | clay total-porosity curve (wet-clay models only); end-points = PhiT in 100 % wet clay per clay mineral | R (`embim260`–`embim261`) | A |
| **ResClay** | clay resistivity curve for effective-porosity shaly-sand equations | R (`embim262`) | A |
| **Output Para** | generic parameter curve (e.g. a variable `m` blended between mineral end-points) | R (`embim263`) | A |
| **SaturationEff** | effective saturation of selected fluids (set 1.0 on the fluids to include) | R (`embim251`) | A |
| **SaturationTot** | total saturation of selected fluids | R (`embim252`) | A |
| generic linear output | any linear equation | R (`embim254`) | A |

Behavioural note worth copying: output curves "are calculated **after each loop** of the mineral solver
and **before** the water saturation calculations are made", so a `Qv` / `PhiTClay` / `ResClay` output can
be fed straight back into the Sw parameter grid. Wiring rule: reference the curve with **no set name or
a set name that does not exist** (e.g. `Model:Qv_ms`) so IP resolves it to the *current model's* output
set rather than a global default.

### 3.3 Water-saturation equations — 11, printed in plain ASCII on `mineral_solver.htm`

Transcribed exactly as printed. These are the **only** unrasterized versions in the chapter
(`minsolveeqandmeth.htm` renders the same set as `embim271`–`embim283`).

| Equation | As printed | Citation given in manual |
|---|---|---|
| Archie | `1/Rt = Phi**m.Sw**n / (a.Rw)` | none |
| Archie PhiT | `1/Rt = PhiT**m.Sw**n / (a.Rw)` — "Same as Archie except PhiT used instead of Phie." | none |
| Simandoux | `1/Rt = Phi**m.Sw**n / (a.Rw) + Vcl.Sw / Rcl` | none |
| Modified Simandoux | `1/Rt = Phi**m.Sw**n / (a.Rw.(1-Vcl)) + Vcl.Sw / Rcl` | none |
| Indonesian (Poupon-Leveaux) | `1/Rt**.5 = ((Phi**m/(a.Rw))**.5 + Vcl**(1-Vcl/2)/Rcl**.5 ).Sw**(n/2)` | none |
| Dual Water | `1/Rt = PhiT**m.SwT**n/a.(1/Rw + Swb/SwT(1/Rwb-1/Rw))` | none |
| Juhasz | `1/Rt = PhiT**m.SwT**n.(1+Bn.Qvn/SwT)/(a.Rw)`; `Qvn = Vcl*PhiClay / PhiT`; `Bn = B normalized, entered parameter` | none |
| Waxman-Smits | `1/Rt = PhiT**m.SwT**n.(1+B.Qv/SwT)/(a.Rw)`; `Qv = a / PhiT + b`; B entered or calculated from T and Rw | none (B formula is `embim282`, rasterized) |
| Woodhouse Tar | `1/Rt = Sw**n.((( Vcl**(1-Vcl))/( Rcl**0.5))+((Phi**(m/2))/((a.Rw)**0.5)))**2` | **Tier B** — R. Woodhouse, *"Athabasca Tar Sands Reservoir Properties Derived from Core and Logs"*, 1976, 17th annual Logging symposium (SPWLA). "This is a modified version of the standard Indonesian (Poupon-Leveaux) equation" |
| Poupon-Aguilera | `1/Rt = Phi**m.Sw**n / (a.Rw(1-Vcl)) + Vcl / Rcl` | **Tier B** — Roberto Aguilera, *"Extensions of Pickett Plots for the Analysis of Shaly Formations by Well Logs"*, The Log Analyst, Sept-Oct 1990. "In previous versions of IP, this was simply called 'Poupon'." Exponents `n` and `m` added by IP. Assumes laminated sands and shales with clean sands |
| Poupon-Tixier | `1/Rt = (1-Vcl).Phi**m.Sw**n/(a.Rw) + Vcl/Rcl` | **Tier B** — Poupon A, Loy ME, Tixier MP (1954), *"A contribution to electric log interpretation in shaly sands"*, Trans AIME 6(06):138–145. `m` and `n` exponents added by IP. Assumes laminated sands and shales with clean sands |

Note IP's stated Vcl-vs-Vshale position, relevant to any SandiBumi doc: "In the original equation Vcl is
Vshale and Rcl is Rshale. In IP we calculate clay volumes and shale is considered a rock type. However
the equation can be used if the interpreter picks parameters to represent shale rather than clay."

### 3.4 Auxiliary relations printed as text

| Relation | As printed | Page | Note |
|---|---|---|---|
| Shell `m` | `m = 1.87 + 0.019 / Phie` | `mineral_solver.htm` | named "the Shell formula" |
| Variable `m*` — Waxman-Smits | `m* = m + Cm(1.128 Y + 0.22 (1-e**(-17.3Y)))` | `mineral_solver.htm` | `Y = Qv PhiT / (1 - PhiT)`; `Cm` input parameter, default 1.0 |
| Variable `m*` — Dual Water | `m* = m + Cm(0.258 Y + 0.20 (1-e**(-16.4Y)))` | `mineral_solver.htm` | printed once with `0.20` and once as `0.2 0` (typographic split) — recorded as printed |
| `Qv` from PhiT | `Qv = a / PhiT + b` | both | `a`, `b` pickable on the 1/PhiT–QvApp crossplot |
| `n` from EPT/Rxo | `n = m + mPlus` | `mineral_solver.htm` | bounded by `min m value` / `max m value` |
| `m` variable with Vcl | if `Vcl > Vcl cut-off` then `m = m*10**(Vcl-Vcl cut-off)` | `mineral_solver.htm` | "has the effect of removing any hydrocarbons in zones of high clay content"; applied **after** any `m*` calculation |
| Sxo from invasion factor | WBM: `Sxo = (Sw + Invasion Factor) / (1 + Invasion Factor)`; OBM: `Sxo = Invasion Factor` | `mineral_solver.htm` | |
| Sxo limit | `Sxo < Sw**SxoLimit` | `mineral_solver.htm` | default exponent 0.2 |
| U from PEF | `U = Pef x (RHOB + 0.1883) x 0.93423` | `mineralsolver.htm` | pre-processor; U must be built outside the solver |
| U of water | `U wat = 0.00481 x Sal + 0.3883` | `minsolveeqandmeth.htm` | |
| Density HC — Conventional | `DenHcApp = RhoH * 2 (10 - 2.5 RhoH) / (16 - 2.5 RhoH)` | `mineral_solver.htm` | **transcribed as printed; almost certainly corrupted** — the HTML lost superscripts, so `* 2` is likely an exponent. Do NOT implement from this transcription |
| Density HC — Modified | `DenHcApp = (5.5 * Rhohy (4-Rhohy) -3) / (16 - 2.5 RhoH)` | `mineral_solver.htm` | same caveat. "corrects better for the calibration of the density tool from electron density to apparent density" |
| Rt for Hingle plot | `Rt_Hingle = Rt^(-1/m)` | `minsolveeqandmeth.htm` | auto-computed, not in the output-curve list |
| Final calcs | `Cwapp = 1 / Rwapp`; `PhiT_recp = 1 / PhiT` | `minsolveeqandmeth.htm` | BVW, BVWsxo, Rwapp, Rmfapp, Qvn, QvApp are `embim299`–`embim310`, rasterized |
| BVWIRR limits | `BVW > BVWIRR > 0` | `minsolveeqandmeth.htm` | bulk volume irreducible water from the Rmf-equivalent residual |
| Wyllie↔Hunt-Raymer bridge | see **Tier-C flag T5** | `minsolveeqandmeth.htm` | vendor-fitted; coefficients deliberately not transcribed here |

Sw / Sxo method switches (both are enumerations worth mirroring): `Sw Method` ∈ {`Rt`, `Min Model`,
`No Calc`, `Sw = 1`}; `Sxo Method` ∈ {`Rxo`, `Min Model`, `Inv Fac`, `No Calc`, `Sxo = 1`}, plus the
`OBM ?` flag and `Sw Sxo Inv Logic` toggle (default true: `Sxo >= Sw` in WBM, `Sxo <= Sw` in OBM).

---

## 4. Secondary porosity (a free by-product worth copying)

Computed automatically whenever a Sonic equation is *present on the model grid* — the `Use` box may be
off, the sonic need not participate in the solve.

- Volume-weighted `Dtmatrix`, `Dtwater`, `Dthydrocarbon`, `Dtclay` are built from the solved volumes,
  then sonic porosity is computed with the model's own `Vclay` and `Sxo`. Wyllie form `embim296`,
  Raymer form `embim297` — rasterized.
- `PhiSecU = Phie - PhiSonic` (wet clay model); `PhiSecU = PhiT - PhiSonic` (dry clay model).
- `PhiSec` = `PhiSecU` clipped at zero.
- Stated calibration intent: adjust parameters so that "the average secondary porosity in non vuggy rock
  is zero" — `PhiSecU` is deliberately allowed negative for exactly this QC.

---

## 5. Calibration workflow (`minsolvecalibrate.htm`)

**What is calibrated against what:** the *mineral end-points* are regressed against **core mineral
volumes** (typically XRD), per equation, by **multiple linear regression**.

```
Input Curve = Min1 x Vol1 + Min2 x Vol2 + Min3 x Vol3 …
```
`Min_i` are the end-points being solved for; `Vol_i` are the input core volume curves.
"Notice that the difference between this equation and the normal multi-linear regression is that there
is **no constant term** in the result co-efficient."

Mechanics:
- Runs **per equation row**, not globally. Reports **`Corr Coeff` (R²)** and **`Num Points`** per row.
- Each mineral is **Fixed** or **Variable**. Fixed end-points are subtracted from the input curve before
  the regression runs. "It may be necessary to fix several of the mineral end-points in order to get a
  sensible result. Several iterations will normally be needed."
- **Multi-well**: any well loaded in memory, with per-well top/bottom depth and a per-well `Use` toggle
  "to quickly evaluate the influence of individual wells."
- **Discriminators** filter the calibration data (example given: permeability > 0.1 mD). "If using log
  data from multiple wells, then any discriminator curve must be available in all wells."
- **One blank input volume is allowed**: "IP will add up all the volumes from the other curves and
  assume the remainder belongs to the blank input entry. It is only possible to leave one input curve
  blank." Input volume units must be declared as `Dec (V/V)` or `Percent %`.
- Prerequisite: XRD dry-weight % must first be converted to **wet volume %** in the Preprocessor.
- `Copy Parameters` pushes selected end-points back to the Model grid, per-cell.
- `Reset` re-syncs to the Model grid but **clears all input volume curve names**.

**Stated numeric defaults for calibration: none**, beyond the two illustrative values already in §2.2
(fresh water 1.0 gm/cc as a fixed end-point; 0.1 mD as a discriminator example).

Separately, the interactive **crossplot calibration** path (`mineral_solver.htm`): end-points render as
draggable red circles labelled with the mineral name; dragging re-runs the model live (`Interactive Run`
can disable this on slow machines); max 8 end-points per crossplot; points shared across multiple open
crossplots move in sync; undo/redo arrows appear once points have moved. Auto (blue) and true-HC-density
(green) cells are excluded from the pickable list.

---

## 6. Interoperability & market intelligence (Tier A)

- **Schlumberger Elan import.** `Load Model` → File Type `Elan Model` reads `.elp` parameter files.
  Stated limits: loads minerals, fluids, input equations and end-points only — "IP will **not** load any
  model constraints or model mixing rules. These are not directly compatible with IP." Elan usually
  carries un-invaded fluids that IP does not need; Sw is calculated differently; the wet/dry clay model
  "will also need sorting out manually." Name mapping is user-editable in **`ElanToIPMapping.par`**.
- **Everything customisable is a plain ASCII file in the install directory**, not a database:
  `MINDEF.PAR`, `MINEQDEF.PAR`, `Neu_Parm_Files.neu` + per-tool `xxx.neu`, `ElanToIPMapping.par`,
  `UnitConversion.par`, `MonteCarloDefaults.par`, `Overlay_Files.ovl`. Models save as individual files
  under `..\Mineral Solver Models` for reuse across wells and projects. This is a design pattern
  SandiBumi/SandiMin can adopt wholesale — it is a *convention*, not IP expression.
- Parameter-set portability caveat, stated explicitly: models must be moved between wells with
  **Load/Save Parameter Sets**, *not* Save Model / Load Model, because only the former carries the
  Mixings.
- Deleted model numbers are **never reused** and deleting a model deletes its curve set.
- `Print Parameters to File` writes "all the models, parameters and mixing's used in the analysis" to a
  `.txt` named after the Parameter Set — a ready-made audit-trail pattern.

---

## 7. The Sand-Silt-Clay / Malay model — in full

### 7.1 Identity, provenance and tier

- Menu: **Interpretation → Sand/Silt Malay Model**. Introduced as an **IP 4.3 New Feature**
  (`intro_whats_new_in_ip.htm`, entry sits between the `IP 4.3 New Features` and `IP 4.3 Enhancements`
  headers).
- **Primary citation given, verbatim:** "based loosely on the paper *'Log Interpretation in the Malay
  Basin by K. Kuttan et al, 21st SPWLA symposium'*." (`sand_silt_malay_model.htm`, repeated in
  `interpretation.htm`.) No page numbers, year, or DOI are given. IP's own words are "based **loosely**".

**Tier ruling: Tier B framework wrapped around a Tier C calibration core.**

- **Tier B** — the three-component sand/silt/clay N-D framework, the matrix-line projection, the
  hydrocarbon-correction loop, the grain-density/PhiT chain and the four total-Sw equations are all
  attributable to published science (Kuttan et al. plus the standard Archie/Dual-Water/Juhász/W-S set).
  Method name + citation recorded; free to reference.
- **Tier C** — the **Lithology Conversion Chart** (`_intclip0061.png`) that actually converts the
  projected matrix-line position into `Fsn` / `Fsi` / `Fdc`. The manual states plainly: *"The position of
  A and B points along Y axis, and the shape of those **five curves** are determined based on the
  characteristics of **core data acquired in Malay Basin**."* That core dataset is unpublished, the chart
  is delivered only as a raster, and the five curve shapes are the model's entire discriminating power.
  **Named and evidenced only. Not reconstructed, not digitised, not approximated.**
- Also vendor-derived and flagged: the **three-way blended Phie equation** (§7.6), which the manual
  derives itself and does not attribute to the paper.

**Practical consequence for SandiBumi:** the published half is reproducible; the half that makes it work
in the Malay Basin is not. A Mahakam-Delta equivalent would have to be calibrated from Jauhar's own core
data, and that calibration is exactly the piece IP treats as proprietary.

### 7.2 Stated design intent and assumptions

Verbatim intent: *"designed for very fine grained sediments with fresh to brackish formation water. It
was built to overcome interpretation problems of standard shaley sand log analysis of the type used in
the 1980's, where an overestimate of clay volume leads to too pessimistic porosities."*

Assumptions as stated:
1. The model is built on the **Neutron-Density crossplot**, with a **Quartz, Silt and Dry clay** point
   defined (plus a fluid point).
2. **"The model assumes little to no clay is found at the silt point"** — relaxed by the
   `Clay at Silt Point` parameter.
3. The three matrix fractions produce a **matrix density**, which with the density log gives porosity —
   i.e. **PhiT comes from the density log alone**, not from a N-D combination.
4. Hydrocarbon corrections are applied to **both** neutron and density logs, **iteratively**.
5. Four **Total** Sw equations are provided (all total-porosity based; there is no effective-porosity Sw
   option).
6. Coal is a **flag**, not a volume solved with the others (`Vcoal` is 0.0 or 1.0).
7. `Vshale` is defined as **`Vcl + Vsilt`** — silt is counted as shale, but *not* as clay. This is the
   central move of the model: it decouples the shale indicator from the clay volume that destroys
   porosity.

### 7.3 The three-component framework, step by step (as printed)

| Step | What happens |
|---|---|
| 1 | **Coal logic** — if `Density Log < Den Coal` **AND** `Neutron Log > Neu Coal`: `PhiT, Phie, Vcl, Vshale, Vquartz, Vsilt, Vdclay = 0`; `SwT, SxoT, Sw, Sxo = 1.0`; `Vcoal = 1.0` |
| 2 | **VshGr** from `Gr`, `GrClean`, `GrShale` — equation `_intclip0056.png`, rasterized |
| 3 | **Apparent HC density and neutron HI** from `Den Hyd` — `_intclip0057/0058.png`, rasterized |
| 4 | **Correct density and neutron for HC** — printed in text (see §7.4) |
| 5 | **VshND** from corrected logs and the wet-clay line — `_intclip0059.png`, rasterized. Named inputs: `Denmat`, `Denfl`, `DenWetClay`, `Dencorr`, `Neumat`, `Neufl`, `NeuWetClay` (itself *calculated from* neutron dry clay), `Neucorr` |
| 6 | **Sxo from N-D separation** (optional) — `SxoT` stepped in increments of **0.01** until `VshND == VshGr`, re-correcting the logs at every step; clamped so `SxoT` never crosses `SwT` (≥ in WBM, ≤ in OBM) |
| 7 | **Bad hole logic** (optional) — assumes `VshND < VshGr` means the density reads too low; raises the density until `VshND == VshGr` or the corrected point lands on the wet-clay line. Manual warns: "Use with care", "can have drastic effects" |
| 8 | **Lithology fractions** — project the corrected point onto the matrix line, then read `Fsn`/`Fsi`/`Fdc` off the **Lithology Conversion Chart** (Tier C). Printed worked example: `Fsn = 0.45`, `Fsi = 0.50`, `Fdc = 0.05`. "If the dry silt point does not fall on the matrix line then it is projected onto the line." **`B point value = Clay at Silt Point` parameter; `A point value = 1.0 - B`** |
| 9 | **Grain density and PhiT** — printed in text (§7.5) |
| 10 | **Matrix and shale volumes** — printed in text (§7.5) |
| 11 | **Effective porosity** — printed in text (§7.6) |
| 12 | **Phie limits and the bound-water sanity clamp** — printed in text (§7.6) |
| 13 | **Total water saturation** — four options, all rasterized |
| 14 | **Flushed-zone SwT (SxoT)** — three routes |
| 15 | **Iteration loop** — see §7.8 |
| 16 | **Final calculations** — printed in text (§7.9) |
| 17 | **Logic flag** — see §7.10 |

### 7.4 Hydrocarbon correction (printed in text — recoverable)

```
DenCorr = Rhob + Phie x (1.0 - Sxo) x (Rhofl - DenHydApp)

NeuCorr = PhiNeu + exfact + Phie x (1.0 - Sxo) * (1.0 - NeuHydHI)

exfact  = √(Rhoma/2.65) x (2.0 x SwH x Phie x Phie + 0.04 x Phie) x (1.0 - SwH)

SwH     = Sxo + (1-Sxo)* NeuHydHI)
```
(as printed, including the unbalanced parenthesis in the `SwH` line). `Rhob` = input density curve,
`PhiNeu` = input neutron curve, `Rhofl` = input density fluid parameter, `Rhoma` = input density matrix
parameter. Note the hard-wired **2.65** inside the excavation term.

### 7.5 Grain density, porosity and volumes (printed in text — recoverable)

```
Rhoma    = Fsn x Denmat + Fsi x Densilt + Fdc x Dendcl
PhiT     = (Rhoma - DenCorr) / (Rhoma - Denfl)

Vsand    = Fsn x (1 - PhiT)        output curve "Vol Sand"
Vsilt    = Fsi x (1 - PhiT)        output curve "Vol Silt"
Vdcl     = Fdc x (1 - PhiT)        output curve "Vol dry clay"

PhiTclay = (Dendcl - Denwcl) / (Dendcl - Denfl)
Vcl      = Vdcl / (1 - PhiTclay)   output curve "Vol wet clay"
Vshale   = Vcl + Vsilt             output curve "Vol shale"
```

### 7.6 Effective porosity — IP's own three-way blend (printed in text)

The manual derives this itself and does **not** attribute it to the Kuttan paper:

> "The implication of this equation in very shaly reservoirs (Vcl > 70%, and Vsand = 0), in some cases
> the Phie value is still significantly big. To adjust for the calculation of high Phie in very shaly
> reservoirs, another equation is used to make sure Phie turns out to 0 when Vcl = 1."

```
(1)  Phie = PhiT - Vcl x PhiTclay
(2)  Phie = PhiT - Vcl x PhiT
(3)  Phie = (1 - PhiT) x (PhiT - Vcl x PhiTclay) + Vcl x (PhiT - Vcl x PhiT)
```
Equation (3) is the one used. Stated behaviour: "When Vcl is closed to 0, then Phie turns out to the
first equation. When Vcl is closed to 1, then Phie turns out to the second equation."

> **Analytical note, offered as a caution not a correction:** the stated blend weights are `(1 - PhiT)`
> and `Vcl`, which do not sum to 1 for general `PhiT`, `Vcl`. Whether the manual's rendering is exact or
> has lost a term is **not resolvable from the text**. Verify against IP behaviour before adopting.

**Phie limits, in order:**
```
Phie <= MaxPhie x (1.0 - Vcl)
```
"The MaxPhie parameter should be set to the maximum porosity seen in a clean sand. To remove the effect
of this limit make MaxPhie a high number (**0.6**)." — a *suggested disabling value*, not a stated default.

If `Vshale > VShale Cutoff` then `Phie = 0`. "Setting Vshale Cutoff to **1.0** will completely remove
this cutoff", and the interactive track states "By default this has a value of **1.0**".

### 7.7 Bound-water treatment (printed in text — the part most relevant to Mahakam)

```
Vbw = PhiT - Phie                          volume bound water

if  Vbw > Vcl x PhiTclay x 1.5
      Vbw  = Vcl x PhiTclay x 1.5
      PhiT = Phie + Vbw
```
"If PhiT has been changed then all the matrix and shale volumes are recalculated." Stated rationale:
with the Phie limits active "it is possible to arrive at situations where the bound water can get
completely out of line with the effective and total porosity." Caution as printed: "The user should
check that these limits are only applied in non-reservoir sections."

So the bound-water model is: **clay-bound water only**, derived entirely from the dry/wet-clay density
contrast (`PhiTclay`), with a hard **1.5×** ceiling. There is no capillary-bound-water term and no
silt-bound water — silt is treated as clay-free matrix unless `Clay at Silt Point` says otherwise.
`Vbw` is an output curve. Interactive plot shows bound water in brown in the porosity track.

Coupling flagged by the manual: "there is a strong correlation between the bound water volume
(PhiT-Phie) and the 'Bn' factor. If you change the bound water by changing the dry or wet clay density
then the Qvn/Cwapp will change and the 'Bn' should be adjusted."

### 7.8 Water saturation, flushed zone, and convergence

Four **Total** Sw equations, selected per zone: **`Waxman Smt`**, **`Juhasz W&S`**, **`Dual Water`**,
**`Archie PhiT`**. All four formulas are rasterized (`_intclip0062`–`0065`), as are `Qv = a/PhiT + b`
(`_intclip0066`), `B` (`embim178.gif`), and effective Sw from total (`_intclip0067`). Symbol glossary is
printed: `Qvn`, `m`, `n`, `a`, `SwT`, `Rw`, `Rwb`, `Rt`, `Bn`, `B`, and **`T = Formation temperature in
degC`** (note: degC here, whereas the Mineral Solver glossary also says centigrade — but the SSC `Fluids`
tab `Temp Units` may be `deg F` or `deg C`, and applies only to the `Rw/Rmf/Rwb/Rmbf Temp` parameters,
*not* to the input temperature curve, whose units are read from the curve's own units field).

`SxoT` routes (`Sxo Logic`): **`ND Separation`** (step 6 above) | **`Input Curve`** (same Sw equation
with Rxo, Rmf, Rfmb) | **`Invasion Factor`**. WBM invasion form is `_intclip0068.png` (rasterized); OBM
is printed: `Sxo = Invasion Factor input parameter`, then `SxoT = Sxo x (1 - Swb) + Swb`.
`SxoT` is limited to be greater than `SwT` (WBM) or less than `SwT` (OBM).

**Convergence (printed):**
```
PhiT difference < 0.0001
SxoT difference < 0.001
```

### 7.9 Final calculations (printed in text — recoverable)

```
BVW     = Phie x Sw          Bulk volume water
BVW     = Phie x Sxo         Bulk volume flushed zone water   [as printed - the manual reuses the name "BVW"
                                                               for both; the output curve is BVWSXO]
Rwapp   = PhiTm x Rt / a     Rw apparent          [PhiTm = PhiT**m; superscript lost in the HTML]
CwApp   = 1.0 / Rwapp        Cw apparent
QvNorm  = VCL x PhiTClay / PhiT      Qv normalized
QvApp   = a / (B x Rt x PhiTm) - 1.0 / (B x Rw)   Qv apparent
RecPhiT = 1.0 / PhiT         reciprocal PhiT
```
`QvNorm`/`CwApp` drive the Juhász `B Factor Juhasz` pick; `QvApp`/`RecPhiT` drive the W&S `Qv a const` /
`Qv b const` pick. Crossplot procedure as stated: for W&S, "plot the data over a wet shaley interval and,
if such a relationship exists, interactively set the line. If no relationship exists then the user must
enter Qv into the model as a fixed value or curve after establishing Qv at each depth level using a
different methodology." For Juhász, "The left side of the interactive line is anchored at the Rw value.
The right side of the line should be interactive moved to follow the trend in the data of increasing Qv.
The normalized B factor is calculated from the slope of the line."

### 7.10 Logic flag (`LogicFlag` output curve) — complete table as printed

| Value | Meaning |
|---|---|
| 0 | Logic run with no problems. |
| 1 | ND SxoT hydrocarbon correction did not match VshGR and VshND. "This does not necessarily mean that there is a problem, could be that this is a water interval hence no hydrocarbon corrections can be applied." |
| 2 | Main Iterative hydrocarbon loop reached limit max 40. |
| 3 | Sxo Correction loop reached max 100. |
| 4 | Sw equation iterative loop did not converge after 10 iterations. |
| 5 | Bad hole correction loop reached max 100. |
| 6 | Bad hole logic used. |
| 7 | PhieMax logic used. |
| 8 | Vshale cutoff logic used. |
| 9 | Input found NullValues so no output. |

(The interactive plot describes value 1 as "A Yellow flag".)

### 7.11 Complete parameter inventory (10 tabs) with stated defaults

| Tab | Parameter | Stated default |
|---|---|---|
| Zones | Zone Name / Top / Bottom / Color / Lock Zone | — |
| VShale | `Gr Clean` | not stated in manual |
| VShale | `Gr Shale` | not stated in manual |
| Logic Options | `Mud Type` (`WBM` / `OBM`) | not stated in manual |
| Logic Options | `Sw Equation` (`Waxman Smt` / `Juhasz W&S` / `Dual Water` / `Archie PhiT`) | not stated in manual |
| Logic Options | `Sxo Logic` (`ND Separation` / `Input Curve` / `Invasion Factor`) | not stated in manual |
| Logic Options | `Bad Hole Logic` (on/off) | not stated in manual |
| Logic Options | `Max Phie` | not stated in manual (0.6 given as the *disabling* value) |
| Logic Options | `Vshale Cutoff` | **1.0** ("By default this has a value of 1.0") |
| Matrix-Clay | `Den Matrix` (sand) | not stated in manual |
| Matrix-Clay | `Neu Matrix` (sand) | not stated in manual |
| Matrix-Clay | `Den Dry Clay` | not stated in manual |
| Matrix-Clay | `Neu Dry Clay` | not stated in manual |
| Matrix-Clay | `Den Silt` | not stated in manual |
| Matrix-Clay | `Neu Silt` | not stated in manual |
| Matrix-Clay | `Clay at Silt Point` | not stated in manual ("If set to zero then all the silt is clay free") |
| Matrix-Clay | `Den Wet Clay` | not stated in manual (used to derive `NeuWetClay` and `VshND`) |
| Fluids | `Temp Units` (`deg F` / `deg C`) | not stated in manual |
| Fluids | `Rw`, `Rw Temp`, `Rmf`, `Rmf Temp` | not stated in manual |
| Fluids | `Den Water`, `Neu Water` | not stated in manual |
| Fluids | `Den Hyd` | not stated in manual |
| Water Saturation | `a factor`, `m exponent`, `n exponent` | not stated in manual |
| Water Saturation | `Invasion factor` | not stated in manual |
| Waxman Smt eq. | `B W&S Source` (`Curve or Value` / `Formula`) | not stated in manual |
| Waxman Smt eq. | `B factor W&S` | not stated in manual |
| Waxman Smt eq. | `Qv W&S Source` (`Curve or Value` / `Equation`) | not stated in manual |
| Waxman Smt eq. | `Qv W&S`, `Qv 'a' const`, `Qv 'b' const` | not stated in manual |
| Juhasz eq. | `B Fact Juhasz` | not stated in manual (**cf. Mineral Solver: 1.0 meq/ml**) |
| Dual Water eq. | `Rwb`, `Rwb Temp`, `Rmfb`, `Rmfb Temp` | not stated in manual |
| Coal | `Coal Logic` (on/off) | not stated in manual |
| Coal | `Den Coal`, `Neu Coal` | not stated in manual |

**This is the single biggest gap in scope D**: the SSC model's entire endpoint set — the sand, silt,
dry-clay and wet-clay N-D points that *define* the model — has **no published default in the manual**.
Recoverable only from an IP parameter-set export or the shipped default `.par`, not from the help.

**Complete output-curve inventory** (26): `PhiT`, `Phie`, `BVW`, `BVWSXO`, `Vbw`, `Vcl`, `Vshale`,
`Vsh_Gr`, `Vsh_ND`, `Vdcl`, `Vsilt`, `Vquartz`, `Vcoal`, `Rhoma`, `Rwapp`, `Rmfapp`, `SwT`, `SwTu`,
`Sw`, `SxoT`, `SxoTu`, `Sxo`, `Rhob_corr`, `Nphi_Corr`, `QvNorm`, `CwApp`, `QvApp`, `RecPhiT`,
`LogicFlag`. Input curves: Density (gm/cc), Neutron (v/v), Rt (ohm.m), Rxo (ohm.m, optional), Gamma Ray,
Temperature (units must start with `F` or `C`).

Reporting discipline stated by the vendor and worth copying: "The clipped curves SwT and SxoT are not
displayed in the interactive plot but **should be used for final plots and summation reports**"; the
unlimited `SwTu`/`SxoTu` exist so the >100 % overshoot is visible during parameterisation only.

---

## 8. Tier-C flags

| ID | Item | Evidence | Status |
|---|---|---|---|
| **T1** | **Malay-Basin Lithology Conversion Chart** — the five curves and the A/B axis positions that convert matrix-line position to `Fsn`/`Fsi`/`Fdc` | `sand_silt_malay_model.htm`, image `_intclip0061.png`: "The position of A and B points along Y axis, and the shape of those five curves are determined based on the characteristics of core data acquired in Malay Basin." | **NEW flag.** Named only. Not reconstructed, not digitised. Proprietary calibration on unpublished core |
| **T2** | **DNOPT Dense Nonlinear OPTimizer** (Stanford University) — the commercial non-linear optimiser IP embeds | `minsolveeqandmeth.htm`, `mineral_solver.htm`: "The Non-Linear Solver used in this module is the DNOPT Dense Nonlinear OPTimizer from Stanford University." | **NEW flag** — third-party *licensed* component, not IP IP. Named for procurement awareness. SandiBumi cannot inherit this; an independent NLP (e.g. an open-source SQP/interior-point solver) would be needed |
| **T3** | **`MINDEF.PAR` / `MINEQDEF.PAR` shipped default content** | `mineral_solver.htm`, `default_settings.htm`, `tools.htm` | **NEW flag.** The *mechanism* (two cooperating ASCII default files, user-extensible) is Tier A and adoptable. The *shipped numeric contents* are vendor data files inside a licensed install — treat any values read from them as vendor reference data requiring its own tier ruling before adoption |
| **T4** | **Elan `.elp` import + `ElanToIPMapping.par`** | `mineral_solver.htm` | Competitor-format interop. Tier A as market intelligence (the *existence* and the *stated incompatibilities* are the valuable part). Do not replicate Schlumberger's `.elp` schema from IP's mapping file |
| **T5** | **Wyllie-Cp ↔ Hunt-Raymer bridging relation** — a 4-coefficient empirical fit IP derived to make Hunt-Raymer solvable in the linear solver | `minsolveeqandmeth.htm`: "a relationship has been determined between the Wyllie compaction factor and the Hunt-Raymer equations above" | **NEW flag, judgment call — reversible.** The coefficients ARE printed in plain text on that page, but they are a *vendor-fitted* relation with no published source, so they were **deliberately not transcribed** into this report or the JSON. If the caller rules this Tier A, the values are one line away at `minsolveeqandmeth.htm` in the Sonic section |
| **T6** | **IP's three-way blended Phie equation** (SSC step 11) | `sand_silt_malay_model.htm` §11 | **NEW flag, low severity.** Vendor-derived, printed in full text, not attributed to Kuttan et al. Transcribed here because it is plainly stated and its provenance is explicit — but it must NOT be cited as "the Malay Basin published method" |

**Already-registered Tier C items** (SonicSaturation / Omovie US 12,242,011 B2, Domain Transfer
Analysis, Experienced Eye, entropy-based borehole-image speed correction, shipped NN weights) — **none
appear anywhere in scope D**. The Mineral Solver and SSC chapters are clean of them.

---

## 9. Gaps and open questions

1. **No mineral end-point table exists in the IP 2018 help.** All defaults live in `MINDEF.PAR` /
   `MINEQDEF.PAR`. Anything claiming to be "IP 2018's kaolinite density" must come from those files, not
   from this manual.
2. **Cross-check for the caller (question, not a finding): does IP 2018's `MINDEF.PAR` share the IP 2025
   `MINDEF.PAR` convention?** Column order, mineral roster, units, wet-vs-dry-clay convention, and the
   values themselves. Answerable only by diffing `C:\Program Files\IP2018\MINDEF.PAR` (read-only)
   against the already-ingested IP 2025 file. **Not guessed here.**
3. **99 formulas in `minsolveeqandmeth.htm` are rasterized** (`embim213`–`embim311`), including the
   objective function itself, all the neutron/density/sonic/EPT/U auto-endpoint relations, the Qv-from-CEC
   conversion, and all 11 Sw equations. The ASCII Sw equations on `mineral_solver.htm` recover 11 of them;
   the rest need OCR or manual reading of the GIFs.
4. **13 SSC equations are rasterized as `_intclip*.png`** and are invisible to the current extractor's
   `equation_images` counter. Re-extraction should treat `_intclip*` as candidate equations, not
   screenshots, on this page.
5. **The SSC model has no published parameter defaults at all** — sand, silt, dry-clay, wet-clay N-D
   points, `Clay at Silt Point`, `Max Phie`, `Gr Clean`/`Gr Shale`, Archie `a`/`m`/`n`. Directly blocking
   for any Mahakam-Delta adaptation.
6. **CEC units are never stated.** Qv/Qvn are defined per unit *total pore volume*; `B fact Juhasz`
   default is printed as `1.0 meq/ml`; but the clay CEC that feeds the `Qv` output equation has no unit
   given, and the conversion is rasterized.
7. **Sigma is not documented as an equation type.** Cased-hole operation is described procedurally only —
   no Sigma end-point convention, no Sigma unit, no auto-endpoint rule.
8. **Suspected typo in the printed `Sch_CNL.neu` table** at `phi = .20`, Dolomite/50 kppm: `-.1960`.
   Transcribed verbatim; verify against the shipped file before use.
9. **The Conventional / Modified apparent-hydrocarbon-density equations are corrupted in the HTML**
   (superscripts lost). Transcribed as printed with an explicit do-not-implement warning.
10. **Invasion-factor default is printed as both 0.5 and 2.0** on different pages, for what appear to be
    two different uses of the same parameter name. Both recorded; not reconciled.
11. **PHIFLAG values 0, 1, 3, 13 are unlisted** in the manual's table. 0 is stated elsewhere to mean
    normal execution; 1, 3, 13 are unknown.
12. **The SSC blended-Phie weights do not obviously sum to 1** (§7.6). Cannot be resolved from text.
13. `plot_the_mineral_solver_result.htm` is entirely UI/plot description — no equations, no numeric
    defaults. Its value is the QC-workflow structure (error track, confidence bands, reconstruction
    tracks, Pickett/Hingle/Cwapp-Qvn/1-over-PhiT crossplot suite), which is Tier A and adoptable.
