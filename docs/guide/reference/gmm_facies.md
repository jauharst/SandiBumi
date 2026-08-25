<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Electrofacies (GMM, soft)

Module id `gmm_facies` · category **Facies** · [reference index](README.md)

Soft electrofacies: a Gaussian mixture model (diagonal covariance, EM, initialized from k-means) clusters this well's samples in the space of the supplied curves. Unlike k-means, every sample gets a membership PROBABILITY per facies — FPROB is the winning facies' posterior (1.0 = unambiguous, ~1/K = boundary/mixed sample), so transitional beds are visible instead of being forced into a class. Labels are ordered by the mean of the first supplied curve (usually GR). Deterministic for a given seed. Outputs: FACIES_GMM (integer 0..K-1), FPROB (max posterior, 0-1).

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

Number of facies (mixture components)

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
| FACIES_GMM | GMM facies index (0..K-1) |
| FPROB | Posterior probability of the winning facies |
| FACIES_GMM_SIL | Silhouette per sample: how far apart the clusters actually are here, which is a different question from how sure the mixture is (FPROB) |
| FACIES_GMM_CRI | Cluster randomness index of the facies at this depth: 1.0 is indistinguishable from a random arrangement, 3.0 means its beds are three times thicker than chance |
