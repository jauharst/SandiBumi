<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Gas Correction (density, iterated)

Module id `gascorr` · category **Prep** · [reference index](README.md)

Removes the gas effect from RHOB (iterated density-neutron loop): density porosity and Archie SWT are solved from the current density, then RHOB_GC = RHOB + PHIT*(1-SWT)*(RHO_FL - GASDEN) replaces the gas volume with liquid, iterated until PHIT moves less than 1e-4 (max 20 passes; non-converging samples stay MISSING). GASDEN is the real-gas density of an SG_GAS gas at FPRESS/FTEMP (Standing pseudo-criticals + Papay z-factor) — run the precalc module first; samples without P/T, RT or Rw stay MISSING rather than passing through uncorrected. The default OPT_GATE = FLAGGED corrects only where GAS_FLAG > 0.5 (chain condflag's XOVER_FLAG, which already excludes coal and washout) and errors if the flag curve has no data. OPT_GATE = EVERYWHERE corrects every sample — beware: high-resistivity low-density beds (coal, resistive washouts) read as gas to the Archie loop and get large spurious corrections. QC per slides 66-67: the detached high-porosity gas cloud on PHIE vs wet-clay collapses after correction. Feed RHOB_GC to phi_den (or use PHIT_GC directly) — NOT to phi_dn or a SandiMin solve that includes NPHI: their gas handling assumes an uncorrected density-neutron pair, so a corrected RHOB with a still-gas-affected NPHI biases porosity low.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Bulk density | `RHOB` | yes | — |
| RT | True formation resistivity | `RES_DEEP` | yes | — |
| FTEMP | Formation temperature (precalc) | `FTEMP` | yes | resolved from computed curves only, never the RAW import store |
| FPRESS | Formation pressure (precalc) | `FPRESS` | yes | resolved from computed curves only, never the RAW import store |
| GAS_FLAG | Gas-zone flag for FLAGGED gating | `XOVER_FLAG` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_MA *(g/cc)*

Matrix density

- **Default:** 2.65 — source: IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16.
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `matrix_density`); the pane lists them with sources at the point of choice.

### RHO_FL *(g/cc)*

Liquid (filtrate) density the correction restores

- **Default:** 1 — source: Geolog V14 phi_dnh.info RHO_MF DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.4
- **Accepted range:** 0.8 to 1.3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.

### SG_GAS

Gas specific gravity (air = 1)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.55 to 1.2

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

### OPT_GATE

Where to apply the correction

- **Choices:**
  - `FLAGGED`
  - `EVERYWHERE`
- **Default:** `FLAGGED`

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
| RHOB_GC | Gas-corrected bulk density |
| PHIT_GC | Density porosity from the corrected RHOB (converged) |
| SWT_GC | Archie SWT at convergence |
| GASDEN | Gas density at reservoir P/T (QC) |
