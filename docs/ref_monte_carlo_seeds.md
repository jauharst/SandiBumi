# Reference — Monte Carlo per-parameter uncertainty seeds (Tier-A import)

Provenance note for the default distribution widths the Monte Carlo dialog puts on a newly added
uncertain parameter (`src/ui/monteCarloDialog.ts`, `IP_MC_SEEDS` / `distDefaults`). This is a
**Tier-A import** under the dev playbook's IP-cleanliness contract — no new math, no derived
physics constants: what is imported is a table of *how wide a prior each model parameter deserves,
and in which units*.

---

## Source

**Interactive Petrophysics 2025.3**, `C:\Program Files\IP2025\MonteCarloDefaults.par` — the
shipped defaults file for IP's Monte Carlo module (local install; see the IP tree reference note).
The file is a fixed-column ASCII table grouped by IP module (`ClayVol`, `PhiSw`, `MinSolve`,
`Cutoff`, `BLA`, `NMR`, `SigmaSw`, and the geomechanics sets), one row per parameter:

    $Number  Parameter            Shift  Distribution  Low Value  High Value
    $        Name                 Type   Type          Shift      Shift
      65     m exponent           Linear Gaussian      0.2        0.2
       1     Rw                   %      Gaussian      20.0       20.0

Its own header defines the shift types:

- **Linear** — the parameter is shifted by an absolute amount, *in the parameter's own curve units*.
- **%** — the parameter is shifted by a percentage of its value.
- **Rec** — the *reciprocal* is shifted linearly (used for resistivity **curves**, not parameters).

Every imported row is `Gaussian` with a symmetric low/high shift.

## What SandiBumi imports, and what it does not

**Imported:** the per-parameter width and its shift type, for the subset of IP parameters that map
1:1 onto a SandiBumi module argument. That subset is the VSH / porosity / saturation core — the
parameters an analyst actually puts a distribution on in a volumetric uncertainty run.

**Not imported:** the geomechanics, pore-pressure, NMR, `Cutoff` and `SatHeight` blocks (no
corresponding chainable SandiBumi module argument), IP's parameter *numbers* (an IP-internal
index), and the `Rec` shift type (it applies to input curves, which SandiBumi does not perturb).

**Not claimed:** bit-for-bit agreement with an IP Monte Carlo run. See the reading below.

## Reading adopted for the shift value

The `.par` header states the shift's units but **not** which percentile of the Gaussian the
tabulated shift corresponds to. SandiBumi therefore adopts an explicit, documented reading:

| Distribution kind | Interpretation of the tabulated shift `w` |
|---|---|
| Normal | **one standard deviation** — σ = `w`, so ≈68% of draws land within ±`w` of the value |
| Uniform | **half-range** — lo = v − `w`, hi = v + `w` |
| Triangular | **half-range** — lo = v − `w`, mode = v, hi = v + `w` |

For a `%` row the width is resolved against the parameter's own central value: `w = |v| · pct/100`.

This is a deliberately conservative reading. The Tier-A content being imported is the *relative*
judgement — that `m` deserves ±0.2 absolute while `Rw` deserves ±20% relative while a GR endpoint
deserves ±10 API — not a claim about reproducing IP's percentiles. Every width remains editable
per row, and the row's inline PDF sparkline shows the resulting shape immediately.

## Mapping table

IP parameter → SandiBumi module argument. `Linear` widths are in the argument's own unit.

| SandiBumi arg | Unit | IP row (block) | Shift | Width |
|---|---|---|---|---|
| `A` | — | `a factor` (PhiSw / MinSolve / BLA) | Linear | 0.1 |
| `M` | — | `m exponent` (PhiSw / MinSolve / BLA) | Linear | 0.2 |
| `N` | — | `n exponent` (PhiSw / MinSolve / BLA) | Linear | 0.2 |
| `RW` | ohmm | `Rw` (PhiSw / MinSolve / BLA) | % | 20 |
| `RT_SH` | ohmm | `Res Clay` (PhiSw #15 / MinSolve #9 / BLA #19) | % | 20 |
| `GR_MA` | gapi | `Gr Clean` (ClayVol #2 / BLA #1) | Linear | 10 |
| `GR_SH` | gapi | `Gr Clay` (ClayVol #3 / BLA #2) | Linear | 10 |
| `RHO_MA` | g/cc | `Rho Matrix` (BLA #8) | Linear | 0.03 |
| `RHO_FL` | g/cc | `Rho Fluid` (BLA #9) | Linear | 0.02 |
| `RHO_SH` | g/cc | `Rho Clay` (BLA #10) | Linear | 0.05 |
| `RHO_DSH` | g/cc | `Rho Dry Clay` (PhiSw #12) | Linear | 0.1 |
| `NPHI_SH` | v/v | `Neu Clay` (ClayVol #7 / BLA #18) | Linear | 0.05 |

Deliberately **left to the generic fallback** (no unambiguous IP analog): `SWT_IRR`, `SWE_IRR`
(saturation limits, not normally varied), `C` (Simandoux VSH exponent), `RHO_W`, `SG_GAS`,
`PHIE_MAX`, and every parameter of a module IP does not have.

## Fallback for unseeded parameters

Unchanged from before this import — a generic width off the parameter's own value, floored so a
zero-valued default still spreads:

- Normal: σ = max(|v|·0.10, 0.01)
- Uniform / Triangular: half-range = max(|v|·0.20, 0.02)

A seeded width that resolves to zero (a `%` seed on a zero-valued parameter) also falls back, so
no row can be seeded into a degenerate point mass.

## Why this matters

The generic fallback scales every width by the parameter's magnitude, which is wrong whenever the
parameter is not naturally relative. `M = 2.0` was getting σ = 0.2 by coincidence, but `GR_SH = 120`
was getting σ = 12 API where the field convention is ±10, `RHO_MA = 2.645` was getting σ = 0.26 g/cc
— an absurd ±10% matrix density, ~9× too wide against the ±0.03 convention — and `A = 1.0` was
getting σ = 0.1 only because its value happens to be 1. Seeding fixes the units and the magnitudes
at once; the tornado and P10/P90 spread it feeds are only as meaningful as these priors.

## UI

A muted `IP` badge on a parameter row means its width came from this table (hover for the source).
No badge = the generic fallback. Widths stay fully editable either way.
