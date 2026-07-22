# IP 2025.3 — Formula language + UserProgram on-ramp (Target H)

Purpose: let SandiBumi's `equations.rs` / python formula layer offer an on-ramp that IP-trained
users recognise. Two distinct programmable surfaces exist in IP; both are described below.
Evidence: `Formula\Empty.frm`, `PL\*.frm`, and `ApiDocumentation\Examples\UserApps\*`.
Tier: this is syntax/format fact (Tier A) — no protected algorithm is reproduced.

---

## A. The interactive **Formula** language (`.frm`)

IP's "Formula" module is a single-expression curve calculator: the user writes one expression
that computes one output curve, optionally gated by a condition and a depth range. Formulas are
saved as `.frm` files (fixed-width text). This is the closest analogue to a Techlog "equation" cell
and the thing an IP-trained user reaches for first.

### A.1 File format (`.frm`)

`Formula\Empty.frm` is the template and contains only the version header:

```
~V4  Formula Parameters
```

Real saved formulas (evidence: `PL\Press_inte.frm`, `PL\Absent Data.frm`) are fixed-width column
blocks. Reconstructed layout:

```
Line 1  <cond-low>   <flag/TRUE>   <cond-high>      # conditional-application row (when to apply)
Line 2  <expression>                                 # the formula expression itself
Line 3  (blank)
Line 4  <OUTPUT_MNEMONIC>   <UNIT>                    # output curve name + unit
Line 5  <TOP>   <BOTTOM>                              # optional depth range (MD), e.g. "TOP.  BOTTOM"
```

Worked examples (verbatim expression only):
- `Press_inte.frm`: expression `5000 + (TVD - 8200) * .44` → output curve `PRESI1R4`, unit `psia`.
  (A pressure-vs-TVD gradient: 0.44 psi/ft below 8200 ft datum.)
- `Absent Data.frm`: expression `-999` → output `SPINP1D1`; condition row uses `TRUE`; range
  `TOP.`/`BOTTOM` — i.e. write the absent value `-999` over the whole interval.

Notes:
- Output mnemonic and unit are stored explicitly with the formula (the formula "knows" its result
  curve name + unit).
- `~V4` is the current on-disk version tag; older `PL` formulas were saved without the header line
  (version drift — a parser should tolerate both a leading `~Vn` header and its absence).

### A.2 Expression syntax (evidenced + standard IP formula grammar)

Evidenced from the samples:
- **Curve mnemonics used as variables**: a bare identifier in the expression is resolved to a
  curve by mnemonic (e.g. `TVD`). SandiBumi equivalent: identifiers bind to loaded curves.
- **Reserved/derived variables**: `TVD` (true vertical depth) is available as a variable even when
  not an explicit input curve. `DEPTH`/MD is likewise a reserved index variable in IP formulas.
- **Arithmetic operators**: `+  -  *  /` and grouping with `( )` (all shown), plus `^` for power in
  IP's grammar. Standard precedence.
- **Numeric literals**: leading-dot floats accepted (`.44`), integers, negatives (`-999`).
- **Absent/null sentinel**: `-999` is the conventional missing-value literal (matches IP's default
  absent value); a formula can both test for and write it.
- **Conditional application**: the leading row is a "when" gate (a comparison / `TRUE`) that
  restricts which samples the expression writes; combined with the TOP/BOTTOM depth window this
  gives IP formulas their `if condition then output` behaviour without an inline `if`.

Standard IP formula functions an IP-trained user will expect the on-ramp to accept (provide these
in SandiBumi's formula evaluator for parity; names are IP-conventional, not reproduced from any
protected source): `log`/`log10`, `ln`, `exp`, `sqrt`, `abs`, `int`, `frac`, `min`, `max`,
`if(cond,a,b)`, trig `sin/cos/tan/atan`, and comparison/logical operators `< <= > >= = <> and or
not`. (Only `+ - * / ( )`, `^`, bare-mnemonic variables, `TVD`, and `-999` are directly evidenced
in the shipped `.frm` files; the rest is the well-known IP formula function set — implement as a
compatibility layer, verify against a live IP if exactness matters.)

### A.3 Design guidance for SandiBumi `equations.rs`
- Accept an expression string with bare curve mnemonics as free variables; resolve against the
  active well's curve set (case-insensitive, alias-aware — reuse Target A aliases).
- Reserve `DEPTH`/MD and `TVD` as auto-available variables.
- Support an optional application predicate + depth window (mirror the `.frm` condition row + TOP/
  BOTTOM), so an IP formula maps 1:1.
- Carry the output curve name + unit as part of the formula object.
- Use `-999` (configurable) as the absent sentinel and make absent-in → absent-out the default.

---

## B. The **UserProgram** on-ramp (C# / VB.NET / IronPython / CPython / ip2py)

For anything beyond one expression, IP users write a **UserProgram** — a compiled/interpreted
module that runs in-process against the `PGL.IP.API` object model. This is IP's equivalent of a
Geolog Python-loglan or a Techlog Python module, and is the more important on-ramp to match for
"IP-trained user writes real petrophysics."

### B.1 Anatomy of a UserProgram (per `Examples\UserApps\<lang>\UserPrograms\<Name>\`)
| File | Role |
|---|---|
| `Parameters` | UI + I/O contract (`~V19` text). Declares input curves, output curves, numeric params (name, default, min, max, decimals, ...), text/flag params, `useZones`, `Compiler`. |
| `UsersCode.cs` / `UsersPythonCode.py` | The user's algorithm — the only file the user edits. |
| `Methods.*` / `Iplink.*` (`IpClassicPythonLink.py`) | **Auto-generated** proxy glue ("DO NOT MANUALLY EDIT"). Binds declared inputs/params to typed accessors. |
| `UserHelp.md` | Module help. |
| `CompileCSharp.bat` / `CompileCPython.bat` | Build step. |

### B.2 Two Python flavours — pick the modern one for parity
IP ships **two** Python styles; SandiBumi should target the modern `ip2py` shape:

1. **Classic per-sample proxy** (`IpClassicPythonLink.py`): the user loops indices
   `for index in range(TopDepth, BottomDepth+1)` and calls typed accessors —
   `InputCurve1(index)`, `Save_OutputCurve1(index, value)`, `InputParam(index)`, plus attribute
   get/set (`Read_Well_Attribute`, `Read_Curve_Attribute`, `Read/Write_Text(flags,index,attr)` where
   flags 0=Well/1=Log/2=Curve), `SetZone(n)`, `TopDepth`, `BottomDepth`, `TotalZones`, `ZoneNumber`.
   Mirrors the C# `IPLink.UserCode()` pattern (see `CS\...\Calculator\UsersCode.cs`):
   a `switch(Operator)` over `InputCurve1(index)` and `InputParam(index)`, `Save_OutputCurve1(...)`.

2. **Modern vectorised `ip2py`** (recommended parity target): pandas/numpy-based, whole-curve
   operations. Modules seen in shipped examples + Jupyter:
   `general` (`ipprint(text, messageboard_tab=)`), `curves`
   (`get_curve_values(well,'SGR',include_depth=True)`, `set_curve_values`,
   `set_curve_values_at_depths(well,name,values_list,depth_list,set_name=)`, `create_curve_set`,
   `get_curves_as_dataframe`, `get_curve_list_from_curve_names`), `userapp`
   (`get_active_well(self)`, `inputs_to_df(self)`), `wells` (`Wells()` iterable, `get_wellname`,
   `create_curveset_with_depth`), `zones`, `parameter_sets` (`paramset_to_df(well,setname)`),
   `debugger`, and **`calculations`** (`gamma_ray_index(gr_min, gr_max, gr, apply_limits=True)` —
   the only shipped FE helper). Curves come back as plain lists or pandas DataFrames.

   Canonical worked example (`ip2py_clay_volume`, verbatim shape):
   ```python
   active_well = userapp.get_active_well(self)
   data  = curves.get_curve_values(active_well, 'SGR', include_depth=True)
   depth, gamma = data[0], data[1]
   clay = [calculations.gamma_ray_index(10.0, 150.0, g, apply_limits=True) for g in gamma]
   curves.set_curve_values_at_depths(active_well, 'CLAY_IP2PY',
                                     values_list=clay, depth_list=depth, set_name='CALCS')
   ```

### B.3 The `Parameters` (`~V19`) I/O contract
Declares the run-window: a fixed block of input-curve slots (`InputCurve1 , Input Curve 1 , ...`),
a block of numeric params (`InputParam , Input Parameter , <default> , <min> , <max> , <decimals> ,
<step> , <bool>`), text params, flag params, plus header keys `useZones=`, `Compiler=`. Zone-aware
runs re-invoke the code per zone (`ResetZoneParameters`), matching the zone-aware `IParameter` model.

### B.4 Out-of-process automation (not a formula on-ramp, but the embedding API)
`StandaloneApps\` shows IP driven from outside via COM ProgID `IntPetro.API`:
`GetObject("","IntPetro.API")` / `new ActiveXObject("IntPetro.API")`, then
`api.GetService("PGL.IP.API.IDatabaseFactory")`, open a file DB
(`CreateFileBasedDatabaseConnection(path)` → `CreateDatabase_2(conn)` → `db.Connect()`), iterate
`db.Wells` (via `IEnumerableComInteropService.GetEnumerator` for script languages), `well.LoadDataFromDB()`,
`well.CurveSets`. Demonstrated in C#, VB.NET, C++, JScript, PowerShell, MATLAB, Excel-VBA, Jupyter.

---

## C. Parity checklist for SandiBumi
1. **Formula string evaluator** matching §A.2 (bare-mnemonic variables, `TVD`/`DEPTH` reserved,
   `+ - * / ^ ()`, standard function set, `-999` absent, optional predicate + depth window,
   output-curve+unit attached).
2. **User-python on-ramp** shaped like `ip2py` (§B.2 modern): whole-curve get/set by name into
   lists/DataFrames, `set_name` (curve-set) routing, per-zone re-entry, a `calculations`-style
   helper namespace, and a message-board `ipprint`.
3. **Parameter/run-window contract** = zone-aware params with default/min/max/decimals (mirror
   `~V19` + `IParameter`), so an IP UserProgram's run window maps onto a SandiBumi node config.
4. Keep the equations themselves sourced from primary papers / Techlog concept pages — the IP
   on-ramp defines the *interface shape*, not the *method math*.
