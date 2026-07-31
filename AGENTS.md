# SandiBumi — Petrophysical Software Engine

> **SandiBumi** (formerly *Arshilla*) — the project folder on disk is still `D:\XX. SandiBumi`; only the
> product/branding was renamed. The compiled binary is now `sandibumi.exe`, bundle id `com.sandibumi.petro`.

Desktop application for multi-well (2000+) petrophysical log analysis. Stack: **Tauri (Rust) + DuckDB (embedded, bundled) + TypeScript/WebGPU**.

This file is the Codex equivalent of `.cursorrules` (kept in this repo for Cursor). Keep both in sync if the rules change.

## Critical implementation rules

1. **Data storage**: optimize DuckDB queries for columnar reading. Use compressed binary blobs or `LIST` types for array logs (NMR, waveforms).
2. **Missing values**: never use `Option<f32>` for continuous logs. Missing data is strictly `f32::NAN`, so matrix arithmetic stays branch-free.
3. **Serialization & IPC**: never pass raw data arrays over Tauri's IPC bridge as JSON strings. Convert `Vec<f32>` matrices to raw bytes with `bytemuck`, return `Vec<u8>`, and cast to `Float32Array` on the frontend.
4. **Concurrency**: `rayon` for CPU-bound cell/well-parallel work; `tokio` for background async scheduling (long-running inversions, I/O).
5. **Code delivery**: concise, modular, production-speed-focused. No extensive unit test blocks unless explicitly requested.
6. **Writes are whitelisted**: the frontend never sends SQL for writes — only explicit commands (`db.rs` `TABLE_SPECS` + update fns). The SQL Query panel is read-only (SELECT/WITH only).
7. **Python equations run as a SUBPROCESS** (`python_engine.rs`), never PyO3/embedded — a missing Python must never stop the app from launching. Discovery: `SANDIBUMI_PYTHON` (the pre-rename `ARSHILLA_PYTHON` is still honoured so no existing setup breaks, but is never named in a message) → `%LOCALAPPDATA%\Programs\Python\Python31x` → PATH; requires numpy. **scipy is OPTIONAL** — when present the worker binds `scipy` plus `signal`/`interpolate`/`optimize`/`stats`/`ndimage` into the user-equation namespace (despike, Savitzky-Golay, resample, `curve_fit`); when absent each name is a stub whose first use raises a message naming the interpreter and the pip command, never a bare `NameError`. A CURVE MNEMONIC ALWAYS SHADOWS a scipy name — the user's data never yields. This is for the user's own equations; core petrophysics stays in Rust. `python_status()` probes numpy+scipy once per session so the editor can say so before a run. **DLIS import (`dlis.rs`) reuses the same subprocess mechanism** (`find_python` + a `dlisio` helper script), never a native parser; needs the `dlisio` pip package (installed: `dlisio 1.0.4` in the Python312 env). A missing `dlisio` fails only the DLIS import, with a clear message — never the app.
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
(portable — do not rely on any machine-local Codex auto-memory for it).

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

Shell UX (2026-07-19, superseded 2026-07-30) — The icon-only **quick access toolbar**
(`.qat`, left of the ribbon tabs) is GONE. Every one of its buttons is a LABELLED ribbon tool
in the **Project** tab, grouped Project (Open / New / **Save Project As…** / Recent ▾) — note
this reverses the old "Save Project As is NOT a ribbon button" rule — Session (Save Session… /
Open Session…), Edit (Undo / Redo), Monitor (History / **Processing** / **Performance**, both
moved out of the Petrophysics Batch group because they watch the whole application, not a
petrophysics run) and Help. Ids are `#undo-btn`/`#redo-btn`/`#save-project-btn`/
`#save-session-btn`/`#open-session-btn`/`#history-btn`/`#processing-btn`/`#health-btn`/
`#help-btn`. The tabstrip is 24px tall and the ribbon body 80px — that height difference is
the whole reason these could not carry captions before. **The unsaved-state dot is mirrored
onto the Project TAB itself** (`.ribbon-tab-dirty`), not only onto Save Session…: the button
now lives inside the tab, and a warning you only see after opening the tab that holds the fix
is not a warning. Keep it a `::after` dot, never a text prefix — a tabstrip that reflows when
work goes dirty shifts every other tab under the cursor.
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
stored in `documents` doc_type `session`. Project ▸ Session buttons `#save-session-btn`/
`#open-session-btn` → ribbon `handleSaveSession`/`handleOpenSession`. dockview toJSON does NOT
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

Curve draw style + crossover shading (2026-07-30) — `CurveStyle` gained three optional
fields, all `#[serde(default)]` so every pre-existing saved layout still loads:
`draw_style: "line" | "step"` ("step" holds each sample's value down to the next sample's
depth, then jumps — the honest display for block-averaged / zone-constant / coarse curves,
where a diagonal would draw a gradient the data never measured), and `fill: "curve"` +
`fill_to` + `fill_color2` for **crossover shading** between two curves in the SAME track.
The reference curve is positioned with **its own min/max** — compatible scaling is the
entire meaning of a neutron-density crossover, so `fill_to` naming a curve outside the
track resolves to nothing and shades nothing. `fill_color` = where the styled curve reads
LEFT of the reference, `fill_color2` = RIGHT. Both renderers implement all of it and must
stay in agreement: `LogCanvasRenderer.buildCrossoverGeometries` (two fill-only geometries,
one colour each, same split `buildBlockGeometries` uses because the fill pipeline binds one
colour uniform per draw) and `composite.rs draw_crossover`. Where the pair crosses INSIDE a
sample interval both split that quad at the crossing so the colours meet on the crossover.
The two curves need not share a sampling — the viewer interpolates the reference onto the
styled curve's depths via `makeSampler` (NaN outside its range or across a NaN gap;
separation is never inferred across a gap), while the composite path already has one shared
depth column. Built-in Standard and Facies layouts now ship the NPHI/RHOB crossover.
Same-session fix: `composite.rs` used to treat ANY `fill` string other than "right" as a
left-edge fill, so a style saved with `fill: "none"` printed shaded while the screen showed
it clean — the match is explicit now (`left`/`right` only).

Point-data tracks (2026-07-30) — `TrackKind::PointData` + `Track.points: Vec<PointStyle>`
(both `#[serde(default)]`, so every older layout loads). A point series is deliberately NOT
a `CurveStyle`: a curve has a value at every depth, a point series only where somebody
sampled, and joining core plugs with a line would state a continuity the data lacks.
`source` is `"core"` (a plug property of the ACTIVE core set, via the new
`db::get_core_point_series` — NULL cells are DROPPED, never read as 0) or `"aux"` (one item
of one point dataset, via the active-set-filtered `list_aux_data`); an aux row carrying
`depth_base` is anchored at the interval's MIDDLE. Four displays: `points`, `text`
(value_text labels, deduped so a densely described core is readable), `box`, `histogram`.
Drawn on the 2D overlay in `logViewPanel.drawPointTracks` (the renderer skips point tracks
for GPU geometry exactly as it skips well-diagram tracks, but still allocates the column)
and in `composite.rs draw_point_series` for print. Off-scale samples are SKIPPED, never
clamped to a track edge — same rule as the core overlay.

**`distribution.rs` + `distribution.ts` are the shared statistics core and must stay in
agreement** (percentile = R type 7 / NumPy / Excel; Tukey whiskers land on a real sample,
never the fence; a percentile whisker rule reports no outliers; histogram DROPS out-of-range
rather than clamping; empty depth bins are omitted). They are **source-agnostic on purpose**
— they take a bare value slice — because binning core plugs over an interval and summarising
N Monte Carlo realizations at one depth are the same operation. Array logs DO reuse them
unchanged (see below); if that ever needs its own code path the abstraction has been broken.
The Rust side carries the unit tests that pin the numbers.

Array logs (2026-07-30) — `array_logs` is now a real store: one row per
`(well_id, set_name, curve_name, depth)` holding a whole VECTOR of values at that depth
(Monte Carlo realizations, NMR T2 distributions, waveforms). The never-written stub
`(well_id, depth, nmr_t2_distribution FLOAT[])` is dropped by `db::migrate_array_logs_store`
(no backup taken — no code path ever wrote a row to it). **`samples` is a BLOB of explicit
little-endian f32, not a DuckDB list**: it is 4 bytes per value with no text round-trip, and
rule 3 already requires arrays to reach the frontend as bytes cast to a `Float32Array`, so
the stored bytes ARE the wire format. **This table DOES carry a PRIMARY KEY, unlike
`computed_curves` — not an inconsistency**: that ART index costs one entry per SAMPLE, here
one row holds a thousand samples, so it is ~1000x cheaper per value while the protection
matters far more (a duplicate depth row would silently double a realization count and bias
every percentile). Write discipline is still DELETE-then-insert per (well, set, curve).

`montecarlo.rs` gains `persist_realizations` (requires `persist`; off by default) writing
`MC_<KEY>_REAL` alongside the existing `MC_<KEY>_LOW/_P50/_HIGH/_BASE` curves, capped by
`realization_cap` (default 256, clamped 8..1024 — the full 1024 at ~2000 samples is ~8 MB per
curve per well, i.e. 3 GB across a field). **Realization ORDER is preserved and NaNs are
kept**: index `r` must mean the same realization at every depth or a spaghetti trace is not a
trace (the sorted percentile buffer must never be what gets stored). The `>= 8 finite` floor
matches the percentile curves', so stored depths and the persisted curves never disagree
about where an answer exists — pinned by
`persist_realizations_stores_a_matrix_that_reproduces_the_percentile_curves`. Matrices go to
`array_logs`, NOT the versioned archive that holds the curves: the archive exists to make
re-runs non-destructive, and versioning data this size would balloon the file.

`TrackKind::ArrayLog` + `Track.arrays: Vec<ArrayStyle>` (both `#[serde(default)]`) give three
displays over ONE stored matrix — `band` (adjustable low/high percentiles + optional P50
line), `spaghetti`, `heatmap` — which is the whole point: **the percentiles are a display
setting, not a reason to re-run the study**. Drawn in `logViewPanel.drawArrayTracks` (2D
overlay; the GPU renderer allocates the column and skips it, exactly as for point and
well-diagram tracks) and `composite.rs draw_array_series` for print — **keep the two in
agreement**. Rules: values CLAMP at the track edge (continuous data, unlike a point sample
which is skipped); a depth where nothing converged is a GAP that splits the band rather than
being spanned; a failed realization BREAKS its own spaghetti trace rather than being bridged;
heat-map density is opacity of the series colour normalised to that depth's own peak (no
second palette to keep in sync), and out-of-range values are dropped by `histogram`, never
clamped. Spaghetti traces come from `distribution::even_indices`, spread evenly rather than
the first N — the first N of an LHS design is a biased corner of the sampled space.

Image tracks (2026-07-31) — `well_images` + `image_sets` are a real store for
depth-registered PICTURES: petrographic thin sections, core photographs, SEM plates, FMI
snapshots. Deliberately its own table rather than an `aux_data` item — an aux row carries one
number or string, and putting megabytes in `value_text` would drop a blob into every
point-data scan. It follows the universal delivery-set rule (`db::ACTIVE_IMAGE_SET`,
correlated on `i.dataset`, so ONE delivery of each dataset is live); **this table carries a
PRIMARY KEY, and that is the `array_logs` argument again, not a `computed_curves`
inconsistency** — one index entry per PICTURE is free, and a duplicate row would print the
same plate twice.

`depth_base IS NULL` means a POINT sample and it is a petrophysical statement, not a missing
field: a thin section is cut from one plug and has no thickness, so it is ANCHORED at its
depth rather than stretched over a guessed interval. A core photograph delivered with a base
depth spans it for real. That distinction is what `ImageStyle.mode` selects (`anchor` /
`depth`), and where two plates would overlap at the current scale **the deeper one is SKIPPED,
never nudged** — a plate moved to make room is a plate attributed to the wrong sand — leaving
a depth tick, and zooming in reveals it. Aspect ratio is NEVER distorted: `fit` is
`contain` or `cover` (crop), and there is deliberately no stretch, because a squashed thin
section misstates grain shape, which is the one thing the plate is there to show.

**`data` is a normalized DISPLAY copy, not the delivered original** — a capped JPEG produced
by `images.rs` through ONE Pillow subprocess for the whole delivery (rule 7: subprocess,
never embedded); `source_path` + `src_width`/`src_height` keep the original traceable. Long
edge defaults to 2400 px at q85, adjustable in the wizard because it is the user's trade-off
between project size and zoom, not a constant to hide. Without Pillow the import still works
for anything the WebView decodes (stored verbatim) and only JPEG is `printable`; TIFF needs
Pillow and says so by name. Import is **probe → confirm → commit** like the core wizard:
`parse_depth_from_name` guesses a depth from each filename (a token qualifies only with a
decimal point or ≥3 integer digits, so the `01` of `BLSO-01` is never read as 1 m; an
adjacent increasing pair separated by one `-`/`_` becomes an interval) and every guess is
shown in an editable table before anything is stored.

`TrackKind::Image` + `Track.images: Vec<ImageStyle>` (both `#[serde(default)]`). Geometry
lives in **`composite.rs image_box` and `logViewPanel.imageBox`, which must stay in
agreement** — the print has to place a plate where the screen did. The viewer holds
METADATA only (`list_well_images` never selects the blob) and fetches pixels per plate as
they scroll into view, capped at `IMAGE_CACHE_MAX` bitmaps that are `close()`d on eviction
and on dispose. **PDF export embeds the JPEG bytes UNTOUCHED** via a `/DCTDecode` image
XObject — `assemble_pdf_with_images` builds object bodies as BYTES (a JPEG is not valid
UTF-8, and base64/hex would inflate a photographed core by a third); `assemble_pdf` still
delegates with an empty list and is pinned byte-identical by
`a_page_with_no_images_writes_the_same_pdf_it_always_did`. `report.rs` must collect images
too — it embeds the composite pages verbatim, so forgetting would reference plates it never
wrote. SVG export inlines a base64 data URI so a delivered file is self-contained. A plate
the PDF cannot embed prints a **named frame**, never a silent gap, so a deliverable can be
checked against the delivery list. Digitizing the plates (OpenCV) is a deliberately separate
later phase; nothing here decodes pixels in Rust.

Office deliverables (2026-07-31) — `office.rs` writes the study as a formatted multi-sheet
**Excel workbook** (Plot ribbon -> Deliverables -> Workbook...), the first consumer of a shared
Python-office spine. `office_support()` probes xlsxwriter/python-docx/python-pptx/openpyxl in ONE
subprocess so a dialog can say what is missing *before* a save dialog appears, and name the
interpreter to install into. **Rule 7 throughout**: the workbook is written by a subprocess, the
native PDF/SVG/LAS paths stay the default, and a missing package fails only this button -- which
is also why the real round-trip test is `#[ignore]`d, so the green gate can never depend on it.

**The runner is deliberately dumb.** Every petrophysical decision is made in Rust and arrives as
a `Sheet` of typed `Cell`s (`Num` / `Text` / `Blank`); the Python side only knows how to draw a
table. So the workbook and `report.rs`'s PDF are two renderings of ONE decision rather than two
implementations that can drift -- the Pay Summary sheet is deliberately the same rows, same
flags, same conventions as the printed table.

Two rules govern the numbers. **Numbers stay numbers**: a cell carries the value with a number
*format*, never a preformatted string, because a text column cannot be pivoted or re-averaged,
which is the only reason to want a workbook. **A blank is not a zero**: where `n_classified == 0`
the well was never interpreted over that zone and net/N-G/HPV are 0 for want of an answer, not
because the sand is wet -- the PDF prints "-", the workbook leaves the cell EMPTY, which is the
one value Excel's own AVERAGE/COUNT skip. `Cell::Blank` serializes as JSON `null` and the runner
SKIPS it; do not "helpfully" write 0 there. Gross is geometry and stays a number regardless.

The **Field Summary** sheet carries TWO N/G columns on purpose, because they answer different
questions and quoting one as the other is a reserves error: `N/G (field)` is sum(net)/sum(gross),
the volumetric ratio, while `Mean N/G` is the average of the per-well values, which is what the
Field Dashboard plots. PHIE and SWE are **net-weighted** (matching `dashboardPanel.ts`'s
`weightedMean`), zones are ordered **shallow to deep by mean top**, and wells that were never
interpreted are counted in their own column rather than dragging an average toward zero. The
export runs `stats_only` -- saving a spreadsheet must never write FLAG curves or version a log
set, the same reasoning the dashboard follows.

Word twin + the stdin encoding rule (2026-07-31) - `office.rs` also writes the **editable
.docx twin** of `report.rs`'s PDF (report pane -> **Save Word...**, and the Batch button's
format select for one file per well). It carries the cover, the pane's editable methodology
table, the zone parameters in the PDF's shape and the pay summary; **the composite log pages
stay in the PDF on purpose** - they are drawn at a true print scale, and a picture pasted into
a document stops being at that scale the moment anyone resizes it.

A document `Block` reuses the workbook's `Sheet`/`Column`/`Cell` model, so **one table
definition is rendered three ways** (PDF, workbook, Word) and cannot drift. The one deliberate
divergence: `Cell::Blank` prints as a DASH in the Word document (`Block::Table.blank_text`) but
stays an EMPTY cell in the workbook. Same decision, two correct renderings - Excel's arithmetic
skips a blank, a document has no arithmetic and a reader's eye needs the mark the PDF prints.
Like the workbook the Word export runs `stats_only`, so unlike the PDF path it writes nothing
back to the project (the pane skips `bumpDataVersion` for it).

**Every Python runner MUST read `sys.stdin.buffer`, never `sys.stdin`.** A piped child's TEXT
stdin decodes with the Windows ANSI codepage (cp1252 here) while `serde_json` emits raw UTF-8,
so any non-ASCII character arrives as mojibake - an en dash in a well name came out as three
junk characters, and a Greek rho came out as a byte pair plus a lone surrogate. `json.loads`
accepts BYTES and assumes UTF-8, which is what was actually sent. `ml.rs` and
`python_engine.rs` always did this correctly; `images.rs` did NOT - a plate whose path held any
non-ASCII character failed with a bare "No such file or directory" naming a filename nobody
had - and is fixed here. Pinned by `a_word_document_keeps_non_ascii_text_intact` (ignored,
needs python-docx). Same family as `parsers::read_text_file`: bytes must be interpreted, never
assumed.

Asset-team deck (2026-07-31) - `office.rs` also writes the **PowerPoint deck** (Plot ribbon ->
Deliverables -> **Deck...**, `deckDialog.ts`), completing the office set. Slides are built from
the pay-summary DATA with matplotlib figures, **deliberately not from composite pages**: a
composite is drawn at a true print scale, and python-pptx embeds PNG/EMF rather than vectors, so
a pasted log plot would be a picture that stops being at 1:200 the moment anyone resizes it.
That was Jauhar's explicit call when the choice was put to him.

Slides: title, scope-and-cutoffs, field roll-up by zone (the workbook's `field_sheet`, a FOURTH
rendering of the same table definition), net + HPV per zone, N/G / PHIE / SWE distributions, a
well ranking by HPV, and a closing slide **naming every well that produced nothing** - the
honest counterpart to every average before it. `DeckSpec.flag` picks ONE cutoff level (default
PAY) and the title slide says which; mixing PAY with SAND on one axis would be three questions
in one picture. 16:9, one subprocess for every figure and the deck.

Three rules the runner must keep. **Box statistics are computed by `distribution.rs` and passed
in matplotlib's `ax.bxp` vocabulary** - never `ax.boxplot(raw)`, which would apply matplotlib's
own percentile convention and make the deck disagree with the Field Dashboard for the same
wells; `BoxSpec.n` rides along because a box drawn from three wells is not the statement a box
from ninety is. **A `None` in a `Series` is a gap, not a zero** - the zone still gets its axis
label but no bar, the same statement the workbook's blank cell makes. **Long tables continue on
further slides** (`DECK_ROWS_PER_SLIDE`) and the well ranking is capped at `DECK_RANK_WELLS`
with the cap stated in the slide note - a deck is read from across a room, and a silent top-N
reads as "all of them". Like every other export here it runs `stats_only` and writes nothing
back.

Saved ML models (2026-07-31) - a fitted model is an ARTIFACT now, not a by-product.
`MlRequest` carried the training wells and the apply wells in ONE call, so the model died with
the subprocess: there was no way to train on the cored wells and apply THAT SAME model to the
rest of the field later, and "which model produced this PERM curve?" had no answer. A refit on
different data is a different model.

`ml_models` (schema in `db.rs`, picked up by `create_schema` on every open - no migration) holds
a joblib dump in `data`, plus the description every reader needs: task, algorithm, ORDERED
feature curves, target, params, metrics, the wells that actually contributed, n_train,
standardize, the scikit-learn version that wrote it, and a note. **PRIMARY KEY is the
`well_images` argument again** - one index entry per MODEL is free, and a duplicate would make a
cited model ambiguous. `list_ml_models` NEVER selects `data`; only `apply_ml_model` fetches it.
Names are unique and auto-suffix like a delivery set (`PERM_RF` -> `PERM_RF_1`): **retraining
makes a NEW model rather than replacing the one an existing curve was made with**, which is the
provenance the whole feature exists for.

Three contracts. **The scaler travels with the estimator** in the same dump - refitting a
StandardScaler on the apply wells would be a different transform, and the predictions would be
quietly wrong rather than obviously broken. **Feature ORDER is part of the contract**: the
artifact carries its own feature list, `apply_ml_model` drives the fetch from the MODEL's list so
a caller cannot restate or reorder it, and `ML_APPLY_RUNNER` re-checks inside the artifact and
REFUSES rather than predicting - a model fitted on [GR, RHOB] fed [RHOB, GR] returns confident
nonsense nothing downstream can catch (pinned by
`a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order`). **Saving never fails a run**:
the model is stored AFTER the curves are written, so a storage problem costs the artifact, not
the work, and says so in `MlResult.notes`.

Supervised only. Clustering and reduction are fitted on the very wells they are applied to by
construction, so "apply it later" would mean something different - and supervised classification
with a FACIES/litho target is already the train-on-cored-wells route. Applied curves record
`ml:apply:<model name>` with the model id in provenance. UI: "Save model as" (supervised only)
plus a **Saved models** list in `mlDialog.ts` with Apply to scope / Rename / Delete.

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
backlog, priority-ordered P0–P3**; Phase 9 remaining: lazy catalog loading / decimation
cache / UI responsiveness + 2000-well stress fixture — write-path perf and the **per-well
parameter override table** (`wellParamsDialog.ts`, 2026-07-30) both done; Phase 10 done:
facies + block track + GMM + full ML suite — supervised facies = ML classification with a
FACIES/litho target; remaining ML deferral: autoencoders). Shipped 2026-07-30: multi-well
context overlays on crossplot/histogram/Pickett (T-SHELL-16, shared machinery in
`plotCommon.ts`), Pickett v2 completion, and the **MID plot module** (`lithology.rs` —
UMAA/RHOMAA feeding the Lith-6 overlay, with a real Por-11 crossplot-porosity lookup).
Open follow-ups: CSV import into the per-well parameter grid, and a per-well colour-stability
rule for the multi-well overlays.
Import sets shipped (2026-07-30, T-IMP-02) — the Geolog/IP curve-set model. `ingest.rs`
`LasImportOptions {set_name, attach}`: Import LAS opens a set dialog (`importSetDialog.ts`,
suggestion from the filenames' common token — `blso*_fprooh.las` → FPROOH); with attach ON
(default) a file whose well name matches exactly ONE existing well lands as a new named set
on that record (generic store only — `standard_curves` is never touched on attach); set
names auto-suffix per well (`FPROOH` → `FPROOH_1`, never overwrite). DLIS import asks for a
set too (RAW = legacy replace-and-count path, anything else suffixes). Wells pane ▸ twisty
expands well → sets → curves (`objectTree.ts`, lazy per-well catalog). **Resolution
contract: set RAW keeps ABSOLUTE priority in `fetch_generic_curve_aligned` — only a
mnemonic RAW lacks falls through to attached sets (ordered by set_name). Do not weaken
this; it is what keeps every pre-set-era project byte-identical.** Example datasets:
`dataset for test/examples/` (generated by `tools/make_example_data.py`, parsed as fixtures
by `example_data_test.rs` on every gate run — includes the malformed `bad_*.las` pair).
Core/aux import v2 shipped (2026-07-30, T-IMP-07/-10/-11) — probe → confirm → commit.
`parsers::probe_core_table` (role guesses incl. WELL/WN with prefer-textual candidate
choice, per-column type sniff, units-row skip, percent + depth-unit detection) +
`ingest::import_core_table` (rows route per normalized well name, exactly-one-match rule,
unmatched/ambiguous reported never guessed, blank cells skipped, ft↔m conversion to the
project unit, per-well replace + dedup). `coreImportDialog.ts` confirms the mapping once
BY HEADER NAME and re-resolves per file (multi-select, `.txt`/tab/semicolon sniffed).
Aux imports route by a WELL column the same way. Jauhar's standing requirement: **BLSO is
an exemplar, not the spec — imports must accept any delimited text with mixed column
types**. Completed by `CoreMapping.extras` (2026-07-30): columns no core role claims are
carried into `aux_data` from the same wizard — `parse_core_table_mapped` returns them as
RAW TEXT + header names (`MappedCoreTable`), `import_core_table(..., extras_dataset)`
(default "CORE") writes them at the converted plug depths, typed PER CELL (numeric →
value_num, else value_text), blank cells skipped, riding the plugs' depth-dedup so they
stay aligned; replace-on-reimport per (well, dataset). **Extras are stored VERBATIM — no
percent/unit conversion; do not add one silently.** `extras` is `#[serde(default)]`, so
older IPC payloads still deserialize.

Delivery sets are UNIVERSAL (2026-07-30, T-IMP-08/-12) — **every non-curve store** follows
the set model: core plugs, SCAL Pc, deviation surveys and every point dataset (`aux_data`:
XRD, CEC, oil show, petrography, perforations, core extras), with a deliberately DIFFERENT
resolution rule from curves: curve sets are read together (RAW priority, others fill gaps),
but two such deliveries measure the SAME plugs or samples, so **exactly ONE core set, ONE
SCAL set, ONE survey and one set per (well, dataset) are active, and every reader follows
them**. `core_data.set_name` (PK well_id,set_name,depth) + `core_sets`;
`well_path.survey_name` + `well_surveys` (active/source/datum/imported_at);
`aux_data.set_name` + `aux_sets` (PK well_id,dataset,set_name); `scal_pc.set_name` +
`scal_sets`. On the PK-less tables (`aux_data`, `scal_pc`) `set_name` is the LAST column —
the Appender is positional and migrated DBs get it appended, so fresh and migrated schemas
must agree. **All such reads go through the shared SQL fragments `db::ACTIVE_CORE_SET` /
`ACTIVE_SURVEY` / `ACTIVE_AUX_SET` / `ACTIVE_SCAL_SET` (the aux one correlates on
`a.dataset`, so one query spans every dataset and still sees one delivery of each) — a
reader that forgets the filter silently unions two deliveries and doubles a φ-k cloud, a
mineral count or a Pc curve; keep new readers on the fragment.** Core EXTRAS are written under the core set's own name, so switching a
well's core switches its extras with it. `db::migrate_point_data_sets`
rebuilds pre-set-era projects (rows become RAW/active — byte-identical readings; core/well_path
are rebuilt for the PK, aux_data/scal_pc only ALTERed + back-filled + registered), idempotent,
backed up per RELEASE §3.2, wired into `project::open_and_migrate`. Imports take a name
(`import_core_table(..., set_name)`, `import_deviation_csv(..., survey_name)`,
`import_aux_file(..., set_name)`, `import_scal_files(..., set_name)` — the files selected
together are ONE delivery), resolved PER WELL via `resolve_core_set_name`/`resolve_survey_name`/
`resolve_aux_set_name`/`resolve_scal_set_name` (auto-suffix, never overwrite), and the new set
becomes active. **`set_active_survey` MUST
re-materialize TVD/TVDSS** (the Tauri command does) — stale stored TVD would keep feeding
height calculations the old geometry. `shift_core_depths` and `update_core_sample` act on the
ACTIVE set only. UI: `dataSetsDialog.ts` (Data → Tools ▾ → Data Sets…, four sections) + the
Wells-pane ▸ tree, which lists Core / SCAL / Surveys / Point data under each well with ● on
the live one and **double-click** to switch (single click is inert on purpose; delete stays
in the dialog).

## Open-path hardening (2026-07-30 — from the BLSO 2.5 GB field report)

Every file-backed open runs `db::tune_connection`: DuckDB's memory_limit is capped at
default/4 clamped to [1 GiB, 4 GiB] (the engine default is ~80% of RAM — it ate ~6 GB of an
8 GB field machine); `SANDIBUMI_DB_MEMORY` overrides verbatim. `db::engine_copy_to`
(ATTACH + COPY FROM DATABASE) is the ONE copy primitive — it writes live rows only, so every
copy is compacted; migration backups, **Save As** and **Compact Project** (Data → Tools ▾)
all go through it. `project::compact_project` rewrites the project at the same path:
engine copy → row-count verification over EVERY catalog table → connection swap under the
DbState mutex → original parked as `.pre-compact-<ts>.duckdb` (never deleted by us); any
failure renames the original back. User-visible one-time events (migration backups, memory
cap, compaction, a ≥10 s open) go through `db::boot_note` → the `boot_report` command →
status line + process history — **never eprintln alone; a built exe has no console, which is
how a 15-minute one-time migration looked like a hang.** DuckDB files never shrink on
DELETE (module re-runs bloated BLSO to ~4× its live size), so point users at Compact
Project when a long-lived project drags.

## RtC calibration (2026-07-31)

`sw_rtc`'s coefficients are a REGRESSION, not a constant, and `lrlc::run_rtc_fit` (Advance ▸
Calibrate RtC…, `rtcFitDialog.ts`) fits them to the user's own water leg. Four rules.

**The regression is the algebraic inverse of `sw_rtc`'s own equation, never a re-derivation
from the method note.** Set Sw = 1 in `Sw = [Rw·(1/Rt − Cex)/φt^M]^(1/N)` and the measured
excess falls out as `1/Rt − φt^M/Rw`; dividing by `φt·RSF` gives a plain 3-parameter OLS in
(CAPBW, Qv, 1). Deriving it this way means a future change to the saturation equation breaks
the fit visibly instead of leaving a calibration that quietly no longer inverts it. `qv_at()`
is shared by the module and the fit for the same reason.

**A water zone must be DECLARED — the fit refuses without a depth range or a wet-flag curve.**
There is no way to find a water zone without already knowing the saturation the calibration is
for, so inferring it would beg the question. The stakes are asymmetric: hydrocarbon REMOVES
conductivity, so pay samples make the apparent excess too small, the fitted model
under-predicts excess, Rt is under-corrected and Sw comes back too HIGH — it erases pay rather
than inventing it. A NaN wet flag is not wet.

**The `Cex <= 0` rejection is a second line of defence, not a substitute, and the tests record
exactly how it leaks**: it drops most obvious pay, but where the rock is most microporous the
true excess is large enough to mask the hydrocarbon and the sample survives — the guard is
weakest precisely where this method is used.

**RSF is held fixed and is not fitted.** It multiplies the whole bracket, so (a, b, c, RSF) are
not jointly identifiable; the returned coefficients belong to the RSF they were fitted with and
the result says so. An unfittable term (constant Qv) is reported as 0 with a note, never
guessed, and every excluded sample is counted and named. The dialog offers **Copy**, not
auto-apply — a calibration is a judgement made after reading R² and the exclusions.

## IMTS S-factor calibration (2026-07-31)

`sw_imts`'s S is the RtC problem again: it is *defined* as a measurement — S = lab CEC / XRD-
theoretical CEC (`docs/method_lrlc_rtc_imts.md`, IMTS §1) — and the app shipped a placeholder
for it. S multiplies the entire clay-charge term, so a wrong S scales Qv_eff directly and moves
SwT with nothing on the log to show for it. `lrlc::run_s_factor_fit` (Advance ▸ Calibrate S…,
`sFactorFitDialog.ts`) fits it from the user's own core. Five rules.

**The regression is the algebraic inverse of the module's own line, not a re-derivation** —
same discipline as RtC. `sw_imts` computes `cec_bulk = S · cec_theo_at(vk, vi, CEC_KAOL,
CEC_ILL)`, so `S = CEC_lab / cec_theo_at(...)`, and `cec_theo_at` is **shared by the module and
the fit** exactly as `qv_at` is shared with the RtC fit. Pinned by
`the_fitted_s_makes_the_module_reproduce_the_measured_cec`, which runs the fitted S back through
`sw_imts` and checks QVEFF lands on the laboratory value.

**The clay must come from the curves the RUN will use, not from the XRD table.** This is the
trap the dialog exists to close: calibrate S against XRD weight fractions, run against a
VDCL-derived VKAOL curve, and S is wrong by the ratio between those two estimates of clay —
silently, because both look like clay volumes.

**Through the origin, and least squares rather than the mean of the ratios.** S is a pure
scaling factor; an intercept would assert cation exchange where the clay model says there is no
clay, a claim the module's equation has nowhere to put. Through-origin OLS weights each plug by
its clay content, which is right — those are the plugs where Qv drives the answer, and on a
nearly clean plug the ratio is measurement noise over a small number.

**The drift detector is the SPREAD of the per-plug ratios (P10-P90), not the median-vs-fit gap.**
Two central values can only differ by as much as the ratio changes between the median plug and
the clay-weighted one; on a 12x clay range with a ratio running 1.13 → 0.40 that is barely 28%,
so a gap threshold loose enough to survive noise never fires on real drift. The spread has no
such ceiling and catches the same case at 2.8x. Both are reported; the gap note is secondary and
says only that the disagreement is *systematic with clay content*. Pinned by
`a_drifting_s_shows_up_in_the_spread_of_the_per_plug_ratios`.

**S above 1 is a note, never a clamp.** The method expects lab CEC *below* the XRD-theoretical
value. Above 1 the clay model is under-calling exchange capacity, and the usual cause is a
mineral it does not carry — smectite runs 80-150 meq/100g against illite's 25, so a few percent
dwarfs the modelled charge — which makes that S wrong wherever the missing mineral's fraction
differs from the cored plugs.

Two further contracts. A plug further than `depth_tol` (default 0.15, one standard 6-inch
sample) from any log sample is **dropped and counted, never snapped** to the nearest one — but
the test records the honest limit: a shift that is a whole number of sample intervals is
invisible to any depth-tolerance check, so this is not a substitute for depth-shifting the core.
And **S and the literature CEC constants are not jointly identifiable** (S multiplies them), so
the constants are held fixed, echoed in the result and copied alongside S.

**Calibration QC scatter** — `fitScatter.ts` is shared by both fit dialogs, because a calibration
reduces a core or a water leg to two or three numbers and R² cannot say *how* it failed:
curvature, one well parked off the trend, a cluster the fit is being dragged by. That is what both
backends return `points` for. Two rules are the reason it is one module rather than two plots.
**A measured-vs-fitted plot forces both axes to the SAME range** so the 1:1 line lands at 45° —
scale them independently and the aspect ratio alone makes a good fit look biased or a biased one
look clean. **A through-the-origin plot forces the origin onto the page**, because proportionality
is the model's claim and cropping to the data hides whether the cloud actually heads for zero.
Points are coloured by WELL (a single well pulling a field calibration is the question the table
cannot answer), out-of-window points are SKIPPED rather than clamped to an edge, and the hover
readout names the well and depth. RtC plots measured against fitted; the S fit plots the
regression itself, lab CEC against modelled CEC, because with one predictor only that version puts
clay content on the x axis and turns the P10-P90 spread into a shape with a name.

**The first paint is synchronous and must stay that way.** `requestAnimationFrame` only fires
while the tab is compositing, so deferring it leaves the plot blank in an occluded or background
window — and `attachResizeRedraw` schedules through rAF too, so there is no fallback. The handle
exposes `redraw()` and each caller invokes it right after inserting the element. Related and
equally load-bearing: the canvas context is scaled by the `dpr` that `fitCanvasBackingStore`
returns, or a HiDPI screen draws the whole plot at half scale in the corner.

**Picking the CEC measurement** — `db::list_aux_item_catalog` returns every measurement name in
the project's point data (from the ACTIVE delivery of each dataset) with its row count, well count
and **numeric-row count**, and the S dialog turns it into two dependent selects. Project-wide and
unfiltered by well for the same reason `list_well_param_overrides` is: one grouped scan beats N
round trips or an `IN (...)` list long enough to hit a binding limit on a 2000-well project, and
"what could this box name" is the question a picker actually asks — a run's own exclusion counts
still report what the chosen wells turned out to hold.

`numeric_rows` is the part that matters. A descriptive item cannot set a scaling factor, so a
text-only one is shown **greyed with "no numeric values"** rather than hidden: "LITHOLOGY is
there but it is text" answers the question the user was about to ask by running the fit. A dataset
with nothing numeric in it gets an explicit "(nothing numeric in this dataset)" placeholder rather
than a silently empty select. With no point data at all the dialog falls back to typed names and
says so in a VISIBLE note — `formRow`'s hint is a tooltip, and "there is nothing here to pick
from" is not something to hide behind a hover.

**Accepting a calibration** — `calibrationApply.ts`, shared by both fit dialogs, writes the
coefficients as `zone_params` overrides through the new atomic `db::set_zone_param_batch(conn,
zone_name, entries)`. `set_well_param_overrides` is now just its `"*"` scope, so the parameter
grid and an accepted calibration take the same transactional path. Four rules.

**The default scope is `wells_fitted`, a field on both results and NOT derived from `points`** —
the display points are decimated, so a well can vanish from them entirely, and a scoped well that
contributed nothing was never calibrated. Applying to the wider scope is offered (fit-here-apply-
there is the point of a field calibration) but it is a choice, it names the uncalibrated wells,
and the option is hidden when it would be identical to the default.

**The held-fixed constants are written in the same batch or not at all.** RtC writes RSF with
A_CAP/B_QV/C0, the S fit writes CEC_KAOL/CEC_ILL with S_FACTOR. In both cases the constant and the
coefficients are not jointly identifiable, so writing one without the other yields a calibration
that is silently for different rock.

**One transaction, one undo.** A half-applied saturation calibration would leave a field carrying
two answers with nothing on the log to say where the boundary fell.

**Undo restores "no override", not zero.** The previous values are read first — from
`list_well_param_overrides` for `*` (one project-wide query) or `list_zone_params` for a named
zone — and a `None` in the batch DELETEs the row. A parameter silently pinned to zero is a wrong
answer that keeps computing. Pinned by
`a_none_in_a_zone_batch_clears_the_row_instead_of_writing_zero` and
`a_named_zone_batch_leaves_the_whole_well_scope_alone`.

Both fit dialogs offer **Copy** as well as Apply. Both also paint their own run-button label:
`buildWellScope` deliberately does not fire `onChange` during construction (`wellScope.ts`), so
a caller that relies on it opens with a blank, disabled button — `rtcFitDialog.ts` did, and is
fixed here.


## Core-to-log depth registration (2026-07-31)

`registration.rs` (Data ▸ Tools ▾ ▸ **Register Depth…**, `depthRegDialog.ts`) proposes the constant
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

## Plate scale and preparation (2026-07-31 — D4 answered)

Jauhar's answer to D4 was **"sometimes"** on both counts: sometimes the section states a scale,
sometimes not; sometimes it is epoxy-impregnated and stained, sometimes not. A uniform answer
either way would have been easier — this one means one delivery holds plates of both kinds, so
`well_images` gained `fov_um`, `prepared` and `stain` **per plate**, all defaulting to absent, all
DECLARED and never inferred (`db::migrate_plate_scale_and_prep`, ADD COLUMN only, no backup;
existing plates get NULL, which is the honest answer).

**Scale is entered as a FIELD OF VIEW WIDTH, not micrometres per pixel.** The stored copy is
resampled to a long-edge cap, so a um/px belongs to whichever copy it was measured on and nothing
in the number says which — while "this picture is 2.5 mm across" is true of every copy of it. um/px
for any copy is `fov_um / that copy's pixel width`, which is what the readout derives. It is also
the form a petrography caption already states. There is **no default**: §3's "no default um/px,
ever" now has teeth, because absent is the normal case rather than a corner.

**Anything dimensional must REFUSE an uncalibrated plate rather than report pixels.** A D50 in
pixels is not a D50, and a number with the right name and the wrong unit is the same failure as a
wrong `m` — it computes, it plots, it ships. A run over a mixed delivery reports how many plates it
skipped and names them; a silent subset looks exactly like a complete answer. Family A (area
fractions) is unaffected, which is why it stays first.

**`prepared` unknown is REFUSED, not assumed either way.** This is the sharper rule, because the
failure is silent in both directions. A blue-epoxy pore rule run over a section nobody impregnated
does not fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge
artefact, which then plots against core helium porosity as though it meant something. Detecting
impregnation from the pixels is the same circular move as detecting a water zone from the
saturation being calibrated: the evidence for "this is blue epoxy" is the blue about to be
measured. `stain` is FREE TEXT for the same reason the RtC water zone is declared — which stain
was used is the laboratory's fact, and a menu invented here would be a protocol nobody performed.

**Delivery-level values fill the blanks; what is stored belongs to the plate.** Magnification
genuinely varies within one delivery — that is the whole content of "sometimes" — so the import
wizard takes one field of view for the delivery plus a per-plate **FOV mm** column that overrules
it, and `ImageImportItem.fov_um` (`#[serde(default)]`) carries the override. Preparation is taken
delivery-wide at import (one impregnation run, one staining bath) but stored per plate so a mixed
delivery can still be corrected.

`src/ui/plateDetails.ts` is the one control, shared by the import wizard and the plate editor — the
same decision, and two copies is two places for the wording to drift (the `followCore.ts`
argument). `db::set_image_details` writes one plate, `db::set_image_delivery_details` writes a whole
live delivery in one statement (the `shift_well_images` argument: a core-photograph delivery is
hundreds of plates). **Every value is written as given, `None` included** — a wrongly typed scale
has to be clearable, and one that cannot be removed is worse than one never entered, because
everything downstream believes it. The delivery-wide button REFUSES "All datasets": that would give
a core photograph the thin sections' magnification. Its undo restores plate by plate, because the
plates need not have agreed before and writing one value back across the delivery would invent a
uniformity that was not there.

Data ▸ Tools ▾ ▸ **Plate Details…** (renamed from Plate Depths…, same dialog).

## Pore area from blue-dyed epoxy (2026-07-31 — Part 2 A1)

`petrography.rs` (Petrophysics ▸ Petrography ▸ **Pore Area…**, `poreAreaDialog.ts`) is the first
measurement taken off a plate, and deliberately the **dimensionless** one: an area fraction needs no
micrometres per pixel, so it runs on every plate rather than only the calibrated ones. The
deliverable is an area fraction per plate, which estimates volume fraction by the Delesse relation.

**A plate must be DECLARED impregnated, and an undeclared one is refused BY NAME.** This is the
whole reason `well_images.prepared` exists (`petrography::epoxy_check`, deliberately split out and
public so a test can pin it without Pillow). A blue rule run over an unimpregnated section does not
fail — it returns a porosity assembled from blue-ish feldspar, stain bleed and edge artefact, and
that number then plots against core helium porosity looking entirely reasonable. Nor can the app
work it out from the pixels: the evidence for "this is blue epoxy" is the blue it was about to
measure, which is the same circle as reading a water zone off the saturation being calibrated. The
plate picker greys out and explains each unqualified plate BEFORE a run rather than after.

**The colour band is the user's, not the app's.** `PoreColorBand::default()` is a plain blue band in
round numbers, offered as the starting point for a VISUAL tuning task, and pinned as generic by
`the_default_colour_band_is_generic_not_a_calibration` (same discipline as `gr_normalize`'s
reference percentiles — a two-decimal threshold would be somebody's regression result).

**The preview comes from the SAME code as the measurement.** Redrawing the mask in the frontend
would put the segmentation in two languages and the two would drift — the standing `composite.rs`
versus log-view-renderer warning. So the Python runner returns the overlay PNG, and what the user
tunes against is literally what gets measured. Tuning re-measures ONE plate (`only_image_id`) and a
stale in-flight answer is dropped by sequence number rather than being allowed to overwrite a newer
one.

**No morphological cleaning.** Opening or closing a mask needs a structuring element measured in
PIXELS, which is a size — and a plate may carry no scale at all, so that size could not be stated in
microns for every plate. Rather than pick a pixel count meaning a different physical distance on
every plate, nothing is smoothed and the speckle stays visible in the preview where it can be
judged. This is the scale gate applied consistently, not an omission.

**Results are POINT DATA, not a curve** — `aux_data`, dataset `PETROGRAPHY`, item `VPORE_TS`, at
each plate's depth. A thin section measures the one plug it was cut from; a line between two of them
would claim rock nobody looked at, the same argument that made point data a track kind rather than a
`CurveStyle`. Its own dataset rather than the image delivery's name, so re-running the measurement
never looks like a second delivery of pictures. **Measuring and saving are separate**: tuning a
threshold means running it many times, and `set_name` is what turns a run into a write.

Rule 7 throughout: numpy + Pillow in ONE subprocess per batch of `CHUNK` (16) plates — a
core-photograph delivery is hundreds of plates at ~1 MB each and one batch would be a gigabyte in
flight. `pore_support()` probes before a run so the dialog can name what is missing. The runner
reads `sys.stdin.buffer`, never `sys.stdin`. The real round-trip test
`a_quarter_blue_plate_measures_a_quarter` is `#[ignore]`d so the green gate never depends on an
optional package; it builds a plate that is exactly a quarter blue plus a pale violet patch that a
hue test alone would count — **it is the saturation floor that rejects that patch**, which is why the
floor exists.

## Calibrating a plate from its own scale bar (2026-07-31)

`src/ui/scaleBarDialog.ts`, reached from the **⇹** button on each row of Plate Details…. The route
that makes a plate measurable when it states its scale as a BAR burned into the image rather than as
a field of view in the caption — which, on Jauhar's "sometimes yes, sometimes not", is a good share
of them. It is the gate everything dimensional sits behind.

**The measurement is a pure RATIO, and that is the whole reason it is safe.** The drag is taken as a
FRACTION of the picture's width, so the field of view is `bar length / that fraction`. Nothing in it
depends on the display zoom, or on the stored copy having been resampled to a long-edge cap — both
lengths shrank by the same factor and the ratio did not move. This is the same property that made a
field of view the right thing to store rather than micrometres per pixel, and it means the answer
comes out already in the form the store wants, with no second conversion to get wrong. Verified in
the browser: the same drag at a displayed width of 848 px and of 400 px both returned 2000 µm.

Endpoints are held as fractions of the natural width/height for the same reason — they survive a
view-mode switch and a window resize without being recomputed.

**A crooked drag costs almost nothing, so there is no snapping.** Off a truly horizontal bar by 5°
the measured length is long by 0.4%, because the error is second-order in the angle. What actually
decides the accuracy is hitting the bar's ENDS, which is what the **Actual size** mode is for — one
pixel of a 100 px bar is 1%, and there is no way to shrink that except to look closer.

**It only FILLS the box.** The row's own Save is still what writes the value, so a calibration is
reviewed like any other typed number rather than being applied by the act of measuring. The optional
"apply to every plate of this delivery" writes row by row rather than through
`set_image_delivery_details`, because each plate must keep its OWN preparation and stain: a scale
must never quietly overwrite what the section was made of. Slower, and the only version that is
right.

`openModal` has no close hook, so the dialog watches `#modal-root` for its content being detached
and resolves `null` — a caller awaiting a calibration must not be left hanging on Esc or ✕.

## Pore geometry (2026-07-31 — Part 2 family C)

`petrography.rs` gained per-pore shape and size, opt-in beside the area fraction in the same
dialog. **One decode, one mask, both answers** — the fraction and the geometry can never describe
different pictures, which two passes would eventually allow.

Outputs per plate: `PORE_N`, `PORE_ASPECT`, `PORE_SHAPE` (all dimensionless, reported for every
plate) and `PORE_D10` / `PORE_D50` / `PORE_D90` in micrometres, **written only where the plate
carries a scale**. Not a NaN in their place — a NaN would still occupy the item and read as a
measurement that failed rather than one that was never possible.

**Four-connectivity for the pore phase.** Two pores meeting at a single corner are joined by a
throat of zero width; that is not one pore body, and 8-connectivity would fuse them.

**The perimeter is a four-direction Crofton estimate, NOT a boundary-pixel count.** A staircase
boundary overestimates a diagonal edge by up to √2, which biases circularity systematically LOW —
systematically, so it never looks like noise. The estimate used is
`P = (π/8)·[(N_h + N_v) + (N_d1 + N_d2)/√2]`, which returns 2πR for a disc. Measured on a synthetic
disc of radius 100: area 31417 against 31416, perimeter 630.1 against 628.3, circularity 0.994. Its
worst case is a perfectly axis-aligned rectangle, where it returns `(π/4)(w+h)(1+√2)` against a true
`2(w+h)` — about 5% low, for any rectangle. Pores are neither circles nor axis-aligned boxes and
circularity is read comparatively, so a few percent of consistent bias does not change which pore is
rounder. Pinned by `the_perimeter_estimator_is_crofton_not_a_boundary_pixel_count` so nobody
"simplifies" it back into a boundary count.

**Aspect ratio comes from second moments, so it carries none of the perimeter's bias.** The `+1/12`
discrete correction is included: a pixel is a unit square rather than a point mass, and without its
own variance in the second moment a small round pore reads as elongated purely from the sampling.
Measured 1.0000 on a disc and 5.0000 on a 40×200 bar.

**A pore cut by the frame is EXCLUDED and counted** (`n_edge`). Its true size is unknown, and
including it biases the size distribution small — the standard stereological edge rule. **A blob
below `min_pore_px` is speckle and is dropped and counted** (`n_small`); that threshold is in PIXELS
on purpose, because it states what the picture can resolve rather than a size in the rock, and it
has to mean the same thing on a plate that carries no scale at all.

**Diameters are AREA-WEIGHTED.** Capillary pressure fills volume, and a count-weighted median on a
digitized section is dominated by the smallest features the scan resolves — which says more about the
scan than about the rock. `weighted_percentile` lives in `petrography.rs` rather than
`distribution.rs` deliberately: that module is source-agnostic on a bare value slice, and a parallel
weight vector is a different contract only this caller needs. Every UNWEIGHTED percentile still goes
through `distribution::percentile`, so a pore percentile and a log percentile are the same operation.

**The runner stays deliberately dumb** (the `office.rs` rule): it returns per-PORE arrays and every
statistic is computed in Rust. Geometry needs **scipy** — only for the connected-component
labelling, which in pure numpy would be a Python-level union-find over millions of pixels — so it is
opt-in and its absence fails only the geometry, never the area fraction. The real round trip
`a_disc_reads_as_round_and_its_diameter_follows_the_declared_scale` is `#[ignore]`d for the same
reason the rest are.

## Refusing a click that needs a well (2026-07-31)

`src/ui/needWell.ts` `requireWell(action)` is the ONE refusal for an action that works on the
selected well, and it opens a named dialog rather than writing to the status bar.

**A status-bar line is the wrong place to refuse a click.** The user picked "Import SCAL…" and
expected a file dialog; what they got was nothing, with the reason in a corner of the window nobody
was looking at. "Nothing happened" is indistinguishable from a broken button, and the usual next
move is to click it again — which does nothing again. It is the same family as every other
refusal in this app (an undeclared stain, an unimpregnated plate, a plug with no partner inside the
depth tolerance): each is refused BY NAME with the fix stated. This was the one place still quiet.

The status line still receives the message, because it belongs in the record of what was attempted;
it simply cannot be the only place it appears. One helper rather than nine copies — the
`followCore.ts` argument: same decision, and nine copies is nine places for the wording to drift.
Callers: Export LAS, Import DLIS, Import SCAL, Import deviation, Import Aux, Import pictures, Data
Sets, Shift Core, Well header.

## Mineral classifier (2026-07-31 — Part 2 family A3)

`petrography.rs` `run_plate_classifier` + `mineralClassDialog.ts` (Petrophysics ▸ Petrography ▸
**Mineral Classifier…**). Quartz against feldspar in plane light is not a colour problem, so this
family is a supervised classifier and never a colour rule — `docs/plan_image_analysis.md` §2.1 A3.

**There is no shipped model and there will not be one.** A model trained on somebody else's
sections, under somebody else's lamp, would produce numbers with the shape of a modal analysis and
none of the content. The training data is this user's clicks on these plates, and the result says
so in its own notes: the lamp, the white balance and the scanner are part of what it learned, so it
is not a model for a differently photographed delivery.

**Clicking IS the method, because it is the workflow that already exists.** Point counting is a
petrographer moving a stage and naming what is under the crosshair. The dialog is that act, and
what it produces is training data rather than a tally.

**The labels are the artefact, not the model.** They persist as a `platelabels` document keyed
`<well_id>/<dataset>`, and the forest is refitted from them — seeded — on every run. A stored model
blob cannot be read, argued with or corrected; a list of clicks can be all three, and the answer
stays reproducible from it. This is deliberately unlike `ml_models`, where the artefact is the
model because the training curves may be gone by the time it is applied.

**Cross-validation groups by CLICK, not by pixel.** A click contributes its immediate neighbourhood
so the fit has some support, but those pixels are near-identical — splitting them across a fold
boundary scores the model on data it has already seen and reports an accuracy nobody can reproduce
on a new plate. Same discipline as blind-well CV in `ml.rs`. Pinned by
`the_classifier_is_cross_validated_by_click_not_by_pixel`.

**Recall is reported PER CLASS and the weak ones are named.** An overall 0.9 sits comfortably on top
of one mineral the model cannot see at all, and that mineral's fraction is then noise wearing a
percentage sign. Below 0.7 the run names it and the dialog colours the row.

**Two refusals before a subprocess is even started.** One class is not a classification — a model
that always says "quartz" is right every time and knows nothing. And a class with fewer than
`MIN_CLICKS_PER_CLASS` (3) clicks cannot have any held out, so its accuracy would be a number about
nothing. Pinned by `the_classifier_refuses_a_training_set_it_could_not_be_checked_on`.

**Features are colour plus TEXTURE**, and the texture is the only reason this can attempt a pair
colour cannot separate: R, G, B, cos/sin of hue, saturation, value, and the local 5×5 mean and
standard deviation of brightness. **Hue enters as its sine and cosine** because it is circular — 359°
and 1° are neighbours, and a raw angle would place them at opposite ends of the feature.

**Measured, not asserted** (`the_classifier_separates_on_texture_and_admits_when_it_cannot`,
`#[ignore]`d, needs scikit-learn). Two halves of a plate with the SAME mean colour differing only in
texture — one smooth, one cloudy: accuracy 1.000, both recalls 1.000, fractions 0.504 / 0.496 against
a true half and half. The CONTROL matters more: label one uniform material as two minerals and
held-out accuracy fell to **0.410** with recalls 0.38 and 0.44, near chance, and the run then names
both classes as unreliable. A classifier that cannot be caught inventing a distinction is worse than
no classifier.

Items are `CLS_<MINERAL>` — **deliberately not `MIN_`**, which the stain rule uses. A fraction a
colour rule produced from a published stain identification and one a classifier produced from this
user's clicks are different claims with different provenance, and one name would leave a report
unable to say which it quoted. Same argument that keeps `GRAIN_D50_APP` apart from `GRAIN_D50_W`.

Label positions are FRACTIONS of the picture, never pixels — the stored copy is resampled to a
long-edge cap, so a pixel coordinate belongs to whichever copy it was taken on and nothing in the
number says which. The scale-bar argument again.

Each plate's fraction is estimated from a systematic sample capped at 400 000 pixels, and the count
is reported rather than being a silent truncation. Needs scipy AND scikit-learn, probed by
`classify_support` so the dialog can name what is missing before a run.

## Stained carbonate (2026-07-31 — Part 2 family A2)

`petrography.rs` reads the stain as well, opt-in beside the pore fraction, the pore geometry and
the grains — same decode, same pore mask, so the mineral fractions and `VPORE_TS` describe ONE
segmentation and sum against each other. Fractions are of the WHOLE plate: **pore + minerals +
unclassified = 1**, verified as exactly 1.000 on a synthetic four-quarter plate.

**A plate is refused unless its OWN declared stain matches the scheme.** Undeclared is refused too,
for the `prepared` reason: it cannot be read off the pixels, because the evidence for "this is
alizarin red" is the red about to be measured. Reading an alizarin-red scheme off a section stained
with something else does not fail — it returns mineral fractions that are wrong and entirely
plausible. Names are compared with punctuation and spacing thrown away (`normalize_stain`), so
"Alizarin Red S" and "alizarin-red-s" are one stain but a different stain is not.

**The identifications are published; the colour bands are not.** `stain_scheme` ships Friedman
(1959) for alizarin red S and Dickson (1966) for the combined alizarin red S + potassium
ferricyanide stain — standard carbonate petrography, already named in
`docs/plan_image_analysis.md` §2.1. What hue a stained calcite *photographs* as depends on the dye
batch, the concentration, the etch, the lamp, the white balance and the scan, so the bands are round
numbers to start a visual tuning from, exactly like the epoxy band, and the class list is editable.
Pinned by `the_stain_schemes_are_published_identifications_with_generic_bands`.

**`StainBand` carries a saturation CEILING, and that is not a decoration.** Dolomite under alizarin
red S is identified by staying COLOURLESS. "Unstained" is the absence of colour and cannot be
written as a floor, which is why this is a different type from `PoreColorBand`.

**Classes are tested IN ORDER, first match wins.** A pixel is one mineral. Overlapping bands are
resolved by the order the user put them in rather than being silently counted twice.

**`MIN_UNCLASS` is written on every run and is the honesty number for the family.** Solid that fell
in no band is reported rather than distributed over the classes; a section where a third of the rock
matched nothing has not been given a mineralogy, whatever the other rows say. Above 25% the run says
so in the notes.

**Blue epoxy and turquoise ferroan dolomite are the same colour, and this is measured, not
theorised.** Under Dickson's stain ferroan dolomite goes turquoise; blue-dyed epoxy is blue. On a
plate that is both impregnated and stained the pore rule claims those pixels first, so the mineral
is counted as porosity. On the synthetic plate, with the default epoxy band (180–260°) the run
returned **pore 0.500 and ferroan dolomite 0.000** — porosity doubled and a mineral erased, both
plausibly. Narrowing the epoxy band to 210–260° returned **pore 0.250 and ferroan dolomite 0.250**,
which is the truth. `epoxy_collides` detects the overlap and NAMES the affected minerals in the
notes; it is never resolved automatically, because which of the two bands to narrow is a judgement
made looking at the plate. Pinned by
`blue_epoxy_and_ferroan_dolomite_are_flagged_as_the_same_colour`, which also checks the check is not
trivially always true.

Items are `MIN_<MINERAL>` (`mineral_item` upper-cases and collapses non-alphanumerics, so "Ferroan
calcite" becomes `MIN_FERROAN_CALCITE`) plus `MIN_UNCLASS`, all in the `PETROGRAPHY` dataset at the
plate depth. Dimensionless throughout, so unlike the grain sizes they run on every plate including
the uncalibrated ones.

`hsv_of` and `in_band` are now the ONE colour conversion in the runner, shared by the pore rule and
every stain class — the same argument that made `shape_stats` shared between the pore and grain
phases.

## Grain size (2026-07-31 — Part 2 family B, D3 closed)

`petrography.rs` gained the grain phase, opt-in beside the pore fraction and the pore geometry in
the same dialog. **One decode, one mask, three answers** — the grain phase is defined as whatever
the pore rule did not claim, so the porosity and the grains describe ONE segmentation. That is also
why grains inherit the blue-epoxy refusal: a plate where pore cannot be told from solid cannot have
its grains outlined either.

Outputs per plate: `GRAIN_N`, `GRAIN_ASPECT` and `GRAIN_CONTACT` (dimensionless, every plate), plus
`GRAIN_D10_APP` / `GRAIN_D50_APP` / `GRAIN_D90_APP` in micrometres and `GRAIN_SORT_APP` in phi where
the plate carries a scale, and the four `_W` twins when the Wicksell correction was asked for.

**D3's answer — "apply wicksell correction is optional" (Jauhar, 2026-07-31) — is implemented as
different ITEM NAMES, not one name and a flag.** There is deliberately no bare `GRAIN_D50`: a name
that sometimes means the section value and sometimes the corrected one cannot be read by anything
downstream, and a report quoting it has no way to say which it got. Pinned by
`apparent_and_corrected_grain_sizes_are_stored_under_different_names`, which matches on the `put(`
call site rather than the bare string — a test that scans its own source must not trip over the
name it is looking for.

**The split is a nearest-centre partition of the solid phase, NOT `scipy.ndimage.watershed_ift`.**
That was tried first and measured: on a welded pair that should split evenly it gave one grain
47792 pixels and the other 9, because its tie-breaking across the quantized cost plateaus lets
whichever marker is reached first take almost everything. The nearest-centre partition splits the
same pair 23957 / 23844, returns 16 of 16 discs in a loose pack at 7845 px against a true 7854, and
keeps a single disc as ONE grain at every separation setting. (scikit-image's watershed would work
too, at the price of a whole new dependency for one function.)

**The search is confined to one connected blob of solid at a time, and that is load-bearing.**
Without it a pixel can be nearer a centre across open pore than its own, and the two disconnected
pieces would then carry one label — one grain in two places, with an area and a shape belonging to
neither. Solid is labelled EIGHT-connected, the complement of the pore phase's four: two grains
meeting at a corner are one piece of rock even though the pores either side of them are not one
pore.

**`GRAIN_CONTACT` is the honesty number and it rides with every grain run, never optionally.**
Where grains are welded by cement or an overgrowth there is nothing in the picture to separate
them, and the algorithm places a boundary at the neck anyway — a geometric artefact, not a grain
contact. The stored value is the median fraction of a grain's outline that is a grain-to-grain
contact rather than open pore; above 0.7 the run says so in the notes and tells the reader to treat
those sizes as a rock-fabric description rather than a grain-size analysis. It is deliberately a
ratio of two counts gathered the same way rather than two Crofton perimeters: the staircase bias
affects both alike and cancels, and this is a quality indicator, not a length.

**Sorting is Folk & Ward (1957) inclusive graphic standard deviation**, `σ_I = (φ84 − φ16)/4 +
(φ95 − φ5)/6` with `φ = −log2(d in mm)`. Chosen over a plain standard deviation because it is what
maps onto the verbal scale a core description already uses. Phi RISES as grains get finer, and a
sign slip there would flip every sorting number in a deliverable while leaving it looking entirely
reasonable — hence `phi_rises_as_grains_get_finer`. Phi is a logarithm of millimetres, so sorting
needs a scale exactly as much as a diameter does.

**Everything is AREA-weighted, and on a section that IS volume weighting.** The chance of a random
plane meeting a grain scales with its diameter and the mean cut area with its square, so the
section area attributable to a size class goes as `n·D³` — which is what a sieve weighs. That is
what makes apparent and corrected comparable to each other and either of them comparable to a
sieve, and it is the same weighting the pore diameters already use, so there is one rule in the
module rather than two.

**The Wicksell unfolding is Saltykov's, DERIVED rather than transcribed.** The published
coefficient table is a set of numbers that can be mis-copied and would then be wrong silently. They
come instead from the chord geometry — a plane at distance `h` from a sphere's centre cuts a circle
of diameter `√(d² − 4h²)`, so `F(x) = 1 − √(d² − x²)/d`, and a random plane meets a sphere at a rate
proportional to its diameter. Twelve logarithmic classes, and class 0 reaches down to ZERO rather
than stopping a decade below the maximum: the published version drops that tail, and losing real
sections to a class boundary would be a silent subset. Negative unfolded populations are clamped
and COUNTED (`w_clamped`) — the inversion is ill-conditioned by nature, and a clamped class is the
signal that this plate's correction is unstable.

**The representative diameter of a class is its UPPER bound, because that is the diameter the
unfolding solved for.** Reporting the class midpoint instead would quote a population the
arithmetic never solved, and on a single-size population it comes back ~11% fine purely from where
the bin edges fell. Its cost is that every class is quoted at its coarse edge.

**What the correction actually buys, measured rather than assumed.** A population of identical
spheres is perfectly sorted; its sections are not, and that spread is the dominant Wicksell effect.
It is on SORTING, not on the median — the apparent median of a monodisperse population is only
about 13% low (the median chord of a sphere is √3/2 of its diameter) and area weighting pulls even
that most of the way back, because it up-weights exactly the near-central cuts. Measured here:
apparent sorting on a perfectly sorted population is 0.19 phi area-weighted, which on the Folk &
Ward verbal scale is still inside "very well sorted"; count-weighted it is worse. So the weighting
choice moves this number more than the correction does, and a user reaching for Wicksell hoping to
move D50 is reaching for it for the wrong reason. Pinned by
`the_correction_earns_its_place_on_sorting_not_on_the_median`, and the unfolding itself by
`a_single_sphere_size_unfolds_back_to_one_class`, which recovers the true diameter exactly.

Two pixel knobs, `min_grain_px` (50) and `grain_sep_px` (20), both ROUND and both stated in PIXELS
for the `min_pore_px` reason: they say what the picture can resolve, not a size in the rock, and
they must mean the same thing on a plate carrying no scale. Over-segmentation is what a
distance-based split gets wrong when it gets anything wrong, so the preview draws the grain
outlines in yellow over the same mask — judged by eye, not from the table. Pinned as generic by
`the_grain_defaults_are_generic_not_a_calibration`.

Geometry needs **scipy**, so grains are opt-in and their absence never touches the area fraction.
The real round trip `welded_grains_still_split_but_say_that_the_boundary_was_inferred` is
`#[ignore]`d for the usual reason.

UI note: the Wicksell label is hidden with `style.display`, NOT the `hidden` attribute. It carries
an inline `display: block`, and a display rule beats `hidden` every time — setting the attribute
left the row fully visible at 19px tall. Same family as the ribbon panels and menus; caught in the
browser, not by the compiler.

## Plug QC — checking a measurement against an independent one (2026-07-31)

`plugqc.rs` + `plugQcPanel.ts` (Petrophysics ▸ Petrography ▸ **Plug QC…**, also in the workspace
＋ menu) plot two measurements made on the SAME plug against each other. The petrography numbers
were the first measurements this app produced that nothing else in it could check: an area fraction
estimating a volume fraction by the Delesse relation is a *claim*, and the only test of it is the
helium porosity of the plug the section was cut from.

Sources are the three plug-scale stores — a routine-core column (CPOR/CPERM/CGD/CSW), any numeric
item of any point dataset (which is where every petrography output lands), and a **pore-throat
radius read off the plug's own capillary-pressure curve**. All three read through the active-set
fragments like every other reader.

**A pair is two measurements of the same plug, and a sample with no partner inside the tolerance is
DROPPED and COUNTED — never snapped.** Same rule as the S-factor calibration and the same reason: a
core that is off by a whole sample interval is invisible to any tolerance check, so widening the
tolerance to win more points quietly pairs a plug with its neighbour. `registration.rs` is the fix,
and the empty-result note points there rather than suggesting a wider tolerance.

**A measurement is used ONCE.** Pairing is greedy on the closest pair first and consumes both
sides. Two sections cut a centimetre apart would otherwise both claim the one plug nearest them,
and that single core porosity would appear twice in the cloud and twice in the correlation,
tightening it for free. Pinned by `one_plug_cannot_be_claimed_by_two_sections`.

**Both a linear and a rank correlation are reported, because they answer different questions.**
Pearson asks "is this a straight line", which is right when the axes are the same quantity measured
twice. Spearman asks only "do they move together", which is the only sensible question for pore
BODIES against pore THROATS — different lengths that must never fall on one line, though a rock
with bigger bodies had better have bigger throats. Spearman is also invariant to any monotone
transform, so it does not move when the pane switches an axis to log, which keeps the number from
disagreeing with the picture beside it. Pinned by
`a_curved_but_monotone_relation_reads_as_rank_agreement_not_a_straight_line`. Both inherit
`tops::pearson`'s four-point floor, and a blank is EXPLAINED in the notes rather than left as an
empty cell that reads as a bug.

**Nothing here converts a unit** — point data is stored verbatim — so the result reports the MEDIAN
of each axis. A 0.19 beside an 18.2 is a percent-versus-fraction delivery the user can see at a
glance, which beats a guess about which one was meant.

**The throat radius is Washburn with the laboratory's OWN σcosθ**, taken from `scal_pc.ift` as
recorded. A plug with no recorded interfacial tension has a pressure but no radius and is excluded
BY NAME — `thomeer.rs` takes the same line for the same reason. Pc is interpolated in **log Pc**:
one curve spans decades, so interpolating linearly between a 10 psi and a 1000 psi step lands an
order of magnitude out. A saturation outside the measured range is **never extrapolated** — a curve
that stopped at 20% mercury cannot state r35, and a radius invented past the last step would be the
strongest-looking number on the plot. The default is **35% mercury**, the Kolodzie (1980) / Winland
r35 convention already used by `rocktyping.rs`, which is what makes this plot directly comparable
to the R35 curve that module predicts from φ and k. `resolved_saturation` is the ONE place the
default is applied, so a caption can never disagree with the number it labels.

`fitScatter.ts` gained the two things this needed and the calibration dialogs did not: a
`{kind: "none"}` reference line and optional log axes. **A comparison of two DIFFERENT quantities
gets no line and independent axes** — a 1:1 line between a pore diameter and a throat radius
asserts an equality nobody claims, and every point sitting below it would read as a disagreement
when it is the physics. The line is SAMPLED across the window rather than drawn end to end, because
`y = slope·x` is not a straight line in log space. A value at or below zero is SKIPPED on a decade
axis, never floored to the smallest positive one. `.form-row[hidden]` was added to `styles.css` for
the mercury-saturation row — a `display` rule beats the `hidden` attribute, the gotcha the ribbon
panels hit twice.

Statistics are computed on EVERY pair before the cloud is decimated to `MAX_POINTS` for the wire,
and the decimation says so in a note; the display points are spread evenly, never the first N.
Changing the reference line or an axis scale redraws from the pairs already in hand — those are
display choices, and re-pairing would be the same answer arrived at more slowly.

## The first real delivery (2026-07-31 — what running it on real rock changed)

Six increments of measurement had been built on top of each other without any of them meeting real
rock. Running the pore rule over a real carbonate petrography delivery — 134 photomicrographs, one
laboratory, one well, one report — changed the design. Three findings, in the order they bite.

**A petrography delivery does not arrive as a folder of pictures.** It arrives as an Excel workbook
with one WORKSHEET per plate: the well, the depth in feet, the plug number and the magnification
typed into cells, and the photomicrographs anchored on top as embedded objects. `images.rs` takes a
list of files and can read none of it. Every plate in this delivery had to be lifted out of the
workbook before anything in this app could see it. That is the actual first barrier between the
petrography suite and a client's rock, and nothing in the suite addresses it yet — a plate importer
that reads a workbook is the missing increment, not another measurement.

**The delivery states a magnification, not a field of view.** Cells read `5x` and `10x`. Turning
that into micrometres needs the camera sensor size and the tube factor, neither of which the
delivery states, so `fov_um` cannot be filled from it. Some plates carry a scale BAR as a separate
embedded graphic beside the picture — a yellow rule captioned `1 mm` — which is what
`scaleBarDialog.ts` exists for, but only once the bar and the plate are in the same picture.
Everything dimensional stays refused on this delivery, which is the designed behaviour and, on real
data, the common case rather than a corner.

**And the finding that changed the code: `epoxy_check` was only half the guard.** It refuses the
plate nobody impregnated. It says nothing about a plate that WAS impregnated but photographed under
a light the colour band was never tuned for — and there the rule swallows the matrix and returns a
porosity anyway. Across these 134 plates the median hue of the picture ran from **26 to 310
degrees**: one blue-cast plate sat at 221 and read **0.97 v/v**, a green-cast plate from the same
core at 149 read 0.06. Twenty-eight plates measured above half the section as pore. Not one of them
would have failed; all of them would have been stored at a real depth and gone on to plot against
core helium porosity.

`petrography::scene_dominated` is the guard. **The test is the plate's OWN median hue, not a cap on
the answer.** A cap would be arbitrary — one field of view crossing a large vug genuinely can be
mostly pore — but rock is mostly rock, so on a plate the band is reading correctly the TYPICAL
pixel is a grain and its hue falls OUTSIDE the pore band. When the median pixel is pore-coloured,
the band has stopped discriminating and is describing the scene. On this delivery that flagged
every one of the 28 plates reading above 0.5 v/v, and the highest an unflagged plate reached was
0.387 — a plausible carbonate. What would be stored went from a 0.000–0.972 range with a 0.231
median to 0.000–0.387 with a 0.115 median.

**The fraction is still measured, shown, and previewed; what is refused is the WRITE.** Tuning the
band is exactly how a user fixes this and they cannot tune against a number they are not shown, so
the plate appears in the table in `var(--warn)` with the reason on hover. Nothing off that plate is
stored — not the fraction, not the pore shapes, not the minerals — because they all come off the
same mask, and if the mask is the background then every number derived from it is about the
background. The run also reports the delivery's hue SPREAD when it exceeds 60 degrees, because that
is what decides whether one band can serve the whole delivery: here it could not, and the honest
instruction is to measure the plates in groups. Pinned by
`a_plate_whose_own_median_hue_is_pore_coloured_is_not_measured`,
`the_scene_check_reads_a_wrapped_band_the_way_the_runner_does` (the guard must read a band written
across 0 degrees as two arcs, exactly as the runner's `in_band` does, or it would silently disable
itself for anyone using one) and the round trip `a_blue_cast_plate_is_shown_but_never_stored`.

**A synthetic fixture the guard rejected was the fixture's fault, and fixing it mattered.**
`welded_grains_still_split_but_say_that_the_boundary_was_inferred` drew small discs floating in
epoxy — 87% pore, which is a mount rather than a rock. It now draws grain-dominated plates. A
fixture that could not exist is a fixture that cannot catch the bug the real delivery found.

Still open, found and not yet fixed: a delivery can mix photomicrographs with SEM plates and scale
graphics in one folder, and a colour rule run over a greyscale SEM image returns **0.000** — a
plausible-looking number for a tight rock, and the mirror of the 0.97 case. The obvious test
(saturation) did not separate them on this data, so nothing was shipped rather than a guessed
threshold.

## Provenance discipline (2026-07-31)

The repo is intended to be **licensed**, and its author runs consulting studies under
confidentiality agreements. Three rules follow, and they are enforced by tests, not goodwill.

**No client identifier in the tree.** No operator, block, field, project number, study name,
well ID or delivery path in source, tests, test data or comments. The real deliveries the
`#[ignore]`d integration tests need are found through **`SANDIBUMI_FIELD_FIXTURES`**
(`field_fixtures.rs`: `<root>/las/*.las`, `<root>/core/*.csv`) — the tests take whatever the
folder holds and **skip with a printed reason** when it is unset, so a fresh clone stays green.
Test wells are `SANDI-*`, matching `dataset for test/examples/`. Delivery *shapes* are still
documented (that is why the parser rules exist) — describe the shape, never name the delivery.

**No client-fitted number ships as a default.** A regional calibration is somebody's analytical
work product AND is silently wrong in another basin — normalized GR always looks plausible, so
the user gets no warning. `gr_normalize` is pinned by
`gr_normalize_reference_defaults_are_generic_not_a_field_calibration`, which also rejects a
non-integer reference on the grounds that a two-decimal endpoint is a regression result.
Real calibrations live outside the repo as local presets.

**Never strip an attribution while its asset still ships.** The study citation in `lrlc.rs`, the
tooltips naming which vendor tables seeded a default, the comments recording why a parser rule
exists — all are the record, and deleting one while shipping what it describes is concealment,
not a fix. Attribution comes out only when the asset comes out. The register is
`docs/IP_PROVENANCE.md` (§2.7 is the client tier); the sweep that produced it is
`docs/provenance_sweep_prompt.md`; findings with `file:line` and the lawyer packet are in the
gitignored `docs/commercial/`. **`THIRD-PARTY-LICENSES.md` is generated** — re-run
`node tools/gen-third-party-licenses.mjs` after any dependency change, never hand-edit.

## Text-import encoding (2026-07-30)

**Every** text import goes through `parsers::read_text_file` — never `read_to_string` or
`BufReader<File>`, both of which reject a whole file on one stray byte. `decode_text` honours a
BOM first (UTF-8, UTF-16LE/BE — Excel's "Unicode text" export is UTF-16, and decoding it as
cp1252 would silently yield NUL-riddled nonsense instead of an error), then tries UTF-8, then
falls back to **cp1252**, which cannot fail. Found by a real Duri core table: 330 KB of pure
ASCII except two `0x95` bullets opening a lithology description, refused with "stream did not
contain valid UTF-8" — 3,045 plugs lost to two characters in a comment field. Bytes are
interpreted, never rejected; the worst case is a mangled character in a description, not a lost
delivery. Tests: `parsers::encoding_tests` (plus `probe_real_duri_core`, `#[ignore]`d, which
runs against the real file on this machine).

## Startup: the window exists before the database does (2026-07-30)

`run()` builds the Tauri app on an **empty in-memory placeholder** connection; `setup()` spawns
`open_startup_project` on a background thread (the recovery ladder — project → temp recovery file
→ memory-only — lives there now). It publishes an `OpenOutcome` into `DbInit`
(`Mutex<Option<_>>` + `Condvar`); the async `await_project_open` command waits on it via
`spawn_blocking`. **The outcome is STORED, not just signalled** — a fast open publishes before the
frontend asks, so a pure signal hangs every quick launch (test:
`fast_open_published_before_the_wait`). **`main.ts` must await that gate before building any panel
or issuing any other command** — until it resolves the live connection is the placeholder and
every query truthfully returns nothing; that ordering IS the contract, there is no per-command
"not ready" check. `src/bootOverlay.ts` covers the wait (400 ms delay so fast opens don't flash,
elapsed clock, polls `boot_report`, hint after 20 s) and hands its drained notes back to `main.ts`
to record once the history's database is open. Long-running commands are `async` +
`tauri::async_runtime::spawn_blocking`; a **sync** `#[tauri::command]` runs on the main event-loop
thread and freezes the window (and cannot call `spawn_blocking` at all — not a Tokio worker; use
`std::thread`, as `run_workflow_chain` does).

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
   fails, set `SANDIBUMI_PYTHON` to the interpreter path.
4. Try plain `npm run tauri dev` first — the vcvars 14.29 pin below is a
   reference-machine-specific workaround, only needed if the default MSVC toolset is
   broken.
5. `tools/chartdig` (chart digitizer) needs `npm i pdfjs-dist@4.10.38` **in that folder**
   and the chartbook PDF (`chartbook.pdf`, Schlumberger Log Interpretation Charts 2013 —
   copyrighted, NOT in the repo; point the `CHARTBOOK_PDF` environment variable at
   your own copy). Only needed to digitize NEW charts — the
   extracted data is already committed in `src/ui/chartOverlays.ts`.
6. Codex auto-memory is machine-local — everything durable lives in this file,
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
```

Verify every change: `npx tsc --noEmit` + `cargo check` + a browser functional test.

Two hard runtime rules (both learned the painful way):
- **Never force-kill `npm run tauri dev`** (task-kill, shell timeout) — an unclean kill
  mid-write corrupts the project DuckDB WAL (see "DuckDB WAL resilience" below).
- After browser verification against the vite dev server, **stop the server so port 1420
  is free** for the user's own `npm run tauri dev`.

## Delegating work to subagents

(Tool-specific form of this rule lives in `CLAUDE.md` — keep the principle in sync.)

Split by **task shape, not task size**. The cost driver in this repo is the verify loop
(`cargo check` through vcvars, ~minutes), not tokens: a cheap-model edit that fails to
compile twice costs more wall-clock than one correct expensive-model pass.

**The rule: cheap model + cheap verification = good. Cheap model + expensive verification
= bad. Never delegate to a cheaper model when a wrong answer would be SILENT** — a number
that is wrong but compiles ships into a client report, and no `cargo check` catches it.

- **Cheapest tier** — read-only retrieval, inventory and grep sweeps. Verification is free.
- **Mid tier** — mechanical edits behind a compiler gate: renames, Tauri command wrappers,
  docs, test scaffolding, TS/dockview plumbing, i18n entries.
- **Strongest tier (default)** — anything numeric or convention-bound: `equations.rs`,
  `multimin.rs`/`multimin2.rs`, `ssc.rs`, `lrlc.rs`, `satheight.rs`, `thomeer.rs`,
  `hfu.rs`, `montecarlo.rs`, chart overlays, the theme var contract, dockview layout.

Reduce reasoning effort before dropping a tier on domain work — lower effort keeps the
petrophysics judgment, a tier drop discards it. A delegated edit is not done until
`npx tsc --noEmit` + `cargo check` pass; never report a subagent's result as verified on
the subagent's own say-so. **Physics defaults** (must trace to `docs/` or a cited source)
and **anything touching the DuckDB write discipline** stay with the main agent regardless
of size.

## Collaboration protocol (Jauhar ↔ Codex)

Jauhar is a petrophysicist (Mahakam Delta, Indonesia) and a beginner programmer — explain
in petrophysics terms, not programming jargon. The working rhythm, on every machine:

1. Work the backlog (`ROADMAP.md`, currently §4b audit items + queued increments) in
   **increments**. Each increment: implement → verify (tsc + cargo test + browser) →
   add a `REVIEW.md` checklist entry → commit (and push once a remote exists) → send a
   completion report that leads with outcomes and proposes the next increment.
2. Jauhar replies **"go ahead"** to accept the proposal; anything else redirects.
3. He field-verifies against real well data via `REVIEW.md` (`[o]` OK / `[x]` wrong /
   `[ ]` untested) — check for new `[x]` marks at session start.
4. **Git/GitHub**: the repo is private; credentials are Jauhar's own. Codex NEVER runs
   `gh auth login` or handles tokens/passwords — he authenticates himself, then Codex
   may create repos/push using his session. Commit messages: plain descriptive, avoid
   embedded double quotes (PowerShell 5.1 quoting).
5. Physics defaults come from documented sources (the reference suite `.info` exports, his studies,
   the chartbook) — cite the source in a comment; when a method spec conflicts with
   code, the specs in `docs/` win.

## Project layout

- `src-tauri/` — Rust backend: DuckDB access, parsers, IPC commands, petrophysics engine.
- `src/` — TypeScript frontend: WebGPU log canvas renderer, Tauri IPC calls.
- `src-tauri/icons/` — app icon set + brand assets: `logo.png` (master), `logo-mark.svg`/`logo-mark.png` (square monogram), `logo-full.svg`/`logo-full.png` (full lockup). Frontend favicon/ribbon assets in `public/`.
- `docs/` — method math + solver specs (SSC/SSPW, LRLC RtC/IMTS, workflow standards, the reference suite/IP multimin extraction). Portable knowledge lives here, not in machine-local memory. **`docs/plan_image_analysis.md` (2026-07-31) is the phase plan for core depth registration + plate digitizing** (ROADMAP C2 item 8) — read it before touching `images.rs`, `shift_core_depths` or anything that pairs a core sample with a log depth.
- `tools/chartdig/` — chartbook vector digitizer (generates `src/ui/chartOverlays.ts`).
- `Prompt/` — original phase-by-phase spec (`Claude_Implementation_Guide.pdf`). Listed in `.gitignore`, but the PDF was **committed before that rule was added and is still tracked** — a gitignore entry never untracks a file that is already in. It DOES exist on a fresh clone. Untracking it is an open decision (provenance sweep 2026-07-31, finding 3).

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.

Planning artifacts live in .castforge/ (plan.md, research.md, decisions.md, ui-spec.md, verification.md); peer work-logs live in .castforge/roles/; per-phase records (plan slice, completion summary, verification verdict) live in .castforge/phases/<phase>/; investigation notes live in .castforge/debug/. Read them before starting work.
