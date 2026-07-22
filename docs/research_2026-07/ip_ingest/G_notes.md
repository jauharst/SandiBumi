# Target G — IP2025 data model, reference tables, cross-tool bridges

Source (read-only): `C:\Program Files\IP2025`. Calibration: Techlog `G_import_configs.json`.
All extracted content is Tier A (catalogs / lookup tables / structural facts) or Tier D
mechanism prose (bridges) — no algorithm code, no decompilation.

## Key finding: the 6 XSDs are NOT the project data model
The prompt expected the 6 top-level `.xsd` files to "define IP's data model". They do not.
They are **per-feature tool-configuration schemas**:
- `CasedHoleTool.xsd` — cased-hole (CBL/SBT/impedance) tool input-curve map (scope: out)
- `FormationTestingTool.xsd` — wireline pretest/RFT/MDT probe-drawdown model (scope: later)
- `ImageTool.xsd` + `ImageToolPad.xsd` — borehole-image/dipmeter/caliper tool geometry (scope: later)
- `SanitizingSchema.xsd` — code/unit sanitizing map for import normalisation (scope: v1-core concept)
- `UserProgram.xsd` — user-module **menu manifest** (not algorithm)

IP's real project data model is the **compiled .NET object model**, documented in
`PGL.IP.API.xml` (861 KB XML-doc). I recovered it from there → `G_datamodel_schemas.json`.

## The IP object model (the actual answer to "how IP structures a project")
`Database → Well → CurveSet → Curve → LogReading`, plus per-Well
`ZoneSet → Zone → Parameter`, `Discriminators`, `ImagePickSets`, header/location/identity/datum.
- **CurveSet** = a group of curves sharing ONE depth curve + spacing (`IsIrregular` for uneven).
  This is IP's word for Techlog "dataset" / Geolog "set" / SandiBumi curve-set.
- **Curve** carries rich provenance (create/update user+module+date, FinalVersion, CurveStatus,
  Locked) and explicit array dims (`XArraySize/XDimension/YDimension`) for waveform/image.
- **Zone.Parameters** — interpretation constants (Rw/m/n/a/cutoffs) live per-zone, not global.
- **ICurveStatistics** ships `Percentile1/2/3` — aligns with the GR P3/P97 normalisation standard.
- Enums worth adopting verbatim: `DepthReferenceType {MD, TVD_KB, TVD_GL, TVD_SS, TVD_SB}`,
  `DataType {Double/Single/Short/UShort/Byte/SByte/Binary16/String}`, `DepthCurveValueOrder`.

This maps 1:1 to SandiBumi's DuckDB `project>well>curveset>curve` and validates the current design.

## Reference tables (`G_reference_tables.json`) — bigger than Techlog's
- **CasingSizes** 168 rows (OD/ID/weight/thickness, imperial+metric) — full API casing catalog.
- **HoleSizes** 22 bit sizes; **CasingProgram** 21 hole↔casing telescoping-default pairs.
- **PaperSizes** 68 (ANSI/Arch/ISO/DIN/JIS + oversize) for plot output.
- **FTOutputCurves** 22 — formation-tester output-curve catalog (drawdown/spherical/radial
  permeability & mobility, P*) — scope: later.
- **UnitConfig** (curve-type→unit-category) + **IP↔Petrel unit-string map** (28 rows).

## Bridges (`G_geolog_openworks_bridges.json`)
- **Geolog / OpenWorks DB links** (`PGLWowlnk`, `PGLWdblnk`): heavyweight legacy Unix/Solaris
  CGI-over-Apache-or-SSH bridges to Geolog6 / Landmark OpenWorks / Oracle. Requires a Geolog6
  license. Mechanism only — **not a model for SandiBumi interop.** Tier D.
- **The usable Geolog interop is file-level:**
  - `GeologASCII_options.txt` — tops-set recognition prefixes, Geolog→IP set-name aliasing, and
    the depth-mnemonic candidate list `DEPTH/TDEP/DEPT/MD/INDEX/TVD`. Drop-in for a Geolog-ASCII
    importer. **v1-core.**
  - `GeologIPShadingMapping.txt` — 60 color + lithology-fill (CALC→Limestone, DOL→Dolomite,
    ILL, PYR, CM1→Clay) + colormap→palette mappings for layout import. (display / later)
- **DLIS import** (`DLISCurveAttributesMappings.configUpdate`) — per-vendor (Baker/Halliburton/SLB)
  axis/channel attribute → IP waveform-frame name map (sample-rate/start/stop/Rx-Rx spacing).
  Relevant if SandiBumi ingests DLIS sonic/image frames. **v1-core-adjacent.**
- **Language-migration configs** (`LlsToVbConfig.xml` Geolog Loglan→VB, `PowerLogToVbConfig.xml`
  PowerLog→VB, `IpClassicPythonlink.py`) — noted, not portable science. SandiBumi reimplements
  methods from primary papers.
- **Petrolog / Geosteering / WITSML (RequestedDataDescriptionLists)** configs noted; PetrologSets
  ships empty; WITSML PTK realtime channel lists are scope: out.

## vs Techlog G
Techlog exposed its model as thin property-alias XMLs (`Dlis.xml`/`Las.xml`/`Geolog.xml`/
`propertyDict.xml`). IP exposes a full documented object model (`PGL.IP.API.xml`) — a much
richer, directly comparable data-model reference — plus far larger casing/hole/paper reference
catalogs. The Geolog naming bridge is confirmed in both: IP CurveSet == Techlog Dataset ==
Geolog Set.
