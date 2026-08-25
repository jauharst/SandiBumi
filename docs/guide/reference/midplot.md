<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Apparent Matrix (MID plot: UMAA / RHOMAA)

Module id `midplot` · category **Lithology** · [reference index](README.md)

Apparent matrix density RHOMAA and apparent matrix volumetric photoelectric factor UMAA — the two axes of the Schlumberger Lith-6 MID plot (crossplot X = UMAA, Y = RHOMAA, then switch on the 'Lith-6 Umaa-Rhomaa MID plot' chart overlay). U = PEF * rho_e with rho_e = (RHOB + 0.1883)/1.0704; the fluid is then removed from both: RHOMAA = (RHOB - phi*RHO_FL)/(1 - phi), UMAA = (U - phi*U_FL)/(1 - phi). PHI here is the APPARENT porosity, the one judgement call in the method. OPT_PHIA = CHART (default) reads it off the density-neutron crossplot the way you would by hand on Por-11: it solves for the porosity at which the density and the neutron imply the SAME matrix, interpolating across the chartbook's sandstone, limestone and dolomite curves (pick the curve family with TOOL/SALINITY, exactly as in Neutron Matrix Conversion). Build a rock's two tool readings, feed them back, and this returns that rock: porosity to 1e-3 and RHOMAA onto its own matrix line, for sandstone, limestone and dolomite alike. Rocks denser than every matrix line (anhydrite, pyrite) clamp to the end of the search and stay heavy rather than dropping out, and gas pushes points low-left just as it does on the printed chart. XPLOT is the analytic average commercial suites take — kept for comparison, not for accuracy: it drags points toward the assumed RHO_MA_A, leaving dolomite about 0.06 g/cc light and 0.34 b/cm3 left of its chart point. NEUTRON uses the neutron alone. LOG takes a porosity curve you already trust. NPHI must be APPARENT LIMESTONE for CHART and NEUTRON — run Neutron Matrix Conversion first if the log is recorded in sandstone or dolomite units. Density-only apparent porosity is deliberately NOT offered: it is algebraically degenerate (it returns RHO_MA_A for every sample). Barite mud makes PEF, and therefore UMAA, unreadable — mask those intervals.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Bulk density log, as logged | `RHOB` | yes | — |
| NPHI | Neutron porosity log, apparent limestone | `NPHI` | yes | — |
| PEF | Photoelectric factor | `PEF` | yes | — |
| PHI_IN | Apparent porosity curve, used only when OPT_PHIA = LOG | `PHIT` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_MA_SS *(g/cc)*

Sandstone matrix line, CHART lookup

- **Default:** 2.65 — source: IP/Techlog/SandiMin sandstone matrix endpoint 2.65 g/cm3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 2 to 3.2 g/cc

### RHO_MA_LS *(g/cc)*

Limestone matrix line, CHART lookup

- **Default:** 2.71 — source: IP/Techlog/SandiMin limestone matrix endpoint 2.71 g/cm3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 2 to 3.2 g/cc

### RHO_MA_DOL *(g/cc)*

Dolomite matrix line, CHART lookup

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc

### RHO_MA_A *(g/cc)*

Apparent matrix density for the density leg (limestone basis)

- **Default:** 2.71 — source: IP/Techlog/SandiMin limestone matrix endpoint 2.71 g/cm3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 2 to 3.2 g/cc

### RHO_FL *(g/cc)*

Fluid density

- **Default:** 1 — source: IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.5 to 1.5 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.

### U_FL *(b/cm3)*

Fluid volumetric photoelectric factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 3 b/cm3

### PHIA_MAX *(v/v)*

Reject samples whose apparent porosity exceeds this

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 0.9 v/v

## Options

### OPT_PHIA

Apparent porosity basis (see the method note — this choice moves the points)

- **Choices:**
  - `CHART`
  - `XPLOT`
  - `NEUTRON`
  - `LOG`
- **Default:** `CHART`

### TOOL

Neutron measurement the log comes from, for the CHART lookup (same chart families as Neutron Matrix Conversion)

- **Choices:**
  - `TNPH`
  - `NPHI`
  - `APLC`
  - `FPLC`
  - `SNP`
- **Default:** `TNPH`

### SALINITY

Formation salinity for the CHART lookup (TNPH curves only; SALT_250K = 250,000 ppm)

- **Choices:**
  - `FRESH`
  - `SALT_250K`
- **Default:** `FRESH`

## Output curves

| Name | Description |
|---|---|
| U | Volumetric photoelectric factor |
| RHOMAA | Apparent matrix density |
| UMAA | Apparent matrix volumetric photoelectric factor |
| PHIA | Apparent porosity actually used |
