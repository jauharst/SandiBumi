<!-- GENERATED — do not hand-edit. Regenerate with `node tools/gen-module-reference.mjs` (source: `docs/generated/module_manifests.json`, kept fresh by `manifest_reference_test.rs`). Hand-written prose belongs in `notes/<module>.md`. -->

# VSH from Density-Neutron

Module id `vsh_dn` · category **VSH** · [reference index](README.md)

Two-log crossplot VSH: the (RHOB, NPHI) point's position between the clean matrix line and the shale point. Density in g/cc. CAUTION: the neutron shale response is hydroxyl-driven, so a single NPHI_SH endpoint is clay-type sensitive — a 4-OH clay (illite/smectite) gives ~12 p.u. N-D separation vs ~35 p.u. for an 8-OH clay (kaolinite/chlorite). Supply GR to raise VSH_DN_FLAG where the N-D VSH diverges from the clay-type-insensitive GR VSH (clay-type or gas ambiguity), or falls off the matrix–shale–fluid triangle.

## Input curves

| Role | Description | Resolves to | Required | Notes |
|---|---|---|---|---|
| RHOB | Density log | `RHO_COR` → `RHOB_EC` → `RHOB` | yes | — |
| NPHI | Neutron porosity log | `NPHI_COR` → `NPHI_EC` → `NPHI` | yes | — |
| GR | Gamma ray (optional clay-type cross-check) | `GR_COR` → `GR_EC` → `GR` | no | — |

## Parameters

Whole-well defaults; per-zone values from the Zones pane take precedence inside their zones (except where a parameter is marked one-value-per-well).

### RHO_MA *(g/cc)*

Matrix density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 2 to 3.2 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `matrix_density`); the pane lists them with sources at the point of choice.
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### RHO_SH *(g/cc)*

Shale density

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 1.5 to 3 g/cc
- Competing shipped values exist for this parameter across installed tools (topic `shale_density`); the pane lists them with sources at the point of choice.
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### RHO_FL *(g/cc)*

Fluid density

- **Default:** 1 — source: Geolog vsh_dn.info RHO_FL DEFAULT 1000 k/m3; Techlog petrophysics-vsh-from-neutrondensity.html RHO fluid 1.0 g/cm3; docs/PRD_v2/10_clay-volume.md §5
- **Accepted range:** 0.5 to 1.5 g/cc
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### NPHI_MA *(v/v)*

Matrix neutron porosity

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** -0.15 to 0.5 v/v
- Competing shipped values exist for this parameter across installed tools (topic `matrix_neutron_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### NPHI_SH *(v/v)*

Shale neutron porosity

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 0.8 v/v
- Competing shipped values exist for this parameter across installed tools (topic `shale_neutron_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### NPHI_FL *(v/v)*

Fluid neutron porosity

- **Default:** 1 — source: Geolog vsh_dn.info NPHI_FL 1 v/v; Techlog petrophysics-vsh-from-neutrondensity.html NPHI fluid 1.0; docs/PRD_v2/10_clay-volume.md §5
- **Accepted range:** 0.5 to 1.2 v/v
- **Guidance:** IP constructs the clean line from two interpreter-picked points; Geolog and Techlog constrain it through matrix and fluid points. Pick the shale point off the clean line and retain the chosen construction with the interpretation.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F15; IP clayequationsandmethodology.htm; Geolog vsh_dn.lls; Techlog VSH neutron-density page

### GR_MA *(gAPI)*

Clean GR (clay-type cross-check)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 0 to 150 gAPI
- Competing shipped values exist for this parameter across installed tools (topic `gr_clean_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP derives endpoints by pooling a Percentile Group, pre-clipping at 0%/98%, computing a selected percentile and linearly extrapolating. Its clay percentile is 130%; its clean percentile is unstated. Techlog offers 5%/95%; P3/P97 is an optional named house preset. Treat these as alternative procedures, not a generic endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; IP clayparameters.htm (57, 59, 60); Techlog VSH single-log pages; docs/workflow_standards.md

### GR_SH *(gAPI)*

Shale GR (clay-type cross-check)

- **No shipped default.** You supply this value, and a run entering it explicitly must also cite the source that covers it.
- **Accepted range:** 40 to 400 gAPI
- Competing shipped values exist for this parameter across installed tools (topic `gr_shale_endpoint`); the pane lists them with sources at the point of choice.
- **Guidance:** IP derives endpoints by pooling a Percentile Group, pre-clipping at 0%/98%, computing a selected percentile and linearly extrapolating. Its clay percentile is 130%; its clean percentile is unstated. Techlog offers 5%/95%; P3/P97 is an optional named house preset. Treat these as alternative procedures, not a generic endpoint value.
  Source: docs/PRD_v2/10_clay-volume.md §3.5 F17 and §5; IP clayparameters.htm (57, 59, 60); Techlog VSH single-log pages; docs/workflow_standards.md

### FLAG_TOL *(v/v)*

Flag |VSH(N-D) − VSH(GR)| above this

- **Default:** 0.25 — source: docs/PRD_v2/10_clay-volume.md §5.1 — SandiBumi diagnostic threshold
- **Accepted range:** 0.05 to 1 v/v

## Output curves

| Name | Description |
|---|---|
| VSH_DN | VSH from density-neutron (unlimited) |
| VSH | Limited volume of shale |
| VSH_DN_FLAG | 1 where N-D VSH is unreliable (off-model, or diverges from GR VSH) |
