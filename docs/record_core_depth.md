# Putting core back on the log's depth scale

Build record for core-to-log depth registration: the correlogram, the per-barrel shifts,
`core_data.depth_orig` as the record, and how a late delivery follows the core it was measured on.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## Core-to-log depth registration (2026-07-31)

`registration.rs` (Data ▸ Core ▸ **Register Depth…**, `depthRegDialog.ts`) proposes the constant
shift that puts a well's core back on the log's depth scale. Until now the only tool was a number
typed into Shift Core — you had to already know the answer. Five rules.

**It is not a new algorithm.** Matching a core profile against a wireline log is the problem
`tops.rs` already solves to propagate a marker between wells, so this borrows its two primitives
(`tops::interp`, `tops::pearson`, both promoted to `pub(crate)`) instead of growing a second
implementation. `best_shift`/`warp_refine` are the same family and are what a later per-core-run
piecewise shift should reuse.

**The reference's STRENGTH is reported, because core gamma is only sometimes delivered** (Jauhar,
2026-07-31: "not always, sometimes"). A delivered core gamma against the wireline GR is
**like-for-like** — the same physical quantity, which must agree in sign as well as shape. A core
porosity against GR is a **proxy**: different quantities that co-vary, and *inversely*, because the
shaly intervals that raise GR are the ones that lose porosity. So the search rule is
**like-for-like → maximise r; proxy → maximise |r| and report which sign won**, and the result says
which it did. A coefficient of −0.82 means "well aligned" in one case and "something is wrong" in
the other; a number that reads the same in both is a number that misleads. Pinned from BOTH sides
by `a_porosity_proxy_registers_on_the_inverse_relationship` (fails on a signed score) and
`a_like_for_like_pairing_never_accepts_an_inverted_alignment` (fails on |r| everywhere) — either
test alone would let the lazier implementation through.

Family resolution goes through `registration::reference_family`: `curves::family_for` first, then a
LOCAL `CORE_FAMILIES` table for POR/PERM/GD/SW. Those are deliberately not added to
`curves::FAMILIES` — that table drives curve resolution for the whole project, and widening it to
settle a labelling question here would change how every module finds its inputs. `bare_mnemonic`
strips CORE/PLUG/LAB tokens so `CORE_GR` and `GR` are the same measurement. An unrecognised name is
a **proxy, never a guessed match**.

**The whole correlogram is returned, not just its peak.** One sharp peak means the shift is
determined; a comb of near-equal peaks means the section repeats and the maximum is a coin toss —
the same number, completely different situations. The dialog draws r against shift on a fixed −1..1
axis (cropping to the data's own range makes a weak peak look decisive) and counts rival peaks
within 5% into a note. **Nothing is applied automatically**: the proposal populates an editable
field and the user accepts.

**A candidate shift must keep `MIN_PAIR_FRACTION` (0.75) of the best-populated shift's pairs**, and
at least `MIN_PAIRS` (8) outright. Without that floor, sliding the core off the end of the log is a
legitimate way to win — the few plugs still overlapping can correlate almost perfectly by chance,
and the scan would return a large shift with a beautiful coefficient computed from almost no data.
The log is interpolated onto the plug depths rather than the core resampled onto the log: core is
sparse and irregular, and resampling it would invent samples between plugs that then vote.

**A depth shift moves the plugs and the measurements made ON those plugs, together, in one
transaction.** `db::shift_core_depths` gained an `aux_data` pass and returns `CoreShiftCounts
{plugs, extras}`. Core extras (core gamma, lithology, Kv/Kh) live in `aux_data` under the core
delivery's OWN set name, so moving `core_data` alone silently decoupled every one of them: the
porosity would register against the log while the core gamma that JUSTIFIED the shift would not, and
a second pass would compute a fresh non-zero shift from the same core. Nothing downstream can detect
that. **Which datasets ride along is NOT inferred from the set name alone** — a separately imported
XRD delivery is also called RAW by default — so `db::core_extra_datasets` returns the candidates and
the dialog lists them with checkboxes before applying. Whether an XRD or CEC suite belongs to these
plugs is a core-handling judgement, not something to guess. Pinned by
`a_core_shift_carries_the_plug_extras_and_leaves_other_deliveries_alone`, which also checks that an
interval sample keeps its thickness (`depth_base + delta` is NULL-safe, so a point stays a point)
and that the whole thing reverses exactly, which is what makes it undoable.

**Plate depths (2026-07-31)** — `plateDepthDialog.ts` (Data ▸ Tools ▾ ▸ **Plate Depths…**) is the
missing caller for `update_well_image`, which had been written and tested since the image track
shipped with nothing invoking it: a thin section delivered at the wrong depth could only be fixed by
deleting the delivery and importing it again.

**An empty base means a POINT sample and stays one.** `depth_base IS NULL` is a petrophysical
statement — a section is cut from one plug and has no thickness — so a blank field is never filled
in from the plate below, and typing a base is a deliberate claim that the picture spans an interval
(reversible by clearing it). A base ABOVE the top is **refused, not silently swapped**: a reversed
pair is a typo or a wrong column, and guessing which hides it.

`db::shift_well_images` moves a whole delivery in ONE statement, following `ACTIVE_IMAGE_SET` like
every other image reader. Per-plate `update_well_image` calls would be hundreds of IPC round trips
for a core-photograph delivery, which is exactly the delivery most likely to be off by one tally
error. `depth_base + delta` is NULL-safe, so a shift moves a point sample without giving it a
thickness — pinned by `shifting_plates_moves_the_live_delivery_and_keeps_a_point_a_point`, which
also checks that an interval keeps its span and that a superseded delivery does not move.

**D2 is answered TENTATIVELY (Jauhar, 2026-07-31: "yes, but its tentative")** — thin sections should
follow their plugs when core is re-registered. A tentative yes is deliberately NOT wired as an
automatic link: what shipped is the explicit bulk shift above, which the user applies knowing they
applied it. Making plates ride `shift_core_depths` silently is increment 1d and waits on a firm
answer, because a picture that moves without being asked is the same class of error as a core extra
that fails to.

**Per-barrel shifts and the core depth record (2026-07-31)** — core comes up a barrel at a time and
each barrel carries its own tally error, so one number for a whole well is right in the middle of
the cored interval and wrong at both ends. Pieces also move INSIDE a barrel between the core face
and the lab bench, which is why `db::RunShift` is a free interval rather than a fixed barrel length:
splitting a row into two shorter rows is how that case is handled. UI is the barrel table in
`depthRegDialog.ts`, where each row proposes its own shift through the same `registration.rs`
engine restricted to that range.

**`core_data.depth_orig` is the record**, added by `db::migrate_core_depth_orig` (non-destructive —
one ADD COLUMN and a back-fill, so unlike `migrate_point_data_sets` it needs no backup; it must run
AFTER that one, which rebuilds the table). `depth` is where the rock is, `depth_orig` is where the
lab said it was, and **nothing ever shifts `depth_orig`**. It must stay the LAST column: the
Appender is positional and a migrated database gets it appended.

That column is what makes a later delivery follow. An XRD or CEC table arrives months after the
core was registered, still written at the depths the core report used; `db::core_depth_pairs` +
`db::map_core_depth` place it where that rock now sits. **The map lives in the core itself rather
than in a side table of shift history** — it survives per-barrel shifts, single-plug nudges and
re-registrations with no bookkeeping, and cannot drift out of sync with the data it describes.
Between plugs the correction is INTERPOLATED, which is the whole point when pieces moved inside a
barrel: the offset genuinely varies along the core. Outside the cored interval the nearest end's
correction is held and the result is flagged `extrapolated`, because there is no evidence out
there and a caller must be able to show which samples were guessed.

Two rules `apply_core_run_shifts` enforces, both in a Rust dry run before anything is written:
**no set of shifts may reorder the core** (two barrels shifted into each other's depths would put
deeper rock above shallower rock, and no reader downstream could tell), and **two ranges may not
overlap** (across a real overlap the first match silently wins and "which barrel was this plug in?"
stops being answerable). Ranges that TOUCH at one depth are fine — `2000–2010` and `2010–2020` is
how anyone writes two adjacent barrels — and the shared depth goes to the first range listed.

**The inverse is computed by the backend and returned in `CoreShiftCounts.inverse`; a caller must
never build its own.** Negating each delta and shifting the caller's own ranges looks equivalent
and is not: two barrels that never overlapped can land on overlapping ranges once each moves by a
different amount, and first-match-wins then returns some plugs by their neighbour's correction.
The returned boundaries sit halfway between one run's deepest plug and the next run's shallowest,
so every plug is inside its own range and none is inside two. Pinned by
`undoing_per_barrel_shifts_returns_every_plug_to_where_it_started`, which asserts the naive inverse
really does overlap before checking the computed one does not.

The write itself is ONE set-wise `UPDATE ... CASE`, not a row per plug, because `depth` is part of
the primary key: moving 1000→1001 row by row collides with the plug already at 1001 even when the
finished result is perfectly valid. An interval sample is placed by its TOP so a barrel boundary
cannot split one sample into two different shifts, and its base moves by the same amount.

**A late delivery can follow the core (2026-07-31)** — `ingest::import_aux_file` gained
`follow_core`, exposed as the **"These depths came from the core report"** tick-box in Data ▸
Import Aux…. A laboratory writes the depths from the original core report; if that core has since
been registered against the log, those depths are stale by exactly however far the core moved, and
the samples get attributed to rock they were never measured on. With the box ticked each row is
placed through the target well's `core_depth_pairs` map.

**Off by default, and never silently on.** A file already written on the log's depth scale must not
be moved, and there is nothing in a delimited text file that reliably says which scale it uses — so
this is the user's declaration, exactly as the RtC fit's water zone is. The mirror case is covered
too: ticking the box on a well with no core, or where the record cannot be read, imports unmapped
and SAYS so in the notes rather than appearing to have mapped something.

**The mapping is per WELL**, resolved inside the row-building closure rather than once per file,
because a multi-well delivery routes by its WELL column and each well has its own core record.

**An interval is placed by its TOP and its base takes the same offset** — the same rule the barrel
shifts use. Mapping the two ends independently could invert a thin sample where the correction
changes steeply across a barrel boundary, and a sample that measured 20 cm of rock still measured
20 cm of rock.

Three things are reported rather than assumed: samples that fell **outside the cored interval**
(placed by holding the nearest correction — there is no evidence out there), a core that has **not
been shifted** (so the box worked and simply had nothing to correct, which beats silence), and a
well with **no core to follow**. Pinned by
`ingest::tests::a_late_delivery_can_follow_the_core_it_was_measured_on`, which registers two
barrels by different amounts and checks a sample from each lands on its own barrel's correction.

Not yet wired the same way: SCAL and image imports. Both take lab-written depths and both would
benefit; neither is offered yet.

**Following the core is now offered everywhere lab depths arrive (2026-07-31)** — the tick-box
added for point data extends to **SCAL** (`ingest::import_scal_files(..., follow_core)`) and
**plates** (`images::ImageImportRequest.follow_core`, `#[serde(default)]` so an older payload still
deserializes). All three are measured ON core and all three carry the depths the core report used.

`src/ui/followCore.ts` is the one control, shared by the three dialogs — it is the same decision,
and three copies of a checkbox is three places for the wording to drift.

SCAL plugs ARE core plugs, so their depths map directly; a record with **no depth is left alone**
because there is nothing to correct, and that case gets its own note rather than being folded into
"placed". For plates the top is mapped and **the base takes the same offset**, so a core photograph
keeps the thickness it was logged with — the same rule the barrel shifts and the point-data import
use, and for the same reason: mapping the two ends independently could invert a thin plate where
the correction changes steeply at a barrel boundary. A section with no base stays a point sample.

`ScalImportResult` gained `note` for this; `ImageImportResult.note` already existed and now carries
it alongside the unit-conversion and Pillow messages. Pinned by
`ingest::tests::scal_points_can_follow_the_core_they_were_cut_from`,
`images::tests::plates_can_follow_the_core_they_were_cut_from` (which checks the photograph keeps
its 1 m while the section stays a point) and
`images::tests::following_a_core_that_is_not_there_says_so`.

**The image tests needed a real JPEG.** `tiny_jpeg()` in `images::tests` is a header-only stub —
correct for exercising `sniff`, and Pillow refuses it, so anything going through `import_images`
fails on it. `REAL_JPEG_HEX` is a genuinely decodable 159-byte 2x2 greyscale JPEG, which works on
BOTH paths: Pillow decodes it, and the no-Pillow fallback stores a JPEG verbatim. Do not swap it
back for the stub.

**D2 is now answerable by doing rather than deciding.** Jauhar's tentative "yes" on plates
following plugs is served by the explicit tick-box at import; wiring plates into
`shift_core_depths` so an ALREADY-imported delivery moves automatically is still increment 1d and
still waits on a firm answer.

**Already-imported deliveries follow a later re-registration (2026-07-31, increment 1d)** — a core
registration moves rock that other deliveries were measured on, so `db::shift_core_depths` and
`db::apply_core_run_shifts` now take a `ShiftTargets { aux_datasets, scal, image_datasets }` and
carry the chosen point datasets, the live SCAL delivery and each chosen image delivery with the
plugs, in the same transaction. `CoreShiftCounts` reports `plugs / extras / scal / plates`.

**Which deliveries belong to the core is RECORDED, not guessed.** `aux_sets`, `scal_sets` and
`image_sets` gained `on_core_depths`, written from the user's own "these depths came from the core
report" declaration at import (`db::mark_aux_set_on_core` and its two siblings). Without it there
is no way to tell a core-depth delivery from a log-depth one, and moving the wrong one is silent —
a perforation record is on the driller's scale and must never be dragged along with the core.
Migration `db::migrate_delivery_depth_basis` is ADD COLUMN only (no rebuild, no backup) and gives
existing deliveries **0**, the safe answer: an older delivery is left alone rather than moved on a
guess.

`db::core_shift_candidates` lists every live delivery **with** its flag rather than filtering by
it, because the flag only exists for deliveries imported since it did — filtering would make an
older project look as though it had nothing to move. The dialog pre-ticks the flagged ones, lists
the rest with "not marked as core-depth data", and lets the user override either way.

**The tick-boxes live at dialog level, not inside the result block**, so the single-shift Apply and
the per-barrel Apply use the SAME choices. They were briefly inside `renderResult`, which meant the
barrel path silently ignored them — caught in the browser, not by the compiler.

`ShiftTargets` is `Option` at the command boundary: **omitted** means "the extras that provably
came in with the core table" (the old behaviour, still what `Shift Core…` uses), an **empty object**
means plugs only. The two must stay distinguishable.

**The core carries its own depth history (2026-07-31, increment 1f)** — `core_registrations` holds
one row per moved range, written by `db::write_registration` inside the SAME transaction as the
move (`shift_core_depths` and `apply_core_run_shifts` both take a `RegistrationNote` now). Not a
separate "log it afterwards" call: a depth registration that committed without its reason is
exactly the state this exists to prevent. There is deliberately no "do not record" value —
recording is the default, and `RegistrationNote::default()` means a manual shift.

**It is an EVENT LOG, not a state table.** An undo appends its own reversal rather than deleting
the row it reverses. Deleting would make the record agree with the current depths at the cost of
the only question it answers: a core that was registered, judged wrong and put back is not the
same as a core nobody ever touched, and nothing downstream can tell those apart afterwards. Pinned
by `an_undo_appends_a_reversal_instead_of_erasing_the_record`, which also checks the plugs really
are back where they started — so the log is the only thing that still remembers.

**The correlation stored is the one at the shift ACTUALLY applied, not the peak of the scan.** The
user is free to overrule the proposal (`correlationAt` in `depthRegDialog.ts` reads the applied
delta off `res.scan`), and filing the peak would describe an alignment nobody chose. Outside the
scanned window nothing is stored rather than extrapolated.

**Agreement is per RANGE, not per apply.** Each barrel is proposed against its own correlogram, so
`RunShift` gained `correlation` / `n_pairs` (`#[serde(default)]` — absent on the computed inverse
and on any range typed by hand). One number for the whole operation would file the well-matched
barrel's confidence against the doubtful one. **A blank is "not measured", never zero** — a 0.00
there would read as a registration that matched nothing.

`top`/`base` are NULL for a whole-core shift, a statement rather than a missing field: no range was
declared, so the correction applied everywhere. `seq` counts within (well, set) rather than keying
on the timestamp — two applies can land in the same microsecond, and a primary-key collision there
would fail the SHIFT, not just its record. The set name is STORED as it was at the time, so
switching the active delivery later cannot rewrite what this one has been through.

No migration: `CREATE TABLE IF NOT EXISTS` runs on every open, the `ml_models` precedent. Nothing
is written when a shift moved no plugs.

