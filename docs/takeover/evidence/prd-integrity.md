# Gate 1 PRD structural-integrity audit

Generated from `91_REQUIREMENTS_INDEX.md` and the checked-out `docs/PRD_v2` directory.
A recorded discrepancy remains open; generation does not resolve or amend its source.

## Consolidated requirements

- Rows: `931`.
- Unique IDs: `931`.
- Declared roll-up total: `931`.
- Total status: `CLOSED-AS-RECORDED`.

## Roll-up comparisons

| Dimension | Value | Declared | Derived | State |
|---|---|---:|---:|---|
| Status | `PRESENT-DIVERGENT` | 111 | 110 | `INCONSISTENT` |
| Status | `PRESENT-OK` | 113 | 114 | `INCONSISTENT` |

## Requirement-shape findings

- Blank priorities (15): `SB-INS-006`, `SB-INS-007`, `SB-INS-009`, `SB-INS-011`, `SB-INS-012`, `SB-INS-013`, `SB-INS-017`, `SB-INS-018`, `SB-INS-019`, `SB-INS-020`, `SB-INS-021`, `SB-INS-022`, `SB-INS-024`, `SB-INS-025`, `SB-INS-026`.
- Blank statuses (62): `SB-POR-001`, `SB-POR-002`, `SB-POR-003`, `SB-POR-004`, `SB-POR-005`, `SB-POR-006`, `SB-POR-007`, `SB-POR-008`, `SB-POR-009`, `SB-POR-010`, `SB-POR-011`, `SB-POR-012`, `SB-POR-013`, `SB-POR-014`, `SB-POR-015`, `SB-POR-016`, `SB-POR-017`, `SB-POR-018`, `SB-POR-019`, `SB-POR-020`, `SB-POR-021`, `SB-POR-022`, `SB-POR-023`, `SB-POR-024`, `SB-POR-025`, `SB-POR-026`, `SB-POR-027`, `SB-POR-028`, `SB-POR-029`, `SB-POR-030`, `SB-POR-031`, `SB-POR-032`, `SB-POR-033`, `SB-POR-034`, `SB-POR-035`, `SB-POR-036`, `SB-POR-037`, `SB-POR-038`, `SB-POR-039`, `SB-POR-040`, `SB-POR-041`, `SB-POR-042`, `SB-POR-043`, `SB-POR-044`, `SB-POR-045`, `SB-POR-046`, `SB-POR-047`, `SB-POR-048`, `SB-POR-049`, `SB-POR-050`, `SB-POR-051`, `SB-POR-052`, `SB-POR-053`, `SB-POR-054`, `SB-POR-055`, `SB-POR-056`, `SB-POR-057`, `SB-POR-058`, `SB-POR-059`, `SB-POR-060`, `SB-POR-061`, `SB-POR-062`.
- Contract-invalid statuses (2): `SB-CORE-030` = `UNMEASURED`, `SB-CORE-033` = `ABSENT — designed, parked`.
- Requirements without an owned acceptance-test ID (137): `SB-CORE-003`, `SB-CORE-005`, `SB-CORE-012`, `SB-CORE-013`, `SB-CORE-030`, `SB-CORE-031`, `SB-CORE-032`, `SB-CORE-033`, `SB-CORE-034`, `SB-CORE-035`, `SB-CORE-036`, `SB-CORE-042`, `SB-CORE-043`, `SB-CORE-044`, `SB-INS-001`, `SB-INS-002`, `SB-INS-003`, `SB-INS-004`, `SB-INS-005`, `SB-INS-006`, `SB-INS-007`, `SB-INS-008`, `SB-INS-009`, `SB-INS-010`, `SB-INS-011`, `SB-INS-012`, `SB-INS-013`, `SB-INS-014`, `SB-INS-015`, `SB-INS-016`, `SB-INS-017`, `SB-INS-018`, `SB-INS-019`, `SB-INS-020`, `SB-INS-021`, `SB-INS-022`, `SB-INS-023`, `SB-INS-024`, `SB-INS-025`, `SB-INS-026`, `SB-PLT-001`, `SB-PLT-002`, `SB-PLT-003`, `SB-PLT-004`, `SB-PLT-005`, `SB-PLT-006`, `SB-PLT-007`, `SB-PLT-008`, `SB-PLT-009`, `SB-PLT-010`, `SB-PLT-011`, `SB-PLT-012`, `SB-PLT-013`, `SB-PLT-014`, `SB-PLT-015`, `SB-PLT-016`, `SB-PLT-017`, `SB-PLT-018`, `SB-PLT-019`, `SB-PLT-020`, `SB-PLT-021`, `SB-PLT-022`, `SB-PLT-023`, `SB-PLT-024`, `SB-PLT-025`, `SB-PLT-026`, `SB-PLT-027`, `SB-PLT-028`, `SB-PLT-029`, `SB-PLT-030`, `SB-PLT-031`, `SB-PLT-032`, `SB-PLT-033`, `SB-PLT-034`, `SB-PLT-035`, `SB-POR-001`, `SB-POR-002`, `SB-POR-003`, `SB-POR-004`, `SB-POR-005`, `SB-POR-006`, `SB-POR-007`, `SB-POR-008`, `SB-POR-009`, `SB-POR-010`, `SB-POR-011`, `SB-POR-012`, `SB-POR-013`, `SB-POR-014`, `SB-POR-015`, `SB-POR-016`, `SB-POR-017`, `SB-POR-018`, `SB-POR-019`, `SB-POR-020`, `SB-POR-021`, `SB-POR-022`, `SB-POR-023`, `SB-POR-024`, `SB-POR-025`, `SB-POR-026`, `SB-POR-027`, `SB-POR-028`, `SB-POR-029`, `SB-POR-030`, `SB-POR-031`, `SB-POR-032`, `SB-POR-033`, `SB-POR-034`, `SB-POR-035`, `SB-POR-036`, `SB-POR-037`, `SB-POR-038`, `SB-POR-039`, `SB-POR-040`, `SB-POR-041`, `SB-POR-042`, `SB-POR-043`, `SB-POR-044`, `SB-POR-045`, `SB-POR-046`, `SB-POR-047`, `SB-POR-048`, `SB-POR-049`, `SB-POR-050`, `SB-POR-051`, `SB-POR-052`, `SB-POR-053`, `SB-POR-054`, `SB-POR-055`, `SB-POR-056`, `SB-POR-057`, `SB-POR-058`, `SB-POR-059`, `SB-POR-060`, `SB-POR-061`, `SB-POR-062`.

## Chapter references

| Chapter | Requirements | Files resolved | State |
|---|---:|---:|---|
| `04_CORE_REQUIREMENTS.md` | 25 | 1 | `CLOSED-AS-RECORDED` |
| `10_clay-volume.md` | 55 | 1 | `CLOSED-AS-RECORDED` |
| `11_porosity.md` | 62 | 1 | `CLOSED-AS-RECORDED` |
| `12_saturation.md` | 51 | 1 | `CLOSED-AS-RECORDED` |
| `13_mineral-solver.md` | 46 | 1 | `CLOSED-AS-RECORDED` |
| `14_cutoffs-summation-mc.md` | 61 | 1 | `CLOSED-AS-RECORDED` |
| `15_sat-height-rocktyping.md` | 42 | 1 | `CLOSED-AS-RECORDED` |
| `16_nmr.md` | 38 | 1 | `CLOSED-AS-RECORDED` |
| `17_thinbed-laminated.md` | 66 | 1 | `CLOSED-AS-RECORDED` |
| `18_geomech-ppfg.md` | 52 | 1 | `CLOSED-AS-RECORDED` |
| `19_toc-unconventional.md` | 43 | 1 | `CLOSED-AS-RECORDED` |
| `20_envcorr-qc.md` | 58 | 1 | `CLOSED-AS-RECORDED` |
| `21_data-io.md` | 63 | 1 | `CLOSED-AS-RECORDED` |
| `22_database-model.md` | 43 | 1 | `CLOSED-AS-RECORDED` |
| `23_plotting-interactivity.md` | 35 | 1 | `CLOSED-AS-RECORDED` |
| `24_ml-advanced.md` | 65 | 1 | `CLOSED-AS-RECORDED` |
| `25_fluidsub-rockphysics.md` | 52 | 1 | `CLOSED-AS-RECORDED` |
| `26_production-logging.md` | 48 | 1 | `CLOSED-AS-RECORDED` |
| `27_ip-install-blockers.md` | 26 | 1 | `CLOSED-AS-RECORDED` |

- Domain chapter files on disk: `18`.
- Domain chapter files not represented by consolidated rows: None.

## Document-map artifacts

| Artifact | Present | State |
|---|---|---|
| `00_INDEX.md` | yes | `CLOSED-AS-RECORDED` |
| `01_PRODUCT.md` | yes | `CLOSED-AS-RECORDED` |
| `02_RISKS_AND_CONTRADICTIONS.md` | yes | `CLOSED-AS-RECORDED` |
| `03_EVIDENCE_BASE.md` | yes | `CLOSED-AS-RECORDED` |
| `04_CORE_REQUIREMENTS.md` | yes | `CLOSED-AS-RECORDED` |
| `05_STRATEGY.md` | yes | `CLOSED-AS-RECORDED` |
| `06_SEQUENCING_AND_GATES.md` | yes | `CLOSED-AS-RECORDED` |
| `CONTRACT.md` | yes | `CLOSED-AS-RECORDED` |
| `10_clay-volume.md` | yes | `CLOSED-AS-RECORDED` |
| `11_porosity.md` | yes | `CLOSED-AS-RECORDED` |
| `12_saturation.md` | yes | `CLOSED-AS-RECORDED` |
| `13_mineral-solver.md` | yes | `CLOSED-AS-RECORDED` |
| `14_cutoffs-summation-mc.md` | yes | `CLOSED-AS-RECORDED` |
| `15_sat-height-rocktyping.md` | yes | `CLOSED-AS-RECORDED` |
| `16_nmr.md` | yes | `CLOSED-AS-RECORDED` |
| `17_thinbed-laminated.md` | yes | `CLOSED-AS-RECORDED` |
| `18_geomech-ppfg.md` | yes | `CLOSED-AS-RECORDED` |
| `19_toc-unconventional.md` | yes | `CLOSED-AS-RECORDED` |
| `20_envcorr-qc.md` | yes | `CLOSED-AS-RECORDED` |
| `21_data-io.md` | yes | `CLOSED-AS-RECORDED` |
| `22_database-model.md` | yes | `CLOSED-AS-RECORDED` |
| `23_plotting-interactivity.md` | yes | `CLOSED-AS-RECORDED` |
| `24_ml-advanced.md` | yes | `CLOSED-AS-RECORDED` |
| `25_fluidsub-rockphysics.md` | yes | `CLOSED-AS-RECORDED` |
| `26_production-logging.md` | yes | `CLOSED-AS-RECORDED` |
| `27_ip-install-blockers.md` | yes | `CLOSED-AS-RECORDED` |
| `90_GAP_ANALYSIS.md` | no | `OPEN` |
| `91_REQUIREMENTS_INDEX.md` | yes | `CLOSED-AS-RECORDED` |

Missing promised artifacts: `90_GAP_ANALYSIS.md`.

## RESUME chapter-count claim

- Claimed written: `11`.
- Claimed total: `18`.
- Domain chapter files on disk: `18`.
- State: `INCONSISTENT`.

## Spine-pending register

| Item | Title | State |
|---|---|---|
| `SP-001` | Tier-C implementation policy | `CLOSED-AS-RECORDED` |
| `SP-002` | Unconventional gas-content scope | `OPEN` |
| `SP-003` | Omovie Sonic Saturation independent-derivation evidence | `OPEN` |
| `SP-004` | Mineral-solver sonic bridge independent-derivation evidence | `OPEN` |
| `SP-005` | Missing record of the superseded PRD-v1 follow-on documents | `OPEN` |
| `SP-006` | Requirements total is 931, not 932 | `OPEN` |
| `SP-007` | `SB-CORE` requirement-number gaps | `OPEN` |
| `SP-008` | `SB-CORE` test-number gaps — CLOSED 2026-08-09 | `CLOSED-AS-RECORDED` |
| `SP-009` | Porosity requirements omit status and tests omit T26–T27 | `OPEN` |
| `SP-010` | Fifteen installer requirements omit priority | `OPEN` |
| `SP-011` | Two `SB-CORE` statuses are outside the contract vocabulary | `OPEN` |
| `SP-012` | The lack-of-compaction correction runs BACKWARDS on its own shipped default | `OPEN` |
| `SP-013` | The `RHG` option is a one-segment approximation under a three-segment name | `OPEN` |
| `SP-014` | A basin name ships in module dialog text | `OPEN` |
| `SP-015` | Two `phi_son` behaviours were correct but uncited, and are now citable | `OPEN` |

## Dashboard counts

- Consolidated requirements: `931`.
- Roll-up mismatches: `2`.
- Blank priorities: `15`.
- Blank statuses: `62`.
- Invalid statuses: `2`.
- Requirements without an owned test ID: `137`.
- Missing promised artifacts: `1`.
- Stale RESUME claims: `1`.

## Interpretation boundary

This audit reports structural agreement and disagreement. It does not repair PRD text, infer a
missing priority or status, supply a test, or convert an open spine item into a closed one.
