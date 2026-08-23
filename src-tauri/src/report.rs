//! Report generator (Phase 8b) — assembles the client-style petrophysics report PDF
//! following Jauhar's real report structure (his standard clastic/carbonate template): cover page →
//! methodology (parameter–method–remarks table) → per-zone parameter table → pay
//! summary (cutoffs) → composite log pages. Every page is a `Vec<DrawOp>` in mm
//! space, serialized through the same SVG/PDF machinery as the composite plot
//! (`composite::svg_page` / `pdf_content` / `assemble_pdf`), so one command yields
//! one client-ready multi-page PDF (or per-page SVGs for the in-dialog preview).

use crate::composite::{
    self, Anchor, CompositePage, CompositeResult, CompositeSpec, DrawOp,
};
use crate::db;
use crate::paysummary::{run_pay_summary, PaySummaryRequest};
use duckdb::Connection;
use serde::Deserialize;
use std::sync::Mutex;

const MARGIN: f64 = 16.0;
const LINE_H_FACTOR: f64 = 1.45; // line height as a multiple of font size (mm)
/// Average Helvetica glyph advance as a fraction of the font size — the same
/// approximation the composite PDF writer uses for centred text.
const CHAR_W: f64 = 0.52;

#[derive(Debug, Clone, Deserialize)]
pub struct MethodRow {
    pub parameter: String,
    pub method: String,
    pub remarks: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportSpec {
    /// Composite settings (well, layout, scale, page size, depth window). The report's
    /// page size and target well come from here.
    pub composite: CompositeSpec,
    /// Study title on the cover, e.g. "Petrophysical Evaluation — Sandi Field".
    pub title: String,
    #[serde(default)]
    pub author: String,
    /// Methodology table rows; empty = built-in default template.
    #[serde(default)]
    pub methodology: Vec<MethodRow>,
    /// Pay-summary cutoffs (pay-summary convention).
    /// SB-CUT-016. `None` = UNFILTERED on this property, and the deliverable says so
    /// rather than printing a number that was never applied. No default: four shipped
    /// vendor sets disagree, two of them from one vendor.
    pub vsh_max: Option<crate::paysummary::CutoffSpec>,
    pub phie_min: Option<crate::paysummary::CutoffSpec>,
    pub swe_max: Option<crate::paysummary::CutoffSpec>,
    #[serde(default)]
    pub perm_min: Option<crate::paysummary::CutoffSpec>,
    /// Report the interpretation stored in THIS log set rather than whatever the current curve
    /// values happen to be. A deliverable that cannot name the version it quotes is a deliverable
    /// nobody can reproduce (Jauhar, 2026-08-05); an empty name keeps the previous behaviour.
    #[serde(default)]
    pub input_set: Option<String>,
    /// Explicit operator and source/reference for the pay-summary FLAG curves that
    /// the PDF report has historically persisted. Editable Office exports are
    /// read-only and do not consume this field.
    #[serde(default)]
    pub custody: Option<crate::ancestry::RunCustody>,
    /// Skip the composite pages (tables-only report).
    #[serde(default)]
    pub tables_only: bool,
}

/// `pub(crate)` because the Word twin (`office.rs`) must quote the SAME methodology table the
/// PDF does — a client comparing the two documents is comparing one study, not two.
pub(crate) fn default_methodology(spec: &ReportSpec) -> Vec<MethodRow> {
    let row = |p: &str, m: &str, r: &str| MethodRow {
        parameter: p.into(),
        method: m.into(),
        remarks: r.into(),
    };
    vec![
        row("Data conditioning", "Mnemonic standardization, splice, depth shift, bad-hole flag", "Environmental corrections where required"),
        row("GR normalization", "Two-point percentile (P3/P97) to regional reference", "QC by multi-well GRN histogram overlay"),
        row("Shale volume", "GR linear (SSC/density-neutron where run)", "Zone parameters from crossplots"),
        row("Porosity", "Density / density-neutron (SSC or SSPW where run)", "PHIE = PHIT − clay-bound water"),
        row("Water saturation", "Indonesia / Simandoux / Archie (RtC or IMTS in LRLC zones)", "Rw from Pickett and/or water sample"),
        row("Permeability", "Timur / Coates / por-perm transform", "Calibrated to core where available"),
        row(
            "Cutoffs",
            "VSH / PHIE / SWE flags → SAND, RESERVOIR, PAY",
            &format!(
                "VSH ≤ {}, PHIE ≥ {}, SWE ≤ {}{}",
                crate::paysummary::cutoff_label(spec.vsh_max.as_ref(), 2),
                crate::paysummary::cutoff_label(spec.phie_min.as_ref(), 2),
                crate::paysummary::cutoff_label(spec.swe_max.as_ref(), 2),
                match crate::paysummary::cutoff_phrase(
                    spec.perm_min.as_ref(),
                    crate::paysummary::CutoffSense::Minimum,
                    1,
                )
                .as_str()
                {
                    "" => String::new(),
                    phrase => format!(", PERM {phrase}"),
                }
            ),
        ),
    ]
}

// ---------------------------------------------------------------------------
// Small drawing helpers
// ---------------------------------------------------------------------------

/// Greedy word-wrap into lines of at most `max_chars` characters.
fn wrap(s: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(4);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= max_chars {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
        // A single word longer than the cell is hard-split.
        while cur.chars().count() > max_chars {
            let head: String = cur.chars().take(max_chars).collect();
            lines.push(head);
            cur = cur.chars().skip(max_chars).collect();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn text(ops: &mut Vec<DrawOp>, x: f64, y: f64, size: f64, anchor: Anchor, bold: bool, color: &str, s: impl Into<String>) {
    ops.push(DrawOp::Text { x, y, size, anchor, color: color.into(), bold, s: s.into() });
}

fn hline(ops: &mut Vec<DrawOp>, x1: f64, x2: f64, y: f64, color: &str, sw: f64) {
    ops.push(DrawOp::Line { x1, y1: y, x2, y2: y, stroke: color.into(), sw });
}

/// Running page header for table pages (title left, well right, rule underneath).
fn page_header(ops: &mut Vec<DrawOp>, pw: f64, section: &str, well: &str) -> f64 {
    text(ops, MARGIN, MARGIN + 3.0, 4.2, Anchor::Start, true, "#111111", section);
    text(ops, pw - MARGIN, MARGIN + 3.0, 2.8, Anchor::End, false, "#555555", well);
    hline(ops, MARGIN, pw - MARGIN, MARGIN + 5.5, "#333333", 0.35);
    MARGIN + 10.0
}

/// The "Made in SandiBumi" mark, in the bottom margin.
///
/// Every other surface in the deliverable set already carried it — the report cover, every
/// composite page, the Word document and the PowerPoint deck — and the PDF's TABLE pages were the
/// one exception, so a reader who extracted or photocopied the pay summary got an unattributed
/// page (`docs/review_triage.md` finding 15). Applied per page rather than per table, because a
/// long pay summary paginates and the mark has to survive being read one page at a time.
///
/// Deliberately smaller and paler than the cover's: a footer that competes with the table is
/// worse than no footer.
fn page_footer(ops: &mut Vec<DrawOp>, pw: f64, ph: f64) {
    text(ops, pw / 2.0, ph - MARGIN + 5.0, 2.4, Anchor::Middle, false, "#999999", "Made in SandiBumi");
}

/// A single page carrying just a section header and a wrapped note line — used when a section has
/// no table to show (a compute/storage error, or a legitimately empty result) so a deliverable
/// never silently drops the section: the header still appears, with an explicit reason underneath.
fn note_page(section: &str, well: &str, note: &str, pw: f64, ph: f64) -> Vec<DrawOp> {
    let mut ops: Vec<DrawOp> = Vec::new();
    let y = page_header(&mut ops, pw, section, well);
    let size = 3.0;
    let max_chars = ((pw - 2.0 * MARGIN) / (CHAR_W * size)) as usize;
    for (i, line) in wrap(note, max_chars).into_iter().enumerate() {
        text(&mut ops, MARGIN, y + size + i as f64 * size * LINE_H_FACTOR, size, Anchor::Start, false, "#222222", line);
    }
    page_footer(&mut ops, pw, ph);
    ops
}

/// Paginated table: fixed column widths (mm), wrapped cells, header row repeated on
/// every page. Returns one `Vec<DrawOp>` per page.
///
/// `caveat` is a line about the table's own trustworthiness — today, a permeability cutoff that
/// could not be applied. It is drawn under the header on EVERY page of the table, not only the
/// first: a pay summary paginates, and a page read on its own must carry the reason its numbers
/// are not comparable with the next well's.
#[allow(clippy::too_many_arguments)]
fn table_pages(
    section: &str,
    well: &str,
    caveat: Option<&str>,
    cols: &[(&str, f64, Anchor)],
    rows: &[Vec<String>],
    pw: f64,
    ph: f64,
    size: f64,
) -> Vec<Vec<DrawOp>> {
    let line_h = size * LINE_H_FACTOR;
    let pad = 1.4;
    let y_max = ph - MARGIN;
    let x0 = MARGIN;

    let col_x: Vec<f64> = cols
        .iter()
        .scan(x0, |acc, (_, w, _)| {
            let x = *acc;
            *acc += w;
            Some(x)
        })
        .collect();
    let table_w: f64 = cols.iter().map(|(_, w, _)| w).sum();

    let header_row = |ops: &mut Vec<DrawOp>, y: f64| -> f64 {
        let h = line_h + 2.0 * pad;
        ops.push(DrawOp::Rect {
            x: x0,
            y,
            w: table_w,
            h,
            fill: Some("#e8e4da".into()),
            stroke: Some("#333333".into()),
            sw: 0.3,
        });
        for (ci, (title, w, _)) in cols.iter().enumerate() {
            text(ops, col_x[ci] + w / 2.0, y + pad + size, size, Anchor::Middle, true, "#111111", *title);
        }
        y + h
    };

    const CAVEAT_SIZE: f64 = 2.6;
    let start_page = |ops: &mut Vec<DrawOp>| -> f64 {
        let mut y = page_header(ops, pw, section, well);
        if let Some(c) = caveat {
            let max_chars = ((pw - 2.0 * MARGIN) / (CHAR_W * CAVEAT_SIZE)) as usize;
            let lines = wrap(c, max_chars);
            for (i, line) in lines.iter().enumerate() {
                let ly = y + CAVEAT_SIZE + i as f64 * CAVEAT_SIZE * LINE_H_FACTOR;
                text(ops, MARGIN, ly, CAVEAT_SIZE, Anchor::Start, false, "#8a3b2a", line.clone());
            }
            y += lines.len() as f64 * CAVEAT_SIZE * LINE_H_FACTOR + 2.0;
        }
        y
    };

    let mut pages: Vec<Vec<DrawOp>> = Vec::new();
    let mut ops: Vec<DrawOp> = Vec::new();
    let mut y = start_page(&mut ops);
    y = header_row(&mut ops, y);

    for row in rows {
        // Wrap every cell, row height = tallest cell.
        let wrapped: Vec<Vec<String>> = row
            .iter()
            .enumerate()
            .map(|(ci, cell)| {
                let w = cols.get(ci).map(|c| c.1).unwrap_or(30.0);
                wrap(cell, ((w - 2.0 * pad) / (CHAR_W * size)) as usize)
            })
            .collect();
        let n_lines = wrapped.iter().map(|c| c.len()).max().unwrap_or(1);
        let row_h = n_lines as f64 * line_h + 2.0 * pad;

        if y + row_h > y_max {
            pages.push(std::mem::take(&mut ops));
            y = start_page(&mut ops);
            y = header_row(&mut ops, y);
        }

        ops.push(DrawOp::Rect {
            x: x0,
            y,
            w: table_w,
            h: row_h,
            fill: None,
            stroke: Some("#999999".into()),
            sw: 0.2,
        });
        for (ci, cell_lines) in wrapped.iter().enumerate() {
            let (_, w, anchor) = cols[ci];
            let tx = match anchor {
                Anchor::Start => col_x[ci] + pad,
                Anchor::Middle => col_x[ci] + w / 2.0,
                Anchor::End => col_x[ci] + w - pad,
            };
            for (li, line) in cell_lines.iter().enumerate() {
                text(&mut ops, tx, y + pad + size + li as f64 * line_h, size, anchor, false, "#222222", line.clone());
            }
        }
        y += row_h;
    }
    // Column separators are drawn per page via the outer rects only — vertical rules:
    // (drawn last on the final page of each table page above; simple approach: skip.)
    pages.push(ops);
    // Marked here, after pagination, rather than at each `pages.push` — there are two of those
    // and a mark added at one of them would silently miss every continuation page of a long pay
    // summary, which is exactly the page most likely to be extracted on its own.
    for p in &mut pages {
        page_footer(p, pw, ph);
    }
    pages
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

/// `interval` is the LOGGED interval — the rock this study covers. `window` is the composite's
/// print window when one was set and narrows it, stated separately rather than replacing the
/// interval: the tables ignore the window entirely, so quoting it as the interval would date the
/// whole report to a display setting (`docs/review_triage.md` finding 18).
fn cover_page(
    spec: &ReportSpec,
    header: &composite::WellHeader,
    interval: (f32, f32),
    window: Option<(f32, f32)>,
    // The project's stored depth unit. Every depth on this cover is a stored depth printed
    // verbatim — the logged interval, the print window, TD and KB — so the unit is named, never
    // assumed. TD and KB ride along because they are depths on the same grid.
    depth_unit: crate::units::DepthUnit,
    pw: f64,
    ph: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let cx = pw / 2.0;

    // Earth-tone top band + rules, deliberately plain (client template friendly).
    ops.push(DrawOp::Rect { x: 0.0, y: 0.0, w: pw, h: 14.0, fill: Some("#5b4a36".into()), stroke: None, sw: 0.0 });
    text(&mut ops, cx, 9.5, 4.0, Anchor::Middle, true, "#f5f0e6", "PETROPHYSICAL EVALUATION REPORT");

    let mut y = ph * 0.28;
    for line in wrap(&spec.title, 34) {
        text(&mut ops, cx, y, 7.0, Anchor::Middle, true, "#111111", line);
        y += 10.0;
    }
    hline(&mut ops, pw * 0.2, pw * 0.8, y + 2.0, "#5b4a36", 0.8);
    y += 14.0;

    text(&mut ops, cx, y, 5.2, Anchor::Middle, true, "#333333", format!("Well: {}", header.name));
    y += 8.0;
    if let Some(f) = &header.field {
        text(&mut ops, cx, y, 3.6, Anchor::Middle, false, "#333333", format!("Field: {f}"));
        y += 6.0;
    }
    let du = depth_unit.label();
    let mut meta = format!("Interval: {:.1} – {:.1} {du}", interval.0, interval.1);
    if let Some((wt, wb)) = window {
        meta.push_str(&format!("   ·   Log pages printed over {wt:.1} – {wb:.1} {du}"));
    }
    if let Some(td) = header.td {
        meta.push_str(&format!("   ·   TD: {td:.1} {du}"));
    }
    if let Some(kb) = header.kb {
        meta.push_str(&format!("   ·   KB: {kb:.1} {du}"));
    }
    text(&mut ops, cx, y, 3.2, Anchor::Middle, false, "#555555", meta);

    let mut yb = ph - 34.0;
    if !spec.author.is_empty() {
        text(&mut ops, cx, yb, 3.4, Anchor::Middle, false, "#333333", format!("Prepared by: {}", spec.author));
        yb += 6.0;
    }
    text(&mut ops, cx, yb, 2.8, Anchor::Middle, false, "#777777", "Made in SandiBumi");
    ops.push(DrawOp::Rect { x: 0.0, y: ph - 6.0, w: pw, h: 6.0, fill: Some("#5b4a36".into()), stroke: None, sw: 0.0 });
    ops
}

fn fmt_num(v: f32, dec: usize) -> String {
    if v.is_nan() {
        "-".into()
    } else {
        format!("{v:.dec$}")
    }
}

/// Builds every report page as DrawOps. Locks the connection only while reading.
fn report_pages_with_degradations(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
) -> Result<(Vec<Vec<DrawOp>>, f64, f64, String, Vec<String>), String> {
    let (composite_pages, pw, ph, well_name, header, zones, zparams, logged, ml_prov, depth_unit) = {
        let conn = db.lock().unwrap();
        // Every depth and thickness printed below comes from the project's own depth column and
        // is NOT converted — `run_pay_summary` accumulates raw sample thickness, and the zone
        // tops are stored depths. So the headings name the project's unit. They used to say "(m)"
        // unconditionally: on a foot project this PDF stated a client's net pay and hydrocarbon
        // pore thickness in metres over numbers that were feet, overstating both by 3.28084x
        // while every figure stayed plausible. The workbook path (`office.rs`) already did this
        // correctly, which is what proves the raw values are deliberately project-native.
        let depth_unit = crate::units::project_depth_unit(&conn)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let header = composite::fetch_header(&conn, &spec.composite.well_id)?;
        let zones = db::list_zones(&conn, &spec.composite.well_id).map_err(|e| e.to_string())?;
        let zparams = db::list_zone_params(&conn, &spec.composite.well_id).map_err(|e| e.to_string())?;
        // SB-MLA-010. Read from the CURRENT store, so it describes the curves this report will
        // actually print rather than every ML run the well has ever seen.
        let ml_prov = crate::ml::ml_provenance(&conn, &spec.composite.well_id);
        // The cover's interval, taken from the LOG rather than from the composite pagination —
        // see `db::logged_interval`. The lock is released at the end of this block —
        // run_pay_summary takes it itself.
        let logged = db::logged_interval(&conn, &spec.composite.well_id);
        let (cpages, pw, ph, name) = composite::render_pages(&conn, &spec.composite)?;
        (cpages, pw, ph, name, header, zones, zparams, logged, ml_prov, depth_unit)
    };
    let du = depth_unit.label();

    // The PDF's pay-summary pass is a real persisted computation. Run it before
    // collecting disclosures so the report carries the ancestry of the exact FLAG
    // curves it just produced (or reused), never the snapshot from one moment earlier.
    let pay_result = run_pay_summary(
        db,
        pool,
        &PaySummaryRequest {
            // SB-CUT-001 (DEC-071): exports run the product-default model (CENTRED).
            discretisation: Default::default(),
            well_ids: vec![spec.composite.well_id.clone()],
            vsh_max: spec.vsh_max.clone(),
            phie_min: spec.phie_min.clone(),
            swe_max: spec.swe_max.clone(),
            enabled_unset: Vec::new(),
            cutoff_use: Default::default(),
            perm_min: spec.perm_min.clone(),
            input_set: spec.input_set.clone(),
            skip_version: false,
            stats_only: false,
            custody: spec.custody.clone(),
            frame: Default::default(),
            weighting: Default::default(),
        },
    );
    let ancestry = {
        let conn = db.lock().map_err(|error| error.to_string())?;
        crate::ancestry::curve_ancestry_disclosures(
            &conn,
            std::slice::from_ref(&spec.composite.well_id),
            spec.input_set.as_deref(),
        )?
    };

    {
        // What the composite actually paginated. Equal to the logged interval unless a depth
        // window was set on the render.
        let printed = (
            composite_pages.first().map(|p| p.top).unwrap_or(0.0),
            composite_pages.last().map(|p| p.bot).unwrap_or(0.0),
        );
        // The study's rock. Falls back to the pagination only for a well with no curve rows at
        // all, which is the same 0.0 – 0.0 this always printed rather than a new failure mode.
        let interval = logged.unwrap_or(printed);
        // A print window is a display setting, and the tables ignore it: `run_pay_summary` works
        // per zone and knows nothing about it. So it is stated BESIDE the interval, never instead
        // of it — a cover that quotes the window as the interval describes a study nobody did.
        // Only shown when it genuinely narrows, and never on a tables-only render, where there
        // are no log pages for it to describe.
        let window = (!spec.tables_only
            && logged.is_some_and(|(lo, hi)| printed.0 > lo + 0.05 || printed.1 < hi - 0.05))
            .then_some(printed);

        let mut pages: Vec<Vec<DrawOp>> = Vec::new();
        let mut degradations: Vec<String> = Vec::new();

        // 1 — cover
        pages.push(cover_page(spec, &header, interval, window, depth_unit, pw, ph));

        // 2 — methodology table
        let rows_src = if spec.methodology.is_empty() { default_methodology(spec) } else { spec.methodology.clone() };
        let m_rows: Vec<Vec<String>> = rows_src
            .iter()
            .map(|r| vec![r.parameter.clone(), r.method.clone(), r.remarks.clone()])
            .collect();
        let usable = pw - 2.0 * MARGIN;
        let m_cols: [(&str, f64, Anchor); 3] = [
            ("Parameter", usable * 0.24, Anchor::Start),
            ("Method", usable * 0.38, Anchor::Start),
            ("Remarks", usable * 0.38, Anchor::Start),
        ];
        pages.extend(table_pages("Methodology", &well_name, None, &m_cols, &m_rows, pw, ph, 2.7));

        // SB-CORE-010. A report that carries a computed value also carries the full
        // record needed to identify who ran it, from which curves and zones, with
        // which sourced parameters, and by which derivation.
        if !ancestry.is_empty() {
            let labels = [
                "Curve / set",
                "Module",
                "Inputs",
                "Parameters / source",
                "Zones / source",
                "Operator / time",
                "Derivation",
            ];
            let mut a_rows = Vec::with_capacity(ancestry.len() * labels.len());
            for disclosure in &ancestry {
                for (label, value) in labels.iter().zip(disclosure.cells()) {
                    a_rows.push(vec![(*label).to_string(), value]);
                }
            }
            let a_cols: [(&str, f64, Anchor); 2] = [
                ("Ancestry field", usable * 0.23, Anchor::Start),
                ("Recorded value", usable * 0.77, Anchor::Start),
            ];
            pages.extend(table_pages(
                "Computed curve ancestry",
                &well_name,
                None,
                &a_cols,
                &a_rows,
                pw,
                ph,
                2.5,
            ));
        }

        // 2b — ML provenance (SB-MLA-010), only where a model-derived curve is actually live on
        // this well. This is the point of the whole provenance group: a parameter that carries the
        // paper it came from, through the computation, into the deliverable — until now the
        // lineage stopped at the database boundary.
        //
        // Its own section rather than rows in the methodology table, because the methodology table
        // describes the METHOD and this describes a specific fitted artifact: same algorithm, two
        // different models, two different sets of rock. It sits immediately after, so a reader who
        // has just read "Permeability — por-perm transform" meets "and this well's PERM was
        // predicted by a model, here is how well it travels" before any number built on it.
        if !ml_prov.is_empty() {
            let p_rows: Vec<Vec<String>> = ml_prov.iter().map(|r| r.cells().to_vec()).collect();
            let h = crate::ml::ML_PROV_HEADERS;
            let p_cols: [(&str, f64, Anchor); 6] = [
                (h[0], usable * 0.13, Anchor::Start),
                (h[1], usable * 0.17, Anchor::Start),
                (h[2], usable * 0.18, Anchor::Start),
                (h[3], usable * 0.13, Anchor::Start),
                (h[4], usable * 0.25, Anchor::Start),
                (h[5], usable * 0.14, Anchor::Start),
            ];
            pages.extend(table_pages(
                "Machine-learning provenance",
                &well_name,
                Some(crate::ml::ML_PROV_CAVEAT),
                &p_cols,
                &p_rows,
                pw,
                ph,
                2.5,
            ));
        }

        // 3 — per-zone parameter table (zones without params still listed)
        let mut z_rows: Vec<Vec<String>> = Vec::new();
        for z in &zones {
            let params: Vec<&db::ZoneParamEntry> =
                zparams.iter().filter(|p| p.zone_name == z.zone_name).collect();
            if params.is_empty() {
                z_rows.push(vec![
                    z.zone_name.clone(),
                    format!("{:.1}", z.top_depth),
                    format!("{:.1}", z.bottom_depth),
                    "-".into(),
                    "-".into(),
                ]);
            }
            for (i, p) in params.iter().enumerate() {
                let val = p
                    .value_num
                    .map(|v| format!("{v}"))
                    .or_else(|| p.value_text.clone())
                    .unwrap_or_else(|| "-".into());
                z_rows.push(vec![
                    if i == 0 { z.zone_name.clone() } else { String::new() },
                    if i == 0 { format!("{:.1}", z.top_depth) } else { String::new() },
                    if i == 0 { format!("{:.1}", z.bottom_depth) } else { String::new() },
                    p.param_name.clone(),
                    val,
                ]);
            }
        }
        if !z_rows.is_empty() {
            let (z_top, z_bottom) = (format!("Top ({du})"), format!("Bottom ({du})"));
            let z_cols: [(&str, f64, Anchor); 5] = [
                ("Zone", usable * 0.26, Anchor::Start),
                (z_top.as_str(), usable * 0.13, Anchor::End),
                (z_bottom.as_str(), usable * 0.13, Anchor::End),
                ("Parameter", usable * 0.28, Anchor::Start),
                ("Value", usable * 0.20, Anchor::End),
            ];
            pages.extend(table_pages("Zone Parameters", &well_name, None, &z_cols, &z_rows, pw, ph, 2.7));
        }

        // 4 — pay summary (locks internally). Emit the section header unconditionally: a storage
        // error or an uninterpreted well must leave a visible note in the deliverable rather than a
        // silently missing section — this is a client PDF, and dropping a table it cannot support
        // (unwrap_or_default previously collapsed both Err and empty into "no section at all") is
        // exactly the cardinal-rule failure the report path must not allow.
        // SB-CUT-002: the printed table names its discretisation identity in the heading.
        // The step is stated only when every row shares one; wells on different frames say so
        // rather than quoting one well's step as everyone's.
        let pay_section = format!(
            "Pay Summary  (VSH ≤ {}, PHIE ≥ {}, SWE ≤ {}, {} model{})",
            crate::paysummary::cutoff_label(spec.vsh_max.as_ref(), 2),
            crate::paysummary::cutoff_label(spec.phie_min.as_ref(), 2),
            crate::paysummary::cutoff_label(spec.swe_max.as_ref(), 2),
            pay_result
                .as_ref()
                .ok()
                .and_then(|rows| rows.first())
                .map(|row| row.discretisation_model.clone())
                .unwrap_or_else(|| crate::paysummary::DiscretisationModel::default().token().to_string()),
            match pay_result.as_ref().ok().and_then(|rows| {
                let mut steps: Vec<f32> =
                    rows.iter().map(|r| r.sample_interval).filter(|s| s.is_finite()).collect();
                steps.sort_by(|a, b| a.partial_cmp(b).unwrap());
                steps.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
                match steps.as_slice() {
                    [only] => Some(format!(", step {only:.4}")),
                    [] => None,
                    _ => Some(", mixed steps — see workbook".to_string()),
                }
            }) {
                Some(s) => s,
                None => String::new(),
            }
        );
        match pay_result {
            Ok(pay_rows) if !pay_rows.is_empty() => {
                // A permeability cutoff this well has nothing to answer with. Stated on the page
                // rather than only in the row struct, because the deliverable is where a reader
                // actually meets the number: every zone reports zero net pay, which on the page is
                // indistinguishable from a wet well (`docs/review_triage.md` finding 7).
                let pay_caveat = pay_rows
                    .iter()
                    .any(|r| r.perm_cutoff_no_data)
                    .then(|| {
                        format!(
                            "Note: this well carries no permeability curve, so every sample fails the \
                             PERM {} cutoff for want of data. The zero net pay below records an \
                             absence of evidence, not a dry reservoir — compute or import a permeability \
                             curve, or lift the cutoff, before reading these rows.",
                            // SB-CUT-020: the comparison comes from the cut-off itself. The note
                            // used to hard-code `≥`, which an exclusive bound or a two-sided
                            // window makes untrue — and this note exists to be read literally.
                            crate::paysummary::cutoff_phrase(
                                spec.perm_min.as_ref(),
                                crate::paysummary::CutoffSense::Minimum,
                                1,
                            )
                        )
                    });
                let p_rows: Vec<Vec<String>> = pay_rows
                    .iter()
                    .map(|r| {
                        vec![
                            r.zone.clone(),
                            r.flag.clone(),
                            fmt_num(r.top, 1),
                            fmt_num(r.bottom, 1),
                            fmt_num(r.gross, 1),
                            // A well whose VSH/PHIE/SWE were never computed classifies to NaN
                            // everywhere, leaving net/ntg/hpv at exactly 0 — indistinguishable in
                            // print from a genuine wet zone. This is a client deliverable, so it
                            // must not assert a zero it cannot support: emit the same "-" the NaN
                            // averages already use, and let the reader see the row was not interpreted.
                            if r.n_classified == 0 { "-".to_string() } else { fmt_num(r.net, 1) },
                            if r.n_classified == 0 { "-".to_string() } else { fmt_num(r.ntg, 2) },
                            fmt_num(r.avg_vsh, 2),
                            fmt_num(r.avg_phie, 3),
                            fmt_num(r.avg_swe, 2),
                            if r.n_classified == 0 { "-".to_string() } else { fmt_num(r.hpv, 2) },
                        ]
                    })
                    .collect();
                // Every length column names the unit, not only HPV. Four unlabelled thickness
                // columns beside one labelled "(m)" invites the reader to assume the rest match
                // it — which on a foot project they never did.
                let p_len = |name: &str| format!("{name} ({du})");
                let (p_top, p_bottom) = (p_len("Top"), p_len("Bottom"));
                let (p_gross, p_net, p_hpv) = (p_len("Gross"), p_len("Net"), p_len("HPV"));
                let p_cols: [(&str, f64, Anchor); 11] = [
                    ("Zone", usable * 0.16, Anchor::Start),
                    ("Flag", usable * 0.11, Anchor::Start),
                    (p_top.as_str(), usable * 0.08, Anchor::End),
                    (p_bottom.as_str(), usable * 0.08, Anchor::End),
                    (p_gross.as_str(), usable * 0.08, Anchor::End),
                    (p_net.as_str(), usable * 0.08, Anchor::End),
                    ("NTG", usable * 0.07, Anchor::End),
                    ("VSH", usable * 0.07, Anchor::End),
                    ("PHIE", usable * 0.08, Anchor::End),
                    ("SWE", usable * 0.07, Anchor::End),
                    (p_hpv.as_str(), usable * 0.12, Anchor::End),
                ];
                pages.extend(table_pages(&pay_section, &well_name, pay_caveat.as_deref(), &p_cols, &p_rows, pw, ph, 2.4));
            }
            // Ran cleanly but produced no rows: the well has no curve frame, or its zones could not
            // be read. Show the header + an explicit note rather than a blank gap.
            Ok(_) => pages.push(note_page(
                &pay_section,
                &well_name,
                "No pay summary — this well has no curve data to classify.",
                pw,
                ph,
            )),
            // The pay numbers were computed in memory but a storage-side error (read-only DB, disk
            // full, appender failure) failed the FLAG_* write. Surface it in the document instead of
            // dropping the section, and keep the rest of the report (composite pages) intact.
            Err(e) => {
                let message = format!("Pay Summary unavailable — {e}");
                degradations.push(message.clone());
                pages.push(note_page(&pay_section, &well_name, &message, pw, ph));
            }
        }

        // 5 — composite log pages
        if !spec.tables_only {
            pages.extend(composite_pages.into_iter().map(|p| p.ops));
        }

        Ok((pages, pw, ph, well_name, degradations))
    }
}

fn report_pages(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
) -> Result<(Vec<Vec<DrawOp>>, f64, f64, String), String> {
    let (pages, pw, ph, well_name, _) = report_pages_with_degradations(db, pool, spec)?;
    Ok((pages, pw, ph, well_name))
}

/// SVG preview of the whole report (one SVG per page, same shape as the composite result).
pub fn render_report(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
) -> Result<CompositeResult, String> {
    let (pages, pw, ph, well_name) = report_pages(db, pool, spec)?;
    let out = pages
        .iter()
        .enumerate()
        .map(|(i, ops)| CompositePage {
            svg: composite::svg_page(ops, pw, ph),
            top_depth: 0.0,
            bottom_depth: 0.0,
            index: i,
        })
        .collect();
    Ok(CompositeResult {
        pages: out,
        page_width_mm: pw,
        page_height_mm: ph,
        scale: spec.composite.scale,
        well_name,
        // Report ancestry is already visible as complete table pages. This IPC
        // field belongs to standalone composite metadata and is not duplicated.
        ancestry: Vec::new(),
    })
}

/// The full report as one multi-page PDF (bytes).
pub fn render_report_pdf(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
) -> Result<Vec<u8>, String> {
    render_report_pdf_with_degradations(db, pool, spec).map(|(bytes, _)| bytes)
}

fn render_report_pdf_with_degradations(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
) -> Result<(Vec<u8>, Vec<String>), String> {
    let (pages, pw, ph, _, degradations) = report_pages_with_degradations(db, pool, spec)?;
    let streams: Vec<String> = pages.iter().map(|ops| composite::pdf_content(ops, pw, ph)).collect();
    // The report embeds the composite pages verbatim, so it inherits their image tracks —
    // collect the XObjects here too or a report would reference plates it never wrote.
    let op_pages: Vec<&[composite::DrawOp]> = pages.iter().map(|ops| ops.as_slice()).collect();
    let bytes = composite::assemble_pdf_with_images(&streams, pw, ph, &composite::collect_images(&op_pages));
    Ok((bytes, degradations))
}

/// Batch export: one report PDF per well into `dest_dir`, named `<WELL>_report.pdf`.
/// Wells that fail (e.g. no curve data) are skipped with their error collected.
pub fn export_report_batch(
    db: &Mutex<Connection>,
    pool: &crate::reader_pool::ReaderPool,
    spec: &ReportSpec,
    well_ids: &[String],
    dest_dir: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut written = Vec::new();
    let mut errors = Vec::new();
    // Stems already claimed by THIS batch. Only within-batch collisions are suffixed: re-running
    // a batch into the same folder should overwrite its own previous output, which is what the
    // user expects, and suffixing around files already on disk would grow a folder of _2, _3, _4
    // every time they pressed the button.
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
    for wid in well_ids {
        // Resolved BEFORE the render, so the success and failure paths identify the well the same
        // way. They did not: the success path looked the name up for the filename and the failure
        // path reported the raw UUID, so an error you could not attribute and a success that
        // silently replaced a file were the same gap (`docs/review_triage.md` finding 12).
        let name = {
            let conn = db.lock().unwrap();
            conn.query_row("SELECT well_name FROM wells WHERE well_id = ?1", duckdb::params![wid], |r| {
                r.get::<_, String>(0)
            })
            .unwrap_or_else(|_| wid.clone())
        };
        let mut s = spec.clone();
        s.composite.well_id = wid.clone();
        match render_report_pdf_with_degradations(db, pool, &s) {
            Ok((bytes, degradations)) => {
                let path =
                    format!("{}/{}_report.pdf", dest_dir.trim_end_matches(['/', '\\']), unique_stem(&mut used, &name, wid));
                match std::fs::write(&path, &bytes) {
                    Ok(()) => {
                        written.push(path);
                        errors.extend(degradations.into_iter().map(|message| format!("{name}: {message}")));
                    }
                    Err(e) => errors.push(format!("{name}: {e}")),
                }
            }
            Err(e) => errors.push(format!("{name}: {e}")),
        }
    }
    Ok((written, errors))
}

/// One filename stem per well, never two wells on one stem.
///
/// `well_name` carries no uniqueness constraint — two wells can share one, and an import with
/// attach OFF creates a second record under the same name by design. The sanitizer widens the
/// collision further, because every non-alphanumeric maps to `_`, so `SANDI/1` and `SANDI 1` land
/// on one stem too. When they collided the second write silently OVERWROTE the first and both
/// paths were still reported as written, so a 3-well batch said "wrote 3 file(s)" over 2 files on
/// disk and the report kept was the last well's under the first well's name. Nothing in the status
/// line, the Processing panel or the folder said a well was missing.
///
/// A name that sanitizes to nothing at all falls back to the well id — a file called `_report.pdf`
/// is not a deliverable, and the id is at least resolvable.
fn unique_stem(used: &mut std::collections::HashSet<String>, name: &str, well_id: &str) -> String {
    let sanitize = |s: &str| -> String {
        s.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
    };
    let base = {
        let s = sanitize(name);
        if s.trim_matches('_').is_empty() { sanitize(well_id) } else { s }
    };
    if used.insert(base.clone()) {
        return base;
    }
    // `_2` for the second well of that name, matching how a duplicate is usually written by hand.
    for n in 2u32.. {
        let candidate = format!("{base}_{n}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("u32 exhausted")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::db;
    use uuid::Uuid;

    /// Structural computed-input fixture for report tests. The numeric values belong to
    /// each owning test; this helper supplies only the complete custody record required
    /// to persist them, using the fixture well's imported standard curves as named inputs.
    fn write_report_inputs(
        conn: &Connection,
        well_id: &str,
        depth: &[f32],
        curves: &[(&str, &[f32])],
    ) {
        let mut inputs = Vec::new();
        for curve in ["GR", "RES_DEEP", "NPHI"] {
            if let Ok(input) =
                crate::ancestry::resolve_ancestry_input(conn, well_id, curve, curve, None, None)
            {
                inputs.push(input);
            }
        }
        assert!(
            !inputs.is_empty(),
            "the structural fixture must name at least one imported input"
        );
        let ancestry = crate::ancestry::CurveAncestry {
            schema_version: crate::ancestry::CURVE_ANCESTRY_SCHEMA_VERSION,
            method_derivation: None,
            module: "report_test_fixture".into(),
            module_version: env!("CARGO_PKG_VERSION").into(),
            inputs,
            parameters: vec![],
            parameter_state: Some(crate::schema_vocab::ProvenanceAbsentState::NotApplicable),
            zone_scope: crate::ancestry::AncestryZoneScope::WholeWell,
            actor: crate::workflow::test_run_custody().actor,
            timestamp_utc_ms: crate::ancestry::ancestry_timestamp_utc_ms().unwrap(),
            outputs: curves
                .iter()
                .map(|(curve, _)| crate::ancestry::AncestryOutput {
                    curve: (*curve).to_string(),
                    derivation: format!("structural report fixture:{curve}"),
                })
                .collect(),
            depth_frame: None,
            zone_set: None,
            stochastic: None,
            applied_model: None,
            physics_attributes: Vec::new(),
        };
        let spec = crate::ancestry::CompleteLogSetSpec::try_new("REPORT_INPUTS", ancestry).unwrap();
        let (set_id, _) = crate::ancestry::create_complete_log_set(conn, well_id, &spec).unwrap();
        crate::ancestry::write_computed_curves_with_ancestry(conn, well_id, depth, curves, &set_id)
            .unwrap();
    }

    /// A well with curves renders; one without is the "broken well" the batch step puts in
    /// scope. Deliberately the same shape as the composite tests' fixture so the two agree
    /// about what a renderable well is.
    fn seed_batch_well(conn: &Connection, name: &str, with_curves: bool) -> String {
        let wid = Uuid::new_v4();
        db::insert_well(conn, wid, name, Some("Sandi Field"), Some(1300.0), Some(20.0)).unwrap();
        if with_curves {
            let n = 40;
            let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
            db::insert_standard_curves_as_opened_project(
                conn,
                wid,
                depths,
                vec![50.0; n],
                vec![2.0; n],
                vec![0.25; n],
                vec![2.4; n],
                vec![f32::NAN; n],
                vec![f32::NAN; n],
            )
            .unwrap();
        write_report_inputs(
                conn,
                &wid.to_string(),
                &(0..n).map(|i| 1000.0 + i as f32 * 0.5).collect::<Vec<_>>(),
                &[
                    ("VSH", vec![0.20f32; n].as_slice()),
                    ("PHIE", vec![0.20f32; n].as_slice()),
                    ("SWE", vec![0.30f32; n].as_slice()),
                ],
            );
        }
        wid.to_string()
    }

    fn batch_spec() -> ReportSpec {
        ReportSpec {
            input_set: None,
            custody: Some(crate::workflow::test_run_custody()),
            composite: composite::CompositeSpec {
                well_id: String::new(),
                layout: crate::layout::standard_layout(),
                depth_top: None,
                depth_bottom: None,
                scale: 500,
                page_size: composite::PageSize::A4,
            },
            title: "Petrophysical Evaluation".into(),
            author: "Tester".into(),
            methodology: vec![],
            vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(crate::paysummary::CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(crate::paysummary::CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            tables_only: true,
        }
    }

    /// A scratch directory that deletes itself, so a failing assertion never leaves files in
    /// the user's temp folder.
    struct ScratchDir(std::path::PathBuf);
    impl ScratchDir {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!("sandibumi_report_batch_{}", Uuid::new_v4()));
            std::fs::create_dir_all(&p).unwrap();
            ScratchDir(p)
        }
        fn path(&self) -> String {
            self.0.to_string_lossy().to_string()
        }
        fn files(&self) -> Vec<String> {
            let mut v: Vec<String> = std::fs::read_dir(&self.0)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                .collect();
            v.sort();
            v
        }
    }
    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// T-REP-12. One unrenderable well in scope must cost exactly that well. Every healthy
    /// well still gets its own complete PDF, and the broken one leaves NO file behind — not
    /// an empty one, not a truncated one — because `std::fs::write` is only reached after
    /// `render_report_pdf` has returned bytes.
    ///
    /// The broken well is listed FIRST on purpose: a loop that gave up on the first failure
    /// would pass a test that put it last.
    #[test]
    fn one_unrenderable_well_costs_only_itself_in_a_batch_export() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let broken = seed_batch_well(&conn, "SANDI-BATCH-BROKEN", false);
        let a = seed_batch_well(&conn, "SANDI-BATCH-A", true);
        let b = seed_batch_well(&conn, "SANDI-BATCH-B", true);
        let dbm = Mutex::new(conn);

        let dir = ScratchDir::new();
        let (written, errors) =
            export_report_batch(&dbm, &crate::reader_pool::ReaderPool::new(), &batch_spec(), &[broken.clone(), a, b], &dir.path()).unwrap();

        assert_eq!(written.len(), 2, "both healthy wells must be written: {written:?}");
        assert_eq!(errors.len(), 1, "exactly one failure expected: {errors:?}");
        assert_eq!(
            dir.files(),
            vec!["SANDI-BATCH-A_report.pdf".to_string(), "SANDI-BATCH-B_report.pdf".to_string()],
            "the broken well must leave no file at all"
        );

        // The failure names the WELL, not its UUID. It used to report the raw id, because the
        // success path looked the name up for the filename and the error path did not — so the
        // status line read "failed: 3f2a…: no curve data" and the user could not tell which well
        // that was (`docs/review_triage.md` finding 12). One lookup before the render now serves
        // both paths, which is what makes them agree rather than merely both being correct.
        assert!(errors[0].starts_with("SANDI-BATCH-BROKEN"), "the error identifies the well: {}", errors[0]);
        assert!(errors[0].contains("no curve data for this well"), "and states the reason: {}", errors[0]);
        assert!(!errors[0].contains(&broken), "and does not make the reader resolve a UUID: {}", errors[0]);

        // Each file is a real PDF for ITS OWN well, not two copies of the first.
        for f in dir.files() {
            let bytes = std::fs::read(self_path(&dir, &f)).unwrap();
            assert!(bytes.starts_with(b"%PDF"), "{f} is not a PDF");
            assert!(bytes.len() > 500, "{f} is too small to be a report");
        }
        let pa = std::fs::read(self_path(&dir, "SANDI-BATCH-A_report.pdf")).unwrap();
        let pb = std::fs::read(self_path(&dir, "SANDI-BATCH-B_report.pdf")).unwrap();
        assert_ne!(pa, pb, "each well's report must be its own — identical bytes means the cover well never changed");
    }

    /// SB-CORE-002 / SB-CORE-T07. CORRECTNESS: `04_CORE_REQUIREMENTS.md` assigns the
    /// recovered R4 contract to both user-facing artefacts. A section dependency failure
    /// degrades the PDF instead of suppressing it, and the same degradation must be named
    /// in the batch result rather than counted as an unqualified success.
    #[test]
    fn a_failed_pay_summary_is_named_in_the_pdf_and_in_the_batch_run_record() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let well_id = seed_batch_well(&conn, "PAY_SECTION_DEPENDENCY_MISSING", true);
        conn.execute_batch(
            "ALTER TABLE computed_curves RENAME TO computed_curves_read_source;
             CREATE VIEW computed_curves AS SELECT * FROM computed_curves_read_source;",
        )
        .unwrap();
        let dbm = Mutex::new(conn);
        let dir = ScratchDir::new();

        let (written, errors) =
            export_report_batch(&dbm, &crate::reader_pool::ReaderPool::new(), &batch_spec(), &[well_id], &dir.path()).unwrap();

        assert_eq!(written.len(), 1, "the intact report sections must still be delivered");
        assert_eq!(dir.files(), vec!["PAY_SECTION_DEPENDENCY_MISSING_report.pdf"]);
        let bytes = std::fs::read(&written[0]).unwrap();
        assert!(bytes.starts_with(b"%PDF"), "the delivered artefact must be a PDF");
        let pdf = String::from_utf8_lossy(&bytes);
        assert!(pdf.contains("Pay Summary"), "the failed section heading must remain in the PDF");
        assert!(pdf.contains("Pay Summary unavailable"), "the PDF must name the section degradation");

        assert_eq!(errors.len(), 1, "one degraded section must produce one batch-record entry");
        assert!(errors[0].starts_with("PAY_SECTION_DEPENDENCY_MISSING:"), "the batch record names the well: {}", errors[0]);
        assert!(errors[0].contains("Pay Summary unavailable"), "the batch record names the failed section: {}", errors[0]);
    }

    fn self_path(dir: &ScratchDir, name: &str) -> std::path::PathBuf {
        dir.0.join(name)
    }

    /// T-REP-12, second half. Two wells whose names sanitize to the SAME string used to write to
    /// one path: the second silently overwrote the first, yet BOTH paths were returned as written,
    /// so the status line reported "wrote 3 file(s)" over 2 files on disk and the report kept was
    /// the last well's under the first well's name (`docs/review_triage.md` finding 12, fixed
    /// 2026-08-01 by suffixing).
    ///
    /// Not hypothetical — `well_name` carries no uniqueness constraint, and an import with
    /// attach OFF creates a second record under the same name by design. The sanitizer widens
    /// it further: every non-alphanumeric maps to `_`, so `SANDI/DUP` and `SANDI DUP` land on one
    /// stem even though the names are distinct.
    ///
    /// BOTH routes are in this fixture, deliberately. A fix that only compared well NAMES would
    /// still let the sanitizer collision write over a file, and would look entirely correct in a
    /// test that only used identical names.
    #[test]
    fn two_wells_with_one_name_each_get_their_own_report_file() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wells = [
            seed_batch_well(&conn, "SANDI-DUP", true),  // -> SANDI-DUP
            seed_batch_well(&conn, "SANDI/DUP", true),  // -> SANDI_DUP
            seed_batch_well(&conn, "SANDI-DUP", true),  // same NAME       -> SANDI-DUP_2
            seed_batch_well(&conn, "SANDI DUP", true),  // same STEM only  -> SANDI_DUP_2
        ];
        let dbm = Mutex::new(conn);

        let dir = ScratchDir::new();
        let (written, errors) = export_report_batch(&dbm, &crate::reader_pool::ReaderPool::new(), &batch_spec(), &wells, &dir.path()).unwrap();

        assert!(errors.is_empty(), "all four wells render: {errors:?}");
        assert_eq!(written.len(), 4, "four wells, four reports");
        assert_eq!(
            dir.files().len(),
            4,
            "and four files on disk — the count the caller is given must be true: {:?}",
            dir.files()
        );
        assert_eq!(
            written.iter().collect::<std::collections::HashSet<_>>().len(),
            4,
            "no two wells may share a path: {written:?}"
        );

        // The FIRST well of a colliding pair keeps the plain name, so a folder delivered to a
        // client does not suddenly rename the well anybody was expecting.
        for (i, stem) in ["SANDI-DUP", "SANDI_DUP", "SANDI-DUP_2", "SANDI_DUP_2"].iter().enumerate() {
            assert!(written[i].ends_with(&format!("{stem}_report.pdf")), "well {i}: {}", written[i]);
        }

        // Each file really is its own well's render rather than a copy of the first — checked on
        // the pair whose NAMES differ, since the cover carries the name. (Wells 0 and 2 share a
        // name and identical synthetic curves, so identical bytes there is correct, not a bug.)
        let bytes: Vec<Vec<u8>> = written.iter().map(|p| std::fs::read(p).unwrap()).collect();
        assert_ne!(bytes[0], bytes[1], "the loop must re-render per well, not copy the first");
    }

    /// A permeability cutoff a well has no data for is stated ON the pay page.
    ///
    /// Since 2026-08-01 the cutoff applies to every well it is asked for (`docs/review_triage.md`
    /// finding 7, Jauhar: *"no relation between em, wells still can have perm curves"*), so a well
    /// carrying no PERM books zero pay rather than full pay. That is the defensible answer and it
    /// creates its own reading problem: on the page, zero pay for want of a curve looks exactly
    /// like zero pay because the reservoir is wet. The note is what separates them, and it belongs
    /// in the deliverable rather than only in the row struct, because the deliverable is where a
    /// client actually meets the number.
    #[test]
    fn a_perm_cutoff_the_well_has_no_data_for_is_stated_on_the_pay_page() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-NOPERM", None, None, None).unwrap();
        let w = wid.to_string();
        let n = 20usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depth.clone(), vec![50.0; n], vec![2.0; n], vec![0.25; n],
            vec![2.4; n], nan.clone(), nan,
        )
        .unwrap();
        // Pay rock, and deliberately NO PERM curve anywhere in the well.
        write_report_inputs(&conn, &w,
            &depth,
            &[("VSH", vec![0.2f32; n].as_slice()), ("PHIE", vec![0.20f32; n].as_slice()), ("SWE", vec![0.30f32; n].as_slice()),
            ],
        );
        let dbm = Mutex::new(conn);
        let mut spec = batch_spec();
        spec.composite.well_id = w;
        spec.tables_only = true;

        let pay_text = |spec: &ReportSpec| -> String {
            let (pages, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), spec).expect("render");
            pages
                .last()
                .unwrap()
                .iter()
                .filter_map(|op| match op {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        };

        // Control first: with no cutoff requested there is nothing to say, and a note that
        // appeared anyway would be on every report anyone ever ran.
        spec.perm_min = None;
        let quiet = pay_text(&spec);
        assert!(!quiet.contains("no permeability curve"), "no cutoff requested, no note: {quiet}");

        spec.perm_min = Some(crate::paysummary::CutoffEntry { value: 1000.0, unit: "mD".into() }.into());
        let noted = pay_text(&spec);
        assert!(noted.contains("no permeability curve"), "the reason must be stated: {noted}");
        assert!(noted.contains("1000.0 mD"), "and name the cutoff it could not answer: {noted}");
        assert!(
            noted.contains("absence of evidence"),
            "and say what the zero below actually means, which is the whole point: {noted}"
        );
        // The zero it is explaining must really be there — a note about a number the page does not
        // print would be worse than no note.
        assert!(noted.contains("PAY"), "the pay row is on the page: {noted}");
    }

    /// From the Codex whole-repository review of a6565bd9 (P0). The report PDF is the client
    /// deliverable, and every depth and thickness it prints comes straight off the project's own
    /// depth column — `run_pay_summary` accumulates raw sample thickness and the zone tops are
    /// stored depths. The headings said "(m)" unconditionally, so a foot-declared project handed
    /// a client a document stating net pay and hydrocarbon pore thickness in metres over numbers
    /// that were feet: both overstated by 3.28084x, every figure plausible, nothing on the page
    /// to catch it.
    ///
    /// Two arms because a fix that just wrote "ft" everywhere would be as wrong as the original.
    /// The metre arm must still read "(m)", and the four thickness columns that carried NO unit
    /// at all — Top, Bottom, Gross, Net, beside a labelled HPV — must now name it too: unlabelled
    /// columns next to one that says metres invite exactly the assumption that was false.
    #[test]
    fn the_report_pdf_heads_its_thicknesses_in_the_projects_own_depth_unit() {
        use crate::units::DepthUnit;

        let headings = |feet: bool| -> String {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            if feet {
                crate::units::set_project_depth_unit(&conn, DepthUnit::Feet).unwrap();
            }
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-UNITS", None, None, None).unwrap();
            let w = wid.to_string();
            let n = 20usize;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves_as_opened_project(
                &conn, wid, depth.clone(), vec![50.0; n], vec![2.0; n], vec![0.25; n],
                vec![2.4; n], nan.clone(), nan,
            )
            .unwrap();
            db::upsert_md_zone(&conn, &w, "ZoneA", 1000.0, 1009.5).unwrap();
            crate::db::set_zone_param(&conn, &w, "ZoneA", "RW", Some(0.3), None).unwrap();
            write_report_inputs(
                &conn,
                &w,
                &depth,
                &[
                    ("VSH", vec![0.2f32; n].as_slice()),
                    ("PHIE", vec![0.20f32; n].as_slice()),
                    ("SWE", vec![0.30f32; n].as_slice()),
                ],
            );
            let dbm = Mutex::new(conn);
            let mut spec = batch_spec();
            spec.composite.well_id = w;
            spec.tables_only = true;
            let (pages, _pw, _ph, _name) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("render");
            pages
                .iter()
                .flatten()
                .filter_map(|op| match op {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        };

        let metric = headings(false);
        assert!(metric.contains("HPV (m)"), "a metre project must be unchanged: {metric}");
        assert!(metric.contains("Top (m)"), "and its zone table too: {metric}");
        assert!(metric.contains("1009.5 m"), "and its cover interval: {metric}");
        assert!(!metric.contains("(ft)"), "with no feet anywhere: {metric}");

        let foot = headings(true);
        assert!(foot.contains("1009.5 ft"), "the cover states the interval in feet: {foot}");
        for heading in ["Top (ft)", "Bottom (ft)", "Gross (ft)", "Net (ft)", "HPV (ft)"] {
            assert!(
                foot.contains(heading),
                "a foot project must head every thickness column {heading}: {foot}"
            );
        }
        assert!(!foot.contains("(m)"), "and must not print metres anywhere: {foot}");
    }

    /// A well whose name sanitizes to nothing at all still gets a usable filename. `_report.pdf`
    /// is not a deliverable, and two such wells would collide on it.
    #[test]
    fn a_well_whose_name_sanitizes_to_nothing_falls_back_to_its_id() {
        let mut used = std::collections::HashSet::new();
        let stem = unique_stem(&mut used, "###", "3f2a-9910");
        assert_eq!(stem, "3f2a-9910", "the id is at least resolvable: {stem}");
        assert_ne!(unique_stem(&mut used, "!!!", "3f2a-9910"), stem, "and two of them still differ");
    }

    /// T-REP-09. "Tables only" must produce a document that is COMPLETE without the composite
    /// pages — which means the cover still has to state a real logged interval. A cover reading
    /// "Interval: 0.0 – 0.0 m" would be a client deliverable that says the well was never logged.
    ///
    /// This also pins the shape of the audit's known slowness. `report_pages` renders the
    /// composite UNCONDITIONALLY at `report.rs:314` and only skips APPENDING it at `:463`, which
    /// looks like a missing `if` — and is not. The comment at `:312` says why: "Composite pages
    /// (also gives the true interval for the cover)". The interval is read straight off the
    /// composite pagination (`:319`), so the expensive render is what supplies the cover's one
    /// remaining fact. Anyone making tables-only fast has to give the cover its own cheap source
    /// (a MIN/MAX depth query) FIRST, or the fix trades a slow report for a wrong one.
    #[test]
    fn tables_only_drops_the_composite_pages_and_still_dates_the_cover_to_real_rock() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-REP-09", Some("Sandi Field"), Some(1300.0), Some(20.0)).unwrap();
        let w = wid.to_string();

        // Logged 1000.0 .. 1019.5 — deliberately not starting at zero, so a cover that lost the
        // interval cannot pass by accident.
        let n = 40usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depth.clone(), vec![50.0; n], vec![2.0; n], vec![0.25; n],
            vec![2.4; n], nan.clone(), nan,
        )
        .unwrap();
        write_report_inputs(&conn, &w,
            &depth,
            &[("VSH", vec![0.2f32; n].as_slice()), ("PHIE", vec![0.20f32; n].as_slice()), ("SWE", vec![0.30f32; n].as_slice()),
            ],
        );
        db::upsert_md_zone(&conn, &w, "UPPER", 1000.0, 1010.0).unwrap();

        let dbm = Mutex::new(conn);
        let text_of = |ops: &Vec<DrawOp>| -> String {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };

        let mut spec = batch_spec();
        spec.composite.well_id = w.clone();
        spec.tables_only = true;
        let (tables, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("tables-only render");
        let t_texts: Vec<String> = tables.iter().map(text_of).collect();

        // Exactly the five sections, in order, with the complete ancestry table
        // spanning two pages and nothing after the pay summary.
        assert_eq!(tables.len(), 6, "cover + methodology + computed ancestry + zone params + pay summary: {t_texts:#?}"
        );
        assert!(t_texts[0].contains("Well: SANDI-REP-09"), "page 1 is the cover: {}", t_texts[0]);
        assert!(t_texts[1].contains("Methodology"), "{}", t_texts[1]);
        assert!(t_texts[2].contains("Computed curve ancestry"), "{}", t_texts[2]);
        assert!(t_texts[3].contains("Computed curve ancestry"),
            "{}",
            t_texts[3]
        );
        assert!(t_texts[4].contains("Zone Parameters"), "{}", t_texts[4]);
        assert!(t_texts[5].contains("Pay Summary"), "{}", t_texts[5]);

        // The cover dates the study to the rock that was actually logged.
        assert!(
            t_texts[0].contains("Interval: 1000.0 \u{2013} 1019.5 m"),
            "the cover must state the true logged interval: {}",
            t_texts[0]
        );
        assert!(t_texts[0].contains("TD: 1300.0 m") && t_texts[0].contains("KB: 20.0 m"));

        // The control: with the composite ON, those same four pages are still the first four and
        // the log pages come after. Without this, a tables-only mode that silently dropped a
        // table would pass every assertion above.
        let mut full = spec.clone();
        full.tables_only = false;
        let (all, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &full).expect("full render");
        assert!(all.len() > tables.len(), "the composite pages must actually be appended");
        for i in 0..4 {
            assert_eq!(text_of(&all[i]), t_texts[i], "page {i} differs between the two modes");
        }
    }

    /// The cover's interval used to be read off the COMPOSITE pagination, which honours the
    /// render's depth window — so setting one re-dated the whole report, including the tables the
    /// window never touched (`docs/review_triage.md` finding 18, fixed 2026-08-01).
    ///
    /// The pay summary is computed per ZONE by `run_pay_summary` and knows nothing about the
    /// composite window, so a report rendered over 1005–1010 carried a pay table covering every
    /// zone in the well under a cover announcing a 5 m interval — and on a tables-only render
    /// there were no log pages left to show the reader that the window was only ever a print
    /// setting. That is the case this fixture builds: its one zone sits ENTIRELY outside the
    /// window, so a cover that quoted the window would contradict the table on the next page.
    ///
    /// Both modes are checked, because the window means different things in them: on a full
    /// render it describes log pages that exist and is worth stating; on tables-only it describes
    /// nothing in the document and would be noise.
    #[test]
    fn a_cover_dates_itself_to_the_log_and_states_a_print_window_separately() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-REP-09B", None, None, None).unwrap();
        let w = wid.to_string();
        let n = 40usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depth.clone(), vec![50.0; n], vec![2.0; n], vec![0.25; n],
            vec![2.4; n], nan.clone(), nan,
        )
        .unwrap();
        write_report_inputs(&conn, &w,
            &depth,
            &[("VSH", vec![0.2f32; n].as_slice()), ("PHIE", vec![0.20f32; n].as_slice()), ("SWE", vec![0.30f32; n].as_slice()),
            ],
        );
        // One zone, wholly OUTSIDE the print window below.
        db::upsert_md_zone(&conn, &w, "DEEP", 1012.0, 1019.0).unwrap();

        let dbm = Mutex::new(conn);
        let mut spec = batch_spec();
        spec.composite.well_id = w;
        spec.tables_only = true;
        spec.composite.depth_top = Some(1005.0);
        spec.composite.depth_bottom = Some(1010.0);

        let joined = |page: &Vec<DrawOp>| -> String {
            page.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };

        let (pages, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("render");
        let cover = joined(&pages[0]);

        assert!(
            cover.contains("Interval: 1000.0 \u{2013} 1019.5 m"),
            "the cover dates itself to the LOGGED interval, not the 1005-1010 print window: {cover}"
        );
        assert!(
            !cover.contains("printed over"),
            "and on tables-only the window describes nothing in the document, so it is not stated: {cover}"
        );
        // The table underneath now AGREES with the cover: zone DEEP (1012-1019) sits inside the
        // stated interval, where before it fell outside the 5 m the cover announced.
        assert!(joined(&pages[3]).contains("DEEP"), "the pay table reports zone DEEP: {}", joined(&pages[3]));

        // Same window, full render: the log pages exist, so the window is worth stating — beside
        // the interval, never instead of it.
        spec.tables_only = false;
        let (full, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("full render");
        let cover = joined(&full[0]);
        assert!(cover.contains("Interval: 1000.0 \u{2013} 1019.5 m"), "{cover}");
        assert!(
            cover.contains("printed over 1005.0 \u{2013} 1010.0 m"),
            "a print window is stated as what it is: {cover}"
        );

        // And with no window at all there is nothing to state — a report that always carried the
        // line would train the reader to skip it.
        spec.composite.depth_top = None;
        spec.composite.depth_bottom = None;
        let (plain, _pw, _ph, _n) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("unwindowed render");
        assert!(!joined(&plain[0]).contains("printed over"), "{}", joined(&plain[0]));
    }

    /// T-REP-06. The report is the client deliverable, so its structure and its pay numbers are
    /// checked together: the page ORDER the plan describes (cover, methodology, zone parameters,
    /// pay summary), and the domain invariants the reader relies on to trust the table —
    /// Net <= Gross, 0 <= NTG <= 1, HPV >= 0, and PAY within RESERVOIR within SAND.
    ///
    /// The nesting is guaranteed at the SAMPLE level by `classify_sample` (pay is defined as
    /// reservoir AND ..., reservoir as sand AND ...), but the printed rows are thickness sums
    /// with their own zone clamping, so the property has to be re-checked where it is read.
    #[test]
    fn a_rendered_report_carries_the_plans_page_order_and_a_self_consistent_pay_table() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "SANDI-REP-06", Some("Sandi Field"), Some(1300.0), Some(20.0)).unwrap();
        let w = wid.to_string();

        // 40 samples at 0.5 m over two 10 m zones. Every fourth sample is deliberately stopped
        // at a different cutoff, so the three flags cannot collapse onto each other: a fixture
        // where SAND == PAY would let a broken nesting rule pass.
        let n = 40usize;
        let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let nan = vec![f32::NAN; n];
        db::insert_standard_curves_as_opened_project(
            &conn, wid, depth.clone(), vec![50.0; n], vec![2.0; n], vec![0.25; n],
            vec![2.4; n], nan.clone(), nan,
        )
        .unwrap();

        let mut vsh = Vec::with_capacity(n);
        let mut phie = Vec::with_capacity(n);
        let mut swe = Vec::with_capacity(n);
        for i in 0..n {
            match i % 4 {
                0 => { vsh.push(0.80); phie.push(0.20); swe.push(0.30); } // shale — fails SAND
                1 => { vsh.push(0.20); phie.push(0.05); swe.push(0.30); } // tight — fails RESERVOIR
                2 => { vsh.push(0.20); phie.push(0.20); swe.push(0.90); } // wet   — fails PAY
                _ => { vsh.push(0.20); phie.push(0.20); swe.push(0.30); } // pay
            }
        }
        write_report_inputs(&conn, &w,
            &depth,
            &[("VSH", vsh.as_slice()), ("PHIE", phie.as_slice()), ("SWE", swe.as_slice())] );
        db::upsert_md_zone(&conn, &w, "UPPER", 1000.0, 1010.0).unwrap();
        db::upsert_md_zone(&conn, &w, "LOWER", 1010.0, 1020.0).unwrap();
        db::set_zone_param(&conn, &w, "UPPER", "RW", Some(0.02), None).unwrap();

        let dbm = Mutex::new(conn);
        let mut spec = batch_spec();
        spec.composite.well_id = w.clone();
        spec.title = "Petrophysical Evaluation".into();
        spec.author = "Tester".into();

        // --- the numbers, before asking what they look like on the page ---
        let rows = run_pay_summary(
            &dbm,
            &crate::reader_pool::ReaderPool::new(),
            &PaySummaryRequest {
            // SB-CUT-001 (DEC-071): exports run the product-default model (CENTRED).
            discretisation: Default::default(),
                well_ids: vec![w.clone()],
                vsh_max: spec.vsh_max.clone(),
                phie_min: spec.phie_min.clone(),
                swe_max: spec.swe_max.clone(),
                enabled_unset: Vec::new(),
                cutoff_use: Default::default(),
                perm_min: spec.perm_min.clone(),
                input_set: spec.input_set.clone(),
                skip_version: true,
                stats_only: true,
                custody: None,
                frame: Default::default(),
                weighting: Default::default(),
            },
        )
        .expect("pay summary");
        assert_eq!(rows.len(), 6, "two zones x three flags");

        for r in &rows {
            assert!(r.net <= r.gross + 1e-3, "{} {}: net {} exceeds gross {}", r.zone, r.flag, r.net, r.gross);
            assert!((0.0..=1.0).contains(&r.ntg), "{} {}: NTG out of range: {}", r.zone, r.flag, r.ntg);
            assert!(r.hpv >= 0.0, "{} {}: negative HPV {}", r.zone, r.flag, r.hpv);
            assert!(r.n_classified > 0, "{} {}: the fixture is interpreted everywhere", r.zone, r.flag);
        }

        for zone in ["UPPER", "LOWER"] {
            let net = |flag: &str| -> f32 {
                rows.iter().find(|r| r.zone == zone && r.flag == flag).unwrap().net
            };
            let (sand, res, pay) = (net("SAND"), net("RESERVOIR"), net("PAY"));
            // Strictly decreasing, not merely non-increasing: each cutoff must actually be
            // rejecting something, or the invariant is being satisfied by an inert fixture.
            assert!(sand > res && res > pay, "{zone}: SAND {sand} > RESERVOIR {res} > PAY {pay}");
            assert!((sand - 7.5).abs() < 1e-3, "{zone}: three of every four samples are sand: {sand}");
            assert!((res - 5.0).abs() < 1e-3, "{zone}: two of every four clear the porosity cutoff: {res}");
            assert!((pay - 2.5).abs() < 1e-3, "{zone}: one of every four is pay: {pay}");
        }

        // --- and now the document ---
        let (pages, _pw, _ph, well_name) = report_pages(&dbm, &crate::reader_pool::ReaderPool::new(), &spec).expect("render");
        assert_eq!(well_name, "SANDI-REP-06");
        let text_of = |ops: &Vec<DrawOp>| -> String {
            ops.iter()
                .filter_map(|op| match op {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };
        let texts: Vec<String> = pages.iter().map(text_of).collect();
        let first_with = |needle: &str| -> usize {
            texts
                .iter()
                .position(|t| t.contains(needle))
                .unwrap_or_else(|| panic!("no page carries {needle:?}; pages were {texts:#?}"))
        };

        // Page 1 is the cover.
        assert!(texts[0].contains("Petrophysical Evaluation"), "cover title: {}", texts[0]);
        assert!(texts[0].contains("Well: SANDI-REP-06"), "cover well: {}", texts[0]);
        assert!(texts[0].contains("Prepared by: Tester"), "cover author: {}", texts[0]);

        // Then methodology, zone parameters, pay summary — in that order. Located by first
        // occurrence rather than by index, so a table that paginates does not break the check.
        // SB-CUT-019 tightened this: the deliverable names the UNIT beside every cut-off, so a
        // client PDF can never say "PHIE ≥ 0.10" without saying in what.
        let pay_section =
            "Pay Summary  (VSH \u{2264} 0.50 v/v, PHIE \u{2265} 0.10 v/v, SWE \u{2264} 0.60 v/v, CENTRED model, step 0.5000)";
        let (m, z, p) = (first_with("Methodology"), first_with("Zone Parameters"), first_with(pay_section));
        assert!(0 < m && m < z && z < p, "page order was cover {m} methodology, {z} zones, {p} pay");

        // The zone parameter the run was made with has to be ON the page — a report that states
        // its cutoffs but not its overrides cannot be reproduced from itself.
        assert!(texts[z].contains("RW"), "the RW override must be listed: {}", texts[z]);
        assert!(texts[z].contains("0.02"), "the override VALUE must be listed: {}", texts[z]);
        assert!(texts[z].contains("LOWER"), "a zone with no override is still listed: {}", texts[z]);

        // And the printed pay table carries the numbers checked above, so the invariants are
        // pinned to what SHIPS rather than to an intermediate the reader never sees.
        assert!(texts[p].contains("SAND") && texts[p].contains("RESERVOIR") && texts[p].contains("PAY"));
        for net in ["7.5", "5.0", "2.5"] {
            assert!(texts[p].contains(net), "printed net {net} missing from: {}", texts[p]);
        }
        // EVERY page carries the "Made in SandiBumi" mark, which is what T-REP-06 always asked
        // for. The table pages used to be the one unmarked surface in the whole deliverable set —
        // the cover has it, so does every composite page, the Word document and the PowerPoint
        // deck — so a reader who extracted or photocopied the pay summary got an unattributed
        // page (`docs/review_triage.md` finding 15, fixed 2026-08-01).
        //
        // Asserted over every page rather than a sample, because the failure mode is one page
        // type being missed, and a spot check is how it stayed missed.
        for (i, t) in texts.iter().enumerate() {
            assert!(t.contains("Made in SandiBumi"), "page {i} carries no mark: {t}");
        }

        // tables_only keeps the composite out, so the document ends at the pay summary.
        assert_eq!(p, pages.len() - 1, "tables_only must not append composite pages");
    }

    /// T-REP-06, second half. "HPV >= 0" is listed as a domain check the reader applies to the
    /// printed table, and until 2026-08-01 it was merely true of tidy data rather than an
    /// invariant: the pay summary summed `PHIE * (1 - SWE) * h` with no floor, so the SAND row
    /// inherited the sign of PHIE.
    ///
    /// The route is ordinary rather than exotic. A tight carbonate streak reads low GR, so it
    /// clears the VSH cutoff and is flagged SAND; a density porosity computed on a sandstone
    /// matrix reads slightly NEGATIVE there, which is a routine artefact of a vendor PHIE and
    /// not a corrupt curve. Its contribution was then subtracted from the SAND row's HPV —
    /// measured at more than 20 % below the floored answer, in the reassuring direction, while
    /// RESERVOIR and PAY stayed byte-identical because the streak fails the porosity cutoff. The
    /// two rows anyone checks first agreed with each other while the third quietly did not.
    ///
    /// Jauhar's call, 2026-08-01 (`docs/review_triage.md` finding 16): *"always limit phie to
    /// 0.001"*. `paysummary::floored_phie` applies it to every pay calculation, so a curve from ANY
    /// source is covered — the porosity modules floor what they write, but a vendor PHIE never
    /// passes through one, and the vendor curve is the whole scenario.
    ///
    /// The last assertion is the one that keeps the floor honest: a floored streak must still FAIL
    /// the porosity cutoff. A floor set anywhere near `phie_min` would stop a dense stringer being
    /// subtracted by quietly promoting it into reservoir instead, which is a worse answer wearing
    /// a better-looking number.
    #[test]
    fn a_dense_stringer_no_longer_subtracts_from_the_sand_rows_hpv() {
        let build = |phie_streak: f32| -> Vec<crate::paysummary::PaySummaryRow> {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-STRINGER", None, None, None).unwrap();
            let w = wid.to_string();
            let n = 10usize;
            let depth: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
            let nan = vec![f32::NAN; n];
            db::insert_standard_curves_as_opened_project(
                &conn, wid, depth.clone(), vec![30.0; n], vec![2.0; n], vec![0.2; n],
                vec![2.4; n], nan.clone(), nan,
            )
            .unwrap();
            // Clean throughout — every sample clears the VSH cutoff — with a 2.5 m tight streak
            // through the middle. Only its porosity differs between the two runs.
            let mut phie = vec![0.20f32; n];
            for p in phie.iter_mut().take(8).skip(3) {
                *p = phie_streak;
            }
            write_report_inputs(&conn, &w, &depth, &[
                    ("VSH", vec![0.10f32; n].as_slice()),
                    ("PHIE", phie.as_slice()),
                    ("SWE", vec![0.30f32; n].as_slice()),
                ],
            );
            db::upsert_md_zone(&conn, &w, "SAND-A", 1000.0, 1005.0).unwrap();
            let dbm = Mutex::new(conn);
            run_pay_summary(
                &dbm,
                &crate::reader_pool::ReaderPool::new(),
                &PaySummaryRequest {
            // SB-CUT-001 (DEC-071): exports run the product-default model (CENTRED).
            discretisation: Default::default(),
                    input_set: None,
                    well_ids: vec![w],
                    vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
                    phie_min: Some(crate::paysummary::CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
                    swe_max: Some(crate::paysummary::CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
                    enabled_unset: Vec::new(),
                    cutoff_use: Default::default(),
                    perm_min: None,
                    skip_version: true,
                    stats_only: true,
                    custody: None,
                    frame: Default::default(),
                    weighting: Default::default(),
                },
            )
            .expect("pay summary")
        };
        let hpv = |rows: &[crate::paysummary::PaySummaryRow], flag: &str| -> f32 {
            rows.iter().find(|r| r.flag == flag).unwrap().hpv
        };

        // A tight streak at zero porosity: contributes no hydrocarbon, which is the honest answer.
        let floored = build(0.0);
        // The same streak as a vendor PHIE actually delivers it.
        let negative = build(-0.05);
        // And an absurd one, to show the floor does not scale with how wrong the input was.
        let absurd = build(-5.0);

        assert!(hpv(&floored, "SAND") > 0.0, "control SAND HPV: {}", hpv(&floored, "SAND"));
        assert_eq!(
            hpv(&negative, "SAND"),
            hpv(&floored, "SAND"),
            "a negative PHIE inside net sand must not be subtracted from HPV"
        );
        assert_eq!(
            hpv(&absurd, "SAND"),
            hpv(&floored, "SAND"),
            "the floor is a floor, not a proportional correction"
        );

        // "HPV >= 0" is now an invariant rather than a hope, which is what T-REP-06 asserts.
        for rows in [&floored, &negative, &absurd] {
            for r in rows.iter() {
                assert!(r.hpv >= 0.0, "HPV must never be negative, was {} on {}", r.hpv, r.flag);
            }
        }

        // RESERVOIR and PAY are untouched, as they always were — the streak fails the porosity
        // cutoff. This was the reason the SAND-row error was easy to miss, and it stays true:
        // the floor must NOT smuggle the streak into reservoir.
        assert_eq!(hpv(&negative, "RESERVOIR"), hpv(&floored, "RESERVOIR"));
        assert_eq!(hpv(&negative, "PAY"), hpv(&floored, "PAY"));
        let net = |rows: &[crate::paysummary::PaySummaryRow], flag: &str| -> f32 {
            rows.iter().find(|r| r.flag == flag).unwrap().net
        };
        assert!(
            net(&negative, "SAND") > net(&negative, "RESERVOIR"),
            "a floored streak is still 0.001 v/v and must stay well below the 0.1 porosity cutoff"
        );
    }

    #[test]
    fn wrap_respects_width_and_splits_long_words() {
        let lines = wrap("porosity from density neutron crossplot", 12);
        assert!(lines.iter().all(|l| l.chars().count() <= 12));
        assert!(lines.len() >= 3);
        let long = wrap("supercalifragilistic", 8);
        assert!(long.len() >= 3 && long.iter().all(|l| l.chars().count() <= 8));
    }

    #[test]
    fn table_paginates_when_rows_overflow() {
        let cols: [(&str, f64, Anchor); 2] =
            [("A", 60.0, Anchor::Start), ("B", 100.0, Anchor::Start)];
        let rows: Vec<Vec<String>> =
            (0..200).map(|i| vec![format!("row {i}"), "some content that is short".into()]).collect();
        let pages = table_pages("Test", "WELL-1", None, &cols, &rows, 210.0, 297.0, 2.7);
        assert!(pages.len() > 1, "200 rows must overflow one A4 page");
        // Every page carries a header row (the shaded rect) and at least one text op.
        for p in &pages {
            assert!(p.iter().any(|op| matches!(op, DrawOp::Rect { fill: Some(f), .. } if f == "#e8e4da")));
        }
    }

    #[test]
    fn note_page_shows_section_header_and_message() {
        // The section header and the note text must both render — this is what stops a failed or
        // empty pay run from leaving no trace at all in the client PDF.
        let ops = note_page(
            "Pay Summary  (VSH ≤ 0.50 v/v, PHIE ≥ 0.10 v/v, SWE ≤ 0.60 v/v)",
            "SANDI-001",
            "Pay Summary unavailable — read-only database",
            210.0,
            297.0,
        );
        let texts: Vec<String> = ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { s, .. } => Some(s.clone()),
                _ => None,
            })
            .collect();
        let joined = texts.join(" ");
        assert!(joined.contains("Pay Summary"), "section header must be present");
        assert!(texts.iter().any(|s| s.contains("SANDI-001")), "well name must be present");
        assert!(joined.contains("unavailable"), "the failure note must render, not be dropped");
    }

    #[test]
    fn cover_page_carries_title_and_well() {
        let spec = ReportSpec {
            input_set: None,
            custody: None,
            composite: serde_json::from_str(
                r#"{"well_id":"w1","layout":{"name":"t","tracks":[]},"scale":200,"page_size":"a4"}"#,
            )
            .unwrap(),
            title: "Petrophysical Evaluation — Sandi Field".into(),
            author: "Jauhar".into(),
            methodology: vec![],
            vsh_max: Some(crate::paysummary::CutoffEntry { value: 0.5, unit: "v/v".into() }.into()),
            phie_min: Some(crate::paysummary::CutoffEntry { value: 0.1, unit: "v/v".into() }.into()),
            swe_max: Some(crate::paysummary::CutoffEntry { value: 0.6, unit: "v/v".into() }.into()),
            perm_min: None,
            tables_only: true,
        };
        let header = composite::WellHeader {
            name: "SANDI-001".into(),
            field: Some("Sandi".into()),
            td: Some(900.0),
            kb: Some(15.0),
        };
        let ops =
            cover_page(&spec, &header, (400.0, 850.0), None, crate::units::DepthUnit::Metres, 210.0, 297.0);
        let texts: Vec<String> = ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { s, .. } => Some(s.clone()),
                _ => None,
            })
            .collect();
        // The title may wrap across lines — check the joined text.
        let joined = texts.join(" ");
        assert!(joined.contains("Sandi") && joined.contains("Field"));
        assert!(texts.iter().any(|s| s.contains("SANDI-001")));
        assert!(texts.iter().any(|s| s.contains("Jauhar")));
        assert!(texts.iter().any(|s| s.contains("400.0")));
    }
}
