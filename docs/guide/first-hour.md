# SandiBumi — your first hour

This guide takes a petrophysicist from a fresh install to a finished one-well
interpretation, following one path. It is not a reference manual: each chapter does one
thing, in order, on the example dataset that ships in the repository
(`dataset for test/examples/` — three synthetic wells, **SANDI-01/02/03**, one shared
geology: a shale cap, a gas sand, a water sand, a base seal). Everything photographed
here is the real application driven through this exact flow.

**Setting up the machine is not this guide's job.** `CONTRIBUTING.md` §1–2 is the
install checklist, and `docs/INSTALLATION_PREREQUISITES.md` lists exactly which
capabilities need an optional Python package and what happens without it. The short
version: the core application — projects, LAS import, log views, plots, every Rust
petrophysics module, PDF/SVG export — needs **no Python at all**. Python (with numpy)
adds user-written equations; `dlisio` adds DLIS import; `scikit-learn` adds the ML
suite; Pillow adds picture imports beyond what the window itself can decode;
`xlsxwriter` / `python-docx` / `python-pptx` add the Excel / Word / PowerPoint
deliverables. A missing package fails only its own button, with a message naming the
interpreter and the pip command — never the application.

## The path

1. **Before you start** — what SandiBumi is, what is optional, and the example dataset.
2. **First launch and your project** — the app opens straight into a workspace; the
   Project tab owns New / Open / Save Project As / Recent, and the dot that means
   unsaved work.
3. **Import your first wells** — LAS import and the curve-set dialog: naming the
   delivery, attach, declaring the sampling style, and reading the import warnings.
4. **The Wells & Tops pane runs the workspace** — clicking a well or a top changes what
   every panel shows; the ▸ twisty lists each well's deliveries curve by curve.
5. **Reading the log view** — layouts, depth scale, tracks, the neutron-density
   crossover, and adding a track of your own.
6. **Run your first module** — *written below.*
7. **Where results live** — the Curve Catalog, log-set versions, Ancestry, the
   Processing History — and why a module run is re-run, never undone.
8. **Export something** — Export LAS from the Data tab; a print-scale composite PDF
   from the Plot tab.
9. **Where to go next** — the Advance tab methods, per-panel Help, and the deeper
   documents in `docs/`.

Chapters 1–5 and 7–9 follow after the voice of chapter 6 is agreed.

---

## 6. Run your first module

You have three wells with GR, resistivity, neutron and density, and tops splitting them
into a shale cap, two sands and a base seal. The first interpretation step is the same
as it would be on paper: a shale volume from gamma ray. This chapter runs it, and on the
way meets the four mechanics every later module run reuses — where parameters come
from, who the run says it was run by, what "degraded" means, and where the output went.

### Open the module

Modules live in the **Petrophysics** ribbon tab, grouped the way you would group them
yourself: Data Prep, Condition, Frame, VSH, Porosity, Lithology, Saturation,
Permeability, Facies, Rock Typing, and so on. Click **VSH ▾ → VSH from Gamma Ray**.

It opens as a working pane docked beside your log view — not a popup — because you will
tune it while looking at the log. Every module pane in SandiBumi is built the same way,
generated from the module's own manifest, so once you can read this one you can read
all of them.

![The VSH from Gamma Ray pane, with the GR endpoints entered](img/guide-module-pane.png)

Reading the pane top to bottom:

- **RUN ON** — who gets computed. It opens on **All** ("Running on every well in the
  project", here 3 wells). **Selection** is the well you clicked in the Wells pane;
  **Pinned** is the wells you starred; **Custom…** is an explicit list. One run, many
  wells — that is the normal way to work, not a batch afterthought.
- **OPT_GR** — the GR-index transform (LINEAR here; the dropdown holds the nonlinear
  forms). Notice the text under it: every choice and every range check in this pane
  cites where it came from — a spec section, a reference implementation, line numbers.
  That is not decoration; it is the app telling you its defaults are sourced, not
  invented.
- **GR_MA and GR_SH have no default.** The clean-sand and shale gamma-ray endpoints are
  a property of *your* basin, and a number that ships pre-filled would be somebody
  else's calibration silently applied to your field. The link under each field —
  *"Shipped values elsewhere (3) — this number is not settled"* — shows where other
  installed tools carry values, precisely so you can see they disagree. You will supply
  these two numbers yourself, next.
- **GR** input — "Auto — GR_COR → GR_EC → GR" is the preference order: if a corrected
  gamma ray exists on the well it is used, otherwise the raw curve. You can pin a
  specific mnemonic instead.
- **MASK** — optionally name a flag curve (for example a bad-hole flag) and every
  flagged sample becomes missing in the output.
- **INPUT / OUTPUT LOG SET** — the run reads "(current values)" and writes into a named
  output set, **INTERP** by default. Chapter 7 shows what that buys you.
- The green callout is worth memorising: *"Values here are the whole-well defaults —
  per-zone parameters from the Zones pane take precedence inside their zones."* When
  you later give SAND-A its own endpoints, this pane's numbers keep governing everywhere
  else.

### Pick the endpoints from your own data

Where do GR_MA and GR_SH come from? From the histogram — the same place you would pick
them in any interpretation. Open **Plot → Histogram**; it builds for the selected well
and opens on GR.

![The GR histogram of SANDI-01, with P5 / P50 / P95 marked](img/guide-histogram-picks.png)

SANDI-01's gamma ray is cleanly bimodal — sands on the left, shales on the right — and
the stat chips above the plot do the reading for you: **P5 = 43.5, P95 = 111.8 gAPI**,
drawn as dashed lines on the plot. We round to **GR_MA = 44** and **GR_SH = 112** and
type them into the pane. Percentile endpoints (rather than min/max) are a deliberate
choice with a visible consequence you will meet in a moment.

These numbers are picks from *this* dataset, made the way the guide shows so you can
repeat the method on your own field. They are not recommended values for anywhere.

### The run wants to know who and why

Press **Run VSH from Gamma Ray**. The first time, it refuses — in the pane, by name:

> Enter the session operator identity before computing.

Every run in SandiBumi is recorded with who ran it. Type your name or initials into
**Session operator**. Run again and it refuses once more:

> Enter the source/reference covering this run's explicit values.

You typed explicit numbers (44 and 112), so the run demands the reference that covers
them — the same discipline a reviewed study applies to every parameter. Our honest
citation is simply where the picks came from: `GR histogram P5/P95 picks, SANDI-01
(43.5 / 111.8 gAPI, rounded)`. This text is stored with the run and follows the output
curves; when someone asks in two years where 44 came from, the answer is attached to
the curve, not lost in a notebook.

### Read the outcome — "degraded" is the app being honest

Run again. The pane reports into the **Processing** panel and summarises:

> 0 clean · 3 degraded · 0 failed. Open Processing → details for the per-well report.

![The Processing panel's per-well report after the run](img/guide-run-outcome.png)

Expand **details** in the Processing panel. Each well says the same thing:

> degraded result - CLAMPED: calculated value was clamped to the existing range [0, 1]
> (46 occurrences)

This is the consequence of the P5/P95 pick, reported instead of hidden. SANDI-01's GR
actually spans 41.5–115.2 gAPI; every sample cleaner than 44 or shalier than 112
computes a GR index below 0 or above 1, and the limited output clamps it. About 46 of
395 samples — the tails the percentile pick deliberately excluded. "Degraded" never
means the run half-worked; it means the result carries a warning you should read once
and then either accept (as here — clamping at the endpoints is exactly what percentile
picks imply) or fix by re-running with different picks. A well that actually failed
says **failed**, and writes nothing.

### Where the answer went

The run wrote three curves per well into the **INTERP** output set — open the
Inspector's **Curve Catalog** tab to see them listed with the RAW curves:

- **VSH** — the limited volume of shale, 0 to 1. This is the interpretation.
- **VSH_GR** — the *unlimited* GR index (here −0.04 to 1.05). The apparent answer and
  the corrected answer get different names on purpose, so a report can never quote one
  as the other.
- **VSH_PROV** — a per-sample flag saying *why* each sample is what it is (computed,
  input missing, masked, endpoint invalid). Categorical — a reason, never a mask.

The catalog's log-set line states the storage rule in one breath: **every run is kept
as a version — nothing is overwritten**. Re-running with better endpoints writes
INTERP v2; v1 stays. That is why a module run is *re-runnable, not undoable*: undo is
for edits, while a run you disagree with is simply run again, and both versions remain
comparable. Each computed curve also carries an **Ancestry** action — the recorded
account of the module, inputs and parameters that produced it. *(Named gap: the
ancestry view itself is not yet walked through in this guide.)*

### Put VSH on the log

Seeing the number beside the rock is the point. With the log view active, open
**Plot → Properties…**, press **＋** to add a track, name it VSH, set its single curve's
mnemonic to `VSH` with a 0–1 scale, and OK.

![SANDI-01 at 1:500 with the computed VSH beside GR](img/guide-vsh-logview.png)

At 1:500 the interpretation reads at a glance: VSH near zero through both sands, near
one in the shale cap and below TOP_SHALE_2, tracking the gamma ray it came from. The
Curve Catalog beside it shows the three INTERP v1 curves; the Processing panel still
holds the per-well report. If anything looks wrong, the pane you ran from is still
open — change a number and run again. That loop — pane, run, look, adjust — is how
every module in SandiBumi is meant to be worked.

One more habit worth forming now: click any pane and press the **? Help** button (in
the pane header, or Project → Help for the active panel). A module pane's help is the
module's own documentation, the same text its manifest carries.
