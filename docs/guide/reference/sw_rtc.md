<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# SW — RtC (Clay + Capillary Correction)

Module id `sw_rtc` · category **Saturation** · [reference index](README.md)

LRLC RtC method: excess conductivity from clay chemistry and capillary (micropore) water is regressed as Cex = (A_CAP·CAPBW + B_QV·Qv + C0)·PHIT·RSF and removed from the measured conductivity before Archie: Sw = [Rw·(1/Rt − Cex)/PHIT^M]^(1/N). Qv comes from the QV input log when present, else from CEC·RHOG·(1−PHIT)/(100·PHIT). NO CALIBRATION COEFFICIENTS SHIP AS DEFAULTS. A foreign calibration here does not announce itself: it yields a smooth, plausible Sw that is simply wrong. Fit your own with Advance ▸ Calibrate RtC…, which regresses A_CAP/B_QV/C0 from excess conductivity over an interval you declare water-bearing. CAPBW pairs naturally with SSC's CWSH or SSPW's CAPBW_SSPW. The correction is capped at 98% of the measured conductivity so Rt_corr stays finite.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RT | Deep resistivity | `RES_DEEP` | yes | — |
| PHIT | Total porosity | `PHIT_SSC` | yes | or satisfied by `PHIT_SSPW` |
| CAPBW | Capillary-bound water volume | `CWSH` | no | — |
| QV | Qv log (meq/cm3), optional | `QV` | no | — |
| CBW | Clay-bound water (for SWE), optional | `CBW` | no | — |
| PHIT_SSPW | Total porosity — SSPW fallback (used where PHIT is absent) | `PHIT_SSPW` | no | — |
| CAPBW_SSPW | Capillary water — SSPW fallback | `CAPBW_SSPW` | no | — |
| CBW_SSPW | Clay-bound water — SSPW fallback | `CBW_SSPW` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RW *(ohm.m)*

Formation water resistivity at FT

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0.001 to 100 ohm.m

### M

Cementation exponent

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4

### N

Saturation exponent

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1 to 4

### A_CAP

Capillary water coefficient

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -10 to 10

### B_QV

Qv coefficient

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -10 to 10

### C0

Regression intercept

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -10 to 10

### RSF

Resistivity scaling factor

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 20

### CEC *(meq/100g)*

CEC when no QV log (meq/100g)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 100 meq/100g

### RHOG *(g/cc)*

Grain density for Qv

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc

## Output curves

| Name | Description |
|---|---|
| SWT_RTC | SWT from RtC (unlimited) |
| SWE_RTC | SWE from RtC (unlimited) |
| SWT | Limited total water saturation |
| SWE | Limited effective water saturation |
| VOL_UWAT | Volume of water (unflushed) |
| SW_METHOD | Producing saturation equation (categorical method code) |
| RT_CORR | Clay/capillary-corrected resistivity |
| CEX_RTC | Excess conductivity removed |
