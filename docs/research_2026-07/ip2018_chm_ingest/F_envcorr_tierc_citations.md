# F - Environmental Corrections, Tier-C Register and Citation Harvest

**Source** `C:\Users\ARUNIKA\AppData\Local\Temp\c18\_text\` - 278 decompiled pages of the Interactive
Petrophysics 2018 help manual (vendor PGL / Senergy / Lloyd's Register / Geoactive), plus two read-only
supplementary files from the install at `C:\Program Files\IP2018` (`EULA.rtf`, and the shipped correction
data files). Nothing in the install was modified.

**Role of this agent** cleanliness gate and breadth sweep. The five deep-dive agents own specific methods;
this document owns (F1) the environmental-correction package, (F2) the full-manual Tier-C register,
(F3) the deduplicated bibliography, and (F4) the page index.

**Binding rule applied throughout** every value and every citation below is one the manual actually prints,
quoted with its page. Where a method is recognisable but the manual gives no reference, the entry says
`not given in manual`. No citation has been completed, corrected or supplied from outside knowledge.
`[[EQUATION_IMAGE: ...]]` markers were treated as non-recoverable and no formula behind one was reconstructed.

Companion machine-readable outputs:

- `F_tierC_register.json` - 32 entries
- `F_citations.json` - 124 entries
- `F_manual_map.md` - all 278 pages, grouped and marked for recoverability

---

## F1 - Environmental Corrections

### F1.0 The headline answer: chartbook or proprietary?

**Both, and the manual tells you which is which per service company.** This is the single most important
finding of the F1 read, because it splits the package cleanly into an adoptable half and a forbidden half.

| Service company | What the manual says the corrections were built from | Adoptable route for SandiBumi? |
|---|---|---|
| **Schlumberger** (wireline) | "uses the algorithms **distributed by Schlumberger in their Green Book library**" | **NO.** Vendor-supplied algorithm library. But the manual separately names which *published* charts each tab "relates to" - use those. |
| **Anadrill** (SLB LWD) | "uses the **charts from** the 2000 Schlumberger Log Interpretation Charts book and the 2005 Schlumberger Log Interpretation Charts book" | **YES** - published chart books, chart numbers given. |
| **Baker Atlas** | "**Due to Senergy Ltd. receiving the Baker Atlas chart book as a series of algorithms** it has proved difficult to assign charts numbers from the actual chart book to the tabs" | **NO.** The most explicit admission in the manual. Vendor supplied fitted algorithms; even the chart numbering could not be reconstructed. |
| **Halliburton** | "uses the **charts from** the 1994 Halliburton Log Interpretation Charts book" | **YES** - chart numbers given per tab. |
| **Baker Hughes INTEQ** (LWD) | "uses the **charts from** the publication 2002 Log Interpretation Charts book" | **YES** - chart ranges given per tool size. |
| **Sperry-Sun** (LWD) | "uses the **charts from** the 1998 Sperry-Sun Log Interpretation Charts book" | **YES** - chart numbers given per tool size. |
| **Weatherford / Reeves** | "uses the **algorithms distributed by Weatherford** including the old Reeves/Precision tools plus the Compact tool series" | **NO** for the algorithms; the four named chart books are still valid acquisition targets. |
| **PathFinder** (LWD) | "uses the uses the **charts from** the 2007 PathFinder LWD Log Interpretation Charts" | **YES** - chart ranges given per tab. |

So five of eight companies are stated to be **published-chartbook-derived** and three
(Schlumberger wireline, Baker Atlas, Weatherford) are stated to be **vendor-algorithm-derived**.

Notably, the Techlog equivalent of this package ships compiled, and IP2018's does not - which is why this
page was worth a deep read. The corresponding IP2018 implementation is not merely documented, it is
**shipped as open data**, which answers the architecture question as well as the legal one (see F1.5).

### F1.1 Reference chart books, verbatim from the References section

From `environmentalcorrections` lines 807-841, quoted exactly as printed:

```
Schlumberger - Log Interpretation Charts (2000)
Halliburton - Log Interpretation Charts (1994)
Baker Atlas - Log Interpretation Charts (1984)
Baker Hughes Inteq 2002 Log Interpretation Charts (2002)
Sperry Sun - Log Interpretation Charts (1996)
Weatherford Log interpretation Charts Compact Tool Series (2007)
Weatherford Log interpretation Charts Standard Tool Series (2007)
Weatherford Log Interpretation Charts LWD Tool Series (2005)
PathFinder LWD Log Interpretation Charts (2007)
```

Two internal inconsistencies are recorded rather than silently resolved:

- Sperry-Sun is **1996** in the References list but **1998** in the module description on the same page.
- The Baker Atlas entry is **1984** in the References list, yet the module description says the chart book
  was never actually received as a chart book at all.

The manual also records the acquisition channel per vendor, which is real cost intelligence:
Schlumberger charts are a public URL; **Baker Atlas, Baker Hughes INTEQ, Halliburton and Sperry-Sun all
require a vendor user login**; Weatherford is a public URL. The manual closes with
"NOTE: Senergy Software Ltd cannot guarantee that links to external websites will remain active."

### F1.2 Corrections offered, per company and per tool

Chart numbers below are the manual's own, quoted as printed.

**Schlumberger (wireline)** - 10 tabs

| Tab | Tools and charts as printed |
|---|---|
| Gamma Ray | Gamma Ray correction charts GR1, GR2 and GR3 |
| Density | Formation Density Log (FDC) and Litho-Density Log (LDT), chart Por-15a |
| CNL | Compensated Neutron Log (CNL) borehole charts Por-14a, Por-14c, Por-14d; Ecoscope charts Neu-43, Neu-44, Neu-45, Neu-46 |
| Neutron Conversion | CNL NPHI-TNPH conversion chart Por-14e |
| SNP | Sidewall Neutron Porosity mudcake and matrix charts Por-15a and Por-13a |
| DLL/MSFL/MLL | Dual Laterolog (DLT-D/E) borehole charts Rcor-2b, Rcor-2c; Micro Laterolog mudcake chart Rxo-2; Micro SFL mudcake chart Rxo-3 |
| Induction | Induction borehole chart Rcor-4a; SFL borehole chart Rcor-3; Phasor Induction charts Rcor-4b, Rcor-4c |
| HALS Laterolog | **"No information at present."** (tab exists, provenance undocumented) |
| EPT | EPT-G mudcake charts EPTcor-3a through EPTcor-4b |
| DIL Invasion | DIL-SFL invasion charts Rint-2b, Rint-2c; Deep Induction-SFL-Rxo chart Rint-5; DIL-Rxo chart Rint-10 |
| DLL Invasion | Dual Laterolog (DLT-D/E)-Rxo invasion chart Rint-9b |

**Anadrill (SLB LWD)** - 3 tabs

| Tab | Charts as printed |
|---|---|
| CDN | 6 1/2" CDN: Por-19, Por-20a, Por-20b, Por-20c, Por-20d. 8" CDN: Por-24c, Por-24d, Por-24e |
| ADN | adnVISION475: Neu-31, Neu-33. adnVISION675: Neu-35, Neu-37. adnVISION475 BIP: Neu-32, Neu-34. "adnVISION475 BIP" 6 3/4": Neu-36, Neu-38 (printed name appears to be a typo for adnVISION675 BIP - quoted as printed, not corrected) |
| CDR | 6 1/2": Rcor-11a. 8": Rcor-11b. 9 1/2": Rcor-11c |

**Baker Atlas** - 8 tabs (chart numbers are the manual's best effort only, per its own admission)

Gamma Ray (4-1, 4-3, 4-13) | Density: Pe and bulk density borehole size for Compensated Z-Densilog
Series 2222 (6-7, 6-8, 6-9) and Series 2227 (6-10) | Neutron: Series 2418 (6-11, 6-29, 6-32 to 6-35),
Series 2420 (6-12 to 6-17, 6-26, 6-30, 6-32 to 6-35), Series 2435 (6-18 to 6-23, 6-27, 6-28, 6-31,
6-32 to 6-35, 6-39), Series 2436 (charts unnumbered), **Neutron 2446 "no information available"** |
SWNeu (6-24, 6-36) | Laterolog/Micro (unnumbered) | Induction: 809/815/818 (7-6, 7-7), 811 (7-2, 7-3),
814 (7-4, 7-5), Dual Induction 1503/1506 (7-8, 7-9) | Spectral GR (4-5, 4-7, 4-9, 4-11, 4-15, 4-17) |
DIL invasion and DLL invasion (unnumbered).

**Halliburton** - 8 tabs

Gamma Ray (GR1) | Density: SDL (POR-1), SLD (POR-2), HSDL (POR-3) | Neutron: DSN-II (POR-4a, POR-4b,
POR-5a), CNT-K (POR-6a, POR-6b, POR-7a), HDSN (POR-8a, POR-8b, POR-9a) | SNL (POR-16) |
DLL/MSFL: DLT-A (DLTA-1a, DLTA-1b), DLT-F (DLTF-1a, DLTF-1b), MSFL mudcake (Rxo-1), Micro guard (Rxo-2) |
Induction: DIL (DIL-1, DIL-2), DILTA (DILTA-1, DILTA-2), HRI (HRI-1, HRI-2), HDIL (HDIL-1, HDIL-2) |
DIL Invasion (DIL-4a/4b, DIL-5a/5b, DILTA-4a/4b, HRI-4a/4b, HDIL-4a to HDIL-6b) |
DLL Invasion (DLTA-3a/3b, DLTF-3a/3b) | Spectral GR (GR2).

**Baker Hughes INTEQ (LWD)** - 2 tabs

Resistivity: 3 1/8" MPR (1-2 to 1-17), 4 3/4" MPR (2-2 to 2-21), 6 3/4" MPR (3-2 to 3-29),
8 1/4" MPR (4-2 to 4-22), 6 3/4" NaviGator (5-2 to 5-13), 8 1/4" NaviGator (6-2 to 6-15) |
Neutron: 6 3/4" caliper-corrected (10-2 to 10-5), 8 1/4" caliper-corrected (11-2 to 11-4).

**Sperry-Sun (LWD)** - 4 tabs

Gamma Ray: DGR 4 3/4" (2-1, 2-8, 2-9, 2-10), 6 3/4" (2-2, 2-11 to 2-13), 8" (2-3, 2-14 to 2-16),
9 1/2" (2-4, 2-17 to 2-19), High Flow DGR 8" (2-5, 2-20 to 2-22), SOLAR 175 4 3/4" (2-6, 2-23 to 2-25),
SOLAR 175 6 1/2" (2-7, 2-26 to 2-28) | Resistivity: EWR-Phase 4 4 3/4" (3-1 to 3-4, 3-14 to 3-16),
6 3/4" (3-5 to 3-8, 3-17, 3-18), 8" (3-9 to 3-12, 3-19 to 3-22) | Neutron: CTN 4 3/4" (4-1, 4-2, 4-7),
CNF 6 3/4" (4-3, 4-4, 4-8), CNF 8" (4-5, 4-6, 4-8) | Lithology: CTN 4 3/4" (4-9), CNF 6 3/4" (4-10),
CNF 8" (4-11).

**Weatherford / Reeves** - 7 tabs

Gamma Ray: Compact (Gam-1, Gam-2, Gam-3), Reeves/Precision (4-1) | Neutron: Compact MDN
(Npor-5, Npor-6a, Npor-8 - printed as "Npor-5Npor-6a"), Reeves/Precision (6-4 to 6-6, 6-11, 6-16,
6-18 to 6-22) | Laterolog/Micro: Compact MDL (Lat-4, Lat-5, Lat-10, Lat-11, Micro-3),
Reeves/Precision DLL (5-2, 5-12, 5-13) | **Induction: "No details available."** |
FE Borehole: Compact Focused Electric (SFE-2) | DLL Invasion: Compact MDL (Lat-6, Lat-12) |
Spectral GR: Reeves/Precision (4-3, 4-5).

**PathFinder (LWD)** - 3 tabs

HDS1 Gamma Ray (POR-1a to POR-1j) | DNSC neutron (POR-2a to POR-2f) | SDNSC neutron (POR-2g to POR-2j).

### F1.3 Parameters and stated defaults, per tool type

The manual is explicit that its per-tool walkthrough "is not exhaustive but meant to clarify the operation
of the tabs to someone already experienced with applying environmental corrections". What follows is
everything it does state.

**Common to every tab**

- Caliper input curve **or** a fixed hole size in inches - "The Caliper and temperature input boxes always
  allow the user to enter a curve or a fixed value."
- Top Depth / Bottom Depth interval.
- Filter Curve checkbox with a level count. **Stated constraint: "the levels must be a whole, odd number."**
  Filtering can be applied *without* borehole correction by clearing the Borehole Correct box - i.e.
  resolution matching is a separate, orthogonal operation.
- Borehole Correct master checkbox - none of the individual corrections apply unless it is set.
- Save / Load of the parameter set to a disk file, in addition to automatic storage in the well database.
- **Environmental zones**: multiple zones per well, navigated with arrow buttons, each with its own saved
  set-up. The manual's stated use cases are "a new hole section with different bit size, a change of mud
  chemistry, change of logging tools".
- **Run Tab runs only the currently displayed tab.** Repeated on every company section, and reinforced by
  "NOTE: To update the current parameters you have to click Run Tab".

**Gamma Ray tab**

| Parameter | Stated options / default |
|---|---|
| Tool Position | Ecentered / Centered |
| Mud Type | Non barite / Barite |
| Hole Type | Open Hole / Cased Hole |
| Mud weight | user value, no default given |
| Tool Diameter | user value, no default given |
| Standoff | user value; "should be completed if Ecentered tool position was used" |
| Cased Hole | casing details, applied only if the GR is to be corrected for casing effects |

**Density tab**

| Parameter | Stated options / default |
|---|---|
| Density input curve, PEF input curve | required inputs |
| Mud weight | user value, no default given |
| Density Tool | FDC / LDT |

**Neutron (CNL) tab** - the richest tab, and the one with a real ordering mechanism

| Parameter | Stated options / default and behaviour |
|---|---|
| Neutron tool | CNT-A / CNT-C/D |
| Input Matrix / Output Matrix | Limestone / Sandstone / Dolomite |
| Input Neutron hole size corrected | checkbox; **if set, the existing caliper correction is backed out and re-applied through the Hole size correction** |
| Hole size | "The neutron tools response is calibrated for a hole of a particular diameter" |
| Mudcake | derived from Bit size vs Caliper/Hole Size. **Worked example given: "if the mudcake is 0.5\" and the bit size is 8.5\", enter the caliper as 8\"."** |
| Borehole Salinity | Kppm (NaCl eq) |
| Mud Weight | plus a Barite Mud checkbox selecting Natural Mud vs Barite Mud correction |
| Borehole Temperature | from the temperature curve or fixed value; DegF / DegC radio button |
| Pressure | single value, or **calculated from a depth curve plus mud weight as hydrostatic pressure**; Oil Mud checkbox with a Compressibility Multiplier. "NOTE: For deviated wells the TVD curve should be used for this correction." |
| Formation Salinity | Kppm (NaCl eq), for limestone formation salinity |

Matrix conversion is usable standalone: "If the user wishes simply to convert a Limestone matrix Neutron
curve to Sandstone or Dolomite matrix, clear the Hole size, Mudcake, Temperature etc.. check boxes, select
the Output Matrix radio button for the appropriate matrix and press the Run Tab button."

**Resistivity (DLL/MSFL/MLL) tab**

| Parameter | Stated options / default |
|---|---|
| Deep Laterolog, Shallow Laterolog, Micro Resistivity | inputs in ohmm |
| Mud Resistivity | with a reference temperature, "Mud Res Temp" |
| Mudcake Resistivity | with a reference temperature, "Mudcake Res Temp"; thickness from Bit size vs Caliper/Hole Size |
| Dual Laterolog Tool Type | DLT-B / DLT-D/E |
| Tool Position | Ecentered / Centered |
| Micro resistivity Tool Type | MSFL(regular) / MSFL(Slim hole) / MLL |

**No numeric petrophysical defaults are stated anywhere on this page** - no default hole size, no default
mud weight, no default salinity, no default temperature gradient. The companion `temperaturegradient` page
likewise gives no default gradient value; it only states the gradient is entered "in degrees per 100 feet
or metres, depending on the units of the well" and that the output curve's F/C unit flag is load-bearing
because "they are used in the interpretation modules to make the correct temperature conversions".
**Do not invent defaults for these.**

### F1.4 Order in which corrections must be applied

**The manual states no explicit global ordering for the wireline/LWD environmental corrections.** Saying so
plainly matters more than guessing. What it *does* state, and what can be relied on:

1. **Per-tab, not per-module.** "The Run Tab button only runs the corrections for the currently displayed
   tab page." Each tab is an independent operation with its own plot format; the user sequences them.
2. **Correction is zoned before it is ordered.** Environmental zones are created per hole section / mud
   change / tool change, and the correction set-up is per zone. Zoning is the outer loop.
3. **The one real ordering interlock is the neutron hole-size back-out.** The "Input Neutron hole size
   corrected" checkbox exists precisely because a caliper correction may already have been applied by the
   contractor; IP backs it out and re-applies its own. This is an explicit idempotency guard and SandiBumi
   should copy the *idea*: record whether a correction has already been applied, and make re-application
   safe rather than silently doubled.
4. **Matrix normalisation is a precondition of interpretation, stated twice.** Both `clayvolume` and
   `porosityandwatersaturation` state: "IP makes the assumption that any neutron curve entered is in
   **Limestone matrix units**. If this is not the case, then the curve should be converted to Limestone
   porosity units using the appropriate service company environmental correction module or the Basic Log
   Analysis Functions module." That is a hard ordering constraint: environmental correction and matrix
   conversion precede clay volume and PhiSw.
5. **Filtering is orthogonal, not sequential.** The filter can be run with borehole correction switched off,
   so resolution matching is deliberately decoupled from environmental correction.
6. **Contrast - image corrections DO have a mandated order**, and the manual says so in capitals
   (`imageanalysisoverview` line 158): "Image corrections MUST be applied in order, listed on the screen with
   the earliest correction at the top and the latest correction at the bottom. Applying a correction means
   previous corrections can not be run on the output Data." The fact that the manual imposes this for images
   and does *not* for log environmental corrections is itself evidence that no fixed log ordering is intended.

### F1.5 How the corrections actually ship - the architectural finding

Verified by inspecting (read-only) the IP2018 install directory. The environmental corrections are **not
compiled**. They ship as plain-text, column-oriented, user-editable lookup tables:

- **48 `.neu`** neutron correction tables, **149 `.ovl`** overlay tables, **12 `.cht`**, **18 `.bor`** -
  197 correction data files in total, named per contractor and tool
  (`Sch_CNL.neu`, `Sch_EcoScope_TNPH.neu`, `BA_2490_DEN_fresh.ovl`, `BHInteq_475_SDNv2.50_100.ovl`,
  `PA_675DNSC85.neu`, `WA_2435.neu`, `Reeves.neu`, `Hal_SNL.neu`, ...).
- A **registry file**, `Neu_Parm_Files.neu`, maps contractor to tool to table file, with its own documented
  constraints: "A maximum of 20 contractors is allowed. The tool name is read in first 15 characters.
  Tool and contractor names (Characters 1-10, 10 max). The 'lookup parameter' file name (Characters 16-46)".
- Each table is self-describing. `Sch_CNL.neu` header, quoted:
  "Contains lookup table for Schlumberger CNL TNPH ... Data is arranged in columns as follows :
  True Phi (Limestone Matrix), Sandstone Matrix, Dolomite Matrix, Salinity corr Sand, Salinity corr Lime,
  Salinity corr Dol ... Formation Salinity corrections are for following values 50, 100, 150, 200, 250 Kppm
  and in this order ... **Porosity values must not be changed**".

**What SandiBumi may take and what it may not.**

- **Take the architecture (Tier A).** Digitized chart tables as open text files + a contractor/tool registry
  + interpolation + per-zone parameter sets + per-tab execution. This is a proven, transparent, auditable
  design and it is exactly the opposite of Techlog's compiled black box. It is a genuine differentiator: a
  petrophysicist can see and QC the correction table.
- **Take the same file-format idea for other correction data.** The pore-pressure module uses the identical
  pattern - `.obg` overburden-gradient tables indexed by a master `OBG_Files.obg`.
- **Do not take the numbers.** The values in IP's `.neu`/`.ovl` files are IP's own digitization, and for
  Schlumberger wireline / Baker Atlas / Weatherford they are transcriptions of *vendor-supplied algorithms*
  rather than of public charts. Re-digitize from the published chart books listed in F1.1 (Jauhar already
  holds the digitization toolchain - `tools/chartdig`, dash-tip vector extraction, from the 2013 chartbook work).

### F1.6 Eastern European Resistivity Corrections (EERC) - narrative only

45.6k chars, **105 rasterized formulas**, and the `REFERENCES` heading is present but its body is **empty**
in the decompiled text. This page is therefore narrative-only by construction, and no equation on it may be
reconstructed.

What is recoverable and useful:

- **Provenance (Tier C module, Tier B theory).** "a specialist tool developed by the A.G.H University of
  Science and Technology, Krakow, and integrated into IP."
- **Scope.** Corrects Normal and Lateral resistivity curves for hole size and environmental effects,
  producing Rt, Ri, Rxo and Di (diameter of invasion). Two tabs: combined Lateral+Normal (largely automated,
  continuous, needs a **minimum of 3** Lateral and Normal curves), and Multiple Lateral Logs (bed-by-bed,
  requires manual bed-boundary definition, for when only Lateral curves exist).
- **Tool length rules, stated explicitly with worked examples** - fully recoverable and directly implementable:
  - Lateral devices: `Tool Length = AB/2 + AM`
  - Normal devices: `Tool Length = AM`
  - Worked: `B2.5A0.25M` (Normal), spacing B-2.5m-A-0.25m-M, tool length **0.25 m**.
    `B0.5A2M` (Lateral), spacing B-0.5m-A-2.0m-M, tool length **0.5/2 + 2.0 = 2.25 m**.
    `B7.5A0.75M` (Normal), tool length **0.75 m**.
  - "Often in FSU/Eastern European logging, the Sonde name provides the required information."
- **Radius of investigation.** "for Normal devices it is approximately twice the length"; the short Normal
  device is assumed to read the flushed zone (`Chapellier 4 1992`), and Ri is the average of the two Normal
  device resistivities.
- **Theoretical-curve regimes.** Lateral reference charts for `h/d > 32`; Lateral thin-bed charts for
  `h/d <= 32`. Chart sets are printable and copyable to clipboard from within the module.
- **Stated boundary condition, fully recoverable.** "When a measured point lies outside the space of the
  calculated theoretical curves, it is assumed that Rt/Rm = 1000 if it is above or Rt/Rm = 0.5 if it is
  below. In either case there is no differentiation between Dx0 and D."
- **Degraded-input rules.** With only three curves the F function is still computed from whichever ratios
  are available; if the second Normal device is missing only Rxo is obtained; if the shortest Normal device
  is missing the flushed zone is not determined and Ri comes from the remaining device alone.
- **Zoning and persistence.** Same environmental-zone model as the main package, saved to an external
  `.env` file.
- **Citations.** Ten inline superscript references survive (Stegun 1970, Alpin 1964, Bala et al 1999,
  Chapellier 1992, Jarzyna et al 1999, Dakhnov 1967, Jarzyna et al 2002, Ossowski 1990, Pierkov 1964,
  Pirson 1963). **The full reference list is not recoverable - do not complete these from memory.**

### F1.7 Calculation and Correction menu - the surrounding module set

From `calculationandcorrection` (3.9k, fully recoverable). The menu groups: User Formula, Multi Line User
Formula, Basic Log Functions, Temperature Gradient, Rw from SP, Gas Analysis, True Vertical Depth, TVT/TST,
Curves from Zones/Parameters, Curve Integration, and Environmental Corrections. Two design points worth
carrying across:

- **Basic Log Functions is tab-wise like the corrections** - "Each tab works as a stand-alone operation and
  the Run Tab button must be clicked for each page" - **except** in the Multi-Well Batch module, where
  "each tab will be run automatically, if an output curve name is specified and the module tick box is
  selected." Interactive execution is manual and per-tab; batch execution is driven by output-curve presence.
- **TVD is required over the whole well interval.** "It is essential that the TVD curve is calculated over
  the whole well interval. If the survey does not cover the whole well interval then a couple of assumptions
  are made by IP" - the assumptions themselves are not stated on this page.

---

## F2 - Tier-C Register (full-manual sweep)

Full detail in `F_tierC_register.json` (32 entries). Method: regex sweep of all 278 pages for
`patent`, `proprietary`, `licen[sc]ed`, `trademark`, `(TM)`, `(R)`, `copyright`, `all rights reserved`,
`courtesy of`, `used with permission`, `developed by`, `commercial product`, `consortium`, `supplied by`,
plus targeted vendor/brand-name passes, plus a read of the dedicated `third_party_software_licenses` page,
which turned out to be a three-line stub pointing at `EULA.rtf` in the install directory - so that file was
read too (read-only).

**Erring toward flagging** as instructed: several entries below are low legal risk (a bundled diff tool, a
chart control) and are included anyway, because the pattern they reveal - that the vendor's own EULA list
is *incomplete* - is itself the finding.

### F2.1 The five already-registered items, checked against IP2018

| Already-registered item | Present in IP2018? | Evidence |
|---|---|---|
| **SonicSaturation** (Omovie, US Patent 12,242,011 B2) | **NO** - zero hits for `sonic satur` or `omovie` across all 278 pages | absent |
| **Domain Transfer Analysis (DTA)** | **YES** | `curvepredictionusingdta`, 15.5k chars, **zero equations, zero citations**. The module is described purely operationally and defined circularly: "The Domain Transfer Analysis module uses Domain Transfer Analysis to allow the prediction of a result curve". |
| **Experienced Eye** | **NO** - zero hits | absent |
| **Entropy-based borehole-image speed correction** | **NO** as such - zero hits for `entropy` | IP2018 ships conventional **accelerometer-based** speed correction (`image_analysis` line 209), which is standard practice and adoptable. The entropy variant is a later replacement. |
| **Shipped neural-network weights** | **YES, and now explained** | See F2.2 item 1 - the engine is NeuroSolutions. |

So two of five are present in IP2018, and one of those two (the neural-network item) is materially clarified.

### F2.2 New Tier-C items found in IP2018

Ranked by how much they matter to SandiBumi.

1. **Neural Networks = NeuroSolutions 5.5, a commercial third-party product.**
   `neural_networks` line 57: "The neural network that IP uses is a commercial product by Neuro Solutions -
   http://www.neurosolutions.com/ . The IP Neural Network are built with Neuro Solutions 5.5. The number of
   Hidden layers = 1." This resolves the already-registered "shipped neural-network weights" item: the
   weights are NeuroSolutions artifacts. **And NeuroSolutions is absent from the IP2018 EULA third-party
   list** - so its terms are undocumented in shipped material and it should be treated as maximally restricted.

2. **The entire Unconventional Resources (UCR) toolbox was developed by Apache Corporation.**
   `ucr` line 3048: "This tool suite was developed by Apache Corporation and is licenced for use in
   Interactive Petrophysics." That is 92k characters - oil & gas pressure, fluid properties, oil & gas in
   place, brittleness, rock-mechanics profiling, multi-well spreadsheet reporting - all operator IP, not
   vendor method. Nothing on that page is vendor-published. Its two literature citations (Barree et al
   SPE 118701; Whitson & Brule *Phase Behavior* Monograph 20) **are** Tier B and independently usable, and
   the manual itself says the Whitson & Brule algorithms "are standard algorithms in use in the petroleum
   industry."

3. **DNOPT Dense Nonlinear OPTimizer (Stanford) is the Mineral Solver's non-linear core**, and
   **Numerical Recipes (Cambridge University Press) SVD is its linear core.**
   `mineral_solver` lines 601-603. Both are separately licensed code. The *methods* (SVD on a normalised
   linear system; dense non-linear optimisation) are Tier A/B and SandiBumi should use LAPACK-derived SVD
   and an independently licensed optimizer. The *implementations* must not be adopted. **IMSL Numerical
   Fortran Libraries (RogueWave)** also appears in the EULA, with no module attributed in the help.

4. **Three vendors supplied environmental-correction algorithms rather than charts** - Schlumberger
   ("Green Book library"), Baker Atlas ("received ... as a series of algorithms"), Weatherford
   ("algorithms distributed by Weatherford"). Covered in F1.0; repeated here because it is the highest-value
   *negative* finding of the whole sweep.

5. **Two operator/academic-contributed modules**, both of which happen to be documented well enough to be
   reachable by other means: **Eastern European Resistivity Corrections** (A.G.H University, Krakow - theory
   openly attributed to Alpin/Dakhnov/Pierkov/Pirson et al) and **Laminated Reservoir Fluid Substitution**
   ("developed by Chris Skelt, Chevron" - and Skelt published both papers, which the manual cites). The
   modules are Tier C; the underlying methods are Tier B.

6. **Two self-labelled proprietary algorithms inside otherwise-open pages.**
   - `acoustic_waveform_processing` line 865: "**A proprietary fit function** is applied to the 2D Frequency
     Semblance map" for dispersion correction - on a 74k page whose semblance core is openly credited to
     Kimball and Marzeta 1984.
   - `total_organic_carbon_content` line 57: "There are also **2 proprietary regressions**, a 3rd order and
     a 5th order" - on a page whose Delta LogR implementation is openly credited to Passey et al 1990.
   In both cases the boundary is sharp and favourable: the cited part is adoptable, the proprietary part is not.

7. **Licence-gated or operator-donated data and programs.** CSM-UH FLAG fluid calculator ("Only available to
   Colorado School of Mines-University of Houston consortium members"); the Unocal Offshore Texas/Louisiana
   overburden-gradient dataset ("$ Supplied by Unocal March 2003" inside the shipped `.obg` file). By
   contrast the two Amoco GOM OBG curves *are* traced to published work (Eaton & Eaton Figure 2; Barker &
   Wood 1997) and are therefore re-derivable.

8. **Proprietary real-time data links** - Osprey Connect ("a Schlumberger proprietary data link technology")
   and XStream Connect ("technology developed by DK Energy"). The underlying **WITSML standard is open** -
   the manual records it as a BP/Statoil/Shell-sponsored industry initiative, and the Energistics Standards
   DevKit is Apache 2.0 in the EULA. So SandiBumi can implement WITSML; it cannot implement either link.

9. **Third-party database connectors and marks** - Paradigm GEOLOG6(R), Landmark OpenWorks(R), PETCOM
   Powerlog(R), Shell LOGIC, TIBCO OpenSpirit, Schlumberger Petrel, Senergy ODM3, and the "Petrel (TM)"
   tops export format. The OpenWorks and GEOLOG links additionally depend on vendor-side binaries
   (`owlnk`, `gllnk.cgi`) that cannot be reimplemented. Tier A takeaway: this *is* IP2018's competitive
   integration surface, and it is worth knowing precisely.

10. **Bundled commercial components** - Larson CGM converter, ExamDiff (PrestoSoft, "used with permission"),
    TeeChart.NET 2010/2017 and TeeChart Pro v8 (Steema), DotNetBar (DevComponents), MstGrid, Bytescout PDF,
    VB Migration Partner, Delphi/CodeGear RAD Studio 2007 + Embarcadero XE4, Intel Fortran Compiler, Oracle
    Data Provider for .NET, and a long Microsoft stack including Silverlight. Zero method content, but the
    *composition* is intelligence: **VB Migration Partner + Delphi + Intel Fortran + .NET says IP2018 is a
    multi-generation migrated codebase**, which explains the module-by-module inconsistency in the manual
    and is a direct argument for SandiBumi's single-stack design.

11. **Two components are bundled but absent from the vendor's own EULA list** - NeuroSolutions and Larson
    CGM. **Treat the EULA list as indicative, not as a complete clearance record.**

12. **Copyleft flag** - the EULA declares "StackOverflow & Wikipedia references, CC By-SA v3.0". Unusual for
    a commercial vendor, and a share-alike obligation. Lesson for SandiBumi: track and attribute anything
    lifted from those sources rather than silently absorbing it.

13. **Vendor lineage** (Tier A, market intelligence). The help disclaimer says "LR" while the shipped
    `EULA.rtf` still says "Senergy Software Limited", and `environmentalcorrections` still says "Senergy
    Software Ltd". IP2018 is mid-rebrand PGL/Senergy to Lloyd's Register; pages still saying "Senergy" are
    likely unchanged since the Senergy era, which is a useful dating heuristic for the rest of the ingest.

### F2.3 Class-level flag: vendor tool trade names

Rather than one row per mark, one class entry: the manual uses roughly 50 vendor tool trade names
(Ecoscope, adnVISION475/675, MRIL / MRIL-Prime / MRI-LWD, NaviGator, MPR, SOLAR 175, DGR, EWR-Phase 4,
Compact MDN/MDL/SFE, HDS1, DNSC/SDNSC, HALS, Phasor, FDC, LDT, CNL, SNP, EPT, DLT-A/B/D/E/F, MSFL, MLL, MG,
HDIL, HRI, DIL, DILTA, DSN-II, CNT-A/C/D/K, HDSN, SDL, SLD, HSDL, SNL, Z-Densilog 2222/2227,
CN 2418/2420/2435/2436/2446, CBIL, STAR, XRMI, EMI, OMRI, OBMI, USI, SCMT, CAST, SBT, RBT).

Using these names to **identify** which tool produced a curve is ordinary nominative use and is Tier A -
SandiBumi needs them in its tool registry. The hazard is the **response function** behind each name, which
is Tier A/B or Tier C depending entirely on the F1.0 table. **Never assume a named tool implies an
adoptable correction.**

---

## F3 - Citation Harvest

124 deduplicated entries in `F_citations.json`. Method: two automated sweeps over all 278 pages (a
year+venue+author-pattern pass, then a `published / paper by / as described by / comes from / derived by`
pass), plus a named-author pass over ~90 method-author surnames, plus targeted reads of every page with a
`References` section. Then manual verification of each hit against its source line.

**Every citation below is quoted as the manual prints it.** Where the manual is internally inconsistent,
typo'd or incomplete, that is preserved and annotated rather than corrected - a fabricated reference is
worse than a missing one. Where a method is present but uncited, the row says `not given in manual`.

**This section is the reimplementation reading list.** For each row: what to cite, what it buys you, where
in the manual it came from.

### F3.1 Environmental corrections and chart books

| Citation as printed | Supports | Page |
|---|---|---|
| Schlumberger - Log Interpretation Charts (2000) | SLB wireline correction tabs; also the Anadrill LWD tabs | environmentalcorrections |
| 2005 Schlumberger Log Interpretation Charts book | Anadrill CDN / ADN / CDR tabs | environmentalcorrections |
| Schlumberger Log Interpretation Charts 2010 - Chart CEM 1 | Cement Evaluation expected CBL response (mV) vs casing OD and cement strength | cementeval |
| Halliburton - Log Interpretation Charts (1994) | Halliburton GR / Density / Neutron / SNL / DLL-MSFL / Induction / invasion / Spectral GR tabs | environmentalcorrections |
| Baker Atlas - Log Interpretation Charts (1984) | Baker Atlas tabs - **but the manual states the corrections came as algorithms, not as this chart book** | environmentalcorrections |
| Baker Hughes Inteq 2002 Log Interpretation Charts (2002) | INTEQ LWD MPR / NaviGator resistivity, caliper-corrected neutron | environmentalcorrections |
| Sperry Sun - Log Interpretation Charts (1996) | Sperry-Sun tabs. **Same page also says 1998; both printed** | environmentalcorrections |
| Weatherford Log interpretation Charts Compact Tool Series (2007) | Weatherford Compact tabs (Gam, Npor, Lat, Micro, SFE) | environmentalcorrections |
| Weatherford Log interpretation Charts Standard Tool Series (2007) | Weatherford reference chart book | environmentalcorrections |
| Weatherford Log Interpretation Charts LWD Tool Series (2005) | Weatherford LWD reference chart book | environmentalcorrections |
| Precision Wireline Services Log Interpretation Chart Book | Weatherford/Reeves legacy tabs. **No year printed** | environmentalcorrections |
| PathFinder LWD Log Interpretation Charts (2007) | PathFinder HDS1 GR, DNSC and SDNSC neutron tabs | environmentalcorrections |
| the Western Atlas chartbook | Timur, Morris Biggs oil and Morris Biggs gas permeability defaults | basiclogcalculations |
| Western Atlas chart book (8-6 Rev1 12-95), charts 8-4 and 8-6 | Sigma Oil from GOR; wet-gas and condensate corrections to methane sigma | basiclogcalculations |
| the Schlumberger chartbook - Schlumberger Chart K3 | Permeability default | basiclogcalculations |
| Schlumberger chart for Methane (Tcor-1) | Sigma Gas; sigma hydrocarbon lookup | basiclogcalculations; sigma |
| Schlumberger Tcor-2 | Sigma Water from formation water salinity | sigma |

### F3.2 Mud, fluid and formation properties

| Citation as printed | Supports | Page |
|---|---|---|
| "Estimation of Mud Filtrate Resistivity in Fresh Water Drilling Muds" The Log Analyst (March-April 1986) [Lowe and Dunlap] | Rmf from Rm (this option does not calculate Rmc) | basiclogcalculations |
| "A Correlation of the Electrical Properties of Drilling Fluids with Solids Content" Transactions AIME (1958) [Overton and Lipson] | Rmf/Rmc from Rm; valid below 70 Kppm | basiclogcalculations |
| Batzle and Wang "Seismic Properties of Pore Fluids", Geophysics (1992) | Downhole oil/gas/water densities; Rock Physics and Fluid Substitution fluid properties | basiclogcalculations; fluidsubstitution; laminatedfluidsubs |
| Whitson, Curtis H. and Brule, Michael L.: Phase Behavior, Monograph Volume 20, Henry L. Doherty Series, SPE, Richardson, Texas (2000) | UCR fluid property algorithms - manual states these are standard industry algorithms | ucr |
| NMR Logging: Principles and Applications by Coates, Xiao and Prammer (1999) | Gas hydrogen index and T1 relationships | basiclogcalculations |
| not given in manual [Arps equation] | Temperature correction of the resistivity log in Pore Pressure | porepressurecalculations2 |

### F3.3 Clay volume, porosity and water saturation - **the notable gap**

| Citation as printed | Supports | Page |
|---|---|---|
| **not given in manual** [Clavier - printed only as "Clavier :" and "As per Clavier et al."] | Clay volume from GR - Clavier | clayequationsandmethodology; clayparameters |
| **not given in manual** [Stieber (South Louisiana Miocene and Pliocene); Stieber Constant shape parameter **default = 2.0**] | Clay volume from GR - Stieber | clayequationsandmethodology; clayparameters |
| **not given in manual** [Larionov older rocks (Mesozoic); Larionov younger rocks (Tertiary clastics)] | Clay volume from GR - Larionov | clayequationsandmethodology; clayparameters |
| **not given in manual** [Archie; Simandoux; Modified Simandoux; Indonesian (Poupon-Leveaux); Waxman-Smits; Dual Water; Juhasz (Waxman-Smits)] | The core Sw equations. Equations printed, **no primary reference for any of them** | swequationsandmethodology; swparameters; basicloganalysis |
| **not given in manual** [Wyllie time average; Hunt-Raymer / Raymer] | Sonic porosity. Manual recommends Raymer as default but cites neither | swequationsandmethodology; basicloganalysis; minsolveeqandmeth |
| "A contribution to electric log interpretation in shaly sands", Poupon A, Loy ME, Tixier MP (1954) Trans AIME 6(06):138-145 | Poupon-Tixier Sw, with added m and n exponents | swequationsandmethodology; minsolveeqandmeth |
| "Extensions of Pickett Plots for the Analysis of Shaly Formations by Well Logs", Roberto Aguilera (The Log Analyist, Sept-Oct 1990) | Poupon-Aguilera Sw (modified Poupon with added exponents) | swequationsandmethodology; minsolveeqandmeth |
| SPWLA paper "Athabasca Tar Sands Reservoir Properties Derived from Core and Logs" 1976 17th annual Logging symposium by R. Woodhouse | "Woodhouse Tar" Sw equation | swequationsandmethodology; minsolveeqandmeth |
| the Juhasz SPWLA paper "Assessment of the distribution of shale, porosity and hydrocarbon saturation in shaley sands" | Thomas-Stieber laminated/dispersed/structural volumes, reformulated for Phie and Vcl. **No year or paper number given** | porosityandwatersaturation |
| "Log Interpretation in the Malay Basin" by K. Kuttan et al, 21st SPWLA symposium | Sand/Silt Malay Model - "based loosely on" | sand_silt_malay_model; interpretation |
| Numerical Recipes (Cambridge University Press) - Singular Value Decomposition | Mineral Solver linear solver. **Tier C - see register** | mineral_solver; minsolveeqandmeth |

**This is the most consequential row-set in the harvest, and it is consequential for what is missing.**
IP2018 prints the Clavier, Stieber and Larionov equations and the full Sw equation family **with no primary
citation whatsoever**. If a deep-dive agent reports a citation for any of these from IP2018, it is fabricated.
The equations themselves are almost entirely rasterized (`clayequationsandmethodology` 16 images,
`swequationsandmethodology` 121 images), so this manual is *not* a usable source for either the formulas or
their provenance. Route both to Jauhar's own documented sources - the ITB team shelf, the chartbook, the
petro-kb notes - never to IP2018.

### F3.4 Petrophysics - NMR, capillary pressure, flow units, TOC, images

| Citation as printed | Supports | Page |
|---|---|---|
| (Coates et al, "A new characterization of bulk-volume irreducible using magnetic resonance", paper QQ 38th Annual SPWLA Symposium 1977) | Timur/Coates and Modified Coates NMR permeability. **Year and symposium number as printed are internally inconsistent - quoted verbatim, not corrected** | nmrinterpretation |
| Constructing Capillary Pressure Curves from NMR log data in the presence of Hydrocarbons, Yakov Volotkin, Wim Looyestijn, Walter Slijkerman, Jan Hofman, SPWLA 40th Annual Logging Symposium, May 30 - June 3, 1999 | NMR-derived capillary pressure | nmrinterpretation |
| Hill, Shirley and Klein 1979 (SPWLA 20th annual Symposium Paper AA - "The Central Role of Qv and Formation Water Salinity in the Evaluation of Shaley Formations") | Capillary pressure clay correction | cappressuresetup |
| not given in manual [Leverett J Function; Thomeer; Brooks Corey; Skelt Harrison] | Pc curve-fitting functions. Forms printed, no primary citation | cappressurefunctions |
| Jerry Lucia book Carbonate Reservoir Characterization, 2007 published by Springer | Lucia Rock Fabric Number classes and class Swi equations | hfu |
| SPE paper 84942 ... August 2003 by James W. Jennings Jr and F. Jerry Lucia | Lucia rock class typing / carbonate permeability | hfu |
| not given in manual [Winland R35 - "created by Dale Winland of Amoco"; 0.5 micron R35 pay cut from the Spindle Field] | HFU Winland method | hfu |
| not given in manual [Pittman; Kozeny-Carmen; RQI] | HFU RQI and Pittman methods | hfu |
| "A Practical Model for Organic Richness from Porosity and Resistivity Logs" by Q.R.Passey, S.Creaney, J.B.Kulla, F.J.Morettu and J.D.Stroud, AAPG Bulletin V.74 No 12 December 1990 | Delta LogR TOC (sonic, density, neutron overlays; LOM; Passey Modifier) | total_organic_carbon_content |
| "Fracture apertures from electrical borehole scans" by S. M. Luthi and P. Souhaite, Geophysics, Vo1.55, No. 7, July 1990 | Fracture aperture, fracture porosity and permeability from image data | plotting_image_analysis_data |
| Well Log Normalization : Methods and Guidelines - Daniel E. Shier, Petrophysics, Vol45, No.3 (2004) | Histogram log normalization | histogram |

### F3.5 Acoustics, rock physics and seismic tie

| Citation as printed | Supports | Page |
|---|---|---|
| "Semblance processing of borehole acoustic array data" by Kimball and Marzeta, "Geophysics" Vol.49 Mo.3, March 1984 | Acoustic Waveform Processing slowness/time coherence core | acoustic_waveform_processing |
| Greenberg-Castagna (1992) empirical relationships | DTS creation and shear-velocity QC; defaults are the 100% brine-saturated constants | shearsonic |
| P. Connolly, The Leading Edge (1999), equation 4.1 (high angle inversion) | Elastic Impedance | elastic_impedance; rockphysics |
| Brie A, et al, "Shear Sonic Interpretation in Gas-Bearing Sands", 1995, SPE 30595 (pp701-710) | Two-phase fluid mixing law exponent (patchy saturation) | fluidsubstitution; laminatedfluidsubs |
| The Rock Physics Handbook ... by G. Mavko, T. Mukerji and J. Dvorkin (1999) | Default mineral matrix properties | fluidsubstitution |
| Mavko, Mukerji and Dvorkin, 1998, "The Rock Physics Handbook", Cambridge University Press | Laminated fluid subs rock/mineral properties | laminatedfluidsubs |
| Gregory (1977) [also printed "as per Gregory 1977, Hampson & Russell"] | Vs from dry rock Poisson ratio for Gassmann | fluidsubstitution |
| Skelt C, "Fluid substitution in Laminated Sands", The Leading Edge, May 2004 | Laminated Reservoir Fluid Substitution | laminatedfluidsubs |
| Skelt C, "The influence of shale distribution on sensitivity of compressional slowness to reservoir fluid changes", SPWLA 45th Annual Symposium, 2004 | Laminated Reservoir Fluid Substitution | laminatedfluidsubs |
| Published data from Murphy (1982) [coordination number] | Grain-contact model parameter | laminatedfluidsubs |
| R.Beardsley, personal communication, 2007 | Laminated fluid subs parameter - **not a citable publication** | laminatedfluidsubs |
| Aki & Richards, 1980 - first order linear form of the Zoeppritz equation | Angle-dependent reflectivity | rockphysics; synthetic_seismorgram |
| Zoeppritz equations (1912) | Reflection coefficients at angle of incidence | synthetic_seismorgram |
| Backus average (Backus, 1962) | Optional log upscaling to seismic frequency | synthetic_seismorgram |
| Gardner G. L. F., Gardner L.W., & Gregory A.R. (1974) ... Geophysics 39, 770-780 | Density from sonic (Gardner); overburden gradient | densityestimation2; porepressurecalculations2 |
| Bellotti, P. Di Lorenzo, V. & Giacca, D. - Overburden gradient from sonic log, Trans. SPWLA, London March 1979 | Density from sonic (AGIP Bellotti); overburden gradient | densityestimation2; porepressurecalculations2 |
| Lindseth, R. O., (1979) Synthetic Sonic Logs ..., Geophysics v.44 no.1 p.3-26 | Density from sonic (Lindseth) | densityestimation2; porepressurecalculations2 |

The Synthetic Seismogram module additionally prints a 15-item reference list (Box & Lowrey 2003;
Box et al 2004; Burch 2002 x2; Ewing 1997 x2; Henry 1997 x2 and 2000; Peterson, Fillippone & Coker 1955;
Ricker 1953 - printed "19530"; Roden & Sepulveda 1999; Stewart & Kong 1984; White & Hu 1998;
Ziolkowski et al 1998). All 15 are captured verbatim in `F_citations.json`.

### F3.6 Geomechanics and pore pressure - the densest cited area in the manual

`rock_strength`, `rock_stress`, `wellbore_stability` and `porepressurecalculations2` are, unusually,
**almost fully recoverable** (0, 0, 0 and 13 rasterized images respectively across 172k characters) **and**
every model is attributed. This is the best-documented part of IP2018.

| Citation as printed | Supports | Page |
|---|---|---|
| Sarda et Al., 1993 (SPE 26454, "Use of Porosity as a Strength Indicator for Sand") | UCS sand from porosity and from NPHI | rock_strength |
| Formel et Al., 1993 (SPE 36533, "FORMEL: A Step Forward in Strength Logging") | UCS sand from porosity, NPHI and sonic | rock_strength |
| Vernik et Al., 1993 (Int. J. Rock Mech. Min. Sci. & Geomech., "Empirical relations between compressive strength and porosity of siliciclastic rocks") | UCS sand - Vernik and Modified Vernik (with Vclay) | rock_strength |
| Coates and Denoo, 1981 (SPWLA 1981 22nd Annual Logging Symposium, "Mechanical properties program using borehole analysis and Mohr's circle") | UCS sand from static Young's modulus and Vclay | rock_strength |
| Chang, 2004 ("Empirical rock strength logging in boreholes penetrating sedimentary formations") | UCS carbonate from sonic and from porosity | rock_strength |
| Khaksar et. al, 2009 (SPE121972, "Rock strength form core and logs: where we stand and ways to go") | UCS carbonate/dolomite and friction angle | rock_strength |
| Lashkaripour and Dusseault, 1993 (Probabilistic Methods in Geotechnical Eng., "A Statistical Study on Shale Properties ...") | UCS shale from porosity | rock_strength |
| Horsrud, 2001 (June 2001 SPE Drilling & Completion, "Estimating Mechanical Properties of Shale from Empirical Correlations") | UCS shale from sonic and porosity | rock_strength |
| Horsrud, 2001 (SPE 56017, "Estimating Mechanical Properties on Shale from Empirical Correlations") | Friction angle for shale from sonic. **Printed as a distinct source id from the row above; both appear** | rock_strength |
| Veeken et Al., 1991 (SPE 22792, "Sand Production Prediction Review: Developing an Integrated Approach") | TWC sand strength from density and sonic | rock_strength |
| Perkins and Weingarten, 1988 (SPE18244, "Stability and Failure of Spherical Cavities in Unconsolidated Sand and Weakly Consolidated Rock") | Friction angle from porosity | rock_strength |
| Stein and Hilchie, 1972 (Jour. Pet. Technol., "Estimating the maximum production rate possible from friable sandstones without using sand control") | Ec combined elasticity modulus without dipole sonic | rock_strength |
| Lacy et Al., 1997 (SPE 38716, "Dynamic Rock Mechanics Testing for optimized Fracture Designs") | Static from dynamic Young's modulus | rock_strength |
| Morales et Al., 1993 (SPE 26561, "Fracturing of High Permeability Formations: Mechanical Properties Correlations") | Static from dynamic Young's modulus | rock_strength |
| Breckels and Van Eekelen, 1982 (Soc. Pet. Eng. Journ., "Relationship between Horizontal Stress and Depth in Sedimentary Basins") | Minimum horizontal stress from TVD and pore pressure | rock_stress |
| Anderson et al., 1973 (SPE-4135-PA, "Determining Fracture Pressure Gradients from Well Logs") | Min horizontal stress from Pp, Sv, Poisson, Biot | rock_stress |
| Daines, 1982 (SPE-9254-PA, "Prediction of Fracture Pressures for Wildcat Wells") | Min horizontal stress - Eaton plus tectonic term | rock_stress |
| Anderson, 1951 (Oliver and Boyd, "The Dynamics of Faulting and Dyke Formation") | Theoretical elastic horizontal stress model | rock_stress |
| Addis, Last and Yassir, March 1996 (SPE-28140-PA, "Estimation of Horizontal Stresses at Depth in Faulted Regions ...") | Tectonic modification of the elastic model | rock_stress |
| Mark D Zoback, "Reservoir Geomechanics", Cambridge 2007 (ISBN 978-0-521-14619-7) | Wellbore Stability module | wellbore_stability |
| Barree, R.D., Gilbert, J.V., and Conway, M.W.: "Stress and Rock Property Profiling for Unconventional Reservoir Stimulation," paper SPE 118701, SPE Hydraulic Fracturing Technology Conference, Woodlands, Texas (January 19-21, 2009) | UCR static vs dynamic Young's modulus | ucr |
| Mouchet, J-P., Mitchell, A. Abnormal Pressures While Drilling - Manuels Techniques 2 Elf Aquitaine (1989) | Pore pressure methodology; cited as critical of all fracture-gradient methods | porepressurecalculations2 |
| Traugott, M. - Pore Pressure and Fracture Pressure Determinations in Deepwater - Deepwater Technology Supplement to World Oil, August 1997 | Amoco average-sediment-density OBG | porepressurecalculations2 |
| Eaton B.A. and Eaton T.L. Fracture gradient prediction for the new generation - World Oil (October 1997) | Poisson's Ratio for US Gulf Coast and Deepwater GOM, 0-5000 ft and >5000 ft below mud line | porepressurecalculations2 |
| Barker, J. W. and Wood, T.D. - Estimating Shallow below Mudline Deepwater Gulf of Mexico Fracture Gradients (1997) Houston AADE Annual Technical Forum | Deepwater GOM shallow-BML fracture gradient | porepressurecalculations2 |
| Eaton, B.A. (1972) Graphical Method Predicts Geopressures Worldwide - World Oil 182, 6, 51-56 | Eaton pore pressure | porepressurecalculations2 |
| Matthews, W.R. and Kelly, J.(1967) - How to Predict Formation Pressure and Fracture Gradient - Oil &Gas Journal, 65, p92-106 | Matthews & Kelly fracture gradient. Manual warns it assumes constant OBG of 1 psi/ft and is "of little use outside the GOM coast region" | porepressurecalculations2 |
| Eaton B. A., (1969) Fracture Gradient Prediction and its Application in Oil Field Operations - J.Pet.Tech. 21, p1353-1360 | Eaton fracture gradient | porepressurecalculations2 |
| Daines S.R. (1980) The Prediction of Fracture Pressures For Wildcat Wells - SPE 9254 | Daines fracture gradient with tectonic component | porepressurecalculations2 |
| Simmons E.L. & Rau, W.E. (1988) Predicting Deepwater Fracture Pressures: A Proposal (SPE 180250) | Offshore water-column modification to Eaton | porepressurecalculations2 |
| Pore Pressure and Fracture Gradients - SPE Reprint Series No.49 (1999 edition) | Pore pressure general reference | porepressurecalculations2 |

### F3.7 Statistics, prediction and mud-gas

| Citation as printed | Supports | Page |
|---|---|---|
| Cuddy, S. (1997) "The Application of the Mathematics of Fuzzy Logic to Petrophysics" (Paper S. 38th Annual Symposium of the SPWLA) | Fuzzy Logic Curve Prediction | statisticalcurveprediction; specialinterpretation |
| 'Spherical self-organizing map using efficient indexed geodesic data structure', Neural Networks 19 (2006), Yingxin Wua, Masahiro Takatsukab | SOM distortion (IP uses a stated modified version) | som |
| J.L Hodges and E. L Lehmann, Basic Concepts of Probability and Statistics, Holden-Day 1970 | 'Lateral' VCL average - median of pair products, stated as almost identical to Hodges-Lehmann | curve_average; clayvolume |
| "Statistical Regression Line-Fitting in the Oil and Gas Industry" by Richard (Dick) Woodhouse, 2002, PennWell Publishers | Crossplot regression background reading | crossplot_functions |
| "Interpretation of Hydrocarbon Shows Using Light (C1-C5) Hydrocarbon Gases from Mud-Log Data" Haworth et al (1985) | Gas Analysis - Haworth Wh (GWR), Bh (LHR), Ch (OCQ) ratios | gas_analysis |
| **not given in manual** [Domain Transfer Analysis] | DTA curve prediction - **no reference of any kind in IP2018** | curvepredictionusingdta |

Directly relevant to Jauhar's Mahakam mud-log work: the Haworth 1985 citation is the provenance for the gas
ratios, and the manual prints the IP formulations as
`GWR = (C2 + C3 + C4 + C5) / (C1 + C2 + C5) * 100 (which is Haworth's Wh)`,
`LHR = (C1 + C2) / (C3 + C4 + C5) (which is Haworth's Bh)`,
`OCQ = (iC4 + nC4 + C5) / C3 (which is Haworth's Ch)`. Quoted as printed - note the C5 appearing in the GWR
denominator, which is worth checking against Haworth before adoption rather than assumed to be a typo.

### F3.8 East European resistivity theory

Ten inline superscript references, recoverable as author + year only. The reference-list body is empty in
the decompiled text and **must not be completed from memory**: `Stegun 1 1970`, `Alpin 2 1964`,
`Bala et al 3 1999`, `Chapellier 4 1992`, `Jarzyna et al. 5 1999`, `Dakhnov 6 1967`, `Jarzyna et al. 7 2002`,
`Ossowski 8 1990`, `Pierkov 9 (1964)`, `Pirson 10 1963`. What each supports is recorded in `F_citations.json`.

---

## F4 - Manual Map

Delivered as `F_manual_map.md`: all 278 pages in 15 functional groups, each row carrying the page id, title,
character count, and recoverability marked `FULL` (`eq_images == 0`) or `PARTIAL (n)`.

Headline numbers: **278 pages, 252 fully recoverable (91%), 26 partial, 587 rasterized formulas in total.**

The rasterization is extremely concentrated. Six pages hold 461 of the 587 images - **78%**:

| Page | Title | Rasterized formulas |
|---|---|---|
| `swequationsandmethodology` | Porosity and Sw Equations and Methodology | 121 |
| `easteuroperescorrections` | Eastern European Resistivity Corrections | 105 |
| `minsolveeqandmeth` | Mineral Solver Equations and Methodology | 99 |
| `fluidsubstitution` | Fluid Substitution | 64 |
| `laminatedfluidsubs` | Laminated Reservoir Fluid Substitution | 38 |
| `nmrinterpretation` | NMR Interpretation | 34 |

The pattern is not accidental and is worth stating plainly: **the pages that carry the actual method
mathematics are precisely the pages whose mathematics is not recoverable.** Everything else - workflow,
parameters, defaults, provenance, citations, file formats, architecture - is open text. The correct posture
for the rest of this ingest is therefore to mine IP2018 for *parameters, provenance, architecture and
citations*, and to source the *equations themselves* from the cited primary literature and from Jauhar's own
documented references.

The three highest-value fully-recoverable pages for SandiBumi are `environmentalcorrections` (38.5k, 0 images -
the subject of F1), `rock_strength` (51.8k, 0 images - 14 fully attributed strength models) and
`acoustic_waveform_processing` (74k, 0 images - a complete slowness-processing description with a Tier B core).
