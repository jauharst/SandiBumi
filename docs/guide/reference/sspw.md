<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SSPW — Sandstone Petrophysical Workflow

Module id `sspw` · category **Porosity** · [reference index](README.md)

Three-component sandstone workflow (quartz + shale + water). PHIT from density with a VSH-mixed dry matrix (RHOB_MAT / RHOB_DSH); shale total porosity PHIT_SH = (RHOB_DSH − RHOB_SH)/(RHOB_DSH − RHOB_FL); CBW = VSH·VOL_CBW_SH; CAPBW = VSH·(PHIT_SH − VOL_CBW_SH). Key message: PHIE = PHIT − CBW (clay bound only); PHIFF = PHIT − CBW − CAPBW is the movable-fluid porosity; SWIRR = (CBW+CAPBW)/PHIT floored at SWIRR_MIN. NPHI must be sandstone units. Exec arithmetic reconstructed from the reference spec — check against the reference PHIT/PHIE LAS output.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Bulk density | `RHOB` | yes | — |
| NPHI | Neutron porosity (sandstone units) | `NPHI` | no | — |
| VSH | Shale volume | `VSH` | yes | accepts quantity kind: VSH |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHOB_MAT *(g/cc)*

Bulk density of matrix point

- **Default:** 2.65 — source: IP/Techlog/SandiMin sandstone matrix endpoint 2.65 g/cm3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 2 to 3 g/cc

### NPHI_MAT *(v/v)*

Neutron of matrix point

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.1 to 0.2 v/v

### RHOB_SH *(g/cc)*

Bulk density of measured (wet) shale

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1.5 to 3.5 g/cc

### NPHI_SH *(v/v)*

Neutron of measured shale

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### RHOB_DSH *(g/cc)*

Dry shale grain density (0 p.u. shale)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3 g/cc

### VOL_CBW_SH *(v/v)*

Clay-bound water volume in wet shale

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### SWIRR_MIN *(v/v)*

Minimum irreducible water saturation

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### GAS_C

Gas-conditioning weight (0 = density only, 1 = even, 2 = neutron only)

- **Default:** 1.6 — source: porosity_sspw.lls (2022) gas branch c = 1.6; RULED by Jauhar DEC-086 on field observation that the even split still reads optimistic
- **Accepted range:** 0 to 2

### RHO_W *(g/cc)*

Formation water density

- **Default:** 1 — source: Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.8 to 1.3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `formation_water_density`); the pane lists them with sources at the point of choice.

### RHOB_FL *(g/cc)*

Density of invaded-zone fluid

- **Default:** 1 — source: Geolog V14 phi_dnh.info RHO_MF DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.4
- **Accepted range:** 0.5 to 1.5 g/cc

### NPHI_FL *(v/v)*

Neutron response of flushed-zone fluid

- **Default:** 1 — source: Geolog V14 vsh_dn.info and Techlog VSH neutron-density NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5
- **Accepted range:** 0.5 to 1.2 v/v

## Output curves

| Name | Description |
|---|---|
| PHIT_SSPW | Total porosity |
| PHIE_SSPW | Effective porosity (PHIT − CBW) |
| PHIFF_SSPW | Free fluid porosity |
| CBW_SSPW | Clay-bound water volume |
| CAPBW_SSPW | Capillary-bound water volume |
| BW_SSPW | Total bound water volume |
| SWIRR_SSPW | Irreducible water saturation |
