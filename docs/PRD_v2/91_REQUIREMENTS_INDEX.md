# PRD v2 — consolidated requirements index

This is the counted index of the requirement definitions in `04_CORE_REQUIREMENTS.md` and domain
chapters `10` through `27`, as held on 2026-08-08. It answers the release-gate question from
`06_SEQUENCING_AND_GATES.md` §§23.1 and 24: which `P0` requirements are not `PRESENT-OK`, and what
explicit verification does each requirement own?

## Reading rules

- A row is counted only from a requirement definition in §4 (or §15 in
  `04_CORE_REQUIREMENTS.md`), never from a mention in rationale, traceability, or a test.
- `Title`, `Priority`, and `Status` preserve the chapter's wording. `11_porosity.md` and
  `15_sat-height-rocktyping.md` do not give separate short titles, so their verbatim opening
  requirement statement is carried as the title. Blank priority or status cells mean the chapter
  did not state that field; no value is inferred.
- `Verified by` contains only test IDs explicitly owned by the requirement's own **Verified by**
  clause. A blank cell is intentional. Tests elsewhere in a chapter are not guessed onto a
  requirement.
- `UNMEASURED` and `ABSENT — designed, parked` are preserved exactly from
  `04_CORE_REQUIREMENTS.md`; neither is a status defined by `CONTRACT.md` §3.
- The commissioning brief says 932 distinct IDs. The reproducible count is **931**: 25 `SB-CORE`
  definitions plus 906 domain definitions. The mismatch is recorded in `_SPINE_PENDING.md`.

## Open `P0` — 235 requirements

This table contains every requirement whose priority is explicitly `P0` and whose status is not
exactly `PRESENT-OK`. The 17 blank statuses are the `P0` rows in `11_porosity.md`; they remain open
here because the chapter does not state that they are `PRESENT-OK`.

| ID | Title | Status | Chapter | Verified by |
|---|---|---|---|---|
| `SB-CLY-001` | Refuse and flag degenerate endpoints, never null silently | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T01`, `SB-CLY-T24`, `SB-CLY-T32` |
| `SB-CLY-009` | Domain clamps computed from transform parameters | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T07`, `SB-CLY-T08` |
| `SB-CLY-010` | A clamped sample is marked as clamped | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T10` |
| `SB-CLY-013` | Limestone-matrix precondition on neutron indicators | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T17` |
| `SB-CLY-016` | Validate `R_clay < R_clean` before branching | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T20` |
| `SB-CLY-021` | Degenerate crossplot geometry is refused and reported | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T24`, `SB-CLY-T32` |
| `SB-CLY-027` | Clip each indicator before combining, never after | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T29` |
| `SB-CLY-029` | A zero is a value, not an absence | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T31` |
| `SB-CLY-031` | Every clay/shale volume carries a provenance curve | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33`, `SB-CLY-T34` |
| `SB-CLY-034` | No magic sentinel for a rejected sample | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T35`, `SB-CLY-T44` |
| `SB-CLY-043` | Shale volume and clay volume are distinct typed quantities | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T28`, `SB-CLY-T43` |
| `SB-CLY-050` | Where the vendors disagree, ship no default and surface the conflict | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T18`, `SB-CLY-T19`, `SB-CLY-T20` |
| `SB-CLY-054` | Unit-typed quantities; no magic scale constants | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T21`, `SB-CLY-T42` |
| `SB-CORE-001` | Depth unit is a first-class, carried property | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T01`, `SB-CORE-T01b`, `SB-CORE-T02` |
| `SB-CORE-002` | A degraded or failed result is never presented as a clean one | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T03`, `SB-CORE-T04`, `SB-CORE-T05`, `SB-CORE-T06`, `SB-CORE-T07`, `SB-CORE-T08`, `SB-CORE-T09` |
| `SB-CORE-004` | No parameter ships without a source | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T10`, `SB-CORE-T11` |
| `SB-CORE-006` | One name, one equation | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T17`, `SB-CORE-T18` |
| `SB-CORE-007` | One definition for every constant and every transform | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T19`, `SB-CORE-T20`, `SB-CORE-T23` |
| `SB-CORE-015` | No artifact ships that SandiBumi's own reader rejects | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T14`, `SB-CORE-T15`, `SB-CORE-T16` |
| `SB-CORE-040` | Verification is indexed by capability | `ABSENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T12` |
| `SB-CORE-041` | The tree builds and tests from a fresh clone | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T13` |
| `SB-CUT-016` | Ship no cut-off value | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-017` | Carry a source string on every default | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-018` | Resolve every cut-off entry point from one authority | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T35` |
| `SB-CUT-031` | Make the shift-to-σ multiple explicit and mandatory, and set it to 2 for IP-sourced widths | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T13` |
| `SB-CUT-041` | Never clamp before accumulation | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T15`, `SB-CUT-T23`, `SB-CUT-T25` |
| `SB-DBM-001` | One run record per computed curve, resolvable in one hop | `PARTIAL` | `22_database-model.md` | `SB-DBM-T03`, `SB-DBM-T10` |
| `SB-DBM-002` | The run record pins module identity by version, not by name | `ABSENT` | `22_database-model.md` | `SB-DBM-T04`, `SB-DBM-T15` |
| `SB-DBM-003` | Every petrophysical parameter in a run record carries a source string | `ABSENT` | `22_database-model.md` | `SB-DBM-T05`, `SB-DBM-T09`, `SB-DBM-T30` |
| `SB-DBM-004` | The run record stores the effective parameter set, not only the overrides | `PARTIAL` | `22_database-model.md` | `SB-DBM-T06`, `SB-DBM-T15` |
| `SB-DBM-005` | The run record carries a method-derivation citation, not only parameter values | `ABSENT` | `22_database-model.md` | `SB-DBM-T07`, `SB-DBM-T10` |
| `SB-DBM-006` | Inputs are recorded as resolved identities, with the rule that chose them and the candidates it rejected | `PARTIAL` | `22_database-model.md` | `SB-DBM-T08` |
| `SB-DBM-010` | Provenance travels into the deliverable | `ABSENT` | `22_database-model.md` | `SB-DBM-T10` |
| `SB-DBM-014` | Every stochastic operation records its seed and its seeding rule | `PARTIAL` | `22_database-model.md` | `SB-DBM-T14`, `SB-DBM-T15` |
| `SB-DBM-015` | The re-run manifest is enumerated, stored, and checkable | `ABSENT` | `22_database-model.md` | `SB-DBM-T15`, `SB-DBM-T16` |
| `SB-DBM-018` | Training-set identity is recorded as ids and intervals, not as names | `PARTIAL` | `22_database-model.md` | `SB-DBM-T18`, `SB-DBM-T20` |
| `SB-DBM-019` | A stored model carries its seed, its full library set and an artifact hash | `PARTIAL` | `22_database-model.md` | `SB-DBM-T19`, `SB-DBM-T21` |
| `SB-DBM-020` | Both apply paths stamp the model identity into the produced curve's provenance | `PARTIAL` | `22_database-model.md` | `SB-DBM-T20` |
| `SB-DBM-021` | Model artifacts are native-only; a foreign artifact is refused at the store boundary | `ABSENT` | `22_database-model.md` | `SB-DBM-T21` |
| `SB-DBM-028` | A declared sampling style is verified against the reference column on ingest, and the verdict is stored | `ABSENT` | `22_database-model.md` | `SB-DBM-T27` |
| `SB-DBM-030` | Null discipline: a threshold, not an equality; and "no value" is not "no parameter" | `ABSENT` | `22_database-model.md` | `SB-DBM-T29`, `SB-DBM-T30` |
| `SB-DBM-039` | A job result distinguishes clean, degraded and failed, and the store records which | `PARTIAL` | `22_database-model.md` | `SB-DBM-T39`, `SB-DBM-T41` |
| `SB-DIO-004` | Null recognition MUST be one relative-tolerance transform, and recognition MUST NOT rewrite. | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T06`, `SB-DIO-T07`, `SB-DIO-T08` |
| `SB-DIO-015` | An index with no declared unit anywhere MUST refuse. | `PARTIAL` | `21_data-io.md` | `SB-DIO-T22`, `SB-DIO-T23`, `SB-DIO-T24` |
| `SB-DIO-016` | The DLIS index unit MUST be read and reconciled. | `ABSENT` | `21_data-io.md` | `SB-DIO-T25`, `SB-DIO-T26` |
| `SB-DIO-017` | The LAS writer MUST write the depth unit it actually used. | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T27`, `SB-DIO-T28` |
| `SB-DIO-023` | Numeric columns MUST be validated against physical bounds, not against their labels. | `ABSENT` | `21_data-io.md` | `SB-DIO-T36`, `SB-DIO-T37`, `SB-DIO-T38` |
| `SB-DIO-031` | A different curve's data MUST NOT be supplied under a requested name. | `ABSENT` | `21_data-io.md` | `SB-DIO-T47` |
| `SB-DIO-051` | Provenance MUST be carried into the deliverable. | `ABSENT` | `21_data-io.md` | `SB-DIO-T71`, `SB-DIO-T72`, `SB-DIO-T73` |
| `SB-DIO-054` | Every skipped frame, channel, curve and row MUST be counted and named. | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T77`, `SB-DIO-T78`, `SB-DIO-T79` |
| `SB-DIO-055` | An export that omits data MUST say what it omitted. | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T80`, `SB-DIO-T81` |
| `SB-DIO-061` | Malformed input MUST be located, counted, named, and regression-tested against a corpus. | `PARTIAL` | `21_data-io.md` | `SB-DIO-T91`, `SB-DIO-T92`, `SB-DIO-T93`, `SB-DIO-T94` |
| `SB-ENV-003` | A violated precondition produces a refusal or a flagged result, never an unmarked number | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T02`, `SB-ENV-T03`, `SB-ENV-T04`, `SB-ENV-T05` |
| `SB-ENV-004` | Every parameter carries a source string, built as one change with the validity field | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T06`, `SB-ENV-T07` |
| `SB-ENV-005` | A corrected curve carries the list of steps actually applied | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T08`, `SB-ENV-T09`, `SB-ENV-T10` |
| `SB-ENV-006` | A curve named "corrected" MUST have been corrected | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T11`, `SB-ENV-T12` |
| `SB-ENV-009` | A method-selection string that matches no known method is an error | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T03`, `SB-ENV-T15` |
| `SB-ENV-012` | Neutron matrix scale is a declared property of the curve and is validated at every consumer | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T18`, `SB-ENV-T19` |
| `SB-ENV-014` | Correction coefficients ship with a source or ship ABSENT | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T06`, `SB-ENV-T07`, `SB-ENV-T21` |
| `SB-ENV-016` | A measured property of the formation or the borehole ships no default | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T07`, `SB-ENV-T25` |
| `SB-ENV-024` | Bad-hole thresholds ship ABSENT with cited presets | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T07`, `SB-ENV-T33` |
| `SB-ENV-025` | Bit size is an input, never a default | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T33`, `SB-ENV-T34` |
| `SB-ENV-026` | DRHO's unit is declared on the curve and validated at the threshold | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T35` |
| `SB-ENV-027` | A module whose purpose is to produce a value where the mask says there is none MUST be exempt from the mask | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T36`, `SB-ENV-T37` |
| `SB-ENV-030` | One flag polarity, defined once, as a type | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T38`, `SB-ENV-T39` |
| `SB-ENV-043` | One formation-temperature definition, one mnemonic | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-CORE-T23`, `SB-ENV-T50`, `SB-ENV-T51` |
| `SB-ENV-044` | Formation temperature is a function of true vertical depth | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T51`, `SB-ENV-T52` |
| `SB-ENV-045` | The geothermal gradient carries a declared, validated compound unit | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T52`, `SB-ENV-T53` |
| `SB-ENV-048` | The resistivity temperature constant is defined once, cited, and surfaced | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T56`, `SB-ENV-T57` |
| `SB-ENV-057` | One token for "a length in the project's depth unit", validated once | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T43`, `SB-ENV-T67` |
| `SB-GEO-001` | Gate six independently versioned domain units | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T01`, `SB-GEO-T02` |
| `SB-GEO-002` | Type every depth and reference datum | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T03`, `SB-GEO-T04` |
| `SB-GEO-003` | Integrate vertical stress from the physical anchor | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T05`, `SB-GEO-T06`, `SB-GEO-T07` |
| `SB-GEO-004` | Preserve measured and synthetic density provenance | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T08`, `SB-GEO-T09` |
| `SB-GEO-006` | Enforce every correlation's applicability contract | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T12`, `SB-GEO-T13` |
| `SB-GEO-007` | Never make a vendor table implementation truth | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T14` |
| `SB-GEO-008` | Resolve one shared water density | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T15`, `SB-GEO-T16` |
| `SB-GEO-009` | Anchor normal pressure at the water/formation boundary | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T17`, `SB-GEO-T18` |
| `SB-GEO-010` | Apply Terzaghi effective stress with explicit Biot alpha | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T19` |
| `SB-GEO-011` | Implement four distinct Eaton forms | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T20`, `SB-GEO-T21` |
| `SB-GEO-012` | Require readable trend inputs and output them | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T22` |
| `SB-GEO-013` | Calibrate Bowers before emission | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T23`, `SB-GEO-T24` |
| `SB-GEO-014` | Make Bowers unloading algebraically consistent | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T25`, `SB-GEO-T26` |
| `SB-GEO-015` | Block methods whose primary equation is missing | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T27` |
| `SB-GEO-016` | Apply one uniform pressure-limit policy | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T28`, `SB-GEO-T29` |
| `SB-GEO-017` | Emit pressure and gradient as separate typed curves | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T30` |
| `SB-GEO-018` | Implement the alpha-aware generalized fracture equation | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T31`, `SB-GEO-T32` |
| `SB-GEO-021` | Enforce the Matthews–Kelly overburden premise | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T35` |
| `SB-GEO-022` | Enforce declared source geography | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T12`, `SB-GEO-T36` |
| `SB-GEO-026` | Limit fracture pressure only by explicit policy | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T41` |
| `SB-GEO-028` | Name strains by stress direction | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T44` |
| `SB-GEO-030` | Require sourced dynamic-to-static transforms | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T47` |
| `SB-GEO-031` | Make modulus conversions dimensional | `PARTIAL` | `18_geomech-ppfg.md` | `SB-GEO-T48`, `SB-GEO-T49`, `SB-GEO-T50` |
| `SB-GEO-032` | Keep stress and stress gradient distinct | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T51` |
| `SB-GEO-034` | Assert total versus effective input state | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T54` |
| `SB-GEO-039` | Validate every stability input before solve | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T60` |
| `SB-GEO-040` | Bind strength correlations to native units | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T49`, `SB-GEO-T50`, `SB-GEO-T61` |
| `SB-GEO-041` | Use atan2 for every angle back-transform | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T62` |
| `SB-GEO-042` | Make unset sourced parameters block execution | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T63` |
| `SB-GEO-044` | Version local calibration without promoting it to default | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T65` |
| `SB-GEO-045` | Refuse extrapolation outside a declared range | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T66` |
| `SB-GEO-046` | Prohibit raster- and binary-only implementation truth | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T27`, `SB-GEO-T67` |
| `SB-GEO-049` | Keep shared parameters single-valued within a run | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T15`, `SB-GEO-T70` |
| `SB-INS-001` | Ship a qualified native Windows installer | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-003` | Publish truthful capability-level prerequisites | `PRESENT-DIVERGENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-004` | Maintain one dependency/capability manifest | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-005` | Resolve one interpreter with explainable precedence | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-008` | Support offline and managed deployment | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-010` | Separate immutable templates from user configuration | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-014` | Key parameters by semantic identifier and ordinal | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-015` | Refuse registry mismatch and ambiguity | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-016` | Use a canonical typed unit registry | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-023` | Gate releases on clean-machine scenarios | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-MIN-007` | Refuse a clay whose bound-water parameter is absent; never treat it as zero | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T07` |
| `SB-MIN-008` | Ship `CEC` and `WCLP` only as a matched pair from one library | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T08` |
| `SB-MIN-009` | Carry provenance on every endpoint value, not every endpoint column | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T09` |
| `SB-MIN-010` | Declare the wet/dry clay convention on every clay row and every clay curve | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T10`, `SB-MIN-T24` |
| `SB-MIN-011` | Declare the CEC unit and refuse implausible magnitudes | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T11` |
| `SB-MIN-027` | Store WCLP in v/v and refuse a p.u. value instead of switching route | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T27` |
| `SB-MIN-041` | Keep retired modules resolvable and refuse to run them, carrying no orphan defaults | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T40`, `SB-MIN-T41` |
| `SB-PLG-001` | Ship three independently gated domain units | `ABSENT` | `26_production-logging.md` | `SB-PLG-T01`, `SB-PLG-T02` |
| `SB-PLG-002` | Type every production unit at ingest | `ABSENT` | `26_production-logging.md` | `SB-PLG-T03`, `SB-PLG-T04`, `SB-PLG-T05` |
| `SB-PLG-003` | Calibrate spinner slopes from zonal averages | `ABSENT` | `26_production-logging.md` | `SB-PLG-T06`, `SB-PLG-T07`, `SB-PLG-T08` |
| `SB-PLG-004` | Compute apparent fluid velocity exactly | `ABSENT` | `26_production-logging.md` | `SB-PLG-T09`, `SB-PLG-T10` |
| `SB-PLG-006` | Stop before unsupported phase rates | `ABSENT` | `26_production-logging.md` | `SB-PLG-T12` |
| `SB-PLG-007` | Store sensor geometry per family and tool | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T13`, `SB-PLG-T14` |
| `SB-PLG-012` | Make Chronolog epochs and operation order explicit | `ABSENT` | `26_production-logging.md` | `SB-PLG-T20`, `SB-PLG-T21`, `SB-PLG-T22` |
| `SB-PLG-014` | Normalize nulls before station reduction | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T24` |
| `SB-PLG-016` | Bind cutoff polarity to measurement family | `ABSENT` | `26_production-logging.md` | `SB-PLG-T26`, `SB-PLG-T27` |
| `SB-PLG-017` | Implement logarithmic attenuation bond index | `ABSENT` | `26_production-logging.md` | `SB-PLG-T28`, `SB-PLG-T29` |
| `SB-PLG-018` | Name and require the bond interpolation method | `ABSENT` | `26_production-logging.md` | `SB-PLG-T30` |
| `SB-PLG-019` | Derive coverage from valid array width | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T31`, `SB-PLG-T32` |
| `SB-PLG-020` | Exclude collars without deleting data | `ABSENT` | `26_production-logging.md` | `SB-PLG-T33` |
| `SB-PLG-024` | Validate probability-term switches | `ABSENT` | `26_production-logging.md` | `SB-PLG-T39` |
| `SB-PLG-032` | Emit four named casing-loss quantities | `ABSENT` | `26_production-logging.md` | `SB-PLG-T48`, `SB-PLG-T49` |
| `SB-PLG-033` | Retain signed apparent loss | `ABSENT` | `26_production-logging.md` | `SB-PLG-T50` |
| `SB-PLG-035` | Require an ovality definition | `ABSENT` | `26_production-logging.md` | `SB-PLG-T52`, `SB-PLG-T53` |
| `SB-PLG-036` | Compute Barlow only from sourced strength | `ABSENT` | `26_production-logging.md` | `SB-PLG-T54`, `SB-PLG-T55` |
| `SB-PLG-037` | Source nominal casing geometry | `ABSENT` | `26_production-logging.md` | `SB-PLG-T56` |
| `SB-PLG-042` | Refuse untracked environmental correction | `ABSENT` | `26_production-logging.md` | `SB-PLG-T61` |
| `SB-PLG-043` | Canonicalize casing weight and tension | `ABSENT` | `26_production-logging.md` | `SB-PLG-T62`, `SB-PLG-T63` |
| `SB-PLG-045` | Stamp full run provenance | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T65` |
| `SB-PLG-046` | Separate computed, imported and interpreted identities | `ABSENT` | `26_production-logging.md` | `SB-PLG-T66` |
| `SB-PLG-048` | Preserve array width and per-row validity end to end | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T31`, `SB-PLG-T68` |
| `SB-PLT-001` | Persist semantic intent and concrete resolution separately | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-002` | Resolve axes through one explicit precedence chain | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-003` | Overlay compatibility is quantity-and-unit typed | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-004` | Valid and display ranges remain distinct | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-005` | Unit-limit content is audited before activation | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-006` | One canonical histogram-bin contract | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-008` | Percentile probability and range position are different types | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-011` | Pickett states what is and is not identifiable | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-013` | Missing and out-of-range policy is channel-specific | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-014` | Multi-well allocation follows finite-pair screening | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-015` | Decimation preserves pairing, endpoints and provenance | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-016` | Depth-step reconciliation is explicit and conservative | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-020` | Plot-derived parameter writes carry full provenance | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-023` | Every rendered chart is provenance-complete | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-031` | No silent record truncation | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-032` | Plot performance is gated on declared hardware | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-POR-003` | Every porosity method **MUST** emit a per-sample **branch and limit flag** stream (working name `PHIFLAG`) recording which branch produced the sample and which limit, if any, bound it. IP is the only incumbent that publishes such a stream (F17, F21 — codes 6, 7, 9, 16), and SandiBumi currently has **none** (§3.0). Every clamp identified in §3 — `VSH ≥ 0.95`, the `phie_max·(1−VSH)` ceiling that binds at 0.24 in §3.2's worked case, the `[1.95, 3.0]` and `[−0.015, 0.40]` shale-reduction clamps, `phi_son`'s `[0, 1]` — currently fires **silently**. This is the single cheapest fail-loud-where-they-fail-silent win in the domain. |  | `11_porosity.md` |  |
| `SB-POR-004` | The mnemonic dictionary **MUST** carry a **porosity family** (`PHIE`, `PHIT`, `PHIA`, `DPHI`, `NPHI_COR`, `PHIE_LIM`, …) and each porosity curve **MUST** carry the method and the volume convention that produced it. Two porosity modules **MUST NOT** write the same output mnemonic: `phi_den` and `phi_dn` both write `PHIE`/`PHIT` today, so the second run silently overwrites the first (§3.4). `curves.rs:21-37` registers fourteen families and no porosity family at all. Without this, F16's "PhiT is not one quantity" is unrepresentable and an imported Techlog `PhiT` resolves to a computed Geolog-convention `PHIT` by name collision. |  | `11_porosity.md` |  |
| `SB-POR-006` | Every porosity method that consumes a shale/clay volume **MUST** consume a **typed** volume and **MUST** refuse an untyped one. A `VSH` (shale-endpoint) and a `VCL` (wet-clay-endpoint) volume are not interchangeable (F15); the endpoint subtracted must match the volume supplied. The refusal is the requirement — silently accepting either is how a 100 %-shale-point correction gets applied to a clay volume. |  | `11_porosity.md` |  |
| `SB-POR-008` | Clay-bound-water porosity **MUST** be defined once as `PHIT_SH = (RHO_DSH − RHO_SH)/(RHO_DSH − RHO_W)` with `RHO_W` the **formation water** density, in one shared helper, and **MUST** be exported to the `CLY` chapter's `clsr_porosity_corrected` (SB-CLY-044). The shared helper exists and uses the correct fluid (`modules.rs:705-710`); the requirement pins it and publishes it across the seam. The **shale-subtraction** term `(RHO_MA − RHO_SH)/(RHO_MA − RHO_FL)` is a **different quantity** and **MUST NOT** share a name with it (F16). |  | `11_porosity.md` |  |
| `SB-POR-013` | The **shale-correction convention** **MUST** be an explicit, named, per-method selection — `NORMALISED` (Geolog: reduce, floor, then rescale by `1 − VSH`) or `SUBTRACTIVE` (Techlog: one pre-correction, answer already effective) — and one method **MUST NOT** mix them. `phi_son`'s `RHG` branch mixes them today: a normalise-convention transform paired with a Wyllie subtractive shale term (`modules.rs:907` + `:915`, §3.3). The convention is worth **1.30–1.55 p.u.** across vendors on identical inputs (F2) and is invisible in every parameter value. |  | `11_porosity.md` |  |
| `SB-POR-014` | Sonic porosity methods **MUST** be named for what they compute and **MUST NOT** be named for a method they are not. Specifically: the branch at `modules.rs:907` **MUST** be renamed from `RHG (Raymer-Hunt-Gardner)` to `FIELD_OBSERVED`, its coefficient 0.625 **MUST** become a parameter, and any method offered as "Raymer-Hunt-Gardner" **MUST** be one of the three published renderings in F3 with its vendor identified. Shipping IP's recommended-over-Wyllie method name against a different transform is the kind of overclaim CONTRACT §5 warns costs the deal. |  | `11_porosity.md` |  |
| `SB-POR-016` | Matrix transit time **MUST** be selected per lithology from a cited family, and SandiBumi **MUST NOT** ship a single lithology-agnostic default. Techlog's `DTma 47.5` applied to a clastic section moves Wyllie porosity by **4.5 p.u.** against a sandstone value (F1). The sandstone family itself spans 1.65 p.u. across four cited vendor values and therefore ships as a **cited choice list, not a number** (§5). |  | `11_porosity.md` |  |
| `SB-POR-021` | SandiBumi **MUST** implement a **chart-free analytic neutron-density crossplot** as its primary N-D porosity method, following the Bateman & Konen (1977) family that Geolog's `phi_dnbk` implements and that Techlog's neutron-sonic algorithm independently reproduces in structure (F13). This is the method that lets SandiBumi ship a real crossplot porosity without transcribing a single vendor chart value, and it is what §3.2's arithmetic average is standing in for at a cost of **1.64–1.79 p.u.** |  | `11_porosity.md` |  |
| `SB-POR-023` | The arithmetic average `(φD + φN)/2` and the RMS `sqrt((φD² + φN²)/2)` **MUST NOT** be presented as crossplot porosity methods, and the doc string at `modules.rs:770-771` claiming they are *"the standard analytic equivalent"* of chart lookups **MUST** be removed. They **MAY** ship as explicitly labelled quick-look comparison curves. **No vendor ships either as a porosity method** (F14), and IP states of the field shortcuts verbatim that *"they should not be used for anything other than this"*. |  | `11_porosity.md` |  |
| `SB-POR-024` | N-D crossplot porosity **MUST** refuse to run on a neutron curve whose **matrix units** are not declared, and **MUST** state the declared basis in its output provenance. A limestone-unit neutron against a sandstone matrix reads **~0.04 v/v low in clean water sand** — a fact `condflag`'s doc string already states verbatim (`modules.rs:1261-1264`) and `phi_dn` neither states nor checks (§3.6). All three vendors solve this with chart data; SandiBumi solves it with `nphimat` and must then require it. |  | `11_porosity.md` |  |
| `SB-POR-029` | The apparent hydrocarbon **electron density** **MUST** be the *Conventional* form, and its validity envelope **MUST** be stated in the product: it tracks the Gaymard-Poupon quadratic to better than **1.5 %** for `ρ_h ≥ 0.225 g/cc` and degrades monotonically to **−3.1 % at 0.10** (F10). IP's *Modified* form gives **0.0761 vs 0.2452 g/cc at ρ_h = 0.20** — a factor 3.22 — and IP's own two modules disagree about which to use. |  | `11_porosity.md` |  |
| `SB-POR-030` | The hydrocarbon **hydrogen index** on the neutron side **MUST** be the Gaymard-Poupon quadratic `N_h = 0.15 + 0.2(0.9 − ρ_h)²`, corroborated by Techlog's `9ρN_h` to **1.2 %** and by Poupon's own Eq A-9 to **1.5 %** at gas density. Geolog's `α = 1.67ρ − 0.17` is **1.51× Poupon's gas value** and over-corrects `NPHI` by **+4.1 p.u.** (F11); it is a fix Geolog made on its density side and never propagated to its neutron side, and **MUST NOT** be adopted. |  | `11_porosity.md` |  |
| `SB-POR-033` | The hydrocarbon chain **MUST** refuse or hard-flag samples outside the validity bounds of the selected model, specifically: `ρ_h < 0.1414 g/cc` (IP Modified goes negative), `ρ_h < 0.1018 g/cc` (Geolog `α` goes negative), and `ρ_h < 0.188 g/cc` (any `N_h` exceeding methane's hydrogen mass fraction `4 × 1.008 / 16.04 = 0.2514`, which is stoichiometry, not a parameter). **Dry gas at shallow-to-moderate reservoir pressure sits inside that band routinely** (F9), and a negative apparent density biases density porosity **low** exactly where the correction matters most. This is the most consequential fail-loud requirement in the chapter. |  | `11_porosity.md` |  |
| `SB-POR-035` | The flushed-zone saturation exponent (`Sxo = Swe^n`) **MUST** ship with **no default** and **MUST** be an explicit user decision. Geolog defaults **0.2** and Techlog/IP default **1** — at `Swe = 0.30` that is `Sxo = 0.786` versus `0.300`, a **0.49 difference in `Sxo`** feeding every hydrocarbon correction, with no parameter ever out of range (F12). These are opposite modelling assumptions, not a tolerance. |  | `11_porosity.md` |  |
| `SB-POR-055` | Every petrophysical parameter in this domain **MUST** carry a source string and tier, and where the held sources disagree with no defensible adjudication the parameter **MUST** ship `ABSENT — ships with no default` with the competing values visible. This is a standing project decision. It applies immediately to `RHO_SH`, `RHO_DSH`, `NPHI_SH`, `DT_SH` and `RHO_MA`, all of which ship today as uncited numbers (§3.1) — and `RHO_DSH = 2.65` matches **no held source at all** while setting `PHIT_SH` a factor **1.73 low** against the nearest vendor. For Techlog specifically, neither its script nor its doc may be treated as authoritative alone: **nine** shipped quantities disagree between the two, including two values inside one equation (F23). |  | `11_porosity.md` |  |
| `SB-POR-058` | A module **MUST NOT** present a parameter its computation does not read. `sspw_spec` declares `NPHI_MAT`, `NPHI_SH` and `NPHI_FL` (`ssc.rs:370`, `:372`, `:377`) and `sspw()` reads none of them (§3.8). Until the re-port against `sspw.lls` is signed off, those parameters **MUST** be removed from the spec or marked inactive in the dialog. An honest module header (`ssc.rs:37-41`) is invisible to `moduleDialog.ts`; a user who tunes `NPHI_SH` and sees no change has been told a falsehood by the UI. |  | `11_porosity.md` |  |
| `SB-POR-059` | `sspw()`'s gas conditioning **MUST** be brought to the same RMS midpoint `sqrt((φD² + NPHI²)/2)` that `ssc()` uses. `ssc.rs:433` still runs the weight that `ssc.rs:172-178` records as *inverting the D-N crossover* and that was fixed in `ssc()` on 2026-07-29. At `φD = 0.25, NPHI = 0.10` the two shipped modules return **0.1903943** and **0.1431782** — **4.72 p.u. apart, with `sspw` biased low in gas**, the direction that under-reports pay. |  | `11_porosity.md` |  |
| `SB-RPH-001` | Use one typed SI-with-GPa elastic state | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T01`, `SB-RPH-T02`, `SB-RPH-T03` |
| `SB-RPH-003` | Implement guarded Gassmann forward and inverse | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T06`, `SB-RPH-T07`, `SB-RPH-T08` |
| `SB-RPH-005` | Persist method, state and failure provenance | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T10`, `SB-RPH-T11` |
| `SB-RPH-010` | Govern elastic endpoints without copying vendor tables | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T17`, `SB-RPH-T18` |
| `SB-RPH-011` | Keep critical and depositional porosity distinct | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T19` |
| `SB-RPH-014` | Require Hertz–Mindlin adhesion explicitly | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T22`, `SB-RPH-T23` |
| `SB-RPH-023` | Make SH/SV assignment an explicit measured decision | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T34`, `SB-RPH-T35` |
| `SB-RPH-025` | Reject non-positive-definite stiffness states | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T37`, `SB-RPH-T38` |
| `SB-RPH-032` | Make image geometry corrections reversible | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T46`, `SB-RPH-T47` |
| `SB-RPH-035` | Prevent magnetic-declination double application | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T51`, `SB-RPH-T52` |
| `SB-RPH-041` | Refuse metadata-free fracture outputs | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T60`, `SB-RPH-T61` |
| `SB-RPH-051` | Persist enough provenance to reproduce every number | `PARTIAL` | `25_fluidsub-rockphysics.md` | `SB-RPH-T10`, `SB-RPH-T76` |
| `SB-SAT-001` | Name every saturation model by its equation, never by a vendor adjective | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T01`, `SB-SAT-T02`, `SB-SAT-T30` |
| `SB-SAT-002` | Ship effective and total Archie as separate named methods | `ABSENT` | `12_saturation.md` | `SB-SAT-T03`, `SB-SAT-T04` |
| `SB-SAT-010` | Juhász MUST flag a negative excess-conductivity coefficient | `ABSENT` | `12_saturation.md` | `SB-SAT-T15` |
| `SB-SAT-012` | `B` MUST be a unit-typed quantity, canonically `L·S/(eq·m)` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T17`, `SB-SAT-T18` |
| `SB-SAT-013` | `Qv` MUST be unit-typed, canonically meq/mL | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T19`, `SB-SAT-T20` |
| `SB-SAT-014` | `B(T,Rw)` MUST consume typed °C and clamp `B ≥ 0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T21`, `SB-SAT-T22` |
| `SB-SAT-028` | Non-convergence MUST return null, never a partial iterate | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T41` |
| `SB-SAT-031` | `Rw` ships with no default | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T45` |
| `SB-SAT-034` | `a`, `m`, `n`, `m*`, `n*` ship with no default | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T49` |
| `SB-SAT-035` | `Rsh` and `φt_sh` ship with no default, and the current values are withdrawn | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T50` |
| `SB-SAT-038` | Every parameter carries a source string, and the build fails without one | `ABSENT` | `12_saturation.md` | `SB-SAT-T31` |
| `SB-SAT-043` | A saturation result carries its parameters, their sources and their papers | `ABSENT` | `12_saturation.md` | `SB-SAT-T59` |
| `SB-SAT-047` | One model, one number, whichever engine computes it | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T30`, `SB-SAT-T61` |
| `SB-SHR-001` | **Every** height-domain saturation model MUST convert the height above the free-water level into the unit in which that model's own coefficients are defined, and the conversion MUST be driven by the project's declared depth unit. No branch of any saturation-height model may consume a raw height. Adding a new model family MUST NOT be possible without declaring the unit its length-dimensioned coefficients are in. | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T01`, `SB-SHR-T02` |
| `SB-SHR-002` | Every shape parameter carrying a length dimension — Skelt-Harrison `B` and `D`, Thomeer entry height `Hd`, Brooks-Corey entry height `He`, and the free-water level itself — MUST carry an explicit unit in its registration, and the product MUST re-express its value when the project depth unit changes. A length-dimensioned parameter with a hard-coded unit string is a defect. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T03` |
| `SB-SHR-003` | The domain MUST refuse to perform height arithmetic when the project depth unit is undeclared, and MUST say so by name. It MUST NOT substitute a default unit. This requirement binds the domain's own entry points; the carrier's parse-time behaviour is `21_data-io.md`'s. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T04` |
| `SB-SHR-005` | Water density, hydrocarbon density and reservoir `σ·cosθ` MUST each have **exactly one** default in the product, shared by the fitting path and the forward-apply path. Where a fit and an apply can disagree on a fluid property, the product MUST refuse the apply rather than compute it. | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T06` |
| `SB-SHR-009` | Every fitted object this domain produces — saturation-height law (pooled and per rock type), Thomeer fit, HFU cluster model, Lorenz flow-unit partition — MUST be persisted as a first-class, named, versioned object. Each MUST carry its training provenance: the wells, the log set and curve versions, the sample count, the full exclusion ledger, the fluid properties and FWL in force, the fitting method, and the fit-quality statistic. An object that cannot state what it was fitted on MUST NOT be applicable. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T10`, `SB-SHR-T11` |
| `SB-SHR-010` | The forward-apply path MUST consume a **stored fitted object**, not hand-entered coefficients. Where a user overrides a stored coefficient, the applied result MUST record the override and the value it replaced. Hand transcription of a fit into a module parameter MUST NOT be the supported workflow. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T12` |
| `SB-SHR-011` | The free-water level MUST be a **first-class uncertain parameter, not a scalar input.** The FWL scan MUST be mandatory output, MUST report an uncertainty interval alongside its optimum — the range of candidate levels whose residual is statistically indistinguishable from the minimum — and MUST NOT present the argmin alone. Every saturation-height result MUST carry a **per-zone FWL confidence** alongside its fit statistic, and a fit whose FWL cannot be constrained MUST say so rather than report a coefficient set. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T13` |
| `SB-SHR-012` | A Brooks-Corey fit MUST declare its exponent convention explicitly and MUST emit **both** `λ` and `N = 1/λ`, each labelled. Import or export of a Brooks-Corey coefficient without a declared convention MUST be refused. | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T14` |
| `SB-SHR-013` | A Thomeer fit MUST declare the logarithm base of its shape factor `G` and MUST emit both the base-10 (`G`) and natural-log (`2.302585·G`) forms, each labelled. A `G` imported without a declared base MUST be refused. | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T15` |
| `SB-SHR-014` | The product MUST NOT ship a default apex basis for Swanson permeability. The bulk-volume basis (fraction, percent, or pore-volume-normalised saturation) MUST be an explicit, named user choice with no default, the chosen basis MUST travel with every Swanson result, and the coefficient pair MUST carry its own source. Until a basis is chosen, Swanson permeability MUST be `MISSING`, not computed. | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T16`, `SB-SHR-T17` |
| `SB-SHR-017` | Every correlation in this domain that regresses on `log φ` MUST enforce its porosity **unit** as a precondition, not document it. A porosity input outside the declared unit's valid range MUST fail the run and name the curve; it MUST NOT be silently skipped. | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T20` |
| `SB-SHR-022` | Every module in this domain MUST carry the exclusion ledger the fitting path already implements: a named reason and a count for every sample not computed, returned with the result and persisted with any curve it wrote. A curve with materially reduced coverage MUST state why. | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T25` |
| `SB-SHR-040` | **No numerical result in this domain may depend on a display setting.** A closure pick, an entry pressure, a fitted coefficient or a rock class MUST be identical whether an axis is drawn linear or logarithmic, whether a plot is open, and whatever the current zoom or theme. Where a method genuinely requires a log-domain pick, the log domain MUST be a property of the **method**, not of the plot. | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T42` |
| `SB-TBD-006` | One name, one equation: the picker and the module MUST implement the same construction | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T05`, `SB-TBD-T06` |
| `SB-TBD-007` | Never clamp a derived volume fraction or a derived sand porosity | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T07`, `SB-TBD-T08` |
| `SB-TBD-009` | Retire `PHIE_LAM`; emit `PHIT_SS` and `PHIE_SS` as distinct curves | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T10`, `SB-TBD-T11` |
| `SB-TBD-066` | Withdraw the two uncited endpoint defaults | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T23`, `SB-TBD-T24` |
| `SB-TOC-001` | Make TOC a unit-tagged wt% quantity | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T01`, `SB-TOC-T04` |
| `SB-TOC-002` | Ship no numeric delta-log-R baseline | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T14`, `SB-TOC-T15` |
| `SB-TOC-005` | Apply the cited LOM cap with a flag | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T11`, `SB-TOC-T13` |
| `SB-TOC-006` | Treat background TOC as part of the baseline pick | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T09`, `SB-TOC-T14`, `SB-TOC-T15` |
| `SB-TOC-008` | Mask invalid geology and borehole conditions before TOC | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T22`, `SB-TOC-T23` |
| `SB-TOC-010` | Carry TOC method and pick provenance | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T57` |
| `SB-TOC-015` | Keep the 1.10 kerogen endpoint and correct its provenance | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T19`, `SB-TOC-T20` |
| `SB-TOC-019` | Make measured gas inputs required and sourced | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T30`, `SB-TOC-T32` |
| `SB-TOC-020` | Couple Langmuir capacity to the matching organic input | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T06`, `SB-TOC-T07`, `SB-TOC-T31` |
| `SB-TOC-023` | Name the Bg standard condition | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T34` |
| `SB-TOC-025` | Reserve gas-content and gas-in-place names by quantity | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T40` |
| `SB-TOC-031` | Refuse static moduli with dynamic endpoints | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T47` |
| `SB-TOC-040` | Persist visual picks as computation parameters | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T15`, `SB-TOC-T56` |
| `SB-TOC-041` | Keep visualization and compute equations identical | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T06`, `SB-TOC-T56` |
| `SB-TOC-042` | Emit stable unconventional QC flags | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T57` |
| `SB-TOC-043` | Migrate existing projects without changing meanings silently | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T58` |
<!-- OPEN_P0_ROWS -->

## Roll-ups

**Total: 931 requirements.**

### By priority

| Priority | Count |
|---|---:|
| `P0` | 266 |
| `P1` | 401 |
| `P2` | 199 |
| `P3` | 46 |
| `P4` | 4 |
| Not stated by chapter | 15 |
| **Total** | **931** |

### By status

| Status | Count |
|---|---:|
| `ABSENT` | 507 |
| `PARTIAL` | 124 |
| `PRESENT-OK` | 113 |
| `PRESENT-DIVERGENT` | 111 |
| `PRESENT-UNVERIFIED` | 12 |
| `UNMEASURED` *(not defined by CONTRACT.md)* | 1 |
| `ABSENT — designed, parked` *(not defined by CONTRACT.md)* | 1 |
| Not stated by chapter | 62 |
| **Total** | **931** |

### Priority × status

| Priority | ABSENT | PARTIAL | PRESENT-OK | PRESENT-DIVERGENT | PRESENT-UNVERIFIED | UNMEASURED | ABSENT — designed, parked | Status not stated | Total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `P0` | 119 | 37 | 31 | 59 | 3 | 0 | 0 | 17 | 266 |
| `P1` | 206 | 57 | 62 | 43 | 7 | 1 | 0 | 25 | 401 |
| `P2` | 128 | 24 | 18 | 9 | 2 | 0 | 1 | 17 | 199 |
| `P3` | 41 | 1 | 1 | 0 | 0 | 0 | 0 | 3 | 46 |
| `P4` | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 |
| Priority not stated | 9 | 5 | 1 | 0 | 0 | 0 | 0 | 0 | 15 |
| **Total** | **507** | **124** | **113** | **111** | **12** | **1** | **1** | **62** | **931** |

## Tier −1 candidates for Jauhar's review

These are candidates only. None is added to the Tier −1 list here. Each is an explicitly `P0`
requirement (or a `P0` in the status-less porosity chapter) whose chapter describes a shipped path
that can produce a wrong number rather than merely a missing capability.

- `SB-CLY-009` — `10_clay-volume.md`: hard-coded transform bounds include an inward-rounded
  Clavier bound and cannot remain correct when Stieber's shape parameter changes.
- `SB-CLY-050` — `10_clay-volume.md`: the shipped density-neutron shale endpoints produce an
  uncited third answer between vendor values that already differ by 41.5% relative.
- `SB-CUT-031` — `14_cutoffs-summation-mc.md`: all twelve IP-seeded Gaussian priors are passed as σ
  at twice the width supported by the cited IP convention.
- `SB-CUT-041` — `14_cutoffs-summation-mc.md`: accumulation uses limited curves, biasing a wet-zone
  mean toward hydrocarbon by about four saturation units at the cited σ example.
- `SB-ENV-006` — `20_envcorr-qc.md`: a missing correction input can pass an uncorrected curve under
  a name that asserts the correction was applied.
- `SB-ENV-024` — `20_envcorr-qc.md`: one shipped bad-hole threshold is half every delivered-study
  precedent held by the chapter, changing which samples survive QC.
- `SB-ENV-025` — `20_envcorr-qc.md`: the 8.5 in bit-size default can make a real 6 in washed-out
  hole pass the caliper gate cleanly.
- `SB-ENV-044` — `20_envcorr-qc.md`: one shipped temperature path evaluates geothermal gradient on
  measured depth while another uses TVDSS, so deviation changes the computed temperature.
- `SB-ENV-045` — `20_envcorr-qc.md`: a bare geothermal-gradient number can be accepted under the
  wrong compound unit and produce a smooth, essentially isothermal well.
- `SB-MIN-008` — `13_mineral-solver.md`: the shipped clay library mixes `CEC` and `WCLP` sources
  even though they parameterize the same bound-water quantity.
- `SB-MIN-027` — `13_mineral-solver.md`: a wet-clay porosity unit mistake can silently switch the
  solver to a different bound-water route instead of refusing the value.
- `SB-POR-013` — `11_porosity.md`: `phi_son` mixes normalized and subtractive shale conventions,
  worth 1.30–1.55 porosity units on the chapter's identical-input comparison.
- `SB-POR-021` — `11_porosity.md`: the shipped arithmetic-average N-D shortcut stands in for an
  analytic crossplot solution at a cost of 1.64–1.79 porosity units.
- `SB-POR-024` — `11_porosity.md`: `phi_dn` neither declares nor checks neutron matrix units, which
  can read about 0.04 v/v low in clean water sand.
- `SB-POR-035` — `11_porosity.md`: the competing shipped flushed-zone exponents give `Sxo` values
  0.49 apart at the chapter's `Swe = 0.30` case.
- `SB-POR-055` — `11_porosity.md`: the shipped dry-shale density matches no held source and drives
  `PHIT_SH` a factor 1.73 low against the nearest vendor value.
- `SB-POR-059` — `11_porosity.md`: the shipped SSPW gas weighting is 4.72 porosity units below SSC
  on the same gas-sand inputs, in the direction that under-reports pay.
- `SB-SAT-012` — `12_saturation.md`: the same Waxman-Smits `B` quantity remains expressible on two
  scales separated by a factor of 100.
- `SB-SAT-013` — `12_saturation.md`: incompatible `Qv` and `CEC` unit conventions remain
  representable as raw numbers in the saturation path.
- `SB-SAT-028` — `12_saturation.md`: a non-converged iterative saturation path can return its last
  partial iterate as if it were an answer.
- `SB-SAT-031` — `12_saturation.md`: `Rw` still ships as a numeric default even though the chapter
  requires a sourced, study-specific value.
- `SB-SAT-034` — `12_saturation.md`: `a`, `m`, `n`, `m*`, and `n*` still ship defaults that can
  silently determine saturation without a cited study choice.
- `SB-SAT-035` — `12_saturation.md`: current `Rsh` and shale-porosity defaults remain active despite
  the requirement withdrawing them.
- `SB-SHR-005` — `15_sat-height-rocktyping.md`: fit and forward-apply paths use different
  hydrocarbon-density defaults, moving the chapter's round trip by 5.3 saturation units.
- `SB-SHR-014` — `15_sat-height-rocktyping.md`: Swanson permeability computes under an undeclared
  apex basis, so the same coefficient pair can mean different numbers.
- `SB-TBD-006` — `17_thinbed-laminated.md`: the picker and module expose one method name while
  implementing different constructions.
- `SB-TBD-007` — `17_thinbed-laminated.md`: derived laminated volume and porosity are clamped before
  the result is reported, turning out-of-domain arithmetic into plausible rock volumes.
- `SB-TBD-066` — `17_thinbed-laminated.md`: two uncited endpoint defaults remain in live thin-bed
  calculations whose outputs still plot and sum normally.
- `SB-TOC-002` — `19_toc-unconventional.md`: a numeric delta-log-R baseline ships where the chapter
  requires a picked, sourced baseline.
- `SB-TOC-005` — `19_toc-unconventional.md`: the cited LOM validity cap is not enforced with a flag,
  allowing the current path to extrapolate a plausible TOC number.
- `SB-TOC-006` — `19_toc-unconventional.md`: background TOC is omitted from the baseline pick,
  changing the computed overlay separation.
- `SB-TOC-019` — `19_toc-unconventional.md`: measured gas inputs that should be required can be
  replaced by shipped values and still produce a clean gas-content result.
- `SB-TOC-025` — `19_toc-unconventional.md`: an intensive gas-content quantity is exposed under a
  gas-in-place name despite having no area or thickness term.
- `SB-TOC-041` — `19_toc-unconventional.md`: visualization and compute paths use different
  equations for the same interpreted quantity.

## Requirements with no owned acceptance test — 137

The list below is stricter than “a test exists somewhere in the chapter.” It identifies
requirements whose own **Verified by** field is empty, which is the gap `SB-CORE-040` must close.

| ID | Priority | Status | Chapter |
|---|---|---|---|
| `SB-CORE-003` | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-005` | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-012` | `P2` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-013` | `P2` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-030` | `P1` | `UNMEASURED` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-031` | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-032` | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-033` | `P2` | `ABSENT — designed, parked` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-034` | `P2` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-035` | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-036` | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-042` | `P1` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-043` | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |
| `SB-CORE-044` | `P1` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` |
| `SB-INS-001` | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-002` | `P0` | `PRESENT-OK` | `27_ip-install-blockers.md` |
| `SB-INS-003` | `P0` | `PRESENT-DIVERGENT` | `27_ip-install-blockers.md` |
| `SB-INS-004` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-005` | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-006` |  | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-007` |  | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-008` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-009` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-010` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-011` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-012` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-013` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-014` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-015` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-016` | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-017` |  | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-018` |  | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-019` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-020` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-021` |  | `PARTIAL` | `27_ip-install-blockers.md` |
| `SB-INS-022` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-023` | `P0` | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-024` |  | `PRESENT-OK` | `27_ip-install-blockers.md` |
| `SB-INS-025` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-INS-026` |  | `ABSENT` | `27_ip-install-blockers.md` |
| `SB-PLT-001` | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-002` | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |
| `SB-PLT-003` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-004` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-005` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-006` | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |
| `SB-PLT-007` | `P1` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-008` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-009` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-010` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-011` | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |
| `SB-PLT-012` | `P1` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-013` | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-014` | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |
| `SB-PLT-015` | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-016` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-017` | `P1` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-018` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-019` | `P1` | `PRESENT-OK` | `23_plotting-interactivity.md` |
| `SB-PLT-020` | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |
| `SB-PLT-021` | `P2` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-022` | `P2` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-023` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-024` | `P0` | `PRESENT-OK` | `23_plotting-interactivity.md` |
| `SB-PLT-025` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-026` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-027` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-028` | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-029` | `P0` | `PRESENT-OK` | `23_plotting-interactivity.md` |
| `SB-PLT-030` | `P1` | `PRESENT-OK` | `23_plotting-interactivity.md` |
| `SB-PLT-031` | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-PLT-032` | `P0` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-033` | `P1` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-034` | `P2` | `ABSENT` | `23_plotting-interactivity.md` |
| `SB-PLT-035` | `P2` | `PARTIAL` | `23_plotting-interactivity.md` |
| `SB-POR-001` | `P1` |  | `11_porosity.md` |
| `SB-POR-002` | `P1` |  | `11_porosity.md` |
| `SB-POR-003` | `P0` |  | `11_porosity.md` |
| `SB-POR-004` | `P0` |  | `11_porosity.md` |
| `SB-POR-005` | `P1` |  | `11_porosity.md` |
| `SB-POR-006` | `P0` |  | `11_porosity.md` |
| `SB-POR-007` | `P1` |  | `11_porosity.md` |
| `SB-POR-008` | `P0` |  | `11_porosity.md` |
| `SB-POR-009` | `P1` |  | `11_porosity.md` |
| `SB-POR-010` | `P2` |  | `11_porosity.md` |
| `SB-POR-011` | `P1` |  | `11_porosity.md` |
| `SB-POR-012` | `P2` |  | `11_porosity.md` |
| `SB-POR-013` | `P0` |  | `11_porosity.md` |
| `SB-POR-014` | `P0` |  | `11_porosity.md` |
| `SB-POR-015` | `P1` |  | `11_porosity.md` |
| `SB-POR-016` | `P0` |  | `11_porosity.md` |
| `SB-POR-017` | `P1` |  | `11_porosity.md` |
| `SB-POR-018` | `P1` |  | `11_porosity.md` |
| `SB-POR-019` | `P2` |  | `11_porosity.md` |
| `SB-POR-020` | `P2` |  | `11_porosity.md` |
| `SB-POR-021` | `P0` |  | `11_porosity.md` |
| `SB-POR-022` | `P1` |  | `11_porosity.md` |
| `SB-POR-023` | `P0` |  | `11_porosity.md` |
| `SB-POR-024` | `P0` |  | `11_porosity.md` |
| `SB-POR-025` | `P1` |  | `11_porosity.md` |
| `SB-POR-026` | `P2` |  | `11_porosity.md` |
| `SB-POR-027` | `P2` |  | `11_porosity.md` |
| `SB-POR-028` | `P1` |  | `11_porosity.md` |
| `SB-POR-029` | `P0` |  | `11_porosity.md` |
| `SB-POR-030` | `P0` |  | `11_porosity.md` |
| `SB-POR-031` | `P1` |  | `11_porosity.md` |
| `SB-POR-032` | `P2` |  | `11_porosity.md` |
| `SB-POR-033` | `P0` |  | `11_porosity.md` |
| `SB-POR-034` | `P1` |  | `11_porosity.md` |
| `SB-POR-035` | `P0` |  | `11_porosity.md` |
| `SB-POR-036` | `P2` |  | `11_porosity.md` |
| `SB-POR-037` | `P2` |  | `11_porosity.md` |
| `SB-POR-038` | `P1` |  | `11_porosity.md` |
| `SB-POR-039` | `P1` |  | `11_porosity.md` |
| `SB-POR-040` | `P2` |  | `11_porosity.md` |
| `SB-POR-041` | `P2` |  | `11_porosity.md` |
| `SB-POR-042` | `P3` |  | `11_porosity.md` |
| `SB-POR-043` | `P1` |  | `11_porosity.md` |
| `SB-POR-044` | `P1` |  | `11_porosity.md` |
| `SB-POR-045` | `P1` |  | `11_porosity.md` |
| `SB-POR-046` | `P2` |  | `11_porosity.md` |
| `SB-POR-047` | `P1` |  | `11_porosity.md` |
| `SB-POR-048` | `P1` |  | `11_porosity.md` |
| `SB-POR-049` | `P2` |  | `11_porosity.md` |
| `SB-POR-050` | `P1` |  | `11_porosity.md` |
| `SB-POR-051` | `P1` |  | `11_porosity.md` |
| `SB-POR-052` | `P2` |  | `11_porosity.md` |
| `SB-POR-053` | `P1` |  | `11_porosity.md` |
| `SB-POR-054` | `P1` |  | `11_porosity.md` |
| `SB-POR-055` | `P0` |  | `11_porosity.md` |
| `SB-POR-056` | `P2` |  | `11_porosity.md` |
| `SB-POR-057` | `P2` |  | `11_porosity.md` |
| `SB-POR-058` | `P0` |  | `11_porosity.md` |
| `SB-POR-059` | `P0` |  | `11_porosity.md` |
| `SB-POR-060` | `P2` |  | `11_porosity.md` |
| `SB-POR-061` | `P3` |  | `11_porosity.md` |
| `SB-POR-062` | `P3` |  | `11_porosity.md` |
<!-- NO_TEST_ROWS -->

## Consolidated requirements

Sorted by domain prefix and then numeric ID.

| ID | Title | Priority | Status | Chapter | Verified by |
|---|---|---|---|---|---|
| `SB-CLY-001` | Refuse and flag degenerate endpoints, never null silently | `P0` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T01`, `SB-CLY-T24`, `SB-CLY-T32` |
| `SB-CLY-002` | Stieber as one generic shape parameter | `P1` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T05`, `SB-CLY-T06`, `SB-CLY-T07` |
| `SB-CLY-003` | Resolve vendor Stieber labels by alias, fail the import if unresolvable | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T12`, `SB-CLY-T13` |
| `SB-CLY-004` | Larionov in the exact normalised form | `P1` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T02`, `SB-CLY-T04` |
| `SB-CLY-005` | Keep the decimal Larionov reachable, for parity only | `P2` | `PRESENT-OK` | `10_clay-volume.md` | `SB-CLY-T03` |
| `SB-CLY-006` | `LARINOV3` warns that it has no published provenance | `P1` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T14` |
| `SB-CLY-007` | Clavier over its full analytic domain | `P2` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T08`, `SB-CLY-T09` |
| `SB-CLY-008` | Implement the Curved transform | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T11` |
| `SB-CLY-009` | Domain clamps computed from transform parameters | `P0` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T07`, `SB-CLY-T08` |
| `SB-CLY-010` | A clamped sample is marked as clamped | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T10` |
| `SB-CLY-011` | SP indicator | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T15` |
| `SB-CLY-012` | Three neutron single-indicator forms, no default | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T16` |
| `SB-CLY-013` | Limestone-matrix precondition on neutron indicators | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T17` |
| `SB-CLY-014` | Two-sided warning on the neutron clean endpoint | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T18` |
| `SB-CLY-015` | Four resistivity forms, no default | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T19` |
| `SB-CLY-016` | Validate `R_clay < R_clean` before branching | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T20` |
| `SB-CLY-017` | Cite Coriband where the Coriband form is used | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-018` | One canonical bilinear form for every double indicator | `P1` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T21` |
| `SB-CLY-019` | Two-point clean line with an explicit constructor | `P1` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T22` |
| `SB-CLY-020` | Linkage semantics: `c1` linkable, `c2` never; doubles are not singles | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T23` |
| `SB-CLY-021` | Degenerate crossplot geometry is refused and reported | `P0` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T24`, `SB-CLY-T32` |
| `SB-CLY-022` | Refuse the printed sonic-density denominator | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T25` |
| `SB-CLY-023` | Thorium and Potassium indicators | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T26` |
| `SB-CLY-024` | EM-propagation indicator, parameter named once | `P4` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T27` |
| `SB-CLY-025` | M–N crossplot Vsh is deliberately not implemented | `P4` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-026` | NMR clay volume is typed as a clay volume | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T28`, `SB-CLY-T43` |
| `SB-CLY-027` | Clip each indicator before combining, never after | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T29` |
| `SB-CLY-028` | Only bounded-safe combiners | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T30` |
| `SB-CLY-029` | A zero is a value, not an absence | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T31` |
| `SB-CLY-030` | Three distinct absences, distinguishable in the output | `P1` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T32` |
| `SB-CLY-031` | Every clay/shale volume carries a provenance curve | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33`, `SB-CLY-T34` |
| `SB-CLY-032` | One closed provenance vocabulary, substitution recorded separately | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T34` |
| `SB-CLY-033` | Per-flag override generality | `P2` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T36` |
| `SB-CLY-034` | No magic sentinel for a rejected sample | `P0` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T35`, `SB-CLY-T44` |
| `SB-CLY-035` | Discriminator tests are two-sided by default | `P2` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T36` |
| `SB-CLY-036` | Per-indicator coal branch with its own provenance token | `P2` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T37` |
| `SB-CLY-037` | A complete percentile endpoint pipeline | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T38` |
| `SB-CLY-038` | Two-way binding between percentile and value | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T39` |
| `SB-CLY-039` | The P3/P97 house preset is a cited, recorded preset | `P2` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T38` |
| `SB-CLY-040` | Warn where a percentile endpoint lands near a transform pole | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T10`, `SB-CLY-T40` |
| `SB-CLY-041` | Prefer the corrected input alias, uniformly | `P2` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T43` |
| `SB-CLY-042` | Picking conventions stated as help text, not encoded as defaults | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-043` | Shale volume and clay volume are distinct typed quantities | `P0` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T28`, `SB-CLY-T43` |
| `SB-CLY-044` | Both bridges named; no default ratio | `P1` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T41` |
| `SB-CLY-045` | Endpoint conversion identities are explicit and tested | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T42` |
| `SB-CLY-046` | Register the Vsh/Vcl curve families | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T43` |
| `SB-CLY-047` | Organic-shale pre-correction in renormalised form | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-048` | Guard the renormalisation denominator | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-049` | Do not iterate kerogen and heavy-mineral volumes inside the indicator | `P3` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-050` | Where the vendors disagree, ship no default and surface the conflict | `P0` | `PRESENT-DIVERGENT` | `10_clay-volume.md` | `SB-CLY-T18`, `SB-CLY-T19`, `SB-CLY-T20` |
| `SB-CLY-051` | The vendor artefact path is the primary source string | `P1` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-052` | Import by ordinal **and** semantic key | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T33` |
| `SB-CLY-053` | Matrix travel time is module-scoped and carries its artefact | `P2` | `ABSENT` | `10_clay-volume.md` | `SB-CLY-T25` |
| `SB-CLY-054` | Unit-typed quantities; no magic scale constants | `P0` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T21`, `SB-CLY-T42` |
| `SB-CLY-055` | LAS null discipline on every domain curve | `P1` | `PARTIAL` | `10_clay-volume.md` | `SB-CLY-T35`, `SB-CLY-T44` |
| `SB-CORE-001` | Depth unit is a first-class, carried property | `P0` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T01`, `SB-CORE-T01b`, `SB-CORE-T02` |
| `SB-CORE-002` | A degraded or failed result is never presented as a clean one | `P0` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T03`, `SB-CORE-T04`, `SB-CORE-T05`, `SB-CORE-T06`, `SB-CORE-T07`, `SB-CORE-T08`, `SB-CORE-T09` |
| `SB-CORE-003` | Validity conditions are enforced preconditions | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-004` | No parameter ships without a source | `P0` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T10`, `SB-CORE-T11` |
| `SB-CORE-005` | Vendor-derived defaults are re-sourced to primary literature | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-006` | One name, one equation | `P0` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T17`, `SB-CORE-T18` |
| `SB-CORE-007` | One definition for every constant and every transform | `P0` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T19`, `SB-CORE-T20`, `SB-CORE-T23` |
| `SB-CORE-010` | Every computed curve answers "how was I made?" | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T14`, `SB-CORE-T15` |
| `SB-CORE-011` | A project re-runs byte-identically | `P1` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T16` |
| `SB-CORE-012` | Named interpretation scenarios with A/B diff | `P2` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-013` | Vendor divergence is visible at the point of choice | `P2` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-014` | A learned model carries its training provenance | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T21`, `SB-CORE-T22` |
| `SB-CORE-015` | No artifact ships that SandiBumi's own reader rejects | `P0` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T14`, `SB-CORE-T15`, `SB-CORE-T16` |
| `SB-CORE-030` | Portfolio-scale target is declared and measured | `P1` | `UNMEASURED` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-031` | A benchmark harness exists and is part of the gate | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-032` | The compute path does not hold the global lock across long work | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-033` | Compute results are cached on content, not recomputed | `P2` | `ABSENT — designed, parked` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-034` | Interactive surfaces stay responsive at portfolio scale | `P2` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-035` | Well scoping is enforced in the backend | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-036` | Cancellation is honest | `P1` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-040` | Verification is indexed by capability | `P0` | `ABSENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T12` |
| `SB-CORE-041` | The tree builds and tests from a fresh clone | `P0` | `PRESENT-DIVERGENT` | `04_CORE_REQUIREMENTS.md` | `SB-CORE-T13` |
| `SB-CORE-042` | A green gate that a machine enforces | `P1` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-043` | Architecture and decisions are written down | `P1` | `ABSENT` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CORE-044` | Tier-C boundary is a shipped, auditable policy | `P1` | `PARTIAL` | `04_CORE_REQUIREMENTS.md` |  |
| `SB-CUT-001` | Make the depth discretisation model an explicit parameter | `P1` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T01`, `SB-CUT-T02`, `SB-CUT-T02b`, `SB-CUT-T03`, `SB-CUT-T03b`, `SB-CUT-T03c` |
| `SB-CUT-002` | Name the discretisation model on every thickness-bearing result | `P1` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T02b`, `SB-CUT-T31` |
| `SB-CUT-003` | Partition gross footage four ways | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T11`, `SB-CUT-T22` |
| `SB-CUT-004` | Report net-to-gross both with and without the unknown footage | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T11` |
| `SB-CUT-005` | Reconcile the footage partition with a named tolerance and a recorded residual | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T22` |
| `SB-CUT-006` | Implement averaging as a generalised power mean with an explicit exponent | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T05` |
| `SB-CUT-007` | Compute the geometric average in weight-normalised form, with a non-positive guard | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T04`, `SB-CUT-T09` |
| `SB-CUT-008` | Make the harmonic average skip non-positive samples rather than refuse the interval | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T09` |
| `SB-CUT-009` | Key porosity-weighting off an explicit per-curve flag, never off the mnemonic | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T06`, `SB-CUT-T07` |
| `SB-CUT-010` | Hold the volumetric identity between summed and reconstructed hydrocarbon pore volume | `P1` | `PRESENT-UNVERIFIED` | `14_cutoffs-summation-mc.md` | `SB-CUT-T07` |
| `SB-CUT-011` | Exclude samples outside every zone from cumulative results | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T10` |
| `SB-CUT-012` | Treat the reference frame as part of a result's identity | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T08` |
| `SB-CUT-013` | Model bed amalgamation with three independent thresholds | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T31` |
| `SB-CUT-014` | Emit bed statistics twice, pre- and post-amalgamation | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T31` |
| `SB-CUT-015` | State the reported bed-thickness convention explicitly | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T31` |
| `SB-CUT-016` | Ship no cut-off value | `P0` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-017` | Carry a source string on every default | `P0` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-018` | Resolve every cut-off entry point from one authority | `P0` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T35` |
| `SB-CUT-019` | Require a unit on cut-off entry | `P1` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T26` |
| `SB-CUT-020` | Express a cut-off as a two-sided range with an explicit bounds operator | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T24` |
| `SB-CUT-021` | Allow a cut-off to be supplied as a curve | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T24` |
| `SB-CUT-022` | Make cut-off activation an explicit flag, and share one value across the reservoir and pay tiers | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T24`, `SB-CUT-T36` |
| `SB-CUT-023` | Evaluate cut-off criteria as a boolean expression | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T32` |
| `SB-CUT-024` | Support arbitrary named flag tiers over arbitrary cut-off sets | `P2` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-025` | Treat lumps as a many-to-one reporting transform over flags | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T31` |
| `SB-CUT-026` | Leave saturation off at the reservoir tier by default | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-027` | Impose no cap on curves, cut-offs, report tiers or flags | `P2` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T36` |
| `SB-CUT-028` | Emit `SWE` and `SWT`, never a bare `SW` | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T06` |
| `SB-CUT-029` | Carry null markers as typed sibling fields | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T37` |
| `SB-CUT-030` | Separate the accumulate, flag-test and present stages of clamping | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T15`, `SB-CUT-T23`, `SB-CUT-T25` |
| `SB-CUT-031` | Make the shift-to-σ multiple explicit and mandatory, and set it to 2 for IP-sourced widths | `P0` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T13` |
| `SB-CUT-032` | Store the shift type with the width, and refuse to coerce a reciprocal shift | `P1` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T20`, `SB-CUT-T30` |
| `SB-CUT-033` | Import measurement priors, not only parameter priors | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T20` |
| `SB-CUT-034` | Make the seed mandatory and part of the result record | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T12` |
| `SB-CUT-035` | Provide log-domain distributions | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T14` |
| `SB-CUT-036` | Ship every prior as a (value, basis, sigma-multiple) triple with units | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T13`, `SB-CUT-T21` |
| `SB-CUT-037` | Store the centring rule per prior | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T28` |
| `SB-CUT-038` | Truncate Gaussian draws and report the resulting variance deficit | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T14` |
| `SB-CUT-039` | Set the iteration default from a cited source and auto-stop on the reported percentile | `P1` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T33` |
| `SB-CUT-040` | Draw one offset per section per iteration | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T27` |
| `SB-CUT-041` | Never clamp before accumulation | `P0` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T15`, `SB-CUT-T23`, `SB-CUT-T25` |
| `SB-CUT-042` | Perturb cut-offs under Monte Carlo, per zone | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T18` |
| `SB-CUT-043` | Compute derived ratios inside the iteration | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T19` |
| `SB-CUT-044` | Store per-iteration joint records and report iteration-consistent percentile cases | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T16` |
| `SB-CUT-045` | Withhold a statistic whose preconditions fail | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T29` |
| `SB-CUT-046` | Name the percentile interpolation method on the output record | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T16` |
| `SB-CUT-047` | Report percentile cases as reserves categories with their actual probabilities | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T16` |
| `SB-CUT-048` | Merge cases, not statistics, when rolling percentiles up across zones | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T16` |
| `SB-CUT-049` | Report the realised correlation alongside the requested one | `P1` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T18` |
| `SB-CUT-050` | Re-derive data-picked parameters each iteration rather than correlating them | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T18` |
| `SB-CUT-051` | Emit tornado bars in absolute output units | `P2` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T34` |
| `SB-CUT-052` | Ship perturbation off | `P1` | `PRESENT-DIVERGENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T15`, `SB-CUT-T23` |
| `SB-CUT-053` | Report physically impossible realizations, never exclude them | `P1` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T25` |
| `SB-CUT-054` | Fail a Monte Carlo study whose chain step failed on every realization | `P0` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T37c` |
| `SB-CUT-055` | Never report an uninterpreted well as a zero result | `P0` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T37` |
| `SB-CUT-056` | Never omit a report section because its computation failed | `P0` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T37b` |
| `SB-CUT-057` | Cross the IPC boundary in snake_case with unknown fields rejected | `P0` | `PRESENT-OK` | `14_cutoffs-summation-mc.md` | `SB-CUT-T38` |
| `SB-CUT-058` | Sweep more than one cut-off at a time | `P2` | `PARTIAL` | `14_cutoffs-summation-mc.md` | `SB-CUT-T32` |
| `SB-CUT-059` | Solve backwards from a target | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T32` |
| `SB-CUT-060` | Address imported parameters by block, ordinal and semantic key | `P2` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T17`, `SB-CUT-T21` |
| `SB-CUT-061` | Validate display precision against field width | `P3` | `ABSENT` | `14_cutoffs-summation-mc.md` | `SB-CUT-T39` |
| `SB-DBM-001` | One run record per computed curve, resolvable in one hop | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T03`, `SB-DBM-T10` |
| `SB-DBM-002` | The run record pins module identity by version, not by name | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T04`, `SB-DBM-T15` |
| `SB-DBM-003` | Every petrophysical parameter in a run record carries a source string | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T05`, `SB-DBM-T09`, `SB-DBM-T30` |
| `SB-DBM-004` | The run record stores the effective parameter set, not only the overrides | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T06`, `SB-DBM-T15` |
| `SB-DBM-005` | The run record carries a method-derivation citation, not only parameter values | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T07`, `SB-DBM-T10` |
| `SB-DBM-006` | Inputs are recorded as resolved identities, with the rule that chose them and the candidates it rejected | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T08` |
| `SB-DBM-007` | A missing provenance element is a named state, never an empty string | `P1` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T09` |
| `SB-DBM-008` | The run record names the operator and the zone set in force | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T11` |
| `SB-DBM-009` | Provenance timestamps are stored UTC and displayed local | `P2` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T11` |
| `SB-DBM-010` | Provenance travels into the deliverable | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T10` |
| `SB-DBM-011` | Structured audit entries, as name-value pairs with a controlled vocabulary | `P1` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T11`, `SB-DBM-T12` |
| `SB-DBM-012` | A parameter-state diff is a database join, not an external differ | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T12` |
| `SB-DBM-013` | No configuration, deployment mode or preference may disable the provenance record | `P1` | `PRESENT-OK` | `22_database-model.md` | `SB-DBM-T13` |
| `SB-DBM-014` | Every stochastic operation records its seed and its seeding rule | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T14`, `SB-DBM-T15` |
| `SB-DBM-015` | The re-run manifest is enumerated, stored, and checkable | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T15`, `SB-DBM-T16` |
| `SB-DBM-016` | Re-run output does not depend on iteration order | `P1` | `PRESENT-UNVERIFIED` | `22_database-model.md` | `SB-DBM-T16` |
| `SB-DBM-017` | A metadata attribute that drives physics is an input of the module that consumes it | `P1` | `ABSENT` | `22_database-model.md` | `SB-DBM-T17` |
| `SB-DBM-018` | Training-set identity is recorded as ids and intervals, not as names | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T18`, `SB-DBM-T20` |
| `SB-DBM-019` | A stored model carries its seed, its full library set and an artifact hash | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T19`, `SB-DBM-T21` |
| `SB-DBM-020` | Both apply paths stamp the model identity into the produced curve's provenance | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T20` |
| `SB-DBM-021` | Model artifacts are native-only; a foreign artifact is refused at the store boundary | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T21` |
| `SB-DBM-022` | The feature-vector order contract is verified at apply time, not assumed | `P1` | `PRESENT-UNVERIFIED` | `22_database-model.md` | `SB-DBM-T22` |
| `SB-DBM-023` | Schema-level vocabularies live in one registry, and every consumer resolves through it | `P1` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T23` |
| `SB-DBM-024` | Every capacity limit is unit-typed, carries a source string, and is the source of its own documentation | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T24` |
| `SB-DBM-025` | A constant that crosses a module boundary is registered with its source | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T23`, `SB-DBM-T24` |
| `SB-DBM-026` | Two samples may not share a depth in one curve, and the resolution is declared | `P1` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T25`, `SB-DBM-T26` |
| `SB-DBM-027` | A referential-integrity checker exists, reports every dangling class by name and count, and never reports "clean" without checking | `P1` | `ABSENT` | `22_database-model.md` | `SB-DBM-T26` |
| `SB-DBM-028` | A declared sampling style is verified against the reference column on ingest, and the verdict is stored | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T27` |
| `SB-DBM-029` | A module never writes to the reference column of a frame it reads | `P1` | `PRESENT-UNVERIFIED` | `22_database-model.md` | `SB-DBM-T28` |
| `SB-DBM-030` | Null discipline: a threshold, not an equality; and "no value" is not "no parameter" | `P0` | `ABSENT` | `22_database-model.md` | `SB-DBM-T29`, `SB-DBM-T30` |
| `SB-DBM-031` | Every depth quantity declares its datum, and cross-datum comparison is refused | `P1` | `ABSENT` | `22_database-model.md` | `SB-DBM-T31` |
| `SB-DBM-032` | A stored parameter carries a dual handle, and a disagreement is a load failure | `P1` | `ABSENT` | `22_database-model.md` | `SB-DBM-T32` |
| `SB-DBM-033` | A categorical curve is a distinct type and is never linearly interpolated | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T33` |
| `SB-DBM-034` | Every bulk operation returns `{matched, unmatched, ambiguous}` and drops nothing silently | `P1` | `ABSENT` | `22_database-model.md` | `SB-DBM-T34` |
| `SB-DBM-035` | The archive is append-only, and restoring a prior version is a first-class operation | `P1` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T35` |
| `SB-DBM-036` | No operation whose duration scales with well count holds the global lock | `P1` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T36`, `SB-DBM-T38` |
| `SB-DBM-037` | Well scoping is enforced in the backend, not in the client | `P1` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T37` |
| `SB-DBM-038` | The interactive set is the only thing materialised | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T38` |
| `SB-DBM-039` | A job result distinguishes clean, degraded and failed, and the store records which | `P0` | `PARTIAL` | `22_database-model.md` | `SB-DBM-T39`, `SB-DBM-T41` |
| `SB-DBM-040` | Cancellation honesty is regression-locked | `P1` | `PRESENT-OK` | `22_database-model.md` | `SB-DBM-T40` |
| `SB-DBM-041` | A count presented as a total is a total; the inspector exposes the provenance tables | `P1` | `PRESENT-DIVERGENT` | `22_database-model.md` | `SB-DBM-T41`, `SB-DBM-T42` |
| `SB-DBM-042` | The format-version gate and the pre-migration backup are contractual, and the backup names the format it can restore | `P0` | `PRESENT-OK` | `22_database-model.md` | `SB-DBM-T01`, `SB-DBM-T02`, `SB-DBM-T43` |
| `SB-DBM-043` | A deterministic parameter sweep records every trial, uncapped and ordered | `P2` | `ABSENT` | `22_database-model.md` | `SB-DBM-T44` |
| `SB-DIO-001` | A single declared sentinel MUST reach every writer. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T01`, `SB-DIO-T02` |
| `SB-DIO-002` | The default export path MUST NOT be the one that bypasses the sentinel. | `P1` | `PRESENT-UNVERIFIED` | `21_data-io.md` | `SB-DIO-T03` |
| `SB-DIO-003` | "This channel has no null" MUST be a first-class state. | `P2` | `ABSENT` | `21_data-io.md` | `SB-DIO-T04`, `SB-DIO-T05` |
| `SB-DIO-004` | Null recognition MUST be one relative-tolerance transform, and recognition MUST NOT rewrite. | `P0` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T06`, `SB-DIO-T07`, `SB-DIO-T08` |
| `SB-DIO-005` | Null values MUST be per-channel and plural. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T09` |
| `SB-DIO-006` | The null-exception rule shape MUST be many-to-many. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T10` |
| `SB-DIO-007` | Absent MUST be distinguishable from nulled. | `P2` | `ABSENT` | `21_data-io.md` | `SB-DIO-T11` |
| `SB-DIO-008` | Coverage-aware alias resolution MUST be preserved. | `P1` | `PRESENT-OK` | `21_data-io.md` | `SB-DIO-T12`, `SB-DIO-T13` |
| `SB-DIO-009` | The alias choice MUST be reported. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T14` |
| `SB-DIO-010` | Prefer a structural index declaration; fall back to names; record which mechanism fired. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T15`, `SB-DIO-T16` |
| `SB-DIO-011` | Index aliases MUST be namespace-aware and MUST have one definition per path. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T17` |
| `SB-DIO-012` | A non-monotonic index MUST be detected and reported, never silently accepted. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T18` |
| `SB-DIO-013` | When neither structure nor name resolves an index, the user MUST designate it. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T19` |
| `SB-DIO-014` | TVD MUST NOT be read as an MD index. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T20`, `SB-DIO-T21` |
| `SB-DIO-015` | An index with no declared unit anywhere MUST refuse. | `P0` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T22`, `SB-DIO-T23`, `SB-DIO-T24` |
| `SB-DIO-016` | The DLIS index unit MUST be read and reconciled. | `P0` | `ABSENT` | `21_data-io.md` | `SB-DIO-T25`, `SB-DIO-T26` |
| `SB-DIO-017` | The LAS writer MUST write the depth unit it actually used. | `P0` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T27`, `SB-DIO-T28` |
| `SB-DIO-018` | Canonical units MUST have exactly one definition. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T29`, `SB-DIO-T30` |
| `SB-DIO-019` | Changing the project depth unit MUST NOT silently rescale stored data. | `P1` | `PRESENT-UNVERIFIED` | `21_data-io.md` | `SB-DIO-T31` |
| `SB-DIO-020` | Duplicate depths MUST be resolved by a declared policy, and the count reported. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T32`, `SB-DIO-T33` |
| `SB-DIO-021` | Resampling on read MUST be explicit, named, and off by default. | `P2` | `PRESENT-OK` | `21_data-io.md` | `SB-DIO-T34` |
| `SB-DIO-022` | Re-grid on write MUST be named correctly and default OFF. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T35` |
| `SB-DIO-023` | Numeric columns MUST be validated against physical bounds, not against their labels. | `P0` | `ABSENT` | `21_data-io.md` | `SB-DIO-T36`, `SB-DIO-T37`, `SB-DIO-T38` |
| `SB-DIO-024` | Unit conversion MUST NOT be applied silently by default. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T39` |
| `SB-DIO-025` | Conversion coverage MUST be declared, and an unconvertible unit MUST be reported rather than passed through. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T40`, `SB-DIO-T41` |
| `SB-DIO-026` | Unit conversion MUST support affine transforms. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T42` |
| `SB-DIO-027` | A vendor alias that is wrong or ambiguous MUST NOT be inherited. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T43` |
| `SB-DIO-028` | A conversion factor MUST be correct and MUST show its derivation. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T44` |
| `SB-DIO-029` | An unadjudicable unit ambiguity MUST ship with no default. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T45` |
| `SB-DIO-030` | An alias rename MUST be reported. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T46` |
| `SB-DIO-031` | A different curve's data MUST NOT be supplied under a requested name. | `P0` | `ABSENT` | `21_data-io.md` | `SB-DIO-T47` |
| `SB-DIO-032` | A substitution offered to the user MUST be explicit and recorded. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T48` |
| `SB-DIO-033` | Curve-selection state MUST be explicit and inspectable. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T49` |
| `SB-DIO-034` | Curves MUST NOT be auto-selected by curve type on read. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T50` |
| `SB-DIO-035` | An import MUST NOT extend an existing object's declared interval. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T51` |
| `SB-DIO-036` | The duplicate-name policy MUST NOT default to merge. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T52` |
| `SB-DIO-037` | Channels that could not be loaded MUST be named, and a partial load MUST NOT be reported as success. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T53` |
| `SB-DIO-038` | Multi-dimensional channels MUST be imported through the published RP66 container. | `P2` | `ABSENT` | `21_data-io.md` | `SB-DIO-T54`, `SB-DIO-T55` |
| `SB-DIO-039` | The DLIS sentinel screen MUST be per-channel overridable and MUST count what it deleted. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T56`, `SB-DIO-T57` |
| `SB-DIO-040` | Wrapped LAS MUST be read; the writer MUST emit unwrapped. | `P2` | `PRESENT-OK` | `21_data-io.md` | `SB-DIO-T58` |
| `SB-DIO-041` | A LAS 3.0 file MUST be recognised, and what is not read MUST be named. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T59` |
| `SB-DIO-042` | LAS 3.0 associated sections MUST be read. | `P3` | `ABSENT` | `21_data-io.md` | `SB-DIO-T60` |
| `SB-DIO-043` | LAS 1.2 MUST be readable and MUST NOT be writable. | `P2` | `ABSENT` | `21_data-io.md` | `SB-DIO-T61` |
| `SB-DIO-044` | Section-parse strictness MUST be declared and consistent. | `P2` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T62` |
| `SB-DIO-045` | A multi-well container MUST produce multiple wells, never one merged well. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T63`, `SB-DIO-T64` |
| `SB-DIO-046` | A missing interpreter or library MUST produce a named, actionable, per-format refusal. | `P1` | `PRESENT-OK` | `21_data-io.md` | `SB-DIO-T65` |
| `SB-DIO-047` | Storage precision MUST be declared and MUST NOT silently truncate. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T66` |
| `SB-DIO-048` | Well identity in a container MUST come from the container, never from the filename. | `P2` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T67` |
| `SB-DIO-049` | Writing a file our own reader would reject MUST be an error, not a warning. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T68`, `SB-DIO-T69` |
| `SB-DIO-050` | A re-gridded input MUST be detectable at import. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T70` |
| `SB-DIO-051` | Provenance MUST be carried into the deliverable. | `P0` | `ABSENT` | `21_data-io.md` | `SB-DIO-T71`, `SB-DIO-T72`, `SB-DIO-T73` |
| `SB-DIO-052` | Final and working curves MUST be distinguishable in an export. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T74` |
| `SB-DIO-053` | Well-header fields MUST be mapped explicitly and identity MUST NOT be invented. | `P2` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T75`, `SB-DIO-T76` |
| `SB-DIO-054` | Every skipped frame, channel, curve and row MUST be counted and named. | `P0` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T77`, `SB-DIO-T78`, `SB-DIO-T79` |
| `SB-DIO-055` | An export that omits data MUST say what it omitted. | `P0` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T80`, `SB-DIO-T81` |
| `SB-DIO-056` | A declared `STEP` MUST be verified across the whole index. | `P1` | `PRESENT-DIVERGENT` | `21_data-io.md` | `SB-DIO-T82`, `SB-DIO-T83` |
| `SB-DIO-057` | A zero on a log-scale curve MUST NOT be committed as a reading. | `P1` | `ABSENT` | `21_data-io.md` | `SB-DIO-T84`, `SB-DIO-T85` |
| `SB-DIO-058` | Old `.xls` plate workbooks MUST be read from the published specification. | `P2` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T86`, `SB-DIO-T87` |
| `SB-DIO-059` | Tabular `.xls` MUST be readable without the drawing layer. | `P2` | `ABSENT` | `21_data-io.md` | `SB-DIO-T88` |
| `SB-DIO-060` | Format MUST be recognised by signature, and signature collisions MUST be handled. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T89`, `SB-DIO-T90` |
| `SB-DIO-061` | Malformed input MUST be located, counted, named, and regression-tested against a corpus. | `P0` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T91`, `SB-DIO-T92`, `SB-DIO-T93`, `SB-DIO-T94` |
| `SB-DIO-062` | Text encoding MUST be detected, not assumed. | `P1` | `PARTIAL` | `21_data-io.md` | `SB-DIO-T95` |
| `SB-DIO-063` | Non-ASCII paths and payloads MUST survive every sidecar boundary. | `P1` | `PRESENT-OK` | `21_data-io.md` | `SB-DIO-T96` |
| `SB-ENV-001` | Declare validity conditions as data on the module spec | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T01`, `SB-ENV-T02`, `SB-ENV-T03`, `SB-ENV-T38` |
| `SB-ENV-002` | Evaluate preconditions in the runner, before the module body | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T02`, `SB-ENV-T04`, `SB-ENV-T38` |
| `SB-ENV-003` | A violated precondition produces a refusal or a flagged result, never an unmarked number | `P0` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T02`, `SB-ENV-T03`, `SB-ENV-T04`, `SB-ENV-T05` |
| `SB-ENV-004` | Every parameter carries a source string, built as one change with the validity field | `P0` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T06`, `SB-ENV-T07` |
| `SB-ENV-005` | A corrected curve carries the list of steps actually applied | `P0` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T08`, `SB-ENV-T09`, `SB-ENV-T10` |
| `SB-ENV-006` | A curve named "corrected" MUST have been corrected | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T11`, `SB-ENV-T12` |
| `SB-ENV-007` | Per-sample correction flag channel | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T11`, `SB-ENV-T13` |
| `SB-ENV-008` | Validity conditions are visible before the run, not only after it | `P2` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T14` |
| `SB-ENV-009` | A method-selection string that matches no known method is an error | `P0` | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T03`, `SB-ENV-T15` |
| `SB-ENV-010` | The GR borehole correction models hole size, mud weight, tool position and mud type | `P2` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T08`, `SB-ENV-T16` |
| `SB-ENV-011` | The neutron correction chain exposes all ten steps, and an unavailable step is reported | `P2` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T08`, `SB-ENV-T09`, `SB-ENV-T17` |
| `SB-ENV-012` | Neutron matrix scale is a declared property of the curve and is validated at every consumer | `P0` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T18`, `SB-ENV-T19` |
| `SB-ENV-013` | The density borehole correction models mudcake as well as hole size | `P2` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T20` |
| `SB-ENV-014` | Correction coefficients ship with a source or ship ABSENT | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T06`, `SB-ENV-T07`, `SB-ENV-T21` |
| `SB-ENV-015` | The correction-chart lookup interface is specified independently of any chart data | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T22`, `SB-ENV-T23`, `SB-ENV-T24` |
| `SB-ENV-016` | A measured property of the formation or the borehole ships no default | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T07`, `SB-ENV-T25` |
| `SB-ENV-017` | Chart baselines and intermediates are named, single-assignment quantities | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T26` |
| `SB-ENV-018` | Conditioning and correction order is a declared, checkable contract | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T27`, `SB-ENV-T28` |
| `SB-ENV-019` | Per-tool uncertainty is computed over the steps actually applied, and says which | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T09`, `SB-ENV-T29` |
| `SB-ENV-020` | Correction-chain QC: what did the corrections actually do? | `P2` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T30` |
| `SB-ENV-021` | Bad-hole detection degrades to the inputs that exist, and says which it used | `P1` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T31`, `SB-ENV-T32` |
| `SB-ENV-022` | Bad-hole flag carries a reason channel | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T31` |
| `SB-ENV-023` | The density correction's sign is preserved and reported | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T31` |
| `SB-ENV-024` | Bad-hole thresholds ship ABSENT with cited presets | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T07`, `SB-ENV-T33` |
| `SB-ENV-025` | Bit size is an input, never a default | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T33`, `SB-ENV-T34` |
| `SB-ENV-026` | DRHO's unit is declared on the curve and validated at the threshold | `P0` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T35` |
| `SB-ENV-027` | A module whose purpose is to produce a value where the mask says there is none MUST be exempt from the mask | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T36`, `SB-ENV-T37` |
| `SB-ENV-028` | The mask is recorded in the run's provenance | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T27` |
| `SB-ENV-029` | Conditioning flags validate their own stated preconditions | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T18`, `SB-ENV-T19` |
| `SB-ENV-030` | One flag polarity, defined once, as a type | `P0` | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T38`, `SB-ENV-T39` |
| `SB-ENV-031` | The despike cutoff shows its contamination ceiling, live | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T40`, `SB-ENV-T69`, `SB-ENV-T70` |
| `SB-ENV-032` | The MAD consistency constant is defined once, named, and cited | `P2` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T41` |
| `SB-ENV-033` | A degenerate window is declared, not silently substituted | `P2` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T42` |
| `SB-ENV-034` | Every window, gap and thickness parameter is a thickness in the project's depth unit | `P0` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T43` |
| `SB-ENV-035` | Smoothing never bridges a gap, and never invents a sample | `P0` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T44` |
| `SB-ENV-036` | Outlier and spurious-population culling exists as a distinct operation | `P2` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T27` |
| `SB-ENV-037` | Every removed or replaced sample is recoverable | `P1` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T45` |
| `SB-ENV-038` | Gap filling states its boundary comparison and refuses an open-ended gap | `P1` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T46` |
| `SB-ENV-039` | Clip refuses rather than repairs | `P2` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T47` |
| `SB-ENV-040` | A conditioning output is never the input's own mnemonic | `P0` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T48` |
| `SB-ENV-041` | The filter kernel and its normalisation are declared in the output | `P2` | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T49` |
| `SB-ENV-042` | Interactive edits carry provenance, not only undo | `P1` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T45` |
| `SB-ENV-043` | One formation-temperature definition, one mnemonic | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-CORE-T23`, `SB-ENV-T50`, `SB-ENV-T51` |
| `SB-ENV-044` | Formation temperature is a function of true vertical depth | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T51`, `SB-ENV-T52` |
| `SB-ENV-045` | The geothermal gradient carries a declared, validated compound unit | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T52`, `SB-ENV-T53` |
| `SB-ENV-046` | A mudline / water-bottom branch exists for offshore wells | `P2` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T54` |
| `SB-ENV-047` | A declared parameter that does not enter the answer is removed or used | `P1` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T55` |
| `SB-ENV-048` | The resistivity temperature constant is defined once, cited, and surfaced | `P0` | `PRESENT-UNVERIFIED` | `20_envcorr-qc.md` | `SB-ENV-T56`, `SB-ENV-T57` |
| `SB-ENV-049` | A superseded module delegates to the survivor and says so | `P1` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T58` |
| `SB-ENV-050` | A depth-trend parameter is well-scoped, and a compartment parameter is not | `P1` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T59` |
| `SB-ENV-051` | Percentiles are exact order statistics, never histogram bin means | `P0` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T60` |
| `SB-ENV-052` | The normalisation reference pair ships ABSENT | `P0` | `PRESENT-OK` | `20_envcorr-qc.md` | `SB-ENV-T61` |
| `SB-ENV-053` | Normalisation is recorded, reviewable and overridable per well | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T62` |
| `SB-ENV-054` | Normalisation percentiles are computed over a declared common interval | `P1` | `PARTIAL` | `20_envcorr-qc.md` | `SB-ENV-T62`, `SB-ENV-T63` |
| `SB-ENV-055` | A normalisation reference pair is named and sourced separately from a `Vsh` endpoint pair | `P1` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T64` |
| `SB-ENV-056` | Log-QC limits ship ABSENT, and band precedence is specified once | `P1` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T65`, `SB-ENV-T66` |
| `SB-ENV-057` | One token for "a length in the project's depth unit", validated once | `P0` | `PRESENT-DIVERGENT` | `20_envcorr-qc.md` | `SB-ENV-T43`, `SB-ENV-T67` |
| `SB-ENV-058` | Borehole-image speed correction, derived independently | `P3` | `ABSENT` | `20_envcorr-qc.md` | `SB-ENV-T68` |
| `SB-GEO-001` | Gate six independently versioned domain units | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T01`, `SB-GEO-T02` |
| `SB-GEO-002` | Type every depth and reference datum | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T03`, `SB-GEO-T04` |
| `SB-GEO-003` | Integrate vertical stress from the physical anchor | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T05`, `SB-GEO-T06`, `SB-GEO-T07` |
| `SB-GEO-004` | Preserve measured and synthetic density provenance | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T08`, `SB-GEO-T09` |
| `SB-GEO-005` | Select or fit synthetic density explicitly | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T10`, `SB-GEO-T11` |
| `SB-GEO-006` | Enforce every correlation's applicability contract | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T12`, `SB-GEO-T13` |
| `SB-GEO-007` | Never make a vendor table implementation truth | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T14` |
| `SB-GEO-008` | Resolve one shared water density | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T15`, `SB-GEO-T16` |
| `SB-GEO-009` | Anchor normal pressure at the water/formation boundary | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T17`, `SB-GEO-T18` |
| `SB-GEO-010` | Apply Terzaghi effective stress with explicit Biot alpha | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T19` |
| `SB-GEO-011` | Implement four distinct Eaton forms | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T20`, `SB-GEO-T21` |
| `SB-GEO-012` | Require readable trend inputs and output them | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T22` |
| `SB-GEO-013` | Calibrate Bowers before emission | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T23`, `SB-GEO-T24` |
| `SB-GEO-014` | Make Bowers unloading algebraically consistent | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T25`, `SB-GEO-T26` |
| `SB-GEO-015` | Block methods whose primary equation is missing | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T27` |
| `SB-GEO-016` | Apply one uniform pressure-limit policy | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T28`, `SB-GEO-T29` |
| `SB-GEO-017` | Emit pressure and gradient as separate typed curves | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T30` |
| `SB-GEO-018` | Implement the alpha-aware generalized fracture equation | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T31`, `SB-GEO-T32` |
| `SB-GEO-019` | Keep K relationships explicit | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T33` |
| `SB-GEO-020` | Source Matthews–Kelly coefficients from the paper | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T14`, `SB-GEO-T34` |
| `SB-GEO-021` | Enforce the Matthews–Kelly overburden premise | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T35` |
| `SB-GEO-022` | Enforce declared source geography | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T12`, `SB-GEO-T36` |
| `SB-GEO-023` | Expose every Poisson-polynomial breakpoint | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T37`, `SB-GEO-T38` |
| `SB-GEO-024` | Rebuild the Daines table from primary literature | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T14`, `SB-GEO-T39` |
| `SB-GEO-025` | Plot published fracture-pressure bounds as an envelope | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T40` |
| `SB-GEO-026` | Limit fracture pressure only by explicit policy | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T41` |
| `SB-GEO-027` | Compute minimum and maximum horizontal stress explicitly | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T42`, `SB-GEO-T43` |
| `SB-GEO-028` | Name strains by stress direction | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T44` |
| `SB-GEO-029` | Reuse dynamic properties without broadening their meaning | `P1` | `PARTIAL` | `18_geomech-ppfg.md` | `SB-GEO-T45`, `SB-GEO-T46` |
| `SB-GEO-030` | Require sourced dynamic-to-static transforms | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T47` |
| `SB-GEO-031` | Make modulus conversions dimensional | `P0` | `PARTIAL` | `18_geomech-ppfg.md` | `SB-GEO-T48`, `SB-GEO-T49`, `SB-GEO-T50` |
| `SB-GEO-032` | Keep stress and stress gradient distinct | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T51` |
| `SB-GEO-033` | Transform inclined stresses in a declared frame | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T52`, `SB-GEO-T53` |
| `SB-GEO-034` | Assert total versus effective input state | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T54` |
| `SB-GEO-035` | Preserve omitted physical terms as explicit inputs | `P2` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T55` |
| `SB-GEO-036` | Implement failure criteria from public equations | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T56`, `SB-GEO-T57` |
| `SB-GEO-037` | Solve Drucker–Prager numerically from invariants | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T58` |
| `SB-GEO-038` | Classify shear-failure modes separately from failure | `P2` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T59` |
| `SB-GEO-039` | Validate every stability input before solve | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T60` |
| `SB-GEO-040` | Bind strength correlations to native units | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T49`, `SB-GEO-T50`, `SB-GEO-T61` |
| `SB-GEO-041` | Use atan2 for every angle back-transform | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T62` |
| `SB-GEO-042` | Make unset sourced parameters block execution | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T63` |
| `SB-GEO-043` | Address parameter files semantically and ordinally | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T64` |
| `SB-GEO-044` | Version local calibration without promoting it to default | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T65` |
| `SB-GEO-045` | Refuse extrapolation outside a declared range | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T66` |
| `SB-GEO-046` | Prohibit raster- and binary-only implementation truth | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T27`, `SB-GEO-T67` |
| `SB-GEO-047` | Separate imported, computed and interpreted identities | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T68` |
| `SB-GEO-048` | Export a complete geomechanics run record | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T69` |
| `SB-GEO-049` | Keep shared parameters single-valued within a run | `P0` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T15`, `SB-GEO-T70` |
| `SB-GEO-050` | Execute every worked example | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T71` |
| `SB-GEO-051` | Keep post-processing visible | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T72` |
| `SB-GEO-052` | Gate acquisition-dependent methods individually | `P1` | `ABSENT` | `18_geomech-ppfg.md` | `SB-GEO-T02`, `SB-GEO-T73` |
| `SB-INS-001` | Ship a qualified native Windows installer | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-002` | Keep native core launch independent of Python | `P0` | `PRESENT-OK` | `27_ip-install-blockers.md` |  |
| `SB-INS-003` | Publish truthful capability-level prerequisites | `P0` | `PRESENT-DIVERGENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-004` | Maintain one dependency/capability manifest | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-005` | Resolve one interpreter with explainable precedence | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-006` | Probe packages and versions before work begins |  | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-007` | Give interpreter-specific remediation |  | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-008` | Support offline and managed deployment | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-009` | Pin and attest optional runtime contents |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-010` | Separate immutable templates from user configuration | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-011` | Migrate configuration explicitly and reversibly |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-012` | Provide a corporate policy layer |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-013` | Make precedence visible and deterministic |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-014` | Key parameters by semantic identifier and ordinal | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-015` | Refuse registry mismatch and ambiguity | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-016` | Use a canonical typed unit registry | `P0` | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-017` | Preserve observed unit and encoding tokens |  | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-018` | Reject missing and empty unit mappings |  | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-019` | Generate aliases, families and units from one registry |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-020` | Version and attest configuration packs |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-021` | Produce a reproducible support report |  | `PARTIAL` | `27_ip-install-blockers.md` |  |
| `SB-INS-022` | Preserve user data through upgrade and uninstall |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-023` | Gate releases on clean-machine scenarios | `P0` | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-024` | Generate and review third-party obligations |  | `PRESENT-OK` | `27_ip-install-blockers.md` |  |
| `SB-INS-025` | Enforce the evidence-acquisition firewall |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-INS-026` | Keep release claims derived from executable evidence |  | `ABSENT` | `27_ip-install-blockers.md` |  |
| `SB-MIN-001` | Solve as a bounded, non-negative least-squares problem | `P1` | `PRESENT-OK` | `13_mineral-solver.md` | `SB-MIN-T01` |
| `SB-MIN-002` | Disclose the solver-class divergence from IP in the run record | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T02` |
| `SB-MIN-003` | Impose unity as a hard equality over the non-X components | `P0` | `PRESENT-OK` | `13_mineral-solver.md` | `SB-MIN-T03` |
| `SB-MIN-004` | Report the unity convention alongside any misfit statistic | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T14` |
| `SB-MIN-005` | Enforce the fluid volume ceiling structurally, not by name lookup | `P2` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T04` |
| `SB-MIN-006` | Compute bound water from CEC with the salinity expansion term | `P1` | `PRESENT-OK` | `13_mineral-solver.md` | `SB-MIN-T05`, `SB-MIN-T06` |
| `SB-MIN-007` | Refuse a clay whose bound-water parameter is absent; never treat it as zero | `P0` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T07` |
| `SB-MIN-008` | Ship `CEC` and `WCLP` only as a matched pair from one library | `P0` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T08` |
| `SB-MIN-009` | Carry provenance on every endpoint value, not every endpoint column | `P0` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T09` |
| `SB-MIN-010` | Declare the wet/dry clay convention on every clay row and every clay curve | `P0` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T10`, `SB-MIN-T24` |
| `SB-MIN-011` | Declare the CEC unit and refuse implausible magnitudes | `P0` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T11` |
| `SB-MIN-012` | Mix photoelectric response volumetrically on U, never on Pe | `P0` | `PRESENT-OK` | `13_mineral-solver.md` | `SB-MIN-T12` |
| `SB-MIN-013` | Define `RECON` against a stated equation and pin it with a test | `P1` | `PRESENT-UNVERIFIED` | `13_mineral-solver.md` | `SB-MIN-T13` |
| `SB-MIN-014` | Emit IP-comparable and Geolog-comparable misfit statistics beside `RECON` | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T14` |
| `SB-MIN-015` | Report a conditioning number and refuse to present an unstable solve as trusted | `P1` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T15` |
| `SB-MIN-016` | Report degrees of freedom and refuse to let a zero-DOF fit read as validation | `P0` | `PRESENT-OK` | `13_mineral-solver.md` | `SB-MIN-T16` |
| `SB-MIN-017` | Separate "tool off" from "tool weighted to zero" | `P1` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T17` |
| `SB-MIN-018` | Provide a per-tool weight multiplier separate from the tool uncertainty | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T18` |
| `SB-MIN-019` | Store a tool's MIN, MAX and printed default uncertainty as three independent fields | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T19` |
| `SB-MIN-020` | Ship a default tool-uncertainty library with per-value sources | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T19`, `SB-MIN-T20` |
| `SB-MIN-021` | Make the conductivity root exponent an explicit, recorded model input | `P1` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T21` |
| `SB-MIN-022` | Ship no default for the Shell porosity-dependent cementation constant | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T22` |
| `SB-MIN-023` | Implement variable `m*` with the corroborated coefficient set | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T23` |
| `SB-MIN-024` | Convert wet↔dry clay with an explicit bound-water density, not a hard-coded 1.0 | `P2` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T24` |
| `SB-MIN-025` | Support a per-equation invasion factor | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T25` |
| `SB-MIN-026` | Make the neutron response set a named, recorded model input | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T26` |
| `SB-MIN-027` | Store WCLP in v/v and refuse a p.u. value instead of switching route | `P0` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T27` |
| `SB-MIN-028` | Offer a named endpoint library and surface the inter-library disagreement | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T28` |
| `SB-MIN-029` | Declare the fluid sonic endpoint source at the point of use | `P2` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T28`, `SB-MIN-T29` |
| `SB-MIN-030` | Treat silt as a first-class term and never merge two different Simandoux equations under one label | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T30` |
| `SB-MIN-031` | Allow a per-clay shale resistivity where the saturation model uses one | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T31` |
| `SB-MIN-032` | Persist the fully resolved parameter set with every run | `P1` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T32` |
| `SB-MIN-033` | Name the conflicting rows when a constraint set is infeasible | `P1` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T33` |
| `SB-MIN-034` | Impose the water-mud constraint as an inequality, iterated to feasibility | `P1` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T34` |
| `SB-MIN-035` | Impose `Tool` constraints as hard equality plus pseudo-measurement, and emit the tie residual | `P1` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T35` |
| `SB-MIN-036` | Complete the output nomenclature and declare each curve's convention | `P2` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T37` |
| `SB-MIN-037` | Propagate endpoint uncertainty by Monte Carlo with a recorded seed | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T38` |
| `SB-MIN-038` | Report a predicted uncertainty per solved volume | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T39` |
| `SB-MIN-039` | Offer balanced pre-solve tool uncertainties | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T20` |
| `SB-MIN-040` | Make the two bound-water routes mutually exclusive and record the choice | `P1` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T08`, `SB-MIN-T27` |
| `SB-MIN-041` | Keep retired modules resolvable and refuse to run them, carrying no orphan defaults | `P0` | `PRESENT-DIVERGENT` | `13_mineral-solver.md` | `SB-MIN-T40`, `SB-MIN-T41` |
| `SB-MIN-042` | Implement the oil-based-mud constraint pair | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T36` |
| `SB-MIN-043` | Offer the opt-in physical ceiling constraints | `P3` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T36` |
| `SB-MIN-044` | Canonicalise units at the boundary and prove invariance | `P2` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T42` |
| `SB-MIN-045` | Bound the formation temperature and record any fallback | `P2` | `PARTIAL` | `13_mineral-solver.md` | `SB-MIN-T43` |
| `SB-MIN-046` | Gate the clay density triple for self-consistency | `P2` | `ABSENT` | `13_mineral-solver.md` | `SB-MIN-T44` |
| `SB-MLA-001` | Record the effective parameter set, not the supplied one | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T01`, `SB-MLA-T08` |
| `SB-MLA-002` | A saved model records the input log set it was trained from | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T02` |
| `SB-MLA-003` | A saved model identifies the exact training rows | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T03` |
| `SB-MLA-004` | A saved model records the exclusion mask and its effect | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T04` |
| `SB-MLA-005` | A saved model records the runtime that produced it | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T05`, `SB-MLA-T12` |
| `SB-MLA-006` | A curve produced by a fitted model names that model | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T06` |
| `SB-MLA-007` | A model cited by a stored curve cannot be deleted silently | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T07` |
| `SB-MLA-008` | A recorded ML run re-runs to byte-identical curves | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T01`, `SB-MLA-T08` |
| `SB-MLA-009` | Blind-well performance travels with the curve | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T09` |
| `SB-MLA-010` | The deliverable carries the ML provenance block | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T10` |
| `SB-MLA-011` | Training and apply membership are recorded per well | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T11` |
| `SB-MLA-012` | Artifact version skew fails loudly, and a substituted algorithm is never silent | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T12`, `SB-MLA-T27` |
| `SB-MLA-013` | An unclusterable well fails; it never emits a clean empty curve | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T13`, `SB-MLA-T23` |
| `SB-MLA-014` | A reduced cluster count is reported, never substituted silently | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T14` |
| `SB-MLA-015` | A floored mixture component is reported | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T15` |
| `SB-MLA-016` | Convergence and iteration exhaustion are distinguished | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T16` |
| `SB-MLA-017` | A cancelled run leaves no partially populated log set | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T17` |
| `SB-MLA-018` | The non-interruptible phase is declared, not hidden | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T18` |
| `SB-MLA-019` | A cross-validation protocol that degraded MUST NOT report a score as if it had not | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T19` |
| `SB-MLA-020` | A metric computed on a subsample says so | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T20` |
| `SB-MLA-021` | Density-based noise is a reported class, not a missing value | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T21` |
| `SB-MLA-022` | The ordered-feature refusal is verified on the default test gate | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T22` |
| `SB-MLA-023` | One k-means, one definition | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T23` |
| `SB-MLA-024` | One seed concept, one default | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T24` |
| `SB-MLA-025` | One within-cluster-sum-of-squares partition, three declared applications | `P1` | `PRESENT-DIVERGENT` | `24_ml-advanced.md` | `SB-MLA-T25` |
| `SB-MLA-026` | The leaderboard evaluates the model the run will fit | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T27` |
| `SB-MLA-027` | Every reported score names its protocol | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T28` |
| `SB-MLA-028` | Every fitted transform is fitted inside the fold | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T29` |
| `SB-MLA-029` | A facies mnemonic names the engine that produced it | `P1` | `PARTIAL` | `24_ml-advanced.md` | `SB-MLA-T30` |
| `SB-MLA-030` | Probability outputs are typed | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T31` |
| `SB-MLA-031` | Shipped vendor defaults are surfaced at the point of choice | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T32` |
| `SB-MLA-032` | The normalisation basis is a recorded choice, not an implicit one | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T33` |
| `SB-MLA-033` | A fixed normalisation basis is available, so adding a well does not move existing boundaries | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T34` |
| `SB-MLA-034` | Every automatic pre-transform is announced | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T35` |
| `SB-MLA-035` | A transformed quantity is a distinct quantity with its own name and unit | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T36` |
| `SB-MLA-036` | Enumerated methods are addressed by id, never by display string | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T37` |
| `SB-MLA-037` | Fuzzy combination across curves is the reciprocal sum | `P1` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T38` |
| `SB-MLA-038` | Equal-population binning reports its actual populations | `P2` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T39` |
| `SB-MLA-039` | The fuzzy uncertainty band has a defined edge behaviour | `P2` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T40` |
| `SB-MLA-040` | The bin-count weighting is explicit, with no hidden default | `P2` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T38` |
| `SB-MLA-041` | SOM decay is parameterised by total iterations, and the degenerate form is refused | `P3` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T45` |
| `SB-MLA-042` | Map quality is reported by a defined distortion measure | `P3` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T46` |
| `SB-MLA-043` | The cluster randomness index ships | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T41`, `SB-MLA-T42` |
| `SB-MLA-044` | The native clustering path reports cluster quality | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T43` |
| `SB-MLA-045` | Restart spread is reported as a convergence diagnostic | `P2` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T44` |
| `SB-MLA-046` | Hierarchical linkage is a named enumeration with a sourced default | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T37` |
| `SB-MLA-047` | PCA reports loadings and correlation-circle coordinates | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T26` |
| `SB-MLA-048` | Component sign is fixed by a stated convention | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T08` |
| `SB-MLA-049` | Nearest-neighbour prediction is a normalised weighted average, and its weight function is SandiBumi's | `P2` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T47`, `SB-MLA-T48` |
| `SB-MLA-050` | Feature scoring by leave-one-out excludes the held-out frame | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T49` |
| `SB-MLA-051` | A contingency table carries both normalisations, each labelled with its axis | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T50` |
| `SB-MLA-052` | The tie-in acceptance threshold ships absent and visible | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T51` |
| `SB-MLA-053` | A tolerance expressed in standard deviations is named for its unit | `P3` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T52` |
| `SB-MLA-054` | The depth-resampling decision is logged for every ML input | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T54` |
| `SB-MLA-055` | A class label is never interpolated | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T53` |
| `SB-MLA-056` | Null discipline holds through the ML path with no opt-out | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T55` |
| `SB-MLA-057` | A threshold value can never be confused with a missing value | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T56` |
| `SB-MLA-058` | Tier-C capabilities are named, never approximated | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T57` |
| `SB-MLA-059` | The user need behind a Tier-C capability may be served by an independently derived feature | `P3` | `ABSENT` | `24_ml-advanced.md` | `SB-MLA-T57` |
| `SB-MLA-060` | No vendor model or weight file is read, converted or imported | `P0` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T57` |
| `SB-MLA-061` | A missing interpreter is a named, actionable failure | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T58` |
| `SB-MLA-062` | A long fit does not hold the global write lock | `P1` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T59` |
| `SB-MLA-063` | Every capacity cap is a declared limit, not a silent truncation | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T60` |
| `SB-MLA-064` | The model registry lists without materialising artifacts | `P2` | `PRESENT-OK` | `24_ml-advanced.md` | `SB-MLA-T61` |
| `SB-MLA-065` | A portfolio-scale ML run is bounded, cancellable and honestly reported | `P2` | `PARTIAL` | `24_ml-advanced.md` | `SB-MLA-T17`, `SB-MLA-T18` |
| `SB-NMR-001` | Carry the physical T2 axis through storage, IPC and UI | `P1` | `PRESENT-DIVERGENT` | `16_nmr.md` | `SB-NMR-T01`, `SB-NMR-T45` |
| `SB-NMR-002` | Validate array geometry before accepting a distribution | `P1` | `PARTIAL` | `16_nmr.md` | `SB-NMR-T02`, `SB-NMR-T04` |
| `SB-NMR-003` | Record acquisition and processing provenance | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T48` |
| `SB-NMR-004` | Reject defective recognition presets | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T05` |
| `SB-NMR-005` | Log every normalization or rebinning decision | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T49` |
| `SB-NMR-006` | Cutoffs ship absent and require explicit acceptance | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T06` |
| `SB-NMR-007` | Partition is conservative across a cutoff inside a bin | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T07`, `SB-NMR-T08` |
| `SB-NMR-008` | Support values, saddle-point and spectral methods without hiding the branch | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T06`, `SB-NMR-T10`, `SB-NMR-T11` |
| `SB-NMR-009` | Implement the cited thin-film spectral weighting | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T09` |
| `SB-NMR-010` | Emit both cutoff and spectral volumes in one run | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T10` |
| `SB-NMR-011` | T2 log mean is a time-windowed geometric mean | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T12`, `SB-NMR-T13` |
| `SB-NMR-012` | Timur–Coates parameters are semantic and unit-typed | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T14`, `SB-NMR-T18` |
| `SB-NMR-013` | Guard the BVI denominator without disguising it | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T19` |
| `SB-NMR-014` | Modified Coates is optional and calibrated | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T20`, `SB-NMR-T21` |
| `SB-NMR-015` | SDR is null and flagged in hydrocarbon-bearing intervals | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T22`, `SB-NMR-T23` |
| `SB-NMR-016` | Carbonate SDR requires a sourced surface relaxivity | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T50` |
| `SB-NMR-017` | Swanson remains a sourced, disabled extension | `P3` | `ABSENT` | `16_nmr.md` | `SB-NMR-T51` |
| `SB-NMR-018` | Full DMR is blocked until lambda is sourced | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T24` |
| `SB-NMR-019` | Gas hydrogen index is computed from gas density | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T25` |
| `SB-NMR-020` | DMR propagates input uncertainty and flags clamps | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T27` |
| `SB-NMR-021` | Fluid properties remain measured or explicitly sourced | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T52` |
| `SB-NMR-022` | Pc gain and offset carry both unit ends | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T30`, `SB-NMR-T31` |
| `SB-NMR-023` | Kappa is unit-typed and never defaulted | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T28`, `SB-NMR-T29` |
| `SB-NMR-024` | Every Pc output carries pressure unit, datum and saturation convention | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T32`, `SB-NMR-T53` |
| `SB-NMR-025` | T2-to-Pc requires a water-saturated distribution | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T33` |
| `SB-NMR-026` | Implement primary-source MRIAN as the canonical NMR saturation method | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T37`, `SB-NMR-T38` |
| `SB-NMR-027` | Clay-water conductivity coefficient ships absent | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T35`, `SB-NMR-T36` |
| `SB-NMR-028` | Keep effective and total irreducible saturation distinct | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T54` |
| `SB-NMR-029` | IP compatibility uses the resolved positive Z slope | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T39`, `SB-NMR-T40` |
| `SB-NMR-030` | Tortuosity is not silently dropped between saturation paths | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T41` |
| `SB-NMR-031` | NMR pore-volume ratios never masquerade as shale volumes | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T42` |
| `SB-NMR-032` | Pseudo-water substitution enforces ordering and water-leg calibration | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T43`, `SB-NMR-T44` |
| `SB-NMR-033` | Hydrocarbon typing is independently derived from published NMR literature | `P3` | `ABSENT` | `16_nmr.md` | `SB-NMR-T55` |
| `SB-NMR-034` | Echo inversion is excluded from first-release NMR | `P2` | `ABSENT` | `16_nmr.md` | `SB-NMR-T56` |
| `SB-NMR-035` | Detect but do not reproduce undocumented fast-relaxation correction | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T46` |
| `SB-NMR-036` | NMR heatmaps use the physical T2 axis | `P1` | `PRESENT-DIVERGENT` | `16_nmr.md` | `SB-NMR-T45` |
| `SB-NMR-037` | Every output carries method and parameter provenance | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T57` |
| `SB-NMR-038` | QC flags are explicit curves and run-summary counts | `P1` | `ABSENT` | `16_nmr.md` | `SB-NMR-T47` |
| `SB-PLG-001` | Ship three independently gated domain units | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T01`, `SB-PLG-T02` |
| `SB-PLG-002` | Type every production unit at ingest | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T03`, `SB-PLG-T04`, `SB-PLG-T05` |
| `SB-PLG-003` | Calibrate spinner slopes from zonal averages | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T06`, `SB-PLG-T07`, `SB-PLG-T08` |
| `SB-PLG-004` | Compute apparent fluid velocity exactly | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T09`, `SB-PLG-T10` |
| `SB-PLG-005` | Normalize multi-pass weights | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T11` |
| `SB-PLG-006` | Stop before unsupported phase rates | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T12` |
| `SB-PLG-007` | Store sensor geometry per family and tool | `P0` | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T13`, `SB-PLG-T14` |
| `SB-PLG-008` | Use an explicit three-phase holdup schema | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T15`, `SB-PLG-T16` |
| `SB-PLG-009` | Keep temperature-flow assumptions visible | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T17` |
| `SB-PLG-010` | Enforce selective-inflow data sufficiency | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T18` |
| `SB-PLG-011` | Differentiate cumulative inflow with a declared length | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T19` |
| `SB-PLG-012` | Make Chronolog epochs and operation order explicit | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T20`, `SB-PLG-T21`, `SB-PLG-T22` |
| `SB-PLG-013` | Restrict station import to evidenced grammars | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T23` |
| `SB-PLG-014` | Normalize nulls before station reduction | `P0` | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T24` |
| `SB-PLG-015` | Preserve phase semantics | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T25` |
| `SB-PLG-016` | Bind cutoff polarity to measurement family | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T26`, `SB-PLG-T27` |
| `SB-PLG-017` | Implement logarithmic attenuation bond index | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T28`, `SB-PLG-T29` |
| `SB-PLG-018` | Name and require the bond interpolation method | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T30` |
| `SB-PLG-019` | Derive coverage from valid array width | `P0` | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T31`, `SB-PLG-T32` |
| `SB-PLG-020` | Exclude collars without deleting data | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T33` |
| `SB-PLG-021` | Compute slurry acoustic impedance in declared units | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T34`, `SB-PLG-T35` |
| `SB-PLG-022` | Keep expected-CBL correlation optional and attributed | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T36` |
| `SB-PLG-023` | Keep probability and confidence separate | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T37`, `SB-PLG-T38` |
| `SB-PLG-024` | Validate probability-term switches | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T39` |
| `SB-PLG-025` | Explain the single-service ceiling | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T40` |
| `SB-PLG-026` | Implement channel detection with an explicit direction warning | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T41` |
| `SB-PLG-027` | Separate derivative, smoothing and vertical statistics | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T42`, `SB-PLG-T43` |
| `SB-PLG-028` | Preserve four-direction microdebond evidence | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T44` |
| `SB-PLG-029` | Keep cement classifications distinct | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T45` |
| `SB-PLG-030` | Enforce isolation-report interval length | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T46` |
| `SB-PLG-031` | Make waveform extraction reproducible | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T47` |
| `SB-PLG-032` | Emit four named casing-loss quantities | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T48`, `SB-PLG-T49` |
| `SB-PLG-033` | Retain signed apparent loss | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T50` |
| `SB-PLG-034` | Make prior-survey merge explicit | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T51` |
| `SB-PLG-035` | Require an ovality definition | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T52`, `SB-PLG-T53` |
| `SB-PLG-036` | Compute Barlow only from sourced strength | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T54`, `SB-PLG-T55` |
| `SB-PLG-037` | Source nominal casing geometry | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T56` |
| `SB-PLG-038` | Bind grades to their measurement quantity | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T57` |
| `SB-PLG-039` | Keep three despike stages distinct and auditable | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T58` |
| `SB-PLG-040` | Preserve named correction recipes | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T59` |
| `SB-PLG-041` | Distinguish one- and two-depth calibration | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T60` |
| `SB-PLG-042` | Refuse untracked environmental correction | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T61` |
| `SB-PLG-043` | Canonicalize casing weight and tension | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T62`, `SB-PLG-T63` |
| `SB-PLG-044` | Detect collars with correct window semantics | `P1` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T64` |
| `SB-PLG-045` | Stamp full run provenance | `P0` | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T65` |
| `SB-PLG-046` | Separate computed, imported and interpreted identities | `P0` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T66` |
| `SB-PLG-047` | Export machine-readable reports with masks | `P2` | `ABSENT` | `26_production-logging.md` | `SB-PLG-T67` |
| `SB-PLG-048` | Preserve array width and per-row validity end to end | `P0` | `PARTIAL` | `26_production-logging.md` | `SB-PLG-T31`, `SB-PLG-T68` |
| `SB-PLT-001` | Persist semantic intent and concrete resolution separately | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-002` | Resolve axes through one explicit precedence chain | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-003` | Overlay compatibility is quantity-and-unit typed | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-004` | Valid and display ranges remain distinct | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-005` | Unit-limit content is audited before activation | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-006` | One canonical histogram-bin contract | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-007` | Overplot thresholds expose the comparator | `P1` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-008` | Percentile probability and range position are different types | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-009` | Statistics disclose population, estimator and exclusions | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-010` | Regression is a versioned scientific result | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-011` | Pickett states what is and is not identifiable | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-012` | Hingle uses the negative reciprocal exponent | `P1` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-013` | Missing and out-of-range policy is channel-specific | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-014` | Multi-well allocation follows finite-pair screening | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-015` | Decimation preserves pairing, endpoints and provenance | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-016` | Depth-step reconciliation is explicit and conservative | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-017` | Zoom beyond loaded data triggers an identified refetch | `P1` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-018` | Linked selections are named, typed and persistable | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-019` | Every plot subscribes to the same invalidation contract | `P1` | `PRESENT-OK` | `23_plotting-interactivity.md` |  |
| `SB-PLT-020` | Plot-derived parameter writes carry full provenance | `P0` | `PRESENT-DIVERGENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-021` | Expression-valued channels are sandboxed and reproducible | `P2` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-022` | Faceting precedes decimation | `P2` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-023` | Every rendered chart is provenance-complete | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-024` | Vendor chart payloads are never transcribed | `P0` | `PRESENT-OK` | `23_plotting-interactivity.md` |  |
| `SB-PLT-025` | Plot templates are schema-versioned and scope-aware | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-026` | Export reruns the scientific draw at paper scale | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-027` | Plot state is portable without embedding restricted payloads | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-028` | Static and interaction layers have separate invalidation | `P1` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-029` | Asynchronous plot loads are generation-safe | `P0` | `PRESENT-OK` | `23_plotting-interactivity.md` |  |
| `SB-PLT-030` | Interactive canvases remain keyboard and assistive-technology reachable | `P1` | `PRESENT-OK` | `23_plotting-interactivity.md` |  |
| `SB-PLT-031` | No silent record truncation | `P0` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-PLT-032` | Plot performance is gated on declared hardware | `P0` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-033` | Pressure-gradient crossplots preserve the geomechanics sign convention | `P1` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-034` | Ternary plots normalize visibly | `P2` | `ABSENT` | `23_plotting-interactivity.md` |  |
| `SB-PLT-035` | Clay-volume interactive plots use the governed equation | `P2` | `PARTIAL` | `23_plotting-interactivity.md` |  |
| `SB-POR-001` | SandiBumi **MUST** present all deterministic porosity methods through one module family with one limiting contract, one flag contract and one output-naming contract. Today three modules ship three of each (§3.3), which is a maintenance liability and a source of answers that differ for reasons the analyst cannot see. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-002` | Every porosity method **MUST** emit an **unlimited** pair and a **limited** pair under distinct mnemonics. `phi_den` and `phi_dn` already do (`modules.rs:750-755`, `:846-851`); `phi_son` **MUST** be brought to the same contract. The unlimited curve is what a QC crossplot needs; the limited curve is what pay summation needs, and conflating them hides every clamp. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-003` | Every porosity method **MUST** emit a per-sample **branch and limit flag** stream (working name `PHIFLAG`) recording which branch produced the sample and which limit, if any, bound it. IP is the only incumbent that publishes such a stream (F17, F21 — codes 6, 7, 9, 16), and SandiBumi currently has **none** (§3.0). Every clamp identified in §3 — `VSH ≥ 0.95`, the `phie_max·(1−VSH)` ceiling that binds at 0.24 in §3.2's worked case, the `[1.95, 3.0]` and `[−0.015, 0.40]` shale-reduction clamps, `phi_son`'s `[0, 1]` — currently fires **silently**. This is the single cheapest fail-loud-where-they-fail-silent win in the domain. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-004` | The mnemonic dictionary **MUST** carry a **porosity family** (`PHIE`, `PHIT`, `PHIA`, `DPHI`, `NPHI_COR`, `PHIE_LIM`, …) and each porosity curve **MUST** carry the method and the volume convention that produced it. Two porosity modules **MUST NOT** write the same output mnemonic: `phi_den` and `phi_dn` both write `PHIE`/`PHIT` today, so the second run silently overwrites the first (§3.4). `curves.rs:21-37` registers fourteen families and no porosity family at all. Without this, F16's "PhiT is not one quantity" is unrepresentable and an imported Techlog `PhiT` resolves to a computed Geolog-convention `PHIT` by name collision. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-005` | Where a quantity has both a **correction** direction and a **forward-model** direction — pre-eminently excavation, `NPHI_corrected = NPHI + Δ` versus `NPHI_modelled = NPHI − Δ` (F7) — SandiBumi **MUST** implement two separately named functions. It **MUST NOT** implement one function with a sign flag. Techlog's two published forms differ by exactly this and are both correct in their own context; a single sign-flagged function is the shape that eventually gets called with the wrong flag. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-006` | Every porosity method that consumes a shale/clay volume **MUST** consume a **typed** volume and **MUST** refuse an untyped one. A `VSH` (shale-endpoint) and a `VCL` (wet-clay-endpoint) volume are not interchangeable (F15); the endpoint subtracted must match the volume supplied. The refusal is the requirement — silently accepting either is how a 100 %-shale-point correction gets applied to a clay volume. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-007` | `ModuleSpec` parameters **MUST** carry a **source citation and evidence tier** alongside name, unit, default and range, and the dialog **MUST** surface them. The struct has no such field today (§3.9), so §5's `ABSENT — ships with no default` discipline currently has nowhere to live in the product — only in this document. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-008` | Clay-bound-water porosity **MUST** be defined once as `PHIT_SH = (RHO_DSH − RHO_SH)/(RHO_DSH − RHO_W)` with `RHO_W` the **formation water** density, in one shared helper, and **MUST** be exported to the `CLY` chapter's `clsr_porosity_corrected` (SB-CLY-044). The shared helper exists and uses the correct fluid (`modules.rs:705-710`); the requirement pins it and publishes it across the seam. The **shale-subtraction** term `(RHO_MA − RHO_SH)/(RHO_MA − RHO_FL)` is a **different quantity** and **MUST NOT** share a name with it (F16). | `P0` |  | `11_porosity.md` |  |
| `SB-POR-009` | `PHIT ≥ PHIE` **MUST** hold at every sample by construction — limit `PHIE` first, then rebuild `PHIT` from the limited value (Geolog's ordering, F21). `phi_den:743-747` and `phi_dn:839-843` already do this; the invariant **MUST** additionally be asserted, not merely relied on. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-010` | Every porosity curve **SHOULD** carry, in the project audit trail, the method name, the full parameter set and the input curve identities that produced it, sufficient to re-derive it without the session. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-011` | Matrix density **MUST** be a single shared parameter across modules that a documented workflow chains. `gascorr` ships `RHO_MA 2.65` while `phi_den`, `phi_dn` and `condflag` ship `2.645` (`modules.rs:1637`ish vs `:687`, `:775`, `:1280`) and `gascorr`'s own doc instructs chaining them (§3.6). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-012` | SandiBumi **SHOULD** implement the `CSR` clay-shale-ratio bridge between the `VSH` and `VCL` endpoint families (F15's four IP relations). Neither Geolog nor Techlog can round-trip between the conventions at all, so this is a capability rather than a port. `CSR` **MUST** ship with no default (§5) — silently defaulting it to 1.0 is wrong in every shaly sand in the flattering direction. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-013` | The **shale-correction convention** **MUST** be an explicit, named, per-method selection — `NORMALISED` (Geolog: reduce, floor, then rescale by `1 − VSH`) or `SUBTRACTIVE` (Techlog: one pre-correction, answer already effective) — and one method **MUST NOT** mix them. `phi_son`'s `RHG` branch mixes them today: a normalise-convention transform paired with a Wyllie subtractive shale term (`modules.rs:907` + `:915`, §3.3). The convention is worth **1.30–1.55 p.u.** across vendors on identical inputs (F2) and is invisible in every parameter value. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-014` | Sonic porosity methods **MUST** be named for what they compute and **MUST NOT** be named for a method they are not. Specifically: the branch at `modules.rs:907` **MUST** be renamed from `RHG (Raymer-Hunt-Gardner)` to `FIELD_OBSERVED`, its coefficient 0.625 **MUST** become a parameter, and any method offered as "Raymer-Hunt-Gardner" **MUST** be one of the three published renderings in F3 with its vendor identified. Shipping IP's recommended-over-Wyllie method name against a different transform is the kind of overclaim CONTRACT §5 warns costs the deal. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-015` | Non-Wyllie sonic branches **MUST** operate on the shale-reduced, matrix-floored slowness and **MUST** rescale by `(1 − VSH)`, per Geolog's **executed code**, not its doc block. The doc-block form differs by up to **6.3 p.u.** (F4). Wyllie **MUST** retain the subtractive form, on which Geolog and Techlog agree exactly. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-016` | Matrix transit time **MUST** be selected per lithology from a cited family, and SandiBumi **MUST NOT** ship a single lithology-agnostic default. Techlog's `DTma 47.5` applied to a clastic section moves Wyllie porosity by **4.5 p.u.** against a sandstone value (F1). The sandstone family itself spans 1.65 p.u. across four cited vendor values and therefore ships as a **cited choice list, not a number** (§5). | `P0` |  | `11_porosity.md` |  |
| `SB-POR-017` | The Wyllie lack-of-compaction correction **MUST** be guarded so it can only reduce porosity: SandiBumi **MUST** require `Cp ≥ 1` (equivalently `DT_SH > 100 µs/ft`) and **MUST** refuse or flag the sample otherwise. At the shipped `DT_SH = 90` the correction **adds 2.30 p.u. to `PHIE`** — the opposite of its documented purpose (§3.3) — with every value inside every declared range. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-018` | Any shale-corrected slowness **MUST** be floored at `DT_MA` before use. Unfloored, Wyllie returns negative porosity and Raiga inverts its ratio above `V_sh = (Δt − Δtma)/(Δtsh − Δtma)` — `V_sh = 0.195` at ordinary values (F5). Geolog added this floor in July 1997 and dates it in its own history block; Techlog publishes the equation without it. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-019` | Where a method requires a matrix endpoint **and** a fitted exponent, the two **SHOULD** be selected as a **matched pair per mineral** (Geolog's 55.5/1.60, 47.6/1.76, 43.5/2.00), not as two independent parameters (F1). | `P2` |  | `11_porosity.md` |  |
| `SB-POR-020` | SandiBumi **SHOULD** implement exactly one Raymer-Hunt-Gardner rendering as the default, cite which vendor's rendering it is, and **MAY** expose the other two as labelled comparison methods. Three vendors ship three different closed forms under one name (F3); claiming "Raymer-Hunt- Gardner" without saying whose is not a defensible claim. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-021` | SandiBumi **MUST** implement a **chart-free analytic neutron-density crossplot** as its primary N-D porosity method, following the Bateman & Konen (1977) family that Geolog's `phi_dnbk` implements and that Techlog's neutron-sonic algorithm independently reproduces in structure (F13). This is the method that lets SandiBumi ship a real crossplot porosity without transcribing a single vendor chart value, and it is what §3.2's arithmetic average is standing in for at a cost of **1.64–1.79 p.u.** | `P0` |  | `11_porosity.md` |  |
| `SB-POR-022` | Chart-derived porosity paths **MUST** come only from SandiBumi's own gated digitisation pipeline, with its validation gates enforced at build time. `nphimat`/`neutron_charts.rs` is the pattern (§3.6); no vendor lookup table is transcribed, ever. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-023` | The arithmetic average `(φD + φN)/2` and the RMS `sqrt((φD² + φN²)/2)` **MUST NOT** be presented as crossplot porosity methods, and the doc string at `modules.rs:770-771` claiming they are *"the standard analytic equivalent"* of chart lookups **MUST** be removed. They **MAY** ship as explicitly labelled quick-look comparison curves. **No vendor ships either as a porosity method** (F14), and IP states of the field shortcuts verbatim that *"they should not be used for anything other than this"*. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-024` | N-D crossplot porosity **MUST** refuse to run on a neutron curve whose **matrix units** are not declared, and **MUST** state the declared basis in its output provenance. A limestone-unit neutron against a sandstone matrix reads **~0.04 v/v low in clean water sand** — a fact `condflag`'s doc string already states verbatim (`modules.rs:1261-1264`) and `phi_dn` neither states nor checks (§3.6). All three vendors solve this with chart data; SandiBumi solves it with `nphimat` and must then require it. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-025` | Where a method's endpoints depend on borehole fluid salinity, SandiBumi **SHOULD** evaluate the fresh and salt cases and interpolate on fluid density, per Geolog's two-call structure (F13). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-026` | Gas crossover **SHOULD** be detected and surfaced as a flag on the porosity output. `condflag` already computes `XOVER_FLAG` (`modules.rs:1303`); the requirement is the wiring. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-027` | A **neutron-sonic** crossplot porosity **SHOULD** be offered, built on the same two-point apparent-endpoint lever structure as SB-POR-021, and **MUST NOT** reproduce Techlog's published `φ_sh` form (see SB-POR-053 and §7). | `P2` |  | `11_porosity.md` |  |
| `SB-POR-028` | The shale-reduction clamps currently hard-coded at `modules.rs:826-827` (`[1.95, 3.0]` g/cc and `[−0.015, 0.40]` v/v) **MUST** become cited parameters, and hitting them **MUST** raise SB-POR-003's flag. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-029` | The apparent hydrocarbon **electron density** **MUST** be the *Conventional* form, and its validity envelope **MUST** be stated in the product: it tracks the Gaymard-Poupon quadratic to better than **1.5 %** for `ρ_h ≥ 0.225 g/cc` and degrades monotonically to **−3.1 % at 0.10** (F10). IP's *Modified* form gives **0.0761 vs 0.2452 g/cc at ρ_h = 0.20** — a factor 3.22 — and IP's own two modules disagree about which to use. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-030` | The hydrocarbon **hydrogen index** on the neutron side **MUST** be the Gaymard-Poupon quadratic `N_h = 0.15 + 0.2(0.9 − ρ_h)²`, corroborated by Techlog's `9ρN_h` to **1.2 %** and by Poupon's own Eq A-9 to **1.5 %** at gas density. Geolog's `α = 1.67ρ − 0.17` is **1.51× Poupon's gas value** and over-corrects `NPHI` by **+4.1 p.u.** (F11); it is a fix Geolog made on its density side and never propagated to its neutron side, and **MUST NOT** be adopted. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-031` | The hydrocarbon correction **SHOULD** be structured as Poupon 1971's `A`/`B` factor architecture (`A` on the density side, `B` on the neutron side, both scaled by `φ·Shr`), which is the primary source all three vendors cite and the only structure in which the vendors' variants can be compared term by term. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-032` | Mud-filtrate density `ρmf` and filtrate hydrogen-loss `Pmf` **MUST** be parameters, not literals. Poupon's `ρmf(1 − Pmf) = 0.98` is a *worked-example* value, not a default. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-033` | The hydrocarbon chain **MUST** refuse or hard-flag samples outside the validity bounds of the selected model, specifically: `ρ_h < 0.1414 g/cc` (IP Modified goes negative), `ρ_h < 0.1018 g/cc` (Geolog `α` goes negative), and `ρ_h < 0.188 g/cc` (any `N_h` exceeding methane's hydrogen mass fraction `4 × 1.008 / 16.04 = 0.2514`, which is stoichiometry, not a parameter). **Dry gas at shallow-to-moderate reservoir pressure sits inside that band routinely** (F9), and a negative apparent density biases density porosity **low** exactly where the correction matters most. This is the most consequential fail-loud requirement in the chapter. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-034` | Hydrocarbon model selection **MUST** be explicit and named by vendor, with all variants available for cross-tool verification, and **MUST** be recorded in the output provenance. Four vendor renderings exist for one physical quantity and three of them fail in gas. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-035` | The flushed-zone saturation exponent (`Sxo = Swe^n`) **MUST** ship with **no default** and **MUST** be an explicit user decision. Geolog defaults **0.2** and Techlog/IP default **1** — at `Swe = 0.30` that is `Sxo = 0.786` versus `0.300`, a **0.49 difference in `Sxo`** feeding every hydrocarbon correction, with no parameter ever out of range (F12). These are opposite modelling assumptions, not a tolerance. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-036` | A per-zone **force-100 %-wet** switch **SHOULD** be offered, suppressing all hydrocarbon corrections to porosity and raising SB-POR-003's flag. IP's PHIFLAG 16 is the only such switch any vendor publishes (F21). | `P2` |  | `11_porosity.md` |  |
| `SB-POR-037` | The computed hydrogen index **SHOULD** be asserted against the stoichiometric ceiling 0.2514 at every sample as a cheap internal consistency check. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-038` | The existing `gascorr` module **MUST** be documented as a **density-log correction**, distinct from the porosity hydrocarbon chain, and the two **MUST NOT** be chained without an explicit statement of which correction has already been applied — double-correcting is otherwise invisible. `gascorr`'s non-convergence discipline (`modules.rs:1766-1782`, samples stay MISSING) **MUST** be preserved and extended to the porosity chain. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-039` | SandiBumi **MUST** implement the neutron excavation effect using the **additive** rendering, `K·(0.02φ + φ^1.8·S_HC·(0.6493 + 0.2149·S_HC))·(1 − S_HC)` in Techlog's parameterisation, with the lithology term as `ρma^2.1` or the equivalent `(ρma/2.65)²` — the two independent implementations that agree to **0.8 %** across the lithology range (F8). Techlog's multiplied rendering is a **typesetting defect** worth a factor **220** (F7) and **MUST NOT** be implemented. IP SSM's `sqrt(ρma/2.65)` is a **four-fold weaker** lithology sensitivity and is the outlier against two independent implementations. The term is **2.9–3.2 p.u.** at the reference case and SandiBumi has none of it today (§3.0). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-040` | Excavation **MUST** be exposed in both directions as two named functions per SB-POR-005. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-041` | Excavation **SHOULD** be suppressed for epithermal and array-neutron tools — real physics that IP and Geolog silently ignore — but the gate **MUST** key on a **resolved tool identity from SandiBumi's own tool register**, never on a vendor tool-name string. Techlog's gate string contains a token matching nothing (`APSC`), a token matching two entries (`SNP`), a token reachable only through a tool whose casing differs between its own two artefacts (`BPHI`/`EcoScope`), and its enum cannot be split on its own delimiter without corrupting every index past the thirteenth (F20). SandiBumi **MUST NOT** copy the string. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-042` | The published lithology constants `K` for the classic `K(2φ²Sw + 0.04φ)(1 − Sw)` form **SHOULD** be obtained from Segesman & Liu (1971) or *Log Interpretation Principles* (1969) Ch. 13 and used to adjudicate SB-POR-039's exponent. Until then the exponent ships as a **cited choice between two agreeing implementations**, not as a settled value. | `P3` |  | `11_porosity.md` |  |
| `SB-POR-043` | The high-shale kill threshold **MUST** be a cited parameter, not a literal. `VSH >= 0.95` is hard-coded at `modules.rs:732` and `:817`, inherited from Geolog, and produces a **step discontinuity** in `PHIE` at a value the analyst cannot move. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-044` | A **smooth** high-shale roll-off **SHOULD** be offered as an alternative to the step, following IP's `(PhiMax + ΔPhiMax)(1 − Vcl)·10^(−10(Vcl − VclCutoff)^1.6)` shape. Its three parameters ship with **no defaults** — IP publishes none (F21). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-045` | The value `PHIE` is **set to** when the floor binds **MUST** ship with no default and **MUST** be a documented user decision. IP's own manual states **0.001** and **0.0001** for the same quantity in three places (F17); SandiBumi hard-codes `0.001` at `modules.rs:335` with no note that the question is open (§3.5). The quantity only bites in tight and zero-porosity intervals — which is exactly where a net-pay cutoff sits. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-046` | If the `VSILT = 1 − VCL − PHIE/PHIMAX` index is offered, IP's own do-not-trust warning **MUST** be surfaced with it. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-047` | Porosity methods **MUST** accept the existing `BADHOLE` flag (`modules.rs:1183-1241`) as a declared input and **MUST** record its effect through SB-POR-003, rather than depending on the analyst remembering to set a generic Mask (§3.7). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-048` | Porosity methods **MUST** consume `condflag`'s `COAL_FLAG`, `TIGHT_FLAG` and `COND_FLAG` (`modules.rs:1301-1305`) as declared inputs with defined branch behaviour. SandiBumi's conditioning module is **better than any incumbent's** on this point — parameterised, bed-thickness aware, and bad-hole aware so a washout is never called coal — and it is currently not wired to the modules that need it. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-049` | SandiBumi **MUST NOT** ship hard-coded lithology-kill literals. Techlog's `φ_n > φ_d ∧ 2.91 ≤ ρ_b ≤ 3.5 ∧ φ_n ≤ 0.04 ⇒ φ = 0` is the only numeric kill any vendor publishes and it will zero real porosity in a tight carbonate with no flag and no parameter (F24). | `P2` |  | `11_porosity.md` |  |
| `SB-POR-050` | Every iterative porosity solve **MUST** expose its convergence tolerance and iteration cap as parameters, **MUST** state the tolerance as an inequality on the absolute change, and **MUST** treat cap-exhaustion as non-convergence rather than emitting the last iterate. Techlog publishes its N-D test as an **equality** at a **1 p.u.** tolerance — an order of magnitude looser than its own hydrocarbon loop — and ships two different caps (10 in the script, 50 in the doc) for the same loop (F19). `gascorr` already sets the correct precedent (`modules.rs:1766-1782`). | `P1` |  | `11_porosity.md` |  |
| `SB-POR-051` | Where more than one unknown may be varied to reach a solution, the **precedence MUST be documented and deterministic**. IP is the only vendor that publishes one — Hc density, then grain density, then `Vcl`, then, as a last resort, **reducing the input log itself** under PHIFLAG 6/7 (F18). A four-free-parameter solve with no stated order is under-specified, and Geolog and Techlog leave it so. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-052` | Invalid solver configurations **MUST** be rejected at configuration time. IP documents verbatim that a variable-`Sxo` run requires another variable flag to be active, and then does not enforce it (F18). | `P2` |  | `11_porosity.md` |  |
| `SB-POR-053` | Shale porosity in any crossplot **MUST** be formed as a fluid-minus-matrix span. SandiBumi **MUST NOT** implement Techlog's published neutron-sonic `φ_sh = (ΔT_shale − 47.6)/(ΔT − 47.6)`, which divides by the sample's own transit time, returns **4.23** in a fast clean sand, and removes **21 p.u.** at `Vsh = 0.05` (F6). Where a rendered vendor equation is dimensionally inconsistent with every sibling equation in the same product, the vendor equation is the finding. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-054` | SandiBumi **MUST** state one canonical sign convention for every matrix/fluid/log transform and **MUST** carry a test proving algebraic identity with the inverted forms that Geolog (`por_from_rhob.lls`) and Techlog (N-D crossplot page) publish (F22). Two independent vendors write these with both numerator and denominator inverted; a reader porting either line verbatim without noticing both flips ships a sign error that is invisible in review. | `P1` |  | `11_porosity.md` |  |
| `SB-POR-055` | Every petrophysical parameter in this domain **MUST** carry a source string and tier, and where the held sources disagree with no defensible adjudication the parameter **MUST** ship `ABSENT — ships with no default` with the competing values visible. This is a standing project decision. It applies immediately to `RHO_SH`, `RHO_DSH`, `NPHI_SH`, `DT_SH` and `RHO_MA`, all of which ship today as uncited numbers (§3.1) — and `RHO_DSH = 2.65` matches **no held source at all** while setting `PHIT_SH` a factor **1.73 low** against the nearest vendor. For Techlog specifically, neither its script nor its doc may be treated as authoritative alone: **nine** shipped quantities disagree between the two, including two values inside one equation (F23). | `P0` |  | `11_porosity.md` |  |
| `SB-POR-056` | Porosity **MUST** be carried internally in `v/v`, transit time in `µs/ft` and density in `g/cc`, with display units a presentation concern. Geolog ships `K/M3` and `US/M` internally (F22) and Techlog ships filtrate salinity in **four** unit/value combinations (F23); the canonical-unit rule is what keeps an import from either from arriving 1000× out. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-057` | Quick-look comparison curves **MUST** be visually and structurally distinguishable from computed methods — different mnemonic family, flagged in provenance, excluded by default from pay summation. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-058` | A module **MUST NOT** present a parameter its computation does not read. `sspw_spec` declares `NPHI_MAT`, `NPHI_SH` and `NPHI_FL` (`ssc.rs:370`, `:372`, `:377`) and `sspw()` reads none of them (§3.8). Until the re-port against `sspw.lls` is signed off, those parameters **MUST** be removed from the spec or marked inactive in the dialog. An honest module header (`ssc.rs:37-41`) is invisible to `moduleDialog.ts`; a user who tunes `NPHI_SH` and sees no change has been told a falsehood by the UI. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-059` | `sspw()`'s gas conditioning **MUST** be brought to the same RMS midpoint `sqrt((φD² + NPHI²)/2)` that `ssc()` uses. `ssc.rs:433` still runs the weight that `ssc.rs:172-178` records as *inverting the D-N crossover* and that was fixed in `ssc()` on 2026-07-29. At `φD = 0.25, NPHI = 0.10` the two shipped modules return **0.1903943** and **0.1431782** — **4.72 p.u. apart, with `sspw` biased low in gas**, the direction that under-reports pay. | `P0` |  | `11_porosity.md` |  |
| `SB-POR-060` | SandiBumi **SHOULD** import vendor parameter sets (IP `.par`-style, Geolog `.info` defaults, Techlog parameter decks) as **cited, tiered, read-only** parameter sets that populate SB-POR-007's provenance rather than becoming SandiBumi defaults. | `P2` |  | `11_porosity.md` |  |
| `SB-POR-061` | A porosity **method audit report** **SHOULD** be producible per well: every method run, every parameter with its source and tier, every flag raised with its sample count, and every limit that bound. This is the deliverable-defence artefact none of the three incumbents produces. | `P3` |  | `11_porosity.md` |  |
| `SB-POR-062` | Core-porosity calibration **MAY** be offered as a post-check against computed porosity, reporting bias and scatter per method, with no automatic adjustment of any parameter. | `P3` |  | `11_porosity.md` |  |
| `SB-RPH-001` | Use one typed SI-with-GPa elastic state | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T01`, `SB-RPH-T02`, `SB-RPH-T03` |
| `SB-RPH-002` | Derive the complete isotropic elastic suite | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T04`, `SB-RPH-T05` |
| `SB-RPH-003` | Implement guarded Gassmann forward and inverse | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T06`, `SB-RPH-T07`, `SB-RPH-T08` |
| `SB-RPH-004` | Re-synthesize density and velocities from the substituted state | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T06`, `SB-RPH-T09` |
| `SB-RPH-005` | Persist method, state and failure provenance | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T10`, `SB-RPH-T11` |
| `SB-RPH-006` | Derive fluid properties from published physics | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T12` |
| `SB-RPH-007` | Select a named fluid-mixing law | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T13`, `SB-RPH-T14` |
| `SB-RPH-008` | Preserve Brie's liquid-lumping semantics | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T13`, `SB-RPH-T14`, `SB-RPH-T15` |
| `SB-RPH-009` | Compute mineral bounds beside every mixture | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T16` |
| `SB-RPH-010` | Govern elastic endpoints without copying vendor tables | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T17`, `SB-RPH-T18` |
| `SB-RPH-011` | Keep critical and depositional porosity distinct | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T19` |
| `SB-RPH-012` | Implement critical-porosity and suspension domains | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T20` |
| `SB-RPH-013` | Implement Krief with the cited exponent | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T21` |
| `SB-RPH-014` | Require Hertz–Mindlin adhesion explicitly | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T22`, `SB-RPH-T23` |
| `SB-RPH-015` | Keep empirical shear scaling separate | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T24` |
| `SB-RPH-016` | Distinguish soft, stiff and external dry frames | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T25` |
| `SB-RPH-017` | Gate effective-medium models by their validity domains | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T26` |
| `SB-RPH-018` | Support finite-shear pore fillers only from primary equations | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T27` |
| `SB-RPH-019` | Specify Bayesian inversion without cloning its solver | `P3` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T28` |
| `SB-RPH-020` | Lock empirical shear correlations to their native units | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T29`, `SB-RPH-T30` |
| `SB-RPH-021` | Keep alternative shear methods semantically addressed | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T31` |
| `SB-RPH-022` | Produce a complete Backus TI tensor | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T32`, `SB-RPH-T33` |
| `SB-RPH-023` | Make SH/SV assignment an explicit measured decision | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T34`, `SB-RPH-T35` |
| `SB-RPH-024` | Separate TIV, tilted-TIV and orthotropic input contracts | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T36` |
| `SB-RPH-025` | Reject non-positive-definite stiffness states | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T37`, `SB-RPH-T38` |
| `SB-RPH-026` | Emit the full named elastic-attribute suite | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T04`, `SB-RPH-T39` |
| `SB-RPH-027` | Treat Elastic Impedance as unit-system-dependent | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T40` |
| `SB-RPH-028` | Provide exact and declared approximate reflectivity | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T41`, `SB-RPH-T42` |
| `SB-RPH-029` | Build synthetics from explicit wavelet and sampling state | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T43` |
| `SB-RPH-030` | Keep simple dispersion distinct from fitted dispersion | `P3` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T44` |
| `SB-RPH-031` | Consume array logs without flattening their geometry | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T45` |
| `SB-RPH-032` | Make image geometry corrections reversible | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T46`, `SB-RPH-T47` |
| `SB-RPH-033` | Condition buttons and pads after speed correction | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T48` |
| `SB-RPH-034` | Recover dip direction with full quadrants | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T49`, `SB-RPH-T50` |
| `SB-RPH-035` | Prevent magnetic-declination double application | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T51`, `SB-RPH-T52` |
| `SB-RPH-036` | Calibrate image porosity to interval electrical parameters | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T53` |
| `SB-RPH-037` | Require calibrated fracture-aperture constants and convention | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T54`, `SB-RPH-T55` |
| `SB-RPH-038` | Expose all three Terzaghi policies | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T56`, `SB-RPH-T57` |
| `SB-RPH-039` | Compute fracture intensity with explicit geometry | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T58` |
| `SB-RPH-040` | Name pooled and area-weight statistics separately | `P2` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T59` |
| `SB-RPH-041` | Refuse metadata-free fracture outputs | `P0` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T60`, `SB-RPH-T61` |
| `SB-RPH-042` | Preserve non-destructive core-photo conditioning | `P0` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T62`, `SB-RPH-T63` |
| `SB-RPH-043` | Separate colour correction from detail-changing operations | `P1` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T64` |
| `SB-RPH-044` | Require interval geometry before core-log extraction | `P0` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T65` |
| `SB-RPH-045` | Keep white-light and ultraviolet meanings distinct | `P0` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T66`, `SB-RPH-T67` |
| `SB-RPH-046` | Keep image-derived lithology a labeled proxy | `P1` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T68`, `SB-RPH-T69` |
| `SB-RPH-047` | Preserve fractional lane geometry and inspectable strips | `P1` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T70`, `SB-RPH-T71` |
| `SB-RPH-048` | Keep automatic core advice proposal-only | `P1` | `PRESENT-OK` | `25_fluidsub-rockphysics.md` | `SB-RPH-T72`, `SB-RPH-T73` |
| `SB-RPH-049` | Make every method batch-safe and versioned | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T74` |
| `SB-RPH-050` | Validate method-specific inputs before calculation | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T75` |
| `SB-RPH-051` | Persist enough provenance to reproduce every number | `P0` | `PARTIAL` | `25_fluidsub-rockphysics.md` | `SB-RPH-T10`, `SB-RPH-T76` |
| `SB-RPH-052` | Never accept and ignore a parameter | `P1` | `ABSENT` | `25_fluidsub-rockphysics.md` | `SB-RPH-T77` |
| `SB-SAT-001` | Name every saturation model by its equation, never by a vendor adjective | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T01`, `SB-SAT-T02`, `SB-SAT-T30` |
| `SB-SAT-002` | Ship effective and total Archie as separate named methods | `P0` | `ABSENT` | `12_saturation.md` | `SB-SAT-T03`, `SB-SAT-T04` |
| `SB-SAT-003` | Ship a vendor alias table and resolve imports through it | `P1` | `ABSENT` | `12_saturation.md` | `SB-SAT-T01`, `SB-SAT-T05` |
| `SB-SAT-004` | Simandoux: two variants, with `C` on the Schlumberger variant only | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T06`, `SB-SAT-T07` |
| `SB-SAT-005` | Simandoux `a` ships with no default | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T08`, `SB-SAT-T31` |
| `SB-SAT-006` | Indonesia with a parameterised shale exponent | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T09`, `SB-SAT-T10`, `SB-SAT-T30` |
| `SB-SAT-007` | Woodhouse Tar as a cited alias of Indonesia `k = 2` | `P2` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T10`, `SB-SAT-T35` |
| `SB-SAT-008` | Total Shale as a preset of `simandoux_modified_slb` with `n` fixed at 2 | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T11`, `SB-SAT-T12` |
| `SB-SAT-009` | Juhász: shale-derived coefficient, shale-based normalization, model's own `m*` | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T13`, `SB-SAT-T14`, `SB-SAT-T30` |
| `SB-SAT-010` | Juhász MUST flag a negative excess-conductivity coefficient | `P0` | `ABSENT` | `12_saturation.md` | `SB-SAT-T15` |
| `SB-SAT-011` | Waxman-Smits with `a` exposed | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T16` |
| `SB-SAT-012` | `B` MUST be a unit-typed quantity, canonically `L·S/(eq·m)` | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T17`, `SB-SAT-T18` |
| `SB-SAT-013` | `Qv` MUST be unit-typed, canonically meq/mL | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T19`, `SB-SAT-T20` |
| `SB-SAT-014` | `B(T,Rw)` MUST consume typed °C and clamp `B ≥ 0` | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T21`, `SB-SAT-T22` |
| `SB-SAT-015` | `B` method ships with no default and four named options | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T23`, `SB-SAT-T31` |
| `SB-SAT-016` | Dual water ships in two named forms | `P2` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T24`, `SB-SAT-T25` |
| `SB-SAT-017` | The excess-conductivity coefficient MUST be `Swb·(Cwb − Cw)` | `P1` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T26` |
| `SB-SAT-018` | `vQ` MUST switch temperature form on the expansion branch | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T27` |
| `SB-SAT-019` | α MUST include the Debye-Hückel activity ratio | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T28` |
| `SB-SAT-020` | β MUST carry the salinity dilution factor | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T29` |
| `SB-SAT-021` | `Qv > 1/vQ` MUST flag; `Swb ≤ 1 − φe/φt` MUST clamp | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T32` |
| `SB-SAT-022` | `vQ0` ships absent | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T33` |
| `SB-SAT-023` | The effective back-out is per model, never blanket | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T34`, `SB-SAT-T35`, `SB-SAT-T36` |
| `SB-SAT-024` | `SWE_IRR` is an effective quantity, transformed per model | `P2` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T37` |
| `SB-SAT-025` | Every method emits a clipped and an unclipped curve | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T38` |
| `SB-SAT-026` | Never emit a bare `SW`; always emit a method-flag curve | `P1` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T39`, `SB-SAT-T40` |
| `SB-SAT-027` | One shared root-finder with Geolog's guards | `P1` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T12`, `SB-SAT-T41` |
| `SB-SAT-028` | Non-convergence MUST return null, never a partial iterate | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T41` |
| `SB-SAT-029` | Inherit the documented guard rails, including the volume detail | `P1` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T42`, `SB-SAT-T43` |
| `SB-SAT-030` | `Vsh → 1` MUST flag before the singularity, not silently return water | `P1` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T44` |
| `SB-SAT-031` | `Rw` ships with no default | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T45` |
| `SB-SAT-032` | `Rw` correlations with the temperature conversion bound to the branch | `P1` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T45`, `SB-SAT-T46`, `SB-SAT-T48` |
| `SB-SAT-033` | The Kennedy floor is 0.0412 and the vendor doc is wrong | `P2` | `PRESENT-OK` | `12_saturation.md` | `SB-SAT-T47` |
| `SB-SAT-034` | `a`, `m`, `n`, `m*`, `n*` ship with no default | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T49` |
| `SB-SAT-035` | `Rsh` and `φt_sh` ship with no default, and the current values are withdrawn | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T50` |
| `SB-SAT-036` | Two named `m*`/`n*` routes, with core preferred | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T51` |
| `SB-SAT-037` | Shell / Elan variable `m` as one parameterised route with no default coefficient | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T31`, `SB-SAT-T52` |
| `SB-SAT-038` | Every parameter carries a source string, and the build fails without one | `P0` | `ABSENT` | `12_saturation.md` | `SB-SAT-T31` |
| `SB-SAT-039` | `MUDBASE` is model-scoped | `P3` | `ABSENT` | `12_saturation.md` | `SB-SAT-T53` |
| `SB-SAT-040` | Clay-bound-water `F`: both unit forms, `ρ_brine` open, `Swb = 1 − F` opt-in only | `P3` | `ABSENT` | `12_saturation.md` | `SB-SAT-T54`, `SB-SAT-T55` |
| `SB-SAT-041` | Poupon-Aguilera / Poupon-Tixier with the laminated interlock | `P3` | `ABSENT` | `12_saturation.md` | `SB-SAT-T56`, `SB-SAT-T57` |
| `SB-SAT-042` | The SSM bound-water cap fires and is flagged | `P3` | `ABSENT` | `12_saturation.md` | `SB-SAT-T58` |
| `SB-SAT-043` | A saturation result carries its parameters, their sources and their papers | `P0` | `ABSENT` | `12_saturation.md` | `SB-SAT-T59` |
| `SB-SAT-044` | Surface the cross-tool disagreement to the interpreter | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T60` |
| `SB-SAT-045` | Model-selection guidance, exposed as guidance and never as an automatic switch | `P2` | `ABSENT` | `12_saturation.md` | `SB-SAT-T60` |
| `SB-SAT-046` | Sxo and the flushed zone | `P3` | `PARTIAL` | `12_saturation.md` | `SB-SAT-T61` |
| `SB-SAT-047` | One model, one number, whichever engine computes it | `P0` | `PRESENT-DIVERGENT` | `12_saturation.md` | `SB-SAT-T30`, `SB-SAT-T61` |
| `SB-SAT-048` | LRLC coefficients are declared as one field's calibration | `P2` | `PRESENT-UNVERIFIED` | `12_saturation.md` | `SB-SAT-T59`, `SB-SAT-T62` |
| `SB-SAT-049` | Carry the Worthington 1985 type per model | `P4` | `ABSENT` | `12_saturation.md` | `SB-SAT-T59` |
| `SB-SAT-050` | Apparent-`Rw` inversion, one per saturation model | `P3` | `ABSENT` | `12_saturation.md` | `SB-SAT-T63` |
| `SB-SAT-051` | Per-mineral conductivity is a recorded capability gap, not a silent one | `P4` | `ABSENT` | `12_saturation.md` | `SB-SAT-T60` |
| `SB-SHR-001` | **Every** height-domain saturation model MUST convert the height above the free-water level into the unit in which that model's own coefficients are defined, and the conversion MUST be driven by the project's declared depth unit. No branch of any saturation-height model may consume a raw height. Adding a new model family MUST NOT be possible without declaring the unit its length-dimensioned coefficients are in. | `P0` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T01`, `SB-SHR-T02` |
| `SB-SHR-002` | Every shape parameter carrying a length dimension — Skelt-Harrison `B` and `D`, Thomeer entry height `Hd`, Brooks-Corey entry height `He`, and the free-water level itself — MUST carry an explicit unit in its registration, and the product MUST re-express its value when the project depth unit changes. A length-dimensioned parameter with a hard-coded unit string is a defect. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T03` |
| `SB-SHR-003` | The domain MUST refuse to perform height arithmetic when the project depth unit is undeclared, and MUST say so by name. It MUST NOT substitute a default unit. This requirement binds the domain's own entry points; the carrier's parse-time behaviour is `21_data-io.md`'s. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T04` |
| `SB-SHR-004` | Every height-dimensioned output curve, parameter prompt, plot axis and export header in this domain MUST be labelled in the project's declared depth unit. A numerically correct value under an incorrect unit label is a reportable defect. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T05` |
| `SB-SHR-005` | Water density, hydrocarbon density and reservoir `σ·cosθ` MUST each have **exactly one** default in the product, shared by the fitting path and the forward-apply path. Where a fit and an apply can disagree on a fluid property, the product MUST refuse the apply rather than compute it. | `P0` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T06` |
| `SB-SHR-006` | The Leverett J constant and the hydrostatic gradient per unit specific gravity MUST each be **derived from first principles in the product**, defined once, and carry a machine-readable source. Neither may be a transcription of a vendor's printed rounding. The derivation MUST be expressed as an evaluable expression, not a literal. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T07` |
| `SB-SHR-007` | A physical constant used by more than one module MUST have exactly one definition site. `RQI_C` and `PERM_C` MUST be imported by every consumer. The product MUST NOT contain a second literal of a constant it already defines — in code, in user-facing text, or in a test fixture. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T08` |
| `SB-SHR-008` | Interfacial tension and contact angle MUST be carried as **separate** quantities. The product MUST NOT store or ship a fused `σ·cosθ` product as a single constant, and every laboratory and reservoir system MUST declare both components. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T09` |
| `SB-SHR-009` | Every fitted object this domain produces — saturation-height law (pooled and per rock type), Thomeer fit, HFU cluster model, Lorenz flow-unit partition — MUST be persisted as a first-class, named, versioned object. Each MUST carry its training provenance: the wells, the log set and curve versions, the sample count, the full exclusion ledger, the fluid properties and FWL in force, the fitting method, and the fit-quality statistic. An object that cannot state what it was fitted on MUST NOT be applicable. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T10`, `SB-SHR-T11` |
| `SB-SHR-010` | The forward-apply path MUST consume a **stored fitted object**, not hand-entered coefficients. Where a user overrides a stored coefficient, the applied result MUST record the override and the value it replaced. Hand transcription of a fit into a module parameter MUST NOT be the supported workflow. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T12` |
| `SB-SHR-011` | The free-water level MUST be a **first-class uncertain parameter, not a scalar input.** The FWL scan MUST be mandatory output, MUST report an uncertainty interval alongside its optimum — the range of candidate levels whose residual is statistically indistinguishable from the minimum — and MUST NOT present the argmin alone. Every saturation-height result MUST carry a **per-zone FWL confidence** alongside its fit statistic, and a fit whose FWL cannot be constrained MUST say so rather than report a coefficient set. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T13` |
| `SB-SHR-012` | A Brooks-Corey fit MUST declare its exponent convention explicitly and MUST emit **both** `λ` and `N = 1/λ`, each labelled. Import or export of a Brooks-Corey coefficient without a declared convention MUST be refused. | `P0` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T14` |
| `SB-SHR-013` | A Thomeer fit MUST declare the logarithm base of its shape factor `G` and MUST emit both the base-10 (`G`) and natural-log (`2.302585·G`) forms, each labelled. A `G` imported without a declared base MUST be refused. | `P0` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T15` |
| `SB-SHR-014` | The product MUST NOT ship a default apex basis for Swanson permeability. The bulk-volume basis (fraction, percent, or pore-volume-normalised saturation) MUST be an explicit, named user choice with no default, the chosen basis MUST travel with every Swanson result, and the coefficient pair MUST carry its own source. Until a basis is chosen, Swanson permeability MUST be `MISSING`, not computed. | `P0` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T16`, `SB-SHR-T17` |
| `SB-SHR-015` | Pore-throat port-size classification MUST be a named, selectable scheme. Both published boundary sets MUST be available, the active scheme MUST be recorded on every classified result, and there MUST be no silent default. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T18` |
| `SB-SHR-016` | The mnemonic `RQI` MUST be namespace-disambiguated between Amaefule's Reservoir Quality Index and the identically-named quantity in the shaly-sand saturation family. The product MUST refuse to consume one where the other is expected, and MUST NOT resolve the collision by curve-name matching alone. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T19` |
| `SB-SHR-017` | Every correlation in this domain that regresses on `log φ` MUST enforce its porosity **unit** as a precondition, not document it. A porosity input outside the declared unit's valid range MUST fail the run and name the curve; it MUST NOT be silently skipped. | `P0` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T20` |
| `SB-SHR-018` | Where the incumbent corpus ships mutually inconsistent defaults for a quantity this domain consumes, the product MUST surface the divergence **at the point of choice**, quantified in the units the user is working in. The hydrocarbon gradient MUST show the height consequence of each candidate rather than the gradient value alone. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T21` |
| `SB-SHR-019` | A coefficient whose value is scoped to a particular interfacial tension by the implementation that produced it MUST be recorded `NON-ADOPTABLE` and MUST NOT be used as a SandiBumi default in any form, including as a seed. Where such a coefficient is displayed for verification, its σ scope MUST be displayed with it. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T22` |
| `SB-SHR-020` | The Lucia rock-fabric-number validity band MUST be enforced on **both** sides. An `RFN` below the calibrated floor MUST NOT be assigned the lowest class. The `RFN` curve itself MUST be flagged or null outside the calibrated range, so that the value curve and the class curve cannot tell different stories about the same sample. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T23` |
| `SB-SHR-021` | Samples falling in a published correlation's documented extrapolation regime MUST be flagged **per sample**, on the output, not only in the module description. The product MUST NOT clamp such values into a monotone ordering the source paper does not publish. | `P1` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T24` |
| `SB-SHR-022` | Every module in this domain MUST carry the exclusion ledger the fitting path already implements: a named reason and a count for every sample not computed, returned with the result and persisted with any curve it wrote. A curve with materially reduced coverage MUST state why. | `P0` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T25` |
| `SB-SHR-023` | A classifier MUST distinguish "did not meet any class criterion" from "meets the lowest class". Its class boundaries MUST each carry a source or be recorded `ABSENT` and require user entry. | `P1` | `PRESENT-DIVERGENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T26` |
| `SB-SHR-024` | A constant the product's own source records as unverified MUST surface that state as a **result-level flag** carried to every consumer — dialog, plot, export and report — not as a code comment. A result computed from an unverified constant MUST NOT be presentable as clean. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T17` |
| `SB-SHR-025` | The deliverable report MUST name the saturation-height model and the rock-typing scheme actually used, with their parameters and their free-water level, and MUST NOT present a fixed methodology row naming a method the study did not use. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T27` |
| `SB-SHR-026` | Laboratory capillary pressure MUST be convertible to reservoir conditions by the published relation `Pc_res = Pc_lab · (σ·\|cosθ\|)_res / (σ·\|cosθ\|)_lab`, with both systems named and both components of each product declared separately. Conversion MUST be refused when either system is undeclared. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T28` |
| `SB-SHR-027` | Closure / conformance correction MUST be available with all four published treatments — shift, proportional normalisation, crop, extrapolate — as a named user choice with no silent default, and the chosen treatment MUST travel with the corrected curve and with any entry pressure or Thomeer parameter derived from it. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T29` |
| `SB-SHR-028` | Net-overburden-stress correction and clay-bound-water correction MUST be available, MUST be applied to the **non-wetting** phase, and MUST record which correction was applied. A correction applied to the wetting phase inverts its sign and MUST be rejected by construction, not by documentation. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T30` |
| `SB-SHR-029` | Pore-throat-size distributions MUST be tested for **modality**, and a multimodal result MUST be reported as an explicit finding — it is the direct evidence that one saturation-height law is insufficient for the sample set. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T31` |
| `SB-SHR-030` | Automatic flow-unit partitioning MUST be available by **inflection of the Lorenz curve** — partitioning on the change in the flow-capacity/storage-capacity gradient rather than on a statistical criterion — as a selectable method alongside the existing exact Ward segmentation. The active method MUST be recorded on the partition. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T32` |
| `SB-SHR-031` | The Dykstra-Parsons coefficient `VDP = (k₅₀ − k₁₅.₉)/k₅₀` MUST be available alongside the Lorenz coefficient, and the two MUST be reported together — they disagree in informative ways on layered systems. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T33` |
| `SB-SHR-032` | Permeability from capillary pressure MUST offer routes other than Swanson — at minimum Purcell and Katz-Thompson — each named, each with its own source, and their answers MUST be presentable side by side rather than one being chosen silently. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T34` |
| `SB-SHR-033` | Aguilera R35 and the Permadi-Susilo pore-geometry-and-structure indicator MUST be available as **separately named** indicators, never merged into or substituted for Winland R35. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T35` |
| `SB-SHR-034` | This domain MUST expose the fluid-gradient conversion as a service that `14_cutoffs-summation-mc.md` calls to turn an `HCPV` thickness into a volume. The service MUST use the single derived gradient of `SB-SHR-006`, MUST take the fluid densities from the stored fitted object where one is in force (`SB-SHR-009`), and MUST refuse when the depth unit is undeclared (`SB-SHR-003`). | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T36` |
| `SB-SHR-035` | This domain MUST derive candidate reservoir and pay cut-offs **from** its own evidence — the rock type partition, the flow-unit boundaries, and the capillary-pressure curve — and MUST hand them to the cut-off machinery as sourced values carrying the evidence they were derived from. It MUST NOT select a cut-off; selection is `14_cutoffs-summation-mc.md`'s, and the two named-paper closures for selection are not on this machine (see §7.5). | `P1` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T37` |
| `SB-SHR-036` | The Lambda saturation-height family (`Sw = a·Pc^(−λ) + b`) MUST be available as a sixth family once its parameter sources are established. Until then it is recorded `ABSENT` rather than approximated from an adjacent family. | `P2` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T38` |
| `SB-SHR-037` | Any saturation quoted at a capillary pressure — `Swirr` above all — MUST carry its **reference condition as part of the parameter type**, never as free text: the pressure *and* whether that pressure is laboratory or reservoir. A `Swirr` at an undeclared reference MUST be refused. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T39` |
| `SB-SHR-038` | The SCAL importer MUST refuse a corrected capillary-pressure curve that carries no correction-provenance tag naming the correction applied and the implementation that applied it. Module identity on import MUST be keyed on the manifest's declared name, never on a free-text specification line. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T40` |
| `SB-SHR-039` | The product MUST provide a **deterministic model-selection sweep** across this domain's competing choices — saturation-height family, rock-typing scheme, apex saturation, partitioner, port-size scheme — evaluated against core or capillary-pressure control data. The sweep MUST be **exhaustive over the declared grid rather than randomly sampled**, MUST be **uncapped in depth samples**, MUST be reproducible from its recorded inputs, and MUST record the full ranking rather than only the winner. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T41` |
| `SB-SHR-040` | **No numerical result in this domain may depend on a display setting.** A closure pick, an entry pressure, a fitted coefficient or a rock class MUST be identical whether an axis is drawn linear or logarithmic, whether a plot is open, and whatever the current zoom or theme. Where a method genuinely requires a log-domain pick, the log domain MUST be a property of the **method**, not of the plot. | `P0` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T42` |
| `SB-SHR-041` | A published regression MUST NOT be algebraically inverted to predict in the direction it was not fitted. Where a source publishes both a forward and an inverse fit, the one matching the prediction direction MUST be used, and the direction MUST be recorded on the result. | `P1` | `PARTIAL` | `15_sat-height-rocktyping.md` | `SB-SHR-T43` |
| `SB-SHR-042` | Contact angle MUST be stored with a single declared convention and a single validation range across the whole product, and every capillary expression MUST take `\|cos θ\|` consistently. A stored angle and its cosine MUST NOT be independently editable. | `P1` | `ABSENT` | `15_sat-height-rocktyping.md` | `SB-SHR-T44` |
| `SB-TBD-001` | Ship the LRLC recognition screen as a decision, not a chapter of prose | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T01`, `SB-TBD-T02` |
| `SB-TBD-002` | Route by bed thickness against tool resolution | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T02` |
| `SB-TBD-003` | The cause→method route table is data, and it discloses open defects on a route | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T03` |
| `SB-TBD-004` | Declare and enforce the two-component limit | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T04` |
| `SB-TBD-005` | Declare the excluded lithologies | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T04` |
| `SB-TBD-006` | One name, one equation: the picker and the module MUST implement the same construction | `P0` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T05`, `SB-TBD-T06` |
| `SB-TBD-007` | Never clamp a derived volume fraction or a derived sand porosity | `P0` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T07`, `SB-TBD-T08` |
| `SB-TBD-008` | Constrain in the total-porosity direction, and record the shift | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T07`, `SB-TBD-T09` |
| `SB-TBD-009` | Retire `PHIE_LAM`; emit `PHIT_SS` and `PHIE_SS` as distinct curves | `P0` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T10`, `SB-TBD-T11` |
| `SB-TBD-010` | Back-solve the below-left diagnostic instead of constraining it | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T12` |
| `SB-TBD-011` | Assert the `PHIE_SS ≡ PHIE/(1 − VSH_LAM)` identity as a property test | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T11` |
| `SB-TBD-012` | Every interactive endpoint pick carries its provenance | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T13` |
| `SB-TBD-013` | One admissible range per parameter, in one place | `P1` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T14` |
| `SB-TBD-014` | Ship the laminar-structural branch, analyst-selected | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T15`, `SB-TBD-T16` |
| `SB-TBD-015` | No automatic per-level branch switching | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T15` |
| `SB-TBD-016` | Shale cutoffs carry their action class, and the cosmetic one says so | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T17` |
| `SB-TBD-017` | Keep the two laminar-shale estimates separately named | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T18` |
| `SB-TBD-018` | An imported Thomas-Stieber curve carries its parameterization, not just its name | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T19` |
| `SB-TBD-019` | One flag convention across the suite, counted in the run summary | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T20` |
| `SB-TBD-020` | Accept both spellings of Stieber on import | `P2` | `PARTIAL` | `17_thinbed-laminated.md` | `SB-TBD-T21` |
| `SB-TBD-021` | Ship the complete Thomas-Stieber triangle | `P1` | `PARTIAL` | `17_thinbed-laminated.md` | `SB-TBD-T22` |
| `SB-TBD-022` | Implement the series branch as a resistivity mix | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T25` |
| `SB-TBD-023` | The canonical anisotropic form is the quadratic, and every repair is labelled | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T26`, `SB-TBD-T27` |
| `SB-TBD-024` | Retain and report both roots | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T28` |
| `SB-TBD-025` | Select the root by a quadrant classifier, and record the branch | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T29`, `SB-TBD-T30` |
| `SB-TBD-026` | Ship `RV_SH_flip` as an interoperability advisory | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T29`, `SB-TBD-T31` |
| `SB-TBD-027` | Reject the impossible quadrant on the inputs, before the solve | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T32` |
| `SB-TBD-028` | Hard-flag the `RV_SH ≥ RV` singularity | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T33`, `SB-TBD-T34` |
| `SB-TBD-029` | Enforce the horizontal shale-pick proximity guidance in code | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T35` |
| `SB-TBD-030` | Validate `RV_SH ≥ RH_SH` at parameter-entry time | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T36` |
| `SB-TBD-031` | Flag the tensor sand-resistivity bounds; never clamp to them | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T37` |
| `SB-TBD-032` | The parallel-route saturation bound MUST NOT be applied to the tensor route | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T37` |
| `SB-TBD-033` | Never a hard-coded sign | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T29` |
| `SB-TBD-034` | Ship the anisotropy validity conditions as machine-readable data on the module spec | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T29`, `SB-TBD-T32`, `SB-TBD-T33`, `SB-TBD-T36`, `SB-TBD-T38` |
| `SB-TBD-035` | Detect the parallel-route pole; never clamp through it | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T39`, `SB-TBD-T40` |
| `SB-TBD-036` | A negative sand resistivity is a distinct diagnosis | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T39` |
| `SB-TBD-037` | Never silently reduce an anisotropy ratio | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T41` |
| `SB-TBD-038` | Implement the Moran-Gianzero relation, forward and inverse | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T42`, `SB-TBD-T43` |
| `SB-TBD-039` | Refuse to default the relative dip to zero | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T44` |
| `SB-TBD-040` | Assert the √(Rh·Rv) ceiling | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T42` |
| `SB-TBD-041` | Carry the bedding-normal convention in the parameter name | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T43` |
| `SB-TBD-042` | Ship the multi-well dip-fit route for wells without a triaxial tool | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T45` |
| `SB-TBD-043` | Enforce the 40° dip-span precondition on the multi-well route | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T46` |
| `SB-TBD-044` | Dispatch saturation on the sand fraction, never on bulk | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T47` |
| `SB-TBD-045` | Emit the sand-referenced curve set explicitly | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T47`, `SB-TBD-T48` |
| `SB-TBD-046` | Offer `sw_rtc` and `sw_imts` on the sand fraction | `P2` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T49` |
| `SB-TBD-047` | Block all three Poupon-family equations under the laminated model | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T50` |
| `SB-TBD-048` | The minimum-Sw guard emits its unclipped twin | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T51` |
| `SB-TBD-049` | Ship the Vlam reconciliation classifier | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T52` |
| `SB-TBD-050` | Ship the reconciliation track | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T52` |
| `SB-TBD-051` | Ship laminar net sand and net pay, summed on the sand fraction | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T53` |
| `SB-TBD-052` | The laminar summation mode is labelled, opt-in, and off by default | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T53` |
| `SB-TBD-053` | Sand-fraction and bulk cutoffs are never interchangeable | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T54` |
| `SB-TBD-054` | Ship the Klein / butterfly crossplot with the mixing overlays | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T55`, `SB-TBD-T56` |
| `SB-TBD-055` | The Timur coefficient is unit-typed | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T57` |
| `SB-TBD-056` | `Qv` and `B` are converted as a pair or not at all | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T58` |
| `SB-TBD-057` | Angle and temperature conventions are typed, not assumed | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T43`, `SB-TBD-T59` |
| `SB-TBD-058` | Refuse to map the ambiguous vendor sand-fraction curves | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T60` |
| `SB-TBD-059` | Propagate uncertainty through an interval Monte-Carlo | `P3` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T63` |
| `SB-TBD-060` | Ship the anisotropy track | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T62` |
| `SB-TBD-061` | Ship the clay-mineral correction | `P3` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T64` |
| `SB-TBD-062` | Disambiguate the sand-fraction suffix from the model-name suffix | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T48` |
| `SB-TBD-063` | Renormalize the sand-referenced saturation back to bulk for reporting | `P1` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T66` |
| `SB-TBD-064` | Compute permeability on the sand fraction and convert back explicitly | `P2` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T61` |
| `SB-TBD-065` | Resolution enhancement is derived from published literature, never from a vendor model | `P3` | `ABSENT` | `17_thinbed-laminated.md` | `SB-TBD-T65` |
| `SB-TBD-066` | Withdraw the two uncited endpoint defaults | `P0` | `PRESENT-DIVERGENT` | `17_thinbed-laminated.md` | `SB-TBD-T23`, `SB-TBD-T24` |
| `SB-TOC-001` | Make TOC a unit-tagged wt% quantity | `P0` | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T01`, `SB-TOC-T04` |
| `SB-TOC-002` | Ship no numeric delta-log-R baseline | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T14`, `SB-TOC-T15` |
| `SB-TOC-003` | Compute all three overlay separations | `P1` | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T05`, `SB-TOC-T08`, `SB-TOC-T34` |
| `SB-TOC-004` | Preserve the native Passey wt% and clamp order | `P0` | `PRESENT-OK` | `19_toc-unconventional.md` | `SB-TOC-T03`, `SB-TOC-T09`, `SB-TOC-T10` |
| `SB-TOC-005` | Apply the cited LOM cap with a flag | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T11`, `SB-TOC-T13` |
| `SB-TOC-006` | Treat background TOC as part of the baseline pick | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T09`, `SB-TOC-T14`, `SB-TOC-T15` |
| `SB-TOC-007` | Preserve overlay alternatives and spread | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T24` |
| `SB-TOC-008` | Mask invalid geology and borehole conditions before TOC | `P0` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T22`, `SB-TOC-T23` |
| `SB-TOC-009` | Calibrate each overlay and final TOC against lab data | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T25` |
| `SB-TOC-010` | Carry TOC method and pick provenance | `P0` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T57` |
| `SB-TOC-011` | Retain the cited Schmoker cross-check | `P1` | `PRESENT-OK` | `19_toc-unconventional.md` | `SB-TOC-T16` |
| `SB-TOC-012` | Offer the generalized density-deficit form with harmonic grain density | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T17`, `SB-TOC-T18` |
| `SB-TOC-013` | Implement uranium TOC only with its environmental warning | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T21` |
| `SB-TOC-014` | Make kerogen conversion bidirectional and guarded | `P1` | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T19` |
| `SB-TOC-015` | Keep the 1.10 kerogen endpoint and correct its provenance | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T19`, `SB-TOC-T20` |
| `SB-TOC-016` | Keep pyrite coupling source-gated | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T26` |
| `SB-TOC-017` | Compute S2 from typed TOC and selected kerogen type | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T02`, `SB-TOC-T27` |
| `SB-TOC-018` | Implement the complete RockEval carbon balance | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T04`, `SB-TOC-T28`, `SB-TOC-T29` |
| `SB-TOC-019` | Make measured gas inputs required and sourced | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T30`, `SB-TOC-T32` |
| `SB-TOC-020` | Couple Langmuir capacity to the matching organic input | `P0` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T06`, `SB-TOC-T07`, `SB-TOC-T31` |
| `SB-TOC-021` | Keep Langmuir temperature correction opt-in and provenance-bound | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T33` |
| `SB-TOC-022` | Include oil and non-combustible gas in free-gas content | `P1` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T35`, `SB-TOC-T36` |
| `SB-TOC-023` | Name the Bg standard condition | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T34` |
| `SB-TOC-024` | Parameterize and flag the Ambrose correction | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T37`, `SB-TOC-T39` |
| `SB-TOC-025` | Reserve gas-content and gas-in-place names by quantity | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T40` |
| `SB-TOC-026` | Add an internally consistent areal GIP layer | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T41` |
| `SB-TOC-027` | Preserve guarded critical-desorption and in-situ derate behavior | `P1` | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T44`, `SB-TOC-T45` |
| `SB-TOC-028` | Make the isotherm-fit estimator explicit | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T42` |
| `SB-TOC-029` | Force zero intercept in TOC-to-Langmuir calibration | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T43` |
| `SB-TOC-030` | Keep dynamic Rickman brittleness on a declared [0,1] scale | `P0` | `PRESENT-OK` | `19_toc-unconventional.md` | `SB-TOC-T46` |
| `SB-TOC-031` | Refuse static moduli with dynamic endpoints | `P0` | `PARTIAL` | `19_toc-unconventional.md` | `SB-TOC-T47` |
| `SB-TOC-032` | Preserve mineralogical brittleness method identity | `P1` | `PRESENT-OK` | `19_toc-unconventional.md` | `SB-TOC-T48` |
| `SB-TOC-033` | Resolve C1-C5 channel identity before ratios | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T49` |
| `SB-TOC-034` | Require an explicit GWR denominator mode | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T50` |
| `SB-TOC-035` | Make classification rules total and disjoint | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T51`, `SB-TOC-T52` |
| `SB-TOC-036` | Use only cross-tool-corroborated Haworth thresholds | `P1` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T51` |
| `SB-TOC-037` | Gate compiled mud-gas extensions on primary equations | `P3` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T53` |
| `SB-TOC-038` | Keep mud-gas normalization absent until its constant is sourced | `P3` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T54` |
| `SB-TOC-039` | Add component-sum versus total-gas QC | `P2` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T55` |
| `SB-TOC-040` | Persist visual picks as computation parameters | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T15`, `SB-TOC-T56` |
| `SB-TOC-041` | Keep visualization and compute equations identical | `P0` | `PRESENT-DIVERGENT` | `19_toc-unconventional.md` | `SB-TOC-T06`, `SB-TOC-T56` |
| `SB-TOC-042` | Emit stable unconventional QC flags | `P0` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T57` |
| `SB-TOC-043` | Migrate existing projects without changing meanings silently | `P0` | `ABSENT` | `19_toc-unconventional.md` | `SB-TOC-T58` |
<!-- ALL_REQUIREMENT_ROWS -->

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
