//! Office deliverables — the study as a formatted Excel workbook.
//!
//! `export.rs` writes LAS, `composite.rs`/`report.rs` write PDF and SVG. Everything else a
//! finished study produces (pay summary, zone parameters, field roll-up) left the app as flat
//! CSV or as numbers on a printed page, so the last mile — the table an asset team actually
//! works in — was re-typed by hand. This module closes that.
//!
//! **Rule 7 holds throughout**: the workbook is written by a Python subprocess (`xlsxwriter`),
//! never an embedded interpreter, so a machine without it loses this one button and nothing
//! else. The native PDF/SVG/LAS paths stay the default deliverables.
//!
//! The runner is deliberately DUMB. Every petrophysical decision — what a blank means, which
//! average is net-weighted, which rows are trustworthy — is made here in Rust and arrives as a
//! plain [`Sheet`] of typed [`Cell`]s. The Python side only knows how to draw a table, so a
//! reader comparing the workbook to the report is comparing two renderings of one decision,
//! not two independent implementations that could drift.
//!
//! Two rules govern the numbers themselves:
//!
//! * **Numbers stay numbers.** A cell carries the value with a number *format*, never a
//!   preformatted string. A workbook whose columns are text cannot be pivoted, re-averaged or
//!   charted, which is the whole reason to want one.
//! * **A blank is not a zero.** Where `n_classified == 0` the well was never interpreted over
//!   that zone — VSH/PHIE/SWE resolved to NaN everywhere — and its net, N/G and HPV are
//!   exactly 0 for want of an answer, not because the zone is wet. The PDF prints "-" there
//!   (`report.rs`); the workbook leaves the cell EMPTY, which is the spreadsheet equivalent:
//!   Excel's own AVERAGE and COUNT skip a blank and would have been dragged toward zero by a 0.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::python_engine::{find_python, hide_console};
use crate::units;
use crate::workflow::{run_pay_summary, PaySummaryRequest, PaySummaryRow};

/// Flag order as the pay summary itself reports it: progressively stricter cutoffs.
const FLAG_ORDER: [&str; 3] = ["SAND", "RESERVOIR", "PAY"];

// ---------------------------------------------------------------------------
// Which office packages this machine actually has
// ---------------------------------------------------------------------------

/// What the discovered interpreter can write. Asked once when a deliverables dialog opens, so
/// a button that cannot work says why instead of failing at save time.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OfficeSupport {
    /// The interpreter SandiBumi found, for the "install into THIS python" message.
    pub python: Option<String>,
    pub xlsxwriter: bool,
    pub docx: bool,
    pub pptx: bool,
    pub openpyxl: bool,
}

/// One subprocess for all four packages: starting Python is the expensive part, and asking
/// four times over would cost four times as much to learn one answer.
const SUPPORT_PROBE: &str = r#"
import json, importlib
out = {}
for key, mod in (("xlsxwriter", "xlsxwriter"), ("docx", "docx"), ("pptx", "pptx"), ("openpyxl", "openpyxl")):
    try:
        importlib.import_module(mod)
        out[key] = True
    except Exception:
        out[key] = False
print(json.dumps(out))
"#;

#[derive(Deserialize, Default)]
struct ProbeReply {
    #[serde(default)]
    xlsxwriter: bool,
    #[serde(default)]
    docx: bool,
    #[serde(default)]
    pptx: bool,
    #[serde(default)]
    openpyxl: bool,
}

pub fn office_support() -> OfficeSupport {
    let Some(python) = find_python() else { return OfficeSupport::default() };
    let mut support = OfficeSupport {
        python: Some(python.to_string_lossy().to_string()),
        ..Default::default()
    };
    let mut cmd = Command::new(&python);
    cmd.args(["-c", SUPPORT_PROBE]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    hide_console(&mut cmd);
    if let Ok(out) = cmd.output() {
        if let Ok(reply) = serde_json::from_slice::<ProbeReply>(&out.stdout) {
            support.xlsxwriter = reply.xlsxwriter;
            support.docx = reply.docx;
            support.pptx = reply.pptx;
            support.openpyxl = reply.openpyxl;
        }
    }
    support
}

// ---------------------------------------------------------------------------
// The sheet model handed to the runner
// ---------------------------------------------------------------------------

/// How a column's numbers are DISPLAYED. The stored value is always the real one — a percent
/// format shows 0.185 as 18.5% while the cell still holds 0.185, so a client's own formula
/// against the column gets the fraction it expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CellFormat {
    Text,
    Int,
    Num1,
    Num2,
    Num3,
}

#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub header: String,
    /// Width in Excel character units.
    pub width: f64,
    pub fmt: CellFormat,
}

impl Column {
    fn new(header: &str, width: f64, fmt: CellFormat) -> Self {
        Column { header: header.to_string(), width, fmt }
    }
}

/// One cell. `Blank` is a first-class value, not an absence: it is how "this was never
/// interpreted" is stated, and it must survive to the sheet as a genuinely empty cell.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Cell {
    Num(f64),
    Text(String),
    Blank,
}

/// A finite measurement becomes a number; NaN — the project's one missing-value marker — is a
/// blank. Nothing in this module ever writes -999.25 or 0 for a missing sample.
fn num(v: f32) -> Cell {
    if v.is_finite() {
        Cell::Num(v as f64)
    } else {
        Cell::Blank
    }
}

fn numf(v: f64) -> Cell {
    if v.is_finite() {
        Cell::Num(v)
    } else {
        Cell::Blank
    }
}

fn text(s: impl Into<String>) -> Cell {
    Cell::Text(s.into())
}

/// Tint every row whose `col` reads `equals` — used to lift the PAY rows out of a sheet that
/// also carries SAND and RESERVOIR.
#[derive(Debug, Clone, Serialize)]
pub struct Shade {
    pub col: usize,
    pub equals: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sheet {
    pub name: String,
    pub title: String,
    pub notes: Vec<String>,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Cell>>,
    /// Freeze below the header so the column names stay on screen at field scale.
    pub freeze: bool,
    pub autofilter: bool,
    pub shade: Option<Shade>,
}

impl Sheet {
    fn new(name: &str, title: &str, columns: Vec<Column>) -> Self {
        Sheet {
            name: name.to_string(),
            title: title.to_string(),
            notes: Vec::new(),
            columns,
            rows: Vec::new(),
            freeze: true,
            autofilter: true,
            shade: None,
        }
    }
}

// ---------------------------------------------------------------------------
// The xlsxwriter runner
// ---------------------------------------------------------------------------

/// Draws a table and nothing else. It receives typed cells and column formats; it decides no
/// petrophysics. `None` reaches it as JSON `null` and is skipped entirely, so a blank stays
/// blank rather than becoming 0.
const XLSX_RUNNER: &str = r##"
import json, sys

try:
    import xlsxwriter
    from xlsxwriter.utility import xl_col_to_name
except Exception:
    sys.stderr.write("xlsxwriter-missing\n")
    sys.exit(2)

# sys.stdin.buffer, never sys.stdin: a piped child's text stdin decodes with the Windows
# ANSI codepage (cp1252 here), and serde_json emits raw UTF-8. A well named "Bekasap-1"
# with an en dash, an Indonesian field name, or the middot in the cover line would arrive
# mojibake in a client document. json.loads accepts bytes and assumes UTF-8, which is what
# was actually sent. (ml.rs and python_engine.rs already read stdin this way.)
req = json.loads(sys.stdin.buffer.read())
wb = xlsxwriter.Workbook(req["dest"], {"constant_memory": False})

NUMFMT = {
    "text": None,
    "int": "#,##0",
    "num1": "#,##0.0",
    "num2": "#,##0.00",
    "num3": "0.000",
}
cache = {}
def numeric(key):
    if key not in cache:
        nf = NUMFMT.get(key)
        cache[key] = wb.add_format({"num_format": nf} if nf else {})
    return cache[key]

title_f = wb.add_format({"bold": True, "font_size": 13})
note_f = wb.add_format({"italic": True, "font_color": "#5A6572"})
head_f = wb.add_format({"bold": True, "bg_color": "#E8EEF6", "bottom": 1, "text_wrap": True, "valign": "vcenter"})
str_f = wb.add_format({})
shade_f = wb.add_format({"bg_color": "#FFF3CD"})

written = 0
for sh in req["sheets"]:
    ws = wb.add_worksheet(sh["name"][:31])
    r = 0
    if sh.get("title"):
        ws.write_string(r, 0, sh["title"], title_f)
        r += 1
    for n in sh.get("notes") or []:
        ws.write_string(r, 0, n, note_f)
        r += 1
    if r:
        r += 1  # one blank line between the banner and the table
    head = r
    cols = sh["columns"]
    for c, col in enumerate(cols):
        ws.write_string(head, c, col["header"], head_f)
        ws.set_column(c, c, col.get("width", 12))
    r = head + 1
    for row in sh["rows"]:
        for c, v in enumerate(row):
            if v is None:
                continue  # a blank is a statement, not a zero - leave the cell empty
            if isinstance(v, str):
                ws.write_string(r, c, v, str_f)
            elif isinstance(v, bool):
                ws.write_string(r, c, str(v), str_f)
            else:
                ws.write_number(r, c, v, numeric(cols[c]["fmt"] if c < len(cols) else "num2"))
        r += 1
    n = len(sh["rows"])
    if sh.get("freeze"):
        ws.freeze_panes(head + 1, 0)
    if sh.get("autofilter") and n and cols:
        ws.autofilter(head, 0, head + n, len(cols) - 1)
    shade = sh.get("shade")
    if shade and n and cols:
        letter = xl_col_to_name(shade["col"])
        ws.conditional_format(head + 1, 0, head + n, len(cols) - 1, {
            "type": "formula",
            "criteria": '=${0}{1}="{2}"'.format(letter, head + 2, shade["equals"]),
            "format": shade_f,
        })
    written += 1

wb.close()
print(json.dumps({"ok": True, "sheets": written}))
"##;

#[derive(Deserialize)]
struct RunnerReply {
    #[serde(default)]
    sheets: usize,
}

/// Hands the sheets to xlsxwriter, which writes straight to `dest`.
///
/// The file is written by the runner rather than piped back as bytes: a field workbook is
/// megabytes, and a pipe would carry every one of them through this process for no gain.
fn write_workbook(sheets: &[Sheet], dest: &str) -> Result<usize, String> {
    let python = find_python().ok_or_else(|| {
        "no Python found - install Python 3.10+ with xlsxwriter, or set ARSHILLA_PYTHON".to_string()
    })?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", XLSX_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    let req = serde_json::json!({ "dest": dest, "sheets": sheets });
    {
        let stdin = child.stdin.as_mut().ok_or("python stdin closed")?;
        stdin.write_all(req.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }
    // Dropping the handle closes the pipe, which is how the runner sees end-of-input.
    drop(child.stdin.take());
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("workbook write failed");
        return Err(if last.contains("xlsxwriter-missing") {
            "xlsxwriter is not installed in the Python SandiBumi found (pip install xlsxwriter)".to_string()
        } else {
            last.trim().to_string()
        });
    }
    let reply: RunnerReply =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad workbook reply: {e}"))?;
    Ok(reply.sheets)
}

// ---------------------------------------------------------------------------
// The workbook itself
// ---------------------------------------------------------------------------

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkbookSpec {
    pub well_ids: Vec<String>,
    /// Pay cutoffs, pay-summary convention (see `cutoffs.ts` for the single source of defaults).
    pub vsh_max: f64,
    pub phie_min: f64,
    pub swe_max: f64,
    #[serde(default)]
    pub perm_min: Option<f64>,
    #[serde(default)]
    pub title: String,
    #[serde(default = "yes")]
    pub include_pay: bool,
    #[serde(default = "yes")]
    pub include_field: bool,
    #[serde(default = "yes")]
    pub include_zone_params: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkbookResult {
    pub path: String,
    pub sheets: usize,
    /// Wells asked for.
    pub wells: usize,
    /// Wells that produced at least one interpreted zone row.
    pub wells_with_results: usize,
    pub pay_rows: usize,
    pub bytes: u64,
}

/// A row is only evidence if the classifier could judge at least one sample in it. Everything
/// downstream — the field roll-up, the "wells with results" count — filters on this, because a
/// well whose VSH/PHIE/SWE were never computed reports a perfect zero for net, N/G and HPV.
fn interpreted(r: &PaySummaryRow) -> bool {
    r.n_classified > 0
}

/// Sheet 2: every well × zone × flag, exactly the table `report.rs` prints — same rows, same
/// conventions, same "not interpreted" rule — so the workbook and the client PDF can never
/// disagree about a number.
pub fn pay_sheet(rows: &[PaySummaryRow], unit: &str) -> Sheet {
    let mut sheet = Sheet::new(
        "Pay Summary",
        "Pay summary by well and zone",
        vec![
            Column::new("Well", 22.0, CellFormat::Text),
            Column::new("Zone", 16.0, CellFormat::Text),
            Column::new("Flag", 12.0, CellFormat::Text),
            Column::new(&format!("Top ({unit})"), 11.0, CellFormat::Num1),
            Column::new(&format!("Bottom ({unit})"), 12.0, CellFormat::Num1),
            Column::new(&format!("Gross ({unit})"), 11.0, CellFormat::Num1),
            Column::new(&format!("Net ({unit})"), 11.0, CellFormat::Num1),
            Column::new("N/G", 9.0, CellFormat::Num2),
            Column::new("VSH (v/v)", 10.0, CellFormat::Num2),
            Column::new("PHIE (v/v)", 11.0, CellFormat::Num3),
            Column::new("SWE (v/v)", 10.0, CellFormat::Num2),
            Column::new(&format!("HPV ({unit})"), 11.0, CellFormat::Num2),
            Column::new("Samples", 10.0, CellFormat::Int),
        ],
    );
    sheet.notes.push(
        "Fractions are v/v, matching the report PDF. A BLANK means the well was not interpreted over \
         that zone - it is not a zero, and Excel's AVERAGE/COUNT skip it."
            .into(),
    );
    sheet.shade = Some(Shade { col: 2, equals: "PAY".into() });
    for r in rows {
        // gross is geometry - it is known whether or not anything was interpreted. Everything
        // else is a result, and a result nobody computed is blank.
        let judged = interpreted(r);
        sheet.rows.push(vec![
            text(&r.well_name),
            text(&r.zone),
            text(&r.flag),
            num(r.top),
            num(r.bottom),
            num(r.gross),
            if judged { num(r.net) } else { Cell::Blank },
            if judged { num(r.ntg) } else { Cell::Blank },
            if judged { num(r.avg_vsh) } else { Cell::Blank },
            if judged { num(r.avg_phie) } else { Cell::Blank },
            if judged { num(r.avg_swe) } else { Cell::Blank },
            if judged { num(r.hpv) } else { Cell::Blank },
            Cell::Num(r.n_classified as f64),
        ]);
    }
    sheet
}

/// Sheet 3: the field roll-up per zone and flag.
///
/// Two N/G columns on purpose, because they answer different questions and quoting one as the
/// other is a real reservoir error: **N/G (field)** is Σnet / Σgross, the volumetric ratio that
/// belongs in a resource calculation, while **Mean N/G** is the plain average of the per-well
/// values, which is what the Field Dashboard plots and what a well-count-weighted statement
/// means. A thick 0.2 well and a thin 0.9 well give very different answers.
///
/// PHIE and SWE are **net-weighted** (`Σ v·net / Σ net`), the same weighting the dashboard uses:
/// a mean of per-well means would let a 2 m sliver count as much as a 40 m sand.
pub fn field_sheet(rows: &[PaySummaryRow], unit: &str) -> Sheet {
    let mut sheet = Sheet::new(
        "Field Summary",
        "Field roll-up by zone",
        vec![
            Column::new("Zone", 16.0, CellFormat::Text),
            Column::new("Flag", 12.0, CellFormat::Text),
            Column::new("Wells", 8.0, CellFormat::Int),
            Column::new("Not interpreted", 15.0, CellFormat::Int),
            Column::new(&format!("Σ Gross ({unit})"), 13.0, CellFormat::Num1),
            Column::new(&format!("Σ Net ({unit})"), 12.0, CellFormat::Num1),
            Column::new("N/G (field)", 12.0, CellFormat::Num2),
            Column::new("Mean N/G", 11.0, CellFormat::Num2),
            Column::new("PHIE (net-wtd)", 14.0, CellFormat::Num3),
            Column::new("SWE (net-wtd)", 13.0, CellFormat::Num2),
            Column::new(&format!("Σ HPV ({unit})"), 13.0, CellFormat::Num2),
        ],
    );
    sheet.notes.push(
        "N/G (field) = sum(net)/sum(gross), the volumetric ratio. Mean N/G is the average of the \
         per-well values, as the Field Dashboard shows it. PHIE and SWE are net-weighted."
            .into(),
    );
    sheet.notes.push(
        "Wells counts only the wells that were interpreted in that zone; the ones that were not \
         are counted separately and contribute to nothing."
            .into(),
    );
    sheet.shade = Some(Shade { col: 1, equals: "PAY".into() });

    // Shallow to deep: zones are ordered by their mean top depth, because that is how a
    // petrophysicist reads a field table, not alphabetically.
    let mut zones: Vec<(String, f64)> = Vec::new();
    for r in rows {
        if let Some(entry) = zones.iter_mut().find(|(z, _)| *z == r.zone) {
            entry.1 = entry.1.min(r.top as f64);
        } else {
            zones.push((r.zone.clone(), r.top as f64));
        }
    }
    zones.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));

    for (zone, _) in &zones {
        for flag in FLAG_ORDER {
            let group: Vec<&PaySummaryRow> =
                rows.iter().filter(|r| &r.zone == zone && r.flag == flag).collect();
            if group.is_empty() {
                continue;
            }
            let judged: Vec<&&PaySummaryRow> = group.iter().filter(|r| interpreted(r)).collect();
            let blind = group.len() - judged.len();
            let mut wells: Vec<&str> = judged.iter().map(|r| r.well_id.as_str()).collect();
            wells.sort_unstable();
            wells.dedup();

            let sum = |f: &dyn Fn(&PaySummaryRow) -> f32| -> f64 {
                judged.iter().filter_map(|r| { let v = f(r); v.is_finite().then_some(v as f64) }).sum()
            };
            let sum_gross = sum(&|r| r.gross);
            let sum_net = sum(&|r| r.net);
            let sum_hpv = sum(&|r| r.hpv);
            let mean_ntg = {
                let vals: Vec<f64> =
                    judged.iter().filter_map(|r| r.ntg.is_finite().then_some(r.ntg as f64)).collect();
                if vals.is_empty() { f64::NAN } else { vals.iter().sum::<f64>() / vals.len() as f64 }
            };
            let net_weighted = |f: &dyn Fn(&PaySummaryRow) -> f32| -> f64 {
                let mut wsum = 0.0;
                let mut w = 0.0;
                for r in &judged {
                    let v = f(r);
                    // A zero-net row carries no weight, so it neither helps nor hurts - the same
                    // rule the dashboard's weightedMean applies.
                    if v.is_finite() && r.net.is_finite() && r.net > 0.0 {
                        wsum += v as f64 * r.net as f64;
                        w += r.net as f64;
                    }
                }
                if w > 0.0 { wsum / w } else { f64::NAN }
            };

            sheet.rows.push(vec![
                text(zone),
                text(flag),
                Cell::Num(wells.len() as f64),
                Cell::Num(blind as f64),
                if judged.is_empty() { Cell::Blank } else { numf(sum_gross) },
                if judged.is_empty() { Cell::Blank } else { numf(sum_net) },
                if sum_gross > 0.0 { numf(sum_net / sum_gross) } else { Cell::Blank },
                numf(mean_ntg),
                numf(net_weighted(&|r| r.avg_phie)),
                numf(net_weighted(&|r| r.avg_swe)),
                if judged.is_empty() { Cell::Blank } else { numf(sum_hpv) },
            ]);
        }
    }
    sheet
}

/// Sheet 4: the interval parameters an interpretation actually used. Long format (one row per
/// parameter) rather than a wide grid, because which parameters a zone carries varies from
/// well to well and a union of every column would be mostly empty.
fn zone_param_sheet(conn: &Connection, wells: &[(String, String)]) -> Result<Sheet, String> {
    let mut sheet = Sheet::new(
        "Zone Parameters",
        "Interval parameters by well and zone",
        vec![
            Column::new("Well", 22.0, CellFormat::Text),
            Column::new("Zone", 16.0, CellFormat::Text),
            Column::new("Parameter", 20.0, CellFormat::Text),
            Column::new("Value", 14.0, CellFormat::Num3),
            Column::new("Text", 22.0, CellFormat::Text),
        ],
    );
    sheet.notes.push(
        "Zone * holds the whole-well default, which a named zone overrides. Values are stored \
         exactly as the interpretation used them."
            .into(),
    );
    for (well_id, well_name) in wells {
        let entries = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;
        for e in entries {
            sheet.rows.push(vec![
                text(well_name),
                text(&e.zone_name),
                text(&e.param_name),
                e.value_num.map(num).unwrap_or(Cell::Blank),
                e.value_text.map(text).unwrap_or(Cell::Blank),
            ]);
        }
    }
    Ok(sheet)
}

/// Sheet 1: what this workbook is and what was fed to it. A deliverable that cannot say which
/// cutoffs produced it is not auditable, and a client will ask.
fn summary_sheet(
    spec: &WorkbookSpec,
    stamp: &str,
    unit: &str,
    wells: &[(String, String)],
    without: &[&str],
) -> Sheet {
    let mut sheet = Sheet::new(
        "Summary",
        if spec.title.trim().is_empty() { "Petrophysical summary" } else { spec.title.trim() },
        vec![Column::new("Item", 28.0, CellFormat::Text), Column::new("Value", 46.0, CellFormat::Num3)],
    );
    sheet.freeze = false;
    sheet.autofilter = false;
    let mut row = |k: &str, v: Cell| sheet.rows.push(vec![text(k), v]);
    row("Exported by", text(format!("SandiBumi {}", env!("CARGO_PKG_VERSION"))));
    row("Exported", text(stamp));
    row("Depth unit", text(unit));
    row("Wells requested", Cell::Num(wells.len() as f64));
    row("Wells with results", Cell::Num((wells.len() - without.len()) as f64));
    row("", Cell::Blank);
    row("Cutoff - VSH max (v/v)", Cell::Num(spec.vsh_max));
    row("Cutoff - PHIE min (v/v)", Cell::Num(spec.phie_min));
    row("Cutoff - SWE max (v/v)", Cell::Num(spec.swe_max));
    match spec.perm_min {
        Some(p) => row("Cutoff - PERM min (mD)", Cell::Num(p)),
        None => row("Cutoff - PERM min (mD)", text("not applied")),
    }
    row("", Cell::Blank);
    row(
        "Blank cells",
        text("A blank means the well was not interpreted over that zone - it is not a zero."),
    );
    if !without.is_empty() {
        row("", Cell::Blank);
        for name in without {
            row("Well without results", text(*name));
        }
    }
    sheet
}

/// Builds the workbook and writes it to `dest`.
pub fn export_workbook(
    db_lock: &Mutex<Connection>,
    spec: &WorkbookSpec,
    dest: &str,
) -> Result<WorkbookResult, String> {
    if spec.well_ids.is_empty() {
        return Err("no wells in scope".into());
    }

    // `stats_only`: an export must not churn the project. Writing FLAG_* curves (and a log-set
    // version per well) as a side effect of saving a spreadsheet would bloat the file and put a
    // fake interpretation event in the history. Persisting flags stays the job of the explicit
    // Cutoffs & Summary run - the same reasoning the Field Dashboard follows.
    let pay_rows = if spec.include_pay || spec.include_field {
        run_pay_summary(
            db_lock,
            &PaySummaryRequest {
                well_ids: spec.well_ids.clone(),
                vsh_max: spec.vsh_max,
                phie_min: spec.phie_min,
                swe_max: spec.swe_max,
                perm_min: spec.perm_min,
                skip_version: true,
                stats_only: true,
            },
        )?
    } else {
        Vec::new()
    };

    let (stamp, unit, wells) = {
        let conn = db_lock.lock().map_err(|e| e.to_string())?;
        let stamp: String = conn
            .query_row("SELECT strftime(now(), '%Y-%m-%d %H:%M')", [], |r| r.get(0))
            .unwrap_or_else(|_| String::new());
        let unit = units::project_depth_unit_or_default(&conn).label().to_string();
        let mut wells: Vec<(String, String)> = Vec::with_capacity(spec.well_ids.len());
        for id in &spec.well_ids {
            let name: String = conn
                .query_row("SELECT well_name FROM wells WHERE well_id = ?1", params![id], |r| r.get(0))
                .unwrap_or_else(|_| id.clone());
            wells.push((id.clone(), name));
        }
        (stamp, unit, wells)
    };

    let with_results: Vec<&str> = {
        let mut v: Vec<&str> =
            pay_rows.iter().filter(|r| interpreted(r)).map(|r| r.well_id.as_str()).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let without: Vec<&str> = wells
        .iter()
        .filter(|(id, _)| !with_results.contains(&id.as_str()))
        .map(|(_, name)| name.as_str())
        .collect();

    let mut sheets = vec![summary_sheet(spec, &stamp, &unit, &wells, &without)];
    if spec.include_pay {
        sheets.push(pay_sheet(&pay_rows, &unit));
    }
    if spec.include_field {
        sheets.push(field_sheet(&pay_rows, &unit));
    }
    if spec.include_zone_params {
        let conn = db_lock.lock().map_err(|e| e.to_string())?;
        sheets.push(zone_param_sheet(&conn, &wells)?);
    }

    let written = write_workbook(&sheets, dest)?;
    let bytes = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    Ok(WorkbookResult {
        path: dest.to_string(),
        sheets: written,
        wells: wells.len(),
        wells_with_results: with_results.len(),
        pay_rows: pay_rows.len(),
        bytes,
    })
}

// ---------------------------------------------------------------------------
// The editable Word twin of report.rs's PDF
// ---------------------------------------------------------------------------

/// One element of a document. Deliberately built on the SAME [`Sheet`] the workbook uses:
/// a table is defined once and rendered twice, so the workbook, the Word report and the PDF
/// cannot quote three different versions of one number.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    Cover {
        title: String,
        well: String,
        field: String,
        meta: String,
        author: String,
    },
    Heading {
        text: String,
    },
    Para {
        text: String,
    },
    Table {
        sheet: Sheet,
        /// What an unmeasured cell reads as. **This is where the Word twin deliberately
        /// diverges from the workbook**: a spreadsheet leaves the cell EMPTY because Excel's
        /// own arithmetic skips a blank, but a document has no arithmetic — a reader's eye
        /// needs a mark, so it prints the same "-" the PDF does. Same decision, two correct
        /// renderings.
        blank_text: String,
    },
    PageBreak,
}

const BLANK_MARK: &str = "-";

/// Renders blocks with python-docx. As dumb as the xlsxwriter runner, and for the same reason.
const DOCX_RUNNER: &str = r##"
import json, sys

try:
    from docx import Document
    from docx.shared import Pt, RGBColor
    from docx.enum.text import WD_ALIGN_PARAGRAPH
    from docx.enum.table import WD_TABLE_ALIGNMENT
    from docx.oxml.ns import qn
    from docx.oxml import OxmlElement
except Exception:
    sys.stderr.write("docx-missing\n")
    sys.exit(2)

# sys.stdin.buffer, never sys.stdin: a piped child's text stdin decodes with the Windows
# ANSI codepage (cp1252 here), and serde_json emits raw UTF-8. A well named "Bekasap-1"
# with an en dash, an Indonesian field name, or the middot in the cover line would arrive
# mojibake in a client document. json.loads accepts bytes and assumes UTF-8, which is what
# was actually sent. (ml.rs and python_engine.rs already read stdin this way.)
req = json.loads(sys.stdin.buffer.read())
doc = Document()

DECIMALS = {"int": 0, "num1": 1, "num2": 2, "num3": 3}

def fmt(value, key, blank):
    if value is None:
        return blank
    if isinstance(value, str):
        return value
    d = DECIMALS.get(key)
    if d is None:
        return str(value)
    return "{0:,.{1}f}".format(value, d) if d else "{0:,.0f}".format(value)

def shade(cell, hexcolor):
    tcPr = cell._tc.get_or_add_tcPr()
    el = OxmlElement("w:shd")
    el.set(qn("w:val"), "clear")
    el.set(qn("w:color"), "auto")
    el.set(qn("w:fill"), hexcolor)
    tcPr.append(el)

def run(par, text, bold=False, size=None, color=None, italic=False):
    r = par.add_run(text)
    r.bold = bold
    r.italic = italic
    if size:
        r.font.size = Pt(size)
    if color:
        r.font.color.rgb = RGBColor.from_string(color)
    return r

tables = 0
for b in req["blocks"]:
    kind = b["kind"]
    if kind == "cover":
        for _ in range(6):
            doc.add_paragraph()
        p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
        run(p, b["title"], bold=True, size=22)
        p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
        run(p, b["well"], bold=True, size=16)
        if b.get("field"):
            p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
            run(p, b["field"], size=12, color="333333")
        if b.get("meta"):
            p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
            run(p, b["meta"], size=10, color="555555")
        for _ in range(8):
            doc.add_paragraph()
        if b.get("author"):
            p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
            run(p, b["author"], size=11, color="333333")
        p = doc.add_paragraph(); p.alignment = WD_ALIGN_PARAGRAPH.CENTER
        run(p, "Made in SandiBumi", size=9, color="777777")
    elif kind == "heading":
        doc.add_heading(b["text"], level=1)
    elif kind == "para":
        p = doc.add_paragraph()
        run(p, b["text"], size=9, color="5A6572", italic=True)
    elif kind == "page_break":
        doc.add_page_break()
    elif kind == "table":
        sheet = b["sheet"]
        blank = b.get("blank_text", "-")
        if sheet.get("title"):
            doc.add_heading(sheet["title"], level=1)
        for n in sheet.get("notes") or []:
            p = doc.add_paragraph()
            run(p, n, size=8, color="5A6572", italic=True)
        cols = sheet["columns"]
        t = doc.add_table(rows=1, cols=len(cols))
        t.style = "Table Grid"
        t.alignment = WD_TABLE_ALIGNMENT.CENTER
        for c, col in enumerate(cols):
            cell = t.rows[0].cells[c]
            cell.text = ""
            run(cell.paragraphs[0], col["header"], bold=True, size=8)
            shade(cell, "E8EEF6")
        sh = sheet.get("shade")
        for row in sheet["rows"]:
            cells = t.add_row().cells
            lit = bool(sh) and sh["col"] < len(row) and row[sh["col"]] == sh["equals"]
            for c, v in enumerate(row):
                if c >= len(cols):
                    break
                cells[c].text = ""
                run(cells[c].paragraphs[0], fmt(v, cols[c]["fmt"], blank), size=8)
                if lit:
                    shade(cells[c], "FFF3CD")
        tables += 1

doc.save(req["dest"])
print(json.dumps({"ok": True, "tables": tables, "blocks": len(req["blocks"])}))
"##;

#[derive(Deserialize)]
struct DocxReply {
    #[serde(default)]
    tables: usize,
}

fn write_docx(blocks: &[Block], dest: &str) -> Result<usize, String> {
    let python = find_python().ok_or_else(|| {
        "no Python found - install Python 3.10+ with python-docx, or set ARSHILLA_PYTHON".to_string()
    })?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", DOCX_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    let req = serde_json::json!({ "dest": dest, "blocks": blocks });
    {
        let stdin = child.stdin.as_mut().ok_or("python stdin closed")?;
        stdin.write_all(req.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("document write failed");
        return Err(if last.contains("docx-missing") {
            "python-docx is not installed in the Python SandiBumi found (pip install python-docx)".to_string()
        } else {
            last.trim().to_string()
        });
    }
    let reply: DocxReply =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad document reply: {e}"))?;
    Ok(reply.tables)
}

fn table(sheet: Sheet) -> Block {
    Block::Table { sheet, blank_text: BLANK_MARK.to_string() }
}

/// The zone-parameter table in the REPORT's shape (zone/top/bottom repeated only on the first
/// parameter of each zone), not the workbook's flat one — this document is the twin of the
/// PDF, so it must read like the PDF.
fn report_zone_sheet(conn: &Connection, well_id: &str, unit: &str) -> Result<Sheet, String> {
    let zones = db::list_zones(conn, well_id).map_err(|e| e.to_string())?;
    let zparams = db::list_zone_params(conn, well_id).map_err(|e| e.to_string())?;
    let mut sheet = Sheet::new(
        "Zone Parameters",
        "Zone Parameters",
        vec![
            Column::new("Zone", 18.0, CellFormat::Text),
            Column::new(&format!("Top ({unit})"), 12.0, CellFormat::Num1),
            Column::new(&format!("Bottom ({unit})"), 12.0, CellFormat::Num1),
            Column::new("Parameter", 20.0, CellFormat::Text),
            Column::new("Value", 14.0, CellFormat::Text),
        ],
    );
    for z in &zones {
        let params: Vec<&db::ZoneParamEntry> =
            zparams.iter().filter(|p| p.zone_name == z.zone_name).collect();
        // A zone with no parameters is still LISTED. Dropping it would make the document say the
        // zone was not evaluated, when in truth it simply took the defaults.
        if params.is_empty() {
            sheet.rows.push(vec![
                text(&z.zone_name),
                num(z.top_depth),
                num(z.bottom_depth),
                Cell::Blank,
                Cell::Blank,
            ]);
        }
        for (i, p) in params.iter().enumerate() {
            let value = p
                .value_num
                .map(|v| format!("{v}"))
                .or_else(|| p.value_text.clone())
                .map(text)
                .unwrap_or(Cell::Blank);
            sheet.rows.push(vec![
                if i == 0 { text(&z.zone_name) } else { Cell::Blank },
                if i == 0 { num(z.top_depth) } else { Cell::Blank },
                if i == 0 { num(z.bottom_depth) } else { Cell::Blank },
                text(&p.param_name),
                value,
            ]);
        }
    }
    Ok(sheet)
}

/// Assembles the document. Pure enough to test: everything petrophysical is decided here and
/// the runner only draws it.
pub fn build_report_blocks(
    db_lock: &Mutex<Connection>,
    spec: &crate::report::ReportSpec,
) -> Result<(Vec<Block>, String), String> {
    let well_id = spec.composite.well_id.clone();

    // `stats_only`: same rule as the workbook — exporting a document must not write FLAG curves
    // or version a log set. (The PDF path persists them in place; that is its long-standing
    // behaviour and is left alone, but a NEW export has no business changing the project.)
    let pay_rows = run_pay_summary(
        db_lock,
        &PaySummaryRequest {
            well_ids: vec![well_id.clone()],
            vsh_max: spec.vsh_max,
            phie_min: spec.phie_min,
            swe_max: spec.swe_max,
            perm_min: spec.perm_min,
            skip_version: true,
            stats_only: true,
        },
    )
    .unwrap_or_default();

    let conn = db_lock.lock().map_err(|e| e.to_string())?;
    let header = crate::composite::fetch_header(&conn, &well_id)?;
    let unit = units::project_depth_unit_or_default(&conn).label().to_string();
    let zones = db::list_zones(&conn, &well_id).map_err(|e| e.to_string())?;

    // The evaluated interval comes from the well's own zones rather than from a composite
    // render: this document carries no log plots, so paginating one just to print a depth
    // range would be minutes of work for one line of text.
    let interval = match (zones.first(), zones.last()) {
        (Some(a), Some(b)) => format!("Zoned interval: {:.1} - {:.1} {unit}", a.top_depth, b.bottom_depth),
        _ => String::new(),
    };
    let mut meta = interval;
    if let Some(td) = header.td {
        if !meta.is_empty() {
            meta.push_str("   ·   ");
        }
        meta.push_str(&format!("TD: {td:.1} {unit}"));
    }
    if let Some(kb) = header.kb {
        if !meta.is_empty() {
            meta.push_str("   ·   ");
        }
        meta.push_str(&format!("KB: {kb:.1} {unit}"));
    }

    let mut blocks = vec![
        Block::Cover {
            title: if spec.title.trim().is_empty() { "Petrophysical Evaluation".into() } else { spec.title.trim().into() },
            well: header.name.clone(),
            field: header.field.clone().map(|f| format!("Field: {f}")).unwrap_or_default(),
            meta,
            author: if spec.author.trim().is_empty() { String::new() } else { format!("Prepared by: {}", spec.author.trim()) },
        },
        Block::PageBreak,
    ];

    // 1 — methodology, from the pane's editable table or the shared default template.
    let method_rows = if spec.methodology.is_empty() {
        crate::report::default_methodology(spec)
    } else {
        spec.methodology.clone()
    };
    let mut m = Sheet::new(
        "Methodology",
        "Methodology",
        vec![
            Column::new("Parameter", 22.0, CellFormat::Text),
            Column::new("Method", 34.0, CellFormat::Text),
            Column::new("Remarks", 34.0, CellFormat::Text),
        ],
    );
    for r in &method_rows {
        m.rows.push(vec![text(&r.parameter), text(&r.method), text(&r.remarks)]);
    }
    blocks.push(table(m));

    // 2 — zone parameters
    let z = report_zone_sheet(&conn, &well_id, &unit)?;
    if !z.rows.is_empty() {
        blocks.push(Block::PageBreak);
        blocks.push(table(z));
    }
    drop(conn);

    // 3 — pay summary, the SAME sheet the workbook exports.
    blocks.push(Block::PageBreak);
    let mut p = pay_sheet(&pay_rows, &unit);
    p.title = format!(
        "Pay Summary  (VSH <= {:.2}, PHIE >= {:.2}, SWE <= {:.2}{})",
        spec.vsh_max,
        spec.phie_min,
        spec.swe_max,
        spec.perm_min.map(|v| format!(", PERM >= {v:.1} mD")).unwrap_or_default()
    );
    if pay_rows.is_empty() {
        // Never a silent gap in a client document: say the section could not be supported.
        blocks.push(Block::Heading { text: p.title.clone() });
        blocks.push(Block::Para {
            text: "No pay summary - this well has no curve data to classify.".into(),
        });
    } else {
        blocks.push(table(p));
    }

    // 4 — why the log plots are not in here.
    blocks.push(Block::Para {
        text: "Composite log plots are issued as the PDF deliverable. They are drawn at a true \
               print scale (1:200 / 1:500 / 1:1000), and a picture pasted into a document stops \
               being at that scale the moment anyone resizes it."
            .into(),
    });

    Ok((blocks, header.name))
}

/// Writes one well's editable Word report.
pub fn export_report_docx(
    db_lock: &Mutex<Connection>,
    spec: &crate::report::ReportSpec,
    dest: &str,
) -> Result<String, String> {
    let (blocks, _name) = build_report_blocks(db_lock, spec)?;
    write_docx(&blocks, dest)?;
    Ok(dest.to_string())
}

/// One `.docx` per well into `dest_dir`, named `<WELL>_report.docx`. Per-well failures are
/// collected rather than aborting the batch — one uninterpreted well must not cost the other
/// 539 their documents.
pub fn export_report_docx_batch(
    db_lock: &Mutex<Connection>,
    spec: &crate::report::ReportSpec,
    well_ids: &[String],
    dest_dir: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut written = Vec::new();
    let mut errors = Vec::new();
    for id in well_ids {
        let mut one = spec.clone();
        one.composite.well_id = id.clone();
        match build_report_blocks(db_lock, &one) {
            Ok((blocks, name)) => {
                let safe: String =
                    name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
                let path = std::path::Path::new(dest_dir).join(format!("{safe}_report.docx"));
                let path_s = path.to_string_lossy().to_string();
                match write_docx(&blocks, &path_s) {
                    Ok(_) => written.push(path_s),
                    Err(e) => errors.push(format!("{name}: {e}")),
                }
            }
            Err(e) => errors.push(format!("{id}: {e}")),
        }
    }
    Ok((written, errors))
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn row(well: &str, zone: &str, flag: &str, net: f32, phie: f32, n: usize) -> PaySummaryRow {
        PaySummaryRow {
            well_id: format!("id-{well}"),
            well_name: well.into(),
            zone: zone.into(),
            flag: flag.into(),
            top: 1000.0,
            bottom: 1100.0,
            gross: 100.0,
            net,
            ntg: net / 100.0,
            avg_vsh: 0.3,
            avg_phie: phie,
            avg_swe: 0.4,
            hpv: net * phie * 0.6,
            n_classified: n,
        }
    }

    #[test]
    fn a_blank_serializes_as_json_null_so_the_runner_leaves_the_cell_empty() {
        let json = serde_json::to_string(&vec![Cell::Num(1.5), Cell::Text("A".into()), Cell::Blank]).unwrap();
        assert_eq!(json, r#"[1.5,"A",null]"#);
    }

    #[test]
    fn a_nan_measurement_is_a_blank_not_a_zero_and_not_minus_999() {
        assert_eq!(num(f32::NAN), Cell::Blank);
        assert_eq!(num(0.0), Cell::Num(0.0));
    }

    #[test]
    fn an_uninterpreted_well_leaves_results_blank_but_still_states_its_geometry() {
        // n_classified == 0: VSH/PHIE/SWE were never computed, so net/ntg/hpv are 0 for want of
        // an answer. Gross is geometry and is known regardless.
        let sheet = pay_sheet(&[row("BLSO-01", "Z1", "PAY", 0.0, 0.0, 0)], "m");
        let r = &sheet.rows[0];
        assert_eq!(r[5], Cell::Num(100.0), "gross is geometry and stays a number");
        for i in [6usize, 7, 8, 9, 10, 11] {
            assert_eq!(r[i], Cell::Blank, "column {i} must be blank, not 0");
        }
        assert_eq!(r[12], Cell::Num(0.0), "the sample count is the evidence for the blanks");
    }

    #[test]
    fn field_averages_are_net_weighted_not_a_mean_of_well_means() {
        // A 90 m sand at PHIE 0.10 and a 10 m sliver at PHIE 0.30. The mean of the two well
        // values is 0.20; the honest field number is 0.12.
        let rows = vec![
            row("A", "Z1", "PAY", 90.0, 0.10, 900),
            row("B", "Z1", "PAY", 10.0, 0.30, 100),
        ];
        let sheet = field_sheet(&rows, "m");
        let r = &sheet.rows[0];
        assert_eq!(r[2], Cell::Num(2.0), "two wells");
        match r[8] {
            // f32 tolerance: the stored curves are f32, so 0.10 is really 0.100000001...
            Cell::Num(v) => assert!((v - 0.12).abs() < 1e-6, "net-weighted PHIE was {v}, expected 0.12"),
            _ => panic!("PHIE must be a number here"),
        }
    }

    #[test]
    fn the_two_ng_columns_answer_different_questions() {
        // 90/100 and 10/100 -> volumetric 100/200 = 0.50, but the mean of the wells is 0.50 too
        // only by coincidence; use an asymmetric pair so they genuinely differ.
        let rows = vec![
            row("A", "Z1", "PAY", 80.0, 0.10, 800), // gross 100, N/G 0.80
            row("B", "Z1", "PAY", 10.0, 0.10, 100), // gross 100, N/G 0.10
        ];
        let sheet = field_sheet(&rows, "m");
        let r = &sheet.rows[0];
        assert_eq!(r[6], Cell::Num(0.45), "field N/G = 90/200");
        match r[7] {
            Cell::Num(v) => assert!((v - 0.45).abs() < 1e-6, "equal gross makes them agree here: {v}"),
            _ => panic!("mean N/G must be a number"),
        }
        // Now make the gross thicknesses differ so the two measures separate.
        let mut thin = row("C", "Z2", "PAY", 9.0, 0.10, 90);
        thin.gross = 10.0;
        thin.ntg = 0.9;
        let mut thick = row("D", "Z2", "PAY", 10.0, 0.10, 100);
        thick.gross = 100.0;
        thick.ntg = 0.1;
        let sheet = field_sheet(&[thin, thick], "m");
        let r = &sheet.rows[0];
        assert_eq!(r[6], Cell::Num(19.0 / 110.0), "field N/G is volumetric");
        match r[7] {
            Cell::Num(v) => assert!((v - 0.5).abs() < 1e-6, "mean of 0.9 and 0.1 is 0.5, not {v}"),
            _ => panic!("mean N/G must be a number"),
        }
    }

    #[test]
    fn an_uninterpreted_well_is_counted_separately_and_drags_no_average_down() {
        let rows = vec![
            row("A", "Z1", "PAY", 90.0, 0.20, 900),
            row("B", "Z1", "PAY", 0.0, 0.0, 0), // never interpreted
        ];
        let sheet = field_sheet(&rows, "m");
        let r = &sheet.rows[0];
        assert_eq!(r[2], Cell::Num(1.0), "one well contributed");
        assert_eq!(r[3], Cell::Num(1.0), "one well is reported as not interpreted");
        assert_eq!(r[5], Cell::Num(90.0), "the blind well adds no net");
        match r[8] {
            Cell::Num(v) => assert!((v - 0.20).abs() < 1e-6, "PHIE unmoved by the blind well: {v}"),
            _ => panic!("PHIE must be a number"),
        }
    }

    #[test]
    fn zones_are_ordered_shallow_to_deep_not_alphabetically() {
        let mut deep = row("A", "ALPHA", "PAY", 10.0, 0.1, 10);
        deep.top = 2000.0;
        let mut shallow = row("A", "ZULU", "PAY", 10.0, 0.1, 10);
        shallow.top = 1000.0;
        let sheet = field_sheet(&[deep, shallow], "m");
        assert_eq!(sheet.rows[0][0], Cell::Text("ZULU".into()), "shallowest zone first");
        assert_eq!(sheet.rows[1][0], Cell::Text("ALPHA".into()));
    }

    /// The real round-trip through the real runner. `#[ignore]`d because it needs a Python with
    /// xlsxwriter, which the green gate must NOT require — rule 7 says a missing package fails
    /// its own button, so it must not be able to fail the build. Run with
    /// `cargo test -- --ignored writes_a_real_workbook`.
    #[test]
    #[ignore]
    fn writes_a_real_workbook_that_opens_as_a_zip() {
        let support = office_support();
        assert!(support.xlsxwriter, "this test needs xlsxwriter: {support:?}");
        let dest = std::env::temp_dir().join("sandibumi_office_roundtrip.xlsx");
        let dest_s = dest.to_string_lossy().to_string();
        let rows = vec![row("A", "Z1", "PAY", 90.0, 0.2, 900), row("B", "Z1", "PAY", 0.0, 0.0, 0)];
        let sheets = vec![pay_sheet(&rows, "m"), field_sheet(&rows, "m")];
        let n = write_workbook(&sheets, &dest_s).expect("workbook written");
        assert_eq!(n, 2);
        let bytes = std::fs::read(&dest).expect("file exists");
        assert!(bytes.len() > 2000, "a two-sheet workbook is not 2 kB: {}", bytes.len());
        assert_eq!(&bytes[..2], b"PK", "xlsx is a zip container");
        let _ = std::fs::remove_file(&dest);
    }

    // --- The Word twin -----------------------------------------------------

    fn zoned_db() -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let id = uuid::Uuid::new_v4();
        db::insert_well(&conn, id, "BLSO-01", Some("Balam South"), Some(1800.0), Some(12.0)).unwrap();
        let well = id.to_string();
        db::upsert_zone(&conn, &well, "MENGGALA", 1500.0, 1560.0).unwrap();
        db::upsert_zone(&conn, &well, "BEKASAP", 1600.0, 1680.0).unwrap();
        db::set_zone_param(&conn, &well, "MENGGALA", "RW", Some(0.35), None).unwrap();
        db::set_zone_param(&conn, &well, "MENGGALA", "M", Some(1.8), None).unwrap();
        (conn, well)
    }

    #[test]
    fn a_zone_with_no_parameters_is_still_listed_in_the_report() {
        // Dropping it would tell the client the zone was not evaluated, when in truth it simply
        // took the defaults.
        let (conn, well) = zoned_db();
        let sheet = report_zone_sheet(&conn, &well, "m").unwrap();
        let zones: Vec<&Cell> = sheet.rows.iter().map(|r| &r[0]).collect();
        assert!(zones.contains(&&Cell::Text("BEKASAP".into())), "the parameterless zone is present");
    }

    #[test]
    fn the_zone_column_repeats_only_on_a_zones_first_parameter() {
        // The PDF's convention — this document is its twin, so it must read the same way.
        let (conn, well) = zoned_db();
        let sheet = report_zone_sheet(&conn, &well, "m").unwrap();
        let menggala: Vec<&Vec<Cell>> =
            sheet.rows.iter().filter(|r| r[3] == Cell::Text("M".into()) || r[3] == Cell::Text("RW".into())).collect();
        assert_eq!(menggala.len(), 2, "two parameters on MENGGALA");
        assert_eq!(menggala[0][0], Cell::Text("MENGGALA".into()), "named on the first row");
        assert_eq!(menggala[0][1], Cell::Num(1500.0), "with its depths");
        assert_eq!(menggala[1][0], Cell::Blank, "and not repeated on the second");
    }

    #[test]
    fn a_document_prints_a_dash_where_the_workbook_leaves_the_cell_empty() {
        // The SAME Sheet, rendered two ways on purpose: Excel's arithmetic skips an empty cell,
        // so a blank is the honest value there; a document has no arithmetic, so a reader's eye
        // needs the mark the PDF already prints.
        let sheet = pay_sheet(&[row("A", "Z1", "PAY", 0.0, 0.0, 0)], "m");
        assert_eq!(sheet.rows[0][6], Cell::Blank, "the sheet itself still carries a blank");
        match table(sheet) {
            Block::Table { blank_text, .. } => assert_eq!(blank_text, "-"),
            _ => panic!("table() must build a Table block"),
        }
    }

    #[test]
    fn blocks_serialize_with_the_tag_the_runner_dispatches_on() {
        let json = serde_json::to_string(&Block::PageBreak).unwrap();
        assert_eq!(json, r#"{"kind":"page_break"}"#);
        let json = serde_json::to_string(&Block::Heading { text: "Methodology".into() }).unwrap();
        assert_eq!(json, r#"{"kind":"heading","text":"Methodology"}"#);
    }

    /// Real python-docx round-trip, INCLUDING non-ASCII text.
    ///
    /// `#[ignore]`d for the same reason as the workbook's: rule 7 says a missing package fails
    /// its own button, so it must not be able to fail the build.
    ///
    /// The non-ASCII half is the regression guard. serde_json sends raw UTF-8; a piped child's
    /// TEXT stdin decodes with the Windows ANSI codepage, so `sys.stdin.read()` turned an en
    /// dash into two characters of noise inside a client document. The runners read
    /// `sys.stdin.buffer` instead — this proves the bytes arrive intact all the way into the
    /// saved file.
    #[test]
    #[ignore]
    fn a_word_document_keeps_non_ascii_text_intact() {
        let support = office_support();
        assert!(support.docx, "this test needs python-docx: {support:?}");
        let dest = std::env::temp_dir().join("sandibumi_office_report.docx");
        let dest_s = dest.to_string_lossy().to_string();
        // An en dash in the well name, a middot separator, and rho-ma spelled with the Greek
        // letter: all three are things this app really writes.
        let needle = "Bekasap\u{2013}1  \u{00b7}  \u{03c1}ma 2.65";
        let rows = vec![row("A", "Z1", "PAY", 90.0, 0.2, 900), row("B", "Z1", "PAY", 0.0, 0.0, 0)];
        let blocks = vec![
            Block::Cover {
                title: "Petrophysical Evaluation".into(),
                well: needle.into(),
                field: "Field: Balam South".into(),
                meta: "TD: 1800.0 m".into(),
                author: "Prepared by: Jauhar".into(),
            },
            Block::PageBreak,
            table(pay_sheet(&rows, "m")),
        ];
        let tables = write_docx(&blocks, &dest_s).expect("document written");
        assert_eq!(tables, 1);
        let bytes = std::fs::read(&dest).expect("file exists");
        assert!(bytes.len() > 5000, "a report docx is not 5 kB: {}", bytes.len());
        assert_eq!(&bytes[..2], b"PK", "docx is a zip container");

        // Read it back with python-docx and demand the exact string. `ensure_ascii` on the way
        // back means the comparison itself cannot be fooled by another encoding hop.
        let python = find_python().expect("python");
        const READBACK: &str = r#"
import json, sys
from docx import Document
doc = Document(sys.argv[1])
sys.stdout.buffer.write(json.dumps([p.text for p in doc.paragraphs if p.text], ensure_ascii=True).encode("ascii"))
"#;
        let mut cmd = Command::new(&python);
        cmd.args(["-c", READBACK, &dest_s]).stdout(Stdio::piped()).stderr(Stdio::piped());
        hide_console(&mut cmd);
        let out = cmd.output().expect("readback ran");
        let paras: Vec<String> = serde_json::from_slice(&out.stdout).expect("readback json");
        assert!(
            paras.iter().any(|p| p == needle),
            "the en dash / middot / rho did not survive the pipe: {paras:?}"
        );
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn the_pay_sheet_shades_the_pay_rows_and_carries_every_flag() {
        let rows = vec![
            row("A", "Z1", "SAND", 90.0, 0.2, 900),
            row("A", "Z1", "RESERVOIR", 60.0, 0.2, 900),
            row("A", "Z1", "PAY", 40.0, 0.2, 900),
        ];
        let sheet = pay_sheet(&rows, "m");
        assert_eq!(sheet.rows.len(), 3, "all three cutoff levels are exported, not just PAY");
        let shade = sheet.shade.as_ref().expect("PAY rows are highlighted");
        assert_eq!(shade.col, 2, "the Flag column drives the shading");
        assert_eq!(shade.equals, "PAY");
    }
}
