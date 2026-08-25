<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SW — IMTS (Mineral-Textural Scaling)

Module id `sw_imts` · category **Saturation** · [reference index](README.md)

LRLC IMTS model: Waxman-Smits-family conductivity with the clay charge referenced to the ACTIVE water — Qv_eff = Qv_bulk/(1−Swirr), where Qv_bulk is the clay MASS per unit dry-rock mass times literature CEC constants (kaolinite 8 / illite 25 meq/100g of DRY ROCK), i.e. (V_kaol*RHO_KAOL*CEC_KAOL + V_ill*RHO_ILL*CEC_ILL)/(RHOG*(1-PHIT)) - the clay's own grain density is what converts its volume to the mass the charge sits on. Calibrated to lab CEC by scaling factor S. Iterates Ct = SwT^N*/F*·(Cw + B·Qv_eff/SwT) with F* = A/PHIT^M* and Juhasz B(T, Rw) until SwT is stable. SWE from CBW. VKAOL/VILL resolve from the selected clay curves. S = measured lab CEC / XRD-theoretical CEC, so it is A PROPERTY OF THE ROCK AND OF THE CLAY CURVES IT IS PAIRED WITH and ships absent. S multiplies the whole clay-charge term, so getting it wrong scales Qv_eff directly and moves Sw with no outward sign. Fit your own with Advance ▸ Calibrate S…, which regresses S from lab CEC measurements against the clay content of the very curves this run will use. S is on the GRAIN-WEIGHT basis and is NAMED S_FACTOR_GW for that reason (DEC-094): a value fitted under the older bulk-volume denominator reads roughly a fifth high at ordinary porosity.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RT | Deep resistivity | `RES_DEEP` | yes | — |
| PHIT | Total porosity | `PHIT_SSC` | yes | — |
| VKAOL | Kaolinite volume fraction | `VDCL` | no | — |
| VILL | Illite volume fraction | `VILL` | no | — |
| SWIRR | Irreducible Sw (for Qv_eff) | `SWIRR_T` | no | — |
| CBW | Clay-bound water (for SWE), optional | `CBW` | no | — |
| PHIT_SSPW | Total porosity — SSPW fallback (used where PHIT is absent) | `PHIT_SSPW` | no | — |
| CBW_SSPW | Clay-bound water — SSPW fallback | `CBW_SSPW` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RW *(ohm.m)*

Formation water resistivity at FT

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 100 ohm.m

### TEMP_C *(degC)*

Formation temperature

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 15 to 200 degC

### A

Tortuosity factor a

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.5 to 3

### MSTAR

Shaly-sand cementation exponent m*

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4

### NSTAR

Shaly-sand saturation exponent n*

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4

### S_FACTOR_GW

CEC scaling factor S (lab/XRD), grain-weight basis

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.01 to 2

### RHO_KAOL *(g/cc)*

Kaolinite grain density

- **Default:** 2.62 — source: SandiMin's own endpoint library carries this grain density (sandimin.rs LIB: clay Kaolinite RHOB 2.62, clay Illite RHOB 2.78), and docs/multimin_ref_spec.md:62 verifies the same pair against the reference-suite Multimin bound-water coefficients (Illite 0.1841, Kaolinite 0.0694). IP 2025 ships the matching un-expanded illite coefficient 0.185 (docs/research_2026-07/ip2025_chm_ingest/C_mineral_solver.md 3.4), so the two tools agree on this pair to three decimals
- **Accepted range:** 1 to 4 g/cc

### RHO_ILL *(g/cc)*

Illite grain density

- **Default:** 2.78 — source: SandiMin's own endpoint library carries this grain density (sandimin.rs LIB: clay Kaolinite RHOB 2.62, clay Illite RHOB 2.78), and docs/multimin_ref_spec.md:62 verifies the same pair against the reference-suite Multimin bound-water coefficients (Illite 0.1841, Kaolinite 0.0694). IP 2025 ships the matching un-expanded illite coefficient 0.185 (docs/research_2026-07/ip2025_chm_ingest/C_mineral_solver.md 3.4), so the two tools agree on this pair to three decimals
- **Accepted range:** 1 to 4 g/cc

### CEC_KAOL *(meq/100g)*

Kaolinite CEC constant

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 50 meq/100g

### CEC_ILL *(meq/100g)*

Illite CEC constant

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 100 meq/100g

### RHOG *(g/cc)*

Grain density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc

### SWIRR_DEF *(v/v)*

Swirr fallback when no SWIRR log

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.95 v/v

## Output curves

| Name | Description |
|---|---|
| SWT_IMTS | SWT from IMTS (unlimited) |
| SWE_IMTS | SWE from IMTS (unlimited) |
| SWT | Limited total water saturation |
| SWE | Limited effective water saturation |
| VOL_UWAT | Volume of water (unflushed) |
| SW_METHOD | Producing saturation equation (categorical method code) |
| QVEFF | Effective Qv (meq/cm3) |
