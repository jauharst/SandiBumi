# SandiBumi — Development Roadmap

Grounded in the the reference suite V14 helpset catalog (`C:\Program Files\AspenTech\the reference install\doc\helpset`,
~120 module help books) and the original Techlog-style UX redesign plan. **Restructured
2026-07-20 by status** — what's Done, what's Open, what's Future — so it's readable at a
glance. (The earlier standalone UI/UX redesign plan `jolly-skipping-dove.md` is folded in and
fully superseded by this document.)

## How to read this

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
- **Performance** (§4b): crossplot redraw memoize (#127) ✅, **batch curve reads (#130)** ✅ **persistent Python worker (#132)** ✅ and **raw-IPC ArrayBuffers (#131)** ✅ **shipped + committed 2026-07-21**. Remaining: async commands (#128) and connection pool [**high-risk**] (#129) both need a live 100-well run to sign off.
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
  explicit discovery order (`ARSHILLA_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python31x`
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
  Override-count badge + Reset. Persists in the `workflow` document. **Remaining**: per-well parameter
  override table (→ [Part B](#b4-carried-forward-deferrals-from-the-build-arc)).
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

Cross-cutting notes: (11) Balam South testing is the per-increment verification standard, not a separate
item. New suites must land as panes (Wave A first), use the 15-var theme contract, manifest-driven
dialogs where they fit, and expose outputs to Python/SQL per §5.

---
---

# ◻ PART B · OPEN (do next)

The actionable backlog. Roughly ordered: safe frontend wins first, then Performance (which needs a live
100-well run to sign off), then the Wave B feature suites, then carried-forward deferrals.

## B1. Hardening backlog (§4b)

**Performance (was "P2") — speed at field scale (100+ wells)** — all 6 mapped by a read-only
investigation wave (file:line + risk). **4 of 6 shipped: #127 (crossplot memoize), #130 (batch curve
reads), #131 (raw-IPC ArrayBuffers), #132 (persistent Python worker).** Tasks #127–132. The remaining connection-semantics items are architecturally invasive —
they change DB connection semantics and **cannot be signed off without running `tauri dev` on 100+
real wells** (the human can't be replaced for perf benchmarking).
- [ ] **(#128)** Long commands are synchronous Tauri commands — `run_workflow_chain`/`run_ml`/`run_multimin`
      are sync `fn` on the IPC thread, so a chain run blocks IPC for minutes and Cancel can't fire until it
      finishes. Move to async + spawn_blocking + progress events. Interacts with the pool item below (the
      global `Mutex<Connection>` still serializes spawn_blocking on the lock).
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
      **Follow-up, the one real approximation left: a true chart-lookup DN crossplot porosity.**
      OPT_PHIA=XPLOT averages the apparent-limestone density and neutron porosities analytically,
      which drags points toward the assumed RHO_MA_A: quartz lands ~0.013 g/cc heavy (UMAA within
      0.001 of the chart) but dolomite ~0.06 g/cc light and ~0.34 b/cm³ left of its chart point —
      20% of the shortest triangle edge, so minerals never cross over, but the bias is real. The fix
      is a genuine Por-11-style 2-D lookup (solve for the matrix/porosity pair that satisfies BOTH
      tools); `neutron_charts.rs` already holds the digitized per-matrix neutron tables, so the
      remaining work is the density leg plus a root-find. Until then OPT_PHIA=LOG (feed a trusted
      porosity) is the accurate route.
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
- **Per-well parameter override table** in the Workflow Builder (Phase 9-2).
- **Monte Carlo** (Phase 9-3): per-zone parameter distributions (currently well-wide); persisted
  P10/P50/P90 *curves*. *(Plus the §4 New-capability "print LOW/BASE/HIGH curves" item in [C5](#c5-new-capability-misc-4).)*
- **Full-field responsiveness** (Phase 9-5): lazy catalog loading, decimation cache, keep the UI responsive
  during full-field runs, 2000-well synthetic stress fixture (100-well is the current proof).
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
