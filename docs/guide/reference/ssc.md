<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SSC — Sand-Silt-Clay (Kuttan)

Module id `ssc` · category **Porosity** · [reference index](README.md)

Sand-Silt-Clay model on the N-D crossplot (Kuttan Malay Basin, SandiBumi edit). Data points are projected from the fluid point onto the dry rock line (matrix→dry clay); sand/silt/clay fractions come from the projection position, matrix density from the fraction mix, PHIT from density. Bound water is split into clay-bound (CBW) and capillary-bound in silt/shale (CWSH): PHIE = PHIT − VWCL·PHIT_CL, PHIFF = PHIT − CBW − CWSH, SWIRR_T = BW/PHIT. GR-equivalent volumes rescale the SSC volumes to honour VSHGR. Study-specific crossplot endpoints ship absent and must be supplied from the active interpretation.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| GR | Gamma ray (normalized) | `GRN` | yes | — |
| RHOB | Bulk density (corrected) | `RHOB` | yes | — |
| NPHI | Neutron porosity (sandstone units) | `NPHI` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### GR_MA *(gapi)*

Gamma ray matrix (clean)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 100 gapi

### GR_SH *(gapi)*

Gamma ray clay

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 gapi

### RHOB_MA *(g/cc)*

Density matrix

- **Default:** 2.65 — source: IP/Techlog/SandiMin sandstone matrix endpoint 2.65 g/cm3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 1 to 4 g/cc

### NPHI_MA *(v/v)*

Neutron matrix

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.1 to 1.2 v/v

### RHOB_FL *(g/cc)*

Density fluid

- **Default:** 1 — source: IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.5 to 4 g/cc

### NPHI_FL *(v/v)*

Neutron fluid

- **Default:** 1 — source: Geolog V14 vsh_dn.info and Techlog VSH neutron-density NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5
- **Accepted range:** -0.1 to 1.2 v/v

### RHOB_WCL *(g/cc)*

Bulk density wet clay

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4 g/cc

### NPHI_WCL *(v/v)*

Neutron porosity wet clay

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.1 to 1.2 v/v

### RHOB_DCL *(g/cc)*

Bulk density dry clay

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4 g/cc

### NPHI_WSI *(v/v)*

Neutron porosity wet silt

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.1 to 1.2 v/v

### DCLF_SI *(v/v)*

Dry clay fraction at dry silt

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### PHIT_CL *(v/v)*

Total porosity of clay

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.8 v/v

### SWIRR_MIN *(v/v)*

Minimum total irreducible Sw

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### PHIT_TIGHT *(v/v)*

Total porosity below which all non-clay-bound porosity is capillary-held

- **Default:** 0.05 — source: SandiBumi's own SSC conditioning rule, not the Loglan's: added to keep CWSH positive and reliable, since CWSH always exists even where small; KEPT and parameterised under DEC-093 (2026-08-22). 0.05 is the value the port has run since it was written; it is a parameter now so a tight carbonate stringer and a shaly sand need not share it
- **Accepted range:** 0 to 0.5 v/v

### GAS_C

Gas-conditioning weight (0 = density only, 1 = even, 2 = neutron only)

- **Default:** 1.6 — source: sspw.lls (2025-02-28) gas branch writes the even split, PHIT = ((phiD^2 + NPHI^2)/2)^0.5, i.e. c = 1 - and that is what SSC ran until DEC-088 OVERRODE it, ruling 1.6 here too and extending DEC-086's field observation that the even split still reads optimistic. The source is unchanged; the shipped default departs from it deliberately
- **Accepted range:** 0 to 2

## Options

### OPT_VSHGR

VSH from gamma ray method

- **Choices:**
  - `LINEAR`
  - `STIEBER1`
  - `STIEBER2`
  - `STIEBER3`
  - `LARINOV1`
  - `LARINOV2`
  - `LARINOV3`
  - `CLAVIER`
- **Default:** `LINEAR`

## Output curves

| Name | Description |
|---|---|
| VSAND | Dry sand volume (bulk) |
| VSILT | Dry silt volume (bulk) |
| VDCL | Dry clay volume (bulk) |
| VWCL | Wet clay volume |
| VSH_SSC | Vshale equivalent (VWCL + VSILT) |
| VSHGR | VSH from gamma ray |
| VSHND | VSH from density-neutron |
| PHIT_SSC | Total porosity |
| PHIE_SSC | Effective porosity (PHIT − CBW) |
| PHIFF_SSC | Free fluid porosity (PHIT − CBW − CWSH) |
| CBW | Clay-bound water |
| CWSH | Capillary-bound water in silt/shale |
| BW | Total bound water |
| SWIRR_T | Total irreducible water saturation |
| SWIRR_EFF | Effective irreducible water saturation |
| VSAND_GR | Sand volume, GR-equivalent |
| VSILT_GR | Silt volume, GR-equivalent |
| VDCL_GR | Dry clay volume, GR-equivalent |
| CBW_GR | Clay-bound water, GR-equivalent |
| CWSH_GR | Capillary water, GR-equivalent |
| PHIFF_GR | Free fluid, GR-equivalent |
| PHIE_GR | Effective porosity, GR-equivalent |
| PHIT_GR | Total porosity, GR-equivalent |
