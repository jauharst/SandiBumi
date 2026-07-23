# Playbook build progress — first-half push

Living tracker for the "do half the playbook" push (2-week window). Keyed to
`docs/sandibumi_dev_playbook.md` **Part II — the nine enrichment prompts** and its
**Build order** (playbook line 598). Updated + committed as each increment lands.

Legend: ✅ done (commit) · 🔨 in progress · ⏳ waiting on external (your manual pass / a running job) · ▫ queued · ⏸ second half (not now)

Build-order dependencies (playbook §Build order): **#3 → #4**; **#2 organic → #7**;
**#1 + #2 → #8**; **#6.1 before rest of #6**; **#5 & #9 independent (9A cheapest)**.
First-half selection: **#6.1, #3, #2, #1, #9A**. Second half: #4, #7, #8, #5, #9B–D, #6.2.

**✅ FIRST HALF COMPLETE (2026-07-23)** — all five shipped: #6.1 (prior), #3 Rock typing,
#2 SandiMin, #1 Monte Carlo, #9A theming. Both math-heavy features (#3, #1, #2) adversarially
reviewed via Workflow with all confirmed findings fixed pre-commit. Final verification: cargo
279/0/7, tsc 0, zero warnings, production build clean.

**✅ SECOND HALF COMPLETE (2026-07-23)** — all shipped in build order: #4 SHF, #7 Unconventional
(inc1–5), #8 Results-QC (inc1–4), #5 Autocorrelate (inc1–3), #9B–D UI polish, #6.2 assisted
contacts. Every math-heavy feature was adversarially reviewed via Workflow with all confirmed
findings fixed pre-commit. The #9 polish then ran follow-on rounds beyond the B–D base — true-vector
**SVG** export, single-chart **PDF** export, and the free-form **net-flag polygon** — so the
deferred-#9 list is now fully exhausted. Final verification: cargo **354/0/7**, tsc 0.

**Push state:** Jauhar has pushed through `9107294` (now origin/master); the six commits after it
(`9a6b041 → a351ba5 → db3dc61 → 3c648b9 → 201352e → a4e05e9`) are local/unpushed for him.

**Remaining (needs a fresh pick):** the first-half residuals + the maturation (DECIDE) track below,
and — outside this tracker — Feature Wave B, Performance #128–132 (need a live 100-well benchmark),
and the §4 interpretation-workflow open items. Jauhar's manual click-through (REVIEW.md) gates
release, not the build.

---

## Phase 0 — Pre-flight (serves "pre-release ready to compile")

| # | Item | Status | Commit |
|---|---|---|---|
| P1 | Full build — `cargo build --release` + `npm run build` | ✅ | (verify-only) |
| P2 | Lint — zero the 10 build warnings | ✅ | `1c963e4` |
| P3 | UAT doc for Rounds 5–8 (`docs/uat_rounds_5_8.md`) | ✅ | `1c963e4` |
| P4 | Cross-feature review of the 4 shipped commits | ✅ 1 HIGH seam bug found + fixed (TVDSS import shadowing) | REVIEW Round 9 |

## Phase 1 — Playbook #6 · Correlation + fluid contact — Increment 1 ("land what exists")

| Step | Playbook basis | Status | Commit |
|---|---|---|---|
| 6.1 backend+editor | contacts store + TVDSS-flat editor | ✅ already built & committed | (prior) |
| 6.1 field-verify | REVIEW "Try:" — OWC flat in TVDSS, tilted in MD | ⏳ your manual pass (UAT 6.4) | — |
| 6.2 assisted picking | crossover/inflection auto-suggest + cross-well plane | ⏸ second half | — |

## Phase 2 — Playbook #3 · Rock typing (complete the quartet + plots)

Target `rocktyping.rs` + `faciesTieDialog.ts`. Tier-B methods cited; seed apex/Winland
coeffs from IP perm-r35 charts (Tier A) per the Cross-Map — not hand-derived.

| Step | Playbook basis | Status |
|---|---|---|
| 3a | Stratigraphic Modified Lorenz Plot (Σkh vs Σφh, slope-segment flow units) + `cargo test` (segment count on synthetic 3-unit column) | ✅ `lorenz.rs` + `run_lorenz`/`runLorenz`; 9 tests (incl. 3-unit column); cargo 265/0, tsc 0; adversarially reviewed (1 IPC-nullability fix); method banked in `docs/ref_rock_typing.md` (REVIEW Round 10) |
| 3b | Pittman full-apex table (r25/r35/r50) alongside Winland R35; bank math in `docs/ref_rock_typing.md` | ✅ already built — `rocktyping.rs::pittman_rx` (full r10–r75 + APEX selector, port class); ref doc now banked |
| 3c-1 | Lorenz dialog — SMLP curve + flow-unit table + Lc headline (＋ add-panel ▸ Lorenz Plot) | ✅ `lorenzDialog.ts` + workspace wiring; browser-smoke-tested (3-regime stub → 3 units, speed/baffle, row-click highlight); tsc 0 |
| 3c-2 | Winland/Pittman crossplot grid + faciesTie k-variance-reduction; RT FACIES track = existing `fill:"blocks"` | ✅ `crossplotPanel.ts` rock-type iso-radius grid (Winland R35 / Pittman r25/r35/r50, both axis orientations) + `facies_tie.rs` ANOVA k-var-reduction on core plugs (2 tests) + `faciesTieDialog.ts` readout; cargo 267/0, tsc 0; browser-smoke-tested |

**Playbook #3 Rock typing — COMPLETE** (3a solver, 3b pre-existing, 3c-1 Lorenz UI, 3c-2 crossplot grid + tie-in). 3-stretch SOM/MRGC electrofacies deferred (⏸).
| 3-stretch | SOM/MRGC electrofacies engine | ⏸ |

## Phase 3 — Playbook #2 · SandiMin (reconstruction QC + presets)

Target `multimin2.rs` + `multiminDialog.ts`. Consult `docs/multimin_ref_spec.md` /
`multimin_ip_spec.md` before physics. **Gate: settle the smectite-density review first
(data-QC) — I'll flag it to you before touching endpoint density.**

| Step | Playbook basis | Status |
|---|---|---|
| 2a | Per-sample reconstruction curves (measured vs reconstructed + residual per active tool) + combined INCOHERENCE (weighted RMS; cite Quanti.Elan Eq 79) as computed_curves; `cargo test` (3-mineral round-trip; incoherence ~0 perfect, rises with injected endpoint error) | ✅ `recon_qc` flag → `<prefix>_<KEY>_REC`/`_DIF` per tool; RECON documented as Eq-79 incoherence; 2 tests; cargo 269/0, tsc 0 (backend; view = 2d) |
| 2b | Degrees-of-freedom guard (warn #unknowns > #tool-equations; show DOF) | ✅ `MultiminResult.dof` + `dof_note` when exactly determined; test; surfaced in ipc.ts (dialog badge = 2d) |
| 2c | Model presets: quartz-clay-water, SSC (tie ssc.rs), carbonate, **ORGANIC/coal** (feeds #7); seed endpoints from the three-way table (Tier A) | ✅ 4 presets as groupings of EXISTING components (smectite kept as-is per Jauhar 2026-07-22 — no endpoint changes); manual tick → custom; browser-smoke-tested; tsc 0 |
| 2d | Reconstruction-QC view UI (measured-vs-reconstructed overlay + color-filled incoherence track) | ✅ `multiminDialog.ts` "Reconstruction QC" checkbox → DOF line + measured-vs-reconstructed crossplot (per tool, 1:1 line); browser-smoke-tested (checkbox→run→DOF+canvas); tsc 0 |

**Playbook #2 SandiMin — COMPLETE** (2a recon curves, 2b DOF, 2c presets [smectite kept as-is per Jauhar], 2d QC view). Smectite endpoint revision stays an open data-QC item for the team.

## Phase 4 — Playbook #1 · Monte Carlo (close the gap to a commercial engine)

Target `montecarlo.rs` + `monteCarloDialog.ts`. Tornado + Spearman already shipped — do
not rebuild. Seed distribution defaults from IP `MonteCarloDefaults.par` (Tier A).

| Step | Playbook basis | Status |
|---|---|---|
| 1.1 | INCREMENT 1 — Latin Hypercube Sampling default (stratify CDF + permute) + optional Iman-Conover rank correlation + convergence check (running P10/P50/P90, early stop + sparkline); `cargo test` (LHS analytic mean; Iman-Conover target rank corr; convergence stops on stationary series) | ✅ `build_draws` N×P matrix (LHS default, `random` = legacy byte-identical), Iman–Conover via Cholesky re-coloring + rank-match (Spearman→Pearson 2·sin(πρ/6) pre-adjust), batched convergence trace with random-mode-only early stop (LHS never truncates, no runt checkpoints); 5 tests; adversarially reviewed — 4 confirmed findings fixed incl. pre-existing dry-zone tornado null-crash; cargo 274/0, tsc 0 (sparkline UI = 1.3) |
| 1.2 | INCREMENT 2 — per-ZONE parameter distributions + persist P10/BASE/P50/P90 as LOW/BASE/HIGH curves (NAN + versioned log-set) | ✅ `McParam.zone` (span-resolved per well, "PARAM @ ZONE" sensitivity labels, unknown/inverted-zone notes) + `persist` → `MC_<KEY>_LOW/_P50/_HIGH/_BASE` per PRODUCED output via `create_log_set`("MONTECARLO") + versioned write (stale-family reclaim, archive keeps history); 5 tests; adversarially reviewed — 7 confirmed findings fixed (incl. inverted-zone panic, fake input bands, Warned job state); cargo 279/0, tsc 0 |
| 1.3 | UI — LHS/MC toggle, correlation-pair mini-editor, convergence + output histograms, P10/P50/P90 card | ✅ Sampling select (LHS default/Random legacy), per-row zone box (datalist from scoped well), Correlations mini-editor (A↔B, ρ), Convergence + Save-curves checkboxes, per-well convergence sparkline + badge, notes panel, status shows sampling/early-stop/saved-count; browser-smoke-tested (stubbed run → request carries all new fields; sparkline + Avg-PHIE tornado with null endpoint paint clean); tsc 0 |

**Playbook #1 Monte Carlo — COMPLETE** (1.1 LHS/Iman–Conover/convergence, 1.2 zone-scoped + persisted curves, 1.3 UI). Both math increments adversarially reviewed (18 + 27 agents; 4 + 7 confirmed findings all fixed pre-commit).

## Phase 5 — Playbook #9 · UI polish — Increment A only (theming, "cheapest high-value")

Target the visual kit (plotCanvas.ts / LogCanvasRenderer / charts). Scope = theming, not
new petrophysics.

| Step | Playbook basis | Status |
|---|---|---|
| 9A | Hunt & remove hard-coded hex bypassing the theme (highlightsOverlay.PALETTE, well-diagram casing, perf colors, canvas fonts) → readTheme / the 15 vars; tokenize typography; QA light+dark incl. branded palettes (screenshot before/after); adopt IP 96-pattern lithology names as clean-redrawn SVG hatch | ✅ inventory workflow (4 sweeps, 111 bypasses/20 files) → all fixed: `--font-canvas`/`--font-mono` tokens + `canvasFont()` helper (PlotTheme.fontFamily) replace ~55 ctx.font literals; well-diagram casing/shoe → --text, perf → --warn, crossplot no-data gray → --text-dim, highlights palette + Add-curve default from live accents; browser-QA'd all 6 branded palettes (6 distinct accents/markers, all valid hex); tsc 0, prod build clean. **Deferred to 9A-follow: IP 96-pattern lithology SVG hatches** (bigger asset task, not a theming bypass) |
| 9B–D | colorbar/tooltips/brush/lasso/accessibility | ⏸ second half |

---

## Second half (complete 2026-07-23)

Order per playbook §Build order — every dependency is now met (#3→#4 ✅; #2 organic→#7 ✅;
#1+#2→#8 ✅).

| # | Feature | Target | Status |
|---|---|---|---|
| #4 | SHF — Leverett-J in dialog, Thomeer, per-rock-type fits | `shf_fit.rs` + `satheight.rs` + `shfDialog.ts` | ✅ `abf3df4` 4a solvers (Thomeer cited 1960, log-Leverett-J with Tier-A fluid seeds, per-RT grouped fits, exclusion/empty-well honesty, Buckles note; 4 tests; adversarially reviewed — 37 agents, 4 distinct defects fixed incl. a Thomeer bounds panic; math banked in `docs/ref_shf.md`) · 4b dialog (5-family select, Leverett-J PERM/fluid-prop seeds, per-RT tabs, RMS readout, exclusion/notes panel, draggable + click-to-pick FWL) `9ffb94a`; cargo 283/0, tsc 0 |
| #7 | Unconventional/shale suite (TOC Passey/Schmoker, kerogen, GIP/Langmuir, brittleness) | NEW `unconventional.rs` + panel | ✅ inc1 `adb4e29` toc_passey (ΔlogR resistivity-sonic/-density + Schmoker; LOM + editable baseline) · inc2 `269c9ba` kerogen vol + OM-corrected PHIT · inc3 `247a210` gip (free + Langmuir adsorbed + CBM desorption) · inc4 `a4c3c73` brittleness (elastic + mineralogical) · inc5 `1294e72` ΔlogR overlay + Langmuir isotherm panel; Tier-B primaries cited, math banked `docs/ref_unconventional.md` |
| #8 | Results-QC / Sw-comparison dashboard | NEW `resultsQcPanel.ts` (+ `resultsqc.rs`) | ✅ inc1 `a85fa07` Sw-method spread backend (Archie/Simandoux/Indonesia/Juhász envelope; WS/DW only when a Qv/Swb curve exists — never fabricated; adversarial-reviewed, 6 fixed) · inc2 `82c8cf3` panel + per-zone scorecard (Sw-spread + Buckles/BVW traffic-lights) · inc3 `7afe750` Sw-envelope track + Buckles crossplot + CSV + hoverDepth · inc4 `d28127b` recon/MC/cutoff rollup rows (each degrades to grey na, never a silent pass) |
| #5 | Autocorrelate — elastic warp (DTW) + multi-marker | `tops.rs` + `autoCorrDialog.ts` | ✅ inc1 `45fdc2b` elastic depth warp (subsequence DTW, monotone) · inc2 `dfe68b2` multi-marker simultaneous propagation · inc3 `882e99d` dialog warp/shift toggle + per-marker accept/reject review |
| #9B–D | Colorbar/tooltips · brush/lasso linking · accessibility | `plotCanvas.ts` + panels | ✅ 9B `a7ed7bc` shared colorbar + scatter hover tooltip · 9C `9a6b041` linked brushing (crossplot → log view + histogram; 2 real bugs fixed) · 9D `a351ba5` a11y (aria-label/tabindex + keyboard pan/zoom) + prefers-reduced-motion. **Follow-on rounds below.** |
| #6.2 | Assisted contact picking + cross-well plane | `correlationPanel.ts` + Rust detector | ✅ incA `8818693` assisted contact-picking backend (crossover/inflection suggest) · incB `cec3173` panel UI (accept/edit, snap-to-feature) |

### #9 UI-polish — follow-on rounds (beyond the B–D base)

After 9B–D landed, the deferred-#9 items were picked and shipped in turn; the deferred-#9 list is
now fully exhausted. Each carries a REVIEW.md "Try:" line (Rounds 47–48 cover chart-PDF + flag-polygon).

| Round | Item | Commit | Notes |
|---|---|---|---|
| 9C follow-ons | Pickett brush-rings + crossplot scalar cutoff-region (net-box on the param handle) | `db3dc61` | brushed samples ringed on the Pickett log-log; an explicit net-sense dropdown shades the cutoff quadrant + live in-region count; 2 review bugs fixed |
| 9B vector SVG | True-vector **SVG** export for the Canvas-2D charts | `3c648b9` | `SvgRecorder` — a recording 2D context the `PlotCanvas` ctor paints into, so the SAME draw code emits SVG (zero chart re-implementation); ⭳ SVG button + right-click on all 3 panels; 1 review fix |
| chart-PDF | Single-chart true-vector **PDF** export | `201352e` | `PdfRecorder` sibling → PDF content stream, wrapped by the reused `composite.rs::assemble_pdf` (frontend owns operators, backend owns the document); exact text-matrix; 1 review fix (round → butt cap/miter join) |
| flag-polygon | Free-form **net-flag polygon** on the crossplot | `a4e05e9` | `netflag.rs` even-odd point-in-polygon in the axes' drawing plane (exact on log) writes a 0/1/NaN net curve; lasso UI whose live count is a verified twin of the backend; 1 review fix (dblclick during draw) |

## Audit vs the source docs (2026-07-23) — what ELSE is left

Checked `docs/sandibumi_dev_playbook.md` (Parts I + II) and `docs/sandibumi_maturation_prompt.md`
against the repo. Two buckets beyond the second-half table above.

### First-half residuals (sub-items in the shipped prompts that did not ship)

| From | Residual | Lands in |
|---|---|---|
| #1 MC | Per-row live distribution (PDF) preview sparkline (convergence sparkline shipped; per-row preview did not) | `monteCarloDialog.ts` |
| #1 MC | Reject/flag impossible combos (Sw>1, PHIE<0) + report the rejected fraction | `montecarlo.rs` |
| #1 MC | Seed per-parameter defaults from IP `MonteCarloDefaults.par` (Tier A) — today a generic ±20% width heuristic | `monteCarloDialog.ts` |
| #2 SandiMin | Report RMS vs core when core mineral volumes/porosity exist | `multimin2.rs` |
| #3 Rock typing | MICP-calibrated LOCAL Winland/Pittman coeffs (deferred in `docs/ref_rock_typing.md` header) | `rocktyping.rs` |
| #3 Rock typing | Stretch: SOM/MRGC electrofacies engine | `facies.rs` |
| #9A | IP 96-pattern lithology names → clean-redrawn SVG hatches ("9A-follow"; Tier-A names, Tier-D assets — redraw) | composite / log view |

Slotting: the #1/#2 residuals sit in #8 Results-QC's neighborhood — fold them in when #8 starts.
MICP, SOM/MRGC and the hatches stay opt-in extras; none block the second half.

### Maturation (DECIDE) track — never run; queued as separate design sessions

None of the DECIDE output artifacts exist (`docs/competitive/`, `docs/design/`, `docs/roadmap/`
are absent). Per playbook Part 0.1 master sequence:

| Stage | Output | Status |
|---|---|---|
| Stage 0+1 — grounding + capability triage (cheap, scopes everything) | `docs/competitive/maturation-triage.md` | ▫ queued |
| Stage A — direct-adoption register + the Tier-A imports themselves (alias catalogs → importer, naming bridges, vendor chart tables, endpoint library, `.plt` conventions, provenance columns, fixture EULA check) | `docs/competitive/direct-adoption-register.md` + code | ▫ queued — partial credit: per-feature seeds already pulled ad hoc (IP perm-r35 charts → #3; three-way endpoint table → #2 presets) |
| Stage 2 — leapfrog/parity design briefs (per domain) | `docs/design/leapfrog-briefs/*.md` | ▫ after triage |
| Stage 3 — roadmap merge + consolidated Tier-C register | `docs/roadmap/ingest-informed-additions.md` | ▫ last |

These are DECIDE-layer sessions (documents, not code) — runnable any time on request; Stage A's
import rows are the "fast wins" backlog and can interleave with the build.

## Per-increment discipline (playbook acceptance bar)

Every increment: explore-and-restate → implement in small steps → `tsc --noEmit` + `cargo
check` + solver `cargo test` → REVIEW.md entry with a concrete "Try:" line on real data →
commit (plain message; you push). Tier-B math carries a primary-source citation in a code
comment; new method math banked in `docs/ref_*.md`. Tier-C never built.
