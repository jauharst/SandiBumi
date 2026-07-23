# E — Indexer + Search at 100k–1M Scale

Runtime engine of the standalone tool: one workstation, no server, single binary + SQLite.
Deterministic throughout; CPU/GPU parallelism for speed only, results bit-identical run-to-run.
Deck cites refer to the SONAR extraction note (tech-kb).

---

## 1. Indexer

### 1.1 Pipeline shape (multi-core from the start)

```
walker (1 thread, canonical order) ─► work queue
  ├─► hasher pool   (N cores: SHA-256, size/mtime short-circuit)
  ├─► sniffer pool  (magic bytes, target D stage b)
  ├─► parser pool   (per-route parsers below)
  └─► single writer (one SQLite connection, WAL, batched transactions)
```

Determinism under parallelism: workers are stateless pure functions per file; the single writer
commits in canonical path order (sort key = normalized relative path), so the resulting DB is
byte-identical regardless of scheduling. Content-derived IDs (`file_id = first 16 hex of
SHA-256` + path row) — no autoincrement dependence on arrival order, no timestamps inside
derived rows (scan wall-clock lives only in the scan-run log table).

### 1.2 Scan + dedup (adopt p10)

- Recursive scan; **SHA-256 per file**; `FileIndex` check: same path+hash → untouched; same
  hash, new path → duplicate link row (`file_duplicate(hash, path)` — dedup is *recorded*, not
  silently skipped, because duplicate placement is itself information about shared folders).
- **Incremental re-scan**: (size, mtime_ns) match vs stored → skip hashing entirely; else
  re-hash. Deleted paths marked `missing`, retained for provenance.
- **ZIP descent**: bounded depth (default 3), members hashed and routed through the same
  cascade; `zip://outer.zip!/inner/path` addressing in FileIndex.

### 1.3 Per-route parsers (registry-driven, target D decides route)

| Route | Work | Budget (per file, warm cache, mid laptop core) |
|---|---|---|
| LAS (full parse) | Header + ~Curve + ~ASCII; mnemonic→standard via super-dictionary; **per-curve QC stats SONAR never computes**: P3/P97, min/max, null fraction, depth range/step, monotonicity, spike count (Hampel), badhole co-flag vs caliper if present | **< 100 ms** typical (1–20 MB LAS); vs SONAR 1–3 s/file summary-only (p10) → 10–30× + more information |
| DLIS/LIS | Header-level: SUL, origin, frame/channel inventory → curve list + units (no bulk decode at index time) | < 50 ms |
| PDF born-digital | Text-layer extraction (pdftotext-class), first N pages full + per-page offsets; content signatures (D stage d); text → FTS5 | 50–300 ms |
| PDF scanned | Detect (no text layer) → route to F queue; index filename/path evidence only | < 20 ms |
| DOCX/XLSX/PPTX | OOXML structured extraction (text, sheet cells, headings) | 50–200 ms |
| CSV/ASCII | Sniff delimiter/header; detect curve-dump shape (depth-monotonic first column → treat as LOG.ASCII with curve stats) | < 50 ms |
| Images (TIFF/JPG/PNG) | Magic + EXIF/dimensions only; optional GPU batch ops later (deskew/thumbnail for F) — image *processing*, never inference | < 10 ms |

### 1.4 Throughput budget (100k-file cold index, 8-core laptop)

Corpus shaped like the deck's (700k total, <1% digitization-class; assume ~10% LAS, ~40% PDF,
~25% Office, ~25% other): hashing ~100 GB at NVMe speed ≈ 20–40 min; parsing dominated by
PDF/Office at ~150 ms avg × 90k / 8 cores ≈ 30 min; LAS 10k × 100 ms / 8 ≈ 2 min. **Cold index
≈ 1–2 h — comfortably overnight even at 1M files (≈ 10–20 h) on one workstation.** Incremental
daily pass = stat walk (minutes at 100k–1M) + changed files only. SONAR's own numbers for
comparison: 10–40 s/file on the document path (p10) ⇒ 700k docs ≈ 80–320 GPU-days — their
full-corpus cost is why "700 ter-index" (p2) is 0.1% of the archive.

## 2. Storage — SQLite schema

Dictionaries/well master (target C) plus:

```sql
CREATE TABLE FileIndex (
  file_id     TEXT PRIMARY KEY,          -- content-derived
  path        TEXT NOT NULL UNIQUE,      -- normalized, zip-aware
  size        INTEGER, mtime_ns INTEGER,
  sha256      TEXT NOT NULL,
  format      TEXT NOT NULL,             -- sniffed container format
  scanned_pdf INTEGER DEFAULT 0,
  status      TEXT NOT NULL DEFAULT 'active'   -- active|missing
);
CREATE INDEX ix_fi_sha ON FileIndex(sha256);

CREATE TABLE file_duplicate ( sha256 TEXT, path TEXT, PRIMARY KEY(sha256, path) );

CREATE TABLE WellLink (      -- file ↔ well via alias resolution
  file_id TEXT REFERENCES FileIndex, master_well_id INTEGER REFERENCES well_master,
  rule_id TEXT NOT NULL, confidence REAL NOT NULL, state TEXT NOT NULL,
  PRIMARY KEY(file_id, master_well_id)
);

CREATE TABLE TypeAssignment (
  file_id TEXT PRIMARY KEY REFERENCES FileIndex,
  type_id TEXT NOT NULL REFERENCES doc_type,
  score REAL NOT NULL, state TEXT NOT NULL,          -- verified|pending|flagged
  rule_trace TEXT NOT NULL                           -- JSON evidence list
);

CREATE TABLE LasHeader (
  file_id TEXT PRIMARY KEY, las_version TEXT, well_name_raw TEXT,
  field_raw TEXT, company TEXT, service TEXT, log_date TEXT,
  top_depth REAL, bottom_depth REAL, step REAL, null_value REAL, depth_unit TEXT
);

CREATE TABLE CurveStats (
  file_id TEXT, mnemonic_original TEXT, mnemonic_standard TEXT, curve_family TEXT,
  unit_raw TEXT, unit_id INTEGER,
  n INTEGER, null_frac REAL, p3 REAL, p50 REAL, p97 REAL, vmin REAL, vmax REAL,
  depth_top REAL, depth_bottom REAL, monotonic_depth INTEGER, spike_count INTEGER,
  PRIMARY KEY(file_id, mnemonic_original)
);

CREATE VIRTUAL TABLE DocText USING fts5(
  content, title, tokenize='unicode61 remove_diacritics 2',
  content_rowid handled via shadow map to file_id + page/section
);
CREATE TABLE DocTextMap ( rowid INTEGER PRIMARY KEY, file_id TEXT, page INTEGER,
                          section TEXT );        -- small-to-big expansion anchor (p12 adopt)
```

**FTS5 at 1M rows**: FTS5 is proven far beyond this scale; with page-granularity rows
(~5–20 M rows, ~2–10 GB index for a text-heavy corpus) BM25 queries with a metadata pre-filter
(join on pre-filtered file_id set) stay in the tens of ms. Mitigations if needed: partition
DocText per top-level category; `detail=column` off; contentless-delete mode; `optimize` after
cold index. Note: BM25 ranking depends only on corpus content, not insertion order → determinism
holds (ties broken by file_id).

## 3. Search

### 3.1 Core pattern — SONAR's best trick promoted (p12)

**Metadata pre-filter BEFORE text search**, always:

```
candidates = FileIndex ⋈ WellLink ⋈ TypeAssignment ⋈ CurveStats facets
hits       = DocText MATCH query AND rowid ∈ map(candidates)   -- BM25 rank
```

Facets: well (via alias resolver on the query string — same WL-* rules), field/structure, type
(registry subtree), format, date range, depth range, **curve availability** (e.g. "has RT_DEEP
and NPHI over 2,300–2,500 m"), state (verified/pending/flagged). FTS supports
boolean/phrase/prefix (`NEAR`, `"drill stem test"`, `perf*`).

Result cards carry citation-style tags (adopt p12): `[FILE_ID | well | type | "Title"]` + the
rule trace on hover — parameter-level provenance, not vibes.

### 3.2 Structured query API — deterministic equivalents of SONAR's tools (p13)

| SONAR tool | Ours (arsip-core fn / CLI subcommand) | Implementation |
|---|---|---|
| ListWells | `wells list [--field]` | well_master scan |
| GetWellDocuments | `wells docs <well> [--type]` | WellLink join |
| GetWellInfo | `wells info <well>` | master + attrs + doc/curve rollup |
| GetWellInfoStats / GetStats | `stats [--by field\|type\|state]` | GROUP BY counts |
| FindNearestWells | `wells near <well\|lat,lon> [--k]` | haversine over well_master (geo.rs math already exists in SandiBumi) |
| QueryLasPoint | `las point <well> <curve> <depth>` | best LAS by depth coverage via CurveStats → seek ~ASCII row (parse-on-demand, cached) |
| SearchDocuments | `search "<query>" [--facets]` | §3.1 |
| GetFileDetail | `file <id>` | FileIndex + assignments + trace |
| ClassifyFile / ListCategories | `classify <path>` / `types list` | cascade dry-run with printed rule trace |

Latency target: **< 50 ms interactive** on a laptop for faceted queries at 100k–1M files
(indexed joins + FTS5 with pre-filter); `stats` style counts are pure index scans, < 10 ms.

The deck's own motivating question (p4) — *"How many wells have core data in Krisna Field?"* —
becomes:

```sql
SELECT COUNT(DISTINCT w.master_well_id)
FROM well_master w JOIN WellLink l USING(master_well_id)
JOIN TypeAssignment t USING(file_id)
WHERE w.field_name='KRISNA' AND t.type_id LIKE 'CORE.%' AND t.state='verified';
```

Faster AND exact, where SONAR runs GPU inference to approximate the same count.

### 3.3 Explicitly OUT + mitigation

- **Chat answer synthesis** — out. Mitigation: result cards + structured queries; the engineer
  reads the document, which is what happens after a RAG answer anyway (the citations ARE the
  product).
- **Semantic similarity search over prose** — out. Mitigation: bilingual keyword lexicon
  (target D) covers domain synonymy where it matters ("Uji Kandungan Lapisan" finds DSTs without
  embeddings); facets + curve-availability filters answer the engineer questions in the deck;
  FTS prefix/boolean covers fuzzy recall. Residual gap stated in FINDINGS §2: conceptual
  paraphrase search over unstructured prose ("wells with anomalous pressure behavior") has no
  deterministic equivalent — that's a human reading task, and the facets get the human to the
  right 20 documents in 50 ms.

## 4. GPU policy

Allowed for deterministic throughput only: batch image preprocessing for target F (deskew,
binarize, thumbnail), optional GPU hashing experiments. Fixed-point or exactly-specified float
kernels only where results feed stored values; anything reduction-order-sensitive stays on CPU.
No inference hardware requirement at all — contrast with SONAR's dedicated
large-VRAM GPU server for Qwen3 30B (p6) and VRAM-pinned P6000 batch cycles (p10). Our
deployment cost is ~zero and runs air-gapped at client sites.
