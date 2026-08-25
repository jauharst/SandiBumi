<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Gas-in-place (free + Langmuir adsorbed)

Module id `gip` · category **Unconventional** · [reference index](README.md)

Per-sample gas-in-place as gas CONTENT (scf per ton of rock), so it composites like any curve. Adsorbed via the Langmuir isotherm GIP_ADS = VL·P/(PL+P); free via GIP_FREE = 32.0368·φ·(1−Sw)/(RHOB·Bg) with Bg = 0.02827·z·T/P (T in Rankine); GIP_TOTAL = free + adsorbed. MODE=cbm applies the dry-ash-free correction GIP_ADS·(1−F_ASH−F_MOIST) and, given a measured in-situ gas content GC, emits the critical desorption pressure PCD = PL·GC/(VL−GC). Langmuir VL/PL ship absent and require matching core desorption/isotherm data. Ambrose pore-volume correction deferred. Cite: Langmuir 1918; Ambrose et al. 2010; GRI/Mavor-Nelson 1996. See docs/ref_unconventional.md §3.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHI | Porosity (effective, or OM-corrected total) | `PHIE` | yes | — |
| SW | Water saturation | `SWE` | yes | — |
| RHOB | Bulk density | `RHOB` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RES_P *(psia)*

Reservoir (pore) pressure

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 30000 psia

### TEMP_F *(degF)*

Reservoir temperature

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 32 to 600 degF

### Z_FAC *(-)*

Gas deviation (compressibility) factor z

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.2 to 2 -

### VL *(scf/ton)*

Langmuir volume (max sorption)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 5000 scf/ton

### PL *(psia)*

Langmuir pressure (Gs = VL/2)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 30000 psia

### F_ASH *(-)*

Ash weight fraction (cbm)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 -
- **Checked before the run:**
  - `f_ash.required_when_cbm` — F_ASH is required when MODE = cbm. Applies when: [object Object].
    Source: docs/PRD_v2/19_toc-unconventional.md SB-TOC-019 and §5

### F_MOIST *(-)*

Moisture weight fraction (cbm)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 -
- **Checked before the run:**
  - `f_moist.required_when_cbm` — F_MOIST is required when MODE = cbm. Applies when: [object Object].
    Source: docs/PRD_v2/19_toc-unconventional.md SB-TOC-019 and §5

### GC *(scf/ton)*

In-situ gas content for PCD (cbm; 0 = saturated)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 5000 scf/ton
- **Checked before the run:**
  - `gc.required_when_cbm` — GC is required when MODE = cbm. Applies when: [object Object].
    Source: docs/PRD_v2/19_toc-unconventional.md SB-TOC-019 and §5

## Options

### MODE

Reservoir type (cbm adds ash/moisture + critical desorption)

- **Choices:**
  - `shale`
  - `cbm`
- **Default:** `shale`

## Output curves

| Name | Description |
|---|---|
| BG | Gas formation volume factor |
| GIP_ADS | Adsorbed gas content (Langmuir) |
| GIP_FREE | Free gas content |
| GIP_TOTAL | Total gas content (free + adsorbed) |
| PCD | Critical desorption pressure (cbm) |
