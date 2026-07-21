# Review checklist — for Jauhar's click-through in `npm run tauri dev`

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time, marking items as you go.
Marks: **`[x]` = confirmed done** (works as described); `[ ]` = not yet checked. If something is
**wrong**, tell me directly (like your 540-well notes) and I'll fix it and log it in
**ROADMAP.md §4 (Field-review backlog)**.

## 540-well test — perf & crash fixes (2026-07-21)

From your ~540-well stress test. A read-only 5-agent diagnosis traced every "not responding"
freeze to one root cause (heavy commands run **synchronously on the UI thread**) plus a specific
speed bug per subsystem. **Rust 176 pass / 0 fail; tsc EXIT 0. Nothing committed.** The async
piece can't be verified without running the app, so these especially want your click-through.

- [ ] **Field Dashboard no longer crashes on ~540 wells** (dashboardPanel.ts, summaryDialog.ts).
      "Compute failed: TypeError: Cannot read properties of null (reading 'toFixed')" was a zero-net
      zone row whose avg VSH/PHIE/SWE come back as NaN → serde encodes non-finite floats as JSON
      **null**, and the old `Number.isNaN` guard doesn't catch null. The formatter now shows "—" for
      null/NaN. Test: **Field Dashboard ▸ Compute** across all wells → the grid renders (empty
      aggregates show "—"), no crash. Same latent fix in the single-well **Cutoffs & Summary** table.
- [ ] **Field Dashboard is fast now** (workflow.rs `stats_only`). Compute took >5 min because it
      secretly wrote 3 FLAG_* curves per well (~1,600 DB transactions) on every press, though the
      panel only reads the returned numbers. It now computes the stats without persisting anything.
      Test: Compute on all wells → **seconds, not minutes**; tweak a cutoff and re-Compute → still
      fast. Behavior change to note: the dashboard no longer leaves FLAG_* curves in the wells —
      persist those from **Cutoffs & Pay Summary** (unchanged) when you actually want them written.
      Test `pay_summary_stats_only_persists_nothing`.
- [ ] **Workflow chain runs without freezing the app + live progress works** (lib.rs — `DbState` is
      now `Arc<Mutex<Connection>>`; `run_workflow_chain` now runs on a background thread). Build a
      chain (e.g. vsh_gr → phi_dn → sw_indo) and run it on a batch of wells: (a) the window stays
      **draggable/responsive** during the run (was frozen "not responding"), (b) the **progress bar
      advances** step-by-step, and (c) **Cancel** actually stops it. This is the first of the async
      conversions — import, dashboard, multimin, Monte Carlo and equations will follow the *same*
      pattern, so confirming this one works validates the whole approach.
- [ ] **A chain of many wells now finishes in seconds, and Cancel is near-instant**
      (workflow.rs two-phase batched write; equations.rs `create_log_sets_batch` +
      `write_computed_curves_versioned_batch`; chain.rs). The 30-min chain / 30-min-to-cancel
      was ~2 fsync-bound DB transactions **per well** per step (≈1,000 commits on 500 wells). Each
      step now computes every well in parallel (reads only), then does **one** batched versioned
      write — ~2 commits per step. Cancel is checked per well, so it drains in a well or two.
      Test: run a chain (vsh_gr → phi_dn → sw_indo) on a big well set → **seconds**, and **Cancel
      stops almost immediately**. Test `batched_module_run_writes_every_well_correctly` proves two
      wells write distinct, un-crossed, correctly-versioned results.
- [ ] **Cancel empties the progress bar** (workflowDialog.ts). Pressing **Cancel** now clears the
      bar to empty (and hides it) the instant you click, then the status confirms "Cancelled at
      step N". Test: start a chain, hit Cancel → the bar goes empty right away.
- [ ] **Input/Output are now "cons" (constellation) pickers, not free text** (workflowDialog.ts,
      moduleDialog.ts, new `list_log_set_names` command). Terminology changed from "set" to **cons**
      throughout the UI (Workflow, module dialogs, Curve Catalog "Constellations"). **Input cons** is
      a strict dropdown of existing constellations (blank = latest values — you can only read from
      one that exists). **Output cons** is an editable combobox: pick an existing constellation *or*
      type a brand-new name. Both are filled from the project's real constellation names. Test: open
      Workflow / any module → Input cons lists your existing constellations; Output cons suggests
      them but also accepts a new name like `FINAL2`.
- [ ] **Universal Processing panel — live per-well progress + Cancel for the whole run**
      (new `jobs.rs` registry + `list_jobs`/`cancel_job`; `processingPanel.ts`; ribbon **Processing**
      button). New dock panel that shows, for a running workflow chain: a **progress bar with an
      integrated Cancel**, the current **"Step 2/3: sw_indo"** line, a live **counts row**
      (▶ running · ✓ done · ⚠ warned · ✗ failed · ⏳ pending), and a **details** toggle listing the
      *notable* wells (running/warned/failed) with messages — so you can see **which well failed and
      why** at 500-well scale without a 500-row dump. It **auto-opens** when you press Run in the
      Workflow Builder, or open it anytime from the **Processing** ribbon button. Cancel here shares
      the *same* flag as the run, so it stops the chain whether launched from the panel or the
      builder. This is the reusable spine: import, module runs, multimin, Monte Carlo and reports
      will each report into it as they move off the IPC thread. Test: Run a chain → the Processing
      panel opens and fills live; click a well's ⚠/✗ in details to read the message; hit Cancel →
      the bar stops within a well or two.
- [ ] **Processing panel: the step-boundary "pause" now says what it's doing** (workflow.rs).
      Each chain step computes every well (bar fills), then does ONE big batched DB write with no
      per-well signal — so the bar used to sit at the boundary / 100% looking frozen. It now shows
      **"Writing N well(s)…"** during that write, so the wait reads as working, not stuck. Test: run
      a chain and watch between steps and at the end → the current line reads "Writing … well(s)"
      during the pause, then advances/completes.
- [ ] **Workflow Builder no longer shows its own redundant progress bar** (workflowDialog.ts). The
      inline `<progress>` bar is gone now that the Processing panel owns the live bar + Cancel; the
      builder keeps a one-line status ("Step 2/2: … — see Processing panel", "Done: …"). Test: run a
      chain → progress shows only in the Processing panel; the builder just shows a status line.
- [ ] **Hardware Health Monitor** (new `health.rs` + `health_snapshot` command; `healthPanel.ts`;
      ribbon **Health** button). A Petrel-PHM-style panel of four colour-coded gauges — **MEM System**
      (system memory %), **GPU Memory** (GPU video-memory current/budget %), **USER Objects** and
      **GDI Objects** (this process's handle counts vs the 10,000-per-process ceiling — the classic
      desktop-leak signal, raw count shown in the value). **Green < 60% · Yellow 60–80% · Red > 80%**,
      polled every 1.5 s. Open from the **Health** ribbon button (next to Processing). Metrics are
      Windows-only; any unavailable value shows **n/a** (so if GPU Memory reads "n/a" on your machine,
      tell me — the DXGI path is best-effort and I'll adjust). Note: this is GPU *memory* load, not
      engine-utilisation % (that needs PDH GPU counters — a possible refinement). Test: open Health →
      MEM/USER/GDI show live %; leave a few heavy panels open and watch GDI/USER climb.

## P0 senior-audit backlog — correctness & data-integrity fixes (2026-07-20)

The eight P0 findings from `AUDIT-2026-07-20.md` (ROADMAP §4b), plus the LAS-import
robustness residual (#118) that closing them surfaced, are implemented and unit-tested
(full lib suite **160 pass / 0 fail**, tsc clean). These are the ones that made answers
wrong or silently lost data, so they matter most for Mahakam work:

- [x] **MASK now blanks module INPUTS, not just outputs** (workflow.rs). Run **GR
      Normalize** (or **Log Predict**) with a BADHOLE / COND*FLAG curve set as the
      **Mask**. The well P3/P97 (and the KNN training set) are now computed from the
      unmasked samples only — casing/washout/hot-streak GR no longer shifts the two-point
      transform, so good-hole output stops drifting. For log_predict the repaired synthetic
      now survives \_inside* the masked (washout) interval it exists to fill, instead of
      being blanked there. Test: `mask_excludes_flagged_samples_from_gr_normalize_percentiles`.
- [x] **SW-height uses TVD and allows a subsea FWL** (satheight.rs). The sw_height module
      now takes an optional **TVD** input (defaults to measured depth when absent) and the
      **FWL** field accepts negative (subsea TVDSS) values. On a deviated well, height above
      the contact — and therefore SWH — is no longer optimistically overstated by ~1/cos(inc).
      Run on a deviated Mahakam well with the TVD curve mapped and confirm SWH rises vs the
      old MD-based result. Test: the negative-TVDSS deviated case in satheight.rs.
- [x] **Pay summary: thin-zone clamp + honest averages** (workflow.rs). Each sample's
      thickness is clamped to its overlap with the zone, so the last in-zone sample no longer
      bleeds a full step past the zone base and **net can never exceed gross** (sub-step-thick
      zones). SAND-row `avg_phie` is normalised over the thickness where PHIE is actually
      valid, so a sample with good VSH but missing PHIE no longer drags the average toward
      zero. Cross-check a well with thin zones / patchy PHIE against the old numbers. Test:
      `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`.
- [x] **Crash-safe curve writes** (db.rs `with_txn`). Every delete-then-append writer
      (computed curves, restore/delete log set, core/aux/SCAL/curve-sample/well-path inserts,
      group members, zones-from-tops) now runs inside a single BEGIN/COMMIT/ROLLBACK, so an
      app kill mid-write can no longer leave the DELETE committed but the re-append lost.
      Nothing to click — just note that a tauri-dev restart mid-run won't silently drop curves.
- [x] **IMTS clay-conductivity direction fixed** (lrlc.rs sw*imts). The excess-conductivity
      term now \_divides* by Sw (Waxman-Smits `Cw + B·Qv_eff/Sw`), so it grows as hydrocarbon
      displaces water instead of vanishing. IMTS SwE now sits at/just below Waxman-Smits in
      pay (the old `·Sw` form gave Sw^(n\*+1) and over-stated Sw exactly in the LRLC pay this
      method exists to find). Re-run sw_imts on an LRLC interval and confirm SwE dropped.
      Test: `imts_credits_clay_conductivity_in_pay_zone`.
- [ ] **DLIS null sentinels + no silent overwrite** (dlis.rs). DLIS absent/sentinel values
      (−999.25/−9999, non-finite, |v|>1e30) are screened to MISSING on import, and each DLIS
      frame gets its own run number so a frame-0 channel no longer silently replaces a
      same-mnemonic LAS curve — the status line reports "replaced N existing curve(s)" when a
      collision does happen. Re-import a DLIS over a well that already has LAS curves and check
      the replaced-count note.
- [ ] **SandiMin refuses under-determined models** (multimin2.rs). Selecting fewer than
      (components − 1) input tools is now rejected up front ("need at least N input logs to
      constrain M components") instead of solving to an arbitrary vertex, and per-sample the
      solver skips depths with too few live curves. Test: `rejects_underdetermined_request`.
- [ ] **LAS import survives duplicate/odd-depth files on BOTH stores** (parsers.rs, ingest.rs).
      Non-finite and duplicate depths are dropped (first occurrence kept) before insert, so a
      **spliced/merged LAS with a repeated depth section** imports instead of aborting on the
      `(well_id, depth)` PK — and the fix now covers the _generic_ store too (PEF/CALI/extra
      runs), which previously PK-failed silently and left the well without those curves. Extras:
      a Schlumberger **TDEP**-indexed (or any non-`DEPT`) file resolves depth via the first
      column; an auxiliary **MD/TDEP track** in a later column can no longer steal the depth
      role from the true first-column index; a file whose depth is entirely null now **errors
      cleanly** instead of creating an empty orphan well; and a non-monotonic depth (column 0
      wasn't really the index) is surfaced as an import warning. The dropped/duplicate/odd-index
      counts appear in the import status line and History. Import a re-spliced LAS and a TDEP
      file and confirm both load with the expected row counts. Tests:
      `duplicate_depth_las_imports_standard_and_generic_curves`,
      `all_null_depth_las_errors_without_creating_well`,
      `parse_las_2_auxiliary_md_curve_does_not_steal_depth`,
      `sanitize_dedups_signed_zero_depths`, `parse_las_2_tdep_index_populates_depth`.

## P1 — reliability (frontend state) (2026-07-20)

The three P1 reliability findings from ROADMAP §4b (stale plots, async races, listener
leaks). Frontend-only (TypeScript, `tsc` clean); these are async-lifecycle behaviors with
no unit tests, so they were hardened by an adversarial review (4 lenses → per-finding
skeptical verify: **6 confirmed / 0 refuted**, all fixed — including one real HIGH bug in
the first-pass init guard) plus a focused **second-pass verify of the fixes** (renderer-dispose

- sticky-reset lenses: **clean, 0 defects**). Click-through in `npm run tauri dev`:

* [ ] **Plots refresh after a module run, keeping their viewport** (histogramPanel,
      crossplotPanel, pickettPanel, correlationPanel, logViewPanel). Open a Histogram of
      PHIE (zoom in), then run a module/equation that recomputes PHIE (or import / undo —
      anything that bumps `dataVersion`). The plot now re-reads the new curve **in place**,
      preserving your current zoom/pan, instead of showing the pre-run curve until you close
      and reopen the panel. Each builder subscribes to `appState.dataVersion` and calls
      `reload(preserveView=true)`; a `dataPrimed` guard swallows the subscribe's immediate
      fire so nothing double-loads on open.
* [ ] **Fast well/curve/zone switching never shows stale data** (loadWell, reload,
      createPlot). Click quickly through 5 wells in the Log view, or spam curve/zone changes
      in the Crossplot/Pickett/Histogram. A slow earlier load can no longer land after and
      overwrite a newer selection — each async load captures a generation token before its
      first `await` and bails if superseded. And a viewport **reset** intent (switching wells
      / changing the curve) still fires exactly once even if a `preserveView` refresh commits
      first, via a sticky `resetPending`/`viewResetPending` flag — so you neither keep a stale
      zoom that should have reset nor lose a reset that a background refresh raced past.
* [ ] **Opening/closing Log panels doesn't leak listeners or GPU loops** (logViewPanel
      dispose, LogCanvasRenderer). Repeatedly open and close Log view panels. The renderer's
      `window` pointerup/pointermove handlers are now removed and its `requestAnimationFrame`
      loop cancelled on `dispose()` (previously leaked one set per open); disposing a panel
      _during_ WebGPU init now disposes the fully-initialized local renderer rather than a
      no-op on an already-nulled field. Nothing visible per-close, but memory/handler count
      stays flat over a long session.
* [ ] **Dialog Escape is scoped to the dialog — closes the P1 modal-Escape sliver** (modal.ts).
      The carried-over P1 sliver ("overlapping dialogs share one Escape handler"). The listener-
      leak half was already handled — `openModal` single-instances via `activeClose` (a new dialog
      closes the prior one and removes its keydown listener; no modal opens a nested modal, so
      there's no stack). The remaining gap: the dialog's Escape was a `document`-level handler that
      closed the dialog but didn't `stopPropagation`, so one Escape also bubbled to `window`/app-level
      Escape handlers. It now stops there — kept on the **bubble** phase so the numeric-edit guard's
      capture-phase `stopPropagation` still shields a dialog from closing while you edit a number
      field. Also tears down any in-flight title-bar drag listeners on close (no leak if a dialog is
      dismissed mid-drag). tsc clean. Test: start drawing a **map polygon**, open any dialog (⚙
      Properties, an import dialog…), press **Escape** → the dialog closes and the half-drawn polygon
      is **still there** (Escape no longer cancels it too); double-click a number field in a dialog,
      press Escape → you exit the field's edit mode and the **dialog stays open**; a second Escape
      closes the dialog.

## Polish — UX (veteran-interpreter friction) (2026-07-20)

Hardening-backlog **Polish** tier (ROADMAP §4b, was "P3"). Small, mostly-frontend fixes;
each tsc-clean. Mapped by a read-only investigation wave before implementing.

- [ ] **Cursor readout: real units + no more mangled values** (plotCommon.ts `formatValue`,
      viewerChrome.ts `renderReadout`, logViewPanel.ts). The log-view cursor readout used a
      blanket `toFixed(2)`, which flattened permeability 0.003 → "0.00" and showed no units.
      It now uses an adaptive significant-figures formatter (perm stays "0.003", RT reads
      "2151", φ "0.18") and appends the unit the curve catalog already carries (RT "ohm.m",
      PHI "v/v", RHOB "g/cc"). Units are cached per well-load and refresh on `dataVersion`.
      Test: hover the log view over a permeability track and a deep-resistivity track — values
      keep resolution and each shows its unit. (Values whose catalog unit is blank show no unit.)
- [ ] **Correlation: fresh well list + Ctrl+wheel zoom** (correlationPanel.ts). The Wells
      menu was built once and never refreshed, so a newly imported well never appeared. It now
      re-fetches the well list on `dataVersion` — new wells appear (and draw as strips), deleted
      wells drop out, active-group filter re-applies. Also added **Ctrl/Cmd+wheel zoom about the
      cursor** (same factors as the other plots); plain wheel still pans through depth. Test:
      open Correlation, import/deviation-load another well → it shows up without reopening the
      panel; Ctrl+wheel over the strips zooms at the cursor depth.
- [ ] **Processing history now covers every operation** (processLog.ts + call sites across
      ribbon / inspectorPanel / mlDialog / monteCarloDialog / workflowDialog / zonesDialog /
      topsEditor / mapPanel / cutoffDialog). The audit trail (History panel / QAT History)
      previously logged only LAS import/export, module runs, core shift, well header, curve edits,
      project/session, exports. It now ALSO records: **DLIS / deviation / SCAL / core / tops
      imports; equation runs; ML runs; Monte Carlo; workflow chains; log-set restore/delete; zone
      add/edit/delete + per-zone parameter overrides; manual tops add/edit/delete; cutoff-default
      saves; map-polygon→group assignment.** Test: perform each and confirm it appears in the
      History panel with the right `[kind]` and detail. (Batch/field-wide actions — equation, ML,
      MC, workflow, log-set, cutoffs — intentionally show no well name.)
- [ ] **Pickett v2 — properties dialog, typed M/Rw, configurable axes, Z-color** (pickettPanel.ts).
      The Pickett plot's RT/PHIE axes were hard-coded (0.1–1000 / 0.01–1) with no properties
      dialog. Now a **⚙ / right-click Properties** dialog sets RT & PHIE axis ranges, point size,
      and **color-by-curve** (rainbow/viridis, optional log-Z), persisted via plotprops. The
      toolbar gained **M and Rw** fields next to N — type them and the Sw=1 / iso-Sw lines follow;
      a two-point pick still fits the line and fills the same M/Rw fields (one shared source).
      Zoom/pan and the line survive a data refresh (P1 preserveView). Test: open Pickett on a well
      with RES_DEEP + computed PHIE; pick two points on the wet trend → M/Rw fill and lines draw;
      type a different Rw → lines follow; right-click → set RT axis 0.2–200 and color by SW/VSH →
      points recolor; reopen the panel → settings persist.
- [ ] **Pay-summary provenance — FLAG\_\* versioned + cutoffs recorded** (workflow.rs, backend).
      run*pay_summary wrote FLAG_SAND/RESERVOIR/PAY with the old in-place `write_computed_curve`
      — no version history, and the VSH/PHIE/SWE cutoffs that produced them were recorded nowhere.
      Now the explicit **Cutoffs & Pay Summary** run versions the three flags into a **PAYFLAG**
      log set whose provenance = module `pay_summary` + the cutoffs (in `log_sets.params_json`) +
      inputs, exactly like every other module output — so a re-run keeps history and any version
      is restorable/prunable from the Curve Catalog. The **Field Dashboard** and **report** passes
      set `skip_version` (field-wide QC side-effect) so they keep overwriting in place — no version
      churn per refresh. Test `pay_summary_versions_flags_with_cutoffs_in_provenance` (161 lib
      tests pass / 0 fail / 7 ignored; tsc clean). Click-through: run Cutoffs & Pay Summary on a
      well → Curve Catalog shows a PAYFLAG version whose provenance lists the cutoffs; re-run →
      version N+1; run the Field Dashboard → FLAG*\* update but no new version piles up.

## Performance (field-scale speed) (2026-07-20)

Hardening-backlog **Performance** tier (ROADMAP §4b, was "P2"). The rest of the tier (#128–132)
changes DB/IPC semantics and needs a live 100-well benchmark to sign off; this first item is the
one pure-frontend, low-risk win.

- [ ] **Crossplot: Z coloring memoized across pan/zoom/hover** (crossplotPanel.ts). Every crossplot
      redraw rebuilt the whole per-point color array from scratch — for a continuous Z that's **two
      `percentile` sorts** (each allocates + sorts a NaN-filtered copy of all samples) plus an
      N-length `colorRampEx` string array; for a discrete Z, an N-length `categoricalColors` map.
      That ran on **every** pan-drag / zoom-wheel / handle-drag `mousemove` and every synchronized-
      hover frame, even though the colors depend only on the Z data + colormap, never the viewport.
      The color computation is now a pure `computeCrossplotColors()` that the panel **memoizes**,
      keyed by (Z curve, colormap, log-Z, fixed color, data generation); pan/zoom/hover reuse the
      cached array and only a data or color-setting change recomputes. Output is pixel-identical —
      this is a speed change only, most visible on dense (100-well / full-field) clouds. tsc clean.
      Test: open a crossplot colored by a curve (e.g. NPHI-RHOB by GR, or a PERM Z with log-Z on),
      drag the parameter handle / Ctrl+wheel-zoom / pan / hover from a log view — motion stays
      smooth on a big cloud; the colors, color-bar range, and facies legend are unchanged; switching
      the Z curve, colormap, or log-Z toggle still recolors immediately; a module re-run (dataVersion)
      recolors against the new data.

## Low-tier correctness & data-integrity sweep (2026-07-21)

The 15 low-severity findings from `AUDIT-2026-07-20.md` (never adversarially verified at audit time)
were each re-checked against the current code by an independent per-finding verifier. One was
**already fixed** (SandiMin all-zero conductivity-row guard, `multimin2.rs`), two are **held for your
sign-off** because they change numeric output (Wyllie compaction correction; histogram re-bin), one is
**held to land with the depth-scale-ratio fix** (scale-dropdown staleness). The rest — the safe
correctness/crash/data-integrity fixes below — are applied. **Rust suite green; tsc clean.** Nothing
committed.

Backend (with new regression tests):

- [ ] **SSC `SWIRR_EFF` no longer 0 at a 100 %-shale point** (`ssc.rs`). At the wet-clay point effective
      porosity is floored to 0, and the `1 − φt·(1−SWIRR_T)/φie` divide gave `−inf→0` ("all water
      movable") or `0/0→NaN` — exactly backwards. Now a zero-effective-porosity sample reports
      `SWIRR_EFF = 1.0` (fully bound). _Only the degenerate φie==0 samples change; every producing
      point is unchanged._ (The deeper SWIRR_T/SWIRR_EFF ordering inconsistency is the separate held
      item.) Test: run SSC on a shale-heavy well; SWIRR_EFF in massive shale reads ~1, not 0.
- [ ] **Archie `SWT_ARCH` no longer writes `+Infinity`** (`modules.rs` `sw_arch`). A coal/tight sample
      with PHIT=0 but PHIE absent used to fall through to `a/0^m = +inf` and store it in the SWT_ARCH
      curve, poisoning catalog min/max and plot autoscale. The zero-porosity "all water" guard now
      keys on PHIT alone. Test (regression `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf`):
      the curve catalog's SWT_ARCH min/max stays finite over coal/tight zones.
- [ ] **Simandoux (SCHLUMBERGER) no longer divides by zero at VSH=1** (`modules.rs` `sw_sim`). Pure
      shale hit a `1/(1−VSH)` singularity and the sample was silently dropped; it now resolves to
      all-water (SWE=1), matching the low-porosity and Indonesia branches. Test:
      `sw_sim_schlumberger_pure_shale_is_all_water`.
- [ ] **LAS import fails loudly on a truncated row** (`parsers.rs`, both parsers). A physically short
      `~A` row used to shift every following value one column left silently (GR into RES, RHOB into
      DT) for the rest of the file. Leftover tokens at EOF now raise a clear import error instead of
      mis-columning. Test: import a LAS whose last data line is cut mid-row → you get an explicit
      "leftover token(s)…truncated or corrupt LAS?" error, not corrupted curves.
- [ ] **DB-inspector edit no longer reports success on a 0-row update** (`db.rs`, all three sample
      editors). If the matched depth had moved/been rewritten, the UPDATE hit 0 rows but the UI said
      "saved" and pushed a bogus undo entry. It now errors, and the inspector already reverts the cell + shows "Edit failed". Test (`…is_err()` assertion added): edit a sample, then edit a
      non-existent depth → "no … sample matched depth …", cell reverts, no phantom undo.
- [ ] **Well Header shows current TD / KB** (`db.rs` list_wells + `ipc.ts` + `ribbon.ts`). The dialog
      used to open with blank TD/KB, so you edited the datum blind — and KB silently drives TVDSS in
      deviation import. TD/KB are now carried on `WellSummary` and prefilled. Test: open Well Header on
      a well with a KB set → the field shows it, not an empty box.

Frontend:

- [ ] **Stats / regression reject `±Infinity`, not just NaN** (`plotCanvas.ts`: basicStats, linearFit,
      percentile, drawScatter/drawDiamonds). One inf sample (e.g. a Python `1/phi` at phi=0) used to
      make the histogram's Mean/Std chips read "Infinity" and silently kill a crossplot regression.
      Now non-finite values are skipped everywhere. Test: compute an equation that divides by a
      zero-porosity sample, then histogram/crossplot it — chips and the fit stay sane.
- [ ] **Zone-param "Set" button surfaces write failures** (`plotCommon.ts` pickRow — histogram Pick
      A/B, Pickett M/Rw). A rejected `setZoneParam` used to be swallowed while the status still said
      "set". It now shows "Failed to set …". Test: (hard to force by hand) — behaviour only differs on
      a backend write error; the success path is unchanged.
- [ ] **Duplicate track titles prevented** (`layoutPropsDialog.ts`). Renaming a track to an existing
      track's title collapsed both in every title-keyed lookup (weights, cursor hit-testing, core
      overlay, drag-drop). A colliding rename is now auto-suffixed ("RES 2"). Test: in Layout
      Properties, rename a track to another track's exact name → it becomes "name 2"; retyping a
      track's own name is a no-op.
- [ ] **Histogram: constant curves render; the `n` never silently disagrees** (`histogramPanel.ts`).
      A constant curve (flag/class curve, single-sample zone) used to show "No valid data"; it now
      draws one central bar. And when the P2–P98 axis window clips tail samples, the axis label reads
      `n = X of Y` so it no longer contradicts the stats chips (which count all samples). _(The full
      full-range re-bin — which would change every bar height — is the held item.)_ Test: histogram a
      constant/flag curve (draws), and a curve with fat tails (label shows "of").
- [ ] **Log-view smoothness** (`LogCanvasRenderer.ts`, speed only). The clear color is no longer read
      via `getComputedStyle` every rendered frame (cached, invalidated on theme change), and the
      cursor readout uses a binary search instead of scanning every sample per mouse-move. Values and
      colors are identical. Test: drag-pan a busy log view — motion is smoother; theme switch still
      repaints; the cursor readout still tracks correctly.

### Held-item resolutions (2026-07-21 — your call: 1 yes / 2 leave / 3 yes / 4 yes + Bahasa Jawa)

Your answers to the four held items above. **Rust suite 164 pass / 0 fail; tsc EXIT 0; the two
browser-observable pieces verified live in the vite preview.**

- [ ] **Wyllie lack-of-compaction (Cp) correction — shipped as opt-in** (`modules.rs` `phi_son`,
      `OPT_CP` **default OFF**). ON divides the WYLLIE porosity by `Cp = DT_SH/100`; RHG is
      self-compacting and is never touched. Nothing changes until you switch it on. Test
      (`phi_son_wyllie_cp_opt_in_only_scales_wyllie`): OFF unchanged; ON ≈ +11 % at DT_SH=90; RHG
      unaffected. In the app: run Porosity → Sonic with OPT_CP=ON on a shallow well → PHIT rises a
      few p.u.; OPT_CP=OFF reproduces the old numbers exactly.
- **Histogram full-range re-bin — left as-is** at your request (bars keep clipping the extreme tails;
  bar heights and the mode/P50 you read off them are unchanged).
- [ ] **Depth-scale dropdown now shows the TRUE scale + the mislabel is fixed**
      (`LogCanvasRenderer.ts`, `logViewPanel.ts`). The default was labelled "1:100" but was really
      ~1:3937, and the `[0.02, 20]` px/unit clamp made **1:20, 1:50 and 1:100 all collapse to the same
      zoom**. Now: a true 1:1 = `96/0.0254` px per depth unit is single-sourced; the view opens at an
      honest **1:2000**; the clamp reaches a true 1:10; and after any Ctrl+wheel/± zoom the selector
      re-reads the live ratio (a transient "1:N ⟳" entry when it's between presets). Test: pick 1:50
      then 1:100 → visibly different scales (identical before); Ctrl+wheel zoom → the box tracks the
      real ratio instead of freezing on the last preset.
- [ ] **Quiet Ctrl+S save + Escape closes ribbon menus** (`ribbon.ts`). Ctrl/Cmd+S re-saves the
      current session in place once it has a name (no dialog), falling back to Save Session As the
      first time; it's ignored while typing in an input/CodeMirror so editors keep their own Save.
      Escape closes any open ribbon dropdown without disturbing modal Escape handling. _(A Ctrl+P
      print-active-plot shortcut was deliberately deferred — resolving "the active canvas" from the
      ribbon is fragile; the per-plot Print button still works.)_ Test: name a session, edit the
      workspace, Ctrl+S → "Session … saved" with no dialog and the unsaved dot clears.
- [ ] **Bahasa Jawa (jv) added + fuller Bahasa Indonesia / Basa Sunda** (`i18n.ts`, `index.html`).
      A full Javanese (ngoko) dictionary joins id/su, and ~55 common UI phrases (New/Open/Edit/Search/
      Print/Value/Zone/Session/… + statuses) were added to all three — petrophysics jargon still stays
      English by design. Test: Project → Language → **Basa Jawa** → menus/buttons switch (Save→Simpen,
      Depth→Jero, Reload→"Muat manèh"); switch back to English → everything reverts from source.

## Reference-library correctness fixes (2026-07-20)

Two physics fixes distilled from the ITB team reference shelf (Ellis Ch12/14, Halliburton FE Ch27):

- [ ] **Multimin — PEF now converts to U before mixing.** In the Multimin (SandiMin) dialog,
      select **Photoelectric (PEF)** as an input tool (instead of, or alongside, U) and run on
      a well with a PEF curve + RHOB. Confirm VOL\_\* volumes are sensible and RECON is low in
      clean zones. Physics: per-electron PEF does NOT mix by volume — the solver now converts
      the PEF curve to U = Pe·ρe per sample (ρe from RHOB) and mixes against the U endpoints.
      Picking U directly is unchanged. Needs a RHOB curve present; where RHOB is missing the
      PEF row is simply skipped that sample.
- [ ] **VSH from Density-Neutron — new VSH_DN_FLAG clay-type guard.** Run **VSH from
      Density-Neutron** with the optional **GR** input mapped and set GR_MA/GR_SH/FLAG_TOL.
      Confirm a new `VSH_DN_FLAG` curve = 1 where the N-D VSH is off-model (gas crossover /
      beyond the shale point) or diverges from the GR VSH by more than FLAG_TOL (0.25 v/v
      default) — the signature of clay-type or gas ambiguity. Leaving GR unmapped still flags
      off-model samples. VSH/VSH_DN themselves are unchanged.

## Field Map — well surface coordinates + polygon → group (2026-07-20 #27)

**Field Map** (View/Batch ribbon map button, or the Petrophysics ▸ Field Map… button) — a
standalone dock pane that posts wells by UTM surface location and lets you rubber-band a
polygon to select wells into a well group. Coordinates arrive two ways: **Data ▸ Import Well
Locations…** (a CSV/TXT with EASTING/NORTHING, optional WELL and ZONE columns, plus a
choosable default UTM zone covering Indonesia — zones 46–54, N and S — applied to rows/files
without a ZONE column), or per-well via **Tools ▸ Well Header** (Surface X / Surface Y / UTM
zone fields). Coordinates persist as DOUBLE in new `wells` columns
(surface_x/surface_y/utm_zone) — southern-hemisphere northings ≈ 9.4e6 exceed f32's ~1 m
precision, so f64 is required. The pane draws markers with pan (drag), cursor-anchored
wheel-zoom, a faint coordinate grid, a scale bar, and labels for ≤80 wells; the active well
group is ringed. Draw mode: click to drop polygon vertices, close near the first vertex /
double-click / Enter; vertices are draggable; enclosed wells highlight live (a TS
point-in-polygon mirrors the Rust ray-cast). **Assign to group…** runs the authoritative
backend `wells_in_polygon` (PNPOLY, half-open crossing rule) and unions the result into a new
or existing group. Raw easting/northing is plotted directly — no reprojection; a multi-zone
project is a documented follow-on. The polygon is a transient selection tool — the persistent
artifact is the well-group membership (persisting polygon shapes as documents is a noted
follow-on vs the roadmap's original wording).

Adversarial review (4 lenses — geometry-math / import-parse / integration /
frontend-robustness, each finding skeptically verified): **3 defects confirmed, all fixed; 0
refuted.** (1) [high] Well Header Save wrote surface_x/y/zone unconditionally from a stale
`selectedWell` snapshot that is never re-broadcast on a data change, so re-saving after an
import (or a prior save) NULLed out the just-set coordinates — fixed by re-reading the well
from the DB when the dialog opens, so the fields always reflect current state. (2) [medium] A
blank WELL cell in a multi-well locations file collapsed into the same "no well column" case
as a headerless file and was routed to the selected well, silently overwriting an unrelated
well's location — fixed by returning a `has_well_column` flag from the parser so the importer
only falls back to the selected well for a genuinely column-less file, and skips (and
reports) blank-cell rows; the import loop is now wrapped in a transaction so a mid-file error
rolls back instead of leaving a partial write reported as a total failure. (3) [medium] When
coordinates first arrived while the pane was already open, the data-driven reload never fit
the view, so markers rendered off-screen until a manual Fit — fixed by fitting on the first
appearance of laid-out wells. cargo test 143/143; tsc clean.

- [ ] **Data ▸ Import Well Locations…** — pick your Indonesia UTM zone (e.g. 50S for
      Mahakam), import a CSV with WELL/EASTING/NORTHING → the status line reports N wells
      located; open **Field Map** and confirm the wells post at the right relative geometry.
- [ ] Import a file that has a WELL column but a blank cell in one row → that row is skipped
      and surfaced as "1 blank-WELL row(s)", and no unrelated well's location changes.
- [ ] **Tools ▸ Well Header** on a located well → Surface X/Y/zone show the imported values
      (not blank); change only TD and Save → the coordinates survive (were being wiped before).
- [ ] With Field Map already open on a project that had no coordinates, run Import Well
      Locations → the map fits to the new wells automatically (no manual Fit needed).
- [ ] Field Map ▸ **Draw polygon**, enclose a few wells, **Assign to group…** → the enclosed
      wells land in the chosen/new well group; the group filter elsewhere reflects it.

## φmax porosity ceiling — phimax module (2026-07-20 #26)

**Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** — caps a computed porosity at
the field's compaction-controlled upper limit (the deck slide-64 "max core porosity"
line). A `MODE` dropdown picks the ceiling model: **constant** (a flat `PHIMAX0`,
per-zone overridable — the literal max-line), **linear** (`φmax = PHIMAX0 −
PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000`), or **athy** (`φmax = PHIMAX0·exp(−ATHY_K·
(TVDSS − TVDSS_REF)/1000)`, the exponential compaction law). TVDSS is a
positive-downward depth-below-datum curve (same convention as **precalc**), so deeper
= larger TVDSS = lower ceiling; with no TVDSS curve it falls back whole-curve to
measured DEPTH (fine for near-vertical wells). All four parameters are zone-overridable,
so each formation (Post-Main / Main / Massive / Talang Akar) can carry its own ceiling
or its own trend coefficients. Writes `<PHI>_CAP = min(PHI, φmax)` (preserving MISSING)
and the ceiling curve `<PHI>_MAX` for a QC overlay; the input porosity is never
modified. The dialog is auto-generated from the manifest, so it appears in the Porosity
dropdown with no bespoke UI. Standalone by design — it caps _any_ porosity output
(phi_den/phi_dn/phi_son or SandiMin's PHIT); a solver-internal φmax box constraint is a
noted follow-on.

Adversarial review (4 lenses — math / integration / edge / contract — each finding
verified): **0 defects confirmed, 4 refuted** (all were test-coverage/doc-completeness
notes over correct, deliberate behaviour). Two of the flagged-untested paths were
locked in with regression guards anyway: the ceiling clamp to [0,1] (a sub-zero trend
ceiling forces porosity to 0; a super-unit one clamps to 1), and the partial-NaN TVDSS
pass-through (a NaN-depth sample gets a MISSING ceiling and passes porosity through
uncapped). cargo test 136/136; tsc clean.

- [ ] **Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** opens (auto-dialog). Run
      **constant** mode, PHIMAX0 = your field max (e.g. 0.35), input **PHIE** → the
      `PHIE_CAP` curve should equal PHIE below 0.35 and flatten at 0.35 above it;
      `PHIE_MAX` is a flat 0.35 line.
- [ ] Crossplot `PHIE_CAP` vs depth (or overlay `PHIE_MAX` on the porosity track) — the
      capped cloud should sit under the ceiling with no points poking above it.
- [ ] Switch to **linear** (or **athy**) with your TVDSS trend: PHIMAX0 at a shallow
      `TVDSS_REF`, a sensible `PHIMAX_GRAD` (or `ATHY_K`). `PHIE_MAX` should fall with
      depth; confirm a deep zone's ceiling is lower than a shallow zone's.
- [ ] Set **per-zone** `PHIMAX0` (or trend coeffs) in Zones/zone params and re-run —
      each formation's ceiling should honour its own value.
- [ ] Well with **no TVDSS curve**: linear/athy should still run (trend reads against MD
      DEPTH) — sanity-check that the ceiling still declines with depth. Deviated wells
      want a real TVDSS curve (survey→TVDSS bridge is a follow-on).
- [ ] Feed `PHIE_CAP` into **Cutoffs & Pay Summary** as the PHIE input — pay should drop
      where the cap trimmed optimistic porosity.

## Cutoff Sensitivity pane (2026-07-20 #25)

**Reporting ▸ Cutoff Sensitivity** — two ways to defend a VSH/PHIE/SWE pay cutoff
against DST-tested rock (KKT ONWJ deck slides 84–87), in one dock pane with a
Sweep / DST-Crossplot toggle. **Sweep** varies one cutoff across a range while the
other two stay fixed and plots the pay response per well — net thickness, HC
pore-thickness (HPV), or net-to-gross — so the _elbow_ shows where loosening the
cutoff stops adding real pay; the shared pay math is the **same `classify_sample`
the pay summary uses**, so the numbers reconcile. **DST Crossplot** is PHIE vs a
shale/Sw curve with every sample dim and DST-interval samples coloured per well,
plus a draggable red crosshair at the candidate cutoffs. Either mode's pick writes
into the VSH/PHIE/SWE fields and can be **saved as the pay-summary default** so the
cutoff you defended flows straight into the report. Optional zone and DST/perf
filters scope both modes.

Adversarial review raised 13, confirmed 10 (from two independent full passes),
all fixed before shipping: the sweep's PERM-cutoff scope now matches the pay
summary exactly (whole-well, not just the analysed window); overlapping
perforation/DST intervals are unioned so N:G isn't understated; switching the
swept property/metric after a run clears the stale plot so a pick can't be written
into the wrong cutoff; the "(all samples)" DST choice survives editing the well
set; the zone/DST pickers union over _all_ checked wells (was capped at 16); the
crosshair stays inside the plotted range; empty-state text is centred on HiDPI; the
plot repaints on theme change; wells with no pay / missing inputs are flagged
rather than shown as a silent flat line. cargo test 131/131; tsc clean.

- [ ] **Reporting ▸ Cutoff Sensitivity** opens as a dock pane. Tick a few wells,
      keep **Sweep**, property **VSH**, metric **Net**, Compute — one line per well;
      the curve should rise and flatten (an elbow), not be a straight ramp.
- [ ] Click/drag on the plot to place the red cutoff line; the readout shows the
      net/HPV/N:G each well delivers _at that cutoff_. Click **Use pick as VSH
      cutoff** → the VSH field updates.
- [ ] Switch the metric to **N:G**, Compute again; switch property to **PHIE** — the
      plot should **clear** and ask you to Compute (it must not keep showing the VSH
      sweep while the button says "PHIE").
- [ ] Cross-check one well against **Cutoffs & Pay Summary** at the same fixed
      VSH/PHIE/SWE (whole well, no zone/DST): the sweep's Net at those cutoffs should
      match the pay summary's Net for that well. Repeat with a **PERM ≥** cutoff set.
- [ ] **DST Crossplot** mode with a well that has a DST/perf set: dim cloud + coloured
      DST points; drag the crosshair; **Apply crosshair → cutoffs** writes PHIE and
      VSH (or SWE if the X curve is an Sw). Pick a "PHIE vs Sw" preset and confirm the
      Apply maps to SWE.
- [ ] Pick a DST set, then switch the DST dropdown to **(all samples)**, then tick/untick
      a well — the dropdown must **stay** on "(all samples)" (not snap back to the DST set).
- [ ] **Save as pay-summary default** → open **Cutoffs & Pay Summary**: its VSH/PHIE/SWE
      inputs should already carry your saved cutoffs.
- [ ] Switch the app theme with the pane open — the plot repaints immediately in the
      new palette (no stale colours).

## All tools as dockview panes (2026-07-20 #24)

Your ask: "i want all tools shows as pane, for existing and future tools." Every
computation/analysis tool now opens as a **dockable pane** instead of a pop-up. The
big one: the **auto-generated module form** (every Petrophysics ▸ Data Prep / VSH /
Porosity / Saturation module) is now a pane — one per module — so you can keep
several docked side by side and re-run each as you iterate, and **any new module I
add in Rust gets its pane automatically** with no extra UI work. **Zones,
Autocorrelate Tops, Composite Log, and Report** are panes too; they follow the
selected well the way the plots do, and refresh their lists when data changes. Quick
pop-ups stayed pop-ups on purpose (curve editor, layout properties, save/open
session, import prompts). Adversarial review found 9 real issues, all fixed before
this shipped (pin-off panes catching up to a selection, no stale-well writes after a
project switch, the autocorrelate "pick a top first" message re-checking itself once
you pick one, etc.). tsc clean; module-pane behavior browser-verified.

- [ ] Open a module (e.g. Gas Correction) from the Petrophysics tab — it should
      appear as a pane you can dock/split/float, not a pop-up. Run it; the result
      lines stay in the pane (no auto-close). Open a second module — both panes
      coexist (the old pop-ups could not).
- [ ] With a module pane open, compute a curve, then open another module: the new
      curve should already be selectable in its input dropdowns (the pane refreshes
      its lists on data changes without losing what you'd already picked).
- [ ] Multi-select several wells in Wells & Tops, THEN open a module: all selected
      wells should be pre-ticked (not just the active one).
- [ ] Open the **Zones** / **Composite** / **Report** pane with no well selected —
      it shows "Select a well… will follow" instead of a "select a well first"
      toast; pick a well and it fills in and the tab title updates.
- [ ] **Autocorrelate Tops** on a well with no tops: the pane says "pick one in the
      log view first" — go pick a top, and the pane should update itself (no need to
      close/reopen). Apply a correlation: the proposals clear.
- [ ] Switch projects with a Zones/Report pane docked in a background tab: it must
      reset to the "select a well" hint, NOT keep showing a well from the old
      project (this prevents editing the new project with a stale well).
- [ ] Docking sanity: the panes save/restore with the workspace layout, appear in
      the ＋ "add panel" menu, and the log-view right-click "Print / export layout…"
      opens the Composite pane.

## Gas Correction module — iterated density de-gassing (2026-07-20 #23)

**Petrophysics ▸ Data Prep ▸ Gas Correction (density, iterated)** — the KKT deck
slide-65 loop. Density porosity and Archie SWT are solved from the current density,
then RHOB_GC = RHOB + Φt·(1−Sw)·(RHO_FL − GASDEN) replaces gas with liquid, iterated
to |ΔΦt| < 1e-4 (non-converging samples stay MISSING). GASDEN is the real-gas density
of an SG_GAS 0.65 gas at FPRESS/FTEMP (Standing pseudo-criticals + Papay z, pinned
0.1297 g/cc at the KK example's 2743 psi / 93.9 °C) — **run precalc first**; FTEMP and
FPRESS accept only precalc/log-set curves, never a raw import (a Geolog LAS's degF
FTEMP can't sneak in as degC). Default **OPT_GATE = FLAGGED** corrects only where the
gas flag > 0.5 (chain condflag's XOVER_FLAG, which excludes coal and washout) and
errors loudly if the flag curve has no data; **EVERYWHERE** is there for wells without
condflag, but beware coals/resistive washouts — high RT + low density reads as gas to
the Archie loop. The adversarial review raised 13 confirmed findings → all fixed
(FLAGGED default, flag > 0.5 gate, no-flag-data error, degenerate RHO_MA/RHO_FL and
RHOB<RHO_FL and Rw≤0 guards, non-convergence → MISSING, NaN-proof Archie cap,
computed-only P/T inputs, RHOG→GASDEN rename, doc rewrite). 127 cargo tests green.

- [ ] Run precalc → condflag → Gas Correction (defaults) on a KK-style gas well: the
      detached high-porosity gas cloud on PHIE vs wet-clay (slides 66–67) should
      collapse after correction; RHOB_GC ≈ RHOB in water zones (self-limiting there).
- [ ] Check a coal streak stays untouched under the FLAGGED default (XOVER_FLAG
      excludes coal) — no phantom high-porosity pay in coals.
- [ ] Without condflag run: the FLAGGED default must error "gas flag has no data —
      run condflag first or set OPT_GATE = EVERYWHERE", not silently pass through.
- [ ] Without precalc run: outputs stay MISSING (never uncorrected pass-through),
      even if the well's LAS carries its own raw FTEMP/FPRESS curves.
- [ ] Feed RHOB_GC to **phi_den** (or use PHIT_GC directly). Do NOT feed it to phi_dn
      or a SandiMin solve that includes NPHI — their gas handling assumes an
      uncorrected density-neutron pair (the module doc says this too).

## SandiMin: wet→dry clay converter + fluid autofill from precalc (2026-07-20 #22)

Two additions inside the **SandiMin** pane (Advance tab), from your Multimin
Parameters.xlsx workflow (Wave E item 18). **Wet clay → dry clay** panel: enter the
wet-clay picks from a shale interval (RHOB/NPHI/GR, optional DT) and the assumed
dry-clay density (2.70 marine / 2.78 deltaic per the KKT deck slide 60); it computes
φ_clay = (ρdry−ρwet)/(ρdry−1) and the dry endpoints with the xlsx formulas verbatim
(water 1.00 g/cc, 189 µs/ft), previews them live, and **Apply** writes them to the
chosen clay, ticks it + BoundWater, and sets a **CEC_eq** on the clay that makes the
solver's Dual-Water bound-water constraint enforce exactly v_bw = φ/(1−φ)·v_dryclay —
the deck's slide-59 bookkeeping (SWB = VOL_UBNDWAT/PHIT). Unphysical picks error
instead of applying: NPHI must be a fraction (percent entry rejected — Geolog habit
guard), GR positive, wet DT above the 189·φ water term. **Autofill from precalc**
(fluid box): pick a zone of the selected well and **Read** — fills Formation temp
from FTEMP_F and the Rmf sample from precalc's RMF (retied to formation temp, an
Arps no-op, only when both curves came back; a raw RMF without FTEMP_F is refused
as not-precalc). The zone dropdown follows your well selection live.

- [ ] KK-1 Post Main check: wet 2.18333/0.48958/110 with dry density 2.70 → the
      preview must read φ_clay 0.3039, NPHI 0.2667, GR 158.0 (the xlsx values).
- [ ] Apply to Illite, then run SandiMin with CT on: solved VOL_UBNDWAT/VOL_DRYCLAY
      should sit at ~0.4366 (= φ/(1−φ)) in clay-rich intervals; SWB = VOL_UBNDWAT/PHIT
      comparable to the deck's slide-59 CWB-panel behaviour.
- [ ] Note the pairing rule: CEC_eq is tied to the clay's **RHOB endpoint** and the
      fluid **T/Rw/α** at Apply time — if you edit any of those afterwards, re-Apply
      (the status line and the CEC column tooltip both say so now).
- [ ] Autofill on a precalc'd well: Read (whole well and one zone) fills FTEMP/Rmf
      and the previews update; on a well without precalc it must refuse with "run
      the precalc module first", not fill garbage.
- [ ] Switch wells with the SandiMin pane open: the autofill zone list must follow
      the selection (it re-reads the new well's zones).

## Neutron Matrix Conversion module — NPHI LS/SS/DOL (2026-07-20 #21)

New Prep module **Neutron Matrix Conversion** (`nphimat`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). Converts a neutron log recorded in
one matrix convention into all three — **NPHI_LS / NPHI_SS / NPHI_DOL** — using the
chartbook porosity-equivalence curves digitized at vector precision: **Por-5** for
the CNL thermal tools (**NPHI** ratio method; **TNPH** env-corrected, FRESH / 250 kppm
SALT variants) and **Por-4** for the epithermal tools (**APLC/FPLC** = APS, **SNP** =
legacy sidewall). Tell it what the log is (TOOL + MATRIX_IN); the input convention
passes through unchanged and the other two are read through the chart (SS/DOL inputs
invert back to the apparent-limestone axis first). The book's printed worked example
(TNPH 18 pu @ 250 kppm → sandstone 24 pu) reproduces to 0.04 pu. Feed the output
matching your RHO_MA (NPHI_SS with 2.65) — that removes the ~0.04 LS-vs-SS offset the
condflag doc warns about, so XOVER_MIN can stay at 0.04. Also in this increment:
APS/legacy neutron mnemonics (APLC/FPLC/SNP/NPOR/HNPO/NEUT/FSTP) now fill the
standard NPHI column at LAS import, an all-NaN standard column now falls back to the
raw store (family alias) instead of silently feeding NaN to modules, and workflow-
builder input dropdowns now offer every module's outputs so `nphimat → phi_dn
(NPHI = NPHI_SS)` is buildable in a fresh project.

- [ ] Run nphimat on a Mahakam well (TOOL matching the delivery, MATRIX_IN per the
      LAS header — usually LS or SS): NPHI_SS ≈ NPHI_LS + 0.03-0.04 in clean sand,
      NPHI_DOL well below both (thermal dolomite bow).
- [ ] Sanity vs the paper chart: pick one depth, read Por-5 by hand, compare all
      three outputs (expect agreement within ~0.5 pu).
- [ ] Feed NPHI_SS + RHO_MA 2.65 into phi_dn / condflag: crossover in a known gas
      sand appears at XOVER_MIN 0.04 without the limestone-unit offset fudge.
- [ ] Workflow builder in a fresh project: chain nphimat → phi_dn with the NPHI
      input overridden to NPHI_SS (now offered in the dropdown before any run).
- [ ] If you have an APS well (APLC): import fills NPHI now — check the curve
      arrives and nphimat TOOL=APLC gives sensible (small) matrix shifts.

## Data Conditioning Flags module — coal / tight / crossover + shoulder (2026-07-20 #20)

New Prep module **Data Conditioning Flags** (`condflag`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). One run writes five 0/1 flag
curves: **COAL_FLAG** (RHOB < 1.9 & NPHI > 0.35, plus DT > 100 µs/ft where a sonic
exists; samples with BADHOLE = 1 are never called coal — washouts mimic coal),
**TIGHT_FLAG** (density porosity and NPHI both < 0.05; DPHI uses **RHO_MA/RHO_FL —
the same params and zone overrides as the density-porosity modules**),
**XOVER_FLAG** (gas crossover DPHI − NPHI > 0.04; coal and bad hole excluded —
NPHI must be matrix-consistent with RHO_MA, else raise XOVER_MIN to ~0.08 for
limestone-unit neutron), **SHOULDER_FLAG** (the adjustment you asked for: samples
within SHOULDER of a coal/tight bed edge — or a bad-hole interval ≥ MIN_THICK —
carry boundary-averaged readings and get flagged so no shoulder log survives the
mask), and **COND_FLAG** (combined mask: coal | tight | badhole | shoulder, plus
crossover only when OPT_XCOND = YES). Beds thinner than MIN_THICK are dropped as
spikes; a missing sample inside a bed does not split it. MIN_THICK/SHOULDER are
in the depth curve's unit (defaults suit metres — roughly ×3 for feet). Run
badhole first; feed COND_FLAG as the Mask on later runs, but leave the Mask empty
on the condflag run itself. BADHOLE and COND_FLAG are now always offered in every
Mask dropdown, even in a fresh project where they haven't been computed yet.

- [ ] Run badhole → condflag on a Mahakam well with coals: COAL_FLAG picks the
      coal streaks (check against the density track), and no coal call inside
      washouts.
- [ ] TIGHT_FLAG on a calcite-cemented/tight streak; XOVER_FLAG on a known gas
      sand; crossover NOT flagged over coals.
- [ ] SHOULDER_FLAG brackets each coal/tight bed by ~SHOULDER depth units; a
      lone one-sample BADHOLE blip is masked in COND_FLAG but does NOT dilate.
- [ ] MIN_THICK: single-sample spikes dropped; a real bed with one null sample
      in the middle is kept whole.
- [ ] Feed COND_FLAG as Mask on a porosity run: flagged + shoulder samples go
      missing in the outputs; confirm COND_FLAG appears in the Mask dropdown of
      a fresh workflow before condflag has ever run.
- [ ] Zone overrides: RHO_MA 2.71 in a carbonate zone shifts TIGHT/XOVER there
      (same override the density-porosity modules use).

## Wave E-17: pre-calculation module — P / T / Rmf / Ct / Cxo (2026-07-20 #19)

New Prep module **Pre-Calculation (P / T / Rmf / Ct / Cxo)** in the Data Prep
dropdown and the workflow builder (ROADMAP §4c item 17, from your KKT ONWJ
workflow). One run writes six curves: FTEMP (**always degC** — the unit every
downstream module assumes) plus FTEMP_F (the degF twin, for SandiMin fluid
entry) and FPRESS as linear trends in TVDSS (gradients per depth unit of the
TVDSS curve — per-metre values for metric wells; no TVDSS curve → measured
depth is used), RMF at formation temperature (ARPS from a surface Rmf
measurement, or TREND regression `RMF_A + RMF_B·log10(TVDSS)` for wells
without mud data — the shipped defaults are the ONWJ **feet-based** fit), and
CT = 1000/RT, CXO = 1000/RXO in mmho/m as QC/plotting conductivities (note:
SandiMin's CT/CXO tool rows read the resistivity curves directly — don't feed
these to them). Params are SURF_TEMP/TEMP_GRAD (own names, so zone overrides
never cross-apply with Formation Temperature's degC-only TSURF/TGRAD); entry
unit degF/degC via OPT_TU.

- [ ] Run it on a KKT-style well with your fits (SURF_TEMP 77 / TEMP_GRAD
      0.0260292, PSURF 44.2823 / PGRAD 0.539812, degF): FTEMP_F/FPRESS match
      the deck's trend lines; spot-check one depth by hand; FTEMP = same in degC.
- [ ] Deep resistivity input defaults to the RES*DEEP family (same as the sw*\*
      modules) so CT fills for wells whose deep curve is ILD/LLD/AT90 etc. —
      confirm CT is not blank on a standard import.
- [ ] ARPS mode: RMF at depth ≈ your surface Rmf pulled down by (T₁+6.77)/(T₂+6.77);
      TREND mode with A 0.517068 / B −0.116517 reproduces the field regression.
- [ ] degC mode on a metric well (e.g. SURF_TEMP 25, TEMP_GRAD 0.03 degC/m):
      FTEMP in degC, FTEMP_F in degF, RMF still Arps-correct.
- [ ] CT/CXO: 1000/RT and 1000/RXO, missing where RT/RXO are missing or ≤ 0.
- [ ] Zone overrides: give one zone a different TEMP_GRAD in the Zones dialog —
      the FTEMP trend kinks at the zone boundary (per-zone params resolve per
      sample).

## Wave A-4: workflow grid inspector (2026-07-20 #18)

The Workflow Builder pane has a **List | Grid** toggle above the step list
(ROADMAP §4c item 12). Grid = the multi-line inspector: rows are your chain's
steps, columns are the union of every step's inputs/params/options (+ Mask), so a
parameter shared by several modules lines up in one column. The italic **Set all**
row under the header edits a parameter across every step that takes it in one go.

- [ ] Build your standard chain (vsh → phi → sw\_\* …), switch to **Grid**: input
      curves come first, then numeric params, then options, then Mask; steps that
      don't take a column show "—". Header tooltips = parameter descriptions.
- [ ] **Set all → RW**: type one RW in the Set-all row — every sw\_\* step that takes
      RW updates at once (status bar reports how many). A value outside one
      module's allowed range is skipped for that module only and reported.
- [ ] Edited cells tint amber and the step's override badge counts up — same
      only-store-differences rule as the per-step editors, so a value typed equal
      to a module's default clears that override (cell untints). Zone params still
      override these whole-well values per zone at run time, as before.
- [ ] **Set all → Mask** sets opts.MASK (e.g. BADHOLE) on every step in one edit.
- [ ] Toggle List ↔ Grid: values, badges and invalid-input flagging stay in sync
      (both views edit the same steps). The chosen view is remembered.
- [ ] Save the workflow, reload it, re-run — saved JSON is unchanged in shape, so
      old saved workflows load into the grid fine.

## Wave A-3: project open/switch, IP style (2026-07-20 #17)

You can now keep separate project databases (balam.duckdb, minas.duckdb, …) and
switch between them inside the app (ROADMAP §4c item 2). Project ribbon tab, new
group left of Appearance:

- [ ] **New Project…** creates a fresh, empty .duckdb and switches to it — import a
      couple of Balam South LAS files there, confirm they do NOT appear in your main
      project, then switch back.
- [ ] **Open Project…** switches to an existing file; **Recent ▾** lists the last 12
      projects (current one marked ●, deleted files greyed "(missing)"), stored in
      `%APPDATA%\SandiBumi\projects.json` — outside any project.
- [ ] On switch: window title + group caption show the project name, well list /
      plots / catalogs all reload, well selection and undo history clear (old-project
      undo entries would corrupt the new one — deliberate).
- [ ] **Next launch reopens the last project you had open** (falls back to the old
      `project.duckdb` if the recents list is empty — first launch after this update
      behaves exactly as before).
- [ ] Switching is refused while a workflow chain is running (try it: start a long
      chain, then Open Project — you should get a clear error, not a corrupted run).
- [ ] Note: QAT **Save Project As** stays a backup copy (app keeps working on the
      current file) — tell me if you'd rather it switch to the copy, IP-style.

## Wave A-2: compact import ribbon (2026-07-20 #16)

The Data tab's eleven flat import buttons are now three Office-style dropdowns
(ROADMAP §4c item 4) — same handlers, just organized:

- [ ] **Import Logs ▾** (LAS, DLIS), **Import Data ▾** (Core, SCAL, Tops, Aux,
      Deviation), **Export LAS** (unchanged flat button), **Tools ▾**
      (Autocorrelate Tops, Shift Core, Well Header). Run one import of each kind —
      behaviour must be identical to the old buttons; tooltips moved onto the
      menu entries.
- [ ] Only one menu opens at a time; picking an item or clicking elsewhere closes it.
- [ ] Bahasa Indonesia / Basa Sunda: the new labels translate (Impor Log / Impor
      Data / Alat) including the previously untranslated Import Tops / Import Aux /
      Autocorrelate entries.

## Wave A-1: tool panes + theme compliance (2026-07-20 #15)

Four tools moved from popup dialogs to dock panes (ROADMAP §4c item 14) — they now
dock/float/tab like the Workflow Builder and can't be dismissed by a stray click:

- [ ] **Cutoffs & Pay Summary**, **ML Models**, **Monte Carlo**, **SandiMin** ribbon
      buttons each open a PANE (singleton: clicking again focuses the existing one).
      Run each on Balam South data — results should be identical to the old popups.
- [ ] The ＋ add-panel menu on any window now lists all four (under Workflow Builder);
      the right-click menu inside each pane shows its own heading.
- [ ] SandiMin's endpoints matrix now uses the full pane width (was capped at 620px).
- [ ] Panes reopen after an app restart (from the autosaved workspace) in their
      docked position — internal selections (cutoff values etc.) reset, same as the
      Workflow Builder.
- [ ] **Theme check** (switch to Dark, then Pertamina): the log-view cursor readout
      pill now inverts with the theme (was unreadable in dark); crossplot/Pickett/
      histogram pick swatches + histogram pick markers follow the theme accents
      (Pertamina = blue/lime, was always brown/green); core-plug diamond outlines
      visible in dark; workflow invalid-input red and error text use the theme warn
      color; the composite preview surface is no longer light grey in dark themes.

## Chartbook overlay library + audit quick fixes (2026-07-20 #14)

The single D-N overlay grew into a **chart overlay library** (Properties → Overlays →
Chart overlay): every crossplot-family chart from your 2013 chartbook, digitized from
the PDF vector artwork with the same validation stack (graduation sequences, 5-multiple
long dashes, worked examples). Charts matching the current axes are listed first; a
chart draws only when the plot axes actually match it (either orientation).

- [x] **CNL Por-11/12** (as before, now via the new select — old saved props migrate).
- [x] **EcoScope Por-18 (BPHI) / Por-19 (TNPH)** on an LWD well — these are the ones
      that matter for your Mahakam development wells; check a known sand against the
      sandstone line for both BPHI and TNPH inputs.
- [x] **adnVISION675 Por-16** if you have ADN wells.
- [x] **APS Por-13/14** (APLC and FPLC variants listed separately).
- [x] **PEF: Lith-3/4** on a PEF-RHOB crossplot — quartz ~1.65-1.8, calcite ~5.08,
      dolomite ~3.1 curves with 10-pu labels.
- [x] **Sonic-neutron Por-20** (both time-average AND field-observation families) on
      a DT-NPHI crossplot — TA curves reproduce Wyllie with tf 190 to R² 0.99999.
- [x] **Density-sonic Por-22** (TA + FO) on a DT-RHOB crossplot, with the 7 mineral
      points (Sylvite, Salt, Trona, Gypsum, Sulfur, Polyhalite, Anhydrite).
- [x] **Th-K clay chart Lith-2** on a POTA-THOR crossplot — the Th/K ratio fan is
      drawn at the _labeled_ ratios (the chartbook's own printed lines sag a few %
      off their labels; ours are exact), plus the dashed clay/feldspar lines and
      mineral-field labels. Judge your Mahakam illite/kaolinite mix against it.
- [x] **Pe-K and Pe-Th/K clay boxes Lith-1** (the Th/K variant needs the X axis in
      log mode — turn on X log in Properties).
- [x] **Umaa-Rhomaa MID Lith-6** — the ternary triangle with 20/40/60/80 subdivisions + K-feldspar/Barite/Anhydrite/Kaolinite/Illite/Salt points. Needs computed
      UMAA/RHOMAA curves (equation engine for now; a dedicated module is a good next
      increment if you want it).

**Audit quick fixes** (from the full senior audit — see AUDIT-2026-07-20.md and
ROADMAP §4b for the 35-finding backlog):

- [x] **Pay summary change**: with a PERM cutoff active, samples with **missing PERM
      now FAIL the cutoff** (they silently passed before). Re-run a pay summary on a
      well with patchy PERM — net pay may legitimately decrease. Tell me if you'd
      rather missing-PERM samples pass (Geolog's default behavior differs by setup).
- [ ] **LAS import**: the file's own ~W NULL declaration is now honored (deliveries
      using -99999 etc. no longer import sentinels as data), and **multi-word well
      names survive** ("BALAM SOUTH-01" no longer truncates to "SOUTH-01"). Re-import
      one such file and check the Wells pane name.
- [ ] **Depth scale presets are now TRUE ratios** (1:200 = 1 m of well per 5 mm of
      screen at standard DPI). They were ~39x too compressed before, so 1:200 will
      look much more stretched than you're used to — the numbers are honest now.
- [ ] **Tops editor**: adding a top with an existing name is an overwrite; Ctrl+Z now
      restores the previous depth instead of deleting the top.
- [x] Case-insensitive computed-curve lookup (lowercase equation outputs now resolve).

## P2-f+ — D-N chartbook overlay (2026-07-20 #13)

Digitized from the Schlumberger 2013 chartbook you sent (Por-11 fresh / Por-12 salt,
extracted from the PDF's vector artwork — graduation-dash positions, not eyeballed;
calcite identity check rms 0.13 pu, both charts' worked examples reproduce).

- [x] **Crossplot Properties → Overlays → D-N chart**: pick _Fresh mud (Por-11)_ on an
      NPHI-RHOB crossplot → quartz/calcite/dolomite curves appear with porosity
      graduation dots + labels every 5 pu, dashed iso-porosity connectors, and curve
      names written along the lines. Compare against your paper chartbook page 225.
- [ ] **A real Mahakam sand interval** should plot on/left of the quartz sandstone line
      (shale pulls points right/down toward higher NPHI). Crossplot porosity read off
      the graduations should match your PHIE within ~1-2 pu in clean sand.
- [x] **Salt variant** (Por-12) shifts the curves left at high porosity — only relevant
      if you ever work salt-mud wells; check it renders and the graduations differ from
      Fresh.
- [x] **Zoom/pan**: the overlay must stay registered to the data under Ctrl+wheel zoom
      (it's drawn in data space). Also check the flipped orientation (X=RHOB, Y=NPHI).
- [ ] **Gating**: on a GR-RHOB plot or with a log axis the overlay silently stays off
      (chart geometry only means something on linear NPHI-RHOB).
- [x] **Note**: the chartbook draws its dolomite curve for ρma **2.85** (validated
      against the chart's own graduation ticks), while the _Matrix points_ overlay keeps
      the textbook single point at 2.87 — so Dol point and Dol curve start won't
      coincide exactly. Tell me if you'd rather I move the matrix point to 2.85.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [x] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [x] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [x] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [x] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [x] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [x] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
      Windows scaling): the desktop app now launches WebView2 with `--enable-lcd-text`,
      which forces ClearType subpixel antialiasing on GPU-composited panels (dockview
      layers otherwise fall back to fuzzy grayscale smoothing). **Needs the `npm run tauri
dev` restart** (config change). Look closely at ribbon/dialog text afterward — if it
      still reads soft, next steps are a base-size bump 12→13px and/or semibold.
- [ ] **T-S triangle now appears** (your "not showing (?)"): the triangle is drawn on
      VSH (0–1) vs PHIT axes — before, ticking it on the default NPHI-RHOB crossplot put
      every line off-scale, so nothing visibly happened. Now ticking **T-S triangle**
      auto-switches the X/Y axes to the well's VSH/PHIT curves (status bar tells you), and
      if the well has no VSH/porosity curves yet it says to run those modules first.
      Check: tick it on a fresh crossplot → axes flip, triangle + drag handles visible.

## P1-a — Interaction safety batch (2026-07-19 #3)

- [x] **Right-click lockdown**: right-click anywhere that has no SandiBumi menu (ribbon,
      buttons, tables, empty space) → **nothing** appears (the WebView menu with its
      dangerous Refresh is gone). Panel backgrounds still show our own menus; right-click
      inside a text box still shows the normal cut/copy/paste menu.
- [ ] **Reload guard**: press **F5** or **Ctrl+R** → a blocking confirm appears instead of
      an instant refresh; Cancel keeps everything, Reload restarts the workspace. Alt+←/→
      and the mouse back/forward side-buttons do nothing.
- [x] **Double-click-to-edit numbers** (app-wide): single-click any numeric parameter
      field (module dialogs, plot properties, SandiMin, zones…) → it focuses with a dashed
      outline but typing/arrows/wheel change **nothing**; **double-click** → solid outline,
      value selected, editing works. Tab-into-field still edits directly (deliberate).
      Scrolling a dialog with the wheel can no longer spin a value.
- [x] **Workflow Builder is a pane**: Petrophysics → Workflow… now opens a docked
      **Workflow Builder** pane (tab, movable/floatable like any panel) instead of a popup.
      No more losing a half-built chain to a stray click; it survives layout changes and
      reopens via the ＋ panel menu too. Run/cancel/progress unchanged; closing the pane
      mid-run cancels the chain.

## P1-b — Crash safe-mode, autosave, unsaved markers (2026-07-19 #3)

- [x] **Autosave**: the workspace (panes, arrangement, active well, every log view's
      layout) autosaves every 10 seconds. Nothing to click — just know it's there.
- [x] **Crash recovery**: if the app dies abnormally (crash, force-kill, power loss),
      the next launch shows a choice **before** anything loads: _Restore autosaved
      workspace_ (everything back as it was moments before the exit) or _Start in Safe
      Mode_ (clean default layout; the autosaved workspace is stashed as a "Recovered …"
      session under Open Session, so nothing is lost). To test without crashing for real:
      end the task from Task Manager while the app is open, then relaunch.
- [x] **Normal restart is less lossy now**: on a clean exit + relaunch, the app also
      brings back the **active well** and each log view's **layout/track state** (before,
      only the pane arrangement survived).
- [ ] **Unsaved markers**: edit a log view (track widths, properties, curve visibility)
      → its tab shows **●** and the QAT Save-Session button gets a red dot. **Save
      Layout** clears that panel's ●; **Save Session** clears everything. The dot means
      "not in a named save yet" — the crash autosave protects you regardless.

## P1-c — Log sets: versioning, provenance, catalog search (2026-07-19 #3)

- [ ] **Never overwrite**: every module dialog now has an **Output set** field (default
      INTERP; type any name — FINAL, TEST, …). Run a module, then re-run it with different
      parameters: the Curve Catalog's "Log sets" section shows **v1 AND v2** — the old
      run's values are kept, not destroyed. Plots/log views show the latest (v2).
- [ ] **Restore a version**: in Inspector → Curve Catalog, click **Restore** on v1 → all
      open log views and plots flip back to the v1 curves. Restore v2 to return.
- [ ] **Per-curve provenance**: the catalog now lists every computed curve's **set + version,
      module, and timestamp** (hover a set row for the exact parameters and input curves
      it was run with). Answering "where did this VSH come from?" is now one glance.
- [ ] **Catalog search/filter/sort**: one search box matches mnemonic, set, module, unit,
      or date; click any column header (Mnemonic, Set, When, n, Min, Max, Mean…) to sort,
      click again to reverse. Statistics (n/min/max/mean) shown per computed curve.
- [ ] **One version per chain run**: the Workflow Builder also has an Output set field —
      a whole chain run (VSH → porosity → Sw) lands as ONE version, not one per step.
- [ ] **Prune old versions**: Delete on a set version (two clicks — it asks "Confirm
      delete") removes only that version's history; current curves are never touched.
      Equation runs land in set EQUATION, ML in ML, SandiMin in SANDIMIN, automatically.
- [ ] **Input set** (the other half of set in/out): run VSH into Output set **FINAL**,
      then re-run with different parameters into **INTERP** (current values are now
      INTERP's). Open a module that consumes VSH (e.g. sw_indo), set **Input set =
      FINAL** → the run uses FINAL's VSH, not the current one. Blank Input set = normal
      behavior (latest values). Works in the Workflow Builder too; curves the input set
      never wrote (GR, RHOB…) still come from the usual sources.

## P2-a — Tops-style imports (2026-07-19 #4)

- [ ] **Import Tops…** (Data tab): pick a CSV or TXT tops file. With a WELL column
      (WELL/WELLNAME/UWI…) every matching project well gets its tops in one import —
      names match case-insensitively, unmatched names are reported in the status bar.
      Without a WELL column the tops land in the selected well. Columns understood:
      TOP/MARKER/SURFACE/FORMATION/HORIZON + DEPTH/MD/TOP_MD; also bare headerless
      "NAME DEPTH" text lines. Delimiters auto-detected (comma / semicolon / tab /
      spaces). Re-import updates depths but keeps colors you've set.
- [ ] **Import Aux…** (Data tab): petrography, XRD, or perforation data for the
      selected well (or a custom-named dataset). Needs a TOP/DEPTH column; a
      BASE/TO column makes rows intervals (perforations); every other column becomes
      an item — numbers (mineral %, grain size) and text (status, remarks) both kept.
      Re-importing a dataset replaces only that dataset for that well.
- [ ] **View it**: Data → DB Inspector → table "Aux Data" shows the imported rows
      per well (read-only — re-import the file to change values). Tops appear
      immediately in the Wells & Tops pane and all log views/correlation.

## P2-f — Crossplot v2 (2026-07-20 #12)

- [x] **Properties dialog**: double-click or right-click the crossplot (or ⚙ Properties)
      → sectioned dialog (Plot / Axes / Z color / Regression / Overlays). The old
      always-visible properties row is gone; the toolbar is just X/Y/Color/Zone.
- [x] **Marginal histograms + percentiles**: enable marginals on NPHI-RHOB — X histogram
      on top, Y histogram on the right, aligned with the axes (RHOB's inverted axis
      included). Percentiles `25, 75` draw dashed reference lines on both axes.
- [x] **Regression options**: on a PHIE-vs-PERM cloud try Power + RMA — the fit line
      must be straight on log axes and curved on linear ones, equation + R² + method
      tag shown top-left. Compare Y-on-X vs RMA slope on a noisy cloud (RMA steeper).
- [x] **Log-safe Z coloring**: color by PERM with "Log Z scale" + Viridis — low and high
      decades must stay distinguishable (rainbow + linear crams everything in one hue);
      the color bar is labeled "(log)".
- [x] **Plot size**: set Fixed 500×400 — the plot stops stretching with the pane
      (consistent exported figures). "Fill panel" restores the old behavior.
- [x] **Universal defaults**: Qtz/Cal/Dol matrix points no longer appear on NPHI-RHOB
      unless ticked in Properties; Color has a "— None —" option (custom point color
      applies); the pick rows + drag handle can be hidden ("Show parameter pickers" —
      still ON by default so your drag-to-set-shale-point workflow is unchanged).

## P2-e — Histogram v2 (2026-07-20 #11)

- [x] **Properties dialog**: double-click or right-click the histogram plot (or the ⚙
      Properties button) → one dialog holds display mode (bars/line), bins, normalize,
      cumulative overlay, box plot, color, percentiles, statistics placement, and the
      parameter-picker toggle. When zoomed, the first double-click resets the zoom, the
      next one opens properties.
- [x] **Box plot + cumulative overlay together**: enable both on a GR histogram — the
      P25–P75 box with P50 line and P5/P95 whiskers sits under the marker labels, and
      the cumulative % curve (secondary color, % labels on the right edge) tracks the
      bars. Zoom in with Ctrl+wheel: box and whiskers follow the axis.
- [x] **User percentiles**: type `10, 90` in Properties → P10/P90 marker lines on the
      plot and removable chips above it (click a chip to drop that percentile). Values
      must match what you'd read off the cumulative curve.
- [x] **Statistics inside the plot**: set Statistics → "Inside the plot" (chips hide) or
      "Both" — the in-plot block shows the active stats incl. new Min/Max. Check it in a
      dark theme too (block background must follow the theme).
- [x] **Universal by default**: a fresh histogram opens with NO Pick A/B rows and clicking
      the plot does nothing — enable "Show parameter pickers" in Properties to get the
      GR_MA/GR_SH picking workflow back. Your saved bar color / percentiles / etc. must
      survive closing and reopening the panel.

## P2-d — Log-view layout interaction (2026-07-19 #10)

- [x] **Collapsible track headers**: ▤ in the log-view toolbar cycles full → compact
      (curve names as inline chips, no scale lines) → titles only. Headers also cap at
      ~a third of the pane and scroll inside, so a 15-curve track can't eat the screen.
      Try it on your densest layout.
- [x] **Move/copy curves between tracks**: drag a curve name from one track header onto
      another track's header — the curve MOVES there (its color/scale/fill travel with
      it). Hold **Ctrl** while dropping to COPY instead (e.g. overlay NPHI on the GR
      track). Ctrl+Z undoes either.
- [x] **Track borders**: ▦ in the toolbar — solid / dashed / none, width 1–4 px, theme
      color (follows light/dark) or a custom color. Default is a thin solid separator
      at every track boundary; check it looks right in dark themes too.
- [x] **Readout follows ONE track now**: hovering shows only the curves of the track
      under the cursor (not all 15). CLICK a track to lock the readout to it (header
      highlights, click again to release) — then you can run the cursor over the whole
      layout while reading just that track's values.
- [x] **Right-click log editing**: right-click on a track → "Edit CURVE…" for each of
      its curves. Ops: **Wireline shift** (whole-curve depth shift, resampled onto its
      own grid — NaN where it slides past the logged interval), **Set constant**,
      **Blank (erase)**, **Interpolate across** (bridge a bad interval linearly),
      **Scale a·v + b** (recalibration). Works on raw (GR/RHOB…), computed, and
      imported generic-store curves alike; every apply is ONE Ctrl+Z entry that
      restores the previous samples bit-exactly, and lands in the History panel.
      Suggested check: blank a washout interval on RHOB, interpolate across it,
      then Ctrl+Z twice — the original curve must come back exactly.

## P2-c — Well pin rework + multi-select (2026-07-19 #9)

- [x] **Pin is now a mode, not a lock.** 📌 ON (default): clicking a well in Wells &
      Tops moves EVERY log view and plot to it — the old behavior. 📌 OFF: each view
      keeps the well it's showing; only the panel you're working in (the active tab)
      follows your clicks. Open two log views, turn the pin off, activate the second
      view, click different wells — only the second view changes. That's the
      side-by-side compare workflow.
- [x] **The old lock is gone**: no more "Active well is locked" blocking when you
      click other wells, and no more weird interaction with a second wells pane.
- [x] **Multi-select**: Ctrl-click wells to build a selection (highlighted with an
      accent edge, count shown in the Wells label), Shift-click for a range,
      ⇄ inverts within the visible list, plain click clears it. Then open any batch
      dialog (module run, Workflow Builder, Multimin, ML, Monte Carlo, Cutoffs &
      Summary) — the multi-selected wells come pre-ticked instead of just the active
      well.

## P2-b — Petrel-style tops editor + autocorrelation (2026-07-19 #4/#13)

- [ ] **Tops lines in the log view**: every log view now draws the well's tops as
      colored labeled lines across all tracks (like the correlation view). They track
      pan/zoom exactly and repaint on theme change.
- [ ] **🏷 edit mode** (log view toolbar): toggle it on, then — **click** an empty
      depth to add a top (name/depth/color dialog, name auto-uppercased); **drag** a
      line to move it (dashed preview while dragging); **double-click** a line to
      rename, change color, or delete. Mouse-wheel zoom still works while editing.
      Everything is undoable (Ctrl+Z) and instantly visible in Wells & Tops, other
      log views, and correlation.
- [ ] **Crossing warnings**: after any pick/move, SandiBumi compares this well's top
      order with every other well. If a pair is reversed vs the majority (e.g. TOP_B
      above TOP_A here but below it elsewhere), a ⚠ warning appears in the status bar
      naming the pair and the vote (e.g. "below it in 4 of 5 other wells").
- [ ] **Autocorrelate…** (Data tab): pick a top in the selected (source) well, choose
      the log (GR default), pattern window ±m and search range ±m — SandiBumi slides
      the source log shape over each target well (active group) and proposes the
      best-match depth with its correlation coefficient r. Strong matches (r ≥ 0.7)
      come pre-ticked; weak ones are dimmed for your judgment. **Apply** writes the
      ticked picks as ONE undoable batch. Try it on a marker you know — e.g. pick an
      MFS on GR in one Balam well and propagate to the rest, then check r values
      against your hand picks.

Issues you marked `[x]` that need real work (all in ROADMAP §4, P1/P2): well-pin
semantics rework, right-click lockdown (accidental refresh), TVD depth scale UI.
Everything you marked `[o]` has been cleared out of this file.

## Theme switch repaints everything immediately (2026-07-19)

- [x] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [x] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

## SandiMin — Geolog-parity mineral solver (2026-07-19, v2)

Rebuilt to Geolog Multimin / IP Mineral Solver conventions (spec extracted from your
Geolog-V14 helpset + IP2018 install → `docs/multimin_geolog_spec.md`, `docs/multimin_ip_spec.md`).

- [ ] **Advance → SandiMin…** now shows the full IP mineral list, grouped: 12 minerals (Calcite,
      Quartz, Dolomite, Orthoclase, Albite, Anhydrite, Halite, Gypsum, Pyrite, Siderite, Muscovite,
      Biotite), 6 clays (Glauconite, Kaolinite, Chlorite, Illite, Montmorillonite, Clay — each with
      an editable **CEC**), and 7 zone-typed fluids (Water Sxo / Water Sw / BoundWater / Oil Sxo /
      Oil Sw / Gas Sxo / Gas Sw; "flushed"/"unflushed" badges). Defaults: Quartz, Illite,
      Water Sxo, Water Sw.
- [ ] **Input logs**: 16 tools — Density, Neutron, Sonic, Total GR on by default; PEF, U, spectral
      Th/K/U, Vp, Vs, EPT, EATT, Sigma optional; **CT (Unflushed Conductivity, from RES_DEEP)** on
      by default and **CXO (from RXO)** optional — CT/CXO take a RESISTIVITY mnemonic; the backend
      converts to conductivity (dual-water linear: Ct^(1/w) row, w = 0.75m + 0.25n). Their σ is
      auto (0.03·C^(1/w)) unless you type one. **+ Add user-defined input** adds a custom log with
      its own endpoint column (default σ 0.015, Geolog's user-defined default).
- [ ] **Endpoints matrix**: editable per component×tool; unflushed-zone fluid cells show "—" for
      nuclear tools (only CT sees them — Geolog convention); CT/CXO cells show "auto"; per-row
      **Max** bound (fluids default 0.5, Geolog's cap).
- [ ] **Fluid properties** panel (visible when CT/CXO on): Rw@temp, Rmf@temp, formation temp, m, n,
      mud type. The preview line shows the computed w, Cw, Cmf, Cbw, α(x/u) and auto CT/CXO σ —
      sanity-check Cw against your Pickett Rw (Cw = 1/Rw@FT, mho/m).
- [ ] **Run** on a Balam well with RHOB+NPHI+DT+GR+RES*DEEP: writes VOL*\* per component +
      MM_PHIE, MM_PHIT, MM_SWE, MM_SWT (+ MM_SXOT, MM_MOVEDHC when both zones present),
      MM_VSH (clays + bound water), MM_RECON. Check: **Σ(minerals + unflushed fluids) ≈ 1**,
      **MM_SWT is sensible vs your sw_indo/RtC runs** (this is the new resistivity coupling —
      "resistivity convert to ct and cxo" as requested), and MM_RECON spikes where the model fails.
- [ ] Add **BoundWater** with Illite selected: VOL_BOUNDWATER should track ≈ 0.18×VOL_ILLITE at
      ~150°F (the Geolog dual-water bound-water constraint, k = 96·CEC·ρ/(T°C+298)).
- [ ] Add **Oil Sxo + Oil Sw** with CXO available: SXOT ≥ SWT in water-based mud (WATER MUD
      constraint) and MM_MOVEDHC = unflushed HC − flushed HC ≥ 0 across invaded pay.
- [ ] Requested upgrade (ROADMAP §4 P3): optional **nonlinear Sw equation iterated to
      convergence** inside the solve loop.

## ML suite (2026-07-19)

- [x] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [x] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [x] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [x] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [x] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [x] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [x] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [x] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
      Carlo, and Field Dashboard now sit in one row inside the Batch group.

## FACIES block track (2026-07-19)

- [ ] **Facies layout renders colored blocks.** Run Electrofacies on a well (Petrophysics →
      Electrofacies), then pick the new built-in "Facies" layout in the ribbon layout picker:
      the FACIES track should show solid colored blocks (same colors as the crossplot's
      categorical Z-coloring), with gaps where FACIES is missing. The track header shows a
      striped swatch and "class blocks" instead of a min/max scale.
- [ ] **Blocks survive pan/zoom and well switching**, and the header swatch toggles the whole
      track's visibility like any other curve.
- [ ] **Any discrete curve can be block-rendered.** Layout Properties → a curve's Fill
      dropdown now has "Facies blocks" — try it on FLAG_PAY in a custom layout.
- [ ] **Composite export shows the blocks.** Export a composite (SVG or PDF) with the Facies
      layout: the FACIES track should print as colored rectangles at true scale.

## Electrofacies — k-means (Phase 10 increment 1, 2026-07-18)

- [x] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [x] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [x] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [x] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
      curves** from the chosen result percentiles.

## Phase 8.5 — your method suite in core (remaining validations)

- [ ] **SSC — Sand-Silt-Clay (Advance tab)**: run on an LQR-style well with
      GRN + RHOB + NPHI (sandstone units). Check VSAND/VSILT/VDCL/VWCL, PHIT/PHIE/PHIFF,
      CBW/CWSH/BW, SWIRR_T/SWIRR_EFF and the `*_GR` GR-equivalent volumes against your
      Geolog run. Defaults are the LQR `.info` values (wet clay 2.3/0.6, dry clay 2.71,
      wet silt NPHI 0.3, DCLF_SI 0.1). Two deliberate deviations, flag if they matter:
      (1) `RANNORMAL(SWIRR_MIN·PHIT, 0.005)` is deterministic here; (2) the Loglan's
      NPHIMA limit 0.5–5 (a copy-paste of the RHOMA limit) is corrected to 0–1.
- [ ] **SSPW (Advance tab)**: the Loglan exec body wasn't on disk, so the
      arithmetic (PHIT from VSH-mixed dry matrix, CBW = VSH·VOL_CBW_SH,
      CAPBW = VSH·(PHIT_SH − VOL_CBW_SH), PHIE = PHIT − CBW, PHIFF, SWIRR floor) is
      **reconstructed from the spec — please validate against your Geolog "LAS PHIT
      PHIE" exports** and tell me any systematic difference; I'll adjust the equations.

## Phase 8b — report generator (2026-07-18)

- [x] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [x] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [x] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [x] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [x] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [x] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

## Module-panel cleanup, Help tool, bulk Processing report, responsive resize (2026-07-21)

Five asks from your VSH-panel screenshot (SandiMin deferred for later review).

- [ ] **Module form no longer lists per-well results**: run a module (e.g. VSH from
      Gamma Ray) — the form now shows one summary line ("All N well(s) computed. Per-well
      details are in the Processing panel." or "…N need attention…") and the Processing
      panel comes forward. The old `✓ well: samples → curves` list is gone from the form.
- [ ] **Per-well detail lives in Processing → details**: expand a job's **▸ details** —
      running wells show individually; the narration paragraph that used to sit at the top
      of the module form is gone (it moved to Help, below).
- [ ] **Bulk failure report**: when many wells fail the SAME way, Processing → details shows
      **one card per reason** — "N well(s) failed — <message>", the well list (first 12 +
      "…(+K more)"), and a "→ what to do" advice line — instead of one row per well.
- [ ] **Help (?) tool**: click the **?** in the top quick-access bar (or right-click any
      panel → **Help for this panel…**). A guide opens for whatever panel is active — a
      module pane shows that method's description; other panels show a short blurb. (This is
      the placeholder that will later link to the full illustrated help library.)
- [ ] **Ribbon dropdowns still work**: open **Petrophysics → VSH/Porosity/Saturation**, and
      **Data → Import Logs/Import Data/Tools**, and **Project → Recent** — each menu must drop
      fully below the ribbon (this was a regression the review caught and fixed).
- [ ] **Resize the whole window**: the content panes (log views / plots / inspector) reflow to
      fill; **Wells & Tops, Processing, and Performance keep their width** (they're a fixed
      sidebar now). Try both wider and narrower — nothing should get clipped or leave dead space,
      and the ribbon stays reachable (scrolls if very narrow).
- [ ] **Close panes without the sidebar growing**: close a plot/log view — the freed space goes
      to the other content, NOT to the sidebar. Close *everything* down to just the sidebar and a
      blank **Workspace** pane fills the rest (rather than the sidebar stretching); open any log
      view/plot and the blank pane disappears again.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
