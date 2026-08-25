<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Porosity from Density

Module id `phi_den` · category **Porosity** · [reference index](README.md)

PHIE = (RHO_MA - RHOB)/(RHO_MA - RHO_FL) - VSH*(RHO_MA - RHO_SH)/(RHO_MA - RHO_FL). PHIT = PHIE + VSH*PHIT_SH, where PHIT_SH = (RHO_DSH - RHO_SH)/(RHO_DSH - RHO_W). Above 95% VSH the sample is treated as shale.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| BADHOLE | Bad-hole flag (flagged samples excluded, exclusion recorded) | `BADHOLE` | no | — |
| GAS_FLAG | Gas-crossover flag from condflag (provenance record only - never a correction) | `XOVER_FLAG` | no | — |
| COAL_FLAG | Coal flag from condflag (flagged samples blanked - coal apparent porosity is never rock porosity) | `COAL_FLAG` | no | — |
| TIGHT_FLAG | Tight flag from condflag (indicator only - never alters an output) | `TIGHT_FLAG` | no | — |
| COND_FLAG | Conditioning flag from condflag (indicator only - never alters an output) | `COND_FLAG` | no | — |
| RHOB | Density log | `RHOB` | yes | — |
| VSH | Limited volume of shale | `VSH` | yes | accepts quantity kind: VSH |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_MA *(g/cc)*

Matrix density

- **Default:** 2.65 — source: IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `matrix_density`); the pane lists them with sources at the point of choice.

### RHO_SH *(g/cc)*

Shale density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1.5 to 3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `shale_density`); the pane lists them with sources at the point of choice.

### RHO_FL *(g/cc)*

Fluid density

- **Default:** 1 — source: IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.5 to 1.5 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.

### RHO_DSH *(g/cc)*

Dry shale density

- **Default:** 2.7 — source: Jauhar adjudication DEC-069 (2026-08-18): 2.70 g/cc from multi-basin Indonesian experience; clay-mineral bracket kaolinite 2.62-smectite 2.68 g/cc; 2.65 rejected (matches no held source: IP 2.78, Techlog 2.85, Geolog none); docs/takeover/DECISIONS.md
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `dry_shale_density`); the pane lists them with sources at the point of choice.

### RHO_W *(g/cc)*

Formation water density

- **Default:** 1 — source: Geolog V14 phi_den.info RHO_W DEFAULT 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.8 to 1.3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `formation_water_density`); the pane lists them with sources at the point of choice.

### PHIE_MAX *(v/v)*

Maximum allowed PHIE

- **Default:** 0.3 — source: Geolog V14 phi_den.info PHIE_MAX DEFAULT 0.3; docs/PRD_v2/11_porosity.md §5.3
- **Accepted range:** 0.05 to 0.5 v/v
- Competing shipped values exist for this parameter across installed tools (topic `max_effective_porosity`); the pane lists them with sources at the point of choice.

### PHIE_FLOOR *(v/v)*

Floor applied to limited PHIE where the limit binds

- **Default:** 0.001 — source: Jauhar DEC-043 (2026-08-16) ruled 0.001 over ship-absent; DEC-067 (2026-08-18) ships it as the cited DEFAULT, user-settable per the chapter's documented-user-decision clause; DEC-091 (2026-08-21) closed the bottom of the range AT that default so no run can write a floor the pay summary would re-floor, putting IP's competing 0.0001 (F17) deliberately out of reach; upper guard 0.01 stays below any real cutoff; docs/takeover/DECISIONS.md
- **Accepted range:** 0.001 to 0.01 v/v

### DPHIMAX *(v/v)*

Smooth roll-off increment above PHIE_MAX (IP DeltaPhiMax)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v
- **Checked before the run:**
  - `dphimax.required_when_smooth_rolloff` — DPHIMAX is required when OPT_PHIEMAX = SMOOTH_ROLLOFF. Applies when: [object Object].
    Source: IP 2025 Interact.chm porosity-limit pages (image form embim71, D-11 adopted over the malformed swparameters.htm ASCII); NO default published - the corpus negative result and F21; docs/research_2026-07/ip2025_chm_ingest/B_core_petro.md; DEC-066 (2026-08-18)

### VCL_CUTOFF *(v/v)*

Smooth roll-off onset shale volume (IP VclCutoff)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v
- **Checked before the run:**
  - `vcl_cutoff.required_when_smooth_rolloff` — VCL_CUTOFF is required when OPT_PHIEMAX = SMOOTH_ROLLOFF. Applies when: [object Object].
    Source: IP 2025 Interact.chm porosity-limit pages (image form embim71, D-11 adopted over the malformed swparameters.htm ASCII); NO default published - the corpus negative result and F21; docs/research_2026-07/ip2025_chm_ingest/B_core_petro.md; DEC-066 (2026-08-18)

### VSH_SHALE *(v/v)*

High-shale kill threshold (at or above it: PHIE = 0, PHIT = PHIT_SH)

- **Default:** 0.95 — source: Geolog V14 phi_*.lls hard-coded VSH >= 0.95 (all six modules); docs/PRD_v2/11_porosity.md §5 line 1229 makes it a parameter in SandiBumi defaulting to 0.95 with this source
- **Accepted range:** 0 to 1 v/v
- Competing shipped values exist for this parameter across installed tools (topic `high_shale_branch_threshold`); the pane lists them with sources at the point of choice.

## Options

### OPT_PHIEMAX

PHIE limiting method

- **Choices:**
  - `SHALE_REDUCED`
  - `MAXIMUM`
  - `SMOOTH_ROLLOFF`
- **Default:** `SHALE_REDUCED`

## Output curves

| Name | Description |
|---|---|
| PHIE_DEN | PHIE from density (unlimited) |
| PHIT_DEN | PHIT from density (unlimited) |
| PHIE | Limited effective porosity |
| PHIT | Limited total porosity |
