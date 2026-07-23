# C — Data Foundation: Well Master + Super-Dictionary + Units

All-deterministic backbone of the standalone tool (working name sandi-arsip; boundary contract
in [B_gap_matrix.md](B_gap_matrix.md) §3). Schema DDL + build plan only — no code in this
session. Deck page cites refer to the SONAR extraction note
(`knowledge-base\tech-kb\sonar_phe_oses_hackathon2026.md`).

---

## 1. Well master + alias resolution

SONAR's shape (p7): 2,418 wells × 207 attributes keyed by `master_well_id` / `uwi_sonar` /
`official_well_name`, connecting alias, field, structure, platform, basin, type, status. We
adopt the shape but size it for **multi-project consultant work**: wells arrive per engagement,
from different operators, with colliding names ("B-1" exists in every basin on Earth).

### 1.1 DDL

```sql
CREATE TABLE well_master (
  master_well_id   INTEGER PRIMARY KEY,
  uwi              TEXT UNIQUE,            -- if known; consultant data often lacks UWIs
  official_name    TEXT NOT NULL,
  name_normalized  TEXT NOT NULL,          -- see normalization rules below
  field_name       TEXT, structure_name TEXT, platform_name TEXT, basin_name TEXT,
  operator         TEXT,
  well_type        TEXT,                   -- exploration/development/injector/...
  status           TEXT,                   -- active/suspended/P&A/...
  surface_lat REAL, surface_lon REAL, crs TEXT,
  kb_elev REAL, td_md REAL,
  project_tag      TEXT,                   -- consultant engagement scoping
  created_utc TEXT NOT NULL, updated_utc TEXT NOT NULL
);
CREATE INDEX ix_wm_norm  ON well_master(name_normalized);
CREATE INDEX ix_wm_field ON well_master(field_name, structure_name);

CREATE TABLE well_alias (
  alias_id        INTEGER PRIMARY KEY,
  master_well_id  INTEGER NOT NULL REFERENCES well_master,
  alias           TEXT NOT NULL,
  alias_normalized TEXT NOT NULL,
  alias_kind      TEXT NOT NULL,   -- 'official'|'short'|'historic'|'vendor'|'filename-observed'
  source          TEXT NOT NULL,   -- who asserted it (import file, engineer, rule)
  state           TEXT NOT NULL DEFAULT 'verified',  -- verified|pending|flagged (p11 states)
  UNIQUE(alias_normalized, master_well_id)
);
CREATE INDEX ix_wa_norm ON well_alias(alias_normalized);
```

Wide operator attributes (SONAR's 207 columns) go in a key-value side table
`well_attr(master_well_id, attr_name, attr_value, source)` rather than 207 physical columns —
consultant corpora never fill them all, and the set differs per client.

### 1.2 Alias-resolution service (the highest-leverage deterministic component)

SONAR's WellAliasService (p12, RAMD-14 ↔ RAMOS DELTA-14) is what lets filename rules resolve
wells without AI. Ours, fully specified:

**Normalization (rule IDs `WN-*`)** — applied to both stored aliases and query strings:
1. Uppercase; trim; collapse whitespace.
2. Unify separators: `_`, `.`, multiple spaces, `#` → single `-` (WN-02).
3. Strip decorations: `WELL`, `SUMUR`, `ST` suffix handling (sidetrack: keep as suffix token),
   quote marks (WN-03).
4. Split trailing number: `RAMD14`, `RAMD-14`, `RAMD 14` → stem `RAMD` + num `14`; zero-pad
   removed (`RAMD-014` ≡ `RAMD-14`) (WN-04).
5. Expansion pairs live in DATA, not code: `RAMD ↔ RAMOS DELTA` style stem expansions are just
   two `well_alias` rows pointing at one master id.

**Lookup order (first hit wins, each step logs its rule ID):**
1. Exact `alias_normalized` match, scoped to active project tag (WL-01).
2. Exact match unscoped (WL-02 — flags cross-project hit for review).
3. Stem+number match where stem matches an alias stem uniquely (WL-03).
4. Bounded fuzzy: Damerau-Levenshtein distance ≤ 1 on stem, number must match exactly
   (WL-04, result state = `pending`, never auto-`verified`). Deterministic tie-break: reject on
   tie (→ collision row), never pick arbitrarily.

**Explicit collision handling:** a `well_alias_collision(alias_normalized, candidate_ids,
first_seen_file)` table. A resolver hit on a collided alias returns *no* well and routes the
file to the review queue — silence is a bug, a queue entry is the design.

**Build plan:** seed from project-kb's 46 decision-record projects (well names + fields already
curated) + LAS ~Well sections of the 6,668 pooled final-log LAS (well name, field, company
harvested deterministically) + Jauhar's per-project well lists. Acceptance: resolver correctly
links ≥95% of the 6,668 LAS filenames to wells with zero false merges (false merge = two real
wells joined; audited by field mismatch).

## 2. Mnemonic super-dictionary

Adopt SONAR's 11-column schema (p8) verbatim as the core, add provenance columns. SONAR: 492
rows → 218 standards, hand-built. We out-populate from vendor catalogs we already extracted —
sources SONAR doesn't have.

### 2.1 DDL

```sql
CREATE TABLE mnemonic_dict (
  id                 INTEGER PRIMARY KEY,
  mnemonic_original  TEXT NOT NULL,     -- as seen in files (may include tool suffixes)
  mnemonic_clean     TEXT NOT NULL,     -- uppercased, suffix-stripped
  mnemonic_standard  TEXT NOT NULL,     -- our canonical name (GR, RHOB, NPHI, RT_DEEP, ...)
  curve_family       TEXT NOT NULL,     -- Techlog-style family (2,181-entry family tree)
  curve_role         TEXT,              -- measured/computed/flag/aux
  log_type           TEXT,              -- WL|LWD|ELAN|PLT|NMR|CBL|MUDLOG|CORE...
  data_category      TEXT,
  description        TEXT,
  typical_units      TEXT,
  unit_family        TEXT REFERENCES unit_family(name),
  mapping_confidence REAL NOT NULL,     -- 1.0 vendor-official, 0.9 cross-vendor agree, ...
  source             TEXT NOT NULL,     -- 'techlog'|'techcore'|'geolog'|'ip'|'jauhar'|'sonar-shape'
  source_ref         TEXT,              -- file/row provenance in the vendor catalog
  state              TEXT NOT NULL DEFAULT 'verified',   -- verified|pending|flagged
  UNIQUE(mnemonic_original, source)
);
CREATE INDEX ix_md_clean ON mnemonic_dict(mnemonic_clean);
CREATE INDEX ix_md_std   ON mnemonic_dict(mnemonic_standard);
```

Reverse map (SONAR p15) is a view: `SELECT mnemonic_standard, group_concat(mnemonic_original)
... GROUP BY 1` — no separate table to drift.

### 2.2 Population plan (target ≥ 5,000 validated rows vs SONAR's 492)

| Source (already extracted — do not re-mine) | Where | Contribution |
|---|---|---|
| Techlog mnemonic aliases | techlog_ingest `A_mnemonic_alias.json` — 379 Techlog + 344 Techcore | ~723 alias rows |
| Techlog family assignments | `A_family_assignment.json` — 2,839 assignments onto 2,181 families (`A_families.json`) | ~2,800 rows with family + unit_family |
| Geolog V14 `alias.alias` | Geolog install anatomy ingest | vendor #2 alias set (hundreds–thousands; count at build) |
| IP 2025.3 curve/family catalogs | ip_ingest B/D notes | vendor #3 set incl. MINDEF-linked names |
| Jauhar's working aliases | [workflow_standards.md](../../workflow_standards.md) | field-proven Mahakam set, confidence 1.0 |
| Variable affixes | techlog_ingest `A_variable_affixes.json` | suffix-stripping rules for `mnemonic_clean` (e.g. `GR_EDTC` → `GR`) — rules, not rows |

Merge arithmetic: 723 + 2,839 + Geolog + IP + Jauhar comfortably clears 5,000 raw; the honest
metric is **cross-vendor-validated** rows, so the acceptance test is: after dedup on
(mnemonic_clean → standard), ≥5,000 distinct original→standard mappings, and every mapping that
appears in ≥2 vendors gets `mapping_confidence ≥ 0.9`.

**Conflict-resolution rules (rule IDs `MD-*`), applied at build, logged per row:**
1. MD-01: vendors agree → confidence 0.95, source list concatenated.
2. MD-02: vendors disagree on standard (e.g. one maps `RD` → RT_DEEP, another → generic RES) →
   keep both rows `state='flagged'`, resolution recorded by engineer once; resolution persists as
   a `mnemonic_override` row (Jauhar's call outranks vendors, confidence 1.0).
3. MD-03: context-dependent mnemonics (unit disambiguates: `DT` µs/ft vs µs/m variants;
   log_type disambiguates: `TENS` WL vs drilling) → resolution requires the context key; the
   resolver API therefore takes `(mnemonic, unit, log_type)` not just the string.
4. MD-04: unknown mnemonic at index time → recorded in `mnemonic_unknown(mnemonic, count,
   example_file)` — the review queue that grows the dictionary by data.

Cross-check at build time: SONAR's four example mappings (p8: GR-group, RHOB-group, NPHI-group,
RT_DEEP-group) must all resolve identically in our merged dictionary — cheap sanity fixture.

## 3. Units dictionary

SONAR carries `unit_family` only as a column (p8). We make units first-class — needed by the
physics validator (target F) and curve QC (target E).

```sql
CREATE TABLE unit_family ( name TEXT PRIMARY KEY, quantity TEXT NOT NULL );
   -- e.g. ('density','mass/volume'), ('slowness','time/length')

CREATE TABLE unit (
  unit_id     INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,            -- canonical: 'g/cm3'
  family      TEXT NOT NULL REFERENCES unit_family,
  to_base_mul REAL NOT NULL,            -- value_base = mul*value + off
  to_base_off REAL NOT NULL DEFAULT 0,  -- nonzero only for temperature
  source      TEXT NOT NULL,
  UNIQUE(name, family)
);

CREATE TABLE unit_alias (
  alias TEXT NOT NULL, unit_id INTEGER NOT NULL REFERENCES unit,
  source TEXT NOT NULL, PRIMARY KEY(alias, unit_id)
);  -- 'G/C3','GM/CC','g/cc','K/M3' → g/cm3 (case-preserving aliases; LAS is case-chaotic)

CREATE TABLE unit_system (         -- Techlog SystemsUnits.xml shape
  system_name TEXT NOT NULL,       -- 'Metric','English','Mixed-Mahakam',...
  family      TEXT NOT NULL REFERENCES unit_family,
  unit_id     INTEGER NOT NULL REFERENCES unit,
  PRIMARY KEY(system_name, family)
);
```

Population: Techlog `SystemsUnits.xml` (+ `A_unit_systems.json` extraction — note it contains
case-duplicate keys like `frac`/`FRAC`, which is exactly why `unit_alias` is case-preserving),
Geolog units catalog, IP units. Conversion factors validated by round-trip property test
(x → base → x within 1 ppm) and by cross-vendor factor comparison; disagreement > 1e-6 relative
→ flagged row, engineer picks (physics is not vendor-negotiable).

## 4. Build order

1. Units (smallest, everything references it).
2. Super-dictionary merge (needs unit families for MD-03 disambiguation).
3. Well master + alias service (independent; parallel with 2).
4. Fixtures: SONAR p8 example groups; Jauhar alias list as golden set; 36 Techlog training LAS
   resolved end-to-end (every curve → standard + family + unit, every well name → master row).

All three are TABLES + a small resolver — no inference anywhere, identical output on every
rebuild from the same sources, every row carrying `source`/`rule` provenance.
