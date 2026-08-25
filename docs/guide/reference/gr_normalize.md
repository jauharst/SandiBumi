<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# GR Normalization (Two-Point Percentile)

Module id `gr_normalize` · category **Prep** · [reference index](README.md)

GRN = (GR − Plow_well)·(Phigh_ref − Plow_ref)/(Phigh_well − Plow_well) + Plow_ref. The well percentiles are computed from this run's GR samples (mask the run to a common reference interval so every well is measured over comparable rock); the reference percentiles are parameters. SET YOUR OWN FIELD REFERENCE PAIR — that is the entire point of the module. The pair ships absent: a reference pair from one basin is the wrong reference in another. Derive yours from the field's own multi-well GR distribution, or from a reference well everyone agrees on, then use the SAME pair for every well in the study. QC across wells with a GRN histogram overlay — the P3/P97 of every normalized well should coincide.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| GR | Gamma ray log | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### P_LOW *(%)*

Low percentile

- **Default:** 3 — source: docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3
- **Accepted range:** 0 to 50 %
- Competing shipped values exist for this parameter across installed tools (topic `percentile_reference_low`); the pane lists them with sources at the point of choice.
- **Guidance:** P3/P97 is a named SandiBumi house preset for selecting well percentiles. It selects positions in the distribution; it is not a gamma-ray endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5.1; docs/workflow_standards.md

### P_HIGH *(%)*

High percentile

- **Default:** 97 — source: docs/workflow_standards.md GR normalization P3/P97; docs/PRD_v2/20_envcorr-qc.md §5.3
- **Accepted range:** 50 to 100 %
- Competing shipped values exist for this parameter across installed tools (topic `percentile_reference_high`); the pane lists them with sources at the point of choice.
- **Guidance:** P3/P97 is a named SandiBumi house preset for selecting well percentiles. It selects positions in the distribution; it is not a gamma-ray endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5.1; docs/workflow_standards.md

### GR_LOW_REF *(gapi)*

Reference GR at low percentile

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 gapi
- **Guidance:** Compute well percentiles over a common reference interval containing comparable rock. Derive one reference pair from the study distribution or an agreed reference, then use that same pair for every well in the study.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; docs/workflow_standards.md

### GR_HIGH_REF *(gapi)*

Reference GR at high percentile

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 gapi
- **Guidance:** Compute well percentiles over a common reference interval containing comparable rock. Derive one reference pair from the study distribution or an agreed reference, then use that same pair for every well in the study.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; docs/workflow_standards.md

## Output curves

| Name | Description |
|---|---|
| GRN | Normalized gamma ray |
