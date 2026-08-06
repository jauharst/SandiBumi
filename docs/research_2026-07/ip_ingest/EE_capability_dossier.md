# Experienced Eye (Interactive Petrophysics) — capability dossier

**Purpose.** Competitive/market intelligence on Geoactive's *Experienced Eye* (EE) machine-learning
module in IP, at a depth sufficient to design SandiBumi's counterpart — **without** reconstructing
any proprietary algorithm.

**IP tier.** EE and Domain Transfer Analysis (DTA) are registered **Tier C** (proprietary /
never implement, approximate, or reverse-engineer). Everything below is capability, workflow,
and market positioning, all of it **voluntarily disclosed by the vendor in public webinars**.
No binary was decompiled. See `Tier boundary` at the end for what may and may not cross into
SandiBumi.

**Evidence base.** Five public Geoactive webinars, ingested to timestamped transcripts. Every
load-bearing claim below carries its `[h:mm:ss]` and source tag. Nothing here is written from
model knowledge of ML.

| Tag | Video | Title | Date | Length |
|---|---|---|---|---|
| **V1** | `6g-RAof8wFc` | IP Webinar — Introducing Experienced Eye | 2024-04-30 | 55:48 |
| **V2** | `TKJ7PhBfSf8` | IP — Multi Well Experienced Eye — Webinar 2025 | 2025-10-30 | 41:27 |
| **V3** | `R2C9C5aQ7ng` | Multi-Well Experienced Eye (teaser) | 2025-11-11 | 2:01 |
| **V4** | `QrSbltn2YOY` | What's New in IP & IC 2025 | 2025-04-04 | 59:25 |
| **V5** | `7SxNqtpeo44` | What's New in IP 2026 | 2026-04-16 | 1:00:38 |

Speakers: **Andy McDonald**, product manager and EE co-author (V1 `[0:00:22]`); **Ross Braben**,
co-author, present on the V1 call (V1 `[0:28:06]`, ASR renders him "Ross braen").

Transcripts: `…\scratchpad\ee_yt\<id>.transcript.md`. ASR garbles are marked `(ASR: "…")` where
they touch a number or a name.

**Plus primary evidence (added 2026-08-06): a live IP 2025 session on Jauhar's licensed
install** — 11 screenshots of a real single-well EE run (Inputs / Outputs / Options / Data
Summary / Results) and a read-only inspection of the module's Options panel. Claims tagged
**[LIVE]** below are read off the running product, which outranks webinar narration wherever
the two differ.

**Plus the official IP 2025 manual (added 2026-08-06).** IP 2025's online help is
access-gated, but the install ships its offline-help installer (`IP_2025_Help.exe`); its
payload — `Interact.chm`, the full IP 2025 manual, vendor build 13-Mar-2025 — was extracted
and decompiled locally (6,222 files, 0 errors, same ChmExtract route as the IP2018 ingest).
Claims tagged **[CHM-EE]** are from `experiencedeye.htm`, **[CHM-DTA]** from
`curvepredictionusingdta.htm` (the standalone DTA module page), **[CHM-ML]** from
`machinelearning.htm`. Note the build date: this CHM predates update 3, so it documents
**single-well** EE — the manual's own phrase is "This process works on a single well"
[CHM-EE]. Vendor prose is quoted for verification only (Tier D — never reproduced as UI or
document text in the product). The CHM itself is archived at
`D:\01. Work\00. Guidebook\08. Guidebook IP\IP 2025 Help\Interact.chm`.

---

## 1. Executive judgement

**Experienced Eye is an orchestration harness, not a novel algorithm.**

It automates a search that petrophysicists otherwise do by hand: *which subset of input curves,
fed to which model, predicts the target best?* It answers that by running the **full cross-product**
of (feature-selection method × prediction model × feature-count) and ranking the results in an
interactive leaderboard.

Three of its four feature-selection methods are textbook (Pearson filter, backward/recursive
feature elimination, user-defined order). Only the fourth is proprietary, and it is not a new
idea either — it is a wrapper that reuses DTA's existing maths to score feature importance
(V1 `[0:38:27]`, V5 `[0:33:01]`).

**Consequence for SandiBumi: the valuable, defensible part of EE is the harness, and the harness
is not protected.** A cross-product sweep with a leaderboard is architecture. SandiBumi can build
an equivalent — arguably a better one, since every EE constraint below is a vendor-stated limit —
using entirely open, citable selection and modelling methods.

---

## 2. Version and commercial history

| Release | State |
|---|---|
| **IP 2023** | EE introduced. "Experienced eye was brought in to IP in version 2023" (V2 `[0:14:27]`). Built "from the ground up" as a new module. **Single-well.** Part of the base/foundation licence. |
| **IP 2025 update 3** (Oct 2025) | **Multi-well.** "Jump forward to pretty much the start of this month… we released update 3 for IP 2025 and we've now made the module multi-well aware" (V2 `[0:16:27]`–`[0:16:53]`). Plus interactive visualisations, multi-threading, and an improved EE selection algorithm (V2 `[0:18:26]`). |
| **IP 2026** | Moved out of the foundation licence into the **Machine Learning suite** — "it's basically… a paid for module now because we've put huge upgrades into it and it's fully multi-well" (V5 `[0:30:26]`–`[0:30:50]`). |

### Date conflict — resolved

V5 `[0:30:26]` says EE "was released in 2025"; V2 `[0:14:27]` says "version 2023". These reconcile:
the V5 speaker is describing the **shipping vehicle for the multi-well version** (marketed as part
of the IP 2026 ML suite) and compressing the history, while V2 — the EE-specific deep dive, given
by the module's own co-author — states the original introduction. V2 is the better authority on
provenance, and V1 being titled *"Introducing Experienced Eye"* in April 2024 is consistent with an
IP 2023 ship. **Adopted: introduced IP 2023 (single-well); multi-well in IP 2025 update 3;
repriced as a paid module in IP 2026.**

**Commercially, the repricing is the headline.** EE moved from "free with the base licence" to a
paid module in one release. Any Geoactive customer who used EE casually now faces a purchase
decision — which is the exact moment an alternative gets evaluated.

---

## 3. What it does — full capability inventory

### 3.1 The pipeline

Per V2 `[0:15:17]`–`[0:16:27]`:

1. Take input curves (multi-well grid, restricted to a zone/interval)
2. Run automatic data-quality checks
3. Run each **feature-selection method** → produce ranked curve lists
4. From each ranking, take the **top-3 through top-8** subsets
5. Feed every subset into every **prediction model**
6. Score all combinations on standard metrics; present a leaderboard

### 3.2 Feature-selection methods (4)

Official panel abbreviations **[LIVE]**: EEFS, PC, BFE, User.

| # | Method | Type | Disclosed mechanism | Tier |
|---|---|---|---|---|
| 1 | **Experienced Eye Feature Selection (EEFS)** | Embedded | "based on DTA or domain transfer analysis… an embedded method that is using DTA to work out the feature importance or the most important features" (V1 `[0:38:27]`–`[0:38:39]`). "a derivative of domain transfer analysis" (V2 `[0:25:03]`). "using n-dimensional partial differential equations. It's based on the same maths as DTA" (V5 `[0:32:59]`–`[0:33:02]`). The manual's own description **[CHM-EE]**: the variable space is "transformed into hyperparameter domain and resolved using optimisation procedures to provide influence factors", which are then "processed using Domain Transfer Analysis methodology to produce the ranking solution output", via "unconstrained optimisation and undetermined coefficients for partial differential equations… along with numerical solution based on Trust Region Approach (TRA)" — a description, not a specification (Tier C stands). Reports a per-feature **Influence %** in the results grid **[LIVE]** — a quantitative importance share, not just a rank order. | **C** |
| 2 | **Pearson's Correlation (PC)** | Filter | Ranked on **absolute** value, so −1 and +1 both rank as strong (V2 `[0:25:27]`, `[0:28:55]`); results grid column is literally "PC Absolute Score" **[LIVE]**. | B |
| 3 | **Backward Feature Elimination (BFE)** | Wrapper | "doing a little bit more behind the scenes with say a multilinear regression, and it's looking at the p-value… of each of the features in comparison to the target feature. And then we can remove some of the values that have a higher significance level" (V2 `[0:25:27]`–`[0:25:51]`). Vendor notes it is also called **recursive** feature elimination — "just different terminologies" (V2 `[0:25:18]`). **Significance Level default = 0.05** — "p-values above this value will be eliminated from the ranking"; eliminated features shown greyed-out in the results grid **[LIVE]**. Manual confirms both: "uses multiple linear regression to determine significance. The default p-value threshold is set to 0.05", and eliminated results stay in the grid greyed with metrics still computed **[CHM-EE]**. | B |
| 4 | **User Defined Order (User)** | Human baseline | You rank the inputs in the grid; EE scores your ordering against the automated ones — "see how well your domain knowledge compares against the machine" (V2 `[0:15:41]`, `[0:25:51]`–`[0:26:14]`). | A |

Panel note **[LIVE]**: *"Only the training data points are used for Feature Selection"* — the
test set is untouched by selection, i.e. no selection leakage across the split.

> Method 4 is the quiet design win and costs nothing to copy: it makes the petrophysicist's own
> prior a *competitor in the leaderboard*, which both builds trust and occasionally proves the
> machine right. SandiBumi should have it on day one.

### 3.3 Prediction models (3)

Domain Transfer Analysis (**Tier C**, proprietary), multilinear regression, neural network
(V2 `[0:26:36]`).

Roadmap stated 2024: "we are looking to add more models into this, including a potential decision
forest or random forest type model as well as a few of our other modules within IP" (V1
`[0:39:53]`–`[0:40:03]`). **As of the 2026 webinar this had not happened inside EE** — random-forest-class
algorithms arrived instead in a *separate* module (§6).

For context, IP 2025's full Machine Learning menu **[CHM-ML]**: Experienced Eye, Fuzzy Logic
Curve Prediction (Cuddy 1997, SPWLA — a published, citable method), Multiple Linear Regression,
Neural Networks, Cluster Analysis for Rock Typing, Self-Organising Maps, Principal Component
Analysis, Contingency Table, Textural Facies Analysis, and Curve Prediction using DTA. EE
orchestrates only 3 of these 10 — the classification-capable ones (Cluster, SOM, Fuzzy Logic)
sit outside its sweep, consistent with §5.3.

### 3.4 Architectural disclosure: DTA has no hyperparameters

The single most useful technical disclosure in the whole corpus, given as an aside:

> "MLR and DTA — because DTA's partial differential equations, it's a one-time run. **There's no
> parameterization to play with**, but most machine learning algorithms there is. You can fiddle
> around with nodes and layers…" — V5 `[0:40:48]`–`[0:41:00]`

DTA is a **one-shot deterministic solve**, not an iteratively fitted model. That explains its
sample ceiling (§5.1), why EE optimises *features* rather than *parameters*, and why Geoactive
needed a whole separate module to do hyperparameter search. It also means DTA has no
training-time stochasticity — a property SandiBumi's reproducibility posture would value, but
which any deterministic solver (not just DTA) provides.

**DTA does expose one dial [LIVE]** — the EE Options panel carries exactly one DTA parameter:

> `Maximum levels   100   (max = 475, default = 100)`

The manual now defines the term outright **[CHM-EE]**: *"Maximum Levels: The number of **depth
levels** to use in the modeling process. This is limited to a maximum of 475 depth levels, and
defaults to 100."* So a **level is a depth level — one data row** (the earlier open question is
closed), the hard ceiling is **475**, and inside EE the shipped default is **100**. The
webinar's "about 470 data points" (V1 `[0:52:08]`) was this cap, loosely recalled but right in
kind. The manual also concedes the reason on record: *"computation speed increasing
exponentially rather than linearly"* **[CHM-EE]**.

**And the standalone DTA module is configured differently [CHM-DTA]** — same 475 ceiling, but
**default 200**, and a different over-limit strategy: *"the program automatically reduce them to
the maximum by sorting the data and selecting data to represent the full variability of the
data"*, with the cost quantified — *"Doubling the number of data levels in the model will
increase the model build time by more than 5 fold."* Inside EE, by contrast, over-limit *"depth
levels will be selected **randomly** from the available data"* **[CHM-EE]**. So **EE runs DTA
at half the standalone default with the worse (random) subsampler** — the V1 claim of
distribution-aware sampling (§5.1) describes the standalone module, not EE. The standalone page
also states the input ceiling: *"You can use up to eight input curves per well"* **[CHM-DTA]**
— which is why EE's feature sweep stops at top-8.

### 3.5 Automatic data QC (the drop rules)

EE drops, before modelling (V1 `[0:37:55]`–`[0:38:12]`; V2 `[0:23:30]`–`[0:24:43]`, `[0:26:59]`–`[0:27:21]`):

| Rule | Detail |
|---|---|
| **Constant curves** | "straight" / fixed-value curves. Demo dropped **BS (bit size)** — "obviously it's a fixed value curve" (V2 `[0:27:45]`). |
| **Monotonic curves** | "whether we've got monotonically increasing curves" — catches depth and index curves leaking in as features. |
| **Null rows** | Any row with a missing value in **any** selected curve is excluded entirely — "that entire row is not used within the modeling process" (V1 `[0:40:39]`). Listwise deletion. |
| **Near-duplicate curves** | Excluded via a **Pearson correlation cutoff** — shipped default **0.99** (panel: "min = 0, max = 1, default = 0.99"; "Curves with a score greater than this value are removed from the model") **[LIVE]**. |

The manual states the exact processing order **[CHM-EE]**: null reference-curve values → null
input values (whole-null curve excluded; any null at a depth removes the **entire row** —
listwise deletion, now in vendor prose, not just webinar narration) → constant curves →
monotonic curves → duplicate curves.

**Live confirmation of the funnel [LIVE]** — a real single-well run (6 inputs incl. TVDSS,
target RHOB) printed in its Data Summary:

- `Curves excluded for being monotonic: TVDSS` — the depth-curve leak caught exactly as
  described.
- **93,401 rows in data range → 120 qualifying rows** (0.13 %). Listwise deletion over curves
  with ~8.8–8.9 % nulls each, intersected with the target's limited coverage, collapsed the
  dataset by three orders of magnitude. The final model fitted on **87 training / 33 test
  points** (72.5 / 27.5 % actual against the 70/30 nominal).
- Per-curve null percentages, per-curve validation mode (`- Linear`), and standard deviations
  of every valid curve are printed in the same summary — the traceability panel is real, not
  demo-ware.

> The 93,401 → 120 collapse is the strongest single argument in this dossier. Listwise deletion
> plus a sparse target turns a rich log dataset into a hand-count of points, and the UI reports
> it only in a text panel a user may never read. SandiBumi should make survivorship loud —
> report the funnel (rows in range → post-null → post-QC → train/test) as a first-class output
> and warn when the surviving fraction is pathological.

The near-duplicate rule exists for a specifically petrophysical failure: *"we've got a porosity in
decimal and a porosity in percentage… a straight comparison would flag up as two different curves
because they're different ranges. But if we plotted them on the same scale… they're exactly the
same"* (V2 `[0:23:54]`–`[0:24:19]`). **This is a real trap and SandiBumi should implement the same
guard.** It is a data-hygiene convention, not IP.

### 3.6 Train/test splitting

- **Automatic** — random point selection across the whole data set. Default **70/30**
  (V2 `[0:22:44]`, `[0:24:43]`).
- **By well** *(new in the multi-well version)* — nominate specific wells for training vs testing,
  e.g. "train on the outer edges… predict on wells nearby or slightly in towards the centre of the
  field", or train on key wells and test on the rest (V2 `[0:21:34]`–`[0:22:44]`).
- Vendor guidance: smaller data sets want **80/20 or 90/10** rather than 70/30 (V2 `[0:22:44]`).

> **Split-by-well is the correct default for geological data**, not random point splitting — random
> splitting leaks information between adjacent depth samples of the same well and flatters the test
> metric badly. Geoactive shipped it only in late 2025; SandiBumi should treat it as the *default*,
> not the option.

### 3.7 Outputs

**Only two curves** (V2 `[0:23:08]`–`[0:23:30]`), default mnemonics **[LIVE]**:
1. `EE:RowQualifiesFlag` — whether a data row **qualifies** (survived QC)
2. `EE:TrainTestFlag` — whether the row was assigned to **training or testing**

*(The manual lists **three** output curves — Input Row Qualifies Flag, Training Model Row Flag,
Test Model Row Flag **[CHM-EE]** — where the live 25.3.3 run produced the two mnemonics above.
Either the train/test pair was merged into one flag after the Mar-2025 CHM build, or
`EE:TrainTestFlag` encodes both states; recorded as a version-drift discrepancy, not resolved.)*

Both are viewable on a log plot. The chosen model is handed off by **right-clicking it in the
leaderboard**, which auto-populates IP's separate *Curve Prediction* module, where you untick
"model build" and apply it to new wells (V2 `[0:39:08]`–`[0:39:31]`).

> Note what is *not* output: no serialised model artifact, no feature-importance curve, no per-depth
> prediction interval or uncertainty. EE is a **model-selection** tool that then hands you off to a
> different module to actually deploy. That seam is a product weakness (§5.4).

### 3.8 Metrics

R², **MAE**, **RMSE** — computed on the fly from actual vs predicted, switchable between training
and testing sets, aggregated across wells with per-well breakdown (V2 `[0:30:05]`–`[0:30:49]`).

The vendor explicitly warns against R² alone: *"it's not just R squared as sometimes that can be
misleading"* (V2 `[0:30:05]`), and *"if we just focused on the metrics like R squared, we may end up
being pulled the wrong way in terms of what is the best performing model"* (V2 `[0:17:18]`).
RMSE is presented as the outlier-sensitive one (V2 `[0:30:27]`).

### 3.9 Visualisation and QC surface

This is where the multi-well version spent its effort, and it is the most copyable part of the
product (all V2, `[0:27:45]`–`[0:38:22]`):

| Panel | What it shows | Why it matters |
|---|---|---|
| **Data Summary tab** | training ratio, curves dropped **and why**, curves selected | Explicitly built for audit: *"handy to copy and paste into a report so that you've got full transparency… and it keeps that traceability"* `[0:28:08]` |
| Feature-selection results | ranked curve list per method, side by side | shows methods disagreeing on rank order |
| Prediction-model performance grid | all metrics, all wells, toggleable per well | |
| **Metrics range per well** (box plot) | each × = one well's one metric | isolates a problem well from a problem model |
| Prediction-model comparison | one model × all selection methods × top-3…top-N | "is there any degradation as we move along?" `[0:32:26]` |
| Feature-selection comparison | one selection method × all models × top-3…top-N | |
| **Train vs test comparison** | training result against testing result | **the overfit diagnostic** — see below |
| Single-well scatter + histogram | predicted vs actual; active well red, others grey | |
| Embedded log plot (Single Well Plots) | single-track depth strip of actual vs predicted target curve; **vertically rescalable, scrollable in depth** — a deliberately simple layout, not a full log-plot composer **[LIVE]** | quick per-depth eyeball of where the prediction fails |
| Interactive log plot | zoom, pan, **detachable**, linked *or* locked windows | side-by-side well comparison on multi-monitor |
| Multi-well log plot | all wells together; **visual limit ~15–20 wells** `[0:37:11]` | |
| Multi-well crossplot | all wells, active highlighted | |
| Best-model jump links | "best model", "2nd best", "top 1–3", **per metric** | `[0:38:22]`; metrics can disagree on the top 3 `[0:38:45]` |

**The train-vs-test chart encodes a diagnostic worth copying verbatim in meaning** (V2 `[0:34:02]`–`[0:34:27]`):
- training ≫ testing → the model is "memorizing some patterns… within the training data" (overfit)
- testing > training → "could be an indication that there might be some problems there in the data
  or that the model is needing tuned"

### 3.10 Scale and speed

- Demo ran **60 model combinations in about a minute**, multi-threaded (V2 `[0:27:21]`, `[0:40:42]`).
- "tens of thousands of data rows within a few minutes or less" (V2 `[0:18:03]`).
- Honest caveat given: "if you added 100 wells then yes the time will get slower" (V2 `[0:18:03]`).
- Curve aliasing auto-populates the grid; wells missing a required curve are **flagged in the grid**
  and must be removed or filled (V2 `[0:20:01]`–`[0:20:49]`).

---

## 4. The combinatorics — proof it is an exhaustive sweep

The vendor gives two run-counts. They are not in conflict; both are exact cross-products, and that
is the tell.

**Stated manual-effort case** (V2 `[0:12:56]`–`[0:14:07]`):
> 3 feature-selection methods × 3 models = **9 runs**; adding the top-3…top-8 groups → *"that can
> rack up to about **54** separate runs"*, which by hand takes *"several hours to days"*.

`3 × 3 × 6 = 54` ✔ (six feature-counts: top-3,4,5,6,7,8)

**Actual demo** (V2 `[0:27:21]`): *"we're processing **60** different combinations of models"* — with
**4** selection methods enabled, and only **7** usable curves after BS was dropped from 8, so the
sweep ran top-3…top-7.

`4 × 3 × 5 = 60` ✔

Both reconcile exactly. **Experienced Eye enumerates the full grid — there is no guided search,
no early stopping, no Bayesian optimisation.** V5 confirms it plainly: *"It will try all the
combinations"* (`[0:33:50]`).

> This is the most strategically useful fact in the dossier. Brute force over a small grid is
> cheap to replicate and easy to beat: the entire "proprietary" advantage reduces to one of four
> ranking functions, and the harness around it is a nested loop.

**Third confirmation [LIVE].** A real single-well run (5 input curves + target RHOB; TVDSS
dropped as monotonic → 4 usable features) produced a leaderboard of exactly
**12 columns × 2 rows = 24 cells**: 4 selection methods × 3 models × 2 feature-counts (top-3,
top-4 — the only counts possible with 4 features). Three independent run-counts, three exact
cross-products.

### 4.1 What the live leaderboard shows that no webinar said

Observations from the same run, worth more than any marketing slide:

1. **The proprietary selector and the textbook one flatly contradicted each other.** EEFS ranked
   CALI #1 with **91.2 % influence**; BFE **eliminated** CALI outright (p = 0.4657 > 0.05,
   greyed out) and ranked DRHO #1. Same 87 training points, opposite verdicts on the top
   feature.
2. **The textbook method won.** Best model of the whole sweep: **DTA with BFE, top-3, R² 0.9473**
   — beating DTA with EEFS (0.9211) on the vendor's own leaderboard. (#2 NN-EEFS 0.9460,
   #3 NN-PC 0.9438.)
3. **At top-4 the selection methods collapse into one.** With only 4 usable features, "top 4" is
   the full set regardless of ranking, and the grid shows it: DTA scores 0.9336 across all four
   selection methods, MLR 0.6314 across all four. The cross-product spends 8 of its 24 cells
   computing the same four numbers — redundancy a smarter harness would skip.
4. **NN is not deterministic.** On identical top-4 feature sets, NN returned four different
   scores (0.9215 / 0.9332 / 0.9206 / 0.8630) — training stochasticity or input-order
   sensitivity, either way a reproducibility caveat DTA and MLR don't have.
5. **Small-n caution the UI does not surface.** The test set is 33 points; the #1-vs-#2 gap is
   0.0013 of R². Ranking at that resolution is noise, and nothing in the interface says so.
6. Right-panel controls not narrated in webinars: **Export top N** link, **Max number of
   inputs** selector (up to 8), Train/Test subset radio, R²/MAE/RMSE metric radio,
   performance-grid colour palette with adjustable range.

*(Run details generic by intent — client well names stay out of this document.)*

---

## 5. Stated limits — every one of these is on the vendor's own record

### 5.1 DTA's sample ceiling: "Maximum levels" 100 default / 475 max

> "with DTA we are limited to… I think it was about **470 data points**. As we start to go above
> that then the processing time for DTA can then become **exponentially large**, so we limit it to
> 470 data points at the very most." — V1 `[0:52:00]`–`[0:52:24]`

**Corrected by the product panel [LIVE] and defined by the manual [CHM]:** the parameter is
`Maximum levels`, one level = one **depth level** (data row) [CHM-EE], ceiling **475**. The
defaults diverge by context: **100 inside EE** [LIVE][CHM-EE], **200 in the standalone DTA
module** [CHM-DTA] — either way a casual user runs DTA at a fraction of its already-small
ceiling without knowing it.

The subsampling when data exceeds the cap also diverges (§3.4): the standalone module sorts and
selects "to represent the full variability of the data" [CHM-DTA] — the behaviour V1
`[0:52:27]`–`[0:52:40]` described as "taking samples from the highest values as well as the
lowest values" — while **EE-embedded DTA subsamples randomly** [CHM-EE], the worse strategy in
the very context (automated sweeps, no user eyes on the sample) where it matters most.

The cost of the cap is quantified on the vendor's own record: computation is *"exponential
rather than linear"* [CHM-EE]; *"Doubling the number of data levels in the model will increase
the model build time by more than 5 fold"* [CHM-DTA]. Asked directly in 2024 whether the
ceiling would improve: *"that's something that we might look at in the future… because it is
quite comp[utationally expensive]"* (V1 `[0:52:51]`).

**This is the single largest technical opening.** A modern gradient-boosted tree trains on 10⁶
rows in seconds. IP's flagship proprietary model is capped at 475 depth levels, ships at
100–200, and more-than-quintuples its build time per doubling.

### 5.2 Random sampling only — stratified sampling still not shipped

> "we just take a random sampling through that data, but we **are looking at adding** more methods
> in such as **stratified sampling**, which is probably a bit more appropriate for **geological
> data**." — V1 `[0:39:25]`–`[0:39:41]`

Stated as a gap in 2024. The 2025 multi-well release added *split-by-well* (§3.6) but **no
stratified sampling is claimed** in V2 or V5. The vendor has conceded on record that random
sampling is the wrong choice for geological data and has not fixed it in two years.

### 5.3 Regression only — no classification, no facies

> "this whole experience module is **currently focused on regression problems**. So we're trying to
> predict curves rather than **classes and rock types**, but that is something that we are looking
> to bring in in the next phase." — V2 `[0:16:05]`–`[0:16:27]`

EE cannot do facies, lithology, or rock-typing — arguably the highest-value ML task in
petrophysics. Still true as of the 2026 webinar.

### 5.4 Thin outputs and a hand-off seam

Two flag curves only (§3.7). No exported model, no importance curve, no uncertainty. Deployment
requires leaving EE for the Curve Prediction module.

### 5.5 No hyperparameter optimisation

EE optimises *which curves*, never *how the model is configured*. Geoactive's answer to this is a
**separate, non-composing module** (§6) — you cannot search features and hyperparameters together.

### 5.6 Display ceiling

Multi-well log plot caps at ~15–20 wells for legibility (V2 `[0:37:11]`).

---

## 6. New in IP 2026 — the ML.NET sibling module

A second, distinct module ships in IP 2026 that **inverts** EE (all V5 `[0:38:41]`–`[0:42:26]`):

> "another new tool… similar in many ways to experienced eye but it's working slightly differently
> and **under the hood it's using ML.NET**." `[0:38:57]`

| | Experienced Eye | IP 2026 ML.NET module |
|---|---|---|
| Optimises | **features** (which curves) | **hyperparameters** (how each algorithm is configured) |
| Features are | searched | **fixed — you pick them** |
| Algorithms | 3 (DTA, MLR, NN) | wide ML.NET portfolio |
| Task types | regression only | curve prediction **and** cluster-analysis algorithms |

> "with experienced eye… it was really about ranking the input curves, the feature selection, and
> then comparing the individual models. In this ML.NET instance, **you pick the features**… it's not
> going to mess around with 'oh, what if I use these curves' — you pick the curves, you pick the
> wells… what it's really doing is it will train and train and train an optimized model… trying to
> find the best parameters for each algorithm. **The features are fixed.**" `[0:40:15]`–`[0:41:09]`

*(At `[0:41:11]` the speaker says "whereas in Experienced Eye we're trying to optimize the
parameters" — a plain misspeak for* features*, contradicted by everything either side of it and by
V2 in full. Recorded here so the transcript isn't misread later; not treated as fact.)*

Two further disclosures:

- **Why ML.NET at all** — "rather than coding up our own random forest, we might be doing that, but
  for now you can access it through ML.NET"; it brings algorithms "currently not in IP" `[0:39:13]`–`[0:39:40]`.
- **What stays in-house** — "Every one of the algorithms that's in IP, like neural network, it's
  dedicated code, or DTA, that's our own proprietary code. You're not going to find that on the web.
  You're not going to find that digging around in some Python library somewhere." `[0:39:13]`
- **"Fast forest"** — "effectively that algorithm is the same as random forest. It's just random
  forest is trademarked, I think, by the original author, and fast forest is basically the same
  thing by another name." `[0:41:54]`–`[0:42:05]`
  *(The naming is a Microsoft ML.NET convention; the trademark rationale is the speaker's belief,
  offered with "I think". Not verified here, and it does not need to be — it's a naming aside.)*

> **Strategic read.** Geoactive is now leaning on an open Microsoft library (ML.NET, MIT-licensed)
> for everything except DTA and their neural net. Their moat is narrowing to two components by
> their own account, and one of those two (DTA) is capped at 475 levels. A tool that searches
> features *and* hyperparameters *together* — which neither IP module does — would exceed both.

---

## 7. The published papers — the legitimate Tier-B route, now fully identified

The manual's own References section **[CHM-EE]** names the papers the webinars alluded to —
and corrects a name: the co-author ASR rendered as "Ross braen" (read earlier as "Braben") is
**Ross Brackenridge**. The EE lineage, with DOIs:

| Year | Citation | What it is |
|---|---|---|
| 2019 | Arkalgud, R., McDonald, A. & Crombie, D. — *Domain Transfer Analysis — A Robust New Method for Petrophysical Analysis*. SPWLA 60th Ann. Logging Symp. **doi:10.30632/T60ALS-2019_HHHH** | DTA itself (Tier C — cited for provenance only) |
| 2020 | Arkalgud, R., McDonald, A. & Brackenridge, R. — *Automated Selection of Inputs for Log Prediction Models Using Domain Transfer Analysis DTA Derivative*. ADIPEC, Abu Dhabi. **doi:10.2118/203094-MS** | the DTA-derivative feature-selection method — EEFS's published precursor |
| 2021 | Arkalgud, R., McDonald, A. & Brackenridge, R. — *Automated Selection of Inputs for Log Prediction Models Using a New Feature Selection Method*. SPWLA 62nd Ann. Logging Symp. **doi:10.30632/SPWLA-2021-0091** | **the Experienced Eye paper** (cited twice in the manual's reference list) |
| 2021 | McDonald, A. — *Data Quality Considerations for Petrophysical Machine-Learning Models*. Petrophysics 62, pp. 585–613. **doi:10.30632/PJV62N6-2021a1** | the QC-rules rationale (the §3.5 drop rules trace here) |

The webinar recollection that started the hunt:

> "my colleague Ross *(ASR: "Ross braen" — per the manual's references, **Brackenridge**)*, who's
> on the call, and myself — we were working on the paper for Experienced Eye a few years ago, and
> it probably took a week or two just to go through all of the models that we had to get our
> results for the paper. So it was very manual, we're doing it step by step." — V1
> `[0:28:01]`–`[0:28:26]`

V2 `[0:09:52]` reprises its worked example, and the **experiment design is fully described and
reproducible**:

- 14 logging curves; target = **porosity**
- Selection by **Pearson |r| threshold**, swept from 0.1 to 0.7 in 0.1 increments
  *(ASR renders the top of the range as "7"; 0.7 is the only reading consistent with a Pearson
  cutoff — flagged rather than silently corrected)*
- Model re-run at each threshold; actual-vs-predicted scatter and metrics recorded each time
- Result: converged to **3 features — RHOB, DTC, and total/average gas** (V2 `[0:11:47]`)
- Interpretation offered: gas was retained because of a probable **light-hydrocarbon effect on
  porosity** that the gas curve was proxying (V2 `[0:12:11]`)
- Behaviour observed: performance *worsened* mid-sweep (after dropping C1/C4 ratio and DRHO) before
  improving to the 3-feature optimum (V2 `[0:11:23]`) — i.e. the curve is **not monotonic**, which
  is exactly why an exhaustive sweep beats a greedy one

Whether this worked example sits in the 2020 ADIPEC paper or the 2021 SPWLA paper is not
determinable from the webinars — both are retrieval targets.

**These are Tier-B artifacts: published, citable, and reimplementable from the primary paper**
(except the 2019 DTA paper, which stays Tier C). Obtaining the 2021 SPWLA paper is the
highest-value next step (§9) — and with the DOI in hand it is now a lookup, not a hunt.

---

## 8. What SandiBumi should build — and what it must not

### 8.1 Adopt (Tier A — conventions and architecture, not IP)

1. **The harness itself** — cross-product sweep over (selection method × model × feature-count),
   with a ranked leaderboard. Nested loops; no IP attaches.
2. **The four-slot selection design, including the human baseline.** Ship user-defined ordering as
   a first-class competitor in the leaderboard (§3.2 #4).
3. **The QC drop rules** — constant, monotonic, null-row (listwise), and near-duplicate by
   |r| ≥ 0.99. The φ-decimal-vs-φ-percent trap is real and cheap to guard (§3.5).
4. **The Data Summary traceability panel** — what was dropped, why, what was used, copy-pasteable
   into a report. This aligns exactly with SandiBumi's existing audit posture and is arguably the
   feature most worth matching.
5. **Split-by-well as the default**, with random splitting demoted to an option (§3.6).
6. **The train-vs-test diagnostic chart** and its two-sided reading (§3.9).
7. **Metric plurality** — never rank on R² alone; show R², MAE, RMSE, and allow the "top 3" to
   differ by metric.

### 8.2 Reimplement from open sources (Tier B — cite the primary reference)

- **Pearson |r| filter ranking** — trivial; cite Arkalgud, McDonald & Brackenridge 2021
  (SPWLA-2021-0091) once obtained (§7).
- **Backward / recursive feature elimination** via p-value against the target, standard OLS
  machinery (§3.2 #3).
- **Prediction models** — any open regressor. Gradient-boosted trees, random forest, linear
  models, MLP.

### 8.3 Never implement (Tier C)

- **Experienced Eye feature selection** and **Domain Transfer Analysis**. Do not implement,
  approximate, benchmark-fit against, or reverse-engineer. Do not attempt an "n-dimensional
  partial differential equation" importance scorer on the strength of V5 `[0:32:59]` — that
  sentence is a description, not a specification, and treating it as one is exactly the
  contamination the Tier-C rule exists to prevent.

**The open substitute for EE feature selection**, with none of its limits:
permutation importance, SHAP, mutual information, or model-embedded importance from a GBM. All
open, all citable, all uncapped in sample count, and all applicable to classification as well as
regression — so they clear §5.1 and §5.3 simultaneously.

### 8.4 Where SandiBumi can exceed IP

Each of these targets a limit the vendor stated publicly:

| Opening | IP's position | SandiBumi |
|---|---|---|
| Sample scale | DTA capped at 475 levels, ships at 100, exponential cost (§5.1) | GBM on full data set |
| Survivorship visibility | 93,401 → 120 row collapse reported only in a text panel (§3.5) | funnel as a first-class output, with a pathological-fraction warning |
| Sampling | random only; stratified conceded-but-unshipped since 2024 (§5.2) | stratified + split-by-well as defaults |
| Task type | regression only; facies "next phase" (§5.3) | classification from the start |
| Search space | features **or** hyperparameters, in two non-composing modules (§5.5, §6) | joint search in one module |
| Outputs | 2 flag curves; no model artifact, no uncertainty (§5.4) | serialised model, importance curves, prediction intervals |
| Deployment | right-click hand-off to a separate module (§3.7) | apply in place |
| Cost | now a paid add-on module (§2) | included |

---

## 9. Coverage assessment — and what to provide next

**Coverage achieved: ~90% of the capability picture; 0% of the proprietary algorithm, by design.**

Fully covered: workflow, all four selection methods and their disclosed mechanisms, all three
models, the QC rules (now live-verified, with a real survivorship funnel), splitting, metrics,
output mnemonics, every visualisation panel, run combinatorics (three exact cross-products),
the panel defaults (Pearson cutoff 0.99, BFE significance 0.05, DTA Maximum levels 100/475),
speed, scale limits, version history, licensing change, and the 2026 sibling module.

**Item 2 of the original ask-list (live product access) is now closed** — Jauhar ran EE on his
licensed IP 2025 and granted a read-only inspection session (2026-08-06). Item 3 is
half-answered by the same fact: his IP 2025 licence runs EE and DTA today, so a like-for-like
benchmark is possible; the IP 2026 repricing question remains open only for future versions.

Not covered, and not recoverable from these sources:

| Gap | Status |
|---|---|
| EE feature-selection internals | **Deliberately out of scope** — Tier C. Not a gap to close. |
| DTA internals | Same. ~~What one "level" is~~ **Resolved [CHM-EE]:** one level = one depth level (data row). The solver's internals remain out of scope. |
| Exact UI layout / panel geometry | Seen live; Tier D (expression) — must not be reproduced anyway. |
| The published paper's actual numbers | **Closable** — exact citations + DOIs now in hand (§7); the PDF itself is the remaining ask. |
| Whether IP 2026 changed EE's algorithm again | V5 says "massive upgrades" but details only the multi-well/visualisation work. |
| EE help-page prose | **CLOSED 2026-08-06.** The Help button opens Geoactive's online, access-gated docs ("Access Denied" on this machine); the offline installer `IP_2025_Help.exe` refuses to run (broken install-detection — the vendor registry keys it checks are empty on this machine). With approval, its payload was extracted directly (innoextract 1.10-dev; single file `Interact.chm`, 234 MB, build 13-Mar-2025) and decompiled (6,222 files, 0 errors). `experiencedeye.htm`, `curvepredictionusingdta.htm`, and `machinelearning.htm` are harvested into this dossier as **[CHM-*]**. The CHM is archived in the Guidebook tree; the **full** IP 2025 manual ingest (343 topic pages) is available as a follow-on but not yet commissioned. |

### Remaining asks

1. **Arkalgud, McDonald & Brackenridge 2021, SPWLA-2021-0091** *(highest value; now a DOI
   lookup, not a hunt — §7)*. This is the **Tier-B legitimate route** — a published method may
   be reimplemented and cited. It would also give the real numbers behind the 14-curve porosity
   experiment (§7), a ready-made validation case: reproduce their published result with open
   methods, and the harness is proven. The 2020 ADIPEC precursor (SPE-203094-MS) and McDonald's
   2021 data-quality paper (PJV62N6-2021a1) are secondary targets in the same order.

2. **Decision: commission the full IP 2025 manual ingest?** The decompiled CHM (343 topic
   pages) is the IP2025 counterpart of the IP2018 ingest — it would close the remaining
   module-parameter gaps (and the IP2018 ingest's two OPEN discrepancies, if the 2025 prose
   differs). Not started without explicit go-ahead.

---

## Tier boundary — standing rule for this document

- **Tier A** (§8.1) — conventions, workflow shape, QC hygiene, audit surfaces. **Adoptable.**
- **Tier B** (§8.2, §7) — published, citable science. **Reimplementable from the primary source,
  with citation.** Not from this dossier's paraphrase.
- **Tier C** (§8.3) — EE feature selection, DTA. **Never implemented, approximated, benchmark-fitted,
  or reverse-engineered.** No binary was decompiled to produce this document, and none should be.
- **Tier D** — UI layout, panel design, vendor prose and assets. **Never reproduced.**

Every quotation in this dossier is from a **public vendor webinar** (timestamped transcript) or
the **vendor's own manual** ([CHM-*] tags — quoted for verification, never reproduced as
product text). No vendor file, chart table, tool definition, or test data crosses into the
SandiBumi repo as a result of this work; the extracted CHM lives in the Guidebook raw-source
library, outside the repo.

---

*Compiled 2026-08-06 from `…\scratchpad\ee_yt\*.transcript.md` (V1–V5), then upgraded the same
day with primary evidence: 11 screenshots of a real EE run plus a read-only live-app inspection
of IP 2025 ([LIVE] tags), then with the official IP 2025 manual pages extracted from
`IP_2025_Help.exe` ([CHM-*] tags). Sibling to `ip2018_chm_ingest/` and `techlog_ingest/`.*
