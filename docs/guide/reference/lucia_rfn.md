<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Lucia Rock-Fabric Number (carbonate)

Module id `lucia_rfn` · category **Rock Typing** · [reference index](README.md)

Carbonate rock typing by Lucia rock-fabric number (Jennings & Lucia 2003). Inverts the global transform log10 k = (A − B·log10 RFN) + (C − D·log10 RFN)·log10 φip analytically for RFN, then bins: RFN 0.5–1.5 = Class 1 (grainstone), 1.5–2.5 = Class 2, 2.5–4 = Class 3 (mud-dominated). PHI should be INTERPARTICLE porosity (subtract vuggy/separate-vug porosity if available); k in mD. Writes RFN and RT_LUCIA (1–3; MISSING outside the 0.5–4 band). Clastic-dominated fields use this only for carbonate stringers. Constants transcribed from the paper — verify first.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHI | Interparticle porosity | `PHIE` | yes | — |
| PERM | Permeability | `PERM` | yes | — |

## Output curves

| Name | Description |
|---|---|
| RFN | Lucia rock-fabric number |
| RT_LUCIA | Lucia class (1–3) |
