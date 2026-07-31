# Phase plan — depth registration, then plate digitizing

_Drafted 2026-07-31, from Jauhar's scoping answer: **all three measurement families** (modal
analysis, grain size, pore geometry), and **"it should be depth registered first, then the
quantification or qualitative analysis."**_

This is the plan document, not an increment. It says what the two tiers contain, what already
exists that they build on, what must never be automated, and the four decisions that are his to
make. `ROADMAP.md` C2 (7) and the §B3 OpenCV note both point here.

---

## 0. Why the ordering is right, and what it costs us

A measurement carries two numbers: the value and the depth it belongs to. Digitizing produces the
first. Nothing in this repo can check the second — a modal analysis attributed 3 m too shallow is
a confident, well-formed, plausible number on the wrong sand, and it will survive every gate we
have, exactly like a wrong `m` survives `cargo check`.

We already know this failure is live rather than hypothetical. The S-factor fit drops any plug
further than 0.15 m from a log sample, and its own test records the limit honestly: **a shift
that is a whole number of sample intervals is invisible to any depth-tolerance check.** The
tolerance protects against a plug delivered at the wrong depth; it cannot see a core delivered on
the wrong datum. That is a registration problem, and no downstream statistic can find it.

So the cost of doing registration first is a tier of work before the first pretty picture, and
the benefit is that every existing core-based calculation — φ-k, the S calibration, the SHF fit,
the rock typing, the facies tie — gets the same fix. The plates are the newest consumer of core
depth, not the only one.

---

## Part 1 — Depth registration

### 1.0 What exists today

| Piece | State | Gap |
|---|---|---|
| `db::shift_core_depths(well, delta)` | works, tested | one rigid delta for a whole well, typed as a number into a ribbon prompt. No preview, no reference curve, no correlation, no per-run breakdown. |
| `db::update_well_image(...)` | works, tested, `updateWellImage` in `ipc.ts` | **no caller.** A plate's depth cannot be corrected from the UI at all; re-import is the only route. |
| `tops.rs` correlation engine | `best_shift`, `pearson`, `normalize_window`, `subseq_dtw`, `warp_refine` | used **only** for propagating markers between wells. It is a general best-lag + monotone-elastic matcher and never sees core. |
| Delivery sets | universal (`ACTIVE_CORE_SET`, `ACTIVE_IMAGE_SET`, ...) | a re-registration is a new interpretation of the same delivery, and the set model has no opinion about that yet. See decision **D2**. |

The important line in that table is the third. **Core-to-log registration does not need a new
algorithm.** Matching a core gamma profile against a wireline GR is the same problem as matching
a marker's log shape between two wells: find the lag that maximises correlation, then optionally
relax to a monotone warp for differential tally error. `tops.rs` already does both, is already
tested, and already enforces the constraint that matters (the warp cannot reorder anything).

### 1.1 Increments

**1a — the registration pane.** Not a dialog: this is a task you look at for a while. Two depth
tracks side by side, wireline reference on the left, core measurement on the right, a draggable
shift with a live Pearson readout and the sample count behind it. Apply writes through
`shift_core_depths` on the **active** core set, and is undoable.

The core side needs a reference. In order of strength: delivered **core gamma** (a point dataset,
already importable), then **core φ against a log porosity**, then core grain density against
RHOB. Only the first is a like-for-like comparison — the others are comparisons of two different
physical quantities that happen to co-vary, and the pane must say which one it is looking at
rather than presenting a correlation coefficient as if all three meant the same thing. See **D1**.

**1b — proposed shift, never applied shift.** `tops.rs::best_shift` over the displayed interval
returns a lag and an r. It populates the shift box and draws the proposed alignment; the user
accepts. This follows the marker autocorrelation precedent exactly, and for the same reason: a
correlation maximum is a suggestion, and a core with a repeated sand can have a very good one in
the wrong place.

**1c — per-run shift.** A cored well is delivered in **runs**, and each run accumulates its own
tally error. One delta for the whole well is a simplification the data does not support: it will
be right in the middle of the cored interval and wrong at both ends. This increment is a table of
(interval, delta) with monotonicity enforced — two runs may not be shifted into each other's
depths, because that would reorder rock. Under the hood this is `warp_refine`'s constraint, and
the honest presentation is a piecewise-constant shift the user can read, not a smooth warp they
cannot.

**1d — plates follow their plugs.** A thin section is cut *from* a plug. If the plug moves, the
section moves; a section left where the lab wrote it while its own plug moves is now attributed
to rock it was never cut from. Whether that link is automatic is **D2** — it is a real choice,
because a core photograph is registered by the photograph's own depth marks and may deserve to
stay put while the plugs move.

**1e — plate depth editing.** A small table over `update_well_image`, closing the follow-up
recorded when the image track shipped. Independently useful and independently small; it may go
first if the pane takes longer than expected.

**1f — a registration is a record, not an edit.** Whatever the pane applies should leave the
answer to "why is this core at this depth?" — the shift, the reference used, the correlation, the
date. This is a small down-payment on Phase 11 lineage and is cheap while the code is being
written; it is expensive to retrofit.

---

## Part 2 — Digitizing

Only after Part 1, and gated by one physical fact.

### 2.0 The scale gate

**A pixel is not a micron.** Nothing dimensional — grain size, pore size, throat radius — can be
computed from a plate that does not carry its scale, and a plate's stored copy in this project is
a JPEG normalized to a 2400 px long edge, which means the *original's* µm/px is not the stored
copy's. `src_width`/`src_height` are already kept precisely so the ratio is recoverable, but the
delivered scale has to come from somewhere. Three routes, in order of reliability:

1. **A stated scale per delivery** (µm/px, or field-of-view width, or objective magnification with
   a known camera) typed once in the import wizard.
2. **A drawn calibration**: the user drags a line along the plate's own scale bar and types its
   length. Per plate, tedious, and the only option when the delivery is heterogeneous.
3. **Embedded resolution tags** (TIFF `XResolution`, EXIF). Present sometimes, correct less often,
   and never to be trusted silently — at most a pre-filled guess in route 1.

Consequence for sequencing: **modal analysis is dimensionless and can ship before the scale
question is settled.** Area fractions, aspect ratios and shape factors need no calibration. Grain
size and pore size do. That is why the three families below are ordered as they are, and it is not
an arbitrary order.

### 2.1 Family A — modal analysis (point counting without the point counter)

The deliverable is an area fraction per phase, which under the standard stereological argument
(Delesse) estimates the volume fraction. Two levels, and the distinction is not cosmetic:

**A1 — pore, by blue epoxy.** Sections impregnated with blue-dyed epoxy separate pore from solid
in colour space about as cleanly as any petrographic problem separates. Output: `VPORE_TS` (and
its complement), a genuinely useful curve to plot against core helium porosity — where the two
disagree, the disagreement is informative (microporosity below the resolution of the section,
plucking, epoxy that did not penetrate).

**A2 — stained carbonate.** Where the lab stained (alizarin red S for calcite, potassium
ferricyanide for ferroan phases), those colours are diagnostic by design, and segmenting them is
legitimate. **The stain protocol must come from his laboratory report, not from this plan** — the
same rule as any other parameter. A stain assumed is a mineral fraction invented.

**A3 — everything else is a classifier, not a colour rule.** Quartz against feldspar in plane
light is not a colour problem, and any code that claims otherwise is producing numbers with the
shape of a modal analysis and none of the content. The honest route is the ML suite we already
have (`ml.rs`, supervised classification), trained on **his own point counts** on **his own
sections** — which makes it a Tier 3 item that cannot start until there is labelled data. It is
listed here so it is not later mistaken for a gap.

### 2.2 Family B — grain size

Needs the scale gate closed. Watershed segmentation of the grain binary, then apparent 2D
intercepts → D50 and a sorting coefficient.

**The stereological caveat belongs in the output, not in a footnote.** A random plane through a
population of grains rarely cuts any of them through the centre, so 2D apparent diameters are
systematically *smaller* than true 3D diameters, and apparent sorting is systematically *worse*.
The classical correction is Wicksell's. Whether we apply a correction or report apparent values
and label them "apparent" is **D3** — reporting apparent values labelled honestly is the safer
default, because a correction has assumptions of its own (grain shape, size distribution family)
that a deliverable would then be carrying invisibly.

### 2.3 Family C — pore geometry

Also dimensionless in part, which is useful: pore aspect ratio, perimeter-to-area shape factor and
the pore-area *distribution shape* need no scale; absolute pore areas do.

This is the family with the most existing machinery to plug into. `thomeer.rs`, `hfu.rs` and
`rocktyping.rs` already partition rock by pore-system character inferred from Pc and φ-k. A
thin-section-derived pore-type fraction at the same depths is an **independent** cross-check on a
rock type that is currently inferred entirely from bulk measurements — arguably the highest-value
output in the whole tier, and the one that most needs Part 1 to be right, since it is compared
against curves sample by sample.

### 2.4 Storage: nothing new

- **One number per plate** (VPORE_TS, D50, mean aspect ratio) → `aux_data`, at the plate's depth,
  under a delivery set. Every existing reader, plot and picker then sees it for free, including
  the S-factor dialog's item catalogue.
- **A distribution per plate** (the grain-size histogram, the pore-area distribution) → the
  `array_logs` store, which is exactly a vector of values at one depth. `distribution.rs` already
  computes percentiles the same way everywhere.
- **The segmentation mask** is a derived picture. It should be storable as a plate in its own
  image set beside the original, so the user can *look at what was counted* — a modal analysis you
  cannot audit visually is a number to distrust.

### 2.5 The runner

OpenCV is **not installed** and is a subprocess, exactly like Pillow, `dlisio`, scikit-learn and
the office spine (rule 7). It reads `sys.stdin.buffer`. A missing `cv2` fails **this button** with
a message naming the interpreter and the pip command — never the app, never the image track,
never the import. `python_status()` gains a probe so a dialog can say so before a run.

Nothing decodes pixels in Rust. Rust decides what to measure and stores the result; the runner
draws and counts.

---

## 3. What must not be automated

- **The lag is proposed; the user accepts it.** A correlation maximum in a repetitive sand section
  is confidently wrong.
- **A plate that cannot be segmented is reported by name**, never dropped from an average. Same
  rule as the PDF's named frame for an unembeddable plate: a deliverable must be checkable against
  the delivery list.
- **A modal fraction never becomes a curve by interpolation.** Sections exist where somebody cut
  one; joining them into a continuous log states a continuity the data does not have. That is the
  same argument that made point data a track kind rather than a `CurveStyle`.
- **No default µm/px, ever.** It is not a physical constant; it is a property of a microscope.
- **No stain assumed**, per §2.1 A2.

---

## 4. Decisions for Jauhar

| # | Decision | Blocks |
|---|---|---|
| **D1** | **Do your deliveries include core gamma?** If yes, 1a is a like-for-like match and is straightforward. If not, registration has to key on core φ vs a log porosity, which is a weaker comparison and changes how the pane presents its correlation. | **1a — the first increment.** |
| **D2** | **When core is re-registered, should thin sections and SEM plates at those depths move with it?** My reading is yes for sections (cut from the plug) and no for core photographs (registered by their own marks) — but this is a core-handling question, not a software one. | 1d |
| **D3** | **Grain size: apparent, or Wicksell-corrected?** I would default to *apparent, labelled apparent*, and offer the correction as an explicit option, so a corrected number never leaves the app without saying so. | Family B |
| **D4** | **How do your sections carry their scale** — a bar burned into the image, a stated magnification, a µm/px column in the lab spreadsheet, or nothing? And **are they blue-epoxy impregnated**, and stained? | Families B and C; A1 needs the epoxy answer. |

Only **D1** blocks the first increment. D2–D4 can be answered while Part 1 is being built.

---

## 5. Sequencing

```
Part 1   1e  plate depth editing            small, closes a shipped follow-up
         1a  registration pane              needs D1
         1b  proposed best-lag              reuses tops.rs
         1c  per-run piecewise shift
         1d  plates follow plugs            needs D2
         1f  registration recorded

Part 2   2.0 scale calibration              needs D4
         A1  pore by blue epoxy             dimensionless, first real digitizing
         A2  stained carbonate              needs the lab's stain protocol
         C   pore geometry                  highest cross-check value
         B   grain size                     needs the scale gate + D3

Tier 3   A3  trained mineral classifier     needs his own point counts as labels
```
