<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Synthetic Log (KNN Predict)

Module id `log_predict` · category **Prep** · [reference index](README.md)

Facimage-style synthetic log: trains on the samples of THIS run where TARGET and every supplied predictor are present, then predicts TARGET everywhere the predictors exist by distance-weighted K-nearest-neighbour regression (predictors z-scored; training set decimated to ≤4000 points). OPT_COMBINE: SYNTHETIC writes the pure prediction; FILL_MISSING keeps the raw value where present; MAX_RAW takes max(raw, synthetic) — the washout rule for RHOB, since bad hole only pushes RHOB down. Output is named <TARGET>_SYN. Mask the run to good-hole intervals so bad samples never train the model.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| TARGET | Curve to predict | `RHOB` | yes | — |
| P1 | Predictor 1 | `GR` | yes | — |
| P2 | Predictor 2 (optional) | `NPHI` | no | — |
| P3 | Predictor 3 (optional) | `DT` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### K

Number of neighbours

- **Default:** 10 — source: Geolog V14 facimage_05_using_hc.5.05.html Nearest Neighbors Default 10; docs/PRD_v2/24_ml-advanced.md §5
- **Accepted range:** 1 to 50

## Options

### OPT_COMBINE

How to combine with the raw curve

- **Choices:**
  - `SYNTHETIC`
  - `FILL_MISSING`
  - `MAX_RAW`
- **Default:** `SYNTHETIC`

## Output curves

| Name | Description |
|---|---|
| SYN | Synthetic curve |
