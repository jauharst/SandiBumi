# 11. Porosity — requirements

**Source dossier:** `docs/research_2026-08/cross_tool/porosity.md` (1,980 lines), including its
method inventory (§1), its equation comparison (§2), its "differences that matter" analysis (§3),
its ledger disposition (§4.1), its adoption spec (§5), its gaps/escalations and 14-item OPEN list
(§6), its source register (§7) and its authoritative `## Critique disposition` (§8, 25 findings).
**Evidence tiers held in this domain:** **T1p** (Poupon, Hoyle & Schmidt 1971 / SPE 2925, the
primary paper, PDF read directly), **T1** (Geolog V14 `phi_*.lls` / `pha_*.lls` / `*.info`
executable source; Techlog 2018.2 shipped `PorosityAndLithologyComputation.py`), **T3-eq**
(Techlog `Doc\concept\` equation GIF/PNG rasters rendered at 5–6× and transcribed), T2 (IP2018 /
IP2025 full-manual CHM ingest reports), T3 (Techlog `Doc\` pages and API docs, `ip_ingest` /
`techlog_ingest` catalogue JSON), T4 (memory atomics, petro-kb, project-kb delivered-study
records).
**Tier note.** **T1p outranks everything.** The domain has a primary source that all three vendors
cite and none reproduces exactly; where a vendor disagrees with Poupon 1971, the vendor is the
finding. `T3-eq` is the dossier's own refinement of CONTRACT §1.2's T3 — a vendor raster read
visually, but of a *published equation*, not a chart. It is carried unchanged, and it is the tier
that closed ESC-6: Techlog's equations were on disk the whole time, as images under `<h2>Equations</h2>`
headings containing no ASCII.
**Author date:** 2026-08-07.
**Requirements:** 62. **P0:** 17. **Parameters:** 74 (18 ship `ABSENT — ships with no default`).
**Acceptance tests:** 41. **Dossier items dispositioned in §8:** 260 of 260.

**Delegation statement.** This chapter ran entirely on the session model. No subagent was used at
any point — every equation, coefficient, endpoint, clamp and `file.rs:line` below was read in
session, per the standing rule that petrophysical parameters and method math are never delegated.

---

## 1. Scope and boundary

This chapter owns the **deterministic porosity layer**: single-log density, neutron and sonic
porosity; the two-log neutron-density and neutron-sonic crossplot porosities; the shale/clay
correction applied inside those transforms; the **total-porosity reconstruction** `PHIT = PHIE +
VSH·PHIT_SH` and the clay-bound-water term it consumes; the **light-hydrocarbon correction** (the
Poupon `A`/`B` factors, the hydrocarbon apparent electron density and hydrogen index, and the
fixed-point iteration that binds them); the **neutron excavation effect**; the **porosity limiting,
floor, bad-hole, coal and kill branches**; the **apparent-vs-effective output pair**; and the
per-sample **branch and limit flag stream** that records which of those fired.

It does **not** own the following. Each seam is named here rather than discovered at index time.

**Multi-mineral solver (`MIN`).** Total and effective porosity also emerge from the volumetric
solver — SandiMin's `multimin2.rs` solves porosity as a fluid volume and rebuilds `PHIE`/`PHIT`
from the clay bound-water constraint (`multimin2.rs:1664-1665` emits `{prefix}_PHIE` and
`{prefix}_PHIT`). None of the solver's requirements are duplicated here. **The seam is a typing
obligation, not a numeric one:** a deterministic-transform porosity and a solver porosity are
different estimates of the same quantity, produced under different assumptions, and must not
resolve to one another by mnemonic collision — see SB-POR-004. The one place the two touch
numerically is the clay/shale endpoint pair: `dry_clay_calc` (`multimin2.rs:676-720`) converts a
wet-clay reading to a dry-clay endpoint with `φ_clay = (ρ_dry − ρ_wet)/(ρ_dry − 1.0)`, which is
algebraically the same construction as this chapter's `PHIT_SH = (RHODSH − RHOSH)/(RHODSH − RHOW)`
with `RHOW` pinned to the literal 1.0. SB-POR-008 fixes the fluid term; `MIN` owns what the solver
does with the result.

**Clay and shale volume (`CLY`).** Every shale-corrected porosity in this chapter consumes `VSH`
or `VCL` as an input. `CLY` fixes what that volume *is* and what type it carries; this chapter
fixes what is done with it. `CLY`'s SB-CLY-044 raises the `φsh` term that `clsr_porosity_corrected`
needs and that `CLY` does not define — **this chapter defines it**, as `PHIT_SH` (SB-POR-008) for
the bound-water route and as the shale's apparent porosity `(RHOMA − RHOSH)/(RHOMA − RHOFL)` for
the subtraction route, and §2 F16 records that those are two physically different quantities that
vendors give one name. The `VSH`/`VCL` refusal contract (SB-POR-006) is stated here because the
equations that break are porosity equations; `CLY` owns producing a typed volume.

**Environmental corrections and log QC (`ENV`).** Borehole-corrected `RHOB`/`NPHI`/`DT`, the
neutron matrix-unit gate, and the bad-hole discriminator curve are produced by `ENV`. This chapter
owns only what porosity does with them: the limestone-matrix precondition on the N-D crossplot
(SB-POR-024), the bad-hole substitution branch (SB-POR-047) and the coal/kill branches
(SB-POR-048). SandiBumi's `nphimat` (`modules.rs:1507-1622`) sits on that seam — it is a `Prep`
module by its own `category` field (`modules.rs:1511`) and belongs to `ENV`, but the requirement
that N-D porosity *refuses* an unconverted neutron is this chapter's.

**Cutoffs, summation and Monte Carlo (`CUT`).** Techlog's `VSHlimit 0.3` and `PHImax 0.35` are
computation-scope and ceiling parameters and are carried here; the *net-pay* cutoff on porosity is
`CUT`'s. `VSILT = 1 − VCL − PHIE/PHIMAX` (IP) is a porosity-derived index with a vendor
do-not-trust warning attached and is raised here (SB-POR-046) only because it is wired directly to
`PHIMAX`; its display use is `PLT`'s.

**Thin-bed and laminated analysis (`TBD`).** Thomas-Stieber consumes `PHIT` and `VSH`. This
chapter's obligation stops at delivering a `PHIT` that is typed, flagged and provenance-carrying
enough for `TBD` to consume.

**Rock physics and fluid substitution (`RPH`).** The real-gas density used by SandiBumi's
`gascorr` (Standing pseudo-criticals + Papay z-factor, `modules.rs:1683-1699`) is a fluid-property
model and belongs to `RPH`. This chapter owns only the *porosity* consequences of the gas
correction — SB-POR-038.

**NMR (`NMR`).** Techlog's density-magnetic-resonance porosity (TL-5) and IP's NMR-derived
porosity are `NMR`'s. Nothing in this chapter depends on them.

**TOC and unconventional (`TOC`).** IP's organic-shale porosity chain (IP-14 — kerogen and
heavy-mineral terms on the D, N and D-N routes) is named here for inventory completeness and
allocated to `TOC`.

**Data import/export (`DIO`) and database (`DBM`).** LAS null discipline and vendor parameter-set
import are `DIO`'s. The mnemonic-family dictionary is `DBM`'s structure; SB-POR-004 raises a
porosity-domain requirement against it because **`curves.rs:21-37` has no porosity family at all**
— fourteen families are registered and not one of them is `PHIE`, `PHIT`, `PHIA` or `DPHI`.

---

## 2. What the incumbents do — the requirement-bearing findings

Twenty-four findings. Each generates at least one obligation in §4. Findings from the dossier that
generate no obligation are accounted for in §8, not padded into here.

### F1 — Sandstone Δt_matrix is a five-value family and one vendor's default is a carbonate number

**Tier T1 / T1′ / T2 / T3. Tools: all three, and two vendors disagree with themselves.**
Wyllie `φ = (Δt − Δtma)/(189 − Δtma)` at `Δt = 90 µs/ft` (dossier §3.6, values re-derived and
pinned there):

| Δtma | Witness | Tier | φ |
|---|---|---|---|
| 53.0 | Techlog `QM_MineralTable` quartz | T3 | **0.2721** |
| 55.0 | IP BLA `DT Matrix` sandstone; IP `MINDEF.PAR` quartz | T2 | 0.2612 |
| 55.5 | Geolog `phi_son.info` shipped `DT_MA` = 182.1 us/m | T1 | 0.2584 |
| 56.0 | IP PhiSw `Sonic Sand`; `PhiSw.hlp` | T2 | **0.2556** |
| 47.5 | Techlog Quanti `DTma` default, **lithology-agnostic** | T3 | **0.3004** |

The four sandstone values span **1.65 p.u. (6.5 % relative)**. Including Techlog's single
lithology-agnostic default the spread is **4.5 p.u.** — that number is a limestone-ish value
shipped as the one default for every lithology, and it is the most dangerous number in the domain
for a clastic province.

The provenance error matters more than the spread. The dossier's earlier sittings cited Geolog's
"Silica 56.0" as corroboration for a **Wyllie** value of 56.0. `phi_son.lls` L80-85 scopes that
table verbatim: *"For **both the FLD_OBSA and FLD_OBSB**…"* (T1). Geolog's Wyllie inherits the
module-level `DT_MA` = **55.5**. So the honest reading is **IP-Wyllie 56.0 vs Geolog-Wyllie 55.5**
— a real 0.5 µs/ft two-vendor disagreement that a mis-shelved citation had been concealing as an
agreement. Geolog further ships Δtma and `EXP_AFF` as a **matched pair per mineral** (55.5/1.60,
47.6/1.76, 43.5/2.00) where Techlog ships one of each, unpaired.

→ SB-POR-016, SB-POR-055.

### F2 — "Raiga-Clemenceau" and "field observation" are one NAME with two equations, worth 1.3–1.6 p.u.

**Tier T1 (Geolog code) vs T3-eq (Techlog images). Both vendors ship both method names.**
The transform agrees; the **shale convention** does not.

| | Geolog `phi_son` (T1, code) | Techlog Quanti (T3-eq) |
|---|---|---|
| shale-corrected Δt | `dtsr = (Δt − VSH·DT_SH)/(1 − VSH)`, then `MAX(dtsr, DT_MA)` — **normalise, floor** | `Δt_cc = Δt − Vsh·(Δsh − Δtma)` — **subtract, unfloored** |
| effective scaling | answer `× (1 − VSH)` | none; `φ_E` is already effective |

At `Vsh = 0.20, Δt = 90, Δtsh = 100, Δtma = 55.5` with every shared coefficient matched
(`CFO`/`coef_Field` = 0.67, `EXP_AFF`/`coef_R` = 1.60), Geolog's `dtsr` = **87.5** and Techlog's
`Δt_cc` = **81.1** — two "shale-corrected slownesses" **6.4 µs/ft apart on identical inputs**:

| Transform | Geolog code | Techlog published | Δ |
|---|---|---|---|
| Field-observed | **0.1960** | **0.2115** | **1.55 p.u.** |
| Raiga / AFF | **0.1981** | **0.2111** | **1.30 p.u.** |

That is the same order as F1's Δtma spread, arising from nothing but the phrase "shale correction"
meaning two different operations. **The control is the more useful half:** on **Wyllie** both
vendors are algebraically identical (Geolog `phi_son.lls` L238-239 is a `~` line continuation
carrying the subtractive shale term) and both return **0.1917604**. So the divergence is localised
exactly to the two *nonlinear* branches, where Geolog switched to normalise-then-rescale and
Techlog kept one subtractive pre-correction for all four.

→ SB-POR-013, SB-POR-015, SB-POR-014.

### F3 — "Raymer-Hunt" is three different closed forms in three tools, and SandiBumi ships a fourth thing under that name

**Tier T2 / T1 / T3-eq.**
- **IP** (`embim53`, T2): a root in **velocity** —
  `[(2Vma − Vfc) − sqrt((2Vma − Vfc)² − 4Vma(Vma − Vlog))]/(2Vma)`, with the clay term evaluated as
  the *same* root at the clay point and subtracted, `Phi = Phi_son − Phi_clay·Vcl`.
- **Geolog `FLD_OBSA`** (T1): **Newton-Raphson** on `V_LOG = V_MA(1−φ)^C_EXP + V_FL·φ`, 20
  iterations, seed 0.15, tol 1e-5, and `phi_son.lls` L276-284 carries a **commented-out closed form
  it replaced** — Geolog moved from a closed form to iteration, the opposite direction from IP.
- **Techlog** (T3-eq): a root in **slowness** — `C = Δtma/(2Δtf)`,
  `φ_E = 1 − C − sqrt(C² − Δtma/Δtf + Δtma/Δtcc)`.

Three vendors, three renderings of one method name, none previously compared. IP recommends
Raymer over Wyllie verbatim (*"…automatically takes care of the problem of unconsolidated sands,
whilst the Wyllie equation has an extra compaction factor parameter which has to be estimated"*,
`basicloganalysis.htm`, T2); Techlog's and Geolog's defaults are both Wyllie.

**SandiBumi's `phi_son` "RHG" branch is none of these** — see §3.

→ SB-POR-014, SB-POR-020.

### F4 — Geolog's published doc block omits the shale reduction its own code performs, worth up to 6.1 p.u.

**Tier T1 (code is the authority; the doc block is the only part a Geolog user normally sees).**
`phi_son.lls` documents FLD_OBSA/FLD_OBSB/AFF on the **raw** transit time (L66, L72, L89) while the
executed code (L270-273, L321-325, L343-347) uses the **shale-reduced, matrix-floored** `dtsr` and
then multiplies by `(1 − VSH)`. At `Vsh = 0.20, Δt = 90, Δtsh = 100, Δtma = 55.5`:

| Transform | code (`dtsr`, then ×(1−VSH)) | doc form ×(1−VSH) | doc form as literally written |
|---|---|---|---|
| FLD_OBSB, `CFO` 0.67 | **0.1960** | 0.2055 (+0.95 p.u.) | 0.2568 (**+6.1 p.u.**) |
| AFF, `EXP_AFF` 1.60 | **0.1981** | 0.2086 (+1.05 p.u.) | 0.2608 (**+6.3 p.u.**) |

All three non-Wyllie branches share one shale discipline; the doc block states none of it. This is
the FINDINGS §6 rule-10 class (stale hand-maintained prose), and it is why every equation adopted
in §4 is transcribed from code or from a rendered vendor equation, never from vendor prose.

→ SB-POR-015, SB-POR-T09, SB-POR-T10.

### F5 — Techlog's shale-corrected slowness has no floor; Geolog added one in July 1997

**Tier T3-eq vs T1. Not a dispute — a fix one vendor made and the other did not.**
Techlog publishes `Δt_cc = Δt − V_sh(Δ_sh − Δt_ma)` with no clamp. Geolog floors the equivalent
quantity at three separate sites (`phi_son.lls` L271, L322, L344: `dtsr = MAX(dtsr, DT_MA)`) and
dates the change in its own history block (L6: *"Jul 1997 (WWC) Lower limit of dtsr set to DT_MA"*).

Unfloored, `Δt_cc` falls below `Δtma` whenever `V_sh > (Δt − Δtma)/(Δtsh − Δtma)` — at
`Δt = 70, Δtsh = 130, Δtma = 55.5` that is `V_sh = 0.195`, well inside ordinary shaly-sand range.
Below the floor Wyllie returns **negative** porosity and Raiga's `1 − (Δtma/Δtcc)^(1/coef_R)` goes
negative with the ratio inverted. Techlog's Raymer discriminant `C² − Δtma/Δtf + Δtma/Δtcc` also
moves the wrong way.

→ SB-POR-018.

### F6 — Techlog's neutron-sonic shale porosity divides by the log reading and returns 423 %

**Tier T3-eq, rendered at 6×. A dimensional error, not a modelling choice.**
`petrophysics-porosity-from-neutronsonic.html` publishes `φ_sh = (ΔT_shale − 47.6)/(ΔT − 47.6)`.
Every other shale-porosity form in the domain — Techlog's own sonic page
(`φ_tsh = (Δsh − Δtma)/(Δtf − Δtma)`), Geolog's `phi_son`, IP's clay term — divides by a
fluid-minus-matrix span. This one divides by the sample's own transit time, so `φ_sh` becomes a
function of the log at every depth and **diverges as `Δt → 47.6`**. In a fast clean sand
(`Δt = 60, Δtsh = 100`) it returns `52.4/12.4 = **4.23**` — a shale porosity of 423 p.u., which then
multiplies `VSH` in `φ_E = φ_X − φ_sh·VSH`. **At `Vsh = 0.05` that alone removes 21 p.u.**

The same page hard-codes **47.7** in the `φ_S` numerator and **47.6** in its denominator, and 47.6
again in `φ_sh` — three literals, two values, neither equal to the same product's own `DTma`
parameter default of 47.5.

→ SB-POR-053, and the refusal in §7.

### F7 — Techlog's two published excavation renderings disagree by a factor of 220

**Tier T3-eq. Both images re-read at native resolution; the contradiction is vendor-side.**

- `image2043.jpg`: `0.2731·ρma^2.1·(1 − S_HC)·( 0.02·φ **+** φ^1.8·S_HC·(0.6493 + 0.2149·S_HC) )`
- `sqp-theory-excavation-effect-equation-1.png`: the same prefactors × `(0.02 × Phit × Phit^1.8)`
  × `SwH × (0.6493 + 0.2149·SwH)` — the two bracket terms **multiplied**, not added.

At `φ = 0.25, S_HC = 0.7509, ρma = 2.65`: additive **2.91 p.u.**, multiplied **0.0132 p.u.** —
**220×**. The additive form is structurally the classic `K(2φ²Sw + 0.04φ)(1 − Sw)`: its `0.02φ`
scaled by `0.2731·2.65^2.1 = 2.114` gives `0.042φ` against the classic `0.04φ`, and its `φ^1.8·S`
term scaled likewise gives `0.80·φ^1.8` against `2φ²Sw`, matching to 0.4 % at `φ = 0.2, S = 0.5`.
The multiplied rendering loses the low-porosity term entirely and is essentially zero. **The
additive form is correct; the multiplied rendering is a vendor typesetting defect.**

A *second* sign difference on the same pair — `NPHI_HCC = NPHI + ΔNPHI_EX` (correction direction)
versus `NPHIcomput = NPHI − Deltaexc` (forward-model direction) — is **not** a defect and must not
be "fixed". It binds SandiBumi, not the vendor: the two directions MUST be two named functions,
never one function with a sign flag.

→ SB-POR-039, SB-POR-005.

### F8 — The excavation lithology exponent: two independent implementations agree, one vendor module is 4× weaker

**Tier T2 / T3-eq. Ledger item D-09, advanced but not closed.**

| Implementation | ρ_ma sensitivity | at 2.71 | at 2.87 |
|---|---|---|---|
| IP PhiSw (`embim50`) | `(ρma/2.65)²` | **1.0458** | **1.1729** |
| Techlog Quanti (`image2043`) | `ρma^2.1` | **1.0481** | **1.1823** |
| IP SSM (`sand_silt_malay_model.htm`) | `sqrt(ρma/2.65)` | 1.0113 | 1.0407 |

IP PhiSw and Techlog agree to **0.8 % across the whole lithology range**; SSM's `sqrt` is a
**four-fold weaker** lithology sensitivity and is the outlier against two independent vendor
implementations. Scaled to dolomite at the reference case the term is **3.18 p.u.** (power 2) vs
**2.82 p.u.** (`sqrt`). The decisive check — the published lithology constants `K` for the
`K(2φ²Sw + 0.04φ)(1 − Sw)` form — is **held nowhere in this corpus** and needs Segesman & Liu
(1971) or Schlumberger *Log Interpretation Principles* 1969 Ch. 13. Two implementations agreeing
is strong but is not primary evidence.

→ SB-POR-039, ESC in §7.

### F9 — Three of the four vendor hydrocarbon closed forms go negative or unphysical in dry gas

**Tier T1 / T2 / T3-eq / T1p. This is the single most dangerous band in the domain.**

| Form | Source | Fails below |
|---|---|---|
| Geolog `phi_dnh` `α = 1.67ρ − 0.17` | `phi_dnh.lls` L831 (T1) | **negative below ρ_h = 0.1018 g/cc** |
| IP `Den Hc app [Modified]` `(5.5ρ(4−ρ) − 3)/(16 − 2.5ρ)` | `swparameters.htm` (T2) | **negative below ρ_h = 0.1414 g/cc** (roots 0.1414 and 3.8586) |
| Gaymard-Poupon `N_h = 0.15 + 0.2(0.9−ρ)²` | Geolog `phi_dh` + Techlog `image1762` | **exceeds methane's hydrogen mass fraction below ρ_h ≈ 0.188 g/cc** |

The third bound is stoichiometry, not a vendor number and not a petrophysical parameter: pure
methane's hydrogen mass fraction is `4 × 1.008 / 16.04 = 0.2514`, the maximum for any hydrocarbon,
and `N_h = 0.2514` solves at `ρ_h ≈ 0.188`. **Dry gas at shallow-to-moderate reservoir pressure
sits in that band routinely.** A negative apparent electron density is not merely wrong — in IP's
denominator `ρma − ρfl·Sxo − ρ_HyAp(1 − Sxo)` it *increases* the denominator and biases density
porosity **low**, in exactly the case where the correction matters most.

→ SB-POR-033, SB-POR-034.

### F10 — IP's own two hydrocarbon-density models differ by a factor 3.22, and its two modules use different ones

**Tier T2. An IP-internal split with no citation on either side.**
At `ρ_h = 0.20 g/cc`: `Rho_HyAp` = **0.2452** (Conventional) vs **0.0761** g/cc (Modified). In IP's
density denominator at `ρb = 2.20, ρma = 2.65, Sxo = 0.55`:

| Model | denominator | φ_density | Δ |
|---|---|---|---|
| Conventional | 2.65 − 0.55 − 0.45×0.245161 = 1.989677 | 0.226167 | — |
| Modified | 2.65 − 0.55 − 0.45×0.076129 = 2.065742 | 0.217834 | **−0.83 p.u.** |

**IP's SSM module uses Modified exclusively**, so an IP user who runs PhiSw and SSM on the same
interval gets two different density porosities from the same inputs. IP gives **no citation for
either** (IP2018 A §2.7, §17). Against the primary source Conventional is the correct one, with the
envelope stated honestly: it tracks the Gaymard-Poupon quadratic to better than 1.5 % for
`ρ_h ≥ 0.225 g/cc` and degrades monotonically to **−3.1 % at 0.10** — i.e. it fails precisely
inside F9's band.

→ SB-POR-029, SB-POR-034, ESC in §7.

### F11 — Geolog's neutron hydrocarbon factor is 1.51× the primary source's gas value

**Tier T1 vs T1p. Not a modelling choice — an un-propagated fix.**
`B = (ρmf(1−Pmf) − α)/(ρmf(1−Pmf))` with `ρmf(1−Pmf) = 0.98`:

| Source of α | α | B | `B·φ·Shr` | × Geolog's `E = 1.3` |
|---|---|---|---|---|
| Poupon Eq A-9 (gas) | 0.4400 | 0.5510 | 6.20 p.u. | 8.06 p.u. |
| Techlog `9ρN_h` | 0.4464 | 0.5445 | 6.13 p.u. | 7.96 p.u. |
| IP `NeuHyHI` | 0.4065 | 0.5853 | 6.58 p.u. | — |
| **Geolog `phi_dnh`** | **0.1640** | **0.8327** | **9.37 p.u.** | **12.18 p.u.** |

Geolog's neutron hydrocarbon correction is **1.51× Poupon's own gas equation**, a **+4.1 p.u.
over-correction of NPHI** before the crossplot; fed into an N-D crossplot that moves crossplot
porosity by roughly half as much — **~2 p.u. on a 25 p.u. sand, ~8 % relative on pore volume**.
The tell is inside Geolog itself: its sibling module `phi_dh` was upgraded to the Gaymard-Poupon
quadratic on the density side (the superseded `1.15 * RHO_HC` line is still there, commented out,
`phi_dh.lls` L844) and **`phi_dnh` never received the equivalent upgrade on the neutron side**.

→ SB-POR-030, SB-POR-034.

### F12 — The flushed-zone saturation exponent has opposite defaults in two tools and no range check can see it

**Tier T1 vs T3.** Same equation, `Sxo = Swe^exponent`. Geolog `phi_dnh.info` `SW_EXP` default
**0.2**; Techlog/IP `invasion factor` default **1**. At `Swe = 0.30` that is
`Sxo = 0.786` versus `Sxo = 0.300` — **a 0.49 difference in Sxo**, feeding `Shr = 1 − Sxo` in every
hydrocarbon correction in the domain. These are not a near-miss; they are opposite modelling
assumptions (heavy flushing vs none). A "same method" port between the two tools moves `PHIE`
through the whole iteration with no parameter ever appearing out of range.

→ SB-POR-035.

### F13 — All three vendors solve the neutron matrix problem with chart data SandiBumi may not copy — and one published analytic route exists

**Tier T1 / T3.** Geolog `phi_dn` calls compiled chart functions twice (fresh and salt) and
interpolates linearly on fluid density; Techlog's `porNeutronDensity` is compiled and
tool-keyed over 38 (doc) / 39 (script) tools; IP uses per-tool `.neu` tables. Techlog's own
published N-D algorithm brackets its core with two steps stated only as *"The conversion depends on
the tool, according to the matrix"* — vendor chart data, deliberately not read.

Against that, **Geolog `phi_dnbk` implements a fully analytic, chart-free crossplot** with a
published reference (Bateman, R.M. & Konen, C.E., *The Log Analyst*, Nov-Dec 1977), and **Techlog's
neutron-sonic algorithm is structurally the same transform** — a branch test on two logs selects a
pair of apparent-endpoint constants, one a plain number and one a `10^(linear)` expression, and the
answer is the two-point lever rule between them. Two vendors independently implementing the same
published 1977 family with different fitted constants is materially stronger support for adopting
the analytic route than a one-vendor case.

→ SB-POR-021, SB-POR-022, SB-POR-027.

### F14 — None of the three ships the arithmetic-average or RMS combination as a porosity method

**Tier T1 / T2 / T3 (absence, verified across all three inventories) + T4.**
The Gaymard RMS rule `φe = √(½φD² + ½φN²)` and the 2/3 rule `(2φD + φN)/3` appear in course
material and field practice (memory `reference_vsh_porosity_methods.md`, T4) and in **none** of the
three tools as a porosity method. All three do a proper iterative hydrocarbon-plus-excavation
solve. The correct product decision is to ship the rigorous route as the answer and the quick rules
only as explicitly labelled comparison curves — which is what IP's own caveat says of them
(*"they should not be used for anything other than this"*, IP2018 A §16 item 4, T2).

This finding is the one that most changes what SandiBumi must build; see §3.

→ SB-POR-021, SB-POR-023, SB-POR-057.

### F15 — Clay-endpoint and shale-endpoint volumes are different quantities, bridged by one parameter that has no default anywhere

**Tier T2 (the bridge is IP's, and only IP's).**
IP's density porosity subtracts `Vcl × (ρma − ρcl)` — a **wet-clay** endpoint. Geolog and Techlog
subtract `VSH × (ρma − ρsh)` — a **shale** endpoint. IP publishes **four** relations linking them
(IP2018 A §11, all ASCII, T2):

```
Rho Wet Clay = Rho Matrix + (( Rho Shale - Rho Matrix ) / CSR )
Neu Wet Clay = Neu Matrix + (( Neu Shale - Neu Matrix ) / CSR )
Son Wet Clay = Son Matrix + (( Son Shale - Son Matrix ) / CSR )
Vshale       = VWCL / CSR                       (clamped to a maximum of 1.0)
```

`CSR` is defined verbatim as *"the percentage of clay in 100% shale in decimals (v/v)"* and **IP
states no numeric default for it**. **Geolog and Techlog publish no equivalent parameter at all**,
so neither tool can round-trip between the two volume conventions — that absence is itself the
finding, and it is a SandiBumi capability rather than a port. A build that silently defaults `CSR`
to 1.0 compiles, plots, and is wrong in every shaly sand in the direction that flatters the
reservoir.

→ SB-POR-006, SB-POR-012.

### F16 — "PhiT" is not one quantity across the three tools

**Tier T1 / T2 / T3-eq.** Geolog and IP build total porosity as `PHIE + Vsh·PhiT_sh`, where
`PhiT_sh` is built from a **dry** endpoint (`RHO_DSH`, `Rho Dry Clay` 2.78). **Techlog's published
"Total porosity from density" is the uncorrected density porosity** `(ρma − ρB)/(ρma − ρf)` —
not `PHIE + Vsh·PhiT_sh` at all. A user moving a curve named `PhiT` between Techlog and
Geolog/IP is moving two different definitions.

Techlog *also* publishes a `phi_Tsh`, but it is the **shale-subtraction term**
`(ρma − ρsh)/(ρma − ρf)` — wet-shale, ρma-anchored, formed and consumed inside one expression —
not the clay-bound-water quantity. Same symbol shape, different physics. Consequently the choice
of fluid term inside the bound-water quantity is a **two-way** comparison, not three-way: **IP uses
`Rho_fl` (filtrate), Geolog uses `RHO_W` (formation water)**, and Geolog changed to `RHO_W`
deliberately in March 1997 (recorded in every `phi_*` history block, T1). Clay-bound water is
formation water. With fresh filtrate over salt formation water the two disagree; at
`ρ_dsh = 2.78, ρ_sh = 2.50, ρ_w = 1.10, ρ_fl = 1.00` they give **0.1667 vs 0.1573**.

→ SB-POR-008, SB-POR-004.

### F17 — IP's porosity floor value is attested at two magnitudes 10× apart, inside one manual

**Tier T2. Ledger item D-16 / OPEN-14, opened by the dossier's revision pass.**
Verbatim from `ip2018_chm_ingest\A_porosity_sw.md` §6: the Limits/Badhole bullet summary says PHIE
is set to **0.0001**; the `Phie Limit` parameter entry says **0.001**; `Vcl Limit` and PHIFLAG code
9 both say **0.0001**. *"Three statements, two numbers."* The quantity is the value PHIE is *set
to* when the floor binds — i.e. in every shale interval in every well — and it only bites in
tight/zero-porosity intervals, which is exactly where a net-pay cutoff sits. Not resolvable from
held evidence: IP's own manual contradicts itself.

→ SB-POR-045.

### F18 — IP is the only vendor that publishes a solver precedence, and its last resort modifies the input log

**Tier T2, verbatim.** IP's four "variable" flags *"operate in series and can all be active at one
depth"*: `Variable Hc Den` first; `Variable GD` only *"if the Hc Den is outside its limits"*;
`Variable Vcl` only *"if both are at their limits"*; and if all three are at their limits, *"the
neutron and/or density **input curve will be reduced** in order to resolve the solution… the
PHIFLAG curve will be set to **6** to indicate a reduction in density, and **7** … neutron."*

**A four-free-parameter N-D solve with no documented order is under-specified**, and Geolog varies
nothing while Techlog bounds `ρ_HC` and grain density without stating an order between them. IP
also documents a configuration trap verbatim: *"it is required to tick another of the 'Variable'
calculations along with 'Variable Sxo'"* — a variable-Sxo-only run is invalid by the vendor's own
statement and IP does not reject it.

→ SB-POR-051, SB-POR-052.

### F19 — Techlog's neutron-density convergence test is published as an equality, at a 1 p.u. tolerance

**Tier T3-eq.** `petrophysics-porosity-neutrondensity-crossplot.html` states the iteration
terminates when `φ_nd(n−1) − φ_nd(n) = 0.01`. Read literally that is a loop that never terminates
under floating point. It is evidently `≤ 0.01` mis-typeset — and **0.01 v/v = 1 p.u. is an order of
magnitude looser** than the 0.001 used by the same vendor's hydrocarbon loop, by IP PhiSw, and by
Geolog's `|Δρma| < 1 k/m3`. Techlog also ships two different iteration caps for the same loop:
**50** on the doc page and **`maxiterations = 10`** in the shipped script (L1705).

→ SB-POR-050.

### F20 — Techlog's excavation tool blacklist gates on tokens that do not resolve, and its tool enum cannot be split safely

**Tier T1 (script) vs T3 (API doc). Four defects in one string.**
`PorosityAndLithologyComputation.py` L162/L166 `CorrExcF`: *"Make excavation factor correction?
(default on, switch off for SNP, APLC, APSC and BPHI)"*.

1. **`APSC` matches nothing.** The APS family in the 39-entry enum is `Schlumberger APS-APLC` and
   `Schlumberger APS-FPLC`. There is no `APSC`, no `APS-SC`, no `APSC`-suffixed entry. That gate
   entry would never fire.
2. **`SNP` matches two entries** (`Schlumberger SNP`, `Gearhart SNP`) — ambiguous as written; a
   naive substring match is the only thing that accidentally gets it right.
3. **`BPHI` resolves only via a tool whose casing differs between artefacts** — `EcoScope` in the
   script, `Ecoscope` in `Doc\topic\pythonlib\porneutrondensity.html`. A string-equality lookup that
   crosses the two fails silently.
4. **The enum itself cannot be split on its own delimiter.** Techlog declares it as a
   `/`-delimited string and one tool name contains a slash: the doc writes `"Atlas CN 2418/2420"`,
   the script smuggles it as `Atlas CN 2418*-\/2420`. A naive `/`-split yields **40 fields, not
   39**, two of them fragments, corrupting **every positional index after entry 13**. The counts
   are **39** (script) and **37** (doc), neither of them the "38" both prior reports asserted.

The *principle* — excavation off for epithermal/array neutron tools — is real physics that IP and
Geolog silently ignore and is worth adopting. Shipping an unresolved vendor string as a hard gate
is not: it fails silently for one entry in four and unpredictably for another.

→ SB-POR-041.

### F21 — High-shale behaviour is three different behaviours, none of them a shared convention

**Tier T1 / T2 / T3.** Geolog: a **hard-coded `VSH >= 0.95`** in all six `phi_*` modules ⇒
`PHIE = 0`, `PHIT = PHIT_SH`, `MTH_PHI = 'SHALE'`. IP: a user parameter `Vcl Limit` ⇒
`Phie = 0.0001`, all Sw = 1.0, PHIFLAG 9. Techlog: `LimitPhi` defaults to **"Do Not Constrain
Porosity"** — off. Geolog's step at a hard-coded threshold creates a discontinuity in the answer;
IP's smooth roll-off `(PhiMax + ΔPhiMax)(1 − Vcl) × 10^(−10(Vcl − VclCutoff)^1.6)` does not, but
IP states **no numeric default for any of its three parameters**. Geolog's *ordering* — clamp
`PHIE`, then rebuild `PHIT = PHIE + VSH·PHIT_SH` — is the correct one because it guarantees
`PHIT ≥ PHIE` by construction.

IP's full limiter family is larger than the one ceiling: `Phie Limit` (rewrites PHIE),
`Phie Sw Limit` (identical trigger shape, **porosity untouched** — two limiters one letter apart
with different effects), `Swi Limit` (explicitly feeds back into PHIE), `Vcl Limit`,
`Force 100% Wet` (PHIFLAG 16 — *"no hydrocarbon corrections will be made to the porosity"*, the
only per-zone HC kill switch any vendor publishes), `Kill Logic` (arbitrary user predicate zeroing
porosity), and the `VSILT` index. Every one has **no numeric default**.

→ SB-POR-043, SB-POR-044, SB-POR-046.

### F22 — Two independent vendors write the density transform with both signs inverted

**Tier T1 / T3-eq. A cross-vendor porting trap, promoted from a curiosity.**
Geolog `por_from_rhob.lls` L58-69 writes `PHIE_DEN = (RHO_MIN − RHOB)/(RHO_MIN − RHO_FL)` and
`PHIE_WYLLIE = (DT_MIN − DT)/(DT_MIN − DT_FL)` — the sonic one inverted on **both** numerator and
denominator relative to the textbook form. Techlog's N-D crossplot page writes
`φ_d = (ρ_b − ρ_lim)/(ρ_mf − ρ_lim)` — also inverted on both. Both are algebraically identical to
the standard form; **a reader porting either line verbatim without noticing both flips ships a
sign error**. Geolog additionally renames the endpoint `RHO_MIN`/`DT_MIN` ("solid mineral mixture")
and declares units `K/M3` and `US/M`.

→ SB-POR-054, SB-POR-T22.

### F23 — Techlog contradicts itself on nine shipped values, and the pattern is now a finding in its own right

**Tier T1 (script) vs T3 (doc pages). OPEN-7, extended twice.**
Every Techlog quantity that appears in both the shipped script and the shipped doc has been checked
and **a majority disagree**: ρ_HC 0.7 vs 0.8 g/cc; Δt_HC 210 vs 265 µs/ft; ρ_shale 2.4 vs 2.5
g/cc; NPHI_shale 0.40 vs 0.45; filtrate salinity 30 ppk vs 100,000 ppm — plus a further pair **in
two units on a single doc page** (`Nacl` ppk default 0 alongside `Mud Salinity` ppm default
100000); iteration cap 10 vs 50; `HI_gas` default 0.3 against its own formula's 0.446 at
ρ_h = 0.20; **ρ_dolomite 2.87 (`QM_MineralTable`) vs 2.9 (N-D crossplot page)**; and the
**47.5 / 47.6 / 47.7** limestone-slowness family, two of whose members sit inside a single
equation. For Techlog specifically, *neither* artefact can be treated as authoritative alone.

Dolomite grain density now has **four** values across the corpus (IP 2.85 / Techlog 2.87 / Techlog
2.9 / SandiMin 2.847) and **two of them are Techlog's**. Filtrate salinity has **four** unit/value
combinations across the three tools.

→ SB-POR-055, and the `ABSENT — ships with no default` rows in §5.

### F24 — The only numeric lithology-kill thresholds any vendor publishes will zero real porosity

**Tier T3-eq.** Inside Techlog's N-D core algorithm: `if φ_n > φ_d, 2.91 ≤ ρ_b ≤ 3.5 and
φ_n ≤ 0.04 ⇒ computed porosity = 0`. Three hard-coded literals, no user parameter, and it is the
**only** one of the three vendors' lithology kills that ships numbers — IP states *"no numeric
default for any of the nine thresholds"* for its coal/salt/anhydrite test, and Geolog keeps the
threshold logic outside the module entirely (`OPT_COAL` consumes an externally-computed `COAL`
log). A tight carbonate that drifts into that box loses its porosity with no flag and no
parameter to move.

→ SB-POR-048, SB-POR-049.

---

## 3. SandiBumi as-built

Written from the source, not from the docs folder. Every claim below carries `file:line`.

### 3.0 Inventory

| Module | Entry | Category | Status |
|---|---|---|---|
| `phi_den` — Porosity from Density | `modules.rs:677` spec, `:712` body | Porosity | **PRESENT-OK** on the equation, **PRESENT-DIVERGENT** on three defaults |
| `phi_dn` — Porosity from Density-Neutron | `modules.rs:763` spec, `:794` body | Porosity | **PRESENT-DIVERGENT** — the combination rule is a field shortcut, not a vendor method |
| `phi_son` — Porosity from Sonic | `modules.rs:858` spec, `:884` body | Porosity | **PRESENT-DIVERGENT** — the second branch is mislabelled and the compaction default inverts |
| `phimax` — porosity ceiling from a compaction trend | `modules.rs:929` spec, `:960` body | Porosity | **PRESENT-OK**; no vendor equivalent |
| `midplot` — MID / UMAA-RHOMAA | `lithology.rs:143` spec, `:235` body | Lithology | **PARTIAL** — `OPT_PHIA = XPLOT` inherits the same average shortcut |
| `nphimat` — neutron matrix conversion | `modules.rs:1507` spec, `:1592` body | Prep | **PRESENT-OK**; the only chart-derived path, gated (`neutron_charts.rs:1-22`) |
| `gascorr` — iterative gas density correction | `modules.rs:1629` spec, `:1701` body | Prep | **PRESENT-OK** as a fluid model, **PARTIAL** as a porosity HC chain |
| `condflag` — data-conditioning flags | `modules.rs:1249` spec | Prep | **PRESENT-OK**, but **not consumed by any porosity module** |
| `badhole` | `modules.rs:1183` spec, `:1206` body | Prep | **PRESENT-OK**, same disconnection |
| `ssc` — sand-silt-clay | `ssc.rs:74` spec | SSC | **PRESENT-OK** on gas conditioning (fixed 2026-07-29) |
| `sspw` — SSC pore-water | `ssc.rs:354` spec, `:392` body | SSC | **PRESENT-DIVERGENT** — carries the superseded gas weight and three dead parameters |
| Multi-mineral solver porosity | `multimin2.rs:1664-1665` | Minerals | Owned by `MIN`; the seam is §1's |

**Absent, verified by exhaustive grep of `src-tauri/src`:** no excavation effect (no `excav`/`Segesman` token anywhere); no Gaymard-Poupon hydrocarbon quadratic; no `PHIFLAG` or `MTH_PHI` branch-flag stream of any kind; no `CSR` clay-shale ratio; no Raiga-Clemenceau / AFF; no Bateman-Konen **crossplot** — the `Bateman-Konen` tokens at `modules.rs:1937`, `modules.rs:1966` and `multimin2.rs:547` are the unrelated **Rw↔NaCl-salinity** relation, which is a different Bateman & Konen result and must not be mistaken for coverage of F13.

### 3.1 `phi_den` — the equation is right; three of its five densities are not

The transform at `modules.rs:740-741` is the cross-vendor-agreed form, and the `PHIE`-then-rebuild
ordering at `:743-747` is Geolog's correct ordering from F21 — `PHIE` is limited first and `PHIT` is
rebuilt from the limited value, so `PHIT ≥ PHIE` holds by construction. `PHIT_SH` is factored into
one shared helper (`modules.rs:705-710`) and uses **`RHO_W`, formation water** — the F16-correct
choice, matching Geolog's deliberate March-1997 change and not IP's filtrate. That is three
structural decisions already right.

The defaults are the problem.

| Parameter | Shipped | Line | Assessment |
|---|---|---|---|
| `RHO_MA` | **2.645** | `:687` | Matches no vendor. IP and Techlog both ship 2.65 for sandstone. **0.17 p.u. on PHIE** at ρb 2.20, VSH 0.20 (0.2528883 vs 0.2545455) |
| `RHO_SH` | **2.5** | `:688` | Uncited. Equal to Techlog's *script* value, where its *doc* says 2.4 (F23) |
| `RHO_DSH` | **2.65** | `:690` | **Matches no held source.** IP's `Rho Dry Clay` is 2.78, Techlog's dry-shale basis 2.85 |
| `RHO_FL` | 1.0 | `:689` | Defensible; the vendors agree on 1.0 for the fresh case |
| `RHO_W` | 1.0 | `:691` | Correct quantity, but 1.0 is a fresh-water value shipped as a universal default |

`RHO_DSH = 2.65` is the serious one. It makes the dry-shale grain density equal to the *sandstone
matrix* density, so `PHIT_SH = (2.65 − 2.5)/(2.65 − 1.0) = **0.090909**` where IP's 2.78 gives
**0.157303** and Techlog's 2.85 gives **0.189189** — a factor **1.73 low** against the nearest
vendor. Total porosity is `PHIE + VSH·PHIT_SH`, so at `VSH = 0.5` SandiBumi's `PHIT` is
**3.3 p.u. below** the IP-parameterised answer from identical logs. In a shale-rich deltaic
section that is the difference between a defensible total-porosity curve and one that silently
under-reports clay-bound water everywhere.

Two further as-built facts: `VSH >= 0.95` is **hard-coded** (`modules.rs:732`) exactly as Geolog
does it, inheriting F21's discontinuity with no parameter to move it; and the shale-reduced ceiling
`phie_max*(1.0 - v)` (`:742`) is IP's linear roll-off **without** IP's `10^(−10(Vcl−cutoff)^1.6)`
smoothing — and when it binds, `PHIE` is silently rewritten with nothing recording that it did.

### 3.2 `phi_dn` — the crossplot is F14's shortcut, and the doc string says otherwise

`modules.rs:826-834` shale-reduces both logs on **Geolog's normalise convention**
(`(r − v·ρ_sh)/(1 − v)`, then `×(1 − v)` at `:836`) — the F2-correct convention, with Geolog's own
clamps `[1.95, 3.0]` and `[−0.015, 0.40]` carried across. Then the combination is
`(phid + nphisr)/2.0` or `sqrt((phid² + nphisr²)/2.0)`.

**Those are exactly the two rules F14 establishes that none of the three vendors ships as a
porosity method.** The module's own doc string at `modules.rs:770-771` states the opposite:

> *"(Commercial suites use service-company chart lookups here; this is the standard analytic
> equivalent.)"*

An arithmetic average of density and neutron porosity is not an analytic equivalent of a
service-company crossplot chart, and neither is the RMS. **This overclaim is in shipped code**, and
under CONTRACT §5 an overclaim discovered by a customer costs more than the feature it defends.
There *is* a genuine chart-free analytic equivalent — Bateman-Konen (F13) — and SandiBumi does not
implement it.

Worked, at `RHOB 2.20, NPHI 0.30, VSH 0.20` on shipped defaults: `rhosr = 2.125`,
`nphisr = 0.2875`, `phid = 0.3161094`, average `phix = 0.3018047`, `PHIE_DN = **0.2414438**`. The
Bateman-Konen route on the same shale-reduced pair returns `PHIE = **0.2578699**` — **1.64 p.u.
higher**. And the ceiling binds: `phie_lim = 0.3 × 0.8 = 0.24`, so the delivered `PHIE` is
**0.24**, clamped, with no flag — widening the gap to **1.79 p.u.**

### 3.3 `phi_son` — a mislabelled branch and a compaction default that inverts the correction

**The Wyllie branch is exactly right.** `modules.rs:909` plus the shale term at `:915` reproduce
the form on which Geolog and Techlog agree byte-for-byte (F2's control), and at the dossier's
reference case with `DT_SH = 100` it returns **0.1917604** — the fixture value. Only the *default*
moves it: shipped `DT_SH = 90` (`modules.rs:875`) gives **0.2067416**, **+1.50 p.u.**, from an
uncited endpoint.

**The `RHG` branch is not Raymer-Hunt-Gardner.** `modules.rs:907` is
`0.625 * (d - dt_ma) / d`, and `modules.rs:864` names it *"RHG (Raymer-Hunt-Gardner)"*. Compare
F3: IP's RHG is a quadratic root in velocity, Geolog's is a 20-iteration Newton-Raphson solve, and
Techlog's is a root in slowness. `0.625·(Δt − Δtma)/Δt` is **none of them** — it is the
**field-observation transform**, structurally Geolog's `FLD_OBSB` (`CFO·(dtsr − DT_MA)/dtsr`) with
`CFO` fixed at 0.625, run on **raw `Δt`** rather than the shale-reduced `dtsr` Geolog's code
actually uses (F4), and paired with a **Wyllie** shale subtraction at `:915` rather than the
`×(1 − VSH)` scaling its parent transform requires. Three separate divergences in one line.

At the reference case (`Δt 90, Δtma 55.5, Δtsh 100, VSH 0.20`), all at the same coefficient 0.625
and matched endpoints:

| Route | PHIE | vs SandiBumi |
|---|---|---|
| **SandiBumi `RHG`** (`modules.rs:907` + `:915`) | **0.1729167** | — |
| Geolog `FLD_OBSB` convention (`dtsr` 87.5, ×(1−VSH)) | 0.1828571 | **+0.99 p.u.** |
| Techlog field-observation convention (`Δt_cc` 81.1) | 0.1972873 | **+2.44 p.u.** |

The label is the finding, not the arithmetic: an analyst who selects "Raymer-Hunt-Gardner" because
IP's manual recommends it over Wyllie (F3) gets a different published transform, mis-cited, with
neither vendor's shale discipline.

*(Corrected 2026-08-20 — DEC-017 executed. `OPT_SON` is now `WYLLIE | RHG80 | FIELD_OBSERVED`:
`RHG80` is the paper's own three-segment transform (constants per DEC-079's verification; the
low segment inverted as IP's printed quadratic root), `FIELD_OBSERVED` is the old branch under
its honest name with `CFO` a cited ABSENT parameter (Geolog 0.67 / Techlog 0.625 disclosed via
the registry), and both non-Wyllie branches now run Geolog's executed shale convention —
`dtsr = (Δt − VSH·DT_SH)/(1 − VSH)` floored at `DT_MA`, answer × (1 − VSH) — closing all three
divergences named above. The legacy `RHG` option value resolves to no method: a saved run is
re-pointed deliberately, never silently remapped. Pinned by
`rhg80_inverts_the_papers_three_segment_transform_on_each_segment`,
`field_observed_ships_geologs_executed_shale_convention_not_the_doc_block` — which pins this
section's own 0.1828571 reference value — and
`the_rhg_label_now_means_rhg_1980_and_the_old_approximation_answers_only_as_field_observed`.
SB-POR-020's vendor-rendering choice remains open per DEC-017's own words.)*

**The compaction correction is applied in the wrong direction at its own default.**
`modules.rs:904` computes `cp = DT_SH/100`, and `:909` divides Wyllie porosity by it. The module
doc at `:865-868` states the intent correctly — *"undercompacted shaly sands … read porosity high
… so the WYLLIE porosity is divided by Cp"* — and cites Hilchie at `:901`. Dividing by `Cp` only
*reduces* porosity when `Cp > 1`, i.e. `DT_SH > 100 µs/ft`, which is the undercompacted condition
the correction is for. **The shipped `DT_SH` default is 90** (`:875`), giving `Cp = 0.90`, and the
parameter range opens down to 60 (`Cp = 0.60`). At the shipped default with `OPT_CP = ON`:

| | `OPT_CP = OFF` | `OPT_CP = ON`, `DT_SH 90` | Δ |
|---|---|---|---|
| `PHIT_SON` | 0.2584270 | 0.2871411 | **+2.87 p.u.** |
| `PHIE_SON` | 0.2067416 | 0.2297129 | **+2.30 p.u.** |

The correction whose stated purpose is to remove excess porosity **adds 2.30 p.u.** at the values
the product ships, and the failure is silent — every number stays inside every declared range. The
correction is opt-in (`OPT_CP` defaults `OFF`, `:872`), which contains the blast radius but does not
remove it, and there is no guard tying `Cp` to `Cp ≥ 1`.

**Three modules, three limiting disciplines.** `phi_den` and `phi_dn` floor at `PHIE_FLOOR` and cap
at `phie_max·(1−VSH)`; `phi_son` floors at **0.0** and caps at **1.0** (`modules.rs:911, :916`) —
no `PHIE_MAX`, no `PHIE_FLOOR`, no shale reduction, and no `VSH ≥ 0.95` branch at all. `phi_son`
also tolerates a missing `VSH` and emits `PHIT_SON` alone (`:912`), where the other two skip the
sample entirely (`:723`, `:807`). Same domain, same product, three contracts.

### 3.4 Output mnemonics collide, and porosity has no family

`phi_den` writes `PHIE_DEN`/`PHIT_DEN` plus **`PHIE`/`PHIT`** (`modules.rs:750-755`); `phi_dn`
writes `PHIE_DN`/`PHIT_DN` plus **`PHIE`/`PHIT`** (`:846-851`). Running both on one well means the
second silently overwrites the first's `PHIE` and `PHIT` — the two canonical curves the whole
downstream (pay summary, cutoffs, Sw, Monte Carlo) reads. `phi_son` writes neither, using
`PHIE_SON`/`PHIT_SON` only, so the three modules do not even collide consistently.

Underneath, `curves.rs:21-37` registers fourteen mnemonic families — GR, SP, CALI, BS, RHOB, DRHO,
PEF, NPHI, DT, DTS, RES_DEEP, RES_MED, RES_SHAL, RXO — and **not one porosity family**. So `PHIE`,
`PHIT`, `PHIA`, `DPHI` and a vendor-supplied `PHIE` arriving by LAS all carry no type, cannot be
distinguished by provenance, and cannot be prevented from resolving to one another. That is the
seam where F16's "PhiT is not one quantity" becomes a live data-integrity defect rather than a
vendor observation: SandiBumi cannot currently represent the difference between a Techlog `PhiT`
(uncorrected density porosity) and a Geolog `PHIT` (`PHIE + VSH·PHIT_SH`) even if it wanted to.

`workflow.rs:916-917` compensates downstream by re-flooring any `PHIE` at `PHIE_FLOOR` before pay
summation, with a documented motivating case (`workflow.rs:898-905`: an imported vendor `PHIE`
reading slightly negative over a tight streak took HPV **more than 20 % below** the floored answer
while `RESERVOIR` and `PAY` stayed byte-identical). That fix is correct and well-reasoned, and it
is also evidence for the requirement: the defence exists because the type system does not.

### 3.5 `PHIE_FLOOR` silently resolves an unresolvable vendor contradiction

`modules.rs:335` declares `pub(crate) const PHIE_FLOOR: f64 = 0.001;` — a compile-time constant,
consumed at `modules.rs:734, :743, :819, :839` and again at `workflow.rs:917` and `:2105`. This is
one side of F17's 10× contradiction, chosen with no note that IP's own manual states both 0.001 and
0.0001 for the same quantity. A test at `modules.rs:3571` asserts `PHIE_FLOOR < 0.01` — i.e. the
codebase already understands the value is cutoff-adjacent — but not which value it should be, or
that the question is open.

### 3.6 `gascorr` and `nphimat` — sound, but on different parameters and outside the porosity path

`gascorr` (`modules.rs:1701`) is a genuine iterative correction: Standing pseudo-criticals with a
Papay z-factor (`modules.rs:1683-1699`), an Archie inner loop, `RHOB_GC = RHOB + PHIT·(1−SWT)·(RHO_FL
− GASDEN)`, capped at 20 passes with tolerance 1e-4, and — correctly — **non-converging samples stay
MISSING** rather than emitting a last iterate (`modules.rs:1766-1782`). That last choice is the
fail-loud discipline CONTRACT §5 asks for, already present.

Two defects. First, `gascorr` ships `RHO_MA = 2.65` while `phi_den`/`phi_dn`/`condflag` ship
**2.645** (`modules.rs:687`, `:775`, `:1280`) — and `gascorr`'s own doc instructs chaining it into
those modules. Two matrix densities for one rock in one workflow. Second, `gascorr` corrects the
**density log** for gas; it is not the F9/F10/F11 hydrocarbon architecture, which corrects both
density *and* neutron inside the porosity solve using `ρ_HyAp` and `HI_h` with an excavation term.
SandiBumi has the fluid-property half and none of the porosity half.

`nphimat` (`modules.rs:1592`, tables `:1580-1590`, interpolation `:1558-1577`) is the one chart-
derived path and is properly gated: `neutron_charts.rs:1-22` records generation by
`tools/chartdig/gen_por45.mjs` with hard validation (grid RMS, calcite identity, monotonicity, and
reproduction of a published worked example). It is a `Prep` module (`modules.rs:1511`), so nothing
in `phi_dn` requires that it has been run — F13's and `condflag`'s matrix-unit trap is documented in
`condflag`'s doc string (`modules.rs:1261-1264`, verbatim: limestone-unit neutron against a
sandstone `RHO_MA` reads *"about 0.04 low in clean water sand"*) and **nowhere in `phi_dn`**, which
is the module that actually crossplots the two.

### 3.7 `condflag` and `badhole` exist and porosity ignores them

`condflag` (`modules.rs:1249-1308`) produces `COAL_FLAG`, `TIGHT_FLAG`, `XOVER_FLAG`,
`SHOULDER_FLAG` and a combined `COND_FLAG`, with parameterised thresholds, bed-thickness spike
rejection, shoulder widening, and a bad-hole exclusion so a washout is never called coal. Against
F24 this is **better than any vendor**: Techlog hard-codes three literals with no parameter, IP
states no numeric default for any of nine thresholds, and Geolog pushes the problem outside the
module. SandiBumi's version is parameterised, defaulted and bed-aware.

It is also **not wired to porosity**. No `phi_*` spec declares `COND_FLAG` or `BADHOLE` as an
input (`modules.rs:694-695`, `:783-785`, `:876-877`); the flag reaches porosity only if the user
remembers to set it as a generic Mask on each subsequent run, which the doc string asks for in
prose (`modules.rs:1272-1274`). The capability is built and the wiring is missing.

### 3.8 SSC / SSPW — an internal contradiction between two shipped porosity modules

`ssc()` conditions gas with the **RMS midpoint** `mid = sqrt((φD² + NPHI²)/2)` (`ssc.rs:181`), and
`ssc.rs:172-178` records why, in terms that are themselves the finding:

> the previous form *"weighted the pull by 1.6/2 = 0.8 per side, which overshoots the midpoint and
> **inverts** the D-N crossover (phid² became 0.2·φD²+0.8·φN²)"*

`sspw()` still runs that exact superseded form — `ssc.rs:433`:
`(phidi*phidi - 1.6*(phidi*phidi - np*np).abs()/2.0).max(0.0).sqrt()`, which expands to
`sqrt(0.2·φD² + 0.8·φN²)`. The fix of 2026-07-29 was applied to one of the two modules. At
`φD = 0.25, NPHI = 0.10` (an ordinary gas crossover) `ssc` returns **0.1903943** and `sspw` returns
**0.1431782** — **4.72 p.u. apart**, with `sspw` biased **low in gas**, which is the direction that
under-reports the pay. Both modules are shipped, both are in the SSC suite, and neither warns.

`sspw_spec` additionally declares `NPHI_MAT` (`ssc.rs:370`), `NPHI_SH` (`:372`) and `NPHI_FL`
(`:377`) as user parameters. `sspw()` (`ssc.rs:392-475`) never reads any of them — verified by
inspection of the whole body; the only neutron quantity used is the `NPHI` **log**, inside the gas
branch. `ssc.rs:37-41` states this honestly in the module header (*"read by the UI but unused by
the math"*), and the pending re-port against `sspw.lls` is correctly held for Jauhar's sign-off.
But the dialog still presents three tuneable endpoints that cannot change the answer — a user who
adjusts `NPHI_SH` to fit a shale and sees no change has been told a falsehood by the UI, and the
honest header is not visible from the dialog.

`sspw` also computes `PHIT_SH` from `RHOB_DSH 2.71` / `RHOB_SH 2.4` (`ssc.rs:354-390`) giving
**0.1812865**, against `phi_den`'s **0.0909091** — the same physical quantity, **a factor 1.99
apart**, in one product. At `VSH = 0.5` that is **4.5 p.u. of `PHIT`** determined purely by which
module the analyst opened. `RHOB_SH 2.4` is Techlog's *doc* value and `phi_den`'s `RHO_SH 2.5` is
Techlog's *script* value, so SandiBumi has shipped **both sides of F23's vendor-internal
contradiction simultaneously**, uncited, with no note that they disagree.

### 3.9 Frontend

The UI is generic: `src/ui/moduleDialog.ts` renders any `ModuleSpec` from its `args`, so every
parameter above is already user-editable and zone-overridable with no per-module UI code, and the
ribbon carries a "Porosity" group (`src/ui/ribbon.ts:584`). **Nothing in §4 requires new dialog
infrastructure** — new parameters, new options and new outputs appear automatically. What the
frontend cannot currently do is show provenance: a `ModuleSpec` parameter carries a name,
description, unit, default and range, and has **no field for a source citation or tier**, so the
`ABSENT — ships with no default` discipline of §5 has no place to live in the current struct.

---

## 4. Requirements

Sixty-two requirements. RFC-2119 verbs are used strictly per CONTRACT §1.4: **MUST** = a defect if
absent, **SHOULD** = strong default that may be traded with a recorded reason, **MAY** = optional.

### Group A — Architecture, typing and the seams (SB-POR-001 … 012)

**SB-POR-001** *(P1)* — SandiBumi **MUST** present all deterministic porosity methods through one
module family with one limiting contract, one flag contract and one output-naming contract.
Today three modules ship three of each (§3.3), which is a maintenance liability and a source of
answers that differ for reasons the analyst cannot see.

**SB-POR-002** *(P1)* — Every porosity method **MUST** emit an **unlimited** pair and a **limited**
pair under distinct mnemonics. `phi_den` and `phi_dn` already do (`modules.rs:750-755`, `:846-851`);
`phi_son` **MUST** be brought to the same contract. The unlimited curve is what a QC crossplot needs;
the limited curve is what pay summation needs, and conflating them hides every clamp.

**SB-POR-003** *(P0)* — Every porosity method **MUST** emit a per-sample **branch and limit flag**
stream (working name `PHIFLAG`) recording which branch produced the sample and which limit, if any,
bound it. IP is the only incumbent that publishes such a stream (F17, F21 — codes 6, 7, 9, 16), and
SandiBumi currently has **none** (§3.0). Every clamp identified in §3 — `VSH ≥ 0.95`, the
`phie_max·(1−VSH)` ceiling that binds at 0.24 in §3.2's worked case, the `[1.95, 3.0]` and
`[−0.015, 0.40]` shale-reduction clamps, `phi_son`'s `[0, 1]` — currently fires **silently**. This
is the single cheapest fail-loud-where-they-fail-silent win in the domain.

**SB-POR-004** *(P0)* — The mnemonic dictionary **MUST** carry a **porosity family** (`PHIE`, `PHIT`,
`PHIA`, `DPHI`, `NPHI_COR`, `PHIE_LIM`, …) and each porosity curve **MUST** carry the method and the
volume convention that produced it. Two porosity modules **MUST NOT** write the same output mnemonic:
`phi_den` and `phi_dn` both write `PHIE`/`PHIT` today, so the second run silently overwrites the first
(§3.4). `curves.rs:21-37` registers fourteen families and no porosity family at all. Without this,
F16's "PhiT is not one quantity" is unrepresentable and an imported Techlog `PhiT` resolves to a
computed Geolog-convention `PHIT` by name collision.

**SB-POR-005** *(P1)* — Where a quantity has both a **correction** direction and a **forward-model**
direction — pre-eminently excavation, `NPHI_corrected = NPHI + Δ` versus `NPHI_modelled = NPHI − Δ`
(F7) — SandiBumi **MUST** implement two separately named functions. It **MUST NOT** implement one
function with a sign flag. Techlog's two published forms differ by exactly this and are both correct
in their own context; a single sign-flagged function is the shape that eventually gets called with
the wrong flag.

**SB-POR-006** *(P0)* — Every porosity method that consumes a shale/clay volume **MUST** consume a
**typed** volume and **MUST** refuse an untyped one. A `VSH` (shale-endpoint) and a `VCL`
(wet-clay-endpoint) volume are not interchangeable (F15); the endpoint subtracted must match the
volume supplied. The refusal is the requirement — silently accepting either is how a 100 %-shale-point
correction gets applied to a clay volume.

**SB-POR-007** *(P1)* — `ModuleSpec` parameters **MUST** carry a **source citation and evidence tier**
alongside name, unit, default and range, and the dialog **MUST** surface them. The struct has no such
field today (§3.9), so §5's `ABSENT — ships with no default` discipline currently has nowhere to live
in the product — only in this document.

**SB-POR-008** *(P0)* — Clay-bound-water porosity **MUST** be defined once as
`PHIT_SH = (RHO_DSH − RHO_SH)/(RHO_DSH − RHO_W)` with `RHO_W` the **formation water** density, in one
shared helper, and **MUST** be exported to the `CLY` chapter's `clsr_porosity_corrected` (SB-CLY-044).
The shared helper exists and uses the correct fluid (`modules.rs:705-710`); the requirement pins it and
publishes it across the seam. The **shale-subtraction** term `(RHO_MA − RHO_SH)/(RHO_MA − RHO_FL)` is a
**different quantity** and **MUST NOT** share a name with it (F16).

**SB-POR-009** *(P1)* — `PHIT ≥ PHIE` **MUST** hold at every sample by construction — limit `PHIE`
first, then rebuild `PHIT` from the limited value (Geolog's ordering, F21). `phi_den:743-747` and
`phi_dn:839-843` already do this; the invariant **MUST** additionally be asserted, not merely relied on.

**SB-POR-010** *(P2)* — Every porosity curve **SHOULD** carry, in the project audit trail, the method
name, the full parameter set and the input curve identities that produced it, sufficient to re-derive
it without the session.

**SB-POR-011** *(P1)* — Matrix density **MUST** be a single shared parameter across modules that a
documented workflow chains. `gascorr` ships `RHO_MA 2.65` while `phi_den`, `phi_dn` and `condflag`
ship `2.645` (`modules.rs:1637`ish vs `:687`, `:775`, `:1280`) and `gascorr`'s own doc instructs
chaining them (§3.6).

**SB-POR-012** *(P2)* — SandiBumi **SHOULD** implement the `CSR` clay-shale-ratio bridge between the
`VSH` and `VCL` endpoint families (F15's four IP relations). Neither Geolog nor Techlog can round-trip
between the conventions at all, so this is a capability rather than a port. `CSR` **MUST** ship with no
default (§5) — silently defaulting it to 1.0 is wrong in every shaly sand in the flattering direction.

### Group B — Shale-correction conventions and the sonic family (SB-POR-013 … 020)

**SB-POR-013** *(P0)* — The **shale-correction convention** **MUST** be an explicit, named, per-method
selection — `NORMALISED` (Geolog: reduce, floor, then rescale by `1 − VSH`) or `SUBTRACTIVE` (Techlog:
one pre-correction, answer already effective) — and one method **MUST NOT** mix them. `phi_son`'s
`RHG` branch mixes them today: a normalise-convention transform paired with a Wyllie subtractive shale
term (`modules.rs:907` + `:915`, §3.3). The convention is worth **1.30–1.55 p.u.** across vendors on
identical inputs (F2) and is invisible in every parameter value.

**SB-POR-014** *(P0)* — Sonic porosity methods **MUST** be named for what they compute and **MUST NOT**
be named for a method they are not. Specifically: the branch at `modules.rs:907` **MUST** be renamed
from `RHG (Raymer-Hunt-Gardner)` to `FIELD_OBSERVED`, its coefficient 0.625 **MUST** become a
parameter, and any method offered as "Raymer-Hunt-Gardner" **MUST** be one of the three published
renderings in F3 with its vendor identified. Shipping IP's recommended-over-Wyllie method name against
a different transform is the kind of overclaim CONTRACT §5 warns costs the deal.

**SB-POR-015** *(P1)* — Non-Wyllie sonic branches **MUST** operate on the shale-reduced, matrix-floored
slowness and **MUST** rescale by `(1 − VSH)`, per Geolog's **executed code**, not its doc block. The
doc-block form differs by up to **6.3 p.u.** (F4). Wyllie **MUST** retain the subtractive form, on which
Geolog and Techlog agree exactly.

**SB-POR-016** *(P0)* — Matrix transit time **MUST** be selected per lithology from a cited family, and
SandiBumi **MUST NOT** ship a single lithology-agnostic default. Techlog's `DTma 47.5` applied to a
clastic section moves Wyllie porosity by **4.5 p.u.** against a sandstone value (F1). The sandstone
family itself spans 1.65 p.u. across four cited vendor values and therefore ships as a **cited choice
list, not a number** (§5).

**SB-POR-017** *(P1)* — The Wyllie lack-of-compaction correction **MUST** be guarded so it can only
reduce porosity: SandiBumi **MUST** require `Cp ≥ 1` (equivalently `DT_SH > 100 µs/ft`) and **MUST**
refuse or flag the sample otherwise. At the shipped `DT_SH = 90` the correction **adds 2.30 p.u. to
`PHIE`** — the opposite of its documented purpose (§3.3) — with every value inside every declared range.

**SB-POR-018** *(P1)* — Any shale-corrected slowness **MUST** be floored at `DT_MA` before use.
Unfloored, Wyllie returns negative porosity and Raiga inverts its ratio above
`V_sh = (Δt − Δtma)/(Δtsh − Δtma)` — `V_sh = 0.195` at ordinary values (F5). Geolog added this floor in
July 1997 and dates it in its own history block; Techlog publishes the equation without it.

**SB-POR-019** *(P2)* — Where a method requires a matrix endpoint **and** a fitted exponent, the two
**SHOULD** be selected as a **matched pair per mineral** (Geolog's 55.5/1.60, 47.6/1.76, 43.5/2.00),
not as two independent parameters (F1).

**SB-POR-020** *(P2)* — SandiBumi **SHOULD** implement exactly one Raymer-Hunt-Gardner rendering as the
default, cite which vendor's rendering it is, and **MAY** expose the other two as labelled comparison
methods. Three vendors ship three different closed forms under one name (F3); claiming "Raymer-Hunt-
Gardner" without saying whose is not a defensible claim.

### Group C — The neutron-density crossplot (SB-POR-021 … 028)

**SB-POR-021** *(P0)* — SandiBumi **MUST** implement a **chart-free analytic neutron-density crossplot**
as its primary N-D porosity method, following the Bateman & Konen (1977) family that Geolog's
`phi_dnbk` implements and that Techlog's neutron-sonic algorithm independently reproduces in structure
(F13). This is the method that lets SandiBumi ship a real crossplot porosity without transcribing a
single vendor chart value, and it is what §3.2's arithmetic average is standing in for at a cost of
**1.64–1.79 p.u.**

**SB-POR-022** *(P1)* — Chart-derived porosity paths **MUST** come only from SandiBumi's own gated
digitisation pipeline, with its validation gates enforced at build time. `nphimat`/`neutron_charts.rs`
is the pattern (§3.6); no vendor lookup table is transcribed, ever.

**SB-POR-023** *(P0)* — The arithmetic average `(φD + φN)/2` and the RMS `sqrt((φD² + φN²)/2)`
**MUST NOT** be presented as crossplot porosity methods, and the doc string at `modules.rs:770-771`
claiming they are *"the standard analytic equivalent"* of chart lookups **MUST** be removed. They
**MAY** ship as explicitly labelled quick-look comparison curves. **No vendor ships either as a porosity
method** (F14), and IP states of the field shortcuts verbatim that *"they should not be used for
anything other than this"*.

**SB-POR-024** *(P0)* — N-D crossplot porosity **MUST** refuse to run on a neutron curve whose **matrix
units** are not declared, and **MUST** state the declared basis in its output provenance. A
limestone-unit neutron against a sandstone matrix reads **~0.04 v/v low in clean water sand** — a fact
`condflag`'s doc string already states verbatim (`modules.rs:1261-1264`) and `phi_dn` neither states
nor checks (§3.6). All three vendors solve this with chart data; SandiBumi solves it with `nphimat` and
must then require it.

**SB-POR-025** *(P1)* — Where a method's endpoints depend on borehole fluid salinity, SandiBumi
**SHOULD** evaluate the fresh and salt cases and interpolate on fluid density, per Geolog's two-call
structure (F13).

**SB-POR-026** *(P2)* — Gas crossover **SHOULD** be detected and surfaced as a flag on the porosity
output. `condflag` already computes `XOVER_FLAG` (`modules.rs:1303`); the requirement is the wiring.

**SB-POR-027** *(P2)* — A **neutron-sonic** crossplot porosity **SHOULD** be offered, built on the same
two-point apparent-endpoint lever structure as SB-POR-021, and **MUST NOT** reproduce Techlog's
published `φ_sh` form (see SB-POR-053 and §7).

**SB-POR-028** *(P1)* — The shale-reduction clamps currently hard-coded at `modules.rs:826-827`
(`[1.95, 3.0]` g/cc and `[−0.015, 0.40]` v/v) **MUST** become cited parameters, and hitting them
**MUST** raise SB-POR-003's flag.

### Group D — Hydrocarbon correction (SB-POR-029 … 038)

**SB-POR-029** *(P0)* — The apparent hydrocarbon **electron density** **MUST** be the *Conventional*
form, and its validity envelope **MUST** be stated in the product: it tracks the Gaymard-Poupon
quadratic to better than **1.5 %** for `ρ_h ≥ 0.225 g/cc` and degrades monotonically to **−3.1 % at
0.10** (F10). IP's *Modified* form gives **0.0761 vs 0.2452 g/cc at ρ_h = 0.20** — a factor 3.22 —
and IP's own two modules disagree about which to use.

**SB-POR-030** *(P0)* — The hydrocarbon **hydrogen index** on the neutron side **MUST** be the
Gaymard-Poupon quadratic `N_h = 0.15 + 0.2(0.9 − ρ_h)²`, corroborated by Techlog's `9ρN_h` to
**1.2 %** and by Poupon's own Eq A-9 to **1.5 %** at gas density. Geolog's `α = 1.67ρ − 0.17` is
**1.51× Poupon's gas value** and over-corrects `NPHI` by **+4.1 p.u.** (F11); it is a fix Geolog made
on its density side and never propagated to its neutron side, and **MUST NOT** be adopted.

**SB-POR-031** *(P1)* — The hydrocarbon correction **SHOULD** be structured as Poupon 1971's `A`/`B`
factor architecture (`A` on the density side, `B` on the neutron side, both scaled by `φ·Shr`), which
is the primary source all three vendors cite and the only structure in which the vendors' variants can
be compared term by term.

**SB-POR-032** *(P2)* — Mud-filtrate density `ρmf` and filtrate hydrogen-loss `Pmf` **MUST** be
parameters, not literals. Poupon's `ρmf(1 − Pmf) = 0.98` is a *worked-example* value, not a default.

**SB-POR-033** *(P0)* — The hydrocarbon chain **MUST** refuse or hard-flag samples outside the validity
bounds of the selected model, specifically: `ρ_h < 0.1414 g/cc` (IP Modified goes negative),
`ρ_h < 0.1018 g/cc` (Geolog `α` goes negative), and `ρ_h < 0.188 g/cc` (any `N_h` exceeding methane's
hydrogen mass fraction `4 × 1.008 / 16.04 = 0.2514`, which is stoichiometry, not a parameter). **Dry gas
at shallow-to-moderate reservoir pressure sits inside that band routinely** (F9), and a negative
apparent density biases density porosity **low** exactly where the correction matters most. This is the
most consequential fail-loud requirement in the chapter.

**SB-POR-034** *(P1)* — Hydrocarbon model selection **MUST** be explicit and named by vendor, with all
variants available for cross-tool verification, and **MUST** be recorded in the output provenance. Four
vendor renderings exist for one physical quantity and three of them fail in gas.

**SB-POR-035** *(P0)* — The flushed-zone saturation exponent (`Sxo = Swe^n`) **MUST** ship with **no
default** and **MUST** be an explicit user decision. Geolog defaults **0.2** and Techlog/IP default
**1** — at `Swe = 0.30` that is `Sxo = 0.786` versus `0.300`, a **0.49 difference in `Sxo`** feeding
every hydrocarbon correction, with no parameter ever out of range (F12). These are opposite modelling
assumptions, not a tolerance.

**SB-POR-036** *(P2)* — A per-zone **force-100 %-wet** switch **SHOULD** be offered, suppressing all
hydrocarbon corrections to porosity and raising SB-POR-003's flag. IP's PHIFLAG 16 is the only such
switch any vendor publishes (F21).

**SB-POR-037** *(P2)* — The computed hydrogen index **SHOULD** be asserted against the stoichiometric
ceiling 0.2514 at every sample as a cheap internal consistency check.

**SB-POR-038** *(P1)* — The existing `gascorr` module **MUST** be documented as a **density-log
correction**, distinct from the porosity hydrocarbon chain, and the two **MUST NOT** be chained
without an explicit statement of which correction has already been applied — double-correcting is
otherwise invisible. `gascorr`'s non-convergence discipline (`modules.rs:1766-1782`, samples stay
MISSING) **MUST** be preserved and extended to the porosity chain.

### Group E — Excavation effect (SB-POR-039 … 042)

**SB-POR-039** *(P1)* — SandiBumi **MUST** implement the neutron excavation effect using the
**additive** rendering, `K·(0.02φ + φ^1.8·S_HC·(0.6493 + 0.2149·S_HC))·(1 − S_HC)` in Techlog's
parameterisation, with the lithology term as `ρma^2.1` or the equivalent `(ρma/2.65)²` — the two
independent implementations that agree to **0.8 %** across the lithology range (F8). Techlog's
multiplied rendering is a **typesetting defect** worth a factor **220** (F7) and **MUST NOT** be
implemented. IP SSM's `sqrt(ρma/2.65)` is a **four-fold weaker** lithology sensitivity and is the
outlier against two independent implementations. The term is **2.9–3.2 p.u.** at the reference case
and SandiBumi has none of it today (§3.0).

**SB-POR-040** *(P2)* — Excavation **MUST** be exposed in both directions as two named functions per
SB-POR-005.

**SB-POR-041** *(P2)* — Excavation **SHOULD** be suppressed for epithermal and array-neutron tools —
real physics that IP and Geolog silently ignore — but the gate **MUST** key on a **resolved tool
identity from SandiBumi's own tool register**, never on a vendor tool-name string. Techlog's gate
string contains a token matching nothing (`APSC`), a token matching two entries (`SNP`), a token
reachable only through a tool whose casing differs between its own two artefacts (`BPHI`/`EcoScope`),
and its enum cannot be split on its own delimiter without corrupting every index past the thirteenth
(F20). SandiBumi **MUST NOT** copy the string.

**SB-POR-042** *(P3)* — The published lithology constants `K` for the classic
`K(2φ²Sw + 0.04φ)(1 − Sw)` form **SHOULD** be obtained from Segesman & Liu (1971) or
*Log Interpretation Principles* (1969) Ch. 13 and used to adjudicate SB-POR-039's exponent. Until then
the exponent ships as a **cited choice between two agreeing implementations**, not as a settled value.

### Group F — Limits, branches and flags (SB-POR-043 … 049)

**SB-POR-043** *(P1)* — The high-shale kill threshold **MUST** be a cited parameter, not a literal.
`VSH >= 0.95` is hard-coded at `modules.rs:732` and `:817`, inherited from Geolog, and produces a
**step discontinuity** in `PHIE` at a value the analyst cannot move.

**SB-POR-044** *(P1)* — A **smooth** high-shale roll-off **SHOULD** be offered as an alternative to the
step, following IP's `(PhiMax + ΔPhiMax)(1 − Vcl)·10^(−10(Vcl − VclCutoff)^1.6)` shape. Its three
parameters ship with **no defaults** — IP publishes none (F21).

**SB-POR-045** *(P1)* — The value `PHIE` is **set to** when the floor binds **MUST** ship with no
default and **MUST** be a documented user decision. IP's own manual states **0.001** and **0.0001**
for the same quantity in three places (F17); SandiBumi hard-codes `0.001` at `modules.rs:335` with no
note that the question is open (§3.5). The quantity only bites in tight and zero-porosity intervals —
which is exactly where a net-pay cutoff sits.

**SB-POR-046** *(P2)* — If the `VSILT = 1 − VCL − PHIE/PHIMAX` index is offered, IP's own
do-not-trust warning **MUST** be surfaced with it.

**SB-POR-047** *(P1)* — Porosity methods **MUST** accept the existing `BADHOLE` flag
(`modules.rs:1183-1241`) as a declared input and **MUST** record its effect through SB-POR-003, rather
than depending on the analyst remembering to set a generic Mask (§3.7).

**SB-POR-048** *(P1)* — Porosity methods **MUST** consume `condflag`'s `COAL_FLAG`, `TIGHT_FLAG` and
`COND_FLAG` (`modules.rs:1301-1305`) as declared inputs with defined branch behaviour. SandiBumi's
conditioning module is **better than any incumbent's** on this point — parameterised, bed-thickness
aware, and bad-hole aware so a washout is never called coal — and it is currently not wired to the
modules that need it.

**SB-POR-049** *(P2)* — SandiBumi **MUST NOT** ship hard-coded lithology-kill literals. Techlog's
`φ_n > φ_d ∧ 2.91 ≤ ρ_b ≤ 3.5 ∧ φ_n ≤ 0.04 ⇒ φ = 0` is the only numeric kill any vendor publishes and
it will zero real porosity in a tight carbonate with no flag and no parameter (F24).

### Group G — Iteration and solver discipline (SB-POR-050 … 052)

**SB-POR-050** *(P1)* — Every iterative porosity solve **MUST** expose its convergence tolerance and
iteration cap as parameters, **MUST** state the tolerance as an inequality on the absolute change, and
**MUST** treat cap-exhaustion as non-convergence rather than emitting the last iterate. Techlog
publishes its N-D test as an **equality** at a **1 p.u.** tolerance — an order of magnitude looser than
its own hydrocarbon loop — and ships two different caps (10 in the script, 50 in the doc) for the same
loop (F19). `gascorr` already sets the correct precedent (`modules.rs:1766-1782`).

**SB-POR-051** *(P1)* — Where more than one unknown may be varied to reach a solution, the **precedence
MUST be documented and deterministic**. IP is the only vendor that publishes one — Hc density, then
grain density, then `Vcl`, then, as a last resort, **reducing the input log itself** under PHIFLAG 6/7
(F18). A four-free-parameter solve with no stated order is under-specified, and Geolog and Techlog
leave it so.

**SB-POR-052** *(P2)* — Invalid solver configurations **MUST** be rejected at configuration time.
IP documents verbatim that a variable-`Sxo` run requires another variable flag to be active, and then
does not enforce it (F18).

### Group H — Provenance, refusals and comparison (SB-POR-053 … 062)

**SB-POR-053** *(P1)* — Shale porosity in any crossplot **MUST** be formed as a fluid-minus-matrix
span. SandiBumi **MUST NOT** implement Techlog's published neutron-sonic
`φ_sh = (ΔT_shale − 47.6)/(ΔT − 47.6)`, which divides by the sample's own transit time, returns
**4.23** in a fast clean sand, and removes **21 p.u.** at `Vsh = 0.05` (F6). Where a rendered vendor
equation is dimensionally inconsistent with every sibling equation in the same product, the vendor
equation is the finding.

**SB-POR-054** *(P1)* — SandiBumi **MUST** state one canonical sign convention for every
matrix/fluid/log transform and **MUST** carry a test proving algebraic identity with the inverted
forms that Geolog (`por_from_rhob.lls`) and Techlog (N-D crossplot page) publish (F22). Two independent
vendors write these with both numerator and denominator inverted; a reader porting either line
verbatim without noticing both flips ships a sign error that is invisible in review.

**SB-POR-055** *(P0)* — Every petrophysical parameter in this domain **MUST** carry a source string and
tier, and where the held sources disagree with no defensible adjudication the parameter **MUST** ship
`ABSENT — ships with no default` with the competing values visible. This is a standing project
decision. It applies immediately to `RHO_SH`, `RHO_DSH`, `NPHI_SH`, `DT_SH` and `RHO_MA`, all of which
ship today as uncited numbers (§3.1) — and `RHO_DSH = 2.65` matches **no held source at all** while
setting `PHIT_SH` a factor **1.73 low** against the nearest vendor. For Techlog specifically, neither
its script nor its doc may be treated as authoritative alone: **nine** shipped quantities disagree
between the two, including two values inside one equation (F23).

**SB-POR-056** *(P2)* — Porosity **MUST** be carried internally in `v/v`, transit time in `µs/ft` and
density in `g/cc`, with display units a presentation concern. Geolog ships `K/M3` and `US/M`
internally (F22) and Techlog ships filtrate salinity in **four** unit/value combinations (F23); the
canonical-unit rule is what keeps an import from either from arriving 1000× out.

**SB-POR-057** *(P2)* — Quick-look comparison curves **MUST** be visually and structurally
distinguishable from computed methods — different mnemonic family, flagged in provenance, excluded by
default from pay summation.

**SB-POR-058** *(P0)* — A module **MUST NOT** present a parameter its computation does not read.
`sspw_spec` declares `NPHI_MAT`, `NPHI_SH` and `NPHI_FL` (`ssc.rs:370`, `:372`, `:377`) and `sspw()`
reads none of them (§3.8). Until the re-port against `sspw.lls` is signed off, those parameters
**MUST** be removed from the spec or marked inactive in the dialog. An honest module header
(`ssc.rs:37-41`) is invisible to `moduleDialog.ts`; a user who tunes `NPHI_SH` and sees no change has
been told a falsehood by the UI.

**SB-POR-059** *(P0)* — `sspw()`'s gas conditioning **MUST** be brought to the same RMS midpoint
`sqrt((φD² + NPHI²)/2)` that `ssc()` uses. `ssc.rs:433` still runs the weight that `ssc.rs:172-178`
records as *inverting the D-N crossover* and that was fixed in `ssc()` on 2026-07-29. At
`φD = 0.25, NPHI = 0.10` the two shipped modules return **0.1903943** and **0.1431782** — **4.72 p.u.
apart, with `sspw` biased low in gas**, the direction that under-reports pay.

**SB-POR-060** *(P2)* — SandiBumi **SHOULD** import vendor parameter sets (IP `.par`-style, Geolog
`.info` defaults, Techlog parameter decks) as **cited, tiered, read-only** parameter sets that populate
SB-POR-007's provenance rather than becoming SandiBumi defaults.

**SB-POR-061** *(P3)* — A porosity **method audit report** **SHOULD** be producible per well: every
method run, every parameter with its source and tier, every flag raised with its sample count, and
every limit that bound. This is the deliverable-defence artefact none of the three incumbents produces.

**SB-POR-062** *(P3)* — Core-porosity calibration **MAY** be offered as a post-check against computed
porosity, reporting bias and scatter per method, with no automatic adjustment of any parameter.

---

## 5. Parameters

Seventy-four rows: the dossier's §5.2 table (72) transcribed byte-exact on every value, plus two
model-validity rows this chapter adds. **`ABSENT — ships with no default` is a first-class state:
the compute core refuses to run rather than substituting.** `NON-ADOPTABLE — cited for verification`
means the value is recorded so a SandiBumi answer can be checked against a vendor's, and is never a
SandiBumi default.

Tier key as CONTRACT §1.2, with the dossier's refinements: **T1p** primary literature read directly;
**T1** vendor executable source (Geolog `.lls`/`.info`, Techlog shipped `.py`); **T2** vendor manual
(IP CHM ingest); **T3** vendor doc page or catalogue JSON; **T3-eq** vendor equation raster rendered
and transcribed; **T4** house/project record.

### 5.1 Matrix, fluid and shale densities

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Sandstone matrix density | ρma | 2.65 | g/cm3 | `ip_ingest/E_threeway_endpoint_compare.json` Quartz RHOB — IP MINDEF, Techlog `QM_MineralTable` and SandiMin all 2.65 (3-way AGREE) | T3 |
| Sandstone matrix density, Geolog shipped | ρma | 2.645 | g/cm3 | Geolog `phi_den.info` `RHO_MA` DEFAULT = 2645 k/m3 (shipped default, differs from the endpoint libraries) | T1 |
| Limestone matrix density | ρma | 2.71 | g/cm3 | same 3-way AGREE; also IP `swequationsandmethodology.htm` verbatim "2.71 100% lime" | T2/T3 |
| Dolomite matrix density | ρma | **ABSENT — ships with no default.** Attested: 2.85 (IP) / 2.87 (Techlog `QM_MineralTable`) / 2.9 (Techlog Quanti N-D page) / 2.847 (SandiMin) | g/cm3 | IP `swequationsandmethodology.htm` verbatim "2.85 100% Dolomite"; `E_threeway_endpoint_compare.json` Dolomite RHOB; `petrophysics-porosity-neutrondensity-crossplot.html` verbatim "ρ_dol: Dolomite grain density, default 2.9 g/cm³" | T2/T3/T3-eq |
| Grain density when unset (IP) | ρGD | 2.71 | g/cm3 | IP `swequationsandmethodology.htm` verbatim "If a Rho GD is not entered then it is assumed to be 2.71." | T2 |
| Grain-density lithology rule | — | 2.65 → 100 % sandstone; 2.71 → 100 % limestone; 2.85 → 100 % dolomite; linear between | g/cm3 | IP `swequationsandmethodology.htm` verbatim, incl. "an input Rho GD of 2.68 will result in 50% Sandstone and 50% Limestone." | T2 |
| Clastic case low/base/high | ρma | 2.62 / 2.65 / 2.68 | g/cm3 | `project-kb\records\tiara-bumi.md` §Parameters, MoM Progress Meeting III 2025-02-07 item B.7 | T4 |
| Carbonate case low/base/high | ρma | 2.71 / 2.84 / 2.87 | g/cm3 | same record, same item | T4 |
| Zoned carbonate matrix | ρma | 2.71 limestone → 2.85 dolostone, varied by zone | g/cm3 | `project-kb\records\kolibri-pepc.md` — precedent for zone-varying ρma | T4 |
| Fluid density, fresh | ρfl | 1.00 | g/cm3 | IP `basicloganalysis.htm` verbatim "Defaults to 1.0 gm/cc for fresh water"; Geolog `phi_den.info` `RHO_FL` = 1000 k/m3; Techlog `…effective-porosity-from-density.html` RHOB_fluid = 1 | T1/T2/T3 |
| Fluid density, salt | ρfl | 1.10 | g/cm3 | IP `basicloganalysis.htm` verbatim "Set to 1.1 gm/cc for salt water" | T2 |
| Formation water density | ρw | 1.00 | g/cm3 | Geolog `phi_den.info` `RHO_W` DEFAULT = 1000 k/m3, validation 500:2000 | T1 |
| Shale density | ρsh | **ABSENT — ships with no default.** Techlog attests 2.4 (doc) and 2.5 (script) | g/cm3 | IP `swparameters.htm` verbatim "A value must be entered if a density tool is selected"; Geolog `phi_den.info` DEFAULT = `RHO_SH` (a reference, not a number); Techlog `…effective-porosity-from-density.html` 2.4 vs `PorosityAndLithologyComputation.py` `DEN_shale` 2.5 | T1/T2/T3 |
| Dry shale / dry clay density | ρdsh | **ABSENT — ships with no default.** Attested: 2.78 (IP `Rho Dry Clay`) / 2.85 (Techlog `DEN_dryshale`) — **different quantities** | g/cm3 | IP2025 B §3.2 "Rho Dry Clay 2.78 — 2018 htm agrees"; Techlog script parameter block; Geolog `RHO_DSH` has no default (validation 2500:5000 k/m3) | T1/T2 |
| Shale neutron endpoint | φNsh | **ABSENT — ships with no default.** Attested: 0.40 (Techlog doc) / 0.45 (Techlog script) | v/v | `…effective-porosity-from-neutrondensity.html` NPHI_shale = 0.4; `PorosityAndLithologyComputation.py` `NEUT_shale` = 0.45; Geolog no numeric default | T1/T3 |
| **Wet-clay** neutron endpoint | φNcl | **ABSENT — ships with no default** | v/v | IP2018 `A_porosity_sw.md` §2.2, parameter `Neu Wet Clay`, "a value must be entered". **Not interchangeable with the shale endpoint** — bridged only by `CSR` | T2 |
| Clay/shale ratio | CSR | **ABSENT — ships with no default** | v/v | IP2018 `A_porosity_sw.md` §11. Four relations incl. verbatim `Vshale = VWCL/CSR (clamped to a maximum of 1.0)`. Geolog and Techlog publish no equivalent parameter at all | T2 |

**Note on `RHO_DSH`.** SandiBumi ships **2.65** (`modules.rs:690`), which matches neither attested
value and equals the sandstone matrix density. It **MUST** become `ABSENT` per SB-POR-055.

### 5.2 Sonic transit times and coefficients

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Sandstone Δtma, Wyllie | Δtma | **ABSENT — ships with no default.** Attested: 56.0 (IP) / 55.5 (Geolog) | µs/ft | IP `swparameters.htm` "Default 56", corroborated by `PhiSw.hlp`; Geolog `phi_son.info` `DT_MA` = 182.1 us/m ≡ 55.5 µs/ft, inherited by Wyllie | T1/T2 |
| Sandstone Δtma, AFF/Raiga | Δtma | 55.5 (paired `EXP_AFF` 1.60) | µs/ft | Geolog `phi_son.lls` L91-96 AFF table "Silica 55.5, EXP_AFF 1.60" | T1 |
| Sandstone Δtma, field-observed | Δtma | 56.0 | µs/ft | Geolog `phi_son.lls` L80-85, explicitly scoped to FLD_OBSA **and** FLD_OBSB | T1 |
| Limestone Δtma, Wyllie | Δtma | 49.0 (IP only) | µs/ft | IP `swparameters.htm` "Sonic Lime Default 49" | T2 |
| Limestone Δtma, AFF/Raiga | Δtma | 47.6 (paired `EXP_AFF` 1.76) | µs/ft | Geolog `phi_son.lls` AFF table "Calcite 47.6, EXP_AFF 1.76" | T1 |
| Limestone Δtma, field-observed | Δtma | 49.0 | µs/ft | Geolog `phi_son.lls` L80-85 FLD_OBSA/B table | T1 |
| Dolomite Δtma, Wyllie | Δtma | 44.0 (IP only) | µs/ft | IP `swparameters.htm` "Sonic Dol Default 44" | T2 |
| Dolomite Δtma, AFF/Raiga | Δtma | 43.5 (paired `EXP_AFF` 2.00) | µs/ft | Geolog `phi_son.lls` AFF table "Dolomite 43.5, EXP_AFF 2.00" | T1 |
| Dolomite Δtma, field-observed | Δtma | 44.0 | µs/ft | Geolog `phi_son.lls` L80-85 FLD_OBSA/B table | T1 |
| Neutron-sonic hard-coded limestone Δt | — | **NON-ADOPTABLE — cited for verification.** 47.7 (φ_S numerator) / 47.6 (φ_S denominator and φ_sh, both places) | µs/ft | Techlog `petrophysics-porosity-from-neutronsonic.html`, rendered at 6×. **Two literals differ by 0.1 inside one equation**; neither is a parameter | T3-eq |
| Shale-correction convention | — | **ABSENT — the user picks; the pick is written into the run header.** `GEOLOG_NORMALISED` \| `TECHLOG_SUBTRACTIVE` | enum | Geolog `phi_son.lls` L303 `PHIE_SON = phi_tmp*(1-VSH)`; Techlog `…effective-porosity-from-sonic.html` `Δt_cc = Δt − V_sh*(Δ_sh − Δt_ma)` | T1/T3-eq |
| Shale porosity, neutron-sonic | φsh | `(DTSH − DTMA)/(DTFL − DTMA)` | v/v | SandiBumi canonical form, matching Techlog's own sonic page and Geolog. Techlog's neutron-sonic page publishes `(ΔT_shale − 47.6)/(ΔT − 47.6)` and that form is **recorded as a defect, not an option** | T3-eq |
| Fluid transit time | Δtfl | 189 | µs/ft | IP `swparameters.htm` "Sonic water Default 189"; Geolog `phi_son.info` `DT_FL` = 620 us/m; Techlog `…from-sonic.html` = 189 | T1/T2/T3 |
| Fluid transit time, salt-saturated | Δtfl | ≈174 | µs/ft | IP `basicloganalysis.htm` verbatim "For salt-saturated formation water use about 174 usec/ft" | T2 |
| Shale transit time | Δtsh | **ABSENT — ships with no default.** Techlog attests 100; IP and Geolog state none | µs/ft | Techlog `PorosityAndLithologyComputation.py` `DTshale` = 100; Geolog validation 150:600 us/m, no numeric default; IP "A value must be entered" | T1/T2 |
| Hydrocarbon transit time | Δthc | **ABSENT — ships with no default.** Attested: 265 (Quanti doc) / 210 (Techlog script) | µs/ft | `…petrophysics-hydrocarbon-correction.html` "gas (265 microseconds per foot)"; `PorosityAndLithologyComputation.py` `HCslowness` = 210 | T1/T3 |
| Compaction factor | Cp | 1.0 | — | IP `swparameters.htm` "Sonic Cp (Default 1.0)"; Geolog `phi_son.info` `BCP` = 1 | T1/T2 |
| Compaction rule | — | `Cp = Δtsh/100` **when Δtsh > 100 µs/ft** | — | IP `basicloganalysis.htm` verbatim rule of thumb; Geolog `phi_son.lls` `if DT_SH > 328.084 then PHIE_SON *= 328.084/DT_SH` (328.084 us/m ≡ 100 µs/ft) | T1/T2 |
| Field-observed coefficient | CFO | 0.67, range 0.625–0.70 | — | Geolog `phi_son.info` `CFO` DEFAULT 0.67 + doc "normal range of 0.625 to 0.7"; Techlog ships 0.625 (`…from-sonic.html` "File coefficient") | T1/T3 |
| Tortuosity exponent | C_EXP | 2, range 1–10 | — | Geolog `phi_son.info` `C_EXP` DEFAULT 2, VALIDATION 1:10; "derived for porosities in the range 0 - 0.37" | T1 |
| Raiga/AFF exponent | EXP_AFF | 1.60 silica / 1.76 calcite / 2.00 dolomite | — | Geolog `phi_son.lls` AFF table; Techlog default 1.76 (`…from-sonic.html` "Raiga coefficient") | T1/T3 |

**Note on the compaction rule.** The `Δtsh > 100` guard is part of the cited rule in **both** vendor
sources. SandiBumi implements the ratio without the guard (`modules.rs:904`) — SB-POR-017.
*(Corrected 2026-08-20: the guard now ships — DEC-012 executed; `phi_son` refuses the run when
`OPT_CP=ON` and any effective `DT_SH < 100 µs/ft`, checked per sample so a zone override cannot
bypass it; `DT_SH = 100` is `Cp = 1` exactly and passes. Pinned by
`a_wyllie_compaction_that_would_inflate_porosity_refuses_the_run`.)*

### 5.3 Limits, ceilings and branch thresholds

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Maximum effective porosity | PHIE_MAX | 0.30 | v/v | Geolog `phi_den.info` / `phi_dn.info` / `phi_son.info` `PHIE_MAX` = 0.3 | T1 |
| Maximum porosity (Techlog) | PHImax | 0.35 | m3/m3 | Techlog `PorosityAndLithologyComputation.py` `PHImax` = 0.35 | T1 |
| Porosity limiting mode | — | `SHALE_REDUCED` | — | Geolog `.info` `OPT_PHIEMAX` DEFAULT = SHALE_REDUCED (all porosity modules). Techlog defaults to **no constraint** — not adopted | T1 |
| IP roll-off triple | PHIMAX, DPHIMAX, VCLCUT | **ABSENT — ships with no default** | v/v | IP `swparameters.htm` — the roll-off form is resolved but IP states no numeric defaults | T2 |
| Secondary porosity index max | SPI_MAX | 0.10 | v/v | Geolog `phi_dn.info` `SPI_MAX` = .1 | T1 |
| High-shale branch threshold | — | 0.95 | v/v | Geolog `phi_*.lls` hard-coded `VSH >= 0.95` (all six modules) — **a parameter in SandiBumi, defaulting to 0.95 with this source** | T1 |
| PHIE floor value | — | **ABSENT — ships with no default.** 0.001 and 0.0001 both attested inside one IP manual | v/v | IP2018 `A_porosity_sw.md` §6 bullet summary (0.0001) vs the `Phie Limit` parameter entry (0.001); `Vcl Limit` and PHIFLAG 9 give 0.0001 | T2 |
| Neutron shale-reduced clamp | — | [−0.015, 0.40] chart mode; [−0.015, 1.0] Bateman-Konen mode | v/v | Geolog `phi_dn.lls` and `phi_dnbk.lls` respectively | T1 |
| Density shale-reduced clamp | — | [1.950, 3.000] chart mode; none in Bateman-Konen mode | g/cm3 | Geolog `phi_dn.lls` / `phi_dnbk.lls` | T1 |
| Anhydrite zero-gate (Techlog) | — | **NON-ADOPTABLE — cited for verification.** `φ_n > φ_d` **and** `2.91 ≤ ρ_b ≤ 3.5` **and** `φ_n ≤ 0.04` → `φ = 0` | mixed | Techlog `petrophysics-porosity-neutrondensity-crossplot.html`, rendered. Three hard-coded literals, no user parameter | T3-eq |

### 5.4 Hydrocarbon correction

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Hydrocarbon density | ρhc | **ABSENT — ships with no default.** Attested: 1.0 (IP if blank) / 0.7 (Techlog script) / 0.8 (Techlog Quanti doc) | g/cm3 | IP `swparameters.htm` "If Hc Den is left blank, the value defaults to 1.0 gm/cc"; `PorosityAndLithologyComputation.py` `HCdensity` = 0.7; `…effective-porosity-from-density.html` = 0.8 | T1/T2/T3 |
| Hydrocarbon density bounds | — | 0.1 / 0.8 | g/cm3 | Techlog `PorosityAndLithologyComputation.py` `DHCmin` / `DHCmax` | T1 |
| Variable grain-density bounds | — | 2.65 / 3.0 | g/cm3 | Techlog `PorosityAndLithologyComputation.py` `DGmin` / `DGmax`. IP's `Rho GD max`/`min`: no default | T1/T2 |
| Hydrocarbon-correction bypass | — | 0.90 | g/cm3 | Geolog `phi_dnh.lls`: `if RHO_HC > 900 [k/m3] -> skip HC correction entirely` | T1 |
| Mud filtrate density | ρmf | 1.00 | g/cm3 | Geolog `phi_dnh.info` `RHO_MF` DEFAULT = 1000 k/m3, validation 500:2000 | T1 |
| Filtrate salinity | Pmf | **ABSENT — ships with no default.** Techlog attests 30 ppk (script) and 100,000 ppm (doc); house precedent 20,000 ppm | mass fraction | Geolog `phi_dnh.info` `SALMF` DEFAULT = `SALMF` reference, validation 0:400000 PPM; `project-kb\records\tiara-bumi.md` slide 6 | T1/T3/T4 |
| Flushed-zone saturation exponent | SW_EXP | **ABSENT — ships with no default.** Geolog 0.2 vs IP 1.0 — **opposite modelling assumptions** | — | Geolog `phi_dnh.info` `SW_EXP` DEFAULT 0.2, range 0-1, `SXOE = SWE**SW_EXP`; IP `Sxo = Swe^invasionfactor`, `invasionfactor` default 1 | T1/T2 |
| Water hydrogen index | HI_w | 1.0 fresh; else `ρmf(1−Pmf)` | — | Poupon et al. 1971 p.1005 verbatim "hydrogen index of a NaCl solution is equal to ρmf(1 − Pmf)"; Techlog `HIWXO` default 1 | **T1p**/T3 |
| Techlog shipped HI defaults | HI_gas, HI_fluid | **NON-ADOPTABLE — cited for verification.** 0.3 / 0.3 | — | `…effective-porosity-from-neutrondensity.html` and `…total-porosity-from-neutrondensity.html` Parameters tables. **Not self-consistent with Techlog's own α formula, which gives 0.446 at ρh 0.20** — compute from ρh instead | T3 |
| Flushed-zone HC volume limit | — | 0.02 | v/v | IP `swparameters.htm` "Volume of hydrocarbon limit seen in the flushed zone in decimals (Phi*(1-Sxo)) default 0.02" | T2 |
| Irreducible bulk-volume limit | BVIrr | 0.02 | m3/m3 | Techlog `PorosityAndLithologyComputation.py` `BVIrr` | T1 |
| Vsh limit for HC correction | VSHlimit | 0.30 | v/v | Techlog `PorosityAndLithologyComputation.py` — "Make hydrocarbon correction if Vsh < VSHlimit" | T1 |
| Apparent-HC-density model validity floors | — | ρh < 0.1414 (IP Modified → negative); ρh < 0.1018 (Geolog α → negative); ρh < 0.188 (any HI above methane's ceiling) | g/cm3 | Derived in dossier §3 from the vendor closed forms; roots of `(5.5ρ(4−ρ) − 3)/(16 − 2.5ρ)` are 0.1414 and 3.8586 | T1/T2 (derived) |
| Methane hydrogen-mass-fraction ceiling | HI_max | 0.2514 | — | Stoichiometry, `4 × 1.008 / 16.04`. **Not a petrophysical parameter** — a physical bound used as an assertion | — |

### 5.5 Excavation effect

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Lithology exponent | EXC_EXP | 2.0 | — | IP2025 B §2.5 `embim50` `(Rho_ma/2.65)^2`, corroborated within 0.8 % by Techlog `ρma^2.1` (`image2043`). **Not primary-sourced — see §7 ESC-1** | T2/T3-eq |
| ELAN constant term | EXC_FAC / K | 1.0 | — | Techlog `…elanplus-neutron-excavation-term.html` verbatim "The response parameter for the constant term, K, is called EXC_FAC. It has a default value of 1" | T3 |
| Geolog legacy scalar | NPHI_EXC | **NON-ADOPTABLE — cited for verification.** 1.3, range 1.0–1.5 | — | Geolog `phi_dnh.info` `NPHI_EXC` DEFAULT 1.3 VALIDATION 1:1.5; Poupon 1971 Eq A-10 "E is generally equal to about 1.3 but may be adjusted in an empirical manner". **Legacy mode only** | T1/**T1p** |
| Tool blacklist, evidence string | — | **NON-ADOPTABLE — cited for verification.** The four bare mnemonics `SNP`, `APLC`, `APSC`, `BPHI` | — | Techlog `PorosityAndLithologyComputation.py` L162/L166, `CorrExcF` verbatim: "Make excavation factor correction? (default on, switch off for SNP, APLC, APSC and BPHI)". **Evidence string only — not executable** | T1 |
| Tool blacklist, resolved | — | **NON-ADOPTABLE — cited for verification.** `{Schlumberger SNP, Gearhart SNP, Schlumberger APS-APLC, Schlumberger EcoScope BPHI}` — **and `APSC` UNRESOLVED** | tool-enum tokens | Derived by splitting the 39-entry `TOOL` list at `PorosityAndLithologyComputation.py` L150; `SNP` matches two entries, `BPHI` crosses an `EcoScope`/`Ecoscope` casing boundary, `APSC` matches nothing | T1 (derived) |

### 5.6 Crossplot, iteration and convergence

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Bateman-Konen crossplot constants | — | **NON-ADOPTABLE — cited for verification.** 2.71 / 4.00 / 0.7 / −5 / −0.16 / −2.06 / −1.17 / −16 / −0.4 | mixed | Geolog `phi_dnbk.lls` `DN_XPLOT`; reference Bateman & Konen, *The Log Analyst*, Nov-Dec 1977 | T1 |
| Porosity iteration cap / tolerance | — | 50 / 1e-3 | — | Geolog `phi_dnh.lls` (50); Techlog `…hydrocarbon-correction.html` (50, tol 0.001); IP PhiSw tol 0.001 | T1/T2/T3 |
| Sxo iteration cap / tolerance | — | 20 / 2e-3 | — | IP2025 B §3.2 `\|ΔSxo\| < 0.002`; IP PHIFLAG 2 caps the HC loop at 20 | T2 |
| N-D limestone grain density (Techlog) | ρ_lim | 2.71 | g/cm3 | Techlog `petrophysics-porosity-neutrondensity-crossplot.html` verbatim "ρ_lim: Limestone grain density, default 2.71" | T3-eq |
| N-D sandstone grain density (Techlog) | ρ_sand | 2.65 | g/cm3 | same page, "ρ_sand … default 2.65" | T3-eq |
| N-D mud filtrate density (Techlog) | ρ_mf | 1 | g/cm3 | same page, "ρ_mf: Mud filtrate density, default 1". **Anchors on mud filtrate, not formation water** — a different quantity from the fresh fluid density even though both are 1.00 | T3-eq |
| N-D convergence tolerance (Techlog) | — | **NON-ADOPTABLE — cited for verification.** 0.01, published as the **equality** `φ_nd(n−1) − φ_nd(n) = 0.01` | v/v | Techlog same page. SandiBumi implements `abs(Δφ) <= tol` and records the published form as an authoring defect. 0.01 v/v = 1 p.u. is an order of magnitude looser than the 0.001 used by Techlog's own HC loop and by IP | T3-eq |

**Every parameter that ships `ABSENT` is a deliberate refusal, not a gap in research.** Eighteen of
the seventy-four rows carry it. In each case the vendors are in genuine, evidenced conflict, and a
SandiBumi default would be a number invented to fill a shape — which is the one failure mode that
computes, plots and ships silently.

---

## 6. Acceptance tests

Forty-one. `SB-POR-T01 … T27` are the dossier's `POR-T01 … T25` plus `T14b` and `T18b`, carried with
their pinned values unchanged. `SB-POR-T28 … T41` are new, and target the as-built defects in §3 that
the dossier could not know about.

### 6.1 Carried from the dossier (T01 … T27)

| ID | Dossier ID | Test | Pinned expectation |
|---|---|---|---|
| `SB-POR-T01` | `POR-T01` | Apparent HC electron density vs Poupon A-5/A-6 at ρh ∈ {0.20, 0.25, 0.30, 0.50, 0.70, 0.80} | within **1.5 %** each; values 0.2496 / 0.3086 / 0.3666 / 0.5910 / 0.8106 / 0.9216 |
| `SB-POR-T02` | `POR-T02` | HC hydrogen index vs Poupon A-8/A-9, same sweep | gas within **4.1 %**, oil within **3.0 %**; 0.4464 / 0.5276 / 0.5994 / 0.8190 / 0.9954 / 1.0944 |
| `SB-POR-T03` | `POR-T03` | Geolog-legacy `B` divergence is caught (ρh 0.2, ρmf 1.0, Pmf 0.02) | **0.5445 vs 0.8327**; legacy path reachable only behind `legacy_geolog_dnh`, warns below ρh 0.5 |
| `SB-POR-T04` | `POR-T04` | Excavation reference case, φ 0.25, Sxo 0.55, ρh 0.2, ρma 2.65, Vcl 0 | **2.71 p.u. ± 0.02** (IP form, Swx 0.7329); Techlog form **2.91 p.u.**; the two agree within 0.25 p.u. |
| `SB-POR-T05` | `POR-T05` | Excavation lithology sensitivity at ρma 2.87 | ratio **1.1729 ± 0.001** for exponent 2; a silent revert to `sqrt` returns **1.0407** and fails. Absolutes 3.18 vs 2.82 p.u. |
| `SB-POR-T06` | `POR-T06` | Additive-bracket guard (X-01) | multiplied rendering returns **0.0132 p.u.**; fail if result < 0.5 p.u. |
| `SB-POR-T07` | `POR-T07` | Forward/inverse round trip through the HC chain | recovers φ to **1e-4**, Sxo to **2e-3**, within the iteration cap |
| `SB-POR-T08` | `POR-T08` | Bateman-Konen vs the digitised-chart branch, clean water-bearing limestone | agree within **1 p.u.** over φ 0–40 p.u.; gas envelope documented |
| `SB-POR-T09` | `POR-T09` | Unit invariance: {g/cc, µs/ft, ppm} vs {kg/m³, µs/m, mass fraction} | identical to machine precision |
| `SB-POR-T10` | `POR-T10` | Δtma spread at Δt 90, Δtfl 189, for {53.0, 55.0, 55.5, 56.0, 47.5} | **0.2721 / 0.2612 / 0.2584 / 0.2556 / 0.3004** — fails loudly if a global `DTMA` default is introduced |
| `SB-POR-T11` | `POR-T11` | Limit ordering on a high-Vsh interval | `PHIT ≥ PHIE` always; `PHIT` rebuilt **after** `PHIE` is clamped |
| `SB-POR-T12` | `POR-T12` | Clamp logging: gas sand with shale-reduced NPHI below −0.015 | clamp fires, flag emitted, value logged; silent truncation fails |
| `SB-POR-T13` | `POR-T13` | IP `NeuHyHI`-forces-`Exfact`-zero parity behind a compat flag | flag on ⇒ `DPHI_EX == 0`; flag off ⇒ excavation applies and warns |
| `SB-POR-T14` | `POR-T14` | Excavation tool gate keys on **resolved enum tokens** | `DPHI_EX == 0` for `Schlumberger SNP`, `Gearhart SNP`, `Schlumberger APS-APLC`, `Schlumberger EcoScope BPHI`; **≠ 0** for `Schlumberger APS-FPLC`; lookup case-insensitive across `EcoScope`/`Ecoscope` |
| `SB-POR-T14b` | `POR-T14b` | `APSC` stays unresolvable | token appears in an `UNRESOLVED_VENDOR_REFERENCE` diagnostic at config load; **a build that resolves it to anything fails** |
| `SB-POR-T15` | `POR-T15` | `PHIT_SH` fluid term: ρw 1.10 with ρfl 1.00, ρdsh 2.78, ρsh 2.50 | uses ρw ⇒ **0.1667**; the ρfl variant gives 0.1573 — assert the ρw path |
| `SB-POR-T16` | `POR-T16` | Non-convergence discipline (ρh = ρmf) | NULL + flag; no partial iterate, no exception |
| `SB-POR-T17` | `POR-T17` | Low-ρh validity sweep 0.05 → 0.90 through every reachable HC form | `RHOHC_APP` and `HI_HC` strictly positive throughout; warning below **0.19**; IP Modified (0.1414) and Geolog α (0.1018) unreachable without an explicit legacy flag |
| `SB-POR-T18` | `POR-T18` | Sonic shale-reduction path, FLD_OBSB and AFF, Vsh 0.20, Δt 90, Δtsh 100, Δtma 55.5 | `dtsr` = **87.5**; φ = **0.1960** / **0.1981**. Doc-block route returns 0.2055 / 0.2086 (or 0.2568 / 0.2608) and fails |
| `SB-POR-T18b` | `POR-T18b` | `FLD_OBSA` Newton-Raphson seeds from `dtsr`, not raw `DT` | seeded from **87.5** not 90; `iter == 20 → missing` fires rather than returning a partial iterate |
| `SB-POR-T19` | `POR-T19` | Apparent vs effective parity, one code path | apparent = `(ρma−ρb)/(ρma−ρfl)` exactly; effective differs by exactly `Vsh(ρma−ρsh)/(ρma−ρfl)` |
| `SB-POR-T20` | `POR-T20` | `SHALE_CONV` is a real fork | field-observed **0.1960 vs 0.2115** (1.55 p.u.); Raiga **0.1981 vs 0.2111** (1.30 p.u.); intermediates `dtsr` **87.5** vs `Δt_cc` **81.1**; active convention written to the run header |
| `SB-POR-T21` | `POR-T21` | **Control** — Wyllie agrees across vendors | both return **0.1917604**. *If this ever diverges, the harness is broken, not the tools* |
| `SB-POR-T22` | `POR-T22` | N-S shale-porosity term bounded (Δt 60, Δtsh 100, Δtma 55.5, Δtfl 189) | shipped form **0.3341**; Techlog's form returns **4.23** and must be **unreachable**, not even behind a legacy flag; `0 ≤ φsh ≤ 1` over Δt 50–140 |
| `SB-POR-T23` | `POR-T23` | `CSR` refusal is enforced, not advisory | core **refuses to run** and names `CSR`; at `CSR` 0.7 the four bridges reproduce and `VSH = min(VCL/0.7, 1.0)` clamps above `VCL` 0.7 |
| `SB-POR-T24` | `POR-T24` | N-D convergence is a comparison, not an equality | terminates on `abs(Δφ) <= tol` within the cap, cap-hit flagged; tolerance is a **parameter**, not the literal 0.01 |
| `SB-POR-T25` | `POR-T25` | `Sxo` exponent divergence is visible at Sw 0.30 | **0.300 (IP) vs 0.786 (Geolog)**; build ships no default and refuses the HC loop until set |

### 6.2 New — targeting the as-built defects of §3

| ID | Test | Pinned expectation |
|---|---|---|
| `SB-POR-T28` | **The branch named `RHG` is not Raymer-Hunt-Gardner.** Run `modules.rs:907`'s form at Δt 90, Δtma 55.5, Δtsh 100, Δtfl 189, VSH 0.20 and compare against the three published RHG renderings and against the field-observed conventions | current branch returns **0.1729167**; Geolog field-observed convention **0.1828571** (+0.99 p.u.); Techlog field-observed convention **0.1972873** (+2.44 p.u.). Assert the method registry contains **no** method named `RHG`/`Raymer-Hunt-Gardner` unless it computes one of F3's three forms |
| `SB-POR-T29` | **Compaction correction cannot inflate porosity.** Wyllie at `DT_SH` 90, `OPT_CP` ON vs OFF | OFF: `PHIT` 0.2584270, `PHIE` **0.2067416**. ON at `Cp` 0.90: `PHIT` 0.2871411, `PHIE` **0.2297129** (**+2.30 p.u.**). Assert the ON case is **refused or hard-flagged** because `Cp < 1`; sweep `DT_SH` 60→150 and assert `Cp < 1` never silently applies |
| `SB-POR-T30` | **One `PHIT_SH` per product.** Compute the clay-bound-water porosity through every module that forms it, on one parameter set | all modules agree exactly. Regression pins the current disagreement: `phi_den` defaults give **0.0909091**, `sspw` defaults give **0.1812865** — a factor **1.99**, i.e. **4.5 p.u. of `PHIT` at VSH 0.5** |
| `SB-POR-T31` | **No output-mnemonic collision.** Run `phi_den` then `phi_dn` on one well | both results survive under distinct mnemonics; a build in which the second overwrites the first's `PHIE`/`PHIT` fails |
| `SB-POR-T32` | **Porosity family is registered.** Assert `curves.rs` `FAMILIES` resolves `PHIE`, `PHIT`, `PHIA`, `DPHI` and that an imported vendor `PHIE` is distinguishable from a computed one by provenance | family present; the two are distinguishable. Today `FAMILIES` has fourteen entries and none is a porosity family |
| `SB-POR-T33` | **SSPW gas conditioning matches SSC.** φD 0.25, NPHI 0.10 through both modules' gas branches | both return **0.1903943**. Pins the current defect: `sspw` returns **0.1431782**, **4.72 p.u. low**, from the `0.2·φD² + 0.8·φN²` weight that `ssc.rs:172-178` records as inverting the crossover |
| `SB-POR-T34` | **No dead parameters.** For every `ModuleSpec`, assert each declared `param`/`opt` is read by the module body on at least one reachable path | passes for all modules. Today `sspw`'s `NPHI_MAT`, `NPHI_SH` and `NPHI_FL` fail it |
| `SB-POR-T35` | **One matrix density across a chained workflow.** Run the documented `gascorr` → `phi_den` chain | both stages use the same `RHO_MA`. Today they ship **2.65** and **2.645** |
| `SB-POR-T36` | **Average and RMS are not methods.** Assert the crossplot method registry contains no entry whose computation is `(φD + φN)/2` or `sqrt((φD² + φN²)/2)`, and that any such curve is tagged as a comparison curve and excluded from pay summation by default | registry clean; comparison curves tagged. Also assert the doc string no longer claims chart-lookup equivalence |
| `SB-POR-T37` | **Bateman-Konen vs the average shortcut, pinned.** RHOB 2.20, NPHI 0.30, VSH 0.20, shipped shale endpoints | shale-reduced pair `rhosr` **2.125**, `nphisr` **0.2875**; average route `PHIE_DN` **0.2414438**; Bateman-Konen route **0.2578699** — **1.64 p.u.** |
| `SB-POR-T38` | **The ceiling cannot bind silently.** Same inputs as T37, `PHIE_MAX` 0.3, `OPT_PHIEMAX` `SHALE_REDUCED` | ceiling evaluates to **0.24**, binds (unlimited value 0.2414438), and **raises a flag**. A run in which `PHIE` is rewritten with no flag fails |
| `SB-POR-T39` | **One limiting contract.** Drive each porosity method into its floor and its ceiling | all methods floor and cap through the same code path with the same flags. Today `phi_son` floors at 0.0 and caps at 1.0 while `phi_den`/`phi_dn` floor at `PHIE_FLOOR` and cap at `phie_max·(1−VSH)` |
| `SB-POR-T40` | **The PHIE floor is a parameter.** Assert no compile-time constant governs the floor value, and that both **0.001** and **0.0001** are reachable by configuration with the choice recorded | passes. Today `modules.rs:335` is a `const` |
| `SB-POR-T41` | **Conditioning flags are wired.** Run a porosity method over an interval carrying `BADHOLE = 1` and one carrying `COAL_FLAG = 1` | the flags are declared inputs, the branch taken is recorded per sample, and the behaviour is defined rather than depending on a user-set generic Mask |

---

## 7. Open items, escalations and refusals

### 7.1 Escalations — carried from the dossier

| ID | What is needed | Why it blocks |
|---|---|---|
| **ESC-1** | **Segesman, F. & Liu, O., "The Excavation Effect", SPWLA Twelfth Annual Logging Symposium, 1971** (Poupon 1971 Ref. 12); and/or **Schlumberger, *Log Interpretation Principles*, 1969, Chapter 13** (Ref. 5) | The only primary source for the excavation term. Would settle SB-POR-039's exponent definitively and independently verify Techlog's fitted constants. **The single highest-value acquisition in the domain.** |
| **ESC-2** | **Gaymard, R. & Poupon, A., *The Log Analyst*, Sept.-Oct. 1968, Vol. IX, No. 5, pp. 3-12** | The year dispute is settled (Techlog right, Geolog wrong). What remains: whether the 1968 paper states a validity range in ρh — which decides SB-POR-033's behaviour below 0.188 g/cc — and its own bracketing of the `A` factor |
| **ESC-3** | `TechlogQuanti` binary — `neuCorrectionHydrocarbon`, `denCorrectionHydrocarbon`, `porNeutronDensity` | Would settle which of the two contradictory excavation renderings actually ships and which `SwH` definition. Compiled and absent from the local tree; not closable by further reading |
| **ESC-5** | Geolog's compiled chart functions | **Deliberately not to be closed.** Their content is vendor chart data. SandiBumi uses its own digitisations |
| **ESC-7** | IP's `Den Hyd model` provenance | IP flags the "Modified" model as given without citation; it is 3.22× low against the primary source and negative below ρh 0.1414. If it has a real source, SandiBumi should know before rejecting it |

`ESC-4` and `ESC-6` are **CLOSED** in the dossier and are carried here only as history: the Geolog
`pha_*` family is the unlimited apparent-porosity lineage (no new equations), and Techlog's
"unpublished" equations were on disk all along as images under empty `<h2>Equations</h2>` headings.

**One escalation this chapter adds:**

| ID | What is needed | Why |
|---|---|---|
| **ESC-POR-8** | **Bateman, R.M. & Konen, C.E., *The Log Analyst*, Nov-Dec 1977** | SB-POR-021 adopts the Bateman-Konen **method** on two-vendor structural corroboration. The nine constants in §5.6 are **Geolog's rendering** of it, not the paper's, and the paper is not held locally. This chapter therefore ships them `NON-ADOPTABLE — cited for verification`, which **diverges from the dossier's implicit adoption of `BK_CONSTANTS` as a shipping parameter set**. The divergence is deliberate and is flagged for Jauhar: adopting a vendor's fitted constants for a published method, without the publication, is exactly the "carried over from a neighbouring vendor" failure the parameter discipline exists to prevent. Until the paper is held, SB-POR-021's constants are ABSENT and the method cannot ship as a default |

### 7.2 Open items — carried, with what this chapter did with each

| ID | Carried status | This chapter |
|---|---|---|
| **OPEN-1** | D-09 exponent not primary-sourced | SB-POR-039 adopts power-2/2.1 on two-implementation agreement, **states the evidence basis**, SB-POR-042 holds the escalation |
| **OPEN-2** | X-01, 220× excavation contradiction | Adjudicated: additive, on structural grounds (F7). `SB-POR-T06` guards it |
| **OPEN-3** | X-01b, two `SwH` definitions | Neither adopted; IP's `Swx` used. Unchanged |
| **OPEN-4** | `A_FACTOR` bracketing | Inside form adopted on primary-source authority; 0.12 p.u. at the reference case. Unchanged |
| **OPEN-5** | X-02, Geolog neutron `B` | SB-POR-030 rejects it and states why; whether Geolog's is deliberate is still undetermined |
| **OPEN-6** | X-03, Poupon's `1/(1 − Vclay)` scaling applied by nobody | **Still open. Not implemented either way** — ~20 % of the excavation term at Vsh 0.2, and implementing it silently in either direction would be the worse error |
| **OPEN-7** | Techlog internal conflicts | Promoted to a finding in its own right (F23) and discharged through §5's `ABSENT` rows |
| **OPEN-8** | ρma 2.65 vs 2.645 | **Escalated to a live product defect** — SandiBumi ships 2.645 in three modules and 2.65 in a fourth (SB-POR-011, §3.6). Still needs a source string, not a rounding |
| **OPEN-9** | `CSR` | **CLOSED** as a refusal contract; carried as SB-POR-006/012 and `SB-POR-T23` |
| **OPEN-10** | Low-ρh validity of Gaymard-Poupon | SB-POR-033 **warns and proceeds; does not silently clamp**, per the dossier. The right behaviour there needs ESC-2 |
| **OPEN-11** | X-04 / X-05 | Discharged: SB-POR-033 gates X-04's zero-crossing; SB-POR-015 and `SB-POR-T18` gate X-05 |
| **OPEN-12** | Techlog `Δt_cc` unfloored | Discharged as SB-POR-018 |
| **OPEN-13** | `APSC` resolves to nothing | Discharged as SB-POR-041 and `SB-POR-T14b`. **Still open at the vendor** — do not guess |
| **OPEN-14** | IP `PHIE` floor at two magnitudes | Discharged as SB-POR-045; **and escalated**, because SandiBumi has already silently picked one side in shipped code (`modules.rs:335`) |

### 7.3 Refusals — things SandiBumi will not implement, and why

1. **Techlog's neutron-sonic `φ_sh = (ΔT_shale − 47.6)/(ΔT − 47.6)`.** A dimensional error that
   returns 4.23 in a fast clean sand. Not reachable, not behind a legacy flag (SB-POR-053).
2. **Techlog's multiplied excavation bracket.** A typesetting defect worth a factor 220 (SB-POR-039).
3. **Geolog's `α = 1.67ρ − 0.17` neutron hydrocarbon factor** as a default. 1.51× the primary
   source in gas; reachable only behind an explicit legacy flag (SB-POR-030, `SB-POR-T03`).
4. **IP's "Modified" apparent hydrocarbon density** as a default. Negative below ρh 0.1414 and
   uncited (SB-POR-029, ESC-7).
5. **Guessing `APSC`.** The mapping is not in the held corpus. It stays in a diagnostic list
   (SB-POR-041, `SB-POR-T14b`).
6. **Any vendor chart lookup table**, from any of Schlumberger, Halliburton, Baker, Weatherford,
   Sperry-Sun, PathFinder, Anadrill or GE. Charts are cited by existence, attribution and purpose;
   the only chart-derived numbers SandiBumi ships come from its own gated digitisation pipeline
   (SB-POR-022).
7. **The arithmetic-average and RMS combinations as porosity methods** (SB-POR-023). Retained only
   as labelled comparison curves, with IP's own warning attached.
8. **Hard-coded lithology-kill literals** (SB-POR-049).
9. **`BK_CONSTANTS` as shipping defaults** until the 1977 paper is held (ESC-POR-8). This is the one
   place this chapter deliberately diverges from the dossier's adoption spec.

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

---

## 8. Traceability — dossier disposition

**Count reconciliation.** The dossier has no single "finding count" — it publishes `open_count = 13`
and "25 of 25 critique findings fixed", which are two different tallies of two different things. The
enumeration used here counts every item the dossier gives an identifier, a table row or a numbered
block: **260**.

| Dossier section | Items | Where dispositioned |
|---|---|---|
| §1 method inventory (IP-1…21, GL-1…11, TL-1…12) | 44 | §8.1 |
| §2 discrepancies (X-01, X-01b, X-02…X-08) | 10 | §8.2 |
| §3 differences that matter (§3.1…§3.7) | 7 | §8.3 |
| §4 optimal choice per item | 18 | §8.4 |
| §4.1 ledger and FINDINGS-rule dispositions | 10 | §8.5 |
| §5.1 canonical equation blocks (1, 1b, 2…8 + the VSH/VCL contract) | 10 | §8.6 |
| §5.2 parameter rows | 72 | §8.7 (grouped — §5 is the row-for-row table) |
| §5.3 FINDINGS §6 rules | 8 | §8.8 |
| §5.4 tests (POR-T01…T25, T14b, T18b) | 27 | §8.9 (grouped — §6 is the row-for-row table) |
| §5.5 module-boundary notes | 4 | §8.10 |
| §6 escalations (ESC-1…7) | 7 | §7.1 |
| §6 open items (OPEN-1…14) | 14 | §7.2 |
| `## Critique disposition` (BLK-1…3, MAJ-1…12, MIN-1…10) | 25 | §8.11 |
| "Findings opened by this revision pass", unnumbered rows | 4 | §8.12 |
| **Total** | **260** | |

### 8.1 Method inventory (44)

| ID | Item | Disposition | Where |
|---|---|---|---|
| IP-1 | Density porosity, clay + dual-fluid | ADOPTED | SB-POR-029, §5.1 |
| IP-2 | Neutron porosity, clay + matrix + excavation + salinity + HI | ADOPTED | SB-POR-030, 039 |
| IP-3 | Neutron excavation term `Exfact` | ADOPTED | SB-POR-039 |
| IP-4 | Sonic — Wyllie | ADOPTED | SB-POR-014, §5.2 |
| IP-5 | Sonic — Raymer-Hunt (quadratic in velocity) | ADOPTED as one of three named renderings | SB-POR-020 |
| IP-6 | N-D crossplot, variable matrix | DEFERRED to `MIN` | §1 seam |
| IP-7 | N-D Variable Sxo solver | ADOPTED (precedence) | SB-POR-051, 052 |
| IP-8 | N-D Variable Hc Den solver | ADOPTED (precedence) | SB-POR-051 |
| IP-9 | N-D Variable GD solver | ADOPTED (precedence) | SB-POR-051 |
| IP-10 | N-D Variable Vcl solver + curve-reduction fallback | ADOPTED (precedence, incl. PHIFLAG 6/7) | SB-POR-051, 003 |
| IP-11 | Neutron/Sonic variable-matrix | ADOPTED | SB-POR-027 |
| IP-12 | Single-tool neutron and density modes | ADOPTED | SB-POR-001 |
| IP-13 | Pass-through with PhiT↔PhiE conversion | ADOPTED | SB-POR-002, 008 |
| IP-14 | Organic-shale porosity (kerogen + heavy minerals) | DEFERRED to `TOC` | §1 seam |
| IP-15 | Total porosity / PhiT-clay / Swb | ADOPTED | SB-POR-008 |
| IP-16 | Porosity limits: shale-zone, bad hole | ADOPTED | SB-POR-043, 044, 047 |
| IP-17 | BLA simplified porosity (separate defaults) | EVIDENCE-ONLY (Δtma 55.0 witness) | §5.2 |
| IP-18 | Basic Log Calculations φ-density, ρb back-calc, M/N | DEFERRED to `LIT` | — |
| IP-19 | SSM porosity chain (own `exfact`) | REJECTED on the `sqrt` exponent; EVIDENCE-ONLY otherwise | SB-POR-039, F8, X-06 |
| IP-20 | Per-tool neutron look-up tables (`.neu`) | REJECTED — vendor chart data | SB-POR-022, §7.3 |
| IP-21 | Coal / salt / anhydrite discrimination | ADOPTED via `condflag` | SB-POR-048 |
| GL-1 | `phi_den` | ADOPTED — the as-built core | SB-POR-055, §3.1 |
| GL-2 | `phi_dn` (chart-lookup crossplot, 12 tool types) | REJECTED (chart data); structure ADOPTED | SB-POR-021, 022 |
| GL-3 | `phi_son` (4 transforms) | ADOPTED | SB-POR-013, 014, 015 |
| GL-4 | `phi_ns` (chart lookup) | REJECTED (chart data) | SB-POR-022 |
| GL-5 | `phi_dnbk` — Bateman-Konen, fully analytic | ADOPTED as method; constants ESCALATED | SB-POR-021, ESC-POR-8 |
| GL-6 | `phi_nsbk` | ADOPTED as method | SB-POR-027 |
| GL-7 | `phi_dh` — HC-corrected density | ADOPTED (its upgraded quadratic) | SB-POR-029 |
| GL-8 | `phi_dnh` — HC-corrected N-D | REJECTED on the neutron `B`; architecture ADOPTED | SB-POR-030, 031 |
| GL-9 | `pha_*` apparent-porosity family | ADOPTED | SB-POR-002, `SB-POR-T19` |
| GL-10 | `phi_nsbk` shale-corrected sibling | ADOPTED | SB-POR-027 |
| GL-11 | `por_from_rhob` (table-pipe utility, inverted-sign Wyllie) | EVIDENCE-ONLY | SB-POR-054, F22 |
| TL-1 | Quanti effective/total porosity from density | ADOPTED | §5.1, F16 |
| TL-2 | Quanti from neutron-density (core crossplot algorithm) | ADOPTED structurally; the anhydrite gate REJECTED | SB-POR-021, 049 |
| TL-3 | Quanti from neutron-sonic (9-step set) | ADOPTED except `φ_sh` | SB-POR-027, 053 |
| TL-4 | Quanti from sonic — four transforms | ADOPTED | SB-POR-013 … 020 |
| TL-5 | Shear-sonic, microlog, bin, core-calibrated, DMR porosity | DEFERRED to `NMR`/`RPH`; core-calibrated → SB-POR-062 | §1 seam |
| TL-6 | Iterative HC correction (D + N + sonic) | ADOPTED | SB-POR-031, 050 |
| TL-7 | Excavation additive term | ADOPTED (additive rendering only) | SB-POR-039 |
| TL-8 | `TechlogQuanti` API (compiled) | ESCALATED | ESC-3 |
| TL-9 | `PorosityAndLithologyComputation.py` (2,697 lines) | ADOPTED as T1 evidence throughout | §5 |
| TL-10 | Quanti.Elan / ELANPlus excavation (Eq 34-37) | EVIDENCE-ONLY (`EXC_FAC` = 1) | §5.5 |
| TL-11 | ELANPlus linear-NPHI endpoint construction | DEFERRED to `MIN` | §1 seam |
| TL-12 | Neutron tool-type table (39 script / 37 doc) | EVIDENCE-ONLY; gate rebuilt on SandiBumi's own register | SB-POR-041 |

### 8.2 Discrepancies (10)

| ID | Disposition | Where |
|---|---|---|
| X-01 — two Techlog excavation renderings, 220× | ADOPTED additive; multiplied REJECTED | F7, SB-POR-039, `SB-POR-T06` |
| X-01b — two `SwH` definitions | DEFERRED — neither adopted, IP's `Swx` used | OPEN-3, ESC-3 |
| X-02 — Geolog neutron `B` 1.51× Poupon | REJECTED | F11, SB-POR-030, `SB-POR-T03` |
| X-03 — Poupon's `1/(1−Vclay)` scaling applied by nobody | DEFERRED — not implemented in either direction | OPEN-6 |
| X-04 — IP Modified negative below ρh 0.1414 | REJECTED as default; gated | F9, SB-POR-033, `SB-POR-T17` |
| X-05 — Geolog doc omits the `dtsr` reduction its code performs | ADOPTED (code is authoritative) | F4, SB-POR-015, `SB-POR-T18` |
| X-05b — same divergence on `FLD_OBSA` | ADOPTED | SB-POR-015, `SB-POR-T18b` |
| X-06 — IP SSM `A_ssm` 18.2 % low → 1.14 p.u. | REJECTED | SB-POR-029 |
| X-07 — normalised vs subtractive shale conventions | ADOPTED as an explicit fork | F2, SB-POR-013, `SB-POR-T20`/`T21` |
| X-08 — Techlog N-S `φ_sh` divides by the log reading | REJECTED — unreachable | F6, SB-POR-053, `SB-POR-T22` |

### 8.3 Differences that matter (7)

| § | Subject | Disposition | Where |
|---|---|---|---|
| 3.1 | Geolog `phi_dnh` neutron `B` wrong in gas | ADOPTED as a finding | F11 |
| 3.2 | IP Conventional vs Modified, 3.3× swing | ADOPTED as a finding | F10 |
| 3.3 | D-09 excavation exponent, evidence settles it | ADOPTED with the basis stated | F8, SB-POR-039 |
| 3.4 | X-01, 220× | ADOPTED | F7 |
| 3.5 | Three excavation formulations disagree ~25 % | ADOPTED | F8, `SB-POR-T04` |
| 3.6 | Sonic matrix default 53/55/55.5/56/47.5 | ADOPTED | F1, SB-POR-016, `SB-POR-T10` |
| 3.7 | Structural divergences with no numeric bridge | ADOPTED | F14, F15, F16 |

### 8.4 Optimal choice per item (18)

| Item | Dossier choice | Disposition | Where |
|---|---|---|---|
| Density porosity core | Geolog `phi_den` shale-endpoint form | ADOPTED | §5.1, §3.1 |
| PhiT_sh fluid term | Geolog `RHO_W`, not IP `Rho_fl` | ADOPTED | SB-POR-008, `SB-POR-T15` |
| N-D crossplot default route | Bateman-Konen analytic | ADOPTED as method; constants ESCALATED | SB-POR-021, ESC-POR-8 |
| N-D crossplot alternate route | Chart-based, own digitisations only | ADOPTED | SB-POR-022 |
| Sonic transform default | Raymer-Hunt / field-observed, not Wyllie | ADOPTED with the naming corrected | SB-POR-014, 020 |
| Sonic transform set | Ship all four, each labelled | ADOPTED | SB-POR-014 |
| Sonic compaction | Geolog two-mode `OPT_CP`, default `DT_SH` mode | ADOPTED **with a guard the dossier did not require** | SB-POR-017, `SB-POR-T29` |
| HC electron density | Gaymard-Poupon quadratic form | ADOPTED | SB-POR-029 |
| HC hydrogen index | Techlog's `9ρ_h·N_h` | ADOPTED | SB-POR-030 |
| Water/filtrate HI | `ρmf(1 − Pmf)` | ADOPTED | §5.4 |
| HC correction architecture | Additive log correction + re-solve | ADOPTED | SB-POR-031 |
| Excavation | Additive Segesman-Liu, power 2.0-2.1 | ADOPTED | SB-POR-039 |
| Excavation applicability | Techlog's gate principle, not its string | ADOPTED | SB-POR-041 |
| Excavation `Sw`-analogue | IP's `Swx` with the clay term | ADOPTED | §5.1 |
| Sonic HC | IP's fluid-Δt mixing + Techlog's warning | ADOPTED | §5.1 |
| Porosity limit | IP's smooth roll-off over Geolog's step | ADOPTED | SB-POR-044 |
| Flags | IP numbered `PHIFLAG` **and** Geolog `MTH_PHI` | ADOPTED | SB-POR-003 |
| Distribution-mode φ corrections | Do not implement | ADOPTED as a refusal | §7.3 |

### 8.5 Ledger and FINDINGS-rule dispositions (10)

| Item | Disposition | Where |
|---|---|---|
| D-09 excavation exponent | ADOPTED on two-implementation agreement; ESCALATED for primary source | SB-POR-039, 042, ESC-1 |
| D-11 shale-zone porosity limit malformed in ASCII | ADOPTED (form resolved; defaults absent) | SB-POR-044 |
| D-13 Sonic Sand 56 vs "(180 µS/m)" | ADOPTED with the mis-shelved citation corrected | F1, SB-POR-016 |
| D-16 IP PHIE floor, 0.0001 vs 0.001 | ADOPTED as `ABSENT`; ESCALATED because SandiBumi already picked a side | SB-POR-045, `SB-POR-T40` |
| B §8 item 3 — D-09 unresolved | ADOPTED as still-open | OPEN-1 |
| D-15 `SW` vs `SWE`/`SWT` nomenclature | DEFERRED to `SAT`; the no-bare-`SW` rule adopted here | §5.3 rule 8 |
| FINDINGS rule 1 — no raster-only truth | ADOPTED | §5.1 canonical form |
| FINDINGS rule 3 — unit-typed quantities | ADOPTED | SB-POR-056, `SB-POR-T09` |
| FINDINGS rule 9 — defaults cited or absent | ADOPTED | SB-POR-055, §5 |
| FINDINGS rule 11 — worked examples must reproduce | ADOPTED | §6 |

### 8.6 Canonical equation blocks (10)

| Block | Disposition | Where |
|---|---|---|
| VSH/VCL volume contract + `CSR` bridge | ADOPTED | SB-POR-006, 012, `SB-POR-T23` |
| 1. Single-log responses, shale-endpoint form | ADOPTED | §3.1, SB-POR-055 |
| 1b. `SHALE_CONV` selector | ADOPTED | SB-POR-013, `SB-POR-T20` |
| 2. N-D crossplot, chart-free (Bateman-Konen) | ADOPTED as method; constants ESCALATED | SB-POR-021, ESC-POR-8 |
| 3. Hydrocarbon response (Gaymard-Poupon) | ADOPTED incl. the validity gate | SB-POR-029, 030, 033 |
| 4. Excavation effect (additive) | ADOPTED | SB-POR-039 |
| 5. Applying corrections (inverse direction) | ADOPTED as two named functions | SB-POR-005, 040 |
| 6. Iteration | ADOPTED | SB-POR-050, 051 |
| 7. Totals and limits (Geolog ordering) | ADOPTED | SB-POR-009, 043, 045 |
| 8. Neutron-sonic shale term (do not implement Techlog's) | ADOPTED as a refusal | SB-POR-053 |

### 8.7 Parameter rows (72) — grouped

**§5 of this chapter is the row-for-row disposition**; all 72 dossier rows appear there, values
byte-exact, plus 2 model-validity rows this chapter adds (74 total). By disposition: **50 ADOPTED**
(a cited value ships), **18 ADOPTED-AS-REFUSAL** (`ABSENT — ships with no default`, where the
vendors are in evidenced conflict), **6 EVIDENCE-ONLY** (`NON-ADOPTABLE — cited for verification`:
the Techlog N-S hard-coded slowness pair, the Techlog shipped HI defaults, the anhydrite zero-gate,
the Geolog legacy `NPHI_EXC` scalar, the two excavation tool-blacklist rows, and the Techlog N-D
convergence literal). `BK_CONSTANTS` moves from the dossier's implicit ADOPTED to **EVIDENCE-ONLY +
ESCALATED** here — the one deliberate divergence, recorded in §7.1 as ESC-POR-8.

### 8.8 FINDINGS §6 rules bound to this domain (8)

| Rule | Disposition | Where |
|---|---|---|
| 1 — no raster-only truth | ADOPTED | §5.1 is the canonical machine-readable form |
| 3 — unit-typed quantities, no magic constants | ADOPTED | SB-POR-056, `SB-POR-T09` |
| 5 — one flag convention | ADOPTED | SB-POR-003, SB-POR-005 |
| 6 — null discipline | ADOPTED | SB-POR-050, `SB-POR-T16`; already as-built in `gascorr` |
| 8 — no bare `SW` | ADOPTED | §5.1 block 3; naming enforced across `SXO`/`SHR`/`SWE`/`SWT`/`SW_HI` |
| 9 — defaults cited or absent | ADOPTED | SB-POR-055 |
| 11 — worked examples must reproduce | ADOPTED | §6, all pinned values |
| 15 — resolution & depth snapping are logged decisions | ADOPTED | SB-POR-028, `SB-POR-T12` |

### 8.9 Tests (27) — grouped

**§6.1 of this chapter is the row-for-row disposition**: all 27 dossier tests are carried
**ADOPTED**, with their pinned values unchanged, as `SB-POR-T01 … T25`, `T14b`, `T18b`. Fourteen
further tests (`T28 … T41`) are added against as-built defects the dossier could not have known.

### 8.10 Module-boundary notes (4)

| Note | Disposition | Where |
|---|---|---|
| Ship `MTH_PHI`-equivalent **per sample** | ADOPTED | SB-POR-003 |
| Ship the corrected curves (`RHOB_HCC`, `NPHI_HCC`, `DPHI_EX`, `HI_HC`, `RHOHC_APP`, `A_FACTOR`, `B_FACTOR`) | ADOPTED | SB-POR-034; the QC-transparency advantage none of the three offers |
| Ship `SPI = PHIE_DN − PHIE_SON`, clamped `[0, SPI_MAX]` | ADOPTED | §5.3, SB-POR-002 |
| Keep Gaymard RMS / 2-3 rules as labelled comparison curves only | ADOPTED | SB-POR-023, 057 |

### 8.11 Critique disposition (25)

All 25 are recorded in the dossier as **FIXED**, and the dossier's `## Critique disposition` is
authoritative over any body text it corrects. Each is carried into this chapter as follows.

| ID | Carried as |
|---|---|
| BLK-1 Techlog equations *are* published (as images) | ADOPTED — every Techlog equation here is T3-eq, transcribed from a rendered image, never from the ASCII |
| BLK-2 Same method name, different equation | ADOPTED — F2, F3, SB-POR-013, SB-POR-014 |
| BLK-3 `VSH`/`VCL` mixed with no bridge | ADOPTED — SB-POR-006, 012, `SB-POR-T23` |
| MAJ-1 Overstated agreement envelopes | ADOPTED — `SB-POR-T01`/`T02` carry the corrected 1.5 %/4.1 %/3.0 % bounds, not tighter ones |
| MAJ-2 ESC-2 year dispute settled | ADOPTED — §7.1 states the verified 1968 citation |
| MAJ-3 "Three architectures" a false split | ADOPTED — F10 states IP ships both models and its own modules disagree |
| MAJ-4 Techlog Raymer-Hunt missing from inventory | ADOPTED — F3 carries all three renderings |
| MAJ-5 `EXC_TOOL_BLACKLIST` gates on non-matching tokens | ADOPTED — SB-POR-041, `SB-POR-T14`/`T14b` |
| MAJ-6 "38-entry tool list" merges two lists | ADOPTED — F20 states 39 (script) and 37 (doc), neither of them 38 |
| MAJ-7 `DTMA_SANDSTONE_WYLLIE` mis-cited to a Geolog table that excludes Wyllie | ADOPTED — F1 and §5.2 carry IP 56.0 vs Geolog 55.5 as a real disagreement |
| MAJ-8 IP porosity-limit family reduced to one equation | ADOPTED — F21 carries all seven limiters plus `VSILT` |
| MAJ-9 Four IP variable solvers without resolution order | ADOPTED — F18, SB-POR-051 |
| MAJ-10 Clay/shale conflation in the comparison tables | ADOPTED — F15, F16, SB-POR-006 |
| MAJ-11 A stated IP default missing from the parameter table | ADOPTED — §5.1 carries `Rho GD = 2.71 when unset` and the lithology rule |
| MAJ-12 X-05's remedy did not cover `FLD_OBSA` | ADOPTED — `SB-POR-T18b` |
| MIN-1 Geolog `phi_dh` max iterations is 50 | ADOPTED — §5.6 |
| MIN-2 Techlog line-number drift | ADOPTED — line numbers as corrected |
| MIN-3 §3.2 Δφ is −0.83 p.u. | ADOPTED — F10 carries 0.83 |
| MIN-4 OPEN-8 stakes ≈0.22 p.u. | ADOPTED — §3.1 carries 0.22 p.u. on the density term and states the 0.17 p.u. shaled figure separately |
| MIN-5 Two salinity parameters in two units on one page | ADOPTED — F23 |
| MIN-6 `RHO_DSH` validation differs between Geolog modules | ADOPTED — §5.1 records Geolog's no-default plus its validation range |
| MIN-7 Rule mis-citation (rule 10, not 1) | ADOPTED — F4 cites rule 10 |
| MIN-8 IP Raymer cell used `Phi_clay` undefined | ADOPTED — F3 states IP's clay term as the same root evaluated at the clay point |
| MIN-9 Geolog `phi_dn.lls` 13th commented-out `HAL_DSEN` branch | EVIDENCE-ONLY — noted; no requirement, since the chart branch is rejected wholesale (SB-POR-022) |
| MIN-10 Client-project provenance in the docs tree | ADOPTED as advisory — **no individual well name** and no new field, block or operator name is introduced anywhere in this chapter. §5's three T4 rows cite `project-kb` decision-record files by the repository filenames the dossier already uses, because a parameter without a resolvable source string would breach SB-POR-055; if those filenames are later renamed for provenance hygiene, these three cells follow |

### 8.12 Findings opened by the revision pass, unnumbered (4)

| Finding | Disposition | Where |
|---|---|---|
| Techlog ρ_dol 2.9 (N-D page) vs 2.87 (`QM_MineralTable`) — a fourth value, the second Techlog-internal one | ADOPTED — dolomite ρma ships `ABSENT` | §5.1, F23 |
| Techlog's N-D convergence published as an **equality** at 1 p.u. | ADOPTED — EVIDENCE-ONLY parameter + a comparison-not-equality requirement | SB-POR-050, `SB-POR-T24` |
| `Sxo = Swe^invasionfactor`, IP 1.0 vs Geolog 0.2 | ADOPTED — ships `ABSENT` | F12, SB-POR-035, `SB-POR-T25` |
| The 47.5 / 47.6 / 47.7 limestone-slowness family, two members inside one equation | ADOPTED — EVIDENCE-ONLY; parameterised, never hard-coded | §5.2, F6 |

### 8.13 Items this chapter raises that the dossier does not contain

Recorded separately so they are not mistaken for dossier findings. All arise from §3's as-built
reconnaissance, which was outside the dossier's scope.

| Finding | Requirement |
|---|---|
| `phi_son`'s `RHG` branch is the field-observed transform mislabelled, on raw `Δt`, with a Wyllie shale term | SB-POR-014, `SB-POR-T28` |
| The compaction correction inverts at its own shipped `DT_SH` 90 (**+2.30 p.u.**) | SB-POR-017, `SB-POR-T29` |
| `RHO_DSH` 2.65 matches no held source; `PHIT_SH` a factor 1.73 low | SB-POR-055, `SB-POR-T30` |
| `phi_den` and `sspw` compute `PHIT_SH` a factor 1.99 apart (**4.5 p.u. at VSH 0.5**) | SB-POR-008, `SB-POR-T30` |
| `phi_den` and `phi_dn` both write `PHIE`/`PHIT` — silent overwrite | SB-POR-004, `SB-POR-T31` |
| `curves.rs` registers no porosity family | SB-POR-004, `SB-POR-T32` |
| `sspw` still runs the gas weight `ssc` fixed on 2026-07-29 (**4.72 p.u. low in gas**) | SB-POR-059, `SB-POR-T33` |
| `sspw` declares three parameters its body never reads | SB-POR-058, `SB-POR-T34` |
| `RHO_MA` 2.645 vs 2.65 across a chained workflow | SB-POR-011, `SB-POR-T35` |
| The `phi_dn` doc string claims the average/RMS is the analytic equivalent of chart lookups | SB-POR-023, `SB-POR-T36` |
| `PHIE_FLOOR` is a compile-time `const` resolving D-16 one way | SB-POR-045, `SB-POR-T40` |
| `condflag`/`badhole` are built but not wired to porosity | SB-POR-047, 048, `SB-POR-T41` |
| `ModuleSpec` has no field for a parameter's source or tier | SB-POR-007 |

**Gate result: 260 of 260 dossier items dispositioned.** No item is left without a disposition, and
every `DEFERRED` names the chapter that owns it.
