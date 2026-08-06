# Measuring a thin section

Build record for the petrography suite — what each measurement is, what it refuses, and what it
turned out to be worth when it finally met real rock. The phase plan is `plan_image_analysis.md`;
this is what was built and why.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## Plate scale and preparation (2026-07-31 — D4 answered)

Jauhar's answer to D4 was **"sometimes"** on both counts: sometimes the section states a scale,
sometimes not; sometimes it is epoxy-impregnated and stained, sometimes not. A uniform answer
either way would have been easier — this one means one delivery holds plates of both kinds, so
`well_images` gained `fov_um`, `prepared` and `stain` **per plate**, all defaulting to absent, all
DECLARED and never inferred (`db::migrate_plate_scale_and_prep`, ADD COLUMN only, no backup;
existing plates get NULL, which is the honest answer).

**Scale is entered as a FIELD OF VIEW WIDTH, not micrometres per pixel.** The stored copy is
resampled to a long-edge cap, so a um/px belongs to whichever copy it was measured on and nothing
in the number says which — while "this picture is 2.5 mm across" is true of every copy of it. um/px
for any copy is `fov_um / that copy's pixel width`, which is what the readout derives. It is also
the form a petrography caption already states. There is **no default**: §3's "no default um/px,
ever" now has teeth, because absent is the normal case rather than a corner.

**Anything dimensional must REFUSE an uncalibrated plate rather than report pixels.** A D50 in
pixels is not a D50, and a number with the right name and the wrong unit is the same failure as a
wrong `m` — it computes, it plots, it ships. A run over a mixed delivery reports how many plates it
skipped and names them; a silent subset looks exactly like a complete answer. Family A (area
fractions) is unaffected, which is why it stays first.

**`prepared` unknown is REFUSED, not assumed either way.** This is the sharper rule, because the
failure is silent in both directions. A blue-epoxy pore rule run over a section nobody impregnated
does not fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge
artefact, which then plots against core helium porosity as though it meant something. Detecting
impregnation from the pixels is the same circular move as detecting a water zone from the
saturation being calibrated: the evidence for "this is blue epoxy" is the blue about to be
measured. `stain` is FREE TEXT for the same reason the RtC water zone is declared — which stain
was used is the laboratory's fact, and a menu invented here would be a protocol nobody performed.

**Delivery-level values fill the blanks; what is stored belongs to the plate.** Magnification
genuinely varies within one delivery — that is the whole content of "sometimes" — so the import
wizard takes one field of view for the delivery plus a per-plate **FOV mm** column that overrules
it, and `ImageImportItem.fov_um` (`#[serde(default)]`) carries the override. Preparation is taken
delivery-wide at import (one impregnation run, one staining bath) but stored per plate so a mixed
delivery can still be corrected.

`src/ui/plateDetails.ts` is the one control, shared by the import wizard and the plate editor — the
same decision, and two copies is two places for the wording to drift (the `followCore.ts`
argument). `db::set_image_details` writes one plate, `db::set_image_delivery_details` writes a whole
live delivery in one statement (the `shift_well_images` argument: a core-photograph delivery is
hundreds of plates). **Every value is written as given, `None` included** — a wrongly typed scale
has to be clearable, and one that cannot be removed is worse than one never entered, because
everything downstream believes it. The delivery-wide button REFUSES "All datasets": that would give
a core photograph the thin sections' magnification. Its undo restores plate by plate, because the
plates need not have agreed before and writing one value back across the delivery would invent a
uniformity that was not there.

Advance ▸ Petrography ▸ **Plate Details…** (renamed from Plate Depths…, same dialog).

## Pore area from blue-dyed epoxy (2026-07-31 — Part 2 A1)

`petrography.rs` (Advance ▸ Petrography ▸ **Pore Area…**, `poreAreaDialog.ts`) is the first
measurement taken off a plate, and deliberately the **dimensionless** one: an area fraction needs no
micrometres per pixel, so it runs on every plate rather than only the calibrated ones. The
deliverable is an area fraction per plate, which estimates volume fraction by the Delesse relation.

**A plate must be DECLARED impregnated, and an undeclared one is refused BY NAME.** This is the
whole reason `well_images.prepared` exists (`petrography::epoxy_check`, deliberately split out and
public so a test can pin it without Pillow). A blue rule run over an unimpregnated section does not
fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge artefact, and
that number then plots against core helium porosity looking entirely reasonable. Nor can the app
work it out from the pixels: the evidence for "this is blue epoxy" is the blue it was about to
measure, which is the same circle as reading a water zone off the saturation being calibrated. The
plate picker greys out and explains each unqualified plate BEFORE a run rather than after.

**The colour band is the user's, not the app's.** `PoreColorBand::default()` is a plain blue band in
round numbers, offered as the starting point for a VISUAL tuning task, and pinned as generic by
`the_default_colour_band_is_generic_not_a_calibration` (same discipline as `gr_normalize`'s
reference percentiles — a two-decimal threshold would be somebody's regression result).

**The preview comes from the SAME code as the measurement.** Redrawing the mask in the frontend
would put the segmentation in two languages and the two would drift — the standing `composite.rs`
versus log-view-renderer warning. So the Python runner returns the overlay PNG, and what the user
tunes against is literally what gets measured. Tuning re-measures ONE plate (`only_image_id`) and a
stale in-flight answer is dropped by sequence number rather than being allowed to overwrite a newer
one.

**No morphological cleaning.** Opening or closing a mask needs a structuring element measured in
PIXELS, which is a size — and a plate may carry no scale at all, so that size could not be stated in
microns for every plate. Rather than pick a pixel count meaning a different physical distance on
every plate, nothing is smoothed and the speckle stays visible in the preview where it can be
judged. This is the scale gate applied consistently, not an omission.

**Results are POINT DATA, not a curve** — `aux_data`, dataset `PETROGRAPHY`, item `VPORE_TS`, at
each plate's depth. A thin section measures the one plug it was cut from; a line between two of them
would claim rock nobody looked at, the same argument that made point data a track kind rather than a
`CurveStyle`. Its own dataset rather than the image delivery's name, so re-running the measurement
never looks like a second delivery of pictures. **Measuring and saving are separate**: tuning a
threshold means running it many times, and `set_name` is what turns a run into a write.

Rule 7 throughout: numpy + Pillow in ONE subprocess per batch of `CHUNK` (16) plates — a
core-photograph delivery is hundreds of plates at ~1 MB each and one batch would be a gigabyte in
flight. `pore_support()` probes before a run so the dialog can name what is missing. The runner
reads `sys.stdin.buffer`, never `sys.stdin`. The real round-trip test
`a_quarter_blue_plate_measures_a_quarter` is `#[ignore]`d so the green gate never depends on an
optional package; it builds a plate that is exactly a quarter blue plus a pale violet patch that a
hue test alone would count — **it is the saturation floor that rejects that patch**, which is why the
floor exists.

## Calibrating a plate from its own scale bar (2026-07-31)

`src/ui/scaleBarDialog.ts`, reached from the **⇹** button on each row of Plate Details…. The route
that makes a plate measurable when it states its scale as a BAR burned into the image rather than as
a field of view in the caption — which, on Jauhar's "sometimes yes, sometimes not", is a good share
of them. It is the gate everything dimensional sits behind.

**The measurement is a pure RATIO, and that is the whole reason it is safe.** The drag is taken as a
FRACTION of the picture's width, so the field of view is `bar length / that fraction`. Nothing in it
depends on the display zoom, or on the stored copy having been resampled to a long-edge cap — both
lengths shrank by the same factor and the ratio did not move. This is the same property that made a
field of view the right thing to store rather than micrometres per pixel, and it means the answer
comes out already in the form the store wants, with no second conversion to get wrong. Verified in
the browser: the same drag at a displayed width of 848 px and of 400 px both returned 2000 µm.

Endpoints are held as fractions of the natural width/height for the same reason — they survive a
view-mode switch and a window resize without being recomputed.

**A crooked drag costs almost nothing, so there is no snapping.** Off a truly horizontal bar by 5°
the measured length is long by 0.4%, because the error is second-order in the angle. What actually
decides the accuracy is hitting the bar's ENDS, which is what the **Actual size** mode is for — one
pixel of a 100 px bar is 1%, and there is no way to shrink that except to look closer.

**It only FILLS the box.** The row's own Save is still what writes the value, so a calibration is
reviewed like any other typed number rather than being applied by the act of measuring. The optional
"apply to every plate of this delivery" writes row by row rather than through
`set_image_delivery_details`, because each plate must keep its OWN preparation and stain: a scale
must never quietly overwrite what the section was made of. Slower, and the only version that is
right.

`openModal` has no close hook, so the dialog watches `#modal-root` for its content being detached
and resolves `null` — a caller awaiting a calibration must not be left hanging on Esc or ✕.

## Pore geometry (2026-07-31 — Part 2 family C)

`petrography.rs` gained per-pore shape and size, opt-in beside the area fraction in the same
dialog. **One decode, one mask, both answers** — the fraction and the geometry can never describe
different pictures, which two passes would eventually allow.

Outputs per plate: `PORE_N`, `PORE_ASPECT`, `PORE_SHAPE` (all dimensionless, reported for every
plate) and `PORE_D10` / `PORE_D50` / `PORE_D90` in micrometres, **written only where the plate
carries a scale**. Not a NaN in their place — a NaN would still occupy the item and read as a
measurement that failed rather than one that was never possible.

**Four-connectivity for the pore phase.** Two pores meeting at a single corner are joined by a
throat of zero width; that is not one pore body, and 8-connectivity would fuse them.

**The perimeter is a four-direction Crofton estimate, NOT a boundary-pixel count.** A staircase
boundary overestimates a diagonal edge by up to √2, which biases circularity systematically LOW —
systematically, so it never looks like noise. The estimate used is
`P = (π/8)·[(N_h + N_v) + (N_d1 + N_d2)/√2]`, which returns 2πR for a disc. Measured on a synthetic
disc of radius 100: area 31417 against 31416, perimeter 630.1 against 628.3, circularity 0.994. Its
worst case is a perfectly axis-aligned rectangle, where it returns `(π/4)(w+h)(1+√2)` against a true
`2(w+h)` — about 5% low, for any rectangle. Pores are neither circles nor axis-aligned boxes and
circularity is read comparatively, so a few percent of consistent bias does not change which pore is
rounder. Pinned by `the_perimeter_estimator_is_crofton_not_a_boundary_pixel_count` so nobody
"simplifies" it back into a boundary count.

**Aspect ratio comes from second moments, so it carries none of the perimeter's bias.** The `+1/12`
discrete correction is included: a pixel is a unit square rather than a point mass, and without its
own variance in the second moment a small round pore reads as elongated purely from the sampling.
Measured 1.0000 on a disc and 5.0000 on a 40×200 bar.

**A pore cut by the frame is EXCLUDED and counted** (`n_edge`). Its true size is unknown, and
including it biases the size distribution small — the standard stereological edge rule. **A blob
below `min_pore_px` is speckle and is dropped and counted** (`n_small`); that threshold is in PIXELS
on purpose, because it states what the picture can resolve rather than a size in the rock, and it
has to mean the same thing on a plate that carries no scale at all.

**Diameters are AREA-WEIGHTED.** Capillary pressure fills volume, and a count-weighted median on a
digitized section is dominated by the smallest features the scan resolves — which says more about the
scan than about the rock. `weighted_percentile` lives in `petrography.rs` rather than
`distribution.rs` deliberately: that module is source-agnostic on a bare value slice, and a parallel
weight vector is a different contract only this caller needs. Every UNWEIGHTED percentile still goes
through `distribution::percentile`, so a pore percentile and a log percentile are the same operation.

**The runner stays deliberately dumb** (the `office.rs` rule): it returns per-PORE arrays and every
statistic is computed in Rust. Geometry needs **scipy** — only for the connected-component
labelling, which in pure numpy would be a Python-level union-find over millions of pixels — so it is
opt-in and its absence fails only the geometry, never the area fraction. The real round trip
`a_disc_reads_as_round_and_its_diameter_follows_the_declared_scale` is `#[ignore]`d for the same
reason the rest are.

## Mineral classifier (2026-07-31 — Part 2 family A3)

`petrography.rs` `run_plate_classifier` + `mineralClassDialog.ts` (Advance ▸ Petrography ▸
**Mineral Classifier…**). Quartz against feldspar in plane light is not a colour problem, so this
family is a supervised classifier and never a colour rule — `docs/plan_image_analysis.md` §2.1 A3.

**There is no shipped model and there will not be one.** A model trained on somebody else's
sections, under somebody else's lamp, would produce numbers with the shape of a modal analysis and
none of the content. The training data is this user's clicks on these plates, and the result says
so in its own notes: the lamp, the white balance and the scanner are part of what it learned, so it
is not a model for a differently photographed delivery.

**Clicking IS the method, because it is the workflow that already exists.** Point counting is a
petrographer moving a stage and naming what is under the crosshair. The dialog is that act, and
what it produces is training data rather than a tally.

**The labels are the artefact, not the model.** They persist as a `platelabels` document keyed
`<well_id>/<dataset>`, and the forest is refitted from them — seeded — on every run. A stored model
blob cannot be read, argued with or corrected; a list of clicks can be all three, and the answer
stays reproducible from it. This is deliberately unlike `ml_models`, where the artefact is the
model because the training curves may be gone by the time it is applied.

**Cross-validation groups by CLICK, not by pixel.** A click contributes its immediate neighbourhood
so the fit has some support, but those pixels are near-identical — splitting them across a fold
boundary scores the model on data it has already seen and reports an accuracy nobody can reproduce
on a new plate. Same discipline as blind-well CV in `ml.rs`. Pinned by
`the_classifier_is_cross_validated_by_click_not_by_pixel`.

**Recall is reported PER CLASS and the weak ones are named.** An overall 0.9 sits comfortably on top
of one mineral the model cannot see at all, and that mineral's fraction is then noise wearing a
percentage sign. Below 0.7 the run names it and the dialog colours the row.

**Two refusals before a subprocess is even started.** One class is not a classification — a model
that always says "quartz" is right every time and knows nothing. And a class with fewer than
`MIN_CLICKS_PER_CLASS` (3) clicks cannot have any held out, so its accuracy would be a number about
nothing. Pinned by `the_classifier_refuses_a_training_set_it_could_not_be_checked_on`.

**Features are colour plus TEXTURE**, and the texture is the only reason this can attempt a pair
colour cannot separate: R, G, B, cos/sin of hue, saturation, value, and the local 5×5 mean and
standard deviation of brightness. **Hue enters as its sine and cosine** because it is circular — 359°
and 1° are neighbours, and a raw angle would place them at opposite ends of the feature.

**Measured, not asserted** (`the_classifier_separates_on_texture_and_admits_when_it_cannot`,
`#[ignore]`d, needs scikit-learn). Two halves of a plate with the SAME mean colour differing only in
texture — one smooth, one cloudy: accuracy 1.000, both recalls 1.000, fractions 0.504 / 0.496 against
a true half and half. The CONTROL matters more: label one uniform material as two minerals and
held-out accuracy fell to **0.410** with recalls 0.38 and 0.44, near chance, and the run then names
both classes as unreliable. A classifier that cannot be caught inventing a distinction is worse than
no classifier.

Items are `CLS_<MINERAL>` — **deliberately not `MIN_`**, which the stain rule uses. A fraction a
colour rule produced from a published stain identification and one a classifier produced from this
user's clicks are different claims with different provenance, and one name would leave a report
unable to say which it quoted. Same argument that keeps `GRAIN_D50_APP` apart from `GRAIN_D50_W`.

Label positions are FRACTIONS of the picture, never pixels — the stored copy is resampled to a
long-edge cap, so a pixel coordinate belongs to whichever copy it was taken on and nothing in the
number says which. The scale-bar argument again.

Each plate's fraction is estimated from a systematic sample capped at 400 000 pixels, and the count
is reported rather than being a silent truncation. Needs scipy AND scikit-learn, probed by
`classify_support` so the dialog can name what is missing before a run.

## Stained carbonate (2026-07-31 — Part 2 family A2)

`petrography.rs` reads the stain as well, opt-in beside the pore fraction, the pore geometry and
the grains — same decode, same pore mask, so the mineral fractions and `VPORE_TS` describe ONE
segmentation and sum against each other. Fractions are of the WHOLE plate: **pore + minerals +
unclassified = 1**, verified as exactly 1.000 on a synthetic four-quarter plate.

**A plate is refused unless its OWN declared stain matches the scheme.** Undeclared is refused too,
for the `prepared` reason: it cannot be read off the pixels, because the evidence for "this is
alizarin red" is the red about to be measured. Reading an alizarin-red scheme off a section stained
with something else does not fail — it returns mineral fractions that are wrong and entirely
plausible. Names are compared with punctuation and spacing thrown away (`normalize_stain`), so
"Alizarin Red S" and "alizarin-red-s" are one stain but a different stain is not.

**The identifications are published; the colour bands are not.** `stain_scheme` ships Friedman
(1959) for alizarin red S and Dickson (1966) for the combined alizarin red S + potassium
ferricyanide stain — standard carbonate petrography, already named in
`docs/plan_image_analysis.md` §2.1. What hue a stained calcite *photographs* as depends on the dye
batch, the concentration, the etch, the lamp, the white balance and the scan, so the bands are round
numbers to start a visual tuning from, exactly like the epoxy band, and the class list is editable.
Pinned by `the_stain_schemes_are_published_identifications_with_generic_bands`.

**`StainBand` carries a saturation CEILING, and that is not a decoration.** Dolomite under alizarin
red S is identified by staying COLOURLESS. "Unstained" is the absence of colour and cannot be
written as a floor, which is why this is a different type from `PoreColorBand`.

**Classes are tested IN ORDER, first match wins.** A pixel is one mineral. Overlapping bands are
resolved by the order the user put them in rather than being silently counted twice.

**`MIN_UNCLASS` is written on every run and is the honesty number for the family.** Solid that fell
in no band is reported rather than distributed over the classes; a section where a third of the rock
matched nothing has not been given a mineralogy, whatever the other rows say. Above 25% the run says
so in the notes.

**Blue epoxy and turquoise ferroan dolomite are the same colour, and this is measured, not
theorised.** Under Dickson's stain ferroan dolomite goes turquoise; blue-dyed epoxy is blue. On a
plate that is both impregnated and stained the pore rule claims those pixels first, so the mineral
is counted as porosity. On the synthetic plate, with the default epoxy band (180–260°) the run
returned **pore 0.500 and ferroan dolomite 0.000** — porosity doubled and a mineral erased, both
plausibly. Narrowing the epoxy band to 210–260° returned **pore 0.250 and ferroan dolomite 0.250**,
which is the truth. `epoxy_collides` detects the overlap and NAMES the affected minerals in the
notes; it is never resolved automatically, because which of the two bands to narrow is a judgement
made looking at the plate. Pinned by
`blue_epoxy_and_ferroan_dolomite_are_flagged_as_the_same_colour`, which also checks the check is not
trivially always true.

Items are `MIN_<MINERAL>` (`mineral_item` upper-cases and collapses non-alphanumerics, so "Ferroan
calcite" becomes `MIN_FERROAN_CALCITE`) plus `MIN_UNCLASS`, all in the `PETROGRAPHY` dataset at the
plate depth. Dimensionless throughout, so unlike the grain sizes they run on every plate including
the uncalibrated ones.

`hsv_of` and `in_band` are now the ONE colour conversion in the runner, shared by the pore rule and
every stain class — the same argument that made `shape_stats` shared between the pore and grain
phases.

## Grain size (2026-07-31 — Part 2 family B, D3 closed)

`petrography.rs` gained the grain phase, opt-in beside the pore fraction and the pore geometry in
the same dialog. **One decode, one mask, three answers** — the grain phase is defined as whatever
the pore rule did not claim, so the porosity and the grains describe ONE segmentation. That is also
why grains inherit the blue-epoxy refusal: a plate where pore cannot be told from solid cannot have
its grains outlined either.

Outputs per plate: `GRAIN_N`, `GRAIN_ASPECT` and `GRAIN_CONTACT` (dimensionless, every plate), plus
`GRAIN_D10_APP` / `GRAIN_D50_APP` / `GRAIN_D90_APP` in micrometres and `GRAIN_SORT_APP` in phi where
the plate carries a scale, and the four `_W` twins when the Wicksell correction was asked for.

**D3's answer — "apply wicksell correction is optional" (Jauhar, 2026-07-31) — is implemented as
different ITEM NAMES, not one name and a flag.** There is deliberately no bare `GRAIN_D50`: a name
that sometimes means the section value and sometimes the corrected one cannot be read by anything
downstream, and a report quoting it has no way to say which it got. Pinned by
`apparent_and_corrected_grain_sizes_are_stored_under_different_names`, which matches on the `put(`
call site rather than the bare string — a test that scans its own source must not trip over the
name it is looking for.

**The split is a nearest-centre partition of the solid phase, NOT `scipy.ndimage.watershed_ift`.**
That was tried first and measured: on a welded pair that should split evenly it gave one grain
47792 pixels and the other 9, because its tie-breaking across the quantized cost plateaus lets
whichever marker is reached first take almost everything. The nearest-centre partition splits the
same pair 23957 / 23844, returns 16 of 16 discs in a loose pack at 7845 px against a true 7854, and
keeps a single disc as ONE grain at every separation setting. (scikit-image's watershed would work
too, at the price of a whole new dependency for one function.)

**The search is confined to one connected blob of solid at a time, and that is load-bearing.**
Without it a pixel can be nearer a centre across open pore than its own, and the two disconnected
pieces would then carry one label — one grain in two places, with an area and a shape belonging to
neither. Solid is labelled EIGHT-connected, the complement of the pore phase's four: two grains
meeting at a corner are one piece of rock even though the pores either side of them are not one
pore.

**`GRAIN_CONTACT` is the honesty number and it rides with every grain run, never optionally.**
Where grains are welded by cement or an overgrowth there is nothing in the picture to separate
them, and the algorithm places a boundary at the neck anyway — a geometric artefact, not a grain
contact. The stored value is the median fraction of a grain's outline that is a grain-to-grain
contact rather than open pore; above 0.7 the run says so in the notes and tells the reader to treat
those sizes as a rock-fabric description rather than a grain-size analysis. It is deliberately a
ratio of two counts gathered the same way rather than two Crofton perimeters: the staircase bias
affects both alike and cancels, and this is a quality indicator, not a length.

**Sorting is Folk & Ward (1957) inclusive graphic standard deviation**, `σ_I = (φ84 − φ16)/4 +
(φ95 − φ5)/6` with `φ = −log2(d in mm)`. Chosen over a plain standard deviation because it is what
maps onto the verbal scale a core description already uses. Phi RISES as grains get finer, and a
sign slip there would flip every sorting number in a deliverable while leaving it looking entirely
reasonable — hence `phi_rises_as_grains_get_finer`. Phi is a logarithm of millimetres, so sorting
needs a scale exactly as much as a diameter does.

**Everything is AREA-weighted, and on a section that IS volume weighting.** The chance of a random
plane meeting a grain scales with its diameter and the mean cut area with its square, so the
section area attributable to a size class goes as `n·D³` — which is what a sieve weighs. That is
what makes apparent and corrected comparable to each other and either of them comparable to a
sieve, and it is the same weighting the pore diameters already use, so there is one rule in the
module rather than two.

**The Wicksell unfolding is Saltykov's, DERIVED rather than transcribed.** The published
coefficient table is a set of numbers that can be mis-copied and would then be wrong silently. They
come instead from the chord geometry — a plane at distance `h` from a sphere's centre cuts a circle
of diameter `√(d² − 4h²)`, so `F(x) = 1 − √(d² − x²)/d`, and a random plane meets a sphere at a rate
proportional to its diameter. Twelve logarithmic classes, and class 0 reaches down to ZERO rather
than stopping a decade below the maximum: the published version drops that tail, and losing real
sections to a class boundary would be a silent subset. Negative unfolded populations are clamped
and COUNTED (`w_clamped`) — the inversion is ill-conditioned by nature, and a clamped class is the
signal that this plate's correction is unstable.

**The representative diameter of a class is its UPPER bound, because that is the diameter the
unfolding solved for.** Reporting the class midpoint instead would quote a population the
arithmetic never solved, and on a single-size population it comes back ~11% fine purely from where
the bin edges fell. Its cost is that every class is quoted at its coarse edge.

**What the correction actually buys, measured rather than assumed.** A population of identical
spheres is perfectly sorted; its sections are not, and that spread is the dominant Wicksell effect.
It is on SORTING, not on the median — the apparent median of a monodisperse population is only
about 13% low (the median chord of a sphere is √3/2 of its diameter) and area weighting pulls even
that most of the way back, because it up-weights exactly the near-central cuts. Measured here:
apparent sorting on a perfectly sorted population is 0.19 phi area-weighted, which on the Folk &
Ward verbal scale is still inside "very well sorted"; count-weighted it is worse. So the weighting
choice moves this number more than the correction does, and a user reaching for Wicksell hoping to
move D50 is reaching for it for the wrong reason. Pinned by
`the_correction_earns_its_place_on_sorting_not_on_the_median`, and the unfolding itself by
`a_single_sphere_size_unfolds_back_to_one_class`, which recovers the true diameter exactly.

Two pixel knobs, `min_grain_px` (50) and `grain_sep_px` (20), both ROUND and both stated in PIXELS
for the `min_pore_px` reason: they say what the picture can resolve, not a size in the rock, and
they must mean the same thing on a plate carrying no scale. Over-segmentation is what a
distance-based split gets wrong when it gets anything wrong, so the preview draws the grain
outlines in yellow over the same mask — judged by eye, not from the table. Pinned as generic by
`the_grain_defaults_are_generic_not_a_calibration`.

Geometry needs **scipy**, so grains are opt-in and their absence never touches the area fraction.
The real round trip `welded_grains_still_split_but_say_that_the_boundary_was_inferred` is
`#[ignore]`d for the usual reason.

UI note: the Wicksell label is hidden with `style.display`, NOT the `hidden` attribute. It carries
an inline `display: block`, and a display rule beats `hidden` every time — setting the attribute
left the row fully visible at 19px tall. Same family as the ribbon panels and menus; caught in the
browser, not by the compiler.

## Plug QC — checking a measurement against an independent one (2026-07-31)

`plugqc.rs` + `plugQcPanel.ts` (Advance ▸ Petrography ▸ **Plug QC…**, also in the workspace
＋ menu) plot two measurements made on the SAME plug against each other. The petrography numbers
were the first measurements this app produced that nothing else in it could check: an area fraction
estimating a volume fraction by the Delesse relation is a *claim*, and the only test of it is the
helium porosity of the plug the section was cut from.

Sources are the three plug-scale stores — a routine-core column (CPOR/CPERM/CGD/CSW), any numeric
item of any point dataset (which is where every petrography output lands), and a **pore-throat
radius read off the plug's own capillary-pressure curve**. All three read through the active-set
fragments like every other reader.

**A pair is two measurements of the same plug, and a sample with no partner inside the tolerance is
DROPPED and COUNTED — never snapped.** Same rule as the S-factor calibration and the same reason: a
core that is off by a whole sample interval is invisible to any tolerance check, so widening the
tolerance to win more points quietly pairs a plug with its neighbour. `registration.rs` is the fix,
and the empty-result note points there rather than suggesting a wider tolerance.

**A measurement is used ONCE.** Pairing is greedy on the closest pair first and consumes both
sides. Two sections cut a centimetre apart would otherwise both claim the one plug nearest them,
and that single core porosity would appear twice in the cloud and twice in the correlation,
tightening it for free. Pinned by `one_plug_cannot_be_claimed_by_two_sections`.

**Both a linear and a rank correlation are reported, because they answer different questions.**
Pearson asks "is this a straight line", which is right when the axes are the same quantity measured
twice. Spearman asks only "do they move together", which is the only sensible question for pore
BODIES against pore THROATS — different lengths that must never fall on one line, though a rock
with bigger bodies had better have bigger throats. Spearman is also invariant to any monotone
transform, so it does not move when the pane switches an axis to log, which keeps the number from
disagreeing with the picture beside it. Pinned by
`a_curved_but_monotone_relation_reads_as_rank_agreement_not_a_straight_line`. Both inherit
`tops::pearson`'s four-point floor, and a blank is EXPLAINED in the notes rather than left as an
empty cell that reads as a bug.

**Nothing here converts a unit** — point data is stored verbatim — so the result reports the MEDIAN
of each axis. A 0.19 beside an 18.2 is a percent-versus-fraction delivery the user can see at a
glance, which beats a guess about which one was meant.

**The throat radius is Washburn with the laboratory's OWN σcosθ**, taken from `scal_pc.ift` as
recorded. A plug with no recorded interfacial tension has a pressure but no radius and is excluded
BY NAME — `thomeer.rs` takes the same line for the same reason. Pc is interpolated in **log Pc**:
one curve spans decades, so interpolating linearly between a 10 psi and a 1000 psi step lands an
order of magnitude out. A saturation outside the measured range is **never extrapolated** — a curve
that stopped at 20% mercury cannot state r35, and a radius invented past the last step would be the
strongest-looking number on the plot. The default is **35% mercury**, the Kolodzie (1980) / Winland
r35 convention already used by `rocktyping.rs`, which is what makes this plot directly comparable
to the R35 curve that module predicts from φ and k. `resolved_saturation` is the ONE place the
default is applied, so a caption can never disagree with the number it labels.

`fitScatter.ts` gained the two things this needed and the calibration dialogs did not: a
`{kind: "none"}` reference line and optional log axes. **A comparison of two DIFFERENT quantities
gets no line and independent axes** — a 1:1 line between a pore diameter and a throat radius
asserts an equality nobody claims, and every point sitting below it would read as a disagreement
when it is the physics. The line is SAMPLED across the window rather than drawn end to end, because
`y = slope·x` is not a straight line in log space. A value at or below zero is SKIPPED on a decade
axis, never floored to the smallest positive one. `.form-row[hidden]` was added to `styles.css` for
the mercury-saturation row — a `display` rule beats the `hidden` attribute, the gotcha the ribbon
panels hit twice.

Statistics are computed on EVERY pair before the cloud is decimated to `MAX_POINTS` for the wire,
and the decimation says so in a note; the display points are spread evenly, never the first N.
Changing the reference line or an axis scale redraws from the pairs already in hand — those are
display choices, and re-pairing would be the same answer arrived at more slowly.

## The first real delivery (2026-07-31 — what running it on real rock changed)

Six increments of measurement had been built on top of each other without any of them meeting real
rock. Running the pore rule over a real carbonate petrography delivery — 134 photomicrographs, one
laboratory, one well, one report — changed the design. Three findings, in the order they bite.

**A petrography delivery does not arrive as a folder of pictures.** It arrives as an Excel workbook
with one WORKSHEET per plate: the well, the depth in feet, the plug number and the magnification
typed into cells, and the photomicrographs anchored on top as embedded objects. `images.rs` takes a
list of files and can read none of it. Every plate in this delivery had to be lifted out of the
workbook before anything in this app could see it. That is the actual first barrier between the
petrography suite and a client's rock, and nothing in the suite addresses it yet — a plate importer
that reads a workbook is the missing increment, not another measurement.

**The delivery states a magnification, not a field of view.** Cells read `5x` and `10x`. Turning
that into micrometres needs the camera sensor size and the tube factor, neither of which the
delivery states, so `fov_um` cannot be filled from it. Some plates carry a scale BAR as a separate
embedded graphic beside the picture — a yellow rule captioned `1 mm` — which is what
`scaleBarDialog.ts` exists for, but only once the bar and the plate are in the same picture.
Everything dimensional stays refused on this delivery, which is the designed behaviour and, on real
data, the common case rather than a corner.

**And the finding that changed the code: `epoxy_check` was only half the guard.** It refuses the
plate nobody impregnated. It says nothing about a plate that WAS impregnated but photographed under
a light the colour band was never tuned for — and there the rule swallows the matrix and returns a
porosity anyway. Across these 134 plates the median hue of the picture ran from **26 to 310
degrees**: one blue-cast plate sat at 221 and read **0.97 v/v**, a green-cast plate from the same
core at 149 read 0.06. Twenty-eight plates measured above half the section as pore. Not one of them
would have failed; all of them would have been stored at a real depth and gone on to plot against
core helium porosity.

`petrography::scene_dominated` is the guard. **The test is the plate's OWN median hue, not a cap on
the answer.** A cap would be arbitrary — one field of view crossing a large vug genuinely can be
mostly pore — but rock is mostly rock, so on a plate the band is reading correctly the TYPICAL
pixel is a grain and its hue falls OUTSIDE the pore band. When the median pixel is pore-coloured,
the band has stopped discriminating and is describing the scene. On this delivery that flagged
every one of the 28 plates reading above 0.5 v/v, and the highest an unflagged plate reached was
0.387 — a plausible carbonate. What would be stored went from a 0.000–0.972 range with a 0.231
median to 0.000–0.387 with a 0.115 median.

**The fraction is still measured, shown, and previewed; what is refused is the WRITE.** Tuning the
band is exactly how a user fixes this and they cannot tune against a number they are not shown, so
the plate appears in the table in `var(--warn)` with the reason on hover. Nothing off that plate is
stored — not the fraction, not the pore shapes, not the minerals — because they all come off the
same mask, and if the mask is the background then every number derived from it is about the
background. The run also reports the delivery's hue SPREAD when it exceeds 60 degrees, because that
is what decides whether one band can serve the whole delivery: here it could not, and the honest
instruction is to measure the plates in groups. Pinned by
`a_plate_whose_own_median_hue_is_pore_coloured_is_not_measured`,
`the_scene_check_reads_a_wrapped_band_the_way_the_runner_does` (the guard must read a band written
across 0 degrees as two arcs, exactly as the runner's `in_band` does, or it would silently disable
itself for anyone using one) and the round trip `a_blue_cast_plate_is_shown_but_never_stored`.

**A synthetic fixture the guard rejected was the fixture's fault, and fixing it mattered.**
`welded_grains_still_split_but_say_that_the_boundary_was_inferred` drew small discs floating in
epoxy — 87% pore, which is a mount rather than a rock. It now draws grain-dominated plates. A
fixture that could not exist is a fixture that cannot catch the bug the real delivery found.

Still open, found and not yet fixed: a delivery can mix photomicrographs with SEM plates and scale
graphics in one folder, and a colour rule run over a greyscale SEM image returns **0.000** — a
plausible-looking number for a tight rock, and the mirror of the 0.97 case. The obvious test
(saturation) did not separate them on this data, so nothing was shipped rather than a guessed
threshold.

## Plates delivered inside a workbook (2026-07-31)

`images::probe_plate_workbooks` + `WORKBOOK_RUNNER`, wired into the existing Import pictures…
wizard. The barrier the first real delivery exposed: **a petrography delivery does not arrive as a
folder of pictures.** It arrives as a workbook with one WORKSHEET per plate — the well, the depth,
the plug number and the magnification typed into cells, the photomicrographs anchored on top. A
file picker can read none of it. On this machine 165 such workbooks exist against essentially no
folders of loose thin sections.

**It is an EXTRACTOR, not a second importer.** It writes the plates to a temporary folder and hands
them plus a depth table to `import_images`, so normalization, the Pillow long-edge cap, the delivery
set model, `follow_core`, `fov_um` and `prepared` all apply unchanged. Two importers would
eventually disagree about one of those — the standing `composite.rs` versus log-view-renderer
warning — and an extractor plus one importer cannot.

**The depth comes from the CELL, and overrules anything a filename would have said.**
`parse_depth_from_name` exists for a folder of loose files and has to guess; here the laboratory
wrote the depth down. It is read only where a UNIT follows it, because the same header block
carries the plate number and the plug number — on a real delivery the cell reads `4633.50 FT/ 108`
and taking the bare number would be a coin toss. A sheet with no stated depth gets NONE, is
counted, and is reported; it is never filled in from a neighbour. Pinned by
`the_workbook_reader_only_takes_a_depth_that_carries_a_unit`.

**And that number is read under EITHER decimal convention** (2026-07-31). One delivered book wrote
103 of its plate sheets `6980.71 FT` and 18 of them `7016,54 FT` — one laboratory, one report, one
file number, two people. Reading only the dot convention did not FAIL on the comma sheets, which is
what made it dangerous: the comma split the number, `7016` was dropped for carrying no unit, and
`54 FT` matched instead, so a seventh of the delivery was stored at **54 feet on rock cored at
7,000**. A plausible shallow depth on entirely the wrong sand. Same family as
`parsers::read_text_file`: bytes must be interpreted rather than assumed, and so must numbers.

`as_number` in `WORKBOOK_RUNNER` is the one place that decides. **Where both separators appear the
RIGHTMOST is the decimal** — true of `1,234.56` and `1.234,56` alike, and it needs no guess about
which locale typed it. **A single separator is a decimal unless the token is VALIDLY grouped**
(1–3 digits, then exactly 3), which is what keeps `4633.500 FT` reading as three decimal places
rather than becoming 4,633,500. The genuinely ambiguous `1,234` is read as a DECIMAL and REPORTED,
because the wrong answer is then absurd (1.234 ft) rather than plausible (1234 ft) — an absurd
depth gets looked at, a plausible one gets used. Pinned by
`a_comma_decimal_depth_is_read_as_one_number_not_two`, which is EXECUTED through the discovered
interpreter rather than asserted against the source (a source match keeps passing over a regex that
no longer works) and skips with a printed reason where there is no Python, the `field_fixtures`
pattern.

**Known limit, found on the same delivery and deliberately not patched around.** One sheet in 129
writes `7033,50/354 FT (CORE)` — the unit sits on the PLUG number, not the depth — and reads 354 ft.
Every rule that would fix it breaks a commoner shape: "prefer the first number" misreads
`PLATE 12, DEPTH 4633.50 FT`. The defence stays the import wizard's editable table, where a 354
among 7,000s is visible before anything is stored.

**The unit is the sheets' own**, and only when every sheet that stated one agreed; a mixed workbook
returns `None` so the wizard has to ask rather than fall back to the display unit. A foot silently
read as a metre puts a plate more than three times too deep and nothing on the log looks wrong.

**A magnification is not a field of view and is never converted into one.** Turning `10x` into
micrometres needs the camera sensor width and the tube factor, both properties of the laboratory's
microscope rather than of the plate, and neither is in the delivery. It is carried through as text
so the user sees what the sheet claimed, and everything dimensional stays refused until a real
scale is entered. A sheet stating TWO magnifications attaches none — which picture is which cannot
be told without guessing from where the caption sits, and a magnification on the wrong plate is
worse than none.

**`MIN_PLATE_PX` (400) is in PIXELS and round**, the `min_pore_px` argument: it states what a
picture has to be to be a plate, where a byte count would say more about the JPEG quality. A
workbook carries decorations anchored beside the plates — scale-bar graphics, logos, letterheads;
on the real delivery those ran 117x59 and 207x79 against plates of 1920x1080. Every drop is COUNTED
and named per sheet, never silent.

**The old `.xls` is REFUSED BY NAME with the fix** ("Save As .xlsx in Excel"), and it is the
majority format — 107 of the 165 workbooks here. Its pictures can be recovered by scanning the file
for JPEG blobs; what cannot be recovered without a full BIFF parser is which worksheet each one sat
on, and the worksheet is where the depth is. A plate hung off the wrong sand is a wrong conclusion,
so a guessed association is worse than no import. `.xls` stays in the file-dialog filter on purpose:
selecting one gets a named refusal rather than a picker that appears broken. Pinned by
`the_old_workbook_format_is_refused_by_name_with_the_fix`, and its sibling
`the_newer_workbook_formats_are_accepted` exists so nobody tidies `.xlsm` out of the filter — that
is the same package with macros in it.

Rule 7 throughout: openpyxl + Pillow in ONE subprocess for the whole selection, and the runner reads
`sys.stdin.buffer`, never `sys.stdin` — a workbook path with any non-ASCII character would otherwise
arrive as mojibake and fail naming a path nobody has. Pillow is used HEADER-ONLY here (`Image.open`
without `.load()`) to size each embedded picture; it also decodes the EMF plates a vector-illustrated
delivery carries, through the Windows GDI.

The real round trip is `images::workbook_field_tests::plates_come_out_of_a_real_petrography_workbook`,
`#[ignore]`d and driven by `SANDIBUMI_FIELD_FIXTURES` with a `workbooks/` subfolder — it takes
whatever the folder holds and skips with a printed reason when unset, so a fresh clone stays green.
Measured on two real deliveries: **152 plates, every one with a depth from its sheet, unit ft, 33
notes** covering dropped decorations, sheets stating two magnifications and sheets whose header omits
the depth.

## The whole road, and what it measured (2026-07-31)

`petrography::field_tests::a_delivered_book_measures_against_the_petrographers_own_point_count`
drives the entire chain on a real delivery — workbook in, plates at their stated depths, pore area
measured, checked against an independent measurement of the same rock through `plugqc`. Every
increment before it was verified against synthetic plates, which can only ever prove the
arithmetic.

**The independent measurement is the petrographer's own POINT COUNT, deliberately not helium
porosity.** A plug's helium porosity and a section's area fraction differ for two reasons at once —
the measurement and the depth registration — so a disagreement could not be attributed to either.
The petrographer counted the SAME picture, which puts only the measurement under test. (This also
found that a point-count table need not carry its own total: one delivered table left the *Total
porosity* column empty on every row with the six components filled in, and several component cells
read `trace`, which is a word.)

**The answer on this delivery is that it does NOT agree, and that is the finding.** 152 plates
against 50 counted samples paired 35 plugs: counted median 14%, measured median 6.8%, Pearson
**-0.300**, Spearman **-0.092**. Sweeping the band from 180-260 to 220-260 moved the measured median
from 5.8% to 0.5% and never moved either coefficient off zero.

**The measurement was tracking each photograph's colour cast rather than the rock.** On a
green-cast plate (own median hue ~149 degrees) the band found 0.04% against a counted 15%; on a
blue-cast plate (~195 degrees) it found 31% against a counted 9%. Across one laboratory, one core
and one report the plates' median hue spanned 289 degrees. The existing "not photographed under one
light" note was already firing; what was new is how completely it invalidates the numbers rather
than merely qualifying them.

**Within a colour-consistent group it works.** Restricted to the blue-cast plates with a band
tuned to them: Pearson 0.643, Spearman 0.616 on 10 plates. That is the reason the family is worth
keeping and the reason "measure them in groups" is a real instruction rather than a hedge.

**Matching the median is not evidence that the measurement is right, and this is the sharpest
result of the exercise.** On the green-cast group a band can be tuned until the measured median
lands on the counted median almost exactly (15.72 against 15.00) while the per-plate rank agreement
stays at **-0.10**. Tuning a colour band until the average looks right is therefore precisely the
wrong way to tune it: the average is the one statistic that survives a segmentation which has
stopped discriminating. Tune against the PREVIEW on a single plate, and judge a delivery by
agreement, never by its mean.

Still open and not shipped: the mirror of the scene-dominance guard. A plate cast AWAY from the
band returns a fraction near zero, which is a plausible number for a tight rock and is currently
stored. The signature is visible here (0.04% against a counted 15%) but the floor that would
separate it from a genuinely tight section is a judgement, not a measurement, so nothing was
invented.

## A delivery can be vector, and it was vanishing (2026-07-31)

The same run found that half a petrography delivery could not be imported at all. `openpyxl`
**DROPS** the picture formats it cannot decode — WMF and EMF — with a warning nothing downstream
sees. One delivered book of 53 plate sheets and 106 photomicrographs therefore arrived as a
workbook that appeared to hold no pictures: `ws._images` was empty, the sheet was skipped by `if
not imgs: continue`, and the file produced **zero plates and almost no notes**. A silent subset,
which reads as a complete answer — the same failure the scene guard was built for, one layer down.

**So `WORKBOOK_RUNNER` now reads the pictures from the PACKAGE and leaves openpyxl to read the
cells.** That is not a patch around the drop, it removes the failure mode by construction: openpyxl
does what it is good at (the cells the depth is written in) and the pictures come from the zip.
Unlike the old `.xls`, the association is EXPLICIT — workbook -> sheet part -> drawing part -> media
part, every step a relationship file — so nothing is guessed, which is exactly the property `.xls`
lacks and why that format is still refused. Document order in the drawing XML is anchor order, so
the panels keep the order they appear in. Pinned by
`the_workbook_reader_takes_its_pictures_from_the_package_not_from_openpyxl`, which fails if
`_images` ever comes back.

`sniff` recognises **EMF**, or a recovered plate would be called "not a recognised image format" by
the importer that just extracted it. The four-byte record type is far too weak a magic on its own,
so the ` EMF` signature at offset 40 is what identifies it — pinned from both sides, including the
control that the record type alone is NOT enough. `rclBounds` is inclusive, so a picture 1103
device units across reads 0..1102. Pillow decodes EMF through the Windows GDI; without Pillow the
importer says "EMF needs Pillow" by name rather than storing a plate nothing can display.

A worksheet holding no picture is now **counted and reported once per file** rather than skipped in
silence. A cover sheet legitimately holds none — but a delivery whose plates failed to come through
shows up here as a large number instead of as nothing at all.

Measured on the same two real books: **258 plates where there had been 152**, all 258 through the
extractor, 242 through import and measurement (the 16 without a stated depth are counted and
reported, never filled in from a neighbour).

## One band, many lamps (2026-07-31 — the colour fix)

The first real delivery showed the pore rule tracking each photograph's colour cast rather than
the rock: across one core, one laboratory and one report the plates' own median hue spanned 289
degrees, and a band tuned on one plate found 31% on a blue-cast plate the petrographer had counted
at 9% and 0.04% on a green-cast plate they had counted at 15%. `PoreSpec.reference_image_id` names
the plate the band was tuned on, and every other plate is colour-corrected onto it before the band
is applied. Six rules.

**The correction is a per-channel GAIN, not a rotation of the hue wheel.** A wrong white balance is
physically a gain on each sensor channel, so undoing it is a gain back — the von Kries diagonal
model. A fixed hue rotation looks like the same thing and is not: a channel gain moves hues near
the boosted primary much less than hues perpendicular to it, so a rigid rotation lands the matrix
correctly and the epoxy wrong, which is exactly the wrong way round.

**The reference patch is the delivery's own ROCK, never grey.** Grey-world — forcing the three
channel means together — is the textbook white balance and is actively harmful here: a blue-epoxy
section IS genuinely blue-biased, and the more porous it is the more so, so grey-world would
normalize away the very signal being measured and compress every plate toward one answer.
Anchoring on the reference plate's matrix colour assumes only that the rock is the same rock, which
within one core is a far better assumption than "the lamp was the same". Pinned by
`the_colour_correction_is_anchored_on_the_reference_plate_not_on_grey`.

**The matrix colour is the channel-wise median of the pixels the band did NOT claim — never the
whole plate's median.** This shipped as the whole-plate median first and that was wrong in a way
that looked right. The whole-plate median moves with how much epoxy is in the field of view: a
plate with more pore has a bluer median, so anchoring on it partly normalizes away the very
contrast being measured. That is the grey-world trap above, reached by a different route. Measured
against a petrographer's own point count on a real delivery: rank agreement **0.19 uncorrected,
0.05 on the whole-plate anchor, 0.20 on the matrix anchor**. The same delivery photographed each
plug twice, and the two fields of view differ in whole-plate median hue by 66 degrees at p90 —
far more than one lamp can explain, which is what says the whole-plate median is measuring the rock
rather than the light.

Resolving matrix from pore needs the band, and the band needs the correction, so it is ONE
iteration and it terminates: the uncorrected band defines the matrix, the gain follows, the band is
applied again. `scene_hue` stays the WHOLE-plate median hue, because "is the typical pixel
pore-coloured" is genuinely a whole-plate question — only the anchor changed.

Pinned by `a_plate_corrected_onto_one_lit_the_same_way_is_left_alone`, which is the invariant the
first version broke: two plates of one rock under one lamp differing only in porosity must come
back unchanged. Its fixture scatters the pore evenly through a gradient-lit frame rather than
stacking it at one end — scattered pore hides the same share of every part of the gradient, so the
matrix median is identical on both plates while the whole-plate median moves. Stack it and both
anchors are biased and the test proves nothing. The test asserts that discriminating power before
it asserts the invariant.

**A plate the correction cannot reach at all is refused.** Where the band claimed essentially the
whole picture there is no matrix left to anchor on, so no gain can be built — and read as delivered
that plate would be stored at nearly 1.0. On a normalized run that case IS the scene-dominance
refusal, and takes the same message. It is the opposite end of `band_missed`, and the pair is why
neither guard can be dropped.

**The gain is scaled so the LARGEST channel gain is 1.** The correction is a relative rebalance, so
a uniform scale changes nothing that matters — and this way no channel can be pushed past 1 and
clipped, which would distort the hue of exactly the brightest pixels. The cost is a slight uniform
darkening, which the value floor can see.

**A reference plate that is itself scene-dominated REFUSES the whole run.** Everything is corrected
onto it, so a mistake there is inherited by every plate and then agrees with itself everywhere. On
a normalized run the plain per-plate scene test would only restate the reference's, so it is
checked once, up front, by name.

**The stain is read off the SAME corrected picture.** `stain_from` takes the h, s, v the pore rule
was read from rather than re-converting the image, or the minerals and the porosity would describe
two different photographs of one section — and they are required to sum against each other. The
preview overlay is drawn on the corrected copy too, for the standing reason: what the user tunes
against has to be literally what was measured.

Verified end to end by `the_same_rock_under_a_different_lamp_reads_as_the_same_rock` (ignored,
needs Pillow): two plates of identical synthetic rock, one photographed through a lamp 2.0x on
green and 0.55x on blue. Uncorrected the cast plate reads under 1% against its twin's 25% — the
delivery's failure, reproduced. Corrected onto its twin it reads the same quarter. The cast is
applied as channel gains chosen so nothing clips, which is what makes it a genuine white-balance
error rather than a repaint.

**The mirror guard, and why it is conditional** (Jauhar, 2026-07-31: "yes but conditional"). A
plate cast AWAY from the band returns a fraction near zero, and near zero is a perfectly plausible
reading for a tight rock — it plots against helium porosity without ever drawing attention to
itself, which makes it the more dangerous of the two failures. `band_missed` refuses it, and takes
its condition from the user rather than from a threshold: it applies **only on a normalized run**.
Without a reference there is no evidence the band finds epoxy anywhere in this delivery, so an
empty answer could equally mean the band has never been tuned, and refusing then would refuse a
first click. Naming a reference is the user's statement that the band works on THAT plate; once
that is on the record, a plate showing nothing after being corrected onto it is either nonporous or
mis-corrected, and nothing in the picture separates those two. Refusing is the conservative call.

**"Empty" is one resolvable pore's worth of pixels — the user's own `min_pore_px`, not a new
constant.** A band that has not claimed even a single countable pore over a whole field of view has
not found a pore phase; that is not a small porosity, it is not a measurement. Pinned by
`an_empty_measurement_is_refused_only_once_a_reference_plate_says_the_band_works`, which checks
both conditions independently and that raising the floor moves the bar with it.

`cast_shift` — how far this photograph's light sat from the reference's, by
`hue_delta`, the SHORT way round the wheel — rides beside every result and is reported in the
table. It is diagnostic and never a threshold: a plate that had to move a long way is one to look
at, and nothing else on the row would say so. NaN when no reference was named, and the column is
hidden then rather than shown empty — an empty column reads as "every plate matched" instead of
"nothing was compared".

The two guards cover for each other by different routes, which is why neither can be dropped: a
wholly blue plate is refused as scene-dominated on an uncorrected run, and on a corrected run its
own blue has become the matrix, so it is refused as `band_missed` instead. Same outcome, and the
round-trip test asserts both.

**What it is worth on real rock, measured rather than hoped.** On the delivery it was built for it
stops the measurement being actively wrong and does not make it right. Against the petrographer's
own point count over 45 plugs, with the two fields of view per plug averaged: rank agreement 0.19
uncorrected and 0.10–0.22 corrected depending on which plate is the reference; sweeping 57 bands,
the best reachable is 0.25 uncorrected against 0.15–0.36 corrected. Those best-of figures are an
upper bound fitted on the data they are scored on and must never be quoted as accuracy — this same
delivery already taught that tuning until a statistic looks right is how a segmentation that has
stopped discriminating passes for a good one.

**The measurement is repeatable; it is the agreement that is weak.** That delivery photographed two
independent fields of view of every plug, and the two agree with each other at rank 0.85 while
agreeing with the point count at 0.10–0.27. So the disagreement is systematic rather than noise,
and it is not the pictures. A colour band is not yet a substitute for a point count on this rock.

Still open, and deliberately not invented: whether a single reference can serve plates spanning 289
degrees at all. The correction gets less exact the further a plate has to move — shifts of 180
degrees appear on this delivery, which is the far side of the wheel and not a lamp — and how far is
too far is a judgement to be read off the shift column and the preview, not a number to ship.

## The second opinion, and what it moved (2026-07-31 — the helium arm)

Every judgement of the pore rule so far was made against the petrographer's own point count, on the
argument that counting the SAME picture puts only the measurement under test. That argument holds,
and it hid something: **nobody had asked whether the point count agrees with anything either.**

**It does not, much.** Against the laboratory's ambient helium porosity on the same 45 plugs, the
delivered point count reads **Pearson 0.581, Spearman 0.505**, with a median 14.5% against helium's
24.8%. That is the microporosity difference stated plainly — a point count ticks pores VISIBLE under
an optical grid, helium fills every connected pore including micropores far below optical
resolution, and in a carbonate that is most of the pore system. So ~0.5 is about the ceiling for
this rock, and "the colour rule disagrees with the point count" was never on its own evidence that
the colour rule is wrong.

**AMBIENT helium, not overburden.** A section is cut from an unstressed plug and photographed at
atmospheric pressure, so ambient is the like-for-like number; overburden folds in the rock's
compressibility, which is real and is not something a picture can see.

**Against helium the colour rule reaches 0.575 uncorrected and 0.67–0.69 corrected — and that
headline must never be quoted.** The delivery spans two cored intervals of very different rock, ~25%
porosity against ~5%. A coefficient computed across both is largely rewarding the tool for telling a
porous carbonate from a tight one, which an interpreter knows before starting. Scored INSIDE each
interval against helium:

| | shallow core | deep core |
|---|---|---|
| colour rule, uncorrected | 0.01 | 0.27 |
| colour rule, corrected | 0.19 | 0.49 |
| the petrographer's count | 0.51 | not counted |

Three things follow, and they are the reason this arm was worth running.

**The colour correction earns its place on independent data.** It lifts agreement inside BOTH
intervals — roughly doubling the deep one — measured against a laboratory instrument rather than
against the count it was previously scored on. Everything said before about the correction rested on
a reference that itself only reaches 0.5.

**The colour rule still loses to the petrographer where both exist**, 0.19 against 0.51. It is not a
replacement for a point count on this rock, which is the same conclusion as before, now reached
against a yardstick that can be defended.

**A cross-interval coefficient is a trap in this family generally.** Any measurement that separates
two rock types will look strong pooled and may resolve nothing within either. Score within an
interval, or say plainly that the number is a between-core contrast.

Method note that changes the numbers: this delivery photographed TWO fields of view of every plug,
and they are **averaged per depth, never pooled**. Pooling counts each plug twice, inflates n from
45 to 90, and adds no independent rock. Pairing is `plugqc.rs`'s rule throughout — closest pair
first, each measurement consumed once, nothing snapped beyond the tolerance.

Still open and deliberately not chased: the deep core has no point count at all, and the colour rule
reaches 0.49 there against helium. That is the one interval where this suite is doing work nobody
did by hand, and whether the numbers look like the rock is a question for the interpreter rather
than for another statistic.

## Judging a setting instead of eyeballing it (2026-07-31)

`PoreSpec.check_against` + `plugqc::score_against_plugs`, surfaced as **Check against** in the Pore
Area dialog. The reference plate turned out to be a bigger lever on the answer than the colour band
is — a 3.5x spread in rank agreement across three references drawn from one cored interval, with the
worst pick WORSE than not correcting at all — and the dialog offered nothing to tell a good choice
from a bad one except the preview. A setting judged by eye against a picture is judged on how the
picture LOOKS. This is the number that says whether it also tracks the rock. Six rules.

**The pairing is `plugqc`'s, literally the same code.** `score_against_plugs` differs from
`run_plug_qc` only in that one axis arrives as a slice instead of a database read; it shares
`samples_for`, `pair_samples` and `ranks`. A second pairing implementation would drift, and the
drift would be SILENT — both versions return a plausible correlation and nothing on screen says
which rule produced it. Pinned by `scoring_a_run_in_hand_matches_scoring_it_after_it_is_saved`,
which stores the identical values and requires the two paths to agree to the last decimal.

**Scored BEFORE it is saved.** That is the whole reason the slice form exists: tuning that had to be
written first would leave a trail of half-judged answers in the project, the same reasoning that
makes `set_name` optional on a pore run.

**Only the plates that would be STORED are scored.** `storable()` is the single predicate the write
path and the check share, and `storable_samples` is split out so the rule can be pinned without a
Python subprocess. A plate the run has already refused must not vote on whether the run is any good
— and the failure would be quiet rather than loud, because a scene-dominated plate reads near 1.0,
which is exactly the kind of outlier that moves a correlation on its own. Pinned by
`the_agreement_scores_only_the_plates_the_write_would_keep`, which also checks an interval plate
pairs on its MIDDLE, the convention `plugqc` and the point tracks already use.

**The RANK figure is the one to choose a setting on, and the dialog says so.** A section reads
systematically below its plug's helium porosity — microporosity below optical resolution, which on
a carbonate is most of the pore system — without being wrong about which plug is the better rock. A
delivery stored as a percent instead of a fraction does the same thing again, a hundredfold. Pearson
feels both; Spearman feels neither. Both are reported, and so are the two MEDIANS, which is what
makes a unit mismatch visible instead of mysterious. Pinned by
`a_scale_difference_moves_the_medians_and_not_the_rank_agreement`.

**One coefficient is not a decision, so the dialog keeps every setting tried this session.** 0.24 is
a poor result next to 0.53 and a good one next to 0.11, and the only way to know which is to have
seen the alternatives — the same argument as reporting the whole correlogram in `registration.rs`
rather than only its peak. The best is bolded, **but only among rows scored on the same number of
plugs**: changing the reference changes which plates get refused, so two runs can be scored on
different rock, and a coefficient that rose because the awkward plugs dropped out is not an
improvement. A non-comparable row is FLAGGED and never bolded, rather than hidden — it is still
informative, it just cannot be read straight across. Not persisted: it describes an afternoon's
tuning, not the project.

**A well with nothing to check against says so, and nothing is ever snapped.** A 0.00 would read as
"this setting is useless" rather than "nothing was compared". A plate with no plug inside the
tolerance is dropped and counted, and the empty-result note points at Register Depth… rather than at
a wider tolerance — a core off by a whole sample interval passes any tolerance check, so loosening
it quietly pairs each section with its neighbour's plug and returns a confident number about the
wrong rock. Core porosity is picked by DEFAULT where the well has it: a setting nobody thought to
verify is exactly the one that ships.

## A reference plate per cored interval (2026-07-31)

`PoreSpec.reference_zones` (Pore Area ▸ **Per-interval references**) lets one run correct different
depth ranges onto different plates. A delivery spanning two cored intervals is two different rocks,
usually photographed on two different days, and one reference serves both only by accident: on the
real delivery, giving each interval its own lifted rank agreement with core porosity in BOTH (0.19
to 0.24 shallow, 0.49 to 0.53 deep). That is a refinement rather than a rescue — and the point is
that it is now something the user can **measure** with **Check against** rather than be told. Six
rules.

**A plate no interval covers falls back to the delivery-wide reference, and where there is none it
is REFUSED by name — never read as delivered.** This is the rule the whole design hangs off.
`band_missed` only ever fires on a corrected plate, deliberately: with no reference there is no
evidence the band finds epoxy anywhere in this delivery, so an empty answer could equally mean the
band has never been tuned. Read one plate uncorrected inside a normalized run and it sits in the
same stored delivery as corrected ones having silently lost that guard, with nothing downstream able
to tell the two apart. Refusing keeps `normalized` a RUN-level fact, which is why nothing else in
the measurement had to change.

**Intervals may TOUCH but never cross.** `2000-2010` beside `2010-2020` is how anyone writes two
adjacent cored sections and neither should have to be typed a millimetre short, so `contains` is
inclusive at both ends and a shared depth goes to the interval listed FIRST. A genuine overlap is
refused up front, before a single picture is decoded: inside one, which reference a section is
corrected onto would come down to the order of a list nobody sees, so the same settings could give
two answers with nothing on screen saying why. Exactly the rule `db::apply_core_run_shifts` enforces
on core barrels, and for the same reason. A base above its top is refused as a typo rather than
silently swapped.

**Pass 1 harvests colours only; every plate is measured in pass 2.** The single-reference code used
the reference plate's own first-pass result AS its stored result, which is correct when correcting
onto itself is the identity. With several references a plate serving an interval it does not sit in
would have kept an uncorrected number while its neighbours were corrected — silently. Measuring
every plate in pass 2 costs one extra decode per reference and removes the case by construction. The
harvest pass draws no preview: what the user tunes against has to be the CORRECTED picture the
stored number came from. `run_batch` is the one copy of the pipe protocol, shared by both passes.

**Every reference is scene-checked before any other plate is decoded, and one bad one condemns the
run.** Everything in an interval is corrected onto its reference, so a reference that is itself
mostly the colour called pore anchors that interval to the mistake — and agrees with itself
everywhere afterwards. Refusing the whole run rather than just that interval is the conservative
call: a partial result with one interval quietly missing is worse than a named refusal.

**`PlatePore.reference_name` rides beside `cast_shift`, and the column appears only when more than
one plate served.** A shift of 40 degrees means nothing until you know which plate it is 40 from;
with a single reference the column would just repeat the picker on every row.

**Fractions from different intervals are only as comparable as their two references are**, and the
run says so in a note listing which plate served which span. Compare intervals on the agreement
figure rather than by reading their medians against each other.

Pinned by `reference_intervals_may_touch_but_never_cross`,
`a_plate_takes_its_own_intervals_reference_then_the_delivery_wide_one` (both pure, both green on
every gate run) and the round trip `each_interval_is_corrected_onto_its_own_reference` (ignored,
needs Pillow). That fixture's two lamps are deliberately NOT a pure channel gain apart, which is the
realistic case and the whole reason one reference stops serving a delivery: the deep sections are
lost when dragged onto a shallow reference (shift > 100 degrees, band missed) and read their true
quarter when corrected onto their own. Its orphan plate pins the refusal above.

## A thin section is a picture too (2026-08-01)

The conditioning workspace built for core slab photographs now serves plates as well
(Advance ▸ Petrography ▸ **Condition Plates…**), and Pore Area's colour band is a colour rather
than four numbers. Jauhar's rule from the core work — "geologist see image not text" — applied to the
petrography side, which is where it matters most, because a colour threshold is the one setting that
genuinely cannot be judged from a number.

**One workspace, two entry points, not two dialogs.** A thin section arrives with exactly the
problems a core photograph does: lifted out of a workbook at whatever angle it was scanned, under
whatever lamp the microscope had. `openCoreConditionDialog("plate")` retitles, opens on a
thin-section delivery and hides the core-only block; everything else is the same code. Two dialogs
would be two places for the wording, the white-balance rule and the three-state status to drift —
the `followCore.ts` argument.

**The trace and the depth strips stay core-only, and not by omission.** A thin section is cut from
ONE plug and covers no interval, so there is no axis to read a log along and nothing to stretch a
strip over — the same statement `extract_core_log` makes when it refuses a picture with no base
depth.

**Conditioning a plate is upstream of measuring it**, since `petrography.rs` reads the baked `data`.
That is the intended order — correct the plate, then measure it — and it composes with the
reference-plate correction rather than competing: a white balance done by hand leaves the reference
correction less to do, and the reference correction anchors on the matrix colour either way.

### The band, as a colour

`src/ui/colourBand.ts` is the shared control: a hue wheel laid out flat with two draggable ends, the
saturation and brightness floors as sliders whose TRACKS carry the gradient they move along, a live
swatch of what the band accepts, and the numbers still there and still typable — a band that came
off somebody else's run has to be enterable, and a value that can only be dragged cannot be written
down.

**The wheel is a canvas, one column per degree.** A band that WRAPS through red is two arcs, and
dimming everything outside two arcs with layered CSS panels is three special cases that each have to
be got right. `inBand` is the runner's own rule restated here so the picture and the measurement
agree about what a wrapped band means — refusing to draw one would make the control unable to
express a band the runner reads perfectly well.

**Pick the pore colour is the white-balance pick pointed the other way.** There a click says "this
should be neutral"; here it says "this is pore". Both replace a number nobody can picture with the
thing itself. The band keeps its WIDTH and moves its centre, because a click says "this colour is
pore", not "this is the only colour that is pore" — a band collapsed onto one hue finds almost
nothing and reads as a broken tool. The floors drop to just under what was clicked, so the very
pixel the user pointed at is inside the band it just defined.

**The colour is read from the UN-MASKED plate**, which is why `PoreResult` gained `plain_png`: the
same picture at the same size without the overlay. Clicking inside the red mask would otherwise
sample the mask and re-centre the band on the overlay's own colour, which is circular. It is sent
BESIDE the overlay rather than fetched separately — the `CorePreview.before_png` argument, so the
two can never be one plate's mask over another plate's pixels — and it is the CORRECTED picture,
because that is what the band is applied to. A small patch and its MEDIAN, not one pixel: a single
pixel on a scanned plate is as likely to be a speck as the epoxy, the same reason the white-balance
pick takes a median.

It also buys **Hold to compare** on the plate: what the band claimed, against what is actually
there.

Measured in the browser on a plate half blue epoxy and half tan grain: clicking the blue moved the
band from 180–260° to 190–270°, centred on the epoxy's own 230°.

## The delivery, as pictures, everywhere it is picked from (2026-08-01)

`src/ui/plateStrip.ts` is the filmstrip lifted out of the conditioning workspace so the MEASURING
dialogs get it too — Pore Area and the Mineral Classifier. A petrographer choosing which plate to
tune a threshold on, or which plate to point-count next, is choosing a PICTURE; a list of filenames
makes them open six to find the one they meant.

**A plate the tool cannot measure is GREYED with the reason on hover, never hidden.** "TS-2 is
there, but nobody declared it impregnated" is exactly the question the user is about to ask by
running the tool. Hiding it instead turns a refusal into a delivery that silently lost a plate —
the same argument the S-factor dialog makes for showing a text-only measurement greyed rather than
dropping it.

**And a blocked tile is still clickable.** Previewing what the band WOULD claim is how somebody
decides whether the plate is worth declaring; a greyed tile with no way to look at it is a dead end.
The refusal is on the WRITE, which is where it has always been.

**The classifier's tiles carry their own click count**, re-annotated in place rather than by
rebuilding the strip. Point counting means moving through a delivery plate by plate, and "which ones
have I already done" is the question a dropdown cannot answer without opening every entry.
`annotate` exists precisely so a count can change without a single thumbnail being refetched — the
lazy-load rule still holds, and a delivery is routinely hundreds of plates at about a megabyte each.

The counts are refreshed when the LABELS load, not only when a click is placed: the labels arrive
after the strip is built, so annotating only on click would show every tile as uncounted each time a
delivery was reopened.

