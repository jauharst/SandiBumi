<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Depth Shift

Module id `depth_shift` · category **Prep** · [reference index](README.md)

Shifts CURVE by SHIFT metres (+ = the feature moves DEEPER) and resamples it back onto the well's depth grid by linear interpolation. SHIFT is zone-overridable, so different intervals can take different block shifts. The result is written as <CURVE>_DS; the input curve is never modified.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to shift | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### SHIFT *(m)*

Depth shift (+ = deeper)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000 to 1000 m

## Output curves

| Name | Description |
|---|---|
| CURVE_DS | Depth-shifted copy |
