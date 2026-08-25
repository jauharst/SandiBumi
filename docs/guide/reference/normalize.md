<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Normalize

Module id `normalize` · category **Condition** · [reference index](README.md)

Maps a curve onto a common reference frame so wells can be compared and pooled. Works on ANY curve — the arithmetic is not specific to gamma ray.

METHOD:
• TWO_POINT — the workhorse. Reads this run's P_LOW and P_HIGH of the curve and maps them linearly onto REF_LOW and REF_HIGH. Percentiles rather than min/max because a single spike sets a min or a max, and one bad sample would then re-scale the whole well.
• RANGE — the same map from the curve's own MIN and MAX. Reproducible, and spike-sensitive by construction: use it on a curve you have already despiked.
• MEAN_SD — z-score to REF_MEAN and REF_SD. The right choice when the distribution matters more than its ends (feeding a classifier, comparing shapes).

SPACE: LOG works in log10 and inverts afterwards, which is the honest frame for a resistivity or a permeability — those are read on a log scale, and a linear map stretches the bottom decade out of all proportion to the top. Non-positive samples have no logarithm and become MISSING, and the run says how many.

THE REFERENCE PAIR HAS NO DEFAULT, and that is the point of the module. A pair from one basin is the wrong pair in another; normalized output looks entirely plausible either way, and nothing downstream can catch it. Derive yours from the field's own multi-well distribution or from a reference well everyone agrees on, then use the SAME pair for every well in the study. QC by overlaying the normalized histograms — every well's P_LOW and P_HIGH should coincide.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to normalize | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### P_LOW *(%)*

TWO_POINT: low percentile

- **Default:** 3 — source: docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3
- **Accepted range:** 0 to 50 %
- Competing shipped values exist for this parameter across installed tools (topic `percentile_reference_low`); the pane lists them with sources at the point of choice.

### P_HIGH *(%)*

TWO_POINT: high percentile

- **Default:** 97 — source: docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3
- **Accepted range:** 50 to 100 %
- Competing shipped values exist for this parameter across installed tools (topic `percentile_reference_high`); the pane lists them with sources at the point of choice.

### REF_LOW

TWO_POINT / RANGE: reference value at the low end

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000
- **Checked before the run:**
  - `ref_low.required_when_two_point` — REF_LOW is required when OPT_METHOD = TWO_POINT. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters
  - `ref_low.required_when_range` — REF_LOW is required when OPT_METHOD = RANGE. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters

### REF_HIGH

TWO_POINT / RANGE: reference value at the high end

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000
- **Checked before the run:**
  - `ref_high.required_when_two_point` — REF_HIGH is required when OPT_METHOD = TWO_POINT. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters
  - `ref_high.required_when_range` — REF_HIGH is required when OPT_METHOD = RANGE. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters

### REF_MEAN

MEAN_SD: reference mean

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000
- **Checked before the run:**
  - `ref_mean.required_when_mean_sd` — REF_MEAN is required when OPT_METHOD = MEAN_SD. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters

### REF_SD

MEAN_SD: reference standard deviation

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1e-9 to 1000000000
- **Checked before the run:**
  - `ref_sd.required_when_mean_sd` — REF_SD is required when OPT_METHOD = MEAN_SD. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 normalization parameters

## Options

### OPT_METHOD

How the curve is mapped

- **Choices:**
  - `TWO_POINT` — percentiles onto a reference pair
  - `RANGE` — min and max onto a reference pair
  - `MEAN_SD` — z-score onto a reference mean and spread
- **Default:** `TWO_POINT`

### OPT_SPACE

Linear or logarithmic

- **Choices:**
  - `LINEAR` — for GR, NPHI, RHOB, DT, a volume fraction
  - `LOG` — for RT, PERM, anything read on a log scale
- **Default:** `LINEAR`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Normalized curve |
