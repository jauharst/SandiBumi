<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Bed Detect

Module id `bed_detect` · category **Frame** · [reference index](README.md)

Writes the bed number each sample falls in, found from the curve's own steps — the same segmentation Block's AUTO mode uses, exposed on its own so the beds can be LOOKED AT on a log before anything is averaged over them.

That order matters: over-segmentation is what a step-finder gets wrong when it gets anything wrong, and a blocked curve computed from beds nobody checked looks perfectly reasonable. Put the bed curve in a track as class blocks, judge it against the log, then run Block with OPT_BEDS = CLASS pointing at it.

A sample opens a new bed when it sits further from the running bed mean than SENS times the curve's own noise AND the bed already spans MIN_BED. The noise is measured from the curve's own sample-to-sample differences, so it is the curve's noise rather than how much it varies across the well.

MIN_BED has no default: the thinnest thing worth calling a bed is a property of the tool and the rock.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to segment | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### MIN_BED *(depth)*

Thinnest bed worth calling a bed

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 10000 depth

### SENS

How far off the bed's mean is a new bed, in noise units

- **Default:** 2 — source: Two-noise-units convention for change detection (same family as the Hampel K in condition.rs), NOT a field calibration; ruled a shipping starting value by adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md
- **Accepted range:** 0.5 to 20

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Bed index |
