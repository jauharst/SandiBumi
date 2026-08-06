# IP 2025 CHM ingest — discrepancy ledger

Companion to [FINDINGS.md](FINDINGS.md). Sources: the 14 slice reports in this folder
(B–O), each of which carries the full evidence trail (`[img-read:]` tags, page names,
verification notes). Rule unchanged from the 2018 ledger: **nothing is resolved by
preference or textbook knowledge — only by evidence internal to the manuals, or left
OPEN.** Where an agent inferred a reading, the inference is labelled as such.

Numbering:

- **D-01 … D-15** — the *global* ledger. D-01…D-07 carried from
  `ip2018_chm_ingest/DISCREPANCIES.md`; D-08…D-15 added by agent B this ingest
  (`B_core_petro.md` §5). These are the cross-cutting, method-level items.
- **X-…** — slice-internal items, namespaced by agent letter (e.g. `J-D1`,
  `H-D-8`, `L-D-L-02`). The per-slice index in Part 3 uses each report's own IDs;
  the report is the authority. Note M and H both have a local "D-10/D-11" —
  always cite with the agent prefix.

---

## Part 1 — Global ledger after the 2025 ingest

Status index — each item's full adjudication is in its own headed entry below:

| ID | Item | 2025 status |
|---|---|---|
| D-01 | Clip Low % default 0 or 98 | RESOLVED (corroborated) |
| D-02 | Hingle Y-axis stated two ways | RESOLVED |
| D-03 | `Stieber` vs `Steiber` spelling | NOTED |
| D-04 | `RQI` defined two ways | RESOLVED — namespace collision (now three modules) |
| D-05 | Pc↔height with/without 0.433 | RESOLVED (corroborated, sub-item sharpened) |
| D-06 | Pore-size array start 0.01 vs 0.1 µm | RESOLVED — vendor fixed |
| D-07 | Clay-bound-water `F` unbalanced brackets | OPEN |
| D-08 | Waxman-Smits `B(T,Rw)` bracket defect | RESOLVED internally |
| D-09 | Excavation-effect exponent, PhiSw vs SSM | OPEN |
| D-10 | Shell variable-`m` 0.018 vs 0.019 | OPEN — Jauhar's call |
| D-11 | Shale-zone porosity limit malformed in ASCII | RESOLVED — image form |
| D-12 | Juhász / Waxman-Smits prose drops `×Rw` | RESOLVED — raster form |
| D-13 | Sonic Sand default 56 µs/ft vs "(180 µS/m)" | RESOLVED — 56 µs/ft |
| D-14 | Clay parameter (56) missing numbered entry | RESOLVED — authoring defect |
| D-15 | `SW` / `SWE` nomenclature conflict | OPEN — design mandate |

### D-01 — Clip Low % default 0 or 98 — RESOLVED (corroborated)

- 2018 status: RESOLVED — Low 0 / High 98.
- 2025 adjudication: **Corroborated; doc bug still shipped.** `basicloganalysis.htm` is character-identical (duplicated Clip-High block); `clayparameters.htm` (59)/(60) internally consistent in both editions; `ClayVol.hlp` states no default. Adoption unchanged: **0 / 98**. (B §5)

### D-02 — Hingle Y-axis stated two ways — RESOLVED

- 2018 status: OPEN.
- 2025 adjudication: **RESOLVED — `Y = Rt^(−1/m)`** (and `Rxo^(−1/m)`). Closed **independently twice**: B found five consistent computed-curve definitions vs one self-correcting boilerplate paragraph; L (D-L-01) reached the same verdict from the plotting side (3 statements + the executable curve definition vs 1). The `(1/Rt)^(−1/m)` phrasing is a vendor typo — and it is *not* harmless: it is the reciprocal, which would invert the plot. (B §5; L §2.2, §6)

### D-03 — `Stieber` vs `Steiber` spelling — NOTED

- 2018 status: NOTED.
- 2025 adjudication: Unchanged. Alias matching must accept both.

### D-04 — `RQI` defined two ways — RESOLVED (namespace collision, now three modules)

- 2018 status: RESOLVED — namespace collision.
- 2025 adjudication: **Now a three-module collision.** IP 2025's new Normalised-J workflow (cappressurefunctions) uses the 0.0314 form, HFU uses the 0.0314 form, Log-Sw-vs-Height still uses the bare `√(K/φ)`. Same adoption: namespace, never unify — coefficients are not portable between modules (≈31.8× scaling). (E-D1)

### D-05 — Pc↔height with/without 0.433 — RESOLVED (corroborated, sub-item sharpened)

- 2018 status: RESOLVED — 0.433 psi/ft, depth-unit aware.
- 2025 adjudication: Corroborated; the no-0.433 crossplot form persists in 2025 (E-D2, a factor-2.31 trap). **Sub-item sharpened:** back-solving the *2025* worked example (0.70856 from 1.12/0.4/2.27) gives 0.43352 — matching the exact fresh-water gradient 62.428/144 = 0.43353 psi/ft. Labelled inference (E-D9 / E-OPEN-11): IP's internal constant appears unrounded; confirm against a second live report before adopting more digits than the documented 0.433.

### D-06 — Pore-size array start 0.01 vs 0.1 µm — RESOLVED (vendor fixed)

- 2018 status: RESOLVED — 0.01 µm.
- 2025 adjudication: **Vendor fixed it.** IP2025 corrects the Throat Size text to 0.01, making the page self-consistent (E §6.2). The 2018 arithmetic resolution is now the vendor's own text.

### D-07 — Clay-bound-water `F` unbalanced brackets — OPEN

- 2018 status: OPEN.
- 2025 adjudication: **Still OPEN; provenance settled.** The malformed string `F = 1 - [0.6425 * (Salinity ^ (-0.5) + 0.22 ] * Qv]` is **byte-identical in both editions' raw HTML** — this eliminates the "decompiler dropped a glyph" hypothesis; the defect is in the vendor source. Two grammatically reachable readings differ materially (E-OPEN-1: 0.22 inside vs outside the 0.6425 scaling). Resolve **only** against Hill, Shirley & Klein 1979, SPWLA 20th Annual Symposium Paper AA, or the rendered live help. Unit traps stated by the manual: Salinity in Kppm, Qv in meq/ml. (B §5, E-D7)

### D-08 — Waxman-Smits `B(T,Rw)` bracket defect — RESOLVED internally

- 2018 status: *(not captured in 2018)*.
- 2025 adjudication: **RESOLVED internally — net recovery.** The PhiSw raster (embim118) has an unmatched `)`; the Sand/Silt/Malay page states the same formula with balanced brackets. Adopt the SSM rendering: `B = (−1.28 + 0.225·T − 0.0004059·T²) / (1 + Rw^1.23 · (0.045·T − 0.27))`. Temperature units not stated on the equation page (B-OPEN-9: implied degF, not asserted). (B §5)

### D-09 — Excavation-effect exponent `(ρma/2.65)²` (PhiSw) vs `√(ρma/2.65)` (SSM) — OPEN

- 2018 status: *(new)*.
- 2025 adjudication: **OPEN.** Both readings verified at native resolution. May be a genuine module difference; the modules are otherwise near-identical in hydrocarbon handling. Do not assume one form covers both. (B §5)

### D-10 — Shell variable-`m`: 0.018 (raster) vs 0.019 (prose) — OPEN

- 2018 status: *(new)*.
- 2025 adjudication: **OPEN — and found independently by two agents.** B: image (embim120, 6×) 0.018 vs nine-plus prose statements across two editions 0.019. C (§5.1): same split on its own pages, in **both** 2018 and 2025 rasters vs ASCII — a longstanding vendor self-contradiction. C adds: the published Shell formula uses 0.019 (ASCII agrees with literature, raster does not), and the stakes: at φe = 0.02, m = 2.77 vs 2.82 → ~5–10 % Sw error in tight rock. **Decision belongs to Jauhar against the published Shell source; neither agent adopted a value.**

### D-11 — Shale-zone porosity limit malformed in ASCII — RESOLVED (image form)

- 2018 status: *(new)*.
- 2025 adjudication: **RESOLVED — adopt the image form** (embim71): `Phi_limit = (PhiMax + ΔPhiMax) × (1 − Vcl) × 10^(−10 × (Vcl − VclCutoff)^1.6)`. (B §5)

### D-12 — Juhász / Waxman-Smits prose drops the `×Rw` factor — RESOLVED (raster form)

- 2018 status: *(new)*.
- 2025 adjudication: **RESOLVED — adopt the raster form (with `Rw`).** Found independently by B (embim115/116 + two SSM images) and C (§5.2, which also recovers the `m*` exponent for W&S). The ASCII form is dimensionally wrong (adds a conductivity to a dimensionless 1). **The 2018 report currently carries the wrong ASCII form — correction owed** (see FINDINGS §8). Main-agent spot-check of embim115 confirmed the raster reading.

### D-13 — Sonic Sand default 56 µs/ft vs "(180 µS/m)" — RESOLVED (56 µs/ft)

- 2018 status: *(new)*.
- 2025 adjudication: **RESOLVED — 56 µs/ft**; the metric parenthetical is the defect (should read ~184). Decisive: `PhiSw.hlp` independently states 56. Separately noted: BLA defaults sandstone DT Matrix to 55 — a module difference, not an error. (B §5)

### D-14 — Clay parameter (56) "Percentile Clean" missing its numbered entry — RESOLVED (authoring defect)

- 2018 status: *(new)*.
- 2025 adjudication: **RESOLVED as a CHM authoring defect** (both editions jump (55)→(57)); `ClayVol.hlp` carries it correctly at n=56 with full semantics. **No default stated in any source** — the "10th percentile" in prose is an illustration, not a default. (B §5)

### D-15 — `SW` / `SWE` nomenclature conflict — OPEN (design mandate)

- 2018 status: *(new)*.
- 2025 adjudication: **OPEN as a design mandate.** Appendix 1 defines SW = total / SWE = effective; the PhiSw module uses SW = effective / SWT = total. Two incompatible conventions in one manual. SandiBumi must pin one scheme and **never emit a bare `SW`**. (B §5)

---

## Part 2 — Cross-slice reconciliations

Items that touched more than one slice; adjudicated here at synthesis, with the
evidence trail named.

**R-1 — Hingle: CLOSED by independent double-resolution.** B and L worked disjoint
page sets and converged on `Rt^(−1/m)` (see D-02). L's OPEN-L-02 (asking B to
confirm) is hereby satisfied.

**R-2 — Smectite / montmorillonite audit (Rule 8), full-corpus result.**
Eleven of fourteen slices report a hard nil (B's clay/porosity pages, C, D, E, F,
G, I, K, L, M, N). What exists in the whole manual:
- **Alberty/McLean smectite–illite *density* relations — fully recovered** (J §2,
  on `acoustic_to_pressure` / `overburden_tools` / `density_estimation`):
  `RHOB_sm = 2.918 − 0.00517·Dt`, `RHOB_il = 3.044 − 0.00505·Dt`, blended by
  Katahara's tanh with **160 °F onset / 220 °F complete** K-feldspar-breakdown
  endpoints; both calibrated **GoM Miocene and younger** (H-OPEN-2 concurs).
- **Shale-salinity RHOma default 2.59 g/cc**, stated as a reasonable *dry matrix*
  density for smectite **and kaolinite** (`resistivity_to_pressure.htm`; verified
  verbatim at synthesis). Note the wet-vs-dry-clay convention question this
  raises for the SandiMin smectite review is *not* answered by the manual.
- **Alberty velocity-NCT constants A/B/C/D per clay type: never printed**
  (J-O-4). This blocks the flagship smectite/illite pore-pressure method.
- **Montmorillonite sigma endpoint: truncated in the only place it appears**
  (H-OPEN-1): the dropdown row reads `…12 Montmorillonite` with the leading
  digit(s) physically absent from the raster in **both** editions (pixel-diff
  identical). Neighbours: Kaolinite 14.12, Chlorite 24.87, Illite 17.58. Not
  guessed; resolve from the live IP 2025 Sigma dropdown or mineral files on disk.
- GoM-calibration caveat: nothing here is Mahakam-calibrated. Adoption for
  Mahakam smectitic shale requires local recalibration — flagged, not filled.

**R-3 — Monte-Carlo shift defaults for m, n, a, Rw (D-OPEN-4): NOT RESOLVED by
any slice.** D found only Gr Clean/Gr Clay (±10) on an image; the m/n/a/Rw Low/High
panels are on no page in the corpus. D's axis-extent inference (~0.25) remains
deliberately unadopted. Needs the live IP dialog or `MonteCarloDefaults.par`
(which O confirms is ordinal-addressed but never reproduced).

**R-4 — SandPit 3D `S1`/`S2` derivation: CONFIRMED GAP, both owners checked.**
G (who holds the SandPit/MDA pages, G-9.3) reports the transform from
ShMin/SHMax/Sv to cavity-wall stresses is never printed; J (who holds
rock_strength/rock_stress/wellbore_stability) found no SandPit derivation either.
L's OPEN-L-09 routing question is answered: the pages are owned (G + J), and the
equation genuinely is not in the manual. G §2.7's equation set is therefore not
implementable end-to-end from the CHM. (SandPit *defaults* are byte-stable
2018→2025 — G §7.)

**R-5 — Rv/Rh Butterfly & Thomas-Stieber crossplots (L-D-L-08 / L-OPEN-L-13):
owner found.** The plotting pages ship the UI checkboxes but no spec; the
computational specs live on B's pages (§2.7, laminated / thin-bed / tensor
resistivity). Vendor documentation gap on the plotting side only.

**R-6 — Excess pressure / supercharging equation (I-OPEN-7): NOT FOUND in the
corpus.** I's pages cross-reference the Crossplot docs; L's crossplot pages (§2.6
Pressure-Gradient crossplot) document the gradient regression but no
excess-pressure/supercharging definition. Treat as a vendor documentation gap.

**R-7 — psi/ft → g/cc gradient constant (I-OPEN-8): NOT FOUND.** Not on M's PL
Set-Up / Multiphase pages either (M's §3 constant census has no such entry). The
only related evidence in the corpus is the 0.433/0.43353 psi/ft fresh-water
gradient family (D-05). Do not back-derive the constant from I's example slopes.

**R-8 — Sigma endpoints (C-OPEN-10 → H): CLOSED.** H recovered the full
cased-hole sigma parameter set: fluids Water 80 / Hyd 20 / Clay 25 CU (dialog,
spot-verified at synthesis), lithology SigMat tied to ρma (Sand 4.3 @ 2.65,
Lime 7.1 @ 2.71, Dol 4.7 @ 2.85), a 21-mineral library, both ρma→SigMat
branches with clamps, and the SwTDTU inversion + reconstruction chain. Note the
5-vs-6 mineral-column mismatch (H-D-1) and the montmorillonite truncation (R-2)
remain open within that recovery.

**R-9 — Null conventions (F ↔ N, mutually corroborating).** F (Clean Data /
loaders): IP recognises −999, −999.25, −9999, −99 and canonicalises to −999;
Clean Data ships with −999.25→−999 enabled. N (writers): ASCII/LAS export default
−999 (editable); **LIS and DLIS are hard-coded −999.25 with no null field at
all**; ASCII *reader* default −999.00. Consequence stands as the single worst
interop trap: an IP-written LAS carries −999 while the LAS convention is −999.25.
SandiBumi: write −999.25 + explicit `NULL.` line; on read, honour the declared
null and flag the −999 family as suspected nulls (N §8.1).

**R-10 — Parameter ordinals (O ↔ B, D).** O proved the `(Parameter #N)` ordinals
are IP's stable cross-file handles (61/64 ClayVol + 27/27 PhiSw exact matches vs
the 2018 `.hlp` JSON; renames keep numbers; **#41 swapped which curve the clean
point belongs to** — verified at synthesis against both sources). Actions:
`H_module_parameter_reference.json` is **not superseded** (≈166 ordinals exist
only there); Cutoff ordinals *are* printed on `cutoffsandsummation.htm` but were
never hand-parsed (O-OPEN-3) — follow-up assigned to the D scope.

**R-11 — Greenberg–Castagna coefficients: one verified copy.** F's four-mineral
table (from `_rpclip0004.png`, MD5-identical across editions; spot-verified
digit-for-digit at synthesis) is the authoritative copy. I's laminated-page
G/C readings agree at 3 dp; I's OPEN-1/2 (Quartz b/c at 5 dp off-screen, Wet Clay
`a` partly obscured) remain open detail items. The **km/s unit lock** is printed
in the panel itself.

**R-12 — PL User Manual dependencies (M-OPEN-1/2/6): OUT OF CORPUS.** The gas
flowmap, the slippage charts, and the Reynolds-dependent Vmix/Vapp function live
in a separate "PL User Manual" that is not inside `Interact.chm`. No agent holds
it. IP's PL slippage is not reproducible from this ingest — recorded as a hard
boundary, not a to-do.

**R-13 — EERC reference list (F §7.6): 2018 record correction owed.**
`_eercclip0007.png` is byte-identical in both manuals and was simply never read
in 2018; all 10 entries are now captured. `ip2018_chm_ingest/F_envcorr_tierc_citations.md`
§F1.6 ("not recoverable") must be amended. Same class as D-12's correction: the
2018 "lost as raster" verdicts were extraction limits, not evidence of absence.

**R-14 — Stub / duplicate page census (for the coverage table's integrity).**
Content-free vendor stubs shipped in 2025: `bubble-analysis.htm`,
`terminal-events.htm` (M-OPEN-12), `3d-viewer.htm` (hyphen; the underscore twin
is the real page), `analysis-sticks.htm` (K-9.17), `mapping-resources.htm`
(L-D-L-11). Duplicate topic: `pltavailablereports.htm` / `plt-availablereports.htm`
(M-D-10). Title collisions routed correctly: four PL pages in O's bucket → M
(O §8.5); `managewellheaderinfo.htm` is a Fortran User-App example, not the
header module (O §8.6). All are **counted as read** in their assigned slices.

**R-15 — NeuroSolutions attribution scrubbed (G §7).** IP2018 disclosed the NN
engine ("NeuroSolutions 5.5 … Hidden layers = 1"); IP2025 removes the statement
entirely. The Tier-C register's NN entry now rests solely on the IP2018 source —
provenance recorded there.

---

## Part 3 — Per-slice internal discrepancies (index)

One line per item; the slice report is the authority and carries the evidence.
Counts: **165 slice-internal items** across 14 reports (B's 8 new items are in
Part 1 and not repeated here).

### C — Mineral Solver (9)
- C-5.1 Shell m 0.018 raster vs 0.019 ASCII, both editions → merged into **D-10**.
- C-5.2 Juhász/W&S raster carries ×Rw + `m*`; ASCII drops both → merged into **D-12**.
- C-5.3 Bound-water coefficient conventions: 0.15/−0.85 and φ/(1−φ)/−1 are consistent formulations; the ECS dry-clay grid's 0.15/−1 fits **neither** (implies φTclay 0.1304 ≠ 0.15). Flagged.
- C-5.4 Conductivity generalisation: `/a` on the water summation only, not conductive minerals. Confirmed as printed, no ASCII cross-check exists.
- C-5.5 Resistivity-confidence worked example: ± labels inverted. Unchanged since 2018.
- C-5.6 Neutron look-up outlier `-.1960` (almost certainly `-.0196`) byte-identical in 2025; the φ=.25 sand/100kppm non-monotonicity too. Not repaired.
- C-5.7 "Invasion factor" name collision: 0.5 (OBM) vs 2.0 (WBM Sxo empirical). Two parameters, one name.
- C-5.8 Two paragraphs printed twice verbatim in the 2025 source (cosmetic; noted so diffs don't read it as new content).
- C-5.9 "Vhyrocarbon" typo in a limit raster (cosmetic).

### D — Cut-offs, Summation & Monte Carlo (11)
- D-5.1 Average-Sw raster omits the `i` subscript on Sw (typesetting; intended Swᵢ).
- D-5.2 **Geometric average exponent is `1/Σhᵢ`, not `1/n`** — result is not unit-invariant (metric vs imperial runs differ). Transcribed as drawn; spot-verified at synthesis (embim163).
- D-5.3 Reservoir/Pay cut values: setup grid implies independent, Parameters window shows one shared `Res/Pay` column. → D-OPEN-2 (data-model question for SandiBumi).
- D-5.4 Input-curve capacity 50 vs 7: 2025 raised the headline number, left every downstream "7" and the 10-row screenshots unchanged. The parameter/results tabs only ever document 7.
- D-5.5 MC statistic curve naming: prose `XXX MN/PSD/MSD` vs dialog `_mn/_psd/_msd`.
- D-5.6 Output Percentiles 10/50/90 vs Result-Curve percentiles P5/P50/P95 in one module, never reconciled.
- D-5.7 REP3 titled "SW<0.45" ships with Sw cut 0.5 **unticked** (screenshot vs prose, value and applicability both).
- D-5.8 Dependency correlation prose 0.5 vs grid 0.8 (m↔n).
- D-5.9 "±10 % of the valid value" copy-paste defect: shipped input-curve shifts are RHOB ±0.02, DTLN ±2.0, LLD ±0.005, TNPH 5 %.
- D-5.10 `Sw Res Use` off by default — only inferable from the panel, never stated plainly.
- D-5.11 Multi-well worked report arithmetic does not reconcile (All-Wells rows vs member wells). Do not use as a conformance fixture.

### E — Sat-height, Pc & HFU (12)
- E-D1 RQI three-module collision → merged into **D-04**.
- E-D2 Pc↔Height without 0.433 in both function-module format panels (factor 2.31) → part of **D-05**.
- E-D3 **Lucia porosity units: prose "decimals" (×2) vs emitted file header "'Phi' in percent"** — many orders of magnitude at stake; blocking (E-OPEN-3).
- E-D4 σ/θ table row labels shifted in one of three screenshots (`_shmclip0167`); `_shmclip0009` authoritative (Mercury 140°/480; Centrifuge 0°/72; Porous Plate 0°/72 — spot-verified pre-compaction).
- E-D5 Stress/clay correction formulae labelled wetting-phase but algebraically non-wetting (`1−(1−Sw)·F`); pipeline order agrees with the algebra. Labels likely wrong; reported not resolved.
- E-D6 `Restore Defaults` scope: "first 3 columns" prose vs `Restore Lab Defaults` caption (4 columns?). Unresolved.
- E-D7 Clay-correction bracket imbalance → **D-07**.
- E-D8 "32 models" vs 8×7 dropdown arithmetic; the only consistent reading (4 methods don't cross the regressions) is an inference.
- E-D9 Implied internal gradient 0.43352 vs documented 0.433 → **D-05** sub-item.
- E-D10 "Normalised" (prose) vs "Normalized" (UI) — matters for string matching.
- E-D11 `Entry Correction` vs `Closure Correction Curve Out`: two names, one quantity.
- E-D12 Thomeer listed under both model types with different crossplot behaviour.

### F — QC, edit, corrections, TVD (15)
- F-1 Log QC extreme-low GR 117 > user-min GR 59 — one shipped panel wrong; extreme table likelier culprit.
- F-2 Three flag polarities in one QC/edit family (1–7 / 1,0,−999 / −999,1).
- F-3 Fill Data Gaps self-contradiction; dialog max-width default 5 makes "no limit" stale text.
- F-4 Filter length limits 1–121 vs 3–121 vs 2001.
- F-5 EERC Eq 3 divides by 100 while ASCII says G_g in °C/m (consistent only if °C/100 m) — OPEN.
- F-6 EERC "theoretical" rasters visually identical to "measured" ones; pair (4) duplicates pair (2) — OPEN.
- F-7 embim361 `/` vs embim360 `;` in parallel forms.
- F-8 `differentiate` computes a ratio in Fortran/C++/VB/C# but a true finite difference in MATLAB/Python — two operators, one module name.
- F-9 C# example: assignment-for-comparison bug (`=` for `==`).
- F-10 Normalize Array C++ example increments the wrong loop index.
- F-11 Restore-Backup filter suffix `df` vs mask `********BF`.
- F-12 Sperry-Sun chart-book cited 1998 and 1996.
- F-13 Baker Atlas 1984 chart book listed and simultaneously stated never received.
- F-14 Schlumberger CNL salinity default `2.8E-4 Kppm` (0.28 ppm — a conversion artefact) vs GE `0 kppm`. Transcribed as displayed.
- F-15 Stale "IP 2018" screenshot shipped in the 2025 manual.

### G — ML suite & user programming (13)
- G-6.1 NN "Epoch per pass": prose 1000 vs shipped panel 100 — factor 10 on the key hyperparameter; prose unchanged since 2018. OPEN.
- G-6.2 SOM λ = t/log σ₀ with `t` defined as *current* iteration — self-defeating as printed; blocking for reimplementation (G-9.1).
- G-6.3 SOM neighbour-update raster prints `+` where `=` is required (verified 6×); BMU equation on same page is correct.
- G-6.4 "Closeness of fit" = bin distance (Fuzzy) vs |difference| in curve units (NN). Same name, incomparable semantics.
- G-6.5 Fuzzy "weight bin by samples": prose default selected vs panel cleared; scoping unclear.
- G-6.6 User-app crossplot "five interactive lines" but only 1–3 documented.
- G-6.7 Menu-location conflicts across the ML reorganisation; PCA page contradicts itself.
- G-6.8 `mpmaths` vs `ipmaths` dependency name (likely `mpmath`).
- G-6.9 `@` called an "ampersand" on one page.
- G-6.10 Linkage method renamed between sibling pages (Minimum/Minimise).
- G-6.11 AppData folder cited as IntPetro41 / IntPetro47 / intpetro36 across pages.
- G-6.12 Fuzzy tab name prose vs UI; one screenshot doing double duty.
- G-6.13 SOM Input-tab text garbled mid-sentence (copy-editing).

### H — NMR, UCR, TOC, sigma (17)
- H-D-1 Sigma: five-mineral prose vs six-column dialog (both editions).
- H-D-2 Sigma Mineral-Vols branch: ASCII only, normalisation/porosity handling unstated.
- H-D-3 Montmorillonite sigma truncated → R-2 / H-OPEN-1.
- H-D-4 **No stated default for either T2 cut-off** — 90/3 ms is one demo well, not a vendor default.
- H-D-5 Same for perm a/b/c/d (10000/2/4/1 dialog-only).
- H-D-6 Coates tapered-cutoff cited "SPWLA 1977"; the 38th symposium was 1997.
- H-D-7 Polarisation rasters missing the bracket around (1 − e^…); ASCII on the same page correct.
- H-D-8 **TOC: prose claims wt %, the 14 regressions end `*.01` and return fractions.** Live unit trap; PhiSw Organic-Shale defaults expect wt %.
- H-D-9 UCR temperature-equation sign only consistent if depths are subsea elevations; convention never stated.
- H-D-10 UCR eq 14-3 subscript `O_vs` vs defined `O_sv` (typo).
- H-D-11 **GWR denominator omits C3+C4** — verbatim vendor text, both editions (spot-verified at synthesis). Check against Haworth (1985) before reimplementing.
- H-D-12 HCF 4/5 boundary undefined at exactly OCQ = 0.5 (both inequalities strict).
- H-D-13 UCR Langmuir range malformed: "(between 320,369)" — one number, five occurrences.
- H-D-14 UCR eq 13-10 described as mole-fraction weighted average, rendered as a harmonic sum.
- H-D-15 UCR eq 13-5 subscript inconsistency (`G_sga` vs `G_sag` family).
- H-D-16 UCR eq 5-7 bare `ρ` vs symbol list defining only `ρ_b`.
- H-D-17 TOC dialogs show example values contradicting stated defaults.

### I — Fluid substitution, thin-bed, FT (13; all present identically in 2018)
- I-i embim525/527 sign pair contradicts on inversion (one must be wrong; undeterminable from the manual).
- I-ii **304.8 printed where 304.8² is required** (K_b), 304.8 dropped entirely in E (embim584), while embim596 on the same page uses the consistent no-304.8 form. Spot-verified at synthesis (embim576).
- I-iii **embim590 K_fVoigt prints `+ K_HC` where `× K_HC` is required** (operator-detector verified; spot-verified at synthesis).
- I-iv embim607 G_HM lacks the ^(1/3) that K_HM carries (Hertz–Mindlin).
- I-v **Effective probe radius matches neither documented method**: Muskat (0.5·r_p) and Carslaw–Jaeger (2r_p/π) both *reduce*; shipped default is a ×3 multiplier (2 in → 6 in). Numeric reproduction proves the equation uses r_pe, not the labelled "snorkel radius". Mobility ~3–4.7× off either documented form.
- I-vi Parameter file named `FluidSub_Default_Parameters.par` (prose) vs `FluidSub_Default_Rocks.par` (dialog).
- I-vii Rock Physics Handbook cited 1995/1998/1999 on three pages.
- I-viii Author misspellings breaking traceability: "Carlson and Jaeger" (= Carslaw & Jaeger), Krieff/Krief, Glasso/Glaso, "Vasquez Beggs".
- I-ix `_rpclip0057` = `_rpclip0058` (same image presented as two diagrams).
- I-x v_p/v_s defined in km/s (embim577/578) then multiplied by 0.3048×10⁻³ as if ft/s (embim581–585); block header contradicts the definitions.
- I-xi **"1 Mpsi = 0.145038 GPa" is inverted** — correct is 1 GPa = 0.145038 Mpsi (1 Mpsi = 6.8948 GPa); wrong by ~47.5×. Reported as printed.
- I-xii Rho Lam Sh default 2.65 g/cc (quartz grain density) paired with DTc 110 µs/ft — odd for shale, reported as printed.
- I-xiii Radial FT worked example doesn't reproduce with the shown inputs (implied h ≈ 12 ft, not 2.5) — carried as OPEN, not asserted as defect.

### J — Geomechanics & PPFG (17)
- J-D1 **Geertsma uniaxial pore compressibility: dynamic branch `+`, static branch `×`, same named model.** 2018 had `×` in both — the `+` is a 2025 edit (§6.2). Highest severity; spot-verified at synthesis (rmnew_clip0082/0086). Do not copy either; derive from Geertsma and cite.
- J-D2 ESTA declared Mpsi while three UCS legends say psi (10⁶ risk).
- J-D3 Tectonic-strain Youngs parameter Mpsi vs equation legend psi.
- J-D4 SHMAXG_TS gradient defined self-referentially (doc only).
- J-D5 SHMING block header psi vs sub-model equations psi/ft.
- J-D6 Three sea-water constants coexist: 0.442 / 8.5 ppg (≈0.441) / 8.55 ppg.
- J-D7 Veeken TWC ends in 14.5 (bar→psi) while all others use 145.038 — byte-identical to 2018, deliberate; reimplement exactly as printed.
- J-D8 Lal friction angle uses 304878 (m/s) vs 304.8 elsewhere (0.026 %, a source-lineage tell).
- J-D9 Alberty NSC decay puts OC in the numerator, Traugott in the denominator, same prose description for both.
- J-D10 `ppfg_105` labelled FG but computes a pressure.
- J-D11 Zoback / MC-frictional breakouts print `+ αPp` where effective-stress convention subtracts — derive independently (J-O-12).
- J-D12 Fresh-water 0.434 psi/ft vs Breckels & Van Eekelen Ppn 0.45 psi/ft baselines.
- J-D13 ShMin strain symbols dεH/dεh vs ShMax εx/εy, the latter undefined.
- J-D14 **Eberhart-Phillips modified form fails its own conversion table by 10×** (1460 inside vs outside the bracket); resolve against the 1989 paper (J-O-11).
- J-D15 VTI prose labels both embim428/429 "DTSS"; rasters authoritative.
- J-D16 Poro-elastic legends copy thermo-elastic wording (prose typo).
- J-D17 One curve, three spellings (PRDYN_HV / PRDYN_VERVH / PRDYN_VER_VH).

### K — Geophysics & image analysis (10)
- K-6.1 Elastic impedance K: pointwise ratio in the equation, scalar constant in use; `EI_Kconst` purely diagnostic. Never stated in one place.
- K-6.2 **Backus impedances use undefined `V_{P,V}` / `V_{SH,V}`** — SH may be a genuinely different (horizontally-polarised) velocity in a TI medium; not safely dismissable as a typo (K-9.2).
- K-6.3 Backus: which ρ inside the radicals (ρ_b vs ρ_Backus) — printed order leaves it ambiguous.
- K-6.4 **Mean-dip back-transform is quadrant-degenerate**: `cos⁻¹(ȳ'/sin M)` returns 0–180° only, x̄' never used; western azimuths would mirror east. Spot-verified at synthesis (embim403). Singular at horizontal beds. Any implementation must use atan2 — IP's own must, despite its documentation.
- K-6.5 Poisson's-ratio string has an extra `)` (typographic).
- K-6.6 Net2Gross sand-side flag: configurable in prose, hard-coded in output definitions.
- K-6.7 Merged-area statistics are a mean of means (stated openly; not the pooled mean).
- K-6.8 Nav-QC inputs documented as "not currently used" yet exposed.
- K-6.9 Citation typos persisting across editions (Marzeta/Marzetta, Mo.3, Vo1.55).
- K-6.10 `PORMAP` is a histogram, not a map (page says so itself).

### L — Plotting, crossplots, histograms (11)
- L-D-L-01 Hingle self-contradiction → resolved into **D-02**.
- L-D-L-02 Lightness grey-scale `(MaxRGB + MaxRGB)/2` — reduces to Intensity; almost certainly MinRGB intended; reported as printed. Byte-identical in 2018.
- L-D-L-03 Histogram bin-class example can't tile [0,150] in 50 bins; crossplot bin-edge rule (`[low, high)`) is normative.
- L-D-L-04 Box plot: "Display Median" and "Display Mean" both described as plotting the mean. Decide and document; don't copy.
- L-D-L-05 Normalization constants named a/b in the equation, "A and M" in the interval discussion; no M defined.
- L-D-L-06 Install path stated three ways (IntPetro3x/36/…).
- L-D-L-07 Track border width default 3 vs 2 (different dialogs, unreconciled).
- L-D-L-08 Crossplot type list omits two shipped types (Rv/Rh Butterfly, Thomas-Stieber) → R-5.
- L-D-L-09 Ternary apex-value wording confusing but reconcilable.
- L-D-L-10 Out-of-range handling: clamp vs ignore vs NULL across three modules; a unified engine must choose per-context deliberately.
- L-D-L-11 `mapping-resources.htm` is an unwritten vendor page → R-14.

### M — Production logging & cased hole (11)
- M-D-1 Spinner discriminator ±0.1 rps prose vs ±0.50 dialog (5×; dialog may be a customised machine). Don't adopt either without a live check.
- M-D-2 Reynolds multiplier prose 1.0 vs worked-example 1.0300.
- M-D-3 Impedance worked example internally inconsistent at 0.14 % (row 2).
- M-D-4 Titan RIB simultaneously Radial, chart-less-by-rule, and psi-charted.
- M-D-5 **Cement colouring polarity: "above Good = green" inverts CBL-mV physics, and IP's two shipped radial grids order the cutoffs opposite ways.** Free pipe would grade green. Fix with an explicit per-tool polarity flag before writing any cement code (M-S-4).
- M-D-6 Micro-Annulus described as carved from Solid, drawn adjacent to Liquid; counted as Solid.
- M-D-7 `SecBond*` parameters are dB/m for Isolation Scanner but dB/ft for INTex — one name, no unit column, ~3.28× apart.
- M-D-8 Spinner diameter 1.25 vs 3.50 in across screenshots (in-line vs fullbore; recorded so it's not misread as a contradiction).
- M-D-9 Slug/churn deviation multiplier: prose omits the 0.5-factor second branch that exists only in the dialog raster. Implement from the raster.
- M-D-10 Duplicate PLT-reports pages → R-14.
- M-D-11 Conductivity-loss unit string malformed in the dialog; prose (0.25 degF/ft) is authoritative.

### N — Data I/O & loaders (10)
- N-6.1 ASCII load null "-999.00" prose vs "-999" panel (string-level only).
- N-6.2 Export nulls disagree across IP's own four writers → R-9.
- N-6.3 `IntPetro.config` vs `IntPetro.exe.config` (2 pages vs 1) — OPEN.
- N-6.4 Mask-file delimiter rules differ across three sources (DLIS text is the stale narrow one).
- N-6.5 "Extrapolate" used loosely for interior linear interpolation; only one page states the mechanism.
- N-6.6 `$ Geolog Depth Names` section exists only in a screenshot — a depth-recognition list documented nowhere in prose.
- N-6.7 DLIS "use Source to create Curve Sets" control exists in the panel, zero mentions in prose.
- N-6.8 Fill-Gaps "5": hard limit in LAS3, soft default elsewhere.
- N-6.9 Petrel set-name limit text still enumerates IP 3.4–4.0, byte-identical to 2018.
- N-6.10 "(Powerlog)" boilerplate cloned onto GEOLOG/OpenWorks/PETCOM pages — signals cloned docs; calibrate trust accordingly.

### O — Database, config, infrastructure (8)
- O-8.1 Curve Sets per well 500 vs 50 (two pages + dedicated limits list vs one) — treat 500 as correct, 50 stale; residual doubt in O-OPEN-2.
- O-8.2 Curve Set short name 8 vs 4 chars (+"must not start with a number" appearing once) — OPEN.
- O-8.3 Lithology shadings max "39" vs "80" in one page; 39 is the shipped bitmap count leaking into a limit statement. In 2018 the same defect read "30" vs 80 — bumped once, never reconciled.
- O-8.4 Two Parameter-Set type enumerations, neither a superset (Splice vs TVDss_Set/MonteCarlo). Identical divergence in 2018.
- O-8.5 Four PL pages in this bucket by title collision (routed to M) → R-14.
- O-8.6 `managewellheaderinfo.htm` is a Fortran User-App example, not the module (real page: `wellheaderinfo.htm`).
- O-8.7 `toolbox.htm` / `well-queries.htm` document the separately-licensed IC Mapping surface, not IP core.
- O-8.8 Multi-well tops paste silently ignores unmatched wells — **documented** silent failure ("no error message is given").

---

## Part 4 — OPEN items roll-up

**~172 OPEN items** across the 14 reports (B 14 · C 14 · D 10 · E 13 · F 8 ·
G 11 · H 11 · I 14 · J 13 · K 17 · L 13 · M 13 · N 12 · O 9). Every one is a
refusal to guess, catalogued in the report's final section. The **blocking**
subset — items that gate a SandiBumi implementation and need an external source
or a live IP session:

| # | Item | Needs | Ref |
|---|---|---|---|
| 1 | D-07 clay-bound-water `F` bracketing | SPWLA 20th Paper AA (Hill/Shirley/Klein 1979) or rendered live help | B/E |
| 2 | D-10 Shell `m` 0.018 vs 0.019 | Jauhar's decision against the published Shell source | B/C |
| 3 | Lucia porosity decimals vs percent (+ Class-2 `H^(-0407)` missing decimal) | Lucia 2007 / SPE 84942 | E-OPEN-2/3 |
| 4 | Alberty velocity-NCT constants A/B/C/D (smectite & illite) | Alberty & McLean source paper; blocks the flagship PP method | J-O-4 |
| 5 | Montmorillonite sigma endpoint (truncated raster) | live IP 2025 Sigma dropdown or mineral files on disk | H-OPEN-1 |
| 6 | MC shift defaults for m/n/a/Rw | live IP dialog or `MonteCarloDefaults.par` | D-OPEN-4 / R-3 |
| 7 | SOM λ definition (`t` current vs total) | Kohonen source or product test | G-9.1 |
| 8 | SandPit S1/S2 stress transform | not in manual; independent derivation | G-9.3 / R-4 |
| 9 | Backus `V_{P,V}` / `V_{SH,V}` + radical ρ | Backus (1962) | K-9.2/9.5 |
| 10 | Terzaghi weighting factor; Luthi & Souhaite aperture equation | the cited 1990 papers | K-9.3/9.4 |
| 11 | Reynolds Vmix/Vapp function; slippage charts; gas flowmap | the separate PL User Manual (out of corpus) | M-OPEN-1/2/6 / R-12 |
| 12 | TOC ΔlogR equations & constants | Passey et al. 1990 AAPG | H-OPEN-4 |
| 13 | CEC / Qv endpoint units (meq/mL trap) | not stated anywhere; do not assume | C-OPEN-2/3 |
| 14 | embim525/527 sign; whether IP's code squares the 304.8 | numeric test against live IP output | I-OPEN-11/12 |
| 15 | Eberhart-Phillips 1460 placement | Eberhart-Phillips, Han & Zoback 1989 | J-O-11 |
| 16 | Effective-stress sign in breakout inversions | independent derivation | J-O-12 |
| 17 | GWR denominator (C3+C4 omission) before any reimplementation | Haworth 1985 | H-D-11 |
| 18 | Kf coefficient sets ×6, M&K Ki tables, Daines PRs, Hoek-Brown weak-rock defaults | vendor `.par` files on a live install (values not in manual) | J-O-5/6/7/8 |
| 19 | Unit-alias / curve-default tables (`DefaultAlias.cax`, `UnitsConversion.par`, `CparmDef.xml`, `SetDictionary.xml`) + fixed attribute name lists | installed IP 2025 files (`C:\Program Files\IP2025`, read-only) | N-9.1 / O-OPEN-7/8 |
| 20 | Cutoff-module parameter ordinals | hand-parse of `cutoffsandsummation.htm` (assigned to D scope) | O-OPEN-3 / R-10 |

Everything else in the OPEN sections is either detail-level (unread low-yield
image tails, off-screen dialog cells) or explicitly out of scope (Tier-C
internals, vendor chart data, credentials — see FINDINGS §9).
