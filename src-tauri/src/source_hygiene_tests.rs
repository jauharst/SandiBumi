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

/// Every `#[cfg(test)]` item that declares a fully PUBLIC surface, and what makes it honest.
///
/// Such an item is shaped exactly like production code and can never exist in a real build. The
/// compiler emits no `dead_code` warning for it, so `tools/gate2-hygiene.mjs` - the mechanism
/// that OWNS disconnected capability, by inventorying the warnings that capability produces -
/// cannot see it at all. Thirteen accumulated in `plotting.rs` that way, in a file already
/// carrying 21 owned warnings: the ownership mechanism was working, and these were simply
/// invisible to it. Five were second implementations of contracts whose live TypeScript is
/// pinned on the same PRD fixtures, so the copy that could never run was the one being tested.
///
/// Scoped to plain `pub` because that is where the harm was. There are also eleven
/// `pub(crate)` items under `#[cfg(test)]`, counted; every one of them is a probe or a one-line
/// delegation to the production entry point, and none was a second implementation. Widening the
/// rule would add eleven benign entries and dilute the list rather than sharpen it.
///
/// Checked in BOTH directions: an item in the tree that is not listed fails, and a listing that
/// no longer names such an item fails, so an entry cannot outlive its reason.
const TEST_ONLY_PUBLIC_SURFACES: &[(&str, &str, &str)] = &[
    ("db.rs", "update_computed_sample",
     "edits one sample so a test can check what a production read gives back"),
    ("ancestry.rs", "try_new",
     "the short constructor; production builds a spec through try_new_with_legacy"),
    ("ingest.rs", "import_las_files",
     "one line delegating to import_las_files_with, for the legacy test call sites"),
    ("param_sources.rs", "topics",
     "lists the declared topics so a test can hold them against the source"),
    ("plotting.rs", "PersistedPlotDocument",
     "the shape a test reads back after a production write"),
    ("plotting.rs", "list_persisted_plot_states",
     "reads back what a production write stored"),
];

/// The name this line declares, when it declares a fully public item.
fn public_item_name(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("pub ")?;
    let rest = ["struct ", "enum ", "fn ", "trait ", "type ", "const "]
        .iter()
        .find_map(|kind| rest.strip_prefix(kind))?;
    let name: String = rest
        .chars()
        .take_while(|value| value.is_alphanumeric() || *value == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// Every public surface in one file that is gated behind `#[cfg(test)]`.
fn test_only_public_surfaces(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != "#[cfg(test)]" {
            continue;
        }
        // Further attributes and a doc block may sit between the gate and the item it gates.
        let mut probe = index + 1;
        while probe < lines.len()
            && (lines[probe].trim_start().starts_with("#[")
                || lines[probe].trim_start().starts_with("///"))
        {
            probe += 1;
        }
        if let Some(name) = lines.get(probe).and_then(|line| public_item_name(line)) {
            found.push(name);
        }
    }
    found
}

#[test]
fn a_test_only_item_never_wears_a_production_shape_without_saying_why() {
    // AUDIT-2026-08-20 finding 67. The five removed from plotting.rs re-implemented SB-PLT-002,
    // -004, -015 and -016 against code that can never run, while the live TypeScript was already
    // pinned on the same requirement ids and the same chapter fixtures - a keep-in-agreement
    // hazard with one side permanently unrunnable, which is the one shape that cannot be caught
    // by testing either side.
    let mut observed: Vec<(String, String)> = Vec::new();
    for path in sorted_sources() {
        let source = std::fs::read_to_string(&path).expect("read a UTF-8 Rust source file");
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        for name in test_only_public_surfaces(&source) {
            observed.push((file.clone(), name));
        }
    }
    observed.sort();

    let mut declared: Vec<(String, String)> = TEST_ONLY_PUBLIC_SURFACES
        .iter()
        .map(|(file, name, reason)| {
            assert!(!reason.trim().is_empty(), "{file}'s {name} is listed with no reason");
            ((*file).to_string(), (*name).to_string())
        })
        .collect();
    declared.sort();

    let undeclared: Vec<&(String, String)> =
        observed.iter().filter(|item| !declared.contains(item)).collect();
    assert!(
        undeclared.is_empty(),
        "a test-only item wears a production shape and is not declared: {undeclared:?}"
    );

    let stale: Vec<&(String, String)> =
        declared.iter().filter(|item| !observed.contains(item)).collect();
    assert!(
        stale.is_empty(),
        "a declared test-only surface no longer exists, so its reason has outlived it: {stale:?}"
    );

    // The sweep cannot pass by not looking. Assembled rather than written literally, so this
    // file is not an offender against its own scan.
    let shadow = ["#[cfg(test)]", "pub fn resolve_axis_range() {}"].join("\n");
    assert_eq!(
        test_only_public_surfaces(&shadow),
        vec!["resolve_axis_range".to_string()],
        "a public surface behind the test gate must be found"
    );
    let helper = ["#[cfg(test)]", "fn a_private_helper() {}"].join("\n");
    assert!(
        test_only_public_surfaces(&helper).is_empty(),
        "a private helper declares no surface and is not this class"
    );
}


/// The opening word of an instruction a reader can act on.
///
/// A refusal that stops a run has to reach one of these. Stating only what went wrong leaves the
/// reader with the question they already had, and the usual next move is to try the same click
/// again - which refuses again, in the same words.
const FIX_CUES: &[&str] = &[
    "Set ", "Choose ", "Pick ", "Import ", "Re-import ", "Re-run ", "Correct ", "Add ",
    "Declare ", "Enter ", "Give ", "Click ", "Read ", "Tune ", "Widen ", "Delete ",
];

/// The refusals a user meets, each keyed by the phrase that names what was refused.
///
/// The audited set (2026-08-22), deliberately not every `Err` in the tree. An internal fault has
/// nobody to instruct, and a pass-through that forwards a runner's own error must not bury it
/// under a generic instruction - `"{} cannot be a reference plate: {}"` is that shape and is
/// rightly absent. The naming phrase is the key because it is the half a test should pin: it
/// says WHICH condition refused, while the sentence after it is prose that may be improved.
const NAMED_REFUSALS: &[(&str, &str)] = &[
    ("petrography.rs", "has no reference plate."),
    ("petrography.rs", "no plates in {dataset} for this well."),
    ("petrography.rs", "at least two minerals labelled"),
    ("petrography.rs", "gave no matrix colour"),
    ("petrography.rs", "is declared as blue-dyed epoxy."),
    ("petrography.rs", "cannot be a reference plate: it is not among"),
    ("petrography.rs", "cannot be a reference plate: its own median hue"),
    ("petrography.rs", "not measured - no reference plate covers"),
    ("coreimage.rs", "no core photographs in {dataset} for this well."),
    ("ingest.rs", "does not declare a sampling style."),
    ("ingest.rs", "is declared POINT, so it cannot be imported"),
    ("ingest.rs", "but no regular-step tolerance"),
    ("ingest.rs", "the regular-step tolerance must be"),
    ("ingest.rs", "carries no STEP."),
    ("ingest.rs", "its declared STEP is not a number."),
    ("ingest.rs", "STEP is zero or not finite."),
    ("equations.rs", "sampling style has not been verified."),
    ("equations.rs", "the stored sampling verdict"),
    ("equations.rs", "POINT data has no continuous frame."),
    ("ancestry.rs", "sampling style is unrecorded or unreadable."),
    ("ancestry.rs", "duplicate-depth resolution is unrecorded or"),
    ("ancestry.rs", "must declare duplicate-depth resolution REFUSE"),
    ("workflow.rs", "carries duplicate quantity metadata."),
    ("curves.rs", "quantity-kind mismatch:"),
];

/// What a refusal says AFTER its naming phrase, read as the operator sees it.
///
/// Rust's backslash continuation eats the newline and the indent behind it, so a message written
/// across five source lines is one sentence at run time. Joining first is the whole point: a fix
/// that happens to start on the line after a break is otherwise invisible to a line-wise scan.
fn refusal_after(text: &str, marker: &str) -> Option<String> {
    let continuation = regex::Regex::new(r"\\\r?\n\s*").expect("continuation pattern");
    let joined = continuation.replace_all(text, "");
    let start = joined.find(marker)? + marker.len();
    let mut out = String::new();
    let mut rest = joined[start..].chars();
    while let Some(character) = rest.next() {
        match character {
            '\\' => {
                rest.next();
            }
            '"' => return Some(out),
            _ => out.push(character),
        }
    }
    None
}

fn source_named(file: &str) -> String {
    let path = sorted_sources()
        .into_iter()
        .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(file))
        .unwrap_or_else(|| panic!("{file} is in the source tree"));
    std::fs::read_to_string(path).expect("read a UTF-8 Rust source file")
}

#[test]
fn a_refusal_that_stops_a_run_says_what_to_do_about_it() {
    // The house pattern, taken from this app's own best three messages rather than imported:
    // what was refused, naming the thing - why it cannot be guessed - what to do, naming the
    // control. Before this sweep the halves were split across the tree: petrography named a
    // pane and a reason, while the import refusals read like a schema validator talking to its
    // author ("CONTINUOUS_REGULAR requires a finite non-zero declared STEP") and told a
    // petrophysicist nothing they could act on.
    let mut missing = Vec::new();
    let mut unfound = Vec::new();
    for (file, marker) in NAMED_REFUSALS {
        let source = source_named(file);
        match refusal_after(&source, marker) {
            None => unfound.push(format!("{file}: {marker}")),
            Some(tail) => {
                if !FIX_CUES.iter().any(|cue| tail.contains(cue)) {
                    missing.push(format!("{file}: {marker}"));
                }
            }
        }
    }
    assert_eq!(
        unfound,
        Vec::<String>::new(),
        "a named refusal has moved or been reworded away, so this list no longer covers the set"
    );
    assert_eq!(
        missing,
        Vec::<String>::new(),
        "a refusal states what went wrong but never what to do about it"
    );

    // Pinned from both sides. A cue vocabulary that matched anything would report zero offenders
    // just as loudly, so the bare statement of a fault must NOT pass.
    assert!(
        !FIX_CUES.iter().any(|cue| "preparation is not blue-dyed epoxy".contains(cue)),
        "naming the fault is not the same as saying what to do about it"
    );
    // And the continuation join has to happen, or the scan silently stops reading at the first
    // line break and every wrapped fix looks absent. Assembled rather than written literally, so
    // this file is not an offender against the sibling scans above.
    let wrapped = ["\"a fault occurred. \\", "     Set it in Plate Details.\""].join("\n");
    let tail = refusal_after(&wrapped, "a fault occurred.").expect("the marker is present");
    assert!(tail.contains("Set "), "a fix wrapped onto the next source line still counts");
}

#[test]
fn one_refusal_is_worded_once_and_called_twice_rather_than_copied() {
    // Four call sites carried one byte-identical "no pictures..." refusal, two in each of two
    // files. Four copies is four places for the wording to drift apart - the `requireWell`
    // argument, which this repository already makes for the same reason.
    //
    // The needle is assembled rather than written literally, so this file is not an offender
    // against its own scan - the same care the two sweeps above take.
    let needle = ["no pictures", " in "].concat();
    let mut copies = Vec::new();
    for path in sorted_sources() {
        let source = std::fs::read_to_string(&path).expect("read a UTF-8 Rust source file");
        let hits = source.matches(needle.as_str()).count();
        if hits > 0 {
            copies.push(format!("{}: {hits}", path.display()));
        }
    }
    assert_eq!(copies, Vec::<String>::new(), "the copied refusal is back in the tree");

    // Both sides: one wording, and both call sites still reaching it. A helper nobody calls would
    // satisfy the sweep above just as well.
    for (file, helper) in [("petrography.rs", "no_plates"), ("coreimage.rs", "no_photographs")] {
        let source = source_named(file);
        assert_eq!(
            source.matches(&format!("fn {helper}(")).count(),
            1,
            "{file} states the wording exactly once"
        );
        assert_eq!(
            source.matches(&format!("{helper}(&spec.dataset)")).count(),
            2,
            "{file} reaches that one wording from both call sites"
        );
    }
}
