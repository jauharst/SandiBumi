# Playbook build progress — first-half push

Living tracker for the "do half the playbook" push (2-week window). Keyed to
`docs/sandibumi_dev_playbook.md` **Part II — the nine enrichment prompts** and its
**Build order** (playbook line 598). Updated + committed as each increment lands.

Legend: ✅ done (commit) · 🔨 in progress · ⏳ waiting on external (your manual pass / a running job) · ▫ queued · ⏸ second half (not now)

Build-order dependencies (playbook §Build order): **#3 → #4**; **#2 organic → #7**;
**#1 + #2 → #8**; **#6.1 before rest of #6**; **#5 & #9 independent (9A cheapest)**.
First-half selection: **#6.1, #3, #2, #1, #9A**. Second half: #4, #7, #8, #5, #9B–D, #6.2.

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
| 3c | Plots UI — Lorenz + Winland/Pittman crossplots (PlotCanvas + chartOverlay), RT class as FACIES block track, extend faciesTieDialog with k-variance-reduction | 🔨 next |
| 3-stretch | SOM/MRGC electrofacies engine | ⏸ |

## Phase 3 — Playbook #2 · SandiMin (reconstruction QC + presets)

Target `multimin2.rs` + `multiminDialog.ts`. Consult `docs/multimin_ref_spec.md` /
`multimin_ip_spec.md` before physics. **Gate: settle the smectite-density review first
(data-QC) — I'll flag it to you before touching endpoint density.**

| Step | Playbook basis | Status |
|---|---|---|
| 2a | Per-sample reconstruction curves (measured vs reconstructed + residual per active tool) + combined INCOHERENCE (weighted RMS; cite Quanti.Elan Eq 79) as computed_curves; `cargo test` (3-mineral round-trip; incoherence ~0 perfect, rises with injected endpoint error) | ▫ |
| 2b | Degrees-of-freedom guard (warn #unknowns > #tool-equations; show DOF) | ▫ |
| 2c | Model presets: quartz-clay-water, SSC (tie ssc.rs), carbonate, **ORGANIC/coal** (feeds #7); seed endpoints from the three-way table (Tier A) | ▫ |
| 2d | Reconstruction-QC view UI (measured-vs-reconstructed overlay + color-filled incoherence track) | ▫ |

## Phase 4 — Playbook #1 · Monte Carlo (close the gap to a commercial engine)

Target `montecarlo.rs` + `monteCarloDialog.ts`. Tornado + Spearman already shipped — do
not rebuild. Seed distribution defaults from IP `MonteCarloDefaults.par` (Tier A).

| Step | Playbook basis | Status |
|---|---|---|
| 1.1 | INCREMENT 1 — Latin Hypercube Sampling default (stratify CDF + permute) + optional Iman-Conover rank correlation + convergence check (running P10/P50/P90, early stop + sparkline); `cargo test` (LHS analytic mean; Iman-Conover target rank corr; convergence stops on stationary series) | ▫ |
| 1.2 | INCREMENT 2 — per-ZONE parameter distributions + persist P10/BASE/P50/P90 as LOW/BASE/HIGH curves (NAN + versioned log-set) | ▫ |
| 1.3 | UI — LHS/MC toggle, correlation-pair mini-editor, convergence + output histograms, P10/P50/P90 card | ▫ |

## Phase 5 — Playbook #9 · UI polish — Increment A only (theming, "cheapest high-value")

Target the visual kit (plotCanvas.ts / LogCanvasRenderer / charts). Scope = theming, not
new petrophysics.

| Step | Playbook basis | Status |
|---|---|---|
| 9A | Hunt & remove hard-coded hex bypassing the theme (highlightsOverlay.PALETTE, well-diagram casing, perf colors, canvas fonts) → readTheme / the 15 vars; tokenize typography; QA light+dark incl. branded palettes (screenshot before/after); adopt IP 96-pattern lithology names as clean-redrawn SVG hatch | ▫ |
| 9B–D | colorbar/tooltips/brush/lasso/accessibility | ⏸ second half |

---

## Second half (later — after your review of the first half)

#4 SHF (Leverett-J in dialog + per-rock-type fits; depends on #3) · #7 Unconventional
(TOC/kerogen/GIP/brittleness; depends on #2 organic) · #8 Results-QC dashboard (depends
on #1 + #2) · #5 Autocorrelate (elastic warp + multi-marker) · #9 Increments B–D · #6.2
assisted contact picking.

## Per-increment discipline (playbook acceptance bar)

Every increment: explore-and-restate → implement in small steps → `tsc --noEmit` + `cargo
check` + solver `cargo test` → REVIEW.md entry with a concrete "Try:" line on real data →
commit (plain message; you push). Tier-B math carries a primary-source citation in a code
comment; new method math banked in `docs/ref_*.md`. Tier-C never built.
