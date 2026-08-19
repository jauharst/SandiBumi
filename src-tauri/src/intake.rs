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
    /// `CURVE` `IGNORE`. A proposal, never applied — the pane shows it and the user overrules.
    ///
    /// `CURVE` is never PROPOSED, only chosen: a column of numbers at depths is a plug measurement
    /// or a logged curve depending on how the file was sampled, and nothing in the numbers says
    /// which. Storing a log as point data hides it from every module; storing plugs as a log
    /// invents a continuous measurement between them.
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
    pub format: FormatDetection,
    pub text_encoding: String,
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

/// Content-owned format choice. Extensions are retained only to report a disagreement;
/// they never select the reader.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FormatDetection {
    pub detected_format: String,
    pub recognition: String,
    pub choice_report: String,
    pub extension_disagreement: Option<String>,
}

fn byte_slice_contains(bytes: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && bytes.windows(needle.len()).any(|window| window == needle)
}

fn extension_of(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn detection(
    path: &str,
    detected_format: &str,
    recognition: &str,
    choice_report: String,
    expected_extensions: &[&str],
) -> FormatDetection {
    let extension = extension_of(path);
    let extension_disagreement = (!extension.is_empty()
        && !expected_extensions.iter().any(|expected| extension.eq_ignore_ascii_case(expected)))
        .then(|| {
            format!(
                "extension .{extension} disagrees with content; {detected_format} was chosen by {recognition}"
            )
        });
    FormatDetection {
        detected_format: detected_format.to_string(),
        recognition: recognition.to_string(),
        choice_report,
        extension_disagreement,
    }
}

/// Recognises a file from the cited signatures in chapter §2.9/D-31. Raw bytes are used
/// only for signature inspection; every text interpretation still goes through
/// `parsers::read_text_file`.
pub fn detect_format(path: &str) -> ParseResult<FormatDetection> {
    let bytes = std::fs::read(path)?;
    if bytes.starts_with(&[0x09, 0x08, 0x06, 0x00]) {
        let mut found = detection(
            path,
            "BIFF5 workbook stream",
            "09 08 06 00 signature",
            "09 08 06 00 identifies a headerless BIFF5 workbook stream".into(),
            &["xls"],
        );
        if extension_of(path) == "xls" {
            found.extension_disagreement = Some(
                "extension .xls names the Excel family but not its version; signature chose the headerless BIFF5 stream"
                    .into(),
            );
        }
        return Ok(found);
    }
    if bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        return Ok(detection(
            path,
            "OLE2 compound document",
            "D0 CF 11 E0 signature",
            "D0 CF 11 E0 identifies an OLE2 container; its directory structure must choose the contained format"
                .into(),
            &["xls", "doc", "ppt"],
        ));
    }
    if bytes.starts_with(&[0x50, 0x4B]) {
        let xlsx = byte_slice_contains(&bytes, b"[Content_Types].xml")
            && byte_slice_contains(&bytes, b"xl/workbook.xml");
        return Ok(if xlsx {
            detection(
                path,
                "XLSX workbook",
                "PK ZIP signature plus workbook structure",
                "PK is shared by ZIP-based formats; [Content_Types].xml and xl/workbook.xml chose XLSX"
                    .into(),
                &["xlsx", "xlsm"],
            )
        } else {
            detection(
                path,
                "ZIP container",
                "PK ZIP signature plus archive structure",
                "PK is shared by ZIP-based formats; no XLSX workbook structure was present, so generic ZIP was chosen"
                    .into(),
                &["zip"],
            )
        });
    }
    if bytes.starts_with(&[0x05, 0xB4]) {
        return Ok(detection(
            path,
            "SDC Geo Suite ODF",
            "05 B4 nibble-swapped ZIP signature",
            "05 B4 identifies the nibble-swapped ZIP signature used by SDC Geo Suite ODF".into(),
            &["odf"],
        ));
    }

    let text = parsers::read_text_file(path)?;
    let first = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("");
    if first.starts_with("*HEADER") {
        return Ok(detection(
            path,
            "Geolog dump",
            "first-line *HEADER signature",
            "first non-empty line begins *HEADER; Geolog dump was chosen".into(),
            &["dat", "unl"],
        ));
    }
    if first.starts_with('~')
        && first
            .trim_start_matches('~')
            .trim_start()
            .to_ascii_uppercase()
            .starts_with('V')
    {
        return Ok(detection(
            path,
            "LAS text",
            "leading ~Version section structure",
            "first non-empty section is ~Version; LAS was chosen".into(),
            &["las"],
        ));
    }
    if first.contains(',') || first.contains(';') || first.contains('\t') {
        return Ok(detection(
            path,
            "delimited text",
            "first-row delimiter structure",
            "the first non-empty row has delimited-table structure; delimited text was chosen".into(),
            &["csv", "txt", "tsv", "dat"],
        ));
    }
    Ok(detection(
        path,
        "plain text",
        "readable text without a stronger signature",
        "no stronger signature or delimited structure was present; plain text was chosen".into(),
        &["txt", "asc"],
    ))
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
    let format = detect_format(path)?;
    let decoded = parsers::read_text_file_with_encoding(path)?;
    let text_encoding = decoded.encoding;
    let text = decoded.text;
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
            format,
            text_encoding,
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

    let mut notes = vec![
        format.choice_report.clone(),
        format!("Text encoding detected: {text_encoding}."),
    ];
    if let Some(disagreement) = &format.extension_disagreement {
        notes.push(disagreement.clone());
    }
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
        format,
        text_encoding,
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
    /// SB-DBM-031: the datum the delivery's depths are quoted in, declared by the user in
    /// the pane. Serde-defaulted to empty so an old payload REFUSES at the vocabulary
    /// check rather than silently declaring MD.
    #[serde(default)]
    pub depth_datum: String,
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
// **A block keyed by a LABEL LINE is read by the workbook reader's rule.** Some per-plug
// deliveries write `PLUG 12  4633.5 ft` on its own line above each table instead of carrying the
// depth in a column. Which token on that line is the depth and which is the plug number cannot be
// told apart from the numbers alone — so it is not guessed at: the depth is the number that
// carries a UNIT, exactly as `images::WORKBOOK_RUNNER` reads a plate sheet's header cell, and for
// exactly the same reason. A label line whose numbers carry no unit is still REPORTED and left
// unread rather than attributed to rock chosen by a coin toss.

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
    /// Depths carrying more than one sample. Sent as DATA and not only named in a note, because a
    /// warning saying `4633.50` is actionable only if the rows it means can be found — the preview
    /// marks them.
    pub clashes: Vec<DepthClash>,
    pub notes: Vec<String>,
}

/// One depth that more than one sample landed on. `well` is the file's own well name where it has
/// a WELL column, and `None` where the whole file falls back to the selected well — the same key
/// the clash is detected under, so the pane marks exactly the rows the check counted.
#[derive(Debug, Clone, Serialize)]
pub struct DepthClash {
    pub well: Option<String>,
    pub depth: f64,
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

/// Depth units a label line may carry. Deliberately NOT bare `F`: on a line that also names a
/// plug and a facies code, one letter is not evidence of anything.
const LABEL_DEPTH_UNITS: [&str; 8] = ["FT", "FEET", "FOOT", "M", "MTR", "METER", "METRE", "METERS"];

/// Reads the depth out of a label line — the number that carries a UNIT, and no other.
///
/// **The workbook reader's rule, borrowed whole** (`images::WORKBOOK_RUNNER`). A label line reads
/// `PLUG 12  4633.5 ft` or `SAMPLE 7 / 2103,40 M (CORE)`, and it carries at least two numbers: the
/// plug number and the depth. Nothing in the numbers themselves says which is which — taking the
/// first would read plug 12 as a depth of 12 ft, and taking the largest would fail the moment a
/// laboratory numbered its plugs into the thousands. The unit is the only thing on the line that
/// identifies a depth AS a depth, and a delivery that omits it has genuinely not said where the
/// rock is.
///
/// Returns the depth and how many DIFFERENT numbers carried a unit. **A caption names ONE plug and
/// a plug sits at one depth** (Jauhar, 2026-08-05: *"it should be 1 plug number only, should warn
/// user if duplicate"*), so a second depth is not a range to pick an end from — it is a duplicate,
/// and the caller reports it rather than resolving it. The first is used so the block still imports
/// and can be checked against the delivery; discarding it would lose a block over a caption a
/// laboratory very likely typed twice.
///
/// The decimal convention is `parse_number`'s, so a European label line reads the same way a
/// European data row does — the comma-decimal lesson from the plate workbooks, where a seventh of
/// one delivery was stored at 54 feet on rock cored at 7,000.
/// `sep` is the delimiter the file was split ON, and rejoining with it rather than with a space is
/// load-bearing: in a comma-delimited file `4640,0 ft` arrives as TWO cells, and joining them with
/// a space offers the tokenizer `4640 0 ft` — where `0` is the number carrying the unit, and the
/// block is filed at zero feet. The plate workbooks' comma-decimal failure exactly, which put a
/// seventh of one delivery at 54 feet on rock cored at 7,000. Reassembled as written, the
/// comma-decimal rule in `parse_number` reads it as the one number it always was.
fn depth_from_label(cells: &[String], decimal: Option<&str>, sep: char) -> (Option<f64>, usize) {
    let line = cells.join(&sep.to_string());
    let toks: Vec<&str> = line
        .split(|c: char| c.is_whitespace() || matches!(c, '/' | '|' | '(' | ')' | '[' | ']' | ':'))
        .filter(|t| !t.is_empty())
        .collect();
    // A unit is a WORD. Trimming non-letters off both ends instead would make `2103.4M` read as
    // the unit `M`, so the plug number before it would be taken as the depth — which is the one
    // mistake this whole rule exists to prevent.
    let is_unit = |t: &str| {
        let u = t.trim_end_matches(|c: char| !c.is_ascii_alphanumeric()).to_uppercase();
        u.chars().all(|c| c.is_ascii_alphabetic()) && LABEL_DEPTH_UNITS.contains(&u.as_str())
    };
    let mut found: Vec<f64> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        // `4633.5FT` — the unit is glued to the number. Split at the first letter.
        let split = t.find(|c: char| c.is_ascii_alphabetic());
        let (num, glued) = match split {
            Some(p) if p > 0 => (&t[..p], Some(&t[p..])),
            _ => (*t, None),
        };
        let Some(v) = parse_number(num, decimal).0 else { continue };
        let carried = match glued {
            Some(u) => is_unit(u),
            None => toks.get(i + 1).map(|n| is_unit(n)).unwrap_or(false),
        };
        if carried && !found.iter().any(|f| (*f - v).abs() < 1e-9) {
            found.push(v);
        }
    }
    (found.first().copied(), found.len())
}

/// Strips label lines from a stacked file and returns the depth each surviving row belongs to.
///
/// A label line is a row that is NOT a data row: too few of the table's own columns parse as
/// numbers for it to be a sample. Testing what a row PARSES as rather than how long it is matters
/// because a label line written into a delimited file usually keeps the delimiters, so it arrives
/// the full width of the table with the words in the first cell or two.
///
/// Rows before the first label line carry `None`: they belong to no block, and inventing a depth
/// for them from the block BELOW would attribute a header remnant to the first plug.
fn read_label_keys(
    table: &mut Vec<Vec<String>>,
    bins: &[usize],
    decimal: Option<&str>,
    sep: char,
) -> LabelKeys {
    let numeric_fraction = |r: &[String]| -> f32 {
        if bins.is_empty() {
            return 0.0;
        }
        let n = bins.iter().filter(|i| r.get(**i).and_then(|c| parse_number(c, decimal).0).is_some()).count();
        n as f32 / bins.len() as f32
    };
    let mut keys: Vec<Option<f64>> = Vec::with_capacity(table.len());
    let mut kept: Vec<Vec<String>> = Vec::with_capacity(table.len());
    let (mut labels, mut ranged) = (0usize, 0usize);
    let (mut seen, mut repeated): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    let mut current: Option<f64> = None;
    for r in table.drain(..) {
        // Half the axis columns reading as numbers is a sample; a line of words with one depth on
        // it is not. The threshold is deliberately loose — a block's last row is often ragged.
        if numeric_fraction(&r) < 0.5 {
            let (d, n) = depth_from_label(&r, decimal, sep);
            if let Some(d) = d {
                current = Some(d);
                labels += 1;
                if n > 1 {
                    ranged += 1;
                }
                // A depth that keys a SECOND block is two captions claiming one plug. Reported
                // once however many times it recurs — the user needs the depth to go and look at,
                // not a count of how often the laboratory repeated it.
                if seen.iter().any(|s| (*s - d).abs() < 1e-9) {
                    if !repeated.iter().any(|s| (*s - d).abs() < 1e-9) {
                        repeated.push(d);
                    }
                } else {
                    seen.push(d);
                }
            }
            // A non-data line is dropped whether or not it yielded a depth: it is a caption, a
            // rule, or a blank — never a sample of nothing.
            continue;
        }
        keys.push(current);
        kept.push(r);
    }
    *table = kept;
    LabelKeys { keys, labels, ranged, repeated }
}

/// What the captions of a stacked file said, beyond the per-row keys themselves.
struct LabelKeys {
    /// The depth each surviving row belongs to; `None` above the first caption.
    keys: Vec<Option<f64>>,
    /// Captions that yielded a depth.
    labels: usize,
    /// Captions naming MORE than one depth — a duplicate, not a range. See `depth_from_label`.
    ranged: usize,
    /// Depths that keyed more than one block, first occurrence order.
    repeated: Vec<f64>,
}

/// Splits a delimited file into a table, honouring the same options the long path uses.
///
/// Returns the separator alongside the table because a caption line has to be reassembled AS
/// WRITTEN before a number can be read out of it — see `depth_from_label`.
fn split_table(path: &str, opts: &TableOptions) -> ParseResult<(Vec<Vec<String>>, char)> {
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
    Ok((table, delim.map_or(' ', |d| d as char)))
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
    let (mut table, sep) = split_table(path, opts)?;
    if table.is_empty() {
        return Err(parsers::ParseError::Las("file is empty".into()));
    }
    let headers: Vec<String> = table.remove(0).iter().map(|h| h.trim().to_string()).collect();
    let mut notes = Vec::new();
    let blocks_joined = if block { join_blocks(&mut table, &headers) } else { 0 };

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

    // The label-line key. Only reached where BLOCK was asked for and no repeated header was found,
    // because a file that repeats its header has already said where each block starts and reading
    // captions as well would be a second, weaker answer to a question already settled.
    let mut label_keys: Vec<Option<f64>> = Vec::new();
    // Hoisted: the row-level clash check below skips depths already explained here, so a file with
    // two blocks at one depth gets ONE message naming the cause rather than two describing it from
    // both ends.
    let mut caption_dupes: Vec<f64> = Vec::new();
    if block && blocks_joined == 0 {
        let found = read_label_keys(&mut table, &bins, decimal, sep);
        let (labels, ranged) = (found.labels, found.ranged);
        label_keys = found.keys;
        caption_dupes = found.repeated;
        if labels == 0 {
            notes.push(
                "BLOCK was chosen but no repeated header was found and no line above a block \
                 carries a depth with a UNIT, so the file was read as one table. A depth is only \
                 taken from a label line when it says `4633.5 ft` or `2103,4 m`: on a line that \
                 also carries a plug number, nothing but the unit tells the two apart, and \
                 guessing would attribute a whole block to the wrong rock."
                    .into(),
            );
        } else {
            let orphan = label_keys.iter().filter(|k| k.is_none()).count();
            notes.push(format!(
                "{labels} block(s) keyed by a label line — the depth is the number carrying a \
                 unit, the same rule a plate workbook's header cell is read by."
            ));
            if ranged > 0 {
                notes.push(format!(
                    "{ranged} label line(s) name more than one depth. A caption keys ONE plug and \
                     a plug sits at one depth, so the second is a duplicate rather than an \
                     interval to choose an end of — the FIRST is used and the rest ignored. Check \
                     those captions against the delivery."
                ));
            }
            if !caption_dupes.is_empty() {
                let list =
                    caption_dupes.iter().map(|d| format!("{d:.2}")).collect::<Vec<_>>().join(", ");
                notes.push(format!(
                    "{} depth(s) key more than one block: {list}. Two blocks at one depth are two \
                     measurements of the same plug, and a stored array holds ONE vector per depth \
                     — they cannot both be kept. Correct the captions, or split the file, before \
                     importing.",
                    caption_dupes.len()
                ));
            }
            if orphan > 0 {
                notes.push(format!(
                    "{orphan} row(s) sit above the first label line and have no depth. They are \
                     kept and stored without one rather than being given the depth of the block \
                     BELOW them, which would attribute a stray row to the first plug."
                ));
            }
        }
    }

    let mut rows = Vec::new();
    let mut short = 0usize;
    for (ri, r) in table.iter().enumerate() {
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
            // A DEPTH column wins over the block's label line. The column is per SAMPLE and the
            // label is per BLOCK, so where a file carries both the column is the more specific
            // statement; the label fills in only where the column is absent or blank.
            depth: depth_col
                .and_then(|i| parse_number(cell(i), decimal).0)
                .or_else(|| label_keys.get(ri).copied().flatten()),
            sample_no: sample_col.and_then(|i| parse_number(cell(i), decimal).0).map(|v| v as i64),
            values,
        });
    }
    if short > 0 {
        notes.push(format!(
            "{short} row(s) were shorter than the header row; their missing bins are MISSING rather than zero"
        ));
    }

    // **A plug sits at one depth, so two samples cannot share one.** `array_logs` is keyed
    // (well, set, curve, depth) and holds ONE vector per depth, so a second sample at the same
    // depth is a primary-key collision that fails the whole curve's write — with a raw engine
    // message naming nothing the user would recognise. Named here instead, with the depth to go
    // and look at.
    //
    // Reached BEFORE the write in both directions: `intake_probe_arrays` runs this same function
    // for the preview, so the pane names the duplicate ahead of the import, and the import result
    // repeats it for anyone who ran straight past.
    //
    // Counted over ROWS rather than captions so it catches the case a caption check cannot see:
    // one block carrying several rows. That is the same collision reached from inside a single
    // caption instead of across two, and it is the likelier delivery mistake of the pair.
    //
    // Grouped by the file's own well column, because two WELLS sampled at the same depth is
    // entirely ordinary — a check that ignored the well would fire on every multi-well delivery.
    let mut clashes: Vec<(Option<String>, f64)> = Vec::new();
    let mut seen: Vec<(Option<String>, f64)> = Vec::new();
    for r in &rows {
        let Some(d) = r.depth else { continue };
        // Already explained, by name, as two captions claiming one plug.
        if caption_dupes.iter().any(|c| (*c - d).abs() < 1e-9) {
            continue;
        }
        let key = r.well_name.as_ref().map(|n| n.trim().to_uppercase());
        let hit = |v: &[(Option<String>, f64)]| {
            v.iter().any(|(w, s)| *w == key && (*s - d).abs() < 1e-9)
        };
        let (in_seen, in_clash) = (hit(&seen), hit(&clashes));
        if in_seen {
            if !in_clash {
                clashes.push((key, d));
            }
        } else {
            seen.push((key, d));
        }
    }
    if !clashes.is_empty() {
        let list = clashes
            .iter()
            .map(|(w, d)| match w {
                Some(w) => format!("{w} {d:.2}"),
                None => format!("{d:.2}"),
            })
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(format!(
            "{} depth(s) carry more than one sample: {list}. A plug sits at one depth and only \
             one measurement can be stored there, so the rest would be refused. Give them their \
             own depths, or split the file, before importing.",
            clashes.len()
        ));
    }
    if blocks_joined > 0 {
        notes.push(format!("{blocks_joined} repeated header row(s) stripped — the blocks were read as one table"));
    }
    // Both kinds travel as one list for MARKING, while the notes keep them apart for READING: the
    // pane has to highlight every row the store would refuse, and a row does not care which of the
    // two ways its depth came to be duplicated.
    let clash_data: Vec<DepthClash> = caption_dupes
        .iter()
        .map(|d| DepthClash { well: None, depth: *d })
        .chain(clashes.into_iter().map(|(well, depth)| DepthClash { well, depth }))
        .collect();
    Ok(ArrayProbe { axis, axis_labels, non_axis, rows, blocks_joined, clashes: clash_data, notes })
}

/// How many samples an ARRAY preview draws — deliberately far fewer than the long path's
/// `PREVIEW_ROWS`, because a row here is not a row there. A long row is a handful of cells; a wide
/// row is the sample's whole distribution, so an NMR export is a hundred bins per row and thousands
/// of rows. A preview is a CHECK — enough to see that the depths resolved the way the delivery
/// reads — and sending the file to draw a dozen visible lines makes the preview cost more than the
/// import it is meant to precede.
const ARRAY_PREVIEW_ROWS: usize = 40;

/// The wide/block probe as the pane needs it: every judgement made over the WHOLE file, and only a
/// readable slice of the samples carried back.
#[derive(Debug, Clone, Serialize)]
pub struct ArrayPreview {
    pub axis: Vec<f32>,
    pub axis_labels: Vec<String>,
    pub non_axis: Vec<String>,
    pub blocks_joined: usize,
    pub clashes: Vec<DepthClash>,
    pub notes: Vec<String>,
    /// Samples the file holds, counted over ALL of them. A preview reporting its own capped length
    /// would tell the user a 4,000-sample delivery held 40.
    pub n_rows: usize,
    /// The samples drawn, and the index each one sits at in the file — so a row pulled in from
    /// beyond the cap can say where it came from instead of appearing to follow row 40.
    pub rows: Vec<ArrayRow>,
    pub row_index: Vec<usize>,
}

/// Reads a WIDE or BLOCK table for the PANE, without writing anything.
///
/// The same `read_wide` the commit runs, so the preview cannot disagree with the import about what
/// the file says — the standing one-implementation rule. What differs is only how much comes back.
pub fn probe_arrays(
    path: &str,
    opts: &TableOptions,
    roles: &[String],
    block: bool,
) -> ParseResult<ArrayPreview> {
    let p = read_wide(path, opts, roles, block)?;
    let n_rows = p.rows.len();
    let clashing = |r: &ArrayRow| -> bool {
        let Some(d) = r.depth else { return false };
        let key = r.well_name.as_ref().map(|n| n.trim().to_uppercase());
        p.clashes
            .iter()
            .any(|c| (c.depth - d).abs() < 1e-9 && (c.well.is_none() || c.well == key))
    };
    // The first slice, PLUS any duplicated sample that falls beyond it. A preview whose whole
    // purpose is to show the duplicate must not stop just short of it — and a delivery big enough
    // to overflow the cap is exactly the one nobody scrolls through by hand. Capped again so a
    // file that is duplicated throughout cannot send itself back one row at a time.
    let mut row_index: Vec<usize> = (0..n_rows.min(ARRAY_PREVIEW_ROWS)).collect();
    for (i, r) in p.rows.iter().enumerate().skip(ARRAY_PREVIEW_ROWS) {
        if row_index.len() >= ARRAY_PREVIEW_ROWS * 2 {
            break;
        }
        if clashing(r) {
            row_index.push(i);
        }
    }
    let rows = row_index.iter().map(|i| p.rows[*i].clone()).collect();
    Ok(ArrayPreview {
        axis: p.axis,
        axis_labels: p.axis_labels,
        non_axis: p.non_axis,
        blocks_joined: p.blocks_joined,
        clashes: p.clashes,
        notes: p.notes,
        n_rows,
        rows,
        row_index,
    })
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
    let curve = req.curve_name.trim().to_uppercase();
    let project_unit = match crate::units::require_project_depth_unit(conn, "array import") {
        Ok(unit) => unit,
        Err(error) => {
            return req
                .paths
                .iter()
                .map(|path| ArrayImportResult {
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
                    error: Some(error.clone()),
                })
                .collect();
        }
    };
    let file_unit = req
        .depth_unit
        .as_deref()
        .and_then(crate::units::DepthUnit::parse)
        .unwrap_or(project_unit);

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

#[derive(Debug, Clone, Deserialize)]
pub struct CurveCommit {
    pub paths: Vec<String>,
    /// One role per column. `CURVE` marks a continuous log; WELL and DEPTH claim their columns.
    pub roles: Vec<String>,
    #[serde(default)]
    pub opts: TableOptions,
    /// Delivery set name, auto-suffixed per well so an import never overwrites.
    #[serde(default)]
    pub set_name: Option<String>,
    #[serde(default)]
    pub depth_unit: Option<String>,
    #[serde(default)]
    pub fallback_well_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveImportResult {
    pub path: String,
    pub wells: usize,
    pub curves: Vec<String>,
    pub samples: usize,
    pub sets: Vec<String>,
    pub unmatched: Vec<String>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

/// Imports columns marked `CURVE` as continuous logs into the generic curve store.
///
/// The route a delimited file of logs had no way in by: Import LAS reads LAS, and everything else
/// this pane produces is point data. A column of GR every 15 cm is not a plug measurement, and
/// storing it in `aux_data` would make it invisible to every module, plot and export — the
/// `standard_curves` shadowing failure by another road.
///
/// **The curve store, not `standard_curves`.** A delivered mnemonic keeps its own name and its own
/// delivery set, which is the whole import-set model: set RAW has absolute priority in
/// `fetch_generic_curve_aligned` and an attached set fills only the mnemonics RAW lacks, so a
/// second opinion on GR never silently replaces the first.
///
/// **The unit is whatever the file's units row said, kept verbatim.** `curves::normalize_unit`
/// canonicalizes it downstream; inventing one here would state a measurement the delivery did not.
pub fn commit_curves(conn: &Connection, req: &CurveCommit) -> Vec<CurveImportResult> {
    let project_unit = match crate::units::require_project_depth_unit(conn, "curve-table import") {
        Ok(unit) => unit,
        Err(error) => {
            return req
                .paths
                .iter()
                .map(|path| CurveImportResult {
                    path: path.clone(),
                    wells: 0,
                    curves: vec![],
                    samples: 0,
                    sets: vec![],
                    unmatched: vec![],
                    notes: vec![],
                    error: Some(error.clone()),
                })
                .collect();
        }
    };
    let file_unit = req
        .depth_unit
        .as_deref()
        .and_then(crate::units::DepthUnit::parse)
        .unwrap_or(project_unit);

    req.paths
        .iter()
        .map(|path| {
            let mut res = CurveImportResult {
                path: path.clone(),
                wells: 0,
                curves: vec![],
                samples: 0,
                sets: vec![],
                unmatched: vec![],
                notes: vec![],
                error: None,
            };
            let probe = match probe(path, &req.opts) {
                Ok(p) => p,
                Err(e) => {
                    res.error = Some(e.to_string());
                    return res;
                }
            };
            let role_at = |i: usize| req.roles.get(i).map(String::as_str).unwrap_or("");
            let depth_col = (0..probe.columns.len()).find(|i| role_at(*i) == "DEPTH");
            let Some(depth_col) = depth_col else {
                res.error = Some(
                    "No column is marked DEPTH. A log is a measurement AT a depth, so there is \
                     nothing to store without one."
                        .into(),
                );
                return res;
            };
            let well_col = (0..probe.columns.len()).find(|i| role_at(*i) == "WELL");
            let curve_cols: Vec<usize> = (0..probe.columns.len()).filter(|i| role_at(*i) == "CURVE").collect();
            if curve_cols.is_empty() {
                res.error = Some("No column is marked CURVE — nothing here would be stored as a log.".into());
                return res;
            }

            // The whole file, not the preview — the preview is capped for the grid. The header
            // row and, when one was detected, the units row under it are dropped so the rows here
            // line up with the columns the probe described.
            let mut rows = match split_table(path, &req.opts) {
                Ok((r, _)) => r,
                Err(e) => {
                    res.error = Some(e.to_string());
                    return res;
                }
            };
            if !rows.is_empty() {
                rows.remove(0);
            }
            if probe.units_row_skipped && !rows.is_empty() {
                rows.remove(0);
            }
            let decimal = req.opts.decimal.as_deref();

            // Group by the file's own well column, exactly as the point-data path does.
            let mut groups: Vec<(Option<String>, Vec<&Vec<String>>)> = Vec::new();
            for r in &rows {
                let key = well_col
                    .and_then(|i| r.get(i))
                    .map(|c| c.trim().to_uppercase())
                    .filter(|c| !c.is_empty());
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
                let mut keep: Vec<usize> = Vec::new();
                for (k, r) in list.iter().enumerate() {
                    let Some(d) = r.get(depth_col).and_then(|c| parse_number(c, decimal).0) else {
                        continue;
                    };
                    depths.push(d as f32);
                    keep.push(k);
                }
                if depths.is_empty() {
                    continue;
                }
                crate::units::convert_depths(&mut depths, file_unit, project_unit);

                let set = match free_curve_set(conn, &well_id, req.set_name.as_deref().unwrap_or("RAW")) {
                    Ok(s) => s,
                    Err(e) => {
                        res.notes.push(format!("could not name the delivery: {e}"));
                        continue;
                    }
                };
                let mut wrote_any = false;
                for c in &curve_cols {
                    let mnemonic = probe.columns[*c].header.trim().to_uppercase();
                    let values: Vec<f32> = keep
                        .iter()
                        .map(|k| {
                            list[*k].get(*c).and_then(|cell| parse_number(cell, decimal).0).map_or(f32::NAN, |v| v as f32)
                        })
                        .collect();
                    // A column with nothing in it for this well is not stored: an all-MISSING
                    // curve reads as a measurement that failed rather than one never delivered.
                    if !values.iter().any(|v| v.is_finite()) {
                        continue;
                    }
                    // No unit: the probe detects a units ROW and skips it rather than keeping it
                    // per column, so there is nothing here that the delivery actually said.
                    // Inventing one would state a measurement the file never made — the Curve
                    // Catalog is where a unit gets corrected.
                    // The family is what lets a module find this curve by meaning rather than by
                    // exact mnemonic, so it is looked up from the delivered name — and left absent
                    // where the table does not know it, never guessed.
                    let family = crate::curves::family_for(&mnemonic).map(|f| f.family.to_string());
                    // The canonical unit of a KNOWN family is a fact about the family, not a claim
                    // about this file; where the mnemonic is unrecognised there is nothing to say.
                    let unit = crate::curves::family_for(&mnemonic).map(|f| f.canonical_unit.to_string());
                    match db::upsert_curve_meta(
                        conn,
                        &well_id,
                        &set,
                        &mnemonic,
                        unit.as_deref(),
                        family.as_deref(),
                        Some(path),
                        None,
                    ) {
                        Ok(id) => match db::insert_curve_samples(conn, &id, &depths, &values) {
                            Ok(screened) => {
                                // SB-DBM-030: the store's null screen is a flag channel -
                                // a screened delivery says so in this import's own notes.
                                if screened > 0 {
                                    res.notes.push(format!(
                                        "null screen: {screened} large-negative sample(s) on {mnemonic} stored as missing (undeclared Geolog-family null sentinel)"
                                    ));
                                }
                                res.samples += values.iter().filter(|v| v.is_finite()).count();
                                wrote_any = true;
                                if !res.curves.contains(&mnemonic) {
                                    res.curves.push(mnemonic);
                                }
                            }
                            Err(e) => res.notes.push(format!("{mnemonic}: {e}")),
                        },
                        Err(e) => res.notes.push(format!("{mnemonic}: {e}")),
                    }
                }
                if wrote_any {
                    res.wells += 1;
                    if !res.sets.contains(&set) {
                        res.sets.push(set);
                    }
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

/// A delivery name free on this well in the generic curve store, suffixed if taken.
///
/// The `free_array_set` argument, and `resolve_core_set_name`'s before it: an import adds a
/// delivery, it never replaces one.
fn free_curve_set(conn: &Connection, well_id: &str, desired: &str) -> DbResult<String> {
    let base = {
        let t = desired.trim().to_uppercase().replace(' ', "_");
        if t.is_empty() { "RAW".to_string() } else { t }
    };
    let taken = |name: &str| -> DbResult<bool> {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM curve_meta WHERE well_id = ?1 AND upper(set_name) = ?2",
            duckdb::params![well_id, name],
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

#[cfg(test)]
mod tests {
    use super::*;

    /// SB-DIO-060 / T89 recognition half. D-31 cites 09 08 06 00 for a headerless
    /// BIFF5 stream and PK for the shared ZIP family. The ZIP control pins structural
    /// disambiguation: PK alone is insufficient; XLSX needs both named workbook entries.
    #[test]
    fn a_biff5_stream_named_xls_is_chosen_by_signature_and_a_shared_zip_signature_is_disambiguated_by_structure() {
        let biff = std::env::temp_dir().join("sandibumi-biff5-signature.xls");
        std::fs::write(&biff, [0x09, 0x08, 0x06, 0x00, 0x10, 0x00]).unwrap();
        let detected = detect_format(biff.to_str().unwrap()).unwrap();
        assert_eq!(detected.detected_format, "BIFF5 workbook stream");
        assert_eq!(detected.recognition, "09 08 06 00 signature");
        assert!(
            detected.extension_disagreement.as_deref().is_some_and(|note| {
                note.contains(".xls") && note.contains("BIFF5") && note.contains("signature chose")
            }),
            "the family/version disagreement must be explicit: {:?}",
            detected.extension_disagreement
        );

        let xlsx = std::env::temp_dir().join("sandibumi-pk-structure.bin");
        let mut xlsx_bytes = vec![0x50, 0x4B, 0x03, 0x04];
        xlsx_bytes.extend_from_slice(b"[Content_Types].xml....xl/workbook.xml");
        std::fs::write(&xlsx, xlsx_bytes).unwrap();
        let detected = detect_format(xlsx.to_str().unwrap()).unwrap();
        assert_eq!(detected.detected_format, "XLSX workbook");
        assert!(detected.choice_report.contains("PK is shared"));
        assert!(detected.choice_report.contains("xl/workbook.xml"));

        let zip = std::env::temp_dir().join("sandibumi-pk-generic.bin");
        std::fs::write(&zip, [0x50, 0x4B, 0x03, 0x04, b'a', b'.', b't', b'x', b't']).unwrap();
        let generic = detect_format(zip.to_str().unwrap()).unwrap();
        assert_eq!(generic.detected_format, "ZIP container");
        assert!(generic.choice_report.contains("no XLSX workbook structure"));

        std::fs::remove_file(&biff).ok();
        std::fs::remove_file(&xlsx).ok();
        std::fs::remove_file(&zip).ok();
    }

    /// SB-DIO-060 / T90. The `.las` extension is deliberately false. Intake must still
    /// parse the table selected by its content and say why; the CSV control prevents an
    /// implementation that reports an extension disagreement for every delimited file.
    #[test]
    fn a_delimited_text_file_named_las_is_read_as_delimited_and_the_extension_disagreement_is_reported() {
        let body = "WELL,DEPTH,CPOR\nSANDI-SIG,1000,0.20\nSANDI-SIG,1001,0.21\n";
        let disguised = std::env::temp_dir().join("sandibumi-delimited-disguised.las");
        std::fs::write(&disguised, body).unwrap();
        let probe_result = probe(disguised.to_str().unwrap(), &TableOptions::default()).unwrap();
        assert_eq!(probe_result.format.detected_format, "delimited text");
        assert_eq!(probe_result.n_rows, 2, "the signature-selected table reader must actually read it");
        assert_eq!(probe_result.columns[1].header, "DEPTH");
        let disagreement = probe_result.format.extension_disagreement.as_deref().unwrap_or("");
        assert!(disagreement.contains("extension .las disagrees with content"), "{disagreement}");
        assert!(probe_result.notes.iter().any(|note| note == disagreement));

        let ordinary = std::env::temp_dir().join("sandibumi-delimited-control.csv");
        std::fs::write(&ordinary, body).unwrap();
        let control = probe(ordinary.to_str().unwrap(), &TableOptions::default()).unwrap();
        assert_eq!(control.format.detected_format, "delimited text");
        assert!(control.format.extension_disagreement.is_none(), "a truthful extension must not be flagged");

        std::fs::remove_file(&disguised).ok();
        std::fs::remove_file(&ordinary).ok();
    }

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

    /// **A block keyed by a label line is read, and the depth is the number carrying a UNIT.**
    ///
    /// The rule is `images::WORKBOOK_RUNNER`'s, borrowed whole rather than re-invented: a caption
    /// above each block carries the plug number AND the depth, and nothing in the numbers says
    /// which is which. Taking the first would read `PLUG 12` as 12 ft.
    ///
    /// The control is the important half, and it is worse than a refusal. Read WITHOUT the block
    /// flag the label lines parse as nothing across every bin, so the all-MISSING rule drops them
    /// silently — and both blocks then import with NO depth at all, looking like a clean read of a
    /// delivery whose plugs have simply lost their depths.
    #[test]
    fn a_label_line_keys_its_block_by_the_number_that_carries_a_unit() {
        let path = std::env::temp_dir().join("sandi_block_label.csv");
        std::fs::write(
            &path,
            "1,2,4,8\n\
             PLUG 12  4633.5 ft\n\
             0.9,0.7,0.5,0.4\n\
             0.88,0.68,0.48,0.38\n\
             PLUG 13  4640,0 ft\n\
             0.8,0.6,0.4,0.3\n",
        )
        .unwrap();
        let probe = read_wide(path.to_str().unwrap(), &TableOptions::default(), &[], true).expect("read");

        assert_eq!(probe.axis, vec![1.0, 2.0, 4.0, 8.0], "the header row is still the axis");
        assert_eq!(probe.rows.len(), 3, "the captions are keys, not samples");
        assert_eq!(probe.rows[0].depth, Some(4633.5), "the number with the unit, never the plug number");
        assert_eq!(probe.rows[1].depth, Some(4633.5), "and every row of a block takes its block's depth");
        // Which makes this fixture a file that could not actually be stored: two samples at one
        // depth is a primary-key collision in `array_logs`. The warning is the point — it says so
        // here, in the preview, rather than as an engine error part way through a commit.
        assert!(
            probe.notes.iter().any(|n| n.contains("carry more than one sample") && n.contains("4633.50")),
            "two rows under one caption are two plugs at one depth, and it is named: {:?}",
            probe.notes
        );
        // The comma-decimal lesson from the plate workbooks: `4640,0` is one number, not two.
        assert_eq!(probe.rows[2].depth, Some(4640.0), "a comma decimal reads as one number");
        assert!(
            probe.notes.iter().any(|n| n.contains("2 block(s) keyed by a label line")),
            "and the run says how it read them: {:?}",
            probe.notes
        );

        // The control: without the flag the captions vanish and the depths go with them.
        let plain = read_wide(path.to_str().unwrap(), &TableOptions::default(), &[], false).expect("read");
        assert_eq!(plain.rows.len(), 3, "the captions are dropped as all-MISSING rows");
        assert!(
            plain.rows.iter().all(|r| r.depth.is_none()),
            "and every sample imports with no depth, which reads as a clean import of depthless plugs"
        );

        // A caption whose numbers carry NO unit is still refused: `PLUG 12  4633.5` could as
        // easily be plug 4633 at 12 ft, and nothing on the line settles it.
        let bare = std::env::temp_dir().join("sandi_block_label_bare.csv");
        std::fs::write(&bare, "1,2,4\nPLUG 12  4633.5\n0.9,0.7,0.5\n").unwrap();
        let no_unit = read_wide(bare.to_str().unwrap(), &TableOptions::default(), &[], true).expect("read");
        assert!(no_unit.rows.iter().all(|r| r.depth.is_none()), "no unit, no depth");
        assert!(
            no_unit.notes.iter().any(|n| n.contains("carries a depth with a UNIT")),
            "and the reason is named: {:?}",
            no_unit.notes
        );
    }

    /// The unit rule itself, including the two shapes a laboratory actually writes.
    #[test]
    fn a_labels_depth_is_the_number_that_carries_a_unit() {
        let cells = |s: &str| vec![s.to_string()];
        assert_eq!(depth_from_label(&cells("PLUG 12  4633.5 ft"), None, ' ').0, Some(4633.5));
        assert_eq!(
            depth_from_label(&cells("SAMPLE 7 / 2103.4M (CORE)"), None, ' ').0,
            Some(2103.4),
            "a glued unit, and the PLUG NUMBER before it must not be read as the depth"
        );
        assert_eq!(depth_from_label(&cells("PLUG 12  4633.5"), None, ' ').0, None, "no unit, no depth");
        assert_eq!(depth_from_label(&cells("CORE DESCRIPTION"), None, ' ').0, None);
        // A caption naming two depths is a DUPLICATE, not a range to choose an end of: a caption
        // keys one plug and a plug sits at one depth. The first is used so the block still
        // imports, and both are seen so the caller can say there were two.
        let (d, n) = depth_from_label(&cells("BOX 3  2103.4 m to 2104.1 m"), None, ' ');
        assert_eq!((d, n), (Some(2103.4), 2), "both are seen, so the duplicate can be reported");
        // A bare letter is not a unit: on a line naming a facies code, `F` is not evidence.
        assert_eq!(depth_from_label(&cells("PLUG 12 4633.5 F"), None, ' ').0, None);
        // The comma-decimal trap. A comma-delimited file splits `4640,0 ft` into two cells, and
        // rejoining them with a SPACE hands the reader `4640 0 ft` — where the number carrying the
        // unit is zero. Rejoined with the delimiter it is the one number it always was.
        let split = vec!["PLUG 13  4640".to_string(), "0 ft".to_string()];
        assert_eq!(depth_from_label(&split, None, ',').0, Some(4640.0), "reassembled as written");
        assert_eq!(depth_from_label(&split, None, ' ').0, Some(0.0), "and this is what a space would have given");
    }

    /// **A plug sits at one depth, and a duplicate is named before anything is stored.** Jauhar,
    /// 2026-08-05: *"it should be 1 plug number only, should warn user if duplicate"*.
    ///
    /// The stakes are `array_logs`'s PRIMARY KEY (well, set, curve, depth): one stored vector per
    /// depth, so a second sample at the same depth is a constraint violation that fails the whole
    /// curve's write with an engine message naming nothing the user put in the file. Every case
    /// below imports cleanly right up to the moment it does not.
    ///
    /// Two captions claiming one plug and one caption carrying two rows are the same collision
    /// reached from opposite directions, so both are caught — but each is reported ONCE, by the
    /// message that names its own fix, rather than twice from both ends.
    #[test]
    fn a_plug_sits_at_one_depth_and_a_duplicate_is_named() {
        let write = |name: &str, body: &str| {
            let p = std::env::temp_dir().join(name);
            std::fs::write(&p, body).unwrap();
            p
        };
        let dupe_note = |p: &std::path::Path, roles: &[String], block: bool| -> Vec<String> {
            read_wide(p.to_str().unwrap(), &TableOptions::default(), roles, block)
                .expect("read")
                .notes
                .into_iter()
                .filter(|n| n.contains("key more than one block") || n.contains("carry more than one sample"))
                .collect()
        };

        // Two blocks claiming one plug. Named as a CAPTION problem, because that is where the fix
        // is — and only once: the row-level check skips a depth already explained.
        let two_blocks = write(
            "sandi_dupe_blocks.csv",
            "1,2,4\nPLUG 12  4633.5 ft\n0.9,0.7,0.5\nPLUG 13  4633.5 ft\n0.8,0.6,0.4\n",
        );
        let n = dupe_note(&two_blocks, &[], true);
        assert_eq!(n.len(), 1, "one message, not the same fault described twice: {n:?}");
        assert!(n[0].contains("key more than one block") && n[0].contains("4633.50"), "{n:?}");

        // One caption, two rows — the case a caption check cannot see, and the likelier mistake.
        let two_rows =
            write("sandi_dupe_rows.csv", "1,2,4\nPLUG 12  4633.5 ft\n0.9,0.7,0.5\n0.88,0.68,0.48\n");
        let n = dupe_note(&two_rows, &[], true);
        assert_eq!(n.len(), 1, "{n:?}");
        assert!(n[0].contains("carry more than one sample") && n[0].contains("4633.50"), "{n:?}");

        // The control. A clean delivery must say nothing at all — a warning that fires on good
        // files is one nobody reads, and this whole family lives or dies on being believed.
        let clean = write(
            "sandi_dupe_clean.csv",
            "1,2,4\nPLUG 12  4633.5 ft\n0.9,0.7,0.5\nPLUG 13  4640.0 ft\n0.8,0.6,0.4\n",
        );
        assert!(dupe_note(&clean, &[], true).is_empty(), "a clean file is silent");

        // Two WELLS sampled at one depth is entirely ordinary, and the check is grouped by well so
        // it stays quiet. Without the grouping this would fire on every multi-well delivery, which
        // is the fastest way to train a user to ignore the message.
        let roles: Vec<String> =
            ["WELL", "DEPTH", "", "", ""].iter().map(|s| s.to_string()).collect();
        let two_wells = write(
            "sandi_dupe_wells.csv",
            "WELL,DEPTH,1,2,4\nSANDI-1,2000.0,0.9,0.7,0.5\nSANDI-2,2000.0,0.8,0.6,0.4\n",
        );
        assert!(dupe_note(&two_wells, &roles, false).is_empty(), "two wells may share a depth");

        // ...and the same file with ONE well repeated is the collision again, now via a depth
        // COLUMN rather than a caption. Same rule, so it is caught without a second check.
        let one_well = write(
            "sandi_dupe_one_well.csv",
            "WELL,DEPTH,1,2,4\nSANDI-1,2000.0,0.9,0.7,0.5\nSANDI-1,2000.0,0.8,0.6,0.4\n",
        );
        let n = dupe_note(&one_well, &roles, false);
        assert_eq!(n.len(), 1, "{n:?}");
        assert!(n[0].contains("SANDI-1 2000.00"), "the well is named alongside the depth: {n:?}");
    }

    /// **The preview counts the whole file and draws a slice of it — including every duplicate.**
    ///
    /// A preview that stopped at its cap would be at its most useless on exactly the delivery that
    /// needs it: a big export nobody scrolls through by hand, whose duplicate sits at row 900. So
    /// the cap governs how much is DRAWN, never what was checked, and a clashing sample beyond it
    /// is pulled in. `n_rows` stays the file's own count — a preview reporting its capped length
    /// would tell the user a 4,000-sample delivery held 40.
    #[test]
    fn the_preview_counts_every_sample_and_draws_every_duplicate() {
        // One clean sample per depth, well past the cap, then a duplicate of the very first depth
        // right at the end — the position a capped preview would miss.
        let mut body = String::from("DEPTH,1,2,4\n");
        for i in 0..(ARRAY_PREVIEW_ROWS * 3) {
            body.push_str(&format!("{}.0,0.9,0.7,0.5\n", 2000 + i));
        }
        body.push_str("2000.0,0.8,0.6,0.4\n");
        let path = std::env::temp_dir().join("sandi_preview_cap.csv");
        std::fs::write(&path, &body).unwrap();
        let roles: Vec<String> = ["DEPTH", "", "", ""].iter().map(|s| s.to_string()).collect();

        let pv = probe_arrays(path.to_str().unwrap(), &TableOptions::default(), &roles, false)
            .expect("read");

        assert_eq!(pv.n_rows, ARRAY_PREVIEW_ROWS * 3 + 1, "counted over the whole file");
        assert!(pv.rows.len() < pv.n_rows, "and only a slice is carried back");
        assert_eq!(pv.clashes.len(), 1, "one depth is duplicated");

        // The duplicate is the LAST row of the file, far beyond the cap, and it is drawn anyway.
        let last = pv.n_rows - 1;
        assert!(
            pv.row_index.contains(&last),
            "the duplicate must be drawn wherever it sits: {:?}",
            pv.row_index
        );
        // ...and it says WHERE it sits, so it cannot read as following the row above it.
        let at = pv.row_index.iter().position(|i| *i == last).unwrap();
        assert_eq!(pv.rows[at].depth, Some(2000.0));
        assert!(pv.row_index[at] > ARRAY_PREVIEW_ROWS, "its file position is carried, not its place here");

        // The control: a clean file of the same size draws the cap and pulls in nothing extra.
        let clean = std::env::temp_dir().join("sandi_preview_clean.csv");
        std::fs::write(&clean, &body[..body.len() - "2000.0,0.8,0.6,0.4\n".len()]).unwrap();
        let cpv = probe_arrays(clean.to_str().unwrap(), &TableOptions::default(), &roles, false)
            .expect("read");
        assert!(cpv.clashes.is_empty(), "a clean file has none");
        assert_eq!(cpv.rows.len(), ARRAY_PREVIEW_ROWS, "so exactly the cap is drawn");
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
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
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

    /// **A column marked CURVE lands in the curve store, where modules can read it.**
    ///
    /// The route a delimited file of logs had no way in by. Stored as point data instead — which
    /// is what every other role does — a GR every 15 cm would be invisible to every module, plot
    /// and export: the `standard_curves` shadowing failure reached by another road.
    #[test]
    fn a_column_marked_curve_becomes_a_log_rather_than_point_data() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        crate::units::set_project_depth_unit(&conn, crate::units::DepthUnit::Metres).unwrap();
        let wid = uuid::Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-CURVE", None, None, None).unwrap();
        let w = wid.to_string();

        let path = std::env::temp_dir().join("sandi_curve_role.csv");
        std::fs::write(&path, "DEPTH,GR,NOTES
2000.0,45.0,clean
2000.15,52.0,clean
2000.30,88.0,shale
").unwrap();
        let req = CurveCommit {
            paths: vec![path.to_str().unwrap().to_string()],
            roles: vec!["DEPTH".into(), "CURVE".into(), "IGNORE".into()],
            opts: TableOptions::default(),
            set_name: Some("WIRE".into()),
            depth_unit: None,
            fallback_well_id: Some(w.clone()),
        };
        let res = commit_curves(&conn, &req);
        assert!(res[0].error.is_none(), "{:?}", res[0].error);
        assert_eq!(res[0].curves, vec!["GR".to_string()]);
        assert_eq!(res[0].samples, 3);
        assert_eq!(res[0].sets, vec!["WIRE".to_string()]);

        // It is readable AS a curve, which is the whole point — and carries the GR family, so a
        // module looking for a gamma ray by meaning finds it.
        let (family, n): (Option<String>, i64) = conn
            .query_row(
                "SELECT m.family, (SELECT COUNT(*) FROM curve_samples s WHERE s.curve_id = m.curve_id)
                 FROM curve_meta m WHERE m.well_id = ?1 AND m.mnemonic = 'GR'",
                duckdb::params![&w],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(n, 3, "every sample is stored");
        assert_eq!(family.as_deref(), Some("GR"), "and the delivered mnemonic resolves to its family");

        // A second delivery of the same name lands beside the first rather than replacing it.
        let again = commit_curves(&conn, &req);
        assert_eq!(again[0].sets, vec!["WIRE_1".to_string()], "an import never overwrites");

        // A file with no CURVE column is refused rather than importing nothing quietly.
        let none = CurveCommit { roles: vec!["DEPTH".into(), "ITEM".into(), "IGNORE".into()], ..req };
        assert!(commit_curves(&conn, &none)[0].error.is_some());
    }
}
