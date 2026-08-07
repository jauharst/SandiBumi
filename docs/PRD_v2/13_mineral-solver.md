# 13. Multi-mineral solver — requirements

**Dossier:** `docs/research_2026-08/cross_tool/mineral-solver.md` (2,205 lines), including its
discrepancy-ledger disposition (§4.1), its adoption spec (§5) and its `## Critique disposition`
(§8), which is authoritative over any body text it corrects.
**Evidence tiers held:** T1 (SandiBumi's own `multimin2.rs` / `multimin.rs` read line-by-line;
Techlog's shipped `QM_MineralTable.xml` and `PythonScripts\QElan_PostProcess_Using_Conductivities.py`
read directly this session; IP `MINDEF.PAR` / `MINEQDEF.PAR` via `docs/multimin_ip_spec.md`),
T2 (IP 2018/2025 CHM ingest reports), T2-equivalent (Techlog ELANPlus HTML + 16 vision-read
equation rasters), T3 (Geolog Multimin helpset via `docs/multimin_ref_spec.md`), T4 (course notes).
**Geolog is T3-only and cannot be raised on this machine** — Multimin is a compiled module, no
install tree exists locally. A blank Geolog cell is absence of ingest, not absence of capability.
**Author date:** 2026-08-07.
**Requirements:** 46 (`SB-MIN-001` … `SB-MIN-046`). **P0:** 10.
**Acceptance tests:** 44 (`SB-MIN-T01` … `SB-MIN-T44`).
**Parameters:** 78 rows in §5 (plus 11 group-divider rows), of which **10 ship
`ABSENT — ships with no default`**, 5 are `NON-ADOPTABLE — cited for verification`, and 16 are
`VENDOR-DERIVED` pending re-sourcing under `SB-CORE-005`.

> **Front-matter counts corrected 2026-08-07, and the correction is itself the finding.** This block
> originally read 9 P0, 34 tests and 63 parameter rows — the counts as they stood partway through
> drafting. §4's own preamble said "Ten are P0" while this block said nine; the chapter disagreed with
> itself. Recounted mechanically against the finished file: 46 requirement definition headings, 10
> tagged `[P0]`; 44 unique `SB-MIN-Tnn` ids, contiguous `T01`–`T44`; 78 parameter rows once the 11
> group-divider rows are excluded. Logged by the §7 author as OPEN-8 rather than fixed, because that
> pass was scoped append-only and could not touch §1–§6. This is the stale-measurement pattern
> `02_RISKS_AND_CONTRADICTIONS.md` §11 names, caught inside a single document.

> **One correction to the commissioning brief, stated up front because it changes two requirements.**
> The brief carried two findings about the legacy `multimin.rs` as live. **Neither is live.** Both
> were real, both are in the repository's own history, and both were closed before this chapter was
> written: commit `8fc873b` ("R17: the legacy Multimin solver now mixes PEF as volumetric U, not raw
> per-electron Pe") fixed the linear-Pe mixing law *and* the test that forward-modelled with the same
> wrong law, and commit `73f952d` ("R22: gracefully retire the legacy multimin module") removed the
> solver body and its tests entirely. `multimin.rs` is now 67 lines and holds a `ModuleSpec` only.
> The residue that **is** live is narrower and is carried as `SB-MIN-041`. Full disposition in §3.6.

---

## 1. Scope and boundary

This chapter owns the **simultaneous multi-mineral inversion**: the forward response model
`t_k = Σ_i P[k][i]·v_i`, the weighted least-squares objective and its reported statistics, the
constraint system (unity, box bounds, porosity equalisation, bound water, mud-type inequalities),
the endpoint library and its provenance, the bound-water parameterisation, and the solver's
convergence and conditioning diagnostics. It owns SandiBumi's `SandiMin` module
(`src-tauri/src/multimin2.rs`) and the retired legacy `multimin` module.

**Seam to `POR` (Porosity).** SandiMin *emits* porosity rather than consuming it: `MM_PHIE`,
`MM_PHIT` and their X-zone counterparts are solver outputs derived from the fluid volume sets
(`multimin2.rs:1648`, `:1649`). Every requirement here that moves bound water moves `PHIE` by the
same amount under the unity constraint — §2 F-2 quantifies a 1.7 pu case and §2 F-21 a 4.1 pu case.
`POR` owns the deterministic density/neutron/sonic porosity routes and the question of which
porosity a core plug should be matched against; this chapter owns only the solver-derived pair and
the requirement that both are reported (`multimin2.rs:1660`–`:1663`).

**Seam to `SAT` (Water saturation).** SandiMin ships seven saturation models
(`SwModel`, `multimin2.rs:103`–`:123`): a linearised dual-water that runs *inside* the inversion,
and six post-solve closed forms (dual-water non-linear, Archie, Indonesia, Simandoux, Juhász,
Waxman-Smits). The **equations** belong to `SAT`; this chapter owns only (a) the in-inversion
conductivity row and its `^(1/w)` transform, `w = 0.75m + 0.25n`, because that is a response-equation
question, and (b) the post-solve redistribution contract — that `PHIE` and hard unity are preserved
while the water/HC split is replaced (`multimin2.rs:1417`–`:1421`). Where a saturation parameter is
named here (`a`, `m`, `n`, `Rsh`, `B`) it is cited to `SAT` and no number is allocated in that domain.

**Seam to `CLY` (Clay and shale volume).** SandiMin emits `MM_VSH` as `Σ(clay volumes) + Σ(U bound
water)` — a wet-clay volume from the inversion, not a GR index (`multimin2.rs:1681`). The
project-kb precedent records a study taking Vsh from the solver output precisely because a GR index
failed on low-GR shale. `CLY` owns the index methods and their endpoint picking; this chapter owns
the solver-derived Vsh, its wet-vs-dry convention flag, and the per-clay endpoint rows that feed it.

**Seam to `CUT` (Cutoffs, summation, Monte Carlo).** IP's Monte Carlo takes Mineral Solver as a
selectable module and propagates **endpoint** uncertainty, which no other vendor does. The Monte
Carlo *engine*, its distributions and its seeding discipline belong to `CUT`; this chapter owns the
requirement that the engine can shift **mineral endpoints and solver parameters** (`SB-MIN-037`) and
the endpoint-shift default that only IP publishes.

**Seam to `ENV` (Environmental corrections and log QC).** IP's neutron response inside the Mineral
Solver is parameterised by a `.neu` look-up table selected by the **Logging Contractor well-header
field** — a header attribute that silently changes numerical results. The `.neu` table format,
its load-time integrity checks and the environmental-correction chain belong to `ENV`; this chapter
owns the requirement that the table is an explicit *named model input* recorded in the run record
(`SB-MIN-026`).

**Seam to `DBM` (Database and project data model).** Per-value endpoint provenance (`SB-MIN-009`) is
a schema obligation as much as a solver one: a parameter that carries its source through the
computation into the deliverable requires the source string to be a persisted column, not a comment.
`DBM` owns the storage; this chapter owns what must be stored and that a row without it fails to load.

**Seam to `RPH` (Fluid substitution and rock physics).** SandiMin derives `VP` and `VS` endpoints
arithmetically from `DT` (`VP = 304.8/DT`, `VS = VP/1.7` for non-fluids, `multimin2.rs:2116`–`:2117`).
The 1.7 Vp/Vs ratio is a rock-physics parameter with no source string in this domain; it is raised
as an escalation in §7 and belongs to `RPH`.

**Declared overlap.** `SB-MIN-012` (mix U, never Pe) restates a house rule that also binds `POR` and
`ENV`. It is allocated here because the Pe→U conversion is implemented here
(`multimin2.rs:1786`–`:1788`) and is shared by both solvers.

---

## 2. What the incumbents do — the requirement-bearing findings

Twenty-one findings. Each generates at least one obligation in §4. Findings from the dossier that
generate no obligation are accounted for in §8, not restated here.

### F-1 — Non-negativity: mineral deletion (IP) vs bounded optimisation (Elan, Geolog)

**Tier:** T2 (IP `C_mineral_solver.md` §4.3 step 3) · T2-equivalent (Elan `image2982.gif`,
`-internal-constraints.html`) · T3 (Geolog `multimin_ref_spec.md` §D).

IP does not constrain volumes to be non-negative. It solves, and *"if any volume is negative, the
largest negative term is set to zero and **removed from the model**, the solver re-runs, and this
repeats until all volumes are positive"* — an active-set deletion that changes the **dimension of the
system**. Elan uses an always-on internal positivity constraint; Geolog uses hard box bounds
"always honored exactly", defaults `0..1`.

**Consequence, quantified and correctly scoped.** On a 4-component model where Illite goes to
`V = −0.03`, IP deletes the Illite column and Quartz absorbs its response. Using Geolog RF04 6.2 dry
endpoints and IP `MINEQDEF.PAR` confidences: density perturbation `0.03 × (2.78 − 2.65) = 0.0039 g/cc`
against a 0.02 g/cc confidence = **0.195 σ**; neutron `0.03 × (0.247 − (−0.050)) = 0.0089 v/v`
against 0.02 v/v = **0.445 σ**. On a Quartz/Chlorite pair the neutron figure rises to
`0.0171/0.02 = 0.855 σ`. **A 0.2–0.9 σ step is not by itself large, and the finding does not rest on
it.** The portability breaker is that the step is a *discontinuity*: over a shaly interval where a
marginal mineral flickers in and out, IP's volumes are discontinuous in depth while a bounded
solver's are continuous. Two of three vendors, plus published optimisation engines
(Goldfarb & Idnani 1983 dual QP; Powell VF02 SQP), are on the side of bounded optimisation; IP's
mechanism is an uncited heuristic. **A bounded solver will not reproduce IP volume-for-volume even
with identical endpoints, and that is a design choice to document, not a bug to chase.**
→ `SB-MIN-001`, `SB-MIN-002`.

### F-2 — Bound water: a flat zonal φTclay (IP) vs CEC·ρ·T·salinity (Elan, Geolog)

**Tier:** T2 (IP §3.1) · T2-equivalent (Elan Eq 63/64, `image2947.gif`) · T3 (Geolog §B).

IP's `φTclay` is a flat zonal parameter, default **0.15 v/v**, with no temperature and no salinity
term. Elan Eq 64 and Geolog's `k_clay` both compute the same quantity from
`α·V_Q^H·ρ_dcl·CEC_dcl`, with `α = sqrt(0.35 mol/L / n)` below **20,455 ppm NaCl** — the fresh-water
diffuse-layer expansion. The three formulations are algebraically the same object: rearranging Elan
Eq 64 gives `WCLP/(1−WCLP) = α·V_Q^H·ρ_dcl·CEC_dcl`, which is exactly IP's E34 coefficient with the
temperature and salinity frozen.

**Consequence, quantified across the salinity range.** Illite at CEC 0.25 meq/g, ρ_dcl 2.78 g/cc,
T 100 °C (a 2,500–3,000 m reservoir), `k₀ = 0.16764`. At V_dryclay = 0.25 the bound-water volume is
0.0419 bulk v/v at ≥20,455 ppm, **0.0670 at 8,000 ppm** and **0.1094 at 3,000 ppm**. IP running its
0.15 default sits at the saline end and returns 0.0441 — **a 2.3 pu shortfall at 8,000 ppm and a
6.5 pu shortfall at 3,000 ppm.** Under unity that volume comes out of free water: **PHIE is
over-called and SWE under-called by exactly that amount in fresh and brackish sections.** Elan
independently states the room-temperature constant as **0.28 cm³/meq** against Geolog's implied
0.297 — two vendors on the same Clavier-Coates-Dumanoir physics to within 6 %.
→ `SB-MIN-006`, and it is the reason `SB-MIN-007` and `SB-MIN-008` are P0.

### F-3 — The misfit statistic is not portable between products

**Tier:** T2 (IP E1, `_imsclip0122.png`, verified 4×) · T2-equivalent (Elan Eq 79/80,
`image2969.gif`, re-read independently) · T3 (Geolog §H).

Three genuinely different statistics. IP's `Total_err = sqrt(Σ((Crv − Crv_Rec)/Crv_Tol)²)` carries
**no normalisation at all** — no `/NumCrvs`, no `1/N`. Elan's `SDR` divides by the number of tools.
Geolog's `QUALITY` divides by the 95th-percentile χ² at `(ntool − 3)` degrees of freedom.

**Consequence, quantified.** A 6-equation model in which every reconstructed curve misses by exactly
one tolerance scores `sqrt(6) = 2.449` in IP — **red**, because IP colours `TotErr > 1.0` red —
`sqrt(6/6) = 1.000` in Elan, the break-even value, and `sqrt(6/7.815) = 0.876` in Geolog, **good**.
Three tools, the same fit, two of them call it acceptable and one flags it red.

**The dossier's own algebra makes this actionable rather than merely cautionary.** Elan Eq 79 divides
each residual by `LargestWeight` and Eq 80 multiplies the root back by it; the two **cancel
identically**, leaving `SDR = sqrt(Σ_k r_k² / n)` — the plain σ-weighted RMS residual in physical σ
units. So a canonical `RECON = sqrt(Δ²/n_live)` is not Elan-*shaped*, it is **algebraically identical
to Elan Eq 80**, and a SandiMin `RECON` is directly comparable to a Techlog `SDR`.
→ `SB-MIN-013`, `SB-MIN-014`.

### F-4 — Wet/dry clay endpoint convention: the highest-frequency silent-wrongness site

**Tier:** T2 (IP `_imsclip0050.png` wet vs `_imsclip0092.png` dry, same worked example) ·
T1 (Techlog `QM_MineralTable.xml` stores `Bulk Density`, `Rhobdcl` and `Rhobwcl` as three columns) ·
T3 (Geolog stores dry only).

IP's *same* worked example prints Clay density **2.4** in the wet-clay model and **2.78** in the
dry-clay model. IP's observed wet-clay spread across its own examples is 2.4 / 2.414 / 2.429 / 2.446
/ 2.64 / 2.65. Elan solves in wet-clay volumes internally *regardless of the Clay switch* and
**reports in dry-clay volumes plus a separate bound-water curve** — a deliberate input/output
asymmetry, stated verbatim by the vendor.

**Consequence, quantified.** A 30 % wet-clay zone solved with the dry 2.78 where the model convention
wants the wet 2.4 puts `0.30 × (2.78 − 2.4) = 0.114 g/cc` of unexplained density into the RHOB row.
At IP's 0.02 g/cc confidence that is a **5.7 σ** residual — which the solver will not leave alone; it
buys the density back with porosity: `Δφ ≈ 0.114/(2.65 − 1.00) = 0.069 v/v` = **6.9 pu of porosity
error**. At the 2.429 and 2.446 endpoints the stake is 6.4 pu and 6.1 pu. Three related conventions
flip with the same switch: the `ECS_Clay (Wt%)` endpoint (0.85 wet / 1.0 dry), the bound-water
coefficient (`φ/(1−φ)` multi-clay vs `φ` single-clay), and `GrainDensity`, which is **always the dry
density even inside a wet-clay model**. **A bare "clay density" field is a silent-wrongness
generator.** → `SB-MIN-010`, `SB-MIN-024`.

### F-5 — Fluid sonic endpoints diverge by 45 µs/ft on gas

**Tier:** T1 (Techlog `E_mineral_endpoints.json`) · T1 (IP `MINDEF.PAR` via `multimin_ip_spec.md` §A)
· T3 (Geolog `multimin_ref_spec.md` §I).

Oil: IP **200**, Techlog **210**, Geolog **189** µs/ft — spread 21 µs/ft. Gas: IP **220**,
Techlog **265**, Geolog **250** µs/ft — spread **45 µs/ft**. Geolog's oil DT genuinely equals its
water DT at 189 µs/ft, which is the most surprising single value in the table and is correct as
sourced. EPT agrees exactly across IP and Geolog (oil 5.0, gas 3.3 ns/m).

**Consequence, quantified.** In a 20 pu gas sand at Sxo = 0.4 the hydrocarbon volume is 0.12 v/v, so
the DT row contribution differs by `0.12 × (265 − 220) = 5.4 µs/ft` between the Techlog and IP
libraries — **1.8× IP's own 3.0 µs/ft sonic confidence and 2.8× Geolog's 1.951 µs/ft.** A sonic row
therefore dominates the gas-volume solution differently depending on which library seeded it, and
**no evidence settles which is right**: these are three independent tool-model vintages.
→ `SB-MIN-028`, `SB-MIN-029`.

### F-6 — Constraint semantics: soft in IP, hard in the other two

**Tier:** T2 (IP §4.3, §4.5) · T2-equivalent (Elan Eq 85-1a/85-1b, `image2983.gif`) · T3 (Geolog §B).

IP's unity is an ordinary equation with tolerance 0.01 — its own manual concedes *"the unity
equations will not necessarily force the results to absolutely 1.0"* — followed by a **post-hoc
renormalisation of every volume to sum 1.0**. Elan enforces unity as a hard inequality *pair*
(`Σ V ≤ 1.0` and `Σ V ≥ 1.0`), so *"the sum of volumes must be **exactly** equal to 1.0"*. Geolog
makes it a `Tool` row: simultaneously a hard equality and a σ = 0.01 pseudo-measurement.

IP's limit equations are added **one at a time, in grid order**, each triggering a complete re-solve,
and once added are *"treated as a constant equation weighted by its Confidence — so the result can
still fall outside the limit."* Elan and Geolog make the same constraints hard inequalities with no
uncertainty and no degree of freedom. **A solver honouring hard bounds will report infeasible where
IP silently returns an out-of-limit answer with a large `TotErr`. That is an improvement and a
behaviour change users will notice — surface it as a diagnostic, not a crash.**
→ `SB-MIN-003`, `SB-MIN-033`, `SB-MIN-034`, `SB-MIN-035`.

### F-7 — Only `Tool` rows add a degree of freedom

**Tier:** T3 (Geolog `multimin_ref_spec.md` §B, verbatim: *"only `Tool` rows add a degree of
freedom"*).

Geolog defines four volume-constraint row types: `==` (hard equality, no DOF), `Tool` (hard equality
**and** a σ = 0.01 pseudo-measurement that enters the incoherence, **the only type that adds DOF**),
`>=` and `<=`. The distinction is load-bearing because a bound-water tie demoted to a plain equality
silently changes `n_tool`, and `QUALITY`'s `χ²₉₅(n_tool − 3)` denominator with it. Geolog makes
UNITY a `Tool` row too; adopting Elan's HARD unity instead costs one degree of freedom and makes a
SandiMin `QUALITY` read slightly *higher* than a Geolog `QUALITY` on the same fit.
**That difference must be asserted, not discovered.** → `SB-MIN-016`, `SB-MIN-035`.

### F-8 — "Off" is not "weight = 0", stated by the vendor as a design rule

**Tier:** T2-equivalent (Techlog `-default-uncertainties.html`, verbatim).

> *"Dividing the multiplier by four is close to having the tool ignored in the solution. **Truthfully,
> one can never completely turn the tool off with uncertainties. To be totally out of the solution,
> the equation must be removed from the model (Solve process).**"*

A weight of zero and an absent equation are different objects: the row still occupies a degree of
freedom and still perturbs the conditioning of `PᵀUP`. IP's `Use` checkbox, Geolog's `Active Log = No`
and Elan's "remove from the Solve process" are all the *structural* off-switch; the multiplier is only
an attenuator. Elan is also the only vendor with a second multiplier at all — a zonable `xxxx_WM`
where *"a multiplier value of 1.0 means that the tool will influence the answer as strongly as the
Volume Summation tool"*. → `SB-MIN-017`, `SB-MIN-018`.

### F-9 — The conductivity root exponent: IP contradicts itself (proposed ledger item CT-3)

**Tier:** T2, both readings quoted verbatim from **both** editions.

IP's manual states the pre-solver transform of `Crv`, `Crv_Rec` and `Crv_Tol` **two ways**: the
*"**square root** taken of them before applying them in the above equation"* and *"has **1/m th root**
taken of it before using in the solver"*. These describe the same object — `Crv_Tol` *is* the
confidence. The vendor's worked example is printed at `m = 2`, where `1/m = ½` exactly, so it
**cannot arbitrate**. Elan is unambiguous (plain square root, doubly stated). Geolog uses
`1/w` with `w = 0.75m + 0.25n`.

**Consequence, quantified.** At `m = 2.2, n = 1.6`: Elan 0.500, Geolog 0.4878, IP either 0.500 or
0.4545 — **a 9.1 % spread within IP alone, larger than the IP-vs-Geolog gap.** At `m = 2.0, n = 1.6`
the Geolog-vs-IP gap is 5.3 % and survives either reading. **A mis-set conductivity exponent
computes, plots and ships.** → `SB-MIN-021`, escalation `ESC-1`.

### F-10 — The Shell porosity-dependent `m` constant is a three-way conflict (ledger D-10)

**Tier:** T2 (IP 2025 raster 0.018 verified 4×; IP 2025 ASCII 0.019; IP 2018 raster 0.018 verified
6×; IP 2018 ASCII 0.019) · T2-equivalent (Techlog Eq 78 `mc2`, *"the usual value assumed for mc2 is
0.19"*, Table 28 default 0.0).

The same named constant appears as **0.018, 0.019 and 0.19** across two vendors, and Elan never
states whether the `φₑ` inside `mc2/φₑ` is fractional or p.u. The IP 2025 ingest report adds its own
leaning — *"the published Shell formula uses 0.019, so the ASCII agrees with the literature and the
raster does not — but the manual states both and this report does not pick a winner"* — which is a
third-party reading, not a citation to a named Shell paper, and does not close it.

**Consequence, quantified.** At φe = 0.10, m = 2.05 (0.018) vs 2.06 (0.019); at φe = 0.02,
m = 2.77 vs 2.82 — a ~5 % exponent difference propagating to ~5–10 % Sw error in tight rock.
→ `SB-MIN-022`, and the parameter ships `ABSENT` in §5.

### F-11 — IP's neutron response is selected by a well-header field

**Tier:** T2 (`O_db_config_infra.md` §3.3, verbatim).

> *"the **Logging Contractor** field on the Default Parameters tab selects the neutron/density
> crossplot overlays, **sets the Neutron Tool Type for Basic Log Analysis and Mineral Solver**, and
> selects the neutron look-up tables … **A single header dropdown therefore silently changes
> numerical results.**"*

Two IP runs with identical models, identical endpoints and identical curves can return different
volumes. **Any IP-parity fixture is under-specified unless it records the Logging Contractor.** The
platform-side ingest reaches the same conclusion independently: *"If an attribute drives physics, it
must be surfaced in the run record, not buried in a header tab."* → `SB-MIN-026`.

### F-12 — Uncertainty propagation: three vendors, three different questions

**Tier:** T2 (`D_cutoffs_montecarlo.md` §2.7/§3.7/§3.8/§5.8/§5.9) · T2-equivalent (Elan
`-solution-method.html`) · T3 (Geolog §H).

IP is the **only** vendor that propagates *endpoint* uncertainty through the solve, via Monte Carlo
over the Mineral Solver's own parameters — endpoint shifts defaulting to **±10 % of the endpoint
value**, Gaussian, 2000 iterations, with a dependency-correlation matrix. Elan computes **balanced
uncertainties before the volumes are solved**, because *"uncertainties do not include the volume of
the mineral"*. Geolog reports the linearised covariance `sqrt(diag(A⁻¹))·QUALITY` per volume.
**The three are complementary, not competing.**

**But IP's is not reproducible.** *"IP uses a random number generator, **seeded through the CPU clock
time**"*, and no user-settable seed is documented anywhere. Its substitute — rank iterations, pick a
percentile, reload that iteration's saved parameters — is a *replay* mechanism, not reproducibility.
→ `SB-MIN-037`, `SB-MIN-038`, `SB-MIN-045`.

### F-13 — Only Geolog makes ill-conditioning visible before the answer is trusted

**Tier:** T3 (Geolog §H).

Geolog reports `CONDNUM = log₁₀` of the SVD norm ratio of `A = PᵀUP` (**>8 suspect, >10 unstable**,
linear cutoff default 10), plus an explicit degree-of-freedom check and a conflict check. **Neither
IP nor Elan diagnoses an ill-conditioned design matrix at all.** Elan's own manual concedes the
consequence without the diagnostic: *"an average carbonate model, which is underdetermined, will
compute a very geologically questionable 50 % dolomite and 50 % clay in shaly zones"*, and
*"The ELANPlus program believes what you tell it."* **This is the cheapest real improvement available
in the domain.** → `SB-MIN-015`, `SB-MIN-016`.

### F-14 — The three endpoint libraries agree on matrix density and disagree on almost everything else

**Tier:** T3 (`ip_ingest/E_threeway_endpoint_compare.json`, agreement criterion "relative spread
≤ 5 %").

| Property | Agree | Diverge |
|---|---|---|
| RHOB (g/cc) | **12** | 6 |
| NPHI (v/v) | 5 | **13** |
| DT (µs/ft) | 9 | 9 |
| U (b/cc) | 6 | **12** |
| SIGMA (c.u.) | 6 | **12** |
| CEC (meq/g) | 1 | 3 |
| GR (API, Techlog-vs-Geolog only) | 1 | **17** |

RHOB agrees three-way for **all 12 clean non-clay matrix minerals**; Calcite and Halite agree on
every property. GR diverges everywhere partly for a structural reason — IP's `MINDEF.PAR` has **no
GR column at all**, computing it at runtime from K/Th/U weight fractions. **These are
library-provenance differences, not bugs**, and the disagreement is itself information no incumbent
surfaces. → `SB-MIN-028`, `SB-MIN-009`.

### F-15 — Techlog's own clay table is not self-consistent under Techlog's own equation

**Tier:** T1 (`QM_MineralTable.xml`) · T2-equivalent (Elan Eq 11, `image2863.gif`).

Elan Eq 11 states `ρ_dcl = (ρ_wcl − WCLP)/(1 − WCLP)`. Applying it to Techlog's own shipped triples:
Illite `(2.52 − 0.17)/0.83 = 2.8313` against the tabulated `Rhobdcl = 2.70` (**+4.9 %**); Kaolinite
`(2.41 − 0.07)/0.93 = 2.5161` against 2.65 (**−5.1 %**); Chlorite 2.7742 against 2.80 (−0.9 %).
**Anyone who adopts Techlog's `Rhobwcl` + `Phicl` and then derives the dry density via Eq 11 gets a
different number than Techlog's own `Rhobdcl` column.** The shipped table is a curated default
library, not a derived one. No vendor statement explains it. → `SB-MIN-046`, escalation `ESC-6`.

### F-16 — Techlog attaches a resistivity to the clay *mineral*; IP and Geolog do not

**Tier:** T1 (`QM_MineralTable.xml`: Illite `Rsh` **3**, Chlorite **5**, Shale **5**, Kaolinite **7**
ohm-m; `XWater` 0.03).

IP takes `Rcl` as a **zonal** parameter inside Simandoux (E63/E64); Geolog carries no per-clay
resistivity at all. A Techlog multi-clay Simandoux model can therefore give illite and kaolinite
different shale resistivities in **one zone**; IP's cannot. This is a real modelling capability
difference, not a parameter difference. → `SB-MIN-031`, escalation `ESC-7`.

### F-17 — Elan's Simandoux takes silt as a first-class term, and only Techlog ships a silt endpoint

**Tier:** T2-equivalent (Elan Eq 78, `image2967.gif`; Table 28, `image2968.gif`) ·
T1 (`QM_MineralTable.xml` `Silt` row).

Elan Eq 78 carries `V_silt` in both the numerator exponent group and the denominator
`1 − (V_cl + V_silt)^(swshe+1)`. Neither IP's `MINDEF.PAR` nor Geolog's RF04 6.2 carries a silt row.
Elan's Table 28 defaults `ersh = 1.0` and `swshe = 0.5` correspond to Worthington's `x = 1.0`,
`c = 1.5` and *"essentially assume that by default in ELANPlus the silt behaves in the same manner as
clay in relation to the conductivity"*. **IP's E63/E64 "Simandoux" and Elan's Eq 78 "Simandoux" are
different equations with the same name** — Elan's is a documented Worthington Type-2 with a full
citation chain, IP's is the compact form with no citation. → `SB-MIN-030`, and the refusal in §7
against merging them under one label.

### F-18 — Two vendors independently document the same "1.5 % of range" uncertainty rule

**Tier:** T2-equivalent (Elan Table 29/30) · T3 (Geolog §E/§L, *"default = ~1.5 % of the tool's normal
logged range; weight = 1/U²"*).

Elan's balanced-uncertainty column reproduces **1.5 % of (MAX − MIN)** exactly on 9 of Table 29's 14
rows (RHOB, NPHI, GR, U, SIGM, EATT, VOLS, PHIT, ENPA) and deviates on 5 (DT, CUDC, CXDC, TPL, VELC)
by 4–7 %, with a sixth deviation on Table 30's `SDPT`. The deviations' implied ranges (150 µs/ft,
40.0 ns/m, 33.3) are tidier than the tabulated MIN/MAX — they are rounded or hand-adjusted values.
**Reading: 1.5 %-of-range is the *generating* rule, and on four rows the vendor's own table does not
follow it.** The practical instruction is therefore reinforced rather than weakened: **store MIN/MAX
and the printed default as two separate fields, derive nothing at runtime, and let the printed value
win.** IP states no such rule; its `MINEQDEF.PAR` numbers are simply tabulated.
→ `SB-MIN-019`, `SB-MIN-020`.

### F-19 — Variable `m` is bit-identical across two independent products

**Tier:** T2 (IP E68, `embim269.png`) · T2-equivalent (Elan Eq 65/66, `image2948.gif`).

`m* = m + Cm(0.258·Y + 0.2(1 − e^(−16.4·Y)))` with `Y = Qv·φT/(1 − φT)`. **The coefficients 0.258,
0.2 and 16.4 are identical in IP and Schlumberger ELAN** — the strongest single corroboration in the
domain. The base `m` default differs (IP 2.0, Elan `mdw` 1.8). IP additionally carries a
Waxman-Smits variant Elan does not document, `m* = m + Cm(1.128·Y + 0.22(1 − e^(−17.3·Y)))`, with no
second source. → `SB-MIN-023`.

### F-20 — All three vendors mix U, never Pe, and IP's two magic constants are Geolog's conversion

**Tier:** T2 (IP E26, `embim232.png`, printed twice) · T2-equivalent (Elan Eq 16) · T3 (Geolog §I/§J).

IP prints `U = Pef × (ρb + 0.1883) × 0.93423`. Geolog states the density chain
`ρ_a = 1.0704·ρ_e − 188.3 (kg/m³)`. `1/1.0704 = 0.934230…` and `188.3 kg/m³ = 0.1883 g/cc` —
**IP's two constants are exactly the inverse-and-offset of Geolog's electron-density conversion.**
IP has **no Pe equation type at all**. Techlog stores Pe and U as separate columns so either can seed
the dialog — a convenience, not different physics. This confirms the house rule from
`reference_tool_response_constants` independently at three vendors. → `SB-MIN-012`.

### F-21 — NEW: Techlog ships two mutually inconsistent clay libraries in one install

**Tier:** T1, both files read directly this session, read-only:
`Techlog 2018.2 (r22885)\PythonScripts\QElan_PostProcess_Using_Conductivities.py` (lines 736, 749,
762, 775, 788, 801 for WCLP; 814, 827, 840, 853, 866, 879 for CEC; unit declarations at 727, 805) and
`QM_MineralTable.xml` via `techlog_ingest/E_mineral_endpoints.json`.

Techlog's shipped ELANPlus post-processing source declares a per-clay `(CEC, WCLP)` pair with units
stated on every parameter — `CEC_*_unit = u"meq/g"`, `WCLP_*_unit = u"m3/m3"`. Its values **do not
match** the `QM_MineralTable.xml` CEC column in the same install:

| Clay | `QElan_PostProcess…py` CEC | `QM_MineralTable.xml` CEC | Spread |
|---|---|---|---|
| Illite | **0.16** | **0.25** | **56 %** |
| Kaolinite | **0.09** | **0.10** | 11 % |
| Chlorite | **0.15** | **0.15** | 0 % |
| Smectite | **1.0** | **0** (placeholder) | — |
| Glauconite | **0.233** | — | — |
| Shale | **−9999** (explicit "no default") | — | — |

Three consequences, and all three are new:

1. **The `p.u.`-vs-`v/v` conflict on WCLP is settled by T1 vendor source.** The dossier could only
   reach a *leaning* — Elan's Table 28 annotates `WCLP` as `p.u.` while the shipped
   `QM_MineralTable.xml` and Eq 11's dimensional coherence both imply v/v. **Techlog's own executable
   source declares `m3/m3` on every WCLP parameter.** The leaning becomes a statement. This closes
   the WCLP half of the dossier's escalation T-2. It does **not** close D-10, which is gated on the
   separate `φₑ` symbol inside Eq 78's `mc2/φₑ`, and no evidence here bears on that.
2. **`WCLP_Smectite = 1` and `CEC_Smectite = 1.0` are real Techlog values.** The dossier's
   Δ-4(b) declared SandiMin's code comment *"only smectite carries φ = 1.0"* false on the basis of
   `QM_MineralTable.xml`'s `Phicl = 0`. **Both files are Techlog's; the comment is true of the one
   the code actually cites.** Δ-4(b) is rebutted — see §3.4.
3. **Techlog encodes "no default" as an explicit sentinel** (`CEC_Shale = −9999`,
   `WCLP_Shale = −9999`) rather than as zero. **A vendor got the `ABSENT` discipline right that
   SandiBumi currently gets wrong** — see F-22 and `SB-MIN-007`.

→ `SB-MIN-008`, `SB-MIN-009`, `SB-MIN-046`.

### F-22 — NEW: each vendor's `(CEC, WCLP)` pair is internally self-consistent; the pair is the unit of adoption

**Tier:** T1 (the two Techlog files above) · T3 (Geolog `multimin_ref_spec.md` §B, which states both
`k_clay` from CEC **and** `k = WCLP/(1−WCLP)` and verifies they agree) · derived arithmetic, shown.

`CEC` and `WCLP` are two parameterisations of **one** physical quantity — bound water per unit dry
clay, `k = V_bw/V_dryclay`. A library that ships both must ship them consistent, and each vendor
does. Evaluating `k = α·96·CEC[meq/g]·ρ_dcl[g/cc]/(T°C + 298)` against `k = WCLP/(1−WCLP)` at the
dossier's own verified fixture conditions (α = 1, T = 64.4 °C):

| Clay | Library | CEC | WCLP | ρ_dcl | `k` from CEC | `k` from WCLP | Agreement |
|---|---|---|---|---|---|---|---|
| Illite | Geolog RF04 6.2 | 0.25 | 0.1555 | 2.78 | 0.18411 | 0.18413 | **0.01 %** |
| Illite | Techlog `QElan…py` | 0.16 | 0.104 | 2.78 | 0.11783 | 0.11607 | **1.5 %** |
| Kaolinite | Geolog RF04 6.2 | 0.10 | 0.06489 | 2.62 | 0.06940 | 0.06939 | **0.02 %** |
| Kaolinite | Techlog `QElan…py` | 0.09 | 0.058 | 2.62 | 0.06246 | 0.06157 | **1.4 %** |
| Glauconite | Techlog `QElan…py` | 0.233 | 0.156 | 2.96 | 0.18270 | 0.18483 | **1.2 %** |

Arithmetic shown for the first row so it can be checked without re-deriving:
`96 × 0.25 × 2.78 / (64.4 + 298) = 66.72/362.4 = 0.184106`; `0.1555/0.8445 = 0.184133`.

**The obligation this generates.** A `(CEC, WCLP)` pair is a **matched pair from one library**, and
mixing halves of two libraries silently breaks the equivalence of the two bound-water routes.
SandiBumi's shipped library does exactly that — §3.4 quantifies the result at **58.6 % on Illite**,
against ≤1.5 % for either vendor's own pair. → `SB-MIN-008` (P0), `SB-MIN-009` (P0).

---

## 3. SandiBumi as-built

Written from the source this session. Every claim carries `file:line`. The repository was read-only
for this task except this chapter file.

### 3.1 Module inventory and entry points

| Module | File | Lines | State |
|---|---|---|---|
| **SandiMin** — the generalised N-component solver | `src-tauri/src/multimin2.rs` | 4,364 | live; this domain's implementation |
| **Multimin** — the legacy fixed 4-component solver | `src-tauri/src/multimin.rs` | 67 | **retired**; spec-only, see §3.6 |

SandiMin is **not** a `modules.rs` chain step. It has its own request type (`MultiminRequest`,
`multimin2.rs:408`), five Tauri commands (`multimin_library`, `run_multimin`,
`multimin_fluid_calc`, `multimin_dry_clay`, `multimin_fluid_from_precalc` — `src/ipc.ts:1574`,
`:1580`, `:1584`, `:1608`, `:1620`), and a dock-hosted pane rather than a popup
(`src/ui/multiminDialog.ts:151`). It writes a versioned log set defaulting to `SANDIMIN`
(`multimin2.rs:40`, `:423`, `:1705`, `:1725`). Physics defaults are single-sourced in Rust and the
dialog edits a working copy (`multiminDialog.ts:40`) — a good structural choice, and the reason the
endpoint-library defects in §3.4 are fixable in exactly one place.

### 3.2 Forward model, zone assignment and solver

**Forward model — `PRESENT-OK`.** `t_k = Σ_i P[k][i]·v_i` over 14 canonical tool keys (`TOOL_KEYS`,
`multimin2.rs:2050`: RHOB, NPHI, DT, GR, PEF, U, THOR, POTA, URAN, VP, VS, EPT, EATT, SIGMA). Rows
are scaled by `w = 1/σ` (`scaled`, `multimin2.rs:891`; applied per tool kind at `:1329`, `:1343`,
`:1359`), so the effective weight in the squared objective is `1/σ²` — matching all three
incumbents (F-18).

**Pe→U conversion — `PRESENT-OK`, and it is the one thing already right that everything else depends
on.** `rho_e(rhob) = (rhob + 0.1883)/1.0704` (`multimin2.rs:1786`–`:1788`). A PEF tool is tagged
`TKind::Pef(sigma)` at assembly (`multimin2.rs:1120`, enum at `:1800`, key test `is_pef_key` at
`:1779`), its **measured reading** is converted to U before entering the system —
`(raw * re, w, scaled(&rows[t], w))` at `multimin2.rs:1359` — and the reconstruction is inverted back
to Pe for display (`recon_display`, `multimin2.rs:1757`, `:1767`–`:1770`). The `rho_e` doc comment
names the rationale: it is *"Shared with the legacy `multimin` module so both solvers convert Pe↔U
with one relation (divergent Pe physics between the two solvers is the hazard this centralises
away)."* This is the house U-not-Pe rule implemented, and §3.6 records that the legacy module was
brought onto the same relation before it was retired.

**Zone assignment — `PRESENT-DIVERGENT`.** `classify` (`multimin2.rs:801`–`:867`) assigns each fluid
categorically to the X zone, the U zone, or both; `cond_tool_row` (`multimin2.rs:899`–`:918`) then
makes CT read the U set against `Cw`/`Cbw_u` and CXO the X set against `Cmf`/`Cbw_x`. **Every
non-conductivity tool reads whatever fluids are declared shared or X.** This is Geolog's categorical
model, hard-coded. **There is no per-equation invasion factor**: `grep` over the module finds
`invasion` only in prose (`multimin2.rs:455`, `:584`, `:1054`), never as a parameter. IP's continuous
per-equation `IF ∈ [0,1]` has no analogue, so a model that needs the neutron to read a *mixture* of
invaded and virgin fluid — the case IP's own documentation calls out by name — cannot be expressed.
→ `SB-MIN-025`.

**Solver — `PRESENT-OK`.** `solve_bounded_lsq` (`multimin2.rs:1841`–`:2006`) is a bounded,
equality-constrained active-set least squares: it solves the KKT system on the free set with a unity
Lagrange multiplier (`:1920`–`:1926`), line-searches to the first bound crossing (`:1972`–`:1979`),
fixes components at `AtLo`/`AtHi` (`:1986`–`:1994`), and releases a bound component whose KKT
multiplier has the wrong sign (`:1946`–`:1968`). Outer cap `max_outer = 8n + 12` (`:1893`); the inner
solve is Gaussian elimination with partial pivoting returning `None` on a singular system
(`solve_linear_opt`, `:2009`). **This is genuine bounded optimisation, in the Goldfarb-Idnani /
DNOPT class — not IP's mineral-deletion heuristic.** F-1 is satisfied on mechanism. What is missing
is the *disclosure* of the divergence to a user migrating a deck from IP → `SB-MIN-002`.

**Box bounds — `PARTIAL`.** `hi[i] = c.max_vol.min(1.0)`, falling back to 1.0 when `max_vol ≤ 0`
(`multimin2.rs:1189`). The library's `fl` constructor (`multimin2.rs:2073`) sets `max_vol = 0.5` for
fluid rows — Geolog's cited default — while `m` (`:2067`) and `clay` (`:2070`) leave 1.0. **But the
serde default on the wire is 1.0, not 0.5** (`default_one`, `multimin2.rs:74`–`:80`), and the dialog
rebuilds the field as `maxMap.get(c.name) ?? 1` (`multiminDialog.ts:1093`) from a map keyed on the
*library* name (`:180`). **A user-added or renamed fluid therefore silently receives a 1.0 box
instead of 0.5.** The subset that behaves correctly is exactly "fluids whose name still matches a
library row" — a property no type enforces. → `SB-MIN-005`.

### 3.3 Constraints — `PARTIAL`; the divergences are deliberate but invisible in-product

| Constraint | Dossier spec (§5.1 F-4) | As built | Status |
|---|---|---|---|
| UNITY | HARD equality, X-only fluids excluded | HARD — Lagrange row in the KKT system, `unity_c` from `zs.unity` (`multimin2.rs:1190`, `:804`, `:820`, `:826`–`:833`) | `PRESENT-OK` |
| BOX | `0 ≤ v ≤ hi`, fluids 0.5 | `multimin2.rs:1189` | `PARTIAL` (§3.2) |
| POROSITY | `TOOL` σ = 0.01: hard equality **and** pseudo-measurement | **soft half only** — one weighted row `Σ X fluids − Σ U fluids = 0` (`multimin2.rs:1146`–`:1158`, weight `:1170`) | `PRESENT-DIVERGENT` |
| BNDWAT_X / BNDWAT_U | `TOOL` σ = 0.01 | **soft half only** — `bndwat_soft_rows` (`multimin2.rs:924`–`:968`), static at constant T (`:1164`) and rebuilt per sample under an FTEMP curve (`:1380`) | `PRESENT-DIVERGENT` |
| WATER MUD | HARD inequality `Σ(X waters) − Σ(U waters) ≥ 0` | **detect-and-re-solve once, with a soft *equality* row** (`multimin2.rs:1403`–`:1415`) | `PRESENT-DIVERGENT` |
| OBM mirror + `v_Xgas ≤ v_Ugas` | HARD inequality | **absent** — for oil mud the WATER MUD row is simply suppressed (`multimin2.rs:1175`) and nothing replaces it | `ABSENT` |
| PHIMAX | HARD inequality, opt-in | absent | `ABSENT` |
| BVIRR | HARD inequality, opt-in | absent | `ABSENT` |
| IRRWAT | `TOOL` σ = 0.01 | absent | `ABSENT` |

Three of these need stating precisely, because the magnitude is not obvious from the table.

**The `Tool` half that is missing (dossier Δ-1).** The module header states the design honestly at
`multimin2.rs:17`–`:19`: *"Soft `Tool` constraints at σ = 0.01 (treated as pseudo-measurements)."*
Geolog's `Tool` class is **both** a hard equality **and** a σ = 0.01 pseudo-measurement that adds a
degree of freedom; SandiBumi implements the second half only. `SIGMA_CONSTRAINT = 0.01`
(`multimin2.rs:768`) is the default and is user-overridable (`req.sigma_constraint`, `:1169`), which
makes the omission worse rather than better: a user who relaxes σ to 0.1 for convergence also
relaxes the porosity tie, and **nothing reports how far the tie was actually violated.** There is no
tie-residual QC curve. → `SB-MIN-035`.

**WATER MUD is doubly divergent (dossier Δ-2, escalation E-8), and one half of that is new here.**
First, it is IP's pattern — solve, detect violation, re-solve — not Geolog's or Elan's constrained
form; the module header says so at `multimin2.rs:20`. Second, **the re-solve row is an equality, not
an inequality**: `a2.push(wm.iter().map(|e| e * soft_weight).collect()); b2.push(0.0);`
(`multimin2.rs:1409`–`:1410`) drives `Σ(X waters) − Σ(U waters)` toward **zero**, i.e. toward
`Sxo = Sw`, where the physical constraint at `multimin2.rs:455` only requires `≥ 0`. A hard
inequality parks the solution at the boundary and lets the data set the rest; a soft equality pulls
*through* the boundary and suppresses genuine movable hydrocarbon. Third, the correction runs
**once** — `if s < −1e-6 { … }` with no re-check (`multimin2.rs:1406`) — so a re-solve that
re-violates is not caught. This is precisely the oscillation IP's own "add one limit at a time, in
grid order" rule exists to prevent. → `SB-MIN-034`; escalation `ESC-3`.

**Degrees of freedom are counted and reported — `PRESENT-OK`, and this already beats two incumbents.**
`n_extra = soft.len() + bndwat_static.len() + (unity as usize)` (`multimin2.rs:1195`);
`dof = tools + n_extra − n` (`:1204`); and when `dof == 0` a `dof_note` (`:516`–`:518`, `:1205`) says
in words that the reconstruction cannot discriminate the model. A run with too few live tools fails
with a named, countable error rather than solving something meaningless (`:1197`–`:1202`). The
module carries a test whose name is itself the argument —
`an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so`
(`multimin2.rs:2508`), with an over-determined control at `dof = 2` proving the same +0.4 g/cc
endpoint error *is* caught when the DOF exists
(`recon_qc_emits_per_tool_curves_and_flags_endpoint_error`, `:2333`; `assert_eq!(clean.dof, 2)` at
`:2412`). **Neither IP nor Elan ships a DOF check at all** (F-13). This is a differentiator that
already exists and is not yet claimed. → `SB-MIN-016`.

### 3.4 The endpoint library — `PRESENT-DIVERGENT`, and the most expensive finding in this chapter

`LIB` is 27 rows × 14 tool keys (`multimin2.rs:2079`–`:2108`), exposed through `multimin_library()`
(`:2110`), with display units declared once in a block comment (`:2044`–`:2046`) and nowhere in the
type. `IP_PROVENANCE.md` §2.2 already records the table as *"merged from two vendor installs, in
Interactive Petrophysics' mineral-dropdown order"*, rates it Tier A, flags the curated-selection
question as open, and names the fix — *"Add a primary-literature citation per row where one exists
(Schön, Ellis & Singer, SPWLA references) … **This is work SandiBumi should do anyway** and it is
currently unscheduled."* Four concrete defects follow from that unscheduled work.

**(a) The generic `Clay` row contributes zero bound water, silently, under the default route.**
`clay("Clay", 0.00, 0.120, …)` (`multimin2.rs:2098`) ships `CEC = 0.00`. `bound_water_multiplier`
(`multimin2.rs:616`–`:630`) computes `cec_k = if cec > 0.0 { … } else { 0.0 }` (`:617`); under the
**default** `PorositySource::Cec` (`multimin2.rs:136`–`:143`) that returns `k = 0.0` with **no error,
no warning, no NaN.** `WCP_PHYSICAL_CEILING = 0.5` (`multimin2.rs:784`) guards the `φ → 1` pole at
the top; **nothing guards the `CEC = 0` floor at the bottom.**

| Route | `k` | `V_bw` at `V_clay = 0.30` |
|---|---|---|
| `Cec` (**the default**) | **0** | **0.000 v/v** |
| `WetClayPorosity` | `0.120 / 0.880 = 0.136364` | **0.0409 v/v = 4.1 pu** |

**PHIE reads 4.1 pu too high and SWE correspondingly too low on a 30 % clay rock, and one radio
button in the dialog changes it.** The generic `Clay` row is the row a user picks for a quick
sand/clay/fluid model — the wrong answer is the default answer. This is the dossier's Δ-4(a), but
the dossier framed it as a hazard *imported* from a vendor's Smectite row; the live case is worse,
because it is in **SandiBumi's own shipped library, on its most generic clay**. Contrast F-21:
Techlog encodes the identical "no default exists" state as `CEC_Shale = −9999`, an explicit
sentinel. SandiBumi encodes it as `0.0`, which is arithmetically indistinguishable from a measured
zero. CONTRACT §2 already has the right vocabulary for this state — `ABSENT — ships with no
default` — and the code does not. → `SB-MIN-007` (P0), test `SB-MIN-T07`.

**(b) The clay rows pair `CEC` from one vendor with `WCLP` from another, so the two bound-water
routes disagree by up to 58.6 %.** Verified against both vendor files this session. SandiMin's `CEC`
column is Geolog RF04 6.2 (Illite 0.25, Kaolinite 0.10, Chlorite 0.15, Montmorillonite 1.0,
Glauconite 0.20); its `wcp` column is byte-exact Techlog `QElan_PostProcess_Using_Conductivities.py`
(Glauconite 0.156, Kaolinite 0.058, Chlorite 0.101, Illite 0.104, Smectite 1). Per F-22 each
vendor's own pair is self-consistent to ≤1.5 %. The chimera is not:

| Clay | `multimin2.rs` | `k` from CEC | `k` from WCLP | **Divergence** | Cause |
|---|---|---|---|---|---|
| Illite | `:2096` CEC 0.25, wcp 0.104, ρ 2.78 | 0.18411 | 0.11607 | **+58.6 %** | Geolog CEC 0.25 vs Techlog CEC 0.16 |
| Glauconite | `:2093` CEC 0.20, wcp 0.156, ρ 2.96 | 0.15682 | 0.18483 | **−15.2 %** | Geolog CEC 0.20 vs Techlog CEC 0.233 |
| Kaolinite | `:2094` CEC 0.10, wcp 0.058, ρ 2.62 | 0.06940 | 0.06157 | **+12.7 %** | Geolog CEC 0.10 vs Techlog CEC 0.09 |
| Chlorite | `:2095` CEC 0.15, wcp 0.101, ρ 2.81 | 0.11166 | 0.11235 | **−0.6 %** | *both vendors ship 0.15 — no chimera* |
| Montmorillonite | `:2097` CEC 1.0, wcp 1.0, ρ 2.63 | 0.69669 | ceiling → `cec_k` | 0 % | guard at `:623`–`:624` fires correctly |

**The diagnosis is decisive because the divergence appears exactly where the two vendors' CEC values
differ and vanishes on the one clay where they agree.** At `V_dryclay = 0.25`, Illite's bound water
is 0.04603 v/v on the CEC route against 0.02902 on the WCLP route — **1.70 pu of PHIE that moves
with a radio button and no physics.** Had SandiMin shipped Geolog's own `WCLP = 0.1555` beside
Geolog's `CEC = 0.25`, the routes would agree to **0.01 %**.

> **Dossier Δ-4(b) and Δ-4(c) are rebutted on T1 evidence, and the correction changes what must be
> built.** Δ-4 asserts the WCLP column is *"mis-attributed"*, that *"Techlog gives Illite 0.17,
> Kaolinite 0.07, Chlorite 0.07, Smectite 0"*, that three of six values are IP `MINDEF.PAR`
> `PhiTClay` values wearing a Techlog label, and that Illite 0.104 and Montmorillonite 1.0
> *"match neither vendor"*. **That check used `QM_MineralTable.xml`. The code cites a different
> Techlog file** — `multimin2.rs:2059`–`:2060`, verbatim: *"Techlog WCLP defaults from
> QElan_PostProcess_Using_Conductivities.py"* — and **all five values are byte-exact in that file**
> (`WCLP_Glauconite 0.156`, `WCLP_Kaolinite 0.058`, `WCLP_Chlorite 0.101`, `WCLP_Illite 0.104`,
> `WCLP_Smectite 1`). The attribution is correct; the three-row coincidence with IP's `PhiTClay` is
> real but incidental. Δ-4(b) falls with it: `WCLP_Smectite = 1`, so the code comments at
> `multimin2.rs:612` and `:779`–`:780` are true of the file they cite. **What survives Δ-4 — and is
> worse than Δ-4 stated — is (a) above and the pairing defect here.** Neither is an attribution
> error. The real lesson is sharper than the dossier's: **a correct citation on one column and a
> correct citation on another column still produce a wrong answer when the two columns are a matched
> pair.** No per-row provenance scheme that cites columns independently would have caught this.
> → `SB-MIN-008` (P0), `SB-MIN-009` (P0).

**(c) The one clay row that matches no vendor at all is the generic `Clay`.** `wcp = 0.120`
(`multimin2.rs:2098`) appears in neither Techlog file, nor IP's `PhiTClay` column, nor Geolog RF04
6.2; Techlog's corresponding `WCLP_Shale` is the `−9999` sentinel. It is an unsourced number in the
one row whose entire purpose is to be selected without thinking. → `SB-MIN-009`.

**(d) Fluid endpoints are Geolog's, chosen silently.** `Oil Sxo` / `Oil Sw` DT **189.0** and
`Gas Sxo` / `Gas Sw` DT **250.0** (`multimin2.rs:2104`–`:2107`) are Geolog RF04 6.2's values — the
low end of the oil spread and the middle of the gas spread in F-5. There is no library selector, so
the 45 µs/ft gas divergence between vendors — and its ≈5.4 µs/ft row contribution, 1.8× IP's own
sonic confidence — is resolved on the user's behalf without the user being told a choice was made.
`PRESENT-DIVERGENT`. → `SB-MIN-028`, `SB-MIN-029`.

**(e) No row carries a wet/dry flag, a unit label, or a source string.** `LibRow`
(`multimin2.rs:2053`–`:2065`) is `name`, `kind`, `zone`, `fluid_type`, `cec`, `wcp`, `max_vol` and an
11-element `v` array. Illite's `RHOB 2.78` is a **dry**-clay density; Chlorite's `2.81` is an IP
**wet**-clay-convention value. That is F-4's 5.7 σ / 6.9 pu hazard sitting in one shipped table with
nothing in the type system to catch it, and the CEC field's unit (meq/g, documented only in prose at
`multimin2.rs:65` and `:603`) is the unit trap that a factor of 1,000 hides in. `ABSENT`.
→ `SB-MIN-009`, `SB-MIN-010`, `SB-MIN-011`.

### 3.5 Saturation, statistics and diagnostics

**Seven Sw models — `PRESENT-OK`; owned by the `SAT` chapter.** `SwModel` (`multimin2.rs:103`–`:123`)
with `is_post_solve` (`:128`–`:130`). The post-solve contract preserves PHIE and hard unity by
rescaling the water and hydrocarbon groups to the model's Sw (`set_group`, `multimin2.rs:872`–`:888`;
applied `:1417`–`:1428`), and **RECON is computed after the redistribution** (`:1419`–`:1420`), so it
also measures how well the chosen Sw model coheres with every tool. That ordering is a genuinely
good design decision and should be preserved explicitly rather than by accident. `waxman_b(t_c, rw)`
(`multimin2.rs:326`) is the Juhász closed-form fit with T in °C.

**Fluid physics — `PRESENT-OK`.** `fluid_calc_at` (`multimin2.rs:574`–`:600`) computes
`w = 0.75m + 0.25n` with a guard to 2.0 when non-finite or ≤ 0.5 (`:575`–`:576`);
`cbw = 0.0007·(T_C + 8.5)·(T_C + 298)` (`:580`) — Geolog's form, carrying the same
Clavier-Coates-Dumanoir `(T + 8.5)` term as Elan Eq 62; and auto row uncertainties
`u_ct = 0.03·cw^(1/w)`, `u_cxo = 0.03·cmf^(1/w)` (`:597`–`:598`). `alpha_expansion`
(`multimin2.rs:557`–`:563`) is `sqrt(20455/S)` below 20,455 ppm **capped at 5.0** — the cap is a
SandiBumi engineering guard, not a vendor value, and is labelled as such in §5.
`bndwat_multiplier` (`multimin2.rs:604`–`:606`) uses the constant `96.0` with ρ in g/cc, which is the
correct half of the unit trap: Geolog's printed `0.096` requires ρ in kg/m³, and
`1 × 96 × 0.25 × 2.78 / 362.4 = 0.18411` reproduces the dossier's verified fixture. **This is why
SandiMin's fresh-water behaviour is already right**, and it should be stated as a claim rather
than left as an implementation detail.

**FTEMP from a curve — `PRESENT-OK`, and it is the right failure pattern.** `ftemp_curve`
(`multimin2.rs:438`) recomputes every temperature-dependent term per depth (`:1240`, `:1310`–`:1315`,
`:1380`) behind a physical-plausibility window `FTEMP_MIN_F = 32.0` / `FTEMP_MAX_F = 600.0`
(`multimin2.rs:775`–`:776`), which rejects −999.25 and 9999 fills and reverts to the constant
temperature rather than propagating a nonsensical T into the fluid calc. This is fail-safe-and-say-so
done correctly, and is the pattern §3.4(a) is missing.

**`RECON` — `PRESENT-UNVERIFIED`.** Emitted as `{prefix}_RECON` (`multimin2.rs:1685`), described in
code as *"the incoherence (σ-weighted RMS residual over live tool rows; Quanti.Elan Eq 79)"*
(`:1684`), computed at `:1548`–`:1574`, with a per-well `mean_recon` (`:495`, `:1743`). The module
header attributes the statistic's family to **Mayer & Sibbit, SPE 9341 (1980), "GLOBAL, a new
approach to computer-processed log interpretation"** (`multimin2.rs:24`–`:29`) — a named primary
source. Optional per-tool decomposition `{prefix}_{KEY}_REC` and `_DIF` sits behind `recon_qc`
(`multimin2.rs:31`–`:32`, `:444`, `:1687`–`:1692`), which is more than either IP or Elan emits by
default. **Unverified** because no test proves the value equals the long-form Elan computation with
`LargestWeight` retained, so a refactor could reintroduce `LW` asymmetrically without failing
anything. → `SB-MIN-013`, test `SB-MIN-T13`.

**`TOTERR_IP`, `QUALITY`, `CONDNUM` — `ABSENT`.** `grep` over `multimin2.rs` returns zero
occurrences of `toterr`, `TOTERR`, `QUALITY` and `condnum`. A user cannot compare a SandiMin number
against an IP or Geolog number, which is the whole of F-3. More seriously, there is **no conditioning
diagnostic of any kind**: the design matrix is assembled and solved (`multimin2.rs:1913`–`:1930`)
with singularity detected only as a Gaussian-elimination failure returning `None` (`:1927`–`:1929`),
which drops the sample and continues (`:1401`). **An ill-conditioned but still invertible system —
the common case, and the one that produces confident nonsense — carries no flag at all.**
`CONDNUM` is the cheapest differentiator available in this domain (F-13). → `SB-MIN-014`,
`SB-MIN-015`.

**Output nomenclature — `PARTIAL`.** Emitted: `VOL_*` per component (`multimin2.rs:1215`), `PHIE`,
`PHIT`, `SWE`, `SWT` (`:1664`–`:1667`), `SXOT` (`:1674`), `MOVEDHC` (`:1677`), `VSH` (`:1682`),
`RECON` (`:1685`). **No bare `SW` is emitted anywhere** — ledger item D-15 is satisfied in code.
But **`SXOE`, `PHIE_X` and `PHIT_X` are absent** while the adoption spec's F-11 names all three, and
`SXOT` shipping without its effective counterpart is an asymmetry a user will notice on the first
crossplot. `VSH` is `Σ(clays) + Σ(U bound water)` (`:1681`) — that is the dossier's `VOL_WETCLAY`
emitted under a different name, with no wet/dry declaration on the curve.
→ `SB-MIN-036`, and it interacts with `CLY`'s `VSH` seam.

**Absent entirely.** No weight multiplier (F-8), no `active`/`weight = 0` separation, no Monte Carlo
and therefore no seed, no per-volume predicted uncertainty, no balanced pre-solve uncertainty, no
endpoint-library selector, no `.neu` neutron-table handling, no excavation term, no variable `m`, no
Shell `m`, no core-calibration regression, no silt component, no per-clay `Rsh`, and none of
PHIMAX / BVIRR / IRRWAT / OBM.

### 3.6 The legacy `multimin` module — retired, with one live residue

The commissioning brief for this chapter carried two findings about this module as current defects.
The code says otherwise, and the repository's own history says when it changed. Recording the
correction here because the brief's worked example is quantitatively right and is the reason the
present code is correct — it is the fix's rationale, not an outstanding bug.

| Brief's finding | Verified state | Evidence |
|---|---|---|
| Legacy solver "mixes per-electron PEF linearly by volume": 0.5·1.81 + 0.5·0.36 = 1.085 b/e against a correct 1.382 b/e, a 0.30 b/e residual = 1.0× `SIG_PEF`, biasing `VOL_CLAY`/`VSH_MM` upward | **Fixed.** Commit `8fc873b`, *"R17: the legacy Multimin solver now mixes PEF as volumetric U, not raw per-electron Pe"* — every endpoint and the measured reading are converted to `U = Pe·ρe` before the linear system, and the uncertainty is carried in U space as `σ_PEF·ρe`. The brief's worked example **is that commit's own message.** | `git log --oneline -- src-tauri/src/multimin.rs` |
| The test at `multimin.rs:364` forward-models with the same wrong law (`let pef = vs * 1.81 + vw * 0.36;`) so it passes by construction | **Fixed in the same commit.** The forward model became `u_mix = vs·1.81·ρe(2.65) + vw·0.36·ρe(1.0); pef = u_mix/ρe(rhob)`, and a new test `multimin_pef_uses_volumetric_u_mixing` asserts both `pef ≈ 1.382` **and** that the two mixing laws differ by more than 0.25 — the wrong law is now pinned as wrong rather than assumed. | same commit |
| Hidden from UI pickers but "STILL REGISTERED at `modules.rs:195`/`:234`, so any saved chain or saved dockview layout still runs it" | **Superseded** by commit `73f952d`, *"R22: gracefully retire the legacy multimin module"*. `grep -rn multimin src/ --include=*.ts` returns **no** `ribbon.ts` or `workflowDialog.ts` hit. The spec is still catalogued (`modules.rs:382`) **by design**, but `run_module` blocks it first: `if let Some(msg) = retired_module(name) { return Err(msg.to_string()); }` (`modules.rs:418`–`:420`), with the message naming the replacement — *"Re-run this step with SandiMin (Advance ▸ Mineral Solver)"* (`modules.rs:405`–`:408`). A saved chain step **fails loudly and actionably** instead of silently running superseded physics. A test pins both halves: `multimin_is_retired_but_still_cataloged` (`modules.rs:2856`), which also asserts a live module is *not* flagged (`:2859`). | `modules.rs`, `multimin.rs:1`–`:13` |

**Status: `PRESENT-OK` on the retirement mechanism.** Resolve by name, render stored parameters,
refuse to compute is the correct pattern, and it is better than deleting the spec — which would
break every saved chain that references it. The comment at `modules.rs:400`–`:402` states the
trade-off explicitly (*"Adding a name here is the whole retirement; there is no per-spec flag to
thread through the ~40 module literals"*). This is worth a requirement precisely because it is
already right and could be lost: → `SB-MIN-041`'s second clause.

**The live residue.** `multimin_spec()` still ships 20 hard-coded parameter defaults
(`multimin.rs:31`–`:52`), including `RHOB_CLAY 2.55` (`:32`) and `PEF_CLAY 3.10` (`:44`), which
**share no table with `multimin2::multimin_library`** — SandiMin's generic `Clay` row is RHOB 2.65 /
PEF 3.50 (`multimin2.rs:2098`), and **no clay in SandiMin's library carries either legacy value.**
These no longer drive a computation, but `list_modules` returns the spec (`modules.rs:382`) and the
retirement's stated purpose is that a saved step *"can render its stored parameters"* — so they are
**displayed to a user as though they were this product's clay endpoints**, with no provenance and no
relationship to the solver that replaced them. Smaller than the brief describes; still a defect.
`PRESENT-DIVERGENT`. → `SB-MIN-041`.

### 3.7 Status summary

| Area | Status |
|---|---|
| Forward model; U-not-Pe conversion; bounded active-set solve; hard unity; DOF counted and reported; FTEMP plausibility window; CEC bound water with α expansion; post-solve Sw preserving PHIE and unity | `PRESENT-OK` |
| Box bounds — fluid 0.5 ceiling holds only for library-named rows | `PARTIAL` |
| POROSITY / BNDWAT soft half only; WATER MUD as a once-only soft *equality* re-solve | `PRESENT-DIVERGENT` |
| OBM, PHIMAX, BVIRR, IRRWAT constraints | `ABSENT` |
| Endpoint provenance per value; wet/dry flags; `(CEC, WCLP)` matched pairing; generic-clay silent `k = 0` | `PRESENT-DIVERGENT` |
| Fluid sonic endpoints resolved to one vendor silently; no library selector | `PRESENT-DIVERGENT` |
| Zone assignment categorical only; no per-equation invasion factor | `PRESENT-DIVERGENT` |
| Output nomenclature — `SXOE`, `PHIE_X`, `PHIT_X` missing; `VSH` undeclared wet/dry | `PARTIAL` |
| `RECON` equivalence to Elan Eq 80 under `LargestWeight` | `PRESENT-UNVERIFIED` |
| `CONDNUM`, `QUALITY`, `TOTERR_IP`, weight multiplier, `active` flag, Monte Carlo + seed, per-volume uncertainty, endpoint-library selector, variable `m`, Shell `m`, excavation, core calibration, silt, per-clay `Rsh` | `ABSENT` |
| Legacy `multimin` retirement mechanism | `PRESENT-OK` |
| Legacy `multimin` orphan endpoint defaults still rendered | `PRESENT-DIVERGENT` |

---

## 4. Requirements

46 requirements, `SB-MIN-001` … `SB-MIN-046`. Ten are P0. Every block names at least one acceptance
test in §6; a candidate obligation that could not name one was moved to §7 rather than padded in
here.

### 4.1 Solver core

#### SB-MIN-001 — Solve as a bounded, non-negative least-squares problem [P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST obtain component volumes by minimising the σ-weighted sum of squared
tool residuals subject to `0 ≤ v_i ≤ hi_i` and the unity equality, using a bounded optimiser.
SandiBumi MUST NOT achieve non-negativity by deleting a component and re-solving a reduced model.

**Rationale.** F-1 (T2 IP manual, T2-equivalent Elan, T3 Geolog): IP's documented remedy for a
negative volume is to remove the mineral and re-solve, which returns a solution to a *different*
model; Elan and Geolog both use genuine constrained optimisation (Elan over-determined weighted LSQ,
Geolog Goldfarb & Idnani 1983 dual QP with a Powell VF02 SQP fallback). A deletion heuristic is
path-dependent: the answer depends on which component went negative first.

**As-built.** `PRESENT-OK` — `solve_bounded_lsq`, active-set KKT with bound fixing and sign-based
release, `multimin2.rs:1841`–`:2006`; unity Lagrange row `:1920`–`:1926`; outer cap
`max_outer = 8n + 12` at `:1893`.

**Verified by.** SB-MIN-T01

#### SB-MIN-002 — Disclose the solver-class divergence from IP in the run record [P2] [status: ABSENT]

**Requirement.** When a run is imported from, or exported for comparison with, an IP model, SandiBumi
MUST record in the run record that its non-negativity treatment is a bound constraint and not IP's
mineral-deletion heuristic, so a volume difference against IP is attributable rather than mysterious.

**Rationale.** F-1. A migrating user's first action is a side-by-side against their IP deck. The
difference is real, defensible and in SandiBumi's favour — but only if it is stated before the user
finds it. CONTRACT §5.2: where the vendors disagree, the disagreement is the product.

**As-built.** `ABSENT` — the result carries `dof`, `dof_note` and an optional error
(`multimin2.rs:516`–`:523`) and nothing about solver class.

**Verified by.** SB-MIN-T02

#### SB-MIN-003 — Impose unity as a hard equality over the non-X components [P0] [status: PRESENT-OK]

**Requirement.** The volume-summation constraint MUST be a hard equality on the solved system, and
components assigned exclusively to the flushed (X) zone MUST carry coefficient zero in it. A solution
MUST NOT be returned whose non-X volumes sum outside `1 ± 1e-9`.

**Rationale.** F-6/F-7. Elan makes unity hard; Geolog makes it a `Tool` row. Either is defensible, but
a soft unity lets the solver buy misfit reduction with mass, which is the failure a client notices
first. P0 as a **regression guard**: it is already correct, and it is the one contract every
downstream volume, PHIE and Sw depends on.

**As-built.** `PRESENT-OK` — `unity_c` built from `zs.unity` (`multimin2.rs:1190`, `:804`, `:820`),
X-only fluids excluded at `:826`–`:833`, imposed as a Lagrange row in the KKT solve
(`:1920`–`:1926`), and preserved through the post-solve Sw redistribution (`:1417`–`:1428`).

**Verified by.** SB-MIN-T03

#### SB-MIN-004 — Report the unity convention alongside any misfit statistic [P2] [status: ABSENT]

**Requirement.** Any exported misfit statistic MUST carry a label stating whether unity was counted as
a degree-of-freedom-bearing `Tool` row or as a hard equality.

**Rationale.** F-7 (T3 Geolog, verbatim *"only `Tool` rows add a degree of freedom"*). Geolog makes
unity a `Tool` row; SandiBumi makes it hard. That costs one degree of freedom and makes a SandiBumi
`QUALITY` read **higher** than a Geolog `QUALITY` on an identical fit. The difference must be
asserted, not discovered by a user who assumes the two numbers are comparable.

**As-built.** `ABSENT` — no misfit statistic is exported with a convention label, and `QUALITY` is not
computed at all (§3.5).

**Verified by.** SB-MIN-T14

#### SB-MIN-005 — Enforce the fluid volume ceiling structurally, not by name lookup [P2] [status: PARTIAL]

**Requirement.** The default upper bound for a component of fluid kind MUST be 0.5 v/v and MUST be
derived from the component's kind, not from a name match against the shipped library. A user-added or
renamed fluid MUST receive the same default ceiling as a library fluid.

**Rationale.** Geolog's cited default fluid ceiling is 0.5 v/v. A silently-widened box does not fail;
it lets a fluid absorb misfit that belongs to a mineral, which surfaces as a plausible but wrong
porosity rather than as an error.

**As-built.** `PARTIAL` — the ceiling holds only for fluids whose name still matches a library row.
`fl` sets 0.5 (`multimin2.rs:2073`) but the serde default is 1.0 (`default_one`, `:74`–`:80`), the
clamp falls back to 1.0 on any non-positive value (`:1189`), and the dialog rebuilds the field as
`maxMap.get(c.name) ?? 1` (`multiminDialog.ts:1093`) from a library-name-keyed map (`:180`).

**Verified by.** SB-MIN-T04

### 4.2 Bound water, clay convention and the endpoint library

#### SB-MIN-006 — Compute bound water from CEC with the salinity expansion term [P1] [status: PRESENT-OK]

**Requirement.** The CEC bound-water route MUST evaluate
`k = α · 96 · CEC[meq/g] · ρ_dryclay[g/cc] / (T[°C] + 298)` with `α = sqrt(20455 / S_ppm)` for
`0 < S_ppm < 20455` and `α = 1` at or above it.

**Rationale.** F-2 (T2 IP; T2-equivalent Elan Eq 62; T3 Geolog RF04 6.2). IP models bound water as a
flat zonal `φTclay`; Elan and Geolog both make it a function of CEC, dry-clay density, temperature
and — below ~20,455 ppm NaCl — a diffuse-layer expansion factor, which is precisely what a flat
`φTclay` cannot express.

**As-built.** `PRESENT-OK` — `bndwat_multiplier` `multimin2.rs:604`–`:606` (constant `96.0` with ρ in
g/cc, the correct half of the kg/m³-vs-g/cc unit trap); `alpha_expansion` `:557`–`:563`, capped at
5.0. The cap is a SandiBumi engineering guard, not a vendor value, and is labelled as such in §5.

**Verified by.** SB-MIN-T05, SB-MIN-T06

#### SB-MIN-007 — Refuse a clay whose bound-water parameter is absent; never treat it as zero [P0] [status: ABSENT]

**Requirement.** When the active porosity source is `Cec`, a clay component whose `CEC` is
non-positive MUST cause the run to fail with a message naming the component and the missing parameter.
SandiBumi MUST NOT compute a bound-water multiplier of zero from an absent CEC. Symmetrically, under
`WetClayPorosity` a clay whose `WCLP` is non-positive MUST fail the same way.

**Rationale.** CONTRACT §2 `ABSENT — ships with no default` expressed in code, and CONTRACT §5.3 (fail
loud where they fail silent). Techlog encodes the same "no default exists" state as an explicit
`−9999` sentinel (F-21); SandiBumi encodes it as `0.0`, which is arithmetically indistinguishable
from a measured zero. The shipped generic `Clay` row carries `CEC = 0.00` and the `Cec` route is the
**default**, so on a 30 % clay rock all bound water vanishes: PHIE reads **4.1 pu** high
(`0.30 × 0.120/0.880 = 0.0409 v/v`) and SWE correspondingly low, with no error, no warning and no
NaN. A ceiling guard exists at `φ → 1`; no floor guard exists at `CEC → 0`.

**As-built.** `ABSENT` — `bound_water_multiplier` returns `0.0` for `cec ≤ 0` at `multimin2.rs:617`;
default source `PorositySource::Cec` at `:136`–`:143`; the offending row is
`clay("Clay", 0.00, 0.120, …)` at `:2098`. The upper guard that *does* exist is
`WCP_PHYSICAL_CEILING = 0.5` at `:784`.

**Verified by.** SB-MIN-T07

#### SB-MIN-008 — Ship `CEC` and `WCLP` only as a matched pair from one library [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A clay row that carries both `CEC` and `WCLP` MUST take both from the same source
library, and the shipped library MUST satisfy
`|k_CEC(α = 1, T = 64.4 °C) − WCLP/(1 − WCLP)| / k_CEC ≤ 0.02` for every clay. A row that cannot
satisfy this MUST ship one parameterisation only, with the other marked `ABSENT — ships with no
default`.

**Rationale.** F-22 (T1 Techlog `QElan_PostProcess_Using_Conductivities.py`; T3 Geolog RF04 6.2).
`CEC` and `WCLP` are two parameterisations of one quantity, `k = V_bw / V_dryclay`. Each vendor's own
pair is self-consistent to **≤1.5 %** (Geolog's Illite pair to **0.01 %**). SandiBumi ships Geolog's
CEC column against Techlog's WCLP column, and the two routes then disagree by **+58.6 % on Illite**
(0.18411 vs 0.11607), −15.2 % on Glauconite and +12.7 % on Kaolinite — while agreeing to −0.6 % on
Chlorite, the one clay where both vendors happen to ship `CEC = 0.15`. **The divergence appears
exactly where the vendors' CEC values differ and vanishes where they agree**, which is what makes this
a chimera rather than a rounding artefact. At `V_dryclay = 0.25` it is **1.70 pu of PHIE that moves
with a radio button and no physics.**

**As-built.** `PRESENT-DIVERGENT` — `multimin2.rs:2093`–`:2098`; route selection in
`bound_water_multiplier` `:616`–`:630`; attribution comment `:2059`–`:2060`.

**Verified by.** SB-MIN-T08

#### SB-MIN-009 — Carry provenance on every endpoint value, not every endpoint column [P0] [status: ABSENT]

**Requirement.** Every value in the shipped endpoint library MUST carry a machine-readable source
string identifying the primary reference or the vendor library it came from, and the library MUST NOT
mix sources **within a row** for parameters that form a physical pair. A value with no traceable
source MUST be marked `ABSENT — ships with no default` rather than shipped.

**Rationale.** `IP_PROVENANCE.md` §2.2 already records the 27-row `LIB` as *"merged from two vendor
installs, in Interactive Petrophysics' mineral-dropdown order"*, rates the exposure Medium, marks it
blocking before first sale, and names the fix — *"Add a primary-literature citation per row where one
exists (Schön, Ellis & Singer, SPWLA references) … This is work SandiBumi should do anyway and it is
currently unscheduled."* SB-MIN-008 sharpens it: **per-column provenance is not sufficient**, because
two correctly-cited columns still produce a wrong answer when they are a matched pair. F-14 supplies
the scale of the underlying disagreement — the three vendor libraries agree three-way on RHOB for all
12 clean matrix minerals and diverge on 13 of 18 NPHI values, 12 of 18 U values and 12 of 18 SIGMA
values. Those are library-provenance differences, not bugs, and no incumbent surfaces them.
CONTRACT §5.4.

**Discharges.** `SB-CORE-005` (*"Every endpoint in the mineral library MUST cite a primary source.
Rows that cannot be re-sourced MUST be marked as vendor-derived in the UI and in the deliverable"* —
owning chapter `13_mineral-solver.md`, i.e. this one) and `SB-CORE-004` (no parameter ships without a
machine-readable source string; **the build fails otherwise**). This chapter's §5 is where both are
actually discharged, row by row: every §5 row carries a `Source` cell, and a row whose source is a
vendor install rather than primary literature is labelled `VENDOR-DERIVED` there so the UI and the
deliverable can inherit that label rather than infer it.

**As-built.** `ABSENT` — `LibRow` (`multimin2.rs:2053`–`:2065`) has no source field; units are stated
once in a block comment (`:2044`–`:2046`) and nowhere in the type. The generic `Clay` row's
`wcp = 0.120` (`:2098`) matches no vendor file examined in this work.

**Verified by.** SB-MIN-T09

#### SB-MIN-010 — Declare the wet/dry clay convention on every clay row and every clay curve [P0] [status: ABSENT]

**Requirement.** Every clay endpoint row MUST declare whether its properties are wet-clay or dry-clay
values; SandiBumi MUST refuse to solve a model that mixes conventions without an explicit conversion;
and every emitted clay-volume curve MUST carry the same declaration in its metadata.

**Rationale.** F-4 (T2-equivalent Elan Eqs 10/11/11-1a/11-1b/12) — the highest-frequency
silent-wrongness site in this domain. A wet-clay density used where a dry-clay density is required is
a **5.7 σ** density residual and up to **6.9 pu** of porosity. Elan solves in wet-clay volumes
internally and reports dry-clay volumes plus separate bound water; the conversion is real, documented
and one line, and the failure is invisible without a declaration. The shipped library already mixes
the two: Illite's `RHOB 2.78` is a dry-clay value, Chlorite's `2.81` follows IP's wet-clay convention.

**As-built.** `ABSENT` — no convention field on `LibRow` (`multimin2.rs:2053`–`:2065`); the emitted
`VSH` is `Σ(clays) + Σ(U bound water)` (`:1681`–`:1682`), a wet-clay quantity carrying no declaration.

**Verified by.** SB-MIN-T10, SB-MIN-T24

#### SB-MIN-011 — Declare the CEC unit and refuse implausible magnitudes [P0] [status: ABSENT]

**Requirement.** The `CEC` field MUST carry its unit (`meq/g`) in the type or schema, not only in a
comment. SandiBumi MUST reject a clay CEC outside `[0.01, 2.0] meq/g` with a message naming the
likely unit error, and MUST additionally **warn** below `0.05 meq/g` — the dossier's own T-18
threshold, justified against the shipped library floor of `0.10 meq/g` (Kaolinite), the lowest
clay-mineral CEC in any of the three libraries. Neither the message nor the fixture may name a client
project or carry a client core-analysis range.

**Rationale.** The `meq/g` vs `meq/100 g` confusion is a factor of 100 that computes cleanly and plots.
F-14 records that CEC agrees three-way on only **1 of 4** comparable clays, so a user *will* type a
value from a paper, and papers use both units. The bound is set from the shipped library's own span —
the smallest live clay CEC is 0.10 (Kaolinite), the largest is 1.0 (Montmorillonite) — with a decade
of headroom either side, so it excludes unit errors without excluding real smectitic clays.

**As-built.** `ABSENT` — `cec: f64` with the unit in prose only (`multimin2.rs:65`, `:603`); no range
check anywhere in `bound_water_multiplier` (`:616`–`:630`).

**Verified by.** SB-MIN-T11

#### SB-MIN-012 — Mix photoelectric response volumetrically on U, never on Pe [P0] [status: PRESENT-OK]

**Requirement.** Any linear mixing of photoelectric response over component volumes MUST be performed
on the volumetric cross-section `U = Pe · ρe`, with `ρe = (ρb + 0.1883)/1.0704`. A measured Pe reading
MUST be converted to U before entering the linear system, and a reconstructed U MUST be converted back
to Pe for display. SandiBumi MUST NOT mix Pe values linearly by volume.

**Rationale.** F-20 (T2 IP E26, printed twice; T2-equivalent Elan Eq 16; T3 Geolog §I/§J). Pe is a
**per-electron** cross-section; only the volumetric form is additive. All three incumbents mix U, and
IP's two printed constants `(ρb + 0.1883)` and `0.93423` are exactly Geolog's density chain inverted
(`1/1.0704 = 0.934230…`, `188.3 kg/m³ = 0.1883 g/cc`) — three-way agreement on the physics. The
magnitude is this repository's own worked example: 50 % quartz + 50 % water gives **1.085 b/e** under
linear Pe mixing against **1.382 b/e** correctly — a **0.30 b/e** systematic residual, **exactly 1.0×
the legacy default `SIG_PEF` of 0.30** — biasing clay volume upward. P0 as a regression guard: this is
correct now and it was not always (§3.6).

**As-built.** `PRESENT-OK` — `rho_e` `multimin2.rs:1786`–`:1788`; measurement converted at `:1359`;
tool tagged `TKind::Pef` at `:1120` (enum `:1800`, key test `is_pef_key` `:1779`); display inverted in
`recon_display` `:1767`–`:1770`.

**Verified by.** SB-MIN-T12

### 4.3 Misfit, conditioning and diagnostics

#### SB-MIN-013 — Define `RECON` against a stated equation and pin it with a test [P1] [status: PRESENT-UNVERIFIED]

**Requirement.** The reconstruction statistic MUST be documented as the σ-weighted RMS residual over
live tool rows, MUST be computed after any post-solve saturation redistribution, and MUST be pinned by
a test that reproduces it from an independent long-form computation.

**Rationale.** F-3: the misfit statistic is not portable between products, so SandiBumi's own must at
least be unambiguous. Elan's incoherence (Eq 79) and its `SDR` (Eq 80) differ only by a
`LargestWeight` factor that cancels exactly; a refactor reintroducing `LW` asymmetrically would change
every shipped `RECON` and fail no test. Computing `RECON` **after** the Sw redistribution is a
deliberate and good choice — it makes the statistic measure the coherence of the saturation model too
— and it should be protected explicitly rather than by accident.

**As-built.** `PRESENT-UNVERIFIED` — computed `multimin2.rs:1548`–`:1574`, emitted `:1685`, described
as Quanti.Elan Eq 79 at `:1684`, ordered after redistribution at `:1419`–`:1420`, per-well mean at
`:1743`. No equivalence test exists.

**Verified by.** SB-MIN-T13

#### SB-MIN-014 — Emit IP-comparable and Geolog-comparable misfit statistics beside `RECON` [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST additionally emit an un-normalised total error comparable to IP's
`Total_err` and a chi-square-normalised quality comparable to Geolog's
`QUALITY = sqrt(Δ²/χ²₉₅(n_tool − 3))`, each labelled with the convention that produced it
(SB-MIN-004).

**Rationale.** F-3. The three products' misfit numbers are not interconvertible — IP's is
un-normalised, Elan's is σ-weighted RMS, Geolog's is chi-square-normalised against a 95 % critical
value. A user validating a migration cannot compare a `RECON` to a `Total_err`. Emitting all three,
labelled, is cheap and no incumbent does it.

**As-built.** `ABSENT` — `grep` over `multimin2.rs` returns zero occurrences of `toterr`, `TOTERR` and
`QUALITY`.

**Verified by.** SB-MIN-T14

#### SB-MIN-015 — Report a conditioning number and refuse to present an unstable solve as trusted [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST compute `CONDNUM = log₁₀` of the norm ratio of the weighted normal
matrix `PᵀUP` per solved sample, MUST emit it as a curve, and MUST flag samples above a configurable
threshold (default 10) as untrusted. A singular solve MUST be reported, not silently skipped.

**Rationale.** F-13 (T3 Geolog §H): Geolog reports `CONDNUM` with **>8 suspect, >10 unstable** and a
default linear cutoff of 10. **Neither IP nor Elan diagnoses an ill-conditioned design matrix at
all**, and Elan's own manual concedes the consequence without the diagnostic — an underdetermined
carbonate model producing *"a very geologically questionable 50 % dolomite and 50 % clay"*, because
*"The ELANPlus program believes what you tell it."* The dangerous case is not the singular matrix,
which fails; it is the ill-conditioned but invertible one, which returns a confident wrong answer.
CONTRACT §5.1 and §5.3 both point here, and it is the cheapest real improvement available in this
domain.

**As-built.** `ABSENT` — no conditioning diagnostic; singularity surfaces only as `solve_linear_opt`
returning `None` (`multimin2.rs:1927`–`:1929`, `:2009`), after which the sample is dropped and the
loop continues (`:1401`).

**Verified by.** SB-MIN-T15

#### SB-MIN-016 — Report degrees of freedom and refuse to let a zero-DOF fit read as validation [P0] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST report `dof = n_liveTools + n_constraintRows − n_components` on every
run, MUST attach an explicit note when `dof ≤ 0` stating that the reconstruction cannot discriminate
the model, and MUST fail with a named, countable error when there are too few live tools to solve.

**Rationale.** F-13/F-7. At `dof = 0` the solve reproduces the measurements exactly whatever the
endpoints are, so `RECON ≈ 0` regardless of whether the model is right — the reconstruction becomes an
argument *for* a wrong answer. **Neither IP nor Elan ships a DOF check at all.** P0 as a regression
guard: this already works, it is a genuine differentiator, and it is exactly the kind of quiet
safeguard a refactor removes.

**As-built.** `PRESENT-OK` — `n_extra` `multimin2.rs:1195`; `dof` `:1204`; `dof_note` `:516`–`:518`,
`:1205`; too-few-tools error `:1197`–`:1202`. Pinned by
`an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so` (`:2508`) with an
over-determined control at `dof = 2` (`:2412`).

**Verified by.** SB-MIN-T16

### 4.4 Tool weighting and uncertainty

#### SB-MIN-017 — Separate "tool off" from "tool weighted to zero" [P1] [status: ABSENT]

**Requirement.** Every tool row MUST carry an `active` flag that removes the equation from the solved
system entirely, distinct from any weight or uncertainty setting. Setting a weight to zero or an
uncertainty to a very large value MUST NOT be presented in the UI as turning a tool off, and the
reported degree-of-freedom count MUST reflect only active rows.

**Rationale.** F-8 (T2-equivalent, Techlog `-default-uncertainties.html`, verbatim): *"Truthfully, one
can never completely turn the tool off with uncertainties. To be totally out of the solution, the
equation must be removed from the model (Solve process)."* A weight of zero and an absent equation are
different objects — the row still occupies a degree of freedom and still perturbs the conditioning of
`PᵀUP`. All three vendors ship the structural off-switch (IP's `Use` checkbox, Geolog's
`Active Log = No`, Elan's "remove from the Solve process"); the multiplier is only an attenuator.
Without the distinction, `dof` (SB-MIN-016) and `CONDNUM` (SB-MIN-015) both report on rows the user
believes are gone.

**As-built.** `ABSENT` — a tool is present or absent from `req.tools` (`ToolSpec`,
`multimin2.rs:87`–`:91`); there is no active flag and no weight field separate from `sigma`.

**Verified by.** SB-MIN-T17

#### SB-MIN-018 — Provide a per-tool weight multiplier separate from the tool uncertainty [P3] [status: ABSENT]

**Requirement.** Each tool row SHOULD accept a dimensionless weight multiplier, defaulting to 1.0,
applied on top of `1/σ` and stored separately from `σ`. The UI MUST state that a multiplier of 1.0
means the tool influences the answer as strongly as the volume-summation row.

**Rationale.** F-8. Elan is the only vendor with a second multiplier at all — a zonable `xxxx_WM`,
documented as *"a multiplier value of 1.0 means that the tool will influence the answer as strongly as
the Volume Summation tool"*, with *"dividing the multiplier by four is close to having the tool
ignored"*. Folding an interpreter's judgement into `σ` destroys the record of what the tool's
uncertainty actually was, which is the input SB-MIN-032 has to persist.

**As-built.** `ABSENT` — `ToolSpec` carries `key`, `curve`, `sigma` only (`multimin2.rs:87`–`:91`);
the row weight is `1/σ` with nothing else in it (`:1329`).

**Verified by.** SB-MIN-T18

#### SB-MIN-019 — Store a tool's MIN, MAX and printed default uncertainty as three independent fields [P2] [status: ABSENT]

**Requirement.** The default-uncertainty library MUST store each tool's normal logged MIN, MAX and its
printed default uncertainty as separate stored values. SandiBumi MUST NOT derive a shipped default
from MIN and MAX at runtime; where a printed default exists it MUST win.

**Rationale.** F-18 (T2-equivalent Elan Tables 29/30; T3 Geolog §E/§L). Both vendors independently
document the same generating rule — *"default = ~1.5 % of the tool's normal logged range"* — and Elan's
own table reproduces `1.5 % × (MAX − MIN)` exactly on **9 of 14** rows and deviates on **5** (DT, CUDC,
CXDC, TPL, VELC) by 4–7 %, plus a sixth on Table 30's `SDPT`. The deviating rows' implied ranges (150
µs/ft, 40.0 ns/m, 33.3) are tidier than the tabulated MIN/MAX, i.e. hand-adjusted. **A product that
derives the default from the range silently disagrees with the vendor on six rows.**

**As-built.** `ABSENT` — no uncertainty library exists; `sigma` is supplied per run per tool
(`multimin2.rs:87`–`:91`), and the conductivity rows are the only ones with an automatic default
(`u_ct`, `u_cxo`, `:597`–`:598`).

**Verified by.** SB-MIN-T19

#### SB-MIN-020 — Ship a default tool-uncertainty library with per-value sources [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST ship a default uncertainty for each of the 14 supported tool keys,
each carrying its source, and MUST label any value derived by the 1.5 %-of-range rule as derived
rather than cited.

**Rationale.** F-18. Two independent vendors documenting the same rule is the strongest corroboration
available for a default in this domain, and it is a rule rather than a table, so it can be applied to
a tool neither vendor tabulates — provided the derivation is labelled. IP states no such rule; its
`MINEQDEF.PAR` numbers are simply tabulated, which is why an IP-parity fixture cannot be built from
the rule alone. Requiring the label is what keeps SB-MIN-009's provenance chain intact through a
derived value.

**As-built.** `ABSENT` — every `sigma` is caller-supplied.

**Verified by.** SB-MIN-T19, SB-MIN-T20

### 4.5 Saturation-adjacent inputs owned here

#### SB-MIN-021 — Make the conductivity root exponent an explicit, recorded model input [P1] [status: PARTIAL]

**Requirement.** The exponent applied to a conductivity tool row and its uncertainty before the solve
MUST be an explicit named model input with the value actually used recorded in the run record.
SandiBumi MUST support at minimum `1/2` (Elan), `1/w` with `w = 0.75m + 0.25n` (Geolog) and `1/m`, and
MUST NOT silently pick one.

**Rationale.** F-9 / proposed ledger item CT-3 (T2, both readings quoted verbatim from **both** IP
editions). IP's manual states the transform of `Crv`, `Crv_Rec` and `Crv_Tol` two ways — *"square root
taken of them"* and *"has 1/m th root taken of it"* — describing the same object, and its worked
example is printed at `m = 2` where `1/m = ½` exactly, so **the vendor's own example cannot
arbitrate**. Quantified at `m = 2.2, n = 1.6`: Elan 0.500, Geolog 0.4878, IP either 0.500 or 0.4545 —
**a 9.1 % spread inside IP alone, larger than the IP-vs-Geolog gap.** At `m = 2.0, n = 1.6` the
Geolog-vs-IP gap is 5.3 % and survives either reading. A mis-set conductivity exponent computes, plots
and ships.

**As-built.** `PARTIAL` — Geolog's `w = 0.75m + 0.25n` is implemented with a guard to 2.0 for
non-finite or `≤ 0.5` (`multimin2.rs:574`–`:576`) and applied as `^(1/w)` to both the row and its
auto-uncertainty (`:597`–`:598`, `recon_display` `:1760`–`:1766`). It is the only supported convention
and it is not recorded as a choice.

**Verified by.** SB-MIN-T21

#### SB-MIN-022 — Ship no default for the Shell porosity-dependent cementation constant [P2] [status: ABSENT]

**Requirement.** If the Shell porosity-dependent `m` relation is offered, its constant MUST ship
`ABSENT — ships with no default`, MUST require an explicit user value, and the UI MUST display the
three competing published values with their sources.

**Rationale.** F-10 / ledger D-10 (T2 IP 2025 raster 0.018 verified 4×, IP 2025 ASCII 0.019, IP 2018
raster 0.018 verified 6×, IP 2018 ASCII 0.019; T2-equivalent Techlog Eq 78 `mc2`, *"the usual value
assumed for mc2 is 0.19"*, Table 28 default 0.0). The same named constant appears as **0.018, 0.019
and 0.19** across two vendors, and Elan never states whether the `φₑ` inside `mc2/φₑ` is fractional or
p.u. Quantified: at `φe = 0.10`, `m` is 2.05 (0.018) against 2.06 (0.019); at `φe = 0.02`, 2.77
against 2.82 — a ~5 % exponent difference propagating to ~5–10 % Sw error in tight rock. CONTRACT §2:
where vendors disagree and no adjudication is defensible, the parameter ships absent with the
competing values in the body.

**As-built.** `ABSENT` — no Shell `m` relation is implemented; `m` enters only through
`w = 0.75m + 0.25n` (`multimin2.rs:575`) and the `SAT` chapter's Sw models.

**Verified by.** SB-MIN-T22

#### SB-MIN-023 — Implement variable `m*` with the corroborated coefficient set [P2] [status: ABSENT]

**Requirement.** If a variable cementation exponent is offered, the Dual-Water form MUST be
`m* = m + Cm(0.258·Y + 0.2(1 − e^(−16.4·Y)))` with `Y = Qv·φT/(1 − φT)`. The Waxman-Smits variant
`m* = m + Cm(1.128·Y + 0.22(1 − e^(−17.3·Y)))` MAY be offered and MUST be labelled single-sourced.

**Rationale.** F-19 (T2 IP E68; T2-equivalent Elan Eq 65/66). **The coefficients 0.258, 0.2 and 16.4
are bit-identical in Interactive Petrophysics and Schlumberger ELAN** — the strongest single
corroboration found anywhere in this domain, and the base `m` default is the only thing that differs
(IP 2.0, Elan `mdw` 1.8). The Waxman-Smits variant appears in IP alone with no second source, so it
carries a different confidence and must say so.

**As-built.** `ABSENT`.

**Verified by.** SB-MIN-T23

### 4.6 Clay density bookkeeping

#### SB-MIN-024 — Convert wet↔dry clay with an explicit bound-water density, not a hard-coded 1.0 [P2] [status: PRESENT-DIVERGENT]

**Requirement.** The wet↔dry clay conversion MUST take the bound-water density and bound-water sonic
transit time as named parameters with defaults that carry sources. Hard-coded values MUST NOT appear
in the conversion.

**Rationale.** F-4 and the dossier's §2.6 comparison: Geolog's generalised conversion, evaluated
against the fixed-`ρ_bw = 1.0` form, differs by **0.010 g/cc** on the dry-clay density — small,
systematic, and directly in the `RHOB` row that carries the tightest σ of any tool. Bound water in a
smectitic clay at elevated temperature is not 1.000 g/cc, and the parameter is exactly the kind of
buried constant SB-MIN-009's provenance chain exists to eliminate.

**As-built.** `PRESENT-DIVERGENT` — `dry_clay_calc` declares `const RHO_W: f64 = 1.0;` and
`const DT_W: f64 = 189.0;` inside the function (`multimin2.rs:676`–`:699`); neither is reachable from
the request.

**Verified by.** SB-MIN-T24

### 4.7 Model inputs and units that silently change numbers

#### SB-MIN-025 — Support a per-equation invasion factor [P3] [status: ABSENT]

**Requirement.** Each tool equation SHOULD accept an invasion factor in `[0, 1]` selecting the fluid
mix that equation reads, defaulting to the component's categorical zone assignment. Where a factor is
non-default it MUST be recorded per equation in the run record.

**Rationale.** IP models invasion as a continuous per-equation factor and names the neutron as the
common case for `IF < 1.0`; Geolog and Elan assign fluids categorically. SandiBumi has adopted the
categorical model exclusively, so a model needing the neutron to read a mixture of invaded and virgin
fluid cannot be expressed at all — a capability gap, not a numerical divergence, but one that appears
on the first shallow-invasion gas sand a migrating IP user brings across.

**As-built.** `ABSENT` — `classify` assigns X / U / shared categorically (`multimin2.rs:801`–`:867`);
`grep` finds `invasion` only in prose (`:455`, `:584`, `:1054`).

**Verified by.** SB-MIN-T25

#### SB-MIN-026 — Make the neutron response set a named, recorded model input [P2] [status: ABSENT]

**Requirement.** The neutron tool type / response set used by a solve MUST be an explicit model input
recorded in the run record. It MUST NOT be inherited from a well-header field, and any cross-tool
parity fixture MUST record it.

**Rationale.** F-11 (T2, `O_db_config_infra.md` §3.3, verbatim): in IP *"the Logging Contractor field
on the Default Parameters tab … sets the Neutron Tool Type for Basic Log Analysis and Mineral Solver
… A single header dropdown therefore silently changes numerical results."* Two IP runs with identical
models, identical endpoints and identical curves can return different volumes. **Any IP-parity fixture
is under-specified unless it records the Logging Contractor**, which makes this a prerequisite for
every comparison test in §6 that claims IP parity, not merely a feature. CONTRACT §5.3.

**As-built.** `ABSENT` — the neutron endpoint is a plain `NPHI` column in the endpoint map
(`multimin2.rs:2050`, `:63`) with no tool-type concept.

**Verified by.** SB-MIN-T26

#### SB-MIN-027 — Store WCLP in v/v and refuse a p.u. value instead of switching route [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Wet-clay porosity MUST be stored, entered and displayed in v/v (m³/m³), with the unit
declared in the type or schema. A supplied WCLP at or above the physical ceiling MUST fail with a
message naming the likely p.u. entry. SandiBumi MUST NOT silently fall back to a different
bound-water route when a WCLP value is rejected.

**Rationale.** The dossier's escalation T-2 could reach only a *leaning* on whether a vendor's wet-clay
porosity is p.u. or fractional. **Techlog's own executable source settles the WCLP half** (T1):
`QElan_PostProcess_Using_Conductivities.py` declares `WCLP_*_unit = u"m3/m3"` beside description
`'Wet Clay Porosity'` and group `'Clay CEC '`, alongside `CEC_*_unit = u"meq/g"`. The failure this
guards is silent and specific: a user entering `10.4` in the belief that WCLP is in p.u. hits the
ceiling branch and **the code switches to the CEC route without saying so**, returning a plausible
number computed by a method the user did not select — the same silent-substitution class as
SB-MIN-007. This resolves the WCLP half of T-2 only; **it does not close ledger item D-10**, which
turns on the separate `φₑ` symbol inside Elan Eq 78's `mc2/φₑ` and on which no evidence gathered here
bears.

**As-built.** `PRESENT-DIVERGENT` — `bound_water_multiplier` `multimin2.rs:616`–`:630`, ceiling
`WCP_PHYSICAL_CEILING = 0.5` at `:784`, silent route fallback at `:623`–`:624`. The unit appears in no
type; the attribution comment at `:2059`–`:2060` names the file but not the unit.

**Verified by.** SB-MIN-T27

#### SB-MIN-028 — Offer a named endpoint library and surface the inter-library disagreement [P2] [status: ABSENT]

**Requirement.** The endpoint library in use MUST be selectable by name and recorded with the run.
Where two supported libraries disagree on a value by more than 5 % relative, the UI MUST show the
competing values and their sources at the point of selection.

**Rationale.** F-14 (T3 `E_threeway_endpoint_compare.json`, agreement criterion "relative spread
≤ 5 %"). The three libraries agree three-way on RHOB for **all 12** clean non-clay matrix minerals and
diverge on **13 of 18** NPHI values, **12 of 18** U values, **12 of 18** SIGMA values and **17 of 18**
GR values — the last partly structural, since IP's `MINDEF.PAR` has no GR column and computes it at
runtime from K/Th/U weight fractions. These are library-provenance differences, not bugs. **The
disagreement is itself information the interpreter needs, and no incumbent surfaces it** — CONTRACT
§5.2 exactly.

**As-built.** `ABSENT` — one merged library, `LIB` `multimin2.rs:2079`–`:2108`, exposed through
`multimin_library()` (`:2110`); no selector, no alternative.

**Verified by.** SB-MIN-T28

#### SB-MIN-029 — Declare the fluid sonic endpoint source at the point of use [P2] [status: PRESENT-DIVERGENT]

**Requirement.** The oil and gas sonic endpoints MUST display their source library at the point of
use, and where the supported libraries differ by more than the tool's own uncertainty the UI MUST show
the alternatives.

**Rationale.** F-5: the vendors' gas sonic endpoints differ by **45 µs/ft**, contributing ≈5.4 µs/ft to
a solved row at plausible gas volumes — **1.8× IP's own default sonic confidence.** A divergence larger
than the tool's uncertainty is not a preference; it is a decision, and it is currently made silently on
the user's behalf.

**As-built.** `PRESENT-DIVERGENT` — `Oil Sxo`/`Oil Sw` DT 189.0 and `Gas Sxo`/`Gas Sw` DT 250.0
(`multimin2.rs:2104`–`:2107`) are Geolog RF04 6.2 values, shipped without attribution in the UI.

**Verified by.** SB-MIN-T28, SB-MIN-T29

#### SB-MIN-030 — Treat silt as a first-class term and never merge two different Simandoux equations under one label [P3] [status: ABSENT]

**Requirement.** If the Elan-form Simandoux is offered, `V_silt` MUST appear as a first-class model
term in both the numerator exponent group and the `1 − (V_cl + V_silt)^(swshe+1)` denominator, with
`ersh` and `swshe` as named parameters. An equation labelled "Simandoux" MUST state which form it is,
and the compact form MUST NOT be presented as equivalent to the Worthington Type-2 form.

**Rationale.** F-17 (T2-equivalent Elan Eq 78 and Table 28; T1 Techlog `QM_MineralTable.xml` `Silt`
row). **IP's E63/E64 "Simandoux" and Elan's Eq 78 "Simandoux" are different equations with the same
name** — Elan's is a documented Worthington Type-2 with a full citation chain, IP's is the compact
form with no citation. Elan's Table 28 defaults `ersh = 1.0` and `swshe = 0.5` correspond to
Worthington's `x = 1.0`, `c = 1.5`, and Elan states they *"essentially assume that by default in
ELANPlus the silt behaves in the same manner as clay in relation to the conductivity"*. Only Techlog
ships a silt endpoint at all — neither IP's `MINDEF.PAR` nor Geolog's RF04 6.2 carries one.

**As-built.** `ABSENT` — `SwModel::Simandoux` (`multimin2.rs:103`–`:123`) is the compact form and
carries no silt term; no silt row exists in `LIB`.

**Verified by.** SB-MIN-T30

#### SB-MIN-031 — Allow a per-clay shale resistivity where the saturation model uses one [P3] [status: ABSENT]

**Requirement.** A clay component SHOULD accept its own shale resistivity, and a multi-clay saturation
model that consumes one MUST use the per-clay value where supplied, falling back to the zonal value
otherwise.

**Rationale.** F-16 (T1 Techlog `QM_MineralTable.xml`: Illite `Rsh` **3**, Chlorite **5**, Shale
**5**, Kaolinite **7** ohm-m; `XWater` 0.03). IP takes `Rcl` as a **zonal** parameter inside Simandoux
(E63/E64); Geolog carries no per-clay resistivity at all. A Techlog multi-clay model can therefore
give illite and kaolinite different shale resistivities **in one zone** and IP's cannot. **This is a
modelling-capability difference, not a parameter difference** — the ratio across Techlog's own rows is
7:3, well outside any plausible zonal average.

**As-built.** `ABSENT` — `Component` (`multimin2.rs:52`–`:76`) has no resistivity field; the endpoint
map covers the 14 `TOOL_KEYS` only (`:2050`).

**Verified by.** SB-MIN-T31

#### SB-MIN-032 — Persist the fully resolved parameter set with every run [P1] [status: ABSENT]

**Requirement.** Every solve MUST persist, with its outputs, the complete resolved input set actually
used: every component endpoint value, every `σ`, the porosity source, the conductivity exponent, the
zone assignments, the temperature and salinity, and the source string of every parameter that carries
one (SB-MIN-009). Re-running from the persisted set MUST reproduce the outputs bit-for-bit.

**Rationale.** CONTRACT §5.4: *a parameter that carries the paper it came from, through the
computation, into the deliverable, is a claim no incumbent can make.* It is also the only mechanism
that makes SB-MIN-002, SB-MIN-021, SB-MIN-026 and SB-MIN-028 auditable rather than merely displayed,
and the only way a client's third-party reviewer can reproduce a number two years later. The
prerequisite already exists structurally: physics defaults are single-sourced in Rust and the dialog
edits a working copy (`multiminDialog.ts:40`), so there is exactly one place the resolved set can be
captured.

**As-built.** `ABSENT` — `MultiminResult` (`multimin2.rs:495`–`:523`) carries output names, per-well
`mean_recon`, `dof`, `dof_note` and an optional error; the curves are versioned into a log set
(`:1705`, `:1725`) with no record of the inputs that produced them.

**Verified by.** SB-MIN-T32

### 4.8 Constraint completeness and infeasibility

#### SB-MIN-033 — Name the conflicting rows when a constraint set is infeasible [P1] [status: ABSENT]

**Requirement.** When no feasible point exists, SandiBumi MUST fail with a message naming the specific
constraint rows in conflict and the depth at which the conflict occurs. It MUST NOT return a
best-effort solution from a relaxed set without saying which rows were relaxed.

**Rationale.** F-13: Geolog ships an explicit conflict check alongside its DOF check and `CONDNUM`;
IP and Elan ship neither. A constraint conflict is the one solver failure whose cause is fully known
to the solver at the moment it happens — the information is free, and discarding it turns a
five-second fix into an afternoon. CONTRACT §5.3.

**As-built.** `ABSENT` — an infeasible or singular system surfaces as `solve_linear_opt` returning
`None` (`multimin2.rs:1927`–`:1929`, `:2009`), after which the sample is skipped and the loop
continues (`:1401`). The only structured pre-solve refusal is the too-few-tools error
(`:1197`–`:1202`).

**Verified by.** SB-MIN-T33

#### SB-MIN-034 — Impose the water-mud constraint as an inequality, iterated to feasibility [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The water-based-mud constraint MUST be imposed as the inequality
`Σ(X waters) − Σ(U waters) ≥ 0`, not as an equality. The correction MUST iterate until no constraint
is violated or an iteration cap is reached, and reaching the cap MUST be reported.

**Rationale.** Dossier Δ-2 and escalation E-8, extended by this chapter's own reading of the code. The
physical statement is *invasion implies `Sxo ≥ Sw`* — the module's own doc says exactly that
(`multimin2.rs:455`). SandiBumi's re-solve row drives the difference toward **zero**, i.e. toward
`Sxo = Sw`, which is the *boundary* of the admissible region rather than its interior. A hard
inequality parks at the boundary only when the data demand it; a soft equality pulls through it and
**suppresses genuine movable hydrocarbon** — which lands directly in the shipped `MOVEDHC` curve
(`multimin2.rs:1677`). The single-pass structure compounds it: a re-solve that re-violates is not
detected, which is precisely the oscillation IP's own "add one limit at a time, in grid order" rule
exists to prevent.

**As-built.** `PRESENT-DIVERGENT` — violation test `if s < −1e-6` at `multimin2.rs:1406`; the appended
row is built as `a2.push(wm.iter().map(|e| e * soft_weight).collect()); b2.push(0.0);` at `:1409`–
`:1410`, i.e. a σ-weighted **equality** at RHS 0; no loop, no re-check, no report.

**Verified by.** SB-MIN-T34

#### SB-MIN-035 — Impose `Tool` constraints as hard equality plus pseudo-measurement, and emit the tie residual [P1] [status: PRESENT-DIVERGENT]

**Requirement.** A `Tool`-class constraint MUST be imposed as a hard equality **and** as a σ-weighted
pseudo-measurement contributing to the misfit statistic and to the degree-of-freedom count. Where only
the soft half is imposed, SandiBumi MUST emit the tie residual as a QC curve so the violation is
visible.

**Rationale.** F-7 (T3 Geolog, verbatim *"only `Tool` rows add a degree of freedom"*) and dossier Δ-1.
Geolog's `Tool` class is deliberately two things at once; SandiBumi implements the second half only,
and its own module header says so (`multimin2.rs:17`–`:19`). The soft-only form never reports
infeasible where Geolog would, which sounds forgiving and is the problem: **a porosity or bound-water
tie can be violated by many multiples of its σ and nothing surfaces it.** The exposure grows with the
user's own setting — `σ` is overridable at `req.sigma_constraint` (`:1169`), so relaxing it for
convergence silently relaxes the porosity tie too. The tie residual is already computable from
quantities the solve holds; emitting it costs one curve.

**As-built.** `PRESENT-DIVERGENT` — POROSITY soft row `multimin2.rs:1146`–`:1158`; `bndwat_soft_rows`
`:924`–`:968`; `SIGMA_CONSTRAINT = 0.01` `:768`; weight `1/σ` `:1169`–`:1170`. No hard half, no tie
residual, no QC curve.

**Verified by.** SB-MIN-T35

### 4.9 Outputs, uncertainty and reproducibility

#### SB-MIN-036 — Complete the output nomenclature and declare each curve's convention [P2] [status: PARTIAL]

**Requirement.** SandiBumi MUST emit `SXOE` alongside `SXOT`, and `PHIE_X` / `PHIT_X` alongside `PHIE`
/ `PHIT`. It MUST NOT emit a bare `SW`. Every emitted clay or shale volume curve MUST declare its
wet/dry convention in metadata (SB-MIN-010).

**Rationale.** Adoption-spec F-11 names all of `SXOE`, `PHIE_X` and `PHIT_X`; ledger item D-15 forbids
the bare `SW`. `SXOT` shipping without its effective counterpart is an asymmetry a user meets on the
first crossplot, and a `VSH` that is silently `Σ(clays) + Σ(U bound water)` — a **wet**-clay quantity —
is exactly the F-4 convention trap propagating out of the solver into every downstream cutoff and pay
summary.

**As-built.** `PARTIAL` — emitted: `VOL_*` (`multimin2.rs:1215`), `PHIE`/`PHIT`/`SWE`/`SWT`
(`:1664`–`:1667`), `SXOT` (`:1674`), `MOVEDHC` (`:1677`), `VSH` (`:1681`–`:1682`), `RECON` (`:1685`).
`SXOE`, `PHIE_X` and `PHIT_X` are absent; no bare `SW` is emitted anywhere, so D-15 is already
satisfied.

**Verified by.** SB-MIN-T37

#### SB-MIN-037 — Propagate endpoint uncertainty by Monte Carlo with a recorded seed [P3] [status: ABSENT]

**Requirement.** If endpoint-uncertainty propagation is offered, the pseudo-random seed MUST be a
user-settable model input recorded with the run, and re-running with the same seed and inputs MUST
reproduce the result bit-for-bit. SandiBumi MUST NOT seed from wall-clock time.

**Rationale.** F-12 (T2 `D_cutoffs_montecarlo.md` §2.7/§3.7/§3.8/§5.8/§5.9). IP is the **only** vendor
that propagates endpoint uncertainty through the solve — endpoint shifts defaulting to ±10 % of the
endpoint value, Gaussian, 2000 iterations, with a dependency-correlation matrix — **and it is not
reproducible**: *"IP uses a random number generator, seeded through the CPU clock time"*, with no
user-settable seed documented anywhere. IP's substitute is to rank iterations, pick a percentile and
reload that iteration's saved parameters, which is a **replay** mechanism, not reproducibility. A
client deliverable whose uncertainty band cannot be regenerated two years later is not defensible, and
fixing it costs one integer. CONTRACT §5.1.

**As-built.** `ABSENT` — `grep` over `multimin2.rs` returns zero occurrences of `monte` and `seed`.

**Verified by.** SB-MIN-T38

#### SB-MIN-038 — Report a predicted uncertainty per solved volume [P3] [status: ABSENT]

**Requirement.** SandiBumi SHOULD emit a per-component predicted uncertainty from the linearised
covariance of the solved system, scaled by the run's misfit statistic, and MUST state that it is a
linearised estimate that excludes endpoint uncertainty.

**Rationale.** F-12 (T3 Geolog §H): Geolog reports `sqrt(diag(A⁻¹))·QUALITY` per volume. The three
vendors answer three different questions — IP propagates *endpoint* uncertainty, Elan balances
uncertainties *before* the volumes are solved, Geolog linearises *after* — and **they are
complementary, not competing.** Emitting Geolog's is the cheapest of the three because the inverse is
already formed by the solve; the caveat is mandatory because a linearised band silently omits exactly
the term IP's Monte Carlo exists to capture.

**As-built.** `ABSENT` — the KKT inverse is formed and discarded (`multimin2.rs:1913`–`:1930`); the
result carries no per-volume uncertainty (`:495`–`:523`).

**Verified by.** SB-MIN-T39

#### SB-MIN-039 — Offer balanced pre-solve tool uncertainties [P3] [status: ABSENT]

**Requirement.** SandiBumi SHOULD offer a balanced pre-solve uncertainty mode that scales each tool's
σ so no tool dominates the objective by scale alone, and MUST state that the balancing is computed
before the volumes are known.

**Rationale.** F-12 (T2-equivalent Elan `-solution-method.html`): Elan computes balanced uncertainties
**before** the volumes are solved, because *"uncertainties do not include the volume of the mineral"* —
a deliberate design statement, not an approximation. Without balancing, a tool measured in c.u.
(SIGMA, range ~10–60) and a tool measured in v/v (NPHI, range ~0–0.6) enter the same objective at
scales two orders of magnitude apart, and the weighting an interpreter *thinks* they set is not the
weighting the solver applies.

**As-built.** `ABSENT` — every `σ` is caller-supplied per tool (`multimin2.rs:87`–`:91`); only the
conductivity rows get an automatic value (`:597`–`:598`).

**Verified by.** SB-MIN-T20

#### SB-MIN-040 — Make the two bound-water routes mutually exclusive and record the choice [P1] [status: PARTIAL]

**Requirement.** Exactly one bound-water parameterisation MUST be active per run; the choice MUST be
recorded in the run record and stamped on the emitted `PHIE`, `SWE` and `VSH` curves. A component
carrying parameters for both routes MUST be accepted only when SB-MIN-008's matched-pair tolerance
holds.

**Rationale.** SB-MIN-008 quantifies what the choice is worth — **1.70 pu of PHIE on Illite at
`V_dryclay = 0.25`**, moved by a radio button. A number that large must not be recoverable only by
remembering which way the button was set. This is also the mechanism that makes SB-MIN-027's refusal
meaningful: a rejected WCLP must fail, not silently become a CEC answer wearing a WCLP label.

**As-built.** `PARTIAL` — the routes are genuinely exclusive (`PorositySource`, `multimin2.rs:136`–
`:143`; selection `:616`–`:630`), but the choice is recorded nowhere in the output
(`MultiminResult`, `:495`–`:523`) and the ceiling branch crosses between them silently (`:623`–`:624`).

**Verified by.** SB-MIN-T08, SB-MIN-T27

#### SB-MIN-041 — Keep retired modules resolvable and refuse to run them, carrying no orphan defaults [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A retired module's spec MUST remain resolvable by name so a saved chain step can
explain itself, and `run_module` MUST refuse to execute it with a message naming the replacement. A
retired module MUST NOT continue to expose parameter defaults that no live module shares; such
defaults MUST either be removed from the rendered spec or labelled as historical values of a retired
method.

**Rationale.** The refusal half is already right and is the pattern this project should keep: resolve
by name, render stored parameters, refuse to compute. Deleting the spec instead would break every
saved chain that references it; leaving it runnable would silently ship superseded physics. What is
still wrong is narrower — the retired spec ships 20 endpoint defaults, including `RHOB_CLAY 2.55` and
`PEF_CLAY 3.10`, which **share no table with the live solver's library** (SandiMin's generic `Clay`
row is RHOB 2.65 / PEF 3.50, and no clay in `LIB` carries either legacy value). They no longer drive a
computation, but they are still *displayed to a user as this product's clay endpoints*, with no
provenance — which is the exact exposure `SB-CORE-004` and `SB-CORE-005` exist to close, arriving
through a back door neither of them is looking at. P0 for that reason, not for the arithmetic.

> **The retirement also left the canonical lesson about test shape.** The legacy solver's PEF test
> forward-modelled its expected value with the **same wrong mixing law** the solver used
> (`let pef = vs * 1.81 + vw * 0.36;`), so it passed by construction and would have gone on passing
> however wrong the physics was. It was a characterization test wearing a correctness test's costume.
> The replacement asserts against an independently derived expectation (**1.382 b/e**) *and* asserts
> that the two candidate mixing laws differ by more than 0.25 — pinning the wrong law as wrong. §6
> applies that rule chapter-wide: a test whose expected value was produced by the code under test is
> labelled `CHARACTERIZATION` and never counted as verification of a requirement.

**As-built.** `PRESENT-DIVERGENT` — refusal correct: `retired_module` `modules.rs:403`–`:411` returns
`Some(...)` for `"multimin"` with an actionable message, and `run_module` guards on it at `:418`–`:420`
before dispatch; the spec stays catalogued at `:382`; both halves pinned by
`multimin_is_retired_but_still_cataloged` (`:2856`–`:2862`), which also asserts a live module is not
flagged (`:2859`). Residue: 20 hard-coded defaults at `multimin.rs:31`–`:52`, including `:32` and
`:44`.

**Verified by.** SB-MIN-T40, SB-MIN-T41

### 4.10 Remaining constraints, units and environment

#### SB-MIN-042 — Implement the oil-based-mud constraint pair [P3] [status: ABSENT]

**Requirement.** Under oil-based mud SandiBumi MUST impose the mirror inequality on the hydrocarbon
components and `v_Xgas ≤ v_Ugas`, rather than imposing no zone-consistency constraint at all.

**Rationale.** Adoption-spec F-4. Suppressing the water-mud row for oil mud is correct — filtrate is
not water — but it leaves the OBM case with **no** zone-consistency constraint whatsoever, so nothing
prevents a solution in which the flushed zone holds less hydrocarbon than the virgin zone. That is
physically backwards under oil-mud invasion and it currently cannot be detected, only eyeballed.

**As-built.** `ABSENT` — oil mud sets `alpha_x = alpha_u` in the fluid calc (`multimin2.rs:584`) and
suppresses the water-mud row (`:1175`); no replacement constraint is built.

**Verified by.** SB-MIN-T36

#### SB-MIN-043 — Offer the opt-in physical ceiling constraints [P3] [status: ABSENT]

**Requirement.** SandiBumi SHOULD offer opt-in `PHIMAX` (total porosity ceiling), `BVIRR` (irreducible
bulk-volume-water floor) and `IRRWAT` constraints, each off by default and each recorded in the run
record when enabled.

**Rationale.** Adoption-spec F-4 lists all three in the constraint taxonomy; they are the standard
route by which an interpreter injects a rock-physics limit the log data alone cannot supply.
Defaulting them **off** is deliberate: an unlabelled ceiling that silently caps porosity is the same
failure class as SB-MIN-007, so the requirement is the pair *(available, recorded)* rather than
*(available)*.

**As-built.** `ABSENT` — the constraint set is UNITY, BOX, POROSITY, BNDWAT and WATER MUD only
(`multimin2.rs:1146`–`:1195`).

**Verified by.** SB-MIN-T36

#### SB-MIN-044 — Canonicalise units at the boundary and prove invariance [P2] [status: PARTIAL]

**Requirement.** Every endpoint, measurement and parameter MUST be converted to a single declared
internal unit set at the request boundary, and the solved volumes MUST be invariant to the unit system
of the incoming curves.

**Rationale.** The domain's constants are unit-locked in ways that are invisible at the call site:
`96` requires ρ in g/cc where the vendor's printed `0.096` requires kg/m³ (SB-MIN-006); `0.1883 g/cc`
is `188.3 kg/m³` (SB-MIN-012); WCLP is m³/m³ not p.u. (SB-MIN-027); CEC is meq/g not meq/100 g
(SB-MIN-011). Four unit traps in one solver, each of which computes cleanly and plots. A single
declared internal unit set plus an invariance test converts all four from vigilance into a gate.

**As-built.** `PARTIAL` — display units are declared once for the library (`multimin2.rs:2044`–`:2046`)
and temperature is handled explicitly in both °F and °C (`:775`–`:776`, `:604`–`:606`), but there is no
boundary conversion layer and no invariance test; the endpoint map is `HashMap<String, f64>` with the
unit carried only by convention (`:63`).

**Verified by.** SB-MIN-T42

#### SB-MIN-045 — Bound the formation temperature and record any fallback [P2] [status: PARTIAL]

**Requirement.** A formation temperature outside a declared physical window MUST NOT enter the fluid
calculation. Where a curve value is rejected and a constant substituted, the substitution MUST be
counted and reported with the run.

**Rationale.** Every temperature-dependent term in this solver — the bound-water multiplier's
`(T + 298)` (`multimin2.rs:604`–`:606`), the Clavier-Coates-Dumanoir `(T + 8.5)(T + 298)` bound-water
conductivity (`:580`), the Waxman-Smits `B(T, Rw)` (`:326`) — degrades smoothly and wrongly on a fill
value rather than failing. The rejection window already exists and is correct; what is missing is the
*count*, without which a curve that is 90 % fill values produces a run indistinguishable from one that
is clean. This is CONTRACT §5.3 applied to a guard that is already half-built.

**As-built.** `PARTIAL` — `FTEMP_MIN_F = 32.0` / `FTEMP_MAX_F = 600.0` (`multimin2.rs:775`–`:776`)
reject ±999.25 and 9999 fills and revert to the constant temperature (`:1310`–`:1315`, `:1380`); no
count and no report reach `MultiminResult` (`:495`–`:523`).

**Verified by.** SB-MIN-T43

#### SB-MIN-046 — Gate the clay density triple for self-consistency [P2] [status: ABSENT]

**Requirement.** Where a clay row carries wet-clay density, dry-clay density and wet-clay porosity,
SandiBumi MUST check `ρ_dcl = (ρ_wcl − WCLP·ρ_bw)/(1 − WCLP)` and MUST warn, naming the row and the
discrepancy, when the three disagree by more than 1 %. SandiBumi MUST NOT store all three as silently
independent values.

**Rationale.** F-15 (T1 Techlog `QM_MineralTable.xml`; T2-equivalent Elan Eq 11). **Techlog's own clay
table does not satisfy Techlog's own equation**: Illite `(2.52 − 0.17)/0.83 = 2.8313` against the
tabulated `Rhobdcl = 2.70` (**+4.9 %**); Kaolinite `2.5161` against 2.65 (**−5.1 %**); Chlorite 2.7742
against 2.80 (−0.9 %). Anyone adopting Techlog's `Rhobwcl` + `Phicl` and deriving `ρ_dcl` via Eq 11
gets a different number than Techlog's own `Rhobdcl` column, and no vendor statement explains the gap.
This is the same error class as SB-MIN-008 — internally inconsistent parameters that each look
correctly sourced — and CONTRACT §5.1: a vendor defect that is free to get right.

**As-built.** `ABSENT` — `LibRow` carries one `RHOB` per row with no convention marker
(`multimin2.rs:2053`–`:2065`), so the triple cannot be checked because it is not stored.

**Verified by.** SB-MIN-T44

---

## 5. Parameters

One table, complete for the domain. Every parameter any §4 requirement refers to appears here exactly
once. This table is where `SB-CORE-004` (no parameter ships without a machine-readable source string;
the build fails otherwise) and `SB-CORE-005` (every endpoint in the mineral library cites a primary
source; rows that cannot be re-sourced are marked vendor-derived in the UI and in the deliverable) are
discharged for this domain — so the `Source` cell is the deliverable artefact, not decoration.

**What this table deliberately does not contain.** The 27-row × 14-tool-key `LIB`
(`multimin2.rs:2079`–`:2108`) is **not** enumerated here. Rows appear below only where a §4
requirement turns on the specific value. Enumerating the full matrix would be transcription of a
merged vendor library into a second location, which is the exposure `SB-CORE-005` exists to reduce,
not to duplicate. The matrix is carried as an *asset* — 27 rows, 14 columns, `VENDOR-DERIVED`, merged
from two vendor installs in a third vendor's dropdown order per `IP_PROVENANCE.md` §2.2 — and
`SB-MIN-009` is the requirement that re-sources it row by row. No vendor chart lookup-table data,
`.neu`/`.ovl` table, or CHM content is transcribed anywhere in this chapter.

**Reading the Source column.** `VENDOR-DERIVED` marks a value whose only provenance is a vendor
install — these are the rows `SB-CORE-005` requires be labelled as such in the UI and the deliverable
until re-sourced. `NON-ADOPTABLE — cited for verification` marks a value transcribed so a requirement
can be checked, which SandiBumi does **not** ship. `ABSENT — ships with no default` marks a parameter
that must be supplied by the interpreter.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| **— Solver and constraint numerics —** | | | | | |
| Fluid component upper bound | `max_vol` (fluid) | 0.500 | v/v | Geolog RF04 6.2 default fluid ceiling; shipped `multimin2.rs:2073` | T3 |
| Mineral/clay component upper bound | `max_vol` (solid) | 1.000 | v/v | SandiBumi engineering default; `multimin2.rs:2067`, `:2070` | — |
| Tool-constraint uncertainty | `SIGMA_CONSTRAINT` | 0.010 | tool units | Geolog `Tool`-row σ; shipped `multimin2.rs:768`, user-overridable `:1169` | T3 |
| Unity closure tolerance | — | 1e-9 | v/v | SandiBumi engineering; asserted by SB-MIN-T03 | — |
| Active-set outer iteration cap | `max_outer` | `8n + 12` | iterations | SandiBumi engineering; `multimin2.rs:1893` | — |
| Water-mud violation trigger | — | −1e-6 | v/v | SandiBumi engineering; `multimin2.rs:1406` | — |
| **— Pe ↔ U conversion (SB-MIN-012) —** | | | | | |
| Electron-density offset | — | 0.1883 | g/cc | IP E26 `U = Pef × (ρb + 0.1883) × 0.93423`, printed twice; = Geolog `188.3 kg/m³` | T2 · T3 |
| Electron-density divisor | — | 1.0704 | — | Geolog density chain `ρ_a = 1.0704·ρ_e − 188.3`; `1/1.0704 = 0.934230…` = IP's `0.93423` | T3 · T2 |
| **— Bound water, CEC route (SB-MIN-006) —** | | | | | |
| Bound-water CEC constant | — | 96 | (meq/g · g/cc · K)⁻¹ | Geolog RF04 6.2, printed as `0.096` for ρ in kg/m³; shipped in g/cc form `multimin2.rs:605` | T3 |
| Bound-water temperature offset | — | 298 | K-offset on °C | Geolog RF04 6.2; `multimin2.rs:605` | T3 |
| Diffuse-layer expansion threshold | — | 20 455 | ppm NaCl | Clavier-Coates-Dumanoir diffuse-layer expansion, via Geolog RF04 6.2; `multimin2.rs:558` | T3 |
| Diffuse-layer expansion cap | `α_max` | 5.0 | — | **SandiBumi engineering guard — no vendor source.** Bounds `sqrt(20455/S)` as `S → 0`; `multimin2.rs:559` | — |
| Bound-water conductivity coefficient | — | 0.0007 | (c.u.·°C⁻²) | Geolog RF04 6.2 / Elan Eq 62; `multimin2.rs:580` | T3 · T2-equiv |
| Bound-water conductivity temperature term | — | 8.5 | °C-offset | Clavier-Coates-Dumanoir `(T + 8.5)`, in Geolog and Elan Eq 62 alike; `multimin2.rs:580` | T3 · T2-equiv |
| CEC plausibility window | — | [0.01, 2.0] | meq/g | **DERIVED** — a decade either side of the shipped library span (Kaolinite 0.10 … Montmorillonite 1.0); SB-MIN-011 | — |
| **— Bound water, WCLP route (SB-MIN-027) —** | | | | | |
| Wet-clay-porosity physical ceiling | `WCP_PHYSICAL_CEILING` | 0.500 | v/v | **SandiBumi engineering guard**; `multimin2.rs:784`. Bounds the `WCLP/(1−WCLP)` pole | — |
| Wet-clay-porosity unit | `WCLP` | — | m³/m³ (v/v) | Techlog `QElan_PostProcess_Using_Conductivities.py`, `WCLP_*_unit = u"m3/m3"` | T1 |
| Matched-pair agreement tolerance | — | 0.02 | relative | **DERIVED** — SB-MIN-008 gate; set above the 1.5 % worst-case intra-vendor spread (F-22) and far below the 58.6 % chimera | — |
| Matched-pair reference temperature | — | 64.4 | °C | Dossier verification fixture; used only inside the SB-MIN-008 gate | — |
| **— Clay bound-water endpoints: the matched-pair conflict (SB-MIN-008, escalated ESC-2) —** | | | | | |
| Illite CEC (shipped) | — | 0.25 | meq/g | Geolog RF04 6.2; `multimin2.rs:2096`. **VENDOR-DERIVED** | T3 |
| Illite CEC (competing) | — | 0.16 | meq/g | Techlog `QElan_PostProcess_Using_Conductivities.py` — **NON-ADOPTABLE — cited for verification** | T1 |
| Illite WCLP (shipped) | — | 0.104 | v/v | Techlog `QElan_PostProcess_Using_Conductivities.py`; `multimin2.rs:2096`. **VENDOR-DERIVED — mismatched to the shipped CEC** | T1 |
| Illite WCLP (Geolog pair) | — | 0.1555 | v/v | Geolog RF04 6.2 — the value that pairs with CEC 0.25 to 0.01 % | T3 |
| Illite dry-clay density | — | 2.78 | g/cc | Shipped `multimin2.rs:2096`; enters the SB-MIN-008 gate. **VENDOR-DERIVED** | T3 |
| Kaolinite CEC (shipped) | — | 0.10 | meq/g | Geolog RF04 6.2; `multimin2.rs:2094`. **VENDOR-DERIVED** | T3 |
| Kaolinite CEC (competing) | — | 0.09 | meq/g | Techlog `QElan…py` — **NON-ADOPTABLE — cited for verification** | T1 |
| Kaolinite WCLP (shipped) | — | 0.058 | v/v | Techlog `QElan…py`; `multimin2.rs:2094`. **VENDOR-DERIVED — mismatched** | T1 |
| Kaolinite WCLP (Geolog pair) | — | 0.06489 | v/v | Geolog RF04 6.2 — pairs with CEC 0.10 to 0.02 % | T3 |
| Kaolinite dry-clay density | — | 2.62 | g/cc | Shipped `multimin2.rs:2094`. **VENDOR-DERIVED** | T3 |
| Chlorite CEC (shipped) | — | 0.15 | meq/g | Geolog RF04 6.2 **and** Techlog `QElan…py` — the one three-way-consistent clay | T3 · T1 |
| Chlorite WCLP (shipped) | — | 0.101 | v/v | Techlog `QElan…py`; `multimin2.rs:2095`. Pairs to −0.6 % because the CEC values agree | T1 |
| Chlorite WCLP (Geolog pair) | — | ABSENT — ships with no default | v/v | **Not held** — Geolog RF04 6.2 chlorite WCLP was not located (→ ESC-2) | — |
| Chlorite dry-clay density | — | 2.81 | g/cc | Shipped `multimin2.rs:2095`. **VENDOR-DERIVED**; IP's wet-clay-convention value (SB-MIN-010) | T3 |
| Glauconite CEC (shipped) | — | 0.20 | meq/g | Geolog RF04 6.2; `multimin2.rs:2093`. **VENDOR-DERIVED** | T3 |
| Glauconite CEC (competing) | — | 0.233 | meq/g | Techlog `QElan…py` — **NON-ADOPTABLE — cited for verification** | T1 |
| Glauconite WCLP (shipped) | — | 0.156 | v/v | Techlog `QElan…py`; `multimin2.rs:2093`. **VENDOR-DERIVED — mismatched** | T1 |
| Glauconite WCLP (Geolog pair) | — | ABSENT — ships with no default | v/v | **Not held.** Geolog RF04 6.2 glauconite WCLP was not located; adopting Geolog's pair wholesale is therefore not fully sourced (→ ESC-2) | — |
| Glauconite dry-clay density | — | 2.96 | g/cc | Shipped `multimin2.rs:2093`. **VENDOR-DERIVED** | T3 |
| Montmorillonite / Smectite CEC | — | 1.0 | meq/g | Geolog RF04 6.2 **and** Techlog `CEC_Smectite = 1.0` | T3 · T1 |
| Montmorillonite / Smectite WCLP | — | 1.0 | v/v | Techlog `WCLP_Smectite = 1`; `multimin2.rs:2097`. Above `WCP_PHYSICAL_CEILING`, so the CEC route is used | T1 |
| Montmorillonite dry-clay density | — | 2.63 | g/cc | Shipped `multimin2.rs:2097`. **VENDOR-DERIVED** | T3 |
| Generic `Clay` CEC | — | ABSENT — ships with no default | meq/g | **Currently ships 0.00** (`multimin2.rs:2098`), which the solver reads as a real zero and which SB-MIN-007 forbids. Techlog encodes the same state as `CEC_Shale = −9999` | — |
| Generic `Clay` WCLP | — | ABSENT — ships with no default | v/v | **Currently ships 0.120** (`multimin2.rs:2098`), matching no vendor file examined; Techlog's `WCLP_Shale` is the `−9999` sentinel | — |
| Bound-water density | `ρ_bw` | ABSENT — ships with no default | g/cc | **Currently hard-coded 1.000** inside `dry_clay_calc` (`multimin2.rs:676`–`:699`) with no source; SB-MIN-024 requires it become a sourced parameter | — |
| Bound-water transit time | `Δt_bw` | 189.0 | µs/ft | Hard-coded `DT_W` `multimin2.rs:676`–`:699`; equals the shipped water fluid `DT`, Geolog RF04 6.2 | T3 |
| **— Fluid endpoints referenced by requirements (SB-MIN-029) —** | | | | | |
| Water `DT` | — | 189.0 | µs/ft | Geolog RF04 6.2; `multimin2.rs:2102`–`:2103`. **VENDOR-DERIVED** | T3 |
| Oil `DT` | — | 189.0 | µs/ft | Geolog RF04 6.2; `multimin2.rs:2104`–`:2105`. **VENDOR-DERIVED**; low end of the cross-vendor oil spread (F-5) | T3 |
| Gas `DT` | — | 250.0 | µs/ft | Geolog RF04 6.2; `multimin2.rs:2106`–`:2107`. **VENDOR-DERIVED**; cross-vendor spread on gas is 45 µs/ft (F-5) | T3 |
| Oil `RHOB` / `NPHI` | — | 0.80 / 1.00 | g/cc / v/v | Geolog RF04 6.2; `multimin2.rs:2104`–`:2105`. **VENDOR-DERIVED** | T3 |
| Gas `RHOB` / `NPHI` | — | 0.20 / 0.44 | g/cc / v/v | Geolog RF04 6.2; `multimin2.rs:2106`–`:2107`. **VENDOR-DERIVED** | T3 |
| **— Fluid property calculation —** | | | | | |
| Conductivity root exponent (default) | `w` | `0.75m + 0.25n` | — | Geolog RF04 6.2; `multimin2.rs:575`. Alternatives `1/2` (Elan) and `1/m` (IP) are the SB-MIN-021 model input | T3 |
| Conductivity exponent fallback / guard | — | 2.0 / 0.5 | — | SandiBumi engineering; `multimin2.rs:575`–`:576` | — |
| Auto conductivity uncertainty fraction | — | 0.03 | relative | Shipped `multimin2.rs:597`–`:598`; **primary source NOT ESTABLISHED** (→ ESC-5) | — |
| Arps temperature-correction constant | — | 6.77 | °F-offset | Arps (1953) resistivity-temperature relation, °F form; `multimin2.rs:543`–`:545` | primary |
| Rw → NaCl-equivalent salinity fit | — | `10^((3.562 − log₁₀(Rw₇₅ − 0.0123))/0.955)` | ppm | Shipped `multimin2.rs:548`–`:554`; **primary source NOT ESTABLISHED** (→ ESC-5) | — |
| Saturated-brine guard | — | 400 000 at `Rw₇₅ ≤ 0.0124` | ppm | SandiBumi engineering; `multimin2.rs:548`–`:554` | — |
| Formation-temperature validity window | `FTEMP_MIN_F` / `FTEMP_MAX_F` | 32.0 / 600.0 | °F | SandiBumi engineering guard; `multimin2.rs:775`–`:776`. Rejects ±999.25 and 9999 fills | — |
| **— Saturation-adjacent inputs (SB-MIN-021 … SB-MIN-023) —** | | | | | |
| Shell porosity-dependent `m` constant | — | ABSENT — ships with no default | — | Three-way conflict: IP raster **0.018** (verified 4× in 2025, 6× in 2018), IP ASCII **0.019** (both editions), Techlog `mc2` *"the usual value assumed for mc2 is 0.19"* with Table 28 default 0.0. No defensible adjudication (ledger D-10, → ESC-4) | T2 · T2-equiv |
| Variable `m*` coefficients, Dual-Water | — | 0.258, 0.2, 16.4 | — | IP E68 **and** Elan Eq 65/66 — **bit-identical across two independent products** | T2 · T2-equiv |
| Variable `m*` coefficients, Waxman-Smits | — | 1.128, 0.22, 17.3 | — | IP E68 only — **single-sourced; must be labelled as such** | T2 |
| Variable `m*` base exponent | `m` | ABSENT — ships with no default | — | IP defaults 2.0, Elan `mdw` defaults 1.8; no adjudication | T2 · T2-equiv |
| Elan-Simandoux silt exponents | `ersh` / `swshe` | 1.0 / 0.5 | — | Elan Table 28; correspond to Worthington Type-2 `x = 1.0`, `c = 1.5` | T2-equiv |
| Per-clay shale resistivity | `Rsh` | Illite 3, Chlorite 5, Shale 5, Kaolinite 7 | ohm-m | Techlog `QM_MineralTable.xml` — **NON-ADOPTABLE — cited for verification** (SB-MIN-031 requires the *capability*, not these values) | T1 |
| Techlog clay `XWater` | — | 0.03 | v/v | Techlog `QM_MineralTable.xml` — **NON-ADOPTABLE — cited for verification** | T1 |
| **— Diagnostics and uncertainty (SB-MIN-014 … SB-MIN-020, SB-MIN-037 … SB-MIN-039) —** | | | | | |
| Conditioning-number thresholds | `CONDNUM` | 8 suspect / 10 unstable; linear cutoff default 10 | log₁₀ | Geolog §H | T3 |
| `QUALITY` chi-square parameters | — | 95 % critical value, `n_tool − 3` d.o.f. | — | Geolog §H, `QUALITY = sqrt(Δ²/χ²₉₅(n_tool − 3))` | T3 |
| Default tool uncertainty generating rule | — | 1.5 % of (MAX − MIN) | tool units | Elan Tables 29/30 (exact on 9 of 14 rows) **and** Geolog §E/§L, *"default = ~1.5 % of the tool's normal logged range; weight = 1/U²"* — two independent vendors | T2-equiv · T3 |
| Per-tool default uncertainties | `σ_k` | ABSENT — ships with no default | tool units | Elan's own table deviates from its own rule on 6 rows (DT, CUDC, CXDC, TPL, VELC, SDPT) by 4–7 %; IP tabulates without a rule. SB-MIN-019 requires MIN, MAX and the printed default be stored as three fields, not derived | T2-equiv · T3 |
| Weight multiplier | `xxxx_WM` | 1.0 | — | Elan, *"a multiplier value of 1.0 means that the tool will influence the answer as strongly as the Volume Summation tool"* | T2-equiv |
| Monte Carlo endpoint shift | — | ±10 % of endpoint value, Gaussian | relative | IP Mineral Solver Monte Carlo default | T2 |
| Monte Carlo iterations | — | 2000 | iterations | IP Mineral Solver Monte Carlo default | T2 |
| Monte Carlo seed | — | ABSENT — ships with no default | integer | IP *"uses a random number generator, seeded through the CPU clock time"* — **the vendor default is the defect** (SB-MIN-037); SandiBumi MUST require an explicit seed | T2 |
| Clay density triple tolerance | — | 0.01 | relative | **DERIVED** — SB-MIN-046 gate; set below Techlog's own worst self-inconsistency (Kaolinite −5.1 %) so that case is caught | — |
| **— Rock-physics constants in the endpoint builder —** | | | | | |
| `VP` from `DT` conversion | — | 304.8 | m·µs/(ft·s) | Unit conversion, `1 ft = 0.3048 m`; `multimin2.rs:2116` | primary |
| `VS` from `VP` ratio | `VP/VS` | ABSENT — ships with no default | — | **Currently hard-coded 1.7** (`multimin2.rs:2117`) for every non-fluid component, i.e. one Poisson ratio for quartz, calcite, dolomite and clay alike. No source; not lithology-dependent (→ ESC-8) | — |
| **— Retired legacy module residue (SB-MIN-041) —** | | | | | |
| Legacy `RHOB_CLAY` | — | 2.55 | g/cc | `multimin.rs:32`, retired module spec. **No live module shares this value** (SandiMin generic `Clay` RHOB is 2.65); no source | — |
| Legacy `PEF_CLAY` | — | 3.10 | b/e | `multimin.rs:44`, retired module spec. **No live module shares this value** (SandiMin generic `Clay` PEF is 3.50); no source | — |
| Legacy `SIG_PEF` | — | 0.30 | b/e | `multimin.rs`, retired module spec. Cited only because the pre-R17 Pe-mixing error was exactly 1.0× this value (F-20, SB-MIN-012) | — |

**Count.** **78 parameter rows.** Ten ship `ABSENT — ships with no default`: Glauconite and Chlorite
Geolog-pair WCLP, generic `Clay` CEC, generic `Clay` WCLP, `ρ_bw`, the Shell `m` constant, the
variable-`m*` base exponent, the per-tool default uncertainties, the Monte Carlo seed, and `VP/VS`.
**Four of those ten are values SandiBumi ships today** (generic `Clay` CEC 0.00 and WCLP 0.120,
`ρ_bw` 1.000, `VP/VS` 1.7) — they are listed as absent because they carry no defensible source, which
is precisely the state `SB-CORE-004`'s build gate is meant to make impossible. Five further values
are `NON-ADOPTABLE — cited for verification` (three competing Techlog clay CECs, the four per-clay
`Rsh` values, `XWater`); they are transcribed so a requirement can be checked and are not shipped.
Every remaining row carries a source string, and every row whose only provenance is a vendor install
is marked `VENDOR-DERIVED` for `SB-CORE-005`.

---

## 6. Acceptance tests

44 tests, `SB-MIN-T01` … `SB-MIN-T44`. Every requirement in §4 names at least one.

**The rule this chapter enforces on test shape.** An expected value that was produced by the code
under test is not a verification of anything. The canonical failure is in this module's own history:
the legacy solver's photoelectric test forward-modelled its expectation with the **same wrong mixing
law** the solver used (`let pef = vs * 1.81 + vw * 0.36;`), so it passed by construction and would
have kept passing however wrong the physics became — a characterization test wearing a correctness
test's costume. Accordingly: **every test below states where its expected value comes from**, derived
expectations show their arithmetic inline, and a test whose expectation has no external source is
labelled `CHARACTERIZATION` and does not count as verification of a requirement.

### 6.1 Solver core

**`SB-MIN-T01` — bounded solve never deletes a component.**
*Input:* a three-mineral + water model at a depth where the unconstrained least-squares solution
drives one mineral to `−0.04 v/v`. *Operation:* solve. *Expected:* all volumes `≥ 0` and `≤ hi`; the
returned component list has the same length and the same names as the input; unity holds to `1e-9`.
*Source:* SB-MIN-001; the invariant is the constraint set itself, not a vendor number.

**`SB-MIN-T02` — the run record states the solver class.**
*Input:* any solve. *Operation:* read the run record. *Expected:* a field stating that non-negativity
is imposed by bound constraints, not by mineral deletion. *Source:* SB-MIN-002.

**`SB-MIN-T03` — unity is hard and excludes X-only fluids.**
*Input:* a model with one X-only fluid, one shared fluid and three solids. *Operation:* solve.
*Expected:* `Σ(non-X volumes) = 1.000000000 ± 1e-9`; the X-only fluid's coefficient in the unity row
is exactly `0`. *Source:* SB-MIN-003; tolerance is the shipped closure tolerance in §5.

**`SB-MIN-T04` — a renamed fluid keeps the 0.5 ceiling.**
*Input:* the library water row, renamed to `Brine (zone A)`, with `max_vol` omitted from the request.
*Operation:* solve. *Expected:* the applied upper bound is `0.500`, not `1.000`. *Source:* SB-MIN-005;
the value is the Geolog fluid ceiling in §5. **This test fails today** — the name-keyed lookup at
`multiminDialog.ts:1093` and the serde default at `multimin2.rs:78` both yield 1.0.

### 6.2 Bound water and the endpoint library

**`SB-MIN-T05` — the CEC bound-water multiplier reproduces the verified fixture.**
*Input:* `CEC = 0.25 meq/g`, `ρ_dcl = 2.78 g/cc`, `T = 64.4 °C`, `α = 1`. *Operation:*
`bndwat_multiplier`. *Expected:* `k = 0.184106 ± 1e-6`.
*Arithmetic:* `96 × 0.25 × 2.78 = 66.72`; `64.4 + 298 = 362.4`; `66.72 / 362.4 = 0.1841060…`
*Source:* Geolog RF04 6.2 bound-water relation (§5); fixture conditions from the dossier.

**`SB-MIN-T06` — the diffuse-layer expansion factor and its cap.**
*Input:* `S = 5 000 ppm`, then `S = 500 ppm`, then `S = 35 000 ppm`. *Operation:* `alpha_expansion`.
*Expected:* `2.022622 ± 1e-6`; then `5.000000` exactly (the cap); then `1.000000` exactly.
*Arithmetic:* `sqrt(20455/5000) = sqrt(4.0910) = 2.0226221…`; `sqrt(20455/500) = sqrt(40.910) =
6.3961…` → capped at 5.0; `35 000 > 20 455` → 1.0.
*Source:* Clavier-Coates-Dumanoir expansion via Geolog RF04 6.2 (§5). The cap is a SandiBumi guard and
is asserted as such, not as a vendor value.

**`SB-MIN-T07` — an absent clay CEC refuses; it does not silently become zero.** **[P0]**
*Input:* the generic `Clay` component (`CEC = 0.00`) at `V_clay = 0.30`, `PorositySource::Cec`.
*Operation:* solve. *Expected:* the run **fails** with a message containing the component name `Clay`
and the parameter name `CEC`. *Control:* the same component under `PorositySource::WetClayPorosity`
returns `V_bw = 0.040909 ± 1e-6`.
*Arithmetic:* `0.120/(1 − 0.120) = 0.1363636…`; `0.30 × 0.1363636 = 0.0409091 v/v = 4.09 pu`.
*Source:* SB-MIN-007. The control quantifies what the silent-zero path currently discards. **This test
fails today** — the current path returns `k = 0` with no error (`multimin2.rs:617`).

**`SB-MIN-T08` — the (CEC, WCLP) matched-pair gate.** **[P0]**
*Input:* each shipped clay row. *Operation:* evaluate `k_CEC(α = 1, T = 64.4 °C)` and
`k_WCLP = WCLP/(1 − WCLP)` and compare. *Expected:* `|k_CEC − k_WCLP| / k_CEC ≤ 0.02` for every clay.
*Arithmetic, Illite as shipped:* `k_CEC = 0.184106`; `k_WCLP = 0.104/0.896 = 0.1160714`;
`|Δ|/k_CEC = 0.068035/0.184106 = 0.36954` — **18× the gate**, and equivalently **+58.6 %** when
normalised on `k_WCLP`, which is the figure quoted in §3.4.
*Controls, each of which must pass:* Geolog's own Illite pair `(0.25, 0.1555)` →
`0.1555/0.8445 = 0.1841326`, `|Δ|/k_CEC = 0.00015` (0.01 %); Techlog's own Illite pair `(0.16, 0.104)`
→ `96 × 0.16 × 2.78/362.4 = 0.1178278` against `0.1160714`, `|Δ|/k_CEC = 0.0149` (1.5 %).
*Diagnostic control:* shipped Chlorite `(0.15, 0.101)` → `0.1116556` against `0.1123470`,
`|Δ|/k_CEC = 0.0062` — **passes**, because both vendors ship `CEC = 0.15`. That this one clay passes
while the others fail is the evidence the defect is a mixed pair and not a bad formula.
*Source:* SB-MIN-008; the vendor pairs are the §5 rows. **This test fails today on four of five clays.**

**`SB-MIN-T09` — every shipped endpoint value carries a source string.** **[P0]**
*Input:* the shipped library. *Operation:* enumerate every value. *Expected:* every value has a
non-empty source string, and every value whose source is a vendor install is flagged
`VENDOR-DERIVED`. *Source:* `SB-CORE-004` (build gate) and `SB-CORE-005` (mineral-library
re-sourcing), discharged for this domain by §5. **This test fails today** — `LibRow` has no source
field (`multimin2.rs:2053`–`:2065`).

**`SB-MIN-T10` — mixing wet- and dry-clay conventions refuses.** **[P0]**
*Input:* two clay rows, one declared wet-clay and one declared dry-clay, with no conversion.
*Operation:* solve. *Expected:* refusal naming both rows and the differing convention. *Source:*
SB-MIN-010; the hazard magnitude (5.7 σ on `RHOB`, up to 6.9 pu of porosity) is F-4.

**`SB-MIN-T11` — an out-of-range CEC refuses with a unit hint.** **[P0]**
*Input:* `CEC = 16.0` (a `meq/100 g` value entered as `meq/g`), then `CEC = 0.001`. *Operation:*
validate. *Expected:* both refuse, and the message names `meq/g` and the accepted window
`[0.01, 2.0]`. *Source:* SB-MIN-011; the window is the `DERIVED` row in §5.

**`SB-MIN-T12` — photoelectric response mixes on U, not on Pe.** **[P0]**
*Input:* 50 % quartz (`Pe = 1.81`, `ρ = 2.65`) + 50 % water (`Pe = 0.36`, `ρ = 1.00`). *Operation:*
forward-model the PEF response. *Expected:* `PEF = 1.3821 ± 0.0005 b/e`, **and** the test asserts that
the linear-Pe result `1.085` differs from it by more than `0.25 b/e`.
*Arithmetic:* `ρe(2.65) = (2.65 + 0.1883)/1.0704 = 2.6516255`; `U_qz = 1.81 × 2.6516255 = 4.7994422`;
`ρe(1.00) = 1.1883/1.0704 = 1.1101457`; `U_w = 0.36 × 1.1101457 = 0.3996524`;
`U_mix = 0.5 × 4.7994422 + 0.5 × 0.3996524 = 2.5995473`;
`ρb_mix = 1.825`, `ρe(1.825) = 2.0133/1.0704 = 1.8808857`;
`PEF = 2.5995473/1.8808857 = 1.38209`. The linear-Pe law gives `0.5 × 1.81 + 0.5 × 0.36 = 1.085`, a
`0.297 b/e` residual — **0.99× the legacy `SIG_PEF` of 0.30**.
*Source:* IP E26 / Elan Eq 16 / Geolog §I-§J (§5). **The second assertion is the point**: it pins the
wrong law as wrong, which is exactly what the retired module's original test failed to do.

### 6.3 Misfit, conditioning and diagnostics

**`SB-MIN-T13` — `RECON` equals the long-form incoherence computation.**
*Input:* a solved model whose tool weights differ by a factor of 100, so `LargestWeight ≠ 1`.
*Operation:* compute `RECON` from the shipped path and, independently in the test, from the long-form
σ-weighted RMS over live rows. *Expected:* agreement to `1e-9` relative. *Source:* Quanti.Elan Eq 79
and Eq 80, which differ only by a `LargestWeight` factor that cancels exactly; the test exists to keep
that cancellation true under refactor.

**`SB-MIN-T14` — comparable misfit statistics are emitted with convention labels.**
*Input:* any solve. *Operation:* read the outputs. *Expected:* `RECON`, an IP-comparable un-normalised
total error, and a Geolog-comparable `QUALITY`, each carrying a label naming the unity convention used
(hard equality vs `Tool` row). *Source:* SB-MIN-004 and SB-MIN-014; the statistic definitions are F-3.

**`SB-MIN-T15` — a collinear model is flagged, not answered.**
*Input:* two solid components whose endpoints differ by less than 0.5 % on every live tool, plus
water. *Operation:* solve. *Expected:* `CONDNUM > 10` and the samples flagged untrusted; the run does
not report a confident split between the two collinear components. *Source:* Geolog §H thresholds
(§5): `>8` suspect, `>10` unstable.

**`SB-MIN-T16` — degrees of freedom are counted and a zero-DOF fit is flagged.**
*Input:* (a) 4 live tools, 3 components, unity on; (b) 2 live tools, 3 components, unity on.
*Operation:* solve. *Expected:* (a) `dof = 2` and no note; (b) `dof = 0` and a note stating the
reconstruction cannot discriminate the model.
*Arithmetic:* (a) `4 + 1 − 3 = 2`; (b) `2 + 1 − 3 = 0`.
*Source:* SB-MIN-016. Already passing — `multimin2.rs:2412`, `:2487`.

### 6.4 Tool weighting and uncertainty

**`SB-MIN-T17` — deactivating a tool changes the DOF; a large σ does not.**
*Input:* a 4-tool model. *Operation:* (a) set one tool `active = false`; (b) instead set that tool's
`σ` to `1e6`. *Expected:* (a) `dof` falls by exactly 1 and the row is absent from the design matrix;
(b) `dof` is unchanged and the row is still present. *Source:* Techlog `-default-uncertainties.html`,
verbatim: *"To be totally out of the solution, the equation must be removed from the model."*

**`SB-MIN-T18` — the weight multiplier is separate from σ and is a no-op at 1.0.**
*Input:* a solved model. *Operation:* set one tool's multiplier to `1.0`, then to `0.25`, leaving `σ`
fixed. *Expected:* at `1.0` the volumes are bit-identical to the no-multiplier run; at `0.25` the
tool's residual grows and `σ` in the run record is unchanged. *Source:* Elan `xxxx_WM` semantics (§5).

**`SB-MIN-T19` — the printed default wins over the derived one.**
*Input:* the five tool rows where Elan's own table deviates from its 1.5 %-of-range rule (DT, CUDC,
CXDC, TPL, VELC). *Operation:* read the shipped default. *Expected:* the shipped value equals the
**printed** default, and differs from `0.015 × (MAX − MIN)` by 4–7 %. *Source:* Elan Tables 29/30.
This test exists because deriving instead of storing would silently disagree with the vendor on six
rows.

**`SB-MIN-T20` — the 1.5 %-of-range rule reproduces the nine rows it governs, and derived values are
labelled.**
*Input:* the nine Elan rows where the rule holds exactly (RHOB, NPHI, GR, U, SIGM, EATT, VOLS, PHIT,
ENPA). *Operation:* compute `0.015 × (MAX − MIN)`. *Expected:* agreement with the printed default to
0.5 % relative; any value SandiBumi ships from the rule rather than a table is flagged `DERIVED`.
*Source:* Elan Tables 29/30 and Geolog §E/§L — two independent vendors documenting one rule.

### 6.5 Saturation-adjacent inputs

**`SB-MIN-T21` — the conductivity root exponent is explicit and the three conventions differ
measurably.**
*Input:* `m = 2.2`, `n = 1.6`. *Operation:* compute the exponent under each supported convention.
*Expected:* Geolog `1/w = 0.487805 ± 1e-6`; Elan `0.500000`; IP-as-`1/m` `0.454545 ± 1e-6`; and the
run record names which was used.
*Arithmetic:* `w = 0.75 × 2.2 + 0.25 × 1.6 = 1.65 + 0.40 = 2.05`; `1/2.05 = 0.4878049…`;
`1/2.2 = 0.4545455…`. The two IP readings differ by `(0.5 − 0.454545)/0.5 = 9.09 %` — **larger than
the IP-vs-Geolog gap**. *Source:* F-9 / CT-3; both IP readings quoted verbatim from both editions.

**`SB-MIN-T22` — the Shell cementation constant has no default.**
*Input:* a Shell porosity-dependent `m` request with no constant supplied. *Operation:* solve.
*Expected:* refusal naming the parameter, and a UI surface listing `0.018`, `0.019` and `0.19` with
their sources.
*Arithmetic showing why it matters:* at `φe = 0.02` the 0.018 and 0.019 readings differ by
`0.019/0.02 − 0.018/0.02 = 0.95 − 0.90 = 0.05` in `m`, ≈5 % of the exponent.
*Source:* ledger D-10; `ABSENT` row in §5.

**`SB-MIN-T23` — variable `m*` reproduces the two-vendor-corroborated coefficients.**
*Input:* `Qv = 0.5 meq/mL`, `φT = 0.20`, `Cm = 1`, `m = 2.0`. *Operation:* Dual-Water `m*`.
*Expected:* `m* = 2.206503 ± 1e-5`.
*Arithmetic:* `Y = 0.5 × 0.20/(1 − 0.20) = 0.125`; `0.258 × 0.125 = 0.032250`;
`16.4 × 0.125 = 2.05`, `e^(−2.05) = 0.1287349`, `1 − 0.1287349 = 0.8712651`,
`0.2 × 0.8712651 = 0.1742530`; `m* = 2.0 + 0.0322500 + 0.1742530 = 2.2065030`.
*Source:* IP E68 and Elan Eq 65/66 — bit-identical coefficients. The Waxman-Smits variant is asserted
present **and** labelled single-sourced.

### 6.6 Clay density bookkeeping

**`SB-MIN-T24` — the wet↔dry conversion responds to the bound-water density.**
*Input:* `ρ_wcl = 2.52 g/cc`, `WCLP = 0.17`, with `ρ_bw = 1.00` then `ρ_bw = 1.05`. *Operation:*
`dry_clay_calc`. *Expected:* `2.831325 ± 1e-6` then `2.821084 ± 1e-6` — a difference of
`0.010241 g/cc`, matching the dossier §2.6 figure of 0.010 g/cc.
*Arithmetic:* `(2.52 − 0.17 × 1.00)/0.83 = 2.35/0.83 = 2.8313253`;
`(2.52 − 0.17 × 1.05)/0.83 = 2.3415/0.83 = 2.8210843`.
*Source:* Elan Eq 11 generalised with an explicit bound-water density. **This test cannot pass today**
— `RHO_W` is a function-local constant (`multimin2.rs:676`–`:699`).

**`SB-MIN-T44` — the clay density triple is gated for self-consistency.**
*Input:* Techlog's own shipped triples. *Operation:* evaluate
`ρ_dcl = (ρ_wcl − WCLP·ρ_bw)/(1 − WCLP)` and compare with the tabulated `ρ_dcl`. *Expected:* Illite
`2.831325` against `2.70` → `+4.86 %`, **warned**; Kaolinite `2.516129` against `2.65` → `−5.05 %`,
**warned**; Chlorite `2.7742` against `2.80` → `−0.92 %`, **warned** (above the 1 % gate).
*Arithmetic:* `(2.41 − 0.07)/0.93 = 2.34/0.93 = 2.5161290`.
*Source:* Elan Eq 11 applied to Techlog `QM_MineralTable.xml`. The expected values are the vendor's
own inconsistency, transcribed only to prove the gate fires — SandiBumi ships none of them.

### 6.7 Model inputs and units

**`SB-MIN-T25` — a per-equation invasion factor selects the fluid mix.**
*Input:* a gas model with the neutron at `IF = 0.5` and every other tool at `IF = 1.0`. *Operation:*
solve. *Expected:* the neutron row's fluid coefficients are the 50/50 mix of the X and U sets; every
other row is unchanged; the run record carries the per-equation factor. *Source:* SB-MIN-025.

**`SB-MIN-T26` — the neutron response set is recorded and is not inherited from a header.**
*Input:* two runs identical except for the neutron response set. *Operation:* solve both. *Expected:*
the volumes differ, both run records name the set used, and neither reads it from a well-header
field. *Source:* F-11, verbatim: *"A single header dropdown therefore silently changes numerical
results."*

**`SB-MIN-T27` — a p.u. WCLP refuses; it does not become a CEC answer.** **[P0]**
*Input:* `WCLP = 10.4`. *Operation:* compute the bound-water multiplier. *Expected:* refusal naming
the expected unit `m³/m³ (v/v)` and the likely p.u. entry. *Control:* `WCLP = 0.104` returns
`k = 0.116071 ± 1e-6` (`0.104/0.896`). *Source:* Techlog `WCLP_*_unit = u"m3/m3"` (T1, §5).
**This test fails today** — the value falls through the ceiling branch to `cec_k` with no message
(`multimin2.rs:623`–`:624`).

**`SB-MIN-T28` — the endpoint library is selectable and disagreements surface.**
*Input:* the same model under two supported libraries. *Operation:* solve both. *Expected:* the run
record names the library; any value differing by more than 5 % relative between libraries is surfaced
with both values and both sources at the point of selection. *Source:* F-14's own agreement criterion
(relative spread ≤ 5 %).

**`SB-MIN-T29` — the gas sonic endpoint shows its source and its alternatives.**
*Input:* a gas component. *Operation:* open the endpoint editor. *Expected:* `DT = 250.0 µs/ft` shown
with its source library, and the cross-vendor alternatives shown because the 45 µs/ft spread exceeds
the sonic tool's own default uncertainty. *Source:* F-5.

**`SB-MIN-T30` — the Elan-form Simandoux reduces correctly and is not mislabelled.**
*Input:* the Elan Eq 78 form with `V_silt = 0`. *Operation:* evaluate. *Expected:* the result equals
the same equation's no-silt reduction to `1e-9` relative, and the equation's displayed name
distinguishes it from IP's compact E63/E64 form. *Source:* Elan Eq 78 with Table 28 defaults
`ersh = 1.0`, `swshe = 0.5`.

**`SB-MIN-T31` — a per-clay shale resistivity is used where supplied.**
*Input:* a two-clay model with different `Rsh` on each clay in one zone. *Operation:* solve.
*Expected:* both values are used; removing one falls back to the zonal value. *Source:* SB-MIN-031.
The Techlog values in §5 are cited to establish that the capability is real (a 7:3 ratio across two
clays), not to be adopted.

**`SB-MIN-T42` — solved volumes are invariant to the input unit system.**
*Input:* one well's curves in metric and the same curves in imperial. *Operation:* solve both.
*Expected:* every volume agrees to `1e-6 v/v`; every endpoint is converted at the request boundary.
*Source:* SB-MIN-044; the four unit traps it guards are the `96`/`0.096`, `0.1883`/`188.3`,
`m³/m³`/p.u. and `meq/g`/`meq/100 g` rows in §5.

**`SB-MIN-T43` — rejected formation-temperature samples are counted and reported.**
*Input:* an FTEMP curve in which 30 of 100 samples are `−999.25`. *Operation:* solve. *Expected:* the
30 samples use the constant temperature, the run reports `30` substitutions, and the reported count is
non-zero in the result object rather than only in a log line. *Source:* SB-MIN-045; the window is the
`FTEMP_MIN_F`/`FTEMP_MAX_F` row in §5.

### 6.8 Constraints, outputs and reproducibility

**`SB-MIN-T32` — a run replays bit-for-bit from its persisted parameter set.**
*Input:* any solve. *Operation:* persist the resolved parameter set, then re-run from it alone.
*Expected:* every output curve is bit-identical. *Source:* SB-MIN-032.

**`SB-MIN-T33` — an infeasible constraint set names its conflicting rows.**
*Input:* a model with `PHIMAX = 0.10` and a `BVIRR` floor of `0.15`. *Operation:* solve. *Expected:*
failure naming both constraint rows and the depth. *Source:* SB-MIN-033; Geolog's conflict check is
the precedent (F-13).

**`SB-MIN-T34` — the water-mud constraint is an inequality, not an equality.**
*Input:* (a) a case whose unconstrained solve gives `Sxo < Sw`; (b) a case whose unconstrained solve
gives `Sxo > Sw` by a wide margin. *Operation:* solve. *Expected:* (a) the corrected solution
satisfies `Σ(X waters) ≥ Σ(U waters)` and `MOVEDHC ≥ 0`; (b) the solution is **unchanged** — the
constraint does not pull `Sxo` toward `Sw`. *Source:* the physical statement in the module's own doc
(`multimin2.rs:455`), *invasion ⇒ `Sxo ≥ Sw`*. **Case (b) fails today** — the appended row is a
σ-weighted equality at RHS 0 (`multimin2.rs:1409`–`:1410`).

**`SB-MIN-T35` — `Tool` constraints hold hard, and the tie residual is emitted.**
*Input:* a model whose data conflict with the porosity tie. *Operation:* solve. *Expected:* the
porosity tie holds to `1e-9` (not merely to `σ = 0.01`), the tie residual is emitted as a QC curve,
and the constraint contributes to both `dof` and the misfit statistic. *Source:* Geolog's `Tool` class
definition (F-7) and dossier Δ-1.

**`SB-MIN-T36` — the oil-mud and opt-in ceiling constraints behave.**
*Input:* (a) an OBM model whose unconstrained solve gives `v_Xgas > v_Ugas`; (b) the same model with
`PHIMAX = 0.25` enabled. *Operation:* solve. *Expected:* (a) `v_Xgas ≤ v_Ugas` in the solution;
(b) `PHIT ≤ 0.25` and the run record names `PHIMAX` as enabled with its value. *Source:*
adoption-spec F-4.

**`SB-MIN-T37` — the output set is complete and carries no bare `SW`.**
*Input:* any solve. *Operation:* enumerate output curve names. *Expected:* `SXOE`, `PHIE_X` and
`PHIT_X` are present alongside `SXOT`, `PHIE` and `PHIT`; no curve is named exactly `SW`; every clay
or shale volume curve carries a wet/dry declaration. *Source:* adoption-spec F-11 and ledger D-15.
The `SW` half already passes.

**`SB-MIN-T38` — Monte Carlo is reproducible from its seed.**
*Input:* the same model and seed twice, then a different seed. *Operation:* run the endpoint
uncertainty propagation. *Expected:* identical percentile bands for the repeated seed, different bands
for the different seed, and the seed present in the run record. *Source:* SB-MIN-037. IP's own
behaviour — *"seeded through the CPU clock time"* — is the anti-pattern, not the reference.

**`SB-MIN-T39` — the per-volume uncertainty is emitted with its caveat.**
*Input:* a solved over-determined model. *Operation:* read the per-volume uncertainties. *Expected:*
each is `≥ 0`, scales linearly with the misfit statistic, and is labelled as a linearised estimate
excluding endpoint uncertainty. *Source:* Geolog `sqrt(diag(A⁻¹))·QUALITY` (F-12).

**`SB-MIN-T40` — a retired module resolves by name and refuses to run.** **[P0]**
*Input:* a saved chain step naming `multimin`. *Operation:* `list_modules`, then `run_module`.
*Expected:* the spec is present in the catalogue; `run_module` returns `Err` whose message names the
replacement (`SandiMin`); a live module name is not flagged retired. *Source:* SB-MIN-041. Already
passing — `modules.rs:2856`–`:2862`.

**`SB-MIN-T41` — a retired module exposes no unshared endpoint default.** **[P0]**
*Input:* the retired `multimin` spec. *Operation:* compare each endpoint default against the live
library. *Expected:* every rendered endpoint default either matches a live library value or is
labelled a historical value of a retired method. **This test fails today** on at least `RHOB_CLAY 2.55`
(`multimin.rs:32`) against the live generic `Clay` RHOB `2.65`, and `PEF_CLAY 3.10` (`:44`) against
`3.50`. *Source:* SB-MIN-041, `SB-CORE-004`.

---

## 7. Open items, escalations and refusals

Four labelled lists. **§7.1 Escalations** — decisions no agent on this project may take: parameter
adjudications where the vendors disagree and no source arbitrates, and questions needing Jauhar's
method judgement or a live vendor session. **§7.2 Acquisition gaps** — a primary source that would
close an item and is not held on this machine. **§7.3 Open items** — needed, answerable here, not yet
done. **§7.4 Refusals** — what SandiBumi deliberately will not do, and what this chapter declined to
transcribe, with the rule cited.

`ESC-1` … `ESC-8` are the identifiers already cited from §2, §3 and §5; their numbering is fixed by
those references and is not re-ordered here.

### 7.1 Escalations — Jauhar's call, or a live vendor session

#### ESC-1 — The conductivity root exponent: `^(1/2)` or `^(1/m)` inside IP

**Unresolved.** IP's manual states the pre-solver transform of `Crv`, `Crv_Rec` and `Crv_Tol` two
ways — *"square root taken of them"* and *"has 1/m th root taken of it"* — in **both** editions, and
both ingest reports faithfully carry both (dossier §2.2, proposed ledger item CT-3). The two
passages govern the same object: `Crv_Tol` *is* the confidence. The vendor's own worked example
(500 mmho, 5 mmho confidence, → 22.3 / 2.24) is printed at `m = 2`, where `1/m = ½` exactly, so it
**cannot arbitrate**.

**Competing values, with sources.** At `m = 2.2, n = 1.6`: Elan **0.500** (T2-equivalent,
`-elan-theory-uncertainties.html` plus Table 29's `CUDC_UNC` MAX of `√20.0`, doubly stated);
Geolog **0.4878** (`1/w`, `w = 0.75m + 0.25n`, T3 `multimin_ref_spec.md` §E/§L); IP **either 0.500 or
0.4545** (T2, both readings verbatim from both editions). That is a **9.1 % spread inside one
vendor**, larger than the IP-vs-Geolog gap. At `m = 2.0, n = 1.6` the Geolog-vs-IP gap is 5.3 % and
survives either reading.

**What would settle it.** One live IP 2025 run, read-only: a one-row conductivity model, `m = 2.5`,
feed 500 mmho, read the value the solver receives (or reconstruct it from the ± error bounds the
confidence panel prints). `500^(1/2) = 22.36` proves the square root; `500^(1/2.5) = 13.20` proves
`1/m`. The two differ by 41 %, so one significant figure decides it. This is dossier escalation E-10,
and it is ten minutes on `C:\Program Files\IP2025`.

**What the chapter did meanwhile.** `SB-MIN-021` [P1] makes the exponent an explicit named model
input, requires the value actually used to be recorded in the run record, requires support for all
three conventions, and forbids silently picking one. §5 carries `w = 0.75m + 0.25n` as the shipped
Geolog default with the alternatives named. **No IP-parity claim on a conductivity row at `m ≠ 2` is
permitted until this closes**, which is asserted by `SB-MIN-T21`. It ships ABSENT of any IP-parity
default pending his call.

#### ESC-2 — Which vendor's `(CEC, WCLP)` pair the shipped clay library adopts

**Unresolved.** `CEC` and `WCLP` are two parameterisations of one quantity, `k = V_bw/V_dryclay`
(F-22). Each vendor's own pair is self-consistent to ≤ 1.5 %; SandiBumi ships Geolog's CEC column
against Techlog's WCLP column and the two routes disagree by **+58.6 % on Illite**. `SB-MIN-008` [P0]
requires a matched pair — but **which library becomes the pair is a parameter adjudication, and no
source arbitrates it.**

**Competing values, with sources.**

| Clay | Geolog RF04 6.2 (T3) | Techlog `QElan_PostProcess_Using_Conductivities.py` (T1) | Spread on CEC |
|---|---|---|---|
| Illite | CEC 0.25 · WCLP 0.1555 (pair to 0.01 %) | CEC 0.16 · WCLP 0.104 (pair to 1.5 %) | **56 %** |
| Kaolinite | CEC 0.10 · WCLP 0.06489 (pair to 0.02 %) | CEC 0.09 · WCLP 0.058 (pair to 1.4 %) | 11 % |
| Chlorite | CEC 0.15 · **WCLP not held** | CEC 0.15 · WCLP 0.101 | 0 % |
| Glauconite | CEC 0.20 · **WCLP not held** | CEC 0.233 · WCLP 0.156 (pair to 1.2 %) | 16.5 % |

**Consequence in petrophysical units.** On Illite at `T = 64.4 °C, α = 1`: Geolog's pair gives
`k = 96 × 0.25 × 2.78/362.4 = 0.184106`, Techlog's gives `96 × 0.16 × 2.78/362.4 = 0.117828`. At
`V_dryclay = 0.25` that is `V_bw` **0.046027 against 0.029457 v/v — 1.66 pu of PHIE, and SWE by the
same amount under unity**, moved purely by which vendor's library the row came from. (The shipped
chimera is a separate and slightly larger 1.70 pu, quantified in §3.4 — that one is a defect and is
closed by `SB-MIN-008` regardless of this adjudication.)

**Second half of the same escalation.** Geolog RF04 6.2's WCLP is **not held on this machine for
Chlorite or Glauconite** — `multimin_ref_spec.md` §B states only the Illite and Kaolinite pairs. So
"adopt Geolog wholesale" cannot be completed for two of five clays without either OPEN-1 (ingest
`Multimin_Knowledge_Transfer.pdf`) or shipping those two WCLP values ABSENT.

**What would settle it.** Jauhar's ruling only. Two vendors, two internally-consistent libraries,
no third source that adjudicates between them, and the dossier's own §4 item 17 recommends shipping
**selectable libraries** rather than picking — which is `SB-MIN-028`, but a solver must still have a
default.

**What the chapter did meanwhile.** §5 carries every competing value with its source and tier;
Chlorite and Glauconite Geolog-pair WCLP ship **`ABSENT — ships with no default`**; the three
competing Techlog CECs are marked **`NON-ADOPTABLE — cited for verification`**. The shipped library
stays `PRESENT-DIVERGENT` and `SB-MIN-008` gates it at 2 % relative. **No agent picked a library.**

#### ESC-3 — The water-mud constraint: iteration policy, cap, and what happens at the cap

**Unresolved.** `SB-MIN-034` [P1] requires the WBM constraint be a hard inequality
`Σ(X waters) − Σ(U waters) ≥ 0` iterated to feasibility. Three things about that iteration are
method judgements with no vendor precedent worth copying: the iteration cap, the behaviour on
reaching it (fail the sample, or return the last feasible iterate and flag it), and the interaction
when WBM fights an opt-in `PHIMAX` or `BVIRR` (`SB-MIN-043`). Geolog and Elan have no iteration at
all — the inequalities live inside the QP; IP's pattern is *"add one violated limit at a time, in
grid order, and completely re-solve"*, which exists precisely to avoid the oscillation, and IP's own
manual concedes the result can still fall outside the limit.

**What would settle it.** A constructed two-constraint fixture — WBM plus `PHIMAX` — which needs
**no vendor source and is resolvable inside this repository** (dossier escalation E-8, explicitly
"a SandiBumi-internal escalation"); plus his ruling on the cap semantics. The fixture is not written
because `SB-MIN-034`'s inequality form does not exist yet to test against.

**What the chapter did meanwhile.** `SB-MIN-034` states the inequality, requires the iteration and
requires that reaching the cap be reported; **§5 allocates no cap value** — deliberately, since one
would be an invented number. `SB-MIN-T34` pins the two cases the current code gets wrong: case (a)
the corrected solution must satisfy the inequality, case (b) a solution already satisfying it must
be **unchanged** (which fails today, because the appended row is a σ-weighted equality at RHS 0,
`multimin2.rs:1409`–`:1410`, pulling `Sxo` toward `Sw` and suppressing genuine `MOVEDHC`).

#### ESC-4 — The Shell porosity-dependent `m` constant (ledger D-10)

**Unresolved.** The same named constant appears as **0.018, 0.019 and 0.19** across two vendors.

| Value | Source | Tier |
|---|---|---|
| **0.018** | IP 2025 raster `embim275.png`, verified 4×; IP 2018 raster `c18/embim284.gif`, verified 6× | T2 |
| **0.019** | IP 2025 ASCII `mineral_solver.htm`; IP 2018 ASCII — same page, same manual | T2 |
| **0.19** | Techlog Elan Eq 78 `mc2`, *"The usual value assumed for mc2 is 0.19"*; Table 28 default **0.0** | T2-equivalent |

**Consequence in petrophysical units.** At `φe = 0.10`, `m` is 2.05 (0.018) against 2.06 (0.019); at
`φe = 0.02`, 2.77 against 2.82 — a ~5 % exponent difference propagating to **~5–10 % Sw error in
tight rock**. The 0.19 is only comparable at all if Elan's `φₑ` is p.u.: at 3 p.u. it adds `+0.063`,
but read fractionally `0.19/0.03 = 6.33`, which is not a cementation exponent. **That unit is
unstated by the vendor** and is the subject of OPEN-3.

**What would settle it.** (a) The published Shell source — **which the corpus never names**, so it is
also acquisition gap ACQ-4; the IP 2025 ingest report's own leaning (*"the published Shell formula
uses 0.019, so the ASCII agrees with the literature and the raster does not — but the manual states
both and this report does not pick a winner"*) is a third-party reading of the literature, not a
citation, and does not close it. (b) One live IP run: set `m source = Shell`, `φe = 0.02`, read the
emitted `mVar` — **2.77 proves 0.018, 2.82 proves 0.019**. The ledger already assigns this decision
to Jauhar.

**What the chapter did meanwhile.** `SB-MIN-022` [P2] requires the constant ship
**`ABSENT — ships with no default`**, requires an explicit user value, and requires the UI display
all three competing values with their sources at the point of entry. §5 carries it as ABSENT.
`SB-MIN-T22` asserts the refusal and the three-value display.

#### ESC-5 — Two shipped constants with no primary source

**Unresolved.** Two values the solver ships today carry no primary-literature provenance:

1. **The auto conductivity-uncertainty fraction `0.03`** (`multimin2.rs:597`–`:598`,
   `u_ct = 0.03·cw^(1/w)`). The **vendor rendering is held** — Geolog `multimin_ref_spec.md` §E/§L
   states `U_Ct = 0.03·Cfw^(1/w)` and `U_Cxo = 0.03·Cmf^(1/w)` (T3) — but nothing traces it to a
   publication. **Note a §5 defect while recording this: that row's `Source` cell reads "primary
   source NOT ESTABLISHED" and omits the Geolog rendering that does exist; the correct label is
   `VENDOR-DERIVED` with the Geolog §E/§L citation. This chapter is append-only and did not edit
   §5 — the fix belongs to the next revision (see OPEN-8).**
2. **The `Rw` → NaCl-equivalent salinity fit** `10^((3.562 − log₁₀(Rw₇₅ − 0.0123))/0.955)`
   (`multimin2.rs:548`–`:554`). The vendor rendering is held: it is IP's E4, carried in the dossier
   as regression fixture T-1 (`C_mineral_solver.md` §2.3 E4 with `_imsclip0009.png`, round-tripping
   `Rmf = 0.1 @ 60 °F` to the vendor's printed **87.8 Kppm**). The primary source — the chart fit
   those coefficients digitise — is not held.

**Consequence in petrophysical units.** The salinity fit drives `α = sqrt(20455/S)`, which multiplies
bound water directly (`SB-MIN-006`). At the 8,000 ppm case, `α = 1.59902`; a 10 % salinity
error (8,000 → 8,800 ppm) gives `α = 1.52461`, **−4.65 %**, moving `V_bw` from 0.0670 to 0.0639
bulk v/v — **0.31 pu of PHIE**, in the direction the interpreter never sees because the salinity was
itself derived.

**What would settle it.** The primary source for the Arps-family `Rw`↔salinity conversion, and for
Geolog's 0.03 fraction. Absent that, an explicit ruling that both ship `VENDOR-DERIVED`.

**What the chapter did meanwhile.** Both are shipped as coded and recorded in §5 without a primary
source. `SB-MIN-009`'s per-value provenance requirement — and `SB-CORE-004`'s build gate — will fail
both rows until they are labelled. The Arps temperature constant `6.77` beside them **is** primary
(Arps 1953) and is labelled so, which is the contrast that makes the two unlabelled rows visible.

#### ESC-6 — Techlog's clay density triples fail Techlog's own equation

**Unresolved.** Applying Elan Eq 11 to Techlog's own shipped `(Rhobwcl, Phicl, Rhobdcl)` triples:
Illite `(2.52 − 0.17)/0.83 = 2.8313` against the tabulated 2.70 (**+4.9 %**); Kaolinite
`(2.41 − 0.07)/0.93 = 2.5161` against 2.65 (**−5.1 %**); Chlorite 2.7742 against 2.80 (−0.9 %).
**No vendor statement explains the gap** (dossier G-5). Two readings, both live: the table is a
*curated* default library whose three columns were each chosen independently, or it is a
shipped-data defect.

**Consequence in petrophysical units.** A 30 % clay rock solved with `ρ_dcl = 2.8313` where the table
says 2.70 puts `0.30 × 0.1313 = 0.0394 g/cc` into the RHOB row — **1.97 σ at IP's 0.02 g/cc
confidence**, bought back as `0.0394/(2.65 − 1.00) = 0.0239 v/v = 2.4 pu of porosity`.

**What would settle it.** A Techlog vendor statement — not present anywhere in the corpus — or
Jauhar's ruling on two things: whether `SB-MIN-046`'s gate should **warn** (as written) or **refuse**,
and whether Techlog's triples may seed a SandiBumi library at all given that they are internally
inconsistent by more than the gate.

**What the chapter did meanwhile.** `SB-MIN-046` [P2] requires the three-way check with a 1 %
relative gate and a warning naming the row and the discrepancy, and forbids storing the three as
silently independent values. **No §5 row adopts a Techlog triple**; the three values appear only in
F-15 and in `SB-MIN-T44`, whose expected outputs are the vendor's own inconsistency transcribed
solely to prove the gate fires. SandiBumi ships none of them.

#### ESC-7 — Whether a per-clay shale resistivity is a capability IP lacks

**Unresolved.** Techlog attaches `Rsh` to the clay **mineral** (Illite 3, Chlorite 5, Shale 5,
Kaolinite 7 ohm-m — a 7:3 ratio across two clays in one table, T1 `QM_MineralTable.xml`); IP takes
`Rcl` as a **zonal** parameter inside Simandoux E63/E64; Geolog carries no per-clay resistivity at
all. Whether IP's `Rcl` can be made per-clay in a live session is **not answerable from the manual**
(dossier E-9).

**Why it is an escalation rather than an open item.** It is a claim boundary, not a number. If IP
cannot do it, `SB-MIN-031` is a genuine capability IP lacks and may be said so in a deliverable; if
IP can, saying so is an overclaim. PRD v1 §6 is not negotiable here: an admitted gap costs a feature,
a discovered overclaim costs the deal.

**What would settle it.** One live IP 2025 session: open a multi-clay Simandoux model and check
whether `Rcl` is addressable per clay or only per zone.

**What the chapter did meanwhile.** `SB-MIN-031` [P3] requires the **capability** and says nothing
about what the incumbents can do. §5 marks the four Techlog `Rsh` values and `XWater`
**`NON-ADOPTABLE — cited for verification`** — they establish that the capability is real, and
SandiBumi ships none of them.

#### ESC-8 — The hard-coded `VP/VS = 1.7` in the endpoint builder (belongs to `RPH`)

**Unresolved.** `multimin2.rs:2116`–`:2117` derives `VP = 304.8/DT` (a pure unit conversion,
`1 ft = 0.3048 m`, correct and sourced) and then `VS = VP/1.7` for **every non-fluid component**.
The 1.7 has no source string anywhere in this domain's corpus. Arithmetically it is one Poisson
ratio — `ν = (r² − 2)/(2r² − 2) = 0.89/3.78 = 0.2354` — applied identically to quartz, calcite,
dolomite and every clay.

**Consequence in petrophysical units.** The `VS` endpoint scales as `1/r`, so a component whose true
ratio is `r′` has its endpoint wrong by `(r′ − 1.7)/r′`, entering the `VS` row linearly by volume.
**The magnitude cannot be bounded here, because bounding it requires a sourced per-mineral `VP/VS`
or Poisson set — which is exactly what is absent.** Stating a per-mineral value from general
knowledge is the one thing this chapter must not do.

**What would settle it.** A per-mineral `VP/VS` or Poisson set from `RPH`'s own primary sources (the
Greenberg-Castagna family, already examined in the 18-dossier cross-tool crosscheck), adjudicated in
that chapter, not this one.

**What the chapter did meanwhile.** §5 ships `VP/VS` as **`ABSENT — ships with no default`** while
recording that the code carries 1.7, so `SB-MIN-009`'s gate fails the row rather than blessing it.
§1 declares the seam to `RPH` and allocates no number there.

### 7.2 Acquisition gaps — a primary source not held on this machine

Each entry names the source precisely and states what it would close. None is required to ship a
requirement in §4; each would move a value from `VENDOR-DERIVED` to primary, or close a ledger item.

**ACQ-1 — Mayer, C. & Sibbit, A., "GLOBAL, a new approach to computer-processed log interpretation",
SPE 9341, SPE Annual Technical Conference, 1980.** The root citation for the entire simultaneous
log-response inversion family — and SandiMin's own module header already cites it
(`multimin2.rs:24`–`:29`) while **no ingest report and no earlier draft of the dossier named it**
(dossier P-6). It would settle the one question §2's F-3 cannot answer from the manuals: whether the
½ in Elan Eq 79 and the χ² normalisation in Geolog's `QUALITY` descend from one parent statistic or
are independent vendor choices. Closes the provenance of `SB-MIN-013` and `SB-MIN-014`.

**ACQ-2 — Hill, H. J., Shirley, O. J. & Klein, G. E. (1979), SPWLA 20th Annual Logging Symposium,
Paper AA.** `DISCREPANCIES.md` D-07 names it as the **only** admissible resolution for IP's malformed
clay-bound-water relation `F = 1 − [0.6425 · (Salinity^(−0.5) + 0.22] · Qv]`, which is byte-identical
in both IP editions and has two grammatically reachable parses that differ materially in the computed
bound water. Until it is held, **SandiBumi ships neither parse** (REF-6) — and no §4 requirement
offers that route.

**ACQ-3 — Clavier, C., Coates, G. & Dumanoir, J., dual-water.** Both Elan (`0.28 cm³/meq` at room
temperature) and Geolog (`0.096·ρ/(T + 298)`, shipped in g/cc form as `96`) are renderings of one
relation, agreeing to within 6 %. SandiBumi ships the Geolog rendering as its **P1, `PRESENT-OK`**
bound-water route (`SB-MIN-006`) on a **T3** source. The original fixes the temperature form, the
`(T + 8.5)` term (dossier P-4) and the 20,455 ppm / 0.35 mol/L diffuse-layer threshold. **This is the
highest-value acquisition in the domain**, because it is the only one that upgrades a shipping P1
requirement's evidence tier rather than closing an unimplemented option.

**ACQ-4 — The published Shell porosity-dependent-`m` paper.** *The corpus never names it* — which is
itself the gap. Both IP editions print the relation `m = 1.87 + k/φe` with no citation, and the IP
2025 ingest report's appeal to "the published Shell formula" names no author, journal or year. It
would close ledger D-10 / ESC-4 outright.

**ACQ-5 — Segesman, F. & Liu, O. (1971), the neutron excavation effect.** Named by Geolog
(`multimin_ref_spec.md` §E) and by the Schlumberger *Log Interpretation Volume I — Principles*
lineage Elan cites. It would say whether IP's `(ρma/2.65)` factor or its exponent is the innovation
(ledger D-09, where IP's Mineral Solver squares it and IP's own SSC/Malay module square-roots it, and
Elan has no matrix term at all). Needed only if an excavation term ships — none does today.

**ACQ-6 — The Simandoux lineage, four references named on Elan's own page:** Simandoux (1963);
Schlumberger, *Log Interpretation Principles* (1972); Worthington (1985) — the Type-1/Type-2
classification; Poupon et al. (1967). Plus the **SPWLA Shaly Sand Reprint**, cited by name but not as
a numbered reference. Required before `SB-MIN-030`'s Elan-form Simandoux ships, so that "ELAN
Simandoux" carries the provenance chain that distinguishes it from IP's uncited compact form.

**ACQ-7 — IP's own three Sw citations, needed by `SAT` not by this chapter:** Poupon, A., Loy, M. E.
& Tixier, M. P. (1954), *"A contribution to electric log interpretation in shaly sands"*, Trans AIME
6(06):138–145; Aguilera, R. (1990), *"Extensions of Pickett Plots for the Analysis of Shaly
Formations by Well Logs"*, The Log Analyst, Sept–Oct; Woodhouse, R. (1976), *"Athabasca Tar Sands
Reservoir Properties Derived from Core and Logs"*, 17th SPWLA Logging Symposium. Recorded here
because the dossier holds them (P-2b) and because the domain boundary — Techlog documents none of
these three equations — is a finding `SAT` should inherit rather than rediscover.

**ACQ-8 — Goldfarb, D. & Idnani, A. (1983), dual QP; Powell, M. J. D. (1985), VF02 SQP.** Geolog
names both as its solver engines, and `SB-MIN-001`'s rationale cites the family as evidence that two
of three vendors plus the optimisation literature are on the side of bounded optimisation.
SandiBumi's `solve_bounded_lsq` is an active-set KKT method in that class
(`multimin2.rs:1841`–`:2006`). The papers are not held; the citation is currently second-hand
through a T3 helpset ingest.

**ACQ-9 — Elan Table 31 (geochemical uncertainties), rasters `image2978/2979.gif`.** Present on this
machine but **unread**; listed here rather than under OPEN because it is only needed if
ECS/spectroscopy rows ship, and none do (dossier T-3).

**ACQ-10 — The Vmatrix-vs-Umatrix crossplot construction behind Elan's sonic clay-volume
constraint** (Elan Eqs 89–92 are self-contained; the construction in `image2992.gif` is
**unattributed**, and the dossier's P-5 judges it likely Schlumberger internal — i.e. it may not be
publicly obtainable at all). Needed only if the Sonic Clay Volume predefined constraint ships; it is
the one member of Elan's seven that `SB-MIN-043` does not offer, for exactly this reason.

### 7.3 Open items — answerable on this machine, not yet done

**OPEN-1 — The Geolog leg is T3-only and one ingest would change that.**
`D:\01. Work\00. Guidebook\02. Guidebook Geolog\Multimin\Multimin_Knowledge_Transfer.pdf` and four
Multimin video lessons are held and **not ingested** (dossier G-1). Multimin is a compiled module, so
no T1 Geolog evidence exists or can exist here — but the PDF would raise several §5 rows above T3 and
may supply the two missing Geolog WCLP values that block half of ESC-2. A straightforward
`petro-source-extractor` job. **Recommend before any further Geolog-parity work.**

**OPEN-2 — Elan's neutron response tables are unread.** `image2893/2894/2895/2896.gif` (Table 27
region) plus `-neutron-matrix-term.html` and `-neutron-fluid-term.html` (dossier T-1). Would give a
third independent neutron matrix model against IP's `.neu` tables and Geolog's
`φT = a + b·φN + 10^(c + d·φN)` fits. Bears on `SB-MIN-026` and hands off to `ENV`.

**OPEN-3 — Elan's *global* p.u.-vs-v/v convention.** The **WCLP half is now closed** by this chapter
on T1 evidence — `QElan_PostProcess_Using_Conductivities.py` declares `WCLP_*_unit = u"m3/m3"`,
which is what `SB-MIN-027` ships. **The global `φₑ` half is not**, and it gates whether Elan's
`mc2 = 0.19` is even comparable to IP's 0.018/0.019 (ESC-4). Read
`petrophysics-elanplus-conventions.html`, `petrophysics-elan-theory-glossary.html` and
`petrophysics-elanplus-parameter-tables.html` — all present locally (dossier T-2).

**OPEN-4 — Four Elan default-parameter tables unread:** Table 25 dual-water (`image2942.gif`),
Table 26 linear-conductivity (`image2952.gif`), Table 32 sonic-clay-constraint (`image3000.gif`), and
the predefined-constraint forms `image2985–2988.gif` (dossier T-4). Needed only when the matching
options ship.

**OPEN-5 — `QUANTI_INVERSION_CONSTANTS.xml`, a fourth Techlog endpoint table no ingest has touched**
(dossier T-5, `petrophysics-inversion-constants.html`). Bears directly on `SB-MIN-028`'s library
roster: the chapter currently believes there are three libraries, and there may be four.

**OPEN-6 — Elan's invasion model and `EQHY`.** `-elanplus-invasion-model.html`,
`-oil-gas-model-with-rxo.html`, `-oil-gas-model-without-rxo.html` (dossier T-6). Bears on
`SB-MIN-025`; Geolog's `EquivFluids` (uncertainty 0.01) is probably the same object as Elan's Equal
Hydrocarbon Ratio and that is **unverified**.

**OPEN-7 — Six live-install reads, minutes each on `C:\Program Files\IP2025`, read-only.**
(a) dossier E-2 — the `MINDEF.PAR` Qv/CEC column header, which converts §2.8's meq/g inference into a
vendor statement and hardens `SB-MIN-011`'s unit declaration; (b) E-3 — the identity of `MINDEF.PAR`
rows `XPL 2.59` and `XAN 2.74`, unlabelled and deliberately not guessed; (c) E-4 — IP's full mineral
drop-down roster and its smectite endpoint set; (d) E-5 — `MINEQDEF.PAR` verbatim, to confirm the
IP-parity uncertainty column lost nothing in transcription; (e) E-6 — IP's non-negativity behaviour
on a marginal mineral, confirming the column is genuinely *dropped* rather than clamped, which is the
factual basis of `SB-MIN-001`'s and `SB-MIN-002`'s divergence claim; (f) E-7 — the `Sal` unit in
`U_wat = 0.00481·Sal + 0.3883`, currently a strong kppm inference from physical bounds.

**OPEN-8 — This chapter's own front-matter counts are stale.** The front matter states **34**
acceptance tests against the **44** written (`SB-MIN-T01` … `SB-MIN-T44`), **63** parameter rows
against the **78** counted at the foot of §5, and **9** P0 against the **10** allocated in §4
(`SB-MIN-003`, `-007`, `-008`, `-009`, `-010`, `-011`, `-012`, `-016`, `-027`, `-041`). The
requirement count (46) is correct. This task was scoped append-only and did not edit the front
matter; it is a documentation defect for the next revision, recorded here so it is not read as a
finding about the solver.

**OPEN-9 — `SB-MIN-T32`'s "bit-identical" replay has no defined tolerance across toolchains.** The
test asserts bit-for-bit reproduction from the persisted parameter set (`SB-MIN-032`). Whether that
survives a change of LLVM target, optimisation level or floating-point contraction is untested, and
a replay guarantee that silently means "same machine, same build" is weaker than the requirement
reads. Settled by running `SB-MIN-T32` across two toolchains and either tightening the build flags
or restating the guarantee.

**OPEN-10 — Dossier test T-9 has no counterpart in §6, and that is deliberate.** T-9 requires that a
model mixing IP's single-clay bound-water convention (`0.15 / −0.85`) with its multi-clay convention
(`φ/(1−φ) / −1`) fail validation — the C-5.3 vendor-example defect. **SandiBumi implements neither
form**: it has no `BoundWater` constant-equation type at all, only the `k · V_dryclay` tie
(`bndwat_soft_rows`, `multimin2.rs:924`–`:968`). The obligation therefore has no surface to attach
to today. **Trigger: if IP's E33/E34 constant-equation bound-water types are ever implemented for
parity, T-9's assertion must ship with them.**

### 7.4 Refusals

#### Vendor behaviours SandiBumi will not reproduce

**REF-1 — IP's mineral-deletion non-negativity heuristic.** `SB-MIN-001` MUST NOT achieve
non-negativity by deleting a component and re-solving a reduced model. Reason: the mechanism changes
the *dimension of the system* at the crossing, so the answer is path-dependent and volumes are
discontinuous in depth over a shaly interval where a marginal mineral flickers in and out. Elan and
Geolog both use genuine constrained optimisation with published engines; IP's is an uncited
heuristic. The consequence — SandiBumi will not reproduce IP volume-for-volume even with identical
endpoints — is disclosed to the user by `SB-MIN-002`, not hidden.

**REF-2 — IP's soft unity followed by post-hoc renormalisation.** `SB-MIN-003` [P0] makes unity a
hard equality. Reason: IP's own manual concedes *"the unity equations will not necessarily force the
results to absolutely 1.0"* and then rescales every volume in step 4. A soft unity lets the solver
buy misfit reduction with mass; the rescale then propagates the purchase into every downstream PHIE,
Sw and pay summary.

**REF-3 — Clock-seeded pseudo-randomness.** `SB-MIN-037` MUST NOT seed from wall-clock time. Reason:
IP *"uses a random number generator, seeded through the CPU clock time"* with no seed field
documented anywhere, so its uncertainty bands cannot be regenerated. Its substitute — rank
iterations, pick a percentile, reload that iteration's saved parameters — is a **replay** mechanism,
not reproducibility. A client deliverable whose uncertainty band cannot be reproduced two years
later is not defensible, and fixing it costs one integer.

**REF-4 — Presenting two different equations under one name.** `SB-MIN-030` forbids labelling IP's
compact E63/E64 and Elan's Worthington Type-2 Eq 78 both as "Simandoux" without stating which form is
running. Reason: Elan's carries `V_silt` as a first-class term in two places and a full citation
chain; IP's carries neither. Same name, different physics.

**REF-5 — Deriving shipped default uncertainties from MIN/MAX at runtime.** This is a **deliberate
departure from the dossier's own adoption spec** (§4 item 3: *"store MIN/MAX, derive the default"*).
`SB-MIN-019` requires MIN, MAX and the printed default be stored as three independent fields and the
**printed value win**. Reason: §2.4's corrected count shows Elan's own table deviates from Elan's own
1.5 %-of-range rule on **six** rows (DT, CUDC, CXDC, TPL, VELC, and Table 30's `SDPT`) by 4–7 %, with
implied ranges tidier than the tabulated MIN/MAX. A product that derives silently disagrees with the
vendor on six rows. The rule is kept — as a *labelled* derivation for tools no vendor tabulates
(`SB-MIN-020`) — but never as the source of a value the vendor printed.

**REF-6 — Shipping either parse of IP's malformed D-07 clay-bound-water `F` relation.** No §4
requirement offers that route. Reason: the *form* is ambiguous, not the constant, so "no default —
user must set" does not help; and `DISCREPANCIES.md` directs it be resolved only against ACQ-2.

**REF-7 — Adopting `ip_ingest/E_threeway_endpoint_compare.json`'s IP CEC column.** Reason: it was
computed with IP's **wet**-clay density where the file's own note specifies the dry grain density —
verified to 4 dp on four clays (0.26784 / 0.09159 / 0.15042 / 1.37753 against the file's shipped
0.2678 / 0.0916 / 0.1504 / 1.3775). Those four values are systematically biased. **This chapter cites
no value from that column**; where an IP-derived CEC appears in the reasoning it is the corrected
table in dossier §2.8.

#### What this chapter declined to transcribe — CONTRACT §2.1 and §2.2

**REF-8 — The 27-row × 14-tool-key `LIB` matrix is not enumerated in §5.** Rows appear only where a
§4 requirement turns on the specific value. Reason: enumerating the full matrix would copy a merged
vendor library into a second location — the exposure `SB-CORE-005` exists to reduce, not to
duplicate. It is carried as an *asset*: 27 rows, 14 columns, `VENDOR-DERIVED`, merged from two vendor
installs in a third vendor's dropdown order per `IP_PROVENANCE.md` §2.2. `SB-MIN-009` is the
requirement that re-sources it row by row.

**REF-9 — No `.neu` neutron look-up table data is transcribed anywhere in this chapter.**
CONTRACT §2.1 names `.neu` chart tables explicitly. The tables are cited by *format and convention*
only — columns, salinity breakpoints, porosity rows — and the two known non-monotonicities (the
`-.1960` outlier at φ = .20 Dolomite/50 kppm, and the milder φ = .25 sand/100 kppm case) are cited as
**QC facts, not as data**. `SB-MIN-026` requires the table be a named, recorded model input; the
table format, its load-time integrity checks and the "do not silently repair" rule belong to `ENV`,
which is where §1 declares the seam.

**REF-10 — Elan Tables 29/30 are not carried into §5 as an 18-row default set.** §5 carries the
two-vendor-corroborated *generating rule* and **one** row for the per-tool defaults, marked
`ABSENT — ships with no default`. Reason: the set is the deliverable of `SB-MIN-020`, and copying a
vendor default table into the chapter would create a second location with no build gate over it.
**Disclosed consequence:** §5 therefore satisfies CONTRACT §2's "every parameter any requirement
refers to appears here exactly once" for the per-tool σ values with a single ABSENT row rather than
fourteen valued rows. That is a scoping decision, stated rather than drifted into.

**REF-11 — Tier C: IP's Wyllie ↔ Hunt-Raymer `Cp` bridge is named and not implemented.** It is a
vendor-fitted four-term regression (dossier G-6). CONTRACT §2.2: named, never built, never
approximated, never reverse-engineered — **and its coefficient set is deliberately not repeated in
this chapter**, in any section. The capability-level description is all that is permitted and all
that is given. SandiBumi's answer is a design-around by construction rather than by approximation:
it uses the full non-linear sonic form, which is what IP's own non-linear optimiser does, so the fit
is never needed. **No other Tier-C register item is touched by this domain** — Experienced Eye /
EEFS, Domain Transfer Analysis, Omovie Sonic Saturation (US 12,242,011 B2), entropy image
speed-correction, shipped neural-network weight DLLs, Textural Facies `Freq_Tiles` encoding and
frequency-domain dispersion fits do not appear in the mineral-solver dossier and are not described,
proposed or renamed here.

**REF-12 — Client identifiers stay in the research file.** CONTRACT §2.3, and dossier MAJ-9 reached
the same ruling independently. The project-kb precedent's whole-rock CEC range (3.11–7.99 meq/100 g)
and the project it belongs to are **not** carried into any requirement, parameter row, test name,
fixture or shipped warning string. `SB-MIN-011`'s window `[0.01, 2.0] meq/g` and its 0.05 meq/g warn
threshold are derived from the **shipped library floor alone** — Kaolinite 0.10 meq/g, the lowest
clay-mineral CEC in any of the three libraries — and `SB-MIN-011` states in its own text that neither
the message nor the fixture may name a client project or carry a client core-analysis range. Carrying
the precedent into the product is explicitly Jauhar's call and no agent's.

#### The Matthews & Kelly exception was not used, and is not a precedent

**REF-13 — Explicit non-reliance, stated because this chapter reasoned near it.** §5 transcribes a
small number of individual vendor values under `NON-ADOPTABLE — cited for verification`: three
competing Techlog clay CEC values, the four per-clay `Rsh` values, `XWater`, and — inside
`SB-MIN-T44` only — Techlog's three clay density triples. Each is a **single value cited so a stated
requirement can be checked**, none is the content of a lookup table, and **none is shipped**.

**This chapter did not reason from CONTRACT §2.1's Matthews & Kelly exception to justify any of
them.** That exception is scoped to the Matthews & Kelly rows in `Fract_Grad_Coeff.par` — retained
because that file is plain text, self-documenting and user-extensible by its own header, its rows
digitise a published 1967 paper, and a High-rated quantification is uncheckable without them — and it
was ruled on by Jauhar directly on 2026-08-07. **It is not a precedent and this chapter treats it as
none.** Where a second case looked possible — carrying Elan Tables 29/30 wholesale (REF-10), or
Techlog's `Rhobdcl` column into §5 (ESC-6) — the chapter **stopped and escalated rather than
deciding**, which is what CONTRACT §2.1 requires of a chapter that believes it has a second case.

---

## 8. Traceability — dossier disposition

**Source dossier:** `docs/research_2026-08/cross_tool/mineral-solver.md`, 2,205 lines, written
2026-08-06 (second, verification session).

**This is the completeness gate.** Every numbered finding, inventory entry, comparison block, ledger
row, adoption-spec line, canonical form, parameter row, rule, dossier test, delta, escalation, source
register entry and critique finding in that file has a row below. The subsections follow the
dossier's own structure, so a reader can reconcile it section by section.

**Disposition vocabulary.** `ADOPTED` — it became a requirement, a parameter row or a test.
`DEFERRED` — recognised and not carried now; **every DEFERRED row states its trigger**.
`REJECTED` — not carried, **with the evidence for rejection**. `EVIDENCE-ONLY` — it informed the
chapter's reasoning or bounded a claim, but no requirement flows from it. `ESCALATED` — routed to §7
because no agent may decide it.

**Counting convention, stated up front so the arithmetic at §8.17 can be checked.** One dossier item
= one row, with two declared exceptions. **§8.9** (the §5.2 parameter tables) carries **86 dossier
parameter rows in 47 table rows**: blocks whose disposition is uniform are grouped, and each grouped
row states how many dossier rows it covers. **§8.8** carries **11 dossier canonical forms in 13
rows**, because F-3 and F-4 each contain a lettered sub-item the dossier itself flags as a
correction. Every other subsection is 1:1. A lettered sub-item carrying its own evidence is its own
row (`Δ-4a`, `Δ-4b`, `Δ-4c`; `D-07(a)`, `D-07(b)`). §8.16 enumerates the **surplus** — chapter
requirements with no dossier antecedent — separately, so they are never mistaken for dossier
coverage.

### 8.1 Dossier header — the verification pass (7 rows)

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Re-verified: IP objective E1, non-negativity heuristic, Shell 0.018/0.019, D-09/D-10/D-12/C-5.x — all confirmed; the ingest's own "published Shell formula uses 0.019" note added | ADOPTED | F-1, F-8; `SB-MIN-001`, `SB-MIN-022`; ESC-4 records the note as a third-party reading, not a citation |
| Re-verified: Geolog `α` cutoff, `w = 0.75m+0.25n`, box bounds 0.5, `CONDNUM`, RF04 6.9 uncertainties — confirmed verbatim | ADOPTED | §5 rows `α` cutoff / `w` / `bounds.fluid_max` / `condnum_warn` / `condnum_fail`; `SB-MIN-015`, `SB-MIN-021` |
| Re-verified: Geolog `k_clay` density unit — **CORRECTED, not confirmed** (`g/cc scaled`, not kg/m³; an earlier verification row was wrong) | ADOPTED | F-5's `96`-form constant and its `362.4` denominator; `SB-MIN-006`, `SB-MIN-T05`. The self-correction is why the chapter treats verification rows as claims, not warrants |
| Re-verified: IP `MINDEF.PAR` clay + matrix rows, `MINEQDEF.PAR` confidences; §2.8 CEC back-solve reproduces to 4 dp | ADOPTED | §5 CEC block; the same 4-dp reproduction is what convicts the `E_threeway` column in REF-7 |
| Re-verified: Techlog `QM_MineralTable.xml` — every clay, silt, shale, fluid row; three unused columns added (`Silt`, `Rsh`, Chlorite/Shale GR & Σ) | ADOPTED | `SB-MIN-030` (silt first-class), `SB-MIN-031` (`Rsh`), F-16; §5 `NON-ADOPTABLE` rows |
| Re-verified: Elan prose quotes — all verbatim; the "divide the multiplier by four" quote **re-attributed** to `-default-uncertainties.html` with a dropped second sentence restored | ADOPTED | F-7, `SB-MIN-018`, `SB-MIN-T18` — the second sentence is the operative one (the multiplier is applied to the *uncertainty*, not the weight) |
| Re-verified: Elan equation rasters 10/11/11-1a/11-1b/12, 17, 63/64, 78, 79/80, 85-1a/1b, Table 29 re-converted and re-read **independently of the first run**; Table 29 reproduces row-for-row | ADOPTED | F-3 (the `LargestWeight` cancellation), F-6, `SB-MIN-013`, `SB-MIN-024`, `SB-MIN-046`; the independent second read is the reason §2's Elan claims are treated as T2-equivalent rather than T3 |

### 8.2 §0 — evidence tiers and coverage honesty (5 rows)

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Tier scheme T1 / T2 / T2-equivalent / T3 / T4 / Precedent | ADOPTED | The chapter tags every vendor claim in §2 and every §5 `Source` cell with its tier, per CONTRACT §1.2 |
| Escalation performed 1 — 19 Techlog `petrophysics-elan*.html` pages read directly | EVIDENCE-ONLY | Why Elan claims in §2 are T2-equivalent; underwrites F-3, F-6, F-7, F-17 |
| Escalation performed 2 — 14 equation rasters converted and vision-read; **nine equations recovered no prior ingest carries** (Eq 10/11/11-1a/11-1b/12, 16/17, 34/36/37, 60–66, 85-1a/1b, Tables 28/29/30) | ADOPTED | F-6 (`SB-MIN-024`), F-3 (`SB-MIN-013`), F-4 (`SB-MIN-003`, `SB-MIN-042`, `SB-MIN-043`); Eq 85-1a/1b is the source of the summation-as-inequality-pair reading behind `SB-MIN-003` |
| Escalation performed 3 — `multimin2.rs` read to anchor §5 on what ships | ADOPTED | The whole of §3; every `file.rs:line` citation in this chapter descends from it |
| Coverage honesty — Geolog is T3-only, Multimin is a compiled module, **a blank Geolog cell is absence of ingest not absence of capability** | ADOPTED | Carried verbatim in spirit into ESC-2 and OPEN-1; the chapter never infers a Geolog capability gap from a blank cell |

### 8.3 §1 — method inventory (33 rows)

**§1.1 IP Mineral Solver — 8 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Model equation types — 21, exhaustive | EVIDENCE-ONLY | Bounds the tool roster the chapter reasons about; individual members land at `SB-MIN-012` (U), `SB-MIN-025` (Invasion Factor), `SB-MIN-026` (neutron), `SB-MIN-021` (Cond./Res.), `SB-MIN-003` (Unity), `SB-MIN-043` (PhiLimit) |
| Output-only equation types — 8 + 5 new in IP 2025 (`SonicMatrix`, `Output Dry Wt%`, three HC-corrected) | DEFERRED | **Trigger:** output-curve parity work. SandiBumi computes `GrainDensity`, `Qv` and `PhiTClay` natively (`dry_clay_calc`, `bound_water_multiplier`) but exposes no "output equation type" concept |
| Mineral types — 8, exhaustive (`Water Sxo`, `Bound Water`, `Hyd. Sxo`, `Matrix`, `Wet Clay`, `Dry Clay`, `Water Sw`, `Hyd. Sw`) | ADOPTED | `SB-MIN-005` — the fluid ceiling must follow the component **kind**, matching `classify` (`multimin2.rs:801`–`:867`); `SB-MIN-T04` |
| Sw equations — 11, identical in both editions | DEFERRED | **Trigger:** the `SAT` chapter, per the §1 seam. The count correction itself is ledger item CT-2 (§8.7) |
| IP's three Tier-B Sw citations (Woodhouse 1976, Aguilera 1990, Poupon-Loy-Tixier 1954) + the Vcl-vs-Vshale position | EVIDENCE-ONLY | ACQ-7; **and it narrows F-17's claim** — IP ships no provenance for *Simandoux specifically*, not "no provenance at all" |
| Solvers — SVD then DNOPT in series, best-of-two always selected | EVIDENCE-ONLY | `SB-MIN-001`/`SB-MIN-002` rationale: IP's own architecture is a real optimiser, which is why the deletion heuristic (F-1) is the anomaly rather than the design |
| Calibration — MLR of endpoints against core/XRD, no constant term, one blank curve permitted | DEFERRED | **Trigger:** an endpoint-calibration feature. No §4 requirement; when it ships, the no-constant-term convention and the remainder rule must be stated, not inherited silently |
| Multi-model combination — 20 (2018) → 50 (2025) models, 5 mixings, `Mdl Merge Dist` box-filter then renormalise | DEFERRED | **Trigger:** multi-model support. SandiBumi solves one model per run. The 20→50 contradiction is ledger item CT-1 (§8.7) |

**§1.2 Techlog Quanti.Elan — 9 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Solve-process structure — over-determined weighted least squares over formation-component volumes | ADOPTED | `SB-MIN-001`; §3 confirms SandiMin is already this shape (`solve_bounded_lsq`, `multimin2.rs:1841`–`:2006`) |
| Response-equation families — 10 documented (general linear, GR linear/MASSIC, SP→`QVSP_N`, sonic slowness/velocity, non-linear neutron, 6 conductivity, spectroscopy, CRIM, Constant Tools) | EVIDENCE-ONLY | Bounds the roster; `Constant Tools` specifically underwrites `SB-MIN-017`'s distinction between an inactive row and a zero-weight row |
| Sw / conductivity models — 6, verbatim from Table 21 | DEFERRED | **Trigger:** `SAT`. `SB-MIN-021` carries the one part that belongs here — the root exponent applied to the conductivity row |
| "Only one conductivity equation per zone per model" | EVIDENCE-ONLY | ESC-7 — it is the vendor rule that makes a per-clay `Rsh` unreachable in Elan, and therefore the reason `SB-MIN-031` is a capability question at all |
| Sw is never solved directly — Elan solves fluid volumes, Sw is a post-process Function | ADOPTED | `SB-MIN-036` (nomenclature must say what was solved and what was derived); matches SandiMin's own architecture |
| Constraint classes — 3 (internal immutable, predefined inequality opt-in, user-defined linear), and constraints are **absolute limits** contrasted with weighted Constant Tools | ADOPTED | `SB-MIN-033`, `SB-MIN-034`, `SB-MIN-035`, `SB-MIN-043`; the absolute-vs-weighted contrast is exactly the defect `SB-MIN-034` repairs in the WBM re-solve |
| Predefined inequality constraints — 7, exhaustive | ADOPTED | `SB-MIN-042` (the OBM pair), `SB-MIN-043` (Maximum Porosity, Irreducible Water), `SB-MIN-034` (Conductivity Constraint for WBM); Sonic Clay Volume → OPEN-4 / ACQ-6 territory |
| QC outputs — `SDR` (Eq 80) plus one reconstructed log per bound equation | ADOPTED | `SB-MIN-013` pins `RECON` to Eq 80; `SB-MIN-T13` |
| Balanced uncertainties computable **before** volumes are solved ("uncertainties do not include the volume of the mineral") | ADOPTED | `SB-MIN-039`, `SB-MIN-T20` — the quoted clause is the proof that the pre-solve computation is well-defined |

**§1.3 Geolog Multimin — 9 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Unknown vector — minerals once, **two parallel fluid sets** X (9 species) and U (8, no OBM filtrate) | ADOPTED | `SB-MIN-034`'s water-species sums; `SB-MIN-036`'s X/U nomenclature discipline |
| Capacity — max 30 volumes, max 50 equations, `nvol ≤ nequations + ntool_constraints` | EVIDENCE-ONLY | The **inequality** is the same DOF accounting `SB-MIN-016` [P0] enforces (`dof`, `multimin2.rs:1204`). **The two capacity numbers are deliberately not §5 rows** — they are a vendor implementation limit, not a petrophysical parameter, and SandiBumi imposes no such cap |
| Per-tool response methods (neutron LINEAR MATRIX default / per-vendor fits; sonic Wyllie/RHG; GR linear/MASSIC; CT & CXO share one method; excavation default ON) | DEFERRED | **Trigger:** per-tool method parity. `SB-MIN-026` carries the neutron half's requirement (named recorded input); the excavation default-ON is why `SB-MIN-026`'s "recorded" clause matters — a default-on term that no one recorded is the failure mode |
| Volume-constraint row types — 4 (`==`, `Tool`, `>=`, `<=`), and **only `Tool` rows add a degree of freedom** | ADOPTED | `SB-MIN-016` [P0] (DOF accounting), `SB-MIN-035` (Tool rows hard + residual reported), `SB-MIN-T16`, `SB-MIN-T35` |
| Program (automatic) constraints — 7 (UNITY, POROSITY, IRR WATER, X/U BNDWAT, WATER MUD, OIL MUD + OIL MUD GAS) | ADOPTED | `SB-MIN-003`, `SB-MIN-034`, `SB-MIN-042`, `SB-MIN-043`; §3 maps each to its shipped row or records its absence |
| Volume bounds — hard box, always honoured exactly; defaults all 0..1, **every fluid upper bound 0.5** | ADOPTED | §5 `bounds.solid_max` / `bounds.fluid_max`; `SB-MIN-005`, `SB-MIN-T04` — SandiMin ships the same 0.5 at `multimin2.rs:1189` |
| Solver — convex QP (Goldfarb & Idnani 1983) then SQP (Powell VF02 + watchdog) | EVIDENCE-ONLY | `SB-MIN-001` rationale; ACQ-8 records that the two papers are cited second-hand through a T3 helpset |
| QC outputs — `CONDNUM` = log₁₀ SVD norm ratio of `PᵀUP` (>8 suspect, >10 unstable); `QUALITY = sqrt(Δ²/χ²₉₅(ntool−3))`; `NFUN`; per-volume `sqrt(diag(A⁻¹))·QUALITY` | ADOPTED | `SB-MIN-015` (`CONDNUM` + thresholds), `SB-MIN-014` (comparable statistics), `SB-MIN-016` (the `ntool−3` DOF), `SB-MIN-038` (per-volume uncertainty) |
| Model switching — primary + up to 10 secondary (expression, model) pairs; `SKIP`/`NONE`/`IGNORE`; probabilistic `Wᵢ = Pᵢ·∏(1−W_j)` | DEFERRED | **Trigger:** facies/multi-model switching. Note for that work: `IGNORE` is a bad-hole *abandon*, semantically distinct from `SKIP`, and conflating them would silently change which intervals get an answer |

**§1.4 Explicit "no evidence held" — 7 rows.** Recorded because a stated absence is evidence.

| Dossier item | Disposition | Where it landed |
|---|---|---|
| Geolog Multimin QP/SQP source code — compiled module, no install tree | EVIDENCE-ONLY | Fixes the Geolog leg's ceiling at T3 permanently; ACQ-8, OPEN-1 |
| Geolog neutron per-vendor fit coefficients `a,b,c,d` — named, **values not transcribed** | DEFERRED | **Trigger:** `ENV`'s neutron work. The dossier's own decision not to transcribe them is the same discipline as REF-9 here |
| Elan neutron matrix/fluid coefficient tables (Table 27 region) — raster not read | DEFERRED | OPEN-2; `SB-MIN-026` ships without needing them |
| IP Sigma / cased-hole — **not documented by the vendor**; the "routed to slice H" claim was itself removed as unsourced | DEFERRED | **Trigger:** cased-hole support. Also the reason §4 of the dossier carries a second row numbered 20 (§8.6) |
| IP's full mineral drop-down roster — not printed, lives in `MINDEF.PAR` on disk | DEFERRED | OPEN-7(c); bears on `SB-MIN-028`'s library roster |
| Elan geochemical uncertainty Table 31 — raster exists, not read | DEFERRED | ACQ-9. **Trigger:** ECS/spectroscopy rows shipping; none do |
| `Multimin_Knowledge_Transfer.pdf` — held on this machine, **not ingested** | ESCALATED | OPEN-1, and it is half of ESC-2's blocker (the missing Chlorite/Glauconite Geolog WCLP pair) |

### 8.4 §2 — definitions, equations and assumptions compared (33 rows)

Nineteen comparison blocks plus fourteen lettered sub-items that carry their own evidence.

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **§2.1** The forward model — one universal form across three vendors; the invaded/un-invaded split is the first structural divergence (IP per-equation continuous, Geolog categorical CT=U, Elan between) | ADOPTED | `SB-MIN-025` [P2] — per-equation invasion factor, 0.0–1.0, curve-capable; §3 records SandiMin's single global `fl` (`multimin2.rs:2073`) as the gap |
| **§2.2** The objective function — three genuinely different statistics (IP `Total_err` un-normalised, Elan `SDR`/Eq 79–80, Geolog `QUALITY` χ²-normalised) | ADOPTED | `SB-MIN-013`, `SB-MIN-014` [P2], `SB-MIN-016` [P0]; F-3 |
| **§2.2a** The `LargestWeight` **cancels exactly** between Eq 79 and Eq 80 | ADOPTED | F-3 — this is what makes SandiMin's `RECON` *provably* Elan-equivalent rather than Elan-shaped; `SB-MIN-013`, `SB-MIN-T13`. A rare case where an arithmetic proof upgraded a claim instead of demoting it |
| **§2.2b** The conductivity pre-solve root: IP's manual says "square root" and "1/m th root" of the same object, in both editions | ESCALATED | ESC-1; proposed ledger item CT-3; `SB-MIN-021`, `SB-MIN-T21` |
| **§2.3** Weighting / uncertainty semantics — Elan's 4-step algorithm; "divide the multiplier by four" applies to the **uncertainty**, and the dropped second sentence restored | ADOPTED | `SB-MIN-017` [P3] (an inactive row is not a zero-weight row), `SB-MIN-018` [P3] (multiplier semantics stated at the point of entry) |
| **§2.4** Default uncertainties — the three vendor tables side by side | ADOPTED | `SB-MIN-019` [P2], `SB-MIN-020` [P2]; §5 ships one `ABSENT` row for the per-tool set (REF-10) |
| **§2.4a** The "1.5 % of range" generating rule | ADOPTED | `SB-MIN-020` — permitted **only** as a labelled derivation for tools no vendor tabulates, never as the source of a value a vendor printed |
| **§2.4b** The rule has **six** exceptions, not two (corrected twice on revision): DT, CUDC, CXDC, TPL, VELC, and Table 30's `SDPT`, deviating 4–7 % | ADOPTED | `SB-MIN-019`'s "printed value wins" and REF-5 — the six exceptions are the whole evidence for departing from the dossier's own §4 item 3 |
| **§2.5** Constraint handling — the sharpest three-way split (IP soft + iterative one-at-a-time; Elan absolute; Geolog hard in the QP) | ADOPTED | `SB-MIN-033`, `SB-MIN-034` [P1], `SB-MIN-035`, `SB-MIN-043`; ESC-3 owns the iteration policy |
| **§2.6** Wet ↔ dry clay — three formulations, one physics, **three different unit conventions** | ADOPTED | `SB-MIN-010` [P0] (declare the convention), `SB-MIN-024` (explicit `ρ_bw` in any conversion) |
| **§2.6a** Elan Eqs 10 / 11 / 11-1a / 11-1b / 12 recovered from rasters — no prior ingest carried them | ADOPTED | Eq 11 **is** the gate in `SB-MIN-046`; Eq 12 underwrites `SB-MIN-024`; `SB-MIN-T44` |
| **§2.6b** `ρ_bw` is never stated by any of the three vendors in the reformulation, though all three require it | ADOPTED | §5 `ρ_bw` = `ABSENT — ships with no default`; `SB-MIN-024`, and `SB-MIN-T24` **cannot pass today** because `dry_clay_calc` hard-codes `RHO_W = 1.0`, `DT_W = 189.0` (`multimin2.rs:676`–`:699`) |
| **§2.7** Clay-porosity and CEC unit conventions | ADOPTED | `SB-MIN-011` [P0] (CEC unit + physical window), `SB-MIN-027` [P0] (WCLP in v/v, refuse p.u.) |
| **§2.7a** IP's malformed D-07 relation `F = 1 − [0.6425 · (Salinity^(−0.5) + 0.22] · Qv]`, byte-identical in both editions, two reachable parses | REJECTED | REF-6 — SandiBumi ships **neither** parse; no §4 requirement offers the route. Resolution is ACQ-2 only |
| **§2.7b** The p.u.-vs-v/v leaning across Elan's pages | ESCALATED / part-closed | The **WCLP half is closed** by this chapter on T1 (`WCLP_*_unit = u"m3/m3"`) → `SB-MIN-027`; the **global `φₑ` half** stays open → OPEN-3, and it gates ESC-4 |
| **§2.8** CEC units — the meq/mL trap, narrowed by triangulation (C-OPEN-2 / C-OPEN-3) | ADOPTED | `SB-MIN-011`; §5 CEC block carries meq/g explicitly on every row |
| **§2.8a** The meq/mL ↔ meq/g trap itself — a factor-of-ρ error that computes and plots | ADOPTED | `SB-MIN-011`'s declared unit and `[0.01, 2.0] meq/g` window; `SB-MIN-T18` |
| **§2.8b** `ip_ingest/E_threeway_endpoint_compare.json`'s IP CEC column computed with **wet**-clay density where its own note specifies dry grain density — verified to 4 dp on four clays | REJECTED | REF-7 — no value from that column appears anywhere in this chapter; §2's reasoning uses the corrected table |
| **§2.9** Variable cementation exponent — a **bit-identical** equation across two vendors | ADOPTED | `SB-MIN-023` [P2]; §5 carries the `m*` forms with the base exponent `ABSENT` (no vendor states it as a default) |
| **§2.10** The Shell / porosity-dependent-`m` constant — a **three**-way conflict (0.018 / 0.019 / 0.19) | ESCALATED | ESC-4, ACQ-4, ledger D-10; `SB-MIN-022` [P2] ships it `ABSENT` and requires all three be displayed with sources; `SB-MIN-T22` |
| **§2.11** The Simandoux family — IP's compact algebra vs Elan's derived Eq 78 with `V_silt` and a full citation chain | ADOPTED | `SB-MIN-030` [P3] (silt first-class; do not merge the two under one label), F-17, REF-4, ACQ-6 |
| **§2.12** Dual Water and Linear conductivity | DEFERRED / part-ADOPTED | **Trigger:** the `SAT` chapter owns the Sw model set. The part that belongs here — the `Cbw` temperature form — is `SB-MIN-006` [P1], and its primary source is ACQ-3 |
| **§2.13** Neutron excavation — three formulations (IP squares `(ρma/2.65)`, IP's own SSC square-roots it, Elan has no matrix term) | DEFERRED | **Trigger:** an excavation term shipping; none does. Ledger D-09; ACQ-5. `SB-MIN-026` requires the neutron response set be a named recorded input, which is where a default-ON excavation would have to declare itself |
| **§2.14** Hydrocarbon response conversion | DEFERRED | **Trigger:** HC-corrected tool rows (IP 2025's three new output types). Informs `SB-MIN-025`'s invaded/un-invaded bookkeeping; no requirement today |
| **§2.15** U (volumetric photoelectric cross-section) — **exact three-way agreement**, the only one in the dossier | ADOPTED | `SB-MIN-012` [P0] — mix U, never Pe, with `ρe = (ρb + 0.1883)/1.0704`; §5's *Electron-density offset* `0.1883` and *Electron-density divisor* `1.0704` rows; `SB-MIN-T12`. Three-way agreement is why this is P0 rather than a preference |
| **§2.16** Endpoint libraries — the three-way reconciliation | ADOPTED | `SB-MIN-028` [P3] (named selectable libraries, disagreements surfaced), `SB-MIN-009` [P0] (per-value provenance); §5's clay block |
| **§2.16a** Techlog's own `(Rhobwcl, Phicl, Rhobdcl)` triples fail Elan Eq 11 by +4.9 % / −5.1 % / −0.9 % | ESCALATED | ESC-6, dossier G-5; `SB-MIN-046` [P2] gates at 1 % relative; `SB-MIN-T44` uses the vendor's own inconsistency as the fixture |
| **§2.16b** Three `QM_MineralTable.xml` columns the draft had not used: `Silt`, `Rsh`, Chlorite/Shale `GR` & `Σ` | ADOPTED | `SB-MIN-030` (silt as a first-class component), `SB-MIN-031` [P3] (per-clay `Rsh`), F-16 |
| **§2.16c** The smectite/placeholder rows and the claim the dossier **withdrew** on revision | EVIDENCE-ONLY | OPEN-7(c) — IP's smectite endpoint set is unread; §5 ships the generic `Clay` CEC `ABSENT` rather than inferring one. A withdrawn claim is recorded here precisely so it is not silently reinstated |
| **§2.17** Convergence and iteration control | ADOPTED (in part) / DEFERRED | §5 carries SandiBumi's own *Active-set outer iteration cap* `8n + 12` (`multimin2.rs:1893`) and the *Unity closure tolerance* `1e-9`; `SB-MIN-016` carries the DOF half. **IP's PHIFLAG tolerances and iteration caps (0.001 / 0.002 / 20 / 30 / 10) are not §5 rows — trigger:** an IP-parity convergence mode, where they must ship as a named library, not as silent defaults |
| **§2.18** Assumptions stated by each vendor | EVIDENCE-ONLY | Underwrites `SB-MIN-002`'s disclosure duty — SandiBumi states its own divergences because the vendors state theirs |
| **§2.19** Uncertainty propagation *through* the solver — three different answers (IP Monte Carlo, Elan balanced pre-solve, Geolog per-volume posterior) | ADOPTED | `SB-MIN-037`, `SB-MIN-038`, `SB-MIN-039`; `SB-MIN-T38`, `SB-MIN-T39`, `SB-MIN-T20` |
| **§2.19a** IP's Monte Carlo contract — ±10 % of endpoint, 2000 iterations, **clock-seeded RNG**, replay-by-saved-parameters | ADOPTED / REJECTED (split) | The ±10 % default and the iteration count land in §5's MC block; the **clock seed is REJECTED** at REF-3 and `SB-MIN-037` requires an explicit recorded seed with `ABSENT` as the shipped default |

### 8.5 §3 — differences that matter (8 rows)

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **§3.1** Non-negativity: mineral deletion vs bounded optimisation — *the* portability breaker | ADOPTED | `SB-MIN-001` [P1] + `SB-MIN-002` [P1]; REF-1; `SB-MIN-T01`, `SB-MIN-T14`. The chapter refuses the heuristic **and** discloses the consequence, rather than quietly diverging |
| **§3.2** Bound water: fixed `φTclay` (IP) vs `CEC·ρ·T·salinity` (Elan, Geolog) — **the fresh-water item** | ADOPTED | `SB-MIN-006` [P1], `SB-MIN-040` [P2] (the two routes are mutually exclusive and which ran is recorded); `SB-MIN-T05`, `SB-MIN-T27` |
| **§3.3** The misfit statistic is not portable | ADOPTED | `SB-MIN-014` [P2] — SandiBumi emits both an IP-comparable and a Geolog-comparable statistic rather than one number that means neither; `SB-MIN-T14` |
| **§3.4** Wet/dry clay endpoint convention — **the highest-frequency silent-wrongness site** | ADOPTED | `SB-MIN-010` [P0], `SB-MIN-024`; and it is the site the chapter's own T1 read extended into `SB-MIN-008` (§8.16) |
| **§3.5** Fluid sonic endpoints — a **45 µs/ft spread on gas** across vendors | ADOPTED | `SB-MIN-029` [P3] — the fluid sonic endpoint must carry its source; §5's fluid rows; `SB-MIN-T28`, `SB-MIN-T29` |
| **§3.6** Limit/constraint semantics — soft in IP, hard in the others | ADOPTED | `SB-MIN-033`, `SB-MIN-034`, `SB-MIN-035`, `SB-MIN-042`, `SB-MIN-043`; REF-2 |
| **§3.7** `a` asymmetry in IP's linearised-Archie generalisation | DEFERRED | **Trigger:** `SAT` owns `a`, and no §5 row here carries it. The binding half is recorded: **implement as printed, do not "tidy" it** (ledger C-5.4), and `SB-MIN-009`'s per-value provenance is what keeps a one-sided vendor convention visible instead of averaged away |
| **§3.8** What each tool does that the others do not | EVIDENCE-ONLY | Scopes `SB-MIN-028`'s library roster and the §1 seam declarations; the capability asymmetries are recorded, and **no claim of vendor absence is made in any requirement** (see ESC-7 for why that discipline matters) |

### 8.6 §4 — optimal choice per item (22 rows)

**The dossier's table numbers 1 … 21 but contains 22 rows: "20" is used twice** — once for
*Uncertainty propagation* and again, after 21, for *Sigma / cased hole*. Both carry their own
evidence and both are disposed of here. This is the first of the two count discrepancies §8.17
reconciles.

| # | Dossier adoption choice | Disposition | Where it landed |
|---|---|---|---|
| 1 | Objective function — Elan Eq 79/80 as `RECON`, **plus** an IP-comparable un-normalised `TOTERR_IP` | ADOPTED | `SB-MIN-013` [P1], `SB-MIN-014` [P2]; `SB-MIN-T13`, `SB-MIN-T14` |
| 2 | Weighting — `w = 1/U²` in physical units plus an explicit per-equation weight multiplier | ADOPTED | `SB-MIN-018` [P3]; §5's *Weight multiplier* `xxxx_WM = 1.0` row; Elan's worked rationale is quoted in F-8 |
| 3 | Default uncertainties — Elan's 1.5 %-of-range rule, MIN/MAX stored, **"derive the default"** | **ADOPTED WITH A DEPARTURE** | `SB-MIN-019` [P2] requires the **printed** default win and the derivation be reserved for untabulated tools (`SB-MIN-020`). The departure is REF-5 and its evidence is §2.4b's six deviating rows. **This is the only place the chapter overrules the dossier's own adoption spec, and it is stated rather than drifted into** |
| 4 | Unity — hard equality, not IP's soft-plus-renormalise | ADOPTED | `SB-MIN-003` [P0], `SB-MIN-004`; `SB-MIN-T03`, `SB-MIN-T14`; REF-2 |
| 5 | Non-negativity / bounds — hard box, `0..1` solids, `0..0.5` every fluid | ADOPTED | `SB-MIN-001` [P1], `SB-MIN-005` [P2]; §5's *Fluid component upper bound* `0.500` and *Mineral/clay component upper bound* `1.000` rows; `SB-MIN-T01`, `SB-MIN-T04` |
| 6 | Bound water — CEC route `k = α·96·CEC·ρ_dcl/(T + 298)`, with a WCLP override route | ADOPTED | `SB-MIN-006` [P1], `SB-MIN-040` [P1]; `SB-MIN-T05`, `SB-MIN-T06`, `SB-MIN-T27` |
| 7 | Wet↔dry clay — Elan Eqs 10/11/11-1a/11-1b/12 generalised with an explicit bound-water density | ADOPTED | `SB-MIN-024` [P2], `SB-MIN-010` [P0]; `SB-MIN-T24` (**fails today** — `RHO_W = 1.0` hard-coded) |
| 8 | U vs Pe — mix U volumetrically, `U = Pe × (ρb + 0.1883) × 0.93423`, computed once up-front | ADOPTED | `SB-MIN-012` [P0]; `SB-MIN-T12`; §5's *Electron-density offset* and *Electron-density divisor* rows |
| 9 | Variable `m` (Dual Water) — `m* = m + Cm(0.258Y + 0.2(1 − e^(−16.4Y)))` | ADOPTED | `SB-MIN-023` [P2]; §5's `m*` rows. **The base exponent ships `ABSENT`** — the dossier's "default `m` 2.0, expose Elan's 1.8" is two vendor conventions, not an adjudicated default |
| 10 | Variable `m` (Waxman-Smits) — `m* = m + Cm(1.128Y + 0.22(1 − e^(−17.3Y)))`, IP-only | ADOPTED | `SB-MIN-023`, carrying the IP-only provenance explicitly per `SB-MIN-009` |
| 11 | Shell / porosity-dependent `m` — **ship no hard-coded constant**, `m₀` and `k` user-set, in-app note showing all three vendor values | ADOPTED + ESCALATED | `SB-MIN-022` [P2]; §5 row `ABSENT — ships with no default`; `SB-MIN-T22`; ESC-4, ACQ-4 |
| 12 | Juhász / Waxman-Smits conductivity — the IP **raster** form, with `×Rw` and `m*` on W-S | DEFERRED | **Trigger:** `SAT`, which owns the `B(T,Rw)` factor. The adjudication itself is closed and must travel with it: the ASCII form adds a conductivity to a dimensionless 1 and is **dimensionally wrong**. The `m*` half is already here at `SB-MIN-023` |
| 13 | Simandoux — ship IP E63/E64 and Elan Eq 78 **separately**, never merged | ADOPTED | `SB-MIN-030` [P3]; §5's *Elan-Simandoux silt exponents* `ersh`/`swshe` row; REF-4; ACQ-6 |
| 14 | Indonesian — IP E65 form, `EVCL`/`MVCL` exposed (Indonesia 1.0/0.5, Nigeria 1.4/0.0), Elan's `Vcl → 1` singularity warning carried | DEFERRED | **Trigger:** the `SAT` chapter, per the §1 seam. `EVCL`/`MVCL` are **not** §5 rows here — an Sw-equation parameter with no consumer in this domain would be a parameter row no requirement refers to, which CONTRACT §2 forbids. The singularity warning travels with the equation |
| 15 | Excavation — ship both IP forms plus Elan's matrix-free form, cite Segesman & Liu (1971) | DEFERRED | **Trigger:** an excavation term shipping. Ledger D-09 stays open; ACQ-5. `SB-MIN-026`'s "named, recorded" clause is what a default-ON excavation would have to satisfy |
| 16 | Invasion model — per-equation continuous factor 0..1, defaulting to Geolog's categorical assignment (1.0 X-readers, 0.0 Ct/Rt, 1.0 Cxo) | ADOPTED | `SB-MIN-025` [P3], which carries the categorical defaults in its own text; `SB-MIN-T25`. §3 records the single global `fl` (`multimin2.rs:2073`) as the gap. No §5 row exists yet because the per-equation field does not — it appears when `SB-MIN-025` is built |
| 17 | Endpoint library — three selectable libraries with per-value provenance, plus a project library overriding all three | ADOPTED | `SB-MIN-028` [P2], `SB-MIN-009` [P0]; `SB-MIN-T28`. **Which library supplies the shipped default `(CEC, WCLP)` pair is ESC-2 and was not decided here** |
| 18 | Calibration against core — IP's MLR with no constant term, Fixed/Var endpoints, R² and N reported, one blank column | DEFERRED | **Trigger:** an endpoint-calibration feature. When it ships, the no-intercept constraint must be stated as a petrophysical choice (a zero-volume rock reads zero), not inherited as an implementation detail |
| 19 | Diagnostics — Geolog's `CONDNUM` and DOF/conflict checks on top of Elan-style `SDR` | ADOPTED | `SB-MIN-015` [P1], `SB-MIN-016` [P0], `SB-MIN-033` [P1]; `SB-MIN-T15`, `SB-MIN-T16`, `SB-MIN-T33` |
| 20 | Uncertainty propagation — ship **all three**, plus a mandatory explicit integer RNG seed | ADOPTED | `SB-MIN-037` [P3], `SB-MIN-038` [P3], `SB-MIN-039` [P3]; `SB-MIN-T38`, `SB-MIN-T39`, `SB-MIN-T20`; REF-3 |
| 21 | Neutron look-up table selection — an explicit named model input, never a well-header attribute, always in the run record | ADOPTED | `SB-MIN-026` [P2]; F-11; `SB-MIN-T26`. The `.neu` table *content* is refused separately at REF-9 |
| **20 (second)** | Sigma / cased hole — **do not claim parity**; implement Sigma as an ordinary linear tool and document that IP's cased-hole equation is undocumented | DEFERRED | **Trigger:** cased-hole support. The "do not claim parity" half is binding now and is carried by `SB-MIN-002`'s disclosure duty and by the §1 boundary statement. IP's silence is recorded at §8.3's §1.4 row, not treated as an absence of capability |

### 8.7 §4.1 — ledger disposition, and the three proposed new items (20 rows)

Sixteen ledger rows, of which **D-07 carries two dispositions with separate evidence** and is split
into `D-07(a)` and `D-07(b)`, plus the three items the dossier proposes for
`ip2025_chm_ingest/DISCREPANCIES.md` — proposals only; that file is an ingest report and **was not
modified by the dossier or by this chapter**.

| Ledger item | Disposition | Where it landed |
|---|---|---|
| **D-10** — Shell variable-`m` 0.018 / 0.019, now three-way with Techlog's 0.19 | ESCALATED | ESC-4; `SB-MIN-022` ships `ABSENT`; ACQ-4 names the missing primary source. **Still open, and wider than the ledger records it** |
| **D-12** — Juhász / W-S prose drops `×Rw` (and `m*`) | DEFERRED | **Trigger:** `SAT`. The ledger resolution stands and must travel with the equation — adopt the raster form; the ASCII form is dimensionally wrong. The `m*` half is already adopted here at `SB-MIN-023`, so the `B` factor is the only piece outstanding |
| **C-5.3** — bound-water three-way; the ECS grid's `0.15/−1` fits neither consistent formulation | ADOPTED (principle) / DEFERRED (fixture) | The exclusivity principle is `SB-MIN-040` [P1]. The specific dossier-T-9 assertion has no surface today — **trigger at OPEN-10:** if IP's E33/E34 constant-equation bound-water types are implemented, the mixed-convention model must fail validation |
| **C-OPEN-2** — CEC units never stated (the meq/mL trap) | ADOPTED (narrowed, not closed) | `SB-MIN-011` [P0] declares meq/g on every row and gates the magnitude; `SB-MIN-T18`. Still an **inference**, not a vendor statement — OPEN-7(a) is the round-trip that would close it |
| **C-OPEN-3** — `Qv` output endpoint units unstated | ADOPTED (narrowed) | `SB-MIN-011`'s unit declaration extends to the `Qv` endpoint; the `meq/cm³ wet clay` annotation in `multimin_ip_spec.md` §A is the second source. Kept open on the *endpoint field* |
| **C-5.4** — `/a` on the water summation only in E30 | DEFERRED | **Trigger:** `SAT` owns `a`; the chapter carries no `a` row. Two things travel with it, both binding: **implement as printed, do not tidy it**, and warn when `a ≠ 1` *and* a conductive mineral is present (§3.7). Neither has a home today because no conductive-mineral row ships |
| **C-5.5** — the resistivity-confidence worked example has `+error`/`−error` labels inverted | EVIDENCE-ONLY | **Recorded as a trap, and the trap is the point.** The dossier turns that same example into fixture T-25, and its §5.3 rule 11 mandates worked examples become fixtures — so a fixture built from it would bake the inversion in. This chapter ships **no fixture from that example**: `SB-MIN-T21` uses the `m = 2.5` discriminator instead, which the defective example cannot supply. The C-5.5 warning must travel with the example if `SAT` or a later revision picks it up |
| **C-5.6** — `.neu` non-monotonicity, **two** defects (`-.1960` at φ=.20 Dolomite/50 kppm, and φ=.25 sand/100 kppm) | DEFERRED (to `ENV`) + REFUSED (the data) | The **do-not-silently-repair rule** is binding and is stated in §1's `ENV` seam and F-11; the table *content* is refused at REF-9. **Trigger:** `ENV`'s `.neu` loader, which owns the load-time non-monotonicity warning and must exercise **both** defects |
| **C-5.7** — "Invasion factor" name collision, 0.5 (OBM) vs 2.0 (WBM `Sxo(Sw)`) | DEFERRED | **Trigger:** the OBM `Sxo` ceiling (`SB-MIN-042`) and the WBM route shipping together. The naming split the dossier prescribes (`obm_sxo_max` / `wbm_invasion_factor`) is not yet a §5 row because neither parameter has a shipping consumer — **when either ships it must ship split, per FINDINGS rules 7/8** |
| **C-OPEN-4** — `MINDEF.PAR` `XPL 2.59` / `XAN 2.74` unlabelled | ESCALATED | OPEN-7(b). **Not guessed** — and that restraint is the point: two plausible mineral identities exist for each and either would ship a wrong endpoint silently |
| **C-OPEN-8** — `U_wat = 0.00481·Sal + 0.3883`, `Sal` unit unstated | ADOPTED (as a strong inference) | Recorded in §7 OPEN-7(f) as still open on the vendor's own page. The kppm reading is corroborated twice — physical bounds (0.811 b/cc vs an absurd 422) and D-07(b)'s explicit "Salinity in Kppm" |
| **D-09** — excavation exponent `(ρma/2.65)²` vs `√(ρma/2.65)`, IP-internal | DEFERRED | **Trigger:** an excavation term shipping. Elan **cannot arbitrate** — it has no matrix term at all — so the third vendor does not close it; ACQ-5 |
| **D-07(a)** — the malformed `F` relation itself, two reachable parses | REJECTED | REF-6 — **ship neither parse**. `ABSENT` does not help when the *form* is ambiguous rather than the constant. ACQ-2 is the only admissible resolution |
| **D-07(b)** — the unit statement inside it: `Salinity` in Kppm, `Qv` in meq/ml | ADOPTED | Corroborates `SB-MIN-011`'s meq/mL-vs-meq/g discipline from a second equation, and corroborates C-OPEN-8's kppm reading. **A defective equation still carried a sound unit statement** — which is why the row was split rather than rejected wholesale |
| **C-5.8** — two paragraphs printed twice verbatim in the 2025 source | EVIDENCE-ONLY | Cosmetic, no action. Recorded so a future automated 2018↔2025 diff de-duplicates rather than reporting new content |
| **C-5.9** — "Vhyrocarbon" typo in a limit raster | EVIDENCE-ONLY | Cosmetic — **but it bears on FINDINGS rule 7**: a vendor mnemonic must never be matched by exact string against help text. That is precisely what `SB-MIN-005` [P2] requires, deriving the fluid ceiling from component **kind** rather than a name match |
| **D-15** — `SW`/`SWE` nomenclature conflict, a design mandate | ADOPTED | `SB-MIN-036` [P2] — this domain emits `SWE`, `SWT`, `SXOE`, `SXOT`, `SWT_BND`, `SXOT_BND` and **never a bare `SW`**; `SB-MIN-T37` |
| **CT-1** (proposed) — max user models 20 (slice C) vs 50 (slice O), resolved for slice O | DEFERRED | **Trigger:** multi-model support. The transferable lesson is carried: a hand-maintained "verified identical" table decays exactly like the vendor text it audits |
| **CT-2** (proposed) — Sw-equation count, resolved at 11 (slice C's "12" is a heading miscount) | EVIDENCE-ONLY | Belongs to `SAT`. Recorded because the phantom 12th equation once caused an invented "raster recovery" in an earlier dossier draft — a fabrication produced by a miscount, which is the failure mode this chapter's counting discipline exists to prevent |
| **CT-3** (proposed) — conductivity root exponent, **OPEN, neither reading adopted** | ESCALATED | ESC-1; `SB-MIN-021` [P1] exposes the exponent explicitly and hard-codes nothing behind an IP-parity label; `SB-MIN-T21` |

### 8.8 §5.1 — canonical equation forms (11 dossier forms in 13 rows)

| Dossier form | Disposition | Where it landed |
|---|---|---|
| **F-1** Forward model `t_k = Σ P[k][i]·v_i`, wet-clay internal, Elan Eq 17 dry-clay reformulation | ADOPTED | `SB-MIN-010` [P0], `SB-MIN-024`; §3.2 confirms SandiMin already implements the form |
| **F-2** Zone assignment, per-tool `IF ∈ [0,1]`, defaults 1.0 / **0.0 Ct-Rt** / 1.0 Cxo | ADOPTED | `SB-MIN-025` [P3]; the categorical defaults are carried in the requirement text |
| **F-3** Objective — `RECON = sqrt(Δ²/n_live)` ≡ Elan Eq 80 exactly; `QUALITY`; `TOTERR_IP`; conductivity rows enter with `^(1/w)` and their uncertainties transformed with them | ADOPTED | `SB-MIN-013` [P1], `SB-MIN-014` [P2], `SB-MIN-021`; `SB-MIN-T13`, `SB-MIN-T14`. **"Do not implement `LargestWeight` or the leading ½"** is carried as a positive instruction, not an omission |
| **F-3a** `active` is not `weight = 0` — a zero-weight row still consumes DOF and still enters `PᵀUP`'s conditioning | ADOPTED | `SB-MIN-017` [P1], `SB-MIN-016` [P0], `SB-MIN-015`; `SB-MIN-T17`. (Counted inside F-3, not as a twelfth form) |
| **F-4** Constraints — HARD / TOOL / BOX classes, nine named constraint rows, **the class is part of the specification** | ADOPTED | `SB-MIN-003`, `SB-MIN-034`, `SB-MIN-035` [P1], `SB-MIN-042`, `SB-MIN-043`; §5's *Tool-constraint uncertainty* `0.010` row |
| **F-4a** The UNITY asymmetry — F-4 takes **Elan HARD** on unity while taking **Geolog TOOL** on the other four, changing `n_tool` by one and `QUALITY`'s denominator with it; `unity_mode = tool` recorded as the Geolog-parity switch | ADOPTED | `SB-MIN-003` [P0] + `SB-MIN-004` [P2] — the convention must be reported **alongside any misfit statistic**, which is exactly what stops the asymmetry being discovered instead of declared; `SB-MIN-T03`, `SB-MIN-T14`. (Counted inside F-4) |
| **F-5** Bound-water coefficient — CEC route, WCLP route, `α` with its cutoff and cap, `α_X` from Rmf and `α_U` from Rw salinity | ADOPTED | `SB-MIN-006` [P1], `SB-MIN-040` [P1]; §5's `96` / `298` / `20 455` / `α_max 5.0` rows; `SB-MIN-T05`, `SB-MIN-T06` |
| **F-6** Wet↔dry clay — Elan Eqs 10 / 11 / 11-1a / 11-1b / 12 with `ρ_bw` explicit | ADOPTED | `SB-MIN-024` [P2], `SB-MIN-046` [P2]; §5's `ρ_bw` = `ABSENT`; `SB-MIN-T24` (**fails today**), `SB-MIN-T44` |
| **F-7** U from Pe — `ρ_e`, `U`, `U_wat`, the `U_hyd` gas/oil branches | ADOPTED (in part) | `SB-MIN-012` [P0] and §5's two electron-density rows carry the conversion. **`U_wat` and the `U_hyd` branches are not §5 rows — trigger:** a fluid-U consumer; the `Sal` unit behind `U_wat` is OPEN-7(f) |
| **F-8** Hydrocarbon apparent responses — Conventional and Modified density, neutron HI, all in g/cc, **do not carry both unit forms** | DEFERRED | **Trigger:** HC-corrected tool rows. The "do not carry both" rule is the transferable part and matches `SB-MIN-044`'s canonicalisation discipline |
| **F-9** Variable `m` — `Y = Qv·φT/(1−φT)`, DW and WS coefficient sets | ADOPTED | `SB-MIN-023` [P2]; §5's two `m*` coefficient rows, DW marked two-vendor and WS marked **single-sourced** |
| **F-10** Effective from total saturation — `Swb = 1 − PHIE/PHIT`, `SWE = (SWT − Swb)/(1 − Swb)` | DEFERRED | **Trigger:** `SAT`. `SB-MIN-036` carries the half that belongs here: the curve names and the declared convention |
| **F-11** Derived outputs — the Geolog §A naming contract, **no bare `SW` ever emitted** | ADOPTED | `SB-MIN-036` [P2]; `SB-MIN-T37`; ledger D-15 |

*(F-3a and F-4a are lettered sub-items of F-3 and F-4 and are shown for traceability; the dossier
states eleven canonical forms and this subsection disposes of eleven.)*

### 8.9 §5.2 — parameter table with per-value source strings (86 dossier rows in 47 table rows)

The only grouped subsection. Where a block's disposition is uniform the rows are grouped and the row
states how many dossier rows it covers; every row with a distinct disposition is listed individually.

**Solver control — 14 dossier rows in 9.**

| Dossier parameter | Disposition | Where it landed |
|---|---|---|
| `unity_mode = hard` | ADOPTED | `SB-MIN-003` [P0]; §5's *Unity closure tolerance* `1e-9` is the shipped gate; `SB-MIN-T03` |
| `unity_mode = tool` (the Geolog-parity alternative) | DEFERRED | **Trigger:** a Geolog-parity mode. Its consequence is already stated — it changes `n_tool` by one and `QUALITY` with it (`SB-MIN-004`, `SB-MIN-T14`) |
| `bounds.solid_max` 1.0 | ADOPTED | §5 *Mineral/clay component upper bound* `1.000` |
| `bounds.fluid_max` **0.5** | ADOPTED | §5 *Fluid component upper bound* `0.500`; `SB-MIN-005` [P2]; `SB-MIN-T04` |
| `w_exponent` `0.75m + 0.25n` | ADOPTED | §5 *Conductivity root exponent (default)*; `SB-MIN-021` [P1]; ESC-1 |
| `condnum_warn` 8.0 **+** `condnum_fail` 10.0 (2 rows) | ADOPTED | §5 *Conditioning-number thresholds*; `SB-MIN-015` [P1]; `SB-MIN-T15` |
| IP convergence controls — `outer_loop_phie_tol` 0.001, `outer_loop_sxo_tol` 0.002, `max_linearization_iters` 20, `max_solver_iters` 30, `max_sw_loop_iters` 10 (5 rows) | DEFERRED | **Trigger:** an IP-parity convergence mode, where they ship as a named library rather than silent defaults. §5 carries SandiBumi's own *Active-set outer iteration cap* `8n + 12` instead, labelled engineering |
| `max_volumes` 30 | EVIDENCE-ONLY | A vendor implementation limit, not a petrophysical parameter; SandiBumi imposes none |
| `max_equations` 50 | EVIDENCE-ONLY | Same; the `nvol ≤ neq + ntool` **inequality** is what carried through, into `SB-MIN-016` |

**Default uncertainties — 18 dossier rows in 3.**

| Dossier parameter | Disposition | Where it landed |
|---|---|---|
| Elan Table 29 tools — RHOB, NPHI, DT, GR, U, SIGMA, EPT(TPL), EATT, PHIT, VELC, VOLS, CUDC/CXDC (12 rows, MIN/MAX/default/multiplier each) | DEFERRED | §5 carries **one** row, *Per-tool default uncertainties* = `ABSENT — ships with no default`, and `SB-MIN-019` [P2] / `SB-MIN-020` [P2] own the library. **Trigger:** the uncertainty library shipping. Not transcribed here — REF-10 |
| Elan Table 30 tools — EQHY, ENPA/ENPU, QVSP, BMK, SDPT (5 rows) | DEFERRED | Same row, same trigger. `SDPT`'s *"no program default"* is the case `SB-MIN-019`'s three-field storage exists for |
| CT / CXO auto `0.03·Cfw^(1/w)` / `0.03·Cmf^(1/w)` | ADOPTED + ESCALATED | §5 *Auto conductivity uncertainty fraction* `0.03`; ESC-5 — the vendor rendering is held, the primary source is not, and §5's row omits even the vendor citation |

**IP-parity uncertainty set — 13 dossier rows in 1.**

| Dossier parameter | Disposition | Where it landed |
|---|---|---|
| The full IP `MINEQDEF.PAR` confidence set — Unity 0.01, Density 0.02, Neutron 0.02, Sonic 3.0, Cond 10, Res 100, EPT/U/SIGMA 0.2, GR 5, K 1.0, Th/Ur 0.1, ECS 0.02, Linear 1.0, BoundWater/Constant/PhiLimit 0.01 (13 rows) | DEFERRED | **Trigger:** the selectable IP-parity library named in `SB-MIN-020`. Deliberately **not** transcribed into §5 — same reasoning as REF-10, and OPEN-7(d) would re-verify the column against the live install before it ships |

**Endpoint uncertainty for stochastic propagation — 10 dossier rows in 7.**

| Dossier parameter | Disposition | Where it landed |
|---|---|---|
| `mc.endpoint_shift_default` **±10 % of the endpoint** + `mc.distribution` `Gaussian` (2 rows) | ADOPTED | §5 *Monte Carlo endpoint shift* — and the row records the **inverted unit convention** (a percentage of the endpoint, not an absolute in tool units), which is exactly the kind of silent unit flip `SB-MIN-044` exists to catch |
| `mc.gaussian_width` 4σ, `mc.gaussian_truncation` ±2.5σ, `mc.tornado_span` ±2σ (3 rows) | DEFERRED | **Trigger:** the Monte Carlo implementation (`SB-MIN-037` [P3]) |
| `mc.iterations` **2000** | ADOPTED | §5 *Monte Carlo iterations*; corroborated by the 2018→2025 crosscheck addendum |
| `mc.autostop_burn_in` / `_interval` / `_min_total` 200 / 100 / 300 | DEFERRED | **Trigger:** the auto-stop criterion shipping. Note for that work: an auto-stop that ends a run early makes the iteration count part of the result and therefore part of the run record |
| `mc.correlation` `m`↔`n` — **0.8 (grid) vs 0.5 (prose), both cited, no default shipped** | DEFERRED | **Trigger:** the MC correlation matrix. The both-cited-no-default discipline must travel with it; it is the dossier's own internal discrepancy D-5.8 and is the same shape as ESC-4 |
| `mc.correlation` `ρ_wetclay`↔`φN_wetclay` **−0.8** | DEFERRED | Same trigger. Prose and screenshot agree, so unlike the row above it ships with a single cited value |
| `mc.seed` — **no vendor default; SandiBumi requires an explicit integer** | ADOPTED | §5 *Monte Carlo seed* = `ABSENT — ships with no default`; `SB-MIN-037`; REF-3; `SB-MIN-T38`. **The vendor default is the defect**, which is why the row is ABSENT rather than inherited |

**Saturation / electrical — 22 dossier rows in 18.**

| Dossier parameter | Disposition | Where it landed |
|---|---|---|
| `a` 1.0, `m` 2.0, `n` 2.0 (3 rows) | DEFERRED | **Trigger:** `SAT` owns the Archie exponents. Carrying them here would be a §5 row no §4 requirement refers to |
| `m_dw` — **2.0 (IP) / 1.8 (Elan), two defaults, user picks** | ADOPTED | §5 *Variable `m*` base exponent* = `ABSENT — ships with no default`. **The chapter is stricter than the dossier**: "user picks" becomes "no default ships", because two vendor conventions are not an adjudication |
| `Cm` / `Cdw` 1.0 | DEFERRED | **Trigger:** `SAT`; it is the multiplier on the `m*` correction `SB-MIN-023` implements |
| `ersh` 1.0 **+** `swshe` 0.5 (2 rows) | ADOPTED | §5 *Elan-Simandoux silt exponents*, mapped to Worthington Type-2 `x = 1.0`, `c = 1.5`; `SB-MIN-030` [P3] |
| `mc2` default **0.0** with prose *"usually 0.19"* | ESCALATED | ESC-4 — the 0.19 is the third leg of the Shell three-way conflict, and the 0.0 table default is what makes the prose value reachable only by hand |
| Shell `m₀`, `k` — **NO DEFAULT** | ADOPTED | §5 *Shell porosity-dependent `m` constant* = `ABSENT — ships with no default`; `SB-MIN-022`; ESC-4; ACQ-4 |
| `EVCL`/`MVCL` Indonesia 1.0/0.5 **+** Nigeria 1.4/0.0 (2 rows) | DEFERRED | **Trigger:** `SAT`. Elan is the only source documenting the `Vcl → 1` singularity and its mitigation — that warning must travel with the equation |
| `B fact Juhasz` 1.0, **meq/ml** | DEFERRED (value) / ADOPTED (unit) | **Trigger:** `SAT` for the value. The **unit** is load-bearing here and is adopted: it is *"the one CEC-family unit IP does state"*, and it corroborates `SB-MIN-011`'s meq/g-vs-meq/mL discipline |
| `B` (Waxman-Smits) `(−1.28 + 0.225T − 0.0004059T²)/(1 + Rw^1.23(0.045T − 0.27))`, T in °C | DEFERRED | **Trigger:** `SAT`. Verified 4× in IP 2025, identical in 2018, and it matches Jauhar's own `reference_waxman_smits_b` note — carry that corroboration across rather than re-deriving it |
| `Cbw` `0.0007·(T+8.5)·(T+298)` | ADOPTED | §5 *Bound-water conductivity coefficient* `0.0007` and *temperature term* `8.5`; cross-checked Geolog §E against Elan Eq 62; ACQ-3 |
| `V_Q^H` `96/(T+298)` → 0.297 cm³/meq at 25 °C | ADOPTED | §5 *Bound-water CEC constant* `96` and *temperature offset* `298`; `SB-MIN-006`; the vendor cross-check (Elan's 0.28 cm³/meq at room temperature, a 6 % agreement) is what makes the row two-sourced |
| `α` cutoff 20 455 ppm (= 0.35 mol/L NaCl) | ADOPTED | §5 *Diffuse-layer expansion threshold*; `SB-MIN-006`; `SB-MIN-T06` |
| `α` cap 5.0 | ADOPTED | §5 *Diffuse-layer expansion cap*, labelled **SandiBumi engineering guard — no vendor source**. A guard honestly labelled is not a parameter smuggled in |
| `obm_sxo_max` 0.5 | DEFERRED | **Trigger:** `SB-MIN-042`'s OBM constraint pair. Must ship under the split name (C-5.7) |
| `wbm_invasion_factor` 2.0 | DEFERRED | **Trigger:** the WBM `Sxo(Sw)` route. Same split-name obligation; the collision is the whole point of C-5.7 |
| `Sxo Limit` exponent 0.2 | DEFERRED | **Trigger:** `SAT` |
| `Qv 'a' Const` / `Qv 'b' Const` 0.5 / −3 | **REJECTED** | Rejected as a default on the dossier's own evidence: the source string reads *"a vendor EXAMPLE, not a stated default"*. A screenshot value is not a vendor default, and adopting one would fail `SB-MIN-009`'s provenance gate |
| `Rw` fallback 0.1 @ 60 °F | DEFERRED | **Trigger:** `SAT`. Recorded with a condition: IP applies it silently when the field is empty, which is the same defect class as `SB-MIN-045`'s temperature fallback — **if it is adopted it must be recorded in the run record, never applied silently** |

**Clay parameters — 7 dossier rows in 7.** The most load-bearing block in the dossier, and the one
where the chapter found more than the dossier did.

| Dossier clay row | Disposition | Where it landed |
|---|---|---|
| **Illite** CEC 0.25 · WCLP 0.1555 · ρ_dcl 2.78 (Geolog RF04 6.2) | ADOPTED + ESCALATED | §5 ships CEC 0.25 (Geolog) but WCLP **0.104** (Techlog) — the chimera F-22 names; the Geolog-pair 0.1555 is carried as the competing value. `SB-MIN-008` [P0], ESC-2, `SB-MIN-T08` |
| **Kaolinite** CEC 0.10 · WCLP 0.06489 · ρ_dcl 2.62 | ADOPTED + ESCALATED | Same pattern: §5 ships CEC 0.10 with Techlog WCLP 0.058; ρ_dcl 2.62 matches. `SB-MIN-008`, ESC-2 |
| **Chlorite (Mg)** CEC 0.15 · WCLP not held · ρ_dcl 2.67 | ADOPTED + ESCALATED | §5 ships CEC 0.15 (the one three-way-consistent clay) with Techlog WCLP 0.101, and the **Geolog-pair WCLP `ABSENT`** — ESC-2's second half. **Note a divergence the dossier does not carry:** §5's shipped ρ_dcl is **2.81**, not 2.67, and is labelled IP's wet-clay-convention value — exactly what `SB-MIN-010` [P0] exists to make visible |
| **Chlorite (Fe)** CEC 0.15 · ρ_dcl 3.42 | DEFERRED | **Trigger:** an Fe/Mg chlorite split in the library. SandiMin ships one `Chlorite`; a 2.67-vs-3.42 g/cc difference under one name is a silent-wrongness site `SB-MIN-028` would surface |
| **Smectite** CEC 1.00 · ρ_dcl 2.63; **Techlog's row is a placeholder (CEC 0, ρ 0, φ 0) and must not be used** | ADOPTED | §5's *Montmorillonite / Smectite* CEC 1.0 (Geolog **and** Techlog `CEC_Smectite = 1.0`), WCLP 1.0, ρ 2.63. The placeholder warning is the same sentinel problem as the generic `Clay` row — `SB-MIN-007` [P0] |
| **Glauconite** CEC 0.20 · ρ_dcl 2.85 | ADOPTED + ESCALATED | §5 ships CEC 0.20 with Techlog WCLP 0.156 and the Geolog-pair WCLP `ABSENT` (ESC-2). **Second divergence the dossier does not carry:** §5's shipped ρ_dcl is **2.96**, against the dossier's Geolog RF04 6.2 value of **2.85** — a 3.9 % difference on a shipped `VENDOR-DERIVED` row with no reconciliation. `SB-MIN-009` [P0] is the requirement that must resolve it, and it is listed as an open consequence in §8.17 |
| **Generic Clay** CEC **NO DEFAULT** · WCLP 0.15 (*IP zonal `PhiT Clay`, a zonal parameter not a mineral property*) | ADOPTED, sharpened | §5 ships **both** generic `Clay` CEC and WCLP as `ABSENT`. The chapter is stricter than the dossier on two counts: the shipped code carries CEC `0.00` (a real zero the solver believes) and WCLP `0.120` (matching **no** vendor file examined, not even IP's 0.15). `SB-MIN-007` [P0] forbids the zero; Techlog's own `−9999` sentinel is cited as the vendor precedent for "absent" |

**project-kb precedent and the provenance boundary — 2 dossier items in 2.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| The project-kb note — whole-rock CEC 3.11–7.99 meq/100 g = 0.0311–0.0799 meq/g, ~⅓ of clay-fraction library values; *"a whole-rock CEC must not be entered into the per-clay CEC field"* | **REJECTED for the product** / EVIDENCE-ONLY | REF-12 and CONTRACT §2.3: the range, the client and the project stay in the research file. The **physical rule** it demonstrates is adopted — `SB-MIN-011` [P0] warns below 0.05 meq/g and names only the physical cause |
| The provenance-boundary correction — de-identify the shipped string and test, derive the 0.05 threshold from the shipped library floor (Kaolinite 0.10 meq/g) rather than from client data, and leave carrying the precedent into the product as Jauhar's explicit call | ADOPTED | `SB-MIN-011`'s text and `SB-MIN-T11`; REF-12. **This is the single most important disposition in §8.9**: a dossier that caught its own provenance leak on revision is the reason the chapter's §5 has no client data in it |

### 8.10 §5.3 — applicable `FINDINGS.md` §6 rules (10 rows)

| Rule | Disposition | Where it landed |
|---|---|---|
| **1 — No raster-only truth** | ADOPTED | Every equation the chapter states is stated once, in text; §2's F-rows cite the raster only as evidence. IP's own worst defects here (D-10, D-12) are raster-vs-ASCII drift, which is the rule's own justification |
| **3 — Unit-typed quantities, no magic constants** | ADOPTED | `SB-MIN-044` [P2]; §5's *Electron-density offset/divisor*, the `96`-vs-`0.096` row carrying its ρ unit, and `SB-MIN-011`'s typed CEC unit |
| **7 — Ordinal + semantic-name parameter addressing** | ADOPTED | `SB-MIN-005` [P2] — derive from **kind**, never a name match — and `SB-MIN-028`'s library keys. Ledger C-5.9 is the cautionary case |
| **8 — No bare `SW`** | ADOPTED | `SB-MIN-036` [P2]; `SB-MIN-T37`; ledger D-15 |
| **9 — Defaults are cited or absent** | ADOPTED | `SB-MIN-009` [P0] and `SB-CORE-004`; **the ten `ABSENT — ships with no default` rows in §5 are this rule's output**, and four of them are values the code ships today |
| **10 — Docs generated from code** | ADOPTED | `SB-MIN-032` [P1] (persist the fully resolved parameter set) plus `SB-CORE-004`'s build gate over §5's `Source` column — the chapter's own §5 is written to be the generated artefact's contract, not a parallel copy |
| **11 — Worked examples must reproduce**, qualified: *a defective example becomes a fixture that asserts the defect, never one that reproduces it* | ADOPTED | The qualification is the operative half and is applied at C-5.5 (§8.7): the chapter ships **no** fixture from the inverted-label example. A fixture that silently inherits a vendor defect converts a known bug into a passing test |
| **12 — Per-correlation unit flags; do not carry both forms** | ADOPTED | `SB-MIN-044`; and F-8's "the kg/m³ forms are the same equations — do not carry both" is carried as a positive rule |
| **13 — State the reference convention** | ADOPTED | `SB-MIN-010` [P0] (wet/dry on every clay row and curve), `SB-MIN-011` [P0] (CEC unit), `SB-MIN-045` [P2] (temperature window and unit). Δ-4(c) is what this rule looks like when it fails |
| **14 — Silent failures are bugs** | ADOPTED | `SB-MIN-033` [P1] (name the conflicting rows), `SB-MIN-015` [P1] (refuse to present an unstable solve as trusted), `SB-MIN-007` [P0] (never read an absent parameter as zero), `SB-MIN-045` (record any fallback). **This rule is the chapter's spine** — CONTRACT §5.3, "fail loud where they fail silent" |

### 8.11 §5.4 — dossier validation and regression tests, T-1 … T-25 (25 rows)

**Note the id collision:** the dossier uses `T-n` for these tests **and** for its §6 Techlog
deeper-read escalations. They are different sets; §8.13 disposes of the escalation `T-n`. Chapter
test ids are always written `SB-MIN-Tnn` and are never renumbered.

| Dossier test | Disposition | Where it landed |
|---|---|---|
| **T-1** IP salinity round-trip (`Rmf 0.1 @ 60 °F` → 87 700 vs printed 87.8 Kppm) | DEFERRED | **Trigger:** ESC-5's resolution. Recorded for a second reason: this fixture **is** the evidence that the shipped `Rw`→salinity fit is IP's E4, which §5's row does not say |
| **T-2** IP bound-water coefficient arithmetic (`0.156/(1−0.156) = 0.1848` vs printed 0.185) | ADOPTED | `SB-MIN-T27` — the `k = WCLP/(1−WCLP)` identity is the WCLP route's contract |
| **T-3** Geolog bound-water multiplier (`k = 0.1841` at CEC 0.25 / ρ 2.78 / T 64.4 °C, and the WCLP equivalence) | ADOPTED | `SB-MIN-T05`; and the same three numbers are the fixture inside `SB-MIN-T08`'s matched-pair gate, which is why §5 carries 64.4 °C as a parameter row |
| **T-4** U conversion identity to 1e-9, with the `ρ_a` round-trip | ADOPTED | `SB-MIN-T12` |
| **T-5** Hydrocarbon-HI unit equivalence (Geolog kg/m³ ≡ IP g/cc to machine precision) | ADOPTED (in part) / DEFERRED | `SB-MIN-T42` covers unit invariance as a class. **Trigger for the specific HI fixture:** the HC correlations shipping (F-8) |
| **T-6** Elan wet↔dry clay round-trip closes; Eq 11 reproduces `ρ_wcl` | ADOPTED | `SB-MIN-T24` — which **fails today**, because `ρ_bw` is hard-coded 1.0 |
| **T-7** Variable-`m` cross-vendor identity over `Qv ∈ [0,2]`, `φT ∈ [0.02,0.4]` | ADOPTED | `SB-MIN-T23` |
| **T-8** Objective-statistic contract — 6 tools at 1σ → `RECON 1.000`, `QUALITY 0.876`, `TOTERR_IP 2.449`, never interchanged | ADOPTED | `SB-MIN-T14`, `SB-MIN-T13` |
| **T-9** Bound-water convention exclusivity; the mixed `0.15/−1` case fails validation | DEFERRED | OPEN-10. **Trigger:** IP's E33/E34 constant-equation bound-water types being implemented. The exclusivity *principle* is already `SB-MIN-040` |
| **T-10** `.neu` integrity — **both** C-5.6 defects, the milder φ=.25 case being the real test | DEFERRED (to `ENV`) | **Trigger:** the `.neu` loader. Carried with its warning intact: a detector tuned to the gross outlier passes (a) and misses (b). The table content itself is refused here at REF-9 |
| **T-11** Nomenclature — no curve named exactly `SW` | ADOPTED | `SB-MIN-T37` |
| **T-12** Unit invariance, metric vs imperial, to machine precision | ADOPTED | `SB-MIN-T42`; `SB-MIN-044` [P2] |
| **T-13** A clay endpoint row with no wet/dry flag fails to load | ADOPTED | `SB-MIN-T10`; `SB-MIN-010` [P0] |
| **T-14** Bounded-vs-active-set divergence is computed, asserted different, and noted in the run report | ADOPTED | `SB-MIN-T02`; `SB-MIN-002` [P2] |
| **T-15** Salinity bound-water sensitivity — `V_bw` 0.0419 / 0.0670 / 0.1094 at salinity 20455 / 8000 / 3000 | ADOPTED | `SB-MIN-T06` — the α-path regression. These are the same three values ESC-5 quantifies its salinity-error consequence against |
| **T-16** BLSO field fixtures — 36 training LAS end-to-end, no NaN, `CONDNUM < 8` on ≥ 90 % of frames | DEFERRED, **and deliberately not transcribed** | **Trigger:** an end-to-end fixture harness. Adopting the test *as named* would carry a real field name into a PRD chapter; CONTRACT §2.3 flags inherited field names for scrub and forbids adding any. The **shape** of the test is what transfers: a real multi-well dataset, asserted on conditioning rather than on volumes |
| **T-17** `a ≠ 1` **and** a conductive non-clay mineral **and** a linearised-Archie row → C-5.4 warning | DEFERRED | **Trigger:** a conductive-mineral row shipping (§8.7, C-5.4) |
| **T-18** CEC unit guard, de-identified — warn below 0.05 meq/g, justified against the library floor, **no client name or range in the test, fixture or string** | ADOPTED | `SB-MIN-T11`; `SB-MIN-011` [P0]; REF-12. The de-identification is adopted as a **binding property of the test**, not as a stylistic preference |
| **T-19** `active = false` vs `WM = 1e-6` produce different `dof`, `n_live_tools` and `CONDNUM` | ADOPTED | `SB-MIN-T17`; `SB-MIN-017` [P1] |
| **T-20** `RECON` ≡ Elan `SDR` computed the long way with `LargestWeight` retained, to 1e-12 | ADOPTED | `SB-MIN-T13`. Its stated purpose — guarding the cancellation proof against a future "optimisation" that reintroduces `LW` asymmetrically — is carried into the test's rationale |
| **T-21** `Tool`-row DOF accounting: a `Tool` tie increments `n_tool`, a HARD equality does not | ADOPTED | `SB-MIN-T16`, `SB-MIN-T35`; `SB-MIN-016` [P0] |
| **T-22** ELAN-Simandoux with no `Silt` component fails validation with a named error | ADOPTED | `SB-MIN-T30`; `SB-MIN-030` [P3] |
| **T-23** Degenerate clay row (`CEC = 0` **and** `WCLP = 0`) fails validation at three levels, under both `PorositySource` values | ADOPTED | `SB-MIN-T07`; `SB-MIN-007` [P0]. The chapter widened it: the shipped **generic `Clay`** row, not only Techlog's Smectite, is in exactly that state (`CEC 0.00`, `multimin2.rs:2098`) |
| **T-24** Monte Carlo is seeded and bit-reproducible; the seed appears in the run report | ADOPTED | `SB-MIN-T38`; `SB-MIN-037` [P3]; REF-3. Its stated purpose — *so the divergence cannot be optimised away* — is why the requirement is written as a prohibition on clock seeding, not a preference for seeds |
| **T-25** C-5.5 defective worked example — assert the defect, do not inherit it; parameterised on the exponent because the example is degenerate at `m = 2` | EVIDENCE-ONLY | The chapter ships **no fixture from that example** (§8.7). `SB-MIN-T21` uses the `m = 2.5` discriminator instead — which is the same insight T-25's own CT-3 guard states, applied by choosing a better fixture rather than by qualifying a bad one |

### 8.12 §5.5 — spec-vs-code deltas (6 rows: Δ-1, Δ-2, Δ-3, and Δ-4 split three ways)

| Delta | Disposition | Where it landed |
|---|---|---|
| **Δ-1** POROSITY / BNDWAT ship as the σ = 0.01 pseudo-measurement **half only**; the hard-equality half is not imposed | ADOPTED | `SB-MIN-035` [P1] — hard equality **plus** pseudo-measurement, with the tie residual emitted as a QC curve so a violation is visible rather than absorbed; `SB-MIN-T35`. §3.3 records the as-built as `PRESENT-DIVERGENT` |
| **Δ-2** WBM is enforced by a **re-solve when violated** — IP's pattern, not Geolog's or Elan's | ADOPTED + ESCALATED | `SB-MIN-034` [P1] (hard inequality, iterated to feasibility); `SB-MIN-T34`, whose case (b) pins the specific defect — the appended row is a σ-weighted **equality at RHS 0** (`multimin2.rs:1409`–`:1410`), so a solve that already satisfied the constraint is still perturbed. The two-constraint interaction is ESC-3 (dossier E-8) |
| **Δ-3** `RECON` is cited to Elan Eq 79 in the code and to **Mayer & Sibbit SPE 9341 (1980)** in the module header | ADOPTED | `SB-MIN-013` [P1] — **both citations stay**: SPE 9341 is the primary source for the family, Elan Eq 79/80 the specific form the normalisation matches, and §2.2's cancellation proof makes the Eq-80 equivalence exact. ACQ-1 records that SPE 9341 is not held |
| **Δ-4(a)** The `else { 0.0 }` branch: a clay with `CEC = 0` and `Phicl = 0` yields `k = 0` — **bound water silently vanishes under either `PorositySource`**, with no error, warning or NaN | ADOPTED | `SB-MIN-007` [P0] — refuse the row, never treat an absent parameter as zero; `SB-MIN-T07`. **The chapter found the defect is live in SandiBumi's own shipped library**, not only reachable through Techlog's Smectite row: generic `Clay` ships `CEC 0.00` at `multimin2.rs:2098` |
| **Δ-4(b)** Two code comments assert a vendor value the shipped table contradicts (`"only smectite carries φ = 1.0"` against `QM_MineralTable.xml` `Phicl = 0`) | ADOPTED | Recorded in §3.4 as part of the `PRESENT-DIVERGENT` finding; `SB-MIN-009` [P0] is the requirement that makes a comment asserting a vendor value a checkable claim rather than folklore |
| **Δ-4(c)** *"The library's WCLP column is mis-attributed"* — Illite 0.104 and Montmorillonite 1.0 said to *"match neither vendor"* | **REJECTED — rebutted on T1 evidence** | F-21 (NEW this session): Techlog ships **two mutually inconsistent clay libraries in one install**, and `QElan_PostProcess_Using_Conductivities.py` gives exactly `WCLP_Illite = 0.104` and `WCLP_Smectite = 1`. The values are **correctly attributed to Techlog** — to Techlog's *other* library. The real defect is different and worse, and is `SB-MIN-008` [P0]: `CEC` and `WCLP` are taken from **different libraries**, breaking the matched pair (F-22). Δ-4(c)'s remedy (3) — every WCLP value carries its real provenance string — is adopted at `SB-MIN-009` |

### 8.13 §6 — gaps and escalations (31 rows)

**Live-install reads — 4 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **E-2** CEC / Qv endpoint unit, stated rather than inferred | ESCALATED | OPEN-7(a); converts `SB-MIN-011`'s meq/g from an inference into a vendor statement, and closes C-OPEN-2/3 |
| **E-3** `MINDEF.PAR` `XPL 2.59` / `XAN 2.74` mineral identity | ESCALATED | OPEN-7(b); ledger C-OPEN-4. **Not guessed** |
| **E-4** IP's full mineral drop-down roster and smectite endpoint set | ESCALATED | OPEN-7(c); bears on `SB-MIN-028`'s roster and on §2.16c's withdrawn smectite claim |
| **E-5** `MINEQDEF.PAR` verbatim, to confirm the IP-parity column lost nothing in transcription | ESCALATED | OPEN-7(d); gates the IP-parity uncertainty library in `SB-MIN-020` |

**Deeper Techlog reads — 6 rows.** *(These are the dossier's `T-n` escalations, not its `T-n`
tests.)*

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **T-1** Elan Table 27 / neutron response-parameter subset and the matrix/fluid-term equations | DEFERRED | OPEN-2; would give a third independent neutron matrix model. Hands to `ENV` |
| **T-2** The `p.u.`-vs-`v/v` conflict on `WCLP` and Elan's global unit convention | **PART-CLOSED** + DEFERRED | **The WCLP half is closed by this chapter on T1 evidence** (`WCLP_*_unit = u"m3/m3"`) → `SB-MIN-027` [P0] and §5's *Wet-clay-porosity unit* row. The **global `φₑ` half** remains OPEN-3 and still gates ESC-4 |
| **T-3** Elan Table 31, geochemical uncertainties | DEFERRED | ACQ-9. **Trigger:** ECS/spectroscopy rows |
| **T-4** Elan Tables 25 / 26 / 32 — dual-water, linear-conductivity, sonic-clay-constraint parameters | DEFERRED | OPEN-4. **Trigger:** the matching options shipping |
| **T-5** `petrophysics-inversion-constants.html` → `QUANTI_INVERSION_CONSTANTS.xml`, a **fourth** Techlog endpoint table no ingest has touched | DEFERRED | OPEN-5. Bears directly on `SB-MIN-028`: the chapter believes there are three libraries and there may be four |
| **T-6** Elan invasion model and `EQHY`; Geolog's `EquivFluids` is *probably* the same object, **unverified** | DEFERRED | OPEN-6. **Trigger:** `SB-MIN-025`'s invasion work. The unverified equivalence is recorded as unverified — a "probably the same object" is exactly the assumption that ships a wrong parameter |

**Live IP session or numeric round-trip — 6 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **E-1** D-10, the Shell constant — three resolution paths (Jauhar's call, a live `mVar` read, or Elan's φₑ unit) | ESCALATED | ESC-4; ACQ-4; `SB-MIN-022` |
| **E-6** IP's non-negativity behaviour on a marginal mineral — confirm the column is dropped, not clamped | ESCALATED | OPEN-7(e). **It is the factual basis of `SB-MIN-001`/`SB-MIN-002`'s divergence claim**, which is why it is recorded rather than assumed |
| **E-7** `Sal` unit in `U_wat = 0.00481·Sal + 0.3883` | ESCALATED | OPEN-7(f); ledger C-OPEN-8, now corroborated twice but still unstated by the vendor |
| **E-8** The WBM re-solve loop's two-constraint behaviour — *a SandiBumi-internal escalation, resolvable without any vendor source* | ESCALATED | ESC-3; `SB-MIN-034` [P1]; `SB-MIN-T34` |
| **E-9** Whether Elan's per-clay `Rsh` has any IP or Geolog counterpart | ESCALATED | ESC-7; `SB-MIN-031` [P3]. The dossier's own instruction — *state it as such rather than assume it* — is why `SB-MIN-031` claims a capability and not a vendor gap |
| **E-10** CT-3, the conductivity root exponent — *"the highest-value 10 minutes in this dossier"* | ESCALATED | ESC-1; `SB-MIN-021` [P1]; `SB-MIN-T21`. **Blocks any IP-parity claim on a conductivity row at `m ≠ 2`** |

**Named papers — 8 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **P-1** Segesman & Liu (1971), excavation | ESCALATED (acquisition) | ACQ-5. Needed only if an excavation term ships; ledger D-09 |
| **P-2** The Simandoux lineage — Simandoux 1963, Schlumberger 1972, Worthington 1985, Poupon et al. 1967, plus the SPWLA Shaly Sand Reprint | ESCALATED (acquisition) | ACQ-6; `SB-MIN-030` [P3]. The chapter carries the corrected attribution: **Poupon, Loy & Tixier 1954 is not on Elan's page** and must not be credited to Techlog |
| **P-2b** Poupon, Loy & Tixier (1954), Trans AIME 6(06):138–145; Aguilera (1990), The Log Analyst Sept–Oct — **IP's** citations | ESCALATED (acquisition) | ACQ-7, with Woodhouse (1976). Needed only if the Poupon variants ship; `SAT` inherits them |
| **P-3** Clavier, Coates & Dumanoir — `V_Q^H` and the α expansion | ESCALATED (acquisition) | ACQ-3. **The highest-value acquisition in the domain** — it is the only one that would upgrade a *shipping* P1 requirement's evidence tier (`SB-MIN-006`, currently `PRESENT-OK` on T3) |
| **P-4** The `Cbw` temperature form — which of `0.0007(T+8.5)(T+298)` and `β = (T+8.5)/30.5` is the conductivity and which the ratio | ESCALATED (acquisition) | Folded into ACQ-3, which names the `(T + 8.5)` term explicitly; §5's *Bound-water conductivity temperature term* row carries both vendors |
| **P-5** The sonic-clay-volume Vmatrix-vs-Umatrix construction — **unattributed, likely Schlumberger internal** | ESCALATED (acquisition) | ACQ-10. It is why `SB-MIN-043` offers six of Elan's seven predefined constraints and not the seventh |
| **P-6** **Mayer & Sibbit, SPE 9341 (1980)** — the root citation for the whole family, found in SandiMin's own module header and named by **no** ingest report | ESCALATED (acquisition) | ACQ-1; Δ-3; `SB-MIN-013`. *"Obtain it before finalising the objective-function spec"* is carried as ACQ-1's stated purpose |
| **P-7** **Hill, Shirley & Klein (1979), SPWLA 20th, Paper AA** — the only admissible resolution for D-07 | ESCALATED (acquisition) | ACQ-2; REF-6. *"Obtain before shipping any Hill-Shirley-Klein-family bound-water route"* is binding, and no such route is offered in §4 |

**Structural gaps — 7 rows.**

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **G-1** Geolog has no T1 evidence and cannot get any here; `Multimin_Knowledge_Transfer.pdf` **not ingested** | ESCALATED | OPEN-1; also half of ESC-2's blocker. *"Recommend ingesting before any further Geolog-parity work"* is carried verbatim in intent |
| **G-2** Geolog's per-vendor non-linear neutron fit coefficients — named, not transcribed | DEFERRED | §8.3's §1.4 row. **Trigger:** `ENV`'s neutron work. The dossier's own restraint here is the same discipline as REF-9 |
| **G-3** IP's cased-hole Sigma is undocumented; **parity is not achievable from the manual — do not claim it** | ADOPTED (as a prohibition) | §8.6's second row 20; `SB-MIN-002`'s disclosure duty and the §1 boundary. The prohibition binds now even though the feature is deferred |
| **G-4** IP's Elan `.elp` import loads minerals, fluids, equations and endpoints **only** — not constraints, not mixings; `ElanToIPMapping.par` is the name Rosetta stone | DEFERRED | **Trigger:** any vendor model-file import. Recorded with its trap: an import that silently drops the constraint half produces a model that looks complete and solves differently |
| **G-5** Techlog's `Phicl`/`Rhobdcl`/`Rhobwcl` internal inconsistency — *cannot be resolved from the files held* | ESCALATED | ESC-6; F-15; `SB-MIN-046` [P2]; `SB-MIN-T44` |
| **G-6** **Tier-C boundary declaration**, including the ruling that IP's Wyllie↔Hunt-Raymer `Cp` bridge is Tier C for adoption, cited by location and character only | ADOPTED (as a refusal) | REF-11, CONTRACT §2.2. The dossier's own self-correction — it once printed the coefficient set *inside* the sentence declaring it Tier C — is honoured here: **this chapter repeats no coefficient of that fit in any section** |
| **G-7** **No vendor chart lookup-table data was transcribed**; `.neu` cited by format and convention only, the `-.1960` outlier as a QC fact not as data | ADOPTED (as a refusal) | REF-9, CONTRACT §2.1. §5's own preamble states the same boundary for this chapter, and `SB-MIN-026` keeps the table an input rather than content |

### 8.14 §7 — source register (7 rows)

The register is evidence about evidence: it is what makes every tier tag in §2 and every `Source`
cell in §5 checkable by a reader who has this machine.

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **§7.1 T1** — executable source and shipped data files (`multimin2.rs`, `QM_MineralTable.xml`, `MINDEF.PAR` / `MINEQDEF.PAR` via `multimin_ip_spec.md`) | EVIDENCE-ONLY | Underwrites every `T1` tag and every `file.rs:line` citation in §3. The chapter **added** one T1 source the dossier did not use: `QElan_PostProcess_Using_Conductivities.py`, which is where F-21 and F-22 come from |
| **§7.2 T2** — full-manual ingest reports (IP 2025 slice C, IP 2018 slice D, the 2025 crosscheck ADDENDUM) | EVIDENCE-ONLY | Underwrites every `T2` tag; the ADDENDUM's precedence rule (2025 overrides 2018 where they disagree) is what makes the D-10 three-way statable |
| **§7.3 T2-equivalent** — 19 Elan theory pages and 14 equation/table rasters read directly | EVIDENCE-ONLY | Underwrites every `T2-equiv` tag. Without it the Elan leg would be T3 and F-3's cancellation proof would not exist |
| **§7.4 T3** — install-tree and catalog ingests (`techlog_ingest/*`, `ip_ingest/E_threeway_endpoint_compare.json`, `multimin_ref_spec.md`) | EVIDENCE-ONLY | Underwrites every `T3` tag — which is **every Geolog claim in the chapter**. One member is explicitly refused for use: the `E_threeway` CEC column (REF-7) |
| **§7.5 T4** — course notes and decks (petro-kb `geolog` category; the un-ingested Multimin PDF) | EVIDENCE-ONLY | The un-ingested member is OPEN-1. No §5 row rests on T4 alone |
| **§7.6 Memory notes** | EVIDENCE-ONLY | Corroboration only, never provenance: `reference_tool_response_constants` corroborates the U-not-Pe rule and `reference_waxman_smits_b` the `B(T,Rw)` form. **A memory note is not a citable source for a shipped parameter** — every §5 row still names its vendor or paper |
| **§7.7 project-kb decision records** (real-names / local-only) | EVIDENCE-ONLY, **firewalled** | REF-12 and CONTRACT §2.3. The register is where the project-kb precedent legitimately lives; the product side of that boundary is `SB-MIN-011`'s de-identified rule |

### 8.15 §8 — critique disposition (23 rows)

The dossier was reviewed hostilely (0 blockers, 9 majors, 13 minors; 21 of 22 applied, one partially
rebutted). Its dispositions are carried here because **several of the chapter's own requirements
exist only because the critique forced a re-verification** — and because a finding the dossier
rebutted must not be quietly reinstated by this chapter.

| Dossier item | Disposition | Where it landed |
|---|---|---|
| **Blockers — none raised.** The critique hunted specifically for invented or uncited `m`, `n`, `a`, `Rw`, ρma, cutoffs, endpoints and coefficients and **found none** | EVIDENCE-ONLY | The single most load-bearing line in the critique for this chapter: it is the external check that the dossier's parameters are cited rather than invented, which is the precondition for §5 existing at all |
| **MAJ-1** Max models 20 → 50, plus an unflagged slice-C-vs-slice-O contradiction | DEFERRED | Ledger CT-1 (§8.7). **Trigger:** multi-model support |
| **MAJ-2** IP's Monte Carlo entirely absent | ADOPTED | §2.19 → `SB-MIN-037` [P3], `SB-MIN-038`, `SB-MIN-039`; §5's MC block; REF-3. **This whole requirement group exists because of MAJ-2** |
| **MAJ-3** Fabricated provenance for the Poupon equations and a citation misattributed to Techlog | ADOPTED | ACQ-6 / ACQ-7 carry the corrected attributions; §8.3 records IP's three Tier-B citations; CT-2 records the counting artefact. **The chapter must not re-credit Poupon-Loy-Tixier 1954 to Techlog** |
| **MAJ-4** Unflagged conflict on the conductivity root exponent — **partially rebutted** | ADOPTED (with the rebuttal preserved) | ESC-1, `SB-MIN-021`. The rebuttal is load-bearing and is carried: **at `m = 2` the two disputed readings coincide**, so the 5.3 % Geolog-vs-IP gap holds under either reading, while "IP uses a fixed square root" is withdrawn. The chapter states the comparison scoped to `m = 2` and uses `m = 2.5` as the discriminator |
| **MAJ-5** §3.1's σ arithmetic wrong on both figures | ADOPTED | The corrected 0.195 σ / 0.445 σ, **and the re-argued mechanism**: the portability breaker is that active-set deletion changes the *dimension of the system*, so volumes are discontinuous in depth. That re-argument is `SB-MIN-001`'s and `SB-MIN-002`'s rationale — a corrected number that no longer carried its own argument was replaced by the right argument, not by a bigger number |
| **MAJ-6** §3.5's gas spread is 45 µs/ft, not 76 — a 69 % overstatement from a cross-row subtraction | ADOPTED | F-5 and `SB-MIN-029` [P3] use **45**; §5's gas `DT` row. Geolog's oil `DT` genuinely equalling its water `DT` is stated as correct-as-sourced |
| **MAJ-7** §4.1's "every ledger item" omitted four; D-07 kills the unit story | ADOPTED | §8.7 disposes of all sixteen plus the three CT items, with D-07 split into (a) and (b) exactly as the critique's resolution required; ACQ-2 |
| **MAJ-8** The "already special-cases Techlog's Smectite" claim is false, and hides a fourth delta | ADOPTED (and extended) | Δ-4 → `SB-MIN-007` [P0], `SB-MIN-T07`. **The chapter extended it twice**: the same silent-zero state is live in SandiBumi's own generic `Clay` row, and Δ-4(c)'s mis-attribution claim is itself rebutted by F-21 (§8.12) |
| **MAJ-9** Client-identifying provenance directed into shipping artefacts | ADOPTED | REF-12; `SB-MIN-011`; `SB-MIN-T11`. **The chapter treats this as binding rather than advisory**, and CONTRACT §2.3 independently requires it |
| **MIN-1** Geolog `k_clay` unit silently corrected then reported "confirmed verbatim" | ADOPTED | §8.1; §5's `96` row carries the `g/cc`-scaled form that reproduces the vendor's own verified 0.1841 |
| **MIN-2** `E_endpoint_DIFF_vs_sandimin.json` is 50 rows, not 40 | EVIDENCE-ONLY | Register hygiene; no chapter claim rests on the count |
| **MIN-3** Padded source register — ADDENDUM items 3 and 4 | EVIDENCE-ONLY | Item 4 (MC 2000/300/200) became load-bearing in §5's MC block; item 3 is out of domain |
| **MIN-4** "C-OPEN-10 routed to slice H" is not in the source | ADOPTED | §8.3's §1.4 row records the removal. **An inferred routing statement is exactly the kind of plausible addition this chapter's §3 discipline exists to prevent** |
| **MIN-5** Spliced quotation from `multimin2.rs` in §3.3 | ADOPTED | Δ-3; the two locations are cited separately, and the header's Mayer & Sibbit attribution became ACQ-1 |
| **MIN-6** Spliced attribution in §2.8 | ADOPTED | REF-7 — the finding now rests entirely on arithmetic verified to 4 dp, which is why the rejection is defensible without the note |
| **MIN-7** F-4 keeps UNITY HARD while switching the others to Geolog `Tool`, silently | ADOPTED | §8.8's F-4a; `SB-MIN-003` + `SB-MIN-004`; the `unity_mode = tool` parity switch is §8.9's second solver-control row |
| **MIN-8** §2.4's Table-29 counting mixes in a Table-30 row | ADOPTED | §8.4's §2.4b — **six** exceptions, which is the whole evidence base for REF-5 and `SB-MIN-019` |
| **MIN-9** §3.4 uses "2.43", which appears in no source | ADOPTED | F-4 uses **2.4** with the 5.7 σ / 6.9 pu stake, and the full cited wet-clay spread is bounded rather than pinned to one screenshot |
| **MIN-10** §2.7 leaves the p.u.-vs-v/v WCLP conflict unadjudicated when Eq 11 is decisive | ADOPTED, **then closed** | The dossier's leaning (17 p.u. would give a 0.905 g/cc dry clay, lighter than water) became a **statement** in this chapter on T1 evidence → `SB-MIN-027` [P0]. OPEN-3 keeps the global half open |
| **MIN-11** C-5.6 records two `.neu` defects; only one was carried | ADOPTED | §8.7's C-5.6 row and §8.11's T-10 row carry both, with the warning that case (b) is the real test |
| **MIN-12** Tier-C self-contradiction — the `Cp` coefficient set printed inside its own Tier-C declaration | ADOPTED | REF-11. **This chapter repeats no coefficient of that fit anywhere**, which is the corrected behaviour applied rather than merely acknowledged |
| **MIN-13** IP's Logging Contractor header field silently changes Mineral Solver results | ADOPTED | F-11 → `SB-MIN-026` [P2], `SB-MIN-T26`; §4 adoption item 21. A header dropdown that changes numbers is the canonical CONTRACT §5.3 case |

### 8.16 Surplus — chapter content with no dossier antecedent (6 rows)

These did **not** come from the dossier. They came from this chapter's own T1 reads, from §3's
reading of the shipped source, or from the `SB-CORE-nnn` spine. They are enumerated here rather than
folded into §8.1–§8.15, so that "every dossier item is accounted for" stays a checkable claim in
both directions.

| Chapter content | Origin | Why it is not dossier-derived |
|---|---|---|
| **F-21** and **F-22**, and the requirement they produce: `SB-MIN-008` [P0] *Ship `CEC` and `WCLP` only as a matched pair from one library*, with `SB-MIN-T08` and §5's four matched-pair rows (tolerance 0.02, reference T 64.4 °C) | This session's T1 read of Techlog's `QElan_PostProcess_Using_Conductivities.py`, a shipped file **no ingest report and no dossier section had opened** | The dossier's nearest item is Δ-4(c), which claims the shipped WCLP column is *mis-attributed*. **That claim is rejected** (§8.12): the values are correctly attributed to Techlog's *second* library. The real defect — a `(CEC, WCLP)` pair split across two vendors, worth **1.70 pu of PHIE on Illite** — is visible only once both Techlog libraries are read side by side, which is what F-21 does |
| **`SB-MIN-032`** [P1] *Persist the fully resolved parameter set with every run*, with `SB-MIN-T32` | CONTRACT §5.4 (provenance is structural) and `SB-CORE-004` | No dossier item asks for it. The nearest relatives are rule 10 (docs generated from code) and T-24 (MC replay), both narrower. Replay of a run needs the **resolved** set — after defaults, overrides and library selection — which no vendor persists and no dossier row requests |
| **`SB-MIN-041`** [P0] *Keep retired modules resolvable and refuse to run them, carrying no orphan defaults*, with `SB-MIN-T40` and `SB-MIN-T41` | §3.6's reading of the shipped source: `multimin.rs` (67 lines, retired), `modules.rs:403`–`:411` / `:418`–`:420` / `:382`, and the test at `:2856`–`:2862` | The dossier compares three vendors' solvers; it has no view of SandiBumi's own module lifecycle. The finding is a **live residue** — a retired spec still rendering `RHOB_CLAY 2.55` and `PEF_CLAY 3.10` against live values of 2.65 and 3.50, with no source for either. `SB-MIN-T41` fails today |
| **`SB-MIN-045`** [P2] *Bound the formation temperature and record any fallback*, with `SB-MIN-T43` and §5's `FTEMP_MIN_F` / `FTEMP_MAX_F` row | §3.5/§3.7's as-built read (`multimin2.rs:775`–`:776`) | No vendor documents a temperature validity window and no dossier item asks for one. It exists because temperature enters bound water through both `(T + 298)` and `(T + 8.5)`, so a `−999.25` or `9999` fill surviving to the solver produces a *plausible* answer — the silent-wrongness class CONTRACT §5.3 names |
| **`SB-MIN-009`**'s scope — provenance on every **value**, not every **column** — and the explicit `SB-CORE-004` / `SB-CORE-005` discharge in §5's preamble | The spine, not the dossier | The dossier's rule 9 requires a source string per parameter *row*. `SB-MIN-009` is stricter, because §5's own clay block proves a single row can carry values from two vendors: the discharge has to bind at value granularity or `SB-MIN-008`'s defect passes it |
| **Two shipped clay densities no dossier row accounts for**: library `Glauconite` ρ_dcl **2.96 g/cc** against the dossier's Geolog RF04 6.2 value **2.85** (a 3.9 % gap), and `Chlorite` **2.81** against Geolog's Mg 2.67 / Fe 3.42 | Reconciling §5's shipped rows against §8.9's dossier rows while writing this table | Neither divergence appears anywhere in the dossier, which lists the Geolog values without checking them against what SandiMin ships. Both rows are `VENDOR-DERIVED` with no reconciliation. `SB-MIN-009` [P0] is the requirement that must resolve them; **they are open consequences of this chapter, not closed findings** |

### 8.17 Reconciliation

**The arithmetic.** §8.1–§8.15 contain **290 disposition rows**, accounting for **327 dossier
items**. §8.16 adds **6 surplus rows**, giving **296 rows in §8 as a whole**. The two totals differ
because of the two declared grouping exceptions and nothing else: §8.9 carries 86 dossier parameter
rows in 47 rows (+39), and §8.8 carries 11 canonical forms in 13 rows (−2). `290 + 39 − 2 = 327`.

**Counts that differ from a naive reading of the dossier, each with its cause.**

1. **§4 numbers its rows 1 … 21 but contains 22.** The number **20 is used twice** — *Uncertainty
   propagation* and, after 21, *Sigma / cased hole*. Both were added on the dossier's revision pass
   and the second was never renumbered. §8.6 disposes of 22.
2. **§5.5 says "Four" deltas; §8.12 has six.** Δ-4 carries three lettered sub-items, each with
   separate evidence and — decisively — **separate dispositions**: (a) and (b) are ADOPTED, (c) is
   **REJECTED**. Collapsing them would hide the rebuttal.
3. **§4.1's sixteen ledger rows become seventeen.** D-07 is disposed of in two halves by the dossier
   itself: the equation stays open (REJECTED here), while its **unit statement is a genuine vendor
   statement** (ADOPTED here). A defective equation carrying a sound unit statement cannot have one
   disposition.
4. **§5.2 is one heading and 86 rows.** A reader counting tables finds six; a reader counting
   parameters finds 84 plus the two project-kb/provenance notes. §8.9 states the underlying count in
   every grouped row, so the 86 is recoverable.
5. **The `T-n` identifier collides across two dossier sections.** §5.4 numbers 25 *tests* `T-1 … T-25`
   and §6 numbers 6 *Techlog deeper-read escalations* `T-1 … T-6`. They are unrelated sets. A naive
   de-duplication would silently drop six items; §8.11 and §8.13 dispose of both sets and say so.
6. **The critique's "0 blockers, 9 majors, 13 minors" is 22 findings; §8.15 has 23 rows.** The
   *absence* of blockers is itself dispositioned, because it is the external evidence that the
   dossier's parameters are cited rather than invented — the precondition for §5 existing at all.
7. **§1.4's seven "no evidence held" rows are dispositioned as evidence.** A stated absence is a
   finding; skipping them would make the Geolog leg's T3 ceiling look like an oversight rather than a
   recorded structural limit.

**Deliberate one-to-many mappings.** §2.16 (endpoint libraries) reaches `SB-MIN-009`, `-028`, `-030`,
`-031` and `-046`. §3.4 (wet/dry convention) reaches `SB-MIN-010` and `-024`, and is the site the
chapter's own T1 read extended into `SB-MIN-008`. §1.2's seven predefined inequality constraints
reach `SB-MIN-034`, `-042` and `-043`. F-4's constraint table reaches five requirements and one §5
row.

**Deliberate many-to-one mappings.** `SB-MIN-003` [P0] alone discharges §1.2's constraint classes,
§2.5, §4 item 4, F-4, F-4a and MIN-7. `SB-MIN-011` [P0] discharges §2.7, §2.8, §2.8a, C-OPEN-2,
C-OPEN-3, D-07(b), T-18 and MAJ-9. `SB-MIN-021` [P1] discharges §2.2b, CT-3, E-10 and MAJ-4. This is
why a requirement count (46) and a dossier-item count (327) were never going to match, and why the
completeness gate is the **row** count, not the requirement count.

**Deliberate none-to-one mappings** — six, enumerated at §8.16, four of which produce shipping
requirements (`SB-MIN-008`, `-032`, `-041`, `-045`). Three of those four are `PRESENT-DIVERGENT` or
`PARTIAL` against code that ships today, and two of their tests (`SB-MIN-T41`, and `SB-MIN-T08`'s
matched-pair gate) **fail today**.

**Where the chapter overruled the dossier, and where it rejected it.** One overrule: §4 item 3's
*"derive the default"* is refused by `SB-MIN-019` in favour of *"the printed value wins"*, on the
dossier's own evidence that Elan deviates from Elan's own rule on six rows (REF-5). Six outright
rejections, each with its evidence on its row: D-07's equation (§8.7), IP's malformed `F` as a
shippable form (§8.4 §2.7a), the `E_threeway` IP CEC column (§8.4 §2.8b), the `Qv 'a'/'b'` screenshot
values as defaults (§8.9), the client range as product content (§8.9), and Δ-4(c)'s
mis-attribution claim (§8.12). One split disposition: IP's Monte Carlo contract is adopted **except**
its clock seed, which is rejected (§8.4 §2.19a).

**Where every ESCALATED row goes.** Each resolves to exactly one of **28** named destinations in §7 —
`ESC-1 … ESC-8`, `ACQ-1 … ACQ-10`, `OPEN-1 … OPEN-10`. No row escalates to a general note that more
work exists, per CONTRACT §3.

**Two things a reader must not miss.** First, **`SB-MIN-008` is the most expensive finding in this
chapter and it is surplus** — it exists only because a shipped Techlog file that no ingest had opened
was read this session, and it moves **1.70 pu of PHIE** on a 25 % Illite rock. Second, **this
chapter's own front matter is stale** (OPEN-8): it states 34 acceptance tests against the 44 written,
63 parameter rows against the 78 counted in §5, and 9 P0 against the 10 allocated in §4. The
requirement count of 46 is correct. This task was scoped append-only and did not edit it.

**Completeness statement.** Every numbered finding, inventory entry, comparison block, difference,
adoption choice, ledger row, proposed ledger item, canonical form, parameter row, `FINDINGS` rule,
dossier test, spec-vs-code delta, escalation, named paper, structural gap, source-register entry and
critique finding in `docs/research_2026-08/cross_tool/mineral-solver.md` appears in exactly one row
of §8.1 – §8.15. **No dossier item is unaccounted for.** Nothing was carried into a requirement, a
parameter row or a test without a stated disposition, and nothing was dropped silently: every item
not adopted is DEFERRED with its trigger, REJECTED with its evidence, ESCALATED to a named
destination in §7, or recorded as EVIDENCE-ONLY.
