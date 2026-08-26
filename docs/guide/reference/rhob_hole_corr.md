<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Density Hole-Size Correction

Module id `rhob_hole_corr` · category **Prep** · [reference index](README.md)

RHOB_EC = RHOB + K_RHO*(CALI - HD_REF) for CALI beyond HD_REF: in oversize holes the pad reads too much mud, so density is restored upward using supplied, tool-specific chart values. Within gauge RHOB may remain unchanged; the public runner refuses if CALI is missing at any finite RHOB sample. Use with the BADHOLE flag — beyond a few inches of washout no correction is trustworthy.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Density log | `RHOB` | yes | — |
| CALI | Caliper log | `CALI` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### K_RHO *(g/cc/in)*

Correction per inch beyond reference

- **Default:** 0.004 — source: Adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 0.05 g/cc/in

### HD_REF *(in)*

Hole diameter where correction starts

- **Default:** 10 — source: Adjudication DEC-077 (2026-08-19): 10 in multi-basin starting value per DEC-059 - the vendor reference diameter is PER-TOOL and stated on each chart (Halliburton Chart 2-1: 6.5 in for the 4.75-in DGR; SB-ENV-013 records it as a property of tool and bit), so no universal number is adoptable; docs/takeover/DECISIONS.md
- **Accepted range:** 4 to 20 in

## Output curves

| Name | Description |
|---|---|
| RHOB_EC | Environmentally corrected density |
| RHOB_EC_FULL | Correction applied in full at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| RHOB_EC_PARTIAL | Correction applied in part at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| RHOB_EC_NONE | Correction not applied at this sample - the raw value passed through under the corrected name (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| RHOB_EC_REFUSED | Refused on a precondition at this sample (DEC-060 ENVCORR one-hot group; refusals today are whole-run and carry no outputs, so 0 wherever sampled) *(flag: DIAGNOSTIC_INDICATOR)* |
