# Reading a core photograph

Build record for core slab photography: conditioning, the CPHOTO traces, the depth strips, white
light against ultraviolet, and the lane lay-out a packed core-display plate needs. The phase plan
is `plan_core_photo.md`.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## Core slab photographs: conditioning, and a trace read off them (2026-07-31)

`coreimage.rs` + `coreConditionDialog.ts` (Advance ▸ Core Imaging ▸ **Core Photos…**) are ROADMAP
C2 item (7)'s first two halves. A core photograph arrives as somebody's snapshot — the box a degree
off square on the bench, the tray and the tape in frame, and whatever colour the core shed's lights
had that afternoon. None of that is the rock and all of it goes into a report.

**The controls are the picture wherever they can be** (Jauhar, 2026-07-31: "geologist see image not
text"). The delivery is a strip of thumbnails rather than a list of filenames, the crop is a drag on
the image rather than four numbers, the white balance is a click on a grey patch rather than three
gains, the depth lay-out is a row of buttons showing every option at once, and each slider's TRACK
carries the gradient it moves along — blue to amber, green to magenta, grey to vivid. The readout
beside a slider is there to be read back, not typed into.

### The conditioning

**Non-destructive, and `well_images` enforces it rather than claiming it.** `recipe` holds the
settings, `source_data` the un-conditioned display copy — written ONCE, by a `COALESCE` inside the
UPDATE rather than a read-then-write, so two applies in flight cannot let the second file the
first's output as the original. Every later edit re-renders FROM it: editing a recipe must never
stack a second correction on the first, because a brightness raised twice by eye is a photograph
nobody can get back to.

**`source_meta` (`WxH;mime`) is the third column and it is not decoration.** A crop changes the
picture's shape, so a restore that left the baked dimensions behind would have every renderer draw
the whole photograph into the cropped one's box, at the wrong aspect ratio — the one thing this app
never does to a picture. Pinned by `conditioning_keeps_the_import_and_a_restore_puts_back_its_shape`.

**The result is BAKED into `data`, not applied when the picture is drawn.** The PDF exporter embeds
those bytes untouched through a `/DCTDecode` XObject, so a render-time recipe would print the
unconditioned photograph while the screen showed the corrected one — silently, and only on the
deliverable. Baking also leaves the log view, the composite and the PDF nothing to disagree about.
A recipe that changes nothing RESTORES rather than re-encoding: a second JPEG pass to record a
decision to leave the pixels alone is pure loss.

**Everything geometric is a FRACTION of the picture.** A crop in pixels belongs to whichever copy it
was dragged on, and the stored copy is already capped at a long edge — the `fov_um` and scale-bar
argument again. It is also what makes the preview trustworthy: the proxy the user drags on and the
full-size bake apply the identical recipe, checked by shape in
`a_picked_grey_a_crop_and_a_way_back`. A second crop COMPOSES with the first, because it was drawn
on the already-cropped picture.

**The picked white balance is normalised so the LARGEST gain is 1** — it can only darken, and no
channel is pushed past white and clipped, which would distort the hue of exactly the brightest
pixels. The patch is a MEDIAN, not a mean: a speck of dust or a highlight on the tray is one pixel
from the grey that was actually clicked. Same rule the thin-section colour correction follows.

**"Apply this light to the whole run" copies the colour half only**, and the merge is done in Rust
(`CoreRecipe::with_look`) so what "the look" means is one rule rather than one per caller. A
core-shed run is shot under one light in one afternoon, so the colour genuinely belongs to the
delivery — but the box sits differently on the bench in every frame, so the crop and the deskew do
not. Same reasoning as `set_image_delivery_details` refusing "All datasets".

**The preview comes from the backend.** Re-implementing the pipeline in canvas would drag faster and
would put one correction in two languages — the standing `composite.rs`-versus-renderer warning.
What is tuned is literally what gets baked, at a smaller size. Slider moves are coalesced and stale
answers dropped by sequence number.

The dialog distinguishes THREE states, not two: as imported / applied / edited-and-not-yet-written.
"Conditioned" and "conditioned in the project" are different facts and the second is the one that
reaches a report — the status line read "not yet applied" the moment after Apply until this was
fixed. The filmstrip dot follows the PROJECT, never what is being tried on screen.

### The trace

`extract_core_log` reads three measures down the core and can write them as curves:
`CPHOTO_DARK` (1 − Rec. 709 luma), `CPHOTO_RED` (normalised (R−G)/(R+G), so an uneven lamp cancels
in the ratio) and `CPHOTO_TEX` (spread across the core within each slab — lamination and
conglomerate scatter, a clean massive sand does not).

**The prefix is `CPHOTO` and it will never be `VSH`.** Darkness co-varies with shale in most clastic
sections, which is not the same statement as being a shale volume: the same dark band is
organic-rich mudstone in one core, oil stain in another, a wet patch in a third. A curve called VSH
is read by every module downstream AS a shale volume, and an uncalibrated one under that name is a
wrong answer that computes and plots. Turning it into one is a calibration the user makes against
their own GR. Same argument that keeps `GRAIN_D50_APP` apart from `GRAIN_D50`.

**It reads the CONDITIONED picture**, which is why the conditioning came first: a darkness compared
across boxes shot under two different lamps is a comparison of the lamps.

**The agreement with a real log is SIGNED, and that is the point.** Darkness and GR should both rise
into shale, so a strongly negative `CPHOTO_DARK` is a finding rather than a weak result — most often
the depth axis is the other way round, occasionally the dark bands are oil stain. Below −0.3 the run
says so by name and suggests Deepest first. Pinned from both sides by
`the_trace_runs_the_way_the_picture_is_laid_out`, which requires the forward reading above +0.95 and
the reversed one below −0.95: a test that only checked "strong" would pass on the upside-down trace.

**A photograph with no `depth_base` is refused by name.** It is a point sample anchored at one depth
and covers no interval, so there is no axis to read along; stretching it over a guessed thickness
would invent every sample in it. **The depth range is taken to span the picture end to end**, which
makes the conditioning crop also the statement of where the core is in the frame — crop the tray and
the tape away, or they are read as rock.

**Lanes are an approximation and say so.** A four-row core box is split into equal lanes read in
order; a real box has unequal rows and gaps between them. Default is 1, so nobody gets the
approximation without asking, and the note points at cropping to one row for a careful job.

Samples sit at the MIDDLE of the slab they averaged, so a trace read at 2 cm is not shifted a
centimetre shallow against the log it is compared with. Photographs are sorted into depth order
before anything is written — a delivery arrives in whatever order it is stored in, and a
non-monotonic curve is a sawtooth to every reader downstream. Reading and writing are separate
buttons, the `set_name` rule again.

Rule 7 throughout: numpy + Pillow in ONE subprocess per batch of 8 (photographs are large), both
runners read `sys.stdin.buffer`, and `core_image_support()` probes before anything opens. The real
round trips are `#[ignore]`d so the green gate never depends on an optional package.

**Not yet built, and deliberately named**: perspective correction, CLAHE/denoise/sharpen, the
stitched multi-box depth strip, WL/UV pairs, and a log-view strip track. Cross-correlating the
photograph trace against GR to PROPOSE a depth shift is `registration.rs`'s job and would compose
with it — the trace is already a curve.

## Squaring up a box, and the three corrections that change what a trace says (2026-08-01)

`coreimage.rs` finishes the conditioning toolbox. `CoreRecipe` gains `quad` (perspective) and
`denoise` / `clarity` / `sharpen` (detail), every field `#[serde(default)]` so recipes already
stored in a project still load. Six rules.

**Perspective is four draggable corners rather than another slider, because a slider cannot fix
it.** A core box photographed from one end is a trapezoid: the far end is drawn shorter than the
near end, so a depth read straight down the frame runs fast at one end and slow at the other, and
every sample between them is out by an amount that changes along the core. Straighten cannot touch
that — rotating a trapezoid gives a rotated trapezoid. `Quad` is the four corners in reading order
(TL, TR, BR, BL) as FRACTIONS, applied after the rotation and before the crop, because the corners
are dragged onto the picture the user can see and the crop is what states where the rock is.

**Rectifying deliberately CHANGES the aspect ratio, which is the opposite of the rule plates
follow.** A thin section must never be stretched, because its delivered shape is the truth; a box
shot at an angle arrives with its shape already wrong. The output's proportions are measured from
the quadrilateral's OWN sides — inheriting the frame's would put the distortion straight back, and
a box that really is eight times as long as it is wide has to come out eight times as long or the
depth axis is still not linear.

**In corner mode the picture is shown UNRECTIFIED and uncropped.** You cannot point at the box's
corner in a photograph that has already been squared up to it, and a crop would have cut the
corners off. `viewRecipe()` in `coreConditionDialog.ts` is the one place that decides; everything
else edits the real recipe. The polygon is the feedback while dragging, so a corner is stored on
pointer-up without re-rendering — re-rendering rectified on every corner would take the corners off
screen.

**The corners belong to the photograph, so `colour_only` clears them** — and `colour_only` is now
written out field by field rather than with a `..self.clone()` spread, so a new field has to be
classified as framing or as light DELIBERATELY. Getting that wrong is silent: every other box in
the run would quietly take this box's framing, and the only evidence would be crops that look
slightly off on pictures nobody cropped. Pinned by
`applying_a_look_to_a_delivery_carries_the_colour_and_not_the_framing`, which is written as a full
struct literal so a new field fails to compile there.

**CLAHE's tile floor is a handful of pixels, NOT one per histogram bin.** The obvious guard — a
tile smaller than the 256-bin histogram falls back to the identity — turns EVERY tile into the
identity on a box cropped down to a single row, which is forty-odd pixels across. The slider then
does nothing at all, silently, on exactly the pictures most likely to need it. Sparse counts are
what the clip limit is for. Found by a test, not by reading it back.

**Local contrast damages the SCALE, not the shape, and that is the whole reason `touches_detail`
exists.** On a perfect ramp from clean sand into mudstone, Clarity HALVES the darkness contrast
(P10-P90 0.62 to 0.30) while the agreement with a GR rising through the same mudstone barely moves
(+1.00 to +0.97). Pearson is scale-invariant and CLAHE compresses without inverting, so the
correlation has a ceiling on how far it can move — the S-factor calibration's lesson again, where
two central values could only ever disagree by so much and the spread had no such limit. What the
compression costs is comparability: `CPHOTO_DARK` is only useful once it is calibrated against a
real GR, and a transform fitted on an un-equalised box does not hold on an equalised one. Nothing
in either curve says which is which, so `extract_core_log` NAMES the photographs that carry one of
the three. Pinned by `local_contrast_flattens_the_very_trend_the_trace_is_reading`, which also
asserts the correlation STAYS high — so nobody "improves" the test into the check that would find
nothing to warn about.

Denoise and Sharpen are the same family read the other way: one suppresses `CPHOTO_TEX`, the other
inflates it. **Their radius is a FRACTION of the long edge rather than a pixel count**, so the
preview the user judges them on and the full-size bake take the same thing out of the rock — the
`min_pore_px` argument turned around (there the number states what the picture can resolve and must
stay in pixels; here it states a size on the core and must not). Both are capped, because a median
filter costs the square of its radius and nothing past a 9x9 removes speckle any better.

## The core, running down the page beside the log (2026-08-01)

`coreimage::build_core_strips` (Condition Core Photos… ▸ **Build depth strips**) cuts every box of a
delivery into its rows and stacks them into ONE tall picture per box, core running down it, at the
box's own depth interval. The built-in **Core** layout puts that beside GR, `CPHOTO_DARK` and the
porosity crossover. Six rules.

**The lay-out is baked into a picture, not applied while drawing, and that is the whole design.** A
core box has the core running across the frame in several rows; a log track has depth running down
it. Turning one into the other is a rotation and a re-stacking — and doing it at draw time would
mean writing that geometry THREE times, in the WebGPU log view, the SVG export and the PDF export,
with nothing to stop the three drifting apart. That is the standing `composite.rs`-versus-renderer
warning, and this is the version of it that does not need a warning: a strip is an ordinary
depth-registered image, so every renderer already knows how to draw one and what the screen shows is
what prints. It also needed no new `DrawOp`.

It is inspectable for the same reason. A strip appears in the Wells pane, in Plate Details and in a
composite like any other delivery, so a lay-out that came out wrong can be SEEN rather than deduced
from the shape of a curve.

**The strip and the trace lay a box out from ONE statement of how it is laid out**, so they cannot
disagree about which row is shallowest or which way a row runs. `reverse` is a 180-degree rotation
of the frame; then each row of core is rotated 90 degrees CLOCKWISE so its shallow end is at the
top, and the rows are stacked in order. Clockwise because the core runs left to right in the box, so
its left end has to end up at the top — and `np.rot90(a, -1)` rather than a bare transpose, which is
a reflection about the diagonal and would mirror every sedimentary structure across the core.
Verified on a marked fixture: the mark on row 1's shallow end at that row's own top edge lands at
the strip's top RIGHT. Pinned by `a_strip_reads_the_same_way_the_trace_does`, which reads a trace
off the strip as a plain single-lane picture and requires it to match the trace read off the box it
came from — a strip with its rows stacked in the wrong order would still look like a perfectly good
core photograph in a log track, and nothing but this comparison would catch it.

**Rebuilding REPLACES.** A strip is derived, not delivered: pressing Build again with a different
lane count is the same re-run a module makes, not a second delivery of pictures. So unlike an import
it writes one fixed set name rather than auto-suffixing, and tuning a lane count leaves no trail of
`STRIP_1`, `STRIP_2` behind. Writing the strips over the photographs they were built from is refused
by name.

**`ImageStyle.fit` gains "stretch", and it is the one case the never-stretch-a-plate rule does not
cover.** A thin section is never stretched because its delivered shape is the truth and a squashed
plate misstates grain shape. A depth strip is the opposite: its vertical axis IS depth, set by the
print scale, and its width IS the track — neither of them the picture's own, so there is no true
aspect ratio to preserve. Without it `contain` leaves a strip as a hairline down the middle of the
track and `cover` shows a couple of per cent of it blown up; both are what the existing rules give,
and both are useless. Reserve it for pictures whose two axes are both imposed from outside.

**`CPHOTO_DARK` sits BESIDE gamma in the built-in layout, never on top of it.** Overlaying the two
needs a shared scale and there isn't one — darkness is dimensionless, gamma is API units — so a
common axis would be a picture of a calibration nobody has done. Side by side the eye does the
comparison, and the trace's own signed correlation puts a number on it.

**Each box keeps its own depth interval, so a gap between two runs stays a gap.** Stitching the
whole cored interval into one picture would have to invent depths across the gaps, and boxes that
overlap would have to be reconciled — neither is something a display should decide. Storage follows
the same reasoning as everything else here: across-core pixels are capped at `STRIP_MAX_W`, because
a strip is drawn a few centimetres wide and past that the extra columns are storage rather than
detail, with the height following proportionally so nothing is distorted.

Still open on the core-photo road: WL/UV pairs, and feeding the trace into `registration.rs` to
PROPOSE a core-to-log shift (it is already a curve, so that composes).

## The photograph as a registration reference, and a saved curve nothing could read (2026-08-01)

`registration.rs` gains a third reference kind, `"curve"`, offering the core photograph's own
`CPHOTO_*` traces beside the plug columns and the point datasets in Data ▸ Tools ▾ ▸ Register
Depth… Four rules, and one bug the work uncovered.

**It is not a general curve-vs-curve registration.** The `CPHOTO_*` curves are the only ones in a
project MEASURED ON THE CORE, so they carry the core's depth error and a shift found from them is a
shift for the plugs. Any other curve is a wireline reading and registering it against another
wireline reading would answer nothing.

**They are also the densest reference this dialog has.** A plug table gives a few dozen samples a
foot apart; a photograph gives a reading every few millimetres down the whole cored interval. That
is what a cross-correlation wants — the same reason the thing being registered against is a log
rather than a set of picks.

**Darkness is the one proxy whose SIGN is known, and a negative peak is refused in words.** The
shift is still chosen on |r| like any other proxy, because darkness is not a gamma reading and
forcing two different quantities onto one line would be a claim nobody made. But the expected sign
is not a mystery: clay is dark and clay is radioactive, so both rise into shale. A winning peak that
is NEGATIVE says the box is laid out the other way up — which a correlogram cannot tell apart from a
genuine depth error — and accepting it would bake an upside-down photograph into the core's depths
where nothing downstream could find it. `expects_to_rise_with_shale` is deliberately a named
predicate rather than a family entry: giving `CPHOTO_DARK` the GR family would make the pairing
like-for-like, which asserts they are the same quantity.

Pinned from both sides by `the_photograph_trace_can_anchor_a_shift_and_says_when_the_box_is_upside_
down`, which runs the same fixture twice — once as delivered, once inverted — and requires the first
to recover the 2 m error and the second to be named rather than proposed.

### The bug it uncovered: a saved trace nothing could read

`computed_curves` are joined onto the standard depth grid by an **exact** depth match. `extract_core
_log` wrote its curves at the PHOTOGRAPH's own sampling — a reading every couple of centimetres,
landing on a wireline depth only by coincidence — so `CPHOTO_DARK` was written, was counted in the
run's report, and then came back all-NaN to every module, plot and export that read it. The worst
shape a bug can have here: the run says three curves were saved and the project holds three curves
nothing can open.

The trace now resamples onto the well's own depth frame before writing, and says so in its notes. A
well with no wireline frame falls back to the photograph's sampling and says THAT instead, rather
than pretending.

**The resampling is a box AVERAGE, not an interpolation.** The photograph is sampled several times
finer than a log, so linear interpolation between two neighbouring photograph samples is very nearly
picking one of them — and picking one of every seven is aliasing: a lamination every few centimetres
would beat against the log's sampling and come back as a trend that is not in the rock. Each output
sample takes every photograph sample inside the interval reaching halfway to its neighbours.

**An output depth with no photograph inside it is NaN, never the nearest value.** Outside the cored
interval there is no picture, and filling it in would draw core where none was cut.

Pinned by `a_saved_trace_lands_on_the_frame_the_rest_of_the_project_reads`, which checks the
read-back through `fetch_curve_frame` rather than a row count — the read-back is the thing that was
broken — and feeds the resampler a lamination alternating sample by sample, which must come back at
its mean rather than at whichever phase the coarse frame happened to land on. The older test
asserted 200 stored rows, which was pinning the bug; it now asserts the curve is readable and still
carries its trend.

## White light and ultraviolet, side by side (2026-08-01)

A core shed shoots the same box twice — once in white light, once under ultraviolet — and the UV
frame is where an oil show lives, as fluorescence that is simply not in the white-light picture.
Condition Core Photos… gets a **pair picker and a Hold for the pair** button, and Build depth strips
gets an editable **target dataset**. Five rules.

**The two deliveries stay two deliveries.** A UV frame is a different measurement of the same rock,
not a version of the white-light one, so it arrives as its own dataset and follows the delivery-set
model like everything else. That also means everything downstream already works: build strips off
both, put two image tracks side by side, and the log view and the composite need nothing new.

**Held, not toggled** — the before/after argument. The answer is a glance, and a toggle leaves you
one click away from tuning the wrong picture without noticing.

**The pair is matched on the depth INTERVAL, never on the name.** The two deliveries are two
cameras' filenames for one box, and a shed's naming is a shed's business. Matching on OVERLAP rather
than on nearest top means a UV frame shot in two halves still finds the white-light box it belongs
to; a point sample with no thickness falls back to a half-metre proximity so it is not excluded by
having no interval to overlap with.

**Each frame is rendered with its OWN recipe**, through the same preview pipeline. Showing a UV
frame under a white-light photograph's white balance would be a picture of the correction rather
than of the fluorescence — and the white balance is exactly the correction that has no meaning
across two light sources.

**A delivery is never paired with itself.** The picker is rebuilt when the source changes and drops
the source from its own list; otherwise it would show the same picture and read as a control that
does nothing.

**The strip target is editable and suggested rather than fixed.** `build_core_strips` always took a
target; the dialog now shows it, pre-filled from the source's own name — `CORE PHOTO UV` suggests
`CORE STRIP UV`. With one fixed name the second build would quietly replace the first, leaving one
box's two lights reduced to whichever was built last.

## A packed core-display plate, and the conversion as its own tool (2026-08-01)

A whole-core delivery is not a folder of core-box photographs. It is a **core-display plate**: four
COLUMNS of core side by side on a page, each column a separate barrel labelled with its own top and
base, with preserved intervals and part-filled last columns between them, a depth ruler down the
left, a title block above and a caption below. Read as one continuous span divided into four equal
parts — all the old lane count could do — every sample below the first gap lands at the wrong depth.

**`extract_core_log` now reads LANES, and a lane is a barrel.** `Lane {start, end, depth_top,
depth_base}` carries fractions of the across-core axis plus the barrel's own interval;
`PlateLayout {span, lanes}` adds the fraction of the down-core axis that is core, so the title block
is not read as the shallowest rock in the well. Both are per PICTURE (`CoreLogSpec.layouts`, keyed
by image id), because every plate of a delivery carries different barrels, and they are held by the
frontend as a `corelanes` document — the `platelabels` precedent: a list anyone can read and correct
beats a blob.

Four rules, all in `plan_lanes`, which is deliberately split out so they can be pinned without a
Python subprocess:

- **Depths are ALL-OR-NOTHING across a picture's lanes.** Half a plate labelled is REFUSED, because
  the only way to place the unlabelled columns is to assume the core runs on without a break — which
  is exactly what the preserved interval on the same plate disproves. The dialog says so as you
  type, not after a run.
- **With no depths the picture's interval is shared out by lane LENGTH**, not into equal parts. On
  equal lanes those are the same number, which is what keeps every pre-set-era core-box run
  byte-identical; on a detected lay-out with a part-filled last column, only the length version is
  still true.
- **`reverse` flips the down axis and reverses the lane ORDER, rather than mirroring the frame.**
  The old 180° rotation was equivalent for these three measures (a per-slab mean or standard
  deviation does not care which way the across axis runs) and is the version that survives explicit
  spans, which are stated on the picture as the user sees it. The order is only reversed where no
  lane carries depths — where they do, the order carries no information and reversing it would
  silently re-order a labelled plate.
- **A plate whose columns carry depths needs no interval of its own.** It has already said where its
  rock is; requiring an envelope nobody uses would be a second place to get it wrong.

**`detect_core_lanes` proposes and never applies.** The split is Otsu on the picture's own
across-axis mean brightness — core is darker than the page it is printed on and the bench it is shot
against, and mean rather than any texture measure because printed captions and ruler ticks are
textured too. The whole PROFILE comes back and is drawn, the `registration.rs` rule: four clean
columns and a smear the threshold happened to cut in four are the same answer and completely
different situations. **The DEPTHS are never guessed** — nothing in the pixels says what depth a
column of rock came from.

**The conversion is its own tool** (Jauhar, 2026-08-01: *"for core image conversion to log, separate
it from core photos tools, it should have independent tools"*). `coreTraceDialog.ts` → Advance ▸
Core Imaging ▸ **Photo Log…**; conditioning keeps `coreConditionDialog.ts`. Two jobs with two
lifetimes: conditioning is done once per delivery and finished, a trace is read, checked against GR,
re-laid-out and read again.

**`recommend_core_recipe` measures a picture and proposes conditioning, with the reason for every
value.** Read off the IMPORT, never the conditioned copy (advice about an already-corrected picture
would correct it twice); the values land in the same sliders the user would have moved and Apply is
still Apply. Five rules:

- **The neutral is the brightest UNCLIPPED, LEAST COLOURED part of the frame**, never grey-world.
  Averaging the whole picture to grey would treat a genuinely red-stained core as a cast and scrub
  the stain out — the trap the thin-section correction avoids by anchoring on the matrix.
- **The gain is normalised so the largest is 1**, so it can only DARKEN: pushing a channel past
  white clips exactly the brightest pixels and twists their hue. A blue cast pulls BLUE down, which
  reads backwards at a glance — the gain is the reciprocal of how bright the channel already is.
- **With nothing neutral in frame it declines and names the fix** rather than balancing off rock.
- **Detail is NEVER recommended.** Clarity, Sharpen and Denoise rearrange a pixel's neighbours,
  which is what the trace is read from — local contrast roughly halves the darkness contrast
  `CPHOTO_DARK` measures. Where the picture would benefit the advice SAYS so and leaves the slider
  alone, so a user reaching for it knows the cost.
- **A UV plate is recognised and left alone.** Very dark AND with essentially no neutral surface, it
  is a different measurement rather than a badly exposed one: it is MEANT to be dark, the background
  IS the answer, and lifting the exposure to a mid-grey median drowns the fluorescence the plate
  exists to show. A UV lamp is not white light, so there is nothing to make neutral. The control
  test — a dim white-light frame with a tray in it still gets lifted — is what stops the rule
  degenerating into "give up on dark pictures".

**PDF import is NOT being built** (Jauhar, 2026-08-05: *"dont try to import pdf, user will just
provide photo"*). He exports the plates himself and imports them as ordinary pictures. Recorded in
`docs/plan_core_photo.md` §4a with the design kept in a `<details>` block, because the reason is a
workflow choice rather than a technical verdict. What it costs: a hand export loses the captions, so
the barrel depths are TYPED into Photo Log's column table, and which folder is white light and which
is UV is declared at import as two datasets.

Still to come, from Jauhar's UV question (2026-08-01): a DISCRETE sand/shale curve off the
white-light trace (`CPHOTO_LITH`), and an "unfold" that shears each slab to the bed's apparent dip
before averaging, so a dipping contact is not smeared across the core's width.

## Fluorescence off the UV frame (2026-08-05)

`CPHOTO_FLUOR` — Photo Log ▸ **Light: Ultraviolet**. Same `extract_core_log`, not a second function:
the lanes, the barrel depths, the resampling onto the well's frame and the write discipline are one
code path, so the two lights can never disagree about where a barrel is. `CoreLogSpec` gains `light`
(`"white"` default — anything unrecognised is white light, so a typo cannot silently switch the
measurement) and `fluor: Vec<FluorClass>`; the runner's wire format became ONE `cols` map keyed by
curve name for the same reason. Curves: `CPHOTO_FLUOR` (fraction of each slab in any band),
`CPHOTO_FLUOR_I` (its mean brightness), plus `CPHOTO_FLUOR_<NAME>` per class **only when there is
more than one** — with a single band the per-class curve would be a byte-identical copy of the
total, and two names for one answer is how a report ends up unable to say which it quoted.

**It is an INFERRED SHOW and the notes say so on every run.** Mineral fluorescence, drilling-fluid
additives and dead oil all fluoresce, and a drained slab shows nothing. The `CPHOTO` prefix is what
stops any module reading it as a saturation — the `GRAIN_D50_APP` argument.

**The light is DECLARED, never detected.** A UV frame is dark; so is a daylight photograph of dark
shale in a shadowed box, and the evidence for "this is ultraviolet" would be the brightness about to
be measured — the same circle that makes an impregnated thin section something the user states.

**`FluorClass` carries a saturation CEILING, and that is not decoration.** Fluorescence is routinely
described as *dull blue-white*, and white is the ABSENCE of colour — it cannot be written as a
floor. Same type distinction that makes `StainBand` carry one so dolomite is identified by staying
colourless. `default_fluor` ships ONE generic band, deliberately: splitting bright yellow-green from
dull blue-white would assert an INTERPRETATION (that the hue split means live versus dead oil) this
repo has no source for, so the run says a second class can be added and leaves the reading to
whoever writes the show reports.

**`fluor_band_is_saturated` is the guard, and it is deliberately NOT `petrography::scene_dominated`.**
The obvious transfer — is this picture's own median pixel inside the band — was written first and the
round-trip test refused a slab that was exactly half fluorescing, which is the answer the measure
exists to give. "Rock is mostly rock" is true; "a UV frame is mostly background" is NOT, because an
oil-soaked box glows over most of its length. Worse, per-picture it would drop the one heavily
stained box in a clean delivery. So the test is the whole run's **P10 > 0.95**: the band is condemned
only when it claimed nearly everything nearly everywhere, which carries no depth information whatever
the light was. Measured, shown and previewed either way — what is refused is the WRITE, the pore
rule's split exactly. **There is no mirror guard**, and that asymmetry is the point: a core with no
fluorescence is the ordinary answer and is what gives the box above it meaning.

**The two lights watch different halves of the conditioning recipe.** `CoreRecipe::touches_light()`
(gain/warmth/tint/exposure/contrast/saturation) is reported on a UV run because `CPHOTO_FLUOR` counts
pixels against an ABSOLUTE brightness floor; `touches_detail()` stays the white-light warning because
`CPHOTO_DARK` is read comparatively and a correlation does not feel a uniform scale. **And the
darkness-sign note is white-light only** — clay is both dark and radioactive so DARK and GR should
agree, but an oil show sits in the clean sand, so a negative correlation there is ordinary and
printing that paragraph would send the user to reverse a lay-out that was already right.

UI: `colourBand.ts` is reused unchanged (a `PoreColorBand` is structurally a `FluorClass` minus the
name and ceiling), so the fluorescence band and the pore band cannot drift about what a wrapped band
means; the pale limit is one extra slider. Bugs worth remembering: a delete handler must read every
card BEFORE filtering, or removing the first of two reads each survivor off its neighbour's control —
right-looking when you delete the last, silently swapped when you delete the first. And the
round-trip's daylight control must be COLOURED, not grey: a neutral-grey frame is rejected by the
saturation floor on its own merit, which would have made the test pass while testing nothing.

