# Core photographs: what is built, and what the two lights are for

Companion to `docs/plan_image_analysis.md`, which covers thin sections. This one covers whole-core
and slab photography: the delivery shape, the two lights, and the analyses that follow from each.

Written 2026-08-01, from a real whole-core delivery and from Jauhar's question: *"for core photo,
sometimes we have 2 kind of em, UV and daylight or both, give me option or recommendation how to
deal with it… i.e. extract inferred payzone from UV, or extract sand shale bed as discrete with
simple 'unfold' features because sometimes we have dipping bed"*.

No client identifier appears here — the delivery is described by SHAPE, per the provenance rules.

---

## 1. What a whole-core delivery actually looks like

Not a folder of core-box photographs. A **core-display plate**:

- One PDF per cored run, pages in pairs — plate `Na` **white light**, plate `Nb` **UV**, the same
  rock, the same columns, the same depths.
- Each page carries **four COLUMNS** of core side by side, core running DOWN each column, columns
  read left to right.
- Each column is its own **barrel**, captioned with its own top and base (a fixed length, commonly
  3 decimal feet), plus intermediate depth tags printed on the picture.
- A **depth ruler** down the left, a title block above, a caption below.
- **Gaps are printed as gaps**: a white box reading `PRESERVED <top>-<base>` where an interval was
  taken for preservation, and a short last column where recovery ran out.

Three consequences, and all three are why the equal-lane model was not enough:

1. Four columns of one plate are **four separate stretches of depth**, not twelve continuous feet.
2. The last column of a run is routinely **shorter than the others**.
3. The page is mostly **not core** — ruler, title, caption, margins.

## 2. Built (2026-08-01)

- `Lane` / `PlateLayout` / `CoreLogSpec.layouts` — per-picture columns, each with its own barrel
  depths, plus the fraction of the down axis that is core. Rules and their reasoning live in
  `CLAUDE.md`; `plan_lanes` is the one place they are enforced and is unit-tested without Pillow.
- `detect_core_lanes` — proposes the columns from the picture's own brightness profile, returns the
  whole profile so the split can be judged, and never guesses a depth.
- **Photo Log…** (`coreTraceDialog.ts`) — the conversion as its own tool, with the column table.
- `recommend_core_recipe` — measures a picture and proposes conditioning with a stated reason per
  value; declines on a UV plate and says why.

## 3. The two lights

**They are two measurements of the same rock, not two versions of one picture.** So they are two
DELIVERIES (datasets), which is what the delivery-set model already gives, and which is why
Condition Core Photos can already hold one against the other and Photo Log writes strips per source.

| | White light | UV |
|---|---|---|
| What it shows | lithology, grain size, structures, colour | hydrocarbon fluorescence |
| Reads as | sand / shale, bedding, contacts | oil show, its extent, its brightness |
| Wants | white balance, exposure, contrast | **none of those** |
| Trace | `CPHOTO_DARK` / `_RED` / `_TEX` | a fluorescence measure (§4) |

**A UV plate must never be conditioned like a white-light one.** It is meant to be dark; the
background IS the answer; there is nothing neutral to balance against because a UV lamp is not white
light. `recommend_core_recipe` recognises this and declines — see `CLAUDE.md`. What a UV plate *can*
take is a gentle exposure lift applied to BOTH plates of a delivery identically, and only if the
extent is being compared across boxes rather than measured.

## 4. Next, in order

Each item states what it produces and the one rule that keeps it honest.

### 4a. PDF plate import — ~~the blocker~~ **NOT BUILDING (Jauhar, 2026-08-05)**

> *"dont try to import pdf, user will just provide photo"*

**Closed by decision, not by implementation.** The user will export the plates from the PDF himself
and import them as ordinary pictures, which the existing wizard already handles — so this stops
being a blocker and the rest of §4 is reachable today.

Recorded rather than deleted, because the REASON matters to anyone who meets the same delivery: it
is not that a PDF extractor would be hard or wrong, it is that the manual export is a few minutes
per core and buys a route with no caption-parsing guesswork in it at all. If the deliveries ever
arrive by the hundred, everything below is still the design to build.

What that decision costs, stated so it is not a surprise later: the plate captions carry the well,
the core number, the light and each column's depths, and a hand export loses all of it. **Those
depths have to be typed into the column table in Photo Log**, which is exactly what the per-barrel
lane table exists for — and the alternating `Na`/`Nb` page pairing becomes the user's choice of
which folder is white light and which is UV, declared at import as two datasets.

<details><summary>The design, if it is ever needed</summary>

Same class of barrier as the petrography workbook, and the same shape of answer: an **extractor**,
not a second importer — lift each page to an image, hand it plus a depth table to `import_images`,
so the set model, the long-edge cap, `follow_core` and everything else applies unchanged. Two
importers would eventually disagree.

The plate captions carry the well, the core number, the light and each column's depths, in text the
PDF has. Reading them beats retyping four barrels per page across a hundred pages — under the
workbook rule: **take a depth only where a unit follows it**, and show every guess in the editable
table before anything is stored.

Alternating `Na`/`Nb` pages should land in TWO datasets (white light, UV) rather than one, and the
plate suffix is the evidence for which — declared in the wizard, never inferred silently.

</details>

### 4b. `CPHOTO_FLUOR` — the inferred show from UV  ✅ **SHIPPED 2026-08-05**

Fraction of each slab whose fluorescence exceeds a threshold, plus its mean brightness.

**It is an indicator, never a pay flag, and its name must never become one.** The `CPHOTO` prefix
exists for exactly this reason: fluorescence is not saturation. Mineral fluorescence, drilling-fluid
additives and dead oil all fluoresce; live oil is typically bright yellow-green while dead oil and
some minerals go dull blue-white, so a HUE band is part of the measure and its default is a starting
point for tuning, not a calibration. Calibrating it against a real Sw or a show report is the user's
work, and the note must say so — the `gr_normalize` discipline.

**The threshold is tuned against a preview on one plate**, and — the lesson the petrography suite
paid for — judged by AGREEMENT with an independent measurement, never by whether the average looks
right. Matching a median is the one statistic that survives a segmentation which has stopped
discriminating.

### 4c. A discrete sand/shale curve

`CPHOTO_LITH`, a class curve off the white-light trace, rendered by the existing `fill: "blocks"`
track.

**Two classes from a threshold on darkness is a rock-fabric description, not a lithology**, and the
name has to keep saying so. It is worth having because a discrete curve is what a correlation panel
and a facies tie-in can consume, and because a core photograph resolves beds far finer than a
wireline log does. The cut should be proposed from the trace's own distribution (the two-population
split `detect_core_lanes` already uses) and be adjustable, with the class count reported — a
delivery whose darkness has one mode has no two classes in it.

### 4d. "Unfold" — the dipping bed

A slab average across the core smears a dipping contact over the core's width: a bed at 30° across a
10 cm slab is spread over ~6 cm of apparent depth, which is most of a thin bed. Unfolding shears
each slab to the bed's apparent dip before averaging, which sharpens every contact in the trace.

**The dip is measured from the picture and must be reported, not assumed.** The apparent dip of the
layering is estimable from the dominant orientation of the image gradient (a structure tensor over a
window); where the rock is massive there is no orientation to find and the honest answer is no
shear, reported as such. A per-slab dip that swings wildly is the signature of a picture with no
bedding in it, and applying it would carve structure into a homogeneous sand.

**It is an APPARENT dip in the plane of the slab**, not a true dip: the slab is one cut through the
core, so the number cannot be quoted as a structural measurement and the curve must not be named as
though it were. It sharpens the trace; it does not orient the well.

---

## 5. Open questions for Jauhar

1. **PDF captions** — is the well/core/depth text in these plates reliable enough to read
   automatically (with every value shown before storing), or should the import always ask?
2. **Fluorescence hue** — does his show-description practice distinguish bright yellow-green from
   dull blue-white, and is that a distinction `CPHOTO_FLUOR` should carry as two measures rather
   than one?
3. **Unfold** — worth it on the deliveries he actually gets, or is the bedding in them close enough
   to horizontal that the shear buys nothing?
