# 12. Water saturation — requirements

**Dossier.** `docs/research_2026-08/cross_tool/saturation.md` — 1,856 lines, read in full
including §6 gaps, §4.3 discrepancy-ledger dispositions and the `## Critique disposition`
section. Where the disposition corrects the body, the disposition governs (CONTRACT §4.2); no
conflict between the two was found that the disposition had not already resolved in place.

**Evidence tiers held.** **T1** — Geolog V14 `loglan\sw_*.lls` / `.info` executable Loglan and
manifests, read first-hand by the dossier author. **T2** — IP 2025 (343/343 pages) and IP 2018
CHM ingests plus the 2026-08-06 addendum. **T3** — Techlog 2018.2 `Doc\concept\*.html` and
`Doc\image\*` equation rasters. **T4** — memory notes, petro-kb, project-kb decision records.
No T-tier claim in this chapter is unlabelled.

**Author date.** 2026-08-07.

**Requirements.** 51 (`SB-SAT-001` … `SB-SAT-051`). **P0 count.** 13.
**Parameters.** 71 rows in §5, of which **20 ship `ABSENT — ships with no default`** and 8 carry
no evidence tier at all — six being uncited values currently in SandiBumi's source, kept as
defects rather than adopted. **Acceptance tests.** 63 (`SB-SAT-T01` … `SB-SAT-T63`).

**Standing note on parameter discipline.** Every numeric value in §5 is transcribed byte-exact
from the dossier with its source string, or ships `ABSENT — ships with no default`. Three
values currently in SandiBumi's source (`Rsh = 4.0`, `φ_sh = 0.10`, and the `sw_rtc` /
`sw_imts` LRLC regression coefficients) are shipped **with no source anywhere in the dossier or
the vendor corpus**. They are recorded in §3 and §5 as defects, not adopted as defaults, and
are not reproduced here as if cited.

---

## 1. Scope and boundary

This chapter owns the computation of **water saturation from resistivity** — the model
equations, their solvers, the excess-conductivity machinery (`Qv`, `B`, `Swb`, `Cwb`, the
Clavier diffuse-layer chain), the effective↔total conversions, the `Rw`-from-salinity and
`Rw`-with-temperature correlations that feed those equations, the clamps and validity guards,
and the output nomenclature. It owns eleven cross-tool methods (Archie effective, Archie total,
Simandoux Bardon-Pied, Simandoux modified/Schlumberger, Total Shale, Indonesia, Nigeria,
Woodhouse Tar, Juhász, Waxman-Smits, Dual Water) plus Poupon-Aguilera and Poupon-Tixier, and
SandiBumi's two proprietary low-resistivity methods `sw_rtc` and `sw_imts`.

It does **not** own the following, which are declared here rather than discovered at index time.

**Seam with `MIN` — Sw computed inside the solve loop is the solver's requirement, not this
chapter's.** `multimin2.rs` exposes seven Sw models (`linear_dw`, `dual_water_nonlinear`,
`archie`, `indonesia`, `simandoux`, `juhasz`, `waxman_smits`) that run *inside* or immediately
after the mineral inversion, where φt, φe and the bound-water volume `v_bw` are solved
quantities rather than inputs (`multimin2.rs:103-131`, `:1417-1545`). The **equation forms,
parameter provenance, unit contracts and validity guards** specified in this chapter bind those
seven models — a Waxman-Smits `B` in the wrong unit is this chapter's defect wherever it is
computed. The **coupling to the inversion** — how Sw is fed back into the water/HC split, whether
the conductivity tool stays in the objective, how `v_bw` is constrained — is `MIN`'s. Two
consequences are load-bearing and are raised as requirements here because neither chapter alone
would catch them: (i) the same named model must return the same number from the standalone
module and from the solver (`SB-SAT-047`); (ii) the effective back-out differs structurally
between the two contexts, because in the solver φt ≡ φe + v_bw by construction while in a
standalone module they are independent inputs (`SB-SAT-023`).

**Seam with `POR` (porosity).** φt and φe are inputs here, never derived here. The
SSM/sand-silt-clay bound-water hard cap `Vbw ≤ 1.5 × Vcl × PhiTclay` followed by the
`PhiT = Phie + Vbw` re-set lives in a porosity module but is a **saturation-affecting** clamp —
it moves `Swb`, hence Sw, in every shaly interval (T2, dossier §2.9, §4.2). It is specified
here (`SB-SAT-042`) and cross-referenced to `POR`; `POR` owns where it fires. Ledger **D-09**
(excavation-effect exponent) and **D-11** (shale-zone porosity floor) are `POR`'s — the dossier
records D-09 as explicitly out of this domain (§4.3) and this chapter concurs.

**Seam with `CLY` (clay and shale volume).** `Vsh` and `Vcl` are inputs. The distinction that
matters here is that IP's `Vcl` is **wet** clay volume while Geolog's and Techlog's `Vsh` is
shale volume (T2 dossier §2.1 symbol dictionary), so a Juhász normalization keyed on one is not
the other. This chapter fixes which quantity each model consumes (`SB-SAT-009`); `CLY` owns
producing them and owns the dry/wet clay conversion.

**Seam with `TBD` (thin-bed and laminated).** The laminated/tensor `Rt` route feeds `SwT` via
`SwT = 1 − PhiTLam(1−SwTLam)(1−Vlam)/PhiT` (T2 dossier §1.1). That transform is `TBD`'s. What
this chapter owns is the **interlock**: IP's own manual forbids running Poupon-Aguilera or
Poupon-Tixier while a laminated Sw model is enabled, because both correct for laminations and
the pair double-counts (T2, verbatim, `A_porosity_sw.md:697-699`). That interlock is
`SB-SAT-041` and must be enforced across the seam.

**Seam with `SHR` (saturation-height and rock typing).** The Hill-Shirley-Klein / Juhász
clay-bound-water fraction `F` is shipped by **both** IP and Techlog as a **capillary-pressure**
correction (`PcCorr = Pc × F^(−0.5)`, `SwPcCorr = 1 − (1 − SwPc) × F`), not as a log-evaluation
`Swb` source (T2/T3, dossier §2.8). Those two applications belong to `SHR`. This chapter owns
only the `F` relation itself, its two unit forms, and the explicit statement that
`Swb = 1 − F` is **the dossier's own transplant** of a SCAL correction into log evaluation
(`SB-SAT-040`).

**Seam with `ENV` / `DIO`.** Temperature and salinity unit canonicalization at the import
boundary is `ENV`/`DIO`'s. What this chapter owns is that `B(T,Rw)` consumes **°C** and `Qv`
consumes **meq/mL**, and that a value arriving in °F or meq/L must fail rather than compute
(`SB-SAT-013`, `SB-SAT-014`).

**Seam with `CUT` (cutoffs and summation).** Sw cutoffs, φ·h weighting of Sw, and pay
summation are `CUT`'s. The T4 note that Sw must be φ·h-weighted and that no universal Sw cutoff
exists (`reference_shaly_sand_sw_selection`) is recorded here as context only.

**Tier-C boundary.** Omovie Sonic Saturation (US Patent 12,242,011 B2) is a Tier-C capability
under CONTRACT §2.2. Nothing in this chapter implements, approximates or reverse-engineers it,
and no requirement here is that capability under another name. It is not described further
because the dossier holds no capability-level material on it; the dossier's own Tier-C
statement (§6) records that this domain touched no Tier-C register item at all.

---

## 2. What the incumbents do — the requirement-bearing findings

Only findings that generate an obligation appear here. Findings that inform without obligating
are dispositioned `EVIDENCE-ONLY` in §8.

### 2.1 "Archie" names two different methods — 25 saturation units, silently

Geolog's `sw_arch` is **total-porosity only** and back-derives SWE by removing the shale-bound
water: `ff = A/PHIT**m`, `SWT = (ff·Rw/RT)^(1/N)`, `swtsh = 1 − PHIE/PHIT`,
`SWE = MAX((SWT−swtsh)/(1−swtsh), 0)` (**T1** `sw_arch.lls:199-216`). Its own doc block says so:
*"SW_ARCH calculates water saturation using Archies Laws, and is based on total porosity"*
(**T1** `sw_arch.lls:29`). **There is no effective-porosity Archie module in Geolog.** IP ships
both as separate menu entries (`Archie` on φe, `Archie PhiT` on φt with the E78 back-out —
**T2** `B_core_petro.md` §2.6). Techlog's `Archie` sits at the root of the saturation tree and
takes a generic "Porosity" input — the porosity system is the caller's, undeclared (**T3**
`petrophysics-archie.html`).

On the dossier's fresh-water reference case (φt 0.25, φe 0.20, Vsh 0.30, Rw 0.25, Rsh 3,
a = 1, m = n = 2, Rt = 8): IP/Techlog effective Archie gives **Sw 0.884**; Geolog `sw_arch`
gives **SWE 0.634**. **Δ = 25.0 saturation units, and hydrocarbon pore volume differs by a
factor of 3.15** (dossier §3.1). Nothing errors. Both curves are called "Archie".

**Consequence of getting it wrong:** a migration or a cross-tool audit that maps by *name*
rather than by *porosity system* is wrong by 25 su on every shaly interval. The correct mapping
is `Geolog sw_arch ≡ IP "Archie PhiT" (with E78) ≢ IP "Archie"`. Note also that Geolog's Archie
applies the dual-water bound-water term `swtsh` even though the model has no bound-water physics
— deliberately, *"to allow general application when shale volume is not zero"* (**T1**
`sw_arch.lls:29`) — so Geolog's "Archie" is shale-corrected and IP's is not.

→ `SB-SAT-002`, `SB-SAT-003`.

### 2.2 "Modified Simandoux" means the opposite thing in Geolog vs IP/Techlog — 7.3 su, plus 4.6 su from a default

Geolog's `OPT_SIM = MODIFIED` (its own label: "Bardon and Pied") implements
`1/Rt = Phie^m·Sw^n/(A·Rw) + Vsh·Sw/Rsh` (**T1** `sw_sim.lls:207-218`). That is **IP's plain
"Simandoux"** (**T2** C E63). Geolog's `OPT_SIM = SCHLUM` implements
`1/Rt = Phie^m·Sw^n/(A·Rw·(1−Vsh)) + Vsh^C·Sw/Rsh` (**T1** `sw_sim.lls:214-218`), which is
**IP's and Techlog's "Modified Simandoux"** (**T2** C E64; **T3**
`modules-quanti-saturation-simand.gif`).

The vendors are using the same adjective for two different modifications: Geolog's "Modified"
refers to multiplying the shale conductivity term by `Sw`; IP's refers to the `(1−Vcl)` divisor
(**T1** `sw_sim.lls:20 ff`).

Quantified on the reference case at Rt = 8, with `γ = Vsh/Rsh = 0.100`: the no-divisor form
gives **Sw 0.625**, the `(1−Vsh)` form gives **Sw 0.552** — **Δ = 7.3 su, HCPV +19 %**
(dossier §3.2). A user selecting "Modified Simandoux" in Geolog expecting IP's gets IP's plain
Simandoux instead.

**Compounding, and this one is pure default:** Geolog's Simandoux is the **only module in its
own family with `A` defaulted to 0.8** — every other Geolog saturation module defaults `A = 1`
(`sw_arch`, `sw_indo`, `sw_nige`, `sw_ws`, `sw_dual`, `sw_tot`; `sw_juha` has no `A` parameter
at all) (**T1** `sw_sim.info`). Re-running Bardon-Pied at `a = 0.8` gives **Sw 0.579** against
0.625 at `a = 1` — a further **4.6 su from a default nobody changes**. `sw_sim.info` *does*
carry a References block (Simandoux 1963; Bardon & Pied 1969); what it does not do is attribute
**the 0.8 itself** to either paper (dossier §3.2).

Two further naming hazards from the same module, both alias-table obligations: the user-facing
parameter is `OPT_SIM = MODIFIED | SCHLUM` but the **emitted method-flag curve** carries
`OPT_SW = 'SIM_MOD' | 'SIM_SCHL'` (**T1** `sw_sim.lls:146, 148, 210, 228`), so a migration keyed
on the output curve sees different strings than one keyed on the parameter; and the `Vsh^C`
exponent is **`SCHLUM`-branch only** — `sw_sim.lls:212, 230` use plain `VSH/RT_SH`, only
`:216, 234` apply `VSH**C`, gated by `sw_sim.info:69` (`OPT_SIM\:SCHLUM`) (**T1**).

→ `SB-SAT-001`, `SB-SAT-003`, `SB-SAT-004`, `SB-SAT-005`.

### 2.3 The Waxman-Smits `B` ×100 unit trap — 27 saturation units, the worst silent failure in the domain

Three tools, three `B` unit systems, all shipping the same physical quantity:

| Tool | Internal `B` unit | Scale at 25 °C | Source |
|---|---|---|---|
| IP | labelled `meq/ml` — **that is `Qv`'s unit; the vendor label is wrong** | ≈ 3.9 | **T2** IP2018 §3.5 |
| Geolog | `mho·cm²/meq` (in-code comment) | ≈ 0.039 | **T1** `sw_ws.lls:246, 306` |
| Techlog | `L·S/(eq·m)` (parameter table) | default **4** | **T3** `petrophysics-waxmansmits.html` |

The conversion is closed from three independent sources:
`B[L·S/(eq·m)] ≡ B[mho·mL/(m·meq)] = 100 × B[mho·cm²/meq]`, with
`Qv[eq/L] ≡ Qv[meq/mL] ≡ Qv[meq/cm³]` identical numbers (dossier §2.5). The decisive
corroboration is a single vendor shipping **both scales side by side**: Techlog's default `B`
chart (`image700.gif`, Fig. 16 of Waxman & Thomas 1974) carries the ordinate label
`B, (1/ohm·m)/(equiv./liter)` spanning 0–30, while `1972-waxmanbchartrevisedfit.png` carries
`B, mho cm² meq⁻¹` spanning 0–0.28 (**T3**; axes, ranges and isotherm labels only — **no node
values transcribed**, CONTRACT §2.1).

Quantified at φt 0.25, m = n = 2, Rw 0.25, Rt 8, a = 1, Qv 0.3 meq/mL, T 100 °C: the correct
excess term `B·Qv·Rw = 0.7277` gives **SwT 0.431**; Geolog's `b` expression pasted into IP's or
Techlog's equation form — the `/100` kept, the `×100` lost — gives excess `0.00728` and
**SwT 0.704**. **Δ = 27.2 saturation units, a 63 % relative error**, in the conservative
direction, so it **destroys pay rather than creating it** (dossier §3.3). The two conversion
sites in Geolog's own source are thirty lines apart (`sw_ws.lls:259-260` divides by 100,
`:289` multiplies by 100).

**Consequence:** this is the single most expensive number in the domain and it cannot be caught
by inspection of the output. It has to be caught by the type system.

→ `SB-SAT-012`, `SB-SAT-013`.

### 2.4 `B(T,Rw)` takes °C, not °F — and IP's own ingest recorded it wrong

The IP 2025 B-slice recorded `T` as *"degF (implied … not stated)"*. **That is wrong**, and four
independent sources say so: IP 2025's own symbol dictionary — *"`T` formation temperature in
**degrees centigrade**"* (**T2** `C_mineral_solver.md`); IP 2018 verbatim (**T2**); Geolog's
`FTEMP` unit `degc`, range 0:400, with the °C Arps constant 21.5 in the paired `cw25`
(**T1** `sw_ws.info` / `.lls`); Techlog's *"fTemp (degC)"* (**T3** `petrophysics-waxman-b.html`).
The closed form reproduces the T4 anchors exactly in °C: `B(25, 0.1) = 3.895`,
`B(100, 0.05) = 15.51` (dossier §3.4).

**Stakes if °F is fed:** at true 100 °C (212 °F), Rw 0.05, `B` = 15.51 (°C) vs 22.86 (°F),
**+47 % on B**; carried through at φt 0.25, m = n = 2, Rt 20, Qv 0.3, SwT is **0.115 vs 0.092**
— 2.3 su, ~20 % relative. Smaller than §2.3, same class: nothing errors.

The bracket grouping is settled three ways. IP's PhiSw raster `embim118` has an unmatched `)`
(ledger **D-08**); the SSM page renders it cleanly (**T2**), Techlog's `b-juhasz.png` renders it
cleanly (**T3**), and Geolog's executable source has the identical grouping (**T1**
`sw_ws.lls:259-260`). Ledger sub-item **B-OPEN-9 is RESOLVED — °C** (dossier §4.3).

→ `SB-SAT-014`, `SB-SAT-015`.

### 2.5 Juhász: two vendors derive the excess coefficient, one lets the user guess — 14 su, and the sign can flip

Geolog and Techlog are **algebraically identical**: the excess-conductivity coefficient is
derived from the shale point as `Cwsh − Cw` with `Cwsh = 1/(Rsh·φtsh^m*)` and
`Qvn = Vsh·φtsh/φt` (**T1** `sw_juha.lls:34, 42, 54, 251-253`; **T3** `image1252.gif`,
`modules-quanti-qv-qvn-qvn-equation.gif`). IP instead exposes `Bn` as a free user parameter with
a shipped placeholder of **1.0**, to be calibrated on the `Qvn`-vs-`Cwapp` crossplot (**T2**
C E69, IP2018 §3.5).

Quantified on the dossier's fresh case (Rw 0.6, Rsh 2, φtsh 0.35, Vcl 0.30, φt 0.25, Qvn 0.42,
Rt 10, m = n = 2): the derived coefficient `4.082 − 1.667 = 2.415` gives **SwT 0.722**; IP at
its default `Bn = 1.0` gives **SwT 0.862**. **Δ = 14.0 saturation units** (dossier §3.5).

**Worse — the sign can flip, and no tool warns.** At the base fresh-water case (Rw 0.25 ⇒ Cw 4.0;
Rsh 3, φtsh 0.35 ⇒ Cwsh 2.721), `Cwsh − Cw = **−1.279**`. Geolog and Techlog compute a
**negative excess conductivity** — the shale term *reduces* conductivity — while IP's default
`Bn = 1.0` is unconditionally positive. This is a real validity limit of the shale-referenced
normalization: it is physical only while the shale water is fresher (more conductive per unit
porosity) than the formation water. **The dossier records this as a mandatory guard.**

Also structural: IP normalizes on **clay** (`Vcl·PhiTclay`), Geolog and Techlog on **shale**
(`Vsh·PhiT_sh`), and IP's `Vcl` is **wet** clay volume — so the two are not interchangeable as
defaults, though IP's manual explicitly permits reparameterising to shale (**T2**, verbatim
`A_porosity_sw.md:336-339`).

**And the effective back-out is method-specific.** Geolog uses `SWE = MAX((SWT − qvn)/(1 − qvn), 0)`
— `Qvn` *itself* serves as `Swb` (**T1** `sw_juha.lls:262`, doc `:62`). This equals `1 − φe/φt`
only when `φt − φe = Vsh·φt_sh` exactly. On the dossier's fixture (Vcl 0.30, φtclay 0.35,
φt 0.25 ⇒ Qvn 0.42) against `1 − φe/φt = 0.20`, the resulting **SWE differs by tens of
saturation units while SWT matches exactly** — the purest example of the silent, method-specific
divergence class this whole chapter exists to prevent (dossier §2.6, BL-6).

→ `SB-SAT-009`, `SB-SAT-010`, `SB-SAT-023`.

### 2.6 Techlog's own two modules disagree with each other — opposite signs from the same inputs

Techlog's "Dual water" hard-codes `φtsh²` (**T3** `modules-quanti-saturation-dualw.gif`) where
its own "Juhasz" uses `φtsh^m*` (**T3** `image1252.gif`). Identical at `m* = 2`; divergent
otherwise. At φtsh 0.4 (Techlog's own default), Rsh 5, `m* = 1.6` (Jauhar's own KKT
Waxman-Smits value, **T4** `docs/workflow_standards.md`): Dual water gives `Cwsh = 1.250`,
Juhasz gives `Cwsh = 0.866` — **44 % apart**. At Rw 1.0 the coefficient `Cwsh − Cw` is
**+0.250** in Dual water and **−0.134** in Juhasz — **opposite signs, same inputs, same
product** (dossier §3.6).

Techlog's Quanti solvers are compiled, so it cannot be determined from the material held
whether the `²` is a deliberate "dual water assumes m = 2" convention or a documentation
defect. **This is escalation E-3 and it is not adjudicated here.** The obligation it generates
is narrower and safe: SandiBumi's Juhász shale-point exponent MUST be the model's own `m*`, and
the `φtsh²` convention MUST NOT be silently adopted for any model.

→ `SB-SAT-009`, §7 Escalation ESC-3.

### 2.7 Dual Water is three different models under one name, and only Geolog ships the physics

| Tool | What it actually computes | Source |
|---|---|---|
| IP | canonical algebra with **`Swb` and `Rwb` as user parameters**; `Swb` defaults to `1 − Phie/PhiT`. No diffuse-layer physics. | **T2** C E67, E78 |
| Geolog `sw_dual` | **full Clavier-Coates-Dumanoir** — `Swb` and `Cwb` derived from CEC and salinity through Debye-Hückel activity and a diffuse-layer expansion factor | **T1** `sw_dual.lls` |
| Techlog Quanti "Dual water" | `Qv = φtsh·Vsh/φt` with `φtsh` raised to a hard-coded 2 — **this is the Juhász shale-point form under the dual-water name** | **T3** `modules-quanti-saturation-dualw.gif` |
| Techlog Elan | Clavier-family with `b(T)` and a `Qv`-dependent `mdw`, default 1.8 | **T3** |

Geolog's chain, verbatim **T1** (`sw_dual.lls:365-508`): molarity `nu = RHO_W·SALW/(1000·58450)`;
Debye-Hückel activity `γ(x) = 10^(−0.5085·√x/(1 + 0.3281·4.5·√x))`; expansion
`α = MAX(1, √((γ(0.35)·0.35)/(γ(nu)·nu)))`; then the **branch that matters** —

```
if alphau > 1 :  vqhu = 0.3 * SQRT( ( 273 + FTEMP ) / 295 ) ;  vqu = alphau * vqhu
else          :  vqhu = 0.3 * 320 / ( FTEMP + 298 )          ;  vqu = vqhu
```

— then `Swb = vQ·Qv` capped at `1 − φe/φt`, `β = 2.05(T+8.5)/30.5 · (1 − β_const·e^(−2·Cw))`,
`Cwb = β/vQ`, and the excess-conductivity coefficient
`g2 = Swb·(Cwb − Cw)` (`sw_dual.lls:531-533`).

**Three findings the dossier establishes against this code, all requirement-bearing:**

1. **α does not cancel out of `g2`.** Substituting `Swb = α·vQh·Qv` and `Cwb = β/(α·vQh)` gives
   `g2 = β·Qv − α·vQh·Qv·Cw` below the cap and `g2 = (1 − φe/φt)·(β/(α·vQh) − Cw)` at or above
   it. **In both regimes α reduces `g2`; it never cancels.** Quantified at T 100 °C,
   Qv 0.3 meq/cm³, φt 0.25, φe 0.20, the α-dependent share of `g2` is **36.5 % / 21.8 % / 17.7 %
   at 25 000 / 5 000 / 3 000 ppm** — largest exactly in the fresh-water regime (dossier §2.7,
   BL-3). **`g2` MUST be implemented as `Swb·(Cwb − Cw)`, never as
   `β·Qv`.**
2. **The CEC-derived `Swb` never reaches the `SWE` back-out.** `sw_dual.lls:539` overwrites
   `SWB_U` with `(VOL_BNDWAT + VOL_SLTWAT)/PHIT`, which — because `SWB_U` was already capped at
   `1 − φe/φt` at `:452` — evaluates identically to `1 − φe/φt`. The CEC value is restored at
   `:558` **only as a QC output**, and the code comment says so. Geolog therefore takes `SWE` on
   the porosity split always. Choosing otherwise is a deliberate divergence from all three
   vendors, not a detail.
3. **A documented physical bound is shipped disabled.** The doc block states
   *"Qv is limited to be <= ( 1 / alpha * vqh )"* (`sw_dual.lls:129`) — the `Swb ≤ 1` bound in
   `Qv` terms. Both enforcing sites are **commented out** (`:449-450`, `:460-461`). Only the
   weaker `SWB_U ≤ 1 − φe/φt` cap survives, which is a **porosity-model** bound, not a
   **physical** one, and it silently absorbs an inadmissible `Qv` instead of flagging it.

`MUDBASE = WATER | OIL` is a first-class parameter in **`sw_dual` and `sw_sim` only** — no other
Geolog saturation module has it (**T1** `sw_dual.info:164`, `sw_sim.info:74`). On OBM `sw_dual`
sets `vqx = vqu`, `βx = βu` and solves `SXOT` against **`rwtemp`, not `RMF`** (`:444, :496,
:567-572`).

→ `SB-SAT-016` … `SB-SAT-022`, `SB-SAT-039`.

### 2.8 The three tools ship three different factory `Rw` values, and the best-behaved ships none

IP defaults `Rw = 0.1 ohmm` with an explicit warning — *"must be adjusted to the correct
value"* (**T2** IP2018 §3.1). Techlog states **0.03 ohm.m** with no such warning (**T3**).
Geolog has **no `Rw` default at all**: `RW`/`RWS`/`SALW` are required inputs (**T1**). At
m = n = 2, `Sw ∝ √Rw`, so an un-set `Rw` yields Sw differing by `√(0.1/0.03) = 1.83×` between
the two tools that do default it. **Neither value is appropriate for fresh formation water**,
where
9–13 kppm at formation temperature puts `Rw` nearer 0.25–0.35 ohm·m (**T4**
`reference_shaly_sand_sw_selection`) — and that band is cited here as *environment context*,
not as a default to adopt.

The same pattern repeats across the domain and is the strongest structural finding in it:
`a`, `m`, `n` have **no stated default in IP's PhiSw or SSM manual pages at all** — the
1.0/2.0/2.0 commonly quoted are Basic Log Analysis values only (**T2** IP2018 §3.1, verbatim).
`B` method defaults differ three ways: IP the Juhász closed form, Geolog `WAX_THOM`, Techlog the
"1978 Waxman B chart" (**T2**/**T1**/**T3**, dossier §3.9) — so same-named "Waxman-Smits" runs
will not agree across tools **even with identical `a/m*/n*/Rw/Qv`**.

**Geolog is the pattern to copy.** Requiring the input is not a missing feature; it is the only
posture that does not silently adjudicate.

→ `SB-SAT-031`, `SB-SAT-034`, `SB-SAT-035`, `SB-SAT-015`.

### 2.9 Every documented guard rail comes from one or two vendors, never all three

| Guard | Who documents it | Behaviour |
|---|---|---|
| `φe < 0.005 ⇒ all saturations 1` | **Geolog, all nine modules** (**T1** `sw_arch:267`, `sw_dual:596`, `sw_indo:149`, `sw_juha:302`, `sw_nige:146`, `sw_pnl:81`, `sw_sim:152`, `sw_tot:115`, `sw_ws:342`) | and **sets `VOL_UWAT = VOL_XWAT = PHIE`, not 0** (`sw_arch:272, 278`) — the pore volume is declared 100 % water, not declared empty |
| `φe = φt = 0` (coal) | Geolog `sw_arch` (**T1** `:125-133`) | all Sw = 1, volumes 0 |
| `Rt` missing ⇒ all outputs MISSING | Geolog (**T1**, history *"Jan 2012 … Make all SW missing if RT missing"*) | module exits with a message |
| variable-`m` log missing ⇒ all outputs MISSING | Geolog (**T1** `sw_ws:272-282`, `sw_dual:515-525`) | same |
| non-convergence ⇒ MISSING, never a partial | Geolog Newton-Raphson: seed 0.5, **20 iterations**, tol `|del| < 1e-5`, `sat = MAX(0, sat)` each step (**T1** `sw_sim.lls:256-271`) | `sat = MISSING` |
| `Vcl = 100 %` blows the equation up (0/0) | **Techlog Elan only** (**T3**) | *"a good idea to write a constraint to force the volume of water to be greater than about 0.5 p.u."* |
| bound-water hard cap | **IP SSM only**: `Vbw ≤ 1.5 × Vcl × PhiTclay`, then `PhiT = Phie + Vbw` (**T2** `B_core_petro.md:244-253, 839, 1060`) | the 1.5 is a hard-coded constant, not a parameter; the ingest flags it as *"easy to omit and changes `Swb`, hence Sw, in shaly intervals"* |
| unclipped diagnostic curve | Geolog (`SWT_<METHOD>` / `SWE_<METHOD>`) and Techlog (`*_UNCL`); **IP has none** | IP's comparison-curve caveat exists precisely because it lacks them |

Techlog documents **none** of the first five — its Quanti solvers are compiled and clamp
behaviour, iteration counts and null handling are unknown (dossier §1.4, escalation E-8).

**This is the "fail loud where they fail silent" opportunity in its cleanest form:** the
validity conditions are already shipped data in at least one vendor's manifests, so carrying
them as *enforced preconditions* costs care, not research.

→ `SB-SAT-027` … `SB-SAT-030`, `SB-SAT-025`, `SB-SAT-042`.

### 2.10 Nomenclature: Geolog is the only tool that never emits an ambiguous `SW`

Geolog emits `SWE`/`SWT`/`SXOE`/`SXOT`, unlimited `*_<METHOD>` diagnostics, `VOL_UWAT`/
`VOL_XWAT`, and a per-module method-flag curve `OPT_SW` (**T1**). IP's PhiSw uses `Sw`/`SwT`
while its Appendix 1 uses `SWE`/`SW` for the same quantities — an internal conflict recorded as
ledger **D-15** (**T2**). Techlog violates it outright with `SW_AR` for Archie (**T3**).

This is a naming endorsement of Geolog **only**. §2.1, §2.3, §2.7 and §2.11 each record a
Geolog defect that must not be inherited alongside its nomenclature.

→ `SB-SAT-026`.

### 2.11 The vendors' own defects, catalogued — each is a place where care alone is a differentiator

Every item below is a defect in a shipped incumbent, established from the vendor's own material.
They are listed together because CONTRACT §5.1 makes them the primary source of advantage, and
because a SandiBumi that silently inherits any of them has copied a bug rather than a feature.

1. **Geolog's 2008 `swe_irr` fix never reached `sw_ws`.** The irreducible-saturation floor is
   transformed into effective space in `sw_arch` (`:234`), `sw_juha` (`:271`) and `sw_dual`
   (`:552`), but `sw_ws.lls:302` computes `swe_irr = PHIT·SWT_IRR/PHIE`. That is **inconsistent
   with `sw_ws`'s own `SWE` map six lines earlier** (`:296`, `SWE = 1 − (φt/φe)(1 − SWT)`) — the
   two transforms are not inverses, so the floor is applied in the wrong space. Fixture:
   φt 0.30, φe 0.20, `SWT_IRR` 0.20 ⇒ Geolog's form gives **0.30** where the consistent
   effective form gives **0** (**T1**, dossier §2.2, MJ-1).
2. **`sw_ws` computes `Bmax` two different ways for `Sw` and `Sxo` in the same run** — the
   published quartic fit at `:252-255` for the unflushed zone, the compiled built-in `mm_wt74`
   at `:310` for the flushed zone (**T1**). One module, one interval, one temperature, two
   `Bmax` sources. The disagreement is **bounded at ≈3.2 %** at 25 °C: Geolog's own doc states
   `GRAVEST ≡ WAX_THOM` at 25 °C, giving `0.0015814 × 25 = 0.0395350`, while the shipped quartic
   returns `10^(−1.416565) = 0.0383196` (dossier §2.5, MJ-2). At other temperatures it is
   unknown.
3. **Geolog's `Rw` doc block is wrong by a factor of ten.** All eight `sw_*` doc blocks state
   *"minimum resistivity at 75F is **0.412** ohmm"*; **every code path sets `rw75 = 0.0412`**
   (**T1** `sw_arch.lls:49` vs `:188`, and identically in seven other modules). A separate
   commented-out message in `sw_dual` restates the correct magnitude with the temperature scale
   changed to **75C** — a second, independent documentation defect (dossier §2.10).
4. **`sw_dual`'s `Qv` doc block omits two divisors.** The doc writes
   `Qv = VSH·(RHO_SH − PHIT_SH)·CEC_DSH/PHIT`; the code is
   `QV = VSH·(RHO_SH/1000 − phit_sh)·CEC_DSH/PHIT` with `phit_sh = (PHIT − PHIE)/VSH`
   (`:125-127` vs `:415-423`). The doc omits **both** the kg/m³→g/cm³ divisor and the `/VSH`
   (**T1**, MN-9).
5. **`ricec.info` carries a `CBW`-for-`CWB` typo** in its `RI_ZERO` doc string (**T1**, MJ-3).
6. **Geolog's `PHIT_SH` validation is inconsistent across its own modules** — `0:1` in
   `sw_juha.info` and `sw_dual.info`, `0:0.4` in `qv.info` (**T1**, MN-7). The tighter range is
   the one the shared support module enforces, because it feeds the `ρdsh` chain which blows up
   as `PHIT_SH → 1`.
7. **Geolog's `sw_nige` at factory settings does not compute the Nigerian equation.** Its
   `EXP_VSH` default 2 reproduces **Indonesia-SIMPLE**; Elan's Nigerian is `EVCL = 1.4,
   MVCL = 0.0` ⇒ `Vcl^2.8` (**T1**/**T3**, dossier §3.7). Geolog's doc is honest about this, and
   it cites **the Indonesia paper** (Poupon & Leveaux 1971 Paper O) for its Nigeria module —
   naming no Nigerian-specific source. Elan's Table 27 names no reference either. **Neither
   vendor can supply the provenance** (dossier §1.4, E-5).
8. **IP's clay-bound-water `F` string is ungrammatical and byte-identical in both editions** —
   `F = 1 - [0.6425 * (Salinity ^ (-0.5) + 0.22 ] * Qv]`, one `(`, two `]`, one `[` — a
   vendor-source defect with two materially different readings (**T2**, ledger **D-07**).
9. **IP's `B fact Juhasz` default of 1.0 is labelled `meq/ml`, which is `Qv`'s unit** (**T2**),
   and the value itself is an admitted crossplot placeholder, not a recommendation.

**The ledger dispositions that follow from the cross-tool evidence** (dossier §4.3): **D-07**
RESOLVED — reading (i), via Techlog's clean rendering and the `0.084·√58.44 = 0.6421469`
unit bridge, which agrees with IP's `0.6425` to **three** significant figures (0.055 %) and
eliminates reading (ii) (which would move the constant term 36 %); **D-08** confirmed externally
by two vendors; **B-OPEN-9** RESOLVED — °C; **D-12** (the dropped `×Rw` in IP's prose) confirmed
externally by Techlog's own rendering; **D-15** closable for this domain on adopting Geolog's
scheme; **D-10** (Shell `m` 0.018 raster vs 0.019 ASCII) **remains OPEN** — Geolog has no Shell
route at all and Techlog Elan implements the *functional form* `φe^(m + mc2/φe)` but ships
`MC2 = 0.0`, the slot with no value, so **no cross-tool arbitration of the coefficient exists**.

→ `SB-SAT-024`, `SB-SAT-033`, `SB-SAT-036`, `SB-SAT-037`, `SB-SAT-040`, `SB-SAT-044`, and the
escalations in §7.

### 2.12 Where the vendors disagree, the disagreement is itself the deliverable

Collected from §2.1–§2.11, these are one-constant, three-value disagreements that **no incumbent
surfaces to its user**, and that an interpreter needs in order to defend a number:

| Quantity | IP | Geolog | Techlog | Spread |
|---|---|---|---|---|
| `Rw` factory default | 0.1 ohmm (warned) | none — required input | 0.03 ohm.m (unwarned) | 1.83× on Sw |
| Simandoux `a` default | 1.0 (BLA only) | **0.8** | 1 | 4.6 su |
| `B` method default | Juhász closed form | `WAX_THOM` | 1978 chart | ~20 % ceiling ratio on `B` |
| `B` unit | `meq/ml` (mislabel) | mho·cm²/meq | L·S/(eq·m) | **×100 — 27 su** |
| `φ_shale` default | auto-derived | input log; validation `0:1` or `0:0.4` | 0.4 v/v | 16× on `Cwsh` at m = 2 |
| `Rsh` default | picked, none | input log | 5 ohm.m | model-dependent |
| Dual-water `vQ0` | n/a | **0.3 mL/meq @ 22 °C** | **0.28 cm³/meq @ room T** | ~7 % on `Swb` |
| Nigerian half-exponent | n/a | 2 (⇒ Indonesia-SIMPLE) | Elan 1.4 | different equation |
| Juhász coefficient | free `Bn` = 1.0 | `Cwsh − Cw` | `Cwsh − Cw` | 14 su, sign flip |

Note the `vQ0` row: **both values are cited to the same paper** — Geolog's `sw_dual.info:142-144`
names Clavier, Coates & Dumanoir 1984, which is where Elan's 0.28 also comes from. That reframes
the conflict as two *readings* of one paper, not a vendor invention on either side (dossier
§4.2, E-4).

→ `SB-SAT-044`.

### 2.13 Provenance is structural in Geolog's manifests and absent from the other two

Geolog ships published references **inside the module manifests** (`.info` `DESCRIPTION_DETAIL`
References blocks) for every saturation model: Archie 1942 Trans. AIME 146:54-62; Simandoux 1963
Revue de l'IFP with the SPWLA "Shaly Sand" Reprint Volume 1982 translation; Bardon & Pied 1969
SPWLA 10th Paper Z; Poupon & Leveaux 1971 SPWLA 12th Paper O; Woodhouse 1976 SPWLA 17th Paper T;
Waxman & Smits 1968 SPEJ; Waxman & Thomas 1974 SPEJ; Juhász 1979 SPWLA 20th Paper AA and 1981
SPWLA 22nd; Gravestock & Alexander 1989; Clavier, Coates & Dumanoir 1984 SPEJ 153-168;
Skoog & West *Fundamentals of Analytical Chemistry* 4th ed. for the activity coefficient;
Schlumberger *Log Interpretation Principles/Applications* 1989; Keelan & McGinley 1979;
Bateman & Konen 1977; Western Atlas Charts 1994 p. 27 for `Rw` from salinity (**T1**, dossier
§1.2). It also states the **Worthington 1985 type classification per module** — `sw_indo` type 4,
`sw_sim`/`sw_ws`/`sw_juha`/`sw_dual`/`sw_tot` type 2 (**T1**).

**But it does not carry the reference through to the answer.** No vendor emits, alongside a
computed Sw curve, the parameter values used, their sources, and the paper each traces to. A
parameter that carries its paper through the computation into the deliverable is a claim no
incumbent can make (CONTRACT §5.4).

One caution the dossier surfaces and does **not** adjudicate: IP attributes the clay-bound-water
relation to *"Hill, Shirley and Klein 1979 (SPWLA 20th … Paper AA — 'The Central Role of Qv and
Formation Water Salinity …')"* while Geolog attributes a paper with **that exact title** to
**Juhász** at the same symposium, same year, same paper letter (**T2** `E_shf_rocktyping.json:345`
vs **T1** `sw_ws.info:87-88`, `sw_juha.info:77-78`). Techlog cites Juhász 1979 *The Log Analyst*
p 3–14 (**T3**). Same paper, contested authorship. Both readings ship; neither is chosen.

→ `SB-SAT-043`, `SB-SAT-049`, §7 Escalation ESC-1.

---

## 3. SandiBumi as-built

Written from source. Every claim carries `file.rs:line`. The repository was read-only for this
task; nothing under `D:\XX. SandiBumi` was modified except this chapter file.

### 3.1 Two independent saturation engines, and they do not agree with each other

SandiBumi computes Sw in **two places that share no code**:

1. **Deterministic modules** — registered at `modules.rs:363-367` and dispatched at
   `modules.rs:441-453`: `sw_arch`, `sw_indo`, `sw_sim` (Geolog-derived) plus `sw_rtc` and
   `sw_imts` (`lrlc.rs`, Jauhar's own low-resistivity methods). Five registered saturation
   modules.
2. **The mineral solver** — `multimin2.rs`, seven `SwModel` variants (`multimin2.rs:103-123`):
   `LinearDw` (default, in-inversion), `DualWaterNonlinear`, `Archie`, `Indonesia`, `Simandoux`,
   `Juhasz`, `WaxmanSmits`. All but `LinearDw` are post-solve (`multimin2.rs:125-131`), replacing
   the inversion's water/HC split with a closed-form Sw (`multimin2.rs:1417-1545`).

**The two engines disagree about what "Simandoux" means, and the disagreement is exactly the
7.3-saturation-unit trap of §2.2 — reproduced inside one product.**

- `modules.rs:2279-2283`: `OPT_SIM` default `MODIFIED` ⇒ `g1 = 1/(ff·rw)`, `g2 = vs/rt_sh` —
  the **Bardon-Pied** form, no `(1−Vsh)` divisor. Correct, and correctly matched to Geolog's
  own `MODIFIED` label (`modules.rs:2188`).
- `multimin2.rs:167-175`: `sw_simandoux` computes
  `coef_sand = phie^m/(a·rw·(1 − vsh))` — the **`(1−Vsh)` / Schlumberger** form — while its doc
  comment at `multimin2.rs:164` and the enum comment at `multimin2.rs:115` both label it
  **"Modified Simandoux (Bardon-Pied)"**. The label and the equation are the two different
  methods of §2.2.

So a user who runs `sw_sim` at its default and `SwModel::Simandoux` in the solver on the same
interval gets **0.625 vs 0.552** on the dossier's reference case — 7.3 su — from two things both
called Simandoux in one application. Neither path errors.

**Status: `PRESENT-DIVERGENT`** — `multimin2.rs:115, 164, 174`.

### 3.2 Method coverage against the eleven cross-tool methods

| Method (§2 / dossier §5.1) | Deterministic module | Solver | Status |
|---|---|---|---|
| `archie_effective` | — | — | **`ABSENT`** — both engines are total-porosity only (`modules.rs:2058-2059`; `multimin2.rs:267-273`) |
| `archie_total` | `sw_arch` | `SwModel::Archie` | `PRESENT-OK` |
| `simandoux_bardon_pied` | `sw_sim` `OPT_SIM=MODIFIED` | — | `PARTIAL` — module only; the solver has no Bardon-Pied form |
| `simandoux_modified_slb` | `sw_sim` `OPT_SIM=SCHLUMBERGER` | `SwModel::Simandoux` | `PRESENT-DIVERGENT` — mislabelled in the solver (§3.1) |
| `total_shale` (Schlumberger, `n ≡ 2`) | — | — | **`ABSENT`** |
| `indonesia` | `sw_indo`, three variants (`modules.rs:2159-2163`) | `SwModel::Indonesia` (`multimin2.rs:154`) | `PARTIAL` — module has FULL/SIMPLE/TAR_SAND; the solver hard-codes `Vsh^(1−Vsh/2)`, i.e. `k = 1` only |
| `woodhouse_tar` | `sw_indo` `OPT_INDO=TAR_SAND` | — | `PARTIAL` — present as an Indonesia variant, not aliased or cited to Woodhouse 1976 |
| `nigeria` | — | — | **`ABSENT`** |
| `juhasz` | — | `SwModel::Juhasz` (`multimin2.rs:282-298`) | `PARTIAL` — solver only |
| `waxman_smits` | — | `SwModel::WaxmanSmits` (`multimin2.rs:307-313`) | `PARTIAL` — solver only |
| `dual_water_cec` | — | `SwModel::DualWaterNonlinear` (`multimin2.rs:213-221`) | `PRESENT-DIVERGENT` — see §3.5 |
| `dual_water_simple` (`Swb`/`Rwb` as parameters) | — | — | **`ABSENT`** |
| `poupon_aguilera`, `poupon_tixier` | — | — | **`ABSENT`** |
| LRLC `sw_rtc`, `sw_imts` | `lrlc.rs:118`, `lrlc.rs:225` | — | `PRESENT-UNVERIFIED` — SandiBumi-only methods, no vendor counterpart; see §3.7 |

**Six of the thirteen named methods have no implementation at all**, and four more exist in only
one of the two engines — so the choice of engine silently changes which methods are available.

### 3.3 `Rw` resolution — the strongest as-built result in the domain

`modules.rs:1951-1983` implements all four `OPT_RW` branches **with each temperature conversion
bound to its own branch**, which is precisely the silent-error the dossier warns about
(§5.2 implementation note):

- `MEASURED` — `RWS·(RWT + 21.5)/(FTEMP + 21.5)`, Arps °C from the user's own `RWT`
  (`modules.rs:1958`).
- `SALINITY` above 39 161 ppm — Kennedy: `x = SALW/10000 − 29.46518957`,
  `rw75 = 1/(24.30853 − 0.0364x − 0.02922x²)`, then Arps **°F** from 75 °F,
  `·(75 + 6.77)/((1.8·FTEMP + 32) + 6.77)` (`modules.rs:1967-1975`).
- `SALINITY` at or below 39 161 ppm — Bateman-Konen `rw75 = 0.0123 + 3647.5/SALW^0.955`, then
  Arps **°C** from **23.9 °C** (`modules.rs:1977-1978`).
- Kennedy salinity cap `> 275 000 ppm ⇒ rw75 = 0.0412` (`modules.rs:1972-1973`) — **the code
  value, not the vendor doc's erroneous 0.412** (§2.11 item 3).

Every constant matches the dossier's transcription byte-for-byte, the switch is at 39161 (not at
the rounded resistivity), and the two Arps forms are not cross-wired.

**Status: `PRESENT-OK`** for the correlation set — but `PRESENT-UNVERIFIED` as shipped: there is
no test for branch continuity at the switch, no test that the two Arps conversions differ on the
same input, and no source comment recording that `0.0412` deliberately contradicts Geolog's own
documentation. A future reader "fixing" `0.0412` to match the vendor doc would introduce a ×10
`Rw` error with nothing to catch it (dossier test 24).

**A separate divergence sits on top of it:** `rw_args()` ships hard defaults —
`RW = 0.1 ohmm` (`modules.rs:1943`), `RWS = 0.1`, `RWT = 24.0 degC` (`modules.rs:1945`),
`SALW = 20 000 ppm` (`modules.rs:1946`) — and `lrlc.rs:93` and `lrlc.rs:200` ship `RW = 0.3`.
`0.1` is the IP default the dossier explicitly rejects (§2.8); `0.3` is uncited. At m = n = 2 the
0.1-vs-0.3 spread inside SandiBumi's own modules is `√3 = 1.73×` on Sw.

**Status: `PRESENT-DIVERGENT`** — `modules.rs:1943`, `lrlc.rs:93`, `lrlc.rs:200`.

### 3.4 Waxman-Smits `B` — correct formula, correct clamp, **no unit type**

`multimin2.rs:326-336`:

```rust
pub fn waxman_b(t_c: f64, rw: f64) -> f64 {
    let num = -1.28 + 0.225 * t_c - 0.0004059 * t_c * t_c;
    if !(rw > 0.0) { return num.max(0.0); }
    let den = 1.0 + (0.045 * t_c - 0.27) * rw.powf(1.23);
    if !(den > 0.0) { return num.max(0.0); }
    (num / den).max(0.0)
}
```

Verified against the dossier at source: the bracket grouping is the **balanced** form that
resolves ledger D-08 (`multimin2.rs:331`); the temperature is **°C**, resolving B-OPEN-9
(`multimin2.rs:319`); the result is documented as **`mho·mL/(m·meq)`** — the `L·S/(eq·m)` scale
the dossier makes canonical — and is paired with `Qv` in meq/mL and `Cw` in mho/m so that
`B·Qv` lands in mho/m (`multimin2.rs:320-321`, consumed at `multimin2.rs:311` and
`lrlc.rs:276`). `B` is clamped `≥ 0`, with the reason recorded (`multimin2.rs:321`). The doc
string's attribution to Techlog's "1972 Waxman B chart original fit" is **correct** under the
dossier's corrected three-image mapping (`b-juhasz.png` ← 1972 original fit, dossier §2.5, MJ-8).
`multimin2.rs:3833-3856` already asserts the two T4 anchors (3.89520, 15.5144), the `B ≥ 0`
clamp below ~6 °C, and monotonicity in T and Rw.

**Status: `PRESENT-OK`** on the equation, units-as-documented, clamp and test coverage.

**But the unit is a comment, not a type.** `B` is a bare `f64` at every call site
(`multimin2.rs:307`, `:1509`; `lrlc.rs:64-67`, `:267`), as is `Qv` (`multimin2.rs:307`,
`lrlc.rs:40`) and the temperature (`multimin2.rs:326`). Nothing prevents a `mho·cm²/meq` value
reaching `sw_waxman_smits`, and nothing prevents a °F number reaching `waxman_b` — the two
failures worth 27 su and 2.3 su respectively (§2.3, §2.4). There is a second, independent
`juhasz_b` implementation at `lrlc.rs:64-67` with the same formula and **no `≥ 0` clamp of its
own** (the clamp is applied by the caller at `lrlc.rs:267`), so the invariant lives in two
places.

**Status: `PRESENT-DIVERGENT`** on unit safety — `multimin2.rs:307`, `lrlc.rs:64`.

### 3.5 Dual water — the correct algebraic form, on a divergent physical chain

**What is right, and it is the finding the dossier worked hardest for.**
`multimin2.rs:213-221` implements the excess-conductivity coefficient as
`swb * (cwb - cw)` — **not** as `β·Qv`. That is exactly the §2.7(1) mandate, and it means α
correctly fails to cancel out of `g2`. `sw_cond_root` (`multimin2.rs:230-261`) solves
`cw·Swt^n + lin·Swt^(n−1) − a·Ct/φt^m = 0` with a closed quadratic at `n = 2` and monotone
bisection otherwise, and it is shared with Juhász so the two cannot drift.
**Status: `PRESENT-OK`.**

**Three quantified divergences in the chain that feeds it.**

**(a) The diffuse-layer expansion factor drops the Debye-Hückel activity ratio.**
`multimin2.rs:557-563`:

```rust
fn alpha_expansion(salinity: f64) -> f64 {
    if salinity > 0.0 && salinity < 20_455.0 { (20_455.0 / salinity).sqrt().min(5.0) } else { 1.0 }
}
```

Geolog computes `α = MAX(1, √((γ(0.35)·0.35)/(γ(n)·n)))` with
`γ(x) = 10^(−0.5085√x/(1 + 0.3281·4.5·√x))` (**T1** `sw_dual.lls:365-372`). SandiBumi's form is
that expression **with both activity coefficients set to 1** — the 20 455 ppm threshold is
exactly Geolog's `n = 0.35 mol/L` reference converted through `n = ppm/58 450`
(20 455/58 450 = 0.34995). The divergence is therefore the γ ratio alone. Evaluating both
(and reproducing the dossier's §2.7 α column exactly as a check on the arithmetic):

| Salinity | Geolog α (with γ) | SandiBumi α | Divergence |
|---|---|---|---|
| 5 000 ppm | 1.8949 | 2.0226 | **+6.7 %** |
| 3 000 ppm | 2.3976 | 2.6112 | **+8.9 %** |

α scales `Swb` directly and enters `g2` at 17.7–36.5 % (§2.7), so this is a first-order error in
the fresh-water regime.
**Status: `PRESENT-DIVERGENT`** — `multimin2.rs:557-563`.

**(b) `vQ` uses the saline temperature form even when the layer has expanded.**
`multimin2.rs:604-605`:

```rust
fn bndwat_multiplier(cec: f64, rho_gcc: f64, t_c: f64, alpha: f64) -> f64 {
    alpha * 96.0 * cec * rho_gcc / (t_c + 298.0)
}
```

Note first that the coefficient is **`96.0` with `CEC` in meq/g and ρ in g/cc**, not the
`0.096` recorded in local memory — `0.096` would be the coefficient for ρ in kg/m³, and the code
is self-consistent as written. `96/(T + 298)` is Clavier's `vQh` in mL/meq: at T = 22 °C it
returns 0.30, reproducing Geolog's `vqh = 0.3·320/(FTEMP + 298)` exactly.

**The defect is that Geolog uses that form only on the `α = 1` branch.** When `α > 1` Geolog
switches to `vqhu = 0.3·√((273 + FTEMP)/295)` (**T1** `sw_dual.lls:630`). SandiBumi applies the
saline form unconditionally and multiplies by α. At T = 100 °C: Geolog's fresh-branch
`vQh = 0.3·√(373/295) = 0.33732`; SandiBumi's `96/398 = 0.24121` — **28.5 % low**. Cross-check
against the dossier's own §2.7 table: at 3 000 ppm it reports `vQ = 0.8087`, and
0.8087/2.3976 = 0.33732 ✓, confirming Geolog takes the square-root branch there.

`Swb = vQ·Qv` is therefore 28.5 % low in the fresh branch. Because `Swb·Cwb = β·Qv` is
α- and `vQ`-invariant while the subtracted `Swb·Cw` term is not, the error propagates into `g2`
through exactly the term the dossier proved does not cancel.
**Status: `PRESENT-DIVERGENT`** — `multimin2.rs:604-605`.

**(c) The `β` dilution factor is absent.** `multimin2.rs:580` computes
`cbw = 0.0007·(t_c + 8.5)·(t_c + 298)`, then `cbw_u = cbw/alpha_u` (`multimin2.rs:592`). That
collapsed constant is **correct**: Geolog's `β/vQ` chain expands to
`(2.05(T+8.5)/30.5)/(0.3·320/(T+298))/α = 0.00070013·(T+8.5)(T+298)/α`, matching `0.0007` to
four figures. What is missing is Geolog's salinity dilution of β itself —
`βu = βu·(1 − BETA_CONST·e^(−2·Cw))`, `BETA_CONST` default 1 (**T1** `sw_dual.lls:638`). At
Cw = 4 mho/m (Rw 0.25) the factor is 0.99966 and the omission is immaterial; at Cw = 1 mho/m
(Rw 1.0, very fresh) it is `1 − e^(−2) = 0.8647`, so SandiBumi's `Cwb` is **15.7 % high**.
**Status: `ABSENT`** — no `β_const` parameter exists.

**(d) Neither the documented `Qv ≤ 1/vQ` validity bound nor an explicit `Swb ≤ 1 − φe/φt` cap is
present.** `multimin2.rs:1465` clamps `swb` to `[0,1]` only. In the solver φt ≡ φe + v_bw by
construction (`multimin2.rs:1461`), so `Swb ≤ 1 − φe/φt` holds structurally and the omission is
harmless *there*; but no diagnostic reports an inadmissible `Qv`, so the physically impossible
state Geolog's doc block warns about is absorbed silently rather than flagged.
**Status: `ABSENT`.**

### 3.6 Juhász — equation exactly right, guard entirely missing

`multimin2.rs:282-298` implements `qvn = clamp(vsh·φ_sh/φt, 0, 1)`,
`cwsh = 1/(rsh·φ_sh^m)` — using the model's own `m`, **not** Techlog's hard-coded `φtsh²`
(§2.6) — and the coefficient `qvn·(cwsh − cw)` with `a = 1`. That is Geolog and Techlog's
Juhász exactly. **Status: `PRESENT-OK`** on the equation.

**But there is no guard on `cwsh − cw < 0`.** When the formation water is fresher than the shale
water, `lin` goes negative; `sw_cond_root` handles it numerically (`multimin2.rs:244-250`) and
returns a lower Sw with **no flag, no warning and no null**. On the dossier's own base
fresh-water case (Rw 0.25, Rsh 3, φtsh 0.35, m 2) the coefficient is **−1.279** and the model is
outside its
validity. This is the mandatory guard of §2.5.
**Status: `ABSENT`** — `multimin2.rs:297`.

**And the effective back-out uses the porosity split, not `Qvn`.** `multimin2.rs:1497` returns
`(swt·φt − v_bw)/φe`, i.e. `Swb = v_bw/φt`, where Geolog and Techlog use `Swb = Qvn`
(§2.5, dossier BL-6). In the solver this is a **defensible** choice — `v_bw` is a solved
quantity and φt ≡ φe + v_bw — and the source comment says so (`multimin2.rs:1487-1488`). It is
nonetheless a divergence from both vendors, it is undocumented outside that comment, and it
guarantees that a standalone Juhász module (which does not yet exist) built to the vendor rule
would disagree with the solver on `SWE` while agreeing exactly on `SWT`.
**Status: `PRESENT-DIVERGENT`** — `multimin2.rs:1497`.

### 3.7 The two uncited shale parameters, and the LRLC coefficients

`multimin2.rs:377-385` ships `default_rsh() = 4.0` and `default_phit_sh() = 0.10`, mirrored in
the UI at `src/ui/multiminDialog.ts:430` and `:818-820`. **Neither value appears anywhere in the
dossier, in any vendor's material, or in any project record.** Techlog's `Res_shale` default is
5 and its `Porosity shale` default is 0.4 (**T3**); Geolog takes both as input logs with no
default (**T1**); IP has the interpreter pick them (**T2**). SandiBumi's 4.0 and 0.10 match none
of them.

`φ_sh` is the more expensive of the two because Juhász consumes it twice, once linearly in
`Qvn = Vsh·φ_sh/φt` and once as `φ_sh^(−m)` in `Cwsh = 1/(Rsh·φ_sh^m)`. At m = 2 and Rsh 4:
`Cwsh(0.10) = 25.0 mho/m` against `Cwsh(0.40) = 1.5625 mho/m` — **16× apart** — while `Qvn`
differs 4×. Nothing in the product tells the user that number was not measured.

**Status: `PRESENT-DIVERGENT`** — `multimin2.rs:377-385`; **this is a CONTRACT §2 parameter
-discipline violation in shipped code**, not merely a divergence from a vendor.

The LRLC modules are handled differently and better. `sw_rtc` ships `A_CAP 0.45`, `B_QV 0.0057`,
`C0 −0.0071`, `RSF 2.25` (`lrlc.rs:96-99`) and `sw_imts` ships `S_FACTOR 0.5`
(`lrlc.rs:205`) — and both doc strings say, in capitals, that these are **one study's
calibration from one field** and **a placeholder** respectively, that a foreign calibration
*"does not announce itself: it yields a smooth, plausible Sw that is simply wrong"*, and they
name the in-product calibration route that replaces them (`lrlc.rs:83-90`, `:191-197`). Both
calibrators exist and are tested (`run_rtc_fit`, `run_s_factor_fit`; `lrlc.rs:1534-2126`).
That is the right shape — an honest placeholder plus a way to replace it — and it is the model
`Rsh`/`φ_sh` should follow.
**Status: `PRESENT-UNVERIFIED`** for `sw_rtc`/`sw_imts` themselves — the calibration paths are
tested, the forward models are tested against synthetic rock they generated (`lrlc.rs:1399-1441`),
but neither method has a vendor or literature counterpart to check against.

### 3.8 Solver guards, clamps and outputs

**Newton-Raphson, exactly Geolog's.** `modules.rs:2218-2230`: seed 0.5, 20 iterations,
`|del| < 0.00001`, `sat = max(sat − del, 0)` each step, **`MISSING` on non-convergence**. This
is `sw_sim.lls:256-271` transcribed correctly, including the refusal to return a partial answer.
**Status: `PRESENT-OK`.**

**`sw_imts` does the opposite.** `lrlc.rs:271-287` iterates up to 100 times and breaks on
convergence, but on falling out of the loop it keeps the **last iterate** — only a NaN from a
non-positive denominator is caught (`lrlc.rs:277-279`, `:288-290`). A non-converged sample is
therefore emitted as a plausible number. This is the failure mode dossier test 15 exists to
prevent, present in SandiBumi's own method.
**Status: `PRESENT-DIVERGENT`** — `lrlc.rs:271-290`.

**Low-porosity and coal guards are right, including the volume detail.** `φe < 0.005 ⇒ Sw = 1`
with `VOL_UWAT = φe` — not 0 — at `modules.rs:2074-2079` (Archie), `:2140-2144` (Indonesia) and
`:2247-2251` (Simandoux); `φt = 0 ⇒ all Sw = 1, VOL_UWAT = 0` at `modules.rs:2036-2042`. The
`φt = 0` guard is keyed on total porosity alone so it fires even when `PHIE` is absent, with the
reason recorded (`modules.rs:2032-2035`). `Rt ≤ 0` and missing `Rt`/`Rw` drop the sample to
missing in all three modules (`modules.rs:2050`, `:2150`, `:2258`).
**Status: `PRESENT-OK`.**

**The `swe_irr` transform is the correct one.** `modules.rs:2071` computes
`swe_irr = max((swt_irr − swtsh)/(1 − swtsh), 0)` with the `swtsh ≥ 1` degeneracy sent to 0 —
Geolog's 2008 fix, **not** the broken `SWT_IRR·φt/φe` that `sw_ws.lls:302` still ships
(§2.11 item 1). **Status: `PRESENT-OK`**, and worth protecting with a test that names the
Geolog form as forbidden.

**`C` is applied to the Schlumberger branch only** (`modules.rs:2280` plain `vs/rt_sh`,
`:2282` `vs.powf(c)/rt_sh`) — matching the dossier's corrected §5.1.
**Status: `PRESENT-OK`**, except that the validation range is `0.5:2.0` (`modules.rs:2192`)
where Geolog's is `1:2` (**T1** `sw_sim.info`). `C < 1` is supported by no source held.
**Status: `PRESENT-DIVERGENT`** on the range.

**`Vsh ≥ 1` in the Schlumberger branch is silently set to all-water** (`modules.rs:2265-2270`)
rather than flagged. Elan documents this exact case as an equation blow-up requiring a
constraint (§2.9). The current behaviour is defensible arithmetic and an undefensible silence.
**Status: `PRESENT-DIVERGENT`** — fails silent where the requirement is to fail loud.

**Outputs.** Unclipped diagnostics are emitted as `SWT_ARCH` (`modules.rs:2001`), `SWE_INDO`
(`:2109`) and `SWE_SIM` (`:2201`), which satisfies the unclipped-curve requirement for the
*computed* quantity — but `sw_arch` emits no unclipped **effective** curve, only the unclipped
total, so the Archie `SWE` has no diagnostic counterpart (`modules.rs:2001-2004`).
`VOL_XWAT` is emitted nowhere. **No module emits a method-flag curve** (Geolog's `OPT_SW`
pattern). **No standalone module computes `Sxo` or `Sxo`-derived volumes at all**; `SXOT` exists
only as a solver output (`multimin2.rs:1670-1674`), and `MUDBASE` exists nowhere in the
codebase. Mnemonics are correctly `SWE`/`SWT`-suffixed throughout — **no bare `SW` is emitted**
(D-15 satisfied in practice, unenforced by any test).
**Status: `PARTIAL`** on outputs; **`ABSENT`** for `Sxo`, `VOL_XWAT`, `MUDBASE` and method flags.

### 3.9 `Qv` and CEC handling

`lrlc.rs:40-48` computes `Qv = CEC·ρg·(1 − φt)/(100·φt)` with `CEC` in **meq/100 g**, the `/100`
converting to meq/g so the result is meq/cm³ ≡ meq/mL. That is Techlog's
`QV = CEC·ρg·(1−Φ)/Φ` (**T3**) with the unit conversion made explicit, and it is **shared
deliberately** between the `sw_rtc` module and its calibration fit so the two cannot disagree
about what `Qv` means (`lrlc.rs:37-39`). **Status: `PRESENT-OK`.**

The solver builds `Qv` differently — `Σ v_clay·CEC·ρ_clay` over solved clay volumes, divided by
the zone's φt (`multimin2.rs:1432-1442`, `:1508`), with `CEC` in **meq/g** there
(`multimin2.rs:65`). Two `Qv` constructions in one product with two different `CEC` units and no
shared type. Neither is wrong; nothing enforces that they stay consistent.
**Status: `PRESENT-DIVERGENT`** on unit safety.

**Geolog's shale route to `Qv` is absent** — `QV = VSH·(RHO_SH/1000 − φt_sh)·CEC_DSH/φt` with
the `ρdsh` dry-shale chain (**T1** `qv.info`) has no implementation, so a user with a shale-based
`Qv` and no core has no path. **Status: `ABSENT`.**

### 3.10 Test coverage

Saturation-specific tests found: `modules.rs:3791-3944` — eight tests covering the Archie clean
sand, the zero-porosity and non-positive-`Rt` guards on all three modules, the Schlumberger
pure-shale case, Indonesia FULL vs SIMPLE, and `sw_sim` against the closed-form quadratic.
`multimin2.rs:3445-4041` — eleven tests covering hand-computed points for all five closed-form
models, `waxman_b` against the T4 anchors, non-physical input rejection, and post-solve recovery
of a known Sw for Indonesia, dual water and Waxman-Smits. `lrlc.rs:1399-2126` — the IMTS forward
model, `juhasz_b` monotonicity, and an extensive calibration-fit suite.

**What is untested:** every cross-tool equivalence (dossier tests 1–5, 4b, 4c), every unit trap
(6–10b), the negative-Juhász-coefficient guard (11), non-convergence-returns-null (15), the
`Vsh → 1` flag (16), the `Swb` cap (17), clipped/unclipped emission (18), the per-model back-out
(18b), α-dependence of `g2` (18c), the SSM cap (18d), the `Qv` bound flag (18e), the `φe < 0.005`
volume detail (18f), `MUDBASE` scoping (18g), the no-bare-`SW` rule (19), the
every-default-has-a-source rule (20), and all four `Rw` correlation tests (22–25).
**No test asserts a parameter's source string, because no parameter carries one.**

### 3.11 As-built summary

| | Count |
|---|---|
| `PRESENT-OK` | 9 |
| `PRESENT-DIVERGENT` | 11 |
| `PARTIAL` | 6 |
| `PRESENT-UNVERIFIED` | 2 |
| `ABSENT` | 12 |

The eleven `PRESENT-DIVERGENT` findings are the expensive ones, and three of them —
the Simandoux label inversion inside one product (§3.1), the two uncited shale parameters
(§3.7), and the dual-water α/`vQ` chain (§3.5) — produce a wrong number on a run that completes
cleanly.

---

## 4. Requirements

### 4.1 Model identity and naming

#### SB-SAT-001 — Name every saturation model by its equation, never by a vendor adjective [P0] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST identify every saturation model by a stable identifier that
names its *equation*, not a vendor's adjective for it: `archie_effective`, `archie_total`,
`simandoux_bardon_pied`, `simandoux_modified_slb`, `total_shale`, `indonesia`, `nigeria`,
`woodhouse_tar`, `juhasz`, `waxman_smits`, `dual_water_simple`, `dual_water_cec`,
`poupon_aguilera`, `poupon_tixier`. SandiBumi MUST NOT expose a model selector whose option
strings are `Modified` / `Simandoux` / `Modified Simandoux` without the equation-naming suffix.
Every internal function, doc comment and enum variant MUST use the same identifier as the user
-facing name; a doc comment naming a different method than the code implements is a defect.

**Rationale.** "Modified" denotes two different modifications in two vendors — Geolog's refers
to the `Vsh·Sw` shale term, IP's and Techlog's to the `(1−Vcl)` divisor (**T1**
`sw_sim.lls:20 ff`; **T2** C E63/E64; **T3** `modules-quanti-saturation-simand.gif`). Selecting
by adjective costs **7.3 saturation units and +19 % HCPV** (dossier §3.2). SandiBumi currently
reproduces the confusion internally: `multimin2.rs:115, 164` label the `(1−Vsh)` equation
"Bardon-Pied", which is the other method.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2188` exposes `MODIFIED | SCHLUMBERGER`;
`multimin2.rs:115, 164` mislabel the Schlumberger form as Bardon-Pied. The two engines therefore
compute different equations under the same word (§3.1).

**Verified by.** SB-SAT-T01, SB-SAT-T02, SB-SAT-T30

---

#### SB-SAT-002 — Ship effective and total Archie as separate named methods [P0] [status: ABSENT]

**Requirement.** SandiBumi MUST ship `archie_effective` (`Sw = (a·Rw/(Rt·φe^m))^(1/n)`) and
`archie_total` (the same on φt, followed by the model's effective back-out) as two distinct,
separately selectable methods. SandiBumi MUST NOT expose a method named `archie` with an
undeclared porosity system.

**Rationale.** Geolog ships total only, IP ships both, Techlog ships one that takes whatever
porosity the caller supplies (**T1** `sw_arch.lls:29`; **T2** `B_core_petro.md` §2.6; **T3**
`petrophysics-archie.html`). On the reference case the two answer **0.884 vs 0.634 — 25.0
saturation units, HCPV 3.15× apart** (dossier §3.1). This is the largest single cross-tool trap
in the domain and it is invisible in the output.

**As-built.** `ABSENT` — both engines are total-only (`modules.rs:2058-2059`;
`multimin2.rs:267-273`). There is no effective-porosity Archie in SandiBumi.

**Verified by.** SB-SAT-T03, SB-SAT-T04

---

#### SB-SAT-003 — Ship a vendor alias table and resolve imports through it [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST ship a machine-readable alias table mapping each incumbent's
model name — **and each incumbent's emitted method-flag value** — to a SandiBumi model
identifier. It MUST include at minimum: Geolog `MODIFIED`/`SIM_MOD` → `simandoux_bardon_pied`;
Geolog `SCHLUM`/`SIM_SCHL` → `simandoux_modified_slb`; Geolog `sw_tot`/`TOTAL_SH` and Techlog
`total-shale`/`modified-total-shale` → `total_shale`; Geolog `sw_arch` → `archie_total`; IP
`Archie` → `archie_effective`; IP `Archie PhiT` → `archie_total`; IP `Simandoux` →
`simandoux_bardon_pied`; IP/Techlog `Modified Simandoux` → `simandoux_modified_slb`; IP
`"Juhasz (Waxman-Smits)"` → `juhasz`; IP `Woodhouse Tar` and Geolog `TAR_SAND` →
`woodhouse_tar`; Techlog Quanti `Dual water` → `juhasz` (**not** `dual_water_*`); IP `Poupon` →
`poupon_aguilera`. Importing a foreign parameter set MUST resolve through this table and MUST
fail rather than guess on an unmapped name.

**Rationale.** The parameter- and flag-name spellings differ *within* one vendor: Geolog's
user-facing `OPT_SIM` is `MODIFIED|SCHLUM` while the emitted flag curve is
`SIM_MOD|SIM_SCHL` (**T1** `sw_sim.lls:146, 148, 210, 228`), so a migration keyed on one sees
different strings than one keyed on the other (dossier MN-2). IP's menu string is
"Juhasz (Waxman-Smits)", and the same menu carries a separate plain "Waxman-Smits" with a
different equation (**T2**, dossier MN-11). Techlog's "Dual water" is algebraically the Juhász
shale-point form (**T3** `modules-quanti-saturation-dualw.gif`, dossier §2.7).

**As-built.** `ABSENT` — no alias table exists in the codebase.

**Verified by.** SB-SAT-T01, SB-SAT-T05

---

#### SB-SAT-004 — Simandoux: two variants, with `C` on the Schlumberger variant only [P1] [status: PRESENT-DIVERGENT]

**Requirement.** `simandoux_bardon_pied` MUST solve `g1·Sw^n + g2b·Sw + g3 = 0` with
`g1 = φe^m/(a·Rw)`, `g2b = Vsh/Rsh`, `g3 = −1/Rt`. `simandoux_modified_slb` MUST use
`g1 = φe^m/(a·Rw·(1−Vsh))`, `g2b = Vsh^C/Rsh`. The exponent `C` MUST be accepted **only** by
`simandoux_modified_slb`; requesting `C` on `simandoux_bardon_pied` MUST be an error, not a
silently ignored argument. `C` MUST be validated to the range **1:2**.

**Rationale.** `sw_sim.lls:212, 230` (Bardon-Pied branch) use plain `VSH/RT_SH`; only `:216, 234`
apply `VSH**C`, and `sw_sim.info:69` gates `C`'s visibility on `OPT_SIM:SCHLUM` (**T1**). A
`C ≠ 1` applied to Bardon-Pied is a divergence from every source held (dossier "findings this
revision added" item 1). `C = 1` reproduces IP E64 and Techlog's raster exactly.

**As-built.** `PRESENT-DIVERGENT` — the branch scoping is correct (`modules.rs:2280` vs `:2282`)
but the validation range is `0.5:2.0` (`modules.rs:2192`) against Geolog's `1:2`; `C < 1` is
supported by no source. `C` is a module-wide argument, so it is silently ignored rather than
rejected on the Bardon-Pied branch.

**Verified by.** SB-SAT-T06, SB-SAT-T07

---

#### SB-SAT-005 — Simandoux `a` ships with no default [P1] [status: PRESENT-DIVERGENT]

**Requirement.** `a` for both Simandoux variants MUST ship as `NoDefault`. SandiBumi MUST NOT
inherit Geolog's 0.8 or IP/Techlog's 1.0.

**Rationale.** Geolog's Simandoux is the only module in its own family defaulting `A = 0.8`
(**T1** `sw_sim.info`), worth **4.6 saturation units** against `a = 1` on the reference case
(dossier §3.2). `sw_sim.info` carries a References block but attributes **the 0.8 itself** to
neither cited paper — the file has references, the number does not. Choosing between 0.8 and 1.0
is adjudication disguised as a default (CONTRACT §2).

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2189` ships `A = 1.0`.

**Verified by.** SB-SAT-T08, SB-SAT-T31

---

#### SB-SAT-006 — Indonesia with a parameterised shale exponent [P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST implement Indonesia as
`v = Vsh^(2 − k·Vsh)`, `SWE = (1/(Rt·(1/(ff·Rw) + 2√(v/(Rw·ff·Rsh)) + v/Rsh)))^(1/n)` with
`ff = a/φe^m`, exposing `k` with presets `FULL (k=1)`, `SIMPLE (k=0)` and
`TAR_SAND/Woodhouse (k=2)`. Both the deterministic module and the solver MUST use the same
parameterised form.

**Rationale.** All three tools implement algebraically the same equation (dossier §2.4); Geolog's
three-option menu is a clean superset that also absorbs IP's separately-named "Woodhouse Tar"
(**T1** `sw_indo.lls`, `sw_indo.info`; **T2** C E65/E66; **T3** `modules-quanti-saturation-indo*.gif`).

**As-built.** `PARTIAL` — `modules.rs:2159-2163` implements all three variants correctly;
`multimin2.rs:154` hard-codes `Vsh^(1 − Vsh/2)`, i.e. `k = 1` only, so the solver cannot run
SIMPLE or TAR_SAND.

**Verified by.** SB-SAT-T09, SB-SAT-T10, SB-SAT-T30

---

#### SB-SAT-007 — Woodhouse Tar as a cited alias of Indonesia `k = 2` [P2] [status: PARTIAL]

**Requirement.** `woodhouse_tar` MUST be an alias of `indonesia` at `k = 2` and MUST carry the
citation **Woodhouse, R., "Athabasca Tar Sand Reservoir Properties Derived from Cores and Logs",
Transactions SPWLA 17th Annual Logging Symposium, Paper T, 1976**.

**Rationale.** Geolog's `TAR_SAND` (`v = Vsh^(2−2Vsh)`) is exactly IP's Woodhouse Tar
(`Vcl^(1−Vcl)` half-exponent), and Geolog supplies the paper letter IP omits (**T1**
`sw_indo.info:50-54`; **T2** C E66; dossier §2.4).

**As-built.** `PARTIAL` — the equation exists as `OPT_INDO=TAR_SAND` (`modules.rs:2161`); no
alias and no citation.

**Verified by.** SB-SAT-T10, SB-SAT-T35

---

#### SB-SAT-008 — Total Shale as a preset of `simandoux_modified_slb` with `n` fixed at 2 [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST ship Schlumberger Total Shale as a **preset and alias set** of
`simandoux_modified_slb` with `C = 1` and `n` pinned to 2, not as a separate model. `n` MUST NOT
be settable for `total_shale`: a caller supplying `n ≠ 2` MUST be a compile-time error. When
`n = 2` exactly, SandiBumi MAY use the closed-form quadratic root
`Sw = (−g2b + √(g2b² − 4·g1·g3))/(2·g1)` as a fast path, and if it does, that path MUST be
asserted equal to the general solver.

**Rationale.** `sw_tot.lls:150-156` is algebraically `sw_sim`'s `SCHLUM` branch at `C = 1, N = 2`
solved in closed form (**T1**); `sw_tot.info:12` states *"the saturation exponent N is set equal
to 2"* and the argument table has **no `N` row at all**. Shipping it as a distinct model would
duplicate an equation SandiBumi already has and re-open the §2.2 naming trap (dossier BL-2,
§4.2).

**As-built.** `ABSENT`.

**Verified by.** SB-SAT-T11, SB-SAT-T12

---

#### SB-SAT-009 — Juhász: shale-derived coefficient, shale-based normalization, model's own `m*` [P1] [status: PARTIAL]

**Requirement.** `juhasz` MUST compute the excess-conductivity coefficient as
`Qvn·(Cwsh − Cw)` with `Cwsh = 1/(Rsh·φt_sh^(m*))` and `Qvn = clamp(Vsh·φt_sh/φt, 0, 1)`, with
no `a` factor. The shale-point porosity exponent MUST be the model's own `m*`; SandiBumi MUST
NOT hard-code it to 2. IP's free `Bn` MAY be offered as an explicit override and MUST NOT be the
default. Normalization MUST be on **shale** (`Vsh`, `φt_sh`); where a user supplies clay
quantities the model MUST record which convention was used.

**Rationale.** Geolog and Techlog are algebraically identical and agree against IP (**T1**
`sw_juha.lls:34, 42, 54, 251-253`; **T3** `image1252.gif`). IP's `Bn = 1.0` is an admitted
crossplot placeholder and costs **14.0 saturation units** (dossier §3.5). Techlog's own two
modules disagree on the exponent — `φtsh²` in Dual water against `φtsh^m*` in Juhasz, **44 %
apart on `Cwsh` and opposite signs on the coefficient** at `m* = 1.6` (dossier §3.6); the `m*`
reading is the one two of three sources support.

**As-built.** `PARTIAL` — `multimin2.rs:282-298` implements exactly this, using `m` not 2
(`multimin2.rs:296`). No standalone `sw_juha` module exists, and no `Bn` override is offered.

**Verified by.** SB-SAT-T13, SB-SAT-T14, SB-SAT-T30

---

#### SB-SAT-010 — Juhász MUST flag a negative excess-conductivity coefficient [P0] [status: ABSENT]

**Requirement.** When `Cwsh − Cw < 0`, `juhasz` MUST raise a flagged condition on that sample.
SandiBumi MUST NOT return an unflagged saturation from a negative excess-conductivity
coefficient.

**Rationale.** The shale-referenced normalization is physical only while the shale water is
fresher per unit porosity than the formation water. At the dossier's base fresh-water case
(Rw 0.25 ⇒ Cw 4.0; Rsh 3, φtsh 0.35 ⇒ Cwsh 2.721) the coefficient is **−1.279** — the shale term
*reduces* conductivity. **Neither Geolog nor Techlog warns** (**T1**/**T3**, dossier §3.5), and
IP's positive-by-construction `Bn` hides the condition entirely. This is a validity limit that
is cheap to enforce and unmatched by any incumbent (CONTRACT §5.3).

**As-built.** `ABSENT` — `multimin2.rs:297` passes the coefficient straight to `sw_cond_root`,
which handles a negative `lin` numerically (`multimin2.rs:244-250`) and returns a lower Sw with
no flag.

**Verified by.** SB-SAT-T15

---

#### SB-SAT-011 — Waxman-Smits with `a` exposed [P1] [status: PARTIAL]

**Requirement.** `waxman_smits` MUST solve
`Ct = φt^(m*)·(Cw·SwT^(n*) + B·Qv·SwT^(n*−1))/a` with `a` exposed as a parameter. SandiBumi MUST
NOT fix `a = 1`.

**Rationale.** IP and Geolog both expose `a`; Techlog's shipped equation image omits it
(**T2** C E70; **T1** `sw_ws.lls:35, 43`; **T3** `image1879.gif`). Expanding all three gives the
same conductivity sum modulo `a`, so exposing it is a strict superset (dossier §2.5, §4.2).

**As-built.** `PARTIAL` — `multimin2.rs:307-313` implements the equation correctly but passes
`a = 1.0` unconditionally (`multimin2.rs:312`). No standalone module exists.

**Verified by.** SB-SAT-T16

---

### 4.2 Unit safety — the three failures that cost the most

#### SB-SAT-012 — `B` MUST be a unit-typed quantity, canonically `L·S/(eq·m)` [P0] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST represent the Waxman-Smits counterion conductance `B` as a
**unit-typed** quantity whose canonical internal unit is `L·S/(eq·m)` ≡ `mho·mL/(m·meq)`
(the ≈ 4-at-25 °C scale). Conversion to or from `mho·cm²/meq` MUST go through a single named,
tested converter (`×100`). Passing a raw number in the other scale into a saturation equation
MUST be **impossible to express** — a compile-time type error, not a runtime check.

**Rationale.** Three vendors ship three unit systems for one quantity, and one vendor
(Geolog) divides by 100 and multiplies by 100 thirty lines apart in the same file
(**T1** `sw_ws.lls:259-260`, `:289`). Getting it wrong costs **27.2 saturation units — a 63 %
relative error on Sw** — in the conservative direction, so it destroys pay rather than creating
it, and nothing errors (dossier §3.3). IP's own manual mislabels `B` with `Qv`'s unit (**T2**).
This is the single most expensive number in the domain.

**As-built.** `PRESENT-DIVERGENT` — the value and its documented unit are correct
(`multimin2.rs:320-321`) but `B` is a bare `f64` at every call site (`multimin2.rs:307`, `:1509`;
`lrlc.rs:64`, `:267`), so the wrong scale is expressible.

**Verified by.** SB-SAT-T17, SB-SAT-T18

---

#### SB-SAT-013 — `Qv` MUST be unit-typed, canonically meq/mL [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `Qv` MUST be unit-typed with canonical unit **meq/mL ≡ eq/L ≡ meq/cm³**.
Round-tripping between those three MUST be the identity. A `Qv` supplied in **meq/L** MUST be
rejected unless explicitly converted. `CEC` MUST likewise be unit-typed; the two `CEC`
conventions currently in the product (meq/100 g and meq/g) MUST be distinguishable by type.

**Rationale.** `1 eq/L ≡ 1 meq/mL ≡ 1 meq/cm³` is settled by positive identification, not
sanity check: Techlog's own `QV = CEC·ρg·(1−Φ)/Φ` with `CEC` in meq/g and `ρg` in g/cm³
arithmetically yields meq/cm³ while the page labels it `1/L` (**T3**
`petrophysics-qv-function-cec.html`); Geolog labels the same quantity `m/c3` (**T1**
`sw_ws.info`). The meq/mL-not-meq/L confusion is a documented failure mode on this machine
(**T4** `reference_waxman_smits_b`) and is a factor of 1000.

**As-built.** `PRESENT-DIVERGENT` — `Qv` is a bare `f64` in both constructions
(`lrlc.rs:40`, `multimin2.rs:1508`), and the two use different `CEC` units
(`lrlc.rs:100` meq/100 g vs `multimin2.rs:65` meq/g) with nothing enforcing consistency.

**Verified by.** SB-SAT-T19, SB-SAT-T20

---

#### SB-SAT-014 — `B(T,Rw)` MUST consume typed °C and clamp `B ≥ 0` [P0] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST compute
`B = (−1.28 + 0.225·T − 0.0004059·T²)/(1 + (0.045·T − 0.27)·Rw^1.23)` with **`T` a unit-typed
temperature in °C** and `Rw` in ohm·m, clamped `B ≥ 0`. A temperature in °F MUST be rejected or
converted at the boundary, never evaluated. SandiBumi MUST expose a measured-`B` override, and
MUST record in the output which of the two produced the value.

**Rationale.** Four independent sources give °C — IP 2025's own symbol dictionary, IP 2018,
Geolog's `FTEMP` unit and Arps constant, and Techlog's parameter table (**T2**/**T2**/**T1**/**T3**)
— against one ingest note that recorded "implied degF". Feeding °F gives **+47 % on `B`** and
**~20 % relative on Sw** (dossier §3.4). The numerator goes negative below ~6 °C, so the clamp is
required for physicality. The fit is known to overshoot above ~120 °C (**T4**
`reference_waxman_smits_b`), which is why the override exists.

**As-built.** `PRESENT-DIVERGENT` — formula, °C, bracket grouping and clamp are all correct
(`multimin2.rs:326-336`, tested at `multimin2.rs:3833-3856`), and an override exists
(`multimin2.rs:367-371`, applied at `:1509`). The temperature is an untyped `f64`
(`multimin2.rs:326`), and a second implementation at `lrlc.rs:64-67` carries no clamp of its own.

**Verified by.** SB-SAT-T21, SB-SAT-T22

---

#### SB-SAT-015 — `B` method ships with no default and four named options [P1] [status: PARTIAL]

**Requirement.** The `B` method selector MUST ship as `NoDefault` — the user chooses. SandiBumi
MUST offer, as separately named and cited options: the Juhász closed form; Geolog's `WAX_SMIT`
fit; Geolog's `WAX_THOM` quartic fit; Geolog's `GRAVEST` fit; and user-defined. Where a
chart-derived `B` is offered it MUST be implemented from published executable source or a
published closed form, never from transcribed chart node values.

**Rationale.** The three tools ship three different factory choices — IP the closed form, Geolog
`WAX_THOM`, Techlog the "1978 Waxman B chart" (**T2**/**T1**/**T3**, dossier §3.9) — so
same-named Waxman-Smits runs will not agree across tools even with identical `a/m*/n*/Rw/Qv`.
Picking one silently reproduces the disagreement without surfacing it. Geolog's `WAX_SMIT` and
`WAX_THOM` differ by **~20 % at 25 °C before the salinity factor** (`0.046/0.038320 = 1.200`),
and they carry **different salinity factors**, so 20 % is a ceiling ratio, not the ratio at any
given `Rw` (dossier §3.9).

**As-built.** `PARTIAL` — only the Juhász closed form is implemented (`multimin2.rs:326`,
`lrlc.rs:64`), with an override but no method selector.

**Verified by.** SB-SAT-T23, SB-SAT-T31

---

### 4.3 Dual water — the physics chain

#### SB-SAT-016 — Dual water ships in two named forms [P2] [status: PARTIAL]

**Requirement.** SandiBumi MUST ship `dual_water_cec` (the full Clavier-Coates-Dumanoir chain:
molarity → Debye-Hückel activity → expansion factor α → `vQ` → `Swb` → β → `Cwb`) and
`dual_water_simple` (IP's parameter form, `Swb` and `Rwb` supplied directly) as two separately
named methods. Techlog's Quanti "Dual water" MUST be exposed under `juhasz`, aliased, and MUST
NOT appear under either dual-water name.

**Rationale.** Three tools ship three different models under one name (dossier §2.7): only
Geolog has the diffuse-layer physics (**T1** `sw_dual.lls`), IP's is the parameter form (**T2**
C E67), and Techlog Quanti's is the Juhász shale-point form with `φtsh` hard-coded to 2 (**T3**
`modules-quanti-saturation-dualw.gif`). Geolog's own CEC-absent fallback
(`SWB_U = 1 − PHIE/PHIT`, **T1** `sw_dual.lls:471-484`) **is exactly IP's only mode**, so the
two forms are one implementation with two entry points.

**As-built.** `PARTIAL` — `SwModel::DualWaterNonlinear` implements the CEC form
(`multimin2.rs:213-221`); `dual_water_simple` is `ABSENT`, as is the CEC-absent fallback path.

**Verified by.** SB-SAT-T24, SB-SAT-T25

---

#### SB-SAT-017 — The excess-conductivity coefficient MUST be `Swb·(Cwb − Cw)` [P1] [status: PRESENT-OK]

**Requirement.** `dual_water_cec` MUST compute its excess-conductivity coefficient as
`g2 = Swb·(Cwb − Cw)`. SandiBumi MUST NOT implement it as `β·Qv`, and MUST NOT assume α cancels
from it.

**Rationale.** Geolog's coefficient is `SWB_U·(cwbu − 1/rwtemp)` (**T1** `sw_dual.lls:531-533`).
Substituting `Swb = α·vQh·Qv` and `Cwb = β/(α·vQh)` gives `g2 = β·Qv − α·vQh·Qv·Cw` below the
cap and `(1 − φe/φt)·(β/(α·vQh) − Cw)` above it — **in both regimes α reduces `g2`**. The
α-dependent share is **36.5 % / 21.8 % / 17.7 % at 25 000 / 5 000 / 3 000 ppm** at T = 100 °C
(dossier §2.7, BL-3). A `β·Qv` implementation would be α-independent and wrong by that amount,
largest in the fresh-water regime.

**As-built.** `PRESENT-OK` — `multimin2.rs:220` computes `swb * (cwb - cw)`.

**Verified by.** SB-SAT-T26

---

#### SB-SAT-018 — `vQ` MUST switch temperature form on the expansion branch [P1] [status: PRESENT-DIVERGENT]

**Requirement.** `dual_water_cec` MUST compute
`vQh = vQ0·√((273 + T°C)/295)` when `α > 1` and `vQh = vQ0·320/(T°C + 298)` when `α = 1`, then
`vQ = α·vQh`. SandiBumi MUST NOT apply one branch's temperature form across both.

**Rationale.** Geolog branches explicitly (**T1** `sw_dual.lls:630-631`). At T = 100 °C the two
forms give `0.33732` and `0.24121` mL/meq — **28.5 % apart**. `Swb = vQ·Qv`, and because
`Swb·Cwb = β·Qv` is invariant while the subtracted `Swb·Cw` term is not, the error propagates
into `g2` through exactly the term SB-SAT-017 establishes does not cancel.

**As-built.** `PRESENT-DIVERGENT` — `multimin2.rs:604-605` applies `α·96/(T+298)`
unconditionally, i.e. the `α = 1` form scaled by α. (The coefficient `96.0` is correct as
written for `CEC` in meq/g and ρ in g/cc; the `0.096` held in local memory would be the kg/m³
form and does not match this code.)

**Verified by.** SB-SAT-T27

---

#### SB-SAT-019 — α MUST include the Debye-Hückel activity ratio [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The expansion factor MUST be
`α = MAX(1, √((γ(0.35)·0.35)/(γ(n)·n)))` with `γ(x) = 10^(−0.5085·√x/(1 + 0.3281·4.5·√x))` and
`n = ρw·Salinity[ppm]/(1000·58450)` mol/L. SandiBumi MUST NOT approximate it as `√(0.35/n)`.

**Rationale.** Geolog computes both activity coefficients (**T1** `sw_dual.lls:365-372`, citing
Skoog & West 4th ed. for the activity coefficient and Clavier, Coates & Dumanoir 1984 for the
model). Dropping the γ ratio gives α **+6.7 % at 5 000 ppm and +8.9 % at 3 000 ppm**, and α
scales `Swb` directly.

**As-built.** `PRESENT-DIVERGENT` — `multimin2.rs:557-563` computes `√(20455/S)`, which is the
γ-free form with the ppm↔molarity conversion folded into the threshold
(20 455/58 450 = 0.34995). The `.min(5.0)` ceiling is uncited.

**Verified by.** SB-SAT-T28

---

#### SB-SAT-020 — β MUST carry the salinity dilution factor [P2] [status: ABSENT]

**Requirement.** `dual_water_cec` MUST compute
`β = 2.05·(T°C + 8.5)/(22 + 8.5) · (1 − β_const·e^(−2·Cw))` with `β_const` exposed, default
**1**, range 0:1.

**Rationale.** **T1** `sw_dual.lls:637-638`; `sw_dual.info` `BETA_CONST DEFAULT=1
VALIDATION=0:1`. At Cw = 4 mho/m the factor is 0.99966 and the omission is immaterial; at
Cw = 1 mho/m (Rw 1.0) it is `1 − e^(−2) = 0.8647`, so omitting it makes `Cwb` **15.7 % high** —
again worst in fresh water.

**As-built.** `ABSENT` — `multimin2.rs:580` computes the collapsed `0.0007·(T+8.5)·(T+298)`
form, which correctly reproduces `β/vQ` at `β_const = 0` but has no dilution term and no
`β_const` parameter.

**Verified by.** SB-SAT-T29

---

#### SB-SAT-021 — `Qv > 1/vQ` MUST flag; `Swb ≤ 1 − φe/φt` MUST clamp [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST raise a flagged condition when `Qv > 1/vQ`, leaving `Qv`
unmodified, **and** MUST clamp `Swb ≤ 1 − φe/φt`. The two MUST be separate mechanisms: the first
is a physical validity bound, the second a porosity-model bound.

**Rationale.** Geolog documents `Qv ≤ 1/(α·vQh)` (**T1** `sw_dual.lls:129`) — the `Swb ≤ 1`
bound written in `Qv` terms — and **ships both enforcing sites commented out** (`:449-450`,
`:460-461`). Only the weaker porosity-model cap survives, which is *usually* stricter but
silently absorbs an inadmissible `Qv` instead of surfacing it. Flagging rather than clamping
matches Geolog's shipped behaviour while surfacing the state its own doc warns about (dossier
§2.7, MJ-14).

**As-built.** `ABSENT` — `multimin2.rs:1465` clamps `swb` to `[0,1]` only. The porosity cap
holds structurally in the solver because φt ≡ φe + v_bw (`multimin2.rs:1461`), but no diagnostic
exists and a standalone module would have neither.

**Verified by.** SB-SAT-T32

---

#### SB-SAT-022 — `vQ0` ships absent [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The room-temperature bound-water volume per counterion charge `vQ0` MUST ship
as `NoDefault`, with both cited candidate values presented to the user and the choice recorded
in the output.

**Rationale.** Geolog ships **0.3 mL/meq at 22 °C** (**T1** `sw_dual.lls:427`) and Techlog Elan
**0.28 cm³/meq at room temperature** (**T3**), which agrees with the Clavier value held locally
(**T4**). ~7 % apart, and **both are cited to the same paper** — Geolog's `sw_dual.info:142-144`
names Clavier, Coates & Dumanoir 1984, which is where Elan's 0.28 comes from. That makes it two
*readings* of one paper, not a vendor invention, and adjudicating it requires the paper
(escalation ESC-4). Picking one silently is exactly the adjudication CONTRACT §2 forbids.

**As-built.** `PRESENT-DIVERGENT` — `multimin2.rs:605` hard-codes `96.0`, i.e. `vQ0 = 0.30`
folded into a magic constant with no parameter and no citation.

**Verified by.** SB-SAT-T31, SB-SAT-T33

---

### 4.4 Conversions, clamps and solver behaviour

#### SB-SAT-023 — The effective back-out is per model, never blanket [P1] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST apply the effective back-out `SWE = MAX((SWT − Swb)/(1 − Swb), 0)`
with a **per-model** `Swb`: `1 − φe/φt` for `archie_total`, `waxman_smits` and both dual-water
forms; **`Qvn = clamp(Vsh·φt_sh/φt, 0, 1)`** for `juhasz`. `Swb = 1` MUST yield `SWE = 1`, not a
divide-by-zero. Where the solver's construction makes φt ≡ φe + v_bw and therefore collapses the
two rules, SandiBumi MUST record which rule was applied rather than leave it implicit. SandiBumi
MUST also ship the inverse pair `SwT = Sw(1 − Swb) + Swb` and `SxoT = Sxo(1 − Swb) + Swb`, and
a round-trip through the pair MUST be the identity.

**Rationale.** For the first group IP's E78 and Geolog's `SWE = 1 − (φt/φe)(1 − SWT)` are
algebraically identical and all three tools agree (**T2** C E78; **T1** `sw_ws.lls:296`). For
Juhász, Geolog (**T1** `sw_juha.lls:262`) and Techlog use `Qvn`, and the two forms are **not**
equal — on the dossier's fixture (Qvn 0.42 vs `1 − φe/φt` 0.20) `SWE` differs by **tens of
saturation units while `SWT` matches exactly** (dossier §2.6, BL-6). The inverses are shipped by
IP too and must travel alongside E78 or a round-trip is not the identity (**T2**
`B_core_petro.md:276-280`, `:446-454`; dossier MN-11).

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2065-2069` is correct for `archie_total`;
`multimin2.rs:1497` applies the porosity split to Juhász (defensible in the solver, undocumented
outside a source comment, and divergent from both vendors). `lrlc.rs:160-161` and `:295-296`
introduce a **third** rule — `Swb = CBW/φt` from a clay-bound-water curve — which is correct for
the LRLC methods but is nowhere specified. No inverse conversion exists anywhere.

**Verified by.** SB-SAT-T34, SB-SAT-T35, SB-SAT-T36

---

#### SB-SAT-024 — `SWE_IRR` is an effective quantity, transformed per model [P2] [status: PRESENT-OK]

**Requirement.** The effective irreducible floor MUST be
`SWE_IRR = (SWT_IRR − Swb)/(1 − Swb)` using **the same per-model `Swb` as that model's back-out**,
with `SWE_IRR = 0` when `Swb = 1`. SandiBumi MUST NOT implement `SWE_IRR = SWT_IRR·φt/φe`. The
floor MUST be monotone in `SWT_IRR` and MUST never exceed 1.

**Rationale.** Geolog's 2008 fix landed in `sw_arch` (`:234`), `sw_juha` (`:271`) and `sw_dual`
(`:552`) but **never reached `sw_ws`**, which still computes `swe_irr = PHIT·SWT_IRR/PHIE`
(`:302`) — inconsistent with `sw_ws`'s own `SWE` map at `:296`, so the floor is applied in the
wrong space (**T1**, dossier §2.2, MJ-1). Fixture: φt 0.30, φe 0.20, `SWT_IRR` 0.20 ⇒ Geolog's
form gives **0.30**, the consistent form gives **0**.

**As-built.** `PRESENT-OK` — `modules.rs:2071` implements the transform with the degeneracy
handled. It is untested, and nothing records that the Geolog `sw_ws` form is forbidden.

**Verified by.** SB-SAT-T37

---

#### SB-SAT-025 — Every method emits a clipped and an unclipped curve [P1] [status: PARTIAL]

**Requirement.** Every saturation method MUST emit both a clipped curve (`SWE`/`SWT`, bounded to
`[SWE_IRR, 1]` / `[SWT_IRR, 1]`) and an unclipped diagnostic (`SWE_<METHOD>` / `SWT_<METHOD>`).
Where a method produces both a total and an effective result, **both** MUST have an unclipped
counterpart.

**Rationale.** Geolog and Techlog both ship unclipped diagnostics; IP does not, and IP's
comparison-curve caveat exists precisely because it lacks them (**T1**/**T3**/**T2**, dossier
§2.9). A clipped-only curve cannot distinguish "the rock is wet" from "the model went out of
range".

**As-built.** `PARTIAL` — `SWT_ARCH` (`modules.rs:2001`), `SWE_INDO` (`:2109`), `SWE_SIM`
(`:2201`) are emitted unclipped, but `sw_arch`'s **effective** result has no unclipped
counterpart, and the LRLC modules emit clamped values only (`lrlc.rs:153`, `:161`, `:281`).

> **Correction (2026-08-20, AUDIT-2026-08-20 finding 4; DEC-085 "diagnostics stay raw").** The
> `SWE_SIM` half of that as-built statement was **wrong when written**: the curve was declared
> unclipped and named into this family, but every exit of the shared Simandoux root solver
> clamped to [0, 1] (`sandimin::solve_simandoux_root`), so wherever the true root exceeded 1 the
> "unlimited" diagnostic read exactly 1.000 — bit-identical to the `SWE` beside it, and unflagged,
> because `limit` records a clamp only when it changes a value and this one arrived pre-flattened.
> `sw_sim` now takes `sandimin::sw_simandoux_*_unlimited`, which is the same equation without the
> clamp (the clamped entry point is literally it, clamped), so the pair cannot come from two
> implementations. The working `SWE` is bit-identical for every reading. The `VSH >= 1`
> singularity still answers 1.0 in both renderings and stays reported by name under SB-SAT-030 —
> a raw reading requires an equation to have been evaluated. Pinned by
> `the_unlimited_simandoux_diagnostic_reports_the_root_above_one_instead_of_a_second_copy_of_swe`.
> The other two gaps in this note (`sw_arch` effective, the LRLC modules) are unchanged and stand.

**Verified by.** SB-SAT-T38

---

#### SB-SAT-026 — Never emit a bare `SW`; always emit a method-flag curve [P1] [status: PARTIAL]

**Requirement.** No emitted mnemonic MAY equal bare `SW` or `SXO`; every saturation curve MUST
carry an `E` or `T` designator. Every saturation run MUST additionally emit a **method-flag
curve** recording which model produced each sample, and MUST emit `VOL_UWAT`/`VOL_XWAT` (or
`BVWE`/`BVWT`) alongside.

**Rationale.** Ledger **D-15** is an open design mandate (**T2**). Geolog is the only tool of
the three with no ambiguity anywhere in its family, and it emits `OPT_SW` as a first-class
output (**T1**); Techlog violates the rule with `SW_AR` (**T3**). Adopting Geolog's scheme closes
D-15 for this domain (dossier §4.3).

**As-built.** `PARTIAL` — mnemonics are correctly designated throughout and no bare `SW` is
emitted, but this is unenforced by any test; **no method-flag curve exists**; `VOL_UWAT` is
emitted (`modules.rs:2004`) and `VOL_XWAT` is not.

**Verified by.** SB-SAT-T39, SB-SAT-T40

---

#### SB-SAT-027 — One shared root-finder with Geolog's guards [P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST solve every polynomial-form saturation model with one shared
root-finder using seed 0.5, **maximum 20 iterations**, tolerance `|Δ| < 1e-5`, and
`sat = MAX(0, sat)` at each step. Where a closed form exists for a special case (`n = 2`), it
MAY be used as a fast path and MUST be asserted equal to the general solver.

**Rationale.** **T1** `sw_sim.lls:256-271`. Techlog's Levenberg-Marquardt is over-engineered for
a scalar monotone root and its behaviour is undocumented and uninspectable (**T3**, escalation
ESC-8); Geolog's guards are explicit and testable (dossier §4.2).

**As-built.** `PRESENT-OK` — `modules.rs:2218-2230` transcribes Geolog's `CALC_SW` exactly.
`multimin2.rs:230-261` is a second, different solver (closed quadratic at `n = 2`, else 60-step
bisection) — defensible for the solver's monotone forms, but it is a second implementation and
the two are not cross-asserted.

**Verified by.** SB-SAT-T12, SB-SAT-T41

---

#### SB-SAT-028 — Non-convergence MUST return null, never a partial iterate [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A saturation solver that fails to converge within its iteration budget MUST
return null for that sample. SandiBumi MUST NOT emit the last iterate of a non-converged solve.

**Rationale.** Geolog sets `sat = MISSING` on non-convergence (**T1** `sw_sim.lls:256-271`).
A partial iterate is indistinguishable from a converged answer on the log, which is the silent
-failure class CONTRACT §5.3 and IP FINDINGS rule 14 both target.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2229` correctly returns `MISSING`, but
`lrlc.rs:271-290` (`sw_imts`) iterates 100 times and **keeps the last iterate** on falling out of
the loop; only a NaN from a non-positive denominator is caught. SandiBumi's own method has the
defect its vendor-derived module avoids.

**Verified by.** SB-SAT-T41

---

#### SB-SAT-029 — Inherit the documented guard rails, including the volume detail [P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST implement: `φe < 0.005 ⇒ all saturations 1` **and
`VOL_UWAT = VOL_XWAT = φe`, not 0**; `φe = φt = 0 ⇒ all saturations 1, all volumes 0`;
`Rt` missing or ≤ 0 ⇒ every saturation output null; a missing variable-`m` input curve ⇒ every
saturation and volume output null with a message.

**Rationale.** All four are Geolog's documented behaviours, the low-porosity rule appearing in
**all nine** `sw_*` modules (**T1**, dossier §2.9). The volume detail is the one that bites:
setting volumes to 0 there would silently zero bulk-volume water over tight streaks that still
carry porosity — the interval is declared **wet**, not declared **empty** (dossier MN-4).

**As-built.** `PRESENT-OK` — `modules.rs:2074-2079`, `:2140-2144`, `:2247-2251` (low porosity,
with `VOL_UWAT = φe`); `:2036-2042` (coal, keyed on φt alone so it fires when `PHIE` is absent);
`:2050`, `:2150`, `:2258` (`Rt ≤ 0` and missing inputs). No variable-`m` route exists, so its
guard is `ABSENT` by construction.

**Verified by.** SB-SAT-T42, SB-SAT-T43

---

#### SB-SAT-030 — `Vsh → 1` MUST flag before the singularity, not silently return water [P1] [status: PRESENT-DIVERGENT]

**Requirement.** When `Vsh → 1` in `simandoux_modified_slb` (whose `1/(1−Vsh)` term is singular)
or in `indonesia` (where water and effective porosity both go to zero), SandiBumi MUST raise a
flagged condition. It MAY additionally return `Sw = 1`; it MUST NOT return `Sw = 1` unflagged.

**Rationale.** Techlog Elan is the only vendor documenting this failure mode: the equation blows
up at `Vcl = 100 %` (0/0) and *"it is a good idea to write a constraint to force the volume of
water to be greater than about 0.5 p.u."* (**T3**). Neither IP nor Geolog documents it (dossier
§2.9). Returning a plausible number from a singular equation is exactly the fail-silent pattern.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2265-2270` silently sets all-water for
`Vsh ≥ 1` on the Schlumberger branch, with the reason in a source comment and nothing in the
output.

**Verified by.** SB-SAT-T44

---

### 4.5 Parameter provenance — the requirement that outranks the others

#### SB-SAT-031 — `Rw` ships with no default [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `Rw` MUST ship as `NoDefault` in every saturation module and in the solver.
SandiBumi MUST NOT inherit IP's 0.1 ohmm or Techlog's 0.03 ohm.m, and MUST NOT substitute a
value derived from a formation-water environment band.

**Rationale.** Geolog takes `RW`/`RWS`/`SALW` as required inputs with no default (**T1**) and is
the best-behaved of the three. IP's 0.1 and Techlog's 0.03 differ by **1.83× on Sw** at
m = n = 2; IP at least warns *"must be adjusted to the correct value"*, Techlog does not
(**T2**/**T3**, dossier §3.8). The dossier explicitly **withdrew** a project-kb `Rw ≈ 0.21`
figure as unsound corroboration (cross-basin, ambiguous header, and three salinity methods
disagreeing at 24/12/25 kppm in the same record — MN-8), so no default rests on it.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1943` ships 0.1 (IP's rejected value);
`lrlc.rs:93` and `:200` ship 0.3 (uncited). SandiBumi's own two engines are `√3 = 1.73×` apart
on Sw before the user touches anything.

**Verified by.** SB-SAT-T31, SB-SAT-T45

---

#### SB-SAT-032 — `Rw` correlations with the temperature conversion bound to the branch [P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST ship the `Rw` resolution branches as one function with each
temperature conversion **bound to its own branch**: `MEASURED` → Arps °C from the user's `RWT`;
`SALINITY` above 39 161 ppm → Kennedy polynomial then Arps **°F from 75 °F**; `SALINITY` at or
below 39 161 ppm → Bateman-Konen then Arps **°C from 23.9 °C**. Cross-wiring MUST be prevented
by construction and asserted by test.

**Rationale.** **T1** `sw_arch.lls:174-195`, identical in seven sibling modules. Pairing the
wrong conversion with the wrong branch is a silent error at the switch salinity; the two
correlations agree to **0.07 %** at 39 161 ppm, which is the evidence the pair is transcribed
correctly and also the reason a swap will not be visible in the output (dossier §5.2, §2.10).

**As-built.** `PRESENT-OK` — `modules.rs:1951-1983` implements all four branches with the
conversions correctly bound. Untested (§3.3).

**Verified by.** SB-SAT-T45, SB-SAT-T46, SB-SAT-T48

---

#### SB-SAT-033 — The Kennedy floor is 0.0412 and the vendor doc is wrong [P2] [status: PRESENT-OK]

**Requirement.** The Kennedy salinity cap MUST be `SALW > 275 000 ppm ⇒ rw75 = 0.0412 ohm·m at
75 °F`. Both the implementation and its test MUST carry a comment recording that all eight
Geolog `sw_*` doc blocks state **0.412** and are wrong by a factor of ten.

**Rationale.** Every Geolog code path sets `0.0412`; every doc block says `0.412`; a third,
commented-out message restates the correct magnitude with the scale changed to **75C** (**T1**
`sw_arch.lls:188` vs `:49`; `sw_dual.lls:327, 357, 392`). 0.412 ohm·m at 275 000 ppm is
physically far too high. Without the comment, a future reader "fixing" the test to match the
vendor documentation would introduce a ×10 `Rw` error (dossier test 24).

**As-built.** `PRESENT-OK` on the value — `modules.rs:1972-1973`. The explanatory comment is
`ABSENT`, and so is the test.

**Verified by.** SB-SAT-T47

---

#### SB-SAT-034 — `a`, `m`, `n`, `m*`, `n*` ship with no default [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `a`, `m`, `n` and the Waxman-Smits/dual-water `m*`, `n*` MUST ship as
`NoDefault` — a first-class state distinct from any numeric value. A run requesting a saturation
model without them MUST fail with a message naming the missing parameter.

**Rationale.** IP's PhiSw and SSM manual pages state **no default for `a`/`m`/`n` at all**; the
1.0/2.0/2.0 commonly quoted are Basic Log Analysis values only (**T2** IP2018 §3.1, verbatim,
and the dossier records IP FINDINGS §6 rule 9 as directly applicable). Geolog defaults 2 with
range 1:10 and Techlog defaults 2 (**T1**/**T3**), but a cementation exponent is a rock property
measured on core — Jauhar's own delivered studies use SCAL-derived values per field
(**T4** `project-kb/records/limau-phr-zona4.md`, `anambas-kufpec.md`). A shipped exponent is the
highest-consequence silent default in petrophysics.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1991-1993` ships `A 1.0 / M 2.0 / N 2.0`;
`lrlc.rs:94-95` ships `M 2.0 / N 2.0`; `lrlc.rs:203-204` ships `MSTAR 1.9 / NSTAR 1.9`
(uncited); `multimin2.rs:380` ships `archie_a 1.0`.

**Verified by.** SB-SAT-T31, SB-SAT-T49

---

#### SB-SAT-035 — `Rsh` and `φt_sh` ship with no default, and the current values are withdrawn [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `Rsh`/`Rcl` and `φt_sh` MUST ship as `NoDefault`. The values `Rsh = 4.0` and
`φ_sh = 0.10` currently in the product MUST be removed, in the backend and in the UI. `φt_sh`
MUST be validated to **0:0.4** as the accept range with **0.4:1** as a warn band.

**Rationale.** Geolog takes both as input logs with no default (**T1**); IP has the interpreter
pick them (**T2**); Techlog defaults `Res_shale` 5 and `Porosity shale` 0.4 (**T3**).
**SandiBumi's 4.0 and 0.10 match no source held anywhere** — they are uncited numbers in shipped
code, which CONTRACT §2 forbids outright. `φ_sh` is the more expensive: Juhász consumes it
linearly in `Qvn` and as `φ_sh^(−m)` in `Cwsh`, so at m = 2 and Rsh 4, `Cwsh(0.10) = 25.0 mho/m`
against `Cwsh(0.40) = 1.5625 mho/m` — **16× apart**, with `Qvn` 4× apart on top. The validation
range is Geolog's own internal conflict resolved conservatively: `0:1` in `sw_juha.info` and
`sw_dual.info`, `0:0.4` in `qv.info`, where the tighter range is the one the shared support
module enforces because it feeds the `ρdsh` chain that blows up as `PHIT_SH → 1` (**T1**,
dossier MN-7).

**As-built.** `PRESENT-DIVERGENT` — `multimin2.rs:377-385`, mirrored at
`src/ui/multiminDialog.ts:430` and `:818-820`. `modules.rs:2101` and `:2193` ship `RT_SH = 5.0`,
which is at least Techlog's value but is still a default where none is defensible.

**Verified by.** SB-SAT-T31, SB-SAT-T50

---

#### SB-SAT-036 — Two named `m*`/`n*` routes, with core preferred [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST ship `mstar_from_core` (`F_STAR = F_APP(1 + B·Qv·Rw)`,
`M_STAR = −log₁₀F*/log₁₀φt`; `RI_STAR = RI_APP(1 + B·Qv·Rw)/(1 + B·Qv·Rw/Sw)`,
`N_STAR = −log₁₀RI*/log₁₀SwT`) and `mstar_from_qv` (`m* = m + Cm(1.128Y + 0.22(1−e^(−17.3Y)))`
for Waxman-Smits, `m* = m + Cm(0.258Y + 0.20(1−e^(−16.4Y)))` for dual water, `Y = Qv·φt/(1−φt)`)
as two separately named routes. It MUST prefer the core route where core exists, and MUST warn
that both consume `B`, so changing the `B` method also moves `m*` and `n*`.

**Rationale.** IP and Elan derive `m*` from `Qv` **inside** the solve; Geolog derives it from
core `F`/`RI`/CEC in a **separate pre-processing module** and then treats it as data (**T2**
IP2018 §3.4; **T1** `ffcec.info`, `ricec.info`; **T3**). Geolog's is the more defensible
provenance chain — an `m*` that traces to plugs rather than to a fitted relation — and it is the
route Jauhar's delivered studies actually use (**T4**, dossier §2.5, §4.1). `ffcec`/`ricec`
re-expose the same four-option `B` menu, so a `B` change silently moves `m*` too.

**As-built.** `ABSENT` — no `m*`/`n*` derivation route of either kind exists; `MSTAR`/`NSTAR`
are entered directly (`lrlc.rs:203-204`) or taken as `m`/`n` (`multimin2.rs:1426`).

**Verified by.** SB-SAT-T51

---

#### SB-SAT-037 — Shell / Elan variable `m` as one parameterised route with no default coefficient [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the variable-`m` porosity route as
`m = m₀ + c/φ` with `c` a **cited parameter shipping `NoDefault`**, offering the competing
readings as named, sourced presets rather than as a code path.

**Rationale.** Ledger **D-10** is OPEN and unadjudicable from the corpus: IP's rasters give
`0.018` (verified at 6× and 4×, both editions) and IP's ASCII prose gives `0.019` nine-plus times
in both editions (**T2**), with the published Shell formula agreeing with the ASCII (**T4**).
**Techlog Elan implements the same functional form** — `φe^(m + mc2/φe)` (**T3** `image2957.gif`
eq. 77) — but ships `MC2 = 0.0` (**T3** Table 27), i.e. the slot with no value, so **no
cross-tool arbitration of the coefficient exists** (dossier §3.10, BL-4). Stakes: at φe 0.02,
`m` = 2.77 vs 2.82 ⇒ Sw ratio `0.02^(−0.025) = 1.103`, **~10 % higher Sw on the 0.019 reading**,
negligible above φ ≈ 0.10. A parameterised route makes D-10 a configuration decision rather
than a code decision.

**As-built.** `ABSENT` — no variable-`m` route of any kind; `OPT_M` has no counterpart.

**Verified by.** SB-SAT-T31, SB-SAT-T52

---

#### SB-SAT-038 — Every parameter carries a source string, and the build fails without one [P0] [status: ABSENT]

**Requirement.** Every saturation parameter MUST resolve to either a value **with a non-empty
source string** or the explicit `NoDefault` state. A default with an empty source string MUST
fail the build. The source string MUST be a specific checkable reference — a file and section, a
module and parameter name, or a full literature citation.

**Rationale.** IP FINDINGS §6 rule 9, carried by the dossier as directly applicable, and
CONTRACT §2, which makes this the rule that outranks everything else. The domain's own evidence
is the argument: three vendors ship three `Rw` defaults, three `B` method defaults, two `vQ0`
values from the same paper, and a Simandoux `a` that no cited paper supports — and none of them
tells the user. A plausible-but-wrong endpoint computes, plots and ships into a reserves number
without failing.

**As-built.** `ABSENT` — no parameter in `modules.rs`, `lrlc.rs` or `multimin2.rs` carries a
source string; `ArgSpec` has no field for one (`modules.rs:1940-1948`).

**Verified by.** SB-SAT-T31

---

### 4.6 Coverage, provenance and the seams

#### SB-SAT-039 — `MUDBASE` is model-scoped [P3] [status: ABSENT]

**Requirement.** `MUDBASE = WATER | OIL` (default `WATER`) MUST exist on `simandoux_*` and
`dual_water_*` only; requesting it on any other model MUST be a compile-time error. On OBM,
`dual_water_cec` MUST set `α_x = α_u`, `β_x = β_u` and solve `SXOT` against `Rw` at formation
temperature, not against `Rmf`, and MUST gate `Rmf`/`ρ_mf` off.

**Rationale.** **T1** `sw_dual.info:164`, `sw_sim.info:74`; the flushed-zone effect at
`sw_dual.lls:444, 496, 567-572`. `sw_arch`, `sw_ws`, `sw_juha`, `sw_indo` and `sw_tot` have no
`MUDBASE` at all — verified by absence (dossier MN-6, which also rebuts a critique claim that
`sw_arch` carries it).

**As-built.** `ABSENT` — `MUDBASE` does not exist in the codebase. The solver carries a
`mud_type` field used only for the α branch (`multimin2.rs:585`), which is a different mechanism.

**Verified by.** SB-SAT-T53

---

#### SB-SAT-040 — Clay-bound-water `F`: both unit forms, `ρ_brine` open, `Swb = 1 − F` opt-in only [P3] [status: ABSENT]

**Requirement.** SandiBumi MUST store the Hill-Shirley-Klein / Juhász clay-bound-water fraction
in **both** native coefficient/unit pairs with native-unit metadata —
`F = 1 − (0.6425·Salinity[Kppm NaCl-equiv]^(−0.5) + 0.22)·Qv[meq/mL]` and
`F = 1 − (0.084·C₀[meq/cm³]^(−0.5) + 0.22)·Qv[meq/cm³]`, `C₀ = Salinity[g/L]/58.44` — and MUST
NOT normalise one into the other in source; conversion MUST happen at the boundary through a
named function. The `Kppm → g/L` step MUST name a brine-density factor
(`Salinity[g/L] = Salinity[Kppm] × ρ_brine[g/cm³]`); **`ρ_brine(salinity, T)` MUST ship as
`NoDefault`** and MUST NOT be filled from a textbook correlation. SandiBumi MUST flag above
~50 Kppm where the unit-bridge divergence reaches ≥ 3 %. `Swb = 1 − F` MUST be gated behind an
explicit opt-in, emitted as a diagnostic first, and MUST NOT be a default `Swb` source ahead of
`1 − φe/φt` or the CEC chain. Both vendors' actual uses of `F` — `PcCorr = Pc × F^(−0.5)` and
`SwPcCorr = 1 − (1 − SwPc) × F` — MUST be carried alongside so an implementer sees what `F` was
shipped for.

**Rationale.** Ledger **D-07** is resolved to reading (i) by Techlog's clean rendering plus the
`0.084·√58.44 = 0.6421469` unit bridge, which agrees with IP's `0.6425` to **three** significant
figures (0.055 %) — decisive against reading (ii), which would move the constant term 36 %
(**T3** `clay-bound-water-correction-004.png` verified at 6×; **T2** ledger D-07; dossier §2.8,
BL-5). But Techlog prints `0.084` to two significant figures, so the fourth digit is **not**
certified. Kppm is a mass *fraction* and g/L a mass *concentration*: the `÷58.44` bridge is exact
only at `ρ_brine = 1.0 g/cm³`, safe in the fresh-water band (3–13 kppm) and **not** safe at
250 Kppm where the brine is ≈1.19 g/cm³ — a ~19 % error in `C₀` (dossier MJ-6). And
`Swb = 1 − F` is **the dossier's own transplant**: both vendors apply `F` to capillary-pressure
data, neither to log evaluation (dossier MJ-7).

**As-built.** `ABSENT` — no `F` relation of any form exists.

**Verified by.** SB-SAT-T54, SB-SAT-T55

---

#### SB-SAT-041 — Poupon-Aguilera / Poupon-Tixier with the laminated interlock [P3] [status: ABSENT]

**Requirement.** SandiBumi MUST ship `poupon_aguilera` (`g1 = φe^m/(a·Rw·(1−Vcl))`,
`g3 = Vcl/Rcl − 1/Rt`) and `poupon_tixier` (`g1 = (1−Vcl)·φe^m/(a·Rw)`, `g3 = Vcl/Rcl − 1/Rt`)
with their citations attached, and MUST refuse — or hard-warn — when a laminated Sw model is
enabled at the same time.

**Rationale.** Citations, verbatim **T2** `A_porosity_sw.md:330-331`: Poupon-Aguilera →
*"'Extensions of Pickett Plots for the Analysis of Shaly Formations by Well Logs', Roberto
Aguilera (The Log Analyist, Sept-Oct 1990), where the exponents of 'n' and 'm' have been added"*;
Poupon-Tixier → *"'A contribution to electric log interpretation in shaly sands', Poupon A,
Loy ME, Tixier MP (1954) Trans AIME 6(06):138–145 with the addition of 'm' and 'n' exponents"*.
The interlock, verbatim `:697-699`: *"Note this equation assumes a formation of laminated sands
and shales with the sands being clean. **Do not use this equation if the Laminated Sw model
options are turned on since this would be double correcting for laminations.**"* IP also records
that Poupon-Aguilera *"was simply called 'Poupon"* in earlier versions (`:333-334`), which is an
alias-table row.

**As-built.** `ABSENT` — neither model exists; no interlock exists.

**Verified by.** SB-SAT-T56, SB-SAT-T57

---

#### SB-SAT-042 — The SSM bound-water cap fires and is flagged [P3] [status: ABSENT]

**Requirement.** If SandiBumi ships a sand-silt-clay-equivalent model, the bound-water cap
`Vbw ≤ 1.5 × Vcl × PhiTclay` MUST be applied, **followed by the `PhiT = Phie + Vbw` re-set**, and
MUST raise a flag when it fires. The `1.5` MUST be recorded as a hard-coded constant with its
source, not exposed as a parameter.

**Rationale.** **T2** `B_core_petro.md:244-253, 839, 1060` (`sand_silt_malay_model.htm`), where
the ingest flags it as *"easy to omit and changes `Swb`, hence Sw, in shaly intervals"*. It is a
saturation-affecting clamp whose home module is porosity, which is why it is specified here and
cross-referenced to `POR` (§1). A silent cap changes `Swb` and therefore Sw with nothing in the
output to show for it (dossier MJ-12).

**As-built.** `ABSENT` — `ssc.rs` implements the sand-silt-clay porosity model; no 1.5 cap and
no flag exist in the saturation path.

**Verified by.** SB-SAT-T58

---

#### SB-SAT-043 — A saturation result carries its parameters, their sources and their papers [P0] [status: ABSENT]

**Requirement.** Every saturation run MUST emit, alongside its curves, a machine-readable record
of: the model identifier, every parameter value used, each value's source string, the literature
citation the method traces to, and the Worthington 1985 type where one is stated by a source.
That record MUST survive export into the deliverable.

**Rationale.** Geolog ships published references inside every module manifest — Archie 1942,
Simandoux 1963 + Bardon & Pied 1969, Poupon & Leveaux 1971 Paper O, Woodhouse 1976 Paper T,
Waxman & Smits 1968, Waxman & Thomas 1974, Juhász 1979 Paper AA and 1981, Clavier/Coates/
Dumanoir 1984, Schlumberger 1989, Skoog & West 4th ed., Western Atlas Charts 1994 p. 27 (**T1**,
dossier §1.2) — **but no vendor carries the reference through to the answer**. A parameter that
carries the paper it came from, through the computation, into the deliverable is a claim no
incumbent can make (CONTRACT §5.4). It is also the only mechanism that makes SB-SAT-038
auditable downstream rather than only at build time.

**As-built.** `ABSENT` — no provenance record is emitted. Citations exist in some doc strings
(e.g. `modules.rs:2011`, `:2119`) but are not attached to results.

**Verified by.** SB-SAT-T59

---

#### SB-SAT-044 — Surface the cross-tool disagreement to the interpreter [P2] [status: ABSENT]

**Requirement.** Where the incumbents ship materially different values or behaviours for one
quantity, SandiBumi MUST make the disagreement visible at the point of choice: the competing
values, their sources, and the quantified consequence. This applies at minimum to the nine rows
of §2.12.

**Rationale.** CONTRACT §5.2 — three tools shipping three different values for one constant is a
fact the interpreter needs to defend a number, and none of the three surfaces it. The `vQ0` row
is the clearest case: **both values cite the same 1984 paper**, so what the interpreter needs is
not a default but the knowledge that a choice is being made (dossier §4.2, E-4).

**As-built.** `ABSENT`.

**Verified by.** SB-SAT-T60

---

#### SB-SAT-045 — Model-selection guidance, exposed as guidance and never as an automatic switch [P2] [status: ABSENT]

**Requirement.** SandiBumi MAY present the selection rule `Rw ≤ 0.20 ohm·m → (modified)
Simandoux; Rw > 0.20 ohm·m → Indonesia (Poupon-Leveaux)` as **guidance with its source
attached**. SandiBumi MUST NOT switch models automatically on it.

**Rationale.** The rule is the Halliburton Ch27 flowchart recorded in memory (**T4**
`reference_shaly_sand_sw_selection`), independently corroborated in salinity terms by a delivered
study — *"Simandoux used instead of Indonesia specifically where formation-water salinity exceeds
20,000 ppm"* (**T4** `project-kb/records/bunga-block-phe-posco.md`) — and high salinity ↔ low Rw,
so the two agree. Modified Simandoux and Indonesia are the two workhorses across Jauhar's
delivered studies; Waxman-Smits and dual water appear repeatedly as *considered and not adopted*
because they need CEC core data (dossier §4.1). An automatic switch would make a method choice
on the user's behalf, which is the same failure class as an uncited default.

**As-built.** `ABSENT`.

**Verified by.** SB-SAT-T60

---

#### SB-SAT-046 — Sxo and the flushed zone [P3] [status: PARTIAL]

**Requirement.** Every saturation model MUST offer a flushed-zone leg computing `SXOE`/`SXOT`
against `Rxo` and `Rmf`, emitting `VOL_XWAT`. A missing `Rxo` MUST yield `SXOT = SXOE = null`
with `VOL_XWAT = VOL_UWAT`, not zero. Where a model's `Sxo` and `Sw` legs could use different
sources for a shared quantity, they MUST use the same source.

**Rationale.** Geolog ships an `Sxo` leg per module with exactly that missing-`Rxo` behaviour
(**T1** `sw_arch.lls:245-261`); Techlog ships separate flushed-zone modules per method (**T3**);
IP offers Rxo/TPL/InvasionFactor routes with limits `Sw^SxoLimit ≥ Sxo ≥ Sw` (WBM) and
`Sw ≥ Sxo` (OBM) (**T2** E79-E81). The same-source clause exists because `sw_ws` violates it:
it computes `Bmax` from the published quartic for the unflushed zone and from the compiled
`mm_wt74` for the flushed zone **in the same run** (**T1** `sw_ws.lls:252-255` vs `:310`) — one
module, one interval, one temperature, two `Bmax` sources (dossier MJ-2).

**As-built.** `PARTIAL` — `SXOT` is emitted by the solver only (`multimin2.rs:1670-1674`); no
standalone module computes `Sxo`, `VOL_XWAT` is emitted nowhere, and no `Sxo` limits exist.

**Verified by.** SB-SAT-T61

---

#### SB-SAT-047 — One model, one number, whichever engine computes it [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A named saturation model MUST return the same value from the deterministic
module and from the mineral solver given the same inputs and parameters, to a stated tolerance.
Where the two contexts genuinely differ — the solver's φt ≡ φe + v_bw construction being the
known case — the difference MUST be documented at the model level and asserted by a test that
names it, not left to a source comment.

**Rationale.** This is a SandiBumi-internal requirement with no vendor counterpart, and it is
`P0` because the product currently fails it in the most expensive possible way: the two engines
compute **different Simandoux equations under the same name**, 7.3 su apart (§3.1). The same
class of drift would arise between `sw_tot`'s closed form and the general solver, which is why
SB-SAT-008 and SB-SAT-027 both require cross-assertion.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2279-2283` vs `multimin2.rs:167-175` (Simandoux);
`modules.rs:2159-2163` vs `multimin2.rs:154` (Indonesia variants); two `B` implementations
(`multimin2.rs:326`, `lrlc.rs:64`); two `Qv` constructions with different `CEC` units
(`lrlc.rs:40`, `multimin2.rs:1432-1442`).

**Verified by.** SB-SAT-T30, SB-SAT-T61

---

#### SB-SAT-048 — LRLC coefficients are declared as one field's calibration [P2] [status: PRESENT-UNVERIFIED]

**Requirement.** `sw_rtc`'s `A_CAP`/`B_QV`/`C0`/`RSF` and `sw_imts`'s `S_FACTOR` MUST be
presented as **one study's calibration** and **a placeholder** respectively, never as constants;
each MUST name the in-product calibration route that replaces it; and a run using an unreplaced
shipped coefficient MUST be flagged in the provenance record of SB-SAT-043.

**Rationale.** These are SandiBumi's own methods, so no vendor evidence applies and the standard
is the project's own: the coefficients are field-specific and a foreign calibration *"does not
announce itself: it yields a smooth, plausible Sw that is simply wrong"* (`lrlc.rs:83-90`). `S`
multiplies the whole clay-charge term, so getting it wrong scales `Qv_eff` directly and moves Sw
with no outward sign (`lrlc.rs:191-197`).

**As-built.** `PRESENT-UNVERIFIED` — the doc strings already say all of this in terms, and both
calibrators exist and are extensively tested (`lrlc.rs:1534-2126`). What is missing is the
**flag**: a run on shipped coefficients is indistinguishable in the output from a run on fitted
ones.

**Verified by.** SB-SAT-T59, SB-SAT-T62

---

#### SB-SAT-049 — Carry the Worthington 1985 type per model [P4] [status: ABSENT]

**Requirement.** Where a source states a Worthington 1985 classification for a model, SandiBumi
MUST carry it as model metadata and expose it in the provenance record.

**Rationale.** Geolog states it per module in its own manifests: `sw_indo` **type 4** (noting
that Worthington *"fixes the saturation exponent N at a value of 2, unlike the original
formulae"*), and `sw_sim`, `sw_ws`, `sw_juha`, `sw_dual`, `sw_tot` all **type 2**; `sw_arch`,
`sw_nige` and `sw_pnl` carry none (**T1**, dossier §1.2, MN-3). Techlog Elan adds that the
*original* Simandoux is **Type 1** and the ELAN/modified form **Type 2** (**T3**). It is
classification metadata, not a parameter, and it is free to carry.

**As-built.** `ABSENT`.

**Verified by.** SB-SAT-T59

---

#### SB-SAT-050 — Apparent-`Rw` inversion, one per saturation model [P3] [status: ABSENT]

**Requirement.** SandiBumi SHOULD ship an apparent-`Rw` inversion for each saturation model it
implements, plus resistivity-ratio and SP routes.

**Rationale.** Geolog ships an eight-module mirrored family — `rwa_arch`, `rwa_sim`, `rwa_indo`,
`rwa_juha`, `rwa_dual`, `rwa_tot`, `rwa_res`, `rwa_sp` (**T1**, dossier §1.2c). **Neither IP nor
Techlog documents a per-model `Rwa` inversion at this granularity.** Since SB-SAT-031 requires
`Rw` to ship with no default, the product owes the user a first-class way to *determine* it, and
a per-model inversion is the structurally correct one — the `Rwa` must invert the same equation
the forward run will use.

**As-built.** `ABSENT` — `pickettPanel.ts` provides a Pickett-plot route, which is a different
and coarser mechanism.

**Verified by.** SB-SAT-T63

---

#### SB-SAT-051 — Per-mineral conductivity is a recorded capability gap, not a silent one [P4] [status: ABSENT]

**Requirement.** SandiBumi MUST record that conductive accessory minerals (pyrite in
particular) are **not** represented in any implemented saturation model, and MUST NOT present a
saturation result in pyritic rock as if that conductivity had been accounted for. If a linear
-conductivity model is later implemented, per-mineral conductivities MUST ship `NoDefault`.

**Rationale.** Techlog Elan's linear conductivity model is unique in the whole three-tool set:
*"Unlike any other saturation equation, linear conductivity allows any mineral to have
conductivity associated with it"* — e.g. `CUDC_PYRI`, `CUDC_ILLI` (**T3**
`petrophysics-elanplus-water-saturation-linear-conductivity.html`). **It is the only documented
route in any of the three tools for pyrite conductivity in an Sw solve** (dossier §1.3b). An
admitted gap costs a feature; an unadmitted one costs the deal (CONTRACT §5).

**As-built.** `ABSENT` — no mineral carries a conductivity endpoint.

**Verified by.** SB-SAT-T60

---

## 5. Parameters

Seventy-one rows. Every value is transcribed byte-exact from the dossier with its source
string, or ships `ABSENT — ships with no default`. **Twenty rows ship absent**, and that is the
single most important fact in this section: `a`, `m`, `n`, `m*`, `n*`, `Rw`, `Rsh`, `φt_sh`,
`Qv`, `B`, `vQ0` and the variable-`m` coefficient are the parameters whose wrongness is silent,
and on every one of them the incumbents either disagree, ship an admitted placeholder, or ship
nothing.

Eight rows carry no evidence tier (`—`). Six of those are values **currently in SandiBumi's
source with no citation anywhere in the dossier or the vendor corpus**; they appear here because
they are shipping, not because they are adopted — three are marked `WITHDRAWN` and removed by
SB-SAT-031 and SB-SAT-035, and the rest are re-declared as calibration or placeholders by
SB-SAT-048. The remaining two (`ρ_brine`, the α ceiling) are quantities for which no source
exists in any corpus read, and they ship absent.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Tortuosity / lithology factor | `a` | ABSENT — ships with no default | dimensionless | IP 2018 §3.1 PhiSw and SSM pages state **no default for `a`/`m`/`n`**; Geolog `sw_*.info` DEFAULT=1; Techlog Quanti `a` = 1 | T2 / T1 / T3 |
| Simandoux tortuosity factor | `a` (Simandoux) | ABSENT — ships with no default | dimensionless | Geolog `sw_sim.info` DEFAULT=0.8 — the only module in its own family to differ, and **unattributed by either paper in its own References block**; IP and Techlog 1.0. Worth 4.6 su | T1 / T2 / T3 |
| Cementation exponent | `m` | ABSENT — ships with no default | dimensionless | IP 2018 §3.1 no default; Geolog DEFAULT=2 VALIDATION=1:10; Techlog 2. Jauhar's delivered studies use SCAL-derived values per field | T2 / T1 / T3 / T4 |
| Saturation exponent | `n` | ABSENT — ships with no default | dimensionless | as `m` above | T2 / T1 / T3 / T4 |
| Total Shale saturation exponent | `n` (`total_shale`) | 2 — fixed, not settable | dimensionless | Geolog `sw_tot.info:12` *"the saturation exponent N is set equal to 2"*; the argument table has no `N` row | T1 |
| Waxman-Smits cementation exponent | `m*` | ABSENT — ships with no default | dimensionless | Geolog `sw_ws.info`; IP 2018 §3.4; Techlog | T1 / T2 / T3 |
| Waxman-Smits saturation exponent | `n*` | ABSENT — ships with no default | dimensionless | as `m*` above | T1 / T2 / T3 |
| Formation water resistivity | `Rw` | ABSENT — ships with no default | ohm·m | Geolog takes `RW`/`RWS`/`SALW` as **required inputs with no default**; IP 0.1 ohmm with *"must be adjusted to the correct value"*; Techlog 0.03 ohm.m with no warning. IP↔Techlog = 1.83× on Sw at m = n = 2 | T1 / T2 / T3 |
| `Rw` measurement temperature | `RWT` | ABSENT — ships with no default | °C | Geolog `sw_arch.info` required input | T1 |
| Formation water salinity | `SALW` | ABSENT — ships with no default | ppm NaCl equivalent | Geolog required input | T1 |
| Shale / clay resistivity | `Rsh` | ABSENT — ships with no default | ohm·m | Geolog input log, no default; Techlog `Res_shale` 5; IP interpreter-picked | T1 / T3 / T2 |
| Shale total porosity | `φt_sh` | ABSENT — ships with no default | v/v | Geolog input log, no default; Techlog `Porosity shale` 0.4. `Cwsh(0.10)` = 25.0 vs `Cwsh(0.40)` = 1.5625 mho/m at Rsh 4, m 2 — **16× apart** | T1 / T3 |
| Shale total porosity validation range | `φt_sh` | 0:0.4 accept, 0.4:1 warn | v/v | Geolog `qv.info` VALIDATION=0:0.4 — the tighter of two internal Geolog ranges (`sw_juha.info` and `sw_dual.info` state 0:1); `qv` is the shared support module and feeds the `ρdsh` chain that diverges as `φt_sh → 1` | T1 |
| Simandoux shale exponent | `C` | 1 (default), validation **1:2** | dimensionless | Geolog `sw_sim.info` DEFAULT=1 VALIDATION=1:2, visibility gated on `OPT_SIM:SCHLUM` (`sw_sim.info:69`). `C = 1` reproduces IP E64 and the Techlog raster exactly | T1 |
| Indonesia shale-exponent coefficient | `k` in `Vsh^(2 − k·Vsh)` | 1 = FULL, 0 = SIMPLE, 2 = TAR_SAND | dimensionless | Geolog `sw_indo.info` `OPT_INDO` three-option menu | T1 |
| Counterion conductance | `B` | ABSENT — ships with no default | `L·S/(eq·m)` ≡ `mho·mL/(m·meq)` | Four vendor methods, three factory defaults — see the `B` method row. Getting the unit scale wrong costs 27.2 su (63 % relative on Sw) | T1 / T2 / T3 |
| `B` unit conversion | — | ×100 | `mho·cm²/meq` → `mho·mL/(m·meq)` | Geolog `sw_ws.lls:259-260` divides by 100 and `:289` multiplies by 100, thirty lines apart in one file | T1 |
| `B` method | — | ABSENT — ships with no default | — | IP: Juhász closed form. Geolog: `WAX_THOM`. Techlog: the 1978 Waxman `B` chart (cited by existence and attribution only; no chart values transcribed) | T2 / T1 / T3 |
| Juhász `B` closed form | `B(T, Rw)` | `(−1.28 + 0.225·T − 0.0004059·T²) / (1 + (0.045·T − 0.27)·Rw^1.23)` | result `L·S/(eq·m)`; `T` in **°C**; `Rw` in ohm·m | Juhász 1981. °C confirmed by IP 2025 symbol dictionary, IP 2018, Geolog `FTEMP` unit + Arps constant, and the Techlog parameter table — four sources against one "implied degF" ingest note. °F input gives +47 % on `B` | T2 / T1 / T3 / T4 |
| `B` regression check | `B(25, 0.1)` | 3.89520 | `L·S/(eq·m)` | regression value on the Juhász closed form | T4 |
| `B` regression check | `B(100, 0.05)` | 15.5144 | `L·S/(eq·m)` | regression value on the Juhász closed form | T4 |
| Geolog `WAX_SMIT` `B` at 25 °C | — | 0.046 | `mho·cm²/meq`, pre-salinity-factor | Geolog `sw_ws.lls` `WAX_SMIT` branch. Ratio to `WAX_THOM` = 1.200 — a **ceiling**, since the two carry different salinity factors | T1 |
| Geolog `WAX_THOM` `B` at 25 °C | — | 0.038320 | `mho·cm²/meq`, pre-salinity-factor | Geolog `sw_ws.lls` `WAX_THOM` quartic fit | T1 |
| IP normalized-`Qv` coefficient | `Bn` | 1.0 — **admitted crossplot placeholder; MUST NOT be a default** | dimensionless | IP manual. Costs 14.0 su against the shale-derived coefficient | T2 |
| Cation exchange capacity per unit pore volume | `Qv` | ABSENT — ships with no default | **meq/mL ≡ eq/L ≡ meq/cm³** | Techlog `QV = CEC·ρg·(1−Φ)/Φ` with CEC in meq/g and ρg in g/cm³ arithmetically yields meq/cm³ while the page labels it `1/L`; Geolog `sw_ws.info` labels it `m/c3`. meq/L is a factor of 1000 | T3 / T1 / T4 |
| Dry-clay cation exchange capacity | `CEC_dry` | ABSENT — ships with no default | meq/100 g | Geolog `qv.info` input. SandiBumi's two engines currently use **different** units for this quantity (meq/100 g at `lrlc.rs:100`, meq/g at `multimin2.rs:65`) | T1 |
| Bound-water volume per counterion charge | `vQ0` | ABSENT — ships with no default | mL/meq | Geolog `sw_dual.lls:427` = **0.3 at 22 °C**; Techlog Elan = **0.28 at room temperature**. ~7 % apart and **both cite Clavier, Coates & Dumanoir 1984** (`sw_dual.info:142-144`) — two readings of one paper, not a vendor invention. Escalation ESC-4 | T1 / T3 / T4 |
| `vQ` saline branch (α = 1) | `vQh` | `vQ0 · 320/(T°C + 298)` | mL/meq | Geolog `sw_dual.lls:630-631` | T1 |
| `vQ` expanded branch (α > 1) | `vQh` | `vQ0 · √((273 + T°C)/295)` | mL/meq | Geolog `sw_dual.lls:630-631`. At 100 °C the two branches give 0.24121 and 0.33732 — **28.5 % apart** | T1 |
| Diffuse-layer expansion factor | `α` | `MAX(1, √((γ(0.35)·0.35)/(γ(n)·n)))`, `n = ρw·S[ppm]/(1000·58450)` mol/L | dimensionless | Geolog `sw_dual.lls:365-372`, citing Clavier, Coates & Dumanoir 1984. Dropping the γ ratio gives +6.7 % at 5 000 ppm, +8.9 % at 3 000 ppm | T1 |
| Debye-Hückel activity coefficient | `γ(x)` | `10^(−0.5085·√x / (1 + 0.3281·4.5·√x))` | dimensionless | Geolog `sw_dual.lls:365-372`, citing **Skoog & West, 4th ed.** | T1 |
| α reference molarity | — | 0.35 | mol/L | Geolog `sw_dual.lls`. Note `20455/58450 = 0.34995`, i.e. SandiBumi's ppm threshold is this constant with the conversion folded in | T1 |
| NaCl molar mass (α molarity conversion) | — | 58450 | mg/mol | Geolog `sw_dual.lls` | T1 |
| α ceiling | — | ABSENT — no source | dimensionless | SandiBumi ships `.min(5.0)` at `multimin2.rs:562`. No vendor caps α | — |
| Bound-water equivalent conductance | `β` | `2.05 · (T°C + 8.5)/(22 + 8.5) · (1 − β_const·e^(−2·Cw))` | `mho·mL/(m·meq)` — unit derived from `Cwb = β/vQ`; Geolog states none | Geolog `sw_dual.lls:637-638` | T1 |
| β salinity-dilution switch | `β_const` | 1 (default), validation 0:1 | dimensionless | Geolog `sw_dual.info` DEFAULT=1 VALIDATION=0:1. At Cw 4 the factor is 0.99966; at Cw 1 (Rw 1.0) it is 0.8647, so omission makes `Cwb` **15.7 % high** | T1 |
| `Qv` admissibility bound | — | `Qv ≤ 1/(α·vQh)` | meq/mL | Geolog `sw_dual.lls:129` — documented, and **both enforcing sites ship commented out** (`:449-450`, `:460-461`) | T1 |
| Low-porosity guard threshold | `φe` | 0.005 | v/v | Documented in **all nine** Geolog `sw_*` modules. Below it: all saturations 1 and `VOL_UWAT = VOL_XWAT = φe`, **not 0** | T1 |
| Kennedy ↔ Bateman-Konen switch salinity | — | 39161 | ppm | Geolog `sw_arch.lls:174-195`, identical in 7 sibling modules. The two correlations agree to **0.07 %** at this salinity | T1 |
| Kennedy cap salinity | — | 275000 | ppm | Geolog `sw_arch.lls:174-195` | T1 |
| Kennedy floor `Rw` | — | 0.0412 at 75 °F | ohm·m | Geolog `sw_arch.lls:188`. **All eight `sw_*` doc blocks state 0.412 and are wrong by a factor of ten** (`sw_arch.lls:49`; `sw_dual.lls:327, 357, 392`) | T1 |
| Kennedy polynomial | `Rw75` | `1 / (24.30853 − 0.0364·(S/10000 − 29.46518957) − 0.02922·(S/10000 − 29.46518957)²)` | ohm·m at 75 °F | Geolog `sw_arch.lls:174-195` | T1 |
| Bateman-Konen correlation | `Rw75` | `0.0123 + 3647.5 / S^0.955` | ohm·m at 23.9 °C | Geolog `sw_arch.lls:174-195` | T1 |
| Arps constant, °C form | — | 21.5 | °C | Geolog — bound to the `MEASURED` and Bateman-Konen branches only | T1 |
| Arps constant, °F form | — | 6.77 | °F | Geolog — bound to the Kennedy branch only | T1 |
| Bateman-Konen reference temperature | — | 23.9 | °C | Geolog `sw_arch.lls:174-195` | T1 |
| Kennedy reference temperature | — | 75 | °F | Geolog `sw_arch.lls:174-195` | T1 |
| Root-finder seed | — | 0.5 | v/v | Geolog `CALC_SW`, `sw_sim.lls:256-271` | T1 |
| Root-finder iteration budget | — | 20 | iterations | Geolog `CALC_SW`; non-convergence sets `sat = MISSING` | T1 |
| Root-finder convergence tolerance | — | 1e-5 | v/v, absolute on `Δsat` | Geolog `CALC_SW` | T1 |
| Clay-bound-water fraction, Kppm form | `F` | `1 − (0.6425·S[Kppm NaCl-equiv]^(−0.5) + 0.22)·Qv[meq/mL]` | v/v | IP, ledger D-07 reading (i) | T2 |
| Clay-bound-water fraction, molar form | `F` | `1 − (0.084·C₀[meq/cm³]^(−0.5) + 0.22)·Qv[meq/cm³]` | v/v | Techlog `clay-bound-water-correction-004.png`, verified at 6×. **`0.084` is printed to two significant figures**, so the fourth digit of the Kppm form is not certified by this route | T3 |
| `F` unit bridge (D-07 resolving evidence) | — | `0.084·√58.44 = 0.6421469`, i.e. 0.055 % from IP's 0.6425 | — | Derived. Decisive against reading (ii), which would move the constant term by 36 % | T2 / T3 |
| NaCl equivalent weight for `C₀` | — | 58.44 | g/mol | Techlog, `C₀ = Salinity[g/L] / 58.44` | T3 |
| Brine density for the Kppm→g/L bridge | `ρ_brine(S, T)` | ABSENT — ships with no default | g/cm³ | **No source in the corpus.** Kppm is a mass fraction and g/L a mass concentration; the ÷58.44 bridge is exact only at ρ_brine = 1.0. Safe in the fresh-water band (3–13 kppm); ~19 % error in `C₀` at 250 Kppm where brine is ≈1.19 | — |
| SSM bound-water cap | — | `Vbw ≤ 1.5 × Vcl × PhiTclay`, followed by `PhiT = Phie + Vbw` | v/v | IP `sand_silt_malay_model.htm`, via `B_core_petro.md:244-253, 839, 1060`. Hard-coded constant, **not** a parameter | T2 |
| Shell / Elan variable-`m` coefficient | `c` in `m = m₀ + c/φ` | ABSENT — ships with no default | v/v (`m` per unit φ) | Ledger **D-10, OPEN**. IP rasters give 0.018 (verified 6× and 4×, both editions); IP ASCII prose gives 0.019 (9+ occurrences, both editions); published Shell source agrees with the ASCII. **Techlog Elan implements the same form (`image2957.gif` eq. 77) but ships `MC2 = 0.0`** — no cross-tool arbitration exists. At φe 0.02 the readings are 10 % apart on Sw; negligible above φ ≈ 0.10. Escalation ESC-2 | T2 / T3 / T4 |
| Waxman-Smits `m*` from `Qv` | — | `m* = m + Cm(1.128Y + 0.22(1 − e^(−17.3Y)))`, `Y = Qv·φt/(1−φt)` | dimensionless | IP 2018 §3.4. Consumes `B`, so a `B`-method change moves `m*` | T2 |
| Dual-water `m*` from `Qv` | — | `m* = m + Cm(0.258Y + 0.20(1 − e^(−16.4Y)))` | dimensionless | IP 2018 §3.4 | T2 |
| `m*` / `n*` from core | — | `F* = F_app(1 + B·Qv·Rw)`, `m* = −log₁₀F*/log₁₀φt`; `RI* = RI_app(1 + B·Qv·Rw)/(1 + B·Qv·Rw/Sw)`, `n* = −log₁₀RI*/log₁₀SwT` | dimensionless | Geolog `ffcec.info`, `ricec.info` — a separate pre-processing module, not an in-solve fit. The preferred route where core exists | T1 / T4 |
| Flushed-zone limits, water-base mud | `Sxo` | `Sw^SxoLimit ≥ Sxo ≥ Sw` | v/v | IP E79-E81 | T2 |
| Flushed-zone limits, oil-base mud | `Sxo` | `Sw ≥ Sxo` | v/v | IP E79-E81 | T2 |
| Mud base | `MUDBASE` | `WATER` (default), `OIL` | — | Geolog `sw_dual.info:164` and `sw_sim.info:74` — **these two modules only**; `sw_arch`, `sw_ws`, `sw_juha`, `sw_indo`, `sw_tot` have no `MUDBASE` at all | T1 |
| Worthington 1985 type | — | `sw_indo` type 4; `sw_sim`, `sw_ws`, `sw_juha`, `sw_dual`, `sw_tot` type 2; `sw_arch`, `sw_nige`, `sw_pnl` none. Techlog: original Simandoux type 1, ELAN/modified type 2 | — | Geolog module manifests; Techlog Elan concept pages | T1 / T3 |
| Nigeria half-exponent | — | ABSENT — ships with no default | dimensionless | Geolog ships `sw_nige`; no equation form or coefficient is held in the corpus at a transcribable level | T1 |
| LRLC RtC calibration set | `A_CAP`, `B_QV`, `C0`, `RSF` | 0.45, 0.0057, −0.0071, 2.25 | mixed | SandiBumi `lrlc.rs:96-99`. **One study's calibration, not constants** — declared as such at `lrlc.rs:83-90`; replaced by `run_rtc_fit` | — |
| LRLC IMTS scaling factor | `S_FACTOR` | 0.5 | dimensionless | SandiBumi `lrlc.rs:205`. **Placeholder** — replaced by `run_s_factor_fit`. Multiplies the whole clay-charge term | — |
| LRLC kaolinite / illite CEC | `CEC_KAOL`, `CEC_ILL` | 8, 25 | meq/100 g | SandiBumi `lrlc.rs:206-207`. Uncited in the dossier and the vendor corpus | — |
| SandiBumi shale resistivity | `Rsh` | 4.0 — **WITHDRAWN** by SB-SAT-035 | ohm·m | No source. Matches no vendor (Techlog is 5). `multimin2.rs:377-379`, `src/ui/multiminDialog.ts:430, 818` | — |
| SandiBumi shale porosity | `φ_sh` | 0.10 — **WITHDRAWN** by SB-SAT-035 | v/v | No source. Matches no vendor (Techlog is 0.4). `multimin2.rs:383-385`, `src/ui/multiminDialog.ts:820` | — |
| SandiBumi `Rw` defaults | `Rw` | 0.1 (`modules.rs:1943`) and 0.3 (`lrlc.rs:93, 200`) — **WITHDRAWN** by SB-SAT-031 | ohm·m | 0.1 is IP's value, rejected above; 0.3 has no source. The two put SandiBumi's own engines √3 = 1.73× apart on Sw before the user touches anything | — |

**Values deliberately not transcribed.** No node values from the Techlog 1978 Waxman `B` chart,
the Western Atlas Charts 1994 p. 27 `Rw` chart cited by Geolog, or any Schlumberger, Halliburton,
Baker, Weatherford, Sperry-Sun, PathFinder, Anadrill or GE chart. Each is cited above by
existence, attribution and purpose only (CONTRACT §2.1). No Tier-C algorithm is parameterised
here; Omovie Sonic Saturation contributes no row.

---

## 6. Acceptance tests

Sixty-three tests. Each states its input, its operation, its expected output **with a
tolerance**, and the source of the expected value. Tests whose expected value is SandiBumi's own
current output rather than an external reference are labelled `CHARACTERIZATION` — they pin
behaviour, they do not prove correctness.

Where the dossier supplies a computed cross-tool value it is used verbatim. Where it does not,
the test is written as an **invariant** (a relation that must hold between two SandiBumi
outputs, or between an output and a bound) rather than against a number this chapter would have
had to invent. That distinction is deliberate: an invented expected value is an uncited
parameter wearing a test's clothes.

---

**SB-SAT-T01 — Alias table resolves every catalogued vendor name.**
*Input:* the vendor model-name and method-flag strings of SB-SAT-003.
*Operation:* resolve each through the alias table.
*Expected:* each maps to the stated SandiBumi identifier; Techlog Quanti `Dual water` maps to
`juhasz` and **not** to either `dual_water_*`; Geolog `MODIFIED` and `SIM_MOD` both map to
`simandoux_bardon_pied`; Geolog `SCHLUM` and `SIM_SCHL` both map to `simandoux_modified_slb`.
Exact string match, no tolerance.
*Source:* dossier §1.1–§1.3, §2.7; Geolog `sw_sim.lls:146, 148, 210, 228`.

**SB-SAT-T02 — No user-facing model name is a bare vendor adjective.**
*Input:* the full list of exposed model identifiers and UI labels.
*Operation:* assert none equals `Modified`, `Simandoux`, `Modified Simandoux` or `Dual water`
without an equation-naming suffix; assert every enum variant, doc comment and label for one
model agree.
*Expected:* zero violations. Currently fails at `multimin2.rs:115, 164`.
*Source:* dossier §2.2, §3.2 — the naming trap costs 7.3 su.

**SB-SAT-T03 — Effective and total Archie differ on the reference case.**
*Input:* the dossier's Archie reference case.
*Operation:* run `archie_effective` and `archie_total` on identical inputs.
*Expected:* **0.884** and **0.634**, a separation of **25.0 saturation units**; tolerance
±0.002 v/v on each.
*Source:* dossier §3.1.

**SB-SAT-T04 — Archie is exactly its equation.**
*Input:* a grid of `a`, `m`, `n`, `Rw`, `Rt`, φ spanning the validation ranges.
*Operation:* `archie_effective` and `archie_total`.
*Expected:* both reproduce `(a·Rw/(Rt·φ^m))^(1/n)` to ±1e-12 relative on the respective
porosity; `Sw` monotone decreasing in `Rt` and in φ.
*Source:* Archie 1942, cited in Geolog `sw_arch.info`.

**SB-SAT-T05 — An unmapped foreign model name fails rather than guesses.**
*Input:* a parameter set naming a model absent from the alias table.
*Operation:* import.
*Expected:* error naming the unresolved string. No fallback to a "nearest" model, no default
model substituted.
*Source:* CONTRACT §2; dossier MN-2.

**SB-SAT-T06 — The two Simandoux variants are 7.3 saturation units apart.**
*Input:* the dossier's Simandoux fixture.
*Operation:* `simandoux_bardon_pied` and `simandoux_modified_slb` at identical `a`, `m`, `n`,
`Rw`, `Rsh`, `Vsh`, φe.
*Expected:* separation **7.3 saturation units**, with the corresponding HCPV difference
**+19 %**; tolerance ±0.002 v/v on the separation.
*Source:* dossier §3.2.

**SB-SAT-T07 — `C` is scoped to one variant and validated 1:2.**
*Input:* `C` = 0.9, 1.0, 1.5, 2.0, 2.1 against each variant.
*Operation:* construct the model.
*Expected:* `simandoux_bardon_pied` rejects any `C`; `simandoux_modified_slb` accepts 1.0–2.0
and rejects 0.9 and 2.1. At `C = 1` the modified variant reproduces IP E64 and the Techlog
raster form to ±1e-12 relative.
*Source:* Geolog `sw_sim.info:69`, DEFAULT=1 VALIDATION=1:2; IP C E64; Techlog
`modules-quanti-saturation-simand.gif`.

**SB-SAT-T08 — Simandoux `a` 0.8 against 1.0 is 4.6 saturation units.**
*Input:* the §3.2 reference case.
*Operation:* run at `a` = 0.8 and `a` = 1.0.
*Expected:* separation **4.6 saturation units**; tolerance ±0.002 v/v.
*Source:* dossier §3.2 — the reason `a` ships absent rather than defaulted either way.

**SB-SAT-T09 — Indonesia reproduces all three vendors.**
*Input:* a shared fixture across `Vsh`, φe, `Rsh`, `Rt`.
*Operation:* `indonesia` at `k = 1`.
*Expected:* agrees with the IP E65/E66 form and the Techlog raster form to ±1e-12 relative.
*Source:* dossier §2.4; Poupon & Leveaux 1971 Paper O.

**SB-SAT-T10 — The `k` presets are distinct and TAR_SAND is Woodhouse.**
*Input:* `Vsh` sweep 0→1.
*Operation:* `indonesia` at `k` = 0, 1, 2; and `woodhouse_tar`.
*Expected:* the three `k` values give three distinct curves; `k = 2` and `woodhouse_tar` agree
bit-for-bit; `k = 2` reproduces IP's `Vcl^(1−Vcl)` half-exponent form to ±1e-12 relative.
*Source:* Geolog `sw_indo.info:50-54`; IP C E66; dossier §2.4.

**SB-SAT-T11 — Total Shale is Simandoux-SLB at `C = 1`, `n = 2`.**
*Input:* a shared fixture.
*Operation:* `total_shale` against `simandoux_modified_slb` at `C = 1`, `n = 2`.
*Expected:* agreement to ±1e-12 relative. Supplying `n ≠ 2` to `total_shale` is a compile-time
error (asserted by a compile-fail test).
*Source:* Geolog `sw_tot.lls:150-156`, `sw_tot.info:12`.

**SB-SAT-T12 — The `n = 2` closed form equals the general solver.**
*Input:* a randomised sweep of `g1`, `g2b`, `g3` spanning the physical domain.
*Operation:* solve by the closed-form quadratic root and by the general iterative solver.
*Expected:* agreement to ±1e-9 absolute in `Sw`, at every point where both converge.
*Source:* algebraic identity; guards against the fast path drifting from the general path.

**SB-SAT-T13 — Juhász uses the shale-derived coefficient and the model's own `m*`.**
*Input:* `Vsh`, `φt_sh`, `φt`, `Rsh`, `Rw`, `m*` = 1.6.
*Operation:* compute `Cwsh` and `Qvn`.
*Expected:* `Cwsh = 1/(Rsh·φt_sh^1.6)`, not `1/(Rsh·φt_sh²)`; the two differ by **44 % on
`Cwsh`** at `m* = 1.6` and produce **opposite signs** on the excess-conductivity coefficient.
`Qvn = clamp(Vsh·φt_sh/φt, 0, 1)`. No `a` factor appears. Tolerance ±1e-12 relative.
*Source:* Geolog `sw_juha.lls:34, 42, 54, 251-253`; Techlog `image1252.gif`; dossier §3.6.

**SB-SAT-T14 — `Bn = 1` costs 14 saturation units.**
*Input:* the dossier's Juhász fixture.
*Operation:* run with the shale-derived coefficient and with IP's `Bn = 1.0`.
*Expected:* separation **14.0 saturation units**; tolerance ±0.002 v/v.
*Source:* dossier §3.5.

**SB-SAT-T15 — A negative excess-conductivity coefficient is flagged.**
*Input:* `Rw` 0.25 (⇒ `Cw` 4.0 mho/m), `Rsh` 3, `φt_sh` 0.35 (⇒ `Cwsh` 2.721 mho/m).
*Operation:* `juhasz`.
*Expected:* coefficient `Cwsh − Cw` = **−1.279 mho/m** (±0.001), and the sample carries a
raised flag. A run that returns a saturation with no flag fails the test.
*Source:* dossier §3.5. Neither Geolog nor Techlog warns here; IP cannot reach the condition.

**SB-SAT-T16 — Waxman-Smits exposes `a`.**
*Input:* `a` = 0.62, 1.0.
*Operation:* `waxman_smits`.
*Expected:* the two runs differ; at `a = 1` the result agrees with the Techlog rendered form
(which omits `a`) to ±1e-12 relative, and with the IP E70 and Geolog `sw_ws.lls:35, 43`
expansions at any `a`.
*Source:* dossier §2.5.

**SB-SAT-T17 — The wrong `B` unit scale is unrepresentable, and would cost 27.2 su.**
*Input:* a `B` value in `mho·cm²/meq`.
*Operation:* attempt to pass it into a saturation equation without conversion.
*Expected:* compile-time type error (asserted by a compile-fail test). Separately, a numeric
fixture confirms that the unconverted value moves `Sw` by **27.2 saturation units — 63 %
relative** in the conservative direction; tolerance ±0.005 v/v.
*Source:* dossier §3.3; Geolog `sw_ws.lls:259-260, :289`.

**SB-SAT-T18 — The `B` converter round-trips.**
*Input:* a sweep of `B` values.
*Operation:* `to_mho_cm2_per_meq` then `to_canonical`.
*Expected:* identity to ±1e-12 relative; the forward factor is exactly 100.
*Source:* Geolog `sw_ws.lls:259-260, :289`.

**SB-SAT-T19 — `Qv` unit identity.**
*Input:* a sweep of `Qv` values.
*Operation:* construct in meq/mL, eq/L and meq/cm³ and compare.
*Expected:* all three are the identity — the same canonical value, no scaling. Tolerance zero
(bit-exact).
*Source:* Techlog `petrophysics-qv-function-cec.html` arithmetic; Geolog `sw_ws.info` "m/c3".

**SB-SAT-T20 — `Qv` in meq/L is rejected.**
*Input:* a `Qv` typed as meq/L.
*Operation:* pass to any excess-conductivity model.
*Expected:* compile-time type error unless explicitly converted; the explicit converter applies
a factor of exactly 1/1000.
*Source:* memory `reference_waxman_smits_b` — a documented failure mode on this machine.

**SB-SAT-T21 — `B(T, Rw)` regression values, and °F is rejected.**
*Input:* (25 °C, 0.1 ohm·m) and (100 °C, 0.05 ohm·m).
*Operation:* `waxman_b`.
*Expected:* **3.89520** and **15.5144**, tolerance ±1e-4 absolute. Separately, passing a
temperature typed °F is a compile-time error; a numeric fixture confirms that °F evaluated as °C
gives **+47 % on `B`** and **~20 % relative on `Sw`**.
*Source:* dossier §3.4; existing regression at `multimin2.rs:3833-3856`.

**SB-SAT-T22 — `B` is clamped non-negative.**
*Input:* T = 0, 3, 6, 10 °C.
*Operation:* `waxman_b`.
*Expected:* `B ≥ 0` at every point; the raw numerator is negative below ≈6 °C and the clamp
fires there. Tolerance ±1e-12.
*Source:* the Juhász closed form's numerator root; Geolog and IP both bound `B` at zero.

**SB-SAT-T23 — The `B` method selector has no default and four options.**
*Input:* a Waxman-Smits run with no `B` method chosen.
*Operation:* execute.
*Expected:* error naming the missing choice. The selector offers Juhász closed form, `WAX_SMIT`,
`WAX_THOM`, `GRAVEST` and user-defined; `WAX_SMIT` and `WAX_THOM` differ at 25 °C in the ratio
**0.046 / 0.038320 = 1.200** before their (different) salinity factors, tolerance ±0.001 on the
ratio.
*Source:* dossier §3.9.

**SB-SAT-T24 — The two dual-water forms are distinct models.**
*Input:* one fixture.
*Operation:* `dual_water_cec` and `dual_water_simple`.
*Expected:* different results in general; and where `dual_water_cec`'s CEC is absent so its
fallback `Swb = 1 − φe/φt` applies, the two agree bit-for-bit — the fallback **is** the simple
form.
*Source:* Geolog `sw_dual.lls:471-484`; IP C E67; dossier §2.7.

**SB-SAT-T25 — Techlog's "Dual water" lands on Juhász.**
*Input:* a Techlog Quanti Dual-water parameter set.
*Operation:* import and run.
*Expected:* resolves to `juhasz`; result agrees with the Techlog raster form to ±1e-12 relative;
does **not** resolve to either `dual_water_*`.
*Source:* Techlog `modules-quanti-saturation-dualw.gif`; dossier §2.7.

**SB-SAT-T26 — The excess-conductivity coefficient is `Swb·(Cwb − Cw)` and depends on α.**
*Input:* T = 100 °C at 25 000, 5 000 and 3 000 ppm.
*Operation:* compute `g2`, then recompute with α forced to 1.
*Expected:* the α-dependent share of `g2` is **36.5 %, 21.8 % and 17.7 %** respectively,
tolerance ±0.5 percentage points. A `β·Qv` implementation would show 0 % and fails.
*Source:* dossier §2.7, BL-3.

**SB-SAT-T27 — `vQ` switches temperature form on the α branch.**
*Input:* T = 100 °C, `vQ0` = 0.30 mL/meq, with α = 1 and α > 1.
*Operation:* compute `vQh`.
*Expected:* **0.24121** and **0.33732** mL/meq, tolerance ±1e-5 — a separation of **28.5 %**.
Cross-check: `0.8087 / 2.3976 = 0.33732`.
*Source:* Geolog `sw_dual.lls:630-631`; dossier §2.7.

**SB-SAT-T28 — α includes the Debye-Hückel activity ratio.**
*Input:* 5 000 ppm and 3 000 ppm.
*Operation:* compute α with the full γ ratio.
*Expected:* **1.8949** and **2.3976**, tolerance ±0.0005. The γ-free approximation `√(0.35/n)`
gives values **+6.7 %** and **+8.9 %** high and fails the test.
*Source:* Geolog `sw_dual.lls:365-372`, citing Skoog & West 4th ed.; dossier §2.7.

**SB-SAT-T29 — β carries the salinity dilution factor.**
*Input:* `Cw` = 4.0 and `Cw` = 1.0 mho/m, `β_const` = 1.
*Operation:* compute β.
*Expected:* dilution factor **0.99966** and **0.8647**, tolerance ±1e-4. Omitting the factor
makes `Cwb` **15.7 % high** at `Cw` = 1. At `β_const` = 0 the collapsed constant reproduces
Geolog's chain: `0.0007` against `0.00070013`, agreeing to ±0.02 % relative.
*Source:* Geolog `sw_dual.lls:637-638`, `sw_dual.info`.

**SB-SAT-T30 — Engine parity: one model, one number.**
*Input:* for every model implemented in both the deterministic module engine and the mineral
solver, one shared fixture with identical parameters.
*Operation:* run both.
*Expected:* agreement to ±1e-6 absolute in `Sw`. Where the solver's φt ≡ φe + v_bw construction
makes exact agreement impossible, the test asserts the **documented** difference and its bound,
and fails if the difference is undocumented. Currently fails on Simandoux (different equations)
and Indonesia (`k` variants missing from the solver).
*Source:* SandiBumi-internal; no vendor counterpart.

**SB-SAT-T31 — No parameter ships a default without a source string.**
*Input:* the full saturation parameter registry.
*Operation:* enumerate at build time.
*Expected:* every entry is either `NoDefault` or carries a non-empty, structurally valid source
string. Zero exceptions. The build fails otherwise. `a`, `m`, `n`, `m*`, `n*`, `Rw`, `RWT`,
`SALW`, `Rsh`, `φt_sh`, `Qv`, `CEC_dry`, `B`, the `B` method, `vQ0`, `ρ_brine`, the Shell `c`
coefficient and the Nigeria half-exponent are asserted `NoDefault` by name.
*Source:* CONTRACT §2; IP FINDINGS §6 rule 9.

**SB-SAT-T32 — `Qv` above its admissibility bound flags; `Swb` clamps separately.**
*Input:* `Qv` set above `1/(α·vQh)`; separately, a case where `Swb` would exceed `1 − φe/φt`.
*Operation:* `dual_water_cec`.
*Expected:* the first raises a flag and leaves `Qv` unmodified; the second clamps `Swb` and is
observable as a clamp. Two distinct mechanisms, two distinct diagnostics.
*Source:* Geolog `sw_dual.lls:129`, with its enforcement shipped commented out at `:449-450`,
`:460-461`.

**SB-SAT-T33 — `vQ0` has no default and both candidates are offered.**
*Input:* a `dual_water_cec` run with no `vQ0`.
*Operation:* execute.
*Expected:* error naming `vQ0`; the offered candidates are **0.30 mL/meq at 22 °C** (Geolog) and
**0.28 cm³/meq at room temperature** (Techlog Elan), each with its source, and each attributed to
Clavier, Coates & Dumanoir 1984. The two are ~7 % apart; tolerance ±0.1 percentage point on that
ratio.
*Source:* Geolog `sw_dual.lls:427`, `sw_dual.info:142-144`; Techlog Elan.

**SB-SAT-T34 — The effective back-out and its degeneracy.**
*Input:* φt 0.30, φe 0.20, a sweep of `SWT`; and separately `Swb` = 1.
*Operation:* apply the back-out.
*Expected:* `SWE = (SWT − Swb)/(1 − Swb)` agrees with `1 − (φt/φe)(1 − SWT)` to ±1e-12 relative
for the porosity-split models; `SWE ≥ 0` always; at `Swb` = 1 the result is `SWE` = 1 with no
division by zero.
*Source:* IP C E78; Geolog `sw_ws.lls:296`.

**SB-SAT-T35 — Every model declares its citation and its `Swb` rule.**
*Input:* the model registry.
*Operation:* enumerate.
*Expected:* every model carries a literature citation and an explicitly declared `Swb` rule —
porosity split, `Qvn`, or CEC-derived. `woodhouse_tar` carries Woodhouse 1976 SPWLA 17th Paper T.
No model carries a blanket or implicit rule. On the dossier's Juhász fixture the two competing
rules give `Qvn` **0.42** against `1 − φe/φt` **0.20**, and the test asserts the declared rule is
the one applied.
*Source:* Geolog module manifests; dossier §2.6, BL-6.

**SB-SAT-T36 — Effective↔total round-trip is the identity.**
*Input:* a sweep of `Sw`, `Sxo`, `Swb`.
*Operation:* `SwT = Sw(1 − Swb) + Swb` then the back-out; and the same for `SxoT`.
*Expected:* identity to ±1e-12 relative, for `Swb` in [0, 1).
*Source:* IP `B_core_petro.md:276-280, :446-454`.

**SB-SAT-T37 — `SWE_IRR` is transformed, not scaled.**
*Input:* φt 0.30, φe 0.20, `SWT_IRR` 0.20.
*Operation:* compute `SWE_IRR`.
*Expected:* **0.0**, tolerance ±1e-12. Geolog `sw_ws`'s `PHIT·SWT_IRR/PHIE` form gives **0.30**
and fails. Also asserted: `SWE_IRR` monotone non-decreasing in `SWT_IRR`, never > 1, and 0 at
`Swb` = 1.
*Source:* Geolog `sw_arch.lls:234`, `sw_juha.lls:271`, `sw_dual.lls:552` — and the 2008 fix's
omission from `sw_ws.lls:302`, which this test forbids.

**SB-SAT-T38 — Every method emits a clipped and an unclipped curve.**
*Input:* an interval driving `Sw` above 1 and below `SWE_IRR`.
*Operation:* run every method.
*Expected:* the clipped curve is bounded to `[SWE_IRR, 1]`; the unclipped curve exceeds those
bounds where the model does; both a total and an effective unclipped counterpart exist wherever
the method produces both. Zero methods emit clipped values only.
*Source:* dossier §2.9 — Geolog and Techlog ship unclipped diagnostics, IP does not.

**SB-SAT-T39 — No bare `SW`, and `VOL_UWAT`/`VOL_XWAT` accompany.**
*Input:* every emitted mnemonic across every method.
*Operation:* enumerate.
*Expected:* no mnemonic equals `SW` or `SXO`; every saturation mnemonic carries an `E` or `T`
designator; `VOL_UWAT` and `VOL_XWAT` are present alongside.
*Source:* ledger D-15; Geolog's family, against Techlog's `SW_AR`.

**SB-SAT-T40 — A method-flag curve accompanies every saturation run.**
*Input:* a multi-method run.
*Operation:* execute.
*Expected:* a flag curve records the producing model at every sample; its values resolve through
the SB-SAT-003 alias table.
*Source:* Geolog emits `OPT_SW` as a first-class output; dossier §4.3.

**SB-SAT-T41 — Solver guards, and non-convergence returns null.**
*Input:* a set of well-posed cases; and a deliberately non-converging case.
*Operation:* run every polynomial-form model.
*Expected:* seed 0.5, at most **20** iterations, tolerance `|Δ| < 1e-5`, `sat ≥ 0` at every
step. The non-converging case returns **null**, never the last iterate. Asserted for both engines
and for `sw_imts`, which currently fails it at `lrlc.rs:271-290`.
*Source:* Geolog `CALC_SW`, `sw_sim.lls:256-271`.

**SB-SAT-T42 — Low porosity declares wet, not empty.**
*Input:* φe = 0.004 with φt > 0; and φe = φt = 0.
*Operation:* run every method.
*Expected:* first case — all saturations 1 and **`VOL_UWAT` = `VOL_XWAT` = φe**, not 0
(tolerance ±1e-12). Second case — all saturations 1 and all volumes 0.
*Source:* documented in all nine Geolog `sw_*` modules; dossier MN-4.

**SB-SAT-T43 — Missing or invalid inputs null the outputs.**
*Input:* `Rt` missing; `Rt` ≤ 0; a missing variable-`m` input curve.
*Operation:* run.
*Expected:* every saturation output null in the first two cases; every saturation **and volume**
output null with an emitted message in the third.
*Source:* Geolog `sw_*` documented behaviour; dossier §2.9.

**SB-SAT-T44 — `Vsh → 1` flags before the singularity.**
*Input:* `Vsh` = 0.98, 0.999, 1.0.
*Operation:* `simandoux_modified_slb` and `indonesia`.
*Expected:* a flag is raised on approach and at the limit. Returning `Sw` = 1 is permitted;
returning it **unflagged** fails.
*Source:* Techlog Elan — the only vendor documenting the 0/0 at `Vcl` = 100 % and recommending a
water-volume constraint of about 0.5 p.u.

**SB-SAT-T45 — `Rw` has no default and the `MEASURED` branch is Arps in °C.**
*Input:* a run with no `Rw`, `RWS`/`RWT` or `SALW`; then `RWS`, `RWT` supplied.
*Operation:* resolve `Rw`.
*Expected:* first case errors naming the missing input — no 0.1, no 0.3, no 0.03. Second case
gives `RWS·(RWT + 21.5)/(T + 21.5)` to ±1e-12 relative, with the **21.5 °C** constant, never
6.77.
*Source:* Geolog `sw_arch.lls:174-195`; dossier §3.8.

**SB-SAT-T46 — The two `Rw` correlations agree at the switch salinity.**
*Input:* 39 161 ppm.
*Operation:* evaluate both the Kennedy and Bateman-Konen branches at a common temperature.
*Expected:* agreement to **0.07 %** relative, tolerance ±0.02 percentage points. This is both the
transcription check and the reason a branch/conversion swap is invisible in output.
*Source:* dossier §5.2.

**SB-SAT-T47 — The Kennedy floor is 0.0412, and the vendor doc is wrong.**
*Input:* 300 000 ppm.
*Operation:* resolve `Rw` at 75 °F.
*Expected:* **0.0412 ohm·m**, tolerance ±1e-6. The test carries an inline comment recording that
all eight Geolog doc blocks state **0.412** and are wrong by a factor of ten, so that a future
reader does not "correct" the test to the vendor documentation.
*Source:* Geolog `sw_arch.lls:188` against `:49`; `sw_dual.lls:327, 357, 392`.

**SB-SAT-T48 — Each temperature conversion is bound to its own branch.**
*Input:* salinities either side of 39 161 ppm.
*Operation:* resolve `Rw`.
*Expected:* the Kennedy branch uses **6.77 °F from 75 °F**; the Bateman-Konen branch uses
**21.5 °C from 23.9 °C**. A test double that swaps the two is detected — asserted by evaluating
away from the switch salinity, where the swap is numerically visible.
*Source:* Geolog `sw_arch.lls:174-195` and 7 siblings.

**SB-SAT-T49 — `a`, `m`, `n`, `m*`, `n*` have no defaults.**
*Input:* a saturation run omitting each in turn.
*Operation:* execute.
*Expected:* error naming the specific missing exponent. No 1.0, no 2.0, no 1.9 substituted, in
either engine.
*Source:* IP 2018 §3.1; CONTRACT §2.

**SB-SAT-T50 — `Rsh` and `φt_sh` have no defaults, and `φt_sh` is range-checked.**
*Input:* a shaly-sand run omitting each; and `φt_sh` = 0.35, 0.45, 1.1.
*Operation:* execute.
*Expected:* errors naming the missing parameter — **no 4.0, no 0.10** anywhere in backend or UI.
`φt_sh` 0.35 accepted silently, 0.45 accepted with a warning, 1.1 rejected. A fixture confirms
the stakes: `Cwsh(0.10)` = **25.0** against `Cwsh(0.40)` = **1.5625** mho/m at `Rsh` 4, `m` 2 —
16× — tolerance ±0.001 mho/m.
*Source:* Geolog `qv.info` VALIDATION=0:0.4; dossier MN-7.

**SB-SAT-T51 — Both `m*`/`n*` routes exist and core is preferred.**
*Input:* a dataset with core `F`/`RI`/CEC; and one without.
*Operation:* derive `m*` and `n*`.
*Expected:* with core, `mstar_from_core` is selected and reproduces
`F* = F_app(1 + B·Qv·Rw)`, `m* = −log₁₀F*/log₁₀φt` to ±1e-12 relative; without core,
`mstar_from_qv` is selected and reproduces the IP `Cm` relations. Changing the `B` method changes
both `m*` and `n*`, and the run warns that it has.
*Source:* Geolog `ffcec.info`, `ricec.info`; IP 2018 §3.4.

**SB-SAT-T52 — The variable-`m` route is parameterised and `c` has no default.**
*Input:* a run selecting `m = m₀ + c/φ` with no `c`.
*Operation:* execute.
*Expected:* error naming `c`; the presets 0.018 (IP raster) and 0.019 (IP ASCII / published
Shell) are offered with their sources, and Elan's `MC2 = 0.0` is recorded as a third, non-
arbitrating source. A fixture confirms the stakes at φe 0.02: `0.02^(−0.025)` = **1.103**,
i.e. ~10 % on `Sw`, tolerance ±0.002 on the ratio; and confirms the difference is < 1 % above
φ = 0.10.
*Source:* ledger D-10; dossier §3.10.

**SB-SAT-T53 — `MUDBASE` is scoped to two models and drives the OBM branch.**
*Input:* `MUDBASE` requested on `archie_total`, `waxman_smits`, `juhasz`, `indonesia`,
`total_shale`; then `MUDBASE = OIL` on `dual_water_cec`.
*Operation:* construct and run.
*Expected:* the first five are compile-time errors. On the sixth, `α_x = α_u`, `β_x = β_u`,
`SXOT` is solved against `Rw` at formation temperature and not against `Rmf`, and `Rmf`/`ρ_mf`
are gated off.
*Source:* Geolog `sw_dual.info:164`, `sw_sim.info:74`, `sw_dual.lls:444, 496, 567-572`;
dossier MN-6.

**SB-SAT-T54 — Both `F` unit forms agree through the named bridge.**
*Input:* a salinity/`Qv` pair expressed in each form's native units.
*Operation:* compute `F` both ways.
*Expected:* agreement within the bridge's own precision — `0.084·√58.44` = **0.6421469**, which
is **0.055 %** from IP's 0.6425; tolerance ±0.1 % on `F`'s coefficient term. Neither form is
stored normalised into the other; conversion passes through the named function.
*Source:* ledger D-07; Techlog `clay-bound-water-correction-004.png` at 6×.

**SB-SAT-T55 — `Swb = 1 − F` is opt-in, and the density bridge is flagged.**
*Input:* a run at 60 Kppm; and a `Swb` request with no explicit `F` opt-in.
*Operation:* execute.
*Expected:* the first raises the ≥ 3 % unit-bridge divergence flag above ~50 Kppm and errors on
the absent `ρ_brine` rather than assuming 1.0 g/cm³. The second uses `1 − φe/φt` or the CEC
chain, never `1 − F`. Both vendor uses of `F` — `PcCorr = Pc·F^(−0.5)` and
`SwPcCorr = 1 − (1 − SwPc)·F` — are present and exercised.
*Source:* dossier MJ-6, MJ-7.

**SB-SAT-T56 — Poupon-Aguilera and Poupon-Tixier are distinct and cited.**
*Input:* one fixture.
*Operation:* run both.
*Expected:* `poupon_aguilera` uses `g1 = φe^m/(a·Rw·(1−Vcl))` and `poupon_tixier` uses
`g1 = (1−Vcl)·φe^m/(a·Rw)`; the two differ; both carry their full citations; the alias
`Poupon` → `poupon_aguilera` resolves.
*Source:* IP `A_porosity_sw.md:330-331, :333-334`.

**SB-SAT-T57 — The laminated interlock fires.**
*Input:* `poupon_aguilera` or `poupon_tixier` with a laminated `Sw` model enabled.
*Operation:* execute.
*Expected:* refusal or a hard warning naming double-correction for laminations. A silent run
fails.
*Source:* IP `A_porosity_sw.md:697-699`, verbatim.

**SB-SAT-T58 — The SSM bound-water cap fires, re-sets `PhiT`, and flags.**
*Input:* a shaly interval where `Vbw` would exceed `1.5 × Vcl × PhiTclay`.
*Operation:* run the sand-silt-clay-equivalent path into a saturation model.
*Expected:* `Vbw` capped, then `PhiT = Phie + Vbw` re-set (order asserted), and a flag raised.
`1.5` is not settable.
*Source:* IP `sand_silt_malay_model.htm` via `B_core_petro.md:244-253, 839, 1060`.

**SB-SAT-T59 — The provenance record is complete and survives export.**
*Input:* one saturation run of each implemented model, including `sw_rtc` and `sw_imts` on
shipped coefficients.
*Operation:* run, then export the deliverable.
*Expected:* the exported record contains, per run — the model identifier, every parameter value,
each value's source string, the model's literature citation, its Worthington 1985 type where a
source states one (`indonesia` type 4; `simandoux_*`, `waxman_smits`, `juhasz`, `dual_water_*`,
`total_shale` type 2; `archie_*` none), and, for the LRLC methods, an explicit **unfitted-
coefficient flag**. Zero fields empty.
*Source:* Geolog module manifests; dossier §1.2, MN-3; `lrlc.rs:83-90`.

**SB-SAT-T60 — Disagreements, guidance and gaps are disclosed.**
*Input:* the §2.12 disagreement rows; a model-selection prompt at `Rw` = 0.15 and `Rw` = 0.25;
a pyritic mineral model.
*Operation:* open the relevant chooser; run.
*Expected:* each disagreement row presents the competing values, their sources and the quantified
consequence. The selection rule is presented **as guidance with its source** and **no model is
switched automatically** — the model chosen at `Rw` = 0.15 and at `Rw` = 0.25 is whatever the
user set, unchanged. The pyritic run carries a recorded statement that mineral conductivity is
not represented.
*Source:* CONTRACT §5.2; memory `reference_shaly_sand_sw_selection`; Techlog Elan linear
conductivity.

**SB-SAT-T61 — The flushed-zone leg, its missing-input behaviour, and its parity.**
*Input:* a run with `Rxo`; the same with `Rxo` absent; the same across both engines.
*Operation:* compute `SXOE`/`SXOT`.
*Expected:* with `Rxo`, the IP limits hold — `Sw^SxoLimit ≥ Sxo ≥ Sw` on WBM, `Sw ≥ Sxo` on OBM.
Without `Rxo`, `SXOT = SXOE = null` and **`VOL_XWAT = VOL_UWAT`**, not 0. Within one run, the
`Sw` and `Sxo` legs use the **same** `B` source — the `sw_ws` two-source defect is asserted
absent. Both engines agree to ±1e-6.
*Source:* Geolog `sw_arch.lls:245-261`; the defect at `sw_ws.lls:252-255` vs `:310`; IP E79-E81.

**SB-SAT-T62 — LRLC calibration replaces the shipped coefficients.**
*Input:* a calibration dataset.
*Operation:* `run_rtc_fit` and `run_s_factor_fit`, then run `sw_rtc` and `sw_imts`.
*Expected:* the fitted coefficients replace 0.45 / 0.0057 / −0.0071 / 2.25 and 0.5; the
unfitted-coefficient flag of SB-SAT-T59 clears; the calibrators' existing regression suite
continues to pass.
*Source:* SandiBumi `lrlc.rs:83-90, :96-99, :205`; existing tests at `lrlc.rs:1534-2126`.

**SB-SAT-T63 — Apparent-`Rw` inverts the same equation it will forward-run.**
*Input:* a known `Rw`, run forward through each model to produce `Rt`.
*Operation:* invert with the matching `rwa_*` route.
*Expected:* the original `Rw` recovered to ±1e-9 relative, per model. A mismatched pairing
(inverting Archie against a Simandoux forward run) does not round-trip and is detected.
*Source:* Geolog's eight-module `rwa_*` family; structurally required by SB-SAT-031.

---

## 7. Open items, escalations and refusals

### 7.1 Escalations — decisions this chapter cannot make

Ten. Each is a conflict between cited sources, or a value with no source, where adjudicating
from the corpus would mean inventing a parameter. None blocks a P0 requirement: every one is
handled by shipping the parameter absent with both readings attached.

**ESC-1 — SPWLA 20th Annual Symposium (1979) Paper AA: contested authorship.**
IP's `E_shf_rocktyping.json:345` and Techlog's reference string attribute what is evidently one
1979 Paper AA to **different author sets** — one naming Hill, Shirley & Klein, the other Juhász.
Same title, same year, same paper letter. Both readings are carried in §2.13 and neither is
chosen. *Closes with:* the paper itself, or Juhász 1979, The Log Analyst pp 3–14 (the reference
Techlog gives). *Consequence if unresolved:* citation text only — no equation or parameter
depends on it. **Priority: low.**

**ESC-2 — Shell variable-`m` coefficient, 0.018 or 0.019 (ledger D-10, OPEN).**
IP's rasters give 0.018 (verified 6× and 4×, both editions); IP's ASCII prose gives 0.019 (nine-
plus occurrences, both editions); the published Shell formula agrees with the ASCII. Techlog Elan
implements the same functional form but ships `MC2 = 0.0`, so **no cross-tool arbitration
exists**. The ledger already records the literature side, so the T4 shelf corroborates rather
than shifts it. *Closes with:* **Jauhar's call against the published Shell source.**
*Consequence:* ~10 % on `Sw` at φe 0.02, negligible above φ ≈ 0.10. SB-SAT-037 makes this a
configuration decision rather than a code decision, so the escalation does not block the build.
**Priority: medium, tight rock only.**

**ESC-3 — Techlog's two modules disagree on the shale-point exponent.**
Quanti "Dual water" uses `φtsh²`; Quanti "Juhasz" uses `φtsh^m*`. At `m* = 1.6` that is **44 % on
`Cwsh` and opposite signs** on the excess-conductivity coefficient. The solvers are compiled, so
the doc may be defective or the `²` may be deliberate. This chapter adopts `φtsh^m*` (SB-SAT-009)
because Geolog and Techlog's own Juhasz page both support it, i.e. two of three sources.
*Closes with:* a live Techlog session running both modules on one dataset at `m* = 1.6`, or the
Techlog 2018.2 release notes. **Priority: medium.**

**ESC-4 — `vQ0`: 0.30 mL/meq or 0.28 cm³/meq.**
Geolog ships 0.3 at 22 °C; Techlog Elan ships 0.28 at room temperature; **both cite Clavier,
Coates & Dumanoir 1984**. This is two readings of one paper, not a vendor invention, which is
precisely why it cannot be adjudicated from the vendor trees. Ships absent (SB-SAT-022).
*Closes with:* the Clavier 1984 SPEJ paper; petro-kb may hold it, and a targeted KB_INDEX lookup
was not run. *Consequence:* ~7 % on `Swb`, worst in the fresh-water regime. **Priority: medium.**

**ESC-5 — Nigerian equation exponent has no paper anywhere.**
Geolog defaults 2; Elan ships `EVCL = 1.4`. Geolog's `sw_nige.info` cites only Poupon & Leveaux
1971 — the *Indonesia* paper, honestly labelled as such — and Elan's Table 27 names no reference
at all. Confirmed at source, not assumed. **Both vendor trees are exhausted**, so this is
unclosable from shipped material. Ships absent. *Closes with:* a named paper from outside both
vendor trees. **Priority: low.**

**ESC-6 — `ρ_brine(salinity, T)` has no source.**
Required by the clay-bound-water `F` unit bridge (SB-SAT-040). No vendor source read states
either tool's brine-density model. Recorded absent with an **explicit prohibition on filling it
from a textbook correlation** — the ÷58.44 bridge silently assumes 1.0 g/cm³, which is safe in
the fresh-water band and ~19 % wrong at 250 Kppm. *Closes with:* a cited brine-density model.
**Priority: medium above ~50 Kppm, immaterial below.**

**ESC-7 — the fourth digit of the `F` coefficient.**
D-07's *bracket* is resolved (SB-SAT-040, SB-SAT-T54), but Techlog prints `0.084` to **two
significant figures**, which cannot certify IP's `0.6425` to four. The two agree to three s.f.
(0.055 %). SB-SAT-T54's tolerance is set to what the sources support, not tighter.
*Closes with:* the primary Juhász 1979 reference. *Consequence:* `max |ΔF| = 3.53e-4` — four
orders of magnitude below the competing bracket reading's error, so it has no practical effect.
**Priority: low; recommend the ledger record it as a separate open note from D-07 itself.**

**ESC-8 — Techlog Quanti solver behaviour is entirely unknown.**
Clamp behaviour, iteration counts, null handling, and whether `SW_AR` is effective or total are
all undetermined; the solvers are compiled and Techlog is not installed on this machine. Every
Techlog claim in this chapter is doc-level T3, never T1. This is why SB-SAT-027 adopts Geolog's
explicit guards rather than Techlog's Levenberg-Marquardt. *Closes with:* a live Techlog session.
**Priority: medium.**

**ESC-9 — ledger sign-off (dossier E-10).**
The dossier recommends, and this chapter's evidence supports: **D-07 → RESOLVED** (reading (i),
via Techlog plus the 58.44 bridge); **B-OPEN-9 → RESOLVED** (`T` is °C); **D-08 → externally
confirmed**; **D-12 → externally confirmed**; **D-15 → closable for this domain** once the
nomenclature scheme is pinned by SB-SAT-026. These are recommendations to the ledger owner.
`ip2025_chm_ingest\DISCREPANCIES.md` was **not modified by this chapter**.
*Closes with:* Jauhar's sign-off. **Priority: high, and cheap.**

**ESC-10 — Omovie Sonic Saturation patent claims and independent literature.**
The dossier and its critique contain no capability-level evidence for Omovie beyond confirming that
no Tier-C material entered the saturation research. `CONTRACT.md` §2.2 classifies Omovie Sonic
Saturation (US 12,242,011 B2) as patent-claimed. **Draft classification opinion: C-1**, pending a
claims read. *Closes with:* the granted claims themselves plus published sonic-saturation literature
sufficient to establish the user need, a lawful design-around route, and a cited incumbent
limitation for the mandatory `Betters:` line. Until then no method, default, requirement or test is
specified. **Priority: blocked by acquisition, not by implementation.**

### 7.2 Open items — known, scoped, not yet requirements

Eleven. Each is something this chapter deliberately did **not** turn into a requirement, because
the evidence to specify it is not held. They are listed so their absence is a recorded decision.

1. **Nigeria's equation form.** SB-SAT-003 reserves the identifier and §5 ships the exponent
   absent, but no equation is specified — the corpus holds a menu entry and two disagreeing
   defaults, not a form. A requirement written from that would be invention.
2. **PNC / Sigma saturation.** Geolog ships `sw_pnl`; only its `PHIE < 0.005` guard was read.
   The method belongs in this domain but nothing about its equation is held.
3. **Geolog flushed-zone legs beyond seven modules.** `sw_pnl`'s `Sxo` path and the remaining
   `Sxo` legs were not read; SB-SAT-046 specifies the pattern, not those modules' specifics.
4. **Techlog Elan Table 25 (dual water), the `b(T)` algorithm images, and the `Qv`-dependent
   `mdw` relation.** Roughly five images under `Doc\image\`. Would give a fourth independent
   dual-water parameterisation and Elan's `mdw` provenance beyond the prose "1.8".
   **Cheap and high value** — the best-value open item in the list.
5. **IP's compiled `B`-chart implementation.** Bounded, not unknown: Geolog's own doc states
   `GRAVEST ≡ WAX_THOM` at 25 °C, and evaluating both gives 0.039535 vs 0.038321 mho·cm²/meq —
   **3.2 % on `B`, ≈1.5 % on `Sw` at `n = 2`**. Geolog's published quartic is a sufficient
   surrogate carrying that documented uncertainty.
6. **Geolog's compiled `mm_wt74`.** Called at `sw_ws.lls:310`, `ffcec.lls:175`, `ricec.lls:179`.
   Not readable from the install tree. Its divergence from the published quartic is the same
   3.2 % bound as item 5.
7. **The laminated `Sw` model itself.** SB-SAT-041 requires an interlock against it; the model
   is owned by `TBD` and is not specified here.
8. **`Cm*` provenance.** IP ships 1.0 with a source string, but the coefficient's derivation is
   not held, so SB-SAT-036 exposes the `Cm` relations without adjudicating `Cm*` itself.
9. **`ρ_shale` / `CEC_dry` unit reconciliation across SandiBumi's two engines.** SB-SAT-013
   requires the types; the migration of `lrlc.rs`'s meq/100 g and `multimin2.rs`'s meq/g to one
   canonical form is not scoped here.
10. **Elan's linear per-mineral conductivity endpoints.** SB-SAT-051 records the capability gap;
    no endpoint values are held and none would be transcribed from a chart if they were.
11. **`SWE_UNCL`/`SWT_UNCL` mnemonic spelling.** SB-SAT-025 requires the curves; the exact
    mnemonic pattern (`SWE_UNCL` vs `SWE_<METHOD>`) is a nomenclature decision that belongs with
    D-15's closure under SB-SAT-026.

### 7.3 Refusals — what was deliberately not done

Two kinds, listed separately per `CONTRACT.md` §2.2.1.

#### 7.3.1 Transcription refusals — rule compliance

2. **No vendor chart lookup-table data was transcribed.** The Techlog 1978 Waxman `B` chart, the
   1972 original and revised fits, and the Western Atlas Charts 1994 p. 27 `Rw` chart that Geolog
   cites are each referenced by existence, attribution and purpose only. Geolog's `WAX_THOM`
   quartic **is** reproduced, and that is inside the boundary: it is executable Loglan source
   that *replaced* a chart lookup, not the lookup table. No Schlumberger, Halliburton, Baker,
   Weatherford, Sperry-Sun, PathFinder, Anadrill or GE chart data appears anywhere in this
   chapter. No `.neu`/`.ovl`/`.itt`/`.itp`/`.att`/`.bor`/`.eli` content was read.
3. **No petrophysical parameter was inferred, rounded, or carried over from a neighbouring
   vendor.** Twenty parameter rows ship absent for exactly this reason. Where SandiBumi currently
   ships an uncited number — `Rsh` 4.0, `φ_sh` 0.10, `Rw` 0.1 and 0.3, the α ceiling 5.0, the
   LRLC coefficients — it is recorded in §3 and §5 as a defect and withdrawn or re-declared, not
   adopted as a default and not laundered into a citation.
4. **No individual client well name, field, block or operator name was introduced.** No
   interval-level result, SCAL table or log datum from any client dataset appears. Where a
   parameter traces to a delivered study, it is attributed to the project record, not to a well.
   *Note for the release gate:* the **dossier** deliberately retains project-kb record filenames,
   which encode project/block/operator, and is therefore not clean for external distribution as
   written. **This chapter does not reproduce those filenames**, so no redaction step is required
   on this file.
5. **`saturation_critique.md` was not read**, per the assignment. The dossier's
   `## Critique disposition` section is authoritative over any body text it corrects, and no
   conflict was found between the two that the disposition had not already resolved in place.
6. **Nothing under `D:\XX. SandiBumi` was modified except this file.** The repository, the vendor
   install trees (`C:\Program Files\IP2018`, `C:\Program Files\IP2025`, the Techlog and Geolog
   trees) and `ip2025_chm_ingest\DISCREPANCIES.md` were all read-only for this task.
   `D:\XX. Arshilla` was not touched.
7. **No requirement was written to fill the chapter's shape.** Where the evidence supported an
   observation but not an obligation, it went to §7.2 as an open item rather than becoming a
   weak requirement — see items 1, 2 and 7 in particular. The front matter's counts were amended
   to match the content twice during authoring (P0 12→13, tests 41→63, parameters 46→67) rather
   than the content being trimmed or padded to match a promised shape.

#### 7.3.2 Defect refusals — vendor behaviour SandiBumi declines to reproduce

No defect refusal is asserted from §7's evidence. The unresolved vendor disagreements remain named
in §7.1 and ship with their disputed parameters absent; they are not silently converted into claims
that one vendor is wrong.

### 7.4 Independent-derivation requirements

**Omovie Sonic Saturation — no owning requirement yet.**

- **Class:** **Draft classification opinion: C-1 (patent-claimed)**, from `CONTRACT.md` §2.2's
  register. The classification is not treated as final until the granted claims are read.
- **User need:** **not established by the held dossier or critique.** They record no Omovie
  capability detail beyond the boundary declaration, so this chapter does not invent a workflow or
  claim that a specific saturation problem requires it.
- **Primary sources and derivation route:** **named acquisition gap — ESC-10.** Read the granted
  claims, then acquire published sonic-saturation literature from which an independent method could
  be derived without vendor internals or input/output inference. Jauhar's C-1 decision remains:
  read the claims, license, or drop.
- **`Betters:`** **not written.** No cited incumbent limitation is held, and an unsupported line
  would be a clone claim rather than a design-around.
- **Owning requirement and tests:** **not minted.** A new `SB-SAT` requirement, cited defaults and
  acceptance tests are blocked until ESC-10 closes. This is an acquisition gap, not a refusal of
  the capability.

---

## 8. Traceability

**Row count: 211.** Enumerated from the dossier's own structure, not from recall — the section
inventories below were re-read at authoring time to get item labels verbatim:

| Dossier section | Items | Basis for the count |
|---|---|---|
| §1.4 coverage gaps | 6 | bullet list |
| §2.1–§2.10 | 10 | subsection headings |
| §3.1–§3.11 | 11 | subsection headings |
| §4.1 model selection rule | 1 | subsection |
| §4.2 per-item choices | 28 | table rows |
| §4.3 ledger dispositions | 7 | table rows |
| §5.1 canonical forms | 16 | 11 polynomial model rows + 5 named blocks |
| §5.2 parameter table | 48 | 39 parameter rows + 9 `Rw`-correlation rows |
| §5.3 tests to ship | 35 | 25 numbered + 10 sub-items (4b, 4c, 10b, 18a–18g) |
| §6 gaps & escalations | 10 | E-1 … E-10 |
| `## Critique disposition` | 39 | 6 blockers + 14 majors + 11 minors + 8 added findings |
| **Total** | **211** | |

**Note on the count.** An earlier plan for this chapter recorded §5.3 as 34 items; the re-read
gives **35** (25 numbered plus 10 sub-items — the earlier figure dropped one). The table above is
the count that governs. Every one of the 211 rows below is dispositioned; there are no omissions
and no "covered elsewhere" placeholders.

Disposition vocabulary: **ADOPTED** (became a requirement, parameter row or test),
**DEFERRED** (recorded as a §7.2 open item), **REJECTED** (deliberately not carried, with the
reason), **EVIDENCE-ONLY** (informs a rationale but generates no obligation),
**ESCALATED** (§7.1).

### 8.1 Dossier §1.4 — coverage gaps

| Dossier item | Disposition | Where |
|---|---|---|
| Techlog Archie/Simandoux/Indonesia/W-S source is compiled; all Techlog claims are T3 | EVIDENCE-ONLY | Tier labelling throughout; §7.1 ESC-8 |
| Techlog Elan Table 25, `b(T)` and `mdw` images not read | DEFERRED | §7.2 item 4 |
| IP's compiled B-chart / Waxman-Thomas 1978 implementation | DEFERRED | §7.2 item 5; bounded at 3.2 % |
| Geolog's compiled `mm_wt74` | DEFERRED | §7.2 item 6 |
| Geolog `sw_pnl` body and remaining Sxo/flushed-zone paths | DEFERRED | §7.2 items 2, 3 |
| Nigerian equation provenance — unclosable from either vendor tree | ESCALATED | §7.1 ESC-5; SB-SAT-003, §5 Nigeria row |

### 8.2 Dossier §2 — definitions, equations and assumptions

| Dossier item | Disposition | Where |
|---|---|---|
| §2.1 Symbol and unit conventions — three tools disagree | ADOPTED | SB-SAT-012, SB-SAT-013, SB-SAT-014; §2.10 |
| §2.2 Archie — the porosity-system trap | ADOPTED | SB-SAT-002; SB-SAT-T03, T04 |
| §2.3 Simandoux — names inverted between vendors | ADOPTED | SB-SAT-001, SB-SAT-003, SB-SAT-004 |
| §2.4 Indonesia / Poupon-Leveaux — three-way structural agreement | ADOPTED | SB-SAT-006, SB-SAT-007 |
| §2.5 Waxman-Smits — three `B` unit systems | ADOPTED | SB-SAT-011, SB-SAT-012, SB-SAT-015, SB-SAT-036 |
| §2.6 Juhász — IP implements a different equation | ADOPTED | SB-SAT-009, SB-SAT-023 |
| §2.7 Dual Water — three models under one name | ADOPTED | SB-SAT-016 … SB-SAT-022 |
| §2.8 Clay-bound-water fraction (Hill, Shirley & Klein) — D-07 | ADOPTED | SB-SAT-040; ESC-1, ESC-6, ESC-7 |
| §2.9 Solvers, clamps and validity limits | ADOPTED | SB-SAT-027 … SB-SAT-030 |
| §2.10 Parameter defaults with per-value sources | ADOPTED | §5 in full; SB-SAT-038 |

### 8.3 Dossier §3 — differences that matter

| Dossier item | Disposition | Where |
|---|---|---|
| §3.1 "Archie" differs by 25 saturation units | ADOPTED | SB-SAT-002; SB-SAT-T03 |
| §3.2 Simandoux naming inversion — 7.3 su, plus 4.6 from `a` | ADOPTED | SB-SAT-001, SB-SAT-005; SB-SAT-T06, T08 |
| §3.3 Waxman-Smits `B` ×100 — 27 su, the worst silent failure | ADOPTED | SB-SAT-012; SB-SAT-T17, T18 |
| §3.4 `B(T,Rw)` °C vs °F — up to ~20 % on Sw | ADOPTED | SB-SAT-014; SB-SAT-T21 |
| §3.5 Juhász `Bn = 1.0` — 14 su and a sign flip | ADOPTED | SB-SAT-009, SB-SAT-010; SB-SAT-T14, T15 |
| §3.6 Techlog's own two modules disagree — `φtsh²` vs `φtsh^m*` | ADOPTED + ESCALATED | SB-SAT-009; ESC-3 |
| §3.7 Nigeria — Geolog's default is not Nigerian | ESCALATED | ESC-5; §5 Nigeria row |
| §3.8 `Rw` factory defaults differ 3.3× | ADOPTED | SB-SAT-031; SB-SAT-T45 |
| §3.9 `B` method defaults diverge | ADOPTED | SB-SAT-015; SB-SAT-T23 |
| §3.10 Shell variable `m` — both readings retained | ADOPTED + ESCALATED | SB-SAT-037; ESC-2 |
| §3.11 D-07 bracket — the two readings, settled | ADOPTED | SB-SAT-040; SB-SAT-T54 |

### 8.4 Dossier §4.1 — model selection

| Dossier item | Disposition | Where |
|---|---|---|
| §4.1 Model selection rule (`Rw` ≤ 0.20 → Simandoux; > 0.20 → Indonesia) | ADOPTED as guidance, REJECTED as an automatic switch | SB-SAT-045; SB-SAT-T60 |

### 8.5 Dossier §4.2 — the twenty-eight per-item choices

| Dossier item | Disposition | Where |
|---|---|---|
| Archie porosity system — two named methods, never a bare `archie` | ADOPTED | SB-SAT-002 |
| Simandoux naming — name by equation, alias table | ADOPTED | SB-SAT-001, SB-SAT-003 |
| Simandoux `Vsh` exponent — `C` on the SLB variant only | ADOPTED | SB-SAT-004 |
| Simandoux `a` — no default | ADOPTED | SB-SAT-005; §5 row 2 |
| Total Shale — alias/preset with an `n = 2` closed-form fast path | ADOPTED | SB-SAT-008; SB-SAT-T11, T12 |
| Indonesia — canonical form with `k` as a parameter | ADOPTED | SB-SAT-006 |
| Woodhouse Tar — `k = 2` preset, keep the alias, cite Paper T | ADOPTED | SB-SAT-007 |
| Nigeria — free half-exponent, no default | ADOPTED (no default) + ESCALATED (form) | §5 Nigeria row; ESC-5; §7.2 item 1 |
| Juhász — shale-derived coefficient, `Bn` as override, negative-coefficient guard | ADOPTED | SB-SAT-009, SB-SAT-010 |
| Juhász clay-vs-shale — normalize on shale, record the convention | ADOPTED | SB-SAT-009 |
| Juhász effective back-out — `Swb = Qvn`, not `1 − φe/φt` | ADOPTED | SB-SAT-023; SB-SAT-T35 |
| `m*`/`n*` provenance — two named routes, core preferred | ADOPTED | SB-SAT-036; SB-SAT-T51 |
| Shell/Elan `m + c/φ` — one parameterised route, `c` cited, no default | ADOPTED | SB-SAT-037; ESC-2 |
| Waxman-Smits equation — `a` exposed | ADOPTED | SB-SAT-011 |
| Waxman-Smits `B` unit — canonical `L·S/(eq·m)`, named ×100 converter | ADOPTED | SB-SAT-012 |
| `Qv` unit — canonical meq/mL, reject meq/L | ADOPTED | SB-SAT-013 |
| `B(T,Rw)` — °C, clamp ≥ 0, measured override | ADOPTED | SB-SAT-014 |
| `B` method default — none; four named options | ADOPTED | SB-SAT-015 |
| Dual Water — full Clavier chain + IP parameter form; `g2 = Swb·(Cwb − Cw)`; `Qv` bound as a flag | ADOPTED | SB-SAT-016, SB-SAT-017, SB-SAT-021 |
| Dual-water `SWE` back-out — match Geolog's porosity split; emit CEC `Swb` as QC | ADOPTED | SB-SAT-023 |
| `vQ` room-temperature constant — both readings retained | ESCALATED | SB-SAT-022; ESC-4 |
| Effective↔total conversion — per model, not blanket | ADOPTED | SB-SAT-023; SB-SAT-T34, T35, T36 |
| Solver — Newton with Geolog's guards, MISSING on non-convergence | ADOPTED | SB-SAT-027, SB-SAT-028 |
| Clamping — clipped + unclipped, `SWE_IRR` transformed per model, `Swb = 1` degeneracy | ADOPTED | SB-SAT-024, SB-SAT-025 |
| SSM bound-water cap — 1.5, then the `PhiT` re-set, hard-coded | ADOPTED | SB-SAT-042; SB-SAT-T58 |
| Poupon-Aguilera / Poupon-Tixier — citations + laminated interlock | ADOPTED | SB-SAT-041; SB-SAT-T56, T57 |
| Nomenclature — never a bare `SW`, plus a method-flag curve | ADOPTED | SB-SAT-026 |
| Guard rails — the six documented vendor behaviours | ADOPTED | SB-SAT-029, SB-SAT-030 |

### 8.6 Dossier §4.3 — discrepancy-ledger dispositions

| Dossier item | Disposition | Where |
|---|---|---|
| **D-07** — clay-bound-water `F` unbalanced brackets | ADOPTED (reading (i)); ledger closure recommended | SB-SAT-040; ESC-7, ESC-9 |
| **D-08** — W-S `B(T,Rw)` bracket defect, confirmed externally | ADOPTED | SB-SAT-014; ESC-9 |
| **D-10** — Shell `m` 0.018 vs 0.019, remains OPEN | ESCALATED | SB-SAT-037; ESC-2 |
| **D-12** — Juhász/W-S prose drops `×Rw`, confirmed externally | ADOPTED | SB-SAT-009, SB-SAT-011; ESC-9 |
| **D-15** — `SW`/`SWE` nomenclature conflict, design mandate | ADOPTED; closable for this domain | SB-SAT-026; ESC-9 |
| **B-OPEN-9** — `B` temperature units, RESOLVED °C | ADOPTED | SB-SAT-014; ESC-9 |
| **D-09** — excavation-effect exponent, out of domain | REJECTED as out of scope — recorded, not silently dropped | §1 seam with `POR` |

### 8.7 Dossier §5.1 — canonical equation forms

| Dossier item | Disposition | Where |
|---|---|---|
| `archie_effective` polynomial row | ADOPTED | SB-SAT-002; SB-SAT-T04 |
| `archie_total` polynomial row | ADOPTED | SB-SAT-002; SB-SAT-T04 |
| `simandoux_bardon_pied` row (`Vsh/Rsh`, no exponent) | ADOPTED | SB-SAT-004; SB-SAT-T06, T07 |
| `simandoux_modified_slb` row (`(1−Vsh)` divisor, `Vsh^C`) | ADOPTED | SB-SAT-004 |
| `total_shale` row (alias/preset, `n ≡ 2`) | ADOPTED | SB-SAT-008 |
| `poupon_aguilera` row | ADOPTED | SB-SAT-041; SB-SAT-T56 |
| `poupon_tixier` row | ADOPTED | SB-SAT-041; SB-SAT-T56 |
| `waxman_smits` row | ADOPTED | SB-SAT-011 |
| `juhasz` row | ADOPTED | SB-SAT-009; SB-SAT-T13 |
| `dual_water_simple` row | ADOPTED | SB-SAT-016 |
| `dual_water_cec` row | ADOPTED | SB-SAT-016, SB-SAT-017 |
| Indonesia family — closed form, not polynomial | ADOPTED | SB-SAT-006; SB-SAT-T09 |
| Dual-water CEC chain (α, γ, `vQ`, β, `Swb`, `Cwb`) | ADOPTED | SB-SAT-018, SB-SAT-019, SB-SAT-020, SB-SAT-022 |
| Effective back-out — per-model table (4 rows) | ADOPTED | SB-SAT-023; SB-SAT-T35 |
| Clay-bound-water fraction block | ADOPTED (opt-in only) | SB-SAT-040; SB-SAT-T55 |
| Waxman-Smits `B` block | ADOPTED | SB-SAT-012, SB-SAT-014, SB-SAT-015 |

### 8.8 Dossier §5.2 — parameter table (39 parameter rows)

| Dossier item | Disposition | Where |
|---|---|---|
| `a` — no default | ADOPTED | §5 row 1; SB-SAT-034 |
| `m` — no default | ADOPTED | §5 row 3; SB-SAT-034 |
| `n` — no default | ADOPTED | §5 row 4; SB-SAT-034 |
| `m*`, `n*` (W-S/DW) — no default | ADOPTED | §5 rows 6–7; SB-SAT-034 |
| `Rw` — no default | ADOPTED | §5; SB-SAT-031 |
| `Rsh` / `Rcl` — no default | ADOPTED | §5; SB-SAT-035 |
| `φt_sh` — no default, Geolog-internal validation conflict | ADOPTED (both ranges recorded) | §5; SB-SAT-035; SB-SAT-T50 |
| Simandoux `C` — default 1, range 1:2 | ADOPTED | §5; SB-SAT-004 |
| Indonesia `k` — default 1 (FULL) | ADOPTED | §5; SB-SAT-006 |
| Nigeria half-exponent — conflict, no default | ESCALATED | §5; ESC-5 |
| `B` (W-S) — no default | ADOPTED | §5; SB-SAT-012 |
| `B` method — no default | ADOPTED | §5; SB-SAT-015 |
| `Bn` (Juhász override) — no default | ADOPTED (override only) | §5; SB-SAT-009 |
| `Qv` — canonical meq/mL | ADOPTED | §5; SB-SAT-013 |
| `CEC_dry` — meq/g, no default | ADOPTED (unit conflict recorded) | §5; SB-SAT-013; §7.2 item 9 |
| `ρ_shale` — g/cc | ADOPTED | §5 via the `qv` chain; SB-SAT-013 |
| `vQ0` — OPEN, 0.3 vs 0.28 | ESCALATED | §5; SB-SAT-022; ESC-4 |
| `β_const` — default 1, range 0:1 | ADOPTED | §5; SB-SAT-020 |
| `Cm*` — 1.0 | ADOPTED, provenance DEFERRED | §5 `m*`-from-`Qv` row; §7.2 item 8 |
| Shell `m` coefficient — OPEN (D-10) | ESCALATED | §5; SB-SAT-037; ESC-2 |
| CBW `F` coefficients, Kppm form (0.6425, 0.22) | ADOPTED | §5; SB-SAT-040 |
| CBW `F` coefficients, meq/cm³ form (0.084, 0.22) | ADOPTED | §5; SB-SAT-040 |
| Salinity ↔ `C₀` bridge (÷58.44) | ADOPTED, with `ρ_brine` ESCALATED | §5; SB-SAT-040; ESC-6 |
| `SWT_IRR` — default 0 | ADOPTED | §5 via SB-SAT-024; `modules.rs:1994` |
| `T` — °C | ADOPTED | §5; SB-SAT-014 |
| Salinity — ppm, validation 0:400000 | ADOPTED | §5; SB-SAT-032 |
| Solver max iterations — 20 | ADOPTED | §5; SB-SAT-027 |
| Solver tolerance — 1e-5 | ADOPTED | §5; SB-SAT-027 |
| Solver seed — 0.5 | ADOPTED | §5; SB-SAT-027 |
| `φe` floor — 0.005 | ADOPTED | §5; SB-SAT-029 |
| `total_shale` `n` — 2, not settable | ADOPTED | §5; SB-SAT-008 |
| `total_shale` `a` — Geolog 1, range 0.1:10 | REJECTED as a default — `a` ships absent per SB-SAT-034; range retained | §5 row 1; SB-SAT-034 |
| `total_shale` `m` — Geolog 2, range 1:10 | REJECTED as a default — same reason; range retained | §5 row 3; SB-SAT-034 |
| `MUDBASE` — enum, default WATER | ADOPTED | §5; SB-SAT-039 |
| Elan `MC2` — 0.0 | ADOPTED as a non-arbitrating third source | §5 Shell row; SB-SAT-037; SB-SAT-T52 |
| Elan `EVCL` / `MVCL` — 1.0 / 0.5 | EVIDENCE-ONLY — supports ESC-5, generates no default | ESC-5 |
| Elan `CUDC_UNC` — 0.065 mho/m | EVIDENCE-ONLY — per-mineral conductivity is a recorded gap | SB-SAT-051 |
| `ρ_brine(salinity, T)` — OPEN | ESCALATED | §5; ESC-6 |
| `Qv` cap in dual water — `1/(α·vQh)` | ADOPTED as a flag, not a clamp | §5; SB-SAT-021; SB-SAT-T32 |

### 8.9 Dossier §5.2 — `Rw` correlation block (9 rows)

| Dossier item | Disposition | Where |
|---|---|---|
| Arps constant, °C — 21.5 | ADOPTED | §5; SB-SAT-032; SB-SAT-T48 |
| Arps constant, °F — 6.77 | ADOPTED | §5; SB-SAT-032; SB-SAT-T48 |
| Bateman-Konen reference temperature — 23.9 °C | ADOPTED | §5; SB-SAT-032 |
| Kennedy reference temperature — 75 °F | ADOPTED | §5; SB-SAT-032 |
| Kennedy centring constant — 29.46518957 | ADOPTED | §5 Kennedy polynomial row |
| Kennedy polynomial coefficients — 24.30853, −0.0364, −0.02922 | ADOPTED | §5; SB-SAT-032 |
| Bateman ↔ Kennedy switch at 39161 ppm | ADOPTED | §5; SB-SAT-032; SB-SAT-T46 |
| Kennedy salinity cap — 275 000 ppm | ADOPTED | §5; SB-SAT-032 |
| Kennedy `rw75` floor — 0.0412, doc says 0.412 | ADOPTED with the ×10 doc-defect comment mandated | §5; SB-SAT-033; SB-SAT-T47 |

### 8.10 Dossier §5.3 — tests to ship (35 items)

| Dossier item | Disposition | Where |
|---|---|---|
| 1. Indonesia via Geolog `f1+f2+f3` ≡ Techlog `(A+B)^(−2/n)` | ADOPTED | SB-SAT-T09 |
| 2. `indonesia(k=2)` ≡ `woodhouse_tar` | ADOPTED | SB-SAT-T10 |
| 3. `swe_from_swt` ≡ IP E78 ≡ Geolog's form | ADOPTED | SB-SAT-T34 |
| 4. `simandoux_modified_slb(C=1)` reproduces Techlog + IP E64 | ADOPTED | SB-SAT-T07 |
| 4b. `total_shale` closed form ≡ generic solver, 1e-9 | ADOPTED | SB-SAT-T12 |
| 4c. `total_shale` `n` not settable — compile-time error | ADOPTED | SB-SAT-T11 |
| 5. `juhasz` reproduces Geolog + Techlog | ADOPTED | SB-SAT-T13 |
| 6. `B` anchors 3.895 / 15.51 and the `mho·cm²/meq` conversion | ADOPTED | SB-SAT-T21, T18 |
| 7. The ×100 test — type error at compile time | ADOPTED | SB-SAT-T17 |
| 8. `B_juhasz` fed a °F number must be rejected | ADOPTED | SB-SAT-T21 |
| 9. `Qv` in meq/L is a type error; three-unit round-trip is identity | ADOPTED | SB-SAT-T19, T20 |
| 10. `B_juhasz(T < 6 °C)` clamps to 0 | ADOPTED | SB-SAT-T22 |
| 10b. D-07 cross-unit consistency at 1e-3 absolute + 3 s.f. | ADOPTED | SB-SAT-T54 |
| 11. `juhasz` negative coefficient raises a flag | ADOPTED | SB-SAT-T15 |
| 12. `φe = φt = 0` ⇒ Sw 1, volumes 0 | ADOPTED | SB-SAT-T42 |
| 13. `φe < 0.005` ⇒ Sw 1 | ADOPTED | SB-SAT-T42 |
| 14. `Rt` missing ⇒ null, not 1 and not 0 | ADOPTED | SB-SAT-T43 |
| 15. Non-convergence ⇒ null, never a partial `sat` | ADOPTED | SB-SAT-T41 |
| 16. `Vsh → 1.0` in Indonesia flags before the 0/0 | ADOPTED | SB-SAT-T44 |
| 17. CEC `Swb` never exceeds `1 − φe/φt` | ADOPTED | SB-SAT-T32 |
| 18. Clipped and unclipped emitted for every method | ADOPTED | SB-SAT-T38 |
| 18a. `SWE_IRR` is effective, not rescaled; don't port `sw_ws.lls:302` | ADOPTED | SB-SAT-T37 |
| 18b. Per-model `Swb` back-out, one test per model | ADOPTED | SB-SAT-T35 |
| 18c. `g2 = Swb·(Cwb − Cw)`, α-dependent | ADOPTED | SB-SAT-T26 |
| 18d. SSM 1.5 cap fires and is flagged | ADOPTED | SB-SAT-T58 |
| 18e. `Qv ≤ 1/(α·vQh)` is a flag, not a clamp | ADOPTED | SB-SAT-T32 |
| 18f. `φe < 0.005` ⇒ `VOL_UWAT = VOL_XWAT = φe`, not 0 | ADOPTED | SB-SAT-T42 |
| 18g. `MUDBASE` is model-scoped | ADOPTED | SB-SAT-T53 |
| 19. No bare `SW`/`SXO`; every curve carries `E` or `T` | ADOPTED | SB-SAT-T39 |
| 20. Every default has a source string or is `NoDefault` | ADOPTED | SB-SAT-T31 |
| 21. Every §3 numeric case is an executed fixture | ADOPTED — distributed across the tests that carry each figure | SB-SAT-T03, T06, T08, T14, T17, T21, T52 |
| 22. Branch continuity at 39161 ppm, 0.07 % | ADOPTED | SB-SAT-T46 |
| 23. Each branch carries its own Arps conversion | ADOPTED | SB-SAT-T48 |
| 24. Kennedy cap 0.0412, with the anti-"fix" comment | ADOPTED | SB-SAT-T47 |
| 25. `MEASURED` uses `RWT`, not 75 °F | ADOPTED | SB-SAT-T45 |

### 8.11 Dossier §6 — gaps and escalations

| Dossier item | Disposition | Where |
|---|---|---|
| **E-1** D-07 closed; Paper AA authorship collision | ESCALATED | ESC-1; ESC-7 |
| **E-2** D-10 Shell `m` — Jauhar's call | ESCALATED | ESC-2 |
| **E-3** Techlog `φtsh²` vs `φtsh^m*` | ESCALATED | ESC-3 |
| **E-4** `vQ0` 0.3 vs 0.28, both citing Clavier 1984 | ESCALATED | ESC-4 |
| **E-5** Nigerian exponent — vendor trees exhausted | ESCALATED | ESC-5 |
| **E-6** Techlog Elan Table 25, `b(T)`, `mdw` — cheap, high value | DEFERRED | §7.2 item 4 |
| **E-7** `sw_pnl` body and remaining Sxo legs | DEFERRED | §7.2 items 2, 3 |
| **E-8** Techlog compiled solver behaviour unknown | ESCALATED | ESC-8 |
| **E-9** IP `B` chart implementations, bounded at 3.2 % | DEFERRED | §7.2 items 5, 6 |
| **E-10** Ledger update recommended | ESCALATED | ESC-9 |

### 8.12 Dossier `## Critique disposition` — blockers

| Dossier item | Disposition | Where |
|---|---|---|
| **BL-1** `mod_sat` is not a saturation module (Gassmann); nine `sw_*` modules, not ten | ADOPTED — the corrected inventory is what §2 and §3 are built on; `mod_sat` generates no requirement | §2.11; §5 (no `mod_sat` row) |
| **BL-2** `sw_tot` is a distinct method, and is algebraically `sw_sim` SCHLUM at `C=1, n=2` | ADOPTED | SB-SAT-008; SB-SAT-T11, T12 |
| **BL-3** α does not cancel from `g2`; CEC `Swb` never reaches the back-out | ADOPTED | SB-SAT-017, SB-SAT-023; SB-SAT-T26 |
| **BL-4** Techlog *does* implement the Shell form, but ships `MC2 = 0.0` | ADOPTED | SB-SAT-037; §5 Shell row; ESC-2 |
| **BL-5** D-07's precision claim was false; test 10b was unsatisfiable | ADOPTED — the corrected 3-s.f. framing and the 1e-3 tolerance are what this chapter ships | SB-SAT-T54; ESC-7 |
| **BL-6** The blanket effective back-out is wrong for Juhász | ADOPTED | SB-SAT-023; SB-SAT-T35 |

### 8.13 Dossier `## Critique disposition` — majors

| Dossier item | Disposition | Where |
|---|---|---|
| **MJ-1** `sw_ws` never got the 2008 `swe_irr` fix | ADOPTED — the transform is required and Geolog's form is explicitly forbidden | SB-SAT-024; SB-SAT-T37 |
| **MJ-2** `WAX_THOM` quartic is not the canonical path; three of four sites call `mm_wt74` | ADOPTED — drives the same-source clause | SB-SAT-046; SB-SAT-T61; §7.2 item 6 |
| **MJ-3** Four in-domain Geolog modules missing (`ffcec`, `ricec`, `qv`, `rwa_*`) | ADOPTED | SB-SAT-036, SB-SAT-050; SB-SAT-T51, T63 |
| **MJ-4** Geolog's shipped citations under-reported | ADOPTED | SB-SAT-043, SB-SAT-049; SB-SAT-T35, T59 |
| **MJ-5** SPWLA 20th Paper AA authorship collision not surfaced | ESCALATED | ESC-1 |
| **MJ-6** Kppm ↔ g/L bridge assumes `ρ_brine = 1.0` | ADOPTED + ESCALATED | SB-SAT-040; ESC-6; SB-SAT-T55 |
| **MJ-7** `Swb = 1 − F` is the dossier's own transplant | ADOPTED — gated behind opt-in, both vendor uses carried | SB-SAT-040; SB-SAT-T55 |
| **MJ-8** The direct `B`-unit evidence was never opened; three-image mapping resolved | EVIDENCE-ONLY — strengthens SB-SAT-012's rationale; no chart values transcribed | SB-SAT-012; §7.3 item 2 |
| **MJ-9** §5.2's `Rw` block was not implementable | ADOPTED — all four branches with their bound conversions | SB-SAT-032, SB-SAT-033; SB-SAT-T45, T47, T48 |
| **MJ-10** D-10's "new evidence" was already in the ledger | ADOPTED — recorded as corroboration, not as a tally shift | ESC-2 |
| **MJ-11** IP's citations and the `Vcl`-vs-`Vshale` note were dropped; laminated interlock added | ADOPTED | SB-SAT-009, SB-SAT-041; SB-SAT-T57 |
| **MJ-12** IP's SSM 1.5 bound-water cap missing | ADOPTED | SB-SAT-042; SB-SAT-T58 |
| **MJ-13** The compliance statement was false; client names did appear | ADOPTED — this chapter introduces none and reproduces no project-kb filename | §7.3 item 4 |
| **MJ-14** Geolog's documented-but-disabled `Qv` clamp | ADOPTED — as a flag, not a clamp | SB-SAT-021; SB-SAT-T32 |

### 8.14 Dossier `## Critique disposition` — minors

| Dossier item | Disposition | Where |
|---|---|---|
| **MN-1** §3.9 quartic arithmetic — exponent −1.416565, `bmax` 0.038321, spread ~20 % | ADOPTED | §5 `WAX_THOM` row; SB-SAT-T23 |
| **MN-2** `sw_sim` option-name vs branch-name mismatch | ADOPTED | SB-SAT-003; SB-SAT-T01 |
| **MN-3** Worthington classification under-reported | ADOPTED | SB-SAT-049; SB-SAT-T59 |
| **MN-4** `φe < 0.005` sets volumes to `φe`, not 0 | ADOPTED | SB-SAT-029; SB-SAT-T42 |
| **MN-5** D-09 register/body mismatch, withdrawn as out of domain | ADOPTED — recorded as an out-of-scope seam | §1 seam with `POR`; §8.6 |
| **MN-6** `MUDBASE` omitted; `sw_arch` does *not* carry it | ADOPTED — model-scoped, with `archie_*` a compile-time error | SB-SAT-039; SB-SAT-T53 |
| **MN-7** `PHIT_SH` validation inconsistent across Geolog modules | ADOPTED — 0:0.4 accept, 0.4:1 warn, both readings recorded | SB-SAT-035; §5; SB-SAT-T50 |
| **MN-8** §3.8's `Rw ≈ 0.21` corroboration withdrawn | ADOPTED — no `Rw` default rests on it | SB-SAT-031 rationale |
| **MN-9** Uncited `ρfl = 1.0 g/cc` replaced by Geolog's `qv` chain; `sw_dual.lls:125-127` `Qv` doc defect | ADOPTED | §5 `ρ_shale` / `CEC_dry` rows; SB-SAT-013 |
| **MN-10** "IP alone offers the `Qv`-driven `m*`" is wrong | ADOPTED — two named routes with a preference rule | SB-SAT-036 |
| **MN-11** IP menu string "Juhasz (Waxman-Smits)"; the inverse conversions | ADOPTED | SB-SAT-003, SB-SAT-023; SB-SAT-T36 |

### 8.15 Dossier `## Critique disposition` — findings this revision added

| Dossier item | Disposition | Where |
|---|---|---|
| 1. `C` is `SCHLUM`-branch only | ADOPTED | SB-SAT-004; SB-SAT-T07 |
| 2. `sw_dual.lls:125-127` `Qv` doc block is wrong | EVIDENCE-ONLY — a vendor defect not to inherit; SandiBumi's `Qv` construction is specified independently | SB-SAT-013 |
| 3. `ricec.info` `CBW`-for-`CWB` typo | EVIDENCE-ONLY | SB-SAT-036 rationale |
| 4. `GRAVEST ≡ WAX_THOM` at 25 °C bounds the quartic-vs-`mm_wt74` gap at ≈3.2 % | ADOPTED as a documented uncertainty bound | §7.2 items 5, 6 |
| 5. `sw_tot` is `sw_sim` SCHLUM at `C=1, N=2`, closed form | ADOPTED | SB-SAT-008; SB-SAT-T12 |
| 6. IP's laminated double-correction interlock | ADOPTED | SB-SAT-041; SB-SAT-T57 |
| 7. Bateman-Konen and Kennedy agree to 0.07 % at 39161 ppm | ADOPTED | SB-SAT-T46 |
| 8. Test 10b's `max |ΔF| = 3.53e-4` bound | ADOPTED | SB-SAT-T54; ESC-7 |

### 8.16 Coverage summary

Counted from the tables above, not estimated.

| | Count |
|---|---|
| Dossier items enumerated | 211 |
| ADOPTED | 178 |
| ESCALATED | 21 |
| DEFERRED | 8 |
| EVIDENCE-ONLY | 6 |
| REJECTED (with reason) | 4 |
| Rows carrying two dispositions | 6 |
| Unaccounted | **0** |

The disposition counts sum to 217 because six rows carry two dispositions — §3.6, §3.10, §4.2's
Nigeria row, §5.2's `Salinity ↔ C₀` row and MJ-6 are each `ADOPTED + ESCALATED` (the mechanism is
adopted, the contested value is escalated), and §5.2's `Cm*` row is `ADOPTED + DEFERRED`.
217 − 6 = 211.

The 21 `ESCALATED` mentions resolve to the **nine** escalations in §7.1; several dossier items
converge on one decision — D-10, E-2, MJ-10, §3.10, BL-4 and the Shell `m` parameter row are all
ESC-2, for instance.

The four `REJECTED` rows are D-09 (out of domain — a neutron-porosity item, recorded rather than
dropped), `total_shale`'s Geolog `a` and `m` defaults (rejected as defaults under SB-SAT-034, with
their validation ranges retained), and the automatic-switch reading of §4.1's model-selection rule
(kept as guidance, rejected as behaviour). None is a silent omission.

---
