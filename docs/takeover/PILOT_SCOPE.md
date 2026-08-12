# Proposed first-pilot capability manifest

This is a Gate 1 proposal, not an approval and not a release claim. The exact machine-readable
requirement set is `docs/takeover/pilot-scope.json`. Every requirement not listed there is proposed
`DEFERRED`, not deleted and not implied unimportant.

## Pilot promise

One paid, offline, device-wide Windows vertical slice:

1. Create or open a single-user project and preserve one-writer integrity.
2. Import LAS 2.x and delimited text without changing native log sets, native sample grids, declared
   units, nulls, depth identity or well identity.
3. Make any Reframe/resampling operation explicit, named, reviewable and off by default.
4. Perform source-bound QC and parameterized conditioning: bad-hole/mask handling, despike,
   smoothing, gap and clip behavior, with every changed sample recoverable and provenance recorded.
5. Compute linear-GR shale volume from explicit user/cited endpoints; no endpoint default.
6. Compute density porosity and one chart-free analytic neutron-density method. Arithmetic and RMS
   remain visibly labelled quick-look comparisons and are excluded from pay by default.
7. Compute effective/total Archie and parameterized Indonesia saturation from explicit sourced
   inputs; no automatic `Rw`, `a`, `m`, `n`, `Rsh` or shale endpoint default.
8. Apply fixed, sourced, unit-bearing per-zone VSH, PHIE and SWE/SWT cutoffs; produce deterministic
   thickness, NTG and pay summaries.
9. Review basic log, histogram and crossplot surfaces; export truthful LAS/delimited data and
   paper-scale PDF outputs without vendor chart payloads.
10. Save, reopen, back up and recover the project; install and support it on the declared
    Windows/offline matrix. Native pilot work remains usable without Python.

## Explicit first-pilot exclusions

- Geomechanics/PPFG and Linux support remain next-version/later decisions.
- SandiMin/mineral solving, saturation height and rock typing, thin-bed models, NMR, TOC and
  unconventional methods, rock physics/core imaging, machine learning/electrofacies, production
  logging, cement/casing interpretation, and other cased-hole capabilities.
- DLIS/RP66, legacy XLS/plate workbooks, Office-document imports/exports, LAS 1.2, and LAS 3
  associated-section ingestion.
- Environmental/borehole chart corrections, formation-temperature/resistivity correction,
  population culling, automatic normalization, and vendor QC-band defaults.
- Stieber/Larionov/Clavier/Curved, SP, neutron, resistivity and double-indicator clay methods. The
  pilot carries linear GR only.
- Sonic porosity including Wyllie/RHG80, hydrocarbon-response correction, excavation,
  neutron-sonic, chart-derived porosity, SSC/SSPW, Gaymard-Poupon hydrocarbon correction, and the
  coupled porosity-Sxo/Sw iterative path. Their product-inclusion decisions remain intact; they are
  deferred from this first vertical slice.
- Simandoux presets/aliases, Juhasz, Waxman-Smits, dual-water, SSM, LRLC calibration,
  apparent-Rw inversion and automatic Rw correlations. The pilot carries Archie and Indonesia only.
- Monte Carlo, parameter sweeps/optimization, generalized/geometric/harmonic averages, bed
  amalgamation, arbitrary flag tiers, expression cutoffs and probabilistic cutoff perturbation.
- Pickett/Hingle/regression scientific fits, vendor chart overlays, ternary/faceted plots,
  persisted linked-brush state, plot-derived parameter writes, and portable vendor templates.
- Customer-facing well-count, portfolio-scale, full-lineage, “fully offline,” “full library,”
  incumbent-parity or superlative claims.
- Python-backed analytic capabilities. The qualified offline runtime/preflight remains delivery
  infrastructure, not permission to expose ML, Office or imaging paths in this pilot.

## Exact requirement program

| Group | Requirements |
|---|---:|
| `CORE_TRUTH` | 17 |
| `DATA_IO` | 49 |
| `PROJECT_STORE` | 30 |
| `PLOTTING_REPORTING` | 18 |
| `WINDOWS_OFFLINE_INSTALL` | 22 |
| `QC_CONDITIONING` | 31 |
| `CLAY_LINEAR_GR` | 11 |
| `POROSITY_DENSITY_ANALYTIC_DN` | 26 |
| `SATURATION_ARCHIE_INDONESIA` | 15 |
| `DETERMINISTIC_CUTOFFS_PAY` | 23 |
| **Total** | **242** |

## Evidence and work reality

The proposed 242-row program currently contains:

- 40 `ABSENT`, 74 `PARTIAL`, 61 `PRESENT-DIVERGENT`, 14
  `PRESENT-UNVERIFIED`, and 53 `PRESENT-OK` rows;
- 95 silent-wrongness, 103 data-integrity, 17 deployment, 16 degraded-result, 5 recovery,
  4 field-evidence and 2 requested-capability risks; and
- 174 rows with no qualifying whole-contract proof, 19 characterization-tested rows and
  49 correctness-tested rows.

Those counts are not a completion percentage. A PRESENT-OK row may still need field evidence, while
an ABSENT or PRESENT-DIVERGENT row generally needs bounded production work or an explicit refusal.

## Harsh review

- **Founder risk:** competing with Geolog by breadth before proving one trustworthy workflow would
  recreate the current 584-blocker problem at release time. A pilot proves trust, not catalogue
  parity.
- **Engineering risk:** 242 requirement blockers are still a large solo-developer program. Calling
  this “small” would be dishonest; it is only bounded and serializable. If speed is more important,
  the next cut must remove a real capability—most plausibly analytic D-N, Indonesia, or the
  conditioning chain—not pretend their contracts do not exist.
- **Agent risk:** counting adjudicated rows or passing tests as delivered capability would repeat the
  evidence inflation already found in the exact-test audit. Gate 2 must close silent wrongness at the
  reporting surface, and Gate 4 remains Jauhar-owned real-data confirmation.

## Approval boundary

Approval means all 242 listed requirements become the exact first-pilot blocker program and all
other 689 requirements are explicitly deferred. It does **not** approve a parameter, endpoint,
cutoff, unit convention, scientific equation or manual checkbox. Those remain source- and
evidence-bound.

The exact 242-ID set has SHA-256
`0412de0cc43fabbe0c5e32d4c831d65e90536ee1c348802ab67cb0f3dcd70b6b`. Any ID or hash drift
invalidates approval and requires a new owner decision.

Separately, Gate 1 needs approval to amend its all-931 live-as-built criterion narrowly: the exact
52 hashed `SB-GEO` rows may remain visibly `UNADJUDICATED` because DEC-011 defers that domain.
They remain accounted for, release-dispositioned, and mandatory for next-version live adjudication.
