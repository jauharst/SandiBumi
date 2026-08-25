<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Brittleness index (elastic / mineralogical)

Module id `brittleness` · category **Unconventional** · [reference index](README.md)

Brittleness index (0 ductile .. 1 brittle) two ways. METHOD=elastic: dynamic Young's modulus and Poisson's ratio from DT, DTS, RHOB (G=ρ·Vs², K=ρ·(Vp²−4/3·Vs²), ν=(3K−2G)/(2(3K+G)), E=9KG/(3K+G), with Vp,Vs in km/s = 304.8/slowness, moduli in GPa, E→Mpsi), then Rickman et al. 2008 BI=(E_norm+ν_norm)/2 with E normalized over E_LO..E_HI Mpsi and ν over NU_LO..NU_HI (Barnett defaults 1..8 and 0.4..0.15 — recalibrate per basin). METHOD=mineral_jarvie: Jarvie 2007 BI=Qz/(Qz+carbonate+clay). METHOD=mineral_wanggale: Wang & Gale 2009 BI=(Qz+Dol)/(Qz+Dol+calcite+clay+organic) — dolomite counts brittle. Mineral volumes come from a SandiMin run (VOL_*); a missing mineral is treated as absent. Elastic E,ν are DYNAMIC (apply a static correlation before geomechanics). Cite: Rickman et al. 2008 (SPE 115258); Jarvie et al. 2007; Wang & Gale 2009. See docs/ref_unconventional.md §4.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| DT | Compressional Δt (elastic) | `DT` | no | — |
| DTS | Shear Δt (elastic) | `DTS` | no | — |
| RHOB | Bulk density (elastic) | `RHOB` | no | — |
| VQTZ | Quartz volume (mineral) | `VOL_QUARTZ` | no | — |
| VCARB | Calcite volume (mineral) | `VOL_CALCITE` | no | — |
| VDOL | Dolomite volume (mineral) | `VOL_DOLOMITE` | no | — |
| VCLAY | Clay volume (mineral) | `VCL` | no | accepts quantity kind: VCL |
| VORG | Organic / kerogen volume (Wang-Gale) | `VKER` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### E_LO *(Mpsi)*

Young's modulus at BI=0 (ductile)

- **Default:** 1 — source: IP equation and cited publication; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 0 to 20 Mpsi

### E_HI *(Mpsi)*

Young's modulus at BI=1 (brittle)

- **Default:** 8 — source: IP equation and Rickman et al. SPE 115258; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 0 to 20 Mpsi

### NU_LO *(-)*

Poisson's ratio at BI=0 (ductile)

- **Default:** 0.4 — source: IP equation and Rickman et al. SPE 115258; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 0 to 0.5 -

### NU_HI *(-)*

Poisson's ratio at BI=1 (brittle)

- **Default:** 0.15 — source: IP equation and Rickman et al. SPE 115258; docs/PRD_v2/19_toc-unconventional.md §5
- **Accepted range:** 0 to 0.5 -

## Options

### METHOD

Brittleness basis

- **Choices:**
  - `elastic`
  - `mineral_jarvie`
  - `mineral_wanggale`
- **Default:** `elastic`

## Output curves

| Name | Description |
|---|---|
| BI | Brittleness index (0 ductile .. 1 brittle) |
| YME | Dynamic Young's modulus (elastic) |
| PR | Dynamic Poisson's ratio (elastic) |
