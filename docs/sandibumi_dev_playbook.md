# SandiBumi Development Playbook
### Decide what to build (maturation) → build it into the real codebase (enrichment) — under one IP-clean contract.

This merges the two SandiBumi prompts into a single development arc:
- **`docs/sandibumi_maturation_prompt.md`** (the DECIDE layer) — turns three vendor-intelligence
  streams into IP-clean, roadmap-merged feature designs.
- **`01. Reference/sandibumi_enrichment_prompts.md`** (the BUILD layer) — ships nine concrete,
  code-grounded increments into the actual repo, under the codebase's architecture rules.

Neither is deleted; this playbook supersedes both by wiring them together and adding what the
merge creates: (1) IP-cleanliness governance over the build work, (2) a cross-map from each
extracted reference asset to the build prompt it feeds, and (3) the master decide→build sequence.

**How to use:**
- Read Part 0 (inputs + current state) and Part 0.5 (IP-cleanliness) once — they govern both tracks.
- Follow **Part 0.1 (the master sequence)** for order-of-operations.
- **DECIDE work** (Part I): run in one Claude Code session per stage; design documents only.
- **BUILD work** (Part II): paste one prompt into a **fresh** Claude Code session inside `D:\XX. Arshilla`;
  ship in small verified increments. Each build prompt already embeds the architecture contract.

---

## Part 0.1 — THE MASTER SEQUENCE (decide → build)

Per the enrichment-first decision: **enrichment strengthens the ruler; maturation builds new
things you measure with that ruler — so strengthen the ruler first, but let the cheap triage
scope it.** Do not run the full maturation pass before enriching, and do not enrich blind.

1. **Triage (cheap DECIDE slice)** — run maturation **Stage 0 + Stage 1** only. Produces the map:
   what's `Adopt` (reference data → enrich), `Leapfrog`/`Parity` (design), `Later`/`Out`,
   `Tier-C-blocked`. A few hours of model time; prevents wasted enrichment.
2. **Enrich (BUILD the base)** — run the in-scope enrichment prompts (Part II) **and** the
   `Adopt` reference-data imports (Stage A). Low-risk, mostly effort-S, strengthens the validation
   harness (aliasing, charts, endpoint library, fixtures) that every later design brief is tested
   against. **Settle the SandiMin smectite-density review here** (see Part 0, data-QC) before
   building new inversion features on that table.
3. **Design new (DECIDE)** — maturation **Stage 2** per domain, on the now-enriched base, so briefs
   reference real dependencies instead of `TBD`.
4. **Build new (BUILD)** — a graduated Stage-2 brief runs under the same architecture contract +
   acceptance bar as the enrichment prompts (see **The Bridge**).
5. **Merge roadmap (DECIDE)** — maturation **Stage 3**: diff proposals against `ROADMAP.md`, the
   standing Tier-C register, and a sequencing suggestion.

Model note: run all of this on Opus 4.8 (staged, human-in-the-loop is its strength). Reserve Fable
5 for a single unattended long-horizon pass — and de-prescribe the prompt if you do.

---

## Part 0 — INPUTS & CURRENT STATE (shared by both tracks)

**Repo:** `D:\XX. Arshilla` — Tauri (Rust) + DuckDB + TypeScript/WebGPU. Compute is **Rust**;
TS is frontend only. Reliability-hardening stage; `ROADMAP.md` (Done / Open / Future; Phase /
Severity / Wave) is the authoritative plan.

**Intelligence streams (DECIDE inputs):**
- Techlog 2018.2 ingest — `docs/research_2026-07/techlog_ingest/FINDINGS.md` + `*.json`
  (2,181 families / 723 aliases, 112 charts, 31×20 endpoints, Quanti defaults, Elan equation
  chapter, RockPhyEquations.py).
- IP 2025.3 ingest — `docs/research_2026-07/ip_ingest/FINDINGS.md` + `*.json`
  (315 aliases + Loglan/Elan/PowerLog bridges, 233 ASCII charts, 61 module defaults, MINDEF
  endpoints, `.plt`/`.trk` templates, `D_tierC_register.md`).
- (Optional) Geoactive webinar competitive extraction — mark `TBD` if the file is absent; don't invent it.

**Method/spec inputs (both tracks — specs WIN over code and memory):** `docs/multimin_ref_spec.md`,
`docs/multimin_ip_spec.md`, `docs/method_ssc_sspw.md`, `docs/method_lrlc_rtc_imts.md`,
`docs/workflow_standards.md`, and `CLAUDE.md` (architecture). Skills for method math:
`pe-cbm-unconventional`, `petro-*`, `sw-*`, `geolog-loglan`.

**Current state of the enrichment targets (BUILD ground truth — nothing greenfield except #7):**

| # | Feature | Lives in | Status today |
|---|---------|----------|--------------|
| 1 | Monte Carlo | `montecarlo.rs` + `monteCarloDialog.ts` | tornado + Spearman shipped; gaps: no LHS, no convergence, independent sampling, well-wide only |
| 2 | SandiMin | `multimin.rs` (4-comp) + `multimin2.rs` (27-comp dual-water) + `multiminDialog.ts` | mature; gaps: recon QC mean-σ only, no presets, no per-sample incoherence curve |
| 3 | Rock typing | `rocktyping.rs` + `facies.rs` + `faciesTieDialog.ts` | FZI/GHE/Winland/PGS/Lucia/cutoff done; gaps: no Lorenz, no SOM/MRGC, no Pittman, no MICP-calibrated coeffs |
| 4 | SHF | `shf_fit.rs` + `satheight.rs` + `shfDialog.ts` | Cuddy FOIL/Brooks-Corey/Skelt + FWL scan done; gaps: Leverett-J not in dialog, field-pooled, no Thomeer |
| 5 | Autocorrelate | `tops.rs` + `autoCorrDialog.ts` | bulk cross-correlation shift; gaps: no elastic warp/DTW, single-marker |
| 6 | Correlation + fluid contact | `correlationPanel.ts` + `fluid_contacts` store | contacts editor built, **not committed/field-verified**; gaps: no auto contact picking |
| 7 | Unconventional / shale | **absent** | genuine greenfield — no TOC/Langmuir/brittleness anywhere |
| 8 | Results-QC / Sw-comparison dashboard | — | proposed; ties MC + recon + cutoff QC together |
| 9 | UI polish + visualization | all panels | theming/legends/brushing/a11y pass |

**Data-QC to settle during enrichment (from the three-way endpoint reconciliation):** SandiMin
smectite/montmorillonite RHOB (2.63) reads dry-grain vs both wet-clay libraries (IP 2.02 /
Techlog 2.12) — decide the intended basis and check the other clay rows for consistency before
Prompt 2 builds recon QC on that table. Evidence:
`docs/research_2026-07/ip_ingest/E_threeway_endpoint_compare.json`.

---

## Part 0.5 — IP-CLEANLINESS (governs BOTH deciding AND building)

Every capability you design *or implement* — including enrichment work that imports vendor
reference data — is classified into exactly one tier. If ambiguous, take the **more restrictive**
tier. This is the merge's key addition to the build track: the enrichment prompts previously had
no IP-cleanliness governance.

- **Tier A — reference data & market intelligence (free to use).** Curve-alias/unit/lithology
  catalogs, chart lookup tables, mineral-endpoint values, parameter defaults, naming bridges,
  palettes, template *conventions*, casing/hole tables, feature categories, pain points, QC culture.
  Extracted from the ingests → **adopt directly** (a vendor default is a seed, not field truth —
  always expose it as a per-well override). *Redraw* pattern/bitmap assets clean; never lift a
  vendor's actual bitmap.
- **Tier B — published, citable science (implement from the primary source only).** Archie /
  Waxman-Smits / Simandoux / Indonesia / dual-water, Thomas-Stieber, Amaefule FZI, Winland /
  Pittman / Lucia r35, Lorenz, Leverett-J / Brooks-Corey / Thomeer / Cuddy, Batzle-Wang / Gardner,
  Passey ΔlogR / Schmoker, Langmuir, Iman-Conover / LHS, DTW, the Quanti.Elan incoherence /
  conductivity equations (Eq 78/79/80). **Cite the primary paper in a code comment**; identify the
  method + constants from the ingest, then reimplement independently. Never treat vendor wording or
  code as the spec.
- **Tier C — patented/proprietary (do NOT implement, approximate, or reverse-engineer).**
  Pre-seeded from the IP ingest (`ip_ingest/D_tierC_register.md`): Omovie `SonicSaturation`
  (US Patent 12,242,011 B2 — even a Tier-B acoustic-Sw design-around clears counsel first),
  Domain Transfer Analysis, Experienced Eye, entropy image speed-correction, shipped NN weights.
  Where the user-need matters, design a Tier-B alternative and document why it's distinct; tag
  briefs near a patent `LEGAL-REVIEW`. Flag, don't conclude.
- **Tier D — expression (never reproduce).** UI layouts, menu structures, module names, doc prose.
  Name capabilities in SandiBumi's own vocabulary. (Enrichment #9's lithology-pattern *names* and
  color-ramp *values* are Tier-A facts; the literal bitmap assets and screens are Tier D — redraw.)

Maintain the consolidated **Tier-C register** as a standing file for counsel (Stage 3.4).

---

# PART I — DECIDE (Maturation track)

## Role & mission
Product-engineering brain for SandiBumi (two-agent: data conditioning + evaluation; corpus-scale
over 1000+ multi-vintage wells; validated against IP/Geolog/Techlog ground truth). Turn the
intelligence streams into capabilities **functionally superior to — never copies of — the
incumbents**, packaged to merge into the roadmap without disrupting it. Two of three streams are
*install ingests*: exact reference DATA + method identification, not just market signal — adopt the
data where unprotected, re-derive methods from primary literature.

## The three streams differ — treat them differently
| | Install ingests (Techlog, IP) | Webinar intel (optional) |
|---|---|---|
| What it is | exact reference DATA + identified methods | market signal / direction |
| Primary use | *adopt* (Tier A) / *re-derive* (Tier B) / *avoid* (Tier C) | decide *what* to build, *how good* |
| Confidence | high (verbatim, cross-checked) | medium (framing, caption uncertainty) |
| Failure to avoid | copying protected expression/algorithm; treating a UI default as field-authoritative | inflating v1 scope with the incumbent's breadth |

## Design doctrine — what "better than the incumbent" means
1. **Re-derive, don't port.** Go to the petrophysical job-to-be-done; design the *agentic* solution
   from first principles. "IP's dialog, but automated" → start over.
2. **Automate the decision, generate the evidence.** Every automated action emits: the decision, its
   inputs, a confidence measure, the QC artifact a petrophysicist would look at, and an
   exception-queue entry when confidence is low.
3. **Batch-first, override-live second.** Corpus-wide default; interactive override is the review
   layer. Match the incumbents' interactivity *bar* there only.
4. **Beat the honest ceilings.** Row-wise random splits leak spatial autocorrelation; sample caps;
   single-well-then-multi-well tools; desktop binding; one-zone default; core defaults locked in a
   compiled exe → design past with blind-well/stratified validation, no caps, multi-well-native,
   automated zonation, open/portable defaults.
5. **Decisions are assets.** Every parameter/trend/cutoff/zonation/endpoint/model is exportable,
   reloadable, written into the playbook / parameter KB.
6. **Founder's market reality.** Multi-vintage messy Indonesian data, LRLC pay, thin beds,
   carbonates, IP/Geolog/Techlog-trained users, LAS/CSV round-trips, fresh Mahakam muds (SP
   suppression), Malay-basin sand-silt-clay.
7. **Scope discipline.** v1 = openhole petrophysics with the two agents. Pore-pressure/geomech, rock
   physics, NMR, image logs, cased-hole/C-O, PL, WITSML, CCS = intelligence, not v1 (both ingests
   are full of these — don't let their breadth inflate v1). *Exception:* enrichment #7
   (unconventional) is an explicit, user-requested v1 build — see Part II.
8. **Solo-dev honesty.** Every design carries an S/M/L effort tier. Prefer reusing SandiBumi
   infrastructure (module runner, workflow chains, python_engine subprocess, composite renderer,
   versioned DuckDB store) over new subsystems.

## Stage 0 — grounding (always first)
Read `ROADMAP.md` + both ingest `FINDINGS.md` + the current-state table (Part 0). Output <250 words:
current scope/phase (name Done/Open/Future + where reliability-hardening sits), which streams are
present (confirm the webinar file or mark TBD), any assumption the roadmap forces. Roadmap ↔ prompt
conflict → STOP and ask.

## Stage A — direct-adoption of reference data (run any time, parallel to all stages)
Tier-A reference data is import-and-use, not design work. Produce
`docs/competitive/direct-adoption-register.md`:
`| Data asset | Source file(s) | SandiBumi consumer | Adoption action | Effort | Verify | Feeds build prompt |`

**This is where DECIDE meets BUILD — every adopted asset is cross-mapped to the enrichment prompt
it feeds (see The Cross-Map below).** Cover at least: the alias catalogs (Techlog primary + IP
vendor-channel layer → importer), naming bridges (Loglan/Elan/PowerLog), vendor charts (retire
`tools/chartdig` where covered; cross-validate digitized charts), the three-way endpoint table,
parameter/MC defaults, `.plt`/`.trk` template conventions + lithology patterns + color ramps, IP
object-model provenance columns, and the BLSO + IP `Testdata.las` fixtures (EULA-check before
bundling). Most rows are effort-S — this is the "fast wins" backlog and the bulk of step 2 in the
master sequence.

## Stage 1 — capability triage (run once, across all present streams)
Walk every capability from the FINDINGS §1/§2 (+ any webinar §). One row:
`| Source ref (TL:/IP:/WB:) | Capability (SandiBumi wording) | Verdict | Agent | Tier | Reason | Roadmap anchor |`
- **Verdict** ∈ {`Adopt` (Tier-A data → Stage A, list don't brief), `Parity`, `Leapfrog`, `Later`,
  `Out`, `Tier-C-blocked` (register only)}.
- **Agent** ∈ {A1-conditioning, A2-evaluation, Platform, KB}. **Tier** = A/B/C/D + `LEGAL-REVIEW`.
- **Roadmap anchor** = existing Phase/Wave/Severity item, or `NEW`.
Close with: top-5 Leapfrog by (differentiation ÷ solo-dev effort); the full Adopt list (→ Stage A);
every Tier-C item (→ register). Write `docs/competitive/maturation-triage.md`. Don't run Stage 2
in the same session unless told.

## Stage 2 — leapfrog/parity design briefs (per domain, on request)
One brief per `Leapfrog` item and per `Parity` item needing design. Every field mandatory; no
invented numbers; `TBD` is information:
```
## <SandiBumi capability name>
- Source ref: TL:… / IP:… / WB:§…
- Job to be done: <one sentence>
- Incumbent baseline: <=2 lines — what IP/Techlog does + its limitation, from the ingest>
- SandiBumi design: <re-derived agentic design: pipeline position, inputs, decision logic,
  evidence artifacts emitted, exception criteria, override surface>
- Why measurably better: <specific testable claims — corpus-wide per-well logs, blind-well
  validated, no cap, portable defaults — no adjective without a metric or mechanism>
- Method provenance: <Tier-B primary citations by author/year; or "Tier A — reference data">
- IP-cleanliness: <tier + rationale; design-around if Tier-C-adjacent; LEGAL-REVIEW if applicable>
- Data requirements & dependencies: <curves, corpus, KB entries, other SandiBumi modules>
- Validation criterion: <exact test vs the IP/Geolog/Techlog corpus — number/plot that proves it;
  use the BLSO fixtures; what "as good or better than ground truth" means here>
- Playbook/KB hook: <what decision/parameter this writes into the KB>
- Effort: S / M / L + main risk
- Build handoff: <which enrichment file/module it lands in, per the Bridge>   ← merge addition
```
Write to `docs/design/leapfrog-briefs/<slug>.md`. Design only — no code unless asked.

## Stage 3 — roadmap merge (last, once briefs exist)
`docs/roadmap/ingest-informed-additions.md`: (1) diff-style proposals keyed to the roadmap's own
bucket/label names (`ADD … to <Phase/Wave>`, `RESHAPE …`, `DEFER … to Future`, `ADOPT <Stage-A
asset>`); (2) conflicts & open questions (give Jauhar the decision, don't resolve strategy);
(3) sequencing for the top-5 Leapfrog relative to reliability-hardening, noting which Stage-A
adoptions and enrichment increments unblock them; (4) the consolidated Tier-C register (seed from
`ip_ingest/D_tierC_register.md`).

---

# THE BRIDGE — from design brief to build increment

A DECIDE artifact becomes a BUILD increment when a Stage-2 brief (or a Stage-A adoption) graduates
to implementation. The handoff rules:

- **Same architecture contract.** Any implementation — enrichment prompt OR graduated brief — obeys
  the contract in Part II (Rust compute + manifest auto-dialog, NAN/bytemuck, 15 theme vars,
  verify→REVIEW→commit). A brief's `Build handoff` field names the target `.rs`/`.ts` file.
- **Same acceptance bar.** A graduated brief ships under Part II's reusable acceptance bar
  (`tsc` + `cargo check` + solver `cargo test` + a REVIEW.md "Try:" line on real well data +
  self-QC against `docs/qc_audit_prompt_template.md`).
- **Tier discipline carries over.** The brief's `IP-cleanliness` tier governs the build: Tier-A data
  is loaded; Tier-B methods are reimplemented from the cited primary (comment the citation);
  Tier-C is never built.
- **Most adoptions need no new module.** A Tier-A catalog/chart/default import extends an existing
  system (importer, `neutron_charts.rs`/`chartdig`, `multimin2.rs`, dialog defaults) — it's an
  enrichment increment, not a design brief.

## The Cross-Map — which extracted asset feeds which build prompt
This is the concrete wiring the merge creates. Load the asset (Tier A) or cite the method (Tier B);
do not hand-derive what the ingest already provides.

| Extracted asset (ingest / Stage A) | Feeds build prompt | How |
|---|---|---|
| Alias catalogs (Techlog 723 + IP 315 vendor-channel) + naming bridges | **All** + importer | clean corpus load — foundational, do first |
| Three-way endpoint table (IP MINDEF + Techlog QM + SandiMin) | **#2 SandiMin** | alt endpoint library + presets; settle smectite-density review first |
| Quanti.Elan equation chapter (Eq 78/79/80) + IP MINEQDEF | **#2 SandiMin** | citable incoherence/conductivity math (Tier B, cite) for the recon/incoherence curve |
| IP perm-r35 charts (36: Pittman/Winland/Lucia) + Techlog charts | **#3 Rock typing** | seed Pittman full-apex + Winland tables from chart data, not hand-derivation |
| Techlog CPMParameters SwH (Lambda/φ-dependent) + IP cap-pressure fluid defaults | **#4 SHF** | Leverett-J/Thomeer/Brooks-Corey defaults + fluid props |
| Techlog TOC (Passey) module + IP TOC defaults + `pe-cbm-unconventional` skill | **#7 Unconventional** | Passey ΔlogR baselines, Langmuir defaults; SandiMin organic-preset endpoints |
| RockPhyEquations.py (Batzle-Wang/Gardner) | **#7 brittleness** | Young's/Poisson elastic path (Tier B, reimplement from primaries) |
| IP `MonteCarloDefaults.par` (Rw 20%, m/n ±0.2, …) | **#1 Monte Carlo** | seed per-parameter distribution defaults |
| Charts (density-neutron, M-N, MID, Pe-density, Buckles) | **#8 Results-QC** + `chartdig` | Buckles/crossplot overlays; retire hand-digitization where covered |
| Module parameter defaults (Techlog C2 + IP C, 61 modules) | dialogs across **#1–#8** | sensible vendor default seeds (per-well overridable) |
| IP `.plt`/`.trk` template grammar + 96 lithology patterns + COLORREF ramps | **#9 UI polish** | composite conventions; redraw patterns as clean SVG (Tier D — don't lift bitmaps) |
| BLSO fixtures (real Mahakam-adjacent) + IP `Testdata.las` | **All** acceptance bars | the "real well data" the REVIEW "Try:" lines run on |

---

# PART II — BUILD (Enrichment track)

## The architecture contract (paste at the top of EVERY build prompt below)
This is the universal build contract — for the nine enrichment prompts AND any graduated maturation
brief. It is the "how to build in THIS repo" discipline the DECIDE layer relies on.

```
You are enriching SandiBumi (D:\XX. Arshilla), a Tauri(Rust)+DuckDB+TypeScript/WebGPU petrophysics
desktop app. Jauhar (the user) is a petrophysicist and beginner programmer — explain choices in
petrophysics terms, not programming jargon. Read CLAUDE.md fully before touching code; the specs in
docs/ WIN over any code or memory.

IP-CLEANLINESS (from the playbook Part 0.5): Tier-A reference data (catalogs/charts/endpoints/
defaults from the ingests) may be adopted directly but stays a per-well-overridable default, not
field truth. Tier-B methods must be reimplemented from the cited PRIMARY paper (cite it in a code
comment) — never from vendor code/wording. Tier-C methods (Omovie SonicSaturation US 12,242,011 B2,
DTA, Experienced Eye, entropy image speed-correction, shipped NN weights) are NEVER built; design a
Tier-B alternative and flag LEGAL-REVIEW. Redraw vendor pattern/bitmap assets clean (Tier D).

NON-NEGOTIABLE ARCHITECTURE RULES (from CLAUDE.md):
- Petrophysics compute lives in RUST (src-tauri/src). A new method = a Rust fn + a manifest entry in
  modules.rs list_modules()/run_module() using the param()/opt()/log_in()/log_in_computed()/
  log_out() builders. The parameter dialog and ribbon button AUTO-GENERATE from the manifest —
  write ZERO frontend code for a plain module. Heavy solvers get their own .rs file (like
  montecarlo.rs / multimin2.rs / satheight.rs / rocktyping.rs).
- Only build a custom TS panel when the UI is genuinely non-form (a chart/editor). Pattern: export
  async function buildXContent(setStatus): Promise<{el, dispose?}>; add a case in
  Workspace.buildRenderer (asyncPane/wellPane); an openX() via openSingleton; and a #x-btn handler
  in ribbon.ts. Modals: openModal(title, el, width)/formRow from modal.ts (non-blocking, Esc/✕ only
  — never add a blocking scrim). Run-scope selector: buildWellScope from wellScope.ts, and wrap the
  well list in filterByActiveGroup for any batch dialog.
- Missing data is f32::NAN (never Option). IPC passes f32 arrays as raw bytes via bytemuck, never
  JSON. DB writes are whitelisted (db.rs TABLE_SPECS) — the frontend never sends write SQL. Python
  only as a subprocess (never embedded); a missing Python must never break launch.
- Read curves: getCurveData(wellId, names, dMin, dMax) -> TrackCurveSeries{depth,value:
  Float32Array}. WRITE curves only by running a module (Rust writes computed_curves, versioned into
  a log set); then call workspace.notifyDataChanged()/bumpDataVersion() and recordProcess(kind,
  detail, well). Params persist automatically as log_sets.params_json.
- Colors: readTheme(el) from plotCanvas.ts -> {bg,grid,axis,text,accent,accent2,warn}. Use ONLY the
  15 theme CSS vars (--bg-app,--bg-panel,--bg-panel-alt,--bg-hover,--border,--border-strong,--text,
  --text-dim,--accent,--accent-dim,--accent-soft,--accent2,--accent2-soft,--warn,--track-hd) —
  never hard-code hex. Subscribe appState.themeVersion and repaint on change. Categorical fills:
  FACIES_PALETTE/faciesColor; continuous: viridis via colormapColor. Reuse PlotCanvas +
  attachZoomPan + buildImageExportButtons; don't build a parallel plotting kit.
- User-editable data changes are undoable via pushUndo({label,undo,redo}); module runs are
  re-runnable, NOT undone. UI text is English (auto-translated by i18n.ts); mark user data
  (well/curve names) data-no-i18n.

METHOD MATH: cite the source in a code comment. Bank new method math in a new docs/ref_*.md
(portable — do not rely on machine-local memory). Existing specs: docs/multimin_ref_spec.md,
docs/multimin_ip_spec.md, docs/method_ssc_sspw.md, docs/method_lrlc_rtc_imts.md,
docs/workflow_standards.md. Seed defaults/charts/endpoints from the ingest FINDINGS
(docs/research_2026-07/*/FINDINGS.md) per the playbook Cross-Map before hand-deriving.

WORKFLOW: work in small increments. Each increment: implement -> VERIFY (npx tsc --noEmit +
cd src-tauri && cargo check + cargo test for solver math + a browser functional test via the vite
`await import('/src/ui/...')` trick) -> add a REVIEW.md checklist entry with a "Try:" line for
Jauhar (run it on the BLSO / Testdata.las fixtures) -> commit (plain message, no embedded double
quotes; Jauhar pushes himself, you never touch gh auth). NEVER force-kill `npm run tauri dev`
(corrupts the DuckDB WAL); after browser testing, free port 1420. Lead your report with outcomes
and propose the next increment; wait for "go ahead" before starting a new one.

BEFORE CODING: explore the named files, restate in <=10 bullets what exists vs what you'll add, and
list the exact files you'll touch. Then implement.
```

## The nine enrichment prompts
Each is runnable as-is in a fresh session (it already includes the contract above). The header lines
are the merge's additions: the IP-cleanliness tier and the Stage-A asset that feeds it.

### 1. Monte Carlo — close the gap to a commercial uncertainty engine
**IP-cleanliness:** methods Tier B (LHS, Iman-Conover — cite); defaults Tier A (IP `MonteCarloDefaults.par`).
**Fed by (Stage A):** IP per-parameter MC distribution defaults.
```
[paste the architecture contract]

TARGET: montecarlo.rs + monteCarloDialog.ts (buildMonteCarloContent, drawHistogram,
renderSensitivity, drawTornado). Tornado + Spearman already shipped (commit d64bdc7) — do NOT
rebuild. Close these gaps in order:

INCREMENT 1 — sampling quality: Latin Hypercube Sampling as default (keep plain MC optional);
stratify each param CDF into N bins, permute, so P10/P90 stabilize at lower N. Optional input
CORRELATION (rank-correlation between paired params — m & n co-vary) via Iman-Conover rank
reordering; default independent. CONVERGENCE check: track running P10/P50/P90 vs iteration, stop
early when HPV P90 changes < tol over a window, report effective N + a convergence sparkline.
Seed the default distributions from the IP MonteCarloDefaults.par table (Tier A) per the Cross-Map.

INCREMENT 2 — resolution & outputs: per-ZONE parameter distributions (reuse zone_params); persist
P10/BASE/P50/P90 as CURVES (LOW/BASE/HIGH) for composite/report (respect NAN + versioned-log-set).

UI/UX: live PDF preview sparkline per row + a correlation-pair mini-editor; convergence + output
histograms via PlotCanvas + readTheme; P10/P50/P90 summary card; 15 theme vars only.

OPTIMIZATION: keep rayon-parallel/in-memory (only LOW/BASE/HIGH persistence touches DB, once);
reject/flag impossible combos (Sw>1, PHIE<0) and report the rejected fraction. cargo test: LHS
reproduces a known analytic mean; Iman-Conover hits target rank correlation; convergence stops on
a stationary series.
```

### 2. SandiMin (Multimin) — reconstruction QC + presets
**IP-cleanliness:** dual-water/conductivity Tier B (cite Waxman-Smits / dual-water / Elan Eq 78);
endpoint library Tier A (adopt). NOT Omovie/DTA.
**Fed by (Stage A):** three-way endpoint table; Quanti.Elan Eq 78/79/80; IP MINEQDEF.
```
[paste the architecture contract]

TARGET: multimin2.rs (27-comp generalized dual-water — the active one; do NOT touch multimin.rs) +
multiminDialog.ts (buildMultiminContent). Consult docs/multimin_ref_spec.md + multimin_ip_spec.md
BEFORE changing physics. FIRST settle the smectite-density review (playbook Part 0 data-QC).

METHODS/QC (the real gap — today QC is mean recon σ per well): emit per-SAMPLE reconstruction
curves (measured vs reconstructed + residual per active tool RHOB/NPHI/DT/GR/PEF/U/SIGMA), plus a
combined INCOHERENCE curve (weighted RMS) — cite Quanti.Elan Eq 79 (weight = 1/uncertainty) and the
Simandoux conductivity Eq 78 from ip_ingest/H_doc_index.md. Write as normal computed_curves.
Degrees-of-freedom guard: warn when #unknowns > #tool-equations; show DOF. Model PRESETS:
quartz-clay-water, SSC (tie ssc.rs), carbonate (calcite-dolomite-anhydrite), and an ORGANIC/coal
preset (low-density low-U kerogen) that feeds #7 — seed endpoints from the three-way table (Tier A).

UI/UX: reconstruction-QC view (measured vs reconstructed overlay per tool + color-filled incoherence
track, PlotCanvas/readTheme); preset dropdown atop the component editor; keep the editable endpoint
matrix + multiminFluidCalc preview.

OPTIMIZATION: ill-conditioned intervals degrade gracefully and FLAG (high incoherence), never NaN
silently; if core mineral volumes/porosity exist, report RMS vs core. cargo test: a synthetic
3-mineral rock round-trips to known volumes; incoherence ~0 for a perfect sample and rises
monotonically with injected endpoint error.
```

### 3. Rock typing — complete the quartet + add the plots
**IP-cleanliness:** FZI/Winland/Pittman/Lucia/Lorenz Tier B (cite Amaefule/Winland/Pittman/Lucia/
Lorenz); chart data + coeffs Tier A.
**Fed by (Stage A):** IP 36 perm-r35 charts (Pittman/Winland/Lucia); MICP calibration data.
```
[paste the architecture contract]

TARGET: rocktyping.rs (RQI/φz/FZI, GHE bins, Winland R35, PGS, Lucia RFN, rt_cutoff exist) +
faciesTieDialog.ts. ROADMAP §B3 item 8 inc 2. GHE bins + PGS exponent (3.5) are flagged as
literature/recall — calibrate or cite properly.

METHODS: Stratigraphic Modified Lorenz Plot (Σkh vs Σφh, slope segments = flow units); Pittman full
apex table (r25/r35/r50) alongside Winland R35 — seed the apex/Winland coefficients from the IP
perm-r35 chart data (Tier A) per the Cross-Map, not hand-derivation; optional MICP calibration (fit
local Winland/Pittman coeffs per field when core Pc/MICP present → write to a docs/ref_* note);
(stretch) one advanced electrofacies engine (SOM or MRGC) as a new facies.rs option.

UI/UX: Lorenz + Winland/Pittman crossplots via PlotCanvas + chartOverlay; RT class as a discrete
FACIES_PALETTE block track; linked hover via appState.hoverDepth. Extend faciesTieDialog with the
k-variance reduction from typing next to the confusion matrix.

OPTIMIZATION: exclude BADHOLE/NAN from clustering; seed deterministically (SplitMix64); order RT
classes monotonically (ascending FZI/GR). cargo test: FZI->GHE bin for known φ,k; Lorenz segment
count on a synthetic 3-unit column; φ-k variance drops after typing.
```

### 4. SHF — Leverett-J in the dialog + per-rock-type fits
**IP-cleanliness:** Leverett-J/Brooks-Corey/Thomeer/Cuddy Tier B (cite); SCAL/cap-pressure defaults Tier A.
**Fed by (Stage A):** Techlog CPMParameters SwH models; IP cap-pressure fluid defaults. **Depends on #3 (RT curve).**
```
[paste the architecture contract]

TARGET: shf_fit.rs (fit_foil, fit_brooks_corey, fit_skelt, nelder_mead, foil_fwl_scan) +
satheight.rs (fit_leverett_j, sw_height) + shfDialog.ts. ROADMAP §B3 item 8 SHF side.

METHODS: expose Leverett-J fitting IN shfDialog (today fit_leverett_j only runs at SCAL import) —
J = 0.21645·(Pc/IFT)·√(k/φ), fit Sw(J), convert to Sw(h); add Thomeer (hyperbolic Pc, BVO vs G/Pd) —
the carbonate-standard model; PER-ROCK-TYPE fits: take an RT/facies curve as optional grouping and
fit one SHF per rock type (integrates #3) — the single biggest accuracy win for a Mahakam well.
Seed default families/fluid props from the Techlog/IP SwH defaults (Tier A) per the Cross-Map.

UI/UX: family dropdown (FOIL/Brooks-Corey/Skelt/Leverett-J/Thomeer) + "fit per rock type" toggle
with per-RT tabs; Sw-vs-height crossplot with fitted curve + residuals/R² (PlotCanvas); draggable
FWL (foil_fwl_scan / Cuddy Eq 19 stays the auto-FWL).

OPTIMIZATION: bounded fits (refuse Sw>1 near FWL or Sw<Swirr; report excluded points); sanity-check
vs Buckles and flag violators; export fitted coeffs into the sw_height param table PER rock type
(extend, don't replace the single-law export). cargo test: Leverett-J round-trips a synthetic Sw(J);
per-RT split yields ≥2 distinct laws on a 2-facies synthetic; Thomeer fits a known G/Pd.
```

### 5. Autocorrelate — elastic warp + multi-marker
**IP-cleanliness:** DTW/warp Tier B (published DSP); no vendor-protected content.
**Fed by (Stage A):** none directly (GRN P3/P97 reuse is internal).
```
[paste the architecture contract]

TARGET: tops.rs (autocorrelate cross-correlation) + autoCorrDialog.ts. Today: single best-lag BULK
GR shift per well, r-scored, undoable batch.

METHODS: elastic depth WARP (dynamic depth warping) within the window — monotonic, no-inversion,
penalize large local stretch (keep rigid-shift as fast default); MULTI-MARKER simultaneous
propagation (align on shape between several tops, propagate the set consistently); per-interval
confidence (r per propagated marker, not per well).

UI/UX: warp-vs-shift toggle + max-stretch control; alignment as tie-lines with per-interval r;
low-confidence intervals flagged; accept/reject per marker; all proposals one undoable batch.

OPTIMIZATION: normalize the correlation curve (GR P3/P97 — reuse gr_normalize) before matching;
constrain warp monotone (no depth crossovers); keep original vs shifted depth (reversible).
cargo test: rigid shift recovers a known lag; warp recovers a known piecewise stretch; monotonic
constraint holds on noisy input.
```

### 6. Correlation + fluid contact — verify, then auto-pick
**IP-cleanliness:** crossover detection Tier A logic; no protected method.
**Fed by (Stage A):** density-neutron/crossover chart references. **Do increment 1 FIRST (uncommitted code).**
```
[paste the architecture contract]

TARGET: correlationPanel.ts + the fluid_contacts store (OWC/GWC/GOC/GDT/ODT/FWL). ROADMAP §B3
item 9 — code BUILT but NOT COMMITTED and NOT field-verified.

INCREMENT 1 — land what exists: verify the contacts editor + TVDSS-flat rendering end to end
(tsc + browser); REVIEW "Try:" — enter an OWC on a deviated well, confirm flat in TVDSS, tilted in
MD; commit only after it passes. No new scope until verified.

INCREMENT 2 — assisted contact picking (contacts are manual today): suggest a contact from logs
(Sw=0.5 crossover, resistivity inflection, density-neutron crossover within a zone) with a
confidence — user accepts/edits, never auto-commit; snap-to-log-feature on the draggable line;
cross-well consistency (fit a plane/trend to picked contacts — flat in TVDSS — flag disagreeing
wells).

UI/UX: contacts as horizontal lines with cross-well connectors; contact-consistency readout
(predicted vs observed per well + tilt/trend fit); height-above-contact feeds #4 SHF as a
computable input.

OPTIMIZATION: virtualize rendering (10+ wells smooth); persist picks in the versioned fluid_contacts
store. cargo test: TVDSS contact depth flat across deviated wells; crossover detector finds a known
Sw=0.5 depth on synthetic logs.
```

### 7. Unconventional / shale suite — greenfield (explicit v1 build)
**IP-cleanliness:** Passey ΔlogR / Schmoker / Langmuir / brittleness Tier B (cite Passey 1990,
Schmoker, Langmuir; `pe-cbm-unconventional` skill). NOT any patented sonic-Sw method.
**Fed by (Stage A):** Techlog TOC(Passey) + IP TOC defaults; SandiMin organic-preset endpoints (#2).
**Depends on #2 organic preset.**
```
[paste the architecture contract]

TARGET: NEW. Nothing exists (the only "TOC" hit is a placeholder string). Build as Rust registry
modules (auto-dialog) in a new src-tauri/src/unconventional.rs + a custom panel only where a form
won't do. Use the pe-cbm-unconventional skill for Langmuir/material-balance. Bank math in
docs/ref_unconventional.md.

METHODS (each a manifest module → auto-dialog): toc_passey (Passey ΔlogR resistivity-sonic AND
resistivity-density, LOM/maturity input, user-editable non-source baseline; + Schmoker density-TOC;
calibrate to core TOC when present — seed baselines from the Techlog/IP TOC defaults, Tier A);
kerogen (kerogen volume + OM-corrected PHIT; share endpoints with the SandiMin organic preset from
#2); gip (free gas PHIE/Sw/Bg + adsorbed via Langmuir VL/PL at reservoir pressure; CBM branch:
critical desorption pressure + dewatering context); brittleness (from mineralogy tied to SandiMin
volumes, and from Young's/Poisson via DT+RHOB+DTS where present — reimplement elastics from
RockPhyEquations.py's cited primaries, Tier B).

UI/UX: most are plain modules → zero UI. ONE custom panel for the visuals forms can't do: a ΔlogR
overlay track (scaled resistivity vs sonic/density baseline, separation shaded) + a Langmuir
isotherm crossplot (adsorbed gas vs pressure with the reservoir-pressure marker). PlotCanvas/
readTheme; brittleness class as a FACIES_PALETTE block track with completion cutoffs shaded.

OPTIMIZATION: always surface the ΔlogR baseline + LOM as visible params (TOC is very sensitive) —
never hide; calibrate to core TOC and report the fit; cross-check free+adsorbed partitioning.
cargo test: Passey ΔlogR recovers a known TOC on synthetic R/DT; Langmuir returns VL at infinite
pressure and 0 at 0; brittleness monotone in quartz fraction.
```

### 8. Results-QC & Sw-method comparison dashboard ("optimization of results")
**IP-cleanliness:** Buckles/mass-balance/Sw-spread Tier A/B (aggregates existing methods).
**Fed by (Stage A):** density-neutron/Buckles chart overlays; the recon incoherence from #2, MC from #1.
**Depends on #1 + #2.**
```
[paste the architecture contract]

TARGET: NEW custom panel src/ui/resultsQcPanel.ts (buildResultsQcContent), registered like the
other panes (buildRenderer case + openSingleton + #results-qc-btn). Backend: reuse getCurveData,
runPaySummary, run_multimin recon; add a small Rust helper only if a cross-method metric needs it.
The "optimization of results" surface — one place that says whether an interpretation is
trustworthy.

CHECKS: Sw-METHOD SPREAD (Archie/Simandoux/Indonesia/Waxman-Smits/Dual-Water envelope per depth +
where they diverge — matters in fresh-water Mahakam sand); BUCKLES consistency (Sw·PHIE vs the
expected irreducible constant, flag violators above transition); RECON incoherence rollup (#2) +
net-pay CUTOFF sensitivity (small ± on Vsh/PHIE/Sw cutoffs → net-pay swing) + MC P10/P50/P90 (#1) —
one scorecard per zone; mass-balance/unity check (mineral volumes + PHIE sum to 1 within tol).

UI/UX: per-zone QC scorecard (traffic-light per check, theme-var colored via --warn/--accent/
--accent2 — NOT hard-coded red/green) + Sw-envelope track + Buckles crossplot (PlotCanvas); CSV
export via the dashboardPanel pattern; linked to appState.hoverDepth.

OPTIMIZATION: every flag names the failing check + interval, never a silent pass; respect the
active well group (filterByActiveGroup) for field-wide view; frontend-only where possible (no new
write path).
```

### 9. UI polish + visualization — take the UI to its peak
**IP-cleanliness:** expression (Tier D) — but IP lithology-pattern *names* + COLORREF ramp *values*
are Tier-A facts to redraw clean; never lift IP's actual bitmaps or screens.
**Fed by (Stage A):** IP 96 lithology-pattern names + palettes + `.plt`/`.trk` conventions.
```
[paste the architecture contract]

SCOPE: a polish pass across existing visual surfaces — NOT new petrophysics. Kit: plotCanvas.ts
(PlotCanvas, readTheme, attachZoomPan, attachResizeRedraw, colormapColor/viridis, FACIES_PALETTE);
WebGPU LogCanvasRenderer; Canvas-2D charts; inline-SVG dashboard; plotExport.ts (PNG/print) with
composite.rs owning true SVG/PDF. Increments are independently shippable.

INCREMENT A — theming consistency (highest ROI): hunt & remove every hard-coded hex bypassing the
theme (highlightsOverlay.PALETTE, well-diagram casing #5a5a5a/#333, perf #c0392b, fixed canvas font
colors) → route through readTheme/the 15 vars; tokenize typography (family + size scale read once);
QA every theme incl. branded palettes in light AND dark, screenshot before/after. Adopt the IP
96-pattern lithology-name taxonomy + COLORREF ramps as clean-redrawn SVG hatch + ramps (Tier A
names/values, Tier D assets — redraw).

INCREMENT B — visualization richness: continuous COLORBAR for viridis/ramp crossplots; scatter
TOOLTIP bubbles on hover (sample curve values at cursor); extend the existing marginal-density/box
opt-in overlay; ensure every chart has buildImageExportButtons AND routes to composite.rs for
vector SVG/PDF at print scale. Adopt IP `.plt`/`.trk` family-bound conventions for composite defaults.

INCREMENT C — interaction & linked brushing: rectangular/lasso BRUSH on crossplots pushing the
selected sample set to other panels (new appState.selectedSamples Observable; log view + histogram
highlight brushed depths); draggable cutoff polygon writing back to zone params.

INCREMENT D — accessibility & motion: focusable canvases with aria-label; keyboard pan/zoom +
arrow-key crosshair; restrained transitions (respect prefers-reduced-motion).

GUARDRAILS: keep LOGICAL-pixel discipline (fitCanvasBackingStore, CSS-px mouse math); keep zoom/pan
via attachZoomPan/ViewportRef; the WebGPU log view can't toDataURL — route its export through
Composite; verify against multiple themes + the vite harness, screenshot each increment.
```

## Build order (dependencies matter — sits inside master-sequence step 2 & 4)
1. **Rock typing (3)** before **SHF (4)** — SHF's per-rock-type fits need the RT curve.
2. **SandiMin organic preset (2)** before **Unconventional (7)** — the shale suite reuses the kerogen model.
3. **Monte Carlo (1)** and **SandiMin recon (2)** before **Results-QC (8)** — the dashboard aggregates them.
4. **Correlation+contact (6) increment 1** (verify+commit the already-built code) before anything else there.
5. **Autocorrelate (5)** and **UI polish (9)** are independent — anytime; UI polish increment A
   (theming) is the cheapest high-value win in the whole set.
Foundational: the **alias-catalog + endpoint + chart adoptions (Stage A)** underpin the fixtures and
seeds all of the above use — do them at the start of master-sequence step 2.

## Reusable acceptance bar (repeat in every build session — enrichment AND graduated briefs)
- `npx tsc --noEmit` clean, `cargo check` clean, solver `cargo test` passes.
- A REVIEW.md entry with a concrete "Try:" line on real well data (BLSO / `Testdata.las`).
- Self-QC against `docs/qc_audit_prompt_template.md` before declaring done.
- Tier-B methods carry a primary-source citation in a code comment; Tier-C never built.
- Report leads with outcomes (numbers that changed), then proposes the next increment; wait for
  "go ahead."

---

## Notes (outside the prompt)

- **What the merge creates that neither file had:** (1) IP-cleanliness now governs the *build* work,
  not just the design work — every enrichment prompt carries a tier and the contract enforces it;
  (2) the **Cross-Map** wires each extracted asset to the build prompt it feeds, so the build seeds
  from the ingests instead of hand-deriving Pittman tables, MC defaults, TOC baselines, endpoints,
  etc.; (3) the **master sequence** encodes the enrichment-first-gated-by-triage decision; (4) one
  **architecture contract** and one **acceptance bar** now cover both enrichment prompts and
  graduated maturation briefs (The Bridge).
- **Judgment calls (reverse freely):** I kept the webinar stream optional; I treated enrichment #7
  (unconventional) as an explicit v1 exception to the "unconventional = intelligence, not v1" scope
  rule, since it's a user-requested build; I put the smectite-density review as a hard gate before
  Prompt 2's recon QC. If you'd rather keep DECIDE and BUILD as two separate files, this playbook is
  the index — the two sources remain runnable on their own.
- **Skill candidate:** once this stabilizes, it's the `sandibumi-dev` skill — the skill owns the
  tier system + staged format + architecture contract; a runbook owns the per-domain/per-increment
  loop. Any future vendor ingest reuses it unchanged.
```
