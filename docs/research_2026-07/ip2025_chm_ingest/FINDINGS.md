# IP 2025 CHM ingest — master findings

**Commission** (Jauhar, 2026-08-05): ingest *everything* from the IP 2025 help —
every equation, constant, default, assumption, and constraint — to complete the
SandiBumi discrepancies modules and make them better than IP; produce a detailed
reviewable record. **Status: complete.** 343/343 topic pages read; 14 slice
reports + this synthesis + [DISCREPANCIES.md](DISCREPANCIES.md) on disk.

Corpus: `Interact.chm` build 13-Mar-2025 (245,658,511 bytes), decompiled to
`%LOCALAPPDATA%\Temp\c25\` (6,566 files); IP 2018 decompile at `\c18\` used for
differential reads throughout. A copy of the CHM itself is archived at
`D:\01. Work\00. Guidebook\08. Guidebook IP\IP 2025 Help\Interact.chm`. No CHM
content was copied into the SandiBumi repo — only these derived research notes.

---

## 1. Executive summary

1. **The manual, not the product, is the weak layer.** Across 343 pages the
   ingest logged ~165 slice-internal discrepancies and 15 global ones — but
   nearly all are *documentation* defects (typos in rasters, stale prose,
   unit-label drift, unprinted derivations), most byte-identical since 2018.
   The product's numeric core barely moved.
2. **Genuine 2018→2025 numeric drift is tiny** — about ten real changes in the
   whole corpus (§4). Eight slices measured *zero* drift on their pages. The one
   dangerous edit: Geertsma dynamic pore compressibility now prints `+` where
   2018 printed `×` (J-D1) — a regression introduced by the 2025 edition.
3. **Five 2018 ledger items closed, two hardened.** Hingle (D-02) resolved
   twice-independently; D-06 the vendor fixed; D-08/D-11/D-12/D-13/D-14 resolved
   internally. D-07 (CBW brackets) and D-10 (Shell m) are now proven vendor-source
   defects present in both editions — external adjudication required.
4. **The recovered-content haul is large** (§7): the six Cutoff averaging
   equations, the full Greenberg–Castagna table, the Alberty density relations
   with the Katahara tanh blend, the complete sigma parameter system, the EERC
   reference list, Swanson/Normalised-J, the lab→reservoir Pc chain, casing
   metal-loss — all things the 2018 ingest recorded as lost or never held.
5. **A concrete "better than IP" defect catalogue now exists** (§6): fifteen
   classes of error SandiBumi can simply not have, each traceable to an IP page.
6. **~172 items stay OPEN by policy** — every one is a refusal to guess a
   parameter or bracket. The 20 blocking ones, with what each needs, are in
   DISCREPANCIES.md Part 4. Five need only a live IP 2025 session or a read of
   files already on this machine.

---

## 2. Method and verification

**Slicing.** 14 domain slices (B–O), each an opus subagent with an explicit page
list; assignment script-enforced so every one of the 343 `*_text.txt` pages was
owned by exactly one slice (`00_INGEST_PLAN.md`, `manifest.csv`). Delegation per
the machine ladder: extraction is domain-aware but independently checkable →
opus; synthesis (this document, the ledger, all resolutions) stayed on the
session model and was never delegated.

**Provenance protocol.** ASCII math quoted verbatim from the page text. Every
equation raster transcribed by direct vision-read, tagged `[img-read: file.png]`.
Ambiguous glyphs → OPEN items, never inferred. Numbers appearing only in demo
screenshots are labelled *example values, not defaults* (H's T2 finding is the
canonical case). Differential claims ("unchanged since 2018") rest on byte/MD5
comparison of c25 vs c18 sources, not on memory.

**Main-agent verification.** Every returned report was spot-checked by
personally re-reading cited sources before acceptance. All checks passed; zero
mismatches across 14 reports:

| Slice | Spot-checks (all PASS) |
|---|---|
| B | embim115 (Juhász ×Rw), embim118 (W&S B bracket), embim120 (Shell m 0.018), embim71 (shale φ limit) |
| C | `_imsclip0092` bound-water grid 0.15/−1 |
| D | embim163 (geometric average `1/Σhᵢ`) |
| E | `_shmclip0009` σ/θ table (140°/480; 0°/72; 0°/72) |
| F | `_rpclip0004` GC coefficients digit-for-digit + km/s panel lock |
| G | SOM raster `+`-for-`=` (6 occurrences) |
| H | `_chclip0004` sigma 80/20/25; `gas_analysis_text.txt` GWR verbatim; `H_module_parameter_reference.json` #39–41 |
| I | embim576 (un-squared 304.8), embim590 (`S_HC + K_HC`) |
| J | rmnew_clip0072 c25 10⁴ vs c18 10¹⁰; rmnew_clip0082 `CP_DYN_ISO +` vs rmnew_clip0086 `CP_STA_ISO ×`; `resistivity_to_pressure_text.txt` smectite ρma 2.59 |
| K | embim403 (mean-dip cos⁻¹ back-transform) |
| L | Hingle curve-definition grep (Rt^(−1/m)) |
| M | `_ciclip0019` casing joints/grading colours |
| N | `_dsaclip0038` LAS export panel (−999, 4 dp, 2.0/3.0) |
| O | `clayparameters_text.txt` `(39) OD Curv1 Clay` + #41 swap vs 2018 JSON |

Two harness notes for future runs: agent F's `.output` file came back empty and
N's exceeded the read limit — in both cases the full report was in the task
notification itself; and one agent (C) died on a transient API error mid-run and
was resumed by ID with context intact.

---

## 3. Coverage — 343/343

| Slice | Domain | Pages | Report |
|---|---|---:|---|
| B | Core petrophysics (clay, φ, Sw, laminated/thin-bed) | 19 | `B_core_petro.md` |
| C | Mineral Solver / optimisation | 6 | `C_mineral_solver.md` |
| D | Cut-offs, Summation, Monte Carlo | 8 | `D_cutoffs_montecarlo.md` |
| E | Sat-height, Pc, HFU | 7 | `E_sat_height_hfu.md` |
| F | QC, editing, env-corrections, TVD | 36 | `F_qc_edit_corrections.md` |
| G | ML suite, user programming | 28 | `G_ml_userprog.md` |
| H | NMR, UCR, TOC, sigma | 8 | `H_nmr_ucr_toc_sigma.md` |
| I | Fluid-sub, thin-bed, formation testing | 9 | `I_fluidsub_thinbed_ft.md` |
| J | Geomechanics, PPFG | 22 | `J_geomechanics.md` |
| K | Geophysics, image analysis | 20 | `K_geophys_image.md` |
| L | Plotting, crossplots, histograms | 46 | `L_plotting_viz.md` |
| M | Production logging, cased hole | 29 | `M_production_logging.md` |
| N | Data I/O, loaders/writers | 34 | `N_data_io.md` |
| O | Database, config, infrastructure | 71 | `O_db_config_infra.md` |
| | **Total** | **343** | |

Assigned = read for every slice. Known stubs/duplicates counted as read and
catalogued (DISCREPANCIES R-14): two PL stubs + one PLT duplicate (M), the
`3d-viewer` hyphen stub and `analysis-sticks` (K), `mapping-resources` (L), four
PL title-collisions and the Fortran-example page routed correctly (O).

---

## 4. 2018 → 2025 drift map

The complete list of *real* differences found by differential read. Everything
not listed here was byte-identical or cosmetically re-skinned.

**Equation / semantic changes**

| Where | 2018 | 2025 | Verdict |
|---|---|---|---|
| Dynamic moduli EC/G/K (J) | constant 1.34747×10¹⁰ | 1.34747×10⁴ | Unit-system change (psi→Mpsi outputs). Both verified at pixel level. Implement with explicit unit types, not a magic constant. |
| Geertsma dynamic uniaxial (J-D1) | `×` | `+` | **2025 regression.** Static branch kept `×`. Additive form is dimensionally/physically wrong for the named model. |
| ESTA static Young's (J) | `A·EDYN^B` | `A·EDYN^B + C` | Compatible extension (C=0 reproduces 2018). |
| Closure correction (E §6.2) | `SwCorr = Sw + SwClosure`; SwClosure *is the correction* | `SwCorr = Sw + (1 − SwClosure)`; SwClosure = *Sw at closure* | **Semantic flip.** A curve carried between versions stores a different quantity. 1 method → 4 (Shift default, Proportional, Crop, Extrapolate). |
| Z-factor irreducible-Sw prose (H) | 0.2–0.5 | 1.5–1.8 | 2025 fix of a 2018 misprint (dialog was always 1.5–1.8). |
| Throat-size array text (E) | 0.1 µm (contradicting its own table) | 0.01 µm | Vendor fixed → closes D-06 at source. |

**Capacity / default changes**

| Where | 2018 | 2025 |
|---|---|---|
| Cutoff input curves (D) | 10 | 50 (stale "7" text remains downstream) |
| Fuzzy / NN-predict / PCA input curves (G) | 8 | 20 (SOM and Cluster stay 8) |
| SOM Default Zone Size (G) | 5 | 20 |
| Mineral Solver user models (O) | 20 | 50 |
| ClayVol parameter count (O) | 70 ordinals | 72; #39–41 renamed, **#41 changed which curve it points at** |
| Curve Sets per well (O) | (500-vs-50 conflict already present) | unchanged conflict; lithology stale number bumped 30→39 |
| Curves per well (N) | — | 20,000 now stated |

**Removals / attribution**

- NeuroSolutions 5.5 + "Hidden layers = 1" disclosure removed from the NN pages
  (G §7). Tier-C register provenance for the NN engine now rests solely on the
  IP2018 extract — recorded there.

**New in 2025 (documented, ingested)**: Experienced Eye pages (capability-only —
Tier C), the reorganised ML menu, Swanson permeability (constants
1.691/1.901/2.005/2.109 by unit system), Normalised-J workflow, Gas-Free-Water
Level, CO₂-storage workflow pages (D/H), Casing Inspection + advanced cement
(M), curve auto-edit / log-QC / curve-mask / caliper-QC family + GE environment
corrections (F), image-corrections and PORMAP/Power-Law pages (K), the 16-page
PPFG toolbox restructure (J), WITSML / Kingdom / Petrolog / ZIP-archive I/O (N),
multi-user & global-parameter infrastructure (O), Petrel export (E/N).

**Zero-drift slices** (explicitly measured, not assumed): C, F (no constant
changed anywhere in 36 pages), I (all rasters byte-identical incl. the 13 FT
pages), K, M (PL engine identical), N (formats static), D's MC defaults, G's
SandPit defaults, L's plotting constants.

---

## 5. Per-slice digests

One paragraph each; the reports carry the full extraction.

- **B — Core petrophysics.** The load-bearing slice: all Vcl single/dual-curve
  methods, φ from D/N/S with hydrocarbon and excavation branches, the full Sw
  method family (Archie→Juhász→W&S→dual-water→SSM), laminated/tensor Rv/Rh, and
  the appendix nomenclature. Extended the global ledger D-08…D-15; carries the
  evidence for D-02's closure and D-07/D-10's both-editions status.
- **C — Mineral Solver.** Complete inversion spec: response equations with the
  `/a` placement quirk, uncertainty weighting, the bound-water coefficient
  three-way (C-5.3), Shell-m and Juhász splits that became D-10/D-12. CEC/Qv
  units nowhere stated (the meq/mL trap from the W&S memory applies).
- **D — Cutoffs & Monte Carlo.** Six averaging equations recovered (2018 gap);
  geometric-average unit-variance defect; MC machinery fully specced (4σ span,
  ±2.5σ truncation, 200 burn-in, 300 min, dialog default 2000); m/n/a/Rw shift
  defaults remain the one hole (R-3).
- **E — Sat-height / Pc / HFU.** Full lab→reservoir chain (closure → stress →
  clay → σcosθ conversion → height), 8 curve-fit families, HFU/FZI, Lucia
  (blocked on units), Swanson & Normalised-J new. Densest per-page discrepancy
  yield; owns the closure-correction semantic flip.
- **F — QC / corrections / TVD.** 36 pages, no numeric drift. The MD5-raster-
  hash proof technique originates here. EERC equations + reference list fully
  recovered; vendor chart-book digitisations identified as licensed data and not
  transcribed. GR-flag, polarity, and null-canonicalisation findings feed §6.
- **G — ML & user programming.** Every trainable method's update equations
  transcribed (with the SOM defects); the four-language user-program API; the
  Epoch 1000-vs-100 conflict; SandPit S1/S2 gap; NeuroSolutions scrub.
- **H — NMR / UCR / TOC / sigma.** The negative result that matters: **no
  vendor defaults exist** for T2 cutoffs or NMR-perm coefficients — demo values
  only. Full sigma system recovered (closes C-OPEN-10); TOC wt%-vs-fraction
  trap; montmorillonite endpoint physically truncated in both editions.
- **I — Fluid-sub / thin-bed / FT.** Gassmann chain with three raster defects
  (304.8², `+`-for-`×`, missing ^(1/3)) all byte-identical to 2018; the
  probe-radius ×3 anomaly (matches neither documented method); the inverted
  Mpsi/GPa conversion; 13 FT pages static.
- **J — Geomechanics / PPFG.** Largest defect count (17): the Geertsma
  regression, Eberhart-Phillips 10× ambiguity, the 145.038/14.5/304878 constant
  archaeology, three seawater gradients. Alberty density-side smectite/illite
  relations fully recovered; velocity-side NCT constants never printed (J-O-4).
- **K — Geophysics / image.** Elastic impedance, Backus (with the V_SH
  ambiguity), full image-analysis chain; the quadrant-degenerate mean-dip
  back-transform; Terzaghi factor and Luthi & Souhaite aperture equation
  confirmed unprinted.
- **L — Plotting / crossplots.** 46 pages of presentation-layer spec: bin-edge
  rules, normalisation, ternary, box-plot ambiguity, the Lightness formula
  defect; independent Hingle closure; out-of-range policy divergence that a
  unified engine must decide deliberately.
- **M — Production logging / cased hole.** Full PL surface: spinner calibration
  (the over-determined joint-47 derivation pinning the denominator), Reynolds
  multiplier, the cement-polarity inversion, casing metal-loss equation
  recovered, radial-grid conventions. Slippage physics confirmed out-of-corpus
  (PL User Manual).
- **N — Data I/O.** Reader/writer matrix for LAS/LIS/DLIS/ASCII/Geolog/Petrel/
  WITSML+; the null three-way and its LAS-convention collision; mask-file
  delimiter drift; the screenshot-only Geolog depth-names list.
- **O — Database / config / infra.** 71 pages: parameter-set ordinal system
  proven stable (and #41's swap caught), limits census (500/50, 8/4, 39/80),
  multi-user/global-parameter model, licence rebrand, the documented
  silent-failure paste. Credentials on the LiMBR pages deliberately not
  transcribed.

---

## 6. "Better than IP" — the defect catalogue SandiBumi must not reproduce

Each class is traceable to an IP page; each is a design rule for the
discrepancies modules and the compute core.

1. **No raster-only truth.** IP's worst defects live where the equation exists
   only as an image and the ASCII drifted (D-07, D-10, D-12, I-ii/iii, G-6.3).
   SandiBumi: every equation in one canonical machine-readable form, rendered
   views generated from it.
2. **atan2, never cos⁻¹** for any angle back-transform (K-6.4's western-azimuth
   mirror). Same rule for any quadrant-bearing inverse.
3. **Unit-typed quantities, no magic constants.** The 1.34747×10⁴/10¹⁰ pair,
   the 145.038-vs-14.5-vs-304878 archaeology (J), the inverted Mpsi/GPa print
   (I-xi), SecBond dB/m-vs-dB/ft (M-D-7): carry units in the type system;
   conversions are named, tested functions.
4. **Unit-invariant statistics.** The geometric average must exponentiate by
   `1/n` over *thickness-weighted* logs done correctly, not `1/Σhᵢ` (D-5.2) —
   metric and imperial runs of the same data must agree to machine precision;
   make that a regression test.
5. **One flag convention.** IP ships at least three polarities in one module
   family (F-2) plus an inverted cement grading (M-D-5). SandiBumi: a single
   documented flag scheme + explicit per-tool polarity parameters where physics
   varies.
6. **Null discipline.** Write −999.25 with an explicit `NULL.` line; on read,
   honour the declared null, then screen the −999/−9999/−99 family as suspected
   undeclared nulls (R-9). Never hard-code a writer's null without exposing it.
7. **Ordinal + semantic-name parameter addressing.** IP's #41 kept its number,
   changed its name *and* its referent (R-10). SandiBumi parameter files carry
   both an ordinal and a semantic key; a mismatch is a load error, not a silent
   remap.
8. **No bare `SW`.** Emit `SWE`/`SWT` only (D-15). Same for any symbol IP uses
   in two senses (RQI — keep it namespaced per module, D-04).
9. **Defaults are cited or absent.** H proved IP's own dialogs show demo values
   indistinguishable from defaults. Every SandiBumi default carries a source
   string; "no default — user must set" is a first-class state.
10. **Docs generated from code.** IP's stale-text class (7-vs-50 curves,
    2018 screenshots in the 2025 manual, prose 1000 vs shipped 100) is what
    hand-maintained docs decay into. Parameter tables and capacity limits in
    SandiBumi docs are emitted from the source of truth.
11. **Worked examples must reproduce.** IP ships examples that fail their own
    arithmetic (D-5.11, I-xiii, M-D-3). Every SandiBumi doc example is an
    executed test fixture.
12. **Per-correlation unit flags.** Veeken's 14.5 among the 145.038s (J-D7)
    shows correlations keep their native unit systems. Store each correlation
    with its native-unit metadata and convert at the boundary — do not
    "normalise" published constants.
13. **State the reference convention.** UCR's sign flip only works for subsea
    elevations (H-D-9); IP never says so. Every depth/pressure quantity in
    SandiBumi declares its datum explicitly.
14. **Silent failures are bugs.** IP documents that multi-well paste ignores
    unmatched wells without a message (O-8.8). SandiBumi surfaces every skipped
    row.
15. **Curve resolution & depth snapping are logged decisions**, not implicit
    behaviour (N's interpolation-called-extrapolation, screenshot-only depth
    lists).

---

## 7. Recovered-content scorecard

Content the 2018 ingest recorded as lost, absent, or never attempted — now held:

| Item | Where recovered | 2018 status |
|---|---|---|
| Six Cutoff averaging equations | D §2 | "unrecoverable raster" |
| Greenberg–Castagna 4-mineral a/b/c table + km/s lock | F (`_rpclip0004`) | not extracted |
| EERC reference list (10 entries) | F §7.6 | "not recoverable" — 2018 §F1.6 to amend |
| W&S `B(T,Rw)` balanced form | B (SSM page) → D-08 | not captured |
| Full sigma parameter system (fluids/lithology/21 minerals/SwTDTU) | H | C-OPEN-10 gap |
| Alberty ρ-smectite/illite + Katahara tanh (160/220 °F) | J | absent |
| Casing metal-loss equation + joint grading | M | never held (new module) |
| Lab→reservoir Pc chain incl. σcosθ table | E | partial |
| Swanson (1.691/1.901/2.005/2.109), Normalised-J, GFWL | E | didn't exist |
| Kozeny-Carman / Pittman-14 / Winland constants | E/D | partial |
| MC dialog default 2000 iterations (shipped, not advice) | D | open question |
| Cutoff parameter ordinals printed in-page | O → D scope | unknown |

---

## 8. Corrections owed to the 2018 record

Filed as a dated addendum in `..\ip2018_chm_ingest\ADDENDUM_2026-08-06_ip2025_crosscheck.md`
(the 2018 reports themselves are left untouched as a preserved record):

1. **F §F1.6** — EERC reference list *is* recoverable; `_eercclip0007.png` was
   byte-identical in 2018 and simply unread. Full list now in F §7.6 (2025).
2. **C / Juhász–W&S** — the 2018 report transcribed the ASCII (no `×Rw`,
   no `m*`) form; it is dimensionally wrong. Correct raster form in D-12.
3. **D / averaging + MC** — "six equations unrecoverable" and "MC iteration
   default unknown" both closed (D §2, §6).
4. **G / NN provenance** — the NeuroSolutions 5.5 disclosure exists *only* in
   IP2018; IP2025 removed it. The Tier-C register's citation must point at the
   2018 extract.

---

## 9. Compliance statement

- **Tier C untouched.** Experienced Eye, EEFS, DTA, entropy speed-correction,
  NN weight DLLs, Textural Facies `Freq_Tiles`, the frequency-domain
  dispersion-correction fit (and K-9.8's low-frequency fit, flagged as likely
  the same function): documented as *capabilities only*; no implementation,
  approximation, or reverse-engineering anywhere in the 14 reports.
- **Licensed data not transcribed.** Vendor chart-book digitisations
  (Sch/Hal/Baker/Weatherford/Sperry-Sun/PathFinder/Anadrill/GE) and EERC
  digitized curve families: identified, cited, values not copied. No `.itt`/
  `.itp`/`.att`/`.bor`/`.eli`/`.neu`/`.ovl` content reproduced.
- **No credentials.** LiMBR admin password / API key visible in the O pages
  were deliberately not transcribed.
- **No vendor files in the repo.** The SandiBumi tree holds only these research
  notes; the CHM and its decompile live in Temp/Guidebook.
- **No invented parameters.** Every default in the reports carries a page-level
  citation; everything uncitable is OPEN. No client well names appear.

---

## 10. Asks for Jauhar (review checklist)

Ordered by leverage:

1. **D-10 decision** — Shell variable-m 0.018 vs 0.019 (published Shell source;
   ~5–10 % Sw in tight rock rides on it).
2. **SPWLA Paper AA** (Hill, Shirley & Klein 1979) — the only clean resolution
   for D-07's clay-bound-water brackets.
3. **A live IP 2025 session** (or the installed tree at `C:\Program Files\IP2025`,
   read-only) closes five blockers cheaply: montmorillonite sigma (H-OPEN-1),
   MC m/n/a/Rw shifts (D-OPEN-4), the T2/perm "defaults" question (H),
   embim525/527 sign + 304.8² behaviour (I), spinner discriminator (M-D-1).
   On-disk file reads worth commissioning: `MINDEF.PAR`, `Overlay_Files.ovlx`,
   `CparmDef.xml`, `Intpetro.config`, `MonteCarloDefaults.par`,
   `DefaultAlias.cax`, `UnitsConversion.par`, `SetDictionary.xml`.
4. **Source papers** for the remaining method blockers: Lucia 2007/SPE 84942
   (E-OPEN-3), Alberty & McLean NCT constants (J-O-4), Eberhart-Phillips 1989
   (J-O-11), Passey 1990 (H-OPEN-4), Haworth 1985 (H-D-11), Backus 1962
   (K-9.2), the two 1990 image papers (K-9.3/9.4), Kohonen (G-9.1).
5. **Low-yield tails** if ever needed: O's unread `_tclip0115+` attribute-table
   images; the Cutoff-ordinal hand-parse (O-OPEN-3).

Older still-open items not part of this commission: SPWLA-2021-0091 (EE paper),
the four-way mineral-endpoint file diff, McDonald & Brackenridge.
