# F — Digitization: Lean Capture + Physics Validator

**Honest statement up front:** vision-LLM table reading (SONAR p14: per-page images → Qwen-VL →
JSON rows) is the one SONAR capability with **no full deterministic replacement**. A scanned
1970s core-analysis table cannot be read by rules. This target designs the containment: capture
stays lean and generic, validation is where we win, and any AI stays at build time.

**Scope containment is the design problem.** The deck's own numbers size the real exposure
(p3): CORE 1,394 + RCAL 95 + SCAL 83 + PVT&HC 184 + DST 2,716 + WATER 119 = **4,591
digitization-class docs in a 700k corpus (<1%)**. And digitization here is **on-demand, per
active study** — the indexer (D/E) *finds* documents; only what a study actually needs gets
digitized. For a consultant study that is typically 5–30 documents, not 4,591.

---

## 1. One generic capture path, not N bespoke editors

Report-type knowledge lives in **data templates**, keyed to target D registry types; a single
spreadsheet-style grid renders all of them.

```sql
CREATE TABLE capture_template (
  template_id TEXT PRIMARY KEY,       -- 'RCAL_STD', 'SCAL_PC', 'DST_SUMMARY', 'PVT_CCE',
                                      -- 'WATER_IONS', 'RFT_PRETEST', 'DEVIATION_SURVEY'
  type_id     TEXT REFERENCES doc_type,
  name        TEXT, version INTEGER
);
CREATE TABLE template_column (
  template_id TEXT, col_key TEXT, label TEXT, unit_family TEXT, default_unit TEXT,
  dtype TEXT,             -- num|int|text|depth|date
  required INTEGER, col_order INTEGER,
  PRIMARY KEY(template_id, col_key)
);
-- validator rules: see §3 (separate table, also keyed to template columns)
```

Captured rows adopt SONAR's ToolResults shape (p14) + provenance:

```sql
CREATE TABLE captured_row (
  row_id INTEGER PRIMARY KEY,
  template_id TEXT NOT NULL, file_id TEXT NOT NULL,      -- source document
  master_well_id INTEGER, row_order INTEGER,             -- RowOrder (p14)
  data_json TEXT NOT NULL,                               -- DataJson (p14)
  source_page INTEGER,                                   -- provenance: page
  capture_method TEXT NOT NULL,   -- 'manual-xlsx'|'ocr'|'buildtime-ai'|'in-app'
  captured_by TEXT, captured_utc TEXT,
  is_edited INTEGER DEFAULT 0, is_verified INTEGER DEFAULT 0,   -- IsEdited/IsVerified (p14)
  validation_state TEXT NOT NULL DEFAULT 'unchecked'  -- pass|warn|fail|unchecked
);
```

### Variant A — v1 with NO in-app editor (recommended)

XLSX/CSV templates are the entry surface — engineers already live in Excel. The tool ships:

- `arsip template export RCAL_STD --well JANTI-1 --file <id>` → a locked-header XLSX with unit
  row, dropdowns for enums, provenance columns pre-filled;
- `arsip capture import <xlsx>` → parse (via calamine-class reader), map columns by header key,
  run validator (§3), write `captured_row`s with per-row flags;
- review queue lists failing/warning rows first; verification toggles `is_verified`.

Effort estimate: template tables + XLSX round-trip + validator + queue UI ≈ **1.5–3 weeks**
(validator itself is the reusable core, ~1 week, seeded from existing spec docs — see §3).

### Variant B — in-app grid editor

Single generic grid component (arsip-ui, Svelte) rendering any template: column defs from
`template_column`, cell-level validation coloring, row provenance sidebar. Adds ≈ **2–4 weeks**
on top of Variant A (grid UX, undo, clipboard paste from PDF viewers, keyboard nav). Decision:
**ship Variant A first**; add B only if XLSX round-trip friction proves real in use. Both
variants share every table above, so B is additive, not a rework.

## 2. OCR path — boundary stated honestly

- Scope: **typed/tabular scans only**, where layout rules can be written (fixed-column vendor
  layouts, e.g. Corelab RCAL sheets as in the p14 example). Handwriting, degraded microfiche,
  free-form text: out — manual/build-time path.
- Engine: offline Tesseract-class engine, GPU-accelerated *preprocessing* (deskew, binarize,
  de-speckle — deterministic image ops); layout rules (column x-ranges, row banding) live in
  per-vendor-layout TOML.
- **Neural honesty:** modern OCR engines (Tesseract 4+ LSTM, PaddleOCR, docTR) embed small
  neural character recognizers. If the strict no-neural line matters for a client, run
  Tesseract in **legacy engine mode (`--oem 0`, pattern-matching classifier)**: expect
  meaningfully worse accuracy on degraded scans — roughly, legacy mode turns low-90s% character
  accuracy on clean typed scans into ~80s%, and gets much worse with noise (quantify on our own
  fixtures at build; published comparisons vary widely). Default posture: LSTM mode is
  acceptable — it is a *local, deterministic-per-input, auditable* character classifier, not a
  generative model, and every OCR row still passes the physics validator + human verify. But the
  strict option exists and is a config flag, and `capture_method='ocr'` rows are never
  auto-verified either way.
- OCR is an optional feature flag of arsip-core (B §3.3) — the tool is complete without it.

## 3. THE differentiator SONAR lacks: automatic physics validation on every row

SONAR's review is eyeball-only (engineer edits/verifies rows, p14). Ours validates **every row
regardless of capture method** with rule TABLES over template columns:

```sql
CREATE TABLE validation_rule (
  rule_id TEXT PRIMARY KEY,            -- 'V-RCAL-001', ...
  template_id TEXT, col_key TEXT,      -- col_key NULL for cross-column/cross-row rules
  kind TEXT NOT NULL,   -- unit|range|relation|trend|order
  spec_json TEXT NOT NULL,             -- parameters
  severity TEXT NOT NULL               -- fail|warn
);
```

Rule kinds, seeded directly from existing specs in docs/ (log QC gates note, tool-response
constants, [constants_verification_2026-07-22.md](../../constants_verification_2026-07-22.md),
[workflow_standards.md](../../workflow_standards.md)) — shared infrastructure, days not months:

1. **Unit checks** — declared unit ∈ column's unit_family; value plausibility after conversion
   to base (catches the md-vs-D and fraction-vs-percent classics).
2. **Physical ranges** (per column, warn+fail bands): porosity 0–0.45 v/v (fail outside 0–0.5);
   grain density 2.55–3.0 g/cc (warn outside, fail outside 2.0–3.5 — anhydrite/coal exceptions
   whitelisted by lithology column when present); perm > 0, log-normal sanity (warn > 10 D);
   Sw 0–1; salinity/ion balance for water analyses (cation-anion balance within 5%); DST
   pressures ≥ 0 and ≤ lithostatic-at-depth heuristic.
3. **Cross-row consistency** — phi–k trend outliers (fit log k vs phi per rock group, flag
   > 3σ residuals — deterministic least squares); He-porosity vs density-porosity coherence
   when both captured; depth ordering monotonic; sample depths inside the cored interval
   declared for the source document; duplicate sample IDs.
4. **Cross-source coherence** (unique to us, because the indexer knows the LAS): captured plug
   depth must lie inside the well's logged interval (CurveStats depth range); RCAL grain
   density vs log RHOB at same depth within tolerance → warn.

Failing rows are auto-flagged so the engineer **reviews flagged rows first** — verification
effort concentrates where it pays. Validator runs are deterministic and re-runnable; each
verdict stores its rule_id list (same auditability contract as target D).

## 4. Build-time AI extraction — the expected workhorse in practice

When a study needs a table from a scan, Jauhar runs Claude on the scan (as with chartbook
digitization in tools/chartdig); output lands in **the same XLSX template → validator → review
queue**, `capture_method='buildtime-ai'`. Allowed because it never ships in the runtime — the
shipped product contains templates, validator, queue; not the model. Human effort shrinks to
verifying flagged rows — minutes per report, not retyping. Note SONAR's own flow ends the same
way (engineer reviews IsVerified rows, p14): **the human loop never disappears with AI; it just
moves from typing to checking.** Our design makes the checking cheap and targeted; SONAR's makes
it uniform eyeballing.

## 5. Provenance & reuse

- Per-row provenance: source `file_id` + `source_page` + `capture_method` + who/when — carries
  straight into SandiBumi interpretation provenance (project-kb decision-record style,
  parameter-level citations).
- chartdig lessons reused: vector-first extraction when the PDF has vector content (many "scans"
  in archives are vector PDFs of printed tables — check text layer first, always cheaper);
  fixture-driven acceptance (golden tables re-extracted bit-identically).

## 6. Economics

PHE OSES budgets **USD 72/file** for outsourced digitization (p17, MoM WP&B 2026) — Rp 16.55 B
of their 12-yr avoidance case. A validator-backed capture workflow at consultant quality is
therefore also a **sellable service**: per-file pricing anchored against the client's own
outsourcing benchmark, with physics-validated rows (which the USD 72 outsourcer does not
provide) as the differentiator. At even 50 files/study, the validator pays for its ~1-week build
in the first engagement.
