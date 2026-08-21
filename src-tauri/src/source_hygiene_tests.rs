//! Source-hygiene gates over the Rust tree (AUDIT-2026-08-20 findings 72 and 73).
//!
//! One root cause, two visible shapes. A line-wrapping pass broke lines at the wrong column, so
//! punctuation belonging at the end of a statement was stranded on a line of its own, and the same
//! dropped continuation inside a message left a run of spaces torn through mid-sentence.
//!
//! The code half is cosmetic and the compiler has no opinion on it. The message half is not: a
//! refusal that reads "convert with nphimat first" with eighteen spaces through the middle looks
//! like a rendering fault in the application, and the reader's next question is whether the number
//! printed beside it is damaged too. These are the refusals that carry a DEC or SB citation, so
//! they are read precisely when someone is deciding whether to trust an answer.
//!
//! `cargo fmt` is deliberately NOT the gate. Measured on this tree it rewrites 77,865 diff lines
//! across 70 of the 73 source files - the repository has never conformed to rustfmt defaults, so
//! adopting it would bury `git blame` for the whole codebase in order to fix 92 sites. These two
//! scans name the class instead, and stay silent about every other formatting opinion.

use std::path::{Path, PathBuf};

fn rust_sources(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("read the Rust source tree") {
        let path = entry.expect("read a Rust source entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn sorted_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut paths = Vec::new();
    rust_sources(&root, &mut paths);
    paths.sort();
    paths
}

/// Line numbers where a dropped continuation stranded its punctuation: a bare `;` or `,` alone on
/// a line, or a space left in front of one at the end of a line.
fn orphaned_punctuation(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_end();
        if trimmed.trim_start() == ";" || trimmed.trim_start() == "," {
            found.push(index + 1);
            continue;
        }
        let mut tail = trimmed.chars().rev();
        let last = tail.next();
        let previous = tail.next();
        if matches!(last, Some(';') | Some(',')) && previous == Some(' ') {
            found.push(index + 1);
        }
    }
    found
}

/// Line numbers whose message text carries a run of three or more spaces mid-sentence.
///
/// The run must sit between a letter (or closing punctuation) and a letter, which is what
/// separates a torn sentence from deliberate padding. A run led by an indent marker - a bullet,
/// an aligned column, the opening quote itself - is layout, and widening the rule to catch a
/// dash-led gap would flag every bullet list in the report and deck writers.
fn prose_gaps(text: &str) -> Vec<usize> {
    // Compiled once for the whole file. Built per line it costs a hundred seconds of gate time,
    // which is how a scan this cheap ends up looking too expensive to keep.
    let pattern = regex::Regex::new(r#""([^"\\]|\\.)*""#).expect("string-literal pattern");
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for literal in pattern.find_iter(line) {
            let literal = literal.as_str();
            if !spacing_is_the_format(literal) && has_prose_gap(literal) {
                found.push(index + 1);
                break;
            }
        }
    }
    found
}

/// A literal whose spacing IS the format: SQL, a column-aligned DDL fragment, or a LAS mnemonic
/// line. Collapsing one of these changes what the code means rather than how it reads - the LAS
/// header writer pads to fixed columns, and a DDL assertion matches the padding it generated.
fn spacing_is_the_format(literal: &str) -> bool {
    const SQL: &[&str] = &[
        "SELECT", "INSERT", "UPDATE", "DELETE", "FROM", "WHERE", "VALUES", "JOIN", "CREATE",
        "ALTER", "COALESCE", "ADD COLUMN", " AND ", " OR ", " ON ",
    ];
    const DDL: &[&str] = &[
        "FLOAT", "INTEGER", "VARCHAR", "BLOB", "BOOLEAN", "DOUBLE", "TIMESTAMP", "BIGINT",
    ];
    if SQL.iter().chain(DDL.iter()).any(|token| literal.contains(token)) {
        return true;
    }
    literal.contains(" : ") && names_a_las_mnemonic(literal)
}

/// An uppercase mnemonic followed by its unit dot, as every LAS header line begins.
fn names_a_las_mnemonic(literal: &str) -> bool {
    let chars: Vec<char> = literal.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if !chars[index].is_ascii_uppercase() {
            index += 1;
            continue;
        }
        while index < chars.len()
            && (chars[index].is_ascii_uppercase()
                || chars[index].is_ascii_digit()
                || chars[index] == '_')
        {
            index += 1;
        }
        let mut probe = index;
        while probe < chars.len() && chars[probe] == ' ' {
            probe += 1;
        }
        if probe < chars.len() && chars[probe] == '.' {
            return true;
        }
    }
    false
}

fn has_prose_gap(literal: &str) -> bool {
    let chars: Vec<char> = literal.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != ' ' {
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && chars[index] == ' ' {
            index += 1;
        }
        if index - start < 3 || start == 0 || index >= chars.len() {
            continue;
        }
        let before = chars[start - 1];
        let after = chars[index];
        let opens = before.is_ascii_alphabetic()
            || matches!(before, ',' | '.' | ':' | ';' | ')' | ']');
        if opens && (after.is_ascii_alphabetic() || after == '(') {
            return true;
        }
    }
    false
}

#[test]
fn a_dropped_line_continuation_never_leaves_its_punctuation_stranded() {
    // AUDIT-2026-08-20 finding 72. 92 sites at master, counted: 3 bare semicolons, 61 bare
    // commas and 28 spaces left in front of one. Cosmetic, all compiling - which is exactly why
    // nothing catches them and why they accumulate. The finding proposed one `cargo fmt`; that
    // was measured and rejected (see this module's header).
    let mut offenders = Vec::new();
    for path in sorted_sources() {
        let source = std::fs::read_to_string(&path).expect("read a UTF-8 Rust source file");
        for line in orphaned_punctuation(&source) {
            offenders.push(format!("{}:{}", path.display(), line));
        }
    }
    assert_eq!(
        offenders,
        Vec::<String>::new(),
        "punctuation stranded by a dropped line continuation"
    );

    // And the sweep cannot pass by not looking. Built by concatenation so this file is not an
    // offender against its own scan.
    let stranded = ["let value = compute()", "    ;"].join("\n");
    assert_eq!(orphaned_punctuation(&stranded), vec![2], "a bare semicolon is stranded");
    let spaced = ["let value = compute() ;", "let next = 1;"].join("\n");
    assert_eq!(orphaned_punctuation(&spaced), vec![1], "a space in front of one is stranded");
    let clean = ["let value = compute();", "let next = 1;"].join("\n");
    assert!(orphaned_punctuation(&clean).is_empty(), "well-formed punctuation is not an offence");
}

#[test]
fn a_message_an_operator_reads_never_carries_a_dropped_line_continuation() {
    // AUDIT-2026-08-20 finding 73. The same tear, but inside text a person reads: 29 sites,
    // concentrated in the refusals that cite a decision record - the neutron-basis refusal, the
    // no-default cut-off refusal, the percent-versus-fraction import notes.
    let mut offenders = Vec::new();
    for path in sorted_sources() {
        let source = std::fs::read_to_string(&path).expect("read a UTF-8 Rust source file");
        for line in prose_gaps(&source) {
            offenders.push(format!("{}:{}", path.display(), line));
        }
    }
    assert_eq!(
        offenders,
        Vec::<String>::new(),
        "a message carries a run of spaces torn through mid-sentence"
    );

    // Pinned from both sides: the scan must catch a torn sentence, and must leave alone a literal
    // whose spacing is the format. An exclusion that swallowed everything would also report zero.
    let torn = ["\"a refusal with", "   ", "a dropped continuation\""].concat();
    assert!(has_prose_gap(&torn), "a torn sentence must be found");
    assert!(!spacing_is_the_format(&torn), "ordinary prose is not layout");

    let header = ["\"WELL .", "        ", "IDENTITY   : well name\""].concat();
    assert!(spacing_is_the_format(&header), "a LAS mnemonic line is layout, not prose");
    let query = ["\"q.well_id = live.well_id", "     ", "AND q.depth = live.depth\""].concat();
    assert!(spacing_is_the_format(&query), "a SQL fragment is layout, not prose");
    let indent = ["\"", "   ", "note: {n}\""].concat();
    assert!(!has_prose_gap(&indent), "an indent is padding, not a torn sentence");
}
