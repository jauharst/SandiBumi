# SandiBumi takeover decisions

This register separates product-owner policy from engineering fact. `OPEN` means no decision has
been inferred. A decision row changes only from explicit Jauhar direction or named external evidence.

| ID | Decision | State | Current direction | What settles it | Blocks |
|---|---|---|---|---|---|
| DEC-001 | Re-adjudicate original P0 priorities for the paid pilot | DECIDED | Authorized 2026-08-10 | Jauhar authorization | none |
| DEC-002 | Release program | DECIDED | Five gates; Windows-first paid offline pilot | Jauhar authorization | none |
| DEC-003 | Pilot workflow and representative corpus | NEEDS-JAUHAR | OPEN | Jauhar names the workflow and supplies locally controlled representative data | G4 |
| DEC-004 | Customer-facing 2,000-well statement | NEEDS-JAUHAR | Design recommends removal until a defined benchmark proves it | Jauhar chooses removal now or an explicitly non-customer-facing hold | G5 |
| DEC-005 | Licence unit and activation | NEEDS-JAUHAR | OPEN | Commercial decision informed by deployment constraints | G5 |
| DEC-006 | Commercial model and support commitment | NEEDS-JAUHAR | OPEN | Commercial decision with written hours and escalation boundary | G5 |
| DEC-007 | Update delivery and supported-version window | NEEDS-JAUHAR | OPEN | Commercial and deployment decision | G5 |
| DEC-008 | Portfolio benchmark operations and thresholds | NEEDS-JAUHAR | OPEN | Named operations, fixture and hardware profile | later scale claim |
| DEC-009 | Lineage granularity beyond the pilot audit need | NEEDS-JAUHAR | OPEN | Audit requirement from the pilot or buyer | later lineage design |
| DEC-010 | Linux product timing and support contract | DEFERRED | Revisit after the Windows pilot | Named opportunity and support capacity | no Windows-pilot block |
| DEC-011 | Geomechanics / PPFG product timing | DEFERRED | Hold SB-GEO for the next product version; keep the current Windows pilot focused on open-hole petrophysics | Revisit after the petrophysics pilot scope is accepted | no current-pilot block |
| DEC-012 | Wyllie compaction behavior when `Cp < 1` | DECIDED | Refuse the run; Help may explain the condition but cannot substitute for the refusal | Jauhar direction, 2026-08-11 | SB-POR-017 implementation and acceptance test |
| DEC-013 | POR output naming and intentional replacement | DECIDED | Output curve names are user-configurable. Distinct names preserve parallel results; deliberately reusing a current name is an explicit replacement and must remain versioned and undoable. Silent name collision is forbidden | Jauhar directions, 2026-08-11 | SB-POR-004 implementation and acceptance tests |
| DEC-014 | POR quick-look, hydrocarbon-response and iterative-method separation | DECIDED | Arithmetic and RMS remain explicit available contracts; the RMS comparison role and SSC/SSPW RMS-conditioning role stay distinct. The Gaymard-Poupon hydrocarbon-response path and the coupled porosity-`Sxo`/`Sw` iterative path are mandatory, separately selected contracts. No route silently stands in for another | Jauhar direction, 2026-08-11; exact equations and parameters remain chapter/source-bound | SB-POR-023, SB-POR-029 through 038, SB-POR-050 through 052, SB-POR-057 and SB-POR-059 |
| DEC-015 | Exact POR-wide common-contract boundary | NEEDS-JAUHAR | OPEN: decide whether SB-POR-001 retains one literal limiting contract for every method, or permits method-specific numerical limits beneath one common POR family/provenance/flag envelope. The latest `805` reference accompanied the separate `Cp < 1` refusal, so no SB-POR-001 choice is inferred | Jauhar explicitly chooses one boundary | SB-POR-001 through 003 migration |
| DEC-016 | Required POR capability set | DECIDED | Chart-free analytic N-D, the source-gated hydrocarbon-response chain, excavation correction and neutron-sonic all belong in the product. This settles inclusion, not a missing source, parameter, implementation sequence or an otherwise deferred pilot-timing row | Jauhar direction, 2026-08-11; each chapter source gate still applies | SB-POR-021, SB-POR-027 and SB-POR-029 through 042 |
