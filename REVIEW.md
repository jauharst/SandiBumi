# Review checklist — for Jauhar's click-through in `npm run tauri dev`

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time, marking items as you go.
Marks: **`[x]` = confirmed done** (works as described); `[ ]` = not yet checked. If something is
**wrong**, tell me directly (like your 540-well notes) and I'll fix it and log it in
**ROADMAP.md §4 (Field-review backlog)**.

## 2026-07-31 — Trained models are kept, named and re-runnable

Until now a model died with the run: you could not train on your cored wells and apply **that
same model** to the rest of the field later, and a delivered curve could not say which model
made it. Now it can.

- [ ] **Train and keep.** Petrophysics ▸ ML Models…, pick a supervised task (say regression,
      PHIT or PERM as the target), select your cored wells as training wells, and type a name in
      **Save model as** (e.g. `PERM_FROM_CORE`). Run. The status line should end with
      *"model saved as 'PERM_FROM_CORE'"* and it should appear in the **Saved models** list with
      its algorithm, its input curves, how many samples and from how many wells, and its size.
- [ ] **Apply it to wells it has never seen.** Change the well scope to the uncored wells, then
      press **Apply to scope** on the saved model. Nothing is refitted — check the Processing
      monitor says "apply saved model", not "training".
- [ ] **The result is traceable.** The new curves' log set records `ml:apply:<model name>` with
      the model id, so months later you can answer "which model produced this?".
- [ ] **A missing input is named.** Apply a model to a well that lacks one of its input curves —
      it should tell you **which curve by name**, not just "missing input curve data".
- [ ] **Retraining does not overwrite.** Run again with the same name: it should save as
      `..._1` and say so. A model an existing delivered curve was made with must never be
      silently replaced.
- [ ] **The scaler went with it.** If you trained with "Standardize" on, the applied curve should
      look right on wells whose GR/RHOB ranges differ from the training wells. (This is the
      subtle one — re-standardizing on the new wells would give a different, wrong answer.)
- [ ] **Rename and Delete** work, and Delete asks first. Deleting a model does not remove curves
      it already produced — but they can no longer be reproduced from it.
- [ ] **Only supervised models are offered.** The "Save model as" field disappears for
      clustering and reduction, because those are fitted on the very wells they are applied to.
- [ ] **Project size.** A random forest can be a few MB. Check the size column; if your project
      grows more than you like, Data ▸ Tools ▸ Compact Project still works.

## 2026-07-31 — The field as an asset-team deck

Last of the office deliverables. **Plot ▸ Deliverables ▸ Deck…** builds a PowerPoint from the
data — you chose matplotlib figures over pasted composite pages, so that is what it does.

- [ ] **Export a deck.** Pick a scope, a title, who is presenting, and the cutoff level
      (**PAY** by default). Open it in PowerPoint. Seven-ish slides: title, scope and cutoffs,
      field roll-up by zone, net + HPV per zone, N/G–PHIE–SWE distributions, well ranking, and
      any well that produced nothing.
- [ ] **The box plots should match the Field Dashboard.** They are the same statistics — the
      app computes them and matplotlib only draws them, precisely so the two can't disagree.
      Compare a zone's PHIE box against the dashboard. **If they differ, tell me.**
- [ ] **Each box says how many wells are behind it** (`n=` under the label). A box from three
      wells is not the same statement as one from ninety.
- [ ] **A zone nobody interpreted gets no bar — not a zero bar.** It still gets its axis label
      so you can see it exists. Check this on a zone you know is uninterpreted.
- [ ] **The cutoff level is stated on the title slide.** A deck speaks about one level; SAND and
      RESERVOIR stay in the workbook. Try switching to RESERVOIR and confirm the whole deck
      follows.
- [ ] **Long tables continue on more slides** ("1 of 3") rather than shrinking. If your field
      has many zones, check the table is still readable from the back of a room.
- [ ] **The well ranking says what it cut** ("Top 20 of 44 interpreted wells"). A silent top-N
      would read as the whole field.
- [ ] **The last slide names the wells that produced nothing.** That is the counterpart to
      every average on the slides before it.
- [ ] **Everything is editable** — real PowerPoint tables and text, and the charts are pictures
      you can resize or replace.
- [ ] **Without the packages.** If python-pptx or matplotlib is missing, the dialog names which
      one before the save dialog. You have both.

## 2026-07-31 — The report as an editable Word document (+ an encoding bug fixed)

Second of the office deliverables. The report pane now has **Save Word…** next to Save PDF…,
and the **Batch** button has a format select beside it (`as PDF` / `as Word`) so a whole field
can go out either way.

- [ ] **Save Word on one well.** Open the Report pane, set your title/author/methodology as
      usual, press **Save Word…**. Note you do NOT have to press Render first — the document
      carries no log plots, so there is nothing to preview.
- [ ] **It is genuinely editable.** Open the `.docx` and change the methodology wording, drop
      in your client's letterhead, restyle the tables. That's the whole point of this format —
      the PDF stays the deliverable that must not be altered.
- [ ] **The tables match the PDF.** Cover, methodology, zone parameters (zone name and depths
      printed once per zone, not repeated down every parameter row), pay summary with the
      cutoffs in the heading. Export both for the same well and compare — they read from the
      same numbers, so any disagreement is a bug.
- [ ] **A zone with no parameters is still listed.** Dropping it would tell a client the zone
      was not evaluated when it simply took the defaults.
- [ ] **A dash, not a blank, in the document.** Where the workbook leaves an uninterpreted cell
      empty, the Word document prints "-" like the PDF does. That difference is deliberate:
      Excel's arithmetic skips an empty cell, a document has no arithmetic and your eye needs
      the mark.
- [ ] **No composite log pages in the Word file** — on purpose, and the document says so at the
      end. A composite at 1:200 stops being at 1:200 the moment somebody drags its corner in
      Word. If you want them in there anyway, tell me and I'll add a rasterized appendix.
- [ ] **Nothing is written back.** Unlike the PDF path (which writes FLAG curves as it renders),
      the Word export touches nothing in the project.
- [ ] **Batch as Word.** Set the select to `as Word`, pick a folder, and check you get one
      `<WELL>_report.docx` per well in scope.
- [ ] **Names with special characters.** This one is a **bug fix worth testing**: import a
      picture whose file path or folder contains a non-ASCII character (an en dash, `é`, or an
      Indonesian folder name), and check it now imports. Before this, the import failed with
      "No such file or directory" naming a filename you never had — text was being read from
      the wrong character set on the way into Python. The same bug would have mangled a well
      name in the Word report.

## 2026-07-31 — The study as an Excel workbook

First of the office deliverables. Until now `export.rs` wrote LAS and everything else left as
a PDF, an SVG or a flat CSV — so the table an asset team actually works in was re-typed by
hand. **Plot ▸ Deliverables ▸ Workbook…** writes it directly.

- [ ] **Export a workbook.** Pick a scope (group / ★ pinned / selection / all), check the
      cutoffs it opened with — they should be **the same numbers the pay summary and the report
      use**, because all three read one saved default. Press Export, choose a filename, open it
      in Excel.
- [ ] **The numbers are numbers.** Click a Net or PHIE cell: the formula bar should show
      `12.5` / `0.185`, not text. Sort, filter and pivot the Pay Summary sheet — if any of that
      refuses to work, a column came through as text and I want to know.
- [ ] **A blank is not a zero.** Find a well you have NOT interpreted yet (no VSH/PHIE/SWE).
      Its net, N/G, PHIE, SWE and HPV cells must be **empty**, while Gross still shows a number
      (geometry is known either way) and Samples shows 0. Select the Net column: Excel's status
      bar average must ignore those rows. **This is the one thing I most want checked** — a 0.00
      there would quietly drag down a field average.
- [ ] **The Summary sheet is the audit trail.** It should name the cutoffs actually used, the
      depth unit, the export time, and — if any well produced nothing — list those wells by name
      under "Well without results". A well that contributed nothing must never just be missing.
- [ ] **Two N/G columns on the Field Summary sheet.** `N/G (field)` is Σnet/Σgross (the
      volumetric ratio for a resource number); `Mean N/G` is the average of the per-well values,
      which is what the **Field Dashboard** shows. Compare a zone against the dashboard: Mean
      N/G, PHIE and SWE should match it. If they do not, tell me — they read the same rows.
- [ ] **Zones read shallow to deep**, not alphabetically, on the Field Summary sheet.
- [ ] **PAY rows are tinted** on both table sheets, so the pay level stands out from SAND and
      RESERVOIR — all three levels are exported, not just PAY.
- [ ] **Nothing is written back.** Export a workbook, then check the Processing history and the
      Wells pane: no new FLAG curves, no new log-set version. Saving a spreadsheet must not
      count as an interpretation run.
- [ ] **Zone Parameters sheet.** The interval parameters your interpretation used, one row each.
      Zone `*` is the whole-well default. Check a well where you set a per-zone `RW` or `M`.
- [ ] **Without xlsxwriter.** If Python or the package is missing, the dialog says so **before**
      the save dialog and names the interpreter to `pip install` into. It should never fail
      after you have already chosen a filename.
- [ ] **Field scale.** Try it on a few hundred wells. It runs as a job, so the **Processing**
      monitor should show it while it works.

## 2026-07-31 — Pictures in their own track (thin sections, core photos)

Your ask: *"images in separate tracks, such petrography thin section, core photo, or any
picture format that can be adjustable (later we should have capablites to digitize it as
well)"*. Done for the DISPLAY half; digitizing is deliberately a later phase.

- [ ] **Import a folder of thin sections.** **Data ▸ Import Data ▸ Import Images…** with a
      well selected, pick several files. The wizard lists every file with its true pixel size
      and **the depth it read from the file name** — nothing is stored until you press Import.
      Check the guesses: `BLSO-01_1523.50.jpg` should read 1523.50, and a plain `BLSO-01.jpg`
      should read NOTHING (an amber "required" box), because a two-digit well number must
      never be mistaken for a depth. Fix any depth in the table before importing.
- [ ] **A photographed interval.** A file named `..._1523.5-1524.0.jpg` should come in with
      BOTH a depth and a base. You can also type a base by hand. Leave the base empty for a
      thin section — a plug has no thickness, and the empty cell is what says so.
- [ ] **Show them.** Right-click a log view ▸ **Layout Properties…**, add a track, set
      **Track type = Images**, then **＋ Add image series** and pick your dataset. The plates
      appear at their depths with a leader line to the track edge.
- [ ] **Adjustable, as you asked.** In that same editor try: **Width of track** (how big the
      plate is), **Align** left/centre/right, **Placement** — *Anchored at depth* (fixed size,
      centred on the sample) vs *Scaled to interval* (the picture spans its own top-to-base,
      only meaningful when it has a base depth), and for a scaled one **Fit** *Whole picture*
      vs *Fill and crop*. Nothing ever squashes the picture out of shape — tell me if you
      ever see a stretched plate.
- [ ] **Overlapping plates.** Zoom out until two thin sections would collide. The deeper one
      **disappears and leaves a short tick** at its true depth rather than sliding down to fit.
      Zoom back in and it returns. That is deliberate — say if you would rather they stacked.
- [ ] **Print it.** **Plot ▸ Composite…** with that layout — the plates must appear in the PDF
      and in the SVG at the same place and size as on screen. Open the SVG somewhere else
      (a browser) to confirm the pictures travel INSIDE the file, not as broken links.
- [ ] **A second delivery does not double the plates.** Import the same folder again with the
      same delivery name. It should land as `NAME_1`, become the live one, and the track must
      show **one** set of plates, not two. **Data ▸ Tools ▸ Data Sets…** has a new **Images**
      section — switch back to the first delivery and the track follows. The Wells pane ▸
      twisty also lists **Images** per well; double-click switches the live one, and expanding
      a delivery lists each plate with its depth and size.
- [ ] **Project size is visible.** The Data Sets dialog and the tree both show MB per delivery
      — the only store where the cost is worth showing. Stored pictures are capped at 2400 px
      on the long edge by default; the wizard lets you raise it (or set 0 for full resolution)
      if you need to zoom further, at the cost of a much larger project file. Tell me if 2400
      is too soft for your thin sections.
- [ ] **TIFF.** If your petrographer delivers TIFF, it needs Pillow (`pip install pillow`).
      With Pillow present TIFF imports and displays normally. Without it, the wizard says so by
      name rather than failing quietly, and a non-JPEG prints as a **labelled frame** in the
      PDF so a deliverable can be checked against the delivery list.
- [ ] **All three languages** translate the new labels (Import Images…, Images, Placement,
      Align, Fit, Frame, Caption…); technical terms stay English as always.

## 2026-07-30 — Quick-access buttons become labelled Project-tab tools

Your ask: *"those QAT buttons should become labelled tools, together with performance and
processing button moved from petrophysics tabs"*. Done — and the icon strip left of the ribbon
tabs is **gone**, not duplicated.

- [ ] **The icon strip is gone and nothing was lost.** Launch the app: there is no row of small
      icons left of the **Project / Data / Petrophysics / …** tabs. Open **Project** — all seven
      of those buttons are there with words under them:
      **Project** — Open Project… / New Project… / **Save Project As…** / Recent ▾
      **Session** — Save Session… / Open Session…
      **Edit** — Undo / Redo
      **Monitor** — History / **Processing** / **Performance**
      **Appearance** — Theme · **Language** · **Help** — Help
      (The tabstrip is 24px tall and the ribbon body is 80px — that height difference is the
      whole reason these could not carry captions where they used to live.)
- [ ] **Processing and Performance are no longer in Petrophysics.** Open **Petrophysics ▸ Batch**:
      it now holds Workflow… / Monte Carlo… / Field Dashboard… only. Both moved buttons open the
      same panels as before, from **Project ▸ Monitor**. They watch the whole application rather
      than a petrophysics run, which is why they sit with History.
- [ ] **Undo still reads what it will undo.** Make an undoable edit (add a top, edit a curve
      value, shift core). On **Project ▸ Edit**, **Undo** enables and its tooltip names the action
      — e.g. "Undo add top UAT_TOP (Ctrl+Z)"; after clicking, **Redo** enables with the matching
      label. **Ctrl+Z / Ctrl+Y are unchanged and are still the fast path** — the buttons exist to
      make the action readable, not to replace the shortcut.
- [ ] **The unsaved warning is still visible from wherever you are.** This one needs a deliberate
      look: Save Session… now lives *inside* the Project tab, so its red dot alone would only be
      visible to someone who already went looking for it. Sit on the **Petrophysics** tab, edit a
      log view (drag a track wider) — an **amber dot appears on the Project TAB itself** without
      you switching to it (hover: "Unsaved changes — Project ▸ Session ▸ Save Session…"). The tab
      must NOT change width when the dot appears. Save a session; both dots clear.
- [ ] **The ribbon overflow arrows work now.** These have been broken since they were written and
      nothing was ever wide enough to reveal it — Project is the first tab that is. If your window
      is narrower than ~1470px you will see a **›** box at the right edge of the Project tab:
      click it and the ribbon really scrolls (Help comes into view, a **‹** appears, the **›**
      hides); click **‹** to come back. It jumps rather than glides — that is deliberate: smooth
      scrolling is silently a no-op on this element, so an unanimated scroll that works beats a
      pretty one that does not. **Tell me if the overflow bothers you** — on a 1366 laptop the
      Project tab will always need one arrow-click to reach Help, and I can win back about 100px
      by merging Language into Appearance and folding Help into Monitor.
- [ ] **Bahasa / Basa Sunda / Basa Jawa cover the new labels.** Switch language on the Project
      tab: Undo/Redo/History/Processing/Performance and the Session, Edit and Monitor captions all
      translate, and switching back to English restores the exact original wording.
- [ ] **`docs/manual_test_plan.md` was updated with this** — every step that said "QAT" or
      "quick-access bar" now names the real ribbon path (T-SHELL-01/-05/-07/-10/-12/-13/-14 and
      ~20 more). Since you are working through that plan, it should no longer send you looking
      for buttons that moved.

## Round 99 — Depth units, increment 2: the Pc fix and the m/ft view toggle (2026-07-29)

**1. The saturation-height error is fixed.** `pc = 0.433 psi/ft/SG · Δρ · h` is per FOOT of
column, but `satheight.rs` and `shf_fit.rs` multiplied the height by 3.28084 unconditionally,
assuming it arrived in metres. On your foot-declared Rokan projects that scaled an
already-foot height and returned a Pc **3.28× too high**.

The test that pins it takes one physical well described twice — 100 m above the FWL in a
metre project, the identical 328.084 ft in a foot project — and requires the same Sw. Against
the old formula it fails with **Sw 0.2685 vs 0.1670**: a 38% error in water saturation that
computed, plotted and would have shipped. It now passes.

`ModuleContext` carries a typed `depth_unit` rather than a magic options key, deliberately: a
missing string key would silently mean metres, which is the failure mode itself. `FT_PER_M` is
deleted rather than left unused — it *was* the assumption.

**2. The m/ft view toggle you asked for.** A small **m / ft** button in each Log View's own
toolbar, beside the zoom controls. It changes what you READ and never touches stored data —
that separation is the whole point, and the button turns accent-coloured whenever the numbers
on screen are converted rather than stored, so a converted view can't be mistaken for the
real ones. The choice persists per machine and defaults to the project's own unit, so doing
nothing shows depths exactly as your files delivered them.

**3. Print scales no longer lie on foot projects.** `PX_PER_UNIT_1_1` derived px-per-depth-unit
from 96 px/in ÷ 0.0254 m/in — metres, always — so every named 1:N scale on a foot project was
off by 3.28×. It now reads the project unit: 3779.53 px/m or exactly 1152 px/ft (96 px/in ×
12). Verified that **1:200 in a 400 px pane shows 21.17 m in a metric project and 69.44 ft in
a foot one — the same physical section.** Note the scale follows the STORED unit, not the
display toggle: "1:200" is a ratio of rock to paper, so it can't depend on which unit you
happen to be reading.

**4. Re-declaring a project's unit is refused once it holds wells** — their depths are already
stored in the old unit, so a re-declaration would silently reinterpret every one of them
(a 2,438 m well would start reading as 2,438 ft). The error says so and points at the display
toggle instead. Converting stored data would be a real migration, not a settings change.

**Verified:** `cargo test` 384 passed / 0 failed; `npx tsc --noEmit` and `cargo check` clean;
conversion, print-scale and toggle behaviour driven live in the browser (8000/8050/8100 ft →
2438/2454/2469 m on the depth axis, stored unit unchanged, button state and tooltip correct
in both directions).

- [ ] **Try:** open a foot project and check the depth axis reads feet, then click **ft → m** in the Log View toolbar. Depths convert, the button turns accent-coloured, and the status line says the data is unchanged.
- [ ] **Try:** with the display in metres, check the **1:N** dropdown still frames the same physical section it did in feet — the scale must not move when you change what you're reading.
- [ ] **Try:** run **sw_height** (Leverett) on a foot project against a well you know. This is the number that was 3.28× wrong; if the Sw still looks off, tell me before trusting it.
- [ ] **Try:** the depth readout under the cursor now carries its unit ("Depth: 8000.0 ft").
- [ ] **Still metres-only, increment 3:** tops/zones panels, composite scale bar, report pages, dashboard depth columns and depth-coloured plot axes still print raw stored depths without conversion. They are correct on a project whose display unit equals its stored unit — which is the default — but they do not yet follow the toggle.

## Round 98 — Depth units, increment 1: the project declares one, imports convert to it (2026-07-29)

Your instinct was right, and it lands on an **already-verified audit finding** (engineering
review **F2e**, "fix-now", high confidence) that nobody had actioned. The LAS index unit was
being parsed at `parsers.rs` and **thrown away** under `#[allow(dead_code)]`, and `curves.rs`
FAMILIES has no DEPTH entry — so `convert_to_canonical` never touched the index. A foot-indexed
Rokan/Caltex LAS put its raw foot numbers in the same column as a metric Mahakam well, and the
import was reported as clean. A top at 8,000 (ft) and one at 2,438 (m) in the *same formation*
sat 5,500 units apart, and correlation, contact planes and the tops slide window compared them
as if that were real.

Per your two decisions: **the project declares its depth unit and imports must match**, and
**depth first, curve units later**.

**What ships now (the storage layer):**

- `src-tauri/src/units.rs` — one place that knows metres from feet. Exact international foot
  (0.3048; the US survey foot differs by 2 ppm ≈ 5 mm over a 2,500 m well, so it is not
  modelled). Unrecognized unit strings return `None` rather than a guess, because guessing is
  the exact failure this exists to stop.
- **Project setting**: stored as a `documents` row, so no schema migration. A **fresh project
  adopts the unit of its first import** — the common case needs no decision from you at all.
- **Import reconciliation**: a file matching the project stores as-is and says nothing; a file
  in the *other* unit is converted and the import is flagged; a file declaring no unit is
  assumed and flagged. Every case except a clean match produces a note in the import warning
  you already see.
- Both stores convert **identically** — the generic-store loader re-reads the same file, so it
  had to apply the same conversion or the two would hold the same curves 3.28× apart.
- `wells.depth_unit` records what the stored numbers mean, next to the data itself.
- Both `#[allow(dead_code)]` attributes are **gone**, so the compiler can never again hide the
  fact that nothing reads the index unit.

**Verified:** `cargo test` 383 passed / 0 failed, including 5 new unit tests (unit-string
spellings that occur in real field LAS, 8000 ft = 2438.4 m exactly and back, NaN preserved
through conversion, and every project×file unit combination).

**One thing this does NOT yet fix, stated plainly.** A project declared in **feet** still has
two places that assume metres:
`satheight.rs:181` and `shf_fit.rs:897/1069/1284` compute `pc = 0.433·Δρ·(h · 3.28084)`, i.e.
they assume the height above free water arrived in metres — so **Pc is 3.28× off on a
foot-declared project**; and `LogCanvasRenderer.PX_PER_UNIT_1_1` derives the true 1:N print
scale from 96 px/in ÷ 0.0254 m/in, so every named scale is mislabelled by the same factor.
A **metric** project is correct today, and mixed-unit imports are now correct because they
convert. Both sites are increment 2, together with the view toggle.

- [ ] **Try:** import a metric LAS into a fresh project, then a foot-indexed one. The second should import with a note that the depth index was converted, and its tops should line up with the first well's in correlation rather than sitting thousands of units away.
- [ ] **Try:** import a LAS whose `~C` block declares no index unit — expect the note "this file declares no index unit — depths assumed to be m".
**Answered (2026-07-29): feet.** Rokan/Central-Sumatra projects will be declared in FEET, keeping
the depths you know. That makes the increment-2 Pc fix **live rather than theoretical** — a
foot-declared project returns a saturation-height Pc 3.28× too high until it lands. You have
deferred it to your manual-test pass of the saturation-height section, which is a reasonable
call because it surfaces in testing rather than in a deliverable. The one rule that follows:
**do not trust or ship an SHF/`sw_height` result from a foot-declared project until increment 2
is in.** Metric projects are unaffected.

## Round 97 — SHELL field-test fixes: Pin OFF, plot right-click, repeat reload key (2026-07-29)

From your run through **Section SHELL** of `docs/manual_test_plan.md` — 16 of 18 passed;
T-SHELL-16 and T-SHELL-17 failed. Three separate causes, all fixed.

**1. "Pin off, never follow well even for active panel"** — the real bug of the three, and a
good catch. Pin OFF is meant to mean *only the active panel follows*, and it asked
dockview "is this pane active?" **at the moment the selection changed**. But a well is
selected by **clicking it in the Wells tree**, and that click makes the *tree* the active
pane — so at that instant no viewer was active and **nothing followed at all**. The pin
effectively became "freeze everything".

The gate now reads a **working pane** (`src/ui/activeViewer.ts`): the last *viewer* you
clicked into. Browsing panes (Wells, Tops, Inspector) never claim the role, so clicking a
well can't steal it. If no viewer has ever been activated the first one to ask claims it,
so "pin off" can never again degrade to "nobody follows". Applies to log views, plots and
the well-bound tool panes alike.

**2. "right click in xplot showed properties instead of option like in log view"** — the
plot canvases swallowed right-click to open Properties directly, which cost them the pane
menu every other panel has (Split right/down, Float, Maximize, image export, Close).
Right-click on a plot now opens the **normal pane menu with `Properties…` as its first
entry**, so both are one click away. Double-click still opens Properties on histogram and
crossplot; Pickett keeps its ⚙ toolbar button (its double-click is reserved for picks).

**3. "ctrl+R does nothing"** — this one was half a documentation defect. The step said
"Press F5, then Ctrl+R", so by the time Ctrl+R was pressed the F5 dialog was **already
open** — and the guard returned silently rather than opening a second one. Correct
behaviour, invisible feedback. A repeat reload key now **pulses the open dialog** instead.
Two related hardenings while in there: the key is matched on physical `code` as well as
`key` (a non-US layout would have missed it), and **Escape** now closes the confirm even
after focus has left it (it was bound to the dialog, so one stray Tab left Escape dead).
The step-4 wording in the test plan was ambiguous and has been rewritten.

**Verified:** `npx tsc --noEmit` clean, `npm run build` clean, and the reload guard driven
through five live scenarios in the browser (Ctrl+R alone → dialog; foreign-layout `KeyR` →
dialog; F5-then-Ctrl+R → one dialog, pulsed; Escape with focus outside → closes; Cancel →
closes). The working-pane tracker's semantics were unit-exercised live. What I could **not**
drive from a browser is a real dockview activation with real wells — that is exactly what
the re-test below covers.

- [ ] **Try:** two Log Views, pin OFF. Click into Log View 1, select well C in the tree → **only Log View 1** moves. Click into Log View 2, select well A → **only Log View 2** moves. This is the failure you reported; it should now be impossible for nothing to move.
- [ ] **Try:** with pin OFF, open a Crossplot and a Log View side by side. Click the crossplot, pick a well — the crossplot follows and the log view holds. Plots obey the same working-pane rule now.
- [ ] **Try:** right-click a Crossplot, a Histogram and a Pickett canvas → pane menu with **Properties…** on top, then export items, then Split right / Split down / Float / Close. Compare against a right-click in the Log View.
- [ ] **Try:** F5 → Escape. Ctrl+R → Cancel. F5 then Ctrl+R while the dialog is up → one dialog, pulsing. **This is the one to check first — if Ctrl+R on its own still does nothing, the cause is not what I diagnosed and I need to know.**
- [ ] **Try:** the two re-tests above are T-SHELL-16 and T-SHELL-17 — your original Fail marks are left in place as the record; re-run those two rows in the xlsx.

## Round 96 — Non-colour design tokens, and a client-skin colour bug found on the way (2026-07-29)

**The important part of this round is not the polish — it is a pre-existing bug the polish
exposed.** On any machine whose **OS is set to dark** (yours is), the five *light* client
skins — Pertamina, Halliburton, Schlumberger, LAPI-ITB, white — kept their white panels but
silently picked up the **dark** `--qc-*` status colours. Measured contrast of the Results-QC
scorecard against the white panel:

| Token | Was | Now | WCAG AA (4.5:1) |
|---|---|---|---|
| `--qc-ok` | 2.24:1 | **5.13:1** | fail → **pass** |
| `--qc-alert` | 3.49:1 | **5.62:1** | fail → **pass** |
| `--qc-warn` | 2.19:1 | 3.78:1 | fail → still fail |

Cause: `@media (prefers-color-scheme: dark)` was scoped to `:root:not([data-theme="light"])`,
but `theme.ts` **deletes** the attribute for "system" and **sets** it for every other choice —
so the block also caught the explicitly-chosen light brand skins. Now `:root:not([data-theme])`
— "no theme chosen at all" — so an explicit choice ignores the OS preference entirely. The
comment in `:root` claiming the skins "inherit these unchanged" is finally true.

Note `--qc-warn` at 3.78:1 still misses AA. That is the light theme's own designed amber
(`#c07000`), not a regression, and darkening a QC semantic colour is your call — flag it if
you want it changed.

The polish itself: colour was the only axis this stylesheet ever tokenised, so radius, type
size, motion and elevation had been decided per rule by hand — **12 distinct corner radii and
11 font sizes**, four of them half-pixel (11.5/10.5/12.5/9.5px, 45 declarations) which land off
the pixel grid and render soft. Added `--r-*`, `--s-*`, `--fs-*`, `--dur-*`/`--ease`, `--el-*`
and `--focus-ring`, then swept **104 radius** and **201 font-size** literals onto them.
Chips and badges became true pills; dockview's own `--dv-border-radius` / tab font-size /
floating shadow now read from the same scale.

Motion and focus are **one block** near the top of the file, not a line added to forty rules —
reviewable and revertable in one place. Two properties keep it safe: `:where()` has zero
specificity so every existing rule still wins (`.btn` verifiably kept its own 0.12s), and only
**paint** properties are transitioned — never transform/width/position — so dockview drags,
sash resizing and canvas panning stay instant. Buttons got a focus ring (they had none; form
fields already did, and were left alone). Dialogs fade in on opacity only — no transform,
because the modal is drag-positioned and a transform animation would fight an immediate grab.

Verified: ribbon geometry **byte-identical** before and after (112px ribbon / 80px panel /
24px QAT, A/B'd against the stashed original at a fixed viewport — an earlier 122px reading was
a viewport artifact, not a reflow); **663 elements swept for unresolved `var()`, zero found**
(an undefined token would silently collapse radius to 0); all 7 themes checked; gate green
378/0/7.

- [ ] **Try:** switch to a **client skin** (Project ▸ Appearance ▸ Pertamina) and open a
  Results-QC scorecard. The pass/warn/fail colours should now be legible dark green/amber/red
  on white, not the pale dark-theme versions. This is the one item with a real deliverable
  consequence.
- [ ] **Try:** hover the ribbon tabs and buttons — they should ease rather than snap. If
  anything feels laggy against real field data, the whole motion layer is one block in
  `styles.css` and can be cut without touching anything else.
- [ ] **Try:** drag a dockview panel between windows and pan a log view. Both must still feel
  instant — geometry was deliberately excluded from the transitions.
- [ ] **Try:** confirm the tighter type reads as *cleaner* and not *cramped* at your normal
  window size, on a dense panel (Monte Carlo params, the multimin endpoint matrix).

## Round 95 — SSC gas conditioning changed the numbers; the stale test that hid it is fixed (2026-07-29)

**This one needs your eyes on real data — SSC output values moved.** Your `d1f0c1e` commit
re-aligned `ssc.rs` to the Loglan reference, and one change is numerical, not cosmetic: the
gas/HC conditioning now pulls a point onto the sand base line at the **RMS midpoint**
(`sqrt((φD²+φN²)/2)`, matching `sspw.lls`'s gas branch) instead of the old 1.6-weighted form,
which overshot the midpoint and inverted the density-neutron crossover. Any gas-affected
sample will therefore report a different PHIT/PHIE than before. Per `RELEASE.md`, that is a
"numbers that changed" event.

That commit also left the gate red: `ssc_swirr_floor_pads_capillary_water` asserted
`SWIRR_T >= SWIRR_MIN`, which contradicted **its own name** and both references. The floor
(`ssc.rs` `if ... bw / phit < swirr_min`) pads **CWSH** — capillary water — raising BW;
`SWIRR_T` is deliberately the *pre-conditioning* ratio (`.lls` 213-216, and
`docs/method_ssc_sspw.md` §8 computes SWIRR first, then lists the conditioning). So the code
matched the spec and the test was the stale artifact. I did not touch any physics.

The test now pins **both** halves of that contract — the floor must raise CWSH and lift
BW/PHIT to SWIRR_MIN, *and* SWIRR_T must stay the pre-conditioning value — plus a guard that
the fixture actually starts below the floor, so it can't pass vacuously. Gate: green with
nothing stashed.

- [ ] **Try:** re-run SSC on a well with a known gas effect and compare PHIT/PHIE against
  your previous run (or the reference-suite LAS export). The non-gas samples should be
  unchanged; gas-affected ones will differ, and the new values are the ones that match the
  Loglan. If they don't match the reference export, tell me — that is a real finding.

## Round 94 — R-C: closing the app no longer risks losing the writes since the last checkpoint (2026-07-29)

Found by the packaged-build verification, not by code review — and it is the biggest catch of
the session. Tauri exits through `std::process::exit`, which skips Rust destructors, so the
DuckDB connection **never closed cleanly on any exit**: every close — including a plain window
✕ — abandoned a live WAL. Reproduced twice against the packaged app: import a 20-row LAS,
close with ✕, relaunch → the WAL fails replay, `init_db_resilient` moves it aside as
`.corrupt-backup-<ts>`, and the import is **silently gone** (`Wells (0)`). Writes below
DuckDB's auto-checkpoint threshold live only in the WAL, so the writes at risk are exactly the
small, recent ones: an import, a parameter edit, a tops pick made just before closing. This
also explains the WAL-corruption plague CLAUDE.md attributes to `tauri dev` force-kills — every
close abandoned a WAL; the force-kills were just the ones caught badly enough to notice.

Fix: `lib.rs` now runs the app with a `RunEvent::Exit` handler that locks the connection and
executes `CHECKPOINT` — every graceful exit flushes the WAL into the project file while the
process still can. Force-kills stay covered by `init_db_resilient` exactly as before.

Verified end-to-end on the packaged exe (isolated scratch project, `SANDIBUMI_CONFIG_DIR`):
same import-close-relaunch sequence → after close there is **no `.wal` at all** beside the
project, relaunch lists the imported well, and no new corrupt-backup appears. Full green gate:
`GATE GREEN in 68s` (378/0/7, SSC WIP stash-roundtripped).

- [ ] **Try (= T-SHIP-07 in `docs/manual_test_plan.md`):** in a COPY of a project, import one
  LAS, close the app with ✕ immediately, look beside the `.duckdb`: no `.wal` should remain.
  Reopen — the imported well must still be there and no new `.corrupt-backup-*` file appears.

## Round 93 — R-B: a destructive migration now backs up the project file first (2026-07-29)

Requirement R-B from `docs/RELEASE.md` §3.2, the sibling of Round 92's R-A and the other
1.0-gate item. The finding: the PK-drop migration (the one that made 100-well chains 2.4×
faster) **rebuilds the whole `computed_curves` table in place** — `DROP TABLE` mid-sequence —
with no recoverable copy. On a field-scale file, a crash mid-rebuild loses computed results
with nothing to fall back to.

Now: when that migration is actually going to run (and only then — additive migrations like
the R-A stamp and the generic-store backfill are exempt, so backups stay meaningful), the
project is first copied beside itself as `<name>.pre-1-backup.duckdb` and the launch log says
so. Two honesty properties: a **failed backup aborts the migration** (the un-migrated file
still opens fine — the PK only slows writes — so refusing costs nothing, while proceeding
would break the exact promise), and an **existing backup is never overwritten** (collision →
timestamped name, the WAL-recovery convention). One Windows reality the test caught: DuckDB
holds its file with exclusive sharing, so a filesystem copy of an open project is impossible —
the copy is made by the engine itself (`ATTACH` + `COPY FROM DATABASE`), which also preserves
the schema *with* the PK, so the backup is provably the pre-migration file.

Verified: 2 new `db.rs` tests against real temp files — the destructive path writes the backup
first (openable, PK intact, both rows present), a no-op open writes nothing, a fresh project
never accumulates backups, and a name collision takes a new name. Full green gate:
`GATE GREEN in 39s`, **378 passed / 0 failed / 7 ignored** (SSC WIP stash-roundtripped as
before).

- [ ] **Try:** open your real project — since increment 5 already migrated it, the pass
  condition is **absence**: no new `*-backup.duckdb` file beside it, launch not slower.
  To see it fire, open any pre-2026-07-19 project copy that still has the old PK: a
  `<name>.pre-1-backup.duckdb` appears beside it and the console log announces it before
  the rebuild. (Full list of session-wide manual checks: `docs/manual_check_plan.md`.)

## Round 92 — R-A: the project file now carries a format stamp, and an older build refuses a newer file by name (2026-07-29)

Requirement R-A from `docs/RELEASE.md` §3.1 (on the 1.0 gate; the doc arrives with PR #2). The
finding behind it: the project `.duckdb` carried **no format version anywhere** — every table is
`CREATE TABLE IF NOT EXISTS`, read by name — so an older SandiBumi opening a file written by a
newer one would open it, find the tables it knows, silently ignore the rest, and present a partial
project as the whole thing. Months of interpretation, shown with pieces missing, no warning. That
is the cardinal rule (a degraded result presented as clean) with a whole project as the blast
radius, and it was the *default* behaviour.

Now: a `project_meta` table (`format_version`, `written_by`) is stamped into every project on
open. `db::FORMAT_VERSION` starts at 1; the check runs **before** `create_schema` on purpose,
because `CREATE TABLE IF NOT EXISTS` is itself a mutation and a newer file must be refused
*untouched*. Three cases: no stamp (fresh file or legacy project) → stamp it, additive; stamp ≤
current → open normally, re-stamp if older; stamp > current → **refuse**, naming the file's
format, the app that wrote it, and what to do ("this project was written by SandiBumi X (file
format N); this build reads format 1 and lower - upgrade SandiBumi to open it (the file was left
unmodified)"). A missing or unparsable version row counts as legacy, never as newer — refusal
requires positive evidence. The refusal message contains no "WAL", so `init_db_resilient` can
never mistake it for corruption and move a healthy newer file's WAL aside.

Verified: 3 new tests in `db.rs` — fresh project stamped with format 1 + `written_by SandiBumi
0.1.0`; a legacy pre-stamp project (full schema, no meta) is stamped on open; a future-format
file (stamp 999, deliberately without the current schema) is refused with all three message parts
AND left byte-honest — `wells` still absent after (proving `create_schema` never ran), stamp
still 999. Full green gate: `GATE GREEN in 47s`, **376 passed / 0 failed / 7 ignored** (SSC WIP
stashed for the run and restored after, as in Round 91).

- [ ] **Try:** open any existing project normally — everything must work exactly as before (the
  stamp is invisible). Then in the **SQL Query** panel run `SELECT * FROM project_meta` — expect
  two rows: `format_version` = 1 and `written_by` = SandiBumi 0.1.0. The refusal path needs a
  future build to demonstrate for real, which is the point — it exists so that *next year's*
  files are safe in *this year's* app; the test suite stands in for the future build today.

## Round 91 — the green gate: one command that proves the tree is healthy (2026-07-29)

Q3 of the 1.0 quality bar (`docs/V1_SCOPE.md` §5, defined in `docs/RELEASE.md` §5 step 0) — until
now the three verification gates were run by hand, separately, from memory. **`tools\check.ps1`**
runs them in order and exits non-zero at the FIRST failure: (1) `npm run build` (tsc runs inside
it, so no duplicate type-check pass), then (2) full `cargo test` in src-tauri **through vcvars
pinned to 14.29** when that toolset exists (this machine's 14.50 is broken), plain `cargo test`
otherwise — so the same script works on a healthy machine. `-SkipRust`/`-SkipFrontend` exist for
the inner loop, but "green" means the full gate. It also prepends the known node/cargo homes to
PATH, so it works from a fresh shell that missed the installer PATH updates.

Verified with real runs, not by reading the script: **(a) green** — full gate on the committed
tree: frontend 7 s, backend 37 s (373 passed / 0 failed / 7 ignored), `GATE GREEN in 44s`, exit 0;
**(b) red** — its very first full run caught a REAL failure and propagated it (`GATE FAILED at
backend (cargo test) (exit 101)`, script exit 1); **(c) toolchain failure** — a bogus
`-VcVarsVer 99.99` fails fast at vcvars before cargo ever runs, exit 1.

**Worth knowing about (b), because it's a live finding in your working tree:** the failure it
caught is the in-progress `ssc.rs` edit (another session's work, dated 2026-07-29, uncommitted) —
it moves `SWIRR_T` to the pre-conditioning value per the Loglan, and the old test
`ssc_swirr_floor_pads_capillary_water`, which pins the post-conditioning floor semantics, now
fails against it. Proven by stash round-trip: HEAD's `ssc.rs` passes all 6 ssc tests; the WIP
version fails that one. Nothing was changed — the SSC work is mid-edit and its test reconciliation
is that session's to finish — but until it is, **a full-tree gate run will be red**, and that red
is true.

- [ ] **Try:** from PowerShell in the repo root run
  `powershell -ExecutionPolicy Bypass -File tools\check.ps1` — expect the two stage banners and
  `GATE GREEN in ~45s` (first run after a Rust change recompiles, so longer). Then break something
  trivial on purpose (e.g. add `let x: number = "no";` to any .ts file), run it again — it must
  stop at stage 1 with `GATE FAILED` and a red message, and `$LASTEXITCODE` must be 1. Revert the
  break. (If you run it before the SSC session finishes its test reconciliation, expect the honest
  red described above.)

## Round 90 — R30: three dialogs silently computed on GR when the curve they wanted was missing (2026-07-29)

From the F1 sweep (finding #4), verified still open against live code before touching anything.
Three dialogs — **SMLP/Lorenz**, **SHF fitting**, and **Facies tie-in** — had byte-identical private
copies of a curve-dropdown builder that walked the catalog and pre-selected the first "preferred"
name it found (`PERM`, `PHIE`, `TVDSS`, …). The trap was the miss path: **when none of the
preferred names existed in the well, it selected nothing — and an unset `<select>` falls back to
option 0 of the catalog, which is deterministically GR** (the catalog seeds `GR, RES_DEEP, NPHI,
RHOB, DT, SP` ahead of everything else). GR in gAPI (20–150) is numerically indistinguishable from
permeability in mD, so the Lorenz backend — which *does* guard honestly ("permeability curve 'PERM'
has no data in this well") — never got the chance to refuse: the dialog handed it a curve that
**did** have data, and it computed a fully plausible Lorenz coefficient and flow-unit table from
gamma ray. A clean cardinal-rule violation: a wrong result indistinguishable from a right one.

Fixed by deleting all three private copies and routing the **9 call sites** through one shared
helper (`plotCommon.ts preferredCurveSelect`): when no preferred curve exists, the first preferred
name (e.g. `PERM`) stays **selected and visible** in the dropdown — `curveSelect` prepends it as a
real option — so the run reaches the backend's own guard and fails loudly with the named curve,
instead of silently substituting GR. Bonus from the shared path: the private copies never set
`.form-control`, so all 9 dropdowns were also unstyled (the R13 defect class); they now match the
rest of the app. Two legs of the original report were corrected during verification and are noted
for honesty: the faciesTie leg was already *functionally* dead (the backend errors when predicted
== reference, which is what the double-GR fallback produced), and the headline TVDSS example was
weak (shf_fit drops non-positive heights) — the real damage was Lorenz-PERM and SHF-PHIE.

Verified: `tsc` + `vite build` clean; browser functional test against the real modules
(vite-only, server stopped afterward): catalog-without-PERM now yields a dropdown showing `PERM`
(7 options, styled), not GR; a catalog containing the preferred curve selects it with no duplicate
option; the full Lorenz dialog builds with φ=`PHIE`, k=`PERM` on an empty catalog.

- [ ] **Try:** open **Petrophysics → Rock Typing → SMLP / Lorenz…** on a well that has **no**
  permeability curve computed or imported. The Permeability (k) dropdown must show **PERM** (not
  GR). Click **Run** — you must get *"permeability curve 'PERM' has no data in this well"*, not a
  plot. Then compute/import a PERM and reopen — it should be found and selected as before. Same
  shape in **SHF fitting** (φ shows PHIE on a bare well) and **Facies tie-in**. All curve dropdowns
  in these three dialogs now also render with the app's styled look instead of the native browser
  select.
## Round 89 — PRD pass: webview CSP turned on, unused OS capability removed (2026-07-29)

Not an R-chain bug fix — this came out of writing `docs/PRD.md`, where §7.5 asks the question a
client's IT department asks: *what leaves the machine, and what can this app do that it doesn't
need to?* Two answers were worse than they should have been.

**1. The webview had no Content Security Policy at all** (`"csp": null`). That matters here
specifically because of R9 in this file — a hostile well name inside an imported LAS reaching the
DOM. That hole is closed by escaping, but a null CSP meant there was no second line of defence
behind it, and untrusted text arrives with *every* imported file. There is now a real policy in
`tauri.conf.json`. Two relaxations are deliberate: `script-src` keeps `'unsafe-eval'` because Vega
compiles chart expressions through the `Function` constructor and would silently stop rendering
without it, and `style-src` keeps `'unsafe-inline'` because CodeMirror injects a `<style>` element
and the print path writes one into its hidden iframe. Neither re-opens R9 — inline handlers and
inline `<script>` need `'unsafe-inline'` in **script-src**, which is absent.

**2. `tauri-plugin-opener` was registered and permitted but never used.** It grants the app the
ability to hand a URL or path to the OS. There were **zero call sites** anywhere in `src/`, so
nothing was ever passed to it — but a granted capability the product doesn't use is exactly what an
enterprise security review flags. Removed at all four layers: the Rust plugin registration, the
crate dependency, the `opener:default` capability entry, and the npm package.

Also in this pass: `README.md` no longer describes the product as "the reference suite-class"
(competitor-referential copy in the customer-facing document), `CLAUDE.md`'s collaboration protocol
now states this file's **actual** mark convention (`[x]` = accepted — it had preserved the
superseded `[o]` legend, under which your 72 accepted items read as 72 broken ones), and
`docs/IP_PROVENANCE.md` records where every piece of reference data in the repo came from.

Verified: `tsc` + `vite build` clean, `cargo check` clean after the plugin removal.

- [ ] **Try:** the CSP **cannot be tested with `npm run tauri dev`** — with a dev URL the webview
  loads Vite directly and Tauri never delivers the policy. It only applies to a packaged build. So:
  run `npm run tauri build`, install/launch the built app, and exercise the three paths the
  relaxations exist for — (a) open the **Vega** panel and render a chart, (b) open the **Inspector**
  (Equation Editor) and confirm the editor appears and highlights, (c) open any crossplot/histogram
  and use **Print** from its toolbar. All three must work exactly as before. If any of them is
  blank or dead, open DevTools ▸ Console and look for a `Content Security Policy` violation — the
  message names the directive that needs widening. Everything else in the app should be unaffected.
  Separately, confirm nothing anywhere tried to open an external link (nothing should — there were
  no call sites).

## Round 88 — R29: the Equation Editor leaked a whole CodeMirror editor every time you closed it (2026-07-25)

Sixth F5 fix, and a pure hygiene one — nothing renders wrong, no result goes stale, no data is at
risk. `InspectorPanel` had **no `dispose()` at all**. It correctly recycles its CodeMirror `EditorView`
on internal re-renders (pick another equation, switch language), but the **last** view of each panel
lifetime was simply abandoned. That is not just a detached DOM node: an `EditorView` registers four
listeners rooted at `window`/`document` — `resize`, `scroll`, `beforeprint` and `selectionchange`
(verified in `@codemirror/view/dist/index.js:7480-7492`) — and the **only** code path that removes them
is `EditorView.destroy()` (7513→7521). `window` and `document` are GC roots, so each abandoned view
kept itself, its history/autocomplete state, the python parse tree and the detached editor DOM
reachable **for the life of the process** — and every caret move anywhere in the app still dispatched
into every one of them.

It compounds faster than "how often do I close that panel?" suggests: the Inspector is closable, and
`dock.clear()` runs on **every session switch and every workspace reset**, so each of those strands
another editor too. `vegaPanel.ts` already destroyed its own `EditorView`, and the DB Inspector and
History panels are already wired to `dispose()` at `workspace.ts:419/428` — so this was an omission,
not a decision.

Fixed by giving `InspectorPanel` a `dispose()` (destroy + null + a `disposed` flag) and calling it from
the workspace cleanup closure alongside the two existing unsubscribes. The `disposed` flag matters on
its own: the editor now mounts **asynchronously** behind a dynamic `import("codemirror")`, and the
existing `host.isConnected` guard conflates "inactive tab" with "closed panel" (dockview detaches
inactive tabs), so it is not a dispose signal — without the flag, a panel closed during that import
window mounts a brand-new editor into a dead panel, with no remaining reference to destroy it.

Verified: `tsc` + `vite build` clean. A leak is invisible to `tsc` — not destroying an object is
perfectly well-typed — so proof is two-part. (1) A codebase invariant: `src` has exactly **two**
`EditorView` construction sites, `vegaPanel.ts:1053` and `inspectorPanel.ts:250`, and both are now
destroyed on dispose. (2) `inspector_leak_harness.mjs` models the lifecycle against a listener registry
using the real listener set: 15 open/close cycles strand **60** listeners on the old code and **0** on
the new, stranded views are confirmed still-undestroyed and still holding their payload, a panel closed
mid-import mounts an unreachable editor on the old code and refuses to mount on the new, and internal
re-renders still recycle to exactly one live view. **8/8 pass.** Frontend-only.

- [ ] **Try:** hard to see directly — it is memory, not behaviour, so mainly confirm **nothing broke**.
  Open the **Inspector** (Equation Editor), pick an existing equation, switch its language Rhai↔Python,
  edit the script and **Save** — all must behave exactly as before. Then close and reopen the Inspector
  ~10 times and confirm the editor still appears and still loads the selected equation's text each
  time. If you want to see the fix working, open DevTools ▸ Memory, take a heap snapshot before and
  after 10 close/reopen cycles: the `EditorView` count should stay flat instead of climbing by one per
  cycle.

## Round 87 — R28: the Tops pane could window every plot to another well's depths (2026-07-25)

Fifth F5 fix, and the second wrong-well one. `TopsPanel.refresh()` assigned `this.wellId = wellId`
**synchronously** but `this.tops` only **after** `await listTops(wellId)` — and nothing cleared the list
in between. So for the entire width of the DuckDB query the pane showed **well A's rows, still
clickable, under an id that already said well B**. Click one and `toggle()` paired the two live fields
and published `{wellId: B, topName: <A's top>, depthMin: <A's depth>}`. Both consumers accept an
interval on the **id match alone** — `logViewPanel.ts:341` scrolled well B's log view to well A's depth,
and `plotCommon.ts:322` re-windowed every crossplot / histogram / Pickett of well B to a foreign depth
range. That is a **parameter pick (Rw, m/n, cutoffs) read off the wrong zone**, and the wrong numbers
travel into a deliverable long after the session ends. It also defeated the invariant the workspace
explicitly documents at `workspace.ts:917-921` — "followers never see a foreign interval".

Worth stating plainly: this is **not** a lost race. `list_tops` is a synchronous `#[tauri::command]`
(`lib.rs:694`), and Tauri runs non-async commands inline in the IPC handler, so responses already
resolve FIFO — the generation token the original report proposed would have fixed nothing. The defect
was deterministic and fired on the *load window of every well switch*.

Fixed by making the id and the rows **one unit**: a `TopsView { wellId, tops }` snapshot, assigned only
together, **captured into each row's click closure** so a row can only ever emit the interval for the
well it was painted for. On a well change the list is cleared to "Loading tops…" before the await, so
the stale row is not there to be clicked at all; a *same-well* refresh (dataVersion after a run) keeps
its rows, so a recompute does not flicker the pane. A `refreshGen` token is still worth its three lines,
but for the honest reason — it drops a **superseded repaint**, not a stale write. Same snapshot shape as
R26's `GridView`. Also primed the `dataVersion` double-subscribe at `workspace.ts:968`, which was firing
a second identical `list_tops` and a second full DOM rebuild on every pane open.

Verified: `tsc` + `vite build` clean. A wrong-well emit is invisible to `tsc` — every type is correct,
the mismatch is *which* well the id belongs to — so `tops_wrongwell_harness.mjs` models both versions
against a hand-driven `listTops`: the old code emits `{wellId: B, topName: A-Sand 1}` and the harness
confirms a log view on well B **accepts** it, while the new code has nothing clickable during the
window, emits B's own top once B lands, and — even when a row is deliberately held past its refresh —
still emits a self-consistent pair. **9/9 pass.** Frontend-only.

- [ ] **Try:** open the **Tops** pane, a **Log View** and a **Crossplot** on a large project. Click well
  **A**, wait for its tops, then click well **B** and *immediately* click a top row while the pane is
  still mid-load. The pane must show **"Loading tops…"** with nothing clickable — never well A's names
  under well B. Then let B finish, click one of **B's** tops: the log view and plots must window to that
  depth. Also confirm a **recompute** (run any module) refreshes the pane **without** flashing "Loading".

## Round 86 — R27: a Python equation run showed 0% and no failures (2026-07-24)

Fourth F5 fix, and the first backend one of the tier. `run_python_equation` reported per-well progress
on its cancelled / fetch-error / no-data / all-MISSING branches — but **not** on the three that end a
normal run: the successful write, the write failure, and the script error. `finish_item` is the only
thing that increments a job's `done`, and `start_item` has already flipped each well to amber
"Running", so a healthy 20-well Python run rendered **"0%" and "0/20"** with all 20 wells apparently
mid-flight, then flipped to a **"Completed"** card still reading 0/20. Worse for honesty: a plain
Python **syntax or runtime error** — the commonest authoring mistake — left its well amber "Running"
instead of red "Failed", so the Processing panel showed **no failure signal at all** for a script that
never ran. The tell that this was a slip rather than a design choice: the *cancelled* branch did report,
so an **aborted** run displayed more progress than a **successful** one.

Fixed by mirroring the Rhai sibling (`equations.rs`) on all three branches — `finish_item(Ok)` after a
successful write, `finish_item(Failed, e)` on a write error and on a script error. Display/observability
only: `write_equation_output` already ran, so no curve data was ever wrong or lost — but the live
progress and the per-well states were.

Verified: `cargo test` — **373 passed / 0 failed / 7 ignored**, whole crate. New
`python_equation_reports_progress_on_every_terminal_branch` asserts on the `JobView` the panel actually
renders (done-count + item state), not on the return value, and covers the success and script-error
branches end-to-end (python is present here, so they really ran) plus the no-python early return as a
guard on machines without it. I also confirmed it is a **real** guard by reverting just the success
branch and watching it fail with "a successful write must count one unit of progress (was stuck at 0)".
Backend-only.

- [ ] **Try:** save an equation with **language = python** (e.g. `vshp = gr / 100.0`) and Run it over
  several wells. The **Processing** panel must count up to **N/N / 100%** with each well turning green —
  not sit at 0% with amber rows. Then deliberately break the script (e.g. `vshp = undefined_name + 1`)
  and Run again: the wells must go **red/Failed** with the Python error as the message, not stay amber.

## Round 85 — R26: the DB Inspector could write a cell edit to the wrong well (reload race) (2026-07-24)

Third F5 lifecycle-tier fix, and the one with teeth — a **silent wrong-row write into your own
well-log DuckDB**. `dbInspectorPanel.reload()` had no token, and `renderGrid()`/`commitEdit()`
**re-read live state** (`this.tableDef()`, `appState.selectedWell.get()`) at paint/commit time instead
of the scope the shown page was fetched under. Two failure shapes: (a) a lost race — pick Standard
Curves (slow 200-row query), then switch table/well before it lands; the slow page renders under the
now-live def; (b) the sharper one the verifier flagged — switch from well A to B while A's grid is still
on screen (the header flips to "B" synchronously, the grid lags), double-click a GR cell and Enter, and
`commitEdit` re-read `selectedWell` = B → `updateStandardSample(B.well_id, <A's depth>, "gr", v)`.
`db.rs` UPDATEs `WHERE well_id AND depth`, so it's rejected *unless* B has a sample at that depth — and
Mahakam wells share the 0.1524 m grid, so it usually **does**: a real value silently overwritten in the
wrong well, with an undo entry recording the wrong inverse so Ctrl+Z compounds it.

Fixed with the pattern the sibling plots already use (`crossplotPanel`/`pickettPanel` `reloadGen`), plus
the piece a token alone can't cover: bundle the fetched `(def, well, offset, page)` into a `GridView` and
thread it through `renderGrid → beginEdit → commitEdit`, so an edit is **always** bound to the rows on
screen — never a live re-read that a mid-flight reload moved on. A `reloadGen`/`disposed` token drops a
superseded page after its await (and prevents a write to a torn-down panel). One file, no API change, no
backend change, happy path unchanged.

Verified: `tsc && vite build` clean. A race is invisible to `tsc`, so a headless
`dbinspector_race_harness.mjs` models both decision points: with a stale grid on screen the OLD live-
re-read corrupts well B at A's depth while the NEW view-snapshot writes to well A (the row shown), and the
`reloadGen` token drops a slow reload that resolves after a newer one. 5/5. Verified-by-construction
against the two proven token siblings the fix mirrors. Frontend-only.

- [ ] **Try:** open **Database Inspector**, pick **Standard Curves** on a well with a long log. In the
  **Wells & Tops** pane switch to a *different* well and, immediately (before the grid repaints), double-
  click a GR cell and press Enter. The edit must land on the well whose rows you can see — never the newly
  selected one — and the status line's well name must match the grid. Then page/table-switch rapidly a few
  times: no stale rows should ever appear under a mismatched header.

## Round 84 — R25: the Correlation panel leaked a window `pointerup` listener every open/close (2026-07-24)

Second F5 lifecycle-tier fix, and a corroborated one — dimensions F5a and F5b flagged it independently.
`correlationPanel.ts` registered `window.addEventListener("pointerup", () => (dragging = false))` with an
**anonymous** handler and a `dispose()` that released only the ResizeObserver and two subscriptions. A
`window` listener outlives the panel (unlike the canvas-scoped ones, which die with the detached `el`
subtree), so every close stranded one dead handler — and because it closes over `dragging`, which shares a
scope with `strips`, each stranded listener pinned that build's **entire `WellStrip[]`**: per well a
1400-sample decimated curve pair plus a two-`Float64Array` TVDSS map, for every well in the active group.
Correlation panels are `freshId(kind)` (never singletons), so the retained set grew per open/close cycle —
~1.5–7 MB pinned per cycle on Jauhar's 40–200-well groups, monotonic for the process life, surviving Reset
Workspace / Open Session (same dispose path). `LogCanvasRenderer.ts:540-561` even carries a comment warning
about this exact trap; correlation was the lone panel builder that fell into it.

Fixed by the documented house pattern: hoist to a named `const onWindowPointerUp`, register it, and add
`window.removeEventListener("pointerup", onWindowPointerUp)` to `dispose()`. Same edit captures the
`setTimeout(fit, 50)` as `fitTimer` and `clearTimeout`s it in dispose, so a panel closed inside 50 ms can't
run `fit()`→`draw()` against an already-detached canvas. No behaviour change — pure teardown hygiene.

Verified: `tsc && vite build` clean. Proof for a leak is dispose symmetry: a repo-wide grep of every
`window.addEventListener` now shows every **per-panel** listener has a matching `removeEventListener`
(crossplot 2047↔2101, correlation 1049↔1114, map, plotCanvas, vega 1129↔1179, viewerChrome,
LogCanvasRenderer) — the only add-only ones left are the app-shell singletons built once at boot (ribbon,
workspace, autosave, main, interactionGuard), which the F5 review classifies as one-off, not defects. So
correlation was the last panel builder missing its removal, and it no longer is. Verified-by-construction
against the three proven siblings the fix copies. Frontend-only.

- [ ] **Try:** hard to see directly (it's a leak), but sanity-check nothing regressed: open a **Correlation**
  panel on a multi-well group, **drag** a strip up/down to pan (release the mouse *outside* the canvas — panning
  must still stop cleanly), hover to confirm the linked depth still syncs, then close and reopen the panel a few
  times. Everything should behave exactly as before; the fix only frees memory on close.

## Round 83 — R24: the Report pane never opened in the multi-select state (TDZ crash) (2026-07-24)

A flat user-facing bug, not an honesty one: with **no active well group** but a **multi-selection or ★-pins**
present, opening **Report** failed outright — the pane showed "Failed to open the report generator:
ReferenceError: Cannot access 'batchBtn' before initialization". That is exactly the state you are in when
you reach for **batch** report export, which is why it survived: the usual active-group state dodges it.

Root cause is an async-constructor / synchronous-observer collision. `buildWellScope` is `async` and, after
awaiting `listWells`/`listWellGroups`, subscribes to `pinnedWellIds`/`multiSelectedWellIds`. `Observable.subscribe`
fires its listener **synchronously** on subscribe (`state.ts:29`), and when `smartDefault()` lands on "pinned"/
"selection" that first fire runs `emit()` → the caller's `onChange`. But the caller (`reportDialog`) is still
parked on `await buildWellScope(...)`, so the `const batchBtn` its `onChange` reads is still in its temporal dead
zone → `ReferenceError`, which rejects the builder's promise and the whole pane. Same failure mode as the earlier
V3 Vega TDZ, in a different place.

Fixed with the house **primed-flag** pattern (as in `plotCommon.ts:349` / `mapPanel.ts:434`): a `let ready = false`
gates both subscribe callbacks, set `true` only after the scope's own first paint. The synthetic construction-time
fire is suppressed; genuine post-construction pin/select changes still emit. Nothing is lost — every caller does its
own first paint (reportDialog sets the batch label from `getWellIds()`, cutoffDialog awaits `refreshZoneDst()`), and
of the 13 `buildWellScope` callers only those two pass an `onChange` at all. Frontend-only.

Verified: `tsc && vite build` clean. TDZ is a runtime error `tsc` cannot see, so a **headless Node harness**
(`wellscope_tdz_harness.mjs`) models the exact mechanism — a synchronous-fire Observable, an async builder that
subscribes after two awaits, a caller whose `onChange` reads a const declared after its await — and proves it:
the unguarded pattern throws the TDZ ReferenceError, the guarded (`ready`) pattern opens cleanly with the right
label, the construction-time fire is suppressed, and a real post-construction change still emits. 5/5 pass.

- [ ] **Try:** with a project open, leave the group selector on **All wells** (no active group). In the **Wells**
  pane, **Ctrl-click two wells** (or ★-pin one and clear the selection). Ribbon → **Report**. The pane must open
  normally showing a **Batch (N wells)…** button — not "Failed to open". Then pin/select another well while it is
  open: the **Batch (…)** count must update live. Repeat with a group active to confirm nothing regressed there.

## Round 82 — R23: the Field Dashboard Compute posted a redundant "Pay summary" job card (2026-07-24)

The tail of R19. `run_pay_summary`'s silent-run guard was `if req.stats_only && req.skip_version`, but
the Field Dashboard sets `stats_only` **alone** (`skip_version` defaults false) — so that branch matched
**no** caller, and every dashboard **Compute** fell through to `run_simple_job`, posting a
"Pay summary — cutoffs & pay" card in the Processing panel. That card is redundant (the dashboard already
reports "Computing N well(s)…" then the result in its own status line) and mildly misleading — labelled
"cutoffs & pay" for a run that, being `stats_only`, writes nothing (a faint echo of the R19 lie).

Fixed by keying the silence on the real invariant: `if req.stats_only`. A stats-only pay summary persists
nothing (`workflow.rs` gates every FLAG_* write behind `!stats_only`), so it is a pure read and never
needs a job card. The dashboard is the only stats-only caller, so this touches only it; a **persisting**
pay summary — an explicit Cutoffs & Summary run, or a report render (`skip_version`, `stats_only` false) —
still shows a job. The old guard encoded "dashboard" by an incidental two-flag coincidence that the
stats_only refactor had silently broken; the new one ties silence to "persists nothing".

Verified: `cargo test pay_summary` — 4/4 green via the pinned 14.29 toolchain, incl.
`pay_summary_stats_only_persists_nothing` (the invariant this fix relies on); whole-crate compile clean,
no warnings. This is a Tauri command (not directly unit-testable), and the change is grep-proven to affect
only the dashboard (`stats_only: true` has one command-level caller). Backend-only.

- [ ] **Try:** open the **Field Dashboard**, press **Compute** a few times. The **Processing** panel must
  stay quiet — no "Pay summary" card appears — while the dashboard's own status line shows progress and the
  result. Then run **Cutoffs & Summary** (or export a **report**): those must still show a job card as before.

## Round 81 — R22: the legacy Multimin module is retired (your decision) (2026-07-24)

This one is a **decision**, not an F-sweep finding — the follow-on R17 surfaced. The legacy fixed
4-component `multimin` inversion (superseded by SandiMin, hidden from every UI picker since long ago)
was still a **live compute path**: `list_modules` returned it, so any saved workflow chain with a
`multimin` step — or a restored `module:multimin` dockview panel — still ran the old solver, silently,
with endpoint defaults that could drift from SandiMin's library. You chose **graceful retirement** over
a hard delete or a keep-and-consolidate.

Implemented so a retired module fails **loudly and actionably** rather than vanishing or running stale
physics: a new backend registry `modules::retired_module(name)` is the single source of truth;
`run_module` checks it first and returns *"The Multimin module is retired… Re-run this step with
SandiMin (Advance ▸ Mineral Solver)."* before any dispatch. The `multimin` **spec is kept** in the
catalog on purpose — a saved chain step still resolves by name and renders its stored parameters, so
you can see what it was before re-doing it in SandiMin — but the solver body and its R17 physics tests
are removed (unreachable now; R17's reusable `rho_e` Pe↔U relation stays in `multimin2`, where SandiMin
uses it). New-chain wiring already excluded it; the two frontend comments that still claimed *"it runs
in saved chains"* are corrected.

Why graceful, not hard-delete: a hard removal would drop the id from the catalog, so a saved chain would
die with a cryptic *"unknown module 'multimin'"* instead of a message that tells you what to do. Why not
keep-and-consolidate: you asked for retirement — the trade-off is that a delivered chain containing a
`multimin` step can no longer reproduce its old output; it must be re-run in SandiMin.

Verified: full `cargo test` — **372 passed / 0 failed / 7 ignored**, whole-crate compile clean with **no
warnings** (the solver removal left no dead code / unused imports). New `multimin_is_retired_but_still_cataloged`
(registry + still-cataloged) and the converted end-to-end guard `phase7_generic_store_feeds_modules_and_mask`
(running `multimin` now returns a SandiMin error and writes no curves) both pass; every SandiMin/`multimin2`
test still passes. `tsc --noEmit` + `vite build` clean.

- [ ] **Try:** if you have any saved **workflow chain** with a Multimin step, run it — the step must
  fail with "…retired… Re-run this step with SandiMin (Advance ▸ Mineral Solver)", *not* run silently
  and *not* say "unknown module". Confirm SandiMin (Advance ▸ Mineral Solver) still runs normally. New
  chains: the step picker must not offer Multimin.

## Round 80 — R21: ML training wells that contributed zero samples were silently dropped (2026-07-24)

Supervised ML pools labelled rows across the selected training wells. `fetch_curve_frame` returns an
**all-NaN** column for any curve a well lacks, so a training well with **no target curve under the
chosen mnemonic** (or no input, or fully masked) contributed **zero** rows through the `is_finite()`
filter — invisibly. Nothing recorded which training wells were actually used; `MlResult.wells` only
ever carries the *apply* wells. The `n_train < 10` guard never fires because the few real wells supply
tens of thousands of samples at 0.1524 m. So the run returned success with R²/RMSE, and the user
believed a 20-well model was fitted. The scenario is the *normal* one here: core-calibrated PERM/facies
models where CPERM or core-facies exist in a small minority of the field — select 20, have the target
tied to the log grid in 3, and you ship a "20-well field model" that is a **3-well model**, with a
wrong-mnemonic typo (CPERM vs KCORE) producing output identical to a correct run. The **Compare** button
in the *same file* (`run_ml_eval`) already warns about exactly this ("N of M training well(s)
contributed no samples") — only the **Run** button was silent.

Fixed by tracking, per training well, whether it moved the labelled pool at all, and collecting the
ones that didn't — whatever the cause (unreadable, missing target/feature, or fully masked). A new
`notes: Vec<String>` on `MlResult` carries a count summary ("{k} of {n} training well(s) contributed
no usable samples … the model was fit on the remaining {n−k}"), mirroring the `run_ml_eval` sibling;
`mlDialog` renders it as a `⚠` warning at the top of the results (glyph + `--warn`, honouring R16's
redundant-coding rule). The two **dead** `else { continue }` guards the finding flagged (the all-NaN
fallback made them unreachable) are gone, and the previously-silent `fetch_curve_frame` **error** branch
now also lands in the empty-well list instead of vanishing.

The honesty-critical logic — *which wells contribute nothing* — was extracted into a pure
`assemble_training` helper so it is unit-testable **without python** (the existing `run_ml` tests skip
when sklearn is absent). Backend + a small additive frontend note.

Verified: `cargo test ml::` — 11/11 green via the pinned 14.29 toolchain, incl. the new
`assemble_training_flags_wells_with_no_target` (a well with the target contributes all its rows; a
target-less well is flagged empty, not dropped) **and** the python-backed end-to-end tests, which ran
and passed — so the extraction didn't regress the real `run_ml` path. `tsc --noEmit` + `vite build`
clean.

- [ ] **Try:** run a supervised model (e.g. regression PERM, or k-NN facies) over a group where the
  **target** curve exists on only some wells — select 10+ training wells, of which only a few actually
  carry the target under the chosen mnemonic. The results panel must show a **⚠** line like "7 of 10
  training well(s) contributed no usable samples … fit on the remaining 3", not a clean metrics-only
  card. Then run one where every training well has the target: **no** warning line.

## Round 79 — R20: the SQL console reported the LIMIT-capped row count as the true total (2026-07-24)

The SQL console runs every query through `runQuery(sql, 1000)`; the backend wrapped it in
`LIMIT 1000` and returned `total_rows = rows_out.len()`, so a query that matched 400,000 rows came
back as exactly **1000** and the panel printed **"1000 row(s)"** — no truncation marker anywhere,
indistinguishable from a genuine 1000-row result. And this is the *common* case, not an edge: any
row-level query against `standard_curves` blows past 1000 on a single well (a 2000 m interval at
0.1524 m ≈ 13,000 samples), so essentially every non-aggregate query a petrophysicist types — counting
shaly samples above a GR cut, sizing how many rows a cleanup would touch — silently truncated and then
reported the cap as the answer. The **DB Inspector** one dock over renders `${from}–${to} of
${total_rows}` from a real `COUNT(*)`, which actively trains the user to read `total_rows` here as a
true total.

Fixed with a **definitive** signal, not a guess. The sweep's verifier proposed a frontend-only
heuristic (`rows.length === limit`), but that mislabels a result that is *exactly* 1000 rows as
"maybe truncated" — a false positive. Instead the backend now fetches **`LIMIT + 1`**: if more rows
come back than the cap, it sets a new `truncated: bool` on `TablePage` and returns exactly `limit`
rows. A result that fills the cap exactly fetches `limit + 1` = one-too-few and reads as **complete**.
`truncated` is a shared-struct field; the paginated inspector path (real `COUNT(*)`) always sets it
**false**, so the flag cleanly means "`total_rows` may undercount the true result." The panel now
renders "1000 row(s) shown — display cap reached; more rows exist (not the total)" when set.

I chose the backend flag over the verifier's frontend heuristic deliberately: it's **exact** (no
exactly-at-cap false positive) *and* it's the only version that is **cargo-testable** — with the
in-app browser down this session, a frontend-only change would have no verification surface.

Verified: `cargo test inspector_tests` — 11/11 green via the pinned 14.29 toolchain, incl. the new
`readonly_query_flags_truncation_at_the_cap`, which locks all three boundaries (below cap → truncated;
above cap → complete; **exactly at cap → complete**, the heuristic's false positive) and confirms the
inspector path still reports its real `COUNT(*)`. `tsc --noEmit` + `vite build` clean.

- [ ] **Try:** in the **SQL console**, run a row-level query that exceeds 1000 rows — e.g.
  `SELECT depth, gr FROM standard_curves` on any well with a long interval. The footer must read
  "1000 row(s) shown — display cap reached; more rows exist (not the total)", **not** a bare
  "1000 row(s)". Then run a small query (e.g. `SELECT well_name FROM wells` on a <1000-well project):
  the footer must read a plain "N row(s)" with no cap marker.

## Round 78 — R19: the Field Dashboard claimed "FLAG curves written." on the path that writes nothing (2026-07-24)

Pressing **Compute** on the Field Dashboard runs `run_pay_summary` with `stats_only: true` — the
comment three lines above the write even says *"compute the stats, persist nothing."* Yet the panel's
status line asserted **"FLAG curves written."** `workflow.rs` gates the *entire* FLAG-write block
(both the in-place and the versioned branches) behind `if !req.stats_only`, so with `stats_only: true`
**nothing is written** — this is pinned by the unit test `pay_summary_stats_only_persists_nothing`
("stats_only must not write any FLAG_* curve", "…must not create a PAYFLAG log set"). A petrophysicist
who read that line and then opened a Log View or picked `FLAG_PAY` as a crossplot Z-curve found
nothing, with no error to explain it — a classic hunt-for-the-bug-that-is-a-lying-status-message. The
sharper case: if an earlier **Cutoffs & Summary** run already wrote `FLAG_PAY`, the dashboard claimed
"FLAG curves written" after a **cutoff tweak** while Log View still showed **stale** flags computed at
the *old* cutoffs — silently wrong, not merely absent.

Fixed the status line to tell the truth — *"Stats only — no FLAG curves written; run Cutoffs & Summary
to persist flags."* — which covers both the absent-flags and the stale-flags cases (it says **this**
Compute persisted nothing, so any `FLAG_*` in Log View is from a prior run, possibly at other cutoffs).

The lie was not confined to that one string: the same stale attribution — *"the Field Dashboard writes
`FLAG_*` in place / sets `skip_version`"* — was mirrored across **five** comments, the TS doc the sweep
named being merely a mirror of its Rust struct-doc source. Post-`stats_only` refactor the dashboard
sets `stats_only` **alone**; `skip_version`'s only real writer today is the **report/composite render
pass** (`report.rs:398`). Corrected all five (`ipc.ts`, `workflow.rs` struct-doc + write-branch +
test, `lib.rs`) so a future maintainer deciding whether `skip_version`/`stats_only` can be collapsed
reads the truth. All backend edits are **comment-only** — zero logic change.

Surfaced but deliberately **not** changed (behavior decision, needs your call): `lib.rs`'s silent
off-thread guard is `if req.stats_only && req.skip_version`, but the dashboard sets only `stats_only`
(`skip_version` defaults false), so it now takes the **job-card** path — every dashboard Compute posts
a "Pay summary" card, the opposite of the silence that guard was meant to give it. I documented the
gap in the comment rather than silently flipping the guard to `if req.stats_only`.

Verified: `tsc --noEmit` clean + `vite build` clean (frontend string/JSDoc); `cargo test
pay_summary_stats_only_persists_nothing` green via the pinned 14.29 toolchain (whole-crate recompile
clean, so the five comment edits didn't break anything, and the test is itself the proof the old
string lied). Browser-independent.

- [ ] **Try:** open the **Field Dashboard**, press **Compute**. The status line must read
  "…Stats only — no FLAG curves written; run Cutoffs & Summary to persist flags." — never "FLAG curves
  written." Then open a **Log View** on any well: there must be no *newly* written `FLAG_PAY`/`FLAG_SAND`
  from that Compute. To actually persist flags, run **Cutoffs & Summary** and re-open the Log View.

## Round 77 — R18: the report PDF silently dropped the Pay Summary section on error (2026-07-24)

Section 4 of the report did `run_pay_summary(...).unwrap_or_default()`, which collapses **both** an
`Err` (the `FLAG_*` write at `workflow.rs` failing — read-only DB, disk full, appender error) **and**
a legitimately empty result into the same empty `Vec`. The `if !pay_rows.is_empty()` guard then
dropped the **entire** section — header included — from the deliverable PDF, and `report_pages`
returned `Ok`. The PDF was indistinguishable from a well that genuinely has no pay, and
`export_report_batch` recorded the well in `written`, not `errors` — so a 540-well Mahakam batch could
ship 540 "successful" client PDFs, every one missing its pay table, with an **empty error list**. The
sharpest part: the pay numbers are computed in memory *before* the write side-effect, so a storage
error suppressed a table whose values were already fully renderable.

Fixed by emitting the section header **unconditionally** and branching on the `Result`: the table on
rows, an explicit **`Pay Summary unavailable — {e}`** note page on `Err`, and a "no curve data to
classify" note on the legitimately-empty case. It deliberately does **not** propagate the `Err`
(that would abort the whole PDF and lose the composite log pages the user did want over one bad pay
run) — the well is still counted as `written`, but the document now always carries a visible trace of
what happened. New `note_page` helper (section header + wrapped note) for the two non-table branches.

Verified: `cargo test` green via the pinned 14.29 toolchain; new `note_page_shows_section_header_and_message`
asserts the header, the well name, and the failure note all render (the old code rendered none of
them). Whole-crate compile clean. Backend-only, browser-independent.

- [ ] **Try:** export a report (or a **batch** export) for a well whose pay run can't complete — e.g.
  a well with no computed curves, or with the project DB file set read-only. The PDF must still show
  a **Pay Summary** header page with a note ("unavailable — …" or "no curve data …"), never a report
  that simply skips from Zone Parameters straight to the composite log pages with no pay section.

## Round 76 — R17: the legacy Multimin solver mixed PEF by the wrong physics (2026-07-24)

The legacy `multimin` module (superseded by SandiMin/`multimin2`, hidden from every UI picker but
**still registered** at `modules.rs:201`/`:240`, so `list_modules` returns it and any pre-existing
saved chain or dockview layout holding panel id `module:multimin` still runs it) pushed the **raw
per-electron PEF** straight into its NNLS linear system. Photoelectric factor does **not** mix
linearly by volume — the **volumetric** photoelectric factor `U = Pe·ρe` does. `multimin2` already
converts to U before mixing; the legacy solver never did.

The consequence isn't just biased numbers — it's the QC curve lying about **who is at fault**. With
the module's own defaults a 50/50 quartz-water sample carries a 0.30 b/e PEF residual (physical
PEF ≈ 1.38, the linear-Pe law gives 1.085) — **exactly 1.0× the default `SIG_PEF`** — so `RECON_ERR`
reads a full sigma of *model* error and reports it as *log* misfit, telling the user to re-condition
perfectly good PEF data. And the bias is directional: linear mixing under-predicts Pe for a
light-fluid mix, so NNLS over-assigns the high-Pe clay endpoint (3.10), inflating `VSH_MM` and
deflating `PHIT_MM`/pay — the wrong direction for Mahakam-delta shaly sand.

Fixed by converting every PEF endpoint **and** the measured reading to `U = Pe·ρe` before they enter
the system, and carrying the uncertainty in U space (`σ_PEF·ρe`). The `ρe(ρb)` relation is now a
single `pub(crate)` function in `multimin2` that **both** solvers call, so their Pe physics can't
drift apart (the standing hazard the finding flags). A live RHOB is required to get ρe; with RHOB
absent the PEF row is **dropped** rather than mixed wrongly, and the existing `n_tools < 3` gate then
skips the sample honestly. The module's own recovery test was **complicit** — it forward-modelled the
synthetic PEF with the *same* wrong law (`vs*1.81 + vw*0.36`), so it passed by construction and could
never catch this; it now forward-models with the U law, making it a genuine regression guard.

Verified: `cargo test` green (46 passed) via the pinned 14.29 toolchain. Two new tests lock the fix —
`multimin_pef_uses_volumetric_u_mixing` (the finding's 50/50 worked example: asserts the physical
PEF ≈ 1.382, that it differs from the raw-Pe law by > 0.25 b/e, and that the solver recovers 50/50
with `RECON_ERR` < 0.2) and `multimin_drops_pef_when_rhob_absent`. Entirely a backend physics change,
so it's cargo-proven and browser-independent. Backlog (unchanged, separate item): the two solvers
still keep divergent endpoint tables (legacy's hardcoded `PEF_CLAY 3.10` / `RHOB_CLAY 2.55` vs
`multimin2::multimin_library`); unifying or retiring the legacy module is its own decision.

- [ ] **Try:** if you hold a saved workflow chain or a saved dockview layout that references the
  hidden **Multimin — Mineral Inversion** module, re-run it on a well that has a PEF curve. `RECON_ERR`
  should no longer sit near a flat ~1σ floor on clean intervals, and `VSH_MM` should come down
  (PEF-misfit was inflating it). Wells without PEF are unaffected.

## Round 75 — R16: the Results-QC scorecard status was carried by brand colour alone (2026-07-24)

The one panel whose entire job is to tell you a result is degraded encoded each check's verdict
(`ok` / `warn` / `alert` / `na`) as a **9px colour dot only**, and the dot's colour reused the
**brand** `--accent` / `--accent2` / `--warn` tokens — chosen for branding, never for pass/fail
meaning. Two consequences, both live on **default** screens, not just demo skins:

- **Default theme:** `warn` mapped to `--accent2` = `#5f7350` (olive **green**) and `ok` to
  `--accent` = `#b5651d` (ochre). So a Buckles BVW check that trips its warn threshold paints green
  next to a passing check painted orange — **the degraded result reads as the clean one**. That is
  exactly the cardinal data-honesty rule inverted.
- **Halliburton skin:** `ok` = `#e31b23` (bright red) vs `alert` = `#b3141b` (dark red) — at 9px
  these are one colour, so every clean zone reads as an alarm across a 60-dot scorecard. And `warn`
  (graphite `--accent2`) collided with `na` (dimmed `--text-dim`).

Fixed with **redundant coding** — shape *and* hue, so neither channel alone has to carry the
verdict. (1) Each row now shows a **glyph** (`✓` / `⚠` / `✗` / `–`, the set `processingPanel`
already renders as monochrome text in this runtime) via `dot.textContent`, plus `role="img"` +
`aria-label` (`pass` / `warning` / `fail` / `not run`) so a screen reader announces the status word.
(2) New **semantic** `--qc-ok` / `--qc-warn` / `--qc-alert` tokens (green / amber / red) drive the
colour, decoupled from the brand palette — declared once in `:root` (all five brand skins are
light-background, so they inherit an identical, legible triple) with a brighter override in the two
dark contexts. `.rqc-dot` became a glyph carrier instead of a filled circle. This also removes a
standing hazard: every future client skin previously re-rolled the meaning of the QC colours for
free; now it can't.

Purely additive — one DOM line + three CSS vars per theme + the `.rqc-dot` restyle; no computation,
threshold, or shared component touched, and the `na` text rows ("run SandiMin recon QC first", etc.)
and the CSV export were already fully readable and are unchanged. Verified: `tsc && vite build`
clean; grep confirms `--qc-*` defined in `:root` + both dark blocks and consumed only by the three
`.rqc-dot-*` rules, and that no `.rqc-dot` rule still references a brand token. Browser-observable
(needs the full Tauri app to populate a scorecard + a theme switch), and the in-app browser is still
down this session — so this carries a click-through Try line, and the exact before/after colour
mapping is written out above (the `--accent` / `--accent2` / `--warn` hexes vs the new `--qc-*`
triple) rather than shown in a live screenshot.

- [ ] **Try:** run a full interpretation + SandiMin recon + Monte Carlo so the **Results-QC** panel
  shows a scorecard with a mix of pass / warn / fail rows. Each row must show a `✓` / `⚠` / `✗` glyph
  (not a bare dot). Switch the theme to **Halliburton** and to the **default** earth-tone: a passing
  check must never look like a failing one, and a warn must never look like a pass, in **any**
  palette. Confirm the glyphs stay monochrome (not colour-emoji).

## Round 74 — R15: the Vega panel keeps plotting pre-run values after a module run (2026-07-24)

The interactive Vega panel (the V1–V6 work) was the **only plot panel with no `dataVersion`
subscription**. Its siblings — `crossplotPanel`, `histogramPanel`, `pickettPanel`,
`correlationPanel` — each carry the same primed `appState.dataVersion.subscribe(… reload)` block, so
after a SandiMin / equation run they re-fetch and redraw with the new curves. The Vega panel didn't:
it subscribed only to `brushedDepths` and `themeVersion`, and `workspace.createPlot` only rebuilds on
`selectedWell`. So you could run SandiMin to recompute SW, watch the crossplot beside it redraw with
the new cloud, while the Vega scatter of the **same two curves** silently kept showing the pre-run
values — **two contradictory clouds on screen, the stale one presented as a clean result**. That is
exactly this app's cardinal data-honesty violation, and the Vega panel is the one with the SVG/PNG
export path, so a stale cloud can walk straight into a client deliverable. A second symptom: the newly
written curves (`MM_PHIE`, `MM_SW`) never appeared in the X/Y/Colour/Group dropdowns until the panel
was closed and reopened — which reads as "the run didn't write the curves."

Fixed by mirroring the sibling pattern: a **primed** `dataVersion` subscription (first synchronous
fire swallowed, so panel build doesn't double-load) that refills the four curve selects from a fresh
`loadCurveNames()` and calls the existing `render()` (which re-fetches through `getCurveData` and is
already race-guarded by its `gen` counter + `disposed` check). Released in `dispose` alongside
`unsubBrush`/`unsubTheme`. The refill is done by a small `refillCurveSelect` helper that **preserves
the current selection** — a curve that has vanished from the catalog is kept as a leading option so
the axis never silently jumps to a different curve. A `dataVersion` bump resets the vega zoom/pan (a
full `render()`), which the file already accepts explicitly for theme repaints. `loadCurveNames`
failure is caught and still triggers a re-render, so a fetch error surfaces through `render`'s own
"Failed to load curves" path rather than freezing on stale data.

Verified: `tsc && vite build` clean. A 20-check headless harness pins the `refillCurveSelect`
invariant — selection preserved across curves added / removed / renamed, the `— None —` and
`By zone` lead options never duplicated, every outcome resolves to an existing option, idempotent.
The fix is a line-for-line copy of a subscription proven in four sibling panels, so the wiring is
verified by construction; the live redraw itself is browser-observable but needs the full Tauri app
to bump `dataVersion` (a module run), and the in-app browser is still unresponsive this session — so
this one carries a click-through Try line rather than a captured screenshot.

- [ ] **Try:** open a **Vega** scatter of PHIE vs SW for a well, then run **SandiMin** (or any module
  that recomputes SW). The Vega cloud must redraw to match the crossplot beside it — not keep the old
  cloud — and the newly written `MM_*` curves must now be pickable in the X / Y / Colour / Group
  dropdowns **without** closing and reopening the panel. Confirm your current axis selections are
  preserved across the redraw.

## Round 73 — R14: finish the innerHTML sweep R9 deferred — a real well-name XSS was still open (2026-07-24)

R9 closed the five interpolated-`innerHTML` sites it scoped but explicitly deferred "the full 17-site
sweep." Finishing it turned up a **genuine miss of the same RCE class**: `autoCorrDialog` builds an
error row as `tr.innerHTML = \`<td>${wellName}</td><td colspan=4>${wp.error}</td>\``, and `wellName`
is the **LAS-supplied `~W WELL` value, stored verbatim** — the exact R9 vector. With `csp: null`, a
hostile header injects markup into the autocorrelate results table: the same XSS→(via `save_png`)→RCE
reach R9 was about, at a site R9 didn't touch.

Swept **all 14** interpolated-`innerHTML` sites across `autoCorrDialog` / `zonesDialog` / `workspace`
/ `dashboardPanel` / `topsPanel`. Every interpolated **string** value is now wrapped in `escapeHtml()`
(the safeDom primitive); numeric interpolations (`.toFixed()`, `.length`) are left alone (they can't
carry markup). The genuinely untrusted ones: the well name (autoCorr), zone / param names and values
(zones — import-supplied), and backend error strings (`workspace` `${err}`). The rest — dashboard
flag/metric labels, the tops empty-state text, panel/kind labels — are app-controlled today, but
escaping them too keeps the invariant total, so a future dynamic value can't silently reopen a hole.
Table-row sites keep `innerHTML` with `escapeHtml()` on the data (concise, and structure-preserving);
R9's message-only sites had used DOM construction — both are safe, chosen per context.

Verified: `tsc && vite build` clean. A grep of the whole `src` confirms **every** `innerHTML`
interpolation is now `escapeHtml(...)` or a number — **zero** unescaped string interpolations remain
— and there are **no** other HTML-injection sinks (`insertAdjacentHTML`, `outerHTML =`, or
`+`-concatenated `innerHTML`) anywhere. `escapeHtml`'s inertness is the R9-established `textContent`
round-trip (markup → text). This is entirely browser-independent, which is just as well — the in-app
browser is still unresponsive this session.

Still deferred (unchanged from R9): a real `csp` (risks vega-embed / CodeMirror inline styling —
wants live testing the browser can't give right now) and scoping `save_png`.

- [ ] **Try:** import a LAS whose `~W WELL` value is `<img src=x onerror=alert(1)>`, then run
  **Autocorrelate** across wells so one returns an error row. The Well cell must show the literal
  text `<img …>` (not a broken image, and nothing executing). Same for a zone renamed with markup in
  the **Zones** pane, and a computed-curve error surfaced in a panel.

## Round 72 — R13: six module-dialog Run buttons stop rendering as native grey buttons (2026-07-24)

Cosmetic-consistency, but wrong on every theme. The **Facies Tie / HFU / Lorenz / ML / SHF /
Thomeer** dialogs (and the **Workflow** runner) build their Run button as a raw `<button>` whose only
class is `primary` — and `.primary` had **no standalone CSS rule**. It exists only in compound
selectors (`.lp-btn.primary` accent-override, `.guard-confirm button.primary`, and
`.workflow-run-row .primary` which set only `font-weight: 600`), and the app's base `button` rule
sets nothing but font inheritance. So these buttons fell through to the browser's **native grey UA
button** — the only Run buttons in the app not accent-filled. The class was added expecting a
primary-button style that was never written as a base.

Fixed with one scoped rule — `.mc-run-row .primary:not(.mm-run-btn), .workflow-run-row .primary` —
giving the app's accent primary look (accent fill, white text, 4px radius, 6px 24px padding, bold,
`--accent-dim` hover, dimmed `:disabled`), mirroring the multimin `.mm-run-btn`. Scoped to the two
run-rows so nothing else moves: **multimin keeps its own treatment** (excluded via `:not(.mm-run-btn)`
— it legitimately carries both classes and lives in `.mc-run-row`), the **Monte Carlo** run button
isn't a `.primary`, and `.lp-btn.primary` / `.guard-confirm` / the autosave restore button sit in
other containers. The old `.workflow-run-row .primary { font-weight: 600 }` folds into the new rule,
so the Workflow Run button also becomes accent-filled and matches its siblings. Per the R11 lesson I
verified `--accent` and `--accent-dim` are defined in **all 8** palettes first — no repeat of the
undefined-variable trap.

Verified: `tsc && vite build` clean (bundled CSS 182.21 kB, +0.44 kB). Deterministic: I read all
seven call sites and confirmed each appends its `.primary` button into `.mc-run-row` (six) or
`.workflow-run-row` (one), so the selector matches; at specificity 0,3,0 nothing competes for the six
targets, so the accent styling wins where before there was no rule at all. What I could **not** do:
capture a live screenshot — the in-app browser was unresponsive again this session, and the real
dialogs need the Tauri backend plus a user action to open. So this rests on the selector-match +
specificity argument, not a pixel view.

- [ ] **Try:** open **Facies Tie**, **HFU**, **Lorenz**, **ML**, **SHF**, **Thomeer**, and the
  **Workflow** runner. Each Run button should now be an accent-filled button (like Multimin's and the
  plot Run buttons), darkening on hover — not a native grey one. Multimin's Run button should look
  exactly as before.

## Round 71 — R12: one cutoff source, so Monte Carlo net-pay reconciles with the pay summary (2026-07-24)

A data-consistency finding — the quiet-but-expensive kind. The pay-cutoff quartet (VSH ≤ / PHIE ≥ /
SWE ≤ / PERM ≥) was independently hard-coded in **five** panes, and two had drifted: **Monte Carlo**
*and* the **Results-QC cutoff-sensitivity probe** defaulted to **PHIE ≥ 0.08 / SWE ≤ 0.5**, while the
canonical **Cutoffs & Pay Summary** uses **PHIE ≥ 0.1 / SWE ≤ 0.6**. So an MC net-pay run "with
defaults" used *different* cutoffs than the deterministic pay summary "with defaults" — the P50 net
would not reconcile with the deterministic net, and nothing on screen said why (it reads like an
uncertainty result, not a cutoff mismatch). The MC settings tooltip even said **"Cutoffs match the
pay summary"** — an invariant the code documented but did not enforce. Separately, only the pay
summary loaded the project's **saved** default cutoffs; the other four ignored them and showed frozen
literals, so a saved cutoff set never propagated.

Fixed at the root: one shared `src/ui/cutoffs.ts` — a canonical `DEFAULT_CUTOFFS` (VSH 0.5 / PHIE 0.1
/ SWE 0.6 / PERM off) and one `loadCutoffDefaults()` (the saved `cutoffs/__default__` document merged
over the constant → always a complete, finite set). **All five** panes (cutoff editor, pay summary,
Monte Carlo, report, Results-QC) now seed from it, and the cutoff editor's save-fallbacks route
through the same constant. The defaults are now **un-copyable**: "matches the pay summary" is
structurally true, and every pane honours the user's saved cutoffs.

Not a physics change — cutoffs a user explicitly enters are untouched; only the **defaults** a pane
opens with, and only for MC and Results-QC (0.08 / 0.5 → 0.1 / 0.6). Anyone who wants 0.08 sets it
once via **Save default cutoffs** and it now flows everywhere, instead of living in two panes by
accident.

Verified: `tsc && vite build` clean. Headless (`scratchpad/cutoffs_check.mjs`, the shared merge
logic ported verbatim), **10 checks**: canonical fallback on missing / partial / garbage / NaN /
Infinity saved data; finite saved values pass through; `perm_min = 0` kept as a real value (not
"off"); and the two regressions — a fresh project now yields **PHIE 0.1 not 0.08** and **SWE 0.6 not
0.5**. Not verified live (needs the Tauri backend + a project with saved docs); the Try line covers
the click-through.

- [ ] **Try:** in **Cutoffs & Pay Summary** set custom cutoffs and click **Save default cutoffs**.
  Open **Monte Carlo**, the **Report**, and **Results-QC** — each should now open pre-filled with
  those saved values (before, MC and Results-QC showed 0.08 / 0.5 regardless). On a fresh project all
  five should read VSH 0.5 / PHIE 0.1 / SWE 0.6. Run MC and the pay summary with defaults — the
  net-pay now rests on the same cutoffs.

## Round 70 — V6: Raincloud plots in the Vega panel (your PtitPrince ask) (2026-07-24)

A requested feature, not a review item. New **Raincloud** chart type in the Vega panel: per group a
half-violin KDE **cloud** (top), a **box** (IQR + median + Tukey whiskers, middle), and a jittered
strip of raw **rain** points (bottom). A **Group** dropdown drives it — *By zone* (each sample
assigned to the zone whose interval contains its depth) or *any curve* (rounded to categorical
classes: rock-type / facies / RT). It shares the value (X) axis; Y / Colour / Trend don't apply and
dim out. Themed, exportable (PNG/SVG/PDF) and last-used-persisted like the other Vega types.

Design worth recording: Vega-Lite has no native violin, and its density / boxplot / facet paths
fight the panel's `width:"container"` autosize (every other chart type is single-view). So the
geometry — Gaussian KDE (Silverman bandwidth, robust via min σ, IQR/1.349), per-group quartiles +
1.5·IQR fences, and the jitter — is computed in **JS** and drawn with trivial single-view marks
(`area` / `bar` / `rule` / `point`) on a synthetic group-lane y-axis. That drops into the existing
sizing / export / repaint / theme machinery unchanged, and — the real payoff — makes the whole
thing **numerically verifiable** instead of needing a screenshot.

Data honesty, per the cardinal rule: samples outside every zone form an explicit **"(outside
zones)"** lane instead of being dropped; a group curve with **>24** distinct values is **refused**
with a message pointing at categorical curves, not silently binned into noise; samples missing a
group value are counted and surfaced in the status line ("· N with no <curve>").

Verification: `tsc && vite build` clean (vegaPanel lazy chunk 864.35 → 870.82 kB, main bundle
unchanged). Headless (`scratchpad/rc_geom.cjs`, real vega-lite compile + vega render), **13 checks,
all green**: geometry invariants — cloud never inverts, stays inside its lane and actually bulges;
quartiles monotonic; whiskers bracket the box; every rain point sits in its lane; recovered medians
match the injected distribution order — **and** the exact production spec shape both *compiles* with
`container` sizing and *renders* with the real empty-top-data + per-layer-data structure (the two
ways it differs from a toy spec). What I could **not** do: see it in the running app — the panel
needs the Tauri backend plus a well with curves, and the in-app browser was unreliable this session,
so there is no live screenshot. Correctness rests on the numeric geometry proof + the compile/render
check, not a pixel view.

- [ ] **Try:** open a **Vega Chart** panel, Type → **Raincloud**, X → **PHIE** (or Sw), Group → **By
  zone**. Expect one cloud+box+rain stack per zone, each labelled, sharing the value axis, medians
  landing where each zone's distribution centres. Switch Group to a **rock-type / facies** curve →
  one lane per class. Pick a **continuous** curve as Group → it should refuse with "pick a
  categorical curve". Hover a point/box for the tooltip; export SVG and PNG.

## Round 69 — R11: the depth-scale dropdown gets its themed background back in every palette (2026-07-24)

Small but real, and wrong in **all eight** theme blocks. `.lv-scale` — the log-view depth-scale
`<select>` (1:20 … 1:5000, top-right of the track toolbar, `logViewPanel.ts:167`) — set
`background: var(--bg)`. No palette defines `--bg` (the contract is `--bg-app` / `--bg-panel` /
`--bg-panel-alt` / `--bg-hover`), so the declaration was **invalid at computed-value time** and
`background` fell to its initial `transparent`. Being a native `<select>`, it never vanished — it
just quietly stopped matching the filled, themed look of every other control, worst on the brand
palettes (Pertamina/Halliburton/SLB/LAPI-ITB) where a transparent control on tinted chrome reads as
unstyled. This is exactly the failure mode a linter misses: no parse error, no total disappearance.

Fixed to `var(--bg-app)` — the canonical themed form-control surface, the same variable
`.form-control` uses and the one `.mm-dialog select` was explicitly switched to (there's a comment
at `styles.css:4399` enforcing "the same brand surface … so the whole app reads one theme"). The
depth-scale select now matches every other themed input, in all eight palettes.

Verification: grep-proved `--bg` is defined in **zero** palettes and `--bg-app` in **all eight**, so
the change is a deterministic computed-value swap (IACVT-transparent → the palette surface), not an
empirical guess; `tsc && vite build` clean. The one thing I could **not** do live: read the real
select's computed background in-app — the control only exists once a log view is open, which needs
the running Tauri backend, and a static-snapshot of a standalone repro can't run JS to read the
computed style. So this rests on the deterministic CSS proof + the grep evidence, not a screenshot.

- [ ] **Try:** open a log view, look at the depth-scale dropdown at the top-right of the track
  toolbar. It should have the same filled input surface as the other controls, not a see-through
  background — check one brand palette (e.g. Pertamina) where it was most visible.

## Round 68 — R10: a failed undo no longer vanishes silently while claiming success (2026-07-24)

A data-integrity finding, and squarely the cardinal rule of this whole review: a failed
operation must never look like a clean one. `undo()` did `undoStack.pop()` **before**
`await action.undo()` — it committed the stack change before the risky effect. Most reversals are
database writes (`upsertTop`, `updateStandardSample`, …), so when one rejects — a DB locked mid
autocorrelate-sweep, a since-deleted well, a value the Rust side refuses — the popped action was
**gone from both stacks**: un-undoable (popped) and un-redoable (the `redoStack.push` never ran).
Worse, both callers used `void undo().then((label) => …)` with **no** rejection handler, so the
`.then` fulfilment never fired — the status bar kept the *previous* success message — and the
rejection became a console-only unhandled promise. From the user's seat: press Ctrl+Z, the DB
reversal silently failed, the status bar still reads like it worked, and the action has disappeared
so they can't even retry it. The edit is still in the database, contradicting what the UI implies.

Fixed by only mutating durable state after the effect resolves. `undo`/`redo` now capture the
action, run the reversal inside a `try`, and on rejection **push it back where it was** (staying
reversible) and **re-throw** so the caller can report it. Both callers (`undo.ts` hotkeys +
`ribbon.ts` quick-access toolbar) grew a rejection arm: *"Undo failed — the change was not undone:
<err>"*. Both `undo`/`redo` are also now **serialized** through a small promise chain: a held Ctrl+Z
auto-repeats keydown, and without this the unawaited calls overlapped — running two reversals
against the single-writer DuckDB at once. The chain reverses one action at a time, in order, and
absorbs each outcome so one failed reversal doesn't stall the queue behind it. LIFO is preserved: a
top action whose reversal keeps failing blocks undoing *older* ones rather than silently skipping it
and reversing out of order.

Verification: `tsc && vite build` clean. Beyond that I ported the shipped `serialize`/`undo`/`redo`
bodies **character-for-character** into a headless Node harness (`scratchpad/undo_check.mjs`, real
promise scheduling, stub stacks) — 16 checks, all green: a rejected undo keeps its action and
rejects the promise; nothing leaks to the redo stack; a held Ctrl+Z reverses newest-first with max
one DB write in flight and never double-reverses the same action; a transient failure is retried by
the next request once the cause clears; LIFO holds throughout. The harness even caught a wrong
assumption of mine (I expected a persistently-failing top action to fall through to an older one —
it correctly does not). This is a stronger verification than R9 got; what it does **not** cover is
the live desktop path (a real rejected Tauri write), which is the click-through below.

- [ ] **Try:** in the DB inspector, double-click a `standard_curves` cell and commit an edit, then
  make the underlying write fail on undo — easiest repro: start a long Autocorrelate sweep (holds
  the DB), then immediately press Ctrl+Z. The status bar must say **"Undo failed — the change was
  not undone: …"**, the Undo button must stay **enabled** (action still on the stack), and a second
  Ctrl+Z after the sweep finishes must then undo it cleanly. Nothing should vanish silently.

## Round 67 — R9: a hostile LAS well name can no longer run code (2026-07-24)

F4c, a genuine remote-code-execution chain, verified end to end before fixing:
`parsers.rs::extract_well_name` stores the `~W WELL` value **verbatim** (trims whitespace, filters
no characters — confirmed at `parsers.rs:552`), and `vegaPanel.ts:504` wrote `well.well_name`
straight into `innerHTML`. With `tauri.conf.json` carrying `"csp": null` (confirmed), an
`<img src=x onerror=…>` embedded in a vendor's LAS header parses into the live document and runs.
The finding traces it on to the unscoped `save_png` write → a `.bat` in the Startup folder. LAS
files come from service companies, partners and clients, and this app ships client-brand palettes,
so it is meant to leave the developer's machine — "our tool executed a payload from a vendor's LAS"
is a reputational event, not a lint.

Fixed at the vector — escaping, not the sink — so it closes the path for **every** invokable
command at once, not just `save_png`. Scope per the finding's own recommendation: the three
`vegaPanel` message lines (well name + curve mnemonics, both LAS-supplied) plus the two DB-panel
error paths, building each with `textContent` via a new shared `messageNode` helper instead of
`innerHTML`. While there, the three byte-identical private `escapeHtml` copies
(dashboard/inspector/tops, plus inspector's `escapeAttr`) collapse into one `src/ui/safeDom.ts`, so
the next interpolated-innerHTML site has an obvious safe primitive to reach for. Left as backlog,
per the finding: the full 17-site sweep, a real `csp` (risks breaking vega-embed's inline styling),
and scoping `save_png`.

There is now **zero** interpolated-`innerHTML` in the three touched panels (grep-verified). The
DB-cell *value* renderers were already safe (`td.textContent`), so the exposure was only the
message/error lines.

Verification: `tsc && vite build` clean — which type-checks the `replaceChildren`/`messageNode`
usage — and the inertness holds by construction (`textContent` never invokes the HTML parser). I
wrote a standalone repro that runs the exact old vs new paths against a live `<img onerror>`
payload, but **could not execute it — the in-app browser was unresponsive this session**, and the
true end-to-end path also needs the Tauri backend plus a crafted LAS import, which I can't stage
here. So this rests on the construction argument and the source-level proof, not a live repro.

- [ ] **Try (optional):** import a LAS whose `~W` block has `WELL. <b>x</b> : WELL`, open a Vega
  chart on it with a zone that yields no samples. The empty-state line must show the literal text
  `<b>x</b>` (not a bold `x`, and certainly nothing executing). Same for a SQL error in the SQL
  console.

## Round 66 — R8: the test suite compiles from a fresh clone again (2026-07-24)

Not a runtime item — a build-integrity one from the F-sweep. `db.rs`'s WAL-recovery test
`include_bytes!`s two fixtures at **compile time**: `corrupt_torn.duckdb` and `corrupt_torn.wal`.
The `.wal` was committed, but the `.duckdb` was silently caught by the repo-wide `*.duckdb` ignore
rule (there to keep well databases out of the repo). A missing `include_bytes!` file is a hard
compile error, so **the whole `src-tauri` test suite could not build from a fresh clone or in CI** —
it only ever built for us because the file sat untracked in the working tree.

Before versioning it I checked it carries no well data: the `.duckdb` holds only the DuckDB header
and version string, the `.wal` only `create_schema`'s DDL (table/column names). It is a
freshly-created, schema-only project torn mid-write — exactly what the recovery test needs, and
nothing a client would recognise. A scoped `.gitignore` exception now tracks both, with a comment
recording that check and why a synthetic pair can't substitute (the test comment already notes a
garbage WAL doesn't reproduce the same DuckDB internal-error path).

Verified with `git archive HEAD`: a fresh checkout now materialises both fixtures (12288 + 3707 B),
byte-identical to what the test runs against. No code or test logic changed.

- [ ] **Try (optional, for CI/handover):** clone the repo somewhere clean and run
  `cargo test --lib` in `src-tauri`. It must compile — before this it failed at
  `include_bytes!("../tests/fixtures/corrupt_torn.duckdb")` with "No such file or directory".

## Round 65 — R7: the Cancel button is gone from jobs it could never stop (2026-07-24)

This is the **other half of R3's own acceptance criterion**, which I only did half of at the time.
R3 said: for each job kind, *either observe the cancel flag, or do not render the button.* R3
made "Cancelled" honest after the fact — a run that never observed the flag reports Completed, not
a false Cancelled. But the button was still offered on every active job, including the ~20-odd
monolithic ops that cannot observe it. Clicking it did nothing, silently. A control that does
nothing is the same lie R3 set out to remove, just on the other side of the click.

The split is structural, not a hand-maintained list. A `run_simple_job` worker is a bare
`FnOnce() -> Result` — it is handed **no** `JobHandle`, so it *cannot* poll the flag; every render,
export and single subprocess goes through it. A `run_job` worker gets a handle and every current
one polls (Import LAS, Equation, Module, Monte Carlo, ML, SandiMin, and the workflow chain). So
`run_simple_job` hardcodes `cancellable = false` and `run_job` takes it as an **explicit
parameter** — a future non-polling `run_job` caller is forced to pass `false` and cannot silently
inherit a button that would do nothing.

Active jobs that aren't cancellable now show a muted "can't be interrupted" tag where the button
was, so it reads as a deliberate status rather than a missing control.

cargo **373/0/7** (one new test: `cancellable` reaches the `JobView` both ways), release build and
`tsc && vite build` clean.

Not browser-verified — the panel only shows a button when there is a live job, and jobs exist
only under the Tauri backend, which `npm run dev` alone does not start.

- [ ] **Try:** run a per-well operation with many wells (a **Module** run, or **Monte Carlo**) and
  confirm the Processing panel still shows a working **Cancel** — click it and the run must stop
  early, reported Cancelled.
- [ ] **Try:** run a monolithic op — **Report → export PDF**, or **Composite → export SVG**, or a
  **core/tops/SCAL import**. The card must show **"can't be interrupted"** instead of a Cancel
  button. (Before this, it showed a Cancel that did nothing.)

## Round 64 — R6: the app can no longer fail to start without telling you (2026-07-24)

This was on the deferred list from the F-sweep, and it is the worst user-facing item on it.

Three `.expect()` calls ran **before the window was created** — `init_db_resilient` plus the two
launch migrations. The release profile sets `panic = "abort"` and `windows_subsystem = "windows"`,
so any one of them failing killed the process with **no window, no dialog and no console**. You
double-click SandiBumi and *nothing happens*. Nothing to read, nothing to send me.

`init_db_resilient` self-heals a corrupted WAL and nothing else, and the likeliest trigger is
completely mundane: **DuckDB takes an exclusive lock, so launching a second SandiBumi while the
first still has the project open used to silently kill the second one.** A read-only volume, a
network drive that dropped, or a file written by a newer DuckDB did the same.

The runtime path was already graceful — `open_project` returns a `Result` and reports failures
properly. Only startup panicked. So the open-and-migrate sequence is now shared between the two
(`project::open_and_migrate`), and startup treats failure as a value:

1. Open the real project. Normal case, unchanged.
2. If it fails → open a throwaway `sandibumi-recovery-<stamp>.duckdb` in the temp folder, so the
   app starts and can explain itself.
3. If *that* fails → memory only.

All three land you in a running app showing a dialog that names the file, quotes the DuckDB error
verbatim, says plainly that **your project file has not been changed**, and points at the likely
cause. The failed project is deliberately **not** added to the recents — a file that would not
open should not be the first thing tried at the next launch. "Save As" follows the recovery, so a
recovered session cannot checkpoint the temp database and then copy the project that never opened.

Two tests pin the contract startup depends on: an unopenable path returns `Err` rather than
panicking, and a fresh recovery file really is created with a working schema. cargo **372/0/7**,
release build and `tsc && vite build` clean.

I could not exercise this end-to-end myself — it needs two real app instances against a real
project, and the first instance would open your working file read-write.

- [ ] **Try:** launch SandiBumi normally and confirm nothing has changed. Then, **with it still
  open**, launch it a second time. The second window must appear (it used to not appear at all)
  with a dialog naming your project and the lock error. Click Continue — you should be in an
  empty temporary project. Confirm your real project is untouched: close both, reopen once, and
  check your wells are all still there.
- [ ] **Try:** check the recents dropdown after that — the failed project must **not** have been
  pushed to the top of the list by the second instance.

## Round 63 — refining R1–R5: one regression R5 introduced, one defect it met (2026-07-24)

I re-read the five landed diffs adversarially instead of trusting my own summary of them. Two of
the six findings are real defects; the rest are hardening. **Round 62's claim that saving before
the editor mounted was "already safe" was wrong** — see the strikethrough below.

**1. R5 introduced a data-loss window (the important one).** `renderEquationEditor` calls
`this.editor?.destroy()` but never nulls the field. `destroy()` tears down the DOM yet leaves
`view.state` readable — **a destroyed view is not a null view** — so `readFormIntoCurrent` kept
answering with the *previously open* equation's text. That was harmless while the mount was
synchronous, because the field was reassigned on the very next line. R5 put an `await` in that
gap. Result: open equation A, pick equation B, hit **Save** before the CodeMirror chunk finishes
loading, and **A's script is written into B**. The guard I described in Round 62 as already
present did not exist; it does now (`this.editor = null`).

**2. Cancelling a LAS import still reported every file as imported.** R3 added a cancel path that
returns an entry with neither a well nor an error, and `ribbon.ts` counted success as `!r.error` —
so cancelled files counted as imported. Cancel an import of 120 files at file 75 and the status
line read **"Imported 120/120 well(s)"**, with that same sentence written into the permanent
History, followed by 45 per-well notes each saying "cancelled". Exactly the class of defect R4
existed to close, created by R3 landing next to it. Counting is now partitioned on `well_id` —
the only field that proves a well row was actually committed — and cancelled files are reported
as their own count.

**3. The R1 wire guard had a hole on the Rust side.** `SPEC_FIELDS` was hand-maintained and only
TypeScript was compared against it, so a field added to the Rust struct carrying
`#[serde(default)]` — the one shape that deserializes happily forever — could sit there
permanently unknown to `ipc.ts`. The contract is now also checked against **serde's own** field
list, recovered from the `deny_unknown_fields` error text. Proven by dropping a name from the
contract and watching it fail. Worth noting: adding a field to the struct *also* breaks the build
outright, because the tests construct it with struct literals — an incidental second layer I
hadn't credited.

**4–6. Hardening.** The dashboard's row filter set its "n excluded" counter as a side effect while
the CSV handler called it outside the render path, so the note could describe a different
selection than the table — now returns the count with the rows. The out-of-range parameter check
rejected non-finite zone values but let non-finite request values through (unreachable today, as
JSON carries neither NaN nor Infinity — but two rules where there should be one). Plus the
"—" explanation sentence, which parsed as gibberish because the em dash it was describing sat
mid-sentence unquoted.

cargo **370/0/7**, release build and `tsc && vite build` clean. Eager chunk 664.53 kB (+0.18 kB).

- [ ] **Try (the R5 fix, most important):** Inspector → Equation. Open an equation with a
  distinctive script, then pick a **different** equation from the dropdown and hit **Save**
  immediately — before the editor finishes appearing. The saved script must be the one you
  selected, not the one you were just looking at. Re-open both to confirm neither was overwritten.
- [ ] **Try (the import fix):** start a LAS import of a large folder, hit **Cancel** partway.
  The status line must read "Imported *n*/*N* well(s). *m* cancelled before import." with
  *n* matching the wells that actually appeared in the tree — not *N*/*N*. Check the History
  panel says the same thing.
- [ ] **Try (the dashboard fix):** Field Dashboard with at least one uninterpreted well in scope —
  the "*n* interval(s) excluded" note must match what the table shows, and **Export CSV** must
  contain only the interpreted rows.

## Round 62 — R5: 461 kB of CodeMirror off every launch (2026-07-24)

I suspected this during scouting — CodeMirror is a dependency and the Vega spec editor is
documented as dynamic-importing it, yet **no CodeMirror chunk appeared in the build output at
all**. F4a found where it went: `inspectorPanel.ts` imported it **statically**, so the whole CM6
stack sat in the eager startup bundle — **461.3 kB, 41.0% of it** — loaded on every launch for a
panel most sessions never open. That also silently defeated `vegaPanel`'s own dynamic import:
once a module is in the eager chunk, deferring it elsewhere buys nothing.

The Inspector now dynamic-imports it the same way vegaPanel does, and fetches the Python language
mode **only** when the equation is Python — so a Rhai-only session never pays for the lezer parser
either.

The mount became async, which needed two guards: a generation counter, so a re-render (equation
picked, language switched) that lands while the import is in flight owns the host and the stale
mount drops itself; and a check that the host is still connected. ~~Saving in the window before the
editor mounts was already safe — `readFormIntoCurrent` falls back to the stored script rather than
reading a null editor.~~ **← wrong, corrected in Round 63.** It needed a third guard: the editor
field was destroyed but never nulled, and a destroyed CodeMirror view still answers with the *old*
equation's text.

**Measured, not estimated:**

| | before | after |
|---|---|---|
| eager `index` chunk | 1,125.01 kB | **664.35 kB** |
| CodeMirror | in the eager chunk | 3 lazy chunks totalling **461,537 B** |

That 461,537 B matches F4a's predicted 461.3 kB to the byte. The old baseline was quoted in three
places across the tracker and the review prompt; all now record the new one.

- [ ] **Try:** launch the app and confirm it feels no different — then open **Inspector →
  Equation**. The editor should appear after a brief first-load (the chunk fetching), then behave
  exactly as before: syntax highlighting on a Python equation, none on Rhai. Switch the language
  dropdown a few times quickly — the editor must track the last selection, not a stale one.

## Round 61 — R4: four places that reported success they hadn't earned (2026-07-24)

Your cardinal rule is that a degraded or failed result must never be presented as a clean one.
The review found four live violations; this closes all four.

**1 · Monte Carlo swallowed module errors.** A failed chain step was dropped with `if let Ok`,
leaving the pool unchanged — so every downstream step read NaN and the study came back as a
**P10 = P50 = P90 table of zeros** with nothing to explain it. The trigger needs no unusual setup:
`gascorr` with `OPT_GATE = FLAGGED` (the manifest **default**) on a well where `condflag` was
never run. Its own guard exists to stop exactly this, and the message it raises is the actionable
one — and it was being thrown away one call site from where it was written. The first failure is
now captured and the well is reported **Failed** carrying the module's own text.

**2 · A failed full-curve load reported a clean import.** When the generic-store load fails, the
six standard curves are in but PEF, CALI, DTS and any second run are not. That went to `eprintln!`
only — invisible in a release build — while `ImportResult` said success. Every later module that
resolves those mnemonics silently gets all-NaN, with no trace of why. It now rides in the existing
per-well warning. (The import status line also said "N well(s) had depth issues" for *any*
warning; it now says "imported with warnings" and lets the per-well notes speak.)

**3 · Pay summary printed a fabricated zero.** A well whose VSH/PHIE/SWE were never computed
classifies to NaN everywhere, leaving Net 0.0 / N/G 0.00 / HPV 0.00 — **byte-identical to a
genuine wet zone**, and `report.rs` puts it in a client PDF. Rows now carry `n_classified`; when
it is 0 the dialog shows "—" with a note, the PDF prints "-", and the **Field Dashboard excludes
the row entirely** — there, zeros would have dragged every median and box plot down with data
that does not exist, which is worse than a mis-rendered cell.

**4 · The ML dialog claimed wells it never wrote.** It reported the *scope* count, so a k-means
run on a 12-well group where only 2 wells have NPHI+RHOB said "wrote FACIES_ML to 12 well(s)" —
and wrote that into the **permanent History**. The backend was honest the whole time; only this
dialog lied. Now `ok/total`, with "N well(s) need attention", and no History entry at all when
nothing was written.

cargo **370/0/7**, tsc + build clean.

- [ ] **Try (1):** build a chain with `gascorr` (leave `OPT_GATE` at FLAGGED) on a well where you
  have not run `condflag`, and run it through Monte Carlo. Before: a tidy table of zeros. Now:
  the well is marked Failed with gascorr's own explanation.
- [ ] **Try (3):** import a LAS and press **Compute Summary** without running any interpretation.
  Before: Net 0.0 / N/G 0.00 / HPV 0.00, indistinguishable from a wet well. Now: "—" plus a note
  telling you to run VSH/PHIE/SWE. Check the Field Dashboard too — those rows are excluded and
  counted.
- [ ] **Try (4):** run ML on a group where only some wells carry the feature curves, then open
  **History**. It should record `ok/total`, not the full group.

## Round 60 — R3: Cancel stops telling you it worked (2026-07-24)

Found independently by **two** review passes that weren't told about each other (F2d and F5e) —
which is why I trusted it before reproducing it.

**The lie.** A Cancel button is rendered for every active job, but only about **5 of ~27 job
kinds** ever read the flag. The rest ran to completion, **committed their writes**, and were then
reported as **"Cancelled"** — with every item ticked green. Pick the wrong folder, start a 120-file
LAS import, hit Cancel at file 3: all 120 wells were still created, the status bar said
"Imported 120/120 well(s)", and the Processing card said Cancelled. Two contradictory reports of
the same run, and 120 wells you thought you'd stopped.

**The systemic fix is one idea:** *Cancelled* must mean the work actually **stopped**, not that
you clicked. `JobHandle::is_cancelled()` now records the fact that a worker **observed** the flag,
and `run_job` finalizes on that observation instead of on the flag. A worker that never polls
cannot have drained early, so its honest report is **Completed** — the cancel simply arrived too
late. This corrects every job kind at once, with no per-call-site churn.

Two paths read the raw flag instead of going through `is_cancelled()` — chain steps and module
runs. Chains already set their own terminal state so they were fine, but **module runs were not**:
they would have started reporting a genuinely drained run as Completed. That is the same lie in
the opposite direction, so those mark the observation explicitly. I caught this by tracing the
raw-flag readers rather than by a test failing.

**Cancel is now real in three more places**, each a per-item loop that simply never polled:
LAS import (checks *before* each DB write, so it stops wells being created), Rhai equations
(previously the Cancel button's behaviour depended on the equation's **language** — the Python
branch drained, the Rhai branch didn't, same job kind, same button), and the ML write-back loop.

cargo **369/0/7**, two new tests pinning the distinction the whole fix rests on: a set flag alone
is not evidence, and observation is shared across the handle clones rayon workers hold.

- [ ] **Try:** select a folder of ~20 LAS files, Import, and hit **Cancel** after a couple land.
  Before: all 20 imported and the card said Cancelled. Now: the remaining files are skipped and
  marked "cancelled", and the card says Cancelled *because it genuinely stopped*.
- [ ] **Try:** run any **export or render** (composite PDF, report), hit Cancel. It cannot be
  interrupted, so it finishes — and now correctly reports **Completed**, not a false "Cancelled".

**Still open** (deliberately not in R3): the Processing panel offers a Cancel button for *every*
job with no capability check, so on monolithic ops it is now honest but still inert — `JobView`
needs a `cancellable` flag and the button should be hidden when false. Also unfixed: Monte Carlo
polls only between wells (a single-well 100k-realization run is uncancellable), Report batch uses
the single-unit helper so it has no per-well progress, and both Autocorrelate commands hold the
DB lock across the whole sweep.

## Round 59 — R2: three panics reachable from your own data (2026-07-24)

All three came out of the F2a review pass (`docs/review_sweep/F2.md`). Release builds set
`panic = "abort"`, so none of these was a caught error — they killed the run, and one of them
poisoned the DB mutex for the rest of the session.

**1 · A `NaN` top depth panicked Auto-correlate.** `pandas.to_csv(na_rep='NaN')` and `np.savetxt`
write a literal `NaN` for a missing marker, and `f32::from_str` parses that happily — nothing
between the tops importer and the database tested finiteness. The NaN then reached
`markers.sort_by(partial_cmp().unwrap())`. Worse than a dead run: the panic unwound **while the DB
lock was held**, poisoning the mutex, so every later query panicked too and the app was unusable
until restart. Fixed at both ends — the importer drops a non-finite depth row, and the sort no
longer unwraps. An unorderable depth is not a top.

**2 · A percent-entered zone parameter panicked the module run.** `f64::clamp` asserts
`lo <= hi`, and the bounds are themselves parameters: entering irreducible water saturation as
`25` instead of `0.25` produced `limit(swt, 25.0, 1.0)`. The zone override is *designed* to beat
the module dialog, so it also skips the dialog's range check — and the DB Inspector edits
`zone_params.value_num` raw. Now enforced at the one choke point where every path converges
(`workflow::resolve_param_arrays`), using the `min`/`max` the manifest **already declared**.
**Rejected, not clamped** — silently pulling 25 down to the spec maximum would have answered with
a plausible-but-wrong saturation. The error names the parameter, the value, the zone and the valid
range. `modules::limit` was hardened too, as a backstop for future modules.

**3 · An infinity panicked the synthetic-log KNN.** `f32::from_str` returns `inf` for a cell like
`1.0E+40` or the literal token `inf`, and everything downstream screens for missing with
`is_nan()` only — so it survived into the compute cores, where `inf − inf` made the z-score NaN
and `partial_cmp` on two NaNs panicked the neighbour sort. The DLIS importer already stripped
exactly this; the LAS path did not. Fixed at three points, because the verifier found the LAS cell
is *not* the likeliest source: **the Rhai equation engine is** — `1.0/0.0` and `exp(100)` both
reach `inf`, and the existing guard only rejected an *entirely* non-finite column, so a single
infinite sample was written to a computed curve that could then be picked as a predictor. So:
the LAS value path maps non-finite to missing, equation output does the same, and the KNN skips a
non-finite distance instead of sorting on it. The z-score floor also changed from `if *s < 1e-9`
to `if !(*s >= 1e-9)`, because a NaN std slipped straight past the old form.

cargo **367/0/7** — five new tests, each fed the *exact* malformed input from the finding rather
than a synthetic near-miss.

- [ ] **Try (1):** make a tops CSV with a `NaN` in the depth column (or export one from pandas with
  a missing marker), import it, then run **Auto-correlate** from that well. Before: the run died
  with `worker thread failed` and every later action failed until restart. Now: the bad row never
  imports, and correlation runs on the real tops.
- [ ] **Try (2):** Zones → set `SWT_IRR` to `25` for zone `*`, then run **SW-Archie**. Before: opaque
  `worker thread failed`. Now: a message naming `SWT_IRR = 25`, the zone, and the valid range.
  Set it to `0.25` and the run proceeds normally.
- [ ] **Try (3):** run an **equation** like `1/0` or `exp(1000)` into a new curve, then use that curve
  as a predictor in **Synthetic Log (KNN)**. Before: the app aborted. Now: those samples read as
  missing and the prediction runs.

**Not fixed here** (out of R2's scope, still open in F2a): the startup `.expect()` on DB init —
a locked or newer project file kills the process before the window exists, with no dialog, and
`startup_path()` re-selects the same unopenable file every launch.

## Round 58 — R1: the net-flag polygon actually works now (2026-07-24)

**Correction to Round 47.** The flag-polygon feature has never worked — not once, since it shipped
in `a4e05e9`. `NetFlagSpec` was declared in **camelCase** in `ipc.ts` while `netflag.rs` expects
**snake_case**, so `run_net_flag` could not deserialize a single request; `NetFlagResult` had the
same slip in the other direction, so the status line read `undefined`. I marked Round 47 verified
on the strength of a frontend twin-count check that never crossed the wire. That was the gap:
the lasso's live in-polygon count is computed in the browser, so it agreed with itself perfectly
while the backend was rejecting every call.

Found by the F1c pass of the engineering review (`docs/review_sweep/F1.md`), then confirmed by
hand against both files before touching anything.

**The fix went into TypeScript, not Rust.** Struct DTOs cross this wire in snake_case — Tauri
camel-cases only the top-level command *argument* key (`{ spec }`), never the fields inside it,
and `rename_all` is used in this codebase only on enums for their string tag values. Every other
DTO already follows that (LorenzResult, ZoneParamEntry, HighlightEntry). NetFlag's TS was the
outlier, so adding `rename_all` to the Rust would have made it the one struct with a different
wire shape.

Three tests now hold the contract, because a Rust-only serde test cannot see `ipc.ts` — and the
two sides disagreeing *while each was internally consistent* is exactly what happened:

- the spec deserializes from the **literal JSON `crossplotPanel.ts` sends**, and the old camelCase
  shape is asserted to be **rejected** rather than half-parsed into defaults;
- the result serializes under the names the status line reads;
- a **cross-language** test reads the real `src/ipc.ts`, extracts both interfaces, and fails on
  drift. I proved it fires by regressing `ipc.ts` back to the shipped-broken shape and watching it
  fail, then reverting.

`NetFlagSpec` also gained `deny_unknown_fields`, so a TS field Rust doesn't know now fails loudly
instead of being silently dropped — the silent direction of the same class.

cargo **362/0/7**, tsc + build clean.

- [ ] **Try:** open a Crossplot on a well with PHIE/RHOB, draw a lasso polygon around the clean-sand
  cloud, **Write Net Flag** with a name like `NET_TEST`. Before this fix the button did nothing and
  the status line said `Net flag failed: …`. It should now report
  `Net flag NET_TEST: <n> / <m> samples net (<k> written)` with a real curve name, and `NET_TEST`
  should appear in the Curve Catalog and plot as a 0/1 track in the log view. Check the count
  against the lasso's own live in-polygon readout — those two numbers agreeing is the thing that
  was never actually tested.

## Round 57 — SandiMin: RMS vs core (2026-07-24)

Closes the second first-half residual (playbook #2). RECON/incoherence only says the model
reproduces **its own input logs** — it cannot catch endpoints that are wrong in a way those logs
can't see. Core plugs are an *independent* measurement, so a run on a cored well now also reports
how far the solution sits from them.

Three numbers per cored well, each an **RMS of (model − core)** with a signed **bias** (so the sign
says which way the model reads) and the plug count:

- **Core φ vs PHIE** *and* **vs PHIT** — both, because which one a plug should match depends on the
  drying protocol (oven-dried drives off clay-bound water → PHIT; humidity-dried retains some →
  nearer PHIE). Showing the bracket is more honest than picking one for you.
- **Core ρg** — the grain density implied by the solved **solid** volumes (Σv·ρ / Σv over the
  non-fluid components). This is the one that tests the **mineral model** specifically: bound water
  is a fluid here, so it correctly sits outside the sum, matching a cleaned-and-dried plug. Where
  RHOB was not itself an input tool this is a fully independent check.

Plugs tie to the nearest **solved** sample within 1 m (the same tie-in tolerance already used for
core elsewhere); an unsolved sample is skipped rather than matched. A well with no core — or an
all-null column — shows nothing at all, never a 0.000 that would read as a perfect match. Plugs
outside a physically valid range are dropped rather than fitted, so a φ column imported in **percent**
reports "no fit" instead of a confident-looking RMS of ~14.85.

**Try:** open **SandiMin** on a well that has **core** loaded, set up your usual mineral model, and
**Run**. A new **Core calibration** block appears under the results table. (1) Check the **plug
count** is roughly what you'd expect over the solved interval. (2) Look at **Core ρg** — on a sound
quartz/clay model it should sit within a few hundredths of a g/cc; a large bias here points at a
matrix-density endpoint rather than at the logs. (3) Compare **vs PHIE** against **vs PHIT** — your
plugs should sit nearer whichever matches how they were dried; a big gap on *both* is worth a look
at the clay-bound-water setup. (4) Run a well with **no core** and confirm the block is absent
entirely. (Verified: cargo **359/0/7** including two new tests — hand-computed RMS/bias literals
covering the depth tolerance and the NaN-skip, and a full run asserting the fits appear only for the
cored well, with a percent-φ and a 999.25-ρg plug planted *inside* depth tolerance to prove the value
gate rejects them rather than the depth gate; tsc + production build clean.)

## Round 56 — Monte Carlo: per-parameter uncertainty widths seeded from IP (2026-07-24)

Closes a first-half residual (playbook #1). Until now, adding an uncertain parameter gave it a
**generic** width — 10% of its own value as the normal σ, 20% as the uniform/triangular half-range.
That is wrong whenever the parameter isn't naturally relative: **RHO_MA = 2.645** was getting
σ = **0.26 g/cc** (a ±10% matrix density — about **9× too wide** against the ±0.03 convention), and
**GR_SH = 120** was getting σ = 12 API where the field convention is ±10.

Defaults now come from a table imported from IP's `MonteCarloDefaults.par` (Tier-A), so each
parameter gets a width **in its own units**: `M`/`N` ±0.2, `A` ±0.1, `GR_MA`/`GR_SH` ±10 API,
`RHO_MA` ±0.03, `RHO_FL` ±0.02, `RHO_SH` ±0.05, `RHO_DSH` ±0.1, `NPHI_SH` ±0.05, and the two
resistivities `RW`/`RT_SH` as **±20% of their value** (they *are* naturally relative). A muted **IP**
badge on the row marks a seeded width — hover it for the source. Anything unseeded (`C`, `SWE_IRR`,
`PHIE_MAX`, …) keeps the old generic width exactly as before. Widths stay fully editable — this only
changes what a **freshly added** row starts at.

Provenance, the mapping table, and the σ reading adopted (the tabulated shift is taken as **one
standard deviation**; IP's file doesn't state its percentile convention, so this is SandiBumi's
documented choice, not a claim of matching IP run-for-run) are banked in
`docs/ref_monte_carlo_seeds.md`.

**Try:** open **Monte Carlo** on the default chain (VSH → Porosity → SW-Indo). (1) **+ Add uncertain
parameter** and pick **RHO_MA** → confirm the **std dev** reads **0.03** (not 0.26) and an **IP**
badge sits on the row; hover the badge for the source. (2) Add **M** → σ **0.2**; add **GR_SH** → σ
**10**; add **RW** → σ **0.02** (= 20% of 0.1). (3) Switch one of them to **triangular** → confirm
min/mode/max straddle the value by that same width (M → 1.8 / 2.0 / 2.2) and the sparkline redraws.
(4) Pick a parameter with **no** badge (e.g. **SWE_IRR**) → confirm it still gets the old generic
width, and that its fields stay column-aligned with the badged rows. (5) Run it and confirm the
tornado still reads sensibly — the point of the change is that the P10/P90 spread is now built on
priors with the right units. (Verified: tsc + production build clean, MC dialog still a lazy chunk,
main bundle unchanged; a headless check evaluates the real source's seed table and width maths — 36
assertions, all pass — including that every unseeded parameter's fallback is byte-identical to the
old behaviour and that a % seed on a zero value degrades to the floor instead of collapsing the row
to a point mass. No Rust changed.)

## Round 55 — Vega-Lite interactive charts, V5: density + trend overlay (2026-07-24)

Builds on V4 (Round 54). The capstone adds two analytical modes:

- **Density** — a new **chart type**: a 2D binned heatmap (viridis by bin count). This is the view
  for clouds too dense to read as a scatter — a Mahakam NPHI–RHOB cloud overplots into a blob, but
  the binned counts show where the mass actually is. Hover a cell for its bin range and count. (Like
  the histogram it's an aggregate, so it doesn't take part in brushing.)
- **Trend** — a regression overlay on the **Scatter**: tick **Trend** to draw a fit line plus its
  **R²**, with a method dropdown (**linear / log / exp / pow / quad**). It layers over the point
  cloud, so hover / brush / zoom on the points still work; log/exp/pow assume positive data. Works
  alongside a Colour curve and a Zone.

**Try:** open a **Vega Chart**. (1) Set **Type = Density** on a dense NPHI–RHOB pair → confirm a
viridis heatmap of counts, and hover a cell for its bin + count. (2) Back on **Scatter**, tick
**Trend** → confirm a fit line + an "R² = …" label appear; change the **method** (e.g. **log**, the
por–perm shape) and confirm the line + R² update. (3) With Trend on, set a **Colour** curve and a
**Zone** and confirm all three coexist. (Verified: tsc + offline build keep vega + CodeMirror lazy,
main bundle unchanged; a headless check renders the density spec and every trend method — each shows
an R² label, keeps the brush signals, and still dims on a shared brush through the layering. The
headless pass caught two real layered-spec bugs — a duplicate `grid_x` signal and an unresolved
`brushedActive` — now fixed by splitting the params across the layer. The live density hover and the
trend line/R² on field data are what this Try line confirms.)

## Round 54 — Vega-Lite interactive charts, V4: export + spec editor (2026-07-24)

Builds on V3 (Round 53). The Vega Chart panel becomes a report/export surface and gains an escape
hatch for power users:

- **Export.** New toolbar buttons: **⧉ Copy** (PNG to clipboard), **⭳ Image** (save PNG), **⭳ SVG**
  (a true-vector SVG from vega's own renderer), **⎙ Print**. Same affordance as the crossplot /
  histogram export.
- **Spec editor.** **⧉ Spec** reveals a JSON editor showing the *effective* Vega-Lite spec (with the
  data rows elided). Edit the grammar — point size, a title, an extra layer, a scale — and **Apply**;
  the chart re-renders with your override and the current rows are re-injected, so the control bar
  still drives which curves / zone fill it. **Reset** returns to the generated spec. Changing the
  **chart type** clears an override (the grammar is type-specific). Invalid JSON is reported inline;
  an invalid spec shows "render failed" rather than a broken chart. (Linked brushing keeps working
  through an override.)
- **Opens where you left off.** The control selections (type / curves / zone) are remembered, so a
  new Vega chart opens with your last settings.

**Try:** open a **Vega Chart**. (1) **⭳ Image** and **⭳ SVG** — save each and open the files; **⎙
Print**. (2) Click **⧉ Spec**, change something in the JSON (e.g. `"size": 20` → `120`, or add
`"title": "My chart"`), **Apply** → confirm the chart changes; **Reset** → confirm it reverts. Type
some invalid JSON and confirm the inline error. (3) Set Type = Line + a Zone, close the panel, open a
new Vega chart → confirm it opens as Line on that zone. (Verified: tsc + offline build keep vega **and
CodeMirror** as separate lazy chunks — the editor only loads when you open Spec; a headless check
confirms the spec round-trips through the editor, still renders, and keeps its brush signals, and that
an invalid override throws. The live save dialogs, printing and editor typing are what this Try line
confirms.)

## Round 53 — Vega-Lite interactive charts, V3: theme repaint + linked brushing (2026-07-24)

Builds on V2 (Round 52). The Vega Chart panel now joins the rest of the workspace — it repaints with
the theme and takes part in the shared brush:

- **Live theme repaint.** Switch the theme with a Vega chart open and it repaints in the new palette
  immediately (it re-embeds from the cached data, so no re-fetch). One deliberate trade: a theme
  switch resets the chart's zoom/pan back to full extent.
- **Brush → other panels.** On a **Scatter** or **Line**, **drag a box** over the points; the samples
  inside are published as the shared selection, so the crossplot, histogram and log view of the same
  well highlight the *same* depths (live, as you drag). A click on empty space (or a zero-size box)
  clears it.
- **Other panels → Vega.** When you brush in a crossplot (or any panel that publishes a selection),
  the Vega **scatter dims the un-selected points** so the shared samples stand out. (A line is one
  path, so it only emits; a histogram takes part in neither — its bars are aggregates.)
- **Gestures.** Because plain-drag now *brushes*, **pan moved to Shift-drag** and **zoom stays on the
  wheel**. Hover tooltips are unchanged.

**Try:** open a **Vega Chart** (Scatter, e.g. NPHI–RHOB). (1) Switch the theme (ribbon) and confirm
the chart repaints in the new colours. (2) **Drag a box** over a cluster and confirm a crossplot /
histogram / log view of the same well lights up the same samples. (3) Brush in a **crossplot** and
confirm the Vega scatter dims everything except those samples. (4) **Shift-drag** to pan and **scroll**
to zoom. (Verified: tsc + offline build keep vega a separate lazy chunk; a headless vega-lite→vega
compile+render check confirms the brush/pan event selectors and the array-form opacity condition are
valid and that driving the consume signals dims the right points — 2 bright / 238 dimmed. The live
drag/pan gestures are what this Try line confirms — the harness can't drive vega's pointer input.)

## Round 52 — Vega-Lite interactive charts, V2: control bar (type / colour / zone) (2026-07-24)

Builds on V1 (Round 51). The Vega Chart panel gains a real control bar so you can shape the plot
without leaving it:

- **Chart type** — **Scatter / Line / Histogram**. Scatter is the X–Y cloud; Line connects the
  samples in depth order (a trajectory through crossplot space); Histogram is the X curve's
  distribution (binned count).
- **Colour curve** (scatter only) — colour the points by a third curve on a **viridis** scale with a
  legend (e.g. NPHI–RHOB coloured by GR). "— None —" falls back to the theme accent.
- **Zone filter** — restrict the plot to a named zone's depth range (follows the top-interval like
  the other plots); "all" plots the whole well.
- Controls that don't apply to the active type dim out (Y on a histogram, Colour off scatter), so the
  bar reads honestly. Selections carry across a well switch.

**Try:** Plot ribbon → **Vega Chart**. Switch **Type** to Histogram (the X curve's distribution) and
to Line; on Scatter, set **Colour** to a curve (e.g. GR) and confirm the points take a viridis ramp
with a legend; set a **Zone** and confirm the plot restricts to it (status line shows the zone). Pan /
zoom / hover still work on scatter and line. (Verified: tsc + offline build; all three chart types
render non-blank canvases with a clean console on the dev server — the small-canvas note is just the
uncomposited preview pane, not the app.)

## Round 51 — Vega-Lite interactive charts, V1: engine vendored + one live chart (2026-07-24)

New feature (your "Altair on SandiBumi" ask, built as *interactive Vega-Lite in-app*): a chart
rendered by the real **vega** engine, vendored **offline** into the app. V1 lands the engine + one
live chart; richer controls, theme-repaint, brush-linking and a spec editor are V2–V4.

- **New "Vega Chart" button** on the Plot ribbon (next to Crossplot). It opens a well-bound panel:
  pick an X and a Y curve (defaults NPHI / RHOB) and it plots the selected well, following the Wells
  pane like the other plots. Hover for a tooltip; drag to pan; scroll to zoom — vega's built-in
  grammar-of-graphics interactivity, the thing the Canvas-2D plots don't give for free.
- **Offline + lazy.** `vega` / `vega-lite` / `vega-embed` are bundled into the app (no CDN, works with
  no network). The engine is ~850 KB, so it is a **lazy** chunk — it loads only the first time you
  open a Vega chart and stays out of the main startup bundle.
- **Themed** from the active theme's CSS vars (axes, grid, points), so it matches the brand themes.
  Colours are read when the chart builds; live repaint on a mid-session theme switch is V3.

**Try:** Plot ribbon → **Vega Chart** (select a well first). Confirm the scatter draws in your theme's
colours, then **hover a point** (tooltip = X / Y / Depth), **drag** to pan, and **scroll** to zoom.
Switch the X / Y curves and confirm it redraws. (I verified the render, theming and offline bundle
with a screenshot against synthetic data; the live pan / zoom / tooltip is what this Try line
confirms — the automated harness couldn't drive vega's canvas input.)

Note: `npm audit` flags 7 high-severity advisories in vega's dependency tree. I did **not** auto-fix
(it wants breaking changes). For an offline desktop app rendering local numeric data the exposure is
minimal, but say the word if you want me to look at pinning/patching them.

## Round 50 — Monte Carlo: per-row PDF preview sparkline (2026-07-24)

Playbook **#1 (Monte Carlo)** residual: *"per-row live distribution (PDF) preview."* Each uncertain-
parameter row in the Monte Carlo dialog now carries a small inline **sparkline of the distribution
shape** you configured — a bell for Normal (mean/sd), a flat-topped box for Uniform (min/max), a
triangle for Triangular (min/mode/max). It redraws **live as you type**, so you can see the shape
before running anything.

- Purely a preview — it reads the row's own `(kind, a, b, c)` and **never feeds the sampler**, so the
  P10/P50/P90 are untouched. Colours come from the theme (`--accent`/`--border`), so it repaints with
  the brand themes like the rest of the UI.
- Collapsed spreads don't go blank: `sd≤0`, `min==max`, or a NaN field renders a narrow **point-mass
  spike** (a delta). Swapped bounds (min>max) auto-normalize; a Triangular mode outside [min,max]
  clamps to the nearest edge — the preview always shows a sensible shape.
- Verified: `tsc` clean + **15/15 geometry assertions** on the exact path function (bell apex centred
  at the peak, box edges at the right fractions, triangle apex at the right x, every degenerate case →
  spike). The in-app pixel look is what this Try line is for.

**Try:** open **Monte Carlo**, add an uncertain parameter, and watch the little chart beside the
number fields. Switch the kind (normal → uniform → triangular) and edit mean/sd (or min/mode/max): the
sparkline should update as you type — a bell that narrows as you shrink sd, a box that widens with the
range, a triangle whose peak slides with the mode. Set sd to 0 (or min=max) and it should collapse to a
thin spike.

## Round 49 — Monte Carlo: physical-plausibility guard (impossible Sw>1 / PHIE<0 fraction) (2026-07-24)

Playbook **#1 (Monte Carlo)** residual: *"reject/flag impossible combos (Sw>1, PHIE<0) and report the
rejected fraction."* The MC engine now reports, per well, **how often a sampled parameter combination
drove the petrophysics out of physical bounds** — a QC signal that your input distributions may be too
wide.

- The trick: the chain's saturation/porosity modules **clamp** the final `PHIE`≥0 and `SWE`≤1, so the
  impossible values never reach the limited curves. But every one of them also emits an **unlimited
  companion** (`PHIE_DN`, `SWT_ARCH`, `SWE_INDO`, …) where the raw `Sw>1` / `PHIE<0` survives. The
  guard scans those (spec-driven: any produced `v/v` curve named `PHI*`/`SW*`), per realization, over
  the in-zone samples, and counts the ones outside `[0,1]`.
- **Reported, never excluded.** The module clamp already gives an impossible draw the physically-correct
  volumetric answer (an over-dense matrix → zero effective porosity; a supersaturated combo → fully
  wet), so those realizations are **valid low/high tails** — dropping them would bias P10/P90. So the
  headline percentiles are **unchanged**; you just get a new advisory line. A large fraction means
  "narrow your inputs," not "the result is wrong."
- The MC dialog's notes area gains one line per well: **⚠** with the fraction + a `Sw>1` / `PHIE<0`
  breakdown when impossible draws occurred, **✓** when every realization stayed in bounds, and a neutral
  **•** "not checked" when a well had no porosity/saturation to judge (never a fabricated clean pass).

**Verification:** three new `montecarlo.rs` unit tests — matrix density pinned below RHOB → `PHIE_DN<0`
flagged on 100% of realizations; cementation exponent pinned high → Indonesia `Sw>1` flagged (porosity
stays clean); a normal clean-sand study → 0% impossible. The headline HPV still computes in every case,
and the pre-existing reproducibility tests still pass **byte-identical** (the guard is purely
observational — it never touches the RNG or the reported percentiles). Full lib suite **357/0/7**, `tsc`
clean.

**Try:** open **Monte Carlo**, set up any run (e.g. vary `RW` or `M` with a wide spread on a real well),
and run it. Look at the notes area under the results: you should see a **✓ … within physical bounds** on
a well-behaved study. Now widen a distribution aggressively (e.g. `RW` normal with a big σ, or `M` up to
4) and re-run — the line should flip to **⚠ … % of realizations hit impossible petrophysics (Sw>1 …)**,
while the P10/P50/P90 HPV stay sensible. Tell me if the fraction looks off for what you dialed in.

## Round 48 — UI polish #9C follow-on: free-form net-flag polygon on the crossplot (2026-07-23)

The crossplot's scalar cutoff-box (Round 45) is now joined by a **free-form net-reservoir polygon**:
draw an arbitrary shape around a cloud of points and write its interior straight to a **discrete 0/1
net-flag curve** — the general case the rectangular cutoff can't express (e.g. a curved φ-k fairway, an
L-shaped sand window).

- A new **⬡ Net polygon** toolbar toggle enters draw mode: click to drop vertices, and a small bar
  shows **Undo point / Clear / Write net flag…** with a live `N / total points inside` readout. The
  polygon fills faintly, its edges + a dashed closing edge + a rubber-band to the cursor draw as you
  go, and — because vertices are captured in **data space** — it stays registered under zoom/pan.
- **Write net flag…** names the curve (default `NET_FLAG`) and calls the backend, which computes the
  flag over the crossplot's current depth window and writes it as a computed curve like any module
  output: **1** inside / **0** outside / **NaN** where a sample can't be evaluated (either input NaN,
  or ≤ 0 on a log axis — the same samples the crossplot excludes). Other views refresh so the new
  curve shows up.
- **`netflag.rs`** does the work: even-odd point-in-polygon run in the axes' **drawing plane** (log10
  on a log axis), so "inside the drawn polygon" is exact for log scales (straight screen edges are
  straight edges there) and matches the on-screen count. The frontend's live count uses an **exported
  twin** (`netPolygonContains`) of that same test, so the preview equals what gets written.

**Verification:** `netflag.rs` has 5 unit tests — concave (notched-square) point-in-polygon, a written
0/1/NaN curve over a synthetic cloud, the depth-window restriction, the ≥3-points / distinct-axes
guards, and a **log-axis** case (a decade box on a log X axis captures exactly the right samples and
rejects a ≤ 0 vertex). In-browser, the frontend `netPolygonContains` was checked against the *same*
cases and agrees with the backend on every one — linear, concave, and log. Adversarial review caught
one interaction bug (a double-click while drawing dropped two vertices **and** opened the Properties
dialog), now guarded. `tsc` + full lib suite green.

**Try:** open a **Crossplot** (e.g. a φ-k or NPHI-RHOB cloud), click **⬡ Net polygon**, and click
around the group of points you consider net; watch the inside-count update. Click **Write net flag…**,
name it (say `NET_POLY`), and Write — then add that curve to a **log view** track and confirm it reads
1 exactly where your polygon was, 0 elsewhere. Re-draw a different shape to overwrite.

## Round 47 — UI polish: true-vector PDF export for the Canvas-2D plots (2026-07-23)

The vector story is now complete: the **crossplot, histogram, and Pickett** plots also export a
**true-vector single-page PDF** — a portable, self-contained figure to drop straight into a Word/LaTeX
report — via a new **⭳ PDF** button in each plot's toolbar (and an "Export PDF (vector)…" right-click
entry), sitting alongside the ⭳ SVG button from Round 46.

- **`pdfExport.ts` — `PdfRecorder`**: the sibling of `SvgRecorder`. It drives the **same**
  `drawCrossplot` / `drawHistogram` / `drawPickett` code through a recording 2D context, but serialises
  every call into a **PDF content stream** (operators in points, bottom-left origin) instead of SVG — so
  again **no chart is re-implemented** and the PDF can't drift from the screen. Handles the full surface
  the plots use: affine transforms (rotated axis labels via the PDF text matrix), rectangular clips
  (`q … re W n … Q`), circles (as béziers), dashes, text alignment/baseline, and all the colour forms
  the plots emit (`#hex`, `rgb()`, `hsl()`).
- **Split of concerns**: the frontend owns only the *drawing operators*; the backend
  (`save_plot_pdf` → `composite::assemble_single_page_pdf`) wraps them in the PDF *document*
  (catalog, xref, Helvetica fonts) — reusing the exact, already-tested assembler that powers the
  composite-log PDF, so the fiddly document scaffolding lives in one place.
- Text renders in base-14 Helvetica (no font embedding, same as the composite PDF) and transparency is
  flattened against the plot background — *exact* for these plots, which only use alpha for gridlines /
  marginals drawn straight over that background. (The SVG export remains the fully device-independent
  option; the PDF is the portable single-file one.)

**Verification:** the new `assemble_single_page_pdf` has a Rust unit test (valid `%PDF`, one Page,
MediaBox at the requested point size, stream embedded); the full lib suite stays green (356 pass).
In-browser, against the real `PlotCanvas` draw methods (log X + inverted Y, every colour form): the
content stream has balanced `q`/`Q` and `BT`/`ET`, béziers/clips/dashes/text present, **no
NaN/Infinity**, and every colour operand in [0,1]; the text matrix was checked exactly for the
identity and the rotated-y-label cases. Adversarial review caught one fidelity slip (a forced round
cap/join where canvas/SVG use butt/miter), now fixed. `tsc` clean.

**Try:** open a **Crossplot / Histogram / Pickett**, arrange it how you like, then click **⭳ PDF** and
save. Open the `.pdf` in a viewer and zoom right in — text and curves stay razor-sharp — then drop it
into a report to confirm it embeds cleanly. Compare against **⭳ SVG** for the same chart: same figure,
two portable vector formats.

## Round 46 — UI polish #9B: true-vector SVG export for the Canvas-2D plots (2026-07-23)

The **crossplot, histogram, and Pickett** plots previously exported raster PNG only (the log
composite already had a vector path). They now export a **true-vector SVG** — infinitely scalable,
editable in Illustrator/Inkscape/PowerPoint — via a new **⭳ SVG** button in each plot's toolbar
(and an "Export SVG (vector)…" right-click entry).

- **`svgExport.ts` — `SvgRecorder`**: a recording 2D context that duck-types
  `CanvasRenderingContext2D` and serialises every draw call to SVG. A detached canvas carries the
  recorder via a private property that `PlotCanvas` reads, so the **same** `drawCrossplot` /
  `drawHistogram` / `drawPickett` code paints into the recorder — **no chart is re-implemented**,
  so the SVG can't drift from what's on screen. Handles the full surface the plots use: affine
  transforms (rotated axis labels), rectangular clips (incl. nesting), circles, dashed lines,
  alpha, text alignment + baseline, and the colorbar/marginal/regression overlays.
- The export re-runs each panel's **static** draw only (a shared `drawStatic` in the crossplot),
  so transient decorations — hover ring, brush highlight, cutoff shading, parameter handle — are
  omitted: you get the clean, publishable chart. Written to disk as UTF-8 through the existing
  save path (no backend change).

**Verification (in-browser, against the real draw code):** SVGs from all three panels parse as
valid XML (DOMParser), with the correct element counts (e.g. 249 points for a 250-pt cloud with one
NaN, 59 bars for a histogram), balanced/nested clip groups, correct affine composition
(translate∘rotate → exact matrix + mapped points), and no NaN/undefined/Infinity tokens — exercised
with marginals + regression + a viridis colorbar and with log axes. Adversarial review confirmed the
wired panels correct on all fronts and caught one forward-looking gap (a dropped `textBaseline`),
now fixed and re-verified non-regressive. `tsc` clean; no Rust changes.

**Try:** open a **Crossplot / Histogram / Pickett**, arrange it how you like (zoom, colorby, picks),
then click **⭳ SVG** in the toolbar and save. Open the `.svg` in a browser or vector editor and zoom
in — the text and curves stay razor-sharp (unlike the PNG). PDF-for-charts is the natural next step.

## Round 45 — UI polish #9C follow-ons: Pickett brush-rings + crossplot cutoff region (2026-07-23)

Two interaction upgrades that build on Round 43's linked brushing and the crossplot's draggable
parameter handle.

- **Pickett brush-rings** — the **Pickett** plot is now a brushing *consumer*: samples you Shift+drag on
  a **crossplot** of the same well are ringed (accent-2) on the Pickett log-log, so a selection made in
  one plot is visible in the other. Depths match bit-exactly off the shared backend grid; rings are
  clipped to the plot and skip log-invalid points.
- **Crossplot cutoff region** — a new **"Net cutoff"** dropdown next to the pick rows turns the draggable
  parameter handle into a pair of cutoffs. Pick a *net side* (X ≥/≤ pick, Y ≥/≤ pick) and the crossplot
  draws the two cutoff threshold lines through the handle, **shades the net quadrant**, and reads out how
  many plotted points fall inside it (`net cutoff: N / tot pts (P%)`). The sense is chosen explicitly —
  no cutoff direction is inferred from the axes — and the quadrant maps data→pixels through the axis
  extents, so it stays correct under log / inverted axes. Default **off** (unchanged appearance).
  Dragging the handle still writes the two zone parameters as before; the net side persists in plotprops.

**Verification:** the cutoff quadrant→pixel mapping and the 4-sense point-count were unit-tested against
the real `PlotCanvas.toPx` (counts + NaN exclusion exact for all four senses; correct side under linear
and inverted-Y axes). Adversarial review caught and fixed two bugs: a template-apply path that left the
Net-cutoff dropdown out of sync with `opts.netSense`, and an uncancelled hover `requestAnimationFrame` on
Pickett dispose. `tsc` clean.

**Try:** open a **Crossplot** and a **Pickett** of the same well side by side; **Shift+drag** a box on the
crossplot — the same samples ring on the Pickett. Then on the crossplot pick a **Net cutoff** side from the
new dropdown, drag the ringed handle around the cloud, and watch the shaded net box + the live
`net cutoff: N / tot pts (P%)` readout follow.

## Round 44 — UI polish #9D: accessibility & motion (2026-07-23)

The plot canvases were unlabelled and unfocusable, and transitions ignored the OS "reduce motion"
setting. Both fixed, via two shared helpers plus one CSS media query.

- **`makeCanvasAccessible(canvas, label)`** (plotCanvas.ts) — sets `role="img"`, an `aria-label`, and
  `tabindex=0`. The **crossplot / histogram / Pickett** canvases now announce themselves to screen
  readers with a live description (e.g. "Crossplot: RHOB versus NPHI, coloured by GR", "Histogram of
  PHIE", "Pickett plot: RES_DEEP versus PHIE") that updates as the plotted curves change.
- **`attachKeyboardPanZoom({canvas, getPlot, view, redraw, axes})`** (plotCanvas.ts) — a focused plot
  canvas now takes **arrow keys** to pan (Shift = bigger step), **+/−** to zoom around centre, and
  **0/Home** to reset, driving the same `ViewportRef` as the mouse (log-safe, `axes:"x"` on histograms).
  Wired into all three panels; only handled keys are consumed so Tab/Enter still work.
- **`.plot-canvas:focus-visible`** — an accent focus ring so keyboard focus is visible.
- **`@media (prefers-reduced-motion: reduce)`** — neutralises every transition/animation (the 5 CSS
  transitions the survey found: form inputs, `.btn`, mm-chevron, proc-bar, health-bar) for users who
  opt out of motion.

**Verification (in-browser):** `makeCanvasAccessible` → `role=img`, `aria-label="Test chart"`,
`tabindex=0`; `attachKeyboardPanZoom` → ArrowRight panned the viewport (xMin 0→0.8), `+` zoomed in
(width 10→8.3), `0` reset to auto, and the disposer stopped handling; both the reduced-motion media
rule and the `.plot-canvas:focus-visible` rule are live in the stylesheet. `tsc` clean.

**Try:** click a **Crossplot / Histogram / Pickett** plot to focus it (an accent ring appears), then
use **arrow keys** to pan, **+/−** to zoom, **0** to reset — no mouse needed. A screen reader now reads
the chart's axes. Turn on the OS "reduce motion" setting and UI transitions stop animating.

## Round 43 — UI polish #9C: linked brushing (crossplot → log view + histogram) (2026-07-23)

Rectangular **Shift+drag** on a **crossplot** selects a cloud of samples; every plot and log view of
the same well highlights those same samples. A new `appState.brushedDepths` observable
(`{wellId, depths:Set<number>}`) carries the selection; membership is an exact `Set.has` on the shared
well depth grid (all a well's curves come off the same backend f32 grid — verified in the adversarial
review against the Rust `fetch_curve_data`).

- **Crossplot (source + consumer):** Shift+drag draws a selection rectangle (accent2, dashed); on
  release the samples inside are published, and the brushed points are drawn emphasised. A tiny
  rectangle clears the selection. The gesture takes precedence over pan/param-handle/pick — it
  `stopImmediatePropagation()`s so `attachZoomPan` never pans, and marks `movedSinceDown` so the
  trailing click doesn't drop a parameter pick.
- **Histogram (consumer):** the brushed samples' values are over-painted as an accent2
  **sub-distribution** in the same bins — you see where the brushed cloud falls in any property.
- **Log view (consumer):** `HighlightsOverlay.setBrush` paints the brushed depths as thin accent
  **ticks** across every track, redrawn each frame; a well switch re-applies (gen-guarded) so the
  previous well's ticks never linger.

**Adversarial review (subagent):** cleared event-coexistence, the exact-float grid match (checked
against the backend), lifecycle/teardown, published-set correctness, and NaN/null safety. Two real
issues found and **fixed**: (1) the log-view brush re-apply in `loadWell` wasn't gen-guarded — a fast
well-switch could wipe the winning load's ticks; (2) `rafId` wasn't cancelled on dispose in the
crossplot/histogram. Both patched.

**Verification (in-browser):** state plumbing (`setBrushedDepths` → `W1:3` → `clearBrush` → null →
empty-set → null); `drawHistogram(brushValues)` over-painted the sub-distribution (12.1 k changed
pixels); `HighlightsOverlay.setBrush([4 depths])` painted 1400 tick pixels, `setBrush([])` → 0. `tsc`
clean.

**Deferred (9C follow-on):** Pickett rings on brushed samples (same pattern, cheap) and the draggable
cutoff *polygon* → zone params (the crossplot already has a draggable param **handle** that writes
cutoffs; a full lasso/polygon is a separate feature).

**Try:** open a **Crossplot** and a **Log view** of the same well side by side. **Shift+drag** a box
around a cluster on the crossplot — the log view lights up **ticks** at those depths, and if you have a
**Histogram** of PHIE/SWE open, the selected samples show as a highlighted **sub-distribution**. Drag a
tiny box (or Shift-click) to clear.

## Round 42 — UI polish #9B inc 1: shared colour-bar + scatter hover tooltip (2026-07-23)

Visualization richness, starting with two shared primitives in `plotCanvas.ts` so every chart gets the
same treatment instead of a bespoke copy:

- **`drawColorbar(plot, {map, lo, hi, label, log})`** — the continuous Z colour-bar, extracted from
  its one bespoke copy inside `drawCrossplot`. The crossplot now calls it; Pickett/HFU can adopt it in
  one line. Same look, one place to theme.
- **`attachScatterTooltip(canvas, hit)`** — a hover **tooltip bubble** showing the sample under the
  cursor. `hit(px, py)` returns the lines to show (or null to hide); the bubble is a
  `pointer-events:none` node positioned by the cursor and clamped to the viewport, so it never steals
  the canvas's own mouse events. `fmtValue(v)` gives compact 4-sig-fig labels.
- Wired into the **crossplot** (depth + X/Y/Z values, suppressed while dragging a handle) and **Pickett**
  (depth + Rt + porosity, suppressed while panning/picking). New `.plot-tooltip` CSS, all theme vars.

**Still open in 9B:** true **vector SVG/PDF export at print scale** for the Canvas-2D charts. Today only
the log *composite* has a vector path (`export_composite_svg/pdf` via `composite.rs`); the crossplot /
histogram / Pickett charts export raster PNG only. A real vector route needs an SVG-emitting renderer or
a new Rust command — a sizeable increment on its own, flagged for a scoping call rather than rushed.

**Verification (in-browser):** `fmtValue` → `["0.1823","2.5","1.235e+4","1.23e-4","—","0"]`; `drawCrossplot`
with a continuous Z rendered the scatter + colour-bar (87.8 k coloured pixels, non-null plot);
`attachScatterTooltip` showed the bubble (`display:block`, correct text, `pointer-events:none`), hid on
`mouseleave`, and removed its node on dispose. `tsc` clean.

**Try:** open a **Crossplot** (NPHI–RHOB coloured by GR) and hover the cloud — a bubble now shows that
sample's **depth, NPHI, RHOB and GR**. The Z **colour-bar** top-right is unchanged (now shared code).
Open a **Pickett** plot and hover — depth + Rt + porosity. Dragging the parameter handle (crossplot) or
panning suppresses the bubble so it doesn't fight the gesture.

## Round 41 — Results-QC #8 inc 4: recon / MC / cutoff rollup rows (2026-07-23)

The scorecard now reads as **one verdict per zone** — the two on-open checks (Sw-method spread, Buckles)
plus three rollup rows that **aggregate the sibling analyses** so you don't have to open three panels:

- **Recon incoherence** — mean/max of the SandiMin `*_RECON` curve (Quanti.Elan incoherence, σ units)
  over the zone, with the fraction of samples >2σ. Green ≤1σ, amber ≤2σ, red beyond — *do the solved
  volumes rebuild the logs?* Picks the most-recently-written `*_RECON` on the well; read-only.
- **MC uncertainty** — mean P50 and the mean **LOW–HIGH band** of the persisted `MC_<curve>_LOW/_P50/_HIGH`
  curves (PHIE, else SWE/VSH), as a fraction of |P50|. Green ≤15 %, amber ≤35 %, red beyond — *how wide
  is the input-uncertainty envelope?* Read-only.
- **Cutoff sensitivity** — a **live** `run_cutoff_sweep` nudging the PHIE≥ cutoff ±0.02 v/v around its
  operating value (VSH≤ / SWE≤ held), reporting the fractional net-pay move. Green ≤15 %, amber ≤40 %,
  red beyond — *is net pay robust to the cutoff, or does a small change move the number?*

Each row degrades to a **grey "na — run X first"** when its source curves are absent (SandiMin recon-QC
or Monte-Carlo persist not yet run) — never a silent pass. New operating-cutoff inputs (**VSH≤ / PHIE≥ /
SWE≤**, defaults 0.50 / 0.08 / 0.50) sit beside the Sw params; the user confirms them, nothing is
fabricated. CSV gains 12 columns (recon mean/max σ, %>2σ; MC P50/band/rel; cutoff net/sens/peak).

**Verification (in-browser, mocked IPC):** two zones — a shaly SAND-A and a clean SAND-B. Full scenario:
SAND-A flags Recon (2.20σ, 73 % >2σ), MC (band 56 %), Cutoff (±87 % net) all red; SAND-B all green
(0.50σ, 13 %, ±2 %); status line counts 5 flags. Bare scenario (no recon/MC curves): both rows show
"run … first" (na) while the live cutoff row still fires. CSV header + rows confirmed with the new
columns. Guard added: net pay ≤0 at the operating cutoff → na (no "±Infinity %"). `tsc` clean.

**Try:** open **Results QC** on a well where you've run **SandiMin (Reconstruction QC on)** and **Monte
Carlo (Persist curves on)**. Each zone card now shows five rows — the new **Recon incoherence**, **MC
uncertainty**, and **Cutoff sensitivity** lights. Hover any row for the full explanation. On a well where
you *haven't* run those, the recon/MC rows read "run … first" — run them, hit **Recompute**, and watch
the lights populate. Tweak **PHIE≥** and Recompute to see the cutoff-sensitivity light move. **⭳ CSV**
now carries the recon/MC/cutoff columns.

## Round 40 — Results-QC #8 inc 3: Sw-envelope track + Buckles crossplot + CSV (2026-07-23)

The visual payoff for the scorecard — a **detail view** under the cards, plus CSV export. All frontend,
reusing the per-zone data the scorecard already computed (cached, no refetch).

- **Sw-method envelope track** (`PlotCanvas`) — depth (Y, inverted) vs Sw (X): a shaded min/max **band**
  with one line per model (stable colour per model, Archie first), and a dashed **depth marker** that
  tracks `appState.hoverDepth`. This is where a wide fresh-water-sand spread is read at a glance.
- **Buckles crossplot** (`PlotCanvas`) — Sw (X) vs PHIE (Y): the zone's SWE·PHIE samples over dashed
  **constant-BVW hyperbolae** (0.02–0.10), so an irreducible leg lines up on one hyperbola and a
  transition/inconsistency fans across them.
- A **Detail zone** dropdown (and **clicking any scorecard card**) focuses both plots on that zone; a
  legend names the model colours and the band/hyperbola conventions.
- **⭳ CSV** exports the whole per-zone scorecard (zone, top/base, models, mean/max spread, worst-spread
  depth, fraction divergent, BVW mean/CV/n).

Canvas colours all come from `readTheme` (`--accent` band, per-model `faciesColor`, `--warn` marker,
`--grid` hyperbolae); the plots redraw on theme change and resize.

**Verification (in-browser, mocked IPC):** mounted against canned `list_zones` / `sw_method_spread` /
byte-packed `get_curve_data`. Both canvases rendered real content (non-uniform pixel counts ~738/737, not
blank frames); the legend listed Archie/Simandoux/Indonesia/Juhász; the Detail-zone dropdown held both
zones and switching to SAND-B redrew the Buckles plot; setting hoverDepth redrew the envelope's depth
marker; and **⭳ CSV produced the correct header + one row per zone** (mean_spread 0.17/0.01,
frac_divergent 0.7/0, bvw_n 25). tsc exit 0; cargo unchanged at 348. (Screenshot skipped — the preview
pane wasn't compositing; verified via pixel-content + DOM + captured CSV text instead. Console clean of
panel-origin errors.)

> **Try:** open **Results QC**, pick a zone in **Detail zone** (or click its card). The **Sw-method
> envelope** shows the model band — watch Archie ride above the shaly-sand lines in fresh-water sand;
> drag the log crosshair and the dashed depth marker follows. The **Buckles** plot shows your SWE·PHIE
> against constant-BVW curves — a clean pay leg hugs one curve. Hit **⭳ CSV** for the scorecard table.

## Round 39 — Results-QC #8 inc 2: panel + per-zone QC scorecard (2026-07-23)

New well-bound panel `src/ui/resultsQcPanel.ts` (`buildResultsQcContent`), registered like the other
singletons — `buildRenderer` case, `openResultsQc`, the ＋-menu entry, a **Results QC…** ribbon button
(next to Field Dashboard), and `#results-qc-btn` wiring. Follows the selected well (`wellPane`,
`followData`), so it rebuilds when the interpretation changes.

For every zone of the well (or "All depth" when none) it shows a **per-zone card** with a traffic-light
per check:

- **Sw-method spread** — calls the inc-1 `sw_method_spread` per zone and lights **ok / caution / alert**
  on the fraction of divergent depths (≤10 % / ≤40 % / more), with `mean · max @ depth · % divergent`
  and the model list + notes on hover.
- **Buckles (BVW)** — BVW = SWE·PHIE over the zone; lights on the coefficient of variation (≤15 % /
  ≤30 % / more) with `BVW mean · CV% · n`. Framed as a prompt (transition zone vs. inconsistent Sw), not
  a verdict — the crossplot that resolves which comes in inc 3.

A compact Sw-params row (Rw, Rw °F, Form °F, m, n, Rsh, a, divergence threshold — editable defaults the
user confirms, nothing fabricated) drives a **Recompute**. Traffic-light dots are theme-var coloured
(`--accent` ok / `--accent2` caution / `--warn` alert — never hard-coded red/green). The card under the
crosshair highlights via `appState.hoverDepth`.

**Verification (in-browser, mocked IPC):** mounted the panel against canned `list_zones` /
`sw_method_spread` / a byte-packed `get_curve_data`. Two zones rendered correctly — SAND-A: Sw-spread
**alert** (mean 0.180, max 0.190 @ 2010 m, 66 % divergent) + Buckles **ok** (BVW 0.060, CV 1 %); SAND-B:
Sw-spread **ok** (0 % divergent) + Buckles **alert** (CV 31 %); status "2 zone(s) · 2 flagged".
hoverDepth 2020→SAND-A, 2070→SAND-B, null→neither (highlight follows the crosshair). Screenshot confirms
the cards; console shows only the pre-existing backend-absent boot errors — none from the panel. tsc exit
0; cargo unchanged at 348.

> **Try:** ribbon **Batch → Results QC…** (or ＋ → Results QC). With a well selected, each zone gets a
> card: the **Sw-method spread** light goes amber/red where Archie and the shaly-sand models disagree
> (fresh-water sand), and **Buckles (BVW)** flags zones whose bulk-volume-water wanders. Tune Rw/m/n/Rsh
> and hit **Recompute**; move the log crosshair and the matching zone card highlights.

## Round 38 — Results-QC #8 inc 1: Sw-method spread backend (2026-07-23)

First increment of the Results-QC / Sw-comparison dashboard. New Rust module `src-tauri/src/resultsqc.rs`
+ command `sw_method_spread` (ipc `swMethodSpread`) — the one metric the dashboard genuinely needs from
the backend, because the five Sw models are pure `fn`s in `multimin2` that the frontend can't call.

Per depth it evaluates every Sw model whose input curves are present and returns the **envelope**
(sw_min / sw_max / spread), a per-series value set, and a **divergence summary** (mean/max spread, the
depth of worst disagreement, the fraction of comparable depths above a 0.10-Sw threshold, and a notes
trail). **Archie / Simandoux / Indonesia / Juhász** run from the always-available logs; **Waxman-Smits**
joins only with a Qv curve and **Dual-Water** only with a bound-water-saturation curve — no CEC/Qv/Swb is
ever fabricated to force a model in. Fluid conductivities reuse the app's own `fluid_calc`/`waxman_b`
path (no divergence, no invented constants); the classic fresh-water-sand story falls straight out —
Archie over-reads Sw while the clay-aware models cluster below it.

**Adversarial review (1 skeptic, math-heavy) — 3 medium + 3 low, all fixed:** (M) a null Qv silently
collapsed Waxman-Smits to Archie via `(B·Qv).max(0)` → now returns NaN at any non-finite/negative Qv;
(M) `BQV` (= B·Qv) was auto-aliased into the Qv slot and re-multiplied B → dropped from auto-candidates,
needs an explicit override; (M) model activation counted *columns* not *finite data*, so an all-null
column inflated the "active" count and muted the warning → a model is kept only with ≥1 finite Sw, the
insufficient-data note keys on comparable-depth count, dropped columns are reported by name; (L) a note
now fires when PHIE is absent; (L) ambiguous `PHI` moved off the PHIE candidate list onto PHIT; (L)
added numeric Juhász, WS/DW-reduce-to-Archie-at-zero, null-Qv→NaN, and all-null-column tests. The review
also cleared the units (Rw=1/Cw at formation T, Cwb=virgin, B(T,Rw)), envelope, and index-alignment.

**Verification:** cargo 348 passed / 0 failed / 7 ignored (+8 new resultsqc tests); tsc exit 0. Read-only
— computes nothing to disk.

> **Try:** no UI yet — the per-zone scorecard + Sw-envelope track that consume this land in the next
> increment (#8 inc 2/3). The command itself is exercised by those; nothing to click through here.

## Round 37 — Contacts #6.2 inc B: assisted contact picking — the panel UI (2026-07-23)

Second increment — wires inc A into the correlation panel's **Contacts…** editor with two new sections:

- **Suggest from logs** — pick a well and a depth zone (defaults to the visible window), hit **Suggest**,
  and get the ranked candidates (Sw crossover / resistivity drop / density-neutron gas base), each showing
  `type @ depth — method (confidence%)`; low-confidence (<40%) rows are dimmed. **Accept** on a candidate
  creates a well-scoped MD contact at that depth (and appears in the editor's table) — **never
  auto-committed**, one click per pick.
- **Cross-well consistency** — pick a contact type, hit **Check**, and get a readout: `N wells · dip
  plane|flat mean · mean TVDSS · rms`, then a per-well table (TVDSS, predicted, residual) with **⚠-flagged
  wells** that disagree with the flat-TVDSS surface.

**Verification (in-browser, mocked IPC):** mounted the correlation panel, opened the Contacts editor, and
drove both sections from the DOM: Suggest rendered 3 ranked candidates with the 35% one flagged weak;
**Accept called upsert_fluid_contact with `{well W1, OWC, 2148.5, MD}`** and added the row + set the
status (no auto-commit); Check rendered the summary ("3 wells · dip plane · mean 2076.1 · rms 1.4 m") and
flagged the 12 m-off Well-3 while clearing the inliers. tsc green; cargo unchanged at 340. (Console shows
only pre-existing backend-absent boot errors — none from the panel.)

Deferred (noted, not silently dropped): **snap-to-log-feature while dragging a contact line** — contacts
aren't draggable in the panel yet (drag is pan), so a hit-test + drag handler is a larger change left for
a follow-up; the Suggest/Accept flow covers the assisted-picking need in the meantime.

> **Try:** open a **Correlation** panel → **Contacts…**. Under **Suggest from logs**, choose a well with
> Sw/resistivity/density-neutron over a hydrocarbon-water zone and hit **Suggest**; **Accept** the pick you
> trust — it drops in as an OWC in that well. Then set contacts of that type in a few wells, and under
> **Cross-well consistency** hit **Check** — any well off the flat-TVDSS surface shows a ⚠.

## Round 36 — Contacts #6.2 inc A: assisted contact picking — backend (2026-07-23)

First increment of assisted fluid-contact picking (the existing contacts editor + TVDSS-flat
rendering was already built and committed in the Wave-B chain — that is #6 inc 1). New `contacts.rs`
with two read-only commands:

- **`suggest_contacts`** — from one well's logs within a depth zone, proposes contact depths from
  three independent indicators, each with a confidence, ranked: the **Sw = cutoff crossover** (default
  0.5; confidence = the below-minus-above contrast), the **deep-resistivity drop** (steepest downward
  step in log10 Rt; confidence ∝ decades fallen), and the **density-neutron gas base** (where φN−φD
  closes back through −0.03 — gas-down-to). Uses whichever curves are present. Nothing is written — the
  user accepts/edits (inc B).
- **`check_contact_consistency`** — a contact is flat in TVDSS, so it fits a **least-squares dip plane**
  (z = a + b·x + c·y, on centred UTM coords) through every well's pick of a type, converts MD picks to
  TVDSS via each deviation survey, and **flags wells whose residual exceeds a threshold** (default 3 m).
  Falls back to a flat mean when < 3 wells have coordinates.

**Adversarial review** (math-heavy) confirmed the crossing interpolation, the resistivity-drop loop, the
plane solve, and the MD→TVDSS interpolation correct with no panics/divide-by-zeros, and surfaced four
issues I fixed: (1, **medium**) the consistency check was **mixing baselines** — coord wells scored vs
the plane, coordless wells vs the flat mean → false flags and a blended RMS; now it uses **one baseline**
(coordless wells are left *unscored* under a plane, RMS over scored points only); (2) resistivity depth
was ~win/2 shallow → refined to the sharpest single-sample step; (3) noisy Sw flooded candidates → cluster
dedup; (4) the neutron PU/fraction unit is now decided once per curve, not per sample.

**Verification:** cargo **340 passed / 0 failed** — Sw crossover recovers a known 2050 m contact; the
~1.4-decade resistivity drop scores high and lands on the step; the D-N gas base hits 2040 m; `fit_plane`
recovers a known dipping plane; the consistency check flags a 12 m outlier while clearing inliers; and a
coordless well is left unscored (not false-flagged) under a plane. tsc green. Backend-only — the panel
wiring is inc B.

> **Try:** backend-only this round — no new button yet. The **Suggest from logs** action and the
> cross-well consistency readout land in the next increment inside the **Contacts…** editor.

## Round 35 — Autocorrelate #5 inc 3: the dialog — warp toggle, multi-select, per-marker review (2026-07-23)

Third increment — the UI that makes inc 1/2 usable. The **Autocorrelate** pane is rewritten:

- **Tops are now a checkbox list** (with an **All** toggle): tick **one** top to correlate a single marker,
  or **several** to propagate a consistent set together. The run button tracks it — "Correlate 2 wells"
  vs "Correlate 3 tops → 2 wells".
- **Method** dropdown — **Rigid shift (fast)** or **Elastic warp** — wired to inc 1/2's `method`. The
  **Max stretch ×** control appears only for warp; the **Window ±** control appears only for a single top
  (multi derives its window from marker spacing).
- **Per-marker review** — single mode shows a well×proposal table; multi mode shows a (well, marker) table
  grouped by well, each row with its **own r**. Strong matches (r ≥ 0.7) pre-ticked; **low-confidence rows
  flagged** (dimmed) and left unticked; a well with no data shows an error row.
- **Accept/reject per row**, then **Apply** writes only the ticked picks as **one undoable batch** (undo
  restores/deletes each pick).

**Verification (in-browser, mocked IPC):** the pane mounted and every interaction was driven and read back
from the DOM — control show/hide is exactly right (max-stretch only on warp, window only single, label
tracks selection); the multi table renders 3 markers under a well with the low-r marker flagged/unticked
and the errored well shown; **Apply invoked upsert_top for exactly the two ticked markers** (not the weak
one, not the errored well) and set the batch status; the single path passes `method:"shift"` and flags its
r 0.61 row. tsc green; cargo unchanged at 334. (Console shows only the pre-existing backend-absent boot
errors — none from the dialog.)

> **Try:** open the **Autocorrelate** pane (＋ menu or the ribbon). Tick **one** top, set **Method =
> Elastic warp**, give a **Max stretch** (say 1.5), and **Correlate** — review the r per well, untick weak
> matches, **Apply**, then **Ctrl-Z** to confirm the batch undoes. Then tick **several** tops and
> **Correlate** again: you get a per-marker table, and the applied set stays in stratigraphic order (no
> crossings) in the correlation view.

## Round 34 — Autocorrelate #5 inc 2: multi-marker simultaneous propagation — backend (2026-07-23)

Second increment. Adds `autocorrelate_multi` (new `autocorrelate_multi` command): propagate **several
markers together** into each target well as one **consistent** set, each with its **own** confidence.

- **Consistency by construction** — markers are propagated top-down, each warped in its own local window
  (the inc 1 warp), and a **hard monotone guard** forbids a later marker from crossing above an earlier
  one. The guess for each marker is *guided* from the previous proposal (carry the source spacing
  forward), so the search per marker is small and can't lock onto a neighbour's feature.
- **Per-interval confidence** — each propagated marker carries its own Pearson r (the per-marker score
  the spec asks for), not one r for the whole well.
- Empty selection ⇒ all source tops; markers can be named explicitly. Read-only — the dialog (inc 3)
  reviews and applies.

Refactor: the single-marker `autocorrelate_top` and the new multi path now share `build_template` +
`propagate` (rigid best-lag → optional warp-refine with the better-of guard). No behavior change to the
single path — all its tests still pass. inc 2 adds no new math; it reuses inc 1's adversarially-reviewed
primitives, so the review this round was a focused self-check of the guided-guess / monotone-guard
orchestration (no k=0 index underflow; a skipped marker never corrupts the set).

**Verification:** cargo **334 passed / 0 failed** — new test propagates 3 markers through a ×1.25 stretch
(each moves a different amount), recovering all three to <3 m, in strict order, each scored. tsc green.
Still backend-only — nothing browser-observable yet.

> **Try:** backend-only again — no new button this round. The multi-marker propagation becomes usable in
> the next increment's dialog (select several tops, one **Correlate**, review a per-marker table, apply as
> one undoable batch).

## Round 33 — Autocorrelate #5 inc 1: elastic depth warp (subsequence DTW) — backend (2026-07-23)

First increment of the marker-autocorrelation enrichment. `tops.rs` today propagates a top from the
source well to others by a **rigid best-lag** GR match (slide the pick window, keep the max-Pearson
depth). That is unchanged and stays the fast default. This increment adds an **elastic depth-warp**
mode alongside it:

- **`subseq_dtw`** — open-begin/open-end subsequence dynamic-time-warping. The `(1,1)/(1,0)/(0,1)` step
  set makes the alignment **monotone and non-inverting by construction** (no depth crossovers), and a
  per-step stretch penalty keeps it near slope 1 unless the log clearly warps.
- **`warp_refine`** — refines the rigid pick: builds a target window sized for the requested stretch,
  **P3/P97-normalizes** both logs (the `gr_normalize` two-point idea, applied window-locally) so the warp
  compares *shape*, not tool calibration/datum, then reads off the depth the marker (window centre) warps
  to. Reported r is the template-vs-warped-target Pearson — the **same metric as the rigid r**.
- Request gains `method` (`shift`|`warp`) and `max_stretch`, both serde-defaulted, so the existing
  dialog call is byte-identical (`shift`). **No UI yet** — the warp/shift toggle, max-stretch control and
  per-interval tie-lines come in inc 2/3.

**Adversarial review** (math-heavy) confirmed the DTW recurrence, back-pointers, monotonicity, and
marker mapping correct with no OOB/underflow/panic paths, and surfaced three behavioral gaps I then
fixed: (1) warp could **silently regress** a better rigid pick → added a better-of guard (keep warp only
if its r ≥ rigid r − ε); (2) a marker could be placed **in a data gap** with a plausible r → reject a
warp whose marker lands on a NaN sample (fall back to rigid) and raised the NaN step-cost so DTW avoids
nulls; (3) the `max_stretch` doc **overstated** a hard local cap → reworded to the honest soft/window
control it is. cargo: **333 passed / 0 failed** (rigid recovers a known 7.5 m lag; warp recovers a known
×1.5 piecewise-stretched section to ~1 m where rigid is ~10 m biased; warp does not regress a pure shift;
DTW path proven monotone/complete on noisy input). tsc green.

> **Try:** backend-only this round — open the **Autocorrelate** pane and correlate a top as before; it
> should behave **exactly** as it did (rigid shift, unchanged). The warp mode has no button yet; it lands
> in the next increment where you'll get a **Rigid / Elastic-warp** toggle and a max-stretch control.

## Round 32 — Unconventional #7 inc 5: ΔlogR overlay + Langmuir isotherm panel (2026-07-23)

Fifth and final increment — the visual companion to the four compute modules. A new workspace pane,
**Unconventional (ΔlogR + Langmuir)**, opens from **Petrophysics → Unconventional → ΔlogR + Langmuir
Visuals…** (also in every window's ＋ menu). It follows the active well like the other tool panes and
carries two pictures side by side:

- **Passey ΔlogR overlay** (depth track) — deep resistivity on a log/decade axis and a baselined
  porosity curve (sonic **DT** or density **RHOB**) drawn so the two **overlie in non-source rock and
  fan the opposite way over organic-rich intervals**; the shaded lens between them **is ΔlogR**, the
  input to `toc_passey`. Uses that module's exact scaling — resistivity at log10(R/R_base), porosity at
  −0.02·(DT−DT_base) [sonic] / +2.5·(RHOB−RHOB_base) [density] — so the picture and the number agree.
  R_base and the mode's baseline are editable; picks are read on a clay-rich, non-source shale.
- **Langmuir isotherm** — Gs = VL·P/(PL+P) (scf/ton) with the **VL** ceiling, the **PL** half-saturation
  point (Gs=VL/2 at P=PL), the **reservoir-pressure** operating point, and — given an in-situ gas
  content **GC** < VL — the **critical desorption pressure Pcd = PL·GC/(VL−GC)** for undersaturated
  coal/shale. This is the adsorbed term of the `gip` module, drawn.

Display-only (no new physics, no backend). Verified in-browser against synthetic source-rock curves
(a resistivity + Δt/ρb kick): the sonic and density overlays both render the correct opposing fan, and
the isotherm's PL/Pres/Pcd markers land where the formulae put them. **Two defects were caught in that
pass and fixed:** (1) the porosity curve was drawn at `xR − poroTerm` instead of the absolute `−poroTerm`,
which understated ΔlogR and leaned both curves the same way; (2) the baseline field toggled both DT_base
and RHOB_base together, hiding RHOB_base in density mode. tsc green; the four compute modules are
unchanged (330 cargo tests still pass).

> **Try:** open **Petrophysics → Unconventional → ΔlogR + Langmuir Visuals…**, select a well with a deep
> resistivity + sonic (or density) curve. On the left, set **R_base / DT_base** on a clay-rich,
> non-source shale and confirm the two curves overlie there and split (shaded) over your organic zones —
> that lens should track where `toc_passey` gives high TOC. Switch **Overlay = Density** to pair RHOB
> instead. On the right, type your **VL / PL / reservoir pressure**; for undersaturated coal add a **GC**
> and read **Pcd** off the isotherm.

## Round 31 — Unconventional #7 inc 4: brittleness index (elastic + mineralogical) (2026-07-23)

Fourth increment. A new module, **Brittleness index (elastic / mineralogical)** (Petrophysics →
Unconventional), scores rock brittleness (0 ductile .. 1 brittle) two ways:

- **METHOD = elastic** — dynamic Young's modulus and Poisson's ratio from **DT, DTS, RHOB** (moduli in
  GPa via ρ·V², Vp/Vs = 304.8/slowness, E→Mpsi), then Rickman et al. 2008 BI = (E_norm + ν_norm)/2. The
  normalization endpoints (E 1..8 Mpsi, ν 0.4..0.15 — Barnett defaults) are editable **params** so you
  can recalibrate to Mahakam. Also outputs the dynamic **YME** / **PR**.
- **METHOD = mineral_jarvie** — Jarvie 2007 BI = Qz/(Qz+carbonate+clay). **mineral_wanggale** — Wang &
  Gale 2009 BI = (Qz+Dol)/(Qz+Dol+calcite+clay+organic), moving dolomite to the brittle side. Feed the
  SandiMin **VOL_*** volumes (a missing mineral counts as absent); the organic term is the inc-2 **VKER**.

Tier-B, cited (Rickman et al. 2008; Jarvie et al. 2007; Wang & Gale 2009); the elastic moduli
reimplement the Techlog RockPhyEquations forms. Math in `docs/ref_unconventional.md` §4. Elastic E,ν are
dynamic (apply a static correlation before geomechanics, not before the Rickman index).

Verified: **330 cargo tests** (7 new — elastic recovers a known E/ν/BI from slowness, Jarvie/Wang-Gale
groupings, BI monotone in quartz, invalid-shear + negative-Poisson rejection, all-absent→NaN) + tsc,
adversarial review = FIX-FIRST → fixed: the elastic branch now rejects ν<0 (Vp/Vs<√2, a bad shear log)
instead of emitting a negative PR and a falsely max-brittle BI.

> **Try:** for the elastic index, open **Petrophysics → Unconventional → Brittleness index**, keep
> **METHOD = elastic**, set **DT / DTS / RHOB** (needs a shear sonic), and Run — check **BI** rises in
> the stiff, quartz-rich (high-E, low-ν) beds. For the mineral index, run **SandiMin** first, switch
> **METHOD = mineral_jarvie**, and map **VQTZ / VCARB / VDOL / VCLAY** to your VOL_* curves; compare
> against the elastic BI where you have both.

## Round 30 — Unconventional #7 inc 3: gas-in-place (free + Langmuir adsorbed) (2026-07-23)

Third increment. A new module, **Gas-in-place (free + Langmuir adsorbed)** (Petrophysics →
Unconventional), gives per-depth gas CONTENT (scf per ton of rock) so it composites like any curve:

- **GIP_ADS** = VL·P/(PL+P) — Langmuir adsorbed gas. **GIP_FREE** = 32.0368·φ·(1−Sw)/(RHOB·Bg) —
  compressed free gas, with **BG** = 0.02827·z·T/P (T in Rankine). **GIP_TOTAL** = free + adsorbed.
- **MODE = cbm** applies the dry-ash-free correction GIP_ADS·(1−F_ASH−F_MOIST) and, given a measured
  in-situ gas content **GC**, emits the **critical desorption pressure PCD** = PL·GC/(VL−GC) — the
  pressure the coal must be dewatered below before gas desorbs.

The Langmuir VL/PL default to shale placeholders (100 scf/ton, 1000 psia ≈ IP's 7000 kPaa) — override
with core desorption/isotherm data. Feed the **PHI** slot your effective porosity or the inc-2
**PHIT_OMC**. Tier-B, cited (Langmuir 1918; GRI / Mavor-Nelson 1996). The Ambrose pore-volume
correction (which trims free gas by the adsorbed-phase volume, ~10% in high-TOC/high-P shale) is
deferred with its derivation banked — so **GIP_TOTAL is an upper bound** until it lands. Math in
`docs/ref_unconventional.md` §3.

Verified: **323 cargo tests** (7 new — Langmuir at P=PL/0/∞, free gas pinned to an independent hand
literal 167 scf/ton, Bg pinned to 0.0055947, total = free+adsorbed, CBM ash/moisture, Pcd, Sw=1→0,
out-of-range rejection) + tsc, adversarial review = SHIP (constants 32.0368 / 0.02827 recomputed by
hand; all divisions Inf-guarded).

> **Try:** open **Petrophysics → Unconventional → Gas-in-place (free + Langmuir adsorbed)**, set
> **PHI** (PHIE or the inc-2 **PHIT_OMC**), **SW**, **RHOB**, and reservoir **RES_P / TEMP_F / Z_FAC**;
> enter your core **VL / PL**. Run and confirm **GIP_ADS** dominates in the organic-rich (low-φ)
> section while **GIP_FREE** dominates where porosity is higher, with **GIP_TOTAL** their sum. For coal
> switch **MODE = cbm**, set **F_ASH / F_MOIST**, and enter a canister **GC** to see **PCD**.

## Round 29 — Unconventional #7 inc 2: kerogen volume + OM-corrected porosity (2026-07-23)

Second increment. A new module, **Kerogen volume + OM-corrected porosity** (Petrophysics →
Unconventional), turns the TOC curve into a kerogen VOLUME and corrects total porosity for the organic
matter that low-density kerogen inflates on the density log:

- **TOM** = k_toc2om · TOC/100 — organic-matter weight fraction (k_toc2om ≈ 1.2 accounts for the
  H/O/N/S beyond carbon).
- **VKER** = TOM · RHOB / ρ_kero — kerogen volume fraction of the *bulk* rock (Passey/Vernik
  bulk-density conversion). ρ_kero defaults to **1.10 g/cc** to match the SandiMin **Kerogen** mineral,
  so VKER reconciles with a SandiMin **VOL_KEROGEN**.
- **PHIT_OMC** = PHIT − VKER — strips kerogen's apparent-porosity contribution (feed a density-derived
  PHIT).

Chains off inc 1 (reads the **TOC** curve by default) and feeds inc 3 (GIP needs kerogen volume).
Tier-B, cited (Passey et al. 2010; Vernik & Nur 1992). Method math in `docs/ref_unconventional.md` §2.

Verified: **316 cargo tests** (5 new — bulk mass balance recovers a known VKER, OM-correction subtracts
and floors, zero-TOC is inert, VKER rises with TOC, missing RHOB falls back to TOM only) + tsc,
adversarial review = SHIP (fixed one wrong default pre-commit: ρ_kero was 1.20 but SandiMin's Kerogen
is 1.10 — now reconciled).

> **Try:** run **TOC — Passey ΔlogR** first (Round 28) so a **TOC** curve exists, then open
> **Unconventional → Kerogen volume + OM-corrected porosity**, set **RHOB** and (optionally) a
> density-derived **PHIT**, and Run. Check **VKER** is a few percent where TOC is a few wt% (light
> kerogen occupies ~2× its weight fraction), and that **PHIT_OMC** reads a touch below your input PHIT
> in the organic-rich section. Compare **VKER** against a SandiMin **VOL_KEROGEN** run (organic preset)
> — they should track at the default ρ_kero 1.10.

## Round 28 — Unconventional #7 inc 1: TOC from Passey ΔlogR + Schmoker (2026-07-23)

First increment of the unconventional / shale suite (playbook Part II #7). A new **Unconventional**
group on the Petrophysics ribbon, with its first module: **TOC — Passey ΔlogR + Schmoker**. It
estimates total organic carbon two independent ways:

- **Passey (1990) ΔlogR** — the separation between deep resistivity and a *baselined* porosity curve.
  Choose the **overlay**: *sonic* (`ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base)`) or *density*
  (`−2.5·(RHOB−RHOB_base)`). Set the baselines (`R_BASE`, `DT_BASE`/`RHOB_BASE`) on a clean, clay-rich,
  **non-source** interval where the two curves overlie (ΔlogR≈0), then
  `TOC = ΔlogR·10^(2.297−0.1688·LOM) + background`. LOM (maturity, 6..12) defaults to 10.6.
- **Schmoker-Hester (1983)** density-TOC `154.497/RHOB − 57.261` as an independent cross-check
  (writes `TOC_SCHMOKER` whenever a density curve is present, regardless of overlay).

Outputs: `DLOGR` (the raw separation, for the overlay panel coming in inc 5), `TOC` (Passey, wt%),
`TOC_SCHMOKER` (density cross-check). In non-source rock (ΔlogR<0) TOC floors to the *background*
value, not below it. Tier-B, cited in code (Passey et al. 1990; Schmoker & Hester 1983); the LOM and
baseline defaults are Tier-A IP seeds, per-well overridable. The **neutron** overlay is deferred — its
sign convention is inconsistent across the literature and needs core verification. Method math banked
in `docs/ref_unconventional.md` §1.

Verified: **311 cargo tests** (7 new — sonic/density overlays recover a known TOC, TOC decreases with
LOM, non-source floors to background, missing overlay curve falls back to Schmoker) + tsc green +
adversarial review (found & fixed one clamp-order defect pre-commit: a nonzero background must be the
floor, not zero). Additive — nothing existing moves.

> **Try:** open **Petrophysics → Unconventional → TOC — Passey ΔlogR + Schmoker**. Set **overlay =
> sonic**, **RES** = deep resistivity, **DT** = sonic. On a clean *non-source* bed read R and Δt and
> enter them as **R_BASE** / **DT_BASE** (so ΔlogR≈0 there); set **LOM** from your Ro/Tmax (or leave
> 10.6) and Run. Confirm **TOC** rises through the organic-rich section, and compare against
> **TOC_SCHMOKER** where RHOB exists. If you have core TOC, nudge LOM until the Passey curve matches.

## Round 27 — SandiMin per-depth formation temperature (FTEMP curve) (2026-07-23)

Formation temperature can now come from a **per-depth curve** instead of one fixed number. On the
**Fluids** tab there's a new **FTEMP curve (opt)** box next to *Formation temp (°F)*. Leave it blank to
use the fixed value (unchanged). Type a curve name (e.g. **FTEMP_F**, the curve Prep builds from a
gradient/BHT) and, for every depth where that curve is finite, SandiMin recomputes the temperature-
dependent quantities at that sample's temperature:

- **Cw / Cmf / Cbw** (formation-water, filtrate and clay-bound-water conductivities),
- the **auto CT/CXO uncertainties**,
- the **clay bound-water tie** (BNDWAT multiplier k, via t_c),
- the **Waxman-Smits B(T,Rw)**.

The α (diffuse-layer) expansion and salinities come from the *Rw/Rmf* sample temperatures, so they don't
move with formation temperature — only the conductivities do. A sample where the curve is missing or
out of range (a null like ±999.25, or anything outside 32–600 °F) quietly falls back to the fixed °F, so
selecting the curve is safe even on wells that lack it. With the box blank the solve is **byte-for-byte
identical** to before (a test pins that a constant FTEMP curve equal to the fixed value reproduces the
fixed-temperature run exactly), and the per-tool reconstruction-QC curves stay consistent under a curve.

> **Try:** run **Prep** so a **FTEMP_F** curve exists (or import one), then open **SandiMin → Fluids**,
> put **FTEMP_F** in **FTEMP curve (opt)**, and Run. Compare **SWE** with and without the curve over a
> long interval with a real geothermal gradient — the hotter, deeper section reads a bit lower Sw (hotter
> water is more conductive). Blank the box to confirm you get the fixed-temperature numbers back.

## Round 26 — SandiMin Waxman-Smits saturation model (2026-07-23)

The last of the Sw models. **Waxman-Smits (B·Qv)** joins the **Sw model** dropdown (Fluids tab). Like the
other post-solve forms it runs the mineral inversion untouched, then replaces the water/HC split from the
deep resistivity — here via `Ct = φt^m·(Cw·Swt^n + B·Qv·Swt^(n−1))`:

- **Qv** is built from the **solved clay volumes**: `Qv = Σ v_clay·CEC·ρ_clay / φt` (meq/mL). So each clay's
  **CEC** (Clay tab) drives the excess conductivity — a clean sand (no clay ⇒ Qv=0) collapses to Archie.
- **B** is the counterion conductance from the **Juhász (1981) B(T,Rw) fit** — the same closed form Techlog
  and IP use — computed from formation temperature and Rw automatically. Because that fit is known to
  overshoot above ~120 °C, a **B override (0 = auto)** box (shown only for this model) lets you pin a
  core-measured B.
- Uses your **m/n as m\*/n\***. PHIE/PHIT stay exactly as the mineral solve made them; only SWE/SWT/SXOT move.

Verified: the conductivity root and the B(T,Rw) fit are hand-anchored in unit tests (n=2 closed form, n=3
bisection, Qv=0/B=0 → Archie, B(25 °C,0.1)=3.895, B(100 °C,0.05)=15.51, monotonic in T and Rw), plus a
full-run integration test that recovers a known Sw. Nothing else moves — the default model is still linear
dual-water.

> **Try:** open **Petrophysics → SandiMin**, **Fluids** tab, set **Sw model → Waxman-Smits (B·Qv)**. Make sure
> a **CT** (deep-resistivity) tool and a **U-zone hydrocarbon** component are set, and that your clays carry a
> **CEC** (Clay tab). Run and compare **SWE** vs **Archie** (Waxman-Smits reads lower on shaly intervals) and
> vs **Juhász**. Leave **B override** at 0 for the auto B(T,Rw); enter a core B to pin it and re-run.

## Round 25 — SandiMin Constraints tab: porosity source + program-constraint toggles (2026-07-23)

The UI for item B (your image 2). A new **Constraints** tab (after Clay) holds two things:

- **Porosity source** radio — **Cation Exchange Capacity** (default) vs **Wet Clay Porosity**. This picks
  what drives the clay bound-water tie: CEC uses `α·96·CEC·ρ/(T+298)`; WCP uses the geometric `k = φ/(1−φ)`
  from a **per-clay φ editor** now on the **Clay** tab (pre-filled with Techlog WCLP defaults — Illite 0.104,
  Kaolinite 0.058, etc.). Running the dry-clay converter also fills a clay's φ, so the two stay consistent.
- **Program constraints** — enable toggles for **UNITY**, **POROSITY**, **X&U BNDWAT**, **WATER MUD**, plus a
  **Constraint tolerance σ** (default 0.01). All four already ran in the solver; this exposes them. UNITY moved
  here from the run footer (there's no longer a "Hard unity" box down by Run).

Defaults are unchanged behavior: CEC, all four on, σ=0.01 — so an untouched Constraints tab solves exactly as
before (a backend test pins that "absent request fields = on"). WATER MUD defaults on for water-based mud (it
keeps flushed-zone water ≥ virgin water; ignored for OBM) — tell me if you'd rather it default off.

> **Try:** open **Petrophysics → SandiMin**, click the **Constraints** tab. Flip **Porosity source** to
> **Wet Clay Porosity**, check the **Clay** tab's per-clay φ list, then Run and compare **PHIE/SWE** vs CEC
> (WCP moves PHIE for clays). Toggle a constraint off (e.g. **WATER MUD**) or change **σ** and re-run to see
> the effect. Confirm the run footer has **no** "Hard unity" box (it's now the UNITY toggle on this tab).

## Round 24 — SandiMin Wet-Clay-Porosity bound-water source (backend) (2026-07-23)

Starting item B (constraints editor + porosity source). This first slice is the **backend route** for
the **Porosity Source** choice from your image 2: the clay bound-water constraint can now be driven by
either **CEC** (default — `v_bw = α·96·CEC·ρ/(T+298)·v_dryclay`, nothing moves) or **Wet Clay Porosity**
(`v_bw = φ_clay/(1−φ_clay)·v_dryclay`, geometric). It's the same physics the Clay-tab wet→dry converter
already used (`dry_clay_calc`); this exposes it as a selectable source. Clays now carry Techlog's WCLP
defaults (Illite 0.104, Kaolinite 0.058, Chlorite 0.101, Glauconite 0.156, Montmorillonite 1.0, Clay 0.12).

Default stays **CEC**, so every reviewed number is untouched (verified: the CEC path is byte-identical to
before). The **UI radio + per-clay φ editor + the constraints panel (UNITY/POROSITY/X&U BNDWAT/WATER MUD)
land in the next slice** — nothing to click yet. Tests: the WCP multiplier equals the CEC route's
`cec_equiv` (the dry_clay_calc bridge) and drives the same bounded solve; Techlog WCLP defaults asserted;
adversarially reviewed. Note: the WCP source **moves PHIE** for clays (bound water is now geometric, not
CEC-derived) — that's the design you approved.

**Smectite fix (adversarial review caught this before commit).** Techlog carries `WCLP_Smectite = 1.0`,
but it only ever consumes that value *post-solve* for wet-clay-volume reporting (flooring `1−φ` at `1e-4`),
never as an inversion constraint. My first cut fed it straight into the BNDWAT *solver* row as `φ/(1−φ)`
with a `0.95` cap → `k ≈ 19`, ~100× every real clay and ~30× smectite's own CEC route — it would have
swamped the bound-water constraint and forced absurd bound water wherever montmorillonite appears. Fixed:
a degenerate `φ ≥ 0.5` (Techlog's real clays are all ≤ 0.156, so this cleanly isolates the `1.0`
placeholder) now **falls back to the CEC-calibrated multiplier** for that clay, so the two porosity
sources *agree* for smectite (`k ≈ 0.6`) instead of diverging 30×. New test
`wcp_degenerate_smectite_falls_back_to_cec` pins it; `library_has_expected_shape` asserts every
non-smectite clay's WCLP stays a physical geometric porosity. Real clays (Illite φ=0.104, etc.) are
unaffected — they still use the geometric `φ/(1−φ)` route.

## Round 23 — SandiMin Juhász / normalized-Qv Sw (the wet-shale model) (2026-07-23)

The **Juhász (normalized Waxman-Smits)** model — the wet-parameter one you grouped with Indonesia/
Simandoux — is now in the Sw dropdown as **"Juhász / normalized Qv."** Instead of dual water's
temperature-form clay conductivity, it reads the excess conductivity straight from the **shale point**:

    Cwsh = 1/(Rsh·φ_sh^m),   QVN = Vsh·φ_sh/φt,   Cw·Swt^n + QVN·(Cwsh−Cw)·Swt^(n−1) = Ct/φt^m   (a=1)

so it uses your wet-shale parameters directly (Rsh from a shale pick + **φ_sh = wet-clay porosity**, a new
input that appears only for this model). Runs **post-solve** like the others — the mineral solve is
untouched, **PHIE/PHIT/unity preserved**, only SWE/SWT/SXOT move. With Vsh=0 it collapses to clean-sand
Archie (tested). Equation matches the Geolog `sw_juha` / cookbook normalized-Qv form.

Internally, dual-water and Juhász now share one root solver (`sw_cond_root`) — the only difference is the
excess-conductivity coefficient (dual water `Swb·(Cwb−Cw)`; Juhász `QVN·(Cwsh−Cw)`). The dual-water
numbers are unchanged (same 30 tests green). Hand-computed literals at n=2 (closed form) and n=3
(bisection), Vsh=0→Archie, and NaN guards all pass; adversarially reviewed.

**Note on the porosity source:** Juhász here uses φ_sh only *inside the conductivity equation* — the
water/HC split still uses the CEC-solved bound water (so PHIE stays put). The *full* "Wet Clay Porosity"
porosity-source that redefines bound water (image-2 constraints panel) arrives with that editor; the two
are the same underlying mechanism and I'll wire them together there.

- [ ] **Juhász vs Simandoux/Indonesia.** On a shaly interval with a good shale pick (Rsh, φ_sh), confirm
      Juhász SWE sits in a sensible band with the other shaly-sand models; on a clean sand it should track
      Archie. Try: Fluid tab → Sw equation → *Juhász / normalized Qv*, set Rsh + φ_sh, Run.

## Round 22 — SandiMin log-input grid + tidy Run button (2026-07-23 field review)

Two visual fixes from your screenshots:

- **Log inputs (image 3 style).** The cramped single column with wrapping labels
  ("Formation Density" breaking across lines) is now a **multi-column grid** — one column
  when the pane is narrow, more as it widens, scrolling both ways, matching the mineral list.
  Labels ellipsis instead of wrapping so the checkboxes stay aligned; hover shows the full
  name + mnemonic.
- **Run button (image 1 style).** No longer a full-width slab — it's now a **tidy, left-aligned
  button** with standard module proportions like Porosity-from-Density, and (per your "then for run
  button" go-ahead) in the **theme accent** so it matches every other module's Run across the
  client-brand skins. This supersedes the earlier "distinct green" — say the word if you actually
  wanted it kept a different colour and I'll bring the green back.

Verified in the browser against the live CSS: log grid resolves to 2 columns at a 560 px pane
(1 when narrower), labels truncate with ellipsis (no wrap), Run renders 76 px wide (not full
width) in the accent colour (rgb(217,140,63) in the dark skin). tsc clean.

- [ ] **Log inputs read cleanly** at your usual pane width — columns wrap sensibly, no label
      overflow, checkboxes line up.
- [ ] **Run button** looks right in the accent colour where it sits at the top of the pane.

## Round 21 — SandiMin Archie (clean-sand) Sw + deduplicated menu decision (2026-07-23)

You chose the **deduplicated** Sw menu (one entry per distinct model). First of the remaining ones:
**Archie (clean sand)** — `Sw = (a·Rw/(φt^m·Rt))^(1/n)`, no shale term. It's the exactly-invertible
baseline (so there's no separate "Archie linear/nonlinear" — they'd be identical). Runs post-solve like
the others: PHIE/PHIT/unity preserved, only the water/HC split moves; on shaly sand it reads
optimistically high (by design — it's the baseline the shaly-sand forms correct). Tests: hand-computed
literals at n=2 and n=3, clamp/NaN guards, and a check that Archie ≡ Indonesia with Vsh=0. cargo + tsc clean.

Menu now: Linear dual-water (default) · Dual-water non-linear · **Archie** · Indonesia · Simandoux.
Still to come: **Waxman-Smits** (dry BQv, Waxman-Thomas B default) and **Juhasz / Normalized-Qv**
(wet-param — brings in the wet-clay-porosity input that also feeds the image-2 porosity-source toggle).

- [ ] **Archie baseline.** On a clean water/HC sand, confirm Archie SWE matches your quick-look Archie;
      on a shaly interval, confirm it reads higher than Simandoux/Indonesia (the expected over-estimate).

## Round 20 — SandiMin non-linear dual-water Sw (the 4th model you picked) (2026-07-23)

The **non-linear dual-water** you asked me to continue is now in the Fluid-tab "Sw equation" dropdown as
**"Dual-water non-linear (m, n separate)."** Unlike the default *linear* dual-water — which folds the
exponents into a single `w = 0.75m+0.25n` and solves the conductivity as a linear row inside the
inversion — this solves the **exact** Clavier-Coates-Dumanoir form honouring **m and n separately**:

    Ct = (φt^m · Swt^n / a) · [ Cw + (Cwb − Cw)·Swb/Swt ]

It runs **post-solve** (same as Indonesia/Simandoux): the mineral inversion runs untouched (the CT tool
stays in, so the split stays well-posed), then Swt is solved from that equation and the water/HC split
redistributed — **PHIT, PHIE and hard unity are preserved**, only SWE/SWT/SXOT move. The **bound-water
saturation comes straight from the solved bound-water volume** (Swb = v_bw/φt), so no lab Qv is needed,
and the clay-bound-water conductivity Cwb is the temperature form already in the fluid calc. Equation
verified against the Geolog `sw_dual` stdlib form.

Tests: hand-computed numeric-literal point (φt=0.3, Swb=0.2, Cw=2, Cwb=5, m=n=2 ⇒ Rt=10.288 ⇒ SWT=0.6),
the effective-Sw conversion (SWE=0.5), a general-n bisection round-trip, NaN guards, and an end-to-end
run recovering a known deep Sw with PHIE untouched. `linear_dw` stays the default — reviewed numbers
unmoved.

Still to come from your image-1 menu: Archie linear/nonlinear, Waxman-Smits, Juhasz + Normalized
Dual-Water (the wet-param normalized-Qv forms), and the wet/dry-clay-parameter wiring.

- [ ] **Dual-water non-linear.** On a well with CT + an HC component (ideally with a clay + BoundWater so
      Swb>0), run once on Linear dual-water then again on **Dual-water non-linear** with your m and n —
      confirm SWE/SWT move to the exact-equation answer while PHIE/PHIT come out identical to Linear.

## Round 19 — SandiMin dialog layout (your field review: run-on-top, tab order, multi-column) (2026-07-23)

Four layout fixes from your image markups, all in `src/ui/multiminDialog.ts` + `src/styles.css`:

- **Run / apply-to-wells on top.** The Apply-to-wells scope, output options, and the **Run** button now
  sit in a boxed section **above** the parameter tabs, so you launch a run without scrolling past every tab.
- **Run button is a distinct green** (`#2e7d4f`), set apart from other modules' accent-coloured runs.
- **Log inputs tab is first** (Log inputs → Minerals → Fluid → Clay) and the pane **opens on Log inputs**.
- **Minerals / Clays / Fluids lists are multi-column** — they wrap to as many columns as the pane width
  allows and scroll both ways, instead of one endless single column.

Browser-verified in the live DOM: tab order + default tab, run-section-before-tabs, the green run colour
(rgb 46,125,79 on white), and the minerals list laying out in 3 columns at a 920-px pane. tsc 0. Nothing
about the solve changed — this is layout only.

- [ ] **Layout sanity.** Open SandiMin: confirm Run + Apply-to-wells are on top, the Run button is green,
      Log inputs is the first/active tab, and the Minerals/Clays/Fluids lists show multiple columns
      (narrow the pane and confirm they reflow / scroll).

## Round 18 — SandiMin Sw-equation selector on the Fluid tab (your request, increment 3b) (2026-07-23)

The backend from Round 17 is now selectable. The **Fluid tab** has a new **"Sw equation"** dropdown —
**Linear dual-water (default)** / **Indonesia (Poupon-Leveaux)** / **Simandoux (modified)**. Pick a
shaly-sand form and two extra fields appear (**Rsh** shale resistivity, default 4.0 ohmm; **Archie a**,
default 1.0) plus a one-line note explaining it runs post-solve and needs a CT tool + a U-zone HC
component. Leave it on Linear and everything behaves exactly as before. Browser-verified: the three
options render, the Rsh/a fields + note show only for Indonesia/Simandoux and hide again on switch back,
and the selector lives inside the conductivity-gated fluid box (so it's present exactly when Rt is). tsc 0.

Still to come (the 4th option you picked — "all of them"): the **in-inversion non-linear dual-water**
(Gauss-Newton, honours m and n separately). It'll drop into this same dropdown when it's ready.

- [ ] **Pick your Sw equation.** Open SandiMin ▸ Fluid: confirm the "Sw equation" dropdown, that
      choosing Indonesia/Simandoux reveals Rsh + Archie a, and that Rsh prefills 4.0 (**set it from a
      shale pick — a too-high Rsh inflates Sw**, wrong-way for fresh-water LRLC pay).
- [ ] **It changes Sw, not porosity.** On a well with CT + an HC component, run once on Linear, then
      again on Indonesia (or Simandoux) with your Rw/Rsh — confirm SWE/SWT move to the shaly-sand answer
      while PHIE/PHIT come out identical to the Linear run.

## Round 17 — SandiMin saturation models: linear dual-water + Indonesia + Simandoux (your request, increment 3a) (2026-07-23)

You asked for a selectable conductivity/Sw equation, "linear and non linear," because it's significant
to the wet/dry clay framework. This increment lands the **backend + math**; the Fluid-tab selector that
exposes it is the next increment (3b), so there's nothing to click yet — this entry is for the record.

What's in the solver now (`src-tauri/src/multimin2.rs`), all behind a new `sw_model` request field that
**defaults to `linear_dw`, so every run you've already reviewed is byte-for-byte unchanged**:

- **Linear dual-water** (default) — the existing in-inversion `Ct^(1/w) = Σ v·C^(1/w)`, `w = 0.75m+0.25n`.
- **Indonesia (Poupon-Leveaux 1971)** — effective-porosity form `1/√Rt = [Vsh^(1−Vsh/2)/√Rsh + √(φe^m/(a·Rw))]·Sw^(n/2)`.
- **Modified Simandoux (Bardon-Pied)** — `1/Rt = φe^m·Sw^n/(a·Rw·(1−Vsh)) + Vsh·Sw/Rsh` (closed-form quadratic at n=2, bisection otherwise).

Both shaly-sand forms are **post-solve**: the mineral inversion runs as usual (the deep-conductivity tool
stays in, so the solve stays well-posed), then Sw is replaced by the closed form using the solved effective
porosity and shale volume, and the U-zone water/HC split is redistributed to honour it — **φe and hard unity
are preserved**, so only SWE/SWT/SXOT change, never PHIE. New fluid inputs `Rsh` (shale resistivity, default
4.0 ohmm) and Archie `a` (default 1.0) feed the shaly-sand forms; the dual-water model ignores them.

Adversarially reviewed (3 lenses — equation transcription, solver integration, contracts). Confirmed the
equations against the standard references and the linear default as unchanged; fixed a real defect (a
shared-zone fluid would be double-scaled by the U- then X-zone override → silent PHIE/unity corruption; now
the flushed override runs only on a zone-disjoint split) and hardened the tests (added an **independent**
hand-computed Archie/shale check so a transcription error fails rather than being self-confirmed by the
round-trips). cargo 288/0.

- [ ] *(No click-through yet — UI is 3b.)* When the selector ships, the check will be: on a fresh well pick
      **Indonesia** or **Simandoux**, set Rw/Rsh, Run, and confirm SWE moves to the shaly-sand answer while
      PHIE/PHIT stay exactly as the linear run produced them.

## Round 16 — SandiMin dialog polish: theme parity + shrinkable/scrollable lists (your review) (2026-07-23)

Two of the three things you flagged on the tabbed SandiMin pane (the third — the conductivity/Sw
equation selector — is a separate, larger change I'm holding for your model choice):

- **Theme parity.** The pane's inputs, selects, and checkboxes were rendering as raw browser
  controls (white box, OS-blue tick) instead of the themed look every module pane uses (your
  image 2, the Porosity Ceiling pane). They now use the brand surface — `--bg-app` fields with
  `--border`, and checkboxes/radios take the theme accent instead of OS blue — so the whole pane
  reads one theme. Scoped to SandiMin for now; a one-line global rule would fix every other pane's
  checkboxes the same way if you want that (say the word).
- **Shrinkable + scrollable lists.** The mineral list is now three collapsible groups —
  **Minerals** (open), **Clays** and **Fluids** (collapsed by default) — each capped-height and
  scrollable, with a live `selected/total` badge on the head. The **Log inputs** list is likewise
  one collapsible, scrollable group with a `on/total` badge. Click any head to shrink/expand.

Browser-verified: the four groups render with correct open/collapsed defaults and counts
(Minerals 1/4 open, Clays 1/2 collapsed, Fluids 2/3 collapsed, Log inputs 5/16-on open), clicking
a head toggles both the collapsed state and the body, and the themed fields/accent resolve to the
active theme's variables. tsc 0.

- [ ] **The pane matches the app theme.** Open SandiMin: the mineral checkboxes, the endpoint
      inputs, and the fluid/clay fields should look like the Porosity Ceiling pane — brand accent
      ticks, themed field backgrounds — not white boxes with blue ticks.
- [ ] **The lists shrink and scroll.** On **Minerals**, confirm Clays/Fluids start collapsed and
      click their heads to expand; on **Log inputs**, confirm the 16-row list scrolls within its
      box. The head badges should track what you've selected/turned on.

## Round 15 — SandiMin dialog: tabbed setup (your request) (2026-07-23)

The Mineral Solver pane was one long scroll — minerals, log inputs, fluid properties, and the
clay converter all stacked. It's now **tabbed**: **Minerals** (component selection + presets +
the endpoint matrix), **Log inputs** (the tool list + user-defined inputs), **Fluid** (Rw/temps/
m/n/mud + precalc autofill), and **Clay** (the wet→dry converter). The run controls — well scope,
output prefix, unity/reconstruction toggles, the Run button and the results/QC — stay in a
**persistent footer below the tabs**, so you set things up across tabs and run from anywhere
without losing your place. The Fluid tab shows a short hint (instead of going blank) when no
conductivity tool is active, since the fluid numbers only matter to CT/CXO. Nothing about the
solve, endpoints, or wiring changed — this is purely how the pane is organized. Browser-verified:
tab switch shows exactly one panel, the CT toggle flips the fluid hint/grid, the footer stays put.

- [ ] **The pane is easier to navigate.** Open SandiMin (Modules ▸ SandiMin): confirm the four
      tabs across the top, that clicking each shows only that section, and that the Apply-to-wells
      scope + Run button + results stay visible no matter which tab you're on.
- [ ] **Nothing regressed.** Set up a clastic run as before — pick minerals on **Minerals**,
      confirm your tools on **Log inputs**, set Rw/temp on **Fluid**, Run from the footer — and
      check you get the same curves and DOF/incoherence readout as before the reorganization.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

The SHF fitting engine now covers all five families and can split by rock type. **Thomeer** joins
the height-domain forms: the carbonate-standard hyperbola `Sw(H) = 1 − (1−Swirr)·exp(−G/log10(H/Hd))`
(Thomeer 1960), fitted with the same bounded simplex as Skelt — Hd is the entry height (the
displacement pressure expressed in metres above the FWL) and G the pore-geometrical factor
(≈0.1 well-sorted → >2 poorly sorted). **Leverett-J now fits from logs**, not only at SCAL
import: each sample's height becomes reservoir Pc (0.433·Δρ·h_ft), J = 0.21645·Pc/σcosθ·√(k/φ)
from the PERM/PHIE curves, and Sw = A·J^B is regressed in ln-ln space (Leverett 1941). Fluid
defaults are Tier-A seeds — σ·cosθ 26 dyn/cm (IP cap-pressure table, Water-Oil 30 dyn/cm·cos 30°),
HC density 0.7 g/cc (Techlog) — all per-run overridable. **Per-rock-type fits**: hand any family
an RT/facies curve and it fits one law per rock-type class alongside the pooled law (the single
biggest SHF accuracy win on stacked Mahakam sands); classes that can't fit are reported with the
reason, never dropped. **Nothing is dropped silently anymore**: every excluded sample is counted
by reason (Sw > 1, at/below the FWL, below the φ cutoff, no permeability), scoped wells that
contributed zero samples are named in a note, and a Buckles check (Buckles 1965) flags when the
above-transition BVW isn't one constant — the classic sign you need per-rock-type laws. The
breakdown survives even when the fit itself fails — that's when you need it most.

Adversarially reviewed (37 agents, 4 lenses → 3-skeptic verification): 8 confirmed findings → 4
distinct defects, all fixed pre-commit — a Thomeer bounds panic on sub-millimetre height ranges
(HIGH), silent zero-contribution wells, discarded exclusion counters on two FOIL error paths, and
the failed-group NaN→null IPC contract. cargo 283/0, tsc 0. (Dialog UI for all of this = 4b, next.)

## Round 15 — Saturation-height dialog: 5 families, per-rock-type tabs, draggable FWL (playbook #4, increment 4b) (2026-07-23)

The Saturation-Height dialog now drives everything the 4a solvers added. The **SHF-form dropdown
has five entries** (FOIL / Brooks-Corey / Skelt / Thomeer / Leverett-J); picking **Leverett-J**
reveals a permeability-curve picker and a fluid-property block (system dropdown that flips σ·cosθ
between the Water-Oil 26 and Water-Gas 50 dyn/cm Tier-A seeds, plus ρw/ρhc — all editable). A
**"Fit per rock type" checkbox + RT-curve picker** turns any family into per-class fits: the
results panel grows a **tab strip** (All / RT 1 / RT 2 …), each tab showing that class's
parameters, R², and its own Sw-vs-height curve; classes that couldn't fit show a ⚠ tab with the
reason instead of vanishing. Every result now carries a **diagnostics line** — the excluded-sample
breakdown (Sw > 1, at/below the FWL, φ-cutoff, no-perm counts) and the honesty notes (zero-
contribution wells, the Buckles warning) — shown on both success and failure. The **FWL is
draggable**: drag horizontally on any result plot to nudge it (0.2 m/px) and it re-fits on release,
or click straight on the FWL-scan curve to pick a candidate. An **RMS** row joins R² in every
parameter table. tsc 0.

- [ ] **All five families fit.** Analysis ▸ Saturation-Height on BLSO: run each of FOIL,
      Brooks-Corey, Skelt, Thomeer, Leverett-J. Thomeer and Leverett-J should return sensible
      params (Thomeer G ~0.1–2, Leverett B negative) with a curve through the Sw-vs-H cloud.
- [ ] **Leverett-J uses PERM.** Pick Leverett-J → the PERM picker + fluid block appear; switch
      the system Water-Oil↔Water-Gas and watch σ·cosθ flip 26↔50. Fit with your PERM curve.
- [ ] **Per-rock-type split.** Tick "Fit per rock type", pick your RT curve, fit: a tab per RT
      class appears, each with its own law + curve; a thin class shows a ⚠ tab with the reason.
- [ ] **FWL by drag / click.** Drag left-right on the crossplot — the status shows the trial FWL
      and it re-fits on release; on FOIL with the scan on, click the scan curve to jump the FWL.
- [ ] **Nothing hides.** Set the FWL above the whole cloud and fit: the failure now shows an
      "Excluded: at/below the FWL: N" breakdown instead of a bare error.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

## Round 13 — Theme sweep: canvas typography + color tokens (playbook #9A, increment A) (2026-07-22)

Every canvas font and the last hard-coded colors that bypassed the theme are now driven by the
theme system, so plots, dialogs, and overlays stay legible and on-brand across all eight skins
(light / dark / Pertamina / Halliburton / Schlumberger / LAPI-ITB / white / system). An inventory
workflow (4 parallel sweeps) found **111 bypasses across 20 files**; all fixed. New tokens:
`--font-canvas` (the Segoe-variable stack) and `--font-mono` in styles.css, a `canvasFont(theme,
size, weight)` helper on the shared plot scaffolding (`PlotTheme` gained `fontFamily`), so all
~55 `ctx.font` literals now resolve through one token. Color fixes: the well-diagram casing
strings/shoes (was mid-gray `#5a5a5a`/`#333` — invisible on dark) now use `--text`; perforation
ticks use `--warn`; the crossplot/Pickett "no-data" gray marker now derives from `--text-dim`;
the highlights default palette and the "Add curve" default color are built from the live theme
accents instead of fixed light-theme values. Browser-verified across all six branded palettes:
6 distinct accents, 6 distinct no-data markers, the font token resolves and stays stable, and the
derived palettes are all valid hex (safe for the color pickers). tsc 0, production build clean.

- [ ] **Themes stay legible everywhere.** Cycle the theme (ribbon ▸ theme) through dark and a
      client brand (Pertamina/SLB) with a log view, a crossplot, and the well-diagram track open:
      axis/label text, casing strings + perforations, and crossplot no-data points should all
      stay readable — nothing washes out or disappears the way the old mid-gray casing did on dark.
- [ ] **New curves + highlights adopt the brand.** On a branded theme, add a curve in Layout
      Properties and drag a highlight band: both should come up in the theme's accent, not the
      light-theme terracotta.

## Round 12 — Monte Carlo sampling engine: LHS, rank correlation, convergence (playbook #1, increment 1.1) (2026-07-22)

The Monte Carlo engine's draw generation is rebuilt to commercial grade. **Latin Hypercube
Sampling is now the default**: each parameter's probability range is split into N equal strata
with one jittered draw per stratum (order shuffled per parameter), so the sampled CDF matches the
distribution far tighter than independent draws at the same N — P10/P90 bands stabilize with
fewer iterations (McKay–Beckman–Conover 1979). The old scheme survives as `sampling: "random"`
and reproduces pre-upgrade results byte-for-byte at the same seed. Two new opt-ins: **parameter
rank correlations** (Iman–Conover 1982 — e.g. tie RHO_MA to GR_MA at ρ 0.7; marginals are only
reordered, never altered, and inconsistent/unknown pairs come back as notes, not errors) and a
**convergence check** (running P10/P50/P90 of total HPV per batch; in random mode the run stops
early once the trace goes stationary — LHS always runs its full design, since truncating one
would leave strata unsampled). `montecarlo.rs` + `ipc.ts`; 5 new tests (legacy request shapes parse with LHS defaults;
exactly-one-draw-per-stratum + analytic mean; achieved Spearman hits ±targets and marginals are
pure reorderings; flat series early-stops with a consistent truncated result; LHS never
truncates). cargo 274/0, tsc 0. The LHS/random toggle, correlation editor, and convergence
sparkline arrive in the dialog with increment 1.3 — until then the pane simply runs LHS.

Adversarially reviewed (18-agent workflow, 4 lenses × 2 skeptics); all 4 confirmed findings fixed
in the same round: (1) correlation targets are now pre-adjusted by the Spearman→Pearson map
2·sin(πρ/6), so the achieved rank correlation centers on your ρ instead of landing ~0.014 low;
(2) a duplicated/conflicting correlation pair now reports "last entry wins" in `notes` instead of
resolving silently; (3) the convergence trace folds the remainder into its final batch, so the
end-of-run "converged" verdict can't be inflated by a runt 4-realization checkpoint; (4) a
**pre-existing tornado bug**: with a zone that has no pay at the parameter medians, switching the
sensitivity metric to Avg PHIE/Avg SWE crashed the pane (`null.toFixed`), and a single dry sweep
endpoint drew a bar anchored at a fabricated 0 — the renderer now says the base case has no
anchor and drops non-finite endpoints.

- [ ] **LHS is quietly better, not different.** Monte Carlo pane ▸ your usual GR_MA/RHO_MA setup on
      a real well, 1 000 iterations, seed 42 ▸ Run twice — identical results (reproducibility
      holds). Then drop to 300 iterations and re-run a few seeds: the P10/P90 HPV band should sit
      noticeably steadier across seeds than you remember from the old sampler at 300.
- [ ] **Dry-zone tornado no longer crashes.** Monte Carlo ▸ tornado on ▸ pick a marginal zone that
      has no pay at your median cutoffs ▸ after the run, switch the sensitivity Metric to
      Avg PHIE: you should get the "base case yields no Avg PHIE" message (previously this threw
      `TypeError … toFixed` and left a half-drawn panel).

**Increments 1.2 + 1.3 (same round):** distributions can now be **zone-scoped** — each uncertainty
row has a zone box (suggestions from the scoped well's zonation); a scoped draw applies only inside
that zone, everything outside follows the deterministic zone parameters, and the tornado/Spearman
rows are labeled `PARAM @ ZONE`. **Save LOW/BASE/HIGH curves** writes per-sample uncertainty curves
to a fresh **version** of the MONTECARLO log set per well (never overwrites — the Sets manager can
restore any run): `MC_<KEY>_LOW/_P50/_HIGH` are per-sample percentiles across realizations and
`MC_<KEY>_BASE` is one deterministic run at every parameter's median, for each of VSH/PHIE/SWE/PERM
the chain produces. The dialog grew the **Sampling** select (Latin Hypercube default / Random
legacy), the **Correlations** mini-editor (param ↔ param, ρ), **Convergence check** and **Save
curves** checkboxes, a per-well **convergence sparkline** (running P-low/P50/P-high with a
converged/not-converged badge), and a notes panel that surfaces backend advisories (skipped
correlation pairs, persist confirmations). Status line reports sampling, early-stop count, and
saved-curve count. 5 more cargo tests (zone-scoped spread stays in its zone + unknown-zone note;
persisted curves ordered LOW ≤ P50 ≤ HIGH and versioned v1→v2; inverted zone; input-skip;
stale-family reclaim + degenerate base). Browser-smoke-tested end-to-end.

The 1.2 backend was adversarially reviewed too (27-agent workflow); all 7 distinct confirmed
findings fixed before commit: an inverted zone (top ≥ bottom, storable via the DB inspector) now
yields a note instead of **panicking the whole run**; correlations naming a parameter that appears
in several zone-scoped entries note that ρ binds only the first; persisted curves are gated on
what the chain **produces** (inputs it merely consumes no longer come back as zero-width fake
uncertainty bands); the kept-snapshot pool survives convergence early stops (first-N prefix
instead of a precomputed stride); a re-run that writes fewer curve families reclaims the previous
version's stale MC_* rows from the current store (archive keeps every version restorable); a
degenerate all-median base run skips only MC_*_BASE with a note instead of discarding the valid
percentile curves; and a well whose persist write fails now finishes its job item **Warned**, not
Ok.

- [ ] **Zone-scoped uncertainty stays in its zone.** Monte Carlo ▸ add GR_MA, type a real zone name
      in its zone box (the box suggests your zones) ▸ Run: the named zone's P10–P90 band spreads,
      every other zone's collapses to a single value, and the tornado row reads "GR_MA @ <zone>".
- [ ] **Saved uncertainty curves land as a versioned set.** Tick "Save LOW/BASE/HIGH curves" ▸ Run
      ▸ open a layout and add MC_PHIE_LOW/P50/HIGH on a track: a proper uncertainty envelope
      around the P50, with MC_PHIE_BASE hugging your deterministic PHIE. Re-run — the Sets manager
      shows MONTECARLO v2 alongside v1.
- [ ] **Correlated draws + convergence read sensibly.** Add GR_MA and RHO_MA, correlate them at
      ρ 0.7, tick Convergence check, sampling Random, 5 000 iterations ▸ Run: the sparkline
      flattens and the run stops early with "stationary after N" (with LHS it always runs full
      size and says so).



Backend for the SandiMin reconstruction check. The existing **RECON** curve is now documented as the
**incoherence** — the σ-weighted RMS of (reconstructed − measured) over the live tool rows (Quanti.Elan
Eq 79). With the new **`recon_qc`** request flag the reconstruction is **decomposed per tool**:
`<prefix>_<KEY>_REC` = the log rebuilt from the solved volumes (in the tool's display units, so it
overlays the measured curve) and `<prefix>_<KEY>_DIF` = that tool's σ-unit residual (whose RMS over
tools is RECON). The result also reports model **degrees of freedom** `dof = (tools + soft + unity) −
components`, with a note when `dof == 0` (exactly determined → RECON is forced to ~0 and can't validate
the model). `multimin2.rs` + `ipc.ts`; 2 new tests (a forward-modeled 3-mineral well reconstructs to
incoherence ~0 and a wrong illite density inflates it + localizes to the density residual; the
exactly-determined case flags its note). cargo 269/0, tsc 0. **The recon-QC view shipped in the same
round (increment 2d):** a **Reconstruction QC** checkbox in the SandiMin dialog turns the per-tool
curves on; after the run the result shows the **model DOF** (with the exactly-determined warning) and a
**measured-vs-reconstructed crossplot** (each tool min-max normalized, points on the dashed 1:1 line =
a perfect fit, scatter off it = that tool's incoherence). Browser-smoke-tested: checkbox → run → DOF
line + crossplot render.

**Increment 2c** completed **#2** per your call to keep smectite as-is: a **Preset** selector atop the
component picker with four named GROUPINGS of existing library components — **Clastic**
(quartz–illite/kaolinite–water+bound), **SSC-style** (quartz–feldspar–clay, to compare VOL_* against
the SSC module's VSAND/VSILT/VCLAY), **Carbonate** (calcite–dolomite–anhydrite), **Organic/coal**
(quartz–illite–coal–kerogen, whose VOL_KEROGEN feeds the upcoming unconventional workflow). Presets
carry **no endpoint values** — Montmorillonite keeps RHOB 2.63 etc., so no reviewed number changed;
manually ticking a component drops back to "— custom —". Browser-smoke-tested all four.

- [ ] **Presets assemble the right model.** SandiMin ▸ Preset ▸ each of the four: the component
      checklist follows the grouping (note under the selector explains each), endpoints stay exactly
      what the library/your edits hold, and a manual tick resets the selector to custom. Run the
      Clastic preset on a Mahakam well and sanity-check VOL_QUARTZ/VOL_ILLITE against your SSC results.

- [ ] **Reconstruction flags a bad model.** In **SandiMin ▸ tick "Reconstruction QC" ▸ Run**. On a
      good model the crossplot points hug the 1:1 line and the incoherence stays low; force a wrong
      endpoint (or drop a needed mineral) and confirm the points for the broken tool scatter off the
      diagonal and the incoherence rises. The written `<prefix>_<KEY>_REC` curves can also be laid over
      the measured logs in a log view for a depth-by-depth check.
- [ ] **DOF honesty.** Build a model with exactly as many inputs as components (e.g. 3 minerals, 2 logs
      + unity). Confirm the dialog shows **DOF 0** in orange and warns that RECON can't validate the
      model; add one more input log and DOF rises to 1 (RECON becomes meaningful).

## Round 10 — Stratigraphic Modified Lorenz Plot: flow-unit solver (playbook #3, increment 3a) (2026-07-22)

New backend `lorenz.rs` — the **Stratigraphic Modified Lorenz Plot** (Gunter et al. 1997, SPE 38679).
It walks a well's φ + k logs in **depth order**, accumulates flow capacity Σ(k·h) against storage
capacity Σ(φ·h) (each normalized 0..1), segments the depth-ordered log10(k/φ) profile into **flow
units** with an exact contiguous dynamic program (auto-K by marginal gain, or a caller-set K), and
reports the **Lorenz heterogeneity coefficient** (Schmalz & Rahme 1950). Command `run_lorenz` +
`runLorenz` in `ipc.ts`. cargo **265/0** (9 new `lorenz` tests, incl. a synthetic 3-flow-unit column →
3 units), tsc **0**. Adversarially reviewed (4 lenses → **1 confirmed** IPC-nullability fix applied;
math + segmentation lenses clean). Method banked in `docs/ref_rock_typing.md`.

The **visual** (increment 3c-1) shipped in the same round: new pane **Lorenz Plot (flow units)** in
the ＋ add-panel menu — well + φ/k curve pickers (group-filtered, defaults to the selected well;
PERM list prefers PERM/KLOGH/PERM_RT), auto or forced K, optional MD window, then the SMLP curve
coloured by flow unit against the dashed 45° homogeneous diagonal, the per-unit table (top/base,
storage %, flow %, slope, **speed/baffle** character), and the Lorenz-coefficient headline.
Browser-smoke-tested on a stubbed 3-regime column: 3 units recovered, unit 1 = speed with 90 % of
flow from 33 % of storage, row-click highlight dims the other units.

**Increment 3c-2** completed **#3**: (a) a **Winland/Pittman pore-throat grid** on the crossplot —
Crossplot Properties ▸ *Rock-type grid* draws iso-radius lines at the port-class bounds
(0.1/0.5/2.5/10 µm) when one axis is porosity and the other permeability (Kolodzie 1980 R35 or
Pittman 1992 r25/r35/r50); (b) the **facies tie-in now also reports k-variance-reduction** — how
much of the core log10(k) spread the predicted rock-type class explains (ANOVA 1 − SSw/SSt), so the
tie-in is validated against permeability, not just class purity; (c) **RT as a FACIES block track**
needs no new code — set any integer RT curve's fill to **Facies blocks** in the log-view layout
props. cargo 267/0, tsc 0. (3b, the Pittman full-apex r10–r75 table, was already the `pittman_rx`
module.)

- [ ] **SMLP + flow units on a real well.** On a well with PHIE + a permeability curve (imported
      KLOGH, computed PERM, or the rock-typing PERM_RT), open **＋ add-panel ▸ Lorenz Plot (flow
      units)** ▸ Build Lorenz Plot. Confirm the curve ends at (1,1), and steep **speed** segments
      coincide with your best reservoir sands (high k/φ) while flat **baffle** segments fall on
      shale / tight streaks — the flow-unit boundaries should track your net-sand tops.
- [ ] **Lorenz coefficient sanity.** A clean, well-sorted sand gives a **low** coefficient (near 0);
      a layered sand-shale interval a **high** one (→1). Use the MD window (a zone's top/base) to
      Lorenz two zones you know differ in heterogeneity and confirm the number moves the right way.
- [ ] **Winland/Pittman grid on a φ-k crossplot.** New Crossplot ▸ X = PHIE, Y = a permeability
      curve (log Y on) ▸ Properties ▸ **Rock-type grid = Winland R35** (or a Pittman rX). Confirm the
      dashed iso-radius lines (0.1/0.5/2.5/10 µm) fan across the cloud and your best plugs sit in the
      macro/mega band. Flip the axes — the grid should still draw (orientation auto-detected).
- [ ] **Facies tie-in explains permeability.** On a well with a core-derived RT + a log RT and core
      k, run **Facies Tie-in**. Besides purity, confirm the **k variance reduction %** appears and is
      high when the classes separate core k, low when they don't (needs core plugs within 1 m of the
      log samples).

## Round 9 — Cross-feature fix: survey TVD/TVDSS must not shadow an imported one (2026-07-22)

A cross-feature adversarial review of the four shipped feature_work commits (constants/TVD/ML-MASK/
DLIS) found one real HIGH seam bug between TVD materialization (Round 6) and the standard→computed→
generic resolution order (Round 8): importing a deviation survey wrote a **computed** TVD/TVDSS, which
outranks the generic store, so it silently shadowed an authoritative TVDSS a user had imported from a
vendor LAS/DLIS — with a possibly wrong datum (no-KB wells fall back to a sea-level datum) or NaN
outside the survey's MD range, and no recourse via Promote (disabled on a "served by computed" row).
Fixed in `materialize_tvd_curves` (ingest.rs): it now only materializes a name the well does not
already resolve from an import, and clears any stale survey-derived computed curve so the import keeps
winning. cargo 256/0, tsc unchanged. Test `materialize_tvd_keeps_imported_tvdss_authoritative`.

- [ ] **Vendor TVDSS survives a survey import.** On a well that has a TVDSS curve from its LAS, import
      a deviation survey. Confirm the plots/modules still read the **imported** TVDSS (unchanged values,
      full depth coverage) — not a survey-derived one. TVD (if not imported) still appears from the survey.
- [ ] **Recompute is still safe.** Edit KB and run Data ▸ Recompute TVD/TVDSS. A well WITHOUT an imported
      TVDSS refreshes its survey-derived TVDSS; a well WITH an imported TVDSS keeps the imported one.

## Round 8 — DLIS/LAS mnemonic-shadow resolution in the Curve Catalog (2026-07-22)

When a DLIS and an LAS (or two DLIS runs) carry the **same mnemonic**, the Curve Catalog now
detects the collision, badges the resolver's current winner, and lets you **Promote** the one you
want or **Delete** a duplicate — without editing files. Backend `db.rs` (new `pinned` column +
promote/delete), resolver tiebreak in `equations.rs` + `curve_edit.rs`, frontend
`inspectorPanel.ts`/`ipc.ts`/`styles.css`. cargo 255/0, tsc 0. Adversarially reviewed (4 lenses →
**5 confirmed findings, all fixed**): the resolver no longer lets a pin leak across a family, and the
Catalog no longer claims a Promote "wins" when a higher-priority store actually resolves the curve.

- [ ] **Promote resolves a real same-mnemonic shadow.** On a well where a DLIS and an LAS both carry a
      **non-standard** mnemonic (e.g. `PEF`, `CALI`, `DTS`, or a core `PERM` with no computed PERM),
      open the **inspector ▸ Curve Catalog**: the two rows show **`resolves`** / **`shadowed`** badges.
      Click **Promote** on the shadowed one → it flips to `resolves` + `pinned`, and any plot/module
      reading that curve now picks up the promoted values. **Delete** the loser → the sibling resolves.
- [ ] **No false "it now wins" for standard logs.** For `GR / RES_DEEP / NPHI / RHOB / DT / SP`, the
      real curve is served from the standard log column, not the RAW catalog copy. Those rows now show a
      neutral **`served by log`** badge and **Promote is disabled** (tooltip: "resolution comes from the
      standard log column — promoting has no effect"). Previously Promote here claimed victory but changed
      nothing on any plot — that lie is gone.
- [ ] **No false win when a computed curve owns the name.** If you've computed a curve (say `PERM` from
      Coates) and also imported a raw `PERM`, the raw row shows **`served by computed`** and Promote is
      disabled — the computed curve resolves first, so promoting the raw one would have been a silent
      no-op.
- [ ] **A pin doesn't hijack the family (deep-R sanity).** Promoting one same-mnemonic shadow must NOT
      change which curve a **family** request resolves. On a well whose deep-resistivity feeds Sw, promote
      an unrelated same-mnemonic shadow and confirm Sw is unchanged (the pin now applies only to its own
      mnemonic, and family requests rank by base run — deterministic across re-import/reopen).

## Round 7 — MASK support in the ML pipeline (2026-07-22)

Optional flag curve in the ML dialog: samples where the mask = 1 are excluded from training AND left
blank (NaN) in the prediction — the same 0/1 convention as the module MASK. Backend `ml.rs` + frontend
`mlDialog.ts`/`ipc.ts`. cargo 253/0, tsc 0. Adversarially reviewed (3 lenses → 2 confirmed honesty
fixes applied).

- [ ] **Masked training + apply.** On a well carrying a BADHOLE / FLAG_PAY / COAL 0-1 flag curve, open
      ML Models, pick a **Mask (exclude)** curve, run a regression/classification → confirm the output
      curve is BLANK (NaN) at flagged depths and the per-well "Predicted samples" count drops.
- [ ] **Mask governs clustering/PCA too.** For an unsupervised task the mask keeps flagged samples out
      of the fit AND leaves them blank — facies with vs without a mask differ (bad-hole shouldn't shape
      facies).
- [ ] **Leaderboard honesty.** In **Compare algorithms** with a mask that empties a whole training
      well, the header shows the TRUE contributing-well count and a note that blind-well CV fell back
      to random KFold (previously it hid the collapse behind the requested well count).

## Round 6 — TVD/TVDSS as fetchable curves (2026-07-22)

Materialize the deviation survey onto the log depth grid as `TVD` and `TVDSS` computed curves,
so height-based tools can consume them by name. Backend `deviation.rs`/`ingest.rs`/`lib.rs` +
frontend `ipc.ts`/`ribbon.ts`. cargo 250/0, tsc 0.

- [ ] **Deviation import now writes TVD/TVDSS curves.** On a **deviated** well with logs loaded,
      Data ▸ Import Deviation… a survey → confirm `TVD` and `TVDSS` appear as computed curves
      (Curve Catalog / any module's log-input dropdown). TVD should be shallower than MD in the
      built section; TVDSS = KB − TVD.
- [ ] **`sw_height` TVD input now works.** Run the Saturation-Height module selecting the new `TVD`
      curve for the TVD input — on a deviated well the height (HAFWL) and SWH now use true vertical
      depth instead of MD (previously the TVD input was a silent no-op → MD fallback → optimistic pay).
- [ ] **SHF fits can use the materialized TVDSS.** In the Cuddy FOIL / Brooks-Corey / Skelt / Thomeer
      panes, pick the new `TVDSS` curve as the vertical-depth input and confirm the fit runs.
- [ ] **Correlation TVDSS depth-mode** now works from the survey (not only from an imported TVDSS log).
- [ ] **Data ▸ Recompute TVD/TVDSS Curves** — run after importing logs *after* the survey, or after a
      KB edit. Status reports "computed for X of Y surveyed well(s), N samples"; surveyed wells with no
      logs yet are counted as pending. *(Note: the survey-derived TVDSS is written to the computed store,
      which takes precedence over an imported TVDSS log of the same name when fetched.)*

## Round 5 — Rock-typing constants verification vs papers (2026-07-22)

Read-only cross-check of every hardcoded literature constant in `rocktyping.rs` / `shf_fit.rs` /
`thomeer.rs` / `hfu.rs` (+ `satheight.rs`) against `docs/research_2026-07/ref_rocktyping_shf.md` and
the published sources. Full write-up: `docs/constants_verification_2026-07-22.md`. **2 corrections
applied (both number-changing, Jauhar approved); 1 held pending a primary-source glance.** cargo
247/0, tsc N/A (no TS).

- [ ] **GHE FZI bins corrected** (`rocktyping.rs`). Was `…1.5, 2.5, 4, 6, 8`; now the Corbett-Potter
      2004 ×2 series `…1.5, 3, 6, 12, 24`. Run the **Rock Typing (FZI/R35/PGS)** module with
      `METHOD=ghe` on a cored well and confirm the `RT` (GHE class) curve looks right for the
      best-quality rock — high-FZI samples now land in the correct GHE6–GHE10 bands (previously
      compressed). `PERM_RT` follows the class, so it shifts too.
- [ ] **PGS definitions corrected** (`rocktyping.rs`). `PGEOM` is now `√(k/φ)` (was `k/φ`) and the
      `PS_EXP` default is `3.0` (was `3.5`) — the ACS Omega 2024 / Kozeny-Carman form. Diagnostic
      curves only (RT class is unaffected). Confirm `PGEOM`/`PSTRUC` plot sensibly; `PS_EXP` is still
      an editable param if you want a different exponent.
- [ ] **Pittman r75 — HELD (not changed).** The code's r75 row `(1.243, 0.674, −1.517)` diverges from
      the widely-cited `≈(0.778, 0.626, −1.205)` while r10–r50 all match. Couldn't confirm online
      (Pittman's Table 1 is an image; primary is paywalled). If you can check **AAPG Bull. v76 (1992)
      p191-198, Table 1**, tell me the r75 coefficients and I'll fix the one row. Only affects `PR75`
      and `RT_PITT` when APEX=r75 (default r35 is fine).

## Round 4 — AUDIT-2026-07-21 safe-bucket follow-through (2026-07-22): correctness / honesty / robustness

Continuation of task #159 (the 65-finding full-QC audit). After batches 1–3 (`1d6b521`/`5e44620`/`1dcfeba`)
and the RT≤0 fix (`f33e126`), this round works the remaining **safe** bucket — fixes that harden behaviour
or improve reporting honesty WITHOUT changing interpretation numbers for valid data. Audit references were
re-verified against CURRENT code first (several were already fixed by the round-2/3 refactors — e.g.
correlation already subscribes to dataVersion; recordProcess already wired in ML/multimin/inspector).
**cargo 247 pass / 0 fail / 7 ignored; tsc EXIT 0. Nothing committed.**

Backend (Rust, unit-tested):
- [ ] **Cutoff-sweep geometric clamp.** `run_cutoff_sweep` now integrates each sample's clamped overlap
      with the zone ∩ DST interval (mirrors `run_pay_summary`), so NTG can no longer exceed 1 when a
      zone/DST boundary lands mid-sample. Sample-aligned results are byte-identical. **Try:** run Cutoff
      Sensitivity with a DST interval whose edges don't fall on log samples — NTG should stay ≤ 1 and agree
      with the Pay Summary for the same well/zone/cutoff.
- [ ] **Per-well isolation** in `run_pay_summary` + `run_cutoff_sweep`: one well's fetch/zone read error
      now skips just that well instead of zeroing the whole Field Dashboard / sweep response.
- [ ] **All-NaN module runs report honestly.** A module run whose every output sample is MISSING (e.g.
      gascorr with no precalc, or a module fed an all-NaN input, or SW-RtC on a well with no PHIT) is now
      reported as an error / Warned in the Processing panel — not a green "N samples → …" success. Same
      guard on Rhai + Python equations (an unresolvable input/output curve → error, not a clean success).
- [ ] **Python in-place equation guard.** An equation whose output curve name collides with an input
      (a "clean this curve in place" script) no longer silently writes the untouched input back when the
      script forgot to (re)assign it. (Also fixed a worker crash when the output was named `np`/`numpy`.)
- [ ] **LRLC SSPW fallback.** SW-RtC / SW-IMTS now fall back to the SSPW-named curves (PHIT_SSPW /
      CAPBW_SSPW / CBW_SSPW) when the SSC ones are absent — so they run on an SSPW-processed well instead
      of silently producing all-NaN. SSC-only wells are unchanged. **Try:** run SW-RtC on a well processed
      through SSPW porosity (no SSC curves).
- [ ] **LAS duplicate-name warning.** Importing a LAS whose (normalized) well name already exists now
      warns (still creates a separate record — merge is a deliberate action, not automatic). **Try:**
      import the same LAS twice; the second shows a "already exists" warning.
- [ ] **New test coverage** (no behaviour change): phi_den / phi_dn edge cases (VSH≥0.95 shale branch,
      SHALE_REDUCED-vs-MAXIMUM cap, density shale-reduction clamp, AVERAGE-vs-GAS_RMS), SSC `*_GR` family
      closure + degenerate-VWSH guard, and `run_ml`'s DB-integration guards.

Frontend (TS, tsc-clean):
- [ ] **History attribution.** A scoped module run records the wells actually run (single by name, batch
      as null) instead of the globally-selected well (which a scoped run may not have touched).
- [ ] **Blank "(none)" for optional inputs.** Optional log-input dropdowns now offer "(none)" so you can
      deliberately drop a curve slot even when a curve of that name exists in the project.
- [ ] **dataVersion refresh** after equation / ML / report runs and on workflow-chain **cancel/fail**
      (a cancelled chain routinely committed the earlier wells) — open plots/log views no longer show stale
      curves.
- [ ] **Race guards** on the module pane's data refresh (a slow refresh can't overwrite a fresher one) and
      SandiMin's **Autofill-from-precalc** (a well switch mid-fetch no longer stamps stale FTEMP/RMF).
- [ ] **Pay Summary → Processing History** (the FLAG-writing Compute now leaves a trace); **curve-edit
      Set-constant** rejects non-finite (Infinity) input; the deprecated legacy **`multimin`** module is
      filtered out of the Workflow step picker (use SandiMin).

Deferred / needs your call (see the summary I sent):
- Report "Tables only" still computes the composite geometry (efficiency, not correctness) — a truly safe
  fix must reproduce the cover interval exactly, which needs the same expensive fetch. Held.
- Low-value polish left: MC histogram theme-repaint; ml/wellScope dataVersion subscribe.
- **6 findings that WOULD change interpretation numbers** await your sign-off (perm_coates default 100→70;
  phi_son OPT_CP DT_SH>100 gate; log_predict masked-fill survival; legacy-multimin RECON_ERR at 3 tools;
  MC PERM cutoff when chain-produced; MC MASK/computed_only parity).

## Round 3 — Feature Wave B chain (2026-07-22): fluid contacts, ML leaderboard, well-diagram, rock typing + SHF

Four Wave B features built back-to-back after the round-2 commit (`d64bdc7`). Each is tsc-clean and
either cargo-tested or cargo-check-clean; the novel math in each is unit-tested. **Not yet clicked
through in the real app with field data. Nothing committed.**

- [ ] **(9) Fluid contacts in Correlation.** New `fluid_contacts` store (well/field/global scope,
      OWC/GWC/GOC/GDT/ODT/FWL, depth, TVDSS flag, colour) + editor (Correlation ▸ **Contacts…**).
      Contacts draw as horizontal lines + cross-well connectors. New **MD / TVDSS depth mode** on the
      Correlation toolbar: in TVDSS a TVDSS-stored contact is **flat across every well** (converted per
      well via the TVDSS curve; falls back to MD == TVDSS for vertical wells). *(Verified: the TVDSS↔MD
      round-trip math — a TVDSS contact renders flat across two wells with different deviation, an MD
      contact flat only in MD mode; cargo check + tsc clean.)* **Try:** open Correlation, add an OWC as
      TVDSS, switch MD↔TVDSS, watch it flatten.
- [ ] **(3) ML comparison leaderboard.** In the ML pane (supervised tasks), a **Compare algorithms**
      button ranks every algorithm × a curve-subset strategy (full / leave-one-out / singles) by
      **blind-well GroupKFold CV** — whole wells are held out, fixing the depth-leak in the old random
      5-fold. Shows a sortable leaderboard (R²/accuracy + RMSE/macro-F1), **permutation importance** bars,
      and a **confusion matrix** for the selected row. *(Verified: 2 new Rust tests exercise the real
      sklearn GroupKFold path — blind-well R²≈1 for a linear law across 3 wells, 2×2 confusion for a
      classifier. Needs ≥2 train wells.)* **Try:** ML ▸ regression, pick ≥2 train wells + curves ▸ Compare.
- [ ] **(16) Well-diagram track.** Any layout track can be set to **kind = Well diagram** (Layout editor ▸
      Track type). It draws casing/tubing/liner strings (with shoe symbols) + perforation ticks from the
      well's **COMPLETION** and **PERFORATION** aux datasets (Data ▸ Import aux data; value_num = OD in
      inches, depth_top..depth_base = the run). Renders in the log view **and** the composite/report SVG.
      Old saved layouts still load (kind defaults to "curves"). *(Verified: cargo check + tsc clean;
      renderer skips curves for diagram tracks so nothing draws underneath.)* **Try:** import a COMPLETION
      CSV, add a track, set it to Well diagram.
- [ ] **(8) Rock typing + SHF — increment 1.** Two pieces:
      **(a) Rock Typing module** (Petrophysics ribbon ▸ new *Rock Typing* group) — from φ + k writes
      RQI, PHIZ, FZI (Amaefule), Winland **R35**, PGS **PGEOM/PSTRUC**, an **RT class** (GHE fixed FZI
      bins or Winland port classes) and **PERM_RT** (class-grouped geometric-mean-FZI perm estimate).
      *(4 unit tests: FZI→GHE7 for φ0.2/k100, Winland R35→macro, perm predictor, MISSING handling.)*
      **(b) Cuddy FOIL SHF fit** (workspace ▸ **SHF Fit (Cuddy FOIL)**) — pools computed PHIE/SW/TVDSS
      across wells, fits **BVW = a·H^b** above the FWL with a BVW-vs-H log-log crossplot, and an optional
      **FWL scan** (Cuddy 1993 Eq 19) that finds the common contact. *(3 unit tests: recovers a known
      power law, rejects degenerate input, scan lands on the true 2000 m contact.)*
      **NOTE (per the reference doc):** the PGS exponent (3.5) and GHE bins are literature/recall values —
      flagged in the module doc for verification before field release.
- [ ] **(8) increment 2 — first chunk (2026-07-22):** **Lucia Rock-Fabric Number** module
      (Petrophysics ▸ Rock Typing, carbonate) — inverts the Jennings-Lucia transform analytically for
      RFN + a 1–3 class; completes the FZI / Winland / PGS / Lucia rock-typing quartet. *(1 new test:
      Lucia round-trips RFN 1.0/3.0.)* **Try:** run it on a well with carbonate stringers. *(A Mahakam
      phi-k perm preset was built and tested but PULLED from the repo — those are proprietary Pertamina
      Hulu Mahakam production constants; kept out per the client-data rule.)*
- [ ] **(8) increment 2 — SHF forms (2026-07-22):** the **SHF Fit** pane got a form selector — besides
      Cuddy FOIL it now fits **Brooks-Corey** (Sw = Swirr + (1−Swirr)·(He/H)^λ, via a Swirr-grid + log-log
      linear fit) and **Skelt-Harrison** (Sw = 1 − A·exp(−(B/(H+D))^C), via a compact Nelder-Mead) to the
      log-derived Sw-vs-height cloud, with a Sw-vs-H scatter + fitted-curve overlay and a params/R² table.
      *(3 new tests: Brooks-Corey recovers a synthetic curve, Skelt reaches R²>0.98 + monotone Sw, both
      reject too-few points.)* **Try:** SHF Fit ▸ pick Brooks-Corey / Skelt-Harrison. *(Increment 2
      remainder — Thomeer Pc fit, SCAL importers, Pittman full rX table, and Ward/histogram HFU
      clustering — is now all shipped; see the entries below. Task #158 is complete.)*
- [ ] **(8) increment 2 — electrofacies tie-in (2026-07-22):** two parts. **Rock Type from Cutoffs**
      module (Petrophysics ▸ Rock Typing) — a Vsh + PHIE cutoff ladder → **RT_LOG** (1 best / 2 moderate
      / 3 non-net), to propagate rock types to uncored intervals. **Facies Tie-in** pane (workspace ▸
      *Facies Tie-in (RT confusion)*) — cross-tabulates the predicted log RT against a reference/core RT
      curve across wells and reports the **confusion matrix + dominant-class purity** (the check that
      the log classification faithfully reproduces core rock types). *(3 new tests: the cutoff ladder
      classifies clean/moderate/shaly correctly, the confusion tally scores purity, empty input is
      rejected.)* **Try:** run `rt_cutoff` to make RT_LOG, then Facies Tie-in ▸ RT_LOG vs your core RT.
- [ ] **(8) increment 2 — SCAL importers (2026-07-22):** **Import SCAL…** (Data ▸ Import Data) now
      takes **multiple files** and **three formats** (or **Auto-detect** per file): the existing flat
      PC/SW CSV, the **porous-plate wide table** (Corelab-style: preamble junk tolerated, pressure
      columns 1…150 psi as headers, one row per plug with Sample/Depth/Perm/Poro, cells = brine Sw
      %PV — unpivoted to long Pc points), and **centrifuge per-plug blocks** (SAMPLE/DEPTH/PERM/PORO
      key-value lines then a Pc/Sw table; several blocks per file, or multi-select one file per plug —
      the digitized-workbook shape). All selected files land in ONE combined replace-write of the
      well's `scal_pc` rows, then the Leverett-J fit runs over the pooled points as before. Lettered
      plug ids ("12A", "S-16A") keep their numeric part; %PV and %-porosity auto-convert; a bad file
      fails the whole import (nothing partial) and names the file. Also fixed on the way: a `PORO`
      header now resolves as porosity in every core/SCAL CSV import (it previously matched no alias).
      *(6 new tests: wide-table unpivot incl. a missing cell, headerless-file rejection, two-block
      centrifuge parse with no metadata leak between plugs, table-less block rejection, the format
      sniffer on all three shapes, multi-file import + replace-not-append + bad-file atomicity.)*
      **Try:** Import SCAL… ▸ multi-select your W-MND-1 porous-plate/centrifuge CSV exports ▸
      Auto-detect ▸ Import & Fit; then re-import to confirm points replace, and check SHF Fit sees
      the pooled cloud.
      **Post-review hardening (same day, ultracode 3-lens adversarial review — 10 confirmed
      findings, all fixed):** (1) an import that parses ZERO points now refuses the replace-write
      instead of silently wiping the well's existing SCAL data; (2) auto-detect no longer misroutes
      files whose cover sheets contain "No. of Samples,6"/"Sample Type,plug" lines — the centrifuge
      verdict now needs corroboration (a numeric DEPTH/PERM/PORO key-value line or a bare PC/SW
      header); (3) merged centrifuge files where the table header appears only above the first plug
      no longer silently drop plugs 2..N (header carries over); (4) repeated per-page header rows
      and numeric "Average" footers in wide tables no longer import as phantom Sw points (a data
      row must carry a sample id or depth); (5) regional Excel formats parse: ';' list separator
      (sniffed from line 1) and ',' decimals/thousands ("2,695.3", "98,5", "1,000"); (6) the flat
      parser keeps lettered plug ids ("12A"→12) like the other two. The dialog also now warns: ONE
      lab fluid system per import (mixed air-brine + mercury multi-selects would bias the pooled
      J-fit). *(+7 tests, suite 211 passed / 0 failed, tsc EXIT 0.)* **Deferred to the Thomeer /
      J-from-SCAL chunk:** a per-row fluid-system/IFT column in `scal_pc` (schema migration) so
      mixed-system imports can be stored and standardized properly, per the reference doc's long-
      table spec. *(→ delivered same day, see the Thomeer entry below.)*
- [ ] **(8) increment 2 — Thomeer Pc fit (2026-07-22):** new **Pc Fit (Thomeer)** pane (workspace ▸
      add pane). Fits the Thomeer (1960) hyperbola **Bv = Bv∞·exp(−G/log₁₀(Pc/Pd))** per plug over
      the scoped wells' imported SCAL Pc points (Bv = φ·(1−Sw); poro-less plugs are skipped and
      counted, not silently dropped). Per-plug table (row click selects) + the **Bv-vs-Pc QC plot**
      with the fitted hyperbola and Pd marker + the **Pd–G plane** — the Thomeer-class rock-typing
      crossplot. Also reports the Swanson apex (Bv/Pc)max and **Swanson k = 399·(Bv%/Pc)^1.691**
      (constants flagged: verify vs Swanson 1981 before field release, same policy as PGS). ONE
      pore system per plug this increment; multi-modal stacking (2–3 systems, dBv/dlogPc detection)
      is a later increment. **Schema:** `scal_pc` gained per-row **`system` + `ift`** columns
      (ALTER-migrated on old projects; the deferred review item) — the Import SCAL dialog now has a
      **Fluid system** select (air-brine 72 / air-mercury 367 / oil-brine 26 / custom) that
      auto-fills the sigma·cosθ and stamps every stored point. *(3 new tests: synthetic-hyperbola
      recovery pd/G/Bv∞ + R²>0.98, too-few/uninvaded rejection, DB-level grouping + poro-less skip
      + system/ift round-trip.)* **Try:** import MICP as
      Air-mercury ▸ Pc Fit (Thomeer) ▸ Fit — check the Pd–G clusters against your rock types.
      **Post-review hardening (same day, ultracode 2-lens review — 7 confirmed findings, all
      fixed):** (1) **Pc now standardizes to Hg-air equivalent (×367/σcosθ) BEFORE fitting** — the
      review caught Swanson k being applied to raw air-brine/oil-brine Pc (16–88× inflation) and
      the Pd–G plane mixing lab systems; G is scale-invariant so only Pd/apex move, and plugs from
      any system now share one comparable plane. Rows without a recorded σcosθ fit raw, show
      "(raw)" in the new System column, and get NO Swanson k. (2) Plugs group per **well_id** (two
      same-named wells no longer pool) and numbered plugs key on the sample number alone (blank
      depth cells no longer split a plug). (3) The long parser **forward-fills merged-cell plug
      context** (sample/depth/perm/poro on first row only — the common Excel export shape). (4)
      Entry-truncated curves flag **Pd ⚠ (pinned at a search bound)** instead of posing as resolved
      entries; plateau-only data no longer reports R²=0 for a perfect constant fit. (5) "Other"
      fluid system clears the σcosθ field (no stale preset silently stored). (6) perm/swanson_k
      typed `number | null` (NaN→null over IPC). *(+2 tests: air-brine plug recovers the same
      Hg-equivalent Pd as its mercury twin & legacy no-ift rows suppress Swanson; merged-cell
      forward-fill. Suite 216 passed / 0 failed; tsc EXIT 0.)*
- [ ] **(8) increment 2 — Pittman rX + HFU clustering (2026-07-22, closes task #158):** two pieces.
      **Pittman pore-throat radii** — new `pittman_rx` module (Petrophysics ▸ Rock Typing) writes the
      full **Pittman (1992) r10…r75** family (PR10…PR75 µm, each log₁₀ rX = C0 + C1·log₁₀ k + C2·log₁₀ φ%),
      an **APEX** selector (r10…r75, default r35) → **RAPEX** + its Hartmann-Beaumont **RT_PITT** port
      class. The r35 row (0.255/0.565/−0.523) matches the reference doc; the full table is transcribed
      from Pittman 1992 and flagged verify-before-release. **HFU Clustering** — new **HFU Clustering
      (FZI)** pane (workspace ▸ add pane). Reads the scoped wells' **core φ-k** (routine core analysis,
      not log curves), computes FZI, and partitions log₁₀(FZI) into K units by **Ward** (exact
      minimum-variance K-partition via DP — the global optimum, no greedy drift) or **histogram**
      (boundaries at the log-FZI histogram antimodes). Per-HFU table (FZI min/max, geometric-mean FZI,
      φ mean, and the Amaefule perm-transform R²) + the **RQI–φz** unit-slope crossplot coloured by HFU
      + the **log₁₀ FZI histogram** with the cut lines; row click highlights a unit. Read-only (writes
      no curves). *(10 new tests: Pittman r35 vs the published regression, apex-selector switching, Ward
      DP splits two separated bands + recovers each k, histogram finds the bimodal valley, invalid-plug
      skip + distinct-level cap note, empty-input error.)* **Try:** run `pittman_rx` (pick APEX) for the
      radius family; then HFU Clustering (FZI) ▸ pick Ward or Histogram + K ▸ Cluster — check the RQI–φz
      unit-slope lines and the FZI histogram breaks against your rock types.
      **Post-review hardening (same day, ultracode 4-lens adversarial review — 6 confirmed findings, all
      fixed; 2 refuted correctly):** (1) the **histogram path could emit an empty interior HFU**
      (two valleys flanking an empty bin gap) → non-contiguous ids like {1,3} and a boundaries/clusters
      count mismatch; ids are now remapped to contiguous 1..K and boundaries are recomputed from the
      final assignment (one cut per populated pair) for BOTH methods. (2) the selected-row highlight
      (`ml-diag`) was a no-op outside `.ml-confusion` tables → CSS broadened to cover plain mc-table
      selection rows (also repairs the Thomeer pane's identical latent no-op). (3) FZI_gm unit-slope
      lines now **clip to the plot rectangle** (a line whose slope-1 extension overshot could paint over
      the axis label/frame). (4) the pane now **redraws its canvases on resize** (was stale/blurry until
      a row click). (5) frontend histogram bins aligned to the backend clamp (8–40) so bars and cut
      lines share resolution. *(+1 regression test locking HFU-id contiguity across an empty gap. Suite
      227 passed / 0 failed; tsc EXIT 0.)*
- [ ] **Correctness — RT ≤ 0 → +Infinity in the Sw modules (2026-07-22, closes AUDIT-2026-07-21):**
      the three deterministic saturation modules (`sw_arch`, `sw_indo`, `sw_sim`) only screened
      **missing** RT (NaN). A genuine RT value **≤ 0** — almost always a null coded as `0`, or a bad
      processing artifact — flowed through: `sw_arch`'s `(a·Rw/(φ^m·RT))^(1/n)` and `sw_indo`'s
      `1/(RT·…)` both **diverged to +Infinity**, and since the "missing" test is NaN-only, +Inf leaked
      into the *unlimited* raw curves (`SWT_ARCH` / `SWE_INDO`) and **poisoned catalog min/max and plot
      autoscale** (the *limited* SWT/SWE looked fine because `limit()` clamps +Inf → 1.0, which masked
      it). `sw_sim` instead let the Newton-Raphson solver diverge and silently drop the sample. **Fix:**
      added `r <= 0.0` to each module's input guard, so an RT ≤ 0 sample is dropped to **missing (NaN)** —
      exactly matching the existing convention already used by `sw_rtc` / `sw_imts` (LRLC modules) which
      guard `rt_i <= 0.0`. *(Proven complete: an f32-sourced RT can't overflow f64 even at the smallest
      positive value, so no tiny-positive-RT can sneak a +Inf through; the LAS null −999.25 is negative
      → caught. Downstream contract verified safe — `classify_sample` already treats a missing SWE as
      "exclude from PAY", so a garbage RT that used to read as a fabricated `Sw=1.0` water sample now
      simply drops out; net pay is unchanged and average-SWE-over-reservoir is if anything cleaner.)*
      **Verification:** +3 regression tests (RT = 0 *and* −5 → NaN, never ±Inf, in all three modules);
      **suite 230 passed / 0 failed / 7 ignored**. Ran a 3-lens adversarial review (physics / downstream
      contract / edge-cases, 2 skeptics per finding, static-read only) → **0 confirmed, 7 refuted**.
      Two accurate-but-inconsequential observations were recorded, not fixed: *(i)* for the
      doubly-degenerate `(PHIE<0.005 AND RT≤0)` sample the porosity-state branch order makes `sw_arch`→NaN
      but `sw_indo`/`sw_sim`→SWE=1.0 (a non-reservoir sample excluded from pay either way; unifying it
      would mean restructuring `sw_arch`'s tested branch for zero benefit); *(ii)* `resolve_rw` could
      emit +Inf only at FTEMP = *exactly* −21.5 °C in the non-default MEASURED/SALINITY mode
      (physically impossible, pre-existing, orthogonal to this fix). **Try:** load a well whose deep
      resistivity has a zero/null streak and run `sw_arch` — the streak now reads as a gap in `SWT_ARCH`
      instead of pinning the curve autoscale to a huge number.
- [ ] **AUDIT-2026-07-21 full-QC triage — backend robustness batch 1 (2026-07-22):** a 65-finding
      parallel QC audit was triaged against current code (3 already fixed incl. the RT≤0 one above; 51
      safe-to-fix; 6 need your sign-off; 1 needs a live 100-well run; 4 feature-work). **This batch = 12
      safe backend fixes, none of which change any valid interpretation value** (suite 236/0/7):
      **(1)** `vsh_dn` now skips a **degenerate matrix/shale/fluid triangle** (`|c−d|<1e-6`) instead of
      writing ±Infinity into the unlimited VSH_DN (was poisoning catalog min/max + autoscale, same class
      as the RT≤0 bug). **(2)** `ftemp_grad` BHT mode skips a **TD_BHT ≤ 0** zone override (was a
      finite-looking ±Inf FTEMP). **(3)** `perm_wyllie_rose` now skips **negative PHIE** uniformly — the
      integer MORRIS_BIGGS/TIXIER exponent used to fabricate a plausible PERM from it while TIMUR NaN'd it.
      **(4)** `perm_transform` emits **MISSING instead of +Infinity** when `10^(PT_A·φ+PT_B)` overflows the
      f32 cast (reachable at in-range PT_A=100/PT_B=5). **(5)** `nphi_env_corr`'s FTEMP is now a
      **computed-only** input (a raw degF FTEMP can no longer be silently applied as degC), matching
      gascorr. **(6)** SandiMin **output prefix is upper-cased** so a re-cased prefix can't leave a stale
      curve. **(7)** the four computed-curve **delete-then-append writers now DELETE case-insensitively**
      (`upper(curve_name)`), closing the root-cause shadow-row bug where a re-cased equation output left a
      duplicate row that could silently win; the log-set restore subquery too. **(8)** curve-edit
      `locate_curve` got a deterministic `ORDER BY`. **(9)** **LAS export** looks up columns by upper-cased
      name, so a mixed-case computed curve ("Vsh_final") exports its real values instead of an all-NULL
      column. **(10)** Monte Carlo `summarize()` returns **NaN (→ "—")** for a dry/no-data metric instead
      of a fabricated 0.00. **(11)** the IMTS method doc's clay-term formula fixed to divide by Sw (matches
      code). *(+6 new tests locking the guards. No TS changed, so tsc unaffected.)*
- [ ] **AUDIT-2026-07-21 — import-robustness batch 2 (2026-07-22):** five importer fixes so one bad row no
      longer aborts a whole import, all mirroring existing verified patterns (LAS `depth_keep_indices`
      sanitize + the locations importer). **(1)** Core-CSV import **dedups duplicate plug depths** (first
      kept) instead of aborting the well's core import on the `core_data (well_id, depth)` PK. **(2)**
      Deviation-survey import **dedups duplicate station MDs**. **(3)** DLIS import **sanitizes each frame's
      depth** (drops non-finite + dedups) so one bad sample can't abort the file. **(4)** Tops import is now
      **transaction-wrapped** like the sibling Locations importer — a mid-file error no longer strands half
      the tops. **(5)** Tops import now **skips a blank WELL cell in a multi-well file** (was misrouting it
      to the selected well, silently attaching a top to an unrelated well) and reports the dropped count.
      *(+2 tests updated for the new `has_well_column` flag; suite 236/0.)*
- [ ] **AUDIT-2026-07-21 — dead-code removal batch 3 (2026-07-22):** deleted two dead source files and
      their IPC surface. **(1)** `petrophysics.rs` was fully dead (never declared as a `mod`, zero
      references; its math — linear Vsh, density porosity, plain Archie — is long since live in
      `modules.rs`). **(2)** `inversion.rs` was a hardcoded-stub solver (`run_stochastic_inversion`
      returned a fixed `[0.25,0.15,0.20,0.40]` regardless of input) still exposed over IPC as
      `start_inversion`/`get_inversion_status` with **zero frontend callers** and a latent
      `tokio::spawn`-from-sync-command panic; removed both commands from the handler, the
      `.manage(inversion::new_registry())`, the `mod`, and the file. *(No behavior change — nothing
      called either. Suite 236/0.)*

## Round 2 — panes, shift-select, MC plot props + table + polish (2026-07-21, Jauhar feedback batch #2)

Follow-up batch after the first round: (1) Shift-select was painting a native blue text
highlight; (2) the "4 main panes" clarification — they should always **STAY** (never vanish when
other panes pop/close) but stay manually resizable; (3) MC + other UI polish toward the **Cutoff
Sensitivity** panel look (image 3); (4) MC — add **plot property panels** (resize, colour, axes)
for the histogram + tornado, and make the histogram look like a **real histogram**; (5) MC — move
the **results table to the very bottom**. **tsc EXIT 0; browser-verified on an isolated vite (port
1428, never touched your 1420). Nothing committed.**

- [ ] **Shift-select no longer turns blue.** Range-select (Shift-click) was triggering the browser's
      native text selection across the well labels. Added `user-select: none` to the tree nodes and
      both tree bodies (Wells + Tops). *(Verified: `.tree-node` computes `user-select: none`.)*
- [ ] **The 4 anchor panes now STAY.** Wells / Tops / Processing / Performance can no longer be closed
      — the ✕ is hidden on their window header, Close panel/Close window are dropped from their
      right-click menu, and they can't be floated out of the sidebar. So opening/closing other windows
      can never make them disappear. They remain **freely resizable** (drag the splitter; the
      minimum-width floor only stops full collapse). A restored old layout that had lost the Wells pane
      re-adds it. *(If you'd rather they could still be closed, say so.)*
- [ ] **The anchor panes keep their WIDTH when other panes/windows pop up or close.** dockview lays out
      proportionally (that option is hardcoded on and not exposed), so opening/closing a pane was
      reflowing the sidebar. The fix pins each anchor group to a **fixed width (min == max)**, which
      dockview excludes from redistribution entirely — so no add, close, or window resize can move it.
      You can still resize it: grabbing the splitter (`.dv-sash`, caught in the capture phase so the
      drag goes live) unlocks the anchors for the drag, and they're re-pinned at the new width on
      release. *(Two earlier heuristic attempts — restore-on-layout-change — held on close but not on
      add, because an add fires extra reflow passes. This fixed-width approach needs no heuristic.
      Verified end-to-end against the real dockview build in isolation: add 4 panes → held 260; close 2
      → held 260; real DOM sash-drag → 340; add 3 more → held 340.)*
- [ ] **MC results table is at the very bottom.** Order is now **histogram → tornado → table**.
      Click a table row to plot that well-zone's HPV distribution in the histogram above.
      *(Browser-verified: the three result blocks render in that DOM order, table last.)*
- [ ] **Histogram is a real histogram now.** Added a frequency **y-axis** (nice-stepped count ticks
      0/20/40/… with a "count" title), horizontal **gridlines**, x-axis HPV min/mid/max labels, and the
      P10/P50/P90 markers. *(Browser-verified by capturing the canvas draw calls: count ticks + "count"
      + "HPV" + P10/P50/P90 all drawn; canvas re-rasterises crisply on resize.)*
- [ ] **⚙ Plot properties on both plots.** A gear on the histogram and the tornado opens an inline
      panel: **Height (resize)**, **colour** (bar colour / low-side + high-side bar colours), and
      toggles — histogram: P-markers, gridlines, y-axis; tornado: row stripes, ρ labels. Height 0 on the
      tornado = auto-size to the parameter count. *(Browser-verified: height 220→320 px live; bar colour
      set to #1f77d0 and the sampled bar pixel read back rgb(31,119,208).)*
- [ ] **MC UI polished toward the Cutoff panel.** Full-width brown **Run** button (matches Compute),
      `form-control`-styled selects/inputs, and tidier uncertainty-parameter rows (flexible param name,
      compact distribution pill). *(Browser-verified: Run button is full-width with the accent
      background.)*
- [ ] **Rw-for-PHIE gating still holds** after the tornado rewrite. *(Re-verified by capturing drawn
      labels: RW is drawn for HPV — it drives HPV via Sw — and dropped for Avg PHIE.)*

## Pane layout + MC/workflow polish + well-scope selector (2026-07-21, Jauhar feedback batch)

Jauhar's batch: (1) panes — two "Wells", tops-in-wells, non-resizable anchors; (2) MC — polish,
percentiles, table, ugly/stretching plots, and Rw showing sensitivity for PHIE it doesn't affect;
(3) workflow polish; (4) cross-cutting: stop checklisting wells one-by-one — use groups + pins.
**tsc EXIT 0; Rust montecarlo suite 7/7 (1 new: configurable percentiles); cargo check EXIT 0;
browser-verified on an isolated vite (port 1428, never touched your 1420) — see the proofs noted
per item. Nothing committed.**

### Panes
- [ ] **No more "two Wells".** The wells pane had a static "WELLS" title *and* the ObjectTree's own
      "Wells (N)" header — plus a **concurrent-refresh race** that appended the header (and every well)
      **twice**. Fixed both: dropped the static title; added a generation guard to `ObjectTree.refresh`.
      *(Browser-verified: 1 header, 9 well nodes — not 18 — for a 9-well group.)*
- [ ] **Tops is its own pane now.** Split out of the combined "Wells & Tops": a standalone **Tops** dock
      panel that follows the selected well through app state, docked directly below the **Wells** pane.
      It's a real dockview panel — drag it anywhere, tab it, resize it. *(Verified: panel list shows
      separate "Wells" and "Tops".)* Old saved layouts get the Tops pane auto-added on open.
- [ ] **Sidebar panes are resizable.** The Wells / Tops / Processing / Performance anchors were locked
      at a fixed width (min == max). Now they have a **minimum-width floor only** — drag the splitter to
      any width; they still won't collapse or auto-stretch when a neighbour closes. *(This reverses the
      earlier fixed-width lock, per your request — tell me if you preferred fixed.)*
- [ ] **★ pin a well** in the Wells pane (the star to the left of each name; persisted per project).

### Well scope — no more well-by-well checklists (imagine 2000 wells)
- [ ] Every run dialog (**Monte Carlo, Workflow, every module pane, Multimin, ML-apply, Cutoff,
      Summary, Report-batch**) now shows one compact **scope selector** instead of a checkbox per well:
      **Group** (defaults to the active group) · **★ Pinned** · **Selection** (your Ctrl-click set) ·
      **All** · **Custom…** (a searchable checklist for the rare precise pick), with a live "N wells"
      count. *(Verified: defaults to the active group and resolves 9 wells.)*
- [ ] Groups already existed and already scoped dialogs — the gap was purely the UI. **Pinned wells are
      new** (a `well_pins` table + ★ toggle) since a reusable pin-subset didn't exist before (the old 📌
      is only the workspace-follow toggle, unchanged). ML's *Train wells* and Auto-correlation's *targets*
      are deliberately **not** scope-swapped (they're a different concept, not "run on N wells").

### Monte Carlo
- [ ] **Rw no longer shows sensitivity for PHIE.** This was **not** a calculation bug — Rw is correctly
      routed only to the saturation step, so the PHIE *curve* is independent of it. The tornado was
      rendering statistically-insignificant **noise** (finite-N Spearman ≈0.05) and zero-width OAT rows.
      Fixed at the display layer, principled: a parameter appears for a metric **only if its one-at-a-time
      sweep actually moves that metric** (the sweep is deterministic → a non-contributor moves it by
      exactly 0), and ρ labels show **only above the significance floor** (1.96/√N). *(Browser-verified by
      capturing the canvas text: the tornado draws Rw for **HPV** — it does drive HPV via Sw — but **drops
      Rw for Avg PHIE**, while GR_SH/RHO_MA/NPHI_SH/GR_MA remain.)*
- [ ] **Percentile option.** A **Percentiles** dropdown in Settings (P10/P90 default, P25/P75, P5/P95,
      P1/P99) drives both the reported spread **and** the tornado's input sweep. *(Verified: switching to
      P5/P95 re-labels the table columns and the histogram markers.)*
- [ ] **Tidier table.** P50 as the headline number with the (P10–P90) band on a quiet sub-line, a new
      **Gross** column, tabular figures, zebra rows, and dynamic Pxx headers.
- [ ] **Plots don't stretch on pane resize any more.** Both the histogram and tornado canvases now
      re-rasterize to the pane's width via a ResizeObserver (before, the browser scaled a stale bitmap →
      the blur/stretch you saw). *(Verified: shrinking the pane redrew the bitmaps 618→484 px.)* Tornado
      also got rounded bars, alternating row shading, and a height that tracks the parameter count.

### Workflow
- [ ] Same scope selector replaces the well checklist; the rest of the builder (steps, grid, cons in/out)
      is unchanged.

## Monte Carlo parameter sensitivity + tornado (Wave B #13, 2026-07-21)

The uncertainty engine already ran N realizations but **threw away the parameter draws** — it only
kept the resulting P10/P50/P90. It now retains them and reports **which parameters actually drive
the result**. **tsc EXIT 0; Rust montecarlo suite 6/6 pass (3 new); off-by-default so existing runs
are byte-identical.** Nothing committed yet.

- [ ] **Open Monte Carlo** (Advance ribbon → Monte Carlo). There are two new checkboxes under a
      **Sensitivity** row — *Rank sensitivity (Spearman)* and *Tornado sweep (P10 / P90)*, both on by
      default. Add one or two uncertain parameters (e.g. GR_MA, GR_SH, RW), pick a well, **Run**.
- [ ] **Tornado chart** appears below the HPV histogram with a **Zone** and **Metric** selector
      (HPV / Net pay / NTG / Avg PHIE / Avg SWE). With the tornado box ticked it shows **range bars**:
      each parameter swept to its P10↔P90 with the others held at their medians, sorted most-influential
      on top, around a common **base** line, annotated with the Spearman ρ. Untick *Tornado* (leave
      *Rank sensitivity* on) → it falls back to **signed correlation bars** on a −1…+1 axis.
- [ ] **Sanity checks**: (a) the parameter you'd expect to matter most (usually GR_SH or Rw) sits at
      the top; (b) switching **Metric** re-sorts and re-scales; (c) switching **Zone** redraws for that
      well-zone; (d) a parameter you give **zero spread** (sd = 0) shows ρ = NaN / no bar (it can't be
      ranked); (e) unticking **both** boxes → no tornado section, and the headline P10/P50/P90 table is
      unchanged. Verified: Spearman sign+magnitude, tornado low≤base≤high ordering, and opt-out
      reproducibility are covered by unit tests; the live chart render awaits your click-through.

## Highlight tool + ribbon overflow + trademark scrub + typography (2026-07-21)

B2 UI/workflow polish + two follow-ups. **tsc EXIT 0; `cargo check --tests` EXIT 0; Rust 177 pass / 0 fail.** Nothing committed yet.

- [ ] **Ribbon overflow chevrons (Office-style)** (ribbon.ts, styles.css). When the window is too narrow
      to show all the tools on a tab, the raw scrollbar is gone — a boxed **‹ / ›** appears at the
      overflowing edge and scrolls the group row a page at a time (like PowerPoint's ribbon). Test: narrow
      the window until a tab's groups don't all fit → a **›** box appears at the right edge; click it →
      the row scrolls and a **‹** appears at the left; at the end only **‹** shows. Switch tabs / resize →
      the chevrons re-evaluate. (Verified live: at 720px the Petrophysics row overflows 238px, right
      chevron shows at scroll-start, left appears after scrolling, correct box at the right edge.)

- [ ] **Highlight tool — colored depth bands in the Log View** (new `highlightsOverlay.ts`; `highlights`
      table + `list/upsert/delete_highlight` in db.rs/lib.rs; IPC in ipc.ts). Open a **Log View**, then
      in that view's toolbar click **🖍** (next to the 🏷 tops button). Drag vertically over a depth
      interval → a **translucent colored band** appears across the tracks and an **Edit highlight**
      dialog opens. Give it a label (e.g. "Pay") + color → **Save**. Add a couple more with different
      colors. Test: (a) bands render across all tracks, translucent so curves read through; (b) they
      **track pan/zoom**; (c) switch to another well and back → bands **persist** and reload; (d)
      **double-click** a band → dialog to recolor / relabel / edit top+bottom / **Delete** / **Convert
      to zone**; (e) **Convert to zone** creates a zone (check it appears in **Zones** / pay summary);
      (f) **Ctrl+Z** undoes add / edit / delete / convert; (g) **🖍 and 🏷 are mutually exclusive** —
      turning one on turns the other off. Bands sit **below** the tops lines so tops stay legible.
- [ ] **Text sharpness — font hinting** (tauri.conf.json `additionalBrowserArgs`). You flagged text as
      slightly fuzzy/washed-out. I confirmed the CSS is clean and contrast is already high (~12.9:1), so
      it's not a color issue — the softness is Chromium's GPU grayscale AA (WebGPU forces GPU on) plus
      Windows display scaling. I added `--font-render-hinting=medium`. **This only takes effect on a full
      relaunch** (`npm run tauri dev` restart). Test: relaunch, eyeball the panel text vs before. If it's
      still soft, check **Windows Settings ▸ Display ▸ Scale** — at 125%/150% the webview raster-scales;
      tell me and we can add a text-size control or bump the base font. (Not verifiable from my side —
      the browser tools can't reproduce WebView2 rendering.)
- [ ] **AspenTech trademark scrubbed repo-wide (keeping Loglan)** (per your request — "except loglan").
      The prior-tool name is now gone from the whole tree — shipped app, code comments, and dev docs —
      except: **Loglan / `.lls`** (kept deliberately: SandiBumi runs Loglan, so those stay), your real
      data-folder paths in test fixtures (can't rename your disk), the English word "geology", and your
      own verbatim words in `Review.txt`. The comment/doc pass replaced the vendor name with neutral
      wording ("the reference suite", "commercial suite", etc.). Nothing to click-test — grep the repo
      for the old name and you'll only find the exceptions above. Test the one user-visible change: hover
      the **DB Inspector** ribbon button + open **Help** → reads "spreadsheet-style".

## 540-well test — perf & crash fixes (2026-07-21)

From your ~540-well stress test. A read-only 5-agent diagnosis traced every "not responding"
freeze to one root cause (heavy commands run **synchronously on the UI thread**) plus a specific
speed bug per subsystem. **Rust 176 pass / 0 fail; tsc EXIT 0. Nothing committed.** The async
piece can't be verified without running the app, so these especially want your click-through.

- [ ] **Field Dashboard no longer crashes on ~540 wells** (dashboardPanel.ts, summaryDialog.ts).
      "Compute failed: TypeError: Cannot read properties of null (reading 'toFixed')" was a zero-net
      zone row whose avg VSH/PHIE/SWE come back as NaN → serde encodes non-finite floats as JSON
      **null**, and the old `Number.isNaN` guard doesn't catch null. The formatter now shows "—" for
      null/NaN. Test: **Field Dashboard ▸ Compute** across all wells → the grid renders (empty
      aggregates show "—"), no crash. Same latent fix in the single-well **Cutoffs & Summary** table.
- [ ] **Field Dashboard is fast now** (workflow.rs `stats_only`). Compute took >5 min because it
      secretly wrote 3 FLAG_* curves per well (~1,600 DB transactions) on every press, though the
      panel only reads the returned numbers. It now computes the stats without persisting anything.
      Test: Compute on all wells → **seconds, not minutes**; tweak a cutoff and re-Compute → still
      fast. Behavior change to note: the dashboard no longer leaves FLAG_* curves in the wells —
      persist those from **Cutoffs & Pay Summary** (unchanged) when you actually want them written.
      Test `pay_summary_stats_only_persists_nothing`.
- [ ] **Workflow chain runs without freezing the app + live progress works** (lib.rs — `DbState` is
      now `Arc<Mutex<Connection>>`; `run_workflow_chain` now runs on a background thread). Build a
      chain (e.g. vsh_gr → phi_dn → sw_indo) and run it on a batch of wells: (a) the window stays
      **draggable/responsive** during the run (was frozen "not responding"), (b) the **progress bar
      advances** step-by-step, and (c) **Cancel** actually stops it. This is the first of the async
      conversions — import, dashboard, multimin, Monte Carlo and equations will follow the *same*
      pattern, so confirming this one works validates the whole approach.
- [ ] **A chain of many wells now finishes in seconds, and Cancel is near-instant**
      (workflow.rs two-phase batched write; equations.rs `create_log_sets_batch` +
      `write_computed_curves_versioned_batch`; chain.rs). The 30-min chain / 30-min-to-cancel
      was ~2 fsync-bound DB transactions **per well** per step (≈1,000 commits on 500 wells). Each
      step now computes every well in parallel (reads only), then does **one** batched versioned
      write — ~2 commits per step. Cancel is checked per well, so it drains in a well or two.
      Test: run a chain (vsh_gr → phi_dn → sw_indo) on a big well set → **seconds**, and **Cancel
      stops almost immediately**. Test `batched_module_run_writes_every_well_correctly` proves two
      wells write distinct, un-crossed, correctly-versioned results.
- [ ] **Cancel empties the progress bar** (workflowDialog.ts). Pressing **Cancel** now clears the
      bar to empty (and hides it) the instant you click, then the status confirms "Cancelled at
      step N". Test: start a chain, hit Cancel → the bar goes empty right away.
- [ ] **Input/Output are now "cons" (constellation) pickers, not free text** (workflowDialog.ts,
      moduleDialog.ts, new `list_log_set_names` command). Terminology changed from "set" to **cons**
      throughout the UI (Workflow, module dialogs, Curve Catalog "Constellations"). **Input cons** is
      a strict dropdown of existing constellations (blank = latest values — you can only read from
      one that exists). **Output cons** is an editable combobox: pick an existing constellation *or*
      type a brand-new name. Both are filled from the project's real constellation names. Test: open
      Workflow / any module → Input cons lists your existing constellations; Output cons suggests
      them but also accepts a new name like `FINAL2`.
- [ ] **Universal Processing panel — live per-well progress + Cancel for the whole run**
      (new `jobs.rs` registry + `list_jobs`/`cancel_job`; `processingPanel.ts`; ribbon **Processing**
      button). New dock panel that shows, for a running workflow chain: a **progress bar with an
      integrated Cancel**, the current **"Step 2/3: sw_indo"** line, a live **counts row**
      (▶ running · ✓ done · ⚠ warned · ✗ failed · ⏳ pending), and a **details** toggle listing the
      *notable* wells (running/warned/failed) with messages — so you can see **which well failed and
      why** at 500-well scale without a 500-row dump. It **auto-opens** when you press Run in the
      Workflow Builder, or open it anytime from the **Processing** ribbon button. Cancel here shares
      the *same* flag as the run, so it stops the chain whether launched from the panel or the
      builder. This is the reusable spine: import, module runs, multimin, Monte Carlo and reports
      will each report into it as they move off the IPC thread. Test: Run a chain → the Processing
      panel opens and fills live; click a well's ⚠/✗ in details to read the message; hit Cancel →
      the bar stops within a well or two.
- [ ] **Processing panel: the step-boundary "pause" now says what it's doing** (workflow.rs).
      Each chain step computes every well (bar fills), then does ONE big batched DB write with no
      per-well signal — so the bar used to sit at the boundary / 100% looking frozen. It now shows
      **"Writing N well(s)…"** during that write, so the wait reads as working, not stuck. Test: run
      a chain and watch between steps and at the end → the current line reads "Writing … well(s)"
      during the pause, then advances/completes.
- [ ] **Workflow Builder no longer shows its own redundant progress bar** (workflowDialog.ts). The
      inline `<progress>` bar is gone now that the Processing panel owns the live bar + Cancel; the
      builder keeps a one-line status ("Step 2/2: … — see Processing panel", "Done: …"). Test: run a
      chain → progress shows only in the Processing panel; the builder just shows a status line.
- [ ] **Hardware Health Monitor** (new `health.rs` + `health_snapshot` command; `healthPanel.ts`;
      ribbon **Health** button). A Petrel-PHM-style panel of four colour-coded gauges — **MEM System**
      (system memory %), **GPU Memory** (GPU video-memory current/budget %), **USER Objects** and
      **GDI Objects** (this process's handle counts vs the 10,000-per-process ceiling — the classic
      desktop-leak signal, raw count shown in the value). **Green < 60% · Yellow 60–80% · Red > 80%**,
      polled every 1.5 s. Open from the **Health** ribbon button (next to Processing). Metrics are
      Windows-only; any unavailable value shows **n/a** (so if GPU Memory reads "n/a" on your machine,
      tell me — the DXGI path is best-effort and I'll adjust). Note: this is GPU *memory* load, not
      engine-utilisation % (that needs PDH GPU counters — a possible refinement). Test: open Health →
      MEM/USER/GDI show live %; leave a few heavy panels open and watch GDI/USER climb.

## P0 senior-audit backlog — correctness & data-integrity fixes (2026-07-20)

The eight P0 findings from `AUDIT-2026-07-20.md` (ROADMAP §4b), plus the LAS-import
robustness residual (#118) that closing them surfaced, are implemented and unit-tested
(full lib suite **160 pass / 0 fail**, tsc clean). These are the ones that made answers
wrong or silently lost data, so they matter most for Mahakam work:

- [x] **MASK now blanks module INPUTS, not just outputs** (workflow.rs). Run **GR
      Normalize** (or **Log Predict**) with a BADHOLE / COND*FLAG curve set as the
      **Mask**. The well P3/P97 (and the KNN training set) are now computed from the
      unmasked samples only — casing/washout/hot-streak GR no longer shifts the two-point
      transform, so good-hole output stops drifting. For log_predict the repaired synthetic
      now survives \_inside* the masked (washout) interval it exists to fill, instead of
      being blanked there. Test: `mask_excludes_flagged_samples_from_gr_normalize_percentiles`.
- [x] **SW-height uses TVD and allows a subsea FWL** (satheight.rs). The sw_height module
      now takes an optional **TVD** input (defaults to measured depth when absent) and the
      **FWL** field accepts negative (subsea TVDSS) values. On a deviated well, height above
      the contact — and therefore SWH — is no longer optimistically overstated by ~1/cos(inc).
      Run on a deviated Mahakam well with the TVD curve mapped and confirm SWH rises vs the
      old MD-based result. Test: the negative-TVDSS deviated case in satheight.rs.
- [x] **Pay summary: thin-zone clamp + honest averages** (workflow.rs). Each sample's
      thickness is clamped to its overlap with the zone, so the last in-zone sample no longer
      bleeds a full step past the zone base and **net can never exceed gross** (sub-step-thick
      zones). SAND-row `avg_phie` is normalised over the thickness where PHIE is actually
      valid, so a sample with good VSH but missing PHIE no longer drags the average toward
      zero. Cross-check a well with thin zones / patchy PHIE against the old numbers. Test:
      `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`.
- [x] **Crash-safe curve writes** (db.rs `with_txn`). Every delete-then-append writer
      (computed curves, restore/delete log set, core/aux/SCAL/curve-sample/well-path inserts,
      group members, zones-from-tops) now runs inside a single BEGIN/COMMIT/ROLLBACK, so an
      app kill mid-write can no longer leave the DELETE committed but the re-append lost.
      Nothing to click — just note that a tauri-dev restart mid-run won't silently drop curves.
- [x] **IMTS clay-conductivity direction fixed** (lrlc.rs sw*imts). The excess-conductivity
      term now \_divides* by Sw (Waxman-Smits `Cw + B·Qv_eff/Sw`), so it grows as hydrocarbon
      displaces water instead of vanishing. IMTS SwE now sits at/just below Waxman-Smits in
      pay (the old `·Sw` form gave Sw^(n\*+1) and over-stated Sw exactly in the LRLC pay this
      method exists to find). Re-run sw_imts on an LRLC interval and confirm SwE dropped.
      Test: `imts_credits_clay_conductivity_in_pay_zone`.
- [ ] **DLIS null sentinels + no silent overwrite** (dlis.rs). DLIS absent/sentinel values
      (−999.25/−9999, non-finite, |v|>1e30) are screened to MISSING on import, and each DLIS
      frame gets its own run number so a frame-0 channel no longer silently replaces a
      same-mnemonic LAS curve — the status line reports "replaced N existing curve(s)" when a
      collision does happen. Re-import a DLIS over a well that already has LAS curves and check
      the replaced-count note.
- [ ] **SandiMin refuses under-determined models** (multimin2.rs). Selecting fewer than
      (components − 1) input tools is now rejected up front ("need at least N input logs to
      constrain M components") instead of solving to an arbitrary vertex, and per-sample the
      solver skips depths with too few live curves. Test: `rejects_underdetermined_request`.
- [ ] **LAS import survives duplicate/odd-depth files on BOTH stores** (parsers.rs, ingest.rs).
      Non-finite and duplicate depths are dropped (first occurrence kept) before insert, so a
      **spliced/merged LAS with a repeated depth section** imports instead of aborting on the
      `(well_id, depth)` PK — and the fix now covers the _generic_ store too (PEF/CALI/extra
      runs), which previously PK-failed silently and left the well without those curves. Extras:
      a Schlumberger **TDEP**-indexed (or any non-`DEPT`) file resolves depth via the first
      column; an auxiliary **MD/TDEP track** in a later column can no longer steal the depth
      role from the true first-column index; a file whose depth is entirely null now **errors
      cleanly** instead of creating an empty orphan well; and a non-monotonic depth (column 0
      wasn't really the index) is surfaced as an import warning. The dropped/duplicate/odd-index
      counts appear in the import status line and History. Import a re-spliced LAS and a TDEP
      file and confirm both load with the expected row counts. Tests:
      `duplicate_depth_las_imports_standard_and_generic_curves`,
      `all_null_depth_las_errors_without_creating_well`,
      `parse_las_2_auxiliary_md_curve_does_not_steal_depth`,
      `sanitize_dedups_signed_zero_depths`, `parse_las_2_tdep_index_populates_depth`.

## P1 — reliability (frontend state) (2026-07-20)

The three P1 reliability findings from ROADMAP §4b (stale plots, async races, listener
leaks). Frontend-only (TypeScript, `tsc` clean); these are async-lifecycle behaviors with
no unit tests, so they were hardened by an adversarial review (4 lenses → per-finding
skeptical verify: **6 confirmed / 0 refuted**, all fixed — including one real HIGH bug in
the first-pass init guard) plus a focused **second-pass verify of the fixes** (renderer-dispose

- sticky-reset lenses: **clean, 0 defects**). Click-through in `npm run tauri dev`:

* [ ] **Plots refresh after a module run, keeping their viewport** (histogramPanel,
      crossplotPanel, pickettPanel, correlationPanel, logViewPanel). Open a Histogram of
      PHIE (zoom in), then run a module/equation that recomputes PHIE (or import / undo —
      anything that bumps `dataVersion`). The plot now re-reads the new curve **in place**,
      preserving your current zoom/pan, instead of showing the pre-run curve until you close
      and reopen the panel. Each builder subscribes to `appState.dataVersion` and calls
      `reload(preserveView=true)`; a `dataPrimed` guard swallows the subscribe's immediate
      fire so nothing double-loads on open.
* [ ] **Fast well/curve/zone switching never shows stale data** (loadWell, reload,
      createPlot). Click quickly through 5 wells in the Log view, or spam curve/zone changes
      in the Crossplot/Pickett/Histogram. A slow earlier load can no longer land after and
      overwrite a newer selection — each async load captures a generation token before its
      first `await` and bails if superseded. And a viewport **reset** intent (switching wells
      / changing the curve) still fires exactly once even if a `preserveView` refresh commits
      first, via a sticky `resetPending`/`viewResetPending` flag — so you neither keep a stale
      zoom that should have reset nor lose a reset that a background refresh raced past.
* [ ] **Opening/closing Log panels doesn't leak listeners or GPU loops** (logViewPanel
      dispose, LogCanvasRenderer). Repeatedly open and close Log view panels. The renderer's
      `window` pointerup/pointermove handlers are now removed and its `requestAnimationFrame`
      loop cancelled on `dispose()` (previously leaked one set per open); disposing a panel
      _during_ WebGPU init now disposes the fully-initialized local renderer rather than a
      no-op on an already-nulled field. Nothing visible per-close, but memory/handler count
      stays flat over a long session.
* [ ] **Dialog Escape is scoped to the dialog — closes the P1 modal-Escape sliver** (modal.ts).
      The carried-over P1 sliver ("overlapping dialogs share one Escape handler"). The listener-
      leak half was already handled — `openModal` single-instances via `activeClose` (a new dialog
      closes the prior one and removes its keydown listener; no modal opens a nested modal, so
      there's no stack). The remaining gap: the dialog's Escape was a `document`-level handler that
      closed the dialog but didn't `stopPropagation`, so one Escape also bubbled to `window`/app-level
      Escape handlers. It now stops there — kept on the **bubble** phase so the numeric-edit guard's
      capture-phase `stopPropagation` still shields a dialog from closing while you edit a number
      field. Also tears down any in-flight title-bar drag listeners on close (no leak if a dialog is
      dismissed mid-drag). tsc clean. Test: start drawing a **map polygon**, open any dialog (⚙
      Properties, an import dialog…), press **Escape** → the dialog closes and the half-drawn polygon
      is **still there** (Escape no longer cancels it too); double-click a number field in a dialog,
      press Escape → you exit the field's edit mode and the **dialog stays open**; a second Escape
      closes the dialog.

## Polish — UX (veteran-interpreter friction) (2026-07-20)

Hardening-backlog **Polish** tier (ROADMAP §4b, was "P3"). Small, mostly-frontend fixes;
each tsc-clean. Mapped by a read-only investigation wave before implementing.

- [ ] **Cursor readout: real units + no more mangled values** (plotCommon.ts `formatValue`,
      viewerChrome.ts `renderReadout`, logViewPanel.ts). The log-view cursor readout used a
      blanket `toFixed(2)`, which flattened permeability 0.003 → "0.00" and showed no units.
      It now uses an adaptive significant-figures formatter (perm stays "0.003", RT reads
      "2151", φ "0.18") and appends the unit the curve catalog already carries (RT "ohm.m",
      PHI "v/v", RHOB "g/cc"). Units are cached per well-load and refresh on `dataVersion`.
      Test: hover the log view over a permeability track and a deep-resistivity track — values
      keep resolution and each shows its unit. (Values whose catalog unit is blank show no unit.)
- [ ] **Correlation: fresh well list + Ctrl+wheel zoom** (correlationPanel.ts). The Wells
      menu was built once and never refreshed, so a newly imported well never appeared. It now
      re-fetches the well list on `dataVersion` — new wells appear (and draw as strips), deleted
      wells drop out, active-group filter re-applies. Also added **Ctrl/Cmd+wheel zoom about the
      cursor** (same factors as the other plots); plain wheel still pans through depth. Test:
      open Correlation, import/deviation-load another well → it shows up without reopening the
      panel; Ctrl+wheel over the strips zooms at the cursor depth.
- [ ] **Processing history now covers every operation** (processLog.ts + call sites across
      ribbon / inspectorPanel / mlDialog / monteCarloDialog / workflowDialog / zonesDialog /
      topsEditor / mapPanel / cutoffDialog). The audit trail (History panel / QAT History)
      previously logged only LAS import/export, module runs, core shift, well header, curve edits,
      project/session, exports. It now ALSO records: **DLIS / deviation / SCAL / core / tops
      imports; equation runs; ML runs; Monte Carlo; workflow chains; log-set restore/delete; zone
      add/edit/delete + per-zone parameter overrides; manual tops add/edit/delete; cutoff-default
      saves; map-polygon→group assignment.** Test: perform each and confirm it appears in the
      History panel with the right `[kind]` and detail. (Batch/field-wide actions — equation, ML,
      MC, workflow, log-set, cutoffs — intentionally show no well name.)
- [ ] **Pickett v2 — properties dialog, typed M/Rw, configurable axes, Z-color** (pickettPanel.ts).
      The Pickett plot's RT/PHIE axes were hard-coded (0.1–1000 / 0.01–1) with no properties
      dialog. Now a **⚙ / right-click Properties** dialog sets RT & PHIE axis ranges, point size,
      and **color-by-curve** (rainbow/viridis, optional log-Z), persisted via plotprops. The
      toolbar gained **M and Rw** fields next to N — type them and the Sw=1 / iso-Sw lines follow;
      a two-point pick still fits the line and fills the same M/Rw fields (one shared source).
      Zoom/pan and the line survive a data refresh (P1 preserveView). Test: open Pickett on a well
      with RES_DEEP + computed PHIE; pick two points on the wet trend → M/Rw fill and lines draw;
      type a different Rw → lines follow; right-click → set RT axis 0.2–200 and color by SW/VSH →
      points recolor; reopen the panel → settings persist.
- [ ] **Pay-summary provenance — FLAG\_\* versioned + cutoffs recorded** (workflow.rs, backend).
      run*pay_summary wrote FLAG_SAND/RESERVOIR/PAY with the old in-place `write_computed_curve`
      — no version history, and the VSH/PHIE/SWE cutoffs that produced them were recorded nowhere.
      Now the explicit **Cutoffs & Pay Summary** run versions the three flags into a **PAYFLAG**
      log set whose provenance = module `pay_summary` + the cutoffs (in `log_sets.params_json`) +
      inputs, exactly like every other module output — so a re-run keeps history and any version
      is restorable/prunable from the Curve Catalog. The **Field Dashboard** and **report** passes
      set `skip_version` (field-wide QC side-effect) so they keep overwriting in place — no version
      churn per refresh. Test `pay_summary_versions_flags_with_cutoffs_in_provenance` (161 lib
      tests pass / 0 fail / 7 ignored; tsc clean). Click-through: run Cutoffs & Pay Summary on a
      well → Curve Catalog shows a PAYFLAG version whose provenance lists the cutoffs; re-run →
      version N+1; run the Field Dashboard → FLAG*\* update but no new version piles up.

## Performance (field-scale speed) (2026-07-20)

Hardening-backlog **Performance** tier (ROADMAP §4b, was "P2"). The rest of the tier (#128–132)
changes DB/IPC semantics and needs a live 100-well benchmark to sign off; this first item is the
one pure-frontend, low-risk win.

- [ ] **Crossplot: Z coloring memoized across pan/zoom/hover** (crossplotPanel.ts). Every crossplot
      redraw rebuilt the whole per-point color array from scratch — for a continuous Z that's **two
      `percentile` sorts** (each allocates + sorts a NaN-filtered copy of all samples) plus an
      N-length `colorRampEx` string array; for a discrete Z, an N-length `categoricalColors` map.
      That ran on **every** pan-drag / zoom-wheel / handle-drag `mousemove` and every synchronized-
      hover frame, even though the colors depend only on the Z data + colormap, never the viewport.
      The color computation is now a pure `computeCrossplotColors()` that the panel **memoizes**,
      keyed by (Z curve, colormap, log-Z, fixed color, data generation); pan/zoom/hover reuse the
      cached array and only a data or color-setting change recomputes. Output is pixel-identical —
      this is a speed change only, most visible on dense (100-well / full-field) clouds. tsc clean.
      Test: open a crossplot colored by a curve (e.g. NPHI-RHOB by GR, or a PERM Z with log-Z on),
      drag the parameter handle / Ctrl+wheel-zoom / pan / hover from a log view — motion stays
      smooth on a big cloud; the colors, color-bar range, and facies legend are unchanged; switching
      the Z curve, colormap, or log-Z toggle still recolors immediately; a module re-run (dataVersion)
      recolors against the new data.

## Low-tier correctness & data-integrity sweep (2026-07-21)

The 15 low-severity findings from `AUDIT-2026-07-20.md` (never adversarially verified at audit time)
were each re-checked against the current code by an independent per-finding verifier. One was
**already fixed** (SandiMin all-zero conductivity-row guard, `multimin2.rs`), two are **held for your
sign-off** because they change numeric output (Wyllie compaction correction; histogram re-bin), one is
**held to land with the depth-scale-ratio fix** (scale-dropdown staleness). The rest — the safe
correctness/crash/data-integrity fixes below — are applied. **Rust suite green; tsc clean.** Nothing
committed.

Backend (with new regression tests):

- [ ] **SSC `SWIRR_EFF` no longer 0 at a 100 %-shale point** (`ssc.rs`). At the wet-clay point effective
      porosity is floored to 0, and the `1 − φt·(1−SWIRR_T)/φie` divide gave `−inf→0` ("all water
      movable") or `0/0→NaN` — exactly backwards. Now a zero-effective-porosity sample reports
      `SWIRR_EFF = 1.0` (fully bound). _Only the degenerate φie==0 samples change; every producing
      point is unchanged._ (The deeper SWIRR_T/SWIRR_EFF ordering inconsistency is the separate held
      item.) Test: run SSC on a shale-heavy well; SWIRR_EFF in massive shale reads ~1, not 0.
- [ ] **Archie `SWT_ARCH` no longer writes `+Infinity`** (`modules.rs` `sw_arch`). A coal/tight sample
      with PHIT=0 but PHIE absent used to fall through to `a/0^m = +inf` and store it in the SWT_ARCH
      curve, poisoning catalog min/max and plot autoscale. The zero-porosity "all water" guard now
      keys on PHIT alone. Test (regression `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf`):
      the curve catalog's SWT_ARCH min/max stays finite over coal/tight zones.
- [ ] **Simandoux (SCHLUMBERGER) no longer divides by zero at VSH=1** (`modules.rs` `sw_sim`). Pure
      shale hit a `1/(1−VSH)` singularity and the sample was silently dropped; it now resolves to
      all-water (SWE=1), matching the low-porosity and Indonesia branches. Test:
      `sw_sim_schlumberger_pure_shale_is_all_water`.
- [ ] **LAS import fails loudly on a truncated row** (`parsers.rs`, both parsers). A physically short
      `~A` row used to shift every following value one column left silently (GR into RES, RHOB into
      DT) for the rest of the file. Leftover tokens at EOF now raise a clear import error instead of
      mis-columning. Test: import a LAS whose last data line is cut mid-row → you get an explicit
      "leftover token(s)…truncated or corrupt LAS?" error, not corrupted curves.
- [ ] **DB-inspector edit no longer reports success on a 0-row update** (`db.rs`, all three sample
      editors). If the matched depth had moved/been rewritten, the UPDATE hit 0 rows but the UI said
      "saved" and pushed a bogus undo entry. It now errors, and the inspector already reverts the cell + shows "Edit failed". Test (`…is_err()` assertion added): edit a sample, then edit a
      non-existent depth → "no … sample matched depth …", cell reverts, no phantom undo.
- [ ] **Well Header shows current TD / KB** (`db.rs` list_wells + `ipc.ts` + `ribbon.ts`). The dialog
      used to open with blank TD/KB, so you edited the datum blind — and KB silently drives TVDSS in
      deviation import. TD/KB are now carried on `WellSummary` and prefilled. Test: open Well Header on
      a well with a KB set → the field shows it, not an empty box.

Frontend:

- [ ] **Stats / regression reject `±Infinity`, not just NaN** (`plotCanvas.ts`: basicStats, linearFit,
      percentile, drawScatter/drawDiamonds). One inf sample (e.g. a Python `1/phi` at phi=0) used to
      make the histogram's Mean/Std chips read "Infinity" and silently kill a crossplot regression.
      Now non-finite values are skipped everywhere. Test: compute an equation that divides by a
      zero-porosity sample, then histogram/crossplot it — chips and the fit stay sane.
- [ ] **Zone-param "Set" button surfaces write failures** (`plotCommon.ts` pickRow — histogram Pick
      A/B, Pickett M/Rw). A rejected `setZoneParam` used to be swallowed while the status still said
      "set". It now shows "Failed to set …". Test: (hard to force by hand) — behaviour only differs on
      a backend write error; the success path is unchanged.
- [ ] **Duplicate track titles prevented** (`layoutPropsDialog.ts`). Renaming a track to an existing
      track's title collapsed both in every title-keyed lookup (weights, cursor hit-testing, core
      overlay, drag-drop). A colliding rename is now auto-suffixed ("RES 2"). Test: in Layout
      Properties, rename a track to another track's exact name → it becomes "name 2"; retyping a
      track's own name is a no-op.
- [ ] **Histogram: constant curves render; the `n` never silently disagrees** (`histogramPanel.ts`).
      A constant curve (flag/class curve, single-sample zone) used to show "No valid data"; it now
      draws one central bar. And when the P2–P98 axis window clips tail samples, the axis label reads
      `n = X of Y` so it no longer contradicts the stats chips (which count all samples). _(The full
      full-range re-bin — which would change every bar height — is the held item.)_ Test: histogram a
      constant/flag curve (draws), and a curve with fat tails (label shows "of").
- [ ] **Log-view smoothness** (`LogCanvasRenderer.ts`, speed only). The clear color is no longer read
      via `getComputedStyle` every rendered frame (cached, invalidated on theme change), and the
      cursor readout uses a binary search instead of scanning every sample per mouse-move. Values and
      colors are identical. Test: drag-pan a busy log view — motion is smoother; theme switch still
      repaints; the cursor readout still tracks correctly.

### Held-item resolutions (2026-07-21 — your call: 1 yes / 2 leave / 3 yes / 4 yes + Bahasa Jawa)

Your answers to the four held items above. **Rust suite 164 pass / 0 fail; tsc EXIT 0; the two
browser-observable pieces verified live in the vite preview.**

- [ ] **Wyllie lack-of-compaction (Cp) correction — shipped as opt-in** (`modules.rs` `phi_son`,
      `OPT_CP` **default OFF**). ON divides the WYLLIE porosity by `Cp = DT_SH/100`; RHG is
      self-compacting and is never touched. Nothing changes until you switch it on. Test
      (`phi_son_wyllie_cp_opt_in_only_scales_wyllie`): OFF unchanged; ON ≈ +11 % at DT_SH=90; RHG
      unaffected. In the app: run Porosity → Sonic with OPT_CP=ON on a shallow well → PHIT rises a
      few p.u.; OPT_CP=OFF reproduces the old numbers exactly.
- **Histogram full-range re-bin — left as-is** at your request (bars keep clipping the extreme tails;
  bar heights and the mode/P50 you read off them are unchanged).
- [ ] **Depth-scale dropdown now shows the TRUE scale + the mislabel is fixed**
      (`LogCanvasRenderer.ts`, `logViewPanel.ts`). The default was labelled "1:100" but was really
      ~1:3937, and the `[0.02, 20]` px/unit clamp made **1:20, 1:50 and 1:100 all collapse to the same
      zoom**. Now: a true 1:1 = `96/0.0254` px per depth unit is single-sourced; the view opens at an
      honest **1:2000**; the clamp reaches a true 1:10; and after any Ctrl+wheel/± zoom the selector
      re-reads the live ratio (a transient "1:N ⟳" entry when it's between presets). Test: pick 1:50
      then 1:100 → visibly different scales (identical before); Ctrl+wheel zoom → the box tracks the
      real ratio instead of freezing on the last preset.
- [ ] **Quiet Ctrl+S save + Escape closes ribbon menus** (`ribbon.ts`). Ctrl/Cmd+S re-saves the
      current session in place once it has a name (no dialog), falling back to Save Session As the
      first time; it's ignored while typing in an input/CodeMirror so editors keep their own Save.
      Escape closes any open ribbon dropdown without disturbing modal Escape handling. _(A Ctrl+P
      print-active-plot shortcut was deliberately deferred — resolving "the active canvas" from the
      ribbon is fragile; the per-plot Print button still works.)_ Test: name a session, edit the
      workspace, Ctrl+S → "Session … saved" with no dialog and the unsaved dot clears.
- [ ] **Bahasa Jawa (jv) added + fuller Bahasa Indonesia / Basa Sunda** (`i18n.ts`, `index.html`).
      A full Javanese (ngoko) dictionary joins id/su, and ~55 common UI phrases (New/Open/Edit/Search/
      Print/Value/Zone/Session/… + statuses) were added to all three — petrophysics jargon still stays
      English by design. Test: Project → Language → **Basa Jawa** → menus/buttons switch (Save→Simpen,
      Depth→Jero, Reload→"Muat manèh"); switch back to English → everything reverts from source.

## Reference-library correctness fixes (2026-07-20)

Two physics fixes distilled from the ITB team reference shelf (Ellis Ch12/14, Halliburton FE Ch27):

- [ ] **Multimin — PEF now converts to U before mixing.** In the Multimin (SandiMin) dialog,
      select **Photoelectric (PEF)** as an input tool (instead of, or alongside, U) and run on
      a well with a PEF curve + RHOB. Confirm VOL\_\* volumes are sensible and RECON is low in
      clean zones. Physics: per-electron PEF does NOT mix by volume — the solver now converts
      the PEF curve to U = Pe·ρe per sample (ρe from RHOB) and mixes against the U endpoints.
      Picking U directly is unchanged. Needs a RHOB curve present; where RHOB is missing the
      PEF row is simply skipped that sample.
- [ ] **VSH from Density-Neutron — new VSH_DN_FLAG clay-type guard.** Run **VSH from
      Density-Neutron** with the optional **GR** input mapped and set GR_MA/GR_SH/FLAG_TOL.
      Confirm a new `VSH_DN_FLAG` curve = 1 where the N-D VSH is off-model (gas crossover /
      beyond the shale point) or diverges from the GR VSH by more than FLAG_TOL (0.25 v/v
      default) — the signature of clay-type or gas ambiguity. Leaving GR unmapped still flags
      off-model samples. VSH/VSH_DN themselves are unchanged.

## Field Map — well surface coordinates + polygon → group (2026-07-20 #27)

**Field Map** (View/Batch ribbon map button, or the Petrophysics ▸ Field Map… button) — a
standalone dock pane that posts wells by UTM surface location and lets you rubber-band a
polygon to select wells into a well group. Coordinates arrive two ways: **Data ▸ Import Well
Locations…** (a CSV/TXT with EASTING/NORTHING, optional WELL and ZONE columns, plus a
choosable default UTM zone covering Indonesia — zones 46–54, N and S — applied to rows/files
without a ZONE column), or per-well via **Tools ▸ Well Header** (Surface X / Surface Y / UTM
zone fields). Coordinates persist as DOUBLE in new `wells` columns
(surface_x/surface_y/utm_zone) — southern-hemisphere northings ≈ 9.4e6 exceed f32's ~1 m
precision, so f64 is required. The pane draws markers with pan (drag), cursor-anchored
wheel-zoom, a faint coordinate grid, a scale bar, and labels for ≤80 wells; the active well
group is ringed. Draw mode: click to drop polygon vertices, close near the first vertex /
double-click / Enter; vertices are draggable; enclosed wells highlight live (a TS
point-in-polygon mirrors the Rust ray-cast). **Assign to group…** runs the authoritative
backend `wells_in_polygon` (PNPOLY, half-open crossing rule) and unions the result into a new
or existing group. Raw easting/northing is plotted directly — no reprojection; a multi-zone
project is a documented follow-on. The polygon is a transient selection tool — the persistent
artifact is the well-group membership (persisting polygon shapes as documents is a noted
follow-on vs the roadmap's original wording).

Adversarial review (4 lenses — geometry-math / import-parse / integration /
frontend-robustness, each finding skeptically verified): **3 defects confirmed, all fixed; 0
refuted.** (1) [high] Well Header Save wrote surface_x/y/zone unconditionally from a stale
`selectedWell` snapshot that is never re-broadcast on a data change, so re-saving after an
import (or a prior save) NULLed out the just-set coordinates — fixed by re-reading the well
from the DB when the dialog opens, so the fields always reflect current state. (2) [medium] A
blank WELL cell in a multi-well locations file collapsed into the same "no well column" case
as a headerless file and was routed to the selected well, silently overwriting an unrelated
well's location — fixed by returning a `has_well_column` flag from the parser so the importer
only falls back to the selected well for a genuinely column-less file, and skips (and
reports) blank-cell rows; the import loop is now wrapped in a transaction so a mid-file error
rolls back instead of leaving a partial write reported as a total failure. (3) [medium] When
coordinates first arrived while the pane was already open, the data-driven reload never fit
the view, so markers rendered off-screen until a manual Fit — fixed by fitting on the first
appearance of laid-out wells. cargo test 143/143; tsc clean.

- [ ] **Data ▸ Import Well Locations…** — pick your Indonesia UTM zone (e.g. 50S for
      Mahakam), import a CSV with WELL/EASTING/NORTHING → the status line reports N wells
      located; open **Field Map** and confirm the wells post at the right relative geometry.
- [ ] Import a file that has a WELL column but a blank cell in one row → that row is skipped
      and surfaced as "1 blank-WELL row(s)", and no unrelated well's location changes.
- [ ] **Tools ▸ Well Header** on a located well → Surface X/Y/zone show the imported values
      (not blank); change only TD and Save → the coordinates survive (were being wiped before).
- [ ] With Field Map already open on a project that had no coordinates, run Import Well
      Locations → the map fits to the new wells automatically (no manual Fit needed).
- [ ] Field Map ▸ **Draw polygon**, enclose a few wells, **Assign to group…** → the enclosed
      wells land in the chosen/new well group; the group filter elsewhere reflects it.

## φmax porosity ceiling — phimax module (2026-07-20 #26)

**Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** — caps a computed porosity at
the field's compaction-controlled upper limit (the deck slide-64 "max core porosity"
line). A `MODE` dropdown picks the ceiling model: **constant** (a flat `PHIMAX0`,
per-zone overridable — the literal max-line), **linear** (`φmax = PHIMAX0 −
PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000`), or **athy** (`φmax = PHIMAX0·exp(−ATHY_K·
(TVDSS − TVDSS_REF)/1000)`, the exponential compaction law). TVDSS is a
positive-downward depth-below-datum curve (same convention as **precalc**), so deeper
= larger TVDSS = lower ceiling; with no TVDSS curve it falls back whole-curve to
measured DEPTH (fine for near-vertical wells). All four parameters are zone-overridable,
so each formation (Post-Main / Main / Massive / Talang Akar) can carry its own ceiling
or its own trend coefficients. Writes `<PHI>_CAP = min(PHI, φmax)` (preserving MISSING)
and the ceiling curve `<PHI>_MAX` for a QC overlay; the input porosity is never
modified. The dialog is auto-generated from the manifest, so it appears in the Porosity
dropdown with no bespoke UI. Standalone by design — it caps _any_ porosity output
(phi_den/phi_dn/phi_son or SandiMin's PHIT); a solver-internal φmax box constraint is a
noted follow-on.

Adversarial review (4 lenses — math / integration / edge / contract — each finding
verified): **0 defects confirmed, 4 refuted** (all were test-coverage/doc-completeness
notes over correct, deliberate behaviour). Two of the flagged-untested paths were
locked in with regression guards anyway: the ceiling clamp to [0,1] (a sub-zero trend
ceiling forces porosity to 0; a super-unit one clamps to 1), and the partial-NaN TVDSS
pass-through (a NaN-depth sample gets a MISSING ceiling and passes porosity through
uncapped). cargo test 136/136; tsc clean.

- [ ] **Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** opens (auto-dialog). Run
      **constant** mode, PHIMAX0 = your field max (e.g. 0.35), input **PHIE** → the
      `PHIE_CAP` curve should equal PHIE below 0.35 and flatten at 0.35 above it;
      `PHIE_MAX` is a flat 0.35 line.
- [ ] Crossplot `PHIE_CAP` vs depth (or overlay `PHIE_MAX` on the porosity track) — the
      capped cloud should sit under the ceiling with no points poking above it.
- [ ] Switch to **linear** (or **athy**) with your TVDSS trend: PHIMAX0 at a shallow
      `TVDSS_REF`, a sensible `PHIMAX_GRAD` (or `ATHY_K`). `PHIE_MAX` should fall with
      depth; confirm a deep zone's ceiling is lower than a shallow zone's.
- [ ] Set **per-zone** `PHIMAX0` (or trend coeffs) in Zones/zone params and re-run —
      each formation's ceiling should honour its own value.
- [ ] Well with **no TVDSS curve**: linear/athy should still run (trend reads against MD
      DEPTH) — sanity-check that the ceiling still declines with depth. Deviated wells
      want a real TVDSS curve (survey→TVDSS bridge is a follow-on).
- [ ] Feed `PHIE_CAP` into **Cutoffs & Pay Summary** as the PHIE input — pay should drop
      where the cap trimmed optimistic porosity.

## Cutoff Sensitivity pane (2026-07-20 #25)

**Reporting ▸ Cutoff Sensitivity** — two ways to defend a VSH/PHIE/SWE pay cutoff
against DST-tested rock (KKT ONWJ deck slides 84–87), in one dock pane with a
Sweep / DST-Crossplot toggle. **Sweep** varies one cutoff across a range while the
other two stay fixed and plots the pay response per well — net thickness, HC
pore-thickness (HPV), or net-to-gross — so the _elbow_ shows where loosening the
cutoff stops adding real pay; the shared pay math is the **same `classify_sample`
the pay summary uses**, so the numbers reconcile. **DST Crossplot** is PHIE vs a
shale/Sw curve with every sample dim and DST-interval samples coloured per well,
plus a draggable red crosshair at the candidate cutoffs. Either mode's pick writes
into the VSH/PHIE/SWE fields and can be **saved as the pay-summary default** so the
cutoff you defended flows straight into the report. Optional zone and DST/perf
filters scope both modes.

Adversarial review raised 13, confirmed 10 (from two independent full passes),
all fixed before shipping: the sweep's PERM-cutoff scope now matches the pay
summary exactly (whole-well, not just the analysed window); overlapping
perforation/DST intervals are unioned so N:G isn't understated; switching the
swept property/metric after a run clears the stale plot so a pick can't be written
into the wrong cutoff; the "(all samples)" DST choice survives editing the well
set; the zone/DST pickers union over _all_ checked wells (was capped at 16); the
crosshair stays inside the plotted range; empty-state text is centred on HiDPI; the
plot repaints on theme change; wells with no pay / missing inputs are flagged
rather than shown as a silent flat line. cargo test 131/131; tsc clean.

- [ ] **Reporting ▸ Cutoff Sensitivity** opens as a dock pane. Tick a few wells,
      keep **Sweep**, property **VSH**, metric **Net**, Compute — one line per well;
      the curve should rise and flatten (an elbow), not be a straight ramp.
- [ ] Click/drag on the plot to place the red cutoff line; the readout shows the
      net/HPV/N:G each well delivers _at that cutoff_. Click **Use pick as VSH
      cutoff** → the VSH field updates.
- [ ] Switch the metric to **N:G**, Compute again; switch property to **PHIE** — the
      plot should **clear** and ask you to Compute (it must not keep showing the VSH
      sweep while the button says "PHIE").
- [ ] Cross-check one well against **Cutoffs & Pay Summary** at the same fixed
      VSH/PHIE/SWE (whole well, no zone/DST): the sweep's Net at those cutoffs should
      match the pay summary's Net for that well. Repeat with a **PERM ≥** cutoff set.
- [ ] **DST Crossplot** mode with a well that has a DST/perf set: dim cloud + coloured
      DST points; drag the crosshair; **Apply crosshair → cutoffs** writes PHIE and
      VSH (or SWE if the X curve is an Sw). Pick a "PHIE vs Sw" preset and confirm the
      Apply maps to SWE.
- [ ] Pick a DST set, then switch the DST dropdown to **(all samples)**, then tick/untick
      a well — the dropdown must **stay** on "(all samples)" (not snap back to the DST set).
- [ ] **Save as pay-summary default** → open **Cutoffs & Pay Summary**: its VSH/PHIE/SWE
      inputs should already carry your saved cutoffs.
- [ ] Switch the app theme with the pane open — the plot repaints immediately in the
      new palette (no stale colours).

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
FPRESS accept only precalc/log-set curves, never a raw import (a the reference suite LAS's degF
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
instead of applying: NPHI must be a fraction (percent entry rejected — the reference suite habit
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
- [ ] Deep resistivity input defaults to the RES*DEEP family (same as the sw*\*
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

- [ ] Build your standard chain (vsh → phi → sw\_\* …), switch to **Grid**: input
      curves come first, then numeric params, then options, then Mask; steps that
      don't take a column show "—". Header tooltips = parameter descriptions.
- [ ] **Set all → RW**: type one RW in the Set-all row — every sw\_\* step that takes
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
- [ ] Note: **Project ▸ Project ▸ Save Project As…** stays a backup copy (app keeps working on the
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

- [x] **CNL Por-11/12** (as before, now via the new select — old saved props migrate).
- [x] **EcoScope Por-18 (BPHI) / Por-19 (TNPH)** on an LWD well — these are the ones
      that matter for your Mahakam development wells; check a known sand against the
      sandstone line for both BPHI and TNPH inputs.
- [x] **adnVISION675 Por-16** if you have ADN wells.
- [x] **APS Por-13/14** (APLC and FPLC variants listed separately).
- [x] **PEF: Lith-3/4** on a PEF-RHOB crossplot — quartz ~1.65-1.8, calcite ~5.08,
      dolomite ~3.1 curves with 10-pu labels.
- [x] **Sonic-neutron Por-20** (both time-average AND field-observation families) on
      a DT-NPHI crossplot — TA curves reproduce Wyllie with tf 190 to R² 0.99999.
- [x] **Density-sonic Por-22** (TA + FO) on a DT-RHOB crossplot, with the 7 mineral
      points (Sylvite, Salt, Trona, Gypsum, Sulfur, Polyhalite, Anhydrite).
- [x] **Th-K clay chart Lith-2** on a POTA-THOR crossplot — the Th/K ratio fan is
      drawn at the _labeled_ ratios (the chartbook's own printed lines sag a few %
      off their labels; ours are exact), plus the dashed clay/feldspar lines and
      mineral-field labels. Judge your Mahakam illite/kaolinite mix against it.
- [x] **Pe-K and Pe-Th/K clay boxes Lith-1** (the Th/K variant needs the X axis in
      log mode — turn on X log in Properties).
- [x] **Umaa-Rhomaa MID Lith-6** — the ternary triangle with 20/40/60/80 subdivisions + K-feldspar/Barite/Anhydrite/Kaolinite/Illite/Salt points. Needs computed
      UMAA/RHOMAA curves (equation engine for now; a dedicated module is a good next
      increment if you want it).

**Audit quick fixes** (from the full senior audit — see AUDIT-2026-07-20.md and
ROADMAP §4b for the 35-finding backlog):

- [x] **Pay summary change**: with a PERM cutoff active, samples with **missing PERM
      now FAIL the cutoff** (they silently passed before). Re-run a pay summary on a
      well with patchy PERM — net pay may legitimately decrease. Tell me if you'd
      rather missing-PERM samples pass (the reference suite's default behavior differs by setup).
- [ ] **LAS import**: the file's own ~W NULL declaration is now honored (deliveries
      using -99999 etc. no longer import sentinels as data), and **multi-word well
      names survive** ("BALAM SOUTH-01" no longer truncates to "SOUTH-01"). Re-import
      one such file and check the Wells pane name.
- [ ] **Depth scale presets are now TRUE ratios** (1:200 = 1 m of well per 5 mm of
      screen at standard DPI). They were ~39x too compressed before, so 1:200 will
      look much more stretched than you're used to — the numbers are honest now.
- [ ] **Tops editor**: adding a top with an existing name is an overwrite; Ctrl+Z now
      restores the previous depth instead of deleting the top.
- [x] Case-insensitive computed-curve lookup (lowercase equation outputs now resolve).

## P2-f+ — D-N chartbook overlay (2026-07-20 #13)

Digitized from the Schlumberger 2013 chartbook you sent (Por-11 fresh / Por-12 salt,
extracted from the PDF's vector artwork — graduation-dash positions, not eyeballed;
calcite identity check rms 0.13 pu, both charts' worked examples reproduce).

- [x] **Crossplot Properties → Overlays → D-N chart**: pick _Fresh mud (Por-11)_ on an
      NPHI-RHOB crossplot → quartz/calcite/dolomite curves appear with porosity
      graduation dots + labels every 5 pu, dashed iso-porosity connectors, and curve
      names written along the lines. Compare against your paper chartbook page 225.
- [ ] **A real Mahakam sand interval** should plot on/left of the quartz sandstone line
      (shale pulls points right/down toward higher NPHI). Crossplot porosity read off
      the graduations should match your PHIE within ~1-2 pu in clean sand.
- [x] **Salt variant** (Por-12) shifts the curves left at high porosity — only relevant
      if you ever work salt-mud wells; check it renders and the graduations differ from
      Fresh.
- [x] **Zoom/pan**: the overlay must stay registered to the data under Ctrl+wheel zoom
      (it's drawn in data space). Also check the flipped orientation (X=RHOB, Y=NPHI).
- [ ] **Gating**: on a GR-RHOB plot or with a log axis the overlay silently stays off
      (chart geometry only means something on linear NPHI-RHOB).
- [x] **Note**: the chartbook draws its dolomite curve for ρma **2.85** (validated
      against the chart's own graduation ticks), while the _Matrix points_ overlay keeps
      the textbook single point at 2.87 — so Dol point and Dol curve start won't
      coincide exactly. Tell me if you'd rather I move the matrix point to 2.85.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [x] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [x] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [x] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [x] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [x] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [x] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
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

- [x] **Right-click lockdown**: right-click anywhere that has no SandiBumi menu (ribbon,
      buttons, tables, empty space) → **nothing** appears (the WebView menu with its
      dangerous Refresh is gone). Panel backgrounds still show our own menus; right-click
      inside a text box still shows the normal cut/copy/paste menu.
- [ ] **Reload guard**: press **F5** or **Ctrl+R** → a blocking confirm appears instead of
      an instant refresh; Cancel keeps everything, Reload restarts the workspace. Alt+←/→
      and the mouse back/forward side-buttons do nothing.
- [x] **Double-click-to-edit numbers** (app-wide): single-click any numeric parameter
      field (module dialogs, plot properties, SandiMin, zones…) → it focuses with a dashed
      outline but typing/arrows/wheel change **nothing**; **double-click** → solid outline,
      value selected, editing works. Tab-into-field still edits directly (deliberate).
      Scrolling a dialog with the wheel can no longer spin a value.
- [x] **Workflow Builder is a pane**: Petrophysics → Workflow… now opens a docked
      **Workflow Builder** pane (tab, movable/floatable like any panel) instead of a popup.
      No more losing a half-built chain to a stray click; it survives layout changes and
      reopens via the ＋ panel menu too. Run/cancel/progress unchanged; closing the pane
      mid-run cancels the chain.

## P1-b — Crash safe-mode, autosave, unsaved markers (2026-07-19 #3)

- [x] **Autosave**: the workspace (panes, arrangement, active well, every log view's
      layout) autosaves every 10 seconds. Nothing to click — just know it's there.
- [x] **Crash recovery**: if the app dies abnormally (crash, force-kill, power loss),
      the next launch shows a choice **before** anything loads: _Restore autosaved
      workspace_ (everything back as it was moments before the exit) or _Start in Safe
      Mode_ (clean default layout; the autosaved workspace is stashed as a "Recovered …"
      session under Open Session, so nothing is lost). To test without crashing for real:
      end the task from Task Manager while the app is open, then relaunch.
- [x] **Normal restart is less lossy now**: on a clean exit + relaunch, the app also
      brings back the **active well** and each log view's **layout/track state** (before,
      only the pane arrangement survived).
- [ ] **Unsaved markers**: edit a log view (track widths, properties, curve visibility)
      → its tab shows **●**, the **Project ribbon tab** gets an amber dot (visible without
      leaving the tab you are on), and **Project ▸ Session ▸ Save Session…** gets a red dot. **Save
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

- [x] **Properties dialog**: double-click or right-click the crossplot (or ⚙ Properties)
      → sectioned dialog (Plot / Axes / Z color / Regression / Overlays). The old
      always-visible properties row is gone; the toolbar is just X/Y/Color/Zone.
- [x] **Marginal histograms + percentiles**: enable marginals on NPHI-RHOB — X histogram
      on top, Y histogram on the right, aligned with the axes (RHOB's inverted axis
      included). Percentiles `25, 75` draw dashed reference lines on both axes.
- [x] **Regression options**: on a PHIE-vs-PERM cloud try Power + RMA — the fit line
      must be straight on log axes and curved on linear ones, equation + R² + method
      tag shown top-left. Compare Y-on-X vs RMA slope on a noisy cloud (RMA steeper).
- [x] **Log-safe Z coloring**: color by PERM with "Log Z scale" + Viridis — low and high
      decades must stay distinguishable (rainbow + linear crams everything in one hue);
      the color bar is labeled "(log)".
- [x] **Plot size**: set Fixed 500×400 — the plot stops stretching with the pane
      (consistent exported figures). "Fill panel" restores the old behavior.
- [x] **Universal defaults**: Qtz/Cal/Dol matrix points no longer appear on NPHI-RHOB
      unless ticked in Properties; Color has a "— None —" option (custom point color
      applies); the pick rows + drag handle can be hidden ("Show parameter pickers" —
      still ON by default so your drag-to-set-shale-point workflow is unchanged).

## P2-e — Histogram v2 (2026-07-20 #11)

- [x] **Properties dialog**: double-click or right-click the histogram plot (or the ⚙
      Properties button) → one dialog holds display mode (bars/line), bins, normalize,
      cumulative overlay, box plot, color, percentiles, statistics placement, and the
      parameter-picker toggle. When zoomed, the first double-click resets the zoom, the
      next one opens properties.
- [x] **Box plot + cumulative overlay together**: enable both on a GR histogram — the
      P25–P75 box with P50 line and P5/P95 whiskers sits under the marker labels, and
      the cumulative % curve (secondary color, % labels on the right edge) tracks the
      bars. Zoom in with Ctrl+wheel: box and whiskers follow the axis.
- [x] **User percentiles**: type `10, 90` in Properties → P10/P90 marker lines on the
      plot and removable chips above it (click a chip to drop that percentile). Values
      must match what you'd read off the cumulative curve.
- [x] **Statistics inside the plot**: set Statistics → "Inside the plot" (chips hide) or
      "Both" — the in-plot block shows the active stats incl. new Min/Max. Check it in a
      dark theme too (block background must follow the theme).
- [x] **Universal by default**: a fresh histogram opens with NO Pick A/B rows and clicking
      the plot does nothing — enable "Show parameter pickers" in Properties to get the
      GR_MA/GR_SH picking workflow back. Your saved bar color / percentiles / etc. must
      survive closing and reopening the panel.

## P2-d — Log-view layout interaction (2026-07-19 #10)

- [x] **Collapsible track headers**: ▤ in the log-view toolbar cycles full → compact
      (curve names as inline chips, no scale lines) → titles only. Headers also cap at
      ~a third of the pane and scroll inside, so a 15-curve track can't eat the screen.
      Try it on your densest layout.
- [x] **Move/copy curves between tracks**: drag a curve name from one track header onto
      another track's header — the curve MOVES there (its color/scale/fill travel with
      it). Hold **Ctrl** while dropping to COPY instead (e.g. overlay NPHI on the GR
      track). Ctrl+Z undoes either.
- [x] **Track borders**: ▦ in the toolbar — solid / dashed / none, width 1–4 px, theme
      color (follows light/dark) or a custom color. Default is a thin solid separator
      at every track boundary; check it looks right in dark themes too.
- [x] **Readout follows ONE track now**: hovering shows only the curves of the track
      under the cursor (not all 15). CLICK a track to lock the readout to it (header
      highlights, click again to release) — then you can run the cursor over the whole
      layout while reading just that track's values.
- [x] **Right-click log editing**: right-click on a track → "Edit CURVE…" for each of
      its curves. Ops: **Wireline shift** (whole-curve depth shift, resampled onto its
      own grid — NaN where it slides past the logged interval), **Set constant**,
      **Blank (erase)**, **Interpolate across** (bridge a bad interval linearly),
      **Scale a·v + b** (recalibration). Works on raw (GR/RHOB…), computed, and
      imported generic-store curves alike; every apply is ONE Ctrl+Z entry that
      restores the previous samples bit-exactly, and lands in the History panel.
      Suggested check: blank a washout interval on RHOB, interpolate across it,
      then Ctrl+Z twice — the original curve must come back exactly.

## P2-c — Well pin rework + multi-select (2026-07-19 #9)

- [x] **Pin is now a mode, not a lock.** 📌 ON (default): clicking a well in Wells &
      Tops moves EVERY log view and plot to it — the old behavior. 📌 OFF: each view
      keeps the well it's showing; only the panel you're working in (the active tab)
      follows your clicks. Open two log views, turn the pin off, activate the second
      view, click different wells — only the second view changes. That's the
      side-by-side compare workflow.
- [x] **The old lock is gone**: no more "Active well is locked" blocking when you
      click other wells, and no more weird interaction with a second wells pane.
- [x] **Multi-select**: Ctrl-click wells to build a selection (highlighted with an
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

- [x] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [x] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

## SandiMin — the reference suite-parity mineral solver (2026-07-19, v2)

Rebuilt to the reference suite Multimin / IP Mineral Solver conventions (spec extracted from your
the reference install helpset + IP2018 install → `docs/multimin_ref_spec.md`, `docs/multimin_ip_spec.md`).

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
      its own endpoint column (default σ 0.015, the reference suite's user-defined default).
- [ ] **Endpoints matrix**: editable per component×tool; unflushed-zone fluid cells show "—" for
      nuclear tools (only CT sees them — the reference suite convention); CT/CXO cells show "auto"; per-row
      **Max** bound (fluids default 0.5, the reference suite's cap).
- [ ] **Fluid properties** panel (visible when CT/CXO on): Rw@temp, Rmf@temp, formation temp, m, n,
      mud type. The preview line shows the computed w, Cw, Cmf, Cbw, α(x/u) and auto CT/CXO σ —
      sanity-check Cw against your Pickett Rw (Cw = 1/Rw@FT, mho/m).
- [ ] **Run** on a Balam well with RHOB+NPHI+DT+GR+RES*DEEP: writes VOL*\* per component +
      MM_PHIE, MM_PHIT, MM_SWE, MM_SWT (+ MM_SXOT, MM_MOVEDHC when both zones present),
      MM_VSH (clays + bound water), MM_RECON. Check: **Σ(minerals + unflushed fluids) ≈ 1**,
      **MM_SWT is sensible vs your sw_indo/RtC runs** (this is the new resistivity coupling —
      "resistivity convert to ct and cxo" as requested), and MM_RECON spikes where the model fails.
- [ ] Add **BoundWater** with Illite selected: VOL_BOUNDWATER should track ≈ 0.18×VOL_ILLITE at
      ~150°F (the the reference suite dual-water bound-water constraint, k = 96·CEC·ρ/(T°C+298)).
- [ ] Add **Oil Sxo + Oil Sw** with CXO available: SXOT ≥ SWT in water-based mud (WATER MUD
      constraint) and MM_MOVEDHC = unflushed HC − flushed HC ≥ 0 across invaded pay.
- [ ] Requested upgrade (ROADMAP §4 P3): optional **nonlinear Sw equation iterated to
      convergence** inside the solve loop.

## ML suite (2026-07-19)

- [x] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [x] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [x] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [x] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [x] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [x] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [x] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [x] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
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

- [x] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [x] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [x] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [x] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
      curves** from the chosen result percentiles.

## Phase 8.5 — your method suite in core (remaining validations)

- [ ] **SSC — Sand-Silt-Clay (Advance tab)**: run on an LQR-style well with
      GRN + RHOB + NPHI (sandstone units). Check VSAND/VSILT/VDCL/VWCL, PHIT/PHIE/PHIFF,
      CBW/CWSH/BW, SWIRR_T/SWIRR_EFF and the `*_GR` GR-equivalent volumes against your
      the reference suite run. Defaults are the LQR `.info` values (wet clay 2.3/0.6, dry clay 2.71,
      wet silt NPHI 0.3, DCLF_SI 0.1). Two deliberate deviations, flag if they matter:
      (1) `RANNORMAL(SWIRR_MIN·PHIT, 0.005)` is deterministic here; (2) the Loglan's
      NPHIMA limit 0.5–5 (a copy-paste of the RHOMA limit) is corrected to 0–1.
- [ ] **SSPW (Advance tab)**: the Loglan exec body wasn't on disk, so the
      arithmetic (PHIT from VSH-mixed dry matrix, CBW = VSH·VOL_CBW_SH,
      CAPBW = VSH·(PHIT_SH − VOL_CBW_SH), PHIE = PHIT − CBW, PHIFF, SWIRR floor) is
      **reconstructed from the spec — please validate against your the reference suite "LAS PHIT
      PHIE" exports** and tell me any systematic difference; I'll adjust the equations.

## Phase 8b — report generator (2026-07-18)

- [x] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [x] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [x] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [x] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [x] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [x] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

## Module-panel cleanup, Help tool, bulk Processing report, responsive resize (2026-07-21)

Five asks from your VSH-panel screenshot (SandiMin deferred for later review).

- [ ] **Module form no longer lists per-well results**: run a module (e.g. VSH from
      Gamma Ray) — the form now shows one summary line ("All N well(s) computed. Per-well
      details are in the Processing panel." or "…N need attention…") and the Processing
      panel comes forward. The old `✓ well: samples → curves` list is gone from the form.
- [ ] **Per-well detail lives in Processing → details**: expand a job's **▸ details** —
      running wells show individually; the narration paragraph that used to sit at the top
      of the module form is gone (it moved to Help, below).
- [ ] **Bulk failure report**: when many wells fail the SAME way, Processing → details shows
      **one card per reason** — "N well(s) failed — <message>", the well list (first 12 +
      "…(+K more)"), and a "→ what to do" advice line — instead of one row per well.
- [ ] **Help (?) tool**: click the **?** in the top quick-access bar (or right-click any
      panel → **Help for this panel…**). A guide opens for whatever panel is active — a
      module pane shows that method's description; other panels show a short blurb. (This is
      the placeholder that will later link to the full illustrated help library.)
- [ ] **Ribbon dropdowns still work**: open **Petrophysics → VSH/Porosity/Saturation**, and
      **Data → Import Logs/Import Data/Tools**, and **Project → Recent** — each menu must drop
      fully below the ribbon (this was a regression the review caught and fixed).
- [ ] **Resize the whole window**: the content panes (log views / plots / inspector) reflow to
      fill; **Wells & Tops, Processing, and Performance keep their width** (they're a fixed
      sidebar now). Try both wider and narrower — nothing should get clipped or leave dead space,
      and the ribbon stays reachable (scrolls if very narrow).
- [ ] **Close panes without the sidebar growing**: close a plot/log view — the freed space goes
      to the other content, NOT to the sidebar. Close *everything* down to just the sidebar and a
      blank **Workspace** pane fills the rest (rather than the sidebar stretching); open any log
      view/plot and the blank pane disappears again.

## Multi-well crossplot overlay (T-SHELL-16 increment 1, 2026-07-30)

Crossplot only in this increment (histogram is next; Pickett needs a decision — its
m/n/Rw are per-well parameters). Design: extra wells draw as a FADED CONTEXT LAYER
behind the active well; everything interactive stays on the active well.

- [ ] **Wells: Active button** in the crossplot toolbar (after Zone) — click it to open
      the well-scope row: Active / Group / ★ Pinned / Selection / All / Custom…, the same
      control as the batch dialogs. Default **Active** = today's single-well plot, unchanged.
- [ ] **Pick a wider scope** (e.g. All, or a group): the other wells' points fade in
      BEHIND the active well's cloud, one colour per well, with a **Wells legend** top-right
      (active well first; long names truncate; >10 wells collapse to "+N more"). The legend
      footer says "context is display-only".
- [ ] **Context wells are display-only**: brushing (Shift+drag), the draggable parameter
      handle, zone-parameter writes, core overlay, T-S endpoints, regression, tooltips and
      the net polygon all still act on the ACTIVE well only — check the brush highlights
      only active-well points and log views follow only its depths.
- [ ] **Zone windows resolve per well by NAME**: with a zone (or a selected top) chosen,
      each context well shows ITS OWN depths for that same-named zone/top — wells without
      it are skipped and counted in the scope row ("N skipped"), never guessed from the
      active well's depths.
- [ ] **Point budget**: a huge scope decimates context wells to ~60k points total — the
      scope row reports "~N pts (decimated)". The active well is never decimated.
- [ ] **Axis auto-range covers the field**: with context wells on and auto ranges, a
      neighbour whose cloud sits outside the active well's spread is still visible (not
      clipped); manual ranges and mnemonic defaults (NPHI/RHOB…) behave as before.
- [ ] **Scope survives a well switch**: set scope All, click another well in the Wells
      pane — the rebuilt crossplot keeps scope All (and the new active well takes over the
      interactive role). SVG/PDF/PNG export includes the context layer.

## Multi-well histogram overlay (T-SHELL-16 increment 2, 2026-07-30)

Same scope treatment as the crossplot, adapted to distributions: context wells draw
as **stepped outline curves** behind the active well's bars, one colour per well.
The comparability rule: **each context well is normalized to its own sample count**
and scaled to the active axis — you compare distribution SHAPES, so a neighbour
with 3× the samples never dwarfs the active well (this is the GR-normalization
use case). Pickett is deliberately NOT scoped yet — your call pending (m/n/Rw are
per-well parameters).

- [ ] **Wells: Active button** in the histogram toolbar (after Zone) — same scope row
      as the crossplot. Default **Active** = today's single-well histogram, unchanged.
- [ ] **Wider scope**: context wells appear as stepped outlines behind the bars, with a
      **Wells legend top-left** (active well first with a filled swatch, context wells
      with line swatches matching how they render; footer "context: per-well shape ·
      display-only"). Works in bars and line mode, count and Normalize-% mode.
- [ ] **Shape, not size**: overlay a small zone of a big well — its outline peaks near
      the active well's bars (same shape → same height), NOT 3× above them. In
      Normalize-% mode the outline is that well's true per-well percentage.
- [ ] **Pooled X range**: a context well whose distribution sits outside the active
      well's P2–P98 (e.g. an unnormalized hot GR well) stretches the axis so its curve
      is visible, not clipped. Single-well range behaviour unchanged.
- [ ] **Stats stay active-well**: chips, P5/P50/P95/mean markers, user percentiles, box
      plot, cumulative curve, picks A/B and the brushed sub-distribution all still read
      the ACTIVE well only — context outlines never move a statistic.
- [ ] **Same zone-by-name + skip rule** as the crossplot; scope row reports counts,
      decimation and skips. Scope survives a well switch; SVG/PDF export includes the
      outlines and legend.

## Pickett v2 completion + multi-well overlay (T-SHELL-16 increment 3, 2026-07-30)

The Pickett already had free M/N/Rw fields, Properties (axes, point size, Z-colour)
and viewport-preserving N changes from an earlier pass. This increment adds the rest
of the audit items plus the scope overlay. The multi-well decision, as agreed: the
**overlay shows whether neighbours share the ACTIVE well's water line** — m, n and Rw
are per-well parameters and never come from a context well.

- [ ] **Wells: Active button** in the Pickett toolbar (after Zone) — same scope row as
      the other plots. Default **Active** = today's single-well plot, unchanged.
- [ ] **Wider scope**: context wells' clouds fade in behind the active well's, one
      colour per well, Wells legend top-right, footer "context is display-only". The
      water-line readout adds "line = ACTIVE well's parameters" whenever context is on.
      A neighbour sharing the water line hugs the same Sw=1 edge; one with different Rw
      sits visibly shifted — that's the point of the overlay.
- [ ] **Water-line picks, M/N/Rw, brushing, tooltips, zone writes**: all still act on
      the ACTIVE well only. Clicking two points fits M/Rw from the active cloud even
      with context wells showing.
- [ ] **Template bar** (★ Save template / recall / 🗑) — Pickett display settings
      (axes, point size, Z-colour) now save under a name like Histogram/Crossplot.
      Recalling a template with garbage values is safe (everything sanitized).
- [ ] **New default RT axis 0.2–2000 ohmm** (audit fix — 0.1–1000 clipped
      high-resistivity pay). Your saved axis ranges are untouched; only a fresh
      panel/profile sees the new defaults.
- [ ] **Sw lines span the visible window**: set a custom porosity range (e.g.
      0.02–0.5) or zoom — the Sw = 1 / 0.5 / 0.25 lines run edge to edge instead of
      stopping at the old fixed φ = 0.01–1 span.
- [ ] **Scope survives a well switch**; SVG/PDF export includes the context clouds
      and legend. Same zone-by-name + skip rule, budget and scope-row reporting as
      the other two plots.

## MID plot module — UMAA / RHOMAA (2026-07-30)

Feeds the Lith-6 chart overlay that has been sitting in the chartbook library with
nothing to plot on it. New **Lithology** category in the Petrophysics ribbon.

- [ ] **Petrophysics → Lithology → Apparent Matrix (MID plot)** runs on a well with
      RHOB + NPHI + PEF and writes four curves: **UMAA**, **RHOMAA**, **U** (volumetric
      photoelectric factor) and **PHIA** (the apparent porosity it actually used —
      exposed so the basis is never hidden).
- [ ] **Crossplot X = UMAA, Y = RHOMAA** opens on the chart's own window (UMAA 0–16,
      RHOMAA 2.2–3.1 with density increasing downward). Properties → Chart overlay now
      lists **"Lith-6 Umaa-Rhomaa MID plot"** under *For these axes* — switch it on and
      the quartz / calcite / dolomite triangle, the clay and anhydrite points and the
      percentage lines land around your cloud.
- [ ] **Read a known carbonate or a clean sand** and check the cloud sits where the
      lithology says it should. Please push back if the placement disagrees with your
      chartbook reading — the analytic apparent porosity is the one approximation here.
- [ ] **The porosity basis is a visible choice** (OPT_PHIA in the run dialog), and the
      default **CHART** now reads the density-neutron crossplot the way you would by hand
      on Por-11 — it solves for the porosity at which both tools imply the same matrix,
      interpolating across the chartbook's sandstone / limestone / dolomite curves. Pick
      the curve family with **TOOL** / **SALINITY** (same choices as Neutron Matrix
      Conversion). **NPHI must be in apparent-limestone units** — run Neutron Matrix
      Conversion first if your log is recorded in sandstone or dolomite units.
- [ ] **Compare CHART against XPLOT on a dolomite or mixed-carbonate interval**: XPLOT
      (the analytic average commercial suites take, kept for comparison) leaves dolomite
      about 0.06 g/cc light and 0.34 b/cm³ left of its chart point; CHART puts it on the
      dolomite line. If your chartbook reading disagrees with CHART, that's the one I
      most want to hear about.
- [ ] **Anhydrite / pyrite intervals stay heavy** rather than dropping out (denser than
      every matrix line, they clamp to the end of the search and plot in the chart's
      high-RHOMAA corner). **Gas** pushes points low-left, exactly as on the printed
      chart — the module does not "fix" gas, so the gas signature stays readable.
- [ ] **Density-only porosity is deliberately absent** — it is algebraically degenerate
      (it returns the assumed matrix density for every sample, a constant curve that
      would still plot convincingly). There is a unit test stating the trap.
- [ ] **Barite mud warning** is in the method note: PEF is unreadable there. Run with
      **Mask = BADHOLE** on rugose intervals.
- [ ] **Over-porous samples drop out** as blanks rather than as huge numbers (PHIA_MAX,
      default 0.5, is an editable parameter — not a hidden constant).

## Per-well parameter override table (Phase 9-2, 2026-07-30)

The last open Phase 9 item. A workflow step carries one parameter set for every well,
which breaks when a field needs a different Rw per fault block. The storage already
allowed the fix (a `zone_params` row with zone `*` is a whole-well override, and runs
already apply it) — what was missing was a way to reach it for more than one well at a
time. **Resolution order is unchanged: step value → this whole-well override → named
zone.** Nothing about how your existing runs resolve has moved.

- [ ] **Petrophysics → Batch → Workflow…**, build or open a chain, then **Per-well
      parameters…** next to Run. Rows are wells, columns are the numeric parameters the
      chain's steps actually take.
- [ ] **Grey = inherited, amber = overridden.** A fresh grid is all grey (every well
      inherits the step value). Double-click a cell to give one well its own value — it
      turns amber. The cell tooltip tells you which it is.
- [ ] **Double-click to edit, not single-click** — same rule as every other numeric field
      in the app, so a stray click near a parameter can't change it. Enter commits, Escape
      cancels, blank clears the override.
- [ ] **Typing the inherited value back clears the override** (cell returns to grey)
      rather than storing a duplicate — the same only-store-differences rule the per-step
      editors use.
- [ ] **Columns marked ⚠ behave differently on purpose.** If two steps in the chain take
      the same parameter with *different* step values (e.g. Archie RW 0.05, Indonesia RW
      0.07), the header shows ⚠ and the column displays only the first step's number.
      There, typing the displayed value **stores** it instead of clearing — because
      clearing would leave the two steps disagreeing again, when what you meant was "this
      value for every step in this well". Hover the header for the explanation.
- [ ] **Out-of-range values are refused with a status-bar message, not clamped.** Try
      entering RW = 25 (a v/v value typed as a percentage). This matters: the run itself
      REJECTS an out-of-range override and fails the whole chain, so catching it here turns
      a failed 2000-well run into a red cell.
- [ ] **Set for all shown / Clear for all shown**: pick a column, type a value, and every
      well currently listed takes it in one write. Narrow the list first with the **Wells**
      scope (All / Group / ★ Pinned / Selection / Custom) and the **Filter** box — the
      buttons act on exactly what you can see.
- [ ] **One Ctrl+Z reverses a whole sweep.** Set a column across 50 wells, then undo once —
      all 50 revert together, not one per press. Redo re-applies them.
- [ ] **Copy as CSV** puts the shown grid on the clipboard so you can diff it against your
      own well table in Excel. *(CSV import back is the obvious next step and is NOT built
      yet — tell me if you want it, it's small now that the write path exists.)*
- [ ] **Zone parameters still win.** Set a whole-well RW here and a different RW on one
      zone (Zones panel) — the zone value should govern inside that zone and the grid value
      everywhere else. This is the check that matters most.

---

## 2026-07-30 — Example import datasets (`dataset for test/examples/`) + BLSO core header fix

One folder with a working exemplar of EVERY import format, pooled where you asked:
`dataset for test/examples/`. Three synthetic wells (SANDI-01/02/03) with shared,
physically consistent geology — a gas sand and a water sand whose core, SCAL and log
values all agree by construction. The `README.md` in that folder is the map: each file →
exact ribbon menu → what the status bar should say → what each parser accepts (the full
alias lists), so you can shape a confusing real delivery against the nearest analogue.
These files are ALSO parsed by `cargo test` on every gate run (`example_data_test.rs`) —
if a parser ever changes in a way that would break the published examples, the gate goes
red. Regenerate with `py -3 tools/make_example_data.py` (deterministic).

- [ ] Data → Import Logs ▾ → **Import LAS…** → multi-select the three `SANDI-*.las` →
      3 wells, ~394 rows each; PEF/CALI appear in the Curve Catalog (set RAW); Bad-Hole QC
      flags the deliberate 1-m washout gap mid-SAND-A.
- [ ] Follow the README's numbered import order (tops → locations → deviation → core →
      3 SCAL shapes → petrography/XRD/perforations). Every import should succeed with the
      README's stated result — any deviation is a bug, tell me.
- [ ] N/D crossover shows gas in SAND-A on any well; Archie in SAND-B gives Sw ≈ 1 —
      the README's "known-good expected values" section is the eyeball checklist.
- [ ] **Real-data fix:** your BLSO core-log delivery (`blso*_lapi2023_core.csv`,
      `03. Core Logs`) now imports grain density — the `GDEN_1` header resolves (it
      silently dropped before). CPERM_1/CPOR_2/CSW_1 already resolved; the FEET units row
      is skipped safely. Re-import one BLSO core CSV and check CGD in the DB Inspector.

---

## 2026-07-30 — Import sets: one well, many deliveries (T-IMP-02, -03, -04, -06)

Your Geolog screenshots, built. A delivery folder is now a **set**: `01. Final Log`'s RAW,
FPROOH, MULTIMIN, SSC and SSPW can all land on **one** well record instead of five
same-named ones, and you can see which is which.

- [ ] **Import LAS… now asks first.** A "Import LAS — curve set" dialog opens with the set
      name already filled from what your filenames share: pick the FPROOH folder's files and
      it suggests `FPROOH`; MULTIMIN suggests `MULTIMIN`. Verified against all five of your
      BLSO folders. Blank = RAW.
- [ ] **Attach to existing wells (default ON).** Import blso00025 from **RAW**, then again
      from **FPROOH**, then **MULTIMIN** — you should end with **ONE** well carrying three
      sets, not three wells. The status line says how many were new and how many attached.
- [ ] **A set name is never overwritten.** Import the same FPROOH folder twice: the second
      lands as `FPROOH_1` (Geolog's WIRE → WIRE_1 rule). Nothing from the first import moves.
- [ ] **▸ twisty in the Wells pane** expands a well into its sets, and a set into its curves
      (mnemonic + unit; hover for sample count, family, run number). Both FPROOH's PHIE and
      MULTIMIN's PHIE are visible under their own sets — that was the whole ask.
- [ ] **Existing projects behave EXACTLY as before.** This is the check that matters most:
      **set RAW keeps absolute priority** in curve resolution. A module asking for PHIE still
      gets RAW's PHIE when RAW has one; only a mnemonic RAW does *not* carry (e.g. `PHIFF`,
      `VOL_QUARTZ`) is looked up in the attached sets. Re-run a module you have run before
      and confirm the numbers are identical.
- [ ] **Import DLIS… also asks for a set name.** Give a second tape its own name and both are
      kept instead of the second replacing the first — your "we don't always know what's
      inside" point. Leaving it as RAW keeps the old replace-and-count behaviour.
- [ ] **The malformed exemplars you asked for now exist** (you wrote "where do u provide
      dup_depth.las?"): `dataset for test/examples/bad_dup_depth.las` imports with a
      dropped-duplicates warning and 35 rows; `bad_null_depth.las` fails cleanly and creates
      no well row. Both are asserted by cargo test.

*Not built, and worth saying plainly:* selecting files from **two different sets in one
import** (e.g. an FPROOH and a MULTIMIN file together) finds no common name, falls back to
RAW, and mixes them — one import batch is one set by design. Import per folder.

---

## 2026-07-30 — Core & aux import v2: the "hundred wells with cores" workflow (T-IMP-07/-09/-10/-11)

Import Core is now **probe → confirm → commit**: nothing is written until you have seen and
approved what the file means. Your note is the spec: well names come FROM THE DATA, every
property column is confirmed first (name, type, unit, percent), and 1-or-many CSV **or
TXT/tab-delimited** files work in one action. BLSO is just the exemplar — the reader takes
any delimited text and shows each column's sniffed type (number/text/empty).

- [ ] **Data → Import Core… with NO well selected** → pick
      `dataset for test/examples/core_rcal_multiwell.csv` → the wizard shows: WN as the well
      column, 3 wells with row counts, the units row detected and skipped, depth unit `m`,
      CPOR/CSW flagged as percent, and a 5-row preview. Import → plugs land on all three
      SANDI wells by name.
- [ ] **Real data:** multi-select ALL 321 files in `03. Core Logs\BLSO_LAPI2023_CORE` in one
      Import Core. The mapping is confirmed once (by header name) and applied per file;
      depth unit should read `ft` from the units row and convert to the project unit.
      Unmatched well names are listed by name, never guessed.
- [ ] **The Duri trap:** import the parent folder's `Core.csv` — it has a numeric `WELL`
      column (804) AND a textual `WELL NAME` (DURI00804). The wizard must pre-pick **WELL
      NAME** (a pad number can't route rows); check the routing line before importing.
- [ ] **Wrong mapping is refusable:** change Depth to a text column, or blank the well
      column with no well selected — Import refuses with a reason, writes nothing.
- [ ] **Import Aux… routes by WELL now:** pick `xrd_multiwell.txt` (tab-delimited) with any
      well selected → rows land on all three wells; the result box names unmatched/blank
      rows. A file with no WELL column still binds to the selected well as before.
- [ ] **Shift Core (T-IMP-09) is unblocked** — run it as written in the test plan; it still
      shifts the SELECTED well's plugs only.

---

## 2026-07-30 — Core import: the EXTRA columns come in too ("any column, any data type")

`core_data` holds four measurements (porosity, permeability, grain density, Sw). A real lab
export is wider — lithology descriptions, So, Kv/Kh, sample IDs, tape names. Those columns
now ride along from the SAME wizard: they land as **point data at the plug depths**, typed
per cell (numbers as numbers, anything else as text), so a wide delivery imports whole in
one pass instead of needing a second Import Aux run.

- [ ] **Import Core… → `dataset for test/examples/core_rcal_multiwell.csv`** (the exemplar
      now carries `SO_1`, `LITH` text and mixed `SAMPLE_ID`). Tick **Extra columns** → the
      5 leftover columns appear with their type (`LITH (text)`, `TAPE_NAME (empty)`…).
      Untick `TAPE_NAME`/`TOOL_STRING`, leave the rest, Import → the status line reports the
      plugs AND "Plus N point-data value(s) from SO_1, LITH, SAMPLE_ID".
- [ ] **Check the values landed as themselves:** Database Inspector → `aux_data`, dataset
      `CORE` — `LITH` in value_text ("SANDSTONE"), `SO_1` in value_num, depth_base empty
      (they are point samples), depths matching the plug depths.
- [ ] **A column can't be stored twice:** with Extra columns on, re-point **Water saturation
      (CSW)** at `SO_1` — it leaves the extras list immediately and `CSW_1` takes its place.
      Columns you unticked stay unticked.
- [ ] **Dataset name is yours:** change "Store them under dataset" to e.g. `CORE RCAL`;
      re-importing the same file replaces that dataset for the well (same discipline as the
      plugs themselves), it never doubles up.
- [ ] **Real data (the point of it):** a BLSO core CSV's extra columns, or the Duri
      `Core.csv` wide export — everything the four core slots don't claim is available
      without a second import pass.

**Note by design:** extras are stored **verbatim** — no percent→v/v or feet→metres
conversion is applied to them (the depth they hang on IS converted). The wizard confirms
what a column *is*; it does not reinterpret its values. If you want a specific extra
treated as a real curve/measurement instead, tell me which and it becomes a mapped role.

---

## 2026-07-30 — Core sets & survey versions: nothing overwrites anything (T-IMP-08 / T-IMP-12)

You marked T-IMP-08 **Fail** with "refer T-IMP-02 about how duplicated data managed", and
T-IMP-12 the same. That is now the rule for core and surveys as well: **one delivery = one
named set, and an import never overwrites an earlier one.**

One difference from curve sets, on purpose: curve sets are read TOGETHER (a set supplies
mnemonics RAW lacks). Two core deliveries measure the SAME plugs, so reading both would
double your φ-k cloud. Exactly **one core set and one survey are ACTIVE** per well, and
everything reads that one — log overlay, crossplots, HFU, SandiMin calibration, Shift Core,
DB Inspector edits, TVD/TVDSS.

- [ ] **Import the same core file twice.** Import Core suggests a set name from the filename
      (`blso00025_lapi2023_rcal.csv` → `RCAL`). Second import → status says
      `Core set RCAL_1 — 1 well(s) already had a 'RCAL' set, so theirs was suffixed`. Both
      deliveries are kept, the newest is live.
- [ ] **The plug count does NOT double.** Open a φ-k crossplot or the core overlay after that
      second import — same number of points as one delivery, not two.
- [ ] **Data → Tools ▾ → Data Sets…** on that well: both sets listed with plug
      count, source file and import date, ● on the live one. Click **Use** on the older one →
      the plots repaint to that delivery. **Delete** asks first; deleting the live one hands
      over to the next newest (never leaves plugs no panel can see).
- [ ] **Surveys:** import a preliminary survey (`SURVEY`), then a definitive one
      (`DEFINITIVE`). Both listed; TVD at TD reflects the definitive. Switch back with **Use**
      → status says TVD/TVDSS was rebuilt, and TVD at TD changes back. This is the part worth
      checking hardest on a real deviated well — a stale TVD would quietly feed every
      height calculation.
- [ ] **Your existing projects:** open one that already has core and/or a survey. It migrates
      on launch (a backup copy is written beside the project first, per the release rule), the
      old data appears as set/survey **RAW**, active, and **every number reads exactly as
      before**. Check a φ-k plot and a TVD you know.
- [ ] **Duplicated depth inside ONE file** still drops first-kept with the note — that is a
      broken row in a single delivery, not a second delivery.

---

## 2026-07-30 — …and the same rule for EVERY point dataset, plus the tree

Your note: *"not only core, any kind of point data should behave universally like core — we
have a lot such xrd, cec, oil show, etc."* Right — those all live in one store, and until now
a second delivery of any of them silently replaced the first. They now version exactly like
core: **one delivery = one named set, one live per (well, dataset)**.

- [ ] **Import Aux… now has a Set field** (default `RAW`). Import an XRD file twice → the
      result box says `Set RAW_1`, both deliveries are kept, the newest is live, and the
      panel counts show ONE delivery's values, not the sum.
- [ ] **Datasets are independent.** With XRD switched to the older delivery, CEC / oil show /
      perforation stay exactly as they were — activation is per dataset, not per well.
- [ ] **Wells pane ▸ twisty** now shows, under each well: its curve sets (as before), then
      **Core**, **Surveys** and **Point data** with ● on the live one.
      **Double-click** a dimmed row (○) to make it live — panels repaint. Single click does
      nothing on purpose, so a stray click in a long well list can't repoint your data.
      Deleting stays in the manager dialog.
- [ ] **Core extras follow their core set** — a core file's LITH/So/sample-id columns are
      stored under the SAME set name as the plugs, so switching a well's core switches its
      extras with it instead of leaving a mismatched pair.
- [ ] **Old projects:** point data predating this is adopted as set `RAW`, active — your XRD
      and petrography read exactly as before. (Unlike core, this needs no table rebuild.)

---

## 2026-07-30 — SCAL deliveries version too; the manager is now "Data Sets…"

The last store that still overwrote on re-import. A capillary-pressure report is now a named
delivery like everything else — **the files you select together in one Import SCAL are ONE
set** — and only the live one feeds Pc QC, the Leverett-J fit and Thomeer.

- [ ] **Import SCAL… has a SCAL set field** (default `SCAL`). Import a centrifuge set, then a
      porous-plate report → status says `Set SCAL_1`, both are kept, the newest is live, and
      the Pc QC plot shows ONE report's points.
- [ ] **Switch back** in **Data → Tools ▾ → Data Sets…** (renamed — it now has four sections:
      Core, SCAL, Deviation surveys, Point data) or by double-clicking the row in the Wells
      tree → the Pc plot and any J-fit you re-run follow the other report.
- [ ] **Old projects:** existing Pc points are adopted as set `SCAL`… actually `RAW`, active —
      your saturation-height work reads exactly as before.

That completes the sweep: **curves, core, SCAL, surveys and every point dataset now version
the same way.** Nothing in the app silently overwrites a delivery on re-import any more.

---

## 2026-07-30 — Field-scale open hardening: memory cap, Compact Project, visible upgrades

From your BLSO report (2.5 GB file, ~6 GB RAM, 15-minute open). The 15 minutes was the two
one-time storage upgrades each backing up the whole project first — but the file itself was
~75% dead space (632 MB of live data in a 2,487 MB file), the engine was allowed ~80% of the
machine's RAM, and all of it happened silently. All three fixed:

- [ ] **Second open of BLSO is fast.** The upgrades ran once; reopening the project should
      take well under a minute now. If it is still slow, tell me — that would be a
      different problem than the one fixed here.
- [ ] **RAM stays civil.** With BLSO open, SandiBumi's memory should sit near 4 GB at the
      very worst (the engine is capped at min(≈20% of RAM, 4 GB), spilling to disk beyond
      that instead of taking the machine). Power users: set `SANDIBUMI_DB_MEMORY=8GB` in the
      environment to raise it on a big field machine.
- [ ] **Data → Tools ▾ → Compact Project…** on BLSO: after the confirm, the status line
      should report roughly `2,487 MB → ~630 MB`, everything still opens and plots, and the
      original file is parked beside the project as `.pre-compact-<ts>.duckdb` — delete it
      yourself once satisfied. Every table's row count is verified before the swap; any
      failure puts the original back untouched.
- [ ] **Save Project As now compacts too** — it exports through the engine (live rows only),
      so a Save As of a bloated project lands at its true size.
- [ ] **Nothing silent any more:** opening a project that needs a one-time upgrade shows
      "Opening project… (a first open after an update can run one-time storage upgrades…)"
      while it works, and afterwards the status line + History panel say what ran, how long
      it took, and where the backup went.

---

## 2026-07-30 — Audit backlog #128: long operations no longer freeze the window

Follow-on from the open-hardening work. Anything that can run for minutes was still executing on
the app's main event-loop thread, so while it worked the window itself was frozen — Windows shows
"not responding", nothing repaints, no button responds. Six such operations now run on a worker
thread. (Chain/ML/SandiMin runs were already off-thread; this closes the rest.)

- [ ] **Open Project on BLSO** (or any large project): the window stays alive and repainting the
      whole time, the status line's "this can take minutes" message is readable, and the app is not
      greyed out / "not responding". This is the one worth checking first — it is the operation you
      hit the 15 minutes on.
- [ ] **Compact Project** and **Save Project As** on BLSO: same — the window stays responsive
      while gigabytes are rewritten. Panels that need the database will pause until it finishes
      (correct — they must not read a half-swapped project), but the window itself never freezes.
- [ ] **Recompute TVD/TVDSS Curves** across many wells: window stays alive.
- [ ] **SQL Query panel**: run a deliberately heavy query (e.g. a join over `computed_curves`
      with no WHERE). It should be interruptible-feeling — the window stays responsive instead of
      locking up until the query returns.
- [ ] **Nothing changed in behaviour** — same results, same errors, same undo. This increment is
      purely *where* the work runs.

**Startup itself is fixed in the next section.**

---

## 2026-07-30 — The window now opens before the project does

The last and worst version of the same problem: SandiBumi opened your project *before* creating
its window, so during those 15 minutes there was **nothing on screen at all** — you double-clicked
and the machine appeared to ignore you. Now the window comes up immediately and the project opens
behind it.

- [ ] **Launch on BLSO:** a window appears within a second or two, showing a small
      **"Opening project…"** card with a moving bar and a running clock. The app is visibly alive
      and on screen the whole time. After ~20 seconds it adds a line explaining that a first open
      after an update upgrades the project's storage, backs it up first, and happens only once.
- [ ] **The card tracks what the backend is doing** — when the storage upgrade starts, its message
      changes to name the backup file it just wrote.
- [ ] **A normal (fast) launch shows no card at all** — open a small project; it should go
      straight to the workspace with no splash flash.
- [ ] **Afterwards**, the History panel and the status line record how long the open took and what
      ran, so a slow launch has an explanation you can go back and read.
- [ ] **Nothing appears before its data is ready** — no empty well list, no "0 wells" flash. The
      workspace is not built until the project is genuinely open. **If you ever see an empty
      Wells pane on a project that has wells, tell me — that would mean the gate leaked.**
- [ ] **A broken project still explains itself:** the existing "could not open" dialog still
      appears (now after the card, not instead of a window).

---

## 2026-07-30 — Imports no longer refuse a file over its text encoding

Your Duri core table failed with `Core import failed: io error: stream did not contain valid
UTF-8`. The cause, found in the bytes: **330 KB of pure ASCII except two `0x95` bytes** — the
Windows bullet "•" that opens a lithology description — and the whole delivery was refused over
two characters in a comment field. Any file that has been near Excel or Word can carry those
(smart quotes, en/em dashes, °, µ).

Every text import now decodes tolerantly: a byte-order mark is honoured first (so Excel's
"Unicode text" UTF-16 export works too), then UTF-8, and anything left falls back to Windows
cp1252 — which cannot fail, so **an import is never refused over encoding again**. This covers
core, LAS, tops, aux/point data, SCAL and deviation alike, not just the file that reported it.

- [ ] **Import your Duri `Core.csv`** — it should now read 12 columns, **3,045 plugs across 15
      wells** (DURI00513 … DURI01887), depth detected as **ft**, and CPOR/CPERM/CGD(GDEN)/CSW
      mapped automatically. The DESC / LITH / CORE_NO / KV / CSO columns are offered as extra
      point-data columns in the same wizard.
- [ ] **The bullet survives as a bullet** in the description, not as a `?` or a black diamond —
      check a DESC value in the Database Inspector after import.
- [ ] **Nothing else changed**: re-import an ordinary UTF-8 or plain-ASCII file (BLSO core, a
      LAS) and confirm identical results to before.

---

## 2026-07-30 — Wells pane: right-click on everything, and point data expands like curves

Your two asks: expanded items should have a right-click menu (including a route into the Curve
Catalog for editing), and non-curve data should behave like curves — expandable within a set,
with its own menu.

- [ ] **Right-click a curve** (under an expanded set) → Open in Curve Catalog · Edit name /
      unit / family… · Make this curve win its name · Delete. "Open in Curve Catalog" should
      land on the Inspector's Catalog tab **already filtered to that curve**, not on a list of
      everything.
- [ ] **Double-click a curve** opens the same edit dialog (single click stays inert on purpose —
      these rows sit in the same list as wells, and a stray click must not move the workspace).
- [ ] **Rename a curve and check it took**: `GRN_CS` → `GR` on your Duri well. Values must be
      unchanged (same sample count in the Catalog), and a **GR-based module should now see it** —
      that is the real reason to rename, not cosmetics. **Ctrl+Z undoes it.**
- [ ] **Point data / core / SCAL / surveys now have a ▸ twisty** and expand:
      - Core → the properties its plugs actually carry (`CPOR (61)`, `CPERM (61)`, …)
      - Point data → its named items (`LITH (305)`, `CSO (61)` — your Duri core extras)
      - SCAL → one row per plug with its Pc point count
      - Surveys → station count, MD range, TVD at TD, max inclination
      Only the **live** delivery expands; an inactive one says so rather than showing the
      active one's contents (which would be a lie).
- [ ] **Right-click a delivery** → show contents · make it the live one · Open Database
      Inspector · Data Sets…. Deleting still lives only in Data Sets…, never a stray click.
- [ ] **Right-click a well** → expand · Curve Catalog · Database Inspector · Data Sets… · pin.

---

## 2026-07-30 — Blocky curves and crossover shading

Your two display asks: "option to display curves as continuous or blocky style", and "we also
don't have shading to other logs". Both live in the same place — **Layout Properties → the
curve table**, which gained a **Style** column and two new Fill choices.

- [ ] **Blocky (step) curves.** Layout Properties → pick **Blocky** in the new Style column on
      any curve. The value should now hold flat all the way down to the next sample and then
      jump, instead of sliding diagonally between sample centres. Try it on something genuinely
      piecewise-constant — a zone-constant parameter curve, a block-averaged or upscaled log,
      VSH from a coarse pass. **The shading follows the step**: a blocky curve's edge fill is a
      stack of rectangles, not a stack of wedges.
- [ ] **Continuous is still the default** — every existing layout you have saved should open and
      draw exactly as before. Nothing needs re-saving.
- [ ] **Crossover shading.** Layout Properties → Fill → **Crossover to curve**. It auto-picks the
      other curve in the same track as the reference and seeds the two swatches with the two
      curves' own colours, so you can see the separation immediately. **Shading** now shows two
      swatches: left one = where the styled curve reads LEFT of the reference, right one =
      where it reads RIGHT.
- [ ] **The reference must be in the SAME track.** That is deliberate, not a limitation: the
      reference is positioned with **its own min/max**, and compatible scaling is the whole
      meaning of a neutron-density crossover. Naming a curve from another track shades nothing.
- [ ] **The built-in Standard Layout now ships the NPHI/RHOB crossover** (grey where NPHI reads
      left of RHOB — shale / clay-bound water; yellow where it reads right — gas effect). The
      Facies layout's porosity track matches. **Scales are unchanged** (NPHI 0.45→−0.15,
      RHOB 1.95→2.95). Tell me if you would rather the built-ins stayed plain.
- [ ] **Check it on a real gas sand** in BLSO or Duri: the colour should flip exactly where the
      two curves cross, not a sample early or late.
- [ ] **Print agrees with screen.** Plot ribbon → Composite… on a layout using both features —
      the PDF/SVG must show the same blocky steps and the same two-colour crossover.
- [ ] **Bug fixed in passing**: a curve whose Fill you had set to **None** used to print with a
      left-edge shading in the Composite/report PDF even though the screen showed it clean. It
      now prints unshaded. Worth a glance at any deliverable you generated before today.

---

## 2026-07-30 — Point-data tracks: core plugs, XRD, text, box plots and histograms

Your ask: "we dont have any option to show point data, text data, or even image with its own
style option to show it as histogram or box plot per x range interval with its own adjustment
as well such percentile showing, whisker, etc." Images are still to come; everything else is
here. Layout Properties → **Track type → Point data**.

- [ ] **Add a point track**: Layout Properties → set Track type to **Point data** → **＋ Add
      point series**. Source **Core plugs** lists your well's real plug properties
      (CPOR/CPERM/CGD/CSW); source **Point dataset** lists your real datasets — for Duri that
      is CORE with LITH, CSO, KV, and whatever else the wizard carried in as extras.
- [ ] **Points** (default) draws one diamond per plug at its own depth and value. Unlike the
      old core overlay this is a track of its own, so you can scale it how you like instead of
      borrowing a curve's scale.
- [ ] **Text** draws the sample's text at its depth — your `LITH` descriptions, oil show.
      Labels are thinned so a densely described core stays readable rather than a black smear,
      and truncated at the track edge instead of spilling into the neighbour.
- [ ] **Box plot** summarises the plugs inside each depth bin: box edges, median, whiskers,
      outliers. All adjustable per series — **Bin height** (blank = follow the zoom, a value =
      a fixed depth interval that stays put at every scale), **Box low/high %**, **Whiskers**
      (Tukey k×IQR / Percentiles / Full range), and **Show samples** to draw the individual
      plugs as ticks above the box.
- [ ] **The whisker rule is a real choice, so check both.** Tukey answers "which plugs are
      unusual for this interval" and flags outliers individually; Percentiles answers "where
      do 80% of the plugs lie" and flags nothing. Switch between them on a Duri interval with
      a wild plug and confirm the picture changes the way you expect.
- [ ] **Histogram** draws a value-axis histogram per depth bin, bars scaled to that bin's own
      peak count so a thinly sampled interval is still readable next to a dense one.
- [ ] **Nothing is clamped.** A plug outside the track's Min/Max is skipped, not pinned to the
      edge — check by narrowing Max below your highest CPOR and confirming those plugs vanish
      rather than stacking on the right-hand border.
- [ ] **A blank cell is not a zero.** If your core table has an empty CGD column for some
      plugs, those plugs must contribute nothing to a CGD track — not a cloud at 0 g/cc.
- [ ] **Print agrees with screen**: Plot ribbon → Composite… on a layout with a point track.
      Same boxes, same medians, same outliers, same labels.
- [ ] **Existing layouts are untouched** — a saved layout with no point track opens exactly
      as before.

**Note on where this is heading** (your instruction): the box/percentile/whisker machinery is
deliberately written to know nothing about core plugs. It takes a set of values and a depth
bin. That is so **array logs — your 1000-realization Monte Carlo PHIE — reuse it unchanged**,
because 1000 realizations at one depth is the same statistic as 40 plugs over an interval.
When we do array logs, the display options you set here will already mean the same thing.

---

## 2026-07-30 — Array logs: adjustable band, spaghetti and density heat map

This is the array-log increment the point-data note above was written for. The
box/percentile machinery was reused **unchanged** — no second statistics path was created.

**Producing one** (Petrophysics → Batch → Monte Carlo…):

- [ ] **Options** now has **Store realizations (array log)**, greyed out until *Save
      LOW/BASE/HIGH curves* is ticked (it rides the same pass over the kept runs, so on its
      own it would silently do nothing).
- [ ] Run with both ticked. The status line reports the saved curves, and the notes list
      `stored MC_<KEY>_REAL — N depths x M realizations` per well.
- [ ] Only outputs the chain **produces** get a matrix — an input curve it merely reads must
      not come back as a fake zero-width band. (Same rule as the percentile curves.)
- [ ] With more than 256 realizations kept, a note says the stored set is the first 256, so a
      band drawn from it can differ slightly from the MC_*_LOW/_HIGH curves. **Nothing should
      differ silently.**

**Displaying it** (log view → ⚙ → **Track type → Array log** → **＋ Add array series**):

- [ ] The **Array curve** box suggests what this well actually has (`MC_PHIE_REAL`, …). With
      no array logs at all, the panel says so and points at the Monte Carlo option rather
      than offering an empty picker.
- [ ] **Uncertainty band** — shaded P-low to P-high with the P50 line through it.
- [ ] **This is the adjustable part**: change *Band low %* from 10 to 5 (or to 40/60) and the
      band redraws immediately from the same stored realizations. **No re-run.** That is the
      whole reason the matrix is stored rather than just three curves.
- [ ] *Median line* off leaves the shading alone; *Shading* sets the fill opacity.
- [ ] **Spaghetti** — individual realizations. *Traces* sets how many. They are sampled
      **evenly across the run**, not the first N: the first N of a Latin-hypercube design sit
      in one corner of the sampled space and would understate the spread.
- [ ] **Density heat map** — per-depth value histogram, darker where more realizations landed.
      *Value bins* sets the resolution.

**Data-honesty rules to try to break:**

- [ ] **A gap stays a gap.** At a depth where too few realizations converged, the band
      **splits** rather than shading straight through. Shading across it would claim an
      uncertainty range for a depth the study gave no answer for.
- [ ] **A failed realization breaks its own trace** in spaghetti instead of being bridged to
      the next depth — the bridge would draw a path that realization never took.
- [ ] **Off-scale heat-map values are dropped, not clamped.** Narrow the track min/max until
      part of the distribution falls outside: those samples contribute **no** cell rather than
      a false dark column at the track edge.
- [ ] Band and spaghetti, being continuous readings, **clip at the track edge** like any log
      curve — deliberately different from a core plug, which is skipped.

**Print + back-compat:**

- [ ] **Print agrees with screen**: Plot ribbon → Composite… on a layout with an array track.
      Same band, same gaps, same traces, same heat map.
- [ ] **Existing layouts are untouched** — a saved layout with no array track opens exactly as
      before, and an older project migrates without a backup pause (the old `array_logs` stub
      never held a row, so there is nothing to protect).

**Worth knowing:** a stored matrix is the only Monte Carlo output whose size scales with
iterations (~2 MB per curve per well at the 256 default). If a project starts to drag, the
matrices can be dropped without touching the study that produced them, and Data → Tools ▾ →
Compact Project reclaims the space.

---

## Provenance & exposure sweep — Tier A + B applied (2026-07-31)

`docs/provenance_sweep_prompt.md` run end to end: 24 findings, **11 Tier A + 2 Tier B applied**,
6 Tier C and 5 Tier D routed and untouched. Full register with `file:line` in the gitignored
`docs/commercial/PROVENANCE_SWEEP.local.md`; questions for counsel in `LAWYER_PACKET.local.md`.

**Two behaviour changes — check these first, they are the only things that alter what you see:**

- [ ] **GR Normalization defaults changed.** Petrophysics → Prep → GR Normalization now opens
      with `GR_LOW_REF = 20`, `GR_HIGH_REF = 120` gAPI (was 53.68 / 133.93). The old pair was one
      field's regional calibration from 562 wells — somebody else's field standard, shipping to
      every user, and silently wrong anywhere else. The new pair is the app's own generic
      clean/clay endpoints (`vsh_gr`'s GR_MA / GR_SH). **The doc string now tells you to set your
      own field reference** — read it and confirm it says what you would tell a junior.
      *Your real pair is preserved in `docs/commercial/`. Re-runs of old wells will differ; that
      is expected — enter your own reference to reproduce a previous study.*
- [ ] **Python environment variable renamed** to `SANDIBUMI_PYTHON`. Every message that used to
      say `ARSHILLA_PYTHON` — DLIS import, ML, image import, Workbook, Word, Deck, the equation
      editor — now says the new name. **Your existing `ARSHILLA_PYTHON` still works** and is read
      silently; nothing to change on your machine. Confirm by opening Plot → Deliverables →
      Workbook… and reading the message if Python is missing.

**Client material out of the tree:**

- [ ] The 20 hard-coded delivery paths in the `#[ignore]`d field tests are gone. They now read
      **`SANDIBUMI_FIELD_FIXTURES`** — point it at a folder with `las/` and `core/` subfolders
      and the tests use whatever is in it. Verified both ways: unset, all five skip with a
      printed reason; set at the example wells, the core probe resolved 11 headers / 30 rows /
      3 wells and the full chain ran to a pay summary.
- [ ] `dataset for test/Core.csv` — real core plugs from one client well, referenced by no code —
      is **still in the tree**. Removing it is one command; it is left for you because git history
      keeps it either way and that is your decision, not a fix. Same for the tracked
      `Prompt/*.pdf`, which `CLAUDE.md` wrongly claimed was gitignored (now corrected).
- [ ] `Review.txt` / `Review 2.txt` moved to `docs/commercial/` and untracked (superseded by this
      file; one named two client assets).

**Licences — new file:**

- [ ] `THIRD-PARTY-LICENSES.md` now exists: 289 crates, 154 npm packages, **zero undeclared**,
      six weak-copyleft (MPL-family, all transitive, none modified — they permit shipping a closed
      binary). Generated by `node tools/gen-third-party-licenses.mjs`; re-run after any dependency
      change. There is still **no project `LICENSE` file** — that is your text to write.

**One judgement call worth your eye:**

- [ ] `multimin2.rs` cited `docs/multimin_geolog_spec.md` for the incoherence statistic — **a file
      that does not exist**. I replaced it with the primary source I believe is correct: Mayer &
      Sibbit, SPE 9341, *GLOBAL, a new approach to computer-processed log interpretation* (1980).
      Confirm that citation before it is quoted to anyone; it is the one provenance claim in this
      batch I chose rather than found.

**Left alone on purpose** (Tier C/D — do not read these as missed): the four client-branded
themes, the tooltips naming which vendor tables seeded a default, the study citation in `lrlc.rs`,
the RtC regression coefficients (no neutral default exists — that is a petrophysics decision and
it is yours), the 2.9 MB of vendor research extractions, and git history.

---

## scipy in the equation engine (2026-07-31)

Petrophysics → Database Inspector → **Equation Editor**, language **Python (numpy)**. When scipy
is installed in the interpreter SandiBumi picked, your scripts can now use `signal`,
`interpolate`, `optimize`, `stats` and `ndimage` directly — no import line needed. numpy is still
the only requirement; nothing changes if you never touch scipy.

**The note tells you before you write, not after you run:**

- [ ] Open the Equation Editor with language **Python (numpy)**. The grey note under the tab now
      ends with the interpreter path **and** `· scipy 1.18.0`. If scipy were missing it would say
      `· no scipy — install it for signal/interpolate/optimize/stats` — a note, not a warning,
      because the engine is fully usable without it.

**Four things worth trying on a real well** (inputs `GR`, output as named):

- [ ] **Despike** — output `GR_DS`:
      `gr_ds = signal.medfilt(gr, 5)`
      A 5-sample median. Casing collars and washout spikes go; the bed boundaries stay put,
      which a mean filter would smear.
- [ ] **Smooth** — output `GR_SM`:
      `gr_sm = signal.savgol_filter(gr, 11, 2)`
      Savitzky-Golay preserves peak height and shape far better than a running mean.
      **Despike first.** A polynomial fit over an un-despiked curve fits the spike rather than
      the rock — try it both ways on a washed-out interval and you will see it immediately.
- [ ] **Fit your own φ-k** — inputs `PHIE, PERM`, output `PERM_FIT`:
      ```
      import numpy as np
      ok = np.isfinite(phie) & np.isfinite(perm) & (phie > 0) & (perm > 0)
      def model(x, a, b): return a * np.power(x, b)
      p, _ = optimize.curve_fit(model, phie[ok], perm[ok], p0=[1.0, 3.0], maxfev=20000)
      perm_fit = model(phie, *p)
      ```
      Mask the invalid samples yourself — `curve_fit` has no NaN handling and will simply fail.
- [ ] **Resample / fill** — `interpolate.interp1d(depth[ok], curve[ok], bounds_error=False)`.

**Two rules you may want to test deliberately:**

- [ ] **A curve wins a name collision.** If a well ever has a curve called `STATS`, your script
      gets *your curve*, not `scipy.stats`. Your data never yields to a library name.
- [ ] **A missing scipy names the fix.** On a machine without scipy, a script using `signal`
      fails with the interpreter path and the exact `pip install` command — not
      `NameError: name 'signal' is not defined`. Worth checking on a colleague's machine, since
      that is the whole point of the message.

**Also renamed here:** the interpreter override is `SANDIBUMI_PYTHON` (see the previous entry);
your existing `ARSHILLA_PYTHON` still works.

---

## RtC calibration from your own water zone (2026-07-31)

**Advance ▸ Calibrate RtC…** This closes the last open item from the provenance sweep. `sw_rtc`
always told you to "recalibrate per field from water-zone excess conductivity" and never gave
you a way to do it — so in practice one study's coefficients ran on every field. Now you point
it at a water sand and it gives you *your* A_CAP / B_QV / C0.

**Try it on a well where you know the water leg:**

- [ ] Click a water-bearing top in the Tops pane first — the dialog seeds the interval from it.
- [ ] Set Rw / M to **the same values your `sw_rtc` run will use**. They define the clean
      baseline the excess is measured against; a fit against a different Rw is a fit for
      different rock. The dialog says so.
- [ ] Fit, then read **R² and the "Not fitted" line before the coefficients.** If R² is low the
      excess here is not explained by CAPBW and Qv, and the coefficients are not worth having.
- [ ] **Copy**, then paste into the `sw_rtc` parameters. Deliberately not auto-applied — that
      would skip the step that matters.
- [ ] Compare SWE_RTC before and after on a known interval. This is the real test: does your
      own calibration move Sw the way your core and tests say it should?

**Three things to poke at deliberately:**

- [ ] **It refuses without a water zone.** Clear both depth boxes and the flag curve, then Fit.
      You should get a refusal explaining that fitting over pay hands the hydrocarbon's
      resistivity to the clay term. That refusal is the most important behaviour here — over
      hydrocarbon the fit reads Sw too HIGH, so a careless calibration *erases* pay rather than
      inventing it.
- [ ] **Nothing is dropped silently.** The "Not fitted" line counts every excluded sample by
      reason — outside the interval, not flagged wet, incomplete inputs, or "no excess to
      explain" (Rt reads above what clean water-filled rock can be, usually meaning Rw is wrong
      for the interval or it is not actually wet). A calibration from 12 samples of a sand you
      thought held 500 is a different statement.
- [ ] **RSF is held fixed**, and the result says the coefficients are only valid for that RSF.
      Change RSF afterwards and they are void — RSF multiplies the whole bracket, so it and the
      three coefficients cannot be separated by this regression.

**Worth knowing:** with no QV log and CEC = 0 the clay term cannot be fitted at all. It is
reported as **0 with a note** rather than guessed, and the capillary term absorbs whatever
constant clay conductivity is present.

---

## IMTS S-factor calibration from your own lab CEC (2026-07-31)

**Advance ▸ Calibrate S…** Same story as RtC, one module along. `sw_imts` defines S as a
measurement — your lab CEC divided by the CEC the clay model predicts — and the app shipped
**0.5**, which was never measured in any rock. S multiplies the whole clay-charge term, so a
wrong S scales Qv_eff straight through to SwT with nothing on the log to show for it.

**Try it on a well with a CEC suite:**

- [ ] Point it at the dataset and item holding your lab CEC. Get the item name wrong on purpose
      — it should tell you **which items are actually there**, not just "no data".
- [ ] **Name the clay curves your `sw_imts` run will use** (VDCL / VILL by default), not the XRD
      table the CEC came from. This is the trap: calibrate against one estimate of clay and run
      against another and S is wrong by the difference — invisibly, because both are clay
      volumes.
- [ ] Fit, then **Copy** — it copies S together with CEC_KAOL and CEC_ILL, because S multiplies
      those constants and the three are one setting.
- [ ] Run `sw_imts` with your S and compare SWT_IMTS against the shipped 0.5. On clay-rich rock
      the difference should be substantial; that gap is what the placeholder was costing you.

**Read these before the number:**

- [ ] **Plug ratios P10 → P90.** This is the real check, not R². If the plugs' own ratios span
      more than a factor of two, no single S describes them and it says so. Either S genuinely
      drifts with clay content, or the lean plugs are noisy — a small measured CEC divided by a
      small modelled clay volume is a noisy ratio either way.
- [ ] **The "Not fitted" line.** A plug further than the depth tolerance from any log sample is
      **dropped, not snapped** to the nearest one. If most of your plugs land there, the core is
      not depth-shifted to the log. Worth knowing: a shift that happens to be a whole number of
      log samples is invisible to this check — the log grid cannot see it — so the tolerance is
      not a substitute for shifting against core gamma.
- [ ] **Plugs where the clay model says no clay** are excluded rather than divided by zero. If a
      plug there has real measured CEC, that is evidence against your clay curves, not a data
      point.
- [ ] **S above 1** gets flagged. The method expects lab CEC *below* the XRD-theoretical value,
      so above 1 your clay model is under-calling exchange capacity — most often a mineral it
      does not carry. Smectite is 80-150 meq/100g against illite's 25, so a few percent of it is
      enough. That S then only suits rock with the same smectite fraction as your cored plugs.

**Also fixed here:** both calibration dialogs used to open with a blank, greyed-out Fit button
until you touched the well scope. They now label themselves on open.


`sw_rtc`'s own description now says plainly that the shipped defaults are one field's, and
points at this dialog.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
