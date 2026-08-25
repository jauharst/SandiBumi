<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# VSH from Gamma Ray

Module id `vsh_gr` · category **VSH** · [reference index](README.md)

VSH_GR = (GR - GR_MA) / (GR_SH - GR_MA), with optional non-linear corrections (Stieber, Larionov, Clavier). VSH is the result limited to 0–1. The three Stieber forms and Clavier bound the gamma-ray INDEX just short of their own singularity (a pole, or a negative radicand) before transforming it, because past that point they return a negative shale volume or nothing at all; the run reports which branch bounded it. Larionov and the linear form have no singularity and bound nothing. VSH_GR itself stays unlimited either way.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| GR | Gamma ray log | `GR_COR` → `GR_EC` → `GR` | yes | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### GR_MA *(gAPI)*

Gamma ray matrix (clean)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 200 gAPI
- Competing shipped values exist for this parameter across installed tools (topic `gr_clean_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP derives endpoints by pooling a Percentile Group, pre-clipping at 0%/98%, computing a selected percentile and linearly extrapolating. Its clay percentile is 130%; its clean percentile is unstated. Techlog offers 5%/95%; P3/P97 is an optional named house preset. Treat these as alternative procedures, not a generic endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; IP clayparameters.htm (57, 59, 60); Techlog VSH single-log pages; docs/workflow_standards.md
- **Checked before the run:**
  - `vsh_gr.gr_ma_range` — The clean gamma-ray endpoint must remain inside the source manifest range. Range: 0–200 gAPI.
    Source: docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.info L48-L49
  - `vsh_gr.endpoint_order` — The clean gamma-ray endpoint must be strictly below the shale endpoint. Must be strictly below `GR_SH`.
    Source: docs/PRD_v2/10_clay-volume.md §3.3 and SB-CLY-001; Geolog vsh_gr.lls L99-L102

### GR_SH *(gAPI)*

Gamma ray shale

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1000 gAPI
- Competing shipped values exist for this parameter across installed tools (topic `gr_shale_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP derives endpoints by pooling a Percentile Group, pre-clipping at 0%/98%, computing a selected percentile and linearly extrapolating. Its clay percentile is 130%; its clean percentile is unstated. Techlog offers 5%/95%; P3/P97 is an optional named house preset. Treat these as alternative procedures, not a generic endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; IP clayparameters.htm (57, 59, 60); Techlog VSH single-log pages; docs/workflow_standards.md
- **Checked before the run:**
  - `vsh_gr.gr_sh_range` — The shale gamma-ray endpoint must remain inside the source manifest range. Range: 0–1000 gAPI.
    Source: docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.info L48-L49

## Options

### OPT_GR

VSH from gamma ray method

- **Choices:**
  - `LINEAR` — VSH = IGR
  - `STIEBER1` — Stieber, IGR/(3−2·IGR)
  - `STIEBER2` — Stieber, IGR/(2−IGR)
  - `STIEBER3` — Stieber, IGR/(4−3·IGR)
  - `LARINOV1_NORM` — Larionov, Mesozoic and older (exact, reaches 1.0)
  - `LARINOV2_NORM` — Larionov, Tertiary / unconsolidated (exact, reaches 1.0)
  - `LARINOV1` — Larionov, Mesozoic and older (published 0.33 — parity only, 0.990 at IGR 1)
  - `LARINOV2` — Larionov, Tertiary / unconsolidated (published 0.083 — parity only, 0.996 at IGR 1)
  - `LARINOV3` — 0.127·(3.15^(2·IGR) − 1)
  - `CLAVIER` — Clavier et al.
- **Default:** `LINEAR`
- **Checked before the run:**
  - `vsh_gr.method_id` — The selected GR transform must be one of the method ids declared by the manifest.
    Source: docs/PRD_v2/10_clay-volume.md §3.2; Geolog vsh_gr.lls L109-L139

## Output curves

| Name | Description |
|---|---|
| VSH_GR | VSH from gamma ray (unlimited) |
| VSH | Limited volume of shale |
| VSH_PROV | CLY provenance registry v1 (SB-CLY-001, DEC-036): 0 COMPUTED, 1 MISSING_INPUT, 2 MASKED_INPUT, 3 ENDPOINT_INVALID, 4 COAL (reserved for the SB-CLY-036 coal branch); MISSING outside the run. Categorical - the reason, never a mask. |
