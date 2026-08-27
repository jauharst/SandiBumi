# Handoff: SandiBumi "Organic" reskin

## Overview
A redesign direction for SandiBumi (Tauri v2 + vanilla TypeScript petrophysics desktop app). The app's chrome — ribbon, dockable panel frames, dialogs, buttons, tags — takes on a warm, rounded "Organic" look; data surfaces (log tracks, grids, trees, tables) keep the existing professional density. Also includes recolored Halliburton and Schlumberger client skins that exercise the app's existing `[data-theme]` mechanism.

## About the Design Files
The files in this bundle are **design references created in HTML** — they show intended look and behavior, they are NOT production code to copy. The task is to **recreate these designs inside the existing SandiBumi codebase** (`src/styles.css` + the vanilla-TS UI in `src/ui/*`), using its established token layer and class names. Do not introduce a framework; the app is deliberately vanilla TS + CSS variables.

`SandiBumi UI Mockups.dc.html` is the design document (open it in the design tool where it was authored; it references a runtime not bundled here). `organic-tokens.css` carries the full token sheet the mockups consume (`--color-*` ramps, `--font-*`, `--radius-*`, `--shadow-*`). Read values from it rather than eyeballing.

## Fidelity
**High-fidelity.** Colors, radii, type sizes and spacing in the mockups are intentional. Recreate pixel-perfectly within the codebase's own component structure (ribbon.ts, workspace.ts/dockview, modal.ts, dashboardPanel.ts, etc.).

## The core move: token mapping
Everything in SandiBumi already flows from `:root` variables in `src/styles.css`. Most of this redesign is a token pass plus a small number of component-CSS changes.

| Mockup token (organic-tokens.css) | SandiBumi token | Value (default theme) |
|---|---|---|
| `--color-bg` | `--bg-app` | `#f5ead8` (cream ground) |
| `#fff` panel cards | `--bg-panel` | `#ffffff` |
| `--color-neutral-100` | `--bg-panel-alt` | warm neutral tint, ~`#efe9dc` |
| `--color-accent-100` | `--bg-hover` / `--accent-soft` | pale terracotta tint |
| `--color-neutral-200/300` | `--border` / `--border-strong` | warm hairlines |
| `--color-text` | `--text` | `#201e1d` |
| `--color-neutral-600` | `--text-dim` | — |
| `--color-accent` | `--accent` | `#c67139` terracotta |
| `--color-accent-700` | `--accent-dim` | dark terracotta (accent-ramp 700) |
| `--color-accent-2` | `--accent2` | `#7a8a5e` sage |
| `--color-accent-2-100` | `--accent2-soft` | pale sage |

Radius scale changes (`src/styles.css` non-color tokens):
- `--r-md` (buttons/inputs) → pill where the mock shows pills: ribbon tabs, primary/secondary buttons, segmented controls = `999px`
- `--r-lg` (floating surfaces) → `12px`; top-level panel cards = `12px`; dialogs = `16px`
- Keep `--r-sm` for dense inline controls unchanged.

Typography:

> **Superseded 2026-08-28, Jauhar's call** — *"change ALL this kind of font with more
> natural and professional font"*. The Caprasimo display face is retired. The four
> display surfaces below are unchanged as a LIST; what changed is the face that fills
> them: they now use **Figtree at weight 700** (wordmarks and the boot title at 800)
> instead of a separate display family. Every mention of Caprasimo further down this
> file and in the `.dc.html` mockups is the 2026-08-01 handoff as delivered, kept as
> the record — read the face from `organic-tokens.css`, which is the authority for
> values. The rest of the type rule (which surfaces, and the ban on display type in
> data) is unchanged and still binding.

- Display surfaces are ONLY: brand wordmark next to the logo, screen/dialog titles ("Field Dashboard", "VSH — Shale Volume", "Import LAS", "Report — …", start-screen wordmark), and KPI numerals.
- Body/UI face **Figtree** replaces Segoe UI in `--font-canvas` contexts where feasible; data grids stay 11–12.5px. Never use display type in data cells, axis labels or track headers.

Shadows: use soft warm shadows (see `--shadow-sm/md/lg` in organic-tokens.css) instead of the current `--el-*` grey rgba values on light themes.

## Screens / Views (option ids match badges in the mockup document)

### 1a — Main workspace
- Structure preserved from the real app: tabstrip (brand + 6 ribbon tabs) → ribbon body (icon+label tool groups with small-caps captions MODULES / INTERVALS / REPORTING / BATCH) → dock area (Wells & Tops 224px | log view flex | Inspector 206px) → status bar.
- Ribbon tabs are text pills; the active tab is a solid `--accent` pill with white text. Ribbon body is a white rounded (12px) strip inset 8–10px from the window edge on the cream ground.
- **Tight spacing is deliberate**: dock gap 7px, dock padding 8px 10px, panel header padding 6–7px 12px. Engineers on small screens are the audience; do not add air here.
- Panels are white cards, 1px `--border`, radius 12px — replaces dockview's default chrome (map through the existing `--dv-*` variable bridge in styles.css).
- Log view: mini toolbar right-aligned in the panel header (scale tag "1 : 500", zoom, track width, pin). Track headers 44px with curve name + range in curve color. Curve colors: GR terracotta-700, RT near-black text color, RHOB terracotta-700 / NPHI sage-700 dashed, VSH neutral fill from left, PHIE sage-800, SWE terracotta with pale terracotta hydrocarbon fill between SWE and right edge. Tops = 1.5px dashed accent-800 lines with white-pill right-aligned labels. Zone bands = sage-100 at 35% opacity.
- Status bar: white strip, sage status dot, well/group/step readouts, right-aligned undo label.

### 1b — Field dashboard
- Header row: Caprasimo title + neutral tag (group · well count) + right-aligned Export CSV (secondary) and Compute (primary pill).
- Cutoff controls in one rounded neutral-100 strip: VSH ≤ / PHIE ≥ / SWE ≤ / PERM ≥ inputs (74px), Flag and Metric segmented pills (active = accent pill).
- KPI cards row: accent-100 (Total net pay), accent-2-100 (Total HPV), two neutral-100; label 11px/700, value 27px Caprasimo.
- Grid: 11.5px tabular-nums, numeric columns right-aligned, top row highlighted accent-100, excluded zones greyed with a "no results excluded, never averaged as zero" footnote near the box plots.
- Box plots per zone: fill = ramp-200, stroke = ramp-700, median 2.2px ramp-800, whiskers 1.4px.

### 1c — Parameter picking (crossplot · histogram · Pickett)
- Three white panels + a 196px "Synchronized hover" side card. Plot backgrounds are neutral-100 rounded rects with white gridlines.
- Crossplot: points colored by GR in 3 steps (accent-400 / accent-700 / neutral-700), SLB-2013 sandstone/limestone D-N overlay lines, matrix & shale pick markers with bold labels, gradient GR legend, tag "Pick writes zone parameters".
- Histogram: accent-300 bars w/ accent-600 stroke, dashed percentile pick lines GR_MA (P5) and GR_SH (P95).
- Pickett: log-log, solid accent Sw=1 water line, dashed accent-400 Sw=0.5, points terracotta (Sw<0.5) vs sage; Rw/m/n tags below; hint "drag the water line → writes Rw, M".

### 1d — Module pane (VSH; pattern generalizes to every manifest-driven module)
- Header: 34px rounded icon chip (accent-100 bg) + Caprasimo title + "? Help" right.
- RUN ON scope: segmented pill Group / ★ Pinned / Selection / All + well-count tag (matches wellScope.ts semantics).
- 2-column grid of labeled fields (11px/700 uppercase labels; inputs rounded). Units to the right of numeric inputs.
- Sage-100 callout: zone overrides take precedence over whole-well defaults.
- Footer: primary "Run VSH", ghost "Preview one well", right-aligned last-run status.

### 1e — LAS import wizard
- Header: Caprasimo title + step pills (1 Files solid accent, 2 Mnemonics accent-100, 3 Review plain).
- Left rail 280px: file list (selected = accent-100 row; status tags OK sage / warnings outline), dashed drop zone, depth-unit/step/target-set summary.
- Right: mnemonic mapping table (LAS mnemonic mono, family → stored-as, unit, import tag); unmapped row highlighted accent-100 with "pick family ▾".
- Footer note: "Every import is versioned with provenance — re-importing never overwrites RAW."

### 1f — Report generator
- Left form: title input, composite layout + page·scale selects, editable methodology table (Parameter | Method | Remarks), footer buttons Render PDF (primary) / DOCX / PNG pages / Batch (n wells)… ghost.
- Right 380px preview rail on neutral-100: A4 cover thumbnail (logo, Caprasimo title, meta lines), page thumbnail strip (active = 2px accent border), caption of page plan.

### 1g — Start screen
- Left 340px on cream: 72px rounded logo, 34px Caprasimo wordmark, one-line description, New Project (primary) / Open Project (secondary) stacked pills, version + language footer.
- Right white sheet: RECENT PROJECTS rows (38px rounded db icon chip, name + meta, "last session" tag on the most recent), bottom neutral-100 tip card about sessions.

### 2a / 2b — Client skins (Halliburton, Schlumberger)
Recolor ONLY through the existing theme blocks in `src/styles.css` — the values there are already correct and were used verbatim:
- `:root[data-theme="halliburton"]`: `--accent:#e31b23`, `--accent-dim:#b3141b`, `--accent-soft:#fbdcde`, `--accent2:#3f4249` graphite, `--bg-app:#f3f3f4`.
- `:root[data-theme="schlumberger"]`: `--accent:#0033a0`, `--accent-dim:#00246f`, `--accent-soft:#d5e0f4`, `--accent2:#00a3e0`, `--bg-app:#eef1f7`.
Shape language (pills, 12px cards, Caprasimo titles) is theme-independent and must not change per skin. The mockups show the full ramps derived for each brand — if you add ramp steps (100–900) per theme, generate them in OKLCH at the same lightness steps as the Organic ramps in organic-tokens.css.

## Interactions & Behavior
- Hover: tinted fills from the accent ramp (`--accent-soft` / accent-100), never grey. Pressed = one ramp step darker (accent-600/700).
- Focus: `:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }` everywhere — replace any default ring.
- Transitions: keep the app's existing rule — only paint properties (background/border/color/shadow) through `--dur-fast/base` + `--ease`; never transform/size (dockview drags and canvas pans must stay instant).
- Synchronized hover across plots and depth readout behaves as today; visual = point highlight + side-card readout (1c).
- Disabled = 45% opacity (matches both systems).

## State Management
No new state. All screens map to existing panes/dialogs: ribbon.ts, workspace.ts, objectTree.ts, logViewPanel.ts, dashboardPanel.ts, crossplotPanel.ts, histogramPanel.ts, pickettPanel.ts, moduleDialog.ts, importSetDialog.ts, reportDialog.ts, plus the startup surface (bootOverlay.ts / startupNotice.ts).

## Design Tokens
Full sheet in `organic-tokens.css` (authoritative). Key values: ground `#f5ead8`, text `#201e1d`, accent `#c67139` (+100–900 OKLCH ramp), accent-2 `#7a8a5e` (+ramp), warm neutral ramp, `--font-heading` Caprasimo, `--font-body` Figtree, radius 16px base growing to pills, shadow-sm/md/lg tuned to the cream ground.
Minimum data-UI sizes used: 10px small-caps captions, 11.5px table cells, 12.5px control labels; `font-variant-numeric: tabular-nums` on all numeric columns.

## Assets
- `assets/logo-mark.svg`, `assets/logo-full.svg` — from the repo's `public/`. Note: they are traced SVGs with a baked-in cream background; the mockups round their corners (8–20px). A transparent-background logo would be an upgrade.
- Icons: the app's own 20×20 stroke icons (from index.html / ribbon.ts), stroke-width raised 1.5 → 1.8 for warmth. New icons should come from Lucide.

## Files
- `SandiBumi UI Mockups.dc.html` — the design document; option ids 1a–1g (Organic default theme) and 2a–2b (client skins) match this README's sections.
- `organic-tokens.css` — token sheet + component classes (.btn, .tag, .table, .seg, .input, .card) the mockups compose with.
- `assets/` — logos.
