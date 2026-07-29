# SegaraBumi — build prompt (study SONAR, design a deterministic AI-free O&G data tool)

A reusable prompt for ingesting the **Pertamina PHE OSES "SONAR" hackathon deck** and
designing **SegaraBumi**: an oil-and-gas data-management system that — unlike SONAR —
runs **fully offline** on the client's own hardware as **two tiers** — a mandatory
deterministic core (zero-AI, air-gap-capable, reproducible, auditable) plus an *optional*
local-LLM enrichment tier (small models on the user's laptop/PC, no server) — yet captures
**hundreds of thousands of files across the full breadth of O&G data types**. SONAR is the
study object, not the blueprint: SegaraBumi keeps its data model and pipeline discipline,
demotes every AI component to an optional local tier over a deterministic floor SONAR
lacks, and needs no GPU inference server. Unlike the Geolog-V14 / Techlog 2018.2 / IP 2025.3 ingests (install
trees mined for catalogs), the source here is a 20-page architecture deck — extraction is
fast; the value is the AI-dependency audit and the deterministic redesign, which this file
pre-loads.

**The name.** *Segara* (sea) + *Bumi* (earth): a sonar is sea-sounding technology, so
SegaraBumi sounds out what is buried in the earth's data — the deterministic, offline
Indonesian answer to SONAR. It is the sibling of **SandiBumi** (*sandi* = code/cipher) in a
deliberate `-Bumi` product family: **SandiBumi interprets the earth; SegaraBumi navigates
the data about it.** BUILD IT STANDALONE FIRST — SegaraBumi is a self-contained product
(sellable on its own, deployable at client sites), and SandiBumi later expands its
capability by consuming SegaraBumi's index/API. Design every target with that boundary in
mind: shared Rust/SQLite core, but no hard dependency from SegaraBumi back into SandiBumi.

**Source (verified on disk 2026-07-23):**
`D:\XX. Clauding\Materi_FINAL_ Model_Beta_SONAR_Hackathon_2026.pdf` — 20 pages, 5.3 MB,
"Digital Hackathon AI/ML Hulu Migas 2026: Proof of Concept", PHE OSES Data Management,
marked **Confidential**. Read with the pages parameter (max 20/request).

**What SONAR is (one paragraph, so the prompt stands alone):** PHE OSES manages 700k+
subsurface documents (Sunda & Asri Basin, ~226 structures) scattered across shared folders.
SONAR indexes them into a SQLite knowledge base (FileIndex / DocVectors / DocChunks),
classifies documents by embedding similarity, and serves an AI agent (Qwen3 30B via Ollama
+ Semantic Kernel, ASP.NET Core) with hybrid BM25 + dense retrieval (RRF fusion), a
2,418-well master table (207 attributes), a 492-row mnemonic dictionary (218 standard
names), vision-based table digitization (Qwen2.5-VL) with row-level verify flags, and a
deterministic LAS fast path. Claimed impact: search weeks → minutes; cost avoidance
Rp 53.26 B to 2038. Crucially for us: **SONAR's fastest, cheapest, most reliable parts are
already its deterministic parts** (SHA dedup, filename fast path ~200 ms, LAS parser that
"skips LLM", metadata pre-filter, BM25, well-alias resolution, verify flags) — the deck
itself is the evidence that the deterministic spine carries the system.

## Design doctrine (the non-negotiables for SegaraBumi)

1. **Deterministic core, optional local-LLM tier (TWO TIERS — this is the architecture).**
   Revised 2026-07-23: a *local, offline* LLM is now in scope (small models on the
   client's own laptop/PC/workstation, like SONAR's Ollama approach but no server). It is
   an OPTIONAL ENRICHMENT TIER on top of a mandatory deterministic core — never a hard
   dependency, never the source of truth:
   - **Tier 1 — deterministic core (mandatory, always runs).** Format parsers,
     regex/grammars, hashing, SQLite FTS5 (classical IR), rule engines, statistics, the
     dictionaries and well-alias resolver. Must run STANDALONE with no model present:
     reproducible (byte-identical re-index), auditable (every decision → a rule ID), and
     deployable in an air-gapped / AI-prohibited client environment. This tier alone is a
     complete, sellable product. CPU/GPU parallelism of these deterministic workloads is
     welcome for speed (bit-identical results required).
   - **Tier 2 — local-LLM enrichment (optional, degrades gracefully).** When a local
     model is available on the machine, it ADDS the capabilities determinism can't reach:
     semantic/natural-language search (local embeddings), scanned-document + table
     extraction (local vision model), free-text summaries and Q&A over prose. Everything
     here runs offline on the user's own hardware — no cloud, no external inference
     server. **Contract:** Tier-2 output is assistive and clearly labelled (source =
     model, low-confidence, review-required); it is written to SEPARATE columns/tables so
     it never overwrites Tier-1 deterministic facts, and it is flagged for human/physics
     validation before it becomes authoritative. If no model is present, every feature
     still works at Tier-1 quality — the app just shows fewer semantic extras.
   - **Why this beats SONAR outright:** SONAR *requires* a 30B model on a GPU inference
     server to answer anything. SegaraBumi answers structured questions with zero AI at
     millisecond latency AND, where a local model happens to be available, matches SONAR's
     semantic/vision features on the user's own laptop — no server, and with an auditable
     deterministic floor SONAR never has. Same doctrine as before ("SONAR's fast paths win
     by skipping the LLM"), now with the LLM added back only where it earns its place.
   - Build-time AI (Claude authoring dictionaries/rules/fixtures) remains fully in scope
     and separate from either runtime tier.
2. **Auditable.** Every Tier-1 classification and metadata decision traceable to a rule ID
   a petrophysicist can read, test, and override; re-running the deterministic indexer on
   the same corpus gives byte-identical results. Tier-2 LLM outputs carry a model/version
   + confidence stamp and live in their own columns — auditable *as model output*, never
   mistaken for a deterministic fact.
3. **Scale.** Design point: 100,000–1,000,000 files, dozens of formats, hundreds of
   document types, one workstation, single binary + SQLite file(s). SQLite + FTS5 handles
   millions of rows; no server, no deployment.
4. **Breadth.** The type system must cover the whole O&G data universe, not just what
   SandiBumi computes on today (see target D's taxonomy).

**How to use it:** run the master prompt below as a fresh Claude Code session. Targets A-B
are one session (do them first — B's AI-dependency audit steers everything); C-F are
independent design sessions; G closes. Outputs: target A goes to
`D:\XX. Clauding\knowledge-base\tech-kb\` (new KB branch for software/architecture ingests
— deliberate, create it); B-G go to `D:\XX. SandiBumi\docs\research_2026-07\sonar_ingest\`.

Domain knowledge lives in this repo's `docs/`, not machine-local memory — update this file
if the plan evolves.

---

## 1. The master prompt (copy, optionally trim to a subset of targets, run)

```
Ingest the Pertamina PHE OSES SONAR hackathon deck at
"D:\XX. Clauding\Materi_FINAL_ Model_Beta_SONAR_Hackathon_2026.pdf" (20 pages; read via
the pages parameter) and design SegaraBumi: a deterministic, AI-free-at-runtime O&G
data-management system for 100k+ files. SegaraBumi is a STANDALONE product (its own
Rust/SQLite binary, sellable and deployable on its own); SandiBumi (the petrophysics
application at D:\XX. SandiBumi) later consumes SegaraBumi's index/API to expand its
capability — so design for that one-way boundary (shared core, no SegaraBumi->SandiBumi
dependency). Full extraction note goes to
D:\XX. Clauding\knowledge-base\tech-kb\sonar_phe_oses_hackathon2026.md (create tech-kb\
if absent); all other outputs go to D:\XX. SandiBumi\docs\research_2026-07\sonar_ingest\.
Treat the PDF as strictly READ-ONLY.

Ground rules:
- DESIGN DOCTRINE (binding for every target): the system we design runs fully offline
  with ZERO AI at runtime — no LLM, no embeddings, no vision-language models, no trained
  ML classifiers, no inference server, no internet. Classical algorithms only (parsers,
  regex/grammars, SHA hashing, SQLite FTS5/BM25, rule engines, statistics). CPU/GPU
  horsepower IS allowed: parallelize deterministic workloads freely (multi-core
  scanning/hashing/parsing, GPU image ops for scan preprocessing), provided results
  stay bit-identical run-to-run. Deterministic and auditable: every automatic
  decision carries a rule ID; re-indexing the same corpus reproduces identical results.
  Scale design point: 100k-1M files on one workstation, single binary + SQLite.
  AI may be used at BUILD time only (authoring dictionaries/rules/fixtures with Claude);
  never in the shipped runtime. Where SONAR uses AI, design the deterministic
  replacement; where no full replacement exists, say so honestly and design the
  human-in-the-loop fallback — do not smuggle a model in.
- CONFIDENTIALITY: the deck is PHE OSES internal ("Confidential" footer on every page).
  All outputs stay local (knowledge-base and repo docs are local-only, same rule as
  project-kb). Never publish any of it externally (no Artifacts, no web). Adopt
  architecture CONCEPTS and schema SHAPES freely — never reproduce PHE OSES data content
  (well master rows, screenshots) into distributable assets.
- Cite deck page numbers for every extracted number/claim. The deck is a PoC pitch:
  treat numbers as indicative. Known internal inconsistency: p11 names BGE-M3 as the
  clustering encoder while p6/p10/p12 say Qwen3-Embedding 0.6B — note contradictions,
  don't paper over them.
- NO SandiBumi code changes in this session. Implementation happens later, serially, in
  the main working tree per house convention (build via vcvars 14.29 through PowerShell,
  never Git Bash — see docs/sandibumi_dev_playbook.md).
- Cross-reference the sibling ingests instead of re-mining them:
  docs/research_2026-07/techlog_ingest/ (family/alias/unit catalogs, charts, Elan),
  the Geolog-V14 anatomy work (alias.alias, .info manifests), and the IP 2025.3 register.

### Targets (one output file each)

A. FULL DECK EXTRACTION -> knowledge-base\tech-kb\sonar_phe_oses_hackathon2026.md
   Page-anchored architecture note capturing, at minimum:
   - Scale & scope (p2-3): 700k+ docs; data volumes CORE 1394 / RCAL 95 / SCAL 83 /
     PVT&HC 184 / DST 2716 / WATER 119; ~226 structures; Sunda & Asri Basin; source
     format list (PDF, scanned PDF, DOCX, XLSX, PPTX, LAS, CSV, TXT, JPG/PNG/TIFF, ZIP).
   - Stack (p6): ASP.NET Core MVC/Razor; Semantic Kernel (ISonarKernel, SonarAgentPlugin);
     Qwen3 30B via Ollama, Qwen3 8B, Qwen2.5-VL 7B, Qwen3-Embedding 0.6B (1024-dim) +
     BM25; SQLite. Mark each component AI vs deterministic — this feeds target B.
   - Well Master (p7): 2,418 wells x 207 attributes; key fields master_well_id,
     uwi_sonar, official_well_name, alias, field/structure/platform/basin, type, status.
   - Mnemonic dictionary (p8): 492 rows -> 218 standards; column schema
     (mnemonic_original, mnemonic_clean, mnemonic_standard, curve_family, curve_role,
     log_type, data_category, description, typical_units, unit_family,
     mapping_confidence); log_type counts (WL 193, ELAN 93, LWD 71, PLT 29, NMR 21,
     CBL 19) and curve_family counts.
   - Indexing flow (p9-10): SHA-hash dedup vs FileIndex; three routes — filename fast
     path (~200 ms/file, confidence threshold), LAS fast path (deterministic, skips LLM,
     1-3 s/file), document/image path (Vision OCR, 10-40 s/file); small chunks + parent
     chunks; LLM metadata extraction (WellName, FieldName, DocDate, DocTitle, Summary);
     tables FileIndex / DocVectors / DocChunks(+parent ref).
   - Classification (p11): Fase A6 unsupervised greedy clustering, similarity =
     0.7*cosine + 0.3*Jaccard(filename); Fase B engineer-reference categories via
     mean-pooled example vectors + per-category threshold (e.g. Mud Log 0.78), states
     verified/pending/flagged.
   - Retrieval (p12): well/field pre-filter (regex + WellAliasService, alias example
     RAMD-14 <-> RAMOS DELTA-14); BM25 + dense, RRF fusion k=60; small-to-big parent
     expansion; batched IN() metadata enrichment; citation tag injection
     [FILE_ID | well | "Title"]; "PATCH 11" deterministic shortcut bypassing LLM tool
     selection for dashboard-style queries.
   - Agent tools (p13): SearchDocuments, GetFileDetail, ReadDocument, ListAllFiles,
     ClassifyFile, ListCategories, GetStats; Well Intelligence: ListWells,
     GetWellDocuments, GetWellInfo, GetWellInfoStats, FindNearestWells, QueryLasPoint.
   - Run Tabulation (p14): doc -> per-page images -> Qwen-VL table read -> JSON ->
     ToolResults rows {DataJson, RowOrder, IsEdited, IsVerified}; engineer row review;
     RCAL worked example.
   - LAS auto-categorization (p15): header+curve parse, MnemonicMap lookup with reverse
     map, curve-family grouping, outputs las_header / CurvesStandard[] / DataJson /
     ReverseMap; LAS View review page.
   - ML roadmap, cost case, roadmap, team (p16-19): incl. digitization outsourcing
     budget USD 72/file and total claimed avoidance Rp 52.14 B / 12 yr.

B. GAP MATRIX + AI-DEPENDENCY AUDIT (the steering document) -> B_gap_matrix.md
   For EVERY SONAR capability, three columns of analysis:
   (1) Is it genuinely AI-dependent, or is the useful part deterministic underneath?
   (2) What SandiBumi has today (name the real file: ingest.rs, parsers.rs, modules.rs,
       multimin2.rs, composite.rs, equations.rs, neutron_charts.rs, python_engine.rs,
       tools/chartdig).
   (3) Verdict (four buckets, tier-aware): ADOPT-AS-IS (already deterministic: SHA dedup,
       LAS parser, BM25, metadata pre-filter, well-alias resolution, verify flags, batched
       enrichment — Tier 1) / REPLACE-WITH-RULES (LLM metadata extraction -> filename
       grammar + header sniffing; embedding classification -> document-type registry,
       target D — Tier 1, deterministic-first even though a model could do it, because
       reproducibility/audit matter here) / LOCAL-LLM-OPTIONAL (Tier 2 enrichment that
       degrades gracefully: semantic search via local embeddings on top of FTS5, target E;
       vision table/scan extraction via a local vision model feeding the physics
       validator, target F; free-text summaries and NL Q&A over prose) / DROP (only what
       serves nobody offline — e.g. cloud-dependent features). For EACH capability, state
       which tier it lands in and — critically — what the Tier-1 experience is when no
       model is present (the graceful-degradation contract). Nothing may become a hard
       LLM dependency; the deterministic floor must always answer.
   Include the reverse rows: what SandiBumi has that SONAR lacks entirely (a full
   interpretation engine — SSC/SSPW, SandiMin, LRLC Sw methods, chart overlays; SONAR
   retrieves and tabulates, it computes nothing). Architecture direction is DECIDED
   (Jauhar, 2026-07-23): the tool is named SegaraBumi and ships STANDALONE FIRST — its
   own Rust/SQLite binary/workspace, sellable and deployable alone — with SandiBumi
   integration later. So B must design the boundary, not the choice: a core library crate
   (indexer, registry, dictionaries, validator) + CLI + thin UI, with SandiBumi
   integrating afterwards by depending on the core crate and/or opening the same SQLite
   file — one-way only, no SegaraBumi->SandiBumi dependency. Specify the integration
   contract now (stable .db schema + crate API) so nothing needs rework at integration
   time; the super-dictionary and well master live in SegaraBumi as single source of
   truth, SandiBumi reads them.

C. DATA FOUNDATION: WELL MASTER + SUPER-DICTIONARY + UNITS -> C_data_foundation.md
   All-deterministic backbone; design schema DDL + build plan (no code):
   1) Well-master table adopting SONAR's shape (master_well_id, uwi, official name,
      aliases[], field/structure/platform/basin, type, status) sized for multi-project
      consultant work, plus a deterministic alias-resolution service (SONAR's
      WellAliasService, p12): normalized-name matching, alias tables, explicit
      collision handling — the single highest-leverage deterministic component in the
      whole design (it is what lets filename/metadata rules resolve wells without AI).
   2) Merged mnemonic super-dictionary: adopt SONAR's 11-column SCHEMA, populate from
      sources SONAR doesn't have: Geolog alias.alias + Techlog family/alias/unit XMLs
      (already extracted in techlog_ingest target A) + IP 2025.3 catalogs + Jauhar's
      aliases in docs/workflow_standards.md. Target >= 5,000 cross-vendor-validated
      mappings vs SONAR's 492 (~10x), per-row source + confidence + conflict-resolution
      rules for vendor disagreements.
   3) Units dictionary (SONAR has unit_family only as a column): unit names, aliases,
      conversion factors, unit-system sets — from Techlog SystemsUnits.xml et al.

D. DOCUMENT-TYPE REGISTRY + DETERMINISTIC RECOGNITION -> D_type_registry.md
   The deterministic replacement for SONAR's embedding classification (p11), and the
   heart of "capture the whole O&G data universe". Two parts:
   1) TAXONOMY — enumerate the full space of O&G data/document types as a hierarchical
     registry (category -> type -> subtype), covering at least: well logs (LAS 1.2/2/3,
     DLIS, LIS, ASCII exports, composite/CPI PDFs), core (RCAL, SCAL, core photos,
     core description, thin section, XRD/SEM), mudlog/masterlog, gas-while-drilling,
     well tests (DST, PLT, RFT/MDT pressure surveys), fluids (PVT reports, oil/gas/water
     analyses), drilling (daily drilling/geology reports, well plans, BHA, deviation
     surveys), completion (perforation records, completion diagrams, workover),
     wellheads/coordinates, geophysics (checkshot/VSP, SEG-Y nav, velocity), G&G
     studies (geological/petrophysical/reservoir reports, presentations, maps, grids),
     production/pressure histories, admin (AFE, WP&B, correspondence). Seed from the
     deck's own lists (p2 formats, p3 volumes, p6 sources) + project-kb's 46 delivered
     studies + petro-kb + the Techlog/Geolog/IP ingests. Aim for an initial registry of
     100-300 concrete types with stable IDs — extensible by data, not code.
   2) RECOGNITION CASCADE — per file, cheapest test first, all deterministic:
     (a) extension; (b) magic bytes / format header sniff (DLIS SUL, LAS "~V", SEG-Y
     EBCDIC reel header, ZIP/OOXML, PDF text-layer presence); (c) filename + path
     grammar: tokenize against the well-alias table (target C), date patterns, doc-type
     keyword lexicon — BILINGUAL, English + Indonesian ("Laporan Akhir", "Uji Kandungan
     Lapisan", "Analisa Inti Batuan", ...); folder-context inheritance (a file inside a
     well's DST folder inherits candidates); (d) born-digital content signatures: regex
     over extracted text (pdftotext-class extraction is deterministic; scanned PDFs
     detected by absent text layer get routed to target F's queue, not guessed);
     (e) weighted confidence score + threshold -> verified / pending / flagged states
     (adopt SONAR's three states, p11). Every match records its rule ID. Deliverable:
     registry + rule schema (data-driven: rules live in TOML/JSON tables, hot-loadable),
     seed keyword lexicon, worked recognition examples, and a review-queue workflow for
     pending/flagged files.

E. INDEXER + SEARCH AT 100k-1M SCALE -> E_indexer_search.md
   The runtime engine; design for one workstation, no server:
   1) Indexer: recursive scan with SHA-256 dedup vs FileIndex (adopt p10); incremental
     re-scan (size+mtime short-circuit, hash only changed files); ZIP descent; format
     parsers per registry type — LAS fully parsed (header, curves via super-dictionary,
     AND per-curve QC stats SONAR never computes: P3/P97, null fraction, depth
     range/step, monotonicity, spike/badhole flags) at < 100 ms/file vs SONAR's
     1-3 s/file (10-30x, no LLM); DLIS/LIS header-level parse; PDF text-layer
     extraction; DOCX/XLSX structured extraction; CSV/ASCII sniffing. Throughput
     budgets: full 100k-file cold index overnight on a laptop, incremental daily pass
     in minutes; state the per-route ms budgets, and design the scan pipeline for
     multi-core parallelism from the start (GPU offload where it genuinely helps,
     e.g. image-heavy routes — speed, never inference).
   2) Storage: SQLite schema — FileIndex, WellLink (file<->well via alias resolution),
     TypeAssignment (+rule ID + confidence + state), LasHeader, CurveStats, DocText
     (FTS5), plus dictionaries from targets C/D. Note FTS5 capacity/latency at 1M rows.
   3) Search: FTS5 (BM25) full-text + faceted filters (well, field, type, format, date,
     depth range, curve availability) with SONAR's own best trick promoted to the core:
     metadata pre-filter BEFORE text search (p12). Result cards carry citation-style
     tags [FILE_ID | well | type | "Title"] (adopt p12). Structured query API
     equivalents of SONAR's tools (p13): ListWells, GetWellDocuments, GetWellInfo,
     GetStats, FindNearestWells (coordinate math), QueryLasPoint (well, curve, depth ->
     value over indexed curves). Latency target: < 50 ms interactive on laptop. This
     Tier-1 layer is the DEFAULT and always present: faceted structured questions
     ("how many wells have core data in Krisna Field?") become one exact count query —
     faster AND exact where SONAR runs GPU inference to approximate it.
   4) Tier-2 semantic search (OPTIONAL, local embeddings — design the seam, not a hard
     dependency): when a local embedding model is available on the machine, add a dense
     vector index over chunks and fuse it with FTS5 via RRF (k=60, adopt SONAR p12) for
     natural-language/semantic recall over prose (the drilling-report "losses/LCM/total
     loss" problem FTS5 misses). Store vectors in a separate table keyed to chunks; if no
     model is present the search silently runs FTS5-only at full quality. Specify: the
     pluggable embedder interface, how results are marked (semantic vs exact), and that
     ranking stays explainable (show which tier surfaced each hit). Chat answer synthesis
     over retrieved context is likewise a Tier-2 optional feature — grounded, cited,
     labelled as model output — never the only way to get an answer.

F. DIGITIZATION: LEAN CAPTURE + PHYSICS VALIDATOR -> F_digitization_design.md
   Tier picture: table reading is the natural home of the OPTIONAL Tier-2 local vision
   model (SONAR p14's Qwen2.5-VL, but running on the user's own GPU, no server). It is
   allowed at runtime now — NOT only at build time. But it stays optional and its output
   is never trusted raw: it feeds the deterministic physics validator, which is the real
   differentiator SONAR lacks. SCOPE CONTAINMENT is still the design problem — this target
   must NOT balloon into per-report-type UI development:
   - Digitization is ON-DEMAND, per active study, never wholesale archive conversion.
     Size the real exposure from the deck's own numbers (p3): ~4,600 digitization-class
     docs in a 700k corpus (<1%). The indexer (D/E) finds documents; only what a study
     actually needs gets digitized.
   - ONE generic capture path, not N bespoke editors: report-type knowledge lives in
     DATA templates (column schema, units, expected ranges — keyed to target D's
     registry types: RCAL, SCAL, DST, PVT, water analysis), all rendered by a single
     spreadsheet-style grid. v1 may ship with NO in-app editor at all: XLSX/CSV
     templates are the entry surface (engineers already live in Excel) and SandiBumi
     builds only import + validate + flag + store. Design BOTH variants and estimate
     their effort so the choice is explicit.
   - Capture ladder, cheapest first: (a) classical OCR for typed/tabular scans where
     layout rules can be written (offline, GPU-accelerated if useful); (b) OPTIONAL
     Tier-2 local vision model for messy scans/complex tables OCR can't handle — same
     role SONAR gives Qwen-VL, on the user's own hardware, output routed straight into
     the validator; (c) structured manual entry as the always-available floor. If no
     vision model is present, (a)+(c) still fully work — that is the graceful-degradation
     contract. Every captured row records which rung produced it.
   - THE differentiator SONAR lacks: an automatic physics-validation layer on every row
     regardless of capture method — unit checks; physical ranges (porosity 0-45 v/v,
     grain density ~2.55-3.0 g/cc, perm > 0 log-normal, Sw 0-1); cross-row consistency
     (phi-k trend outliers, He-porosity vs density-porosity coherence, depth ordering
     vs cored interval); auto-flag failing rows so the engineer reviews flagged rows
     first; row-level IsEdited/IsVerified audit flags (adopt p14). The validator is
     rule TABLES over template columns — seed directly from the existing QC-gate and
     tool-response-constant specs in docs/ — shared infrastructure, days not months.
   - Per-row provenance: source file + page + capture method (OCR / local-vision-model +
     model version / manual / build-time AI).
   - Expected workhorse in practice: local-vision extraction (Tier 2) OR build-time AI
     extraction (Jauhar runs Claude on a scan) — either way the output lands in the same
     template -> physics validator -> review queue, and human effort shrinks to verifying
     the flagged rows — minutes per report, not retyping. Note SONAR's own flow ends the
     same way (engineer reviews
     IsVerified rows, p14): the human loop never disappears with AI, it just moves.
   Reuse lessons from tools/chartdig. Economics (p17): PHE OSES budgets USD 72/file for
   manual digitization — a validator-backed workflow at consultant quality is also a
   sellable service.

G. FINDINGS + BACKLOG -> FINDINGS.md
   - BEAT TABLE — one row per capability: SONAR baseline (deck number, page) -> our
     design -> mechanism -> honest factor. Seed rows (verify/adjust): infrastructure
     GPU inference server + Ollama + ASP.NET stack -> single offline binary + SQLite
     using the workstation's own CPU/GPU for parallel deterministic processing
     (no inference stack; deployment cost ~0, runs air-gapped at client sites); dictionary 492 rows -> >=5,000
     (~10x, three-vendor merge); LAS indexing 1-3 s/file summary-only -> <100 ms/file
     with full curve QC (10-30x + more information); doc metadata LLM-per-doc
     (10-40 s/file) -> rule cascade at ms/file for born-digital files (scans routed to
     review, not guessed); classification opaque embeddings + thresholds ->
     auditable rule IDs a petrophysicist can read and fix; reproducibility: identical
     re-index guaranteed vs model-dependent drift; search GPU semantic approximation ->
     exact faceted counts < 50 ms; tabulation verify-flags-only -> physics-validated
     rows (catches wrong data, not just unread data); citation doc-level ->
     parameter-level provenance via project-kb decision records; well intelligence
     lookup-only -> lookup + full interpretation engine (categorical).
   - GRACEFUL-DEGRADATION CONTRACT (replaces the old "what we lose" section, since a
     local LLM is now optionally in scope): for each Tier-2 capability — semantic search,
     NL Q&A, auto-summaries, vision extraction — state (a) the Tier-1 experience with NO
     model present (must be genuinely useful, not broken), (b) what the local model adds
     when available, (c) the residual gap vs SONAR's 30B-on-a-server. The headline: SONAR
     needs the GPU server to answer at all; SegaraBumi answers at Tier 1 always and
     reaches Tier-2 parity on the user's own laptop when a model is installed — with an
     auditable deterministic floor SONAR never has. Where SONAR's server approach is
     simply right for THEIR problem (700k files, many casual users), say so.
   - Ranked shortlist by value-per-effort (S/M/L) for Mahakam-delta/LRLC consultant
     workflows + the standalone-product angle, then an ADR-style backlog entry
     (context, decision, acceptance criteria) per survivor. Validation corpus for
     acceptance tests: the 6,668 pooled final-log LAS in
     D:\XX. Clauding\knowledge-base\project-kb + the 36 Techlog training LAS.
   - Adversarial pass before reporting: for each item try to refute "worth building"
     (single-user desktop scope? already covered by Techlog/Geolog/IP ingest findings?
     SONAR-scale problem we don't have?). Only survivors stay.
   - Deck-internal discrepancy list (e.g. BGE-M3 vs Qwen3-Embedding contradiction).
   - One paragraph on commercial reuse: SONAR's cost-avoidance framing (p17) as a
     template for pitching an OFFLINE data-management product to clients — "SONAR
     capability without the GPU server, deployable inside your firewall" — note
     SandiBumi already ships a Pertamina client theme.

Do NOT modify anything in D:\XX. SandiBumi outside the research output folder, and nothing
in the PDF's folder. Implementation happens later, serially, in the main working tree.
```

---

## 2. Calibration: SONAR component -> SegaraBumi's two-tier counterpart

| SONAR component (deck page) | AI? | SegaraBumi counterpart (Tier 1 = deterministic floor, Tier 2 = optional local LLM) | Target |
|---|---|---|---|
| SHA dedup, FileIndex, incremental scan (p10) | no | adopt as-is | E |
| Well Master 2,418 x 207 + WellAliasService (p7, p12) | no | adopt shape; alias resolver becomes the keystone | **C** |
| MnemonicMap 492 rows / 218 standards (p8) | no | super-dictionary >= 5,000 rows from Geolog + Techlog + IP + `docs/workflow_standards.md` | **C** |
| LAS fast path + QueryLasPoint (p10, p13) | no | full parse + per-curve QC stats, < 100 ms/file; 6,668-LAS corpus to validate | **E** |
| Filename fast path (p10) | partly (embeds result) | filename/path grammar + bilingual keyword lexicon + alias resolution — no embedding | **D** |
| Embedding classification, Fase A6/B (p11) | yes | document-type registry (100-300 types) + recognition cascade with rule IDs | **D** |
| LLM metadata extraction, 5 fields (p10) | yes | rule cascade (filename/header/content regex); scans -> review queue, never guessed | D |
| BM25 + dense + RRF retrieval (p12) | half | Tier 1: FTS5 BM25 + metadata pre-filter + facets (always on). Tier 2: optional local-embedding dense index + RRF fusion | E |
| Chat answer synthesis, agent tool selection (p12-13) | yes | Tier 1: faceted/structured queries (SONAR's own "PATCH 11" bypass shows the way). Tier 2: optional grounded/cited local-LLM answers | E |
| Qwen-VL Run Tabulation (p14) | yes | OCR + optional local vision model (Tier 2) + structured entry, ALL feeding the deterministic physics validator SONAR lacks | **F** |
| Interpretation/computation engine | — | SandiBumi's whole raison d'être; SONAR has none | B (reverse row) |

## 3. Practical notes

- A slide deck, not an install tree: target A is ~one sitting; B steers; C, D, E carry
  the most value (in that order), then F. G closes.
- `tech-kb\` is a **new** knowledge-base branch (petro-kb = literature, project-kb =
  decision records; neither fits a software-architecture ingest). Creating it is
  deliberate — future software/architecture ingests go there too.
- The strongest argument in our favor is inside their own deck: SONAR's fast paths are
  fast precisely because they skip the LLM (p10: LAS "skips LLM because deterministic
  parser already provides metadata"; p12: "PATCH 11 can bypass LLM tool selection").
  Our design is that insight taken to 100%.
- Offline/air-gapped is a selling point, not a limitation: O&G subsurface data is
  confidential by default (this very deck is stamped Confidential on every page), and
  client-site deployment with no server, no AI stack, and no data leaving the machine
  (any CPU/GPU it finds is used for speed, never inference) is a pitch no RAG stack can
  match.
- Bilingual (EN/ID) filename and keyword lexicons are mandatory for Indonesian archives —
  SONAR's deck is itself half Indonesian; a rules-based recognizer that only speaks
  English would miss half the corpus.
- Strategic note: PHE OSES is a Pertamina entity and SandiBumi ships a Pertamina theme.
  Compatible well-master / UWI conventions (target C) make any future PHE OSES
  engagement's data exchange trivial.
- The deck credits a petrophysicist (WOPDM team) — the mnemonic dictionary and RCAL
  tabulation choices read as practitioner-informed; treat those two as the most
  battle-tested parts of the design.
