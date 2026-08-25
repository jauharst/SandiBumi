<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Neutron Environmental Correction

Module id `nphi_env_corr` · category **Prep** · [reference index](README.md)

NPHI_EC = NPHI + K_TEMP*(FTEMP - T_REF) + K_SAL*(SALW/100000): linearized formation-temperature and formation-salinity terms. The coefficients ship as DEC-077 practitioner starting values - replace them with values read from the applicable CNL chart for the tool in hand where one is available. Requires FTEMP (run Formation Temperature first) for the temperature term; without it only the salinity term applies. SALW defaults to the chart reference condition itself (fresh, zero), so the salinity term is inert until the study declares its formation salinity.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| NPHI | Neutron porosity log | `NPHI` | yes | — |
| FTEMP | Formation temperature (precalc) | `FTEMP` | no | resolved from computed curves only, never the RAW import store |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### K_TEMP *(v/v per degC)*

Temperature coefficient

- **Default:** 0.0001 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** -0.01 to 0.01 v/v per degC

### T_REF *(degC)*

Chart reference temperature

- **Default:** 21.1 — source: Halliburton LWD Log Interpretation Charts (Sperry Drilling, 2018) book pp. 268/270: 'The reference temperature is 70 F' / fresh water at atmospheric pressure and 70 F (21.1 C); adopted by Jauhar adjudication DEC-077 (2026-08-19), replacing the uncited 24.0; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 100 degC

### K_SAL *(v/v)*

Salinity coefficient per 100 kppm

- **Default:** -0.002 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** -0.05 to 0.05 v/v

### SALW *(ppm)*

Formation water salinity

- **Default:** 0 — source: Fresh-water reference condition (concentration zero) - the three-vendor agreement: GE CNL panel ships 0 kppm (IP 2025 F_qc section 3), the SLB panel value is a unit artifact that is effectively fresh, and Halliburton's chart axes are kppm Cl- referenced to fresh water (book pp. 268-269); adopted by Jauhar adjudication DEC-077 (2026-08-19) - the correction is inert until the study declares its salinity; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 300000 ppm

## Output curves

| Name | Description |
|---|---|
| NPHI_EC | Environmentally corrected neutron porosity |
| NPHI_EC_FULL | Correction applied in full at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| NPHI_EC_PARTIAL | Correction applied in part at this sample (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| NPHI_EC_NONE | Correction not applied at this sample - the raw value passed through under the corrected name (DEC-060 ENVCORR one-hot group) *(flag: DIAGNOSTIC_INDICATOR)* |
| NPHI_EC_REFUSED | Refused on a precondition at this sample (DEC-060 ENVCORR one-hot group; refusals today are whole-run and carry no outputs, so 0 wherever sampled) *(flag: DIAGNOSTIC_INDICATOR)* |
