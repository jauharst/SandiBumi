<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Permeability — Wyllie-Rose

Module id `perm_wyllie_rose` · category **Permeability** · [reference index](README.md)

PERM = (C * PHIE^D / SWE_IRR^E)^2, mD. Defaults per method: TIMUR C=100 D=2.25 E=1; MORRIS_BIGGS_OIL C=250 D=3 E=1; MORRIS_BIGGS_GAS C=79 D=3 E=1; TIXIER C=250 D=3 E=1. The generalized form is Wyllie & Rose (1950), Trans. AIME 189, 105-118, who replaced Carman-Kozeny's specific-surface term with irreducible water saturation; their own shape factor is 2.0-2.5, suggested as 2.25. They warned the result carries only order-of-magnitude significance, and that warning is inherited by every constant set below. TWO OF THE FOUR CARRY A NAME THEIR AUTHOR NEVER ATTACHED TO THEM. TIMUR here is the Schlumberger Chart K-3 curve, not Timur (1968), whose published relation is k = 0.136*phi^4.4/Swi^2 in PERCENT = 8581*phi^4.4/Swi^2 in fractions, i.e. C=92.63 D=2.2 in this squared form - close in good rock, about 14% low in tight rock, where a cutoff is decided. TIXIER is a post-1950 simplification of Wyllie-Rose, not Tixier (1949), which is a resistivity-gradient method; that is why TIXIER and MORRIS_BIGGS_OIL are byte-identical here - same lineage, same oil constant, arrived at twice. The 250/79 pair is attributed to Morris & Biggs (1967, SPWLA 8th, Paper X) by IP and Techlog and to Wyllie & Rose (1950) elsewhere; both authors were at Schlumberger and the 1967 paper is paywalled, so the attribution is UNRESOLVED. Lineage review: Balan, Mohaghegh & Ameri, SPE 30978 (1995).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHIE | Limited effective porosity | `PHIE` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### SWE_IRR *(v/v)*

Irreducible effective water saturation

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.01 to 0.8 v/v
- Competing shipped values exist for this parameter across installed tools (topic `irreducible_swe`); the pane lists them with sources at the point of choice.

## Options

### OPT_WR

Wyllie-Rose variant

- **Choices:**
  - `TIMUR` — TIMUR - Schlumberger Chart K-3, C=100 D=2.25 (not Timur 1968)
  - `MORRIS_BIGGS_OIL` — MORRIS_BIGGS_OIL - Wyllie-Rose oil constant, C=250 D=3
  - `MORRIS_BIGGS_GAS` — MORRIS_BIGGS_GAS - Wyllie-Rose gas constant, C=79 D=3
  - `TIXIER` — TIXIER - Wyllie-Rose oil constant again (not Tixier 1949)
- **Default:** `TIMUR`

## Output curves

| Name | Description |
|---|---|
| PERM_WR | Permeability from Wyllie-Rose |
| PERM | Working permeability |
