# Review checklist — for Jauhar's click-through in `npm run tauri dev`

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time; delete items as you confirm them.
Marks: `[o]` confirmed OK (removed from this file), `[x]` confirmed wrong → logged in
**ROADMAP.md §4 (Field-review backlog)**, `[ ]` not yet tested.

## All tools as dockview panes (2026-07-20 #24)

Your ask: "i want all tools shows as pane, for existing and future tools." Every
computation/analysis tool now opens as a **dockable pane** instead of a pop-up. The
big one: the **auto-generated module form** (every Petrophysics ▸ Data Prep / VSH /
Porosity / Saturation module) is now a pane — one per module — so you can keep
several docked side by side and re-run each as you iterate, and **any new module I
add in Rust gets its pane automatically** with no extra UI work. **Zones,
Autocorrelate Tops, Composite Log, and Report** are panes too; they follow the
selected well the way the plots do, and refresh their lists when data changes. Quick
pop-ups stayed pop-ups on purpose (curve editor, layout properties, save/open
session, import prompts). Adversarial review found 9 real issues, all fixed before
this shipped (pin-off panes catching up to a selection, no stale-well writes after a
project switch, the autocorrelate "pick a top first" message re-checking itself once
you pick one, etc.). tsc clean; module-pane behavior browser-verified.

- [ ] Open a module (e.g. Gas Correction) from the Petrophysics tab — it should
      appear as a pane you can dock/split/float, not a pop-up. Run it; the result
      lines stay in the pane (no auto-close). Open a second module — both panes
      coexist (the old pop-ups could not).
- [ ] With a module pane open, compute a curve, then open another module: the new
      curve should already be selectable in its input dropdowns (the pane refreshes
      its lists on data changes without losing what you'd already picked).
- [ ] Multi-select several wells in Wells & Tops, THEN open a module: all selected
      wells should be pre-ticked (not just the active one).
- [ ] Open the **Zones** / **Composite** / **Report** pane with no well selected —
      it shows "Select a well… will follow" instead of a "select a well first"
      toast; pick a well and it fills in and the tab title updates.
- [ ] **Autocorrelate Tops** on a well with no tops: the pane says "pick one in the
      log view first" — go pick a top, and the pane should update itself (no need to
      close/reopen). Apply a correlation: the proposals clear.
- [ ] Switch projects with a Zones/Report pane docked in a background tab: it must
      reset to the "select a well" hint, NOT keep showing a well from the old
      project (this prevents editing the new project with a stale well).
- [ ] Docking sanity: the panes save/restore with the workspace layout, appear in
      the ＋ "add panel" menu, and the log-view right-click "Print / export layout…"
      opens the Composite pane.

## Gas Correction module — iterated density de-gassing (2026-07-20 #23)

**Petrophysics ▸ Data Prep ▸ Gas Correction (density, iterated)** — the KKT deck
slide-65 loop. Density porosity and Archie SWT are solved from the current density,
then RHOB_GC = RHOB + Φt·(1−Sw)·(RHO_FL − GASDEN) replaces gas with liquid, iterated
to |ΔΦt| < 1e-4 (non-converging samples stay MISSING). GASDEN is the real-gas density
of an SG_GAS 0.65 gas at FPRESS/FTEMP (Standing pseudo-criticals + Papay z, pinned
0.1297 g/cc at the KK example's 2743 psi / 93.9 °C) — **run precalc first**; FTEMP and
FPRESS accept only precalc/log-set curves, never a raw import (a Geolog LAS's degF
FTEMP can't sneak in as degC). Default **OPT_GATE = FLAGGED** corrects only where the
gas flag > 0.5 (chain condflag's XOVER_FLAG, which excludes coal and washout) and
errors loudly if the flag curve has no data; **EVERYWHERE** is there for wells without
condflag, but beware coals/resistive washouts — high RT + low density reads as gas to
the Archie loop. The adversarial review raised 13 confirmed findings → all fixed
(FLAGGED default, flag > 0.5 gate, no-flag-data error, degenerate RHO_MA/RHO_FL and
RHOB<RHO_FL and Rw≤0 guards, non-convergence → MISSING, NaN-proof Archie cap,
computed-only P/T inputs, RHOG→GASDEN rename, doc rewrite). 127 cargo tests green.

- [ ] Run precalc → condflag → Gas Correction (defaults) on a KK-style gas well: the
      detached high-porosity gas cloud on PHIE vs wet-clay (slides 66–67) should
      collapse after correction; RHOB_GC ≈ RHOB in water zones (self-limiting there).
- [ ] Check a coal streak stays untouched under the FLAGGED default (XOVER_FLAG
      excludes coal) — no phantom high-porosity pay in coals.
- [ ] Without condflag run: the FLAGGED default must error "gas flag has no data —
      run condflag first or set OPT_GATE = EVERYWHERE", not silently pass through.
- [ ] Without precalc run: outputs stay MISSING (never uncorrected pass-through),
      even if the well's LAS carries its own raw FTEMP/FPRESS curves.
- [ ] Feed RHOB_GC to **phi_den** (or use PHIT_GC directly). Do NOT feed it to phi_dn
      or a SandiMin solve that includes NPHI — their gas handling assumes an
      uncorrected density-neutron pair (the module doc says this too).

## SandiMin: wet→dry clay converter + fluid autofill from precalc (2026-07-20 #22)

Two additions inside the **SandiMin** pane (Advance tab), from your Multimin
Parameters.xlsx workflow (Wave E item 18). **Wet clay → dry clay** panel: enter the
wet-clay picks from a shale interval (RHOB/NPHI/GR, optional DT) and the assumed
dry-clay density (2.70 marine / 2.78 deltaic per the KKT deck slide 60); it computes
φ_clay = (ρdry−ρwet)/(ρdry−1) and the dry endpoints with the xlsx formulas verbatim
(water 1.00 g/cc, 189 µs/ft), previews them live, and **Apply** writes them to the
chosen clay, ticks it + BoundWater, and sets a **CEC_eq** on the clay that makes the
solver's Dual-Water bound-water constraint enforce exactly v_bw = φ/(1−φ)·v_dryclay —
the deck's slide-59 bookkeeping (SWB = VOL_UBNDWAT/PHIT). Unphysical picks error
instead of applying: NPHI must be a fraction (percent entry rejected — Geolog habit
guard), GR positive, wet DT above the 189·φ water term. **Autofill from precalc**
(fluid box): pick a zone of the selected well and **Read** — fills Formation temp
from FTEMP_F and the Rmf sample from precalc's RMF (retied to formation temp, an
Arps no-op, only when both curves came back; a raw RMF without FTEMP_F is refused
as not-precalc). The zone dropdown follows your well selection live.

- [ ] KK-1 Post Main check: wet 2.18333/0.48958/110 with dry density 2.70 → the
      preview must read φ_clay 0.3039, NPHI 0.2667, GR 158.0 (the xlsx values).
- [ ] Apply to Illite, then run SandiMin with CT on: solved VOL_UBNDWAT/VOL_DRYCLAY
      should sit at ~0.4366 (= φ/(1−φ)) in clay-rich intervals; SWB = VOL_UBNDWAT/PHIT
      comparable to the deck's slide-59 CWB-panel behaviour.
- [ ] Note the pairing rule: CEC_eq is tied to the clay's **RHOB endpoint** and the
      fluid **T/Rw/α** at Apply time — if you edit any of those afterwards, re-Apply
      (the status line and the CEC column tooltip both say so now).
- [ ] Autofill on a precalc'd well: Read (whole well and one zone) fills FTEMP/Rmf
      and the previews update; on a well without precalc it must refuse with "run
      the precalc module first", not fill garbage.
- [ ] Switch wells with the SandiMin pane open: the autofill zone list must follow
      the selection (it re-reads the new well's zones).

## Neutron Matrix Conversion module — NPHI LS/SS/DOL (2026-07-20 #21)

New Prep module **Neutron Matrix Conversion** (`nphimat`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). Converts a neutron log recorded in
one matrix convention into all three — **NPHI_LS / NPHI_SS / NPHI_DOL** — using the
chartbook porosity-equivalence curves digitized at vector precision: **Por-5** for
the CNL thermal tools (**NPHI** ratio method; **TNPH** env-corrected, FRESH / 250 kppm
SALT variants) and **Por-4** for the epithermal tools (**APLC/FPLC** = APS, **SNP** =
legacy sidewall). Tell it what the log is (TOOL + MATRIX_IN); the input convention
passes through unchanged and the other two are read through the chart (SS/DOL inputs
invert back to the apparent-limestone axis first). The book's printed worked example
(TNPH 18 pu @ 250 kppm → sandstone 24 pu) reproduces to 0.04 pu. Feed the output
matching your RHO_MA (NPHI_SS with 2.65) — that removes the ~0.04 LS-vs-SS offset the
condflag doc warns about, so XOVER_MIN can stay at 0.04. Also in this increment:
APS/legacy neutron mnemonics (APLC/FPLC/SNP/NPOR/HNPO/NEUT/FSTP) now fill the
standard NPHI column at LAS import, an all-NaN standard column now falls back to the
raw store (family alias) instead of silently feeding NaN to modules, and workflow-
builder input dropdowns now offer every module's outputs so `nphimat → phi_dn
(NPHI = NPHI_SS)` is buildable in a fresh project.

- [ ] Run nphimat on a Mahakam well (TOOL matching the delivery, MATRIX_IN per the
      LAS header — usually LS or SS): NPHI_SS ≈ NPHI_LS + 0.03-0.04 in clean sand,
      NPHI_DOL well below both (thermal dolomite bow).
- [ ] Sanity vs the paper chart: pick one depth, read Por-5 by hand, compare all
      three outputs (expect agreement within ~0.5 pu).
- [ ] Feed NPHI_SS + RHO_MA 2.65 into phi_dn / condflag: crossover in a known gas
      sand appears at XOVER_MIN 0.04 without the limestone-unit offset fudge.
- [ ] Workflow builder in a fresh project: chain nphimat → phi_dn with the NPHI
      input overridden to NPHI_SS (now offered in the dropdown before any run).
- [ ] If you have an APS well (APLC): import fills NPHI now — check the curve
      arrives and nphimat TOOL=APLC gives sensible (small) matrix shifts.

## Data Conditioning Flags module — coal / tight / crossover + shoulder (2026-07-20 #20)

New Prep module **Data Conditioning Flags** (`condflag`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). One run writes five 0/1 flag
curves: **COAL_FLAG** (RHOB < 1.9 & NPHI > 0.35, plus DT > 100 µs/ft where a sonic
exists; samples with BADHOLE = 1 are never called coal — washouts mimic coal),
**TIGHT_FLAG** (density porosity and NPHI both < 0.05; DPHI uses **RHO_MA/RHO_FL —
the same params and zone overrides as the density-porosity modules**),
**XOVER_FLAG** (gas crossover DPHI − NPHI > 0.04; coal and bad hole excluded —
NPHI must be matrix-consistent with RHO_MA, else raise XOVER_MIN to ~0.08 for
limestone-unit neutron), **SHOULDER_FLAG** (the adjustment you asked for: samples
within SHOULDER of a coal/tight bed edge — or a bad-hole interval ≥ MIN_THICK —
carry boundary-averaged readings and get flagged so no shoulder log survives the
mask), and **COND_FLAG** (combined mask: coal | tight | badhole | shoulder, plus
crossover only when OPT_XCOND = YES). Beds thinner than MIN_THICK are dropped as
spikes; a missing sample inside a bed does not split it. MIN_THICK/SHOULDER are
in the depth curve's unit (defaults suit metres — roughly ×3 for feet). Run
badhole first; feed COND_FLAG as the Mask on later runs, but leave the Mask empty
on the condflag run itself. BADHOLE and COND_FLAG are now always offered in every
Mask dropdown, even in a fresh project where they haven't been computed yet.

- [ ] Run badhole → condflag on a Mahakam well with coals: COAL_FLAG picks the
      coal streaks (check against the density track), and no coal call inside
      washouts.
- [ ] TIGHT_FLAG on a calcite-cemented/tight streak; XOVER_FLAG on a known gas
      sand; crossover NOT flagged over coals.
- [ ] SHOULDER_FLAG brackets each coal/tight bed by ~SHOULDER depth units; a
      lone one-sample BADHOLE blip is masked in COND_FLAG but does NOT dilate.
- [ ] MIN_THICK: single-sample spikes dropped; a real bed with one null sample
      in the middle is kept whole.
- [ ] Feed COND_FLAG as Mask on a porosity run: flagged + shoulder samples go
      missing in the outputs; confirm COND_FLAG appears in the Mask dropdown of
      a fresh workflow before condflag has ever run.
- [ ] Zone overrides: RHO_MA 2.71 in a carbonate zone shifts TIGHT/XOVER there
      (same override the density-porosity modules use).

## Wave E-17: pre-calculation module — P / T / Rmf / Ct / Cxo (2026-07-20 #19)

New Prep module **Pre-Calculation (P / T / Rmf / Ct / Cxo)** in the Data Prep
dropdown and the workflow builder (ROADMAP §4c item 17, from your KKT ONWJ
workflow). One run writes six curves: FTEMP (**always degC** — the unit every
downstream module assumes) plus FTEMP_F (the degF twin, for SandiMin fluid
entry) and FPRESS as linear trends in TVDSS (gradients per depth unit of the
TVDSS curve — per-metre values for metric wells; no TVDSS curve → measured
depth is used), RMF at formation temperature (ARPS from a surface Rmf
measurement, or TREND regression `RMF_A + RMF_B·log10(TVDSS)` for wells
without mud data — the shipped defaults are the ONWJ **feet-based** fit), and
CT = 1000/RT, CXO = 1000/RXO in mmho/m as QC/plotting conductivities (note:
SandiMin's CT/CXO tool rows read the resistivity curves directly — don't feed
these to them). Params are SURF_TEMP/TEMP_GRAD (own names, so zone overrides
never cross-apply with Formation Temperature's degC-only TSURF/TGRAD); entry
unit degF/degC via OPT_TU.

- [ ] Run it on a KKT-style well with your fits (SURF_TEMP 77 / TEMP_GRAD
      0.0260292, PSURF 44.2823 / PGRAD 0.539812, degF): FTEMP_F/FPRESS match
      the deck's trend lines; spot-check one depth by hand; FTEMP = same in degC.
- [ ] Deep resistivity input defaults to the RES_DEEP family (same as the sw_*
      modules) so CT fills for wells whose deep curve is ILD/LLD/AT90 etc. —
      confirm CT is not blank on a standard import.
- [ ] ARPS mode: RMF at depth ≈ your surface Rmf pulled down by (T₁+6.77)/(T₂+6.77);
      TREND mode with A 0.517068 / B −0.116517 reproduces the field regression.
- [ ] degC mode on a metric well (e.g. SURF_TEMP 25, TEMP_GRAD 0.03 degC/m):
      FTEMP in degC, FTEMP_F in degF, RMF still Arps-correct.
- [ ] CT/CXO: 1000/RT and 1000/RXO, missing where RT/RXO are missing or ≤ 0.
- [ ] Zone overrides: give one zone a different TEMP_GRAD in the Zones dialog —
      the FTEMP trend kinks at the zone boundary (per-zone params resolve per
      sample).

## Wave A-4: workflow grid inspector (2026-07-20 #18)

The Workflow Builder pane has a **List | Grid** toggle above the step list
(ROADMAP §4c item 12). Grid = the multi-line inspector: rows are your chain's
steps, columns are the union of every step's inputs/params/options (+ Mask), so a
parameter shared by several modules lines up in one column. The italic **Set all**
row under the header edits a parameter across every step that takes it in one go.

- [ ] Build your standard chain (vsh → phi → sw_* …), switch to **Grid**: input
      curves come first, then numeric params, then options, then Mask; steps that
      don't take a column show "—". Header tooltips = parameter descriptions.
- [ ] **Set all → RW**: type one RW in the Set-all row — every sw_* step that takes
      RW updates at once (status bar reports how many). A value outside one
      module's allowed range is skipped for that module only and reported.
- [ ] Edited cells tint amber and the step's override badge counts up — same
      only-store-differences rule as the per-step editors, so a value typed equal
      to a module's default clears that override (cell untints). Zone params still
      override these whole-well values per zone at run time, as before.
- [ ] **Set all → Mask** sets opts.MASK (e.g. BADHOLE) on every step in one edit.
- [ ] Toggle List ↔ Grid: values, badges and invalid-input flagging stay in sync
      (both views edit the same steps). The chosen view is remembered.
- [ ] Save the workflow, reload it, re-run — saved JSON is unchanged in shape, so
      old saved workflows load into the grid fine.

## Wave A-3: project open/switch, IP style (2026-07-20 #17)

You can now keep separate project databases (balam.duckdb, minas.duckdb, …) and
switch between them inside the app (ROADMAP §4c item 2). Project ribbon tab, new
group left of Appearance:

- [ ] **New Project…** creates a fresh, empty .duckdb and switches to it — import a
      couple of Balam South LAS files there, confirm they do NOT appear in your main
      project, then switch back.
- [ ] **Open Project…** switches to an existing file; **Recent ▾** lists the last 12
      projects (current one marked ●, deleted files greyed "(missing)"), stored in
      `%APPDATA%\SandiBumi\projects.json` — outside any project.
- [ ] On switch: window title + group caption show the project name, well list /
      plots / catalogs all reload, well selection and undo history clear (old-project
      undo entries would corrupt the new one — deliberate).
- [ ] **Next launch reopens the last project you had open** (falls back to the old
      `project.duckdb` if the recents list is empty — first launch after this update
      behaves exactly as before).
- [ ] Switching is refused while a workflow chain is running (try it: start a long
      chain, then Open Project — you should get a clear error, not a corrupted run).
- [ ] Note: QAT **Save Project As** stays a backup copy (app keeps working on the
      current file) — tell me if you'd rather it switch to the copy, IP-style.

## Wave A-2: compact import ribbon (2026-07-20 #16)

The Data tab's eleven flat import buttons are now three Office-style dropdowns
(ROADMAP §4c item 4) — same handlers, just organized:

- [ ] **Import Logs ▾** (LAS, DLIS), **Import Data ▾** (Core, SCAL, Tops, Aux,
      Deviation), **Export LAS** (unchanged flat button), **Tools ▾**
      (Autocorrelate Tops, Shift Core, Well Header). Run one import of each kind —
      behaviour must be identical to the old buttons; tooltips moved onto the
      menu entries.
- [ ] Only one menu opens at a time; picking an item or clicking elsewhere closes it.
- [ ] Bahasa Indonesia / Basa Sunda: the new labels translate (Impor Log / Impor
      Data / Alat) including the previously untranslated Import Tops / Import Aux /
      Autocorrelate entries.

## Wave A-1: tool panes + theme compliance (2026-07-20 #15)

Four tools moved from popup dialogs to dock panes (ROADMAP §4c item 14) — they now
dock/float/tab like the Workflow Builder and can't be dismissed by a stray click:

- [ ] **Cutoffs & Pay Summary**, **ML Models**, **Monte Carlo**, **SandiMin** ribbon
      buttons each open a PANE (singleton: clicking again focuses the existing one).
      Run each on Balam South data — results should be identical to the old popups.
- [ ] The ＋ add-panel menu on any window now lists all four (under Workflow Builder);
      the right-click menu inside each pane shows its own heading.
- [ ] SandiMin's endpoints matrix now uses the full pane width (was capped at 620px).
- [ ] Panes reopen after an app restart (from the autosaved workspace) in their
      docked position — internal selections (cutoff values etc.) reset, same as the
      Workflow Builder.
- [ ] **Theme check** (switch to Dark, then Pertamina): the log-view cursor readout
      pill now inverts with the theme (was unreadable in dark); crossplot/Pickett/
      histogram pick swatches + histogram pick markers follow the theme accents
      (Pertamina = blue/lime, was always brown/green); core-plug diamond outlines
      visible in dark; workflow invalid-input red and error text use the theme warn
      color; the composite preview surface is no longer light grey in dark themes.

## Chartbook overlay library + audit quick fixes (2026-07-20 #14)

The single D-N overlay grew into a **chart overlay library** (Properties → Overlays →
Chart overlay): every crossplot-family chart from your 2013 chartbook, digitized from
the PDF vector artwork with the same validation stack (graduation sequences, 5-multiple
long dashes, worked examples). Charts matching the current axes are listed first; a
chart draws only when the plot axes actually match it (either orientation).

- [ ] **CNL Por-11/12** (as before, now via the new select — old saved props migrate).
- [ ] **EcoScope Por-18 (BPHI) / Por-19 (TNPH)** on an LWD well — these are the ones
      that matter for your Mahakam development wells; check a known sand against the
      sandstone line for both BPHI and TNPH inputs.
- [ ] **adnVISION675 Por-16** if you have ADN wells.
- [ ] **APS Por-13/14** (APLC and FPLC variants listed separately).
- [ ] **PEF: Lith-3/4** on a PEF-RHOB crossplot — quartz ~1.65-1.8, calcite ~5.08,
      dolomite ~3.1 curves with 10-pu labels.
- [ ] **Sonic-neutron Por-20** (both time-average AND field-observation families) on
      a DT-NPHI crossplot — TA curves reproduce Wyllie with tf 190 to R² 0.99999.
- [ ] **Density-sonic Por-22** (TA + FO) on a DT-RHOB crossplot, with the 7 mineral
      points (Sylvite, Salt, Trona, Gypsum, Sulfur, Polyhalite, Anhydrite).
- [ ] **Th-K clay chart Lith-2** on a POTA-THOR crossplot — the Th/K ratio fan is
      drawn at the *labeled* ratios (the chartbook's own printed lines sag a few %
      off their labels; ours are exact), plus the dashed clay/feldspar lines and
      mineral-field labels. Judge your Mahakam illite/kaolinite mix against it.
- [ ] **Pe-K and Pe-Th/K clay boxes Lith-1** (the Th/K variant needs the X axis in
      log mode — turn on X log in Properties).
- [ ] **Umaa-Rhomaa MID Lith-6** — the ternary triangle with 20/40/60/80 subdivisions
      + K-feldspar/Barite/Anhydrite/Kaolinite/Illite/Salt points. Needs computed
      UMAA/RHOMAA curves (equation engine for now; a dedicated module is a good next
      increment if you want it).

**Audit quick fixes** (from the full senior audit — see AUDIT-2026-07-20.md and
ROADMAP §4b for the 35-finding backlog):

- [ ] **Pay summary change**: with a PERM cutoff active, samples with **missing PERM
      now FAIL the cutoff** (they silently passed before). Re-run a pay summary on a
      well with patchy PERM — net pay may legitimately decrease. Tell me if you'd
      rather missing-PERM samples pass (Geolog's default behavior differs by setup).
- [ ] **LAS import**: the file's own ~W NULL declaration is now honored (deliveries
      using -99999 etc. no longer import sentinels as data), and **multi-word well
      names survive** ("BALAM SOUTH-01" no longer truncates to "SOUTH-01"). Re-import
      one such file and check the Wells pane name.
- [ ] **Depth scale presets are now TRUE ratios** (1:200 = 1 m of well per 5 mm of
      screen at standard DPI). They were ~39x too compressed before, so 1:200 will
      look much more stretched than you're used to — the numbers are honest now.
- [ ] **Tops editor**: adding a top with an existing name is an overwrite; Ctrl+Z now
      restores the previous depth instead of deleting the top.
- [ ] Case-insensitive computed-curve lookup (lowercase equation outputs now resolve).

## P2-f+ — D-N chartbook overlay (2026-07-20 #13)

Digitized from the Schlumberger 2013 chartbook you sent (Por-11 fresh / Por-12 salt,
extracted from the PDF's vector artwork — graduation-dash positions, not eyeballed;
calcite identity check rms 0.13 pu, both charts' worked examples reproduce).

- [ ] **Crossplot Properties → Overlays → D-N chart**: pick *Fresh mud (Por-11)* on an
      NPHI-RHOB crossplot → quartz/calcite/dolomite curves appear with porosity
      graduation dots + labels every 5 pu, dashed iso-porosity connectors, and curve
      names written along the lines. Compare against your paper chartbook page 225.
- [ ] **A real Mahakam sand interval** should plot on/left of the quartz sandstone line
      (shale pulls points right/down toward higher NPHI). Crossplot porosity read off
      the graduations should match your PHIE within ~1-2 pu in clean sand.
- [ ] **Salt variant** (Por-12) shifts the curves left at high porosity — only relevant
      if you ever work salt-mud wells; check it renders and the graduations differ from
      Fresh.
- [ ] **Zoom/pan**: the overlay must stay registered to the data under Ctrl+wheel zoom
      (it's drawn in data space). Also check the flipped orientation (X=RHOB, Y=NPHI).
- [ ] **Gating**: on a GR-RHOB plot or with a log axis the overlay silently stays off
      (chart geometry only means something on linear NPHI-RHOB).
- [ ] **Note**: the chartbook draws its dolomite curve for ρma **2.85** (validated
      against the chart's own graduation ticks), while the *Matrix points* overlay keeps
      the textbook single point at 2.87 — so Dol point and Dol curve start won't
      coincide exactly. Tell me if you'd rather I move the matrix point to 2.85.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [ ] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [ ] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [ ] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [ ] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [ ] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [ ] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
      Windows scaling): the desktop app now launches WebView2 with `--enable-lcd-text`,
      which forces ClearType subpixel antialiasing on GPU-composited panels (dockview
      layers otherwise fall back to fuzzy grayscale smoothing). **Needs the `npm run tauri
      dev` restart** (config change). Look closely at ribbon/dialog text afterward — if it
      still reads soft, next steps are a base-size bump 12→13px and/or semibold.
- [ ] **T-S triangle now appears** (your "not showing (?)"): the triangle is drawn on
      VSH (0–1) vs PHIT axes — before, ticking it on the default NPHI-RHOB crossplot put
      every line off-scale, so nothing visibly happened. Now ticking **T-S triangle**
      auto-switches the X/Y axes to the well's VSH/PHIT curves (status bar tells you), and
      if the well has no VSH/porosity curves yet it says to run those modules first.
      Check: tick it on a fresh crossplot → axes flip, triangle + drag handles visible.

## P1-a — Interaction safety batch (2026-07-19 #3)

- [ ] **Right-click lockdown**: right-click anywhere that has no SandiBumi menu (ribbon,
      buttons, tables, empty space) → **nothing** appears (the WebView menu with its
      dangerous Refresh is gone). Panel backgrounds still show our own menus; right-click
      inside a text box still shows the normal cut/copy/paste menu.
- [ ] **Reload guard**: press **F5** or **Ctrl+R** → a blocking confirm appears instead of
      an instant refresh; Cancel keeps everything, Reload restarts the workspace. Alt+←/→
      and the mouse back/forward side-buttons do nothing.
- [ ] **Double-click-to-edit numbers** (app-wide): single-click any numeric parameter
      field (module dialogs, plot properties, SandiMin, zones…) → it focuses with a dashed
      outline but typing/arrows/wheel change **nothing**; **double-click** → solid outline,
      value selected, editing works. Tab-into-field still edits directly (deliberate).
      Scrolling a dialog with the wheel can no longer spin a value.
- [ ] **Workflow Builder is a pane**: Petrophysics → Workflow… now opens a docked
      **Workflow Builder** pane (tab, movable/floatable like any panel) instead of a popup.
      No more losing a half-built chain to a stray click; it survives layout changes and
      reopens via the ＋ panel menu too. Run/cancel/progress unchanged; closing the pane
      mid-run cancels the chain.

## P1-b — Crash safe-mode, autosave, unsaved markers (2026-07-19 #3)

- [ ] **Autosave**: the workspace (panes, arrangement, active well, every log view's
      layout) autosaves every 10 seconds. Nothing to click — just know it's there.
- [ ] **Crash recovery**: if the app dies abnormally (crash, force-kill, power loss),
      the next launch shows a choice **before** anything loads: *Restore autosaved
      workspace* (everything back as it was moments before the exit) or *Start in Safe
      Mode* (clean default layout; the autosaved workspace is stashed as a "Recovered …"
      session under Open Session, so nothing is lost). To test without crashing for real:
      end the task from Task Manager while the app is open, then relaunch.
- [ ] **Normal restart is less lossy now**: on a clean exit + relaunch, the app also
      brings back the **active well** and each log view's **layout/track state** (before,
      only the pane arrangement survived).
- [ ] **Unsaved markers**: edit a log view (track widths, properties, curve visibility)
      → its tab shows **●** and the QAT Save-Session button gets a red dot. **Save
      Layout** clears that panel's ●; **Save Session** clears everything. The dot means
      "not in a named save yet" — the crash autosave protects you regardless.

## P1-c — Log sets: versioning, provenance, catalog search (2026-07-19 #3)

- [ ] **Never overwrite**: every module dialog now has an **Output set** field (default
      INTERP; type any name — FINAL, TEST, …). Run a module, then re-run it with different
      parameters: the Curve Catalog's "Log sets" section shows **v1 AND v2** — the old
      run's values are kept, not destroyed. Plots/log views show the latest (v2).
- [ ] **Restore a version**: in Inspector → Curve Catalog, click **Restore** on v1 → all
      open log views and plots flip back to the v1 curves. Restore v2 to return.
- [ ] **Per-curve provenance**: the catalog now lists every computed curve's **set + version,
      module, and timestamp** (hover a set row for the exact parameters and input curves
      it was run with). Answering "where did this VSH come from?" is now one glance.
- [ ] **Catalog search/filter/sort**: one search box matches mnemonic, set, module, unit,
      or date; click any column header (Mnemonic, Set, When, n, Min, Max, Mean…) to sort,
      click again to reverse. Statistics (n/min/max/mean) shown per computed curve.
- [ ] **One version per chain run**: the Workflow Builder also has an Output set field —
      a whole chain run (VSH → porosity → Sw) lands as ONE version, not one per step.
- [ ] **Prune old versions**: Delete on a set version (two clicks — it asks "Confirm
      delete") removes only that version's history; current curves are never touched.
      Equation runs land in set EQUATION, ML in ML, SandiMin in SANDIMIN, automatically.
- [ ] **Input set** (the other half of set in/out): run VSH into Output set **FINAL**,
      then re-run with different parameters into **INTERP** (current values are now
      INTERP's). Open a module that consumes VSH (e.g. sw_indo), set **Input set =
      FINAL** → the run uses FINAL's VSH, not the current one. Blank Input set = normal
      behavior (latest values). Works in the Workflow Builder too; curves the input set
      never wrote (GR, RHOB…) still come from the usual sources.

## P2-a — Tops-style imports (2026-07-19 #4)

- [ ] **Import Tops…** (Data tab): pick a CSV or TXT tops file. With a WELL column
      (WELL/WELLNAME/UWI…) every matching project well gets its tops in one import —
      names match case-insensitively, unmatched names are reported in the status bar.
      Without a WELL column the tops land in the selected well. Columns understood:
      TOP/MARKER/SURFACE/FORMATION/HORIZON + DEPTH/MD/TOP_MD; also bare headerless
      "NAME DEPTH" text lines. Delimiters auto-detected (comma / semicolon / tab /
      spaces). Re-import updates depths but keeps colors you've set.
- [ ] **Import Aux…** (Data tab): petrography, XRD, or perforation data for the
      selected well (or a custom-named dataset). Needs a TOP/DEPTH column; a
      BASE/TO column makes rows intervals (perforations); every other column becomes
      an item — numbers (mineral %, grain size) and text (status, remarks) both kept.
      Re-importing a dataset replaces only that dataset for that well.
- [ ] **View it**: Data → DB Inspector → table "Aux Data" shows the imported rows
      per well (read-only — re-import the file to change values). Tops appear
      immediately in the Wells & Tops pane and all log views/correlation.

## P2-f — Crossplot v2 (2026-07-20 #12)

- [ ] **Properties dialog**: double-click or right-click the crossplot (or ⚙ Properties)
      → sectioned dialog (Plot / Axes / Z color / Regression / Overlays). The old
      always-visible properties row is gone; the toolbar is just X/Y/Color/Zone.
- [ ] **Marginal histograms + percentiles**: enable marginals on NPHI-RHOB — X histogram
      on top, Y histogram on the right, aligned with the axes (RHOB's inverted axis
      included). Percentiles `25, 75` draw dashed reference lines on both axes.
- [ ] **Regression options**: on a PHIE-vs-PERM cloud try Power + RMA — the fit line
      must be straight on log axes and curved on linear ones, equation + R² + method
      tag shown top-left. Compare Y-on-X vs RMA slope on a noisy cloud (RMA steeper).
- [ ] **Log-safe Z coloring**: color by PERM with "Log Z scale" + Viridis — low and high
      decades must stay distinguishable (rainbow + linear crams everything in one hue);
      the color bar is labeled "(log)".
- [ ] **Plot size**: set Fixed 500×400 — the plot stops stretching with the pane
      (consistent exported figures). "Fill panel" restores the old behavior.
- [ ] **Universal defaults**: Qtz/Cal/Dol matrix points no longer appear on NPHI-RHOB
      unless ticked in Properties; Color has a "— None —" option (custom point color
      applies); the pick rows + drag handle can be hidden ("Show parameter pickers" —
      still ON by default so your drag-to-set-shale-point workflow is unchanged).

## P2-e — Histogram v2 (2026-07-20 #11)

- [ ] **Properties dialog**: double-click or right-click the histogram plot (or the ⚙
      Properties button) → one dialog holds display mode (bars/line), bins, normalize,
      cumulative overlay, box plot, color, percentiles, statistics placement, and the
      parameter-picker toggle. When zoomed, the first double-click resets the zoom, the
      next one opens properties.
- [ ] **Box plot + cumulative overlay together**: enable both on a GR histogram — the
      P25–P75 box with P50 line and P5/P95 whiskers sits under the marker labels, and
      the cumulative % curve (secondary color, % labels on the right edge) tracks the
      bars. Zoom in with Ctrl+wheel: box and whiskers follow the axis.
- [ ] **User percentiles**: type `10, 90` in Properties → P10/P90 marker lines on the
      plot and removable chips above it (click a chip to drop that percentile). Values
      must match what you'd read off the cumulative curve.
- [ ] **Statistics inside the plot**: set Statistics → "Inside the plot" (chips hide) or
      "Both" — the in-plot block shows the active stats incl. new Min/Max. Check it in a
      dark theme too (block background must follow the theme).
- [ ] **Universal by default**: a fresh histogram opens with NO Pick A/B rows and clicking
      the plot does nothing — enable "Show parameter pickers" in Properties to get the
      GR_MA/GR_SH picking workflow back. Your saved bar color / percentiles / etc. must
      survive closing and reopening the panel.

## P2-d — Log-view layout interaction (2026-07-19 #10)

- [ ] **Collapsible track headers**: ▤ in the log-view toolbar cycles full → compact
      (curve names as inline chips, no scale lines) → titles only. Headers also cap at
      ~a third of the pane and scroll inside, so a 15-curve track can't eat the screen.
      Try it on your densest layout.
- [ ] **Move/copy curves between tracks**: drag a curve name from one track header onto
      another track's header — the curve MOVES there (its color/scale/fill travel with
      it). Hold **Ctrl** while dropping to COPY instead (e.g. overlay NPHI on the GR
      track). Ctrl+Z undoes either.
- [ ] **Track borders**: ▦ in the toolbar — solid / dashed / none, width 1–4 px, theme
      color (follows light/dark) or a custom color. Default is a thin solid separator
      at every track boundary; check it looks right in dark themes too.
- [ ] **Readout follows ONE track now**: hovering shows only the curves of the track
      under the cursor (not all 15). CLICK a track to lock the readout to it (header
      highlights, click again to release) — then you can run the cursor over the whole
      layout while reading just that track's values.
- [ ] **Right-click log editing**: right-click on a track → "Edit CURVE…" for each of
      its curves. Ops: **Wireline shift** (whole-curve depth shift, resampled onto its
      own grid — NaN where it slides past the logged interval), **Set constant**,
      **Blank (erase)**, **Interpolate across** (bridge a bad interval linearly),
      **Scale a·v + b** (recalibration). Works on raw (GR/RHOB…), computed, and
      imported generic-store curves alike; every apply is ONE Ctrl+Z entry that
      restores the previous samples bit-exactly, and lands in the History panel.
      Suggested check: blank a washout interval on RHOB, interpolate across it,
      then Ctrl+Z twice — the original curve must come back exactly.

## P2-c — Well pin rework + multi-select (2026-07-19 #9)

- [ ] **Pin is now a mode, not a lock.** 📌 ON (default): clicking a well in Wells &
      Tops moves EVERY log view and plot to it — the old behavior. 📌 OFF: each view
      keeps the well it's showing; only the panel you're working in (the active tab)
      follows your clicks. Open two log views, turn the pin off, activate the second
      view, click different wells — only the second view changes. That's the
      side-by-side compare workflow.
- [ ] **The old lock is gone**: no more "Active well is locked" blocking when you
      click other wells, and no more weird interaction with a second wells pane.
- [ ] **Multi-select**: Ctrl-click wells to build a selection (highlighted with an
      accent edge, count shown in the Wells label), Shift-click for a range,
      ⇄ inverts within the visible list, plain click clears it. Then open any batch
      dialog (module run, Workflow Builder, Multimin, ML, Monte Carlo, Cutoffs &
      Summary) — the multi-selected wells come pre-ticked instead of just the active
      well.

## P2-b — Petrel-style tops editor + autocorrelation (2026-07-19 #4/#13)

- [ ] **Tops lines in the log view**: every log view now draws the well's tops as
      colored labeled lines across all tracks (like the correlation view). They track
      pan/zoom exactly and repaint on theme change.
- [ ] **🏷 edit mode** (log view toolbar): toggle it on, then — **click** an empty
      depth to add a top (name/depth/color dialog, name auto-uppercased); **drag** a
      line to move it (dashed preview while dragging); **double-click** a line to
      rename, change color, or delete. Mouse-wheel zoom still works while editing.
      Everything is undoable (Ctrl+Z) and instantly visible in Wells & Tops, other
      log views, and correlation.
- [ ] **Crossing warnings**: after any pick/move, SandiBumi compares this well's top
      order with every other well. If a pair is reversed vs the majority (e.g. TOP_B
      above TOP_A here but below it elsewhere), a ⚠ warning appears in the status bar
      naming the pair and the vote (e.g. "below it in 4 of 5 other wells").
- [ ] **Autocorrelate…** (Data tab): pick a top in the selected (source) well, choose
      the log (GR default), pattern window ±m and search range ±m — SandiBumi slides
      the source log shape over each target well (active group) and proposes the
      best-match depth with its correlation coefficient r. Strong matches (r ≥ 0.7)
      come pre-ticked; weak ones are dimmed for your judgment. **Apply** writes the
      ticked picks as ONE undoable batch. Try it on a marker you know — e.g. pick an
      MFS on GR in one Balam well and propagate to the rest, then check r values
      against your hand picks.

Issues you marked `[x]` that need real work (all in ROADMAP §4, P1/P2): well-pin
semantics rework, right-click lockdown (accidental refresh), TVD depth scale UI.
Everything you marked `[o]` has been cleared out of this file.

## Theme switch repaints everything immediately (2026-07-19)

- [ ] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [ ] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

## SandiMin — Geolog-parity mineral solver (2026-07-19, v2)

Rebuilt to Geolog Multimin / IP Mineral Solver conventions (spec extracted from your
Geolog-V14 helpset + IP2018 install → `docs/multimin_geolog_spec.md`, `docs/multimin_ip_spec.md`).

- [ ] **Advance → SandiMin…** now shows the full IP mineral list, grouped: 12 minerals (Calcite,
      Quartz, Dolomite, Orthoclase, Albite, Anhydrite, Halite, Gypsum, Pyrite, Siderite, Muscovite,
      Biotite), 6 clays (Glauconite, Kaolinite, Chlorite, Illite, Montmorillonite, Clay — each with
      an editable **CEC**), and 7 zone-typed fluids (Water Sxo / Water Sw / BoundWater / Oil Sxo /
      Oil Sw / Gas Sxo / Gas Sw; "flushed"/"unflushed" badges). Defaults: Quartz, Illite,
      Water Sxo, Water Sw.
- [ ] **Input logs**: 16 tools — Density, Neutron, Sonic, Total GR on by default; PEF, U, spectral
      Th/K/U, Vp, Vs, EPT, EATT, Sigma optional; **CT (Unflushed Conductivity, from RES_DEEP)** on
      by default and **CXO (from RXO)** optional — CT/CXO take a RESISTIVITY mnemonic; the backend
      converts to conductivity (dual-water linear: Ct^(1/w) row, w = 0.75m + 0.25n). Their σ is
      auto (0.03·C^(1/w)) unless you type one. **+ Add user-defined input** adds a custom log with
      its own endpoint column (default σ 0.015, Geolog's user-defined default).
- [ ] **Endpoints matrix**: editable per component×tool; unflushed-zone fluid cells show "—" for
      nuclear tools (only CT sees them — Geolog convention); CT/CXO cells show "auto"; per-row
      **Max** bound (fluids default 0.5, Geolog's cap).
- [ ] **Fluid properties** panel (visible when CT/CXO on): Rw@temp, Rmf@temp, formation temp, m, n,
      mud type. The preview line shows the computed w, Cw, Cmf, Cbw, α(x/u) and auto CT/CXO σ —
      sanity-check Cw against your Pickett Rw (Cw = 1/Rw@FT, mho/m).
- [ ] **Run** on a Balam well with RHOB+NPHI+DT+GR+RES*DEEP: writes VOL*\* per component +
      MM_PHIE, MM_PHIT, MM_SWE, MM_SWT (+ MM_SXOT, MM_MOVEDHC when both zones present),
      MM_VSH (clays + bound water), MM_RECON. Check: **Σ(minerals + unflushed fluids) ≈ 1**,
      **MM_SWT is sensible vs your sw_indo/RtC runs** (this is the new resistivity coupling —
      "resistivity convert to ct and cxo" as requested), and MM_RECON spikes where the model fails.
- [ ] Add **BoundWater** with Illite selected: VOL_BOUNDWATER should track ≈ 0.18×VOL_ILLITE at
      ~150°F (the Geolog dual-water bound-water constraint, k = 96·CEC·ρ/(T°C+298)).
- [ ] Add **Oil Sxo + Oil Sw** with CXO available: SXOT ≥ SWT in water-based mud (WATER MUD
      constraint) and MM_MOVEDHC = unflushed HC − flushed HC ≥ 0 across invaded pay.
- [ ] Requested upgrade (ROADMAP §4 P3): optional **nonlinear Sw equation iterated to
      convergence** inside the solve loop.

## ML suite (2026-07-19)

- [ ] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [ ] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [ ] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [ ] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [ ] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [ ] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [ ] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [ ] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
      Carlo, and Field Dashboard now sit in one row inside the Batch group.

## FACIES block track (2026-07-19)

- [ ] **Facies layout renders colored blocks.** Run Electrofacies on a well (Petrophysics →
      Electrofacies), then pick the new built-in "Facies" layout in the ribbon layout picker:
      the FACIES track should show solid colored blocks (same colors as the crossplot's
      categorical Z-coloring), with gaps where FACIES is missing. The track header shows a
      striped swatch and "class blocks" instead of a min/max scale.
- [ ] **Blocks survive pan/zoom and well switching**, and the header swatch toggles the whole
      track's visibility like any other curve.
- [ ] **Any discrete curve can be block-rendered.** Layout Properties → a curve's Fill
      dropdown now has "Facies blocks" — try it on FLAG_PAY in a custom layout.
- [ ] **Composite export shows the blocks.** Export a composite (SVG or PDF) with the Facies
      layout: the FACIES track should print as colored rectangles at true scale.

## Electrofacies — k-means (Phase 10 increment 1, 2026-07-18)

- [ ] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [ ] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [ ] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [ ] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
      curves** from the chosen result percentiles.

## Phase 8.5 — your method suite in core (remaining validations)

- [ ] **SSC — Sand-Silt-Clay (Advance tab)**: run on an LQR-style well with
      GRN + RHOB + NPHI (sandstone units). Check VSAND/VSILT/VDCL/VWCL, PHIT/PHIE/PHIFF,
      CBW/CWSH/BW, SWIRR_T/SWIRR_EFF and the `*_GR` GR-equivalent volumes against your
      Geolog run. Defaults are the LQR `.info` values (wet clay 2.3/0.6, dry clay 2.71,
      wet silt NPHI 0.3, DCLF_SI 0.1). Two deliberate deviations, flag if they matter:
      (1) `RANNORMAL(SWIRR_MIN·PHIT, 0.005)` is deterministic here; (2) the Loglan's
      NPHIMA limit 0.5–5 (a copy-paste of the RHOMA limit) is corrected to 0–1.
- [ ] **SSPW (Advance tab)**: the Loglan exec body wasn't on disk, so the
      arithmetic (PHIT from VSH-mixed dry matrix, CBW = VSH·VOL_CBW_SH,
      CAPBW = VSH·(PHIT_SH − VOL_CBW_SH), PHIE = PHIT − CBW, PHIFF, SWIRR floor) is
      **reconstructed from the spec — please validate against your Geolog "LAS PHIT
      PHIE" exports** and tell me any systematic difference; I'll adjust the equations.

## Phase 8b — report generator (2026-07-18)

- [ ] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [ ] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [ ] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [ ] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [ ] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [ ] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
