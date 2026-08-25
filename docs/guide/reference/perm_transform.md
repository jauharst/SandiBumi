<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Permeability — Por-Perm Transform

Module id `perm_transform` · category **Permeability** · [reference index](README.md)

log10(PERM) = PT_A * PHIE + PT_B — the classic core-derived porosity-permeability regression. Calibrate PT_A/PT_B per zone from RCAL data.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHIE | Limited effective porosity | `PHIE` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### PT_A

Slope

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 100

### PT_B

Intercept

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -10 to 5

## Output curves

| Name | Description |
|---|---|
| PERM_XFM | Permeability from transform |
| PERM | Working permeability |
