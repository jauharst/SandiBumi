<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Rock Type from Cutoffs (electrofacies)

Module id `rt_cutoff` · category **Rock Typing** · [reference index](README.md)

Log-domain rock-type class from a Vsh + PHIE cutoff ladder — the electrofacies half of the rock-typing tie-in. RT_LOG = 1 (best: Vsh ≤ VSH1 and PHIE ≥ PHI1), 2 (moderate: Vsh ≤ VSH2 and PHIE ≥ PHI2), else 3 (non-net). Requires VSH1 ≤ VSH2 and PHI1 ≥ PHI2. Feed the result to the confusion-matrix QC (Rock Typing ▸ Facies Tie-in) to validate it against a core-derived RT curve, then attach per-class phi-k / SHF laws. Samples with missing Vsh or PHIE stay MISSING.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| VSH | Shale volume | `VSH` | yes | accepts quantity kind: VSH |
| PHIE | Effective porosity | `PHIE` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### VSH1 *(v/v)*

Vsh cutoff for RT1 (best)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v
- **Checked before the run:**
  - `rt_cutoff.vsh_ladder_order` — The best class's shale cutoff must not exceed the moderate class's — equal is allowed, and separates the two on porosity alone.
    Source: docs/research_2026-07/ref_rocktyping_shf.md §Cutoff-based electrofacies tie-in, which writes the middle class as v1 <= Vsh < v2

### PHI1 *(v/v)*

PHIE cutoff for RT1 (best)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### VSH2 *(v/v)*

Vsh cutoff for RT2 (moderate)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### PHI2 *(v/v)*

PHIE cutoff for RT2 (moderate)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v
- **Checked before the run:**
  - `rt_cutoff.phi_ladder_order` — The moderate class's porosity floor must not exceed the best class's — equal is allowed, and separates the two on shale volume alone.
    Source: docs/research_2026-07/ref_rocktyping_shf.md §Cutoff-based electrofacies tie-in, which writes the middle class as p2 <= PHIE < p1

## Output curves

| Name | Description |
|---|---|
| RT_LOG | Cutoff rock-type class (1/2/3) |
