<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# GR Hole-Size Correction

Module id `gr_hole_corr` · category **Prep** · [reference index](README.md)

GR_EC = GR * (1 + K_GR*(CALI - BS)): linear borehole-enlargement correction — gamma rays attenuated by the extra mud annulus are restored. Bit size from the BS curve where present, else BS_DEF. The public runner refuses if CALI is missing at any finite GR sample; it never writes an unmarked uncorrected GR_EC copy.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| GR | Gamma ray log | `GR` | yes | — |
| CALI | Caliper log | `CALI` | no | — |
| BS | Bit size log | `BS` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### K_GR *(1/in)*

Correction per inch of enlargement

- **Default:** 0.0075 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 0.05 1/in

### BS_DEF *(in)*

Bit size when BS curve is absent

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 3 to 30 in

## Output curves

| Name | Description |
|---|---|
| GR_EC | Environmentally corrected gamma ray |
| GR_EC_FULL | Correction applied in full at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| GR_EC_PARTIAL | Correction applied in part at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| GR_EC_NONE | Correction not applied at this sample - the raw value passed through under the corrected name (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| GR_EC_REFUSED | Refused on a precondition at this sample (DEC-060 ENVCORR one-hot group; refusals today are whole-run and carry no outputs, so 0 wherever sampled) *(flag: DIAGNOSTIC_INDICATOR)* |
