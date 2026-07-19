# SandiBumi — Complete Roadmap

Grounded in the Geolog V14 helpset catalog (`C:\Program Files\AspenTech\Geolog-V14\doc\helpset`,
~120 module help books) and the original Techlog-style UX redesign plan. Updated 2026-07-17.
This is the single current roadmap — the earlier standalone UI/UX redesign plan
(`jolly-skipping-dove.md`) is folded into §0 below and fully superseded by this document.

## 0. History — Phases 1–5 (shell, plots, data management, equations) — DONE

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
  coloring matches (/FACIES/ regex), and FACIES_GMM can use the blocks track. Still
  deferred: supervised facies on core (Python subprocess + scikit-learn), field-wide
  pooled clustering for globally consistent labels.
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
  (fits POOLED apply wells → **field-wide, globally consistent ids**, closing that deferral):
  K-Means, GMM (+`_PROB`), hierarchical, DBSCAN (noise → NaN); ids ordered by first-feature
  mean like the native facies modules. Reduction: PCA (PC1…PCn + explained variance), t-SNE
  (TSNE1/2, 20k-sample cap). Supervised tasks pool labelled samples from train wells and
  predict on apply wells; incomplete rows are masked and come back NaN. Metrics (5-fold CV
  R²/accuracy, silhouette, class/cluster counts) surface in the dialog. Autoencoders
  deferred (needs PyTorch). Supervised facies on core = classification with target FACIES/
  core lithology — no longer a separate deferral.
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

## 1. Where SandiBumi already matches — or beats — Geolog

| Geolog module | SandiBumi equivalent | Notes |
|---|---|---|
| Layout | Log View panels (WebGPU) | GPU-rendered, curve fills, synchronized crosshair across views, per-panel layouts, saved layouts. Faster pan/zoom than Geolog's redraw. |
| Frequency | Histogram panel | Bars/line/cumulative, selectable statistic chips, normalization, click-to-pick → zone params. |
| Xplot | Crossplot panel | Z-coloring, matrix points, least-squares regression + R² in lin/log space, click-to-pick. |
| PT04_ParameterPick | Histogram/Crossplot/Pickett picks | Picks write directly into zone parameters. |
| PT05_Determin | Deterministic module library | Ported Loglan modules (vsh, porosity, sw, perm, prep/ftemp), manifest-driven dialogs, multi-well parallel runs. |
| PT13_PaySummary / PaySummary / PayModel | Cutoffs & Summary | Cutoff/lumping engine with FLAG_* curves and per-zone HPV table. |
| Loglan / RF05_TclTk | **Python + numpy equations** | Vectorized numpy beats Loglan for expressiveness; no compile step; NaN-native. Rhai kept for legacy. |
| Text | Database Inspector | Editable grid over every table, paged, **with undo (Ctrl+Z) — Geolog has no undo there**. |
| RF06_GeologSQL / Query | **SQL Query panel** | Full DuckDB SQL (joins, window functions, aggregates) vs GeologSQL's narrow dialect. Clear superiority. |
| FileImporter/FileExporter (LAS) | Import LAS / Export LAS | LAS 2.0 both directions incl. computed curves. |
| RF01_Database | DuckDB single-file project | Columnar, transactional, one file to copy = whole project ("Save Project As"). |
| WellCatalog / WellInventory | Wells & Tops panel | Object tree + tops; inventory columns can come from SQL panel. |
| AuditTrail | (partial) undo stack | Session undo exists; a persistent audit log would be a small addition on `documents`. |
| GS5 shortcuts / docking | dockview workspace | Float/dock/tab/split/maximize any panel — more flexible than Geolog's MDI. |

## 2. Priority gaps — status

- ~~PT09_ThinBeds (Thomas-Stieber)~~ **DONE** — `thin_bed_ts` module (VLAM/VDISP/VSAND/PHIE_LAM).
- ~~Correlation view~~ **DONE** — multi-well strips, tops connectors, flatten on datum.
- ~~SpliceLogs + depth shift~~ **DONE** — `depth_shift` (zone-overridable block shift) +
  `splice` modules; undoable core-to-log "Shift Core…".
- ~~PT12_CoreAnalysis~~ **DONE** — core CSV import (percent→v/v, alias headers), crossplot
  + log-track overlays, inspector editing.
- Everything still open is folded into the master plan below.

## 3. Master plan — Phases 6–12 (to best-in-class)

Phases 1–5 built the shell, plots, data management, and the equation/module engines.
Each phase below is independently shippable, sized like Phases 1–5, and ends with:
`cargo test` + `tsc` green, a browser functional test, benchmark fixtures vs Geolog
output where applicable, and a click-through on real Balam/Minas/Mahakam data.

### Phase 6 — Data foundation: arbitrary curves, units, TVD

*Why first: `standard_curves` is hard-coded to 6 mnemonics (GR/RES/NPHI/RHOB/DT/SP).
PEF, CALI, DRHO, RXO, multiple runs, arrays — none can even be imported. This blocks
multimin (needs PEF), environmental corrections (needs CALI), bad-hole QC, and DLIS.*

- ~~**Database (6a, DONE 2026-07-17)**~~: generic curve store shipped as an **additive**
  layer alongside `standard_curves` (not a replacement yet, to avoid a risky one-shot
  rewrite of every read path) — `curve_meta(curve_id, well_id, set_name, mnemonic, unit,
  family, source, run_no)` + `curve_samples(curve_id, depth, value)`, curve **sets**
  `RAW`/`EDIT`/`FINAL`, `well_path(well_id, md, inc, azi, tvd, tvdss)` for deviation
  (schema only — TVD calc is 6b). `migrate_standard_curves_to_generic_store` runs on
  every launch, idempotently backfilling GR/RES_DEEP/NPHI/RHOB/DT/SP into the generic
  store as set RAW with real units; `upsert_curve_meta`/`insert_curve_samples`/
  `get_curve_samples`/`list_generic_curve_catalog` are the read/write API (`db.rs`), wired
  to new IPC commands `list_generic_curve_catalog`/`get_generic_curve_samples`. Nothing
  that currently reads `standard_curves` was touched — this only proves the store works
  and gives 6b a real place to write new curves into. 18 Rust tests pass (added
  `generic_store_migration_and_manual_curve`, incl. an idempotency check and a NaN-vs-NULL
  fix: DuckDB's `IS NOT NULL` is true for NaN, so the migration's "does this column have
  real data" check needed `AND NOT isnan(col)`).
- ~~**6b (mostly DONE 2026-07-17)**~~ — **Backend**: LAS import now keeps **every** curve.
  `ingest::import_all_curves_into_generic_store` (called after the legacy standard-curve
  insert, failure is non-fatal) re-reads the file with the new `parsers::parse_las_2_all`
  (streams all `~C` curves + units, not just the fixed 6) and writes each into
  `curve_meta`/`curve_samples` as set RAW. Mnemonic dictionary + unit conversion live in
  the new `curves.rs` (`family_for`, `convert_to_canonical` — us/m→us/ft, kg/m³→g/cc,
  pu/%→v/v, mm/cm→in; families GR/SP/CALI/BS/RHOB/DRHO/PEF/NPHI/DT/DTS/RES_*/RXO). Deviation
  survey import + minimum-curvature TVD/TVDSS in the new `deviation.rs` (+ `parse_deviation_csv`,
  `db::insert_well_path`/`get_well_path`, IPC `import_deviation_csv`/`get_well_path`).
  the end-to-end LAS-with-PEF/CALI/metric-sonic import + vertical/deviated TVD check.
- ~~**DLIS import via `dlisio` (DONE 2026-07-17)**~~: new `dlis.rs` runs `dlisio` through
  the Python subprocess — a helper script streams every scalar channel of every frame as a
  JSON header + raw f32 depth/value columns; Rust writes them into the generic store as set
  RAW, family-tagged + unit-canonicalized (frame ordinal → `run_no`). `dlisio 1.0.4`
  installed into the SandiBumi Python env. IPC `import_dlis_file`. Real-file test is
  `--ignored` (gated on `ARSHILLA_TEST_DLIS`); the `read_f32` round-trip + the runner's
  dlisio-import/bad-path path are covered/smoke-tested.
- ~~**6c — Frontend (DONE 2026-07-17)**~~: Curve Catalog now shows the generic store per
  selected well (mnemonic/unit/family/set/source/samples columns + a live text filter over
  all of those, with a "· run N" badge), backed by `list_generic_curve_catalog`, falling
  back to the legacy standard+computed view when no well is selected. Data ribbon gained
  **Import DLIS…**, **Import Deviation…** (datum/KB prompt → minimum-curvature TVD/TVDSS),
  and **Well Header…** (field/TD/KB editor). Browser-verified: all three ribbon buttons
  render; the catalog filter narrows a 4-curve synthetic set to the one CALI curve by
  *family* match ("HCAL · run 1"). tsc + cargo green.
- **Deferred to a later pass** (not blocking — the store is populated and surfaced):
  rewiring `get_track_data` (the **log-view** read path) to *read from* the generic store —
  log views still read `standard_curves`, so PEF/CALI aren't drawable in a track yet. (The
  **module/equation** input path `fetch_curve_frame` DOES now fall back to the generic store,
  done in Phase 7 — that's what unblocked multimin/bad-hole.) Also still deferred: a curve-set
  selector in the layout picker, and the optional TVD depth scale in the log/correlation views
  (its `deviation::tvd_at` + `LasFrame.depth_unit` plumbing is already built and
  `#[allow(dead_code)]`-tagged).
- **Done when**: a real 30+ curve LAS (and a DLIS) imports whole ✓; PEF/CALI in the catalog ✓;
  TVD matches hand calculation ✓; every existing feature still green ✓.

### Phase 7 — Interpretation physics II (the Mahakam pack)

- ~~**Generic-store read fallback (DONE 2026-07-17)**~~ — the deferred Phase 6 unblocker,
  pulled forward because multimin needs PEF and bad-hole needs CALI, and both live only
  in the generic store. `equations::fetch_curve_frame` now resolves any non-standard,
  non-computed curve name from `curve_meta`/`curve_samples` (set RAW) via a new
  `fetch_named_curve_aligned` → `fetch_generic_curve_aligned`, matching on mnemonic first
  then family (so a module asking for "CALI"/"PEF"/"DRHO" finds an HCAL/PEFZ/HDRA curve by
  family), preferring the base run. Additive — log views still read `standard_curves`; this
  only widens what modules/equations can take as input.
- ~~**Bad-hole QC (DONE 2026-07-17)**~~ — `badhole` module (`modules.rs`, Prep category):
  BADHOLE = 1 where |DRHO| > DRHO_MAX or (CALI − bit size) > DCAL_MAX (bit size from BS
  curve or BS_DEF), 0 in good hole, MISSING with no QC curve. **Central mask capability**
  in the runner: any module run passing `opts["MASK"] = "<flag curve>"` gets flagged
  samples (==1) NaN'd out of every output — zero per-module code, mask resolved generic-
  store-aware. UI: one universal "Mask (optional)" picker in the auto-generated module
  dialog (`moduleDialog.ts`) feeding `opts.MASK`, default (none).
- ~~**Multimin (PT07, DONE 2026-07-17)**~~ — `multimin.rs` (kept separate from the existing
  async-job `inversion.rs`): constrained weighted least-squares 4-component inversion
  (SAND/CLAY/WATER/HC) from RHOB/NPHI/DT/PEF (any subset present), non-negative volumes via
  a hand-rolled Lawson-Hanson **NNLS** with a heavily-weighted unity row as a soft
  constraint; each tool equation scaled by 1/sigma. Endpoints + sigmas are params. Outputs
  VOL_SAND/CLAY/WATER/HC, PHIT_MM, VSH_MM, SWT_MM, RECON_ERR (RMS residual in sigma units).
  Wired as a module (auto-generated dialog). Verified: recovers a forward-modelled 70/30
  clean wet sand within 2 %, and the runner integration test proves PEF is read from the
  generic store end-to-end. 30 Rust tests pass.
- ~~**Generalized Multimin — Increment A (DONE 2026-07-19)**~~ — `multimin2.rs` (separate from
  the fixed `multimin.rs` above): ELAN-style **N user-defined** minerals/fluids from an editable
  15-entry library (quartz…halite + water/oil/gas) against any subset of RHOB/NPHI/DT/GR/PEF/U,
  with **hard** unity (Σv=1) + non-negativity via equality-constrained active-set NNLS over the
  probability simplex. Bespoke command `run_multimin`/`multimin_library`; dedicated **Advance →
  Multimin…** dialog (editable endpoint matrix + per-component Clay/Poro/Water roles). Outputs
  VOL_<comp> + `<prefix>`_PHIT/VSH/SWT/RECON. 4 solver tests (2 % recovery, exact unity, boundary
  non-neg, library) + full suite pass.
- ~~**Generalized Multimin v2 — Geolog parity (DONE 2026-07-19)**~~ — spec extracted from the local
  Geolog-V14 Multimin helpset + IP2018 Mineral Solver (docs/multimin_geolog_spec.md,
  docs/multimin_ip_spec.md). **27-component library** in IP dropdown order (12 minerals, 6 clays
  with CEC, 7 zone-typed fluids Sxo/Sw/BoundWater), **16 input logs** (density, neutron, sonic, PEF,
  U, total+spectral GR, Vp/Vs, CT, CXO, EPT, EATT, Sigma) **+ user-defined inputs**. Resistivity
  enters as conductivity via the **dual-water linear transform** (Ct^(1/w) row, w=0.75m+0.25n; CT
  sees the unflushed zone, CXO the flushed) — Sw/Sxo now come out of the volume solve itself
  (supersedes the old Increment-B outer-loop design). Geolog program constraints: hard unity over
  minerals+U-fluids, POROSITY (ΣX=ΣU) and BNDWAT (96·CEC·ρ/(T°C+298)·α, matches Geolog's 0.1841
  Illite multiplier) as soft σ=0.01 rows, WATER MUD re-solve, hard per-component bounds (fluids
  ≤0.5). New `solve_bounded_lsq` (free/at-0/at-hi active set + unity KKT), `multimin_fluid_calc`
  preview command, rebuilt Geolog-style dialog. 7 solver tests incl. Sw=0.40/Sxo=0.80 recovery
  from CT/CXO; 84/84 suite; tsc clean; browser-verified.
- ~~**Saturation-height (PT11, DONE 2026-07-17)**~~ — `scal_pc` table (Pc/Sw points with
  per-plug perm/poro; replace-on-reimport like core_data) + **Import SCAL…** (Data ribbon;
  alias headers, percent Sw/poro auto-detected) + `satheight.rs`: `fit_leverett_j`
  (Sw = A·J^B by log-log LSQ at the lab sigma·cosθ; A/B/R²/n reported straight back in the
  import dialog) and the `sw_height` module — LEVERETT (Pc = 0.433·Δρ·h_ft → J → SWH,
  needs PERM) or SKELT (Skelt-Harrison 1 − A·exp(−(B/(h+D))^C), no perm needed); SWH = 1
  at/below the zone-overridable FWL; outputs SWH + HAFWL. *Not built*: Skelt-Harrison
  auto-fit (params are manual) and the Pc/J-vs-Sw QC plot (`get_scal_pc` IPC is ready).
  *Caveat*: FWL/height work in the well's native depth (MD) until the TVD scale lands.
- ~~**Environmental corrections (PT03, DONE 2026-07-17, pragmatic analytic)**~~ —
  `gr_hole_corr` (GR·(1+K_GR·enlargement)), `nphi_env_corr` (linearized temperature +
  salinity terms; temperature term needs FTEMP), `rhob_hole_corr` (upward beyond HD_REF)
  as Prep modules; coefficients are params at chartbook magnitudes; a missing QC curve
  passes the log through uncorrected. Chart-lookup fidelity stays future work.
- ~~**Thomas-Stieber interactive crossplot (DONE 2026-07-17)**~~ — "T-S triangle" checkbox
  on the crossplot panel (meant for X=VSH, Y=PHIT): laminated line, dispersed line down to
  the pore-filling minimum at VSH = PHI_SD, and **draggable endpoint handles** — sand
  handle (VSH=0) sets PHI_SD_MAX, shale handle (VSH=1) sets PHI_SH, written to the selected
  zone's params on drag release (feeding `thin_bed_ts`). Drag swallows the click so it
  doesn't double as a point pick; endpoints persist with the plot properties.
- **Done when**: multimin volumes on a benchmark well match Geolog within tolerance ✓ (unit);
  SWH tracks core Sw (pending field click-through, `REVIEW.md`); corrections change curves
  in the right direction ✓ (unit-tested direction + magnitude; field check pending).
  37 Rust tests + tsc green.

### Phase 8 — Deliverables: composite plots & PDF reports

*This is what clients and partners actually see — the LQR deliverable.*

- ~~**Composite plot designer + vector export (8a, DONE 2026-07-17)**~~ — `composite.rs`
  renders a `Layout` at a TRUE print scale (1:200/500/1000) into a backend-neutral list of
  `DrawOp`s (mm space), then serializes to **SVG** (screen preview + export) or a
  **dependency-free multi-page PDF** (hand-rolled writer, base-14 Helvetica so no font files
  are embedded — chosen over `svg2pdf`/`usvg` to avoid a heavy font-DB dep tree on the
  already-large bundled-DuckDB build). Page 1 carries the full header block (well/field/TD/KB,
  layout, scale, interval); later pages a running header. Depth axis with nice-stepped
  major/minor grid + labels, per-track frames + linear/log vertical grids + min/max scale
  annotations, curve polylines with NaN/off-page breaks, Techlog-style edge fills (alpha in
  SVG; blended-to-white in PDF), formation-top lines + labels, alternating zone bands. Page
  splitting is exact (page 1 shorter for the header). Curve data comes through
  `fetch_curve_frame`, so standard/computed/generic-store curves all render. IPC:
  `render_composite` (per-page SVG + metadata), `export_composite_svg` (one file per page),
  `export_composite_pdf`. UI: Plot ribbon "Composite…" → dialog with layout/scale/page-size/
  depth-range controls, in-dialog page preview with prev/next, Save SVG… and Save PDF….
  Verified: SVG renders in-browser with a curve path exactly 233 mm tall (= 46.6 m × 5 mm/m
  at 1:200, print scale physically exact); PDF is structurally validated (all xref offsets
  resolve, 2 pages, valid trailer, Tj/re operators, Helvetica). 42 Rust tests + tsc green.
- **Deferred — hatch lithology/facies track + text/arrow annotations** in the composite
  (needs a facies curve → Phase 10, and an annotations store on `documents`).
- ~~**Report generator (8b, DONE 2026-07-18)**~~ — `report.rs` reuses the 8a DrawOp/PDF
  machinery: cover page → methodology parameter–method–remarks table (editable, persisted
  as a `report_template` document; default reflects Jauhar's standard workflow) → per-zone
  parameter table (zone_params) → pay summary table (run_pay_summary cutoffs) → composite
  pages, as one PDF. Paginated word-wrapped tables with repeated header rows. IPC:
  `render_report` (SVG preview), `export_report_pdf`, `export_report_batch` (one PDF per
  well into a folder), `save_png` (frontend-rasterized page PNG for slides). UI: Plot
  ribbon → Deliverables → Report… (`reportDialog.ts`).
- **Deferred from 8b**: histogram/crossplot pages in the report, per-formation narrative
  text, bilingual headings, executive-summary page, SWHF section, correlation-panel export.
- **Done when**: one command produces a client-ready multi-page PDF for a Balam well —
  composite PDF ✓ (8a); full templated report ✓ (8b).

### Phase 8.5 — Jauhar method suite (DONE 2026-07-18)

*His own field-proven methods as first-class core modules, studied from the 7 reference
projects (LQR Balam South, Glagah Kambuna, Wanda Gita, Bunga Block, LRLC research, KKT,
BLSO). Math banked in auto-memory (`method-ssc-sspw-lqr`, `method-lrlc-imts-rtc`,
`method-workflow-standards-jauhar`).*

- ~~**`ssc` (Porosity)**~~ — full port of `ssc_lqr_gap_edit_jau.lls` (Kuttan/GAP 2023 SSC):
  gas conditioning, N-D projection onto the dry rock line, sand/silt/clay fractions,
  PHIT from mixed matrix density, CBW/CWSH bound-water split, SWIRR, GR-equivalent
  volumes. Deterministic replacement for `RANNORMAL`; NPHIMA limit bug in the Loglan
  fixed deliberately (noted in the module header).
- ~~**`sspw` (Porosity)**~~ — PHR-standard sandstone workflow; exec reconstructed from the
  `.info` spec (body not on disk) — **validate vs Geolog "LAS PHIT PHIE" outputs**.
- ~~**`sw_rtc` + `sw_imts` (Saturation)**~~ — the LRLC research models: excess-conductivity
  correction (0.45·CAPBW + 0.0057·Qv − 0.0071, RSF 2.25) and iterative
  mineral-textural-scaled Waxman-Smits with Qv_eff = Qv_bulk/(1−Swirr), Juhasz B(T,Rw).
- ~~**`gr_normalize` (Prep)**~~ — two-point percentile GRN, Rokan reference defaults
  P3 = 53.68 / P97 = 133.93 gAPI.
- ~~**`log_predict` (Prep)**~~ — Facimage-MRGC-style synthetic logs by leave-one-out
  distance-weighted KNN, with the MAX_RAW washout rule for RHOB.
- ~~**Mnemonic dictionary enrichment**~~ — Bunga standardization table merged into
  `curves.rs` FAMILIES (ROBB/SBD2/HDRA/FSTP/ATR/BDAV/RING/PSR/R25P/BSAV/SN/HORD/PEB/DT_S…).
- **Deferred**: SSC-in-multimin presets (Wanda Gita style), variable-m carbonate (SPI)
  module (Bunga), per-zone multimin component presets (KKT), FZI rock typing module.

### Phase 9 — Field scale: batch workflows, uncertainty, dashboards

- **Workflow chains** ✅ (2026-07-18, increment 1): `chain.rs` runs an ordered list of
  modules across many wells — steps sequential (later steps consume earlier outputs),
  wells rayon-parallel per step via `run_workflow_module`. Progress + cancellation via a
  pollable registry (same pattern as `inversion.rs`, not Tauri events): frontend supplies
  the job id, calls `run_workflow_chain`, polls `get_chain_status`, `cancel_workflow_chain`
  flips a shared flag checked between steps. Chains persist as `workflow` documents.
  Frontend: Workflow Builder (`workflowDialog.ts`, Petrophysics → Batch). Steps run at
  manifest defaults, overridden per zone by `zone_params`.
- **Per-step parameter editing** ✅ (2026-07-18, increment 2): each step in the builder has
  an expandable ⚙ editor (manifest-driven, like the module dialog) — input-curve selectors,
  option dropdowns, validated numeric params, and the universal bad-hole Mask. Only values
  that differ from the manifest default are stored on the step (untouched step = empty maps
  = pure manifest + zone_params behaviour), and `zone_params` still override these whole-well
  values per zone at run time (step param only shifts the base). An override-count badge marks
  customized steps; Reset clears them. Persists in the `workflow` document. **Remaining**:
  per-well parameter override table.
- **Monte Carlo uncertainty (PT06)** ✅ (2026-07-18, increment 3): `montecarlo.rs` — put
  normal/uniform/triangular distributions on any model parameter, run N seeded realizations of
  a chain, get P10/P50/P90 net pay / NTG / avg PHIE / avg SWE / HPV **per zone** + an HPV
  histogram. Runs **entirely in memory** (`run_module` returns curve vectors; nothing writes
  `computed_curves`), so it sidesteps the field-scale write bottleneck — 1000 realizations
  finish in well under a second. Realizations are rayon-parallel, each seeded from
  `(seed, index)` for reproducibility. UI: Petrophysics → Batch → **Monte Carlo…**
  (`monteCarloDialog.ts`) — pick a saved chain (or the default VSH→φ→Sw), add uncertain
  parameters (candidates auto-derived from the chain's module params), set cutoffs/iterations,
  run → results table + theme-aware HPV histogram with P10/P50/P90 markers.
  **Deferred**: per-zone parameter distributions (currently well-wide) and persisted
  P10/P50/P90 *curves*.
- **Field dashboard panel** ✅ (2026-07-18, increment 4): `dashboardPanel.ts` — a dock panel
  (Petrophysics → Batch → **Field Dashboard…**, also on the ＋ add-panel menu) that runs the
  existing `run_pay_summary` engine across **every** well at chosen VSH/PHIE/SWE(/PERM) cutoffs,
  then shows: a **per-zone aggregation** table (well count, Σ net, Σ HPV, mean N/G, net-weighted
  mean PHIE/SWE), **per-zone box plots** (min/Q1/median/Q3/max, inline theme-aware SVG) for a
  selectable metric (PHIE/SWE/N-G/HPV/Net), and a **sortable** multi-well × zone interval grid,
  filterable by flag level (PAY/RESERVOIR/SAND), with **CSV export**. Frontend-only — no new
  backend; reuses the pay-summary command (which writes FLAG_* curves as a side effect, same as
  Cutoffs & Summary). Browser-verified with synthetic data (grid/aggregation/box plots render,
  column sort toggles asc/desc). *Caveat*: because it drives `run_pay_summary` per well, a
  full-field compute incurs the known `computed_curves` write cost — the perf-hardening
  increment below addresses that.
- **Performance hardening — write path** ✅ (2026-07-19, increment 5): killed the
  `computed_curves` write bottleneck. Root cause (proven by the in-harness probe in
  `pipeline_blso_test.rs`) was the 3-column `PRIMARY KEY (well_id, depth, curve_name)` — its
  ART uniqueness index cost ~3.4× per inserted row (468k vs 1589k rows/s). **Dropped the PK**
  (`db.rs` schema + `migrate_drop_computed_curves_pk`, which rebuilds the table PK-less on
  launch for existing projects, idempotent via `duckdb_constraints()`); uniqueness is now
  guaranteed by the write discipline (`write_computed_curves_batch` DELETEs a well's target
  curve names before appending; point-updates UPDATE in place — no path inserts a duplicate).
  Also **batched** each well's whole module output into one DELETE + one Appender/flush
  (`equations::write_computed_curves_batch`, used by `workflow.rs`) instead of one cycle per
  curve. Net: the real 100-well × 4-module chain dropped from ~50s to **21s** end-to-end
  (~2.3×; compute/reads/single-writer lock don't speed up). 72 Rust tests (2 new: PK-drop
  migration preserves rows + is idempotent; batch write overwrites-not-duplicates + keeps
  point-update working). **Still open** (non-write, deferred): lazy catalog loading, a
  decimation cache, keeping the UI responsive during full-field runs, and the 2000-well
  synthetic stress fixture (100-well is the current proof).
- **Done when**: a 100-well chain runs with live progress in minutes ✅ (21s); MC with 1000
  realizations per well finishes in seconds ✅ (in-memory, Phase 9-3).

### Phase 10 — Facies & assisted interpretation (pull ahead of Geolog)

- **Electrofacies (PT15)** — unsupervised k-means shipped ✅ (2026-07-18, increment 1):
  `facies.rs` `electrofacies` module (new **Facies** ribbon category). Runs per well through
  the standard module framework (whole-vector, like `log_predict`): up to 5 input curve slots
  (GR required; RHOB/NPHI/DT/SP optional — an absent curve drops that feature dimension),
  each z-scored by default (OPT_STANDARDIZE=NONE to skip), then k-means++ (dependency-free
  SplitMix64 seed, best-of-8 restarts, empty-cluster reseed to worst-fit point) partitions the
  complete samples into K facies (2–12). **Cluster labels are reordered by the ascending mean
  of the first supplied curve** (usually GR), so FACIES 0 is the cleanest class and numbering is
  monotone in shaliness — giving approximate cross-well comparability despite per-well
  clustering. A sample missing any present curve → FACIES = MISSING. Deterministic for a fixed
  seed. Output: FACIES (integer 0..K-1), written to `computed_curves` like any module (mask +
  chain + zone_params machinery all free). Frontend QC: **crossplot categorical coloring** —
  discrete curves (name matches FACIES/CLUSTER/LITHO/CLASS, or values look like small integers)
  now get a fixed qualitative palette (`FACIES_PALETTE`, Tableau-10-ish, wraps past 12) + a
  swatch legend instead of the continuous ramp (`plotCanvas.ts`: `faciesColor`,
  `categoricalColors`, `looksDiscrete`, `distinctValues`). 4 Rust unit tests (separation,
  MISSING propagation, determinism, GR-ordering) + tsc green; browser-verified a 3-facies
  NPHI-RHOB cloud renders with correct palette + F0/F1/F2 legend.
  **Deferred to increment 2**: the dedicated **colored FACIES block track** in the log view
  (needs a discrete-block geometry mode grouping contiguous samples by class + palette, built
  on the existing WebGPU fill pipeline, plus a layout/curve-style tag) — FACIES currently
  renders as a step curve. **Deferred to increment 3**: **GMM** (soft clustering) and
  **supervised mode** trained on core facies via the Python subprocess + scikit-learn, and
  **field-wide clustering** (pool samples across wells for globally consistent labels).
- **Missing-curve synthesis**: train per-field regressors to predict DT/NPHI where absent;
  holdout-well R² report so it's honest.
- **Auto-picks**: per-zone GR_MA/GR_SH percentile suggestions, change-point auto-zonation,
  spike/outlier QC report across the field.
- **Done when**: facies on Minas reproduce the manual sand/shale zonation; synthesized
  curves ship with their holdout metrics.

### Phase 11 — Trust & reproducibility (the professional layer)

- **Audit trail & lineage**: every module/equation run and data edit logged (`runs` table:
  params, inputs, timestamps); any computed curve can answer "how was I made?" with its
  full ancestry.
- **Interpretation scenarios**: named parameter sets; run the same chain under scenario
  A/B; diff view (curve overlay + per-zone stats delta).
- **Project operations**: autosave checkpoints, crash-safe WAL, merge wells from another
  project file.
- **UX**: per-project workspace persistence, command palette (Ctrl+K).
- **Done when**: scenario A/B compare works end-to-end and lineage is visible for every
  curve in the project.

### Phase 12 — Platform & extensibility (the finish line)

- **User-defined Python modules**: a manifest (JSON) + Python script drops into a project
  `modules/` folder and appears in the ribbon with an auto-generated dialog — your
  personal Loglan library, shareable as plain files.
- **Native DLIS** (replace the dlisio bridge if it ever limits), LAS 3.0, WITSML later.
- **Distribution**: Tauri installer + auto-update, bundled sample project, in-app method
  help per module (F1).
- **Long game (demand-driven)**: NMR T2 (array_logs table already exists), borehole
  images, geomechanics, production logs.
- **Done when**: a colleague installs SandiBumi from an installer, imports a DLIS, runs
  your shared Python module, and exports a PDF report — zero developer tools involved.

## 4. Field-review backlog — Jauhar's o/x click-through (2026-07-19)

The complete feature/fix list from Jauhar's full review. Items marked **✅ done** were fixed
the same day; the rest are ordered by suggested priority (P1 = affects daily trust/safety,
P2 = interpretation workflow, P3 = new capability). Each should be delivered in small
increments with REVIEW.md check items.

### Done same-day (2026-07-19)

- ✅ **Ctrl+wheel = zoom** on histogram/crossplot/Pickett; plain wheel scrolls the page.
- ✅ **Pertamina theme** now uses the official palette (#ED1A2F / #006BB8 / #A6C210 / #161B22).
- ✅ **"Light" renamed "Default"** in the theme dropdown.
- ✅ **Advance tab regrouped**: one "Advance Methods" group = SSC, SSPW, RtC, IMTS, **Thin
  Beds** (moved out of Petrophysics). SSPW no longer sits under a "Sand-Silt-Clay" caption.
- ✅ **Multimin renamed → SandiMin** (no trademark collision); the legacy fixed 4-component
  "Multimin — Mineral Inversion" is removed from the Saturation dropdown (mineral solving is
  independent of Sw; still callable from saved workflow chains).
- ✅ **Repo made collaboration-ready**: .gitignore hardened, CONTRIBUTING.md added, work
  committed to git (remote hosting = Jauhar's choice, see CONTRIBUTING.md).

### P1 — Trust & safety (protect the user's work first)

- ✅ **Crash resilience** (2026-07-19, P1-b): `autosave.ts` — running-flag crash detection
  (cleared on pagehide/beforeunload), 10-s rolling autosave of the full session snapshot
  (dock layout + well + log-view layouts) to localStorage; abnormal exit → blocking
  choice dialog before boot: restore autosave, or Safe Mode (default layout; autosave
  stashed as a "Recovered …" session document). Normal launches also reapply well +
  log-view layouts via `applyAutosaveExtras` (dockview JSON doesn't carry them).
- ✅ **Unsaved-changes indicator** (2026-07-19, P1-b): `dirty.ts` registry — log-view user
  edits (properties, track widths/order, curve visibility) mark the panel; its tab shows
  ● and the QAT Save-Session button a dot; Save Layout clears that panel, Save/Open
  Session clears all. Workspace arrangement changes mark too (title-update noise muted
  via `muteDirty`). Dirty = "not in a named save"; the autosave runs regardless.
- ✅ **Click-to-arm, double-click-to-edit inputs** (2026-07-19, P1-a): app-wide via
  `interactionGuard.ts` — a single click arms `input[type=number]` read-only (dashed
  outline, wheel/arrow spin blocked); double-click unlocks (solid outline, value
  selected); blur re-arms. Keyboard Tab focus stays editable (deliberate); per-input
  opt-out with `data-free-edit`.
- ✅ **Right-click lockdown** (2026-07-19, P1-a): default WebView menu killed everywhere
  except editable fields (their native menu is the harmless edit menu); custom panel menus
  untouched. F5/Ctrl+R guarded by a blocking confirm; Alt+arrows and mouse back/forward
  buttons blocked.
- ✅ **Workflow builder as a pane, not a popup** (2026-07-19, P1-a): dock component
  "workflow" (singleton, in the ＋ panel menu and Petrophysics → Workflow…); closing the
  pane mid-run cancels the chain; numeric params follow the double-click-to-edit rule.
- ✅ **Database versioning — never overwrite** (2026-07-19, P1-c): `log_sets` run-event
  table + append-only `computed_curves_archive`; `computed_curves` stays the fast
  "current" store (rows tagged `set_id`), so every read path is unchanged. Module runs,
  chains (one version per chain run), equations (set EQUATION), ML (ML) and SandiMin
  (SANDIMIN) all write versioned: re-run = version N+1, history kept, any version
  restorable/prunable from the Curve Catalog. Provenance per run: module, params,
  inputs, timestamp. Output-set choice in the module dialog + Workflow Builder.
  Catalog: merged imported+computed view with set/version/module/when + n/min/max/mean,
  one search box, click-to-sort headers. **Deferred to a later increment**: per-module
  INPUT-set selection (reads currently resolve latest-current), unit/family columns for
  computed curves, set-qualified log-view tracks.
- ✅ **Set INPUT selection on modules** (2026-07-19, P1-c follow-up): "Input set" field in
  every module dialog and the Workflow Builder — inputs resolve from that set's archived
  values (latest version per well, case-insensitive name); curves the set never wrote fall
  back to the usual sources, so chains still consume earlier steps' outputs. Blank =
  current values (unchanged default). Provenance `inputs_json` records the input set.
  Still deferred: pinning a specific version (workaround: Restore it first), input-set on
  ML/SandiMin dialogs, unit/family columns for computed curves, set-qualified log-view
  tracks.
- ✅ **Curve catalog power features** (2026-07-19, P1-c): one search box across
  mnemonic/set/module/unit/date, click-to-sort columns, per-curve n/min/max/mean.

### P2 — Interpretation workflow

- ✅ **Imports (tops-style)** (2026-07-19, P2-a): "Import Tops…" (CSV/TXT, delimiter
  auto-detect, alias headers, multi-well by WELL column or selected-well fallback,
  headerless NAME-DEPTH accepted, upsert keeps colors) + "Import Aux…" (PETROGRAPHY /
  XRD / PERFORATION / custom into new `aux_data` long-format table: TOP+optional BASE,
  numeric or text values; replace per well+dataset; viewable in DB Inspector).
  Deferred: aux overlays on plots/log tracks (perforation flags, XRD points).
  Original ask: tops from CSV/TXT (menu is missing), petrography, XRD,
  perforation data.
- ✅ **Tops editor, Petrel-style** (2026-07-19): log views draw tops as labeled colored
  lines; 🏷 toolbar toggle enables editing — click to add (name/depth/color dialog),
  drag to move (live preview), double-click to rename/recolor/delete; all undoable;
  automatic **stratigraphic-crossing warning** after every pick (this well's top order
  vs the majority of other wells, `tops.rs::check_top_order`); **marker autocorrelation**
  (Data → Autocorrelate…): source-well log shape (GR default, ±window) slid over each
  target well ±search range → proposed depth + Pearson r, strong matches pre-ticked,
  applied as one undoable batch (`tops.rs::autocorrelate_top`).
  Still deferred: named tops SETS (multiple stratigraphic schemes per project — current
  model is one set of tops per well); tops-set selection mirroring the well pin model.
- ✅ **Well pin semantics rework** (2026-07-19): the 📌 pin is now a MODE, not a lock.
  Pin ON (default) = selecting a well drives the whole workspace. Pin OFF = viewers keep
  their wells and only the ACTIVE panel follows the selection (working-pane model —
  side-by-side multi-well viewing without per-panel pins). The old selection-blocking
  lock is gone, which also removes the "locked" weirdness with a second wells pane.
  **Multi-select**: Ctrl-click toggles, Shift-click ranges, ⇄ inverts within the visible
  list; count shown in the Wells label; batch dialogs (module runs, workflow, Multimin,
  ML, Monte Carlo, Cutoffs & Summary) pre-tick the multi-selection instead of just the
  active well (`defaultRunWellIds`). Fresh panels always adopt the current well even
  with pin OFF.
- ✅ **Log-view layout interaction** (2026-07-19): ▤ collapsible track headers (full →
  compact chips → titles only, plus a 34vh cap with per-track scroll); drag a curve
  between track headers to MOVE it (Ctrl = copy), undoable; ▦ customizable track borders
  (solid/dashed/none, width, theme-or-custom color) drawn on the overlay canvas; hover
  readout scoped to ONE track — the clicked/selected track (header highlighted, click
  again to release) else the track under the cursor; right-click a track → "Edit CURVE…"
  per curve: wireline shift (whole-curve resample at d−Δ), set constant, blank (NaN),
  interpolate across interval, scale a·v+b — `curve_edit.rs` edits whichever store holds
  the curve (standard column / computed incl. set_id preservation / generic RAW by
  mnemonic-then-family) via transactional read-modify-rewrite, returns the changed
  samples' previous values as packed bytes so Ctrl+Z restores bit-exactly
  (`restore_curve_values`); recorded in Processing History.
  **Deferred**: drag-on-canvas interval picking for edit ops (dialog takes top/bottom,
  prefilled around the clicked depth); header-mode/border persistence in sessions;
  interactive wireline-shift preview while dragging the curve.
- **Histogram v2** (double-click or right-click opens properties, Geolog-style): box plot,
  cumulative overlay, bin control, colors, user-input percentiles; **universal** (no
  hard-wired GR_MA/GR_SH — parameter pickers appear only when wanted); statistics
  (min/max/mean/std/percentiles/n) displayable inside or outside the plot.
- **Crossplot v2**: plot-size control, marginal histograms on X and Y, bins/colors/
  percentiles like histogram; regression options (Y-on-X, X-on-Y, RMA, linear, power,
  log); additional colormaps incl. one that survives **logarithmic Z scales** (rainbow
  doesn't); universal parameters (D-N porosity overlay only when requested).
- **Pickett v2**: N together with M and Rw; free user input of line parameters (lines
  follow); Z-value coloring by a chosen log with customizable gradient.
- **Data prep**: **split & merge** of curves/intervals; **normalization with tops-referenced
  intervals** — reference top/bottom from a chosen tops set; missing marker → nearest
  stratigraphic marker (top → shallowest, bottom → deepest); percentiles extrapolated over
  the whole interval and normalized together.
- **Highlight tool** (Geolog-style): multiple depth highlights, same or different colors;
  convert highlights → tops.
- **Typography**: text reads slightly fuzzy/washed-out up close — investigate WebView2
  rendering (display scaling, weight, contrast). Waiting on Jauhar to say whether it's
  blurriness or lightness (see REVIEW.md).

### P3 — New capability

- **SandiMin**: optional nonlinear Sw equation in the solve loop (iterate to convergence —
  Indonesia/Simandoux-style inside the inversion, not just the dual-water CT row).
- **Monte Carlo**: "finalize parameters → print to curves": write LOW / BASE / HIGH curves
  from the chosen result percentiles (named by result value, not optimist/pessimist case,
  computed internally).
- **Plugins (Advance ribbon)**: user-authored Python and Loglan modules — variable
  declarations + code, manifest-style, shareable as files.
- **2D Window (new ribbon)**: lateral analysis — per-well X/Y (directional-survey-corrected
  at each marker), thickness & weighted-property maps per marker interval, category-based
  maps (property/weight/algorithm), contours + wells posted with Z-value gradient; later
  fluid contacts and simple volumetrics.
- **Panes independent of windows**: any pane floatable/resizable like a window; windows
  become pure grouping containers.
- **Data tools (separate, later per Jauhar)**: log digitization, core image input,
  XRD/petrography digitization.
- **User guide PDF**: topic-by-topic, step-by-step with screenshots from the real app using
  Jauhar's database as the worked example; include the outstanding review items as an
  appendix. Produce after the P1 batch lands (or on request, against the current build).

## 5. Standing advantages to protect

- One-file DuckDB project + full SQL access.
- Python/numpy as the scripting surface (every new feature should expose its data to it).
- Undo everywhere a cell or property changes.
- GPU log rendering + synchronized cursor.
- Manifest-driven module dialogs (new module = Rust fn + manifest, zero UI code).

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
