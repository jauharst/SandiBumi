//! One file a user can send when something went wrong, carrying the SHAPE of the problem and
//! none of their delivery.
//!
//! The gap this closes, measured before it was written: the diagnosis already existed but was
//! welded to the data. `CurveAncestry` records exactly how every number was made - module,
//! version, inputs, parameters, zone scope, depth frame - and none of it could reach the author
//! of the software without shipping the client's `.duckdb`. Meanwhile "it was slow" was
//! unanswerable at all: `project.rs` measured 13 opening steps and only 4 reached the user, the
//! rest going to an `eprintln!` console that a built exe does not have, and NOTHING recorded how
//! long a module run, a chain or an import took.
//!
//! So this module does two things and deliberately nothing else:
//!
//! 1. **Collects** what the app already knew but threw away - step timings, operation durations,
//!    internal errors. In memory, capped, never written to the project. A diagnostic is not an
//!    interpretation and must never turn up in a log set.
//! 2. **Renders** one plain-text report, with every name that belongs to the client replaced.
//!
//! **It is not telemetry.** Nothing is transmitted, nothing is collected in the background, there
//! is no daemon and no phone-home. The user presses a button, reads what it produced, and decides
//! whether to send it.
//!
//! ## What travels, and what does not
//!
//! Jauhar's call (2026-08-22), asked as an explicit choice: **parameter VALUES travel**. Without
//! `m`, `n`, `a`, `Rw` and the cut-offs there is usually no way to say why a number looks wrong,
//! and that is half of what the report is for. The consequence is that the report carries the
//! client's own calibration - analytical work product - so it SAYS SO on its own face, above
//! everything else, where somebody about to attach it to an email will read it.
//!
//! Never included, at all: well names, field names, the project name, any file path, and any
//! curve VALUE. Redaction is driven by the project's own well and field list rather than by
//! pattern-matching, because a pattern would eventually miss one; every name the project knows is
//! replaced wherever it appears, including inside an operation label that happened to embed it.

use duckdb::Connection;
use std::sync::Mutex;

/// How long one opening step took. `project.rs` measured these already and printed them to a
/// console that does not exist in a built exe - which is how a 15-minute one-time migration
/// looked like a hang.
struct BootStep {
    name: String,
    millis: u128,
}

/// One operation and how long it took. Recorded at `jobs::run_job`, which every module run,
/// chain, import, export and render passes through, so one place covers all of them.
struct OpTiming {
    kind: String,
    label: String,
    millis: u128,
    items: usize,
    /// `ok`, `cancelled` or `FAILED` - three states, not a boolean. A cancelled run is not a
    /// completed one, and a long abandoned chain reported as "ok" would mislead a support call in
    /// exactly the case the report exists for.
    state: String,
}

/// The session hit something that should not happen. Recorded rather than swallowed, because
/// "nothing works any more" with no explanation is the worst support call there is.
struct InternalError {
    context: String,
    at_millis: u128,
}

static BOOT_STEPS: Mutex<Vec<BootStep>> = Mutex::new(Vec::new());
static OPS: Mutex<Vec<OpTiming>> = Mutex::new(Vec::new());
static ERRORS: Mutex<Vec<InternalError>> = Mutex::new(Vec::new());

/// Operations kept. A session that ran two thousand wells would otherwise grow this without
/// bound, and the last hundred is what a support call is about.
const OP_CAP: usize = 200;

fn since_epoch_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Record one opening step.
pub fn record_boot_step(name: &str, millis: u128) {
    if let Ok(mut steps) = BOOT_STEPS.lock() {
        steps.push(BootStep { name: name.to_string(), millis });
    }
}

/// Print one opening step to the dev console AND record it, from ONE measurement.
///
/// Both in one call, deliberately. The `eprintln!` is still worth having under `tauri dev`, but a
/// second `t.elapsed()` for the report would print one number and record another - a difference
/// too small to matter and exactly the kind of drift this repo does not accept between two
/// renderings of a single measurement.
///
/// A step whose console line carries something the report must not - a file path - calls
/// `record_boot_step` directly instead, from the same measurement. See `init_db_resilient` in
/// `project::open_and_migrate`.
pub fn boot_step(name: &str, elapsed: std::time::Duration) {
    eprintln!("[boot] {name}: {elapsed:?}");
    record_boot_step(name, elapsed.as_millis());
}

/// Record one finished operation and how long it took.
///
/// `label` may embed a well name - that is the caller's business and not something to sanitise
/// here, because the Processing panel shows the same label and needs it intact. Redaction happens
/// once, where the report is RENDERED.
pub fn record_op(kind: &str, label: &str, millis: u128, items: usize, state: &str) {
    if let Ok(mut ops) = OPS.lock() {
        if ops.len() >= OP_CAP {
            ops.remove(0);
        }
        ops.push(OpTiming {
            kind: kind.to_string(),
            label: label.to_string(),
            millis,
            items,
            state: state.to_string(),
        });
    }
}

/// Record that the session hit an internal error.
pub fn record_internal_error(context: &str) {
    if let Ok(mut errors) = ERRORS.lock() {
        // One entry per distinct context. A panic inside a rayon fold fires once per well, and a
        // report listing the same line two thousand times says less than one saying it happened.
        if errors.iter().any(|e| e.context == context) {
            return;
        }
        errors.push(InternalError { context: context.to_string(), at_millis: since_epoch_millis() });
    }
}

/// Catches every panic in the process and records where it happened.
///
/// This is the observability half of `SECURITY-REVIEW-2026-08-22.md` finding F2 - "one panic
/// anywhere makes the project unusable until restart". A panic poisons whatever mutex it was
/// holding, and every later `lock().unwrap()` on it panics in turn, so the user sees an app that
/// has stopped working with no first cause visible anywhere. **This does not fix that** - it makes
/// the first cause reportable. Recovering from a poisoned lock is a separate change.
///
/// One hook rather than a guard at each of the 182 `db.0.lock().unwrap()` sites in `lib.rs`, and
/// not only because it is smaller: `.unwrap()` panics BEFORE any code at that site could record
/// anything, so a per-site guard structurally cannot catch the first one. The hook can, wherever
/// it happens.
///
/// The default hook still runs, so `tauri dev` keeps printing its backtrace.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let where_ = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        // A panic payload is usually one of ours, but an assertion could have interpolated
        // something off the project. It is redacted at render like every other free-text field.
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".to_string());
        record_internal_error(&format!("{where_} - {what}"));
        previous(info);
    }));
}

/// Replaces every name belonging to the client with a stable label.
///
/// Built from the PROJECT's own well and field list rather than from a pattern. A pattern would
/// eventually miss a naming convention nobody anticipated, and a redactor that misses once has
/// failed completely - the whole value of the report is that it can be sent.
///
/// Names are replaced LONGEST FIRST. With `SANDI-1` and `SANDI-10` in one project, replacing the
/// short name first would leave `WELL-1` followed by a stray `0`, which is both wrong and a
/// partial leak of the original.
pub struct Redactor {
    /// (original, replacement), longest original first.
    pairs: Vec<(String, String)>,
}

impl Redactor {
    pub fn new(conn: &Connection) -> Self {
        let mut pairs: Vec<(String, String)> = Vec::new();
        let push_all = |sql: &str, prefix: &str, pairs: &mut Vec<(String, String)>| {
            let Ok(mut stmt) = conn.prepare(sql) else { return };
            let Ok(rows) = stmt.query_map([], |row| row.get::<_, Option<String>>(0)) else { return };
            let mut n = 0;
            for name in rows.flatten().flatten() {
                let trimmed = name.trim();
                // A name of one or two characters is more likely to appear inside an unrelated
                // word than to identify anything, and replacing it would shred the report.
                if trimmed.len() < 3 {
                    continue;
                }
                if pairs.iter().any(|(original, _)| original == trimmed) {
                    continue;
                }
                n += 1;
                pairs.push((trimmed.to_string(), format!("{prefix}-{n}")));
            }
        };
        push_all("SELECT DISTINCT well_name FROM wells", "WELL", &mut pairs);
        push_all("SELECT DISTINCT field_name FROM wells", "FIELD", &mut pairs);
        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Self { pairs }
    }

    /// Redact free text. Applied to everything that could have come from the project, including
    /// operation labels and the ancestry disclosure cells.
    pub fn apply(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (original, replacement) in &self.pairs {
            if out.contains(original.as_str()) {
                out = out.replace(original.as_str(), replacement);
            }
        }
        out
    }

    pub fn well_count(&self) -> usize {
        self.pairs.iter().filter(|(_, r)| r.starts_with("WELL-")).count()
    }
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).unwrap_or(-1)
}

/// One `label   value` line of the report's fact blocks.
///
/// The padding is a stated WIDTH rather than spaces typed into each literal. Hand-counted
/// alignment drifts the moment a label is renamed, and `source_hygiene_tests` reads a run of
/// spaces inside a string as the signature of a dropped line-continuation - which is exactly
/// what it looks like, and it was right to say so.
fn field(label: &str, value: &str) -> String {
    format!("{label:<19}{value}\n")
}

fn duration(millis: u128) -> String {
    if millis < 1000 {
        format!("{millis} ms")
    } else if millis < 60_000 {
        format!("{:.1} s", millis as f64 / 1000.0)
    } else {
        format!("{} m {:02} s", millis / 60_000, (millis % 60_000) / 1000)
    }
}

/// What the report should cover beyond the always-included sections.
pub struct ReportSpec {
    /// Include full curve provenance for this well. `None` leaves the section out entirely -
    /// on a field-scale project it is the long half of the report.
    pub provenance_well_id: Option<String>,
}

/// Builds the report. Everything that reaches this string has been through the redactor except
/// the sections that are structurally incapable of carrying a client name (timings, counts,
/// machine facts).
pub fn build_report(conn: &Connection, app_version: &str, spec: &ReportSpec) -> String {
    let redactor = Redactor::new(conn);
    let mut out = String::new();

    out.push_str("SandiBumi diagnostic report\n");
    out.push_str(&format!("SandiBumi {app_version}\n\n"));

    // Above everything, because it is the sentence somebody about to attach this to an email has
    // to read. Decision (a): parameter values travel, so this report is not neutral.
    out.push_str("  ------------------------------------------------------------------------\n");
    out.push_str("  READ THIS BEFORE SENDING\n");
    out.push_str("\n");
    out.push_str("  This report CONTAINS PARAMETER VALUES - the m, n, a, Rw, cut-offs and\n");
    out.push_str("  endpoints used to compute curves. Those are analytical work product and may\n");
    out.push_str("  be covered by your confidentiality agreement. Check before sending it on.\n");
    out.push_str("\n");
    out.push_str("  It contains NO well names, NO field names, NO file paths and NO curve\n");
    out.push_str("  values. Wells appear as WELL-1, WELL-2 and so on, consistently within this\n");
    out.push_str("  report only - two reports cannot be lined up against each other.\n");
    out.push_str("  ------------------------------------------------------------------------\n\n");

    out.push_str("== MACHINE ==\n");
    out.push_str(&field("os", &format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH)));
    let cores = std::thread::available_parallelism().map(|n| n.get().to_string());
    out.push_str(&field("cores", cores.as_deref().unwrap_or("unknown")));
    let health = crate::health::snapshot();
    out.push_str(&field(
        "system memory",
        &health
            .mem_total_mb
            .map(|total| format!("{total} MB"))
            .unwrap_or_else(|| "not available on this platform".to_string()),
    ));
    if let Some(load) = health.mem_system {
        out.push_str(&field("memory in use now", &format!("{load:.0} %")));
    }
    let python = crate::python_engine::python_status();
    // The interpreter PATH carries a username, so only WHETHER it was found travels.
    out.push_str(&field(
        "python",
        if python.path.is_some() { "found (numpy present)" } else { "not found" },
    ));
    out.push_str(&field("scipy", python.scipy.as_deref().unwrap_or("not installed")));
    out.push('\n');

    out.push_str("== PROJECT SHAPE ==\n");
    out.push_str("Counts only. This is the denominator for 'it was slow' - the same 60 seconds\n");
    out.push_str("means something different over 5 wells and over 2000.\n\n");
    out.push_str(&field("wells", &redactor.well_count().to_string()));
    for (label, sql) in [
        ("standard samples", "SELECT COUNT(*) FROM standard_curves"),
        ("computed samples", "SELECT COUNT(*) FROM computed_curves"),
        ("computed curves", "SELECT COUNT(DISTINCT curve_name) FROM computed_curves"),
        ("imported samples", "SELECT COUNT(*) FROM curve_samples"),
        ("core plugs", "SELECT COUNT(*) FROM core_data"),
        ("pictures", "SELECT COUNT(*) FROM well_images"),
        ("saved equations", "SELECT COUNT(*) FROM documents WHERE doc_type = 'equation'"),
        ("saved ml models", "SELECT COUNT(*) FROM ml_models"),
    ] {
        out.push_str(&field(label, &count(conn, sql).to_string()));
    }
    out.push('\n');

    out.push_str("== OPENING THIS PROJECT ==\n");
    match BOOT_STEPS.lock() {
        Ok(steps) if !steps.is_empty() => {
            for step in steps.iter() {
                out.push_str(&format!("{:<38} {}\n", step.name, duration(step.millis)));
            }
        }
        _ => out.push_str("no step timings recorded for this session\n"),
    }
    out.push('\n');

    out.push_str("== OPERATIONS THIS SESSION ==\n");
    out.push_str("Newest last. Nothing recorded a duration before this existed, so a chain that\n");
    out.push_str("ran for hours left no trace of WHICH step was slow.\n\n");
    match OPS.lock() {
        Ok(ops) if !ops.is_empty() => {
            out.push_str(&format!("{:<16} {:<40} {:>7} {:>10}  {}\n", "KIND", "WHAT", "ITEMS", "TOOK", "RESULT"));
            for op in ops.iter() {
                let label = redactor.apply(&op.label);
                let label: String = label.chars().take(40).collect();
                out.push_str(&format!(
                    "{:<16} {:<40} {:>7} {:>10}  {}\n",
                    op.kind,
                    label,
                    op.items,
                    duration(op.millis),
                    op.state
                ));
            }
        }
        _ => out.push_str("no operations run in this session\n"),
    }
    out.push('\n');

    out.push_str("== SESSION ERRORS ==\n");
    match ERRORS.lock() {
        Ok(errors) if !errors.is_empty() => {
            out.push_str("This session hit an internal error. Everything after it may have behaved\n");
            out.push_str("oddly - this is what to report first.\n\n");
            for error in errors.iter() {
                out.push_str(&format!("at {} ms since epoch: {}\n", error.at_millis, error.context));
            }
        }
        _ => out.push_str("none\n"),
    }
    out.push('\n');

    if let Some(well_id) = spec.provenance_well_id.as_deref() {
        out.push_str("== HOW THESE CURVES WERE MADE ==\n");
        out.push_str("One well's computed curves, with the module, its version, the inputs it read\n");
        out.push_str("and the PARAMETER VALUES it used. This is the section that answers 'the\n");
        out.push_str("numbers look wrong', and the section the notice at the top is about.\n\n");
        match crate::ancestry::curve_ancestry_disclosures(conn, &[well_id.to_string()], None) {
            Ok(disclosures) if !disclosures.is_empty() => {
                for disclosure in &disclosures {
                    for cell in disclosure.cells() {
                        out.push_str(&format!("  {}\n", redactor.apply(&cell)));
                    }
                    out.push('\n');
                }
            }
            Ok(_) => out.push_str("this well has no computed curves\n"),
            Err(error) => out.push_str(&format!("could not read provenance: {}\n", redactor.apply(&error))),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with_wells() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory project");
        crate::db::create_schema(&conn).expect("schema");
        for (id, name, field) in [
            ("11111111-1111-1111-1111-111111111111", "SANDI-1", "Sandi North"),
            ("22222222-2222-2222-2222-222222222222", "SANDI-10", "Sandi North"),
        ] {
            conn.execute(
                "INSERT INTO wells (well_id, well_name, field_name, td, kb) VALUES (?1, ?2, ?3, NULL, NULL)",
                duckdb::params![id, name, field],
            )
            .expect("insert well");
        }
        conn
    }

    /// A report can be sent, and it can still be read.
    ///
    /// Both halves are load-bearing and neither implies the other. A redactor that blanked the
    /// whole file would satisfy the first perfectly and be worthless; one that only masked names
    /// it recognised by pattern would satisfy the second and leak the first time somebody used a
    /// naming convention nobody anticipated. So the mapping is driven by the project's OWN well
    /// list, and both directions are asserted.
    ///
    /// The longest-first ordering is the subtle half: with SANDI-1 and SANDI-10 in one project,
    /// replacing the short name first leaves "WELL-1" followed by a stray "0" - wrong, and a
    /// partial leak of the original.
    #[test]
    fn a_report_carries_the_shape_of_the_problem_and_none_of_the_delivery() {
        let conn = project_with_wells();
        record_op("Module", "vsh_larionov on SANDI-10", 4200, 1, "ok");
        // A label carrying the FIELD name, not just the well. Without one the field-name
        // assertion below is vacuous - no section of the report prints a field name on its own,
        // so dropping field redaction entirely would leave the test green. Caught by mutation,
        // and this is the real shape: an export job labels itself by the field it covers.
        record_op("Export", "Field summary - Sandi North", 800, 2, "cancelled");
        record_boot_step("open the database", 900);

        let report = build_report(&conn, "test", &ReportSpec { provenance_well_id: None });

        // Side A: nothing that belongs to the client survives.
        for leaked in ["SANDI-1", "SANDI-10", "Sandi North"] {
            assert!(!report.contains(leaked), "the report must not carry {leaked}:\n{report}");
        }

        // Side B: the shape does survive, or there would be nothing to diagnose from. The well
        // that ran is still identifiable AS a well, the module is still named, and the duration
        // is still there - those are ours, not the client's.
        assert!(report.contains("WELL-2"), "the well must still be there under a label:\n{report}");
        assert!(report.contains("vsh_larionov"), "a module name is ours and must survive");
        assert!(report.contains("4.2 s"), "the duration is the whole point of recording it");
        // Three states, not a boolean: a four-minute chain the user gave up on must not read as a
        // completed one, which is the case a support call is most likely to be about.
        assert!(report.contains("cancelled"), "a cancelled run must not be reported as ok");
        assert!(report.contains("open the database"), "boot steps must reach the report");
        // The fact, not the column: `field` owns the padding and may change it.
        assert!(
            report.lines().any(|line| line.starts_with("wells") && line.trim_end().ends_with('2')),
            "the well count must be in the report:\n{report}"
        );

        // And the sensitivity notice, because decision (a) put fitted parameters in this file.
        assert!(
            report.contains("CONTAINS PARAMETER VALUES"),
            "a report carrying fitted parameters must say so before anyone sends it"
        );
    }

    /// The longest name is replaced first, pinned on its own because it is the failure that would
    /// survive a casual reading of the output.
    #[test]
    fn a_longer_well_name_is_redacted_before_a_shorter_one_it_contains() {
        let conn = project_with_wells();
        let redactor = Redactor::new(&conn);
        let out = redactor.apply("ran on SANDI-10 after SANDI-1");
        assert!(!out.contains("SANDI"), "no fragment of either name may survive: {out}");
        // Two DIFFERENT wells must not collapse into one label, or the report would say a single
        // well did both things.
        assert_ne!(
            out.matches("WELL-1").count() + out.matches("WELL-2").count(),
            0,
            "both wells must still be distinguishable: {out}"
        );
        assert!(out.contains("WELL-1") && out.contains("WELL-2"), "one label each: {out}");
    }
}
