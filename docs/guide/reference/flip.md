<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Flip Polarity

Module id `flip` · category **Condition** · [reference index](README.md)

Mirrors a curve about a pivot: OUT = 2 x pivot - CURVE. For an SP recorded with the wrong sign convention, or any reading delivered inverted.

PIVOT_FROM:
• VALUE — mirror about PIVOT, a number you give. The only reproducible choice, and the one to use when the pivot is a physical reference (an SP shale baseline).
• MIDRANGE — mirror about (min + max) / 2 of this well's own curve.
• MEAN — mirror about this well's own mean.

MIDRANGE and MEAN are computed PER WELL, so the same run gives each well a different pivot and two wells' flipped curves are no longer on a common scale. That is often what is wanted for a quick look and is almost never what should go into a correlation — the run leaves the pivot it used in the flag curve so it is at least recoverable.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to flip | `SP` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### PIVOT

Value to mirror about (PIVOT_FROM = VALUE)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000

## Options

### OPT_PIVOT

Where the mirror line sits

- **Choices:**
  - `VALUE` — mirror about PIVOT
  - `MIDRANGE` — about (min + max) / 2 of this well's curve
  - `MEAN` — about this well's own mean
- **Default:** `VALUE`

### OPT_FLAG

Write a curve carrying the pivot actually used

- **Choices:**
  - `YES` — write the flag curve
  - `NO` — the conditioned curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Conditioned curve |
| OUT_FLAG | Pivot actually used |
