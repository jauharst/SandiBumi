# IP2018 CHM Ingest — Target C: Cut-offs, Summations, Global Defaults & Monte Carlo

**Source**: decompiled help text of Interactive Petrophysics 2018 (PGL / Lloyd's Register / Geoactive),
`C:\Users\ARUNIKA\AppData\Local\Temp\c18\_text\*.txt`. Install tree `C:\Program Files\IP2018` untouched (read-only).

**Pages read**: `cutoffsandsummation.htm`, `multi_wellcutoffsandsummati.htm`, `cut_off_sensitivity_results.htm`,
`default_settings.htm`, `options.htm`, `define_monte_carlo_parameters.htm`, `batchmontecarlo.htm`.
Supporting pages consulted for the defaults-file inventory: `tools.htm`, `intro_file_formats.htm`.

**Tier**: essentially all Tier A (reference data, unit conventions, method inventory, file formats, workflow
structure). Two Tier-C name-only flags at the end. No Tier-B primary citations appear on these pages — the
cut-off/summation model and the Monte Carlo module are described without literature references.

**Numeric discipline**: every number below is quoted from the manual with its page. Nothing is rounded,
converted, or supplied from outside knowledge. Where the manual states no value, this report says
`not stated in manual`. The six summation equations are rasterized GIFs and are **not** reconstructed.

---

## 1. The net-pay / cut-off & summation model

### 1.1 Shape of the model

IP's cut-off model is a **per-zone, per-report, multi-criterion flag** model, not a fixed
"Vcl→φ→Sw" pipeline. It has four layers:

1. **Zones** — the interpretation interval is divided into zones (with optional Sub Total pseudo-zones).
   Cut-offs are held *per zone per report*, so every zone may carry different cut-off values unless the
   user ticks "Use same cut-off value for all zones" (page: cutoffsandsummation.htm).
2. **Input curves** — "Up to 10 input curves can be entered into this screen for use in the summation
   computation. The first three curves are pre-defined as 1. Porosity , 2. Water Saturation , and
   3. Clay Volume . The additional 7 curves (rows 4 - 10) are clearly defined options and can be any
   selected input curve. For example, a calculated permeability curve might be used."
   (page: cutoffsandsummation.htm)
3. **Reports** — "You also have the option to set up and execute up to 5 Summation Reports . This allows
   you to generate 3 extra sets of output, in addition to the traditional Net Reservoir and Net Pay
   Summation reports." (page: cutoffsandsummation.htm). i.e. **2 fixed reports (Reservoir, Pay) + 3
   optional reports**, each with its own independent cut-off set and its own flag curve.
4. **Flags** — each report emits a flag curve (ResFlag / PayFlag / user-named), and cumulative
   thickness curves are integrated from the flags.

Critically, **Net Pay is not defined as a sub-set of Net Reservoir by construction**. The Pay report is a
separate report with its own independent cut-off column; the only coupling stated is a UI convenience:
"Values typed into the Reservoir report column for Porosity, Water Saturation and Clay Volume automatically
populate the appropriate cells in the Pay report column." (page: cutoffsandsummation.htm). The manual
describes the *conventional* usage as Reservoir = "the application of the Porosity and optional Clay Volume
cut-off criteria" and Pay = "the application of the Vclay, Porosity and Water Saturation cut-off criteria"
(page: cutoffsandsummation.htm) — i.e. Pay = Reservoir + an Sw criterion, but that layering is a convention
of which "Use" boxes are ticked, not an enforced hierarchy.

### 1.2 Cut-off types available

| Cut-off | Sense | Verbatim | Source page |
|---|---|---|---|
| Porosity `(3) Phi Cut Res/Pay` | `>=` | "If the input porosity curve value is greater than or equal to this value, then the level can be considered for Pay or Reservoir if the level also meets all the other cut-offs." | cutoffsandsummation.htm |
| Water saturation `(6) Sw Cut Res/Pay` | `<=` | "If the input water saturation curve is less than or equal to this value, then the level can be considered for pay or reservoir if the level also meets all the other cut-offs." | cutoffsandsummation.htm |
| Clay volume `(9) Vcl Cut Res/Pay` | `<=` | "If the input clay volume curve is less than or equal to this value, then the level can be considered for pay or reservoir if the level also meets all the other cut-offs." | cutoffsandsummation.htm |
| Other cut-off 1 `(12)` | `>=` or `<=`, user-set | "if the other input curve 1 is >= or <= (depending on setting on input curve window) to this value, then the level can be used in the report" | cutoffsandsummation.htm |
| Other cut-off 2 `(15)` | `>=` or `<=`, user-set | as above | cutoffsandsummation.htm |
| Other cut-off 3 `(18)` | `>=` or `<=`, user-set | as above | cutoffsandsummation.htm |
| Minimum height `Min Res Height` | thickness gate | "Allows you to set the minimum thickness of a zone in order for it to count as net reservoir. Default is 0. (All depth intervals will count towards net if they meet the cut-off criteria)." | cutoffsandsummation.htm |
| Minimum height `Min Pay Height` | thickness gate | "Allows you to set the minimum thickness of a zone in order for it to count as Net Pay. Default is 0. (All depth intervals will count towards pay if they meet the cut-off criteria)." | cutoffsandsummation.htm |
| Minimum height `Min XXXX Height` (optional reports) | thickness gate | "Default is 0. (All depth intervals will be counted, if they meet any cut-off criteria that may have been set)." | cutoffsandsummation.htm |

Numbers in parentheses are IP's own indices into `MonteCarloDefaults.par`: "Numbers in parentheses ( ),
prefixing a parameter name, relate to the Monte Carlo Error Analysis module and correspond to Cut-off module
inputs found in the file MonteCarloDefaults.par ." (page: cutoffsandsummation.htm). The 3/6/9/12/15/18
spacing implies three slots per cut-off (value, use-flag, and one more) — the file layout itself is **not
stated in manual**.

Note the cut-offs for permeability, density etc. are not special-cased: any curve can be a cut-off via the
"Other cut-off" slots, with the direction chosen per curve on the Input Curves tab ("The Cut-off Type column
allows you to set the sign of the cut-off type if a curve is to be used as a cut-off criterion.",
page: cutoffsandsummation.htm).

### 1.3 Pre-processing applied by Curve Type

"In effect, setting a Curve Type applies a pre-processing routine to the input curves." (page: cutoffsandsummation.htm)

1. "Phi - clip the input curve to values greater than zero."
2. "Vcl - clip the input curve to values between zero and one."
3. "Sw - clip the input curve to values between zero and one. Also, this Type initiates a computation of a
   porosity-weighted average for the input curve when computing zonal averages."

This is an important SandiBumi design point: **the Sw curve type is what makes Av Sw a φ-weighted average**;
it is a property of the declared curve type, not a global setting.

### 1.4 Depth-interval / integration convention

"Each depth in the data is considered a discrete interval, with the recorded depth being the center of the
interval. Therefore, when making averages over an interval, only half of the top and bottom depth increments
are counted." (page: cutoffsandsummation.htm)

Interval thickness for the detailed-interval report: "The net thickness is determined by Bottom minus Top
plus depth step. The Top is the depth of the first instance of where the flag curve turns on. The Bottom is
last depth where the flag curve is on. To illustrate this further; think of an interval where there is just
one step with the flag curve on. Top = Bottom but thickness will be the depth step."
(page: cutoffsandsummation.htm)

Averaging basis: "Interval averages are the average values of all the intervals. The main report zonal
averages are the thickness weighted averages. Hence interval average multiplied by the number of intervals
does not necessarily equal zonal average. On the main report all zone averages are thickness weighted
averages not arithmetic averages." (page: cutoffsandsummation.htm)

Cumulative curve scope: "The output cumulative curves are calculated after each zonal computation. They
include only the data within all the defined zones. Therefore, if a level is not defined as being in a zone,
it will not be included in the cumulative curves, regardless of whether the level meets the cut-off criteria."
(page: cutoffsandsummation.htm)

### 1.5 Averaging methods

"The options are : Arithmetic , Geometric and Harmonic ." (page: cutoffsandsummation.htm), selectable per
extra input curve. Same three options in multi-well mode (page: multi_wellcutoffsandsummati.htm).

Guard rule: "For the Geometric and Harmonic averages, any input value which is less than or equal to zero,
will be ignored and not included in the final average." (page: cutoffsandsummation.htm)

### 1.6 The six summation equations — rasterized, NOT recovered

The page carries 6 equation images and no textual formulas:

| Quantity | Image | Status |
|---|---|---|
| Average porosity | `embim160.gif` | rasterized - not recoverable |
| Average water saturation | `embim161.gif` | rasterized - not recoverable |
| Average Clay volume | `embim162.gif` | rasterized - not recoverable |
| Extra curves Arithmetic average | `embim163.gif` | rasterized - not recoverable |
| Extra curves Geometric average | `embim164.gif` | rasterized - not recoverable |
| Extra curves Harmonic average | `embim165.gif` | rasterized - not recoverable |

Only the symbol legend is textual: "i = ith input value / h i = ith input interval / n = number of samples"
(page: cutoffsandsummation.htm).

The one place the manual *does* spell out the arithmetic in text is the multi-well roll-up worked example
(§1.9 below) — that is the only recoverable statement of the thickness-weighted-average form.

### 1.7 Summation outputs (per zone, per report)

**Result parameters (non-editable), Reservoir tab** (page: cutoffsandsummation.htm):

- `Gross Interval` — "Gross interval."
- `Net Res` — "Net reservoir interval."
- `Net/Gross Res` — "Net/Gross ratio for the reservoir interval."
- `Av Phi Res` — "Average porosity in the reservoir interval."
- `Av Sw Res` — "Average water saturation in the reservoir interval. This is a porosity-weighted average."
- `Av Vcl Res` — "Average clay volume in the reservoir interval."
- `Other Curve Av (Name) Res` — "Average of the additional input curves 1-7 in the reservoir interval. Average type (Arithmetic, Geometric, Harmonic) is specified on the input curve window."
- `PhiH Res` — "Computed Porosity - Thickness product of reservoir rock meeting the Porosity and optional clay volume cut-off criteria."
- `PhiSoH Res` — "Computed Hydrocarbon Pore thickness product of reservoir rock meeting the Porosity and optional clay volume cut-off criteria."
- `Other Curve (Name)H Res` — "Cumulative thickness of the additional input curves 1-7 in the reservoir interval."
- With a TVD curve selected: `TVD/TVT Gross`, `TVD/TVT Net Res`, `TVD / TVT N/G Res`.

**Pay tab** mirrors it exactly: `Gross Interval`, `Net Pay`, `Net/Gross Pay`, `Av Phi Pay`, `Av Sw Pay`
(also φ-weighted), `Av Vcl Pay`, `Other Curve Av (Name) Pay`, `PhiH Pay`, `PhiSoH Pay`,
`Other Curve (Name)H Pay`, plus `TVD/TVT Gross`, `TVD/TVT Net Pay`, `TVD / TVT N/G Pay`
(page: cutoffsandsummation.htm).

**Printed report column set**, verbatim from the worked example header:
`Zn  Zone Name  Top  Bottom  Gross  Net  N/G  Av Phi  Av Sw  Av Vcl  Phi*H  Phi*So*H`
(page: cutoffsandsummation.htm). Multi-well adds `Well`, `Type`, `Units`
(page: multi_wellcutoffsandsummati.htm).

**Detail Interval Breakdown** report adds, per zone: "a Gross Interval, Total Number of Intervals, Total Net
Thickness, Mean Interval Thickness, Thinnest Interval Thickness and Thickest Interval Thickness for each of
the Reservoir and Pay intervals." (page: cutoffsandsummation.htm) — this is IP's bed-count / bed-thickness
statistics block, worth copying for thin-bed work.

**Null marker**: "If $$ are seen in the columns then this indicates that there is null data in that zone
interval." (page: cutoffsandsummation.htm)

### 1.8 Default output curve names

(page: cutoffsandsummation.htm)

| Output | Default curve name |
|---|---|
| Reservoir flag | `ResFlag` |
| Pay flag | `PayFlag` |
| Cumulative Reservoir Porosity Thickness | `ResPhiH` |
| Cumulative Reservoir Thickness | `ResPhiSoH - net` (label as printed) |
| Cumulative Reservoir Clay Volume Thickness | `ResVclH` |
| Cumulative Pay Porosity Thickness | `PayPhiH` |
| Cumulative Pay Thickness | `PayPhiSoH - pay` (label as printed) |
| Cumulative Pay Clay Volume Thickness | `PayVclH` |
| Optional-report cumulative curve | auto-named from report short name, e.g. `REP4RHOBH` |

"The PayFlag and ResFlag curves are used in the interactive log plot display, and should always be
calculated." (page: cutoffsandsummation.htm). Multi-well is stricter: "The Pay Flag and Reservoir Flag curves
must be output. However, their names can be changed from the default names."
(page: multi_wellcutoffsandsummati.htm)

Default curves for the detailed interval listing: "The default curves for the interval listing are PHIE , SW
and VWCL" (page: cutoffsandsummation.htm). Those three mnemonics (`PHIE`, `SW`, `VWCL`) are also the curves
used throughout the worked example listings on the same page, and `PHIE`/`SW`/`VWCL`/`PHIT` are the Monte
Carlo default input curves (page: define_monte_carlo_parameters.htm).

### 1.9 Multi-well roll-up arithmetic (the one textual equation)

"In Multi-Well mode an average property is computed by adding together the value thickness of the parameter
for all selected wells, then by dividing the result by the total thickness in all wells."
(page: cut_off_sensitivity_results.htm)

Worked example, verbatim (page: cut_off_sensitivity_results.htm):

```
Well 1 (Zone 1)   Av Phi 0.195   Net Thickness 50ft   Phi H 50 x 0.195 = 9.75
                  Av Sw 0.25     Phi So H 9.75 x (1-0.25) = 7.313
Well 2 (Zone 1)   Av Phi 0.165   Net Thickness 20ft   Phi H 20 x 0.165 = 3.3
                  Av Sw 0.30     Phi So H 3.3 x (1-0.3) = 2.31
Combined:         Phi H 9.75 + 3.3 = 13.05
                  Net Thickness 50 + 20 = 70ft
                  Phi So H 7.313 + 2.31 = 9.623
                  Av Phi 13.05 / 70 = 0.187
                  Av Sw 1.0 - 9.623 / 13.05 = 0.263
```

Definitions given on the same page: "Phi = Porosity / Sw = Water Saturation / So = Hydrocarbon saturation =
1 - Sw / Phi So H = Net Hydrocarbon Pore thickness / Phi H = Net Porosity thickness". And:
"These averages are called thickness weighted averages and are the values used inside the Cutoff and
Summation module." Note that **Av Sw is back-computed from the ΦSoH/ΦH ratio, not averaged directly** —
this is the recoverable statement of the φ-weighted-Sw rule that `embim161.gif` hides.

Field averages in the multi-well report follow the same rule: "The field averages are net thickness weighted
averages." (page: multi_wellcutoffsandsummati.htm)

### 1.10 Horizontal / rising-hole handling

"If a TVD summation report is made of a near horizontal well, where the well actually reverses direction and
goes upwards, then the module reports the actual vertical thicknesses cut by the well. The zonal averages
calculated are weighted by the vertical thickness for each depth increment, so in a nearly horizontal well
the zonal TVD averages could be considerably different to MD averages." … "The summation report adds a ?*? in
front of all net and gross TVD thicknesses where the well is rising with measured depth."
(page: cutoffsandsummation.htm)

Worked figure from that page: "TVD top is -6077.38 and bottom is -6136.13 which gives a depth difference of
58.75?. However the report gives a gross TVD thickness of *79.5? this is because IP is reporting the gross
vertical thickness cut by the well". (Values quoted verbatim; the trailing `?` are mangled quote characters
in the decompiled text.)

### 1.11 Example cut-off values shown in the manual — NOT shipped defaults

The manual's worked screen is explicitly an example. Recorded here verbatim, flagged as example values:

> "Reservoir (default) - Porosity >= 0.1, Clay Volume ,<= 0.5
> Pay (default) - Porosity >= 0.1, Clay Volume <= 0.5, Water Saturation <= 0.5
> Sw < 0.45 (optional) - Porosity >=0.1, Clay Volume <= 0.5, Water Saturation <= 0.45
> PHIT > 0.15 (optional) - Clay Volume <= 0.5, Water Saturation <= 0.5, Porosity (note different porosity
> curve selected) >=0.15"
> (page: cutoffsandsummation.htm)

Printed cut-off block from the same page's example listing: `> 0.100  < 0.500  Y  < 0.500` for
`Phi Pay / Sw Pay / Vcl Pay` against curves `PHIE / SW / VWCL`. The multi-well example prints the same
`> 0.100 / < 0.500 / < 0.500` (page: multi_wellcutoffsandsummati.htm).

**The manual never states a shipped default numeric cut-off.** The only stated numeric defaults on the
cut-off side are `Min Res Height = 0`, `Min Pay Height = 0`, `Min XXXX Height = 0` and
`Result Precision = 3` decimal places.

---

## 2. `default_settings.htm` in full — every default the page states

This page is a **defaults-*plumbing*** page: it enumerates the configuration files, their precedence, and the
canonical units, rather than numeric parameter values. That plumbing is the transferable Tier-A asset.

### 2.1 The configuration-file inventory

> "The configuration files currently catered for under this mechanism are as follows; Curve System defaults
> (default and customized), Curve Type defaults (default and customized), Curve Alias configuration, Mineral
> Solver (default and customized), Crossplot Overlay setup file, Monte Carlo settings, Unit Conversions and
> Neutron Tool Types." (page: default_settings.htm)

> "The configurable files (*.opt, *.vol, *.par, *.Neu) could be saved to a network drive folder and then each
> user maps a path to this folder in Corporate Search Folders." (page: default_settings.htm)

| Default file | What it holds (verbatim fragment) | Source page |
|---|---|---|
| `CparmDef.xml` | "the display characteristics for log curves when they are loaded into IP"; "line color, left and right curve display limits, Crossplot & Histogram minimum and maximum scales"; also "Lin/log scaling" | default_settings.htm |
| `CPARMDEF_USER.PAR` | user-level curve display overrides; "the two files CparmDef.xml and CPARMDEF_USER.PAR are merged, with the curve display criteria from the _USER file taking precedence over the CparmDef.xml file" | default_settings.htm |
| `CurveType.opt` | "the list of generic curve Types that IP uses to auto-select curves for some IP modules" | default_settings.htm |
| `UserCurveType.opt` / `UserCurveTypes.opt` | user-added curve types; **the page uses both spellings** (see gaps) | default_settings.htm |
| `CurveAlias.txt` | load-time curve alias pairs | default_settings.htm |
| `MINDEF.PAR` (`MinDef.par`) | "the default minerals and their properties for the Mineral Solver module" | default_settings.htm |
| `MINEQDEF.PAR` (`MinEqDef.par`) | "the mineral Equation default settings for the Mineral Solver module" | default_settings.htm |
| `Overlay_Files.ovl` | "the descriptions and file names of all available Crossplot overlay line files" | default_settings.htm |
| `MonteCarloDefaults.par` | "the default settings for the Monte Carlo Error Analysis module, including the parameters to be included, the distribution types (Gaussian, Square , Triangular) and the high and low shift values for each input curve or parameter" | default_settings.htm |
| `UnitConversion.par` / `UnitsConversion.par` | Density/Sonic/Caliper unit abbreviations + conversion factors; **both spellings used on the page** | default_settings.htm |
| `Neu_Parm_Files.neu` | "a list of Logging companies and neutron tool types for which neutron parameter look-up tables are available"; user files are `xxx.neu` | default_settings.htm |
| `Lithology.opt` | project-level lithology shading | default_settings.htm |
| `DefaultUnits.opt` | project-level units | default_settings.htm |
| `ShadeTypes.opt` | project-level Density/Sonic/Caliper curve settings & colour tables | default_settings.htm |
| `.OVLX` overlay files | editable through "Edit IP Xml File" | default_settings.htm |

Cross-checked file inventory from the file-formats page (page: intro_file_formats.htm), which names the same
files plus: `ProgDefs.opt` ("Default Program Settings"), `DefaultAlias.cax` ("Default Curve Aliasing Grid"),
`ProjectFileDefaultsSets.opt` ("Default Project Files"), `Lithology.opt` ("Default Lithology Settings"),
`OBG_Files.obg` + `*.obg` ("Default Overburden Gradient Files"), `ShadeType.opt` ("Default Shading Types"),
`Zonecolors.opt` ("Default Zone Colors"), `DipSymbols.opt` ("Default Tadpole Symbols"),
`FluidSub_Default_Parameters.par` ("Default Fluid Substitution Parameters"),
`Poisson_Ratio_Lithologies.par`, `NMR_Tools.csv` ("Pre-defined NMR Tools"),
`SetDictionary.xml` + `Sample Set Dictionary.xml`, `IntPetro.config`, `IPtoolbar.ini`, `IPDBList.ini`,
`DBList.ini`, `IPDBProj.dat`, `IPDBWellXXXX.DAT`, `IPSec.dat`, `SettingsFiles.txt`.

### 2.2 The canonical unit contract — the single most reusable statement on the page

> "IP works with Density, Sonic and Caliper log curves defined in units of grams per cubic centimeter,
> microseconds per foot and inches, respectively." … "the associated conversion factors from the input unit
> to gm/cc for Density, to uSec/ft for Sonic and to inches for Caliper."
> (page: default_settings.htm)

So the internal canonical units are, verbatim: **`gm/cc`** (density), **`uSec/ft`** (sonic),
**`inches`** (caliper). Conversion is applied at load time against a user-extensible table. This is exactly
the "unit canonicalization" contract SandiBumi needs, and IP puts it in a *data file*, not in code.

### 2.3 Defaults precedence rules (verbatim)

1. **Project over IP**: "Project Defaults files are files that are created / stored in an IP Project directory
   and will only be called upon by IP when the user opens that particular project. … The Project Level
   Defaults will be used in preference to the IP Defaults, where they exist." (page: default_settings.htm)
2. **User over system**: "IP will use the entries in the CPARMDEF_USER.PAR file in preference to the
   CparmDef.xml entries." (page: default_settings.htm)
3. **Corporate folder order**: "[ Note: When Corporate Folders is enabled all the text files listed are
   searched for in each folder in turn from the top down." (page: default_settings.htm); reinforced at
   "[ Note: the order in which the system searches through and displays the Corporate Folders is from the top
   of the list down]" (page: options.htm).
4. **Upgrade hazard**: "if you have changed any Default files and you wish to keep your edited files when a
   new version of IP is installed on your PC, remember to save the edited files in another folder/directory
   before installing IP" (page: default_settings.htm). SandiBumi should treat this as an anti-pattern —
   IP has no defaults-migration mechanism.

### 2.4 Curve alias file format (verbatim)

> "The file format is very simple and consists of pairs of curve names that should be linked, one pair per
> line in the file. The following characters are acceptable to separate the external file name from the IP
> load name: comma, space, tab and semicolon." (page: default_settings.htm)

Example block as printed (page: default_settings.htm):

```
$ IP
$ Curve Alias Defaults file
$ Curve name order is as shown with external curve name first, followed by IP Alias name.
$ Curve name should be separated by a space, tab, comma or semicolon character.
LLD2 <space> LLD
MSFL1 <tab> MSFLC
PHICORE2; PHICORE
RHOB1, RHOB
```

`$` is the comment prefix. Alias behaviour in batch load differs from a mask: "Unlike a conventional mask
file however, all other curves in the external LAS files will be loaded to IP using their default external
name (a mask file loads only those curves listed in it)." (page: default_settings.htm)

### 2.5 Numeric / tolerance defaults stated on `options.htm`

`default_settings.htm` itself states **no numeric tolerances**. The numeric global defaults live on
`options.htm`:

| Setting | Verbatim value | Source page |
|---|---|---|
| Irregular Set depth tolerance | "Using the default Irregular Set depth tolerance of 0.2 ft" | options.htm |
| Default Log Plot Scale | "Defaults to Full." | options.htm |
| Default Plots Location | "The default setting is to the IP installation directory, Default Plots sub-directory, which is shipped with the application."; "this parameter, previously stored in the ProgDefs.OPT file" | options.htm |
| CSV Delimiter | "allows the user to select either Comma or Semicolon for delimiting their output CSV files. This is especially useful for users of German language Excel." | options.htm |
| Interactive Line Sensitivity | "Specifies how wide the sensitive zone is around interactive lines, in pixels. This can be adjusted to suit touchscreen devices." (default value **not stated in manual**) | options.htm |
| Well security default scope | "All Wells in database - This is the default option and selects all the wells in database whether they are loaded or not." | options.htm |
| Config file | "The Intpetro.config file (found in IP directory - IntPetro36) contains all the default settings" | options.htm |
| Fixed-attribute marker | "All the attributes prefixed with a double asterisk (**) … are considered Fixed attributes"; "to ensure compatibility when updating older IP databases to IP V3.3 and later versions" | options.htm |
| Lat/long display | "(Decimal Degrees, or Degrees-Minutes-Seconds)" | options.htm |

The worked irregular-set tolerance example, verbatim (page: options.htm): existing set depths
`8000.2 / 8010.6 / 8020.3 / 8040.1`, incoming `8000.3 / 8020.6 / 8040.3`, result
`8000.2 / 8010.6 / 8020.3 / 8020.6 / 8040.1` — "The 8000.3 data will be loaded at 8000.2 and the 8040.3 data
will be loaded at 8040.1. A new 8020.6 depth will have to be created."

Config-distribution files (page: options.htm): `WellAttributes.configUpdate`, `LogAttributes.configUpdate`,
`Well AttributeMapping.configUpdate`, `LogAttributeMapping.configUpdate`,
`FileLoaderAttributes.configUpdate` — "any * .configUpdate files are read and merged into the main
configuration file ( Intpetro.CONFIG …) before it in turn is used to configure IP."

Corporate Search Folders serve (page: options.htm): "Log Plot format files. / Crossplot and Histogram saved
format files. / User Programs. / Crossplot Overlay line files. / All the default configuration files
available from Tools → Defaults ." Sub-folder naming convention: `xxx.hst` (histogram), `xxx.xpt`
(crossplot), `xxx.plt` (log plot); user programs must sit in a sub-folder named `UserPrograms`.

### 2.6 Colours / string limits that encode meaning

- **Column-edit mode**: "This will turn the column header box to green. Now, any one parameter that is
  changed in that column will change all the parameters in the column to the same value. To turn the column
  edit off, click the column header again and its color will return to grey."
  (page: cutoffsandsummation.htm). Same convention on the sensitivity grid: "Click in the title cell of a
  column and it turns green (active)." (page: cut_off_sensitivity_results.htm) — **green = broadcast-edit
  armed, grey = off**.
- **Security colours**: no-access wells "shown in red with no details other than the name of the well";
  read-only wells "shown in grey"; security application "Failures are shown in red." (page: options.htm)
- **Module-reorder affordance**: "This will display a Solid Blue Line which you can then drag a Module entry
  line up or down" (page: define_monte_carlo_parameters.htm).
- **Monte Carlo auto plot shading**: "Porosity, clay volume and Sw output curves are recognized as such and
  the light blue shading has been added for these outputs." (page: define_monte_carlo_parameters.htm)
- **Tornado bars**: "The red bands show the effect of the selected input parameter on the output parameter."
  (page: define_monte_carlo_parameters.htm)
- **Tornado annotation**: "A % sign indicates that the shift for the parameter is in percent. An R character
  indicates that this is a reciprocal shift" (page: define_monte_carlo_parameters.htm).

String / size limits:

| Limit | Verbatim | Source page |
|---|---|---|
| Report Title | "Report titles permit a maximum 25 alphanumeric characters in each text box." | cutoffsandsummation.htm |
| Report Short Name | "The Short Name allows a maximum of 4 characters in the name." | cutoffsandsummation.htm |
| Result Precision | "The default setting is 3 decimal places. The maximum setting is 6 places of decimal. However, the text string length to file or printer is set to 8 characters" | cutoffsandsummation.htm |
| Shade names | "Shade Names must be less than 10 characters in length." | tools.htm |
| Lithology shadings | "There is currently a maximum of 30 Lithology shadings allowed in the Edit Default Lithology table." | tools.htm |
| Lithology bitmaps | "Bitmaps for lithology fill should be 16 x 16 pixels dimensions to be compatible with Windows 98 installations." | tools.htm |

---

## 3. Monte Carlo uncertainty analysis

### 3.1 Model

"The Monte Carlo Error Analysis Module uses a Monte Carlo simulation to estimate the errors in a
petrophysical analysis. You enter the distribution of possible errors associated with the interpretation
parameters and the input curves. IP, using the error distributions, randomizes the input parameters and makes
multiple passes through the analysis modules." (page: define_monte_carlo_parameters.htm)

It is a **workflow-level** MC: the user assembles an ordered chain of interpretation modules and IP re-runs
the whole chain per iteration. Modules available (page: define_monte_carlo_parameters.htm):
"Clay Volume, Porosity SW, Mineral Solver, Basic Log Analysis , NMR , Cutoff, Formula, Multi-Line Formula,
Fuzzy Logic, Neural Networks, Multi-Linear Regression, Cluster Analysis, Curve from Zones, Interp_Demo."
plus "Any User Program that is currently available to IP".

Key structural note: "The Cutoff module is not required in the work flow list, but when not used, it limits
the results to only showing the foot by foot errors on the output curves."
(page: define_monte_carlo_parameters.htm) — i.e. **volumetric (zonal) uncertainty only exists if the
cut-off/summation module is inside the MC loop.** This is the single most important architectural coupling
between Targets C's two halves.

### 3.2 Shift types (the perturbation algebra) — verbatim

(page: define_monte_carlo_parameters.htm)

- "Linear : The parameter is changed by adding or subtracting the shift. Result = Input + Shift"
- "Percent : The parameter is changed by using a percent shift. Result = Input x (1 + Shift / 100)"
- "Reciprocal : The reciprocal of the parameter is changed by adding or subtracting the shift.
  Result = 1 / ( 1 / Input + Shift )"

### 3.3 Distributions offered

Three, everywhere: **Gaussian, Triangular, Square** (the Mineral Solver and Input Curves tabs list them as
"Gaussian, Triangle or Square") (page: define_monte_carlo_parameters.htm). The same three are named as the
persisted set: "the distribution types (Gaussian, Square , Triangular)" (page: default_settings.htm).

Parameters each distribution takes: **all three are parameterised identically** — a `Low Value Shift` and a
`High Value Shift` around the current `Initial Value`, plus the `Type Shift` algebra above.

| Aspect | Verbatim | Source page |
|---|---|---|
| Shift sign constraint | "These values must be positive values ." | define_monte_carlo_parameters.htm |
| Gaussian width mapping | "For the Gaussian distribution the Low Value Shift + High Value Shift represents four standard deviations." | define_monte_carlo_parameters.htm |
| Gaussian truncation | "The Gaussian distribution is limited to 2.5 standard deviations either side of the Mean value. If the random number generator comes up with a value outside of this range, then another random number will be chosen." | define_monte_carlo_parameters.htm |
| Why truncated | "since very large shifts in parameters will make some parameters, like Rw, have non-sensible results (negative) and will result in the interpretation module refusing to run." | define_monte_carlo_parameters.htm |
| Initial Value semantics | "Since each module could have multiple zones with different values in each, the maximum and minimum values are shown in the Initial Value entries." | define_monte_carlo_parameters.htm |
| Triangular / Square shape params | **not stated in manual** (no mode/peak location given; the shape diagrams are images) | — |

So Gaussian: `Low + High = 4σ`, truncated at `±2.5σ`, resample-on-reject. That is a fully implementable spec.
Triangular and Square are named but their internal parameterisation beyond the low/high shift pair is not
described in text.

### 3.4 Seed handling

> "IP uses a random number generator, seeded through the CPU clock time, to calculate the shifts for each
> parameter for each simulation. At the start of each simulation, each parameter is changed using a different
> random number." (page: define_monte_carlo_parameters.htm)

**No user-settable seed is offered and none is stated.** Runs are therefore not reproducible in IP2018.
This is a clear differentiation opportunity for SandiBumi (deterministic seeded runs → reproducible
deliverables).

### 3.5 Correlation / dependency handling

(page: define_monte_carlo_parameters.htm)

- "A Correlation parameter of zero (0) will mean that there is no correlation. A value of 1 equates to a 100%
  correlation and a value of -1 equals an inverse 100% correlation."
- Mechanism: "The Correlation works by taking the randomly-selected shift for Parameter 1 and applying the
  same shift to dependent Parameter 2 . If the correlation is negative then the shift will be the inverse
  amount. For correlation coefficients of less than 1, the correlated shift will then have a randomness
  applied to it - depending on the value of correlation. A coefficient of 0.5 will apply a randomness of half
  what would have been selected if the coefficient was 0.0."
- Worked illustration values: "an m and n dependency correlation of 0.5 and a Neu Wet Clay and Rho Wet Clay
  dependency correlation of -0.8."

This is a **pairwise shift-copy scheme, not a covariance/Cholesky scheme** — worth noting, because it means
correlations are directional (Parameter 1 drives Parameter 2) and not guaranteed to produce a consistent
joint correlation matrix over three or more parameters.

### 3.6 Iteration counts, auto-stop and convergence

| Setting | Verbatim | Source page |
|---|---|---|
| Default iteration count | **not stated in manual** (no shipped default given) | — |
| Auto-stop minimum burn-in | "The auto stop will always run a minimum of 200 iterations and then check for a stop case every 100 iterations after this. Hence the minimum number of iterations is 300." | define_monte_carlo_parameters.htm |
| Convergence parameter default | "You can select the ?Result Parameter? to check for convergence between iterations. The default is the hydrocarbon pore volume in the reservoir zone." | define_monte_carlo_parameters.htm |
| Convergence zone default | "The user selects which zone to check for convergence in (?Result Zone?), default is the ?All? zones average value." | define_monte_carlo_parameters.htm |
| Convergence tolerance default | "The simulation stops when the difference between the last check and the current check, ie what is the difference change in the parameter value during the last 100 iterations, is less that the entered value (default 0.1%)." | define_monte_carlo_parameters.htm |
| Convergence criterion | "The program checks for difference in the P10, P50, P90 and mean values and makes sure they are all within the tolerance specified and only then will stop the simulation." | define_monte_carlo_parameters.htm |
| Batch recommended count | "It is recommended that a test run is made for all wells with 1 or 2 simulations set in the Stop simulation at box so that you can confident that every thing is defined correctly before setting the final number of simulation runs (2000)." | batchmontecarlo.htm |
| Batch override | "The Stop simulation at box allows you to select the number of simulations to run for each well. This will override the number set in the regular Monte Carlo Error Analysis parameter file." | batchmontecarlo.htm |

The 2000 figure is a **recommendation in the batch page's prose**, not a stated program default.

### 3.7 Percentile convention — EXACT, and it is asymmetric

> "The Output Percentiles fields enable you to select the percentiles to display in the results listing. By
> default, the 10 th percentile will be the 10 th percent lowest value of all the simulation results, except
> for Sw where it will be the 10 th percent highest value. If this convention is not required, then it can be
> modified by editing the MonteCarloDefaults.par file (stored in the IP directory) and changing the Results
> section at the end of the file."
> (page: define_monte_carlo_parameters.htm)

Reinforced for the output percentile curves:

> "?Output P : xxx? : This outputs the percentile curve value of the number entered. The program sorts for
> each depth level all the results and then calculates the percentile value. P5 will be the 5th percentile
> lowest value. P50 will be the middle value."
> (page: define_monte_carlo_parameters.htm)

**Reading**: IP uses the **statistical/ascending** convention — Pn = the n-th percent *lowest* value — for
every parameter **except Sw**, which is *flipped* to the n-th percent *highest* value. The flip exists so
that P10 is the optimistic case for both a volume and a saturation simultaneously: low Sw ⇒ high
hydrocarbon. So in an IP result listing, **P10 PhiSoH and P10 Sw are the same side of the story
(optimistic), but they are opposite ends of their own sorted arrays.**

Traps for SandiBumi:

- If you implement a single global "Pn = n-th lowest" rule and label an IP-style report, your **Sw column
  will be inverted relative to IP** while every other column matches. Silent error, plausible output.
- The flip is a *configurable convention* stored in the Results section of `MonteCarloDefaults.par`, so two
  IP installations can disagree. Any Sw percentile imported from an IP report must carry the convention with
  it.
- Percentile interpolation method (nearest-rank vs linear) is **not stated in manual**.

### 3.8 Percentiles are computed per-parameter, independently — the consistency caveat

> "At the end of the MonteCarlo simulation run, which could involve several thousand iterations, the values
> used for every parameter are ranked from low to high and basic statistics are run on each one. The Mean and
> Percentile results are calculated on an individual parameter-by-parameter basis. So, for example, the AvPhi
> Res P50 value will not necessarily come from the same simulation iteration as the Av VCL Res parameter P50
> value. This means that a straight multiplication of the P50 Gross thickness, P50 Net to Gross Ratio, P50
> AvPhi Res and P50 (1- Av Sw Res ) values will not yield the exact P50 value for the Net Pay Thickness Res
> ( Phi * So * H ) parameter, though it should be close."
> (page: define_monte_carlo_parameters.htm)

Note "the values used for every parameter are ranked from low to high" — confirming ascending sort as the
base, with the Sw display flip layered on top.

### 3.9 Output structure

| Item | Verbatim | Source page |
|---|---|---|
| Output curve count | "The Output Curve Names allows you to select up to 10 curves to be output." | define_monte_carlo_parameters.htm |
| Per-level statistics | "For each output curve, IP will calculate the Mean and Standard Deviation statistics on a level-by-level basis." | define_monte_carlo_parameters.htm |
| Mnemonic suffixes | "XXX MN Mean Result curve / XXX PSD Plus one standard deviation / XXX MSD Minus one standard deviation. Where XXX is the original curve name" | define_monte_carlo_parameters.htm |
| Save-all default | "If this box is cleared then just the mean value and the standard deviation error curves are saved. Default is to save all results." | define_monte_carlo_parameters.htm |
| Array curve sizing | "an array curve is created for each of the output curves. This array curve will have the same dimensions as the number of iterations. Hence it can be very large." | define_monte_carlo_parameters.htm |
| Percentile cost warning | "The calculation of the percentile curves are very CPU intensive and can be slow when the iteration count is high. Note: It is recommended that the graphics update value is kept high" | define_monte_carlo_parameters.htm |
| Input-curve confidence output | "This will output two curves per input curve set in the ?Input Curves? tab. The two curves will represent the value of the input curve when the ?Low Value Shift? and ?High Value Shift? values are applied" | define_monte_carlo_parameters.htm |
| Retro-percentiles | "The ?Make Output Curves? button allows other percentile curves to be created after the simulation has been run. It uses the array result curves to do this." | define_monte_carlo_parameters.htm |
| Result listing layout | "The top line for each zone lists the original deterministic results from the Cutoff module. The second line of the report gives the Mean value of all the individual probabilistic simulation results. The following lines then display the probabilistic percentile results as previously defined." | define_monte_carlo_parameters.htm |
| Raw dump | "a .csv (comma delimited) file is produced containing all of the raw output data from each simulation run. This file will contain one line of text per simulation run. Each line will have all the input shifts and output results." | define_monte_carlo_parameters.htm |
| Default MC input curves (example) | "the default input curves. The original curves ( PHIE , SW, VWCL and PHIT ) are displayed as dashed lines." | define_monte_carlo_parameters.htm |

Graphics defaults (page: define_monte_carlo_parameters.htm):
- "Up to nine different histograms can be displayed together."
- "Up to nine different crossplots can be displayed together."
- Histogram overlay: "The default is to have a Gaussian distribution. The Triangular distribution will put a
  triangle on the histogram connecting the left most point to the highest point to the right most point."
- Waveform normalisation: "?Normalize Histogram Maximum Height to 1.0? : If turned on then the histogram
  values in the bins are normalized so that the maximum number is 1.0. … When turned on the ?Log low value?
  is set to -1 and the ?Log high value? to 1."
- "The default crossplots and histograms can be changed by editing the MonteCarloDefaults.par file (stored in
  the IP directory)."

Mineral Solver low/high default seeding:

> "Low / High Values - default values are filled in which are equal to the Mineral end-point value plus or
> minus 10% of the valid value. These default values can be modified."
> (page: define_monte_carlo_parameters.htm)

**±10% of the endpoint** is the only stated auto-populated uncertainty magnitude anywhere in the module.

Parameter-selection default: "All parameters by default are selected. Many of these parameters will in fact
not be used in the analysis. This does not cause a problem, since changing a parameter that is not used will
have no effect on the results." (page: define_monte_carlo_parameters.htm)

### 3.10 Tornado plot (sensitivity, deterministic 2-point)

> "For each parameter in the Monte Carlo analysis, two runs are made; one with the parameter set to its low
> value and one set to its high value (± 2 standard deviations for Gaussian distributions). All other
> parameters are kept to their default values."
> (page: define_monte_carlo_parameters.htm)

Note the mismatch worth flagging: the MC sampler truncates Gaussians at **±2.5σ**, but the tornado endpoints
are taken at **±2σ**. Both figures are stated on the same page. Any SandiBumi tornado must pick one
explicitly and say which.

"The plot is displayed with all selected input parameters shown in the Y axis with decreasing importance
towards the bottom of the plot." … "Any change in the Monte Carlo Error Analysis set-up will mean the Tornado
Plot error runs will have to be re-made." (page: define_monte_carlo_parameters.htm)

### 3.11 Batch Monte Carlo

(page: batchmontecarlo.htm)

- Prerequisite: "Before the Batch Monte Carlo can be run, a parameter model file must be created in the
  regular Monte Carlo Uncertainty Analysis module. This is done by setting up and running the Monte Carlo
  Error Analysis module on one well. Once run (only a few simulations are needed), the results are saved as a
  Model file".
- Output routing hazard: "This file will be put into each wells output directory. Therefore, it is important
  that separate output directories are used for each well, otherwise the results will all be written to the
  same file."
- Missing-input tolerance: "If a curve is missing for a well then an Error Dialog will be displayed …
  You can choose to ignore the error and continue. If this is done then this curve will be removed from the
  Monte Carlo input curves." — and for outputs: "You can continue but no statistics will be generated for the
  missing output curve."
- "The first thing the module does is to verify the input for each well before starting the simulations."
- Per-well parameter-set resolution: "The Set Default Set names for all wells button is very useful since it
  searches each well in turn and looks up the default set names for each module in the work flow."

---

## 4. Cut-off sensitivity analysis

### 4.1 What it varies, what it reports

(page: cut_off_sensitivity_results.htm)

- **Varies**: exactly one cut-off parameter at a time. "Select the Cutoff parameter to investigate. This
  could be any of the Cutoff parameters saved by the Cutoff and Summation module, but normally will be Phie,
  Sw or Vcl."
- **Sweep spec**: "Enter the Cutoff Start Value , Cutoff Stop Value and the Cutoff Step for the selected
  Cutoff parameter. These numbers are the minimum and maximum limits for the selected Cutoff parameter and
  the step is the amount to increment the selected parameter between these two limits."
- **Worked example sweep (example, not a default)**: "In the example below, the PHI cutoff limits to test are
  from 0.0 to 35 pu, using a 1.0 pu increment."
- **Reports**: any summation output, multiple simultaneously. "Select the Output Parameter to plot (e.g. PhiH
  Reservoir ). Multiple output results can be selected simultaneously by adding an entry in additional rows
  in the grid. Each result parameter will be shown as a sensitivity line on the Results plot".
- **Per-zone**: "Select a named Zone or ? All Zones ? to get the data from, by clicking in the Zone column,
  for each Output Parameter row."
- **Engine**: "The module cycles through the Cutoff parameter limits from Start to Stop value, using the
  Step, running the Cutoff and Summation module in each selected well." — i.e. it is a brute-force re-run of
  the summation, not an analytic derivative.
- **Two output classes**: "1. Average properties e.g Av Vcl Res, Av Phie Res or Av Sw Pay & 2. Value
  Thickness products e.g. PhiH Res (Net Porosity Thickness Reservoir), PhiSoH Pay (Net Hydrocarbon Pore
  Thickness Pay)".

Motivation, verbatim and worth quoting to stakeholders: "Deciding what values of Vclay, Porosity and Sw to
use as Cutoffs is quite often guesswork and therefore sensitivities run on the Cutoff values can be useful in
helping to make a decision on the appropriate Cutoff value to apply."
(page: cut_off_sensitivity_results.htm)

### 4.2 Percentile defaults on the sensitivity grid

> "IP defaults to display the 10th, 50th and 90th Percentile results. These include the Cutoff value
> corresponding to that Percentile and the Value of the selected output parameter for that percentile."
> (page: cut_off_sensitivity_results.htm)

The grid is bidirectional: "You can type in a new Value , Percentile or Cutoff into the appropriate cell, Key
Enter or click elsewhere on the grid and IP will recalculate the other two values and update the graphic
display." (page: cut_off_sensitivity_results.htm) — i.e. given any one of {percentile, cut-off value, output
value}, IP solves for the other two along the swept curve. That is a genuinely useful UX pattern to copy:
*"what cut-off costs me 10% of my PhiSoH?"*

### 4.3 Files and persistence

| Item | Verbatim | Source page |
|---|---|---|
| Setup file | "allows you to save the Cutoff Sensitivity Parameters in a Cutoff Sensitivity Setup file (*.cos)" | cut_off_sensitivity_results.htm |
| Results file | "the inputs, calculated data tables and output graphic defaults can be saved to an external (.cosr) file" | cut_off_sensitivity_results.htm |
| Single-well save location | "the Save Results operation will save a results file into the active wells sub-folder in the active IP database." | cut_off_sensitivity_results.htm |
| Multi-well save location | "the Save Results operation will save the results file to the IP project folder level." | cut_off_sensitivity_results.htm |
| Export formats | ".txt for data, .emf file for plot"; plus "Data to Clipboard (Spreadsheet)" | cut_off_sensitivity_results.htm |

Cross-checked: "Cut-off Sensitivity Parameter Set | .cos" and "Cutoff Sensitivity Results | .cosr"
(page: intro_file_formats.htm).

---

## 5. Validation rules and constraints (all stated in the manual)

**Net vs Gross**: the manual states **no explicit rule that Net cannot exceed Gross**. All N/G values in the
worked listings are ≤ 1.000 (page: cutoffsandsummation.htm; multi_wellcutoffsandsummati.htm), but the
constraint is never written. Record as an implicit-only invariant.

Stated rules:

1. **Flag curves are mandatory** — "The PayFlag and ResFlag curves are used in the interactive log plot
   display, and should always be calculated." (cutoffsandsummation.htm); multi-well: "The Pay Flag and
   Reservoir Flag curves must be output." (multi_wellcutoffsandsummati.htm)
2. **Porosity and Sw are mandatory inputs; Vcl is conditional** — "One must set up the Porosity and Water
   Saturation curves to be used for the report. A Clay Volume curve is optional." … "The Clay Volume curve is
   optional, but must be entered if clay volume is used as a cut-off." (multi_wellcutoffsandsummati.htm)
3. **Sub Total zones carry no cut-offs** — "Sub Total zones are not displayed on the interactive plot and
   have no associated cut-offs, but use the interpretation results from the normal zones to determine net pay
   or net reservoir average properties." (cutoffsandsummation.htm); "All zones, set up in the Zones tab, will
   be displayed here except Sub Totals ." (multi_wellcutoffsandsummati.htm)
4. **Cumulative curves are zone-scoped** — data outside a defined zone never enters a cumulative curve, even
   if it passes the cut-offs. (cutoffsandsummation.htm)
5. **Geometric/Harmonic reject non-positive values** — "any input value which is less than or equal to zero,
   will be ignored and not included in the final average." (cutoffsandsummation.htm)
6. **Post-Run edit lock** — "after the first time that the Run button is pressed, any changes to the cut-off
   parameter values must be made on the Cut-off and Summation Parameters screens or on the interactive log
   plot." / "once the Run button has been used, the default settings for cut-off values can no longer be
   edited on the second set-up tab." (cutoffsandsummation.htm)
7. **Re-calculation is explicit** — "Simply changing a parameter in the display will not immediately mean
   that a zone is re-calculated . You must click the OK button again in order to re-calculate all zones and
   update all displays." (cutoffsandsummation.htm)
8. **Zone locking** — "The Lock Zone column enables youto lock zones. This prevents any changes being made to
   the parameters in that zone in any of the tabs within the module." (cutoffsandsummation.htm)
9. **Zone gaps are legal** — "It is possible to have gaps between zones." (cutoffsandsummation.htm);
   multi-well: "Blank zone Top and Bottom depth cells indicate that a zone is absent in a well."
   (multi_wellcutoffsandsummati.htm)
10. **X/Y in report requires a survey** — "These coordinates can only be computed if the well has a valid
    deviation survey and surface location loaded into IP." (cutoffsandsummation.htm)
11. **MC shifts must be positive** — "These values must be positive values ."
    (define_monte_carlo_parameters.htm)
12. **MC Gaussian truncation with resampling** at ±2.5σ. (define_monte_carlo_parameters.htm)
13. **MC prerequisite** — "In order for any modules to be used in the work flow, they must have been set up
    and run on the well data before using the Monte Carlo module. You must, therefore, have made an
    interpretation before starting . This includes the User Programs ."
    (define_monte_carlo_parameters.htm)
14. **Formula-module constants cannot be perturbed directly** — "if you want to use a constant value in an
    equation and vary that constant during the simulation; then the constant value must be converted into a
    Constant curve (using the formula module)". (define_monte_carlo_parameters.htm)
15. **Formula modules ignore the depth-range check** — "The User Formula and Multi-Line Formula modules, if
    they are included in the simulation run, do not verify the depth ranges overlap, but just run the interval
    specified on the formula input." (define_monte_carlo_parameters.htm)
16. **Sensitivity prerequisite** — "As a prerequisite to using these modules the ? Cutoff and Summation ?
    module or the Multi-Well Cutoff and Summation module must initially have been run on all wells that are
    to be run in the sensitivity computation." (cut_off_sensitivity_results.htm)
17. **Multi-well sensitivity naming constraint** — "Multi-Well mode requires that input curves (typically the
    Porosity, Clay volume and Water saturation curves) in all the wells have the same names. In addition,
    when using multiple wells, a common Zone set should exist. Note that not all zones have to be present in
    every well." (cut_off_sensitivity_results.htm)
18. **Batch MC requires a saved model file** first. (batchmontecarlo.htm)
19. **Multi-well cut-off overwrites single-well state** — "Once this module is run, all the settings and
    results produced will also be found populating the single well module . Therefore, if one does not want
    to lose the settings and results that are currently in the single well cut-offs they should be saved
    before starting". (multi_wellcutoffsandsummati.htm)
20. **Multi-well setup is not persisted on exit** — "The Multi Well cut-off setup is not saved when you exit
    IP or change the database . Therefore, it is good practice to save the format after running the module."
    (multi_wellcutoffsandsummati.htm)
21. **File Loader attribute names must be unique** — "Attribute names must be unique entries."
    (options.htm)
22. **Fixed attributes (`**` prefix) cannot be deleted** — (options.htm)
23. **Reduced folder structures are not retro-applied** — "This is to prevent users from inadvertently
    deleting data that may reside in the removed folders." (options.htm)

---

## 6. Report output formats (Tier A, directly reusable)

(page: cutoffsandsummation.htm unless noted)

| Format | Extension | Note |
|---|---|---|
| Normal Report → File | `.txt` | "The default file name uses the name, with the " .txt " extension." |
| Comma delimited | `.csv` | "will write the file using commas to separate the fields"; delimiter switchable to semicolon via Tools → Options → Miscellaneous Options → CSV Delimiter |
| Tab delimited | `.txt` | "will also have a " .txt " extension but will be a tab Delimited type" |
| RTF | `.RTF` | "contains the formatting information for the text" |
| Clipboard | — | "for subsequent pasting into Word TM or PowerPoint TM documents" |
| Append mode | — | "Append Report: When selected enable you to then select the Normal Report menu option to amend the results to a previous report." |

TVD line-layout rule: "The Output Report has been changed to put the average values on the second line when
TVD reports are made." and "When the Cumulative curve height is put in the same Column as the average value
(only output to printer, RFT, clipboard) the order is changed as is the order in the header. This means the
average value is on the second line."

Multi-well report options (page: multi_wellcutoffsandsummati.htm): individual files per well vs one file for
all wells; `Separate pay and Reservoir results` → enables `Sort by Zone` / `Sort by Well`;
`Output Field average results` → adds an "All Wells" entry; `Output TVD depths on same line as Measured
Depths` — "The default option (cleared) is to put a second line in the report that contains the
TVD/TVT/ Net results for each zone."; `Output files to well directories if selected (default)`;
`Use Well Name in file name if selected (default) will change the output file name to Summation ---- where
---- is the well name.`

Parameter-set persistence: "The Parameter set name box is used to name the file where the Cutoff parameters
will be stored… The same name is also used to save the Parameter Set listing to the hard disk with an
extension of .TXT ." (cutoffsandsummation.htm). Cut-off & Summation parameter sets use `.set`
(page: intro_file_formats.htm).

Keyboard shortcut: "Alternatively press the Keyboard Short-cut Ctrl_Alt_X." (cutoffsandsummation.htm)

---

## 7. Tier-C flags

Nothing on these seven pages is described as patented or proprietary-algorithmic. Two name-only flags, plus
one cross-reference to an already-registered item:

1. **"Stealth mode"** — a branded IP feature name. Evidence: "Stealth mode - Disguises the depths displayed
   in logplots, so that sensitive or tight hole data can be shown to an audience that would otherwise not be
   permitted." (page: options.htm). Name + evidence only; the depth-obfuscation scheme is not described and
   must not be reconstructed. (The *concept* — hiding absolute depths in a display — is generic; the branded
   name is not.)
2. **"Experienced Eye" / neural-network module** — `Neural Networks` appears in the Monte Carlo module list
   (page: define_monte_carlo_parameters.htm), cross-referencing the already-registered Tier-C item (shipped
   neural-network weights). No new detail on these pages. No action beyond the existing registration.
3. **Not Tier C but noted**: the external-database connectors named on `options.htm` (GEOLOG™, OpenWorks™,
   ODM™, PETCOM™/Powerlog, Openspirit, Shell LOGIC S2001.2) are third-party trademarks referenced as
   integration targets — market intelligence (Tier A), not IP's own IP.

---

## 8. Gaps — what the manual does NOT state

1. **`MonteCarloDefaults.par` contents are never shown.** The manual names the file and its index numbers
   (3, 6, 9, 12, 15, 18 for cut-off inputs; also referenced from clayparameters.htm and swparameters.htm for
   Clay Volume and Phi/Sw inputs) but never lists a single default shift value, default distribution per
   parameter, or the Results-section syntax that controls the percentile convention. The actual defaults are
   in the installed file, not the help.
2. **No shipped default cut-off values.** The 0.1 / 0.5 / 0.5 figures are labelled as an example screen.
   Do NOT adopt them as "IP defaults".
3. **No default Monte Carlo iteration count.** Only the auto-stop floor (200/100/300) and a batch-page
   recommendation of 2000.
4. **The six summation equations are rasterized** (`embim160.gif`–`embim165.gif`) and are not reconstructed
   here. The multi-well worked example (§1.9) is the only textual statement of the averaging arithmetic.
5. **Triangular and Square distribution parameterisation** beyond the low/high shift pair — no mode position,
   no truncation rule. Only Gaussian gets a full spec (4σ span, ±2.5σ truncation).
6. **Percentile interpolation method** (nearest-rank vs linear vs Hazen) not stated.
7. **No seed control** — reproducibility is impossible in IP2018 by design ("seeded through the CPU clock
   time").
8. **±2.5σ (sampler) vs ±2σ (tornado)** are both stated without reconciliation — treat as two separate
   deliberate choices, not an error to be "fixed" by guessing.
9. **Net ≤ Gross is never asserted** as a constraint.
10. **"Interactive Line Sensitivity" pixel default** not stated.
11. **Internal filename inconsistencies in `default_settings.htm`**: `UserCurveType.opt` vs
    `UserCurveTypes.opt`; `UnitConversion.par` vs `UnitsConversion.par`; `Overlay_Files.ovl` (default_settings.htm,
    tools.htm) vs `Overlay_Files.ovlx` (intro_file_formats.htm lists BOTH as "Overlay List File"). If SandiBumi
    ever reads IP config files, probe for both spellings.
12. **Correlation beyond pairwise** — no statement of how three or more mutually-dependent parameters are
    reconciled, or whether the resulting joint correlation matrix is guaranteed positive-definite.
13. **`.cos` / `.cosr` file schemas** — named, never specified.
14. **The Cut-off Type drop-down's actual option strings** (presumably `>=` / `<=`) are never listed
    verbatim; only described as "the sign of the cut-off type".

---

## 9. What SandiBumi should take from this

- **Adopt the report-plus-flag architecture.** N reports × per-zone cut-off sets × per-report flag curve is
  strictly more general than a hard-wired Reservoir/Pay pair and costs nothing extra to build. IP's cap is
  5 reports / 10 input curves; there is no reason for SandiBumi to inherit those caps.
- **Adopt the "Curve Type applies pre-processing" idea**, especially that the `Sw` type is what triggers
  φ-weighted averaging. It puts the weighting rule next to the data declaration instead of in a global flag.
- **Adopt the canonical-unit-table pattern**: gm/cc, uSec/ft, inches, resolved at load time from a
  user-extensible file (§2.2). This is the single cleanest piece of IP design on these pages.
- **Beat IP on the seed.** Deterministic, recorded seeds turn an MC run into a reproducible deliverable.
  IP2018 cannot do this.
- **Beat IP on percentile consistency.** IP's per-parameter independent ranking (§3.8) is a known and
  documented weakness — storing per-iteration joint results and reporting *iteration-consistent* percentile
  cases would be a genuine improvement, not a cosmetic one.
- **Carry the Sw percentile-flip convention explicitly in metadata**, never implicitly in code. Getting it
  backwards produces a plausible, silent, shippable error.
- **Copy the Detail Interval Breakdown block** (count / mean / thinnest / thickest net interval) — it is
  cheap to compute and directly relevant to thin-bed work.
- **Copy the bidirectional sensitivity grid** (§4.2): solve for any one of percentile / cut-off / output
  value given the other two.
