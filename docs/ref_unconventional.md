# Reference — Unconventional / Shale suite (enrichment #7)

Method math for the `unconventional.rs` module suite: **TOC (Passey ΔlogR + Schmoker)**,
**kerogen volume + OM-corrected porosity**, **gas-in-place (free + Langmuir-adsorbed, CBM)**, and
**brittleness (elastic + mineralogical)**. This file is the portable source of truth — it does not
rely on machine-local memory, and every module cites the primary paper in a code comment that points
back here.

**IP-cleanliness (playbook Part 0.5):** every method here is **Tier B** — published, citable science,
reimplemented from the primary source (never from vendor code/wording). The vendor **default values**
(LOM, baselines, Langmuir V_L/P_L, kerogen density) are **Tier A** — adopted as per-well-overridable
seeds from the IP/Techlog ingest, not as field truth. No Tier-C method (Omovie SonicSaturation, DTA)
is used or approximated. Constants below are hard-coded from the literature; defaults are exposed as
editable params.

Ingest evidence: `docs/research_2026-07/ip_ingest/C_module_defaults.json` (IP `TOC` + `GasStorageCap`
modules), `docs/research_2026-07/techlog_ingest/` (`TOC_Computation.py`, `A_family_assignment.json`
GIP/Ambrose families, `E_mineral_endpoints.json` Coal, `A_families.json` Kerogen),
`03. Guidebooks Techlog/…/RockPhyEquations.py` (elastic moduli).

---

## 1. TOC — Passey ΔlogR + Schmoker density  (module `toc_passey`, inc 1)

### 1.1 Passey ΔlogR — *Passey, Creaney, Kulla, Moretti & Stroud (1990), "A practical model for organic richness from porosity and resistivity logs," AAPG Bull. 74(12): 1777–1794.*

ΔlogR is the **separation** between a properly-baselined porosity curve and the deep resistivity
curve, overlain so they track through non-source rock. Three overlays (pick one via `OVERLAY`):

```
resistivity–sonic :   ΔlogR = log10(R / R_base) + 0.02  · (DT   − DT_base)
resistivity–density:  ΔlogR = log10(R / R_base) − 2.5   · (RHOB − RHOB_base)
resistivity–neutron:  ΔlogR = log10(R / R_base) − 4.0   · (NPHI − NPHI_base)
```

Units (exact): `R`, `R_base` in **ohm·m**; `DT`, `DT_base` in **µs/ft**; `RHOB`, `RHOB_base` in
**g/cc**; `NPHI`, `NPHI_base` as **fraction**; ΔlogR is dimensionless (resistivity log10 cycles).

The overlay coefficients are the reciprocal of the porosity span Passey overlays on **one resistivity
decade**: sonic `1/0.02 = 50 µs/ft`, density `1/2.5 = 0.4 g/cc`, neutron `1/4.0 = 0.25` per decade.

**Baselines are picked per-well/zone**, not universal — read where the two overlain curves overlie
and parallel (ΔlogR ≈ 0), i.e. a fine-grained, clay-rich, organically-lean (non-source) interval.
They are exposed as editable params (per-zone overridable); IP itself has no fixed baseline default
(it picks interactively, offsets = 0). Where ΔlogR < 0 the interval is non-source → TOC floors to the
background value.

### 1.2 TOC from ΔlogR — *Passey et al. 1990 (same paper)*

```
TOC = ΔlogR · 10^(2.297 − 0.1688·LOM) + TOC_background      [wt%]
```

- **LOM** = Level of Organic Maturity (Hood–Gutjahr–Heacock 1975 scale), dimensionless. Typical
  **6–12**; the conversion is calibrated only to **LOM ≤ 12** (beyond, into dry gas, it collapses).
  LOM ties to vitrinite Ro / Rock-Eval Tmax: ~6–7 immature/oil-onset (Ro 0.4–0.5), 8–11 oil window,
  ~12 wet-gas/overmature (Ro 1.6–2.0). Set from measured Ro/Tmax, or iterate to match core TOC.
- **TOC_background** = TOC of the non-source baseline rock (wt%); 0 if truly barren.
- Conversion factor at the vendor default LOM 10.6: `10^(2.297 − 0.1688·10.6) = 10^0.5077 = 3.219`.

### 1.3 Density-TOC cross-check — *Schmoker & Hester (1983), "Organic carbon in Bakken Formation," AAPG Bull. 67(12): 2165–2174.*

```
TOC_schmoker = 154.497 / RHOB − 57.261        [wt%], RHOB g/cc, clamp ≥ 0
```

The tidy Bakken-calibrated single-line density-deficit relation (a low-density-kerogen proxy). It is
basin-specific — a quick cross-check on the Passey curve, **not** a substitute. The more general
density-deficit form (Schmoker 1979) with editable ρ_matrix/ρ_kerogen/carbon-fraction lives in the
**kerogen** module (§2), where the kerogen density is already a parameter.

### 1.4 Module `toc_passey` design
- **opt** `OVERLAY` ∈ {sonic, density} — porosity curve paired with resistivity for ΔlogR. The
  **neutron** overlay (`−4.0·(NPHI−NPHI_base)`) is **deferred**: kerogen raises NPHI but the overlay
  scaling sign is inconsistent across the literature; ship only after verifying the sign against a
  Mahakam core. Sonic (+0.02) and density (−2.5) are unambiguous and physically checked (source rock:
  DT↑, RHOB↓, R↑ ⇒ both give a *positive* separation).
- **params** `R_BASE` (2.0 ohm·m), `DT_BASE` (70 µs/ft), `RHOB_BASE` (2.65 g/cc), `LOM` (10.6),
  `TOC_BG` (0.0 wt%).  *(baselines + LOM always visible — TOC is very sensitive to them.)*
- **inputs** `RES` (deep resistivity, required), `DT`, `RHOB` (optional; the one the overlay needs +
  RHOB always feeds the Schmoker cross-check).
- **outputs** `DLOGR` (separation, may be negative), `TOC` (Passey, wt%, clamp ≥ 0 — shared canonical
  name so `kerogen`/`gip`/`brittleness` pick it up by default), `TOC_SCHMOKER` (Bakken density-TOC,
  wt%, when RHOB present).

---

## 2. Kerogen volume + OM-corrected porosity  (module `kerogen`, inc 2)

Convert TOC (weight fraction of rock) to a **kerogen volume fraction**, and correct total porosity
for the organic-matter volume that low-density kerogen inflates on the density log.

TOC is a **weight** fraction; kerogen occupies a **volume** disproportionate to its weight because it
is light. Standard mass-balance conversion (e.g. *Passey et al. 2010, SPE 131350*; *Vernik & Nur
1992*):

```
TOM  = TOC_wt% / 100 · k_toc2om          organic-matter weight fraction (k_toc2om ≈ 1.2–1.4: OM is
                                          more than just carbon; default 1.2)
Vker = (TOM / ρ_kero) / [ TOM/ρ_kero + (1 − TOM)/ρ_matrix ]     kerogen volume fraction (v/v)
```

- **ρ_kero** kerogen density ≈ **1.1–1.25 g/cc** (Techlog Kerogen 1.1; IP RHOTOC 1.25 — default 1.20).
- **ρ_matrix** non-organic grain density ≈ **2.65–2.71 g/cc** (default 2.68).
- **k_toc2om** TOC→organic-matter conversion (Ro-dependent, ≈1.2 immature → ≈1.35 mature; default 1.2).

**OM-corrected total porosity** — the density-derived φ over-reads because kerogen (ρ≈1.1) mimics
pore fluid. Remove the kerogen volume so PHIT reflects the mineral+fluid pore system:

```
PHIT_c = PHIT_in − Vker            (clamp ≥ 0)      when PHIT computed on a mineral matrix that
                                                     excluded kerogen
```

Also emit a **kerogen-inclusive** view for GIP bookkeeping. The kerogen endpoints deliberately match
the SandiMin organic preset (Kerogen ρ 1.10), so a SandiMin `VOL_KEROGEN` and this module agree.

**Outputs** `VKER` (v/v), `PHIT_OMC` (OM-corrected total porosity), `TOM` (OM weight fraction).

---

## 3. Gas-in-place — free + Langmuir-adsorbed (+ CBM)  (module `gip`, inc 3)

Per-sample **intensive** GIP (gas content per ton of rock) so it composites and sums like any curve.

### 3.1 Langmuir adsorbed gas — *Langmuir (1918), J. Am. Chem. Soc. 40(9): 1361–1403; petro-application GRI / Mavor & Nelson (1996).*
```
Gs(P) = V_L · P / (P_L + P)            [scf/ton]
```
- **V_L** Langmuir volume (max sorption as P→∞), scf/ton.  **P_L** Langmuir pressure (Gs = V_L/2),
  psia.  **P** reservoir pressure, psia.
- Coal: correct dry-ash-free isotherm to in-situ:  `Gs_insitu = Gs · (1 − f_ash − f_moist)`.

### 3.2 Free gas (intensive) — *Ambrose et al. (2010) SPE 131772; Lewis et al. (2004).*
```
Gf = 32.0368 · φ · (1 − Sw) / (ρ_b · Bg)      [scf/ton]
```
- **ρ_b** bulk density g/cc; **Bg** gas FVF in reservoir-ft³/scf; 32.0368 = 907185 g/ton ÷ 28316.8
  cm³/ft³ = bulk ft³ per ton of rock.

> **CONSTANT CAVEAT (verified in research):** the often-quoted `1359.7` is **NOT** the free-gas
> constant — it is the **rock-mass** constant for the *adsorbed extensive* term
> (`G_ads[scf] = 1359.7·A·h·ρ_b·Gs`, A acres, h ft), exactly `43560·28316.8/907185` = short tons per
> acre-ft per (g/cc). Free gas is **volume**-based (÷Bg) → constant **43,560 (scf) / 43.56 (Mscf)**
> extensive, **32.0368 (scf/ton)** intensive. Do not put 1359.7 in the free-gas term.

### 3.3 Ambrose pore-volume correction — *Ambrose et al. (2010) SPE 131772.*
The adsorbed phase physically occupies pore volume, so free gas on total φ **double-counts**. Subtract
the adsorbed-phase volume:
```
Gf_corr = Gf − (ρ_b / ρ_ads) · Gs · (unit factor)
```
- **ρ_ads** adsorbed (liquid-like) CH₄ density ≈ **0.34–0.42 g/cc** (Ambrose used 0.34; default 0.34).
  Ignoring it over-estimates GIP in high-TOC/high-P shale. Techlog carries `AmbroseGIP`/
  `AmbroseFreeGIP` families — align naming for interoperability.

### 3.4 Gas FVF
```
Bg = 0.02827 · z · T / P        [reservoir-ft³/scf], T °Rankine (= °F + 459.67), P psia
```
- 0.02827 = 14.696 psia / 519.67 °R. Default **z ≈ 0.9** (first pass; Standing-Katz for rigor).

### 3.5 Total GIP & CBM critical desorption
```
Gtotal = Gf_corr + Gs            [scf/ton]   (+ dissolved, CBM water — <1%, optional)
P_cd   = P_L · Gc / (V_L − Gc)              critical desorption pressure (Gc = in-situ gas content)
```
- **Undersaturated** coal (P_i > P_cd): dewater to P_cd before gas flows. **Saturated** (P_i ≈ P_cd):
  gas from day 1. The isotherm panel (inc 5) marks P_cd and the reservoir-pressure point.
- CBM ash/moisture correction and the 3-stage dewatering curve: see skill `pe-cbm-unconventional`
  (`references/8-reservoir-eng-aspects.md`), Gi three-term split ≈ 90–95 % adsorbed.

### 3.6 Module `gip` design
- **params** `RES_P` (reservoir pressure, psia, 3000), `TEMP` (°F, 200), `Z_FAC` (0.9), `VL`
  (Langmuir volume scf/ton, 100), `PL` (Langmuir pressure psia, 1000 — matches IP 7000 kPaa), `RHO_ADS`
  (0.34 g/cc), `F_ASH` (0.0), `F_MOIST` (0.0), `SW` param fallback.
- **opt** `MODE` ∈ {shale, cbm} — cbm applies ash/moisture + emits P_cd.
- **inputs** `PHIE`/`PHIT`, `SW`, `RHOB` (+ optional per-sample `PRESS`, `GC` for P_cd).
- **outputs** `GIP_FREE`, `GIP_ADS`, `GIP_TOTAL` (scf/ton), `PCD` (psia, cbm).
- IP `GasStorageCap` seeds: V_L 60 cm³/g ≈ 1920 scf/ton and P_L 7000 kPaa ≈ 1015 psia are **generic
  placeholders** — flag that isotherm/desorption core data must override them.

---

## 4. Brittleness — elastic + mineralogical  (module `brittleness`, inc 4)

### 4.1 Dynamic elastic moduli — *Rock Physics Handbook (Mavko et al.); Techlog RockPhyEquations.py elastic2().*
From Vp, Vs (or slowness DT, DTS) and RHOB:
```
Vp = 1e6 / DT     Vs = 1e6 / DTS         [ft/s], DT/DTS µs/ft        (m/s form uses 304800/DT)
G  = ρ · Vs²                             shear modulus
K  = ρ · (Vp² − 4/3·Vs²)                 bulk modulus
ν  = (3K − 2G) / (2·(3K + G))            Poisson's ratio   (≡ (Vp²−2Vs²)/(2(Vp²−Vs²)))
E  = 9·K·G / (3K + G)                    Young's modulus   (≡ 2G(1+ν))
```
Field-unit shortcut: `G[psi] ≈ 1.34e10·RHOB/DTS²` (RHOB g/cc, DTS µs/ft), `E[Mpsi] = 2G(1+ν)/1e6`.
**Dynamic → static caveat:** log moduli are dynamic (high-freq/low-strain) and overestimate static E;
apply an empirical `E_static ≈ 0.4–0.8·E_dyn` factor (param `STAT_FAC`) before the brittleness index.

### 4.2 Elastic brittleness index — *Rickman, Mullen, Petre, Grieser & Kundert (2008), SPE 115258.*
```
E_norm = (E − 1) / (8 − 1)              E in Mpsi, range 1→8 (ductile→brittle)
ν_norm = (ν − 0.4) / (0.15 − 0.4)       ν range 0.4→0.15 (ductile→brittle; note the flip)
BI_elastic = (E_norm + ν_norm) / 2      clamp [0,1]
```

### 4.3 Mineralogical brittleness index — *Jarvie et al. (2007), AAPG Bull. 91; Wang & Gale (2009), GCAGS Trans. 59.*
```
Jarvie    : BI_min = Qz / (Qz + Cc + Clay)
Wang-Gale : BI_min = (Qz + Dol) / (Qz + Dol + Cc + Clay + TOC)     TOC as volume/weight fraction
```
Tie to SandiMin `VOL_<TOKEN>` outputs: quartz→`VOL_QUARTZ`, carbonate→`VOL_CALCITE`(+`VOL_DOLOMITE`),
clay→Σ clay volumes. The module takes them as `log_in` slots (`VQTZ`, `VCARB`, `VDOL`, `VCLAY`) so the
user maps whatever their SandiMin run produced — no hard-coded token dependence.

### 4.4 Module `brittleness` design
- **opt** `METHOD` ∈ {elastic, mineral_jarvie, mineral_wanggale}.
- **params** `STAT_FAC` (0.8), Rickman norms `E_LO` (1), `E_HI` (8), `NU_LO` (0.4), `NU_HI` (0.15).
- **inputs** elastic: `DT`, `DTS`, `RHOB`; mineral: `VQTZ`, `VCARB`, `VDOL`, `VCLAY`, `TOC`.
- **outputs** `BI` (0–1), plus `YME`/`PR` (Young's Mpsi / Poisson) for the elastic method.
- cargo test: BI monotone increasing in quartz fraction.

---

## 5. Tier-A vendor default seeds (per-well overridable)

| Quantity | Seed | Source |
|---|---|---|
| LOM (Passey) | **10.6** | IP `TOC` module |
| Shale/matrix density | **2.71 g/cc** | IP `RHOSH` |
| Kerogen / TOC density | **1.25 g/cc** (IP) / **1.10** (Techlog) → default **1.20** | IP `RHOTOC` / Techlog `A_families` |
| Coal density | **1.5 g/cc** | Techlog `E_mineral_endpoints` Coal #22 |
| Passey baselines (R/DT/RHOB/NPHI) | **interactive, offsets 0** — expose as params | IP (no fixed default) |
| Langmuir V_L / P_L | **≈1920 scf/ton (60 cm³/g) / 1015 psia (7000 kPaa)** — placeholders | IP `GasStorageCap` |
| Adsorbed-phase density ρ_ads | **0.34 g/cc** | Ambrose 2010 (literature; not in ingest) |
| Passey 0.02 / −2.5 / −4.0; TOC 2.297 / −0.1688; Schmoker 154.497 / −57.261 | **literature constants** | Passey 1990 / Schmoker-Hester 1983 (hard-coded) |

---

## 6. Increment plan
1. **`toc_passey`** — §1. Passey ΔlogR (3 overlays) + LOM→TOC + Schmoker-Hester cross-check. *(inc 1)*
2. **`kerogen`** — §2. TOC→kerogen volume + OM-corrected porosity; endpoints match SandiMin organic
   preset. *(inc 2)*
3. **`gip`** — §3. Free (intensive, Ambrose-corrected) + Langmuir-adsorbed + total + CBM P_cd. *(inc 3)*
4. **`brittleness`** — §4. Elastic (Rickman) + mineralogical (Jarvie / Wang-Gale). *(inc 4)*
5. **Custom panel** — ΔlogR overlay track (scaled R vs porosity baseline, separation shaded) +
   Langmuir isotherm crossplot (adsorbed gas vs P, reservoir-pressure + P_cd markers). PlotCanvas /
   readTheme. *(inc 5)*

Each ships as its own verified increment: `cargo test` (synthetic round-trips) + `cargo check` + `tsc`
+ browser smoke → REVIEW.md "Try:" line → commit. Modules 1–4 are plain manifest modules
(auto-dialog, category **Unconventional**); only inc 5 needs custom TS.
