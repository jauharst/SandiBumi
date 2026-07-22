# Interactive Petrophysics 2025.3 ingest — FINDINGS

Ingested `C:\Program Files\IP2025` (Interactive Petrophysics **2025.3**, publisher PGL /
Lloyd's Register / Geoactive; main exe `Intpetro.exe`) on 2026-07-22 for SandiBumi feature
mining. Source treated read-only; all outputs in this folder. This is the **companion ingest
to the Techlog 2018.2 one** (`../techlog_ingest/FINDINGS.md`) — same target scheme (A–I), so
the two vendor install-trees are directly comparable and both feed the SandiBumi maturation
prompt.

**IP-cleanliness note (verified by a completeness critic — held with zero leaks):**
catalog / chart / endpoint / parameter-default DATA extracted verbatim (Tier A reference
data). Published methods (Tier B) were identified by name + constants only, to be
reimplemented from the primary paper. **Patented / proprietary methods (Tier C) — Omovie
`SonicSaturation` (US Patent 12,242,011 B2), Domain Transfer Analysis, Experienced Eye,
entropy image speed-correction, shipped NN weights — were registered by existence + user-need
only; no protected algorithm was read, decompiled, or approximated.** Module algorithm kernels
ship compiled (`UserProgram.dll`) and were never decompiled; only each module's readable
`Parameters` text file was extracted.

Extraction was run as a 10-agent workflow (9 targets A–I + a completeness/cleanliness critic).
20 JSON + 6 MD outputs, all validated.

---

## 0. Headline

IP 2025.3 is a **richer and more open install than the Techlog tree — but open in a different
place.** Three things define it:

1. **Cross-tool naming BRIDGES are IP's unique contribution.** IP ships literal transpiler
   dictionaries: **Geolog Loglan(`.lls`)→VB** (`LlsToVbConfig.xml`, 82 builtins),
   **PowerLog→VB** (22 petro builtins), and **Elan(Techlog)↔IP** mineral/equation maps
   (`ElanToIPMapping.par`). Since SandiBumi *descends from Geolog Loglan*, the Loglan-builtin
   semantics table (MISS→IsNaN, NONMISS overloads, null-safe divide, degrees trig, frame I/O)
   is a ready-made migration aid Techlog's tree simply did not contain.
2. **A third independent mineral-endpoint library** (`MINDEF.PAR`, 30 minerals × 51 columns).
   Cross-checked three ways — IP vs the Techlog `QM_MineralTable` vs SandiMin/AspenTech — it
   **corroborates SandiMin's core matrix endpoints** (RHOB agrees across all three libraries
   for every clean non-clay mineral; Calcite and Halite agree on *every* property) and shows
   the clay/NPHI/Sigma/CEC divergences are library-vintage differences, not SandiMin bugs.
   It also surfaced one genuine **SandiMin-side review item** (smectite density).
3. **The whole vendor chart library is machine-readable ASCII** — 49 `.neu` neutron tables +
   167 `.ovl` crossplot overlays. These are the *same* Schlumberger curves Jauhar hand-
   digitized from the 2013 chartbook (Por-4/5/13/14, CP-16, M-N/MID), shipped as clean X/Y/
   tick points. **This can retire most of `tools/chartdig`'s PDF digitization** and cross-
   validate what's already digitized.

The **honest counterweight** (from the critic): IP keeps its *core openhole deterministic
defaults* (Archie a/m/n, Rw, Vsh Larionov/Clavier coefficients, porosity matrix endpoints)
inside the compiled `Intpetro.exe` interactive panels, **not** in any shippable `.PAR` or
module `Parameters` file. So IP is *not* a source for core-method default constants — those
come from the Techlog ingest and primary literature. IP's install-level openness is in
catalogs, charts, endpoints, the object model, and the scripting on-ramp — not the core math.

---

## 1. Per-target results

### A — Catalogs + cross-tool naming bridges  ✅ complete (bridges = the win)
| Artifact | Count | Output |
|---|---|---|
| Curve-alias catalog (`DefaultAlias.cax`; named `CurveAlias.txt` ships EMPTY) | 315 aliases → 52 CurveTypes | `A_curve_alias.json` |
| Curve-type / lithology / zone-color / shade catalogs | 417 curve labels, 39 litho, 25 zone colors, 195 fills | `A_catalogs.json` |
| Unit tables (IP→Petrel, OpenSpirit vocab, tool-category factors) | 33 + 1132 + (4 cal/10 son/10 den/13 poro) | `A_catalogs.json` |
| **Elan↔IP** mineral/equation bridge | 23 minerals + 12 equations | `A_naming_bridges.json` |
| **Geolog Loglan(`.lls`)→VB** transpiler builtins | 82 builtins | `A_naming_bridges.json` |
| **PowerLog→VB** petro builtins (name + arity only, Tier B) | 22 | `A_naming_bridges.json` |

**IP vs Techlog for auto-aliasing:** Techlog wins decisively on the alias *catalog* (723
aliases / 2,181 families vs IP's 315/52), and the two overlap on only 13 tokens — IP's are
dominated by vendor tool-channel names (MWD/image/azimuthal), Techlog's by canonical
petrophysical mnemonics, so they're **complementary, not redundant**. Recommendation: Techlog
as the primary alias seed, merge IP's ~302 vendor-channel names as an additive layer. IP's
decisive edge is the **bridges**, which Techlog didn't provide.

### B — Vendor chart library  ✅ complete, high value
233 files, all parsed: **49 `.neu`** (neutron matrix + salinity-correction lookup tables,
18-column: 3 matrix + 15 salinity at 50/100/150/200/250 kppm), **167 `.ovl`** (80 density-
neutron, 36 permeability r35 [Pittman/Winland/Lucia], 14 sonic, 8 gas-isotope, 8 pyrolysis,
6 Pe-density, 5 spectral-GR, 3 Buckles, 2 M-N, 2 MID), **17 `.cht`** (cased-hole, scope=out),
**82 `.pal`** palettes. Catalog in `B_charts_catalog.json`; 9 fully-parsed samples in
`B_chart_samples.json`.
- **Directly overlaps `tools/chartdig`** — the Schlumberger `.neu`/`.ovl` files are a machine-
  readable copy of the exact chartbook curves being hand-digitized. Skips PDF digitization for
  the whole neutron-density/Pe/M-N/MID family and every vendor LWD tool, and cross-validates
  already-digitized Por-4/5/13/14.
- **IP vs Techlog:** IP dwarfs Techlog on vendor/LWD neutron tables (49 vs ~11) and perm-r35
  charts (36 vs 4); Techlog has rock-physics + pore-pressure overlays (37) that IP lacks (both
  "later" scope). Note: the raw D-N count gap partly reflects IP's per-file fresh/salt
  granularity vs Techlog's single rhof parameter.

### C — Module parameter defaults  ✅ complete (61 modules)
All **61 interpretation modules** enumerated; defaults pulled from each readable `Parameters`
file (`C_module_defaults.json`), plus 7 top-level `.par` default files
(`C_toplevel_par_defaults.json`). 591 params across 54 modules; 7 modules are runtime/curve-
driven (no static params). Tier split: **41 Tier B / 19 Tier A / 1 Tier C**. Scope split:
**7 v1-core / 50 later / 4 out**.
- **Mahakam-relevant, v1-core:** `Sand_Silt_Malay_Model` (3-mineral SSC with a 4-way Sw switch
  — Waxman-Smits / Juhász W-S / Dual-Water / Archie PhiT; silt as a distinct mineral;
  Den_Dry_Silt 2.68, Den_Wet_Clay 2.5, Den_Dry_Clay 2.78, a=1/m=2/n=2), `RwFromSP`,
  `ShaleLimits` (combined clay+φ+Sw net-pay flags), `LogQC`.
- **Top-level defaults captured:** 30 Poisson-ratio lithologies, 14 Gassmann minerals
  (K/µ/DTc), 5 cap-pressure fluid systems, Eaton/Matthews-Kelly fracture-gradient coeffs, and
  the **`MonteCarloDefaults.par`** per-parameter uncertainty library (Rw 20%, Rmf 20%, m ±0.2,
  n ±0.2, a ±0.1, Qv a/b 10%, endpoint shifts) — a ready seed for SandiBumi's MC engine.
- The 50 "later" modules are the entire PPFG pore-pressure/geomechanics toolbox
  (DT_NCT_* Bowers/Chapman/Miller/EHP, Dxc, Eaton, overburden builders) + source-rock/unconv
  (TOC-Passey, Pyrolysis) — correctly routed out of v1.

### E — Mineral endpoints + three-way cross-check  ✅ highest analytical value
`MINDEF.PAR` → **30 minerals/fluids × 51 columns** (`E_ip_endpoints.json`); 2 `.mdl` MinSolve
models best-effort parsed (`E_mdl_models.json`, secondary — MINDEF is authoritative); the
three-way reconciliation over **18 shared minerals × 7 properties** in
`E_threeway_endpoint_compare.json`.
- **Corroborates SandiMin's core.** RHOB agrees across all three libraries for every clean
  non-clay matrix mineral (12/18); **Calcite and Halite agree on every property**. Clay / NPHI
  / U / Sigma / CEC diverge — expected library-provenance differences, **not bugs**.
- **Two load-bearing IP conventions:** IP has **no direct GR endpoint** (GR is composed from
  K/Th/U concentration columns via the MinSolve GammaRay equation), and IP **CEC is a wet-clay
  volumetric Qv (meq/cm³)**, not meq/g — align accordingly before comparing.
- **Findings worth acting on (see §3):** Techlog's smectite CEC 0.0 is a null-as-zero artifact
  (IP 1.38 / SandiMin 1.0 are real); and **SandiMin's smectite RHOB 2.63 reads like a dry-
  grain value** far from both wet-clay libraries (IP 2.02 / Techlog 2.12) — a genuine SandiMin
  review item.

### F — Presentation templates + lithology assets  ✅ inventoried
`F_templates_inventory.json`, `F_lithology_patterns.json`, `F_palettes.json`, `F_notes.md`:
117 Default Plots (10 `.plt` composites incl. **Composite CPI.plt**, 16 `.trk` tracks, 26
`.xpt` crossplot presets incl. Pickett/Buckles/M-N/MID/Pe-density, 11 `.svg` headers), 38
geomechanics plots (later), 162 shading bitmaps (**96-pattern `*_final` lithology taxonomy**),
82 `.pal` (COLORREF-decoded: Earth/Grayscale/Rainbow/tristate).
- **Key finding:** IP's `.plt`/`.trk` templates are **plain-text keyword records** (TRACK /
  CURVE / SHADE / GRID / ORDER) that **bind curves by FAMILY, not literal name** — far more
  portable to `composite.rs` than Techlog's 600-line-per-track Qt XML. A ~20-line tokenizer
  maps them 1:1 to a SandiBumi composite model; `.trk` files are drop-in single-track blocks.
  `Composite CPI.plt` is a ready Mahakam-suitable default CPI layout.

### G — Data model + reference tables + bridges  ✅ complete (with a correction)
`G_datamodel_schemas.json`, `G_reference_tables.json`, `G_geolog_openworks_bridges.json`.
- **Correction to the plan:** the 6 top-level `.xsd` are per-feature tool configs, *not* IP's
  project data model. The real model is the compiled .NET object model (from `PGL.IP.API.xml`):
  **`Database > Well > CurveSet > Curve`** + per-well **`ZoneSet > Zone > Parameter`** — which
  **maps 1:1 to SandiBumi's DuckDB and validates its design.** The gap SandiBumi could close:
  explicit provenance columns (create/update user+module+date, FinalVersion, CurveStatus,
  Locked) and array-curve dims — plus IP's `DepthReferenceType` enum (MD/TVD_KB/GL/SS/SB) and
  built-in percentile stats (which serve GR P3/P97 with no separate module).
- **Reference tables:** 168 casing sizes, 22 hole sizes, 68 paper sizes, IP→Petrel unit map,
  Geolog-ASCII import heuristics (depth-mnemonic + tops-set detection).

### H — Scripting/formula API + docs  ✅ complete
`H_api_surface.json` (parsed `PGL.IP.API.xml`, 2,323 members / 235 types), `H_doc_index.md`,
`H_formula_language.md`.
- **Documentation form beats Techlog** (text `.NET` XML-doc, one parseable file, vs Techlog's
  rasterized equation GIFs) — **but IP's API is an automation/data-access API, not an equation
  library.** It exposes curves/curve-sets/wells/zones/parameters/units/statistics/plots, and
  **no Archie/Vsh/Sw/porosity math** (only `ip2py.calculations.gamma_ray_index`). So: **cite
  Techlog's concept pages + primary papers for method math; cite IP for the object/parameter
  model and the formula/user-python on-ramp.** They're complementary.
- **On-ramp shapes to match:** the IP Formula (`.frm`) single-expression grammar (bare
  mnemonics as variables, DEPTH/TVD reserved, condition + TOP/BOTTOM window, output curve+unit,
  −999 nulls) and the modern **`ip2py`** vectorized python bridge (curves→DataFrame→numpy/ML→
  write back). Matching these lets IP-trained users port formulas/scripts with minimal edits.

### D — Formula/UserProgram language + Tier-B methods + Tier-C register  ✅ complete
`D_formula_language.md`, `D_readable_algorithms.json`, `D_tierC_register.md`.
- **IP has no bespoke DSL:** a "User Program" is a snippet of a general-purpose language
  (VB.NET/C#/C/FORTRAN/MATLAB/IronPython, `Compiler=` tag) compiled to `UserProgram.dll` and
  called in a per-index depth loop; `ip2py` (pandas/Jupyter) is the modern ML on-ramp. **IP
  ships its SDK examples as OPEN source in 7 languages** — so the *interface pattern*
  (`param(index)` scalar/curve interchange, `Save_*` outputs, −999 nulls, family-tagged I/O,
  Pickett two-point→{m,Rw} back-solve) is fully readable, even though IP's own 65 built-in
  modules stay compiled.
- **Readable Tier-B methods (identified, reimplement from primary):** Hydraulic Flow Units
  (`UserAppCode.cs` — Amaefule RQI/FZI + Winland r35 + Pittman apex) and `Interp_Demo`
  (Archie / Indonesia / Wyllie / Shell-m / Pickett quicklook).
- **Consolidated Tier-C register** (`D_tierC_register.md` — the standing file for counsel): 6
  restricted items (SonicSaturation/Omovie, DTA, Experienced Eye, entropy image speed-
  correction, NN weight DLLs, + Recall) each with existence, user-need, and Tier-B design-
  around; **MLNET (open ML.NET, MIT) explicitly cleared** so a reviewer doesn't tar it with
  DTA's status.

### I — Test data  ✅ complete (weak fixture value)
`I_testdata.json`: 9 files, effectively **one portable well** — `Testdata.las` (LAS 2.0
WRAP=YES, synthetic "Test Well 1", 14 curves × 2249 rows, openhole triple/quad combo +
interleaved core perm/φ/grain-density) + its ASCII twin `Testdata.txt`. Usable as a compact
LAS-parser + quicklook regression fixture (stresses WRAP, an IP `~P` parameter block, −999
nulls, log+discrete-core interleaving). The rest are opaque IP-native binary or WITSML demo
data. **Unlike Techlog's 36 real BLSO Balam South LAS, IP ships a single synthetic demo** —
weaker as a corpus, and **redistribution is EULA-governed (do not bundle without checking).**
One safety note: `XStreamConfig.dat` carries a WITSML username + DPAPI-encrypted password
blobs — observed only, never to be copied into fixtures.

---

## 2. Feature shortlist (ranked by value-per-effort)

| # | IP capability | SandiBumi today | Gap / action | Effort | Value (Mahakam/LRLC) |
|---|---|---|---|---|---|
| 1 | Three-way endpoint corroboration (IP `MINDEF.PAR` as 3rd library) | SandiMin table (AspenTech) + Techlog cross-check | Fold IP as a selectable alt endpoint library + record the 3-way agreement as validation evidence | **S** | High — de-risks the SandiMin matrix endpoints |
| 2 | Geolog Loglan(`.lls`)→target builtin dictionary (82 fns) | descends from Loglan but has no formal builtin map | Adopt as the semantics table for porting legacy Loglan modules into SandiBumi | **M** | High — direct migration aid, Jauhar's own heritage |
| 3 | `.neu`/`.ovl` chart library (233 ASCII files) | `tools/chartdig` digitizes chartbook PDFs one at a time | Ingest directly; retire PDF digitization for vendor neutron-density/Pe/M-N/MID + cross-check digitized charts | **S–M** | High — exact vendor charts incl. LWD/ADN tools that logged Mahakam wells |
| 4 | Elan↔IP mineral/equation name bridge | needs canonical mineral vocab across Elan/Geolog | Drop-in QUAR/DOLO/CALC… → canonical map + equation aliases for cross-tool model import | **S** | Med-High — cleaner multimin mineral naming |
| 5 | `.plt`/`.trk` family-bound text template grammar + `Composite CPI.plt` | `composite.rs` ad-hoc defaults | Adopt the keyword DSL + default CPI layout + SHADE fill primitive | **M** | Med-High — portable templates, sensible default CPI |
| 6 | `Sand_Silt_Malay_Model` default endpoint set + 4-way Sw switch | SSC/SSPW (Kyi-Bonnye, Sugiharto-Gaafar, Kuttan) | Cross-calibrate SandiBumi silt endpoints against a shipping Mahakam-oriented tool (reimplement from primary) | **M** | High — Malay-basin laminated shaly-sand |
| 7 | `ip2py`-style pandas/Jupyter on-ramp + IP `.frm` formula grammar | `equations.rs`/python engine; no IP-shaped formula surface | Offer a compatible expression grammar + vectorized python bridge so IP-trained users port verbatim | **M–L** | Med-High — adoption on-ramp; matches Jauhar's LightGBM/deconv ML workflows |
| 8 | `MonteCarloDefaults.par` uncertainty library | MC engine exists (`montecarlo.rs`), ad-hoc distributions | Seed per-parameter shift-type/distribution/magnitude defaults (Rw/m/n/a/Qv/endpoints) | **S** | Med — sensible P10/P50/P90 envelopes out of the box |
| 9 | IP object-model provenance columns + `DepthReferenceType` enum + built-in percentiles | DuckDB project>well>curveset>curve; versioning shipped | Mirror provenance/array-dim columns; adopt datum enum + percentile stats (feeds GR P3/P97) | **S** | Med — schema hardening, no separate GRN percentile module |
| 10 | HFU permeability (Amaefule RQI/FZI + Winland + Pittman) | no explicit FZI rock-typing→perm module | New perm/rock-typing module reimplemented from SPE 26436 / Pittman 1992 (readable in IP HFU source) | **S–M** | Med-High — rock-typed perm + net-pay consistency |
| 11 | Lithology-pattern taxonomy (96 `*_final` names) + COLORREF ramps + `.xpt` crossplot presets | client-brand palettes, no standard litho-fill set | Redraw pattern names as licence-clean SVG hatch; adopt ramps + 26 crossplot presets | **S–M** | Med — industry-standard litho tracks + presets |
| 12 | RwFromSP + LogQC flag modules | Vsh-PHIE net-pay; conditioning agent | Add SP-based Rw QC + a formal QC-flag contract for the conditioning agent | **S** | Med — cheap independent Rw QC (caution: fresh Mahakam mud suppresses SSP) |

---

## 3. Discrepancy / review list (flag, don't silently fix)

1. **Matrix densities agree across all three libraries** (IP/Techlog/SandiMin) — positive
   cross-validation of SandiMin's core; record as evidence.
2. **SandiMin smectite/montmorillonite RHOB 2.63** reads like a dry-grain value, far from both
   wet-clay libraries (IP 2.02 / Techlog 2.12). **Genuine SandiMin-side review item** — decide
   whether the multimin table intends a wet- or dry-clay smectite density.
3. **Techlog smectite CEC 0.0 is a null-as-zero artifact** (IP 1.38 / SandiMin 1.0 are the
   real values) — the Techlog `E_endpoint_DIFF` "diverge" verdict there is driven by a missing
   datum, not a real disagreement. Corrects a note in the Techlog ingest.
4. **Dolomite Sigma:** IP 6.92 = SandiMin 6.92, but Techlog 4.7 (textbook dolomite Σ ≈ 4.7
   c.u.). IP and SandiMin share an unusually high value — flag for provenance.
5. **Clay densities diverge by modelling convention** — IP stores wet-clay-model density,
   Techlog/SandiMin store dry-clay. Not a data error; align the convention before comparing.
6. **GR endpoints:** IP has none (composes GR from K/Th/U), so GR comparison is Techlog-vs-
   SandiMin only and diverges everywhere — independent reference values, project-calibrate.
7. **Vsh/Sw architectural difference:** Techlog carries a per-module Sw max (0.85) and single
   Vsh method default; IP applies pay limits via a separate `ShaleLimits` module and emits both
   Vsh_GR and Vsh_ND. Same constraints, different placement — a design note, not a bug.

---

## 4. Completeness & what was skipped

- **Fully extracted:** A (315 aliases + 3 bridges), B (233/233 charts), C (61/61 modules +
  7 `.par`), E (30 minerals + 18-mineral 3-way diff), F (full inventory + schema), G (object
  model + reference tables + bridges), H (2,323 API members), I (9/9 files), D (4 readable
  Tier-B methods + full Tier-C register).
- **Genuine gap (structural, not a miss):** IP's **core openhole deterministic defaults**
  (Archie a/m/n, Rw, Vsh Larionov/Clavier, porosity matrix endpoints) are **not in the
  install** — they live in the compiled `Intpetro.exe` panels + per-project DB. Method
  *presence* is documented; default *constants* must come from the Techlog ingest + primary
  literature. Only 7/61 modules are v1-core because IP's core petrophysics isn't a plug-in.
- **Deliberately skipped:** all `.dll`/`.exe` binaries (never decompiled), NN weight DLLs, the
  DTA/Experienced-Eye executables, `DomainTransferAnalysisKeys.txt` (opaque base64), the 2
  `.mdl` binary internals beyond best-effort (MINDEF is authoritative), `.pal` pixel LUTs
  (names/decoded ramps only), cased-hole `.cht`/`.rdlc`, WITSML demo data + credentials.
- **Known limitation in `E_threeway_endpoint_compare.json`:** the 5%-agreement test doesn't
  pin its denominator convention (min/max/mean), so two borderline verdicts (Quartz U, Chlorite
  RHOB) sit on the line. Noted; not material to the corroboration conclusion.
- **Vintage:** IP **2025.3** (current) vs Techlog **2018.2** — note the versions when citing.

---

## 5. Output file manifest
```
FINDINGS.md                          <- this file
A_curve_alias.json                   315 aliases -> 52 CurveTypes (DefaultAlias.cax)
A_catalogs.json                      curve-type/litho/zone-color/unit catalogs
A_naming_bridges.json                Elan<->IP, Loglan->VB (82), PowerLog->VB (22)
B_charts_catalog.json                233 charts typed (49 neu/167 ovl/17 cht/82 pal)
B_chart_samples.json                 9 fully-parsed charts (schema reference)
B_palettes_inventory.json            82 palette inventory
B_notes.md                           chart-library notes + Techlog comparison
C_module_defaults.json               61 modules, params + tier + scope
C_toplevel_par_defaults.json         7 top-level .par (Poisson/Gassmann/MC/FracGrad/...)
E_ip_endpoints.json                  MINDEF.PAR 30 minerals x 51 cols
E_mdl_models.json                    2 MinSolve .mdl models (best-effort)
E_threeway_endpoint_compare.json     IP vs Techlog vs SandiMin, 18 minerals x 7 props
E_notes.md                           endpoint conventions + reconciliation notes
F_templates_inventory.json           117 plots + 38 geomech + symbols + reports
F_lithology_patterns.json            96 *_final lithology pattern names
F_palettes.json                      82 palettes, COLORREF-decoded samples
F_notes.md                           .plt/.trk grammar + adoption notes
G_datamodel_schemas.json             IP object model + 6 xsd summaries
G_reference_tables.json              casing/hole/paper sizes, unit maps
G_geolog_openworks_bridges.json      Geolog/OpenWorks DB-link + import bridges
G_notes.md                           data-model correction + notes
H_api_surface.json                   PGL.IP.API.xml parsed (2323 members/235 types)
H_doc_index.md                       ApiDocumentation index + API-vs-Techlog note
H_formula_language.md                IP .frm + ip2py on-ramp notes
D_formula_language.md                UserProgram/formula language (7 bindings)
D_readable_algorithms.json           readable Tier-B methods (HFU, Interp_Demo)
D_tierC_register.md                  consolidated Tier-C register (for counsel)
I_testdata.json                      9 TestData files (1 portable synthetic LAS)
```
