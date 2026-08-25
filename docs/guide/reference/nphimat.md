<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Neutron Matrix Conversion

Module id `nphimat` · category **Prep** · [reference index](README.md)

Converts a neutron porosity log recorded in one matrix convention into all three (NPHI_LS / NPHI_SS / NPHI_DOL), using the chartbook porosity-equivalence curves: Por-5 for the CNL thermal tools (NPHI ratio method; TNPH environmentally corrected, with 0 and 250,000 ppm salinity variants) and Por-4 for the epithermal tools — APLC and FPLC (APS) plus the legacy sidewall SNP. Limestone units ARE apparent limestone porosity — the chart's x-axis, on which calcite is the identity — so an SS or DOL input is first inverted back to that axis, then read out along each matrix curve; the input convention passes through unchanged. Feed the output whose matrix matches your RHO_MA into density-neutron work (NPHI_SS with RHO_MA 2.65) — that removes the limestone-vs-sandstone convention offset before a sourced XOVER_MIN is applied. SALINITY picks the TNPH curve pair only; the other tools have a single chart curve. SALINITY = INTERPOLATE (SB-POR-025) evaluates the fresh and the 250-kppm chart COMPLETELY and interpolates the finished answers linearly on the declared RHO_FL, per Geolog's phi_dn two-call structure — TNPH only, since only that family carries both digitized curves. Apply environmental corrections (nphi_env_corr) before converting — the charts assume corrected logs. The limestone axis and dolomite curves are digitized to about -0.02..0.40; the sandstone curves leave the chart top at 40 pu true porosity (~0.32-0.36 apparent limestone), and beyond the data every curve is extended linearly on its end segment. Note NPHI_LS is also a common raw-log mnemonic: after a run, by-name lookups resolve the computed version first (the raw log keeps its provenance in the Curve Catalog).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| NPHI | Neutron porosity log (v/v, in MATRIX_IN units) | `NPHI` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_FL *(g/cc)*

Borehole fluid density resolving the fresh/salt two-call interpolation (INTERPOLATE only)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.8 to 1.3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.
- **Checked before the run:**
  - `rho_fl.required_when_interpolate` — RHO_FL is required when SALINITY = INTERPOLATE. Applies when: [object Object].
    Source: Geolog V14 phi_dn.lls SCH_TNPH branch: phix = ((RHO_FL-1000)*(phit_2-phit_1)/190)+phit_1 - the input is the well's own fluid density and Geolog ships no default; docs/PRD_v2/11_porosity.md SB-POR-025 + F13

## Options

### TOOL

Neutron measurement the log comes from (TNPH/NPHI: CNL thermal, Por-5; APLC/FPLC: APS epithermal and SNP: sidewall neutron, Por-4)

- **Choices:**
  - `TNPH`
  - `NPHI`
  - `APLC`
  - `FPLC`
  - `SNP`
- **Default:** `TNPH`

### SALINITY

Formation salinity (TNPH curves only; SALT_250K = 250,000 ppm; INTERPOLATE = evaluate fresh and salt and interpolate on RHO_FL)

- **Choices:**
  - `FRESH`
  - `SALT_250K`
  - `INTERPOLATE`
- **Default:** `FRESH`

### MATRIX_IN

Matrix convention the input log is recorded in

- **Choices:**
  - `LS`
  - `SS`
  - `DOL`
- **Default:** `LS`

## Output curves

| Name | Description |
|---|---|
| NPHI_LS | Neutron porosity, limestone units (apparent limestone) |
| NPHI_SS | Neutron porosity, quartz sandstone units |
| NPHI_DOL | Neutron porosity, dolomite units |
