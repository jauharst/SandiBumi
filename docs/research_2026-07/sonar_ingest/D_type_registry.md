# D — Document-Type Registry + Deterministic Recognition Cascade

The deterministic replacement for SONAR's embedding classification (deck p11) and the heart of
"capture the whole O&G data universe". Two parts: a hierarchical TAXONOMY (data, extensible
without code) and a RECOGNITION CASCADE (cheapest test first, every match logs a rule ID).
Adopts SONAR's three archive states verified/pending/flagged (p11) and its per-category
threshold idea — but thresholds gate *rule scores*, not cosine similarities.

---

## 1. Taxonomy

### 1.1 Structure

```sql
CREATE TABLE doc_type (
  type_id     TEXT PRIMARY KEY,   -- stable, human-readable: 'LOG.LAS.2', 'CORE.RCAL.REPORT'
  parent_id   TEXT REFERENCES doc_type,
  level       TEXT NOT NULL,      -- category|type|subtype
  name_en     TEXT NOT NULL,
  name_id     TEXT,               -- Indonesian name (bilingual registry)
  description TEXT,
  digitizable INTEGER NOT NULL DEFAULT 0,  -- feeds target F scoping
  template_id TEXT                -- capture template if digitizable (target F)
);
```

Stable IDs are load-bearing: `TypeAssignment` rows, capture templates, validator rule tables and
facet filters all key on `type_id`. Renames change `name_*`, never the ID. New types are INSERTs.

### 1.2 Initial registry (category → representative types; ~140 concrete types at v1, headroom to 300)

Seeded from the deck's own lists (p2 formats, p3 volumes, p6 source classes) + project-kb's 46
delivered studies + petro-kb taxonomy + the Techlog/Geolog/IP ingests.

- **LOG — well logs**
  - `LOG.LAS.12 / LOG.LAS.2 / LOG.LAS.3` — LAS by version; `LOG.DLIS`, `LOG.LIS`
  - `LOG.ASCII` — vendor ASCII exports (Geolog, IP, Techlog *.txt/*.csv curve dumps)
  - `LOG.COMPOSITE.PDF` — composite/CPI plots; `LOG.PRINT.TIFF` — raster log prints
  - `LOG.ELAN` — processed interpretation deliverables (ELAN/Multimin/Quanti outputs)
  - `LOG.NMR`, `LOG.IMAGE` (FMI/STAR processed), `LOG.CBL_VDL`, `LOG.MUDGAS_WHILE_DRILL`
- **CORE**
  - `CORE.RCAL.REPORT`, `CORE.RCAL.TABLE` (XLSX/CSV), `CORE.SCAL.REPORT` (Pc, kr, wettability,
    electrical props), `CORE.PHOTO` (white/UV), `CORE.DESCRIPTION` (sed log),
    `CORE.THINSECTION`, `CORE.XRD`, `CORE.SEM`, `CORE.GRAINSIZE` (PSD/Mastersizer)
- **MUDLOG** — `MUDLOG.MASTERLOG`, `MUDLOG.FEL` (final well report mudlog), `MUDLOG.GWD`
  (gas-while-drilling data/report)
- **WTEST — well tests** — `WTEST.DST.REPORT`, `WTEST.DST.CHART`, `WTEST.PLT`,
  `WTEST.RFT_MDT` (pretest/pressure survey tables), `WTEST.PBU_PTA`
- **FLUID** — `FLUID.PVT.REPORT`, `FLUID.OIL_ANALYSIS`, `FLUID.GAS_ANALYSIS`,
  `FLUID.WATER_ANALYSIS` (the p3 WATER class), `FLUID.SALINITY_TABLE`
- **DRILL** — `DRILL.DDR` (daily drilling report), `DRILL.DGR` (daily geology),
  `DRILL.WELLPLAN/PROGNOSIS`, `DRILL.BHA`, `DRILL.DEVIATION` (surveys), `DRILL.MUDREPORT`,
  `DRILL.EOWR` (end-of-well/final well report — bilingual "Laporan Akhir Pemboran")
- **COMPL** — `COMPL.PERF_RECORD`, `COMPL.DIAGRAM`, `COMPL.WORKOVER`, `COMPL.WELLHEAD`
- **SURVEY** — `SURVEY.COORDS` (wellhead coordinates/datum sheets), `SURVEY.CHECKSHOT_VSP`,
  `SURVEY.VELOCITY`, `SURVEY.SEGY_NAV`
- **GG — G&G studies** — `GG.REPORT.GEOLOGY`, `GG.REPORT.PETROPHYSICS`, `GG.REPORT.RESERVOIR`,
  `GG.REPORT.GEOPHYSICS`, `GG.PRESENTATION`, `GG.MAP`, `GG.GRID` (Zmap/CPS-3/Petrel exports),
  `GG.SEISMIC.SEGY`
- **PROD** — `PROD.HISTORY` (rates), `PROD.PRESSURE_HISTORY` (static/flowing surveys),
  `PROD.WELLTEST_MONTHLY` (sumur uji produksi)
- **ADMIN** — `ADMIN.AFE`, `ADMIN.WPNB` (WP&B), `ADMIN.CORRESPONDENCE`, `ADMIN.MOM`,
  `ADMIN.CONTRACT`
- **CONTAINER / OTHER** — `ZIP.ARCHIVE` (descend, classify members), `UNKNOWN` (explicit type,
  never a NULL)

Each digitizable type (`CORE.RCAL.TABLE`, `CORE.SCAL.*`, `WTEST.DST.*`, `FLUID.PVT.*`,
`FLUID.WATER_ANALYSIS`, `WTEST.RFT_MDT`, `PROD.*`) points at a capture template (target F) —
this is the registry↔digitization contract.

## 2. Recognition cascade

Per file, cheapest first; stages only *add* candidate (type_id, score, rule_id) evidence; final
scoring at stage (e). All rules are DATA (TOML tables loaded into SQLite), hot-loadable, no code
changes to extend.

```
(a) extension → (b) magic bytes / format sniff → (c) filename+path grammar
→ (d) born-digital content signatures → (e) weighted score → state
```

### (a) Extension map (rules `RX-*`)
`.las→LOG.LAS.*`, `.dlis→LOG.DLIS`, `.lis/.tif→candidates`, `.segy/.sgy→GG.SEISMIC.SEGY`, … —
weak evidence only (weight ~0.2); extensions lie constantly in shared folders.

### (b) Magic bytes / header sniff (rules `RM-*`) — strong evidence (weight ~0.9)
- LAS: file starts `~V` (after optional BOM/comments); version line disambiguates 1.2/2.0/3.0.
- DLIS: Storage Unit Label at byte 0 (`RSD`-pattern, "V1.00" SUL) — [dlis.rs](../../../src-tauri/src/dlis.rs) already parses this.
- LIS: TIF-padded records heuristic.
- SEG-Y: 3200-byte EBCDIC reel header (test: >60% bytes in EBCDIC printable set) + binary header
  sample-format code in {1,2,3,5,8}.
- ZIP/OOXML: `PK\x03\x04`; OOXML distinguished by `[Content_Types].xml` member
  (docx/xlsx/pptx by main part path). Legacy DOC/XLS: OLE2 `D0 CF 11 E0`.
- PDF: `%PDF-`; **text-layer presence probe**: extract text of first 3 pages; if
  < 50 chars/page average → `scanned=true` (routes to target F queue at stage d, never guessed).
- TIFF/JPG/PNG magics; CSV/TXT: absence of magic + printable ratio → content sniffing (d).

### (c) Filename + path grammar (rules `RF-*`) — the workhorse (weight 0.5–0.8 per hit)
SONAR's filename fast path (p10, ~200 ms/file) reduced to its deterministic core (ours: µs).
Tokenize full path; match three token classes:

1. **Well tokens** — against the alias table (target C resolver). A resolved well is both a
   `WellLink` and type evidence (e.g. path under a well folder).
2. **Date patterns** — `1993`, `04-1993`, `19930412`, Indonesian month names (`Apr`, `April`,
   `Agustus`…) → `doc_date` candidate.
3. **Type keywords — BILINGUAL lexicon** (`RF-KW-*` rows: keyword, lang, type_id, weight):

   | Indonesian | English | type_id |
   |---|---|---|
   | Laporan Akhir (Pemboran) | Final Well Report / EOWR | DRILL.EOWR |
   | Uji Kandungan Lapisan / UKL | DST | WTEST.DST.REPORT |
   | Analisa Inti Batuan / Analisis Batuan Inti | (Routine) Core Analysis | CORE.RCAL.* |
   | Laporan Harian Pemboran | Daily Drilling Report | DRILL.DDR |
   | Analisa Air (Formasi) | Water Analysis | FLUID.WATER_ANALYSIS |
   | Uji Produksi | Production Test | PROD.WELLTEST_MONTHLY |
   | Titik Koordinat | Coordinates | SURVEY.COORDS |
   | Penampang / Peta | Cross-section / Map | GG.MAP |
   | Kerja Ulang | Workover | COMPL.WORKOVER |
   | Berita Acara / Notulen | MoM | ADMIN.MOM |
   | masterlog / mudlog | masterlog / mudlog | MUDLOG.MASTERLOG |
   | checkshot / VSP | checkshot / VSP | SURVEY.CHECKSHOT_VSP |

   (Seed lexicon ~200 keywords at v1; grown from the review queue, mined at build time from
   project-kb filenames — AI may help author rows, rows ship as data.)

4. **Folder-context inheritance (`RF-CTX`)**: a file inside a folder already dominated by a
   type/well inherits those candidates at reduced weight (e.g. anything in
   `...\JANTI-1\DST\` starts with WTEST.DST.* at 0.4 and JANTI-1 at 0.6). Context = computed
   from sibling assignments, recomputed on re-index — still deterministic (pure function of the
   tree, evaluated in canonical path order).

### (d) Born-digital content signatures (rules `RC-*`) — weight 0.6–0.9
Only for files whose text is deterministically extractable (pdftotext-class extraction, OOXML
XML, plain text). Regex/anchor tables per type, e.g.:
- `CORE ANALYSIS RESULTS|POROSITY.*PERMEABILITY.*GRAIN DENSITY` header block → CORE.RCAL.REPORT
- `DRILL STEM TEST|INITIAL FLOW|FINAL SHUT-?IN|ISIP` → WTEST.DST.REPORT
- LAS ~Well section content: `SRVC|COMP` company fields feed metadata; curve set feeds
  `LOG.LAS.*` subtype confidence (curve families via super-dictionary).
- `SALINITY|CHLORIDE|ppm NaCl` + tabular ion names (Ca, Mg, HCO3, SO4) → FLUID.WATER_ANALYSIS.
- **Scanned PDFs (no text layer): stop here.** Assign best filename/path candidate with
  `state='pending'` and enqueue for target F's review/OCR queue. Never guessed from pixels —
  that would be smuggled vision AI.

### (e) Scoring + states (rules `RS-*`)
- `score(type) = 1 - Π(1 - w_i)` over independent evidence hits (noisy-OR; order-independent ⇒
  deterministic), computed per candidate type.
- Per-type thresholds in data (adopting SONAR's per-category threshold idea, p11):
  `t_verified` (e.g. 0.85) and `t_pending` (e.g. 0.5).
  - best ≥ t_verified AND unique winner (margin ≥ 0.15) → `verified`
  - best ≥ t_pending → `pending` (review queue, pre-filled suggestion)
  - else → `flagged` (+ `UNKNOWN` assignment)
- Assignment row (integration contract, B §3.2): `TypeAssignment(file_id, type_id, score,
  state, rule_trace)` where `rule_trace` is the JSON list of every rule ID + weight that fired —
  a petrophysicist can read exactly why a file was classified, and fix the *rule*, not the file.

## 3. Review-queue workflow

One queue, three feeders: (i) pending/flagged type assignments, (ii) alias collisions
(target C), (iii) scanned docs awaiting F. Queue UI (arsip-ui) shows the rule trace and one-click
actions: confirm (→ verified; optionally "add filename token as new RF-KW rule"), reassign
(records an override row `TypeOverride(file_id, type_id, who, when)` — overrides survive
re-indexing and outrank rules), or escalate to digitization. Every action is data; re-running
the cascade on an unchanged corpus + unchanged rules reproduces identical states.

## 4. Worked recognition examples (fixtures for acceptance tests)

1. `...\CORE DATA\JANTI-1\Analisa Inti Batuan JANTI-1 1994.pdf`, born-digital →
   (a) .pdf 0.2 generic; (c) well JANTI-1 via alias table [WL-01], keyword "Analisa Inti
   Batuan" → CORE.RCAL.REPORT 0.7 [RF-KW-014], year 1994; (d) header regex hit 0.8 [RC-003];
   score 0.94 → **verified**, WellLink JANTI-1.
2. `3808_rama_g13_rt.las` (cf. p15 screenshot style) → (b) `~V` magic → LOG.LAS.2 0.9 [RM-01];
   ~Well section WELL=RAMA G-13 → resolver stem RAMA + G13 [WL-03]; curves via dictionary →
   subtype + curve facets; **verified** in ms, no review needed.
3. `SCAN_B1_DST~1.PDF`, no text layer → (b) PDF+scanned [RM-07]; (c) `DST` keyword 0.6, well
   token `B1` **collides** (three B-1 wells in master) → type `pending` (WTEST.DST.REPORT 0.6),
   well unresolved with collision row → review queue with both suggestions shown. Honest stop,
   no guessing.
4. `Laporan Akhir RAMD-14.docx` → OOXML [RM-04], "Laporan Akhir" → DRILL.EOWR [RF-KW-001],
   RAMD-14 → RAMOS DELTA-14 via alias rows (the deck's own p12 example) → **verified**.

## 5. Acceptance criteria

- Registry v1: 100–300 concrete types with stable IDs, bilingual names, digitizable flags.
- Cascade classifies the project-kb corpus (6,668 LAS + report folders of the 46 studies):
  LAS ≥ 99% verified correctly (format sniff makes this nearly free); born-digital reports
  ≥ 80% verified / ≤ 5% wrong-verified (wrong-verified is the metric that matters — pending is
  cheap, silent misfiling is not).
- Re-run on identical corpus → byte-identical TypeAssignment table (determinism gate).
- Zero classifications without a rule trace.
