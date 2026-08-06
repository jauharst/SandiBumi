# N — Data I/O: Loaders, Exporters, External-Database Bridges

Ingest of the Interactive Petrophysics 2025 vendor manual (decompiled CHM), agent N of 14.
Consumer: SandiBumi / SegaraBumi LAS/DLIS ingest and unit/null discipline.

**Provenance convention.** Every fact carries its source. `(pagename.htm)` = vendor prose.
`[img-read: file.png]` = transcribed by reading the PNG directly (vision). Facts with no
source tag do not exist in this document. Where prose and screenshot disagree, both are
recorded and the conflict is listed in §6.

**Delegation note.** This extraction ran entirely on the session model. No subagents: a
mis-transcribed null value or unit rule is silently wrong and verification would mean
re-reading the same page, which is precisely the case the delegation rule excludes.

---

## 1. Scope & page inventory (34/34 accounted for)

All 34 assigned pages were read in full from `<stem>_text.txt`. 18 images were opened and
transcribed. Nothing was copied out of `c25`/`c18`; no vendor file was modified.

| # | Page (stem) | Title | Read | Images read |
|---|---|---|---|---|
| 1 | `datasaving` | Export Data | yes | `_dsaclip0030`, `_dsaclip0038`, `_dsaclip0043`, `_dsaclip0050`, `_dsaclip0005`, `_dsaclip0054`, `_dsaclip0055`, `_dsaclip0056` |
| 2 | `petrel_interface` | Petrel Link 2013 and older | yes | — |
| 3 | `load_ascii_data` | ASCII Load | yes | `_dlclip0005` |
| 4 | `dlis_loader` | DLIS Load | yes | `_dlclip00057` |
| 5 | `connectionprotocols` | External DB Connection Protocols | yes | (0 content images) |
| 6 | `read_write_via_openspirit` | Import/Export via Openspirit | yes | — |
| 7 | `read_write_to_geolog_db` | Import/Export to GEOLOG DB | yes | — |
| 8 | `geolog_ascii_loader` | Geolog ASCII Load | yes | `_dlclip0162` |
| 9 | `witsml_import` | WITSML Import | yes | — (credential screens deliberately not transcribed) |
| 10 | `read_from_openworks_db` | Import/Export to Openworks DB | yes | — |
| 11 | `load_las_data` | LAS Load (LAS3) | yes | `_dlclip00031`, `_dlclip00032` |
| 12 | `load_lis_data` | LIS Load | yes | `_dlclip00034` |
| 13 | `las_batch_load` | LAS Batch Load | yes | `_dlclip00194` |
| 14 | `las_lbs_load` | LAS/LBS Load | yes | `_dlclip00204` |
| 15 | `batch_ascii_loader` | ASCII Batch Load | yes | `_dlclip0149` |
| 16 | `dbase_4_loader` | DBASE 4 Loader | yes | `_dlclip00042` |
| 17 | `intervalloader` | Interval / Spreadsheet Loader | yes | — |
| 18 | `loadwellattributes` | Load Well Attributes | yes | — |
| 19 | `import_oh` | Import OH | yes | — |
| 20 | `import-wells-from-zip-file` | Import Wells from ZIP | yes | — |
| 21 | `kingdom_importer` | Kingdom Importer | yes | — |
| 22 | `witsmllink` | XStream Connect | yes | — (credential screens deliberately not transcribed) |
| 23 | `read_from_logic_db` | Import from LOGIC DB | yes | — |
| 24 | `read_write_from_odm_database` | Import/Export to ODM/IC DB | yes | — |
| 25 | `read_write_to_petcom_db` | Import/Export to PETCOM DB | yes | — |
| 26 | `readfrompetrolog` | Import from Petrolog DB | yes | — |
| 27 | `petrel_link_2014_and_later` | Petrel Link 2014+ | yes | — |
| 28 | `pc_configuration` | PC Configuration for External DBs | yes | — |
| 29 | `connecting-to-external-databas` | Connecting to External DBs (stub) | yes | (0 content images) |
| 30 | `export` | Export hub (PL) | yes | (0 content images) |
| 31 | `input___output` | Import / Export hub | yes | (0 content images) |
| 32 | `dataloading` | Import Data hub | yes | — |
| 33 | `intro_file_formats` | File Formats | yes | (0 content images) |
| 34 | `powerlogloglanconverter` | PowerLog/Loglan Converter | yes | — |

**Smectite / montmorillonite:** a case-insensitive sweep of all 34 raw `.htm` files returned
**zero** hits. No clay endpoints live in the data-I/O page set; they belong to the
mineral-solver / core-petrophysics agents.

---

## 2. Per-format conventions (null / depth / unit / naming per loader)

### 2.1 ASCII Load (`load_ascii_data.htm`)

**Null.** User-settable "Null Value (absent value) which is used in the ASCII file being
loaded. The default value is -999.00." (`load_ascii_data.htm`). The panel screenshot shows
the field pre-filled as `-999` (`[img-read: _dlclip0005.png]`) — see §6.1.

**Implicit null from delimiters.** With any delimiter other than Spaces, blank entries are
ignored, and *two adjoining delimiters imply a null between them*. The manual's worked
example: a comma-delimited line `1000.0,55.2,,100` loads as `1000.0  55.2  -999.0  100.0`
(`load_ascii_data.htm`). This is a silent null-injection path.

**Depth.** Data "will be loaded to the closest sample increment in the well" — IP snaps to
the destination well step; it does not preserve source depths in a regular set
(`load_ascii_data.htm`). Data may be in any depth order in the file. Unreadable lines are
skipped silently and the loader proceeds to the next line.

**Depth/time index naming — mandatory.** The depth curve *must* be named `DEPTH` in the
Curve Name row (it need not be the first column); for time-indexed wells it must be `TIME`
(`load_ascii_data.htm`). If a `DEPTH` curve exists in the file but the Reference Depth Curve
is set to something else, IP errors; the documented workaround is to rename the column to
something IP will not read as measured depth, e.g. `DEPTHin`.

**Units.** The units string on the DEPTH/TIME column drives conversion: `M` = metres,
`FT` = feet, `MS` = milliseconds, `SEC` = seconds (`load_ascii_data.htm`). "A metric data
file can be loaded into an imperial-units database and the data will automatically be
converted." Top/Bottom entered in the dialog are always in **IP well units regardless of
file units**.

**Decimal separator.** "IP assumes that all import files use the decimal point as the
separator for decimal numbers." No locale handling on import (`load_ascii_data.htm`).

**Curve typing.** Curve names are verified against `CparmDef.xml` and `CPARMDEF_USER.PAR`;
matches get their Curve Type set automatically at scan time (`load_ascii_data.htm`).

**Well name parsing.** Read from a chosen line/character to end of line, **unless a colon is
encountered first — a colon terminates the well-name read**. The well name is ignored on
load if the well already exists in IP with a name (`load_ascii_data.htm`).

**Fixed-format spec.** Column widths comma-separated, with repeat brackets:
`8,8,8,8,10,12,12,12` ≡ `4(8),10,3(12)`; a trailing repeat may be omitted
(`8,10,12,8,10,12` ≡ `8,10,12`; `7(10)` ≡ `10`) (`load_ascii_data.htm`).

**Format persistence.** *Full* format saves file format + the middle grid (renames, units,
load flags; a blank curve name means "do not load") and is reusable **only** on files with
the exact same number of input curves in the same order. *Generic* saves only the file
format and the header line numbers, so it generalises across differing curve counts
(`load_ascii_data.htm`).

### 2.2 LAS/LBS Load (`las_lbs_load.htm`)

**Version support.** "wrapped or unwrapped LAS data from version **1.2 and 2.0** LAS files
as well as LBS files. LBS are LAS files with the data encoded in binary format."
(`las_lbs_load.htm`, `dataloading.htm`). **LAS 3.0 is not read by this loader** — that is
the separate LAS3 loader (§2.3).

**Concatenated LAS.** Handled here, not by the LAS3 loader: the *Embedded LAS Sequences*
pane lists each embedded file; each is inspected and edited before loading in turn
(`las_lbs_load.htm`).

**Depth.** The primary depth curve is defined by *Interval to Load (MD)* and *IP Well Step*,
"picked from the LAS file header data section". The first curve (Depth) need not be
selected — all other data is aligned to that primary depth curve. Interval to Load values
are always in **IP Well Units regardless of file units**; feet-in-file into a metric well is
explicitly supported (`las_lbs_load.htm`).

**Depth reference.** For files with no depth curve, set *Reference Depth Curve* to `TVDSS`
or `TVD`; "curves will be loaded in TVD or TVDSS mode" (`las_lbs_load.htm`).

**Time wells.** Depth units may be seconds or milliseconds; "The primary input curve should
be called `TIME`" (`las_lbs_load.htm`).

**Units.** "The Units name for a particular curve is picked up from the LAS file", manually
over-typeable (`las_lbs_load.htm`).

**Curve naming.** Manual over-type, or a *Curve Alias Defaults* file (Tools → Defaults →
Edit Curve Alias Defaults) mapping vendor mnemonics to a corporate set. Prefix/Extension may
be appended to all curve names — **"curve names with more than 10 characters will be
truncated to 10 characters."** (`las_lbs_load.htm`).

**Curve type default.** Looked up in `CparmDef.xml` at scan time (`las_lbs_load.htm`).

**Round-trip of IP-written LAS.** If the file was written by IP with *Use Set Names*, IP
recreates the curve Sets on reload. If written by a pre-v3.3 IP with *Write well parameters*,
those file-borne display properties take precedence over `CparmDef.xml` /
`CPARMDEF_USER.PAR` when *Load LAS Parameters* is on (`las_lbs_load.htm`).

**Fill Data Gaps.** Extrapolates across gaps; *Max Gap width* is expressed in **database
sample increments**, not depth units (`las_lbs_load.htm`).

**Creating a depth curve with no file.** Cancel the file prompt and enter interval, step and
units to build a primary reference depth curve from nothing (`las_lbs_load.htm`).

**Drop targets.** Dropped files are processed only on the IP desktop or database browser;
drops inside a child window are ignored. Multiple LAS files dropped at once divert to LAS
Batch Load (`las_lbs_load.htm`).

### 2.3 LAS Load — LAS3 (`load_las_data.htm`)

Loads LAS3, which carries core plug analysis, DST results and formation tops alongside log
curves. **"this loader can not handle concatenated LAS files. Use the LAS/LBS Load for
this."** (`load_las_data.htm`).

Import options, **all four cleared by default** (`[img-read: _dlclip00032.png]`;
`_dlclip00031.png` shows the same panel over the Curves-to-Import grid):

- *Overwrite Existing Curves* — if cleared, IP warns on identical curve name (rendered
  greyed/disabled in the screenshot).
- *Use "Load Into Set" option* — when set, ignores the LAS3 file's Set names and writes
  everything into the chosen IP Set; when cleared, the file's Sets are created in IP.
- *Fill Gaps* — "extrapolate across any small (**up to 5 depth increments**) data gaps".
  The manual explicitly warns: **leave this cleared for core analysis data** (core porosity,
  grain density, permeability), and do continuous logs and discrete core in two separate
  import passes (`load_las_data.htm`).
- *Link Zones* — imports LAS3 Tops and links them into a continuous IP Zone Set, taking each
  formation's base as the next deepest top.

Per-curve properties editable pre-load: Load into Set, Curve Name, Curve Units, Curve Type
(`[img-read: _dlclip00031.png]`).

### 2.4 LAS Batch Load (`las_batch_load.htm`)

Reads from each LAS header: Well Name, API Number, UWI, curve count, depth interval, depth
units, well step. Displayed identifier follows the user's Primary Well Identifier — **Well
Name by default** (`las_batch_load.htm`).

**Well matching**, three modes: *Create New Well for each file* (unique DB numbers, names may
collide); *Load into wells with matching…* (green cell = matched, white "No Match" =
unmatched — **unmatched wells fail the load with an error unless *Auto Create Well* is on**);
*Select Wells to load into* (manual drop-downs; identifiers become non-editable).
*API as IP Well Name* / *UWI as IP Well Name* push the real well name into the Well Comment
(`las_batch_load.htm`).

**Units — the override.** "*Use IP defined units* will cause the loader to **ignore the units
in the LAS file**, and use IP's database of curve units." (`las_batch_load.htm`). This is a
blanket unit override with no per-curve confirmation.

**Depth.** Top/Bot default to the LAS header range; *Auto Extend Depth* extends the well
automatically, otherwise the user must confirm in the Well Depths Editor before the file
loads. Clicking the Step column header propagates the top row's Step to every file
(`las_batch_load.htm`).

**Curve naming.** Mask file, else — **"If no Mask file is selected but you have a
`CurveAlias.txt` file available … the loaded curves will be named according to the curve name
mapping in the Curve Alias file. NOTE: this is automatically applied, you do not have to
manually select the Curve Alias file."** (`las_batch_load.htm`). Silent, implicit renaming.

**Date/Time format** is user-specified via a Date/Time Formatter; defaults observed:
Display Year/Month/Day all on, format Day/Month/Year, Month Format 2 Digits, 4-digit year on,
date separator `/`, language English (United Kingdom); Minute and Second on, **Milli-Second
off**; time separator `:`; Display Date Before Time on. Worked example renders
`22/06/2017 12:10:48` (`[img-read: _dlclip00194.png]`).

### 2.5 ASCII Batch Load (`batch_ascii_loader.htm`)

Format comes from a format file authored in the single-file ASCII loader. A file is loadable
only if it has a Well Name, **more than 2 curves**, and a column defining the Depth/Time/
DateTime curve; grey Load cells cannot be ticked (`batch_ascii_loader.htm`).

**Units — the trap.** "The units will be read from the index (Depth) column of the load
file… **On load if the Units have been set different to what is read from the file then a
unit conversion will be done on the input index (depth) curve.** Hence it is not a good idea
to change the units on the screen unless they have not been set in the file."
(`batch_ascii_loader.htm`). Editing a display field silently rescales depth.

**Step.** Read by inspecting the **first two data lines only**, and used only when creating
a new well (`batch_ascii_loader.htm`).

**Fill Data Gaps — the only explicit interpolation definition in the page set.** "any gaps in
the data less than the 'Max Gap width' will be extrapolated over using a **linear
interpolation** between the good data. The gap width is the number of load set steps. I.e. is
the gap width is 5 and the set step is 0.5ft then any gaps less than 2.5ft will be filled."
(`batch_ascii_loader.htm`). Default Max Gap width = **5**, checkbox cleared
(`[img-read: _dlclip0149.png]`).

### 2.6 LIS Load (`load_lis_data.htm`)

**Curve names.** "LIS format limits curve names to four characters." Longer IP names are
carried by appending the **Service ID**: "All characters, from the 4th character onwards, are
put into the service ID" (`load_lis_data.htm`).

**Frames / arrays.** *Rep* = repeat count per frame (array packing). *Num Samp* = samples per
frame. Array dimension is derived as Size-Byte ÷ storage size per value; the manual's worked
example is AVCL with Size Byte 20 and Rep Code 68 (4 bytes) ⇒ a 5-element array
(`load_lis_data.htm`).

**Scan artifact.** `LISscan.log` is written automatically to the project directory holding
the last scan result. Listings are *Short* (curve names + intervals) or *Long* (full LIS
structure) (`load_lis_data.htm`). A short listing shows curve/service-ID/file-number triples
plus a frame count and direction, e.g. `1037 data frames: from 4282.00 to 4800.00 FT,
log direction: down` (`[img-read: _dlclip00034.png]`) — **LIS records log direction, so
increasing/decreasing index must not be assumed.**

**Depth/units.** Feet-in-file into a metric well supported; Interval to Load is in well depth
units regardless of file units. Double-clicking a file-sequence row offers that sequence's
actual depth range as the interval (`load_lis_data.htm`).

**Sets.** *Auto. New Sets* creates one set per distinct step interval, named from the *Prefix*
box (`LIS_*`) plus a numeric to force uniqueness (`load_lis_data.htm`).

### 2.7 DLIS Load (`dlis_loader.htm`)

**Encrypted channels.** "If curves have been encrypted by the logging company then they will
appear with an **x** in the Load column. They will **NOT** be loaded into the IP database
along with the other selected curves." (`dlis_loader.htm`). Silent partial load unless the
scan is inspected.

**Frames.** "The DLIS standard groups all curves with the same step interval into Frames."
*Auto. New Sets* creates one IP set per step interval, named Prefix (`DLIS_*`) + the DLIS
**Frame Name** (`dlis_loader.htm`).

**Step index table** as enumerated in the scan (`dlis_loader.htm`), in units of 0.1 in:

| Index | Step | Raw |
|---|---|---|
| 1 | 6 in | 60 × 0.1" |
| 2 | 1 in | 10 × 0.1" |
| 3 | 0.5 in | 5 × 0.1" |
| 4 | 3 in | 30 × 0.1" |
| 5 | 12 in | 120 × 0.1" |
| 6 | 18 in | 180 × 0.1" |
| 7 | 0.1 in | 1 × 0.1" |
| 8 | 0.2 in | 2 × 0.1" |
| 9 | 2 in | 20 × 0.1" |

**Duplicate-name handling — three-way, and the default is a merge.** *Ignore* (matching
curves not loaded) / *Insert New Data into the Existing Curve* / *Create a New Curve with a
Numerical Suffix*. The File Options screenshot shows **"Insert new data into the existing
curve" selected as the default** (`dlis_loader.htm`; `[img-read: _dlclip00057.png]`).

**Parameter overwrite policy.** *Load Parameters* is checked by default with mode
**Underwrite** — blank parameters are filled, populated ones are left alone unless switched
to Overwrite (`dlis_loader.htm`; `[img-read: _dlclip00057.png]`).

**Other File-Options defaults** (`[img-read: _dlclip00057.png]`): *Extend the Well interval
if curves are outside the depth range* **checked**; *Extend the Set interval…* **checked**;
*Load Curves with a high sample rate as array data* **checked**; *Automatically use Frames to
create new Curve Sets* unchecked (Prefix `DLIS` greyed); *Automatically use Source to Create
new Curve Sets* unchecked; *Use Run number from the file* selected; Prefix/Extension empty.

**Multi-dimensional data.** For any file **not** produced by IP, multi-dimensional items are
"treated as containing multiple curves at the same depth **by default**", not as an IP
Z-array — *Split By Dimension* is the default. 2-D splits into one-dimensional arrays with a
numeric suffix and a group name set to the base: a bundled acoustic waveform `TFWV01` holding
8 waveforms × 672 samples loads as `TFWV01_1` … `TFWV01_8`, group `TFWV01`. **3-D data cannot
be loaded** — "Load as Array" and "Average the values to fit the Set" are disabled because
"the current and previous versions of the loader could not load 3-D data." For IP-generated
DLIS, multi-dimensional data is treated as Image Curve data and splitting is disabled
(`dlis_loader.htm`).

**Units / naming.** IP Name and Units columns editable; Curve Alias Defaults available.
*Use Underscore?* replaces spaces in the input curve name with `_` (`Gamma Ray` →
`Gamma_Ray`) (`dlis_loader.htm`).

**Header→curve-header mapping.** "Curve Attributes in the DLIS Channel Attributes or Axis
Attributes will be loaded into the IP Curve Headers depending on the mappings set up in the
**DLIS Curve Attribute Mappings** module." (`dlis_loader.htm`).

**Filter modes.** *No Mask* / *Selected* (combined result of applied masks) / *IP Defaults*
(auto-select any curve name IP already knows a default curve type for) / *Load Mask*. A
separate, **non-user-editable Image Tool Mask** is built into the installation and can be
auto-applied for image logs (`dlis_loader.htm`).

**Cross-run propagation.** *Apply Load Selections to Checked Runs* replicates the curve
selection across runs — "Curve Name must match in all selected runs" (`dlis_loader.htm`).

### 2.8 DBASE 4 Loader (`dbase_4_loader.htm`)

Null: "must be set to whatever Null value is defined in the database file. The default in IP
is **-999**" (`dbase_4_loader.htm`; panel shows `-999`,
`[img-read: _dlclip00042.png]`). Depth Curve Name in database defaults to **`Depth`** and
must be over-typed if the file differs. Units default **Feet**; metric file into imperial
well supported; time wells in seconds or milliseconds with primary input curve `TIME`.
IP Load Set defaults to `Default`, *Use Set Name* unchecked
(`[img-read: _dlclip00042.png]`).

### 2.9 Interval / Spreadsheet Loader (`intervalloader.htm`)

Input format: Top Depth and/or Bottom Depth then values, space/tab/comma separated.
**"The Interval / Spreadsheet Loader only reads lines with numeric data entries. Curve header
text, curve names and curve units are not recognized or loaded. A minimum of three data
columns must exist"** (`intervalloader.htm`). For point data (core plugs, RFT pressures) set
Bottom = Top. Up to **200 curves** loadable/creatable at once.

**Reference-curve coupling.** Auto Extend Well works for curves in irregular sets when the
Reference Depth Curve is left Default, and for curves in regular sets when it is set to that
set's depth curve. Changing the default load set changes the reference curve with it —
regular set ⇒ that set's depth curve; irregular ⇒ the default depth curve
(`intervalloader.htm`).

**Append vs replace.** *Delete Curves before write* clears the existing curve first;
**cleared is the default**, which appends/concatenates onto existing curve data
(`intervalloader.htm`).

Paste must use the dialog's Paste button — the right-click context Paste inserts a single
cell only. Copy/paste works on continuous ranges only (`intervalloader.htm`).

### 2.10 Load Well Attributes (`loadwellattributes.htm`)

Delimiters: space, tab, comma, Other, or Fixed with the same width syntax as ASCII Load.
**A Well Name attribute MUST be assigned to a column.** *Use UWI as IP Well Identifier* loads
UWI into the Well Name field instead of Well Name. *Overwrite existing IP Well header
Information* — when cleared, **only currently-blank IP attributes are updated**
(`loadwellattributes.htm`). Format persists to a `.whf` file.

### 2.11 Import Wells from ZIP (`import-wells-from-zip-file.htm`, `datasaving.htm`)

Only reads zips produced by *Export Wells to Zip File*; anything else reports "Archive is Not
Valid". **Original database well numbers are deliberately not preserved** — wells are
numbered 1,2,3… inside the zip and the importer assigns next-available numbers on import
(`datasaving.htm`, `import-wells-from-zip-file.htm`).

### 2.12 Import OH (`import_oh.htm`)

Thin wrapper: routes to an existing IP set, or to the standard LAS/LBS or ASCII Load modules
with the Load Set pre-populated to the active PL analysis set. Imported open-hole curves must
still be selected in PL Set-up to appear on the Autoplot (`import_oh.htm`).

### 2.13 Curve mask files (shared by LAS/LBS, LAS Batch, DLIS, and export)

Plain text, extension **`.mask`**, supplied examples in the user's working folder. Two
functions: **filter** the curve selection and **rename** on import or export
(`las_lbs_load.htm`, `las_batch_load.htm`, `dlis_loader.htm`).

Separators between external and IP name: **comma, space, tab or semicolon**
(`las_batch_load.htm`). DLIS states the file is space-delimited (`dlis_loader.htm`) — see
§6.4. Comment lines begin with `$` (`[img-read: _dlclip00204.png]`,
`[img-read: _dsaclip0005.png]`).

Import example (`[img-read: _dlclip00204.png]`):

```
$ This is a mask file
$ It is used to filter and re-name curves on import / export.
RHOB
TNPH
SGR  GR
CALI Caliper
DTLN DTComp
```

**Regular-expression syntax is supported and optional.** `DEVEI  DEV` renames one curve;
`(DEV|DEVEI)(\.H)?        DEV` matches `DEV`, `DEVEI`, `DEV.H`, `DEVEI.H` and renames all to
`DEV` (`las_lbs_load.htm`, `las_batch_load.htm`).

Gotcha the manual calls out: when saving a custom mask, beware a hidden `.txt` appended after
`.mask` (`las_lbs_load.htm`).

---

## 3. Defaults & constraints

### 3.1 Null values — the central table

| Path | Null | Settable? | Source |
|---|---|---|---|
| ASCII **load** | `-999.00` (prose) / `-999` (panel) | yes | `load_ascii_data.htm`; `[img-read: _dlclip0005.png]` |
| ASCII load, adjacent delimiters | implicit null injected | no | `load_ascii_data.htm` |
| DBASE4 **load** | `-999` | yes (must match file) | `dbase_4_loader.htm`; `[img-read: _dlclip00042.png]` |
| ASCII **export** | `-999` | yes | `datasaving.htm`; `[img-read: _dsaclip0030.png]` |
| LAS **export** | `-999` | yes | `datasaving.htm`; `[img-read: _dsaclip0038.png]` |
| **LIS export** | **`-999.25` hard-coded** | **no** | `datasaving.htm`; corroborated — LIS Write File Options has **no null field at all** `[img-read: _dsaclip0043.png]` |
| **DLIS export** | **`-999.25` hard-coded** | **no** | `datasaving.htm`; corroborated — DLIS Write File Options has **no null field at all** `[img-read: _dsaclip0050.png]` |
| DBASE4 **export** | `-999` | yes | `datasaving.htm`; `[img-read: _dsaclip0026.png]` (dialog referenced) |
| Petrel Tops export | `-999` in Time / Dip Angle / Dip Azimuth fields | not stated | `[img-read: _dsaclip0055.png]` |

**The headline:** IP's own ASCII and LAS exports default to **-999**, while LIS and DLIS are
**hard-coded -999.25**, and the ASCII *loader* defaults to **-999.00**. A LAS written by IP at
defaults and re-read by a consumer assuming the LAS-standard -999.25 will treat `-999` as
real data.

### 3.2 Export precision & format defaults

| Setting | Default | Source |
|---|---|---|
| Dec. places (ASCII) | **4** | `[img-read: _dsaclip0030.png]` |
| Dec. places (LAS) | **4** | `[img-read: _dsaclip0038.png]` |
| Dec. places limit | "no limit to what can be entered but only reasonable values will be displayed" | `datasaving.htm` |
| LAS Version | **2.0** (alternative 3.0; **1.2 not offered on export**) | `datasaving.htm`; `[img-read: _dsaclip0038.png]` |
| LAS Header Options | **Use Comments** (alt: Use Descriptions) | `[img-read: _dsaclip0038.png]` |
| Write LAS wrapped data | cleared (⇒ one line per depth increment) | `[img-read: _dsaclip0038.png]` |
| Write well parameters | cleared | `[img-read: _dsaclip0038.png]` |
| All wells in one file | cleared (ASCII/LAS/LIS/DLIS) | images above |
| Output each curve set using set depths | cleared for ASCII/LAS/LIS; **checked for DLIS** | `[img-read: _dsaclip0030/0038/0043/0050.png]`, `datasaving.htm` |
| Save in CSV Format | cleared; selecting it forces extension to `csv` | `datasaving.htm`; `[img-read: _dsaclip0030.png]` |
| LIS *Use service ID for long curve names* | **checked** | `[img-read: _dsaclip0043.png]` |
| DLIS *Export grouped curves as 3D curves* | **checked** | `[img-read: _dsaclip0050.png]` |
| File extension | `asc` / `las` / `lis` / `dlis` | `datasaving.htm` |
| Base file name suffix | `_W**` where `**` = database well number | `datasaving.htm` |

### 3.3 Structural limits

| Limit | Value | Source |
|---|---|---|
| Conventional curves per well | **20,000** (array curves count as one) | `dataloading.htm`, `input___output.htm` |
| IP database wells | **9,999** | `read_from_logic_db.htm` |
| Interval/Spreadsheet Loader curves per pass | 200 | `intervalloader.htm` |
| Capillary Pressure Data Loader plugs | 100 | `dataloading.htm` |
| LAS/LBS curve name after prefix/extension | truncated to **10** chars | `las_lbs_load.htm` |
| DBASE4 column header (curve name) | **10** chars; **curve Set names ignored entirely** | `datasaving.htm` |
| LIS curve name | **4** chars (+ Service ID overflow) | `load_lis_data.htm`, `datasaving.htm` |
| IP short curve-Set name (Petrel link) | **8** chars, longer truncated | `petrel_interface.htm` |
| Geolog ASCII parameters per tab | 15 | `geolog_ascii_loader.htm` |
| Illegal filename chars, replaced by `_` | `\ / : ; , " \| > < *` | `datasaving.htm` |
| WITSML irregular-set depth tolerance | **0.01, hard-coded** | `witsml_import.htm` |
| WITSML set-extension chunk | ~50 ft | `witsml_import.htm` |
| Fill-gap default max width | 5 (increments/steps) | `[img-read: _dlclip0005.png]`, `[img-read: _dlclip0149.png]` |
| LAS3 Fill Gaps span | up to 5 depth increments | `load_las_data.htm` |
| Geolog SSH password | must not contain `'` or `"` | `pc_configuration.htm` |
| PowerLog/Loglan app name | no spaces, no illegal Windows dir chars, **must not end with a number** | `powerlogloglanconverter.htm` |
| User App labels (post-convert) | Input/Output Curves < 21 chars; Input Parameters / Text Parameters / Logic Flags ≤ 2 words | `powerlogloglanconverter.htm` |

### 3.4 Resampling / step regularization — does IP resample?

**Yes, in several places, and mostly silently.**

| Path | Behaviour | Source |
|---|---|---|
| ASCII Load | "loaded to the closest sample increment in the well" | `load_ascii_data.htm` |
| Geolog DB import | Single DEPTH curve chosen from the **smallest-step** set; everything referenced to it; discrete core data lands "to the nearest IP well depth step, rather than at their true depth" | `read_write_to_geolog_db.htm` |
| Kingdom import | "If depth steps are mismatched, then the Kingdom curves are **resampled** to fit the existing IP step" | `kingdom_importer.htm` |
| WITSML → regular set | "data will be slotted into the closest depth level… for closely spaced source data, this can potentially lead to **different values over-writing each other**" | `witsml_import.htm` |
| WITSML → irregular set | matched into an existing level within **0.01**, else a new level is added | `witsml_import.htm` |
| XStream Connect | "New curve values are be saved to the closest depth available" | `witsmllink.htm` |
| Petrel export → IP | *Interpolate Values* **checked by default** interpolates between points; unchecked snaps to closest IP depth level | `petrel_interface.htm` |
| LOGIC import | curves faster than the chosen DB step → array curve; slower → sparse data with gaps | `read_from_logic_db.htm` |
| Geolog ASCII | differing sample frequency within a Geolog curve set → converted to **array data** in IP | `geolog_ascii_loader.htm` |

**The escape hatch, stated by the vendor:** irregular-step sets preserve original depths. The
Geolog page recommends routing core plug data via LAS/ASCII into **array curves** rather than
through the DB link, precisely to keep true plug depths
(`read_write_to_geolog_db.htm`, `load_ascii_data.htm`, `intervalloader.htm`).

### 3.5 TVD / MD conventions

- Export *Ref Curve* defaults to the **`DEPTH`** curve for ASCII/LAS/LIS/DLIS; changeable to
  TVD or TVDSS if those curves exist in the well (`datasaving.htm`).
- Loaders accept a TVD/TVDSS reference in place of MD via *Reference Depth Curve*
  (`las_lbs_load.htm`, `load_ascii_data.htm`).
- Interval to Load is labelled **MD** in the LAS/LBS loader (`las_lbs_load.htm`).
- Zone-tops export can include TVDSS, and *Force Negative TVD Values* forces all depths in
  the TVD curve to be written negative below MSL; **unchecked, the TVD is written exactly as
  stored in IP** (`datasaving.htm`).
- Petrel tops require UTM X/Y plus Z = true vertical subsea; without UTMs Petrel cannot load
  the file. EDIST/NDIST from the TVD module feed the UTM construction
  (`datasaving.htm`).

---

## 4. Export behaviour

### 4.1 Shared model (ASCII / LAS / LIS / DLIS)

All four writers share the *Select Wells and Depth Intervals* grid, an Available/Selected
curve pair, and the same file-naming furniture (`datasaving.htm`):

- Top/Bot default to the well top/bottom; **Step defaults to the current well step** and is
  editable — changing it resamples the output.
- Output curve order = order in the Selected Curves list.
- *Use Set Names* prefixes the output curve name with the Set name.
- **Curve aliasing on export:** "If the Curve Aliasing module is turned on then if a curve
  name cannot be found then **a curve of the same Curve Type will be selected instead**."
  A substitution by type, not by name.
- Renaming on export, three routes: over-type Output Name; a `.mask` file of name pairs;
  or auto prefix/suffix. New mask files get four `$` header lines added automatically
  (`[img-read: _dsaclip0005.png]`):

```
$       This file sets up automatic renaming of well curve names for output.
$
$       Output curve names must be entered in pairs, well curve name, output name.
$       Curve names should be separated by a space, tab, comma or semicolon character.
Phie, Porosity
Vcl, Clayvol
Sw, WatSat
```

- **Final-flag trap:** *Select Final Curves All Wells* is a global name-based selection —
  "if any of the other wells has the same curve in the same set, it too will be output, even
  if it is not marked as Final in that well." The fix is to also tick *Only output final
  curves* (`datasaving.htm`).

### 4.2 What survives a round trip

| Metadata | LAS | LIS | DLIS | ASCII | Source |
|---|---|---|---|---|---|
| Curve Set names | only via *Use Set Names* (prefix) — and IP re-creates Sets on reload of its own file | **cannot be output** | via Frames + *Auto. New Sets* on reload (IP v3.5+) | via *Use Set Names* prefix | `datasaving.htm`, `las_lbs_load.htm` |
| Curve description / comment / type | as LAS comments (*Use Descriptions* or *Use Comments*) | — | — | — | `datasaving.htm` |
| Well parameters | *Write well parameters* (stays LAS 2.0-conformant) | — | *Write well parameters* covers General, Position, Default Parameters tabs + logging-run attributes | — | `datasaving.htm` |
| Array data | — | — | X and Y dimension → 2 dimensions of samples per depth | — | `datasaving.htm` |
| Numeric type | ASCII text | — | **all samples as `FSINGL`** (IEEE single-precision float) | ASCII text | `datasaving.htm` |

**DLIS precision note for SandiBumi:** IP writes *everything* as single-precision `FSINGL`.
Any float64 curve exported to DLIS from IP loses precision unconditionally
(`datasaving.htm`).

**LIS long names on export:** with *Use service ID for long curve names*, the Output Name box
shows a colon separating LIS 4-char name from Service ID. The LIS Write screenshot shows
`GrC :`, `RHOB :`, `NPHI : C`, `DT :` — i.e. `NPHIC` splits to name `NPHI` + service ID `C`
(`datasaving.htm`; `[img-read: _dsaclip0043.png]`).

**LIS multi-logical-file control:** *Keep LIS File Open* leaves the physical file open so
further logical files can be appended; the file closes on module exit. *All wells in one
file* and *Output each curve set using set depths* both write multiple logical files into one
physical file (`datasaving.htm`).

**DLIS grouped curves** expand in the Selected list — a grouped `WAVES:TFWV01` selects as
`TFWV01_1` … `TFWV01_8` (`[img-read: _dsaclip0050.png]`), the mirror of the DLIS load-side
split (§2.7).

### 4.3 Zone-tops export formats

**IP format** — `$` comments, space-delimited, `TOP BOTTOM ZONE_NAME`
(`[img-read: _dsaclip0054.png]`):

```
$IP Well Tops: 04/09/2003
$ TOP    BOTTOM   ZONE_NAME
$well: XYZ 4, Top Set: Final Tops
10723.5 10860.5 C Sand
```

**IP format with TVDSS** — switches to **comma-delimited**, five fields
(`[img-read: _dsaclip0056.png]`):

```
$well: 16/0001 - TDT, Top Set: TVDSS Tops
7614, 7772, Zone A, 7515.3984375, 7672.3950195313
```

**Petrel format** — `#` comment character (not `$`), `VERSION 1`, typed
`BEGIN HEADER`/`END HEADER` block, then whitespace-delimited rows
(`[img-read: _dsaclip0055.png]`):

```
#IP Well Tops in Petrel format: 04/09/2003
VERSION 1
BEGIN HEADER
REAL X / REAL Y / REAL Depth / REAL Time / STRING Type / STRING Horizon Name /
STRING Well Name / STRING Symbol / REAL Measured Depth / STRING Pick Name /
STRING Interpreter / REAL Dip Angle / REAL Dip Azimuth
END HEADER
7685.63281 87331.5 -7316.20557 -999 HORIZON "C Sand" "XYZ 4" Unknown 10723.5 "" "" -999 -999
```

Note the Z is negative (TVD subsea) and unused REAL fields carry **-999**. The *Legacy*
Petrel option omits the header block (`datasaving.htm`).

### 4.4 DBASE4 export

Merge or Overwrite an existing database. *Overwrite existing curves* "does not delete the
existing curve but simply changes those values that overlap with the defined load interval."
**Curve Set names are ignored** because DBASE4 stores curve names as 10-char column headers —
"you must be careful not to select the same curve name from different Sets"
(`datasaving.htm`).

### 4.5 Kingdom export

Six steps. Well matching hierarchy (identical to the importer): **IP UWI → Kingdom UWI;
IP Well Name → Kingdom Borehole Name; IP Well Name → Kingdom Well Name**. Unmatched default
to *create new well*. **"IP Wells MUST have a UWI defined in the Well Header for the export to
be successful."** Kingdom has no curve sets, so the default is to prefix the curve name with
`SetName:` — `VCLGR` in set `VCL` becomes `VCL:VCLGR`, and the Kingdom Importer reverses this
on re-import, restoring the original set. Depth curves excluded by default; **array curves are
unsupported and not offered** (`datasaving.htm`).

**Stale-header trap:** "the Well Header information exported from IP is taken from the **saved
well data file**. If the user edits the well header data and then exports without first saving
the well, the values exported to Kingdom will not match those edited values."
(`datasaving.htm`).

**CRS:** Kingdom's Grid System and Geodetic Datum must already exist in the IP CRS list, else
lat/long and X/Y transfer **without transformation** and must be manually corrected. Kingdom
does not allow a datum independent of the grid system, so IP derives the datum from the grid
system (`datasaving.htm`, `kingdom_importer.htm`).

---

## 5. External-database bridges (mapping rules)

### 5.1 Common generic interface

GEOLOG, OpenWorks, PETCOM, ODM/IC, Openspirit, Petrolog and Petrel-2014+ all share one tabbed
interface: **Project / Well / Well Matching / Log Data / Tops Data / Progress / Session**
(`read_write_to_geolog_db.htm`, `read_from_openworks_db.htm`, `read_write_to_petcom_db.htm`,
`read_write_from_odm_database.htm`, `read_write_via_openspirit.htm`, `readfrompetrolog.htm`,
`petrel_link_2014_and_later.htm`).

Shared rules across all of them:

- Direction is inferred from which side is selected in the Well List; selecting IP wells
  flips the button to Export.
- Well-name **filter syntax** (a restricted regex): `.` any single character; `*` repeat
  previous pattern zero or more times; `[0-9]` any single digit; `{charlist}` any single
  character in a list. Worked example `22_2[23].*` lists wells beginning 22_22 and 22_23.
- *Fill data gaps* — gaps extrapolated where possible on import **and** export.
- *Overwrite existing curves*, *Overwrite Survey*, *Overwrite Data* checkboxes.
- **Curve Mappings** dialog: per-set/per-curve rename rules applied source→destination,
  saved by name and reusable.
- Sessions (the whole dialog state) save/load and **can be re-applied to a different database
  or database type**.
- Connection config persists to `IntPetro.config` in the IP directory on Apply
  (`pc_configuration.htm`) — but see §6.3.
- Modules must first be enabled under Tools → Options → External Database Connections.
- Windows NT-derived OS only; not Windows 9x.

### 5.2 GEOLOG DB link — the mapping rules (special tasking g)

**Curve version suffixes.** "GEOLOG6 curve names are displayed with a `_1`, `_2`, `_3` … after
their name. This is the version number of that curve in GEOLOG6. When loading into IP, **the
`_1` will be removed, while any `_2`, `_3` … will be left appended** to the curve name to allow
curves to be discriminated one from another." (`read_write_to_geolog_db.htm`).

**The single-DEPTH-curve rule — the most consequential mapping in this bridge.**
"IP will only allow **one DEPTH curve** to be stored for each well imported from GEOLOG. IP
interrogates the selected GEOLOG well to determine the selected Set / curve with the
**smallest well step increment** and uses this Depth curve as the IP DEPTH curve for the well.
All other curves that are selected for loading into IP are **referenced to this Depth curve**.
Discrete data, for example core plug data (porosity, grain density, permeability etc..) would
be loaded **to the nearest IP well depth step, rather than at their true depth**, if they were
imported to IP in this way." The vendor's own recommended alternative: export core plug data
to LAS or ASCII and load as **Array curves** to retain actual plug depths
(`read_write_to_geolog_db.htm`).

**Connection.** Three transports — HTTP (Apache + suEXEC), SSH (PuTTY/plink), and Windows
(local GEOLOG install). HTTP needs a **PNS file** (Paradigm Name Service) path, which holds
the addresses of all GEOLOG databases on the system; Read URL syntax is
*Protocol – Server Domain Name – User Name – CGI script*. SSH needs the GEOLOG6 `bin`
directory, server DNS name, account, password, and optional environment variables
(`PRDM_GEO_LICENSE_FILE`, `MINSITE`). **`MINSITE` controls the GEOLOG environment — GEOLOG6
sources `MINSITE/bin.geolog6_env.tcl`**, and is required where multiple GEOLOG installs mean
multiple PNS files. Multiple variables are `;`-separated. **The SSH password must not contain
a single quote or a double quotation mark.** SSH needs no UNIX-side install; HTTP needs the
`PGLWdblnk` package under `/opt` with `gllnk_import.cgi` / `gllnk_export.cgi` /
`gllnk.cgi`, the per-user `~/public_html` copy owned by the user with permissions
`-rwxr--r--` (group/world-writable files are refused by suEXEC). A GEOLOG6 licence is
required (`read_write_to_geolog_db.htm`, `pc_configuration.htm`, `connectionprotocols.htm`).
Vendor preference: **"The SSH transport method is generally preferable to the HTTP option, as
it requires less administration and configuration and is also more secure."**

### 5.3 Geolog ASCII Dump loader (special tasking g, file route)

**File recognition.** Files usually carry a `.dat` extension and begin
`*HEADER  GEOLOG LOG Geolog Dump File`. **"IP will check that the first line starts `*HEADER`
and if not found will assume this is not a Geolog ASCII file and will not read it."** Explicit
collision warning: **IP's own database files also use `.dat` but are binary and cannot be read
by this loader** (`geolog_ascii_loader.htm`).

**What maps and what does not** (`geolog_ascii_loader.htm`):

- Curve data → IP curve sets **of the same name**. Array data loads.
- **Differing sample frequency inside one Geolog curve set → converted to array data of that
  frequency in IP** (because in IP all curves in a set share one sample frequency).
- Parameters inside curve sets → IP **Geolog parameter sets**.
- **Geolog text curves are NOT loaded.**
- Geolog sets marked `TOP` → IP picks or tops sets; **only the latest version of the Geolog
  Tops curve is loaded**.
- Geolog **Constants** → IP well and log attributes, but **only if a name mapping exists**;
  they also always land in a Geolog Parameter set. **"Once a Well Header attribute has been
  populated, it will not be over-written by subsequent occurrences of a constant"** —
  first-occurrence-wins.
- Constants not belonging to a curve or Tops set → parameter set named **`WELL_HEADER`**.
- Constants in a curve set → a one-zone parameter set spanning the curve set's depths.
- Constants in a Tops set → appended after the curve-derived parameters, same value on every
  zone.
- In parameter-set column headers, **`_` in Geolog curve names is replaced by blanks**, and a
  trailing number is the Geolog version.
- An optional log attribute named **`GeologSet`** additionally records the source set name.

**Set-name truncation.** "The Geolog Set Name will be **truncated to the maximum allowed IP
short set name**. The full Geolog set name will be copied into the IP long set name (set
description)." A rename map in the options file avoids the truncation
(`geolog_ascii_loader.htm`).

**Version handling on load**, three modes: *Remove version from last version* (strips the
number from the newest only), *Leave version numbers on all versions*, *Load Last Version
only*. The dialog screenshot shows **"Remove version number from…" selected as the default**
(`geolog_ascii_loader.htm`; `[img-read: _dlclip0162.png]`).

**Well matching defaults** (`[img-read: _dlclip0162.png]`): *Load into wells with matching:
**Well name*** selected (not *Create new well for each file*); *Auto create well* checked;
*Auto extend depth* checked; *Auto select matching well names* checked; *Save and close after
**10** wells* selected; *Append to File List* unchecked. Matching key selectable as Well Name,
API or UWI (`geolog_ascii_loader.htm`).

**`GeologASCII_options.txt`** — the configuration file, edited via *Edit Options*. Transcribed
default contents (`[img-read: _dlclip0162.png]`):

```
$ Load Geolog top to IP Picks
Yes
$ Load Geolog TOPS to IP Tops/Parameters
Yes
$ Include Parameters with TOPS
Yes
$ Load certain non-tops sets into IP Tops/Parameters
No
$ Geolog Tops sets name start with :
TOPS
TOP
SURFACE
$ Geolog Top/Zone name curve starts with :
TOPS
FORMATION
ZONE
SURFACE
GEOPARAMS
PARA
$ Geolog to IP Set Name Mapppings
$ Format : 'Geolog Set Name' , 'IP Set Name'
Composite , Comp
$ Geolog Depth Names
DEPTH
TDEP
DEPT
MD
INDEX
TVD
```

**The `$ Geolog Depth Names` list — `DEPTH, TDEP, DEPT, MD, INDEX, TVD` — is the loader's
depth-curve recognition set and is documented nowhere in the page prose.** See §6.6. Note
`TVD` sits in the same list as MD-type names, with no stated disambiguation.

### 5.4 Petrolog

Folder-based DB; connect to the **top-most folder containing all project folders**.
Parameter Sets are imported automatically. **Duplicate curve names in one set get the Petrolog
log number appended** — "if two GRs exist in the same set then the second will be named
`GR-9` (where 9 is the petrolog log no)". A `PetrologSets.xml` file maps Petrolog **visual
separators** to IP curve sets: an entry `FNL_EDIT, Final` routes every curve following
`FNL_EDIT` in Petrolog into an IP set called `Final` (`readfrompetrolog.htm`).

### 5.5 Shell LOGIC

Migration tool only, and **version-locked: "This module only works with LOGIC version
S2001.2. Other versions are NOT supported."** (`read_from_logic_db.htm`).

Pre-requisites: create IP well attributes **`Comment1`, `Comment2`, `XLoc`, `YLoc`** to
receive LOGIC header fields; add non-standard curve names to `CPARMDEF_USER.PAR` so defaults
are picked up. Select the folder containing the `WELL001`, `WELL002` … folders.

Sample-frequency mapping: the chosen IP DB step governs — **faster LOGIC curves → array
curves; slower → sparse data with gaps**. IP DB well number matches the LOGIC number where no
conflict exists; ceiling 9,999 wells.

Capillary pressure lands in a set named **`CapP`**, with **five curves per LOGIC Pc curve**,
suffixing the LOGIC name: `pc` (array, pressures), `sw` (array, saturations), `perm`
(regular), `phi` (regular), `GD` (regular, grain densities). Image data loads as arrays. All
well and curve header data (module, creation/update dates, comments) copies over, and the
LOGIC `journal.dat` is copied into the IP well folder (`read_from_logic_db.htm`).

### 5.6 Openspirit — the unit-loss warning

**"At time of writing OpenSpirit has a limited ability to translate curve measurement units
residing in IP, to external databases. OpenSpirit recognizes **POSC unit acronyms only**. This
means that many of the standard curve units e.g. `g/cc`, `v/v`, `dec.`, `API`, `inches`, will
NOT be copied to the attached external database. The user may have to manually update/correct
the curve units in the external database."** (`read_write_via_openspirit.htm`).

Also: "IP currently only recognizes external database well locations (Latitude / Longitude)
where the parent project co-ordinates are defined in **UTM and WGS84 datum**." Runtime
versions 2.9–3.6 (needs OpenSpirit .NET components, .NET 1.1) or 4.0; only one runtime per
machine; `PGL_IP` runtime licences must come from OpenSpirit; the OpenSpirit User Server must
be running (`read_write_via_openspirit.htm`).

### 5.7 Petrel (2013 and older plug-in) — the unit case-sensitivity trap

**"Unlike IP, Petrel units are case-sensitive; therefore, there is the risk that Petrel may not
recognize the IP curve's unit of measurement. If this happens, the curve is imported into
Petrel with the **General template and with no units of measurement**."** A shipped
IP-unit → Petrel-unit conversion list mitigates this and is user-extensible via *Edit unit
mappings*. Stated flatly: **"IP units are not case sensitive, but Petrel units are case
sensitive."** (`petrel_interface.htm`).

Other rules (`petrel_interface.htm`):

- Mapping Templates predefine IP well→Petrel well, IP curve→Global well log, IP tops→Horizons
  (import) and Petrel well→IP well, Global well log→IP curve (export). **Each mapping must
  have *Use* checked or it is ignored.**
- Default mapping is *New entry*: reuse a same-named Petrel item, else create one.
- **Duplicate mappings:** IP allows same-named curves in different Sets, Petrel's Global well
  logs are generic — "The transfer of data will still go ahead if the user hasn't resolved the
  conflicts. However, **the last named IP well or curve will be the one that is transferred**."
  Silent last-write-wins.
- Overwrite unchecked on a name collision → a new Petrel item with an incremental number
  (`Gamma`, `Gamma 2`, `Gamma 3`).
- Export to IP: `<None>` → IP Default set; **`Petrel` is the default set name**; new set names
  limited to **8 characters** and truncated beyond; tops default to a set named
  **`PetrelTops`**.
- **`Step` of 0 (zero) creates an irregular-step Set** — the documented route for core data.
  Changing Step affects new sets only; existing sets keep their step.
- *Interpolate Values* **checked by default**; unchecked snaps to the closest IP depth level.
- Exporting to an existing IP curve overwrites it.
- Zones imported from IP land in Petrel's *Others* folder and **cannot be exported back**;
  only Petrel *Stratigraphy* zones export to IP, and Zones (not Horizons) are the correct
  selection.
- Locked wells in a shared IP database cannot be imported from or exported to.
- Well location/path require valid X/Y in Manage Well Header Info → Position, plus `NDIST`
  and `EDIST` curves for the path, **saved before transfer**.

### 5.8 Petrel Link 2014 and later

Architecture change: a separate bridge program, driven from IP rather than from Petrel. The
Petrel project **must contain a `Wells` folder** (and a `Well Tops` folder for tops) or the
link will not connect. Version-matched links (Petrel 2014 link for Petrel 2014, etc.). Local
or TCP/IP connection; **the TCP/IP pass code can expire**. Destination Well Settings on export
are *Overwrite* (no new well, existing data overwritten) or *Duplicate* (new well containing
the selected logs). **"When importing from Petrel to IP, all the curves imported will be
copied into the same IP curve set"**, chosen in *Default load Set*
(`petrel_link_2014_and_later.htm`).

### 5.9 Kingdom Importer

Seven steps. Same match hierarchy as the exporter (UWI → Borehole Name → Well Name).
**"Existing wells and sets will be extended as necessary… If depth steps are mismatched, then
the Kingdom curves are resampled to fit the existing IP step."** Curves whose names are
prefixed `SetName:` are recognised and routed to that set, creating sets as needed. Tops
import into a Picks Set named **`Kingdom`** by default, honouring Kingdom's Alias mode, Strat
column, and Author Priority settings. **Wells must be saved in IP afterwards**
(`kingdom_importer.htm`).

### 5.10 ODM / IC

Connection type **Access** or **SQL Server**; for Access an OLEDB connection string is built
automatically from the selected `.mdb` (`read_write_from_odm_database.htm`,
`pc_configuration.htm`).

### 5.11 WITSML Import and XStream Connect (special tasking i — capability only)

**No server URL, username, password, host name, port or pass code appearing in any screenshot
or prose has been transcribed into this document.** Both modules take vendor-supplied
credentials entered by the user; that is recorded as a capability and nothing more.

**WITSML Import** (`witsml_import.htm`):

- Supports WITSML **1.3.1.x and 1.4.1.x**; version selectable as 1.3.1, 1.4.1 or Automatic.
  On validation the server is queried for supported versions and **the most recent supported
  version is used**.
- *Automatically check for new data* is **disabled by default** ⇒ one-time import. Enabled, it
  polls, downloads in the background and extends wells and curve sets as needed.
- Configuration and all data mappings persist to **`WITSML Session.xml`**, and logging to
  **`WITSML Log.txt`**, both in the IP Database folder under the user's Username folder —
  therefore **per-user, per-database**; the session file is portable between user folders.
- Default mapping: current **Active Well**, curve set named after the tree's *Logs* item,
  curve named after the log curve.
- Set-name generation modes: **`Parent`** (default, = Log name, e.g. `8_5inSection`),
  `Parent and Units` (`8_5inSection_ohm_m`), `Parent and Name` (`8_5inSection_A28H`), `Name`
  (`A28H`). Names are sanitised by removing illegal characters and spaces.
- **Depth model:** "Data from a WITSML server is inherently irregular. Every data value is
  sent as a (depth, value) pair. There is no concept of a 'step'." Modules creating a set
  **always create an irregular set**.
  - Regular destination set → nearest-depth slotting, with the explicit warning that closely
    spaced source data **can overwrite each other**.
  - Irregular destination set → matched within a tolerance of **0.01**; otherwise a new depth
    level is inserted. **"Note that we are not using the global IP Irregular Set Depth
    Tolerance value which is set up in Tools > Options. The tolerance value of 0.01 is
    hard-coded in this module."**
- Sets extend in chunks of ~50 ft to avoid repeated one-sample extensions.
- *Overwrite* replaces destination data **only at the depth levels actually downloaded**;
  other levels are untouched.
- Loaded Minimum/Maximum track what has been fetched so only new data is pulled; *Forget the
  loaded intervals* forces a full re-download.
- Batch operations fire only when fresh data actually arrived.

**XStream Connect** (`witsmllink.htm`) — DK Energy technology, the older WITSML route:

- Existing data is **underwritten** (kept) or **overwritten** (replaced), configurable, with
  the default replacement behaviour set under Tools → Options → External Database Options →
  XStream Connect.
- The **Log name identifies the curve set in IP**; a missing set is created as a **new
  irregular set**; well depth extends automatically as data arrive.
- **Sanitisation rules** rename logs, curves and curve units before creation in IP — worked
  examples rename `BGRC` → `GR` and `SGRC` → `GR`.
- Survey data requires **East Distance, North Distance and TVD curves already defined** in
  Manage Well Header Info → Position, **all three within the same curve set**.
- Depth extension keeps the original step for regular sets, adds a new depth for irregular
  sets; "New curve values are be saved to the closest depth available."
- *Maximum Records* caps per-trigger volume; *Download Interval* sets frequency.
- Resetting reloads from the first available point; editing curves or sanitisation rules
  forces a reset; Default Curve Group and Curve Set are immutable once transfer has begun.
- A **Demo mode** simulates transfer with no server connection.

### 5.12 PowerLog / Loglan Converter (special tasking h)

**What it converts** (`powerlogloglanconverter.htm`): a user app written in **Geolog Loglan**
or **PowerLog** into **Visual Basic**, so IP can execute it.

- Source language: Loglan or PowerLog. **Target language: "Currently, only Visual Basic is
  implemented."**
- Loglan input = a parameters file **`.info`** plus a code file **`.lls`**; the two are
  auto-paired by matching name and directory. PowerLog input = **`.pup`** only — "a powerlog
  user app does not use a separate parameters file", and the parameters controls deactivate.
- Output = a directory holding a parameters file (`Parameters`) and **`UsersCode.vb`**.
- App name: no spaces, no characters illegal in a Windows directory name, and **must not end
  with a number**; illegal characters are silently replaced with underscores (with a
  notification dialog).

**Limits — this is a lossy, best-effort translator:**

- If the `.info` file is missing, it falls back to deriving parameters from the `.lls`.
  If no parameters file can be produced at all, processing stops.
- Uninterpretable **parameter** lines raise a warning with continue/abort.
- **Parameter names colliding with Visual Basic or IP keywords, or breaching IP's length
  restrictions, are renamed** (Variables Changed dialog).
- Uninterpretable **code** lines do not stop the conversion: an `ErrorLog` captures the
  exceptions, and **the generated app contains a three-line error stub in place of each
  untranslated line** — a notification, the original line, and the reason. The app will
  compile-fail or misbehave until these are hand-fixed.
- Post-conversion manual work is required: labels must be corrected to IP style —
  **< 21 characters for Input and Output Curves; no more than two words for Input Parameters,
  Input Text Parameters and Input Logic Flags** — then Compile.

**Practical read for a Loglan author:** the converter handles the parameter block and
straight-line arithmetic, and hands back stubs for anything it cannot parse. It is a porting
aid, not an emulator; no Loglan runtime semantics (module chaining, external C/Fortran calls)
are claimed anywhere on the page.

### 5.13 Connection transport summary

`PGLWdblnk` installs under `/opt` via `pkgadd`; if relocated, the full path must be edited
into the head of each CGI file. OpenWorks needs the `owlnk` binary plus `owenv`; SSH also
`owlnk.sh`; HTTP also `owlnk_import.cgi`, `owlnk_export.cgi`, `owlnk.cgi`. Documented UNIX
requirements for OpenWorks: R2003 Sun Solaris, Certification OS 2.8, Oracle 8.1.6, Forte
Developer 6 or later C/C++ libraries, and either an SSH daemon or Apache ≥ 1.3 with suEXEC.
`LD_LIBRARY_PATH` must include `/usr/lib:/usr/openwin/lib:$OWHOME/lib:$ORACLE_HOME/lib`
(`connectionprotocols.htm`).

---

## 6. Internal discrepancies

**6.1 ASCII Load default null: prose vs panel.** Prose says "The default value is **-999.00**"
(`load_ascii_data.htm`); the Input File Defaults screenshot shows the field containing
**`-999`** (`[img-read: _dlclip0005.png]`). Numerically identical, textually not. It matters
only if a consumer does a string comparison against the file's `NULL` line. Recorded, not
resolved.

**6.2 Export null default -999 vs LIS/DLIS hard-coded -999.25.** Within one page
(`datasaving.htm`), ASCII and LAS default to `-999` while LIS and DLIS are hard-coded
`-999.25`. Not an error in the manual — but it means **IP's four export paths do not agree
with each other**, and neither the ASCII nor the LAS default matches the LAS-standard
`-999.25`. Corroborated by the absence of a Null field in both the LIS and DLIS Write panels
(`[img-read: _dsaclip0043.png]`, `[img-read: _dsaclip0050.png]`).

**6.3 Connection config file name.** `read_write_to_geolog_db.htm` and
`read_from_openworks_db.htm` say settings are saved to **`IntPetro.exe.config`**;
`pc_configuration.htm` says **`IntPetro.config`**; `intro_file_formats.htm` lists "External
Database Configuration Details = **`IntPetro.config`**". Two of three say `IntPetro.config`.
Unresolved — see OPEN ITEMS.

**6.4 Mask-file delimiter.** `dlis_loader.htm` calls it "a **space-delimited** text file";
`las_batch_load.htm` says "**comma, space, tab and semicolon**" are all acceptable; the export
mask header written by IP itself says "space, tab, comma or semicolon"
(`[img-read: _dsaclip0005.png]`). The DLIS description appears to be the narrower/older text.

**6.5 "Extrapolate" vs "linear interpolation" for Fill Data Gaps.** Every loader page says
gaps are "extrapolated" (`las_lbs_load.htm`, `load_ascii_data.htm`, `dlis_loader.htm`,
`load_lis_data.htm`, and all DB-bridge pages). Only `batch_ascii_loader.htm` states the actual
mechanism: "extrapolated over using a **linear interpolation** between the good data". The
term "extrapolate" is being used loosely for what is interior interpolation.

**6.6 `GeologASCII_options.txt` — undocumented section.** The prose walks through every
section of the options file *except* `$ Geolog Depth Names`
(`geolog_ascii_loader.htm` lists: Load Geolog Top to IP Picks; Load Geolog TOPS to IP
Tops/Parameters; Include Parameters with TOPS; Load certain non-tops sets; Geolog Tops sets
name start with; Geolog Top/Zone name curve starts with; Geolog to IP Set Name Mappings).
The screenshot shows a seventh section, `$ Geolog Depth Names` = `DEPTH, TDEP, DEPT, MD,
INDEX, TVD` (`[img-read: _dlclip0162.png]`). A depth-recognition list that governs which
column becomes the index, documented only in a picture.

**6.7 DLIS "Automatically use Source to Create new Curve Sets".** Present in the File Options
screenshot (`[img-read: _dlclip00057.png]`) but the string appears **zero** times in
`dlis_loader.htm` prose, which documents only the Frames-based option. Undocumented control.

**6.8 Fill Gaps default state, LAS3 vs elsewhere.** `load_las_data.htm` describes Fill Gaps as
an option with a fixed 5-increment span; the panel shows it **cleared** by default
(`[img-read: _dlclip00032.png]`), consistent with the page's own advice to leave it off for
core data. But `las_lbs_load.htm` and `batch_ascii_loader.htm` expose a **user-settable**
Max Gap width defaulting to 5. So "5" is a hard limit in LAS3 and a soft default elsewhere.

**6.9 Stale version list in a 2025 manual.** The Petrel page still states the IP short set-name
limit as "8 characters for **IP 3.4, IP 3.5, IP 3.6 and IP4.0**" (`petrel_interface.htm`) —
byte-identical to the 2018 manual (§7), with no mention of the current version. Whether the
limit is still 8 in the 2025 build is not established by this text.

**6.10 Openspirit page title vs body.** The Read/Write via Openspirit page describes loading
"from a Openspirit **(Powerlog)** database" (`read_write_via_openspirit.htm`); the same
parenthetical "(Powerlog)" is copy-pasted into the GEOLOG, OpenWorks and PETCOM pages
(`read_write_to_geolog_db.htm`, `read_from_openworks_db.htm`, `read_write_to_petcom_db.htm`).
It is boilerplate, correct only for PETCOM. Cosmetic, but it signals these pages were cloned
and not fully re-edited — relevant when judging how much per-DB detail to trust.

---

## 7. IP2018 numeric diff

Method: located each assigned page's counterpart in `c18`, then grep-compared the load-bearing
numerics. `c18` and `c25` were read only.

**Pages absent from the 2018 manual (new in 2025) — 5 of 34:**
`witsml_import`, `import-wells-from-zip-file`, `kingdom_importer`, `readfrompetrolog`,
`connecting-to-external-databas`.

The remaining 29 pages exist in both. 2025 files are consistently ~15–25 % smaller in raw
bytes, which reflects markup/theme changes rather than content loss on the facts checked.

**Numerics compared — unchanged 2018 → 2025:**

| Fact | 2018 | 2025 |
|---|---|---|
| ASCII load default null | `-999.00` | `-999.00` |
| LIS/DLIS export hard-coded null | `-999.25` (2 occurrences) | `-999.25` (2 occurrences) |
| LAS/LBS versions read | 1.2 and 2.0 | 1.2 and 2.0 |
| DBASE4 column header limit | 10 characters | 10 characters |
| LAS/LBS name truncation | "truncated to 10" present | present |
| LAS3 Fill Gaps span | "up to 5 depth increments" | same |
| Geolog smallest-step DEPTH rule | present | present |
| Geolog curve version `_1` rule | present | present |
| Geolog ASCII save-and-close after 10 wells | present | present |
| Petrel IP set-name limit | "8 characters for IP 3.4, IP 3.5, IP 3.6 and IP4.0" | **byte-identical** |
| LOGIC database well ceiling | 9,999 | 9,999 |
| LAS Batch *Use IP defined units* | present | present |

**Changed:**

| Fact | 2018 | 2025 |
|---|---|---|
| Conventional curves per well | **not stated** on `dataloading` / `input___output` | **20,000** stated on both |

**Reading.** The data-I/O layer of IP is numerically static across seven years. Every null
convention, every truncation limit and every depth-mapping rule that matters to an ingest
implementation is identical in 2018 and 2025. The only new hard number is the 20,000-curve
ceiling, and the only new capability surface is WITSML/Kingdom/Petrolog/ZIP. This stability is
itself the useful finding: these conventions are safe to design against, and the -999/-999.25
inconsistency is long-standing rather than a recent regression.

---

## 8. SandiBumi / SegaraBumi notes

Framed as "what IP does, and what we must do to exceed it."

**8.1 Never inherit IP's null defaults.** IP ships three different nulls across its own
writers (`-999` ASCII/LAS, `-999.25` LIS/DLIS, `-999.00` on the ASCII reader). A SandiBumi
LAS writer should default to **-999.25** (the LAS standard) and must write the value into the
`NULL.` header line, never rely on a convention. On read, treat the file's declared null as
authoritative and additionally flag `-999`, `-999.0`, `-999.00`, `-999.25`, `-9999` as
*suspected* nulls requiring confirmation rather than silently coercing them.

**8.2 The adjacent-delimiter null is a real corruption vector.** IP injects a null between two
consecutive delimiters in CSV/TSV. That is the correct reading of a CSV empty field, but IP
does it *without recording that the value was absent rather than zero*. Our loader should
distinguish "field empty" from "field = null sentinel" in the ingest record so a QC pass can
tell them apart.

**8.3 Depth snapping must be explicit and logged.** Every IP path that snaps to the nearest
step does so silently: ASCII Load, Geolog DB, Kingdom, WITSML-into-regular-set, XStream. The
WITSML page even admits closely spaced samples "can potentially lead to different values
over-writing each other" — that is data loss with no diagnostic. **SegaraBumi should refuse to
resample by default**, and where resampling is requested, emit a per-curve record of samples
dropped/merged and the max depth shift applied. This is the single clearest place to exceed IP.

**8.4 Preserve true depths for discrete data.** IP's own answer to core plugs is "use an
irregular-step set or an array curve", and the Geolog page explicitly routes users away from
the DB link to LAS/ASCII arrays to keep plug depths. Our schema should make irregular/point
data a first-class case, not an escape hatch — core plug, DST, RFT and calcimetry records keep
their measured depth verbatim.

**8.5 Unit handling: the two failure modes to avoid.**
- *IP's blanket override* — LAS Batch's "Use IP defined units ignores the units in the LAS
  file". Powerful and dangerous. If we offer an override it must be per-curve, logged, and
  never the default.
- *Silent unit loss on bridge* — Openspirit drops any non-POSC unit (`g/cc`, `v/v`, `dec.`,
  `API`, `inches` are named as casualties); Petrel drops units entirely on a case mismatch and
  falls back to a "General" template. Our unit layer should canonicalise on ingest, keep the
  **original unit string verbatim** alongside the canonical form, and fail loudly on an
  unrecognised unit rather than dropping it.

**8.6 Curve-name collisions: pick one policy and make it visible.** IP has at least five
different behaviours — DLIS three-way (default: **merge into the existing curve**), Petrolog
`GR-9` log-number suffix, Petrel incremental `Gamma 2`, Geolog `_2`/`_3` version suffixes,
Kingdom `SetName:` prefix. The DLIS default is the dangerous one: silently merging two
different channels of the same name into one curve. SegaraBumi should default to **refuse and
report**, with explicit opt-in to suffix or merge.

**8.7 Alias application must never be implicit.** LAS Batch applies `CurveAlias.txt`
automatically when no mask is selected — "you do not have to manually select the Curve Alias
file". A rename that happens without being asked for is exactly how a wrong mnemonic reaches a
deliverable. Our aliasing should be explicit per run and recorded in provenance.

**8.8 Mask files are a good idea worth stealing.** A plain-text, `$`-commented,
regex-capable filter+rename file that works identically on import and export is a clean,
diffable, version-controllable artifact. Worth adopting wholesale — including the regex form
`(DEV|DEVEI)(\.H)?  DEV`.

**8.9 DLIS specifics for our reader.** Frames group curves by step. Encrypted channels must be
surfaced loudly, not skipped with an `x` in a grid nobody reads. 3-D data is a genuine gap in
IP — an opportunity. IP writes everything as `FSINGL`, so any float64 fidelity claim on a
round trip through IP is false; our writer should offer higher precision and say so.

**8.10 LIS 4-character names.** If we ever write LIS, the Service-ID overflow trick
(`NPHIC` → name `NPHI` + service ID `C`) is the vendor-compatible convention. More usefully:
any LIS we *read* may carry a truncated name whose full form lives in the Service ID.

**8.11 Geolog interop (direct user relevance).** Two routes exist and they are not equivalent.
The **DB link collapses everything onto a single smallest-step DEPTH curve** and snaps
discrete data. The **ASCII Dump route** preserves more (arrays, parameters, constants, tops)
and is configurable via `GeologASCII_options.txt`. For any SandiBumi Geolog bridge, prefer the
ASCII-dump semantics, honour the `$ Geolog Depth Names` list (`DEPTH, TDEP, DEPT, MD, INDEX,
TVD`) for index detection, keep the `_1`-stripping/`_2`-retaining version convention so curve
identity round-trips, and note that **Geolog text curves are simply dropped by IP** — a gap we
can close.

**8.12 Tops formats.** If we emit IP-compatible tops, note the format changes delimiter
(space → comma) when TVDSS is included, and that the Petrel variant uses `#` comments, a typed
header block, and `-999` in unused REAL fields. Petrel refuses tops without UTM X/Y.

**8.13 File-format inventory worth mirroring.** `intro_file_formats.htm` catalogues IP's whole
on-disk surface — `CparmDef.xml` / `CPARMDEF_USER.PAR` (curve defaults), `DefaultAlias.cax`
(alias grid), `UnitsConversion.par` (unit conversions), `SetDictionary.xml` (curve-set
dictionary), `.mask`, `.whf`, `.wst`. If we ever need to read an existing IP installation's
conventions rather than guess them, these are the files.

---

## 9. OPEN ITEMS

1. **Actual unit-alias table contents.** Every loader points at "Tools → Defaults → Edit Curve
   Alias Defaults", `DefaultAlias.cax` and `UnitsConversion.par`, but **no page in my 34
   reproduces a single alias or unit-conversion row**. The behaviour is documented; the table
   is not. Would need the Tools/Defaults pages (another agent's range) or the installed files.

2. **What happens on an unknown unit at load.** §5.6/§5.7 tell us what Openspirit and Petrel
   do on the *bridge*. For the file loaders (LAS/DLIS/LIS/ASCII), the manual says units are
   "picked up from the file" and are editable, but **never states what IP does with a unit
   string it does not recognise** — pass through verbatim, blank it, or attempt conversion.
   Unresolved.

3. **`IntPetro.config` vs `IntPetro.exe.config`** (§6.3). Two of three pages say
   `IntPetro.config`. Not resolvable from text alone.

4. **Whether the 8-character IP short set-name limit still holds** (§6.9). The 2025 manual's
   claim is copied verbatim from 2018 and enumerates only IP 3.4–4.0.

5. **LAS 1.2 on export.** The writer offers only 2.0 and 3.0
   (`[img-read: _dsaclip0038.png]`); 1.2 is readable but apparently not writable. The manual
   never says so explicitly. Inferred from the panel, flagged rather than asserted.

6. **LAS section-parsing quirks.** The manual describes *which* LAS sections/parameters IP
   consumes (well name, mud resistivity, run number) but gives **no statement about
   `~A`/`~C`/`~P`/`~O` section parsing rules, tolerance of malformed headers, or behaviour on
   a missing `NULL.` line.** Tasking (e) asked for "section parsing quirks stated" — the honest
   answer is that essentially none are stated.

7. **The LAS parameter table referenced but not shown.** `las_batch_load.htm` says loadable LAS
   header parameters are listed in a "Table in LAS/LBS File Input", and `load_lis_data.htm`
   says "The following table shows the LIS parameters currently recognized and loaded to IP by
   default" — **and then no table follows in the extracted text.** The LIS one has an image gap
   at that point in the page. Both parameter lists are missing from my extraction; the LIS one
   may be recoverable from an unindexed image.

8. **DLIS Curve Attribute Mappings module contents.** Named as the thing that governs
   DLIS-channel-attribute → IP-curve-header mapping (`dlis_loader.htm`), but the module itself
   is not in my page set. The actual mapping table is unknown.

9. **`$ Geolog Depth Names` semantics** (§6.6). The list includes both MD-type names
   (`DEPTH, TDEP, DEPT, MD, INDEX`) and `TVD`. Whether `TVD` is treated as an index candidate
   of equal standing, or how a file carrying both is disambiguated, is not stated anywhere.
   Material for a Geolog bridge.

10. **`_dsaclip0026.png` (DBASE Write dialog)** was referenced for the export null but read
    only as a prose-corroborating pointer; the DBASE4 **export** null default is taken from
    prose ("−999 is used in IP as the Null value number", `datasaving.htm`). The load-side
    `-999` is image-confirmed. Minor asymmetry in evidence strength.

11. **PowerLog `.pup` grammar coverage.** The converter's supported-construct set is nowhere
    enumerated; failures are only described reactively (error stubs in the output). Cannot
    state what fraction of a real Loglan app converts cleanly.

12. **Petrosys Exchange / OSDU and IP-IC Common Database** are named in the Import/Export hub
    (`input___output.htm`) as connectors covering OSDU, Openworks, Petrel, Petrosys Pro —
    **but neither has a page in my assigned 34.** If OSDU connectivity matters to SegaraBumi,
    those pages sit in another agent's range or were not extracted.
