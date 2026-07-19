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

/// A single deserialized row from a Geolog CSV export.
#[derive(Debug, Clone, Deserialize)]
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

/// Parses a Geolog CSV export into columnar arrays, mapping missing values to `f32::NAN`.
pub fn parse_geolog_csv<P: AsRef<Path>>(path: P) -> ParseResult<CurveColumns> {
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

fn is_las_null(v: f32) -> bool {
    LAS_NULL_VALUES.iter().any(|null| (v - null).abs() < f32::EPSILON)
}

enum LasSection {
    Header,
    WellBlock,
    CurveBlock,
    AsciiData,
}

/// Priority-ordered mnemonic aliases per target curve, mirroring the alias tables Geolog/IP
/// ship (e.g. IP's CurveAlias.txt). Among the aliases present in a file, the one with the
/// most populated (non-null) samples wins; priority order only breaks ties. So a raw GR is
/// preferred over a normalized GRN when both are populated, but an all-null placeholder
/// (e.g. an empty simulated NPHIED) is skipped in favour of its populated sibling NPHI_LS.
const DEPTH_ALIASES: [&str; 2] = ["DEPT", "DEPTH"];
const GR_ALIASES: [&str; 2] = ["GR", "GRN"];
const RES_ALIASES: [&str; 8] = ["RES_DEEP", "RESD", "RT", "RES", "DRES", "ILD", "LLD", "AT90"];
const NPHI_ALIASES: [&str; 4] = ["NPHI", "TNPH", "NPHIED", "NPHI_LS"];
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
            LasSection::Header | LasSection::WellBlock => continue,
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
                    idx_depth = resolve_curve_index(&curve_names, &DEPTH_ALIASES);
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
                            .map(|v| if is_las_null(v) { f32::NAN } else { v })
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
            LasSection::Header | LasSection::WellBlock => continue,
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
                        let v = if is_las_null(*raw) { f32::NAN } else { *raw };
                        columns[i].push(v);
                    }
                }
            }
        }
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
        if in_well_block && trimmed.to_uppercase().starts_with("WELL") {
            if let Some(colon_idx) = trimmed.rfind(':') {
                if let Some(value) = trimmed[..colon_idx].split_whitespace().last() {
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
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
const CORE_CPOR_ALIASES: [&str; 6] = ["CPOR", "CORE_POR", "PHI_CORE", "CPHI", "POROSITY", "POR"];
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
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

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
    for result in rdr.records() {
        let record = result?;
        let get = |idx: Option<usize>| -> f32 {
            idx.and_then(|i| record.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(f32::NAN)
        };
        let pc = get(Some(idx_pc));
        let sw = get(Some(idx_sw));
        if pc.is_nan() || sw.is_nan() {
            continue;
        }
        out.push(ScalPcRecord {
            sample_no: idx_sample
                .and_then(|i| record.get(i))
                .and_then(|s| s.trim().parse::<i32>().ok()),
            depth: {
                let d = get(idx_depth);
                if d.is_nan() { None } else { Some(d) }
            },
            perm: get(idx_perm),
            poro: get(idx_poro),
            pc,
            sw,
        });
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
    Ok(survey)
}

/// Parses every LAS file in `dir` concurrently across all CPU threads via `rayon`.
/// Returns a `(path, result)` pair per file so individual parse failures don't abort the batch.
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

/// Parses a formation-tops file (CSV or TXT — Petrel/Geolog-style exports). Needs a
/// recognizable top-name and depth column; a well column makes it multi-well. Headerless
/// two-column "NAME DEPTH" (or three-column "WELL NAME DEPTH") files are also accepted:
/// if no known headers are found and the last column of the first line parses as a
/// number, the first line is treated as data.
pub fn parse_tops_file<P: AsRef<Path>>(path: P) -> ParseResult<Vec<TopsRecord>> {
    let (headers, mut rows) = read_delimited(path)?;
    if headers.is_empty() {
        return Err(ParseError::Las("tops file is empty".into()));
    }

    let idx_name = resolve_header_index(&headers, &TOPS_NAME_ALIASES);
    let idx_depth = resolve_header_index(&headers, &TOPS_DEPTH_ALIASES);
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
    Ok(out)
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
        let recs = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(recs.len(), 3, "row without depth skipped");
        assert_eq!(recs[0].well.as_deref(), Some("BALAM-1"));
        assert_eq!(recs[2].well.as_deref(), Some("BALAM-2"));
        assert_eq!(recs[1].top_name, "TOP_B");
        assert!((recs[1].depth - 1100.0).abs() < 1e-3);
    }

    #[test]
    fn tops_txt_headerless_whitespace() {
        let p = temp("arshilla_tops_test.txt", "TOP_A  1000.5\nTOP_B\t1100\n");
        let recs = parse_tops_file(&p).unwrap();
        std::fs::remove_file(&p).ok();
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
