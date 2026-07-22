# SandiBumi Maturation Prompt
### Converts THREE vendor-intelligence sources — the Techlog install ingest, the Interactive Petrophysics install ingest, and (optional) Geoactive competitive webinar intel — into IP-clean, roadmap-merged SandiBumi feature designs.

This supersedes the single-source `sandibumi_leapfrog_prompt.md` (which handled one competitive
webinar extraction). It keeps that prompt's proven scaffolding — the four-tier IP-cleanliness
system, the "re-derive don't port" doctrine, the staged triage→design→merge workflow, the
standing Tier-C register — and generalizes it to the real intelligence SandiBumi now holds:
two **fully-executed vendor install ingests** (concrete reference DATA + method identification,
already on disk) plus an optional webinar-competitive stream.

**How to use:**
1. All primary inputs already exist on disk (paths in §INPUTS). No placeholders to fill.
2. Paste everything between `=== PROMPT START ===` / `=== PROMPT END ===` into Claude Code.
3. First run: **Stage 0 + Stage 1** only (grounding + triage across all sources). Then run
   **Stage 2 one domain at a time** so each design pass gets full depth. Run **Stage 3 last**,
   once briefs exist. **Stage A (direct-adoption track) can run any time in parallel** — it is
   import-and-use Tier-A data work, not design work, and does not depend on the other stages.

---

=== PROMPT START ===

# ROLE & MISSION

You are the product-engineering brain for **SandiBumi**, a petrophysics-first geoscience
platform built by a solo developer (Jauhar, a working petrophysicist in the Mahakam Delta,
Indonesia) for Indonesian clients, starting with Pertamina-scale assets. SandiBumi's
architecture is a **two-agent design**: **Agent 1 — Data Conditioning** (large-scale
conditioning of 1000+ multi-vintage wells) and **Agent 2 — Petrophysical Evaluation**
(automated zonation, parameter identification, interpretation), backed by a **decision
playbook** and a **queryable parameter knowledge base** built from Jauhar's 50+ past projects.
The codebase (Rust/Tauri backend + TypeScript/Svelte frontend, DuckDB store) is at the
reliability-hardening stage; most of the app is shipped (see the roadmap). Validation runs
against a 1000+ well corpus with ground-truth interpretations from Interactive Petrophysics
(IP), Geolog, and Techlog.

Your mission: turn the vendor-intelligence sources below into **SandiBumi capabilities that are
functionally superior to — never copies of — the incumbents**, packaged to merge into the
existing roadmap without disrupting it. Two of the three sources are *install ingests*: they
give you exact reference DATA (catalogs, chart tables, mineral endpoints, parameter defaults)
and method identification, not just market signal. Use the data directly where it is
unprotected; re-derive the methods from primary literature where they are not.

# INPUTS (read the relevant FINDINGS before touching a domain)

**Stream 1 — Techlog 2018.2 install ingest** (executed; a copied resource tree, app not
installed here):
- Master report: `docs/research_2026-07/techlog_ingest/FINDINGS.md`
- Extract data: `docs/research_2026-07/techlog_ingest/*.json` + `D_python_algorithms.md`,
  `H_doc_index.md` (targets A–I: 2,181 families / 723 aliases, 112 charts, 31×20 mineral
  endpoints, Quanti workflow defaults, Elan theory equation chapter, RockPhyEquations.py, etc.)

**Stream 2 — Interactive Petrophysics 2025.3 install ingest** (executed; IP IS installed at
`C:\Program Files\IP2025`):
- Master report: `docs/research_2026-07/ip_ingest/FINDINGS.md`
- Extract data: `docs/research_2026-07/ip_ingest/*.json` + `D_tierC_register.md`,
  `D_formula_language.md`, `H_doc_index.md` (targets A–I: 315 aliases + Loglan/Elan/PowerLog
  bridges, 233 ASCII charts, 61 modules with defaults, MINDEF 30-mineral endpoints, `.plt`/`.trk`
  templates, IP object model + formula/ip2py on-ramp, Tier-C register).

**Stream 3 — Geoactive competitive webinar extraction** (OPTIONAL / may not exist yet):
- `docs/competitive/IP_development_extraction_geoactive_webinars.md` — market signal about IP's
  *direction* and pain points. If present, cite its § numbers. If absent, skip Stream-3-only
  rows and note them as `TBD (awaiting webinar extraction)` — do NOT invent its content.

**Merge target & context (never silently rewrite):**
- Roadmap: `ROADMAP.md` (repo root) — authoritative plan, three buckets **Done / Open / Future**,
  three label families **Phase 1…12 / Severity(Critical→Low) / Wave A…E**. You propose diffs
  against it.
- Audit/review debt: `AUDIT-2026-07-20.md`, `AUDIT-2026-07-21-full-qc.md`, `REVIEW.md`.
- SandiBumi house method notes: `docs/method_ssc_sspw.md`, `docs/method_lrlc_rtc_imts.md`,
  `docs/workflow_standards.md`, `docs/multimin_ref_spec.md` (SandiMin endpoint spec),
  `docs/research_2026-07/*` wave specs. Jauhar's field standards (GRN P3/P97, LRLC IMTS/RtC,
  SSC/SSPW) are the calibration bar for "does this fit his practice".

# THE THREE STREAMS DIFFER — TREAT THEM DIFFERENTLY

This is the core reason this prompt exists. Do not flatten them.

| | Install ingests (Streams 1–2) | Webinar intel (Stream 3) |
|---|---|---|
| **What it is** | Exact reference DATA + identified methods from the shipped product | Market signal about needs, direction, positioning |
| **Primary use** | *Adopt* the data (Tier A) / *re-derive* the method (Tier B) / *avoid* the patent (Tier C) | Decide *what* to build and *how good* it must be |
| **Confidence** | High — verbatim files, cross-checked | Medium — presenter framing, auto-caption uncertainty |
| **Failure mode to avoid** | Copying protected expression/algorithm; treating a vendor UI default as field-authoritative | Inflating v1 scope with the incumbent's breadth |

The install ingests unlock a capability the single-source leapfrog prompt did not have:
**direct-adoption of unprotected reference data** (see Stage A). Not everything needs a design
brief — a curve-alias catalog or a neutron chart table is import-and-use.

# NON-NEGOTIABLE IP-CLEANLINESS RULES

Every capability you touch is classified into exactly one tier; the tier's rules are absolute.
If a tier is ambiguous, say so and take the **more restrictive** tier.

**Tier A — Reference data & market intelligence (free to use).** Curve-alias/family/unit/
lithology catalogs, chart lookup tables, mineral-endpoint values, parameter defaults, naming
bridges, palettes, template *conventions*, casing/hole reference tables, feature categories,
workflow pain points, QC culture, benchmarks. From the install ingests these are **extracted
verbatim and adoptable directly** — they are unprotectable facts, not expression. (Caveat: a
vendor's UI *default value* is a seed, not a field-calibrated truth — SandiBumi must let the
user override per well.)

**Tier B — Published, citable science (implement from the primary source only).** Methods with
an open literature trail: Archie / Waxman-Smits / Simandoux / Indonesia(Poupon-Leveaux) /
dual-water, Thomas-Stieber, Gassmann fluid substitution, Amaefule hydraulic flow units,
Winland/Pittman/Lucia r35, Batzle-Wang fluids, Gardner/Faust, Greenberg-Castagna,
Eaton/Bowers pore pressure, Passey ΔlogR, Larionov/Clavier/Stieber Vsh, Bassiouni, the
Quanti.Elan incoherence/conductivity equations, etc. **Rule:** identify the method + its
constants from the ingest, then implement from the **primary paper/textbook**, cite it in the
brief and in code comments, and never treat the vendor's wording/code as the spec. Both ingests
document methods this way (Techlog compiled `.pyc` + equation-image docs; IP compiled
`UserProgram.dll` + open SDK examples) — you get the citation and defaults, you write the code.
If you cannot name a primary source, the method drops to Tier C.

**Tier C — Patented or proprietary (do not implement, approximate, or reverse-engineer).**
Confirmed present and already registered from the IP ingest (`ip_ingest/D_tierC_register.md`):
- **Omovie `SonicSaturation`** — US Patent 12,242,011 B2 (acoustic-Sw). *Even a Tier-B design-
  around must clear counsel before any acoustic-Sw feature.*
- **Domain Transfer Analysis (DTA)** — proprietary transfer-learning (`DomainTransferAnalysis.exe`).
- **Experienced Eye** — proprietary feature selection (`ExperiencedEye.exe`).
- **Entropy-based borehole-image speed correction** — stated patented (Image Tools engine).
- **Shipped Neural-Network weight sets** — treated as proprietary artifacts (method may be
  Tier-B, the *weights* are not).
- (Plus any Recall-library / license-in candidate.)
**Rules:** (1) never implement/approximate/reverse-engineer these; (2) where the user-need
matters, design an alternative from Tier-B methods only, and document why it is methodologically
distinct; (3) where licensing-in could make commercial sense, record a `license-in candidate`
(a business decision, not an engineering task); (4) tag every brief in a patent's neighborhood
`LEGAL-REVIEW`. You are not a lawyer; **flag, don't conclude.** Maintain the consolidated
register as a standing file (§Stage 3.4), seeded from `ip_ingest/D_tierC_register.md`.

**Tier D — Expression (never reproduce).** UI layouts, screen compositions, module names, menu
structures, marketing/doc prose. Name SandiBumi capabilities in SandiBumi's own vocabulary. The
extracted template *conventions* (e.g. IP's family-bound `.plt` grammar) are Tier-A facts; the
literal screens and wording are Tier D.

# DESIGN DOCTRINE — what "better than the incumbent" means

IP and Techlog are excellent **interactive desktop** products: one human, one decision, one
click, automation bolted on and always caveated. SandiBumi is an **agentic, corpus-scale**
product. Therefore:

1. **Re-derive, don't port.** Go back to the petrophysical job-to-be-done and design the
   *agentic* solution from first principles. If a design reads like "IP's dialog, but
   automated," start over from the job.
2. **Automate the decision, generate the evidence.** Every automated action (depth shift,
   splice, normalization, zonation, parameter pick, model/endpoint choice) emits: the decision,
   its inputs, a confidence measure, the QC artifact a petrophysicist would have looked at
   (histogram/crossplot/before-after), and an exception-queue entry when confidence is low.
3. **Batch-first, override-live second.** Corpus-wide execution is the default; interactive
   parameter override on batch results is the review layer. Match the incumbents' interactivity
   *bar* (live recalculation on drag) — but only in the review layer.
4. **Beat the honest ceilings.** Where a source reveals a real limitation (row-wise random
   train/test splits that leak spatial autocorrelation; sample caps; single-well tools later
   multi-well-ified; desktop/Windows binding; one-zone-per-well defaults; core defaults locked
   in a compiled exe), design past it: blind-well/stratified validation by default, no arbitrary
   caps, multi-well-native, automated zonation as flagship, open/portable defaults.
5. **Decisions are assets.** Every parameter choice, trend, cutoff, zonation, endpoint set, and
   model is exportable, reloadable, and written into the decision playbook / parameter KB —
   SandiBumi's version of the incumbents' template-and-catalog moat, one level up.
6. **Design for the founder's market reality.** Multi-vintage messy Indonesian data, low-
   resistivity low-contrast pay, thin beds, carbonates and complex lithologies,
   IP/Geolog/Techlog-trained users, LAS/CSV round-trips as the ecosystem minimum, fresh Mahakam
   muds (SP suppression), Malay-basin sand-silt-clay.
7. **Scope discipline.** SandiBumi v1 is openhole petrophysics with the two agents. Pore
   pressure / geomechanics, rock physics, NMR, image logs, cased-hole/C-O, production logging,
   WITSML real-time, CO2/CCS are *intelligence*, not v1 — classify them Later or Out unless the
   roadmap says otherwise. Both ingests are full of these (IP's 50/61 modules are pore-pressure/
   geomech; Techlog has geomech/NMR/image families) — do not let their breadth inflate v1.
8. **Solo-dev honesty.** Every design carries an effort tier (S/M/L) sized for one developer
   with AI-assisted coding. Prefer designs that reuse SandiBumi's existing infrastructure
   (module runner, workflow chains, python_engine subprocess, composite renderer, DuckDB
   versioned store) over new subsystems.

# STAGE 0 — GROUNDING (always run first)

Read `ROADMAP.md` and both ingest `FINDINGS.md` files fully. Then output, in under 250 words:
(a) your understanding of SandiBumi's current scope and phase structure from the roadmap (name
the Done/Open/Future buckets and where reliability-hardening sits); (b) which ingest streams are
actually present (confirm Stream 3's file exists or mark it TBD); (c) any assumption you must
make because the roadmap is silent. If the roadmap and this prompt's context conflict, STOP and
ask — do not guess.

# STAGE A — DIRECT-ADOPTION TRACK (Tier-A reference data; run any time, parallel to all stages)

This stage does not exist in the single-source leapfrog prompt. The install ingests contain
unprotected reference data that is **import-and-use, not design work.** Produce
`docs/competitive/direct-adoption-register.md`: a checklist, each row =

`| Data asset | Source file(s) | SandiBumi consumer (module/table/file) | Adoption action | Effort | Verify |`

Cover at least these (from the two FINDINGS' shortlists — verify against the JSON, don't trust
the summary):
- **Curve-alias catalogs** — Techlog 723 aliases / 2,181 families (`techlog_ingest/A_*.json`) as
  the primary importer seed; merge IP's ~302 non-overlapping vendor-channel names
  (`ip_ingest/A_curve_alias.json`) as an additive layer → SandiBumi `ingest.rs`/`parsers.rs`.
- **Cross-tool naming bridges** — IP's Loglan(`.lls`)→VB (82 builtins), Elan↔IP, PowerLog→IP
  (`ip_ingest/A_naming_bridges.json`); Techlog's `Geolog.xml` — for legacy-module migration and
  cross-tool model import.
- **Vendor charts** — Techlog's 112 parametric charts + IP's 233 ASCII `.neu`/`.ovl`
  (`*/B_*.json`) → `tools/chartdig` / `neutron_charts.rs`; note which retire hand-digitization
  and which cross-validate already-digitized charts (Por-4/5/13/14, CP-16, M-N/MID).
- **Mineral endpoints** — the three-way reconciled table (IP MINDEF vs Techlog QM vs SandiMin,
  `ip_ingest/E_threeway_endpoint_compare.json`) as a selectable alt endpoint library +
  sensitivity set for `multimin2.rs`/SandiMin; carry the agree/diverge verdicts.
- **Parameter defaults** — Techlog Quanti (`techlog_ingest/C2_method_defaults.json`) + IP
  MonteCarlo/Gassmann/Poisson/FluidSub defaults (`ip_ingest/C_toplevel_par_defaults.json`) as
  dialog/MC-engine seeds. **Flag:** IP does NOT ship core openhole defaults (Archie a/m/n, Rw,
  Vsh coeffs) — those come from Techlog + primary literature, not IP.
- **Presentation** — IP's family-bound `.plt`/`.trk` grammar + `Composite CPI.plt` default +
  96-name lithology-pattern taxonomy + COLORREF ramps (`ip_ingest/F_*.json`) → `composite.rs`.
- **Data-model hardening** — IP object-model provenance columns + `DepthReferenceType` enum +
  built-in percentiles (`ip_ingest/G_datamodel_schemas.json`) → DuckDB schema.
- **Regression fixtures** — Techlog BLSO Balam South 36 LAS (real Mahakam-adjacent) as primary;
  IP `Testdata.las` synthetic as a compact LAS-parser stress fixture (EULA-check before
  bundling either).

Each row still respects the tiers: adopt the DATA, not any adjacent Tier-C algorithm. This
register is the "fast wins" backlog — most rows are effort **S**.

# STAGE 1 — CAPABILITY TRIAGE (run once, across ALL present streams)

Walk **every** capability surfaced by the two ingests (use the FINDINGS §1 per-target results
and §2 shortlists as the spine) plus, if present, every Stream-3 webinar capability. For each,
one row:

`| Source ref | Capability (SandiBumi wording) | Verdict | Agent | Tier | One-line reason | Roadmap anchor |`

- **Source ref** = `TL:<target/file>` (Techlog), `IP:<target/file>` (IP), or `WB:§x` (webinar).
- **Verdict** ∈ {`Adopt` (Tier-A direct data → belongs in Stage A, list it but don't brief it),
  `Parity` (table-stakes SandiBumi must have), `Leapfrog` (build meaningfully better — the
  differentiators), `Later` (real, post-v1), `Out` (not SandiBumi's game),
  `Tier-C-blocked` (register only)}.
- **Agent** ∈ {A1-conditioning, A2-evaluation, Platform, KB}.
- **Tier** = A/B/C/D of the *method* involved, with `LEGAL-REVIEW` appended where applicable.
- **Roadmap anchor** = existing roadmap item (name the Phase/Wave/Severity), or `NEW`.

Close Stage 1 with: (1) the top-5 Leapfrog candidates by (differentiation ÷ solo-dev effort);
(2) the full Adopt list (pointer to Stage A); (3) every Tier-C item (pointer to the register).
Write to `docs/competitive/maturation-triage.md`. Do not proceed to Stage 2 in the same run
unless told to.

# STAGE 2 — LEAPFROG / PARITY DESIGN BRIEFS (run per domain, on request)

For the domain named (e.g. "Stage 2 on saturation" or "Stage 2 on TL/IP charts"), produce one
brief per `Leapfrog` item and per `Parity` item that needs design work. Every field mandatory;
no invented numbers; mark unknowns `TBD`:

```
## <SandiBumi capability name>
- Source ref: TL:… / IP:… / WB:§… (cite the exact ingest file or § — traceability)
- Job to be done: <the petrophysical job, one sentence>
- Incumbent baseline: <=2 lines — what IP/Techlog does and its limitation, from the ingest>
- SandiBumi design: <re-derived agentic design: pipeline position, inputs, decision logic,
  evidence artifacts emitted, exception criteria, override surface>
- Why measurably better: <specific testable claims — corpus-wide per-well logs, blind-well
  validated, no sample cap, portable defaults — no adjective without a metric or mechanism>
- Method provenance: <Tier-B primary citations by author/year; or "Tier A — reference data,
  no protected method"; endpoints/defaults sourced from which ingest file>
- IP-cleanliness: <tier + rationale; design-around notes if Tier-C-adjacent; LEGAL-REVIEW flag>
- Data requirements & dependencies: <curves, corpus needs, KB entries, other SandiBumi modules>
- Validation criterion: <exact test vs the IP/Geolog/Techlog ground-truth corpus — what number/
  plot proves it; use the BLSO fixtures where possible; what "as good or better than the
  ground-truth interpretation" means here>
- Playbook/KB hook: <what decision/parameter this writes into the knowledge base>
- Effort: S / M / L (solo dev, AI-assisted) + main risk
```

Write briefs to `docs/design/leapfrog-briefs/<slug>.md`. Design documents only — no
implementation code unless explicitly asked.

# STAGE 3 — ROADMAP MERGE (run last, once briefs exist)

Produce `docs/roadmap/ingest-informed-additions.md` containing:
1. **Proposed additions/changes as a diff-style list** keyed to the roadmap's own bucket +
   label names: `ADD <item> to <Phase/Wave>`, `RESHAPE <existing item>: <change>`,
   `DEFER <item> to Future`, `ADOPT <Tier-A asset> (Stage A)`. Never a rewritten roadmap.
2. **Conflicts & open questions** — anywhere a proposal tensions with a roadmap commitment
   (scope, sequence, effort); state the tension, give Jauhar the decision. Do not resolve
   strategic conflicts yourself.
3. **A sequencing suggestion** for the top-5 Leapfrog items relative to the reliability-
   hardening stage, with dependency reasoning (and which Stage-A adoptions unblock them).
4. **The consolidated Tier-C register** (seed from `ip_ingest/D_tierC_register.md`): every
   patented/proprietary item, its design-around or license-in status, and its LEGAL-REVIEW
   state — a standing file to hand to counsel.

# OPERATING RULES (all stages)

- Cite the exact ingest file / § for every claim; cite primary literature for every Tier-B
  method. The FINDINGS summaries are a map — **verify against the JSON before relying on a
  number** (the critic pass flagged that summaries can drift from the data).
- Never invent parameters, thresholds, or performance numbers. `TBD` is information.
- One capability = one brief; don't merge distinct jobs to save space.
- A vendor UI default is a seed, not ground truth — always expose it as a per-well override.
- Respect the honest gaps the ingests recorded: IP has no core openhole defaults; EnvCorr/
  Acoustics math is compiled (Techlog) → corrections come from published chartbooks; both
  ingests' module kernels are compiled → method-ID only.
- Carry forward, don't launder, any auto-caption or best-effort-parse uncertainty (e.g. the
  `.mdl` binary endpoints, the 5%-agreement denominator convention).
- Plain, dense technical prose. No marketing language in briefs.
- If context runs short mid-stage, stop at a clean boundary, write what's complete, and state
  the resume point.

=== PROMPT END ===

---

## Notes (outside the prompt)

- **What changed from the leapfrog prompt, and why.** The leapfrog prompt assumed a single
  competitive-webinar extraction that does not exist on disk yet. This version makes the two
  **executed install ingests the primary, concrete inputs** and keeps the webinar stream as an
  optional third input. The biggest structural addition is **Stage A (direct-adoption track)**:
  install ingests yield unprotected reference DATA (alias catalogs, chart tables, endpoints,
  defaults, bridges, templates) that is import-and-use, which a design-brief-only workflow would
  wastefully treat as design work. Tier A was widened from "market intelligence" to "reference
  data & market intelligence" to cover it. The Tier-C register is now **pre-seeded** from the IP
  ingest rather than assembled from clues.

- **The three-way endpoint validation is a genuine asset**, not just a QC note: IP's `MINDEF.PAR`
  independently corroborates SandiMin's core matrix endpoints (a two-witness confirmation), and
  it surfaced one real SandiMin review item (smectite density looks dry-grain vs both wet-clay
  libraries) — flagged as a separate background task, not folded into this prompt.

- **Judgment calls I made (you invited "its okay if u not confident which part to include"):**
  (1) I kept the webinar stream *optional* rather than dropping it, so the prompt still works if
  you produce that extraction later. (2) I anchored Stage 3 to the real `ROADMAP.md` bucket/label
  vocabulary (Done/Open/Future; Phase/Severity/Wave) instead of the leapfrog prompt's generic
  "phase/section". (3) I split "Adopt" out as its own verdict + stage because ~half of what the
  ingests found is data, not design. If you'd rather keep it as a pure design-brief prompt (no
  direct-adoption track), delete Stage A and the `Adopt` verdict — the rest stands.

- **Things worth addressing that are NOT in this prompt** (deliberately, to keep it a *prompt*):
  the SandiMin smectite-density review (separate task), an actual EULA check before bundling any
  IP/Techlog sample LAS as a fixture, and counsel review before any acoustic-Sw feature (Omovie
  patent). These are actions for Jauhar, flagged in the FINDINGS, not steps for the design agent.

- **Natural next step:** once this prompt stabilizes over a few runs, it is the skill candidate
  the leapfrog notes anticipated — `sandibumi-maturation` alongside `petro-source-extractor` and
  the two ingest prompts (`techlog_ingest_prompt.md` + the IP ingest workflow). The skill would
  own the tier system + staged format; a runbook owns the per-domain loop. Any future vendor
  ingest (Geolog webinars, a newer Techlog) reuses it unchanged.
