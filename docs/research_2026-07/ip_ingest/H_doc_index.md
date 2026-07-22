# IP 2025.3 — Scripting/Formula API + Documentation Index (Target H)

Source (READ-ONLY): `C:\Program Files\IP2025\`
Publisher: PGL / Lloyd's Register / Geoactive. Main exe `Intpetro.exe`.
Prepared: 2026-07 ingest, for SandiBumi. Machine-readable extract: `H_api_surface.json`.

---

## 0. Headline finding (read this first)

**IP documents its programmable surface as TEXT, not images — the opposite of Techlog.**
The API is delivered as .NET XML-doc comments (`PGL.IP.API.xml`, 861 KB, 2,323 doc members,
235 public types), i.e. every class/method/property carries a plain-text `<summary>`. That is
strictly better than Techlog's `Doc\`, where method equations are rasterized GIF/PNG images that
must be transcribed by eye (see `techlog_ingest\H_doc_index.md` §1: "the math must be read from
the image").

**But a second, equal-and-opposite finding:** IP's public API is an **automation + data-access
API, NOT an equation library.** `PGL.IP.API` exposes curves, curve-sets, wells, zones, parameters,
units, statistics, plots and reporting — it does **not** expose Archie / Vsh / porosity / Sw / Sw
math functions. In Techlog those DO exist as first-class documented Python callables
(`TechlogQuanti.*`, 2,248 pages under `topic\pythonlib\`). In IP, the FE math is compiled into
module DLLs (`UserProgram.dll` per module) and is reached from user code only through:
  1. the **Formula language** (`.frm`), and
  2. the **UserProgram / ip2py** on-ramp (C#, VB, IronPython, CPython).

So for equation-level method reference, IP is WORSE than Techlog (no documented Sw/Vsh/porosity
function catalog); for API/automation ergonomics and for text-vs-image documentation, IP is BETTER.
Net: the two ingests are complementary — cite Techlog's `concept\petrophysics-*.html` for the
method math (still Tier B, reimplement from primary papers), cite IP here for the
curve/well/zone/parameter object model a SandiBumi Python/formula layer should mirror.

---

## 1. Documentation tree structure

| Path | Bytes / count | Role |
|---|---|---|
| `PGL.IP.API.xml` | 861 KB, 2,323 members | Full .NET API doc-comments (TEXT). The parseable surface. Parsed → `H_api_surface.json`. |
| `ApiDocumentation\IP Object Model Reference.chm` | 2.5 MB (2019) | Compiled HTML-Help rendering of the SAME `PGL.IP.API` object model. Content mirrors the XML (member summaries). Not separately transcribed — the XML is the machine-readable equivalent. (CHM decompile needs an interactive GUI shell; not required since XML carries the same text.) |
| `ApiDocumentation\Examples\StandaloneApps\` | 8 demos | Out-of-process automation clients: C#, VB.NET, C++ (native exports), JavaScript (JScript/ActiveX), PowerShell, MATLAB, Excel (.xlsm). Show the COM/`IntPetro.API` automation entry path. |
| `ApiDocumentation\Examples\UserApps\` | ~45 UserProgram demos × {CS, VB, IronPython, Python(CPython), ip2py} | In-process **UserProgram** examples — the real "formula/algorithm on-ramp". Each has `Parameters` (UI/curve/param definition, `~V19` format), `UsersCode.*` (the user math), `UserHelp.md`, `Iplink.*`/`Methods.*` (generated proxy glue). |
| `Jupyter\Examples\` | 4 `.ipynb` | Curves, Parameter & Zone Sets, Wells, matplotlib — ip2py used from Jupyter. |
| `Formula\Empty.frm` | header only | Formula template. See `H_formula_language.md`. |
| `PL\*.frm` | 3 files | Real formula examples (Production-Logging). Syntax evidence, see `H_formula_language.md`. |

No `concept\`-style equation-page corpus exists in IP (unlike Techlog's 1,417 concept pages). Method
descriptions in IP live in per-module help + the module `Parameters` text files, not a central
equation library.

---

## 2. `PGL.IP.API.xml` — parsed surface (see `H_api_surface.json` for full member lists)

235 public types, categorized:

| Category | #types | What it is | SandiBumi relevance |
|---|---|---|---|
| `petro_data_core` | 41 | Curve/CurveSet/CurveStatistics/LogReading, Well/Zone/ZoneSet, Parameter, Unit, Discriminator, Database, attribute enums | **This is the surface to mirror.** Curve access, statistics, zone/parameter model. |
| `ui_plotting` | 88 | LogPlot / CrossPlot / Histogram object model | v1: log plot + crossplot parity only; most is UI-depth detail. |
| `misc_data` | 58 | enums, indexers, iterators, well identity/location, coordinate systems | supporting types |
| `internal_infra` | 34 | `*.Internal.*`, DllExport attributes | ignore (not public contract) |
| `services_infra` | 9 | `Services.*` (config, event aggregator, file path) | infra |
| `reporting` | 3 | `ReportGenerator.*` report/plot export | later |
| `audit` | 2 | AuditLogging event types | later |
| `image_analysis` (in petro_core count) | 2 | `ImageAnalysis.IDipInformation`, `IImagePickSet` | later (image logs) |

### 2.1 Petrophysically-meaningful classes a SandiBumi formula/python layer must match

**Curve data access**
- `ICurve` (101 members) — the log curve. Key surface: `DisplayName/FullName`, `Units`,
  `MeasurementUnit`, `CurveType`, `CurveDataType` (Numeric/Text), `DataType` (Single/Double/Short/
  Byte/Binary16…), `TopDepth/BottomDepth`, `NumberOfValues`, `XDimension/YDimension` (arrays),
  `IsPackedArray`, `Statistics`, `Backup`, plot props (`LScale/RScale/Logarithmic/Color/LineWidth/
  LineStyle`), `TvdRefCurve*`, `DepthShiftInSamples`; methods `CMin/CMax`, `GetIndex` (7 overloads),
  `FillGaps`, `NullCurve`, `ContainsOnlyNullSamples`, `IsPointCurve`, `Update`, attribute get/set.
- `ICurveSet` (39) — curve container / depth frame: `Curves`, `DepthCurve`, `Spacing`,
  `NumberOfDepthSamples`, `IsIrregular`, `NewCurve`, `NewArrayCurve`, `AddIrregularDepth(s)`,
  `ChangeDepths`, `ChangeIndex`, `MergeWith`, `GetGroupedCurves`. (Irregular = point/core data.)
- `ICurveDataIterator` (69) — the per-sample read/write cursor (typed getters/setters, array support).
- `ILogReading`/`ILogReadings`/`ILogReadingsBase` + `IArrayLogReading(s)` — reading abstraction,
  incl. array/waveform curves.

**Statistics** — `ICurveStatistics` (14): `Minimum, Maximum, Average, StandardDeviation, Median,
Mode, NetInterval, NullCount, DiscriminatorFailCount, Top, Bottom, Percentile1/2/3`.
→ This is the ONLY built-in "math" in the API. Note the 3 configurable percentiles — directly
usable for GR normalization (Jauhar's P3/P97 workflow) without a separate module.

**Zones & parameters (the parameter-set model)**
- `IZone` (28), `IZoneSet` (42), `IZoneSetManager` (20), `IZoneAttribute(s)` — zone tops/bottoms,
  splitting (`SplitZoneResult`, `SetTopBottomOptions`), attributes.
- `IParameter` (11): `ParameterType` ∈ {Text, Boolean, Numeric, CurveName}, plus
  `NumericValue/BoolValue/TextValue/CurveNameValue/CurveNumberValue`, and `Zone` (per-zone params).
  → IP parameters are **zone-aware** (a param can carry a value per zone). SandiBumi's parameter
  layer should support the same per-zone override shape.

**Units** — `IUnit`, `IUnitCategory`, `IConversionBase` (unit conversion); `CurveUnits` enum is only
the 8 **index** units {NONE, METERS, FEET, SECONDS, MILLI_SECS, DATE_TIME, POINT_ONE_INCH, LEVEL} —
measurement-unit conversion is the `IUnit`/`IUnitCategory` service, not this enum.

**Discriminators** — `IDiscriminator` (13), `IDiscriminatorSet` (8): IP's cutoff/flag mechanism
(net-pay style boolean filters over curves); `ICurveStatistics.DiscriminatorFailCount` ties in.

**Well / DB** — `IWell` (82), `IWellIdentity`, `IWellLocation` (13, incl. coordinate systems,
`NorthReference`, `PermanentDatum`), `IZoneSetManager`; `IDatabase`/`IDatabaseConnection`/
`IDatabaseFactory`/`IDatabaseConnectionFactory` (file-based project DB open/connect).
`DepthReferenceType` ∈ {MD, TVD_GL, TVD_KB, TVD_SB, TVD_SS, Undefined}.

**Attribute catalogs (text keys)** — `WellAttributes` (52), `LogAttributes` (59),
`CurveAttributes` (30): the well-header / log-run / curve metadata field names. (Complements
Target A's mnemonic/alias catalog — these are the header attribute keys, not curve mnemonics.)

**Entry points** — `IIntPetroAPI` (`GetService`, `Exit`), `IntPetroAPI` static; COM ProgID
`IntPetro.API`. `IServiceContainer.GetService("PGL.IP.API.<IServiceName>")` is the service locator.

### 2.2 What is NOT here (and where SandiBumi must get it instead)
No Archie/Simandoux/Indonesia/Waxman-Smits/Dual-Water, no Vsh (Larionov/Clavier/Steiber), no
porosity (density/neutron/sonic), no Thomas-Stieber, no hydraulic-flow-units, no mineral solver in
`PGL.IP.API`. Those are compiled modules. Method-level reference for reimplementation → use the
**Techlog H index** (`techlog_ingest\H_doc_index.md`) for the equation catalog + primary-paper
citations, and SandiBumi's own method memory notes. IP contributes the *object model* + *parameter/
zone model* + *formula on-ramp*, not the equations.

---

## 3. Equation-/method-level docs found in IP (thin)

- **ip2py `calculations` module** — the one place IP ships a named FE helper reachable from user
  Python: `calculations.gamma_ray_index(gr_min, gr_max, gr, apply_limits=True)` = linear GR index
  (Vsh). Used in `Examples\UserApps\ip2py\UserPrograms\ip2py_clay_volume`. The `calculations` module
  is shipped as embedded binary (no `.py` source in the install tree), so its full function list is
  not enumerable from files — only `gamma_ray_index` is evidenced. Tag: Tier A method (linear GR
  index is public/trivial); note the module likely has more but is not source-inspectable.
- **Per-module `Parameters` files** carry each module's default constants as text (`~V19` format:
  input curves, numeric params with min/max/default/decimals, text/flag params, zone toggle,
  compiler). These are the intended "read the readable Parameters text for defaults" path per the
  ingest rules. (Target C/E territory for the deterministic modules — noted here as the doc mechanism.)
- **`SandTyping` UserProgram example** (`Examples\UserApps\CS\UserPrograms\SandTyping`) — a worked
  sand-typing algorithm in readable C#; useful as an on-ramp pattern, not a protected method.

No central, per-method equation pages (Techlog's strength) exist in IP.

---

## 4. IP vs Techlog documentation — direct comparison (per calibration request)

| Dimension | Techlog 2018.2 | IP 2025.3 | Winner |
|---|---|---|---|
| How equations are documented | **Rasterized images** (GIF/PNG) inside `concept\petrophysics-*.html`; must be eye-transcribed | Text doc-comments — but there is **no equation corpus at all** in the API | Techlog *has* the equations (as images); IP does not document them |
| Method catalog (Sw/Vsh/φ/TS/FZU/solver) | ~1,417 concept pages + `quanti-elan-theory` chapter; explicit per-method pages with parameter tables | None in API; math is compiled in modules | **Techlog** for method reference |
| Programmable API doc | `topic\pythonlib\` 2,248 pages, one per `TechlogQuanti.*` callable (incl. FE math) | `PGL.IP.API.xml` 2,323 members, TEXT, one per type/member (data/automation only) | Tie on form (both text); Techlog wins on FE-math coverage |
| API delivery format | HTML pages (DITA) | .NET XML doc-comments + CHM | IP (single parseable XML) is easier to ingest |
| FE math callable from user script | **Yes** — `TechlogQuanti.archie(...)` etc. documented | **No** — user must re-implement in Formula/UserProgram (`ip2py.calculations` has only GR index) | **Techlog** |
| Automation / app-embedding API | Python-centric | Rich COM/.NET automation (C#/VB/C++/JS/PS/MATLAB/Excel/Jupyter) + in-process UserPrograms | **IP** |
| Parameter model | family/alias driven | zone-aware `IParameter` + `IDiscriminator` cutoffs + 3 configurable percentiles in stats | IP's zone-aware params + built-in percentiles are a nice model to copy |

**Take-away for SandiBumi:** mirror IP's *object model* (ICurve/ICurveSet/IZoneSet/IParameter with
per-zone values, IDiscriminator cutoffs, ICurveStatistics with configurable percentiles) and its
*multi-language on-ramp* (formula string + user-python), but source the *equations* from Techlog's
concept pages + primary papers, never from either tool's wording.
