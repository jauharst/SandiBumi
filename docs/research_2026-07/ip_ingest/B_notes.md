# TARGET B — IP2025 Vendor Chart Library (notes)

Source (READ-ONLY): `C:\Program Files\IP2025`. Parser: `parse_charts.py` (scratchpad).
Outputs: `B_charts_catalog.json` (all 233 files typed), `B_chart_samples.json` (9 fully-parsed),
`B_palettes_inventory.json` (82). All chart files are clean ASCII / XML — Tier A reference data
(lookup tables, tool taxonomies, matrix endpoints), plus a few Tier-B method overlays (see below).
No compiled code read; no Tier-C material touched.

## Inventory (233 chart files)
| type | count | what it is | scope |
|------|-------|-----------|-------|
| `.neu` | 49 | neutron-porosity lookup tables (True Phi vs SS/Dol matrix + 5-step salinity corr 50–250 kppm) | v1-core |
| `.ovl` | 167 | parametric crossplot overlays (matrix lines, tick porosity, labels) | mostly v1-core |
| `.cht` | 17 | cased-hole cement/casing tool templates (XML) | **out** (cased-hole) |
| `.pal` | 82 | 256-entry COLORREF display palettes (image/NMR/acoustic/cement) | later (display) |

Scope tally across the 216 neu+ovl charts: **v1-core 197, later 19, out(cht) 17**.

## File-format schema (for `tools/chartdig` to ingest directly — no PDF digitization needed)
**`.ovl`** — sectioned ASCII, `$`-comment section headers:
- `$ Tool Types` → next line = `<X-axis tool> <Y-axis tool>` (the crossplot axes)
- `$ Lithology Name` / `$ Names` → space-separated per-polyline labels (e.g. `SS LS DOL`, or contour values)
- `$ Color of lines` → per-polyline color name (Delphi/.NET names: DkGray, Red, Teal…)
- `$ Data: format X, Y, Type` → rows of triplets; **column j = polyline j**. Type is `-` (plain vertex),
  `Tick` (minor graduation), or a numeric string = a labelled porosity graduation (0,10,20,30,40…).
  `- - -` triplets are padding for shorter polylines.
- `$ Data Labels` → `X Y Rotation Color Text…` annotation strings.
- `$ Default Font Size`, `$ Line Width` trailers.

**`.neu`** — `$`-comment header then a fixed 18-column numeric table:
`phi | ss | Dol | {50,100,150,200,250 kppm}×{SS,LS,Dol}`. `phi` = true porosity referenced to
**limestone** matrix; `ss`/`Dol` = apparent-porosity shift to convert to SS/Dol matrix; the 15
salinity columns are additive corrections. "Porosity values must not be changed" (rows are the
lookup key). `Neu_Parm_Files.neu` is the manifest indexing contractor→tool→table.

**`.cht`** — XML `<InteractivePetrophysicsCasedHoleTool>`; carries polynomial coefficient sets
(e.g. CBL attenuation vs cement compressive strength). All 17 are cement-bond / casing-inspection
(CBL/SBT/USI/CAST/Radial/Caliper) → cased-hole, out of v1 scope. Catalogued, not parsed to points.

**`.pal`** — `index=COLORREF` (0x00BBGGRR). Converted first/mid/last to `#RRGGBB` in the inventory.
Mostly 256-entry image display LUTs; not petrophysical reference — inventory only.

## Fully-parsed samples (B_chart_samples.json) — schema reference
7 OVL + 2 NEU parsed to structured points:
`BA_2490_DEN_fresh` (3 matrix lines SS/LS/DOL, 9 pts each, porosity ticks preserved),
`Sch_TNPH_RHOB_fresh`, `Sch_Por_Lithology_fresh` (Pe–density, CP-16), `Baker_M_N_Mineral_Ident`
(29 mineral tie-lines + 15 labels), `Umatrix_RhoMatrix` (MID triangle Qtz/Cal/Dol),
`buckles` (8 iso-BVW hyperbolae), `R35_Winland` (9 iso-perm curves); NEU `Sch_CNL` + `BA_2490`
as full 18-col salinity tables.

## Vendor / tool taxonomy (neu+ovl)
Every major contractor + LWD sub-brand is represented as explicit matrix/salinity tables:
- **Schlumberger** wireline (CNL, TNPH, APLC, FPLC, SNP) + LWD sub-brands **Anadrill** (ADN/CDN 4¾–8")
  and **PathFinder** (DNSC/SDNSC).
- **Baker Hughes**: Baker Atlas/Western Atlas wireline (2420/2435/2436/2446/2490) + **INTEQ** LWD (SDN 4¾–8¼").
- **Halliburton**: wireline (DSN/CNT/HDSN/DSEN) + **Sperry/Sperry-Sun** LWD (CTN 4–8" AmBe/Cf).
- **Weatherford** (CNT/MDN) + **Reeves** (CNS374) + Weatherford LWD TNP (4¾/6¼/8").
- **Generic/method** overlays (63): Buckles, Winland/Pittman/Lucia/K-Phi perm, M-N & MID, Pe-density,
  spectral-GR (Th/K/Pe, NGS), gas geochemistry, source-rock pyrolysis.

## Comparison to Techlog (112 charts, `techlog_ingest/B_charts_catalog.json`)
Both cover the openhole core: D-N (TL 43 / IP 80 ovl + 49 neu), sonic-neutron (TL 13 / IP 11),
Pe-density (TL 3 / IP 6), M-N (2/2), MID (1/2), spectral-GR (3/5), Buckles (TL has iso-BVW too).

**IP has, Techlog largely/entirely lacks:**
- **Deep LWD neutron lookup tables** — IP ships 49 `.neu` contractor tables (every wireline+LWD sub-brand
  incl. Anadrill, PathFinder, Sperry, INTEQ, Reeves, Weatherford-LWD). Techlog exposes only ~11 LWD charts
  and no equivalent salinity-corrected lookup-table format.
- **Permeability r-apex suite** — 36 charts: Pittman R10–R75 (%- and decimal-φ), Winland R35, Lucia rock
  fabric, K-Phi ratio. Techlog has only 4 porosity–permeability charts.
- **Gas geochemistry** (8): Bernard, Milkov, Schoell, Whiticar, Lorant isotope plots. Techlog: none.
- **Source-rock pyrolysis** (8): van-Krevelen HI/OI, Tmax/PI/HI, TOC vs S1/S2/PPO. Techlog: none.
- **Cased-hole cement/casing tool templates** (17 `.cht`). Techlog chart lib: none.

**Techlog has, IP's chart library lacks:**
- **Rock-physics suite** (~35): Vp/Vs/AI/RHOB vs φ for Qtz/Cal/Dol-water, bulk/shear moduli,
  LambdaRho–MuRho, AI–Poisson, VpVs–AI. IP does rock physics inside modules, not as chart overlays.
- **Bowers pore-pressure** crossplot overlays (Effective Stress vs Velocity/Slowness). IP = module, not chart.
- **Rh–Rv anisotropy** (Klein-style) crossplot.
Both rock-physics and pore-pressure are SandiBumi **"later"** scope, so their absence in IP does not hurt v1.

## Overlap with the 2013 Schlumberger chartbook digitization (`tools/chartdig`)
IP's `Sch_*` overlays/tables are a **machine-readable copy of the exact chartbook curves Jauhar
hand-digitized** — direct cross-validation source, no PDF operator-walking needed:
- `Sch_CNL.neu` / `Sch_TNPH_RHOB` / `Sch_NPHI_RHOB` (fresh+salt) ↔ **Por-13/Por-14** D-N.
- `Sch_APLC.neu` / `Sch_FPLC.neu` / `Sch_SNP.neu` ↔ **Por-4/Por-5** neutron matrix equivalence
  (the ones already emitted to `neutron_charts.rs`). Note the solid/dashed TNPH-fresh vs FPLC/salt
  distinction the extractor tracks is here explicit as separate files (`_fresh`/`_salt`).
- `Sch_Por_Lithology_fresh.ovl` ↔ **CP-16** Pe-density porosity-lithology.
- `Sch_M_N_Mineral_Ident.ovl` + `Umatrix_RhoMatrix.ovl` ↔ **CP-14/CP-15** M-N / MID (chartbook memo's
  named "candidates for the same treatment later").
- Dolomite endpoint: IP `.neu` reference φ carries the standard chartbook curve; watch the chartbook's
  drawn **ρma 2.85 dolomite** (per digitization memo) vs endpoint-table 2.87 when cross-checking.

## Tiering
All extracted data is **Tier A** (lookup tables, matrix endpoints, tool taxonomies) or **Tier B**
method overlays whose science is public and cited on the chart itself: Buckles (iso-BVW), Winland R35,
Pittman r-apex, Lucia rock-fabric classes, Thomas-of-none here, van-Krevelen pyrolysis, Bernard/Schoell
gas-genetic plots. For any Tier-B overlay SandiBumi should reimplement from the primary paper (Winland
1972/Kolodzie 1980; Pittman 1992; Lucia 1995; Bernard 1978; Schoell 1983) — the IP overlay only
confirms the curve family and default contour values. No Tier-C content (SonicSaturation / DTA /
Experienced-Eye) exists in the chart library. No compiled code decompiled.
