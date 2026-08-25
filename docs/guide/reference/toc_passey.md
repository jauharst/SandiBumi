<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# TOC — Passey ΔlogR + Schmoker

Module id `toc_passey` · category **Unconventional** · [reference index](README.md)

Total organic carbon from the Passey (1990) ΔlogR overlay — the separation between deep resistivity and a baselined porosity curve — converted to TOC with the maturity term 10^(2.297−0.1688·LOM). ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base) [sonic overlay] or −2.5·(RHOB−RHOB_base) [density overlay]. Baselines are picked on a non-source, clay-rich interval (params, per-zone overridable) where the two curves overlie; where ΔlogR<0 the rock is non-source and TOC floors to the background value. Also writes the Schmoker-Hester (1983) density-TOC 154.497/RHOB−57.261 as an independent cross-check whenever RHOB is present. TOC in wt%. LOM 6..12 (Passey is calibrated to LOM≤12). Cite: Passey, Creaney, Kulla, Moretti & Stroud 1990, AAPG Bull. 74(12); Schmoker & Hester 1983, AAPG Bull. 67(12). See docs/ref_unconventional.md §1. (Neutron overlay deferred — sign convention needs core verification.)

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RES | Deep resistivity | `RT` | yes | — |
| DT | Sonic Δt (sonic overlay) | `DT` | no | — |
| RHOB | Bulk density (density overlay + Schmoker) | `RHOB` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### R_BASE *(ohm.m)*

Baseline resistivity (non-source interval)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 100000 ohm.m

### DT_BASE *(us/ft)*

Baseline sonic Δt (sonic overlay)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 40 to 200 us/ft
- **Checked before the run:**
  - `dt_base.required_when_sonic` — DT_BASE is required when OVERLAY = sonic. Applies when: [object Object].
    Source: docs/PRD_v2/19_toc-unconventional.md §5 resistivity/porosity baselines

### RHOB_BASE *(g/cc)*

Baseline bulk density (density overlay)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1.5 to 3.5 g/cc
- **Checked before the run:**
  - `rhob_base.required_when_density` — RHOB_BASE is required when OVERLAY = density. Applies when: [object Object].
    Source: docs/PRD_v2/19_toc-unconventional.md §5 resistivity/porosity baselines

### LOM *(-)*

Level of organic maturity (Hood scale, 6..12)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 6 to 12 -

### TOC_BG *(wt%)*

Background TOC of the baseline rock

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 10 wt%

## Options

### OVERLAY

Porosity curve paired with resistivity for ΔlogR

- **Choices:**
  - `sonic`
  - `density`
- **Default:** `sonic`

## Output curves

| Name | Description |
|---|---|
| DLOGR | Passey resistivity–porosity separation (log10 cycles) |
| TOC | Total organic carbon (Passey ΔlogR) |
| TOC_SCHMOKER | Density-TOC cross-check (Schmoker-Hester 1983) |
