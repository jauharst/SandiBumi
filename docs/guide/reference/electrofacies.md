<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Electrofacies (K-means)

Module id `electrofacies` · category **Facies** · [reference index](README.md)

Unsupervised electrofacies: k-means clusters the samples of THIS well in the space of the supplied curves (each feature z-scored by default, so mixed units are comparable) into K facies. Any curve slot with no data is dropped; a sample missing any present curve gets FACIES = MISSING. Cluster labels are ordered by the mean of the first supplied curve (usually GR), so FACIES 0 is the cleanest class and the numbering is monotone in shaliness. Clustering is per well and deterministic for a given seed. Output: FACIES (integer 0..K-1).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE1 | Curve 1 (also orders the facies) | `GR` | yes | — |
| CURVE2 | Curve 2 (optional) | `RHOB` | no | — |
| CURVE3 | Curve 3 (optional) | `NPHI` | no | — |
| CURVE4 | Curve 4 (optional) | `DT` | no | — |
| CURVE5 | Curve 5 (optional) | `SP` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### K

Number of facies (clusters)

- **Default:** 5 — source: SandiBumi facies.rs native K default 5, corroborated by two Techlog modules; docs/PRD_v2/24_ml-advanced.md §5
- **Accepted range:** 2 to 12
- Competing shipped values exist for this parameter across installed tools (topic `cluster_count`); the pane lists them with sources at the point of choice.

### SEED

Random seed (reproducibility)

- **Default:** 42 — source: SandiBumi shared ML seed decision at ml.rs:64 and facies.rs SEED_DEFAULT; docs/PRD_v2/24_ml-advanced.md §5
- **Accepted range:** 0 to 1000000000

## Options

### OPT_STANDARDIZE

Feature scaling

- **Choices:**
  - `ZSCORE`
  - `NONE`
- **Default:** `ZSCORE`

## Output curves

| Name | Description |
|---|---|
| FACIES | Electrofacies cluster index (0..K-1) |
| FACIES_SIL | Silhouette per sample: +1 fits its cluster well, 0 on a boundary, NEGATIVE means it sits closer to another cluster than its own |
| FACIES_CRI | Cluster randomness index of the facies at this depth: 1.0 is indistinguishable from a random arrangement, 3.0 means its beds are three times thicker than chance |
