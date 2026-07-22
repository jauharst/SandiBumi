# IP 2025.3 — User-Program / Formula Language (Target D)

**Source (read-only):** `C:\Program Files\IP2025\`
**Date:** 2026-07-22
**Purpose:** Describe IP's user-program / formula-scripting surface (IP's analogue of Geolog Loglan and Techlog Python) from its OPEN-TEXT SDK sources, so SandiBumi can (a) understand how IP users extend the app, and (b) offer a comparable "user formula / custom curve" capability. No compiled algorithm was decompiled.

> **Legal framing.** Everything in this file is language/API structure read from IP's own shipped SDK *example* source (Differentiate, Interp_Demo, Normalize_array, the `Iplink.*` bindings, HFU `UserAppCode.cs`) plus module *manifest* text. These are reference/interface facts (Tier A/D-structure), not protected algorithms. The math inside IP's built-in Modules ships compiled and was not read.

---

## 1. What the "language" actually is

IP does **not** have a bespoke formula DSL like Geolog's Loglan. Instead a **User Program** is a snippet of a **general-purpose language** that IP wraps, compiles, and calls per depth level. The same user-program can be authored in **seven** front-ends; the manifest picks one via a `Compiler=` line:

| Compiler tag | Language | Compiled via | Example entry file |
|---|---|---|---|
| `VB` | VB.NET | .NET (Roslyn/vbc) | `UsersCode.vb` |
| `CSharp` | C# | .NET (csc) | `UsersCode.cs` |
| `C` | C (native) | native C compiler → DLL exporting `usercode_()` | `UsersCode.c` |
| `Fortran` | FORTRAN 77 | native → DLL, `SUBROUTINE UserCode()` | `UsersCode.f` |
| `Matlab` | MATLAB | MATLAB engine | `UsersCode.m` |
| (Python, classic) | IronPython | `ipy.exe` / `ipy64.exe` + `pyc.py` → .NET DLL | `UsersCode.py` |
| (Python, app) | IronPython w/ mixins | as above | `UsersPythonCode.py` |

Compiled output is `UserProgram.dll` inside each program's folder. `pyc.py` is the **Microsoft IronPython command-line compiler** (Apache-2.0, bundled) that turns Python user code into a .NET assembly.

There is a separate, **modern CPython** path for notebooks (§6) — that one uses real numpy/pandas over a COM bridge, not IronPython.

**Cross-tool import.** Two stub files, `LoglanToVb.vb` and `PowerLogToVb.vb`, show IP ships **translators from Geolog Loglan and Landmark PowerLog formulas into IP VB user programs**. (Directly relevant to SandiBumi/Geolog interop — IP treats Loglan as an importable source dialect.)

---

## 2. Execution model (the part SandiBumi must mirror)

Every user program is a partial class `IPLink` (or, in the mixin Python path, `class UserApp(Methods, IPLink)`) exposing one method **`UserCode()`**. The host:

1. calls `SetupParameters(...)` handing over input-curve handles, output-curve handles, numeric params, text params, boolean flags, array-curve dims, **top/bottom depth as integer INDICES**, zone number, total zones, and "parameter-curve" handles;
2. calls `Run()` → `UserCode()`.

The canonical body is an **index loop**, not a depth loop:

```
index = TopDepth            # integer index, not a depth in ft/m
while index <= BottomDepth:
    ... read curves/params at index ...
    Save_<Out>(index, value)
    index += 1
```

`Depth(index)` returns the actual measured depth; the well step is derived as `Depth(top+1) - Depth(top)`.

### Curves and parameters are all accessed as **functions of index**
- **Input curves** → `numCrv(index)`, `Gr(index)`, `Rt(index)`, `Den(index)`, `Son(index)` … (the function name is whatever the manifest names that input slot).
- **Numeric parameters** → also called **with an index**: `xa(index)`, `xm(index)`, `Rw(index)`, `GrClean(index)`, `DenMat(index)` …. This is IP's key design idea: **a parameter and a curve are interchangeable.** `GetParameterValue(p,index)` returns the curve value if the parameter was bound to a curve (`parCnIn[p]>0`), else the scalar. So a user writes `Rw(index)` and it transparently works whether Rw is a constant, a per-zone value, or a full curve. `index = -1` is a convention to fetch the current/zone scalar (used in the Pickett helper).
- **Text parameters** → `SwEq` ("Archie"/"Indonesian"), `PorEq` ("Sonic"/"Density").
- **Flag (boolean) parameters** → `Shellm` (e.g. toggle Shell variable-m).
- **Array / image curves** → `Array_InCrv(index, ix, iy)` with dims `Array_InCrv_MaxX`, `Array_InCrv_MaxY`.

### Writing results
- Scalar/curve out: `Save_<OutName>(index, value)` (e.g. `Save_Phi`, `Save_Sw`, `Save_DiffCrv`).
- Array out: `Save_Array_OutCrv(index, ix, iy, value)`.
- Provenance: `Save_<OutName>_Comments("text")` stamps who wrote the curve.
- Null value convention: **-999** (IP absent-value sentinel; matches SandiBumi/Geolog null discipline).

### Zones, wells, attributes
- `SetZone(n)`, `ZoneNumber`, `TotalZones`; the manifest `useZones=true/false` flag. Parameters resolve **per zone** (`ResetZoneParameters`).
- Well/log/curve metadata: `Read_Well_Attribute("WellName"|"Company"|"Field"|"KBElev")`, `Write_Well_Attribute`, `Read_Log_Attribute(name, runNo)`, `Read_Curve_Attribute(curveNo, attr)`, and a generic `Read_Text(flags, index, attr)` where **flags: 0=Well, 1=Log, 2=Curve**. Curve name/units are exposed as `<Slot>_Name`, `<Slot>_Units`.

### Interactive crossplot ↔ code coupling (notable)
The Interp_Demo program reads the **live Pickett-plot** end-point handles `PPphi1/PPphi2/PPres1/PPres2` (plus `Rwpick`,`Mpick`) and **back-solves m and Rw** from the two user-dragged points, then writes them back to parameters. So a User Program can be driven by an interactive plot and can push results back into it. SandiBumi's Pickett/Hingle plots should expose the same two-point → (m, Rw) round-trip.

---

## 3. The module manifest ("Parameters" file) — IP's `.info` analogue

Each program folder has a plain-text **`Parameters`** file (versioned `~V7` … `~V20`) that is the module's UI + I/O contract — the counterpart of Geolog's `.info`/`.paysum`. Structure observed:

```
~V7                       # manifest version
useZones=false            # zonation on/off
Compiler=VB               # which language front-end
140 , 14                  # grid sizes (max input slots, ...)
numCrv , Curve to differentiate ,          # inputslot: name, description, family/dependency
denCrv , 'With respect to' Curve , Depth    # 'Depth' = dependency family tag
... (fixed grid of empty input slots) ...
Zones
Parameters
Flags
... (numeric params / flags rows) ...
DiffCrv , Result Curve , ... , False , 1 , 1   # outputslot: name, desc, ..., isArray, minX, minY
$Tracks $Curves $Shade $XPlots                 # embedded plot/track layout
```

Each input curve carries a **dependency/family tag** (e.g. `Depth`, `Density`, `ComprSonic`) — this is the same family system used elsewhere in IP (cf. Target A). Output rows carry an **isArray** flag + array min-dimensions. The `$Tracks/$Curves/$Shade/$XPlots` tail means a module ships its own default display. `UserProgram.config` (small XML) places the module in the menu tree (`<menupath>`, `<caption>`, `<siblingnode>`, `<displayat>`).

---

## 4. The `IPLink` binding classes (API surface)

Top-level files define the base plumbing every user program inherits:
- `Iplink.cs` / `Iplink.vb` — `IPLink : IUserProgramExV2`; fields for input/output curves, numeric/text/flag params, in/out array dims, `parCnIn` (parameter-curve handles), top/bottom index, zone/total-zones; methods `SetupParameters` (two overloads incl. grouped curves), `ResetZoneParameters`, `SetupIpProxy(IIntPetEx)`, `Run`, `SetZone`, and the Well/Log/Curve attribute getters/setters. All curve/param math flows through an `IIntPetEx IPProxy` COM-ish proxy (`GetCurveData`, `SetCurveData`, `GetText/SetText`, `GetWellText`, `SetNumericParam`, …).
- `IpClassicPythonlink.py` / `Iplink.py` — the same contract for IronPython (`property(fget=…)` for TopDepth/BottomDepth/TotalZones/ZoneNumber; `_IPProxy` plumbing).
- `IpLink.m` — the MATLAB `classdef IpLink < handle` with the full get/set surface (`GetInputCurveData`, `GetInputCurveArrayDataChunk`, `GetParameterValue`, `SetParameterValue`, `GetWellText`, chunked array I/O via `COM_SafeArraySingleDim`).
- `IPlink.c` / `Iplinkc.c` — the C/native contract (`InOutDef.INC`, `usercode_()`, `Save_***(INDEX,VALUE)`, params as `RW()` functions).

`UsersCode.<ext>` at top level are the **blank skeletons** IP copies when a user creates a new program (empty index loop). The three `UserPrograms/*` folders are the **worked examples** (Differentiate, Interp_Demo, Normalize_array).

---

## 5. The worked examples (what each teaches)

| Example | Teaches | Method tier |
|---|---|---|
| **Differentiate** | minimal index loop, two input curves, one output, null guard, `_Comments`; same logic in all 7 languages | utility (Tier A) — see `D_readable_algorithms.json` |
| **Interp_Demo** | full quicklook (Vcl→φ→Sw→BVW→cutoffs→zonal averages), text/flag params, `SetZone` loop, well attributes, StreamWriter report, **Pickett back-solve** | Tier B methods (Archie/Indonesia/Wyllie/Shell-m/Pickett) — extracted in `D_readable_algorithms.json` |
| **Normalize_array** | 2D array/image-curve access + write, min-max scaling; both IronPython styles (`class IPLink` vs `class UserApp(Methods, IPLink)`) | utility (Tier A) |

---

## 6. Modern Python / Jupyter bridge (`ip2py`)

Separate from the IronPython user-program path, IP ships a **CPython + Jupyter** integration:
- **`C:\Program Files\IP2025\requirements.txt`** → `numpy, pandas, mpmath, ptvsd, pywin32, jupyter, jupyterlab`. So the bridge is **COM via pywin32**, data marshalled as **pandas DataFrames**, debug via **ptvsd**.
- **`Jupyter/Examples/*.ipynb`** import from a package **`ip2py`** with submodules: `jupyter`, `curves`, `calculations`, `wells`, `parameter_sets`, `zones` (and matplotlib plotting).
- Representative calls:
  - `jupyter.get_active_well()`
  - `curves.get_curve_list_from_curve_names(well, ['DEPTH','SGR','RHOB','TNPH'])`
  - `curves.get_curves_as_dataframe(well, curvelist)` → pandas DataFrame
  - `curves.set_curve_values(...)`, `curves.set_curve_values_at_depths(...)`, `curves.create_curve_set(...)`
  - `calculations.gamma_ray_index(...)` (a few canned petro calcs exposed to Python)
  - `wells.get_active_wellnames_list()`, `wells.get_well_latitude/longitude(...)`
- Pattern: pull curves → DataFrame → compute in numpy/pandas → push results back to IP. This is the intended "bring your own Python/ML" surface (and the host for the `MLNET` module).

---

## 7. Comparison to Techlog (calibration)

| Aspect | Techlog 2018.2 | IP 2025.3 |
|---|---|---|
| Scripting language | CPython (2.7-era) via `Techlog` package (Data/Engine/Plot/Utils) over compiled C-extensions | **7 front-ends** for user programs (VB/C#/C/FORTRAN/MATLAB/IronPython) **+** modern CPython/`ip2py` for notebooks |
| SDK examples readability | wrappers readable; **high-value math shipped compiled** (.pyc/.dll) | **SDK examples fully open source** (all 7 languages); IP's own built-in Modules shipped compiled (`UserProgram.dll`) |
| Compiled boundary | between thin wrapper and `EnvCorrPreProcessingPrivate.pyc` / `RockPhysics_EquationsLibrary.pyc` | between open SDK examples and the 65 built-in `Modules/*/UserProgram.dll` |
| Param/curve interchange | family-tagged variables | **every parameter callable as `param(index)`**, transparently scalar/zone/curve |
| Interactive-plot coupling | markers/zonation feed scripts | Pickett-plot end-points read/written directly from a User Program |
| Data marshalling to Python | Techlog dataset objects | **pandas DataFrame** via pywin32 COM (`ip2py`) |
| Cross-vendor formula import | — | **Loglan + PowerLog → VB translators** shipped |

**Net for SandiBumi:** IP's open SDK gives a clean, copyable *interface* pattern (index loop, `param(index)` scalar/curve interchange, `Save_*`, per-zone params, -999 nulls, family-tagged I/O, embedded default plot) without exposing any protected algorithm. The `ip2py` pandas pattern is the model for a SandiBumi "custom Python curve" feature; the Loglan-import stub confirms SandiBumi's Geolog-Loglan bridge is aligned with how a commercial tool treats Loglan (as an importable source dialect).
