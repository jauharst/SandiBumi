# Intake, Statistics, Condition & Frame — the plan

**Status (2026-08-05): all four shipped.** Condition, Frame, Statistics and Intake, plus the
log-set sweep that runs underneath all of them. Named and deferred: Intake's Wide/Block array
layouts, its Curve role and saved templates; Frame's Resample/Regularize (a well-level re-frame,
not a module).

Three tool families asked for on 2026-08-05, scoped in a twelve-question round with Jauhar the same
day. His answers are recorded here verbatim in effect, because several of them overrule the
recommendation I gave and the reason matters more than the choice.

The three are reference-suite shaped in origin (a table importer, a statistics list, a log-edit and
sampling list) and deliberately not reference-suite shaped in delivery. **Our own names throughout**
— Jauhar's instruction — and, where this repo already answers a need, the existing answer is
extended rather than reimplemented.

---

## 0. What the recheck found

Jauhar asked whether log edit and sampling already existed. Inventory taken 2026-08-05:

**Present before this work.** `curve_edit.rs` (interval shift / set / blank / interpolate / scale,
undoable, from the log view's right-click); the `depth_shift`, `splice`, `gr_normalize` and
`log_predict` modules; `coreimage.rs`'s box-average resampler; `distribution.rs` (percentile, Tukey,
histogram); `plugqc.rs` (pairing, Pearson, Spearman).

**Absent entirely.** Despike, smooth, clip, fill-missing, flip polarity, gain/rescale, block,
resample, regularize, reverse/sort. Confirmed against `modules::list_modules` rather than a text
search, because most module specs live outside `modules.rs`.

**Statistics.** No table-producing statistics tool of any kind. Plots compute statistics; nothing
hands back a grid of numbers per well x zone x curve.

**Import.** Five separate importers (LAS/DLIS sets, core tables, aux point data, SCAL,
tops/locations), each with its destination store baked in. None can route one file's columns to two
stores, and none can read an array.

---

## 1. Decisions (2026-08-05)

| # | Question | Answer |
|---|---|---|
| 1 | Importer: replace the existing dialogs or sit beside them? | **Replace the table ones** — core, aux, SCAL, tops, locations. LAS/DLIS keep their own path. |
| 2 | Which array layouts arrive? | **All three, user-selected**: *"can be customized based on data, either long, wide, or block"*. |
| 3 | How are Condition & Frame delivered? | **Modules first, preview pane after.** |
| 4 | Thickness: extend the pay summary, or its own tool? | **Its own tool** — *"since we talk about thickness not only in pay summary"*. |
| 5 | Importer name | **Intake** (Condition and Frame confirmed for the two curve categories). |
| 6 | Despike rejection rules to offer | **All four**: Hampel, absolute threshold, plain median, rate-of-change. |
| 7 | What may the thickness tool count? | **All four**: a flag curve, a class curve, a cutoff on any curve, marker to marker. |
| 8 | Build order | **Condition & Frame first.** |
| 9 | What is a spike, in thickness terms? | **No default — set every run.** |
| 10 | Where does a conditioned curve land? | *"user should be able to define his own name"* — one `OUT` field, defaulting to `<input>_C`. |
| 11 | Fill Gaps | **A limit the user sets, and every filled sample flagged.** |
| 12 | What defines a bed when blocking? | **All four**: fixed interval, flag/class runs, marker to marker, boundaries found from the curve. |

### Two answers that changed the design

**#4 — thickness is not a pay-summary question.** Gross sand by facies, coal thickness, a
marker-to-marker isopach: none of those go through cutoffs. The tool is therefore its own, and the
drift risk does not vanish, it moves: **the thickness tool COUNTS a condition, it never re-derives
one.** Where the condition is pay it reads the `FLAG_PAY` curve the cutoff engine already wrote. One
definition of net, one general thickness engine.

**#10 — one control serves both options I offered.** A blank `OUT` writes `<input>_C` into a new
versioned log set; a typed `OUT` puts the conditioned curve beside the original under its own name.

---

## 2. Condition — SHIPPED 2026-08-05

`src-tauri/src/condition.rs`, ribbon category **Condition** ("Curve Conditioning"). Five modules:
`despike`, `smooth`, `clip`, `fill_gaps`, `flip`.

### Why modules and not an editor

The reference suite keeps its log-edit family in a separate launcher because it has no module
framework. SandiBumi does. A Rust fn plus a manifest buys multi-well rayon-parallel runs,
zone-overridable parameters, workflow chaining, the universal run mask, log-set versioning with
provenance and an auto-generated Organic-styled dialog — none of which a bespoke editor would have
on day one, and all of which would have to be rebuilt inside one. `curve_edit.rs` stays the
interactive single-interval path; this is the batch path.

### The four family rules

1. **A window is a THICKNESS, never a sample count.** Resolved against the curve's own depth column
   (`Frame::windows`, a two-pointer sweep over the finite-depth samples). A sample-count window
   silently covers different amounts of rock after a resample, or between two runs of one well
   logged at 2 inches and 6.
2. **Nothing invents a sample except Fill Gaps, which says so.** Smoothing never bridges a gap; a
   MISSING sample stays MISSING. Fill Gaps is bounded by a user-set maximum and marks every sample
   it wrote in `<OUT>_FILL`.
3. **The output is never the input's own mnemonic.** `equations::fetch_curve_frame` resolves the six
   standard mnemonics from `standard_curves` FIRST and only falls through to `computed_curves` when
   the standard column is entirely NaN — so a despiked curve stored as `GR` would be written,
   counted, reported and invisible to every reader. Same shape as the `CPHOTO_*` trace written at
   the wrong sampling. `out_name` refuses such a name BY NAME with the fix.
4. **A parameter with no generic value has no default** (`modules::param_open`). The despike window
   and the gap limit open empty; required ones are refused in the pane, optional ones (Clip's
   MIN/MAX) read a blank as "no bound on this side", which is a statement rather than an omission.

### MAD implodes, and the fix is load-bearing

The textbook Hampel test compares a sample to `K x 1.4826 x MAD` over its window. **The MAD is zero
whenever more than half the window is identical** — a quiet interval, a coarsely quantized curve, a
tool on its rail — and a single spike among identical neighbours is exactly that case. So the
classic implementation finds nothing on the cleanest possible example of the thing it exists to
find. `window_spread` falls back to the MEAN absolute deviation about the same median, which
collapses only on a window that is constant including the centre, where there is genuinely nothing
to reject. Pinned from both sides by `a_spike_in_a_quiet_interval_is_still_a_spike`, whose control
requires a constant window to reject NOTHING — without it the fall-back would have moved the failure
to the other extreme and eaten every flag curve in the project.

`MIN_HAMPEL_SAMPLES` (5) is the second half. Over three samples the spread is a third of the spike
being judged, so `K = 3` lands precisely on the boundary and the answer is a rounding bit. The run
**refuses** and points at ABS, which needs no spread estimate.

### Other decisions worth not re-litigating

- **Clip defaults to BLANK, not CLAMP.** A resistivity of 1e6 is not very resistive rock, it is a
  reading the tool could not make; pinning it to the bound leaves a real number where there is no
  measurement. CLAMP is for a small arithmetic overshoot of a known physical limit, the way
  `PHIE_FLOOR` works.
- **A reversed MIN/MAX pair is refused, not swapped** — the `plateDepthDialog` rule.
- **SAVGOL fits the real (depth, value) pairs**, not the textbook fixed coefficients, which assume
  even sampling and are wrong on a spliced or depth-shifted frame. Depths are taken relative to the
  window centre or the normal matrix loses most of its precision at 3000 m.
- **Fill Gaps never fills a gap open at one end** — that is extrapolation past where the tool
  stopped. The gap is measured between the LIVE samples either side, which is what actually went
  unmeasured.
- **Flip records the pivot it used** in `<OUT>_PIV`, because MIDRANGE and MEAN are per-well and two
  wells' flipped curves are then no longer on a common scale.
- **`ArgKind::Text`** was added to the manifest framework for the user-named output. It travels in
  `opts` exactly as an Option does; `workflow.rs`, `montecarlo.rs`, `moduleDialog.ts` and
  `workflowDialog.ts` all take both kinds on the same channel.

### Not in this increment, deliberately

**Rescale/Normalize** — it overlaps `gr_normalize` and belongs beside it rather than as a sixth
Condition module that quietly does the same two-point map with different words.

---

## 3. Frame — SHIPPED 2026-08-05

Category **Frame** — the word this repo already uses for a curve's depth grid ("resamples onto the
frame the rest of the project reads"). Members: **Block**, **Resample**, **Regularize**, **Align
Multi-well**. Reverse/Sort do NOT belong here: a non-monotone depth column is an import problem and
is fixed in Intake.

Rules already decided:

- **Coarsening is a box AVERAGE, never an interpolation** — `extract_core_log`'s lesson. Linear
  interpolation onto a coarse frame aliases a lamination into a trend that is not in the rock.
- **A blocked curve is written with `draw_style: "step"`** (already supported by both renderers). A
  diagonal between two block averages draws a gradient the data never measured.
- **Block takes all four bed definitions** (decision 12). Boundaries found from the curve itself are
  INFERRED and the run must say so.

---

## 4. Statistics — SHIPPED 2026-08-05

Five tools; four of them reuse machinery that already exists.

| Ours | Reference-suite name | Reuses |
|---|---|---|
| **Curve Summary** | Single Log Statistics + Min Max | `distribution.rs` unchanged |
| **Pair Summary** | Dual Log Statistics | `plugqc.rs`'s pairing, generalized from plugs to curves |
| **Fit** | Simple + Multiple Regression | one tool, 1..n predictors, blind-well CV, optionally saved as an `ml_models` artifact |
| **Versus** | Compare | two log SETS against each other — the first consumer of log-set provenance |
| **Thickness** | Accumulate, Interval / Lump Thickness | counts a condition; reads `FLAG_*` rather than re-deriving net |

Every one emits the workbook's `Sheet`/`Cell` model, so a statistics table reaches Excel, Word, PDF
and the deck with no second implementation — and a blank stays blank.

---

## 5. Intake — SHIPPED 2026-08-05 (long layout)

One pane replacing five dialogs (decision 1). The grid IS the control: click a column header to
give it a role and the column tints, as in the reference screenshot. Roles: Well, Depth (top/base),
Curve, Plug property, Point item, Array, Ignore — and different roles in one file are fine.

- **Long / Wide / Block is DECLARED by the user** (decision 2), not sniffed. They are three
  row-grouping rules over one parsed grid, not three parsers: Long groups by a key column, Wide
  reads the header row as the array's axis, Block detects a repeated header signature. The mapping,
  units, depth handling and delivery-set rules are identical underneath.
- **The preview IS the commit** — one Rust path renders the grid and writes the rows, the pore-area
  rule.
- **Every guess is a visible, editable proposal with its reason** — header rows, units row, decimal
  convention, depth unit, percent vs fraction.
- **The comma-decimal rule moves into the shared parser.** `WORKBOOK_RUNNER::as_number`'s
  rightmost-separator rule already exists because one delivery wrote both `6980.71 FT` and
  `7016,54 FT`; a delimited core table can do the same.
- **One file = one named delivery set**, auto-suffixed, never overwriting, with the follow-core
  tick-box from `followCore.ts`.
- **Templates are readable documents keyed by header name**, the `platelabels` precedent.
- **Performance**: parse and column statistics in Rust returning a capped preview; the grid is
  virtualized.

---

## 6. Still parked

The discrete sand/shale curve off the white-light core trace (`CPHOTO_LITH`) and the dipping-bed
unfold, both from the UV round. Not dropped — see `docs/plan_core_photo.md`.


---

## 7. The log-set sweep (2026-08-05)

Jauhar, in the same session: *"each tools or modules should give user freedom to define input and
output log set ... and their own curves"* — and, in the same breath, *"i forgot what set refer to
in sandibumi"*. Both halves were real defects.

**The word.** The store, the backend, `ROADMAP.md` and every other petrophysics package say **log
set**; the UI alone said "constellation", abbreviated to "cons" on the only two dialogs that
offered one. A user cannot map that onto anything they have read about their own project. One word
now, everywhere. Nothing about the data model changed.

**The freedom.** Exactly TWO surfaces of nineteen let a user pick a version. ML, SandiMin, the
saturation-height fit, the cutoff sweep, the pay summary, the facies tie, Lorenz, results QC, the
report, the workbook and the deck all read whatever the current values happened to be — and the
three that write curves hardcoded where the output went (`ML`, `SANDIMIN`, and the core photograph
traces had no log set at all). None of that shows up in a result: a report quoting last week's
porosity looks exactly like one quoting today's.

Every one now takes `input_set`; every writer takes `output_set` defaulting to what it used to
hardcode, so an older payload behaves identically. `src/ui/logSetPicker.ts` is the ONE control —
the `followCore.ts` argument — and `equations::tests::every_curve_consuming_request_still_offers_a_
log_set` reads the source and fails if a request struct ever loses the field, which would compile,
run, and silently revert the tool to current values.

**Their own curves.** Modules already chose their INPUTS. Outputs were fixed, and mostly should be
— a module producing `VSH` is producing an answer whose name is part of the answer. So the general
form is a run-wide **output prefix** (`workflow::OUT_PREFIX_OPT`), handled once in the runner for
the reason `MASK` is, letting a whole trial run land as `TEST_VSH`, `TEST_PHIE` beside the
interpretation the field is using. Condition and Frame keep their own per-curve `OUT` field, which
is the right shape there because those modules produce a COPY of an input rather than an answer.

**Monte Carlo REFUSES a prefixed step, by name.** Its plan builder resolves cutoffs and fraction
curves from the manifest's declared LogOut names, so a prefixed run would be planned against
curves it never writes and the study would return plausible percentiles computed from nothing.
