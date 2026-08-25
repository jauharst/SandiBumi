<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Multimin — Mineral Inversion (retired · use SandiMin)

Module id `multimin` · category **Saturation** · [reference index](README.md)

RETIRED — superseded by SandiMin (Advance ▸ Mineral Solver); running this step now returns a message directing you to SandiMin rather than executing the old fixed 4-component solver. The spec is kept only so a saved workflow chain that references it still resolves and shows its stored parameters. The former solver produced SAND/CLAY/WATER/HYDROCARBON volumes plus PHIT_MM/VSH_MM/SWT_MM and RECON_ERR from RHOB/NPHI/DT/PEF.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Density log | `RHOB` | no | — |
| NPHI | Neutron porosity log | `NPHI` | no | — |
| DT | Sonic transit time log | `DT` | no | — |
| PEF | Photoelectric factor log | `PEF` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHOB_SAND *(g/cc)*

Sand grain density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc

### RHOB_CLAY *(g/cc)*

Clay density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc

### RHOB_WATER *(g/cc)*

Water density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.8 to 1.3 g/cc

### RHOB_HC *(g/cc)*

Hydrocarbon density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 1.1 g/cc

### NPHI_SAND *(v/v)*

Sand neutron

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.15 to 0.5 v/v

### NPHI_CLAY *(v/v)*

Clay neutron

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.8 v/v

### NPHI_WATER *(v/v)*

Water neutron

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.5 to 1.2 v/v

### NPHI_HC *(v/v)*

Hydrocarbon neutron

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1.2 v/v

### DT_SAND *(us/ft)*

Sand transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 40 to 70 us/ft

### DT_CLAY *(us/ft)*

Clay transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 60 to 150 us/ft

### DT_WATER *(us/ft)*

Water transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 150 to 220 us/ft

### DT_HC *(us/ft)*

Hydrocarbon transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 150 to 260 us/ft

### PEF_SAND *(b/e)*

Sand photoelectric factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 6 b/e

### PEF_CLAY *(b/e)*

Clay photoelectric factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 6 b/e

### PEF_WATER *(b/e)*

Water photoelectric factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 2 b/e

### PEF_HC *(b/e)*

Hydrocarbon photoelectric factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 2 b/e

### SIG_RHOB *(g/cc)*

RHOB uncertainty

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.005 to 0.5 g/cc

### SIG_NPHI *(v/v)*

NPHI uncertainty

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.005 to 0.5 v/v

### SIG_DT *(us/ft)*

DT uncertainty

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.5 to 50 us/ft

### SIG_PEF *(b/e)*

PEF uncertainty

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.02 to 3 b/e

### W_UNITY

Unity-constraint weight

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 1000000

## Output curves

| Name | Description |
|---|---|
| VOL_SAND | Sand (quartz) volume |
| VOL_CLAY | Clay volume |
| VOL_WATER | Water volume |
| VOL_HC | Hydrocarbon volume |
| PHIT_MM | Total porosity (water + hc) |
| VSH_MM | Shale volume (= clay) |
| SWT_MM | Total water saturation |
| RECON_ERR | Reconstruction error (sigma units) |
