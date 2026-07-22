# Reference — rock typing: flow units, Lorenz plots, pore-throat radii

Method notes + primary-source citations for the rock-typing engine (`rocktyping.rs`, `hfu.rs`,
`lorenz.rs`). Constants transcribed from the cited primaries are flagged **verify-before-release**
where a secondary source was the transcription anchor. This file is the Tier-B math bank for these
modules (per the dev playbook's IP-cleanliness contract): the code carries a one-line citation, the
derivation lives here.

---

## Stratigraphic Modified Lorenz Plot (SMLP) — `lorenz.rs` (playbook #3, increment 3a)

**Primary:** Gunter, G.W., Finneran, J.M., Hartmann, D.J., Miller, J.D. (1997). *Early Determination
of Reservoir Flow Units Using an Integrated Petrophysical Method.* SPE Annual Technical Conference,
SPE-38679-MS. Heterogeneity index after **Schmalz, J.P. & Rahme, H.S. (1950)**, *The variation of
waterflood performance with variation in permeability profile* (Prod. Monthly 15(9):9–12); reviewed
in **Lake, L.W. & Jensen, J.L. (1991)**, *A review of heterogeneity measures used in reservoir
characterization*, SPE-20156, In Situ 15(4).

### Definitions

For samples in **depth (stratigraphic) order**, each carrying porosity φᵢ (v/v), permeability kᵢ (mD),
and bed thickness hᵢ:

- **Storage capacity** contribution: φᵢ·hᵢ  → cumulative, normalized:  x_i = (Σ_{j≤i} φⱼhⱼ) / Σφh
- **Flow capacity** contribution:    kᵢ·hᵢ  → cumulative, normalized:  y_i = (Σ_{j≤i} kⱼhⱼ) / Σkh

Plotting y (flow) vs x (storage) is the **Stratigraphic Modified Lorenz Plot**. Because samples are
NOT reordered, the curve is monotone but kinked. Its **local slope** at sample i is

    dy/dx |_i = (kᵢhᵢ / Σkh) / (φᵢhᵢ / Σφh) = (kᵢ/φᵢ) · (Σφh / Σkh)

— the thickness hᵢ **cancels**, so the tangent depends only on kᵢ/φᵢ and a global constant. The
diagonal (slope = 1) is the well-average k/φ. Interpretation (Gunter 1997):

- slope **> 1** — the interval delivers more flow than its share of storage → a **speed zone**
  (reservoir conduit / flow unit).
- slope **< 1** — a **baffle** (contributes storage but little flow).
- slope **≈ 0** — a **seal / barrier**.

### Flow-unit segmentation

A **flow unit** is a maximal contiguous depth interval of similar SMLP slope. Since the tangent is
∝ kᵢ/φᵢ, we segment the depth-ordered profile of **mᵢ = log10(kᵢ/φᵢ)** (log domain — k/φ spans
decades and is log-normal within a unit, the same rationale as FZI clustering in `hfu.rs`).

- **Exact contiguous partition:** an O(K·m²) dynamic program (`segment_dp` + `backtrack`) finds the
  K-segmentation minimizing total within-segment sum of squares of m — the Ward criterion, but on the
  **natural depth order** (not the value-sorted order `hfu.rs::ward_partition` uses), so segments are
  true depth intervals.
- **Auto-K:** keep adding a boundary while the next split removes ≥ `AUTO_K_TOL` (= 2 %) of the
  single-segment SSE, capped at `AUTO_K_MAX` (= 12). A caller-set `n_units ≥ 1` forces exactly K.

### Lorenz (heterogeneity) coefficient

From the **reordered** Lorenz plot — samples sorted by **descending k/φ** — accumulate the same
normalized (Σφh, Σkh). The curve bulges above the 45° line; with A = trapezoidal area under it,

    Lc = 2·(A − ½),   clamped to [0, 1]

Lc = 0 → homogeneous (curve on the diagonal); Lc → 1 → highly heterogeneous (flow concentrated in a
thin interval). This is the classic Dykstra-Parsons-family single-number heterogeneity index.

### Thickness convention

`local_thickness` uses the midpoint rule on the **full** depth grid (interior sample → half the gap
to each neighbour; edges → the one-sided gap), computed **before** screening invalid samples, so a
valid sample flanked by missing φ/k carries only its own grid step — a missing interval contributes
nothing rather than smearing its thickness onto neighbours. On a uniform grid this reduces to the
constant sample step. Samples with φ∉(0,1), k≤0, or non-finite are skipped.

---

## Related methods (banked elsewhere / in-code)

- **Amaefule 1993 RQI/φz/FZI + Corbett-Potter 2004 GHE bins**, **Kolodzie 1980 Winland R35**,
  **Permadi-Susilo PGS** — `rocktyping.rs` header + `docs/research_2026-07/ref_rocktyping_shf.md`.
  GHE bins + PGS exponents were re-verified 2026-07-22 (`docs/constants_verification_2026-07-22.md`).
- **Amaefule FZI clustering (Ward / histogram antimode)** — `hfu.rs` header.
- **Lucia 1995 / Jennings & Lucia 2003 rock-fabric number** (carbonate) — `rocktyping.rs::lucia_rfn`.
- **Pittman 1992 pore-throat radii r10–r75** (AAPG Bull. v76) — `rocktyping.rs::pittman_rx`; the r35
  row cross-checks ref_rocktyping_shf.md; the full nine-row table is **verify-before-release**.
