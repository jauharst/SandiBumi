<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Porosity Ceiling (φmax)

Module id `phimax` · category **Porosity** · [reference index](README.md)

Caps an input porosity at a maximum ceiling — the field's compaction-controlled upper limit (the crossplot 'max core porosity' line). The ceiling is CONSTANT (PHIMAX0, per-zone overridable), or a TVDSS compaction TREND: LINEAR (φmax = PHIMAX0 − PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000) or ATHY exponential (φmax = PHIMAX0·exp(−ATHY_K·(TVDSS − TVDSS_REF)/1000)). TVDSS is a POSITIVE-downward depth-below-datum curve (same convention as precalc), so DEEPER = larger TVDSS = lower ceiling; all four params are per-zone overridable. No TVDSS curve → measured DEPTH is used instead (fine for near-vertical wells; the trend then reads against MD). Writes <PHI>_CAP = min(PHI, φmax) preserving MISSING, and the ceiling curve <PHI>_MAX for QC overlay; the input porosity is never modified. Constant mode ignores TVDSS.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHI | Porosity to cap | `PHIE` | yes | — |
| TVDSS | True vertical depth subsea (trend modes) | `TVDSS` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### PHIMAX0 *(v/v)*

φmax at TVDSS_REF (also the CONSTANT cap value)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1 v/v

### TVDSS_REF *(depth)*

Reference TVDSS where φmax = PHIMAX0

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -30000 to 30000 depth
- **Checked before the run:**
  - `tvdss_ref.required_when_linear` — TVDSS_REF is required when MODE = linear. Applies when: [object Object].
    Source: docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters
  - `tvdss_ref.required_when_athy` — TVDSS_REF is required when MODE = athy. Applies when: [object Object].
    Source: docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters

### PHIMAX_GRAD *(v/v per 1000)*

LINEAR: φmax lost per 1000 TVDSS units deeper

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1 to 1 v/v per 1000
- **Checked before the run:**
  - `phimax_grad.required_when_linear` — PHIMAX_GRAD is required when MODE = linear. Applies when: [object Object].
    Source: docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters

### ATHY_K *(1/1000)*

ATHY: compaction coefficient per 1000 TVDSS units

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 5 1/1000
- **Checked before the run:**
  - `athy_k.required_when_athy` — ATHY_K is required when MODE = athy. Applies when: [object Object].
    Source: docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters

## Options

### MODE

Ceiling model

- **Choices:**
  - `constant`
  - `linear`
  - `athy`
- **Default:** `linear`

## Output curves

| Name | Description |
|---|---|
| PHI_CAP | Capped porosity |
| PHI_MAX | φmax ceiling curve |
