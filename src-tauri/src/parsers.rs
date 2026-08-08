use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("las parse error: {0}")]
    Las(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

/// cp1252's 0x80–0x9F block — the only place it differs from Latin-1, and the source of every
/// byte that breaks a real delivery: smart quotes, en/em dashes, the bullet. Above 0x9F,
/// cp1252 IS Latin-1 (byte value == Unicode code point), so no table is needed there.
/// Undefined slots (0x81/0x8D/0x8F/0x90/0x9D) map to their control code points, as browsers do.
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// Decodes file bytes into text the way real-world deliveries actually arrive.
///
/// Field data is not reliably UTF-8. A CSV that passed through Excel, Word or an operator's
/// reporting tool on a Windows machine carries **cp1252** bytes, and a single one of them used
/// to fail an entire import with the unhelpful `io error: stream did not contain valid UTF-8`.
/// The case that found this: a 330 KB field core table, pure ASCII apart from **two** 0x95
/// bullets that begin a lithology description — 20,000 good rows rejected over two characters
/// in a comment field.
///
/// Order matters: a BOM is authoritative, so it is honoured first (Excel's "Unicode text"
/// export is UTF-16LE, which decoded as cp1252 would silently yield NUL-riddled nonsense
/// rather than an error). Only when there is no BOM and the bytes are not valid UTF-8 do we
/// fall back to cp1252 — which cannot itself fail, so an import is never refused over encoding
/// again. Bytes are never rejected, only interpreted; the worst case is a mangled character
/// inside a description, not a lost delivery.
fn decode_text(bytes: &[u8]) -> String {
    let utf16 = |chunks: &[u8], be: bool| -> String {
        let units: Vec<u16> = chunks
            .chunks_exact(2)
            .map(|p| if be { u16::from_be_bytes([p[0], p[1]]) } else { u16::from_le_bytes([p[0], p[1]]) })
            .collect();
        String::from_utf16_lossy(&units)
    };
    match bytes {
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, false),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, true),
        _ => match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(),
            Err(_) => bytes
                .iter()
                .map(|&b| match b {
                    0x80..=0x9F => CP1252_HIGH[(b - 0x80) as usize],
                    _ => b as char, // ASCII and, above 0x9F, Latin-1 == Unicode
                })
                .collect(),
        },
    }
}

/// Reads a text/delimited file, decoding it per `decode_text`. **Every** text import must go
/// through this rather than `read_to_string`/`BufReader<File>`, both of which reject a file
/// outright on one stray byte. Files here are per-well or per-delivery (single-digit MB), so
/// reading whole is the right trade for never refusing a real delivery.
pub fn read_text_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    Ok(decode_text(&std::fs::read(path)?))
}

/// A single deserialized row from a generic curve CSV export.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // CSV-export row DTO; paired with parse_csv_export below
pub struct LogDataRow {
    pub depth: f32,
    pub gr: Option<f32>,
    pub res: Option<f32>,
    pub nphi: Option<f32>,
    pub rhob: Option<f32>,
    pub dt: Option<f32>,
    pub sp: Option<f32>,
}

/// Columnar curve data ready to be handed to the DuckDB Appender.
#[derive(Debug, Clone, Default)]
pub struct CurveColumns {
    /// The index column's declared unit, verbatim from the ~C block (e.g. "M", "FT").
    /// `None` when the file declares none. Resolved against the project's depth unit at
    /// ingest — see `units::resolve_index_unit`; storing a foot index in a metric project
    /// was a silent corruption before this was carried through.
    pub depth_unit: Option<String>,
    pub depth: Vec<f32>,
    pub gr: Vec<f32>,
    pub res: Vec<f32>,
    pub nphi: Vec<f32>,
    pub rhob: Vec<f32>,
    pub dt: Vec<f32>,
    pub sp: Vec<f32>,
}

/// Parses a generic curve CSV export into columnar arrays, mapping missing values to `f32::NAN`.
#[allow(dead_code)] // generic-CSV importer, wired into the ribbon in a later increment
pub fn parse_csv_export<P: AsRef<Path>>(path: P) -> ParseResult<CurveColumns> {
    let text = read_text_file(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut cols = CurveColumns::default();
    for result in rdr.deserialize() {
        let row: LogDataRow = result?;
        cols.depth.push(row.depth);
        cols.gr.push(row.gr.unwrap_or(f32::NAN));
        cols.res.push(row.res.unwrap_or(f32::NAN));
        cols.nphi.push(row.nphi.unwrap_or(f32::NAN));
        cols.rhob.push(row.rhob.unwrap_or(f32::NAN));
        cols.dt.push(row.dt.unwrap_or(f32::NAN));
        cols.sp.push(row.sp.unwrap_or(f32::NAN));
    }
    Ok(cols)
}

/// Standard LAS null value sentinels, mapped strictly to `f32::NAN`.
const LAS_NULL_VALUES: [f32; 2] = [-999.25, -9999.0];

const NULL_REL_TOLERANCE: f32 = 1e-5;

fn matches_null(v: f32, null: f32) -> bool {
    (v - null).abs() <= null.abs().max(1.0) * NULL_REL_TOLERANCE
}

pub(crate) fn is_las_null(v: f32) -> bool {
    LAS_NULL_VALUES.iter().any(|&null| matches_null(v, null))
}

/// Null test honoring the file's own `~W NULL` declaration on top of the standard
/// sentinels — deliveries using e.g. -99999 or 999.25 otherwise import as data.
fn is_null_value(v: f32, declared: Option<f32>) -> bool {
    is_las_null(v) || declared.is_some_and(|null| matches_null(v, null))
}

/// Parse the NULL value from a `~W` block line ("NULL .  -999.25 : NULL VALUE").
fn parse_null_line(trimmed: &str) -> Option<f32> {
    if !trimmed.to_uppercase().starts_with("NULL") {
        return None;
    }
    trimmed.split(':').next()?.split_whitespace().last()?.parse::<f32>().ok()
}

fn parse_wrap_line(trimmed: &str) -> Option<bool> {
    let declaration = trimmed.split(':').next().unwrap_or(trimmed);
    let (mnemonic, rest) = declaration.split_once('.')?;
    if !mnemonic.trim().eq_ignore_ascii_case("WRAP") {
        return None;
    }
    match rest.split_whitespace().next()?.to_ascii_uppercase().as_str() {
        "YES" => Some(true),
        "NO" => Some(false),
        _ => None,
    }
}

enum LasSection {
    Header,
    WellBlock,
    CurveBlock,
    AsciiData,
}

/// Priority-ordered mnemonic aliases per target curve, mirroring the alias tables commercial suites/IP
/// ship (e.g. IP's CurveAlias.txt). Among the aliases present in a file, the one with the
/// most populated (non-null) samples wins; priority order only breaks ties. So a raw GR is
/// preferred over a normalized GRN when both are populated, but an all-null placeholder
/// (e.g. an empty simulated NPHIED) is skipped in favour of its populated sibling NPHI_LS.
// Only the LAS 2.0 index mnemonics DEPT/DEPTH. TDEP (Schlumberger) / MD indexes are handled
// by the column-0 fallback in parse_las_2 / parse_las_2_all, NOT listed here: the LAS index
// is always the first column, and matching TDEP/MD by name would let an auxiliary MD or TDEP
// *track* sitting in a later column steal the depth role from the true first-column index. So
// depth resolves to the first DEPT/DEPTH curve, else column 0 — never an all-NaN depth that
// would trip the standard_curves (well_id, depth) PK.
const DEPTH_ALIASES: [&str; 2] = ["DEPT", "DEPTH"];
const GR_ALIASES: [&str; 2] = ["GR", "GRN"];
const RES_ALIASES: [&str; 8] = ["RES_DEEP", "RESD", "RT", "RES", "DRES", "ILD", "LLD", "AT90"];
// Thermal (CNL-family) names lead so they win ties over epithermal/legacy tools;
// APS (APLC/FPLC) and sidewall (SNP) deliveries previously matched nothing and left
// the standard NPHI column all-NaN even though the curve was imported.
const NPHI_ALIASES: [&str; 11] =
    ["NPHI", "TNPH", "NPHIED", "NPHI_LS", "NPOR", "APLC", "HNPO", "NEUT", "FSTP", "FPLC", "SNP"];
const RHOB_ALIASES: [&str; 3] = ["RHOB", "RHOZ", "RHOBED"];
const DT_ALIASES: [&str; 5] = ["DT", "DTC", "DTCO", "AC", "DT24"];
const SP_ALIASES: [&str; 3] = ["SP", "SPC", "SPR"];

fn resolve_curve_index(curve_names: &[String], aliases: &[&str]) -> Option<usize> {
    aliases.iter().find_map(|alias| curve_names.iter().position(|n| n == alias))
}

/// Streams a LAS 2.0 file line-by-line (never loads the whole file into RAM), reading the
/// `~C` (Curve) block to map column indices and the `~A` (ASCII) block for the data rows.
pub fn parse_las_2<P: AsRef<Path>>(path: P) -> ParseResult<CurveColumns> {
    let text = read_text_file(path)?;

    let mut section = LasSection::Header;
    let mut curve_names: Vec<String> = Vec::new();
    let mut curve_units: Vec<Option<String>> = Vec::new();
    let mut cols = CurveColumns::default();

    // Index lookup into curve_names for the columns we care about, resolved once the
    // ~C block is fully parsed and the first ~A line arrives. For the six standard curves
    // we keep *all* matching-alias columns as candidates and buffer each; the populated one
    // is chosen at the end (some deliveries carry an all-null placeholder — e.g. an empty
    // simulated NPHIED — ahead of the column that actually holds the data, NPHI_LS).
    let mut idx_depth: Option<usize> = None;
    // Per standard curve: the candidate column indices (alias-priority order) and a parallel
    // buffer of their sampled values.
    let mut cand: [Vec<usize>; 6] = Default::default();
    let mut cand_buf: [Vec<Vec<f32>>; 6] = Default::default();
    let mut indices_resolved = false;

    // Some LAS exports (especially ones with many curves) wrap each logical depth row
    // across multiple physical lines rather than one line per row. Accumulate tokens and
    // drain a full row's worth at a time instead of assuming line == row.
    let mut token_buffer: Vec<f32> = Vec::new();
    let mut buffer_start_line: Option<usize> = None;
    let mut declared_null: Option<f32> = None;
    let mut wrapped = false;

    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('~') {
            section = match trimmed.chars().nth(1).map(|c| c.to_ascii_uppercase()) {
                Some('W') => LasSection::WellBlock,
                Some('C') => LasSection::CurveBlock,
                Some('A') => LasSection::AsciiData,
                _ => LasSection::Header,
            };
            continue;
        }

        match section {
            LasSection::Header => {
                if let Some(value) = parse_wrap_line(trimmed) {
                    wrapped = value;
                }
                continue;
            }
            LasSection::WellBlock => {
                if let Some(n) = parse_null_line(trimmed) {
                    declared_null = Some(n);
                }
                continue;
            }
            LasSection::CurveBlock => {
                if trimmed.starts_with('#') {
                    continue;
                }
                // LAS curve line format: "MNEM .UNIT  VALUE : DESCRIPTION"
                if let Some(mnem) = trimmed.split('.').next() {
                    curve_names.push(mnem.trim().to_uppercase());
                    // Same extraction as parse_las_2_all, kept parallel to curve_names so
                    // the index column's unit can be read once the depth column resolves.
                    curve_units.push(
                        trimmed
                            .split_once('.')
                            .and_then(|(_, rest)| rest.split_whitespace().next())
                            .map(|u| u.trim().to_string())
                            .filter(|u| !u.is_empty()),
                    );
                }
            }
            LasSection::AsciiData => {
                if trimmed.starts_with('#') {
                    continue;
                }
                if !indices_resolved {
                    // Fall back to column 0 (the LAS index column) when no mnemonic matches,
                    // matching parse_las_2_all — a TDEP/MD/other-indexed file must not produce
                    // an all-NaN depth column.
                    idx_depth = resolve_curve_index(&curve_names, &DEPTH_ALIASES).or(Some(0));
                    cols.depth_unit = idx_depth.and_then(|i| curve_units.get(i).cloned().flatten());
                    let alias_sets =
                        [&GR_ALIASES[..], &RES_ALIASES, &NPHI_ALIASES, &RHOB_ALIASES, &DT_ALIASES, &SP_ALIASES];
                    for (k, aliases) in alias_sets.iter().enumerate() {
                        cand[k] = resolve_curve_candidates(&curve_names, aliases);
                        cand_buf[k] = vec![Vec::new(); cand[k].len()];
                    }
                    indices_resolved = true;
                }
                let expected_per_row = curve_names.len();
                if expected_per_row == 0 {
                    continue;
                }

                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if !wrapped && tokens.len() != expected_per_row {
                    return Err(ParseError::Las(format!(
                        "line {line_number}: ASCII row has {} value(s), but ~C declares {expected_per_row} columns (truncated or corrupt LAS)",
                        tokens.len()
                    )));
                }
                if token_buffer.is_empty() && !tokens.is_empty() {
                    buffer_start_line = Some(line_number);
                }
                for tok in tokens {
                    let v: f32 = tok
                        .parse()
                        .map_err(|e| ParseError::Las(format!("line {line_number}: bad numeric token '{tok}': {e}")))?;
                    // `f32::from_str` accepts "inf"/"-inf" and overflows a cell like `1.0E+40` to
                    // infinity. Everything downstream screens for missing with `is_nan()` only
                    // (modules::is_missing), so an infinity survives into the compute cores and
                    // poisons z-scores and comparison sorts. The DLIS path already strips exactly
                    // this (dlis.rs); mirror it so both importers agree on what "missing" means.
                    token_buffer.push(if v.is_finite() { v } else { f32::NAN });
                }

                while token_buffer.len() >= expected_per_row {
                    let row: Vec<f32> = token_buffer.drain(0..expected_per_row).collect();
                    let get = |idx: Option<usize>| -> f32 {
                        idx.and_then(|i| row.get(i).copied())
                            .map(|v| if is_null_value(v, declared_null) { f32::NAN } else { v })
                            .unwrap_or(f32::NAN)
                    };

                    cols.depth.push(get(idx_depth));
                    for k in 0..6 {
                        for (j, &ci) in cand[k].iter().enumerate() {
                            cand_buf[k][j].push(get(Some(ci)));
                        }
                    }
                    if token_buffer.is_empty() {
                        buffer_start_line = None;
                    }
                }
            }
        }
    }

    // A short/truncated ~A row leaves tokens that never fill a complete column set; from
    // that point on every value is shifted a column left (GR lands in RES, etc.). Fail
    // loudly rather than silently mis-columning the rest of the file.
    if !token_buffer.is_empty() {
        return Err(ParseError::Las(format!(
            "line {}: ASCII data ended with {} leftover token(s) not forming a full {}-column row (truncated or corrupt LAS?)",
            buffer_start_line.unwrap_or(0),
            token_buffer.len(),
            curve_names.len()
        )));
    }

    // Choose, per standard curve, the candidate column with the most finite samples (ties
    // broken by alias priority, since we scan in priority order and only replace on strictly
    // greater coverage). This skips all-null placeholder columns in favour of a populated one.
    let n = cols.depth.len();
    let pick = |cands: &[Vec<f32>]| -> Vec<f32> {
        let mut best: Option<&Vec<f32>> = None;
        let mut best_finite: i64 = -1;
        for c in cands {
            let finite = c.iter().filter(|v| !v.is_nan()).count() as i64;
            if finite > best_finite {
                best_finite = finite;
                best = Some(c);
            }
        }
        best.cloned().unwrap_or_else(|| vec![f32::NAN; n])
    };
    cols.gr = pick(&cand_buf[0]);
    cols.res = pick(&cand_buf[1]);
    cols.nphi = pick(&cand_buf[2]);
    cols.rhob = pick(&cand_buf[3]);
    cols.dt = pick(&cand_buf[4]);
    cols.sp = pick(&cand_buf[5]);

    Ok(cols)
}

/// Summary of what `sanitize_curve_columns` removed, so the importer can report it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DepthSanitizeReport {
    /// Rows dropped because the depth index was non-finite (unresolved/blank/NaN sentinel).
    pub nonfinite: usize,
    /// Rows dropped because their depth duplicated an earlier kept row.
    pub duplicate: usize,
}

impl DepthSanitizeReport {
    pub fn total(&self) -> usize {
        self.nonfinite + self.duplicate
    }
    pub fn is_clean(&self) -> bool {
        self.total() == 0
    }
}

/// Row indices to keep so a depth column can satisfy a `(…, depth)` PRIMARY KEY: drops
/// non-finite depths and depths duplicating an earlier kept row (first occurrence wins, file
/// order preserved). Shared by [`sanitize_curve_columns`] and [`sanitize_las_frame`] so the
/// standard-curves and generic-store import paths drop identical rows for the same file.
pub(crate) fn depth_keep_indices(depth: &[f32]) -> (Vec<usize>, DepthSanitizeReport) {
    let n = depth.len();
    let mut keep: Vec<usize> = Vec::with_capacity(n);
    let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::with_capacity(n);
    let mut report = DepthSanitizeReport::default();
    for (i, &d) in depth.iter().enumerate() {
        if !d.is_finite() {
            report.nonfinite += 1;
            continue;
        }
        // Normalize signed zero: +0.0 and -0.0 have distinct bit patterns but DuckDB's FLOAT
        // PK treats them equal, so a +0.0/-0.0 pair must dedup here too. NaN already excluded.
        let key = if d == 0.0 { 0u32 } else { d.to_bits() };
        if !seen.insert(key) {
            report.duplicate += 1;
            continue;
        }
        keep.push(i);
    }
    (keep, report)
}

/// Drops rows the `standard_curves` (well_id, depth) PRIMARY KEY cannot accept: a non-finite
/// depth (an unrecognized/blank index that resolved to NaN) or a depth that duplicates an
/// earlier row (a spliced/merged LAS with repeat sections). Without this, a single such file
/// aborts the whole import with a cryptic PK-constraint error and leaves an orphan well row.
/// The first occurrence of each finite depth is kept in file order; reads re-sort by depth
/// (`fetch_curve_frame … ORDER BY depth`), so storage order here is irrelevant. A clean
/// column is left untouched (no reallocation).
pub fn sanitize_curve_columns(cols: &mut CurveColumns) -> DepthSanitizeReport {
    let (keep, report) = depth_keep_indices(&cols.depth);
    if report.is_clean() {
        return report;
    }
    let take = |src: &[f32]| -> Vec<f32> { keep.iter().map(|&i| src[i]).collect() };
    cols.depth = take(&cols.depth);
    cols.gr = take(&cols.gr);
    cols.res = take(&cols.res);
    cols.nphi = take(&cols.nphi);
    cols.rhob = take(&cols.rhob);
    cols.dt = take(&cols.dt);
    cols.sp = take(&cols.sp);
    report
}

/// All curve-column indices whose mnemonic matches one of `aliases`, in alias-priority order.
fn resolve_curve_candidates(curve_names: &[String], aliases: &[&str]) -> Vec<usize> {
    aliases
        .iter()
        .filter_map(|alias| curve_names.iter().position(|n| n == alias))
        .collect()
}

/// One curve read verbatim from a LAS `~C` block plus its column of samples — the raw
/// material for the generic curve store (Phase 6). Unlike `CurveColumns` (fixed 6
/// mnemonics), this keeps **every** curve the file carries, at whatever mnemonic and unit
/// it was recorded under.
#[derive(Debug, Clone)]
pub struct RawLasCurve {
    pub mnemonic: String,
    pub unit: Option<String>,
    pub values: Vec<f32>,
}

/// A full LAS file decomposed into its depth column and every other curve, preserving
/// original mnemonics and units (Phase 6 generic import). Depth is separated out because
/// it's the shared index every other curve is sampled against.
#[derive(Debug, Clone, Default)]
pub struct LasFrame {
    // depth_mnemonic/depth_unit feed the Phase 6c TVD-scale + well-header UI (is the file's
    // index depth in metres or feet?); captured now with the rest of the frame.
    #[allow(dead_code)]
    pub depth_mnemonic: String,
    /// Read at ingest to reconcile the file's index against the project's depth unit
    /// (`units::resolve_index_unit`). Deliberately NOT `#[allow(dead_code)]` any more —
    /// it was silenced here for a whole release cycle, which is precisely what hid the
    /// fact that nothing ever consulted it.
    pub depth_unit: Option<String>,
    pub depth: Vec<f32>,
    pub curves: Vec<RawLasCurve>,
}

/// Applies the same depth sanitation as [`sanitize_curve_columns`] to a full [`LasFrame`]:
/// drops rows with a non-finite or duplicate depth from `depth` and every curve's `values`
/// in lockstep, so the generic `curve_samples` (curve_id, depth) PK can't trip on the same
/// spliced/merged LAS the standard-curves path guards against. A clean frame is untouched.
pub fn sanitize_las_frame(frame: &mut LasFrame) -> DepthSanitizeReport {
    let (keep, report) = depth_keep_indices(&frame.depth);
    if report.is_clean() {
        return report;
    }
    let take = |src: &[f32]| -> Vec<f32> {
        keep.iter().map(|&i| src.get(i).copied().unwrap_or(f32::NAN)).collect()
    };
    frame.depth = take(&frame.depth);
    for c in &mut frame.curves {
        c.values = take(&c.values);
    }
    report
}

/// Parses a LAS 2.0 file keeping **all** curves (mnemonic + unit + values), streaming the
/// same way as `parse_las_2` but without collapsing to the fixed standard set. The first
/// column recognized as depth (by `DEPTH_ALIASES`, else column 0) becomes the shared
/// index; every other column is returned as its own `RawLasCurve`.
pub fn parse_las_2_all<P: AsRef<Path>>(path: P) -> ParseResult<LasFrame> {
    let text = read_text_file(path)?;

    let mut section = LasSection::Header;
    let mut curve_names: Vec<String> = Vec::new();
    let mut curve_units: Vec<Option<String>> = Vec::new();
    // One value column per curve, filled in ~A order.
    let mut columns: Vec<Vec<f32>> = Vec::new();
    let mut idx_depth: Option<usize> = None;
    let mut indices_resolved = false;
    let mut token_buffer: Vec<f32> = Vec::new();
    let mut buffer_start_line: Option<usize> = None;
    let mut declared_null: Option<f32> = None;
    let mut wrapped = false;

    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('~') {
            section = match trimmed.chars().nth(1).map(|c| c.to_ascii_uppercase()) {
                Some('W') => LasSection::WellBlock,
                Some('C') => LasSection::CurveBlock,
                Some('A') => LasSection::AsciiData,
                _ => LasSection::Header,
            };
            continue;
        }

        match section {
            LasSection::Header => {
                if let Some(value) = parse_wrap_line(trimmed) {
                    wrapped = value;
                }
                continue;
            }
            LasSection::WellBlock => {
                if let Some(n) = parse_null_line(trimmed) {
                    declared_null = Some(n);
                }
                continue;
            }
            LasSection::CurveBlock => {
                if trimmed.starts_with('#') {
                    continue;
                }
                // "MNEM .UNIT  VALUE : DESCRIPTION" — mnemonic before the first '.',
                // unit is the token immediately after it (may be empty).
                let mnem = trimmed.split('.').next().unwrap_or("").trim().to_uppercase();
                if mnem.is_empty() {
                    continue;
                }
                let unit = trimmed
                    .split_once('.')
                    .and_then(|(_, rest)| rest.split_whitespace().next())
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty());
                curve_names.push(mnem);
                curve_units.push(unit);
            }
            LasSection::AsciiData => {
                if trimmed.starts_with('#') {
                    continue;
                }
                if !indices_resolved {
                    idx_depth = resolve_curve_index(&curve_names, &DEPTH_ALIASES).or(Some(0));
                    columns = vec![Vec::new(); curve_names.len()];
                    indices_resolved = true;
                }
                let expected_per_row = curve_names.len();
                if expected_per_row == 0 {
                    continue;
                }
                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if !wrapped && tokens.len() != expected_per_row {
                    return Err(ParseError::Las(format!(
                        "line {line_number}: ASCII row has {} value(s), but ~C declares {expected_per_row} columns (truncated or corrupt LAS)",
                        tokens.len()
                    )));
                }
                if token_buffer.is_empty() && !tokens.is_empty() {
                    buffer_start_line = Some(line_number);
                }
                for tok in tokens {
                    let v: f32 =
                        tok.parse().map_err(|e| ParseError::Las(format!("line {line_number}: bad numeric token '{tok}': {e}")))?;
                    token_buffer.push(v);
                }
                while token_buffer.len() >= expected_per_row {
                    let row: Vec<f32> = token_buffer.drain(0..expected_per_row).collect();
                    for (i, raw) in row.iter().enumerate() {
                        let v = if is_null_value(*raw, declared_null) { f32::NAN } else { *raw };
                        columns[i].push(v);
                    }
                    if token_buffer.is_empty() {
                        buffer_start_line = None;
                    }
                }
            }
        }
    }

    // A short/truncated ~A row leaves tokens that never fill a complete column set; from
    // that point on every value is shifted a column left. Fail loudly rather than silently
    // mis-columning the rest of the file.
    if !token_buffer.is_empty() {
        return Err(ParseError::Las(format!(
            "line {}: ASCII data ended with {} leftover token(s) not forming a full {}-column row (truncated or corrupt LAS?)",
            buffer_start_line.unwrap_or(0),
            token_buffer.len(),
            curve_names.len()
        )));
    }

    let depth_idx = idx_depth.unwrap_or(0);
    if curve_names.is_empty() || depth_idx >= curve_names.len() {
        return Err(ParseError::Las("LAS file has no curve columns".into()));
    }

    let mut frame = LasFrame {
        depth_mnemonic: curve_names[depth_idx].clone(),
        depth_unit: curve_units[depth_idx].clone(),
        depth: columns.get(depth_idx).cloned().unwrap_or_default(),
        curves: Vec::new(),
    };
    for i in 0..curve_names.len() {
        if i == depth_idx {
            continue;
        }
        frame.curves.push(RawLasCurve {
            mnemonic: curve_names[i].clone(),
            unit: curve_units[i].clone(),
            values: columns.get(i).cloned().unwrap_or_default(),
        });
    }
    Ok(frame)
}

/// Reads just the ~W (Well Information) block to find the WELL mnemonic's value, falling
/// back to the file's stem if the block is missing or the value is blank.
pub fn extract_well_name<P: AsRef<Path>>(path: P) -> ParseResult<String> {
    let path = path.as_ref();
    let text = read_text_file(path)?;

    let mut in_well_block = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('~') {
            in_well_block = trimmed.chars().nth(1).map(|c| c.to_ascii_uppercase()) == Some('W');
            continue;
        }
        let upper = trimmed.to_uppercase();
        let well_line = in_well_block
            && upper.starts_with("WELL")
            && matches!(upper.as_bytes().get(4), None | Some(b'.') | Some(b' ') | Some(b'\t'));
        if well_line {
            if let Some(colon_idx) = trimmed.rfind(':') {
                // "WELL .        SANDI SOUTH-01   : WELL" — the value is everything
                // between the mnemonic(+unit) and the colon, NOT just the last token
                // (multi-word well names must survive intact).
                let after = trimmed[4..colon_idx].trim_start();
                let after = after.strip_prefix('.').unwrap_or(after);
                let value = match after.find(char::is_whitespace) {
                    Some(i) => after[i..].trim(),
                    None => after.trim(),
                };
                if !value.is_empty() {
                    return Ok(value.to_string());
                }
            }
        }
    }

    Ok(path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "UNKNOWN".to_string()))
}

/// Columnar core plug data ready for the DuckDB Appender. Missing/unparseable cells map
/// to `f32::NAN` (never `Option<f32>`, per this project's missing-value convention).
#[derive(Debug, Clone, Default)]
pub struct CoreColumns {
    pub depth: Vec<f32>,
    pub cpor: Vec<f32>,
    pub cperm: Vec<f32>,
    pub cgd: Vec<f32>,
    pub csw: Vec<f32>,
}

const CORE_DEPTH_ALIASES: [&str; 3] = ["DEPTH", "DEPT", "MD"];
const CORE_CPOR_ALIASES: [&str; 7] = ["CPOR", "CORE_POR", "PHI_CORE", "CPHI", "POROSITY", "PORO", "POR"];
const CORE_CPERM_ALIASES: [&str; 8] = ["CPERM", "CORE_PERM", "KAIR", "KL", "KH", "PERMEABILITY", "PERM", "K"];
// GDEN: a real core-log delivery header (`GDEN_1`, resolved via the `_` boundary rule).
const CORE_CGD_ALIASES: [&str; 5] = ["CGD", "GRAIN_DENSITY", "GRAIN_DEN", "RHOG", "GDEN"];
const CORE_CSW_ALIASES: [&str; 3] = ["CSW", "CORE_SW", "SW"];

/// True when `header` is `alias` on its own or followed by a unit/qualifier
/// ("CPOR (%)", "KAIR MD") — but not when the alias is merely a prefix of a longer
/// word ("K" must not match "KB", "POR" must not match "POROSITY"'s tail).
fn header_matches(header: &str, alias: &str) -> bool {
    header == alias
        || (header.starts_with(alias)
            && header[alias.len()..].chars().next().is_some_and(|c| !c.is_ascii_alphanumeric()))
}

fn resolve_header_index(headers: &[String], aliases: &[&str]) -> Option<usize> {
    aliases
        .iter()
        .find_map(|alias| headers.iter().position(|h| header_matches(h, alias)))
}

/// RCAL reports usually quote porosity and saturation in percent (22.5) while every
/// curve in SandiBumi is v/v. Values are only fractions when they all sit in [0, 1],
/// so a median above 1.5 can only mean percent — divide through by 100.
fn percent_to_fraction(vals: &mut [f32]) {
    let mut finite: Vec<f32> = vals.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return;
    }
    finite.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if finite[finite.len() / 2] > 1.5 {
        for v in vals.iter_mut() {
            *v /= 100.0;
        }
    }
}

/// Parses a routine-core-analysis CSV (arbitrary column order, alias-resolved headers —
/// same convention as the LAS mnemonic aliases above) into columnar arrays. Depths that
/// don't line up with the log's standard depth grid are expected and fine: core data is
/// stored and fetched independently, not aligned onto `standard_curves`.
pub fn parse_core_csv<P: AsRef<Path>>(path: P) -> ParseResult<CoreColumns> {
    let text = read_text_file(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(text.as_bytes());

    let headers: Vec<String> =
        rdr.headers()?.iter().map(|h| h.trim().to_uppercase()).collect();
    let idx_depth = resolve_header_index(&headers, &CORE_DEPTH_ALIASES)
        .ok_or_else(|| ParseError::Las("core CSV has no recognizable DEPTH column".into()))?;
    let idx_cpor = resolve_header_index(&headers, &CORE_CPOR_ALIASES);
    let idx_cperm = resolve_header_index(&headers, &CORE_CPERM_ALIASES);
    let idx_cgd = resolve_header_index(&headers, &CORE_CGD_ALIASES);
    let idx_csw = resolve_header_index(&headers, &CORE_CSW_ALIASES);

    let mut cols = CoreColumns::default();
    for result in rdr.records() {
        let record = result?;
        let get = |idx: Option<usize>| -> f32 {
            idx.and_then(|i| record.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(f32::NAN)
        };
        let depth = get(Some(idx_depth));
        if depth.is_nan() {
            continue; // a row with no depth can't be stored (PRIMARY KEY includes depth)
        }
        cols.depth.push(depth);
        cols.cpor.push(get(idx_cpor));
        cols.cperm.push(get(idx_cperm));
        cols.cgd.push(get(idx_cgd));
        cols.csw.push(get(idx_csw));
    }
    percent_to_fraction(&mut cols.cpor);
    percent_to_fraction(&mut cols.csw);
    // Dedup duplicate depths (first occurrence wins, file order kept) so a repeated plug depth
    // can't abort the whole well's core import on the core_data (well_id, depth) PK — mirrors the
    // LAS sanitize path. (NaN depths were already skipped above.)
    let (keep, report) = depth_keep_indices(&cols.depth);
    if !report.is_clean() {
        let take = |src: &[f32]| -> Vec<f32> { keep.iter().map(|&i| src[i]).collect() };
        cols.depth = take(&cols.depth);
        cols.cpor = take(&cols.cpor);
        cols.cperm = take(&cols.cperm);
        cols.cgd = take(&cols.cgd);
        cols.csw = take(&cols.csw);
    }
    Ok(cols)
}

/// One row of a SCAL capillary-pressure delivery: a Pc/Sw point with its plug's
/// perm/poro repeated per row (the usual flat lab-report export shape).
#[derive(Debug, Clone)]
pub struct ScalPcRecord {
    pub sample_no: Option<i32>,
    pub depth: Option<f32>,
    pub perm: f32,
    pub poro: f32,
    pub pc: f32,
    pub sw: f32,
}

const SCAL_SAMPLE_ALIASES: [&str; 4] = ["SAMPLE", "SAMPLE_NO", "PLUG", "NO"];
const SCAL_PC_ALIASES: [&str; 4] = ["PC", "PCAP", "CAP_PRESSURE", "PRESSURE"];
const SCAL_SW_ALIASES: [&str; 4] = ["SW", "SAT", "WATER_SATURATION", "SWI"];

/// Parses a SCAL Pc CSV (alias-resolved headers like the core CSV parser): needs PC and
/// SW columns; SAMPLE/DEPTH/PERM/PORO are optional per-plug context. Sw in percent is
/// detected and divided down; porosity likewise.
pub fn parse_scal_csv<P: AsRef<Path>>(path: P) -> ParseResult<Vec<ScalPcRecord>> {
    let delim = scal_delimiter(&path)?;
    let text = read_text_file(&path)?;
    let mut rdr =
        csv::ReaderBuilder::new().delimiter(delim).has_headers(true).from_reader(text.as_bytes());

    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.trim().to_uppercase()).collect();
    let idx_pc = resolve_header_index(&headers, &SCAL_PC_ALIASES)
        .ok_or_else(|| ParseError::Las("SCAL CSV has no recognizable PC column".into()))?;
    let idx_sw = resolve_header_index(&headers, &SCAL_SW_ALIASES)
        .ok_or_else(|| ParseError::Las("SCAL CSV has no recognizable SW column".into()))?;
    let idx_sample = resolve_header_index(&headers, &SCAL_SAMPLE_ALIASES);
    let idx_depth = resolve_header_index(&headers, &CORE_DEPTH_ALIASES);
    let idx_perm = resolve_header_index(&headers, &CORE_CPERM_ALIASES);
    let idx_poro = resolve_header_index(&headers, &CORE_CPOR_ALIASES);

    let mut out = Vec::new();
    // Merged-cell lab exports write the plug context (sample/depth/perm/poro) only on
    // each plug's FIRST row; forward-fill blanks from the previous row when the file has
    // a SAMPLE column at all. A row naming a DIFFERENT sample starts a new plug and
    // inherits nothing.
    let mut last: Option<(Option<i32>, Option<f32>, f32, f32)> = None;
    for result in rdr.records() {
        let record = result?;
        let get = |idx: Option<usize>| -> f32 {
            idx.and_then(|i| record.get(i)).and_then(parse_f32_cell).unwrap_or(f32::NAN)
        };
        let pc = get(Some(idx_pc));
        let sw = get(Some(idx_sw));
        if pc.is_nan() || sw.is_nan() {
            continue;
        }
        let mut sample_no = idx_sample.and_then(|i| record.get(i)).and_then(parse_sample_no);
        let mut depth = {
            let d = get(idx_depth);
            if d.is_nan() { None } else { Some(d) }
        };
        let mut perm = get(idx_perm);
        let mut poro = get(idx_poro);
        if idx_sample.is_some() {
            if let Some((ls, ld, lk, lp)) = last {
                if sample_no.is_none() || sample_no == ls {
                    if sample_no.is_none() {
                        sample_no = ls;
                    }
                    if depth.is_none() {
                        depth = ld;
                    }
                    if perm.is_nan() {
                        perm = lk;
                    }
                    if poro.is_nan() {
                        poro = lp;
                    }
                }
            }
            last = Some((sample_no, depth, perm, poro));
        }
        out.push(ScalPcRecord { sample_no, depth, perm, poro, pc, sw });
    }

    // Percent detection over the whole file (same heuristic as core CSVs).
    let mut sws: Vec<f32> = out.iter().map(|r| r.sw).collect();
    percent_to_fraction(&mut sws);
    for (r, s) in out.iter_mut().zip(&sws) {
        r.sw = *s;
    }
    let mut poros: Vec<f32> = out.iter().map(|r| r.poro).collect();
    percent_to_fraction(&mut poros);
    for (r, p) in out.iter_mut().zip(&poros) {
        r.poro = *p;
    }
    Ok(out)
}

/// Sample/plug ids in SCAL deliveries are usually plain numbers ("4") but centrifuge
/// workbooks often letter them ("12A", "S-16A"): fall back to the first run of digits.
fn parse_sample_no(s: &str) -> Option<i32> {
    let t = s.trim();
    if let Ok(n) = t.parse::<i32>() {
        return Some(n);
    }
    let start = t.find(|c: char| c.is_ascii_digit())?;
    let digits: String = t[start..].chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<i32>().ok()
}

/// Parses a numeric cell tolerantly: plain f32 first, then the Excel regional shapes lab
/// exports actually contain — thousands separators ("1,250", "2,695.3": every comma group
/// is exactly 3 digits) and a lone decimal comma ("98,5" → 98.5, only in ';'-delimited or
/// quoted cells). Anything ambiguous stays unparsed (None) rather than guessed.
fn parse_f32_cell(s: &str) -> Option<f32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(v) = t.parse::<f32>() {
        return Some(v);
    }
    // Thousands groups: "1,250" / "2,695.3" — the last group may carry the decimal point.
    let groups: Vec<&str> = t.split(',').collect();
    if groups.len() >= 2
        && !groups[0].is_empty()
        && groups[1..].iter().enumerate().all(|(i, g)| {
            let last = i + 1 == groups.len() - 1;
            let int_part = if last { g.split('.').next().unwrap_or("") } else { g };
            int_part.len() == 3
                && int_part.chars().all(|c| c.is_ascii_digit())
                && (!last || g.chars().all(|c| c.is_ascii_digit() || c == '.'))
        })
    {
        if let Ok(v) = t.replace(',', "").parse::<f32>() {
            return Some(v);
        }
    }
    // Regional decimal comma: exactly one comma, no dot ("98,5", "2,1").
    if t.matches(',').count() == 1 && !t.contains('.') {
        if let Ok(v) = t.replace(',', ".").parse::<f32>() {
            return Some(v);
        }
    }
    None
}

/// Detects a SCAL file's delimiter: Excel under Indonesian/European regional settings
/// writes ';' as the list separator. Decided from the first non-empty line only, so
/// decimal commas inside data cells cannot outvote the real separator.
fn scal_delimiter<P: AsRef<Path>>(path: P) -> ParseResult<u8> {
    let text = read_text_file(&path)?;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let semis = line.matches(';').count();
        let commas = line.matches(',').count();
        return Ok(if semis > commas { b';' } else { b',' });
    }
    Ok(b',')
}

/// Parses a pressure-column header cell ("8", "45 psi", "1,000") to psi. Trailing unit
/// words and parens are stripped; a cell with no leading number ("Remarks") is not a
/// pressure column.
fn parse_pressure_header(s: &str) -> Option<f32> {
    let t = s
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c.is_whitespace() || c == '(' || c == ')');
    parse_f32_cell(t).filter(|p| *p > 0.0)
}

/// How many usable cells a raw CSV row has (flexible readers keep trailing empties).
fn non_empty_cells(record: &csv::StringRecord) -> usize {
    record.iter().filter(|c| !c.trim().is_empty()).count()
}

/// Parses a porous-plate capillary-pressure table in the wide lab-report shape
/// (Corelab-style): free-form preamble lines (company/well/OB-stress), then a header row
/// with SAMPLE / DEPTH / PERM / PORO columns plus one column per pressure step whose
/// header IS the pressure in psi (1, 2, 4, 8, ... 150), then one row per plug whose cells
/// are brine saturation in %PV. Unpivots to the long Pc/Sw records `scal_pc` stores.
pub fn parse_scal_wide_csv<P: AsRef<Path>>(path: P) -> ParseResult<Vec<ScalPcRecord>> {
    let delim = scal_delimiter(&path)?;
    let text = read_text_file(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    // Locate the header row: the first row with a recognizable SAMPLE column AND at least
    // three numeric-headed pressure columns (a real porous-plate table has ~12; requiring 3
    // keeps preamble rows that merely contain a stray number from matching).
    let mut idx_sample = None;
    let mut idx_depth = None;
    let mut idx_perm = None;
    let mut idx_poro = None;
    let mut pressure_cols: Vec<(usize, f32)> = Vec::new(); // (column index, Pc psi)
    let mut out = Vec::new();
    let mut header_found = false;

    for result in rdr.records() {
        let record = result?;
        if !header_found {
            let headers: Vec<String> = record.iter().map(|h| h.trim().to_uppercase()).collect();
            let s = resolve_header_index(&headers, &SCAL_SAMPLE_ALIASES);
            if s.is_none() {
                continue;
            }
            let d = resolve_header_index(&headers, &CORE_DEPTH_ALIASES);
            let k = resolve_header_index(&headers, &CORE_CPERM_ALIASES);
            let p = resolve_header_index(&headers, &CORE_CPOR_ALIASES);
            let meta: Vec<usize> = [s, d, k, p].iter().flatten().copied().collect();
            let pcols: Vec<(usize, f32)> = headers
                .iter()
                .enumerate()
                .filter(|(i, _)| !meta.contains(i))
                .filter_map(|(i, h)| parse_pressure_header(h).map(|pc| (i, pc)))
                .collect();
            if pcols.len() < 3 {
                continue; // not the table header — keep scanning the preamble
            }
            (idx_sample, idx_depth, idx_perm, idx_poro) = (s, d, k, p);
            pressure_cols = pcols;
            header_found = true;
            continue;
        }

        // Data row: one plug.
        let get = |idx: Option<usize>| -> f32 {
            idx.and_then(|i| record.get(i)).and_then(parse_f32_cell).unwrap_or(f32::NAN)
        };
        let sample_no = idx_sample.and_then(|i| record.get(i)).and_then(parse_sample_no);
        let depth = {
            let d = get(idx_depth);
            if d.is_nan() { None } else { Some(d) }
        };
        // A real plug row always identifies itself (sample and/or depth). Rows that don't —
        // repeated per-page header rows (whose pressure headers ARE numbers and would
        // otherwise import as phantom Sw points), "Average"/stat footers, units rows —
        // are layout, not data.
        if sample_no.is_none() && depth.is_none() {
            continue;
        }
        let perm = get(idx_perm);
        let poro = get(idx_poro);
        for &(col, pc) in &pressure_cols {
            let sw = record.get(col).and_then(parse_f32_cell);
            if let Some(sw) = sw {
                out.push(ScalPcRecord { sample_no, depth, perm, poro, pc, sw });
            }
        }
    }

    if !header_found {
        return Err(ParseError::Las(
            "porous-plate CSV: no header row with a SAMPLE column and pressure (psi) columns".into(),
        ));
    }

    // %PV → v/v over the whole file (same heuristic as every core/SCAL import).
    let mut sws: Vec<f32> = out.iter().map(|r| r.sw).collect();
    percent_to_fraction(&mut sws);
    for (r, s) in out.iter_mut().zip(&sws) {
        r.sw = *s;
    }
    let mut poros: Vec<f32> = out.iter().map(|r| r.poro).collect();
    percent_to_fraction(&mut poros);
    for (r, p) in out.iter_mut().zip(&poros) {
        r.poro = *p;
    }
    Ok(out)
}

/// Parses a centrifuge capillary-pressure delivery: one or more per-plug blocks, each a
/// short key-value header (SAMPLE / DEPTH / PERM / PORO lines, one per row) followed by a
/// Pc/Sw table (extra columns like speed-RPM are ignored). A new SAMPLE line starts the
/// next plug — the shape of the digitized per-plug workbooks merged into one CSV; a file
/// holding a single plug (one block) is the same format.
pub fn parse_scal_centrifuge_csv<P: AsRef<Path>>(path: P) -> ParseResult<Vec<ScalPcRecord>> {
    let delim = scal_delimiter(&path)?;
    let text = read_text_file(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut sample_no: Option<i32> = None;
    let mut depth: Option<f32> = None;
    let mut perm = f32::NAN;
    let mut poro = f32::NAN;
    let mut idx_pc: Option<usize> = None;
    let mut idx_sw: Option<usize> = None;
    let mut out: Vec<ScalPcRecord> = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let cells: Vec<String> = record.iter().map(|c| c.trim().to_uppercase()).collect();
        if non_empty_cells(&record) == 0 {
            continue;
        }

        // Key-value metadata line: exactly (key, value) — never a table row, which has
        // its Pc AND Sw cells (plus usually a speed column) populated.
        if non_empty_cells(&record) <= 2 && !cells.is_empty() {
            let key = &cells[0];
            let val_kv: Option<&str> = record.iter().skip(1).map(str::trim).find(|s| !s.is_empty());
            if SCAL_SAMPLE_ALIASES.iter().any(|a| header_matches(key, a)) {
                // New plug block: reset the per-plug metadata but CARRY OVER idx_pc/idx_sw —
                // hand-merged workbooks often paste the (Speed, Pc, Sw) table header only
                // above the first block; resetting it here would silently drop every
                // later plug's rows.
                sample_no = val_kv.and_then(parse_sample_no);
                depth = None;
                perm = f32::NAN;
                poro = f32::NAN;
                continue;
            }
            if CORE_DEPTH_ALIASES.iter().any(|a| header_matches(key, a)) {
                depth = val_kv.and_then(parse_f32_cell);
                continue;
            }
            if CORE_CPERM_ALIASES.iter().any(|a| header_matches(key, a)) {
                perm = val_kv.and_then(parse_f32_cell).unwrap_or(f32::NAN);
                continue;
            }
            if CORE_CPOR_ALIASES.iter().any(|a| header_matches(key, a)) {
                poro = val_kv.and_then(parse_f32_cell).unwrap_or(f32::NAN);
                continue;
            }
        }

        // Table header row (PC + SW columns present).
        let pc_col = resolve_header_index(&cells, &SCAL_PC_ALIASES);
        let sw_col = resolve_header_index(&cells, &SCAL_SW_ALIASES);
        if let (Some(p), Some(s)) = (pc_col, sw_col) {
            idx_pc = Some(p);
            idx_sw = Some(s);
            continue;
        }

        // Data row under the current block's table header.
        if let (Some(ip), Some(is)) = (idx_pc, idx_sw) {
            let num = |i: usize| -> f32 { record.get(i).and_then(parse_f32_cell).unwrap_or(f32::NAN) };
            let (pc, sw) = (num(ip), num(is));
            if !pc.is_nan() && !sw.is_nan() {
                out.push(ScalPcRecord { sample_no, depth, perm, poro, pc, sw });
            }
        }
    }

    if out.is_empty() {
        return Err(ParseError::Las(
            "centrifuge CSV: no Pc/Sw rows found (expected SAMPLE/DEPTH/PERM/PORO key-value lines then a PC/SW table per plug)".into(),
        ));
    }

    let mut sws: Vec<f32> = out.iter().map(|r| r.sw).collect();
    percent_to_fraction(&mut sws);
    for (r, s) in out.iter_mut().zip(&sws) {
        r.sw = *s;
    }
    let mut poros: Vec<f32> = out.iter().map(|r| r.poro).collect();
    percent_to_fraction(&mut poros);
    for (r, p) in out.iter_mut().zip(&poros) {
        r.poro = *p;
    }
    Ok(out)
}

/// Sniffs which SCAL Pc shape a file is, scanning the first rows for each format's
/// structural signature: a SAMPLE header row with ≥3 numeric pressure columns
/// (porous-plate wide), a header row with PC and SW columns (flat long table), or the
/// centrifuge block signature. A 2-cell `SAMPLE, <id>` line alone is NOT proof of the
/// block format — cover sheets write "No. of Samples,6" / "Sample Type,plug" too — so it
/// only ARMS a candidate; the centrifuge verdict needs corroboration: a following
/// DEPTH/PERM/PORO key-value line with a numeric value, or a PC/SW table header WITHOUT
/// per-row SAMPLE/DEPTH columns.
pub fn sniff_scal_format<P: AsRef<Path>>(path: P) -> ParseResult<&'static str> {
    let delim = scal_delimiter(&path)?;
    let text = read_text_file(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut armed = false; // saw a `SAMPLE, <id>` key-value line
    for (i, result) in rdr.records().enumerate() {
        if i >= 50 {
            break;
        }
        let record = result?;
        let cells: Vec<String> = record.iter().map(|c| c.trim().to_uppercase()).collect();
        if cells.is_empty() || non_empty_cells(&record) == 0 {
            continue;
        }

        // Exactly (key, value) rows — a lone "Sample preparation:" preamble cell must not match.
        if non_empty_cells(&record) == 2 && !cells[0].is_empty() {
            if SCAL_SAMPLE_ALIASES.iter().any(|a| header_matches(&cells[0], a)) {
                armed = true;
                continue;
            }
            // An armed SAMPLE line followed by a numeric DEPTH/PERM/PORO key-value line
            // is the per-plug block header — that combination doesn't occur in cover sheets.
            let is_meta_key = [&CORE_DEPTH_ALIASES[..], &CORE_CPERM_ALIASES[..], &CORE_CPOR_ALIASES[..]]
                .iter()
                .any(|al| al.iter().any(|a| header_matches(&cells[0], a)));
            let val_numeric =
                record.iter().skip(1).map(str::trim).find(|s| !s.is_empty()).and_then(parse_f32_cell);
            if armed && is_meta_key && val_numeric.is_some() {
                return Ok("centrifuge");
            }
        }

        if let Some(s) = resolve_header_index(&cells, &SCAL_SAMPLE_ALIASES) {
            let d = resolve_header_index(&cells, &CORE_DEPTH_ALIASES);
            let k = resolve_header_index(&cells, &CORE_CPERM_ALIASES);
            let p = resolve_header_index(&cells, &CORE_CPOR_ALIASES);
            let meta: Vec<usize> = [Some(s), d, k, p].iter().flatten().copied().collect();
            let n_pressure = cells
                .iter()
                .enumerate()
                .filter(|(i, _)| !meta.contains(i))
                .filter(|(_, h)| parse_pressure_header(h).is_some())
                .count();
            if n_pressure >= 3 {
                return Ok("porous_plate");
            }
        }
        if resolve_header_index(&cells, &SCAL_PC_ALIASES).is_some()
            && resolve_header_index(&cells, &SCAL_SW_ALIASES).is_some()
        {
            // A PC/SW header carrying per-row SAMPLE or DEPTH columns is a flat long
            // table even after a sample-ish cover line; a bare (Speed,)PC,SW header
            // inside an armed block is the centrifuge table.
            let flat = resolve_header_index(&cells, &SCAL_SAMPLE_ALIASES).is_some()
                || resolve_header_index(&cells, &CORE_DEPTH_ALIASES).is_some();
            return Ok(if armed && !flat { "centrifuge" } else { "long" });
        }
    }
    Ok("long")
}

#[cfg(test)]
mod core_csv_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_temp_csv(name: &str, body: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    fn core_csv_aliases_percent_and_missing() {
        // Realistic RCAL delivery: units in headers, porosity/SW in percent, a gap,
        // an extra unrecognized column, and one row without a depth.
        let path = write_temp_csv(
            "arshilla_core_test.csv",
            "Sample,Depth (m),CPOR (%),Kair (mD),Grain_Density,SW (%),Remarks\n\
             1,2001.5,22.5,150.0,2.65,45.0,good plug\n\
             2,2002.0,18.0,,2.66,50.0,no perm\n\
             3,,10.0,5.0,2.64,60.0,lost depth\n\
             4,2003.2,25.0,300.0,2.65,40.0,\n",
        );
        let cols = parse_core_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(cols.depth.len(), 3, "row without depth must be skipped");
        assert!((cols.cpor[0] - 0.225).abs() < 1e-6, "percent porosity must convert to v/v");
        assert!((cols.csw[2] - 0.40).abs() < 1e-6);
        assert!(cols.cperm[1].is_nan(), "empty cell must be NaN");
        assert!((cols.cperm[2] - 300.0).abs() < 1e-3, "perm stays in mD");
        assert!((cols.cgd[0] - 2.65).abs() < 1e-6);
    }

    #[test]
    fn core_csv_fraction_input_left_alone() {
        let path = write_temp_csv(
            "arshilla_core_frac_test.csv",
            "DEPTH,POR,K\n2001.0,0.225,150\n2002.0,0.18,20\n2003.0,0.25,300\n",
        );
        let cols = parse_core_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert!((cols.cpor[0] - 0.225).abs() < 1e-6, "already-fractional porosity must not be rescaled");
    }

    /// The exact header shape of a real core-log delivery
    /// (one-CSV-per-field core delivery): suffixed mnemonics (`CPOR_2`, `GDEN_1`) that resolve
    /// via the `_` boundary rule, and a UNITS row as the first record — skipped because
    /// its depth cell ("FEET") is not numeric, never imported as a phantom plug.
    #[test]
    fn core_csv_delivery_header_resolves() {
        let path = write_temp_csv(
            "sandibumi_core_delivery_test.csv",
            "TAPE_NAME,TOOL_STRING,WN,DEPTH,CPERM_1,CPOR_2,CSO_1,CSW_1,GDEN_1\n\
             \"\",\"\",\"\",FEET,MD,V/V,V/V,V/V,G/C3\n\
             \"\",\"\",SANDI00001,850.5,120.0,0.24,0.15,0.55,2.66\n\
             \"\",\"\",SANDI00001,851.5,85.0,0.22,0.20,0.60,2.65\n",
        );
        let cols = parse_core_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(cols.depth.len(), 2, "units row must be skipped, both plugs kept");
        assert!((cols.cperm[0] - 120.0).abs() < 1e-3, "CPERM_1 resolves");
        assert!((cols.cpor[0] - 0.24).abs() < 1e-6, "CPOR_2 resolves, fraction untouched");
        assert!((cols.csw[1] - 0.60).abs() < 1e-6, "CSW_1 resolves");
        assert!((cols.cgd[0] - 2.66).abs() < 1e-6, "GDEN_1 resolves to grain density");
    }

    #[test]
    fn header_alias_boundaries() {
        assert!(header_matches("CPOR (%)", "CPOR"));
        assert!(header_matches("KAIR MD", "KAIR"));
        assert!(!header_matches("KB", "K"), "'K' must not match 'KB'");
        assert!(!header_matches("POROSITY", "POR"), "'POR' must not swallow other words (POROSITY has its own alias)");
    }
}

#[cfg(test)]
mod scal_import_format_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_temp_csv(name: &str, body: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    /// The Corelab-style porous-plate delivery: preamble lines, OB stress note, pressure
    /// columns as headers, %PV cells, one missing cell, a units row, and a footer.
    const WIDE_BODY: &str = "\
TOTAL INDONESIE,,,\n\
POROUS PLATE CAPILLARY PRESSURE,,,\n\
OVERBURDEN PRESSURE: 4915 PSI,,,\n\
,,,\n\
Sample,Depth (m),Perm (mD),Poro (%),1,2,4,8,10,20\n\
,m,mD,%,psi,psi,psi,psi,psi,psi\n\
4,2001.5,150.0,22.5,98.5,95.2,88.1,79.4,76.0,68.2\n\
7,2010.2,12.0,18.0,99.0,97.5,93.0,,86.5,80.1\n\
AVERAGE,,,,,,,,,\n";

    #[test]
    fn scal_wide_porous_plate_unpivots() {
        let path = write_temp_csv("sandibumi_scal_wide_test.csv", WIDE_BODY);
        let recs = parse_scal_wide_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(recs.len(), 11, "6 + 5 cells (one missing) unpivot to 11 Pc points");
        let s4: Vec<&ScalPcRecord> = recs.iter().filter(|r| r.sample_no == Some(4)).collect();
        assert_eq!(s4.len(), 6);
        assert!((s4[0].pc - 1.0).abs() < 1e-6, "Pc comes from the column header");
        assert!((s4[5].pc - 20.0).abs() < 1e-6);
        assert!((s4[0].sw - 0.985).abs() < 1e-4, "%PV cells convert to v/v");
        assert!((s4[0].perm - 150.0).abs() < 1e-3, "plug perm repeats on every point");
        assert!((s4[0].poro - 0.225).abs() < 1e-4, "percent poro converts to v/v");
        assert_eq!(s4[0].depth, Some(2001.5));
        let s7: Vec<&ScalPcRecord> = recs.iter().filter(|r| r.sample_no == Some(7)).collect();
        assert_eq!(s7.len(), 5, "the empty 8-psi cell yields no point");
        assert!(!s7.iter().any(|r| (r.pc - 8.0).abs() < 1e-6));
    }

    #[test]
    fn scal_wide_without_header_errors() {
        let path = write_temp_csv("sandibumi_scal_wide_bad_test.csv", "just,some,text\n1,2,3\n");
        let err = parse_scal_wide_csv(&path);
        std::fs::remove_file(&path).ok();
        assert!(err.is_err(), "a file with no porous-plate header must error, not import 0 rows");
    }

    /// Two per-plug centrifuge blocks merged into one CSV (the digitized-workbook shape):
    /// key-value plug headers, a speed column the importer must ignore, Sw in %PV.
    const CENTRIFUGE_BODY: &str = "\
SAMPLE,12A\n\
DEPTH (m),2695.3\n\
PERM (mD),45.2\n\
PORO (%),18.3\n\
Speed (RPM),Pc (psi),Sw (%PV)\n\
500,2.1,95.0\n\
1000,8.4,78.2\n\
2000,33.6,55.4\n\
,,\n\
SAMPLE,S-16A\n\
DEPTH,2701.8\n\
PERM,3.4\n\
PORO,12.1\n\
Speed (RPM),Pc (psi),Sw (%PV)\n\
500,2.0,99.0\n\
2000,32.0,71.5\n";

    #[test]
    fn scal_centrifuge_blocks_parse() {
        let path = write_temp_csv("sandibumi_scal_cf_test.csv", CENTRIFUGE_BODY);
        let recs = parse_scal_centrifuge_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(recs.len(), 5);
        let b1: Vec<&ScalPcRecord> = recs.iter().filter(|r| r.sample_no == Some(12)).collect();
        assert_eq!(b1.len(), 3, "lettered plug id '12A' keeps its numeric part");
        assert_eq!(b1[0].depth, Some(2695.3));
        assert!((b1[0].perm - 45.2).abs() < 1e-3);
        assert!((b1[0].poro - 0.183).abs() < 1e-4, "percent poro converts to v/v");
        assert!((b1[1].pc - 8.4).abs() < 1e-3);
        assert!((b1[1].sw - 0.782).abs() < 1e-4, "%PV Sw converts to v/v");
        let b2: Vec<&ScalPcRecord> = recs.iter().filter(|r| r.sample_no == Some(16)).collect();
        assert_eq!(b2.len(), 2, "'S-16A' → 16; second block's metadata does not leak from the first");
        assert_eq!(b2[0].depth, Some(2701.8));
        assert!((b2[0].perm - 3.4).abs() < 1e-3);
    }

    #[test]
    fn scal_centrifuge_without_table_errors() {
        let path = write_temp_csv("sandibumi_scal_cf_bad_test.csv", "SAMPLE,12A\nDEPTH,2695.3\n");
        let err = parse_scal_centrifuge_csv(&path);
        std::fs::remove_file(&path).ok();
        assert!(err.is_err(), "a block with no Pc/Sw table must error");
    }

    #[test]
    fn scal_sniff_detects_all_three_formats() {
        let wide = write_temp_csv("sandibumi_scal_sniff_wide.csv", WIDE_BODY);
        let cf = write_temp_csv("sandibumi_scal_sniff_cf.csv", CENTRIFUGE_BODY);
        let long = write_temp_csv(
            "sandibumi_scal_sniff_long.csv",
            "SAMPLE,DEPTH,PERM,PORO,PC,SW\n1,2000.5,150,22,5,0.55\n",
        );
        let w = sniff_scal_format(&wide).unwrap();
        let c = sniff_scal_format(&cf).unwrap();
        let l = sniff_scal_format(&long).unwrap();
        for p in [&wide, &cf, &long] {
            std::fs::remove_file(p).ok();
        }
        assert_eq!(w, "porous_plate");
        assert_eq!(c, "centrifuge");
        assert_eq!(l, "long");
    }

    #[test]
    fn scal_sample_no_letter_suffixes() {
        assert_eq!(parse_sample_no("4"), Some(4));
        assert_eq!(parse_sample_no("12A"), Some(12));
        assert_eq!(parse_sample_no("S-16A"), Some(16));
        assert_eq!(parse_sample_no("plug"), None);
    }

    // ---- post-review hardening (ultracode review 2026-07-22) ----

    /// Repeated per-page header rows (whose pressure headers ARE numbers) and numeric
    /// "Average" footer rows must not import as phantom plugs.
    #[test]
    fn scal_wide_skips_repeated_headers_and_numeric_footers() {
        let body = "\
Sample,Depth (m),Perm (mD),Poro (%),1,2,4,8,10,20\n\
4,2001.5,150.0,22.5,98.5,95.2,88.1,79.4,76.0,68.2\n\
Sample,Depth (m),Perm (mD),Poro (%),1,2,4,8,10,20\n\
7,2010.2,12.0,18.0,99.0,97.5,93.0,,86.5,80.1\n\
Average,,132.5,20.3,96.2,94.1,88.0,79.0,76.0,68.0\n";
        let path = write_temp_csv("sandibumi_scal_wide_phantom_test.csv", body);
        let recs = parse_scal_wide_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(recs.len(), 11, "only the two real plugs import (6 + 5 points)");
        assert!(recs.iter().all(|r| r.sample_no.is_some()), "no phantom sample-less rows");
    }

    /// Hand-merged centrifuge workbooks often paste the table header only above the first
    /// block — later blocks must reuse it, not silently vanish.
    #[test]
    fn scal_centrifuge_header_carries_over_blocks() {
        let body = "\
SAMPLE,12A\nDEPTH,2695.3\nPERM,45.2\nPORO,18.3\n\
Speed (RPM),Pc (psi),Sw (%PV)\n500,2.1,95.0\n1000,8.4,78.2\n\
SAMPLE,S-16A\nDEPTH,2701.8\nPERM,3.4\nPORO,12.1\n\
500,2.0,99.0\n2000,32.0,71.5\n";
        let path = write_temp_csv("sandibumi_scal_cf_carry_test.csv", body);
        let recs = parse_scal_centrifuge_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(recs.len(), 4, "block 2 rows parse via the carried-over header");
        let b2: Vec<&ScalPcRecord> = recs.iter().filter(|r| r.sample_no == Some(16)).collect();
        assert_eq!(b2.len(), 2);
        assert_eq!(b2[0].depth, Some(2701.8), "block 2 keeps its own metadata");
        assert!((b2[1].pc - 32.0).abs() < 1e-3);
    }

    /// Cover-sheet key-value lines ("No. of Samples,6", "Sample Type,plug") must not trip
    /// the centrifuge signature; the real header decides. A bare PC/SW table under a
    /// SAMPLE line still sniffs centrifuge.
    #[test]
    fn scal_sniff_ignores_cover_sheet_kv_lines() {
        let wide = format!("No. of Samples,2\nSample Type,Horizontal plug\n{WIDE_BODY}");
        let long = "Sample Type,plug\nSAMPLE,DEPTH,PERM,PORO,PC,SW\n1,2000.5,150,22,5,0.55\n";
        let cf_min = "SAMPLE,12A\nPc (psi),Sw (%PV)\n2.1,95.0\n8.4,78.2\n";
        let pw = write_temp_csv("sandibumi_scal_sniff_cover_wide.csv", &wide);
        let pl = write_temp_csv("sandibumi_scal_sniff_cover_long.csv", long);
        let pc = write_temp_csv("sandibumi_scal_sniff_min_cf.csv", cf_min);
        let w = sniff_scal_format(&pw).unwrap();
        let l = sniff_scal_format(&pl).unwrap();
        let c = sniff_scal_format(&pc).unwrap();
        for p in [&pw, &pl, &pc] {
            std::fs::remove_file(p).ok();
        }
        assert_eq!(w, "porous_plate", "cover KV lines must not hijack a wide file");
        assert_eq!(l, "long", "a PC/SW header with per-row SAMPLE columns is flat");
        assert_eq!(c, "centrifuge", "SAMPLE line + bare PC/SW header is a block file");
    }

    /// Indonesian/European Excel exports: ';' list separator with ',' decimals.
    #[test]
    fn scal_semicolon_and_decimal_comma_parse() {
        let body = "\
SAMPLE;12A\nDEPTH;2695,3\nPERM;45,2\nPORO;18,3\n\
Speed (RPM);Pc (psi);Sw (%PV)\n500;2,1;95,0\n1000;8,4;78,2\n2000;33,6;55,4\n";
        let path = write_temp_csv("sandibumi_scal_semicolon_test.csv", body);
        assert_eq!(sniff_scal_format(&path).unwrap(), "centrifuge");
        let recs = parse_scal_centrifuge_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].depth, Some(2695.3), "decimal-comma depth parses");
        assert!((recs[0].perm - 45.2).abs() < 1e-3);
        assert!((recs[1].pc - 8.4).abs() < 1e-3);
        assert!((recs[1].sw - 0.782).abs() < 1e-4, "%PV with decimal comma converts");
    }

    #[test]
    fn scal_numeric_cell_regional_formats() {
        assert_eq!(parse_f32_cell("98.5"), Some(98.5));
        assert_eq!(parse_f32_cell("2,695.3"), Some(2695.3), "thousands + decimal point");
        assert_eq!(parse_f32_cell("1,250"), Some(1250.0), "thousands group");
        assert_eq!(parse_f32_cell("1,000"), Some(1000.0), "3-digit group reads as thousands");
        assert_eq!(parse_f32_cell("98,5"), Some(98.5), "lone decimal comma");
        assert_eq!(parse_f32_cell("1,0"), Some(1.0), "short group reads as decimal comma");
        assert_eq!(parse_f32_cell("Remarks"), None);
        assert_eq!(parse_f32_cell(""), None);
    }

    /// The flat/long parser keeps lettered plug ids the same way the other formats do.
    #[test]
    fn scal_long_lettered_sample_ids() {
        let path = write_temp_csv("sandibumi_scal_long_letter_test.csv", "SAMPLE,PC,SW\n12A,5,0.55\n12A,10,0.45\n");
        let recs = parse_scal_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].sample_no, Some(12), "'12A' keeps its numeric part in the long parser too");
    }

    /// Merged-cell long exports (plug context only on each plug's first row) forward-fill,
    /// so continuation rows stay with their plug and keep its perm/poro.
    #[test]
    fn scal_long_forward_fills_merged_cells() {
        let body = "\
SAMPLE,DEPTH,PERM,PORO,PC,SW\n\
1,2000.5,150,0.22,1,0.95\n\
,,,,5,0.70\n\
,,,,20,0.45\n\
2,2010.0,12,0.18,1,0.98\n\
,,,,20,0.60\n";
        let path = write_temp_csv("sandibumi_scal_long_ffill_test.csv", body);
        let recs = parse_scal_csv(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(recs.len(), 5);
        assert!(recs[..3].iter().all(|r| r.sample_no == Some(1) && (r.poro - 0.22).abs() < 1e-5));
        assert_eq!(recs[1].depth, Some(2000.5), "continuation rows inherit the plug depth");
        assert!(recs[3..].iter().all(|r| r.sample_no == Some(2) && (r.perm - 12.0).abs() < 1e-3));
        assert!((recs[4].poro - 0.18).abs() < 1e-5, "plug 2 keeps its own context, no bleed from plug 1");
    }
}

/// A deviation survey as parsed from CSV: measured depth, inclination (deg), azimuth (deg).
#[derive(Debug, Clone, Default)]
pub struct DeviationSurvey {
    pub md: Vec<f32>,
    pub inc: Vec<f32>,
    pub azi: Vec<f32>,
}

const DEV_MD_ALIASES: [&str; 4] = ["MD", "DEPTH", "DEPT", "MEASURED_DEPTH"];
const DEV_INC_ALIASES: [&str; 4] = ["INC", "INCL", "INCLINATION", "DEVI"];
const DEV_AZI_ALIASES: [&str; 5] = ["AZI", "AZIM", "AZIMUTH", "HAZI", "AZM"];

/// Parses a deviation-survey CSV (MD/INC/AZI columns, alias-tolerant, arbitrary order).
/// Rows sort by MD ascending; a missing INC/AZI is treated as 0 (vertical/north).
pub fn parse_deviation_csv<P: AsRef<Path>>(path: P) -> ParseResult<DeviationSurvey> {
    let text = read_text_file(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(text.as_bytes());
    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.trim().to_uppercase()).collect();
    let idx_md = resolve_header_index(&headers, &DEV_MD_ALIASES)
        .ok_or_else(|| ParseError::Las("deviation CSV has no recognizable MD column".into()))?;
    let idx_inc = resolve_header_index(&headers, &DEV_INC_ALIASES);
    let idx_azi = resolve_header_index(&headers, &DEV_AZI_ALIASES);

    let mut rows: Vec<(f32, f32, f32)> = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let get = |idx: Option<usize>| -> f32 {
            idx.and_then(|i| record.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(f32::NAN)
        };
        let md = get(Some(idx_md));
        if md.is_nan() {
            continue;
        }
        let inc = get(idx_inc);
        let azi = get(idx_azi);
        rows.push((md, if inc.is_nan() { 0.0 } else { inc }, if azi.is_nan() { 0.0 } else { azi }));
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut survey = DeviationSurvey::default();
    for (md, inc, azi) in rows {
        survey.md.push(md);
        survey.inc.push(inc);
        survey.azi.push(azi);
    }
    // Dedup duplicate station MDs (first kept) so a repeated MD can't abort the whole survey on
    // the well_path (well_id, md) PK. MD is already sorted; a duplicated MD carries no new geometry.
    let (keep, report) = depth_keep_indices(&survey.md);
    if !report.is_clean() {
        let take = |src: &[f32]| -> Vec<f32> { keep.iter().map(|&i| src[i]).collect() };
        survey.md = take(&survey.md);
        survey.inc = take(&survey.inc);
        survey.azi = take(&survey.azi);
    }
    Ok(survey)
}

/// Parses every LAS file in `dir` concurrently across all CPU threads via `rayon`.
/// Returns a `(path, result)` pair per file so individual parse failures don't abort the batch.
#[allow(dead_code)] // batch/folder LAS import, wired into the ribbon in a later increment
pub fn parse_las_directory<P: AsRef<Path>>(dir: P) -> ParseResult<Vec<(String, ParseResult<CurveColumns>)>> {
    let paths: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("las")))
        .collect();

    let results: Vec<(String, ParseResult<CurveColumns>)> = paths
        .par_iter()
        .map(|path| {
            let name = path.display().to_string();
            (name, parse_las_2(path))
        })
        .collect();

    Ok(results)
}

// ---------------------------------------------------------------------------
// P2 tops-style imports: formation tops (CSV/TXT) + generic point/interval
// datasets (petrography, XRD, perforations)
// ---------------------------------------------------------------------------

/// One formation-top row from a tops file. `well` is None when the file has no
/// well column (single-well export) — the importer falls back to the selected well.
#[derive(Debug, Clone)]
pub struct TopsRecord {
    pub well: Option<String>,
    pub top_name: String,
    pub depth: f32,
}

// WN: a real core-log delivery's well-name column.
const TOPS_WELL_ALIASES: [&str; 8] =
    ["WELL", "WELLNAME", "WELL_NAME", "WELLBORE", "BOREHOLE", "UWI", "WELL_ID", "WN"];
const TOPS_NAME_ALIASES: [&str; 9] =
    ["TOP", "TOP_NAME", "TOPS", "MARKER", "SURFACE", "FORMATION", "HORIZON", "ZONE", "NAME"];
const TOPS_DEPTH_ALIASES: [&str; 7] =
    ["DEPTH", "MD", "TOP_MD", "MD_TOP", "TOP_DEPTH", "DEPT", "TVD"];

/// Reads a delimited text file into (headers, rows), auto-detecting the delimiter from
/// the first non-comment line: tab, then semicolon, then comma, else runs of whitespace.
/// Quoted fields are honoured for the csv-crate delimiters; whitespace mode is a plain
/// split (well names with spaces need a tab or comma file). Lines starting with '#' skip.
fn read_delimited<P: AsRef<Path>>(path: P) -> ParseResult<(Vec<String>, Vec<Vec<String>>)> {
    let text = read_text_file(path)?;
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    let Some(first) = lines.first() else {
        return Ok((Vec::new(), Vec::new()));
    };

    let delim: Option<u8> = if first.contains('\t') {
        Some(b'\t')
    } else if first.contains(';') {
        Some(b';')
    } else if first.contains(',') {
        Some(b',')
    } else {
        None // whitespace mode
    };

    let mut table: Vec<Vec<String>> = Vec::with_capacity(lines.len());
    match delim {
        Some(d) => {
            let joined = lines.join("\n");
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(d)
                .has_headers(false)
                .flexible(true)
                .from_reader(joined.as_bytes());
            for rec in rdr.records() {
                let rec = rec?;
                table.push(rec.iter().map(|s| s.trim().to_string()).collect());
            }
        }
        None => {
            for line in &lines {
                table.push(line.split_whitespace().map(str::to_string).collect());
            }
        }
    }
    if table.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let headers: Vec<String> = table.remove(0).iter().map(|h| h.trim().to_uppercase()).collect();
    Ok((headers, table))
}

/// Parses a formation-tops file (CSV or TXT — Petrel-style exports). Needs a
/// recognizable top-name and depth column; a well column makes it multi-well. Headerless
/// two-column "NAME DEPTH" (or three-column "WELL NAME DEPTH") files are also accepted:
/// if no known headers are found and the last column of the first line parses as a
/// number, the first line is treated as data.
/// Returns `(has_well_column, records)` — the flag lets the importer tell a genuinely
/// column-less single-well file (fall back to the selected well) from a multi-well file with a
/// blank WELL cell (skip the row instead of misrouting it), both of which yield `record.well ==
/// None`. Mirrors `parse_locations_file`.
pub fn parse_tops_file<P: AsRef<Path>>(path: P) -> ParseResult<(bool, Vec<TopsRecord>)> {
    let (headers, mut rows) = read_delimited(path)?;
    if headers.is_empty() {
        return Err(ParseError::Las("tops file is empty".into()));
    }

    // Depth resolves FIRST and is excluded from the name search — otherwise a header
    // like "TOP_MD" would satisfy the name alias "TOP" (boundary rule allows '_').
    let idx_depth = resolve_header_index(&headers, &TOPS_DEPTH_ALIASES);
    let idx_name = TOPS_NAME_ALIASES.iter().find_map(|alias| {
        headers
            .iter()
            .enumerate()
            .find(|(i, h)| Some(*i) != idx_depth && header_matches(h, alias))
            .map(|(i, _)| i)
    });
    let (idx_well, idx_name, idx_depth) = match (idx_name, idx_depth) {
        (Some(n), Some(d)) => (resolve_header_index(&headers, &TOPS_WELL_ALIASES), n, d),
        _ => {
            // Headerless fallback: NAME DEPTH or WELL NAME DEPTH, first line is data.
            let last_is_num =
                headers.last().is_some_and(|h| h.replace(',', ".").parse::<f32>().is_ok());
            if !last_is_num || headers.len() < 2 {
                return Err(ParseError::Las(
                    "tops file needs TOP/MARKER/FORMATION and DEPTH/MD columns (or headerless NAME DEPTH lines)"
                        .into(),
                ));
            }
            rows.insert(0, headers.clone());
            match headers.len() {
                2 => (None, 0usize, 1usize),
                _ => (Some(0usize), 1usize, 2usize),
            }
        }
    };

    let mut out = Vec::new();
    for row in rows {
        let depth = row
            .get(idx_depth)
            .map(|s| s.replace(',', "."))
            .and_then(|s| s.parse::<f32>().ok())
            // A literal `NaN` cell is exactly what pandas (`na_rep='NaN'`) and numpy write for a
            // missing marker, and it parses cleanly to f32::NAN. A non-finite top depth cannot be
            // ordered, so drop the row here rather than store a top that panics a later sort.
            .filter(|d| d.is_finite());
        let name = row.get(idx_name).map(|s| s.trim()).filter(|s| !s.is_empty());
        let (Some(depth), Some(name)) = (depth, name) else { continue };
        let well = idx_well
            .and_then(|i| row.get(i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        out.push(TopsRecord { well, top_name: name.to_string(), depth });
    }
    if out.is_empty() {
        return Err(ParseError::Las("tops file has no parsable rows".into()));
    }
    Ok((idx_well.is_some(), out))
}

/// One well-surface-location row. `well` is None when this row carries no well name —
/// either the file has no WELL column at all (single-well export), or it has one but this
/// row's cell is blank/ragged. `parse_locations_file` returns a separate `has_well_column`
/// flag so the importer can tell those apart (fall back to the selected well only for a
/// genuinely column-less file; skip a blank cell in a multi-well file). `zone` is None when
/// the file has no zone column, letting the importer apply a dialog-picked default.
#[derive(Debug, Clone)]
pub struct LocationRecord {
    pub well: Option<String>,
    pub x: f64,
    pub y: f64,
    pub zone: Option<String>,
}

// Easting/northing aliases, most-specific first so a bare "X"/"Y" never wins over an
// explicit column. `read_delimited` upper-cases headers, so these compare case-blind.
const LOC_X_ALIASES: [&str; 8] = ["EASTING", "UTM_X", "SURFACE_X", "X_COORD", "XCOORD", "UTMX", "EAST", "X"];
const LOC_Y_ALIASES: [&str; 8] = ["NORTHING", "UTM_Y", "SURFACE_Y", "Y_COORD", "YCOORD", "UTMY", "NORTH", "Y"];
// Bare "UTM" is deliberately NOT a zone alias — it would swallow "UTM_X" via the boundary
// rule; the zone search also excludes the resolved X/Y columns as a second guard.
const LOC_ZONE_ALIASES: [&str; 4] = ["UTM_ZONE", "UTMZONE", "GRID_ZONE", "ZONE"];

/// Parses a well-surface-location file (CSV/TXT). Needs recognizable easting and northing
/// columns; a WELL column makes it multi-well; an optional ZONE column carries a per-well
/// UTM zone (the importer supplies a default for rows/files without one). Rows whose X or
/// Y is missing or non-numeric are skipped. Returns `(has_well_column, records)` — the flag
/// lets the importer distinguish a headerless single-well file from a multi-well file with a
/// blank WELL cell (both yield `record.well == None`).
pub fn parse_locations_file<P: AsRef<Path>>(path: P) -> ParseResult<(bool, Vec<LocationRecord>)> {
    let (headers, rows) = read_delimited(path)?;
    if headers.is_empty() {
        return Err(ParseError::Las("locations file is empty".into()));
    }
    let idx_x = resolve_header_index(&headers, &LOC_X_ALIASES)
        .ok_or_else(|| ParseError::Las("file has no X / EASTING column".into()))?;
    let idx_y = resolve_header_index(&headers, &LOC_Y_ALIASES)
        .ok_or_else(|| ParseError::Las("file has no Y / NORTHING column".into()))?;
    let idx_well = resolve_header_index(&headers, &TOPS_WELL_ALIASES);
    // Zone must never resolve to the easting/northing columns already claimed above.
    let idx_zone = LOC_ZONE_ALIASES.iter().find_map(|alias| {
        headers
            .iter()
            .enumerate()
            .find(|(i, h)| *i != idx_x && *i != idx_y && header_matches(h, alias))
            .map(|(i, _)| i)
    });

    let mut out = Vec::new();
    for row in &rows {
        let x = row.get(idx_x).map(|s| s.replace(',', ".")).and_then(|s| s.parse::<f64>().ok());
        let y = row.get(idx_y).map(|s| s.replace(',', ".")).and_then(|s| s.parse::<f64>().ok());
        let (Some(x), Some(y)) = (x, y) else { continue };
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        let well = idx_well.and_then(|i| row.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let zone = idx_zone.and_then(|i| row.get(i)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        out.push(LocationRecord { well, x, y, zone });
    }
    if out.is_empty() {
        return Err(ParseError::Las("locations file has no parsable rows".into()));
    }
    Ok((idx_well.is_some(), out))
}

/// A generic point/interval dataset (petrography, XRD, perforations): TOP (+optional
/// BASE) depth per row, every other column an item whose values may be numeric or text.
#[derive(Debug, Clone, Default)]
pub struct IntervalData {
    /// Value column headers, upper-cased, file order.
    pub items: Vec<String>,
    /// (top, base, raw values parallel to `items`; None = empty cell).
    pub rows: Vec<(f32, Option<f32>, Vec<Option<String>>)>,
    /// Parallel to `rows`: the WELL cell, when the file has a well column and the cell is
    /// non-blank (T-IMP-11 — multi-well aux files route rows by name, like tops).
    pub wells: Vec<Option<String>>,
    pub has_well_column: bool,
}

const AUX_TOP_ALIASES: [&str; 7] = ["TOP", "DEPTH", "TOP_MD", "FROM", "TOP_DEPTH", "MD", "DEPT"];
const AUX_BASE_ALIASES: [&str; 6] = ["BASE", "BOTTOM", "TO", "BASE_MD", "BOT", "BOTTOM_DEPTH"];

/// Parses a tops-style dataset file (CSV/TXT, same delimiter detection as tops): needs a
/// TOP/DEPTH column; BASE/BOTTOM makes rows intervals; a WELL column is captured per
/// row so the importer can route multi-well files by name (T-IMP-11) — it never becomes
/// an item. All remaining columns become items.
pub fn parse_interval_file<P: AsRef<Path>>(path: P) -> ParseResult<IntervalData> {
    let (headers, rows) = read_delimited(path)?;
    if headers.is_empty() {
        return Err(ParseError::Las("data file is empty".into()));
    }
    let idx_top = resolve_header_index(&headers, &AUX_TOP_ALIASES)
        .ok_or_else(|| ParseError::Las("file has no recognizable TOP/DEPTH column".into()))?;
    let idx_base = resolve_header_index(&headers, &AUX_BASE_ALIASES);
    let idx_well = resolve_header_index(&headers, &TOPS_WELL_ALIASES);

    let skip: Vec<usize> =
        [Some(idx_top), idx_base, idx_well].iter().flatten().copied().collect();
    let item_idx: Vec<usize> = (0..headers.len()).filter(|i| !skip.contains(i)).collect();
    let items: Vec<String> = item_idx.iter().map(|&i| headers[i].clone()).collect();
    if items.is_empty() {
        return Err(ParseError::Las("file has no value columns besides depth".into()));
    }

    let mut out = IntervalData {
        items,
        rows: Vec::new(),
        wells: Vec::new(),
        has_well_column: idx_well.is_some(),
    };
    for row in rows {
        let top = row
            .get(idx_top)
            .map(|s| s.replace(',', "."))
            .and_then(|s| s.parse::<f32>().ok());
        let Some(top) = top else { continue };
        let base = idx_base
            .and_then(|i| row.get(i))
            .map(|s| s.replace(',', "."))
            .and_then(|s| s.parse::<f32>().ok());
        let values: Vec<Option<String>> = item_idx
            .iter()
            .map(|&i| row.get(i).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
            .collect();
        out.wells.push(
            idx_well
                .and_then(|i| row.get(i))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
        out.rows.push((top, base, values));
    }
    if out.rows.is_empty() {
        return Err(ParseError::Las("data file has no parsable rows".into()));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Core import v2 (T-IMP-07): probe → confirm → commit.
//
// A real core delivery is a wide lab export — well name inside the data (WN /
// WELL NAME), a units row under the headers, percent porosities, feet depths —
// and the old single-well, comma-only path imported it half-blind. The wizard
// flow: `probe_core_table` reads the file once and reports everything the
// dialog needs to CONFIRM (headers, guessed roles, column types, sample rows,
// distinct wells, percent + depth-unit detection); the user adjusts;
// `parse_core_table_mapped` then extracts rows under the CONFIRMED mapping.
// ---------------------------------------------------------------------------

/// Confirmed column mapping for a core table: indices into the file's columns.
/// Serialized both ways — the probe suggests one, the dialog returns one.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct CoreMapping {
    pub well: Option<usize>,
    pub depth: usize,
    pub cpor: Option<usize>,
    pub cperm: Option<usize>,
    pub cgd: Option<usize>,
    pub csw: Option<usize>,
    /// Columns to carry as EXTRA point data (lithology text, So, Kv/Kh, sample ids …).
    /// `core_data` has a fixed four-measurement schema; a real lab export is wider than
    /// that, so the leftovers land in the open-schema `aux_data` store at the same plug
    /// depths — numeric cells as numbers, everything else as text.
    #[serde(default)]
    pub extras: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WellRowCount {
    pub name: String,
    pub rows: usize,
}

/// Everything the import dialog shows before anything is written.
#[derive(Debug, Clone, Serialize)]
pub struct TableProbe {
    /// Upper-cased headers, file order.
    pub headers: Vec<String>,
    /// Data rows (units row, when detected, excluded).
    pub n_rows: usize,
    /// Guessed role → column index; None where no alias resolved. `depth` is a guess
    /// here (Option) — the confirmed `CoreMapping` requires it.
    pub well: Option<usize>,
    pub depth: Option<usize>,
    pub cpor: Option<usize>,
    pub cperm: Option<usize>,
    pub cgd: Option<usize>,
    pub csw: Option<usize>,
    /// "number" | "text" | "empty" per column, sniffed from up to 200 data rows.
    pub column_kind: Vec<String>,
    /// Up to 5 raw data rows for the dialog's preview grid.
    pub sample_rows: Vec<Vec<String>>,
    /// Distinct well-cell values with row counts (capped at 100), when a well column
    /// was guessed. The dialog shows these so routing is confirmed, not assumed.
    pub wells: Vec<WellRowCount>,
    /// Roles ("CPOR"/"CSW") whose values read as percent (median > 1.5) — the import
    /// will divide them to v/v, and the dialog says so out loud.
    pub percent_roles: Vec<String>,
    /// "ft" / "m" when the units row or the depth header names one, else None.
    pub depth_unit_guess: Option<String>,
    /// True when the first data row was a units row (non-numeric depth cell) — skipped.
    pub units_row_skipped: bool,
}

/// Splits `s` into alphanumeric tokens and looks for a depth-unit word.
fn unit_token_guess(s: &str) -> Option<&'static str> {
    for tok in s.split(|c: char| !c.is_ascii_alphanumeric()) {
        match tok.to_ascii_uppercase().as_str() {
            "FT" | "FEET" | "FOOT" => return Some("ft"),
            "M" | "METRE" | "METRES" | "METER" | "METERS" => return Some("m"),
            _ => {}
        }
    }
    None
}

/// True when the first data row is a UNITS row (delivery-style: `,,,FEET,MD,V/V,…`):
/// the depth cell exists but does not parse as a number.
fn is_units_row(row: &[String], depth_col: usize) -> bool {
    row.get(depth_col)
        .map(|c| c.trim())
        .is_some_and(|c| !c.is_empty() && c.replace(',', ".").parse::<f32>().is_err())
}

/// Reads a core table (CSV/TXT, delimiter auto-detected) and reports everything the
/// import dialog needs to confirm the mapping. Writes nothing.
pub fn probe_core_table<P: AsRef<Path>>(path: P) -> ParseResult<TableProbe> {
    let (headers, mut rows) = read_delimited(path)?;
    if headers.is_empty() {
        return Err(ParseError::Las("file is empty".into()));
    }

    let depth = resolve_header_index(&headers, &CORE_DEPTH_ALIASES);
    // Well column: several headers can satisfy the aliases (some exports carry both a
    // numeric WELL and a textual WELL NAME). Prefer the first candidate whose values are
    // mostly NON-numeric — a well NAME routes rows; a bare pad number usually doesn't.
    let well_candidates: Vec<usize> = (0..headers.len())
        .filter(|&i| TOPS_WELL_ALIASES.iter().any(|a| header_matches(&headers[i], a)))
        .collect();
    let mostly_text = |col: usize| -> bool {
        let mut num = 0usize;
        let mut txt = 0usize;
        for row in rows.iter().take(200) {
            let Some(cell) = row.get(col).map(|c| c.trim()).filter(|c| !c.is_empty()) else { continue };
            if cell.replace(',', ".").parse::<f32>().is_ok() { num += 1 } else { txt += 1 }
        }
        txt > num
    };
    let well = well_candidates
        .iter()
        .copied()
        .find(|&c| mostly_text(c))
        .or(well_candidates.first().copied());

    let units_row = depth.is_some_and(|d| rows.first().is_some_and(|r| is_units_row(r, d)));
    let units_cells = if units_row { Some(rows.remove(0)) } else { None };

    // Depth unit: the units row's depth cell first, else the depth header itself.
    let depth_unit_guess = depth.and_then(|d| {
        units_cells
            .as_ref()
            .and_then(|u| u.get(d))
            .and_then(|c| unit_token_guess(c))
            .or_else(|| unit_token_guess(&headers[d]))
            .map(str::to_string)
    });

    // Column kinds from up to 200 data rows.
    let column_kind: Vec<String> = (0..headers.len())
        .map(|col| {
            let mut num = 0usize;
            let mut txt = 0usize;
            for row in rows.iter().take(200) {
                let Some(cell) = row.get(col).map(|c| c.trim()).filter(|c| !c.is_empty()) else { continue };
                if cell.replace(',', ".").parse::<f32>().is_ok() { num += 1 } else { txt += 1 }
            }
            if num == 0 && txt == 0 { "empty" } else if num >= txt { "number" } else { "text" }.to_string()
        })
        .collect();

    // Distinct wells (row counts), file order, capped for the dialog.
    let mut wells: Vec<WellRowCount> = Vec::new();
    if let Some(w) = well {
        for row in &rows {
            let Some(name) = row.get(w).map(|c| c.trim()).filter(|c| !c.is_empty()) else { continue };
            if let Some(e) = wells.iter_mut().find(|e| e.name == name) {
                e.rows += 1;
            } else if wells.len() < 100 {
                wells.push(WellRowCount { name: name.to_string(), rows: 1 });
            }
        }
    }

    // Percent detection on the roles percent_to_fraction would touch.
    let median_of = |col: Option<usize>| -> Option<f32> {
        let mut vals: Vec<f32> = rows
            .iter()
            .filter_map(|r| col.and_then(|c| r.get(c)).and_then(|c| c.trim().replace(',', ".").parse::<f32>().ok()))
            .filter(|v| v.is_finite())
            .collect();
        if vals.is_empty() { return None }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Some(vals[vals.len() / 2])
    };
    let cpor = resolve_header_index(&headers, &CORE_CPOR_ALIASES);
    let csw = resolve_header_index(&headers, &CORE_CSW_ALIASES);
    let mut percent_roles = Vec::new();
    if median_of(cpor).is_some_and(|m| m > 1.5) {
        percent_roles.push("CPOR".to_string());
    }
    if median_of(csw).is_some_and(|m| m > 1.5) {
        percent_roles.push("CSW".to_string());
    }

    Ok(TableProbe {
        n_rows: rows.len(),
        sample_rows: rows.iter().take(5).cloned().collect(),
        well,
        depth,
        cpor,
        cperm: resolve_header_index(&headers, &CORE_CPERM_ALIASES),
        cgd: resolve_header_index(&headers, &CORE_CGD_ALIASES),
        csw,
        column_kind,
        wells,
        percent_roles,
        depth_unit_guess,
        units_row_skipped: units_row,
        headers,
    })
}

/// One core-table row under a confirmed mapping. `well` is the raw cell (None when the
/// mapping has no well column or the cell is blank) — the importer routes/reports it.
#[derive(Debug, Clone)]
pub struct MappedCoreRow {
    pub well: Option<String>,
    pub depth: f32,
    pub cpor: f32,
    pub cperm: f32,
    pub cgd: f32,
    pub csw: f32,
    /// Raw cells of `mapping.extras`, in that order — blank cells are `None`. Kept as
    /// TEXT on purpose: typing happens per cell at the write, so one column may hold
    /// numbers on some plugs and a remark ("below detection") on others.
    pub extras: Vec<Option<String>>,
}

/// One core table read under a confirmed mapping: the rows plus the header names of the
/// extra columns (they become the `item` of each aux row, so they travel with the data).
#[derive(Debug, Clone)]
pub struct MappedCoreTable {
    pub rows: Vec<MappedCoreRow>,
    pub extra_names: Vec<String>,
}

/// Extracts core rows under the dialog-confirmed `mapping`. The units row (when present)
/// is skipped by the same rule the probe used; rows whose depth cell doesn't parse are
/// dropped; CPOR/CSW get the file-wide percent→fraction conversion. Extra columns come
/// back as raw text (typed per cell at the write). Depth-unit conversion is NOT done here
/// — the importer owns it (it knows the project unit).
pub fn parse_core_table_mapped<P: AsRef<Path>>(
    path: P,
    mapping: &CoreMapping,
) -> ParseResult<MappedCoreTable> {
    let (headers, mut rows) = read_delimited(path)?;
    if mapping.depth >= headers.len() {
        return Err(ParseError::Las(format!(
            "depth column {} is out of range for this file ({} columns)",
            mapping.depth,
            headers.len()
        )));
    }
    if rows.first().is_some_and(|r| is_units_row(r, mapping.depth)) {
        rows.remove(0);
    }

    let cell = |row: &Vec<String>, col: Option<usize>| -> f32 {
        col.and_then(|c| row.get(c))
            .map(|c| c.trim().replace(',', "."))
            .filter(|c| !c.is_empty())
            .and_then(|c| c.parse::<f32>().ok())
            .unwrap_or(f32::NAN)
    };
    // Extra columns out of range for THIS file are dropped (multi-file imports confirm the
    // mapping by header name, so a file that simply lacks a column must not abort).
    let extras: Vec<usize> = mapping.extras.iter().copied().filter(|&c| c < headers.len()).collect();
    let extra_names: Vec<String> = extras.iter().map(|&c| headers[c].clone()).collect();

    let mut out: Vec<MappedCoreRow> = Vec::new();
    for row in &rows {
        let depth = cell(row, Some(mapping.depth));
        if !depth.is_finite() {
            continue;
        }
        let well = mapping
            .well
            .and_then(|c| row.get(c))
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());
        out.push(MappedCoreRow {
            well,
            depth,
            cpor: cell(row, mapping.cpor),
            cperm: cell(row, mapping.cperm),
            cgd: cell(row, mapping.cgd),
            csw: cell(row, mapping.csw),
            extras: extras
                .iter()
                .map(|&c| row.get(c).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
                .collect(),
        });
    }
    // File-wide percent→fraction on porosity and saturation (same heuristic and scope as
    // the legacy parser: one decision per file, never per well, so a well whose few plugs
    // all sit under 1.0 can't dodge a conversion the rest of the file clearly needs).
    let mut cpor: Vec<f32> = out.iter().map(|r| r.cpor).collect();
    let mut csw: Vec<f32> = out.iter().map(|r| r.csw).collect();
    percent_to_fraction(&mut cpor);
    percent_to_fraction(&mut csw);
    for (r, (p, s)) in out.iter_mut().zip(cpor.into_iter().zip(csw)) {
        r.cpor = p;
        r.csw = s;
    }
    Ok(MappedCoreTable { rows: out, extra_names })
}

#[cfg(test)]
mod encoding_tests {
    use super::*;
    use std::io::Write;

    fn write_bytes(name: &str, body: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(body).unwrap();
        path
    }

    /// The encoding regression, byte-for-byte. A 330 KB core table that was pure ASCII except for
    /// TWO 0x95 bullets opening a lithology description was refused outright with
    /// "io error: stream did not contain valid UTF-8" — 20,000 good plugs lost to two
    /// characters in a comment field. cp1252 0x95 is "•", and the import must now simply read.
    #[test]
    fn cp1252_bullet_in_a_description_does_not_fail_the_import() {
        let mut body: Vec<u8> = b"WELL,DEPTH,CPOR,CPERM,LITH\n".to_vec();
        body.extend_from_slice(b"SANDI-1,661.0,0.266,0.415,");
        body.push(0x95); // the byte that broke it
        body.extend_from_slice(b" Sst gry f m gr fri wl srt\n");
        let path = write_bytes("sandibumi_cp1252_core.csv", &body);

        let probe = probe_core_table(&path).expect("a cp1252 byte must not fail the import");
        assert_eq!(probe.headers.len(), 5, "every column still parses");
        let text = read_text_file(&path).unwrap();
        assert!(text.contains('\u{2022}'), "0x95 must decode to a real bullet, not a replacement char");
        assert!(text.contains("Sst gry f m gr fri wl srt"), "the description survives intact");
        let _ = std::fs::remove_file(&path);
    }

    /// A BOM is authoritative and must win over the cp1252 fallback: Excel's "Unicode text"
    /// export is UTF-16LE, and decoding those bytes as cp1252 would yield NUL-riddled nonsense
    /// that parses as one giant column instead of erroring — a silently wrong import.
    #[test]
    fn boms_are_honoured_utf8_and_utf16() {
        let mut u8bom: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
        u8bom.extend_from_slice("WELL,DEPTH\nSANDI-1,10.5\n".as_bytes());
        let p1 = write_bytes("sandibumi_bom8.csv", &u8bom);
        let t1 = read_text_file(&p1).unwrap();
        assert!(t1.starts_with("WELL"), "the UTF-8 BOM must be stripped, not parsed as a header char");

        let mut u16le: Vec<u8> = vec![0xFF, 0xFE];
        for u in "WELL,DEPTH\nSANDI-1,10.5\n".encode_utf16() {
            u16le.extend_from_slice(&u.to_le_bytes());
        }
        let p2 = write_bytes("sandibumi_bom16.csv", &u16le);
        let t2 = read_text_file(&p2).unwrap();
        assert!(t2.starts_with("WELL,DEPTH"), "UTF-16LE must decode as text, got {:?}", &t2[..t2.len().min(24)]);
        assert!(!t2.contains('\u{0}'), "must not fall through to cp1252 and leave NULs");

        let probe = probe_core_table(&p2).expect("a UTF-16 export must import");
        assert_eq!(probe.headers.len(), 2);
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
    }

    /// Probe against the real core delivery that reported this bug — a 330 KB table, pure
    /// ASCII apart from two 0x95 bullets in a lithology description, which the old reader
    /// refused outright. Ignored, and skipped with a printed reason when no core folder is
    /// configured (`SANDIBUMI_FIELD_FIXTURES/core/`). Run with:
    ///   cargo test parsers::encoding_tests::probe_real_field_core -- --ignored --nocapture
    #[test]
    #[ignore]
    fn probe_real_field_core() {
        let Some(path) = crate::field_fixtures::core_table() else {
            crate::field_fixtures::skip("probe_real_field_core", 0, 1);
            return;
        };
        let path = path.to_string_lossy().into_owned();
        let p = probe_core_table(&path).expect("the real file must import");
        eprintln!("headers ({}): {:?}", p.headers.len(), p.headers);
        eprintln!("data rows: {}", p.n_rows);
        eprintln!(
            "roles  well={:?} depth={:?} cpor={:?} cperm={:?} cgd={:?} csw={:?}",
            p.well, p.depth, p.cpor, p.cperm, p.cgd, p.csw
        );
        eprintln!("percent roles: {:?}  depth unit: {:?}", p.percent_roles, p.depth_unit_guess);
        eprintln!("wells routed: {}", p.wells.len());
        for w in p.wells.iter().take(8) {
            eprintln!("   {:?}", w);
        }
    }

    /// Plain UTF-8 (the common case) must be untouched by the fallback — including real
    /// multi-byte characters, which cp1252 decoding would mangle into mojibake.
    #[test]
    fn valid_utf8_is_passed_through_unchanged() {
        let body = "WELL,DEPTH,NOTE\nSANDI-1,10.5,µ-porosity 30°C – ok\n";
        let path = write_bytes("sandibumi_utf8.csv", body.as_bytes());
        assert_eq!(read_text_file(&path).unwrap(), body);
        let _ = std::fs::remove_file(&path);
    }
}

#[cfg(test)]
mod tops_aux_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn temp(name: &str, body: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    fn tops_csv_multiwell_aliases() {
        let p = temp(
            "arshilla_tops_test.csv",
            "# exported tops\nWell Name,Surface,MD\nSANDI-1,TOP_A,1000.5\nSANDI-1,TOP_B,1100.0\nSANDI-2,TOP_A,1010.0\n,BAD_ROW,\n",
        );
        let (has_well, recs) = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(has_well, "multi-well file has a WELL column");
        assert_eq!(recs.len(), 3, "row without depth skipped");
        assert_eq!(recs[0].well.as_deref(), Some("SANDI-1"));
        assert_eq!(recs[2].well.as_deref(), Some("SANDI-2"));
        assert_eq!(recs[1].top_name, "TOP_B");
        assert!((recs[1].depth - 1100.0).abs() < 1e-3);
    }

    /// `pandas.to_csv(na_rep='NaN')` and `np.savetxt` write a literal `NaN` for a missing marker,
    /// and `f32::from_str` parses it happily. Nothing between the parser and `db::upsert_top`
    /// tested finiteness, so the NaN reached Auto-correlate's `markers.sort_by(partial_cmp
    /// .unwrap())` and panicked it — while the DB mutex was held, poisoning it for the rest of
    /// the session. An unorderable depth is not a top, so the row is dropped at the door.
    #[test]
    fn tops_csv_drops_nonfinite_depths() {
        let p = temp(
            "arshilla_tops_nonfinite_test.csv",
            "Well Name,Surface,MD\n\
             SANDI-1,TOP_A,1000.5\n\
             SANDI-1,TOP_MISSING,NaN\n\
             SANDI-1,TOP_NAN_LOWER,nan\n\
             SANDI-1,TOP_INF,inf\n\
             SANDI-1,TOP_OVERFLOW,1.0E+40\n\
             SANDI-1,TOP_B,1100.0\n",
        );
        let (_, recs) = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(recs.len(), 2, "only the two real tops survive, got {recs:?}");
        assert!(
            recs.iter().all(|r| r.depth.is_finite()),
            "no non-finite depth may be stored, got {:?}",
            recs.iter().map(|r| r.depth).collect::<Vec<_>>()
        );
        let names: Vec<&str> = recs.iter().map(|r| r.top_name.as_str()).collect();
        assert_eq!(names, ["TOP_A", "TOP_B"]);
    }

    #[test]
    fn tops_txt_headerless_whitespace() {
        let p = temp("arshilla_tops_test.txt", "TOP_A  1000.5\nTOP_B\t1100\n");
        let (has_well, recs) = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(!has_well, "headerless 2-column file has no WELL column");
        assert_eq!(recs.len(), 2);
        assert!(recs[0].well.is_none());
        assert_eq!(recs[0].top_name, "TOP_A");
        assert!((recs[1].depth - 1100.0).abs() < 1e-3);
    }

    #[test]
    fn interval_file_xrd_mixed_types() {
        let p = temp(
            "arshilla_xrd_test.csv",
            "Depth,Quartz,Illite,Remarks\n2000.0,45.2,12.1,clean\n2001.0,40.0,,silty\n",
        );
        let d = parse_interval_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(d.items, vec!["QUARTZ", "ILLITE", "REMARKS"]);
        assert_eq!(d.rows.len(), 2);
        assert_eq!(d.rows[0].0, 2000.0);
        assert!(d.rows[0].1.is_none(), "no BASE column means point rows");
        assert_eq!(d.rows[1].2[1], None, "empty cell stays None");
        assert_eq!(d.rows[1].2[2].as_deref(), Some("silty"));
    }

    #[test]
    fn interval_file_perforation_intervals() {
        let p = temp(
            "arshilla_perf_test.csv",
            "FROM,TO,STATUS\n2050.0,2055.0,OPEN\n2100.0,2104.0,SQUEEZED\n",
        );
        let d = parse_interval_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(d.items, vec!["STATUS"]);
        assert_eq!(d.rows[0].1, Some(2055.0));
        assert_eq!(d.rows[1].2[0].as_deref(), Some("SQUEEZED"));
    }
}

#[cfg(test)]
mod las_depth_tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn temp(name: &str, body: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    // Build a CurveColumns whose companion curves carry their row index as a marker value, so
    // a test can prove the companion columns were filtered in lockstep with the depth column.
    fn cols_from(depth: Vec<f32>) -> CurveColumns {
        let seq: Vec<f32> = (0..depth.len()).map(|i| i as f32).collect();
        CurveColumns {
            depth_unit: None,
            depth,
            gr: seq.clone(),
            res: seq.clone(),
            nphi: seq.clone(),
            rhob: seq.clone(),
            dt: seq.clone(),
            sp: seq,
        }
    }

    /// SB-DIO-004 / SB-DIO-T06..T08. The relative tolerance and its 1.0 floor are
    /// specified in `docs/PRD_v2/21_data-io.md` §5.2. Recognition changes a matched
    /// sentinel to the internal missing representation; it never canonicalises one
    /// finite sentinel into another finite value.
    #[test]
    fn null_recognition_is_one_relative_tolerance_transform_and_recognition_never_rewrites() {
        let represented = (-999.250_06_f32 as f64) as f32;
        assert!(is_las_null(represented), "one f32/f64 representation change stays within tolerance");
        assert!(is_las_null(-999.251), "the relative tolerance accepts a nearby formatter result");
        assert!(!is_las_null(-999.20), "a nearby real reading outside the tolerance must survive");

        let body = "~VERSION\nVERS. 2.0 :\n~WELL\nNULL. -12345 :\nWELL. SANDI-NULL :\n\
                    ~CURVE\nDEPT.M :\nGR.API :\n~ASCII\n1000 -12345.1\n1001 -12344.0\n";
        let p = temp("sandibumi_relative_null_test.las", body);
        let cols = parse_las_2(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(cols.gr[0].is_nan(), "a declared near-sentinel becomes the internal absent value");
        assert_eq!(cols.gr[1], -12344.0, "recognition must not rewrite a surviving value to another sentinel");
    }

    #[test]
    fn sanitize_drops_nonfinite_and_duplicate_depths() {
        // depths: 2000, NaN, 2001, 2000 (dup of row 0), 2002 → keep rows 0, 2, 4.
        let mut c = cols_from(vec![2000.0, f32::NAN, 2001.0, 2000.0, 2002.0]);
        let rep = sanitize_curve_columns(&mut c);
        assert_eq!(rep.nonfinite, 1, "one NaN depth dropped");
        assert_eq!(rep.duplicate, 1, "one duplicate depth dropped");
        assert!(!rep.is_clean());
        assert_eq!(c.depth, vec![2000.0, 2001.0, 2002.0]);
        // Companion columns must follow the same kept indices (0, 2, 4), not slide.
        assert_eq!(c.gr, vec![0.0, 2.0, 4.0]);
        assert_eq!(c.sp, vec![0.0, 2.0, 4.0]);
    }

    #[test]
    fn sanitize_leaves_clean_columns_untouched() {
        let mut c = cols_from(vec![2000.0, 2000.5, 2001.0]);
        let rep = sanitize_curve_columns(&mut c);
        assert!(rep.is_clean() && rep.total() == 0);
        assert_eq!(c.depth.len(), 3);
        assert_eq!(c.gr, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn parse_las_2_tdep_index_populates_depth() {
        // Index mnemonic is TDEP (Schlumberger), not DEPT/DEPTH; -999.25 sentinel in GR.
        let body = "~VERSION\nVERS. 2.0 :\n~WELL\nNULL. -999.25 :\nWELL. XX : NAME\n\
                    ~CURVE\nTDEP.M :\nGR.API :\n~ASCII\n2000.0 55.0\n2000.5 60.0\n2001.0 -999.25\n";
        let p = temp("arshilla_tdep_test.las", body);
        let cols = parse_las_2(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(cols.depth.len(), 3);
        assert!(
            cols.depth.iter().all(|d| d.is_finite()),
            "a TDEP-indexed file must populate depth, got {:?}",
            cols.depth
        );
        assert!((cols.depth[2] - 2001.0).abs() < 1e-3);
        assert!(cols.gr[2].is_nan(), "-999.25 sentinel must map to NaN");
    }

    /// `f32::from_str` returns `Ok(inf)` for an overflowing cell like `1.0E+40` and for the
    /// literal tokens `inf`/`-inf`. Everything downstream screens for missing with `is_nan()`
    /// only (`modules::is_missing`), so an infinity used to survive into the compute cores, where
    /// `inf - inf` made a z-score NaN and panicked the KNN neighbour sort on `partial_cmp`. The
    /// DLIS importer already stripped exactly this; the LAS path did not.
    #[test]
    fn parse_las_2_maps_nonfinite_values_to_missing() {
        let body = "~VERSION\nVERS. 2.0 :\n~WELL\nNULL. -999.25 :\nWELL. XX : NAME\n\
                    ~CURVE\nDEPT.M :\nGR.API :\n~ASCII\n\
                    2000.0 55.0\n2000.5 1.0E+40\n2001.0 -inf\n2001.5 60.0\n";
        let p = temp("arshilla_nonfinite_value_test.las", body);
        let cols = parse_las_2(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(cols.depth.len(), 4, "no ROW is dropped — only the value reads missing");
        assert!(cols.gr[1].is_nan(), "1.0E+40 overflows to +inf and must read missing");
        assert!(cols.gr[2].is_nan(), "the literal token -inf must read missing");
        assert!(
            cols.gr.iter().all(|v| v.is_nan() || v.is_finite()),
            "no infinity may survive the importer, got {:?}",
            cols.gr
        );
        assert!((cols.gr[0] - 55.0).abs() < 1e-3, "good values are untouched");
        assert!((cols.gr[3] - 60.0).abs() < 1e-3);
    }

    #[test]
    fn parse_las_2_unrecognized_index_falls_back_to_column0() {
        // No DEPT/DEPTH mnemonic — the first column is still the LAS index.
        let body = "~CURVE\nXREF.M :\nGR.API :\n~ASCII\n1000.0 10.0\n1000.5 12.0\n";
        let p = temp("arshilla_xref_test.las", body);
        let cols = parse_las_2(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(cols.depth, vec![1000.0, 1000.5], "unrecognized index must fall back to column 0");
    }

    #[test]
    fn parse_las_2_auxiliary_md_curve_does_not_steal_depth() {
        // Column 0 is the true index under an unrecognized mnemonic (TVDSS); a real MD curve
        // sits at column 2. Depth must come from column 0, NOT the MD auxiliary curve — MD is
        // deliberately absent from DEPTH_ALIASES so it can't override the column-0 index.
        let body = "~CURVE\nTVDSS.M :\nGR.API :\nMD.M :\n~ASCII\n\
                    1000.0 55.0 3000.0\n1000.5 60.0 3000.5\n1001.0 62.0 3001.0\n";
        let p = temp("arshilla_aux_md_test.las", body);
        let cols = parse_las_2(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(
            cols.depth,
            vec![1000.0, 1000.5, 1001.0],
            "depth must be the column-0 index (TVDSS), not the MD curve at column 2"
        );
    }

    /// T-IMP-08. A repeated plug depth must be dropped at PARSE time, first occurrence kept.
    ///
    /// `depth_keep_indices` is shared with the LAS path and is well tested there — but a shared
    /// helper being correct says nothing about this caller being wired to it. Remove the dedup
    /// call from `parse_core_csv` and every existing test still passes, while a real core table
    /// with one repeated depth aborts the whole well's import on the `core_data (well_id, depth)`
    /// primary key. One duplicated row in a 3000-plug delivery, and none of it lands.
    ///
    /// **First occurrence wins and file order is kept.** A laboratory that lists a plug twice is
    /// almost always repeating the same measurement, not reporting a second plug at the identical
    /// depth — two plugs cannot occupy one depth. Taking the last would silently prefer whichever
    /// copy the typist edited most recently, which is not a rule anyone can reason about.
    #[test]
    fn a_repeated_plug_depth_is_dropped_not_a_failed_import() {
        let path = std::env::temp_dir().join("sandibumi_core_dup_depth.csv");
        // Row 3 repeats row 1's depth with a DIFFERENT porosity, so which one survives is visible.
        std::fs::write(
            &path,
            "DEPTH,CPOR,CPERM\n\
             2000.00,18.0,50.0\n\
             2000.50,20.0,80.0\n\
             2000.00,31.0,999.0\n\
             2001.00,22.0,120.0\n",
        )
        .unwrap();
        let cols = parse_core_csv(&path).expect("a duplicated depth must not fail the parse");
        std::fs::remove_file(&path).ok();

        assert_eq!(cols.depth, vec![2000.0, 2000.5, 2001.0], "the repeat is dropped, order kept");
        assert_eq!(cols.depth.len(), 3, "four rows in, three plugs out");
        // Percent input converts to v/v, so 18% is the survivor to look for — not 31%.
        assert!(
            (cols.cpor[0] - 0.18).abs() < 1e-4,
            "the FIRST occurrence must win: got {} where 0.18 was expected",
            cols.cpor[0]
        );
        assert!(
            (cols.cperm[0] - 50.0).abs() < 1e-3,
            "and every companion column must follow the same kept row, not slide: {}",
            cols.cperm[0]
        );
    }

    /// T-IMP-12's other half. A repeated station MD is dropped so it cannot abort the survey on
    /// the `well_path (well_id, md)` primary key — the same argument as the plug depth above, and
    /// a duplicated station carries no geometry the first one did not.
    ///
    /// The duplicate here has a **different inclination**, so silently keeping the wrong one would
    /// bend the well: minimum curvature integrates station to station, and an inclination swapped
    /// at one station moves every TVD below it. TVD then feeds saturation-height, so this does not
    /// stay in the survey.
    #[test]
    fn a_repeated_survey_station_is_dropped_not_a_failed_survey() {
        let path = std::env::temp_dir().join("sandibumi_dev_dup_md.csv");
        std::fs::write(
            &path,
            "MD,INC,AZI\n0,0,0\n1000,0,0\n2000,60,45\n2000,5,45\n3000,60,45\n",
        )
        .unwrap();
        let survey = parse_deviation_csv(&path).expect("a duplicated MD must not fail the parse");
        std::fs::remove_file(&path).ok();

        let mds: Vec<f32> = survey.md.clone();
        assert_eq!(mds, vec![0.0, 1000.0, 2000.0, 3000.0], "the repeated station is dropped");
        let at_2000 = mds.iter().position(|m| *m == 2000.0).unwrap();
        assert!(
            (survey.inc[at_2000] - 60.0).abs() < 1e-3,
            "the FIRST station at 2000 m must win — the 5 deg copy would straighten the well \
             and move every TVD below it, got {}",
            survey.inc[at_2000]
        );
    }

    #[test]
    fn sanitize_dedups_signed_zero_depths() {
        // +0.0 and -0.0 have distinct bit patterns but DuckDB's FLOAT PK treats them equal,
        // so the -0.0 must be dropped as a duplicate of the +0.0 (row 0), keeping rows 0 and 2.
        let mut c = cols_from(vec![0.0, -0.0, 1.0]);
        let rep = sanitize_curve_columns(&mut c);
        assert_eq!(rep.nonfinite, 0);
        assert_eq!(rep.duplicate, 1, "-0.0 duplicates +0.0 under DuckDB FLOAT equality");
        assert_eq!(c.depth.len(), 2);
        assert_eq!(c.gr, vec![0.0, 2.0], "companion follows kept indices 0 and 2");
    }
}
