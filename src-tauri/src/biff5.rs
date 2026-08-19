//! SB-DIO-060 (DEC-076: "biff5 reader"): a narrow reader for the wellsite `.xls` that is
//! a BIFF record stream — bare (no container) or inside an OLE2 compound document —
//! derived from the PUBLISHED specifications and nothing else:
//!
//! - [MS-CFB] Compound File Binary File Format (the OLE2 container).
//! - [MS-XLS] Excel Binary File Format; record ids and layouts cited per section below.
//!
//! Deliberately narrow, per the ruling: BIFF5 (and the minimal BIFF2 cell records the
//! wellsite case carries) are READ; BIFF3/BIFF4/BIFF8 are REFUSED BY VERSION NAME — the
//! version disagreement is reported, never guessed around (SB-DIO-T89's second half).
//! The drawing layer is not touched: cells are read without it (SB-DIO-T88); plate
//! anchors belong to SB-DIO-058. No formula is ever evaluated — a FORMULA record
//! contributes only its cached numeric result (DEC-051's no-execution family).

/// One decoded sheet as a dense text grid — the shape `intake::split_table` already
/// consumes, so every downstream behaviour (header row, units row, roles, decimal
/// parsing, the SB-DIO-007 source-cell states) is inherited rather than re-implemented.
#[derive(Debug)]
pub struct BiffTable {
    /// "BIFF2" | "BIFF5" — the version actually read.
    pub version: &'static str,
    /// "bare stream" | "OLE2 compound document".
    pub container: &'static str,
    /// First worksheet's BOUNDSHEET name, when the workbook names one.
    pub sheet: Option<String>,
    /// CODEPAGE record value; 1252 when the stream declares none ([MS-XLS] 2.4.52).
    pub codepage: u16,
    /// Dense grid: numbers rendered, absent cells "".
    pub rows: Vec<Vec<String>>,
    /// Everything skipped or approximated, named — never silent.
    pub notes: Vec<String>,
}

const OLE2_SIG: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// True when the bytes carry either signature `intake::detect_format` cites: the OLE2
/// compound-document header, or a headerless BIFF BOF record at offset zero.
pub fn is_biff(bytes: &[u8]) -> bool {
    if bytes.starts_with(&OLE2_SIG) {
        return true;
    }
    // Bare-stream BOF ids: 0x0009 (BIFF2), 0x0209 (BIFF3), 0x0409 (BIFF4),
    // 0x0809 (BIFF5/BIFF8) — [MS-XLS] 2.4.21.
    matches!(read_u16(bytes, 0), Some(0x0009 | 0x0209 | 0x0409 | 0x0809))
}

fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]))
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *bytes.get(at)?,
        *bytes.get(at + 1)?,
        *bytes.get(at + 2)?,
        *bytes.get(at + 3)?,
    ]))
}

fn read_f64(bytes: &[u8], at: usize) -> Option<f64> {
    let slice: [u8; 8] = bytes.get(at..at + 8)?.try_into().ok()?;
    Some(f64::from_le_bytes(slice))
}

/// cp1252 byte decoding. 0x80–0x9F is the only block where cp1252 differs from Latin-1
/// (same table `parsers.rs` documents); every other byte IS its code point. Bytes are
/// interpreted, never rejected — the text-import doctrine.
fn decode_cp1252(bytes: &[u8]) -> String {
    const HIGH: [char; 32] = [
        '\u{20AC}', '\u{81}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}',
        '\u{2021}', '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{8D}',
        '\u{017D}', '\u{8F}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}',
        '\u{2022}', '\u{2013}', '\u{2014}', '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}',
        '\u{0153}', '\u{9D}', '\u{017E}', '\u{0178}',
    ];
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => HIGH[(b - 0x80) as usize],
            other => other as char,
        })
        .collect()
}

/// RK-number decoding, [MS-XLS] 2.5.276: bit 0 = value was multiplied by 100, bit 1 =
/// the top 30 bits are a signed integer (else the high 34 bits of an IEEE double).
fn decode_rk(rk: u32) -> f64 {
    let value = if rk & 0x02 != 0 {
        ((rk as i32) >> 2) as f64
    } else {
        f64::from_bits(((rk & 0xFFFF_FFFC) as u64) << 32)
    };
    if rk & 0x01 != 0 { value / 100.0 } else { value }
}

fn render_number(value: f64) -> String {
    format!("{value}")
}

// ---------------------------------------------------------------------------
// [MS-CFB]: the OLE2 compound-document walk — just enough to hand back one
// named stream. Version-3 files only (512-byte sectors), which is what every
// BIFF5-era workbook is.
// ---------------------------------------------------------------------------

const ENDOFCHAIN: u32 = 0xFFFF_FFFE;
const FREESECT: u32 = 0xFFFF_FFFF;

fn cfb_stream(bytes: &[u8], names: &[&str], path: &str) -> Result<Vec<u8>, String> {
    // Header, [MS-CFB] 2.2: sector shift at offset 30; number of FAT sectors at 44;
    // first directory sector at 48; the first 109 DIFAT entries at 76.
    let sector_shift = read_u16(bytes, 30)
        .ok_or_else(|| format!("{path}: OLE2 header truncated at the sector-shift field"))?;
    let sector = 1usize << sector_shift;
    let fat_count = read_u32(bytes, 44).unwrap_or(0) as usize;
    let first_dir = read_u32(bytes, 48).unwrap_or(ENDOFCHAIN);
    let mut fat_sectors: Vec<u32> = Vec::new();
    for index in 0..109.min(fat_count) {
        match read_u32(bytes, 76 + index * 4) {
            Some(FREESECT) | None => break,
            Some(entry) => fat_sectors.push(entry),
        }
    }
    let sector_at = |number: u32| -> Result<&[u8], String> {
        let start = 512 + number as usize * sector;
        bytes.get(start..start + sector).ok_or_else(|| {
            format!("{path}: OLE2 sector {number} lies beyond the file - truncated container")
        })
    };
    let mut fat: Vec<u32> = Vec::new();
    for fs in &fat_sectors {
        let data = sector_at(*fs)?;
        for chunk in data.chunks_exact(4) {
            fat.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }
    let chain = |start: u32| -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let mut current = start;
        let mut hops = 0;
        while current != ENDOFCHAIN {
            out.extend_from_slice(sector_at(current)?);
            current = *fat.get(current as usize).ok_or_else(|| {
                format!("{path}: OLE2 FAT has no entry for sector {current} - broken chain")
            })?;
            hops += 1;
            if hops > fat.len() + 1 {
                return Err(format!("{path}: OLE2 FAT chain loops - refusing to read forever"));
            }
        }
        Ok(out)
    };
    // Directory entries, [MS-CFB] 2.6.1: 128 bytes each; UTF-16 name (length at 64),
    // object type at 66 (2 = stream, 5 = root), start sector at 116, stream size at 120.
    let directory = chain(first_dir)?;
    // The mini stream, [MS-CFB] 2.7: a stream SMALLER than the cutoff (4096) lives in
    // 64-byte mini sectors inside the ROOT entry's own stream, chained by the mini FAT -
    // which is exactly the size of a small wellsite workbook, so skipping this would
    // fail the very files this reader exists for.
    let mini_cutoff = read_u32(bytes, 56).unwrap_or(4096) as usize;
    let first_minifat = read_u32(bytes, 60).unwrap_or(ENDOFCHAIN);
    let root = directory
        .chunks_exact(128)
        .find(|entry| entry[66] == 5)
        .map(|entry| {
            (read_u32(entry, 116).unwrap_or(ENDOFCHAIN), read_u32(entry, 120).unwrap_or(0) as usize)
        });
    let mini_chain = |start: u32, size: usize| -> Result<Vec<u8>, String> {
        let (root_start, root_size) =
            root.ok_or_else(|| format!("{path}: OLE2 mini stream requested but no root entry"))?;
        let mut ministream = chain(root_start)?;
        ministream.truncate(root_size);
        let minifat = if first_minifat == ENDOFCHAIN { Vec::new() } else { chain(first_minifat)? };
        let entry_at = |number: u32| -> Option<u32> {
            let at = number as usize * 4;
            minifat.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        let mut out = Vec::new();
        let mut current = start;
        let mut hops = 0;
        while current != ENDOFCHAIN && out.len() < size {
            let at = current as usize * 64;
            let sector = ministream.get(at..(at + 64).min(ministream.len())).ok_or_else(|| {
                format!("{path}: OLE2 mini sector {current} lies beyond the mini stream")
            })?;
            out.extend_from_slice(sector);
            current = entry_at(current).ok_or_else(|| {
                format!("{path}: OLE2 mini FAT has no entry for sector {current}")
            })?;
            hops += 1;
            if hops > minifat.len() / 4 + 2 {
                return Err(format!("{path}: OLE2 mini FAT chain loops"));
            }
        }
        out.truncate(size);
        Ok(out)
    };
    for entry in directory.chunks_exact(128) {
        let name_len = read_u16(entry, 64).unwrap_or(0) as usize;
        if entry[66] != 2 || name_len < 2 {
            continue;
        }
        let name: String = entry[..name_len.saturating_sub(2).min(64)]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .map(|unit| char::from_u32(unit as u32).unwrap_or('\u{FFFD}'))
            .collect();
        if names.iter().any(|wanted| name.eq_ignore_ascii_case(wanted)) {
            let start = read_u32(entry, 116).unwrap_or(ENDOFCHAIN);
            let size = read_u32(entry, 120).unwrap_or(0) as usize;
            if size < mini_cutoff {
                return mini_chain(start, size);
            }
            let mut stream = chain(start)?;
            stream.truncate(size);
            return Ok(stream);
        }
    }
    Err(format!(
        "{path}: OLE2 compound document holds no {} stream - not an Excel workbook",
        names.join("/")
    ))
}

// ---------------------------------------------------------------------------
// The BIFF record stream.
// ---------------------------------------------------------------------------

/// Reads the first worksheet of a `.xls` that is a bare BIFF stream or an OLE2-contained
/// BIFF5 workbook. BIFF3/4/8 refuse BY VERSION NAME - reported, never guessed around.
pub fn parse_biff_table(path: &str) -> Result<BiffTable, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    if bytes.starts_with(&OLE2_SIG) {
        let stream = cfb_stream(&bytes, &["Book", "Workbook"], path)?;
        parse_record_stream(&stream, "OLE2 compound document", path)
    } else {
        parse_record_stream(&bytes, "bare stream", path)
    }
}

fn parse_record_stream(
    stream: &[u8],
    container: &'static str,
    path: &str,
) -> Result<BiffTable, String> {
    let bof_id = read_u16(stream, 0)
        .ok_or_else(|| format!("{path}: empty stream where a BIFF BOF record was expected"))?;
    // BOF, [MS-XLS] 2.4.21: the record id names the era; the payload's first u16 is the
    // version for the 0x0809 id (0x0500 = BIFF5, 0x0600 = BIFF8).
    let version: &'static str = match bof_id {
        0x0009 => "BIFF2",
        0x0209 => {
            return Err(format!(
                "{path}: BIFF3 stream (BOF 0x0209) - the ruled reader covers BIFF5 and the \
                 BIFF2 wellsite records; the version disagreement is reported, not guessed around \
                 (SB-DIO-060)"
            ))
        }
        0x0409 => {
            return Err(format!(
                "{path}: BIFF4 stream (BOF 0x0409) - the ruled reader covers BIFF5 and the \
                 BIFF2 wellsite records (SB-DIO-060)"
            ))
        }
        0x0809 => match read_u16(stream, 4) {
            Some(0x0500) => "BIFF5",
            Some(0x0600) => {
                return Err(format!(
                    "{path}: BIFF8 workbook (BOF version 0x0600) - the ruled reader covers \
                     BIFF5; save as Excel 5.0/95, CSV, or LAS instead (SB-DIO-060)"
                ))
            }
            Some(other) => {
                return Err(format!(
                    "{path}: BOF 0x0809 with unrecognised version 0x{other:04X} (SB-DIO-060)"
                ))
            }
            None => return Err(format!("{path}: BOF record truncated at the version field")),
        },
        other => {
            return Err(format!(
                "{path}: stream does not begin with a BIFF BOF record (found id 0x{other:04X})"
            ))
        }
    };

    let mut codepage: u16 = 1252;
    let mut notes: Vec<String> = Vec::new();
    let mut sheet: Option<String> = None;
    let mut cells: std::collections::BTreeMap<(u16, u16), String> = std::collections::BTreeMap::new();
    // A BIFF5 workbook is substreams: globals BOF..EOF, then one BOF..EOF per sheet.
    // The FIRST worksheet substream is read; further ones are counted, never silently
    // dropped. A bare stream is its own single substream.
    let mut depth = 0usize; // how many BOFs deep we are
    let mut worksheets_seen = 0usize;
    let mut reading_sheet = false;
    let mut skipped_formula_strings = 0usize;

    let mut pos = 0usize;
    while pos + 4 <= stream.len() {
        let id = read_u16(stream, pos).unwrap();
        let len = read_u16(stream, pos + 2).unwrap() as usize;
        let Some(body) = stream.get(pos + 4..pos + 4 + len) else {
            return Err(format!(
                "{path}: record 0x{id:04X} at offset {pos} declares {len} bytes past the end of \
                 the stream - truncated file (SB-DIO-061 locates it rather than reading garbage)"
            ));
        };
        match id {
            0x0009 | 0x0209 | 0x0409 | 0x0809 => {
                depth += 1;
                // Worksheet substream: dt field ([MS-XLS] 2.4.21) - 0x0010 for the
                // 0x0809 family; BIFF2's BOF carries no dt and IS the sheet.
                let is_sheet = if id == 0x0809 {
                    read_u16(body, 2) == Some(0x0010)
                } else {
                    true
                };
                if is_sheet {
                    worksheets_seen += 1;
                    reading_sheet = worksheets_seen == 1;
                }
            }
            // EOF, [MS-XLS] 2.4.103.
            0x000A => {
                depth = depth.saturating_sub(1);
                reading_sheet = false;
                if depth == 0 && container == "bare stream" {
                    break;
                }
            }
            // CODEPAGE, [MS-XLS] 2.4.52.
            0x0042 => {
                if let Some(cp) = read_u16(body, 0) {
                    codepage = cp;
                    if cp != 1252 && cp != 437 && cp != 850 {
                        notes.push(format!(
                            "declared codepage {cp} is decoded with the cp1252 table - byte \
                             values above ASCII may render approximately (bytes are interpreted, \
                             never rejected)"
                        ));
                    }
                }
            }
            // BOUNDSHEET, [MS-XLS] 2.4.28: position u32, options u16, name length u8.
            0x0085 => {
                if sheet.is_none() && body.len() >= 7 {
                    let cch = body[6] as usize;
                    if let Some(raw) = body.get(7..7 + cch) {
                        sheet = Some(decode_cp1252(raw));
                    }
                }
            }
            _ if !reading_sheet => {}
            // NUMBER, [MS-XLS] 2.4.174: rw, col, ixfe, IEEE double.
            0x0203 => {
                if let (Some(rw), Some(col), Some(value)) =
                    (read_u16(body, 0), read_u16(body, 2), read_f64(body, 6))
                {
                    cells.insert((rw, col), render_number(value));
                }
            }
            // RK, [MS-XLS] 2.4.220.
            0x027E => {
                if let (Some(rw), Some(col), Some(rk)) =
                    (read_u16(body, 0), read_u16(body, 2), read_u32(body, 6))
                {
                    cells.insert((rw, col), render_number(decode_rk(rk)));
                }
            }
            // MULRK, [MS-XLS] 2.4.175: rw, colFirst, then (ixfe, RK) pairs, colLast.
            0x00BD => {
                if let (Some(rw), Some(col_first)) = (read_u16(body, 0), read_u16(body, 2)) {
                    let mut at = 4;
                    let mut col = col_first;
                    while at + 6 <= body.len().saturating_sub(2) {
                        if let Some(rk) = read_u32(body, at + 2) {
                            cells.insert((rw, col), render_number(decode_rk(rk)));
                        }
                        at += 6;
                        col = col.saturating_add(1);
                    }
                }
            }
            // LABEL, [MS-XLS] 2.4.148 (BIFF5 form): rw, col, ixfe, cch u16, bytes.
            0x0204 => {
                if let (Some(rw), Some(col), Some(cch)) =
                    (read_u16(body, 0), read_u16(body, 2), read_u16(body, 6))
                {
                    if let Some(raw) = body.get(8..8 + cch as usize) {
                        cells.insert((rw, col), decode_cp1252(raw));
                    }
                }
            }
            // FORMULA, [MS-XLS] 2.4.127: only the CACHED numeric result is taken -
            // 0xFFFF in the last two bytes marks a non-numeric cache, which is skipped
            // and counted. The formula itself is never evaluated.
            0x0006 => {
                if let (Some(rw), Some(col)) = (read_u16(body, 0), read_u16(body, 2)) {
                    if read_u16(body, 12) == Some(0xFFFF) {
                        skipped_formula_strings += 1;
                    } else if let Some(value) = read_f64(body, 6) {
                        cells.insert((rw, col), render_number(value));
                    }
                }
            }
            // BIFF2 cell records ([MS-XLS] 2.4.126/2.4.173/2.4.147): rw, col, 3 cell
            // attribute bytes, then the payload.
            0x0002 if version == "BIFF2" => {
                if let (Some(rw), Some(col), Some(value)) =
                    (read_u16(body, 0), read_u16(body, 2), read_u16(body, 7))
                {
                    cells.insert((rw, col), render_number(value as f64));
                }
            }
            0x0003 if version == "BIFF2" => {
                if let (Some(rw), Some(col), Some(value)) =
                    (read_u16(body, 0), read_u16(body, 2), read_f64(body, 7))
                {
                    cells.insert((rw, col), render_number(value));
                }
            }
            0x0004 if version == "BIFF2" => {
                if let (Some(rw), Some(col)) = (read_u16(body, 0), read_u16(body, 2)) {
                    let cch = body.get(7).copied().unwrap_or(0) as usize;
                    if let Some(raw) = body.get(8..8 + cch) {
                        cells.insert((rw, col), decode_cp1252(raw));
                    }
                }
            }
            // Everything else - formats, fonts, window state, the drawing layer - is
            // outside the ruled scope: cells are read without it (SB-DIO-T88).
            _ => {}
        }
        pos += 4 + len;
    }

    if worksheets_seen > 1 {
        notes.push(format!(
            "workbook carries {worksheets_seen} sheet substreams; only the first was read - \
             the others are named here rather than silently dropped"
        ));
    }
    if skipped_formula_strings > 0 {
        notes.push(format!(
            "{skipped_formula_strings} formula cell(s) cached non-numeric results and were left \
             empty - formulas are never evaluated (SB-DIO-060)"
        ));
    }
    if cells.is_empty() {
        return Err(format!(
            "{path}: {version} {container} carries no readable cell records"
        ));
    }
    let max_row = cells.keys().map(|(r, _)| *r).max().unwrap_or(0) as usize;
    let max_col = cells.keys().map(|(_, c)| *c).max().unwrap_or(0) as usize;
    let mut rows = vec![vec![String::new(); max_col + 1]; max_row + 1];
    for ((r, c), value) in cells {
        rows[r as usize][c as usize] = value;
    }
    Ok(BiffTable { version, container, sheet, codepage, rows, notes })
}
