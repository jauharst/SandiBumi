<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Thin Beds — Thomas-Stieber

Module id `thin_bed_ts` · category **ThinBeds** · [reference index](README.md)

Decomposes bulk VSH into laminar and dispersed shale by comparing the measured (VSH, PHIT) point against the pure-laminated line PHIT = PHI_SD_MAX*(1-VSH) + PHI_SH*VSH and the pure-dispersed line PHIT = PHI_SD_MAX - VSH*(1-PHI_SH). VLAM reduces net sand (VSAND = 1-VLAM); VDISP stays within the sand fraction. PHIE_LAM is the laminar-shale-corrected porosity of the net sand. Structural shale is not modeled.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHIT | Total porosity log | `PHIT` | yes | — |
| VSH | Total (bulk) volume of shale log | `VSH` | yes | accepts quantity kind: VSH |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### PHI_SD_MAX *(v/v)*

Clean sand porosity (endpoint)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.05 to 0.45 v/v

### PHI_SH *(v/v)*

Shale porosity (endpoint)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.45 v/v

## Output curves

| Name | Description |
|---|---|
| VLAM | Laminar shale volume fraction |
| VDISP | Dispersed shale volume fraction |
| VSAND | Net sand (non-laminar) fraction |
| PHIE_LAM | Laminar-shale-corrected sand porosity |
