<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Smooth

Module id `smooth` · category **Condition** · [reference index](README.md)

Averages a curve over a WINDOW stated as a THICKNESS.

**Despike first.** A least-squares smoother fits whatever is in the window, so over an un-despiked curve it fits the spike — the spike is not removed, it is spread over the window and made to look like rock.

METHOD:
• MEAN — arithmetic mean of the live samples in the window.
• MEDIAN — window median; keeps a step edge where a mean would ramp across it.
• SAVGOL — local quadratic least-squares fit evaluated at the sample. Fitted on the real (depth, value) pairs rather than with the textbook fixed coefficients, which assume even sampling and are wrong on an irregular frame.

A MISSING sample stays MISSING and no window is bridged: smoothing does not fill gaps, because a filled sample is a claim about rock nobody logged. Use Fill Gaps, which marks what it invented.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to smooth | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### WINDOW *(depth)*

Smoothing window (thickness, centred)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 depth
- Competing shipped values exist for this parameter across installed tools (topic `conditioning_window`); the pane lists them with sources at the point of choice.

## Options

### OPT_METHOD

How the window is averaged

- **Choices:**
  - `MEAN` — arithmetic mean over the window
  - `MEDIAN` — window median, keeps step edges
  - `SAVGOL` — local quadratic fit (Savitzky-Golay)
- **Default:** `MEAN`

### OPT_FLAG

Write a flag curve marking every sample the smoother changed

- **Choices:**
  - `YES` — write the flag curve
  - `NO` — the conditioned curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Conditioned curve |
| OUT_FLAG | Changed-sample flag *(flag: DIAGNOSTIC_INDICATOR)* |
