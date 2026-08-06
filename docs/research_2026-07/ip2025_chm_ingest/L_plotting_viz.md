# L — Plotting, Crossplots, Histograms & Visualization

**Agent:** L (plotting / crossplots / histograms / visualization)
**Source:** Interactive Petrophysics 2025 vendor manual, decompiled CHM (`c25`), with IP2018 counterpart (`c18`) for numeric diffing.
**Date:** 2026-08-06
**Consumer:** SandiBumi

**Provenance conventions used throughout**
- `(pagename.htm)` — fact read from the extracted prose of that page.
- `[img-read: file.png]` — fact transcribed by reading that raster image directly (vision).
- Overlay-chart *data* (digitized chart-line coordinates) is deliberately **not** transcribed — chart identity, axes and applicability only (ingest rule 5).
- Where two vendor statements conflict, **both are reported** and the conflict is logged in §6. No textbook value has been substituted anywhere.

---

## 1. Scope & page inventory

All 46 assigned pages accounted for. "Content" = substantive extraction; "UI-only" = mechanics with no defaults/equations/conventions worth carrying.

### 1.1 Crossplot group

| # | Page | Outcome |
|---|---|---|
| 1 | `crossplottypes.htm` | **Content.** Enumerates 3D, Frequency, Pressure-Gradient, **Standalone Pickett**, Multi-Well, Multi-Curve, Multi-Zone crossplots + chart tooltips. Pickett constraints and the Rw/m/n/a dialog. Max 500 wells / 8 curves + Z in Multi-Curve. |
| 2 | `createacrossplot.htm` | **Content.** Scales / Discriminators / Regression Discriminators / Z-axis colors / Overlays / Options / Histograms tabs. Most of the crossplot default and limit table (§4.1) comes from here. |
| 3 | `crossplot_functions.htm` | **Content.** Areas, flag-curve creation, regression (OLS/RMA/Robust/Poly/Exp/Power) with the R² equation, pressure gradients, frequency-crossplot binning, user lines, **overlay file format**, logplot↔crossplot interactivity. |
| 4 | `crossplotsfeatures.htm` | **Content.** Crossplot format storage hierarchy, generic curve-type syntax (`@`/`*`), zone-list behaviour and its defaults, Pin default state. |

### 1.2 Histogram group

| # | Page | Outcome |
|---|---|---|
| 5 | `histogram.htm` | **Content (heaviest).** Binning, log-mean equation, statistics set, Gaussian/SD/percentile lines, multi-well histograms, and the **entire curve-normalization specification** (§3). |

### 1.3 Ternary / statistical plots

| # | Page | Outcome |
|---|---|---|
| 6 | `ternary.htm` | **Content.** Unity-equation normalization algorithm (per-depth), apex 0%/100% pre-scaling, overlay polygon → text curve. **No mineral endpoint values printed.** |
| 7 | `box_plots.htm` | **Content.** Percentile box (default 25/75) vs standard-deviation box; statistics options. Contains a median/mean wording defect (§6). |
| 8 | `pie_charts.htm` | **Content.** 40-discrete-value limit incl. Null/Zero; array-curve summing; Max Data Levels truncation. **New page in 2025.** |
| 9 | `star_plots.htm` | **Content.** Auto-scale standardization ((x−mean)/SD), 30-facies limit, ±2 SD default range shading, log10 handling. |
| 10 | `rose-plots.htm` | **Content.** Circular histogram of angular data; Azimuth/Symmetrical/Strike; user-set bin count; Max Data Levels truncation. **New page in 2025.** |

### 1.4 Log plot group

| # | Page | Outcome |
|---|---|---|
| 11 | `logplot_log_plot_styles.htm` | **Content.** All curve styles: numeric, variable shading, VDL, tadpole, waveform (+histogram-of-waveform normalization), picture, dip image, annotation, synthetic seismogram, mini-plots, curve rose, segmented/histogram cross-section, pie, well diagram, array plot, borehole profile, image-tool profile, tick. Curve-count limits and array-averaging rules. |
| 12 | `logplot_edit_the_log_plot_format.htm` | **Content.** Track/curve/grid/shading setup; shading opacity and zone-fill transparency defaults; array high-frequency shading rule. |
| 13 | `plotting.htm` | **Content.** Depth-shift and splice output-set naming defaults; re-sampling rule when input sets differ in step. |
| 14 | `plotoutput.htm` | **Content.** Output targets; **default grid-line spacing per vertical scale**; grid/border widths and colours; PDF page-width limits. |
| 15 | `plottoprinter.htm` | **Content.** **Default plot scale 1/500** and its default vertical grid spacing (independent confirmation of #14). |
| 16 | `logplot_log_plot_track_management.htm` | **Content (light).** Depth-tick synchronization to the Depth Grid Datum; ticks off by default on depth curves; zone-name font 'Automatic'. |
| 17 | `logplot_navigating_the_log_plot_window.htm` | **Content (light).** Vertical-scale selector, Lock = set default plot for well, Value Tips defaults. Rest UI-only. |
| 18 | `logplot_save_default_plot_format.htm` | **Content (light).** Default-plot resolution order and the `intpetro36` local-appdata store. |
| 19 | `logplot_track-templates.htm` | **Content (light).** `*.trk` templates; three-tier search (Working Folder `Default Plots` → Project/Well → Corporate Search Folders). |
| 20 | `logplot_pinning-a-logoplot.htm` | **UI-only** + one default: Pin default state = off; never persisted across sessions. |
| 21 | `plotrangeeditor.htm` | **Content (light).** Named plot ranges (depth range + vertical scale), description max 20 chars. Referenced as the fallback source of default depths by the histogram and crossplot multi-well grids. |
| 22 | `edit-plot-limits.htm` | **Content (light).** Default plot limits = top/bottom of well; applied to all new plots. |
| 23 | `recalculateplotlimits.htm` | **Content (light).** Recomputes per-curve plot limits; only curves whose values changed significantly move; additional curves are lost from the plot. |
| 24 | `editplotannotationsandlogtitle.htm` | **UI-only** + defaults: annotation/title fonts saved **per well**; `T` in the depth column selects the title font. |
| 25 | `plot_header_editor.htm` | **UI-only** + `.hdr` saved to the `PlotHeaders` sub-folder of the Well Folder; templates required. |
| 26 | `plotaheader.htm` | **Content (light).** Logo constrained to a 1.0-inch-high rectangle, aspect ratio preserved; remarks up to 5,000 characters. |
| 27 | `expanded_plot_mode.htm` | **UI-only.** Depth track + one track full width; required for interactive curve editing. |
| 28 | `montagebuilder.htm` | **Content (light).** Standalone montage app; `.ipm` files; **rescaling a plot graphic in the montage changes its hardcopy vertical scale** (scale-integrity warning). |
| 29 | `multi-well_batch_plotting.htm` | **Content (light).** Batch logplot output; one shared `.plt` format or each well's default; initial depths = each well's Default Range from the Plot Range Editor. |
| 30 | `plot_composer.htm` | **Content (light).** Composite-plot assembly; `CompositePlots` sub-directory per well; image layout defaults (centre, neither shrunk nor stretched). |
| 31 | `log_plots.htm` (hub) | **Content (light).** Production-logging "Launch Log Plot" hub; overlays curves from each well condition, colour-coded by condition. |

### 1.5 Autoplot / production-logging group

| # | Page | Outcome |
|---|---|---|
| 32 | `autoplot.htm` | **Content (light).** 16-track PL autoplot (depth, optional well sketch, up to 11 general, Vapp, Q, Q..I). Scales taken from min/max of all curves in a track, **rounded**. |
| 33 | `autoplot2.htm` | **Index page only** — lists the Autoplot sub-topics. |
| 34 | `rebuild-auto-plot.htm` | **UI-only.** Autoplot version saved per well condition; `Rebuild Current Plot`. |
| 35 | `plotdata.htm` | **Stub (305 chars).** Autoplot/Sensor Plot always show the chosen well condition. |

### 1.6 3D / geomechanics / mapping group

| # | Page | Outcome |
|---|---|---|
| 36 | `3d_petrophysics.htm` | **Content.** Multi-well interpretation + parameter distribution. **No gridding/interpolation** (§5 note). Display limit 50 wells. |
| 37 | `sandpit3d.htm` | **Content.** Sanding-prediction workflow and its four input chains; computes **Load Factor (LF)**. Equations are *not* on this page. |
| 38 | `sandpit_3d_settings.htm` | **Content (light).** Auto-populate curve names default = unchecked; per-curve unit override and unit-mismatch warning. |
| 39 | `wellmap.htm` | **Content (light).** Default position Lat 0 / Long 0 (equator); zoom range 500 km → 10 m; well-path requires TVD module or EDIST/NDIST. |
| 40 | `ic-mapping.htm` | **Content (light).** Map canvas, furniture, resources (shapefiles, surfaces, faults, polygons, well queries). No gridding parameters. |
| 41 | `mapping-resources.htm` | **Vendor placeholder.** Body is literally "Enter topic text here." (128 chars) — an unwritten page in the shipped 2025 manual. |

### 1.7 Correlation / well-diagram / misc

| # | Page | Outcome |
|---|---|---|
| 42 | `multiwellcorrelationview.htm` | **Content.** Display limits (10 views × 50 wells, 250 overall), value-tip decimals, per-well width/depth defaults, track-border default. |
| 43 | `well_diagram_manager.htm` | **Content (light).** Units default inches; thickness/metal-loss palettes; default 3D slice 0°→180°. |
| 44 | `well_diagram_cross_section.htm` | **Content (light).** Radial + circumferential (degrees) scales; automatic outer radius from max tubular OD. |
| 45 | `picturecorephotodata.htm` | **Content.** **Full RGB→Intensity/Saturation/Hue/Grey equation set** (§4.5). 10 picture curves/well; slice-angle default 180°; down-sampling on load. |
| 46 | `textcurves.htm` | **Content (light).** Numeric vs text curve creation; lithology descriptions resolved against `Tools > Edit Default Lithology`. |

**Rule-8 check (smectite / montmorillonite):** searched all 46 pages — **no occurrence** of either term. No clay endpoints are printed anywhere in the plotting stack.

---

## 2. Crossplot specifications

### 2.1 PICKETT — Standalone Pickett Plot (**highest-priority tasking (a)**)

#### 2.1.1 Axis assignment — confirmed by image

`[img-read: _cpclip0068.png]` — the standalone Pickett window titled `LLD / PHIE Xplot – Experior {stand alone pickett plot}`:

| Axis | Quantity | Observed scale in the shipped example |
|---|---|---|
| **X** | **Resistivity** — axis labelled `LLD - OHMM` | 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000 (log) |
| **Y** | **Porosity** — `PHIE` (plot title `LLD / PHIE`) | 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1 (log) |

So IP's Pickett plot is **X = Rt (log), Y = φ (log)** — the classical Pickett orientation. Constant-Sw lines run **down-to-the-right** (negative slope in log–log space); in the image the labelled lines 0.2 / 0.3 / 0.5 increase in Sw going *downward*, i.e. toward lower porosity at fixed Rt. `[img-read: _cpclip0068.png]`

Status bar of the same window reports the four live parameters: `m exponent : 1.99`, `a factor : 1`, `n exponent : 2`, `Rw Form Temp : 0.09…` (value truncated in the raster). `[img-read: _cpclip0068.png]`

#### 2.1.2 Hard constraints on the axes

> Both the X and the Y Axis MUST be defined as a LOG SCALE and have a value GREATER THAN ZERO. (`crossplottypes.htm`)

> whatever porosity curve is used it MUST have its Curve Type set to Porosity (`crossplottypes.htm`)

Additional, from the launcher page: "The relevant porosity and resistivity curves, with logarithmic scales, have to be selected before the Crossplot will launch." (`createacrossplot.htm`)

For log-scale axes generally, the gridline count must be set explicitly: "The Vert. Lines / Hori. Lines Number text entry box should be set to **10** to display the correct Logarithmic decade grid lines." (`createacrossplot.htm`)

#### 2.1.3 Parameter defaults — image-read from the shipped dialog

`[img-read: _cpclip0067.png]` — dialog `Sw Pickett plot lines - 10/0002 - DPTH`:

| Field | Shipped value |
|---|---|
| Water Saturation line values | **0.2, 0.3, 0.5** (5 boxes; 2 left blank) |
| `Rw` | **0.1** |
| `m` | **2** |
| `n` | **2** |
| `a` | **1** |
| `Draw lines to edge of plot area` | unchecked |

The dialog carries **padlock toggles on `Rw` and `m` only**. Purpose stated in prose: "The Lock feature allows the Rw or m values to be fixed whilst the interactive line on the Pickett plot is adjusted. Locking one value will allow only the other value to be adjusted and can be used to prevent any rounding errors occurring as the line is moved." (`crossplottypes.htm`) — i.e. IP treats the interactive line as a **two-degree-of-freedom object in (Rw, m)**, which is exactly the Pickett line's (intercept, slope) pair. `n` and `a` are not lockable because they do not participate in positioning the wet line.

The Sw line labels observed on the plot raster (0.2 / 0.3 / 0.5) match the dialog defaults exactly — cross-check passes.

#### 2.1.4 Rw / temperature semantics

> The Rw values calculated and displayed in the Sw Pickett plot lines dialog or in the task bar of the Crossplot are **at the formation temperature**. NOTE: this temperature is the **average formation temperature in the zone being worked on** and is from the inputted temperature curve. (`crossplottypes.htm`)

> The values of Rw, m, n and a **are not fed through to any other IP module** from the Standalone Pickett Plot. (`crossplottypes.htm`)

That last sentence is a hard isolation contract: the *standalone* Pickett plot is a scratchpad, not a parameter source. (The interactive Pickett launched from inside Porosity & Water Saturation is a different object and does write back — that path is on agent B's pages.)

#### 2.1.5 Explicit line equation — **NOT PRESENT**

The 2025 plotting pages **never print the Pickett line equation**. There is no statement of slope = −m, no intercept = a·Rw, no `Rt = a·Rw·φ^(−m)·Sw^(−n)` on any of the 46 pages. The relationship is only *implied* by (i) which four parameters the dialog accepts, (ii) which two are lockable, and (iii) the plotted geometry. Logged as **OPEN-L-01**. Anyone building the Pickett module must take the equation from agent B's Sw pages, not from the plotting documentation.

#### 2.1.6 Launch paths and default format

- Checkbox `Standalone Pickett Crossplot` in the Crossplot Options panel of the Scales tab. `[img-read: _cpclip0064.png]` (`createacrossplot.htm`)
- Or: `Load Format → Project Defaults Crossplot → saturation → Pickett` (`crossplottypes.htm`)
- The Sw-lines dialog is re-openable from the crossplot context menu → `Sw lines for Pickett plot`; undo/redo becomes available once a parameter line has been adjusted. (`crossplottypes.htm`)
- Rw is dragged on the plot with the **right** mouse button held over the red line; the line's **angle** (i.e. `m`) is changed by dragging the square grab boxes at the line ends. `[img-read: _cpclip0068.png]`

### 2.2 HINGLE — plotting-side statement (**tasking (a), D-02**)

#### 2.2.1 Absent from the plotting pages

**Hingle does not appear on any of my 46 pages.** A case-insensitive search of every assigned `_text.txt` and `.htm` returns zero hits. The Crossplot module (`crossplottypes.htm`, `createacrossplot.htm`, `crossplot_functions.htm`, `crossplotsfeatures.htm`) offers Pickett as a standalone crossplot type but **never offers a Hingle crossplot type**. In IP, Hingle lives only inside the interpretation modules (Basic Log Analysis and Porosity & Water Saturation), reached by right-clicking the resistivity track.

The eight pages carrying `Hingle` are: `basicloganalysis`, `mineral_solver`, `minsolveeqandmeth`, `nmrinterpretation`, `plot_the_mineral_solver_result`, `porosityandwatersaturation`, `swequationsandmethodology`, `swplot` — all outside my assignment. I read the axis-defining passages read-only to answer the tasking; they are quoted below with their own provenance so agent B can reconcile.

#### 2.2.2 The exact 2025 statement — and it is self-contradictory *within one paragraph*

Verbatim, the prose sentence (identical in `swplot.htm` and `basicloganalysis.htm`):

> "The Hingle plot Y-axis is built from (1/Rt)^(-1/m) but scaled in resistivity." (`swplot.htm`, `basicloganalysis.htm`)

Two sentences later, **the same paragraph** says:

> "If you show the Format, then the Y scale value are the values actually in the Y curve, ie Rt^(-1/m) not Rt." (`swplot.htm`, `basicloganalysis.htm`)

These are not the same quantity. `(1/Rt)^(−1/m) = Rt^(+1/m)`, whereas `Rt^(−1/m) = (1/Rt)^(+1/m)`. They are reciprocals of each other.

#### 2.2.3 The tie-breaker: the actual computed curve

The curve IP actually creates and plots is defined three separate times, and all three agree with the **second** form:

> `Rt_Hingle = Rt^(-1/m)` (`basicloganalysis.htm`, output-curve equation section)

> "Rt_Hingle: Used for the Interactive Hingle Plot. Calculated as `Rt^(-1/m)`" (`porosityandwatersaturation.htm`)

> "Rxo_Hingle: Used for the Interactive Hingle Plot. Calculated as `Rxo^(-1/m)`" (`porosityandwatersaturation.htm`)

> "The Rt_Hingle curve is needed for the Hingle plot, and is automatically calculated by the module, but is not shown in the output curves." (`basicloganalysis.htm`)

**Plotting-side verdict on D-02:** the implemented Hingle Y-axis is **`Rt^(−1/m)`, equivalently `1/Rt^(1/m)`** — three independent statements including the executable curve definition. The phrase `(1/Rt)^(-1/m)` is a **vendor double-negation typo**; the exponent sign is wrong for the reciprocal form it was trying to write (it should read `(1/Rt)^(+1/m)`). Score: 3 statements to 1, with the dissenting statement being descriptive prose and the majority being the curve IP computes.

This resolves D-02 **in favour of `1/Rt^(1/m)`**. Confidence: high — but flagged rather than closed, because it rests on my reading of pages assigned to agent B. Agent B should confirm from the Sw side.

**Version check:** the contradiction is **byte-identical in IP2018 and IP2025**. Both the `(1/Rt)^(-1/m)` sentence and the `Rt_Hingle = Rt^(-1/m)` definition are unchanged between versions. So D-02 is a **persistent vendor documentation defect, not a 2025 regression** — it has shipped uncorrected for at least seven years.

#### 2.2.4 Remaining Hingle axis facts (plotting-relevant)

- **X axis:** "The X axis is Total or Effective porosity" (`swplot.htm`); "The X axis is Porosity" (`basicloganalysis.htm`).
- **Y axis is *labelled* in resistivity but *holds* the transform** — "built from … but scaled in resistivity". The tick labels show Rt values; the underlying curve values are the transform. This is the trap: reading the Format dialog gives transform-space numbers, not ohm-m.
- **The Y gridlines are m-dependent and auto-generated:** "the Y axis scale lines are unique to the Hingle plot, and are generated automatically. The line position depends on the 'm' parameter, so if you change 'm' the lines move." (`swplot.htm`) — i.e. changing `m` re-renders the axis furniture, not just the data.
- Sw lines are configured from `Function → Sw Lines for Hingle Plot`, adjusting Rw, a, m, n — "This is similar to the Pickett plot." (`swplot.htm`)
- The info window bottom-left shows Rw and Salinity from the **Waters tab of the Phi/Sw Parameters table** (`swplot.htm`) or the **Water Saturation tab of the Basic Log Analysis Parameters table** (`basicloganalysis.htm`).
- An `Rxo_Hingle` companion exists for the flushed zone, same transform on Rxo (`porosityandwatersaturation.htm`).

### 2.3 Standalone crossplot types exposed in the UI

`[img-read: _cpclip0064.png]` — the `Cross Plot Options` panel of the Scales tab ships **six** checkboxes:

| Option | Documented in my page set? |
|---|---|
| Expand Array Curves | Yes (`createacrossplot.htm`) |
| Frequency Crossplot | Yes (`crossplot_functions.htm`) |
| Pressure Gradients | Yes (`crossplot_functions.htm`) |
| Standalone Pickett Crossplot | Yes (`crossplottypes.htm`) |
| **Standalone Rv/Rh Butterfly** | **No — not described on any of my 46 pages** |
| **Standalone Thomas Stieber** | **No — not described on any of my 46 pages** |

The prose type list omits both: "Crossplot Type (Normal, Frequency, Pressure Gradients, Standalone Pickett)" (`createacrossplot.htm`). Specifications for Butterfly and Thomas-Stieber live on `laminated_sands_workflow.htm`, `laminatedfluidsubs.htm`, `swparameters.htm`, `swplot.htm`, `porosityandwatersaturation.htm`, `swequationsandmethodology.htm` — outside my assignment. **Cross-agent flag** (§8, §9): these are directly relevant to thin-bed work.

### 2.4 3D Crossplot

- Enabled by the `3D` checkbox on the crossplot header; requires a Z axis defined in the setup. (`crossplottypes.htm`)
- Rotation by scroll bars or left-drag; axis panning by right-drag.
- **Panning does not un-clip data:** "this does not add data to the Crossplot that might have got clipped due to the original scale values. It is best to go back into Edit Format and change the original Scale values." (`crossplottypes.htm`)
- Overlay-line Z depth is adjustable via its own scroll bar.
- **"No interactive functions are available for the 3D Crossplot."** Return to 2D to interact. (`crossplottypes.htm`)

### 2.5 Frequency Crossplot — binning and symbol encoding

(`crossplot_functions.htm`)

| Element | Specification |
|---|---|
| Bin grid | Set by `Number of Columns` and `Number of Rows`, expressed in input-curve value units |
| `Drop Value` | Minimum bin frequency that will be displayed. 1 ⇒ every bin with ≥1 point shows; 4 ⇒ only bins with ≥4 points |
| Symbol encoding | **1–9 points → digits `1`–`9`; 10–35 points → letters `a`–`z`; >35 points → `#`** |
| `Frequency` divisor | Divides bin counts before symbol assignment. Divisor 2 ⇒ bins up to 70 map onto `a`–`z`. Suggested 50 for multi-well (discriminates up to 1,750 points) |
| Colour-cell mode | `Use Spectrum` or `Use Palette`, slider controls colour count |
| **Bin-edge rule** | "if a Crossplot data point lies on the border line of a data bin then the point is included in the Frequency count for the **next highest** data bin, because the data point value is counted as **'> or equal to' the bin low value**" |

That bin-edge rule is the canonical IP convention: **bins are left-closed, right-open — `[low, high)`**. It is stated only here but applies wherever IP bins.

### 2.6 Pressure-Gradient crossplot

(`crossplot_functions.htm`)

- **Axes are prescribed:** X = pressure (units **psi or bar**); Y = `DEPTH`, `TVD` or `TVDSS` "depending on well deviation and TVD preference - KB or MSL".
- Regression types offered: `x = f(y)`, `x = RMA(y)`, **`x = Robust fit (y)` — explicitly "the default type"**, `y = f(x)`.
- `Fixed Grad` allows typing a gradient directly instead of regressing.
- **Up to 6 regression lines** can be calculated from digitized Areas.
- Reference lines are defined by a **gradient plus one (x,y) point the line passes through**.
- Line Intersections tab clips lines left/right to other lines or fixed values; labels show the intersection as a `(pressure, depth)` pair in parentheses — i.e. the fluid-contact readout.
- **Excess pressure:** "the difference between the formation pressure gradient and the pressure actually measured. Excess pressure is **positive if the measured pressure is greater than the gradient pressure** at the depth." Zero excess pressure is plotted at the **middle of the track**; rendered as triangles.
- Constraint: "The top and bottom depths for the formation gradient curves must be set to **not overlap**, ie there must be only one gradient curve defined for each depth. If they overlap an error message will be displayed."
- Gradient lines export to a logplot as **text formula curves, one per gradient line**; output set defaults to **`Gradients`**; pressure scales on the logplot are inherited from the crossplot X-axis.

### 2.7 Regression engine (all crossplot types)

(`crossplot_functions.htm`)

**Types:** Linear (`Y=f(x)`, `X=f(y)`), RMA, Robust, Poly 2nd/3rd/4th/5th, Exponential, Power. Multiple types may be displayed simultaneously.

**Method statements (vendor's own characterization, paraphrased):**
- OLS minimizes squared error distances: `Y=f(x)` minimizes squared Y-errors; `X=f(y)` minimizes squared X-errors. The variable being predicted goes on Y.
- **RMA** "gives an equation that is midway between the two OLS methods", estimates the theoretical relationship between error-free counterparts of X and Y, presumes equal error fractions in both, and **always reproduces the standard deviation of Y**. Recommended where scatter is caused by poor core-to-log depth pairing. Vendor notes `Y=f(x)` OLS **underestimates** the standard deviation of porosity when depth shifting is the dominant scatter source.
- **Robust** minimizes the sum of **unsquared** Y errors; free to choose gradient and intercept; **not forced through the mean or median** of the calibration data; reduces outlier influence.

**R² equation — image-read, fully unambiguous:**

`[img-read: embim2.png]`

```
        Σ (Yᵢ − Y_mean)²  −  Σ (Yᵢ − Ŷᵢ)²
R²  =  ─────────────────────────────────────
              Σ (Yᵢ − Y_mean)²
```

where (`crossplot_functions.htm`): `Yᵢ` = Y value at the i-th sample; `Y_mean` = mean of all Y samples; `Ŷᵢ` `[img-read: embim3.png]` = Y result at the i-th sample calculated from the regression equation.

**Other regression facts:**
- Point selection for the fit: All points / Areas only / Discrete Points only / both.
- `Force fit` (Linear only) forces the line through a typed (x,y).
- Discrete-point selection tolerance: **click within 2 pixels** of a plotted point.
- `Use Mean Point of Area` reduces an area to a single point using **Arithmetic, Geometric or Harmonic** mean, independently selectable for X and Y.
- Line width range **1–10**.
- Report default filename `Regression.txt`, written to the well sub-directory. Report format shown includes both `Y function of X` and `X function of Y` fits with their R².
- Equations can be pushed straight into the User Formula module via right-click → `Open Equation in User Formula`.
- **Vendor's cited authority:** *"Statistical Regression Line-Fitting in the Oil and Gas Industry"* by Richard (Dick) Woodhouse, 2002, PennWell Publishers. (`crossplot_functions.htm`)

### 2.8 Areas, discrete points and flag curves

(`crossplot_functions.htm`)

- **Up to 10 areas per crossplot**; minimum **3 points** to close a polygon.
- Flag-curve defaults: output curve name **`XpFlag`**; area 1 → value 1, area 2 → value 2, etc.
- **Overlap rule:** "If areas overlap, then points that fall in several areas will be assigned the value of the **highest area number**." (point in areas 2 and 4 → 4)
- `Delete before Write` nulls the output curve first; otherwise only in-area depths are modified.
- User lines: entered **without** the result term — `Y = 2.5*X − 3.5` is typed as `2.5*X - 3.5`; must use literal `X`/`Y`, not curve names. Saved as `.XUL` in the project directory. No limit on the number of user lines per session; unsaved lines are lost when the Crossplot module closes.

### 2.9 Multi-well and multi-curve crossplots

(`crossplottypes.htm`)

- Multi-Curve Crossplot: **up to 500 wells and 8 curves plus a Z-axis curve**; renders the full pairwise matrix with histograms on the diagonal, cluster means as filled circles.
- "Almost the same Crossplot functionality can be found within Cluster Analysis."
- `Show Lower diagonal plots only` exists purely to raise refresh rate on large selections.
- Z-axis colours pick up per-curve display defaults from **`CparmDef.xml`** when the mnemonic is known.
- Right-drag on a plot rescales: vertical drag → Y scale, horizontal drag → X scale. Left-drag box → zoom.
- Depth-copy across wells is validated: "IP verifies that the depths are contained within the depths specified in the original well. If the depths for the subsequent wells are out with the original well depths, the depths for those wells **will not be changed**." Blank top-line depth ⇒ reset to the **Plot Range Editor** default.

---

## 3. Histogram & normalization methods (**tasking (b)**)

### 3.1 Binning

(`histogram.htm`)

- `Number of divisions` = number of classes. Range on the **crossplot-embedded** histogram is **1 to 100** divisions (`createacrossplot.htm`); the standalone histogram page states no explicit cap.
- Histogram size (crossplot-embedded) range **100 to 1000** (`createacrossplot.htm`); applies to the axis showing the number of points.
- Vendor's worked example: GR scaled 0→150 with 50 divisions. **The class list printed by the vendor is arithmetically inconsistent** — see §6, D-L-03.
- Waveform-histogram binning (`logplot_log_plot_styles.htm`): "data that falls outside the scale range is **ignored**" — out-of-range data is dropped, not clamped, in that path. Contrast §4.1 Z-axis clipping, where out-of-range data is **clamped**.

### 3.2 Scale, log handling and the logarithmic mean

(`histogram.htm`)

- Left/Right Scale default to the values in the curve system defaults file **`CparmDef.xml`** where the curve is listed.
- Selecting `Logarithmic` **changes the mean statistic**: "If Logarithmic is selected, a **logarithmic mean** is calculated rather than a straightforward arithmetic mean. The column header reads `Mean(log)` to reflect this."

**Logarithmic mean equation, verbatim from the page:**

```
Mean(log) = 10^[ [ Log10(a(1)) + Log10(a(2)) + ..... Log10(a(n)) ] / n ]
```
(`histogram.htm`) — i.e. the geometric mean.

- **"Negative numbers are treated as NULL values when logarithmic is selected, and are not plotted on the histogram or used in the statistical calculations."** (`histogram.htm`) Note this is *negative*, not *non-positive*; the treatment of exact zero is unstated → **OPEN-L-04**.

### 3.3 Y-axis mode and area normalization

(`histogram.htm`)

- `Plot as Frequency` = actual point count per division; `Plot as Percent` = percentile of total points.
- `Y Axis Max` overrides "the IP default value, which is designed to maximize the histogram size in the window" — the default is auto-fit, no fixed number.
- **`Normalize each curve to 100%`** / **`Equal areas under curves?`** — rescales each histogram curve so equal areas appear under each. Intended for the case where one well has far fewer points than the others. **Only valid when `Plot Curves` is selected.**
- Display types: **Bar chart (default), cumulatively stacked** across wells/zones; `Plot Curves`; `Plot Transparent Bars` (unfilled, **not stacked**, explicitly "intended for a normalization option (normalizing using bar plots)").

### 3.4 Statistics computed

**On-screen / hardcopy statistics table** (`histogram.htm`): Minimum, Maximum, Mode, Mean (or `Mean(log)`), Standard Deviation, plus up to **three user-defined percentiles**.

**Written report** `HistoStats.txt` — `[img-read: _hsclip0041.png]`: header `HISTOGRAM STATISTICS`, date, well name, plot title; then one row per zone plus an `All Zones` row with columns **Minimum | Maximum | Mode | Mean | Std Dev**; then a two-column `Value / Total Cum %` table giving the cumulative-curve values. Note the written report as shipped carries **no percentile columns** and the header reads `Mean`, not `Mean(log)`.

**Statistical overlay lines** (`histogram.htm`): Gaussian Fit (thin dashed over bars/curves, bold solid when bars and curves are both off), ±1 Standard Deviation (green dashed), Minimum, Maximum, Mean, Mode, and Percentile 1/2/3 lines (solid blue over bars). "separate Gaussian fit lines for Multi-Well displays are only available when the `Plot Curves` option, not `Plot Bars`, is selected."

**Two mode caveats, both important:**
> "if there is no single mode value for the data then the Mode statistics cell will be blank." (`histogram.htm`)

> "the **Mode statistic is determined relative to the cumulative values displayed on the histogram**. To be clear, the Mode is **not derived directly from the underlying raw data**." (`histogram.htm`)

So IP's Mode is **bin-dependent** — change the division count and the Mode moves. Any SandiBumi mode must either replicate that or be documented as differing.

### 3.5 Percentile statistics — defaults

> "Each of the percentile statistic boxes are user definable. The **default values are 10th percentile, 50th percentile and 90th percentile**. Users can enter whatever value they like in the percentile statistic boxes i.e. P0.5, P90.5 etc." (`histogram.htm`)

Fractional percentiles are accepted. Each of the three has its own `Show Percentile n` checkbox. **Unchanged from IP2018** (verified by diff).

### 3.6 Curve normalization — full specification (**tasking (b) core**)

#### 3.6.1 The equation

> "The normalization is done by taking the input curve and applying the normalization equation to the values.
> **Result = Input x 'a' constant + 'b' constant.**" (`histogram.htm`)

Confirmed against the shipped UI panel `[img-read: _hsclip0064.png]`: a group box `Normalization Values` reading `Result = Input X [1.0] + [1.11524]` with an `Interactive` button — the multiplier box holds `1.0` and the offset box the fitted shift, exactly matching a 1-point linear shift.

#### 3.6.2 The three normalization types

Verbatim (`histogram.htm`):

| # | Type | Effect on constants |
|---|---|---|
| 1 | **1 point linear shift** | "This changes the **'b'** constant. **'a' constant is set to 1**." |
| 2 | **1 point Scale Factor** | "This changes the **'a'** constant. **'b' constant is set to 0**." |
| 3 | **2 Point** | "This changes the **'a' and 'b'** constants." |

#### 3.6.3 The two methodologies

> "There are two normalization methodologies:
> · Normalize manually dragging the curve to match the reference curve.
> · Normalize using fixed percentiles.
> **The first option is active by default.** The second option is activated by checking the 'Normalize to Fixed Percentiles' box." (`histogram.htm`)

**Manual/interactive mechanics:**
- 1-point linear shift: drag anywhere; the curve translates.
- 1-point scale factor: drag anywhere; the curve **stretches**.
- 2-point: click to plant a **pin** (vertical red line). Dragging the red line shifts linearly (changes `b`); clicking away from the line stretches about it. **The value at the pin is preserved.** Moving the pin itself does not change the amount of stretch already applied.
- `Show un-normalized curve` / `Show Original curve` draws the pre-normalization curve as a dotted outline.

**Fixed-percentile mechanics:**
- `Lower %` and `Upper %` entry boxes + `Calculate`. "If one point normalizing is selected then only the 'Lower %' box needs to be filled in."
- `[img-read: _hsclip0063.png]` — **both the `Lower %` and `Upper %` boxes ship EMPTY. IP supplies no default normalization percentiles.** This is a genuine absence, not an extraction failure. Do not confuse with the P10/P50/P90 *statistics* defaults in §3.5 — different feature, different defaults. Logged as **OPEN-L-05**.
- `Run All` (active only in fixed-percentile mode) applies the same percentiles across every well in one pass.
- `Next` / `Previous` run the current normalization (fixed-percentile mode only, equivalent to `Calculate`) then advance through the well list.

#### 3.6.4 Reference-curve requirement

> "a Reference Curve is only necessary if you are going to use the **Normalize to fixed percentiles** option. For other normalization types the Reference Curve entry box can be left blank." (`histogram.htm`)

#### 3.6.5 Depth-interval semantics — calculate-where vs apply-where

This is the part most likely to be got wrong, and IP separates the two explicitly (`histogram.htm`):

- **Interval Depths mode:** the data on the histogram comes from the Input Depths in the Select Wells and Curves form (defaults to the entire well, but restrictable). "This is the data which will be the source of the normalisation A and M values… and it is the data which will be input into the percentile calculations if that option is selected."
- Once determined, the factors "can be applied to either the **same Input Depths** as the values were calculated from above, or they can be applied to the **Whole Well**, if that is different."
- **Zonal mode:** factors are calculated from the zones **highlighted** (solid colour) and applied to the zones **checked**. "This need not be the same zones which are highlighted for display."
- Application scope options: **Whole well** / **Checked Zones** / **Selected Depths**.
- `Use Discriminators`: "the normalization will only take place over the intervals where the discriminators are true. I.e. over the data which is visible."

#### 3.6.6 Multi-well normalization defaults and workflow

- **`Only display reference and normalized Histograms` — "(defaults to on)"** / "(default)". Stated twice, in `histogram.htm` and `createacrossplot.htm`. It suppresses the display of non-participating curves but "does not force the display of curves and zones".
- Recommended practice from the vendor: "it is best to select all wells and zones while normalizing."
- `Group Zones into one histogram using cumulative sets` — collapses each input well to a single curve; **the well name is used as the cumulative set name**. Without it, a multi-zone multi-well normalization is "practically unusable unless you only select one zone".
- A composite reference is built by giving several wells/zones the **same cumulative set name** (vendor example: `norm ref`). A combined cumulative curve **cannot be selected as the curve to normalize** — "it makes no logical sense".
- `Default extension for result curve` may be blank **only if** the input and output sets differ.
- **Provenance mechanism:** "The current normalization coefficients are looked up in the result curve, if it exists (**coefficients are written into the curves comments field**)." (`histogram.htm`) — IP round-trips a,b through the curve comment.
- Bar-mode normalization: reference curve solid-shaded, curve being shifted drawn as transparent bars.
- Percentile lines, if turned on, **drag with the curve** during interactive normalization.
- On closing the normalization window the user is prompted for a report filename; existing files may be overwritten or appended.

#### 3.6.7 Normalization from within a crossplot

(`createacrossplot.htm`) The same normalization functions are available on crossplot-embedded histograms via right-click → `Normalize curves`. To normalize both axes, open **two** Curve Normalization windows — "Each window is independent in the sense you can select different normalization types. However they are synchronized in that they always display the same reference and normalized well." With `Only display reference and normalized Histograms` on, "the crossplot will show the **normalized** data, not the raw data" and updates after each shift.

#### 3.6.8 Vendor's cited authority

> *Well Log Normalization: Methods and Guidelines* — **Daniel E. Shier, Petrophysics, Vol. 45, No. 3 (2004)**. (`histogram.htm`)

Definition offered: "Normalization is a mathematical process that adjusts for differences among data from varying sources, in order to create a common basis for comparison."

### 3.7 Histogram display / output defaults

(`histogram.htm` unless noted)

| Setting | Default |
|---|---|
| Font size | **12** (range 6–20) |
| Line thickness (cumulative curve) | **1** (range 1–12) |
| Multi-well curve line width | **1** (range 1–12) |
| `Display Colored Background` (Options tab) | **cleared** |
| Histogram Type | **Bar chart, cumulative** |
| `Multiple Active Zones` | **cleared** (Single Active Zone mode is default) |
| `Select When Active` | **selected** |
| Pin button | **off**; never persisted across sessions |
| Discriminators | up to **10 sets**, up to **6 criteria per set**, AND/OR logic |
| Format file | `.hst` |
| Report file | `HistoStats.txt` |
| Graphics formats | EMF, CGM, GIF, TIFF, PNG, JPEG, BMP; clipboard as `.emf` |
| `Histograms on Crossplot (default)` | configurable at `Tools > Options > Miscellaneous Options` (`crossplot_functions.htm`) |

Colored background and the statistics table are globally toggled at `Tools → Options → General → Plotting → Colored Background` / `Histogram Statistics Table`.

---

## 4. Equations & defaults elsewhere in the stack

### 4.1 Crossplot limits, defaults and Z-axis binning (**tasking (f)**)

(`createacrossplot.htm` unless noted)

| Item | Value |
|---|---|
| **Max Z1 colours** | **20** |
| **Max Z2 symbols** | **35** |
| Palette auto-populate divisions | max **20** |
| Point size | range **1–5** |
| Variable point size (Z2 mode) | range **1–50** |
| Z2 display modes | Alphanumeric / Variable Point Size / Transparency |
| Log-axis gridline count | **set to 10** for correct decade gridlines |
| Discriminator sets | max **10**, up to **6** discriminators each |
| Regression discriminators | up to **6**; point size max 5, min 1 |
| Areas per crossplot | max **10** (`crossplot_functions.htm`) |
| Font size | **default 12**, range 6–20 |
| Print formats | Standard / Side-by-Side |
| Format file | `.xpt`; colour scheme file `.zcl` |

**Z-axis clipping convention (important, and opposite to the waveform-histogram rule):**
> "Z Axes Clipping: Enables you to decide whether Z axis curve data values that fall outside the defined scales are to be included on the Crossplot. **When the boxes are not selected the data that are outside the defined scale range will be plotted with the colour or symbol of the maximum or minimum value of the scale range.**" (`createacrossplot.htm`)

⇒ Default (unchecked) behaviour is **clamp to the end colour/symbol**, not drop.

**`Fill From Z1 Settings`:** takes the Z1 Min and Max from the Scales tab and divides the colour bar by the Number of Z1 divisions to generate the bar and its labels. This is IP's Z-axis binning rule: **equal-width bins spanning [Min, Max]**, count = the divisions number.

**Data decimation:**
> "**Display every _n_ th point**: Enables you to filter the amount of data being displayed on the Crossplot. Displaying every **1**th point displays all the data. Displaying every 5th point will display the 1st, 6th, 11th, 16th... etc data points. To change the start point of the data to be filtered alter the top depth to be displayed." (`createacrossplot.htm`)

⇒ Decimation default is **1 (no decimation)**; it is a *stride-from-the-top* filter with **no averaging**, and the phase is controlled only by moving the top depth.

**Array-curve depth averaging:**
> "If the box is selected, the array curve will be expanded so that every point in the array curve has a separate point in the Crossplot. If the box is left cleared, then **the array curve data will be averaged to the well depth step increment**, and the average array curve value will be plotted." (`createacrossplot.htm`)

**Histograms react to discriminators:** "the histograms are updated accordingly so that only the data which meets the discriminator criteria is plotted", and the excluded-point count is reported beneath the crossplot.

**Default crossplot format library** (`createacrossplot.htm`, `crossplotsfeatures.htm`): stored in the IP directory (`C:\Program Files\IntPetro3x`) under `Default Plots`, subdivided into six functional groups — **`core`, `fluid`, `lithology`, `parameter`, `porosity`, `saturation`**. Custom hierarchies are created by making a `NAME.xpt` subdirectory in the project directory; ordering and search paths are governed by `Corporate Search Paths` under `Tools → Options`.

**Generic curve pickup:** typing `@density` picks up "the final (if flagged), or most recently loaded / updated density curve"; `*sonic` picks up "the default sonic curve". (`createacrossplot.htm`, `crossplotsfeatures.htm`)

### 4.2 Log plot vertical scale and grid-line spacing (**tasking (f)**)

Two independent statements, mutually consistent:

> "a Scale of 1:500 has Major, Minor and Lite grid lines Plotted at **50m / 100ft, 25m / 50ft, 5m / 10ft** respectively." (`plotoutput.htm`)

> "**The default plot scale is 1/500.**" … "For the default plot scale of 1/500 the default vertical grid line spacing is: **Heavy every 100' (50m), medium every 50' (25m) and light every 10' (5m)**." (`plottoprinter.htm`)

Only the 1:500 row is printed. Other scales are computed, not tabulated: "For different plot scales, a different vertical grid line spacing is computed to produce a nice looking plot" (`plottoprinter.htm`) — algorithm unstated → **OPEN-L-06**.

Also (`plottoprinter.htm`): the plot scale of any subsequent plot defaults to the **previous plot's** scale; the fit-to-one-page scale is computed only for plots **without a header**; header/title space is independent of log scale.

### 4.3 Log plot output defaults

(`plotoutput.htm`)

| Setting | Default |
|---|---|
| Grid line widths (Heavy, Medium, Light) | **3, 2, 1** |
| Track border width | **3** |
| Grid and border line colour | **DarkGray** |
| Plot background colour | **white** |
| Plot border width | 0 ⇒ no border printed |
| PDF page width cap | **200"** default; **500"** with `Use Large PDF Page Size`; output is **cropped** beyond |
| Footer title | defaults to the Plot Header Title if entered |
| Default zone shown (X-key curve selection) | first available |

Absolute plot calibration is per-printer via `Plotter Calibration.opt` in the IP directory — a printer that renders a log 3% too long can be corrected there.

(`logplot_edit_the_log_plot_format.htm`): shading **opacity default 100** (fully opaque, lower = more transparent); zone-fill background **transparency default 75%**, fill colour = zone colour; `Auto Update Shading` **default On**; shading bitmaps must be **16×16 pixels**.

(`logplot_log_plot_track_management.htm`): depth ticks are **off by default** where a Depth curve is displayed; zone-name font default `Automatic` (auto re-size/re-orient).

(`multiwellcorrelationview.htm`): value-tip decimal places **default 2**; `Change Track Border Width All Wells` **default 2**; per-well Top/Bot depth defaults = well top/bottom.

### 4.4 Log plot capacity limits

| Limit | Value | Source |
|---|---|---|
| Curves per well | **500** | `logplot_log_plot_styles.htm` |
| Conventional curves per waveform | **400** (leaving 100 spare of the 500) | `logplot_log_plot_styles.htm` |
| Picture curves per well | **10** | `picturecorephotodata.htm` |
| Multi-well correlation views open at once | **10** | `multiwellcorrelationview.htm` |
| Wells per correlation view | **50** | `multiwellcorrelationview.htm` |
| Overall individual well plots | **250** | `multiwellcorrelationview.htm` |
| Wells displayable in 3D Petrophysics plot | **50** | `3d_petrophysics.htm` |
| Wells / curves in Multi-Curve Crossplot | **500 / 8 (+Z)** | `crossplottypes.htm` |
| Discrete values in a Pie Chart | **40** incl. Null and Zero | `pie_charts.htm` |
| Discrete facies values in a Star Plot | **30** | `star_plots.htm` |
| Lines per overlay file | **30** | `crossplot_functions.htm` |
| Points per overlay line | **20** | `crossplot_functions.htm` |
| Data labels per overlay | **30** | `crossplot_functions.htm` |
| Plot-range description | **20 characters** | `plotrangeeditor.htm` |
| Header remarks | **5,000 characters** | `plotaheader.htm` |

### 4.5 Core-photo / image curve equations (`picturecorephotodata.htm`)

All transcribed from plain page text (not rasters); verbatim.

```
Intensity  = Max(red, green, blue)

MaxRGB     = Max(red, green, blue)
MinRGB     = Min(red, green, blue)
Saturation = (MaxRGB - MinRGB) / MaxRGB

Hue        = Arctan( Sqrt(3) * (green - blue) / ((2 * red) - green - blue) )
```

Grey-scale, three methods, **Luminosity is the default**:

```
Average     Grey = (red + green + blue) / 3
Lightness   Grey = (MaxRGB + MaxRGB) / 2        ← as printed; see D-L-02
Luminosity  Grey = Red * 0.21 + Green * 0.72 + Blue * 0.07
```

Image inversion: `R = 1 – R, G = 1 – G, B = 1 – B`.

Other defaults: RGB component curves are extracted **by default**; slice-curve angle range **0–360** with **default 180° (the centre line)**, vendor warns against angles near the edges ("could be reading core box values or rubble"); slice curves are named with the angle appended (`Red180`); down-sampling on load via `Load Samples per Depth Unit`; depth-shift curves named `DepOff$_££`, **default `DepOff1_ds`**, and are **overwritten on each interactive depth-shift run**.

### 4.6 Statistical-plot definitions

**Box plots** (`box_plots.htm`):
- **`Percentile Box` is the default plot option, with 25% and 75% values selected.** Top and bottom of the box are the entered percentiles.
- `Standard deviation box`: top = **mean + 1 SD**, bottom = **mean − 1 SD**.
- `Display Standard Deviation` (mean ±1 SD as red dotted lines) is **only available for the Percentile Box plot**.
- Grouping: by Facies curve, by Zone, by Well, or Single data set.
- Statistics export to space-delimited `.txt` or comma-delimited `.csv`.
- See D-L-04 for the median/mean wording defect.

**Star plots** (`star_plots.htm`):
- Left scale value = centre of the star; right scale value = the star-point tip.
- **`Auto Scale` algorithm, verbatim:** "the scales are calculated from the data automatically. The data for each curve is **normalized by subtracting its mean and dividing by its standard deviation**. The maximum and minimum of the standardized data of all curves is then used to calculate the scales. **The number of standard deviations to display on the plot is also taken into account so that all error bars are always shown.**" This is the same scaling SOM and Cluster Analysis use.
- **`Display Data range on plots` default range = mean ± 2 standard deviations** (count is user-changeable).
- Log-flagged input curves are marked with a **red asterisk** and it is **log base 10** of the curve that is displayed.
- Max **30** discrete facies values; grouping by facies curve / zones / well / single group; zone grouping matches **by zone name** across wells.
- The `Spectral Plot` is "an unwrapped star plot [containing] the same exact information".

**Pie charts** (`pie_charts.htm`):
- **Zero and Null are normally ignored** when plotting; each can be opted back in with its own colour.
- Array curve input: **each array element is summed over the defined depth interval** and becomes one segment.
- Text curves matched against the Description field of the Default Lithology table get the corresponding **lithology fill pattern**; otherwise a solid colour.
- Doughnut hole size is driven by the `Outer` scale value.

**Rose plots** (`rose-plots.htm`): "essentially a **circular histogram of angular data**." Styles: Azimuth / Symmetrical / Strike. Bin count user-set. `Outer` scale settable to Automatic (covers max data extent). File `.rpc`.

**Max Data Levels truncation (both pie and rose, tasking (f)):**
> "If the number of samples in the Depth Interval is greater than the `Max Data Levels`, then **only the first `Max Data Levels` samples are used, starting from the top down**." (`pie_charts.htm`, `rose-plots.htm`)

This is **truncation, not decimation** — the bottom of the interval is silently discarded rather than subsampled. Default value of `Max Data Levels` is not stated → **OPEN-L-07**.

### 4.7 Ternary plot algorithm (**tasking (d)**)

(`ternary.htm`)

**Implicit unity equation:** "The data is **always normalised at each depth level** so what is being plotted is the relative proportions of each of the 3 inputs; ie there is an implicit Unity equation where **A + B + C = 1**."

**Per-depth-sample algorithm, verbatim:**
> "· Uses the A/B/C curve 0-100% scale values to calculate a **pre-scaled value** per curve data point, **or to flag a depth sample as an outlier that won't be plotted**.
> · Calculates the **normalised values** for the A/B/C curve values relative to each other, **based on the sum of A+B+C**
> · The normalised values are used to plot the position on the chart."

**Apex scale endpoints:** the 0% and 100% values "should be the curve value that you wish to set for each of the plot Apices. These values are **read from the curve header scale information**" and a `Default` button restores them. "It is rarely the case the 0 and 100% values should be set to anything other than **0 and 1**. These are essentially used to **pre-scale the input curves before the normalisation occurs**."

**Vendor's worked example (reproduced because it defines the semantics):** three mineral volumes summing to the total matrix volume. With no clay or porosity, V1=V2=V3=0.33 plots at the centre. With 30% porosity the same rock gives V1=V2=V3=0.33×0.7=0.233, which cannot be plotted (does not sum to 1), so the values are normalised up and the point **plots at the centre again**. "The values plotted on the Ternary Plot are **not the actual input values** of 0.233, but rather the **relative fractions**."

⇒ The ternary plot is **scale-invariant in the total**: it can never distinguish a matrix-rich from a porosity-rich rock of the same mineral proportions. That is by design, but it is a trap for anyone reading absolute volumes off it.

**No mineral endpoint values are printed anywhere on the page** — no ρma, no φCNL, no Pe, and (per rule 8) no smectite or montmorillonite. The apex endpoints are *curve scale* endpoints (0 and 1), not mineral response constants. Ternary overlays are user-drawn polygons; "several default overlays that are included" are mentioned but **never named** → **OPEN-L-08**.

Other ternary facts: colour by Well / Zone / Palette (palette driven by the Z-axis curve); overlay polygons export to a **text curve** of the area name per depth (useful as a classification/lithology curve); `Snap to grid` snaps polygon vertices to grid points; discriminators are applied **per well**; format file `.tpt`.

### 4.8 Log plot curve-style rules with computational content

(`logplot_log_plot_styles.htm`)

- **Variable shading:** three colour channels, each driven by a curve or fixed value. "When the Control value is **less than or equal to** the Zero colour then the colour intensity is **zero**. When the Control value is **greater than or equal to** the Max colour then the colour intensity is **maximum**." Zero intensity on all three ⇒ **black**; full intensity on all three ⇒ **white**.
- **Cumulative / array curves:** "If an array curve with a depth dimension greater than 1 is included in a cumulative plot then high frequency values will only be plotted **if all curves in the cumulative sum have the same depth dimension and are from the same curve or equivalent curve sets** (equivalent here means with the **same top depth, bottom depth and depth step**) otherwise **the average of the values in the array will be used**… If the X dimension is greater than one then **all values at the same depth are averaged** for plotting."
- Same rule restated for shading (`logplot_edit_the_log_plot_format.htm`): high-frequency shading requires both shading curves to be array curves from equivalent sets with the same depth dimension, "If this is not the case then curve values will be calculated at low frequency using the average array values."
- **Waveform:** zero-crossing value is "the Log value **half way between** the Log high value and the Log low value". Amplitude may be plotted logarithmically. Portion plotted may be given as sample numbers or waveform units (default = the **entire waveform**); to use values rather than sample numbers the Curve Header → Additional tab must carry the start/stop values.
- **Waveform depth averaging (tasking (f)):** "if a CMR array curve T2_DIST with 6 inch sample interval is Plotted every 1 ft, then, **if the `Average Data over interval` box is selected the waveform is calculated as the average of the data in 2 depth steps**. If the box is left cleared then waveform data, between the specified output intervals, is **ignored**."
- **Waveform histogram normalization:** "`Normalize Histogram Maximum Height to 1.0`: If turned on then the histogram values in the bins are normalized so that the maximum number is 1.0… **When turned on the 'Log low value' is set to -1 and the 'Log high value' to 1.**"
- **Images / Dip images:** left and right scale values may be entered and will show in the header, **but "the scales are not used for creating the Image"** — a display-only trap.
- **Dip image caliper:** "if both X and Y axes caliper curves are available… IP then will **average the two caliper values** and use the average value to calculate the magnitude of the dip image curve."
- **Array Plot:** track divided equally by array element count; **every 5th element is always coloured black** regardless of the chosen curve colour.
- **Borehole Profile:** first curve at the **6 o'clock position**, remaining curves **anticlockwise**; **View Inclination default 20°**; **View Declination default 0°**; shading palette scale defaults to the radius scale unless `Use Defined Plot Scales` overrides.
- **Synthetic seismogram:** zero-trace position is halfway between the left and right scale; `Trace Gain` **1.0 = no boost** (>1 amplifies, <1 attenuates); `Display Array Stack` off ⇒ **the average of the array data is plotted at each depth**; `Track width to use` 50% ⇒ left trace starts ¼ across, right trace ¾ across, others equally spaced.
- **Mini-plot `By Fixed Interval`:** mini-plots are constrained to a maximum vertical height and not allowed to extend beyond the specified interval; width then "maxes out at the size required to **maintain their aspect ratio** for the constrained vertical height."

### 4.9 Depth-shift / splice output conventions

(`plotting.htm`)

- Interactive depth shift **from the log plot**: "the default behaviour is to create a **new output set with `_ds` appended to its name**, and the shifted curves in this output set **retain their original names**. This is different to the standalone Interactive Depth Shift module, where we append `_ds` to the **individual curve names**."
- Splice: default output set named **`Spliced`**, spliced curves keep the initial curve's name.
- **Multiple output sets is the default choice** "even for the simple base case where there is only one output set."
- **Re-sampling rule:** "if the multiple input sets are at different step sizes, this will result in the data being **re-sampled** as required into the output set." And: "When the module creates an output set, it will have the **same well step as the first input set on that row**. It is assumed that all other input sets on the same row will have the same well step. If they do not, then during the splice, **that data will be re-sampled** to fit the output set."
- `No extra result tracks` is the default.

### 4.10 3D Petrophysics and SandPit 3D — what they compute (**tasking (e)**)

**3D Petrophysics** (`3d_petrophysics.htm`) is a **multi-well parameter-distribution and batch-interpretation driver**, not a volumetric or gridding engine. It:
- runs the standard single-well interpretation modules across a well set with consistent zonation;
- creates or links parameter sets by one of five strategies — `Use Correlation Set`, `Set Link to Correlation Set`, `Use default well interval (One zone)`, `Use Parameter distribution module`, `Copy from VClay Parameter Set`;
- can link Vclay↔PhiSw zone sets and clay parameters;
- accepts a Tops set **or** a Picks set as the correlation set, which must exist in every well.

Constraints and cautions: display limited to **50 wells** (works with all loaded wells); recommended workflow is interpret a key well → distribute → refine; **"If [Input/Output Curves and Options] are not defined correctly prior to the set creation, then some parameters may default to a zero value and have to be modified manually."**

**There is no interpolation, gridding, kriging, inverse-distance, contouring, cell-size or smoothing parameter anywhere in `3d_petrophysics.htm`, `sandpit3d.htm`, `sandpit_3d_settings.htm`, `wellmap.htm` or `ic-mapping.htm`.** A targeted search for all of those terms across the five pages returns zero hits. The "3D" in 3D Petrophysics means *many wells at once*, not *a 3D grid*. Recorded as a negative finding rather than an open item.

**SandPit 3D** (`sandpit3d.htm`) is a **geomechanics sanding-prediction** module despite living in the 3D family. What it computes:
- "evaluates the **likelihood of rock failure at the borehole or perforation** due to the imbalance of stresses applied to the rock with respect to its intrinsic strength";
- output metric is the **Load Factor (LF)**;
- explicit scope limit: "it **does not evaluate if this rock failure will lead to particles (i.e. sand) traveling up the borehole** or what volume of sand may be observed at the surface."

Two modes: **Multi Depth** (log-driven, whole well, for existing producers) and **Discrete Depth** (scenario-driven, "many scenarios (hundreds)", varying inclination, azimuth, strength, stresses, drawdown, depletion). Recommended runs: **zero depletion (original reservoir pressure) and full depletion (close to pore pressure)**, plus intermediate scenarios to find failure onset.

Four required upstream inputs, each its own module: **Rock Strength** (Density/Sonic/Neutron/Porosity → UCS and TWC, core-calibrated, best model selected as "final"), **Overburden Gradient** (density-to-surface vertical stress, with Density Estimation to extrapolate to surface if the log is short), **Pore Pressure** (from vertical stress + resistivity or sonic, with a **visually established normal compaction trend** adjusting the final curve), **Horizontal Stress** (ShMin from pore pressure + overburden, calibrated to **LOT/FIT** data; "**SHMax … as a simple scale up on ShMin**").

**No SandPit equations appear on either assigned page** — they live on `Sandpit 3D Multi Depth Analysis` / `Sandpit 3D Discrete Depth Analysis` / `Rock Strength` / `Rock Stress` / `Wellbore Stability`, none of which is mine → **OPEN-L-09**.

Settings defaults (`sandpit_3d_settings.htm`): `Automatically populate curve names` **default unchecked** (when checked, matches by curve-name alias for the required curve type, first match wins); `Warn of unsaved multi depth parameter changes` — if off, "the results are **lost** unless they have been explicitly saved previously"; per-curve unit override with a warning when curve units are invalid for the expected type or differ from the chosen unit.

### 4.11 Well map and correlation defaults

(`wellmap.htm`): **default well position is Latitude 0° / Longitude 0°** — wells without a position are "displayed on the Equator", stated twice. Zoom range **500 km (min magnification) to 10 m (max)**. Well paths require either a deviation survey processed through the True Vertical Depth module, or `EDIST`/`NDIST` departure curves.

(`multiwellcorrelationview.htm`): wells plot with their defined Default Plot or last-opened plot; if neither exists the plot is **blank** and a template must be pushed via `File > Load Single Well Format to all Wells`. Widths are in "IP project default units (inches or cm)".

(`well_diagram_manager.htm`): units **default inches**; the 3D **default slice starts at 0° and runs 180°**.

---

## 5. Overlay-chart inventory (**tasking (c)**)

Chart **identity, axes and applicability only** — no digitized chart-line data has been transcribed (rule 5).

### 5.1 Overlays observed in the shipped `Overlay Lines` dropdown

`[img-read: _cpclip0052.png]` — the dropdown, partially scrolled, on a crossplot with X = `NPHI`, Y = `RHOB`:

| Overlay description as listed | Contractor | Chart id / date as stated | Notes |
|---|---|---|---|
| `Reeves Sonic/Density Wyllie Dtf = 189` | Reeves | none stated | Wyllie, fluid transit time 189 |
| `Schlumberger NPHI RHOB overlay Rhofluid = 1.0 (CP-1c 1989)` | Schlumberger | **CP-1c, 1989** | highlighted entry |
| `Schlumberger Den/Neut Corr. Rhof=1.0` | Schlumberger | none stated | "Corr." = corrected-porosity variant |
| `Schlumberger Den/Neut Corr. Rhof=1.19` | Schlumberger | none stated | ρfl 1.19 (salt-mud variant) |
| `Schlumberger Den/Neut Raw Rhof=1.0` | Schlumberger | none stated | "Raw" variant |
| `Schlumberger Den/Neut Raw Rhof=1.1` | Schlumberger | none stated | ρfl 1.1 |
| `Schlumberger Neut/Sonic Wyllie Dtf=189` | Schlumberger | none stated | Wyllie, Δtf 189 |

The list is cut off by the dropdown; **the full shipped overlay set is not enumerated anywhere in the manual** → **OPEN-L-10**.

### 5.2 Second chart identity, from a plotted example

`[img-read: _cpclip0055.png]` — a crossplot titled `TNPH / RHOB` carrying the on-plot label:

> `(SWS) Density Neutron(TNPH) overlay, Rhofluid = 1.0 (CP-1e 1989)`

| Property | Value |
|---|---|
| Chart identity | **CP-1e, 1989**, Schlumberger (labelled `SWS`) |
| X axis | `TNPH - DEC`, observed range −0.05 → 0.6 |
| Y axis | `RHOB - G/C3`, observed range 3.0 (bottom) → 2.0 (top) |
| Lithology lines | **SS, LS, DOL** (3 lines) |
| Porosity ticks on lines | 0, 10, 20, 30, 40 (p.u.) |
| Fluid density | ρfluid = 1.0 |

So IP ships **two distinct Schlumberger 1989 neutron-density charts**, keyed to the neutron tool: **CP-1c for NPHI**, **CP-1e for TNPH**. Both at ρfl = 1.0. Getting these two confused is a silent-wrongness path — the neutron mnemonic determines the chart.

### 5.3 Overlay registry and file format

`[img-read: _cpclip0051.png]` — `Overlay_Files.ovlx`, an XML registry, one `<Row>` per overlay:

```xml
<Row>
  <Contractor>SCH</Contractor>
  <XType>Density</XType>
  <YType>Neutron</YType>
  <Description>Schlumberger NPHI RHOB overlay Rhofluid = 1.0 (CP-1c 1989)</Description>
  <ChartbookNum />
  <ChartbookDate />
  <Filename>Sch_NPHI_RHOB</Filename>
</Row>
```

Two structural observations that matter for SandiBumi:
1. **`<ChartbookNum>` and `<ChartbookDate>` ship EMPTY (self-closing).** The chart identity `CP-1c 1989` exists **only as free text inside `<Description>`**. IP has the fields for structured chart provenance and does not populate them.
2. **`<XType>`/`<YType>` are tool-type tags for auto-matching, not literal axis assignment.** This row reads `XType=Density, YType=Neutron` while the overlay is named "NPHI RHOB" and is applied to a crossplot with X=NPHI, Y=RHOB. Do not read the order as the plotting order.

`[img-read: _cpclip0055.png]` — the `.ovl` file (`Sch_NPHI_RHOB.ovl`) structure, sections in order:
`$ <free-text description line>` → `$ Tool Types` → `$ Lithology Name` → `$ Colour of Lines` → `$ Data: format X, Y, Type where type is '-', Value or 'Tick'` → `$ Data Labels` (`Format X, Y, Rotation, Colour, Label text`).

Format rules (`crossplot_functions.htm`):
- Lines beginning `$` are comments.
- First data line = **Tool Types** ("This information is used for automatically picking up the correct overlay files").
- Second = **line labels**, space-separated; the count of names determines the number of lines in the overlay.
- Third = **line colours**; valid colour names come from `CparmDef.xml`.
- Data points are `X value, Y value, Type` where Type is `Tick`, a numeric value to print, or `-` for nothing.
- Optional `Size <n>` (default font size) and `LineWidth <n n n …>` in the same order as the colour section.
- Lines need not have equal point counts, **but short lines must be padded with hyphens** so every line has the same parameter count.
- Data-label line with X=0 and Y=0 is plotted as a **title label at the bottom-left of the plot**.
- **The `$ Data Labels` line is mandatory** "in order to make the overlay file work correctly".
- Limits: 30 lines/file, 20 points/line, 30 data labels.
- Install path stated inconsistently as `C:\Program Files\IntPetro3x` / `IntPetro36` (§6, D-L-06).

**Draw order:** "The overlay lines are normally displayed **underneath the data** but can optionally be displayed on top" via `Function → Overlays on top of data`. (`crossplot_functions.htm`) An `Overlays` tab controls overlay line colour/weight and label font size/colour. (`createacrossplot.htm`)

### 5.4 What is *not* covered

`createacrossplot.htm` says the Overlay Lines box "allows you to select **Service Company chart book Neutron/Density and Neutron/Sonic porosity lines**". Only those two crossplot families are named. **No M-N, MID, Pe-U, Thomas-Stieber, Pickett or Hingle overlay chart is offered or referenced anywhere in my page set.** Pickett Sw lines are generated parametrically from Rw/m/n/a, not from a chartbook overlay.

---

## 6. Internal discrepancies

| ID | Severity | Description |
|---|---|---|
| **D-L-01** | **HIGH** | **Hingle Y-axis self-contradiction.** One sentence says the Y-axis is built from `(1/Rt)^(-1/m)`; two sentences later the same paragraph says the Y curve holds `Rt^(-1/m)`. These are reciprocals. Three other statements — `Rt_Hingle = Rt^(-1/m)` (`basicloganalysis.htm`) and the `Rt_Hingle`/`Rxo_Hingle` output-curve definitions (`porosityandwatersaturation.htm`) — all support `Rt^(-1/m)`. **Resolution: `Rt^(-1/m)` ≡ `1/Rt^(1/m)`; the `(1/Rt)^(-1/m)` phrasing is a double-negation typo.** Identical in 2018 and 2025. Pages are agent B's; see §2.2. |
| **D-L-02** | **HIGH** | **Lightness grey-scale equation is malformed.** `Grey = (MaxRGB + MaxRGB) / 2` (`picturecorephotodata.htm`). `MinRGB` is defined two lines above and unused. The expression as printed reduces to `MaxRGB`, making Lightness identical to Intensity, which would make it a pointless third option. Almost certainly `(MaxRGB + MinRGB)/2`, but **the page's value is reported as printed and not corrected**. Byte-identical in 2018 and 2025. |
| **D-L-03** | **MEDIUM** | **Histogram bin-class example is arithmetically inconsistent.** GR 0→150 over 50 divisions gives 3.0 per class, but the vendor lists "classes 0-3, 4-6, 7-9, 10-12 …145-147, 148-150" (`histogram.htm`) — the first class spans 4 and the rest span 3, and the sequence cannot tile [0,150] in 50 bins. It also contradicts the crossplot bin-edge rule (`>=` bin low value ⇒ left-closed `[low, high)` intervals, which would read 0-3, 3-6, 6-9…). Treat the crossplot rule as normative and the histogram example as illustrative prose. |
| **D-L-04** | **MEDIUM** | **Box plot "Display Median" plots the mean.** "· Display **Median**. When selected the **mean value** is plotted across the box as a black solid line. · Display **Mean**. When selected the **mean value** is plotted across the box as a green dashed line." (`box_plots.htm`) Two adjacent options both described as plotting the mean. One must be the median — presumably the first — but the page does not say so. Byte-identical in 2018 and 2025. **A SandiBumi box plot must not copy this; decide and document which line is which.** |
| **D-L-05** | **MEDIUM** | **Normalization constants are named inconsistently.** The equation and the three type definitions use **'a' and 'b'** (`Result = Input x 'a' + 'b'`), while the depth-interval discussion twice refers to "the normalisation **A and M** values" (`histogram.htm`). No 'M' is defined anywhere on the page. Same text in 2018 and 2025. Read A≡a and M≡b. |
| **D-L-06** | **LOW** | **IP install path stated three ways** across my pages: `C:\Program Files\IntPetro3x` (`createacrossplot.htm`), `C:\Program Files\IntPetro36` (`crossplot_functions.htm`), and `…\Local Settings\Application Data\intpetro36` for default plots (`logplot_save_default_plot_format.htm`). Version-suffix drift in the docs; does not affect any numeric result. |
| **D-L-07** | **LOW** | **Track border width default: 3 vs 2.** `plotoutput.htm` states "Track Border Width… The default is 3" (log-plot hardcopy output). `multiwellcorrelationview.htm` states "The default line thickness is set to 2" for `Change Track Border Width All Wells`. Different dialogs, plausibly different defaults, but not reconciled by the manual. |
| **D-L-08** | **LOW** | **Crossplot type list omits two shipped types.** Prose lists "Normal, Frequency, Pressure Gradients, Standalone Pickett" (`createacrossplot.htm`) and `crossplottypes.htm` documents no others, but the shipped Scales tab exposes **Standalone Rv/Rh Butterfly** and **Standalone Thomas Stieber** checkboxes `[img-read: _cpclip0064.png]`. Documentation gap in the plotting section; specs live on the laminated-sands/Sw pages. |
| **D-L-09** | **LOW** | **Ternary apex-value wording.** "In this typical case, the 0 and 100% values should simply be **left at 1**" vs, two paragraphs later, "It is rarely the case the 0 and 100% values should be set to anything other than **0 and 1**" (`ternary.htm`). The first is elliptical (meaning the 100% value); the pair is confusing but not contradictory once read carefully. |
| **D-L-10** | **LOW** | **Out-of-range handling differs by module and is nowhere reconciled.** Crossplot Z-axis: unchecked clipping ⇒ **clamp** to end colour/symbol (`createacrossplot.htm`). Waveform histogram: out-of-scale data **ignored** (`logplot_log_plot_styles.htm`). Histogram with log scale: **negatives → NULL** (`histogram.htm`). Three different policies; a unified engine must choose per-context deliberately. |
| **D-L-11** | **LOW** | **`mapping-resources.htm` is an unwritten vendor page** — body is "Enter topic text here." It is referenced from `ic-mapping.htm` ("The Resources will be discussed in the Resources Topic"). Shipped placeholder in the 2025 manual. |

**Cross-checks that PASSED (no discrepancy):**
- Pickett Sw-line defaults 0.2/0.3/0.5 in the dialog `[img-read: _cpclip0067.png]` match the Sw line labels on the plot raster `[img-read: _cpclip0068.png]`.
- The 1:500 grid spacing is stated independently in `plotoutput.htm` (metric-first) and `plottoprinter.htm` (imperial-first) with identical numbers.
- The normalization equation in prose (`Result = Input x 'a' + 'b'`) matches the shipped UI panel `[img-read: _hsclip0064.png]`.
- The Luminosity coefficients 0.21 + 0.72 + 0.07 sum to exactly 1.00.
- `Only display reference and normalized Histograms` is stated as defaulting to on in both `histogram.htm` and `createacrossplot.htm`.

---

## 7. IP2018 numeric diff

Method: `c25/<page>.htm` vs `c18/<page>.htm`, tag-stripped and whitespace-normalized, then targeted extraction of every numeric default recorded above.

### 7.1 Page-level

| Page | 2018 | 2025 | Note |
|---|---|---|---|
| `pie_charts` | **absent** | present | **New module in 2025** |
| `rose-plots` | **absent** | present | **New module in 2025** |
| `histogram` | 154,098 B | 165,078 B | grew — normalization section expanded |
| `crossplot_functions` | 161,649 B | 167,782 B | grew |
| `plotoutput` | 118,054 B | 118,818 B | ~unchanged |
| `3d_petrophysics` | 121,373 B | 123,268 B | ~unchanged |
| `crossplottypes` | 82,697 B | 77,513 B | shrank |
| `createacrossplot` | 87,758 B | 80,703 B | shrank |
| `ternary` | 54,900 B | 51,589 B | shrank, but gained the algorithm section (below) |
| `sandpit3d` | 17,637 B | 14,939 B | shrank |
| `box_plots` / `star_plots` / `picturecorephotodata` / `plottoprinter` | — | — | minor shrink, no numeric change |

### 7.2 Numeric defaults — **every value checked is UNCHANGED**

| Value | 2018 | 2025 |
|---|---|---|
| Box plot percentile default | 25% / 75% | **identical** |
| Star plot data-range default | ±2 standard deviations | **identical** |
| Histogram percentile statistics defaults | 10th, 50th, 90th | **identical** |
| Normalization equation | `Result = Input x 'a' + 'b'` | **identical** |
| Normalization types 1 & 2 (a=1 / b=0) | as stated | **identical** |
| Frequency-crossplot symbol thresholds | 10–35 → letters, >35 → `#` | **identical** |
| Pickett axis constraints (log scale, >0, Curve Type = Porosity) | as stated | **identical** |
| `Rt_Hingle = Rt^(-1/m)` | as stated | **identical** |
| Hingle prose `(1/Rt)^(-1/m)` | as stated | **identical** |
| Greyscale `Average = (r+g+b)/3` | as stated | **identical** |
| Lightness `(MaxRGB + MaxRGB)/2` defect | present | **identical — defect not fixed** |
| Box plot median/mean wording defect | present | **identical — defect not fixed** |

**No numeric default in the plotting stack changed between IP2018 and IP2025.** Anything SandiBumi calibrated against IP2018 plotting behaviour remains valid.

### 7.3 Substantive 2025 additions

**Ternary normalization algorithm is entirely new in 2025.** IP2018's `ternary.htm` contains **zero occurrences** of "normalis"/"normaliz" — the unity equation `A+B+C=1`, the per-depth-sample three-step algorithm, the 0%/100% pre-scaling explanation and the porosity worked example are all new documentation in 2025. The 2018 page documented the UI only. (Whether the *behaviour* changed or merely got documented is unknowable from the manuals → **OPEN-L-11**.)

Also new or expanded in 2025: pie charts and rose plots as first-class View-menu modules; the histogram normalization sections on zones, cumulative-set reference building, `Run All`, and `Next`/`Previous`.

---

## 8. SandiBumi notes

1. **Pickett is implementable from this ingest; Hingle is not.** IP's Pickett axis assignment (X = Rt log, Y = φ log), its four parameters, its defaults (Rw 0.1, m 2, n 2, a 1, Sw lines 0.2/0.3/0.5) and its two-DOF lock model (Rw, m) are all pinned. **But the line equation itself is never printed** — take it from agent B, do not reconstruct it from the geometry.
2. **Hingle Y-axis: build `Rt^(-1/m)`, label the axis in ohm-m.** The transform-vs-label split is the whole trap: the Format dialog exposes transform-space values, and the Y gridlines are a function of `m` and must be regenerated whenever `m` changes. Do not copy the vendor's `(1/Rt)^(-1/m)` phrasing into any SandiBumi doc — it is wrong as written and has been for at least seven years.
3. **Adopt IP's bin-edge rule explicitly:** bins are `[low, high)` — a point on a boundary goes to the higher bin. It is stated once, for the frequency crossplot, and is the only bin-edge statement in the whole plotting stack. Contradicting it silently would make every histogram and frequency plot differ from IP at the edges.
4. **IP's Mode is bin-dependent, not a raw-data statistic.** If SandiBumi computes a true raw-data mode, results will diverge from IP and the divergence must be documented, not treated as a bug.
5. **Normalization: `Result = Input × a + b`, three types, no shipped percentile defaults.** The `Lower %` / `Upper %` boxes ship empty — SandiBumi must either ask or pick a documented default (and cite it as SandiBumi's own choice, not IP's). Copy IP's separation of *calculate-where* from *apply-where*: it is the single most valuable design idea in the normalization module.
6. **Copy the coefficient-provenance trick.** IP writes normalization coefficients into the curve's comments field and reads them back to resume. That is cheap, durable per-curve provenance and fits SandiBumi's citation discipline.
7. **Overlay-chart provenance is structurally weak in IP and should be strengthened.** `Overlay_Files.ovlx` has `<ChartbookNum>` and `<ChartbookDate>` fields that ship **empty**, with the identity (`CP-1c 1989`) buried in free text. SandiBumi should populate structured chart-identity fields as a hard requirement.
8. **Neutron mnemonic selects the chart: NPHI → CP-1c (1989), TNPH → CP-1e (1989), both ρfl = 1.0.** Silently applying one to the other's data is exactly the kind of error this ingest exists to prevent. Also note the ρfl variants shipped: 1.0, 1.1, 1.19, and Wyllie Δtf = 189 for the sonic pairs.
9. **The ternary plot is scale-invariant in the total** and cannot distinguish matrix-rich from porosity-rich rock at equal mineral proportions. Worth surfacing in the UI, since IP does not.
10. **Decimation and truncation are different, and IP does both.** `Display every nth point` is a stride filter with no averaging (default 1). `Max Data Levels` on pie/rose plots **truncates from the top down**, silently discarding the bottom of the interval. The second is a data-integrity hazard; SandiBumi should subsample or warn rather than truncate.
11. **Three different out-of-range policies exist in IP** (clamp / ignore / null-on-negative). Pick one per context deliberately and document it — D-L-10.
12. **Do not copy the box-plot "Display Median"/"Display Mean" labelling.** Both are described as plotting the mean. Decide, implement, and label unambiguously.
13. **Neither Rv/Rh Butterfly nor Thomas-Stieber is documented in the plotting section** despite shipping as crossplot types — both directly relevant to thin-bed work. Their specs are on the laminated-sands pages.
14. **3D Petrophysics does no gridding.** If SandiBumi wants a genuine 3D/mapping capability it is greenfield, not a parity feature — IP's "3D" here means multi-well batch.
15. **Log plot defaults worth matching for familiarity:** default scale 1:500 with heavy/medium/light gridlines at 100'(50 m)/50'(25 m)/10'(5 m); grid widths 3/2/1; borders DarkGray; background white; font 12.

---

## 9. OPEN ITEMS

| ID | Item | Why it is open / what would close it |
|---|---|---|
| **OPEN-L-01** | **The Pickett line equation is never printed** on any of the 46 plotting pages. Slope↔`m`, intercept↔`a·Rw` and the Sw-line family are only inferable from the dialog fields, the lock behaviour and the plotted geometry. | Agent B's Sw pages (`swequationsandmethodology.htm`, `swparameters.htm`). Do **not** fill from textbook knowledge. |
| **OPEN-L-02** | **Hingle resolution rests on pages outside my assignment.** My verdict (`Rt^(-1/m)`) is 3-statements-to-1 and includes the executable curve definition, but all four statements are on agent B's pages. | Agent B to confirm independently. If B concurs, D-02 can be closed as a vendor typo, not a version discrepancy. |
| **OPEN-L-03** | **Formation-temperature Rw value truncated in the raster.** The Pickett status bar reads `Rw Form Temp : 0.09…` with the remaining digits cut off by the window edge. | Cosmetic — it is example data, not a default. No action needed unless someone tries to cite it. |
| **OPEN-L-04** | **Treatment of exact zero under log-scale histograms is unstated.** The page says only "Negative numbers are treated as NULL". Zero is undefined for log10 but is not "negative". | Not resolvable from the manual. SandiBumi must decide and document. |
| **OPEN-L-05** | **No default normalization percentiles ship.** `Lower %` / `Upper %` are empty in the shipped dialog `[img-read: _hsclip0063.png]`. | Confirmed absence, not an extraction failure. SandiBumi must choose its own and cite it as its own. |
| **OPEN-L-06** | **Gridline-spacing algorithm for scales other than 1:500 is not published.** Only the 1:500 row is given; other scales are "computed to produce a nice looking plot". | Would need empirical capture from a running IP, or acceptance of a SandiBumi-defined rule. |
| **OPEN-L-07** | **`Max Data Levels` default value is not stated** on either pie or rose plot pages, despite governing silent truncation. | Would need a running IP to read the shipped default. |
| **OPEN-L-08** | **Ternary "default overlays that are included" are never named.** No list of shipped ternary classification overlays (e.g. sand-silt-clay schemes) appears anywhere. | Would need the IP install's ternary overlay directory. Directly relevant to Jauhar's SSC work. |
| **OPEN-L-09** | **SandPit 3D equations are not on my pages.** LF (Load Factor), the failure criteria, UCS/TWC models, and the ShMin→SHMax scale-up factor are all referenced but unpublished here. | `Sandpit 3D Multi Depth/Discrete Depth Analysis`, `Rock Strength`, `Rock Stress`, `Wellbore Stability` — check whether any agent owns these; they may be unassigned. |
| **OPEN-L-10** | **The complete shipped overlay set is not enumerated.** The dropdown raster is cut off after 7 entries and `Overlay_Files.ovlx` is shown only as a 2-row fragment. | Would need the IP install's `Overlay_Files.ovlx` in full. Until then, treat §5.1 as a partial inventory. |
| **OPEN-L-11** | **Ternary normalization: new behaviour or newly-documented behaviour?** The algorithm text is entirely absent from IP2018 and present in IP2025. | Not decidable from the manuals alone. |
| **OPEN-L-12** | **Neutron/density axis scales seen in two screenshots may or may not be defaults.** `X NPHI −0.245 → 0.795` and `Y RHOB 3.3 → 1.7` appear in two independent rasters `[img-read: _cpclip0064.png]`, `[img-read: _cpclip0052.png]`. No prose states them as defaults; `createacrossplot.htm` says axis min/max come from Manage Curve Headers or `CparmDef.xml`. | Recorded as **observed, not confirmed default**. Do not adopt as a SandiBumi default without confirmation from `CparmDef.xml`. |
| **OPEN-L-13** | **Rv/Rh Butterfly and Thomas-Stieber crossplots undocumented in the plotting section** despite shipping as UI options. | Laminated-sands / Sw pages — confirm an agent owns them. |
