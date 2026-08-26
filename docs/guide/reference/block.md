<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Block (Upscale)

Module id `block` · category **Frame** · [reference index](README.md)

Replaces a curve with one value per bed, held across the bed. The curve stays on the well's own depth frame, so nothing downstream has to know it was upscaled — set its draw style to Step in the curve editor, or the log view draws a gradient between two block values that the data never measured.

OPT_BEDS — what a bed is:
• INTERVAL — equal slices of INTERVAL thickness. Reproducible, and it ignores the rock: a contact falling mid-slice is averaged across.
• CLASS — each run of a constant value in the BEDS curve (FACIES, a rock type, a flag). Boundaries are where the rock changes; this is what a simulation model wants.
• ZONES — one value per marker interval. The coarsest, and what a zone-parameter table or a volumetrics summary consumes.
• AUTO — boundaries found from the curve itself, needing no other curve. They are INFERRED, and the run says so.

OPT_STAT — how a bed's value is taken. **This is a petrophysical choice, not a formatting one.**
• MEAN — right for porosity and for every volume fraction, because those add.
• GEOMETRIC — the standard estimate for PERMEABILITY in randomly heterogeneous rock.
• HARMONIC — permeability across layers in SERIES (flow perpendicular to lamination); the lowest of the three and the one a vertical barrier deserves.
• MEDIAN / MIN / MAX — order statistics, for a flag or a worst-case screen.
• MODE — the bed's commonest value, and the ONLY upscale for a class curve (FACIES, a lithology code). A class code is a name written as a number: the mean of facies 1 and facies 4 is 2.5, which is not a facies, and nothing downstream can tell. A curve declared as a class refuses every averaging statistic here for that reason.

An arithmetic upscale of a laminated sand-shale gives a permeability the rock does not have, and it is always the HIGHEST of the three, so the error never looks like a problem: 1000 mD sand with 0.01 mD shale in equal parts is 500 mD arithmetically, 0.3 mD geometrically and 0.02 mD harmonically.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to upscale | `PHIE` | yes | — |
| BEDS | Class curve defining the beds (OPT_BEDS = CLASS) | `FACIES` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### INTERVAL *(depth)*

Block thickness (OPT_BEDS = INTERVAL)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 10000 depth
- **Checked before the run:**
  - `interval.required_when_interval` — INTERVAL is required when OPT_BEDS = INTERVAL. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 frame parameters

### MIN_BED *(depth)*

Thinnest bed worth calling a bed (OPT_BEDS = AUTO)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 10000 depth
- **Checked before the run:**
  - `min_bed.required_when_auto` — MIN_BED is required when OPT_BEDS = AUTO. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 frame parameters

### SENS

AUTO: how far off the bed's mean is a new bed, in noise units

- **Default:** 2 — source: Two-noise-units convention for change detection (same family as the Hampel K in condition.rs), NOT a field calibration; ruled a shipping starting value by adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md
- **Accepted range:** 0.5 to 20

## Options

### OPT_BEDS

What counts as one bed

- **Choices:**
  - `INTERVAL` — equal slices of INTERVAL thickness
  - `CLASS` — each run of a constant value in the BEDS curve
  - `ZONES` — one value per marker interval
  - `AUTO` — boundaries found from the curve itself (inferred)
- **Default:** `INTERVAL`

### OPT_STAT

How a bed's value is taken

- **Choices:**
  - `MEAN` — arithmetic; right for porosity and volume fractions
  - `GEOMETRIC` — the usual estimate for permeability
  - `HARMONIC` — permeability across layers in series
  - `MEDIAN` — the middle sample of the bed
  - `MIN` — the lowest sample of the bed
  - `MAX` — the highest sample of the bed
  - `MODE` — the bed's commonest value; the only upscale for a class curve
- **Default:** `MEAN`

### OPT_FLAG

Write the bed number each sample fell in

- **Choices:**
  - `YES` — write the bed-index curve
  - `NO` — the blocked curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Blocked curve |
| OUT_BED | Bed index |
