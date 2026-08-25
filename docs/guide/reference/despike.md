<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Despike

Module id `despike` · category **Condition** · [reference index](README.md)

Replaces samples that stand off their neighbours with the local median. WINDOW is a THICKNESS, not a sample count, so it means the same amount of rock whatever the sampling.

**Set WINDOW narrower than the thinnest bed you intend to keep.** A spike is a tool artefact and a thin bed is rock; the only thing separating them here is thickness against the window, so a bed no thicker than the window is indistinguishable from a spike and will be flattened. Even a bed comfortably wider than the window loses its top and bottom sample to the shoulder, where the window straddles the contact.

METHOD:
• HAMPEL — replace when the sample is more than K robust deviations from the window median (the deviation is the cited Gaussian consistency constant x MAD, so one K reads the same on GR, RHOB, NPHI and RT). Needs a WINDOW covering at least five samples, and the run refuses a narrower one: below that the spread being measured against is set by the very sample under test. Where more than half the window is identical — a quiet interval, a coarsely quantized curve, a tool on its rail — the MAD is zero and the mean deviation is used instead, so a lone spike in flat rock is still found.
• ABS — replace when it is more than THRESH away, in the curve's own units.
• MEDIAN — replace every sample with the window median, no test. Changes samples that were fine, which is why the flag curve is near-useless for this method.
• RATE — replace when the change from the previous live sample exceeds MAX_RATE per depth unit. Catches the step (a stuck tool, a bad splice) that a median window can miss when several bad samples sit together.

WINDOW has no default: what counts as a spike is a property of the tool, the sampling and the rock, and no one number is right in two basins.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| CURVE | Curve to despike | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### WINDOW *(depth)*

Filter window (thickness, centred)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 depth
- Competing shipped values exist for this parameter across installed tools (topic `conditioning_window`); the pane lists them with sources at the point of choice.

### K

HAMPEL: deviations from the median before a sample is a spike

- **Default:** 3 — source: Ordinary three-deviation convention (same family as Tukey 1.5 x IQR in distribution.rs), NOT a field calibration; ruled a shipping starting value by Jauhar adjudication DEC-077 (2026-08-19); docs/takeover/DECISIONS.md
- **Accepted range:** 0.5 to 20

### THRESH

ABS: distance from the median, in the curve's units

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000000000
- **Checked before the run:**
  - `thresh.required_when_abs` — THRESH is required when OPT_METHOD = ABS. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 conditioning parameters

### MAX_RATE

RATE: largest honest change per depth unit

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000000000
- **Checked before the run:**
  - `max_rate.required_when_rate` — MAX_RATE is required when OPT_METHOD = RATE. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5.3 conditioning parameters

## Options

### OPT_METHOD

How a spike is told from rock

- **Choices:**
  - `HAMPEL` — off its neighbours vs the curve's own noise (K x MAD)
  - `ABS` — off the local median by more than THRESH
  - `MEDIAN` — plain median filter, every sample replaced
  - `RATE` — change per depth unit above MAX_RATE
- **Default:** `HAMPEL`

### OPT_FLAG

Write a flag curve marking every replaced sample

- **Choices:**
  - `YES` — write the flag curve
  - `NO` — the conditioned curve only
- **Default:** `YES`

## Output curves

| Name | Description |
|---|---|
| OUT_CURVE | Conditioned curve |
| OUT_FLAG | Replaced-sample flag *(flag: DIAGNOSTIC_INDICATOR)* |
| OUT_ORIG | Original values at every changed sample |
| OUT_FBSCALE | Hampel scale diagnostic: 1 = judged on the mean-deviation fallback scale (zero-MAD window), 0 = judged on the true MAD, MISSING where no judgement was made *(flag: DIAGNOSTIC_INDICATOR)* |
