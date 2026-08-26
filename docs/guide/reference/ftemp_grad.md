<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Formation Temperature

Module id `ftemp_grad` · category **Prep** · [reference index](README.md)

GRADIENT: FTEMP = TSURF + TGRAD*depth. BHT: linear interpolation from surface temperature to bottom-hole temperature at TD_BHT.

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### TSURF *(degC)*

Surface temperature (whole well)

- **Default:** 26.7 — source: Adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 50 degC
- **One value per well** — a named-zone override of this parameter is refused.

### TGRAD *(degC/m)*

Temperature gradient (whole well)

- **Default:** 0.0474 — source: Ruling DEC-085 R-3 (2026-08-20): the house default gradient is 0.026 degF/ft - this is that value in this parameter's degC/m unit (0.026 x (5/9) / 0.3048 = 0.04739, rounded 0.0474), ending the 58 percent disagreement with precalc.TEMP_GRAD that AUDIT-2026-08-20 finding 30 measured (~33 degC apart at 2000 m, ~25 percent in Rw). Supersedes the 0.03 degC/m DEC-077 carried; starting value, refit per basin as ever; docs/takeover/DECISIONS.md
- **Accepted range:** 0.005 to 0.1 degC/m
- **One value per well** — a named-zone override of this parameter is refused.

### BHT *(degC)*

Bottom hole temperature (whole well)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 30 to 250 degC
- **One value per well** — a named-zone override of this parameter is refused.
- **Checked before the run:**
  - `bht.required_when_bht` — BHT is required when OPT_FT = BHT. Applies when: [object Object].
    Source: Absent by adjudication DEC-077 (2026-08-19): a well-specific measurement is user input, never a default (Halliburton book 3 worked example); docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters; docs/takeover/DECISIONS.md

### TD_BHT *(m)*

Depth of BHT measurement (whole well)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 100 to 10000 m
- **One value per well** — a named-zone override of this parameter is refused.
- **Checked before the run:**
  - `td_bht.required_when_bht` — TD_BHT is required when OPT_FT = BHT. Applies when: [object Object].
    Source: Absent by adjudication DEC-077 (2026-08-19): a well-specific measurement is user input, never a default (Halliburton book 3 worked example); docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters; docs/takeover/DECISIONS.md

## Options

### OPT_FT

Temperature model

- **Choices:**
  - `GRADIENT`
  - `BHT`
- **Default:** `GRADIENT`

## Output curves

| Name | Description |
|---|---|
| FTEMP | Formation temperature |
