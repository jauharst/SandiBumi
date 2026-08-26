<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Bad-Hole QC Flag

Module id `badhole` · category **Prep** · [reference index](README.md)

BADHOLE = 1 where the borehole departs from gauge or the density correction is large enough to distrust the porosity logs: |DRHO| > DRHO_MAX, or |CALI - bit size| > DCAL_MAX. Bit size comes from the BS curve where present, or the interpreter's optional BS_INPUT; no value is substituted when both are absent, so only DRHO can be evaluated. The flag is 0 in good hole and MISSING where no QC criterion can be evaluated. The two BADHOLE_*_EVALUATED companions record criterion availability with 1 = evaluated and 0 = unavailable; the BADHOLE_CALI / BADHOLE_DRHO_POS / BADHOLE_DRHO_NEG cause flags record which criterion fired, with the DRHO sign carried natively (DEC-060). Feed BADHOLE to any module run as a mask so flagged intervals go missing instead of polluting results.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| DRHO | Density correction log | `DRHO` | no | — |
| CALI | Caliper log | `CALI` | no | — |
| BS | Bit size log | `BS` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### DRHO_MAX

Max acceptable density correction (starting value in g/cc)

- **Default:** 0.05 — source: Adjudication DEC-077 (2026-08-19): 0.05 g/cc multi-basin starting value per DEC-059, ruled with the chapter's own note in view that it matches none of the seven tabulated precedent values; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 0.5

### DCAL_MAX *(in)*

Max acceptable absolute caliper departure from bit size

- **Default:** 1 — source: Adjudication DEC-077 (2026-08-19): 1.0 in multi-basin starting value per DEC-059, ruled with the chapter's own note in view that it is half the 2 in value used by every delivered study; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 12 in

### BS_INPUT *(in)*

Optional explicit bit size when the BS curve is absent — blank means unavailable

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.

## Options

### DRHO_MAX_UNIT

Unit of the density-correction threshold; required when DRHO is present

- **Choices:**
  - `g/cc` — g/cc
  - `kg/m3` — kg/m3

## Output curves

| Name | Description |
|---|---|
| BADHOLE | Bad-hole flag (1 = bad, 0 = good) *(flag: EXCLUSION_MASK)* |
| BADHOLE_CALI | Caliper departure fired (DEC-060 bad-hole cause group; MISSING where the criterion was not evaluable) *(flag: DIAGNOSTIC_INDICATOR)* |
| BADHOLE_DRHO_POS | Density correction fired with positive sign (DEC-060 bad-hole cause group; MISSING where DRHO was not evaluable) *(flag: DIAGNOSTIC_INDICATOR)* |
| BADHOLE_DRHO_NEG | Density correction fired with negative sign (DEC-060 bad-hole cause group; MISSING where DRHO was not evaluable) *(flag: DIAGNOSTIC_INDICATOR)* |
| BADHOLE_CALI_EVALUATED | Caliper criterion availability (1 = evaluated, 0 = unavailable) *(flag: DIAGNOSTIC_INDICATOR)* |
| BADHOLE_DRHO_EVALUATED | Density-correction criterion availability (1 = evaluated, 0 = unavailable) *(flag: DIAGNOSTIC_INDICATOR)* |
