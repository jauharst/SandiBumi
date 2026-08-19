# Reader inventory — SB-DIO-061 / DEC-052 constraint 1

Every file reader, its location, and its boundedness rule. A reader missing from this table
is a reader nobody proved bounded — keep it complete when adding one. Updated 2026-08-19
with the DEC-052 memory contract's landing; the diagnostics half of SB-DIO-061 (artifact +
line/record + affected count on every failure) is still open and tracked on the row.

## Rules

- **STREAMING** — reads bounded chunks with flat memory and NO size ceiling
  (`parsers::stream_text_lines`, DEC-052's ruled route; encoding decided per file in a
  bounded first pass, identical to the whole-file decoder's ladder).
- **WHOLE-FILE (CAPPED)** — decodes the whole delivery in memory through
  `parsers::read_text_file[_with_encoding]`, which refuses above the declared
  `WHOLE_FILE_TEXT_READER_MAX_BYTES` (256 MiB) BY NAME (DEC-052 constraint 3; bound
  derivation documented at the constant).
- **SUBPROCESS** — parsed outside our address space (rule 7); memory is the interpreter's,
  not ours (DEC-052 records DLIS as needing no contract here).
- **REFUSED** — format deliberately unsupported; the reader refuses by name.

## The table

| Reader | Location | Rule |
|---|---|---|
| LAS 2.0 curve import (all variants) | `parsers.rs parse_las_2_all*` | STREAMING (the one size-scaling text format) |
| LAS well-info probe | `parsers.rs` (~W block scan) | WHOLE-FILE (CAPPED) |
| Generic curve CSV export reader | `parsers.rs parse_csv_export` | WHOLE-FILE (CAPPED) |
| Tops CSV/TXT | `parsers.rs` tops readers | WHOLE-FILE (CAPPED) |
| Core table probe + import (CSV/TXT) | `parsers.rs probe_core_table` / `parse_core_table_mapped` | WHOLE-FILE (CAPPED) |
| Aux/point-data tables (XRD, CEC, petrography, perforations…) | `parsers.rs` aux readers | WHOLE-FILE (CAPPED) |
| SCAL Pc tables | `parsers.rs` SCAL readers | WHOLE-FILE (CAPPED) |
| Deviation survey CSV | `parsers.rs` (`DEV_MD_ALIASES` reader) | WHOLE-FILE (CAPPED) |
| Well locations | `parsers.rs` locations reader | WHOLE-FILE (CAPPED) |
| Delimited intake (LONG/WIDE/BLOCK) | `intake.rs` (through the same text funnel) | WHOLE-FILE (CAPPED) |
| DLIS import | `dlis.rs` (dlisio sidecar) | SUBPROCESS |
| Image/plate decode + conditioning | `images.rs` (Pillow sidecar; WebView decode for verbatim stores) | SUBPROCESS |
| `.xls` plate workbooks (BIFF) | `images.rs` (SB-DIO-058/-060) | REFUSED pending the BIFF5 ruling |
| Project database | DuckDB engine | Engine-managed (`db::tune_connection` caps its budget) |

Notes.

- Every text reader above funnels through `parsers::read_text_file[_with_encoding]` or
  `parsers::stream_text_lines` — the cap and the stream are enforced at the funnel, so a new
  reader inherits a rule instead of choosing one silently.
- UTF-16 deliveries (BOM or the even-offset newline-pair heuristic) fall back from the
  stream to the capped whole-file decoder: they are per-delivery tables in practice, and the
  encoding ladder's order is preserved exactly.
- Pinned by: `parsers::streaming_reader_tests` — flat-memory pin (peak carry bounded by one
  chunk while the input grows), line-for-line equivalence with the whole-file decoder on
  UTF-8/cp1252/CRLF/unterminated-tail shapes, and the oversize refusal naming the file and
  the ruling before reading a byte.
