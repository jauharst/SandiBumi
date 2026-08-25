<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Permeability — Coates

Module id `perm_coates` · category **Permeability** · [reference index](README.md)

PERM = (C * PHIE^2 * (1 - SWE_IRR)/SWE_IRR)^2, mD. This free-fluid form is Coates & Denoo (1981), The Producibility Answer Product, Schlumberger Technical Review 29(2). It is NOT Coates & Dumanoir (1974, The Log Analyst 15(1), 17-31; SPWLA 14th, Paper R), which replaces both m and n with a common exponent driven by porosity, resistivity at Swirr, hydrocarbon density and rock class - a different and much heavier model this module does not implement. CONST_COATES IS SCALE-DEPENDENT AND PUBLISHED VALUES ARE NOT INTERCHANGEABLE: 100 in this fractional form is the same rock as 10 in the NMR Timur-Coates (phi/C)^4 form with porosity in percent, and Schlumberger's K-4 chart states 70. Those are unit conventions, not disagreements about rock - check which one a quoted C belongs to before entering it.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHIE | Limited effective porosity | `PHIE` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### CONST_COATES

Coates constant

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 1000

### SWE_IRR *(v/v)*

Irreducible effective water saturation

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.01 to 0.8 v/v
- Competing shipped values exist for this parameter across installed tools (topic `irreducible_swe`); the pane lists them with sources at the point of choice.

## Output curves

| Name | Description |
|---|---|
| PERM_COATES | Permeability from Coates |
| PERM | Working permeability |
