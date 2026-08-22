# 10. Clay and shale volume — requirements

**Source dossier:** `docs/research_2026-08/cross_tool/clay-volume.md` (2,728 lines), including its
discrepancy ledger (§4.11), its OPEN list (§6) and its authoritative `## Critique disposition`
(§8).
**Evidence tiers held in this domain:** T1 (Geolog `vsh_*.lls` / `.info` executable source),
**T1′** (Techlog shipped equation `.gif`/`.png` renderings read at 3–9× from the installed
`Doc\image\` tree), **T1″** (IP decompiled CHM source read directly, including raw bytes and the
`embim*.png` equation images), T2 (IP2018/IP2025 full-manual ingest reports), T3 (install-tree /
catalog ingests: `techlog_ingest\C2_method_defaults.json`, `A_families.json`), T4 (petro-kb
literature, project-kb delivered-study records, memory atomics).
**Tier note.** T1′ and T1″ are the dossier's own refinements of CONTRACT §1.2's T1/T2 and are
carried unchanged. T1″ **outranks T2 on any transcription question** — §3.10 of the dossier is the
worked case where a vendor page overrules an ingest report by 23 %.
**Author date:** 2026-08-07.
**Requirements:** 55 (P0 13, P1 15, P2 19, P3 6, P4 2). **Parameters:** 58, of which **15 ship
`ABSENT — ships with no default`** and one ships `NON-ADOPTABLE — cited for verification`.
**Acceptance tests:** 44.
**Dossier items dispositioned in §8:** 240 — see §8's reconciliation note.

**Delegation statement.** This chapter ran entirely on the session model. No subagent was used at
any point — every equation, coefficient, clamp, endpoint and as-built line number below was read
in session, per the standing rule that petrophysical parameters and method math are never
delegated.

---

## 1. Scope and boundary

This chapter owns the **shale-volume and clay-volume indicator layer**: the single-log indicators
(gamma ray and its transforms, spectral Th/K, SP, neutron, resistivity, EM propagation), the
two-log "double" indicators (neutron-density, sonic-density, neutron-sonic, user pair), the
NMR-native clay volume, the **combination layer** that merges them into one curve, the
**endpoint-picking machinery** that produces the clean and clay/shale parameters those indicators
consume, the **absence model** (bad hole, per-zone disable, coal), the **organic-shale
pre-corrections** applied to the input curves before the indicators run, and the **Vsh ↔ Vcl
bridge**.

It does **not** own the following, and each seam is named here rather than discovered at index
time.

**Porosity (`POR`).** Shale-corrected porosity consumes `VSH`/`VCL` as an input and is where the
shale-porosity term `φsh` lives. This chapter fixes what the volume *is* and what type it carries;
`POR` fixes what is done with it. The one place they touch numerically is
`clsr_porosity_corrected` (`Vcl = CLSR × (Vsh − φsh)`), which needs a `φsh` this chapter does not
define — see SB-CLY-044.

**Multi-mineral solver (`MIN`).** A volumetric solver derives clay as one of its components rather
than from a deterministic indicator. SandiBumi's SandiMin and its retired `multimin` predecessor
sit there. The seam is the *typed* output of SB-CLY-043: a solver-derived clay volume and an
indicator-derived clay volume are different estimates of the same quantity and must not resolve to
one another by mnemonic collision. IP's SSM coal override (which sets `Vcoal = 1` and zeroes
porosity and saturation) is a model-level override and belongs to `MIN`, not here — see
SB-CLY-036.

**Thin-bed and laminated analysis (`TBD`).** Thomas-Stieber partitions a *bulk* `VSH` into
laminar / dispersed / structural components. Techlog ships a full ten-page Thomas-Stieber module
and a shale-volume-vs-porosity plot; IP has its own route via `porosityandwatersaturation`
reformulated for `Phie` and `Vcl`. The dossier read only the plot page (§1.3) and declared the rest
out of scope. SandiBumi's `thin_bed_ts` module (`modules.rs:2432`) consumes `VSH` and `PHIT` and
belongs to `TBD`. This chapter's obligation stops at delivering a `VSH` that is typed, clipped and
provenance-carrying enough for `TBD` to consume.

**Cutoffs, summation and Monte Carlo (`CUT`).** `VShale limit 0.3 v/v` and
`Shale Volume_Min 0 / Max 0.5` in Techlog's shipped Quanti templates are **cutoff/net-pay**
parameters, not clipping bounds on the Vsh computation (dossier §2.11). They are named here only
so they are not mistaken for clamps; they belong to `CUT`.

**Environmental corrections and log QC (`ENV`).** Borehole-corrected GR / RHOB / NPHI, the
neutron matrix-scale gate and the bad-hole discriminator curves themselves are produced by `ENV`.
This chapter owns only what the indicator layer does with them: SB-CLY-041 (prefer the corrected
alias), SB-CLY-013 (enforce the limestone-matrix contract) and SB-CLY-035 (the discriminator
rule).

**Data import, export, formats (`DIO`).** Vendor parameter-set import (IP ordinals, Stieber
spelling aliases, the `Gaymar`/`Gaymard` split) and LAS null discipline are `DIO`'s machinery.
SB-CLY-003, SB-CLY-052 and SB-CLY-055 are stated here because the *semantics* they protect are
clay-volume semantics; `DIO` owns the file format.

**Database and project data model (`DBM`).** The mnemonic-family dictionary is `DBM`'s structure.
SB-CLY-046 raises a clay-domain requirement against it — SandiBumi's `FAMILIES` table
(`curves.rs:21-37`) has **no Vsh and no Vcl family at all**, while Techlog registers four
(`Shale Volume Fraction`, `Shale Volume Fraction Unclipped`, `Clay Volume Fraction`,
`Average Shale Volume`, T3 `A_families.json`). The requirement is allocated here because the type
distinction it encodes is this chapter's; the table is `DBM`'s to hold.

**Plotting and interactivity (`PLT`).** Histogram and crossplot endpoint picking is `PLT`'s
surface. SB-CLY-037 through SB-CLY-039 specify the *derivation rule* behind a pick, not the
gesture.

---

## 2. What the incumbents do — the requirement-bearing findings

Twenty-two findings. Each generates at least one obligation in §4. Findings from the dossier that
generate no obligation are accounted for in §8, not padded into here.

### F1 — IP and Techlog document no domain clamp, and a hot shale reads as a clean sand

**Tier T1 / T1″ / T1′. Tools: Geolog clamps; IP and Techlog do not.**
Geolog clamps the GR index into each transform's analytic domain before evaluating it —
`LIMIT(v, −10, 1.49)` for Stieber `I/(3−2I)`, `1.99` for `I/(2−I)`, `1.33` for `I/(4−3I)`,
`LIMIT(v, −2.53, 1.13)` for Clavier (T1 `vsh_gr.lls` L115/L119/L123/L136), recorded in its own
file history as a deliberate 1997 fix (L6, L8). IP and Techlog state no clamp on any GR transform.

The consequence is not a NaN. At `I = 1.60` with the shipped shape `n = 2`, the unclamped Stieber
gives `1.6/(3 − 3.2) = 1.6/(−0.2) = −8.000`, and a naive final clip to [0,1] maps that to
**0.000**. A 100 %-shale reading emerges as clean sand, and nothing downstream — porosity,
saturation, net-pay summation — can detect it. Clavier goes NaN instead (`√(−0.23)` at `I = 1.20`),
which is at least loud.

**`I > 1.5` is reachable in this practice, not hypothetical.** From the Balam South record (T4
`project-kb\records\lqr-balam-south-phr.md`, Final Report Sec 4.5.1 / Table 4.7),
`GR_MA = 30 gAPI`, `GR_SH = 150 gAPI`; `I = 1.5` needs `GR = 30 + 1.5×120 = 210 gAPI`. Under the
same record's GR-normalisation pair (`P3 = 53.68`, `P97 = 133.93 gAPI`, Sec 4.3.6), `I = 1.5` needs
only `GR = 53.68 + 1.5×80.25 = 174.1 gAPI`. A P97 shale pick puts 3 % of samples above `I = 1.0`
by construction, and a uranium-rich streak at 175–215 gAPI in a Tertiary section is an ordinary log
feature. **The interaction between a percentile-derived shale endpoint and a poled transform is
documented by no vendor**, and it is the finding in this domain most likely to bite this work.

→ SB-CLY-009, SB-CLY-010, SB-CLY-040.

### F2 — The rounded Larionov coefficients break the one boundary condition the family exists to satisfy

**Tier T1′ decisive. Tools: Techlog contradicts itself; the exact form is vendor-printed.**
Three renderings ship: exact normalised `(2^(kI) − 1)/(2^k − 1)` on **Techlog's own Th and K
pages** (T1′ `…-thor3/thor4/pota2.gif`); `0.333` / `0.08336` in IP (T1″ `embim27`/`embim28`);
`0.33` / `0.083` in Geolog (T1 `vsh_gr.lls` L127/L130) **and on Techlog's GR page** (T1′
`image471.gif` / `image470.gif`). `1/(2²−1) = 0.3333333…`; `1/(2^3.7−1) = 1/11.99604 =
0.0833609…`.

The rounded forms are systematically low and the error peaks exactly at pure shale: at `I = 1`
the older-rocks form gives **0.9900** against 1.0000 (**1.00 % relative**) and the younger gives
**0.9957** (**0.43 %**). Every sibling transform closes at 1 exactly — Clavier
(`1.7 − √(3.38 − 2.89) = 1.700 − 0.700 = 1.000`) and every Stieber variant (`1/(1 + n·0) = 1`).
**Only the rounded Larionov forms break it**, and the fix is free. The resolution is evidential,
not aesthetic: one vendor prints the exact form.

→ SB-CLY-004, SB-CLY-005.

### F3 — Four equations are all called "Vsh from resistivity" and they span a factor of 5.0

**Tier T1 / T1″ / T1′. Tools: all three, four distinct forms.**
On `R_clean = 100`, `R_clay = 2`, `Rt = 20 ohm·m` — the moderately resistive hydrocarbon sand over
a conductive shale that all three tools say the indicator is *for*, with `Z = 0.081633`:

| Form | Source | Vsh |
|---|---|---|
| `gaymard_fixed_b` (`b = 1`, Techlog's shipped default) | T1′ `…-shale-r1.gif` + T3 `"b exponent": 1` | **0.0816** |
| `ip_power_branch` `0.5(2Z)^(0.67(Z+1))` | T1″ `embim31`/`embim32` | **0.1344** |
| `gaymard_variable_b` `Z^(0.5/(1−Rsh/Rt))` | T1 `vsh_res.lls` | **0.2486** |
| `log_linear` | T1′ `…-shale-r2.gif` | **0.4114** |

**0.0816 → 0.4114 is 5.04×.** No default is defensible. Geolog's branch is exactly continuous at
`Rsh/Rt = 0.5` (`1/b = 1 ⇒ Z¹ = Z`); IP's is not (Δ = 0.00169 at the boundary) — a regression suite
must pin both behaviours rather than "fix" IP's discontinuity.

→ SB-CLY-015, SB-CLY-017.

### F4 — Clip-before-average and average-then-clip differ by 20–60 % and only IP writes the order down

**Tier T2 / T1. Tools: IP clips first; Geolog clips last; Techlog is silent.**
IP: *"For the VCLAV curve, the separate Vclay indicator curves will first be clipped before the
average curve is created."* (`clayvolume.htm`, T2). Geolog's `vsh_avg.lls` sums the raw `VSH_xx`
and applies `LIMIT(VSH_AVG, 0, 1)` only to the result (T1). These do not commute for a mean.

Case A — a hot shale on GR with good ND and SP, `{1.30, 0.20, 0.30}`: IP gives `(1.00+0.20+0.30)/3
= **0.500**`; Geolog gives `(1.30+0.20+0.30)/3 = **0.600**`. Δ = 0.100 v/v, **20 % relative**.
Case B — a clean sand where the N-D separation over-corrects, `{−0.15, 0.40}`: IP gives **0.200**;
Geolog gives **0.125**. Δ = 0.075 v/v, **60 % relative on the smaller value**. Geolog's own change
log concedes the class in the *minimum* module (*"Jan 2017 (TOS) Remove bug for when any vshale
input is < 0"*, T1 `vsh_min.lls`) while `vsh_avg.lls` still sums raw values. For a minimum the two
orders commute; specifying the rule uniformly makes it testable rather than special-cased.

→ SB-CLY-027.

### F5 — Sandstone Δt_matrix ships as four values from five witnesses, worth 23.6 % on Vsh_SD

**Tier T1 / T1′ / T1″ / T2 / T3. Tools: all three, and two vendors disagree with themselves.**

| Value | Witness | Tier |
|---|---|---|
| 50 uS/ft | Techlog **doc page** `petrophysics-vsh-from-sonicdensity.html` | T1′ |
| 55 uS/ft | IP **BLA** `basicloganalysis.htm` *"Defaults to 55 uSec/ft for sandstone"* | T1″ |
| 55.5 uS/ft | Techlog **shipped Quanti template** `C2_method_defaults.json DT_matrix` | T3 |
| 55.50 uS/ft | Geolog `vsh_ds.info DT_MA 182.1 us/m` ÷ 0.3048 | T1 |
| 56 uS/ft | IP **PhiSw** `Sonic Sand` (ledger D-13 resolved) | T2 |

On a realistic shaly sand (`Δt = 90`, `ρb = 2.35`, `Δt_sh = 100`, `Δt_fl = 189`, `ρ_ma = 2.65`,
`ρ_sh = 2.40`, `ρ_fl = 1.00`) Vsh_SD runs **0.5089 / 0.4307 / 0.4213 / 0.4117** — a 0.097 v/v
absolute, **23.6 % relative** swing from a matrix default alone, monotone-decreasing in Δt_ma as it
must be. **Two of the four values come from one vendor disagreeing with itself**: Techlog's doc
page ships 50 and its own template ships 55.5, a **20.8 % relative** internal spread against IP's
55-vs-56 (2.2 %). An implementer reading only Techlog's Vsh doc page inherits the extreme of the
range while believing they inherited a vendor consensus.

→ SB-CLY-050, SB-CLY-053.

### F6 — A zero-dropping median is unusable on a bounded volume fraction, and a vendor documents one

**Tier T1′. Tools: Techlog documents both definitions, on two pages, of the same nine methods.**
`petrophysics-shale-volume.html`: *"Median: order var i smallest to highest value, **different than
0 (0 value are ignored)**…"*. `petrophysics-vsh-final.html`: *"Median: median value of all the
variables used as inputs."* — no zero clause. On a clean sand reading `{0.00, 0.00, 0.18}` the
zero-dropping median returns **0.18** and the zero-including median returns **0.00** — a
factor-of-∞ disagreement on the cleanest rock in the well, between two definitions printed by one
vendor. Techlog's Harmonic mean carries the same defect by a different route (*"for vari >
0.0001"*), and its Geometric mean and Product collapse to exactly 0 whenever any indicator
legitimately reads 0, which is the normal case in clean sand. **Which page describes Techlog's code
is not settleable from disk** (dossier OPEN 4a) — but the obligation does not depend on the answer:
on a bounded volume fraction whose clean-rock value *is* zero, treating `0.00` as absent conflates
the most common legitimate reading in the reservoir with missing data.

→ SB-CLY-028, SB-CLY-029.

### F7 — The neutron indicator's whole spread is the clean endpoint, and one vendor ships two of them

**Tier T1 / T1″ / T1′ / T3.**
On `φN = 0.30`, `φN_clay = 0.40`: Geolog's ratio gives **0.7500**; IP's √ hybrid at `φN_clean = 0`
gives **0.7500** (it collapses exactly to the ratio, `√(x·x) = x`); at `−0.1` **0.7746**; at `+0.1`
**0.7071**. Techlog's two-point linear gives **0.8000** at its *doc-page* `φN_ma = −0.1 v/v` (T1′)
and **0.7500** at its *shipped-template* `NPHI_matrix = 0` (T3, all four `Q*_PR.xml`).

**At `φN_clean = 0` all three tools return exactly 0.7500.** The entire **0.7071 → 0.8000 spread —
13.1 % relative — is generated by moving the clean endpoint over ±0.1 v/v**, a range narrower than
the gap between one vendor's own two shipped defaults. IP warns only about the positive direction
(*"exercise caution if setting this parameter to any other value than zero. This indicator can
easily under-estimate the clay volume if the parameter is set too high"*, `clayparameters.htm`);
**the negative direction is live, unwarned by anyone, and is what Techlog's doc page ships.** Zero
is the only value at which the three tools coincide, which makes zero the correct silent value and
everything else a value that must announce itself.

→ SB-CLY-012, SB-CLY-014, SB-CLY-050.

### F8 — Bad hole: IP nulls, Geolog and Techlog substitute and label

**Tier T2 / T1 / T3.**
IP nulls the double indicators to **−999** over bad hole and leaves the singles ungated. Geolog
substitutes `VSH_GR` and writes it into its provenance curve. Techlog lets the user choose a
constant or a substitute curve, and its **shipped template chooses GR** — `"VSH selection if flag =
1": "Gamma ray (GR)"` (T3), which is Geolog's hard-coded policy expressed as a preset. **Two vendors
independently converge on "on bad hole, trust GR"; IP is the outlier in nulling.** Over a washed-out
interval IP produces no double-indicator value at all (and no combined value if the combiner is a
min over doubles), while Geolog and Techlog produce a GR-based value *marked as such*. For net-pay
summation over a rugose section that is the difference between a gap and a number, and only
Geolog's `MTH_VSH` makes the substitution auditable.

→ SB-CLY-031, SB-CLY-033.

### F9 — The IP organic-shale corrections renormalise, and the ingest report that dropped it costs 23 %

**Tier T1″ overruling T2 — the dossier's own §3.10, and the most consequential single item in it.**
Both editions of `clayequationsandmethodology.htm` are character-identical at byte level and read
`NeuCorr = (Neu_in - TOCvol x NeuKer \x96 HvyMinVol x NeuHvy) / (1.0 - TOCvol - HvyMinVol)`. The
IP2025 ingest report transcribed this **without the denominator** and asserted a GR/Neutron/Density
"asymmetry" that is not in the manual. At a realistic organic shale (`TOCvol = 0.20`,
`HvyMinVol = 0.03`, `Neu_in = 0.35`, `NeuKer = 0.6`, `NeuHvy = −0.03`) the correct form gives
`NeuCorr = **0.2999**` and the report's form gives **0.2309** — a **0.069 v/v absolute / 23 %
relative** error in the corrected neutron curve, propagating into `VclNeu` and `VclND`.

The `0x96` operator is a **minus**, closed evidentially rather than typographically: the byte
appears at the same position in both editions; the tree is CP1252 despite declaring
`charset=ISO-8859-1` (80 pages carry `0x96`, 119 carry the CP1252-only `0x91`/`0x92`); and the same
vendor uses the identical byte as unambiguous subtraction in `SwN = (Sw \x96 Swirr)/(1 \x96 Swirr)`
on `cappressurefunctions.htm`. **The real asymmetry is GR alone** — GR is an intensity that scales
with the radioactive fraction and does not renormalise; ρb, φN and Δt are volumetric mixing
quantities that must. The report had it backwards.

→ SB-CLY-047, SB-CLY-048, SB-CLY-051.

### F10 — "Stieber", numbered, denotes different equations in different tools

**Tier T1 / T1′ / T1″, each cell read from the vendor artefact.**

| Equation | Shape `n` | Geolog code | Techlog label | IP |
|---|---|---|---|---|
| `I/(2−I)` | 1 | `STIEBER2` (`vsh_gr.lls` L120) | "Stieber variation I" (`…-gr3.gif`) | `STB = 1` |
| `I/(3−2I)` | 2 | `STIEBER1` (L116) | "Stieber — Miocene and Pliocene" (`…-gr4.gif`) | `STB = 2.0` (**shipped default**) |
| `I/(4−3I)` | 3 | `STIEBER3` (L124) | "Stieber variation II" (`…-gr5.gif`) | `STB = 3` |

**The numeral is anti-correlated between Geolog and Techlog.** IP sidesteps the naming entirely by
exposing the shape constant, and its generic form `I/(1 + n(1−I))` subsumes all three as special
cases. Ledger **D-03** records the `Stieber`/`Steiber` spelling split; §2.2 of the dossier shows the
spelling is the *lesser* problem — a correctly-spelled label carrying the wrong equation is worse
than a misspelled one. A vendor variant label that cannot be resolved to a shape constant is an
import error, not a best guess.

→ SB-CLY-002, SB-CLY-003.

### F11 — Geolog's resistivity guard lives in one branch and its ordering defeats its second test

**Tier T1, read from `vsh_res.lls` L98–L115 rather than from the history line.**
Geolog is the only tool with defensive handling on this indicator (*"Jun 1999 (WWC) Prevent root of
negative number"*, L10) — and it has two holes. (1) The guards live only in the nonlinear branch;
the linear branch at L98–L102 has none, so `RT_HI > RT_MAX` there drives `VSH_RES` negative and the
final `LIMIT(...,0,1)` maps it to a clean **0.000** with no message — the same silent-zero signature
as the Stieber pole. (2) `RT_HI >= RT_MAX` is tested first (L106) and `RT_SH >= RT_MAX` only in its
`elseif` (L110); when both hold, the first branch clips `RT_HI` so the numerator stays positive
while the denominator carries `(RT_MAX − RT_SH) ≤ 0`, `Z` goes negative, and it is raised to a
fractional power — **exactly the failure the 1999 fix was written to prevent.** The validity
condition must be checked before branching, not inside one arm of it.

→ SB-CLY-016.

### F12 — Techlog's sonic-density denominator carries a doubled negation

**Tier T1′, verified at 9× from the shipped `modules-quanti-volume-shale-ds.gif`.** It literally
reads `B = (Δt_ma − Δt_f) * (ρ_sh − ρ_f) − (ρ_ma − ρ_f) * −(Δt_sh − Δt_f)`. Read as printed,
`B = −P·w_s − Q·u_s + 2PQ`, which is neither the canonical denominator nor a normalisation of the
printed numerator `A`. Read with a single minus, `B = Q·u_s − P·w_s` — exactly `A` re-evaluated at
the shale point, which is what a normalised ratio requires. `A` was verified to reduce to
`Q·u − P·w` exactly. **That is an argument, not vendor evidence**, so neither reading is adopted;
the canonical form is implemented and the vendor defect is recorded with both readings preserved.

→ SB-CLY-018, SB-CLY-022.

### F13 — Geolog's combination layer ships a mislabelled estimator, a magic sentinel and a two-vocabulary provenance curve

**Tier T1, three separate defects in one module family.**
(a) `vsh_hl` is titled "Hodges-Lehman" and outputs `VSH_HL`, but the code loads the indicators into
`v[9]`, exchange-sorts (L150–156) and takes the plain middle value (L167–173) — a **median**, not a
pseudomedian. Its own doc block already says median and the history shows the prose was fixed
(*"Jan 2010 (WWC) Correct doc"*); **the module name, program title and output curve name were
not** — so the misnomer survives in exactly the places that reach a deliverable header while the
correction sits where nobody exports it.
(b) `vsh_min` detects "nothing selected" with `VSH_MIN = 999.99` then `if (VSH_MIN == 999.99)` — a
magic sentinel in value space — while `vsh_avg` (`ntot` counter) and `vsh_hl` (`vc` counter) in the
same directory use the correct pattern. The vendor already knew.
(c) On bad-hole substitution the code writes `MTH_VSH = OPT_GR` (T1 `vsh_nphi.lls` L102 and the
same line in `vsh_dn/ds/ns/mn`), where `OPT_GR` is the *option* vocabulary (`LINEAR`, `STIEBER1`,
`CLAVIER`) and `MTH_VSH` everywhere else carries the *method* vocabulary (`GR_LIN`, `GR_STIE1`,
`GR_CLAV`). **One curve, two enumerations, and which one you get encodes whether the sample came
from a substitution** — recoverable only by a consumer that knows both vocabularies and why. The
`.info` `DESCRIPTION_DETAIL` documents only one.

→ SB-CLY-031, SB-CLY-032, SB-CLY-034.

### F14 — Techlog ships three indicators whose shipped endpoints their own equations cannot evaluate

**Tier T3, from `C2_method_defaults.json`, present identically in all four `Q*_PR.xml` templates.**
`TH_matrix = TH_shale = 0`; `POTA_matrix = POTA_shale = 0` — a zero-span index `(x − 0)/(0 − 0)`.
`Res_limit = 1` with `Res_shale = 10 ohm.m` — `R_clay > R_clean`, the exact condition Geolog guards
and that inverts the resistivity index. **Two of the five single-indicator endpoint sets Techlog
ships are unevaluable as shipped.** This is not a defensive hypothetical for a `DEGENERATE_ENDPOINTS`
guard; it is the shipped state of three of a competitor's indicators.

→ SB-CLY-001, SB-CLY-016.

### F15 — IP's two-point clean line is strictly more general, and its linkage asymmetry is the feature

**Tier T1″ / T1 / T1′ / T4.**
All three tools implement the identical bilinear double-indicator geometry — verified by reducing
each printed form to the canonical `Vsh = (Q·u − P·w)/(Q·u_s − P·w_s)` (dossier §2.7). They differ
in *parameterisation*: IP's clean line is two arbitrary user points (*"the two ends of the clean
line"*, T1″ `clayequationsandmethodology.htm`), while Geolog's and Techlog's is forced through the
matrix point and the fluid point. Only IP's can express the light-hydrocarbon clean-line-slope
adjustment (Spooner 2014 §8, T4). **Geolog admits the cost in its own doc**: *"the user will
probably need to use an artificial fluid point (NPHI_FL, RHO_FL) to account for the non-linearity
of matrix lines other than limestone."*

IP then links **Clean 1 across doubles sharing a curve** (`ND Den Clean 1` ↔ `SD Den Clean 1`) and
**never links Clean 2** (T1″ `clayparameters.htm`). IP does not say why; the geometry does. `c2` is
where the per-indicator slope adjustment lives, and the light-hydrocarbon correction sets a
*different* `c2` on the ND crossplot than on SD or NS. Linking `c2` would silently propagate one
indicator's hydrocarbon correction onto two others where it is wrong. IP also refuses to link
doubles to singles (*"Setting this parameter to on DOES NOT update Single Clay Indicators"*),
because a single's endpoint is picked on a histogram and a double's on a crossplot and they are not
the same estimate. **The asymmetry is structural, not a UI preference.**

→ SB-CLY-018, SB-CLY-019, SB-CLY-020.

### F16 — Three incompatible Vsh↔Vcl positions, and the two published bridges differ by 30 %

**Tier T2 / T1 / T3 / T4.**
IP forces an explicit pre-run choice (*"a fundamental decision has to be made whether you which to
calculate Clay volume or Shale volume"*) and supplies a bridge `Vshale = VWCL / CSR` with endpoint
identities `Rho Wet Clay = Rho Matrix + ((Rho Shale − Rho Matrix)/CSR)` — but **no default for
CSR**. Geolog is Vsh-native across all eight classical indicators and ships exactly one
deterministic Vcl, `VCL_NMR = limit(VOL_CBW_NMR/PHIT_NMR, 0, 1)` (T1 `vsh_nmr.lls` L54) — a
**measurement-native** clay volume from NMR clay-bound water, not a converted Vsh — and **no
Vsh→Vcl bridge anywhere in the determin chain**. Techlog registers a `Clay Volume Fraction` family
with no evidence of how it is filled. The three positions are therefore **bridge / partial /
unknown**, not "Vcl / Vsh / Vsh".

A competing bridge exists and must not be silently merged with IP's: Halliburton's
`Vcl = CLSR × (Vsh − φsh)` with `CLSR ≈ 0.6` (T4 `reference_vsh_porosity_methods.md`) **subtracts
shale porosity first**. At `Vsh = 0.5`, `φsh = 0.15`, `CSR = CLSR = 0.6`: IP gives `Vcl = 0.300`,
Halliburton gives `0.6 × 0.35 = 0.210`. **A 30 % relative difference in the quantity every
volumetric solver consumes.** Spooner 2014 (T4) adds the validity limit — a fixed CSR extrapolated
blindly into real shale beds clips Vclay and overestimates φe.

→ SB-CLY-043, SB-CLY-044, SB-CLY-045, SB-CLY-046.

### F17 — Only IP specifies endpoint derivation; Geolog has no picking rule of any kind

**Tier T2 / T1 (grep-verified) / T1′.**
IP's pipeline is **pool by `Percentile Group` → clip low/high (`0 %` / `98 %`) → compute percentile
→ linear-extrapolate outside 0–100 % → that is the endpoint**, with two-way binding (turning
percentiles off back-computes the percentile from the entered value) and `Percentile Clay = 130 %`
sitting deliberately *above* the observed maximum. **Geolog has none**: a grep of every
`vsh_*.lls` and `vsh_*.info` for `percentile` and `quantile` returns **zero hits**; its endpoints
are plain well constants with validation ranges only. Techlog has one boilerplate quantile-5/95
menu pasted verbatim across its single-log pages (identical prose on the GR and EM pages), which is
one picking mechanism, not eleven design choices.

Direction matters: IP's 130 % clay percentile sits *above* the observed maximum, Techlog's quantile
95 and this practice's P97 house standard sit *below* it. Higher `GRshale` ⇒ lower `I` ⇒ lower Vsh
and a lower probability of reaching a Stieber pole. **The house standard raises the F1 risk relative
to IP's own convention**, which is why SB-CLY-040 exists.

→ SB-CLY-037, SB-CLY-038, SB-CLY-039, SB-CLY-040, SB-CLY-042.

### F18 — Geolog's corrected-curve discipline reaches five modules of six and is complete in two

**Tier T1, from the `.info` `INPUT` `DEFAULT` column.**
Geolog defaults its GR input to the alias **`GR_COR`**, and `RHO → RHO_COR`, `NPHI → NPHI_COR` on
the doubles — a correct design choice no other tool makes, enforcing "borehole-correct first" as a
default rather than a note. But: `vsh_gr` 1 of 1; `vsh_dn` 2 of 2; `vsh_ds` 1 of 2; `vsh_ns` 1 of 2;
`vsh_mn` 2 of 3; **`vsh_nphi` 0 of 1**. The `DT` exceptions are not a lapse — **no `DT_COR` alias
exists anywhere in the family**. `vsh_nphi` is the real exception and it is a triple one: it is the
only module that defaults to the raw curve, the only one with an **empty** input validation range,
and the only one with an **empty** validation on its sole endpoint `NPHI_SH` (where `vsh_dn.info`
gives `0:1`). That is the module Geolog itself documents as an *upper bound* and therefore the one
most likely to win a `vsh_min`. **A discipline applied to five modules of six is worse than none,
because it teaches users to trust it.**

→ SB-CLY-041.

### F19 — Geolog's M–N module's documentation contradicts its own code by a transposed digit

**Tier T1, same file.** `vsh_mn.lls`'s doc block states the reference line `M = 0.604 + 0.308*N`;
the executable code on the same file uses **`0.388`**. Both are recorded verbatim and **neither is
adopted**. The code is authoritative for what Geolog computes; which value is physically right needs
the 1995 M–N chart the doc block cites — a chart cited here by existence, attribution and purpose
only, with no tabulated data transcribed. The M–N indicator has no IP or Techlog counterpart.

→ SB-CLY-025 (deliberate non-implementation), §7 Escalation E1.

### F20 — The unit surface is where silent wrongness enters, and one tool's internals differ from the other two

**Tier T1 / T1′ / T1″ / T3.** Geolog is internally **k/m3** and **us/m**; IP and Techlog are g/cc
and uS/ft. Techlog adds Th in **ppm**, K in **%** and EM `TPL` in **dB/m**. The conversions are
mandatory, not cosmetic: 0.3048 on slowness, 1000 on density. Geolog's `DT_MA` validation
`100:300 us/m` is `30.5:91.4 us/ft` — a Techlog default of `50 us/ft` is `164 us/m`, inside range;
a carelessly-passed `50 us/m` is `16.4 us/ft`, **silently outside physics and inside no guard on
the IP/Techlog side**. `vsh_mn.lls`'s bare `3.048` and `1000` scale factors are exactly the magic
constant pattern that makes this class of error possible. Techlog's template additionally leaves
`DT_matrix`, `DT_fluid`, `NPHI_matrix`, `RHOB_matrix` and `SP_matrix` with **empty unit strings**.

→ SB-CLY-054.

### F21 — IP alone distinguishes three kinds of absence, and collapsing them corrupts the contributor count

**Tier T1″ `clayplot.htm` / `clayparameters.htm`.**
(a) a genuinely null input at a depth; (b) a bad-hole gate — doubles set to `−999`, depth-scoped;
(c) a per-zone `Use` flag — *"Set to Off for Vclay from gamma ray to be set to **Null** values over
this zone"* — zone-scoped and method-scoped, not a property of the data at a depth. They differ in
scope, in what they mean to a reviewer, and in whether the indicator should count toward the
contributor total in a provenance output. **A reviewer must be able to tell "washed out" from "you
switched it off in this zone" from "coal" without opening the parameter set.**

IP's bad-hole discriminator semantics are inverted relative to their names and must not be
paraphrased: *"(46) **BadH1 Min** … When the Bad Hole Indicator 1 curve values are **greater than**
this minimum value, any double clay indicators will be switched off"*; *"(47) **BadH1 Max** … when
… **less than** this maximum value"*; *"When the parameter is left blank, the discriminator curve is
ignored."* Two discriminator curves × four independently-blankable thresholds, all live at once.
**A `{flag_curve, flag_value, action}` equality test cannot express a threshold on a caliper at
all**, and cannot express a two-sided open interval under any encoding; Techlog's 0/1 flag curve is
the degenerate case (`min = 0.5`, no max), not the general form.

→ SB-CLY-030, SB-CLY-035.

### F22 — Geolog ships a Larionov variant that exists in no other tool, is uncited, and overshoots by 13 %

**Tier T1 `vsh_gr.lls` L133.** `LARINOV3 = 0.127 × (3.15^(2I) − 1)`. At `I = 1` it gives
`0.127 × 8.9225 = **1.1332**`, overshooting the boundary condition by **13.3 %** where every other
transform in the family closes at 1 either exactly or within 1 %. Geolog gives **no citation**.
Whether it is a deliberately aggressive curve clipped at 1, or a defect, is not established.

→ SB-CLY-006, §7 Escalation E2.

---

## 3. SandiBumi as-built

Read from source on 2026-08-07. Every claim below carries `file.rs:line`. The repository was
read-only for this task except this file.

### 3.1 Inventory — two indicators of the twelve the incumbents ship between them

`list_modules` (`modules.rs:342-394`) registers exactly two modules in the `VSH` category:
`vsh_gr_spec()` (`modules.rs:344`) and `vsh_dn_spec()` (`modules.rs:345`), dispatched at
`modules.rs:422-423`. Nothing else in the repository computes a shale or clay volume from logs; a
grep for `vsh_res`, `vsh_sp`, `vsh_nphi`, `vsh_ds`, `vsh_ns`, `vsh_mn`, `vsh_nmr` across
`src-tauri/src/` returns **zero hits**.

| Indicator | Incumbent coverage | SandiBumi status |
|---|---|---|
| Gamma ray + transforms | all three | `PRESENT-DIVERGENT` — §3.2 |
| Neutron-density double | all three | `PRESENT-DIVERGENT` — §3.4 |
| SP | all three | `ABSENT` |
| Neutron (single) | all three, three different equations | `ABSENT` |
| Resistivity | all three, four different equations | `ABSENT` |
| Sonic-density double | all three | `ABSENT` |
| Neutron-sonic double | all three | `ABSENT` |
| Other/user double | IP, Techlog | `ABSENT` |
| Thorium / Potassium | Techlog only | `ABSENT` |
| EM propagation (`TPL`) | Techlog only | `ABSENT` |
| M–N crossplot | Geolog only | `ABSENT` — deliberate, SB-CLY-025 |
| NMR (`VCL_NMR`) | Geolog only | `ABSENT` |
| **Combination layer** (min / mean / median / pseudomedian) | all three | **`ABSENT`** |
| **Endpoint-picking machinery** | IP full, Techlog quantile menu | **`ABSENT`** |
| **Vsh→Vcl bridge** | IP (`CSR`) | **`ABSENT`** — a grep for `CSR` over `src-tauri/src/` and `src/` returns nothing |
| **Organic-shale pre-correction** | IP only | **`ABSENT`** |
| **Provenance curve** (`MTH_VSH` equivalent) | Geolog only | **`ABSENT`** |

The combination layer being absent is the structural consequence of having two indicators: with one
GR curve and one N-D curve there is nothing to merge. It becomes load-bearing the moment
SB-CLY-011 through SB-CLY-015 land.

### 3.2 `vsh_gr` — the eight transforms

`PRESENT-DIVERGENT.` Spec `modules.rs:484-528`; implementation `modules.rs:530-571`.

The index is `(g - gr_ma) / (gr_sh - gr_ma)` (`modules.rs:543`) and the eight transforms are
`modules.rs:544-565`, matching Geolog `vsh_gr.lls` L109–139 option-for-option and
coefficient-for-coefficient:

| Option | Code | Line | Against the adopted form |
|---|---|---|---|
| `LINEAR` | `v` | `modules.rs:564` | agrees |
| `STIEBER1` | `v / (3 − 2v)` | `modules.rs:547` | shape `n = 2` — agrees on the equation, diverges on naming |
| `STIEBER2` | `v / (2 − v)` | `modules.rs:551` | shape `n = 1` |
| `STIEBER3` | `v / (4 − 3v)` | `modules.rs:555` | shape `n = 3` |
| `LARINOV1_NORM` | `(2^(2v) − 1)/(2² − 1)` | `modules.rs::vsh_gr` | **exact normalised form** — closes on 1.0 (added 2026-08-22, DEC-096) |
| `LARINOV2_NORM` | `(2^(3.7v) − 1)/(2^3.7 − 1)` | `modules.rs::vsh_gr` | **exact normalised form** — closes on 1.0 |
| `LARINOV1` | `0.33 · (2^(2v) − 1)` | `modules.rs::vsh_gr` | **rounded** — retained under SB-CLY-005 as a labelled parity option |
| `LARINOV2` | `0.083 · (2^(3.7v) − 1)` | `modules.rs::vsh_gr` | **rounded** — same, parity only |
| `LARINOV3` | `0.127 · (3.15^(2v) − 1)` | `modules.rs:559` | **uncited form, implemented** |
| `CLAVIER` | `1.7 − √(3.38 − (v+0.7)²)` | `modules.rs:562` | agrees |

**The earlier audit finding is confirmed against the code, both halves.** ROADMAP §B1
(`ROADMAP.md:925-947`) records the Larionov *labels* as reversed in the manual test plan while the
code is right — and the code is right. `LARINOV1` at `modules.rs:557` is `0.33·(2^(2·IGR) − 1)`,
Larionov (1969) for Mesozoic-and-older; `LARINOV2` at `modules.rs:558` is `0.083·(2^(3.7·IGR) − 1)`,
the Tertiary / unconsolidated form. Miocene deltaic sections fall in the Tertiary / unconsolidated class, so
**`LARINOV2` is the transform they need**. The divergence at mid-range gamma is **0.330 against 0.216 at IGR 0.5** — more
than half again too high through exactly the interval where the VSH cutoff decides net pay, on a
curve that looks entirely normal. The dropdown was made self-describing on 2026-08-01
(`modules.rs:511-518`), the option **ids** were deliberately not renamed because they are stored in
`params_json` on every saved run (`modules.rs:43-53`), and the mapping is pinned by
`every_vsh_gr_transform_lands_on_its_published_coefficient` (`modules.rs:3705-3788`).

**The endpoints are confirmed not to be a clean 0 and 1.** `modules.rs:3756-3769` asserts, at
`IGR = 1`: `LARINOV1` raw **0.99**, `LARINOV2` raw **0.995671**, `LARINOV3` raw **1.133155**;
`LINEAR`, `STIEBER1/2/3` and `CLAVIER` all close at 1.0 to 1e-5. `VSH_GR` (`modules.rs:566`) keeps
the raw value; `VSH` (`modules.rs:567`) clamps it to [0,1]. **This is the F2 defect present in
SandiBumi's own code**, inherited from Geolog: the three Larionov forms are the only transforms in
the family that fail the boundary condition — two low, one high.

**Divergences, itemised.**

1. **Rounded Larionov coefficients** (`modules.rs:557-558`) — 1.00 % and 0.43 % systematically low
   at `I = 1`, where the exact normalised form is vendor-printed (F2). No exact form and no
   parity/exact distinction exists in the code.
2. **No generic Stieber shape parameter.** Three fixed variants only; a study needing `n = 2.5`
   cannot express it, and every vendor label must be mapped by hand.
3. **No Curved method.** The one three-branch transform two independent vendors agree on
   digit-for-digit (dossier §2.2) is not implemented.
4. **`LARINOV3` is implemented without provenance.** `modules.rs:503-505` states the position
   honestly — *"nothing in the repo cites a source for that form, and inventing one is the move the
   provenance rules forbid"* — and the label at `modules.rs:517` gives coefficients rather than a
   rock age. But the transform is still selectable, still overshoots the boundary condition by
   13.3 %, and emits no warning when chosen.
5. **The GR endpoints are uncited product defaults.** `GR_MA = 20.0` (`modules.rs:521`) and
   `GR_SH = 120.0` (`modules.rs:522`) match **no vendor witness in the dossier** — Techlog ships
   10 / 100 gAPI (T1′ + T3), Geolog defers to well constants, IP auto-picks from the curve. At
   `GR = 70 gAPI` the two pairs give `I = 0.5000` against `I = 60/90 = 0.6667`; through `LARINOV2`
   that is **0.2162 against 0.3758 — 73.8 % relative**. The validation ranges `0–200` and `0–1000`
   *do* match Geolog's `vsh_gr.info` L48–49 exactly.
6. **The GR input resolves to `GR`, not a corrected alias** (`modules.rs:523`). Geolog defaults to
   `GR_COR` (F18). SandiBumi's `FAMILIES` folds `GRN` — the normalised curve — into the same `GR`
   family (`curves.rs:22`), so a run cannot express "borehole-corrected only" as a default at all.

### 3.3 Domain clamps — present, hard-coded, and silent

`PRESENT-DIVERGENT` on the bounds; `ABSENT` on the flag. This is the most valuable finding in §3,
because the capability exists and is quietly narrower than it looks.

`modules.rs:546` `limit(v, -10.0, 1.49)`; `:550` `1.99`; `:554` `1.33`; `:561`
`limit(v, -2.53, 1.13)`. These are Geolog's numbers to the digit (T1 `vsh_gr.lls`
L115/L119/L123/L136) — **including Geolog's inward rounding**, which the dossier's §4.2 explicitly
declines to adopt. Three consequences, each checkable:

1. **The Clavier clamp refuses a sliver of its own valid domain and misreports the unlimited
   curve.** The analytic bound is `√3.38 − 0.7 = 1.1384776…`; the code stops at `1.13`. At
   `I = 1.13`: `(1.83)² = 3.3489`, `3.38 − 3.3489 = 0.0311`, `√0.0311 = 0.1763519`,
   `VSH_GR = 1.7 − 0.1764 = **1.5236**`. At the exact bound the radicand is zero and
   `VSH_GR = **1.7000**`. Both clamp to `VSH = 1.000`, so the deliverable curve is unaffected — but
   `VSH_GR` is the *unlimited* twin whose whole purpose is judging whether the endpoints are right,
   and it reads **0.176 v/v (10.4 %) low** at the top of the range. The refused sliver
   `I ∈ (1.13, 1.1385]` is 0.0085 index units; on the Balam South endpoints (`GR_MA = 30`,
   `GR_SH = 150`, T4) that is 1.02 gAPI of real gamma ray.
2. **The Stieber clamps exist only for `n ∈ {1, 2, 3}`.** They are three literals, one per option.
   Adopting the generic shape parameter (SB-CLY-002) makes them structurally unable to serve: there
   is no expression anywhere in the code that derives a bound from `n`.
3. **No clamp emits anything.** `modules.rs:544-565` writes the clamped value straight into the
   transform. A clamped `VSH = 1.000` and a genuine `VSH = 1.000` are indistinguishable in every
   output SandiBumi produces. That is F1's failure class one step removed: SandiBumi does not
   invert the sign the way IP and Techlog do, but it does not tell the interpreter it saved them
   either.

**The degenerate-endpoint guard is present and silent.** `modules.rs:540` skips a sample when
`gr_ma >= gr_sh` (or any input is missing), which is Geolog's `vsh_gr.lls` L99–102 guard — the only
such guard any vendor ships. But `continue` leaves `VSH_GR` and `VSH` at `f32::NAN`
(`modules.rs:533-534`) with **no flag and no message**, so "your endpoints are inverted" is
indistinguishable from "no GR here". Same pattern in `vsh_dn` at `modules.rs:638-640`.

### 3.4 `vsh_dn` — the double indicator

`PRESENT-DIVERGENT.` Spec `modules.rs:577-608`; implementation `modules.rs:610-671`.

The algebra at `modules.rs:629-641` is Geolog `vsh_dn.lls` L134–138 transcribed —
`a = (ρma−ρf)(φNf−φN)`, `b = (ρ−ρf)(φNf−φNma)`, `c = (ρma−ρf)(φNf−φNsh)`,
`d = (ρsh−ρf)(φNf−φNma)`, `VSH = (a−b)/(c−d)`. The dossier proves this reduces to the canonical
cross-product form exactly (§2.7), so the **arithmetic is right**. What diverges is everything
around it.

1. **The clean line is forced through the matrix and fluid points.** There is no `c1`/`c2` pair and
   no constructor — `modules.rs:629-632` reads `RHO_MA` / `RHO_FL` / `NPHI_MA` / `NPHI_FL`
   directly. SandiBumi has adopted the *restricted* parameterisation that Geolog's own
   documentation admits forces users into an *"artificial fluid point"* workaround, and it cannot
   express the light-hydrocarbon clean-line-slope adjustment at all (F15).
2. **No linkage semantics.** With one double indicator there is nothing to link yet; the point is
   that the `c1`-linkable / `c2`-never asymmetry has no structure to attach to when SD and NS land.
3. **Three shipped endpoints match no vendor witness in the dossier, and they move the answer by up
   to 41 %.** This is the sharpest divergence in the chapter.

   | Parameter | SandiBumi | Line | Techlog doc page (T1′) | Techlog template (T3) | Geolog (T1) |
   |---|---|---|---|---|---|
   | `RHO_MA` | 2.645 | `modules.rs:591` | 2.65 | 2.65 | **2645 k/m3 — agrees** |
   | `RHO_SH` | **2.5** | `modules.rs:592` | 2.40 | 2.45 | well constant |
   | `RHO_FL` | 1.0 | `modules.rs:593` | 1.0 | 1 | 1000 k/m3 — agrees |
   | `NPHI_MA` | **−0.02** | `modules.rs:594` | −0.1 | 0 | well constant |
   | `NPHI_SH` | **0.35** | `modules.rs:595` | 0.4 | 0.4 | well constant |
   | `NPHI_FL` | 1.0 | `modules.rs:596` | 1.0 | 1 | 1 v/v — agrees |

   On a realistic shaly sand (`ρb = 2.35 g/cc`, `φN = 0.30 v/v`), evaluated through
   `modules.rs:629-641`:

   - **SandiBumi's shipped defaults** — `a = 1.645×0.70 = 1.1515`, `b = 1.35×1.02 = 1.3770`,
     `c = 1.645×0.65 = 1.06925`, `d = 1.50×1.02 = 1.53`;
     `VSH_DN = (−0.2255)/(−0.46075) = **0.4894**`.
   - **Techlog's shipped template** — `a = 1.65×0.70 = 1.155`, `b = 1.35×1.00 = 1.350`,
     `c = 1.65×0.60 = 0.99`, `d = 1.45×1.00 = 1.45`;
     `VSH_DN = (−0.195)/(−0.46) = **0.4239**`.
   - **Techlog's doc page** — `a = 1.155`, `b = 1.35×1.10 = 1.485`, `c = 0.99`,
     `d = 1.40×1.10 = 1.54`; `VSH_DN = (−0.330)/(−0.55) = **0.6000**`.

   **0.4239 → 0.6000 is 41.5 % relative across one vendor's own two shipped witnesses, and
   SandiBumi's uncited default sits at 0.4894 — 15.5 % above one and 18.4 % below the other.** It is
   not a compromise between them; it is a third number with no source string. Under CONTRACT §2
   these three parameters must ship `ABSENT — ships with no default`.

4. **`VSH_DN_FLAG` is a SandiBumi original with no vendor counterpart, and it has two holes.**
   `modules.rs:650-663` raises the flag when the sample falls off the matrix–shale–fluid triangle
   (`v < −0.05 || v > 1.05`) or — when GR is supplied — when the N-D volume diverges from a
   GR-derived volume by more than `FLAG_TOL` (0.25 v/v, `modules.rs:599`). The rationale at
   `modules.rs:582-589` is sound and clay-type-aware, and **nothing in IP, Geolog or Techlog does
   this.** But (a) the cross-check computes its own GR volume from `GR_MA = 15.0` / `GR_SH = 120.0`
   (`modules.rs:597-598`) — endpoints belonging to *no run the user performed* — so the flag
   compares against a GR Vsh that appears in no output; and (b) when the degenerate-triangle guard
   fires at `modules.rs:638-640`, `flag_out[i]` is never written and stays `NaN`, so the one flag
   curve in the domain cannot report the one condition that makes the module unevaluable.

### 3.5 Four different GR endpoint pairs inside one product

`PRESENT-DIVERGENT.` This is ledger D-13's "the vendor is not internally uniform" defect (F5),
committed by SandiBumi against itself.

| Consumer | `GR_MA` | `GR_SH` | Line |
|---|---|---|---|
| `vsh_gr` | **20** | **120** | `modules.rs:521-522` |
| `vsh_dn` clay-type cross-check | **15** | **120** | `modules.rs:597-598` |
| `ssc` (Sand-Silt-Clay) | **10** | **150** | `ssc.rs:95-96` |
| `gr_normalize` reference pair | 20 | 120 | `modules.rs:2631-2632` |

Three distinct clean endpoints and two shale endpoints, none carrying a source string. At
`GR = 70 gAPI` the linear index reads **0.5000 / 0.5238 / 0.4286** — a **22.2 % relative spread
inside one application**, against the 2.2 % internal spread the dossier holds against IP.
`gr_normalize`'s doc block (`modules.rs:2615-2626`) is exemplary about exactly this — *"NOT a
calibration for any particular field … A reference pair from one basin is the wrong reference in
another"* — and `gr_normalize_reference_defaults_are_generic_not_a_field_calibration`
(`modules.rs:4777`) pins the disclaimer. `vsh_gr`, `vsh_dn` and `ssc` carry no equivalent
statement.

### 3.6 The GR transform ladder exists twice

`PRESENT-DIVERGENT.` `ssc.rs:57-68` is a second, independent copy of the eight-transform match
block at `modules.rs:544-565` — same coefficients, same clamps, same `LINEAR` fallthrough. The two
agree today. There is no test asserting they agree, no shared constant and no import: a coefficient
corrected in one is silently unchanged in the other. This is precisely the "two code paths that
disagree at the third decimal" failure the dossier's canonical-form adoption (§4.5, §5.1) exists to
prevent, already present in the codebase.

### 3.7 The absence model

`PARTIAL` — one undifferentiated mechanism where the domain needs four.

- **Masking exists and is general.** `workflow.rs:557-590` resolves an optional `MASK` curve
  *before* the module runs, so per-run statistics see only unmasked data, and re-applies it to the
  outputs at `workflow.rs:636-640`. `phase7_generic_store_feeds_modules_and_mask`
  (`workflow.rs:2233`) proves a bad-hole flag masks a `vsh_gr` sample to `NaN`.
- **But a masked sample is `NaN`, identical to a missing input.** There is no substitution action,
  no constant action, no recompute-with action, and no token saying *why* the sample is absent.
  F8's finding — that two of three vendors substitute GR and label it — has no expression here.
- **`badhole` produces the discriminator flag** (`modules.rs:1183-1204`) from `|DRHO| > DRHO_MAX`
  (two-sided, `modules.rs:1226`) and `CALI − BS > DCAL_MAX` (**one-sided**, `modules.rs:1231`).
  Under-gauge / closed hole is not detected. F21's two-sided requirement is therefore `PARTIAL` on
  the caliper limb, and the mechanism is a fixed pair of tests rather than the
  `{disc_curve, min, max, action}` rule the domain needs.
- **Coal exists as a detector, not as an indicator branch.** `condflag` raises `COAL_FLAG` from
  `RHOB < COAL_RHOB` (1.9 g/cc), `NPHI > COAL_NPHI` (0.35 v/v) and `DT > COAL_DT` (100 us/ft)
  (`modules.rs:1282-1284`; output `modules.rs:1301`; logic `modules.rs:1376-1379`), and correctly
  refuses to call coal where the hole is washed out (`modules.rs:1379`). There is **no `OPT_COAL` on
  `vsh_gr` or `vsh_dn`** and no branch that sets `VSH = 0` with a `COAL` provenance token.
- **A per-zone per-indicator `Use` flag does not exist.** Parameters are zone-overridable
  (`modules.rs:66-67`), but there is no construct that disables one indicator over one zone.

### 3.8 Vsh ↔ Vcl

`ABSENT` as a bridge; `PRESENT-OK` as a model-internal relationship inside SSC.

There is no `CSR`, no `CLSR` and no conversion of any kind between a shale volume and a clay
volume: a grep for `CSR` / `clay_shale_ratio` over `src-tauri/src/` and `src/` returns nothing.

`ssc` is the one place both quantities exist and are related by construction rather than by a
fitted scalar: `VDCL` (dry clay, `ssc.rs:239` / `:319`), `VWCL = VDCL/(1 − PHIT_CL)` (`ssc.rs:244` /
`:320`) and `VSH_SSC = VWCL + VSILT` (`ssc.rs:245` / `:321`). That is a *structural* bridge — better
than a fitted ratio, and the design SB-CLY-044 should be read against rather than replaced by. It is
confined to one module and is not offered to the indicator layer.

**Neither quantity is typed.** `curves.rs:21-37` — the whole `FAMILIES` dictionary — has **no `VSH`
family, no `VCL` family and no unclipped-Vsh family**. Fifteen families are registered (GR, SP,
CALI, BS, RHOB, DRHO, PEF, NPHI, DT, DTS, RES_DEEP, RES_MED, RES_SHAL, RXO and one more); every Vsh
and Vcl curve SandiBumi computes lands in the catalog **family-less** (`curves.rs:42-45` returns
`None` for an unrecognised mnemonic). Techlog registers four (T3 `A_families.json`). Nothing
structural stops a `VCL` from SSC and a `VSH` from `vsh_gr` being consumed interchangeably by
`thin_bed_ts`, which asks for a `VSH` by mnemonic (`modules.rs:2448`).

### 3.9 Endpoint picking, organic shale, provenance

- **Endpoint picking:** `ABSENT`. `gr_normalize` (`modules.rs:2609-2637`) computes well percentiles
  (`P_LOW = 3`, `P_HIGH = 97`, `modules.rs:2629-2630`) and maps a *curve* onto a reference pair; it
  does not derive an *endpoint parameter* for an indicator, and it is now a thin delegate to the
  universal `normalize` module (`modules.rs:2650-2665`). The house P3/P97 standard is therefore
  present as a curve-normalisation default and absent as a cited endpoint preset. There is no
  pooling group, no pre-percentile clip, no beyond-0–100 % extrapolation and no two-way
  percentile↔value binding. `pickRow` in `histogramPanel.ts:620-621` writes a picked value into a
  *zone parameter*, opted into via Properties (`histogramPanel.ts:64-65`, `:871`) — the gesture
  exists; the derivation rule does not.
- **Organic-shale pre-correction:** `ABSENT`. No kerogen or heavy-mineral correction of GR, NPHI,
  RHOB or DT exists anywhere in the module library.
- **Provenance:** `ABSENT`. No module emits a method-identity curve. `VSH_DN_FLAG`
  (`modules.rs:605`) is the only per-sample explanatory output in the domain, and it is a single
  boolean covering two different conditions.
- **Parameter dialogs:** `PRESENT-OK` for what they are. There is no clay-volume-specific dialog;
  `moduleDialog.ts` renders every module from its manifest, which is why `choice_labels`
  (`modules.rs:46-55`) was the right place to fix the Larionov naming. It also means a runtime
  warning (SB-CLY-006, SB-CLY-014) has no surface today.

### 3.10 Null discipline

`PARTIAL`. LAS export writes `−999.25` with an explicit `NULL.` line (`export.rs:8`,
`export.rs:80`) — correct. On import, `is_null_value` (`parsers.rs:138-140`) honours the file's own
declared `~W NULL` on top of the two standard sentinels `−999.25` and `−9999.0`
(`parsers.rs:130`). So **IP's `−999` bad-hole null is honoured only if the file declares it**, which
is the correct half of the requirement. The missing half: an **undeclared** `−999` imports as data
with no flag and no warning — and IP is the vendor that writes `−999` into double-indicator curves
over bad hole (F8).

---

## 4. Requirements

Fifty-five requirements, `SB-CLY-001` … `SB-CLY-055`, in document order. Fourteen are P0.

The four sources of genuine advantage (CONTRACT §5) map onto this domain as follows, and each
requirement below is traceable to one of them: **vendor defects** — SB-CLY-001, -004, -009, -010,
-016, -021, -022, -027; **vendor disagreement made into the product** — SB-CLY-012, -015, -044,
-050, -053; **fail loud where they fail silent** — SB-CLY-001, -006, -010, -014, -021, -029, -030,
-034, -040, -048; **structural provenance** — SB-CLY-031, -032, -043, -046, -051, -052, -054.

### 4.1 Indicators and transforms

#### SB-CLY-001 — Refuse and flag degenerate endpoints, never null silently&nbsp;&nbsp;&nbsp;[P0] [status: PARTIAL]

**Requirement.** Every clay/shale indicator MUST validate its endpoint pair before evaluation and
MUST distinguish, in its output, an endpoint failure from absent input data. Where the clean and
shale endpoints are equal or inverted (`GR_MA >= GR_SH`, and the equivalent test for each other
indicator), SandiBumi MUST NOT emit a computed value, MUST set the provenance curve of SB-CLY-031
to a distinct `ENDPOINT_INVALID` token, and MUST surface a run-level message naming the parameter
pair, the zone and the offending values. SandiBumi MUST NOT emit a null indistinguishable from
missing input.

**Rationale.** Geolog is the only vendor that guards this at all (T1, `vsh_gr.lls` L99–102); IP and
Techlog divide by a zero or negative span and produce a sign-inverted curve that plots as a normal
log (dossier F1). Inverted endpoints are the single most common data-entry error in the domain and
the one whose consequence — a Vsh that runs backwards — reverses net-pay. Guarding it is table
stakes; *saying so* is the differentiator.

**As-built.** `PARTIAL` — the guard exists at `modules.rs:540` (`gr_ma >= gr_sh`) and
`modules.rs:638-640` (`|c − d| < 1e-6`), both inherited from Geolog, but both `continue`, leaving
`f32::NAN` (`modules.rs:533-534`) with no flag and no message. In `vsh_dn` the degenerate branch
also leaves `VSH_DN_FLAG` at `NaN` (`modules.rs:663` never reached), so the one flag curve cannot
report the one condition that makes the module unevaluable.

**Verified by.** SB-CLY-T01, SB-CLY-T24, SB-CLY-T32

#### SB-CLY-002 — Stieber as one generic shape parameter&nbsp;&nbsp;&nbsp;[P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST implement the Stieber family as the single form
`Vsh = I / (1 + n·(1 − I))` with `n` a user-editable positive real, and MUST provide named presets
for `n = 1`, `n = 2` and `n = 3`. SandiBumi MUST NOT ship three separate hard-coded transforms as
the only way to reach the family.

**Rationale.** All three vendors ship exactly the `n ∈ {1,2,3}` triple and no vendor exposes `n`
(T1 `vsh_gr.lls` L112–124; T1′ Techlog Stieber pages; T1″ IP CHM). The generic form reproduces all
three exactly, admits intermediate shapes a Miocene deltaic section may need, and — critically —
makes the vendor naming collision (F10) expressible as data rather than as a rename.

**As-built.** `PARTIAL` — `modules.rs:546-555` implements the three fixed variants with the correct
algebra; no expression anywhere takes an `n`.

**Verified by.** SB-CLY-T05, SB-CLY-T06, SB-CLY-T07

#### SB-CLY-003 — Resolve vendor Stieber labels by alias, fail the import if unresolvable&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** SandiBumi MUST maintain an explicit alias table mapping each vendor's Stieber
variant label to the `n` it denotes in that vendor's product, and MUST resolve an imported label
through that table. Where an imported label is ambiguous across the products that could have
written it, SandiBumi MUST fail the import with a message naming the label and the candidate `n`
values. SandiBumi MUST NOT guess.

**Rationale.** "Stieber 1", "Stieber 2" and "Stieber 3" denote different `n` in different products
(F10): the same label carries a different equation depending on which application wrote the
parameter file. A silent mis-resolution changes Vsh by up to 20 % at mid-range and is
undetectable downstream. Making the collision an *error* rather than a *default* is the
fail-loud-where-they-fail-silent move.

**As-built.** `ABSENT` — SandiBumi's option ids `STIEBER1/2/3` (`modules.rs:546-555`) carry
SandiBumi's own meaning with no alias layer and no import path that reads a vendor label.

**Verified by.** SB-CLY-T12, SB-CLY-T13

#### SB-CLY-004 — Larionov in the exact normalised form&nbsp;&nbsp;&nbsp;[P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST compute the Larionov transforms as `Vsh = (2^(k·I) − 1)/(2^k − 1)`
with `k = 2` (Mesozoic and older) and `k = 3.7` (Tertiary / unconsolidated), so that `Vsh = 0` at
`I = 0` and `Vsh = 1` at `I = 1` exactly.

**Rationale.** The published decimal coefficients 0.33 and 0.083 are rounded values of
`1/(2²−1) = 0.3333…` and `1/(2^3.7−1) = 0.0833609…` (this line read `0.083346…` until 2026-08-22; §F2 above always carried the correct digits); using them makes the transform miss its own
boundary condition by 1.00 % and 0.43 % low at `I = 1` (dossier F2). Every vendor ships the rounded
form. The exact form is arithmetically identical in intent, correct at the boundary, and free.

**As-built.** `PRESENT-OK` since 2026-08-22 (DEC-096, AUDIT-2026-08-20 finding 27) — `OPT_GR`
carries **`LARINOV1_NORM`** = `(2^(2·IGR) − 1)/(2² − 1)` and **`LARINOV2_NORM`** =
`(2^(3.7·IGR) − 1)/(2^3.7 − 1)`, both closing on exactly 1.0 at `IGR = 1`, in `modules.rs::vsh_gr`
and its verbatim twin `ssc.rs::vsh_from_gr`. The published-decimal pair keeps its own ids and its
own arithmetic under SB-CLY-005 below — a separate id rather than re-pointed arithmetic, because
the id is what `params_json` stores and re-pointing it would move every saved run in silence. The
label mapping is correct and confirmed. Pinned by
`the_exact_larionov_closes_at_one_and_the_published_pair_deliberately_does_not`, which asserts the
exact pair closes AND that the published pair still falls exactly as short as it always did.

**Verified by.** SB-CLY-T02, SB-CLY-T04

#### SB-CLY-005 — Keep the decimal Larionov reachable, for parity only&nbsp;&nbsp;&nbsp;[P2] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST retain the rounded-decimal Larionov coefficients as an explicitly
labelled parity option whose help text states that it reproduces the vendors' published constants
and does not reach 1.0 at `I = 1`. The parity option MUST NOT be the default, and a run using it
MUST record that fact in the provenance curve of SB-CLY-031.

**Rationale.** Reproducing a client's or a vendor's existing curve digit-for-digit is a real
requirement in a competitive replacement; being unable to do so is a lost bake-off. Making it a
named, recorded, non-default choice satisfies both the parity need and the correctness need.

**As-built.** `PRESENT-OK` since 2026-08-22 (DEC-096) — `LARINOV1` and `LARINOV2` keep the
published decimals 0.33 and 0.083 and their exact previous arithmetic, and their dropdown labels now
read *"published 0.33 — parity only, 0.990 at IGR 1"* and *"published 0.083 — parity only, 0.996 at
IGR 1"*. Neither is the default (`OPT_GR` ships `LINEAR`), and the chosen id is recorded in
`params_json` on every saved run — which is where SB-CLY-031's record of it belongs, because DEC-036
already ruled that method identity is a registry-STRUCTURE field, never a per-sample provenance
code. Pinned by
`the_published_larionov_stays_reachable_as_a_labelled_non_default_parity_option`.

**Verified by.** SB-CLY-T03

#### SB-CLY-006 — `LARINOV3` warns that it has no published provenance&nbsp;&nbsp;&nbsp;[P1] [status: PRESENT-DIVERGENT]

**Requirement.** Where SandiBumi offers the `0.127·(3.15^(2·I) − 1)` transform, it MUST emit a
run-level warning stating that no published source for the form is held, and MUST record the choice
in the provenance curve. SandiBumi MUST NOT present the transform alongside the two Larionov forms
in a way that implies equivalent authority, and MUST NOT label it with a rock age.

**Rationale.** The form appears in Geolog's option list (T1) with no citation anywhere in the
install tree, and no literature source is held for it (dossier §1.4). It overshoots the boundary
condition to 1.1332 at `I = 1` — 13.3 % high — which no published Larionov variant does. Under the
project's parameter discipline an uncited transform may be *offered for parity* but must not be
offered *silently*.

**As-built.** `PRESENT-DIVERGENT` — implemented at `modules.rs:559`; the code comment
(`modules.rs:503-505`) states the provenance position correctly and the dropdown label
(`modules.rs:517`) gives coefficients rather than a rock age, which is right. No warning reaches
the user, and the 1.133155 overshoot is pinned as expected (`modules.rs:3761`).

**Verified by.** SB-CLY-T14

#### SB-CLY-007 — Clavier over its full analytic domain&nbsp;&nbsp;&nbsp;[P2] [status: PRESENT-DIVERGENT]

**Requirement.** SandiBumi MUST implement Clavier as `Vsh = 1.7 − √(3.38 − (I + 0.7)²)` and MUST
clamp `I` to the analytically exact domain `[−0.7 − √3.38, √3.38 − 0.7]` computed at runtime, not
to a rounded literal.

**Rationale.** The equation is identical in all three products (T1 `vsh_gr.lls` L136; T1′ Techlog;
T1″ IP). Geolog's shipped clamp of `1.13` is an inward rounding of `1.1384776…` and truncates the
transform's own valid domain.

**As-built.** `PRESENT-DIVERGENT` — the equation is exact at `modules.rs:562`; the clamp is
Geolog's rounded literal at `modules.rs:561` (`limit(v, -2.53, 1.13)`). At `I = 1.13` the unlimited
`VSH_GR` reads `1.7 − √0.0311 = 1.5236` where the exact bound gives `1.7000` — the unlimited QC
twin is **0.176 v/v (10.4 %) low** at the top of its range. The refused sliver is 0.0085 index units
= 1.02 gAPI on `GR_MA = 30` / `GR_SH = 150` (T4).

**Verified by.** SB-CLY-T08, SB-CLY-T09

#### SB-CLY-008 — Implement the Curved transform&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the three-branch Curved transform in the form on which
the two independent vendor witnesses agree digit-for-digit, with the branch boundaries and
coefficients taken from those witnesses and cited.

**Rationale.** Curved is the one non-trivial transform where two vendors' documentation agrees
exactly (dossier §2.2), which makes it the cheapest correctness win in the family — there is no
adjudication to perform. Techlog documents it and does not offer it (dossier F-defect 1), so
implementing it is also a capability the incumbent claims and lacks.

**As-built.** `ABSENT` — `modules.rs:544-565` has no Curved branch.

**Verified by.** SB-CLY-T11

#### SB-CLY-009 — Domain clamps computed from transform parameters&nbsp;&nbsp;&nbsp;[P0] [status: PRESENT-DIVERGENT]

**Requirement.** Every transform whose closed form has a restricted domain MUST derive its clamp
from the transform's own parameters at evaluation time. SandiBumi MUST NOT hard-code a clamp
literal per named variant. Where the exact bound is a pole rather than a finite value, the clamp
MUST be `bound − ε` with `ε` a single named, documented, tested constant.

**Rationale.** Three consequences follow directly from the as-built literals: the Clavier bound is
wrong by rounding (SB-CLY-007), the Stieber clamps cannot survive the generic-`n` adoption
(SB-CLY-002) because no expression derives a bound from `n`, and any new transform arrives with no
clamp at all. The vendors all hard-code; deriving is both more correct and structurally cheaper.

**As-built.** `PRESENT-DIVERGENT` — four literals at `modules.rs:546`, `:550`, `:554`, `:561`,
transcribed from Geolog including its inward rounding. They cover `n ∈ {1,2,3}` and Clavier and
nothing else.

**Verified by.** SB-CLY-T07, SB-CLY-T08

#### SB-CLY-010 — A clamped sample is marked as clamped&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** Where a domain clamp (SB-CLY-009) or the final `[0,1]` limit alters a sample,
SandiBumi MUST record that the sample was clamped, per sample, in a machine-readable output, and
MUST report the clamped fraction per zone in the run record. A clamped `Vsh = 1.000` MUST be
distinguishable from a computed `Vsh = 1.000`.

**Rationale.** A curve pinned at 1.0 over an interval is the single clearest signal that the shale
endpoint is set too low, and every vendor destroys that signal by clamping silently. The unlimited
twin (`VSH_GR`) exposes it only if the interpreter thinks to load and scale a second curve; a
per-sample flag and a per-zone percentage put it in front of them. This is the cheapest
fail-loud-where-they-fail-silent item in the chapter.

**As-built.** `ABSENT` — `modules.rs:544-565` writes clamped values straight through;
`modules.rs:566-567` emits the unlimited/limited pair but no indication that they differ.

**Verified by.** SB-CLY-T10

#### SB-CLY-011 — SP indicator&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide an SP shale indicator as a two-endpoint linear index
between a clean-sand SP value and a shale-baseline SP value, with both endpoints as ordinary
parameters, and MUST NOT apply a transform ladder to it by default.

**Rationale.** All three vendors ship SP (T1 `vsh_sp`; T1′ Techlog; T1″ IP) and all three treat it
as linear only. It remains the most robust indicator in fresh-mud fresh-formation-water sections
where GR is unreliable, which is the case in parts of the target section (T4).

**As-built.** `ABSENT` — no SP indicator; `curves.rs:21-37` registers the `SP` family but nothing
consumes it for shale volume.

**Verified by.** SB-CLY-T15

#### SB-CLY-012 — Three neutron single-indicator forms, no default&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** SandiBumi MUST offer the three vendor forms of the neutron single-curve shale
indicator as separately named, separately cited options, MUST NOT select one as a default, and MUST
name the vendor and artefact each form came from in its help text.

**Rationale.** The three products ship three genuinely different equations for the same nominal
indicator (dossier §2.4), differing by up to **13.1 %** at mid-range — 0.7071 against 0.8000 on a
common case. No adjudication between them is defensible from the evidence held. Under CONTRACT §2
the disagreement itself is the deliverable: shipping all three, named and cited, is more useful to
an interpreter than shipping any one of them silently.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T16

#### SB-CLY-013 — Limestone-matrix precondition on neutron indicators&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** Every indicator that consumes a neutron porosity MUST record the matrix the input
curve is referenced to, MUST refuse to evaluate where that reference is unknown, and MUST refuse
to mix a sandstone-matrix and a limestone-matrix neutron curve within one run without an explicit
conversion step.

**Rationale.** The neutron endpoint values every vendor ships assume a stated matrix reference, and
a sandstone-matrix `NPHI` fed to a limestone-referenced parameter set is a silent, whole-curve
error of the same magnitude as the endpoint disagreements above. Techlog states the precondition on
its documentation page (T1′); nothing in any product enforces it. Refusing beats warning here
because the error is not recoverable downstream.

**As-built.** `ABSENT` — `curves.rs:21-37` types `NPHI` as a family but carries no matrix
attribute; `vsh_dn` (`modules.rs:610-671`) takes `NPHI_MA` as a bare number.

**Verified by.** SB-CLY-T17

#### SB-CLY-014 — Two-sided warning on the neutron clean endpoint&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where a neutron clean-matrix endpoint is set outside the range spanned by the
vendor witnesses held, SandiBumi MUST warn, naming the witnesses and their values. The warning MUST
fire on both sides of the range.

**Rationale.** The vendor witnesses for `φN_ma` span −0.1 to 0 v/v (T1′ doc page against T3 shipped
template — the same vendor). A value outside that span is not necessarily wrong, but it is
necessarily worth stating.

**As-built.** `ABSENT` — `modules.rs:594` ships `NPHI_MA = −0.02`, itself outside neither bound but
matching no witness, with a validation range only.

**Verified by.** SB-CLY-T18

#### SB-CLY-015 — Four resistivity forms, no default&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** SandiBumi MUST offer the resistivity shale indicator as four separately named,
separately cited forms — Gaymard with fixed exponent, Gaymard with user exponent, the IP power
branch, and the log-linear ratio — MUST NOT select a default, and MUST display, for the current
parameter set, the value each form returns for a representative input so the choice is made with
the spread visible.

**Rationale.** This is the largest quantified divergence in the dossier: the four forms return
**0.0816 to 0.4114 on one identical input — a factor of 5.04** (dossier §2.5). Any product that
picks one silently is asserting an adjudication the evidence does not support. Showing the spread
converts the worst inconsistency in the domain into the clearest argument for the product.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T19

#### SB-CLY-016 — Validate `R_clay < R_clean` before branching&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** Every resistivity shale indicator MUST validate that the shale resistivity
endpoint is less than the clean resistivity endpoint **before** selecting its computational branch,
and MUST refuse and report where it is not.

**Rationale.** The resistivity forms are branch-selective on the relative magnitude of their
endpoints; entering the wrong branch produces a smoothly varying, plausible, wholly wrong curve
(dossier §2.5 and F1's failure class). Unlike GR, there is no visual tell.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T20

#### SB-CLY-017 — Cite Coriband where the Coriband form is used&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where SandiBumi implements a form attributable to the Coriband method, the help
text and the provenance record MUST name it as such.

**Rationale.** Attribution is one of the four structural advantages (CONTRACT §5); the incumbents
present inherited forms as house methods. Naming the lineage costs nothing and is checkable.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T33

#### SB-CLY-018 — One canonical bilinear form for every double indicator&nbsp;&nbsp;&nbsp;[P1] [status: PARTIAL]

**Requirement.** Every two-curve (double) shale indicator MUST be evaluated through one shared
implementation of the canonical 2-D cross-product form, parameterised by the clean line and the
shale point. SandiBumi MUST NOT carry a per-indicator algebraic rearrangement.

**Rationale.** The dossier proves the vendors' visibly different printed algebras for N-D, S-D and
N-S all reduce to the same cross-product (dossier §2.7, §5.1). One implementation means one place
to be correct; per-indicator rearrangements are how two code paths come to disagree at the third
decimal. SandiBumi already has that failure mode latent (see SB-CLY-019's as-built).

**As-built.** `PARTIAL` — `modules.rs:629-641` implements Geolog's printed N-D rearrangement
directly. It is provably equivalent to the canonical form, but it is N-D-specific and nothing else
can reuse it.

**Verified by.** SB-CLY-T21

#### SB-CLY-019 — Two-point clean line with an explicit constructor&nbsp;&nbsp;&nbsp;[P1] [status: PRESENT-DIVERGENT]

**Requirement.** Every double indicator MUST express its clean line as two named points `c1` and
`c2`, and MUST provide a constructor that derives them from a matrix point and a fluid point for
users who want the restricted parameterisation. SandiBumi MUST NOT force the clean line through the
matrix and fluid points as the only available form.

**Rationale.** The restricted parameterisation cannot express a clean line whose slope differs from
the matrix–fluid slope, which is exactly the light-hydrocarbon case (dossier F15). Geolog's own
documentation concedes the workaround is an *"artificial fluid point"* — the vendor is on record
that its parameterisation is inadequate.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:629-632` reads `RHO_MA`/`RHO_FL`/`NPHI_MA`/`NPHI_FL`
directly; there is no `c1`/`c2` pair and no constructor.

**Verified by.** SB-CLY-T22

#### SB-CLY-020 — Linkage semantics: `c1` linkable, `c2` never; doubles are not singles&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where two double indicators share a clean-line point, SandiBumi MUST allow `c1` to
be linked across indicators and MUST NOT allow `c2` to be linked. Double-indicator endpoint
parameters MUST be held separately from single-indicator endpoint parameters of the same nominal
quantity, and MUST NOT be silently shared.

**Rationale.** `c1` is the shared matrix-side anchor; `c2` is per-crossplot geometry and linking it
couples two independent calibrations. The single/double separation is the mechanism by which one
vendor ends up with two different `ρ_sh` values in one project (dossier §3, ledger D-13).

**As-built.** `ABSENT` — with one double indicator there is nothing to link, and no structure
exists to attach the rule to.

**Verified by.** SB-CLY-T23

#### SB-CLY-021 — Degenerate crossplot geometry is refused and reported&nbsp;&nbsp;&nbsp;[P0] [status: PARTIAL]

**Requirement.** Every double indicator MUST detect the degenerate geometry in which the shale
point lies on the clean line — the case in which the canonical denominator vanishes — MUST refuse
to evaluate, and MUST report the condition naming the three points. The refusal MUST be
distinguishable from missing input in the provenance curve, and MUST be written to the indicator's
flag curve rather than left unset.

**Rationale.** The denominator `c − d` vanishing means the crossplot has no clay direction; a
near-vanishing denominator amplifies input noise without bound. Geolog guards it (T1
`vsh_dn.lls`); the guard is worth keeping and worth explaining.

**As-built.** `PARTIAL` — `modules.rs:638-640` tests `(c − d).abs() < 1e-6` and `continue`s, which
is the right test with the wrong consequence: `VSH_DN`, `VSH` and `VSH_DN_FLAG` are all left `NaN`
(`modules.rs:614-616`), so the module's own diagnostic curve is silent about the one condition it
most needs to report.

**Verified by.** SB-CLY-T24, SB-CLY-T32

#### SB-CLY-022 — Refuse the printed sonic-density denominator&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** SandiBumi MUST implement the sonic-density double indicator from the canonical
cross-product form of SB-CLY-018. SandiBumi MUST NOT transcribe the sonic-density denominator as
printed on the vendor documentation page, which carries a sign defect. The defect and its
disposition MUST be recorded in the implementation's source comment.

**Rationale.** The printed form contains a double-minus that inverts the denominator's sign
(dossier §2.8a); a faithful transcription would ship the vendor's own typographic defect. This is
the clearest case in the chapter where "match the vendor" and "be correct" diverge, and the
canonical form resolves it without adjudicating anything.

**As-built.** `ABSENT` — no sonic-density indicator exists.

**Verified by.** SB-CLY-T25

#### SB-CLY-023 — Thorium and Potassium indicators&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** SandiBumi MUST provide Thorium and Potassium shale indicators as two-endpoint
linear indices over the respective spectral-gamma curves, with endpoints as ordinary parameters.

**Rationale.** One vendor ships both (T1′ Techlog Quanti Shale Volume); neither of the others does.
They are the correct indicator where a uranium-bearing organic-rich interval defeats total GR — a
condition present in parts of the target section (T4) and the same condition SB-CLY-047 addresses
from the other direction.

**As-built.** `ABSENT` — no spectral-gamma family is registered in `curves.rs:21-37`.

**Verified by.** SB-CLY-T26

#### SB-CLY-024 — EM-propagation indicator, parameter named once&nbsp;&nbsp;&nbsp;[P4] [status: ABSENT]

**Requirement.** Where SandiBumi implements the EM-propagation (`TPL`) shale indicator, the matrix
travel-time parameter MUST appear exactly once in the interface and in the parameter record.

**Rationale.** One vendor ships this indicator and names the same parameter twice on its own
documentation page, under two different names (dossier F-defect 5). A user cannot tell whether the
two are independent. Naming it once is the fix and the differentiator.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T27

#### SB-CLY-025 — M–N crossplot Vsh is deliberately not implemented&nbsp;&nbsp;&nbsp;[P4] [status: ABSENT]

**Requirement.** SandiBumi MUST NOT implement an M–N crossplot shale volume until the discrepancy
recorded as Escalation E1 is resolved against a primary source. The decision and its reason MUST be
recorded in the module library's documentation so that the absence reads as a choice, not an
oversight.

**Rationale.** The only vendor shipping it (T1 `vsh_mn`) has a documentation constant of `0.308`
against a code constant of `0.388` — a transposition somewhere, and the evidence held does not say
which is right. Implementing either without resolving it would be inventing a petrophysical
parameter, which the project's standing discipline forbids. Three products offer M–N; being the
one that declines *and says why* is defensible, and guessing is not.

**As-built.** `ABSENT` — correctly, and currently by omission rather than by decision.

**Verified by.** SB-CLY-T33

#### SB-CLY-026 — NMR clay volume is typed as a clay volume&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** Where SandiBumi computes a clay volume from NMR bound water, the output MUST be
typed as a clay volume (`VCL`), MUST NOT be aliased to a shale volume, and MUST carry a distinct
provenance token. Its parameter and output naming MUST follow the same conventions as the classical
indicators.

**Rationale.** The one vendor that ships it (T1 `vsh_nmr.lls` L54,
`VCL_NMR = limit(VOL_CBW_NMR/PHIT_NMR, 0, 1)`) is also the one place that vendor breaks all three of
its own family conventions — the unclipped/clipped output pair, the coal option and the method-code
provenance are all absent there. NMR bound water measures clay, not shale, and the distinction is
the whole point of SB-CLY-043.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T28, SB-CLY-T43

### 4.2 Combination, absence and the discriminator layer

#### SB-CLY-027 — Clip each indicator before combining, never after&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** The combination layer MUST clip every contributing indicator to `[0,1]` before
applying the combiner, and MUST NOT clip only the combined result. The clip order MUST be stated in
the run record.

**Rationale.** Clip order changes the answer, and by amounts that matter. On one vendor-realistic
pair the two orders give **0.500 against 0.600 — 20 %**; on another, **0.200 against 0.125 — 60 %**
(dossier §2.11). Clip-then-combine is the order under which a combiner's bound guarantees actually
hold; combine-then-clip lets an out-of-range indicator drag a mean it should not have entered.
Neither of the two vendors that combine states its order in documentation.

**As-built.** `ABSENT` — there is no combination layer (`modules.rs:342-394` registers two
indicators and no combiner).

**Verified by.** SB-CLY-T29

#### SB-CLY-028 — Only bounded-safe combiners&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide minimum, arithmetic mean, median and the Lateral
pseudomedian as combiners. Every combiner offered MUST be one whose output is bounded by the range
of its clipped inputs. SandiBumi MUST NOT offer a combiner that can return a value outside that
range.

**Rationale.** All four are shipped by at least one vendor and all four are bound-preserving on
clipped inputs (dossier §2.10, §2.10a). A combiner that can exceed its inputs re-opens the clamping
problem after the clamps have been applied.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T30

#### SB-CLY-029 — A zero is a value, not an absence&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** The combination layer MUST treat a contributing indicator value of exactly zero as
a valid observation and MUST include it in the combiner. SandiBumi MUST NOT use zero to mean
"indicator not available".

**Rationale.** `Vsh = 0` is the single most common correct answer in a clean sand — the reservoir
interval, precisely where the arithmetic matters most. A combiner that discards zeros biases every
clean interval upward, and does so invisibly because the result is still in range. The dossier
records this as a live risk in one vendor's merge implementation.

**As-built.** `ABSENT` — no combination layer; but the underlying convention is right, because
absence is `f32::NAN` throughout (`modules.rs:533-534`, `is_missing`), not zero.

**Verified by.** SB-CLY-T31

#### SB-CLY-030 — Three distinct absences, distinguishable in the output&nbsp;&nbsp;&nbsp;[P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST distinguish, per sample and in machine-readable form, at least
three reasons a clay/shale volume is absent: input curve missing; sample masked or rejected by a
discriminator; indicator refused (endpoint or geometry invalid). Downstream consumers MUST be able
to act on the distinction.

**Rationale.** Every vendor collapses all three into one null. The interpreter's next action is
completely different in each case — find the curve, widen the discriminator, fix the endpoints —
and the product currently makes them guess. This is the highest-leverage
fail-loud-where-they-fail-silent item after SB-CLY-010.

**As-built.** `PARTIAL` — masking is general and works (`workflow.rs:557-590`, re-applied at
`workflow.rs:636-640`, proven by `workflow.rs:2233`), but a masked sample is `NaN`, identical to a
missing input and identical to a refused evaluation.

**Verified by.** SB-CLY-T32

#### SB-CLY-031 — Every clay/shale volume carries a provenance curve&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** Every module emitting a shale or clay volume MUST also emit a per-sample
provenance curve recording which indicator and which transform produced that sample, or why no
value was produced. The provenance curve MUST be exported alongside the volume and MUST survive a
LAS round trip.

**Rationale.** One vendor ships a per-sample method code (T1, `MTH_VSH`) and is the only product in
which one can answer "which indicator won at this depth" after the fact — and even there the coding
is `ALPHA*8`, opaque and undocumented outside the source. Six months later, on a deliverable being
audited, the provenance curve is the difference between defending a number and re-running the
study. Provenance is structural here and conventional there (CONTRACT §5).

**As-built.** `ABSENT` — no module emits a method-identity curve. `VSH_DN_FLAG`
(`modules.rs:605`, logic `modules.rs:650-663`) is the only per-sample explanatory output in the
domain and it is a single boolean covering two unrelated conditions.

**Verified by.** SB-CLY-T33, SB-CLY-T34

#### SB-CLY-032 — One closed provenance vocabulary, substitution recorded separately&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** The provenance curve MUST draw on a single closed, documented vocabulary of
tokens. Where a sample's inputs were substituted (SB-CLY-035), the substitution MUST be recorded in
a **separate** field, not by overloading the method token.

**Rationale.** The one vendor precedent packs method identity into an opaque eight-character alpha
code; overloading it further with substitution state is how a provenance record stops being
machine-readable. A closed vocabulary is also what makes SB-CLY-030's three-way distinction
testable.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T34

#### SB-CLY-033 — Per-flag override generality&nbsp;&nbsp;&nbsp;[P2] [status: PARTIAL]

**Requirement.** The absence model MUST be expressed as a list of rules of the form
`{discriminator curve, minimum, maximum, action}`, evaluable per indicator and per zone.
SandiBumi MUST NOT hard-code a fixed set of discriminator tests as the only available mechanism.

**Rationale.** The vendors' bad-hole handling is a fixed pair of caliper/DRHO tests; the real
condition varies by hole, by tool string and by interval. A rule list subsumes bad hole, coal, casing
and any client-specific reject condition without new code.

**As-built.** `PARTIAL` — `badhole` (`modules.rs:1183-1204`, logic `modules.rs:1222-1240`) and
`condflag` (`modules.rs:1282-1284`, `:1376-1379`) are exactly the hard-coded form this requirement
replaces; the generic `MASK` opt (`workflow.rs:557-590`) is the consumption half and is already
general.

**Verified by.** SB-CLY-T36

#### SB-CLY-034 — No magic sentinel for a rejected sample&nbsp;&nbsp;&nbsp;[P0] [status: PARTIAL]

**Requirement.** SandiBumi MUST NOT write a numeric sentinel into a curve to mean "rejected". A
rejected sample MUST be absent and MUST be explained by the provenance curve. On import, a value
equal to a known vendor sentinel MUST be treated as absent and MUST raise a warning naming the
sentinel, **whether or not** the file declares it.

**Rationale.** One vendor writes `−999` into double-indicator curves over bad hole (dossier F8). If
the file declares it, everything downstream is fine; if it does not, `−999` imports as a valid
shale volume of minus nine hundred and ninety-nine and propagates into every statistic. The current
half-measure catches the good case and misses the dangerous one.

**As-built.** `PARTIAL` — `parsers.rs:130` recognises `−999.25` and `−9999.0` unconditionally and
`parsers.rs:138-140` honours the file's declared null, but a bare undeclared `−999` is imported as
data with no flag. Export is correct: `export.rs:8` and `export.rs:80` write and declare `−999.25`.

**Verified by.** SB-CLY-T35, SB-CLY-T44

#### SB-CLY-035 — Discriminator tests are two-sided by default&nbsp;&nbsp;&nbsp;[P2] [status: PARTIAL]

**Requirement.** A discriminator rule MUST support both a minimum and a maximum. Caliper-based
bad-hole detection MUST test both over-gauge and under-gauge conditions.

**Rationale.** Under-gauge hole — swelling shale, keyseat, a differentially stuck interval — is a
real data-quality condition and a one-sided test cannot see it. The vendors' caliper tests are
over-gauge only, and SandiBumi inherited that.

**As-built.** `PARTIAL` — `modules.rs:1226` tests `|DRHO| > DRHO_MAX` two-sided, which is right;
`modules.rs:1231` tests `CALI − BS > DCAL_MAX` **one-sided**, which is not.

**Verified by.** SB-CLY-T36

#### SB-CLY-036 — Per-indicator coal branch with its own provenance token&nbsp;&nbsp;&nbsp;[P2] [status: PARTIAL]

**Requirement.** Every indicator MUST offer an optional coal branch that, where the coal
discriminator is satisfied, sets the shale volume to zero and records a distinct `COAL` provenance
token. The option MUST default to off, and the discriminator MUST NOT fire where the hole is
rejected as bad.

**Rationale.** One vendor carries the option on eight of its nine classical indicators and defaults
it off (T1, `OPT_COAL`); the others have no coal logic in the clay module at all. A coal seam reads
as high-GR shale to every indicator in the family, and zeroing it silently is worse than not
zeroing it — hence the provenance token.

**As-built.** `PARTIAL` — the *detector* exists and is well built: `condflag` raises `COAL_FLAG`
(`modules.rs:1301`) from `RHOB < 1.9`, `NPHI > 0.35`, `DT > 100` (`modules.rs:1282-1284`, logic
`modules.rs:1376-1379`) and correctly declines to call coal where the hole is washed out
(`modules.rs:1379`). Neither `vsh_gr` nor `vsh_dn` has an `OPT_COAL` or a zeroing branch.

**Verified by.** SB-CLY-T37

### 4.3 Endpoint picking

#### SB-CLY-037 — A complete percentile endpoint pipeline&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** SandiBumi MUST provide percentile-based endpoint picking with, at minimum: a
selectable pooling group (well, zone, or a named set of wells); an optional pre-percentile clip
applied before the statistic; a percentile pair for the clean and shale endpoints; and support for
percentile values outside 0–100 as an explicit extrapolation with a warning.

**Rationale.** One vendor ships the full machinery and it is the single largest capability gap in
this chapter measured by user time. Endpoint picking is where a shale-volume study actually gets
done; every hour of it is currently manual.

**As-built.** `ABSENT` — `gr_normalize` (`modules.rs:2609-2637`, delegate `modules.rs:2650-2665`)
computes well percentiles `P_LOW = 3` / `P_HIGH = 97` (`modules.rs:2629-2630`) and maps a *curve*
onto a reference pair. It normalises; it does not derive an endpoint parameter for an indicator.
`histogramPanel.ts:620-621` writes a manually picked value into a zone parameter
(`histogramPanel.ts:64-65`, `:871`) — the gesture exists, the derivation rule does not.

**Verified by.** SB-CLY-T38

#### SB-CLY-038 — Two-way binding between percentile and value&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where an endpoint is derived from a percentile, editing the percentile MUST update
the value and editing the value MUST update the displayed percentile. The record MUST state which
of the two the user set.

**Rationale.** Interpreters work in both directions — "what percentile is 30 gAPI here?" is as
common as "what is P5?". Recording which was authoritative is what makes the endpoint reproducible
when the well is re-run with more data.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T39

#### SB-CLY-039 — The P3/P97 house preset is a cited, recorded preset&nbsp;&nbsp;&nbsp;[P2] [status: PARTIAL]

**Requirement.** SandiBumi MAY ship a P3/P97 percentile pair as a named preset for GR endpoint
picking. Where it does, the preset MUST carry its source string, MUST NOT be presented as a
physical constant, and the run record MUST record that the preset was used and what values it
produced in that well.

**Rationale.** P3/P97 is a documented house standard (T4) and therefore has a legitimate source
string — which is exactly what distinguishes a citable preset from an invented default. The
distinction must be visible in the interface, because a percentile preset that looks like a
petrophysical constant is how a basin-specific convention silently crosses a basin boundary.

**As-built.** `PARTIAL` — the values exist as `gr_normalize` defaults (`modules.rs:2629-2630`) and
the module's doc block is exemplary about the danger (`modules.rs:2615-2626`: *"NOT a calibration for
any particular field"*), pinned by `gr_normalize_reference_defaults_are_generic_not_a_field_calibration`
(`modules.rs:4777`). They are not available as an endpoint preset for an indicator.

**Verified by.** SB-CLY-T38

#### SB-CLY-040 — Warn where a percentile endpoint lands near a transform pole&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where a percentile-derived endpoint places a significant fraction of the interval
inside a transform's clamped region or within a stated tolerance of its pole, SandiBumi MUST warn,
reporting the affected fraction.

**Rationale.** The interaction is invisible in both directions: the percentile pick looks
reasonable and the transform output looks reasonable, but the curve is pinned. It is the same
signal SB-CLY-010 preserves, surfaced at the moment the endpoint is chosen rather than after.

**As-built.** `ABSENT` — and there is currently no surface for a runtime warning, because
`moduleDialog.ts` renders every module generically from its manifest.

**Verified by.** SB-CLY-T10, SB-CLY-T40

#### SB-CLY-041 — Prefer the corrected input alias, uniformly&nbsp;&nbsp;&nbsp;[P2] [status: PRESENT-DIVERGENT]

**Requirement.** Every indicator MUST express its input preference as an ordered alias list that
prefers an environmentally corrected curve over a raw one, and the same ordering discipline MUST
apply across all indicators in the domain. Which curve actually resolved MUST appear in the run
record.

**Rationale.** One vendor defaults its GR indicator to the corrected mnemonic (T1, `GR_COR`); the
others take whatever `GR` resolves to. Silently consuming a raw curve where a corrected one exists
is a systematic bias with no visible symptom.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:523` binds the literal mnemonic `GR`, and
`curves.rs:22` folds `GRN` (the *normalised* curve) into the same `GR` family, so "corrected only"
cannot be expressed as a preference at all. The alias mechanism exists; the ordering does not.

**Verified by.** SB-CLY-T43

#### SB-CLY-042 — Picking conventions stated as help text, not encoded as defaults&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** Where a documented convention exists for picking an endpoint — a percentile
habit, a crossplot construction, a preferred interval — SandiBumi MUST surface it as help text
attached to the parameter, with its source. SandiBumi MUST NOT convert a picking convention into a
numeric default.

**Rationale.** A convention is advice about *how to pick*; a default is a *value*. Encoding the
first as the second is precisely how uncited numbers enter a product — and is, mechanically, how
each of the four different GR endpoint pairs in §3.5 got there.

**As-built.** `ABSENT` — parameter descriptions exist (`modules.rs:521-522`) but carry no source
strings and no picking guidance.

**Verified by.** SB-CLY-T33

### 4.4 Vsh, Vcl and the bridge between them

#### SB-CLY-043 — Shale volume and clay volume are distinct typed quantities&nbsp;&nbsp;&nbsp;[P0] [status: ABSENT]

**Requirement.** SandiBumi MUST type shale volume and clay volume as distinct quantities. A module
requiring one MUST NOT silently accept the other. Where a consumer can accept either, it MUST state
which it received in its run record.

**Rationale.** Shale is clay plus silt plus bound water plus whatever else the shale laminae
contain; clay is the mineral fraction. Substituting one for the other biases every saturation and
permeability model downstream, in the direction that overstates pay. Only one vendor even names
both, and none of the three enforces the distinction at the interface.

**As-built.** `ABSENT` — `curves.rs:21-37` registers fifteen families and **neither `VSH` nor
`VCL` is among them**, so `family_for` (`curves.rs:42-45`) returns `None` for every clay-volume
curve the product computes. `thin_bed_ts` consumes a curve named `VSH` by mnemonic
(`modules.rs:2448`) with nothing preventing a `VCL` from being supplied under that name.

**Verified by.** SB-CLY-T43, SB-CLY-T28

#### SB-CLY-044 — Both bridges named; no default ratio&nbsp;&nbsp;&nbsp;[P1] [status: PARTIAL]

**Requirement.** SandiBumi MUST provide the clay/shale-ratio bridge in both directions, with the
ratio as an explicit named parameter, and MUST NOT ship a numeric default for it. Where a
structural bridge is available — a model that computes both volumes from its own components — that
bridge MUST be preferred and the run record MUST say which was used.

**Rationale.** One vendor ships a clay/shale ratio parameter (T1″, `CSR`) with a product default;
that default is a lithology-dependent quantity with no universal value, and adopting it would be
importing an uncited constant. The magnitude at stake: on `Vsh = 0.5`, `φ_sh = 0.15`, ratio 0.6, the
two vendor treatments give **0.300 against 0.210 — 30 %** (dossier §2.13).

**As-built.** `PARTIAL` — no ratio bridge exists anywhere (a grep for `CSR` / `clay_shale_ratio`
over `src-tauri/src/` and `src/` returns nothing), but the **structural** bridge this requirement
prefers is already built inside SSC: `VDCL` (`ssc.rs:239`, `:319`),
`VWCL = VDCL/(1 − PHIT_CL)` (`ssc.rs:244`, `:320`) and `VSH_SSC = VWCL + VSILT` (`ssc.rs:245`,
`:321`). It is model-internal and not offered to the indicator layer.

**Verified by.** SB-CLY-T41

#### SB-CLY-045 — Endpoint conversion identities are explicit and tested&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where an endpoint expressed against one quantity is used by a module expressed
against the other, the conversion MUST be an explicit, named, tested identity. SandiBumi MUST NOT
reuse an endpoint across the Vsh/Vcl boundary unconverted.

**Rationale.** The shale point and the clay point are different points on a crossplot; using one
for the other is the same error class as SB-CLY-043 one level down, and it is invisible because
both are dimensionless volumes in [0,1].

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T42

#### SB-CLY-046 — Register the Vsh/Vcl curve families&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** The curve family dictionary MUST register distinct families for clipped shale
volume, unclipped shale volume, clay volume and the domain's flag/provenance curves, with the
mnemonic aliases each vendor writes. A clay or shale curve MUST NOT resolve to no family.

**Rationale.** This is the mechanism SB-CLY-043 needs and the precondition for import from any
competitor's project. One vendor registers four such families (T3). Without it, every Vsh curve
SandiBumi produces is untyped, plots without a default scale, and can be silently substituted for
any other.

**As-built.** `ABSENT` — `curves.rs:21-37` registers GR (with `GRN` folded in at `curves.rs:22`),
SP, CALI, BS, RHOB, DRHO, PEF, NPHI, DT, DTS, RES_DEEP, RES_MED, RES_SHAL and RXO, and nothing for
this domain, despite `vsh_gr` and `vsh_dn` emitting four curve names (`modules.rs:569`,
`modules.rs:603-605`).

**Verified by.** SB-CLY-T43

#### SB-CLY-047 — Organic-shale pre-correction in renormalised form&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** Where SandiBumi corrects an indicator input for kerogen or heavy minerals, the
correction MUST be applied as a renormalisation of the indicator over the non-organic fraction, and
the corrected input MUST be emitted as a curve rather than consumed invisibly.

**Rationale.** One vendor ships the pre-correction (T1″); the difference against the uncorrected
index on a realistic organic interval is **0.2999 against 0.2309 — 23 %** (dossier §2.14). Emitting
the corrected curve is what lets the interpreter see how much of the answer is the correction.

**As-built.** `ABSENT` — no kerogen or heavy-mineral correction exists in the module library.

**Verified by.** SB-CLY-T33

#### SB-CLY-048 — Guard the renormalisation denominator&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** The organic-shale renormalisation MUST guard its denominator against the case in
which the corrected fractions sum to one or more, MUST refuse rather than clamp, and MUST report
the condition.

**Rationale.** The denominator is `1 − Vker − Vhvy`; as it approaches zero the corrected index
diverges. A clamp would return a plausible number from an impossible input.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T33

#### SB-CLY-049 — Do not iterate kerogen and heavy-mineral volumes inside the indicator&nbsp;&nbsp;&nbsp;[P3] [status: ABSENT]

**Requirement.** SandiBumi MUST take the kerogen and heavy-mineral volumes as inputs to the
pre-correction and MUST NOT solve for them within the clay-volume module. Where they are unknown,
the module MUST refuse rather than assume.

**Rationale.** Solving for them here duplicates — and would silently diverge from — the mineral
solver's job (`MIN`), and the evidence held does not establish whether the vendor iterates
(dossier OPEN 16, carried to §7). A clean seam is the defensible position while the question is
open.

**As-built.** `ABSENT`.

**Verified by.** SB-CLY-T33

### 4.5 Parameter provenance and data discipline

#### SB-CLY-050 — Where the vendors disagree, ship no default and surface the conflict&nbsp;&nbsp;&nbsp;[P0] [status: PRESENT-DIVERGENT]

**Requirement.** Where the evidence held records more than one value for a petrophysical parameter
and no adjudication is defensible, SandiBumi MUST ship that parameter with no default, MUST refuse
to evaluate until it is set, and MUST present the competing values with their sources at the point
of entry. SandiBumi MUST NOT interpolate, average, or select silently between them.

**Rationale.** This is the standing project discipline (CONTRACT §2) and, in this domain, the
single largest correctness item. Three of `vsh_dn`'s six shipped endpoints match no vendor witness
in the dossier, and the vendor witnesses themselves disagree: on `ρb = 2.35 g/cc`, `φN = 0.30 v/v`,
one vendor's shipped template gives `VSH_DN = 0.4239` and the same vendor's documentation page
gives `0.6000` — **41.5 % relative** — while SandiBumi's uncited default returns `0.4894`, sitting
15.5 % above one and 18.4 % below the other. It is not a compromise; it is a third number with no
source string.

**As-built.** `PRESENT-DIVERGENT` — `RHO_SH = 2.5` (`modules.rs:592`), `NPHI_MA = −0.02`
(`modules.rs:594`) and `NPHI_SH = 0.35` (`modules.rs:595`) are uncited; `GR_MA = 20` /
`GR_SH = 120` (`modules.rs:521-522`) are uncited and worth **73.8 %** through `LARINOV2` against the
one vendor witness held (10 / 100 gAPI). The three parameters that *do* match a vendor witness —
`RHO_MA = 2.645` (`modules.rs:591`), `RHO_FL = 1.0` (`modules.rs:593`), `NPHI_FL = 1.0`
(`modules.rs:596`) — agree with Geolog exactly, so the discipline is half-applied rather than
absent.

**Verified by.** SB-CLY-T18, SB-CLY-T19, SB-CLY-T20

#### SB-CLY-051 — The vendor artefact path is the primary source string&nbsp;&nbsp;&nbsp;[P1] [status: ABSENT]

**Requirement.** Every default that ships MUST carry a source string identifying a specific
checkable artefact — a file path and locator within a vendor install, a named publication, or a
named project record. A vendor's product name alone MUST NOT be accepted as a source.

**Rationale.** CONTRACT §2 makes this binding, and this domain shows why it is not pedantry: the
same vendor ships **different numbers for the same parameter** in its documentation and in its
templates (dossier F-defect 4 — `Δt_ma` 50 against 55.5, `φN_ma` −0.1 against 0, `ρ_sh` 2.40
against 2.45). "The vendor's value" is not a well-defined quantity for that vendor. Only the
artefact is.

**As-built.** `ABSENT` — no `ArgSpec` field carries a source string (`modules.rs:46-55`), so no
parameter in the domain has a machine-readable provenance.

**Verified by.** SB-CLY-T33

#### SB-CLY-052 — Import by ordinal **and** semantic key&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Where SandiBumi imports a vendor parameter set, it MUST match each parameter on
both its positional ordinal and a semantic key, and MUST refuse the import where the two disagree.

**Rationale.** Vendor parameter files are positional; a version change that inserts a parameter
shifts every subsequent value into the wrong slot while every value remains individually plausible.
Matching on one key alone cannot detect it. This is the same defect class as the mudlog
column-shift trap (T4) and it is caught the same way.

**As-built.** `ABSENT` — no vendor parameter import path exists.

**Verified by.** SB-CLY-T33

#### SB-CLY-053 — Matrix travel time is module-scoped and carries its artefact&nbsp;&nbsp;&nbsp;[P2] [status: ABSENT]

**Requirement.** Matrix travel time MUST be scoped to the module that uses it, MUST NOT be shared
implicitly across modules, and MUST carry the artefact its value came from. Where two artefacts of
the same vendor disagree, the parameter MUST ship with no default.

**Rationale.** `Δt_ma` has four witnesses across the products held, spanning **23.6 % relative** in
the resulting index — and **20.8 % of that spread is inside one vendor**, between its documentation
page and its shipped template (dossier §3, F-defect 4). A shared global `Δt_ma` would silently
propagate one artefact's number into a module calibrated against another's.

**As-built.** `ABSENT` — no sonic-bearing clay indicator exists, so the parameter does not appear
in this domain yet; the requirement binds when SB-CLY-022 lands.

**Verified by.** SB-CLY-T25

#### SB-CLY-054 — Unit-typed quantities; no magic scale constants&nbsp;&nbsp;&nbsp;[P0] [status: PARTIAL]

**Requirement.** Every petrophysical quantity in the domain MUST carry its unit in the module
manifest, and conversions MUST be performed by named, tested identities. SandiBumi MUST NOT embed
an unexplained scale factor in an equation. A parameter transcribed from a vendor artefact MUST be
recorded in the artefact's unit and converted explicitly.

**Rationale.** Vendors disagree not only on values but on units for the same quantity — one ships
density in k/m3 where the others use g/cc, and porosity as v/v against percent. A transcription
that adjusts the number to fit the house unit without recording the conversion loses the ability to
check it against the source, which is the whole purpose of SB-CLY-051.

**As-built.** `PARTIAL` — `ArgSpec` carries a unit string and it is used consistently in the
domain (`modules.rs:521` `"gapi"`, `modules.rs:591-596` `"g/cc"` and `"v/v"`), and the house density
convention is g/cc throughout. There is no conversion layer and no record of the unit a value was
transcribed in — `RHO_MA = 2.645` (`modules.rs:591`) is Geolog's `2645 k/m3` silently divided by
1000.

**Verified by.** SB-CLY-T21, SB-CLY-T42

#### SB-CLY-055 — LAS null discipline on every domain curve&nbsp;&nbsp;&nbsp;[P1] [status: PARTIAL]

**Requirement.** Every curve this domain emits — volumes, flags and provenance — MUST round-trip
through LAS export and re-import with its absences preserved and its provenance intact. The
declared null MUST be written in the header, and provenance tokens MUST survive as a curve.

**Rationale.** The provenance curve (SB-CLY-031) is worthless if it does not survive the format the
deliverable ships in. A round-trip test is also the cheapest guard against the sentinel problem of
SB-CLY-034.

**As-built.** `PARTIAL` — export writes and declares `−999.25` correctly (`export.rs:8`,
`export.rs:80`) and import honours declared nulls plus two standard sentinels
(`parsers.rs:130`, `parsers.rs:138-140`). There is no provenance curve to round-trip and no
round-trip test for this domain's outputs.

**Verified by.** SB-CLY-T35, SB-CLY-T44

---

## 5. Parameters

Fifty-eight rows, transcribed from the dossier §5.2 with values byte-exact and source strings
carried unchanged. **Fifteen rows ship `ABSENT — ships with no default`** — either because more
than one vendor value exists and no adjudication is defensible, or because no vendor states one —
and one ships `NON-ADOPTABLE — cited for verification`. "No default" is a first-class state, not a
gap (dossier §5.4 rule 9), and SB-CLY-050 makes refusing to evaluate without them a requirement.

Three rows are marked ⚠ **TWO VENDOR VALUES**: a *single* vendor ships two conflicting numbers for
the same parameter, one on its documentation page and one in its shipped templates. Both are
carried with both sources; neither may become a preset without the conflict being shown to the user.

`ρ` is transcribed in the unit each artefact states it in. Where a source states k/m3, that is what
appears below; the conversion to the house g/cc is SB-CLY-054's explicit identity, not a silent
division.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| `stieber.n` | n | 2.0 | — | `IP2025 clayparameters.htm (61)` "Stieber Constant … Default is 2.0"; corroborated `Temp\c25\clayequationsandmethodology.htm` "STB = Stieber Constant shape parameter (default =2.0)", form `Z/(1+STB(1−Z))` in `embim26.png` | T2 / T1″ |
| `larionov.k_older` | k | 2.0 | — | `IP2025 Temp\c25\embim27.png` + `Geolog vsh_gr.lls L127` + `Techlog image471.gif` | T1″ / T1 / T1′ |
| `larionov.k_younger` | k | 3.7 | — | `IP2025 Temp\c25\embim28.png` + `Geolog vsh_gr.lls L130` + `Techlog image470.gif` | T1″ / T1 / T1′ |
| `larionov.normalisation` | — | exact `1/(2^k − 1)` | — | `Techlog modules-quanti-volume-shale-thor3/thor4/pota2.gif` — vendor-printed exact form | T1′ |
| `larionov_vendor.c_older` | — | 0.33 \| 0.333 — **parity only** | — | 0.33 `Geolog vsh_gr.lls L127`; 0.333 `IP2025 Temp\c25\embim27.png` | T1 / T1″ |
| `larionov_vendor.c_younger` | — | 0.083 \| 0.08336 — **parity only** | — | 0.083 `Geolog vsh_gr.lls L130`; 0.08336 `IP2025 Temp\c25\embim28.png` | T1 / T1″ |
| `clavier.a / b / c` | — | 1.7 / 3.38 / 0.7 | — | agreed by all three: `IP2025 Temp\c25\embim25.png`, `Geolog vsh_gr.lls L137`, `Techlog …-gr7.gif` | T1″ / T1 / T1′ |
| `curved.break_lo / break_hi` | — | 0.55 / 0.73 | — | `IP Temp\c25\clayequationsandmethodology.htm` interval prose + `Techlog curved-method-equation.png`; Techlog's closure adopted | T1″ / T1′ |
| `curved.c1 / e1` | — | 0.0006078 / 1.58527 | — | `IP2025 Temp\c25\embim23.png` + `Techlog curved-method-equation.png` — **two independent vendors agree** | T1″ / T1′ |
| `curved.m2 / b2` | — | 2.1212 / 0.81667 | — | `IP2025 Temp\c25\embim24.png` + same Techlog source | T1″ / T1′ |
| `resistivity.ip_branch_k1 / k2` | — | 0.5 / 0.67 | — | `IP2025 Temp\c25\embim32.png` — `0.5 × (2 × Z)^(0.67×(Z+1))` | T1″ |
| `resistivity.geolog_b_num` | b | 0.5 | — | `Geolog vsh_res.lls` doc block + code | T1 |
| `resistivity.gaymard_b_techlog` | b | 1.0 | — | `Techlog modules-quanti-volume-shale-r1.gif` + `C2_method_defaults.json "b exponent": 1` with `"Res method": "Gaymar"` — doc and template agree | T1′ / T3 |
| `clip_low_pct` | — | 0 | % | `IP clayparameters.htm (59)`, both editions — ledger D-01 | T2 |
| `clip_high_pct` | — | 98 | % | `IP clayparameters.htm (60)`, both editions — ledger D-01 | T2 |
| `percentile_clay` | — | 130 | % | `IP clayparameters.htm (57)` | T2 |
| `percentile_clean` | — | ABSENT — ships with no default | % | ledger D-14 — absent from CHM *and* `ClayVol.hlp` | T2 |
| `house_preset.p3_p97` | — | P3 / P97 | percentile | `docs/workflow_standards.md`; realisation `P3 = 53.68`, `P97 = 133.93 gAPI`, `project-kb\records\lqr-balam-south-phr.md` Final Report Sec 4.3.6 | T4 |
| `gr_clean` (Techlog parity) | GR_ma | 10 | gAPI | `Techlog petrophysics-vsh-from-gamma-ray.html` — **starting range only** | T1′ |
| `gr_clay` (Techlog parity) | GR_sh | 100 | gAPI | same — **starting range only** | T1′ |
| `gr_clean` validation | — | 0–200 | gapi | `Geolog vsh_gr.info` VALIDATION | T1 |
| `gr_clay` validation | — | 0–1000 | gapi | same | T1 |
| `rho_matrix` | ρ_ma | ABSENT — ships with no default (vendor presets 2645 k/m3 \| 2.65 g/cm3) | g/cc | `Geolog vsh_dn.info DEFAULT 2645 k/m3` \| `Techlog …-neutrondensity.html 2.65 g/cm3` | T1 / T1′ |
| `rho_fluid` | ρ_f | 1.000 | g/cc | `Geolog vsh_dn.info 1000 k/m3` + `Techlog 1.0 g/cm3` — **both agree** | T1 / T1′ |
| `rho_shale` | ρ_sh | ABSENT — ships with no default — ⚠ **TWO VENDOR VALUES**: 2.40 \| 2.45 | g/cc | 2.40 `Techlog …-neutrondensity.html`; 2.45 `Techlog C2_method_defaults.json RHOB_shale = 2.45 g/cm3` and all four `Q*_PR.xml`. Geolog defers to well constant `RHO_SH`, validation `1000:4000 k/m3` | T1′ / T3 / T1 |
| `rho_dry_shale` | — | NON-ADOPTABLE — cited for verification (Techlog template preset 2.7) | g/cc | `Techlog C2_method_defaults.json "RHOB dry shale": 2.7 g/cm3` — template only, absent from every doc page; **used by no Vsh equation in this dossier**; recorded so it is not confused with `rho_shale` | T3 |
| `nphi_fluid` | φN_f | 1.0 | v/v | `Geolog vsh_dn.info` + `Techlog …-neutrondensity.html` + `Techlog C2 NPHI_fluid = 1` — **all witnesses agree** | T1 / T1′ / T3 |
| `nphi_matrix` | φN_ma | ABSENT — ships with no default — ⚠ **TWO VENDOR VALUES**: −0.1 \| 0 | v/v | −0.1 `Techlog …-neutrondensity.html` and `…-thermal-neutron.html`; 0 `Techlog C2_method_defaults.json NPHI_matrix = 0` and all four `Q*_PR.xml`. Geolog: well constant `NPHI_MA`, validation `−0.2:0.5`. The two give **0.8000 and 0.7500 on the same rock**; at 0 all three tools coincide | T1′ / T3 / T1 |
| `nphi_shale` | φN_sh | ABSENT — ships with no default (Techlog preset 0.4) | v/v | `Techlog …-neutrondensity.html` + `C2 NPHI_shale = 0.4 v/v` — doc and template agree; Geolog validation `0:1` | T1′ / T3 / T1 |
| `dt_matrix_sandstone` | Δt_ma | ABSENT — ships with no default, **module-scoped** — ⚠ **TWO VENDOR VALUES within one vendor** | uS/ft | Four values, five witnesses: **50** `Techlog …-sonicdensity.html` \| **55.5** `Techlog C2_method_defaults.json DT_matrix = 55.5` \| **55** `IP basicloganalysis.htm` "55 uSec/ft for sandstone" \| **55.50** `Geolog vsh_ds.info DT_MA 182.1 us/m` \| **56** `IP swparameters.htm` (ledger D-13). Techlog's own two differ by 20.8 % relative on Vsh_SD | T1′ / T3 / T1″ / T1 / T2 |
| `dt_matrix_limestone` | Δt_ma | ABSENT — ships with no default (IP preset 49) | uS/ft | `IP basicloganalysis.htm` "Set to 49 uSec/ft for limestone" — the only non-sandstone Δt_ma stated by any tool | T1″ |
| `rho_matrix_limestone` | ρ_ma | ABSENT — ships with no default (IP preset 2.71) | g/cc | `IP basicloganalysis.htm` "Set to 2.71 gm/cc for limestone" | T1″ |
| `dt_fluid` | Δt_f | 189 (**fresh water**) | uS/ft | `Techlog …-sonicdensity.html` + `Techlog C2 DT_fluid = 189` + `Geolog vsh_ds.info 620 us/m = 188.98` + `IP basicloganalysis.htm` "Default value is189 uSec/ft" (missing space as-printed) — **all three tools agree**. ⚠ Salinity-dependent: IP states "For salt-saturated formation water use about 174 usec/ft" on the same page, and "adjust (increase) the fluid transit time" for hydrocarbons. 189 is the fresh-water case, not a universal constant | T1′ / T3 / T1 / T1″ |
| `rho_fluid_saltwater` | ρ_f | ABSENT — ships with no default (IP preset 1.1) | g/cc | `IP basicloganalysis.htm` "Set to 1.1 gm/cc for salt water" — the fresh-water 1.0 above is likewise not universal | T1″ |
| `dt_shale` | Δt_sh | ABSENT — ships with no default (Techlog preset 100) | uS/ft | `Techlog …-sonicdensity.html` + `C2 DTshale = 100 us/ft` — doc and template agree; Geolog validation `150:600 us/m` | T1′ / T3 / T1 |
| `sp_clean` \| `sp_clay` | SP_ma \| SP_sh | ABSENT — ships with no default (Techlog template presets −140 \| 20) | mV | `Techlog C2_method_defaults.json SP_matrix = −140`, `SP_shale = 20` — **unit string empty in the template**, read as mV from §2.15; the only shipped SP endpoints in any tool | T3 |
| `res_clean` \| `res_clay` | R_clean \| R_clay | ABSENT — ships with no default | ohm·m | Techlog template ships `Res_limit = 1`, `Res_shale = 10 ohm.m` — a **degenerate** pair (`R_clay > R_clean`) that SB-CLY-016 refuses. **Offered as nothing.** Picking convention instead: `IP clayparameters.htm (12)` "Generally chosen as the highest resistivity in a hydrocarbon-bearing, clay-free zone" | T3 / T1″ |
| `rhob_kerogen` | ρ_ker | 1.1 | gm/cc | `IP clayparameters.htm (64)` | T2 |
| `nphi_kerogen` | φN_ker | 0.6 | v/v | `IP clayparameters.htm (65)` | T2 |
| `sonic_kerogen` | Δt_ker | 150 | uS/ft | `IP clayparameters.htm (71)` — IP2025 only | T2 |
| `rhob_heavymin` | ρ_hvy | 4.3 | gm/cc | `IP clayparameters.htm (66)` | T2 |
| `nphi_heavymin` | φN_hvy | −0.03 | v/v | `IP clayparameters.htm (67)` | T2 |
| `sonic_heavymin` | Δt_hvy | 40 | uS/ft | `IP clayparameters.htm (72)` — IP2025 only | T2 |
| `kerogen_wt_conv` | — | 2.5 | wt%→v/v | `IP clayparameters.htm (68)` | T2 |
| `heavymin_wt_conv` | — | 1.0 | wt%→v/v | `IP clayparameters.htm (69)` | T2 |
| `gr_kerogen` | GR_ker | ABSENT — ships with no default | gAPI | `IP2025 B_core_petro.md §8 item 8` — derivation procedure only | T2 |
| `csr` | CSR | ABSENT — ships with no default | v/v | `IP swparameters.htm (178)` — no default on any page read | T2 |
| `clsr` | CLSR | ABSENT — ships with no default (literature ≈ 0.6, **Halliburton form only**) | v/v | `reference_vsh_porosity_methods.md` — must not migrate onto the IP form | T4 |
| `opt_coal` | — | `false` | flag | `Geolog vsh_gr.info` DEFAULT | T1 |
| `opt_badhole` | — | `false` | flag | `Geolog vsh_dn.info` DEFAULT; IP: "Initially, the bad hole indicator logic is not active" | T1 / T2 |
| `combination.default` | — | `minimum` over {GR, ND} | — | `Techlog C2_method_defaults.json "Final VSH Method": "minimum"`, `"VSH selection": "Gamma ray (GR);Neutron Density (ND)"`; IP: `VCL` min is the default `VWCL` | T3 / T2 |
| `badhole.preset` | — | `substitute(Vsh_GR)` | — | `Geolog vsh_dn.lls L107–115` (hard-coded) + `Techlog C2_method_defaults.json "VSH selection if flag = 1": "Gamma ray (GR)"` — **two vendors agree** | T1 / T3 |
| `gr_input_alias` | — | `GR_COR` (borehole-corrected) | — | `Geolog vsh_gr.info` L48 DEFAULT column; same discipline on `RHO_COR`/`NPHI_COR` for the doubles, **no `DT_COR` alias exists** | T1 |
| `link_clay_params` | — | `false` | flag | `IP clayparameters.htm` — links a shared curve's **clay** endpoint across doubles (`ND Den Clay` ↔ `DS Den Clay`); "DOES NOT update Single Clay Indicators". Ordinal #51, `.hlp`-only | T1″ / T2 |
| `link_clean1_params` | — | `false` | flag | same source — links **Clean 1 only** (`ND Den Clean 1` ↔ `SD Den Clean 1`); **Clean 2 is never linked** and that asymmetry is structural. Ordinal #70, `.hlp`-only | T1″ / T2 |
| `link_phisw_clay` | — | `false` | flag | same source — cross-module link to PhiSw, "only be active if the parameter sets are linked"; links clay endpoints in clay mode and shale endpoints in shale mode ("When the Calculate Shale Volume option is on the shale parameters are linked"). Ordinal #52, `.hlp`-only | T1″ / T2 |
| `indicator_use_flag` (per zone, per indicator) | — | `true` | flag | `IP clayparameters.htm` — "Set to Off for Vclay from gamma ray to be set to Null values over this zone"; zone-scoped null, distinct from bad-hole | T1″ |
| `badhole.threshold_semantics` | — | `disc > min` fires; `disc < max` fires; blank ⇒ ignored | — | `IP clayparameters.htm (46)/(47)` — the naming is inverted relative to the behaviour and must not be paraphrased | T1″ |

### 5.1 SandiBumi defaults that must be withdrawn

Six values currently ship in `modules.rs` with no source string. Four of them contradict the table
above and must be replaced by `ABSENT — ships with no default` under SB-CLY-050; two agree with a
vendor witness and may stay once a source string is attached under SB-CLY-051.

| As-built default | Line | Disposition |
|---|---|---|
| `GR_MA = 20.0` gapi | `modules.rs:521` | **Withdraw** — no vendor witness; the one witness held is 10 gAPI (T1′), worth 73.8 % through `LARINOV2` at GR = 70 |
| `GR_SH = 120.0` gapi | `modules.rs:522` | **Withdraw** — no vendor witness; the one witness held is 100 gAPI (T1′) |
| `RHO_MA = 2.645` g/cc | `modules.rs:591` | **Keep, attach source** — `Geolog vsh_dn.info DEFAULT 2645 k/m3` (T1), converted; record the conversion per SB-CLY-054 |
| `RHO_SH = 2.5` g/cc | `modules.rs:592` | **Withdraw** — matches neither of the two vendor values (2.40 \| 2.45) |
| `RHO_FL = 1.0` g/cc | `modules.rs:593` | **Keep, attach source** — all witnesses agree (T1 / T1′) |
| `NPHI_MA = −0.02` v/v | `modules.rs:594` | **Withdraw** — matches neither of the two vendor values (−0.1 \| 0) |
| `NPHI_SH = 0.35` v/v | `modules.rs:595` | **Withdraw** — the one witness held, on which doc and template agree, is 0.4 |
| `NPHI_FL = 1.0` v/v | `modules.rs:596` | **Keep, attach source** — all witnesses agree (T1 / T1′ / T3) |
| `GR_MA = 15.0` / `GR_SH = 120.0` (`vsh_dn` cross-check) | `modules.rs:597-598` | **Withdraw** — a third endpoint pair inside one product; the cross-check must use the run's own GR endpoints |
| `GR_MA = 10.0` / `GR_SH = 150.0` (`ssc`) | `ssc.rs:95-96` | **Withdraw** — a fourth endpoint pair; 22.2 % relative spread against the other two at GR = 70 |
| `FLAG_TOL = 0.25` v/v | `modules.rs:599` | **Keep** — a SandiBumi diagnostic threshold, not a petrophysical parameter; must be documented as such |
| `P_LOW = 3.0` / `P_HIGH = 97.0` | `modules.rs:2629-2630` | **Keep, attach source** — `docs/workflow_standards.md` (T4); SB-CLY-039 governs how it is presented |

---

## 6. Acceptance tests

Forty-four tests, `SB-CLY-T01` … `SB-CLY-T44`. Every expected value carries the source it was
derived from. Tests whose expectation pins current behaviour with no external authority are
labelled **CHARACTERIZATION** and are not evidence of correctness.

Numeric expectations below were computed from the cited equations and the cited parameter sets;
where a value is stated to four decimals the tolerance is `1e-4` absolute unless a tighter one is
given.

| ID | Input | Operation | Expected (tolerance) | Source of expected value |
|---|---|---|---|---|
| SB-CLY-T01 | `GR = 70`, `GR_MA = 120`, `GR_SH = 20` (inverted) | `vsh_gr`, any transform | No value emitted; provenance token `ENDPOINT_INVALID`; run-level message naming `GR_MA`/`GR_SH`, the zone and both values. **Not** a bare null | SB-CLY-001; guard precedent `Geolog vsh_gr.lls L99–102` (T1) |
| SB-CLY-T02 | `I = 1.0` | Larionov exact, `k = 2` and `k = 3.7` | `1.000000000` both (`1e-9`) | Boundary condition of `(2^(kI)−1)/(2^k−1)`; vendor-printed exact form `Techlog …-thor3/thor4/pota2.gif` (T1′) |
| SB-CLY-T03 | `I = 1.0` | Larionov **parity** mode, `k = 2` and `k = 3.7` | `0.990000` and `0.995671` (`1e-6`) | `Geolog vsh_gr.lls L127` (0.33) and `L130` (0.083) (T1); pinned as-built at `modules.rs:3756-3769` |
| SB-CLY-T04 | `I = 0.5` | Larionov exact, `k = 2` vs `k = 3.7` | `0.333333` vs `0.217148` (`1e-6`) — a 53.5 % relative gap between the two rock-age forms. Parity mode: `0.330000` vs `0.216207` | Direct evaluation of the cited forms; label mapping confirmed at `modules.rs:511-518`, `modules.rs:557-558` |
| SB-CLY-T05 | `I` swept `0 → 1` in 0.01 steps | Generic Stieber `n = 2` against the as-built `STIEBER1` | Identical (`1e-12`) at every step | Algebraic identity `I/(1+2(1−I)) ≡ I/(3−2I)`; as-built `modules.rs:547` |
| SB-CLY-T06 | `I = 0.5`, `n = 2.5` | Generic Stieber | `0.285714` (`1e-6`) — i.e. `0.5/(1+2.5×0.5)` | Direct evaluation of `IP2025 embim26.png` form `Z/(1+STB(1−Z))` (T1″) |
| SB-CLY-T07 | `n` swept over `{0.5, 1, 2, 2.5, 3, 5}` | Derived Stieber clamp | Clamp equals `(1+n)/n − ε` for every `n`, with `ε` the single named constant; the transform is finite and monotonic at the clamp for every `n` | SB-CLY-009; the pole of `I/(1+n(1−I))` is at `I = (1+n)/n` |
| SB-CLY-T08 | `I = √3.38 − 0.7 = 1.13847763…` | Clavier, unlimited output `VSH_GR` | `1.700000` (`1e-6`); radicand `0.0` | Analytic: the radicand `3.38 − (I+0.7)²` vanishes at the bound. Equation agreed by all three (`Geolog vsh_gr.lls L137`, `IP embim25.png`, `Techlog …-gr7.gif`) |
| SB-CLY-T09 | `I = 1.13` (Geolog's rounded clamp) | Clavier, unlimited output | `1.523648` (`1e-6`) — **0.176352 below** the value at the exact bound, a 10.4 % shortfall on the QC twin | Direct evaluation: `(1.83)² = 3.3489`, `3.38 − 3.3489 = 0.0311`, `√0.0311 = 0.1763519`. As-built clamp `modules.rs:561` |
| SB-CLY-T10 | An interval where `GR > GR_SH` for 40 % of samples | Any transform, clipped output | `VSH = 1.000` over that interval **and** a per-sample clamped marker set on exactly those samples; run record reports `clamped = 40.0 %` for the zone (`0.1 %`) | SB-CLY-010 |
| SB-CLY-T11 | `I` sampled either side of `0.55` and `0.73` | Curved | Continuous across both breaks (`1e-6`); branch coefficients `0.0006078 / 1.58527` and `2.1212 / 0.81667` reproduced | `IP2025 embim23.png`, `embim24.png` (T1″) + `Techlog curved-method-equation.png` (T1′) — two independent vendors agree |
| SB-CLY-T12 | A parameter set naming "Stieber 2" written by each vendor in turn | Alias resolution | Each resolves to that vendor's `n`, not to SandiBumi's `STIEBER2` | SB-CLY-003; the label collision is dossier §2.2 / F3 |
| SB-CLY-T13 | A parameter set naming "Stieber 2" with no identifiable writing application | Alias resolution | Import **fails** with a message naming the label and every candidate `n`. No value is assumed | SB-CLY-003 |
| SB-CLY-T14 | `LARINOV3` selected | `vsh_gr` run | Run-level warning stating no published source is held; provenance records the choice; at `I = 1` the unlimited output is `1.133155` (`1e-6`) | Warning per SB-CLY-006; the 1.133155 overshoot pinned at `modules.rs:3761` (**CHARACTERIZATION** for the numeric part — no vendor states a boundary value for this form) |
| SB-CLY-T15 | `SP = −60`, `SP_clean = −140`, `SP_shale = 20` | SP indicator | `0.500000` (`1e-6`) | Linear two-endpoint index; endpoints from `Techlog C2_method_defaults.json SP_matrix = −140`, `SP_shale = 20` (T3) |
| SB-CLY-T16 | One neutron reading with one endpoint pair | The three vendor neutron forms in turn | Three distinct values spanning `0.7071` to `0.8000` — **13.1 % relative**; no form is selected as a default | Dossier §2.4 worked comparison across `Geolog vsh_nphi` (T1), `Techlog …-thermal-neutron.html` (T1′), `IP` (T1″) |
| SB-CLY-T17 | An `NPHI` curve with no recorded matrix reference | Any neutron-consuming indicator | Refuses to evaluate; message names the curve and the missing matrix attribute. A sandstone-referenced and a limestone-referenced curve in one run refuses likewise | SB-CLY-013; precondition stated on `Techlog …-thermal-neutron.html` (T1′) |
| SB-CLY-T18 | `ρb = 2.35 g/cc`, `φN = 0.30 v/v` | `vsh_dn` under three endpoint sets | Techlog template (2.65 / 2.45 / 1.0 / 0 / 0.4 / 1) → `0.4239`; Techlog doc page (2.65 / 2.40 / 1.0 / −0.1 / 0.4 / 1) → `0.6000`; as-built defaults (2.645 / 2.5 / 1.0 / −0.02 / 0.35 / 1) → `0.4894` (all `1e-4`). **41.5 % relative across one vendor's two witnesses.** Setting `φN_ma` outside `[−0.1, 0]` warns; the warning fires on both sides | Equation `Geolog vsh_dn.lls L134–138` (T1) evaluated on the parameter sets in §5; as-built at `modules.rs:629-641`, `:591-596` |
| SB-CLY-T19 | One resistivity reading with one endpoint pair | The four resistivity forms in turn | Four distinct values spanning `0.0816` to `0.4114` — **a factor of 5.04**; all four displayed; none selected | Dossier §2.5 / §3.3 worked comparison across `Geolog vsh_res.lls` (T1), `Techlog …-r1.gif` (T1′), `IP embim32.png` (T1″) |
| SB-CLY-T20 | `R_clean = 1`, `R_clay = 10` (Techlog's own shipped pair) | Any resistivity indicator | Refuses **before** branch selection; message names both endpoints and states `R_clay` must be less than `R_clean` | SB-CLY-016; the degenerate pair is `Techlog C2_method_defaults.json Res_limit = 1`, `Res_shale = 10 ohm.m` (T3) |
| SB-CLY-T21 | 10 000 random `(ρb, φN)` samples over a swept parameter grid, units in both g/cc and k/m3 | Canonical bilinear form vs the as-built N-D rearrangement | Identical (`1e-12`) at every sample and in both unit systems | Algebraic equivalence proven in dossier §2.7; as-built `modules.rs:629-641` |
| SB-CLY-T22 | A matrix point and a fluid point | Clean-line constructor | Produces `c1`, `c2` such that the canonical form reproduces the restricted parameterisation exactly (`1e-12`); a `c2` moved off the matrix–fluid line produces a different, finite result | SB-CLY-019; restricted form `Geolog vsh_dn.lls L134–138` (T1) |
| SB-CLY-T23 | Two double indicators sharing a density curve | Linkage | Editing `c1` on one updates the other; editing `c2` on one does **not**; a single-indicator endpoint of the same nominal quantity is unaffected | `IP clayparameters.htm` ordinals #51 / #70 (T1″ / T2) — Clean 2 is never linked |
| SB-CLY-T24 | Shale point placed on the clean line | Any double indicator | Refuses; provenance token distinct from `MISSING`; the indicator's flag curve is **written**, not left unset | SB-CLY-021; as-built guard `modules.rs:638-640` currently leaves `VSH_DN_FLAG` at `NaN` |
| SB-CLY-T25 | The sonic-density parameter set from `Techlog …-sonicdensity.html` | Sonic-density indicator via the canonical form | Finite, in-range result; the result computed from the **as-printed** denominator has the opposite sign and is rejected by the test. `Δt_ma` is module-scoped and has no default | Dossier §2.8a records the printed double-minus; canonical form per SB-CLY-018 |
| SB-CLY-T26 | Th and K readings with two-endpoint pairs | Thorium and Potassium indicators | Linear indices; endpoints appear as ordinary parameters with no shipped default | `Techlog` Quanti Shale Volume Thorium / Potassium methods (T1′) |
| SB-CLY-T27 | The EM-propagation parameter dialog and the exported parameter record | Inspection | The matrix travel-time parameter appears exactly once in each | SB-CLY-024; dossier §2.8a records the vendor naming it twice |
| SB-CLY-T28 | `VOL_CBW = 0.06`, `PHIT = 0.20` | NMR clay volume | `0.300000` (`1e-6`), typed `VCL`, distinct provenance token; a module expecting `VSH` refuses it | `Geolog vsh_nmr.lls L54` `VCL_NMR = limit(VOL_CBW_NMR/PHIT_NMR, 0, 1)` (T1) |
| SB-CLY-T29 | Two indicator pairs, one with a contributor above 1 and one with a contributor below 0 | Arithmetic mean, clip-then-combine vs combine-then-clip | Case A: `0.500` vs `0.600` — 20 %. Case B: `0.200` vs `0.125` — 60 %. SandiBumi returns the clip-then-combine value and states the order in the run record | Dossier §2.11 / §3.4 worked cases |
| SB-CLY-T30 | Random clipped contributor sets | Each of minimum, mean, median, Lateral pseudomedian | Every output lies within `[min(inputs), max(inputs)]` (`1e-12`), 10 000 trials | SB-CLY-028; bound-preservation established in dossier §2.10 / §2.10a |
| SB-CLY-T31 | Contributors `{0.0, 0.4}` | Arithmetic mean and median | `0.200` and `0.200` (`1e-9`) — the zero is included | SB-CLY-029; dossier §3.6 records a zero-dropping median as a documented vendor behaviour |
| SB-CLY-T32 | One interval with a missing GR, one masked by a discriminator, one with inverted endpoints | Any indicator | All three produce no value; all three carry **different** provenance tokens; a downstream consumer can distinguish them without inspecting the inputs | SB-CLY-030 |
| SB-CLY-T33 | A run combining two indicators over a zoned well | Full workflow | A provenance curve is emitted, one token per sample, drawn from the closed vocabulary; every parameter used appears in the run record with its source string; a parameter with no source string fails the run | SB-CLY-031, SB-CLY-051 |
| SB-CLY-T34 | A sample where bad hole triggered substitution | Full workflow | The method token names the substituting indicator; the substitution is recorded in a **separate** field; the two are independently readable | SB-CLY-032; substitution precedent `Geolog vsh_dn.lls L107–115` + `Techlog C2 "VSH selection if flag = 1"` (T1 / T3) |
| SB-CLY-T35 | A run with rejected samples | LAS export | No numeric sentinel appears in any curve except the declared header null; header declares `−999.25`; provenance tokens are exported as a curve | SB-CLY-034, SB-CLY-055; as-built `export.rs:8`, `export.rs:80` |
| SB-CLY-T36 | `CALI = 6.0`, `BS = 8.5`, `DCAL_MAX = 1.0` (under-gauge) | Bad-hole discriminator | Flag fires. The as-built one-sided test does **not** fire — this test currently fails | SB-CLY-035; as-built `modules.rs:1231` tests `cl − bit > dcal_max` only |
| SB-CLY-T37 | `RHOB = 1.7`, `NPHI = 0.45`, `DT = 130`, coal branch enabled | Any indicator | `VSH = 0.000` with provenance token `COAL`; with the branch disabled (the default) the indicator returns its ordinary value; where the hole is flagged bad the coal branch does not fire | `Geolog OPT_COAL` default FALSE (T1); as-built detector `modules.rs:1282-1284`, `:1376-1379` |
| SB-CLY-T38 | A GR curve with a known distribution | P3/P97 endpoint picking over a named pooling group | Endpoints equal the P3 and P97 of the pooled, pre-clipped data (`1e-6`); the run record names the preset, the pooling group and the realised values | SB-CLY-037, SB-CLY-039; house preset `docs/workflow_standards.md` (T4), realisation `P3 = 53.68`, `P97 = 133.93 gAPI` (T4) |
| SB-CLY-T39 | An endpoint set by percentile, then edited as a value | Endpoint editor | The displayed percentile updates to match; the record states the value was authoritative. Reversing the order reverses the record | SB-CLY-038 |
| SB-CLY-T40 | A percentile pick placing 30 % of an interval inside a transform's clamped region | Endpoint picking | Warning fires reporting `30 %` (`0.1 %`) | SB-CLY-040 |
| SB-CLY-T41 | `Vsh = 0.5`, ratio parameter unset | Vsh→Vcl bridge | Refuses; no default ratio is supplied. With the ratio set to 0.6 under each of the two vendor forms in turn, the results differ by **30 %** (`0.300` vs `0.210`, `1e-4`) and both are labelled with their form | SB-CLY-044; dossier §2.13 worked comparison (`φ_sh = 0.15`) |
| SB-CLY-T42 | A shale endpoint expressed against Vsh, consumed by a Vcl-expressed module; and `ρ_ma` transcribed as `2645 k/m3` | Conversion identities | The Vsh/Vcl conversion is applied by the named identity, not by direct reuse; `2645 k/m3` converts to `2.645 g/cc` (`1e-9`) and the run record states the artefact unit and the conversion applied | SB-CLY-045, SB-CLY-054; source unit `Geolog vsh_dn.info` (T1) |
| SB-CLY-T43 | Every curve mnemonic this domain emits, plus each vendor's Vsh/Vcl mnemonics | `family_for` | Every one resolves to a family; clipped Vsh, unclipped Vsh, Vcl and the flag/provenance curves resolve to **four distinct** families; a `VCL` supplied where `VSH` is required is refused; a raw `GR` is not preferred over a corrected alias | SB-CLY-043, SB-CLY-046, SB-CLY-041; as-built `curves.rs:21-37` registers none of them and folds `GRN` into `GR` at `curves.rs:22` |
| SB-CLY-T44 | A LAS file containing bare `−999` values and **no** `~W NULL` declaration | Import | Values are treated as absent and a warning names the sentinel and the affected curves. The declared-null case and the `−999.25` / `−9999.0` cases continue to pass | SB-CLY-034; vendor behaviour dossier F8 (IP writes `−999` over bad hole); as-built `parsers.rs:130`, `parsers.rs:138-140` |

**Tests that fail against the current build:** SB-CLY-T01, T02, T04 (exact-form limb), T05–T08,
T10–T13, T15–T17, T19, T20, T22–T28, T29–T34, T36–T44. SB-CLY-T03, T09, T14 (numeric limb), T18
(as-built limb), T21 and T35 pass today. This is expected: the chapter describes a domain in which
SandiBumi ships two of twelve indicators.

---

## 7. Open items, escalations and refusals

### 7.1 Open items carried from the dossier

Sixteen numbered items, carried unchanged. Item 11 was closed in the dossier's own revision pass
and its number is retired rather than reused. Each states what would close it; none blocks a
requirement above, because every requirement is written so its adoption decision does not depend on
the answer.

| # | Item | What closes it |
|---|---|---|
| 1 | Geolog M–N line constant: doc `0.308` vs code `0.388` | The 1995 M–N chart the doc cites, or a live Geolog run — **escalated as E1** |
| 2 | `LARINOV3` = `0.127 × (3.15^(2I) − 1)` — present in no other tool, uncited, violates `Vsh(1) = 1` by 13 % | A named paper or a Geolog theory document — **escalated as E2** |
| 3 | Techlog sonic-density denominator double-minus | A live Techlog run on a known fixture — **escalated as E3**. SB-CLY-022 is written so the answer does not change the implementation |
| 4 | Techlog `VSH_FINAL` clip order — undocumented on every page read | A live Techlog run — **escalated as E3**. Worth 20–60 % (SB-CLY-T29); SB-CLY-027 adopts clip-then-combine on its own merits |
| 4a | Techlog's Median, Harmonic mean and Minimum are **defined twice, differently**, in the same shipped tree — one page ignores zeros and thresholds at `> 0.0001`, the other does not | A live Techlog run on a fixture containing a legitimate `0.00` indicator — **escalated as E3**. SB-CLY-029 settles SandiBumi's behaviour independently |
| 5 | IP's `Lateral` average: the text says "median of pair **products**", the estimator it names uses pairwise **averages** | Hodges & Lehmann 1970, or a live IP run — **escalated as E3/E4** |
| 6 | **No tool cites a primary source for the Larionov, Clavier or Stieber GR transforms.** IP gives surnames and rock-age scope; Geolog gives nothing; Techlog gives only a 1994 textbook on its GR page | Larionov 1969, Clavier et al. 1971, Stieber 1970/71 — **escalated as E4**. See refusal R8 on the Thomas & Stieber 1975 lead |
| 7 | `Percentile Clean` has no default in the IP2025 CHM **or** `ClayVol.hlp` | **Nothing — confirmed to have no vendor answer.** A finding, not a gap; ships `ABSENT — ships with no default` |
| 8 | `Gr Kerogen` has no default in any IP source; a derivation procedure is given instead | **Nothing — confirmed to have no vendor answer** |
| 9 | `Clay Shale Ratio` has no default on any IP page read, despite being the tool's only Vsh→Vcl bridge; compounded by the Halliburton `CLSR ≈ 0.6` form, which is a **different equation** | **Nothing — confirmed to have no vendor answer.** Needs a cited source per project, never a global default (SB-CLY-044) |
| 10 | Techlog's `POTA unit` "Kcl effect" flag — one sentence, no equation, no default | A live Techlog run or a deeper Doc read — **escalated as E3**. Affects SB-CLY-023 (P3) only |
| ~~11~~ | ~~Techlog EM-propagation page~~ | **Closed in the dossier's revision pass** — an ordinary two-point index, no new equation (dossier §2.8a). Retained struck-through as the dossier's own worked example that an item deferred "for scope" can cost less to close than to carry |
| 12 | Techlog's clay-not-shale (`Vcl`) path — a `Clay Volume Fraction` family is registered but no bridge was found in the shale-volume docs | A read of the Quanti.Elan chapter and `petrophysics-elanplus-wet-dry-clay.html` — **held sources, closable without escalation** |
| 13 | Is Techlog's Curved method actually selectable? Its own GR page documents the equation and omits it from the method list | A live Techlog session — **escalated as E3**. Bears on how much the two-vendor Curved corroboration is worth, not on whether SB-CLY-008 is implemented |
| 14 | Does the IP2025 ingest's transcription defect appear in slices outside this domain? Partially closed here — all sixteen clay-volume equation images were re-opened and **all matched**; only the organic-shale prose is defective | A targeted re-verification of the remaining report sections against the decompiled CHM — **held sources, closable without escalation.** The dossier calls this the highest-value follow-up in the list, and it is a corpus-wide item, not a `CLY` item |
| 15 | Which of Techlog's **two** shipped values for `DT_matrix`, `NPHI_matrix` and `RHOB_shale` an untouched Quanti method actually uses — doc pages give 50 / −0.1 / 2.40, all four shipped templates give 55.5 / 0 / 2.45 | A live Techlog run: create a fresh Shale Volume method **without** loading a template and read the parameter grid — **escalated as E3**. Until then both are recorded and neither is a preset |
| 16 | IP's `Vker`/`Vhvy` ↔ `Rho_dry_rock` fixed-point iteration — no starting value, no iteration order, no convergence tolerance is given | A live IP run or an IP algorithm document — **escalated as E3**. A scope boundary rather than a blocker: SB-CLY-049 refuses to iterate |

### 7.2 Escalations

**E1 — Geolog's M–N line constant (`0.308` documentation vs `0.388` code).** The code is
authoritative for what Geolog *computes*; nothing held says which value is *correct*. One digit is
transposed somewhere and the evidence does not say where. Implementing either would be inventing a
petrophysical parameter. **Requested:** a decision on whether to book Geolog time or source the
1995 M–N chart. **Until resolved:** SB-CLY-025 makes the absence a recorded decision.

**E2 — `LARINOV3` provenance.** The transform ships in SandiBumi today (`modules.rs:559`) with the
code comment honestly recording that no source is held. It overshoots its boundary condition by
13.3 % and appears in no other product. **Requested:** a decision between (a) keeping it as a
warned, provenance-recorded parity option under SB-CLY-006, and (b) removing it. This chapter
assumes (a) because removing a shipped option breaks saved runs, but the call is Jauhar's.

**E3 — The live-vendor-session queue.** Nine items (3, 4, 4a, 5, 10, 13, 15, 16, and the numeric
limb of 1) are closable only by running the vendor's software on a known fixture. They cluster:
**seven are Techlog**, and of those, **four (4, 4a, 13, 15) are cases where Techlog's own shipped
documentation contradicts either itself or its own shipped templates.** **Requested:** a decision on
whether a single half-day Techlog session is worth booking, and on what fixture. That one session
would close more of this chapter's uncertainty than any other action available, and the questions
are specific enough to answer in minutes each.

**E4 — Three missing primary citations.** Larionov 1969, Clavier et al. 1971 and Stieber 1970/71
are cited by no vendor. The exact-vs-rounded Larionov question is settled evidentially (a vendor
prints the exact normalised form), so **no requirement depends on obtaining them** — but every
transform in SB-CLY-002 through SB-CLY-008 currently traces to a vendor artefact rather than to the
literature, which is a weaker chain than this project normally accepts. **Requested:** whether to
commission the three-paper acquisition. Note the corpus independently records that any *reported*
IP2018 citation for these three is fabricated, so the gap must not be closed from a secondary
source.

**E5 — The Stieber clamp epsilon (engineering, not petrophysical).** SB-CLY-009 requires the
generic Stieber clamp to be `(1+n)/n − ε`. As `ε → 0` the unlimited output diverges without bound,
so `ε` must be a stated constant — but it is a numerical-conditioning choice, **not** a
petrophysical parameter, and this chapter does not treat picking it as a parameter-discipline
exception. **Requested:** confirmation that this reading is right, and a value. This chapter
deliberately does not pick one.

**No second parameter-discipline exception is claimed.** CONTRACT §2.1 requires a chapter that
believes it has a second case to stop and escalate rather than decide. This chapter examined every
disputed value in §5 and found none that warrants one: where vendors disagree, the parameter ships
`ABSENT — ships with no default`, and the one place an adjudication *is* made — the exact Larionov
normalisation over the rounded decimals — is settled by a vendor printing the exact form, which is
evidence rather than judgement.

### 7.3 Refusals

Two kinds, listed separately per `CONTRACT.md` §2.2.1.

#### 7.3.1 Transcription refusals — rule compliance

**R1 — No M–N crossplot shale volume** until E1 resolves (SB-CLY-025). Three products offer it. The
absence is recorded as a decision with a reason.

**R2 — No vendor chart lookup-table data.** The only chart referenced anywhere in this chapter is
the M–N chart in E1, cited by existence, attribution and purpose only. No `.neu`, `.ovl`, `.itt`,
`.itp`, `.att`, `.bor` or `.eli` content was read or reproduced.

**R6 — Refuse to iterate kerogen and heavy-mineral volumes inside the clay module** (SB-CLY-049).
They are inputs. Where unknown, the module refuses.

**R7 — Refuse to synthesise a default from disagreeing vendors.** Fifteen parameters in §5 ship
with no default, and seven current SandiBumi defaults are withdrawn in §5.1 for matching no witness.
Averaging, interpolating or silently picking between competing vendor values is not available as a
resolution — it is how SandiBumi's present `RHO_SH = 2.5`, `NPHI_MA = −0.02` and `NPHI_SH = 0.35`
came to be numbers with no source.

**R8 — Refuse to close the Stieber citation gap with Thomas & Stieber 1975.** One vendor cites that
paper, for the shaly-sand *distribution model*. It is a different result by (partly) the same
author, and adopting it as the GR-transform citation would be the same-name-therefore-same-thing
error. It is a place to start a search, nothing more.

**R9 — No client well, field, block or operator names appear in this chapter.** The one field-scoped
realisation quoted (`P3 = 53.68`, `P97 = 133.93 gAPI`) is carried from a delivered-study record
already in the corpus and is used only to show that the house percentile preset has a realisation,
not to propose it as a default.

#### 7.3.2 Defect refusals — vendor behaviour SandiBumi declines to reproduce

**R4 — Refuse to transcribe the printed sonic-density denominator** (SB-CLY-022). The vendor's
shipped equation image carries a sign defect. SandiBumi implements the canonical cross-product form,
which is correct regardless of how E3 resolves.

**R5 — Refuse the corpus's organic-shale form in favour of the vendor's** (SB-CLY-047). An internal
ingest report omits the renormalising denominator; both editions of the vendor page carry it,
character-identical. The report is wrong by 23 % on a realistic organic interval. Following the
vendor here is following the primary source, not the vendor.

### 7.4 Independent-derivation requirements

No Tier-C item falls in this domain.

---

## 8. Traceability — dossier disposition

**Row count and reconciliation.** 240 rows. The dossier does not state a single finding count for
itself, so the count below is derived by the CONTRACT §3 rule — one row per numbered finding,
discrepancy-ledger entry, OPEN item and adoption-spec line — enumerated as:

| Dossier section | Items | Counting rule applied |
|---|---|---|
| §1.1 / §1.2 / §1.3 inventories | 3 | one row per vendor inventory |
| §1.3 documentation defects | 5 | the dossier numbers them 1–5 |
| §1.4 "no evidence held" statements | 7 | 6 live + 1 struck-through (the dossier's prose says "six"; the seventh bullet is retained struck-through and is dispositioned here so the row set matches the file) |
| §2.1–§2.15 plus §2.8a, §2.10a | 17 | one row per numbered subsection |
| §3.1–§3.10 | 10 | one row per numbered subsection |
| §4.1–§4.10 | 10 | one row per numbered subsection |
| §4.11 discrepancy ledger | 6 | D-01, D-03, D-14, O/R-10, D-13, D-16 |
| §5.1 canonical equation forms | 34 | one row per named form, guard, geometry rule or absence rule in the adoption-spec block |
| §5.2 parameter table | 58 | one row per parameter row |
| §5.3 tests | 41 | numbered 1–41; the file lists them 1–25, 28–41, then 26–27 |
| §5.4 `FINDINGS.md` §6 rules | 10 | one row per rule |
| §6 closed items | 7 | the dossier's own "closed during this pass" bullets |
| §6 OPEN items | 17 | 1–16 including 4a, plus retired 11 |
| §8 critique disposition | 15 | B1, M1–M5, m1–m9 |
| **Total** | **240** | |

Two counts in the dossier's own prose differ from this enumeration and the difference is stated
rather than smoothed over: §1.4 says "six" where the file carries seven bullets (the seventh is the
struck-through EM-propagation item, closed in the revision pass and duplicated as retired OPEN 11);
and §6 says "OPEN items (16)" where seventeen numbered slots exist, because 11 is retired rather
than reused. Both duplicated items are dispositioned once in each place they appear and cross-refer,
so no finding is counted as resolved twice.

The dossier's `## Critique disposition` records **15 of 15 findings acted on; 0 deferred; 0 rejected
outright**, and this chapter treats that disposition as authoritative over any body text it corrects.

### 8.1 Method inventory (§1) — 15 rows

| # | Dossier item | Disposition | Where |
|---|---|---|---|
| 1.1 | IP Clay Volume module inventory — 9 indicators, organic-shale pre-corrections, `VCL`/`VCLAV`/`VCLMIX` combination, bad-hole gating, `CSR` bridge, percentile auto-pick; no coal logic | ADOPTED | SB-CLY-011, -012, -015, -018, -027, -033, -037, -044, -047 |
| 1.2 | Geolog `determin` `vsh_*` family — 13 modules; three family conventions (`OPT_COAL` default FALSE, `MTH_VSH ALPHA*8` provenance, unclipped + clipped output pair) hold for all 8 classical modules; one deterministic Vcl (`VCL_NMR`); **no Vsh→Vcl bridge** | ADOPTED | SB-CLY-026, -031, -036, -044 |
| 1.3 | Techlog Quanti Shale Volume inventory — 11 methods including Thorium, Potassium, EM propagation; 9 merge methods | ADOPTED | SB-CLY-023, -024, -028 |
| 1.3-d1 | **Defect 1** — Curved documented on the GR page but omitted from the list of available methods | ADOPTED + ESCALATED | SB-CLY-008; §7 OPEN 13 / E3 |
| 1.3-d2 | **Defect 2** — `First Present` definition inverted on one page, correct on another in the same shipped tree | EVIDENCE-ONLY | Corroborates SB-CLY-041's "prefer the corrected witness" discipline; SandiBumi ships no `First Present` combiner (SB-CLY-028) |
| 1.3-d3 | **Defect 3** — the resistivity page is copy-pasted from the GR page and carries the wrong symbols | EVIDENCE-ONLY | Grounds R7 and SB-CLY-051: a vendor page is not self-validating |
| 1.3-d4 | **Defect 4** — doc pages and shipped templates carry **different numbers** for three double-indicator endpoints (`Δt_ma` 50 \| 55.5; `φN_ma` −0.1 \| 0; `ρ_sh` 2.40 \| 2.45) | ADOPTED | SB-CLY-050, -051, -053; §5 ⚠ TWO VENDOR VALUES rows; §7 OPEN 15 / E3 |
| 1.3-d5 | **Defect 5** — the EM page names the same parameter twice (`TPL matrix` / `EPT matrix`) | ADOPTED | SB-CLY-024 |
| 1.4-a | Techlog EM-propagation page — *struck through, closed in the revision pass* | EVIDENCE-ONLY | Duplicated as retired OPEN 11 (row 6.11); an ordinary two-point index, SB-CLY-024 |
| 1.4-b | Techlog's Thomas-Stieber module — ten `*thomasstieber*.html` pages exist in the shipped tree, unread | DEFERRED (P3; trigger: the `TBD` chapter's Thomas-Stieber scope) | Out of this chapter's boundary (§1); the distribution model is not the GR transform — see R8 |
| 1.4-c | Techlog's Vcl (clay, not shale) path — a `Clay Volume Fraction` family is registered but no bridge found | ESCALATED | §7 OPEN 12; SB-CLY-044, -046 |
| 1.4-d | Whether Techlog clips each indicator before merging — doc silent | ESCALATED | §7 OPEN 4 / E3; SB-CLY-027 decides independently |
| 1.4-e | Geolog's percentile / endpoint-picking machinery — none exists | EVIDENCE-ONLY | Confirms the picking layer is a differentiator, not a parity item; SB-CLY-037 |
| 1.4-f | IP's Clay Volume coal handling — none found on the seven clay pages in either edition | EVIDENCE-ONLY | Only Geolog ships `OPT_COAL`; SB-CLY-036 |
| 1.4-g | Techlog's coal handling in the Vsh chain — no `coal` token on any page read | EVIDENCE-ONLY | As above; SB-CLY-036 |

### 8.2 Definitions and equations compared (§2) — 17 rows

| # | Dossier item | Disposition | Where |
|---|---|---|---|
| 2.1 | The gamma-ray index — agreed by all three | ADOPTED | §5.1 form 1; SB-CLY-001 guards it |
| 2.2 | Nonlinear GR transforms, exact forms; the Stieber numbering collision fully mapped across three vendors | ADOPTED | SB-CLY-002, -003, -004, -005, -007, -008 |
| 2.3 | Domain clamps — "the largest *silent* divergence" | ADOPTED | SB-CLY-009, -010; §3.3 as-built |
| 2.4 | SP indicator — agreed by all three modulo endpoint naming | ADOPTED | SB-CLY-011 |
| 2.5 | Neutron indicator — three genuinely different equations | ADOPTED | SB-CLY-012, -013, -014 |
| 2.6 | Resistivity indicator — four different equations; Geolog's guard read at L98–L115 and found **inadequate** | ADOPTED | SB-CLY-015, -016 |
| 2.7 | Double indicators — one geometry, three parameterisations; the reduction to a single cross-product **proved** | ADOPTED | SB-CLY-018, -019, -021; SB-CLY-T21 |
| 2.8 | Techlog's spectral GR indicators — no counterpart in IP or Geolog | ADOPTED | SB-CLY-023 |
| 2.8a | Techlog's EM-propagation indicator — `VSH_EATT = (TPL − TPL_matrix)/(TPL_shale − TPL_matrix)`, dB/m, no numeric defaults, quantile-5/95 picking; **and the sonic-density printed double-minus** | ADOPTED | SB-CLY-022, -024; R4 |
| 2.9 | Geolog's M–N indicator — no counterpart in IP or Techlog | ESCALATED | §7 E1; SB-CLY-025; R1 |
| 2.10 | Combination layer — the four bound-preserving combiners | ADOPTED | SB-CLY-027, -028 |
| 2.10a | IP's per-zone `Use` flag and parameter-linking machinery — no counterpart in Geolog or Techlog | ADOPTED | SB-CLY-020, -030, -033 |
| 2.11 | Bad-hole and coal handling; the inverted `Threshold Min`/`Max` naming that must not be paraphrased | ADOPTED | SB-CLY-033, -035, -036; §5 `badhole.threshold_semantics` |
| 2.12 | Endpoint-picking machinery; the corrected finding that Geolog has none and the "min/max of the curve" rule was IP's | ADOPTED | SB-CLY-037, -038, -041 |
| 2.13 | Numeric defaults with units, exactly as each vendor states them | ADOPTED | §5 in full; SB-CLY-050, -051 |
| 2.14 | Vsh ↔ Vcl — three incompatible positions | ADOPTED | SB-CLY-043, -044, -045 |
| 2.15 | Unit conventions per tool — "the silent-wrongness surface", with the added `dB/m` and the Evidence column | ADOPTED | SB-CLY-054; SB-CLY-T42 |

### 8.3 Differences that matter (§3) — 10 rows

| # | Dossier item | Disposition | Where |
|---|---|---|---|
| 3.1 | Missing domain clamps — IP and Techlog produce NaN or a **sign-flipped** answer where Geolog produces the right one | ADOPTED | SB-CLY-001, -009, -010; F1 |
| 3.2 | Larionov coefficients — three renderings; the evidence settles which is exact | ADOPTED | SB-CLY-004, -005; SB-CLY-T02, T03 |
| 3.3 | Resistivity indicator — a **5.04×** spread on one realistic point | ADOPTED | SB-CLY-015; SB-CLY-T19 |
| 3.4 | Clip-before-average vs average-then-clip — up to **60 %** on the combined curve | ADOPTED | SB-CLY-027; SB-CLY-T29 |
| 3.5 | Sandstone `Δt_ma` — four values from five shipped witnesses, **23.6 %** relative on Vsh_SD | ADOPTED | SB-CLY-053; §5 `dt_matrix_sandstone` |
| 3.6 | A zero-dropping median is unusable on a bounded volume fraction — and one vendor documents one | ADOPTED | SB-CLY-029; SB-CLY-T31 |
| 3.7 | Neutron indicator — the spread is the **clean endpoint**, and one vendor ships two of them (0.8000 vs 0.7500) | ADOPTED | SB-CLY-014, -050; SB-CLY-T18 |
| 3.8 | Bad-hole null vs substitute — a data-**availability** difference, not a numeric one | ADOPTED | SB-CLY-030, -032, -034 |
| 3.9 | IP is the only tool with organic-shale pre-correction | ADOPTED | SB-CLY-047, -048, -049 |
| 3.10 | **Correction to the corpus** — an internal ingest report mis-transcribed the organic-shale equations, omitting the renormalising denominator; both vendor editions carry it character-identical; a **23 %** error | ADOPTED | SB-CLY-047; R5; the T1″ ≻ T2 tier rule in this chapter's front matter |

### 8.4 Optimal choices and the discrepancy ledger (§4) — 16 rows

| # | Dossier item | Disposition | Where |
|---|---|---|---|
| 4.1 | GR index and its transforms — adopt the exact normalised Larionov, generic Stieber `n`, Curved from the two-vendor agreement | ADOPTED | SB-CLY-002 … -008 |
| 4.2 | Domain clamps — adopt Geolog's **and go further** (compute the bound, do not round it) | ADOPTED | SB-CLY-009, -010 |
| 4.3 | Resistivity indicator — ship all four forms, no default | ADOPTED | SB-CLY-015, -016 |
| 4.4 | Neutron indicator — ship all three forms, no default, state the matrix precondition | ADOPTED | SB-CLY-012, -013, -014 |
| 4.5 | Double indicators — one canonical form, two-point clean line, `c1`-linkable / `c2`-never | ADOPTED | SB-CLY-018, -019, -020, -021 |
| 4.6 | Combination layer — clip first, bounded-safe combiners only, written so the OPEN 4a answer cannot change the decision | ADOPTED | SB-CLY-027, -028, -029 |
| 4.7 | Endpoint picking — the full pipeline, two-way binding, cited house preset | ADOPTED | SB-CLY-037, -038, -039, -040 |
| 4.8 | Vsh ↔ Vcl — both bridges named, no default ratio, explicit endpoint conversion | ADOPTED | SB-CLY-043, -044, -045 |
| 4.9 | Coal — per-indicator branch, default off, own provenance token | ADOPTED | SB-CLY-036 |
| 4.10 | Organic-shale correction — implement the **vendor** form with the renormalising denominator, not the report form | ADOPTED | SB-CLY-047, -048, -049; R5 |
| D-01 | Clip Low % / Clip High % defaults are 0 / 98; the doc bug still ships in both editions | ADOPTED | §5 `clip_low_pct`, `clip_high_pct`; SB-CLY-037; dossier test 25 |
| D-03 | `Stieber` vs `Steiber` spelling; the **variant label** is equally untrustworthy; an unresolvable label is an import error, not a best guess | ADOPTED | SB-CLY-003; SB-CLY-T12, T13 |
| D-14 | `Percentile Clean` — no default in the CHM **or** the `.hlp`; negative percentiles legal, linear extrapolation below 0 % | ADOPTED | §5 `percentile_clean` = `ABSENT — ships with no default`; SB-CLY-037, -039 |
| O/R-10 | ClayVol parameter ordinals; **#41 changed which curve the clean point belongs to**; ordinals #51–54, #56, #70 exist only in the `.hlp`-derived reference | ADOPTED | SB-CLY-052; SB-CLY-T23; §5 link-flag rows |
| D-13 | Sonic Sand 56 uS/ft vs "180 uS/m"; extended by the revision pass to show the **same vendor is not internally uniform**, and Techlog more severely (20.8 % vs 2.2 %) | ADOPTED | SB-CLY-053; §3.5 as-built (SandiBumi's own four endpoint pairs); SB-CLY-050 |
| D-16 | **New, raised by the dossier against the corpus** — the ingest report omits the renormalising denominator and asserts a GR/Neu/Den asymmetry that is not in the manual; simultaneously closes the IP2018 `?`-operator item as a minus (CP1252 `0x96`), verified at byte level | ADOPTED | SB-CLY-047, -048; R5; dossier tests 28–30 |

### 8.5 Canonical equation forms (§5.1) — 34 rows

| # | Adoption-spec line | Disposition | Where |
|---|---|---|---|
| 5.1-01 | `I = (log − log_clean)/(log_clay − log_clean)` — the shared index | ADOPTED | SB-CLY-001; as-built `modules.rs:543` |
| 5.1-02 | `linear : V = I` | ADOPTED | as-built `modules.rs:564` |
| 5.1-03 | `clavier : V = 1.7 − sqrt(3.38 − (I+0.7)^2)` | ADOPTED | SB-CLY-007; as-built `modules.rs:562` |
| 5.1-04 | `stieber(n) : V = I/(1 + n(1 − I))`, `n` default 2.0 | ADOPTED | SB-CLY-002 |
| 5.1-05 | `larionov(k) : V = (2^(kI) − 1)/(2^k − 1)` | ADOPTED | SB-CLY-004 |
| 5.1-06 | `larionov_vendor(c,k) : V = c(2^(kI) − 1)` — parity only, `c` cited | ADOPTED | SB-CLY-005 |
| 5.1-07 | `curved` — three branches with the stated coefficients | ADOPTED | SB-CLY-008 |
| 5.1-08 | `V = (SP − SP_clean)/(SP_clay − SP_clean)` | ADOPTED | SB-CLY-011 |
| 5.1-09 | `neutron_ratio : V = phiN/phiN_clay` [upper bound] | ADOPTED | SB-CLY-012 |
| 5.1-10 | `neutron_linear : V = (phiN − phiN_ma)/(phiN_clay − phiN_ma)` | ADOPTED | SB-CLY-012 |
| 5.1-11 | `neutron_hybrid : V = sqrt(...)` | ADOPTED | SB-CLY-012 |
| 5.1-12 | `Z = (R_clay(R_clean − Rt))/(Rt(R_clean − R_clay))` — the shared resistivity index | ADOPTED | SB-CLY-015 |
| 5.1-13 | `gaymard_fixed_b : V = Z` [Techlog `b = 1`] | ADOPTED | SB-CLY-015; §5 `resistivity.gaymard_b_techlog` |
| 5.1-14 | `gaymard_variable_b` — branch on `Rc/Rt >= 0.5` | ADOPTED | SB-CLY-015; §5 `resistivity.geolog_b_num` |
| 5.1-15 | `ip_power_branch` — branch on `Rt <= 2·R_clay` | ADOPTED | SB-CLY-015; §5 `resistivity.ip_branch_k1/k2` |
| 5.1-16 | `log_linear : V = (log10 Rt − log10 R_clean)/…` | ADOPTED | SB-CLY-015 |
| 5.1-17 | Resistivity guards (all forms): `Rt >= R_clean → clip Rt = R_clean − eps` [flag] | ADOPTED | SB-CLY-016; dossier tests 10, 31, 32 |
| 5.1-18 | Double-indicator geometry: an explicit 2-point clean line `(c1, c2)` plus one clay point, each a pair of log values | ADOPTED | SB-CLY-019 |
| 5.1-19 | Geolog / Techlog compatibility constructor: `c1 = matrix point`, `c2 = fluid point` | ADOPTED | SB-CLY-019; SB-CLY-T22 |
| 5.1-20 | Linkage semantics (IP parity) — structural, not a UI option | ADOPTED | SB-CLY-020; SB-CLY-T23, T36, T37, T38 |
| 5.1-21 | The 2-D cross product of `(a→b)` with `(a→p)` — signed area = offset from the clean line | ADOPTED | SB-CLY-018 |
| 5.1-22 | Normalisation of that offset by the clay point's own offset gives the volume fraction | ADOPTED | SB-CLY-018; SB-CLY-T21 |
| 5.1-23 | The single expression reproduces all three vendors (proved in §2.7) | ADOPTED | SB-CLY-018; SB-CLY-T21 |
| 5.1-24 | `Vcl_nmr = clamp(CBW/phiT_nmr, 0, 1)` | ADOPTED | SB-CLY-026 |
| 5.1-25 | `Vsh_nmr = clamp(BFV/phiT_nmr, 0, 1)` | ADOPTED | SB-CLY-026, -043 |
| 5.1-26 | Combiners offered: minimum, arithmetic mean, median, Lateral pseudomedian | ADOPTED | SB-CLY-028 |
| 5.1-27 | **NOT OFFERED**: geometric mean, harmonic mean, product, sum | ADOPTED as a refusal | SB-CLY-028 ("MUST NOT offer a combiner that can return a value outside that range") |
| 5.1-28 | Three distinct absences, modelled separately; collapsing them corrupts the contributor count in the provenance output | ADOPTED | SB-CLY-030, -032 |
| 5.1-29 | Absence rule (a): discriminator rule, **depth-scoped**, on a continuous curve | ADOPTED | SB-CLY-033, -035 |
| 5.1-30 | Absence rule (b): per-zone `Use` flag, **zone-scoped**, per indicator | ADOPTED | SB-CLY-030, -033 |
| 5.1-31 | Absence rule (c): per-indicator `OPT_COAL`; when set `V = 0`, provenance `COAL` | ADOPTED | SB-CLY-036; SB-CLY-T37 |
| 5.1-32 | `csr_ratio : Vcl = CSR · Vsh` [IP] | ADOPTED | SB-CLY-044 |
| 5.1-33 | `clsr_porosity_corrected : Vcl = CLSR(Vsh − phi_sh)` [Halliburton] | ADOPTED | SB-CLY-044; the two must not migrate onto each other |
| 5.1-34 | Endpoint conversion (IP): `X_wet_clay = X_matrix + (X_shale − X_matrix)/CSR` | ADOPTED | SB-CLY-045; SB-CLY-T42 |

### 8.6 Parameter table (§5.2) — 58 rows

All 58 rows are transcribed into §5 of this chapter with values byte-exact and sources carried
unchanged. The disposition column below records what each row *does* in the requirements.

| # | Parameter | Disposition | Where |
|---|---|---|---|
| 5.2-01 | `stieber.n` | ADOPTED | §5; SB-CLY-002 |
| 5.2-02 | `larionov.k_older` | ADOPTED | §5; SB-CLY-004 |
| 5.2-03 | `larionov.k_younger` | ADOPTED | §5; SB-CLY-004 — the transform this work needs |
| 5.2-04 | `larionov.normalisation` | ADOPTED | §5; SB-CLY-004; the one adjudication in the chapter, settled evidentially |
| 5.2-05 | `larionov_vendor.c_older` | ADOPTED (parity only) | §5; SB-CLY-005 |
| 5.2-06 | `larionov_vendor.c_younger` | ADOPTED (parity only) | §5; SB-CLY-005 |
| 5.2-07 | `clavier.a / b / c` | ADOPTED | §5; SB-CLY-007 |
| 5.2-08 | `curved.break_lo / break_hi` | ADOPTED | §5; SB-CLY-008 |
| 5.2-09 | `curved.c1 / e1` | ADOPTED | §5; SB-CLY-008 |
| 5.2-10 | `curved.m2 / b2` | ADOPTED | §5; SB-CLY-008 |
| 5.2-11 | `resistivity.ip_branch_k1 / k2` | ADOPTED | §5; SB-CLY-015 |
| 5.2-12 | `resistivity.geolog_b_num` | ADOPTED | §5; SB-CLY-015 |
| 5.2-13 | `resistivity.gaymard_b_techlog` | ADOPTED | §5; SB-CLY-015 |
| 5.2-14 | `clip_low_pct` | ADOPTED | §5; SB-CLY-037; ledger D-01 |
| 5.2-15 | `clip_high_pct` | ADOPTED | §5; SB-CLY-037; ledger D-01 |
| 5.2-16 | `percentile_clay` | ADOPTED | §5; SB-CLY-037 |
| 5.2-17 | `percentile_clean` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-037, -050; §7 OPEN 7 |
| 5.2-18 | `house_preset.p3_p97` | ADOPTED as a cited preset | §5; SB-CLY-039 |
| 5.2-19 | `gr_clean` (Techlog parity) | ADOPTED as a starting range, not a default | §5; SB-CLY-050; withdraws `modules.rs:521` |
| 5.2-20 | `gr_clay` (Techlog parity) | ADOPTED as a starting range, not a default | §5; SB-CLY-050; withdraws `modules.rs:522` |
| 5.2-21 | `gr_clean` validation `0–200` | ADOPTED | §5; matches as-built `modules.rs:521` exactly |
| 5.2-22 | `gr_clay` validation `0–1000` | ADOPTED | §5; matches as-built `modules.rs:522` exactly |
| 5.2-23 | `rho_matrix` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-050; as-built 2.645 keeps its Geolog source |
| 5.2-24 | `rho_fluid` | ADOPTED | §5; all witnesses agree; as-built `modules.rs:593` retained |
| 5.2-25 | `rho_shale` ⚠ two vendor values | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-050; **withdraws `modules.rs:592`** |
| 5.2-26 | `rho_dry_shale` | ADOPTED as `NON-ADOPTABLE — cited for verification` | §5; used by no Vsh equation; recorded to prevent confusion with `rho_shale` |
| 5.2-27 | `nphi_fluid` | ADOPTED | §5; all witnesses agree; as-built `modules.rs:596` retained |
| 5.2-28 | `nphi_matrix` ⚠ two vendor values | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-014, -050; **withdraws `modules.rs:594`** |
| 5.2-29 | `nphi_shale` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-050; **withdraws `modules.rs:595`** |
| 5.2-30 | `dt_matrix_sandstone` ⚠ two vendor values within one vendor | ADOPTED as `ABSENT — ships with no default`, module-scoped | §5; SB-CLY-053; ledger D-13 |
| 5.2-31 | `dt_matrix_limestone` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-013, -053 |
| 5.2-32 | `rho_matrix_limestone` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-013 |
| 5.2-33 | `dt_fluid` | ADOPTED with the salinity caveat carried | §5; SB-CLY-051 — 189 is the fresh-water case, not a constant |
| 5.2-34 | `rho_fluid_saltwater` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-050 |
| 5.2-35 | `dt_shale` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-050 |
| 5.2-36 | `sp_clean` \| `sp_clay` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-011; SB-CLY-T15 |
| 5.2-37 | `res_clean` \| `res_clay` | ADOPTED as `ABSENT — ships with no default`; the shipped pair is degenerate | §5; SB-CLY-016; SB-CLY-T20 |
| 5.2-38 | `rhob_kerogen` | ADOPTED | §5; SB-CLY-047 |
| 5.2-39 | `nphi_kerogen` | ADOPTED | §5; SB-CLY-047 |
| 5.2-40 | `sonic_kerogen` | ADOPTED | §5; SB-CLY-047 |
| 5.2-41 | `rhob_heavymin` | ADOPTED | §5; SB-CLY-047 |
| 5.2-42 | `nphi_heavymin` | ADOPTED | §5; SB-CLY-047 |
| 5.2-43 | `sonic_heavymin` | ADOPTED | §5; SB-CLY-047 |
| 5.2-44 | `kerogen_wt_conv` | ADOPTED | §5; SB-CLY-047, -054 |
| 5.2-45 | `heavymin_wt_conv` | ADOPTED | §5; SB-CLY-047, -054 |
| 5.2-46 | `gr_kerogen` | ADOPTED as `ABSENT — ships with no default` | §5; §7 OPEN 8 — confirmed to have no vendor answer |
| 5.2-47 | `csr` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-044; §7 OPEN 9 |
| 5.2-48 | `clsr` | ADOPTED as `ABSENT — ships with no default` | §5; SB-CLY-044 — must not migrate onto the IP form |
| 5.2-49 | `opt_coal` | ADOPTED | §5; SB-CLY-036 — default off |
| 5.2-50 | `opt_badhole` | ADOPTED | §5; SB-CLY-033 — default off |
| 5.2-51 | `combination.default` | ADOPTED | §5; SB-CLY-028 |
| 5.2-52 | `badhole.preset` = `substitute(Vsh_GR)` | ADOPTED | §5; SB-CLY-032, -034; two vendors agree |
| 5.2-53 | `gr_input_alias` = `GR_COR` | ADOPTED | §5; SB-CLY-041; **diverges from as-built `modules.rs:523`** |
| 5.2-54 | `link_clay_params` | ADOPTED | §5; SB-CLY-020; SB-CLY-T23 |
| 5.2-55 | `link_clean1_params` | ADOPTED | §5; SB-CLY-020 — Clean 2 is never linked |
| 5.2-56 | `link_phisw_clay` | ADOPTED | §5; SB-CLY-020, -043 — follows the Vsh/Vcl mode |
| 5.2-57 | `indicator_use_flag` | ADOPTED | §5; SB-CLY-030, -033 |
| 5.2-58 | `badhole.threshold_semantics` | ADOPTED verbatim, not paraphrased | §5; SB-CLY-033, -035 |

### 8.7 Tests to ship (§5.3) — 41 rows

Thirty-six of the dossier's 41 tests map onto one or more of the 44 tests in §6. Five (23, 25, 28,
29, 30) are DEFERRED because the code path they exercise does not exist and is not required ahead
of its own requirement's priority. Eleven of the chapter's own tests have no dossier counterpart —
SB-CLY-T04, T06, T13, T14, T24, T26, T27, T39, T41, T43, T44 — and pin either an as-built
divergence found in §3 or a capability the dossier describes without proposing a test for.

| # | Dossier test | Disposition | Where |
|---|---|---|---|
| 5.3-01 | `curved_ip_techlog_identical` | ADOPTED | SB-CLY-T11 |
| 5.3-02 | `stieber_variant_mapping` | ADOPTED | SB-CLY-T05, T12 |
| 5.3-03 | `larionov_boundary_condition` | ADOPTED | SB-CLY-T02, T03 |
| 5.3-04 | `nd_parameterisation_equivalence` | ADOPTED | SB-CLY-T21 |
| 5.3-05 | `resistivity_four_forms` | ADOPTED | SB-CLY-T19 |
| 5.3-06 | `stieber_pole_is_clamped_not_flipped` | ADOPTED | SB-CLY-T07 |
| 5.3-07 | `clavier_domain_clamp` | ADOPTED | SB-CLY-T08, T09 |
| 5.3-08 | `clamp_bound_is_computed_not_hardcoded` | ADOPTED | SB-CLY-T07, T08; SB-CLY-009 |
| 5.3-09 | `degenerate_endpoints_null_not_inf` | ADOPTED, **extended** — the chapter requires a flag, not only a null | SB-CLY-T01; SB-CLY-001 |
| 5.3-10 | `resistivity_guards` | ADOPTED | SB-CLY-T20 |
| 5.3-11 | `clip_before_average` | ADOPTED | SB-CLY-T29 |
| 5.3-12 | `clip_commutes_for_minimum` | ADOPTED | SB-CLY-T29, T30 |
| 5.3-13 | `median_includes_zeros` | ADOPTED | SB-CLY-T31 |
| 5.3-14 | `zero_is_not_null` | ADOPTED | SB-CLY-T31, T32 |
| 5.3-15 | `no_unbounded_combiners` | ADOPTED | SB-CLY-T30 |
| 5.3-16 | `unit_roundtrip_sonic` | ADOPTED | SB-CLY-T42; SB-CLY-054 |
| 5.3-17 | `dt_matrix_spread_is_visible` | ADOPTED | SB-CLY-T25; SB-CLY-053 |
| 5.3-18 | `neutron_matrix_unit_precondition` | ADOPTED | SB-CLY-T17 |
| 5.3-19 | `provenance_curve_emitted` | ADOPTED | SB-CLY-T33 |
| 5.3-20 | `badhole_substitution_is_labelled` | ADOPTED | SB-CLY-T34 |
| 5.3-21 | `clamped_one_differs_from_true_one` | ADOPTED | SB-CLY-T10; SB-CLY-010 |
| 5.3-22 | `null_written_as_minus_999_25` | ADOPTED | SB-CLY-T35; passes today (`export.rs:8`, `:80`) |
| 5.3-23 | `other_double_ordinal_semantic_mismatch_is_error` | DEFERRED (P2; trigger: the vendor parameter-import path lands) | SB-CLY-052 — there is no import path to test today |
| 5.3-24 | `stieber_spelling_aliases` | ADOPTED | SB-CLY-T12, T13 |
| 5.3-25 | `clip_defaults_are_zero_and_ninetyeight` | DEFERRED (P1; trigger: SB-CLY-037's percentile pipeline lands) | §5 rows 5.2-14/15 hold the values; ledger D-01 |
| 5.3-26 | `balam_south_gr_endpoints` | ADOPTED | SB-CLY-T38; SB-CLY-039 |
| 5.3-27 | `p97_pole_interaction_warning` | ADOPTED | SB-CLY-T40 |
| 5.3-28 | `organic_shale_renormalises` | DEFERRED (P3; trigger: SB-CLY-047 lands) | SB-CLY-047; ledger D-16 |
| 5.3-29 | `organic_shale_gr_does_not_renormalise` | DEFERRED (P3; trigger: SB-CLY-047 lands) | SB-CLY-047; ledger D-16 |
| 5.3-30 | `organic_shale_denominator_guard` | DEFERRED (P3; trigger: SB-CLY-047 lands) | SB-CLY-048 |
| 5.3-31 | `resistivity_guard_is_pre_branch` | ADOPTED | SB-CLY-T20; SB-CLY-016 |
| 5.3-32 | `resistivity_both_guards_together` | ADOPTED | SB-CLY-T20 |
| 5.3-33 | `two_valued_presets_are_flagged` | ADOPTED | SB-CLY-T18; SB-CLY-050 |
| 5.3-34 | `neutron_clean_endpoint_spread` | ADOPTED | SB-CLY-T18; SB-CLY-014 |
| 5.3-35 | `degenerate_shipped_endpoints_are_rejected` | ADOPTED | SB-CLY-T20 |
| 5.3-36 | `clean2_is_never_linked` | ADOPTED | SB-CLY-T23; SB-CLY-020 |
| 5.3-37 | `doubles_and_singles_never_link` | ADOPTED | SB-CLY-T23; SB-CLY-020 |
| 5.3-38 | `phisw_link_follows_vsh_vcl_mode` | ADOPTED | SB-CLY-T23; SB-CLY-020, -043 |
| 5.3-39 | `three_absences_are_distinguishable` | ADOPTED | SB-CLY-T32; SB-CLY-030 |
| 5.3-40 | `two_sided_badhole_gate` | ADOPTED — and it **fails against the current build** | SB-CLY-T36; `modules.rs:1231` is one-sided |
| 5.3-41 | `nmr_module_is_the_documented_exception` | ADOPTED | SB-CLY-T28; SB-CLY-026 |

### 8.8 Applicable `FINDINGS.md` §6 rules (§5.4) — 10 rows

| # | Rule | Disposition | Where |
|---|---|---|---|
| 5.4-01 | **1 — No raster-only truth.** All 13 IP clay equations existed only as raster in IP2018 | ADOPTED | The T1″ tier and its precedence over T2 in this chapter's front matter; R5 |
| 5.4-02 | **3 — Unit-typed quantities, no magic constants.** Geolog is k/m3 + us/m; Techlog is g/cm3 + us/ft; Techlog EM is dB/m | ADOPTED | SB-CLY-054; SB-CLY-T42; §5 transcribes each value in its artefact's unit |
| 5.4-03 | **6 — Null discipline.** IP nulls bad-hole doubles to **−999**, not −999.25 | ADOPTED | SB-CLY-034, -055; SB-CLY-T44 — the undeclared case is the one the current build misses |
| 5.4-04 | **7 — Ordinal + semantic-name addressing.** The #41 swap | ADOPTED | SB-CLY-052; ledger O/R-10 |
| 5.4-05 | **9 — Defaults are cited or absent.** `Percentile Clean`, `Gr Kerogen`, `CSR`/`CLSR` and the rest | ADOPTED | SB-CLY-050, -051; 15 of 58 §5 rows ship `ABSENT — ships with no default`, one ships `NON-ADOPTABLE — cited for verification` |
| 5.4-06 | **10 — Docs generated from code.** Geolog's `vsh_mn.lls` doc block says `0.308`; its code says `0.388` | ESCALATED | §7 E1; SB-CLY-025; R1 |
| 5.4-07 | **11 — Worked examples must reproduce.** Every numeric in the dossier's §3 is written as an executable case | ADOPTED | §6 in full — every expected value in the test table carries the source it was derived from |
| 5.4-08 | **9 / 11 extension — a transcription is not a source.** §3.10 is the worked case | ADOPTED | R5; the T1″ ≻ T2 tier rule; SB-CLY-051 (the artefact, not the product name, is the source) |
| 5.4-09 | **14 — Silent failures are bugs.** A zero-dropping median silently discards the clean intervals | ADOPTED | SB-CLY-001, -010, -029, -030, -034, -040; the whole fail-loud group |
| 5.4-10 | **15 — Curve resolution and snapping are logged decisions.** Geolog's `MTH_VSH` is the only vendor precedent | ADOPTED | SB-CLY-031, -032, -041, -046 |

### 8.9 Items closed during the dossier pass (§6) — 7 rows

Recorded so they are not re-opened.

| # | Closed item | Disposition | Where |
|---|---|---|---|
| 6.C1 | The IP `?`-mojibake operator in the organic-shale corrections — **closed as a minus**, byte `0x96` (CP1252) at the same position in both editions, with three independent evidential steps | EVIDENCE-ONLY | Ledger D-16; SB-CLY-047; the IP2018 report had deferred it to "a live IP run" — it needed a hex read and a grep |
| 6.C2 | Techlog's EM-propagation indicator — **closed by reading the page**: an ordinary two-point index, no new equation | EVIDENCE-ONLY | SB-CLY-024; retired OPEN 11 |
| 6.C3 | Whether Geolog ships any deterministic Vcl — **closed: yes, one** (`VCL_NMR`, `LICENCE = DETERMIN`, consumed by all three combiners) | EVIDENCE-ONLY | SB-CLY-026; what Geolog lacks is the *bridge*, not the quantity |
| 6.C4 | Whether Geolog has any endpoint-picking rule — **closed: none.** The "min/max of the curve" rule was IP's, transposed | EVIDENCE-ONLY | SB-CLY-037 — the picking layer is a differentiator, not a parity item |
| 6.C5 | Whether the IP2025 or IP2018 organic-shale form is correct — **closed in favour of the IP2018 structure**; the denominator is present, character-identical, in both editions | EVIDENCE-ONLY | R5; ledger D-16; SB-CLY-047 |
| 6.C6 | Geolog's resistivity guard adequacy — **closed as inadequate**, by reading L98–L115 rather than trusting the 1999 history line | ADOPTED | SB-CLY-016 — SandiBumi's guard must be pre-branch, which Geolog's is not |
| 6.C7 | The Stieber numbering collision — **closed and fully mapped** across all three vendors from primary artefacts | ADOPTED | SB-CLY-002, -003; ledger D-03 |

### 8.10 OPEN items (§6) — 17 rows

All seventeen are reproduced with their closing conditions in §7.1; the dispositions are recorded
here for the gate.

| # | OPEN item | Disposition | Where |
|---|---|---|---|
| 6.01 | Geolog M–N line constant `0.308` vs `0.388` | ESCALATED | §7 E1; SB-CLY-025; R1 |
| 6.02 | `LARINOV3` uncited, violates `Vsh(1) = 1` by 13 % | ESCALATED | §7 E2; SB-CLY-006 — and it **ships today** at `modules.rs:559` |
| 6.03 | Techlog sonic-density denominator double-minus | ESCALATED | §7 E3; SB-CLY-022; R4 — the requirement does not depend on the answer |
| 6.04 | Techlog `VSH_FINAL` clip order — undocumented | ESCALATED | §7 E3; SB-CLY-027 decides on its own merits |
| 6.04a | Techlog's Median / Harmonic mean / Minimum defined **twice, differently**, in the same shipped tree | ESCALATED | §7 E3; SB-CLY-029 settles SandiBumi's behaviour independently |
| 6.05 | IP's `Lateral` average: text says pair **products**, the named estimator uses pairwise **averages** | ESCALATED | §7 E3/E4; SB-CLY-028 — the pseudomedian is adopted on its bound-preservation, not on the disputed reading |
| 6.06 | **No tool cites a primary source** for the Larionov, Clavier or Stieber GR transforms | ESCALATED | §7 E4; R8 — the Thomas & Stieber 1975 lead is explicitly not accepted as the closure |
| 6.07 | `Percentile Clean` default absent from the CHM and the `.hlp` | ADOPTED as a finding — confirmed to have **no vendor answer** | §5 row 5.2-17 = `ABSENT — ships with no default`; ledger D-14 |
| 6.08 | `Gr Kerogen` default absent from every IP source | ADOPTED as a finding — **no vendor answer** | §5 row 5.2-46 |
| 6.09 | `Clay Shale Ratio` default absent from every IP page, compounded by the Halliburton `CLSR ≈ 0.6` form which is a **different equation** | ADOPTED as a finding — **no vendor answer** | §5 rows 5.2-47/48; SB-CLY-044 |
| 6.10 | Techlog's `POTA unit` "Kcl effect" flag — one sentence, no equation | ESCALATED | §7 E3; affects SB-CLY-023 (P3) only |
| ~~6.11~~ | ~~Techlog EM-propagation page~~ — **closed in the revision pass**, number retired not reused | EVIDENCE-ONLY | Duplicates row 1.4-a and 6.C2; the dossier retains it as a worked example that an item deferred "for scope" cost less to close than to carry |
| 6.12 | Techlog's Vcl (clay, not shale) path | DEFERRED (P2; trigger: the Quanti.Elan read, which needs no vendor session) | SB-CLY-044, -046; §7 OPEN 12 |
| 6.13 | Is Techlog's Curved method actually selectable? | ESCALATED | §7 E3; SB-CLY-008 is implemented regardless — the answer changes how much the corroboration is worth, not whether the transform ships |
| 6.14 | Does the ingest-report transcription defect appear in slices outside this domain? Partially closed here — **all sixteen** clay-volume equation images re-opened and all matched | DEFERRED (corpus-wide, not a `CLY` item; trigger: the cross-chapter verification pass) | The dossier calls it the highest-value follow-up in the list; it is outside this chapter's boundary (§1) |
| 6.15 | Which of Techlog's **two** shipped values an untouched Quanti method uses | ESCALATED | §7 E3; §5 ⚠ TWO VENDOR VALUES rows; neither is a preset until it resolves |
| 6.16 | IP's `Vker`/`Vhvy` ↔ `Rho_dry_rock` fixed-point iteration — no starting value, order or tolerance given | ESCALATED | §7 E3; SB-CLY-049 refuses to iterate, making this a scope boundary rather than a blocker; R6 |

### 8.11 Critique disposition (§8 of the dossier) — 15 rows

The dossier's own `## Critique disposition` is authoritative over any body text it corrects, and
this chapter is written against the corrected text throughout. **15 of 15 findings acted on; 0
deferred; 0 rejected outright.**

| # | Critique finding | Dossier disposition | How this chapter carries it |
|---|---|---|---|
| B1 | **Blocker** — Techlog ships a second, conflicting set of Vsh **endpoints** in `C2_method_defaults.json`, the same file the dossier already cited | FIXED | The single most load-bearing correction in the chapter: it creates §1.3 defect 4, the ⚠ TWO VENDOR VALUES rows in §5, the 41.5 % worked case in SB-CLY-050, and the withdrawal of three as-built defaults in §5.1 |
| M1 | §1.2's claim that Geolog computes no Vcl is false and self-contradicting; the grep evidence named the wrong file | FIXED | `VCL_NMR` is carried as real and typed — SB-CLY-026, row 5.1-24 |
| M2 | §2.12 attributed an endpoint picker to Geolog that no Geolog source states | FIXED | SB-CLY-037's rationale rests on IP alone; the chapter does not claim Geolog parity for picking |
| M3 | The "Techlog Median drops zeros" finding rests on one page while another page says otherwise | FIXED | Recorded as OPEN 4a and escalated (E3); SB-CLY-029 is written so the answer cannot change SandiBumi's behaviour |
| M4 | §1.3 defect 2 ("First Present is inverted") is overstated — the correct definition is on another page in the same tree | FIXED | Row 1.3-d2 carries the corrected, low-severity reading; the chapter does not use it as evidence of a vendor equation defect |
| M5 | Compaction — IP's parameter-linking machinery and per-zone `Use`-flag nulling were dropped from the exact sections that needed them | FIXED (§2.10a added) | SB-CLY-020, -030, -033; §5 rows 5.2-54 … 5.2-57; SB-CLY-T23 |
| m1 | Line-number slip, §2.3 — two history quotations cited as `vsh_gr.lls` L7–8 | FIXED (L6 and L7) | No line citation in this chapter derives from the slipped pair |
| m2 | Silent correction inside a "verbatim" quotation, §2.5 — a vendor typo was tidied | FIXED | The chapter quotes vendor strings as-printed, including `"Default value is189 uSec/ft"` in §5 row 5.2-33 |
| m3 | The `0x96` closure was "an inference dressed as byte evidence" | FIXED — three independent evidential steps, none typographic | Row 6.C1; the closure is treated as evidence, not as a reading |
| m4 | Underivable count, §2.12 — "four modules out of six" | FIXED from the `.info` INPUT rows | SB-CLY-041's alias-preference claim rests on the corrected count |
| m5 | Missed variants in the Techlog tree; `dB/m` absent from §2.15's "complete" unit table | FIXED, and it closed OPEN 11 | Row 5.4-02; SB-CLY-024, -054 |
| m6 | Uncited and out-of-scope rows in §2.15 (`CEC/Qv`, `Salinity`) | FIXED — §2.15 now carries an Evidence column | Neither appears in this chapter; both belong to `SAT`, declared out of boundary in §1 |
| m7 | Verification coverage overstated for IP — every IP *equation* was still T2 | FIXED — the T1″ tier exists because of this | The front-matter tier note and the T1″ ≻ T2 rule; R5 |
| m8 | Weak source string, §5.2 — the IP witness for `dt_fluid = 189` was a bare tier tag | FIXED — cited to the specific CHM page | §5 row 5.2-33 carries the page and the as-printed quotation |
| m9 | Assorted dropped detail — non-sandstone alternates, the two-sided bad-hole idiom, the `Res Clean` picking convention | FIXED | §5 rows 5.2-31, 5.2-32, 5.2-34 (alternates); SB-CLY-035 and row 5.2-58 (two-sided idiom); §5 row 5.2-37 (picking convention as text, per SB-CLY-042) |

### 8.12 Disposition summary

| Disposition | Rows |
|---|---|
| ADOPTED | 190 |
| ESCALATED | 15 |
| EVIDENCE-ONLY | 12 |
| DEFERRED | 8 |
| REJECTED | 0 |
| Dossier-internal (critique dispositions, carried) | 15 |
| **Total** | **240** |

**Nothing in this chapter is REJECTED.** Two things that read like rejections are not: the four
combiners the dossier marks NOT OFFERED (row 5.1-27) are adopted *as a refusal*, which is a
requirement (SB-CLY-028); and the M–N indicator (row 2.9) is escalated, not rejected — it is absent
pending E1, and SB-CLY-025 makes the absence a recorded decision rather than an omission.

**Tier-C boundary check.** Nothing in this domain touches a Tier-C item. No Experienced Eye/EEFS,
Domain Transfer Analysis, Omovie Sonic Saturation, entropy image speed-correction, neural-network
weight DLL, Textural Facies `Freq_Tiles` encoding or frequency-domain dispersion fit appears in the
clay-volume chain, and no requirement above is a Tier-C item under another name. No vendor chart
lookup-table data was transcribed: the only chart referenced is the M–N chart in E1, cited by
existence, attribution and purpose only. No `.neu`, `.ovl`, `.itt`, `.itp`, `.att`, `.bor` or `.eli`
content was read or reproduced.
