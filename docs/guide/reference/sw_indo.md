<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SW — Indonesia (Poupon-Leveaux)

Module id `sw_indo` · category **Saturation** · [reference index](README.md)

1/RT = (v/RT_SH + PHIE^M/(A*Rw) + 2*sqrt(v*PHIE^M/(A*Rw*RT_SH))) * SW^N, v = VSH^(2-VSH) (FULL), VSH^2 (SIMPLE), VSH^(2-2*VSH) (TAR_SAND). Poupon & Leveaux (1971).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| FTEMP | Formation temperature (precalc, for MEASURED/SALINITY) | `FTEMP` | no | resolved from computed curves only, never the RAW import store |
| RT | True formation resistivity | `RES_DEEP` | yes | — |
| PHIE | Limited effective porosity | `PHIE` | yes | — |
| VSH | Limited volume of shale | `VSH` | yes | accepts quantity kind: VSH |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### A

Tortuosity constant

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 5
- Competing shipped values exist for this parameter across installed tools (topic `archie_a`); the pane lists them with sources at the point of choice.

### M

Cementation exponent

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4
- Competing shipped values exist for this parameter across installed tools (topic `archie_m`); the pane lists them with sources at the point of choice.

### N

Saturation exponent

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4
- Competing shipped values exist for this parameter across installed tools (topic `archie_n`); the pane lists them with sources at the point of choice.

### RT_SH *(ohmm)*

Shale resistivity

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 500 ohmm
- Competing shipped values exist for this parameter across installed tools (topic `shale_resistivity`); the pane lists them with sources at the point of choice.

### SWE_IRR *(v/v)*

Irreducible effective water saturation

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.6 v/v
- Competing shipped values exist for this parameter across installed tools (topic `irreducible_swe`); the pane lists them with sources at the point of choice.

### RW *(ohmm)*

Rw at formation temperature (CONSTANT)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 20 ohmm
- Competing shipped values exist for this parameter across installed tools (topic `formation_water_resistivity`); the pane lists them with sources at the point of choice.
- **Checked before the run:**
  - `rw.required_when_constant` — RW is required when OPT_RW = CONSTANT. Applies when: [object Object].
    Source: docs/PRD_v2/12_saturation.md §5 formation-water parameters

### RWS *(ohmm)*

Measured water sample resistivity

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 20 ohmm
- **Checked before the run:**
  - `rws.required_when_measured` — RWS is required when OPT_RW = MEASURED. Applies when: [object Object].
    Source: docs/PRD_v2/12_saturation.md §5 formation-water parameters

### RWT *(degC)*

Temperature of RWS measurement

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 150 degC
- **Checked before the run:**
  - `rwt.required_when_measured` — RWT is required when OPT_RW = MEASURED. Applies when: [object Object].
    Source: docs/PRD_v2/12_saturation.md §5 formation-water parameters

### SALW *(ppm)*

Formation water salinity

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 100 to 300000 ppm
- **Checked before the run:**
  - `salw.required_when_salinity` — SALW is required when OPT_RW = SALINITY. Applies when: [object Object].
    Source: docs/PRD_v2/12_saturation.md §5 formation-water parameters

## Options

### OPT_INDO

Indonesia VSH exponent variant

- **Choices:**
  - `FULL`
  - `SIMPLE`
  - `TAR_SAND`
- **Default:** `FULL`

### OPT_RW

Formation water resistivity source

- **Choices:**
  - `CONSTANT`
  - `MEASURED`
  - `SALINITY`
- **Default:** `CONSTANT`

## Output curves

| Name | Description |
|---|---|
| SWE_INDO | SWE from Indonesia (unlimited) |
| SWE | Limited effective water saturation |
| VOL_UWAT | Volume of water (unflushed) |
| SW_METHOD | Producing saturation equation (categorical method code) |
