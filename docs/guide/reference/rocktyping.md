<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Rock Typing (FZI / R35 / PGS)

Module id `rocktyping` · category **Rock Typing** · [reference index](README.md)

Per-sample rock-typing indicators from porosity and permeability. Writes RQI = 0.0314·√(k/φ), PHIZ = φ/(1−φ), FZI = RQI/PHIZ (Amaefule 1993); Winland R35 = 10^(0.732 + 0.588·log10 k − 0.864·log10 φ%) (Kolodzie 1980); and the Permadi-Susilo PGS pair PGEOM = √(k/φ), PSTRUC = k/φ^PS_EXP. RT_CLASS is the rock-type class from the chosen METHOD — GHE fixed FZI bins (Corbett-Potter 2004) or Winland port classes (nano..mega). PERM_RT is the class-grouped permeability estimate k = 1014.24·FZI_mean(RT_CLASS)²·φ³/(1−φ)² using each class's GEOMETRIC-MEAN FZI over this well. k in mD, φ in v/v; samples with φ∉(0,1) or k≤0 stay MISSING. GHE bins follow the Corbett-Potter 2004 ×2 series and PGS uses √(k/φ) / k/φ³ (verified 2026-07-22).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHI | Effective porosity | `PHIE` | yes | — |
| PERM | Permeability | `PERM` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### PS_EXP *(-)*

PGS pore-structure exponent (k/φ^PS_EXP)

- **Default:** 3 — source: docs/constants_verification_2026-07-22.md Permadi-Susilo exponent re-verification; docs/PRD_v2/15_sat-height-rocktyping.md §5.4
- **Accepted range:** 1 to 6 -

## Options

### METHOD

Rock-type class basis

- **Choices:**
  - `ghe`
  - `winland_port`
- **Default:** `ghe`

## Output curves

| Name | Description |
|---|---|
| RQI | Reservoir quality index 0.0314·√(k/φ) |
| PHIZ | Normalized porosity φ/(1−φ) |
| FZI | Flow zone indicator RQI/PHIZ |
| R35 | Winland R35 pore-throat radius |
| PGEOM | PGS pore geometry √(k/φ) |
| PSTRUC | PGS pore structure k/φ^PS_EXP |
| RT_CLASS | Rock-type class (GHE 1..10 or port 1..5) |
| PERM_RT | Class-grouped permeability estimate |
