<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Pre-Calculation (P / T / Rmf / Ct / Cxo)

Module id `precalc` · category **Prep** · [reference index](README.md)

Reservoir-condition inputs for saturation and SandiMin work, from trend fits: formation temperature = SURF_TEMP + TEMP_GRAD*TVDSS and FPRESS = PSURF + PGRAD*TVDSS, both linear in true vertical depth. Gradients — and the TREND fit below — are per depth unit of the TVDSS curve: enter per-metre values (and a metric refit) for metric wells. The shipped 77 degF / 0.026 deg/ft starting values are one study's feet-based fits, attributed to the owner by DEC-077 — refit per basin, and convert before a metric well (SB-ENV-045 records the 66 degC cost of skipping that). SURF_TEMP / TEMP_GRAD / RMF_TEMP are entered in OPT_TU units, but the FTEMP curve is always written in degC (the unit every downstream module assumes); FTEMP_F is the same trend in degF for SandiMin fluid-property entry. RMF at formation temperature comes either from a surface mud-filtrate measurement Arps-converted per sample (ARPS) or from a field regression RMF = RMF_A + RMF_B*log10(TVDSS) already fit at formation temperature (TREND, for wells with no mud data). CT = 1000/RT and CXO = 1000/RXO are QC/plotting conductivities in mmho/m — SandiMin's CT/CXO tool rows read the RESISTIVITY curves directly and convert internally, so do not feed these curves to them. No TVDSS curve → measured DEPTH is used instead (fine for near-vertical wells).

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| TVDSS | True vertical depth subsea | `TVDSS` | no | — |
| RT | Deep resistivity | `RES_DEEP` | no | — |
| RXO | Flushed-zone resistivity | `RXO` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### SURF_TEMP *(degF|degC)*

Surface temperature (intercept, whole well)

- **Default:** 77 — source: Jauhar adjudication DEC-077 (2026-08-19): 77 degF starting value per DEC-059, one study's feet-based fit re-attributed to the owner - entered in OPT_TU units (default degF); refit per basin, and convert for metric wells (SB-ENV-045); docs/takeover/DECISIONS.md
- **Accepted range:** -50 to 150 degF|degC
- **One value per well** — a named-zone override of this parameter is refused.

### TEMP_GRAD *(deg/ft|m)*

Temperature gradient per TVDSS unit (whole well)

- **Default:** 0.026 — source: Jauhar adjudication DEC-077 (2026-08-19): 0.026 deg/ft starting value per DEC-059, one study's feet-based fit re-attributed to the owner - per depth unit of the TVDSS curve; refit per basin, and convert for metric wells (SB-ENV-045); CONFIRMED as the house default by DEC-085 R-3 (2026-08-20, verbatim: the right one is 0.026 degF/ft) - ftemp_grad.TGRAD now carries the same physical gradient in degC/m; docs/takeover/DECISIONS.md
- **Accepted range:** 0.0005 to 0.2 deg/ft|m
- **One value per well** — a named-zone override of this parameter is refused.

### PSURF *(psi)*

Formation pressure intercept

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -500 to 5000 psi

### PGRAD *(psi/ft|m)*

Pressure gradient per TVDSS unit

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.05 to 5 psi/ft|m

### RMF_MEAS *(ohmm)*

Rmf measured at surface (ARPS)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 20 ohmm
- **Checked before the run:**
  - `rmf_meas.required_when_arps` — RMF_MEAS is required when OPT_RMF = ARPS. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters

### RMF_TEMP *(degF|degC)*

Rmf measurement temperature (ARPS)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -50 to 150 degF|degC
- **Checked before the run:**
  - `rmf_temp.required_when_arps` — RMF_TEMP is required when OPT_RMF = ARPS. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters

### RMF_A *(ohmm)*

RMF trend intercept (TREND, ft-based fit)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 5 ohmm
- **Checked before the run:**
  - `rmf_a.required_when_trend` — RMF_A is required when OPT_RMF = TREND. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters

### RMF_B *(ohmm)*

RMF trend slope on log10(TVDSS) (TREND — fit must use the TVDSS curve's depth unit)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -2 to 2 ohmm
- **Checked before the run:**
  - `rmf_b.required_when_trend` — RMF_B is required when OPT_RMF = TREND. Applies when: [object Object].
    Source: docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters

## Options

### OPT_TU

Temperature unit for entered params

- **Choices:**
  - `degF`
  - `degC`
- **Default:** `degF`

### OPT_RMF

RMF source

- **Choices:**
  - `ARPS`
  - `TREND`
- **Default:** `ARPS`

## Output curves

| Name | Description |
|---|---|
| FTEMP | Formation temperature (always degC) |
| FTEMP_F | Formation temperature in degF (SandiMin fluid entry) |
| FPRESS | Formation pressure |
| RMF | Mud filtrate resistivity at FTEMP |
| CT | Deep conductivity 1000/RT (QC/plotting) |
| CXO | Flushed conductivity 1000/RXO (QC/plotting) |
