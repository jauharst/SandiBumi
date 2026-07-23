# FINDINGS — SONAR Ingest → SandiBumi Offline Counterpart

Session 2026-07-23. Deck extraction: `knowledge-base\tech-kb\sonar_phe_oses_hackathon2026.md`
(local-only, confidential-derived). Designs: [B](B_gap_matrix.md) gap matrix,
[C](C_data_foundation.md) data foundation, [D](D_type_registry.md) type registry,
[E](E_indexer_search.md) indexer/search, [F](F_digitization_design.md) digitization.
Architecture decided: **standalone first** (companion tool, working name sandi-arsip), SandiBumi
integrates later via the core crate and/or shared `arsip.db` (contract in B §3).

---

## 1. Beat table

| Capability | SONAR baseline (deck) | Our design | Mechanism | Honest factor |
|---|---|---|---|---|
| Infrastructure | GPU inference server (Qwen3 30B, large VRAM) + Ollama + ASP.NET + Semantic Kernel (p6); VRAM-pinned P6000 batches (p10) | Single offline binary + SQLite; workstation's own CPU/GPU for parallel deterministic processing | No inference stack at all | Deployment cost ~0; runs air-gapped at client sites. Categorical, not a speedup |
| Mnemonic dictionary | 492 rows → 218 standards, hand-built (p8) | ≥ 5,000 cross-vendor-validated mappings (Techlog 723 aliases + 2,839 family assignments + Geolog alias.alias + IP + Jauhar) | Three-vendor merge with conflict rules (C §2) | ~10× rows; better provenance (per-row source + confidence). Fair caveat: their 492 are OSES-tuned; ours needs the same field-tuning loop (mnemonic_unknown queue) |
| LAS indexing | 1–3 s/file, summary-only, "skips LLM" but still embeds (p10) | < 100 ms/file with FULL curve QC (P3/P97, null %, step, monotonicity, spikes) | Parse everything, embed nothing (E §1.3) | 10–30× faster AND more information per file |
| Doc metadata | LLM per doc, 10–40 s/file (p10) | Rule cascade at ms/file for born-digital; scans routed to review, not guessed | Filename/path grammar + header sniff + alias resolver (D) | ~1000× on born-digital; on scans we honestly do LESS (no auto-metadata) — that residual is F's queue |
| Classification | Embeddings + thresholds, opaque (p11; encoder itself ambiguous — BGE-M3 vs Qwen3) | Registry + rule IDs a petrophysicist can read and fix | Noisy-OR evidence scoring, rule_trace per file (D §2e) | Comparable coverage on named types; infinitely more auditable; loses zero-config clustering (see §2) |
| Reproducibility | Model-dependent: re-index drift with model/prompt/version changes | Bit-identical re-index guaranteed (canonical-order writer, content-derived IDs) | E §1.1 determinism design | Categorical — this is the auditability product |
| Search | GPU semantic approximation + BM25 + RRF (p12) | Exact faceted counts + FTS5 BM25, < 50 ms laptop | Metadata pre-filter (their own best trick, p12) promoted to core (E §3.1) | The deck's own demo question (p4) resolves to one SQL count — faster AND exact |
| Tabulation | Vision-LLM rows + verify flags only (p14) | Physics-validated rows (units, ranges, phi–k trends, ion balance, log coherence) + verify flags | Rule tables seeded from docs/ QC gates + constants (F §3) | Catches WRONG data, not just unread data. But capture itself is slower without vision AI (human/build-time AI does the reading) |
| Citation | Doc-level tags [FILE_ID \| well \| "Title"] (p12) | Same tags + rule trace + per-row page-level provenance; joins to project-kb decision-record parameter citations | E §3.1, F §5 | Parameter-level vs doc-level |
| Well intelligence | Lookup tools (ListWells … QueryLasPoint) (p13) | Same lookups (E §3.2) + a full interpretation engine behind them (modules.rs, multimin2.rs, ssc.rs, lrlc.rs, montecarlo.rs) | SandiBumi integration | Categorical: SONAR retrieves and tabulates; it computes nothing |
| Dedup / scan | SHA + FileIndex skip (p10) | Same + duplicate topology recorded, incremental stat-walk re-scan, ZIP descent | E §1.2 | Adopt with modest improvements |

## 2. What we lose by dropping AI — stated plainly

| Lost | Mitigation | Residual gap |
|---|---|---|
| Natural-language Q&A over the archive | Structured queries + facets cover the deck's enumerable questions | Real: casual users can't type a sentence and get prose. For a single expert user (Jauhar) this costs little; for SONAR's audience (many casual enterprise users) it's the whole product |
| Semantic search over unstructured prose | Bilingual keyword lexicon + FTS boolean/phrase/prefix + facets | Real: conceptual paraphrase recall ("anomalous pressure behavior") has no deterministic equivalent; facets narrow to ~20 docs, human reads |
| Auto-summaries (DocTitle/Summary fields, p10) | Title from filename/header rules; no summary | Real but small: summaries are a convenience; citations were always the trustworthy part |
| Zero-config clustering of unlabeled corpora (Fase A6, p11) | Curated registry + review queue grows types by data | Real for a truly unknown corpus. In our domain the type universe is enumerable (D proves it in ~140 types); clustering was solving cold-start, which a domain expert doesn't have |
| Vision table reading of scans (p14) | Template capture + OCR-where-rule-able + build-time Claude + validator | The one genuine loss at runtime; contained to <1% of corpus (p3 arithmetic) and on-demand per study |

**Where SONAR's approach is simply right for THEIR problem:** a 700k-file enterprise archive
with many casual users, an on-premise GPU already justified, and no interpretation engine to
integrate with — RAG + agent tools is a sensible product there, and their deterministic spine
(dedup, alias service, mnemonic map, pre-filter, PATCH 11) shows the team already knows where
LLMs don't pay. Our doctrine isn't a refutation of SONAR; it's a different operating point:
one expert user, air-gapped client sites, auditability as a deliverable.

## 3. Ranked shortlist (value-per-effort) + ADR-style backlog

Adversarial pass applied first; survivors below, kills in §4.

### Rank 1 — Well master + alias resolver (C §1) — **S/M effort, highest leverage**
- *Context:* every downstream rule (filename grammar, WellLink, facets) needs deterministic
  well resolution; SONAR's WellAliasService is its single best deterministic idea (p12).
- *Decision:* build `well_master`/`well_alias` + WN/WL rule normalization in arsip-core first.
- *Acceptance:* ≥95% of the 6,668 project-kb LAS filenames resolve to correct wells, zero false
  merges; collisions produce queue rows, never silent picks; re-run deterministic.

### Rank 2 — Mnemonic super-dictionary + units (C §2–3) — **M effort**
- *Context:* three vendor catalogs already extracted (techlog_ingest, Geolog anatomy, IP
  register); SONAR proves the 11-col schema works at 492 rows; SandiBumi's LAS import
  currently uses scattered hardcoded aliases (parsers.rs).
- *Decision:* merged dictionary ≥5,000 mappings with MD-* conflict rules; units first-class
  with round-trip-tested conversions; SandiBumi later reads it at import (single source of
  truth per B §3.2).
- *Acceptance:* SONAR p8 example groups resolve identically; 36 Techlog training LAS resolve
  every curve to standard+family+unit; cross-vendor disagreements all carry flagged rows.

### Rank 3 — Indexer + FTS5 search (E) — **M/L effort**
- *Context:* the actual daily pain a consultant has: "which of these 30 client folders has the
  SCAL report for well X"; 100k-file cold index overnight is achievable on laptop (E §1.4).
- *Decision:* scanner/hasher/parser pipeline + FileIndex/WellLink/TypeAssignment/CurveStats/
  DocText schema + faceted CLI search with citation cards.
- *Acceptance:* cold-index the project-kb pool (6,668 LAS + study folders) < 1 h; incremental
  pass < 5 min; the Krisna-style faceted count query < 50 ms; byte-identical re-index.

### Rank 4 — Type registry + recognition cascade (D) — **M effort** (interleaves with Rank 3)
- *Acceptance:* D §5 criteria (LAS ≥99% verified; born-digital reports ≥80% verified,
  ≤5% wrong-verified; zero assignments without rule trace).

### Rank 5 — Physics validator + XLSX capture (F, Variant A) — **S/M effort, sellable**
- *Context:* validator rules are transcription of existing docs/ QC-gate + constants specs;
  capture v1 is XLSX round-trip, no editor.
- *Decision:* validator core (~1 wk) + template export/import + review queue; build-time Claude
  extraction as the practical scan path; OCR later behind a feature flag.
- *Acceptance:* golden RCAL/water-analysis fixtures with seeded errors (unit swap,
  fraction/percent, phi–k outlier, depth out of cored interval) all auto-flagged; clean tables
  pass; verdicts reproducible.

### Rank 6 — Curve-QC stats in the indexer (part of Rank 3, called out for the product story)
- P3/P97/null/step/spike per curve per file at index time — feeds GRN normalization
  (P3/P97 house method) and badhole screening before a project even opens. SONAR has nothing
  comparable (p10 stores a summary for embedding).

Deferred (not killed): in-app capture grid (F Variant B) — only if XLSX friction proves real;
OCR integration — only when a study hits a rule-able scan corpus; DLIS deep parse — header-level
suffices until a client corpus demands bulk decode.

## 4. Adversarial pass — kills and near-kills

- **KILLED: dense-vector/semantic leg (any form, even "offline embeddings precomputed at build
  time")** — violates doctrine at query time or drifts at re-index; facets cover the enumerated
  questions; residual accepted openly in §2.
- **KILLED: auto-summaries via build-time AI batch** — tempting ("AI at build time is allowed")
  but it puts model-authored prose inside the runtime DB at corpus scale, making re-index
  non-reproducible in exactly the way the doctrine exists to prevent. Titles from rules; prose
  only as human notes or per-study build-time artifacts clearly stamped `buildtime-ai`.
- **KILLED: SONAR-style agent/tool router** — single-user desktop tool; a CLI/UI with typed
  queries IS the router. SONAR itself added PATCH 11 to bypass its router (p12) — we start
  where their patch ended up.
- **NEAR-KILL (survives, descoped): FindNearestWells** — trivial (geo.rs math exists) but
  low-value for single-basin consulting; ships as a free by-product of well_master coordinates,
  no dedicated effort.
- **NEAR-KILL (survives, reframed): 100–300-type registry** — "is this SONAR-scale
  over-engineering for one consultant?" Partly: v1 ships ~140 types; the defense is that the
  registry costs rows, not code, and project-kb's 46 studies already exercise most categories.
- **Checked against sibling ingests:** no overlap-waste — techlog/Geolog/IP ingests provide the
  *inputs* (catalogs) to Rank 2, they don't already build any of Ranks 1–5.

Validation corpus for all acceptance tests: **6,668 pooled final-log LAS** in
`D:\XX. Clauding\knowledge-base\project-kb` + **36 Techlog training LAS**.

## 5. Deck-internal discrepancy list

1. **Encoder contradiction:** p11 "SONAR Encoder (BGE-M3)" vs p6/p10/p12 Qwen3-Embedding 0.6B
   (1024-dim). Not reconciled in the deck.
2. **Cost total:** p2 "Rp 53,26 M s.d. 2038" vs p17 Rp 52.140.046.800 (Rp 52,14 B) — 1.1 B gap,
   likely a stale slide number.
3. **PoC coverage:** "700 ter-index" of 700,000+ (p2) — 0.1%; all timing/quality figures are
   small-sample extrapolations.
4. **"LAS fast path skips LLM" (p10):** true for metadata, but the path still embeds (GPU), and
   1–3 s/file for a header parse implies the embedding dominates — the "deterministic" label is
   only half the pipeline.
5. **License-avoidance accounting (p17):** Rp 25.22 B assumes a commercial platform purchase
   that building in-house avoids, with no offsetting internal build/maintenance cost.
6. **Two-LLM split (30B vs 8B)** is implied (p6/p10/p12/p13) but never stated as policy.

## 6. Commercial reuse (one paragraph)

SONAR's cost-avoidance framing (p17) is a ready-made pitch template for an offline
data-management product: three avoidance lines (commercial platform license ~USD 127k/yr;
digitization outsourcing at USD 72/file; external validation experts ~USD 52k/yr) that clients'
own planning documents already legitimize — PHE OSES cited its MoM WP&B 2026 for the per-file
rate. Our pitch inverts the infrastructure line: "SONAR-class capability without the GPU
inference server — a single signed binary + SQLite file, deployable inside your firewall,
air-gapped, with every automated decision auditable to a rule ID," and adds the two things the
AI version cannot claim: bit-identical reproducibility (a data-governance property auditors
understand) and physics-validated digitization (catches wrong data, not just unread data).
SandiBumi already ships a Pertamina client theme, so the demo story — index a client's shared
folder overnight on a laptop, answer the "how many wells have core data in field X" question
exactly, in milliseconds — lands in their own branding.

## 7. Session log

- 2026-07-23: Deck read (20 pp), extraction note written to tech-kb (created), targets B–G
  authored. No SandiBumi code touched; no files modified outside
  `docs/research_2026-07/sonar_ingest/` and `knowledge-base/tech-kb/`. PDF untouched.
- Grounding checks run this session: techlog_ingest catalog counts (723 aliases, 2,181
  families, 2,839 family assignments — PowerShell count over A_*.json); SandiBumi src-tauri file
  inventory; ingest.rs import surface (10 pub import fns, no scanner/hasher).
- Implementation, when it starts, goes serially in the main working tree per
  [sandibumi_dev_playbook.md](../../sandibumi_dev_playbook.md) (vcvars 14.29 via PowerShell,
  never Git Bash) — though note Ranks 1–5 are a NEW standalone workspace, not SandiBumi edits;
  only the eventual integration touches SandiBumi.
