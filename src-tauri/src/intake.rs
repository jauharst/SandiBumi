//! **Intake** — one importer for any delimited text, replacing the five table-shaped dialogs
//! (Jauhar, 2026-08-05: *"Replace the table ones"*).
//!
//! ## An extractor and a front end, NOT a second write path
//!
//! The single most important decision here. `ingest::import_core_table` already does everything
//! that is hard about writing a delivered table: it routes rows to wells by name under the
//! exactly-one-match rule, converts feet to metres, halves a percent column, replaces per well,
//! dedups depths, and carries every column no core role claimed into `aux_data` typed per CELL.
//! Intake produces a `CoreMapping` and calls it. A second write path would eventually disagree
//! with the first about one of those rules, silently — the standing `composite.rs`-versus-renderer
//! warning, and the same reasoning that made the plate workbook reader an EXTRACTOR feeding
//! `import_images` rather than a second image importer.
//!
//! What Intake adds is the front end the five dialogs never had: the grid IS the control, every
//! guess is a visible editable proposal with its reason, several files at once, and pasted text
//! treated exactly like a file.
//!
//! ## Four rules
//!
//! **Nothing is sniffed that the user can state.** The delimiter, the header row, the units row
//! and the decimal convention are all GUESSED and all overridable, and the grid re-reads live so
//! a wrong guess is visible rather than inferred from a bad result afterwards. This is the same
//! discipline as the declared stain, the declared impregnation, the declared UV light and the
//! declared long/wide/block layout: where the evidence for a reading is the thing being read,
//! the user states it.
//!
//! **The decimal convention comes from the workbook reader.** One delivered petrography book
//! wrote 103 sheets `6980.71 FT` and 18 of them `7016,54 FT` — one laboratory, one report, two
//! people — and reading only the dot convention put a seventh of the delivery at 54 feet on rock
//! cored at 7,000. A delimited core table can do exactly the same, so [`parse_number`] applies
//! the same rule: with both separators present the RIGHTMOST is the decimal; a single separator
//! is a decimal unless the token is validly grouped.
//!
//! **A column with no role is CARRIED, never dropped.** `core_data` has four measurement slots
//! and a real lab export is wider than that, so everything else lands in `aux_data` at the same
//! plug depths. Dropping the unclaimed columns is how a lithology description, a Kv/Kh or an
//! oil-show note goes missing between the delivery and the project.
//!
//! **The preview is a CHECK, not a picture.** The grid shows the file's own text — a user needs
//! to see what was delivered — and every cell that sits in a numeric column and did not parse is
//! flagged ([`IntakeProbe::preview_bad`]). A stray unit, a spreadsheet's `#N/A` or a depth read
//! under the wrong decimal convention is therefore visible BEFORE anything is stored, which is the
//! one thing the five dialogs this replaces could only report afterwards.

use crate::db::{self, DbResult};
use crate::parsers::{self, CoreMapping, ParseResult};
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// Rows shown in the preview grid. Enough to see a units row, a change of convention part way
/// down, and a block of blanks; far short of holding a 200,000-row export in a browser.
const PREVIEW_ROWS: usize = 200;

/// Rows sampled when sniffing a column's type and the file's decimal convention. Reading the
/// whole file to answer "is this column numeric" is the wrong trade on a large delivery, and the
/// answer does not improve past a few hundred rows.
const SNIFF_ROWS: usize = 400;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TableOptions {
    /// `","` `";"` `"\t"` `"ws"` — or absent to auto-detect.
    #[serde(default)]
    pub delimiter: Option<String>,
    /// Lines to skip before the header (a title block, a company banner).
    #[serde(default)]
    pub skip_lines: usize,
    /// `"dot"` | `"comma"` | absent to decide per token (see [`parse_number`]).
    #[serde(default)]
    pub decimal: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntakeColumn {
    pub header: String,
    /// `"number"` | `"text"` | `"empty"`, sniffed from up to [`SNIFF_ROWS`] rows.
    pub kind: String,
    /// The role Intake proposes: `WELL` `DEPTH` `DEPTH_BASE` `CPOR` `CPERM` `CGD` `CSW` `ITEM`
    /// `IGNORE`. A proposal, never applied — the pane shows it and the user overrules.
    pub role: String,
    /// Why that role was proposed, in words, so a wrong guess can be argued with.
    pub reason: String,
    /// Non-empty cells among the sniffed rows — a column that is 3% populated is usually a
    /// stray, and nothing else on the row would say so.
    pub filled: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntakeProbe {
    pub path: String,
    pub columns: Vec<IntakeColumn>,
    /// Data rows after the header (and the units row, when one was detected).
    pub n_rows: usize,
    pub preview: Vec<Vec<String>>,
    /// The delimiter actually used, so the pane can show what it decided.
    pub delimiter: String,
    /// True when the first data row read as units rather than data and was skipped.
    pub units_row_skipped: bool,
    /// `"ft"` / `"m"` when the units row or a depth header named one.
    pub depth_unit_guess: Option<String>,
    /// The decimal convention in force, and how it was arrived at.
    pub decimal: String,
    /// `(row, column)` of every preview cell that sits in a NUMBER column and did not parse.
    ///
    /// This is what makes the preview a check rather than a picture. The grid shows the file's
    /// own text — a user needs to see what was delivered — so a value read under the wrong
    /// decimal convention, a stray unit suffix or a `#N/A` from a spreadsheet would otherwise
    /// look identical to a good cell. Flagged, they are visible BEFORE anything is stored, which
    /// is the one thing the five dialogs this replaces could only report afterwards.
    pub preview_bad: Vec<(usize, usize)>,
    /// Values that are ambiguous under any convention (`1,234` — 1.234 or 1234?). Reported and
    /// read as a DECIMAL, because the wrong answer is then absurd rather than plausible, and an
    /// absurd depth gets looked at while a plausible one gets used. The workbook reader's rule.
    pub ambiguous_numbers: usize,
    pub notes: Vec<String>,
}

/// Reads a number under an explicit or inferred decimal convention.
///
/// Lifted from `images.rs`'s `WORKBOOK_RUNNER::as_number`, where it was written because one
/// delivered book used both conventions in one file. The failure mode is what makes it worth
/// having in two languages: reading only the dot convention did not FAIL on the comma rows, it
/// split `7016,54` and matched `54` — a plausible shallow depth on entirely the wrong sand.
///
/// * both separators present → the RIGHTMOST is the decimal (true of `1,234.56` and `1.234,56`
///   alike, and needs no guess about which locale typed it)
/// * one separator → a decimal, UNLESS the token is validly grouped (1–3 digits, then exactly 3)
/// * `1,234` is genuinely ambiguous and is read as `1.234`, and counted
pub fn parse_number(raw: &str, forced: Option<&str>) -> (Option<f64>, bool) {
    let s = raw.trim();
    if s.is_empty() {
        return (None, false);
    }
    let body: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(f) = forced {
        let cleaned = match f {
            "comma" => body.replace('.', "").replace(',', "."),
            _ => body.replace(',', ""),
        };
        return (cleaned.parse::<f64>().ok(), false);
    }
    let dot = body.rfind('.');
    let comma = body.rfind(',');
    let cleaned = match (dot, comma) {
        (Some(d), Some(c)) => {
            if d > c {
                body.replace(',', "")
            } else {
                body.replace('.', "").replace(',', ".")
            }
        }
        (Some(_), None) => body.clone(),
        (None, Some(c)) => {
            // Validly grouped means thousands: 1–3 digits, then exactly 3 after the separator,
            // and no other separator anywhere. `4633.500` stays three decimal places rather than
            // becoming four and a half million.
            let (lhs, rhs) = body.split_at(c);
            let rhs = &rhs[1..];
            let grouped = (1..=3).contains(&lhs.len())
                && lhs.chars().all(|ch| ch.is_ascii_digit())
                && rhs.len() == 3
                && rhs.chars().all(|ch| ch.is_ascii_digit());
            if grouped {
                // Ambiguous: read as a DECIMAL and flagged, so 1.234 ft against a well of
                // 7,000 is absurd on sight rather than quietly wrong.
                return (body.replace(',', ".").parse::<f64>().ok(), true);
            }
            body.replace(',', ".")
        }
        (None, None) => body.clone(),
    };
    (cleaned.parse::<f64>().ok(), false)
}

/// Header aliases, checked in order. Deliberately a LOCAL table rather than an addition to
/// `curves::FAMILIES`: that table drives curve resolution for the whole project, and widening it
/// to settle a labelling question in an importer would change how every module finds its inputs
/// — the same line `registration::CORE_FAMILIES` takes.
const ROLE_ALIASES: [(&str, &[&str]); 7] = [
    ("WELL", &["WELL", "WELLNAME", "WELL NAME", "WELL_NAME", "WN", "UWI", "BOREHOLE"]),
    ("DEPTH", &["DEPTH", "MD", "DEPT", "SAMPLE DEPTH", "PLUG DEPTH", "TOP", "DEPTH_TOP"]),
    ("DEPTH_BASE", &["BASE", "DEPTH_BASE", "BOTTOM", "DEPTH BOTTOM"]),
    ("CPOR", &["CPOR", "POR", "POROSITY", "PHI", "HELIUM POROSITY", "PHIT_CORE"]),
    ("CPERM", &["CPERM", "PERM", "KAIR", "K AIR", "PERMEABILITY", "KLINK"]),
    ("CGD", &["CGD", "GD", "GRAIN DENSITY", "RHOG", "GRAIN_DENSITY"]),
    ("CSW", &["CSW", "SW", "SW CORE", "WATER SATURATION"]),
];

fn guess_role(header: &str, kind: &str, taken: &[String]) -> (String, String) {
    let h = header.trim().to_uppercase();
    let squashed: String = h.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    for (role, aliases) in ROLE_ALIASES {
        // Each measurement role is claimed ONCE. A lab export routinely carries CPOR and
        // CPOR_CORRECTED; the first is the proposal and the second becomes an item, which keeps
        // it rather than letting the two overwrite each other.
        if taken.iter().any(|t| t == role) {
            continue;
        }
        for a in aliases {
            let asq: String = a.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            if squashed == asq {
                return (role.to_string(), format!("header matches \"{a}\""));
            }
        }
    }
    match kind {
        "empty" => ("IGNORE".into(), "no values in the sampled rows".into()),
        "text" => ("ITEM".into(), "text column — carried as point data".into()),
        _ => ("ITEM".into(), "unrecognised header — carried as point data".into()),
    }
}

/// Reads a table and reports everything the pane needs to confirm the mapping. Writes nothing.
pub fn probe(path: &str, opts: &TableOptions) -> ParseResult<IntakeProbe> {
    let text = parsers::read_text_file(path)?;
    let mut lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    if opts.skip_lines > 0 && opts.skip_lines < lines.len() {
        lines.drain(..opts.skip_lines);
    }
    let Some(first) = lines.first().copied() else {
        return Ok(IntakeProbe {
            path: path.into(),
            columns: vec![],
            n_rows: 0,
            preview: vec![],
            delimiter: "none".into(),
            units_row_skipped: false,
            depth_unit_guess: None,
            decimal: "auto".into(),
            preview_bad: vec![],
            ambiguous_numbers: 0,
            notes: vec!["The file has no readable lines.".into()],
        });
    };

    let delim: Option<u8> = match opts.delimiter.as_deref() {
        Some(",") => Some(b','),
        Some(";") => Some(b';'),
        Some("\t") => Some(b'\t'),
        Some("ws") => None,
        _ => {
            if first.contains('\t') {
                Some(b'\t')
            } else if first.contains(';') {
                Some(b';')
            } else if first.contains(',') {
                Some(b',')
            } else {
                None
            }
        }
    };
    let delim_name = match delim {
        Some(b'\t') => "tab",
        Some(b';') => "semicolon",
        Some(b',') => "comma",
        _ => "whitespace",
    }
    .to_string();

    let mut table: Vec<Vec<String>> = Vec::new();
    match delim {
        Some(d) => {
            let joined = lines.join("\n");
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(d)
                .has_headers(false)
                .flexible(true)
                .from_reader(joined.as_bytes());
            for rec in rdr.records() {
                table.push(rec?.iter().map(|s| s.trim().to_string()).collect());
            }
        }
        None => {
            for line in &lines {
                table.push(line.split_whitespace().map(str::to_string).collect());
            }
        }
    }
    if table.is_empty() {
        return Err(parsers::ParseError::Las("file is empty".into()));
    }
    let headers: Vec<String> = table.remove(0).iter().map(|h| h.trim().to_uppercase()).collect();
    let ncol = headers.len();

    let mut notes = Vec::new();
    // A units row: the row under the header whose cells are units words rather than data. Judged
    // on the whole row rather than on one column, because at this point no depth column has been
    // agreed — and a row that is entirely non-numeric where the table below it is numeric is a
    // units row whichever column you look at.
    let mut units_row_skipped = false;
    let mut depth_unit_guess = None;
    if let Some(row) = table.first() {
        let filled: Vec<&String> = row.iter().filter(|c| !c.trim().is_empty()).collect();
        let numeric = filled.iter().filter(|c| parse_number(c, None).0.is_some()).count();
        if !filled.is_empty() && numeric == 0 {
            for c in &filled {
                let up = c.to_uppercase();
                if up == "FT" || up == "FEET" {
                    depth_unit_guess = Some("ft".to_string());
                } else if up == "M" || up == "METRES" || up == "METERS" {
                    depth_unit_guess = Some("m".to_string());
                }
            }
            units_row_skipped = true;
            table.remove(0);
            notes.push("The row under the header held no numbers, so it was read as units and skipped.".into());
        }
    }
    if depth_unit_guess.is_none() {
        for h in &headers {
            if h.contains("FT") || h.contains("FEET") {
                depth_unit_guess = Some("ft".into());
            }
        }
    }

    // Column kinds and the decimal-convention count, in one pass over the sniff window.
    let mut ambiguous = 0usize;
    let mut columns: Vec<IntakeColumn> = Vec::with_capacity(ncol);
    let mut taken: Vec<String> = Vec::new();
    for c in 0..ncol {
        let (mut num, mut txt, mut filled) = (0usize, 0usize, 0usize);
        for row in table.iter().take(SNIFF_ROWS) {
            let Some(cell) = row.get(c).map(|s| s.trim()).filter(|s| !s.is_empty()) else { continue };
            filled += 1;
            let (v, amb) = parse_number(cell, opts.decimal.as_deref());
            if amb {
                ambiguous += 1;
            }
            if v.is_some() {
                num += 1;
            } else {
                txt += 1;
            }
        }
        let kind = if filled == 0 {
            "empty"
        } else if num >= txt {
            "number"
        } else {
            "text"
        };
        let header = headers[c].clone();
        let (role, reason) = guess_role(&header, kind, &taken);
        if role != "ITEM" && role != "IGNORE" {
            taken.push(role.clone());
        }
        columns.push(IntakeColumn { header, kind: kind.into(), role, reason, filled });
    }

    if !taken.iter().any(|t| t == "DEPTH") {
        notes.push(
            "No column read as a depth. Nothing can be stored without one — set the DEPTH role on \
             the right column before importing."
                .into(),
        );
    }
    if ambiguous > 0 {
        notes.push(format!(
            "{ambiguous} value(s) are ambiguous — a comma with exactly three digits after it is \
             either a decimal or a thousands separator. They are read as DECIMALS, so a wrong \
             reading is absurd rather than plausible. Set the decimal convention if the file uses \
             thousands separators."
        ));
    }

    let preview: Vec<Vec<String>> = table
        .iter()
        .take(PREVIEW_ROWS)
        .map(|r| (0..ncol).map(|c| r.get(c).cloned().unwrap_or_default()).collect())
        .collect();

    // A cell is BAD only in a column the sniff called numeric, and only when it holds something.
    // An empty cell is a legitimate missing measurement — the "blank is not a zero" rule — and
    // flagging it would bury the real failures under a wall of colour on a sparse delivery.
    let mut preview_bad = Vec::new();
    for (ri, row) in preview.iter().enumerate() {
        for (ci, cell) in row.iter().enumerate() {
            if columns[ci].kind == "number"
                && !cell.trim().is_empty()
                && parse_number(cell, opts.decimal.as_deref()).0.is_none()
            {
                preview_bad.push((ri, ci));
            }
        }
    }
    if !preview_bad.is_empty() {
        notes.push(format!(
            "{} cell(s) in the shown rows sit in a numeric column and did not read as a number —              they are marked in the grid. A stray unit, a spreadsheet's #N/A, or the wrong decimal              convention.",
            preview_bad.len()
        ));
    }

    Ok(IntakeProbe {
        path: path.into(),
        columns,
        n_rows: table.len(),
        preview,
        delimiter: delim_name,
        units_row_skipped,
        depth_unit_guess,
        decimal: opts.decimal.clone().unwrap_or_else(|| "auto".into()),
        preview_bad,
        ambiguous_numbers: ambiguous,
        notes,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntakeCommit {
    pub paths: Vec<String>,
    /// One role per column, in file order — what the pane shows, confirmed by the user.
    pub roles: Vec<String>,
    /// `"ft"` / `"m"`; absent leaves the depths as delivered.
    #[serde(default)]
    pub depth_unit: Option<String>,
    /// Delivery set name. One file selection is ONE delivery, auto-suffixed per well so an import
    /// never overwrites — the universal set rule.
    #[serde(default)]
    pub set_name: Option<String>,
    /// Dataset the unclaimed columns land in as point data. Defaults to CORE.
    #[serde(default)]
    pub extras_dataset: Option<String>,
    /// Rows whose well cannot be routed fall back to this well, when the pane offers one.
    #[serde(default)]
    pub fallback_well_id: Option<String>,
    /// The depths in this delivery came from the core report and should follow the core.
    #[serde(default)]
    pub follow_core: bool,
}

/// Turns the pane's confirmed roles into the mapping `ingest::import_core_table` already takes.
///
/// The one place the Intake vocabulary meets the existing write path. Everything not claimed by a
/// core role becomes an EXTRA rather than being dropped — including a column the user explicitly
/// marked `ITEM`, which is the same destination said out loud.
pub fn mapping_from_roles(roles: &[String]) -> Result<CoreMapping, String> {
    let find = |want: &str| roles.iter().position(|r| r == want);
    let depth = find("DEPTH").ok_or(
        "No column is marked DEPTH. Every row has to land at a depth, so there is nothing to \
         store without one.",
    )?;
    let claimed: Vec<usize> = ["WELL", "DEPTH", "DEPTH_BASE", "CPOR", "CPERM", "CGD", "CSW"]
        .iter()
        .filter_map(|r| find(r))
        .collect();
    let extras: Vec<usize> = (0..roles.len())
        .filter(|i| !claimed.contains(i) && roles[*i] != "IGNORE")
        .collect();
    Ok(CoreMapping {
        well: find("WELL"),
        depth,
        cpor: find("CPOR"),
        cperm: find("CPERM"),
        cgd: find("CGD"),
        csw: find("CSW"),
        extras,
    })
}

// ---------------------------------------------------------------------------
// Array layouts — WIDE and BLOCK
// ---------------------------------------------------------------------------
//
// Jauhar, 2026-08-05: *"this new tools have capabilites to import any kind of text data with
// personalized user data, even its array data such scal"*, scoped the same day to *"can be
// customized based on data, either long, wide, or block"*.
//
// **The layout is DECLARED, never sniffed.** A wide table and a long one are both rectangles of
// numbers; the difference is what the header row MEANS, and there is nothing in the characters to
// say which. Reading a long Pc table as wide would take its column headers for pressures and
// store a capillary-pressure curve made of column indices — a plausible-looking array of
// nonsense. This is the declared-stain rule again.
//
// **What each layout is, exactly:**
//
// * **LONG** — one row per point: a key column (well/depth/sample), an axis column, a value
//   column. That is what `import_core_table` already reads, so a long array is point data and
//   needs nothing here.
// * **WIDE** — one row per SAMPLE, and the HEADER ROW IS THE AXIS: a column per pressure step, a
//   column per T2 bin. The porous-plate delivery, the NMR export, the sieve analysis.
// * **BLOCK** — several tables stacked in one file, each preceded by a repeat of the header. Once
//   the repeated headers are stripped the rows are exactly the file they came from, so BLOCK is a
//   pre-pass over either of the other two rather than a third way of reading a table.
//
// **A block keyed by a LABEL LINE is not read, and says so.** Some per-plug deliveries write
// `PLUG 12  4633.5 ft` on its own line above each table instead of carrying the depth in a
// column. Which token on that line is the depth and which is the plug number cannot be told apart
// without guessing — the workbook reader met exactly this and answered it by requiring a unit — so
// a block whose key is not in a column is REPORTED and left unread rather than attributed to rock
// chosen by a coin toss.

/// One array read out of a wide table: the sample it belongs to, and its values across the axis.
#[derive(Debug, Clone, Serialize)]
pub struct ArrayRow {
    pub well_name: Option<String>,
    pub depth: Option<f64>,
    pub sample_no: Option<i64>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrayProbe {
    /// The axis read off the header row, one entry per array column.
    pub axis: Vec<f32>,
    /// The header TEXT each axis value was read from, so the pane can show what was parsed and
    /// from what — `100 psi` reading as 100 is worth being able to see.
    pub axis_labels: Vec<String>,
    /// Headers that are not numbers and so cannot be axis values. Reported BY NAME: a stray
    /// `TOTAL` column on a porous-plate export would otherwise be stored as a measurement at an
    /// invented pressure, or vanish without a word.
    pub non_axis: Vec<String>,
    pub rows: Vec<ArrayRow>,
    /// Repeated header lines found and stripped (the BLOCK pre-pass).
    pub blocks_joined: usize,
    pub notes: Vec<String>,
}

/// Strips repeated header lines from a stacked (BLOCK) file.
///
/// A block file is one table written several times with its header repeated; once the repeats are
/// gone the rows are the file they came from. Matched on the JOINED CELLS rather than on the raw
/// line, so a delivery that re-exported one block with different spacing is still recognised.
fn join_blocks(table: &mut Vec<Vec<String>>, headers: &[String]) -> usize {
    let key = |r: &[String]| r.iter().map(|c| c.trim().to_uppercase()).collect::<Vec<_>>().join("|");
    let want = key(headers);
    let before = table.len();
    table.retain(|r| key(r) != want);
    before - table.len()
}

/// Splits a delimited file into a table, honouring the same options the long path uses.
fn split_table(path: &str, opts: &TableOptions) -> ParseResult<Vec<Vec<String>>> {
    let text = parsers::read_text_file(path)?;
    let mut lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    if opts.skip_lines > 0 && opts.skip_lines < lines.len() {
        lines.drain(..opts.skip_lines);
    }
    let first = lines.first().copied().unwrap_or("");
    let delim: Option<u8> = match opts.delimiter.as_deref() {
        Some(",") => Some(b','),
        Some(";") => Some(b';'),
        Some("\t") => Some(b'\t'),
        Some("ws") => None,
        _ if first.contains('\t') => Some(b'\t'),
        _ if first.contains(';') => Some(b';'),
        _ if first.contains(',') => Some(b','),
        _ => None,
    };
    let mut table: Vec<Vec<String>> = Vec::new();
    match delim {
        Some(d) => {
            let joined = lines.join("\n");
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(d)
                .has_headers(false)
                .flexible(true)
                .from_reader(joined.as_bytes());
            for rec in rdr.records() {
                table.push(rec?.iter().map(|s| s.trim().to_string()).collect());
            }
        }
        None => {
            for line in &lines {
                table.push(line.split_whitespace().map(str::to_string).collect());
            }
        }
    }
    Ok(table)
}

/// Reads the axis value out of one column header.
///
/// A laboratory routinely writes the unit into the header (`100 psi`, `3.5 ms`, `0.5PSI`), so a
/// trailing unit is stripped before the number is read. Everything else — `TOTAL`, `AVG`, a blank
/// — is not an axis value and the column is dropped by name rather than counted at an invented
/// position.
fn axis_of(header: &str, decimal: Option<&str>) -> Option<f64> {
    let numeric: String = header
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | '+'))
        .collect();
    if numeric.is_empty() {
        return None;
    }
    parse_number(&numeric, decimal).0
}

/// Reads a WIDE table: one row per sample, the header row as the axis.
///
/// `roles` is the same per-column role list the long path uses. WELL, DEPTH and SAMPLE are
/// claimed and everything else is an array bin whose header IS its axis value.
pub fn read_wide(path: &str, opts: &TableOptions, roles: &[String], block: bool) -> ParseResult<ArrayProbe> {
    let mut table = split_table(path, opts)?;
    if table.is_empty() {
        return Err(parsers::ParseError::Las("file is empty".into()));
    }
    let headers: Vec<String> = table.remove(0).iter().map(|h| h.trim().to_string()).collect();
    let mut notes = Vec::new();
    let blocks_joined = if block { join_blocks(&mut table, &headers) } else { 0 };
    if block && blocks_joined == 0 {
        notes.push(
            "BLOCK was chosen but no repeated header was found, so the file was read as one \
             table. A block keyed by a label line above each table rather than by a column is not \
             read: which token on that line is the depth cannot be told from which is the plug \
             number, and guessing would attribute the whole block to the wrong rock."
                .into(),
        );
    }

    let role_at = |i: usize| roles.get(i).map(String::as_str).unwrap_or("");
    let well_col = (0..headers.len()).find(|i| role_at(*i) == "WELL");
    let depth_col = (0..headers.len()).find(|i| role_at(*i) == "DEPTH");
    let sample_col = (0..headers.len()).find(|i| role_at(*i) == "SAMPLE");

    let decimal = opts.decimal.as_deref();
    let (mut axis, mut axis_labels, mut non_axis, mut bins) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (i, h) in headers.iter().enumerate() {
        // A column the user gave a role to is a key or a deliberate skip, never a bin.
        if !role_at(i).is_empty() {
            continue;
        }
        match axis_of(h, decimal) {
            Some(v) => {
                axis.push(v as f32);
                axis_labels.push(h.clone());
                bins.push(i);
            }
            None => non_axis.push(h.clone()),
        }
    }
    if axis.is_empty() {
        return Err(parsers::ParseError::Las(
            "no column header reads as a number, so there is no axis. In a WIDE table each array \
             column's header IS its axis value — the pressure of the step, the T2 of the bin. \
             Check the header row, or the layout."
                .into(),
        ));
    }
    if !non_axis.is_empty() {
        notes.push(format!(
            "{} column(s) dropped, a header that is not a number cannot be an axis value: {}",
            non_axis.len(),
            non_axis.join(", ")
        ));
    }
    // Reported, never sorted. Sorting the axis would have to reorder every row's values with it,
    // and a delivery whose columns run high-to-low is perfectly ordinary — what matters is that
    // the user can see it before a display reads the values in column order.
    if axis.windows(2).any(|w| w[1] <= w[0]) {
        notes.push(
            "The axis does not increase from left to right. Stored exactly as delivered — check \
             the header row reads in the order you expect."
                .into(),
        );
    }

    let mut rows = Vec::new();
    let mut short = 0usize;
    for r in &table {
        let cell = |i: usize| r.get(i).map(String::as_str).unwrap_or("");
        let values: Vec<f32> = bins
            .iter()
            .map(|i| parse_number(cell(*i), decimal).0.map_or(f32::NAN, |v| v as f32))
            .collect();
        // A row with nothing anywhere across the axis is padding, not a sample of nothing.
        if values.iter().all(|v| !v.is_finite()) {
            continue;
        }
        if r.len() < headers.len() {
            short += 1;
        }
        rows.push(ArrayRow {
            well_name: well_col.map(|i| cell(i).to_string()).filter(|s| !s.is_empty()),
            depth: depth_col.and_then(|i| parse_number(cell(i), decimal).0),
            sample_no: sample_col.and_then(|i| parse_number(cell(i), decimal).0).map(|v| v as i64),
            values,
        });
    }
    if short > 0 {
        notes.push(format!(
            "{short} row(s) were shorter than the header row; their missing bins are MISSING rather than zero"
        ));
    }
    if blocks_joined > 0 {
        notes.push(format!("{blocks_joined} repeated header row(s) stripped — the blocks were read as one table"));
    }
    Ok(ArrayProbe { axis, axis_labels, non_axis, rows, blocks_joined, notes })
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArrayCommit {
    pub paths: Vec<String>,
    /// One role per column, in file order — WELL / DEPTH / SAMPLE claim a column, IGNORE drops
    /// one, and everything left is an array bin.
    pub roles: Vec<String>,
    /// `"wide"` or `"block"`. `"long"` never reaches here: a long array is point data and goes
    /// through the ordinary commit.
    pub layout: String,
    #[serde(default)]
    pub opts: TableOptions,
    /// What the array is called — `T2`, `PC_SW`, `GRAINSIZE`.
    pub curve_name: String,
    /// Delivery set. Auto-suffixed per well like every other set, so an import never overwrites.
    #[serde(default)]
    pub set_name: Option<String>,
    #[serde(default)]
    pub depth_unit: Option<String>,
    #[serde(default)]
    pub fallback_well_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrayImportResult {
    pub path: String,
    pub curve: String,
    pub wells: usize,
    pub samples: usize,
    pub bins: usize,
    /// The two ends of the axis, so the pane can show what was read without echoing every bin.
    pub axis_first: f64,
    pub axis_last: f64,
    /// Sets actually written, one per well — the suffixed name where the chosen one was taken.
    pub sets: Vec<String>,
    /// Well names in the file that matched no well, or more than one. Reported, never guessed.
    pub unmatched: Vec<String>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

/// A delivery name free on this well for this curve, suffixed if taken.
///
/// `db::write_array_log` REPLACES its (well, set, curve) rows — right for a Monte Carlo re-run,
/// which must never union two runs' realizations, and wrong for an import, where it would eat the
/// previous delivery. Jauhar, 2026-08-05: *"dont eat it, thats why i request user can define their
/// intake cons, so it wont eat anything"*. Same auto-suffix rule as every other delivery set.
fn free_array_set(conn: &Connection, well_id: &str, curve: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "RAW".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM array_logs
             WHERE well_id = ?1 AND upper(set_name) = ?2 AND upper(curve_name) = upper(?3)",
            duckdb::params![well_id, name, curve],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    };
    if !taken(&base)? {
        return Ok(base);
    }
    for i in 1..1000 {
        let cand = format!("{base}_{i}");
        if !taken(&cand)? {
            return Ok(cand);
        }
    }
    Ok(base)
}

/// Imports one or more WIDE/BLOCK files into the array store.
///
/// Rows route to wells exactly as the long path does — normalized name, exactly one match, and
/// anything else reported rather than guessed — and depths convert to the project unit through
/// the same converter. What is different is only the SHAPE being read, which is the whole point of
/// the layout being a declaration.
pub fn commit_arrays(conn: &Connection, req: &ArrayCommit) -> Vec<ArrayImportResult> {
    let block = req.layout.eq_ignore_ascii_case("block");
    let project_unit = crate::units::project_depth_unit_or_default(conn);
    let file_unit = req.depth_unit.as_deref().and_then(crate::units::DepthUnit::parse).unwrap_or(project_unit);
    let curve = req.curve_name.trim().to_uppercase();

    req.paths
        .iter()
        .map(|path| {
            let mut res = ArrayImportResult {
                path: path.clone(),
                curve: curve.clone(),
                wells: 0,
                samples: 0,
                bins: 0,
                axis_first: f64::NAN,
                axis_last: f64::NAN,
                sets: vec![],
                unmatched: vec![],
                notes: vec![],
                error: None,
            };
            if curve.is_empty() {
                res.error = Some("Name the array before importing it — it is stored under that name.".into());
                return res;
            }
            let probe = match read_wide(path, &req.opts, &req.roles, block) {
                Ok(p) => p,
                Err(e) => {
                    res.error = Some(e.to_string());
                    return res;
                }
            };
            res.bins = probe.axis.len();
            res.axis_first = probe.axis.first().copied().unwrap_or(f32::NAN) as f64;
            res.axis_last = probe.axis.last().copied().unwrap_or(f32::NAN) as f64;
            res.notes = probe.notes.clone();

            // Group by the file's own well column; rows with none fall back to the selected well.
            let mut groups: Vec<(Option<String>, Vec<&ArrayRow>)> = Vec::new();
            for r in &probe.rows {
                let key = r.well_name.as_ref().map(|n| n.trim().to_uppercase());
                match groups.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, list)) => list.push(r),
                    None => groups.push((key, vec![r])),
                }
            }

            for (name, list) in &groups {
                let well_id = match name {
                    Some(n) => {
                        let ids: Vec<String> = match conn
                            .prepare("SELECT well_id FROM wells WHERE upper(trim(well_name)) = ?1 ORDER BY well_id")
                        {
                            Ok(mut stmt) => stmt
                                .query_map(duckdb::params![n], |r| r.get::<_, String>(0))
                                .map(|rows| rows.filter_map(Result::ok).collect())
                                .unwrap_or_default(),
                            Err(_) => vec![],
                        };
                        match ids.len() {
                            1 => ids[0].clone(),
                            // 0 or many: the exactly-one-match rule. A near miss is a different
                            // well, and picking one of two would put a whole delivery on the
                            // wrong rock with nothing to show for it.
                            _ => {
                                res.unmatched.push(n.clone());
                                continue;
                            }
                        }
                    }
                    None => match &req.fallback_well_id {
                        Some(w) => w.clone(),
                        None => {
                            res.unmatched.push("(no well column and no well selected)".into());
                            continue;
                        }
                    },
                };

                let mut depths: Vec<f32> = Vec::new();
                let mut samples: Vec<Vec<f32>> = Vec::new();
                let mut no_depth = 0usize;
                for r in list {
                    let Some(d) = r.depth else {
                        no_depth += 1;
                        continue;
                    };
                    depths.push(d as f32);
                    samples.push(r.values.clone());
                }
                if no_depth > 0 {
                    // An array is stored AT a depth; a sample with none has nowhere to go, and
                    // taking the row above it would attribute a measurement to rock it was not
                    // made on.
                    res.notes.push(format!("{no_depth} sample(s) had no depth and were not stored"));
                }
                if depths.is_empty() {
                    continue;
                }
                crate::units::convert_depths(&mut depths, file_unit, project_unit);

                let set = match free_array_set(conn, &well_id, &curve, req.set_name.as_deref().unwrap_or("RAW")) {
                    Ok(s) => s,
                    Err(e) => {
                        res.notes.push(format!("could not name the delivery: {e}"));
                        continue;
                    }
                };
                match db::write_array_log(conn, &well_id, &set, &curve, &depths, &samples, Some(&probe.axis)) {
                    Ok(written) => {
                        res.wells += 1;
                        res.samples += written;
                        if !res.sets.contains(&set) {
                            res.sets.push(set);
                        }
                    }
                    Err(e) => res.notes.push(format!("{e}")),
                }
            }
            if !res.unmatched.is_empty() {
                res.notes.push(format!(
                    "{} well name(s) matched no well, or more than one, and were skipped: {}",
                    res.unmatched.len(),
                    res.unmatched.join(", ")
                ));
            }
            res
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A comma decimal is read as one number, not two** — the workbook reader's rule, now in
    /// the delimited path where a delivery can mix conventions just as easily.
    ///
    /// This is the failure worth remembering: reading only the dot convention did not fail on
    /// the comma rows, it split `7016,54` and matched `54`. A plausible shallow depth on
    /// entirely the wrong sand.
    #[test]
    fn a_comma_decimal_is_one_number_and_a_grouped_thousand_is_flagged() {
        assert_eq!(parse_number("7016,54", None).0, Some(7016.54));
        assert_eq!(parse_number("6980.71", None).0, Some(6980.71));
        // Both separators: the rightmost is the decimal, under either locale.
        assert_eq!(parse_number("1,234.56", None).0, Some(1234.56));
        assert_eq!(parse_number("1.234,56", None).0, Some(1234.56));
        // Three decimal places must not become a million.
        assert_eq!(parse_number("4633.500", None).0, Some(4633.5));
        // The genuinely ambiguous case is read as a decimal AND reported.
        let (v, amb) = parse_number("1,234", None);
        assert_eq!(v, Some(1.234));
        assert!(amb, "a validly grouped comma must be reported, not silently chosen");
        // Told the convention, it is not ambiguous at all.
        assert_eq!(parse_number("1,234", Some("comma")).0, Some(1.234));
        assert_eq!(parse_number("1,234", Some("dot")).0, Some(1234.0));
    }

    /// A role is claimed ONCE, and everything unclaimed is carried rather than dropped. A lab
    /// export routinely has CPOR and CPOR_CORR; keeping both beats letting the second overwrite
    /// the first or vanish.
    #[test]
    fn an_unclaimed_column_is_carried_as_point_data() {
        let roles: Vec<String> = ["WELL", "DEPTH", "CPOR", "ITEM", "IGNORE", "ITEM"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let m = mapping_from_roles(&roles).expect("mapping");
        assert_eq!(m.well, Some(0));
        assert_eq!(m.depth, 1);
        assert_eq!(m.cpor, Some(2));
        assert_eq!(m.extras, vec![3, 5], "ITEM columns are carried; IGNORE is the only way out");
        assert!(m.cperm.is_none());
    }

    /// Without a depth there is nothing to store, and that is refused by name rather than
    /// defaulting to the first column — which would import a whole delivery at the wrong depths
    /// and look entirely successful.
    #[test]
    fn a_table_with_no_depth_is_refused() {
        let roles: Vec<String> = ["WELL", "CPOR"].iter().map(|s| s.to_string()).collect();
        let err = mapping_from_roles(&roles).expect_err("must refuse");
        assert!(err.contains("DEPTH"), "{err}");
    }

    /// **A cell that cannot be a number in a numeric column is flagged, and an EMPTY one is not.**
    ///
    /// This is what makes the preview a check. An empty cell is a legitimate missing measurement
    /// (the blank-is-not-a-zero rule), and flagging it would bury the real failures under a wall
    /// of colour on a sparse delivery — which is the same as not flagging anything.
    #[test]
    fn only_a_cell_that_should_be_a_number_and_is_not_gets_flagged() {
        let dir = std::env::temp_dir().join("sandibumi-intake-test");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("probe.csv");
        // A stray unit on one depth, an #N/A in the porosity, and blanks that are just missing.
        std::fs::write(
            &path,
            "WELL,DEPTH,CPOR,LITHOLOGY
             SANDI-01,2010.1,0.18,sandstone
             SANDI-01,2011.6 ft,0.17,sandstone
             SANDI-01,2012.1,#N/A,siltstone
             SANDI-01,2013.1,,shale
",
        )
        .expect("write");
        let p = probe(path.to_str().unwrap(), &TableOptions::default()).expect("probe");

        let headers: Vec<&str> = p.columns.iter().map(|c| c.header.as_str()).collect();
        assert_eq!(headers, ["WELL", "DEPTH", "CPOR", "LITHOLOGY"]);
        assert_eq!(p.n_rows, 4);

        // Row 1 col 1 is "2011.6 ft"; row 2 col 2 is "#N/A". The blank at row 3 is NOT flagged.
        assert!(p.preview_bad.contains(&(1, 1)), "a depth with a unit stuck on it: {:?}", p.preview_bad);
        assert!(p.preview_bad.contains(&(2, 2)), "a spreadsheet #N/A: {:?}", p.preview_bad);
        assert_eq!(
            p.preview_bad.len(),
            2,
            "an empty cell is a missing measurement, not a parse failure: {:?}",
            p.preview_bad
        );
        let _ = std::fs::remove_file(&path);
    }

    /// The role proposal reads a header the way a petrophysicist would, and says WHY — a guess
    /// nobody can argue with is a guess that gets accepted.
    #[test]
    fn a_role_proposal_explains_itself() {
        let (role, why) = guess_role("HELIUM POROSITY", "number", &[]);
        assert_eq!(role, "CPOR");
        assert!(why.contains("HELIUM POROSITY"), "{why}");

        // Already claimed → the second candidate is carried rather than overwriting the first.
        let (role, _) = guess_role("POROSITY", "number", &["CPOR".to_string()]);
        assert_eq!(role, "ITEM");

        // An unknown numeric column is point data, not a dropped column.
        assert_eq!(guess_role("KV_KH", "number", &[]).0, "ITEM");
        // An empty column is proposed for IGNORE — it has nothing to carry.
        assert_eq!(guess_role("NOTES", "empty", &[]).0, "IGNORE");
    }

    /// **A WIDE table's header row IS the axis, and a header that is not a number is dropped by
    /// name.** The porous-plate SCAL delivery Jauhar named: one row per plug, one column per
    /// pressure step.
    ///
    /// The `TOTAL` column is the part that matters. Counted as a bin it would be a saturation
    /// stored at an invented pressure — plausible, and at the end of the curve where a Thomeer or
    /// Leverett fit is most sensitive. Dropped silently it would be a column the user delivered
    /// and never sees again. Named, it is neither.
    #[test]
    fn a_wide_table_reads_its_header_row_as_the_axis() {
        let path = std::env::temp_dir().join("sandi_wide_pc.csv");
        std::fs::write(
            &path,
            "WELL,DEPTH,0.5,1,2,4,8,TOTAL\n\
             SANDI-W1,2000.0,1.00,0.92,0.71,0.55,0.44,1.0\n\
             SANDI-W1,2010.0,1.00,0.88,0.64,0.48,0.39,1.0\n",
        )
        .unwrap();
        let roles = vec!["WELL".to_string(), "DEPTH".to_string()];
        let probe = read_wide(path.to_str().unwrap(), &TableOptions::default(), &roles, false).expect("read");

        assert_eq!(probe.axis, vec![0.5, 1.0, 2.0, 4.0, 8.0], "the pressures come off the header row");
        assert_eq!(probe.non_axis, vec!["TOTAL".to_string()], "and a non-numeric header is named, not counted");
        assert!(probe.notes.iter().any(|n| n.contains("TOTAL")), "the drop is reported: {:?}", probe.notes);
        assert_eq!(probe.rows.len(), 2);
        assert_eq!(probe.rows[0].depth, Some(2000.0));
        assert_eq!(probe.rows[0].values.len(), 5, "one value per bin, TOTAL excluded");
        assert!((probe.rows[1].values[4] - 0.39).abs() < 1e-6);
    }

    /// **A unit written into the header is stripped before the number is read.** A laboratory
    /// writes `100 psi`, not `100` — and reading that column as non-numeric would drop the whole
    /// delivery one bin at a time, reporting it as "no axis".
    #[test]
    fn an_axis_value_survives_the_unit_its_laboratory_wrote_beside_it() {
        assert_eq!(axis_of("100 psi", None), Some(100.0));
        assert_eq!(axis_of("0.5PSI", None), Some(0.5));
        assert_eq!(axis_of("3.5 ms", None), Some(3.5));
        assert_eq!(axis_of("TOTAL", None), None);
        assert_eq!(axis_of("", None), None);
        // A unit FIRST is not an axis value: "psi100" says the header is a label, and reading a
        // number out of the middle of it would be a guess.
        assert_eq!(axis_of("psi100", None), None);
    }

    /// **BLOCK is stacked tables, and the repeated headers are stripped rather than read as
    /// data.** Left in, each repeat becomes a row whose every bin fails to parse — which the
    /// padding rule then drops silently, so the file would import looking complete while the
    /// blocks were never actually joined.
    ///
    /// And the honest limit, asserted rather than described: a block keyed by a LABEL LINE instead
    /// of a column is not read, and the run says so.
    #[test]
    fn block_joins_stacked_tables_and_says_when_the_key_is_not_in_a_column() {
        let path = std::env::temp_dir().join("sandi_block_pc.csv");
        std::fs::write(
            &path,
            "DEPTH,1,2,4\n\
             2000.0,0.9,0.7,0.5\n\
             DEPTH,1,2,4\n\
             2010.0,0.8,0.6,0.4\n",
        )
        .unwrap();
        let roles = vec!["DEPTH".to_string()];
        let probe = read_wide(path.to_str().unwrap(), &TableOptions::default(), &roles, true).expect("read");
        assert_eq!(probe.blocks_joined, 1, "the repeated header is stripped");
        assert_eq!(probe.rows.len(), 2, "and both blocks' rows survive");
        assert_eq!(probe.rows[1].depth, Some(2010.0));

        // The control, and it is worse than "a block goes missing". Read WITHOUT the block flag,
        // the repeated header becomes a REAL-LOOKING SAMPLE: its cells under the bin columns are
        // the axis numbers themselves, so they parse, and the row survives carrying saturations of
        // 1, 2 and 4. Only its missing depth stops it being stored, and that is luck rather than a
        // guard — a delivery whose repeated header sat under a DEPTH column would import it.
        let plain = read_wide(path.to_str().unwrap(), &TableOptions::default(), &roles, false).expect("read");
        assert_eq!(plain.blocks_joined, 0);
        assert_eq!(plain.rows.len(), 3, "the repeated header survives as a row of nonsense");
        let bogus = plain.rows.iter().find(|r| r.depth.is_none()).expect("and it is the one with no depth");
        assert_eq!(bogus.values, vec![1.0, 2.0, 4.0], "its values are the axis, read as measurements");

        // A file with no repeated header, imported as BLOCK, says the key may be in a label line.
        let single = std::env::temp_dir().join("sandi_block_single.csv");
        std::fs::write(&single, "DEPTH,1,2\n2000.0,0.9,0.7\n").unwrap();
        let one = read_wide(single.to_str().unwrap(), &TableOptions::default(), &roles, true).expect("read");
        assert!(
            one.notes.iter().any(|n| n.contains("label line")),
            "the limit must be stated rather than silently producing one block: {:?}",
            one.notes
        );
    }

    /// **An import never eats a delivery already there.** Jauhar, 2026-08-05: *"dont eat it, thats
    /// why i request user can define their intake cons, so it wont eat anything"*.
    ///
    /// `db::write_array_log` REPLACES its (well, set, curve) rows, which is right for a Monte
    /// Carlo re-run — two runs' realizations must never be unioned into one distribution — and
    /// wrong for an import, where it would silently discard the previous delivery. So an array
    /// import resolves a FREE set name per well exactly as core, SCAL, aux and image deliveries
    /// do, and the second import of the same name lands beside the first.
    #[test]
    fn a_second_delivery_lands_beside_the_first_instead_of_replacing_it() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-ARR", None, None, None).unwrap();
        let w = wid.to_string();

        let path = std::env::temp_dir().join("sandi_arr_delivery.csv");
        std::fs::write(&path, "DEPTH,1,2,4\n2000.0,0.9,0.7,0.5\n2001.0,0.8,0.6,0.4\n").unwrap();
        let req = ArrayCommit {
            paths: vec![path.to_str().unwrap().to_string()],
            roles: vec!["DEPTH".into()],
            layout: "wide".into(),
            opts: TableOptions::default(),
            curve_name: "PC_SW".into(),
            set_name: Some("LAB".into()),
            depth_unit: None,
            fallback_well_id: Some(w.clone()),
        };
        let first = commit_arrays(&conn, &req);
        assert!(first[0].error.is_none(), "{:?}", first[0].error);
        assert_eq!(first[0].samples, 2);
        assert_eq!(first[0].bins, 3);
        assert_eq!(first[0].sets, vec!["LAB".to_string()]);

        let second = commit_arrays(&conn, &req);
        assert_eq!(second[0].sets, vec!["LAB_1".to_string()], "the same name must not overwrite");

        // Both deliveries are still there.
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT set_name) FROM array_logs WHERE well_id = ?1 AND curve_name = 'PC_SW'",
                duckdb::params![&w],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2, "the first delivery survives the second");

        // And the axis came back with the values, or the array is a list of numbers about nothing.
        let axis: Vec<u8> = conn
            .query_row(
                "SELECT axis FROM array_logs WHERE well_id = ?1 AND set_name = 'LAB' LIMIT 1",
                duckdb::params![&w],
                |r| r.get(0),
            )
            .unwrap();
        let decoded: Vec<f32> = axis.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        assert_eq!(decoded, vec![1.0, 2.0, 4.0], "the axis is stored with the values");
    }
}
