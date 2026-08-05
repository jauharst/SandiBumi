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

use crate::parsers::{self, CoreMapping, ParseResult};
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
}
