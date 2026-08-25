<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# Data Conditioning Flags

Module id `condflag` · category **Prep** · [reference index](README.md)

Flags samples whose density/neutron readings should not feed porosity or mineral solving. COAL_FLAG: RHOB < COAL_RHOB and NPHI > COAL_NPHI, plus DT > COAL_DT where a sonic exists; a washed-out hole mimics coal, so samples with BADHOLE = 1 are never called coal. TIGHT_FLAG: density porosity (from RHO_MA / RHO_FL — the same parameters, and zone overrides, as the density-porosity modules) and NPHI both below TIGHT_PHI. XOVER_FLAG: gas crossover, density porosity exceeding NPHI by more than XOVER_MIN — coal and bad hole are excluded because they fake the same light-density signature. NPHI must be in matrix units consistent with RHO_MA: limestone-unit neutron against a sandstone RHO_MA reads about 0.04 low in clean water sand, right at the XOVER_MIN threshold — convert the neutron first, then supply a sourced threshold for the declared neutron convention. Flagged beds thinner than MIN_THICK are dropped as spikes (missing samples inside a bed do not split it). SHOULDER_FLAG is the transition adjustment: logs average across bed boundaries, so samples within SHOULDER of a coal / tight bed — or a bad-hole interval at least MIN_THICK thick — still carry mixed readings; masking only the bed itself would leave those shoulder values in the conditioned data. COND_FLAG combines coal, tight, bad hole and shoulder (and crossover when OPT_XCOND = YES — leave NO when gas zones will be corrected rather than discarded); feed it as the Mask on later module runs, but leave the Mask empty on the condflag run itself — masking this run with BADHOLE would blank COND_FLAG exactly where it must read 1. MIN_THICK and SHOULDER are in the depth curve's declared unit; their DEC-077 starting values are metre-scale conventions - rescale for feet wells. Run the badhole module first so its flag is available here.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Density log | `RHOB` | yes | — |
| NPHI | Neutron porosity log (matrix units matching RHO_MA) | `NPHI` | yes | — |
| DT | Sonic transit time log | `DT` | no | — |
| BADHOLE | Bad-hole flag from the badhole module | `BADHOLE` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_MA *(g/cc)*

Matrix density

- **Default:** 2.65 — source: IP MINDEF, Techlog QM_MineralTable and SandiMin all 2.65 (3-way AGREE); docs/PRD_v2/11_porosity.md §5.1. SB-POR-011: one shared matrix density across chained modules, owner-selected 2026-08-16 over Geolog phi_den.info's shipped 2645 k/m3.
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `matrix_density`); the pane lists them with sources at the point of choice.

### RHO_FL *(g/cc)*

Fluid density

- **Default:** 1 — source: IP basicloganalysis.htm fresh-water 1.0 gm/cc; Geolog phi_den.info RHO_FL 1000 k/m3; docs/PRD_v2/11_porosity.md §5.1
- **Accepted range:** 0.5 to 1.5 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `fluid_density`); the pane lists them with sources at the point of choice.

### COAL_RHOB *(g/cc)*

Coal: density below

- **Default:** 1.9 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 1.2 to 2.4 g/cc

### COAL_NPHI *(v/v)*

Coal: neutron above

- **Default:** 0.35 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 0.15 to 0.8 v/v

### COAL_DT *(us/ft)*

Coal: sonic above (when DT present)

- **Default:** 100 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 70 to 160 us/ft

### TIGHT_PHI *(v/v)*

Tight: both porosities below

- **Default:** 0.05 — source: Jauhar adjudication DEC-077 (2026-08-19): multi-basin practitioner starting value per DEC-059 - vendors correct through charts and ship no adoptable number for this quantity (corpus negative, docs/takeover/DRAFT_ENV004_source_adjudication.md); docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 0.2 v/v

### XOVER_MIN *(v/v)*

Crossover: DPHI - NPHI above (~0.08 for limestone-unit NPHI)

- **Default:** 0.04 — source: Jauhar adjudication DEC-077 (2026-08-19): 0.04 v/v multi-basin starting value per DEC-059, ruled with the chapter's own warning in view that 0.04 equals the matrix-scale error size (SB-ENV-012/029) - convert the neutron convention first; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 0.3 v/v

### MIN_THICK *(depth)*

Drop flagged beds thinner than

- **Default:** 0.25 — source: Jauhar adjudication DEC-077 (2026-08-19): 0.25 starting value per DEC-059 - a resolution-scale convention stated in metres, in the depth curve's declared unit; rescale for feet wells; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 10 depth

### SHOULDER *(depth)*

Shoulder width beyond bed edges

- **Default:** 0.5 — source: Jauhar adjudication DEC-077 (2026-08-19): 0.5 starting value per DEC-059 - a resolution-scale convention stated in metres, in the depth curve's declared unit; rescale for feet wells; docs/takeover/DECISIONS.md
- **Accepted range:** 0 to 5 depth

## Options

### OPT_XCOND

Include gas crossover in COND_FLAG

- **Choices:**
  - `NO`
  - `YES`
- **Default:** `NO`

## Output curves

| Name | Description |
|---|---|
| COAL_FLAG | Coal flag (1 = coal) *(flag: DIAGNOSTIC_INDICATOR)* |
| TIGHT_FLAG | Tight-zone flag (1 = tight) *(flag: DIAGNOSTIC_INDICATOR)* |
| XOVER_FLAG | Gas crossover flag (1 = crossover) *(flag: DIAGNOSTIC_INDICATOR)* |
| SHOULDER_FLAG | Bed-transition shoulder flag (1 = shoulder) *(flag: DIAGNOSTIC_INDICATOR)* |
| COND_FLAG | Combined conditioning mask (1 = exclude) *(flag: EXCLUSION_MASK)* |
