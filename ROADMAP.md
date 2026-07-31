# SandiBumi — Development Roadmap

Grounded in the the reference suite V14 helpset catalog (`C:\Program Files\AspenTech\the reference install\doc\helpset`,
~120 module help books) and the original Techlog-style UX redesign plan. **Restructured
2026-07-20 by status** — what's Done, what's Open, what's Future — so it's readable at a
glance. (The earlier standalone UI/UX redesign plan `jolly-skipping-dove.md` is folded in and
fully superseded by this document.)

## How to read this

> **Sequencing across both products** — which of these items to do *first*, and why, lives in
> [`docs/FUTURE_PLAN.md`](docs/FUTURE_PLAN.md) (2026-07-31): the competitive scan vs
> Geolog/Techlog/IP, the three positioning axes, the credibility floor, and the OSDU question.
> That document sits above this one — it never overrides an item here, it orders them.

**Three status buckets** — every item lives in exactly one:

| | Bucket | Meaning |
|---|---|---|
| ✅ | **[Part A · Done](#-part-a--done)** | shipped, tested, in the app (some carry small deferred slivers, flagged) |
| ◻ | **[Part B · Open](#-part-b--open-do-next)** | the actionable backlog — do next |
| 🔮 | **[Part C · Future](#-part-c--future)** | bigger lifts, planned but not scheduled |

**Three label families** (unchanged — these name *kinds of work*, not status; the bare tags
`P0`–`P3` are **retired**):

| Family | Used for | IDs |
|---|---|---|
| **Phase** | the chronological build arc (product milestones) | Phase 1 … Phase 12 — sub-steps like `6a`/`8b`/`9-3` are detail *inside* a phase |
| **Severity** | engineering-hardening backlog (audit/review debt), §4b | **Critical → Reliability → Performance → Polish → Low** |
| **Wave** | new-capability feature suites (Jauhar's requests), §4c | Wave A … Wave E |

**Old → new crosswalk** (so older notes, tasks, memory and REVIEW.md still map):
- Hardening backlog (§4b): `P0`→**Critical** ✅ · `P1`→**Reliability** ✅ · `P2`→**Performance** ◻ · `P3`→**Polish** ◻ · Low ◻.
- Field-review tiers (§4, a *different* 2026-07-19 list that also once used P1/P2/P3): `P1`→**Trust & safety** ✅ · `P2`→**Interpretation workflow** ✅ · `P3`→**New capability** (open items now live in the Wave backlog).
- Inline provenance tags inside §4 like `(…, P1-b)` / `P2-e` are historical increment IDs from that dated review — read them through the crosswalk above.
- Checkbox marks: `[x]` done · `[ ]` open · `[~]` partial.

The old section numbers are **kept as identifiers** (§0–§5, §4b, §4c) because memory, REVIEW.md
and the `docs/research_2026-07/` specs reference them by number — they now appear as tags on the
headers below rather than as the primary structure.

---

## At a glance

### ✅ Done — the bulk of the app
- **Shell & workspace** (Phases 1–4): dockview MDI, 5-tab ribbon, light/dark/system themes, DB Inspector + undo/redo, read-only SQL panel, selection-driven workspace, i18n (EN / Bahasa Indonesia / Basa Sunda).
- **Plots** (Phase 3 + v2s): WebGPU log view, Histogram v2, Crossplot v2, Pickett, multi-well Correlation, Field Dashboard — all HiDPI, zoom/pan, properties dialogs, export/templates.
- **Data foundation** (Phase 6): generic curve store + units/TVD; LAS / DLIS / deviation / core / SCAL / tops / aux / well-location import.
- **Interpretation physics** (Phases 7–8.5): deterministic module library; **SandiMin** 27-component the reference suite-parity mineral solver; Sw suite (RtC / IMTS / Indonesia / dual-water); SSC / SSPW porosity; saturation-height; environmental corrections; Thomas-Stieber thin beds; Jauhar's field method suite.
- **Deliverables** (Phase 8): composite plots at true print scale, multi-page PDF reports, batch export.
- **Field scale** (Phase 9): workflow chains, Monte Carlo uncertainty, Field Dashboard, write-path perf hardening (100-well chain 50s→21s).
- **Facies & ML** (Phase 10): electrofacies (k-means + GMM), full scikit-learn ML suite.
- **Hardening**: **Critical** (8 correctness/data-integrity fixes) + **Reliability** (frontend-state races/leaks) tiers — done & adversarially verified.
- **Feature waves**: **Wave A** (all tools as panes, compact ribbon, project picker, workflow grid) + **Wave E** (KKT ONWJ: precalc, dry-clay, gascorr, φmax, cutoff-sensitivity, map/polygons, condflag, nphimat) — all shipped.
- **Polish so far**: units on readouts + adaptive value formatting (Polish-1); correlation well-list refresh + Ctrl-wheel zoom (Polish-2).

### ◻ Open — do next  → [Part B](#-part-b--open-do-next)
- **Polish tail** (§4b): ✅ all shipped — units #122, correlation #123, history-coverage #124, Pickett v2 #125, pay-summary provenance #126.
- **Performance** (§4b): crossplot redraw memoize (#127) ✅, **batch curve reads (#130)** ✅ **persistent Python worker (#132)** ✅ and **raw-IPC ArrayBuffers (#131)** ✅ **shipped + committed 2026-07-21**; **async commands (#128)** ✅ **shipped 2026-07-30** (project open/switch, Save As, Compact, TVD rebuild and SQL query all off the event loop). **pre-window startup** ✅ **shipped 2026-07-30** (window first, project opens behind a boot overlay). Remaining: connection pool [**high-risk**] (#129), needs a live 100-well run to sign off.
- **Reliability sliver**: modal Escape-key stacking — ✅ **shipped 2026-07-20** (Escape scoped to the top dialog; single-instance already prevented leaked handlers).
- **Interpretation-workflow open** (§4): data-prep split/merge + tops-referenced normalization, highlight tool, typography check.
- **Feature Wave B** (§4c): MC parameter **sensitivity/tornado** (13), ML comparison + leaderboard (3), fluid contacts in correlation (9), well-diagram track (16), rock typing + SHF fitting (8).
- **Low backlog** (§4b, 15 items): ✅ **fully closed 2026-07-21** — #134 shipped 10 safe fixes (1 already fixed); the 4 held items are now resolved per Jauhar (#135): Wyllie Cp opt-in ✅, depth-scale dropdown + mislabel ✅, quiet Ctrl+S + ribbon-Esc ✅, Bahasa Jawa + fuller id/su ✅; histogram full-range re-bin **declined (left as-is)**. cargo 164 / tsc 0, browser-verified.
- **Carried-forward deferrals** from the build arc: per-well param override table, MC print-to-curves + per-zone distributions, missing-curve synthesis, auto-picks / auto-zonation, lazy catalog + decimation cache + 2000-well stress fixture.

### 🔮 Future — bigger lifts  → [Part C](#-part-c--future)
- **Method-suite waves** (§4c Wave C): thin-bed / LRLC suite (10), TOC / unconventional (1a), 1D geomechanics MEM (1b), rock physics (15).
- **New data-model suites** (§4c Wave D): NMR (5), image logs (6), core-photo digitization (7).
- **Trust & reproducibility** (Phase 11): audit lineage, scenario A/B compare, command palette.
- **Platform & extensibility** (Phase 12): user Python modules, native DLIS / LAS 3.0 / WITSML, installer + auto-update, in-app help; long game — NMR arrays, images, geomechanics, production logs.
- **New-capability misc** (§4): 2D map window + volumetrics, plugins ribbon, panes independent of windows, data digitization tools, user-guide PDF.

---
---

# ✅ PART A · DONE

Shipped, tested, and in the app. A few phases carry a small **Deferred:** sliver — those open
bits are collected in [Part B](#-part-b--open-do-next) / [Part C](#-part-c--future).

## A0. History — Phases 1–5 (§0): shell, plots, data management, equations

Built from the original Techlog-style redesign brief (MDI workspace, ribbon reorg, layout
quality, plot upgrades, data management, Python equations). All five shipped, with a few
deliberate deviations from the original plan where reality disagreed with the plan:

- **Phase 1 — Shell**: dockview-core MDI workspace (float/dock/tab/split/maximize any
  panel), 5-tab Office-style ribbon (Project/Data/Petrophysics/Plot/View), light/dark/system
  theme setting, per-window tab-bar actions (＋ add-panel, maximize, float, dock-back, close).
- **Phase 2 — Layout quality**: WebGPU curve fills (fill/fill_color/fill_opacity), Layout
  Properties dialog, named layout save via the `documents` table, synchronized crosshair +
  per-track hover readout.
- **Phase 3 — Plot upgrades**: histogram (bars/line/cumulative + stat chips + normalize),
  crossplot (least-squares regression + R², manual ranges, log axes), plot properties
  persisted per plot kind, synchronized `hoverDepth` across all open panels.
- **Phase 4 — Data management**: Database Inspector (whitelisted paged reads + explicit
  update commands, `TABLE_SPECS` in `db.rs`), global undo/redo (`src/undo.ts`, Ctrl+Z/Y),
  read-only SQL Query panel (full DuckDB SQL, SELECT/WITH only).
- **Phase 5 — Equations**: **deviated from the original PyO3 plan** — Python runs as a
  **subprocess** (`python_engine.rs`), not embedded via PyO3. Reason: PyO3 links
  `python312.dll` at load time, and this machine's PATH resolves to a bare Python 3.8 with
  no numpy, which would have made the app fail to launch. The subprocess approach with
  explicit discovery order (`SANDIBUMI_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python31x`
  → PATH) is documented in `CLAUDE.md` rule 7 and is the standing constraint for any future
  Python integration (including Phase 6's DLIS import via `dlisio` and Phase 10's
  scikit-learn facies work — both use the same subprocess mechanism, not PyO3).
- Also shipped ahead of the original 5-phase plan, pulled forward from the Mahakam gap
  analysis: Thomas-Stieber thin beds, multi-well correlation view, splice/depth-shift,
  core data import + overlay (§2 below), and DuckDB WAL self-healing (`db::init_db_resilient`,
  see `CLAUDE.md` "DuckDB WAL resilience").
- **Selection-driven workspace ✅ (2026-07-19)** — the Wells & Tops pane now decides what
  every panel displays: histogram/crossplot/pickett panels **follow well selection**
  (rebuild for the new well, carrying curve/zone choices over via `PlotContent.getState` →
  builder `initial`; retitle to "Kind — WELL"; a panel opened with no well shows a
  placeholder and builds on first selection). **Tops are clickable**: clicking a top
  selects the interval down to the next top (last top → TD, open-ended `depthMax: null`)
  as `appState.selectedInterval`; every zone dropdown grows an auto-selected
  "Top X (min–max)" window option (fires `change` → plots reload windowed, pick writes
  target zone = top name), and every log view of that well scrolls to the top
  (`LogCanvasRenderer.scrollToDepth`). Interval clears when a different well is selected
  (before the well broadcast, so followers never see a foreign interval). Selected
  well/top rows are highlighted (`.tree-selected`/`.top-selected`). Module/summary/MC/
  workflow dialogs already pre-selected the selected well — processing side unchanged.
- **FACIES block track ✅ (2026-07-19)** — discrete class curves render as colored
  depth-interval blocks. `CurveStyle.fill` gained a `"blocks"` variant (free-form
  `Option<String>` in Rust, so no serde change): contiguous same-class runs become
  full-track-width rectangles colored from the shared facies palette
  (`plotCanvas.ts FACIES_PALETTE`, duplicated in `composite.rs` — keep in sync), NaN
  runs stay empty, the last run extends one average sample step. Implemented in both
  renderers: WebGPU viewer (`LogCanvasRenderer.buildBlockGeometries`, one geometry per
  class through the existing alpha-blended fill pipeline) and print composite
  (`composite.rs draw_class_blocks`). New built-in **"Facies" layout**
  (GR / RES_DEEP / NPHI-RHOB / FACIES blocks) in the ribbon layout picker; Layout
  Properties fill dropdown gained "Facies blocks"; track header shows a striped palette
  swatch + "class blocks" instead of an editable min/max scale. Min/max decimation
  passes discrete values through unaveraged, so no backend data change was needed.
- **UX fix batch ✅ (2026-07-19, from Jauhar's click-through)** — Batch ribbon group
  overflow fixed (`.ribbon-btn-row`); Import DLIS moved to Import/Export; dialogs made
  non-blocking (pointer-transparent scrim, app stays clickable, Esc/✕ close); log views
  now all follow well selection with a per-panel 📌 pin to hold a well; depth scale/zoom/
  track-width moved from the View ribbon into a per-log-view mini toolbar
  (`logViewPanel.buildTools`); window resize keeps pane sizes (only the largest pane
  absorbs the delta — `workspace.relayoutKeepingPaneSizes`).
- **GMM soft electrofacies ✅ (2026-07-19, Phase 10-3a)** — `gmm_facies` module in
  `facies.rs`: diagonal-covariance Gaussian mixture fitted by EM (log-sum-exp E-step,
  variance floor 1e-4), initialized from the best-of-8 k-means run so it agrees with
  `electrofacies` on separated data. Outputs FACIES_GMM (hard label by max posterior,
  GR-ordered like k-means) + **FPROB** (winning posterior 0–1) — boundary/mixed beds show
  low FPROB instead of being silently forced into a class. Same CURVE1–5 slots, z-score,
  seed determinism; auto-appears in the Facies ribbon dropdown, categorical crossplot
  coloring matches (/FACIES/ regex), and FACIES_GMM can use the blocks track.
- **UI language option ✅ (2026-07-19)** — English / Bahasa Indonesia / Basa Sunda select in
  the Project ribbon tab (`src/i18n.ts`). Dictionary-driven DOM translation (text nodes +
  title/placeholder/aria-label/optgroup-label) with a MutationObserver covering dynamically
  built panels/dialogs; persists in `localStorage("sandibumi.locale")`. English stays the
  source language in code; **phrases missing from the dictionary intentionally stay
  English** — that is how technical vocabulary (Thin Beds, Monte Carlo, Pickett, mnemonics)
  is kept untranslated per Jauhar's request. Never add jargon to the dictionaries.
- **ML suite ✅ (2026-07-19, Phase 10-4)** — Jauhar's full supervised/unsupervised catalog
  behind one "ML Models…" dialog (Petrophysics → Machine Learning): `ml.rs` runs
  scikit-learn in the python_engine subprocess pattern (JSON header + raw f32). Regression:
  RF, Gradient Boosting (XGBoost w/ sklearn fallback), SVR, MLP-ANN, linear/polynomial.
  Classification: SVM, KNN, RF, Gaussian NB, logistic — writes class + `_PROB`. Clustering
  (fits POOLED apply wells → **field-wide, globally consistent ids**): K-Means, GMM (+`_PROB`),
  hierarchical, DBSCAN (noise → NaN); ids ordered by first-feature mean like the native facies
  modules. Reduction: PCA (PC1…PCn + explained variance), t-SNE (TSNE1/2, 20k-sample cap).
  Supervised tasks pool labelled samples from train wells and predict on apply wells;
  incomplete rows are masked and come back NaN. Metrics (5-fold CV R²/accuracy, silhouette,
  class/cluster counts) surface in the dialog. Autoencoders deferred (needs PyTorch).
- **Interactive plots ✅ (2026-07-19)** — the histogram/crossplot/Pickett panels were fixed
  bitmaps (720×460) CSS-stretched to the panel, so they looked like blurry screenshots.
  `PlotCanvas` now works in logical (CSS) pixels with a `fitCanvasBackingStore()` HiDPI
  backing store sized to the real panel area, re-rendered via `attachResizeRedraw`
  (ResizeObserver). New `attachZoomPan` gives wheel-zoom-to-cursor + drag-pan + double-click
  reset in each axis's transformed space (log-correct) via a shared `ViewportRef`. The
  crossplot gained a draggable **parameter handle** at (X pick, Y pick): dragging it sets
  both zone parameters live (shale/matrix point), generalizing the old T-S-only drag; a
  press on a handle vetoes the pan, and a moved pointer suppresses the click-pick.
  Correlation already did HiDPI. **Deferred**: draggable cutoff polygon (lasso a region to a
  class); per-axis independent zoom lock.
- **Well groups ✅ (2026-07-19)** — user-defined named subsets of wells so a 2000+ well
  field stays workable. Tables `well_groups` + `well_group_members` (manual membership;
  `rule_json` reserved for future rule-based auto-membership), whitelisted CRUD +
  `set_active_well_group` (at most one active) in `db.rs`/`lib.rs`. `appState.activeWellGroup`
  + `filterByActiveGroup()`; the Wells pane got a group dropdown + ⚙ manager (create/rename/
  delete/membership checklist), and **every batch dialog** (module run, workflow, Monte
  Carlo, dashboard, report, correlation, pay summary, ML, equation run-all) is scoped to the
  active group — "global filter" semantics. **Deferred**: rule-based auto-membership (by
  field/name/attribute); create-group-from-tree-multiselect.
- **Shell UX batch ✅ (2026-07-19)** — five UX asks from Jauhar's annotated screenshot.
  (1) **Quick access toolbar** left of the ribbon tabs: Undo/Redo (`undo.ts` gained
  `onUndoChange`/`redoDepth`/`nextUndo|RedoLabel`; buttons + Ctrl+Z/Y), Save Project As
  (moved out of the Project ribbon tab), and History — wired in `ribbon.ts`, markup in
  `index.html`. (2) **Processing history** (`processLog.ts` + `historyPanel.ts`): a
  timestamped audit of imports/module-runs/edits/exports/pins, persisted to `documents`
  (`history`/`log`) so it survives restarts and Save-As; `recordProcess()` called at the
  operation sites. (3) **Global well lock** replaces the per-panel log-view pin:
  `appState.pinnedWellId` + `setPinnedWell`/`isSelectionBlocked`; the Wells & Tops group bar
  has a 📍/📌 lock that freezes the active well (tree click on another well is blocked while
  locked). (4) **Right-click restriction**: `workspace.attachContextMenu` now suppresses the
  custom menu over any interactive control/editor/table/toolbar/tree-row, so it only appears
  on empty pane background + plot canvases. (5) **Plot export + templates**: `plotExport.ts`
  (copy-to-clipboard / save-PNG via `save_png` / print-via-iframe) added as toolbar buttons
  AND canvas context-menu entries on all four canvas plots; log view routes to the Composite
  dialog. Named plot templates via `buildPlotTemplateBar` in `plotCommon.ts` (doc_type
  `plottmpl:<kind>`) on Histogram + Crossplot. **Deferred**: templates on Pickett/Correlation
  (export only there); re-runnable-workflow view of the history; WebGPU log-view direct image
  capture (uses Composite instead).

## A1. Priority gaps — all closed (§2)

- ~~PT09_ThinBeds (Thomas-Stieber)~~ **DONE** — `thin_bed_ts` module (VLAM/VDISP/VSAND/PHIE_LAM).
- ~~Correlation view~~ **DONE** — multi-well strips, tops connectors, flatten on datum.
- ~~SpliceLogs + depth shift~~ **DONE** — `depth_shift` (zone-overridable block shift) +
  `splice` modules; undoable core-to-log "Shift Core…".
- ~~PT12_CoreAnalysis~~ **DONE** — core CSV import (percent→v/v, alias headers), crossplot
  + log-track overlays, inspector editing.
- Everything still open is in [Part B](#-part-b--open-do-next) / [Part C](#-part-c--future).

## A2. Data foundation — Phase 6 (§3): arbitrary curves, units, TVD  ✅ (mostly)

*Why it came first: `standard_curves` is hard-coded to 6 mnemonics (GR/RES/NPHI/RHOB/DT/SP).
PEF, CALI, DRHO, RXO, multiple runs, arrays — none could even be imported, which blocked
multimin (needs PEF), environmental corrections (needs CALI), bad-hole QC, and DLIS.*

- ~~**Database (6a, DONE 2026-07-17)**~~: generic curve store shipped as an **additive**
  layer alongside `standard_curves` — `curve_meta(curve_id, well_id, set_name, mnemonic, unit,
  family, source, run_no)` + `curve_samples(curve_id, depth, value)`, curve **sets**
  `RAW`/`EDIT`/`FINAL`, `well_path(well_id, md, inc, azi, tvd, tvdss)` for deviation.
  `migrate_standard_curves_to_generic_store` runs on every launch, idempotently backfilling
  GR/RES_DEEP/NPHI/RHOB/DT/SP into the generic store as set RAW with real units;
  `upsert_curve_meta`/`insert_curve_samples`/`get_curve_samples`/`list_generic_curve_catalog`
  are the read/write API (`db.rs`). 18 Rust tests pass (incl. an idempotency check and a
  NaN-vs-NULL fix: DuckDB's `IS NOT NULL` is true for NaN, so the migration's "does this column
  have real data" check needed `AND NOT isnan(col)`).
- ~~**6b (mostly DONE 2026-07-17)**~~ — **Backend**: LAS import now keeps **every** curve.
  `ingest::import_all_curves_into_generic_store` re-reads the file with `parsers::parse_las_2_all`
  (streams all `~C` curves + units) and writes each into `curve_meta`/`curve_samples` as set RAW.
  Mnemonic dictionary + unit conversion live in `curves.rs` (`family_for`, `convert_to_canonical`
  — us/m→us/ft, kg/m³→g/cc, pu/%→v/v, mm/cm→in). Deviation survey import + minimum-curvature
  TVD/TVDSS in `deviation.rs` (+ `parse_deviation_csv`, `db::insert_well_path`/`get_well_path`,
  IPC `import_deviation_csv`/`get_well_path`).
- ~~**DLIS import via `dlisio` (DONE 2026-07-17)**~~: `dlis.rs` runs `dlisio` through the Python
  subprocess — a helper streams every scalar channel of every frame as a JSON header + raw f32
  columns; Rust writes them into the generic store as set RAW, family-tagged + unit-canonicalized
  (frame ordinal → `run_no`). `dlisio 1.0.4` in the SandiBumi Python env. IPC `import_dlis_file`.
- ~~**6c — Frontend (DONE 2026-07-17)**~~: Curve Catalog shows the generic store per selected
  well (mnemonic/unit/family/set/source/samples + live text filter, "· run N" badge), backed by
  `list_generic_curve_catalog`, falling back to the legacy standard+computed view when no well is
  selected. Data ribbon gained **Import DLIS…**, **Import Deviation…** (datum/KB → minimum-curvature
  TVD/TVDSS), **Well Header…** (field/TD/KB editor).
- **Deferred** (carried to [Part B](#b4-carried-forward-deferrals-from-the-build-arc)): rewiring
  `get_track_data` (the log-view read path) to read from the generic store — log views still read
  `standard_curves`, so PEF/CALI aren't drawable in a track yet. (The **module/equation** input
  path `fetch_curve_frame` DOES fall back to the generic store — that's what unblocked
  multimin/bad-hole.) Also deferred: a curve-set selector in the layout picker, and the optional
  TVD depth scale in the log/correlation views (`deviation::tvd_at` + `LasFrame.depth_unit`
  plumbing already built, `#[allow(dead_code)]`-tagged).
- **Done when**: a real 30+ curve LAS (and a DLIS) imports whole ✓; PEF/CALI in the catalog ✓;
  TVD matches hand calculation ✓; every existing feature still green ✓.

## A3. Interpretation physics — Phase 7 (§3): the Mahakam pack  ✅

- ~~**Generic-store read fallback (DONE 2026-07-17)**~~ — `equations::fetch_curve_frame` resolves
  any non-standard, non-computed curve name from `curve_meta`/`curve_samples` (set RAW) via
  `fetch_named_curve_aligned` → `fetch_generic_curve_aligned`, matching on mnemonic first then
  family (so a module asking for "CALI"/"PEF"/"DRHO" finds an HCAL/PEFZ/HDRA curve by family),
  preferring the base run. Additive — log views still read `standard_curves`.
- ~~**Bad-hole QC (DONE 2026-07-17)**~~ — `badhole` module (Prep): BADHOLE = 1 where |DRHO| >
  DRHO_MAX or (CALI − bit size) > DCAL_MAX (bit size from BS curve or BS_DEF), MISSING with no QC
  curve. **Central mask capability** in the runner: any module run passing `opts["MASK"] = "<flag
  curve>"` gets flagged samples (==1) NaN'd out of every output — zero per-module code. UI: one
  universal "Mask (optional)" picker in the auto-generated module dialog.
- ~~**Multimin (PT07, DONE 2026-07-17)**~~ — `multimin.rs` (separate from async-job `inversion.rs`):
  constrained weighted least-squares 4-component inversion (SAND/CLAY/WATER/HC) from RHOB/NPHI/DT/PEF,
  non-negative volumes via a hand-rolled Lawson-Hanson **NNLS** with a heavily-weighted unity row;
  each tool equation scaled by 1/sigma. Outputs VOL_SAND/CLAY/WATER/HC, PHIT_MM, VSH_MM, SWT_MM,
  RECON_ERR. Recovers a forward-modelled 70/30 clean wet sand within 2 %. 30 Rust tests.
- ~~**Generalized Multimin — Increment A (DONE 2026-07-19)**~~ — `multimin2.rs`: ELAN-style **N
  user-defined** minerals/fluids from an editable 15-entry library against any subset of
  RHOB/NPHI/DT/GR/PEF/U, with **hard** unity (Σv=1) + non-negativity via equality-constrained
  active-set NNLS over the probability simplex. Command `run_multimin`/`multimin_library`; **Advance
  → Multimin…** dialog (editable endpoint matrix + Clay/Poro/Water roles). Outputs VOL_<comp> +
  `<prefix>`_PHIT/VSH/SWT/RECON. 4 solver tests + suite pass.
- ~~**Generalized Multimin v2 — the reference suite parity (DONE 2026-07-19)**~~ — spec from the local the reference install
  Multimin helpset + IP2018 Mineral Solver. **27-component library** in IP dropdown order (12
  minerals, 6 clays with CEC, 7 zone-typed fluids), **16 input logs + user-defined**. Resistivity
  enters as conductivity via the **dual-water linear transform** (Ct^(1/w) row, w=0.75m+0.25n) —
  Sw/Sxo come out of the volume solve itself (supersedes the old outer-loop design). Hard unity over
  minerals+U-fluids, POROSITY (ΣX=ΣU) and BNDWAT soft σ=0.01 rows, WATER MUD re-solve, hard bounds
  (fluids ≤0.5). New `solve_bounded_lsq`, `multimin_fluid_calc` preview, rebuilt the reference suite-style dialog.
  7 solver tests incl. Sw=0.40/Sxo=0.80 recovery from CT/CXO; 84/84 suite; tsc clean.
- ~~**Saturation-height (PT11, DONE 2026-07-17)**~~ — `scal_pc` table + **Import SCAL…** +
  `satheight.rs`: `fit_leverett_j` (Sw = A·J^B by log-log LSQ) and the `sw_height` module — LEVERETT
  (needs PERM) or SKELT (no perm); SWH = 1 at/below the zone-overridable FWL; outputs SWH + HAFWL.
  *Not built*: Skelt-Harrison auto-fit (manual params) and the Pc/J-vs-Sw QC plot (`get_scal_pc` ready).
- ~~**Environmental corrections (PT03, DONE 2026-07-17, pragmatic analytic)**~~ — `gr_hole_corr`,
  `nphi_env_corr` (needs FTEMP), `rhob_hole_corr` as Prep modules; coefficients are params at
  chartbook magnitudes; a missing QC curve passes the log through uncorrected. Chart-lookup fidelity
  stays future work.
- ~~**Thomas-Stieber interactive crossplot (DONE 2026-07-17)**~~ — "T-S triangle" on the crossplot
  (X=VSH, Y=PHIT): laminated + dispersed lines with **draggable endpoint handles** (sand handle sets
  PHI_SD_MAX, shale handle sets PHI_SH → zone params on drag release, feeding `thin_bed_ts`).
- **Done when**: multimin volumes match the reference suite within tolerance ✓ (unit); SWH tracks core Sw (field
  click-through pending, `REVIEW.md`); corrections change curves in the right direction ✓. 37 tests.

## A4. Deliverables — Phase 8 (§3): composite plots & PDF reports  ✅

*This is what clients and partners actually see — the LQR deliverable.*

- ~~**Composite plot designer + vector export (8a, DONE 2026-07-17)**~~ — `composite.rs` renders a
  `Layout` at a TRUE print scale (1:200/500/1000) into backend-neutral `DrawOp`s (mm space), then
  serializes to **SVG** or a **dependency-free multi-page PDF** (hand-rolled writer, base-14
  Helvetica — chosen over `svg2pdf`/`usvg` to avoid a heavy font-DB dep on the already-large
  bundled-DuckDB build). Full header block, depth axis + grids, per-track frames + scales, curve
  polylines with NaN/off-page breaks, edge fills, top lines + labels, zone bands, exact page split.
  Data via `fetch_curve_frame` (standard/computed/generic all render). IPC `render_composite` /
  `export_composite_svg` / `export_composite_pdf`. UI: Plot ribbon "Composite…". Verified: SVG curve
  path exactly 233 mm tall (46.6 m × 5 mm/m at 1:200); PDF structurally validated. 42 Rust tests.
- **Deferred — hatch lithology/facies track + text/arrow annotations** in the composite (needs a
  facies curve → Phase 10 done, and an annotations store on `documents`).
- ~~**Report generator (8b, DONE 2026-07-18)**~~ — `report.rs` reuses the 8a DrawOp/PDF machinery:
  cover → methodology parameter–method–remarks table (editable, persisted as `report_template`) →
  per-zone parameter table → pay summary table → composite pages, as one PDF. Paginated word-wrapped
  tables. IPC `render_report` / `export_report_pdf` / `export_report_batch` / `save_png`. UI: Plot
  ribbon → Deliverables → Report… (`reportDialog.ts`).
- **Deferred from 8b**: histogram/crossplot pages, per-formation narrative text, bilingual headings,
  executive-summary page, SWHF section, correlation-panel export.
- **Done when**: one command produces a client-ready multi-page PDF for a Balam well ✓ (8a + 8b).

## A5. Jauhar method suite — Phase 8.5 (§3)  ✅ (DONE 2026-07-18)

*His own field-proven methods as first-class core modules, studied from the 7 reference projects
(LQR Balam South, Glagah Kambuna, Wanda Gita, Bunga Block, LRLC research, KKT, BLSO). Math banked in
auto-memory (`method-ssc-sspw-lqr`, `method-lrlc-imts-rtc`, `method-workflow-standards-jauhar`).*

- ~~**`ssc` (Porosity)**~~ — full port of `ssc_lqr_gap_edit_jau.lls` (Kuttan/GAP 2023 SSC): gas
  conditioning, N-D projection onto the dry rock line, sand/silt/clay fractions, PHIT from mixed
  matrix density, CBW/CWSH bound-water split, SWIRR, GR-equivalent volumes. Deterministic replacement
  for `RANNORMAL`; NPHIMA limit bug in the Loglan fixed deliberately (noted in the module header).
- ~~**`sspw` (Porosity)**~~ — PHR-standard sandstone workflow; exec reconstructed from the `.info`
  spec — **validate vs the reference suite "LAS PHIT PHIE" outputs**.
- ~~**`sw_rtc` + `sw_imts` (Saturation)**~~ — the LRLC research models: excess-conductivity
  correction and iterative mineral-textural-scaled Waxman-Smits with Qv_eff = Qv_bulk/(1−Swirr),
  Juhasz B(T,Rw).
- ~~**`gr_normalize` (Prep)**~~ — two-point percentile GRN, Rokan reference defaults P3 = 53.68 /
  P97 = 133.93 gAPI.
- ~~**`log_predict` (Prep)**~~ — Facimage-MRGC-style synthetic logs by leave-one-out
  distance-weighted KNN, with the MAX_RAW washout rule for RHOB.
- ~~**Mnemonic dictionary enrichment**~~ — Bunga standardization table merged into `curves.rs`
  FAMILIES (ROBB/SBD2/HDRA/FSTP/ATR/BDAV/RING/PSR/R25P/BSAV/SN/HORD/PEB/DT_S…).
- **Deferred**: SSC-in-multimin presets (Wanda Gita), variable-m carbonate (SPI) module (Bunga),
  per-zone multimin component presets (KKT), FZI rock typing module (→ Wave B item 8).

## A6. Field scale — Phase 9 (§3): batch workflows, uncertainty, dashboards  ✅

- **Workflow chains** ✅ (2026-07-18, inc. 1): `chain.rs` runs an ordered list of modules across
  many wells — steps sequential, wells rayon-parallel per step via `run_workflow_module`. Progress +
  cancellation via a pollable registry (not Tauri events): frontend supplies the job id, calls
  `run_workflow_chain`, polls `get_chain_status`, `cancel_workflow_chain` flips a shared flag checked
  between steps. Chains persist as `workflow` documents. Frontend: Workflow Builder
  (`workflowDialog.ts`, Petrophysics → Batch).
- **Per-step parameter editing** ✅ (2026-07-18, inc. 2): each step has an expandable ⚙ editor
  (manifest-driven) — input selectors, options, validated params, universal bad-hole Mask. Only
  non-default values are stored on the step; `zone_params` still override per zone at run time.
  Override-count badge + Reset. Persists in the `workflow` document.
- **Per-well parameter override table** ✅ (2026-07-30, Phase 9-2 closed): `wellParamsDialog.ts`,
  opened from the Workflow Builder's run bar. Rows = wells, columns = the numeric params the chain's
  steps take. No new resolution machinery — a cell writes the `zone_params` whole-well row (`zone_name
  = '*'`) that `resolve_param_arrays` already applies, so the order stays step → whole-well → named
  zone. Columns are keyed by param NAME, not by step, because the storage is: one RW override applies
  to every step taking RW, and a column per step would imply an independence that does not exist.
  Backend `list_well_param_overrides` (one scan, no per-well round trips) + `set_well_param_overrides`
  (one transaction — fill-column and its undo are the same atomic shape). Cells are text until
  double-clicked (the app-wide click-to-arm rule, and what keeps thousands of rows cheap); amber =
  overridden, grey = inherited; typing the inherited value clears the override; out-of-range values
  are REFUSED, mirroring `resolve_param_arrays`, so a percent-typed fraction becomes a red cell rather
  than a failed 2,000-well run. A column sweep is one undo entry. Copy-as-CSV out; **CSV import back
  is the obvious follow-up and is not built**.
- **Monte Carlo uncertainty (PT06)** ✅ (2026-07-18, inc. 3): `montecarlo.rs` — put
  normal/uniform/triangular distributions on any model parameter, run N seeded realizations of a chain,
  get P10/P50/P90 net pay / NTG / avg PHIE / avg SWE / HPV **per zone** + an HPV histogram. Runs
  **entirely in memory** (`run_module` returns curve vectors; nothing writes `computed_curves`), so it
  sidesteps the field-scale write bottleneck — 1000 realizations finish in well under a second.
  Rayon-parallel, each seeded from `(seed, index)` for reproducibility. UI: Petrophysics → Batch →
  **Monte Carlo…** (`monteCarloDialog.ts`). **Deferred** (→ [Part B](#b4-carried-forward-deferrals-from-the-build-arc)):
  per-zone parameter distributions (currently well-wide), persisted P10/P50/P90 *curves*, and
  parameter **sensitivity/tornado** (that's Wave B item 13).
- **Field dashboard panel** ✅ (2026-07-18, inc. 4): `dashboardPanel.ts` runs `run_pay_summary`
  across **every** well at chosen cutoffs → per-zone aggregation table, per-zone box plots (inline SVG),
  sortable multi-well × zone grid, filterable by flag level, CSV export. Frontend-only; reuses the
  pay-summary command. *Caveat*: a full-field compute incurs the `computed_curves` write cost — the
  perf-hardening increment below addressed the worst of it.
- **Performance hardening — write path** ✅ (2026-07-19, inc. 5): killed the `computed_curves` write
  bottleneck. Root cause (proven by the in-harness probe in `pipeline_blso_test.rs`) was the 3-column
  `PRIMARY KEY (well_id, depth, curve_name)` — its ART uniqueness index cost ~3.4× per row. **Dropped
  the PK** (`migrate_drop_computed_curves_pk` rebuilds the table PK-less on launch, idempotent);
  uniqueness now guaranteed by the write discipline (DELETE target curve names before appending;
  point-updates UPDATE in place). Also **batched** each well's whole module output into one DELETE +
  one Appender/flush (`write_computed_curves_batch`). Net: the real 100-well × 4-module chain dropped
  from ~50s to **21s** (~2.3×). 72 Rust tests. **Still open** (→ [Part B](#b4-carried-forward-deferrals-from-the-build-arc)):
  lazy catalog loading, decimation cache, UI responsiveness during full-field runs, 2000-well stress
  fixture.
- **Done when**: a 100-well chain runs with live progress in minutes ✅ (21s); MC 1000 realizations
  per well finishes in seconds ✅.

## A7. Facies & assisted interpretation — Phase 10 (§3)  ✅ (facies shipped)

- **Electrofacies (PT15)** — unsupervised k-means ✅ (2026-07-18, inc. 1): `facies.rs` `electrofacies`
  module (Facies ribbon category). Up to 5 input curve slots (GR required; RHOB/NPHI/DT/SP optional),
  z-scored by default, then k-means++ (dependency-free, best-of-8 restarts) partitions complete samples
  into K facies (2–12). **Labels reordered by ascending mean of the first curve** (usually GR), so
  FACIES 0 is cleanest and numbering is monotone in shaliness. Missing any present curve → MISSING.
  Deterministic. Output FACIES → `computed_curves`. Frontend QC: **crossplot categorical coloring**
  (FACIES/CLUSTER/LITHO/CLASS → fixed qualitative `FACIES_PALETTE` + swatch legend). 4 Rust tests.
  **GMM (soft clustering)** and **field-wide pooled clustering** shipped via the ML suite (A0) and
  `gmm_facies` (A0). **Colored FACIES block track** shipped (A0).
- **Missing-curve synthesis** — open (→ [Part B](#b4-carried-forward-deferrals-from-the-build-arc)):
  train per-field regressors to predict DT/NPHI where absent; holdout-well R² report.
- **Auto-picks** — open (→ [Part B](#b4-carried-forward-deferrals-from-the-build-arc)): per-zone
  GR_MA/GR_SH percentile suggestions, change-point auto-zonation, spike/outlier QC across the field.

## A8. Field review — Trust & safety + Interpretation workflow (§4)  ✅

The complete feature/fix list from Jauhar's o/x click-through (2026-07-19). Two tiers here are **done**;
the Interpretation-workflow tier has a few open items pulled into [Part B](#b2-interpretation-workflow-open-4).

### Done same-day (2026-07-19)
- ✅ **Ctrl+wheel = zoom** on histogram/crossplot/Pickett; plain wheel scrolls the page.
- ✅ **Pertamina theme** now uses the official palette (#ED1A2F / #006BB8 / #A6C210 / #161B22).
- ✅ **"Light" renamed "Default"** in the theme dropdown.
- ✅ **Advance tab regrouped**: one "Advance Methods" group = SSC, SSPW, RtC, IMTS, **Thin Beds**.
- ✅ **Multimin renamed → SandiMin**; the legacy fixed 4-component "Multimin — Mineral Inversion" is
  removed from the Saturation dropdown (still callable from saved chains).
- ✅ **Repo made collaboration-ready**: .gitignore hardened, CONTRIBUTING.md added, work committed
  to git (remote hosting = Jauhar's choice).

### Trust & safety — DONE  _(field-review tier, was "P1"; protect the user's work first)_
- ✅ **Crash resilience** (P1-b): `autosave.ts` — running-flag crash detection, 10-s rolling autosave
  of the full session snapshot to localStorage; abnormal exit → blocking choice before boot (restore
  autosave, or Safe Mode). Normal launches reapply well + log-view layouts via `applyAutosaveExtras`.
- ✅ **Unsaved-changes indicator** (P1-b): `dirty.ts` registry — log-view edits mark the panel (tab ●,
  QAT Save-Session dot); Save Layout clears that panel, Save/Open Session clears all.
- ✅ **Click-to-arm, double-click-to-edit inputs** (P1-a): app-wide via `interactionGuard.ts` — a
  single click arms `input[type=number]` read-only; double-click unlocks; blur re-arms. Tab focus stays
  editable; per-input opt-out with `data-free-edit`.
- ✅ **Right-click lockdown** (P1-a): default WebView menu killed except editable fields; F5/Ctrl+R
  guarded by a blocking confirm; Alt+arrows and mouse back/forward blocked.
- ✅ **Workflow builder as a pane, not a popup** (P1-a): dock component "workflow" (singleton); closing
  the pane mid-run cancels the chain.
- ✅ **Database versioning — never overwrite** (P1-c): `log_sets` run-event table + append-only
  `computed_curves_archive`; `computed_curves` stays the fast "current" store (rows tagged `set_id`).
  Module runs, chains, equations (EQUATION), ML (ML), SandiMin (SANDIMIN) all write versioned; re-run =
  version N+1, any version restorable/prunable. Provenance per run: module, params, inputs, timestamp.
  Catalog: merged view with set/version/module/when + n/min/max/mean, one search box, click-to-sort.
- ✅ **Set INPUT selection on modules** (P1-c follow-up): "Input set" field in every module dialog and
  the Workflow Builder; inputs resolve from that set's archived values, falling back to the usual
  sources. Blank = current values. Provenance `inputs_json` records the input set.
- ✅ **Curve catalog power features** (P1-c): one search box across mnemonic/set/module/unit/date,
  click-to-sort columns, per-curve n/min/max/mean.

### Interpretation workflow — DONE  _(field-review tier, was "P2")_
- ✅ **Imports (tops-style)** (P2-a): "Import Tops…" (CSV/TXT, alias headers, multi-well) + "Import
  Aux…" (PETROGRAPHY / XRD / PERFORATION / custom into `aux_data`). Deferred: aux overlays on plots.
- ✅ **Tops editor, Petrel-style**: log views draw tops as labeled colored lines; 🏷 toolbar toggle
  enables click-add / drag-move / double-click edit (all undoable); **stratigraphic-crossing warning**
  (`tops.rs::check_top_order`); **marker autocorrelation** (Data → Autocorrelate…, `autocorrelate_top`).
  Deferred: named tops SETS (multiple schemes per project).
- ✅ **Well pin semantics rework**: the 📌 pin is now a MODE. Pin ON = selecting a well drives the
  whole workspace. Pin OFF = viewers keep their wells, only the ACTIVE panel follows. **Multi-select**:
  Ctrl-click toggles, Shift-click ranges, ⇄ inverts; batch dialogs pre-tick the multi-selection
  (`defaultRunWellIds`).
- ✅ **Log-view layout interaction**: ▤ collapsible track headers; drag a curve between tracks to MOVE
  (Ctrl = copy), undoable; ▦ customizable track borders; hover readout scoped to ONE track; right-click
  → "Edit CURVE…" (shift/const/blank/interpolate/scale) via `curve_edit.rs` transactional read-modify-
  rewrite, undoable bit-exactly (`restore_curve_values`), recorded in Processing History.
- ✅ **Histogram v2** (2026-07-20): the reference suite-style Properties dialog (mode, bins, normalize, cumulative-%,
  box-plot strip P5–P25–P50–P75–P95, custom bar color, user percentiles, Min/Max, statistics placement).
  Parameter pickers now opt-in — a fresh histogram is a neutral frequency tool.
- ✅ **Crossplot v2** (2026-07-20): sectioned Properties dialog — plot size (fill or fixed W×H),
  marginal histograms on X/Y, custom point color + "— None —" Z, user percentiles, regression = model
  (linear/power/log10-X/exponential) × method (Y-on-X / X-on-Y / RMA), Z color = colormap
  (rainbow / **viridis**) + **log Z scaling**, overlays. Matrix points + pickers now opt-in.
  ~~D-N porosity overlay~~ DONE 2026-07-20 (Por-11/Por-12 digitized into `dnChartData.ts`; fresh & salt
  variants; chart dolomite ρma 2.85 per its own graduation ticks).

## A9. Hardening — Critical & Reliability tiers (§4b)  ✅

Jauhar asked for a full "30-year senior petrophysicist" recheck. Five parallel review passes
(methods, frontend bugs, performance, UX, data integrity) + an adversarial verify pass on every
high/medium claim: **35 confirmed, 0 refuted, 15 low**. Full detail with file:line evidence in
`AUDIT-2026-07-20.md`. The Performance / Polish / Low tiers are open — see [Part B](#b1-hardening-backlog-4b).

**Critical (was "P0") — answers were wrong or silently unsafe** — all eight fixed + unit-tested
2026-07-20, both residuals closed under #118 (lib suite 160 pass / 0 fail, tsc clean; click-through in
REVIEW.md "P0 senior-audit backlog"). #118 was adversarially reviewed (4 lenses → per-finding verify;
5 confirmed findings folded in below).
- [x] MASK now blanks module INPUTS before the run, not just outputs — gr_normalize P3/P97 and
      log_predict KNN training see only unmasked samples, and the masked log_predict synthetic survives
      inside the washout. (workflow.rs; test `mask_excludes_flagged_samples_from_gr_normalize_percentiles`.)
- [x] sw_height takes an optional TVD input (defaults to MD) and accepts a negative-TVDSS FWL, so
      deviated-well SWH is no longer optimistic. (satheight.rs; negative-TVDSS test.)
- [x] Pay summary: sample thickness clamped to zone overlap (net ≤ gross, no step bleed past base),
      SAND avg_phie normalised over valid-PHIE thickness, PERM-missing excluded. (workflow.rs; test
      `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`.)
- [x] LAS/DLIS import: ~W NULL now parsed (`declared_null`), multi-word well name fixed, DLIS sentinels
      screened + per-frame run numbers so a frame-0 channel no longer silently overwrites a same-mnemonic
      LAS curve. **#118:** `parse_las_2` falls back to column 0 for TDEP/MD/other-indexed files, and
      `sanitize_curve_columns` drops non-finite + duplicate depths (first kept) so a spliced/repeat-depth
      LAS imports instead of aborting on the (well_id, depth) PK. The *generic* store is sanitized the
      same way via `sanitize_las_frame`; an all-null-depth file errors instead of committing an empty
      orphan well; MD/TDEP dropped from the alias list so an auxiliary track can't steal depth; ±0.0
      normalized in the dedup key; non-monotonic depth surfaced as a warning. (parsers.rs, ingest.rs;
      tests `duplicate_depth_las_imports_standard_and_generic_curves`,
      `all_null_depth_las_errors_without_creating_well`, `parse_las_2_auxiliary_md_curve_does_not_steal_depth`,
      `sanitize_dedups_signed_zero_depths`, `parse_las_2_tdep_index_populates_depth`.)
- [x] All delete-then-append writers wrapped in a `with_txn` BEGIN/COMMIT/ROLLBACK (db.rs; applied
      across equations.rs, db.rs, ingest.rs).
- [x] sw_imts clay term now divides by Sw (Sw^(n*−1) Waxman-Smits tail) so it credits clay conductivity
      in pay. (lrlc.rs; test `imts_credits_clay_conductivity_in_pay_zone`; memory corrected.)
- [x] Computed-curve lookup is case-insensitive (fixed 2026-07-20).
- [x] SandiMin refuses < (components−1) input tools up front and skips under-live samples. (multimin2.rs;
      test `rejects_underdetermined_request`.) **#118:** an all-zero CT/CXO response row is now also
      rejected (test `rejects_all_zero_conductivity_row`).

**Reliability (was "P1") — frontend state** — done & adversarially verified (6/0 + second-pass clean;
REVIEW.md P1 section):
- [x] Plots subscribe to dataVersion — histogram/crossplot/pickett/correlation/log-view now
      reload(preserveView=true) on a dataVersion bump (dataPrimed-guarded); stale curves after a module
      run are gone. (2026-07-20.)
- [x] loadWell/reload/createPlot race guards (generation tokens) + a sticky reset flag
      (resetPending / viewResetPending) so a superseded reset intent still fires once; listener leaks on
      dispose fixed (logViewPanel init disposes the LOCAL renderer since dispose() nulls the field;
      LogCanvasRenderer removes its window pointerup/pointermove handlers and cancels the rAF loop via a
      cleanups array). (2026-07-20.)
- [x] Undo: failed undo/redo no longer vanishes silently; "Add top" overwrite undo restores the previous
      depth instead of deleting the top (fixed 2026-07-20).
- [x] Modal Escape-key stacking sliver — ✅ **shipped 2026-07-20** (Escape scoped to the top dialog; see §B1 / REVIEW "P1").

## A10. Feature waves shipped — Wave A + Wave E (§4c)

Source: Jauhar's 2026-07-20 feature list, researched by a 10-agent sweep over his reference library,
real project data, the the reference install install, and the codebase. **Full method specs are banked in
`docs/research_2026-07/` — read the matching file before implementing any Part B/C wave item.** Item
numbers = Jauhar's original ordering. **Per-increment verification standard: run on Balam South data
(item 11)** in addition to cargo test / tsc / browser checks.

**Wave A — UI foundation (done first so every new suite is born into it):**
- [x] **(14) Tools as panes, not popups + theme compliance.** _(Done 2026-07-20)_ **moduleDialog is
      now a dock pane** (component "module", id "module:<name>", spec via listModules so layout restore
      rebuilds from the id alone → EVERY current/future manifest module gets a pane automatically), plus
      a `wellPane` host for **zones, autocorr, composite, report**. Ribbon buttons + the ＋ menu + the
      log-view Print/export item all open panes; "select a well first" guards became in-pane hints. Kept
      modal on purpose: layoutProps, curveEdit, session/layout/header/shift-core, imports. Theme fixes:
      killed phantom CSS vars, re-skinned `.cursor-readout`/`.workflow-invalid`, replaced hard-coded hex
      in crossplot/pickett/histogram/logView TS with `plotColors()`. Adversarial review (16 agents → hand
      re-verify on Opus): 9 real → all fixed. REVIEW #24. → `code_ui_shell.md`.
- [x] **(4) Compact import ribbon.** _(Done 2026-07-20)_ Import Logs ▾ / Import Data ▾ / Export ▾ /
      Tools ▾ via `buildRibbonDropdown`, i18n'd; Manage group kept. → `code_data_db_import.md`.
- [x] **(12) Multi-line inspector in workflows.** _(Done 2026-07-20)_ Workflow Builder List|Grid toggle:
      rows = steps, columns = union of args, Set-all row edits a shared parameter (RW across sw_*) on every
      step that takes it; per-step delete-if-default + manifest-limit validation preserved; view choice
      persisted. → `code_compute_ml_mc.md`.
- [x] **(2) Project open/switch, IP style.** _(Done 2026-07-20)_ `project.rs` recents in
      %APPDATA%\SandiBumi, live connection swap, open/new/recent UI, reopen-last at startup, chain-running
      guard; save_project_as stays a backup copy. → `code_data_db_import.md`.

**Wave E — KKT ONWJ additions (Jauhar 2026-07-20; sources = his KKT ONWJ full-field deck +
Multimin Parameters.xlsx, extracted into `ref_kkt_onwj_wave_e.md` — client files stay OUT of the repo):**
- [x] **(17) Pre-calculation module** (deck slide 31): mud + temp/pressure gradients → FTEMP, FPRESS,
      RMF (Arps to formation temp), CT = 1000/RT, CXO = 1000/RXO. _(DONE 2026-07-20: `precalc` module —
      linear FTEMP (degC canonical) + FTEMP_F / FPRESS in TVDSS with whole-curve DEPTH fallback, RMF via
      ARPS (shared `multimin2::arps_f`) or TREND log10 regression, CT/CXO mmho/m QC curves with R ≤ 0
      guards; four unit tests. Adversarial review 6/6 → fixed: RES_DEEP default, FTEMP degC-only, own
      SURF_TEMP/TEMP_GRAD names, TREND ft-fit + CT/CXO-are-QC caveats.)_
- [x] **(18) Wet→dry clay endpoint conversion** for the PHIT-basis Sw model: φ_clay = (ρdry−ρwet)/(ρdry−1);
      NPHI/GR/DT rescaled onto the dry fraction. _(DONE 2026-07-20: `dry_clay_calc` + `fluid_from_precalc`
      in multimin2.rs, converter panel + precalc autofill in the SandiMin pane. CBW bookkeeping settled
      against deck slide 59 (SWB = VOL_UBNDWAT/PHIT); CEC_eq inverts the BNDWAT multiplier so the solver
      enforces v_bw = φ/(1−φ)·v_dryclay with no solver change. Unphysical-pick guards. Adversarial review
      15 → 7 confirmed → all fixed. 122 tests. REVIEW #22.)_
- [x] **(19) Gas correction, iterated** (deck slide 65): ρb_corr = RHOB + Φt·(1−Sw)·(1.00−ρg_res), outer
      loop solve→correct→re-solve until |ΔΦt| converges. Depends on (17). _(Done 2026-07-20: `gascorr`
      Prep module — Standing+Papay GASDEN, Archie in-loop Sw, 20-pass fixed point, rw_args merged with
      required precalc FTEMP/FPRESS (computed-only). Adversarial review 13 → all fixed. NPHI analog
      deliberately not built (doc steers RHOB_GC → phi_den/PHIT_GC). 127 tests. REVIEW #23.)_
- [x] **(20) φmax porosity cap from compaction trend** (deck slide 64): optional zone-overridable
      PHI_MAX in porosity modules + SandiMin. _(DONE 2026-07-20: `phimax` Porosity module — MODE
      constant/linear/athy; φmax = PHIMAX0 or PHIMAX0 − GRAD·(TVDSS−REF)/1000 or PHIMAX0·exp(−ATHY_K·…);
      TVDSS positive-down, whole-curve DEPTH fallback; all 4 params zone-overridable. Outputs <PHI>_CAP +
      <PHI>_MAX. Jauhar chose the TVDSS trend (linear + athy shipped). Ultracode review 0 confirmed / 4
      refuted, +2 guards. 136 tests. REVIEW #26.)_
- [x] **(21) Cutoff sensitivity tools** (deck slides 84–87): Method-1 pay-sensitivity sweep plots +
      Method-2 DST-highlighted crossplots with draggable cutoff crosshairs; picked cutoffs write per-zone
      pay-summary defaults. _(DONE 2026-07-20: one "cutoff" dock pane, Sweep / DST-Crossplot toggle.
      `compute_sweep`/`run_cutoff_sweep` reuse the pay-summary math via extracted `classify_sample`
      (byte-identical, tested); NET/HPV/NTG; zone + DST-interval filter. Frontend: per-well sweep lines
      with pick-and-write, DST crossplot with draggable crosshair, save-as-default. Two review passes:
      13 → 10 confirmed, all fixed. 131 tests. REVIEW #25.)_ Per-zone cutoff APPLICATION inside
      run_pay_summary remains a noted follow-on.
- [x] **(22) Map pane + editable polygons → well groups**: well header surface X/Y, Map pane, polygon
      draw/edit, point-in-polygon → well group. _(DONE 2026-07-20: `wells` gained surface_x/surface_y
      (DOUBLE — S-hemisphere northings exceed f32) + utm_zone; `geo.rs` point_in_polygon (PNPOLY) +
      wells_in_polygon; `parse_locations_file` + `import_locations_file`. Frontend: standalone Field Map
      pane (pan/wheel-zoom/grid/scale-bar, draggable polygon vertices, live highlight, Assign→group),
      Import Well Locations dialog with Indonesia UTM zone (46–54 N/S), Well Header X/Y/zone. Raw UTM (no
      reprojection — multi-zone follow-on). Adversarial review: 3 confirmed, all fixed. 143 tests. REVIEW #27.)_
- [x] **(23) Data-conditioning flags module** (mid-Wave-E request): flag badhole / tight / gas-crossover
      / coal + a shoulder-adjustment flag. _(DONE 2026-07-20: `condflag` Prep module — COAL_FLAG
      (density/neutron/sonic, BADHOLE-excluded), TIGHT_FLAG, XOVER_FLAG, SHOULDER_FLAG (dilation of
      coal/tight + ≥MIN_THICK badhole), COND_FLAG combined mask; MIN_THICK despike with NaN-bridged runs;
      8 tests. Adversarial review 12 → 8 confirmed → fixed: RHO_MA/RHO_FL shared with porosity modules,
      NaN-in-bed despike bridging, degenerate-guard, doc caveats, BADHOLE/COND_FLAG always in Mask
      dropdowns.)_
- [x] **(24) Neutron matrix conversion module** (mid-Wave-E request): convert NPHI between limestone /
      sandstone / dolomite conventions. _(DONE 2026-07-20: `nphimat` Prep module — chartbook Por-5 (CNL
      thermal) and Por-4 (epithermal APS + legacy SNP) digitized at vector precision into
      `neutron_charts.rs` (12 tables, generator `tools/chartdig/gen_por45.mjs` with hard gates). Pivots
      through the apparent-limestone axis, outputs NPHI_LS/NPHI_SS/NPHI_DOL; 6 tests. Adversarial review
      13 → 8 confirmed → fixed: APS/legacy mnemonics added to aliases + family; all-NaN standard column in
      fetch_curve_frame falls back to computed/generic (APS wells no longer all-NaN); workflow-builder
      log_in dropdowns offer every module's log_out; SNP no longer mislabeled as APS.)_

**Import data-management wave (from Jauhar's manual-test-plan T-IMP notes, 2026-07-30):**
- [x] **Example import datasets as executable fixtures.** _(Done 2026-07-30)_ `dataset for
      test/examples/` — one exemplar per accepted format (3 synthetic wells with coupled
      geology, core RCAL, 3 SCAL shapes, tops, deviation, locations, petrography, XRD,
      perforations) + 2 deliberately malformed LAS closing T-IMP-03/-04 ("where do u provide
      dup_depth.las?"). Generated by `tools/make_example_data.py`; parsed by
      `example_data_test.rs` on every gate run so examples can't drift from the parsers.
      Also fixed: `GDEN` core alias — the BLSO delivery's `GDEN_1` silently dropped grain density.
- [x] **(T-IMP-02) Import sets — one well, many deliveries.** _(Done 2026-07-30)_ The
      Geolog/IP set model. `LasImportOptions{set_name, attach}` + `canonical_set_name` /
      `resolve_set_name` (auto-suffix `FPROOH` → `FPROOH_1`, never overwrite);
      `attach_curves_to_existing_well` writes only the generic store, so an attached delivery
      never disturbs the first one's `standard_curves`. Ambiguous (>1 same-named well) falls
      back to a separate record WITH a warning. `importSetDialog.ts` suggests the set name from
      the filenames' common non-leading token (verified against all five BLSO folders);
      `objectTree.ts` grew a lazy well → sets → curves twisty. DLIS import takes a set name too
      (T-IMP-06 duplicates). **Resolution compatibility: set RAW keeps ABSOLUTE priority** in
      `fetch_generic_curve_aligned`, so every existing project resolves byte-identically and
      only mnemonics RAW lacks reach the attached sets.
- [x] **(T-IMP-07/-10/-11) Core & aux import v2 — auto-detect well names.** _(Done 2026-07-30)_
      Import Core is probe → confirm → commit: `parsers::probe_core_table` (role guesses incl.
      a WELL/WN column with prefer-textual-over-numeric candidate choice, per-column type
      sniff, sample rows, distinct wells, percent + depth-unit detection from units row /
      header suffix) + `ingest::import_core_table` (routes rows per normalized well name —
      exactly-one-match rule, unmatched/ambiguous reported never guessed; blank cells skipped;
      feet↔metres conversion to the project unit; per-well replace + depth dedup).
      `coreImportDialog.ts` wizard confirms the mapping ONCE by header name and re-resolves it
      per file (multi-select of e.g. all 321 BLSO core CSVs); `.txt`/tab/semicolon sniffed by
      `read_delimited`. Aux imports (`import_aux_file`) now route by a WELL column the same
      way. Unblocks T-IMP-09. Per Jauhar (2026-07-30): **BLSO is an exemplar, not the spec —
      importer must accept ANY delimited text with mixed column types (alpha/int/real/…);**
      delimiter sniffing + type detection are in, see the follow-up below.
- [x] **Core files' EXTRA columns → point-data store.** _(Done 2026-07-30)_ Completes Jauhar's
      "any column, any data type" requirement. `CoreMapping.extras` (serde-default, so older
      payloads still deserialize) carries the columns no core role claims;
      `parse_core_table_mapped` returns them as RAW TEXT plus their header names
      (`MappedCoreTable`), and `import_core_table(..., extras_dataset)` writes them to
      `aux_data` at the same converted plug depths — typed PER CELL (`parse::<f32>` → value_num,
      else value_text), blank cells skipped, riding the same depth-dedup as the plugs so they
      stay aligned. Default dataset "CORE"; replace-on-reimport per (well, dataset). The wizard
      shows the leftover columns with their sniffed type, opt-in, checked-state remembered by
      header NAME across role changes (a column claimed by a role leaves the list). Values are
      stored VERBATIM — no percent/unit conversion is applied to extras, and the dialog says so.
- [x] **(T-IMP-05) Silent import guards** — **done 2026-07-31.** Core was already covered. Every
      remaining action that needs a selected well refused only in the STATUS BAR: the user picked
      "Import SCAL…", expected a file dialog, and got nothing, with the reason in a corner nobody
      was looking at. "Nothing happened" is indistinguishable from a broken button, and the usual
      next move is to click it again. `src/ui/needWell.ts` `requireWell(action)` is now the one
      refusal — a named modal that says which action, why, and what to do — used by Export LAS,
      Import DLIS, Import SCAL, Import deviation, Import Aux, Import pictures, Data Sets, Shift
      Core and Well header. The status line still receives the message; it just is not the only
      place it appears. One helper rather than nine copies, the `followCore.ts` argument.
- [x] **(T-IMP-08/-12) Duplicate/versioning for core + deviation data.** _(Done 2026-07-30)_
      Core plugs and surveys now follow the set model, with a deliberately DIFFERENT resolution
      rule from curves: two curve sets can both be read (a set supplies mnemonics RAW lacks),
      but two core deliveries measure the SAME plugs, so **exactly one core set and one survey
      are ACTIVE per well** and every reader follows it (`db::ACTIVE_CORE_SET` /
      `ACTIVE_SURVEY`, one shared SQL fragment so no reader can silently union two deliveries
      and double a φ-k cloud). Schema: `core_data.set_name` + `core_sets` registry,
      `well_path.survey_name` + `well_surveys` registry (active/source/datum/imported_at);
      `db::migrate_point_data_sets` rebuilds pre-set-era projects (existing rows become
      RAW, active — same numbers as before), idempotent, backed up per RELEASE §3.2. Imports
      take a name (Core wizard suggests it from the filename, deviation dialog defaults
      SURVEY), auto-suffix per well rather than overwriting, and the new set/survey goes live;
      the status line names the set and says which wells were suffixed. `set_active_survey`
      re-materializes TVD/TVDSS so stored curves never keep the previous geometry. UI =
      `dataSetsDialog.ts` (Data → Tools ▾ → Data Sets…): both lists, ● active, Use /
      Delete, delete-the-active hands over to the newest survivor. Shift Core and DB Inspector
      cell edits target the ACTIVE set only.
- [x] **Delivery sets are UNIVERSAL — every point dataset, not just core.** _(Done 2026-07-30,
      Jauhar: "not only core, any kind of point data should behave universally like core — xrd,
      cec, oil show, etc.")_ `aux_data` gained `set_name` (+ `aux_sets` registry keyed
      (well, dataset, set)), so a second XRD / CEC / oil-show / petrography / perforation
      delivery lands beside the first and one set per (well, dataset) is live —
      `db::ACTIVE_AUX_SET` correlates on `a.dataset`, so a single query spans every dataset and
      still returns one delivery of each (`list_aux_datasets` counts the active set, never the
      sum). `aux_data` has no PK, so migration is an ALTER + back-fill + registration rather than
      a rebuild; the column is LAST because the Appender is positional. Import Aux takes a Set
      name; core EXTRAS are written under the core set's own name so a core switch carries them.
      The Wells-pane ▸ tree now lists **Core / Surveys / Point data** under each well with ● on
      the live one and double-click to switch (single click inert; delete stays in the dialog),
      and the manager dialog has a third section grouped by dataset.
- [x] **SCAL Pc deliveries too — the last store that replaced wholesale.** _(Done 2026-07-30)_
      `scal_pc.set_name` (+ `scal_sets` registry, `db::ACTIVE_SCAL_SET`): the files selected in
      one Import SCAL are ONE delivery, named and auto-suffixed rather than overwriting the
      previous report, and `get_scal_pc` — hence Pc QC, the Leverett-J fit and Thomeer — reads
      only the live one. Migration is an ALTER + back-fill + registration (no PK). The manager
      dialog is now **Data Sets…** with four sections (core / SCAL / surveys / point data) and
      the tree gained a SCAL kind. Every delivery-shaped store in the app now versions the same
      way; nothing left that silently overwrites on re-import.

Cross-cutting notes: (11) Balam South testing is the per-increment verification standard, not a separate
item. New suites must land as panes (Wave A first), use the 15-var theme contract, manifest-driven
dialogs where they fit, and expose outputs to Python/SQL per §5.

---
---

# ◻ PART B · OPEN (do next)

The actionable backlog. Roughly ordered: safe frontend wins first, then Performance (which needs a live
100-well run to sign off), then the Wave B feature suites, then carried-forward deferrals.

## B1. Hardening backlog (§4b)

**Correctness — OPEN, awaiting Jauhar's method decision (found 2026-07-31)**

- [ ] **A per-zone TEMP_GRAD override makes a STEP in FTEMP, not a kink.** `precalc` computes
      every sample as `SURF_TEMP + gradient(sample) × depth(sample)` — the gradient is applied
      **from surface**, never integrated down through the zones above it. So the moment a lower
      zone carries its own gradient, the temperature profile is discontinuous at the boundary.
      Measured on a 0.03 °C/m well with 0.035 below 1500 m: **67.0 °C at 1400 m, 77.5 °C at
      1500 m — a 10.5 °C jump where the undisturbed trend rises 3.0.** Rock temperature is
      continuous, so the profile is not physical, and it does not stay in FTEMP: the Arps
      correction turns temperature into Rw, and Rw goes into Sw. T-PREP-05's own expected result
      says the trend should *kink* with "no discontinuity artifacts", so the plan and the code
      disagree and the plan describes the physical answer.

      **Pinned as-is, not fixed** — `a_per_zone_gradient_override_reaches_exactly_its_own_samples`
      (`workflow.rs`) asserts the 10.5 °C step explicitly, so it cannot drift or be changed
      silently. The fix is method math, not a refactor: integrating per zone means choosing what
      temperature each zone *starts* at (carry the zone above's value down, or re-anchor on
      surface), and that needs a cited source. Same question applies to PGRAD, which is computed
      the same way. Logged in `docs/review_triage.md` finding 6 and in T-PREP-05's known-issue line.

- [ ] **A well with NO permeability is EXEMPTED from an active PERM cutoff.** `classify_sample`
      correctly fails a *sample* whose PERM is missing, and there is a confirmed `[x]` for that.
      But `run_pay_summary` decides whether the cutoff runs at all per WELL —
      `perm_min.is_some() && perm.iter().any(|v| !v.is_nan())` — so a well carrying no permeability
      anywhere switches it off for itself. Measured on two wells of identical rock at PERM ≥ 1000:
      **the well that measured 1 mD reported net 0; the well that measured nothing reported all of
      it.** The less data a well has, the more pay it books, and `n_classified` is non-zero for
      both, so no consumer downstream — dashboard, workbook, PDF — can tell them apart.

      **Pinned as-is, not fixed** — `a_well_with_no_perm_at_all_quietly_escapes_an_active_perm_cutoff`
      (`workflow.rs`). Exclude or exempt is a petrophysical decision that changes reserves.
      `docs/review_triage.md` finding 7; T-BATCH-08 carries the known-issue line.

- [ ] **Adding a permeability model to a Monte Carlo chain switches off the permeability cutoff.**
      Already in AUDIT-2026-07-21, but the trigger is broader than recorded there. PERM reaches
      `has_perm_cut` only if a step CONSUMES it and none PRODUCES it, so the cutoff works on a
      chain ending in Rock Typing and goes dead the moment `perm_coates` is inserted ahead of it —
      which is exactly the study whose permeability cutoff matters. Pinned as-is by
      `adding_a_permeability_model_to_a_chain_switches_off_the_permeability_cutoff`
      (`montecarlo.rs`), with the working chain beside it as the control. Finding 8.

- [ ] **Pittman PR75 exceeds PR50 above about 79 mD.** Mercury enters the widest throats first, so
      the 75 % radius must be the smaller one. The nine rows are independent regressions, and
      `PR50 − PR75 = −0.634 − 0.066·log k + 0.543·log φ%` changes sign in ordinary good sand.
      Measured at φ = 25 %, k = 100 mD: **PR50 2.907 µm against PR75 2.953 µm.** PR10–PR50 stay
      monotone, which narrows it to the PR75 row. `pittman_rx_spec`'s own doc already flags the
      table *verify before field release* — this is that verification failing. **Not fixed:
      correcting a published coefficient needs Pittman 1992 in hand, and inventing one to make the
      ordering come out right is precisely what the provenance rules forbid.** Pinned by
      `the_pittman_radius_family_inverts_between_r50_and_r75_in_good_sand`. Finding 9.

- [ ] **A failed run still writes its empty curves into the Curve Catalog.** Phase 2 of
      `run_workflow_module_into` writes for any well whose outcome is `Computed` with a non-empty
      output map, and an all-MISSING map is still non-empty. rocktyping on a well with porosity but
      no permeability reports its error **and versions all eight outputs** as curves blank from top
      to bottom. The values are honestly MISSING, so nothing is corrupted — but the catalog stops
      distinguishing "never run" from "run and could not answer", and a log-set version is spent
      recording the second as an interpretation. Pinned both halves by
      `rocktyping_without_a_permeability_curve_fails_and_writes_no_curves` (`workflow.rs`).
      Suppressing the write is one filter, but it changes behaviour for every module. Finding 10.

- [ ] **Two wells with one name overwrite each other's report, and the batch count says
      otherwise.** `export_report_batch` builds each filename from the well NAME with every
      non-alphanumeric mapped to `_`, and `well_name` has no uniqueness constraint — an import with
      attach OFF creates a duplicate by design, and `SANDI/1` / `SANDI 1` collide as well. The
      second write silently replaces the first while BOTH paths are pushed onto `written`, so a
      3-well batch reports "wrote 3 file(s)" over 2 files and the surviving report carries the
      wrong well's name. The same function identifies wells two ways — the success path looks the
      name up for the filename, the failure path reports the raw UUID — which is the same gap seen
      from the other side. Pinned as-is by
      `two_wells_with_one_name_silently_overwrite_each_others_report` (`report.rs`). **Not fixed:
      suffixing the duplicate or falling back to the well id both change what lands in a client
      folder.** Finding 12.

- [ ] **A Rhai equation that raises on only SOME samples reports a clean success.** A Rhai error is
      caught per sample and written as MISSING, and the all-MISSING guard — the only thing that
      turns a script error into a reported failure — fires only when EVERY sample failed. So a
      half-raising script yields a curve with holes and the full row count, indistinguishable from
      a curve whose inputs were absent there, which is the ordinary innocent case. Same shape as
      finding 10: the honest signal exists but is gated on the failure being total. Pinned as-is by
      `a_script_that_raises_on_only_some_samples_still_reports_a_clean_success` (`equations.rs`),
      with a control that raises everywhere and IS caught. The Python path is unaffected — it runs
      the whole well's array at once, verified by
      `a_python_raise_in_one_well_leaves_the_rest_of_the_batch_intact`. **Not fixed: counting the
      raises changes the run summary, and whether a partially failed curve should be written at all
      is a judgement about how the equation editor is used.** Finding 13.

- [x] **Legacy-multimin RECON_ERR at 3 tools — CLOSED 2026-07-31, no sign-off needed.** REVIEW.md
      still lists this among the findings awaiting a decision because it would change
      interpretation numbers. It does not need one. Legacy `multimin` is **retired** — `run_module`
      blocks it, the solver body is gone — so that instance cannot occur. The concern was inherited
      by SandiMin and is inherent rather than fixable: with as many equations as components the
      solve reproduces the logs exactly whatever the endpoints are, so the residual carries no
      information about them. SandiMin **detects** it (`dof == 0`) and returns `dof_note` telling
      the user RECON is forced to ~0 and to add an input log. Measured on one well, one set of
      logs, correct endpoints throughout: **dof 0 → RECON ~0.00; dof 2 → RECON 0.62**; with a
      0.4 g/cc endpoint error the square case still reports ~0 while the clay volume moves, and the
      dof-2 case goes 0.62 → 1.22. Pinned by
      `an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so`
      (`multimin2.rs`). **Remaining work is bookkeeping plus one UI judgement:** drop the item from
      REVIEW.md's sign-off list, and check during click-through that the SandiMin pane makes the
      note hard to miss. `docs/review_triage.md` finding 11; T-RT-18 carries a superseded block.

**Performance (was "P2") — speed at field scale (100+ wells)** — all 6 mapped by a read-only
investigation wave (file:line + risk). **5 of 6 shipped: #127 (crossplot memoize), #128 (long
commands off the event loop), #130 (batch curve reads), #131 (raw-IPC ArrayBuffers), #132
(persistent Python worker).** Tasks #127–132. Only **#129 (connection pool)** remains — it changes
DB connection semantics and **cannot be signed off without running `tauri dev` on 100+ real wells**
(the human can't be replaced for perf benchmarking).

- [x] **Startup opened the project before the window existed** — **done 2026-07-30.** `run()` now
      builds the Tauri app on an EMPTY in-memory placeholder database and `setup()` spawns
      `open_startup_project` on a background thread (the whole recovery ladder — project → temp
      recovery file → memory-only — moved there unchanged). It publishes an `OpenOutcome` through
      a `Mutex<Option<_>>` + `Condvar` (`DbInit`), which the new async `await_project_open`
      command waits on off the event loop. **The value is STORED, not merely signalled** — on a
      normal fast launch the open finishes before the frontend ever asks, so a pure signal would
      hang every quick launch on the splash (pinned by `fast_open_published_before_the_wait`).
      Frontend: `bootOverlay.ts` covers the wait (shown only after 400 ms so a fast open never
      flashes it; elapsed timer; polls `boot_report` for live progress; a "this happens once"
      hint after 20 s), and `main.ts` awaits the gate before building ANY panel — that ordering is
      what keeps every command off the placeholder. Notes drained by the overlay are handed back
      and written to the processing history once the database that stores it is open.
- [x] **(#128)** ~~Long commands are synchronous Tauri commands~~ — **done 2026-07-30.** The three
      commands this item named were already fixed when it was written: `run_ml`/`run_multimin` run
      through `jobs::run_job` (async) and `run_workflow_chain` returns immediately after spawning an
      OS thread (a plain `std::thread`, NOT `spawn_blocking` — a sync command is not on a Tokio
      worker, so `spawn_blocking` panics there; see the comment at its definition). A re-audit of
      **every** sync `#[tauri::command]` found the class of bug alive in the project-lifecycle and
      whole-field commands instead, all now `async` + `tauri::async_runtime::spawn_blocking`:
      `open_project`, `new_project` (a field-scale open runs one-time migrations — ~15 min on the
      2.5 GB BLSO project, the entire time with the window frozen), `save_project_as` +
      `compact_project` (gigabyte engine copies), `materialize_tvd` (every selected well) and
      `run_query` (user-authored SQL = unbounded cost). Concurrency is unchanged where it matters:
      the DB `Mutex` still serializes, so nothing observes a half-swapped project.
      The STARTUP open (pre-window) was the remaining gap and is **also done 2026-07-30** — see
      the startup item below.
- [ ] **(#129) [HIGH-RISK]** Rayon over wells is defeated by the single global `Mutex<Connection>` — every
      well locks the same conn. Split reads (read-only connection pool) from the single serialized writer;
      writes (computed_curves DELETE+append in `with_txn`) **must stay single-writer** to protect the
      WAL/131MB file. Corruption modes must be reasoned explicitly.
- [x] **(#131)** ~~"Binary" curve IPC ships bytes as JSON numbers (~4× size, main-thread parse)~~ —
      **done + committed 2026-07-21.** The three curve-data commands
      (`get_track_data`/`get_curve_data`/`get_core_data`) now return ONE length-prefixed binary buffer
      via `tauri::ipc::Response` (→ JS `ArrayBuffer`) instead of a JSON-serialized
      `Vec<TrackCurveSeries>` whose `data: Vec<u8>` serde-encoded as a number array. New
      `equations::pack_curve_series` (`[u32 count]{[u32 name_len][name][u32 pc][f32 depth×pc][f32
      value×pc]}…`, LE) + frontend `decodeCurveBuffer` replacing `unpackCurveSeries`; the `ipc.ts`
      wrapper output (`TrackCurveSeries[]` with depth/value Float32Arrays) is UNCHANGED, so no
      plot/log-view consumer changed. +1 Rust roundtrip test + a Node format-agreement check (incl. the
      non-4-aligned data offset after an odd-length name + NaN survival). cargo 168 / tsc 0. The runtime
      transport (invoke → ArrayBuffer) + the real byte-size/parse win need Jauhar's live run.
- [x] **(#130)** ~~One query per curve per well load (~100 scans of computed_curves)~~ — **done
      2026-07-21.** `fetch_curve_frame` now defers every non-standard name into ONE
      `SELECT upper(curve_name), depth, value FROM computed_curves WHERE well_id=? AND upper(curve_name)
      IN (?..)`, buckets rows per curve + aligns them in Rust, and keeps the computed-then-generic
      (RAW mnemonic/family alias) precedence byte-for-byte. Two new `equations::` tests pin the
      semantics (standard passthrough, case-insensitive match, off-grid→NaN, absent→generic fallback).
      `cargo test --lib` = 166 pass / 0 fail. Speedup is unmeasured pending a live 100-well run.
      **Committed 2026-07-21.**
- [x] **(#132)** ~~Python engine spawns one subprocess per well (re-importing numpy each time)~~ —
      **done 2026-07-21.** The numpy equation engine now runs ONE persistent worker process
      (JSON-header + raw-f32 request/response loop over stdin/stdout), reused for every well and every
      run, instead of spawning `python.exe` per well. Per-request script-error isolation (fresh namespace
      each request; a bad script is reported without killing the worker) + respawn-and-retry on a dead
      worker; `run_python_equation` is now sequential over the shared worker. +1 test
      (`worker_survives_a_script_error`). `ml.rs` was already single-spawn-per-run, so left as-is.
- [x] **(#127)** Crossplot redraw rebuilds per-point rgb() color strings + re-sorts on every frame —
      **done 2026-07-20.** Extracted the color computation into pure `computeCrossplotColors()` and
      **memoized** it in the panel, keyed by (Z curve, colormap, log-Z, fixed color, data-generation);
      the two `percentile` sorts + N-length color array now run once per data/setting change instead of
      per pan/zoom/hover frame — only the viewport transform + scatter draw stay per-frame. Output
      pixel-identical (speed-only). tsc clean; REVIEW.md "Performance". **The one pure-frontend,
      low-risk Performance win — shipped.**

**Polish (was "P3") — UX (veteran-interpreter friction):**
- [x] Depth-scale presets mislabeled (~39× off a true 1:100 — fixed 2026-07-20).
- [x] **(Polish-1, #122)** Units on readouts + adaptive value formatting — `formatValue()`/`loadCurveUnits()`
      in plotCommon; the cursor readout keeps small-value resolution (perm 0.003, not 0.00), trims big
      values (RT → 2151), appends the catalog unit; cached per-load, refreshed on dataVersion. tsc clean.
      (2026-07-20; REVIEW.md "Polish — UX".)
- [x] **(Polish-2, #123)** Correlation stale well list + missing Ctrl+wheel zoom —
      `correlationPanel.refreshWells()` re-fetches the well list on dataVersion (imports appear, deletes
      drop, group re-applies) + Ctrl/Cmd+wheel zoom about the cursor (matching attachZoomPan). tsc clean.
      (2026-07-20; REVIEW.md.)
- [x] **(Polish-3, #124)** Processing history now audits equation/chain/ML/MC runs, log-set
      restore/delete, zone add/edit/delete + params, manual tops edits, DLIS/deviation/SCAL/core
      imports, cutoff saves, map polygon→group — exhaustive `recordProcess()` sweep. (2026-07-20.)
- [x] **(Polish-4, #125)** Pickett v2 — ⚙/right-click Properties dialog (configurable RT/PHIE axes,
      point size, color-by-curve rainbow/viridis+logZ) persisted via `plotprops`; toolbar M/Rw fields
      (typed line follows, picks fill the same fields). Absorbs the §4 Interpretation-workflow "Pickett
      v2" item. (2026-07-20.)
- [x] **(Polish-5, #126) [backend]** Pay-summary FLAG_* curves now versioned into a **PAYFLAG** log set
      with provenance (module `pay_summary` + the cutoffs in `log_sets.params_json` + inputs) on the
      explicit run; Dashboard/report set `skip_version` to overwrite in place (no churn). Atomic via
      `with_txn`; test `pay_summary_versions_flags_with_cutoffs_in_provenance`. (2026-07-20.)

**Reliability sliver (was "P1") — ✅ CLOSED 2026-07-20:**
- [x] Modal Escape-key stacking (overlapping dialogs share one Escape handler) — **done 2026-07-20**
      (modal.ts). The listener-leak half was already handled by `openModal`'s single-instance
      `activeClose` (a new dialog closes the prior + removes its keydown listener; no modal nests, so
      no stack needed). Closed the remaining gap: the dialog's `document`-level Escape now
      `stopPropagation`s so one Escape no longer also fires window/app-level Escape handlers (e.g.
      cancelling an in-progress map polygon while a dialog is open) — kept on the **bubble** phase so
      the numeric-edit guard's capture-phase stop still shields a dialog from closing mid-number-edit.
      Also tears down in-flight title-bar drag listeners on close. tsc clean; REVIEW "P1".

**Low (15 items)** — see `AUDIT-2026-07-20.md`. **Fully closed 2026-07-21: #134 applied 10 safe
fixes (1 already fixed); #135 resolved the 4 held items per Jauhar (below). Committed — #134 in
`2c1f67b`, #135 in `5bae536`; cargo 166 / tsc 0.**
- [x] SSC `SWIRR_EFF` ÷0 guard (100%-shale no longer reads "all water movable") — `ssc.rs`.
- [x] Archie `SWT_ARCH` no longer writes `+Infinity` at PHIT=0/PHIE-absent — `modules.rs` (+test).
- [x] Simandoux SCHLUMBERGER ÷0 at VSH=1 → all-water — `modules.rs` (+test).
- [x] ±Infinity rejected in stats/regression/percentile/scatter — `plotCanvas.ts`.
- [x] Zone-param "Set" button surfaces write failures — `plotCommon.ts` pickRow.
- [x] Track rename can't create duplicate titles — `layoutPropsDialog.ts`.
- [x] Histogram: constant curves render + honest `n = X of Y` label — `histogramPanel.ts`.
- [x] Log-view smoothness: cached clear-color + binary-search cursor — `LogCanvasRenderer.ts`.
- [x] Well Header prefills current TD/KB (no blind datum edit) — `db.rs`/`ipc.ts`/`ribbon.ts`.
- [x] LAS wrap-mode: truncated row → loud error, no silent column-shift — `parsers.rs` (both).
- [x] DB-inspector edit errors on a 0-row update (no phantom edit/undo) — `db.rs` (+test assertion).
- [x] ~~SandiMin all-zero CT/CXO response-row guard~~ — **already fixed** (`multimin2.rs`, has a test).
- [x] Wyllie lack-of-compaction (Cp) correction — shipped as **opt-in `OPT_CP` (default OFF)** in
      `phi_son` (+test); RHG never Cp-corrected. (#135)
- [x] Depth-scale dropdown shows the **true live scale** + fixed the mislabel/clamp that collapsed
      1:20/1:50/1:100 — `LogCanvasRenderer.ts`/`logViewPanel.ts` (true 1:1 = 96/0.0254 single-sourced,
      opens 1:2000, clamp reaches 1:10). (#135)
- [x] Quiet **Ctrl+S** session re-save (skips text inputs/CodeMirror) + **Escape** closes ribbon
      menus — `ribbon.ts`. (Ctrl+P active-plot print deferred — fragile active-canvas resolution.) (#135)
- [x] i18n: **Bahasa Jawa (jv)** locale added + ~55 common phrases across id/su/jv — `i18n.ts`,
      `index.html`; jargon still English by design. (#135)
- [~] Histogram full-range re-bin — **declined (left as-is)** by Jauhar (would change bar heights /
      the mode-P50 read). (#135)

## B2. Interpretation-workflow open items (§4)

_(field-review tier, was "P2"; the rest of the tier is done in [A8](#a8-field-review--trust--safety--interpretation-workflow-4).)_

### Log-view display queue (Jauhar, 2026-07-30 click-through)

Five display gaps he raised in one pass. Four have shipped; two follow-ups remain — **images in
their own track** (with digitizing later — see §B3) and the ambiguous **QAT-in-ribbon** item,
which he has since clarified (also §B3).

- [x] **Curve draw style — continuous vs blocky** — **SHIPPED 2026-07-30.** `CurveStyle.draw_style`
      (`"line"` default / `"step"`), implemented in both renderers, exposed as a **Style** column in
      Layout Properties. Step holds each sample's value down to the next sample's depth, so a
      block-averaged or zone-constant curve stops drawing a gradient it never measured; the edge
      shading follows the step (rectangles, not wedges).
- [x] **Shading to another log (crossover)** — **SHIPPED 2026-07-30.** `fill: "curve"` + `fill_to` +
      `fill_color2`. The reference must be a curve in the SAME track because it is positioned with
      **its own min/max** — compatible scaling is the meaning of the display. Quads split at the
      crossing point so the two colours meet exactly on it; the viewer interpolates the reference
      onto the styled curve's depths (`makeSampler`) so the two curves need not share a sampling.
      Built-in Standard + Facies layouts now ship the NPHI/RHOB crossover. Fixed in passing: the
      composite exporter treated any `fill` string other than `"right"` as a left-edge fill, so a
      style saved as `fill: "none"` printed shaded while the screen showed it clean.
- [x] **Point / text data as a track type** — **SHIPPED 2026-07-30** (increments a + b together).
      `TrackKind::PointData` + `Track.points: Vec<PointStyle>`; sources = core plug properties
      (ACTIVE core set, NULL cells dropped not read as 0) and aux point-dataset items; displays =
      points / text / box plot / histogram, with per-series depth bin, box percentiles, whisker
      rule (Tukey k·IQR / percentile pair / full range) and show-samples. Both renderers:
      `logViewPanel.drawPointTracks` and `composite.rs draw_point_series`. Off-scale samples are
      skipped, never clamped. **Increment (c) — image display (core photo / borehole image) —
      remains open** and needs a blob store, so it is genuinely separate.
- [x] **Array logs in the log view** — **SHIPPED 2026-07-30.** His answer to the open question was
      "adjustable band, and all of those in ur mention", so all three displays landed over ONE
      stored matrix: `band` (adjustable low/high percentiles + optional P50 line), `spaghetti`,
      `heatmap`. That single-matrix choice is what makes the band genuinely *adjustable* —
      persisting three percentile curves instead would have made changing P10→P5 a re-run.
      `array_logs` became a real store (BLOB of little-endian f32, PK'd, `migrate_array_logs_store`
      drops the never-written stub); `montecarlo.rs` gained `persist_realizations` +
      `realization_cap` (default 256 — the full 1024 would be ~3 GB across a field);
      `TrackKind::ArrayLog` + `ArrayStyle` render in `logViewPanel.drawArrayTracks` and
      `composite.rs draw_array_series`. **The statistics were reused unchanged** —
      `distribution.rs`/`.ts` gained only `band` and `even_indices`, both source-agnostic; no second
      path was written, which was Jauhar's explicit instruction when accepting the point-data
      increment. Verified: a band drawn from the stored matrix reproduces the persisted
      MC_*_LOW/_HIGH curves to 1e-5.
- [x] **QAT tools in the Project ribbon** — **SHIPPED 2026-07-30.** The icon-only quick-access
      strip is gone; its seven buttons are labelled tools in the **Project** tab, grouped Project
      (Open / New / **Save Project As…** / Recent ▾), Session (Save Session… / Open Session…),
      Edit (Undo / Redo), Monitor (History / **Processing** / **Performance**, both moved out of
      the Petrophysics Batch group) and Help. CLAUDE.md/AGENTS.md updated — the "Save Project As
      is NOT a ribbon button anymore" line is explicitly marked as reversed, not silently dropped.
      Two things fell out of the move and were fixed in the same increment:
      **(a)** the unsaved-state dot is now mirrored onto the **Project ribbon tab** as well as the
      Save Session… button — the button moved *inside* a tab, and a warning you only see after
      opening the tab that holds the fix is not a warning. It is a `::after` dot, never a text
      prefix, so the tabstrip never reflows and shifts the other tabs under the cursor.
      **(b)** the Office-style ribbon **overflow chevrons never worked** — latent since they were
      written, and only reachable once a tab was wide enough to need them. Project (1471px of
      groups) is the first, so it surfaced now. `scrollBy({behavior:"smooth"})` is a silent no-op
      on `.ribbon-panel` in the WebView; so is a `scrollLeft` assignment under CSS
      `scroll-behavior: smooth`. Only a plain assignment moves it. `scrollActive` now assigns
      directly, clamped, and refreshes the chevrons synchronously. **Do not "restore" smooth
      scrolling there** — it reverts the fix. Measured after: right → 211 (clamped to max), Help
      scrolls into view, chevrons swap; left → 0.
      Follow-up left open: at a 1280-wide window the Project tab is the only one that overflows.
      That is what the chevrons are for and it is now genuinely usable, but if Jauhar works on a
      1366 laptop and dislikes it, the cheapest fixes are merging Language into the Appearance
      group and folding Help into Monitor (≈100px); anything more means restyling the ribbon
      fields, which is a design change he should call.
- [x] **Images in their own track** — **SHIPPED 2026-07-31** (display half; digitizing stays a
      later phase). His ask: "images in separate tracks, such petrography thin section, core photo,
      or any picture format that can be adjustable (later we should have capablites to digitize it
      as well)". `well_images` + `image_sets` are the blob store, on the universal delivery-set
      rule (`db::ACTIVE_IMAGE_SET`); `TrackKind::Image` + `ImageStyle` render in
      `logViewPanel.drawImageTracks` and `composite.rs draw_image_series`, with the geometry in
      ONE place per side (`imageBox` / `image_box`) so screen and print agree.
      Three decisions worth remembering, all petrophysical rather than cosmetic:
      **(a)** `depth_base IS NULL` means a POINT sample — a thin section is cut from one plug and
      has no thickness, so it is anchored at its depth instead of being stretched over a guessed
      interval; a core photograph with a base depth spans it for real. That is what
      `mode: anchor|depth` selects.
      **(b)** Two plates that would overlap: the deeper one is **skipped and leaves a tick**, never
      nudged — a plate moved to make room is a plate attributed to the wrong sand. Zooming in
      brings it back. (Flagged in REVIEW in case he would rather they stacked.)
      **(c)** Aspect ratio is never distorted — `fit` is contain or cover, and there is no stretch
      option, because a squashed thin section misstates grain shape.
      Storage: the stored blob is a **normalized display JPEG** (one Pillow subprocess for the
      whole delivery, long edge 2400 px / q85 by default and adjustable in the wizard), with
      `source_path` + the original's pixel size kept so the delivered file stays traceable. Without
      Pillow the import still works for anything the WebView decodes, stored verbatim; TIFF needs
      Pillow and says so by name. Import is probe → confirm → commit like the core wizard, with
      `parse_depth_from_name` guessing a depth per file (a token needs a decimal point or ≥3
      integer digits, so `BLSO-01` is never read as 1 m) and every guess editable before the write.
      Print: `assemble_pdf_with_images` embeds JPEG bytes **untouched** as a `/DCTDecode` XObject —
      object bodies became BYTES for it, and `assemble_pdf` is pinned byte-identical for the
      no-image case. `report.rs` collects images too (it embeds composite pages verbatim). SVG
      inlines base64 so a delivered file is self-contained, and a plate the PDF cannot embed prints
      a **named frame**, never a silent gap.
      **Digitizing remains the deliberate later phase** — see the OpenCV note in §B3; nothing here
      decodes pixels in Rust.
      Open follow-ups: whether the overlap rule should stack instead of skip; whether 2400 px is
      too soft for his thin sections; and a picture is not yet re-registerable from the UI (the
      `update_well_image` command exists and is tested, but no dialog calls it yet — a re-import
      is currently how a depth gets corrected).

### B3 — Python capability audit (2026-07-30, at Jauhar's request)

He asked which of a list of Python packages could empower the existing tools, aiming at ML and
office-document output. Verified against the interpreter SandiBumi actually discovers
(`%LOCALAPPDATA%\Programs\Python\Python312`), not against the list in the abstract.

**Already installed there** (so these cost no install): `numpy`, `scipy`, `pandas`, `sklearn`,
`joblib`, `python-docx`, `python-pptx`, `openpyxl`, `xlsxwriter`, `matplotlib`, `Pillow`,
`dlisio`. **Not installed**: `cv2`, `onnxruntime`, `jax`, `tensorflow`, `mediapipe`.

Everything below rides the existing subprocess mechanism, so **rule 7 holds throughout: a
missing package fails only its own button, never the app**, and the native PDF/SVG/LAS paths
stay the default. The one real cost is the install matrix for a client machine, which should be
tiered in CONTRIBUTING.md: nothing required → numpy (equations) → dlisio (DLIS) → sklearn +
joblib (ML) → office four (deliverables) → opencv (digitizing).

- **Office deliverables** — the largest gap in the product. `export.rs` exported **LAS only**;
      everything else left as a native PDF or a CSV, so a finished study had to be re-typed into
      Excel and PowerPoint by hand.
      - [x] **`xlsxwriter` → SHIPPED 2026-07-31** — `src-tauri/src/office.rs` + `workbookDialog.ts`
        (Plot ▸ Deliverables ▸ Workbook…). Four sheets: **Summary** (audit trail — cutoffs used,
        depth unit, export stamp, and every well that produced *no* results named rather than
        silently absent), **Pay Summary** (the same rows `report.rs` prints, so the workbook and
        the client PDF cannot disagree), **Field Summary** (per-zone roll-up), **Zone Parameters**.
        Formatted: number formats per column, frozen header, autofilter, PAY rows tinted.
        Architecture: a shared Python-office spine — `office_support()` probes all four packages
        in ONE subprocess so a dialog reports what is missing *before* the save dialog, and the
        xlsxwriter runner is deliberately DUMB (typed `Cell`s in, a table out) so every
        petrophysical decision stays in Rust. Rule 7 holds: the real round-trip test is
        `#[ignore]`d precisely so the green gate can never depend on a Python package.
        Three decisions worth not re-litigating: **numbers stay numbers** (a text column cannot be
        pivoted, which is the only reason to want a workbook); **a blank is not a zero** — where
        `n_classified == 0` the well was never interpreted, and an empty cell is the one value
        Excel's own AVERAGE/COUNT skip, where a `0` would drag a field average down; and the
        Field Summary carries **two N/G columns** (volumetric Σnet/Σgross *and* the mean of
        per-well values that the dashboard plots) because quoting one as the other is a reserves
        error. The export runs `stats_only` — saving a spreadsheet must never write FLAG curves.
        Open follow-ups: a per-well curve-data sheet (LAS covers it today); whether he wants PHIE
        displayed as a percent format rather than the v/v decimal that matches the PDF; and a
        saved workbook "template" (which sheets, which cutoffs) like the plot templates.
      - [x] **`python-pptx` → SHIPPED 2026-07-31** — asset-team deck (Plot ▸ Deliverables ▸
        **Deck…**, `deckDialog.ts` → `export_deck`). The rasterization question was put to Jauhar
        and he chose **matplotlib figures from the data** over pasted composite pages — the right
        call: python-pptx embeds PNG/EMF, not vectors, so a composite at 1:200 would become a
        picture that stops being at 1:200 the moment anyone resizes it.
        Slides: title, scope-and-cutoffs, field roll-up by zone (the workbook's `field_sheet` — a
        FOURTH rendering of one table definition), net + HPV per zone, N/G / PHIE / SWE
        distributions, well ranking by HPV, and a closing slide naming every well that produced
        nothing. `DeckSpec.flag` picks ONE cutoff level (default PAY) and the title slide says
        which. 16:9; one subprocess for every figure and the deck.
        Rules not to re-litigate: **box statistics come from `distribution.rs` and are passed in
        matplotlib's `ax.bxp` vocabulary**, never `ax.boxplot(raw)` — otherwise matplotlib's own
        percentile convention would make the deck disagree with the Field Dashboard for the same
        wells, and `BoxSpec.n` rides along because a box from three wells is not the statement a
        box from ninety is; **a `None` in a `Series` is a gap, not a zero** (axis label, no bar);
        long tables paginate (`DECK_ROWS_PER_SLIDE`) and the ranking cap (`DECK_RANK_WELLS`) is
        stated in the slide note, because a silent top-N reads as the whole field.
        Open follow-ups: a per-well slide set (needs the frontend rasterization route after all),
        a client `.pptx` template to build onto instead of python-pptx's default, and whether he
        wants a zone-by-zone map slide once the map panel can export.
      - [x] **`python-docx` → SHIPPED 2026-07-31** — the EDITABLE `.docx` twin of `report.rs`'s
        PDF: report pane ▸ **Save Word…**, plus a format select on the Batch button (`as PDF` /
        `as Word`) driving `export_report_docx` / `export_report_docx_batch`. Same title, author,
        editable methodology table, zone parameters *in the PDF's shape* (zone + depths printed
        once per zone; a zone with no parameters is still listed, because dropping it would say
        the zone was not evaluated) and pay summary. **The native PDF stays the default path**
        and keeps the composite log pages — deliberately NOT in the Word file, because they are
        drawn at a true print scale and a picture pasted into a document stops being at that
        scale the moment anyone resizes it; the document says so rather than leaving a gap.
        A `Block` reuses the workbook's `Sheet`/`Column`/`Cell`, so one table definition renders
        three ways and cannot drift. One deliberate divergence: `Cell::Blank` prints as a dash
        here but stays an empty cell in the workbook (Excel's arithmetic skips a blank; a
        document has no arithmetic). Runs `stats_only`, so unlike the PDF path it writes nothing
        back. Open follow-ups: a rasterized composite appendix if he wants the plots in there
        after all, and a client `.docx` style template to load rather than Word's defaults.
      - **Bug found and fixed on the way (2026-07-31)**: every Python runner must read
        `sys.stdin.buffer`, never `sys.stdin` — a piped child's text stdin decodes with the
        Windows ANSI codepage while `serde_json` sends raw UTF-8, so all non-ASCII arrived as
        mojibake. `ml.rs`/`python_engine.rs` were already correct; **`images.rs` was not**, and a
        plate whose path held any non-ASCII character failed with "No such file or directory"
        naming a filename nobody had. Proven both ways against a real file and pinned by
        `a_word_document_keeps_non_ascii_text_intact`.
- [x] **`joblib` model persistence → SHIPPED 2026-07-31** — `ml_models` (a joblib dump plus the
      full description) + `MlRequest.save_model_as` + `apply_ml_model` / `list_ml_models` /
      `rename_ml_model` / `delete_ml_model`, with "Save model as" and a **Saved models** list in
      `mlDialog.ts`. Picked up by `create_schema` on every open, so no migration.
      Contracts not to weaken: **the scaler is dumped WITH the estimator** (refitting a
      StandardScaler on the apply wells is a different transform, and the predictions would be
      quietly wrong rather than obviously broken); **feature ORDER is part of the contract** —
      the artifact carries its own feature list, the apply path drives the fetch from it so a
      caller cannot reorder it, and the runner re-checks inside the artifact and refuses rather
      than predicting; **retraining auto-suffixes rather than overwriting**, because the model an
      existing delivered curve was made with must stay reproducible; **saving happens after the
      curves are written** so a storage failure costs the artifact, not the run. Supervised only —
      clustering/reduction are fitted on the wells they are applied to by construction.
      Open follow-ups: exporting/importing a model file so it can move between projects, and
      showing a model's metrics in the saved list rather than only in its tooltip.
- [ ] ~~**`joblib` model persistence**~~ — a confirmed capability hole, not a guess: `MlRequest`
      carries `train_well_ids` and `apply_well_ids` in the SAME call, so the fitted model dies
      with the subprocess. There is currently no way to train on the cored wells and apply *that
      same model* later — a refit on different data is a different model. Persisting makes a
      trained model a named, citable, re-runnable artifact (fits the delivery-set pattern).
      Small; ride it along with the next ML touch.
- [x] **SHIPPED 2026-07-31 — `scipy` in the equation engine.** The worker binds `scipy` plus
      `signal`, `interpolate`, `optimize`, `stats` and `ndimage` into the script namespace, so
      `signal.medfilt` (despike), `signal.savgol_filter`, `interpolate.interp1d` and
      `optimize.curve_fit` are one line in a user equation. **numpy remains the only
      requirement**: with scipy absent each name is a stub whose first use raises a message
      naming the interpreter and the exact pip command, rather than a bare `NameError` that says
      nothing about what to install or into which of three Pythons. `python_status()` now returns
      `{path, scipy}` and the Equation Editor states it while the script is being written, the
      same "probe before the dialog" discipline `office.rs` uses. **A curve mnemonic always
      shadows a scipy name** — a well logged with a curve called STATS must not silently receive
      `scipy.stats`. Boundary held: this is for the user's equations; core petrophysics stays in
      Rust. Verified end to end through the real runner (medfilt removed a 300 gAPI spike;
      `curve_fit` recovered a φ-k power law to <2%) and both branches are tested — installed and
      absent.
      *Open follow-up:* the editor note could carry a worked despike→smooth snippet, since
      Savitzky-Golay over an un-despiked curve fits the spike rather than the rock.
- [x] **SHIPPED 2026-07-31 — RtC calibration from the user's own water zone** (closes the last
      Tier-B item from the provenance sweep). `lrlc::run_rtc_fit` + Advance ▸ Calibrate RtC…
      regresses A_CAP/B_QV/C0 from measured excess conductivity over a declared water-bearing
      interval. The regression is `sw_rtc`'s own saturation equation inverted at Sw = 1, so the
      fit and the run cannot drift apart. The fit REFUSES without a declared water zone: over
      hydrocarbon the apparent excess is too small, the calibration under-corrects Rt and Sw
      comes back too high, which erases pay. RSF is held fixed (not jointly identifiable), an
      unfittable Qv term is reported as 0 rather than guessed, and every excluded sample is
      counted by reason. `sw_rtc`'s description now states the shipped defaults are one field's.
      (Both follow-ups shipped 2026-07-31 — the measured-vs-fitted cross-plot and the
      write-into-`zone_params` apply step; see below.)
- [x] **SHIPPED 2026-07-31 — IMTS S-factor calibration from the user's own lab CEC.**
      `lrlc::run_s_factor_fit` + Advance ▸ Calibrate S… fits `sw_imts`'s CEC scaling factor from
      laboratory CEC point data against the clay content of the very curves the run will use.
      Same discipline as RtC: the regression is the module's own clay-charge line inverted, via
      the shared `cec_theo_at`, so a fitted S provably makes `sw_imts` reproduce the measured
      CEC. Through the origin (S is a scaling factor, an intercept would be a different claim)
      and clay-weighted rather than a mean of ratios. The drift detector is the P10-P90 SPREAD of
      the per-plug ratios, not the median-vs-fit gap — two central values cannot diverge far
      enough to catch real drift. S > 1 is flagged as a probable unmodelled mineral (smectite at
      80-150 meq/100g against illite's 25), never clamped. A plug outside the depth tolerance is
      dropped, not snapped. `sw_imts`'s description now says the shipped 0.5 was never measured
      anywhere. Also fixed: both fit dialogs opened with a blank run button, because
      `buildWellScope` does not fire `onChange` during construction.
      (Both follow-ups shipped 2026-07-31 - the QC scatter and the dataset/item picker.)
- [x] **SHIPPED 2026-07-31 - calibration QC scatter on both fits** (`fitScatter.ts`, shared).
      RtC plots measured vs fitted excess with a dashed 1:1 line; the S fit plots the regression
      itself, lab vs modelled CEC, with the fitted line through the origin - only that version
      puts clay content on the x axis, which is what turns the P10-P90 spread into a shape with a
      name. Points coloured by WELL with a legend, hover naming well/depth, and the standard
      Copy/Image/Print buttons. Two geometry rules make it one shared module: a measured-vs-fitted
      plot forces both axes to the SAME range so the 1:1 line is at 45 degrees, and a
      through-the-origin plot forces the origin onto the page. Two bugs found by building it: the
      first paint must NOT be deferred to requestAnimationFrame (it does not fire in a
      non-compositing window, and attachResizeRedraw schedules through rAF too, so there is no
      fallback), and the canvas context must be scaled by the dpr fitCanvasBackingStore returns.
- [x] **SHIPPED 2026-07-31 - accepting a calibration writes it** (`calibrationApply.ts`, shared).
      An Apply row under Copy writes the coefficients as `zone_params` overrides through the new
      atomic `db::set_zone_param_batch(conn, zone_name, entries)`; `set_well_param_overrides` is
      now just its `*` scope, so the parameter grid and an accepted calibration share one
      transactional path. Default scope is `wells_fitted` - a new field on both fit results,
      deliberately NOT derived from the decimated display points - because a scoped well that
      contributed nothing was never calibrated; the wider sweep is offered and names the
      uncalibrated wells. The held-fixed constants (RSF; CEC_KAOL/CEC_ILL) are written in the
      SAME batch, since they are not jointly identifiable with the coefficients. One transaction,
      one undo, and undo restores "no override" rather than zero.
- [x] **SHIPPED 2026-07-31 - the S dialog picks the CEC measurement from the project.**
      `db::list_aux_item_catalog` returns every point-data measurement name from the ACTIVE
      delivery of each dataset, with row / well / NUMERIC-row counts, and the dialog turns it into
      two dependent selects. Project-wide and unfiltered by well for the `list_well_param_overrides`
      reason (one grouped scan, no IN-list binding limit). A text-only item is shown GREYED with
      "no numeric values" rather than hidden - a lithology description cannot set a scaling
      factor, and saying so beats a run that fails invisibly. A dataset with nothing numeric gets
      an explicit placeholder; a project with no point data falls back to typed names with a
      VISIBLE warning, since `formRow` hints are tooltips.
- [ ] **`Pillow`** — already present, and enough for the *display* half of the image-track item
      above (read JPEG/PNG/TIFF, dimensions, downsample). No install needed.
- [ ] **OpenCV** — NOT installed, and deliberately deferred to the **digitizing** phase of the
      image track, where it is the actual engine: thin-section modal analysis by colour
      segmentation (point counting without the point counter), core-photo depth registration and
      lithology banding, borehole-image processing. **Scoped 2026-07-31 into
      `docs/plan_image_analysis.md`** (C2 item 8) — note that core-to-log *depth registration* turns
      out NOT to need OpenCV at all: `tops.rs` already carries the best-lag and monotone-warp engine,
      it has simply never been pointed at core.

**Rejected, with reasons** (so this is not re-litigated): `pandas` — DuckDB already is the
columnar frame, and routing through pandas copies data out of the store just to copy it back
(fine inside a runner script, wrong as architecture). `jax` / `tensorflow` / `onnxruntime` —
sklearn's `MLPRegressor` covers the neural-net case for logs; a ~200 MB deep-learning runtime to
unlock the deferred autoencoder is a bad trade. `mediapipe` — face/pose perception, irrelevant.

- [x] **Pickett v2** — **COMPLETE 2026-07-30.** N with M and Rw, free line-parameter input, Z-colour
      by a chosen log: all shipped as Polish-4/#125 above. The tail landed with the multi-well work:
      template bar, RT default widened to 0.2–2000 (audit), `sanitizePickettProps`, Sw lines spanning
      the visible φ window, and the T-SHELL-16 context overlay (line stays the ACTIVE well's).
- [x] **UMAA / RHOMAA MID-plot module** — **SHIPPED 2026-07-30.** `src-tauri/src/lithology.rs`
      (`midplot`), new **Lithology** ribbon category; writes UMAA, RHOMAA, U and PHIA, feeding the
      already-digitized `lith6_mid` chart overlay. Physics: rho_e = (RHOB+0.1883)/1.0704, U = PEF·rho_e,
      then the fluid stripped from both. RHOB is used AS LOGGED — verified against the chart's own
      quartz point (2.6489 = the tool reading for quartz, not its true 2.654 density), which is what
      makes the minerals register. Crossplot gained UMAA/RHOMAA axis defaults (0–16; 2.2–3.1 inverted).
      Six unit tests pin it, incl. each mineral landing nearest its OWN chart point and an explicit
      test that a density-only apparent porosity is **algebraically degenerate** (returns the assumed
      matrix density for every sample) — that option is deliberately absent.
- [x] **Chart-lookup DN crossplot porosity for the MID plot** — **SHIPPED 2026-07-30**, same day,
      closing the approximation the module shipped with. `OPT_PHIA=CHART` is now the default and
      reads the crossplot the way Por-11 is read by hand. The trick that makes it cheap: the two
      unknowns (matrix, porosity) collapse to one, because at any trial porosity each tool implies
      a matrix density on its own — density's rises with φ, the neutron's falls — so their
      difference is strictly monotone with exactly one root. Bisection, ~20 halvings to 1e-6, no
      derivatives, no initial guess, cannot diverge. `TOOL`/`SALINITY` pick the curve family
      (reusing `nphimat`'s tables and `chart_lerp`, now `pub(crate)`); `RHO_MA_SS/LS/DOL` are the
      three matrix lines, refused if a zone override crosses them out of order.
      Round-trip tested and that is the real proof: build the two readings a known rock produces,
      feed them back, and the solver returns that rock — φ to 1e-3 and RHOMAA onto its own matrix
      line for all three matrices at 0/5/12/25/35 pu. The dolomite bias is gone (0.002 vs XPLOT's
      0.06 g/cc). Off-family samples clamp instead of vanishing, so anhydrite still plots heavy and
      the gas signature stays low-left. XPLOT is retained for comparison with commercial suites.
- [ ] **Data prep**: **split & merge** of curves/intervals; **normalization with tops-referenced
      intervals** — reference top/bottom from a chosen tops set; missing marker → nearest stratigraphic
      marker (top → shallowest, bottom → deepest); percentiles extrapolated over the whole interval and
      normalized together.
- [ ] **Highlight tool** (the reference suite-style): multiple depth highlights, same or different colors; convert
      highlights → tops.
- [ ] **Typography**: text reads slightly fuzzy/washed-out up close — investigate WebView2 rendering
      (display scaling, weight, contrast). Waiting on Jauhar to say whether it's blurriness or lightness.
- [ ] **Depth units, increment 2** (2026-07-29; increment 1 shipped — `units.rs`, project-declared
      unit, import conversion, `wells.depth_unit`). Remaining, all gated on a project declared in
      FEET (a metric project is correct today):
      (a) **`satheight.rs:181` + `shf_fit.rs:897/1069/1284`** — `pc = 0.433·Δρ·(h·FT_PER_M)`
      assumes `h` arrived in metres, so Pc is 3.28× off on a foot project. Needs the project unit
      inside `ModuleContext`; the two production construction sites are `workflow.rs:320` and
      `montecarlo.rs:1128` (the other ~15 are test helpers, so a reserved opts key beats widening
      the struct). **Silent numeric error — do not delegate.**
      **LIVE, not theoretical (Jauhar, 2026-07-29): he will declare Rokan/Central-Sumatra
      projects in FEET**, so every saturation-height run on those projects returns a Pc 3.28×
      too high until this lands. He has deliberately deferred it to his manual-test pass of the
      saturation-height section — so it must be fixed BEFORE any foot-project SHF result is
      trusted or shipped, and this is the highest-priority item in increment 2.
      (b) **`LogCanvasRenderer.PX_PER_UNIT_1_1`** (96/0.0254) hardcodes metres, so every named
      1:N print scale is mislabelled by 3.28× on a foot project.
      (c) **The view toggle** Jauhar asked for: `appState.displayDepthUnit`, independent of the
      stored unit, live-switchable. Display sites: log-view depth axis (`logViewPanel.refreshDepthAxis`,
      one function), depth readout, tops panel + tops editor, zones, composite scale bar, report
      pages, dashboard depth columns, crossplot/histogram depth-coloured axes.
      (d) **Settings UI** to re-declare the project unit, refusing (or offering a full migration)
      once wells exist — re-declaring alone would silently reinterpret every stored depth.
      (e) Tops/zones/core/deviation CSV imports carry no unit and are assumed to be in the project
      unit; say so at the import dialog.
      Curve units (RHOB g/cc↔kg/m³, CALI in↔cm, DT us/ft↔us/m) are a later wave — Jauhar chose
      depth-first. Downstream: this is the interchange contract with **SegaraBumi**.
- [ ] **Multi-well plots — crossplot SHIPPED 2026-07-30** (T-SHELL-16 increment 1): the
      additive context-overlay design below, exactly as recorded — well-scope button in the
      crossplot toolbar (Active default = old behaviour byte-identical), context wells fetched
      per-well with zone/top windows resolved BY NAME in each well's own depth frame (missing →
      skipped + counted), 60k-point total budget with stride decimation, per-well colours +
      legend with the display-only contract stated on the plot, auto-range over the combined
      cloud, `getState` round-trips the scope (`wells:` spec). `wellScope.ts` gained an
      "Active" mode + `serialize()`/`describe()`; `drawScatter` accepts a uniform colour.
      **Increment 2 (2026-07-30): histogram scope SHIPPED** — context wells as stepped
      outlines behind the active bars, each normalized to its OWN sample count and scaled
      to the active axis (shape comparison — a 3×-bigger well never dwarfs the active
      one; in Normalize-% mode outlines are true per-well percentages), pooled X range,
      legend top-left with the display-only contract. The context machinery (zone/top-by-
      name window resolution, budgeted concurrent fetch, stride decimation) moved to
      `plotCommon.ts` (`contextZoneWindow`/`fetchContextLayers`) and the crossplot now
      shares it — one source of truth for the correctness-critical rules.
      **Increment 3 (2026-07-30): Pickett SHIPPED** — decision taken: overlay context
      clouds while the Sw lines / M/N/Rw stay the ACTIVE well's (stated on the plot:
      "line = ACTIVE well's parameters") — the overlay's purpose is showing whether
      neighbours share the active well's water line. Also completed the queued Pickett
      v2 tail: template bar, audit's RT 0.2–2000 default, saved-props sanitizer
      (`sanitizePickettProps`), Sw lines spanning the visible φ window instead of the
      fixed 0.01–1. T-SHELL-16 is now CLOSED except: a per-well colour-stability rule
      if Jauhar wants colours pinned across scope edits (cosmetic, on request).
      Original design notes follow (kept for reference):
- [ ] **Multi-well plots — DESIGNED, not yet built** (T-SHELL-16, 2026-07-29). Design settled
      during the units session; build it as its own increment rather than a tail-end change to
      `crossplotPanel.ts` (~2,100 lines, field-verified, and the most interaction-dense panel
      in the app).
      **Approach: additive overlay, not a rewrite of the fetch path.** Scope defaults to
      "Active well", where behaviour must be byte-identical to today; any additional wells are
      fetched separately and drawn as a context layer BEHIND the active well's points, in a
      per-well colour with a legend. This keeps the blast radius off the existing single-well
      path, which is what the field review already accepted.
      **The complication that forces the design** (found while scoping): linked brushing maps
      crossplot points back to depths via `setBrushedDepths`, and a depth only means something
      relative to ONE well. So brushing, the parameter-pick handle, the zone selector, core
      overlay and Thomas-Stieber endpoints stay bound to the ACTIVE well; context wells are
      display-only. That has to be stated in the UI, not just assumed, or a user will brush a
      cloud and get the wrong well's depths highlighted.
      Also needed: a total-point budget with decimation (2,000 wells x ~5,000 samples is 10M
      points — the existing single-well path never had to care), and `getState` round-tripping
      of the scope so a well switch doesn't silently drop the overlay.
      Same treatment afterwards for histogram (simpler — no brushing handle) and a decision on
      whether Pickett takes a scope at all, since its m/n/Rw are per-well parameters.
- [ ] **Multi-well plots, original note** (T-SHELL-16, 2026-07-29): "histo, xplot etc (except log view) cant display
      multiple groups together, better have option for well selections like modules". Histogram,
      crossplot and Pickett are hard-wired to `selectedWell` — one well per pane, so a field-wide
      crossplot means opening N panes. Give them the same **well-scope selector the run dialogs use**
      (`wellScope.ts buildWellScope`: All / Active / ★ Pinned / Group / Custom), fetch and concatenate
      the scoped wells, and colour/legend by well. Needs: scope selector in each plot toolbar, a
      multi-well fetch path, per-well colour + legend, a point-budget/decimation rule for 2000 wells,
      and `getState`/plot-template round-tripping of the scope. Pickett and the chart overlays already
      assume one well's parameters — decide whether they take the scope too or stay single-well.
      **Medium-sized increment; not a bug fix — deliberately not bundled with the Round 97 fixes.**

## B3. Feature Wave B (§4c) — leverage existing engines (small-to-medium, high payoff)

**Status 2026-07-22:** (13) shipped+committed `d64bdc7`. (9) fluid contacts, (3) ML leaderboard,
(16) well-diagram track — all built + cargo-tested + tsc-clean, NOT committed, NOT field-verified.
(8) rock typing + SHF — **increment 1 done** (rocktyping module FZI/GHE/Winland/PGS + Cuddy FOIL
SHF fit + FWL scan, unit-tested); **increment 2 open** (task-tracked separately).

- [x] **(13) Monte Carlo parameter sensitivity.** montecarlo.rs already samples per realization but
      **discards the draws** — keep them, add Spearman rank correlation of param vs output across
      realizations + **tornado chart**, plus a one-at-a-time sweep mode; scope selector = single tool or
      whole workflow (bulk). *(This is what "MC sensitivity analysis" means — the uncertainty engine is
      done in [A6](#a6-field-scale--phase-9-3-batch-workflows-uncertainty-dashboards--); this adds the
      sensitivity ranking.)* → `code_compute_ml_mc.md`.
- [x] **(3) ML comparison quantification.** Loop algorithms × input-curve subsets in one job; leaderboard
      ranking; **well-grouped CV + blind-well holdout** (current random 5-fold leaks depth correlation);
      feature importance (permutation); confusion matrix. exec_ml is already one-shot per call — harness
      loops it. → `code_compute_ml_mc.md`.
- [x] **(9) Fluid contacts in well correlation.** New `fluid_contacts` store (well/field, type
      OWC|GWC|GOC|GDT|ODT, depth, TVDSS flag, color), editor UI, rendering in correlationPanel as
      horizontal lines + connectors; requires adding a **TVDSS depth mode** to correlation (contacts are
      flat in TVDSS, not MD; deviation.rs paths exist). Optional: show in log-view tops overlay.
      Supersedes the §4 New-capability 2D-Window fluid-contacts note. → `code_data_db_import.md`.
- [x] **(16) Well-diagram track in layout.** `Track` gains a `kind` field (`"curves" | "well_diagram"`,
      serde-default for saved-layout compat); draw casing/shoe/tubing/perfs on the 2D overlay canvas;
      perfs already in aux_data; casing/completion needs a store + importer (reserved aux_data dataset
      `COMPLETION`); BS available as curve. Mirror in composite/report export. → `code_data_db_import.md`.
- [~] **(8) Rock typing + SHF building.** *Increment 1 shipped: `rocktyping.rs` module (RQI/FZI + GHE bins, Winland R35, PGS) + `shf_fit.rs` Cuddy FOIL fit + FWL scan (unit-tested). Increment 2 open — see below.* Rock typing: FZI/HFU (Amaefule + GHE bins), Winland R35/Pittman,
      Lucia RFN, PGS (Permadi & Susilo ITB — verify exponent vs paper), perm binning per Mahakam phi-k-laws
      preset, electrofacies tie-in with confusion-matrix QC. SHF *fitting* side (forward `sw_height` exists):
      Leverett-J, Brooks-Corey, Thomeer, Skelt-Harrison, Cuddy FOIL/BVW, log-derived per-RT Sw(h), FWL scan
      (Cuddy Eq 19) + gradient-intersection; SCAL importers for porous-plate wide tables + centrifuge
      workbooks. Fitted laws export into the existing sw_height parameter table. → `ref_rocktyping_shf.md`.

## B4. Carried-forward deferrals from the build arc

Small-to-medium open bits left behind by shipped phases (each linked from its phase above):
- **Log-view read path from the generic store** (Phase 6): rewire `get_track_data` so PEF/CALI are
  drawable in a track; curve-set selector in the layout picker; optional TVD depth scale in log/correlation
  (plumbing already built, dead-code-tagged).
- ~~**Per-well parameter override table** in the Workflow Builder (Phase 9-2).~~ **DONE 2026-07-30** —
  see Phase 9-2 above. Follow-up left open: CSV **import** back into the grid (export exists).
- **Monte Carlo** (Phase 9-3): per-zone parameter distributions (currently well-wide); persisted
  P10/P50/P90 *curves*. *(Plus the §4 New-capability "print LOW/BASE/HIGH curves" item in [C5](#c5-new-capability-misc-4).)*
- **Full-field responsiveness** (Phase 9-5): lazy catalog loading, decimation cache, keep the UI responsive
  during full-field runs, 2000-well synthetic stress fixture (100-well is the current proof).
  **Shipped 2026-07-30 (from the BLSO 2.5 GB / 6 GB RAM / 15-min-open field report):** DuckDB
  memory cap on every open (default/4 clamped [1,4] GiB, `SANDIBUMI_DB_MEMORY` overrides),
  **Compact Project** (engine rewrite in place, all-table row-count verification, original parked
  as `.pre-compact-<ts>`), Save As now engine-copies (compacted export), and boot/migration
  notices surface in the status line + History (`boot_report`) instead of an invisible stderr.
- **Missing-curve synthesis** (Phase 10): per-field regressors for DT/NPHI where absent, with holdout-well
  R² report.
- **Auto-picks** (Phase 10): per-zone GR_MA/GR_SH percentile suggestions, change-point auto-zonation,
  field-wide spike/outlier QC.
- **Smaller UI deferrals**: draggable cutoff polygon / per-axis zoom lock (interactive plots); rule-based
  auto-membership + create-group-from-multiselect (well groups); named tops SETS; templates on
  Pickett/Correlation; composite hatch-lithology track + annotations; report histogram/crossplot pages +
  narrative/bilingual/exec-summary/SWHF/correlation-export.

---
---

# 🔮 PART C · FUTURE

Bigger lifts, planned but not scheduled. The method-suite and data-model waves each have a full spec in
`docs/research_2026-07/` — read it before starting.

## C1. Method-suite waves — Wave C (§4c): from Jauhar's reference canon

- [ ] **(10) Thin-bed / LRLC suite** (his specialty; richest reference grounding — Passey 2006, Bateman
      1990, Thomas-Stieber 1975, Mollison/Mezzatesta 2002, Klein 1995/97 + Jauhar's own Klein-plot Excel,
      Yadav 2010, Elhadidy 2020, Madjid-Worthington 2012, Worthington 2000 all read). Build order:
      Worthington LRP screening → Madjid-Worthington scenario router → Thomas-Stieber per-depth solver
      (crossplot overlay exists) → Bateman binary-lithology + Rt enhancement → Klein plot widget +
      Hagiwara/Fanini Vshl-Rsd tensor solver, with **Elhadidy multi-well dip-fit as the no-triaxial
      fallback (the Mahakam case)** → Passey VLSA interval Monte Carlo → (later) Mollison LSSA full
      inversion. Note: printed Mollison eq 19-21 have suspected typos (kv/kh swapped, Coates ratio
      inverted) — implement physics-correct forms. → `ref_thin_bed_lrlc.md`.
- [ ] **(1a) TOC / unconventional.** Passey ΔlogR (sonic/density/neutron + generalized calibrated form;
      interactive baseline picker), Schmoker & Myers-Jenkyns density TOC, Schmoker-Hester inverse, uranium
      excess (warn: unreliable for deltaic Type-III OM — mask coals first), Meyer-Nederlof discriminant,
      MLR/ML TOC, RockEval/LECO calibration layer, brittleness index, adsorbed/free gas. Needs one-time Hood
      LOM chart digitization. → `ref_toc_unconventional.md`.
- [ ] **(1b) Geomechanics 1D MEM.** Phased: (i) conditioning (Faust DT, DTS regression, RHOB extrapolation)
      + Sv integration + NCT + Eaton PP + FG (Eaton-Poisson, Thiercelin-Plumb, Matthews-Kelly) + dynamic/
      static moduli + UCS/φ correlations; (ii) Kirsch + failure criteria (Mohr-Coulomb, Mogi-Coulomb,
      Drucker-Prager, Modified Lade — closed forms from the 221102 LAPI-ITB deck) → collapse MW + mud window
      + max injection (CFF); (iii) Bowers loading/unloading (needed for Mahakam overpressure) + breakout
      SHmax inversion. Ship CSB/Rokan calibrations as a named preset. → `ref_geomechanics.md`.
- [ ] **(15) Rock physics.** Mirror the reference suite GP02 (incl. two known the reference suite bugs NOT to copy): Phase 1 =
      Batzle-Wang fluids + VRH solid mix (consumes SandiMin volumes) + fluid mixing (Reuss/patchy/Brie) +
      Gassmann clean & Vsh + Vs prediction (Greenberg-Castagna iterative, Han, mudrock) + elastic attributes
      (AI/SI/VpVs/Poisson/LMR) + reflectivity/EI; Phase 2 = bounds + Krief/critical-porosity + contact
      models; Phase 3 (defer) = Xu-White/Xu-Payne/DEM, time-domain synthetics. Standardize SI internally.
      → `ref_rock_physics.md`.

## C2. New data-model suites — Wave D (§4c): biggest lifts, each needs new storage

- [ ] **(5) NMR suite.** Needs array-curve storage (T2 bins as LIST/FLOAT[] per depth + bin-time metadata;
      DLIS array channels, LAS BIN01..NN re-pack). Then CBW/BVI/FFI partition (T2cutoffs 4/33/92 ms
      defaults), SBVI, T2LM, Timur-Coates + SDR perm, Swirr, MPHI/MSIG QC, DMR gas-corrected porosity;
      pseudo-Pc (Kappa/T2) with MICP calibration; MRIAN dual-water Sw (ties into his LRLC work); defer
      dual-TW/TE typing. Coates 1999 in library, read. → `ref_nmr.md`.
- [ ] **(6) Image log suite.** Largest single item. Data model (pad arrays, oriented array, versioned
      _S/_ISC/_H/_STATIC/_DYNAMIC chain per Techlog convention): speed correction, pad creation/EMEX, button
      harmonization + dead-button repair, concatenation/orientation, static+dynamic normalization; then
      interactive dip picking (5 modes → true dip), dip datasets + classification, auto-dip, stereonet/rose/
      walkout/cumulative plots, structural dip removal, fracture counting w/ Terzaghi, aperture
      (Luthi-Souhaite), image porosity + binarization + sand count. → `ref_image_core.md`.
- [ ] **(7) Core photo digitization.** Non-destructive recipe model: crop/deskew/perspective, color-card +
      white-balance, CLAHE/denoise/sharpen, depth registration + stitched strip pyramid, core-to-log shift
      (photo-proxy-log cross-correlation vs GR), WL/UV pairs, log-view strip track. Absorbs the §4
      New-capability "core image input" stub. → `ref_image_core.md`.
- [ ] **(8) Depth registration, then plate digitizing** — scoped 2026-07-31 at Jauhar's direction
      ("all of those, it should be depth registered first, then the quantification or qualitative
      analysis"). **The plan is `docs/plan_image_analysis.md`**; it supersedes the loose OpenCV note
      in §B3 and overlaps (7) on the registration half. Two tiers: **Part 1** core-to-log depth
      registration (a pane, a proposed best-lag reusing `tops.rs`'s existing `best_shift`/`warp_refine`
      rather than a new algorithm, per-core-run piecewise shift, plates following their plugs, and the
      still-uncalled `update_well_image`), then **Part 2** digitizing (modal analysis by colour, pore
      geometry, grain size) through an OpenCV **subprocess**, storing results in `aux_data` and
      `array_logs` so nothing new is needed downstream. Registration is first because nothing in the
      repo can check a depth: the S-fit's own test records that a whole-sample shift is invisible to
      any depth-tolerance check. Four open decisions are listed in the plan's §4; only **D1** (does he
      receive core gamma?) blocks the first increment.

- [x] **Core-to-log depth registration (Part 1, increments 1a+1b)** — **SHIPPED 2026-07-31.**
      `registration.rs` + `depthRegDialog.ts` (Data ▸ Tools ▾ ▸ Register Depth…). Answers D1
      ("not always, sometimes"): a delivered core gamma against GR is **like-for-like** and the
      search maximises signed r; a core porosity against GR is a **proxy**, co-varies inversely,
      and the search maximises |r| and reports the sign. Pinned from both sides — a signed score
      fails the proxy test, |r| everywhere fails the like-for-like test. Reuses `tops.rs`'s
      `interp`/`pearson` rather than a second implementation; the whole **correlogram** comes back
      so a comb of near-equal peaks is visible rather than reduced to one confident number, and
      nothing is applied without the user accepting it. A pair-count floor (75% of the
      best-populated shift) stops the core sliding off the log's end and winning on six lucky
      plugs. Also fixed here: **`db::shift_core_depths` now moves the plugs and the point data
      measured ON them in one transaction** (`CoreShiftCounts`), because core extras live in
      `aux_data` under the core set's own name and were being left behind — the core gamma that
      justified a shift did not move with the porosity it was judged against. Which datasets ride
      along is offered (`db::core_extra_datasets`), never inferred from the set name alone.

- [x] **Plate depth editing (Part 1, increment 1e)** — **SHIPPED 2026-07-31.** `plateDepthDialog.ts`
      (Data ▸ Tools ▾ ▸ Plate Depths…) is the missing caller for `update_well_image`, closing the
      follow-up left open when the image track shipped — a plate at the wrong depth previously
      needed a delete-and-re-import. Adds `db::shift_well_images` for the whole-delivery case (ONE
      statement following `ACTIVE_IMAGE_SET`; per-plate calls would be hundreds of IPC round trips
      for a core-photograph delivery). **A blank base stays a POINT sample** through both paths —
      `depth_base + delta` is NULL-safe, so a shift never gives a thin section a thickness — and a
      base above the top is refused rather than swapped. Per-plate edits and bulk shifts are both
      undoable. **D2 answered tentatively** ("yes, but its tentative"): plates riding
      `shift_core_depths` automatically is increment 1d and is deliberately NOT wired yet, because a
      picture that moves without being asked is the same class of error as a core extra that fails
      to move at all.

- [x] **Per-barrel shifts + the core depth record (Part 1, increment 1c)** — **SHIPPED 2026-07-31.**
      From Jauhar: core comes up in barrels, pieces can shift inside a barrel too, and **the core
      set must record the shift so later deliveries follow it**. `db::RunShift` +
      `apply_core_run_shifts` (free intervals, so a moved piece is just a shorter range) with the
      barrel table in `depthRegDialog.ts`, each row proposing its own shift through the existing
      `registration.rs` engine. New `core_data.depth_orig` (migration
      `db::migrate_core_depth_orig`, non-destructive, must run after `migrate_point_data_sets`)
      never moves, so `core_depth_pairs` + `map_core_depth` place a later XRD/CEC delivery written
      at the lab's depths onto where that rock now sits — interpolated between plugs (the offset
      really does vary when pieces moved), held and FLAGGED outside the cored interval. Two rules
      enforced in a dry run before any write: no shift may reorder the core, and two ranges may not
      overlap (touching is fine). **The inverse for undo is computed by the backend**
      (`CoreShiftCounts.inverse`) — a browser check caught that negating the caller's own ranges
      produces overlapping inverse ranges when barrels move by different amounts, which would undo
      some plugs by their neighbour's correction. Remaining on this thread: wiring the map into the
      import wizards so it is offered rather than available (Part 1 follow-up).

- [x] **A late delivery follows the core (Part 1, increment 1c follow-up)** — **SHIPPED 2026-07-31.**
      `ingest::import_aux_file(..., follow_core)` + the "These depths came from the core report"
      tick-box in Data ▸ Import Aux…. Turns the depth record from something the app *can* do into
      something it *offers*: XRD/CEC/petrography written at the original core depths is placed
      through `db::core_depth_pairs`, per WELL (a multi-well file routes by its WELL column and each
      well has its own record). Off by default — nothing in a delimited text file says which depth
      scale it uses, so this is the user's declaration, like the RtC fit's water zone. An interval
      is placed by its top with the base taking the same offset (mapping the ends independently
      could invert a thin sample at a barrel boundary). Reported rather than assumed: samples
      outside the cored interval, a core that was never shifted, and a well with no core to follow.
      Pinned by `a_late_delivery_can_follow_the_core_it_was_measured_on`. **Not yet offered for SCAL
      or image imports**, which also arrive at lab-written depths.

- [x] **SCAL and image imports follow the core too (Part 1)** — **SHIPPED 2026-07-31.** Extends the
      point-data tick-box to `ingest::import_scal_files(..., follow_core)` and
      `images::ImageImportRequest.follow_core` (`#[serde(default)]`). All three sources are measured
      ON core and carry the core report's depths. One shared control, `src/ui/followCore.ts` — the
      same decision in three dialogs, and three copies would drift. SCAL rows with no depth are left
      alone and said so; a plate's top is mapped with the base taking the SAME offset, so a core
      photograph keeps its logged thickness and a section with no base stays a point sample.
      `ScalImportResult` gained `note`. Test note: the image round-trip needed a genuinely decodable
      `REAL_JPEG_HEX` (159 bytes) — the existing `tiny_jpeg()` stub is header-only and Pillow refuses
      it. **Still not automatic**: a delivery already in the project does not move when the core is
      re-registered afterwards — that is increment 1d, waiting on Jauhar firming up D2.

- [x] **Mineral classifier (Part 2, family A3 — Tier 3)** — **SHIPPED 2026-07-31**. A supervised
      per-pixel classifier trained on the user's OWN clicks; **nothing ships pre-trained**, because
      a model fitted under somebody else's lamp gives numbers with the shape of a modal analysis and
      none of the content. Clicking is the method — it is point counting, producing training data
      instead of a tally. **The labels are the artefact, not the model**: they persist as a
      `platelabels` document and the seeded forest is refitted from them, so the answer stays
      readable and reproducible (deliberately unlike `ml_models`). **CV groups by CLICK**, since a
      click's neighbouring pixels are near-identical and splitting them across a fold reports an
      accuracy nobody can reproduce. **Recall is per class and the weak ones are named** — an
      overall 0.9 sits comfortably on a mineral the model cannot see. One class and under-3-click
      classes are refused before the subprocess starts. Features are colour + local texture, with
      hue entering as sin/cos because it is circular. Measured: two halves of identical mean colour
      differing only in texture gave accuracy 1.000 and fractions 0.504/0.496; the control — one
      uniform material labelled as two minerals — fell to **0.410**, near chance, and was named as
      unreliable. Items are `CLS_<MINERAL>`, never `MIN_`.
- [x] **Stained carbonate (Part 2, family A2)** — **SHIPPED 2026-07-31**. Mineral area fractions
      from a DECLARED stain, in the same run and off the same pore mask, so pore + minerals +
      unclassified = 1 (measured exactly 1.000). **A plate whose own stain does not match the
      scheme is refused by name, and undeclared is refused too** — reading the wrong scheme returns
      fractions that are wrong and entirely plausible, and the evidence for "this is alizarin red"
      is the red about to be measured. Identifications are published (Friedman 1959, Dickson 1966);
      the colour bands are round starting points and editable, the same split as the epoxy band.
      `StainBand` carries a saturation CEILING because "unstained dolomite" is the absence of
      colour. `MIN_UNCLASS` written every run — the honesty number. **The blue-epoxy / turquoise
      ferroan-dolomite collision is real and measured**: with the default epoxy band the synthetic
      plate returned pore 0.500 and ferroan dolomite 0.000; narrowed, 0.250 and 0.250.
      `epoxy_collides` names the affected minerals and never resolves it automatically.
- [x] **Grain size (Part 2, family B — D3 closed)** — **SHIPPED 2026-07-31**. Jauhar's answer
      ("apply wicksell correction is optional") shipped as apparent-by-default with the correction
      as a tick, and implemented as **different item names** rather than one name and a flag:
      `GRAIN_D50_APP` vs `GRAIN_D50_W`, with no bare `GRAIN_D50` anywhere. The split is a
      **nearest-centre partition** of the solid phase, not `watershed_ift` — that was tried and
      measured, giving one grain 47792 px and the other 9 on a welded pair the new code splits
      23957/23844. Confined to one connected blob at a time, or a pixel can be nearer a centre
      across open pore and one label lands in two places. **`GRAIN_CONTACT` rides with every run**:
      where the rock is cemented there is no pore for the picture to see and the boundary was
      placed rather than observed, so above 0.7 the notes say to read those sizes as rock fabric.
      Sorting is **Folk & Ward (1957)** in phi; everything is area-weighted, which on a section IS
      volume weighting (`n·D³`), so apparent, corrected and a sieve are all comparable. Wicksell is
      **Saltykov derived from the chord geometry, not a transcribed coefficient table**, twelve
      log classes with class 0 reaching zero so nothing is lost to a bin edge, negative classes
      clamped and counted. Measured finding recorded in the tests: the correction earns its place
      on SORTING, not on D50 — area weighting already absorbs most of the median bias.
- [x] **Plug QC — the petrography numbers meet an independent measurement** — **SHIPPED
      2026-07-31**. `plugqc.rs` + `plugQcPanel.ts` (Petrophysics ▸ Petrography ▸ Plug QC…) pair two
      measurements of the SAME plug: a routine-core column, any numeric point-data item (where every
      petrography output lands), or a pore-throat radius read off that plug's own Pc curve. **A
      sample with no partner inside the depth tolerance is dropped and counted, never snapped** (the
      S-fit rule — a core off by a whole sample interval is invisible to any tolerance check), and
      **a measurement is used ONCE** so two nearby sections cannot both claim one plug and tighten
      the correlation for free. **Pearson AND Spearman**, because bodies-against-throats should move
      together without falling on a line — and Spearman survives a log axis, so the number never
      disagrees with the picture. Throat radius is Washburn with the lab's own σcosθ from
      `scal_pc.ift` (no ift → excluded by name, as `thomeer.rs` does), Pc interpolated in **log Pc**,
      a saturation outside the measured range **never extrapolated**, default 35% = the
      Kolodzie/Winland r35 convention already in `rocktyping.rs`. Medians of both axes reported so a
      percent-versus-fraction delivery is visible rather than silently ruining a 1:1 comparison.
      `fitScatter.ts` gained a `{kind:"none"}` line and log axes; `.form-row[hidden]` added to
      styles.css.
- [x] **Pore geometry (Part 2, family C)** — **SHIPPED 2026-07-31**. Per-pore shape and size beside
      the area fraction, from the SAME mask in the same pass so the two can never describe different
      pictures. Outputs `PORE_N` / `PORE_ASPECT` / `PORE_SHAPE` for every plate and
      `PORE_D10/D50/D90` in µm only where a scale exists — no NaN placeholder, which would read as
      a measurement that failed rather than one never possible. **Four-connectivity** (a corner
      contact is a throat of zero width, not one pore). **Crofton perimeter, not a boundary-pixel
      count** — a staircase overestimates a diagonal edge by up to √2 and biases circularity
      systematically low; measured 630.1 against a true 628.3 on a disc, worst case ~5% low on an
      axis-aligned rectangle, both pinned by test. **Aspect from second moments** with the +1/12
      discrete correction, exact on a disc and on a 5:1 bar. Edge-touching pores excluded and
      counted; speckle below a PIXEL threshold dropped and counted. **Diameters area-weighted**,
      because capillary pressure fills volume. Runner returns per-pore arrays; every statistic is
      computed in Rust. Needs scipy for the labelling only, so it is opt-in and its absence never
      touches the area fraction.
- [x] **Scale bar calibration (Part 2, the scale gate opened)** — **SHIPPED 2026-07-31**.
      `src/ui/scaleBarDialog.ts`, the ⇹ button on each Plate Details… row: drag along the plate's
      own printed scale bar, type what it reads, get a field of view. **The measurement is a pure
      ratio** — the bar as a FRACTION of the picture's width — so it is invariant to display zoom
      and to the stored copy's resampling, and comes out already in the form the store wants
      (verified: the same drag at 848 px and at 400 px displayed width both returned 2000 µm).
      No snapping, because a 5° error is 0.4%; Actual size is the mode that matters, because
      hitting the bar's ends is what decides the accuracy. It only FILLS the box — the row's Save
      still writes it. The optional apply-to-delivery goes row by row so each plate keeps its own
      preparation and stain. This opens the gate for the dimensional families (B grain size, the
      sized parts of C).
- [x] **Pore area from blue-dyed epoxy (Part 2, family A1)** — **SHIPPED 2026-07-31**.
      `petrography.rs` + `poreAreaDialog.ts`, Petrophysics ▸ Petrography ▸ Pore Area…. The first
      measurement off a plate and deliberately the dimensionless one, so it runs on every plate
      rather than only the calibrated ones. **A plate must be declared impregnated and an
      undeclared one is refused BY NAME** (`epoxy_check`) — the rule the whole feature rests on,
      because a blue rule on an unimpregnated section returns a plausible porosity instead of
      failing, and impregnation cannot be read off the pixels without begging the question. The
      colour band is the user's, tuned visually; **the preview overlay comes from the same runner
      that does the measuring**, so the two can never drift. **No morphological cleaning** — a
      structuring element is a size in pixels and a plate may carry no scale, so nothing is
      smoothed and the speckle stays visible. Results are POINT DATA (`PETROGRAPHY` / `VPORE_TS`)
      at each plate's depth, never a curve; measuring and saving are separate buttons so tuning
      writes nothing. numpy + Pillow in one subprocess per 16 plates (rule 7), with the real
      round-trip test `#[ignore]`d so the gate never depends on an optional package.
      Next in Part 2: A2 stained carbonate (needs the lab's stain protocol), then C pore geometry.
- [x] **Plate scale and preparation (Part 2, increment 2.0 — D4 answered)** — **SHIPPED
      2026-07-31**. Jauhar answered "sometimes" on both the scale and the epoxy, so one delivery
      holds plates of both kinds: `well_images` gained `fov_um` / `prepared` / `stain` PER PLATE,
      all declared, all defaulting to absent (`db::migrate_plate_scale_and_prep`, ADD COLUMN only).
      **Scale is a field of view WIDTH, not um/px** — the stored copy is resampled, so a ratio
      belongs to whichever copy it was measured on while a field of view survives resampling; um/px
      is derived per copy. **Unknown preparation is refused, never assumed**: a blue-epoxy rule over
      an unimpregnated section returns a plausible porosity rather than failing, and detecting
      impregnation from the pixels is circular. Delivery-level values fill the blanks, the per-plate
      **FOV mm** column overrules them. `src/ui/plateDetails.ts` is the one shared control;
      `db::set_image_details` / `set_image_delivery_details` write one plate or a whole live
      delivery, with `None` written as given so a wrong entry is clearable. Data ▸ Tools ▾ ▸
      **Plate Details…** (renamed from Plate Depths…). Next: A1, pore area from blue epoxy.
- [x] **The core carries its own depth history (Part 1, increment 1f — COMPLETES PART 1)** —
      **SHIPPED 2026-07-31**. `core_registrations` holds one row per moved range, written inside
      the SAME transaction as the move: `shift_core_depths` and `apply_core_run_shifts` take a
      `RegistrationNote`, and there is no "do not record" value. **An event log, not a state
      table** — an undo appends its own reversal rather than erasing the row it reverses, because
      a core that was registered, judged wrong and put back is not the same as one nobody touched.
      The stored correlation is the one at the shift ACTUALLY applied (`correlationAt` reads the
      applied delta off the scan), and it is **per range**, so `RunShift` gained
      `correlation`/`n_pairs` — each barrel is judged on its own correlogram, and a range typed by
      hand records a blank rather than a zero. History shown at the foot of Register Depth…
      D4 also closed this session ("sometimes" for both scale and epoxy/stain, so both become
      declared per-plate properties defaulting to absent — `docs/plan_image_analysis.md` §4.1).
      **Part 1 of `docs/plan_image_analysis.md` is complete.**
- [x] **Already-imported deliveries follow a later re-registration (Part 1, increment 1d)** —
      **SHIPPED 2026-07-31**, on Jauhar's firm yes (D2 closed). `ShiftTargets` on
      `db::shift_core_depths` / `db::apply_core_run_shifts` carries the chosen point datasets, the
      live SCAL delivery and each chosen image delivery with the plugs in one transaction;
      `CoreShiftCounts` reports plugs / extras / scal / plates. **Which deliveries belong to the
      core is recorded, not guessed**: `aux_sets`/`scal_sets`/`image_sets` gained `on_core_depths`,
      written from the import tick-box (`db::migrate_delivery_depth_basis`, ADD COLUMN only,
      existing rows get 0 = leave alone). `db::core_shift_candidates` lists every live delivery WITH
      its flag rather than filtering, so an older project does not look empty; the dialog pre-ticks
      the flagged ones and marks the rest "not marked as core-depth data". The tick-boxes sit at
      dialog level so the single-shift and per-barrel Apply share them — they were briefly inside
      the result block, which made the barrel path ignore them, caught in the browser rather than by
      the compiler. Part 1 of `docs/plan_image_analysis.md` is now complete except 1f (recording why
      the core sits where it does).

- [x] **Run the petrography suite on a real delivery** — **SHIPPED 2026-07-31**. 134 real
      photomicrographs, one carbonate delivery. Three findings. (1) **The plates arrive inside an
      Excel workbook**, one worksheet per plate with well/depth/plug/magnification in cells and the
      pictures anchored on top; `images.rs` takes files and can read none of it — see the open item
      below. (2) The delivery states a MAGNIFICATION (`5x`/`10x`), not a field of view, so `fov_um`
      cannot be filled from it and everything dimensional stays correctly refused; some sheets carry
      a scale bar as a SEPARATE graphic beside the plate, which `scaleBarDialog.ts` cannot use.
      (3) The one that changed the code: `epoxy_check` was only half the guard. Median hue across
      the delivery ran 26–310 degrees; a blue-cast plate read **0.97 v/v**, a green-cast plate from
      the same core 0.06, and 28 plates came back above 0.5. `petrography::scene_dominated` refuses
      the WRITE (never the measurement — tuning needs the number) when the plate's own median hue
      falls inside the pore band. Caught every plate above 0.5; highest unflagged was 0.387. Stored
      range went 0.000–0.972 (median 0.231) → 0.000–0.387 (median 0.115). Flagged rows render in
      `var(--warn)` with the reason on hover; the run also reports the delivery's hue spread when it
      exceeds 60 degrees. The synthetic welded-grain fixture was 87% pore — a mount, not a rock —
      and is now grain-dominated.

### Open, from that run

- [x] **Import plates from a petrography workbook** — **SHIPPED 2026-07-31**.
      `images::probe_plate_workbooks` + `WORKBOOK_RUNNER`, wired into the existing Import pictures…
      wizard, which now accepts `.xlsx` directly. **An EXTRACTOR, not a second importer**: it writes
      the plates to a temp folder and hands them plus a depth table to `import_images`, so
      normalization, the Pillow cap, the set model, `follow_core`, `fov_um` and `prepared` all apply
      unchanged. **The depth comes from the sheet's own CELL and overrules any filename guess**, and
      only where a UNIT follows it — the header also carries the plate and plug numbers
      (`4633.50 FT/ 108`), so a bare number is a coin toss. The unit is the sheets' own and only
      when they all agreed; otherwise the wizard must ask. **A magnification is never converted into
      a field of view** (needs the camera sensor width and tube factor, which no delivery states);
      a sheet stating two attaches none. `MIN_PLATE_PX` (400, round, in pixels) drops the
      decorations anchored beside the plates — scale bars and letterheads at 117x59 and 207x79
      against plates of 1920x1080 — counted per sheet, never silently. **The old `.xls` is refused
      BY NAME with the fix** (Save As `.xlsx`), and it is the majority format here: its pictures can
      be scanned out, but the worksheet each belongs to — and the worksheet is where the depth is —
      needs a full BIFF parser, and a guessed association hangs a plate off the wrong sand. Real
      round trip `workbook_field_tests::plates_come_out_of_a_real_petrography_workbook`
      (`SANDIBUMI_FIELD_FIXTURES` + `workbooks/`, skips with a printed reason when unset): measured
      **152 plates from 2 real deliveries, every one with a depth, unit ft, 33 notes**.
- [ ] **Read the old `.xls` directly**, if the Save As step proves too tedious in practice —
      107 of the 165 petrography workbooks on the reference machine are that format. Needs a BIFF8
      Escher/OBJ walk to tie each embedded picture to its worksheet. **Ask before building**: the
      Save As route is five seconds per file and provably correct.
- [ ] **Magnification → field of view** would need a per-delivery camera sensor width and tube
      factor. Both are properties of the laboratory's microscope, not of the plate, so they are a
      declaration — ask whether that beats measuring a scale bar.
- [x] **Run the whole road end to end, and check it against an independent measurement** —
      **SHIPPED 2026-07-31**. `petrography::field_tests::a_delivered_book_measures_against_the_
      petrographers_own_point_count` drives workbook → plates → pore area → `plugqc` on a real
      delivery, checked against the petrographer's own point-counted visible porosity (the SAME
      picture, so only the measurement is under test — helium porosity would confound it with the
      depth registration). Fixture: `SANDIBUMI_FIELD_FIXTURES` with `workbooks/` and `petrography/`.
      **The measurement does NOT agree on this delivery**: 35 pairs, counted median 14%, measured
      median 6.8%, Pearson −0.300, Spearman −0.092, and no band in the 180–260…220–260 sweep moves
      either coefficient off zero. Cause: the plates' own median hue spans **289°** across one core
      and one report, and the rule tracks the cast (green-cast plate 0.04% against a counted 15%;
      blue-cast plate 31% against a counted 9%). **Within the colour-consistent blue-cast group with
      a band tuned to it: Pearson 0.643, Spearman 0.616 on 10 plates** — the method is sound and
      "measure a delivery in groups" is a real instruction. **Sharpest result: on the green-cast
      group a band can be tuned until the measured median matches the counted median (15.72 vs
      15.00) while per-plate rank agreement stays at −0.10** — tuning until the average looks right
      is exactly the wrong way to tune it.
- [x] **A vector plate book was importing as zero plates** — **SHIPPED 2026-07-31**, found by the
      run above. `openpyxl` DROPS WMF/EMF with a warning nothing downstream sees, so a book of 53
      sheets and 106 photomicrographs produced nothing and almost no notes (`ws._images` empty →
      `if not imgs: continue`). `WORKBOOK_RUNNER` now reads pictures from the PACKAGE (workbook →
      sheet part → drawing part → media part, every step an explicit relationship file; document
      order is anchor order) and leaves openpyxl to read the cells — removing the failure mode by
      construction rather than patching around it. `sniff` recognises EMF (` EMF` at offset 40, not
      the far-too-weak record type; `rclBounds` is inclusive) so a recovered plate is never called
      unreadable by the importer that just extracted it; without Pillow it says "EMF needs Pillow"
      by name. A worksheet holding no picture is counted and reported once per file. Measured:
      **258 plates where there had been 152**, 242 imported and measured. Pinned by
      `the_workbook_reader_takes_its_pictures_from_the_package_not_from_openpyxl` (fails if
      `_images` returns) and `an_enhanced_metafile_plate_is_recognised_rather_than_called_unreadable`
      (with the control that the record type alone is not enough).
- [x] **SHIPPED (2026-07-31) — one band, many lamps.** `PoreSpec.reference_image_id` names the
      plate the band was tuned on and every other plate is colour-corrected onto it before the band
      is applied: a per-channel (von Kries) gain putting each plate's matrix colour where the
      reference's sits, anchored on the delivery's own ROCK rather than on grey — grey-world would
      normalize away the porosity signal itself, since a blue-epoxy section is genuinely blue and
      the more porous the more so. The matrix colour is a channel-wise median, which is legal
      exactly where `scene_dominated` passes, so the guard and the correction hold each other up.
      The gain is scaled so no channel clips; the stain and the preview are read off the same
      corrected picture. A reference plate that is itself scene-dominated refuses the whole run by
      name. Verified by `the_same_rock_under_a_different_lamp_reads_as_the_same_rock`: a plate shot
      through a 2.0x-green / 0.55x-blue lamp reads under 1% uncorrected against its identical twin's
      25%, and the same quarter once corrected.
- [x] **SHIPPED (2026-07-31) — the empty-measurement refusal, conditionally** (Jauhar: "yes but
      conditional"). `band_missed` refuses a plate whose band claimed less than one resolvable
      pore's worth of pixels, **only on a normalized run**. That is where the condition comes from
      rather than from a threshold: without a reference there is no evidence the band finds epoxy
      anywhere in the delivery, so an empty answer might only mean it has never been tuned. "Empty"
      is the user's own `min_pore_px`, not a new constant. `cast_shift` (`hue_delta`, the short way
      round the wheel) rides beside every result as a diagnostic and is shown in the table; the
      column is hidden on an uncorrected run rather than shown empty. Pinned by
      `an_empty_measurement_is_refused_only_once_a_reference_plate_says_the_band_works` and
      `the_cast_shift_measures_the_short_way_round_the_colour_wheel`.
- [x] **SHIPPED (2026-07-31) — the anchor was the whole plate and had to be the matrix.** Running
      the point-count comparison on the real delivery showed every coefficient turning NEGATIVE
      with the correction on. Cause: the whole-plate median moves with how much epoxy is in the
      field of view, so anchoring on it partly cancels the porosity contrast — the grey-world trap
      reached by a different route. Now anchored on the pixels the band did not claim, resolved in
      one terminating iteration. Measured over 45 plugs with the two fields of view averaged: rank
      agreement 0.19 uncorrected, 0.05 whole-plate anchor, 0.20 matrix anchor; best of 57 bands
      0.25 against 0.15-0.36. Also closed a hole the fix exposed: a plate whose band claimed the
      whole picture has no matrix to anchor on, and was falling through to be stored at nearly 1.0
      — it is now the scene-dominance refusal. Pinned by
      `a_plate_corrected_onto_one_lit_the_same_way_is_left_alone`, whose fixture scatters pore
      through a gradient-lit frame so that it can discriminate the two anchors at all.
- [ ] **The colour cast is not only the lamp, and that caps what any correction can do.** The same
      delivery photographed two fields of view per plug; they agree with EACH OTHER at rank 0.85
      but differ in whole-plate median hue by 66 degrees at p90, and shifts of 180 degrees appear
      across the delivery. A white balance cannot do that — auto white balance on the camera would.
      Worth asking the laboratory before building anything further, because it is a setting rather
      than a re-shoot.
- [ ] **A colour band is not yet a substitute for a point count on this rock.** Repeatable (0.85
      between two views of one plug) but agreeing with the petrographer at only 0.10-0.27, so the
      gap is systematic. The likely mechanism is that a point count ticks VISIBLE pores under a
      grid while the rule counts every blue pixel including microporous haze — which would make
      this a difference in definition rather than an error. Needs Jauhar's judgement before more
      colour work is done.
- [ ] **Can ONE reference serve a 289-degree delivery?** The correction gets less exact the further
      a plate has to move, and nothing yet says how far is too far. Deliberately not invented: the
      answer has to come from Jauhar reporting the largest Shift on a plate whose preview still
      looked right. Until then the shift column and the preview are the judgement.
- [ ] **A colour rule over a greyscale SEM plate returns 0.000** — the mirror of the 0.97 case, and
      more dangerous because it looks like a tight rock rather than an absurdity. A delivery mixes
      thin sections, SEM plates and scale graphics in one folder. The obvious test (saturation) did
      NOT separate them on the real data (p99 saturation ≥ 0.34 on every plate, including the ones
      that are grey with a coloured annotation), so nothing shipped rather than a guessed threshold.
- [ ] **A point-count table need not carry its own total.** One delivered table left the *Total
      porosity* column EMPTY on every row with the six components filled in, and several component
      cells read `trace` — a word, not a number. The core-import wizard has no notion of "sum these
      columns", so the independent measurement had to be assembled by hand. Worth a mapping role if
      point-count tables are going to be a routine import.

## C3. Trust & reproducibility — Phase 11 (§3)

- **Audit trail & lineage**: every module/equation run and data edit logged (`runs` table: params, inputs,
  timestamps); any computed curve can answer "how was I made?" with its full ancestry. *(Log-set provenance
  from Wave A P1-c is a partial down-payment on this.)*
- **Interpretation scenarios**: named parameter sets; run the same chain under scenario A/B; diff view
  (curve overlay + per-zone stats delta).
- **Project operations**: autosave checkpoints, crash-safe WAL, merge wells from another project file.
  *(Autosave + WAL resilience already shipped; merge is the new part.)*
- **UX**: per-project workspace persistence, command palette (Ctrl+K).
- **Done when**: scenario A/B compare works end-to-end and lineage is visible for every curve.

## C4. Platform & extensibility — Phase 12 (§3): the finish line

- **User-defined Python modules**: a manifest (JSON) + Python script drops into a project `modules/` folder
  and appears in the ribbon with an auto-generated dialog — your personal Loglan library, shareable as
  plain files. *(Overlaps the §4 New-capability "Plugins" item.)*
- **Native DLIS** (replace the dlisio bridge if it ever limits), LAS 3.0, WITSML later.
- **Distribution**: Tauri installer + auto-update, bundled sample project, in-app method help per module (F1).
- **Long game (demand-driven)**: NMR T2 (array_logs table already exists), borehole images, geomechanics,
  production logs. *(NMR/images/geomech now have dedicated Wave C/D specs above.)*
- **Done when**: a colleague installs SandiBumi from an installer, imports a DLIS, runs your shared Python
  module, and exports a PDF report — zero developer tools involved.

## C5. New-capability misc (§4)

_(field-review tier, was "P3"; longer-horizon items not already absorbed by a Wave above.)_
- **SandiMin**: optional nonlinear Sw equation in the solve loop (iterate to convergence —
  Indonesia/Simandoux-style inside the inversion, not just the dual-water CT row).
- **Monte Carlo "finalize → print to curves"**: write LOW / BASE / HIGH curves from the chosen result
  percentiles (named by result value, not optimist/pessimist case, computed internally).
- **Plugins (Advance ribbon)**: user-authored Python and Loglan modules — variable declarations + code,
  manifest-style, shareable as files. *(= Phase 12 user-modules.)*
- **2D Window (new ribbon)**: lateral analysis — per-well X/Y (directional-survey-corrected at each marker),
  thickness & weighted-property maps per marker interval, category-based maps, contours + wells posted with
  Z-value gradient; later fluid contacts and simple volumetrics. *(Fluid-contacts part → Wave B item 9.)*
- **Panes independent of windows**: any pane floatable/resizable like a window; windows become pure grouping
  containers.
- **Data tools (separate, later per Jauhar)**: log digitization, core image input (→ Wave D item 7),
  XRD/petrography digitization.
- **User guide PDF**: topic-by-topic, step-by-step with screenshots from the real app using Jauhar's
  database as the worked example; include the outstanding review items as an appendix. Produce against the
  current build or on request.

---
---

# Reference

## R1. Where SandiBumi already matches — or beats — the reference suite (§1)

| the reference suite module | SandiBumi equivalent | Notes |
|---|---|---|
| Layout | Log View panels (WebGPU) | GPU-rendered, curve fills, synchronized crosshair across views, per-panel layouts, saved layouts. Faster pan/zoom than the reference suite's redraw. |
| Frequency | Histogram panel | Bars/line/cumulative, selectable statistic chips, normalization, click-to-pick → zone params. |
| Xplot | Crossplot panel | Z-coloring, matrix points, least-squares regression + R² in lin/log space, click-to-pick. |
| PT04_ParameterPick | Histogram/Crossplot/Pickett picks | Picks write directly into zone parameters. |
| PT05_Determin | Deterministic module library | Ported Loglan modules (vsh, porosity, sw, perm, prep/ftemp), manifest-driven dialogs, multi-well parallel runs. |
| PT13_PaySummary / PaySummary / PayModel | Cutoffs & Summary | Cutoff/lumping engine with FLAG_* curves and per-zone HPV table. |
| Loglan / RF05_TclTk | **Python + numpy equations** | Vectorized numpy beats Loglan for expressiveness; no compile step; NaN-native. Rhai kept for legacy. |
| Text | Database Inspector | Editable grid over every table, paged, **with undo (Ctrl+Z) — the reference suite has no undo there**. |
| RF06 SQL / Query | **SQL Query panel** | Full DuckDB SQL (joins, window functions, aggregates) vs the reference suite's narrow SQL dialect. Clear superiority. |
| FileImporter/FileExporter (LAS) | Import LAS / Export LAS | LAS 2.0 both directions incl. computed curves. |
| RF01_Database | DuckDB single-file project | Columnar, transactional, one file to copy = whole project ("Save Project As"). |
| WellCatalog / WellInventory | Wells & Tops panel | Object tree + tops; inventory columns can come from SQL panel. |
| AuditTrail | log sets + undo stack | Session undo + versioned log sets with provenance; full lineage is Phase 11. |
| GS5 shortcuts / docking | dockview workspace | Float/dock/tab/split/maximize any panel — more flexible than the reference suite's MDI. |

## R2. Standing advantages to protect (§5)

- One-file DuckDB project + full SQL access.
- Python/numpy as the scripting surface (every new feature should expose its data to it).
- Undo everywhere a cell or property changes.
- GPU log rendering + synchronized cursor.
- Manifest-driven module dialogs (new module = Rust fn + manifest, zero UI code).

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
