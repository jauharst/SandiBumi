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
10a. **Import sets (2026-07-30)**: one delivery = one named SET in the generic store. LAS/DLIS
    import take a set name (`ingest::LasImportOptions`, `canonical_set_name`/`resolve_set_name`);
    a name already used on a well is auto-suffixed (`FPROOH`→`FPROOH_1`) — **an import never
    overwrites an existing set**. With `attach`, a file whose well name matches exactly ONE
    existing well writes ONLY the generic store on that record (never `standard_curves`, never
    a second well row); >1 match is ambiguous → separate record + warning. **Curve resolution:
    set RAW has ABSOLUTE priority in `equations::fetch_generic_curve_aligned` — do not
    reorder that; other sets are consulted only for mnemonics RAW does not carry.** Browse via
    the Wells-pane ▸ twisty (`objectTree.ts`, lazy per well).
11. **Module inputs are generic-store aware**: `equations::fetch_curve_frame` resolves any non-standard, non-computed curve name from `curve_meta`/`curve_samples` (set RAW) by mnemonic-then-family, so modules/equations can take PEF/CALI/DRHO/extra runs — not just the fixed six. (Log-view rendering `get_track_data` still reads only `standard_curves`.) Runs can pass `opts["MASK"]="<flag curve>"` (e.g. BADHOLE) to NaN-out flagged samples in every output.

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
- `docs/` — method math + solver specs (SSC/SSPW, LRLC RtC/IMTS, workflow standards, the reference suite/IP multimin extraction), plus five reusable prompts, boundaries kept sharp (the table in `stewardship_prompt.md` is authoritative): `maintenance_scaling_prompt.md` (one increment — expand / debug / maintain), `engineering_review_prompt.md` (whole-app behaviour sweeps F1–F5), `qc_audit_prompt_template.md` (one tool end-to-end), `stewardship_prompt.md` (whole-repo structure + onboarding), `product_definition_prompt.md` (what the product IS — PRD, target architecture, v1.0 gate; licensed-product posture). Portable knowledge lives here, not in machine-local memory. Separate family, not in that table: the one-shot vendor-intelligence prompts (`sandibumi_maturation_prompt.md`, `techlog_ingest_prompt.md`, `sonar_ingest_adopt_prompt.md`). **`docs/FUTURE_PLAN.md` (2026-07-31) is the cross-product strategic layer above `ROADMAP.md`** — competitive scan vs Geolog/Techlog/IP, the three positioning axes, credibility floor, OSDU, and the tier sequencing across SandiBumi *and* SegaraBumi (`D:\XX. SegaraBumi`, P6 gate closed, its own PRD/ARCHITECTURE/SEGARA-CONTRACT).
- `tools/chartdig/` — chartbook vector digitizer (generates `src/ui/chartOverlays.ts`).
- `Prompt/` — original phase-by-phase spec (`Claude_Implementation_Guide.pdf`). **Gitignored** — local-only, won't exist on a fresh clone.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
