# 15 — Saturation-height functions and rock typing

**Domain code:** `SHR` · **Requirements:** `SB-SHR-nnn` · **Tests:** `SB-SHR-Tnn`
**Evidence dossier:** `docs/research_2026-08/cross_tool/sat-height-rocktyping.md` (2,480 lines; 25 numbered verification corrections; 19 ranked findings; 13 open escalations)
**Owns a P0 core defect:** the domain half of `SB-CORE-001` (depth unit carried and enforced).

---

## 1. Scope and boundary

### 1.1 What this chapter owns

This chapter specifies everything that turns **height above a free-water level, or a
capillary-pressure measurement, into a saturation** — and everything that partitions rock into
**classes whose petrophysical behaviour is distinct enough to carry their own such law**. The two
halves are one chapter because they are one workflow: a saturation-height function fitted across
mixed rock quality is the single largest avoidable error in a field-scale Sw model, and the only
defensible fix is to fit one law per rock type. Splitting them would put the fit on one side of a
document boundary and the reason it must be split on the other.

Concretely, in scope:

| Group | Content |
|---|---|
| **Height ↔ pressure** | The capillary-pressure law `Pc = grad·(ρw − ρhc)·h`, the hydrostatic gradient constant, the depth-unit arithmetic that law depends on, and the free-water-level reference contract |
| **Saturation-height families** | Cuddy FOIL, Brooks-Corey, Skelt-Harrison, Thomeer (height domain), Leverett-J, Lambda; their fitting, their forward application, and the provenance of the fitted object |
| **FWL determination** | The FOIL free-water-level scan and its uncertainty |
| **Laboratory capillary pressure** | MICP / porous-plate / centrifuge ingest, closure and conformance correction, lab→reservoir σ·cosθ conversion, stress and clay-bound-water corrections, modality |
| **Pore-geometry indicators** | Pittman r10–r75, Winland R35, Aguilera R35, Swanson apex permeability, Thomeer *G*/*Pd*/*B*∞, port-size classification |
| **Flow-unit rock typing** | Amaefule RQI / φz / FZI, Corbett-Potter GHE bins, the FZI clustering methods, the inverse permeability transform |
| **Carbonate rock typing** | Lucia / Jennings-Lucia rock-fabric number and its class bands |
| **Heterogeneity and flow units** | Stratigraphic Modified Lorenz Plot, Lorenz coefficient, flow-unit segmentation, Dykstra-Parsons |
| **Cut-off derivation** | Deriving reservoir/pay cut-offs *from* the rock-typing and capillary-pressure evidence — the derivation only; see §1.2 |

### 1.2 Seams this chapter does not cross

Four boundaries are named by sibling chapters and are picked up or handed back here explicitly.

**`12_saturation.md` owns the electrical Sw models.** `Rw`, the Archie exponents `a`/`m`/`n`, the
Waxman-Smits and Indonesia and Simandoux formulations, and every `Rt`-driven saturation belong
there. This chapter consumes a log-derived `Sw` curve as *training data* for a height fit, and
produces `SWH` as an *independent* saturation — the two are compared, never merged. Where this
chapter names an `Rw` or an `m`, it is citing chapter 12, not defining one.

> One collision is real and is specified here rather than left implicit: the mnemonic **`RQI`** is
> Amaefule's Reservoir Quality Index in this chapter and, in part of the incumbent corpus, a
> quantity inside a shaly-sand saturation model. The two are not the same number and the dossier
> quantifies the confusion at **11.8× in `Swirr`** when one is fed to the other. Disambiguating the
> namespace is `SB-SHR-016`; the saturation-model side of the name stays with chapter 12.

**`14_cutoffs-summation-mc.md` owns the cut-off machinery; this chapter owns the cut-off
derivation.** Chapter 14 records the split and hands two things across:

1. It **explicitly refuses to convert an `HCPV` thickness into a volume inside the summation
   module**, on the grounds that the fluid-gradient conversion belongs here. That conversion is
   the same `Pc = grad·Δρ·h` law this chapter already owns the constant for, so it is picked up:
   `SB-SHR-024` specifies the gradient service that chapter 14 calls, and the constant it uses is
   the single derived one specified in `SB-SHR-006`. Chapter 14 continues to own the summation,
   the net-flag logic and the Monte Carlo.
2. It records that **two named-paper closures for cut-off *selection* are not on this machine** —
   Worthington & Cosentino 2005 and Qassamipour et al. 2020. Those are acquisition gaps against
   the *selection* method, which is chapter 14's. This chapter's `SB-SHR-023` specifies the
   *derivation* input — a cut-off inferred from the rock-type partition and the Pc curve — and does
   not attempt to select a cut-off without them. The gap is restated in §7.5 as inherited, not
   re-escalated.

**`21_data-io.md` owns the parse and carry of the depth unit; `23_plotting-interactivity.md` owns
the renderer constant.** `SB-CORE-001` allocates the *arithmetic* to this chapter. This chapter
therefore specifies what the height-domain equations must do with a declared unit and what they
must do when none is declared — not how the LAS header is read, and not how a track is scaled on
screen. §3.1 nevertheless quotes the carrier's current behaviour, because the domain requirement
cannot be stated without it.

**`17_facies-ml.md` owns learned classifiers.** Electrofacies, GMM facies and any supervised
lithology predictor are theirs. This chapter's rock typing is **deterministic and unsupervised** —
an FZI is computed, not learned; a Ward partition is exact, not trained. The one place the
boundary is genuinely thin is `SB-CORE-014` (a learned model carries its training provenance),
whose scope note states that every chapter that fits anything inherits the gap. This chapter fits
six families and produces cluster centroids, so it inherits it in full; see `SB-SHR-011`.

### 1.3 Standing constraints inherited

- **`SB-CORE-004`** — no parameter ships without a source. Discharged by §5; every row there is
  either cited or recorded `ABSENT`.
- **`SB-CORE-002`** — no degraded result presented as clean. This domain holds both a genuine
  strength and a genuine violation; both are in §3.
- **`SB-CORE-006`** and **`SB-CORE-007`** — one name/one equation, one definition per constant.
  Sibling chapters found real instances of both. So does this one, quantified in §3.3.
- **`SB-CORE-013`** — vendor divergence visible at the point of choice. This domain is where the
  divergences are largest: the dossier's ranked findings include a **10⁴ spread in a shipped
  permeability estimator** and a **2.76× spread in a shipped fluid gradient**.

---

## 2. What the incumbents do — the requirement-bearing findings

Every claim below carries its evidence tier per `CONTRACT.md` §1.2. Only findings that generate a
requirement are here; the dossier's full inventory of 55+ Geolog manifests, 9 Techlog files and 6
IP module groups is not restated, and §8 accounts for what was dropped.

The findings sort into four kinds, in descending order of what they cost a study.

### 2.1 One physical constant, three or four printed roundings — and the roundings disagree

The height→pressure→saturation chain rests on two constants, and neither incumbent derives either.
Both are transcribed at printing precision, and the transcriptions differ.

**The Leverett J constant.** The oilfield-unit form `J = C·(Pc/σcosθ)·√(k/φ)` needs
`C = √(9.869233×10⁻¹³ cm²/mD) × 6.894757×10⁴ dyn·cm⁻²/psi`, which evaluates to
**0.21660110384…** (T1, derived from the CODATA/SPE unit definitions, not read from any vendor).
The incumbents print **0.2166** — low by 0.00051%, immaterial. That is the benchmark against which
SandiBumi's own value is judged in §3.3, and it does not go the same way.

**The hydrostatic gradient per unit specific gravity.** Two independent derivations agree:
imperial `62.428 lb/ft³ ÷ 144 in²/ft² = 0.4335277778 psi/ft` and SI
`1000 kg/m³ × 9.80665 m/s² → 0.4335275224 psi/ft`, agreeing to **5.89×10⁻⁷** (T1, derived).
Against that reference:

| Source | Printed value | Deviation from derived |
|---|---|---|
| IP (T2, module parameter default) | 0.433 psi/ft | **0.1217 % low** |
| Techlog (T2) | 0.0981 bar/m | 0.0342 % high |
| Derived (T1) | 0.4335278 psi/ft | — |

A tenth of a percent in the gradient is a tenth of a percent in `Pc`, which for a Leverett law with
exponent −0.4 is roughly 0.05 % in `Sw` — negligible on its own. It is listed because it is the
*same* constant the whole industry rounds differently, and because **SandiBumi inherited IP's
rounding rather than the derivation** (§3.3).

**The hydrocarbon-gradient defaults are a different order of problem.** Geolog ships **three**
hydrocarbon gradients in different manifests (T1, read from the shipped `.lls`/manifest sources).
Taken at a fixed `Pc = 10 psi`, they place the same capillary entry at **27.9 ft, 54.6 ft and
77.0 ft** above the free-water level — a **2.76× spread in height**, from three defaults inside one
product. This is the clearest `SB-CORE-013` case in the domain: a user who accepts the default
without noticing which manifest supplied it gets a transition zone nearly three times too tall or
too short, and nothing in the output says which one was used.

**The mercury-air interfacial tension** is the same pattern one level down. The pore-throat radius
`r[µm] = 2 × 0.1450377 × σ·|cosθ| / Pc[psi]` reduces, for mercury at σ = 485 dyn/cm and θ = 140°,
to **106.6334 / Pc** (T1, derived). SandiBumi's own reference bank carries **107.6** — 0.907 %
high (§3.3). The relevant point for requirements is that `σ` and `cosθ` must be carried
*separately* and never as a fused product, because a fused product cannot be re-scoped when the
lab system changes.

### 2.2 Conventions that silently flip a number by tens of saturation units

These are the findings that cost the most and are the hardest to see, because both conventions are
correct and neither output is labelled with which one it used.

**Brooks-Corey: λ or 1/N.** Brooks & Corey (1964) write `Se = (Pd/Pc)^λ`. Part of the incumbent
corpus parameterises the same curve as `Se = (Pd/Pc)^(1/N)` and calls the stored coefficient `N`
(T2). Feeding a λ into an `N` slot, or the reverse, is dimensionally legal and numerically
catastrophic: the dossier measures the same rock returning saturations **26 saturation units**
apart. There is no way to tell from the number itself which convention produced it.

**Thomeer: the log base.** Thomeer (1960) writes `Bv/B∞ = exp(−G / log₁₀(Pc/Pd))`. One incumbent
implements the same expression with a natural log (T1, read from the shipped source), which makes
its shape factor `G_Techlog = 2.302585 × G_Thomeer`. Round-tripping a published Thomeer *G* through
that implementation moves the curve by **12 saturation units**. Again: two correct programs, two
incompatible numbers, one shared parameter name.

**Porosity fraction or porosity percent, inside a log-space regression.** Every pore-throat
correlation in this domain regresses on `log φ`, so a unit error is not a factor of 100 — it is
100 raised to the porosity exponent. The dossier computes the exact stakes:

| Correlation | φ exponent | Error if φ is fed in the wrong unit |
|---|---|---|
| Pittman R25 | −1.415 | **100^1.415 = 676.08×** |
| Apex radius correlation | −1.185 | 100^1.185 = 234.42× |
| Oklahoma University | −3.06 | **100^3.06 = 1.318×10⁶** |

A 676× error in a pore-throat radius does not look like a unit error. It looks like a different
rock. This is the single strongest argument in the domain for `SB-CORE-003`-style enforced
preconditions rather than documented ones, and it drives `SB-SHR-019`.

**Lucia rock-fabric number: decimals or percent.** The Jennings-Lucia global transform is
calibrated on interparticle porosity as a **fraction**. The same plug evaluated with φ in percent
returns **RFN 7.41** instead of **RFN 2.07** — Class 2 rock reported as outside the calibrated
band entirely (T1/T2, resolved in the dossier as ledger item E-OPEN-3).

**Beta and the `GFT⁻¹` / `FT⁻¹` mismatch.** One incumbent's Thomeer-family parameter file declares
a unit of `GFT-1` where the consuming equation expects `FT-1` — a factor of **10⁹** (T1, read
directly). Recorded here as evidence that vendor unit *declarations* cannot be trusted as
specifications: they are read by humans and ignored by code.

**σ-scoped coefficients.** One incumbent's Pc handling divides by interfacial tension inside the
transform (`pc /= ift`, T1, read from the shipped source). Every coefficient downstream of that
line is therefore **scoped to a specific σ** — ×72 for a porous-plate system, ×480 for mercury.
Such a coefficient is not portable and must never be adopted as a default in another product; it
is `NON-ADOPTABLE` in §5's sense.

### 2.3 Shipped estimators whose answers disagree by orders of magnitude

**Swanson apex permeability — the largest single divergence in this domain.** Swanson's method
takes the apex of the `Bv/Pc` hyperbola and maps it to permeability by a power law. The
incumbents disagree not on the power law but on **what `Bv` means at the apex**: bulk-volume
fraction, bulk-volume percent, or pore-volume-normalised saturation. The dossier evaluates all
three bases on one sample and gets **0.00864 mD, 0.000193 mD and 1.97 mD** — a spread of **10⁴**.
No incumbent states its basis in its output.

The dossier escalates this as **ESC-7, BLOCKING**, with an explicit instruction: **ship no Swanson
default**. SandiBumi ships one anyway (§3.4). This is the most serious as-built finding in the
chapter after the depth-unit spine.

**The FZI porosity basis.** Amaefule's `φz = φ/(1−φ)` is defined on *effective* porosity. Feeding
total porosity instead changes FZI by a fixed ratio of **1.63583** on the dossier's worked sample —
and, critically, that ratio is **scale-free in permeability**: at k = 10 mD the two FZIs are
1.45282 and 0.88813, and the ratio is identical at any k. A constant multiplicative offset in FZI
is exactly the error a cluster-count sweep cannot detect, because it moves every sample by the
same amount and the partition looks perfectly clean.

### 2.4 Four different automatic answers to "where are the flow units?"

All four incumbent approaches to partitioning an FZI or `k/φ` population are defensible and none
agrees with another on a real well:

| Approach | Source | Tier |
|---|---|---|
| Second-derivative-of-Lorenz inflection detection | Techlog `RFUIStep2.py`, shipped Python source read directly | **T1** |
| Permeability-range binning | Geolog module manifests | T1 |
| K-means (k = 25) followed by hierarchical agglomeration | IP module documentation | T2 |
| Ward segmentation with histogram-antimode boundary picking | SandiBumi's own | T1 (own source) |

The dossier recommends the **Lorenz-inflection** approach as the automatic default, on the stated
grounds that it is the only one of the three vendor approaches with a **readable reference
implementation on this machine** and the only one whose boundaries have a physical meaning (a
change in the flow-capacity/storage-capacity gradient) rather than a statistical one. That
recommendation is picked up as `SB-SHR-026` — derived from Gunter et al. 1997 rather than from the
vendor file, for the reason set out in §7.4.

**Lucia RFN out-of-range handling diverges.** One incumbent nulls `RFN > 20` outright (T1, read
from the shipped `RFUILibrary.py`); SandiBumi emits the number and nulls only the *class*. Both
are defensible; only one of them puts an uncalibrated number on a curve a client will plot.

### 2.5 Where one incumbent has a capability and the others do not

Recorded because these set the completeness bar, not because any is copied:

- **Closure / conformance correction** on a MICP curve — four distinct treatments exist across the
  corpus (shift, proportional normalisation, crop, extrapolate), and the choice moves the entry
  pressure materially. Only some incumbents expose all four.
- **Overburden / net-stress correction** and **clay-bound-water correction** (Hill-Shirley-Klein
  and Dual-Water forms) applied to the **non-wetting** phase. The non-wetting detail matters: the
  correction is a bulk-volume adjustment and applying it to the wetting phase inverts its sign.
- **Modality detection** on a pore-throat-size distribution — the difference between a rock with
  one throat population and a rock with two is the difference between one saturation-height law and
  two, and it is detectable from the MICP curve itself.
- **Permeability from MICP** by routes other than Swanson: Purcell, Ruth-Lindsay, Katz-Thompson,
  Oklahoma University, MODE. Their existence is why Swanson must not be the silent default.
- **Aguilera R35** as a distinct correlation from Winland R35, and **Permadi-Susilo PGS** as a
  distinct rock-typing indicator from FZI.
- **Dykstra-Parsons `VDP = (k₅₀ − k₁₅.₉)/k₅₀`** as a heterogeneity index alongside the Lorenz
  coefficient.

---

## 3. SandiBumi as-built

Every pointer below was re-opened at the source while writing this chapter. Where a line pointer
inherited from another document no longer resolved, or resolved to something different from what
that document claimed, that is stated rather than smoothed — §3.1 is entirely such a case.

**Files in this domain:** `satheight.rs` (356 lines, forward apply, registered module),
`shf_fit.rs` (1,380 lines, fitting engine, Tauri command), `thomeer.rs` (456 lines, MICP-domain
Thomeer + Swanson, Tauri command), `hfu.rs` (560 lines, FZI clustering, Tauri command),
`lorenz.rs` (654 lines, SMLP + flow units + Lorenz coefficient, Tauri command),
`rocktyping.rs` (905 lines, deterministic indicators, four registered modules),
`units.rs` (301 lines, the depth-unit carrier this domain's arithmetic depends on).
Front end: `shfDialog.ts`, `thomeerDialog.ts`, `hfuDialog.ts`, `lorenzDialog.ts`,
`crossplotPanel.ts`. Method banks: `docs/ref_shf.md`, `docs/ref_rock_typing.md`.

### 3.1 The P0 spine — `SB-CORE-001`, the depth-unit arithmetic

**Status: `PRESENT-DIVERGENT`.** The finding here differs materially from the description carried
in `04_CORE_REQUIREMENTS.md` and in the risk register, and the difference goes both ways: **one
half of the defect is closed and regression-tested; the other half is open, untested, and worth up
to 47.7 saturation units.**

**What is closed.** The Leverett-J branch of the forward model no longer multiplies by a
hard-coded 3.28084. `satheight.rs:189` reads:

```rust
let pc = PSI_PER_FT_PER_SG * (rho_w - rho_hc) * crate::units::to_feet(h, ctx.depth_unit);
```

The two fitting-side twins do the same (`shf_fit.rs:911`, `shf_fit.rs:1095`), and
`satheight.rs:246` carries a regression test —
`saturation_height_is_identical_whichever_unit_the_project_declares` — that describes one physical
well twice, as 100 m and as 328.084 ft above the same free-water level, and asserts the same
answer. That test is real and it passes the claim it makes. **Any document still describing the
Leverett Pc law as unconditionally metre-assuming is stale.** `units.rs:14-25` is one such
document: its module doc still narrates the broken state, including the literal string
`satheight.rs / shf_fit.rs compute pc = 0.433 * dRho * (h * FT_PER_M)`. So is `docs/ref_shf.md:9`,
which still writes `h_ft = H·3.28084`.

**What is open — the Skelt-Harrison branch has no unit conversion at all.** The height is formed
in the project's own depth unit at `satheight.rs:154`:

```rust
let h = fwl - dv; // metres above the FWL (negative below it); FWL shares dv's reference
```

The Leverett branch then converts it (`:189`). The Skelt branch does not — `satheight.rs:175`:

```rust
1.0 - a * (-(b / (h + dd)).powf(c)).exp()
```

`b` is `SH_B`, declared with unit **`"m"`** and default **30.0** at `satheight.rs:117`; `dd` is
`SH_D`, also declared `"m"` at `satheight.rs:119`. Both are compared directly against `h`, which
is in feet on a foot-declared project. The ratio `b/(h+dd)` is therefore wrong by 3.28084 on every
foot-declared project, raised to the power `c` (default 1.5, `satheight.rs:118`), inside an
exponential. On the shipped defaults, one physical column evaluated both ways:

| Height above FWL | Metre-declared `SWH` | Foot-declared `SWH` | Divergence |
|---|---|---|---|
| 30 m ≡ 98.43 ft | 0.6321 | 0.1549 | **47.7 saturation units** |
| 100 m ≡ 328.08 ft | 0.1515 | 0.0273 | **12.4 saturation units** |
| 300 m ≡ 984.25 ft | 0.0311 | 0.0053 | 2.6 saturation units |

The error is **largest in the transition zone**, which is the only part of the column a
saturation-height function exists to describe. The regression test at `satheight.rs:246` does not
catch it because it supplies `PHIE` and `PERM` and leaves `OPT_SWH` at its default `LEVERETT`
(`satheight.rs:109`) — it tests the branch that was fixed, not the branch that was not.

**What is open — the project depth unit defaults instead of refusing.** `SB-CORE-001` requires the
product to refuse to run when the depth unit is undeclared. `units.rs:179-180` does the opposite:

```rust
pub fn project_depth_unit_or_default(conn: &Connection) -> DepthUnit {
    project_depth_unit(conn).ok().flatten().unwrap_or(DepthUnit::Metres)
}
```

`impl Default for DepthUnit` (`units.rs:47`) is `Metres`, and `resolve_index_unit` returns
`IndexUnitAction::Assumed(DepthUnit::Metres)` when nothing is declared (`units.rs:225`). The
`Assumed` variant *is* surfaced to the user (`units.rs:210`), which is better than silence — but
the domain arithmetic still runs. The carrier is chapter 21's; the **refusal at the point of
height arithmetic is this chapter's**, and it is absent.

**What is open — height-dimensioned outputs are labelled with a hard-coded unit.**
`satheight.rs:125` declares the `HAFWL` output curve's unit as the literal `"m"`, and
`satheight.rs:110` declares the `FWL` parameter's unit as `"m"` with a default of 2000.0. On a
foot-declared project both are numerically correct and **textually wrong**: the curve carries feet
and says metres, and a user entering a free-water level is prompted in the wrong unit. This is the
same class as `SB-CORE-002` — a correct number presented under an incorrect label — and it is the
version of the depth-unit defect a client actually sees, because it appears on the plot legend and
in any LAS export of `HAFWL`.

*(Corrected 2026-08-20: the `FWL` prompt half is closed — see the as-built note under `SB-SHR-004`
in §4.1. The `HAFWL` half is not a labelling defect and is open for a ruling, also recorded there.)*

**Net.** The domain half of `SB-CORE-001` is not one defect but four, of which one is closed. §4
specifies the remaining three and §6 supplies the tests, including the branch-parity test that
would have caught the Skelt case at the time the Leverett case was fixed.

### 3.2 What is already right, and should be defended rather than rebuilt

Three things in this domain are better than the incumbents and are recorded as `PRESENT-OK` so
that later work does not regress them.

**The `shf_fit.rs` honesty contract is a working instance of `SB-CORE-002`.** Every fit result
carries `excluded: Vec<(String, usize)>` (`shf_fit.rs:514-515`, `:816-817`), populated at
`:616-629` and `:1132-1143` with a named reason and a count for every candidate sample dropped —
`Sw > 1` as non-physical, `Sw ≤ 0`, at or below the FWL, below the porosity cut-off, no
permeability for a Leverett fit. Scoped wells that contributed **zero** samples are named
individually (`:605-615`, `:1121-1131`) so that a law described as field-wide cannot silently have
been fitted on a subset. Per-rock-type groups that fail to converge are **returned with their
reason** rather than dropped (`fit_groups`, `shf_fit.rs:957-983`). A Buckles check
(`buckles_note`, `:989-1010`) flags when the top height-quartile bulk-volume water has
IQR/median > 0.6 — the diagnostic that says a single pooled law is inadequate. Nothing in the
incumbent corpus does this; it is a direct discharge of `03_EVIDENCE_BASE.md` §14.3 (fail loud
where they fail silent) and it should be the template for the modules path, which does not have it
(§3.7).

**`lorenz.rs` refuses a run rather than computing on the wrong curve.** The test at
`lorenz.rs:526`, `a_missing_curve_fails_by_name_rather_than_computing_on_another`, asserts both
that a missing permeability curve fails the run *and names the missing curve*, and — as a control —
that moving the absence to the porosity side moves the name in the message. That control is what
makes the test worth having: a guard that always blamed the same curve would pass a one-sided
test. This is the correct pattern for the whole domain.

**`rocktyping.rs`'s Pittman table is single-sited and paper-checked.** `PITTMAN_TABLE1`
(`rocktyping.rs:322-337`) carries **all fourteen** published rows in publication order, and the
nine-row shipped subset `PITTMAN_RX` (`:349-359`) deliberately carries **no coefficients** — it
looks them up. The module documents why: a hand-copied subset previously put Table 1's `r45`
coefficients under the `PR50` label and left `PR75` matching no published equation at all, which
produced a family that **inverted** (a wider throat at higher mercury saturation, which cannot
happen in rock) above ~79 mD at 25 % porosity. The fix was structural, not a re-typing, and it is
guarded by `every_shipped_pittman_row_matches_the_published_table`. Porosity is passed in
**percent** at `rocktyping.rs:456` (`let phi_pct = phi * 100.0;`) with the v/v input validated to
`(0,1)` at `:453` — which is the correct side of the 676× trap in §2.2.

### 3.3 Constants and definition sites — `SB-CORE-007` instances

Four instances, in descending order of what they cost.

**(a) `RHO_HC` has two different defaults, and the workflow crosses between them.**
`shf_fit.rs:729-731` defines the fitting default as **0.7** g/cc, sourced in the doc comment at
`shf_fit.rs:756-759` to the Techlog sand-summary default (T2, via `techlog_ingest/FINDINGS.md` §C).
`satheight.rs:112` — the forward-apply module the fit is meant to feed — defaults to **0.8** g/cc,
with **no source at all**.

This is not a cosmetic mismatch, because there is **no automated hand-off**: `docs/ref_shf.md:64`
records that the fitted-family export "lands with the 4b dialog export", and `shfDialog.ts:20`
confirms the dialog "writes no curves — it produces the law(s) for the forward `sw_height` apply".
The user therefore reads `A` and `B` off the fit and types them into the module — and the `ρhc`
does not travel with them. Fitting at Δρ = 0.3 and applying at Δρ = 0.2 makes the applied `Pc`
33.3 % low; through the shipped Leverett exponent `SWH_B = −0.4` (`satheight.rs:115`) that is
`Sw` **17.6 % high** — at a fitted `Sw` of 0.30, an applied `Sw` of 0.353, i.e. **5.3 saturation
units** of pure round-trip error. It computes, it plots and it ships.

**(b) `J_CONST` and `PSI_PER_FT_PER_SG` are vendor roundings, not derivations.**
`satheight.rs:14` carries `J_CONST = 0.21645`; the derived value is 0.2166011 (§2.1), so
SandiBumi is **0.0698 % low** — and, unlike the vendors' 0.2166, it is low in the third
significant figure. `satheight.rs:20` carries `PSI_PER_FT_PER_SG = 0.433`, which is **IP's
rounding**, 0.1217 % below the derived 0.4335278.

**These are stated honestly as small in magnitude.** Through `SWH_B = −0.4` the `J_CONST` error
moves `Sw` by 0.028 % and the gradient error by 0.049 % — neither is a field problem. They are
`P1`, not `P0`, and inflating them would be exactly the overclaim `CONTRACT.md` §5 warns about.
What makes them requirement-bearing is provenance: **neither constant carries a source string**,
which is a live `SB-CORE-004` failure in a domain whose entire competitive claim is that its
constants are derived rather than transcribed. A product that markets derivation while shipping a
competitor's rounding of the same constant has a demonstration problem, not an accuracy problem.

Both appear in **user-facing text as well as code**: `shfDialog.ts:124` prints the tooltip
`J = 0.21645·Pc/σcosθ·√(k/φ); Pc from height (0.433·Δρ·h_ft)`, so correcting the constants means
correcting three surfaces, not one. There is also a **test that hard-codes the wrong value**:
`ingest.rs:2313` synthesises its SCAL fixture with `let j = 0.21645 * pc / 72.0 * ...`, so the
round-trip assertion cannot detect a wrong `J_CONST` — the fixture and the code would move
together. That is a test-integrity finding in its own right.

**(c) The Amaefule constants are defined twice.** `hfu.rs:22` and `:24` define
`RQI_C = 0.0314` and `PERM_C = 1014.24`. `rocktyping.rs:98` and `:132` re-state the same two
numbers as **inline literals** in a different module:

```rust
let rqi_i = 0.0314 * (k / phi).sqrt();                                  // rocktyping.rs:98
let k = 1014.24 * fm * fm * phi.powi(3) / (1.0 - phi).powi(2);          // rocktyping.rs:132
```

Today the four values agree, so nothing is wrong *now*. Nothing asserts that they will continue
to. `hfu.rs:554-555` records that the two are deliberately **not** exact reciprocal-squares
(`1/0.0314² = 1014.24001`), kept as the literature values — which is the right call and is
precisely the kind of subtlety that a future "tidy-up" in one file and not the other would
destroy. Two definition sites for a physical constant is the `SB-CORE-007` pattern.

**(d) `HG_AIR_IFT` is a fused σ·|cosθ| product with no source.** `thomeer.rs:163` carries
`const HG_AIR_IFT: f64 = 367.0;` and `thomeer.rs:226` uses it to standardise every imported Pc
row to a mercury-air basis (`HG_AIR_IFT / v`, where `v` is the row's own stored `ift`). The
dossier's derived mercury constant, 106.6334/Pc, implies σ·|cosθ| = **367.606**, so 367.0 is
0.165 % low — again immaterial in magnitude. The **structural** problem is that σ and cosθ are
fused into one number, so a laboratory that used a contact angle other than the one baked in
cannot be re-scoped at all, and the standardisation at `:226` silently applies the wrong ratio.
`docs/ref_shf.md:53-55` already carries the unfused values for four systems (mercury 480/140°,
centrifuge and porous plate 72/0°, reservoir water-oil 30/30°, water-gas 50/0°, T2 sourced to
IP's `Cap_Pressure_Fluid_Prop_Defaults.par`), so the fix is a plumbing change, not a research
task.

### 3.4 Swanson ships a default the evidence says must not exist

**Status: `PRESENT-DIVERGENT`. This is the most serious as-built finding after §3.1.**

`thomeer.rs:263-264`:

```rust
let swanson_k =
    if all_ift && apex > 0.0 { 399.0 * (apex * 100.0).powf(1.691) } else { f64::NAN };
```

`apex` is `max(bv/pc)` over the sample's points (`thomeer.rs:257-261`), where
`bv = poro * (1.0 - sw)` (`:249`) — a **bulk-volume fraction**. The `* 100.0` at `:264` converts it
to bulk-volume **percent**. SandiBumi has therefore unilaterally selected one of the three
mutually-inconsistent apex bases identified in §2.3, and it has adopted **IP's uncited `399` and
`1.691` coefficient pair** to go with it. The dossier escalates the basis question as **ESC-7,
BLOCKING**, and its instruction is explicit: **ship no Swanson default**.

The stake is the full **10⁴** spread from §2.3 — 0.00864 mD versus 0.000193 mD versus 1.97 mD on
one sample. A permeability estimate wrong by four orders of magnitude does not fail a plausibility
check; it reads as a different facies, and it propagates into RQI, FZI, GHE class, the
permeability transform and every flow unit downstream.

The module's own header admits the problem — `thomeer.rs:13-14` states that the Swanson
permeability "ships flagged: verify the constants against the paper before field release". A
comment is not a flag. Nothing in the returned result, the dialog or any export carries that
caveat to the user, so the caveat exists only for whoever next reads the file. Under
`SB-CORE-002`, a result whose own author has recorded that it is unverified and which is presented
without that caveat **is** a degraded result presented as clean.

### 3.5 The fitted objects do not exist as objects — `SB-CORE-014`

**Status: `ABSENT`.** Every fitting path in this domain is a **Tauri command that returns JSON to
a dialog and nothing else**. `shf_fit::run_shf_fit`, `shf_fit::run_cuddy_foil`,
`thomeer::run_thomeer_fit`, `hfu::run_hfu_cluster` and `lorenz::run_lorenz` are registered at
`lib.rs:3276-3281`; none of them writes a curve, a table or a row. A search of `db.rs` finds
`scal_pc` and `scal_sets` (the *input* Pc measurements, `db.rs:696-719`) and **no table for a
fitted law of any kind**.

The consequences are concrete:

- A Cuddy FOIL law, a Brooks-Corey (Swirr, He, λ), a Skelt (A, B, C, D), a Thomeer (Swirr, Hd, G),
  a Leverett (A, B), an HFU cluster assignment and a Lorenz flow-unit partition **all evaporate
  when the dialog closes**. The only way to keep one is a screenshot or a hand transcription.
- Nothing records **what the law was fitted on** — which wells, which log set, which curve
  versions, how many samples survived the exclusions, what `ρw`/`ρhc`/`σcosθ` were in force, what
  FWL. `SB-CORE-014`'s scope note says every chapter that fits anything inherits the gap; this
  domain inherits it six times over.
- The `ρhc` round-trip error in §3.3(a) is a **direct consequence** of this absence. If the fit
  were an object, applying it would carry its own fluid properties and the 5.3-saturation-unit
  error could not arise.
- The `excluded`/`notes` honesty contract of §3.2 is computed, shown once, and then discarded —
  the highest-value provenance in the domain has the shortest lifetime in the domain.

**The FWL scan reports a point, not an interval.** `foil_fwl_scan` (`shf_fit.rs:107-125`)
implements Cuddy's Eq 19 — step a candidate FWL through a range, refit, keep the minimum
mean-squared log residual — and returns the scan curve plus the argmin. It returns **no
uncertainty interval**. Dossier escalation **ESC-12** requires one, and it matters commercially:
the FWL is usually the single most contested number in a volumetric review, and a bare argmin
invites a false precision the residual curve does not support.

### 3.6 Validity bands enforced on one side only — `SB-CORE-003`

**Lucia rock-fabric number.** `lucia_rfn` (`rocktyping.rs:166-177`) is correct on the porosity
unit — it takes `phi_ip` in v/v and validates it to `(0,1)`, which is the right side of the
`RFN 2.07 vs 7.41` trap in §2.2, and resolves the dossier's E-OPEN-3 in SandiBumi's favour. The
**class** function `lucia_class` (`:179-191`) then returns `NaN` above 4.0 as outside the
calibrated band — but returns **class 1.0 for any RFN below 1.5**, including values far below
Lucia's calibrated floor of 0.5. The band is enforced at the top and open at the bottom. A
super-permeable plug returning RFN 0.02 is reported as Class 1 rock with no flag.

Separately, the **RFN curve itself is emitted unclamped**. One incumbent nulls `RFN > 20` outright
(§2.4). SandiBumi nulls only the class, so a client plot of `RFN` carries values the transform's
calibration does not support, beside a class curve that has correctly gone blank — two curves
telling different stories about the same sample.

**Port-size classification adopts one scheme silently.** `rocktyping.rs:32` fixes
`PORT_BOUNDS: [f64; 4] = [0.1, 0.5, 2.5, 10.0]` — the Hartmann-Beaumont scheme, with the
Macro/Meso boundary at 2.5 µm. Another incumbent uses 2.0 µm at the same boundary. Both are named,
published schemes; the choice reclassifies every plug whose apex radius falls between 2.0 and
2.5 µm, and nothing in the output says which scheme produced the class. Dossier escalation
**ESC-6** requires both to be selectable with **no silent default**.

**Pittman's own extrapolation caveat is documented but not enforced.** `rocktyping.rs:400-410`
states plainly that the family stops falling monotonically below about 11 % porosity — at 5 %
porosity and 1 mD, `PR40 = 0.77 µm` but `PR50 = 0.86` and `PR75 = 1.11` — and that nothing is
clamped, "because forcing the ordering would report radii the paper never published". **That
reasoning is correct and must not be changed.** What is missing is the other half: the samples in
that regime are not *flagged*. The user is told in a module description; the affected rows carry
no marker.

### 3.7 The modules path has no honesty contract

The five registered modules — `sw_height` (`modules.rs:383`), `rocktyping`, `lucia_rfn`,
`pittman_rx`, `rt_cutoff` (`modules.rs:386-388`) — all follow the same convention: a sample that
fails a precondition is **left MISSING and not counted** (`rocktyping.rs:453-455`,
`satheight.rs:148-150`, `:162-165`, `:179-181`). No exclusion ledger, no reason, no count. A
`rocktyping` run over an interval where 80 % of the samples had `φ ≥ 1` because the porosity curve
arrived in percent produces a `RQI` curve with 20 % coverage and says nothing at all.

This is the same defect `shf_fit.rs` already solved (§3.2) inside the same product. The contrast
is the finding: the domain contains both the best and the worst instance of `SB-CORE-002` in the
codebase, twenty files apart.

`rt_cutoff` (`rocktyping.rs:259-279`) is a further case: it defaults `VSH1 = 0.15`, `PHI1 = 0.12`,
`VSH2 = 0.35`, `PHI2 = 0.06` (`rocktyping.rs:235-257`) with **no source of any kind**, and returns
class 3 for anything that fails both tests — so "did not meet the criteria" and "is genuinely poor
rock" are the same output value.

### 3.8 Where the results go — persistence, plotting, reporting

| Surface | Coverage |
|---|---|
| **Curves (persisted)** | `SWH`, `HAFWL`, `RQI`, `PHIZ`, `FZI`, `GHE`, `R35`, `RTPORT`, `PERM_RT`, `RFN`, `RFN_CLASS`, `PR10`–`PR75`, `RAPEX`, `RT_PITT`, `RT_LOG` — via the module/computed-curve path only |
| **Fitted laws** | **Nothing persisted** (§3.5) |
| **Plots** | `shfDialog.ts`, `thomeerDialog.ts`, `hfuDialog.ts`, `lorenzDialog.ts`, `crossplotPanel.ts` — all dialog-local canvases |
| **PDF / Word report** | **No coverage.** `report.rs` contains no reference to any mnemonic, method or parameter in this domain |
| **Export** | `export.rs` contains no reference to any of them either; the curves leave only as generic curves |

The report gap has a specific edge. `report::default_methodology` (`report.rs:61-91`) emits a
fixed methodology table for every study, and its **Water saturation** row reads
`"Indonesia / Simandoux / Archie (RtC or IMTS in LRLC zones)"`. A study whose pay was computed from
`SWH` — a saturation-height function — ships a methodology table naming **a method it did not
use**, with no row for the saturation-height model, no row for the rock-typing scheme, and no
statement of the FWL. There is a `Cutoffs` row that quotes the actual cut-off values from the
spec, so the shape for doing this correctly already exists; the saturation-height and rock-typing
rows simply were never added. Under `SB-CORE-002` this is the most externally visible instance in
the chapter, because the methodology table is the page a client reviewer reads first.

---

## 4. Requirements

RFC-2119 verbs per `CONTRACT.md` §1.4. Each requirement carries its priority, its as-built status,
the `SB-CORE` allocation it discharges where there is one, and the test that verifies it.

### 4.1 Depth-unit arithmetic — the domain half of `SB-CORE-001`

**`SB-SHR-001`** · P0 · `PRESENT-DIVERGENT` · discharges `SB-CORE-001`
> **Every** height-domain saturation model MUST convert the height above the free-water level into
> the unit in which that model's own coefficients are defined, and the conversion MUST be driven by
> the project's declared depth unit. No branch of any saturation-height model may consume a raw
> height. Adding a new model family MUST NOT be possible without declaring the unit its
> length-dimensioned coefficients are in.

The Leverett branch complies (`satheight.rs:189`); the Skelt-Harrison branch does not
(`satheight.rs:175`), at a cost of up to 47.7 saturation units (§3.1). The second sentence is the
part that keeps this closed: the defect arose because a fix was applied per branch rather than at
the point where a height enters a model.
*Verified by* `SB-SHR-T01`, `SB-SHR-T02`.

**`SB-SHR-002`** · P0 · `ABSENT` · discharges `SB-CORE-001`
> Every shape parameter carrying a length dimension — Skelt-Harrison `B` and `D`, Thomeer entry
> height `Hd`, Brooks-Corey entry height `He`, and the free-water level itself — MUST carry an
> explicit unit in its registration, and the product MUST re-express its value when the project
> depth unit changes. A length-dimensioned parameter with a hard-coded unit string is a defect.

`satheight.rs:110`, `:117` and `:119` all hard-code `"m"`.
*Verified by* `SB-SHR-T03`.

**`SB-SHR-003`** · P0 · `ABSENT` · discharges `SB-CORE-001`
> The domain MUST refuse to perform height arithmetic when the project depth unit is undeclared,
> and MUST say so by name. It MUST NOT substitute a default unit. This requirement binds the
> domain's own entry points; the carrier's parse-time behaviour is `21_data-io.md`'s.

`units.rs:179-180` currently returns `Metres` for an undeclared project.
*Verified by* `SB-SHR-T04`.

**`SB-SHR-004`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-001`, `SB-CORE-002`
> Every height-dimensioned output curve, parameter prompt, plot axis and export header in this
> domain MUST be labelled in the project's declared depth unit. A numerically correct value under
> an incorrect unit label is a reportable defect.

`satheight.rs:125` labels `HAFWL` as `"m"` unconditionally; `satheight.rs:110` prompts for `FWL`
in `"m"` unconditionally.
*Verified by* `SB-SHR-T05`.

**As-built correction, 2026-08-20 — the prompt half is closed; the output-curve half is not, and
is not a labelling question.** The `FWL` parameter and the `TVD` input channel now declare
`modules::PROJECT_DEPTH_UNIT_TOKEN`, and every dialog that prints a module argument's unit
resolves that token to the project's **stored** unit (`depthUnitPref::argumentUnitLabel`). That
was an unambiguous defect: `sw_height` computes `h = FWL - dv` against the raw depth sample and
converts only afterwards, so the number entered was always project-native and the label
contradicted the arithmetic. No numbers moved. Pinned by
`the_free_water_level_is_declared_in_the_unit_the_height_is_actually_measured_in` and
`a_project_native_parameter_is_labelled_in_the_stored_unit_and_never_follows_the_view_preference`.

`HAFWL` is deliberately untouched and this requirement as written does not fit it. The curve is
not a mislabelled project-native height — `sw_height` **converts** the height to metres before
writing it, so `"m"` is a true statement about the values stored. Making it follow the project
would change the numbers written on every foot-declared project, which is a product decision
about what a height-above-contact curve should be delivered in, not a label fix. **Open for
Jauhar's ruling**, stated as the choice it is: `HAFWL` always in metres (as built, and consistent
with the Skelt-Harrison constants, which are metres by the published form), or `HAFWL` in the
project's unit (this requirement as written, and consistent with how every other depth-dimensioned
curve is stored). Until then nothing changes.

### 4.2 Constants: one derivation, one definition site, one source

**`SB-SHR-005`** · P0 · `PRESENT-DIVERGENT` · discharges `SB-CORE-007`
> Water density, hydrocarbon density and reservoir `σ·cosθ` MUST each have **exactly one** default
> in the product, shared by the fitting path and the forward-apply path. Where a fit and an apply
> can disagree on a fluid property, the product MUST refuse the apply rather than compute it.

`shf_fit.rs:729-731` defaults `ρhc` to 0.7; `satheight.rs:112` defaults it to 0.8; the round trip
costs 5.3 saturation units (§3.3a). This is P0 because the error is silent, quantified and
reachable through the intended workflow.
*Verified by* `SB-SHR-T06`.

**`SB-SHR-006`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-004`, `SB-CORE-007`
> The Leverett J constant and the hydrostatic gradient per unit specific gravity MUST each be
> **derived from first principles in the product**, defined once, and carry a machine-readable
> source. Neither may be a transcription of a vendor's printed rounding. The derivation MUST be
> expressed as an evaluable expression, not a literal.

`satheight.rs:14` (`0.21645`) and `satheight.rs:20` (`0.433`, IP's rounding) are literals with no
source. The magnitude is small — 0.028 % and 0.049 % in `Sw` — and the requirement is P1 for that
reason; it is not P2 because `SB-CORE-004` makes a sourceless registered default a build failure.
*Verified by* `SB-SHR-T07`.

**`SB-SHR-007`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-007`
> A physical constant used by more than one module MUST have exactly one definition site.
> `RQI_C` and `PERM_C` MUST be imported by every consumer. The product MUST NOT contain a second
> literal of a constant it already defines — in code, in user-facing text, or in a test fixture.

`hfu.rs:22`/`:24` versus the inline literals at `rocktyping.rs:98`/`:132`; the tooltip at
`shfDialog.ts:124`; the fixture at `ingest.rs:2313` that hard-codes `0.21645` and so cannot detect
a wrong `J_CONST` (§3.3b, §3.3c). The test-fixture clause is the one that matters most: a
duplicated constant inside the test that guards it makes the guard inert.
*Verified by* `SB-SHR-T08`.

**`SB-SHR-008`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-007`
> Interfacial tension and contact angle MUST be carried as **separate** quantities. The product
> MUST NOT store or ship a fused `σ·cosθ` product as a single constant, and every laboratory and
> reservoir system MUST declare both components.

`thomeer.rs:163` fuses them into `HG_AIR_IFT = 367.0` and `thomeer.rs:226` standardises against
it; the unfused values are already banked in `docs/ref_shf.md:53-55`.
*Verified by* `SB-SHR-T09`.

### 4.3 Fitted objects and their provenance — `SB-CORE-014` in this domain

**`SB-SHR-009`** · P0 · `ABSENT` · discharges `SB-CORE-014`
> Every fitted object this domain produces — saturation-height law (pooled and per rock type),
> Thomeer fit, HFU cluster model, Lorenz flow-unit partition — MUST be persisted as a first-class,
> named, versioned object. Each MUST carry its training provenance: the wells, the log set and
> curve versions, the sample count, the full exclusion ledger, the fluid properties and FWL in
> force, the fitting method, and the fit-quality statistic. An object that cannot state what it was
> fitted on MUST NOT be applicable.

No such store exists; `db.rs` holds the input `scal_pc` measurements and nothing else (§3.5).
*Verified by* `SB-SHR-T10`, `SB-SHR-T11`.

**`SB-SHR-010`** · P0 · `ABSENT` · discharges `SB-CORE-014`, `SB-CORE-007`
> The forward-apply path MUST consume a **stored fitted object**, not hand-entered coefficients.
> Where a user overrides a stored coefficient, the applied result MUST record the override and the
> value it replaced. Hand transcription of a fit into a module parameter MUST NOT be the supported
> workflow.

This is the structural fix for `SB-SHR-005`: a law that travels as an object cannot be applied
under different fluid properties than it was fitted under.
*Verified by* `SB-SHR-T12`.

**`SB-SHR-011`** · **P0** · `ABSENT`
> The free-water level MUST be a **first-class uncertain parameter, not a scalar input.** The FWL
> scan MUST be mandatory output, MUST report an uncertainty interval alongside its optimum — the
> range of candidate levels whose residual is statistically indistinguishable from the minimum —
> and MUST NOT present the argmin alone. Every saturation-height result MUST carry a **per-zone
> FWL confidence** alongside its fit statistic, and a fit whose FWL cannot be constrained MUST say
> so rather than report a coefficient set.

`foil_fwl_scan` (`shf_fit.rs:107-125`) returns the curve and the argmin only. This is **P0, not
P1**, and the reason is commercial rather than numerical. Dossier ESC-12 records a delivered
deltaic-clastic study in which the saturation-height function **was built and then not adopted** —
the asset team chose a `Swe`-`Phie` function instead — **because the free water level could not be
reliably picked per layer** in a multi-layer, proximal-to-distal deltaic sequence. The binding
constraint on an SHF in exactly this operating environment is FWL pickability, not curve fitting.
A tool that fits excellent per-rock-type laws and cannot state its FWL uncertainty reproduces the
failure that got that deliverable rejected. Neither incumbent reports an FWL uncertainty at all.
*Verified by* `SB-SHR-T13`.

### 4.4 Convention divergence surfaced at the point of choice — `SB-CORE-006`, `SB-CORE-013`

**`SB-SHR-012`** · P0 · `PARTIAL` · discharges `SB-CORE-006`
> A Brooks-Corey fit MUST declare its exponent convention explicitly and MUST emit **both** `λ` and
> `N = 1/λ`, each labelled. Import or export of a Brooks-Corey coefficient without a declared
> convention MUST be refused.

`fit_brooks_corey` (`shf_fit.rs:146-203`) correctly stores `lambda`, which is the published
convention — the gap is that nothing emits or demands the `N` form, so a coefficient exchanged with
an `N`-convention tool moves the curve by 26 saturation units (§2.2).
*Verified by* `SB-SHR-T14`.

**`SB-SHR-013`** · P0 · `PARTIAL` · discharges `SB-CORE-006`
> A Thomeer fit MUST declare the logarithm base of its shape factor `G` and MUST emit both the
> base-10 (`G`) and natural-log (`2.302585·G`) forms, each labelled. A `G` imported without a
> declared base MUST be refused.

`thomeer.rs:33` and `thomeer_sw` (`shf_fit.rs:361-367`) both use `log10`, which is Thomeer's
published base — again correct, again undeclared, again a 12-saturation-unit exchange hazard
(§2.2).
*Verified by* `SB-SHR-T15`.

**`SB-SHR-014`** · P0 · `PRESENT-DIVERGENT` · discharges `SB-CORE-002`, `SB-CORE-013`
> The product MUST NOT ship a default apex basis for Swanson permeability. The bulk-volume basis
> (fraction, percent, or pore-volume-normalised saturation) MUST be an explicit, named user choice
> with no default, the chosen basis MUST travel with every Swanson result, and the coefficient pair
> MUST carry its own source. Until a basis is chosen, Swanson permeability MUST be `MISSING`, not
> computed.

`thomeer.rs:263-264` ships bulk-volume-percent with IP's uncited `399`/`1.691`; the basis question
is dossier ESC-7, **BLOCKING**, with a stake of 10⁴ in permeability (§3.4).
*Verified by* `SB-SHR-T16`, `SB-SHR-T17`.

**`SB-SHR-015`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-013`
> Pore-throat port-size classification MUST be a named, selectable scheme. Both published boundary
> sets MUST be available, the active scheme MUST be recorded on every classified result, and there
> MUST be no silent default.

`rocktyping.rs:32` fixes one scheme; dossier ESC-6.
*Verified by* `SB-SHR-T18`.

**`SB-SHR-016`** · P1 · `ABSENT` · discharges `SB-CORE-006`
> The mnemonic `RQI` MUST be namespace-disambiguated between Amaefule's Reservoir Quality Index
> and the identically-named quantity in the shaly-sand saturation family. The product MUST refuse
> to consume one where the other is expected, and MUST NOT resolve the collision by curve-name
> matching alone.

Stake: 11.8× in `Swirr` (§1.2). Coordinated with `12_saturation.md`, which owns the other name.
*Verified by* `SB-SHR-T19`.

**`SB-SHR-017`** · P0 · `PARTIAL` · discharges `SB-CORE-003`
> Every correlation in this domain that regresses on `log φ` MUST enforce its porosity **unit** as
> a precondition, not document it. A porosity input outside the declared unit's valid range MUST
> fail the run and name the curve; it MUST NOT be silently skipped.

The stake is `100^exponent`: 676× for Pittman R25, 1.32×10⁶ for the Oklahoma-University
correlation (§2.2). `rocktyping.rs:453` and `:456` already validate to `(0,1)` and convert —
correct, and the pattern to generalise. The `PARTIAL` is that failures are skipped rather than
raised (§3.7).
*Verified by* `SB-SHR-T20`.

**`SB-SHR-018`** · P1 · `ABSENT` · discharges `SB-CORE-013`
> Where the incumbent corpus ships mutually inconsistent defaults for a quantity this domain
> consumes, the product MUST surface the divergence **at the point of choice**, quantified in the
> units the user is working in. The hydrocarbon gradient MUST show the height consequence of each
> candidate rather than the gradient value alone.

Three vendor hydrocarbon gradients place the same `Pc = 10 psi` contact at 27.9, 54.6 and 77.0 ft
above the FWL — a 2.76× spread (§2.1). A gradient list is not a choice; a height list is.
*Verified by* `SB-SHR-T21`.

**`SB-SHR-019`** · P1 · `ABSENT`
> A coefficient whose value is scoped to a particular interfacial tension by the implementation
> that produced it MUST be recorded `NON-ADOPTABLE` and MUST NOT be used as a SandiBumi default in
> any form, including as a seed. Where such a coefficient is displayed for verification, its σ
> scope MUST be displayed with it.

§2.2, the `pc /= ift` finding: every downstream coefficient in that implementation is scoped ×72
or ×480.
*Verified by* `SB-SHR-T22`.

### 4.5 Validity conditions enforced — `SB-CORE-003`

**`SB-SHR-020`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-003`
> The Lucia rock-fabric-number validity band MUST be enforced on **both** sides. An `RFN` below the
> calibrated floor MUST NOT be assigned the lowest class. The `RFN` curve itself MUST be flagged or
> null outside the calibrated range, so that the value curve and the class curve cannot tell
> different stories about the same sample.

`lucia_class` (`rocktyping.rs:179-191`) returns `NaN` above 4.0 but class 1.0 below 1.5 without
limit; the `RFN` value is emitted unclamped (§3.6).
*Verified by* `SB-SHR-T23`.

**`SB-SHR-021`** · P1 · `PARTIAL` · discharges `SB-CORE-003`
> Samples falling in a published correlation's documented extrapolation regime MUST be flagged
> **per sample**, on the output, not only in the module description. The product MUST NOT clamp
> such values into a monotone ordering the source paper does not publish.

`rocktyping.rs:400-410` documents the sub-11 %-porosity inversion and correctly declines to clamp;
the missing half is the per-sample marker (§3.6).
*Verified by* `SB-SHR-T24`.

### 4.6 Honesty contract extended to the whole domain — `SB-CORE-002`

**`SB-SHR-022`** · P0 · `PRESENT-DIVERGENT` · discharges `SB-CORE-002`
> Every module in this domain MUST carry the exclusion ledger the fitting path already implements:
> a named reason and a count for every sample not computed, returned with the result and persisted
> with any curve it wrote. A curve with materially reduced coverage MUST state why.

`shf_fit.rs:514-515`, `:616-629` is the working implementation; `rocktyping.rs:453-455`,
`satheight.rs:148-150` and the other module paths silently leave samples `MISSING` (§3.2, §3.7).
*Verified by* `SB-SHR-T25`.

**`SB-SHR-023`** · P1 · `PRESENT-DIVERGENT` · discharges `SB-CORE-002`, `SB-CORE-004`
> A classifier MUST distinguish "did not meet any class criterion" from "meets the lowest class".
> Its class boundaries MUST each carry a source or be recorded `ABSENT` and require user entry.

`rt_cutoff` (`rocktyping.rs:235-279`) ships four sourceless boundaries and collapses both meanings
into class 3 (§3.7).
*Verified by* `SB-SHR-T26`.

**`SB-SHR-024`** · P1 · `ABSENT` · discharges `SB-CORE-002`
> A constant the product's own source records as unverified MUST surface that state as a
> **result-level flag** carried to every consumer — dialog, plot, export and report — not as a code
> comment. A result computed from an unverified constant MUST NOT be presentable as clean.

`thomeer.rs:13-14` records the Swanson constants as unverified; nothing carries that to the user
(§3.4).
*Verified by* `SB-SHR-T17`.

**`SB-SHR-025`** · P1 · `ABSENT` · discharges `SB-CORE-002`
> The deliverable report MUST name the saturation-height model and the rock-typing scheme actually
> used, with their parameters and their free-water level, and MUST NOT present a fixed methodology
> row naming a method the study did not use.

`report::default_methodology` (`report.rs:61-91`) has no row for either and its water-saturation
row names electrical models only (§3.8).
*Verified by* `SB-SHR-T27`.

### 4.7 Capability completeness

**`SB-SHR-026`** · P1 · `ABSENT`
> Laboratory capillary pressure MUST be convertible to reservoir conditions by the published
> relation `Pc_res = Pc_lab · (σ·|cosθ|)_res / (σ·|cosθ|)_lab`, with both systems named and both
> components of each product declared separately. Conversion MUST be refused when either system is
> undeclared.

Depends on `SB-SHR-008`. Values for four systems are already banked at `docs/ref_shf.md:53-55`.
*Verified by* `SB-SHR-T28`.

**`SB-SHR-027`** · P1 · `ABSENT`
> Closure / conformance correction MUST be available with all four published treatments — shift,
> proportional normalisation, crop, extrapolate — as a named user choice with no silent default,
> and the chosen treatment MUST travel with the corrected curve and with any entry pressure or
> Thomeer parameter derived from it.
*Verified by* `SB-SHR-T29`.

**`SB-SHR-028`** · P2 · `ABSENT`
> Net-overburden-stress correction and clay-bound-water correction MUST be available, MUST be
> applied to the **non-wetting** phase, and MUST record which correction was applied. A correction
> applied to the wetting phase inverts its sign and MUST be rejected by construction, not by
> documentation.
*Verified by* `SB-SHR-T30`.

**`SB-SHR-029`** · P2 · `ABSENT`
> Pore-throat-size distributions MUST be tested for **modality**, and a multimodal result MUST be
> reported as an explicit finding — it is the direct evidence that one saturation-height law is
> insufficient for the sample set.

Complements the existing Buckles diagnostic at `shf_fit.rs:989-1010`, which detects the same
condition from logs; the two MUST be reported together where both are available.
*Verified by* `SB-SHR-T31`.

**`SB-SHR-030`** · P1 · `ABSENT`
> Automatic flow-unit partitioning MUST be available by **inflection of the Lorenz curve** —
> partitioning on the change in the flow-capacity/storage-capacity gradient rather than on a
> statistical criterion — as a selectable method alongside the existing exact Ward segmentation.
> The active method MUST be recorded on the partition.

Independently derived; see §7.4. SandiBumi's existing `lorenz.rs` already computes the cumulative
curve (`lorenz.rs:293-309`) the method needs, so the increment is the boundary rule, not the plot.
*Verified by* `SB-SHR-T32`.

**`SB-SHR-031`** · P2 · `ABSENT`
> The Dykstra-Parsons coefficient `VDP = (k₅₀ − k₁₅.₉)/k₅₀` MUST be available alongside the Lorenz
> coefficient, and the two MUST be reported together — they disagree in informative ways on
> layered systems.
*Verified by* `SB-SHR-T33`.

**`SB-SHR-032`** · P2 · `ABSENT`
> Permeability from capillary pressure MUST offer routes other than Swanson — at minimum Purcell
> and Katz-Thompson — each named, each with its own source, and their answers MUST be presentable
> side by side rather than one being chosen silently.

This is the structural companion to `SB-SHR-014`: the reason no Swanson default may ship is that
alternatives exist and disagree.
*Verified by* `SB-SHR-T34`.

**`SB-SHR-033`** · P2 · `ABSENT`
> Aguilera R35 and the Permadi-Susilo pore-geometry-and-structure indicator MUST be available as
> **separately named** indicators, never merged into or substituted for Winland R35.

`rocktyping.rs:101` implements Winland R35 only; the name `R35` alone is ambiguous across the
corpus.
*Verified by* `SB-SHR-T35`.

### 4.8 The seams handed across

**`SB-SHR-034`** · P1 · `ABSENT`
> This domain MUST expose the fluid-gradient conversion as a service that
> `14_cutoffs-summation-mc.md` calls to turn an `HCPV` thickness into a volume. The service MUST
> use the single derived gradient of `SB-SHR-006`, MUST take the fluid densities from the stored
> fitted object where one is in force (`SB-SHR-009`), and MUST refuse when the depth unit is
> undeclared (`SB-SHR-003`).

Chapter 14 explicitly refuses this conversion inside the summation module on the grounds that it
belongs here. Picked up.
*Verified by* `SB-SHR-T36`.

**`SB-SHR-035`** · P1 · `PARTIAL`
> This domain MUST derive candidate reservoir and pay cut-offs **from** its own evidence — the rock
> type partition, the flow-unit boundaries, and the capillary-pressure curve — and MUST hand them to
> the cut-off machinery as sourced values carrying the evidence they were derived from. It MUST NOT
> select a cut-off; selection is `14_cutoffs-summation-mc.md`'s, and the two named-paper closures
> for selection are not on this machine (see §7.5).

The `PARTIAL` is `rt_cutoff` (`rocktyping.rs:259-279`), which applies cut-offs but derives none.
*Verified by* `SB-SHR-T37`.

**`SB-SHR-036`** · P2 · `ABSENT`
> The Lambda saturation-height family (`Sw = a·Pc^(−λ) + b`) MUST be available as a sixth family
> once its parameter sources are established. Until then it is recorded `ABSENT` rather than
> approximated from an adjacent family.

`docs/ref_shf.md:58-59` records the relevant vendor parameter store as identified but not mined,
and explicitly says to revisit it before adding this family. That instruction is honoured here.
*Verified by* `SB-SHR-T38`.

### 4.9 Reference conditions, import safety, and model selection

**`SB-SHR-037`** · P1 · `ABSENT` · discharges `SB-CORE-006`, `SB-CORE-007`
> Any saturation quoted at a capillary pressure — `Swirr` above all — MUST carry its **reference
> condition as part of the parameter type**, never as free text: the pressure *and* whether that
> pressure is laboratory or reservoir. A `Swirr` at an undeclared reference MUST be refused.

Dossier ESC-9 records one incumbent module family shipping `PC_SWIR = 200 psi` described as
*laboratory* in two manifests and `PC_SWIRR = 100 psi` labelled *Reservoir* in a third, with the
reference condition carried only in a comment. At mercury-lab-to-reservoir scaling of ≈0.0707, a
200 psi laboratory `Pc` is ≈14 psi at reservoir — so the two defaults pick `Swirr` at points
roughly an **order of magnitude apart on the same curve**, inside one product. The numeric default
is `ABSENT` in §5; what this requirement fixes is that the ambiguity cannot recur.
*Verified by* `SB-SHR-T39`.

**`SB-SHR-038`** · P1 · `ABSENT` · discharges `SB-CORE-002`
> The SCAL importer MUST refuse a corrected capillary-pressure curve that carries no
> correction-provenance tag naming the correction applied and the implementation that applied it.
> Module identity on import MUST be keyed on the manifest's declared name, never on a free-text
> specification line.

Dossier ESC-10 catalogues four shipped-contract defects in one incumbent's printed correction
formulae, of which the overburden-porosity form reads **×12.9** literally against **×1.155** as
intended at φe = 0.15 / φt = 0.20. Whether the compiled implementation matches its own printed
manifest **cannot be determined from the tree** — so an imported corrected `Pc` curve may be wrong
by up to an order of magnitude with nothing in the file to say so. The same escalation records a
manifest whose specification line was copy-pasted from an unrelated module, which is why identity
must key on the declared name.
*Verified by* `SB-SHR-T40`.

**`SB-SHR-039`** · P1 · `ABSENT` · discharges `SB-CORE-010`, `SB-CORE-014`
> The product MUST provide a **deterministic model-selection sweep** across this domain's competing
> choices — saturation-height family, rock-typing scheme, apex saturation, partitioner, port-size
> scheme — evaluated against core or capillary-pressure control data. The sweep MUST be
> **exhaustive over the declared grid rather than randomly sampled**, MUST be **uncapped in depth
> samples**, MUST be reproducible from its recorded inputs, and MUST record the full ranking rather
> than only the winner.

Independently derived; see §7.4 for its sources and its `Betters:` line. The need is the central
unsolved user problem of this chapter — the dossier's own optimal-choice table runs to roughly 35
rows of "which one" — and it is currently answered by hand.
*Verified by* `SB-SHR-T41`.

### 4.10 Reproducibility and regression direction

**`SB-SHR-040`** · P0 · `ABSENT` · discharges `SB-CORE-002`
> **No numerical result in this domain may depend on a display setting.** A closure pick, an entry
> pressure, a fitted coefficient or a rock class MUST be identical whether an axis is drawn linear
> or logarithmic, whether a plot is open, and whatever the current zoom or theme. Where a method
> genuinely requires a log-domain pick, the log domain MUST be a property of the **method**, not of
> the plot.

Dossier finding 3.14 records an incumbent whose closure auto-pick and entry pressure differ
depending on whether an axis checkbox is set logarithmic. A number that changes when a checkbox
changes cannot be reproduced from a saved project, cannot be audited, and cannot be defended in a
review.
*Verified by* `SB-SHR-T42`.

**`SB-SHR-041`** · P1 · `PARTIAL`
> A published regression MUST NOT be algebraically inverted to predict in the direction it was not
> fitted. Where a source publishes both a forward and an inverse fit, the one matching the
> prediction direction MUST be used, and the direction MUST be recorded on the result.

Dossier finding 3.7: inverting Pittman's `R = f(k, φ)` fits to predict `k` diverges from
Pittman's own published `k = f(φ, R)` fits by **2.0× at φ = 5 %**, 1.13× at 15 % and 1.9× at 30 % —
in **opposite directions at the two ends**, which is the signature of an inverted least-squares fit
rather than a transcription error. SandiBumi's `pittman_radius` (`rocktyping.rs:376-379`)
implements the forward direction and is correct for predicting `R`; the `PARTIAL` is that nothing
prevents a future `k`-from-`R` path being built by inversion.
*Verified by* `SB-SHR-T43`.

**`SB-SHR-042`** · P1 · `ABSENT` · discharges `SB-CORE-007`
> Contact angle MUST be stored with a single declared convention and a single validation range
> across the whole product, and every capillary expression MUST take `|cos θ|` consistently. A
> stored angle and its cosine MUST NOT be independently editable.

Dossier finding 3.10: the corpus stores the same laboratory system as ≈40° (cos = +0.765) in one
tool and 140° (cos = −0.766) in two others; one of those takes the absolute value in its
pore-throat equation but not in its `Pc` conversion, and validates θ to `0:360` in two modules and
`0:180` in four others — a **sign flip on every mercury `Pc`** depending on which module ran.
Depends on `SB-SHR-008` (σ and cosθ unfused).
*Verified by* `SB-SHR-T44`.

**Architectural finding recorded as `PRESENT-OK`.** Dossier finding 3.18 establishes that rock
typing is a **partitioning** concern in all three incumbents — there is no rock-type term inside
any saturation-height function — and recommends SandiBumi adopt the separation. It already has:
`fit_groups` (`shf_fit.rs:957-983`) fits one independent law per rock-type class and no family's
equation carries an RT term. No requirement is needed; it is recorded so a later refactor does not
"simplify" the separation away.

---

## 5. Parameters

Per `CONTRACT.md` §2. Values are transcribed byte-exact and no unit conversion is performed in the
table. `ABSENT — ships with no default` means no defensible adjudication exists on this machine.
`NON-ADOPTABLE — cited for verification` means a value exists and must not be adopted.
`UNSOURCED` marks an as-built default with no source string — each is a live `SB-CORE-004` exposure
and is listed so the build gate can find it.

**Two disclosures before the table.**

**(i) The tier ladder has no rung for primary published literature.** `CONTRACT.md` §1.2 defines
T1 (executable or declarative source read directly), T2 (vendor manual text), T3 (vendor raster
read visually) and T4 (course notes, project records, secondary literature). A coefficient taken
from a peer-reviewed primary paper — Amaefule 1993, Pittman 1992, Kolodzie 1980, Gunter 1997 — fits
none of them; it is stronger evidence than T4 but is not vendor material. Those rows are marked
**T4** with the paper named in the Source column, so a reader can see that the citation is a paper
and not a course note. This is escalated in §7.1 as a contract gap rather than resolved here.

**(ii) The laboratory and reservoir interfacial-tension defaults are refused, not transcribed.**
The only citation for them available on this machine is a vendor `.par` default set, and
`CONTRACT.md` §2.1 bars `.par` table data from transcription by name. This chapter does **not**
claim the Matthews & Kelly exception and does not reason from it — those rows are recorded
`ABSENT` and escalated in §7.5 as an acquisition gap. The consequence is stated plainly: the
product currently ships `IFT_RES = 26.0` sourced only to that file, so this is an existing
exposure that the chapter surfaces rather than creates.

### 5.1 Height ↔ pressure

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Leverett J constant (derived) | `J_C` | 0.21660110384 | — | Derived in-product from `6.894757e4 × sqrt(9.869233e-12)`; dossier §5.1 derivation | 1 |
| Leverett J constant (as-built) | `J_CONST` | 0.21645 | — | **UNSOURCED** — `satheight.rs:14`; 0.0698 % below derived | — |
| Leverett J constant (incumbent print) | — | 0.2166 | — | **NON-ADOPTABLE — cited for verification**; incumbent printed rounding, 0.00051 % below derived | 2 |
| Hydrostatic gradient per SG (derived, imperial route) | `PSI_PER_FT_PER_SG` | 0.4335277778 | psi/ft/SG | Derived from `62.428 / 144`; dossier §5.1 | 1 |
| Hydrostatic gradient per SG (derived, SI route) | — | 0.4335275224 | psi/ft/SG | Derived from `1000 × 9.80665`; agrees with the imperial route to 5.89e-7 | 1 |
| Hydrostatic gradient (as-built) | `PSI_PER_FT_PER_SG` | 0.433 | psi/ft/SG | **UNSOURCED** — `satheight.rs:20`; an incumbent rounding, 0.1217 % below derived | — |
| Hydrostatic gradient (incumbent, SI print) | — | 0.0981 | bar/m | **NON-ADOPTABLE — cited for verification**; 0.0342 % above derived | 2 |
| Metres per foot | `M_PER_FT` | 0.3048 | m/ft | Exact by definition of the international foot; `units.rs:34` | 1 |
| Natural-log ↔ base-10 factor for Thomeer `G` | — | 2.302585 | — | Derived, `ln(10)`; §2.2 | 1 |
| Water density default | `RHO_W` | 1.0 | g/cc | `docs/ref_shf.md:56` → Techlog sand-summary default, `techlog_ingest/FINDINGS.md` §C | 2 |
| Hydrocarbon density default (fitting path) | `RHO_HC` | 0.7 | g/cc | `docs/ref_shf.md:56-57` → Techlog sand-summary default, `techlog_ingest/FINDINGS.md` §C; range 0.1–0.8 | 2 |
| Hydrocarbon density default (forward-apply path) | `RHO_HC` | 0.8 | g/cc | **UNSOURCED** — `satheight.rs:112`; disagrees with the fitting default, worth 5.3 saturation units (§3.3a) | — |
| Hydrocarbon gradient default | — | `ABSENT — ships with no default` | psi/ft | Three incumbent defaults span 2.76× in height at fixed `Pc` (§2.1); no adjudication is defensible | — |
| Free-water level default | `FWL` | 2000.0 | m | **UNSOURCED** — `satheight.rs:110`; a project-specific quantity for which no default is meaningful | — |

### 5.2 Interfacial tension and contact angle

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Reservoir σ·cosθ, water-oil | `IFT_RES` | `ABSENT — ships with no default` | dyn/cm | Only citation on this machine is a vendor `.par` default set, barred by `CONTRACT.md` §2.1; no primary source for laboratory IFT/contact-angle standards is present. Acquisition gap — §7.5 | — |
| Reservoir σ·cosθ, water-gas | `IFT_RES` | `ABSENT — ships with no default` | dyn/cm | As above | — |
| Laboratory σ·cosθ, mercury injection | — | `ABSENT — ships with no default` | dyn/cm | As above | — |
| Laboratory σ·cosθ, centrifuge / porous plate | — | `ABSENT — ships with no default` | dyn/cm | As above | — |
| Reservoir σ·cosθ (as-built) | `IFT_RES` | 26.0 | dyn/cm | **UNSOURCED in code** — `satheight.rs:113`, `shf_fit.rs:733`; the doc comment at `shf_fit.rs:756-759` traces it to the barred `.par` set | — |
| Mercury-air σ·\|cosθ\| (as-built, fused) | `HG_AIR_IFT` | 367.0 | dyn/cm | **UNSOURCED** — `thomeer.rs:163`; a fused σ·cosθ product, forbidden by `SB-SHR-008`; 0.165 % below the value implied by the derived Hg constant | — |
| Pressure-unit constant for pore-throat radius | `PTR_CONST` | 0.1450377 | psi/(dyn·cm⁻²) | Derived unit conversion; dossier §5.1 | 1 |
| Mercury pore-throat constant | — | 106.6334 | µm·psi | Derived, `2 × PTR_CONST × σ·\|cosθ\|` for the mercury system; dossier §5.1 | 1 |
| Mercury pore-throat constant (SandiBumi reference bank) | — | 107.6 | µm·psi | **NON-ADOPTABLE — cited for verification**; 0.907 % above derived (§2.1) | — |

### 5.3 Saturation-height model coefficients

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Leverett forward coefficient | `SWH_A` | 0.5 | — | **UNSOURCED** placeholder — `satheight.rs:114`; must come from a stored fit (`SB-SHR-010`) | — |
| Leverett forward exponent | `SWH_B` | -0.4 | — | **UNSOURCED** placeholder — `satheight.rs:115` | — |
| Skelt-Harrison A | `SH_A` | 1.0 | — | **UNSOURCED** placeholder — `satheight.rs:116` | — |
| Skelt-Harrison B | `SH_B` | 30.0 | m | **UNSOURCED** placeholder — `satheight.rs:117`; hard-coded unit, see `SB-SHR-002` | — |
| Skelt-Harrison C | `SH_C` | 1.5 | — | **UNSOURCED** placeholder — `satheight.rs:118` | — |
| Skelt-Harrison D | `SH_D` | 0.0 | m | **UNSOURCED** placeholder — `satheight.rs:119`; hard-coded unit | — |
| Irreducible-water lower clamp | `SWT_IRR` | 0.0 | v/v | **UNSOURCED** — `satheight.rs:120`; 0.0 is a no-op, not a petrophysical value | — |
| Minimum porosity for a height signal | — | 0.005 | v/v | SandiBumi own screening threshold — `satheight.rs:162`; algorithmic, not petrophysical | 1 |
| Brooks-Corey exponent convention | `λ` | published form `Se = (Pd/Pc)^λ` | — | Brooks & Corey 1964, primary; implemented `shf_fit.rs:146-203` | 4 |
| Thomeer shape-factor log base | `G` | base 10 | — | Thomeer 1960, JPT 12(3) / Trans. AIME 219, primary; implemented `thomeer.rs:33`, `shf_fit.rs:361-367` | 4 |
| Thomeer `G` interpretive range | `G` | ≈0.1 (well sorted) to >2 (poorly sorted) | — | `docs/ref_shf.md:24`, project method bank | 4 |
| Buckles-check dispersion threshold | — | 0.6 | — | SandiBumi own heuristic on IQR/median — `shf_fit.rs:989-1010`; the Buckles concept is Buckles 1965, the threshold is not from it | 1 |

### 5.4 Rock-typing indicators

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Amaefule RQI constant | `RQI_C` | 0.0314 | µm·(mD)^-½ | Amaefule et al. 1993, SPE 26436, primary; via `docs/ref_rock_typing.md`; `hfu.rs:22` | 4 |
| Amaefule inverse permeability constant | `PERM_C` | 1014.24 | mD | Amaefule et al. 1993, primary; deliberately not the exact reciprocal-square (`1/0.0314² = 1014.24001`), kept as the literature value — `hfu.rs:23-24`, `hfu.rs:554-555` | 4 |
| Corbett-Potter GHE boundaries | `GHE_BOUNDS` | 0.0938, 0.1875, 0.375, 0.75, 1.5, 3.0, 6.0, 12.0, 24.0 | FZI units | Corbett & Potter 2004 ×2 geometric series, primary; corrected and re-verified 2026-07-22 per `docs/constants_verification_2026-07-22.md`; `rocktyping.rs:28` | 4 |
| Winland R35 coefficients | — | 0.732, 0.588, -0.864 | — | Kolodzie 1980, primary; `rocktyping.rs:101`; φ in **percent**, k in mD | 4 |
| Pittman r10–r75 coefficient table | `PITTMAN_TABLE1` | 14 published rows, held single-sited at `rocktyping.rs:322-337` | — | Pittman 1992, AAPG Bulletin v.76 no.2 p.191–198, Table 1 (p. 196), primary; verified against the paper 2026-08-01, `docs/review_triage.md` finding 9. Not restated here: the source of truth is the paper, and a second transcription is a second thing to get wrong | 4 |
| Pittman correlation-coefficient range | `R` | 0.926 at r20 falling to 0.820 at r75 | — | Pittman 1992, Table 1; the paper states accuracy diminishes above the 55th percentile (p. 195) | 4 |
| Lucia / Jennings-Lucia transform coefficients | `LUCIA_A..D` | 9.7982, 12.0838, 8.6711, 8.2965 | — | Jennings & Lucia 2003, primary, **via a secondary transcription anchor** (`docs/research_2026-07/ref_rocktyping_shf.md`); `rocktyping.rs:155-162` still marks these **verify-before-release**. Status `PRESENT-UNVERIFIED` | 4 |
| Lucia calibrated RFN band | `RFN` | 0.5 to 4 | — | Jennings & Lucia 2003, primary; class bands 1 / 1.5 / 2.5 / 4 at `rocktyping.rs:179-191` | 4 |
| Lucia out-of-range null threshold | — | `ABSENT — ships with no default` | — | One incumbent nulls above 20 (T1); no primary basis for that number is on this machine, and SandiBumi nulls only the class (§3.6) | — |
| Port-size class boundaries (Hartmann-Beaumont) | `PORT_BOUNDS` | 0.1, 0.5, 2.5, 10.0 | µm | Hartmann & Beaumont scheme; `rocktyping.rs:32`. MUST be one of two selectable named schemes per `SB-SHR-015`, never a silent default | 4 |
| Port-size Macro/Meso boundary (alternative scheme) | — | 2.0 | µm | Second published scheme carried by another incumbent; must be selectable alongside the above | 2 |
| Permadi-Susilo exponent | `PS_EXP` | 3.0 | — | `rocktyping.rs:59`; re-verified 2026-07-22 per `docs/constants_verification_2026-07-22.md` | 4 |
| Reservoir-flag cut-off defaults | `VSH1`, `PHI1`, `VSH2`, `PHI2` | 0.15, 0.12, 0.35, 0.06 | v/v | **UNSOURCED** — `rocktyping.rs:235-257`; see `SB-SHR-023` | — |
| FZI porosity basis | `φz` | effective porosity | v/v | Amaefule et al. 1993, primary; feeding total porosity offsets FZI by a **k-invariant** factor (1.63583 on the dossier's worked sample) that a cluster sweep cannot detect (§2.3) | 4 |

### 5.5 Capillary-pressure-derived permeability

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Swanson apex bulk-volume basis | — | `ABSENT — ships with no default` | — | Three mutually inconsistent bases across the corpus give 0.00864 / 0.000193 / 1.97 mD on one sample — a 10⁴ spread. Dossier ESC-7, **BLOCKING**; `SB-SHR-014` forbids a default | — |
| Swanson coefficient pair (as-built) | — | 399.0, 1.691 | mD, — | **NON-ADOPTABLE — cited for verification** — `thomeer.rs:264`; an incumbent's uncited pair, shipped with the module's own header recording it as unverified (`thomeer.rs:13-14`) | 2 |
| Purcell / Katz-Thompson / Ruth-Lindsay / Oklahoma-U constants | — | `ABSENT — ships with no default` | — | Not implemented; primary sources not yet acquired. Acquisition gap — §7.5 | — |
| Oklahoma-University porosity exponent | — | -3.06 | — | **NON-ADOPTABLE — cited for verification**; recorded only to quantify the 1.318e6× porosity-unit stake (§2.2), not to implement the correlation | 2 |
| Beta parameter unit declaration | `BETA` | `GFT-1` declared vs `FT-1` consumed | — | **NON-ADOPTABLE — cited for verification**; a 1e9 mismatch inside one incumbent's own parameter file (§2.2) | 1 |

### 5.6 Partitioning and heterogeneity

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Auto-K marginal-gain tolerance | `AUTO_K_TOL` | 0.02 | fraction of single-segment SSE | SandiBumi own — `lorenz.rs:33`; algorithmic, not petrophysical | 1 |
| Auto-K upper bound | `AUTO_K_MAX` | 12 | segments | SandiBumi own — `lorenz.rs:35`; matches the HFU cluster cap at `hfu.rs:210` | 1 |
| HFU requested-cluster clamp | — | 1 to 12 | clusters | SandiBumi own — `hfu.rs:210` | 1 |
| Lorenz coefficient definition | `Lc` | `2·(A − ½)`, clamped to [0,1] | — | Schmalz & Rahme 1950, reviewed in Lake & Jensen 1991 (SPE-20156), primary; `lorenz.rs:204-223`, `docs/ref_rock_typing.md:15-18` | 4 |
| SMLP slope interpretation boundary | — | 1.0 (the well-average k/φ) | — | Gunter et al. 1997, SPE-38679-MS, primary; the diagonal, not a tuned constant — `lorenz.rs:14-15` | 4 |
| Dykstra-Parsons percentiles | `VDP` | k₅₀ and k₁₅.₉ | mD | Published definition `(k₅₀ − k₁₅.₉)/k₅₀`; not implemented (`SB-SHR-031`) | 4 |
| Flow-unit boundary rule (Lorenz inflection) | — | `ABSENT — ships with no default` | — | To be derived from Gunter et al. 1997 per §7.4; the inflection criterion itself is a first-principles property of the cumulative curve, but no numeric tolerance is adopted from any source yet | — |

---

## 6. Acceptance tests

Each test states the assertion that must hold, not an implementation. Tests marked **must fail
today** are the ones that would catch a live defect; the rest pin behaviour that is already
correct so that it cannot regress.

### 6.1 Depth-unit arithmetic

**`SB-SHR-T01`** — *Branch parity under unit declaration.* One physical column, described twice
(H m in a metre-declared project, `H × 3.28084` ft in a foot-declared one), evaluated through
**every** saturation-height family the product ships. Every family MUST return the same `SWH` to
within float tolerance. **Must fail today** on the Skelt-Harrison branch (`satheight.rs:175`) and
pass on Leverett. Verifies `SB-SHR-001`.
*Control:* the test MUST be parameterised over the family list and MUST fail when a new family is
added without a unit declaration — a per-branch test is what let this defect through.

**`SB-SHR-T02`** — *No raw height reaches a model.* A static check: no expression in the domain may
consume the height variable produced at `satheight.rs:154` without passing it through the unit
conversion. Verifies `SB-SHR-001`.

**`SB-SHR-T03`** — *Length-dimensioned parameters re-express on unit change.* Register a Skelt law
with `B = 30` on a metre-declared project, switch the project to feet, and assert `B` reads
98.4252 and the computed `SWH` is unchanged. **Must fail today.** Verifies `SB-SHR-002`.

**`SB-SHR-T04`** — *Undeclared unit refuses.* With no project depth unit declared, every entry point
in this domain MUST return a refusal naming the missing declaration, and MUST compute nothing —
asserted on the result, not on a log line. **Must fail today** (`units.rs:179-180` returns
`Metres`). Verifies `SB-SHR-003`.

**`SB-SHR-T05`** — *Height-dimensioned labels follow the project.* On a foot-declared project, the
`HAFWL` output curve's unit string, the `FWL` prompt's unit, the plot axis label and the LAS export
header MUST all read feet. **Must fail today** (`satheight.rs:110`, `:125`). Verifies `SB-SHR-004`.

### 6.2 Constants

**`SB-SHR-T06`** — *One fluid-property default across fit and apply.* Assert that the fitting
default and the forward-apply default for `RHO_W`, `RHO_HC` and `IFT_RES` resolve to the same
object. Then the behavioural half: fit a synthetic Leverett law, apply it forward, and assert the
recovered `Sw` matches the fitted `Sw` to within 0.5 saturation units. **Must fail today** — the
round trip is 5.3 saturation units off (§3.3a). Verifies `SB-SHR-005`.

**`SB-SHR-T07`** — *Constants are derived, not literal.* Assert `J_C` equals
`6.894757e4 × sqrt(9.869233e-12)` and the gradient equals `62.428/144`, each to full double
precision, and that each carries a non-empty source string. Assert additionally that the two
independent gradient derivations agree to better than 1e-6. **Must fail today.** Verifies
`SB-SHR-006`.

**`SB-SHR-T08`** — *One definition site per constant.* A repository-level check that each of
`J_C`, the gradient, `RQI_C` and `PERM_C` appears as a literal exactly once, including in
user-facing strings and test fixtures. **Must fail today** on four counts: `rocktyping.rs:98`,
`rocktyping.rs:132`, `shfDialog.ts:124`, `ingest.rs:2313`. Verifies `SB-SHR-007`.
*Why the fixture clause matters:* with `ingest.rs:2313` synthesising its data from the same
literal the code uses, the existing round-trip assertion cannot detect a wrong `J_CONST`. The test
MUST synthesise from the derived expression and assert against the shipped constant.

**`SB-SHR-T09`** — *σ and cosθ are never fused.* Assert no constant in the domain stores a
σ·cosθ product, and that changing a stored contact angle changes every derived pore-throat radius.
**Must fail today** (`thomeer.rs:163`). Verifies `SB-SHR-008`.

### 6.3 Fitted objects

**`SB-SHR-T10`** — *A fit survives the dialog.* Fit a law, close the dialog, reopen the project,
and assert the law is retrievable with identical coefficients. **Must fail today** — nothing is
persisted. Verifies `SB-SHR-009`.

**`SB-SHR-T11`** — *A fit states what it was fitted on.* Assert every persisted fitted object
carries: well list, log set and curve versions, sample count, the complete exclusion ledger,
`ρw`/`ρhc`/`σcosθ`, FWL, method, and fit statistic — and that a fit missing any of these is not
applicable. Verifies `SB-SHR-009`, discharges this domain's share of `SB-CORE-014`.

**`SB-SHR-T12`** — *Forward apply consumes the object.* Assert the forward module can be driven
only from a stored fitted object; assert that a user override is recorded together with the value
it replaced. Verifies `SB-SHR-010`.

**`SB-SHR-T13`** — *The FWL scan reports an interval.* On a synthetic dataset with a deliberately
flat residual curve, assert the returned interval is wide and the result says so; on a
sharply-minimised curve, assert it is narrow. **Must fail today** — only an argmin is returned.
Verifies `SB-SHR-011`.

### 6.4 Convention divergence

**`SB-SHR-T14`** — *Brooks-Corey emits both conventions.* Assert a fit returns λ and `N = 1/λ`,
each labelled, and that importing a coefficient without a declared convention is refused. Assert
the numerical stake directly: the same curve evaluated under the two conventions differs by ≈26
saturation units, so a silent swap cannot be mistaken for noise. Verifies `SB-SHR-012`.

**`SB-SHR-T15`** — *Thomeer declares its log base.* Assert both `G` forms are emitted and that
`G_ln = 2.302585 × G_log10` to full precision; assert an undeclared-base import is refused.
Verifies `SB-SHR-013`.

**`SB-SHR-T16`** — *No Swanson default exists.* Assert that with no apex basis selected, Swanson
permeability is `MISSING` and the result says why. **Must fail today** (`thomeer.rs:263-264`).
Verifies `SB-SHR-014`.

**`SB-SHR-T17`** — *The basis travels, and unverified constants surface.* Assert the selected apex
basis is present on every Swanson result and on every export of one; assert a result computed from
a constant flagged unverified carries that flag to the dialog, the export and the report. Assert
the three bases on one sample reproduce the dossier's 0.00864 / 0.000193 / 1.97 mD, so the test
also pins the magnitude of the choice. Verifies `SB-SHR-014`, `SB-SHR-024`.

**`SB-SHR-T18`** — *Port-class scheme is a recorded choice.* Assert both schemes are selectable,
that a plug with apex radius 2.2 µm classifies differently under each, and that the active scheme
appears on the result. **Must fail today** (`rocktyping.rs:32`). Verifies `SB-SHR-015`.

**`SB-SHR-T19`** — *`RQI` namespace collision is refused.* Assert that feeding the saturation-model
`RQI` into the Amaefule consumer is refused by type or by declared quantity, not resolved by name
matching. Assert the collision's magnitude — 11.8× in `Swirr` — is reproduced in the fixture so the
test documents why it matters. Verifies `SB-SHR-016`.

**`SB-SHR-T20`** — *Porosity unit is a precondition, not a comment.* Feed each `log φ` correlation a
porosity curve in percent where fraction is declared; assert every one fails the run and names the
curve. Assert the untrapped magnitudes as fixture constants: 676.08× for Pittman R25, 234.42× for
the apex correlation, 1.3183e6× for the Oklahoma-University form. **Must partially fail today** —
`rocktyping.rs:453` skips rather than raises. Verifies `SB-SHR-017`.

**`SB-SHR-T21`** — *Gradient divergence is shown as height.* Assert that where multiple candidate
hydrocarbon gradients exist, the chooser displays the height each implies at a fixed `Pc`, and that
the shipped spread reproduces 27.9 / 54.6 / 77.0 ft at `Pc = 10 psi`. Verifies `SB-SHR-018`.

**`SB-SHR-T22`** — *σ-scoped coefficients cannot become defaults.* Assert no coefficient marked
`NON-ADOPTABLE` is reachable as a default or a seed anywhere in the domain. Verifies
`SB-SHR-019`.

### 6.5 Validity conditions

**`SB-SHR-T23`** — *Lucia band is two-sided.* Assert an `RFN` of 0.2 is not classified as Class 1
and is flagged out-of-band, and that the `RFN` curve and the class curve agree about which samples
are in range. **Must fail today** (`rocktyping.rs:179-191`). Verifies `SB-SHR-020`.
*Control:* assert the porosity unit is unchanged — a plug at φ = 0.05 v/v must give RFN 2.07, not
7.41, confirming the fraction basis is not disturbed by the fix.

**`SB-SHR-T24`** — *Extrapolation regime is flagged per sample.* At φ = 5 %, k = 1 mD, assert
`PR40 = 0.77`, `PR50 = 0.86`, `PR75 = 1.11` µm — the published non-monotone ordering — and assert
each of those samples carries an extrapolation flag. Assert **no clamping occurs**: the values must
not be forced monotone. Verifies `SB-SHR-021`.

### 6.6 Honesty contract

**`SB-SHR-T25`** — *Every module carries the exclusion ledger.* Run each module in this domain over
a frame where a known fraction of samples fails a precondition, and assert the result names the
reason and the count, and that the count reconciles exactly with the missing samples. **Must fail
today** for every registered module. Verifies `SB-SHR-022`.
*Regression guard:* assert the existing `shf_fit.rs` ledger still populates for all five documented
exclusion reasons, so extending the pattern cannot break the one working instance.

**`SB-SHR-T26`** — *Unclassified is not the lowest class.* Assert a sample failing every
`rt_cutoff` criterion returns an explicit unclassified value distinct from class 3, and that each
boundary either carries a source or requires user entry. **Must fail today**
(`rocktyping.rs:270-276`). Verifies `SB-SHR-023`.

**`SB-SHR-T27`** — *The report names the method used.* Render a report for a study whose pay came
from `SWH` and assert the methodology table contains a saturation-height row naming the family, its
coefficients and its FWL, and does **not** present an electrical-model row as the method used.
**Must fail today** (`report.rs:61-91`). Verifies `SB-SHR-025`.

### 6.7 Capability completeness

**`SB-SHR-T28`** — *Lab→reservoir conversion.* Assert `Pc_res = Pc_lab · (σ|cosθ|)_res /
(σ|cosθ|)_lab` round-trips exactly, and that conversion is refused when either system is
undeclared. Verifies `SB-SHR-026`.

**`SB-SHR-T29`** — *Closure correction is a recorded choice.* Assert all four treatments are
available, that they give measurably different entry pressures on one synthetic curve, that no
default is applied, and that the chosen treatment travels with every derived Thomeer parameter.
Verifies `SB-SHR-027`.

**`SB-SHR-T30`** — *Corrections act on the non-wetting phase.* Assert stress and clay-bound-water
corrections change the non-wetting bulk volume and that applying either to the wetting phase is
structurally impossible, not merely discouraged. Verifies `SB-SHR-028`.

**`SB-SHR-T31`** — *Modality is detected and reported.* On a synthetic bimodal throat distribution,
assert two modes are found and that the result recommends more than one saturation-height law.
Assert the log-side Buckles diagnostic (`shf_fit.rs:989-1010`) is reported alongside it where both
are available. Verifies `SB-SHR-029`.

**`SB-SHR-T32`** — *Lorenz-inflection partitioning.* On a synthetic column with three known
flow-capacity/storage-capacity gradient regimes, assert the inflection partitioner recovers the
three boundaries at their true depths, and that the active partitioning method is recorded on the
result. Assert it is selectable alongside the existing Ward segmentation and that the two are
allowed to disagree. Verifies `SB-SHR-030`.
*Reuses:* `lorenz.rs:438-454` already builds exactly this three-regime synthetic for the Ward path.

**`SB-SHR-T33`** — *Dykstra-Parsons alongside Lorenz.* Assert `VDP` is computed and reported with
`Lc`, and that on a strongly layered synthetic the two indices are both returned rather than one
being preferred. Verifies `SB-SHR-031`.

**`SB-SHR-T34`** — *Multiple MICP permeability routes.* Assert at least Purcell and Katz-Thompson
are available alongside Swanson, each with its own source, and that their answers are presented
side by side. Verifies `SB-SHR-032`.

**`SB-SHR-T35`** — *Indicators are separately named.* Assert Winland R35, Aguilera R35 and PGS are
distinct outputs and that no code path substitutes one for another. Verifies `SB-SHR-033`.

### 6.8 Seams

**`SB-SHR-T36`** — *The gradient service serves chapter 14.* Assert
`14_cutoffs-summation-mc.md`'s `HCPV` thickness→volume conversion calls this domain's gradient
service, uses the single derived constant, takes fluid densities from the fitted object in force,
and refuses on an undeclared depth unit. Verifies `SB-SHR-034`.

**`SB-SHR-T37`** — *Cut-offs are derived with their evidence.* Assert a derived cut-off carries the
rock-type partition and Pc curve it came from, and that the domain does not select one. Verifies
`SB-SHR-035`.

**`SB-SHR-T38`** — *Lambda is absent, not approximated.* Assert the Lambda family is unavailable and
that no adjacent family is offered in its place. Verifies `SB-SHR-036`.

### 6.9 Reference conditions, import safety, model selection

**`SB-SHR-T39`** — *Reference condition is part of the type.* Assert a `Swirr` cannot be
constructed without both a pressure and a lab/reservoir flag, and that the same numeric pressure
under the two flags selects points an order of magnitude apart on one synthetic curve — pinning
why the flag exists. Verifies `SB-SHR-037`.

**`SB-SHR-T40`** — *Untagged corrected Pc is refused.* Assert the SCAL importer refuses a corrected
`Pc` curve with no correction-provenance tag, and accepts the same curve once tagged. Assert
module identity resolves from the declared name when the specification line names a different
module. Verifies `SB-SHR-038`.

**`SB-SHR-T41`** — *The sweep is exhaustive, uncapped and reproducible.* Assert the model-selection
sweep evaluates every combination on its declared grid — compared against an independently
enumerated cross-product, not against its own output — that it consumes every depth sample rather
than a capped or sampled subset, that two runs on identical inputs give identical rankings, and
that the full ranking is retained. Verifies `SB-SHR-039`.
*Control:* the reproducibility assertion must fail if any random sampling is introduced; a sweep
that is merely *usually* the same is not deterministic.

### 6.10 Reproducibility and regression direction

**`SB-SHR-T42`** — *No result depends on a display setting.* Compute a closure pick, an entry
pressure and a rock class with the plot closed, with a linear axis and with a logarithmic axis;
assert all three runs agree bit-for-bit. Verifies `SB-SHR-040`.

**`SB-SHR-T43`** — *Regression direction is preserved and recorded.* Assert that predicting `k`
from a Pittman radius does not use an algebraic inversion of the published `R = f(k, φ)` fit;
assert the recorded direction is present on the result. Pin the stake: an inverted route diverges
by 2.0× at φ = 5 % and 1.9× at 30 % **in opposite directions**, so the fixture must span both ends.
Verifies `SB-SHR-041`.

**`SB-SHR-T44`** — *Contact-angle convention is single-valued.* Assert one validation range
product-wide, assert every capillary expression takes `|cos θ|`, and assert that a stored angle
and its cosine cannot disagree. Pin the failure mode: 40° and 140° must give the same `|cos θ|` to
3 decimal places (0.766 vs 0.765 is the corpus's own rounding spread), so a sign flip is detectable
but a rounding difference is not mistaken for one. Verifies `SB-SHR-042`.

---

## 7. Open items, escalations and refusals

### 7.1 Contract and core-requirement observations

**No new `SB-CORE` id is minted by this chapter.** Two candidates were considered and one is
raised for Jauhar's decision.

- *Rejected as already covered:* "a fit and its forward apply must share their parameter defaults."
  This is `SB-CORE-007` (one definition per constant), applied to a default rather than a constant.
  It is specified as `SB-SHR-005` under that allocation and needs no new id.
- **Raised for a decision:** *a test fixture MUST NOT be synthesised from the same literal the
  test guards.* `ingest.rs:2313` builds its SCAL fixture from `0.21645` — the exact value of
  `J_CONST` — so the round-trip assertion moves with the constant and cannot detect a wrong one.
  This is a **testing-discipline** requirement rather than a domain one, it is not covered by any
  existing `SB-CORE`, and it is very unlikely to be unique to this chapter. If sibling chapters
  have found the same pattern it should be a core requirement; this chapter has specified it
  locally as a clause of `SB-SHR-007` in the meantime. **I may not mint the id and have not.**

**The tier ladder has no rung for primary published literature** (§5, disclosure (i)).
`CONTRACT.md` §1.2 runs T1 executable source, T2 vendor manual, T3 vendor raster, T4 course
notes / project records / secondary literature. A coefficient from Amaefule 1993, Pittman 1992,
Kolodzie 1980 or Gunter 1997 is stronger evidence than any of these and belongs to none. Marking
such rows T4 understates them and puts a peer-reviewed table in the same bucket as a course note.
This chapter has marked them T4 with the paper named rather than invent a tier. **Recommendation:
a `T0` or `TP` rung for primary published literature, applied across all eighteen chapters at
once** — a per-chapter fix would make the tiers incomparable, which is worse than the current
understatement.

**`SB-CORE-001`'s description is partly stale for this domain.** `04_CORE_REQUIREMENTS.md` and the
risk register describe the capillary-pressure code as multiplying height by 3.28084 on an
assumption of metres. For the **Leverett** path that is no longer true: `satheight.rs:189`,
`shf_fit.rs:911` and `shf_fit.rs:1095` all route through the unit carrier, and
`satheight.rs:246` regression-tests the equivalence. The live defect is the **Skelt-Harrison**
branch, the undeclared-unit default, and the hard-coded unit labels (§3.1). I have not edited that
file. **Request: update `SB-CORE-001`'s status text to name the Skelt branch, so the next reader
does not chase a closed defect and miss an open one.**

### 7.2 Escalations — decisions needed from Jauhar

**`ESC-SHR-1` — the vendor `.par` fluid-property transcription boundary. Stopped and escalated,
as instructed; no exception is claimed.** The reservoir and laboratory σ / cosθ defaults have
exactly one citation on this machine: a vendor `.par` default set, whose file type
`CONTRACT.md` §2.1 bars from transcription by name. This chapter therefore records all four
systems `ABSENT` in §5.2 and ships no default. **The problem is that the shipped product already
carries them:** `satheight.rs:113` and `shf_fit.rs:733` default `IFT_RES` to 26.0, traced in
`shf_fit.rs:756-759` and `docs/ref_shf.md:53-55` to that same barred file. So the choice is not
whether to transcribe — it is whether to keep what is already transcribed.

Three ways out, all requiring Jauhar's decision, none taken here:
1. **Acquire a primary source.** A standard core-analysis text gives laboratory and reservoir
   interfacial tensions and contact angles as physical properties, not vendor IP. This is the
   clean answer and is listed in §7.5 as the highest-value acquisition in the chapter.
2. **Rule that a scalar fluid property is not "table data"** and that §2.1's `.par` clause targets
   bulk lookup grids. Defensible, but it is a contract amendment and I may not make it.
3. **Remove the defaults from the product** and require user entry. Safe, and costs a feature.

I have not reasoned from the Matthews & Kelly exception and do not propose one.

**`ESC-SHR-2` — Swanson ships against a BLOCKING escalation.** `thomeer.rs:263-264` ships a
default apex basis and an uncited coefficient pair. Dossier ESC-7 says ship no default; the stake
is 10⁴ in permeability. `SB-SHR-014` requires the default's removal. **This is a petrophysical
parameter decision and is Jauhar's**: remove the default outright, or select a basis with a
citation. It should not ship as it stands in either case, because the module's own header records
the constants as unverified and nothing carries that to the user.

**`ESC-SHR-3` — which hydrocarbon-density default is the house value, 0.7 or 0.8?**
`shf_fit.rs:729-731` says 0.7 with a vendor-manual trace; `satheight.rs:112` says 0.8 with no
trace. The round trip costs 5.3 saturation units. Whichever survives, the other must go, and the
answer is a petrophysical judgement, not an editorial one.

**`ESC-SHR-4` — the Lucia coefficients are still `verify-before-release`.** `rocktyping.rs:155-162`
has carried that flag since before 2026-07 and the transcription anchor is a secondary document,
not the paper. Status in §5.4 is `PRESENT-UNVERIFIED`. Closing artefact: Jennings & Lucia 2003
itself. Until then, carbonate rock typing should not go into a deliverable unflagged — and
carbonate is secondary in Jauhar's clastic-dominated setting, so this is genuinely low-cost to
hold.

**`ESC-SHR-5` — dossier ESC-12 is a product requirement awaiting a decision, and this chapter has
pre-empted it.** The dossier records it as OPEN pending Jauhar, because it is a design decision no
vendor artefact will settle. This chapter has specified it as `SB-SHR-011` at **P0** on the
strength of the delivered-study precedent (a saturation-height function built and then rejected
because the FWL could not be picked per layer). **If Jauhar disagrees with P0, the requirement
stands but the priority is his call.**

### 7.3 Defect refusals — things SandiBumi correctly declines to reproduce

These are **competitive wins**, not gaps. Each discharges `03_EVIDENCE_BASE.md` §14.1.

**(a) It refuses to clamp the Pittman family into a monotone ordering.**
`rocktyping.rs:400-410` documents that below ≈11 % porosity the independently-regressed rows stop
falling monotonically — at 5 % porosity and 1 mD, `PR40 = 0.77 µm` but `PR50 = 0.86` and
`PR75 = 1.11` — and declines to force the ordering "because forcing the ordering would report radii
the paper never published." **This is correct and must not be changed.** The rows are separate
regressions, not samples of one curve; a clamp would manufacture a number Pittman never fitted.
What SandiBumi does instead: publishes the true values, documents the regime, and (per
`SB-SHR-021`) will flag the affected samples.

**(b) It refuses the three printed-wrong capillary-pressure correction formulae.**
Dossier ESC-10 catalogues an incumbent's shipped manifests in which a Kozeny-Carman throat ratio
has its square root scoped to the numerator only (dimensionally inconsistent), an overburden `Pc`
correction that makes `Pc` *decrease* as the throat shrinks (backwards — `Pc ∝ 1/r`), and a
porosity-ratio correction whose literal bracket reading gives **×12.9** where the intended form
gives **×1.155** at φe = 0.15 / φt = 0.20. SandiBumi adopts the internally-consistent,
independently-corroborated forms instead, and — per `SB-SHR-038` — will refuse an imported
corrected `Pc` curve that cannot say which reading produced it.

**(c) It refuses the natural-log Thomeer and the `1/N` Brooks-Corey conventions.**
`thomeer.rs:33` and `shf_fit.rs:361-367` use base 10, Thomeer's published base;
`fit_brooks_corey` (`shf_fit.rs:146-203`) stores `λ`, Brooks and Corey's published exponent. Each
refusal is worth 12 and 26 saturation units respectively against a tool that chose the other side
silently. The remaining work (`SB-SHR-012`, `SB-SHR-013`) is to *declare* the convention, not to
change it.

**(d) It refuses the percent-porosity Lucia basis.** `lucia_rfn` (`rocktyping.rs:166-177`) takes
interparticle porosity in v/v and validates it — the side of the trap that returns RFN 2.07 rather
than 7.41, i.e. Class 2 rock reported as Class 2 rather than as outside the calibrated band
entirely.

**(e) It refuses σ-scoped coefficients.** Where an incumbent divides `Pc` by interfacial tension
inside its transform — making every downstream coefficient scoped ×72 or ×480 —
`thomeer.rs:226` instead standardises with an **explicit** σ ratio carried per row. The
standardisation constant itself is a separate defect (`SB-SHR-008`); the architecture is right.

**(f) It refuses to make two literature constants agree.** `hfu.rs:554-555` records that
`1/0.0314² = 1014.24001`, not `1014.24`, and keeps both published values rather than deriving one
from the other. Tidying that would silently substitute a number Amaefule did not publish.

**(g) It refuses to compute a flow-unit partition on a substituted curve.** `lorenz.rs:526` fails
the run and names the missing curve, with a control asserting the blame moves when the absence
moves. A partition computed off the wrong curve returns a plausible `Lc` and a plausible unit
count and nothing downstream can detect it.

**(h) It refuses to drop a failed per-rock-type fit.** `fit_groups` (`shf_fit.rs:957-983`) returns
failed classes with their reason. A dropped class looks like a class that was never there.

**(i) This chapter refuses to transcribe the vendor `.par` fluid-property set** (§7.2,
`ESC-SHR-1`), accepting an `ABSENT` default rather than an unciteable one.

**Refusals specified but not yet implemented.** Stated separately, because SandiBumi has not built
the affected capability yet and cannot be credited with declining something it never reached:

- **(j) A numerical result that depends on a plot checkbox.** Dossier finding 3.14 records an
  incumbent whose closure pick and entry pressure differ according to whether an axis is drawn
  logarithmic. `SB-SHR-040` forbids the pattern product-wide before closure correction is built,
  which is the cheap moment to forbid it.
- **(k) A control-flow fall-through that reports a correction the code did not apply.** Dossier
  finding 3.15 traces an incumbent's closure routine in which a trailing `else` binds to the wrong
  `if`, so on the **shipped default method** a fallback runs after the real correction and
  overwrites the *reported* correction identifiers with the fallback's. The output states one
  method and carries another. `SB-SHR-027`'s requirement that the chosen treatment travel with the
  corrected curve — and `SB-SHR-038`'s refusal of untagged imports — are the structural answers.
- **(l) A height↔pressure relation that drops the gradient.** Dossier finding 3.9 records
  crossplot panel text giving `Height = Pc/(ρw − ρhc)` where the correct form carries the gradient,
  a **2.309× error in height** — 33.3 ft against 77.0 ft at `Pc = 10 psi`, `Δρ = 0.3 SG`.
  `SB-SHR-006` and `SB-SHR-034` make the relation a single service, so there is no second place for
  it to be written down differently.

### 7.4 Independent-derivation requirements

**The dossier states that this domain does not touch the Tier-C register.** Verbatim: the Tier-C
items — Experienced Eye/EEFS, Domain Transfer Analysis, entropy speed-correction, neural-network
weight files, Textural Facies tile encodings, frequency-domain dispersion fits — are *"not touched
by this domain"*, because *"every method above sits in published literature or plain-text vendor
contracts, none Tier-C"* (dossier §7, verified at source). That finding is accepted. What follows
is therefore **not** a Tier-C register entry but two capabilities specified under the same
discipline, because the risk the amendment guards against applies to both.

---

**`SB-SHR-030` — automatic flow-unit partitioning by Lorenz-curve inflection.**

*Class:* **boundary case — not Tier C.** The reference implementation available on this machine is
plain-text Python read directly (T1), not a binary, weight file or undisclosed encoding, so
consuming it would not be the prohibited path. **It is nonetheless derived independently by
election**, because the amendment's technical rationale binds regardless of tier: a method taken
from a vendor's implementation inherits that implementation's defects and forfeits
`03_EVIDENCE_BASE.md` §14.1, which is the product's primary competitive claim. The vendor file is
cited as corroboration that the approach is in commercial use — nothing more.

*Primary sources:* Gunter, Finneran, Hartmann & Miller (1997), *Early Determination of Reservoir
Flow Units Using an Integrated Petrophysical Method*, SPE-38679-MS — the Stratigraphic Modified
Lorenz Plot and the flow-unit/baffle/seal interpretation of its slope. Schmalz & Rahme (1950),
reviewed in Lake & Jensen (1991), SPE-20156 — the heterogeneity index. The inflection criterion
itself is a first-principles property of the cumulative curve: a flow unit boundary **is** a change
in `dy/dx`, and `dy/dx = (kᵢ/φᵢ)·(Σφh/Σkh)` with the thickness cancelling
(`docs/ref_rock_typing.md:31`). No numeric tolerance is adopted from any source; §5.6 records it
`ABSENT` pending calibration on real data.

*Betters:* removes the **statistical-criterion blindness** of the alternatives. The Ward criterion
SandiBumi already implements (`lorenz.rs:152-188`) minimises within-segment sum of squares, and
the incumbent alternatives bin on permeability range or cluster by K-means — all of which place
boundaries where the *values* separate, not where the **flow-capacity/storage-capacity gradient
changes**. On a column whose `k/φ` drifts smoothly, a sum-of-squares partitioner will invent a
boundary in the middle of one flow unit; a gradient-inflection partitioner will not. The documented
limitation removed is the one the dossier states directly in recommending this method: it is the
only candidate whose boundaries carry a **physical** meaning rather than a statistical one, and
the incumbent alternatives are respectively range-binned (Geolog, T1) and K-means-seeded
(IP, T2) — neither of which references the Lorenz gradient at all.

*Owning requirement:* `SB-SHR-030`. *Test:* `SB-SHR-T32`.

---

**`SB-SHR-039` — deterministic model-selection sweep across this domain's competing choices.**

*Class:* **C-2 — proprietary implementation, publicly described** — by analogy rather than by
register membership. The domain's Tier-C register is empty (above), but the *capability* an
Experienced-Eye-class harness provides — sweep a parameter and method cross-product, rank the
results against control data — is precisely the unmet need here, and the contract's C-2 terms are
the right terms for specifying it.

*Primary sources:* SPWLA-2021-0091 (Brackenridge et al.) as the public description of the
capability class. The domain content of the sweep is entirely this chapter's own published
sources — Amaefule 1993, Pittman 1992, Kolodzie 1980, Gunter 1997, Brooks & Corey 1964, Thomeer
1960, Skelt & Harrison 1995, Cuddy 1993/2017, Leverett 1941 — none of which is proprietary. No
vendor implementation is read, and none is needed: the corpus already establishes that the
capability is **a brute-force cross-product harness, not an algorithm**, proven by three exact
cross-product reproductions.

*Betters:* removes three documented limitations of the incumbent implementation, each cited in
`CONTRACT.md` §2.2's own worked case. (1) It is **uncapped in depth samples**, where the shipped
incumbent is **capped at 475 depth levels** — a cap that silently truncates any field-scale run.
(2) It is **exhaustive and deterministic**, where the shipped incumbent **samples 100 randomly**
while the same vendor's standalone tool uses **200 sorted** — so the incumbent's answer is not
reproducible and is not even self-consistent across that vendor's own two surfaces. (3) It
**records its provenance** under `SB-CORE-010` and `SB-CORE-014`, where the incumbent records
none. Three stated axes, none requiring their code.

*Owning requirement:* `SB-SHR-039`. *Test:* `SB-SHR-T41`.

---

**Boundary note.** Domain Transfer Analysis and Textural Facies (both C-2) serve facies and
image-log needs, not saturation-height or rock-typing needs, and are owned by `17_facies-ml.md`
and the image-log chapter respectively. Declining to specify them here is a **scope statement, not
a refusal** — the owning chapters must specify them under §2.2's terms.

### 7.5 Acquisition gaps — specific missing sources

Ranked by what they unblock.

| # | Missing source | Unblocks | Consequence today |
|---|---|---|---|
| 1 | A primary reference for laboratory and reservoir interfacial tension and contact angle (standard core-analysis text) | `SB-SHR-026`, and removes `ESC-SHR-1` entirely | Four `ABSENT` rows in §5.2; the shipped `IFT_RES = 26.0` has no citable source |
| 2 | Jennings & Lucia (2003) | Clears `PRESENT-UNVERIFIED` on the four Lucia coefficients | Carbonate rock typing cannot go into a deliverable unflagged |
| 3 | Swanson (1981) primary | `SB-SHR-014`; adjudicates the apex basis | The 10⁴ BLOCKING escalation stays open |
| 4 | Purcell, Katz-Thompson, Ruth-Lindsay primaries | `SB-SHR-032` | Swanson is the only MICP permeability route, which is why a default is dangerous |
| 5 | Worthington & Cosentino (2005); Qassamipour et al. (2020) | Cut-off **selection** — `14_cutoffs-summation-mc.md`'s, inherited here, **not re-escalated** | Chapter 14's gap, restated so this chapter does not appear to have closed it |
| 6 | Amaefule et al. (1993), SPE 26436, full text | Would close dossier ESC-14 (`Fs`, `τ`, `Sgv` undefined) | Moot: no requirement asks for FZI decomposition, and FZI absorbs all three by construction |
| 7 | A SCAL dataset with both porous-plate and mercury plugs | Dossier ESC-9's numeric default | `SB-SHR-037` fixes the *type* without it; only the default value waits |
| 8 | One licensed run of the incumbent's overburden-correction module | Dossier ESC-10 — whether the compiled code matches its printed manifest | `SB-SHR-038` refuses untagged imports either way, so this is a warning-accuracy question, not a blocker |

The dossier's six coverage gaps (`G-1`…`G-6`) are areas where **no claim is made in either
direction** — an unread compiled GUI module, unread workflow XMLs, compiled binaries, an unread
conditioning family, an unrendered help page, and held-back primary citations for one incumbent's
saturation-height families. They are recorded here so a later pass does not read silence as a
negative result. None of them blocks a requirement in §4.

### 7.6 Where this chapter is unsure

- **The Skelt-Harrison divergence figures in §3.1 are computed from the shipped defaults, not
  measured from a run.** The arithmetic is elementary and the branch has no unit conversion, so the
  direction and rough magnitude are not in doubt — but `SB-SHR-T01` should be written and run
  before the 47.7-saturation-unit figure is quoted outside this document.
- **The `PARTIAL` on `SB-SHR-017`** rests on reading four correlation sites; there may be more
  `log φ` regressions in `rocktyping.rs` beyond the ones inspected. The requirement is written to
  bind all of them regardless.
- **The claim that no fitted object is persisted anywhere** rests on a search of `db.rs` for a fit
  table and of `report.rs` / `export.rs` for any domain mnemonic, both of which came back empty.
  That is strong evidence but it is negative evidence.
- **`SB-SHR-039`'s class assignment is a judgement.** The dossier says this domain has no Tier-C
  items and it is right about the register; specifying the sweep under C-2 terms is a deliberate
  election to hold it to the stricter standard, not a claim that it is registered.

---

## 8. Traceability — dossier disposition

### 8.1 Chapter totals

| Artefact | Count |
|---|---|
| Requirements `SB-SHR-001` … `SB-SHR-042` | **42** (P0 13, P1 23, P2 6) |
| Acceptance tests `SB-SHR-T01` … `SB-SHR-T44` | **44** |
| Parameter rows (§5.1–§5.6) | **61** |
| Disposition rows (§8.2–§8.10) | **79** (14 inventory + 19 findings + 13 escalations + 6 gaps + 7 test groups + 3 parameter-table differences + 11 surplus + 6 not-carried) |
| `SB-CORE` requirements cited | 9 (`-001`, `-002`, `-003`, `-004`, `-006`, `-007`, `-010`, `-013`, `-014`) |
| New `SB-CORE` ids minted | **0** (one candidate raised in §7.1 for decision) |

### 8.2 Dossier item inventory, counted at the source

Counted directly from `sat-height-rocktyping.md` while writing this chapter, not taken from any
summary:

| Dossier section | Items | Count |
|---|---|---|
| §0 / §0.1 | Verification corrections `C-1` … `C-25` | 25 |
| §1.1–§1.4 | Method-inventory groups (IP module groups, Techlog shipped files, Geolog manifests, SandiBumi's own banks) | 4 |
| §2.1–§2.17 | Definition/equation comparisons | 17 |
| §3.1–§3.19 | Ranked differences that matter | 19 |
| §4 | Optimal-choice table, data rows | 40 |
| §4.1 | Ledger / OPEN-item disposition rows | 19 |
| §5.1 | Canonical equation forms | 1 block |
| §5.2 | Parameter registry, data rows | 56 |
| §5.3 | Validation and regression tests (numbered 1–24, including 14b and 14c) | 26 |
| §6.1–§6.2 | OPEN escalations (`ESC-1`, `2`, `4`–`14`; `ESC-3` closed in §6.4) | 13 |
| §6.3 | Coverage gaps `G-1` … `G-6` | 6 |
| §6.4 | Items closed during the dossier | 8 |
| §7.1–§7.8 | Source-register subsections | 8 |
| §8 | Critique-disposition findings (`B-1`, `M-1`–`M-6`, `m-1`–`m-7`), all fixed, none rebutted | 14 |

### 8.3 The 19 ranked findings — every one disposed

| Dossier | Finding | Disposition |
|---|---|---|
| 3.1 | Swanson apex volume basis — 10⁴ in k, BLOCKING | `SB-SHR-014`, `SB-SHR-032`; §5.5 `ABSENT` basis + `NON-ADOPTABLE` pair; as-built violation in §3.4; escalated `ESC-SHR-2` |
| 3.2 | Lucia porosity units — RFN 2.07 vs 7.41 | Resolved in SandiBumi's favour; recorded as defect refusal §7.3(d); guarded by `SB-SHR-T23` control |
| 3.3 | `RQI` namespace collision — 11.8× in `Swirr` | `SB-SHR-016`, `SB-SHR-T19`; seam stated in §1.2 |
| 3.4 | Brooks-Corey λ vs 1/N — 26 saturation units | `SB-SHR-012`, `SB-SHR-T14`; refusal §7.3(c) |
| 3.5 | An incumbent fits `Pc/σ` — every coefficient σ-scoped | `SB-SHR-019`, `SB-SHR-T22`; §5 `NON-ADOPTABLE` discipline; refusal §7.3(e) |
| 3.6 | Thomeer log base — 12 saturation units | `SB-SHR-013`, `SB-SHR-T15`; refusal §7.3(c); §5.1 ln-10 factor row |
| 3.7 | Inverted vs published forward Pittman regression — 2.0× in k | `SB-SHR-041`, `SB-SHR-T43` |
| 3.8 | Three inconsistent hydrocarbon gradients — 2.76× in height | `SB-SHR-018`, `SB-SHR-T21`; §5.1 `ABSENT` row |
| 3.9 | Missing gradient in crossplot height relation — 2.309× | `SB-SHR-006`, `SB-SHR-034`; refusal §7.3(l) |
| 3.10 | Contact-angle convention — sign flip on every mercury `Pc` | `SB-SHR-042`, `SB-SHR-T44`; depends on `SB-SHR-008` |
| 3.11 | `Swirr` is a method choice, not a parameter; one vendor disagrees with itself | `SB-SHR-037`, `SB-SHR-T39` |
| 3.12 | φE/φT ambiguity inside FZI — 1.63583, scale-free in k | §5.4 FZI-basis row; `SB-SHR-017`; the k-invariance is stated in §2.3 as the reason a cluster sweep cannot detect it |
| 3.13 | Closure default — Shift vs Proportional | `SB-SHR-027`, `SB-SHR-T29` — four named treatments, no default |
| 3.14 | Closure auto-pick depends on a plot checkbox | `SB-SHR-040`, `SB-SHR-T42`; refusal §7.3(j) |
| 3.15 | Closure `if/else` fall-through reports the wrong correction | `SB-SHR-027`, `SB-SHR-038`; refusal §7.3(k) |
| 3.16 | `BETA` unit mismatch — `GFT-1` vs `FT-1`, 1e9 | §5.5 `NON-ADOPTABLE` row; `SB-SHR-002` (declared units) |
| 3.17 | Two different quantities both called "R35" | `SB-SHR-033`, `SB-SHR-T35` |
| 3.18 | Rock typing is a partitioning concern in all three tools | **No requirement needed** — SandiBumi already separates it (`shf_fit.rs:957-983`); recorded `PRESENT-OK` at the end of §4.10 so a refactor cannot remove it |
| 3.19 | Corrections operate on the non-wetting phase | `SB-SHR-028`, `SB-SHR-T30` — structural, not documentary |

### 8.4 The 13 open escalations — every one disposed

| Dossier | Subject | Disposition in this chapter |
|---|---|---|
| ESC-1 | Hill-Shirley-Klein salinity unit: Kppm or ppm | **Carried open.** The CBW capability is `SB-SHR-028`; the unit question needs the rendered help page (`G-5`) and is an acquisition item |
| ESC-2 | Lucia Class 2 height exponent missing a decimal point | **Carried open, gates an optional method.** SandiBumi does not implement Lucia-`Swi`; §5.4 records the coefficients `PRESENT-UNVERIFIED` and §7.2 `ESC-SHR-4` requests the paper |
| ESC-4 | Porosity unit in an incumbent's MICP-permeability regressions | **Addressed on SandiBumi's side** by `SB-SHR-017` (enforced precondition) and `SB-SHR-032` (`ABSENT` until primaries acquired). The incumbent-side question stays open |
| ESC-5 | Three outputs with no equation anywhere on disk (`HI`, `PHIR`, `BETA`) | **Carried open.** Nothing is adopted from them; `BETA` appears in §5.5 as `NON-ADOPTABLE` only |
| ESC-6 | Port-size Macro/Meso boundary, 2.0 or 2.5 µm | **Closed by requirement** — `SB-SHR-015` makes both selectable with no default; `SB-SHR-T18` |
| ESC-7 | Swanson apex basis — **BLOCKING** | **Escalated for decision** (`ESC-SHR-2`) and specified — `SB-SHR-014` forbids the default; the as-built violation is §3.4 |
| ESC-8 | An incumbent's hyperbolic `Pc` form is not on disk | **Carried open as a coverage matter.** No requirement depends on it |
| ESC-9 | `Swirr` evaluation pressure — 200 psi lab or 100 psi reservoir | **Type fixed, default deferred** — `SB-SHR-037` makes the reference condition part of the parameter type; the numeric default stays `ABSENT` pending a mixed SCAL dataset (§7.5 #7) |
| ESC-10 | Four shipped-contract defects in printed correction formulae | **Refused** — §7.3(b), §7.3(k); `SB-SHR-038` refuses untagged corrected `Pc` imports either way |
| ESC-11 | Brooks-Corey intercept description self-inconsistent | **Closed by requirement** — `SB-SHR-012` requires a declared convention and refuses undeclared imports |
| ESC-12 | FWL uncertainty as a product requirement | **Specified at P0** — `SB-SHR-011`, `SB-SHR-T13`. The dossier holds it OPEN pending Jauhar; §7.2 `ESC-SHR-5` records that the requirement is written and the priority is his call |
| ESC-13 | An incumbent's implied 0.43352 vs its documented 0.433 | **Moot for adoption** — `SB-SHR-006` derives the gradient from first principles, which is not a claim about anyone's binary. Recorded, not pursued |
| ESC-14 | `Fs`, `τ`, `Sgv` never given values or ranges | **Moot** — no requirement decomposes FZI; §7.5 #6 records why |

**Reconciliation: 13 in, 13 out** — 3 closed by requirement (ESC-6, 11, and ESC-12 as specified),
2 escalated for decision (ESC-7, ESC-2), 2 rendered moot by independent derivation (ESC-13, 14),
2 addressed on SandiBumi's side with the incumbent-side question left open (ESC-4, ESC-9),
1 refused (ESC-10), 3 carried open as acquisition or coverage matters (ESC-1, 5, 8).

### 8.5 The 6 coverage gaps

`G-1` (compiled saturation-height GUI module), `G-2` (a second vendor's GUI-side capability),
`G-3` (compiled binaries behind the manifests), `G-4` (the supporting SCAL conditioning family),
`G-5` (an unrendered clay-bound-water help page), `G-6` (held-back primary citations for one
incumbent's SHF families) are all carried forward unchanged into §7.5's closing paragraph. **None
blocks a requirement in §4**, and none is treated as a negative result. `G-5` is the only one that
gates an open escalation (ESC-1).

### 8.6 The 26 dossier tests

All 26 are represented in §6. The mapping is not one-to-one and the reason is stated rather than
smoothed:

| Dossier tests | §6 coverage |
|---|---|
| 1, 2 (constant derivation, gradient round-trip) | `SB-SHR-T07` — including the dossier's own strengthening: assert to 8 significant figures, not 4, and assert the two derivation routes agree to better than 1e-6 |
| 3–6 (unit invariance, conventions) | `SB-SHR-T01`–`T03`, `T14`, `T15` |
| 7–11 (rock-typing indicators, namespaces) | `SB-SHR-T19`, `T20`, `T23`, `T24`, `T35` |
| 12–14, 14b, 14c (Swanson, apex, closure) | `SB-SHR-T16`, `T17`, `T29` |
| 15–18 (partitioning, heterogeneity) | `SB-SHR-T32`, `T33` |
| 19–21 (import safety, provenance) | `SB-SHR-T10`–`T12`, `T40` |
| 22–24 (corrections, reference conditions) | `SB-SHR-T28`, `T30`, `T39` |

**Discrepancy, stated:** the dossier specifies **26** tests; this chapter specifies **44**. The
18-test surplus is not padding — it is the as-built dimension the dossier did not have. The
dossier tests the *adoption spec* (are the constants right, are the conventions declared); §6
additionally tests **defects found in the shipped code while writing this chapter**: the
Skelt-branch unit blindness (`T01` parameterised over families), the `ρhc` round trip (`T06`), the
duplicated-constant sweep including test fixtures (`T08`), the undeclared-unit refusal (`T04`), the
unit-label surfaces (`T05`), the module exclusion ledgers (`T25`), the classifier's conflated
unclassified case (`T26`) and the report methodology row (`T27`). **Seventeen of the 44 are marked
must fail today** (sixteen outright, one — `T20` — partially), which is the honest measure of how
much of this domain is specified but not yet built correctly.

### 8.7 §4's optimal-choice table and §4.1's ledger rows

The dossier's 40 optimal-choice rows and 19 ledger-disposition rows are the *inputs* to §4 and §5
of this chapter rather than items requiring separate disposition: each names a method, a unit
basis, a convention or a partitioner, and each is discharged either by a requirement in §4, a row
in §5, or an escalation in §7.2. The two that carry their own escalation numbers (`D-04` the RQI
namespace, `D-05`/`E-D2` the missing gradient) are in §8.3 at findings 3.3 and 3.9. **Discrepancy,
stated:** this chapter does **not** reproduce the optimal-choice table. Reproducing 40 rows of
"which tool's version wins" would duplicate the dossier without adding a requirement, and
`CONTRACT.md` §4 forbids restating a source to fill a shape.

### 8.8 §5.2's 56 parameter rows versus §5's 61

The two tables are **not comparable one-to-one**, and this is the largest bookkeeping discrepancy
in the chapter. Three differences, all deliberate:

1. **This chapter adds as-built rows the dossier did not table** — every shipped default with no
   source (`J_CONST = 0.21645`, `PSI_PER_FT_PER_SG = 0.433`, `RHO_HC = 0.8`, `HG_AIR_IFT = 367.0`,
   `SWH_A`/`SWH_B`, the four Skelt placeholders, `SWT_IRR`, the four `rt_cutoff` boundaries).
   These are the `SB-CORE-004` exposures the build gate has to find, and they belong in a
   requirements document even though they are not adoption candidates.
2. **This chapter demotes four rows the dossier tabled with values to `ABSENT`** — the reservoir
   and laboratory σ·cosθ set (§5.2), because their only citation on this machine is a barred `.par`
   file. This is the §7.2 `ESC-SHR-1` refusal, and it **costs the chapter four defaults**. It is
   recorded as a cost, not presented as rigour.
3. **This chapter does not re-table the Pittman coefficients.** The dossier's registry and
   `rocktyping.rs:322-337` each hold them; a third copy in a PRD is a third thing to get wrong, and
   the module's own history (a mis-transcribed subset producing an inverted radius family) is the
   evidence for that. §5.4 carries a single row pointing at the paper and the single-sited table.

### 8.9 Surplus requirements — no dossier antecedent

Eleven requirements originate in this chapter's own as-built verification pass rather than in the
dossier. They are enumerated separately so a reader can see exactly what the source sweep added:

1. **`SB-SHR-001`** — branch parity for the depth-unit conversion. The dossier's evidence is about
   *vendors*; this defect is SandiBumi's own, found at `satheight.rs:175`.
2. **`SB-SHR-002`** — length-dimensioned shape parameters must declare and re-express their unit.
3. **`SB-SHR-003`** — refusal on an undeclared depth unit at the point of height arithmetic.
4. **`SB-SHR-004`** — height-dimensioned outputs labelled in the project's unit.
5. **`SB-SHR-005`** — one fluid-property default shared by fit and forward apply. The 5.3-saturation
   -unit round trip is a SandiBumi-only defect.
6. **`SB-SHR-007`** (the test-fixture clause) — a fixture may not be synthesised from the literal it
   guards. Found at `ingest.rs:2313`.
7. **`SB-SHR-009`, `SB-SHR-010`** — fitted objects and their forward application. The dossier
   assumes a fitted law is an object; the code has no such thing.
8. **`SB-SHR-022`** — the exclusion ledger extended to the modules path.
9. **`SB-SHR-023`** — unclassified distinguished from lowest-class in `rt_cutoff`.
10. **`SB-SHR-024`** — an unverified constant must surface as a result-level flag.
11. **`SB-SHR-025`** — the report must name the saturation-height and rock-typing methods used.

`SB-SHR-039` (the deterministic model-selection sweep) is a twelfth item without a dossier
antecedent, but it originates in the `CONTRACT.md` §2.2 amendment rather than in the source sweep,
so it is listed here for completeness and specified in §7.4.

### 8.10 What was read and not carried

- **The dossier's 25 verification corrections (`C-1`…`C-25`)** are corrections to the dossier's own
  earlier drafts. They are not restated here; they are the reason the numbers in §2 can be relied
  on. Two of them changed conclusions this chapter depends on and are honoured accordingly: the
  retraction of the "imperial and SI gradients differ by 0.0025 %" claim (they agree to
  0.0000589 %), and the resolution of the Lucia porosity basis in favour of decimals.
- **The 17 definition comparisons (§2.1–§2.17)** are the evidence behind §2 of this chapter. Each
  is either quoted into §2, cited in §5, or set aside as not requirement-bearing.
- **The 4 method-inventory groups (§1.1–§1.4)** are summarised in §2.4 and §2.5 only where a
  capability difference generates a requirement. A full inventory of 55+ manifests, 9 files and 6
  module groups is not restated.
- **The 8 items closed during the dossier (§6.4)** are not re-escalated, per `CONTRACT.md` §4.
- **The 14 critique-disposition findings (§8 of the dossier)** — `B-1`, `M-1`–`M-6`, `m-1`–`m-7`,
  all fixed, none rebutted — are treated as **authoritative over the dossier body they correct**,
  per `CONTRACT.md` §4. No `*_critique.md` file was read.
- **The 8 source-register subsections (§7.1–§7.8)** back the tier assignments in §2 and §5. §7.8's
  record of which files were and were not written is the basis for §7.5's acquisition gaps.

### 8.11 Coverage statement

Every numbered item in the dossier's §3 (19), §6.1–§6.2 (13) and §6.3 (6) has an explicit
disposition above — **38 of 38**. The §5.3 tests (26) are mapped in §8.6 as groups rather than
individually, which is a stated grouping, not a gap. §0's corrections (25), §1's inventory groups
(4), §2's comparisons (17), §4's rows (40), §4.1's rows (19), §5.2's rows (56), §6.4's closed items
(8), §7's register subsections (8) and §8's critique findings (14) are disposed by class in §8.7,
§8.8 and §8.10 rather than row by row — also stated, and the reason given in each case.
