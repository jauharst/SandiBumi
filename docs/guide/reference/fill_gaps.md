<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Fill Gaps

Module id `fill_gaps` · category **Condition** · [reference index](README.md)

Fills holes in a curve that are no wider than MAX_GAP, and marks every sample it invented in <OUT>_FILL.

A filled sample is not a measurement. That is the whole reason for the flag curve (Jauhar, 2026-08-05): without it a filled value is indistinguishable from a logged one in a crossplot, a histogram, a net count or a report, and the person reading the number is not the person who chose the limit. Mask on <OUT>_FILL to take them back out of any run.

**A gap open at one end is never filled.** A hole at the top or the bottom of the curve has live data on one side only, so filling it is extrapolation — inventing rock past where the tool stopped. Only a gap bounded above AND below is a candidate.

METHOD:
• LINEAR — a straight line between the live samples either side.
• HOLD — the last live value carried down. The honest choice for a curve that is blocky by nature (a facies code, a flag, a zone constant), where a ramp would draw a transition the rock does not have.

MAX_GAP has no default: how far it is defensible to interpolate depends on why the data is missing and on the bed thickness, and no single value is right twice.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to fill | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### MAX_GAP *(depth)*

Widest hole that may be filled (thickness)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 10000 depth

## Options

### OPT_METHOD

How the hole is filled

- **Choices:**
  - `LINEAR` — straight line between the live samples either side
  - `HOLD` — carry the last live value down
- **Default:** `LINEAR`

### OPT_FLAG

Write a flag curve marking every invented sample

- **Choices:**
  - `YES` — write the flag curve
  - `NO` — the conditioned curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Conditioned curve |
| OUT_FLAG | Filled-sample flag *(flag: DIAGNOSTIC_INDICATOR)* |
| OUT_ORIG | Original values at every changed sample |
