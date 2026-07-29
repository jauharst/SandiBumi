# DIALOG-vs-PANE architecture + theme compliance (SandiBumi, D:\XX. SandiBumi)

## Current state
== 1. DIALOG INVENTORY ==
All popups go through one helper: openModal() in src\ui\modal.ts (single #modal-root at index.html:411; NON-blocking pointer-transparent scrim per styles.css ~1326; draggable title bar; Escape/close-button close; root.innerHTML="" on every open, so only ONE dialog can exist and opening a second silently destroys the first without running its close(), leaking its document keydown listener). No showModal()/<dialog> anywhere; only other fixed overlays are contextMenu, dock-add-menu, guard-confirm scrim, and a hidden print iframe in plotExport.ts:42.

Standalone tool dialogs (module, ~size, content, complexity):
- moduleDialog.ts (262 ln, 560px) openModuleDialog — auto-generated per-module param dialog (the reference suite .info model): well checklist + curve selects + option dropdowns + validated numeric params + Run + result lines. Form+run. Auto-closes 900 ms after all-wells success (moduleDialog.ts:254). THE "tools" popup — launched from ribbon.ts:416 for every calc module.
- multiminDialog.ts (502 ln, 940px) — SandiMin mineral solver: 27-component library, editable endpoint matrix, 16 input logs with sigma, fluid calc. Form + large editable tables. Captures close but never calls it (line 501 `void close`).
- mlDialog.ts (452 ln, 660px) — ML catalog (regression/classification/clustering): algo pick, params, train/apply well lists, results. Never closes itself.
- monteCarloDialog.ts (514 ln, 640px) — MC uncertainty over workflow chains: per-param distributions, percentiles, zone results table. Never closes itself. Has a proper cssVar() helper (line 395) — good precedent.
- reportDialog.ts (400 ln, 620px) — report deliverable: layout/method rows, SVG page preview, PDF/PNG/batch export.
- compositeDialog.ts (266 ln, 560px) — composite log print/export: layout, print scale, page size, depth window, SVG preview.
- summaryDialog.ts (121 ln, 900px) — cutoffs & pay summary: cutoffs form -> per-well/zone stats table, writes FLAG_ curves. Simplest tool dialog.
- zonesDialog.ts (181 ln, 620px) — zone manager + per-zone param overrides for the selected well.
- autoCorrDialog.ts (211 ln, 560px) — top autocorrelation proposals table (corr coefficient, tick/untick), apply = one undoable batch, then close (line 210).
- curveEditDialog.ts (133 ln, 420px) — per-curve edit ops (shift/set/blank/interpolate/scale); launched from log-view right-click with clickedDepth; closes on apply.
- layoutPropsDialog.ts (317 ln, 880px) — track list + curve style table; edits a structuredClone, hands the result back via onApply (true modal-completion contract).
- wellGroups.ts (560px) openWellGroupManager — group CRUD + membership.
- workflowDialog.ts (545 ln) — ALREADY PORTED to a pane: buildWorkflowContent() returns {el, dispose}; header comment records the rationale ("popup was too easy to dismiss mid-build", Jauhar 2026-07-19). This is the porting template.

Inline mini-modals: ribbon.ts x8 (Save Layout As :489, Save Session As :571, Open Session picker :618, Shift Core :785, Import SCAL :892, Import Aux Data :1006, Import Deviation :1083, Well Header :1145 — all small close-on-complete forms); topsEditor.ts New top :241 / Edit top :318; per-plot property popups: crossplotPanel.ts:1207 (Crossplot Properties), histogramPanel.ts:606 (Histogram Properties), logViewPanel.ts:367 (Track borders), plotCommon.ts:157 (Save Template).

== 2. PANE CREATION TODAY ==
src\ui\workspace.ts: Workspace class wraps DockviewComponent (dockview-core, custom theme "dockview-theme-sandibumi"). Panel types are string keys in buildRenderer() switch (line 260): logview, wellsTops, inspector, dbInspector, sqlQuery, history, dashboard, workflow, histogram, crossplot, pickett, correlation (+ unknown fallback). Generic adapter EXISTS: class DomPanel (line 47) — takes className + fill(host, params) returning optional cleanup — plus the async PlotContent pattern {el, dispose?, getState?} (plotCommon.ts:11) with generation-counter/closed-flag handling (createPlot, dashboard, workflow cases). Singletons focus-or-move via openSingleton(); every group has a "+" add-panel menu (showAddPanelMenu, line 198) and a per-type right-click context menu (contextItemsFor). workflow and compositeDialog are lazy-loaded with dynamic import() to avoid ribbon<->workspace cycles.
Lifecycle facts a port must handle: (a) layout auto-persists to localStorage "sandibumi.workspace" and to named SessionSnapshots — panels are recreated on boot from only {id, component, title}; dockview addPanel params are NOT used anywhere, and panel-internal state is not serialized (log-view layouts are special-cased via SessionSnapshot.logViewLayouts). A ported tool pane will re-init empty after restart unless given the same treatment. (b) moduleDialog is parameterized by ModuleSpec — porting it needs panel params carried through toJSON/fromJSON and a singleton-per-module policy. (c) Dialogs snapshot the selected well at open; panes are expected to subscribe to appState.selectedWell/dataVersion/wellPinned (see createPlot's follow/pin logic). (d) Modal-completion contracts (layoutPropsDialog onApply-on-clone, curveEdit/autoCorr/topsEditor close-on-apply) have no pane equivalent — panes persist, so these must become live-apply or keep Apply buttons without close. (e) New panel types must be added in three places: buildRenderer switch, showAddPanelMenu entries, contextItemsFor headings.

== 3. THEME COMPLIANCE ==
Contract = exactly 15 vars in styles.css :root (lines 16-32): --bg-app, --bg-panel, --bg-panel-alt, --bg-hover, --border, --border-strong, --text, --text-dim, --accent, --accent-dim, --accent-soft, --accent2, --accent2-soft, --warn, --track-hd; 7 theme blocks (light default, dark + prefers-color-scheme fallback, pertamina, halliburton, schlumberger, lapi-itb, white); dockview chrome mapped via --dv-* (lines ~475-498). theme.ts only flips data-theme.
Violations found (worst first):
A. PHANTOM VARIABLES — var() names outside the contract, never defined in any theme, so their hard-coded fallbacks always render in every theme: var(--danger, #c0392b) x4 (styles.css 1221-1222, 1277-1278 — inspector eq-status/danger), var(--muted, #8a8170 / #6f6858 / #6f6857) x6 (2525, 2549, 2571, 2575, 2616, 2673 — Workflow Builder + Monte Carlo sections), var(--panel-2, #f4f4f4) (2467 .composite-preview — light grey panel even in dark theme), var(--surface-2, rgba(0,0,0,0.03)) x2 (2537), var(--bg-subtle, transparent) (2265). The Workflow Builder / Monte Carlo / composite-preview CSS was written against an imagined contract and is effectively hard-coded.
B. .cursor-readout (styles.css 904-926): background rgba(20,15,8,0.78), color #f2ebdc, highlight #ffd9a0 — light-theme-branded overlay baked in for all themes.
C. .workflow-invalid (2559-2560): border-color #c0392b !important + rgba(192,57,43,0.08) — bypasses --warn.
D. TS chrome colors: crossplotPanel.ts:844-845 and pickettPanel.ts:152-153 pickRow buttons hard-code "#b5651d"/"#5f7350" (copies of the LIGHT theme accent/accent2 — wrong under dark/brand themes); histogramPanel.ts:438 same pair; placeholder text ctx.fillStyle "#888" (crossplotPanel.ts:889, histogramPanel.ts:489); logViewPanel.ts:489 crosshair "rgba(0,0,0,0.65)" (near-invisible on dark) and :540 "#999". Fix = read via plotCanvas.ts plotColors()/getComputedStyle like monteCarloDialog.ts:395 already does.
E. color:#fff on accent-background buttons ~10 places (1207, 1443, 1576, 1581, 1703, 2269, 2338, 2817, 2830, 3136, 3155) — assumes every theme's accent is dark; moderate risk for future light-accent brand themes.
Legitimate (leave alone): plotCanvas.ts:32-38 var reads with fallbacks; Tableau-10 categorical + facies palettes (plotCanvas.ts:581+, data colors); reportDialog.ts:356 #ffffff PNG export background (print deliverable); topsEditor DEFAULT_COLOR #e2b93d and layoutPropsDialog:282 #b5651d (persisted data colors); box-shadow rgba(0,0,0,x) shadows.

## Gaps
PORTING ORDER (dialog -> pane), following the buildWorkflowContent precedent:
EASY (self-contained form+results, never self-close, zero context args beyond setStatus — wrap in DomPanel, add to buildRenderer/add-menu/context-menu):
1. summaryDialog (Cutoffs & Pay Summary) — smallest, pure form+table; already 900px wide, pane-shaped.
2. mlDialog — same shape.
3. monteCarloDialog — same shape; already theme-aware via cssVar helper.
4. multiminDialog — large but fully self-contained (its close() is already dead code); its 940px matrix actually wants pane space.
MEDIUM (need selected-well subscription instead of well-at-open snapshot; reuse createPlot's follow/pin + generation-counter pattern):
5. zonesDialog (takes well arg -> subscribe to appState.selectedWell).
6. wellGroups manager.
7. compositeDialog and reportDialog — preview-centric, benefit most visually; async SVG preview needs the closed-flag lifecycle.
HARD (infrastructure or semantics):
8. moduleDialog — highest user value ("the tools") but parameterized by ModuleSpec: requires dockview addPanel params plumbed through createComponent + toJSON restore (params are currently unused anywhere), a singleton-per-module policy, and removal of the auto-close-on-success timer. Do after 1-4 prove the pattern.
9. autoCorrDialog — apply-batch + undo completion semantics; needs a redesigned "review then apply" pane flow.
RECOMMEND KEEPING AS POPUPS (porting hurts UX): layoutPropsDialog (clone + onApply contract), curveEditDialog and topsEditor New/Edit top (click-position context micro-forms), the 8 ribbon save/open/import mini-forms, per-plot Properties/Save-Template popups (alternative: convert Properties into in-pane side drawers). If any popups remain, fix modal.ts's single-root replacement leak (second openModal destroys the first without its close(), stranding its keydown listener).
PREREQ INFRA: (a) extend SessionSnapshot (workspace.ts) so tool panes can persist internal state like logViewLayouts does, or explicitly accept re-init-empty; (b) each new pane type = 3 registration points (buildRenderer switch, showAddPanelMenu, contextItemsFor); (c) use dynamic import() from workspace.ts to avoid ribbon<->workspace cycles.
THEME FIX LIST (independent, low-risk, do first): define --danger/--muted/--panel-2/--surface-2/--bg-subtle in all 7 theme blocks OR rewrite those ~15 declarations onto the 15-var contract (--warn/--text-dim/--bg-panel-alt cover them); re-skin .cursor-readout and .workflow-invalid onto vars; replace hard-coded "#b5651d"/"#5f7350"/"#888"/"#999"/rgba(0,0,0,0.65) in crossplotPanel/pickettPanel/histogramPanel/logViewPanel with plotColors() reads; audit color:#fff accent buttons if a light-accent brand theme is ever added.

## Key files
- D:\XX. SandiBumi\src\ui\modal.ts
- D:\XX. SandiBumi\src\ui\workspace.ts
- D:\XX. SandiBumi\src\ui\workflowDialog.ts
- D:\XX. SandiBumi\src\ui\moduleDialog.ts
- D:\XX. SandiBumi\src\ui\multiminDialog.ts
- D:\XX. SandiBumi\src\ui\mlDialog.ts
- D:\XX. SandiBumi\src\ui\monteCarloDialog.ts
- D:\XX. SandiBumi\src\ui\summaryDialog.ts
- D:\XX. SandiBumi\src\ui\reportDialog.ts
- D:\XX. SandiBumi\src\ui\compositeDialog.ts
- D:\XX. SandiBumi\src\ui\zonesDialog.ts
- D:\XX. SandiBumi\src\ui\autoCorrDialog.ts
- D:\XX. SandiBumi\src\ui\curveEditDialog.ts
- D:\XX. SandiBumi\src\ui\layoutPropsDialog.ts
- D:\XX. SandiBumi\src\ui\wellGroups.ts
- D:\XX. SandiBumi\src\ui\topsEditor.ts
- D:\XX. SandiBumi\src\ui\ribbon.ts
- D:\XX. SandiBumi\src\ui\plotCommon.ts
- D:\XX. SandiBumi\src\ui\plotCanvas.ts
- D:\XX. SandiBumi\src\ui\crossplotPanel.ts
- D:\XX. SandiBumi\src\ui\histogramPanel.ts
- D:\XX. SandiBumi\src\ui\pickettPanel.ts
- D:\XX. SandiBumi\src\ui\logViewPanel.ts
- D:\XX. SandiBumi\src\styles.css
- D:\XX. SandiBumi\src\theme.ts
- D:\XX. SandiBumi\index.html
