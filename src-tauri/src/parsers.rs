use rayon::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));

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

pub(crate) fn is_las_null(v: f32) -> bool {
    LAS_NULL_VALUES.iter().any(|null| (v - null).abs() < f32::EPSILON)
}

/// Null test honoring the file's own `~W NULL` declaration on top of the standard
/// sentinels — deliveries using e.g. -99999 or 999.25 otherwise import as data.
fn is_null_value(v: f32, declared: Option<f32>) -> bool {
    is_las_null(v) || declared.is_some_and(|n| (v - n).abs() <= n.abs().max(1.0) * 1e-5)
}

/// Parse the NULL value from a `~W` block line ("NULL .  -999.25 : NULL VALUE").
fn parse_null_line(trimmed: &str) -> Option<f32> {
    if !trimmed.to_uppercase().starts_with("NULL") {
        return None;
    }
    trimmed.split(':').next()?.split_whitespace().last()?.parse::<f32>().ok()
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
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut section = LasSection::Header;
    let mut curve_names: Vec<String> = Vec::new();
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
    let mut declared_null: Option<f32> = None;

    for line in reader.lines() {
        let line = line?;
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
            LasSection::Header => continue,
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

                for tok in trimmed.split_whitespace() {
                    let v: f32 = tok
                        .parse()
                        .map_err(|e| ParseError::Las(format!("bad numeric token '{tok}': {e}")))?;
                    token_buffer.push(v);
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
                }
            }
        }
    }

    // A short/truncated ~A row leaves tokens that never fill a complete column set; from
    // that point on every value is shifted a column left (GR lands in RES, etc.). Fail
    // loudly rather than silently mis-columning the rest of the file.
    if !token_buffer.is_empty() {
        return Err(ParseError::Las(format!(
            "ASCII data ended with {} leftover token(s) not forming a full {}-column row (truncated or corrupt LAS?)",
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
    #[allow(dead_code)]
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
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut section = LasSection::Header;
    let mut curve_names: Vec<String> = Vec::new();
    let mut curve_units: Vec<Option<String>> = Vec::new();
    // One value column per curve, filled in ~A order.
    let mut columns: Vec<Vec<f32>> = Vec::new();
    let mut idx_depth: Option<usize> = None;
    let mut indices_resolved = false;
    let mut token_buffer: Vec<f32> = Vec::new();
    let mut declared_null: Option<f32> = None;

    for line in reader.lines() {
        let line = line?;
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
            LasSection::Header => continue,
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
                for tok in trimmed.split_whitespace() {
                    let v: f32 =
                        tok.parse().map_err(|e| ParseError::Las(format!("bad numeric token '{tok}': {e}")))?;
                    token_buffer.push(v);
                }
                while token_buffer.len() >= expected_per_row {
                    let row: Vec<f32> = token_buffer.drain(0..expected_per_row).collect();
                    for (i, raw) in row.iter().enumerate() {
                        let v = if is_null_value(*raw, declared_null) { f32::NAN } else { *raw };
                        columns[i].push(v);
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
            "ASCII data ended with {} leftover token(s) not forming a full {}-column row (truncated or corrupt LAS?)",
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
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut in_well_block = false;
    for line in reader.lines() {
        let line = line?;
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
                // "WELL .        BALAM SOUTH-01   : WELL" — the value is everything
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
const CORE_CGD_ALIASES: [&str; 4] = ["CGD", "GRAIN_DENSITY", "GRAIN_DEN", "RHOG"];
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
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

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
    let file = File::open(&path)?;
    let mut rdr =
        csv::ReaderBuilder::new().delimiter(delim).has_headers(true).from_reader(BufReader::new(file));

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
    let file = File::open(&path)?;
    for line in std::io::BufRead::lines(BufReader::new(file)) {
        let line = line?;
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
    let file = File::open(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(BufReader::new(file));

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
    let file = File::open(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(BufReader::new(file));

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
    let file = File::open(&path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(BufReader::new(file));

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
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));
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

const TOPS_WELL_ALIASES: [&str; 7] =
    ["WELL", "WELLNAME", "WELL_NAME", "WELLBORE", "BOREHOLE", "UWI", "WELL_ID"];
const TOPS_NAME_ALIASES: [&str; 9] =
    ["TOP", "TOP_NAME", "TOPS", "MARKER", "SURFACE", "FORMATION", "HORIZON", "ZONE", "NAME"];
const TOPS_DEPTH_ALIASES: [&str; 7] =
    ["DEPTH", "MD", "TOP_MD", "MD_TOP", "TOP_DEPTH", "DEPT", "TVD"];

/// Reads a delimited text file into (headers, rows), auto-detecting the delimiter from
/// the first non-comment line: tab, then semicolon, then comma, else runs of whitespace.
/// Quoted fields are honoured for the csv-crate delimiters; whitespace mode is a plain
/// split (well names with spaces need a tab or comma file). Lines starting with '#' skip.
fn read_delimited<P: AsRef<Path>>(path: P) -> ParseResult<(Vec<String>, Vec<Vec<String>>)> {
    let text = std::fs::read_to_string(path)?;
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
            .and_then(|s| s.parse::<f32>().ok());
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
}

const AUX_TOP_ALIASES: [&str; 7] = ["TOP", "DEPTH", "TOP_MD", "FROM", "TOP_DEPTH", "MD", "DEPT"];
const AUX_BASE_ALIASES: [&str; 6] = ["BASE", "BOTTOM", "TO", "BASE_MD", "BOT", "BOTTOM_DEPTH"];

/// Parses a tops-style dataset file (CSV/TXT, same delimiter detection as tops): needs a
/// TOP/DEPTH column; BASE/BOTTOM makes rows intervals; a WELL column is ignored (the
/// import dialog binds the file to one well). All remaining columns become items.
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

    let mut out = IntervalData { items, rows: Vec::new() };
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
        out.rows.push((top, base, values));
    }
    if out.rows.is_empty() {
        return Err(ParseError::Las("data file has no parsable rows".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tops_aux_tests {
    use super::*;
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
            "# exported tops\nWell Name,Surface,MD\nBALAM-1,TOP_A,1000.5\nBALAM-1,TOP_B,1100.0\nBALAM-2,TOP_A,1010.0\n,BAD_ROW,\n",
        );
        let (has_well, recs) = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(has_well, "multi-well file has a WELL column");
        assert_eq!(recs.len(), 3, "row without depth skipped");
        assert_eq!(recs[0].well.as_deref(), Some("BALAM-1"));
        assert_eq!(recs[2].well.as_deref(), Some("BALAM-2"));
        assert_eq!(recs[1].top_name, "TOP_B");
        assert!((recs[1].depth - 1100.0).abs() < 1e-3);
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
            depth,
            gr: seq.clone(),
            res: seq.clone(),
            nphi: seq.clone(),
            rhob: seq.clone(),
            dt: seq.clone(),
            sp: seq,
        }
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
