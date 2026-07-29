# SandiBumi — Petrophysical Software Engine

> **SandiBumi** (formerly *Arshilla*) — the project folder on disk is still `D:\XX. SandiBumi`; only the
> product/branding was renamed. The compiled binary is now `sandibumi.exe`, bundle id `com.sandibumi.petro`.

Desktop application for multi-well (2000+) petrophysical log analysis. Stack: **Tauri (Rust) + DuckDB (embedded, bundled) + TypeScript/WebGPU**.

This file is the Claude Code equivalent of `.cursorrules` (kept in this repo for Cursor). Keep both in sync if the rules change.

## Critical implementation rules

1. **Data storage**: optimize DuckDB queries for columnar reading. Use compressed binary blobs or `LIST` types for array logs (NMR, waveforms).
2. **Missing values**: never use `Option<f32>` for continuous logs. Missing data is strictly `f32::NAN`, so matrix arithmetic stays branch-free.
3. **Serialization & IPC**: never pass raw data arrays over Tauri's IPC bridge as JSON strings. Convert `Vec<f32>` matrices to raw bytes with `bytemuck`, return `Vec<u8>`, and cast to `Float32Array` on the frontend.
4. **Concurrency**: `rayon` for CPU-bound cell/well-parallel work; `tokio` for background async scheduling (long-running inversions, I/O).
5. **Code delivery**: concise, modular, production-speed-focused. No extensive unit test blocks unless explicitly requested.
6. **Writes are whitelisted**: the frontend never sends SQL for writes — only explicit commands (`db.rs` `TABLE_SPECS` + update fns). The SQL Query panel is read-only (SELECT/WITH only).
7. **Python equations run as a SUBPROCESS** (`python_engine.rs`), never PyO3/embedded — a missing Python must never stop the app from launching. Discovery: `ARSHILLA_PYTHON` → `%LOCALAPPDATA%\Programs\Python\Python31x` → PATH; requires numpy. **DLIS import (`dlis.rs`) reuses the same subprocess mechanism** (`find_python` + a `dlisio` helper script), never a native parser; needs the `dlisio` pip package (installed: `dlisio 1.0.4` in the Python312 env). A missing `dlisio` fails only the DLIS import, with a clear message — never the app.
8. **Data/UI edits must be undoable** (`src/undo.ts` `pushUndo`); module runs are re-runnable, not undone.
9. **New petrophysics modules** = Rust fn + manifest in `modules.rs`; parameter dialogs auto-generate — write no UI code for them. Heavy solvers can live in their own file (e.g. `multimin.rs` — deterministic NNLS mineral inversion; do NOT confuse with `inversion.rs`, which is the separate background async stochastic-job registry) and be referenced from `modules.rs::list_modules`/`run_module`.
10. **Module inputs are generic-store aware**: `equations::fetch_curve_frame` resolves any non-standard, non-computed curve name from `curve_meta`/`curve_samples` (set RAW) by mnemonic-then-family, so modules/equations can take PEF/CALI/DRHO/extra runs — not just the fixed six. (Log-view rendering `get_track_data` still reads only `standard_curves`.) Runs can pass `opts["MASK"]="<flag curve>"` (e.g. BADHOLE) to NaN-out flagged samples in every output.

## Current state (2026-07-20)

Phases 1–7 CODE-COMPLETE + Phase 8a shipped: dockview docking workspace, Office-style
5-tab ribbon, light/dark/system + client-branded themes (Pertamina/Halliburton/Schlumberger/
LAPI-ITB/white — `theme.ts` THEMES + `:root[data-theme="…"]` var blocks in styles.css; theme
change fires `appState.themeVersion` so canvas panels repaint live), WebGPU log views,
histogram/crossplot/Pickett/correlation with synchronized
hover, Database Inspector with undo, Python(numpy)/Rhai equations, LAS 2.0 both ways + DLIS
import (dlisio subprocess), generic curve store (any mnemonic, units canonicalized,
family-tagged, RAW/EDIT/FINAL sets) feeding module inputs by mnemonic-or-family,
deviation/TVD (minimum curvature), bad-hole QC + universal run mask, multimin NNLS
inversion (`multimin.rs`), saturation-height (`satheight.rs`, scal_pc + Leverett-J fit +
sw_height), pragmatic environmental corrections, interactive Thomas-Stieber crossplot
with draggable endpoints → zone params, and **composite log plots** (`composite.rs`) at
true print scale exporting vector SVG + a dependency-free multi-page PDF (Plot ribbon →
Composite…).

Phase 8.5 shipped (2026-07-18) — **Jauhar's method suite** ported from his 7 reference
projects: `ssc.rs` (SSC Kuttan/GAP-2023 port of `ssc_lqr_gap_edit_jau.lls` + SSPW
reconstructed from spec — *SSPW needs validation vs his the reference suite LAS exports*), `lrlc.rs`
(sw_rtc excess-conductivity + sw_imts iterative mineral-textural-scaled Waxman-Smits from
his LRLC research), `gr_normalize` (two-point percentile GRN, Rokan P3 53.68/P97 133.93)
and `log_predict` (leave-one-out KNN synthetic logs, MAX_RAW washout rule) in `modules.rs`,
Bunga mnemonic table merged into `curves.rs` FAMILIES. Method math is banked IN THIS REPO:
`docs/method_ssc_sspw.md`, `docs/method_lrlc_rtc_imts.md`, `docs/workflow_standards.md`
(portable — do not rely on any machine-local Claude auto-memory for it).

Phase 8b shipped (2026-07-18) — **report generator** (`report.rs` + `reportDialog.ts`,
Plot ribbon → Deliverables → Report…): cover → editable methodology table (persisted as
`report_template` document) → zone-parameter table → pay-summary table → composite pages,
one PDF via the 8a DrawOp machinery; batch export per well; PNG page export (`save_png`).

Phase 9 STARTED (2026-07-18) — **workflow chains** (`chain.rs`): run an ordered list of
modules across many wells in one click, steps sequential + wells rayon-parallel per step,
pollable progress + cancellation (registry pattern like `inversion.rs`; frontend supplies
the job id and polls `get_chain_status`). Chains persist as `workflow` documents; UI =
`workflowDialog.ts` (Petrophysics → Batch → Workflow…). Steps use manifest defaults,
overridden per zone by `zone_params`; increment 2 adds **per-step parameter editing** (an
expandable ⚙ editor per step — manifest-driven curve/option/param fields + Mask, storing only
non-default overrides; zone_params still win per zone). Increment 3 adds **Monte Carlo
uncertainty** (`montecarlo.rs`, Petrophysics → Batch → Monte Carlo…): normal/uniform/triangular
distributions on model params → N seeded realizations of a chain, P10/P50/P90 net/NTG/PHIE/SWE/
HPV per zone + HPV histogram. Runs **entirely in memory** (`run_module` returns vectors; zero
`computed_curves` writes), rayon-parallel, seeded from `(seed, index)` — dodges the write
bottleneck by design. Also this session: **BLSO real-data pipeline test**
found+fixed two bugs — coverage-aware LAS alias resolution (`parsers.rs`, skip all-null
placeholder columns) and sw_rtc/sw_imts default RT input (`lrlc.rs`, `RT`→`RES_DEEP`). See
`pipeline_blso_test.rs`.

Phase 9 increment 5 shipped (2026-07-19) — **write-path perf hardening**. The field-scale
write bottleneck was `computed_curves`'s 3-column `PRIMARY KEY` (ART uniqueness index = ~3.4×
insert overhead, proven by the probe in `pipeline_blso_test.rs`). Fix: **dropped the PK**
(schema is now PK-less; `db::migrate_drop_computed_curves_pk` rebuilds existing projects on
launch, idempotent via `duckdb_constraints()`) — uniqueness is upheld by the write discipline
(`equations::write_computed_curves_batch` DELETEs a well's target curve names before appending;
point-updates UPDATE in place; **nothing inserts a duplicate — do not add ON CONFLICT/upsert
paths that assume a PK**). Also batched each well's whole module output into one DELETE + one
Appender (used by `workflow.rs`). Real 100-well × 4-module chain: ~50s → **21s**. GPU stays
render-only; DuckDB single-writer (`Mutex<Connection>`) is fundamental, so the win is the
per-row index removal, not parallelism.

Phase 9 increment 4 shipped (2026-07-18) — **Field Dashboard** (`dashboardPanel.ts`,
Petrophysics → Batch → Field Dashboard…): runs `run_pay_summary` across every well →
per-zone aggregation table + per-zone box plots (inline SVG) + a sortable multi-well×zone
grid (flag-filterable) + CSV export. Frontend-only; reuses the pay-summary command.

Phase 10 increment 1 shipped (2026-07-18) — **Electrofacies** (`facies.rs`
`electrofacies` module, new **Facies** ribbon category): per-well k-means (k-means++,
seeded SplitMix64, best-of-8) over up to 5 z-scored curve slots (GR required) → FACIES
curve (0..K-1), cluster labels ordered by ascending GR mean so numbering is monotone in
shaliness. Runs through the normal module framework (mask/chain/zone_params free). QC:
crossplot **categorical coloring** for discrete curves (`plotCanvas.ts` `faciesColor`/
`categoricalColors`/`looksDiscrete`, `FACIES_PALETTE`) with a swatch legend. Deferred:
colored FACIES block track in the log view (increment 2), GMM + supervised (core-trained,
scikit-learn subprocess) + field-wide clustering (increment 3).

Workspace UX pass shipped (2026-07-18) — **per-panel right-click context menus**
(`contextMenu.ts`; items vary by panel type, then a shared window block), **Ctrl+scroll
zoom-at-cursor** on log views (`LogCanvasRenderer.zoomAt`; plain scroll still pans),
**Split right / Split down** window actions (group header buttons + context menu), and a
**refined Equation Editor** (styled `.field-label` inputs, grouped fields, `.btn`/`.btn-accent`
actions). Panels already drag between windows (dockview built-in).

Selection-driven workspace shipped (2026-07-19) — **the Wells & Tops pane decides what
every panel shows**. `appState.selectedInterval` (`TopInterval` in `state.ts`) is set by
clicking a top in `topsPanel.ts` (interval = that top down to the next; last top → TD via
`depthMax: null` — backend treats min/max independently). `workspace.ts` `createPlot`
rebuilds histogram/crossplot/pickett when the selected well changes: builders take an
`initial` state and expose `PlotContent.getState`, so curve/zone choices survive the
rebuild (generation counter drops stale async builds; panel retitles "Kind — WELL"; the
old "select a well first" dead placeholder now builds on first selection).
`buildZoneSelect` (plotCommon.ts) follows `selectedInterval`: inserts/updates an
auto-selected `@top` option and **fires `change` on the select** so panels reload without
any panel-side wiring — but it returns a `dispose` every builder must call. Log views
scroll to the selected top (`LogCanvasRenderer.scrollToDepth`). Interval is cleared
BEFORE the well broadcast on well switch (order matters — followers must never see a
foreign interval). Selected rows highlight via `.tree-selected`/`.top-selected`.

UX conventions (2026-07-19 fix batch, from Jauhar's click-through): dialogs are
NON-BLOCKING — `.modal-scrim` is pointer-transparent, only Esc/✕ close (never re-add
scrim-click-to-close or a blocking overlay). Ribbon groups with >1 button MUST wrap
buttons in `.ribbon-btn-row` or they stack past the 80px ribbon height. Log views all
follow `selectedWell` unless per-panel PINNED (📌 in the panel's own `.logview-tools`
toolbar, which also owns depth scale/zoom/track width — these were removed from the View
ribbon; don't re-add ribbon controls that target "the active log view"). Window resize
preserves pane sizes: `workspace.relayoutKeepingPaneSizes` restores every grid group's
size except the largest (dockview hardcodes `proportionalLayout: true`, so this is done
by snapshot/restore around `dock.layout()`).

Interactive plots (2026-07-19) — `PlotCanvas` draws in LOGICAL (CSS) pixels, not raw
canvas.width. Every redraw calls `fitCanvasBackingStore(canvas)` first (sizes the backing
store to clientW/H × dpr, capped at 2.5) so plots are crisp at the real panel size; the
panel root is `.plot-content` (flex column) so the canvas fills. Mouse handlers use CSS
px (`clientX - rect.left`), NOT the old `/rect.width*canvas.width` scaling — don't
reintroduce that. Zoom/pan is `attachZoomPan({canvas,getPlot,view,redraw,axes?,onPanStart?})`
driving a shared `ViewportRef` (wheel=zoom-to-cursor, drag=pan, dblclick=reset), math done
in each axis's transformed space so log axes are correct; `attachResizeRedraw` re-renders
on ResizeObserver. drawCrossplot/drawHistogram/drawPickett take an optional `view` that
overrides the axis window (histogram keeps full-range binning). Crossplot draggable
parameter handle lives at (X pick, Y pick); a press on a handle vetoes the pan, and a
pointer that moved >4px suppresses the click-pick. Reset the viewport on new data.

Well groups (2026-07-19) — `well_groups` + `well_group_members` tables; at most ONE group
active (`set_active_well_group` clears all then sets one). `appState.activeWellGroup` is
kept in sync by `syncWellGroups()` (call after any mutation, then `bumpWellGroupsVersion()`).
Global-filter semantics: the Wells pane filters to members and EVERY batch dialog wraps its
well list in `filterByActiveGroup(...)` — when adding a new batch/run dialog, do the same so
it respects the active group. Membership is explicit (manual); `rule_json` is reserved.

Shell UX (2026-07-19) — The **quick access toolbar** (`.qat` in index.html, left of the
ribbon tabs) holds Undo/Redo/Save-As/History; Save Project As is NOT a ribbon button anymore.
Undo state is broadcast via `undo.ts::onUndoChange` (with `redoDepth`/`nextUndoLabel`).
**Processing history** is `processLog.ts`: call `recordProcess(kind, detail, well?)` after any
meaningful op (imports, module runs, edits, exports, pins) — it persists to the `documents`
table (`history`/`log`, debounced) and `main.ts` calls `loadProcessLog()` at startup;
`HistoryPanel` subscribes. **Well lock** is a GLOBAL active-well lock (`appState.pinnedWellId`
+ `setPinnedWell`/`isSelectionBlocked`), NOT per-panel — log views just follow `selectedWell`,
which the lock holds steady; the tree's click handler must call `isSelectionBlocked` before
switching. The old per-panel log-view pin was removed. **Custom right-click** only fires on
empty pane background + `.plot-canvas`; `attachContextMenu` bails on any interactive control/
editor/table/toolbar/tree-row (keep that denylist current when adding controls). **Plot image
export** (`plotExport.ts`: `copyCanvasToClipboard`/`saveCanvasAsPng` via `save_png`/`printCanvas`
via hidden iframe) is added to each canvas plot's toolbar (`buildImageExportButtons`) and its
canvas context menu; the WebGPU log view can't `toDataURL`, so it routes to the Composite
dialog. **Plot templates** (`buildPlotTemplateBar` in plotCommon, doc_type `plottmpl:<kind>`)
save/recall a plot kind's display opts by name — distinct from `savePlotProps` (last-used) and
from named log **layouts** (doc_type `layout`).

Advance tab + Sessions (2026-07-19) — The **Advance ribbon tab** (between Petrophysics and
Plot) holds Jauhar's flagship methods, promoted OUT of the auto-generated dropdowns. `ribbon.ts`
`Ribbon.ADVANCED_MODULE_IDS = ["ssc","sspw","sw_rtc","sw_imts"]`; `loadAllModules` fetches
manifests once, `renderCategoryModules` filters those ids OUT (so they don't double-appear) and
`renderAdvancedModules` builds grouped icon buttons in `#advance-modules`. The ML Models button
was moved into the Advance panel in index.html (id-based wiring unchanged). To promote another
module: add its `name` to ADVANCED_MODULE_IDS + a META `[short, caption, iconPath]` entry. **Sessions**
are named workspace snapshots (distinct from Save Project As, which copies the whole .duckdb):
`workspace.ts` `SessionSnapshot {version, layout: dock.toJSON(), well}` + `snapshotSession()`/
`applySession()` (sets selectedWell BEFORE `dock.fromJSON` so recreated panels init on it),
stored in `documents` doc_type `session`. QAT buttons `#qat-save-session`/`#qat-open-session`
(right of Save) → ribbon `handleSaveSession`/`handleOpenSession`. dockview toJSON does NOT
serialize a log view's chosen Layout, so `snapshotSession` also captures `logViewLayouts`
(panelId → Layout, from each view's `getLayout()`) and `applySession` reapplies them by id
right after `dock.fromJSON` (which synchronously recreates the log-view panels, repopulating
the `logViews` map). Plot-panel internal state (selected curves/props) is still not carried —
extend the snapshot the same way if that's needed.

Generalized Multimin v2 (2026-07-19) — `src-tauri/src/multimin2.rs` is a the reference suite-Multimin/IP-Mineral-
Solver-style optimizer, SEPARATE from the fixed 4-component `multimin.rs` (untouched). Spec was
extracted from the local the reference install helpset + IP2018 install: `docs/multimin_ref_spec.md` and
`docs/multimin_ip_spec.md` — consult those before touching the physics. Architecture:
`Component{name, kind: mineral|clay|fluid, zone: ""|X|U, fluid_type, endpoints, cec, max_vol}`;
27-entry `LIB` (IP dropdown order; merged the reference suite/IP endpoint defaults over 14 tool keys — RHOB,
NPHI, DT, GR, PEF, U, THOR, POTA, URAN, VP, VS, EPT, EATT, SIGMA). Zone convention: only CT (deep
conductivity) sees U-zone fluids, everything else sees X-zone; CT/CXO tools take a RESISTIVITY
curve, converted per sample to conductivity and entered as the DUAL WATER LINEAR row
Ct^(1/w) = Σ v·C^(1/w) with w = 0.75m+0.25n (fluid_calc: Arps→Cw/Cmf, Bateman-Konen salinity,
Cbw = 0.0007(T+8.5)(T+298), α expansion below 20,455 ppm; auto σ = 0.03·C^(1/w)). Constraints:
hard UNITY over minerals+U-fluids (X excluded) via KKT equality; hard box 0≤v≤max_vol (fluids
0.5); soft σ=0.01 rows (the reference suite "Tool" constraints) POROSITY (ΣX=ΣU) and BNDWAT
(Σ 96·CEC·ρ/(T°C+298)·α·v_clay = v_bw, reproduces the reference suite's 0.1841 Illite multiplier); WATER MUD
(ΣXwater ≥ ΣUwater, WBM) enforced by re-solve on violation. Solver `solve_bounded_lsq` = active-set
with three states (free / at-0 / at-hi), fixed-at-hi folded into KKT RHS. Outputs VOL_<comp> +
`<prefix>`_PHIE/PHIT/SWE/SWT (+SXOT/MOVEDHC with X/U split)/VSH/RECON. Commands: `run_multimin`,
`multimin_library`, `multimin_fluid_calc` (dialog preview). Dialog `src/ui/multiminDialog.ts`
(Advance-tab `#multimin-btn`): grouped component picker, 16 input logs + "+ Add user-defined input"
(custom endpoint column, σ 0.015), editable endpoints matrix (U-zone fluid cells "—", CT/CXO
"auto"), CEC + Max columns, fluid-properties panel with live `multiminFluidCalc` preview.

FACIES block track shipped (2026-07-19) — `CurveStyle.fill: "blocks"` marks a discrete
class curve (electrofacies) to render as full-track-width colored interval blocks instead
of a value line. Both renderers implement it: `LogCanvasRenderer.buildBlockGeometries`
(one geometry per class, existing fill pipeline; the line pass draws 0 vertices) and
`composite.rs draw_class_blocks` for print/SVG/PDF. Class colors come from
`FACIES_PALETTE` in `plotCanvas.ts`, **duplicated in `composite.rs` — keep the two in
sync**. Built-in "Facies" layout lives in `layout.rs`; min/max on a blocks curve is
ignored (header shows "class blocks" instead of editable scale). min/max decimation never
averages, so decimated class values stay valid integers.

UI language (2026-07-19) — `src/i18n.ts` translates visible DOM text (+ title/placeholder/
aria-label/optgroup-label) to Bahasa Indonesia / Basa Sunda by exact-phrase dictionary
lookup, live via MutationObserver. English is the source language: keep writing UI strings
in English; add dictionary entries only for phrases that read naturally translated —
**technical terms (Thin Beds, Monte Carlo, Pickett, mnemonics) stay English by design**
(Jauhar's explicit request). `data-no-i18n` skips a subtree (used on the language select
itself). Never key dictionaries on user data strings (well/layout/curve names).

ML suite shipped (2026-07-19, Phase 10-4) — `ml.rs` + `src/ui/mlDialog.ts`
("ML Models…" in Petrophysics). scikit-learn subprocess, same protocol style as
python_engine.rs (JSON header line + raw f32; stderr last line = error; keep runner
messages ASCII). Algorithm ids in the TS catalog and ML_RUNNER must stay in sync.
Supervised (regression/classification) fits on pooled labelled samples from train wells;
unsupervised (clustering/reduction) fits on the POOLED apply wells — field-wide by
construction. Cluster ids are reordered by ascending mean of the FIRST feature curve
(same convention as facies.rs — put GR first). Rust masks incomplete rows and scatters
predictions back NaN-padded; outputs = base name + suffix (`_PROB`, `1..n`). Autoencoders
intentionally not wired (needs PyTorch).

P1/P2 field-review series shipped (2026-07-19/20, from Jauhar's ROADMAP §4 click-through):
**P1-a** right-click/reload lockdown + double-click-to-edit + workflow builder as a pane;
**P1-b** crash safe-mode + autosave + unsaved-change indicators; **P1-c** log sets
(versioned outputs RAW/EDIT/FINAL, provenance, catalog search, input-set selection on
module runs); **P2-a** tops/petrography/XRD/perforation CSV-TXT imports; **P2-b**
Petrel-style interactive tops editor (`topsEditor.ts`; Svelte 5 is now available for new
UI); **P2-c** well-pin semantics rework + multi-select with inverse; **P2-d** log-view
layout interaction (collapsible headers, curve move/copy, borders, scoped readout,
right-click curve editing); **P2-e** Histogram v2 and **P2-f** Crossplot v2 (per-plot
properties dialogs, box plot/cumulative/percentiles, marginal histograms, regression
options, log-safe colormaps, opt-in overlays).

Chartbook overlay library shipped (2026-07-20) — `src/ui/chartOverlays.ts` (GENERATED —
do not hand-edit; regenerate with `tools/chartdig`, see its README) holds **19 vector-
digitized Schlumberger-2013 chart defs**: CNL Por-11/12, APS Por-13/14 (APLC+FPLC),
adnVISION675 Por-16, EcoScope Por-18 BPHI + Por-19 TNPH, PEF Lith-3/4, sonic-neutron
Por-20 TA+FO, density-sonic Por-22 TA+FO (+7 mineral points), Pe-K / Pe-Th/K Lith-1,
Th-K clay Lith-2, Umaa-ρmaa MID Lith-6. Crossplot option `chartOverlay` (replaces the old
`dnOverlay`; normalize migrates fresh→por11 / salt→por12); `matchOverlayAxes` gates
drawing to matching axis pairs (either orientation) + log-state; the Properties dialog
groups the chart select by applicability to the current axes.

Senior audit banked (2026-07-20) — `AUDIT-2026-07-20.md`: 35 confirmed findings
(5 review dimensions, each finding adversarially verified), triaged P0–P3 in **ROADMAP
§4b** — that section is the active correctness/perf backlog. Six fixes already landed:
pay-summary PERM semantics (missing PERM now FAILS an active cutoff — numbers changed),
LAS declared-NULL honored + multi-word well names, case-insensitive computed-curve
lookup, true depth-scale print ratios, tops-editor overwrite undo.

**`REVIEW.md` is the user's pending click-through checklist — keep it current when
shipping features.** Full roadmap and deferred items: `ROADMAP.md` (**§4b = senior-audit
backlog, priority-ordered P0–P3**; Phase 9 remaining: per-well parameter override table,
lazy catalog loading / decimation cache / UI responsiveness + 2000-well stress fixture —
write-path perf done; Phase 10 done: facies + block track + GMM + full ML suite —
supervised facies = ML classification with a FACIES/litho target; remaining ML deferral:
autoencoders). Queued next increments: **Pickett v2** (N with M and Rw, free line params,
Z-color gradient — absorbs the audit's Pickett findings) and a **UMAA/RHOMAA computation
module** to feed the Lith-6 MID overlay.

## DuckDB WAL resilience

`tauri dev` restarts `sandibumi.exe` on every Rust source change; an unclean kill
mid-write can corrupt `project.duckdb.wal` badly enough that DuckDB's own replay
hits an internal assertion and the app panics on startup. `db::init_db_resilient`
(used in `lib.rs::run()`) catches this, moves the WAL aside as a timestamped
`.corrupt-backup-<unix-ts>` file (never deleted), and reopens from the last
checkpoint. If the app still won't launch, check for `.corrupt-backup-*` files in
`src-tauri/` — recovery already ran and is not the problem; look elsewhere.
**Never test-launch `npm run tauri dev` in a way that force-kills the process before
it exits on its own** (e.g. a background shell timeout) — that recreates the exact
corruption this code defends against.

## dockview-core gotchas (cost whole sessions — do not rediscover)

- Theme must be passed as a `theme: {name, className, gap, dndTabIndicator}` OBJECT;
  a bare `className` option silently falls back to the dark "abyss" theme.
- Drive `dock.layout(w, h)` from your own ResizeObserver — built-in auto-resize misses
  the initial layout and every group stays 100×100.
- `initialWidth` on addPanel is unreliable pre-layout; call `panel.api.setSize()` after.
- Inactive tabs are DETACHED from the DOM (instances survive; querySelector only sees
  the active tab's content).
- Any CSS `display` rule overrides the `hidden` attribute — always pair with
  `[hidden] { display: none }` (this bit us twice: ribbon panels, ribbon menus).
- `group.api.moveTo({position})` with no target group auto-creates a grid group and
  moves the panels in (the ⇱ dock-back button); the old group and its header-actions
  renderer are disposed — don't hold element refs across it.

## Browser verification trick

Vite serves raw TS modules, so in a browser at the dev URL you can
`await import('/src/ui/anything.ts')` and drive real components with synthetic data —
no Tauri backend needed. In vite-only preview every `invoke` error
("Cannot read properties of undefined (reading 'invoke')") is benign.

## Environment notes

### Setting up a NEW machine (clone → running app)

`CONTRIBUTING.md` §1–2 is the canonical checklist. Summary:

1. **Rust** stable-msvc via rustup; **Node.js LTS**; **VS Build Tools** C++ workload
   (WebView2 runtime is preinstalled on Win 11).
2. `npm install` in the repo root. DuckDB is the `bundled` Cargo feature (compiles from
   source — the first `cargo build` is long; no system DuckDB needed).
3. **Python 3.10+ with `numpy`** for the equation engine (subprocess — see rule 7);
   `pip install dlisio` for DLIS import, `scikit-learn` for the ML suite. If discovery
   fails, set `ARSHILLA_PYTHON` to the interpreter path.
4. Try plain `npm run tauri dev` first — the vcvars 14.29 pin below is a
   reference-machine-specific workaround, only needed if the default MSVC toolset is
   broken.
5. `tools/chartdig` (chart digitizer) needs `npm i pdfjs-dist@4.10.38` **in that folder**
   and the chartbook PDF (`chartbook.pdf`, Schlumberger Log Interpretation Charts 2013 —
   copyrighted, NOT in the repo; on the reference machine it's at
   `D:\01. Work\00. Guidebook\chartbook.pdf`). Only needed to digitize NEW charts — the
   extracted data is already committed in `src/ui/chartOverlays.ts`.
6. Claude auto-memory is machine-local — everything durable lives in this file,
   `docs/`, `ROADMAP.md`, `REVIEW.md`, `AUDIT-*.md`. Trust the repo over memory.

### Reference machine (ARUNIKA / D:\XX. SandiBumi)

Rust, Node.js, and the MSVC linker are all installed and working — **but new shells may not pick up PATH updates from installers**. If `cargo`/`node`/`npm` report "not found," don't assume they're missing; verify with the full paths below before reinstalling anything:

- `cargo`/`rustc`/`rustup`: `C:\Users\ARUNIKA\.cargo\bin\`
- `node`/`npm`: `C:\Program Files\nodejs\`

Prepend both to `PATH` at the start of a shell session, e.g. (Git Bash):
```sh
export PATH="/c/Program Files/nodejs:$USERPROFILE/.cargo/bin:$PATH"
```

## Dev commands

**IMPORTANT: MSVC toolset 14.50 on the reference machine is broken (missing clui.dll).**
There, any command that compiles Rust must go through vcvars pinned to 14.29 (on a healthy
machine, plain `npm run tauri dev` is fine):

```
cmd.exe /c "call \"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat\" -vcvars_ver=14.29 && set PATH=C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH% && cd /d \"D:\XX. SandiBumi\" && npm run tauri dev"
```

```sh
npm install                   # install frontend deps (already done)
npm run tauri dev             # full desktop app (use the pinned command above)
npx tsc --noEmit              # fast frontend type check
cd src-tauri && cargo check   # fast Rust-only compile check (no vcvars needed)
npm run tauri build           # production bundle (size-optimized [profile.release])
powershell -ExecutionPolicy Bypass -File tools\check.ps1   # THE GREEN GATE: npm build + cargo test (vcvars-pinned), non-zero on first failure
```

Verify every change: `npx tsc --noEmit` + `cargo check` + a browser functional test — and
before a commit that claims "verified", `tools\check.ps1` is the one-command version of the
full bar (`-SkipRust`/`-SkipFrontend` for the inner loop only; green means the FULL gate).

Two hard runtime rules (both learned the painful way):
- **Never force-kill `npm run tauri dev`** (task-kill, shell timeout) — an unclean kill
  mid-write corrupts the project DuckDB WAL (see "DuckDB WAL resilience" below).
- After browser verification against the vite dev server, **stop the server so port 1420
  is free** for the user's own `npm run tauri dev`.

## Delegating work to subagents

Split by **task shape, not task size**. The cost driver in this repo is the verify loop
(`cargo check` through vcvars, ~minutes), not tokens: a cheap-model edit that fails to
compile twice costs more wall-clock than one correct expensive-model pass.

**The rule: cheap model + cheap verification = good. Cheap model + expensive verification
= bad. Never delegate to a cheaper model when a wrong answer would be SILENT** — a number
that is wrong but compiles ships into a client report, and no `cargo check` catches it.

| Task shape | Model | Why |
|---|---|---|
| Read-only retrieval — "which modules lack tests", "find every call site of `phie`", inventory/grep sweeps | **haiku** | Verification is free: you read the answer |
| Mechanical edits with a compiler gate — renames, a Tauri command wrapper, docs, test scaffolding, TS/dockview plumbing, i18n dictionary entries | **sonnet** | `cargo check` / `npx tsc --noEmit` is the verifier; a wrong answer is caught, not shipped |
| Anything numeric or convention-bound — `equations.rs`, `multimin.rs`/`multimin2.rs`, `ssc.rs`, `lrlc.rs`, `satheight.rs`, `thomeer.rs`, `hfu.rs`, `montecarlo.rs`, chart overlays, the theme var contract, dockview layout | **session model (default)** | Silent numeric/behavioural wrongness that no compiler catches |

The ladder is session-relative. On an **opus** session the strong tier IS the session model.
On a **fable** session, **opus** additionally becomes a mid-strong delegation tier — full
domain judgment at half fable's rate — for domain-aware work the main agent will
independently re-check (second-opinion reviews of numeric modules, domain test suites);
final judgment and sign-off still never leave the session model.

Mechanics:

- The `Agent` tool takes `model: haiku | sonnet | opus | fable`. Subagents otherwise
  inherit the session model.
- Only `Workflow` scripts expose per-agent `effort`. **Lower effort before downgrading the
  model** on domain work — `opus` at `effort: "low"` keeps the petrophysics judgment while
  cutting emitted tokens; downgrading the model throws the judgment away.
- Whatever the tier, a delegated edit is not done until `npx tsc --noEmit` + `cargo check`
  pass. Do not report a subagent's result as verified on the subagent's own say-so.
- Two things stay with the main agent regardless of size: **physics defaults** (they must
  be traced to `docs/` or a cited source per collaboration rule 5) and **anything touching
  the DuckDB write discipline** (the PK-less `computed_curves` contract).

## Collaboration protocol (Jauhar ↔ Claude)

Jauhar is a petrophysicist (Mahakam Delta, Indonesia) and a beginner programmer — explain
in petrophysics terms, not programming jargon. The working rhythm, on every machine:

1. Work the backlog (`ROADMAP.md`, currently §4b audit items + queued increments) in
   **increments**. Each increment: implement → verify (tsc + cargo test + browser) →
   add a `REVIEW.md` checklist entry → commit (and push once a remote exists) → send a
   completion report that leads with outcomes and proposes the next increment.
2. Jauhar replies **"go ahead"** to accept the proposal; anything else redirects.
3. He field-verifies against real well data via `REVIEW.md`: **`[x]` = accepted** (clicked
   through, works as described) / `[ ]` = not yet checked. If something is wrong he says so
   directly rather than marking it — it then gets fixed and logged in `ROADMAP.md` §4. Check
   for new `[x]` marks at session start. (The single legacy `[o]` at `REVIEW.md:4317` is the
   original mark style, superseded by `[x]`; do not read `[x]` as "wrong".)
4. **Git/GitHub**: the repo is private; credentials are Jauhar's own. Claude NEVER runs
   `gh auth login` or handles tokens/passwords — he authenticates himself, then Claude
   may create repos/push using his session. Commit messages: plain descriptive, avoid
   embedded double quotes (PowerShell 5.1 quoting).
5. Physics defaults come from documented sources (the reference suite `.info` exports, his studies,
   the chartbook) — cite the source in a comment; when a method spec conflicts with
   code, the specs in `docs/` win.

## Project layout

- `src-tauri/` — Rust backend: DuckDB access, parsers, IPC commands, petrophysics engine.
- `src/` — TypeScript frontend: WebGPU log canvas renderer, Tauri IPC calls.
- `src-tauri/icons/` — app icon set + brand assets: `logo.png` (master), `logo-mark.svg`/`logo-mark.png` (square monogram), `logo-full.svg`/`logo-full.png` (full lockup). Frontend favicon/ribbon assets in `public/`.
- `docs/` — method math + solver specs (SSC/SSPW, LRLC RtC/IMTS, workflow standards, the reference suite/IP multimin extraction), plus five reusable prompts, boundaries kept sharp (the table in `stewardship_prompt.md` is authoritative): `maintenance_scaling_prompt.md` (one increment — expand / debug / maintain), `engineering_review_prompt.md` (whole-app behaviour sweeps F1–F5), `qc_audit_prompt_template.md` (one tool end-to-end), `stewardship_prompt.md` (whole-repo structure + onboarding), `product_definition_prompt.md` (what the product IS — PRD, target architecture, v1.0 gate; licensed-product posture). Portable knowledge lives here, not in machine-local memory. Separate family, not in that table: the one-shot vendor-intelligence prompts (`sandibumi_maturation_prompt.md`, `techlog_ingest_prompt.md`, `sonar_ingest_adopt_prompt.md`).
- `tools/chartdig/` — chartbook vector digitizer (generates `src/ui/chartOverlays.ts`).
- `Prompt/` — original phase-by-phase spec (`Claude_Implementation_Guide.pdf`). **Gitignored** — local-only, won't exist on a fresh clone.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
