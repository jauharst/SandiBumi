# Getting data in, and letting a log set carry its own sampling

Build record for Intake, Statistics, Condition, Frame and Reframe, and for the three limits that
were closed afterwards. The plan and every decision behind them are in `plan_data_tools.md`.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## Intake, Statistics, Condition, Frame and Reframe (2026-08-05)

Three tool families Jauhar asked for from Geolog screenshots — a table importer, a statistics list,
a log-edit and sampling list — delivered under our own names, plus the log-set freedom he asked for
in the same breath. The plan and every decision are in `docs/plan_data_tools.md`; this records only
the contracts a future session must not break.

**`condition.rs` (Condition) and `frame.rs` (Frame) are MODULES, not a bespoke editor.** A Rust fn
plus a manifest buys multi-well rayon-parallel runs, zone-overridable parameters, chaining, the
universal mask, log-set versioning and an auto-generated dialog. `curve_edit.rs` remains the
interactive one-interval path. Four family rules: **a window is a THICKNESS, never a sample count**
(a sample window silently changes how much rock it covers when a curve is resampled); **nothing
invents a sample except Fill Gaps, which flags every one**; **the output is never the input's own
mnemonic**; **a parameter that cannot have a generic value has no default** (`param_open`).

**The Hampel filter's textbook form fails on the easiest case.** One spike among identical
neighbours makes the MAD exactly zero, so `|v − m| > K·0` is false and the filter finds nothing on
a flat curve with an obvious spike. `window_spread` falls back to the mean absolute deviation, and
`MIN_HAMPEL_SAMPLES = 5` refuses a window too narrow to measure a spread against. The control test
(a constant window must reject NOTHING) is what stops the fall-back becoming a spike detector that
fires everywhere.

**Upscaling permeability arithmetically gives a rock that does not exist**, and it is the one of
the three means that always reads highest, so the error never looks like a problem: 1000 mD and
0.01 mD are 500 mD arithmetically, 3.16 geometrically, 0.02 harmonically. `frame::block`'s
`OPT_STAT` and `reframe`'s averaging both offer all three and the docs name the case for each;
MEAN stays the default only because it is right for porosity and every volume fraction.
Statistics' Curve Summary reports all three SIDE BY SIDE rather than as a setting — showing one is
how a mean permeability gets quoted arithmetically — and withholds the two log-scale means
entirely where any sample is non-positive, rather than computing them over the positive subset and
putting a statistic about a different set of samples in the same row.

### Every output name is decided in ONE place

`ArgSpec.default` on a **LogOut** is the default NAME (`log_out_as("SYN", "{TARGET}_SYN", …)`), the
exact parallel of its meaning on a LogIn. `workflow::resolve_output_names` expands the pattern,
applies any `__OUT_<declared>` rename, and validates. **A module returns its DECLARED key and never
builds a name of its own** — pinned by
`every_module_returns_the_output_keys_its_manifest_declares`, which drives the whole catalog
through one synthetic frame.

Five modules used to `format!` their own names, so the manifest described a curve the run did not
write: a dialog reading "Outputs: SYN" was untrue, and offering a rename would have meant a second
copy of each module's naming rule. **The shadowing refusal moved here with it** — it lived in
`condition.rs` and again in `frame.rs`, and the other forty modules had none, so a rename could
have put a computed curve on `GR` and produced one nothing can read. `equations::STANDARD_COLUMNS`
is now the single list. Two outputs resolving to one name are refused too: which survived would
otherwise depend on hash order.

The module pane's **Output curves** card is a row per declared output; the workflow builder has the
same boxes in its per-step editor and as `→ NAME` columns in the grid. **No Set-all on an output
name** — two steps writing their VSH under one name is the collision above. `OUT_PREFIX` composes
on top (renames first, then the prefix) and **Monte Carlo refuses either form by name**, because
its plan builder resolves cutoffs from the declared LogOut names and would otherwise return
plausible percentiles computed from nothing.

### A log set can carry its own sampling (`reframe.rs`)

**Every curve read in this app is an exact depth match onto the well's standard grid** —
`fetch_curve_frame` reads `standard_curves ORDER BY depth`, and the generic store and the log-set
archive are then looked up with `by_depth.get(&d.to_bits())`. A 0.1524 m delivery attached to a
well whose grid came from a 0.5 m LAS therefore contributes almost nothing: no error, no warning,
a curve that reads mostly MISSING. Reframe (Data ▸ Sampling) resamples a source onto a declared
sampling and writes a new set carrying its own depth column, marked `log_sets.frame = 'OWN'`
(`db::migrate_log_set_frame`, ADD COLUMN only; existing sets are `'STANDARD'`, which is a fact
rather than a guess since nothing could write anything else). `fetch_curve_frame_from_set` then
makes that set's depths the RUN frame and resamples everything else onto it through
`reframe::resample_onto` — the same function the tool uses, so the preview and the run cannot
disagree.

**Written to the ARCHIVE only.** `write_computed_curves_versioned` DELETEs a curve's current rows
before appending, so a re-frame through the ordinary path would blank the readable interpretation
and replace it with rows that align with nothing, reporting success as it went.

Three rules the tests found rather than the code: **boxes are half-open `[lo, hi)`** (closed at
both ends counts a boundary sample twice, worst where the sampling divides evenly); **a one-sample
frame owns the whole source** rather than silently returning nothing; and **`looks_discrete` needs
more than "small non-negative integers"** — a GR alternating 40 and 80 API is two such integers,
and the first version mode-averaged it to 80 where the rock averages 60. Codes must also be dense
in their own range, or small enough (`OBVIOUS_CLASS_CODE`) that no measurement could be mistaken
for them. It stays a guess, which is why the resolved method is REPORTED per curve.

### Regularize, and one frame across wells (2026-08-07)

Two capabilities the roadmap listed under a ✅ from the day they were SCOPED, and which did not
exist — `resample`, `regularize` and `align_multiwell` were carried as shipped Frame modules for
two days. The correction is not just the checkbox: **they were never Frame modules and cannot be.**
A module is handed a curve frame and returns a vector aligned to it, so it cannot change the sample
count at all. `frame::block` looks like an exception and is the proof — it upscales by replacing
values at the well's own depths rather than by producing fewer of them, which is exactly why a
blocked curve is written `draw_style: "step"`. Anything that changes the sampling has to write a
new set with its own depth column, and that is Reframe. Jauhar's own redirect on 2026-08-05 said
so before the code did: *"resample and regularize, log cons/set should be have independent
sampling."*

**Regularize takes the source's OWN spacing when no step is given.** The operation is "make this
uniform", not "make this coarser": a delivery whose depths wobble around 0.1524 should come out at
0.1524, and asking the user to read that number off the probe and type it back is only a chance to
get it wrong. It falls back to the median rather than the mean or the modal gap — one dropped
interval in a thousand moves a mean and cannot move a median. A source whose depths do not advance
is refused by name rather than divided by zero.

**A shared step is not a shared frame, and this closed a real defect.** The `step` target anchored
each well on its own first depth (`target.top.unwrap_or(src_top)`), so ten wells re-framed at 0.5
came out on 1500.00, 1500.50 … and 1498.25, 1498.75 …: the same STEP, no common DEPTH. Every read
here is an exact depth match, so nothing downstream could line those wells up — **the exact failure
Reframe exists to fix, reappearing one level up.** `TargetSpec.align` folds a MIN/MAX depth query
across the selected wells and hands every one of them the same top, base and step. `match_well` and
`match_set` never had the defect and ignore the flag, because a borrowed frame is taken WHOLE — the
file already reasoned that this is what makes two wells come out on the same rows; `align` gives a
computed frame the same guarantee.

Depths a well has no data for come back **MISSING**, stated in the run's notes rather than left to
be discovered. That is the same rule the borrowed frame follows, and the honest one: a shallow well
on a field-wide frame has no measurements at the deep end, and a gap says so where an interpolation
would not.

**Regularize plus align without an explicit step is REFUSED.** Each well has its own spacing, so
the fallback would have to elect one — and whichever well won would silently become the standard
for the field, in a run whose output looks entirely plausible. It is the `gr_normalize` argument:
where there is no generic answer, refuse by name rather than pick.

The shared interval comes from `source_extent`, a `MIN(depth)/MAX(depth)` aggregate per source
kind, so the alignment pass does not read every well's curve data a second time — the per-well pass
still reads each source exactly once. It deliberately spans the source's WHOLE range rather than
only the curves being re-framed: the frame is a property of the delivery, and a set whose extent
changed with the curve selection would not be one frame at all.

Pinned by `aligned_wells_land_on_identical_depths_not_merely_the_same_step`, which asserts from
both sides — the aligned wells must share every depth AND the unaligned ones must share none, or a
flag that did nothing would pass — plus
`regularize_adopts_the_sources_own_spacing_when_no_step_is_given` and
`regularize_across_wells_refuses_rather_than_electing_one_wells_spacing`.

### An imported LAS stays on its own sampling in the log view (2026-08-10)

**Displaying a curve is not Reframe.** A set-qualified imported curve is fetched from
`curve_samples` on that set's own stored depths (after intake's declared-unit reconciliation and
chosen depth-sanitize policy). It is never projected onto `standard_curves` merely because the log
view needs pixels. The curve identity is `(set_name, mnemonic)` all the way through layout,
legend, visibility, readout, WebGPU renderer and composite SVG/PDF lookup, so `WIRE/GR` and
`WIRE_1/GR` cannot silently select one another on screen or in a client deliverable. Old layouts
with no set remain readable through the legacy resolved-curve path; a user can make the choice
explicit in Layout Properties, whose Set list only offers sources carrying that mnemonic.

**Decimation is viewport-only and disposable.** The first reduction carries the true source top and
base as structural points even when neither value is a bucket extreme, so it establishes the exact
whole-well depth extent. Once pan or zoom settles, the backend filters to that visible depth interval
BEFORE display decimation and the renderer swaps only the visible series while retaining that
extent. Zooming therefore requests denser source samples for the smaller interval instead of
magnifying a coarse whole-well trace. Neither the display query nor its decimator writes a row or
invents a sampling; Reframe remains the only tool allowed to create a new frame.

**The catalog and tree now ask different database questions.** Curve Catalog reports total rows,
finite valid rows, missing rows, and finite-only min/max/mean. A curve containing only missing
values has counts but no invented statistics. Wells/Set expansion uses a metadata-only inventory
and reuses its cache for a pure collapse/expand; it must not scan or group `curve_samples` just to
draw a tree. A data-changing action invalidates that cache and fetches fresh inventory.

**Normal LAS import is one decoded parse and one atomic columnar write.** The primary parse retains
the WELL value and every non-index channel, so the file is not reopened for its name or a second
full-curve parse. Concurrently live parsed deliveries are bounded to the Rayon worker count.
The well row, stored unit, six-column standard projection, generic metadata and every native curve's
samples commit in one outer transaction; samples are staged as Arrow vectors and inserted through
DuckDB in bulk while the `(curve_id, depth)` primary key remains enforced. Length/cast/constraint
failure rolls the whole delivery back instead of reporting a partial well as success. The
real-delivery gate retained exactly 91,392 and 27,857 source depth rows; the first like-for-like
debug measurement fell from 89.6 s to 61.2 s. That measurement is evidence of improvement, not a
shipping-time promise: disk cache and primary-key maintenance still move the wall time, so the field
checklist measures the release build on the target machine.

**Declared STEP is audited before f32 storage.** Adjacent source depth tokens and the declared STEP
are compared as exact decimals; missing/unparseable depth breaks adjacency. Deep measured depths
therefore cannot acquire a false “possibly re-gridded” warning from f32 rounding, while a genuine
source-token mismatch remains named with its first row pair.

### One Normalize, for any curve

`condition::normalize` — any curve, three methods (percentile pair / min-max / z-score), LINEAR or
LOG space. Jauhar, 2026-08-05: *"dont dupilcates, normalize tools here should be universal for all
logs"*. `gr_normalize` DELEGATES to it and is hidden from the pickers
(`Ribbon.SUPERSEDED_MODULE_IDS`, `DEPRECATED_STEP_MODULES`) — **still runnable**, unlike the
retirement list in `modules.rs`, because the answer is unchanged and retiring it would fail every
saved chain carrying the step.

**The reference pair has no default and the run refuses without one**: a pair from one basin is the
wrong pair in another, and normalized output looks plausible either way. MEAN_SD is the deliberate
exception — mean 0, spread 1 is a definition. LOG works in log10 and inverts, which is the honest
frame for a resistivity: three decades mapped linearly onto 1–100 put the geometric middle at 4
instead of 10. Non-positive samples stay MISSING rather than being floored onto the low reference
the whole map is anchored on.

Found while writing it, and worth remembering: **`distribution::percentile` takes an
ALREADY-SORTED slice.** The first version handed it samples in depth order and returned whatever
sits 3% of the way down the WELL.

### Intake replaces the table-shaped importers

One pane (`intake.rs` + `intakePanel.ts`) for any delimited text. **An extractor and a front end,
never a second write path**: it produces a `CoreMapping` and calls `ingest::import_core_table`, the
plate-workbook precedent. Four rules: nothing is sniffed the user can state; the decimal convention
is the workbook reader's (rightmost separator wins, `1,234` read as a decimal and flagged); **a
column with no role is CARRIED, never dropped**; and the preview is a CHECK — every cell in a
numeric column that did not parse is flagged before anything is stored.

**Import Aux… is gone** (Jauhar: *"for other aux delete it, except core and scal"*). Two things had
to close first, and the second was destructive: `follow_core` was on the form and on the IPC struct
and `import_core_table` never took it, so the setting was dropped in flight; and **a table claiming
no core measurement still went through `insert_core_data`, which registers its set and makes it
ACTIVE** — so importing XRD or CEC through Intake replaced the well's real plugs with a set of
empty ones, and every core reader follows the active set, so the φ-k cloud, Plug QC, Register Depth
and the S-factor fit would all have gone quiet at once.

**LONG / WIDE / BLOCK is DECLARED, never sniffed.** A wide table and a long one are both rectangles
of numbers, and reading a long Pc table as wide would store a capillary-pressure curve made of
column indices. WIDE is one row per sample with the HEADER ROW as the axis; BLOCK is stacked tables
with the header repeated, which once stripped is the file it came from — a pre-pass over either of
the others rather than a third way of reading a table. **A header that is not a number is dropped
BY NAME** (a `TOTAL` column counted as a bin is a saturation at an invented pressure, at the end of
the curve where a Thomeer fit is most sensitive), and **without the block flag a repeated header
survives as a real-looking sample whose values are the axis numbers** — only its absent depth stops
it being stored, which is luck rather than a guard. A block keyed by a LABEL LINE rather than a
column is reported and left unread: which token is the depth cannot be told from which is the plug
number.

`array_logs` gained an **`axis` BLOB** (`db::migrate_array_log_axis`, ADD COLUMN, last column) —
what each stored value is a measurement AT. NULL keeps its old meaning: a Monte Carlo realization
is not a measurement at 7 of anything.

**The `CURVE` role** writes a column to the generic curve store, the route a delimited file of logs
had no way in by. It is never PROPOSED, only chosen: a column of numbers at depths is a plug
measurement or a logged curve depending on how the file was sampled, and nothing in the numbers
says which. Family and unit come from the delivered mnemonic and are left ABSENT where it is
unrecognised.

**An import never eats a delivery** (Jauhar: *"dont eat it, thats why i request user can define
their intake cons, so it wont eat anything"*). Every set name auto-suffixes per well —
`free_array_set` and `free_curve_set` join `resolve_core_set_name` and its siblings. This matters
most for arrays, where `db::write_array_log` REPLACES by design: right for a Monte Carlo re-run,
which must never union two runs' realizations, and wrong for an import.

**Saved mappings** (`intaketmpl` documents) are applied by column **NAME, never by position** — a
delivery that gains a column would otherwise shift every role one to the right, silently, and a
saved mapping exists precisely for the deliveries nobody re-checks. Columns the mapping does not
name keep their proposed role and are listed.

### The one word for a log set

The store, the backend, the docs and every other package say **log set**; the UI alone said
"constellation", abbreviated to "cons", which is why Jauhar could not map it onto anything he had
read. `src/ui/logSetPicker.ts` is now the ONE input/output control and every tool that reads or
writes a curve carries it — before this, two surfaces of nineteen offered one, and three writers
hardcoded where their output went. The input picker is a strict dropdown and the output an editable
combobox: you can only read from a set that exists, while naming a new one is the ordinary act.
A name already in use gets a new VERSION of that same set (`create_log_set`), which is what Jauhar
described: *"it can replace with version number for logs, but for cons stiil same"*.

### The two remaining core-photograph items (2026-08-05)

**`CPHOTO_LITH`** — a two-class cut of the darkness trace, 0 lighter and 1 darker, proposed by
**Otsu on this core's own trace** (a method, not a calibration) or at a cut the user gives. The
codes are ORDERED for `facies.rs`'s reason: a class curve whose numbering could swap between wells
is not correlatable. **It will never be `VSH` or `LITH`** — the same dark band is organic-rich
mudstone in one core, oil stain in another, and a curve under a name every module reads as
lithology would be an uncalibrated answer that computes and plots. Refused under UV, where the
brightness IS the fluorescence; refused on a core of one lithology rather than inventing a contact
through the middle of it. Deliberately unsmoothed — suppressing flicker needs a bed thickness and
no value is right in two cores, so the run points at Frame ▸ Block with `OPT_STAT = MODE`.

**`CoreLogSpec.unfold`** — the dipping-bed correction, stated as **the depth DROP across the core's
width, not an angle**. An angle needs the core's diameter to become a pixel shear and nothing here
stores one; the drop is read off the picture by noting a contact's depth at each edge. Sent to the
runner as a FRACTION of each lane's own depth span so it becomes pixels against that picture's own
height. **Rows sheared in from beyond the lane are MISSING** — never the edge row repeated, never
wrapped from the other end of the barrel — which is why the slab statistics are nan-aware.
Pinned by `unfolding_a_dipping_bed_sharpens_a_contact_the_slab_average_would_smear`, which runs
real pixels and requires the ramp to collapse to under a third of its width **while both plateaux
read the same**: a sharpness check alone would pass on a run that had quietly rescaled the trace.

## The three things that were still open (2026-08-05)

Jauhar, on the completion report: *"solve em"*. Each had been left as a stated limit rather than a
defect, and each is now closed.

### A block keyed by a label line (`intake.rs`)

A per-plug delivery writes `PLUG 12  4633.5 ft` above each block instead of carrying the depth in a
column, and Intake reported that and read nothing. **The answer was already in this repo**:
`images::WORKBOOK_RUNNER` met the same problem on a plate sheet's header cell and settled it by
requiring a UNIT. `depth_from_label` borrows the rule whole — the depth is the number carrying a
unit and no other. Taking the first number would read `PLUG 12` as 12 ft; taking the largest fails
the moment a laboratory numbers its plugs into the thousands. A caption whose numbers carry no unit
is still refused BY NAME.

**A label line is reassembled with the DELIMITER, never with a space**, and that is the whole of why
`split_table` now returns the separator. In a comma-delimited file `4640,0 ft` arrives as two cells;
joined with a space the reader is handed `4640 0 ft`, where the number carrying the unit is ZERO —
the plate workbooks' comma-decimal failure exactly, which put a seventh of one delivery at 54 feet on
rock cored at 7,000. Pinned from both sides in `a_labels_depth_is_the_number_that_carries_a_unit`,
which asserts what the space would have given so nobody "simplifies" the join back.

**A unit is a WORD.** Trimming non-letters off both ends makes `2103.4M` read as the unit `M`, so the
plug number before it becomes the depth — the one mistake the rule exists to prevent. Found by the
test, not by reading it.

A label line is identified by what it PARSES as (fewer than half the axis columns read as numbers),
not by its length: written into a delimited file a caption usually keeps the delimiters and arrives
full width. Rows above the first caption keep NO depth rather than taking the block below them. A
DEPTH column still wins where a file carries both — it is per sample, the caption is per block.

The control in `a_label_line_keys_its_block_by_the_number_that_carries_a_unit` is the important half
and it is worse than a refusal: read without the block flag the captions parse as nothing across
every bin, so the all-MISSING rule drops them silently and both blocks import with no depth at all —
which looks like a clean read of a delivery whose plugs simply never had depths.

### A plug sits at one depth (2026-08-05)

The open question was what a caption naming an INTERVAL should do — first, mid-point or top. Jauhar
settled it by rejecting the premise: *"it should be 1 plug number only, should warn user if
duplicate"*. A caption keys one plug and a plug sits at one depth, so a second depth is a
**duplicate**, not a range to choose an end of. The first is still used — discarding the block over
a caption a laboratory very likely typed twice would lose real data — and the run says so.

**The stakes are `array_logs`'s PRIMARY KEY** `(well_id, set_name, curve_name, depth)`: ONE stored
vector per depth, so a second sample at the same depth is a constraint violation that fails the
whole curve's write, with a raw engine message naming nothing the user put in the file. Every case
below imports cleanly right up to the moment it does not.

Three shapes, one rule, and the row-level check is the general one:

- **Two captions claiming one plug** — reported as a CAPTION problem, because that is where the fix
  is. `read_label_keys` returns `LabelKeys.repeated`.
- **One caption carrying several rows** — the same collision from inside a single caption, and the
  likelier delivery mistake of the pair. A caption check cannot see it; a row check can. This is
  what the existing label-line fixture had been describing all along: two rows under `PLUG 12`,
  both keyed 4633.5, i.e. a file that could never have been stored.
- **A DEPTH column with repeats** — caught by the same row check, with no second rule.

**Each is reported ONCE.** The row-level scan SKIPS depths the caption check already explained, so
two blocks at one depth get one message naming the cause rather than two describing it from both
ends. **Grouped by the file's own well column**, because two WELLS sampled at one depth is entirely
ordinary — a check that ignored the well would fire on every multi-well delivery, which is the
fastest way to train a user to ignore the message. The control in
`a_plug_sits_at_one_depth_and_a_duplicate_is_named` pins the silence on a clean file for the same
reason.

The result line goes `var(--warn)` when a duplicate is named, because it contradicts the sample
count printed next to it.

### The wide/block preview (2026-08-05)

`intake::probe_arrays` + `intake_probe_arrays`, closing the gap the duplicate check exposed: the
LONG path had a preview since it shipped and the array path had none, so a duplicated depth — which
the store REFUSES — was only named once the import had run and half-written. **The same `read_wide`
the commit runs**, so the preview cannot disagree with the import about what the file says; only how
much comes back differs.

**It shows what reading the file AS an array made of it**, which the raw grid above it cannot: for a
block file the depths come from captions the grid draws as ordinary lines, so without this there is
nothing on screen saying a caption was understood. The header row's parsed axis is shown beside the
TEXT it was read from — `100 psi` reading as 100 is obviously right once seen and uncheckable
otherwise.

**`ARRAY_PREVIEW_ROWS` is 40 against the long path's 200, and the difference is not a preference.** A
long row is a handful of cells; a wide row is the sample's whole distribution, so an NMR export is a
hundred bins per row across thousands of rows — shipping it to draw a dozen visible lines makes the
preview cost more than the import it precedes.

**The cap governs what is DRAWN, never what was checked**, and a duplicated sample beyond it is
pulled in anyway. A preview that stopped at its cap would be most useless on exactly the delivery
that needs it — a big export nobody scrolls by hand, whose duplicate sits at row 900. Each drawn row
carries its index IN THE FILE (`row_index`), so a row fetched from beyond the cap cannot read as
following the one above it, and `n_rows` stays the file's own count — a preview reporting its capped
length would say a 4,000-sample delivery held 40. Pinned with its control by
`the_preview_counts_every_sample_and_draws_every_duplicate`.

**`DepthClash` travels as DATA, not only as prose in a note.** A warning naming `4633.50` is
actionable only if the rows it means can be found, so the preview tints them — across the WHOLE row
rather than per cell like `.intake-bad`, because the fault is not in any one value: the row
duplicates another row, and it is the row that cannot be stored.

### The array write is one transaction, and a duplicate is refused before it (2026-08-05)

Found while writing the duplicate check: `db::write_array_log` is DELETE-then-append and was doing
it **outside a transaction**, so a failure part way through the inserts committed the delete and kept
only some of the new rows. Not a visible breakage — a realization matrix quietly missing depths, and
every percentile read off it then computed from a different population than the one that was run. It
now uses `db::with_txn`, whose own doc names this exact hazard; the writer simply predated its use
here. Neither caller (`montecarlo::persist_realizations`, `intake::commit_arrays`) is inside a
transaction, so there is no nesting — DuckDB has none.

**A duplicated depth is refused BY NAME before any of that**, in `db.rs` rather than in the pane, so
it protects every caller and not only the one whose front end happens to check. The engine's own
constraint message names an internal table and no depth, arriving on an import the user was just
told had succeeded. Checked over the rows that would actually be INSERTED — a depth whose vector is
empty is skipped by the writer and never reaches the table, so counting it would refuse a write the
store would have accepted.

`a_refused_array_write_leaves_the_stored_curve_untouched` pins the refusal and its message, and its
doc records what it does NOT pin: the refusal short-circuits before `with_txn` is entered, so nothing
tests the rollback. The transaction is there for what no pre-check can foresee — an unclean kill, I/O,
a constraint added later. It earns its place by being used.

**And the preview settles whether a BLOCK file has depths at all**, which fixed a bug that made the
label-line feature unreachable: `validate()` required a DEPTH role, a caption-keyed block file has no
depth column by definition, so the reader resolved every block correctly and the Import button stayed
disabled. Whether the captions actually yielded depths is a fact about the FILE, not about the roles,
so it is read off the preview. The refusal text is layout-aware — a block file is told it may caption
its blocks instead. **One-way**: `renderArrayPreview` calls `validate()`, and `validate()` must never
be what triggers the fetch, or it is a loop with a file read in it. Stale answers are dropped by
sequence number, the `poreAreaDialog` rule — a role click can outrun a file read, and a preview of
the mapping before last is a picture of a decision already changed.

### A minimum bed thickness for `CPHOTO_LITH` (`coreimage.rs`)

`lith_min_bed`, **no default** (`param_open`'s rule): a minimum bed thickness is a statement about
the rock and about what the study is for — 5 cm of shale is a bed a core description records and a
flow simulator never sees — so a shipped value would silently rewrite everybody's lithology. Blank
keeps every flicker and says so.

Counted in SAMPLES (`thickness / step`), which is what makes a barrel gap harmless: unphotographed
metres contribute no samples, so a bed either side of a gap is not credited with the gap.

**Thinnest first, runs rebuilt after every absorption.** Absorbing a bed merges the rock either side
into one thicker bed, which can lift a neighbour above the threshold — a single sweep keeps going
and strips beds that had become legitimate. Thinnest-first is also the only order giving the same
answer from either end of the core.

**A stretch isolated between two MISSING gaps is LEFT and counted.** It is a short barrel, not a
flicker: there is no neighbouring rock to absorb it into, and flipping it would invent a lithology.

### The unfold proposes a dip (`coreimage.rs`)

`unfold_scan` — the widest drop to search — returns an `UnfoldScan` and **nothing is applied**.
`registration.rs`'s contract: the whole scan comes back, the peak is a proposal, the user types it
in. One sharp peak means the dip is determined; a flat scan means the core carries no bedding
contrast to find one from and the maximum is whichever candidate noise favoured; a comb means the
section repeats. All three return a number, and only the SHAPE tells them apart — so the pane draws
it, with the score axis anchored at zero because cropping to the data makes a 2% wobble fill the box.

**The score is the trace's own contrast.** At the true dip every slab is one rock, so a contact comes
back as a step and the spread is greatest; shear too little or too much and the same contact is
averaged across, which fills the middle in. Peaked at the truth, falling away either side — a
correlogram's shape, for a correlogram's reason. Scored per LANE, because the mean darkness genuinely
differs between barrels and pooling them adds a between-barrel spread no shear can change, diluting
the peak with a constant.

**A candidate must keep 75% of the best-populated candidate's live samples** (`UNFOLD_MIN_COVERAGE`,
the `MIN_PAIR_FRACTION` argument). Shearing empties every lane's corners, so the widest drops read
the fewest slabs — and a handful of slabs can be spread wide by chance, which would make sliding the
core off its own frame the winning move. An unscored candidate is drawn as an EMPTY slot, not a short
bar: "not tried" and "tried and poor" are different statements.

**One lane reader in the runner, shared by the measurement and the scan.** Two copies of the shear
would be two things to keep in agreement, and a proposal computed by a shear the run does not apply
is a number that looks right and is not. One decode, every candidate, first plane only.

Pinned pure by `the_unfold_scan_proposes_a_peak_and_refuses_a_flat_one` (peak, flat, starved
candidate, peak-at-the-edge) and end to end by the round trip, which recovers the 1 m dip that was
drawn and — the half that matters — proposes exactly 0.0 on the same picture with a horizontal
contact.



## A missing inclination is not a vertical station (2026-08-20)

Codex whole-repository review, P1. `parse_deviation_csv` made INC and AZI OPTIONAL and replaced
every gap - an absent column, a blank cell, an unparseable one - with **0**, i.e. with a measured
vertical/north station. So a cell lost in export straightened the well, and because minimum
curvature integrates station to station, every TVD below it moved with it. The result is finite,
plausible, persisted through `well_path`, materialized onto the log grid as a normal TVD curve, and
read by saturation-height as the height above the contact. Documenting the coercion in the doc
comment - which it was - does not make *not measured* equivalent to *measured vertical*, and no
caller ever asked anyone to confirm the substitution.

**The remedy is Jauhar's, and it is better than the one first shipped.** The first pass REFUSED the
whole survey. He asked whether the station could be filled in from its neighbours instead, and the
answer is that it does not even need filling in: **minimum curvature already draws a circular arc
between consecutive stations**, so simply LEAVING OUT a station with no geometry draws that arc
between the neighbours that were actually measured. On a constant build that reproduces the full
survey exactly - three stations at 0, 30 and 60 deg over 2000 m give TVD 1653.99 m, and so do the
two survivors alone - while substituting 0 deg gives 1826.99 m, missing by 173.01 m. Dropping
degrades only in proportion to how non-uniform the real build was and how far apart the survivors
are; substituting asserted a vertical station, which is wrong essentially always.

So a station carrying no usable geometry is **dropped and REPORTED**, never substituted and never
fatal. `DeviationSurvey.dropped` carries each one's MD and reason out of the parser,
`import_deviation_csv` turns them into `CoreImportResult.warnings`, and the import pane stays OPEN
showing the warning instead of closing on success - a survey that quietly got shorter is its own
silent failure. The warning also lands in the process history, so it is still findable after the
pane is dismissed. Two refusals survive, both because there is nothing to draw an arc between: an
absent INC COLUMN, and a file where not one station carries a usable inclination.

**The azimuth half is deliberately narrower, and the narrowness is algebra rather than leniency.**
`minimum_curvature` reaches azimuth only through `sin(i1)*sin(i2)*cos(a2-a1)` in the dogleg term,
so a station declared exactly vertical multiplies its own azimuth by zero and the value cannot
reach the answer. That is what keeps the commonest survey there is - a vertical well delivered as
`MD,INC` with no azimuth column at all - importable whole. A blank azimuth at a station that is
actually deviated is missing geometry and costs that station. **No tolerance is invented**:
exactly-zero is the declared-vertical case, because a threshold would be a silent decision about
how vertical counts as vertical, which is the same class of decision this whole fix exists to stop.

Pinned by `a_survey_station_with_no_inclination_is_dropped_and_reported_never_read_as_vertical`,
which carries all six arms plus both TVD figures computed rather than quoted, so neither can
quietly move without the test saying the survey geometry changed.
