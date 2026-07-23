# B — Gap Matrix + AI-Dependency Audit (steering document)

Source deck: PHE OSES SONAR Hackathon 2026 PoC (extraction:
`D:\XX. Clauding\knowledge-base\tech-kb\sonar_phe_oses_hackathon2026.md`; page cites below refer
to that deck). Confidential-derived, local-only.

Doctrine (binding): our counterpart runs fully offline with **zero AI at runtime** — classical
algorithms only, deterministic and auditable, every automatic decision carries a rule ID,
re-indexing reproduces bit-identical results. CPU/GPU parallelism allowed for deterministic
workloads. AI at build time only. Scale point: 100k–1M files, one workstation, single binary +
SQLite.

Architecture direction is **DECIDED** (Jauhar, 2026-07-23): **standalone first** — see §3 for the
boundary contract.

---

## 1. Capability-by-capability audit

Verdict legend: **ADOPT** (already deterministic — take as-is), **REPLACE** (deterministic
replacement designed in targets C/D/E), **HUMAN-LOOP** (no full deterministic replacement —
honest fallback in target F), **DROP** (LLM-only luxury we don't need).

| # | SONAR capability (page) | (1) Genuinely AI-dependent? | (2) SandiBumi today | (3) Verdict |
|---|---|---|---|---|
| 1 | Recursive scan + **SHA dedup** vs FileIndex (p10) | No — pure hashing/IO | Nothing equivalent; [ingest.rs](../../../src-tauri/src/ingest.rs) imports individually selected files (`import_las_files`, `import_scal_files`, …), no corpus scanner, no hashing | **ADOPT** → target E indexer |
| 2 | Well Master, 2,418 wells × 207 attrs (p7) | No — a curated table | No well master; wells exist per-project in SQLite ([db.rs](../../../src-tauri/src/db.rs)), locations via `import_locations_file` (ingest.rs) | **ADOPT** shape → target C §1 |
| 3 | **WellAliasService** + regex well/field pre-filter (RAMD-14 ↔ RAMOS DELTA-14, p12) | No — normalized-name matching + alias tables | Nothing; single highest-leverage missing piece | **ADOPT** → target C §1 (alias resolution service) |
| 4 | Mnemonic dictionary 492 rows → 218 standards, 11-col schema (p8) | No — a lookup table; only its authoring was manual | Hardcoded aliases scattered in [parsers.rs](../../../src-tauri/src/parsers.rs) LAS import + Jauhar's alias list in [workflow_standards.md](../../workflow_standards.md); Techlog/Geolog/IP catalogs already extracted (techlog_ingest A_*.json: 723 aliases, 2,181 families, 2,839 family assignments) | **ADOPT** schema, out-populate ~10× → target C §2 |
| 5 | **LAS fast path** — deterministic header/curve parse, skip LLM, 1–3 s/file (p10, p15) | No — SONAR itself calls it deterministic; the 1–3 s is embedding overhead | **Stronger already**: parsers.rs full LAS parse (all curves, data), [dlis.rs](../../../src-tauri/src/dlis.rs) DLIS header parse, `import_all_curves_into_generic_store` (ingest.rs) | **ADOPT** ours; add curve-QC stats (P3/P97, nulls, step, spikes) SONAR never computes → target E |
| 6 | LAS auto-categorization: MnemonicMap lookup + reverse map + curve-family grouping + LAS View (p15) | No | parsers.rs maps mnemonics at import but keeps no reverse map, no curve-availability matrix UI | **ADOPT** → targets C §2 + E §3 (curve-availability facet) |
| 7 | Filename fast path — pattern match + confidence threshold, ~200 ms/file (p10) | No — it's pattern matching; SONAR merely embeds the synthesized text afterwards | Nothing | **ADOPT** mechanics, drop the embedding step → target D cascade stage (c); ours runs in µs–ms |
| 8 | **LLM metadata extraction** — WellName/FieldName/DocDate/DocTitle/Summary via Qwen3 8B, 10–40 s/file (p10) | Partly — for born-digital files the same fields fall out of filename/path grammar + header sniffing + alias resolution; genuinely AI-ish only for scanned/opaque docs | Nothing | **REPLACE** with rule cascade (target D); scans routed to review queue, not guessed. Summary field: DROP (LLM-only) |
| 9 | Embedding classification — Fase A6 greedy clustering (0.7 cos + 0.3 Jaccard) + Fase B reference vectors + thresholds (p11) | Fase A6 yes (embeddings + LLM naming). Fase B is *engineer-authored categories + a similarity score* — the useful part (curated taxonomy, per-type threshold, verified/pending/flagged states) is process, not AI | Nothing | **REPLACE** with document-type registry + recognition cascade with rule IDs (target D); adopt the three states verbatim |
| 10 | Hybrid retrieval — BM25 + dense, RRF k=60 (p12) | BM25 no; dense yes. RRF is arithmetic | Nothing (SandiBumi has no document search at all) | **ADOPT** BM25 via SQLite FTS5; **DROP** dense leg; keep RRF only if we ever fuse >1 deterministic rankers → target E |
| 11 | Metadata **pre-filter before search** (p12, "special" #1) | No — SQL WHERE before FTS | n/a | **ADOPT** as the core search pattern → target E §3 |
| 12 | Small-to-big parent expansion (p12) | No — parent-pointer lookup | n/a | **ADOPT** (FTS5 hit → surrounding section/page context) → target E |
| 13 | Batched `IN()` metadata enrichment, no N+1 (p12) | No | db.rs already does batched queries for curves | **ADOPT** → target E |
| 14 | Citation tags `[FILE_ID \| well \| "Title"]` (p12) | No — string formatting | Report provenance exists in [report.rs](../../../src-tauri/src/report.rs) | **ADOPT** on result cards → target E §3 |
| 15 | "PATCH 11" deterministic shortcut bypassing LLM tool routing (p12) | Inverse — it's SONAR routing *around* its own AI | n/a | **ADOPT** the lesson: our entire query layer is "PATCH 11 everywhere" — typed queries, no router |
| 16 | Agent tools ListWells / GetWellDocuments / GetWellInfo / GetWellInfoStats / GetStats / GetFileDetail / ListAllFiles / ListCategories (p13) | No — SQL queries wearing agent costumes | Partial per-project equivalents in db.rs/[project.rs](../../../src-tauri/src/project.rs) | **ADOPT** as CLI/UI structured queries → target E §3 |
| 17 | FindNearestWells — coordinate math (p13) | No | [geo.rs](../../../src-tauri/src/geo.rs) + `import_locations_file`; [deviation.rs](../../../src-tauri/src/deviation.rs)/`materialize_tvd_curves` go further (trajectories, TVD) | **ADOPT** (we already exceed it) |
| 18 | QueryLasPoint — curve value at depth (p13) | No | [curves.rs](../../../src-tauri/src/curves.rs)/db.rs read curves routinely; [composite.rs](../../../src-tauri/src/composite.rs) renders them | **ADOPT** over the *indexed corpus* (new: answer without importing into a project) → target E §3 |
| 19 | Vision table reading of scans → JSON rows (p14) | **Yes — the one capability with no full deterministic replacement** | tools/chartdig (dash-tip vector extraction) proves the adjacent skill; no table capture | **HUMAN-LOOP**: template capture (XLSX/CSV) + optional OCR + physics validator + review queue → target F. Build-time Claude extraction allowed (never ships) |
| 20 | ToolResults rows {DataJson, RowOrder, IsEdited, IsVerified} + per-row engineer review (p14) | No — audit flags | [resultsqc.rs](../../../src-tauri/src/resultsqc.rs) has QC flags on results; no per-row provenance for captured lab data | **ADOPT** row schema + add per-row provenance (source file/page/method) → target F |
| 21 | Chat answer synthesis (Qwen3 30B, p12); auto-summaries, cross-well insight prose (p6 step 5) | Yes — irreducibly LLM | n/a | **DROP** — faceted exact queries answer the deck's own example ("how many wells have core data in Krisna Field?") faster and exactly |
| 22 | Cluster auto-naming (p11 Fase A6 step 4) | Yes | n/a | **DROP** — the registry (target D) is named by a human once, correctly |
| 23 | Semantic similarity search over prose (p6) | Yes | n/a | **DROP** with documented mitigation (target E §3: FTS5 boolean/phrase/prefix + facets); residual gap stated honestly in FINDINGS |
| 24 | ML roadmap use cases (p16) | Yes (but SONAR itself defers them to 2028) | [ml.rs](../../../src-tauri/src/ml.rs), [facies.rs](../../../src-tauri/src/facies.rs) already do deterministic clustering/regression in-app | Out of scope for the data tool; SandiBumi keeps its own compute |

## 2. Reverse rows — what SandiBumi has that SONAR lacks entirely

SONAR retrieves and tabulates; **it computes nothing**. The entire interpretation engine is our
categorical advantage:

| SandiBumi capability | Where | SONAR equivalent |
|---|---|---|
| Full petrophysical module chain (Vsh, porosity, Sw, perm, cutoffs) | [modules.rs](../../../src-tauri/src/modules.rs) (171 KB), [equations.rs](../../../src-tauri/src/equations.rs), [workflow.rs](../../../src-tauri/src/workflow.rs), [chain.rs](../../../src-tauri/src/chain.rs) | none |
| SSC / SSPW sand-silt-clay models | [ssc.rs](../../../src-tauri/src/ssc.rs) | none |
| SandiMin multimineral optimizer | [multimin2.rs](../../../src-tauri/src/multimin2.rs) (172 KB), [multimin.rs](../../../src-tauri/src/multimin.rs) | none |
| LRLC Sw methods (IMTS/RtC) | [lrlc.rs](../../../src-tauri/src/lrlc.rs) | none |
| Chart-overlay corrections (digitized vendor charts) | [neutron_charts.rs](../../../src-tauri/src/neutron_charts.rs), tools/chartdig | none |
| Monte Carlo uncertainty | [montecarlo.rs](../../../src-tauri/src/montecarlo.rs) (93 KB) | none |
| Sat-height / SHF fitting, rock typing, HFU, Lorenz, Thomeer | [satheight.rs](../../../src-tauri/src/satheight.rs), [shf_fit.rs](../../../src-tauri/src/shf_fit.rs), [rocktyping.rs](../../../src-tauri/src/rocktyping.rs), [hfu.rs](../../../src-tauri/src/hfu.rs), [lorenz.rs](../../../src-tauri/src/lorenz.rs), [thomeer.rs](../../../src-tauri/src/thomeer.rs) | none |
| Composite plots, report generation | composite.rs, [layout.rs](../../../src-tauri/src/layout.rs), report.rs | screenshots of grids |
| Python extensibility | [python_engine.rs](../../../src-tauri/src/python_engine.rs) | Python only *outside* the system (p16) |
| Physics knowledge as data (QC gates, tool-response constants) | docs/ (log QC gates, constants_verification) | none — SONAR verifies rows by eyeball only |

Consequence: SONAR's pitch is "find and tabulate so an engineer can compute elsewhere." Ours is
"find, validate, and compute in the same ecosystem." The physics validator (target F) is the
bridge nobody else has.

## 3. The boundary design (standalone-first, integration-ready)

Decision already made; what B specifies is the **contract** so integration needs zero rework.

### 3.1 Crate layout

```
sandi-arsip/                     (working name; separate repo/workspace)
├─ crates/
│  ├─ arsip-core/       library: scanner, hasher, registry, dictionaries,
│  │                    alias resolver, recognition cascade, validator, search
│  ├─ arsip-cli/        thin CLI: scan / index / query / validate / export
│  └─ arsip-ui/         thin Tauri UI: search, review queues, LAS view, capture grid
└─ schema/              versioned SQL DDL + rule tables (TOML/JSON)
```

- **arsip-core owns everything**; CLI and UI are shells. SandiBumi later adds
  `arsip-core = { path/git }` and/or opens the same `.db` — both supported by construction.
- No SandiBumi code changes now. Integration later = one dependency line + read-only queries.

### 3.2 Integration contract (stable from v1)

1. **One SQLite file** `arsip.db`, schema-versioned via `PRAGMA user_version` + a
   `schema_migrations` table. Additive migrations only within a major version.
2. **Single source of truth**: the super-dictionary, units dictionary, and well master live in
   `arsip.db` (targets C). SandiBumi **reads** them (mnemonic resolution at LAS import, alias
   lookup at well creation) and never writes them from app code.
3. **Read API surface** (crate functions, stable signatures):
   `resolve_well(name) -> Option<WellId>`, `resolve_mnemonic(mnemonic, context) -> Standard`,
   `search(query, facets) -> Vec<ResultCard>`, `files_for_well(well_id, type_filter)`,
   `curve_stats(file_id)`, `las_point(well, curve, depth)`, `validated_rows(template, well)`.
4. **Concurrency**: WAL mode; the tool is the only writer; SandiBumi opens read-only
   (`?mode=ro`). No cross-process write coordination needed, ever.
5. **Determinism guarantee as API**: every `TypeAssignment` and `WellLink` row carries
   `rule_id`, `confidence`, `state` (verified/pending/flagged — SONAR p11 states adopted) so any
   consumer can audit any automatic decision.
6. **Rules are data**: recognition rules, lexicons, validator ranges ship as TOML/JSON in
   `schema/`, loaded into the DB; upgrading rules never requires recompiling consumers.

### 3.3 What stays out of the core crate

UI review queues (UI crate), OCR integration (optional feature flag, target F), and anything
SandiBumi-specific (interpretation writes results to its own project DBs, not to arsip.db).

## 4. Steering summary

- **Adopt as-is** (deterministic already in SONAR): SHA dedup, well master shape, alias
  resolution, dictionary schema, LAS parsing, BM25, metadata pre-filter, small-to-big, batched
  enrichment, citation cards, three review states, per-row audit flags, coordinate math,
  LAS-point query.
- **Replace with rules**: LLM metadata extraction → filename/path grammar + header sniffing
  (D); embedding classification → type registry + cascade with rule IDs (D); semantic search →
  FTS5 + facets (E).
- **Human-loop**: vision table reading → template capture + OCR-where-rule-able + physics
  validator + review queue (F); build-time Claude extraction as the practical workhorse, never
  shipped.
- **Drop**: chat synthesis, auto-summaries, cluster auto-naming, dense retrieval.
- SONAR's own architecture concedes the case twice: it hard-codes a deterministic bypass
  ("PATCH 11", p12) because LLM routing was unreliable for structured queries, and it brands its
  LAS path "deterministic — faster, cheaper, reliable" (p10). We build the system those two
  admissions point at.
