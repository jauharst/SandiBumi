<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Porosity from Sonic

Module id `phi_son` · category **Porosity** · [reference index](README.md)

Sonic porosity, three transforms each named for what it computes (SB-POR-014). WYLLIE: PHIT = (DT - DT_MA)/(DT_FL - DT_MA), shale-corrected SUBTRACTIVELY for PHIE - the one convention both vendors agree on exactly (SB-POR-013). RHG80: the genuine three-segment Raymer-Hunt-Gardner 1980 transform (DEC-017; primary source: Raymer, Hunt & Gardner, SPWLA 21st Annual Logging Symposium 1980, paper P, copy in Jauhar's library; constants paper-verified under DEC-079): phi < 37% inverts V = (1-phi)^2*Vma + phi*Vf as the quadratic root, phi > 47% inverts the fluid-suspension form (needs RHO_MA/RHO_FL, the paper's own density pairings), 37-47% is the paper's dt-linear interpolation. FIELD_OBSERVED: PHI = CFO*(DT - DT_MA)/DT with a CITED coefficient (Geolog CFO default 0.67, normal range 0.625-0.70; Techlog ships 0.625) - the transform the pre-DEC-017 build mislabelled RHG; the old option value RHG no longer resolves. Non-Wyllie methods use the NORMALISED shale convention per Geolog's EXECUTED code (SB-POR-013/-015, F4): dtsr = (DT - VSH*DT_SH)/(1 - VSH) floored at DT_MA (the Jul-1997 floor, F5), then PHIE = transform(dtsr)*(1 - VSH). OPT_CP=ON applies the Wyllie lack-of-compaction correction (Cp = DT_SH/100) to WYLLIE only: it requires DT_SH >= 100 us/ft (Cp >= 1) and the run is refused otherwise (DEC-012, SB-POR-017) - below 100 the division would inflate porosity, the opposite of its purpose. RHG80 and FIELD_OBSERVED are self-compacting and never Cp-corrected. DT_MA must be strictly below DT_SH (DEC-063) - an inverted pair turns the shale subtraction into an addition and is refused before the run. PHIT_SON/PHIE_SON are the bare unlimited expressions; PHIT/PHIE carry the unit-interval and ordering clamps.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| BADHOLE | Bad-hole flag (flagged samples excluded, exclusion recorded) | `BADHOLE` | no | — |
| GAS_FLAG | Gas-crossover flag from condflag (provenance record only - never a correction) | `XOVER_FLAG` | no | — |
| COAL_FLAG | Coal flag from condflag (flagged samples blanked - coal apparent porosity is never rock porosity) | `COAL_FLAG` | no | — |
| TIGHT_FLAG | Tight flag from condflag (indicator only - never alters an output) | `TIGHT_FLAG` | no | — |
| COND_FLAG | Conditioning flag from condflag (indicator only - never alters an output) | `COND_FLAG` | no | — |
| DT | Sonic transit time log | `DT` | yes | — |
| VSH | Limited volume of shale | `VSH` | yes | accepts quantity kind: VSH |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### DT_MA *(us/ft)*

Matrix transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 40 to 70 us/ft
- Competing shipped values exist for this parameter across installed tools (topic `matrix_transit_time`); the pane lists them with sources at the point of choice.
- **Checked before the run:**
  - `phi_son.endpoint_order` — The matrix transit time must be strictly below the shale transit time. Must be strictly below `DT_SH`.
    Source: Jauhar ruling DEC-063 (2026-08-18): DT MA should always be lower than DT SH; an inverted pair turns the shale subtraction into an addition (SB-POR-009 ordering)

### DT_FL *(us/ft)*

Fluid transit time

- **Default:** 189 — source: IP swparameters.htm Sonic water Default 189; Geolog phi_son.info DT_FL 620 us/m; docs/PRD_v2/11_porosity.md §5.2
- **Accepted range:** 150 to 220 us/ft
- Competing shipped values exist for this parameter across installed tools (topic `fluid_transit_time`); the pane lists them with sources at the point of choice.

### DT_SH *(us/ft)*

Shale transit time

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 60 to 150 us/ft
- Competing shipped values exist for this parameter across installed tools (topic `shale_transit_time`); the pane lists them with sources at the point of choice.

### CFO

Field-observed coefficient (PHI = CFO*(dt - DT_MA)/dt)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.625 to 0.7
- Competing shipped values exist for this parameter across installed tools (topic `field_observed_coefficient`); the pane lists them with sources at the point of choice.
- **Checked before the run:**
  - `cfo.required_when_field_observed` — CFO is required when OPT_SON = FIELD_OBSERVED. Applies when: [object Object].
    Source: SB-POR-014 (docs/PRD_v2/11_porosity.md section 5.2): Geolog phi_son.info CFO DEFAULT 0.67 with doc range 0.625-0.70 (T1); Techlog porosity-from-sonic File coefficient 0.625 (T3); DEC-017 requires a cited coefficient

### RHO_MA *(g/cc)*

Matrix density (RHG80 suspension segment)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `matrix_density`); the pane lists them with sources at the point of choice.
- **Checked before the run:**
  - `rho_ma.required_when_rhg80` — RHO_MA is required when OPT_SON = RHG80. Applies when: [object Object].
    Source: RHG 1980 paper lithology pairings (docs/research_2026-08/plt024_route2_sources_2026-08-19.md; DEC-079)

### RHO_FL *(g/cc)*

Fluid density (RHG80 suspension segment)

- **Default:** 1 — source: RHG 1980: water at rho 1.0, Vf 5,300 ft/s throughout the paper's figures (DEC-079 verified constants)
- **Accepted range:** 0.5 to 1.5 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.

## Options

### OPT_SON

Sonic porosity method

- **Choices:**
  - `WYLLIE`
  - `RHG80`
  - `FIELD_OBSERVED`
- **Default:** `WYLLIE`

### OPT_CP

Wyllie lack-of-compaction correction (Cp = DT_SH/100)

- **Choices:**
  - `OFF`
  - `ON`
- **Default:** `OFF`

## Output curves

| Name | Description |
|---|---|
| PHIT_SON | Total porosity from sonic (unlimited) |
| PHIE_SON | Effective porosity from sonic (unlimited) |
| PHIE | Limited effective porosity |
| PHIT | Limited total porosity |
