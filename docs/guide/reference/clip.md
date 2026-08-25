<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Clip

Module id `clip` · category **Condition** · [reference index](README.md)

Holds a curve inside a range. MIN and MAX are in the curve's own units and either may be left EMPTY, which is a statement that the curve is unbounded on that side rather than an omission.

ACTION:
• BLANK — a sample outside the range becomes MISSING. The right answer when the range is a validity check: a resistivity of 1e6 is not a very resistive rock, it is a reading the tool could not make, and pinning it to the bound would leave a real number where there is no measurement.
• CLAMP — a sample outside the range is pulled to the bound. Only defensible when the excursion is a small arithmetic overshoot of a known physical limit, the way PHIE is floored at 0.001 rather than blanked.

BLANK is the default because it is the one that cannot manufacture a measurement.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to clip | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### MIN

Lowest honest value — blank for no lower bound

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000

### MAX

Highest honest value — blank for no upper bound

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -1000000000 to 1000000000

## Options

### OPT_ACTION

What happens to a sample outside the range

- **Choices:**
  - `BLANK` — outside the range becomes MISSING
  - `CLAMP` — outside the range is pulled to the bound
- **Default:** `BLANK`

### OPT_FLAG

Write a flag curve marking every sample outside the range

- **Choices:**
  - `YES` — write the flag curve
  - `NO` — the conditioned curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Conditioned curve |
| OUT_FLAG | Out-of-range flag *(flag: DIAGNOSTIC_INDICATOR)* |
| OUT_ORIG | Original values at every changed sample |
