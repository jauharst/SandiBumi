# DRAFT — SB-ENV-004 tier-1 source adjudication table (awaiting per-row rulings)

**Status: DRAFT, delivered 2026-08-19 under DEC-072/DEC-073.** DEC-072 ruled ENV-004 closes
as the CITATION row it is: engineering drafts the candidate-source table for the
SHIPPED-UNCITED environmental parameters from the ingested corpora, and Jauhar adjudicates
EACH row — **adopt** (cite the named source), **mine** (his multi-basin value with
practitioner attribution, the DEC-069/DEC-059 pattern), or **absent** (the user supplies per
study). No value below is adopted without his word; per DEC-060(c) parameters keep shipping
STARTING VALUES either way, and `BHT`/`TD_BHT` stay flagged as the two rows where a shipped
starting value sits uneasily. Tier 2 (full chart transforms via chartdig) and tier 3
(per-tool-generation vendor suites) are out of this row's scope per the same ruling.

**Corpus ground rules.** Candidates come from the ingested corpora only, cited to file and
section with evidence tier; a demo value in a vendor dialog is NOT a default (corpus
convention); vendor chart lookup DATA is never transcribed. **The Halliburton extraction
ran 2026-08-19** (Halliburton Sperry Drilling, "LWD Log Interpretation Charts", 2018 — the
local chartbook; targeted tier-1 read recorded in the gitignored
`docs/research_2026-08/halliburton_lwd_chartbook_ingest.md`; book page N = PDF N+12), and
it settled the eChartbook's identity: the printed book is deliberately a SUBSET, and the
**eChartBook™ DEC-072 named is Halliburton's ONLINE chart generator**
(eChartBook.Halliburton.com, Foreword p.xi and p.268 — "for any combination of conditions,
for both LWD and wireline tools") — a per-study generator, not an extractable static
default source. The column below therefore carries what the printed book actually states,
and "generate per study via eChartBook" is the honest vendor route for conditions beyond it.

**The corpus's central negative, stated once.** IP and Geolog perform environmental
corrections through CHART transforms, not through SandiBumi's pragmatic linearized
coefficients — so for the coefficient rows below the honest tier-1 finding is that **no
vendor ships an adoptable number for that coefficient**; the admissible options are "mine"
or "absent", and the chart route is tier 2 by ruling.

## The 29 rows (chapter 20 §5's own SHIPPED-UNCITED inventory: 19 acquire-a-source + the
## 10 re-opened by DEC-060(c))

### GR and density hole corrections

| Parameter | Ships | IP 2025 | Geolog | SLB chartbook | HAL eChartbook | Admissible options |
|---|---|---|---|---|---|---|
| `K_GR` (GR hole coefficient, 1/in) | 0.0075 | No linearized coefficient — chart route (corpus negative) | No unc_* linear coefficient found | Chart family exists (identity only; tier 2) | Per-tool multiplicative scale-factor charts to STATED reference conditions (fresh-water-filled nominal hole, centred tool; e.g. Chart 2-1: 6½-in, book 16-17); no universal coefficient — NEGATIVE for K_GR | mine / absent |
| `K_RHO` (density hole coefficient, g/cc per in) | 0.004 | Same negative | Same | Chart family (tier 2) | No density hole-correction coefficient in the printed LWD subset; per-study via eChartBook | mine / absent |
| `HD_REF` (reference hole diameter, in) | 10.0 | — | — | — | The reference diameter is PER-TOOL and stated on each chart (6½-in for the 4¾-in DGR, Chart 2-1) — corroborates SB-ENV-013 exactly | mine / absent (SB-ENV-013: a property of tool and bit) |

### Neutron environmental correction

| Parameter | Ships | IP 2025 | Geolog | SLB chartbook | HAL eChartbook | Admissible options |
|---|---|---|---|---|---|---|
| `K_TEMP` (v/v per °C) | 0.0001 | Chart route; no linear coefficient (negative) | — | CNL temperature charts (tier 2) | Chart waterfall only, no linear coefficient (book 268-270) — NEGATIVE | mine / absent |
| `T_REF` (chart reference °C) | 24.0 | Dialog shows Deg C units, no stated default (F_qc §3, img-read _encclip0003) | — | Chart reference conditions (tier 2) | **A REAL CANDIDATE: "The reference temperature is 70°F" (book 270); mud-weight reference "fresh water at atmospheric pressure and 70°F (21.1°C)" (book 268)** — i.e. 21.1 °C, against the uncited 24.0 shipped | **adopt (21.1 °C, cited)** / mine / absent |
| `K_SAL` (v/v per 100 kppm) | −0.002 | Chart route (negative) | — | Tier 2 | Chart waterfall only (book 268-269) — NEGATIVE | mine / absent |
| `SALW` (formation salinity, ppm) | 20000 | **Vendor defaults exist and disagree by five orders**: SLB CNL panel ships `2.8E-4 Kppm` (recorded as-is; 0.28 ppm, a unit-artifact-looking number — F_qc §3:380/:541, T2 img-read) and GE ships `0 kppm` (:388); ledger item 14 | — | — | Reference condition is a concentration of ZERO — fresh water (book 268-269); and the axes are **kppm Cl⁻**, not NaCl ppm — a unit-identity fact for whatever value is adopted | adopt-with-caution (three vendors now agree the REFERENCE is fresh) / mine / absent |

### Bad-hole and conditioning flags (DEC-057(c)/DEC-060(c): starting values re-opened)

| Parameter | Ships | Candidates | Admissible options |
|---|---|---|---|
| `DRHO_MAX` (g/cc) | 0.05 | The chapter's own §5 note: matches NONE of the seven precedent values it tabulates | mine (his studies') / absent |
| `DCAL_MAX` (in) | 1.0 | The chapter's own §5 note: **half the value used by every delivered study** — the "mine" option has a concrete recorded referent | mine / absent |
| `BS` (bit size fallback, in) | 8.5 | No admissible source; SB-ENV-025 records the slim-hole silence | mine / absent |
| `COAL_RHOB` / `COAL_NPHI` / `COAL_DT` | 1.9 / 0.35 / 100.0 | DEC-057(c) kept them as starting values; DEC-060 names DEC-059's practitioner attribution as the natural home | mine (attribute) / absent |
| `TIGHT_PHI` | 0.05 | Same DEC-057(c) family | mine (attribute) / absent |
| `XOVER_MIN` | 0.04 | §5's own warning: equals the matrix-scale error size (SB-ENV-012/029) — worth his explicit look | mine / absent |
| `MIN_THICK` / `SHOULDER` (depth units) | 0.25 / 0.5 | No vendor source; resolution-scale conventions | mine (attribute) / absent |

### Conditioning estimator constants

| Parameter | Ships | Candidates | Admissible options |
|---|---|---|---|
| Hampel `K` | 3.0 | Self-declared "ordinary three-deviation convention, NOT a field calibration" (condition.rs); IP's despike is the same mean ± k·σ SHAPE (`curve_despike.htm`, F_qc §2.5, T2) but the manual states no default k (negative) | adopt-the-convention-with-literature-citation (his to supply) / mine / absent — ESC-16 escalation noted |
| `MIN_HAMPEL_SAMPLES` | 5 | An estimator property, not rock (§5's own note) | mine (attribute) / absent |
| `SENS` (bed detect, noise units) | 2.0 | In-house heuristic (see the DBM-005 map: bed_detect is IN-HOUSE) | mine (attribute) / absent |
| `DEFAULT_DIVERGENCE` (Sw v/v) | 0.10 | QC display threshold, user-overridable | mine (attribute) / absent |

### Temperature chain (DEC-060(c)'s hardest rows)

| Parameter | Ships | Candidates | Admissible options |
|---|---|---|---|
| `TSURF` / `TGRAD` (ftemp_grad) | 26.7 °C / 0.03 °C/m | **Corpus negative with TWO vendor citations now**: IP 2018 states no default gradient, no default surface temperature, no default reference depth (`A_porosity_sw.md:559`, T2); Halliburton's Charts 1-2/1-3 implement the SAME linear form with NO default — the user must supply the gradient or a measured reference-depth temperature, and the mean surface temperature "varies according to the geographical location" (book 3-5). Halliburton's plotted metric gradient family is 1–3 °C/100 m; the shipped 0.03 °C/m sits at its TOP (factual note, not an adjudication) | mine (his basin) / absent (twice vendor-corroborated) |
| `SURF_TEMP` / `TEMP_GRAD` (precalc) | 77 °F / 0.026 °/ft | The module doc already says these are one study's feet-based fits — the SB-ENV-045 66 °C metric-project error is the cost of keeping them | mine (re-attributed) / absent |
| `BHT` / `TD_BHT` | 100.0 °C / 2000.0 m | **FLAGGED per DEC-060(c), not decided here**: well-specific facts with no defensible general default — the one place the starting-value ruling sits uneasily, in the ruling's own words. Halliburton's own worked example (book 3) takes a MEASURED temperature at a known TVD as user INPUT — the vendor pattern is well data, never a default | his call: absent, or a starting value he explicitly owns |

### Owned elsewhere (listed so the 29 count is complete, not for adjudication here)

| Parameter | Ships | Owner |
|---|---|---|
| `RHO_MA` (condflag) | 2.645 | The value belongs to `13_mineral-solver.md`/`11_porosity.md`; chapter 20's row records only that condflag must READ the single definition (the SB-CORE-007 draft's C3 item 3 carries the topic-level question) |
| `RHO_FL` (condflag) | 1.0 | `11_porosity.md` |

## The T07 31/32 identity reconciliation (docs-only, per DEC-072)

Chapter 20 §5's authoritative count says **32** parameters are specified `ABSENT — ships
with no default`; T07's header says **31**. The chapter is a SOURCE and is not edited; this
draft records the discrepancy and the mechanical finding: the `ABSENT — ships with no
default` token appears 39 times in the chapter, 23 of them in single-line rows whose
identity parses mechanically (`K_GR, GR_TOOL_POS, GR_TOOL_SIZE, K_TEMP, T_REF, K_SAL, SALW,
SOCN, MUDBASE, MUDTYPE, FPRESS, K_RHO, HD_REF, LITHSCALE, DRHO_MAX, DCAL_MAX, BS, WINDOW,
THRESH, MAX_RATE, MAX_GAP, INTERVAL, MIN_BED`), the rest in multi-line rows needing a
row-aware pass. The exact 31-vs-32 answer requires that by-hand enumeration against T07's
intended set, lands with the engineering that follows adjudication (the zero-exception
T06/T07 build gates), and is recorded in DECISIONS when resolved — never by silently editing
either count.

## What follows the rulings

Per row adjudicated: adopt → the source string lands on the `ArgSpec` (T06's build gate
covers it); mine → the DEC-059 practitioner attribution; absent → `param_open` and the run
refuses without a value. Then the ENV-domain identity inventory, the combined
source+validity `params_json` record, and zero-exception T06/T07 — and SB-CORE-003's
whole-pilot-registry re-audit closes behind this row, as its own tracker states.
