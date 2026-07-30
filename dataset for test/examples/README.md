# Example import datasets — one exemplar per format SandiBumi accepts

One synthetic field, three wells (**SANDI-01 / SANDI-02 / SANDI-03**), one shared
geology: a shale cap, a **gas sand (SAND-A)**, a **water sand (SAND-B)**, and a base
seal. Zone tops shift per well (SANDI-02 is 10 m deeper, SANDI-03 is 5 m shallower),
so tops import, multi-well plots and correlation all have real structure to show.
The numbers are internally consistent — the core porosities derive from the same
RHOB profile the LAS carries, and each SCAL file describes the same plugs — so
QC cross-checks (density porosity vs core, J-function fits) behave like a clean
real delivery, not random noise.

**These files are also fixtures**: `cargo test` parses every one of them
(`src-tauri/src/example_data_test.rs`), so if the app accepts a file here, it
accepts your real file of the same shape. Regenerate after editing the recipe with
`py -3 tools/make_example_data.py` (deterministic — a clean regeneration changes nothing).

## Import order that builds a complete little project

| # | File | Ribbon path | What to expect |
|---|------|-------------|----------------|
| 1 | `SANDI-01.las` `SANDI-02.las` `SANDI-03.las` | Data → Import Logs ▾ → **Import LAS…** (multi-select all 3) | 3 wells, ~394 rows each, metric. Standard six (GR/ILD/NPHI/RHOB/DT/SP) drive the log views; PEF + CALI additionally land in the generic curve store (Curve Catalog, set RAW). Each well has a deliberate 1-m NPHI/PEF null gap with a CALI washout mid-SAND-A — Bad-Hole QC will flag it. |
| 2 | `tops_multiwell.csv` | Data → Import Data ▾ → **Import Tops…** | 9 tops routed to all 3 wells by the WELL column (no well selection needed). |
| 3 | `well_locations.csv` | Data → Import Data ▾ → **Import Well Locations…** | 3 wells post on the Field Map, UTM 50S (Mahakam-range coordinates). |
| 4 | `deviation_SANDI-02.csv` | select SANDI-02 → Data → Import Data ▾ → **Import Deviation…** | Vertical to 300 m, builds to 25° by 800 m, holds. TVD < MD below the kickoff. |
| 5 | `core_rcal_SANDI-01.csv` | select SANDI-01 → Data → Import Data ▾ → **Import Core…** | ~15 plugs across both sands at native (off-grid) depths. Import Core opens the **confirm-mapping wizard**: check the detected columns, unit and percent notes, then Import. Gas-zone plugs read Sw ≈ 0.32, water-zone ≈ 0.84. |
| 5b | `core_rcal_multiwell.csv` | Data → Import Data ▾ → **Import Core…** (no well selection needed) | The WHOLE FIELD's core in one file, BLSO/PHR delivery shape: `WN` well-name column, a units row under the headers, suffixed mnemonics (`CPOR_2`), percent values. The wizard detects all of it and routes rows to SANDI-01/02/03 by name. It is also **wider than the four core slots** — `SO_1` (number), `LITH` (text), `SAMPLE_ID` (mixed): tick **Extra columns** to store those as point data at the plug depths (dataset `CORE`, see below). |
| 5c | `xrd_multiwell.txt` | Data → Import Data ▾ → **Import Aux…** → XRD | Tab-delimited TXT with a WELL column — rows route to all three wells in one import; the result box says where they went. |
| 6 | `scal_pc_long_SANDI-01.csv` | select SANDI-01 → Data → Import Data ▾ → **Import SCAL…** | 3 plugs × 8 Pc points, flat lab shape. Plug context (sample/depth/perm/poro) appears only on each plug's first row — merged-cell style — and forward-fills. |
| 7 | `scal_porous_plate_wide_SANDI-01.csv` | same menu | Corelab-style report: preamble lines, then a header row whose pressure columns ARE psi values (1…150), one row per plug, cells = brine sat %PV. |
| 8 | `scal_centrifuge_SANDI-01.csv` | same menu | Per-plug key-value blocks (SAMPLE/DEPTH/PERM/PORO) + a SPEED,PC,SW table. The table header appears only above the first block on purpose — the parser carries it over. |
| 9 | `petrography_SANDI-01.csv` | select SANDI-01 → Data → Import Data ▾ → **Import Aux…** → Petrography | 6 described intervals (TOP+BASE), text values. |
| 10 | `xrd_SANDI-01.csv` | same menu → XRD | 5 point samples (DEPTH only, no BASE), 9 mineral wt% columns. Clay fraction climbs into the base seal. |
| 11 | `perforations_SANDI-01.csv` | same menu → Perforation | 2 open intervals in SAND-A, 1 squeezed in SAND-B. |

After step 1–5 you can already run the full module chain, and after 6–8 the
saturation-height tools (SCAL Pc → J-function fit → Sw(height)) have real input.

## Malformed exemplars — for the failure-path tests (T-IMP-03 / T-IMP-04)

These two files are BROKEN on purpose, so you can watch the app refuse them gracefully
instead of having to doctor a file yourself:

| File | What's wrong | Expected on import |
|------|--------------|--------------------|
| `bad_dup_depth.las` | rows 10–14 repeat row 9's depth | Imports **with a warning** — status/History note says 5 row(s) dropped for duplicate depth; well SANDI-BAD-DUP appears with 35 rows. |
| `bad_null_depth.las` | every depth is −999.25 | **Clean error**, nothing imported — no orphan SANDI-BAD-NULL well row appears in the Wells pane or the database. |

Delete SANDI-BAD-DUP afterwards if you imported it into a real project.

## What each parser actually requires (for shaping your real files)

Headers are **case-insensitive** and **alias-resolved** — column order never matters.
The lists below are the aliases as implemented in `src-tauri/src/parsers.rs`.

- **LAS 2.0** — standard `~V/~W/~C/~A` sections; `NULL.` honored; well name from
  `WELL.` (multi-word names fine). Standard-six recognition: GR/GRN · RES_DEEP/RESD/
  RT/RES/DRES/ILD/LLD/AT90 · NPHI + 10 more neutron aliases · RHOB/RHOZ/RHOBED ·
  DT/DTC/DTCO/AC/DT24 · SP/SPC/SPR. **Every** curve in the file additionally lands in
  the generic store, whatever its mnemonic. Foot-indexed files auto-convert to the
  project's declared depth unit.
- **DLIS** — no text exemplar is possible (binary; `dlisio` reads but cannot write).
  Use any real .dlis: Data → Import Logs ▾ → Import DLIS… onto an **existing selected
  well**. Every scalar channel of every frame imports; frames get run numbers so a
  DLIS never silently overwrites same-named LAS curves. Requires the Python
  environment with `dlisio` installed.
- **Core CSV/TXT** — needs a DEPTH/DEPT/MD column; recognized value columns: CPOR (or
  POROSITY/PORO/POR/CPHI…), CPERM (or PERM/KAIR/KL/KH/K…), CGD (or GRAIN_DENSITY/
  RHOG), CSW (or SW). A WELL/WN/WELL NAME column routes rows per well. Percent values
  auto-convert when the column median says so; the file's depth unit converts to the
  project's. **Every other column** — lithology text, So, Kv/Kh, sample ids — can be
  ticked in the wizard and stored as **point data** at the plug depths (dataset `CORE`
  by default), typed per cell: numeric cells as numbers, everything else as text,
  stored verbatim (no percent/unit conversion on extras). So a wide lab export like
  `Core.csv` (in the parent folder) imports whole, in one pass.
- **SCAL CSV** — three shapes auto-offered at import: long (PC + SW columns
  required), wide porous-plate (SAMPLE + numeric psi headers), centrifuge blocks.
- **Tops CSV/TXT** — TOP/MARKER/FORMATION name column + DEPTH/MD column; WELL column
  makes it multi-well; headerless `NAME DEPTH` (or `WELL NAME DEPTH`) lines also
  accepted. Petrel exports work as-is.
- **Deviation CSV** — MD (or DEPTH/MEASURED_DEPTH) + INC (INCL/INCLINATION/DEVI) +
  AZI (AZIM/AZIMUTH/HAZI/AZM). Missing INC/AZI reads as vertical/north. Rows
  auto-sort by MD; duplicate stations drop (first wins).
- **Well locations CSV/TXT** — EASTING/UTM_X/X + NORTHING/UTM_Y/Y; optional WELL
  and UTM_ZONE columns (dialog default fills rows without a zone).
- **Aux (petrography / XRD / perforation)** — TOP (or DEPTH/FROM/MD) required, BASE
  (or BOTTOM/TO) optional; every other column becomes a data item, numeric or text.

## Known-good expected values (for eyeballing after import)

- SAND-A on any well: GR ≈ 45 API, RHOB ≈ 2.22, NPHI ≈ 0.14 (gas crossover on the
  N/D overlay), ILD ≈ 120 Ω·m. SAND-B: RHOB ≈ 2.34, NPHI ≈ 0.27, ILD ≈ 3.2 Ω·m.
- Archie in SAND-B with Rw ≈ 0.10 Ω·m gives Sw near 1; SAND-A gives Sw ≈ 0.3 —
  matching the core CSW by construction.
- Tops correlate 1:1 across the three wells with +10 m / −5 m structural shifts.
