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
| #1 MC | ✅ `d13df84` Per-row live distribution (PDF) preview sparkline — inline theme-aware SVG per row (Normal bell / Uniform box / Triangular triangle), redraws live as the numbers are typed; purely informational (reads the row's own kind/a/b/c, never feeds the sampler); collapsed spreads → point-mass spike; REVIEW Round 50 | `monteCarloDialog.ts` |
| #1 MC | ✅ `55ef847` Reject/flag impossible combos (Sw>1, PHIE<0) + report the rejected fraction — physical-plausibility guard scanning the unlimited `PHIE_DN`/`SWT_ARCH`/`SWE_INDO` companions; reported per well (⚠/✓/• not-checked), **not** excluded (the module limits already clamp to correct volumetrics); REVIEW Round 49 | `montecarlo.rs` |
| #1 MC | ✅ `f3d7b51` Seed per-parameter defaults from IP `MonteCarloDefaults.par` (Tier A) — `IP_MC_SEEDS` gives each parameter a width in **its own unit** (M/N ±0.2, A ±0.1, GR_MA/GR_SH ±10 API, RHO_MA ±0.03, RHO_FL ±0.02, RHO_SH ±0.05, RHO_DSH ±0.1, NPHI_SH ±0.05; RW/RT_SH ±20% — the two that really are relative), replacing the generic magnitude heuristic that gave RHO_MA a ±0.26 g/cc σ (~9× too wide). Bare-name keying verified collision-free across `modules.rs`/`lrlc.rs`/`ssc.rs`; unseeded params keep the old heuristic byte-identical; a % seed on a zero value falls back rather than collapsing to a point mass. Muted **IP** badge marks a seeded row. Provenance + mapping + the adopted 1σ reading banked in `docs/ref_monte_carlo_seeds.md` (no claim of matching IP run-for-run). Headless check over the real source: 36 assertions pass; tsc + build clean, MC dialog still lazy, main bundle unchanged, no Rust touched; REVIEW Round 56 | `monteCarloDialog.ts` |
| #2 SandiMin | ✅ `2b4a30c` Report RMS vs core — per cored well, RMS of (model − core) + signed bias + plug count for **core φ vs PHIE _and_ vs PHIT** (the drying protocol decides which a plug should match — oven-dried → PHIT, humidity-dried → nearer PHIE — so the bracket is reported rather than one interpretation chosen) and **core ρg** vs the density implied by the solved SOLID volumes Σv·ρ/Σv. The ρg channel is what tests the MINERAL model: bound water is a fluid component so it correctly sits outside the sum (matching a cleaned+dried plug) and the clay term is the dry-clay endpoint, and it is fully independent when RHOB was not an input tool. Plugs tie to the nearest **solved** sample within 1 m (the `facies_tie.rs` tie-in convention); no core → `None`, never a 0.000 that would read as a perfect fit; plug values gated to physical ranges so a percent-unit φ column reports *no fit* instead of a confident RMS ≈ 14.85. Adds `db::get_core_por_gd` (the existing core accessor never reads the `cgd` column), `CoreFit` on the well result, ipc types, and a dialog **Core calibration** table rendered only when some well has plugs. cargo **359/0/7** (2 new tests incl. hand-computed RMS/bias literals + a planted percent-φ/999.25-ρg pair proving the *value* gate rejects them), tsc + build clean; REVIEW Round 57. **Still open:** tying XRD/petrography mineral % (`aux_data`) to component names — needs a component↔item mapping, so it is its own increment | `multimin2.rs` |
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

### Engineering-craft review (Track F) — running 2026-07-24

Prompt: `docs/engineering_review_prompt.md`. Sweeps ONE code-quality property across the WHOLE
app per pass — a different axis from `docs/qc_audit_prompt_template.md`, which audits one TOOL
end to end. Findings become normal serial increments afterward; fixes never land inside a report.

**A skill-driven domain sweep (Tracks A–D, 17 passes against the 45 petrophysics skills) was
drafted and dropped** — Jauhar's call, not planned work. Recoverable from `85e7d69` if wanted.

**No skill authority exists for engineering craft** (verified 2026-07-24: all 48 skills on this
machine are geoscience; no Anthropic frontend/UX/optimization skill is installed). Authoring
house engineering skills was **declined** — the petro-skill pipeline needs source material to
distil and there is none on disk, so it would be a sourcing project. Substitute authorities:
`/code-review ultra` (Jauhar-triggered, diff-scoped) for F1/F2; the inline checklists for the
rest. **F3/F4/F5 are app-wide invariants ultra structurally cannot see** — that is why they stay.

| Pass | Scope | Status |
|---|---|---|
| F1 | Frontend architecture — module seams, cross-dialog duplication + drift, `ipc.ts`↔Rust type fidelity, dead exports, async/error shape | ✅ `docs/review_sweep/F1.md` — 25 survived / 5 refuted. **Headline: the net-flag feature (`a4e05e9`) has never worked** — `NetFlagSpec` is camelCase in TS, snake_case in Rust with no `rename_all`, so `run_net_flag` can never deserialize; `NetFlagResult.outputCurve` likewise reads `undefined`. Verified by hand. Also: a rejected undo/redo deletes the action from both stacks and reports success; `curveSel(preferred)` triplicated and drifted so SHF can fit against the wrong curve; the Workflow copy of the param form has no "(none)" so optional module inputs can't be dropped in a chain |
| F2 | Rust idiom & hot paths — panics on user data, per-sample allocation, error shape + batch isolation, rayon/DB-lock discipline, dead IPC surface | ✅ `docs/review_sweep/F2.md` — 30 survived / 6 refuted. **Cardinal-rule violations:** Monte Carlo swallows module errors → all-NaN volumetrics reported as success; a failed generic-store curve import is `eprintln!`'d only, never in `ImportResult`. **Panics on user data:** NaN top depth from a tops CSV panics `partial_cmp().unwrap()` *while holding the DB lock*; `f64::clamp` panics on inverted zone-parameter bounds (bypasses the dialog's range check); ±inf from a LAS token panics the KNN synthetic-log sort. Plus: legacy `multimin.rs` mixes PEF linearly by volume — the exact error `multimin2.rs` documents and corrects |
| F3 | UX & theming — 6-palette contract, `themeVersion`/`dataVersion` coverage, dialog outliers, empty/failed states, a11y beyond 9D | ✅ `docs/review_sweep/F3.md` — 36 survived / 5 refuted (largest set). **Honesty defects:** pay summary fabricates Net 0.0 / N/G 0.00 / HPV 0.00 for wells whose VSH/PHIE/SWE were never computed; the ML dialog reports green success and writes a History entry claiming every scoped well when only some were written; Field Dashboard says "FLAG curves written." on the one path that deliberately writes nothing. **Theme:** `var(--bg)` is undefined in all 7 palettes with no fallback; Results-QC "ok" renders **bright red** under Halliburton. **a11y:** `formRow()` emits a `<label>` with no `for` across 179 call sites — zero label associations app-wide |
| F4 | Build & bundle — main-bundle composition vs the 664.35 kB (was 1,125.01 kB before R5) baseline, dead deps/config/assets, vega advisories as a real threat model, strictness flags | ✅ `docs/review_sweep/F4.md` — 14 survived / 4 refuted. **The CodeMirror suspicion was right and now has a number: the whole CM6 stack is in the eager index bundle — 461.3 kB, 41.0% of it — silently defeating vegaPanel's dynamic import.** Also: **`cargo test` cannot compile from a fresh clone** (an `include_bytes!` fixture is gitignored and untracked); a LAS-controlled well name is interpolated into `innerHTML` on the Vega empty-data path; `save_png` is an unrestricted arbitrary-path write callable from page JS and absent from the capability file; the full Svelte toolchain is installed and dev-prebundled for zero components |
| F5 | Lifecycle & leaks — dispose symmetry, listener accumulation across open/close, `dataVersion`/`themeVersion` subscription correctness, `filterByActiveGroup` coverage, backend resource lifecycle (jobs registry, Python subprocess, temp files, project-switch cache) | ✅ `docs/review_sweep/F5.md` — 5 dims, 51 agents, 23 raw → **20 survived / 3 refuted** (2 of the survivors are the same defect found independently by two dims, so **19 actionable**). Frontend discipline is good (21 of 23 dock components exactly dispose-symmetric); **the backend is where the problems are** — 4 of 7 Highs are in `jobs.rs`/`lib.rs`/`python_engine.rs`/`chain.rs` and are correctness, not hygiene. Headline: **Cancel is offered for ~27 job kinds but only 5 read the flag** — the other 22 finish, commit their writes, and are then reported "Cancelled"; a **project switch swaps the DuckDB connection under 8 in-flight commands**; a runaway **Python script orphans a CPU-pinned process** with no timeout/kill; a **chain-runner panic wedges both registries "Running" forever**. Plus one flat bug: **the Report pane never opens** (TDZ on `buildWellScope`'s synchronous first subscribe fire — same failure mode as the V3 Vega TDZ crash). Findings are static + agent-verified, **not yet reproduced at runtime** |

Measured up front (2026-07-24, evidence for F4): main `index` **664.35 kB (was 1,125.01 kB before R5)** — unchanged from
baseline; `vegaPanel` 864.37 kB + 22 dialog chunks all lazy; `npm audit` **7 high, all vega**,
every fix semver-major (vega 5→6.3.1, vega-lite 5→6.4.3, vega-embed 6→7.1.0). 969
`unwrap`/`expect` in Rust, most in `#[cfg(test)]`. 53 raw hex literals in TS after the #9A sweep.

**Two of those scouting numbers were wrong and F1d corrected them:** there are **2** `any` (both
in `src/svelte.d.ts`, which nothing imports) and **93** non-null assertions, not 3 and 12 — the
scouting regex `\w!\.` only matched the `x!.foo` form and missed `querySelector<T>(…)!`,
`Map.get(k)!` and post-`.filter()` narrowing. The conclusion still holds: all 93 were read and
fall into four load-bearing idiom families TypeScript cannot narrow, with **zero** `@ts-ignore`,
TODO/FIXME markers or commented-out code anywhere in `src`. Separately, "lazy-chunk discipline
holding" was too generous — the main bundle is unchanged in *size*, but F4a found **41.0% of it
is CodeMirror**, eagerly bundled despite the dynamic import.

### Fix increments from the review — queued 2026-07-24

Ranked by damage ÷ effort out of the 125 survivors. Each follows the per-increment discipline
below (small steps → `tsc --noEmit` + `cargo check` + `cargo test` → REVIEW.md entry with a real
"Try:" line → commit). **Reproduce before fixing** — every finding is static, none was observed
at runtime.

| # | Increment | Scope | Acceptance beyond the usual bar | Status |
|---|---|---|---|---|
| R1 | **NetFlag wire** | `#[serde(rename_all = "camelCase")]` on `NetFlagSpec` + `NetFlagResult` (`netflag.rs:16,37`). Two attribute lines that resurrect the whole flag-polygon feature from `a4e05e9`, which has never worked | A test that actually **crosses serde** — deserialize the TS-shaped JSON literal into `NetFlagSpec` — so the class cannot recur silently. The original increment's frontend-only twin-count check is exactly what missed it; a live "Try:" line must draw a polygon and confirm a curve is written | ✅ `2645b59` — the fix went to **TypeScript**: struct DTOs cross this wire in snake_case (Tauri renames only the top-level arg key). Verified the convention rather than assuming it — a scripted check of all 440 camelCase field names in `ipc.ts` found 22 apparent orphans, every one a top-level `invoke` arg key. Three tests, incl. a **cross-language** one that reads the real `ipc.ts` and fails on drift; proved it fires by regressing the file and watching it fail |
| R2 | **Three panics on user data** | NaN top depth → `partial_cmp().unwrap()` *while the DB lock is held* (`tops`); `f64::clamp` on inverted zone-parameter bounds (bypasses the module dialog's range check); `±inf` from a LAS ASCII token → KNN synthetic-log sort | One regression test per panic, each feeding the **exact malformed input** named in F2a — not a synthetic near-miss. The tops one must also prove the lock is released | ✅ `c796496` — importer drops non-finite top depths + sort no longer unwraps; `ArgSpec.min/max` now enforced in `resolve_param_arrays` (**rejected, not clamped** — clamping 25 to 0.6 would answer with a plausible-but-wrong Sw); non-finite mapped to missing in the LAS path **and equation output** (the verifier established Rhai `1.0/0.0` is the likelier inf source than any LAS cell), KNN skips non-finite distances, z-score floor negated so a NaN std is caught. 5 tests |
| R3 | **Cancel honesty** | F2d + F5e are the same defect class found by two independent runs: Cancel is offered for ~27 job kinds but only 5 read the flag; the other 22 finish, **commit their writes**, and are reported "Cancelled" (LAS import, Rhai equations, ML confirmed) | A per-job-kind decision, not a blanket fix: either observe the flag, or **do not render the button**. Offering a control that does nothing is the defect — a job that legitimately cannot be interrupted should say so | ✅ `fccb5c0` — one idea fixes every job kind: `is_cancelled()` records the **observation**, and `run_job` finalizes on that, so a worker that never polled reports Completed instead of a false Cancelled. Caught a regression while doing it — module runs read the raw flag, so they needed `note_cancel_observed()` or a genuinely drained run would have reported Completed. Real polls added to LAS import, Rhai equations, ML write-back. 2 tests |
| R4 | **Four fabricated-success defects** | MC swallows module errors → all-NaN volumetrics as success; failed generic-store curve import `eprintln!`'d only, never in `ImportResult`; pay summary fabricates Net 0.0 / N/G 0.00 / HPV 0.00 for never-computed wells; ML dialog claims every scoped well when only some were written | Each must degrade **visibly** — the existing convention (⚠/✓/• not-checked, grey `na`) already exists in Results-QC and should be reused rather than a new pattern invented | ✅ `cf4a420` — MC records the first module failure (`OnceLock`, first writer wins across rayon) and fails the well with the module's own message; failed full-curve load now rides in the per-well warning; `n_classified` added so the dialog shows "—", the PDF "-", and the **Dashboard excludes the row** (zeros would drag every median); ML dialog reports `ok/total` and writes no History entry when nothing landed. 1 test |
| R5 | **CodeMirror lazy** | Find the static import defeating `vegaPanel`'s dynamic one; CM6 is 461.3 kB = **41.0%** of the eager index bundle | The build output is the proof: main `index` must drop from **664.35 kB (was 1,125.01 kB before R5)** and a `codemirror-*.js` chunk must appear. Record the new baseline here — the old number is quoted in three places | ✅ `ac0f74f` — `inspectorPanel` imported CodeMirror **statically**, which also defeated `vegaPanel`'s dynamic import. Now lazy, with the Python mode fetched only for Python equations; async mount guarded by a generation counter. **Measured: eager `index` 1,125.01 → 664.35 kB, CodeMirror in 3 lazy chunks totalling 461,537 B** — matching F4a's predicted 461.3 kB to the byte |

The remaining ~120 findings stay in `docs/review_sweep/F1–F5.md`, each already carrying an
independent fix-now / backlog / drop judgement, so the backlog is pre-triaged rather than a flat
list. Nothing there blocks these five.

#### Refinement pass over R1–R5 (`017b3d7`, REVIEW Round 63)

Re-read the five landed diffs adversarially rather than trusting the summary of them. Two real
defects, four hardening items. The lesson worth keeping: **the two defects were both at the seams
between increments, not inside any one of them** — R5's own logic was right but it put an `await`
into a gap another line depended on, and R3's new cancel path landed next to R4's success-counting
without either noticing the other.

| # | Finding | Fix |
|---|---|---|
| 1 | **R5 introduced a data-loss window.** `destroy()` leaves `view.state` readable — a destroyed CodeMirror view is not a null one — so `readFormIntoCurrent` kept returning the *previous* equation's script. Harmless while the mount was synchronous; R5 put an `await` in that gap. Pick equation B, Save before the chunk loads → **A's script written into B**. Round 62 claimed this window was already guarded; it was not | `this.editor = null` alongside the destroy; Round 62's claim struck through and corrected |
| 2 | **Cancelled LAS imports counted as imported.** R3's cancel path returns an entry with neither well nor error, and `ribbon.ts` counted success as `!r.error`. Cancel at file 75 of 120 → **"Imported 120/120 well(s)"** in the status line *and* the permanent History, above 45 notes each saying "cancelled" | Partition on `well_id` (the only proof a well row was committed); cancelled reported as its own count |
| 3 | **R1's guard had a hole on the Rust side.** `SPEC_FIELDS` was hand-kept and only TS was compared to it, so a Rust field with `#[serde(default)]` could stay permanently unknown to `ipc.ts` | Contract now also checked against **serde's own** field list, parsed out of the `deny_unknown_fields` error. Proven by dropping a name and watching it fail. (Adding a field also breaks the build outright — the tests use struct literals — an incidental layer I hadn't credited) |
| 4–6 | Dashboard row filter set its "n excluded" counter as a side effect while the CSV handler called it off the render path; the range check rejected non-finite zone values but not non-finite request values (unreachable today — JSON carries neither NaN nor Infinity — but two rules where there should be one); the "—" note sentence parsed as gibberish | Filter returns the count with the rows; one rule both sides; sentence rewritten |

cargo **370/0/7** (no new tests — an existing guard was strengthened, not added), release build and
`tsc && vite build` clean, eager chunk **664.53 kB** (+0.18 kB for the added logic).

#### Continued fix chain R6–R10 (REVIEW Rounds 64–68)

Same discipline, one finding per commit, each with a REVIEW "Try:" line. These came off the tail of
the F-sweep + two chains the review surfaced late. Common thread: the cardinal rule — **a degraded
or failed result must never be presented as a clean one** — applied to five different surfaces.

| R | Finding | Fix | Commit |
|---|---|---|---|
| R6 | The app could **die before any window** on a bad project DB — `panic = "abort"` in release means a startup `open`/migrate panic aborts silently, no message | `open_and_migrate` extracted + shared with the runtime path (already graceful); `run()` treats failure as a *value* — 3-tier fallback (real → temp recovery file → in-memory), a `StartupProblem` surfaced to a guard-styled `textContent` dialog; does **not** `register_recent` on failure. 2 tests | `3d70615` |
| R7 | Cancel was **rendered for jobs that structurally can't honour it** — R3 fixed the reporting, but `run_simple_job` has no `JobHandle` to poll, so the button was still inert on those | `cancellable: bool` threaded through `Job`/`JobView`/`run_job`/`register`; `run_simple_job` passes `false`; the panel shows "can't be interrupted" instead of a dead button. 1 test | `bf4d817` |
| R8 | The **`src-tauri` test suite couldn't compile from a fresh clone** — `include_bytes!` fixture `corrupt_torn.duckdb` was swallowed by the repo-wide `*.duckdb` ignore | Scoped `.gitignore` exception after verifying the fixture carries **no** well data (header + schema DDL only); both fixtures now tracked | `35bd979`, `d631e44` |
| R9 | A **hostile LAS well name → RCE** — `extract_well_name` stores `~W WELL` verbatim, `vegaPanel` wrote it into `innerHTML`, `csp:null`, sink is `save_png` | Fixed at the **vector**: 5 interpolated-`innerHTML` sites → `textContent` via new `src/ui/safeDom.ts` `messageNode`; 3 private `escapeHtml` copies consolidated. Backlog: 17-site sweep, real CSP, scope `save_png` | `871ca5c` |
| R10 | A **failed undo vanished silently while the UI implied success** — `undoStack.pop()` ran *before* `await action.undo()`, so a rejected DB reversal left the action gone from both stacks; callers had no rejection arm | State mutated only *after* the effect resolves; on rejection the action is restored + the promise re-thrown; both callers report "Undo failed — the change was not undone"; `undo`/`redo` serialized so held Ctrl+Z can't overlap DB writes. Verified via a character-faithful headless port (16 checks) | `efaa01e` |
| R11 | The **depth-scale `<select>` had a transparent background in all 8 palettes** — `.lv-scale` used `var(--bg)`, a variable no palette defines, so it fell to `transparent` (IACVT) instead of the themed surface | → `var(--bg-app)`, the canonical form-control surface (matches `.form-control` + `.mm-dialog select`). Deterministic computed-value swap; grep-proved `--bg` in 0 palettes, `--bg-app` in all 8 | *(this update)* |

`tsc && vite build` clean throughout; R6–R8 also cargo-green. R10 has no cargo/vitest surface (pure
TS promise logic, no frontend test harness exists) — its proof is the headless port in
`scratchpad/undo_check.mjs`. Live desktop click-through for all five stays on REVIEW's Try lines.

## Per-increment discipline (playbook acceptance bar)

Every increment: explore-and-restate → implement in small steps → `tsc --noEmit` + `cargo
check` + solver `cargo test` → REVIEW.md entry with a concrete "Try:" line on real data →
commit (plain message; you push). Tier-B math carries a primary-source citation in a code
comment; new method math banked in `docs/ref_*.md`. Tier-C never built.

## Feature: Vega-Lite interactive charts (Jauhar's "Altair on SandiBumi" ask, 2026-07-24)

Not a playbook item — a requested feature. Chosen shape (via clarifying question): **interactive
Vega-Lite in-app** (vendor the real vega engine), over the export-spec and Python-render options.
Enabled by `tauri.conf.json` `csp: null` (no eval blocker) + npm vendoring (offline). vega is heavy
(~850 KB), so the panel is a **lazy** chunk (`vegaPanel-*.js`), out of the main startup bundle.

**Relationship to the Canvas-2D plots (Jauhar's call, 2026-07-24): complementary, not merged.** The
domain crossplot / histogram / pickett (chartbook + T-S + matrix-point overlays, Pickett Rw/m line,
cutoff regions, linked brushing) stay as the parameter-picking tools; Vega is the general-purpose
**interactive / exploratory** surface. Its ribbon button lives in its own **Interactive** group,
kept out of the domain "Parameter Selection" group so the two roles read clearly.

| Increment | Scope | Status |
|---|---|---|
| V1 | Vendor `vega`/`vega-lite`/`vega-embed`; lazy well-bound **Vega Chart** dock panel + Plot-tab ribbon button; one live interactive crossplot (X/Y curve pickers, tooltip, drag-pan, scroll-zoom) themed from CSS vars | ✅ (this update) — tsc + vite build (offline lazy chunk verified); render/theme screenshot-verified vs synthetic data; live pan/zoom/tooltip → REVIEW Round 51 Try line. `npm audit`: 7 high advisories in vega deps (not auto-fixed — breaking) |
| V2 | Control bar: X/Y/color curve pickers, zone filter, chart type (scatter / line / histogram) | ✅ (this update) — chart-type switch, viridis color curve (scatter-only), zone filter via `getCurveData` depth window; inapplicable controls dim; tsc + offline build; all 3 types render-verified on dev server. REVIEW Round 52 |
| V3 | Live theme repaint on `themeVersion`; vega interval-selection ⇄ `appState.brushedDepths` (linked brushing with the Canvas-2D plots) | ✅ (this update) — themeVersion re-embeds from cached rows (no re-fetch; resets zoom); plain-drag interval brush emits in-box depths via `setBrushedDepths` (pan→Shift-drag, zoom→wheel); consume dims un-selected scatter points via `brushedActive`/`brushedObj` runtime signals + array-form opacity condition (`brushedObj[datum.depth]`). Fixed a TDZ crash (themeVersion.subscribe fires synchronously, read `embedded` before its `let`). Verified: tsc + offline lazy-chunk build; headless vega-lite→vega compile+render check (event selectors + opacity expr valid; consume signals dim 2 bright / 238 faint). Live drag/pan → REVIEW Round 53 |
| V4 | CodeMirror JSON spec editor (already bundled) + native SVG/PNG export + session persistence of the panel/spec | ✅ (this update) — export toolbar (Copy/Save PNG + Print reuse `buildImageExportButtons` off vega's canvas; SVG via `view.toSVG()` → `saveSvg`); Spec editor = CodeMirror (dynamic-imported, stays a lazy chunk) JSON view of the effective spec with data elided, Apply sets an override (rows re-injected via `specFor`, brush signals preserved) / Reset reverts / type-change clears it / inline JSON+render errors; last-used control selections persisted via `savePlotProps("vega")`. NOT persisted: the spec override itself (transient, per-panel). Verified: tsc + offline build (vega + codemirror both lazy, main bundle unchanged); headless override round-trip renders + keeps signals; invalid override throws. Live export/editor → REVIEW Round 54 |
| V5 (capstone) | Analytical modes: **Density** 2D binned heatmap (chart type) + **Trend** regression overlay on scatter (fit line + R², method linear/log/exp/pow/quad) | ✅ (this update) — Density = `rect` mark, x/y `bin:{maxbins:40}`, `aggregate:count` viridis (non-brushable, like histogram); Trend = layered scatter (points + regression line + R² text via `transform:{regression, params:true}` + `calculate`), scatter-only, method dropdown, cached for theme-repaint + persisted. Two layered-spec bugs caught by the headless check and fixed: selection params moved onto the points layer (`Duplicate signal name: grid_x` when top-level) and variable signals kept top-level (`Unrecognized signal name: brushedActive`). Verified: tsc + offline lazy build (main bundle unchanged); headless density renders; all 5 trend methods compile+render with an R² label, keep the brush/grid signals, and consume still dims under the layering. Live interaction → REVIEW Round 55 |
| V6 | **Raincloud** chart type (Jauhar's PtitPrince ask, 2026-07-24): per group a half-violin KDE **cloud** + **box** (IQR/median/Tukey whiskers) + jittered **rain**; **Group** dropdown = by zone or by a categorical curve (rock-type/facies/RT) | ✅ (this update) — geometry (Gaussian KDE + quartiles + jitter) computed in **JS**, drawn with trivial single-view marks (`area`/`bar`/`rule`/`point`) on a synthetic group-lane y so it inherits the panel's `container` sizing/export/theme (VL has no violin; its density/boxplot/facet paths fight container autosize). Data-honest: "(outside zones)" lane not dropped, >24 classes refused (pick a categorical curve), missing group values counted in the status. Verified: tsc + offline lazy build (vegaPanel 864.35→870.82 kB); headless `rc_geom.cjs` 13 checks — geometry invariants (cloud never inverts/stays in lane/bulges, quartiles monotonic, whiskers bracket, rain in-lane, medians recover injected order) **and** the exact production spec compiles at `container` + renders with the real empty-top-data + per-layer-data. NOT seen live (needs Tauri backend + a well; browser flaky this session). Live click-through → REVIEW Round 70 |
