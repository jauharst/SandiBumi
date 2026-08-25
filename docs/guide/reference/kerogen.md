<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Kerogen volume + OM-corrected porosity

Module id `kerogen` · category **Unconventional** · [reference index](README.md)

Converts TOC (weight %) to kerogen VOLUME and corrects total porosity for the organic matter that low-density kerogen inflates on the density log. TOM = k_toc2om·TOC/100 (organic-matter weight fraction; k_toc2om≈1.2 accounts for the H/O/N/S beyond carbon), then VKER = TOM·RHOB/ρ_kero (kerogen volume fraction of the BULK rock — the Passey/Vernik bulk-density conversion, directly comparable to SandiMin VOL_KEROGEN). PHIT_OMC = PHIT − VKER removes kerogen's apparent-porosity contribution (ρ_kero≈ρ_fluid, first order). ρ_kero default 1.10 g/cc matches the SandiMin Kerogen mineral (sandimin.rs), so VKER reconciles with VOL_KEROGEN (IP's RHOTOC seed is 1.25 — override if you prefer it). Cite: Passey et al. 2010 (SPE 131350); Vernik & Nur 1992. See docs/ref_unconventional.md §2.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| TOC | Total organic carbon | `TOC` | yes | — |
| RHOB | Bulk density | `RHOB` | yes | — |
| PHIT | Total porosity to OM-correct (optional) | `PHIT` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_KERO *(g/cc)*

Kerogen (organic-matter) grain density

- **Default:** 1.1 — source: Like-for-like IP endpoint and matching shipped mineral endpoint; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 0.9 to 1.6 g/cc

### K_TOC2OM *(-)*

TOC→organic-matter factor (1.2 immature .. 1.35 mature)

- **Default:** 1.2 — source: Techlog and two Geolog modules; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 1 to 1.6 -

## Output curves

| Name | Description |
|---|---|
| TOM | Organic-matter weight fraction |
| VKER | Kerogen volume fraction (bulk) |
| PHIT_OMC | OM-corrected total porosity |
