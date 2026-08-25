<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Splice Curves

Module id `splice` · category **Prep** · [reference index](README.md)

SPLICED = TOP_CURVE above SPLICE_DEPTH, BOT_CURVE at and below it — the classic run-to-run splice. Written as <TOP_CURVE>_SPL; inputs are never modified.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| TOP_CURVE | Curve used above the splice depth | `GR` | yes | — |
| BOT_CURVE | Curve used below the splice depth | `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### SPLICE_DEPTH *(m)*

Depth where BOT_CURVE takes over

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 20000 m

## Output curves

| Name | Description |
|---|---|
| SPLICED | Spliced curve |
