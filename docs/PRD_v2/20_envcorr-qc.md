# 20. Environmental corrections and log QC — requirements

**Source dossier:** `docs/research_2026-08/cross_tool/envcorr-qc.md` (2,841 lines), including its
method inventory (§1, with the 126-module / 10-family Geolog enumeration at §1.3.1 and Techlog's
own per-tool coverage table at §1.2.1), its equation comparison (§2, eighteen subsections), its
"differences that matter" analysis (§3, ten quantified divergences), its optimal-choice
dispositions (§4, fourteen items), its adoption spec (§5, including the test suite at §5.4 — numbered
`T-1`…`T-38` but containing **39** rows, see §8.0 — and
the chart-lookup interface contract at §5.3), its gaps and escalations (§6 — `O-1…O-15`,
ledger `F-1…F-15`, escalations `E-1…E-13`), its source register (§7), its Compliance statement,
and its authoritative `## Critique disposition` (`BLK-1…2`, `MAJ-1…10`, `MIN-1…13`).

**Evidence tiers held in this domain:** **T1** (Geolog V14 `.lls` Loglan sources and — the finding
that carries this chapter — the `bin\*.info` declarative manifests, read directly; Techlog 2018.2
shipped Python and its Toolbox scripts; SandiBumi's own Rust and TypeScript), **T2** (IP2018 and
IP2025 full-manual CHM ingest reports, `IP25-F` slice reports and `DISCREPANCIES.md`), **T3**
(vendor doc pages, install-tree catalogues, equation and dialog rasters), **T4** (house QC gates,
memory atomics, project-kb delivered-study records).

**Tier note.** This domain's decisive evidence is **T1-declarative**, not T1-executable, and the
distinction is the chapter's whole argument. Geolog's environmental corrections earn their
"fail-loud" reputation in the `.info` **manifests** — the VALIDATION columns that state each
correction's stated range — not in the `.lls` algorithm sources. Four earlier passes read the
sources and missed the manifests entirely. A port that lifts the algorithm without the manifest
inherits a **fail-silent** copy of a fail-loud product, and it will pass every numerical
regression test written against it. `03_EVIDENCE_BASE.md` §12.1 calls this the single most
transferable finding in the corpus; `SB-CORE-003` exists because of it, and this chapter is its
primary owning chapter.

**Author date:** 2026-08-07.
**Requirements:** 58. **P0:** 23. **Parameters:** 83 rows (32 ship `ABSENT — ships with no default`;
16 are `NON-ADOPTABLE — cited for verification`; 29 are recorded `SHIPPED-UNCITED`).
**Acceptance tests:** 70 (`SB-ENV-T01`…`SB-ENV-T70`, three of them build gates).
**Dossier items dispositioned in §8:** **178 of 178.**

*(These figures were re-counted from the finished tables at the end of the run. The pre-drafting
estimates in an earlier version of this line — 19 P0, 71 parameters, 44 tests, 166 dossier items —
were all low. They are recorded here rather than silently replaced, on the same principle §8.0
applies to the dossier's own miscount: a number that moved is evidence about how the document was
built, and hiding the movement is the smaller version of the defect this chapter is about.)*

*(**Post-authoring verification pass, 2026-08-07.** Headline claims were re-derived against source
after the chapter was written. Two survived unchanged: the dual-`FTEMP` divergence of §2.7 / `SB-ENV-043`
(86.7 °C from `ftemp_grad`'s metric defaults against 119.8 °C from `precalc`'s feet-based fits at the
same 2 000 m TVD, propagating to 14.3 % relative on `Sw` through `Rw(T)`), and the `ArgSpec`
enforcement-field argument of §2.2. Two did not. §2.5 stated the despike masking direction backwards
in two places while stating it correctly in two others, and attached IP's `1/(k²+1)` to the Hampel
estimator SandiBumi actually ships — corrected, with `min(1/k, ½)` derived for the shipped fallback
branch, `ESC-16` raised on the `K = 3.0` default, and tests `T69`/`T70` added. `SB-ENV-047` rested on
a misread of a two-branch `if` and is corrected from `PRESENT-DIVERGENT` to `PRESENT-OK`. Both
corrections are recorded in place rather than rewritten away.)*

**Delegation statement.** One read-only subagent (haiku) was used at the outset to inventory which
files in the repository hold this domain, and nothing else. Its output was treated as a map, not as
evidence: **every `file.rs:line` pointer below was re-opened and re-verified in session**, and the
inventory's line ranges were wrong often enough to justify that — it reported `badhole_spec` at
`modules.rs:1183–1204` where the parameter block this chapter quotes sits at `1195–1197`. Every
equation, coefficient, threshold, unit conversion and arithmetic result below was derived on the
session model, per the standing rule that petrophysical parameters and method math are never
delegated.

---

## 1. Scope and boundary

This chapter owns **everything that happens to a log between the field tape and the interpretation
model**, and the machinery that decides whether the result of that interpretation may be believed.
Concretely:

- **Environmental corrections** — the borehole, mud, temperature, pressure, salinity, standoff and
  mudcake corrections applied per tool and per vendor family: gamma-ray hole-size correction, the
  neutron correction chain, density hole-size and mudcake correction, and the resistivity
  temperature correction that turns a surface `Rw` into a formation-temperature `Rw`.
- **The correction-chart interface** — how a correction implemented as a chart lookup declares its
  tabulated span, interpolates inside it, refuses or clamps outside it, and records which of those
  happened. Not the chart *data*; see the boundary statement below and §7.3.
- **Validity conditions as enforced preconditions** — the declarative mechanism by which a method
  carries its own stated range and the runner evaluates it before computing. This is
  `SB-CORE-003`, and this chapter is its primary owning chapter.
- **Hole-condition and data-condition flagging** — bad-hole detection from `DRHO` and caliper, the
  coal / tight / gas-crossover / shoulder conditioning flags, flag polarity as a single-source
  convention, and the universal run mask that consumes them.
- **Curve conditioning** — despiking, smoothing and filtering, clipping, gap filling, polarity
  flipping, outlier and spurious-population culling, and the conditioning **order** as a checkable
  contract rather than a convention.
- **Log normalisation** — the two-point percentile map, the reference-pair discipline, and the
  order-statistic-versus-histogram-bin question that decides whether the map is right.
- **Formation temperature and the geothermal gradient** — the linear-geotherm and mudline branches,
  the four unit conventions they are stated in, and the single-definition obligation on the
  temperature curve every downstream saturation model consumes.
- **Depth control** — depth shifting and its provenance, resampling and reframing, blocking and
  regularisation, and the depth-unit consistency half of `SB-CORE-001` that lives on the *operator*
  rather than on the parser.
- **Per-tool uncertainty** — the `unc_*` twin of each correction, which converts a corrected curve
  into a corrected curve *with a stated error*, and which no other incumbent ships.
- **Results QC** — the cross-method spread, reconciliation and sensitivity machinery that tells an
  interpreter whether the model choice changed the answer, and the honesty rules that machinery must
  satisfy.

### 1.1 Named seams

Each seam is declared here rather than discovered at index time.

**Data import, parsing and formats (`DIO`, `21_data-io.md`).** `DIO` owns reading the file, parsing
its headers, and the **parse-and-carry** half of `SB-CORE-001`: establishing that a depth column is
in feet or metres at the moment the file is read, and attaching that fact to the curve. This chapter
owns the **consume-and-enforce** half: every operator here that takes a *thickness* — a despike
window, a fill-gap limit, a shoulder width, a minimum bed thickness, a depth shift, a resampling
increment — must resolve that thickness against the carried unit and refuse rather than assume. The
seam is exactly where the number stops being metadata and starts multiplying. `DIO` also owns null
representation on the wire (`-999.25` and its variants); this chapter owns null *semantics* inside
an operator (§2.10 of the dossier), which is a different obligation: `DIO` must make the sentinel a
NaN, and this chapter must guarantee no operator ever writes a value across one without saying so.

**Plotting, display and interactivity (`PLT`, `23_plotting-interactivity.md`).** `PLT` owns the
**display** of everything this chapter produces: the QC dashboard's layout, the histogram overlay
used to check a normalisation across wells, the crossplot the chart overlays are drawn on, the
interactive depth-shift drag, and the rendering of flag curves as shaded intervals. This chapter
owns what those displays are **required to show** and what they may never omit — a corrected curve
that was not corrected, a masked interval, a filled gap, a clamped chart lookup, a refused
precondition. The rule at the seam: **this chapter specifies the obligation, `PLT` specifies the
pixels.** Where a requirement below says "the user MUST be able to see", it is stating an
information obligation that `PLT` discharges; where it says "the panel MUST show", the requirement
is jointly owned and is marked as such.

**Porosity (`POR`, `11_porosity.md`).** The gas / light-hydrocarbon correction is **`POR`'s**, not
this chapter's, and the line is drawn by what the correction is a function of. A correction driven
by the *borehole* — hole size, mud weight, mud salinity, temperature, pressure, standoff, mudcake —
is environmental and lives here. A correction driven by the *formation fluid* — the Poupon `A`/`B`
light-hydrocarbon factors, the hydrocarbon apparent hydrogen index, the neutron excavation effect —
is a porosity method and lives in `11_porosity.md`. SandiBumi's shipped `gascorr` module
(`modules.rs:1629-1795`) is on `POR`'s side of that line and is not specified here; it appears in
this chapter only where it is a *consumer* of something this chapter owns — it reads the `precalc`
formation-temperature curve, and it is the module the runner's own progress reporting singles out
as producing an all-missing output when `precalc` has not been run (`workflow.rs:655-658`).

**Multi-mineral solver (`MIN`, `13_mineral-solver.md`).** `MIN` owns the tool-response endpoints and
the neutron **matrix** response of each mineral. This chapter owns the neutron **matrix scale** as a
*declared property of a curve* — the fact that a neutron curve is on a limestone, sandstone or
dolomite scale, that it must be stated, and that mixing scales silently changes a number. Dossier
§3.7 records the same trap in all three tools; SandiBumi's own `condflag` documentation
(`modules.rs:1261-1264`) quantifies it at about 0.04 v/v in clean water sand, which lands exactly on
its own `XOVER_MIN` default of 0.04 v/v (`modules.rs:1286-1293`). The **conversion arithmetic**
between scales is `MIN`'s; the **obligation to declare the scale and refuse a mismatch** is this
chapter's, because it is a validity condition and validity conditions are `SB-CORE-003`.

**Machine learning and prediction (`MLA`, `24_ml-advanced.md`).** Curve *repair by prediction* —
IP's `curve_autoedit` regression bank, and SandiBumi's `log_predict` — is a modelling capability and
belongs to `MLA`; dossier §4.14 dispositions `curve_autoedit` the same way. This chapter owns the
seam that broke: the interaction between the universal run mask and a repair module, which is a
**runner** defect, not a modelling one, and is specified here (`SB-ENV-027`) because the mask is
this chapter's.

### 1.2 What this chapter deliberately does not contain

**No vendor correction-chart data, in any form.** `CONTRACT.md` §2.1 bars transcription of vendor
chart lookup-table data and names `.neu` / `.ovl` chart tables explicitly. This chapter is the
highest chart-transcription-risk chapter in the set after `MIN`, and the bar is treated as absolute:
the chapter describes **which corrections a vendor ships, for which tool families, in which
edition, under which stated validity conditions, and with what coverage gaps** — that is capability
intelligence and it is wanted — and it carries **no correction coefficient, no chart node, no
tabulated pair and no reconstructed polynomial** from any vendor chart. Where the temptation arose,
it is recorded as a numbered refusal in §7.3 rather than resolved silently. `CONTRACT.md` §2.1's one
recorded exception is not reasoned from and is not extended.

**No extension of R1.** `02_RISKS_AND_CONTRADICTIONS.md` R1 records that two shipped SandiBumi
assets declare in their own headers that they are digitized from a 2013 vendor chartbook —
`src/ui/chartOverlays.ts` and `src-tauri/src/neutron_charts.rs`, whose header states the source
edition and charts outright (`neutron_charts.rs:1-22`) — and `IP_PROVENANCE.md` §2.1 calls this
"the single most exposed item in the product". This chapter **cites that exposure, specifies
requirements around it, and states what would reduce it**. It adds no digitized chart, proposes no
new digitization, and specifies no requirement whose only possible implementation is a larger
transcription. Requirement `SB-ENV-015` deliberately specifies the chart-lookup *interface* — span,
interpolation, clamp, refusal and flag — in a form that is complete and testable **without any chart
data existing**, so that the interface can ship, be verified, and be pointed at a
user-supplied or independently-derived table later.

**No Tier-C reconstruction.** Per `CONTRACT.md` §2.2 as amended 2026-08-07, the prohibited thing is
the derivation *path*, not the capability. One Tier-C item falls in this domain — **entropy-based
borehole-image speed correction** — and it is handled in §7.4 as an independent-derivation
requirement with its class, its primary-source position and its `Betters:` line, not as a refusal.

---

## 2. What the incumbents do — the requirement-bearing findings

Fifteen findings. Each generates at least one requirement in §4. Findings from the dossier that
generate no obligation are accounted for in §8 as `EVIDENCE-ONLY` and are not padded into here.

### 2.1 Geolog's fail-loud reputation lives in the manifests, not in the code — and a port that reads only the code inherits a fail-silent copy

**Tier T1-declarative. Tools: Geolog V14 (`bin\*.info`, `loglan\*.lls`).** Dossier §1.3.1, §2.6.1.

Geolog ships **126 environmental-correction modules across 10 vendor tool families**. That inventory
is not discoverable by reading the algorithm sources: it is discoverable only by enumerating the
`bin\*.info` **manifests**, which is why four earlier passes over the same install reported a much
smaller correction set. The manifests are the product's declarative layer — each one states, per
parameter, the parameter's unit, its default, and its **VALIDATION** range, and the module runner
evaluates that column before the Loglan body executes.

The consequence is the whole reason this chapter exists. **A correction's stated validity range is
not in the algorithm. It is in the manifest that sits beside the algorithm.** An engineer porting
`evs_tnph` by reading `evs_tnph.lls` gets arithmetic that is numerically identical to Geolog's and
behaviourally opposite: Geolog refuses, or clamps and says so, at the edge of the chart; the port
extrapolates a two-segment linear fit off the end of a chart into a region the vendor never
measured, produces a finite number, and reports success. **Every numerical regression test written
against in-range data passes.** The failure only appears in the wells where it matters — the hot
ones, the washed-out ones, the ones with salt-saturated mud — and it appears as a plausible curve.

`evs_tnph.info:75-83` (T1) is the worked case: the module's ten correction steps are individually
switchable (`OPT_BSCO`, `OPT_MWCO`, `OPT_PTCO_T`, `OPT_PTCO_P`, `OPT_FSCO`, `OPT_CACO`,
`OPT_HSCO`, `OPT_SOCO`, …), each carries a default, and the temperature step declares its input
`FTEMP` in **degF** with a clamp of **50–300 °F**. The unit and the clamp are both manifest facts.
Neither is in the `.lls`.

**A second manifest fact the code contradicts.** On re-verification the mud-weight clamp splits:
the header documents the chart axis as **8–18 lb/gal** (`unc_tnph.lls:68`) while the code clamps the
**normal-mud** branch to **8–13** and the **barite** branch to **8–18** (`unc_tnph.lls:340, 346`).
A single documented range would have been wrong for one of the two branches. Validity is
**per-branch**, not per-module — which is a structural requirement on the mechanism, not a detail.

**Obligation.** `SB-ENV-001` … `SB-ENV-008`. This is `SB-CORE-003`'s primary discharge.

### 2.2 Geolog's neutron uncertainty is an under-estimate of its own correction, by construction, and nothing in the output says so

**Tier T1. Tools: Geolog (`unc_tnph.lls`, `evs_tnph.info`), IP 2025, house QC gate (T4).** Dossier
§2.6, §2.6.1, §3.9.

Geolog's neutron chain is the most readable end-to-end correction chain in the corpus and it splits
into a **twin pair**: `evs_tnph` applies the correction, `unc_tnph` computes its uncertainty. The
applying module ships **all ten** steps enabled by default. The uncertainty module has **five of the
ten commented out in shipped source** (`unc_tnph.lls:317-429`) — borehole salinity, mud weight,
borehole temperature, pressure and formation salinity — along with its final `TNPH_COR` assembly
(`:514-521`). It computes an uncertainty over three live steps for a curve corrected with ten.

**The output states neither number's basis.** A user reads `TNPH_COR` and `TNPH_COR_UNC` side by
side and has no way to learn that the second is an envelope over a strict subset of the first. This
is the vendor instance of the corpus's own FINDINGS §6 rule 14, and it is the strongest available
argument for `SB-ENV-005`: **a corrected curve must carry the list of steps actually applied, and an
uncertainty must be computed over that same list and declare it.**

**Quantified, from the house gate (T4, `memory\reference_log_qc_gates.md` §Neutron).** Two of the
five disabled effects are independently quantified in house standards:

| Effect | House-gate magnitude | `unc_tnph` (uncertainty) | `evs_tnph` (correction) |
|---|---|---|---|
| Borehole temperature | apparent φn drops **~2.4 p.u. per 50 °F** (≡ −0.048 p.u./°F) | **commented out** — no `sig` term | `OPT_PTCO_T` = **yes** |
| Standoff | **0.5 in standoff in a 12¼″ hole makes NPHI read ~2 p.u. too high** | enabled | `OPT_SOCO` = **yes**, `SOCN` default **0.5 in** |

Read the second row twice. **The vendor's shipped standoff default is exactly the house gate's
worked warning case.** A user who accepts Geolog's defaults has not chosen a standoff; they have
inherited one worth about 2 p.u. against carbonate `PHIE` means of 0.05–0.07 v/v in delivered
work (T3) — a correction of the same order as the entire porosity being measured. That is the
argument for `SB-ENV-016`: where a shipped default materially moves the answer, the default must
appear in the chain manifest next to the value actually used, or it must not ship at all.

**Obligation.** `SB-ENV-005`, `SB-ENV-012`, `SB-ENV-016`, `SB-ENV-019`.

**Explicit non-licence.** The commented-out blocks contain two-segment linear fits to a vendor
chart. Those coefficients are chart-derived data, they are not reproduced in the dossier, and they
are not reproduced here. Refusal `RF-1`, §7.3.

### 2.3 One method name, two different laws — the `Arps`-labelled resistivity temperature correction

**Tier T1 (Techlog Toolbox Python, Geolog `.lls`), T2 (IP).** Dossier §2.1, §3.1, §4.1.

Techlog's `Exxon` branch and Geolog's shipped constant are **provably the same law in two unit
systems**: `Rw₂ = Rw₁ · (T₁ + c)/(T₂ + c)` with `c = 6.77` °F ≡ `c = 21.5` °C. Two independent
implementations agreeing is strong evidence. A third branch in `TempCorr_Resistivity.py`, **labelled
`Arps`**, uses `c = −6` °F and stands alone.

It is worse than a third option, because of *how* it is reached: it is the **fall-through** for any
`Method` string that is not exactly `'Exxon'` (`TempCorr_Resistivity.py:80-83`, T1). A typo, a
locale, a parameter set imported from elsewhere, or an unrecognised method name does not error — it
silently selects the outlier law.

Quantified at `tref = 200 °F`, `Rw₁ = 0.1` ohm·m:

| Measured at | Classical (`T+6.77` °F) | `Arps`-labelled (`T−6` °F) | Δ on Rw | Δ on Sw (Archie, `Sw ∝ √Rw`) |
|---|---|---|---|---|
| 60 °F | 0.03229 | 0.02784 | **−13.8 %** | **−7.2 %** — Sw 0.350 → 0.325 |
| 80 °F | 0.04196 | 0.03814 | −9.1 % | −4.7 % — Sw 0.350 → 0.334 |
| 150 °F | 0.07582 | 0.07423 | −2.1 % | −1.1 % |

The error is **largest at low measurement temperature**, which is precisely where surface `Rw` and
`Rmf` references are quoted. Two saturation units on a commercial deliverable is material, and it
arrives without a warning of any kind.

**Scope, stated precisely.** These numbers size what happens *if the `Arps` branch runs*. Techlog's
compiled production environmental-correction engine uses a constant that is not visible in any
evidence held (escalation `E-12` in the dossier; carried forward here). The operational risk is
therefore narrow and exact: anyone running or adapting that Toolbox script, or importing a parameter
set naming `Arps`, gets the outlier form by default and silently.

This is `SB-CORE-006` (one name, one equation) and `SB-CORE-007` (one definition per constant) in a
vendor product, and it is the model for both `SB-ENV-048` and the general refusal in `SB-ENV-018`:
**a method-selection string that does not match a known method MUST be an error, never a
fall-through to a default branch.**

**Obligation.** `SB-ENV-048`, `SB-ENV-049`, `SB-ENV-018`.

### 2.4 The bad-hole cutoff is a 1000× unit trap in one direction and a silent no-op in the other — and the house has no consensus value

**Tier T1 (Geolog `badhole.info`, `unc_ldt.lls`), T2 (IP), T3/T4 (seven delivered studies).** Dossier
§2.2, §3.2, §4.2.

**The unit trap.** Geolog declares `DRHO` and `DRHO_MAX` in **kg/m³** (`badhole.info:30,33`, T1); IP
declares `DRHO` in **g/cc** (T2). `0.1 g/cc = 100 kg/m³`. An IP-shaped `0.1` entered into Geolog's
field means `0.0001 g/cc` and **every sample flags bad hole** — loud, and therefore survivable. The
reverse, a Geolog-shaped `100` entered into a g/cc field, means `100 g/cc` and **nothing ever
flags**. That is the direction that ships: an interpreter sees no bad-hole intervals and concludes
the well has none.

**Geolog is internally inconsistent on the same parameter name.** `badhole.info` declares `DRHO_MAX`
in `k/m3`; `unc_ldt.lls:59,98` declares a same-named `DRHO_MAX` in `G/C3`. Geolog's per-argument unit
engine makes this safe *inside* Geolog. Any importer, exporter or parameter-file bridge that keys on
the **name** — which is what every such bridge does — gets it wrong by 1000×.

**No house consensus on the value.** Seven delivered studies (project-kb, T3) span **DRHO > 0.02 to
DRHO > 0.15 g/cc — a 7.5× range** — while the caliper cutoff is consistently **≈ 2 in**, and **three
of the seven use no DRHO term at all**. IP ships `|DRHO| > 0.1 g/cc`. The ITB module gate states
`DRHO > 0.15 g/cc`. There is no defensible single default, and shipping one dresses a single field
calibration in the authority of a manifest.

**Why three studies drop DRHO is availability, not preference, and it changes the requirement.** One
brownfield study logged **DRHO on 3 wells out of 362**, against RHOB on 358 and caliper on 355 (T3).
A bad-hole rule with DRHO as a *required* term is inapplicable at that scale. The obligation is
therefore not "require both inputs" but **degrade to whichever inputs exist and say in the output
which terms were evaluated** — which Geolog already does correctly, branching on `CALI_POR` and
`DRHO` separately (`badhole.lls:88-101`, T1).

**Obligation.** `SB-ENV-021` … `SB-ENV-026`. Parameter dispositions in §5.

### 2.5 IP's despike estimator has a closed-form failure threshold, a looser cutoff lowers it, and the user is never told

**Tier T2 (IP), T1 (Techlog, Geolog). Derivation performed in the dossier and re-derived here.**
Dossier §2.3, §3.3, §4.3.

Three tools ship a "despike" and no two run the same algorithm. IP's is a **mean ± kσ** window test
with a shipped `SpikeCutoff = 2` SD. Techlog's runs **MAD** — and runs it on the **first difference**,
which catches a step-spike that a value-domain test cannot see. Geolog's adds the **angular** case
and emits a reversible removed-value flag. `SB-CORE-006` applies directly: three algorithms, one
name.

The requirement-bearing part is IP's failure mode, because it fails under exactly the conditions
despiking is deployed for. Let `f` be the contaminated fraction of the window, `d` the spike offset,
`k` the SD multiple, clean scatter zero. Then `mean = μ + f·d` and `σ = d·√(f(1−f))`, so a spike at
`μ + d` escapes its own rejection band when

```
f·d + k·d·√(f(1−f)) ≥ d    ⇒    k²f ≥ 1 − f    ⇒    f ≥ 1/(k² + 1)
```

Three consequences, and the third is the one that must reach the user:

1. **At `k = 2` the threshold is exactly 1/5 = 20 %.** Not approximately — exactly. IP's estimator
   has a **0 % breakdown point** in the formal sense; MAD's is **50 %**.
2. **The threshold is independent of spike amplitude `d`.** A 10 g/cc spike masks at the same 20 % as
   a 0.1 g/cc one, because it inflates σ in exact proportion. Bigger spikes do not help.
3. **The threshold is set entirely by `k`, and it falls as `k` rises.** `k = 1.5` → 30.8 %; `k = 2` →
   20 %; `k = 3` → 10 %. The trap is not that the direction is inverted — it is that **the breakdown
   dial and the false-positive dial are the same dial, pulled opposite ways.** A user raises `k` to
   stop the filter eating good samples, which is the cautious-looking move, and pays for it in
   contamination resistance: at `k = 3` a mere 10 % of the window defeats the test outright. Nothing
   on screen prices that trade, and the failure leaves no mark in the output.

Under the sample-σ convention the threshold becomes `1/(1 + k²N/(N−1))` = **19.19 %** at `k = 2,
N = 20` — *lower*, because a sample σ is larger on the same data, so the band is wider and masking is
easier. Which convention IP uses is not established in evidence held (dossier `O-15`); the direction
of the sensitivity is settled and does not rescue the estimator.

Operationally: **masking begins at 19–20 % contamination in the best case** — clean population at
exactly one value with zero scatter — and earlier once real scatter is admitted. On a washout-riddled
log, 20 % local contamination inside a 10 ft window is entirely ordinary.

**The shipped estimator has its own ceiling, and it is not this one.** SandiBumi despikes with a
Hampel filter — window median, scale `1.4826 × MAD`, `K` defaulting to 3.0 (`condition.rs:154-172`,
`:253-256`) — so `f* = 1/(k²+1)` does **not** describe it. Printing that number beside SandiBumi's
`K` would state a ceiling the code does not have. Three branches, three thresholds:

| Branch | Condition | Ceiling | At the shipped `K = 3` |
|---|---|---|---|
| True MAD | clean scatter > 0 ⇒ `MAD > 0` (`:166-169`) | **50 %** — the breakdown point of median and MAD alike, for any spike clearing `k·σ_clean` | 50 % |
| Mean-deviation fallback | clean population at one value ⇒ `MAD = 0` (`:170-171`) | `f* = min(1/k, ½)` | **33.3 %** |
| IP's `mean ± kσ` — not shipped here | — | `f* = 1/(k²+1)` | 10 % |

The fallback derivation: with the clean population at one value, `MAD = 0` for any `f < 50 %`, so
`window_spread` returns the **mean** absolute deviation `f·d` (`:170`) and the band is `k·f·d`. A
spike at distance `d` escapes when `d ≤ k·f·d`, i.e. `f ≥ 1/k`. Past 50 % the median itself moves
into the spike population and the filter begins destroying the clean samples instead, so the true
ceiling is `min(1/k, ½)` — the median's own breakdown point is the wall that `1/k` cannot cross.

Two consequences the incumbent analysis does not reach:

- **The trade only begins above `k = 2`.** `min(1/k, ½)` is flat at 50 % for every `k ≤ 2` and falls
  only above it. Below the knee, `k` is free on the robustness axis and costs only false positives;
  above it, `k` is spending breakdown resistance as well. **SandiBumi's shipped `K = 3.0` sits above
  the knee** — 33.3 % where 50 % was available — and the code comment at `:253-255` states plainly
  that 3.0 is "the ordinary three-deviation convention … NOT a field calibration". Escalated at
  `ESC-16`; not changed here, because a despike cutoff is a parameter and parameters are cited or
  asked about, never adjusted because an analysis made a different number look attractive.
- **The fallback is 3.3× better than IP at the same `k`, not a concession.** The comment at
  `:145-148` already argues the fallback is "less resistant to a second spike inside the same
  window" and calls the trade worth making. `f* = 1/k` is what "less resistant" is worth: 33.3 %
  against IP's 10 % at `k = 3`. The code's qualitative claim is now quantified, and it holds.

`SB-ENV-031` therefore requires the ceiling of **the estimator that will actually run** — not one
constant. A dialog reading "ceiling 10 %" over a Hampel run that holds to 33 % is a wrong number
wearing the authority of a computed one, which is `SB-CORE-002` in the UI layer.

**Obligation.** `SB-ENV-031` … `SB-ENV-037`, `SB-ENV-018`.

### 2.6 Geolog's normalisation histogram drops its bottom bin, kills its top bin, and can fail to terminate at exactly the percentile the house uses

**Tier T1, code read (`log_normalization.lls:228, 291-345`).** Dossier §2.8, §3.4, §4.6.

Four defects in one routine, and they compound. Writing `step = (log_max − log_min)/BINS`:

**(a) The bottom bin is never counted.** `bin_lim[1]` is initialised **one step above `log_min`**
(`:310`) while the membership test starts at `i = 1` requiring `>= bin_lim[1]` (`:318`). Every sample
in `[log_min, log_min + step)` matches no bin. But the percentile target is computed against **all**
frames (`:326`) while the tally walk accumulates only *binned* counts (`:332-335`), so the walk
reaches its target **late** and the low endpoint is **biased high**. The bias is not bounded at one
bin: it is one bin width when the low tail is dense and larger when it is sparse. At `BINS = 50` over
a GR range of 20–200 gAPI, one bin ≈ **3.6 gAPI**.

**(b) The top bin is dead.** For `i = BINS` the test requires `< bin_lim[BINS+1]`, and
`bin_lim[BINS+1]` was initialised to **0** (`:228`) and never assigned. For any positive-valued log
that test can never be true. `log_max` is excluded and the effective histogram is `BINS − 1` bins.

**(c) The upper walk has no termination guard, and P97 is what makes it reachable.** `:341-344` is a
`dowhile tally <= cnt_max` with no bound on `i`. Because (a) and (b) remove samples from the binned
total, the loop fails to terminate whenever `n_bottom + n_top ≥ (1 − PCT_MAX/100)·frames`. **At
`PCT_MAX = 97` that threshold is 3 % of the data** — and the bottom 1/BINS of a GR range is a
thoroughly ordinary home for 3 % of samples in a sand-rich well. The walk then runs past the last
populated bin into a 999-element array and `:345` evaluates `bin_cum[i]/bin_cnt[i]` on a zero count.
**The house standard's P3/P97 is the exact percentile pair that makes this reachable; Geolog's own
suggested 90–95 is not.**

**(d) Independent of the bug, the returned value is a bin mean, not a percentile.**
`min_pnt = bin_cum[i]/bin_cnt[i]` (`:336`) is the **mean of the bin the percentile falls in**. At
`BINS = 50` the quantisation alone is ±½ bin ≈ **1.8 gAPI** on a 180 gAPI range. This is a resolution
limit rather than a defect, and it is the second, independent reason to specify **exact order
statistics**.

**Stakes, quantified.** For linear GR-index `IGR = (GR − GR_clean)/(GR_shale − GR_clean)`,
`∂IGR/∂GR_clean = (GR − GR_shale)/span²`. At a delivered field's endpoints (`GR_clean ≈ 54`,
`GR_shale ≈ 134`, span 80 gAPI, T3) that is **−0.0125 per gAPI at the clean end**, so a 3.6 gAPI
shift on the low endpoint is **≈ 4.5 % Vsh absolute**, applied field-wide and systematically in one
direction. The sensitivity is maximal at the clean end and zero at the shale end — which is where the
dropped bin lives.

**Two further unguarded limits in the same routine, same class.** `log_val[i]` is written with an
index never tested against its `[99999]` declaration (`:291-293` vs `:159`) — reachable at 0.1 ft
sampling over ~10,000 ft of accepted data, and *coupled to a user parameter*, since the index
advances only for samples inside the cutoff pair. And `BINS` is never checked against the
999-element bin arrays. The requirement this generates is not "use bigger arrays": **a bin count and
a frame count are validated inputs, and exceeding either is an error with a message, never a silent
write past the end.**

**A separate, firm requirement with no causal claim attached.** A delivered carbonate study records
manual post-normalisation adjustment in specific wells where automatic normalisation produced
unrealistically low shale values (T3). The software behind that run is not identified and the
direction does not match this code read, so **it is not evidence for the binning defect** and is not
offered as such. It is firm evidence that **automatic normalisation output must be reviewable and
overridable per well**.

**Obligation.** `SB-ENV-051` … `SB-ENV-055`, `SB-ENV-018`.

### 2.7 Four geothermal-gradient conventions, and the dangerous one is silent

**Tier T1/T2/T3.** Dossier §2.9, §3.5, ledger `F-5`.

Geolog states the gradient in **°F/ft**. Techlog states it in **°C/100 m**. IP states it as
**°/100** with the length unit following the well. IP's EERC page is internally unresolved between
**°C/m and °C/100 m** — its raster divides by 100 while its ASCII text says °C/m (ledger `F-5`,
OPEN).

A gradient of **3 °C/100 m** — an ordinary value for a warm deltaic clastic basin — is
**0.01646 °F/ft**. Enter `3` into Geolog's °F/ft
field and the well heats **182 °F per 100 ft** — absurd, immediate, loud. Enter `0.0165` into
Techlog's °C/100 m field and the well is **essentially isothermal** — no error, no warning, a
perfectly smooth temperature curve, and it propagates into every temperature-corrected `Rw`, every
neutron temperature correction and every resistivity temperature correction downstream.

This is `SB-CORE-001` in its most consequential form, and it is not solved at the parser: a gradient
is a **compound** unit whose numerator and denominator are independently ambiguous, and the number
alone can never disambiguate it. The requirement is a declared unit on the parameter itself.

**Obligation.** `SB-ENV-043` … `SB-ENV-047`.

### 2.8 No tool enforces a conditioning order, and the house order is load-bearing

**Tier T2 (IP), T3 (Techlog), T1 (Geolog).** Dossier §3.6, §4.12.

**IP explicitly declines to impose one** — "the manual states no explicit global ordering for the
wireline/LWD environmental corrections"; execution is per-tab, per-zone and user-sequenced. The
contrast the ingest itself draws is the tell: for *image* corrections the same manual mandates order
**in capitals**. The absence for logs is deliberate, not an oversight.

**Techlog states one local ordering only** — outlier cleaning **before** despike.

**Geolog implies order through data flow** — `badhole.lls` emits a flag that later modules consume,
so the order is real but nowhere declared, and nothing prevents running them backwards.

Against that, the house standard is a **15-step conditioning order** in which the steps are not
commutative: normalising before despiking lets a spike set the percentile endpoints; despiking before
bad-hole flagging deletes the evidence that would have flagged the hole; correcting a neutron before
verifying its matrix scale corrects the wrong curve. A pipeline that permits any order permits every
wrong one, and produces a curve that carries no record of which order produced it.

**Obligation.** `SB-ENV-018`, `SB-ENV-028`, `SB-ENV-053`.

### 2.9 Neutron matrix scale — and the proof that validation must live with the algorithm, not beside it

**Tier T1/T2/T3.** Dossier §2.6, §3.7, §4.7.

The same trap in three tools, handled three ways:

- **IP** — an *unenforced assumption*: "IP makes the assumption that any neutron curve entered is in
  Limestone matrix units. If this is not the case, then the curve should be converted…" Stated on two
  separate pages (T2). **Nothing checks it.**
- **Geolog** — a *declared parameter*, `LITHSCALE ∈ {SANDSTONE, DOLOMITE, LIMESTONE}`, setting
  `tnph_lith = missing` on an unrecognised value **in the code itself** (`unc_tnph.lls:204-224`, T1).
  A real `else`. Fail-loud.
- **Techlog** — a *named method*, "neutron matrix conversion … from a current matrix into a new
  matrix" (T3).

The house 15-step order makes "verify neutron matrix scale" **step 2** and names the failure
explicitly: *log on sandstone but software defaults to limestone = silent error*.

**Now the part that decides the architecture.** "Geolog is fail-loud" is not uniformly true, and the
counter-examples are the single best argument for where validation must live:

| Site | `.lls` behaviour read alone | The `.info` guard | Net |
|---|---|---|---|
| `unc_gr.lls:116-153` | `if/elseif` on `GR_TOOL_SIZE` and `GR_TOOL_POS` with **no `else`**. On an unrecognised value `cor`/`cnom` are never assigned, so **the previous frame's values persist** into `sig_hs = (GR/2)·(cor − cnom)` at `:157` | `unc_gr.info:40-41` `VALIDATION` enumerates the four position spellings and the two tool sizes | rejected before the code runs |
| `ftemp.lls:53` | tests `== 'MEASUREMENT_REFERENCE'` and routes **every** other string — including a typo — to the mudline branch. No third case | `ftemp.info:27-32` `VALIDATION` enumerates the two branches | rejected before the code runs |

**The GR case is the worse of the two, and the reason is reproducibility.** Its failure is
*frame-dependent*: the first frame after an unrecognised value carries the **previous frame's**
correction constants, so the same input produces different output depending on what preceded it. A
wrong constant is at least a wrong constant every time. This is not reproducible, and
non-reproducibility defeats every regression test that would otherwise catch it.

**Therefore: SandiBumi's rule is that validation lives with the algorithm**, so the algorithm is safe
wherever it is called from — including from a chained workflow, a saved run, a zone override or a
future API caller that never passes through a dialog. Geolog's manifest achieves the right behaviour
by the wrong construction; the correct construction puts the enumeration where the branch is.

**Obligation.** `SB-ENV-001` … `SB-ENV-004`, `SB-ENV-013`, `SB-ENV-029`.

### 2.10 IP's shipped log-QC limits would flag most real logs, and its extreme band inverts its user band

**Tier T2 (IP2025 `IP25-F` §4.1, §5.9; ledger `F-1`).** Dossier §3.8, §4.13.

IP2025 ships user limits **GR 59–168**, density 1.8–3, neutron −0.1–0.6, DTC 40–240; and extreme
limits **GR 117–256**, density 1.5–3.5, neutron −0.2–1, DTC 40–240. The IP2025 ingest's own §5.9
states the problem without prompting: these "are not physical limits — the shipped GR range 59–168 is
a data-specific example, not a validity envelope."

Two things follow. First, a GR user-minimum of **59 gAPI sits above a delivered field's clean-sand P3
of 53.68 gAPI** (T3): the entire clean-sand population of a real study would be flagged "outside user
limits". Second, ledger `F-1` — **extreme-low GR (117) exceeds user-min GR (59)**, inverting the
semantics, since the extreme band must bracket the user band; and the same page's narrative discusses
flagging GR below zero, which a lower bound of 117 cannot express. One of the two shipped panels is
wrong and the ledger's leaning is that the extreme table is the likelier culprit; which panel is
authoritative remains OPEN.

The obligation is not to pick a better number. It is that **a QC limit is a per-field, per-tool,
per-campaign quantity and has no shippable default**, while the *precedence semantics* between a user
band and an extreme band are general and must be specified once and enforced.

**Obligation.** `SB-ENV-056`, `SB-ENV-057`, and the `ABSENT` dispositions in §5.

### 2.11 Geolog ships a per-tool uncertainty family that has no competitor, and it is already the shape of delivered work

**Tier T1 (Geolog `unc_*`), T3 (project-kb).** Dossier §2.7, §3.10, §4.11.

Geolog ships an `unc_*` module per tool, computing a per-sample uncertainty on the corrected curve
from the correction terms actually applied. **Neither IP nor Techlog holds an equivalent.** That is
a capability gap in the incumbents, not a SandiBumi gap, and `03_EVIDENCE_BASE.md` §14.1 makes vendor
gaps the product's primary competitive claim.

It matters because delivered house work already carries uncertainty — as **hand-set constants**.
Replacing a hand-set constant with a value computed from the corrections that were actually applied
converts a stated assumption into a derived quantity, and it is the natural consumer of `SB-ENV-005`'s
applied-step list: an uncertainty that knows which steps ran is the only kind that can be honest, and
§2.2 is the worked example of what happens when it does not.

**Obligation.** `SB-ENV-019`, `SB-ENV-020`.

### 2.12 The correction-chart architecture splits three ways, and the vendors' own chart provenance is incomplete

**Tier T2/T3.** Dossier §2.12, §4.7, ledger `F-12`, `F-13`.

The three incumbents implement corrections three structurally different ways: as **compiled chart
lookups** against shipped table files, as **analytic fits** to those charts, and as **user-editable
parameter tables**. The architectural choice is more consequential than the arithmetic, because it
decides what happens off the end of the chart — a lookup can refuse, a fit will always return a
number.

The capability intelligence worth recording, and the limit of it: IP documents a Sperry-Sun chart
source citing **1998 in one place and 1996 in another** (ledger `F-12`), and records that it
"received the Baker Atlas chart book as a series of algorithms" — i.e. **the vendor could not
reconstruct chart numbers at all** — while separately listing a 1984 book it states was never
received as a chart book (ledger `F-13`). Two readings on the table, no adjudication.

This is capability intelligence and it is wanted: it establishes that **chart coverage is uneven,
edition-ambiguous and vendor-specific even inside a commercial product**, which is the strongest
available argument for `SB-ENV-015` — specify the chart *interface* so that coverage can be declared,
audited and extended, rather than assumed. **No chart datum from any of those sources appears in this
chapter.** Refusal `RF-2`, §7.3.

**Obligation.** `SB-ENV-015`, `SB-ENV-017`, `SB-ENV-020`.

### 2.13 IP ships three mutually incompatible flag polarities inside one workflow family

**Tier T2 (ledger `F-2`).** Dossier §2.13, §4.2, §4.14.

Within IP's own conditioning family the bad-hole/validity flag appears with **three different
polarities** — including `curve_autoedit`'s `−999` invalid / `1` valid, which is a third convention
inside the same vendor's same feature set. Geolog ships one polarity and gets the benefit.

A flag is consumed by a mask, and a mask inverted is not a degraded result — it is the exact
complement of the intended one. It deletes the good rock and keeps the bad, and the output is a
complete, plausible, fully populated curve. **There is no downstream check that can catch it**, which
puts it in the same class as the depth-unit error: a single-bit convention with unbounded
consequence.

**Obligation.** `SB-ENV-026`, `SB-ENV-030`.

### 2.14 Two vendor self-contradictions that are requirements in disguise: gap filling and filter length

**Tier T2 (ledger `F-3`, `F-4`).** Dossier §2.5, §2.14, §4.5, §4.10.

**Gap filling (`F-3`).** IP's page says gaps are filled when the gap is *less than* the stated
maximum; its dialog implies otherwise. The dossier resolves the contradiction against the dialog. The
requirement this generates is not a boundary convention — it is that **the comparison at the boundary
must be stated in the module's own documentation and asserted in a test**, because an off-by-one at
the maximum gap is the difference between inventing a sample and declining to.

**Filter length (`F-4`).** IP's own documentation states the filter-length limit three ways —
**1–121, 3–121 and 2001** — in three places. A filter length of 1 is a no-op; a length of 2 has no
centre; the three ranges cannot all be right. And the deeper issue is that **the parameter is in
samples at all**: a length in samples silently changes the amount of rock it covers the moment the
curve is resampled, or when one curve of a run came in at 2 in and another at 6 in.

**Obligation.** `SB-ENV-035`, `SB-ENV-036`, `SB-ENV-034`.

### 2.15 IP's shipped neutron salinity default is not a value — it is nonsense, and it is the argument against shipping any

**Tier T2 (ledger `F-14`), T1 (Geolog `evs_tnph.info`).** Dossier §2.6.1, §4.7.

IP's environmental-correction neutron (CNL) tab ships borehole salinity and formation salinity both
as **`2.8E-4 Kppm`**. That is **0.28 ppm** — fresher than distilled water, by orders of magnitude,
in a field where formation waters run tens of thousands of ppm. It is confirmed nonsense, not a
defensible choice. The same tab ships input **and** output matrix as Limestone, which is §2.9's trap
wearing a default.

Geolog's applying module ships **50 kppm** for the same quantity. GE's documentation states **0
kppm**. Three vendors, three values spanning **five orders of magnitude**, one of them physically
impossible.

There is no arithmetic that reconciles these and no basis for choosing among them. **Formation water
salinity is a measured property of a specific formation**; a shipped default is a guess wearing a
manifest's authority, and it enters the neutron correction, the resistivity temperature correction
and `Rw` itself. `SB-ENV-016` ships it `ABSENT`.

**Obligation.** `SB-ENV-016`, and the `ABSENT` disposition in §5.

---

## 3. SandiBumi as-built

Written from source. Every pointer below was opened and re-verified in this session; none is
repeated from another document. The repository was read-only for this task.

The headline: **the conditioning layer is the strongest part of this domain and in several respects
already beats all three incumbents; the environmental-correction layer is the weakest part of the
product, and it is weak in the specific way §2 says is most expensive** — it produces plausible
numbers from uncited coefficients and names the output "environmentally corrected" whether or not a
correction occurred.

### 3.1 What is already right, and why it is the pattern for the rest

These are not filler. Each is a documented incumbent defect that SandiBumi has already avoided, and
each is cited in §4 as the precedent a divergent module must be brought up to.

**A window is a thickness, never a sample count** — `condition.rs:15-20`, enforced through
`Frame::windows` and declared with the unit token `"depth"` (`condition.rs:252`, `frame.rs:242`). This
is the direct fix for §2.14's filter-length-in-samples problem, and the module header states the
reasoning: a window in samples "silently changes the amount of rock it covers the moment a curve is
resampled, or when one curve of a run came in at 2 inches and another at 6 — and nothing downstream
can see that it did."

**Nothing invents a sample except Fill Gaps, which says so** — `condition.rs:22-27`. Smoothing never
bridges a gap; a MISSING sample stays MISSING. This **inverts** Geolog's `PRESERVE_MISSING = FALSE`
default. Fill Gaps is bounded by a user-set maximum (`MAX_GAP`, `param_open`), refuses a gap open at
either end (`condition.rs:718-720`), measures the gap between the **live** samples either side
(`:726`), and flags every sample it writes.

**Robust despiking by construction** — `condition.rs:154-172` computes a window spread as
`1.4826 × median(|v − median|)`, i.e. MAD, with a mean-absolute-deviation fallback when the MAD is
zero, and a `MIN_HAMPEL_SAMPLES = 5` floor (`:176`). SandiBumi therefore never ships §2.5's 0 %-breakdown
estimator. The detector is `sd.is_finite() && sd > 0.0 && |v − med| > k·sd` (`:379`) — the
finite-and-positive guard is the correct handling of a degenerate window.

**Exact order statistics in Normalize** — `condition.rs:991-998` sorts before calling
`distribution::percentile`, with a comment stating precisely why: a percentile taken on a
depth-ordered slice "returns whatever value happens to sit 3% of the way down the well." This is the
direct fix for §2.6(d), and by using order statistics rather than a histogram it is structurally
immune to §2.6(a)–(c) as well.

**Normalize refuses without a reference pair** — `REF_LOW`/`REF_HIGH` are `param_open`
(`condition.rs:974-980`), so the field opens empty and the run refuses. `param_open`'s own doc
(`modules.rs:145-158`) states the principle in the form this chapter needs: a shipped default would be
"somebody's field calibration wearing the authority of a manifest."

**Clip refuses rather than repairs** — it requires at least one bound and **refuses a reversed pair
rather than swapping it** (`condition.rs:591-634`). A tool that silently swaps a reversed pair has
decided the user made a typo; a tool that refuses has noticed that it cannot know.

**`log_in_computed` is a working unit-contract enforcement** — `modules.rs:61-65`. A `LogIn` marked
`computed_only` resolves from computed provenance and **never** the raw import store, "for
unit-contract inputs like FTEMP/FPRESS where a raw curve with the same mnemonic (a commercial LAS
export's degF FTEMP) would silently masquerade as the degC/psi curve the module assumes."
`nphi_env_corr` uses it on `FTEMP` (`modules.rs:1869`). **This is `SB-CORE-003` already working, on
one parameter, in one direction** — see §3.9.

**Results QC is honest under-specification** — `resultsqc.rs`. A model whose input column exists but
is entirely null is **dropped and reported**, not counted as active (`:171-183`, test at `:539-554`);
spread is NaN below two comparable models (`:256-263`); and Waxman-Smits and Dual-Water join the
envelope **only** when a real `Qv`/`Swb` curve is supplied, with `Qv` null-guarded at `:204-210`
because `(B·Qv).max(0)` would otherwise collapse a null to the clean-sand Archie branch and "both
mislabel WS as evaluated and understate the spread in exactly the shaly zone that matters." Nine
`notes` strings explain every omission to the user. **This is the `SB-CORE-002` compliance model for
the whole domain**, and §4 repeatedly asks the environmental corrections to meet the bar this file
already sets.

**Reframe's five rules** — `reframe.rs:21-51`. An own-frame set is declared (`log_sets.frame = 'OWN'`)
rather than inferred from its depths, because "a set that happens to fall on the standard grid and a
set deliberately re-framed onto it are different claims." An own-frame set is written to the archive
only, never to the current store, because doing otherwise would blank the readable curve and "the
interpretation silently emptied by a resample." Downsampling averages and upsampling interpolates,
**per curve**, with `Method::Auto` choosing by inspecting the values rather than the name. **An
output sample with no input inside it is MISSING, never the nearest value.** And permeability is
averaged in the geometry the flow has — `frame.rs:19-29` states the arithmetic/geometric/harmonic
spread as 500 / 0.3 / 0.02 mD on an equal-parts 1000 mD sand and 0.01 mD shale, and notes that the
arithmetic answer "is the ONE of the three that always reads highest — so the error never looks like
a problem."

**Status:** `PRESENT-OK` for all of the above. §4 converts each into a requirement so it cannot
regress.

### 3.2 Environmental corrections — `PARTIAL` and `PRESENT-DIVERGENT`

Three modules, all in `modules.rs`, under a family header that states its own limitation
(`modules.rs:1797-1804`):

> "linearized, coefficient-driven equivalents of the service-company chartbook corrections — the
> coefficients are parameters with **chartbook-magnitude defaults**, so they can be tuned per
> tool/field. Chart-lookup fidelity comes later (ROADMAP). Each writes a corrected copy
> (`<LOG>_EC`); inputs are never modified, and **a missing QC input (e.g. no caliper) passes the log
> through uncorrected rather than blanking.**"

Both bolded clauses are the defects. Taken in turn:

**`gr_hole_corr`** (`modules.rs:1806-1849`) — `GR_EC = GR·(1 + K_GR·(CALI − BS))`, with `K_GR` = 0.0075
per inch (`:1816-1817`) and undersize clamped. **Status: `PARTIAL` and `PRESENT-DIVERGENT`.** It is a
single-term correction on hole size alone. Every incumbent's GR correction is a function of hole size
**and** mud weight **and** tool position (centred/eccentred) **and** mud type (barite-loaded or not)
— §2.9's table shows Geolog's `unc_gr` branching on `GR_TOOL_SIZE` and `GR_TOOL_POS` before it
computes anything. SandiBumi models one of four. `K_GR` = 0.0075 /in carries **no source**: the
family header calls it "chartbook-magnitude", and no chartbook, edition, chart number or tool is
named anywhere in the module.

**The silent pass-through is the more serious half.** At `:1838`, when the caliper is missing the
module writes the **uncorrected** GR into `GR_EC` and returns success. A curve named
"environmentally corrected" that was not corrected, with nothing in the output, the flag stream, the
provenance or the panel recording that fact. `SB-CORE-002` in one line. The same pattern is in
`rhob_hole_corr`.

**`nphi_env_corr`** (`modules.rs:1851-1892`) —
`NPHI_EC = NPHI + K_TEMP·(FTEMP − T_REF) + K_SAL·(SALW/100000)`, with `K_TEMP` = 0.0001 v/v per degC,
`T_REF` = 24.0 degC, `K_SAL` = −0.002 v/v per 100 kppm and `SALW` = 20000 ppm (`:1862-1865`).
**Status: `PARTIAL` and `PRESENT-DIVERGENT`, and this is the most divergent module in the domain.**

*Coverage.* It implements **two of the ten** steps in §2.2's chain — temperature and formation
salinity. Absent: hole-size back-out, hole size, mudcake, borehole salinity, mud weight, pressure,
standoff, and the matrix scale. §2.2's own table names two of the absent eight as worth p.u.

*Magnitude.* The house gate (T4) states the neutron temperature effect as **−0.048 p.u./°F**. In this
module's own units that is 0.048 × 1.8 = 0.0864 p.u./°C = **0.000864 v/v per °C**. The shipped
`K_TEMP` is **0.0001 v/v per °C — 8.6× smaller**. At a formation temperature of ~110 °C — what
`ftemp_grad`'s own defaults give at 2,800 m on a 3 °C/100 m gradient, i.e. an ordinary hot clastic
section — that is 86 °C above `T_REF`:

| | slope (v/v/°C) | correction at ΔT = 86 °C |
|---|---|---|
| SandiBumi shipped `K_TEMP` | 0.0001 | 0.0086 v/v = **0.9 p.u.** |
| House gate's stated slope | 0.000864 | 0.0743 v/v = **7.4 p.u.** |
| **Difference** | | **≈ 6.5 p.u.** |

Against delivered carbonate `PHIE` means of 0.05–0.07 v/v (T3), 6.5 p.u. is larger than the porosity
being measured. **The arithmetic above is a linear extension of the house gate's own 50 °F
linearisation** — which is the same linear form this module uses — so it establishes an order of
magnitude, not a coefficient. It is stated here as a **verification check, not as a value to adopt**;
the real correction is a non-linear chart and the requirement (`SB-ENV-016`) is that the coefficient
ship `ABSENT` with a cited source, not that it be replaced with 0.000864.

*Partial application.* The salinity term is applied **whether or not `FTEMP` is present** — with no
`FTEMP` only the salinity term runs, and the output is still named `NPHI_EC` with no record that half
the correction did not happen.

*One genuine strength.* `log_in_computed("FTEMP", …)` at `:1869` means a raw degF `FTEMP` from a
commercial LAS cannot masquerade as the degC curve the module assumes. That is the right mechanism —
see §3.9.

**`rhob_hole_corr`** (`modules.rs:1894-1932`) — `RHOB_EC = RHOB + K_RHO·(CALI − HD_REF)`, clamped at
zero, with `K_RHO` = 0.004 g/cc/in and `HD_REF` = 10.0 in (`:1906-1907`). **Status: `PARTIAL` and
`PRESENT-DIVERGENT`.** One term (hole size); no mudcake thickness, no mud weight, no mudcake density.
Both coefficients uncited. `HD_REF` = 10.0 in is a **reference hole diameter presented as a
universal**: on an 8½ in hole in gauge, `CALI − HD_REF` = −1.5 in and the clamp makes the correction
zero, which is correct by accident; on a 12¼ in hole in gauge it is +1.75 in and the module applies
**0.007 g/cc of correction to a perfectly good hole**. The parameter is a property of the tool and
the bit, not of the software.

**No `unc_*` twin exists for any of the three.** §2.11's capability gap is currently SandiBumi's gap
too. Status: `ABSENT`.

**No chart-lookup path exists.** `neutron_charts.rs` holds neutron **matrix-equivalence** tables
(apparent limestone porosity → true porosity per matrix), which is a `MIN`/`POR` concern, not an
environmental correction; there is no borehole-correction chart table anywhere in the tree, and no
interface through which one could be supplied. Status: `ABSENT` — and `SB-ENV-015` deliberately
specifies the interface, not the data.

### 3.3 Hole-condition and data-condition flagging

**`badhole`** (`modules.rs:1183-1247`). **Status: `PRESENT-DIVERGENT`.**

The **logic is right and matches Geolog's three-way branch** (`:1222-1238`): it tracks `any` and
`bad` separately, evaluates the DRHO term only when DRHO is present and the caliper term only when
both caliper and bit size are present, and leaves the flag MISSING when neither input exists. That
is §2.4's graceful-degradation requirement, already met.

The **defaults are the divergence**, and all three are uncited (`:1195-1197`):

| Shipped | Value | Against the evidence |
|---|---|---|
| `DRHO_MAX` | 0.05 g/cc | House precedent spans 0.02–0.15 g/cc across seven studies and **matches none of them**; IP ships 0.1; the ITB gate says 0.15 |
| `DCAL_MAX` | 1.0 in | House precedent is consistently **≈ 2 in**. At 1.0 in, a 9.5 in caliper on an 8½ in bit flags bad hole where **every delivered study** requires 10.5 in — roughly **twice the masked footage** in a mildly rugose hole |
| `BS_DEF` | 8.5 in | A **default bit size**. This does not estimate a parameter; it invents hole geometry |

`BS_DEF` deserves its own line because its failure is asymmetric in the dangerous direction. With a
real 12¼ in hole and no BS curve, a gauge caliper gives `12.25 − 8.5 = 3.75` in and **everything
flags** — loud, survivable. With a real 6 in hole, a gauge caliper gives `6.2 − 8.5 = −2.3` in and
**nothing ever flags** — the interpreter concludes the slim-hole well has no bad hole. That is §2.4's
silent direction, reproduced inside SandiBumi.

**No reason channel and no sign channel.** The output is a single 0/1 `BADHOLE` (`:1238`). An
interpreter cannot learn from it whether the caliper term fired, the DRHO term fired, or both — nor,
where DRHO fired, **which way DRHO went**, which is the difference between a washout and a mudcake
and therefore between "the density read mud" and "the density read cake". Both are diagnostic, both
are free to emit, and neither exists.

**`condflag`** (`modules.rs:1249-1440`). **Status: `PRESENT-OK` with one unenforced precondition.**
This module is well built: bad hole never counts as coal (`:1379`); flagged beds thinner than
`MIN_THICK` are dropped as spikes with runs **bridged across missing samples** so a null inside a bed
cannot shave it thin (`bridged_runs`, `:1329-1341`, `:1401-1413`); a degenerate matrix/fluid pair
leaves the density-porosity flags MISSING rather than flagging on ±inf (`:1381-1387`); a bad-hole
interval earns shoulders only when it is a real bed (`:1415-1423`); and the doc warns against masking
the `condflag` run itself with `BADHOLE`, which would blank `COND_FLAG` exactly where it must read 1
(`:1272-1274`).

The gap is §2.9's: the doc **states** the neutron-matrix precondition in prose — "NPHI must be in
matrix units consistent with `RHO_MA`: limestone-unit neutron against a sandstone `RHO_MA` reads
about 0.04 low in clean water sand, **right at the `XOVER_MIN` default**" (`:1261-1264`) — and then
nothing checks it. `XOVER_MIN` ships at 0.04 (`:1286-1293`), so the stated error is exactly the size
of the threshold it corrupts: a limestone-scale neutron run against a sandstone matrix suppresses
gas crossover almost exactly to the flagging threshold. This is `SB-CORE-003` with the condition
already written down in the right file, in the right words, in the wrong medium.

**Flag polarity** is consistent at `1.0 = true` across `badhole`, `condflag` and the Condition
family's flag channels, but it is a **convention, not a type** — there is no enum, no validator and
no single definition site, so §2.13's three-polarity failure is prevented by discipline rather than
by construction.

### 3.4 Formation temperature — two modules, one mnemonic, 33 °C apart

**Status: `PRESENT-DIVERGENT`. This is the domain's `SB-CORE-006` and `SB-CORE-007` instance, and it
is native — found in SandiBumi's own source, not inherited from a vendor.**

Two shipped modules compute formation temperature and **both emit the mnemonic `FTEMP`**
(`modules.rs:1055` and `modules.rs:1170`):

| | `ftemp_grad` (`modules.rs:1010-1056`) | `precalc` (`modules.rs:1090-1177`) |
|---|---|---|
| Surface intercept | `TSURF` = **26.7 degC** (`:1021`) | `SURF_TEMP` = **77.0** `degF\|degC` (`:1090`) |
| Gradient | `TGRAD` = **0.03 degC/m** (`:1022`) | `TEMP_GRAD` = **0.026** `deg/ft\|m` (`:1091`) |
| Depth reference | **measured depth** — `ctx.log("DEPTH")` (`:1031`), used directly at `:1051` | **TVDSS**, falling back to `DEPTH` when absent (`:1122-1124`), used at `:1144` |
| Output | `FTEMP` | `FTEMP` |

Three separate failures live in that table.

**(a) One mnemonic, two answers.** At 2,000 m TVD (6,561.7 ft), on each module's own shipped
defaults and its own intended unit system:

- `ftemp_grad`: 26.7 + 0.03 × 2000 = **86.7 °C**
- `precalc` at its feet-based fit: 77 + 0.026 × 6561.7 = 247.6 °F = **119.8 °C**

**33.1 °C apart, under one curve name.** Whichever ran last is what every downstream consumer reads,
and `nphi_env_corr` (`:1869`), `gascorr` and the whole saturation chain read exactly this curve.
Propagated through the classical `Rw(T) = Rw_ref·(T_ref + 21.5)/(T + 21.5)`, believing 86.7 °C when
the truth is 119.8 °C inflates `Rw` by a factor 141.3/108.2 = 1.306, and Archie's `Sw ∝ √Rw` turns
that into **Sw over-reading by 14.3 % relative** — a true `Sw` of 0.35 computed as 0.40, five
saturation units, on a commercial deliverable.

**(b) A depth-reference divergence hiding inside (a).** One module runs on measured depth and the
other on TVDSS. In a vertical well they agree; in any deviated well they do not, and the geothermal
gradient is a property of **true vertical depth**. `ftemp_grad` is simply wrong in a deviated well,
and nothing says so.

**(c) The unit trap of §2.7, reproduced.** `precalc`'s own documentation admits the shipped defaults
are one study's **feet-based** fits. On a metric project a user who accepts them gets
77 + 0.026 × 2000 = 129 °F = **53.9 °C** where 119.8 °C belongs — a **66 °C error**, silent, smooth
and perfectly plottable. In the other direction, `ftemp_grad`'s 0.03 degC/**m** applied to a foot
project's depth numbers gives 26.7 + 0.03 × 6561.7 = **223.6 °C**. This is `02_RISKS_AND_CONTRADICTIONS.md`
R14's 3.28× shape, in this domain, on the one curve that feeds every temperature-dependent method.

**What is already right here, and is worth preserving verbatim.** `precalc`'s `TEMP_GRAD` and
`SURF_TEMP` are `param_well` and carry `well_scope: true`, which **refuses a named-zone override**.
The reasoning at `modules.rs:66-87` is the best single piece of petrophysical engineering writing in
the repository: because `precalc` computes `SURF_TEMP + TEMP_GRAD × TVDSS` from surface at every
sample rather than integrating down through the zones above, a per-zone gradient makes the profile
**jump** rather than bend — "a 0.03 °C/m well with a 0.035 override below 1500 m stepped **10.5 °C
across 100 m** where the undisturbed trend rises 3.0". And the asymmetry is deliberate and correct:
the same restriction is **not** applied to `PSURF`/`PGRAD`, because "a pressure step at a formation
top is a pressure compartment, which is a real thing rock does."

**Absent branches.** Neither module implements a **mudline / water-bottom** branch, which is
Geolog's second `ftemp` branch and is required for any offshore well; nor a **BHT-based** branch
that solves the gradient from a measured bottom-hole temperature rather than assuming it — although
`ftemp_grad` declares `BHT` and `TD_BHT` parameters (`modules.rs:1023-1024`) that its computation at
`:1051` does not use. A declared parameter that does not enter the answer is worse than an absent
one: the dialog invites the user to set it and then discards it.

### 3.5 Normalisation

**Status: `PRESENT-OK`, with one `SB-CORE-007` crossing.**

`crate::condition::normalize` (`condition.rs:868-1041`) is the single implementation.
`gr_normalize` (`modules.rs:2609-2673`) is a thin adapter kept runnable only so saved chains still
resolve, and its own doc comment says so — "it is NOT a second implementation" (`:2639-2649`), the
pickers hide it via `SUPERSEDED_MODULE_IDS`, and it maps `GR_LOW_REF`/`GR_HIGH_REF` onto
`REF_LOW`/`REF_HIGH` and delegates (`:2650-2672`). **This is exactly the right handling of a
superseded module** and it is the model for `SB-ENV-049`: one equation, one implementation, the old
name kept resolvable rather than retired, with the reason recorded in source.

`P_LOW` = 3.0 and `P_HIGH` = 97.0 (`condition.rs:915-916`) match the house standard. The reference
pair is `param_open` and the run refuses without it. The percentile is an exact order statistic.

**The crossing.** `gr_normalize`'s own reference defaults are `GR_LOW_REF` = 20.0 and `GR_HIGH_REF`
= 120.0 (`modules.rs:2631-2632`), and its doc is explicit and correct that these are *not* a
calibration — "SET YOUR OWN FIELD REFERENCE PAIR — that is the entire point of the module … A
reference pair from one basin is the wrong reference in another" (`:2618-2626`). But 20/120 is one
of the **four** clean/shale GR endpoint pairs `SB-CORE-007` records across the codebase, and
`04_CORE_REQUIREMENTS.md` quantifies that spread at **22.2 % `Vsh` at GR 70 gAPI**. A normalisation
reference pair and a `Vsh` endpoint pair are different quantities that happen to share a value here;
the requirement (`SB-ENV-055`) is that they be **separately named and separately sourced**, so that
fixing the `Vsh` endpoint does not silently move every normalised curve in the project.

### 3.6 Depth control, framing and the depth-unit declaration split

**`depth_shift`** (`modules.rs:2501-2566`). **Status: `PRESENT-DIVERGENT` on its declaration.** The
arithmetic is unit-agnostic and correct — `out[i] = interp_at(&depth, &vals, d − shift)` (`:2563`),
so `SHIFT` is applied in whatever unit the depth column carries — but the manifest declares the unit
as **`"m"`** (`:2512`) and the doc says "Shifts CURVE by SHIFT **metres**" (`:2506`). On a foot
project the label is simply false, and the label is the only thing the user sees.

That exposes a genuine `SB-CORE-007` instance: **the codebase declares "a length in the project's
depth unit" three different ways.**

| Declaration | Sites | Meaning |
|---|---|---|
| `"depth"` | `condition.rs:252` (`WINDOW`), `frame.rs:242-243` (`INTERVAL`, `MIN_BED`) | the project's depth unit — correct |
| `"m\|ft"` | `modules.rs:1294-1295` (`MIN_THICK`, `SHOULDER`) | same meaning, different token |
| `"m"` | `modules.rs:2512` (`SHIFT`) | asserts metres |

`condflag`'s doc compensates in prose — "`MIN_THICK` and `SHOULDER` are in the depth curve's unit —
the defaults suit metres, **roughly triple them for feet**" (`:1275-1276`) — which is the right
warning delivered by the wrong mechanism. One token, one meaning, validated once (`SB-ENV-057`).

**Interactive editing** (`curve_edit.rs:1-12, 23`) provides shift / set / blank / interpolate /
scale over a depth interval, on whichever store holds the curve, as a transactional
read-modify-rewrite that returns the previous `(depth, value)` pairs for an exact undo. The undo
contract is good. **What is absent is provenance**: an interactively shifted curve carries no record
that it was shifted, by how much, over what interval, or by whom.

**Blocking** (`frame.rs:210-279`) is module-shaped and correct — a blocked curve stays on the
original frame, which the header argues is "the honest way to store it, since nothing downstream then
has to know it was upscaled." `INTERVAL` and `MIN_BED` are both `param_open`. `OPT_STAT` offers
MEAN/GEOMETRIC/HARMONIC with the doc naming the case for each and **deliberately declining to pick a
default that is right for permeability** (`frame.rs:19-29`).

**Reframing** (`reframe.rs`) is well-specified — see §3.1. **Speed correction of any kind is
`ABSENT`**; see §7.4.

### 3.7 The universal run mask, and the audited defect in it

**Status: `PRESENT-DIVERGENT`, with the defect already pinned by a test.**

The mask is resolved once in the runner rather than in forty modules (`workflow.rs:557-579`) and
applied **twice**: flagged samples are blanked in the module's **inputs** before the run
(`:583-593`), so per-run statistics such as `gr_normalize`'s percentiles and `log_predict`'s training
set are not anchored by casing or washout samples; and blanked again in the **outputs** afterwards
(`:636-644`). The input-side blanking is a genuine improvement over masking outputs alone, and the
reasoning is recorded at `:557-563`.

The defect is that the rule has **no exemption for a module whose purpose is to produce a value where
the mask says there is none.** `workflow.rs:3142` carries a test named
`a_masked_washout_defeats_the_very_module_meant_to_repair_it`, pinned "as the audited defect rather
than as correct behaviour" (`:3124`). Its doc states the case exactly: `log_predict`'s `MAX_RAW` mode
exists to repair a density log inside a washout; the mask exists to remove washout samples from
everything; **run them together — which is what the module's own documentation instructs — and the
mask wins**, so the one curve built to fill the bad hole comes back MISSING inside the bad hole.

The test also records the fix's true shape, which matters for `SB-ENV-027`: **there are two blanks,
not one**, and exempting `log_predict` from the output pass alone would leave the output exactly as
MISSING as it is now, "so whoever takes this on knows before they start" (`:3132-3137`). The unmasked
control in the same test proves the module works and the runner discards the answer.

A second, milder honesty behaviour is already right: a run whose outputs are entirely MISSING is
reported as **Warned, not Ok**, "so the panel doesn't read as a successful correction"
(`workflow.rs:655-658`) — and `gascorr` with no `precalc` is the example the comment names.

### 3.8 Results QC

**Status: `PRESENT-OK`** — see §3.1. The gap is scope, not quality: `resultsqc.rs` answers "does the
Sw model choice change the answer?" and nothing answers the equivalent question for this domain —
*does the correction chain choice change the answer?* No module compares a corrected curve against
its uncorrected self, reports how much each step moved it, or surfaces which steps were unavailable.
That is `SB-ENV-020`, and `resultsqc.rs`'s `notes` mechanism is the pattern it should copy.

### 3.9 The validity-condition mechanism — what already exists to build on

This is the section `SB-CORE-003` turns on, and the finding is better than expected: **SandiBumi does
not need a new mechanism. It needs a fourth member of a family it has already built twice.**

`ArgSpec` (`modules.rs:35-88`) is the declarative spec every module parameter is described by, it is
`Serialize`, and the auto-generated dialog and the saved-run `params_json` are both driven from it.
It already carries **three** enforcement-shaped declarative fields:

| Field | Site | What it declares | Enforced where |
|---|---|---|---|
| `min` / `max` | `:56-58` | numeric validity range | dialog |
| `computed_only` | `:59-65` | this input MUST come from computed provenance, never the raw store — the `FTEMP` degF-masquerade guard | input resolution |
| `well_scope` | `:66-87` | a named-zone override of this parameter is REFUSED — the geothermal-trend guard | parameter resolution |

Both of the latter two are **exactly `SB-CORE-003` in miniature**: a validity condition, stated as
data on the spec, evaluated by the runner before the module body executes, producing a refusal rather
than a number. `computed_only` even carries its rationale in the doc comment, which is
`SB-CORE-004`'s source string in prose form.

What is missing is the general case: a field that can express **"this correction is valid for hole
sizes 6–16 in", "this chart is tabulated for 8–13 lb/gal on the normal-mud branch and 8–18 on the
barite branch", "this parameter's value must be one of `{SANDSTONE, DOLOMITE, LIMESTONE}`"** — and a
runner pass that evaluates such conditions against the **data**, not only against the dialog entry,
because §2.9's failures are data-dependent and per-sample.

`choice_labels` (`:46-55`) shows the cost of not having it. Its doc records that `OPT_GR`'s bare
choice strings meant "the only place a user was told which is which was the manual test plan — and
the plan had them the wrong way round", and that picking the wrong one "returns 0.33 where 0.216
belongs: a shale volume more than half again too high through the whole intermediate-GR interval,
which is exactly where the VSH cutoff decides net pay. **The curve looks entirely normal and nothing
downstream can catch it.**" That is the same failure shape as every §2 finding, and it was fixed by
adding a declarative field to `ArgSpec`. The precedent is established; `SB-ENV-001` extends it.

**The position this chapter is required to take.** `SB-CORE-003`'s design note says the validity
mechanism and `SB-CORE-004`'s source string are the same mechanism and should be built together.
**Concur, and the source above is why.** They are both per-parameter declarative metadata on
`ArgSpec`, serialized through the same path, rendered by the same auto-generated dialog, persisted
into the same `params_json`, and consumed by the same runner pass. Building them separately means
two migrations of one struct, two dialog changes, two serialization-compatibility problems for saved
runs, and two chances to disagree about where a "source" ends and a "validity condition" begins —
when in fact `computed_only` already demonstrates that they are one thing: it is a validity condition
*whose justification is its source*. Specified as one change in `SB-ENV-001` and `SB-ENV-004`, with
`SB-ENV-004` explicitly noting the joint build.

### 3.10 Status summary

| Capability | Status | Evidence |
|---|---|---|
| Declarative validity conditions (general) | `ABSENT` | `modules.rs:35-88` — three special cases, no general field |
| Runner precondition pass | `ABSENT` | no evaluation site in `workflow.rs` |
| Applied-step manifest on a corrected curve | `ABSENT` | — |
| GR borehole correction | `PARTIAL` / `PRESENT-DIVERGENT` | `modules.rs:1806-1849` — 1 of 4 terms, uncited `K_GR`, silent pass-through `:1838` |
| Neutron environmental correction | `PARTIAL` / `PRESENT-DIVERGENT` | `modules.rs:1851-1892` — 2 of 10 steps, `K_TEMP` 8.6× below the house-gate slope |
| Density borehole correction | `PARTIAL` / `PRESENT-DIVERGENT` | `modules.rs:1894-1932` — 1 term, uncited, `HD_REF` universal |
| Correction-chart lookup interface | `ABSENT` | no borehole-correction table or interface in tree |
| Per-tool uncertainty (`unc_*` twin) | `ABSENT` | — |
| Bad-hole flagging — logic | `PRESENT-OK` | `modules.rs:1222-1238` |
| Bad-hole flagging — defaults | `PRESENT-DIVERGENT` | `modules.rs:1195-1197` — three uncited, `DCAL_MAX` half the house value |
| Bad-hole reason / DRHO sign channel | `ABSENT` | `modules.rs:1238` emits one 0/1 curve |
| Conditioning flags (`condflag`) | `PRESENT-OK` | `modules.rs:1249-1440` |
| Neutron matrix-scale precondition | `ABSENT` (stated in prose only) | `modules.rs:1261-1264` |
| Flag polarity as a type | `ABSENT` (convention only) | no enum or validator |
| Despike (Hampel/MAD) | `PRESENT-OK` | `condition.rs:154-176, 379` |
| Despike `K` contamination ceiling in UI | `ABSENT` | `condition.rs:253` — bare number |
| MAD constant `1.4826` | `PRESENT-DIVERGENT` | `condition.rs:166` — truncated literal, single site, not derived |
| Smoothing / kernel normalisation declaration | `PRESENT-UNVERIFIED` | `condition.rs` — kernel choice not declared in output |
| Clip / Fill Gaps / Flip | `PRESENT-OK` | `condition.rs:591-634, 687-747, 796-842` |
| Outlier / population cull | `ABSENT` | no `tp_cull` equivalent |
| Formation temperature | `PRESENT-DIVERGENT` | `modules.rs:1055` and `:1170` both emit `FTEMP`, 33.1 °C apart |
| Mudline / water-bottom branch | `ABSENT` | — |
| BHT branch | `PRESENT-DIVERGENT` | `modules.rs:1023-1024` declares `BHT`/`TD_BHT`; `:1051` ignores them |
| Resistivity temperature correction constant | `PRESENT-UNVERIFIED` | `multimin2::fluid_calc` — single site, constant not surfaced |
| Normalisation | `PRESENT-OK` | `condition.rs:868-1041, 991-998` |
| Normalisation reference pair vs `Vsh` endpoints | `PRESENT-DIVERGENT` | `modules.rs:2631-2632` shares 20/120 with three other sites |
| Depth shift — arithmetic | `PRESENT-OK` | `modules.rs:2563` |
| Depth shift — unit declaration | `PRESENT-DIVERGENT` | `modules.rs:2506, 2512` declare metres |
| Depth-unit token | `PRESENT-DIVERGENT` | three tokens for one meaning |
| Interactive edit provenance | `ABSENT` | `curve_edit.rs` — undo yes, provenance no |
| Blocking / reframing | `PRESENT-OK` | `frame.rs:210-279`, `reframe.rs:21-51` |
| Universal run mask | `PRESENT-DIVERGENT` | `workflow.rs:3124-3141` — defect pinned by its own test |
| Conditioning order as a contract | `ABSENT` | — |
| Results QC (Sw spread) | `PRESENT-OK` | `resultsqc.rs:171-183, 200-214, 256-263` |
| Correction-chain QC | `ABSENT` | — |
| Image speed correction | `ABSENT` | §7.4 |

---

## 4. Requirements

58 requirements, `SB-ENV-001` … `SB-ENV-058`. Nineteen are P0.

### 4.1 Validity conditions as enforced preconditions — `SB-CORE-003`

#### SB-ENV-001 — Declare validity conditions as data on the module spec   [P1] [status: ABSENT]

**Requirement.** Every method whose source states a validity condition MUST carry that condition as
machine-readable data on its `ArgSpec` / `ModuleSpec`, not as prose in a `doc` string. The
representation MUST express, at minimum: (a) an **enumeration** constraint on an option or a
string-valued parameter; (b) a **numeric range** on a parameter *or on an input curve's samples*,
with a unit; (c) a **branch-conditional** range, so that a condition may differ between branches of
the same module; and (d) a **required-companion** constraint, so that a correction may declare that
it is invalid without a named input. Each condition MUST carry a human-readable statement of what it
means and a source reference (`SB-ENV-004`).

**Rationale.** Dossier §1.3.1 and §2.6.1 (T1-declarative): Geolog's 126 correction modules carry
their stated ranges in `bin\*.info` VALIDATION columns, and its fail-loud reputation is a property
of those manifests rather than of the `.lls` sources — §2.9's table shows two chains
(`unc_gr.lls:116-153`, `ftemp.lls:53`) that are fail-**silent** when the code is read alone. A port
that lifts the algorithm without the manifest inherits a silent previous-frame carry and a silent
wrong-branch, and every in-range regression test still passes. Requirement (c) is not
generalisation: `unc_tnph.lls:340, 346` clamps the normal-mud branch to 8–13 lb/gal and the barite
branch to 8–18 while its header documents a single 8–18 range, so a per-module condition would be
wrong for one branch.

**As-built.** `ABSENT` in the general case, but the mechanism exists in three special cases —
`min`/`max` (`modules.rs:56-58`), `computed_only` (`:59-65`) and `well_scope` (`:66-87`), the latter
two being validity conditions in exactly this sense. `choice_labels` (`:46-55`) is the precedent that
adding a declarative field to `ArgSpec` is the established way to fix a silent-wrongness class in
this codebase.

**Verified by.** SB-ENV-T01, SB-ENV-T02, SB-ENV-T03, SB-ENV-T38

#### SB-ENV-002 — Evaluate preconditions in the runner, before the module body   [P1] [status: ABSENT]

**Requirement.** The runner MUST evaluate every declared validity condition **before** dispatching to
the module body, and MUST evaluate data-dependent conditions **per sample**. Validation MUST NOT
depend on the dialog: a run reached from a saved chain, a workflow, a zone override, a batch
multi-well run or a future API caller MUST be validated identically to one launched from the dialog.

**Rationale.** §2.9, consequence 3: Geolog achieves the right behaviour by the wrong construction —
its manifest validates, so its algorithm is unsafe wherever it is called from another path. The
house rule is that **validation lives with the algorithm**. Per-sample evaluation is not optional
because the conditions that matter are data-dependent: a hole size is a curve, a mud weight can vary
by section, a formation temperature varies continuously with depth.

**As-built.** `ABSENT`. `workflow.rs:595-596` builds `ModuleContext` and calls `run_module` with no
precondition pass between them.

**Verified by.** SB-ENV-T02, SB-ENV-T04, SB-ENV-T38

#### SB-ENV-003 — A violated precondition produces a refusal or a flagged result, never an unmarked number   [P0] [status: ABSENT]

**Requirement.** When a precondition is violated, the module MUST produce either (a) a **labelled
refusal** naming the condition, the offending value, the expected range and the source of that
range; or (b) an **explicitly flagged result** in which the affected samples are marked in a
companion flag channel and the violation is recorded in the run's provenance. It MUST NOT produce an
unmarked number. Where the violation affects a subset of samples, option (b) MUST be available so
that a single out-of-range interval does not discard an otherwise valid run.

**Rationale.** `SB-CORE-003` verbatim; `SB-CORE-002` for the honesty half. This is the requirement
the whole chapter exists to discharge, and §2 supplies four independent instances of what its
absence costs: a linear chart fit extrapolated off its measured span (§2.1), a temperature branch
selected by typo (§2.9), a resistivity law selected by fall-through (§2.3), and a neutron curve
corrected on the wrong matrix scale (§2.9).

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T02, SB-ENV-T03, SB-ENV-T04, SB-ENV-T05

#### SB-ENV-004 — Every parameter carries a source string, built as one change with the validity field   [P0] [status: PARTIAL]

**Requirement.** Every parameter in this domain MUST carry a machine-readable source reference on its
`ArgSpec`: a citation to a named document, edition, page or file, or the explicit token `ABSENT`
meaning the parameter ships with no default and the run refuses until the user supplies one. The
source field and the validity field of `SB-ENV-001` MUST be added in **one change** to `ArgSpec`, and
they MUST share the serialization, dialog-rendering and `params_json` persistence path.

**Rationale.** `SB-CORE-004`. The joint-build position is required of this chapter by
`SB-CORE-003`'s design note, and §3.9 is the evidence for concurring: `computed_only` is already a
validity condition *whose justification is its source*, so the two are one thing in the code that
exists. Building them separately means two migrations of one `Serialize` struct, two dialog changes,
two saved-run compatibility problems, and a standing boundary dispute about where a source ends and a
condition begins.

**As-built.** `PARTIAL`. The discipline exists in prose — `modules.rs:145-158` (`param_open`'s
rationale), `modules.rs:66-87` (`well_scope`'s), `condition.rs:39-41` — and in one machine-readable
form, `param_open` itself, which encodes "this parameter has no defensible default" as a type. What
is absent is the citation.

**Verified by.** SB-ENV-T06, SB-ENV-T07

#### SB-ENV-005 — A corrected curve carries the list of steps actually applied   [P0] [status: ABSENT]

**Requirement.** Every environmentally corrected output MUST carry a machine-readable **applied-step
manifest** recording, for each step in that tool's correction chain: whether it was applied, skipped
because it was unavailable, skipped because the user disabled it, or refused because a precondition
failed — and for each applied step, the parameter values used. The manifest MUST be persisted with
the curve and MUST be retrievable without re-running.

**Rationale.** §2.2 is the vendor demonstration of the cost: Geolog's `unc_tnph` computes an
uncertainty over three live steps for a curve its twin `evs_tnph` corrected with ten, and **nothing
in either output states the mismatch**. §3.2 is the SandiBumi instance: `nphi_env_corr` applies the
salinity term with or without `FTEMP` and names the result `NPHI_EC` either way. A corrected curve
without its step list is not interpretable and cannot be audited.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T08, SB-ENV-T09, SB-ENV-T10

#### SB-ENV-006 — A curve named "corrected" MUST have been corrected   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A module MUST NOT write an uncorrected copy of an input into an output whose name
asserts correction. Where a required correction input is absent, the module MUST either refuse, or
emit the output with **every affected sample marked** in a companion flag channel and the omission
recorded in the applied-step manifest of `SB-ENV-005`.

**Rationale.** `SB-CORE-002`. This is the single most direct violation found in the shipped
environmental-correction family and the family header states it as an intentional design choice —
"a missing QC input (e.g. no caliper) passes the log through uncorrected rather than blanking"
(`modules.rs:1802-1803`). The intent behind it is right: blanking a whole curve because one QC input
is missing is worse than passing it through. The naming is what is wrong. `resultsqc.rs` shows the
resolution already implemented elsewhere in the repo — proceed with what you have, and say in the
output exactly what you did not do.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1838` (`gr_hole_corr`) and the equivalent branch in
`rhob_hole_corr` (`modules.rs:1894-1932`) write `<LOG>_EC` unchanged when the caliper is missing.

**Verified by.** SB-ENV-T11, SB-ENV-T12

#### SB-ENV-007 — Per-sample correction flag channel   [P1] [status: ABSENT]

**Requirement.** Every environmental correction MUST emit a companion per-sample flag channel
recording, at minimum: correction applied in full, applied in part (with the step set identifying
which part), not applied, and refused on a precondition. Flag values MUST use the single polarity
enum of `SB-ENV-030`.

**Rationale.** `SB-ENV-005`'s manifest is per-run; corrections fail per interval. A well with a
caliper over two-thirds of its length and a washout in the reservoir needs the distinction at the
sample, not at the run. Geolog's despike already demonstrates the pattern by emitting a reversible
removed-value flag (§2.5).

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T11, SB-ENV-T13

#### SB-ENV-008 — Validity conditions are visible before the run, not only after it   [P2] [status: ABSENT]

**Requirement.** The module dialog MUST display each parameter's declared validity condition and its
source alongside the field, and MUST indicate — before the run is launched — where a condition cannot
be evaluated because a required input is absent from the well. *(Jointly owned with
`23_plotting-interactivity.md`, which owns the presentation; this chapter owns the obligation and
the content.)*

**Rationale.** A precondition that is only discoverable by running is a precondition the user
discovers on a deliverable. The `choice_labels` precedent (`modules.rs:46-55`) established that the
fix for a silent-wrongness class in this codebase is to surface the declarative fact in the dialog.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T14

#### SB-ENV-009 — A method-selection string that matches no known method is an error   [P0] [status: PRESENT-UNVERIFIED]

**Requirement.** Where a method, branch, tool family, matrix scale or convention is selected by name,
an unrecognised name MUST produce a labelled refusal. It MUST NOT fall through to a default branch,
and it MUST NOT leave a computed quantity at its previous value.

**Rationale.** Three independent vendor instances, all T1: Techlog's `TempCorr_Resistivity.py:80-83`
routes every non-`'Exxon'` string to the outlier `Arps` law (§2.3, worth 7.2 % on `Sw` at 60 °F);
`ftemp.lls:53` routes every non-`MEASUREMENT_REFERENCE` string to the mudline branch (§2.9);
`unc_gr.lls:116-153` has no `else` at all, so an unrecognised tool position **carries the previous
frame's correction constants forward** — a failure that is not reproducible, which is worse than a
wrong constant because no regression test can pin it.

**As-built.** `PRESENT-UNVERIFIED`. `ArgSpec.choices` (`modules.rs:45`) constrains options at the
dialog and `ctx.o()` reads a string; there is no test that an out-of-enum value reaching a module by
any other path is refused rather than defaulted.

**Verified by.** SB-ENV-T03, SB-ENV-T15

### 4.2 Environmental corrections

#### SB-ENV-010 — The GR borehole correction models hole size, mud weight, tool position and mud type   [P2] [status: PARTIAL]

**Requirement.** The gamma-ray borehole correction MUST accept hole size, mud weight, tool position
(centred / eccentred) and mud type (barite-loaded / non-barite) as declared inputs, MUST refuse or
flag when a term it needs is absent, and MUST record in the applied-step manifest which terms
entered the answer.

**Rationale.** All three incumbents model four terms; SandiBumi models one. Geolog's `unc_gr` branches
on `GR_TOOL_SIZE` and `GR_TOOL_POS` before computing anything (§2.9, T1), and the four position
spellings its manifest enumerates are evidence that tool position is a first-class input, not a
refinement. A barite mud is the case where a GR correction is largest and a single hole-size term is
furthest wrong.

**As-built.** `PARTIAL` — `modules.rs:1806-1849`, `GR_EC = GR·(1 + K_GR·(CALI − BS))`, hole size only.

**Verified by.** SB-ENV-T08, SB-ENV-T16

#### SB-ENV-011 — The neutron correction chain exposes all ten steps, and an unavailable step is reported   [P2] [status: PARTIAL]

**Requirement.** The neutron environmental correction MUST expose all ten steps of the standard chain
— hole-size back-out, hole size, mudcake, standoff, borehole salinity, mud weight, borehole
temperature, pressure, formation salinity, and matrix scale — each individually switchable. A step
that is unavailable for want of an input MUST be **reported as unavailable** in the applied-step
manifest, never silently skipped.

**Rationale.** §2.2 (T1): Geolog's applying module ships all ten enabled by default. SandiBumi ships
two. Two of the eight absent are quantified by the house gate at ~2.4 p.u./50 °F and ~2 p.u.
respectively — against delivered carbonate `PHIE` means of 0.05–0.07 v/v, either is comparable to
the whole porosity being measured.

**As-built.** `PARTIAL` — `modules.rs:1851-1892` implements temperature and formation salinity only.

**Verified by.** SB-ENV-T08, SB-ENV-T09, SB-ENV-T17

#### SB-ENV-012 — Neutron matrix scale is a declared property of the curve and is validated at every consumer   [P0] [status: ABSENT]

**Requirement.** A neutron curve MUST carry its matrix scale as declared metadata drawn from a closed
enumeration. Every module consuming a neutron curve alongside a matrix-dependent parameter MUST
validate the pairing and refuse or flag a mismatch. An unrecognised or absent scale MUST NOT default
to limestone.

**Rationale.** §2.9: IP states the limestone assumption on two separate pages and checks it nowhere;
the house 15-step order makes verifying it **step 2** and names the failure "silent error". Geolog's
`unc_tnph.lls:204-224` is the correct pattern — `LITHSCALE` validated in the code itself, with a real
`else` setting the result MISSING. The magnitude is already documented **inside SandiBumi**:
`modules.rs:1261-1264` states that a limestone-unit neutron against a sandstone `RHO_MA` reads about
0.04 v/v low in clean water sand, which is exactly the shipped `XOVER_MIN` of 0.04 (`:1286-1293`) —
so the mismatch suppresses gas crossover to precisely its own detection threshold.

**As-built.** `ABSENT`. The condition is stated in prose in `condflag`'s doc and checked nowhere.

**Verified by.** SB-ENV-T18, SB-ENV-T19

#### SB-ENV-013 — The density borehole correction models mudcake as well as hole size   [P2] [status: PARTIAL]

**Requirement.** The density borehole correction MUST accept mudcake thickness and mudcake density
alongside hole size, and its reference hole diameter MUST be a declared property of the tool and bit
rather than a shipped constant.

**Rationale.** §3.2: `HD_REF` ships at 10.0 in as a universal (`modules.rs:1907`). On a gauge 12¼ in
hole that applies 0.007 g/cc of "correction" to a hole that needs none; on a gauge 8½ in hole the
clamp hides the error rather than fixing it. A density correction without a mudcake term is missing
the term that dominates in exactly the wells where the correction is needed.

**As-built.** `PARTIAL` — `modules.rs:1894-1932`, one term.

**Verified by.** SB-ENV-T20

#### SB-ENV-014 — Correction coefficients ship with a source or ship ABSENT   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** No environmental-correction coefficient MAY ship with a default that is not traceable
to a named source with edition and page or file reference. A coefficient without such a source MUST
ship `ABSENT` and the run MUST refuse until the user supplies a value. The phrase "chartbook
magnitude" — or any equivalent gesture at an uncited source — MUST NOT appear as the justification
for a shipped value.

**Rationale.** `SB-CORE-004`. The family header states the problem in its own words: the coefficients
are "chartbook-magnitude defaults" (`modules.rs:1800`) and no chartbook, edition, chart number or
tool is named. §3.2 quantifies what one of them costs: `K_TEMP` = 0.0001 v/v/°C is **8.6× below** the
only quantified source in evidence, a gap of ~6.5 p.u. at ΔT = 86 °C. A default that is an order of
magnitude out and carries no source is worse than no default, because it produces a small, plausible,
confidence-inspiring correction.

**As-built.** `PRESENT-DIVERGENT` — `K_GR` (`modules.rs:1816-1817`), `K_TEMP`, `T_REF`, `K_SAL`
(`:1862-1865`), `K_RHO`, `HD_REF` (`:1906-1907`): six uncited coefficients.

**Verified by.** SB-ENV-T06, SB-ENV-T07, SB-ENV-T21

#### SB-ENV-015 — The correction-chart lookup interface is specified independently of any chart data   [P1] [status: ABSENT]

**Requirement.** Where a correction is implemented as a table lookup, the implementation MUST provide
an interface that: (a) declares the **tabulated span** of each axis with its unit; (b) states the
**interpolation rule** inside the span; (c) on a query **outside** the span, either refuses or clamps
to the boundary — the choice declared per chart, never inferred — and in the clamping case **flags
every clamped sample**; (d) never extrapolates a fitted form beyond the span it was fitted to; and
(e) records, per sample, whether the value was interpolated, clamped or refused. The interface MUST
be implementable, shippable and testable **with no chart data present**, against synthetic tables.

**Rationale.** §2.1: the difference between Geolog's behaviour and a naive port is entirely in what
happens off the end of the chart, and that behaviour is declared in the manifest, not the algorithm.
§2.12: chart coverage is uneven, edition-ambiguous and vendor-specific even inside a commercial
product — one vendor could not reconstruct chart numbers for a whole tool family at all. An interface
that declares coverage can be audited and extended; an analytic fit silently covers everything and is
right nowhere in particular.

The clause requiring testability without data is deliberate and is the requirement's main engineering
content: it lets the enforcement layer — the part that actually prevents the failure — ship and be
verified now, and be pointed at a user-supplied or independently-derived table later, **without this
chapter creating any obligation to acquire or transcribe chart data**. See §7.3, `RF-2`.

**As-built.** `ABSENT`. No borehole-correction table and no lookup interface exist in the tree.

**Verified by.** SB-ENV-T22, SB-ENV-T23, SB-ENV-T24

#### SB-ENV-016 — A measured property of the formation or the borehole ships no default   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Formation water salinity, borehole (mud filtrate) salinity, standoff, mudcake
thickness, mud weight and bit size MUST ship `ABSENT`. These are measured properties of a specific
well; a shipped value is a guess wearing a manifest's authority. Where such a parameter is required
by a selected correction step, the run MUST refuse rather than substitute.

**Rationale.** §2.15: three vendors ship formation salinity as `2.8E-4 Kppm` (= 0.28 ppm, confirmed
nonsense), 50 kppm and 0 kppm — **five orders of magnitude, one physically impossible**. §2.2: the
vendor's shipped standoff default is precisely the house gate's worked warning case, worth ~2 p.u. —
a user accepting it has not chosen a standoff, they have inherited one. §3.3: `BS_DEF` = 8.5 in does
not estimate hole geometry, it invents it, and its failure is silent in the slim-hole direction.

**As-built.** `PRESENT-DIVERGENT` — `SALW` = 20000 ppm (`modules.rs:1865`) and `BS_DEF` = 8.5 in
(`modules.rs:1197`) both ship defaults. The correct precedent is already in the codebase:
`param_open` (`modules.rs:145-158`), used for the despike window and the normalisation reference pair.

**Verified by.** SB-ENV-T07, SB-ENV-T25

#### SB-ENV-017 — Chart baselines and intermediates are named, single-assignment quantities   [P1] [status: ABSENT]

**Requirement.** Within a correction chain, a named intermediate — a chart baseline, a nominal
response, a back-out reference — MUST be assigned exactly once. A step requiring a
differently-referenced value MUST request it **by name** as a separate quantity. Reassigning a named
intermediate mid-chain is prohibited.

**Rationale.** Dossier §2.6.3 (T1, `unc_tnph.lls:238-267`), open item `O-14`. In the back-out branch,
line 246 overwrites `tnph_base` with a bit-size lookup, discarding the caliper-referenced value
computed at line 241; the no-back-out branch writes its lookups to a temporary and `tnph_base`
survives. `tnph_base` is declared "TNPH chart baseline", not a scratch variable, and is consumed
downstream by the excess term, the 0–0.50 v/v clamp, mudcake, and the entire six-hole-size standoff
family. **The two branches therefore carry different baselines into every subsequent step, and the
difference is exactly the actual-hole-size versus bit-size neutron response — it vanishes in gauge
hole and grows with washout, which is precisely where the correction matters.**

Stated as a probable defect and **not adjudicated**: a reading in which line 246 is deliberate,
because mudcake and standoff charts are nominal-hole-referenced, cannot be excluded from source
alone. What is not in doubt is that the two branches are inconsistent with each other. The
requirement is that SandiBumi decide the question explicitly instead of inheriting the ambiguity —
which single-assignment naming forces at compile time.

**As-built.** `ABSENT` — no chain exists yet in which an intermediate could be named.

**Verified by.** SB-ENV-T26

#### SB-ENV-018 — Conditioning and correction order is a declared, checkable contract   [P1] [status: ABSENT]

**Requirement.** The order in which conditioning and correction steps are applied to a curve MUST be
recorded with the curve, and a chain MUST be checkable against a declared ordering contract: each
step declares what it requires to have already happened and what it invalidates. A chain violating
the contract MUST warn with the specific violation named. The contract MUST be data, not
documentation.

**Rationale.** §2.8: no incumbent enforces an order. IP declines explicitly for logs while mandating
one **in capitals** for images — the asymmetry shows the absence is a decision, not an oversight.
Techlog states one local ordering; Geolog implies order through data flow and prevents nothing. The
house standard's 15 steps are non-commutative in ways that are individually silent: normalise before
despike and a spike sets the endpoints; despike before bad-hole flag and the evidence is gone;
correct a neutron before verifying its matrix scale and the wrong curve is corrected.

**As-built.** `ABSENT`. Order is currently whatever the user chained, and is not recorded.

**Verified by.** SB-ENV-T27, SB-ENV-T28

#### SB-ENV-019 — Per-tool uncertainty is computed over the steps actually applied, and says which   [P1] [status: ABSENT]

**Requirement.** Every environmentally corrected curve MUST be able to emit a companion per-sample
uncertainty curve computed **over the step set recorded in `SB-ENV-005`'s manifest**, and that
uncertainty MUST declare the step set it covers. An uncertainty whose step set differs from its
curve's applied-step set MUST NOT be emitted.

**Betters:** Geolog's `unc_tnph` returns `TNPH_COR_UNC` computed over three live steps for a curve
`evs_tnph` corrected with ten, and nothing in either output states the mismatch (§2.2, T1). Neither
IP nor Techlog ships a per-tool uncertainty family at all (§2.11), so the incumbent field is one
product with a by-construction under-estimate and two with nothing.

**Rationale.** §2.11: delivered house work already carries uncertainty as hand-set constants;
computing it from the corrections actually applied converts a stated assumption into a derived
quantity. The coupling to `SB-ENV-005` is the whole point — an uncertainty that does not know which
steps ran cannot be honest, and §2.2 is the worked example of the failure.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T09, SB-ENV-T29

#### SB-ENV-020 — Correction-chain QC: what did the corrections actually do?   [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a QC view that, for a corrected curve, reports the
uncorrected curve, the corrected curve, and the per-step contribution of each applied correction in
the curve's own units — together with the list of steps that were unavailable and why. *(Jointly
owned with `23_plotting-interactivity.md` for presentation.)*

**Betters:** No incumbent exposes per-step contributions. IP's corrections execute per tab with no
decomposition; Techlog's production engine is compiled; Geolog's chain is decomposable in source but
emits only the final curve and an uncertainty computed over a different step set (§2.2).

**Rationale.** §3.8: `resultsqc.rs` already answers "does the Sw model choice change the answer?" and
nothing answers the same question for the correction chain — which is the question that decides
whether a correction was worth applying at all. The `notes` mechanism at `resultsqc.rs:216-238` is
the pattern.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T30

### 4.3 Hole-condition flagging and the run mask

#### SB-ENV-021 — Bad-hole detection degrades to the inputs that exist, and says which it used   [P1] [status: PARTIAL]

**Requirement.** Bad-hole detection MUST evaluate each of its terms independently, MUST produce a
flag from whichever terms are available, and MUST leave the flag MISSING — never `0` — when no term
can be evaluated. It MUST record in its output which terms were evaluated.

**Rationale.** §2.4: availability, not preference, is why three of seven delivered studies use no
DRHO term — one brownfield study logged DRHO on **3 wells out of 362** against caliper on 355. A rule
requiring both inputs is inapplicable at that scale, and a rule that returns `0` when an input is
missing tells the interpreter the hole is good. Geolog already branches correctly
(`badhole.lls:88-101`, T1).

**As-built.** `PARTIAL` — the degradation logic is correct and already implemented
(`modules.rs:1222-1238`: `any`/`bad` tracked separately, flag left MISSING when `any` is false). What
is missing is the record of which terms fired, which is `SB-ENV-022`.

**Verified by.** SB-ENV-T31, SB-ENV-T32

#### SB-ENV-022 — Bad-hole flag carries a reason channel   [P1] [status: ABSENT]

**Requirement.** Bad-hole detection MUST emit a companion reason channel identifying, per sample,
which criterion fired: caliper, density correction, both, or neither-evaluable.

**Rationale.** A single 0/1 flag cannot be reviewed. An interpreter deciding whether to trust an
interval, or a reviewer auditing a deliverable, needs to know whether the call came from geometry or
from the density tool's own correction, because the two have different remedies — a caliper flag
means reconstruct the porosity, a DRHO flag may mean the density is recoverable. The information is
free at the point of computation and unrecoverable afterwards.

**As-built.** `ABSENT` — `modules.rs:1238` emits one 0/1 curve.

**Verified by.** SB-ENV-T31

#### SB-ENV-023 — The density correction's sign is preserved and reported   [P1] [status: ABSENT]

**Requirement.** Where bad-hole detection uses `DRHO`, the **sign** of the exceedance MUST be
preserved in the reason channel.

**Rationale.** A positive and a negative `DRHO` excursion are different physical events —
broadly, a washout in which the tool read borehole fluid, versus a mudcake in which it read cake.
Collapsing both into `|DRHO| > threshold` discards the diagnosis while keeping the alarm. The house
QC gate treats the DRHO sign as diagnostic in its own right (T4).

**As-built.** `ABSENT` — `modules.rs:1231` tests `dr.abs() > drho_max` and discards the sign.

**Verified by.** SB-ENV-T31

#### SB-ENV-024 — Bad-hole thresholds ship ABSENT with cited presets   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** `DRHO_MAX` and the differential-caliper cutoff MUST ship `ABSENT`. The application
MAY offer **named, cited presets** drawn from delivered-study precedent, each labelled with its
source; selecting a preset MUST record which preset was used in the run's provenance.

**Rationale.** §2.4: house precedent spans **0.02–0.15 g/cc, a 7.5× range**, across seven studies,
and three of them use no DRHO term at all. IP ships 0.1; the ITB gate says 0.15. There is no
defensible single default. The shipped `DCAL_MAX` = 1.0 in is worse than arbitrary — it is **half**
the value every delivered study uses, so it flags roughly twice the footage as bad hole in a mildly
rugose well, and the extra masked footage is invisible because a masked interval looks like an
interval that was never logged.

**As-built.** `PRESENT-DIVERGENT` — `DRHO_MAX` = 0.05 g/cc and `DCAL_MAX` = 1.0 in
(`modules.rs:1195-1196`), both uncited, and `DRHO_MAX` matching none of the seven precedent values.

**Verified by.** SB-ENV-T07, SB-ENV-T33

#### SB-ENV-025 — Bit size is an input, never a default   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Bit size MUST come from a curve, a well-header value or an explicit user entry. It
MUST NOT ship a default. Where no bit size is available, the caliper term of the bad-hole rule MUST
be reported unavailable and the DRHO term used alone.

**Rationale.** §3.3: `BS_DEF` = 8.5 in does not estimate a parameter, it invents hole geometry, and
its failure is asymmetric in the dangerous direction. On a real 12¼ in hole with a gauge caliper it
gives `12.25 − 8.5 = 3.75` in and everything flags — loud and survivable. On a real 6 in hole it
gives `6.2 − 8.5 = −2.3` in and **nothing ever flags**, so the interpreter concludes a slim-hole well
has no bad hole. That is §2.4's silent direction reproduced inside SandiBumi.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1197`.

**Verified by.** SB-ENV-T33, SB-ENV-T34

#### SB-ENV-026 — DRHO's unit is declared on the curve and validated at the threshold   [P0] [status: ABSENT]

**Requirement.** A density-correction curve MUST carry its unit as declared metadata, and any module
comparing it against a threshold MUST validate that the threshold's unit matches. A unit mismatch
MUST refuse. Import of a `DRHO` curve with no declared unit MUST require the user to state it.

**Rationale.** §2.4, the 1000× trap. `0.1 g/cc = 100 kg/m³`. The loud direction — an IP-shaped `0.1`
in a kg/m³ field — flags every sample. The **silent** direction — a Geolog-shaped `100` in a g/cc
field — means `100 g/cc` and nothing ever flags, and that is the direction that ships. Geolog is
internally inconsistent on the same parameter *name* (`badhole.info` in `k/m3`, `unc_ldt.lls:59,98`
in `G/C3`), which is safe inside Geolog's per-argument unit engine and wrong for **every importer,
exporter and parameter bridge that keys on the name** — which is what all of them do. This is
`SB-CORE-001` applied to a curve that is not the depth column.

**As-built.** `ABSENT` — `modules.rs:1195` declares the threshold's unit as `"g/cc"` in the manifest
string; nothing validates the curve against it.

**Verified by.** SB-ENV-T35

#### SB-ENV-027 — A module whose purpose is to produce a value where the mask says there is none MUST be exempt from the mask   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** The universal run mask MUST support a per-module **repair exemption**. An exempt
module MUST NOT have its inputs blanked at masked samples, MUST NOT have its outputs blanked at
masked samples, and MUST mark every sample it produced at a masked depth so the result is never
mistaken for a measurement. A module declaring the exemption MUST state why in its spec.

**Rationale.** The defect is already audited inside SandiBumi and pinned by its own test,
`workflow.rs:3142`, `a_masked_washout_defeats_the_very_module_meant_to_repair_it`, kept "as the
audited defect rather than as correct behaviour" (`:3124`). `log_predict`'s `MAX_RAW` mode exists to
repair a density log inside a washout; the mask exists to remove washout samples; run them together —
"which is precisely what the module's own documentation tells you to do" — and the mask wins, so the
curve built to fill the bad hole comes back MISSING inside the bad hole. The unmasked control in the
same test proves the module works and the runner discards the answer.

**The requirement is written in both halves deliberately.** The test's own doc records that "there
are TWO blanks, not one, and the audit finding names only the second" (`:3132`): the runner blanks
inputs before the run (`workflow.rs:583-593`) and outputs after (`:636-644`), so exempting the output
pass alone would leave the result exactly as MISSING as it is now and the symptom would look unfixed.

**As-built.** `PRESENT-DIVERGENT` — `workflow.rs:583-593`, `:636-644`, defect pinned at `:3124-3143`.

**Verified by.** SB-ENV-T36, SB-ENV-T37

#### SB-ENV-028 — The mask is recorded in the run's provenance   [P1] [status: ABSENT]

**Requirement.** Every run MUST record which mask curve was applied, or that none was. A curve
produced under a mask MUST be distinguishable from one produced without.

**Rationale.** A masked run and an unmasked run of the same module with the same parameters produce
different curves and are currently indistinguishable after the fact. §2.8: order and context are part
of what produced a number, and a curve that cannot say what conditioned it cannot be reproduced. The
mask is resolved in one place (`workflow.rs:564`) so recording it is a small change with a large
audit return.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T27

#### SB-ENV-029 — Conditioning flags validate their own stated preconditions   [P1] [status: ABSENT]

**Requirement.** `condflag` MUST validate the neutron matrix-scale precondition its documentation
states, and MUST refuse or flag rather than compute a crossover flag from a mismatched pair.

**Rationale.** The precondition is already written down in the right file and the right words —
"NPHI must be in matrix units consistent with RHO_MA … reads about 0.04 low in clean water sand,
right at the `XOVER_MIN` default" (`modules.rs:1261-1264`) — and the stated error is exactly the size
of the threshold it corrupts (`XOVER_MIN` = 0.04, `:1286-1293`). A limestone-scale neutron run
against a sandstone matrix therefore suppresses gas crossover to precisely its own detection
threshold. This is `SB-CORE-003` with the condition in the wrong medium.

**As-built.** `ABSENT` — stated in prose, checked nowhere.

**Verified by.** SB-ENV-T18, SB-ENV-T19

#### SB-ENV-030 — One flag polarity, defined once, as a type   [P0] [status: PRESENT-UNVERIFIED]

**Requirement.** Every flag curve produced anywhere in this domain MUST use a single polarity defined
in exactly one place as a type, not a convention. A second polarity MUST be a compile-time
impossibility. Flag curves MUST carry their flag type as declared metadata so a consumer can
distinguish an exclusion mask from a diagnostic indicator.

**Rationale.** §2.13 (ledger `F-2`): IP ships **three** mutually incompatible polarities inside one
workflow family, including `curve_autoedit`'s `−999` invalid / `1` valid. A flag feeds a mask, and a
mask inverted is not a degraded result — it is the exact complement, deleting the good rock and
keeping the bad, and the output is a complete, plausible, fully populated curve. **No downstream
check can catch it.** `SB-CORE-007`.

**As-built.** `PRESENT-UNVERIFIED` — `1.0 = true` is used consistently across `badhole`, `condflag`
and the Condition family, but there is no enum, no validator and no single definition site, so the
property holds by discipline rather than by construction.

**Verified by.** SB-ENV-T38, SB-ENV-T39

### 4.4 Curve conditioning

#### SB-ENV-031 — The despike cutoff shows its contamination ceiling, live   [P1] [status: ABSENT]

**Requirement.** Wherever a despike cutoff `k` is exposed, the dialog MUST display the **contamination
ceiling of the estimator that will actually run**, updating live as the user changes `k` or switches
method, together with a statement of its meaning: *above this fraction of contaminated samples in a
window, spikes mask each other and are not detected.* The ceiling MUST be computed per branch —
`min(1/k, ½)` for the mean-deviation fallback, 50 % for the true-MAD branch, `1/(k²+1)` for a
`mean ± kσ` estimator should one ever be offered — each under that estimator's own σ convention. **A
single formula applied across methods is a defect under this requirement, not a partial
implementation of it.** *(Jointly owned with `23_plotting-interactivity.md` for presentation.)*

**Rationale.** §2.5. Two properties make this a UI requirement rather than a documentation one. The
ceiling is **independent of spike amplitude** — a 10 g/cc spike masks at the same fraction as a
0.1 g/cc one, because it inflates the scale estimate in exact proportion — so no amount of visual
inspection reveals it. And it **falls as `k` rises**, which is the opposite of what a user believes
they are buying when they raise `k` to stop the filter eating good samples: the cautious-looking
move is the one that spends breakdown resistance. For the shipped Hampel the ceiling is flat at
50 % up to `k = 2` and falls beyond it, so the default `K = 3.0` already sits at 33.3 % — a number
no one chose, because nothing displays it. The formula differs for every method the dropdown
offers, it is cheap to compute, and it cannot be inferred from anything on screen today.

**As-built.** `ABSENT` — `condition.rs:256` presents `K` as a bare number with a 0.5–20.0 range and
no ceiling of any kind; `:253-255` justifies the value 3.0 as a convention but says nothing of what
it costs.

**Verified by.** SB-ENV-T40, SB-ENV-T69, SB-ENV-T70

#### SB-ENV-032 — The MAD consistency constant is defined once, named, and cited   [P2] [status: PRESENT-DIVERGENT]

**Requirement.** The Gaussian consistency constant `C_MAD = 1/Φ⁻¹(3/4) = 1.482602…` MUST exist as one
named constant with a source, used by every robust estimator in the codebase. A second numeric
literal of this constant MUST NOT appear.

**Rationale.** `SB-CORE-007`. The **numeric** divergence today is negligible — the shipped `1.4826`
is within 1.5 parts per million of the exact value and no result changes. The requirement is
structural: this constant will be needed again by the outlier cull of `SB-ENV-036`, by any robust
normalisation, and by the uncertainty work of `SB-ENV-019`, and a second literal is exactly how
`SB-CORE-007`'s four-definition-site problem started elsewhere in this codebase. Fixing it while
there is one site costs nothing; fixing it after there are three costs a migration and an argument.

**As-built.** `PRESENT-DIVERGENT` (structural, not numeric) — `condition.rs:166`, an inline literal
with no name and no source.

**Verified by.** SB-ENV-T41

#### SB-ENV-033 — A degenerate window is declared, not silently substituted   [P2] [status: PRESENT-DIVERGENT]

**Requirement.** Where a robust spread estimate is degenerate — a zero MAD, or fewer samples than the
estimator's minimum — the substitution used MUST be recorded per sample in the module's flag channel,
and the module's documentation MUST state the substitution and why it was chosen.

**Rationale.** A zero MAD means over half the window holds one identical value, which happens in
real data — a clipped tool, a filled interval, a constant-value section from a previous edit. The
fallback then decides the answer, and the user cannot see that a fallback ran. `SB-CORE-002` in
miniature.

**As-built.** `PRESENT-DIVERGENT` — `condition.rs:154-172` falls back from MAD to mean absolute
deviation when `mad == 0.0`, and `MIN_HAMPEL_SAMPLES = 5` (`:176`) sets a floor. Both are correct
choices; neither is reported.

**Verified by.** SB-ENV-T42

#### SB-ENV-034 — Every window, gap and thickness parameter is a thickness in the project's depth unit   [P0] [status: PRESENT-OK]

**Requirement.** No conditioning or framing parameter MAY be expressed in samples. Every window, gap
limit, bed thickness, shoulder width and filter length MUST be stated as a physical thickness in the
project's depth unit and resolved against the curve's own depth column.

**Rationale.** §2.14 (ledger `F-4`): IP documents its filter-length limit **three ways** — 1–121,
3–121 and 2001 — and the deeper problem is that the parameter is in samples at all. SandiBumi's own
module header states the argument better than the dossier does: a window in samples "silently changes
the amount of rock it covers the moment a curve is resampled, or when one curve of a run came in at 2
inches and another at 6 — and nothing downstream can see that it did" (`condition.rs:15-20`). This
requirement exists to prevent regression, not to fix a defect.

**As-built.** `PRESENT-OK` — `condition.rs:15-20, 252`; `frame.rs:242-243`. Resolution via
`Frame::windows`.

**Verified by.** SB-ENV-T43

#### SB-ENV-035 — Smoothing never bridges a gap, and never invents a sample   [P0] [status: PRESENT-OK]

**Requirement.** No smoothing, filtering or averaging operation MAY write a value at a sample that
was MISSING on input. Only an operation that declares itself a gap-filling operation may do so.

**Rationale.** `SB-CORE-002`. `condition.rs:22-27` states the reasoning: filling and smoothing are
different claims, and "a curve that quietly acquired values across a washout reads exactly like one
that was logged there." This **inverts** Geolog's `PRESERVE_MISSING = FALSE` default, which is the
competitive win, and it is recorded here so it cannot be traded away for convenience.

**As-built.** `PRESENT-OK` — `condition.rs:22-27`.

**Verified by.** SB-ENV-T44

#### SB-ENV-036 — Outlier and spurious-population culling exists as a distinct operation   [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide population-level outlier culling as an operation distinct
from despiking, MUST document that culling precedes despiking, and MUST emit a reversible record of
every culled sample.

**Rationale.** Dossier §2.4, §4.4: Geolog's `tp_cull` is adopted wholesale and has no SandiBumi
equivalent. Despiking and culling answer different questions — a spike is a local excursion against
its neighbours, a spurious population is a globally implausible cluster that may be locally smooth.
Techlog states the ordering explicitly (outlier cleaning **before** despike), which is the one local
ordering any incumbent commits to (§2.8), and it exists because a spurious population inflates the
window statistics a despiker depends on.

**As-built.** `ABSENT`.

**Verified by.** SB-ENV-T27

#### SB-ENV-037 — Every removed or replaced sample is recoverable   [P1] [status: PARTIAL]

**Requirement.** Despiking, culling, clipping and gap filling MUST each emit a record from which the
original values can be exactly restored, and the restoration MUST be exercised by a test.

**Rationale.** §2.5: Geolog's despike emits a reversible removed-value flag and that is the right
pattern. Conditioning is a judgement call that reviewers second-guess; a conditioning step that
cannot be undone forces a re-run from the raw import and loses everything downstream of it.

**As-built.** `PARTIAL` — the interactive path has an exact undo (`curve_edit.rs:10-12`: `edit_curve`
returns the previous `(depth, value)` pairs and `restore_curve_values` writes them back). The batch
Condition modules do not: they write a new curve and leave the input untouched, which preserves the
input but does not record **which** samples were changed or what they were.

**Verified by.** SB-ENV-T45

#### SB-ENV-038 — Gap filling states its boundary comparison and refuses an open-ended gap   [P1] [status: PRESENT-OK]

**Requirement.** Gap filling MUST state in its own documentation whether a gap exactly equal to the
maximum is filled, MUST assert that boundary in a test, MUST never fill a gap that is open at either
end of the curve, MUST measure the gap between the live samples either side, and MUST flag every
sample it writes.

**Rationale.** §2.14 (ledger `F-3`): IP's page and its dialog contradict each other on the boundary
comparison. The requirement is not a boundary convention — it is that the convention be stated and
asserted, because an off-by-one at the maximum gap is the difference between inventing a sample and
declining to. The open-ended clause is separate and physical: a gap at the top or bottom of a curve
has data on one side only, so filling it is extrapolation wearing interpolation's name.

**As-built.** `PRESENT-OK` — `condition.rs:687-747`: `MAX_GAP` is `param_open` (`:695`), a gap open at
either end is never filled (`:718-720`), the gap is measured between live samples (`:726`), every
filled sample is flagged.

**Verified by.** SB-ENV-T46

#### SB-ENV-039 — Clip refuses rather than repairs   [P2] [status: PRESENT-OK]

**Requirement.** Clipping MUST refuse to run with neither bound supplied, and MUST refuse a reversed
bound pair rather than swapping it.

**Rationale.** A tool that silently swaps a reversed pair has decided the user made a typo. A tool
that refuses has noticed that it cannot know whether the user inverted the pair or inverted their
intent. The optional form is reserved for a genuine one-sided bound, where leaving one side empty is
a statement that the curve is unbounded there rather than an omission (`modules.rs:156-158`).

**As-built.** `PRESENT-OK` — `condition.rs:591-634`.

**Verified by.** SB-ENV-T47

#### SB-ENV-040 — A conditioning output is never the input's own mnemonic   [P0] [status: PRESENT-OK]

**Requirement.** No conditioning, correction or framing module MAY write its output under the input
curve's own standard mnemonic. Such a name MUST be refused **by name, with the reason**, before the
module runs.

**Rationale.** `condition.rs:28-37` states the mechanism and the stakes: `fetch_curve_frame` resolves
the six standard mnemonics from `standard_curves` **first**, falling through to `computed_curves`
only when the standard column is entirely NaN. A despiked curve written back as plain `GR` would be
"stored, counted, reported — and invisible to every module, plot and export that reads GR", and the
run would report success. `workflow::resolve_output_names` refuses such a name in one place in front
of every module rather than in a copy per module.

**As-built.** `PRESENT-OK` — `condition.rs:28-37`, `workflow::resolve_output_names`.

**Verified by.** SB-ENV-T48

#### SB-ENV-041 — The filter kernel and its normalisation are declared in the output   [P2] [status: PRESENT-UNVERIFIED]

**Requirement.** A smoothing operation MUST record which kernel it applied and how that kernel was
normalised, including its behaviour at the ends of the curve and at the edges of gaps.

**Rationale.** Dossier §2.5: the three incumbents split three ways on kernel normalisation, and the
same nominal filter length gives different answers under each. A smoothed curve whose kernel is not
recorded cannot be reproduced, and reproduction is what a QC review consists of.

**As-built.** `PRESENT-UNVERIFIED` — smoothing exists in `condition.rs`; the kernel choice is a
parameter and is not carried into the output's provenance.

**Verified by.** SB-ENV-T49

#### SB-ENV-042 — Interactive edits carry provenance, not only undo   [P1] [status: PARTIAL]

**Requirement.** An interactively edited curve MUST carry a persistent record of every edit — the
operation, the interval, the parameters and the time — retrievable without inspecting an undo stack.

**Rationale.** An undo stack is a session artifact; a deliverable outlives the session. An
interactively shifted, blanked or scaled curve is indistinguishable from an unedited one in the
delivered project, which defeats every audit `SB-CORE-010` exists for. The batch path has run
provenance; the interactive path is the hole in it.

**As-built.** `PARTIAL` — `curve_edit.rs:10-12` gives an exact undo via returned `(depth, value)`
pairs and a transactional rewrite; no persistent edit record exists.

**Verified by.** SB-ENV-T45

### 4.5 Formation temperature and the geothermal gradient

#### SB-ENV-043 — One formation-temperature definition, one mnemonic   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** Exactly one implementation MAY produce the formation-temperature curve. Any
additional module offering formation temperature MUST delegate to it, as `gr_normalize` delegates to
`normalize`. Two implementations MUST NOT emit the same mnemonic.

**Rationale.** `SB-CORE-007`, whose requirement was extended on 2026-08-07 to cover output-mnemonic
ownership specifically because of this instance, and which now carries it as its third verified
instance with the build gate `SB-CORE-T23`. Note it is **not** an `SB-CORE-006` instance despite the
surface resemblance: both modules compute the same linear-trend equation, so `SB-CORE-T17` — which
hands both engines one shared fixture — would **pass**. The divergence lives entirely in the shipped
defaults, which is why `SB-CORE-T23` forbids the fixture from supplying parameters (`SB-CORE-015`).
This is the domain's native instance and it is quantified in §3.4: two
shipped modules both emit `FTEMP` and, on their own defaults at 2,000 m TVD, disagree by **33.1 °C**
(86.7 °C from `ftemp_grad`, 119.8 °C from `precalc`'s feet-based fit). Whichever ran last is what
every downstream consumer reads — including `nphi_env_corr` (`modules.rs:1869`), `gascorr` and the
whole saturation chain. Through `Rw(T) = Rw_ref·(T_ref + 21.5)/(T + 21.5)` and Archie's `Sw ∝ √Rw`,
a 33.1 °C error inflates `Rw` by 141.3/108.2 = 1.306 and `Sw` by **14.3 % relative** — a true `Sw` of
0.35 delivered as 0.40.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1055` (`ftemp_grad`) and `modules.rs:1170`
(`precalc`) both emit `FTEMP`.

**Verified by.** SB-ENV-T50, SB-ENV-T51, and `SB-CORE-T23` at the spine.

#### SB-ENV-044 — Formation temperature is a function of true vertical depth   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** The geothermal trend MUST be evaluated against true vertical depth. Where TVD is
unavailable, the module MUST refuse or emit an explicitly flagged result stating that measured depth
was substituted — it MUST NOT substitute silently.

**Rationale.** The geothermal gradient is a property of vertical depth; in a deviated well measured
depth over-states it monotonically, and the error grows with deviation. The two shipped modules
disagree on this point (`ftemp_grad` uses `ctx.log("DEPTH")` at `modules.rs:1031`; `precalc` uses
TVDSS at `:1144`), which means the divergence in `SB-ENV-043` is partly a depth-reference divergence
wearing a defaults divergence. `precalc` already has the correct fallback *shape* at
`modules.rs:1122-1124` — what it lacks is the declaration that a fallback occurred.

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:1031, 1051` (measured depth); `:1122-1124` (silent
fallback).

**Verified by.** SB-ENV-T51, SB-ENV-T52

#### SB-ENV-045 — The geothermal gradient carries a declared, validated compound unit   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A geothermal gradient parameter MUST declare both its temperature unit and its
length unit explicitly, and the runner MUST validate the length unit against the project's depth
unit. A gradient MUST NOT be accepted as a bare number.

**Rationale.** §2.7: four conventions across three tools, one of them internally unresolved (ledger
`F-5`). A realistic 3 °C/100 m is 0.01646 °F/ft. Entering `3` into a °F/ft field heats the well
182 °F per 100 ft — loud. Entering `0.0165` into a °C/100 m field gives an **essentially isothermal
well** — silent, smooth and plottable, and it propagates into every temperature-dependent `Rw`,
every neutron temperature correction and every resistivity temperature correction. §3.4 reproduces
both directions inside SandiBumi: `precalc`'s feet-based defaults on a metric project give 53.9 °C
where 119.8 °C belongs (a **66 °C** error), and `ftemp_grad`'s °C/m gradient on a foot project gives
**223.6 °C**. `SB-CORE-001`; `02_RISKS_AND_CONTRADICTIONS.md` R14's shape on the one curve that feeds
everything temperature-dependent.

**As-built.** `PRESENT-DIVERGENT` — `TGRAD` declared `"degC/m"` (`modules.rs:1022`) and `TEMP_GRAD`
declared `"deg/ft|m"` (`:1091`); neither is validated against the project's depth unit.

**Verified by.** SB-ENV-T52, SB-ENV-T53

#### SB-ENV-046 — A mudline / water-bottom branch exists for offshore wells   [P2] [status: ABSENT]

**Requirement.** Formation temperature MUST offer a water-bottom branch in which the trend is
referenced to the mudline rather than to the surface, with the mudline depth as a declared input, and
the branch MUST be selected from a validated enumeration rather than by string fall-through.

**Rationale.** Dossier §2.9, §4.8: Geolog's `ftemp` ships exactly two branches and the mudline branch
is the offshore case — the water column is not part of the geothermal trend, so a surface-referenced
gradient over-states temperature by the whole water depth's worth of gradient. The
fall-through clause is not decoration: `ftemp.lls:53` tests `== 'MEASUREMENT_REFERENCE'` and routes
**every other string, including a typo**, to the mudline branch, and only its manifest's enumeration
prevents that (§2.9). SandiBumi must adopt the two branches **and** the enumeration that gates them,
in the code (`SB-ENV-009`).

**As-built.** `ABSENT` — both shipped modules implement a surface-referenced linear geotherm only.

**Verified by.** SB-ENV-T54

#### SB-ENV-047 — A declared parameter that does not enter the answer is removed or used   [P1] [status: PRESENT-OK]

**Requirement.** A module MUST NOT declare a parameter its computation does not consume. Either the
parameter is used, or it is removed from the spec.

**Rationale.** Because `ArgSpec` drives the auto-generated dialog, the declaration **is** the promise
to the user. A declared parameter that is discarded is worse than an absent one: the dialog invites
the user to enter a value, the user believes the answer was anchored to it, and it was not — and
nothing in the output can reveal the difference. The requirement exists to keep that property true
under change, not to repair a live defect.

> **Correction, 2026-08-07 — this requirement previously carried a false instance and a
> `PRESENT-DIVERGENT` status.** The first draft asserted that `ftemp_grad` declares `BHT` and
> `TD_BHT` (`modules.rs:1023-1024`) and never consumes them, citing `:1051`. That reading is wrong.
> `:1051` is the `else` arm of a two-branch `if bht_mode` selected by `OPT_FT` (`:1019`,
> `:1032`, `:1040`); the BHT branch consumes **both** parameters — `BHT` at `:1041` and `:1049`,
> `TD_BHT` at `:1042`, `:1046` and `:1049` — as `tsurf + (bht − tsurf)·d/td`, exactly the
> BHT-anchored interpolation the spec doc promises at `:1015-1016`. The branch is also guarded
> (`td <= 0.0` → skip, `:1046-1048`, against a zone override producing a finite-looking ±∞) and
> covered by tests at `modules.rs:3038`, `:3047` and `:3081`. The as-built status is corrected to
> `PRESENT-OK` and the "resolve by implementing a BHT-anchored gradient" obligation is withdrawn:
> it is already implemented. Recorded rather than deleted, because a requirement whose only
> evidence evaporated is evidence about how this chapter was built.

**As-built.** `PRESENT-OK` — every declared parameter in this domain's module specs is consumed on
some reachable branch. `SB-ENV-T55` is the build gate that keeps it so; it passes today.

**Verified by.** SB-ENV-T55

#### SB-ENV-048 — The resistivity temperature constant is defined once, cited, and surfaced   [P0] [status: PRESENT-UNVERIFIED]

**Requirement.** The resistivity temperature-correction constant MUST exist as exactly one named,
cited constant, expressed in one unit system with the conversion derived rather than tabulated, and
its value MUST be visible to the user wherever a temperature-corrected `Rw` is computed.

**Rationale.** §2.3 and `SB-CORE-007`. Techlog's `Exxon` branch and Geolog's shipped constant are
provably the same law in two unit systems — `c = 6.77` °F ≡ `c = 21.5` °C — which is two independent
implementations agreeing. A third branch, labelled `Arps`, uses `c = −6` °F and stands alone, and it
is reached as the **fall-through** for any method string that is not exactly `'Exxon'`
(`TempCorr_Resistivity.py:80-83`). Quantified at `tref = 200 °F`, that is **−13.8 % on `Rw` and
−7.2 % on `Sw` at 60 °F** — worst exactly where surface `Rw` and `Rmf` references are quoted.

**As-built.** `PRESENT-UNVERIFIED` — the correction is applied in `multimin2::fluid_calc`, consumed
by `resultsqc.rs:106-115`; the constant is neither named nor surfaced, and no test pins it against
both unit systems.

**Verified by.** SB-ENV-T56, SB-ENV-T57

#### SB-ENV-049 — A superseded module delegates to the survivor and says so   [P1] [status: PRESENT-OK]

**Requirement.** Where a module is superseded by a more general one, it MUST remain runnable so saved
chains and stored runs still resolve, MUST delegate to the survivor rather than re-implement it, MUST
be hidden from the pickers, and MUST record in source why it was kept.

**Rationale.** `SB-CORE-006`. This is the pattern that prevents §2.3's failure from arising natively,
and SandiBumi has already executed it correctly once. `gr_normalize` maps its GR-specific argument
names onto the universal ones and delegates (`modules.rs:2650-2672`), its doc states "it is NOT a
second implementation" (`:2640`), and it explains the choice: retiring it "would fail every saved
chain carrying a `gr_normalize` step, and unlike superseded physics the answer here is unchanged"
(`:2648-2649`). Recorded as a requirement so the next supersession follows it.

**As-built.** `PRESENT-OK` — `modules.rs:2639-2673`.

**Verified by.** SB-ENV-T58

#### SB-ENV-050 — A depth-trend parameter is well-scoped, and a compartment parameter is not   [P1] [status: PRESENT-OK]

**Requirement.** A parameter defining a **continuous trend against depth** MUST refuse a named-zone
override. A parameter defining a quantity that may legitimately step at a boundary MUST NOT carry
that restriction. The distinction MUST be recorded per parameter with its physical justification.

**Rationale.** `modules.rs:66-87` states it exactly: because the trend is computed from surface at
every sample rather than integrated down through the zones above, a per-zone gradient makes the
profile **jump** rather than bend — "a 0.03 °C/m well with a 0.035 override below 1500 m stepped
**10.5 °C across 100 m** where the undisturbed trend rises 3.0. Rock temperature is continuous — a
10 °C discontinuity at a formation top is not something the earth does — and it does not stay in
FTEMP, because the Arps correction turns temperature into Rw and Rw goes straight into Sw." The
asymmetry is the requirement's substance: the same restriction is deliberately **not** applied to
`PSURF`/`PGRAD`, because "a pressure step at a formation top is a pressure compartment, which is a
real thing rock does."

**As-built.** `PRESENT-OK` — `ArgSpec.well_scope` (`modules.rs:86-87`), applied via `param_well` to
`TSURF`/`TGRAD`/`BHT`/`TD_BHT` (`:1021-1024`) and `SURF_TEMP`/`TEMP_GRAD` (`:1090-1091`).

**Verified by.** SB-ENV-T59

### 4.6 Normalisation, QC limits and depth units

#### SB-ENV-051 — Percentiles are exact order statistics, never histogram bin means   [P0] [status: PRESENT-OK]

**Requirement.** Every percentile used in normalisation, QC limiting or endpoint selection MUST be an
exact order statistic computed on sorted values. A histogram-bin approximation MUST NOT be used.

**Rationale.** §2.6. Geolog's histogram implementation is wrong four ways — the bottom bin is never
counted (`log_normalization.lls:310, 318`), the top bin is dead (`:228`), the upper walk has no
termination guard and becomes reachable at **exactly `PCT_MAX = 97`**, and the returned value is a
bin mean rather than the percentile, quantised at ±½ bin ≈ **1.8 gAPI** on a 180 gAPI range. Order
statistics are structurally immune to all four. Quantified: a 3.6 gAPI shift on the clean endpoint is
**≈ 4.5 % `Vsh` absolute**, field-wide and systematically in one direction, since
`∂IGR/∂GR_clean = (GR − GR_shale)/span² = −0.0125` per gAPI at span 80.

**As-built.** `PRESENT-OK` — `condition.rs:991-998` sorts before calling `distribution::percentile`,
with a comment stating that a percentile on a depth-ordered slice "returns whatever value happens to
sit 3% of the way down the well."

**Verified by.** SB-ENV-T60

#### SB-ENV-052 — The normalisation reference pair ships ABSENT   [P0] [status: PRESENT-OK]

**Requirement.** The reference percentile values a normalisation maps onto MUST ship with no default.
The run MUST refuse until the user supplies them.

**Rationale.** A reference pair is a field calibration. Shipping one makes somebody's basin the
default for every basin, and a normalised curve looks entirely plausible whichever pair produced it.
`param_open`'s own doc states the general principle (`modules.rs:145-158`).

**As-built.** `PRESENT-OK` — `REF_LOW`/`REF_HIGH` are `param_open` and the run refuses without them
(`condition.rs:974-980`).

**Verified by.** SB-ENV-T61

#### SB-ENV-053 — Normalisation is recorded, reviewable and overridable per well   [P1] [status: ABSENT]

**Requirement.** A normalisation run MUST record per well the reference pair, the computed well
percentiles, the resulting linear map and the interval the percentiles were computed over; the result
MUST be reviewable per well before acceptance; and a per-well manual override MUST be supported and
recorded as an override.

**Rationale.** §2.6, final paragraph: a delivered carbonate study required manual post-normalisation
adjustment in specific wells where automatic normalisation produced unrealistically low shale values
(T3). The software behind that run is not identified and the direction does not match the code-read
defect, so this is **not** offered as evidence for that defect — it is firm, independent evidence
that delivered work has needed per-well review and override, and currently has neither.

**As-built.** `ABSENT` — the map is computed and applied; nothing is recorded or reviewable.

**Verified by.** SB-ENV-T62

#### SB-ENV-054 — Normalisation percentiles are computed over a declared common interval   [P1] [status: PARTIAL]

**Requirement.** A multi-well normalisation MUST record the interval over which each well's
percentiles were computed and MUST warn when those intervals are not comparable across the set.

**Rationale.** A two-point percentile map is only meaningful if every well's percentiles were
measured over comparable rock. `gr_normalize`'s own doc instructs the user to "mask the run to a
common reference interval so every well is measured over comparable rock" (`modules.rs:2615-2616`) —
correct advice, delivered as prose, unenforced and unrecorded. A set normalised over different
intervals per well is indistinguishable afterwards from one normalised correctly.

**As-built.** `PARTIAL` — the mask mechanism exists (`workflow.rs:557-593`) and correctly excludes
masked samples from the percentile computation, which is the hard half. What is absent is recording
the interval and comparing it across wells.

**Verified by.** SB-ENV-T62, SB-ENV-T63

#### SB-ENV-055 — A normalisation reference pair is named and sourced separately from a `Vsh` endpoint pair   [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The clean and shale GR values used as a normalisation reference MUST be distinct,
separately named and separately sourced parameters from the clean and shale GR endpoints used in a
`Vsh` transform, even where a project chooses the same numbers for both.

**Rationale.** `SB-CORE-007`. `04_CORE_REQUIREMENTS.md` records four definition sites for the
clean/shale GR endpoints across this codebase, giving a **22.2 % `Vsh` spread at GR 70 gAPI**, and
`gr_normalize`'s reference defaults of 20/120 (`modules.rs:2631-2632`) are one of them — its doc says
so explicitly, "matching `vsh_gr`'s `GR_MA`/`GR_SH`" (`:2620`). These are different quantities: a
normalisation reference is where you are mapping the well *to*, and a `Vsh` endpoint is where clean
rock and shale *are*. Coupling them means that correcting a `Vsh` endpoint silently re-normalises
every curve in the project, and the re-normalisation is invisible because both curves still look
right.

`gr_normalize`'s documentation is otherwise exemplary on this point and should be preserved verbatim
— "SET YOUR OWN FIELD REFERENCE PAIR — that is the entire point of the module … A reference pair
from one basin is the wrong reference in another" (`:2618-2622`).

**As-built.** `PRESENT-DIVERGENT` — `modules.rs:2631-2632`.

**Verified by.** SB-ENV-T64

#### SB-ENV-056 — Log-QC limits ship ABSENT, and band precedence is specified once   [P1] [status: ABSENT]

**Requirement.** Log-QC user limits and extreme limits MUST ship with no numeric defaults. The
**precedence semantics** between the two bands MUST be specified once and enforced: the extreme band
MUST bracket the user band, and a configuration in which it does not MUST be refused at entry.

**Rationale.** §2.10. IP's shipped user limits would flag most real logs — its own ingest says so,
and a GR user-minimum of 59 gAPI sits **above** a delivered field's clean-sand P3 of 53.68 gAPI, so
the entire clean-sand population of a real study would be flagged out of range. Ledger `F-1`:
extreme-low GR (117) **exceeds** user-min GR (59), inverting the semantics, while the same page
discusses flagging GR below zero — which a lower bound of 117 cannot express. One of the two shipped
panels is wrong and which one is OPEN. The semantics, unlike the numbers, are general and can be
specified now.

**As-built.** `ABSENT` — no QC-limit facility exists.

**Verified by.** SB-ENV-T65, SB-ENV-T66

#### SB-ENV-057 — One token for "a length in the project's depth unit", validated once   [P0] [status: PRESENT-DIVERGENT]

**Requirement.** A parameter expressing a length in the project's depth unit MUST declare it with a
single token, defined once. A module MUST NOT assert a specific length unit in a parameter
declaration or a doc string where the arithmetic is in fact unit-agnostic.

**Rationale.** `SB-CORE-007` and `SB-CORE-001`. Three tokens for one meaning are in the shipped code:
`"depth"` (`condition.rs:252`, `frame.rs:242-243`), `"m|ft"` (`modules.rs:1294-1295`) and `"m"`
(`modules.rs:2512`). The last is the harmful one — `depth_shift`'s arithmetic applies the shift in
whatever unit the depth column carries (`:2563`), so on a foot project the manifest and the doc's
"Shifts CURVE by SHIFT **metres**" (`:2506`) are simply false, and the label is the only thing the
user sees. `condflag`'s doc compensates in prose — "the defaults suit metres, roughly triple them for
feet" (`:1275-1276`) — which is the right warning by the wrong mechanism.

**As-built.** `PRESENT-DIVERGENT` — three tokens; `modules.rs:2506, 2512` assert metres wrongly.

**Verified by.** SB-ENV-T43, SB-ENV-T67

### 4.7 Independently derived capability

#### SB-ENV-058 — Borehole-image speed correction, derived independently   [P3] [status: ABSENT]

**Requirement.** Where SandiBumi supports borehole-image logs, it MUST provide tool-speed correction
derived independently from published sources: an **accelerometer-integration** path, which recovers
true tool displacement from the tool's own z-axis accelerometer and re-maps the image onto corrected
depth, and an **inter-pad / inter-pass cross-correlation** path, which recovers relative displacement
from the image data itself where accelerometer data is absent. The correction MUST emit the
displacement it applied as a curve, and MUST NOT be applied silently. **The entropy-optimisation
formulation MUST NOT be used, approximated, renamed or reverse-engineered.**

**Class.** Tier C. The contract's register (`CONTRACT.md` §2.2) classifies entropy image
speed-correction as **C-3 — opaque artifact**. **This chapter escalates that classification** —
SandiBumi's own Tier-C register describes the same item as "**stated patented** (algorithm only)"
(`docs/research_2026-07/ip_ingest/D_tierC_register.md:43`), which is C-1 terms, under which
re-derivation does not clear a granted method claim and a design-around must be checked against the
claims. The two classifications imply different obligations. See escalation `ESC-6`, §7.2.

**Betters:** the incumbent's speed correction is embedded in a compiled image engine, so a user
cannot see what displacement was applied, cannot audit it, and cannot reproduce it from the delivered
data. Both paths specified here **emit the applied displacement as a curve**, which makes the
correction reviewable and reversible — and the accelerometer path is derived from a measured quantity
rather than from an image-quality objective, so it does not optimise an image toward looking correct.

**Rationale.** `CONTRACT.md` §2.2 as amended: what is prohibited is the derivation path, not the
capability, and a chapter owning the user need must specify an independently derived capability
rather than decline. Stick-slip depth distortion is the dominant artifact in wireline image logs and
nothing downstream can correct for it. SandiBumi's own Tier-C register already identifies the
open design-around and names the boundary precisely: "accelerometer-based speed correction (industry
standard) or image cross-correlation depth-shifting between pads/passes. **The entropy-optimization
step is the patented novelty — avoid it specifically; the accelerometer/cross-correlation approaches
are open**" (`D_tierC_register.md:47`).

**Acquisition gap.** No primary paper for either path is held on this machine, so no equation and no
coefficient is specified here — per `CONTRACT.md` §2.2's terms, the specific missing sources are named
and escalated rather than guessed. See `ESC-7`, §7.2. **P3 because image logs are not v1**
(`D_tierC_register.md:48`); the requirement is recorded now so the capability is not later built by
the prohibited path for want of a specification.

**Verified by.** SB-ENV-T68 *(specified as a contract test; the numeric tests follow the sources)*

---

## 5. Parameters

83 rows. **32 ship `ABSENT — ships with no default`.** Grouped by method for scanning; this is
the single place every number in the domain appears.

Three conventions apply throughout:

- **`ABSENT — ships with no default`** means the field opens empty and the run refuses. It is used
  wherever the evidence holds competing values with no basis for choosing, or wherever the quantity
  is a measured property of a specific well. This is the `param_open` mechanism that already exists
  (`modules.rs:145-158`).
- **`NON-ADOPTABLE — cited for verification`** means a vendor or house value recorded so an
  implementation can be checked against it, and **not** adopted as a SandiBumi default. Every such
  row exists to make a divergence checkable, not to supply a number.
- **`SHIPPED-UNCITED`** in the Source column means SandiBumi currently ships that value with no
  traceable source. Every one of these is a `SB-ENV-014` or `SB-ENV-004` violation and is listed so
  the count is visible: there are **29**.

### 5.1 Environmental corrections

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| GR hole-size correction coefficient | `K_GR` | `ABSENT — ships with no default` (ships 0.0075) | 1/in | `modules.rs:1816-1817` — `SHIPPED-UNCITED`; family header calls it "chartbook-magnitude" (`:1800`) with no chartbook named | — |
| GR tool position | `GR_TOOL_POS` | `ABSENT — ships with no default`; enumeration required | — | Geolog `unc_gr.info:40-41` VALIDATION enumerates four spellings — capability evidence for the *existence* of the input | T1 |
| GR tool size | `GR_TOOL_SIZE` | `ABSENT — ships with no default`; enumeration required | in | Geolog `unc_gr.info:40-41` VALIDATION | T1 |
| Neutron temperature coefficient | `K_TEMP` | `ABSENT — ships with no default` (ships 0.0001) | v/v per °C | `modules.rs:1862` — `SHIPPED-UNCITED` | — |
| Neutron temperature effect, magnitude check | — | −0.048 (≡ −2.4 p.u. per 50 °F ≡ 0.000864 v/v per °C) | p.u./°F | `memory\reference_log_qc_gates.md` §Neutron — **`NON-ADOPTABLE — cited for verification`**. A linearised house gate over a 50 °F span, not a tool coefficient. Cited because the shipped `K_TEMP` is **8.6× below** it | T4 |
| Neutron chart reference temperature | `T_REF` | `ABSENT — ships with no default` (ships 24.0) | °C | `modules.rs:1863` — `SHIPPED-UNCITED` | — |
| Neutron salinity coefficient | `K_SAL` | `ABSENT — ships with no default` (ships −0.002) | v/v per 100 kppm | `modules.rs:1864` — `SHIPPED-UNCITED` | — |
| Formation water salinity | `SALW` | `ABSENT — ships with no default` (ships 20000) | ppm | `modules.rs:1865` — `SHIPPED-UNCITED`; three vendor values span five orders of magnitude (§2.15) | — |
| Formation water salinity, vendor reference | — | 50 | kppm | Geolog `evs_tnph.info` — **`NON-ADOPTABLE — cited for verification`** | T1 |
| Formation/borehole salinity, vendor reference | — | 2.8E-4 (= 0.28 ppm) | Kppm | IP2025 CNL tab, ledger `F-14` — **`NON-ADOPTABLE`**, confirmed nonsense; fresher than distilled water | T2 |
| Formation salinity, vendor reference | — | 0 | kppm | GE documentation — **`NON-ADOPTABLE — cited for verification`** | T2 |
| Borehole (mud filtrate) salinity | — | `ABSENT — ships with no default` | ppm | measured property of the well; `SB-ENV-016` | — |
| Standoff | `SOCN` | `ABSENT — ships with no default` | in | `SB-ENV-016` | — |
| Standoff, vendor default | — | 0.5 | in | Geolog `evs_tnph.info` — **`NON-ADOPTABLE`**. Cited because the house gate states this exact value "makes NPHI read ~2 p.u. too high" in a 12¼″ hole (§2.2): the vendor default *is* the warning case | T1/T4 |
| Standoff, vendor default | — | 0 | in | IP2025 CNL tab — **`NON-ADOPTABLE — cited for verification`** | T2 |
| Mudcake thickness | — | `ABSENT — ships with no default` | in | measured property; `SB-ENV-013`, `SB-ENV-016` | — |
| Mudcake density | — | `ABSENT — ships with no default` | g/cc | measured property; `SB-ENV-013` | — |
| Mud weight | — | `ABSENT — ships with no default` | lb/gal | measured property; `SB-ENV-016` | — |
| Mud weight, chart span (normal-mud branch) | — | 8–13 | lb/gal | Geolog `unc_tnph.lls:340` — **`NON-ADOPTABLE — cited for verification`**. A stated validity span, not chart data; recorded because the module header states a single 8–18 (`:68`) and a per-module condition would be wrong for this branch (`SB-ENV-001`(c)) | T1 |
| Mud weight, chart span (barite branch) | — | 8–18 | lb/gal | Geolog `unc_tnph.lls:346` — **`NON-ADOPTABLE — cited for verification`** | T1 |
| Mud base | `MUDBASE` | `ABSENT — ships with no default`; enumeration required | — | Geolog `evs_tnph.info` ships `WATER`; enumeration cited, value not adopted | T1 |
| Mud type | `MUDTYPE` | `ABSENT — ships with no default`; enumeration required | — | Geolog `evs_tnph.info` ships `NORMAL`; enumeration cited, value not adopted | T1 |
| Neutron temperature input clamp | — | 50–300 | °F | Geolog `evs_tnph.info:75-83` — **`NON-ADOPTABLE — cited for verification`**. A stated validity condition, cited as the model for `SB-ENV-001`; SandiBumi's own clamp follows its own chart | T1 |
| Compressibility multiplier | — | 4 | — | IP2025 CNL tab — **`NON-ADOPTABLE — cited for verification`** | T2 |
| Formation pressure | `FPRESS` | `ABSENT — ships with no default` | kPSI | `SB-ENV-011`; Geolog declares the unit, value is a well property | T1 (unit only) |
| Density hole correction coefficient | `K_RHO` | `ABSENT — ships with no default` (ships 0.004) | g/cc per in | `modules.rs:1906` — `SHIPPED-UNCITED` | — |
| Reference hole diameter | `HD_REF` | `ABSENT — ships with no default` (ships 10.0) | in | `modules.rs:1907` — `SHIPPED-UNCITED`; a property of tool and bit, not of the software (`SB-ENV-013`) | — |
| Neutron matrix scale | `LITHSCALE` | `ABSENT — ships with no default`; enumeration `{SANDSTONE, LIMESTONE, DOLOMITE}` | — | Geolog `unc_tnph.lls:204-224` — the **enumeration and the fail-loud `else`** are adopted (`SB-ENV-012`); no value is adopted | T1 |

### 5.2 Bad-hole and conditioning flags

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Density-correction bad-hole cutoff | `DRHO_MAX` | `ABSENT — ships with no default` (ships 0.05) | g/cc | `modules.rs:1195` — `SHIPPED-UNCITED`, and matching **none** of the seven precedent values | — |
| — preset, tight-tolerance clastic study | — | 0.02 | g/cc | `project-kb` `bunga-block-phe-posco.md:34` — selectable cited preset (`SB-ENV-024`) | T3 |
| — preset, carbonate studies + ITB module gate | — | 0.15 | g/cc | `carbonate-genting-oil.md:34`, `kelok-phr.md:44`, `memory\reference_log_qc_gates.md` §Density | T3/T4 |
| — vendor value, IP2025 | — | 0.1 | g/cc | IP25-F §4.1 — **`NON-ADOPTABLE — cited for verification`** | T2 |
| Differential-caliper cutoff | `DCAL_MAX` | `ABSENT — ships with no default` (ships 1.0) | in | `modules.rs:1196` — `SHIPPED-UNCITED`, and **half** the value used by every delivered study | — |
| — preset, five delivered studies | — | 2 | in | `carbonate-genting-oil.md:34`, `genting-v2-well-database.md:28`, `kelok-phr.md:44`, `lqr-balam-south-phr.md:29`, `ggr-pusaka-bob.md:36` | T3 |
| — preset, caliper-change term | — | 0.3 | in | `carbonate-genting-oil.md:34`, `genting-v2-well-database.md:28` | T3 |
| Bit size | `BS` | `ABSENT — ships with no default` (ships `BS_DEF` 8.5) | in | `modules.rs:1197` — `SHIPPED-UNCITED`; invents hole geometry, silent in the slim-hole direction (`SB-ENV-025`) | — |
| DRHO unit | — | declared per curve; no default | g/cc \| kg/m³ | `SB-ENV-026`. Geolog `badhole.info` declares `k/m3`, `unc_ldt.lls:59,98` declares `G/C3` for the same name — a 1000× bridge trap | T1 |
| Matrix density (conditioning flags) | `RHO_MA` | 2.645 | g/cc | `modules.rs:1280` — `SHIPPED-UNCITED` **and** one side of `SB-CORE-007`'s 2.645/2.65 split. The **value** is owned by `13_mineral-solver.md` / `11_porosity.md`; this row records that `condflag` is a fourth consumer and must read the single definition, not carry its own | — |
| Fluid density (conditioning flags) | `RHO_FL` | 1.0 | g/cc | `modules.rs:1281` — `SHIPPED-UNCITED`; value owned by `11_porosity.md` | — |
| Coal density ceiling | `COAL_RHOB` | 1.9 | g/cc | `modules.rs:1282` — `SHIPPED-UNCITED` | — |
| Coal neutron floor | `COAL_NPHI` | 0.35 | v/v | `modules.rs:1283` — `SHIPPED-UNCITED` | — |
| Coal sonic floor | `COAL_DT` | 100.0 | µs/ft | `modules.rs:1284` — `SHIPPED-UNCITED` | — |
| Tight-zone porosity ceiling | `TIGHT_PHI` | 0.05 | v/v | `modules.rs:1285` — `SHIPPED-UNCITED` | — |
| Gas-crossover threshold | `XOVER_MIN` | 0.04 | v/v | `modules.rs:1286-1293` — `SHIPPED-UNCITED`. Recorded because the module's own doc states a limestone-scale neutron against a sandstone `RHO_MA` reads "about 0.04 low" (`:1261-1264`) — the matrix-scale error is exactly the size of this threshold (`SB-ENV-012`, `SB-ENV-029`) | — |
| Minimum flagged bed thickness | `MIN_THICK` | 0.25 | project depth unit | `modules.rs:1294` — `SHIPPED-UNCITED`; declared `"m\|ft"`, one of three tokens (`SB-ENV-057`) | — |
| Shoulder width | `SHOULDER` | 0.5 | project depth unit | `modules.rs:1295` — `SHIPPED-UNCITED` | — |
| Flag polarity | — | `1 = true`, single definition, as a type | — | `SB-ENV-030`. IP ships three polarities in one family (ledger `F-2`); Geolog ships one | T1/T2 |
| Log-QC user limits | — | `ABSENT — ships with no default` | per curve | `SB-ENV-056`; IP's own ingest §5.9 states its shipped values "are not physical limits" | T2 |
| Log-QC extreme limits | — | `ABSENT — ships with no default` | per curve | `SB-ENV-056`; ledger `F-1` — IP's extreme-low GR (117) exceeds its user-min GR (59) | T2 |
| — vendor values, IP2025 | — | user GR 59–168, density 1.8–3, neutron −0.1–0.6, DTC 40–240; extreme GR 117–256, density 1.5–3.5, neutron −0.2–1, DTC 40–240 | mixed | IP25-F §4.1 — **`NON-ADOPTABLE — cited for verification`**. Recorded whole so the `F-1` inversion is checkable; a GR user-minimum of 59 sits above a delivered field's clean-sand P3 of 53.68 | T2 |

### 5.3 Curve conditioning

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Despike / smoothing window | `WINDOW` | `ABSENT — ships with no default` | project depth unit | `condition.rs:252` (`param_open`); Jauhar, 2026-08-05: *"No default — I set it every run"*, rationale at `modules.rs:145-158` | T4 |
| Hampel deviation multiple | `K` | 3.0 | — | `condition.rs:256`, comment `:253-255` — `SHIPPED-UNCITED`, self-declared "ordinary three-deviation convention … NOT a field calibration". Contamination ceiling `f* = min(1/k, ½)` = **33.3 %** at this value, against the **50 %** available at any `k ≤ 2`; must be displayed (`SB-ENV-031`) and the default itself is escalated at `ESC-16` | — |
| — vendor value, IP `SpikeCutoff` | — | 2 | SD | IP — **`NON-ADOPTABLE — cited for verification`**. Ceiling `f*` = **exactly 20 %** (population σ) or 19.19 % (sample σ, N=20); on a non-robust mean±kσ estimator this is the failure, not a tuning choice (§2.5) | T2 |
| Despike absolute threshold | `THRESH` | `ABSENT — ships with no default` | curve unit | `condition.rs:255` (`param_open`) | — |
| Despike maximum rate of change | `MAX_RATE` | `ABSENT — ships with no default` | curve unit per depth unit | `condition.rs:256` (`param_open`) | — |
| Maximum fillable gap | `MAX_GAP` | `ABSENT — ships with no default` | project depth unit | `condition.rs:695` (`param_open`) | — |
| MAD Gaussian consistency constant | `C_MAD` | 1.482602 (= 1/Φ⁻¹(3/4)) | — | derived, not measured; one named constant required (`SB-ENV-032`). Shipped as the inline literal `1.4826` at `condition.rs:166` — within 1.5 ppm, so **no result changes**; the requirement is structural | — |
| Minimum samples for a Hampel window | `MIN_HAMPEL_SAMPLES` | 5 | samples | `condition.rs:176` — `SHIPPED-UNCITED`. A sample count is correct here: it is a property of the estimator, not of the rock | — |
| Normalisation low percentile | `P_LOW` | 3.0 | % | `condition.rs:915`; house standard, `memory\method_workflow_standards.md` §GR normalization (P3/P97) | T4 |
| Normalisation high percentile | `P_HIGH` | 97.0 | % | `condition.rs:916`; same source. **Recorded with its hazard:** P97 is the exact percentile that makes Geolog's unguarded upper walk reachable (§2.6(c)) — an argument for order statistics, not against P97 | T4/T1 |
| — vendor guidance, Geolog | — | 90–95 | % | `log_normalization.lls:30` — **`NON-ADOPTABLE — cited for verification`** | T1 |
| Normalisation reference pair | `REF_LOW`, `REF_HIGH` | `ABSENT — ships with no default` | curve unit | `condition.rs:974-980` — the run refuses without them. `PRESENT-OK` and locked in by `SB-ENV-052` | T4 |
| GR normalisation reference, low | `GR_LOW_REF` | 20.0 | gAPI | `modules.rs:2631` — a **deliberately generic** placeholder, documented as "NOT a calibration for any particular field" (`:2618-2622`). Also one of `SB-CORE-007`'s four GR-endpoint sites (`SB-ENV-055`) | — |
| GR normalisation reference, high | `GR_HIGH_REF` | 120.0 | gAPI | `modules.rs:2632` — as above | — |
| Filter kernel | — | declared per run; recorded in provenance | — | `SB-ENV-041`. The three incumbents split three ways on kernel normalisation (dossier §2.5) | T1/T2/T3 |
| Filter length | — | expressed as a thickness only; a sample count is prohibited | project depth unit | `SB-ENV-034`. IP documents its own limit three ways — 1–121, 3–121, 2001 (ledger `F-4`) — **`NON-ADOPTABLE`**, and the deeper defect is the unit | T2 |

### 5.4 Formation temperature and the resistivity temperature correction

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Surface temperature (`ftemp_grad`) | `TSURF` | 26.7 | °C | `modules.rs:1021` — `SHIPPED-UNCITED` | — |
| Geothermal gradient (`ftemp_grad`) | `TGRAD` | 0.03 | °C/m | `modules.rs:1022` — `SHIPPED-UNCITED`. On a foot project this yields **223.6 °C at 2,000 m TVD** (`SB-ENV-045`) | — |
| Surface temperature (`precalc`) | `SURF_TEMP` | 77.0 | °F \| °C | `modules.rs:1090` — `SHIPPED-UNCITED`; the module's own doc states these are **one study's feet-based fits** | — |
| Temperature gradient (`precalc`) | `TEMP_GRAD` | 0.026 | °/ft \| m | `modules.rs:1091` — `SHIPPED-UNCITED`. On a metric project this yields **53.9 °C where 119.8 °C belongs** — a 66 °C error (`SB-ENV-045`) | — |
| Bottom-hole temperature | `BHT` | 100.0 | °C | `modules.rs:1023` — `SHIPPED-UNCITED`. **Consumed** on the `OPT_FT = BHT` branch at `:1041`, `:1049`; an earlier draft of this table called it unused, which was a misread of the `else` arm (see `SB-ENV-047`) | — |
| Depth of BHT measurement | `TD_BHT` | 2000.0 | m | `modules.rs:1024` — `SHIPPED-UNCITED`. **Consumed** at `:1042`, `:1046`, `:1049`, with a `td <= 0` guard | — |
| Mudline / water-bottom depth | `zmudline` | `ABSENT — ships with no default` | project depth unit | `SB-ENV-046`. Geolog's `ftemp` treats a non-negative mudline depth as missing — a validity condition, cited as such | T1 |
| Formation-temperature branch | — | `ABSENT — ships with no default`; enumeration `{MEASUREMENT_REFERENCE, WATER_BOTTOM}` | — | Geolog `ftemp.info:27-32` VALIDATION. The **enumeration** is adopted (`SB-ENV-009`, `SB-ENV-046`) precisely because `ftemp.lls:53` routes every other string — including a typo — to the mudline branch | T1 |
| Resistivity temperature constant | `c` | **6.77 (°F form) ≡ 21.5 (°C form)** | °F, °C | Techlog `Exxon` branch and Geolog's shipped constant — **two independent implementations of the same law in two unit systems** (dossier §2.1). **ADOPTED.** The two correspond to within 0.04 °C (6.77 °F ≡ 21.54 °C), negligible against the divergence below. One named constant, one unit system, conversion derived (`SB-ENV-048`) | T1 |
| — rejected branch | `c` | −6 (°F form) | °F | Techlog `TempCorr_Resistivity.py`, `Arps`-labelled — **`NON-ADOPTABLE — REJECTED`**. Stands alone against two agreeing implementations, and is reached as the **fall-through** for any non-`'Exxon'` string (`:80-83`). Worth **−13.8 % on `Rw` / −7.2 % on `Sw` at 60 °F** against `tref` 200 °F (§2.3). Refusal `RF-4`, §7.3 | T1 |

### 5.5 Framing, depth control and results QC

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| Depth shift | `SHIFT` | 0.0 | project depth unit | `modules.rs:2512` declares `"m"` and `:2506` says "metres", while `:2563` applies it in the depth column's own unit — the declaration is wrong on a foot project (`SB-ENV-057`) | — |
| Blocking interval | `INTERVAL` | `ABSENT — ships with no default` | project depth unit | `frame.rs:242` (`param_open`) | — |
| Minimum bed thickness | `MIN_BED` | `ABSENT — ships with no default` | project depth unit | `frame.rs:243` (`param_open`) | — |
| Bed-detection sensitivity | `SENS` | 2.0 | noise units | `frame.rs:245`, `:378` — `SHIPPED-UNCITED` | — |
| Blocking statistic | `OPT_STAT` | `MEAN` (default), with `GEOMETRIC` and `HARMONIC` offered | — | `frame.rs:19-29`, `:257`. The default is **deliberately not right for permeability** and the doc says so: 1000 mD and 0.01 mD in equal parts give 500 / 0.3 / 0.02 mD, and "the arithmetic answer is the ONE of the three that always reads highest — so the error never looks like a problem" | T4 |
| Reframe method | — | per curve; `Auto` classifies by inspecting values, not names | — | `reframe.rs:36-41`. Downsampling averages, upsampling interpolates, and an output sample with no input inside it is MISSING — never the nearest value | T4 |
| Sw-model divergence threshold | `DEFAULT_DIVERGENCE` | 0.10 | Sw units (v/v) | `resultsqc.rs:23` — `SHIPPED-UNCITED`. User-overridable at `:396` | — |

**Count — counted from the table, not estimated.** **83 rows**: 67 SandiBumi parameters and declared
contracts, plus 16 `NON-ADOPTABLE` vendor or house reference values that exist only to make a
divergence checkable. Of the 67:

- **32 are specified `ABSENT — ships with no default`** — of which **10 currently ship an uncited
  numeric value** that this specification removes rather than sources;
- **19 more currently ship `SHIPPED-UNCITED`** and are specified to *acquire* a source rather than be
  emptied — they are quantities with a real answer that simply has not been written down;
- **16 are enumerations, declared-unit contracts, or values that already carry a citation.**

The **29** `SHIPPED-UNCITED` rows (10 + 19) are the concrete, countable work that `SB-ENV-014` and
`SB-ENV-004` name. *(The front-matter figure of 71 and an intermediate figure of 79 were both
pre-count estimates written before the table was complete. The table is the authority; the front
matter is corrected to 83. Both stale figures are recorded here rather than silently overwritten,
because a count that moved twice is itself evidence about how the section was built.)*

---

## 6. Acceptance tests

68 tests, `SB-ENV-T01` … `SB-ENV-T68`. Tests whose expected value is a snapshot of current behaviour
rather than a sourced quantity are labelled **`CHARACTERIZATION`**. Three tests
(`T06`, `T07`, `T55`) are **build gates** — they enumerate the codebase rather than run a module, and
they fail the build rather than a suite.

### 6.1 Validity conditions and preconditions

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **T01** | A `ModuleSpec` declaring an enumeration validity condition on a string parameter | serialize, deserialize, round-trip through `params_json` | the condition survives the round trip and is present on the deserialized spec; exact equality | `SB-ENV-001`; `ArgSpec` is `Serialize` (`modules.rs:35`) and `choices` already round-trips (`:43-45`) |
| **T02** | A run whose parameter violates a declared range | dispatch through the runner | `run_module` is **never entered**; the failure is raised before it | `SB-ENV-002`, §2.9 — Geolog's manifest rejects before the `.lls` executes |
| **T03** | Branch selector set to a string not in the enumeration | run any module with an enumerated branch | labelled refusal naming the parameter, the value and the permitted set; **not** a fall-through to any branch | `SB-ENV-009`; `TempCorr_Resistivity.py:80-83`, `ftemp.lls:53` |
| **T04** | The same violating run launched four ways: dialog, saved chain, batch multi-well, zone override | run each | all four produce the identical refusal; no path is unvalidated | `SB-ENV-002` — §2.9 consequence 3, "validation lives with the algorithm" |
| **T05** | A violated precondition | inspect the refusal payload | contains the condition name, the offending value, the expected range **and the source string** of that range | `SB-ENV-003`, `SB-ENV-004` |
| **T06** | The whole module registry | enumerate every `ArgSpec` in this domain | **build gate** — every parameter carries either a non-empty source or the explicit `ABSENT` token; zero exceptions | `SB-ENV-004`; `SB-CORE-004` names this a build gate |
| **T07** | The 31 parameters specified `ABSENT` in §5 | inspect each `ArgSpec` | **build gate** — each is `param_open`-shaped, ships no numeric default, and the run refuses without a value | `SB-ENV-016`, `SB-ENV-024`, `SB-ENV-025`, `SB-ENV-052`, `SB-ENV-056` |
| **T14** | A module with a declared validity condition and a well missing one of its inputs | open the dialog | the condition and its source are displayed with the field, and the un-evaluable condition is marked **before** the run is launched | `SB-ENV-008` |
| **T15** | A branch selector set to an unrecognised value, run on a frame following a valid frame | run | no computed quantity retains its previous-frame value; the result is a refusal, not a stale constant | `SB-ENV-009`; `unc_gr.lls:116-153` — the frame-dependent failure |
| **T38** | The whole codebase | enumerate every flag-emitting site | **compile-time**: exactly one polarity definition exists; a second is a type error. Every flag curve carries a declared flag type | `SB-ENV-030`; ledger `F-2` — IP ships three polarities in one family |
| **T39** | An exclusion mask and a diagnostic indicator curve | read both | the two are distinguishable by declared flag type without inspecting values | `SB-ENV-030` |

### 6.2 Environmental corrections

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **T08** | A correction run on a well with a partial input set | run, then read the output's manifest | the manifest lists every step in the chain with one of {applied, unavailable, user-disabled, refused}, and the parameter values used for each applied step | `SB-ENV-005`, `SB-ENV-010`, `SB-ENV-011` |
| **T09** | A correction applying steps *S*, and its uncertainty | compare step sets | the uncertainty's declared step set equals *S* exactly; a mismatched pair is **not emitted** | `SB-ENV-019`; §2.2 — Geolog's `unc_tnph` covers 3 of the 10 `evs_tnph` applies |
| **T10** | A completed correction run | restart the application, read the curve | the applied-step manifest is retrievable without re-running | `SB-ENV-005` |
| **T11** | A well with GR and **no caliper** | run the GR borehole correction | either a refusal, or `GR_EC` with **every sample flagged** as uncorrected and the step marked unavailable in the manifest. **Never** an unflagged copy of `GR` | `SB-ENV-006`, `SB-ENV-007`; the current behaviour at `modules.rs:1838` is the defect this pins |
| **T12** | Every module emitting a `*_EC` output | run each with a missing correction input | no `*_EC` output is byte-identical to its input without a flag saying so | `SB-ENV-006`; `SB-CORE-002` |
| **T13** | A well whose caliper exists over two-thirds of its length | run a correction | the per-sample flag channel distinguishes corrected, partly corrected and uncorrected intervals at the sample, not the run | `SB-ENV-007` |
| **T16** | A GR correction run with mud weight and tool position supplied, then with each withheld | run | each term's presence changes the answer, and each withheld term is reported unavailable | `SB-ENV-010`; Geolog `unc_gr.lls:116-157` branches on both |
| **T17** | A neutron correction with all ten steps available, then with each withheld in turn | run eleven times | each step is individually switchable; each withheld step is reported unavailable; the manifest differs in exactly one entry per run | `SB-ENV-011`; §2.2 — `evs_tnph` ships all ten |
| **T18** | A limestone-scale neutron curve paired with a sandstone `RHO_MA` | run `condflag` and the neutron correction | both refuse or flag; neither computes a crossover flag from the mismatched pair | `SB-ENV-012`, `SB-ENV-029`; `unc_tnph.lls:204-224` is the pattern |
| **T19** | Clean water sand, limestone-unit neutron against sandstone `RHO_MA` | compute the apparent offset | ≈ **0.04 v/v low**, i.e. the same magnitude as the `XOVER_MIN` default of 0.04 (±0.01) — so an unchecked mismatch suppresses crossover to its own detection threshold. **`CHARACTERIZATION`** against `modules.rs:1261-1264`, which states the figure without citing a source | `SB-ENV-012` |
| **T20** | A density correction run with and without a mudcake term | run | the mudcake term changes the answer and is reported in the manifest; `HD_REF` is read from tool/bit inputs, not from a constant | `SB-ENV-013` |
| **T21** | `K_TEMP` = 0.0001 v/v/°C; `ΔT` = 86 °C | compute the neutron temperature correction, and compare against the house gate's slope | shipped: 0.0001 × 86 = **0.0086 v/v (0.9 p.u.)**. House gate: −0.048 p.u./°F × 1.8 = 0.0864 p.u./°C = 0.000864 v/v/°C → 0.000864 × 86 = **0.0743 v/v (7.4 p.u.)**. **Ratio 8.64×; difference ≈ 6.5 p.u.** The test asserts the *ratio*, not a replacement value | `memory\reference_log_qc_gates.md` §Neutron (T4). A **verification** test for `SB-ENV-014`; it does **not** adopt 0.000864 |
| **T22** | A synthetic monotone two-axis table and a query strictly inside its span | chart lookup | interpolated per the declared rule, to the interpolant's stated tolerance; sample flagged `interpolated` | `SB-ENV-015` |
| **T23** | The same table, query **outside** the span, once per declared out-of-span policy | chart lookup | under `clamp`: value equals the boundary node exactly and the sample is flagged `clamped`. Under `refuse`: labelled refusal. **No extrapolation in either case** | `SB-ENV-015`; §2.1 — the difference between Geolog and a naive port is entirely off-chart behaviour |
| **T24** | No chart data present anywhere in the build | run the whole chart-interface suite against synthetic tables | the suite passes; the interface is shippable and verifiable with zero vendor data | `SB-ENV-015`; §7.3 `RF-2` |
| **T25** | A correction step requiring salinity, standoff, mudcake or bit size | run without supplying it | labelled refusal; **no substitution** | `SB-ENV-016`; §2.15 — three vendor values across five orders of magnitude |
| **T26** | A correction chain with a named chart baseline consumed by three later steps | run both branches (back-out enabled and disabled) | the named baseline is assigned exactly once per branch; a step needing a differently-referenced value requests it under a **different name**; the two branches carry consistent, declared baselines | `SB-ENV-017`; dossier `O-14`, `unc_tnph.lls:238-267` |
| **T29** | An uncertainty output | read its metadata | it declares the step set it covers | `SB-ENV-019` |
| **T30** | A corrected curve with four applied steps | open the correction-chain QC view | it reports the uncorrected curve, the corrected curve, and each step's contribution in the curve's own unit, plus the unavailable steps and why | `SB-ENV-020` |

### 6.3 Flags, mask and bad hole

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **T31** | A well with DRHO excursions of both signs and caliper enlargement | run bad-hole detection | the reason channel distinguishes caliper-only, DRHO-only and both; and where DRHO fired, the **sign** is preserved | `SB-ENV-022`, `SB-ENV-023` |
| **T32** | A well with caliper and bit size but **no DRHO** | run | the caliper term is evaluated, the flag is produced, and the DRHO term is reported unavailable. The flag is **MISSING**, never `0`, where neither term can be evaluated | `SB-ENV-021`; `badhole.lls:88-101` (T1); the 3-of-362 DRHO availability case (T3) |
| **T33** | Bad-hole detection with no thresholds supplied | run | refusal. With a named preset selected, the run proceeds and the **preset name and its source** are recorded in provenance | `SB-ENV-024`; the 0.02–0.15 g/cc spread across seven studies |
| **T34** | A well with a caliper and **no bit size** | run | the caliper term is reported unavailable; no default bit size is substituted; the DRHO term is used alone if present | `SB-ENV-025`; the `BS_DEF` = 8.5 in silent slim-hole failure |
| **T35** | A `DRHO` curve declared in kg/m³ and a threshold entered in g/cc | run | refusal naming the unit mismatch. The reciprocal case (`0.1` into a kg/m³ field) also refuses rather than flagging every sample | `SB-ENV-026`; `badhole.info` vs `unc_ldt.lls:59,98` — the same name, two units |
| **T36** | A synthetic well with a washout, a truth density, and `log_predict` in `MAX_RAW` mode, run **with** the bad-hole mask | run | `RHOB_SYN` at the washout is **finite**, exceeds the raw `RHOB` by > 0.2 g/cc, and is within **0.1 g/cc** of the truth — i.e. matches the unmasked control the existing test already asserts | `SB-ENV-027`. The unmasked control and its tolerances are the existing assertions at `workflow.rs:3208-3220`; this test flips the masked case from "pinned defect" to "required behaviour" |
| **T37** | The same run | instrument both mask passes | the exempt module's **inputs** are not blanked *and* its **outputs** are not blanked; every sample it produced at a masked depth is marked | `SB-ENV-027`; `workflow.rs:3132-3137` — "there are TWO blanks, not one" |
| **T27** | A chain running normalisation before despiking | validate against the ordering contract | a warning naming the specific violation and the invalidated step; the chain's order is recorded with the curve | `SB-ENV-018`, `SB-ENV-028`, `SB-ENV-036` |
| **T28** | Any completed run | read the curve's provenance | it records the applied mask (or that none was applied) and the step's position in the chain | `SB-ENV-028` |

### 6.4 Conditioning

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **T40** | METHOD = HAMPEL on a window whose clean population is one repeated value (`MAD = 0`, the fallback branch); `k` = 3.0, then 2.0, then 1.5 | read the displayed contamination ceiling | **33.33 %**, **50.00 %**, **50.00 %** (± 0.01 pp), from `f* = min(1/k, ½)`. The last two are equal because the median's own 50 % breakdown binds below `k = 2` — a display that reads 66.67 % at `k = 1.5` has implemented `1/k` without the wall and FAILS | `SB-ENV-031`; derivation in §2.5 |
| **T69** | The same window with clean scatter admitted (`MAD > 0`, the true-MAD branch), `k` = 3.0 | read the displayed ceiling | **50.00 %**, not 33.33 % — the displayed number follows the branch the run will take, not the parameter alone | `SB-ENV-031`; §2.5 |
| **T70** | A `mean ± kσ` estimator, if one is ever offered, at `k` = 2 | read the displayed ceiling | **20.00 %** from `f* = 1/(k²+1)`. A build that shows 20 % for a HAMPEL run at `k = 2`, or 50 % for a `mean ± kσ` run, has hardcoded one formula across methods and FAILS `SB-ENV-031` | `SB-ENV-031`; §2.5 |

*(`T69` and `T70` are placed here beside `T40` rather than at the end of the table because the three
are one test of one display and are useless read apart. They carry sequential IDs, not `T40b`/`T40c`
suffixes, for the reason §8.0 gives about the dossier's unnumbered `T-1b`: an ID a `T[0-9]+` scan
cannot find is an ID that goes uncounted. Position is a convenience; the number is the contract.)*
| **T41** | The whole codebase | grep for the MAD consistency constant | exactly one named definition; **zero** numeric literals of 1.4826 or 1.482602 outside it | `SB-ENV-032` |
| **T42** | A window in which > 50 % of samples share one value (zero MAD), and a window with 4 samples | run the despiker | both are reported per sample in the flag channel as having used the declared fallback; neither silently substitutes | `SB-ENV-033`; `condition.rs:154-176` |
| **T43** | The same 1.0 m window on the same curve resampled at 0.1 m and at 0.5 m | run the despiker on both | the window covers 1.0 m of rock in both runs; results agree to within the coarser sampling's resolution. A sample-count window would cover 5× different rock | `SB-ENV-034`, `SB-ENV-057`; `condition.rs:15-20` |
| **T44** | A curve with a MISSING interval | run every smoothing and filtering operation | every input-MISSING sample is output-MISSING. Only Fill Gaps may write across one | `SB-ENV-035`; inverts Geolog's `PRESERVE_MISSING = FALSE` |
| **T45** | A despike, a cull, a clip and a gap fill on the same curve; and an interactive shift | apply, then restore from the emitted record | the restored curve is **bit-identical** to the original; the interactive edit's persistent provenance record survives a restart | `SB-ENV-037`, `SB-ENV-042`; `curve_edit.rs:10-12` is the existing pattern |
| **T46** | Gaps of exactly `MAX_GAP`, `MAX_GAP − ε`, `MAX_GAP + ε`, and a gap open at the top of the curve | run Fill Gaps | the boundary case matches the module's own stated comparison; the open-ended gap is **never** filled; the gap is measured between live samples; every written sample is flagged | `SB-ENV-038`; ledger `F-3`; `condition.rs:718-726` |
| **T47** | Clip with no bounds; Clip with `MIN` > `MAX` | run each | refusal in both cases. The reversed pair is **not** silently swapped | `SB-ENV-039`; `condition.rs:591-634` |
| **T48** | A conditioning run whose output name is set to `GR` | resolve output names | refusal **by name, with the reason**, before the module runs | `SB-ENV-040`; `condition.rs:28-37`, `workflow::resolve_output_names` |
| **T49** | A smoothed curve | read its provenance | the kernel, its normalisation, its end behaviour and its gap-edge behaviour are all recorded | `SB-ENV-041` |

### 6.5 Formation temperature, normalisation and units

| ID | Input | Operation | Expected (tolerance) | Source |
|---|---|---|---|---|
| **T50** | The module registry | enumerate producers of the formation-temperature mnemonic | exactly **one** implementation; any second module delegates to it and emits no independent arithmetic | `SB-ENV-043`; currently two (`modules.rs:1055`, `:1170`) |
| **T51** | A vertical well, 2,000 m TVD, each module on its own shipped defaults | run both | they currently differ by **33.1 °C**: `ftemp_grad` = 26.7 + 0.03 × 2000 = **86.7 °C**; `precalc` = 77 + 0.026 × 6561.7 = 247.6 °F = **119.8 °C**. After `SB-ENV-043`/`-044` the two paths MUST agree to **± 0.1 °C** and MUST both run on TVD | `SB-ENV-043`, `SB-ENV-044`; arithmetic shown, derived from the shipped defaults at `modules.rs:1021-1022, 1090-1091` |
| **T52** | Gradient 3 °C/100 m entered on a metric project, then the same well as a foot project | run | the gradient is accepted only with a declared compound unit; the foot project requires **0.016459 °F/ft** (= 0.03 × 1.8 × 0.3048) and refuses a bare `3`; the two projects produce the same temperature curve to **± 0.1 °C** | `SB-ENV-045`; §2.7 |
| **T53** | `TGRAD` in °C/m applied to a foot-unit depth column | run | refusal. The current behaviour — 26.7 + 0.03 × 6561.7 = **223.6 °C** — MUST NOT be reachable | `SB-ENV-045`; `SB-CORE-001`, R14 |
| **T54** | An offshore well with a mudline depth | run the water-bottom branch, and run it with a mistyped branch name | the branch is referenced to the mudline; the mistyped name **refuses** rather than selecting mudline | `SB-ENV-046`, `SB-ENV-009`; `ftemp.lls:53` + `ftemp.info:27-32` |
| **T55** | Every `ModuleSpec` in this domain | cross-reference declared parameters against parameters read in the module body, **across every branch** the module can take | **build gate** — every declared parameter is consumed on at least one reachable branch. Passes today. The gate MUST NOT be written as a single-path scan: `BHT`/`TD_BHT` are consumed only under `OPT_FT = BHT` (`modules.rs:1041-1049`), and a scanner reading only the `else` arm at `:1051` reports them unused — which is exactly the misread this requirement's first draft made | `SB-ENV-047` |
| **T56** | `Rw` = 0.1 ohm·m at 75 °F, corrected to 200 °F, computed once in each unit system | run | the °F form (`c` = 6.77) and the °C form (`c` = 21.5) agree to **± 0.05 %**; the constants correspond to within **0.04 °C** (6.77 °F ≡ 21.54 °C) | `SB-ENV-048`; two independent vendor implementations agree (T1) |
| **T57** | The resistivity temperature correction | enumerate its constants and its selection paths | exactly one named, cited constant. `Rw₁` = 0.1 at 60 °F, `tref` = 200 °F gives **0.03229** ohm·m (±1e-5); the `Arps`-labelled 0.02784 (−13.8 %, −7.2 % on `Sw`) is **unreachable by any path**, including an unrecognised method string | `SB-ENV-048`, `SB-ENV-009`; §2.3, refusal `RF-4` |
| **T58** | A saved chain containing a superseded module id | run it | it resolves, delegates to the survivor, produces the identical answer, and is hidden from the pickers | `SB-ENV-049`; `modules.rs:2639-2673` |
| **T59** | A named-zone override on a depth-trend parameter, and on a compartment parameter | apply each | the trend parameter **refuses** the override; the compartment parameter accepts it. The refusal message states the physical reason | `SB-ENV-050`; `modules.rs:66-87` — the 10.5 °C step |
| **T60** | A GR distribution over a 180 gAPI range with a dense low tail | compute P3 | the exact order statistic. A bin-mean implementation at `BINS` = 50 differs by up to ±½ bin = **1.8 gAPI**, and a dropped bottom bin biases it **high** by ≥ one bin width (3.6 gAPI) — which at span 80 gAPI is **4.5 % `Vsh` absolute**, from `∂IGR/∂GR_clean = (GR − GR_shale)/span² = −0.0125` per gAPI | `SB-ENV-051`; §2.6, arithmetic shown |
| **T61** | Normalize with no reference pair | run | refusal | `SB-ENV-052`; `condition.rs:974-980` |
| **T62** | A multi-well normalisation | run, then inspect | per well: the reference pair, the computed well percentiles, the resulting map and the interval used are all recorded and reviewable before acceptance; a per-well manual override is recorded **as an override** | `SB-ENV-053`, `SB-ENV-054` |
| **T63** | A multi-well set normalised over different depth intervals per well | run | a warning naming the wells whose intervals are not comparable | `SB-ENV-054`; `modules.rs:2615-2616` states the requirement in prose today |
| **T64** | A project whose `Vsh` clean endpoint is changed | re-run | **no normalised curve changes.** The two parameters are separately named and separately sourced | `SB-ENV-055`; `SB-CORE-007`'s 22.2 % `Vsh` spread at GR 70 gAPI |
| **T65** | Log QC with no limits supplied | run | refusal; no numeric defaults exist to fall back on | `SB-ENV-056`; IP25-F §5.9 |
| **T66** | Extreme band GR 117–256 with user band GR 59–168 | enter | refusal at entry: the extreme band must bracket the user band. **`CHARACTERIZATION`** of the vendor configuration only — the numbers are `NON-ADOPTABLE` and are used solely as the inverted case | `SB-ENV-056`; ledger `F-1` |
| **T67** | Every parameter expressing a length in the project's depth unit | enumerate declarations | exactly one token in use. `modules.rs:2512` (`"m"`) and `:1294-1295` (`"m|ft"`) currently fail it; `condition.rs:252` and `frame.rs:242-243` (`"depth"`) pass | `SB-ENV-057` |
| **T68** | A borehole image with a known synthetic speed anomaly | run speed correction | the applied displacement is emitted as a curve and the correction is reversible from it. **Contract test only** — the numeric expectations follow the primary sources named in `ESC-7` and are not specified until those are held | `SB-ENV-058`; `CONTRACT.md` §2.2 |

---

## 7. Open items, escalations and refusals

### 7.1 Open — needed, not yet answerable

**OI-1 — The canonical order of the ten neutron correction steps.** `SB-ENV-011` requires all ten;
their order changes the answer, because each step's chart is referenced to the state left by the
previous one. Geolog's order is readable in `evs_tnph` but §2.6.3 shows its baseline handling is
itself ambiguous. *Settled by:* deciding `OI-1` jointly with `ESC-5`, then fixing the order in the
declarative chain contract of `SB-ENV-018`.

**OI-2 — Whether the chart interface must support three or more axes.** `SB-ENV-015` is specified
for a two-axis table. Several corrections in evidence are functions of three or four variables. A
two-axis interface that later needs a third is a breaking change to every stored chart.
*Settled by:* enumerating the axis count of the correction set SandiBumi intends to support before
`SB-ENV-015` is implemented, not after.

**OI-3 — The form of the per-tool uncertainty model.** `SB-ENV-019` requires an uncertainty computed
over the applied steps and says nothing about how each step's contribution is estimated. Geolog's
`unc_*` family is the only implementation in evidence and is itself the subject of §2.2's defect.
*Settled by:* a decision on whether contributions combine in quadrature or by an explicit covariance
statement — which is a modelling choice, not a reading of a vendor.

**OI-4 — Where the applied-step manifest is persisted.** `SB-ENV-005` requires retrieval without
re-running. The existing candidates are the log-set archive, the run record, and a per-curve metadata
table. *Settled by:* the same decision that resolves `SB-ENV-028`'s mask record and `SB-ENV-042`'s
interactive-edit provenance — all three are the same storage question and should be answered once.

**OI-5 — How a module declares a repair exemption.** `SB-ENV-027` requires the exemption; whether it
is a `ModuleSpec` flag, a category, or a per-output declaration is open. A per-output declaration is
the most precise — `log_predict`'s `MAX_RAW` output is a repair and its `SYNTHETIC` output arguably is
not — and it is also the most work. *Settled by:* enumerating which shipped outputs are repairs.

**OI-6 — What happens between the user band and the extreme band.** `SB-ENV-056` fixes the
precedence relation (extreme brackets user) but not the semantics of a sample in the gap: flagged,
warned, or excluded. IP's own two panels contradict each other on this (ledger `F-1`), so there is no
incumbent answer to adopt. *Settled by:* Jauhar's own QC practice, which the house gates document
qualitatively but not as a state machine.

**OI-7 — Whether the bad-hole reason channel is one encoded curve or several boolean curves.** One
curve is compact and needs a decode table; several are self-describing and multiply the curve count
by four. *Settled by:* the same choice made for the correction flag channel of `SB-ENV-007`, which
has identical structure — they should not be decided separately.

**OI-8 — Whether Hodges-Lehmann joins the smoothing set.** Dossier §2.15 records the pairwise-average
estimator as an incumbent capability; SandiBumi has no equivalent and no requirement above claims
one. *Settled by:* whether a robust location estimator is wanted for curve averaging as distinct from
the robust *scale* estimator already used in despiking.

### 7.2 Escalations — need Jauhar or a source not on this machine

**ESC-1 — Which bad-hole presets ship, and under what names?** `SB-ENV-024` ships the thresholds
`ABSENT` with cited presets. The competing values are **0.02 g/cc** (one clastic study),
**0.15 g/cc** (two carbonate studies and the ITB module gate) and **0.1 g/cc** (IP's shipped value,
`NON-ADOPTABLE`). *Consequence in real units:* on a well with moderate rugosity, 0.02 versus 0.15
g/cc is the difference between flagging most of the section and flagging almost none of it; the
masked footage propagates into net pay through every masked module run. **Exact question:** which
presets ship, what are they named so the name does not imply universality, and does the caliper
preset ship at 2 in given that five studies agree on it and none disagrees?

**ESC-2 — What source does the neutron temperature coefficient come from?** `SB-ENV-014` requires
`K_TEMP` to be cited or `ABSENT`. The only quantified source held is the house gate's −0.048 p.u./°F,
which is a linearisation over a 50 °F span and not a tool-specific coefficient; the real correction is
a non-linear chart that this chapter refuses to transcribe (`TR-1`, `TR-2`). *Consequence:* ~6.5 p.u.
at ΔT = 86 °C against carbonate `PHIE` means of 0.05–0.07 v/v — larger than the porosity being
measured. **Exact question:** is the acceptable v1 behaviour (a) `ABSENT` with the house gate offered
as a named preset, (b) `ABSENT` with no preset, or (c) acquire a primary neutron-response publication
and derive the coefficient independently? This is an **acquisition gap** under option (c): the
specific missing source is a published neutron tool-response paper giving the temperature dependence
of thermal-neutron porosity — not a service-company chartbook.

**ESC-3 — Which of IP's two QC-limit panels is authoritative?** Ledger `F-1`, unresolved: the
extreme-low GR (117) exceeds the user-min GR (59), and the ledger's own leaning is that the extreme
table is the likelier culprit. *Consequence:* none for SandiBumi's shipped values, which are `ABSENT`
either way — but it determines whether `SB-ENV-056`'s bracketing rule can cite IP as precedent or
must stand on its own reasoning. **Settled by:** dossier escalation `E-4`, a live IP 2025 session.

**ESC-4 — What constant does Techlog's compiled production engine use?** Dossier `E-12`. The Toolbox
script's three branches are readable; the production environmental-correction engine's resistivity
temperature constant is compiled and unknown. *Consequence:* it decides whether §2.3's finding is
"one vendor ships an outlier in a script" or "one vendor ships an outlier in production", which is a
materially different competitive claim and must not be overstated in any client-facing form.
**Settled by:** a Techlog session, or vendor documentation not held.

**ESC-5 — Is `unc_tnph.lls:246`'s baseline reassignment deliberate or a defect?** Dossier `O-14`.
Both readings survive a source-only reading: line 246 may be discarding the caliper-referenced
baseline by mistake, or deliberately switching to a nominal-hole reference because the mudcake and
standoff charts are nominal-referenced. *Consequence:* the difference is exactly the actual-hole
versus bit-size neutron response — zero in gauge hole, growing with washout, i.e. largest where the
correction matters. **Exact question:** which baseline do the downstream steps require? This is
answerable from a neutron tool-response primary source, not from more reading of the vendor.

**ESC-6 — Tier-C classification conflict on entropy image speed correction.** `CONTRACT.md` §2.2
classifies it **C-3 (opaque artifact)**. SandiBumi's own Tier-C register describes the same item as
"**stated patented** (algorithm only)" (`docs/research_2026-07/ip_ingest/D_tierC_register.md:43`),
which is **C-1** terms — under which re-derivation does not clear a granted method claim and a
design-around must be checked against the granted claims before any requirement is specified.
*Consequence:* under C-3, `SB-ENV-058` may be built from the literature as written. Under C-1, the
patent claims must be read first, and Jauhar's standing C-1 decision is "read the claims, license, or
drop." **Exact question:** which classification governs? This chapter has specified `SB-ENV-058` to
avoid the entropy formulation entirely, which satisfies both readings — but the classification still
needs settling because it determines whether the accelerometer path may proceed unblocked.

**ESC-7 — Primary sources for speed correction are not on this machine.** An **acquisition gap**, not
a refusal, per `CONTRACT.md` §2.2. `SB-ENV-058` names two open paths and specifies no equation and no
coefficient, because none is held. **Specific missing sources:** (a) a published treatment of
accelerometer-integration depth correction for wireline logging tools, giving the integration scheme
and its drift-control method; (b) a published treatment of inter-pad or inter-pass image
cross-correlation depth alignment. Neither is a service-company chartbook and both are ordinary
SPWLA/SPE literature. Until they are held, `SB-ENV-058` carries a contract test only.

**ESC-8 — The normalisation binning defect is a code read, not a runtime observation.** Dossier
`E-6`. §2.6 is the highest-stakes finding in this domain and it has not been confirmed against a
Geolog run. *Consequence:* it must not be asserted to a client until confirmed, and the confirmation
run should be sized to probe the **two unguarded array limits** as well as the percentile behaviour —
`log_val[99999]` driven by accepted-sample count, and `BINS` against the 999-element arrays.

**ESC-9 — Which σ convention does IP's despike use?** Dossier `O-15`. *Consequence:* the masking
threshold is 20.00 % under population σ and 19.19 % under sample σ at `k = 2, N = 20`. The direction
is settled and neither rescues the estimator, so this does not change any requirement — it changes
what `SB-ENV-031`'s live display must compute for a non-robust estimator, if one is ever offered.

**ESC-10 — A candidate `SB-CORE` requirement, not minted here.** `SB-ENV-047` — *a module MUST NOT
declare a parameter its computation does not consume* — is not domain-specific. It is a general
consequence of `ArgSpec` driving the auto-generated dialog: the declaration **is** the promise to the
user, and a discarded parameter is a false promise the user cannot detect. It applies to every
chapter with a module registry. **This chapter does not mint a `SB-CORE` id.** Raised for Jauhar's
decision on whether it becomes one or stays a per-domain requirement.

**Amended 2026-08-07.** The instance that motivated this escalation was a misread — `BHT`/`TD_BHT`
*are* consumed, on the branch the first draft did not follow (see `SB-ENV-047`). **There are zero
known live violations in this domain.** That weakens the case for minting an `SB-CORE` id now: the
rule is preventive, and a core requirement with no instance anywhere is a gate looking for a defect.
The **test** is still worth having — `SB-ENV-T55` costs one build gate and would have caught the
misread itself — so the recommendation is now: keep `SB-ENV-047` as a per-domain requirement with
its build gate, and mint an `SB-CORE` id only when a second chapter finds a real instance.

*(Considered and not raised: the "validation lives with the algorithm, not the manifest or the
dialog" rule of §2.9. It is fully within `SB-CORE-003`'s existing wording — "the runner MUST evaluate
it before computing" — and needs no new id, only the explicit statement made in `SB-ENV-002`.)*

**ESC-11 — Acquire Theys, *Log Data Acquisition and Quality Control*, Editions Technip (1991).**
**The highest-value single acquisition for this domain, and an acquisition gap rather than an open
item.** All 21 of Geolog's `unc_*` modules cite it, and `unc_ldt.lls:21-22` attributes its error-
propagation form to it specifically. It is the **independent source that lets SandiBumi derive the
uncertainty term decomposition of `SB-ENV-019` from published work instead of from Geolog's fitted
coefficients** — which is precisely the `CONTRACT.md` §2.2 required path rather than the prohibited
one. Not on the ITB shelf per `memory\reference_itb_team_library.md`. Without it, `SB-ENV-019` is
specified as a contract (step set, declaration, monotonicity) with no term decomposition, which is
what §7.1 `OI-3` records.

**ESC-12 — Geolog's chart-function interpolation contract is undocumented and is needed to specify
`SB-ENV-015` faithfully.** The mode strings and their argument orders are inferred from call sites;
the interpolation algorithm — linear or spline, in which axis — appears in no `.lls`. *Consequence:*
`SB-ENV-015` currently specifies that the rule be **declared**, not what it is, which is the correct
conservative position but leaves the default unspecified. **Settled by:** Geolog function-library
documentation or a controlled run. **The chart files themselves are not to be opened** — dossier
`E-3` states this and this chapter holds the same line (`TR-2`).

**ESC-13 — Licensing ruling: may installed vendor chart functions be used derivatively at all?**
Dossier `E-13`. A vendor product on this machine ships chart-function files for six of the eight
families another vendor re-digitised. The dossier reads none of them and recommends purchasing the
source books anyway, on the ground that no licence permits derivative use of files delivered inside a
third-party product. That is a legal judgement stated by a technical document and it needs
confirming by whoever owns SandiBumi's licensing position before any procurement. *Consequence:* it
decides whether `SB-ENV-015`'s interface can ever be pointed at anything on this machine. **Until it
is answered the operating rule is unchanged: directories are listed, files are never opened.**

**ESC-14 — Where does environmental correction sit in the SandiBumi pipeline?** Dossier `O-6`, `E-7`.
The house 15-step conditioning order places environmental correction **second-to-last**; the
study-level workflow order in delivered work omits it. The two documented house orderings disagree
on whether it appears at all. *Consequence:* `SB-ENV-018`'s ordering contract cannot be populated
without this — the contract mechanism is specifiable now, its content is not. **Exact question:**
which of the two orderings is authoritative, and does environmental correction precede or follow
normalisation?

**ESC-15 — Jauhar's ruling on the GR uncertainty model.** Dossier `E-10`. Adopting a computed GR
uncertainty gives 2–4× tighter values than the hand-set constant already used in delivered multimin
runs, and the vendor's uncertainty model carries **no mud-chemistry term** although the house gate
records KCl and barite muds as GR inflators. *Consequence:* this is a change to delivered
methodology, not an implementation detail — a tighter uncertainty on the same data changes every
downstream confidence statement. The dossier's own resolution is the tractable one and is adopted as
the shape of `SB-ENV-019`: **two vendors correct for mud chemistry in their applying modules and
neither propagates it into an uncertainty, so the term is derived from the correction module's own
declared inputs rather than invented.** **Exact question:** does the computed uncertainty ship
opt-in behind the hand-set default until it is validated against a re-run of a delivered well?

**ESC-16 — The shipped Hampel `K = 3.0` sits above the robustness knee.** Raised by this chapter, not
by a dossier. The despike contamination ceiling is `f* = min(1/k, ½)`: flat at **50 %** for every
`k ≤ 2` and falling above it, so `K = 3.0` buys **33.3 %** where 50 % was free on that axis. The
value is `SHIPPED-UNCITED` and the code comment (`condition.rs:253-255`) says so itself — "the
ordinary three-deviation convention … NOT a field calibration". *Consequence:* lowering `K` toward 2
raises breakdown resistance at the cost of flagging more clean samples, and where that balance
belongs is a judgement about Jauhar's curves, not a result the derivation can settle. **Not changed
here** — a despike cutoff is a petrophysical parameter, and a parameter is cited or asked about,
never adjusted because an analysis made a different number look attractive. **Exact question:** does
`K` stay at 3.0 with the ceiling displayed (`SB-ENV-031`) so the cost is at least visible, or move to
2.0 and take the false positives?

### 7.3 Refusals

Two kinds, listed separately per `CONTRACT.md` §2.2.1.

#### 7.3.1 Transcription refusals — rule compliance

Four. Each names the rule and what was done instead.

**TR-1 — The commented-out neutron correction blocks in `unc_tnph.lls`.** These contain **two-segment
linear fits to a vendor correction chart**. They were read to establish *which steps exist* and *that
the uncertainty module covers fewer steps than its twin* — both capability facts — and **no
coefficient, breakpoint or segment slope from them appears in this chapter**. *Rule:* `CONTRACT.md`
§2.1, vendor chart lookup-table data. *Instead:* `SB-ENV-011` specifies the step **set**;
`SB-ENV-014` requires each coefficient to be cited or `ABSENT`; `ESC-2` names what would have to be
acquired to supply one.

**TR-2 — Vendor correction chart tables, including `.neu` and `.ovl`.** §2.12 records that chart
coverage is uneven, edition-ambiguous and incomplete even inside a commercial product — one vendor
"received the Baker Atlas chart book as a series of algorithms" and could not reconstruct chart
numbers at all (ledger `F-13`), and another cites a Sperry-Sun source as 1998 in one place and 1996
in another (ledger `F-12`). That is **capability intelligence and it is recorded**; **no tabulated
value from any of them is.** *Rule:* `CONTRACT.md` §2.1, which names `.neu`/`.ovl` explicitly.
*Instead:* `SB-ENV-015` specifies the chart **interface** — declared span, interpolation rule,
declared out-of-span policy, clamp flagging, no extrapolation, per-sample provenance — and
`SB-ENV-T24` requires the entire interface suite to pass **against synthetic tables with no chart
data present anywhere in the build.** The enforcement layer, which is the part that actually prevents
the failure, therefore ships and is verified without creating any obligation to acquire or transcribe
chart data.

**TR-3 — The `.itt` / `.itp` image tool-descriptor files.** Not read and not copied. They were
identified only as vendor tool-descriptor taxonomy while establishing the boundary of `SB-ENV-058`.
*Rule:* `CONTRACT.md` §2.1's blanket clause — no vendor file in any form.

**TR-4 — R1 was not extended.** `src/ui/chartOverlays.ts` and `src-tauri/src/neutron_charts.rs`
declare in their own headers that they are digitized from a 2013 vendor chartbook
(`neutron_charts.rs:1-22` names the edition and the charts), and `IP_PROVENANCE.md` §2.1 calls this
"the single most exposed item in the product". This chapter **cites that exposure and specifies
requirements around it**; it adds no digitized chart, proposes no new digitization, reproduces no
tabulated pair from either file, and specifies no requirement whose only possible implementation is a
larger transcription. *What would reduce the exposure, stated without adopting it:* `SB-ENV-015`'s
interface makes the tables **replaceable** — a user-supplied or independently-derived table satisfies
the same interface — which converts R1 from a structural dependency into a swappable asset. That is
`IP_PROVENANCE.md` §2.1's option (c) made cheap, and it is the only R1 mitigation this chapter
enables.

**Discipline note, stated so it is on the record.** `CONTRACT.md` §2.1's single recorded exception
(the Matthews & Kelly rows) was **not reasoned from at any point** in this chapter. No second case
was identified. Had one been, it would appear here as an escalation rather than a decision.

#### 7.3.2 Defect refusals — vendor behaviour SandiBumi declines to reproduce

Ten. These are competitive wins and discharge `03_EVIDENCE_BASE.md` §14.1.

**RF-1 — The `Arps`-labelled resistivity temperature law, and the fall-through that selects it.**
`c = −6` °F stands alone against two agreeing implementations and is reached by any method string
that is not exactly `'Exxon'` (`TempCorr_Resistivity.py:80-83`). *Worth −13.8 % on `Rw` and −7.2 % on
`Sw` at 60 °F.* **SandiBumi does instead:** one named, cited constant expressed in one unit system
with the conversion derived (`SB-ENV-048`), and an unrecognised method name is an error, never a
default branch (`SB-ENV-009`).

**RF-2 — A non-robust mean ± kσ despike estimator.** 0 % breakdown point, masking from 19–20 %
contamination at the shipped `k = 2`, and a ceiling that falls further with every increase in `k`.
**SandiBumi does instead:** Hampel/MAD — 50 % breakdown on the true-MAD branch and `min(1/k, ½)` on
the zero-scatter fallback, already shipped (`condition.rs:154-176`) — which is **33.3 % against 10 %
at the same `k = 3`** — plus the live per-branch contamination ceiling no incumbent displays
(`SB-ENV-031`).

**RF-3 — A histogram-bin percentile.** Geolog's implementation drops its bottom bin, kills its top
bin, has an unguarded upper walk reachable at exactly `PCT_MAX = 97`, and returns a bin mean rather
than a percentile. **SandiBumi does instead:** exact order statistics on sorted values, already
shipped (`condition.rs:991-998`), which is structurally immune to all four.

**RF-4 — Unguarded array access driven by user-settable quantities.** `log_val[99999]` written with
an unchecked index, and `BINS` unchecked against 999-element arrays. **SandiBumi does instead:**
treats a bin count and a frame count as **validated inputs** — exceeding either is an error with a
message, never a silent write past the end (`SB-ENV-001`, `SB-ENV-003`).

**RF-5 — `PRESERVE_MISSING = FALSE`.** A smoothing operation that writes values across a gap.
**SandiBumi does instead:** a MISSING sample stays MISSING through every smoothing, filtering and
averaging operation; only Fill Gaps may write across one, it is bounded by a user-set maximum, and
every sample it writes is flagged (`SB-ENV-035`, `SB-ENV-038`; `condition.rs:22-27`).

**RF-6 — An uncertainty computed over a different step set than the correction it describes.**
Geolog's `unc_tnph` covers three of the ten steps `evs_tnph` applies and nothing in either output
says so. **SandiBumi does instead:** the uncertainty declares its step set, and a mismatched pair is
not emitted at all (`SB-ENV-019`, `SB-ENV-T09`).

**RF-7 — Shipping a default for a measured property of the well.** Formation salinity at `2.8E-4
Kppm` (= 0.28 ppm, physically impossible), standoff at exactly the value the house gate names as
worth ~2 p.u., and a default bit size. **SandiBumi does instead:** every such parameter ships
`ABSENT` and the run refuses (`SB-ENV-016`, `SB-ENV-025`).

**RF-8 — Shipping log-QC limits that would flag most real logs, in a band configuration that inverts
its own semantics.** **SandiBumi does instead:** no numeric defaults, and the extreme band is
required to bracket the user band, refused at entry if it does not (`SB-ENV-056`).

**RF-9 — Three flag polarities inside one workflow family.** **SandiBumi does instead:** one
polarity, defined once, as a type, with a second polarity a compile-time impossibility
(`SB-ENV-030`).

**RF-10 — A window, filter length or gap limit expressed in samples.** IP documents its own limit
three ways and the deeper defect is the unit. **SandiBumi does instead:** every window, gap, bed
thickness and shoulder is a physical thickness in the project's depth unit, resolved against the
curve's own depth column — already shipped (`SB-ENV-034`; `condition.rs:15-20`).

### 7.4 Independent-derivation requirements

One item in this domain.

**Entropy-based borehole-image speed correction — `SB-ENV-058`.**

- **Class:** `CONTRACT.md` §2.2 lists it **C-3 (opaque artifact)**; SandiBumi's own Tier-C register
  states it is patented, which is **C-1** terms. The conflict is escalated as `ESC-6` and is not
  resolved here. `SB-ENV-058` is written to satisfy both readings by avoiding the entropy
  formulation entirely rather than designing around it.
- **User need:** correcting wireline and LWD borehole-image logs for tool-speed variation and
  stick-slip. Stick-slip depth distortion is the dominant artifact in wireline image logs and nothing
  downstream can compensate for it.
- **Primary sources:** **not held on this machine.** The two specific missing sources are named in
  `ESC-7`. Per `CONTRACT.md` §2.2, this is recorded as an **acquisition gap and escalated**, not as a
  refusal, and no equation or coefficient is invented in the interim.
- **The prohibited path, named so it is unambiguous:** the entropy-optimisation formulation MUST NOT
  be used, approximated, renamed or inferred from observed behaviour. SandiBumi's own register draws
  the line in the same place: "the entropy-optimization step is the patented novelty — avoid it
  specifically; the accelerometer/cross-correlation approaches are open"
  (`D_tierC_register.md:47`).
- **`Betters:`** the incumbent's speed correction is embedded in a compiled image engine, so the
  applied displacement is invisible, unauditable and unreproducible from the delivered data. Both
  specified paths **emit the applied displacement as a curve**, making the correction reviewable and
  reversible; and the accelerometer path derives displacement from a **measured** quantity rather
  than from an image-quality objective, so it does not optimise an image toward looking correct.
- **Owning requirement:** `SB-ENV-058` (P3 — image logs are not v1). Recorded now so the capability
  is not later built by the prohibited path for want of a specification.

*(No other Tier-C item falls in this domain. Experienced Eye / EEFS, Domain Transfer Analysis,
Textural Facies `Freq_Tiles`, the shipped neural-network weight files and the frequency-domain
dispersion fits belong to `24_ml-advanced.md`, `19_facies-electrofacies.md` and the sonic/geomechanics
chapters, and are not specified here.)*

---

## 8. Traceability

### 8.0 How this table was counted, and one discrepancy that is not smoothed

Every numbered finding, comparison subsection, disposition, adoption-spec line, shipped test, OPEN
item, ledger entry, escalation, source-register class and critique-disposition row in
`docs/research_2026-08/cross_tool/envcorr-qc.md` gets exactly one row below. The universe is **178
rows**, enumerated by family rather than estimated:

| Dossier family | Rows | Where counted |
|---|---|---|
| §1 method inventory (1.1, 1.2, 1.2.1, 1.3, 1.3.1) | 5 | §8.1 |
| §2 definitions & equations (2.1–2.15 plus 2.6.1, 2.6.2, 2.6.3) | 18 | §8.2 |
| §3 differences that matter (3.1–3.10) | 10 | §8.3 |
| §4 optimal choice per item (4.1–4.14) | 14 | §8.4 |
| §5 adoption spec (5.1, 5.2, 5.3) | 3 | §8.5 |
| §5.4 tests to ship | **39** | §8.6 |
| §5.5 applicable `FINDINGS.md` §6 rules | 13 | §8.7 |
| §6 OPEN items `O-1`…`O-15` | 15 | §8.8 |
| §6 ledger dispositions `F-1`…`F-15` | 15 | §8.9 |
| §6 escalations `E-1`…`E-13` | 13 | §8.10 |
| §7 source register (T1, T2, T3, T4, Tools) | 5 | §8.11 |
| Compliance statement | 1 | §8.11 |
| Critique disposition — blockers `BLK-1`, `BLK-2` | 2 | §8.12 |
| Critique disposition — majors `MAJ-1`…`MAJ-10` | 10 | §8.12 |
| Critique disposition — minors `MIN-1`…`MIN-13` | 13 | §8.12 |
| Critique disposition — "Not applied", "What this revision did NOT do" | 2 | §8.12 |
| **Total** | **178** | |

**The discrepancy, stated rather than smoothed.** The dossier's §5.4 test table is numbered `T-1`
through `T-38`, which reads as 38 tests. It contains **39 rows**. An unnumbered extra row, **`T-1b`
("Kernel normalisation convention")**, sits physically between `T-5` and `T-6` — out of sequence with
its own label as well as absent from the `T-1`…`T-38` run. It was evidently inserted during a
revision pass and given a sub-letter rather than renumbering the table. Two consequences worth
recording:

1. **Any count of the dossier's tests taken from its highest number is wrong by one.** A naive
   `T-[0-9]+` scan also misses it, because `T-1b` matches the `T-1` pattern with no word boundary. My
   own first enumeration in this chapter said 38 and was corrected; the earlier figure is recorded
   here rather than quietly replaced.
2. **The row is substantive, not editorial.** `T-1b` asserts that a kernel declared `PEAK`-normalised
   is divided by `Σw` before use — a real requirement (`SB-ENV-041`), not a duplicate of `T-1`. It
   would have been dropped by a mechanical port of the test list.

This is the same failure class the chapter is about: a correct artifact whose *manifest* — here, its
numbering — under-reports it, so a consumer reading the label rather than the contents inherits a
silently incomplete version.

**The traceability chain runs in two hops, not one.** This table maps *dossier item → requirement*.
The second hop, *requirement → test*, is carried in the Source column of every row of §6, where each
acceptance test names the `SB-ENV-nnn` it discharges. Tests are therefore not duplicated here; a
dossier item's test coverage is read by following its requirement id into §6.

**Disposition vocabulary.** `ADOPTED` → became or shaped a requirement. `DEFERRED` → real, but not
for this chapter or not for now, with a priority and a trigger. `REJECTED` → deliberately not
carried, with the reason. `EVIDENCE-ONLY` → used as evidence in §2 or §3, generating no requirement
of its own. `ESCALATED` → cannot be settled here; routed to §7.1 or §7.2. `REFUSED` → the chapter
declines to reproduce or to read it (§7.3).

### 8.1 §1 — Method inventory (5 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §1.1 IP 2018 / IP 2025 inventory | EVIDENCE-ONLY | §2.10, §2.13, §2.14, §2.15; sizes `SB-ENV-030`, `SB-ENV-056` |
| §1.2 Techlog 2018.2 inventory | EVIDENCE-ONLY | §2.3, §2.12; sizes `SB-ENV-031`, `SB-ENV-015` |
| §1.2.1 Techlog's per-tool correction taxonomy | ADOPTED | `SB-ENV-011` — the step set becomes a declared capability matrix, not a fixed chain; `SB-ENV-005` |
| §1.3 Geolog V14 inventory | EVIDENCE-ONLY | §2.1, §2.11; sizes `SB-ENV-019` |
| §1.3.1 **126 modules, 10 vendor families, discoverable only from the `bin` manifests** | ADOPTED — the chapter's centre of gravity | §2.1 → `SB-ENV-001`, `SB-ENV-002`, `SB-ENV-003`, and the position taken on `SB-CORE-003` / `SB-CORE-004` in §4.1 |

### 8.2 §2 — Definitions, equations and assumptions compared (18 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §2.1 Resistivity temperature correction — name collision | ADOPTED | §2.3 → `SB-ENV-048`, `SB-ENV-009`; the defect itself refused at `RF-1` |
| §2.2 Badhole flagging | ADOPTED | §2.4 → `SB-ENV-021`, `SB-ENV-022`, `SB-ENV-024`, `SB-ENV-025` |
| §2.3 Despike — three genuinely different algorithms under one name | ADOPTED | §2.5 → `SB-ENV-031`, `SB-ENV-032`, `SB-ENV-009` |
| §2.4 Outlier / spurious-population removal | ADOPTED | `SB-ENV-036` |
| §2.5 Filters and smoothing | ADOPTED, partly deferred | `SB-ENV-035`, `SB-ENV-041`, `SB-ENV-033`; the circular branch DEFERRED with `T-10` |
| §2.6 Neutron correction chain — the readable end-to-end chain | ADOPTED | §2.2 → `SB-ENV-011`, `SB-ENV-005`, `SB-ENV-007`, `SB-ENV-019` |
| §2.6.1 The applying module, with the "disabled" steps shipped ON | ADOPTED | §2.2, §2.15 → `SB-ENV-011`, `SB-ENV-016`; the shipped 50 kppm salinity refused at `RF-7` |
| §2.6.2 The IP-vs-Geolog neutron chain, step for step | EVIDENCE-ONLY | §2.2; sizes `SB-ENV-011`'s ten-step set and `OI-1` |
| §2.6.3 Chart-baseline clobber — a probable defect in the *enabled* part | ADOPTED | `SB-ENV-017`; confirmation ESCALATED at `ESC-5` |
| §2.7 Per-tool uncertainty models (no IP or Techlog equivalent) | ADOPTED | §2.11 → `SB-ENV-019`; term decomposition OPEN at `OI-3`, its source ESCALATED at `ESC-11` |
| §2.8 Log normalization | ADOPTED | §2.6 → `SB-ENV-051`…`SB-ENV-055` |
| §2.9 Formation temperature — four unit conventions | ADOPTED | §2.7 → `SB-ENV-043`…`SB-ENV-047` |
| §2.10 Nulls | DEFERRED — P0, owned elsewhere | `21_data-io.md`. Null canonicalisation is a read/write contract; named as a seam in §1.1 and deliberately not duplicated here |
| §2.11 Depth shifting | ADOPTED, partly deferred | `SB-ENV-042`, `SB-ENV-057`; the storage model and the multi-resolution pass DEFERRED to `21_data-io.md` / `23_plotting-interactivity.md` |
| §2.12 Correction-chart architecture — the three-way split | ADOPTED as an **interface only** | §2.12 → `SB-ENV-015`; the chart data itself REFUSED at `TR-2` |
| §2.13 Flag conventions | ADOPTED | §2.13 → `SB-ENV-030`, `SB-ENV-022`; the vendor behaviour refused at `RF-9` |
| §2.14 Gap filling | ADOPTED | §2.14 → `SB-ENV-038` |
| §2.15 "Lateral" pairwise average | REJECTED for this chapter | A curve-averaging operator, not a conditioning one. Recorded as `OI-8` so the boundary call is visible rather than silent; the vendor-doc correction it implies is carried by dossier `E-8` |

### 8.3 §3 — Differences that matter (10 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §3.1 The `Arps`-labelled law disagrees with the classical one — quantified | ADOPTED | §2.3 → `SB-ENV-048`; refused at `RF-1`; which reading is right ESCALATED at `ESC-4` |
| §3.2 DRHO cutoff — a 1000× unit trap, and no house consensus on the value | ADOPTED | §2.4 → `SB-ENV-026`, `SB-ENV-024`; preset naming ESCALATED at `ESC-1` |
| §3.3 Despike mean ± kσ breaks down under exactly its deployment conditions | ADOPTED — **including the UI half** | §2.5 → `SB-ENV-031` (the ceiling `f* = 1/(k²+1)` is shown live, beside the cutoff); refused at `RF-2` |
| §3.4 The normalization histogram silently drops the bottom bin | ADOPTED | §2.6 → `SB-ENV-051`; refused at `RF-3` and `RF-4`; runtime confirmation ESCALATED at `ESC-8` |
| §3.5 Geothermal gradient — four conventions, two multiplicative traps | ADOPTED | §2.7 → `SB-ENV-045` |
| §3.6 No tool enforces a conditioning order, and the house order is load-bearing | ADOPTED — mechanism only | §2.8 → `SB-ENV-018`; **the order's content** ESCALATED at `ESC-14` |
| §3.7 Neutron matrix scale — the same trap in all three tools | ADOPTED | §2.9 → `SB-ENV-012` |
| §3.8 IP's shipped log-QC limits would flag most real logs | ADOPTED | §2.10 → `SB-ENV-056`; refused at `RF-8`; panel authority ESCALATED at `ESC-3` |
| §3.9 Five neutron steps commented out in the *uncertainty* module | ADOPTED | §2.2 → `SB-ENV-019`, `SB-ENV-011`; refused at `RF-6`; the commented blocks themselves REFUSED at `TR-1` |
| §3.10 Computed per-tool uncertainty vs the hand-set constants in delivered work | ESCALATED | `ESC-15`; the resolution's *shape* is adopted into `SB-ENV-019` |

### 8.4 §4 — Optimal choice per item (14 rows)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §4.1 Resistivity temperature → adopt the classical form, reject the Toolbox default | ADOPTED | `SB-ENV-048` (one constant, cited, unit-tagged, surfaced); the rejection is `RF-1` |
| §4.2 Badhole → the rule shape and flag design, no shipped numeric default | ADOPTED | `SB-ENV-021`, `SB-ENV-022`, `SB-ENV-024`, `SB-ENV-025`, `SB-ENV-026` |
| §4.3 Despike → the robust statistic, the guards, the published numbers | ADOPTED | `SB-ENV-031`, `SB-ENV-032`, `SB-ENV-033`, `SB-ENV-037`; the sonic despike taxonomy folded into `SB-ENV-031`'s method enumeration |
| §4.4 Outlier / population culling → adopt the cull module wholesale | ADOPTED | `SB-ENV-036` (distinct operation, with the summary report that makes it non-silent) |
| §4.5 Filtering → kernels, mode set, method set, window typing, padding, circular branch | ADOPTED, partly deferred | `SB-ENV-035` (padding and gap contract), `SB-ENV-041` (kernel declaration), `SB-ENV-034` (window typing); the circular branch DEFERRED P2 with `T-10` |
| §4.6 Normalization → the house method, with the algorithm bug fixed | ADOPTED | `SB-ENV-051`…`SB-ENV-055`; the bug refused at `RF-3` |
| §4.7 Environmental corrections → one vendor's interface, another's storage, a third's taxonomy | ADOPTED | `SB-ENV-005`, `SB-ENV-011`, `SB-ENV-015`, `SB-ENV-016`, `SB-ENV-020`. The acquisition table is EVIDENCE-ONLY; no chart content was read (`TR-2`) |
| §4.8 Formation temperature → both branches | ADOPTED | `SB-ENV-043`, `SB-ENV-044`, `SB-ENV-046`, `SB-ENV-047` |
| §4.9 Depth shifting → storage model, multi-resolution pass, pre-conditioning, pick hygiene | ADOPTED, partly deferred | `SB-ENV-042`, `SB-ENV-057`; storage model and multi-resolution pass DEFERRED to `21_data-io.md` / `23_plotting-interactivity.md` |
| §4.10 Gap filling → the form, with its self-contradiction resolved against the dialog | ADOPTED | `SB-ENV-038` |
| §4.11 Per-tool uncertainty → adopt the `unc_*` family; it has no competitor | ADOPTED | `SB-ENV-019`; the soft-rejection sentinel idea folded into `SB-ENV-021`'s degradation contract; term decomposition OPEN at `OI-3` |
| §4.12 Conditioning order → make it a first-class, checkable pipeline contract | ADOPTED | `SB-ENV-018`; content ESCALATED at `ESC-14` |
| §4.13 Log QC limits → adopt the precedence semantics, reject every shipped number | ADOPTED | `SB-ENV-056`; numbers refused at `RF-8`; the between-bands case OPEN at `OI-6` |
| §4.14 Dispositions for the remaining in-domain modules | ADOPTED in part, DEFERRED in part | Adopted: `SB-ENV-036` (cull), `SB-ENV-039` (clip), `SB-ENV-049` (superseded-module delegation), `SB-ENV-037` (restore/backup). Deferred: the regression-bank curve editor to the regression/prediction chapter; rescale to P2 with `O-13` |

### 8.5 §5 — Adoption spec (3 rows; §5.4 and §5.5 are itemised separately)

| Dossier item | Disposition | Where it went |
|---|---|---|
| §5.1 Canonical equation forms | ADOPTED selectively | `SB-ENV-032` (the MAD consistency constant, restated **with its derivation** `1/Φ⁻¹(3/4) = 1.482602` rather than as an asserted number), `SB-ENV-048`, `SB-ENV-051`. Forms whose only source is a vendor chart were NOT restated — see `TR-2` |
| §5.2 Parameter table — every value carries its source string | ADOPTED and extended | §5 of this chapter: **83 rows**, of which 32 are specified `ABSENT — ships with no default` and 29 are recorded `SHIPPED-UNCITED` as concrete open violations of `SB-ENV-004` / `SB-ENV-014` |
| §5.3 Chart-lookup interface contract | ADOPTED as an interface | `SB-ENV-015`, `SB-ENV-017`. Axis count OPEN at `OI-2`; interpolation contract ESCALATED at `ESC-12`; licensing ESCALATED at `ESC-13`; chart data REFUSED at `TR-2` |

### 8.6 §5.4 — Tests to ship (39 rows; see §8.0 for the `T-1b` discrepancy)

| Dossier test | Disposition | Where it went |
|---|---|---|
| `T-1` Bell kernel normalisation | ADOPTED | `SB-ENV-041` |
| `T-2` Bell kernel even-length rounding | ADOPTED | `SB-ENV-041` |
| `T-3` Temperature-correction round trip | ADOPTED | `SB-ENV-048` |
| `T-4` Temperature-correction unit equivalence (6.77 °F ≡ 21.5 °C) | ADOPTED | `SB-ENV-048`, `SB-ENV-045` |
| `T-5` Reject the `(T − 6)` parameter set | ADOPTED | `SB-ENV-009`, `SB-ENV-048`; `RF-1` |
| **`T-1b`** Kernel normalisation convention (the unnumbered row) | ADOPTED | `SB-ENV-041`. Carried explicitly because a mechanical port of the numbered list would drop it |
| `T-6` Normalization exact-percentile | ADOPTED | `SB-ENV-051` |
| `T-7` Normalization unit invariance | ADOPTED | `SB-ENV-034`, `SB-ENV-057` |
| `T-8` Normalization idempotence | ADOPTED | `SB-ENV-053` |
| `T-9` Despike masking reproduces the closed form | ADOPTED | `SB-ENV-031` |
| `T-10` Despike circular / azimuth wrap | DEFERRED — P2 | Trigger: the first module in this domain that consumes an angular curve. No azimuth or dip curve is in v1 scope; `SB-ENV-031`'s method enumeration reserves the branch |
| `T-11` Despike reversibility | ADOPTED | `SB-ENV-037` |
| `T-12` Cull zero-flanked criterion | ADOPTED | `SB-ENV-036` |
| `T-13` Null canonicalisation | DEFERRED — owned elsewhere | `21_data-io.md` (seam §1.1). The write side of it is a read/write contract, not a conditioning one |
| `T-14` DRHO unit guard | ADOPTED | `SB-ENV-026` |
| `T-15` Gradient unit guard | ADOPTED | `SB-ENV-045` |
| `T-16` Depth-shift round trip | ADOPTED | `SB-ENV-042`, `SB-ENV-057` |
| `T-17` Depth-shift step/depth conversion on import | DEFERRED — owned elsewhere | `21_data-io.md`. The conversion is an import-time interpretation of a foreign parameter set |
| `T-18` Idempotent environmental correction | ADOPTED | `SB-ENV-005` (the applied-step manifest is what makes the second application detectable) |
| `T-19` Order enforcement | ADOPTED | `SB-ENV-018` |
| `T-20` Neutron matrix declared | ADOPTED | `SB-ENV-012` |
| `T-21` Flag polarity single-source | ADOPTED | `SB-ENV-030` (specified as a **build gate**, not a runtime test — a second polarity is a type error) |
| `T-22` Gap-fill provenance | ADOPTED | `SB-ENV-038`, `SB-ENV-037` |
| `T-23` Hodges-Lehmann worked example | REJECTED for this chapter | Follows §2.15's boundary call: the pairwise average is a curve-averaging operator. Recorded at `OI-8` so the call is reviewable; if `OI-8` resolves the other way, this test comes with it |
| `T-24` Uncertainty monotonicity | ADOPTED | `SB-ENV-019`. **This is the test that lets the chart interface ship verified against synthetic tables with no chart data present** — the structural discharge of the transcription bar |
| `T-25` Bad-hole reason precedence | ADOPTED | `SB-ENV-022`; encoding form OPEN at `OI-7` |
| `T-26` Normalization percentile-walk termination | ADOPTED | `SB-ENV-051`; `RF-3`, `RF-4` |
| `T-27` Bad-hole degrades gracefully | ADOPTED | `SB-ENV-021` |
| `T-28` Correction-step availability is reported | ADOPTED | `SB-ENV-005`, `SB-ENV-011`, `SB-ENV-020` |
| `T-29` Mud-weight clamp is branch-specific | DEFERRED — P2 | Trigger: `SB-ENV-010`'s chart-backed GR correction. The clamp is a property of a chart family SandiBumi does not yet hold; specifying the test now would require the chart (`TR-2`) |
| `T-30` Normalization endpoints are order statistics, not bin means | ADOPTED | `SB-ENV-051` |
| `T-31` Normalization preset carries its field and reference interval | ADOPTED | `SB-ENV-053`, `SB-ENV-054`, `SB-ENV-055` |
| `T-32` Rescale log base is declared | DEFERRED — P2 | Trigger: a curve-rescale module. Paired with `O-13`, which is the same gap seen from the evidence side |
| `T-33` Sonic-array splice refused | DEFERRED — owned elsewhere | The sonic chapter. The *principle* — a splice across incompatible acquisition types is refused — is generalised here into `SB-ENV-040` and `SB-ENV-018` |
| `T-34` Inverted true-stratigraphic-thickness refused | DEFERRED — owned elsewhere | Deviation and structural geometry, not conditioning |
| `T-35` One differentiate operator | ADOPTED | `SB-ENV-049`; discharges `SB-CORE-006` inside this domain |
| `T-36` A threshold never crosses statistics | ADOPTED | `SB-ENV-001`, `SB-ENV-004` — a threshold is stored with the statistic it belongs to, and its source string says which |
| `T-37` Chart baseline is single-assignment | ADOPTED | `SB-ENV-017` |
| `T-38` Validation lives with the algorithm | ADOPTED — **the chapter's load-bearing test** | `SB-ENV-002`, `SB-ENV-001`, `SB-ENV-003`. This is the test that distinguishes a port that inherited fail-loud from one that inherited fail-silent |

### 8.7 §5.5 — Applicable `FINDINGS.md` §6 rules (13 rows)

| Rule | Disposition | Where it went |
|---|---|---|
| 1 — no raster-only truth | ADOPTED | `SB-ENV-015` (the chart interface is machine-readable and generates its own documentation); `TR-1`, `TR-2` are the compliance side |
| 2 — `atan2`, never `cos⁻¹` | DEFERRED — P2 | With `T-10`; no angular curve in v1 scope |
| 3 — unit-typed quantities | ADOPTED | `SB-ENV-026` (DRHO), `SB-ENV-045` (gradient), `SB-ENV-057` (depth-unit token), `SB-ENV-016` (salinity) |
| 4 — unit-invariant statistics | ADOPTED | `SB-ENV-034`, `SB-ENV-051`, `SB-ENV-057` |
| 5 — one flag convention | ADOPTED | `SB-ENV-030` plus `SB-ENV-022`'s reason channel |
| 6 — null discipline | DEFERRED — owned elsewhere | `21_data-io.md`; `SB-ENV-035` holds the conditioning-side half (a MISSING sample stays MISSING) |
| 9 — defaults cited or absent | ADOPTED — the chapter's most-used rule | `SB-ENV-004`, `SB-ENV-014`, `SB-ENV-016`, `SB-ENV-024`, `SB-ENV-025`, `SB-ENV-052`, `SB-ENV-056`; 32 of §5's 83 rows |
| 10 — docs generated from code | ADOPTED | `SB-ENV-008`, and the two build gates in §6 that enumerate the registry rather than run a module |
| 11 — worked examples reproduce | PARTIAL | Its two instances split: `T-17` DEFERRED to `21_data-io.md`, `T-23` REJECTED with §2.15. The rule itself is honoured by `SB-ENV-032`, whose constant is reproduced from its own derivation |
| 12 — per-correlation unit flags | ADOPTED | `SB-ENV-048` — the 6.77/21.5 pair is stored as one unit-tagged constant, not two numbers |
| 13 — state the reference convention | ADOPTED | `SB-ENV-044` (TVD, not MD), `SB-ENV-046` (mudline vs measurement reference) |
| 14 — silent failures are bugs | ADOPTED — the chapter's organising rule | `SB-ENV-003`, `SB-ENV-005`, `SB-ENV-006`, `SB-ENV-020`, `SB-ENV-021`, `SB-ENV-036`; it is also the rule `SB-CORE-002` states at project level |
| 15 — resolution and depth snapping logged | PARTIAL | `SB-ENV-042` holds the edit-provenance half; the resampling half is DEFERRED to `21_data-io.md`, with `O-11` as the open evidence gap |

### 8.8 §6 — OPEN items `O-1`…`O-15` (15 rows)

| Item | Disposition | Where it went |
|---|---|---|
| `O-1` Is `(T − 6)` a transcription defect or a distinct sourced correlation? | ESCALATED, and neutralised in the meantime | `ESC-4`. `SB-ENV-048` does not depend on the answer: it requires **one** cited constant, and `RF-1` refuses the fall-through that selects the unlabelled branch either way |
| `O-2` No shipped bad-hole cutoffs — only validation ranges | ADOPTED as corroboration | `SB-ENV-024`. The absence is the incumbent's own answer to the question `SB-ENV-024` asks, and it is the strongest single argument that `ABSENT — ships with no default` is a vendor-supported position, not an evasion |
| `O-3` Median-AD or mean-AD in the outlier cleaner? | OPEN, and neutralised | `OI` register / `SB-ENV-031`, which requires the estimator to be **named in the output**. SandiBumi's answer does not depend on which the incumbent chose |
| `O-4` "Smooth with missing values" semantics contradict the doc | ADOPTED — resolved by specification | `SB-ENV-035`. SandiBumi states the semantics rather than inheriting an ambiguity: a MISSING sample contributes no weight and is never bridged |
| `O-5` One correction floored at zero, another not | OPEN | `OI-1` / `SB-ENV-011`. Whether a negative corrected neutron is physical (gas) or a defect is a step-set question, and `OI-1` is where the ten-step canonical order is settled |
| `O-6` House conditioning order vs house study order | ESCALATED | `ESC-14`. `SB-ENV-018` ships the contract mechanism; the content waits on the ruling |
| `O-7` Bad-hole flag latches across an interval | ADOPTED — designed out | `SB-ENV-022`. A per-sample reason channel has no latching state to leak; the interval-level summary becomes a derived quantity rather than the primary one |
| `O-8` The normalization histogram defect is a code read, not a run | ESCALATED | `ESC-8`. `RF-3` refuses the algorithm regardless — SandiBumi's exact-order-statistic requirement (`SB-ENV-051`) does not become wrong if the runtime behaviour differs |
| `O-9` Two house sources disagree on a normalisation well count | EVIDENCE-ONLY — no requirement | A project-record bookkeeping discrepancy, not a method question. Recorded so the disposition table is complete; it generates nothing in this chapter and belongs to the project-record maintainer |
| `O-10` ~~Are the disabled uncertainty steps deliberate?~~ **RESOLVED in the dossier** | EVIDENCE-ONLY | The resolution (deliberate) is what makes §2.2 a *by-construction* finding rather than a suspected bug, and is therefore load-bearing for `SB-ENV-019` and `RF-6` |
| `O-11` Five tool descriptors declare no sampling rate | DEFERRED — owned elsewhere | `21_data-io.md` resampling contract; paired with `FINDINGS` rule 15 and `E-11` |
| `O-12` The cross-correlation normaliser puts the product inside the sum | DEFERRED — P2 | Trigger: automatic depth-matching. `SB-ENV-042` covers manual shifts; auto-correlation is not in v1 |
| `O-13` Rescale logarithmic-mode base not read | DEFERRED — P2 | With `T-32`. `SB-ENV-045`'s declared-unit principle generalises to it when the module is built |
| `O-14` Chart-baseline asymmetry between the two branches | ESCALATED, and adopted as a requirement anyway | `ESC-5` for confirmation; `SB-ENV-017` requires single-assignment named intermediates whether or not the incumbent's asymmetry is deliberate |
| `O-15` Which σ convention the despike uses | ESCALATED, and neutralised | `ESC-9`. §2.5 gives the ceiling under **both** conventions (20.00 % population, 19.19 % sample at k = 2, N = 20) and `SB-ENV-031` requires SandiBumi's own convention to be declared, so the answer changes a displayed number by 0.8 pp and changes no design |

### 8.9 §6 — Ledger dispositions `F-1`…`F-15` (15 rows)

| Item | Disposition | Where it went |
|---|---|---|
| `F-1` Extreme-low QC limit above the user-min limit — confirmed vendor defect | ADOPTED | `SB-ENV-056`; `RF-8`; which panel is authoritative ESCALATED at `ESC-3`; the between-bands case OPEN at `OI-6` |
| `F-2` Three flag polarities in one family | ADOPTED | `SB-ENV-030` (one polarity, as a type), `SB-ENV-022` (the reason channel that removes the need for a second) |
| `F-3` Gap-fill "no limit" prose is stale text | ADOPTED | `SB-ENV-038` — the boundary comparison is stated and an open-ended gap is refused |
| `F-4` Filter length limits disagree three ways | ADOPTED | `SB-ENV-041`; the limit is emitted from source (`FINDINGS` rule 10), so SandiBumi's documentation cannot drift from its own bound |
| `F-5` Gradient units in the image-speed-correction context remain OPEN | REFUSED — do not implement | `RF-5` register and `SB-ENV-045`. `SB-ENV-058` specifies the derived alternative and explicitly avoids the formulation this item sits inside |
| `F-6` Raster pairs visually identical / duplicated | NOT IN SCOPE | Out-of-domain ledger item; it concerns raster provenance in the image-log material |
| `F-7` Parallel documented forms differ by a separator character | NOT IN SCOPE | Out-of-domain ledger item |
| `F-8` One documented operator computes a ratio in four languages and a finite difference in two | ADOPTED | `SB-ENV-049` and `T-35` → discharges `SB-CORE-006` in this domain: exactly one first-difference operator, one name, one equation |
| `F-9` Vendor example code contains an assignment-for-comparison bug | EVIDENCE-ONLY | A defect in **example** code, not in a shipped algorithm. It supports the chapter's general position that vendor documentation is not a specification, and generates no requirement |
| `F-10` Vendor example code increments the wrong loop index | EVIDENCE-ONLY | Same class as `F-9` |
| `F-11` Restore-backup-curves capability | ADOPTED | `SB-ENV-037` (every removed or replaced sample is recoverable), `SB-ENV-042` (interactive edits carry provenance, not only undo) |
| `F-12` Chart-source edition ambiguity for one vendor family | EVIDENCE-ONLY — acquisition note | Feeds `SB-ENV-015`'s "the chart identity includes its edition" clause. **No chart content was read** (`TR-2`) |
| `F-13` A chart source listed in the acquisition table yet stated never received | EVIDENCE-ONLY — acquisition note | Same as `F-12`; the gap is recorded, not filled |
| `F-14` A shipped default formation salinity that is not a physical value | ADOPTED — and it is §2.15's argument | `SB-ENV-016` (a measured property of the formation or borehole ships no default); `RF-7`. Three vendor values spanning five orders of magnitude is the evidence that the right default is no default |
| `F-15` A stale screenshot from an earlier release shipped in the current manual | NOT IN SCOPE | Out-of-domain ledger item; documentation hygiene, no method content |

### 8.10 §6 — Escalations `E-1`…`E-13` (13 rows)

| Item | Disposition | Where it went |
|---|---|---|
| `E-1` One vendor's environmental-correction math is permanently closed to comparison (compiled) | ACKNOWLEDGED — no action, by rule | §1.2's exclusions. `CONTRACT.md` §2.2 forbids the reconstruction path; a compiled engine is not to be reverse-engineered. The consequence is stated in §2.12 and §5's `NON-ADOPTABLE` rows: certain vendor behaviour can be *cited* but never *verified against* |
| `E-2` ~~Where are environmental corrections actually applied?~~ **CLOSED in the dossier** | EVIDENCE-ONLY — and the chapter's headline | The manifest enumeration that closed it **is** §2.1's finding: the capability was invisible from the sources and visible only from the manifests |
| `E-3` The chart-function interpolation contract is undocumented | ESCALATED | `ESC-12`. `SB-ENV-015` requires the rule to be **declared**; it does not guess what it is |
| `E-4` A live session of the current vendor release would close `F-1` | ESCALATED | `ESC-3`. `SB-ENV-056` ships `ABSENT` limits, so the answer changes a preset, not a design |
| `E-5` Acquire the independent log-QC and error-propagation reference | ESCALATED — **the highest-value acquisition for this domain** | `ESC-11`. It is what would let `SB-ENV-019`'s term decomposition be **derived from published work** rather than lifted from fitted vendor coefficients — the required path under `CONTRACT.md` §2.2, not merely a nicer one. Until then, `OI-3` holds |
| `E-6` Confirm the normalization binning defect against a run | ESCALATED | `ESC-8` |
| `E-7` Ruling on where environmental correction sits in the pipeline | ESCALATED | `ESC-14` (merged with `O-6`, which is the same question from the evidence side) |
| `E-8` A correction owed to an earlier ingest record about the pairwise-average operator | ROUTED OUT — not actioned here | The brief forbids modifying anything under the research trees, and this chapter writes to one file only. Recorded here so the correction is not lost; it belongs with the record's owner. Related boundary call at `OI-8` |
| `E-9` Named papers for independent validation of two new vendor modules | DEFERRED — owned elsewhere | The regression/prediction chapter, with the module itself (§4.14). Noted here because the *principle* — independent validation before adoption — is what `SB-ENV-014` requires of every coefficient |
| `E-10` Ruling on the GR uncertainty model | ESCALATED | `ESC-15`. The dossier's own resolution is adopted as `SB-ENV-019`'s **shape**: the mud-chemistry term is derived from the correction module's declared inputs, because two vendors correct for it and neither propagates it |
| `E-11` Per-tool resolution behaviour needs a live session | DEFERRED — owned elsewhere | `21_data-io.md`, with `O-11` |
| `E-12` The compiled production engine's resistivity temperature constant is unknown | ESCALATED | `ESC-4`. This is what narrows §2.3's finding from "the vendor ships a wrong law" to "a Toolbox script ships a wrong law, and the product's own constant is unverifiable" — a smaller claim, and a true one |
| `E-13` Licensing ruling on derivative use of installed vendor chart functions | ESCALATED | `ESC-13`. Until answered the operating rule is unchanged and stricter than the question: **directories are listed, files are never opened** (`TR-2`) |

### 8.11 §7 source register and the compliance statement (6 rows)

| Item | Disposition | Where it went |
|---|---|---|
| `T1` — executable / declarative source read this session | EVIDENCE-ONLY — and it sets the Tier column | Every `T1` citation in §5 is a manifest, a source file or a declaration read directly. §2.1's finding exists *because* the `T1` boundary was drawn around manifests as well as code |
| `T2` — full-manual ingest reports | EVIDENCE-ONLY | Sources §2.10, §2.13, §2.14, §2.15 and the `NON-ADOPTABLE` vendor rows in §5 |
| `T3` — install-tree / catalogue ingests and vendor doc pages | EVIDENCE-ONLY | Sources §2.12's chart census. **Census only** — a file count, never a file read (`TR-2`) |
| `T4` — course notes, house standards, project records | EVIDENCE-ONLY | Sources the house gates cited in §2.2 and §2.4 as **verification checks, never as adopted values**. This distinction is enforced in §5 by the `NON-ADOPTABLE — cited for verification` marker |
| `Tools` | EVIDENCE-ONLY | Methodological only; generates nothing |
| Compliance statement | ADOPTED as this chapter's own model, with one correction inherited | The dossier's compliance statement was itself corrected during critique (`MIN-9`): a blanket "no client well names" claim was **false and retracted** in favour of a narrower, checkable one. This chapter follows the corrected practice — it states what was verified and how, not what it hoped was true, and it states the exceptions rather than the aspiration. **Verified by scan, for this file:** (a) **no client well name appears** — no well identifier of any form is present in the text, tables or citations; (b) **no operator asset name appears in the prose** — the owner's 2026-08-07 directive that an operator's asset is not the basis of SandiBumi's methods is honoured by describing the *conditions* (fresh formation water, high formation temperature, thinly interbedded clastics) rather than the basin; (c) **no vendor chart datum appears** — not one number traceable to a correction chart is reproduced anywhere (`TR-2`). **The one exception, stated rather than hidden:** five rows in §5.2 cite delivered-study decision records **by filename**, and three of those filenames contain a field or block name. They are kept because `CONTRACT.md`'s parameter discipline requires every value to trace to a *named, retrievable* source, and an anonymised filename is not retrievable — provenance outranks the cosmetic scrub. Those rows are `NON-ADOPTABLE — cited for verification` presets, not adopted defaults, and this is an internal, local-only document citing a local-only records set. Written out this way because a compliance claim that is broader than its own verification is the same defect class as a curve labelled "corrected" that was not corrected (`SB-ENV-006`) |

### 8.12 Critique disposition (27 rows)

The dossier's critique disposition is not evidence about the incumbents — it is evidence about **the
dossier**. Each row below records what this chapter consumed as a result, because in five cases the
*corrected* finding is stronger than the one it replaced and is the version this chapter is built on.

| Item | Disposition | Where it went |
|---|---|---|
| `BLK-1` The manifest library was never enumerated; three "no evidence held" statements were false | ADOPTED — **this is §2.1** | The fix produced the chapter's centre of gravity. Without it the domain would have been specified from the sources alone, which is precisely the fail-silent port `SB-ENV-001`…`SB-ENV-003` exist to prevent. A finding this large arriving from a *correction* is itself an argument for `SB-ENV-008` |
| `BLK-2` The chart-function library was under-reported by more than half | EVIDENCE-ONLY | §2.12; `TR-2`; `ESC-13`. It changed an acquisition conclusion, not a requirement |
| `MAJ-1` The commented-step count was wrong on both numbers | ADOPTED in corrected form | §2.2 rests on the corrected count and on `O-10`'s resolution, not on the withdrawn headline |
| `MAJ-2` A Toolbox script was over-generalised into a product default | ADOPTED — scope narrowed | §2.3 states the narrower claim; `ESC-4` holds what remains unknown. `SB-ENV-048` is unaffected either way |
| `MAJ-3` A manufactured contradiction, replaced by a genuine three-way split | ADOPTED in corrected form | §2.3 → `SB-ENV-031`, `SB-ENV-009` |
| `MAJ-4` An uncertainty comparison mixed a single term against a total | ADOPTED in corrected form | §2.11, §3.10 → `ESC-15`, `SB-ENV-019` |
| `MAJ-5` Unit error in the neutron temperature sensitivity row | ADOPTED in corrected form — **and it is the divergence this chapter quantifies** | The corrected figure is what makes the shipped coefficient checkable at all. §5 carries it as `NON-ADOPTABLE — cited for verification`; the divergence it exposes is `ESC-2`; the requirement is `SB-ENV-014`. It is deliberately **not** adopted as a value |
| `MAJ-6` An uncited coefficient inside the adopted equation set | ADOPTED in corrected form | `SB-ENV-032` — the constant ships **with its derivation**, `1/Φ⁻¹(3/4) = 1.482602`, which is why it is one of the few numbers in this chapter that is neither `ABSENT` nor `SHIPPED-UNCITED` |
| `MAJ-7` Systematic compaction of a vendor's parameter contract | ADOPTED in corrected form | §2.2's step-for-step comparison → `SB-ENV-011`, `OI-1` |
| `MAJ-8` Two in-domain doc pages never opened; one falsified a "no evidence held" claim | EVIDENCE-ONLY | §2.12 |
| `MAJ-9` Whole in-domain modules had no disposition | ADOPTED | §4.14 → `SB-ENV-036`, `SB-ENV-039`, `SB-ENV-049`, `SB-ENV-037` |
| `MAJ-10` The ledger in/out-of-domain split was wrong in five places | ADOPTED | §8.9 is built on the corrected split. Had it not been corrected, three in-domain requirements (`SB-ENV-049`, `SB-ENV-037`, `SB-ENV-042`) would have had no antecedent |
| `MIN-1` Mud-weight clamps are branch-specific, not single | ADOPTED | `SB-ENV-010`; test DEFERRED with `T-29` |
| `MIN-2` The chart-baseline clobber | ADOPTED | §2.6.3 → `SB-ENV-017`, `ESC-5` |
| `MIN-3` Two fail-silent paths confirmed — no `else`, and a non-matching branch routed silently | ADOPTED — **directly load-bearing** | `SB-ENV-009` (an unmatched method string is an error) and `SB-ENV-003`. A fall-through with no `else` is the mechanism, not a metaphor |
| `MIN-4` Two unguarded array writes driven by user-settable quantities | ADOPTED as a refusal | `RF-4` |
| `MIN-5` Normalisation differs in **two** ways, not one | ADOPTED in corrected form | `SB-ENV-051` covers both the percentile choice **and** the endpoint estimator; the withdrawn "only the percentile differs" would have produced a requirement that missed the estimator |
| `MIN-6` The σ convention is now declared; the finding's own arithmetic was rebutted | ADOPTED, rebuttal included | §2.5 → `SB-ENV-031`, `ESC-9`. See the "Not applied" row below |
| `MIN-7` Attribution corrected to the exact quoted source | EVIDENCE-ONLY | House-source citation hygiene; nothing in §5 depends on which of the two records is cited, but the citations in §2.4 use the corrected one |
| `MIN-8` A line pointer was off by two and is corrected | EVIDENCE-ONLY — **and it is a discipline, not a typo** | This is exactly why the brief requires every `file:line` pointer to be re-verified at source rather than repeated from another document. Every as-built pointer in §3 and §5 of this chapter was read in-session for that reason |
| `MIN-9` The "no client well names" compliance claim was false and is retracted | ADOPTED as practice | §8.11's compliance row states only what was checked. A compliance claim that is broader than its verification is the same defect class as a curve labelled "corrected" that was not corrected (`SB-ENV-006`) |
| `MIN-10` A third estimator name appears in a third doc page | ADOPTED | `O-3`, `SB-ENV-031` — three names for one operation is the `SB-CORE-006` instance inside the despike family |
| `MIN-11` Vendor-name grouping in the chart census is lower quality than stated | EVIDENCE-ONLY | §2.12; it weakens a census, not a requirement |
| `MIN-12` The correction owed to an earlier record is the fifth, not the fourth | ROUTED OUT | With `E-8`; not actioned here, by the brief's write scope |
| `MIN-13` A sonic despike taxonomy changes the design, not just the prose | ADOPTED | `SB-ENV-031`'s method enumeration carries the distinct sonic artifact classes rather than treating "despike" as one operation |
| "Not applied" — the rebutted claim about the masking direction and magnitude | ADOPTED as evidence — **the chapter's numeric centre** | §2.5 derives `f* = 1/(k²+1)` for IP's estimator (exactly 20 % at `k = 2`, independent of spike amplitude) and `f* = min(1/k, ½)` for the Hampel SandiBumi actually ships (33.3 % at `K = 3`). The ceiling **falls as `k` rises** in both. A first draft of this section read the direction backwards and asserted the opposite in two places while stating it correctly in two others; corrected 2026-08-07 against the derivation at `condition.rs:154-172`, which is also what turned up the wall at `min(…, ½)` and `ESC-16` |
| "What this revision did NOT do" | ADOPTED as the compliance model | Its four commitments — no default from memory, no chart data read, no Tier-C item reconstructed, nothing correct shortened — are the four this chapter holds itself to, and §7.3 is where it reports against them |

### 8.13 Requirements with no dossier antecedent

Fourteen requirements originate in the **shipped SandiBumi source**, not in the dossier. They exist
because the dossier compares incumbents to each other, while this chapter also compares the incumbents
to what is on disk — and the as-built code has divergences the incumbents do not have. Listed so the
traceability runs both directions and no requirement appears from nowhere.

| Requirement | Origin |
|---|---|
| `SB-ENV-006` — a curve named "corrected" MUST have been corrected | The env-correction family's documented pass-through: a missing QC input "passes the log through uncorrected rather than blanking", producing a `*_EC` curve identical to its input with nothing marking it |
| `SB-ENV-008` — validity conditions visible before the run | Follows from §2.1 plus `SB-CORE-003`: if enforcement lives in a manifest, the manifest is also what the dialog should be able to show |
| `SB-ENV-020` — correction-chain QC | Synthesis of `SB-ENV-005` and the as-built results-QC module; no incumbent offers it |
| `SB-ENV-023` — the density correction's sign is preserved and reported | As-built: the bad-hole rule takes the absolute value of the density correction, discarding the sign that distinguishes mudcake from washout |
| `SB-ENV-025` — bit size is an input, never a default | As-built: a shipped 8.5 in bit-size default is a well property wearing a manifest's authority |
| `SB-ENV-027` — a repair module MUST be exempt from the mask | As-built, and an **audited defect already documented in the codebase's own test**: the universal run mask blanks both the inputs and the outputs of the very module meant to repair the masked interval |
| `SB-ENV-028` — the mask is recorded in the run's provenance | Same as-built finding; a result computed under a mask and one computed without it are indistinguishable after the fact |
| `SB-ENV-029` — conditioning flags validate their own stated preconditions | As-built: a matrix-density precondition stated only in a source comment, with nothing checking it |
| `SB-ENV-040` — a conditioning output is never the input's own mnemonic | As-built (already satisfied); recorded so the property is a requirement rather than an accident |
| `SB-ENV-043` — one formation-temperature definition, one mnemonic | As-built: **two shipped modules both emit `FTEMP`**, differing by 33.1 °C at 2 000 m TVD and propagating to ≈14 % relative on `Sw`. The domain's own `SB-CORE-006`/`SB-CORE-007` instance |
| `SB-ENV-047` — a declared parameter that does not enter the answer is removed or used | As-built (already satisfied); recorded so the property is a requirement rather than an accident, and kept because `SB-ENV-T55` is what would have caught this requirement's own first-draft misread. Basis of `ESC-10`, now amended down to preventive |
| `SB-ENV-049` — a superseded module delegates to the survivor and says so | As-built (already satisfied, and explicitly documented as "not a second implementation"); generalised into the rule that discharges `SB-CORE-006` |
| `SB-ENV-050` — a depth-trend parameter is well-scoped, a compartment parameter is not | As-built: the well-scope mechanism exists and its justification is recorded in source with a worked failure case; the requirement makes the *criterion* explicit rather than leaving it per-parameter judgement |
| `SB-ENV-057` — one token for "a length in the project's depth unit" | As-built: **three tokens** for one concept, one of which is false on a foot project. The domain's `SB-CORE-007` instance in units rather than constants |

One further requirement, `SB-ENV-058`, has no dossier antecedent either, but for a different reason:
it is an **independent-derivation requirement** created under the amended `CONTRACT.md` §2.2 and is
accounted for in §7.4 rather than here.

### 8.14 Coverage summary

**178 dossier rows, 178 dispositions, no row unaccounted for.** Counted from the tables above, not
asserted:

| Disposition | Rows | Share |
|---|---|---|
| ADOPTED (including 13 qualified forms — *in corrected form*, *as an interface only*, *in part*) | 115 | 65 % |
| EVIDENCE-ONLY | 21 | 12 % |
| DEFERRED | 15 | 8 % |
| ESCALATED | 14 | 8 % |
| NOT IN SCOPE (out-of-domain ledger items) | 3 | 2 % |
| REJECTED for this chapter | 2 | 1 % |
| PARTIAL (split across two chapters) | 2 | 1 % |
| ROUTED OUT (belongs to a file this chapter may not write) | 2 | 1 % |
| OPEN | 2 | 1 % |
| REFUSED | 1 | 1 % |
| ACKNOWLEDGED — no action available, by rule | 1 | 1 % |
| **Total** | **178** | |

**What the shape of that table says.** Two figures are worth reading together. **Sixty-five per cent
adopted** is high for a cross-tool dossier, and it is high for a specific reason: this domain's
findings are overwhelmingly about *enforcement* rather than about *equations*, and enforcement is
portable in a way that a fitted coefficient is not. A requirement that says "the validity condition is
evaluated before the module body runs" costs nothing to adopt and depends on no vendor's numbers.

The second figure is the counterweight. **Thirty-two of §5's 83 parameter rows ship
`ABSENT — ships with no default`, and sixteen more are `NON-ADOPTABLE — cited for verification`.** So
the chapter adopts almost every *rule* the dossier found and almost none of its *numbers*. That is the
intended asymmetry, and it is the concrete meaning of `CONTRACT.md` §2.2 in a domain whose numbers
live in charts this project does not hold: **the enforcement layer ships; the chart data does not.**

The single largest deferred group is not a shortfall in this chapter but a seam: null discipline,
depth-unit parse-and-carry, import-time parameter conversion, and per-tool resampling all belong to
`21_data-io.md`, and are listed in §1.1 as owned there rather than duplicated here.

**Where the completeness gate is loosest, stated plainly.** Three rows disposition an item to a
chapter that does not yet exist in final form (`21_data-io.md`, the sonic chapter, the
regression/prediction chapter). If any of those chapters declines the item, it returns here rather
than disappearing — which is why each such row names the trigger as well as the destination.

---

**End of chapter.**
