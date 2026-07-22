# Techlog 2018.2 ingest — FINDINGS

Ingested `D:\01. Work\00. Guidebook\03. Guidebooks Techlog\Techlog 2018.2 (r22885)` on
2026-07-22 for SandiBumi feature mining. Source treated read-only; all outputs in this folder.
Plan of record: `docs/techlog_ingest_prompt.md`. Calibration bar: the Geolog-V14 ingest
(`reference_geolog_v14_install_anatomy` memory).

**IP note:** catalog/endpoint/chart DATA extracted verbatim (it's reference data). Algorithm
sources (target D) were read only to identify the published equation each implements —
SandiBumi reimplements from the cited publication, never from Techlog code.

Extraction is deterministic (a Python/lxml pass, `scratchpad/extract_techlog.py`) for the
structured targets A/B/C/E/F/G/I; targets D and H were read by subagents (see
`D_python_algorithms.md`, `H_doc_index.md`).

---

## 0. Headline

The tree is a **genuine, mostly-readable install** — far more mineable than feared. The
biggest wins:

1. **Family/alias catalogs (A)** — 2,181 curve families + 2,839 assignment rules + 723
   mnemonic aliases, all clean XML. This is a drop-in upgrade for SandiBumi's import
   auto-aliasing, an order of magnitude larger than Geolog's `alias.alias`.
2. **QM_MineralTable (E)** — a full **31-mineral × 20-property** endpoint table. An
   *independent* second source to cross-check SandiMin's endpoints (which came from AspenTech
   Multimin). Result: matrix densities agree; clay/Sigma/GR endpoints diverge (expected).
3. **112 parametric vendor charts (B)** — machine-readable neutron-density / neutron-sonic /
   rock-physics crossplots (Halliburton, Baker Hughes, Gearhart, GE). **Directly overlaps the
   hand-digitized chartbook effort (`tools/chartdig`)** and can retire much of it for vendor
   charts.
4. **Quanti.Elan theory chapter (H)** — a numbered-equation derivation of the multi-mineral
   solver (incoherence Eq 79/80, ELAN Simandoux conductivity Eq 78, Dual-Water, Indo/Nigeria)
   — an audit-grade, citable basis for SandiMin that no textbook states as cleanly.
5. **`RockPhyEquations.py` (D)** — Batzle-Wang 1992 fluids, Gardner, Faust, mixing laws,
   fully readable with constants; the seed for a SandiBumi rock-physics/fluid-substitution
   module. (Counterweight: EnvCorr/Acoustics math is compiled — taxonomy only.)

---

## 1. Per-target results

### A — Family / alias / unit catalogs  ✅ high value, complete
| Artifact | Count | Output |
|---|---|---|
| Curve families (taxonomy, unit, scale, min/max, color, 2D type) | **2,181** across 54 main families | `A_families.json` |
| Family→variable assignment rules (wildcards, vendor context, unit) | **2,839** | `A_family_assignment.json` |
| Mnemonic aliases (Techlog 379 + Techcore 344) | **723** | `A_mnemonic_alias.json` |
| Unit systems (Canadian/English/Default/Metric/Russian/Undefined) | 6 systems, 60–97 units each | `A_unit_systems.json` |
| Variable prefixes/suffixes | empty in this install (2-byte files) | — |

Main families include the full FE set: Gamma Ray, Neutron, Density, Resistivity, Sigma,
Photoelectric Factor, Porosity (**70 sub-families**), Saturation (**37**), Permeability
(**26**), Salinity, Spectroscopy, NMR, Geomechanics, Core Analysis Routine/Special. Every
family carries default unit, display scale (lin/log), value range, and logview color —
enough to seed both auto-aliasing *and* sensible default plot styling.

### B — Digitized vendor charts  ✅ high value, complete
112 XML charts, **all 112 parsed** (105 typed `CrossPlot`). **74** are
density/neutron/sonic crossplots. Vendors: Halliburton (9), Baker Hughes (5), Gearhart (2),
GE (1), plus large generic + rock-physics suites (IsisRockPhysics, RockPhysics_* modulus/
velocity vs porosity families, Bowers). Format is **parametric**: each chart carries x/y
parametric equations (e.g. `x = (t/100)*dtf + (1-t/100)*dtma`), matrix/fluid endpoint
constants per lithology line, and grid definitions — i.e. reconstructable analytically, not
just as sampled points. `TechlogMnemonicChartAutoSelect.xml` maps mnemonics→chart.
- Catalog: `B_charts_catalog.json`; 6 full sample XMLs (schema ref): `B_chart_samples.json`;
  autoselect rules: `B_chart_autoselect.xml.txt`.

### C — Quanti workflows & parameters  ✅ complete (2-pass)
Pass 1: 18 Quanti workflow *container* XMLs (`C_quanti_workflows.json`, 3 full samples in
`C_quanti_samples.json`) — layout/validator config + pointers to parameter files.
Pass 2 (gap closed): the numeric petrophysical defaults live in `ProjectParameters\*_PR.xml`
— **the direct Geolog `.info`-manifest equivalent**, flat `parameter Name/Value/Unit` pairs,
**131–140 defaults per workflow** (`C2_method_defaults.json`). Extracted values for the
neutron-density effective workflow include:

| Param | Value | | Param | Value |
|---|---|---|---|---|
| Vsh min / max | 0 / 0.5 | | RHOB matrix / shale / dry-shale | 2.65 / 2.45 / 2.7 g/cc |
| Porosity min / max | 0 / 0.5 | | DT matrix / shale / fluid | 55.5 / 100 / 189 us/ft |
| Sw min / max | 0 / **0.85** | | NPHI shale / fluid | 0.4 / 1 |
| GR matrix / shale / method | 10 / 100 / Linear | | PEF shale / U shale | 4 / 9.4 |
| Res shale / method | 10 ohm.m / **Gaymard** | | HC density (min/def/max) | 0.1 / 0.7 / 0.8 g/cc |
| Grain density min / max | 2.65 / 3.0 g/cc | | Mineral 1–4 density | 2.65 / 2.71 / 2.85 / 2.98 |

These are directly usable as SandiBumi module-dialog defaults. Also: `CPMParameters\` holds
**capillary-pressure / saturation-height models** (Lambda function, porosity-dependent
coefficients) relevant to SandiBumi `sw_height`.

### E — Quanti.Elan mineral endpoints  ✅ highest analytical value, complete
`QM_MineralTable.xml` → **31 minerals × 20 properties** (`E_mineral_endpoints.json`):
Minerals (row idx), CEC, Compressional & Shear Slowness, GR, Neutron Porosity, PEF, Phicl,
Effective/total Porosity, K conc, Res, Bulk Density, Rhobdcl (dry clay), Rhobwcl (wet clay),
Rsh, Formation Sigma, Th conc, U (volumetric Pe), Ur conc. Simple density list
(`Minerals.xml`, 61 minerals) in `E_minerals_density.json`.

**Cross-check vs SandiMin** (`E_endpoint_DIFF_vs_sandimin.json`, 14 shared minerals):
- **Matrix densities agree** — Quartz, Calcite, Dolomite, Halite, Anhydrite, Gypsum all
  within tolerance. This independently validates SandiMin's core density endpoints.
- **Clay, Sigma, and GR endpoints diverge materially** (Illite GR 270 vs 160, Illite Sigma
  18 vs 40.56; Smectite, Kaolinite, mica similar). Expected: these are two independent
  vendor-default libraries, and clay response is the most basin-dependent. **Not bugs** —
  Techlog's values are a useful alternative/sensitivity set; for Mahakam clays, the project's
  own calibrated clay-typing values should override *both*.
- Techlog stores **Pe (barns/electron) separately from U (barns/cc)** — SandiMin stores U.
  Techlog therefore lets you seed either; a small convenience for the Elan-style dialog.

### F — Presentation templates  ✅ inventoried
`F_templates_inventory.json` (counts + schema samples): LayoutTemplates 360, TemplateTracks
74, plus Palettes, Headers, patterns, LithologyCatalog, CrossPlots. XML schema captured for
one LayoutTemplate + one TemplateTrack. Enough to seed `composite.rs` default track/scale/
color conventions and a lithology-pattern set; full port is a larger design task.

### G — Import/export configs  ✅ small, complete
`G_import_configs.json` (+ `G_geolog_mapping.xml.txt`). `Dlis.xml`/`Las.xml`/`Geolog.xml`
mapping configs captured. `Geolog.xml` (the Techlog↔Geolog Rosetta stone) is small (943 B)
in this install — thin, but confirms the naming-bridge concept. `propertyDict.xml`,
`wellStatus.xml` captured for reference.

### I — Training dataset  ✅ directly reusable, complete
`I_training_dataset.json`: **36 LAS files — the BLSO (Balam South) wells**, i.e. the *same
field* as SandiBumi's existing `pipeline_blso_test` regression fixture and the
`method_ssc_sspw_lqr` (LQR Balam South) method note. These are real Mahakam-adjacent open-hole
wireline data (`*_wire_fprooh.las`) — high-value regression fixtures. (Curve-name capture in
this pass grabbed the LAS `~W` section; the `~C` curve list needs SandiBumi's own LAS parser —
trivial follow-up since the files import natively.)

### D — Python algorithm sources  ✅ complete → `D_python_algorithms.md`
~538 sources across 7 packages + ~370 loose top-level scripts; ~35 read in full, rest swept.
**Key architectural finding:** EnvCorr and Acoustics use a *descriptor pattern* — the
readable `.py` files only declare which curves/mnemonics/resolution each vendor tool needs;
the actual correction/DSP math ships **compiled** (`EnvCorrPreProcessingPrivate.pyc`,
`TechlogAcoustic`, C++ wizards). So EnvCorr yields a rich tool→curve taxonomy but **no chart
coefficients** (reimplement corrections from the published chartbooks — the intended
independent path, and exactly what the chartbook-digitization effort covers), and Acoustics
yields no reimplementable geophysics.

- **Fully readable, transcribed with constants:** `RockPhyEquations.py` — complete
  **Batzle-Wang 1992** fluid properties, **Gardner** (per-lithology a,b), **Faust**,
  **Reuss/Voigt/Brie/Hill** mixing, elastic conversions; `TempCorr_Resistivity.py` —
  **Arps/Exxon** resistivity-temperature correction; **Han/Eberhart-Phillips** Vp/Vs
  coefficients; **Connolly/Whitcombe EI/EEI**; **Goodway LMR**; BP regional geotherms
  (incl. a Malaysia model).
- **Rw/salinity: Chart Gen-6** (`RwGen6`/`SalinityGen6`, LaVigne/ELAN, Schlumberger Charts
  2009, valid 0–260 °C) — kernel compiled but method + citation + validity range identified;
  reimplement from the published chart.
- **Method-identified, kernel compiled** (citations + defaults captured for literature
  reimplementation): Gassmann, Hertz-Mindlin soft/stiff/contact-cement (φc 0.4, Cn 9),
  Krief (α 2.7), Xu-White, DEM, Ciz-Shapiro 2007, Kuster-Toksöz, HS bounds, Berryman SC,
  Walton, BAM, Backus.
- **Readable top-level FE scripts flagged:** `Archie.py`, `VSH_GRequation/library.py`,
  `PorosityAndLithologyComputation.py` (120 KB), `TOC_Computation.py` (Passey ΔlogR /
  Schmoker), `QVn.py`, `BoreholeComputation.py`, geomech `UCS_*/YME_*/FG_*/GME_*` families.
- **Dead ends (don't chase):** RST (`RSTCOT_x64.dll`), all `RPI_*.pyc`/`Library_*.pyc`/
  `Raformula.pyc`/`RockPhysics_EquationsLibrary.pyc`, the EnvCorr chart engine,
  `TechlogAcoustic`, the `TechlogDatabase/Math/Stat/Plot` C-extensions. Nothing was
  decompiled.

### H — Offline documentation  ✅ complete → `H_doc_index.md`
~3,808 DITA-generated XHTML pages. The substance is `Doc\concept\` (1,417 pages; FE methods
are `petrophysics-<method>.html`) plus `Doc\topic\pythonlib\` (2,248 pages — one per Python
API callable, with argument names/units/defaults). **Structural fact that matters for
citation:** every method page stores symbols/units/**default values as machine-readable HTML
tables**, but the formula itself is a rasterized GIF/PNG in `Doc\image\` — ~20 key equation
images were transcribed by hand into `H_doc_index.md` with page + image-filename citations.

Equation-level (citable) coverage:
- **Sw family** — Archie, Simandoux (flushed + modified), Indonesia/Poupon-Leveaux,
  Waxman-Smits, Dual-Water, Total Shale, Juhasz, plus flushed-zone Sxo variants of each.
- **Vsh** — GR index + 7 named nonlinear variants (Clavier, Larionov ×2, Stieber ×3, Curved),
  K- and Th-based variants; literature ref Bassiouni 1994.
- **Porosity** — density, neutron-density (RHOB_sh 2.4 / NPHI_sh 0.4 defaults), sonic;
  **permeability** — Coates, Wyllie-Rose with Morris-Biggs/Timur coefficient tables.
- **The standout: Quanti.Elan theory chapter** (`quanti-elan-theory.html` + children) — a
  genuine numbered-equation derivation: incoherence minimization
  `incoherence = ½·Σ_tools[(x_REC − x)·UNC_WM/(UNC·LargestWeight)]²` (Eq 79) with
  weight = 1/uncertainty (Eq 80), ELAN Simandoux conductivity **Eq 78** with full derivation
  and defaults (ersh=1.0, swshe=0.5, mc2=0.0 / 0.19-tight), Dual-Water b(T)/mdw=1.8,
  Indonesian/Nigerian (EVCL/MVCL = 1.0/0.5 vs 1.4/0.0), Linear conductivity, wet/dry-clay
  bound-water partitioning. This is a citable equation-level basis for SandiMin's
  multi-mineral inversion that no textbook in the ITB shelf states as cleanly.

How-to-only (no standalone equations): landing/index pages, combined shale-volume merge,
Pickett-plot picking, environmental corrections. **Trap flagged:**
`concept\petrophysics-inversion.html` is CMR/NMR T2 inversion, *not* the ELAN mineral
solver — don't miscite it.

---

## 2. Feature shortlist (ranked by value-per-effort)

| # | Techlog capability | SandiBumi today | Gap | Effort | Value (Mahakam/LRLC) |
|---|---|---|---|---|---|
| 1 | 2,181-family + 2,839-rule + 723-alias catalog | `ingest.rs`/`parsers.rs` alias set (Geolog-derived, smaller) | Import the catalog as a second/primary alias source | **S** | High — fewer manual re-aliases on every import |
| 2 | 74 parametric neutron-density/sonic vendor charts | hand-digitized 2013 chartbook via `tools/chartdig`, `neutron_charts.rs` | Parse parametric XML → analytic overlays; may retire hand-digitization for vendor charts | **M** | High — exact vendor charts, no manual tracing |
| 3 | QM_MineralTable 31×20 endpoints | SandiMin table (AspenTech-derived) in `multimin_ref_spec.md` | Add as selectable alt endpoint library + sensitivity set | **S** | Med-High — clay sensitivity for LRLC |
| 4 | Family default units/scales/colors | ad-hoc defaults | Seed plot defaults from family catalog | **S** | Med — nicer default composite plots |
| 5 | BLSO training LAS (36 wells) | `pipeline_blso_test` fixture | Add wells as extra regression fixtures | **S** | Med — broader real-data test coverage |
| 6 | CPMParameters saturation-height (Lambda) models | `satheight.rs`/`sw_height` | Compare Lambda/Brooks-Corey defaults | **S** | Med — SwH parameter defaults |
| 7 | LayoutTemplate/TemplateTrack schema | `composite.rs` defaults | Port track/scale/color conventions | **M** | Med — template system |
| 8 | Quanti method numeric defaults (131–140/workflow, extracted → `C2_method_defaults.json`) | module dialogs use ad-hoc defaults | Adopt as dialog defaults | **S** | Med — sensible vendor defaults out of the box |
| 9 | Quanti.Elan theory chapter — numbered equations (incoherence Eq 79/80, Simandoux conductivity Eq 78, Dual-Water, Indo/Nigeria, wet/dry clay) | SandiMin implements the math but cites AspenTech spec + textbooks | Use as the citable equation-level reference in SandiMin docs/help; verify solver math term-by-term against Eq 78/79 | **S** | High — audit-grade citations for the LRLC/multimin workflows |
| 10 | Method-page parameter tables (defaults incl. Sw/Vsh/porosity/perm; Coates & Wyllie-Rose w/ Morris-Biggs/Timur coefficients) | perm not yet a first-class module; method help text is thin | Transcribed equations + defaults seed SandiBumi's method help panels and a perm module | **S–M** | Med-High — in-app method documentation users can trust |

| 11 | `RockPhyEquations.py` — Batzle-Wang fluids, Gardner, Faust, Reuss/Voigt/Brie/Hill, Han, EI/EEI, LMR (fully transcribed with constants) | no rock-physics module | New rock-physics/fluid-substitution module reimplemented from the cited papers | **M** | Med-High — gas fluid-substitution feeds the LRLC gas problem |
| 12 | Chart Gen-6 Rw↔salinity (LaVigne/ELAN, 0–260 °C) + Arps resistivity-T correction | Arps-style approximations in `equations.rs` utilities | Reimplement Gen-6 from the published 2009 chart; adopt validity range | **S** | Med — exact Rw handling under Mahakam temperature gradients |
| 13 | EnvCorr tool→curve taxonomy (65 tool descriptors: mnemonics, resample rates, WL/LWD, calibration properties) | no env-correction awareness at import | Use taxonomy for import QC/tool recognition; correction math comes from published chartbooks (chartbook-digitization path) | **S** (taxonomy) | Med — tool-aware ingest; corrections stay a chartbook job |

*(Shortlist final — all nine targets landed.)*

---

## 3. Discrepancy list (flag, don't fix)

1. **Clay/Sigma/GR endpoints: Techlog ≠ SandiMin** (target E). Independent-library
   difference, not an error. Action: none required; optionally expose Techlog values as an
   alternate library. Detail: `E_endpoint_DIFF_vs_sandimin.json`.
2. **Matrix densities: Techlog = SandiMin** — a positive cross-validation of SandiMin's core
   endpoints; worth recording as evidence they're right.
3. **Baryte mapping is not comparable** — Techlog "Baryte" is a solid mineral; SandiMin's
   "X Special Fluid (barite)" is a mud additive. The diff row is flagged invalid, not a bug.

---

## 4. Completeness & what was skipped

- **Fully extracted:** A (100%), B (112/112 charts), E (31/31 minerals + diff), F (inventory
  + schema), G (all configs), I (36/36 LAS listed).
- **Partial:** C — workflow containers yes, numeric method defaults no (need parameter-file
  format pass).
- **Deliberately skipped:** `bin64\`, bundled `Python27_x64`/`Python36_x64` runtimes, fonts,
  shaders, sounds, png/gif assets — opaque or irrelevant binaries.
- **Delegated (subagents), both complete:** D (538 PythonScripts sources → 
  `D_python_algorithms.md`; compiled `.pyc`/`.dll` kernels deliberately NOT decompiled —
  method + citation + defaults only), H (3,808 Doc HTML pages → `H_doc_index.md`; ~20
  equation images transcribed with page + image citations).
- **Vintage:** 2018.2 — fine for catalogs/physics; note the version when citing values.

---

## 5. Output file manifest
```
FINDINGS.md                        <- this file
A_families.json                    2,181 curve families
A_family_assignment.json           2,839 assignment rules
A_mnemonic_alias.json              723 mnemonic aliases
A_unit_systems.json                6 unit systems + unit-alias sample
A_variable_affixes.json            (empty in this install)
B_charts_catalog.json              112 charts (families/params/vendor)
B_chart_samples.json               6 full parametric chart XMLs (schema)
B_chart_autoselect.xml.txt         mnemonic->chart rules
C_quanti_workflows.json            18 Quanti workflow containers
C_quanti_samples.json              3 full workflow XMLs
C2_method_defaults.json            131-140 numeric defaults/workflow (.info equivalent)
A_alias_addendum_various.json      DCN2Alias LWD/REW extra alias tables
E_mineral_endpoints.json           31x20 QM mineral endpoint table
E_minerals_density.json            61-mineral density list
E_endpoint_DIFF_vs_sandimin.json   cross-check vs SandiMin sec-I
F_templates_inventory.json         template counts + 2 schema samples
G_import_configs.json              DLIS/LAS/Geolog import configs
G_geolog_mapping.xml.txt           Techlog<->Geolog mapping
I_training_dataset.json            36 BLSO (Balam South) LAS
D_python_algorithms.md             (subagent) algorithm sources
H_doc_index.md                     (subagent) doc index + equations
_extract_stats.json                run stats
```
