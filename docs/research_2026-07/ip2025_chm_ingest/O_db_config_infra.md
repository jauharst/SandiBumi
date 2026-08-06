# O — Database model, parameter architecture, configuration, infrastructure

**Ingest agent O.** Source: Interactive Petrophysics 2025 vendor help (`Interact.chm`, build 13-Mar-2025), decompiled to `Temp\c25`. IP2018 counterpart at `Temp\c18` used for numeric diffing. Every fact below carries its source page in `(parentheses)`; image transcriptions carry `[img-read: file.png]`.

Vendor is **Geoactive Limited** (rebranded from Lloyd's Register — see §9). Prose is paraphrased throughout; direct quotes are short and marked.

---

## 1. Scope & page inventory (71 pages, all accounted for)

| # | Page | One-line content |
|---|---|---|
| 1 | `intro_whats_new_in_ip.htm` | Full version ladder IP2025 → IP3.6 with per-release feature lists (§6). |
| 2 | `tools.htm` | Tools menu census: Defaults files, Shading Types, Global Sets, Zone Colors, Dip Symbols, Palettes, Default Lithology, Curve Aliasing, Default Units/Printer, Workflow Designer, Custom Menus/Toolbars, Message Board, Licensing, Options. |
| 3 | `utilities_modules.htm` | PPFG Toolbox utility modules (10), with numeric defaults — pressure limit 50,000 psi, MDT fallback gradient 0.465 psi/ft, psi/ft→lb/gal factor 0.052, seven temperature models, mud-gas normalisation formula. |
| 4 | `options.htm` | The Options module: General/Well/Attributes/External Database/More. Source of most application defaults (§4). |
| 5 | `installandregisterip.htm` | Vendor identity, three licence models, hardware/software requirements, install path, Database Upgrader, Corporate Folders deployment, silent-install `/A` switch and `IP_LICSERVER` env var. |
| 6 | `intro_getting_started_-_step_by_step.htm` | New/open database workflow; ad-hoc per-well upgrade vs bulk Database Upgrader (with backup). |
| 7 | `managewells.htm` | Well Security Manager: encrypted per-well ACL, optional password, per-user none/read-only/full, "All Other Users", Project Defaults; read-only load forces a "Copy of" well. |
| 8 | `global_parameters.htm` | Project-level Global Parameter Sets; `.set` files named `<Module>_Global_<name>`; Pick-anchored zones; batch run across wells; set deleted from targets after run by default. |
| 9 | `fileoptions.htm` | Database menu; Projects (`.prj`) vs Well Lists (`.wst`); recent-DB list of 10; Save Reminder min 5 min, non-persistent; new wells fill first vacant `IPDBWellXXXX.dat` slot; sessions stored as DB folders. |
| 10 | `managezones.htm` | Zone Set creation/split/copy, TVDss sets, Copy as Tops, 90-colour palette (White resets to default). |
| 11 | `toolbox.htm` | **Map Toolbox of the new 2025 Mapping module**, written in IC terminology — not IP core (see §8). |
| 12 | `default_settings.htm` | Census of the user-editable IP default/config files and their roles (§4.2). |
| 13 | `managecurveheaders.htm` | Six tabs (General/Additional/Descriptions/Statistics/History/…); curve-selection-by-Type prefers most-recently-modified with `Final` flag; Lock flag; `Shift Inc` in samples (positive = downhole); statistics ignore null (-999); Net Interval = non-null count × step. |
| 14 | `parameter-and-attribute-handli.htm` | User App API: curves and parameters share one accessor model; zone navigation; well/log attribute read-write; array accessors; MATLAB chunk accessors (§2.5). |
| 15 | `well-queries.htm` | IC-derived Well Query builder shipped with the Mapping module — distinct from IP Query (§8). |
| 16 | `working_with_parameters_in_mul.htm` | Multi-Well Change Parameters (zone × well grid per parameter) and Multi-Well Parameter Distribution (§2.6). |
| 17 | `intro_database_browser.htm` | Hierarchical tree (wells → Curve Sets / Parameter Sets / Zone Sets); four well filters incl. attribute-based Advanced Filter; well-folder navigation; grouped curves; per-curve statistics inline; drag-copy between sets with regular→irregular prompt. |
| 18 | `cm_copy_curves_from_well.htm` | Copy curves well→well or set→set (both wells must be in memory); depth-range subset; name suffix; output-set override; `Fill data gaps` interpolation; Date-Time wells must match type. |
| 19 | `managepicks.htm` | Picks as an alternative to zones; one-way Pick→Zone link (editing a linked zone demotes it to a plain depth). |
| 20 | `wellheaderinfo.htm` | Manage Well Header Info: tabs General/Position/Default Parameters/Logging/Plot Remarks/Plot Annotations/Drilling Data; 25 logging runs; Logging Contractor drives neutron tool type and overlays (§3.3). |
| 21 | `managecurvesets.htm` | Curve Set model: Short Name ≤8 chars unique per well, `Set:Curve` naming; Regular vs Irregular; changing Step auto-interpolates; **Overwrite = "replace and concatenate" (auto-splice), not whole-curve replace**; Set Dictionary. |
| 22 | `intro_new_modern_ui.htm` | Modern UI (IP2021+): sidebar navigation replacing menus; searchable menus; **the navigation-group table is the best single module census in the manual** (§5.1). |
| 23 | `project_setup.htm` | PPFG project setup: mandated curve-set naming standard (15 set names) and the PPFG dataset rule (1 sample/ft, ≥200 ft below TD; duplicate FPRESS depths offset 0.01 ft). |
| 24 | `curvelisting.htm` | Curve Listing / Curve Edit: ASCII listing and tabular edit; `Display Depths Set` shows data exactly as the interpretation modules see it; Expand Array Data; 6 discriminators; Fill Range / Null Range / Undo. |
| 25 | `manage-multi-well-curve-header.htm` | Multi-well curve header edit; up to 1024 curves at once; filters (Type/Name/Set/Update Module/date/Final); `Row all Same`, `Column Same`, `Lock All`, `Delete All`; bulk null-curve deletion via Statistics-tab mean sort. |
| 26 | `using_the_license_server_manag.htm` | LiMBR License Server: service + management console, default TCP port 11362, six admin tabs, online/offline licence flows, reporting anonymisation and its registry lockdown. |
| 27 | `parametersets.htm` | **The core parameter-architecture page** — set types, working sets, attribute lookups, value polymorphism (§2). |
| 28 | `aboutipdatabase.htm` | Database limits and on-disk anatomy (§3.1). |
| 29 | `gettinghelpforip.htm` | Support channels: `ipsupport@geoactive.com`, ticketing, support portal. |
| 30 | `arraydatacurves.htm` | Curves↔Array Data conversion; `Output high sample rate data` controls whether output stays array-rate or is Z-averaged to well step; Radial Resistivity Image from multi-DOI curves. |
| 31 | `historymodule.htm` | Audit trail: columns ID/Event/Date/Item/User_Name/Comments; Difference of two same-type parameter rows via ExamDiff; **editable SQL Row Filter (new in 2025)**. |
| 32 | `createeditlithologycurves.htm` | `LITH` flag curve mapped to bitmaps in `Lithology.opt`; multiple `Lithology_xxxx.opt` files selectable; project file overrides user Working Folder file; optional `LITH_T` text curve. |
| 33 | `multiwell.htm` | Multi-Well menu census (14 modules). |
| 34 | `well_multi-user-access-guide.htm` | Concurrency model (§3.5). |
| 35 | `manage-multi-well-curve-sets.htm` | Bulk set rename/delete across wells; wildcard filters; **states New Set Name max 4 chars** (conflicts with the 8-char rule — §8). |
| 36 | `user_set_up.htm` | **Production Logging preferences, not IP application preferences** (§8). Line-width defaults 2/2/3/2; logo constrained to 0.75″ × 2.0″. |
| 37 | `manage-multi-well-zone-set-lin.htm` | Batch zone-set linking across wells; base set at top propagates names/depths; equal zone counts required; two "do not match" toggles both default checked. |
| 38 | `curvemanagement.htm` | Hub listing the curve/curve-set modules; notes text curves merged into regular curves at IP 4.2; **states max 50 Curve Sets per well** (conflicts with 500 — §8). |
| 39 | `intro_curves_and_curve_sets.htm` | Restates the database limits; curve values stored as floating point; `*Type` / `@Type` generic curve-name syntax. |
| 40 | `intro_text_boxes.htm` | Text-box editing conventions (cut/copy/paste/undo/select-all/RTL). |
| 41 | `dataviewing.htm` | View menu census: Log Plot, Histogram, Crossplot, Ternary, Rose, Star, Box, Pie, 3D Parameter Viewer, Multi-Well Correlation Viewer, Well Map, Montage Builder. |
| 42 | `ip_query.htm` | IP Query: cross-project index in a Microsoft SQL Compact Edition `.sdf`; asynchronous gather; outputs `.wst` or `.prj`. |
| 43 | `createcurvearraycurve.htm` | Create regular or array curves; array X and Z dimensions; Z controls sub-step depth resolution (Z=6 → nearest 1 inch in a 6-inch-step well). |
| 44 | `saveparametersets.htm` | Save parameter set to DB, disk `.set`, or Global; All Sets current well / all wells archive option. |
| 45 | `link_zone_sets.htm` | Single-well zone-set linking; Base Set = top of column; yellow = depth mismatch, blue = name mismatch. |
| 46 | `curvefromzonesparameters.htm` | Writes zone/parameter values out as curves — confirms tilted parameters are emitted **interpolated**. |
| 47 | `wells.htm` | Well menu hub: security, the `IPDBLock` self-clearing lock (4–5 min), 2000-well memory limit and RAM guidance, Delete Parameter Sets, Plot Range Editor (description ≤20 chars), Take Well Notes. |
| 48 | `manage-multi-well-header-info.htm` | Spreadsheet edit of cultural attributes across wells; `Column Same`; Geodatum/Grid System read-only here; output to `WellHeaders.Txt` or clipboard. |
| 49 | `manage-multi-well-zones_picks.htm` | Multi-well zone/tops editing; paste-from-spreadsheet with Well Name in column 1; up to 100,000 rows; **rows whose well cannot be matched are silently ignored**; match by Name, API or UWI. |
| 50 | `managewellheaderinfo.htm` | **A User App (Fortran 77) worked example** that fills well-header fields, not the header module itself (§8). Exposes `FIRST_AVAILABLE_LOG_RUN` / `LAST_AVAILABLE_LOG_RUN` constants. |
| 51 | `createeditpointcurve.htm` | Interactive point-curve digitising into a log-plot track; depths exportable to clipboard. |
| 52 | `drilling-data-in-the-well-head.htm` | Well Header → Drilling Data tab: Mud Weight/Pressure table (step-change default, optional Linear ramp, Additional Pressure offset) and Drilling Incidents; writes `Mud_Press` curve. |
| 53 | `abbreviations_and_definitions.htm` | The manual's only symbol glossary — geomechanics only (§7). |
| 54 | `manage-multi-well-working-sets.htm` | Batch-set Working Input/Output Sets across wells; counts of wells per set name; optional dynamic creation of missing output set. |
| 55 | `load___save_parameter_set.htm` | Parameter Set dialog: load/save DB, disk, or Global; cross-type load keeps only Top/Bottom/Zone name; All Wells / All Sets views. |
| 56 | `manage_zones_and_picks.htm` | **Second, differing enumeration of parameter-set types** (§2.1, §8). |
| 57 | `multi-well_notes.htm` | View/export per-well free-text notes across wells. |
| 58 | `navigationanddataentry.htm` | Work Areas: multiple concurrent, tabbed, renameable, thumbnail view; limited only by RAM. |
| 59 | `intro_useful_links.htm` | External links; confirms `geoactive.com` domains and `ipsupport@geoactive.com`. |
| 60 | `intro_toolbars_and_menus.htm` | Classic-UI toolbars; toolbar positions saved at exit. |
| 61 | `intro_discalimer_of_warranty.htm` | Warranty disclaimer; copyright held by **Geoactive Limited**. |
| 62 | `intro_short-cut-keys.htm` | Complete keyboard shortcut list (§4.5). |
| 63 | `delete_parameter_set.htm` | Deletes parameter sets from the active well only (never from disk); multi-well deletion via Manage Multi-Well Zones/Tops. |
| 64 | `printparametersets.htm` | Export Parameter Set Reports to printer or `.txt`; CSV output for Vclay and PhiSw. |
| 65 | `cm_drag_and_drop_curves.htm` | Drag-and-drop curves into sets; Refresh needed after active-well change. |
| 66 | `cm_delete_curves.htm` | Delete curves dialog; select-all and select-by-set. |
| 67 | `open.htm` | **Production Logging** run/analysis creation (§8). |
| 68 | `third_party_software_licenses.htm` | Points to `EULA.rtf` in the install folder. |
| 69 | `project.htm` | **Production Logging** project functions (§8). |
| 70 | `workflow.htm` | **Production Logging** workflow stepper (§8). |
| 71 | `welcometointeractivepetrophysics.htm` | Splash page — a single image `[img-read: hmfile_hash_5f73c0c6.png]`: Interactive Petrophysics + **Geoactive** logos, "What's New in IP 2025", Install and Register IP, Start to Use IP, YouTube Video Library, IP Support Portal. |

---

## 2. Parameter architecture

### 2.1 Parameter Sets are typed, zoned containers

A Zone/Tops Set is a list of intervals, each with Name, Top Depth, Bottom Depth and a colour. **A Parameter Set is an extended Zone Set** in which every zone additionally carries the full parameter vector of one interpretation module (`manage_zones_and_picks.htm`). The number and names of the parameters follow from the module (`parametersets.htm`). This is the single most important structural idea in IP: *there is no separate parameter table — parameters are columns hung off zone rows, and zonation is therefore the unit of parameterisation.*

Set Types (per `parametersets.htm`):

`Tops`, `Clay`, `PhiSw`, `Cutoff`, `Splice`, `Basic_Loganal`, `TDT_Stand_Alone`, `TDT_Time_Lapse`, `NMR`, `MinSolve`, `Pore_Pres_Grad`, `UP******` (one per named user program).

`manage_zones_and_picks.htm` gives a **different** list: `Tops`, `TVDss_Set`, `Basic_loganal`, `Clay`, `PhiSw`, `Cutoff`, `UP`, `MonteCarlo`, `MinSolve`, `NMR`, `Pore_Pres_Grad`, `TDT_Stand_Alone`, `TDT_Time_Lapse`. The union across the two pages is the practical type list; see §8.

`Tops` is referenced to measured depth and IP converts entered depths to MD for display; `TVDss_Set` keeps entered depths as TVDss (`manage_zones_and_picks.htm`).

### 2.2 Persistence and lifecycle

- The **working** Parameter Set of every interpretation module is automatically written into the well every time the well is saved, and restored when the well is loaded (`parametersets.htm`). There is no explicit "save my parameters" step for the active state.
- Named sets can additionally be persisted to three places (`load___save_parameter_set.htm`): inside the database, to an external `.set` disk file, or to a **Global Parameter Set** in the project's `Global Parameters` folder.
- `.set` files default to the output data directory set under *Set Default File Location* (`saveparametersets.htm`).
- Bulk archive: *All Sets current Well* and *All Sets all wells* write every set in the project to disk in one action — the manual explicitly frames this as making an archive record of a project's parameterisation (`saveparametersets.htm`).
- Deletion is scoped: *Delete Parameter Sets* removes sets from the **active well only** and never touches disk copies (`delete_parameter_set.htm`); multi-well deletion is done from Manage Multi-Well Zones/Tops with a confirmation prompt.
- Reporting: *Export Parameter Set Reports* → printer or `.txt` named after the set; Vclay and PhiSw additionally offer CSV (`printparametersets.htm`).

**Cross-type load rule (important):** loading a set whose Type differs from the module you loaded it from transfers **only** Top, Bottom and Zone name — every parameter reverts to that module's internal IP defaults (`load___save_parameter_set.htm`, `parametersets.htm`). Loading a same-type set overwrites the existing one wholesale.

### 2.3 What a parameter value may be

A single parameter cell is polymorphic (`parametersets.htm`, `swparameters.htm` [agent B's page, cited for the mechanism only]):

| Form | Meaning |
|---|---|
| Numeric constant | Constant over the zone. |
| **Curve name** | Evaluated depth-by-depth inside the zone. Mixed use is allowed — some zones constant, some curve, in the same set (`resistivity_to_pressure.htm`, agent J's page, states this explicitly). |
| **Attribute lookup** `$(Well.Def_Rw)` | Resolved from the well header at run time. *Resolve Attribute Lookups* freezes them to static values. |
| **Tilted** `top:bottom` | Two values with interpolation across the zone. |
| **Log-tilted** `Lg top:bottom` | As above but interpolated logarithmically, so the trend is straight on a log grid (used for Rw). |
| Text list / logic flag | Method selectors and yes/no flags. |

### 2.4 Interpolation between zone boundaries — tilted parameters

The definitive mechanics live on `logplot_edit_the_log_plot_format.htm` (agent L's page; recorded here because tasking (a) asks for it):

- Tilted values display in the parameter grid as `top:bottom`, and may be typed in that form.
- Logarithmic tilt is flagged by an `Lg` prefix and may be created manually by typing it.
- Un-tilting = drag the line vertical, or replace the pair with a single number.
- Once tilted, tilt can be changed whether or not the Tilt mode is on.
- Viewing a tilted parameter on an interactive histogram/crossplot shows the range with arrows; **any interactive edit there collapses the parameter back to un-tilted.**
- `curvefromzonesparameters.htm` (mine) confirms the Curves-from-Zones/Parameters module writes out the **interpolated** values as displayed on the log plot — i.e. the tilt is a first-class interpolation, not a display trick.

Interpolation is **within** a zone only. There is no interpolation across a zone boundary: at a boundary the parameter steps to the next zone's value (or to that zone's top-of-tilt value). Any smooth field-wide trend must therefore be expressed either as a parameter *curve* or as a chain of tilted zones whose endpoints are made to agree.

### 2.5 Parameters and curves are the same object to the API

`parameter-and-attribute-handli.htm` is unusually explicit: *"Curve and parameters are handled exactly the same inside the code."* In the User App API:

```
Xrw = Rw(index)                 ' parameter OR curve, same call signature
Save_PhiAv(index, XphiA)
ZoneNumber() / TotalZones() / SetZone(iZone)
XXX_Name() / XXX_Units() / XXX_Comments() / Save_XXX_Comments(Text)
Read_Well_Attribute(name) / Write_Well_Attribute(name, value)
Read_Log_Attribute(name, run) / Write_Log_Attribute(name, value, run)   ' run -1 semantics
Array_XXX(Index, Xindex, Yindex) / Save_Array_XXX(...)
Array_XXX_MaxX() / Array_XXX_MaxY()
XXX_Chunk(index, count)         ' MATLAB only, for speed
```

Constants `FIRST_AVAILABLE_LOG_RUN` and `LAST_AVAILABLE_LOG_RUN` address log runs symbolically (`managewellheaderinfo.htm`). If a parameter was entered as a fixed value, the depth index passed to its accessor is simply ignored — that is the whole of the constant-vs-curve abstraction. The API also carries a hard rule: **never write back to the Depth curve.**

### 2.6 Multi-well parameter operations

`working_with_parameters_in_mul.htm`:

- **Multi Well Change Parameters** — pick a Parameter Set and one parameter; the grid is *zones × wells*. Zones with the same name align on one row; unnamed zones list in depth order. Clicking a row or column header turns it green and makes it an edit target — changing one cell then propagates along that row/column, including when the value is a curve reference. *Apply to All Wells All Zones* sets everything at once; flag parameters get a checkbox instead of a value box. Changes are not committed until OK or Re-Run Analysis; **switching Parameter Set before committing silently discards the edits.**
- **Multi Well Parameter Distribution** — copies whole Parameter Sets from the focus well to other wells, realigning each set's zone tops onto a nominated *common* Tops/Zones Set in the target well. With `Copy using zone names` off, zonation schemes may differ entirely and depths are realigned via the common set — this is what allows a vertical-well parameterisation to be pushed into a horizontal well. With it on, zone names must match, which is what enables repeat-penetration horizontals. Locked zones propagate as locked. `Link distributed parameters to common set` links the resulting sets to the common tops set in each well.
- Missing output sets are created in the target well, taking top/bottom/step from that well's Default Set.

### 2.7 Global Parameter Sets

`global_parameters.htm`: stored at **project** level in a `Global Parameters` folder as `.set` files named `<Module>_Global_<name>`. Their zones must be anchored to **Picks**, not fixed depths, so that one global set lands correctly in every well. A batch run applies them across a well selection; **by default the set is then deleted from each target well**, on the reasoning that the audit record lives in the well's History file rather than in a proliferation of copies. The cutoff module is excepted from that deletion.

### 2.8 Global (curve) Sets

Distinct from Global Parameter Sets. `tools.htm` → *Global Sets*: a list of Short Name (≤8 chars) + Full Name + Regular/Irregular flag that is **auto-created in every in-memory well, and in any well subsequently loaded**, for as long as the name stays in the list. Removing a name stops future propagation but does not remove existing sets. `Save as Project Default` writes `ProjectFileDefaultsSets.OPT` into the project root and scopes the list to that database; unchecked, the list applies to every database the user opens.

### 2.9 The `(Parameter #N)` numbering — cross-check against `H_module_parameter_reference.json`

**The numbering is real, load-bearing, and stable.** The literal token `Parameter #` does not appear anywhere in the IP2025 CHM (verified by exhaustive grep of both `_text.txt` and `.htm` in `c25`, 0 hits) — it is an artefact of how the `.hlp` files were extracted for `H_module_parameter_reference.json`. What the CHM *does* carry is the same ordinal, printed as a parenthesised prefix:

> "Numbers in parentheses ( ), prefixing a parameter name, relate to the Monte Carlo Error Analysis module and correspond to Clay Volume inputs found in the file `MonteCarloDefaults.par`." (`clayparameters.htm`; identical wording on `swparameters.htm` and `cutoffsandsummation.htm`.)

`nmrinterpretation.htm` states the mechanism outright: to add a parameter to the Monte Carlo simulation you edit `MonteCarloDefaults.par` and **"The Parameter Number is required for this"**, followed by a 45-entry lookup list. So the ordinal is IP's *stable external handle for a parameter within a set type* — the equivalent of a column ID.

**Cross-check performed** against `D:\XX. SandiBumi\docs\research_2026-07\ip2018_chm_ingest\H_module_parameter_reference.json` (extracted from IP2018 `.hlp` files, which are present unchanged in IP2025):

| Module | `.hlp` numbered params | Ordinals also printed in the IP2025 CHM | Shared ordinals | Name agreement |
|---|---|---|---|---|
| ClayVol | 70 (max n = 70) | 66 (max n = **72**) | 64 | **61 exact, 3 renamed** |
| PhiSw | 188 (max n = 189) | 27 recovered by regex | 27 | **27 / 27 exact** |
| Cutoff | 0 (all `n: null` — the `.hlp` extraction carries no ordinals for this module) | present in the CHM | — | not cross-checkable |

Findings:

1. **The numbering is stable across seven years and two major versions.** Every shared ordinal in PhiSw matches by name; 61 of 64 match in ClayVol.
2. **Three ClayVol renames, same ordinal:** #39 `OD Ot1 Clay` → `OD Curv1 Clay`; #40 `OD Ot2 Clay` → `OD Curv2 Clay`; #41 `OD Ot2 Clean1` → `OD Curv1 Clean1`. The #41 change is not purely cosmetic — `Ot2` → `Curv1` swaps which of the two "Other Double" indicator curves the clean point belongs to. **Anyone porting IP2018-era Other-Double clay parameters should verify #41 by behaviour, not by name.**
3. **Two ClayVol ordinals are new in IP2025:** #71 `Sonic Kerogen` and #72 `Sonic Heavy_Min.` — the organic-shale correction extended to the sonic (§9).
4. **Six ClayVol ordinals exist in the `.hlp` but are not printed in the IP2025 manual:** #51 `Link Clay Paras`, #52 `Link PhiSw Clay`, #53 `Vcl Av Method`, #54 `Vcl Mix Method`, #56 `Percentile Clean`, #70 `Link Clean Paras`. These are the linking/averaging/percentile controls. They are documented in prose on `clayparameters.htm` but without the parenthesised number — so `H_module_parameter_reference.json` remains the **only** source for their ordinals. Keep that file.
5. **Numbering is sparse and never renumbered.** The NMR list runs 1,2,3,4,5,8,9,10,11,12,14,15,16,17,19,20,23,24,25,26,32,38,… — the gaps are retired or non-numeric parameters. Ordinals are permanent handles; IP appends rather than compacts.

### 2.10 Defaults precedence hierarchy for parameters

Established across `swparameters.htm`, `tools.htm`, `default_settings.htm`, `options.htm`:

1. **Explicit value in the zone's Parameter Set** (constant, curve, tilt, or resolved attribute lookup).
2. **Well header → Default Parameters tab.** Explicitly: "IP will attempt to read the Rw and Rw Temp parameters from the Well Header > Default Parameters tab" (`swparameters.htm`).
3. **Hard-coded module default.** Same sentence: "If this is empty, then the Phi/Sw module will use default values of 0.1 at 60 degrees."

For *display/curve* defaults, the chain is file-based and project-over-system in every case:

- `CPARMDEF_METRIC_User.PAR` (project) > `CPARMDEF_METRIC.PAR` (IP) — when *Use Metric File* is on;
- `CPARMDEF_USER.PAR` (project) > `CparmDef.xml` (IP) — when *Use Project Defaults* is on;
- `DefaultUnits.opt` (project) > `ProgDefs.opt` (IP);
- `ShadeType.opt`, `Lithology.opt`, `Zonecolors.opt`, `DipSymbols.opt`, `ProjectFileDefaultsSets.OPT` — project copy wins over the IP/user-Working-Folder copy;
- **Corporate Search Folders sit above all of it** for every file under Tools → Defaults, plus plot/crossplot/histogram formats, User Apps and overlay files (`options.htm`).

To reset a project back to IP defaults you must close the project and physically delete the project-level file — the checkbox alone does not do it (`tools.htm`).

### 2.11 Curve resolution — the aliasing chain

Not strictly parameters, but it is the sibling mechanism and SandiBumi needs it (`tools.htm` → Curve Aliasing, `manage-multi-well-working-sets.htm`):

- **Working Input Set is searched first**; if the required curve type is not found there, the Curve Aliasing logic takes over.
- Aliasing modes: **Off** (curve *Type* only — `@Density` picks the most recent Density-type curve), **Manual** (as Off unless the name is prefixed `#`, which requests a specific curve with type-based fallback), **Automatic** (name-not-found falls back to the alias grid by type).
- `Use Final Curve` restricts the search to curves flagged Final.
- Set-search sub-modes: `Use Set Selection Grid` (ignore the input set entirely, walk the grid order), `Curves from input set only` (fail if absent), `Curves from input set first` (then fall through to the grid).
- Generic-type name syntax in any curve text box: `*GammaRay` or `@GammaRay`. **In formulas use `@`, never `*`** — `*` is the multiplication operator (`intro_curves_and_curve_sets.htm`).
- The alias grid persists as `DefaultAlias.cax`, per-user or per-project, and round-trips through Excel as CSV.

---

## 3. Database model & audit trail

### 3.1 Hard limits and on-disk anatomy (`aboutipdatabase.htm`, `intro_curves_and_curve_sets.htm`)

| Limit | Value |
|---|---|
| Wells per database | 9,999 |
| Wells loadable in memory | 2,000 |
| Curves per well | 20,000 |
| Curve Sets per well | 500 |
| Depths per Curve Set | 3,000,000 |
| Zone Sets per well | 500 |
| Logging runs per well | 25 (`wellheaderinfo.htm`) |
| Wells per correlation plot | 50 |
| Curves editable at once, multi-well header module | 1,024 |
| Tops rows pasteable at once, multi-well | 100,000 |

Curve values are stored as **floating point only** — an integer-valued flag curve is stored as `1.0000000` (`intro_curves_and_curve_sets.htm`). There is no integer or categorical curve type; text curves were merged into ordinary curves at IP 4.2, so one curve can hold numeric *and* textual data (`curvemanagement.htm`).

Project directory contents: `IPDBWellXXXX.DAT` (one binary file per well, new wells filling the first vacant slot number), `IPDBWellList`, `IPDBLock`, `IPDBWellxxxx.history`, `ipfolder.ico`, `Desktop.ini`. A per-user sub-directory holds `IntPetro.ini` and `IPDBProj.dat`. Sessions are stored as folders inside the database, with a default session named after the user (`fileoptions.htm`).

File-format census (`intro_file_formats.htm` — assigned to another agent; recorded here because the DB model depends on it): `.dat` well data, `.wst` well list, `.prj` project, `.whf` well header format, `.set` all interpretation parameter sets, `.history` well history, `.sharing` + `.sharing-journal` multi-user state, `.sfo`/`.sft` depth shifts, `.bls` baseline shifts, `.ztc` zones-to-curves, `.cap`/`.cos`/`.cosr`/`.mpp` cap-pressure and cutoff-sensitivity sets, `.frm`/`.mlf`/`.xul` formulas, `.env` environmental corrections, `.flt` filters, `.obu`/`.obg` overburden, `.plt`/`.xpt`/`.hst`/`.fpt`/`.ovl`/`.ovlx` plot and overlay formats, `.ipm` montage, `.dpv` deep plot view, `.fta` formation testing, `.wmp`/`.fbt` multi-well format and batch.

### 3.2 Curve Sets

`managecurvesets.htm`, `intro_curves_and_curve_sets.htm`:

- Short Name **≤8 characters**, unique within the well; underscores allowed. Full Name unconstrained. Curves outside the Default Set are referenced as `Set:Curve`.
- **Regular** sets inherit the well's Default step; **Irregular** sets get their own depth curve. Changing a regular set's Step auto-interpolates its curves.
- Irregular-set depth matching is tolerance-based, not exact (§4.1).
- **Overwrite semantics are a trap:** overwriting a curve in IP means *replace-and-concatenate* — the incoming interval replaces the overlapping part and is spliced into the existing curve. It does **not** discard the rest of the old curve. Any port of IP behaviour must reproduce this or silently lose/keep data.
- Copying a curve from a regular into an irregular set prompts: expand the irregular set's depths to accept all data, or keep only the points that already match existing depths (`intro_database_browser.htm`).
- Set names and metadata are governed by `SetDictionary.xml`; with Corporate Search Folders enabled IP uses the **first** `SetDictionary.xml` found in the folder list (`options.htm`).

### 3.3 Attributes — the three-layer metadata model (`options.htm` → Attributes, `wellheaderinfo.htm`)

IP stores metadata in three attribute namespaces, each an editable reference table:

| Layer | Holds | Surfaces in |
|---|---|---|
| **Well Attributes** | Cultural: Country, Field, Company, Spud Date, Location… | Manage Well Header Info → General |
| **Log Attributes** | Per-run logging parameters: Rm, Rmf, Rmc, BHT, bit size… | Manage Well Header Info → Logging (25 run columns) |
| **Curve Attributes** | Per-curve properties, auto-populated from DLIS | Manage Curve Headers → Additional |

Each table separates **Fixed** attributes (grey background; Log Attributes prefix them `**`) from user-**Customizable** ones. Fixed attributes exist purely to keep old databases upgradeable and cannot be deleted, though their Display Alias may be renamed — including into another language.

Loading is mediated by a **File Loader** mnemonic table plus three mapping tables: *Well Mappings* (File Loader mnemonic → Well Attribute), *Log Mappings* (→ Log Attribute), *DLIS Curve Mappings* (DLIS Channel or Axis property → Curve Attribute, with a Vendor column because Axis IDs differ by vendor). Unmapped loader mnemonics raise a visible warning marker rather than being dropped silently.

Every attribute and mapping lives in **`Intpetro.config`** (XML, in the user's Working Folder). Each table can *export* a `*.configUpdate` file; dropping that file into another user's Working Folder causes IP to import it into their `Intpetro.config` and then delete it — a fire-and-forget config distribution channel with no server.

Attribute values feed back into interpretation: the **Logging Contractor** field on the Default Parameters tab selects the neutron/density crossplot overlays, sets the Neutron Tool Type for Basic Log Analysis and Mineral Solver, and selects the neutron look-up tables for limestone→sandstone/dolomite matrix and salinity correction (`wellheaderinfo.htm`). A single header dropdown therefore silently changes numerical results.

### 3.4 Array curves

Two dimensions: **X** (samples across, e.g. 12 pad buttons) and **Z** (sub-depth samples within one well step) (`createcurvearraycurve.htm`). Z controls resolution below the well step: in a 6-inch-step well, Z = 6 resolves to the nearest inch, Z = 12 to the nearest half-inch. Core plug data loaded as an array therefore keeps its true plug depth instead of being rounded to the database step — the manual's stated motivation.

When a module is not array-aware, or `Expand Array Data` is off, IP **averages the array over the sample interval** and presents a single value (`curvelisting.htm`, `arraydatacurves.htm`). Example given: array at 0.1″ in a 6″ well → the output value is the mean of the 60 Z-samples. Silent averaging is the default behaviour, not an error.

`arraydatacurves.htm` converts both ways: Curves→Array (combining `ABC001.H … ABC004.H` into array `ABC` by prefix/suffix/number rules) and Array→Curves (one output curve per X sample, named `<base><n>`), with `Output high sample rate data` selecting array-rate vs Z-averaged output.

### 3.5 Audit trail — the History module (`historymodule.htm`, `managecurveheaders.htm`)

- One `IPDBWellxxxx.history` file per well; a single history file type since IP v4.2 (`aboutipdatabase.htm`).
- Columns: **ID, Event, Date, Item, User_Name, Comments**.
- Recorded events include curve creation and every module that updated a curve, so Manage Curve Headers can report per-curve "how it was loaded, what corrections/maths were performed, by whom, on what date/time" (`curvemanagement.htm`). The multi-well curve-header filter exposes *Update Module* and *Update Date From/To* as first-class query fields — the audit trail is queryable, not just readable.
- **Parameter differencing:** select exactly two rows of the same parameter-producing event type and click Difference; IP shells out to **ExamDiff** (PrestoSoft freeware) to show a textual diff of the two parameter states. This is IP's answer to "what changed between these two interpretations".
- **Row Filter (new in IP2025)** exposes the underlying SQL query and lets the user edit it.
- Multi-user conflict resolutions are logged to the well history with the conflicts and the resolutions chosen (`well_multi-user-access-guide.htm`).
- The rationale for deleting Global Parameter Sets from target wells after a batch run is explicitly "the audit lives in the History file" (`global_parameters.htm`) — i.e. IP treats the history as the authoritative record of what was applied.

### 3.6 Locking, concurrency and security

**Pessimistic well lock (default).** Opening a well locks it via `IPDBLock` in the database root. A second user is offered "Open as a New Well", which copies it into memory and, on save, creates an additional well entry. After an IP crash the locks self-clear in **4–5 minutes**; the alternative is deleting `IPDBLock`, which the manual warns must never be done if anyone else might be using the database (`wells.htm`, `aboutipdatabase.htm`).

**Optimistic multi-user access (opt-in, introduced IP 4.3).** `well_multi-user-access-guide.htm`:

- Granting is either per-session (enter a Windows username; the well is saved as part of the grant; **the grant is live for five minutes** and is revoked if the well is not opened in that window; access then lasts only while the well is open) or permanent via the Well Security Manager's *Multi User Access* flag, which can be set on Project Defaults so all new/unsecured wells are shareable.
- Visual state: green dot = shared; hand = another user is connected. Curve-set selectors everywhere show which sets other users are in.
- Recommended etiquette, which is really the concurrency model's precondition: finish all loading, resampling and depth-unit changes **before** sharing; one user per curve set; one user per interpretation module; save important module settings to file because only the last-saved settings survive in the well.
- **Per-user zone-set backups**: if a user creates or modifies a zone set, the next save creates/updates a per-user backup so each user's interpretation parameters can be recovered. These backups are hidden in the browser unless *Backup parameter sets* is enabled in Options (`options.htm`).
- On save, conflicts are grouped into four classes: **Conflicts** (you modified what someone already modified and saved), **Duplicate Names**, **Resampled** (you edited data in a set someone resampled), **Warnings** (your change may lose data others saved). Each has one or more resolvers; renaming resolvers validate names live. **Overwrite** and **Don't Save** are sticky for the remainder of the session. Cancelling abandons the save entirely and the same conflicts will recur.
- State files: `.sharing` and `.sharing-journal` (`intro_file_formats.htm`).

**Well Security** (`managewells.htm`): per-well settings are encrypted, optionally password-protected, and grant each named user none / read-only / full, with an explicit "All Other Users" catch-all and a Project Defaults template. A user with read-only access who loads the well gets a "Copy of" well rather than an error.

---

## 4. Application defaults

### 4.1 Options module — verbatim defaults

Options → General → User Interface `[img-read: _tclip0107.png]`:

| Setting | Default |
|---|---|
| Classic UI | unchecked (Modern UI is the default) |
| Theme | **Silver Spooner** |
| Welcome message | unchecked |
| Database selector on startup | unchecked (last-opened database auto-loads) |
| Fading parameter windows | unchecked |
| Work area close button | **checked** |
| Loaded well count | unchecked |
| Background | **Solid** (vs Windows default) |

Options → More/Miscellaneous `[img-read: _tclip0110.png]`:

| Setting | Default |
|---|---|
| Irregular Set Tolerance, depth wells | **0.2 ft** |
| Irregular Set Tolerance, DateTime wells | **0.5 seconds** |
| Output File Delimiter | **Comma** (alternative: Semi-colon) |
| Use LAS Batch Loader for Drag-and-Drop Files | unchecked |
| Allow anonymous telemetry and crash reporting | **unchecked**; takes effect only after restart |

The irregular-set tolerance is the rule that decides, when loading into an irregular set, whether an incoming depth *is* an existing depth or creates a new one (`options.htm`) — a data-integrity contract, not a cosmetic setting.

Options → Well → Primary Identifier `[img-read: _tclip0111.png]`: Default Primary Well Identifier = **Well Name**, with a per-database override (also Well Name for the example database). Choices are Well Name / UWI / API.

Options → Well → Position: Magnetic model default **IGRF** (alternatives BGGM, which requires a separate BGS licence and `.dat` file, and WMM). Lat/long display style is a global default (DD or DMS) that each well may override — **but the Multi-Well Header module ignores the per-well override and always uses the global default** (`options.htm`, `wellheaderinfo.htm`).

Options → More → LAS 3 Configuration `[img-read: _tclip0133.png]` — the loader's known data-tag tables:

- *Discrete Data Tags* — Depth column: `DEPT` (text tag `CDES`), `DEPTH`, `MD`, `DateTime`, `Time`, `TDEP`, `DPTH`.
- *Interval Data Tags* (Top / Bottom / Text): `TOPT`/`TOPB`, `TSTT`/`TSTB`/`DDES`, `CORT`/`CORB`/`CDES`, `C_TP`/`C_BS`, `PERFT`/`PERFB`, `CORET`/`COREB`/`CDES`, `DTOP`/`DBOT`/`DDES`.

Other Options defaults stated in prose: Database Browser auto-open, Curve Grouping (must be enabled before grouping can be configured in Manage Curve Headers), single-click vs double-click well activation, Parameter set warnings, Backup parameter sets visibility, Use Default Database Path; Default Plots folder points at the user's AppData `Default Plots`; Interactive Line Sensitivity in pixels (tunable for touchscreens); Mask Depths (digit count + offset) for showing tight-hole data to an audience.

### 4.2 Configuration file census (`default_settings.htm`, `tools.htm`, `intro_file_formats.htm`)

| File | Governs |
|---|---|
| `CparmDef.xml` | System curve display defaults on load |
| `CPARMDEF_USER.PAR` | Project-level override of the above |
| `CPARMDEF_METRIC.PAR` / `CPARMDEF_METRIC_User.PAR` | Metric (Canada) curve defaults — µsec/m, kg/m³, mm |
| `CurveType.opt` / `UserCurveTypes.opt` | Curve type vocabulary |
| `CurveAlias.txt` / `DefaultAlias.cax` | Alias vocabulary and the alias grid |
| `MINDEF.PAR` / `MINEQDEF.PAR` | Mineral Solver mineral systems and equations |
| `Overlay_Files.ovl` / `.ovlx` | Crossplot overlay registry |
| `MonteCarloDefaults.par` | Monte Carlo parameter selection (**by ordinal**), distribution type (Gaussian/Square/Triangular), high/low shifts, output percentile convention, default plots |
| `UnitConversion.par` | Recognised unit abbreviations and conversion factors |
| `Neu_Parm_Files.neu` | Neutron tool look-up tables |
| `Lithology.opt` / `Lithology_xxxx.opt` | LITH value → bitmap mapping |
| `ShadeType.opt` | Colour/pattern/bitmap fills |
| `Zonecolors.opt` | Zone colour palette (25 default colours; 90 available on the zone editor) |
| `DipSymbols.opt` | Tadpole symbol table |
| `ProgDefs.opt` / `DefaultUnits.opt` | Program/unit settings |
| `ProjectFileDefaultsSets.opt` | Global (curve) Set list |
| `SetDictionary.xml` / `Sample Set Dictionary.xml` | Curve-set dictionary |
| `Intpetro.config` (+ `*.configUpdate`) | Attributes, mappings, Corporate Folders switch |
| `CorporateFolders.xml`, `ClientFiles.txt`, `dbHistory.xml`, `DBList.ini`, `IPtoolbar.ini`, `SettingsFiles.txt` | Deployment and session state |
| `FluidSub_Default_Parameters.par`, `Poisson_Ratio_Lithologies.par`, `NMR_Tools.csv`, `OBG_Files.obg` | Module-specific default tables |

IP's canonical internal units are **g/cc, µsec/ft, inches**; crossplot overlay disk files must be authored in those units and are converted on load (`tools.htm`, `default_settings.htm`).

### 4.3 Save, backup and upgrade behaviour

- **Save Reminder** minimum interval **5 minutes**; it is a per-session setting and does not persist across restarts (`fileoptions.htm`). There is no autosave.
- The recent-database list holds **10** entries (`fileoptions.htm`).
- **Database Upgrader** (`installandregisterip.htm`): handles "Interactive Petrophysics 3.6 and later well files"; backs up each database folder by default into a sub-folder named e.g. `4.7 Upgrade Backup 13 May 2022`; upgrades wells in numerical file order; a well-level error is logged and the run continues; a backup or connect failure abandons that whole database; the machine is kept awake and a shutdown stops the run cleanly after the current well; CSV log; `databaseupgrader /?` for a command-line mode.
- **Compatibility is one-directional.** "The current version of IP only provides backward compatibility… IP does NOT provide forward compatibility" (`installandregisterip.htm`). Saving a well from an older database prompts an ad-hoc upgrade — which, unlike the bulk tool, takes **no backup**; the manual recommends the bulk tool for exactly that reason (`intro_getting_started_-_step_by_step.htm`).

### 4.4 Installation, licensing and deployment

`installandregisterip.htm`, `using_the_license_server_manag.htm`:

- Default install path `C:\Program Files\IntPetroXX\`, **660.7 Mb**. Requires Windows 10 2004+, .NET 4.8 Full, 64-bit only; minimum 16 GB DDR5, recommended 32 GB. (`wells.htm` separately gives the older working-memory guidance: 512 MB suffices for conventional curves, ≥2 GB for acoustic/electric image data.)
- Three licence models: **Personal** (Customer License ID, activates against the Geoactive licence server), **Shared Network** (on-premises LiMBR server), **Cloud** (subscription, Geoactive-operated).
- **LiMBR License Server**: service + management console, either or both installable; default TCP port **11362**, changeable via a `Port` DWORD under `HKEY_LOCAL_MACHINE\SOFTWARE\Geoactive\Geoactive Licence Server` (decimal) plus a firewall change, after which clients address `hostname:port`. One console manages many servers; one server can serve IP, IC, IM and RiskSpectrum. Six admin tabs: Server, Settings, Features, Users, Reporting, Error Log. Feature logging is **on by default**; usage logs can auto-delete or auto-anonymise (usernames and hostnames → GUIDs) after N days, and the whole reporting-admin surface can be locked out remotely with a `DisableReportingAdmin` DWORD so it can only be changed at the console. A Reporting API key allows third-party reporting tools.
  *(Default admin credentials and API-key mechanics are described in the manual; not transcribed here.)*
- **Silent/managed deployment**: a setup folder containing `CorporateFolders.xml`, `ProgDefs.opt`, `dbHistory.xml`, `Intpetro.config` (plus those filenames added to `ClientFiles.txt`) is applied by launching the installer with `/A` (or `/A:"<path>"`), which copies it over the install folder. A machine environment variable `IP_LICSERVER` set to the licence-server hostname suppresses the licensing dialog entirely.
- Corporate Folders are switched on by `<add key="EnableCorporateFolders" value="True" />` in `Intpetro.config`.

### 4.5 Keyboard shortcuts (`intro_short-cut-keys.htm`)

`F1` help · `F5`/`F6` previous/next work area · `F7` new work area · `F8` close work area · `F9` annotate active window · `F10` close active window · `F11`/`F12` previous/next well.
`Ctrl+B` Basic Log Functions · `Ctrl+N` Create Well · `Ctrl+O` Load Wells · `Ctrl+L` Log Plot · `Ctrl+P` Horizontal Log Plot · `Ctrl+Y` Crossplot · `Ctrl+H` Histogram · `Ctrl+D` Drag/drop curves · `Ctrl+U` User Formula · `Ctrl+W` Well Diagram Manager · `Ctrl+T` Well Notes.
`Ctrl+Alt+`: `C` Clay Volume · `P` Phi Sw · `X` Cutoff & Summation · `M` Monte Carlo · `T` Cluster Analysis · `A` PCA · `S` Self-Organising Maps · `N` Neural Networks · `R` Multiple Linear Regression · `F` Fuzzy Logic · `E` Elastic Impedance · `W` Fluid Substitution · `L` Laminated Fluid Subs · `D` Rock Physics Density Estimation · `U` Rock Physics Create Time Curve · `V` Synthetic Seismic · `O` Overburden Gradient · `G` Pore Pressure Gradient.

### 4.6 PPFG project conventions (`project_setup.htm`)

A mandated curve-set naming standard: `CKSHT, DEFAULT, DIRECTIONAL, EDIT, EVAL, FPIT, FPIT_PROJ, FPRESS, FPRESS_PROJ, InvCkSht, LWD, LWD_RT, MUDLOG, PPFG, WLRAW`. The PPFG set must be a regularly sampled depth set running from drill floor to **at least 200 ft beyond expected TD**, typically at **1 sample/ft**. Duplicate FPRESS depths must be offset by **0.01 ft** so no two pressure samples share a depth.

---

## 5. Module census

### 5.1 Modern UI navigation groups (`intro_new_modern_ui.htm`)

The most complete single inventory in the manual. Groups and their contents:

- **Browser** — toggles the database browser pane.
- **Database** — Create/Open Database, Edit Database Connection, Create Database Shortcut, Load/Save Session, Explore Database Folder, Save Current Well (+ As), Save All Wells, Save Wells To, Save Reminder, Exit.
- **Well** — Load Wells from Database, Create New Well, Select Well, Create/Edit Well List, Save/Close/Delete/Reset Well, Manage Well Header, Manage Curve Sets, Manage Curve Headers, Manage Zones/Picks, Link Zone Sets, Delete Curves, Copy Curves from Well, Plot Range Editor, Date/Time Format Setup, Take Notes; sub-menus for Multi-Well Options and Delete Parameter Sets.
- **In/Out** — Import: ASCII, LAS/LBS, LAS3, LIS, DLIS, FT LIS/LAS/DLIS, DBASE 4, BLA, Kingdom Connector, LAS and ASCII Batch Loaders, Zone Tops, Picture Curves, Interval/Spreadsheet, Cap Pressure, Well Attributes, XStream Connect (WITSML real-time). Export: ASCII, LAS, LIS, DLIS, DBASE 4, Kingdom, Zone Tops, Composite Plot data. Other: Export Time curves to Depth Well, **Petrosys Exchange**, IP-IC Common Database, and connectors to ODM/IC, GEOLOG 6, OpenWorks, LOGIC, Petrolog, Geolog ASCII, OpenSpirit, Petrel. Parameter-set export and reports also live here.
- **View** — Log Plot, Horizontal Log Plot, Well Diagrams, Histogram, Crossplot, Ternary, Rose, Star, Box, Pie; IP Query, Multi-Well Correlation Viewer, 3D Parameter Viewer, Well Map, Plot Composer, Montager, Header Editor, Curve Listing/Edit, Curve Stats, Custom Menus.
- **Edit** — Interactive Curve Edit, Baseline Shift, Trend/Square and Auto Trend/Square, Depth Shift tools, Curve Splicing, Create Curve/Array/Lith/Point curves, Filter, Average, Rescale, Fill Gaps, Log QC, Auto-Edit, De-Spike; Array Image Data (legacy, superseded by IA); Picture Curves.
- **Calculation** — User Formula, Multi-Line Formula, Basic Log Functions, Temperature Gradient, Rw from SP, Gas Analysis, TVD tools, Curves from Zones/Parameters, Curve Integration, Numeric Facies to Text; Environmental Corrections (per logging contractor).
- **Interpretation** — Basic Log Analysis, CO2 Storage, TOC, Vclay, Porosity/Saturation, Cutoff/Summation (+ their parameter sets), NMR Interpretation, Monte Carlo and Batch Monte Carlo, 3D Petrophysics, multi-well Cutoff and Summation; Cased Hole → Sigma and Sigma Time Lapse; Unconventional Resources Toolkit.
- **Advanced Interpretation** — Mineral Solver, HFU, East European Resistivity Corrections, Formation Testing, Sand/Silt Malay Model; Saturation Height (CP): Setup / Function Fitting / Sat-vs-Height; Saturation Height (Log Curves): Function Fitting / Sat-vs-Height.
- **Imaging** — the Image Analysis workflow.
- **Geomech** — multi-well Geomechanics, legacy single-well Pore Pressure, PPFG toolbox, Geosteering.
- **Geophysics** — Rock Physics, Synthetic Seismogram, Acoustic Waveform Processing.
- **Cased Hole** — Cement Evaluation, Casing Inspection, Production Logging, Sigma Sw, C/O Sw, Well Diagram Manager.
- **Machine Learning** — Curve Prediction and Rock Typing: Experienced Eye, Fuzzy Logic, MLR, Neural Networks, Cluster Analysis, Self-Organising Maps, PCA, Contingency Table, Textural Facies Analysis, Domain Transfer Analysis.
- **Multi-Well** — multi-well Well Header / Curve Sets / Curve Headers / Zones-Picks / Zone Set Links, Batch Plotting, Batch TVD, 3D Petrophysics, parameter management and distribution, Correlation Viewer.
- **Apps** — create/edit/run User Apps (multiple languages).
- **Tools** — themes; Shading Types, Global Sets, Zone Colours, Dip Symbols, Palettes, Default Lithology, Curve Aliasing; Default Units and Printer; Graphical Workflow Designer; Options; Defaults (the config-file shortcuts); Licensing; Custom menus.

### 5.2 Tools menu, module by module (`tools.htm`)

| Tool | What it does |
|---|---|
| Defaults → 8 shortcuts | Direct editing of `CparmDef.xml`, `CurveType.opt`, curve alias config, `MINDEF.PAR`, `MINEQDEF.PAR`, `Overlay_Files.ovl`, `MonteCarloDefaults.par`, `UnitConversion.par`, `Neu_Parm_Files.neu`. |
| Shading Types | Colour/pattern/bitmap fill definitions (`ShadeType.opt`). Shade names <10 chars; bitmaps 16×16 px. **All log plots must be closed before edits can be saved.** Without *Save as Project Default*, edits overwrite the IP-wide file. |
| Global Sets | Propagate named curve sets into every in-memory and subsequently loaded well (§2.8). |
| Zone Colors | `Zonecolors.opt`, 25 default colours, extensible. |
| Dip Symbols | `DipSymbols.opt` — tadpole colour/shape/name per value; values need not be sequential. |
| Palettes | 256-colour palettes, interactive (two-endpoint ramp) or manual per-index. |
| Default Lithology | Maps LITH index numbers to shading names and descriptions (`Lithology.opt`). Index numbers must be unique; the tool warns on duplicates. Only edits the default file — custom `Lithology_xxxx.opt` files must be produced by copy-edit-rename outside the tool. |
| Curve Aliasing | The alias grid and its three modes (§2.11); Auto Populate sweeps the whole database; imports/exports via Excel CSV. |
| Set Default Units | How to interpret Sonic/Density/Caliper curves with unrecognised units; also drives the units in which parameters are *displayed*. Mixed units still compute correctly but break interactive parameter picking. |
| Set Default Printer | Session-scoped default printer (log plots excepted). |
| Graphical Workflow Designer | Builds workflows for the Graphical Workflow Manager. |
| Custom Menus / Toolbars | Drag-built menus (both UIs) and toolbars (Classic UI only). |
| Message Board | Event log; opens automatically on error; Stay on Top / Show Automatically / verbose toggles. |
| Licensing | View status, install/return, borrow/return. |
| Options | The main configuration module (§4.1). |

### 5.3 PPFG Toolbox utilities (`utilities_modules.htm`) — 10 modules, with their defaults

Pressure Limit default **50,000 psi**; Fit Gradient to MDT falls back to **0.465 psi/ft** when a fit cannot be made; % OVBD range 0–150 %; RHOB Shale Picker window max 100 ft with bias capped at 2σ; psi/ft → lb/gal factor **0.052**; mud-gas normalisation `GAS(normalized) = (Gas × Flow rate × 5.0028) / (ROP × Diameter²)`; seven temperature models with published equations (e.g. GoM Deepwater `BHT = 0.011 × TVDbml + 39`); ResVel Regression defaults — window 1000, step 5, slope −65, GR 30/120, Rt 0.3/18, DT 50/190. The three shale-picker modules (acoustic/resistivity/Dxc to pressure) share one algorithm: statistical shale-value picking in windows defined by *Number of ft to reduce to a single value*, with GR/RHOB/Rt discriminators and a bias of up to ±2σ, then linear interpolation and optional HP pentadiagonal-inversion smoothing.

### 5.4 Query and mapping

- **IP Query** (`ip_query.htm`) — builds and searches a cross-project index held in a **Microsoft SQL Compact Edition `.sdf`**; the gather runs asynchronously; results export as a well list (`.wst`) or project (`.prj`).
- **Well Queries** and **Map Toolbox** (`well-queries.htm`, `toolbox.htm`) — belong to the *new IP2025 Mapping module*, which is derived from IC and written in IC's terminology. They are a second, parallel query surface, not an evolution of IP Query.

---

## 6. Version intelligence (What's New)

`intro_whats_new_in_ip.htm` carries the complete ladder IP2025 back to IP3.6.

**IP 2025 — new modules:**
- **Mapping** (separately licensed) — the IC-derived mapping/well-query capability.
- **Mud Gas Normalization** (AGIP method).
- **Plot Pinning.**
- **Petrosys Exchange** — OSDU-capable data exchange.
- **Well & Set Grouping.**

**IP 2025 — notable upgrades:**
- **Mineral Solver maximum user models raised 20 → 50.**
- **Python 3.12** for the scripting/User App runtime.
- History module gains the editable **SQL Row Filter**.
- Clay Volume organic-shale correction extended to the sonic — new parameters #71 `Sonic Kerogen`, #72 `Sonic Heavy_Min.`

**Earlier milestones relevant to this domain:**
- **IP 2023** — 32-bit deprecated; help moved to a cloud-hosted location.
- **IP 2021** — the Modern UI (sidebar navigation replacing menus/toolbars).
- **IP 4.3** — **Multi-User Access** introduced.
- **IP 4.2** — text curves merged into ordinary curves; single well-history file type.
- **IP 3.3** — Manage Well Header Info re-engineered to be attribute-driven and customisable.

Documentation artefacts: the shipped PDF is `IP Help Manual v40.pdf` and the CHM is `Interact.chm` (`intro_file_formats.htm`).

---

## 7. Abbreviations dictionary

`abbreviations_and_definitions.htm` is the manual's only glossary and it is **geomechanics-only** — there is no general petrophysics symbol table anywhere in the CHM. Complete contents:

| Symbol | Name | Definition (paraphrased) |
|---|---|---|
| α | Biot Factor | Correction for imperfect pressure support in the rock matrix. |
| ν | Poisson's Ratio | Measure of the Poisson effect — expansion vs compression of the rock. |
| σH or σhmax | Maximum Horizontal Stress | Principal stress, usually aligned with fracture direction. |
| σh or σhmin | Minimum Horizontal Stress | Principal stress, 90° to σH. |
| σv | Vertical Stress | Principal stress from the overburden weight above the zone. |
| γ | Stress Path Factor | Proportional relation between changes in horizontal principal stress and changes in reservoir pressure (Δσh/ΔP). Also called the Depletion Constant. |
| BHFP | Bottom Hole Flowing Pressure | Fluid pressure in the wellbore. |
| CBHP | Critical Bottom Hole Pressure | BHFP at which the rock may fail. |
| CDP | Critical Drawdown Pressure | Drawdown at which the rock may fail; varies with reservoir pressure. |
| E | Young's Modulus | Stiffness of an isotropic elastic material. |
| Pres / P0 / Pp | Reservoir Pressure | Fluid pressure in the section of interest. |
| — | Reservoir Pressure Path | How BHFP and reservoir pressure vary over an extended period. |
| Depth | Measured Depth | Along-hole depth from drill-pipe length; diverges from true depth in deviated wells. |
| TVDSS | True Vertical Depth Sub-Sea | True vertical depth relative to sea level. |
| TVDKB | True Vertical Depth Kelly Bushing | True vertical depth relative to the drilling reference elevation; differs from TVDSS onshore. |
| TWC | Thick Wall Cylinder | Core-plug test simulating wellbore collapse pressure; can be correlated from UCS. |
| UCS | Uniaxial Compressive Strength | Measure of rock strength. |

---

## 8. Internal discrepancies

1. **Curve Sets per well: 500 vs 50.** `aboutipdatabase.htm` and `intro_curves_and_curve_sets.htm` both state 500. `curvemanagement.htm` states "A maximum of 50 Curve Sets can be created per well." Two pages against one, and the 500 figure appears in the dedicated limits list — treat 500 as correct and 50 as stale.
2. **Curve Set Short Name length: 8 vs 4.** `managecurvesets.htm` and `tools.htm` (Global Sets) both say ≤8 characters. `manage-multi-well-curve-sets.htm` instructs "type in a character string (max. 4 characters and must not start with a number)" for New Set Name. The 4-character rule appears only in the multi-well rename workflow; the "must not start with a number" constraint appears nowhere else and may be real. Unresolved — see OPEN ITEMS.
3. **Lithology shadings maximum: 39 vs 80, in one page.** `tools.htm` line ~111 (Shading Types section): "a maximum of 39 Lithology shadings allowed in the Edit Default Lithology table." Line ~216 (Default Lithology section): "an upper limit of 80 lithology shadings permitted in the Edit Default Lithology table." Line ~221 confirms the shipped `Lithology.opt` contains 39 bitmaps. The 39 figure is the shipped count leaking into a limit statement; 80 is the limit. Confirmed as a long-standing defect — see §9.
4. **Two different Parameter Set type enumerations.** `parametersets.htm` lists `Tops, Clay, PhiSw, Cutoff, Splice, Basic_Loganal, TDT_Stand_Alone, TDT_Time_Lapse, NMR, MinSolve, Pore_Pres_Grad, UP******` — no `TVDss_Set`, no `MonteCarlo`. `manage_zones_and_picks.htm` lists `Tops, TVDss_Set, Basic_loganal, Clay, PhiSw, Cutoff, UP, MonteCarlo, MinSolve, NMR, Pore_Pres_Grad, TDT_Stand_Alone, TDT_Time_Lapse` — no `Splice`. Neither list is a superset. Identical divergence exists in IP2018.
5. **Four PL pages sit in this bucket by title collision.** `user_set_up.htm` ("Preferences") is *Production Logging* preferences, not IP application preferences; `project.htm`, `open.htm` and `workflow.htm` are PL project/run/workflow pages. Content routed to agent M.
6. **`managewellheaderinfo.htm` is not the Manage Well Header Info module** — it is a Fortran 77 User App worked example that writes header fields. The real module page is `wellheaderinfo.htm`. Content overlaps agent G's User Apps domain.
7. **`toolbox.htm` and `well-queries.htm` document the IC-derived Mapping module**, not IP core, and use IC terminology throughout. They are IP2025-new but describe a separately licensed product surface.
8. **Silent failure documented as behaviour.** Multi-well tops paste: rows whose well name cannot be matched are ignored and **"no error message is given"** (`manage-multi-well-zones_picks.htm`). This is stated, not implied.

---

## 9. IP2018 → IP2025 numeric diff

Method: HTML-stripped regex comparison of matched page pairs in `Temp\c18` and `Temp\c25`, plus the ordinal cross-check in §2.9.

**Unchanged** (verified identical wording and figures): all database limits (9,999 / 2,000 / 20,000 / 500 / 3 M / 50); Curve Set Short Name 8 characters; Save Reminder minimum 5 minutes; the 80-shading upper limit; the shipped 39-bitmap `Lithology.opt`; the Parameter Set type list on `parametersets.htm`; the ExamDiff parameter-differencing mechanism; the `IPDBLock` 4–5 minute self-clear.

**Changed:**

| Item | IP2018 | IP2025 |
|---|---|---|
| "maximum of N Lithology shadings" (Shading Types section) | **30** | **39** |
| Vendor / licence-server branding | "LR software products" (Lloyd's Register), *LiMBR License Server Manager* | **Geoactive**; *Geoactive License Server Manager*; registry key `HKLM\SOFTWARE\Geoactive\Geoactive Licence Server`; support at `geoactive.com` |
| Licence-server product family | IP and IC | IP, IC, **IM, RiskSpectrum** |
| ClayVol parameter #39/#40/#41 | `OD Ot1 Clay` / `OD Ot2 Clay` / `OD Ot2 Clean1` | `OD Curv1 Clay` / `OD Curv2 Clay` / `OD Curv1 Clean1` |
| ClayVol parameter count | 70 ordinals | **72** (adds #71 `Sonic Kerogen`, #72 `Sonic Heavy_Min.`) |
| History module row filtering | not present | editable **SQL Row Filter** |
| Mineral Solver max user models | 20 | **50** |

Note that the internal 39-vs-80 lithology contradiction is *older* than IP2025 — in IP2018 it read 30-vs-80. The smaller number has been bumped once without the discrepancy ever being reconciled.

**Pages entirely new since IP2018** (absent from `c18`), all in this bucket: `well_multi-user-access-guide.htm`, `manage-multi-well-working-sets.htm`, `manage-multi-well-zone-set-lin.htm`, `global_parameters.htm`, `well-queries.htm`, `intro_new_modern_ui.htm`, `utilities_modules.htm`, `project_setup.htm`. The shape of the last seven years of IP development in this domain is therefore: **multi-user concurrency, multi-well batch parameterisation, and a PPFG/geomechanics vertical.**

---

## 10. SandiBumi notes

Ordered by how much they should change what we build.

1. **Adopt "a parameter set is a zone set with columns" — but fix the two things IP got wrong.** IP's model is sound and is why its audit story works: parameters are versioned with the zonation that produced them, and a parameter set is a single serialisable object (`.set`). Two defects worth not inheriting: (a) *cross-type load silently resets every parameter to internal defaults* while keeping the zonation — a plausible-looking set that is numerically empty; (b) *switching parameter set in Multi Well Change Parameters before committing discards edits with no warning*. Both are silent-loss failures of exactly the kind our data-integrity contract forbids.
2. **Copy the parameter-value polymorphism wholesale — constant | curve | header-lookup | tilted | log-tilted — and make tilt a first-class value, not a UI mode.** The `Lg` prefix for logarithmic interpolation (used for Rw) is the right primitive: the interpolation law travels *with the value*, so a Rw that is tilted across a zone is unambiguously log-interpolated wherever it is read. Our Sw work in the Mahakam delta routinely needs Rw trending across a thick interval; `top:bottom` with a declared law is better than either a constant or a hand-built curve. Note the IP boundary rule to preserve: **interpolation is within-zone only; parameters step at zone boundaries.**
3. **Ordinal parameter IDs are the sleeper feature.** IP addresses every parameter within a set type by a permanent, sparse ordinal — never renumbered, gaps left where parameters retire — and that ordinal is what `MonteCarloDefaults.par` and the uncertainty machinery reference. It is a stable external handle that survives renames (proven: #39–41 were renamed and kept their numbers). If SandiBumi wants sensitivity analysis, batch parameter overrides, or a scriptable API, we need the same thing: a numeric or GUID parameter handle decoupled from display name. Retro-fitting one after names are in circulation is painful.
4. **`H_module_parameter_reference.json` is not superseded by this ingest — keep it.** Six ClayVol ordinals (#51–54, #56, #70 — the Link/Average/Mix/Percentile controls) exist only in the `.hlp` extraction; the 2025 manual documents them in prose without numbers. The PhiSw `.hlp` carries 188 numbered parameters against roughly 27 recoverable from the manual. For parameter-level fidelity the `.hlp` extraction remains the better source; the CHM is the better source for *behaviour*.
5. **Curve resolution needs a declared, inspectable precedence chain.** IP's is: Working Input Set → alias-grid modes (Off / Manual `#name` / Automatic) → Final-only filter → set-search sub-mode (grid order / input set only / input set first) → curve *Type* with most-recently-modified tie-break. That is powerful and completely opaque at run time — a module silently picks a different curve depending on three separate settings. SandiBumi should implement the same capability but **log the resolution decision per curve per run**, so a deliverable can answer "which GR did this Vsh actually use".
6. **Overwrite must mean one thing and say which.** IP's "Overwrite" on curve load is *replace-and-concatenate* (splice), not whole-curve replace. It is a reasonable default for log data and a terrible surprise if you expect the other. Whatever we choose, name it explicitly (`splice` vs `replace`) rather than reusing the word "overwrite".
7. **Array curves: never average silently.** IP averages array data over the well step whenever the consuming module is not array-aware — 60 Z-samples collapsed to one mean with no flag. For core-plug and image data that is a real loss of information at the point of use. If we average, emit a marker curve or a run-log entry.
8. **Two-tier locking is the right shape for our deployments.** Pessimistic per-well lock as the default (with a self-clearing timeout, because crashes happen), optimistic multi-user as an opt-in per well or per project. IP's four conflict classes — Conflicts / Duplicate Names / Resampled / Warnings — are a good taxonomy, and *Resampled* in particular is the class nobody thinks of until it bites. Its **per-user zone-set backup on every save** is cheap insurance we should copy: parameter sets are small and the recovery value is high.
9. **The attribute + mapping architecture is directly reusable and better than what we have.** Three namespaces (well / log / curve), each split into fixed-for-compatibility and user-extensible, mediated by a File Loader mnemonic table and three mapping tables, all in one XML file, with a drop-in `.configUpdate` distribution mechanism that needs no server. That is exactly the shape our mnemonic super-dictionary work is converging on. The failure mode to avoid: **an attribute value silently changing numerical results** — IP's Logging Contractor dropdown reselects neutron look-up tables and crossplot overlays. If an attribute drives physics, it must be surfaced in the run record, not buried in a header tab.
10. **Defaults precedence: project-over-system, with an explicit corporate tier above both.** IP's chain (Corporate Search Folders > project file > user/system file) is right, and Corporate Folders solves the real problem of keeping a team on one set of endpoints. The defect to avoid: **an unchecked "Save as Project Default" box overwrites the system-wide defaults file**, and the only way to revert a project to system defaults is to close it and delete the file by hand. Make the scope of a save explicit and make reverting a command.
11. **History as first-class, and diffable.** IP logs ID/Event/Date/Item/User/Comments per well, makes Update Module and Update Date queryable in bulk edit filters, and can diff two parameter states — but only by shelling out to third-party ExamDiff. A native structured parameter diff (which zone, which parameter, old → new) is a small amount of work and a large differentiator, given that IP's own docs treat the history file as the authoritative record of what a batch run applied.
12. **Ordinal-addressed uncertainty config.** `MonteCarloDefaults.par` stores, per parameter ordinal, a distribution type (Gaussian / Square / Triangular) and high/low shifts, plus the output percentile convention (note IP's asymmetry: the 10th percentile is the 10th-lowest of everything **except Sw, where it is the 10th-highest**). If we build uncertainty analysis, that inversion for saturation is the convention users will expect.
13. **Housekeeping conventions worth stealing:** `Final` flag on curves plus a Final-only search mode; a `Lock` flag making a curve read-only; statistics that ignore nulls and a Net Interval defined as non-null count × step; bulk null-curve deletion by sorting on mean; well lists (`.wst`) as reusable, saveable selections that flow between modules.

**Smectite / montmorillonite (rule 7).** No smectite or montmorillonite endpoint appears on any of my 71 pages. The single mention is `intro_whats_new_in_ip.htm` line 178, "Alberty Smectite/Illite models", as a release-note feature name with no numbers attached. All actual endpoints live on other agents' pages: `acoustic_to_pressure.htm`, `density_estimation.htm`, `overburden_tools.htm`, and `resistivity_to_pressure.htm` — the last carrying a stated default RHOma of 2.59 g/cc for smectite and kaolinite. Routing note for agents B and J.

---

## 11. OPEN ITEMS

1. **Curve Set Short Name: 4 or 8 characters in the multi-well rename path?** `manage-multi-well-curve-sets.htm` says max 4 and "must not start with a number"; every other page says 8 with no leading-character rule. Cannot be resolved from the manual; needs testing against the application or an IP database.
2. **Curve Sets per well: 500 or 50?** Two pages against one (§8.1). The 50 figure may be a genuine limit on a specific creation path in `curvemanagement.htm` rather than stale text.
3. **`(Parameter #N)` ordinals for the Cutoff module.** `H_module_parameter_reference.json` has 220 Cutoff parameters with `n: null` — the `.hlp` extraction recovered no ordinals for that module — while `cutoffsandsummation.htm` does print parenthesised numbers. A regex pass recovered too few to cross-check. Re-extracting Cutoff ordinals from `cutoffsandsummation.htm` with a hand-tuned parser would complete the three-module set. Flagged for agent D, whose page it is.
4. **PhiSw ordinals above the ~27 recovered.** The `.hlp` has 188 numbered PhiSw parameters (max n = 189, so one gap); the manual prints them but with formatting my regex only partially matched. All 27 recovered matched exactly, so confidence in the numbering is high, but the full mapping is unverified.
5. **Zone Sets per well = 500** appears in `intro_curves_and_curve_sets.htm` but the equivalent sentence was not found in the IP2018 page during diffing — unclear whether that limit is new or simply worded differently in 2018.
6. **`Splice`, `MonteCarlo` and `TVDss_Set` set types.** Neither type list is complete (§8.4). Whether `Splice` and `MonteCarlo` sets are true persisted types or transient is not established anywhere in the manual.
7. **Images not read.** Roughly 700 content images are attached to these 71 pages. I read four (`_tclip0107`, `_tclip0110`, `_tclip0111`, `_tclip0133`) plus the splash `hmfile_hash_5f73c0c6`, selecting the panels that carry actual default values or lookup tables. The remainder are dialog screenshots whose settings are already described in prose, workflow illustrations, or licence-server admin screens deliberately skipped. Not read but potentially carrying values: `_tclip0108` (Database options states), `_tclip0109` (Plotting defaults — default scale, symbol, line sensitivity), `_tclip0112` (Position defaults), `_tclip0115`–`_tclip0126` (the Attributes reference tables — these are the actual fixed-attribute name lists, the single largest unread dataset in this bucket), `_wmclip0024` (Well header Default Parameters tab), `_psclip00012`–`_psclip00020` (parameter-set dialogs).
8. **The complete fixed Well/Log/Curve Attribute name lists** are only available as images (`_tclip0115` onward) and in `Intpetro.config` on an installed system. If SandiBumi wants to mirror IP's attribute vocabulary, reading those images or the installed config file is the way to get it — the local IP 2025 install at `C:\Program Files\IP2025` would give it directly and authoritatively.
9. **Default admin password and Reporting API key mechanics** for the LiMBR server were deliberately not transcribed per the credential rule. The registry lockdown mechanism (`DisableReportingAdmin`) is documented above because it is a policy control, not a credential.
