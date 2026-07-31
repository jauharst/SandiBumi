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

**1a — the registration workspace.** _(Shipped 2026-07-31 as a **dialog**, `depthRegDialog.ts`, not
a docked pane. Dialogs here are already non-blocking and pointer-transparent, so the log view stays
visible and usable beside it — the pane form buys only docking and session persistence, and can be
promoted later without touching the engine.)_ Two depth
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

**1c — per-run shift.** _(Shipped 2026-07-31, with the depth record Jauhar asked for on top: the
core keeps its as-delivered depths so a later XRD/CEC delivery written at the lab's depths can be
placed where that rock now sits.)_ A cored well is delivered in **runs**, and each run accumulates its own
tally error. One delta for the whole well is a simplification the data does not support: it will
be right in the middle of the cored interval and wrong at both ends. This increment is a table of
(interval, delta) with monotonicity enforced — two runs may not be shifted into each other's
depths, because that would reorder rock. Under the hood this is `warp_refine`'s constraint, and
the honest presentation is a piecewise-constant shift the user can read, not a smooth warp they
cannot.

**1d — everything measured on the core follows a re-registration.** _(Shipped 2026-07-31.)_ A thin
section is cut *from* a plug. If the plug moves, the section moves; a section left where the lab
wrote it while its own plug moves is now attributed to rock it was never cut from. The same is true
of an XRD table and of SCAL plugs.

What made this safe was not deciding it globally but **recording it per delivery**: `aux_sets`,
`scal_sets` and `image_sets` carry `on_core_depths`, set from the user's own declaration at import.
A core photograph registered by its own depth marks, or a perforation record on the driller's
scale, is listed but left unticked — because the opposite error, moving something that was already
right, is just as silent as failing to move something that was wrong.

**1e — plate depth editing.** _(Shipped 2026-07-31, `plateDepthDialog.ts`.)_ A table over
`update_well_image`, closing the follow-up recorded when the image track shipped, plus
`db::shift_well_images` for the whole-delivery case — a core-photograph delivery is hundreds of
plates and per-plate calls would be hundreds of IPC round trips to apply one decision. A blank base
stays a point sample throughout, and a reversed top/base is refused rather than swapped.

**1f — a registration is a record, not an edit.** _(Shipped 2026-07-31, `core_registrations`.)_
One row per moved range, written in the same transaction as the move, so a shift cannot commit
without its reason. Two things turned out to matter more than expected. It is an **event log**: an
undo appends its own reversal instead of erasing what it reversed, because a core that was
registered, judged wrong and put back is not the same as one nobody touched, and afterwards nothing
else can tell them apart. And the stored correlation is the one at the shift **actually applied**,
per **range** rather than per apply — each barrel is proposed against its own correlogram, so one
number for the operation would file the well-matched barrel's confidence against the doubtful one.
A range typed by hand records a blank, never a zero.

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
| ~~**D1**~~ | **ANSWERED 2026-07-31 — "not always, sometimes."** Which is the demanding case: the tool must handle both and must never present them as equivalent. Shipped in `registration.rs` as the like-for-like / proxy split, with the search rule differing between them (signed r vs \|r\|) and the result naming which applied. | ~~1a~~ — done |
| ~~**D2**~~ | **CLOSED 2026-07-31 — firm yes**, after an interim "yes, but its tentative". Shipped as increment 1d. The tentative stage was not wasted: it produced the explicit plate shift (1e) and the import tick-box, and it forced the realisation that "should this move?" has to be **recorded per delivery** rather than decided globally — a perforation record is on the driller's scale and must never ride along. So what moves is the deliveries the user declared as core-depth, pre-ticked and overridable, never everything. | ~~1d~~ — done |
| **D3** | **Grain size: apparent, or Wicksell-corrected?** I would default to *apparent, labelled apparent*, and offer the correction as an explicit option, so a corrected number never leaves the app without saying so. | Family B |
| ~~**D4**~~ | **ANSWERED 2026-07-31 — "sometimes yes, sometimes not, so it will be option"** for scale, and **"same, sometimes stained and epoxy, sometimes not"** for preparation. Both are therefore DECLARED per plate (defaulting per delivery), never inferred, and both default to ABSENT. See §4.1. | scale gate + A1/A2 designed, not blocked |

**D1, D2 and D4 are closed; D3 is the one live decision** and it gates only Family B. **Part 1 is
complete.**

### 4.1 What D4's answer means for the design

A uniform answer either way would have been easier. "Sometimes" is the demanding case, and it
settles three things.

**Scale is a per-plate property with no default.** It cannot be a project setting or even a
delivery setting, because one delivery holds both kinds. A delivery-level value typed once in the
wizard is a convenience that fills the blanks; the stored value lives on the plate. There is no
fallback constant — §3's "no default µm/px, ever" now has teeth, because the absent case is the
normal case rather than a corner.

**Anything dimensional refuses an uncalibrated plate rather than reporting pixels.** A D50 in
pixels is not a D50, and a number carrying the right name and the wrong unit is the same failure as
a wrong `m`: it computes, it plots, it ships. A run over a mixed delivery therefore reports how
many plates it skipped and names them — a silent subset looks exactly like a complete answer.
Family A is unaffected, which is why it stays first: an area fraction is dimensionless and every
plate qualifies.

**Preparation is declared, and A1 refuses a plate not declared impregnated.** This is the sharper
one, because the failure is silent in both directions. Run a blue-epoxy rule over a section that
was never impregnated and it does not fail — it returns a porosity assembled from blue-ish
feldspar, stain bleed and edge artefact, which then plots against core helium porosity as though
it meant something. Detecting impregnation from the pixels is the same circular move as detecting
a water zone from the saturation you are calibrating: the evidence for "this is blue epoxy" is the
blue you were about to measure. So `prepared` is a field on the plate, set at import, defaulting
to unknown, and unknown is refused. Same for the stain, whose protocol still comes from the
laboratory report per §2.1 A2.

---

## 5. Sequencing

```
Part 1   1a  registration + proposal        SHIPPED 2026-07-31 (registration.rs)
         1b  proposed best-lag              SHIPPED with it
         1e  plate depth editing            SHIPPED 2026-07-31 (plateDepthDialog.ts)
         1c  per-barrel shift + depth record   SHIPPED 2026-07-31
         1d  everything follows a re-registration   SHIPPED 2026-07-31 (D2 closed)
         1f  registration recorded            SHIPPED 2026-07-31 (core_registrations)

Part 2   2.0 scale + preparation per plate  SHIPPED 2026-07-31 (declared, no default)
         A1  pore by blue epoxy             SHIPPED 2026-07-31 (petrography.rs)
         A2  stained carbonate              needs the lab's stain protocol
         C   pore geometry                  highest cross-check value
         B   grain size                     needs the scale gate + D3

Tier 3   A3  trained mineral classifier     needs his own point counts as labels
```
