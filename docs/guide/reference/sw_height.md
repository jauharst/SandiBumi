<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SW — Saturation-Height

Module id `sw_height` · category **Saturation** · [reference index](README.md)

SWH from height above the free-water level. LEVERETT: Pc = 0.433*(RHO_W-RHO_HC)*h_ft, J = 0.21645*Pc/IFT_RES*sqrt(PERM/PHIE), SWH = SWH_A * J^SWH_B (fit SWH_A/SWH_B from core Pc data via Import SCAL — the fit is reported there). SKELT (Skelt-Harrison): SWH = 1 - SH_A*exp(-(SH_B/(h+SH_D))^SH_C), h in metres (SH_B and SH_D are metres by the published form, so the height is converted for that branch alone — the HAFWL curve itself is written in the project's own depth unit). Below the FWL (h <= 0) SWH = 1. Result limited to [SWT_IRR, 1]. FWL is zone-overridable for stacked reservoirs with different contacts. Height is measured from the TVD input when a TVD curve is supplied (else measured depth) so deviated wells are not over-stated — MD height overstates true height by ~1/cos(inc); enter FWL on the SAME reference (a negative value for a sub-sea TVDSS FWL).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| PHIE | Limited effective porosity | `PHIE` | yes | — |
| PERM | Working permeability (LEVERETT only) | `PERM` | no | — |
| TVD | True vertical (sub-sea) depth for height; defaults to measured depth | `TVD` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### FWL *(depth)*

Free-water level (same reference as the vertical-depth input; negative = subsea TVDSS)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -10000 to 20000 depth

### RHO_W *(g/cc)*

Water density

- **Default:** 1 — source: docs/ref_shf.md:56 and Techlog sand-summary water-density default; docs/PRD_v2/15_sat-height-rocktyping.md §5.1
- **Accepted range:** 0.8 to 1.3 g/cc

### RHO_HC *(g/cc)*

Hydrocarbon density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.05 to 1.1 g/cc

### IFT_RES *(dyn/cm)*

Reservoir sigma*cos(theta)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 500 dyn/cm
- **Checked before the run:**
  - `ift_res.required_when_leverett` — IFT_RES is required when OPT_SWH = LEVERETT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Leverett parameters

### SWH_A

Leverett coefficient A (from J-fit)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 100
- **Checked before the run:**
  - `swh_a.required_when_leverett` — SWH_A is required when OPT_SWH = LEVERETT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Leverett parameters

### SWH_B

Leverett exponent B (from J-fit, usually negative)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -5 to 0
- **Checked before the run:**
  - `swh_b.required_when_leverett` — SWH_B is required when OPT_SWH = LEVERETT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Leverett parameters

### SH_A

Skelt-Harrison A

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 1
- **Checked before the run:**
  - `sh_a.required_when_skelt` — SH_A is required when OPT_SWH = SKELT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Skelt-Harrison parameters

### SH_B *(m)*

Skelt-Harrison B

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 5000 m
- **Checked before the run:**
  - `sh_b.required_when_skelt` — SH_B is required when OPT_SWH = SKELT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Skelt-Harrison parameters

### SH_C

Skelt-Harrison C

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.1 to 10
- **Checked before the run:**
  - `sh_c.required_when_skelt` — SH_C is required when OPT_SWH = SKELT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Skelt-Harrison parameters

### SH_D *(m)*

Skelt-Harrison D

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -100 to 1000 m
- **Checked before the run:**
  - `sh_d.required_when_skelt` — SH_D is required when OPT_SWH = SKELT. Applies when: [object Object].
    Source: docs/PRD_v2/15_sat-height-rocktyping.md §5 Skelt-Harrison parameters

### SWT_IRR *(v/v)*

Irreducible water saturation (lower clamp)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.8 v/v

## Options

### OPT_SWH

Saturation-height model

- **Choices:**
  - `LEVERETT`
  - `SKELT`
- **Default:** `LEVERETT`

## Output curves

| Name | Description |
|---|---|
| SWT_HGT | SWT from height function (unlimited) |
| SWT | Limited total water saturation |
| SW_METHOD | Producing saturation equation (categorical method code) |
| HAFWL | Height above free-water level |
