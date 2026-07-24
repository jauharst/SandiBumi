//! Report generator (Phase 8b) — assembles the client-style petrophysics report PDF
//! following Jauhar's real report structure (BLSO / Bunga template): cover page →
//! methodology (parameter–method–remarks table) → per-zone parameter table → pay
//! summary (cutoffs) → composite log pages. Every page is a `Vec<DrawOp>` in mm
//! space, serialized through the same SVG/PDF machinery as the composite plot
//! (`composite::svg_page` / `pdf_content` / `assemble_pdf`), so one command yields
//! one client-ready multi-page PDF (or per-page SVGs for the in-dialog preview).

use crate::composite::{
    self, Anchor, CompositePage, CompositeResult, CompositeSpec, DrawOp,
};
use crate::db;
use crate::workflow::{run_pay_summary, PaySummaryRequest};
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
    /// Study title on the cover, e.g. "Petrophysical Evaluation — Balam South".
    pub title: String,
    #[serde(default)]
    pub author: String,
    /// Methodology table rows; empty = built-in default template.
    #[serde(default)]
    pub methodology: Vec<MethodRow>,
    /// Pay-summary cutoffs (pay-summary convention).
    pub vsh_max: f64,
    pub phie_min: f64,
    pub swe_max: f64,
    #[serde(default)]
    pub perm_min: Option<f64>,
    /// Skip the composite pages (tables-only report).
    #[serde(default)]
    pub tables_only: bool,
}

fn default_methodology(spec: &ReportSpec) -> Vec<MethodRow> {
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
                "VSH ≤ {:.2}, PHIE ≥ {:.2}, SWE ≤ {:.2}{}",
                spec.vsh_max,
                spec.phie_min,
                spec.swe_max,
                spec.perm_min.map(|p| format!(", PERM ≥ {p:.1} mD")).unwrap_or_default()
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

/// A single page carrying just a section header and a wrapped note line — used when a section has
/// no table to show (a compute/storage error, or a legitimately empty result) so a deliverable
/// never silently drops the section: the header still appears, with an explicit reason underneath.
fn note_page(section: &str, well: &str, note: &str, pw: f64) -> Vec<DrawOp> {
    let mut ops: Vec<DrawOp> = Vec::new();
    let y = page_header(&mut ops, pw, section, well);
    let size = 3.0;
    let max_chars = ((pw - 2.0 * MARGIN) / (CHAR_W * size)) as usize;
    for (i, line) in wrap(note, max_chars).into_iter().enumerate() {
        text(&mut ops, MARGIN, y + size + i as f64 * size * LINE_H_FACTOR, size, Anchor::Start, false, "#222222", line);
    }
    ops
}

/// Paginated table: fixed column widths (mm), wrapped cells, header row repeated on
/// every page. Returns one `Vec<DrawOp>` per page.
fn table_pages(
    section: &str,
    well: &str,
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

    let mut pages: Vec<Vec<DrawOp>> = Vec::new();
    let mut ops: Vec<DrawOp> = Vec::new();
    let mut y = page_header(&mut ops, pw, section, well);
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
            y = page_header(&mut ops, pw, section, well);
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
    pages
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

fn cover_page(
    spec: &ReportSpec,
    header: &composite::WellHeader,
    interval: (f32, f32),
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
    let mut meta = format!("Interval: {:.1} – {:.1} m", interval.0, interval.1);
    if let Some(td) = header.td {
        meta.push_str(&format!("   ·   TD: {td:.1} m"));
    }
    if let Some(kb) = header.kb {
        meta.push_str(&format!("   ·   KB: {kb:.1} m"));
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
fn report_pages(
    db: &Mutex<Connection>,
    spec: &ReportSpec,
) -> Result<(Vec<Vec<DrawOp>>, f64, f64, String), String> {
    let (composite_pages, pw, ph, well_name, header, zones, zparams) = {
        let conn = db.lock().unwrap();
        let header = composite::fetch_header(&conn, &spec.composite.well_id)?;
        let zones = db::list_zones(&conn, &spec.composite.well_id).map_err(|e| e.to_string())?;
        let zparams = db::list_zone_params(&conn, &spec.composite.well_id).map_err(|e| e.to_string())?;
        // Composite pages (also gives the true interval for the cover). The lock is
        // released at the end of this block — run_pay_summary takes it itself.
        let (cpages, pw, ph, name) = composite::render_pages(&conn, &spec.composite)?;
        (cpages, pw, ph, name, header, zones, zparams)
    };

    {
        let interval = (
            composite_pages.first().map(|p| p.top).unwrap_or(0.0),
            composite_pages.last().map(|p| p.bot).unwrap_or(0.0),
        );

        let mut pages: Vec<Vec<DrawOp>> = Vec::new();

        // 1 — cover
        pages.push(cover_page(spec, &header, interval, pw, ph));

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
        pages.extend(table_pages("Methodology", &well_name, &m_cols, &m_rows, pw, ph, 2.7));

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
            let z_cols: [(&str, f64, Anchor); 5] = [
                ("Zone", usable * 0.26, Anchor::Start),
                ("Top (m)", usable * 0.13, Anchor::End),
                ("Bottom (m)", usable * 0.13, Anchor::End),
                ("Parameter", usable * 0.28, Anchor::Start),
                ("Value", usable * 0.20, Anchor::End),
            ];
            pages.extend(table_pages("Zone Parameters", &well_name, &z_cols, &z_rows, pw, ph, 2.7));
        }

        // 4 — pay summary (locks internally). Emit the section header unconditionally: a storage
        // error or an uninterpreted well must leave a visible note in the deliverable rather than a
        // silently missing section — this is a client PDF, and dropping a table it cannot support
        // (unwrap_or_default previously collapsed both Err and empty into "no section at all") is
        // exactly the cardinal-rule failure the report path must not allow.
        let pay_section = format!(
            "Pay Summary  (VSH ≤ {:.2}, PHIE ≥ {:.2}, SWE ≤ {:.2})",
            spec.vsh_max, spec.phie_min, spec.swe_max
        );
        match run_pay_summary(
            db,
            &PaySummaryRequest {
                well_ids: vec![spec.composite.well_id.clone()],
                vsh_max: spec.vsh_max,
                phie_min: spec.phie_min,
                swe_max: spec.swe_max,
                perm_min: spec.perm_min,
                skip_version: true, // report render side-effect — don't version the pay flags
                stats_only: false,  // report persists FLAG_* in place (unchanged behavior)
            },
        ) {
            Ok(pay_rows) if !pay_rows.is_empty() => {
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
                let p_cols: [(&str, f64, Anchor); 11] = [
                    ("Zone", usable * 0.16, Anchor::Start),
                    ("Flag", usable * 0.11, Anchor::Start),
                    ("Top", usable * 0.08, Anchor::End),
                    ("Bottom", usable * 0.08, Anchor::End),
                    ("Gross", usable * 0.08, Anchor::End),
                    ("Net", usable * 0.08, Anchor::End),
                    ("NTG", usable * 0.07, Anchor::End),
                    ("VSH", usable * 0.07, Anchor::End),
                    ("PHIE", usable * 0.08, Anchor::End),
                    ("SWE", usable * 0.07, Anchor::End),
                    ("HPV (m)", usable * 0.12, Anchor::End),
                ];
                pages.extend(table_pages(&pay_section, &well_name, &p_cols, &p_rows, pw, ph, 2.4));
            }
            // Ran cleanly but produced no rows: the well has no curve frame, or its zones could not
            // be read. Show the header + an explicit note rather than a blank gap.
            Ok(_) => pages.push(note_page(
                &pay_section,
                &well_name,
                "No pay summary — this well has no curve data to classify.",
                pw,
            )),
            // The pay numbers were computed in memory but a storage-side error (read-only DB, disk
            // full, appender failure) failed the FLAG_* write. Surface it in the document instead of
            // dropping the section, and keep the rest of the report (composite pages) intact.
            Err(e) => pages.push(note_page(
                &pay_section,
                &well_name,
                &format!("Pay Summary unavailable — {e}"),
                pw,
            )),
        }

        // 5 — composite log pages
        if !spec.tables_only {
            pages.extend(composite_pages.into_iter().map(|p| p.ops));
        }

        Ok((pages, pw, ph, well_name))
    }
}

/// SVG preview of the whole report (one SVG per page, same shape as the composite result).
pub fn render_report(db: &Mutex<Connection>, spec: &ReportSpec) -> Result<CompositeResult, String> {
    let (pages, pw, ph, well_name) = report_pages(db, spec)?;
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
    })
}

/// The full report as one multi-page PDF (bytes).
pub fn render_report_pdf(db: &Mutex<Connection>, spec: &ReportSpec) -> Result<Vec<u8>, String> {
    let (pages, pw, ph, _) = report_pages(db, spec)?;
    let streams: Vec<String> = pages.iter().map(|ops| composite::pdf_content(ops, pw, ph)).collect();
    Ok(composite::assemble_pdf(&streams, pw, ph))
}

/// Batch export: one report PDF per well into `dest_dir`, named `<WELL>_report.pdf`.
/// Wells that fail (e.g. no curve data) are skipped with their error collected.
pub fn export_report_batch(
    db: &Mutex<Connection>,
    spec: &ReportSpec,
    well_ids: &[String],
    dest_dir: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut written = Vec::new();
    let mut errors = Vec::new();
    for wid in well_ids {
        let mut s = spec.clone();
        s.composite.well_id = wid.clone();
        match render_report_pdf(db, &s) {
            Ok(bytes) => {
                let name = {
                    let conn = db.lock().unwrap();
                    conn.query_row(
                        "SELECT well_name FROM wells WHERE well_id = ?1",
                        duckdb::params![wid],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| wid.clone())
                };
                let safe: String =
                    name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
                let path = format!("{}/{}_report.pdf", dest_dir.trim_end_matches(['/', '\\']), safe);
                match std::fs::write(&path, &bytes) {
                    Ok(()) => written.push(path),
                    Err(e) => errors.push(format!("{name}: {e}")),
                }
            }
            Err(e) => errors.push(format!("{wid}: {e}")),
        }
    }
    Ok((written, errors))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let pages = table_pages("Test", "WELL-1", &cols, &rows, 210.0, 297.0, 2.7);
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
            "Pay Summary  (VSH ≤ 0.50, PHIE ≥ 0.10, SWE ≤ 0.60)",
            "BLSO-001",
            "Pay Summary unavailable — read-only database",
            210.0,
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
        assert!(texts.iter().any(|s| s.contains("BLSO-001")), "well name must be present");
        assert!(joined.contains("unavailable"), "the failure note must render, not be dropped");
    }

    #[test]
    fn cover_page_carries_title_and_well() {
        let spec = ReportSpec {
            composite: serde_json::from_str(
                r#"{"well_id":"w1","layout":{"name":"t","tracks":[]},"scale":200,"page_size":"a4"}"#,
            )
            .unwrap(),
            title: "Petrophysical Evaluation — Balam South".into(),
            author: "Jauhar".into(),
            methodology: vec![],
            vsh_max: 0.5,
            phie_min: 0.1,
            swe_max: 0.6,
            perm_min: None,
            tables_only: true,
        };
        let header = composite::WellHeader {
            name: "BLSO-001".into(),
            field: Some("Balam".into()),
            td: Some(900.0),
            kb: Some(15.0),
        };
        let ops = cover_page(&spec, &header, (400.0, 850.0), 210.0, 297.0);
        let texts: Vec<String> = ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { s, .. } => Some(s.clone()),
                _ => None,
            })
            .collect();
        // The title may wrap across lines — check the joined text.
        let joined = texts.join(" ");
        assert!(joined.contains("Balam") && joined.contains("South"));
        assert!(texts.iter().any(|s| s.contains("BLSO-001")));
        assert!(texts.iter().any(|s| s.contains("Jauhar")));
        assert!(texts.iter().any(|s| s.contains("400.0")));
    }
}
