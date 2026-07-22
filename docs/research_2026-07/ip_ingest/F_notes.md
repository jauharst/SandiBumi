# Target F — IP2025 Presentation / Plot Templates + Lithology Patterns

Source (read-only): `C:\Program Files\IP2025`
Extracted 2026-07-22. Outputs: `F_templates_inventory.json`, `F_lithology_patterns.json`, `F_palettes.json`.
Tier: all Tier A (reference data — plot conventions, pattern names, colour ramps). Nothing here is patented/proprietary algorithm.

## 1. The `.plt` / `.trk` format (the headline finding)

IP's native plot template is **plain-text ASCII**, whitespace-delimited keyword records with `$` comments and CRLF —
**not XML, not binary**. This is the sharpest contrast with the Techlog ingest, where every template
(`LayoutTemplates`, `TemplateTracks`, `QuantiTemplates`, palettes, lithology catalog) was verbose Qt/`TLObject*`
**XML** (600+ line files per track, DOM-serialised widget state).

IP record grammar (one line per record):
- `TRACK  Width(in)  Grid  LogGrid  Decades  MinorGridStep  MinorGrid  ?  GridLines  [Overview]  [Name]`
- `CURVE  *Type|Name  LeftScale  RightScale  Backup(RBU/LBU/WRAP/NONE)  Thickness  Color  Log/Lin  PointPlot  [Visible]`
- `SHADE  *curveA ref  *curveB ref  FillFlag  Color|Variable[pal]  :>:Label`
- `ZONES` / `TOPS` / `ORDER` / `PARAS` / `FUNCS` / `GRID` / `PRINTOUT`

Key semantics worth adopting:
- **Curve TYPE indirection**: a leading `*` means the token is a *curve family* (`*GammaRay`, `*Density`, `*Neutron`,
  `*DeepRes`, `*Sw`, `*Vcl`, `*Phi`, `*Perm`…), resolved to an actual mnemonic through the alias system (Target A).
  A `*` in a scale field means "use the tool/family default scale." So a template is **data-agnostic** — bind by family,
  not by literal curve name. SandiBumi's composite.rs should do the same (track binds a family, alias resolves the curve).
- **`SHADE` is a three-mode fill rule** on a single line: (a) solid named colour, (b) **lithology-named fill**
  (`Clay`, `Silt`, `Sandstone`, `Blank`) — these names tie straight into the shading-bitmap set, and (c) **gradient fill
  via a `.pal` palette** (e.g. `Variable *Vcl … Earth.pal`). Between-curve fills reference `*curve Curve` (the curve line)
  or `*curve 0.` (a constant) as the two bounds. This one primitive covers D-N sand/shale crossover shading, Sw→BVW
  hydrocarbon fill, and the cumulative Vcl/Vsilt/Phi lithology column.
- **`ORDER`** fixes the per-track render stack (GridLines → Pictures → Images → VDL → VertLines → Waveform → Shading →
  Curves → PointPip → Tadpoles → Zones → Numerics → DIPImage). A direct z-order spec for SandiBumi's track compositor.
- **17 named colours only** (Aqua, Black, Blue, DkGray, Fuchsia, Gray, Green, Lime, LtGray, Maroon, Navy, Olive, Purple,
  Red, Teal, White, Yellow). Continuous colour lives in `.pal` palettes, not inline.

**Portability verdict: HIGH.** The schema is deterministic and trivially parseable (a 20-line tokeniser). Each
`TRACK`→column, `CURVE`→trace, `SHADE`→fill, `GRID`→depth-grid, `ORDER`→z-stack maps 1:1 onto a composite-track model.
`.trk` files are single-track fragments (`Blank-lin`, `Depth`, `Resistivity`, `Neu-Den Shaded`, `CPI Vol`, `CPI Sw`…) —
**drop-in reusable building blocks**, exactly the granularity SandiBumi wants for a track library. This is a much easier
adoption target than Techlog's XML; recommend SandiBumi model its track/scale/fill DSL on the IP `.plt` grammar
(text, family-bound) rather than Techlog's DOM dump.

Two dialect notes: the older Default-Plots `.plt` (e.g. `Triple Combo.PLT`) use the short TRACK header; the
Composite/Geomechanics `.plt` add a `Lin/Log` token, a trailing track Name, richer `ORDER`, and **module-coupled**
curve refs like `PP1InputCurves:GeoInCurve_ShaleDiscriminator` with `PARAS`/`FUNCS` records binding the track to a
module parameter set — those are less portable (tie a track to a specific IP module run).

## 2. Default Plots (117 files) — what SandiBumi's default track/scale conventions can adopt (v1-core)

The v1-relevant petrophysics templates:
- **`Composite CPI.plt`** — the canonical openhole CPI. Track order + scales are a ready-made SandiBumi default:
  GR 0–150 / SP −200–200 (Earth.pal GR-shaded) · Index+TVD · Res 0.2–20 log (Deep/Med/Micro) · Caliper 6–16 mirror ·
  Density 1.95–2.95 / Neutron 0.45–−0.15 / Drho −1–0.25 / Sonic 140–40 with sand(Yellow)/shale(Green) crossover shade ·
  Temp/RhoMatrix/CoreGD · Res+Pay flag track · **Sw 1→0 with hydrocarbon(Red) shade** · PhiT/Phi/BVW/CorePhi 0.5→0 with
  hydrocarbon fill · **Vcl/Vsilt/Phi cumulative lithology column** (Clay/Silt/Sandstone/Porosity fills) · Perm 0.1–10000 log.
- **`Triple Combo.PLT`** — GR-Cali-SP / Res (0.2–2000, 4-decade log) / D-N (2.95–1.95, N 0.45–−0.15) baseline.
- **`Density Neutron - Shading.plt`** — the standard D-N limestone-compatible overlay with Yellow/Green crossover fill.
- Reusable `.trk` blocks: `CPI Gr / Phi / Sw / Perm / Vol / Ntg / Rhoma`, `Neu-Den`, `Neu-Den Shaded`, `Resistivity`,
  `Sonic`, `Gr-Cali-SP`, `Depth`, `Blank-lin/log`.
- **11 `.svg`** header/footer/title/logo templates (`Standard`, `Full`, `Minimum`, `Detailed Log Header`,
  `Standard Footer`, multiwell + two-logo variants) — "limited SVG"; reusable as page-furniture reference.
- **26 `.xpt` cross-plots** (XML, `urn:PGL/IntPetro/Xplot`; X/Y/Z1/Z2 axes, log flags, overlays), grouped Core / Fluid /
  Lithology / Parameter / Porosity / Saturation. Includes the standard interpretation crossplots SandiBumi will want:
  **Pickett, Buckles, Neutron-Density, Sonic-Density (+matrix), U-Density matrix (M-N/MID lithology), Vshale-Phit,
  Pe-Density, Compress-Shear, Density/Neutron/Sonic-Rt, Rt-Rxo, RwApp**.

**"later" bucket inside Default Plots** — the dip/image-log & structural plot families (image-log domain, not v1):
`.vwp` (dip-angle/azimuth vectors ×8), `.dsp` (histogram/scatter ×14), `.snp` (stereonet ×8), `.dpp` (dip-azimuth/rose ×6),
`.rpc` (rose/cross-section ×6), `.cdp` (cumulative dip ×4), `.dhp` (image histogram ×2), `.pcf` (pie ×2), `.wcs`/`.wdcs`
(well cross-section ×2 each). Tag **later** (goes with NMR/image-log phase).

## 3. Geomechanics Plot Formats (38) — tag "later"

32 `.plt` + 5 `.xpt` + 1 `.hst`. Pore-pressure & sanding/wellbore-stability result plots (`Wellbore Single Well PP
Result`, `Vertical Stress`, `Elastic Moduli`, `Shear Failure`, `UCS`, `Sand Result`; `Sanding … TWC/UCS/HStress`;
multi-well "1 well format" variants). Pore-pressure/geomechanics → **later**, not v1 openhole. These `.plt` are the
module-coupled dialect (`PARAS`/`FUNCS`, `MPnInputCurves:GeoInCurve_*`).

## 4. Field Plot Templates & Reports

- **Field Plot Templates (2)**: `Field Plot.cplt` + `Field Plot/LogPlot.plt`. `.cplt` is a **.NET DataContract XML**
  wrapper (`SSL.Common.LogPlotComposer`) that references child `.plt` files — a composite-of-composites (multi-panel page
  layout). Concept is portable (a page that arranges plot panels) even though the wrapper format is .NET-specific.
- **Reports (2)**: `CasingInspectionReport.rdlc`, `CementReport.rdlc` — Microsoft **RDLC** (SSRS client report XML),
  both **cased-hole** → scope **OUT**. IP's openhole "report" output is driven from the plot/xpt templates, not RDLC.

## 5. Shading Bitmaps (162) — the lithology pattern set (Tier A names)

Format: 8-bit `.bmp` tiling fill textures (monochrome/greyscale hatch), + 1 `.emf` and 1 `.ppt` legend, + `CustomBrushPatterns`.
Two overlapping sets:
- **Legacy top-level set (63 bmp)** — the names the `.plt`/`.trk` `SHADE` records reference by keyword
  (`Sst`, `Shale`, `Silt`, `Dolomite`, `Lst`, `Anhydrite`, `Halite`, `Salt`, `Coal`, `Chert`, `Marl`, `Chalk`, `Tuff`,
  `Igneous`, `Basement`, `Conglomerate`, plus qualified variants `Sst_calc/silty/argill/carb/volcanic/tuffaceous`,
  `Dol_sandy/muddy`, `Lst_sandy/muddy`, image-quality glyphs).
- **Curated `*_final` set (96 bmp)** organised by rock class — the modern, complete taxonomy SandiBumi should adopt as
  its pattern name set:
  - `clastic_final` (27): Sandstone bedded/crossbedded/massive/ripple/calcareous/dolomitic/shaly; Shale silty/calc/
    carbonaceous/cherty/dolomitic/oil; Clay 1/2 + bentonite/glauconite/limonite/siderite/underclay; Siltstone; Silt;
    Conglomerate; Breccia.
  - `carbonate_final` (24): Limestone (argillaceous/cherty/clastic/crossbedded/dolomitic/fossiliferous/nodular/oolitic/
    sandy/silty); Dolomite (argillaceous/cherty/oolitic/sandy/silty); Chalk; Subgreywacke; Diatomaceous; Fossiliferous.
  - `others_final` (18): Anhydrite 1/2, Gypsum 1/2, Halite, Salt, Chert bedded/fossiliferous, Flint, Peat, Impure Coal,
    Loess, Till, Phosphatic-nodular, and 4 **Interbedded** patterns (Lst-Shale, Sand-Shale, Sand-Shale rippled, Sand-Silt).
  - `igneous_final` (18): Granite, Basalt Flows, Crystal/Devitrified/Tuffaceous tuff, Porphyritic 1/2, Vitrophyre,
    Volcanic Breccia 1/2, Zeolitic, Quartz, Schistose/Gneissoid Granite, Igneous 1–5.
  - `meta_final` (9): Gneiss(+contorted), Schist(+contorted), Schist+Gneiss, Quartzite, Slate, Serpentinite/Talc, Metamorphism.

Recommendation: adopt the **`*_final` name taxonomy** (non-protectable lithology vocabulary) but **redraw as vector/SVG
hatch patterns** rather than shipping the bitmaps — cleaner at any DPI and licence-clean. Full name lists in
`F_lithology_patterns.json`.

## 6. Palettes — 82 top-level `.pal` (+ 141 in the Techlog set for comparison)

Format: plain-text INI, `index=packedColor`, **256 entries (0–255)**. `packedColor` is a **Windows COLORREF integer
(0x00BBGGRR)** — confirmed by probing `WhiteRed.pal` (last entry decodes to pure red only under COLORREF). Decode:
`R = v&0xFF, G = (v>>8)&0xFF, B = (v>>16)&0xFF`. Continuous ramps for image-log / Z-axis / facies colour mapping.
Fully portable — decode to RGB triplets.

Vendor grouping (see `F_palettes.json`): **generic 44** (Earth, Rainbow_Bright/Soft, Spectrum, HeatMap, Grayscale(+rev),
BlueRed, RedGreen, Pastel, Contour#1/2, Gamma_Facies, SandCount6/7, tristate(+rev)…), **Baker Hughes 27**
(acoustic/CBL-VDL/NMR/triaxial/geochemical), **Halliburton 6**, **Schlumberger 3** (incl. OBMI-LGC), **Weatherford 1**,
**Geoactive/PGL 1**. Verified swatches: Grayscale white→black; Earth pale-yellow→orange→dark-red; Schlumberger classic
FMI white→orange→dark; tristate blue/orange/green 3-facies.

For SandiBumi `composite.rs` **default colour conventions**, the useful, brand-neutral picks: `Earth.pal` (GR/Vcl gradient
shade, matches the `SHADE … Earth.pal` usage), `Grayscale`/`Grayscale_Reversed` (image logs), `Rainbow_Bright`/`Spectrum`
(Z-axis crossplot colouring), `Gamma_Facies`/`tristate` (discrete facies). Skip the vendor-branded ramps as defaults.

## 7. Techlog vs IP — same-class comparison (for the merge)

| Class | Techlog 2018.2 | IP 2025.3 | Note for SandiBumi |
|---|---|---|---|
| Plot/track template format | Qt/`TLObject*` **XML**, 600+ lines/track, DOM widget dump | **plain-text ASCII** keyword records, family-bound | IP far more portable; model SandiBumi DSL on IP `.plt` grammar |
| Template granularity | LayoutTemplates(360) + TemplateTracks(74) split | Composite `.plt`(10) + reusable `.trk`(16) fragments | both give per-track building blocks; IP's are terser |
| Curve binding | by family/alias | by `*Type` family + `*` default scale | same concept, IP more explicit inline |
| Cross-plots | `.xml` (7) | `.xpt` XML `urn:PGL/IntPetro/Xplot` (26) | IP ships more standard interp crossplots (Pickett/Buckles/M-N…) |
| Lithology patterns | 349 assets (16 xml + 333 png) | 162 bmp, `*_final` taxonomy (96) + legacy (63) | adopt IP `*_final` **names**, redraw vector; Techlog already png |
| Palettes | 141 `.xml` | 82 `.pal` INI COLORREF, 256-stop | IP simpler to parse; fewer but covers the needful ramps |
| Headers/footers | `.xml` headers (21) | `.svg` (11) | IP SVG is directly web/SandiBumi-friendly |
| Reports | (n/a in Target F extract) | 2 `.rdlc` (cased-hole, OUT) | neither is a v1 need |

**Bottom line for SandiBumi v1:** take (1) the `Composite CPI.plt` track/scale/shade layout as the default openhole CPI,
(2) the `.plt`/`.trk` **family-bound text grammar** as the template DSL model (over Techlog XML), (3) the `SHADE`
three-mode fill primitive, (4) the `*_final` lithology-pattern **name set** (redrawn as SVG hatch), and (5) `Earth` /
`Grayscale` / `Rainbow_Bright` / `tristate` as brand-neutral default ramps (COLORREF-decoded). Defer geomechanics (38),
dip/image plots (~50 in Default Plots), and cased-hole RDLC reports.
