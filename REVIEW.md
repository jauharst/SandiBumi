# Review checklist — for Jauhar's click-through in `npm run tauri dev`

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time; delete items as you confirm them.
Marks: `[o]` confirmed OK (removed from this file), `[x]` confirmed wrong → logged in
**ROADMAP.md §4 (Field-review backlog)**, `[ ]` not yet tested.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [ ] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [ ] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [ ] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [ ] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [ ] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [ ] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
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

Issues you marked `[x]` that need real work (all in ROADMAP §4, P1/P2): well-pin
semantics rework, right-click lockdown (accidental refresh), TVD depth scale UI.
Everything you marked `[o]` has been cleared out of this file.

## Theme switch repaints everything immediately (2026-07-19)

- [ ] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [ ] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

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

- [ ] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [ ] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [ ] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [ ] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [ ] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [ ] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [ ] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [ ] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
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

- [ ] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [ ] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [ ] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [ ] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
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

- [ ] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [ ] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [ ] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [ ] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [ ] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [ ] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
