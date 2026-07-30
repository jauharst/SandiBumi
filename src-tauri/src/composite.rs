//! Composite log plot (composite-plot equivalent). Renders a `Layout`
//! at a TRUE print scale (1:200 / 1:500 / 1:1000) as one page per depth window — a header
//! block, a depth axis, curve tracks (linear/log with grids, curve polylines and edge
//! fills), formation-top lines, and zone bands.
//!
//! Each page is first built as a backend-neutral list of `DrawOp`s in millimetre space
//! (top-left origin), then serialized to either SVG (screen preview / vector export) or a
//! dependency-free multi-page PDF (base-14 Helvetica, so no font files are embedded). The
//! SVG declares its size in mm with a matching viewBox, so 1 user unit == 1 mm and the
//! print scale is physically exact — at 1:200, one metre of formation is 1000/200 = 5 mm.

use crate::equations;
use crate::layout::{Layout, ScaleType};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;

// Page geometry (mm).
const MARGIN_L: f64 = 12.0;
const MARGIN_R: f64 = 10.0;
const MARGIN_T: f64 = 10.0;
const MARGIN_B: f64 = 10.0;
const HEADER_H_FIRST: f64 = 32.0; // full metadata block on page 1
const HEADER_H_RUN: f64 = 8.0; // running header on later pages
const TRACK_HEADER_H: f64 = 12.0; // per-page track title + scale strip
const DEPTH_TRACK_W: f64 = 14.0; // depth-label column width

const PT_PER_MM: f64 = 72.0 / 25.4; // PDF user-space unit is 1 pt = 1/72 in

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageSize {
    A4,
    A3,
    Letter,
}

impl PageSize {
    /// Portrait (width, height) in mm.
    fn dims(self) -> (f64, f64) {
        match self {
            PageSize::A4 => (210.0, 297.0),
            PageSize::A3 => (297.0, 420.0),
            PageSize::Letter => (215.9, 279.4),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompositeSpec {
    pub well_id: String,
    pub layout: Layout,
    /// Depth window; when omitted, the well's full logged interval is used.
    #[serde(default)]
    pub depth_top: Option<f32>,
    #[serde(default)]
    pub depth_bottom: Option<f32>,
    /// Print scale denominator: 200, 500, or 1000 (others allowed; these are the presets).
    pub scale: u32,
    pub page_size: PageSize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompositePage {
    pub svg: String,
    pub top_depth: f32,
    pub bottom_depth: f32,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompositeResult {
    pub pages: Vec<CompositePage>,
    pub page_width_mm: f64,
    pub page_height_mm: f64,
    pub scale: u32,
    pub well_name: String,
}

pub(crate) struct WellHeader {
    pub(crate) name: String,
    pub(crate) field: Option<String>,
    pub(crate) td: Option<f32>,
    pub(crate) kb: Option<f32>,
}

// ---------------------------------------------------------------------------
// Backend-neutral drawing primitives (millimetre space, top-left origin).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Anchor {
    Start,
    Middle,
    End,
}

pub(crate) enum DrawOp {
    Rect { x: f64, y: f64, w: f64, h: f64, fill: Option<String>, stroke: Option<String>, sw: f64 },
    Line { x1: f64, y1: f64, x2: f64, y2: f64, stroke: String, sw: f64 },
    /// Open polyline (a curve run).
    Poly { pts: Vec<(f64, f64)>, stroke: String, sw: f64 },
    /// Closed filled polygon (a curve edge fill / shading).
    Fill { pts: Vec<(f64, f64)>, fill: String, opacity: f64 },
    Text { x: f64, y: f64, size: f64, anchor: Anchor, color: String, bold: bool, s: String },
}

pub(crate) struct PageOps {
    pub(crate) ops: Vec<DrawOp>,
    pub(crate) top: f32,
    pub(crate) bot: f32,
    pub(crate) idx: usize,
}

pub(crate) fn fetch_header(conn: &Connection, well_id: &str) -> Result<WellHeader, String> {
    conn.query_row(
        "SELECT well_name, field_name, td, kb FROM wells WHERE well_id = ?1",
        params![well_id],
        |row| {
            Ok(WellHeader {
                name: row.get(0)?,
                field: row.get(1)?,
                td: row.get::<_, Option<f32>>(2)?,
                kb: row.get::<_, Option<f32>>(3)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

/// XML-escapes text for safe inclusion in SVG element bodies/attributes.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A "nice" round step near `target` (1/2/5 × 10^n) — for depth-grid major spacing.
fn nice_step(target: f64) -> f64 {
    if target <= 0.0 {
        return 1.0;
    }
    let exp = target.log10().floor();
    let base = 10f64.powf(exp);
    let f = target / base;
    let nice = if f < 1.5 { 1.0 } else if f < 3.5 { 2.0 } else if f < 7.5 { 5.0 } else { 10.0 };
    nice * base
}

/// Maps a curve value to an x fraction [0,1] across a track for the given scale, honoring
/// reversed min/max (min > max ⇒ the axis runs the other way, e.g. porosity). Returns
/// None for NaN or non-positive values on a log scale.
fn value_frac(v: f32, min: f32, max: f32, scale: ScaleType) -> Option<f64> {
    if v.is_nan() {
        return None;
    }
    let (v, min, max) = (v as f64, min as f64, max as f64);
    match scale {
        ScaleType::Linear => {
            if (max - min).abs() < 1e-12 {
                return None;
            }
            Some((v - min) / (max - min))
        }
        ScaleType::Log => {
            if v <= 0.0 || min <= 0.0 || max <= 0.0 {
                return None;
            }
            let (lv, lmin, lmax) = (v.ln(), min.ln(), max.ln());
            if (lmax - lmin).abs() < 1e-12 {
                return None;
            }
            Some((lv - lmin) / (lmax - lmin))
        }
    }
}

/// Builds every page's draw-op list once (shared by the SVG and PDF serializers).
pub(crate) fn render_pages(conn: &Connection, spec: &CompositeSpec) -> Result<(Vec<PageOps>, f64, f64, String), String> {
    let header = fetch_header(conn, &spec.well_id)?;

    let curve_names: Vec<String> = spec
        .layout
        .tracks
        .iter()
        .flat_map(|t| t.curves.iter().map(|c| c.curve_name.clone()))
        .collect();
    let (depth, columns) =
        equations::fetch_curve_frame(conn, &spec.well_id, &curve_names).map_err(|e| e.to_string())?;
    if depth.is_empty() {
        return Err("no curve data for this well".into());
    }

    let data_top = *depth.first().unwrap();
    let data_bot = *depth.last().unwrap();
    let top = spec.depth_top.unwrap_or(data_top).max(data_top);
    let bottom = spec.depth_bottom.unwrap_or(data_bot).min(data_bot);
    if !(bottom > top) {
        return Err("empty depth range".into());
    }

    let tops = crate::db::list_tops(conn, &spec.well_id).map_err(|e| e.to_string())?;
    let zones = crate::db::list_zones(conn, &spec.well_id).map_err(|e| e.to_string())?;
    // Well-diagram tracks (if any) draw these; absent datasets simply yield an empty diagram.
    let has_diagram = spec.layout.tracks.iter().any(|t| t.kind == crate::layout::TrackKind::WellDiagram);
    let completion = if has_diagram {
        crate::db::list_aux_data(conn, &spec.well_id, Some("COMPLETION")).unwrap_or_default()
    } else {
        Vec::new()
    };
    let perforations = if has_diagram {
        crate::db::list_aux_data(conn, &spec.well_id, Some("PERFORATION")).unwrap_or_default()
    } else {
        Vec::new()
    };
    // Point-data tracks (if any) draw measured samples. Fetched once for the whole render;
    // both readers are active-set filtered, so this is one delivery of each.
    let has_points = spec.layout.tracks.iter().any(|t| t.kind == crate::layout::TrackKind::PointData);
    let core: Vec<(String, f32, f32)> = if has_points {
        crate::db::get_core_point_series(conn, &spec.well_id).unwrap_or_default()
    } else {
        Vec::new()
    };
    let aux = if has_points {
        crate::db::list_aux_data(conn, &spec.well_id, None).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Array logs (a distribution at every depth) for any array track, fetched once for the whole
    // render and keyed by upper-cased curve name. Read whole rather than page by page: a page
    // boundary must not change which realizations a band was computed from.
    let mut arrays: HashMap<String, Vec<crate::db::ArrayRow>> = HashMap::new();
    for track in spec.layout.tracks.iter().filter(|t| t.kind == crate::layout::TrackKind::ArrayLog) {
        for a in &track.arrays {
            let key = a.curve_name.trim().to_uppercase();
            if key.is_empty() || arrays.contains_key(&key) {
                continue;
            }
            let rows = crate::db::read_array_log(conn, &spec.well_id, a.set_name.as_deref(), &key)
                .unwrap_or_default();
            arrays.insert(key, rows);
        }
    }

    let (pw, ph) = spec.page_size.dims();
    let mm_per_m = 1000.0 / spec.scale as f64;

    let track_h_first = ph - MARGIN_T - MARGIN_B - HEADER_H_FIRST - TRACK_HEADER_H;
    let track_h_run = ph - MARGIN_T - MARGIN_B - HEADER_H_RUN - TRACK_HEADER_H;
    let m_per_page_first = track_h_first / mm_per_m;
    let m_per_page_run = track_h_run / mm_per_m;

    let mut pages = Vec::new();
    let mut d0 = top as f64;
    let bottom = bottom as f64;
    let mut idx = 0;
    while d0 < bottom - 1e-6 {
        let first = idx == 0;
        let m_this = if first { m_per_page_first } else { m_per_page_run };
        let d1 = (d0 + m_this).min(bottom);
        let ops = build_page(
            spec, &header, &depth, &columns, &tops, &zones, &completion, &perforations, &core, &aux,
            &arrays, pw, ph, mm_per_m, first, d0 as f32, d1 as f32, idx,
        );
        pages.push(PageOps { ops, top: d0 as f32, bot: d1 as f32, idx });
        d0 = d1;
        idx += 1;
    }

    Ok((pages, pw, ph, header.name))
}

/// Renders the composite to one vector SVG per page.
pub fn render_composite(conn: &Connection, spec: &CompositeSpec) -> Result<CompositeResult, String> {
    let (pages, pw, ph, well_name) = render_pages(conn, spec)?;
    let out = pages
        .into_iter()
        .map(|p| CompositePage {
            svg: svg_page(&p.ops, pw, ph),
            top_depth: p.top,
            bottom_depth: p.bot,
            index: p.idx,
        })
        .collect();
    Ok(CompositeResult { pages: out, page_width_mm: pw, page_height_mm: ph, scale: spec.scale, well_name })
}

/// Renders the composite to a single multi-page PDF (bytes).
pub fn render_composite_pdf(conn: &Connection, spec: &CompositeSpec) -> Result<Vec<u8>, String> {
    let (pages, pw, ph, _) = render_pages(conn, spec)?;
    let streams: Vec<String> = pages.iter().map(|p| pdf_content(&p.ops, pw, ph)).collect();
    Ok(assemble_pdf(&streams, pw, ph))
}

/// Writes the rendered pages to disk as SVG. A single page goes to `dest_path` as given;
/// multiple pages get a zero-padded `_pNN` suffix inserted before the extension. Returns
/// the paths actually written.
pub fn export_svg_files(result: &CompositeResult, dest_path: &str) -> Result<Vec<String>, String> {
    use std::path::Path;
    let mut written = Vec::new();
    if result.pages.len() == 1 {
        std::fs::write(dest_path, &result.pages[0].svg).map_err(|e| e.to_string())?;
        written.push(dest_path.to_string());
        return Ok(written);
    }
    let p = Path::new(dest_path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("composite");
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("svg");
    let dir = p.parent();
    for page in &result.pages {
        let name = format!("{stem}_p{:02}.{ext}", page.index + 1);
        let full = match dir {
            Some(d) => d.join(&name),
            None => Path::new(&name).to_path_buf(),
        };
        std::fs::write(&full, &page.svg).map_err(|e| e.to_string())?;
        written.push(full.to_string_lossy().into_owned());
    }
    Ok(written)
}

// ---------------------------------------------------------------------------
// Page builder (produces DrawOps) — backend-independent geometry.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_page(
    spec: &CompositeSpec,
    header: &WellHeader,
    depth: &[f32],
    columns: &HashMap<String, Vec<f32>>,
    tops: &[crate::db::TopEntry],
    zones: &[crate::db::ZoneEntry],
    completion: &[crate::db::AuxRow],
    perforations: &[crate::db::AuxRow],
    core: &[(String, f32, f32)],
    aux: &[crate::db::AuxRow],
    arrays: &HashMap<String, Vec<crate::db::ArrayRow>>,
    pw: f64,
    ph: f64,
    mm_per_m: f64,
    first: bool,
    page_top: f32,
    page_bot: f32,
    idx: usize,
) -> Vec<DrawOp> {
    let mut ops: Vec<DrawOp> = Vec::new();

    let header_h = if first { HEADER_H_FIRST } else { HEADER_H_RUN };
    let header_bottom = MARGIN_T + header_h;
    draw_header(&mut ops, header, spec, pw, first, page_top, page_bot, idx);

    let area_x0 = MARGIN_L;
    let area_x1 = pw - MARGIN_R;
    let tracks_x0 = area_x0 + DEPTH_TRACK_W;
    let track_top = header_bottom;
    let grid_top = track_top + TRACK_HEADER_H;
    let grid_bot = ph - MARGIN_B;

    let y_of = |d: f32| grid_top + (d - page_top) as f64 * mm_per_m;

    // Track x spans from width weights.
    let total_w: f32 = spec.layout.tracks.iter().map(|t| t.width_weight.max(0.05)).sum();
    let avail = area_x1 - tracks_x0;
    let mut xs: Vec<(f64, f64)> = Vec::new();
    let mut cx = tracks_x0;
    for t in &spec.layout.tracks {
        let w = avail * (t.width_weight.max(0.05) / total_w) as f64;
        xs.push((cx, cx + w));
        cx += w;
    }

    // Zone bands (behind everything).
    for (zi, z) in zones.iter().enumerate() {
        let zt = z.top_depth.max(page_top);
        let zb = z.bottom_depth.min(page_bot);
        if zb <= zt {
            continue;
        }
        let (y0, y1) = (y_of(zt), y_of(zb));
        let shade = if zi % 2 == 0 { "#f4f1ea" } else { "#eef2f4" };
        ops.push(DrawOp::Rect {
            x: tracks_x0,
            y: y0,
            w: area_x1 - tracks_x0,
            h: y1 - y0,
            fill: Some(shade.into()),
            stroke: None,
            sw: 0.0,
        });
    }

    // Depth-track frame.
    ops.push(DrawOp::Rect {
        x: area_x0,
        y: grid_top,
        w: DEPTH_TRACK_W,
        h: grid_bot - grid_top,
        fill: None,
        stroke: Some("#333333".into()),
        sw: 0.2,
    });

    // Depth grid + labels.
    let major = nice_step(22.0 / mm_per_m);
    let minor = major / 5.0;
    let first_minor = (page_top as f64 / minor).ceil() * minor;
    let mut d = first_minor;
    while d <= page_bot as f64 + 1e-6 {
        let y = y_of(d as f32);
        let is_major = ((d / major).round() * major - d).abs() < 1e-6;
        let (stroke, w) = if is_major { ("#c9c9c9", 0.25) } else { ("#ececec", 0.15) };
        ops.push(DrawOp::Line { x1: tracks_x0, y1: y, x2: area_x1, y2: y, stroke: stroke.into(), sw: w });
        if is_major {
            ops.push(DrawOp::Line {
                x1: area_x0 + DEPTH_TRACK_W - 1.5,
                y1: y,
                x2: area_x0,
                y2: y,
                stroke: "#333333".into(),
                sw: 0.25,
            });
            ops.push(DrawOp::Text {
                x: area_x0 + DEPTH_TRACK_W / 2.0,
                y: y - 0.6,
                size: 2.6,
                anchor: Anchor::Middle,
                color: "#222222".into(),
                bold: false,
                s: (d.round() as i64).to_string(),
            });
        }
        d += minor;
    }

    // Tracks: frame, vgrid, curves, header.
    for (ti, track) in spec.layout.tracks.iter().enumerate() {
        let (tx0, tx1) = xs[ti];
        ops.push(DrawOp::Rect {
            x: tx0,
            y: grid_top,
            w: tx1 - tx0,
            h: grid_bot - grid_top,
            fill: None,
            stroke: Some("#333333".into()),
            sw: 0.2,
        });
        if track.kind == crate::layout::TrackKind::ArrayLog {
            for a in &track.arrays {
                let rows = arrays.get(&a.curve_name.trim().to_uppercase()).map(Vec::as_slice).unwrap_or(&[]);
                draw_array_series(
                    &mut ops, a, track.scale_type, rows, tx0, tx1, page_top, page_bot, &y_of,
                );
            }
        } else if track.kind == crate::layout::TrackKind::PointData {
            for ps in &track.points {
                draw_point_series(
                    &mut ops, ps, track.scale_type, core, aux, tx0, tx1, page_top, page_bot, &y_of,
                );
            }
        } else if track.kind == crate::layout::TrackKind::WellDiagram {
            draw_well_diagram(&mut ops, completion, perforations, tx0, tx1, page_top, page_bot, &y_of);
        } else {
            draw_vgrid(&mut ops, track, tx0, tx1, grid_top, grid_bot);
            for cs in &track.curves {
                let Some(vals) = columns.get(&cs.curve_name.trim().to_uppercase()) else { continue };
                // Crossover shading needs the reference curve's samples AND its own min/max,
                // taken from the same track — compatible scaling is the whole point.
                let xover = cs.fill_to.as_deref().and_then(|to| {
                    let key = to.trim().to_uppercase();
                    let rs = track.curves.iter().find(|o| o.curve_name.trim().to_uppercase() == key)?;
                    Some((columns.get(&key)?.as_slice(), rs.min, rs.max))
                });
                draw_curve(
                    &mut ops, cs, track.scale_type, vals, depth, xover, tx0, tx1, page_top, page_bot,
                    &y_of,
                );
            }
        }
        draw_track_header(&mut ops, track, tx0, tx1, track_top, grid_top);
    }

    // Formation tops.
    for t in tops {
        if t.depth < page_top || t.depth > page_bot {
            continue;
        }
        let y = y_of(t.depth);
        let color = t.color.as_deref().unwrap_or("#b0413e").to_string();
        ops.push(DrawOp::Line { x1: tracks_x0, y1: y, x2: area_x1, y2: y, stroke: color.clone(), sw: 0.5 });
        ops.push(DrawOp::Text {
            x: tracks_x0 + 1.0,
            y: y - 0.8,
            size: 2.6,
            anchor: Anchor::Start,
            color,
            bold: false,
            s: t.top_name.clone(),
        });
    }

    ops
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    ops: &mut Vec<DrawOp>,
    header: &WellHeader,
    spec: &CompositeSpec,
    pw: f64,
    first: bool,
    page_top: f32,
    page_bot: f32,
    idx: usize,
) {
    let x0 = MARGIN_L;
    let y0 = MARGIN_T;
    let w = pw - MARGIN_L - MARGIN_R;
    if first {
        ops.push(DrawOp::Rect {
            x: x0,
            y: y0,
            w,
            h: HEADER_H_FIRST,
            fill: None,
            stroke: Some("#333333".into()),
            sw: 0.3,
        });
        let text = |ops: &mut Vec<DrawOp>, dy: f64, size: f64, bold: bool, color: &str, s: String| {
            ops.push(DrawOp::Text { x: x0 + 3.0, y: y0 + dy, size, anchor: Anchor::Start, color: color.into(), bold, s });
        };
        text(ops, 7.0, 5.5, true, "#111111", header.name.clone());
        let field = header.field.clone().unwrap_or_else(|| "—".into());
        let td = header.td.map(|v| format!("{v:.1} m")).unwrap_or_else(|| "—".into());
        let kb = header.kb.map(|v| format!("{v:.1} m")).unwrap_or_else(|| "—".into());
        text(ops, 13.5, 3.0, false, "#333333", format!("Field: {field}    TD: {td}    KB: {kb}"));
        text(
            ops,
            19.0,
            3.0,
            false,
            "#333333",
            format!("Layout: {}    Scale 1:{}    Interval {:.1}\u{2013}{:.1} m", spec.layout.name, spec.scale, page_top, page_bot),
        );
        text(ops, HEADER_H_FIRST - 2.5, 2.6, false, "#888888", "Made in SandiBumi \u{2014} composite log".into());
    } else {
        ops.push(DrawOp::Text {
            x: x0,
            y: y0 + 5.0,
            size: 3.2,
            anchor: Anchor::Start,
            color: "#111111".into(),
            bold: true,
            s: format!("{}  \u{2014}  1:{}  (p.{})", header.name, spec.scale, idx + 1),
        });
    }
}

/// Well-diagram track (Track kind = well_diagram): schematic casing/tubing strings with shoe
/// markers + perforation ticks, from COMPLETION (value_num = OD in inches) and PERFORATION aux.
#[allow(clippy::too_many_arguments)]
fn draw_well_diagram(
    ops: &mut Vec<DrawOp>,
    casing: &[crate::db::AuxRow],
    perfs: &[crate::db::AuxRow],
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &impl Fn(f32) -> f64,
) {
    let cx = (tx0 + tx1) / 2.0;
    let span = tx1 - tx0;
    let max_half = span * 0.36;
    let max_od = casing.iter().filter_map(|c| c.value_num).fold(1.0_f32, f32::max).max(1.0);

    for c in casing {
        let top_d = c.depth_top;
        let base_d = c.depth_base.unwrap_or(page_bot);
        if base_d < page_top || top_d > page_bot {
            continue;
        }
        let od = c.value_num.unwrap_or(max_od);
        let half = ((od / max_od) as f64 * max_half).max(0.8);
        let yt = y_of(top_d.clamp(page_top, page_bot));
        let yb = y_of(base_d.clamp(page_top, page_bot));
        for sx in [cx - half, cx + half] {
            ops.push(DrawOp::Line { x1: sx, y1: yt, x2: sx, y2: yb, stroke: "#5a5a5a".into(), sw: 0.4 });
        }
        // Shoe markers (small filled squares) at the casing base.
        if c.depth_base.is_some() && base_d >= page_top && base_d <= page_bot {
            for sx in [cx - half, cx + half] {
                ops.push(DrawOp::Rect {
                    x: sx - 0.6,
                    y: yb - 1.2,
                    w: 1.2,
                    h: 1.2,
                    fill: Some("#333333".into()),
                    stroke: None,
                    sw: 0.0,
                });
            }
        }
        // OD label at the top of the string (only when its top is on this page).
        if top_d >= page_top && top_d <= page_bot {
            let label = c
                .value_text
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| c.value_num.map(|od| format!("{od}\"")).unwrap_or_else(|| c.item.clone()));
            ops.push(DrawOp::Text {
                x: cx,
                y: yt - 0.8,
                size: 2.2,
                anchor: Anchor::Middle,
                color: "#333333".into(),
                bold: false,
                s: label,
            });
        }
    }

    // Perforations: red ticks radiating from the well centre over each perf interval.
    let tick_half = (span * 0.28).min(max_half);
    for p in perfs {
        let d_top = p.depth_top;
        let d_bot = p.depth_base.unwrap_or(p.depth_top);
        if d_top > page_bot || d_bot < page_top {
            continue;
        }
        let ylo = y_of(d_top.min(d_bot).clamp(page_top, page_bot));
        let yhi = y_of(d_top.max(d_bot).clamp(page_top, page_bot));
        let mut yy = ylo;
        loop {
            ops.push(DrawOp::Line {
                x1: cx - tick_half,
                y1: yy,
                x2: cx - tick_half * 0.4,
                y2: yy,
                stroke: "#c0392b".into(),
                sw: 0.4,
            });
            ops.push(DrawOp::Line {
                x1: cx + tick_half * 0.4,
                y1: yy,
                x2: cx + tick_half,
                y2: yy,
                stroke: "#c0392b".into(),
                sw: 0.4,
            });
            yy += 1.5;
            if yy > yhi {
                break;
            }
        }
    }
}

fn draw_vgrid(ops: &mut Vec<DrawOp>, track: &crate::layout::Track, tx0: f64, tx1: f64, top: f64, bot: f64) {
    match track.scale_type {
        ScaleType::Linear => {
            for k in 1..4 {
                let x = tx0 + (tx1 - tx0) * k as f64 / 4.0;
                ops.push(DrawOp::Line { x1: x, y1: top, x2: x, y2: bot, stroke: "#ececec".into(), sw: 0.15 });
            }
        }
        ScaleType::Log => {
            if let Some(cs) = track.curves.first() {
                let (mut lo, mut hi) = (cs.min.min(cs.max), cs.min.max(cs.max));
                if lo <= 0.0 {
                    lo = 0.01;
                }
                if hi <= lo {
                    hi = lo * 10.0;
                }
                let (llo, lhi) = ((lo as f64).ln(), (hi as f64).ln());
                let mut dec = lo.log10().floor();
                loop {
                    let val = 10f64.powf(dec as f64);
                    if val > hi as f64 * 1.0001 {
                        break;
                    }
                    if val >= lo as f64 * 0.9999 {
                        let frac = (val.ln() - llo) / (lhi - llo);
                        let x = tx0 + (tx1 - tx0) * frac;
                        ops.push(DrawOp::Line { x1: x, y1: top, x2: x, y2: bot, stroke: "#e2e2e2".into(), sw: 0.2 });
                    }
                    dec += 1.0;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_curve(
    ops: &mut Vec<DrawOp>,
    cs: &crate::layout::CurveStyle,
    scale: ScaleType,
    vals: &[f32],
    depth: &[f32],
    xover: Option<(&[f32], f32, f32)>,
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    let tw = tx1 - tx0;
    let x_at = |v: f32| value_frac(v, cs.min, cs.max, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    // "step" holds each sample's value down to the next sample's depth, so a run gains a
    // second point at the same x — the blocky display. Kept in sync with the viewer's
    // LogCanvasRenderer, which builds the identical corner.
    let step = cs.draw_style.as_deref() == Some("step");
    let hold_to = |i: usize| -> Option<f64> {
        let d = *depth.get(i + 1)?;
        Some(y_of(d.clamp(page_top, page_bot)))
    };

    match cs.fill.as_deref() {
        // Discrete class blocks: full-track-width colored rectangles per contiguous
        // same-class run — the print equivalent of the viewer's facies track.
        Some("blocks") => {
            draw_class_blocks(ops, cs, vals, depth, tx0, tx1, page_top, page_bot, y_of);
            return;
        }
        // Crossover shading against another curve in the same track.
        Some("curve") => {
            if let Some(reference) = xover {
                draw_crossover(
                    ops, cs, scale, vals, depth, reference, tx0, tx1, page_top, page_bot, y_of,
                );
            }
        }
        // Edge fill: closed polygon between the curve run and the chosen track edge. Matched
        // explicitly so a style saved with fill = "none" (what the properties dialog writes
        // when you pick None) prints unshaded, exactly as the viewer draws it.
        Some(side @ ("left" | "right")) => {
            let edge_x = if side == "right" { tx1 } else { tx0 };
            let fill_color = cs.fill_color.clone().unwrap_or_else(|| cs.color.clone());
            let opacity = cs.fill_opacity.unwrap_or(0.25) as f64;
            let mut run: Vec<(f64, f64)> = Vec::new();
            let flush = |ops: &mut Vec<DrawOp>, run: &mut Vec<(f64, f64)>| {
                if run.len() >= 2 {
                    let mut pts = Vec::with_capacity(run.len() + 2);
                    pts.push((edge_x, run[0].1));
                    pts.extend_from_slice(run);
                    pts.push((edge_x, run.last().unwrap().1));
                    ops.push(DrawOp::Fill { pts, fill: fill_color.clone(), opacity });
                }
                run.clear();
            };
            for (i, &v) in vals.iter().enumerate() {
                let d = depth[i];
                if d < page_top || d > page_bot {
                    flush(ops, &mut run);
                    continue;
                }
                match x_at(v) {
                    Some(x) => {
                        run.push((x, y_of(d)));
                        if step {
                            if let Some(y) = hold_to(i) {
                                run.push((x, y));
                            }
                        }
                    }
                    None => flush(ops, &mut run),
                }
            }
            flush(ops, &mut run);
        }
        _ => {}
    }

    // Curve line: contiguous runs, breaking at NaN / off-page gaps.
    let mut run: Vec<(f64, f64)> = Vec::new();
    let flush_line = |ops: &mut Vec<DrawOp>, run: &mut Vec<(f64, f64)>| {
        if run.len() >= 2 {
            ops.push(DrawOp::Poly { pts: run.clone(), stroke: cs.color.clone(), sw: 0.35 });
        }
        run.clear();
    };
    for (i, &v) in vals.iter().enumerate() {
        let d = depth[i];
        if d < page_top || d > page_bot {
            flush_line(ops, &mut run);
            continue;
        }
        match x_at(v) {
            Some(x) => {
                run.push((x, y_of(d)));
                if step {
                    if let Some(y) = hold_to(i) {
                        run.push((x, y));
                    }
                }
            }
            None => flush_line(ops, &mut run),
        }
    }
    flush_line(ops, &mut run);
}

/// `fill = "curve"`: shading between this curve and a reference curve in the same track —
/// the neutron-density separation display. Each sample interval contributes one quad
/// between the two curves; where the pair actually crosses INSIDE an interval the quad is
/// split at the crossing point, so the two colours meet on the crossover instead of one
/// bleeding a whole sample past it. A NaN on either curve simply leaves that interval
/// unshaded — a crossover is never inferred across a gap.
#[allow(clippy::too_many_arguments)]
fn draw_crossover(
    ops: &mut Vec<DrawOp>,
    cs: &crate::layout::CurveStyle,
    scale: ScaleType,
    vals: &[f32],
    depth: &[f32],
    reference: (&[f32], f32, f32),
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    let (rvals, rmin, rmax) = reference;
    let tw = tx1 - tx0;
    let x_a = |v: f32| value_frac(v, cs.min, cs.max, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    let x_b = |v: f32| value_frac(v, rmin, rmax, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    let left = cs.fill_color.clone().unwrap_or_else(|| cs.color.clone());
    let right = cs.fill_color2.clone().unwrap_or_else(|| left.clone());
    let opacity = cs.fill_opacity.unwrap_or(0.3) as f64;
    let step = cs.draw_style.as_deref() == Some("step");

    let n = vals.len().min(depth.len()).min(rvals.len());
    for i in 0..n.saturating_sub(1) {
        let (d0, d1) = (depth[i], depth[i + 1]);
        if d0 < page_top || d0 > page_bot || d1 < page_top || d1 > page_bot {
            continue;
        }
        let (Some(a0), Some(b0)) = (x_a(vals[i]), x_b(rvals[i])) else { continue };
        // A stepped curve holds its value across the interval, so both edges stay vertical
        // and the pair can never cross inside one interval.
        let (a1, b1) = if step {
            (a0, b0)
        } else {
            let (Some(a1), Some(b1)) = (x_a(vals[i + 1]), x_b(rvals[i + 1])) else { continue };
            (a1, b1)
        };
        let (y0, y1) = (y_of(d0), y_of(d1));
        let (s0, s1) = (a0 - b0, a1 - b1);
        let side = |s: f64| if s < 0.0 { left.clone() } else { right.clone() };
        if (s0 < 0.0) != (s1 < 0.0) && s0 != s1 {
            let t = s0 / (s0 - s1);
            let (ym, xm) = (y0 + (y1 - y0) * t, a0 + (a1 - a0) * t);
            ops.push(DrawOp::Fill {
                pts: vec![(a0, y0), (b0, y0), (xm, ym)],
                fill: side(s0),
                opacity,
            });
            ops.push(DrawOp::Fill {
                pts: vec![(xm, ym), (b1, y1), (a1, y1)],
                fill: side(s1),
                opacity,
            });
        } else {
            ops.push(DrawOp::Fill {
                pts: vec![(a0, y0), (b0, y0), (b1, y1), (a1, y1)],
                fill: side(s0),
                opacity,
            });
        }
    }
}

/// Gathers one point series' samples for the print path. Core reads the ACTIVE core set's
/// plug property; aux reads one item of one point dataset. An interval sample (depth_base
/// present) is anchored at its middle, where the measurement actually applies. Mirrors
/// `logViewPanel.pointSamples` — keep the two gathering the same rows.
fn point_samples(
    ps: &crate::layout::PointStyle,
    core: &[(String, f32, f32)],
    aux: &[crate::db::AuxRow],
) -> (Vec<f32>, Vec<f32>, Vec<String>) {
    let item = ps.item.trim().to_uppercase();
    let (mut d, mut v, mut t) = (Vec::new(), Vec::new(), Vec::new());
    if ps.source == "core" {
        for (name, depth, value) in core {
            if name.as_str() != item {
                continue;
            }
            d.push(*depth);
            v.push(*value);
            t.push(String::new());
        }
        return (d, v, t);
    }
    let dataset = ps.dataset.as_deref().map(|s| s.trim().to_uppercase());
    for r in aux {
        if dataset.as_deref().is_some_and(|ds| r.dataset.to_uppercase() != ds) {
            continue;
        }
        if r.item.to_uppercase() != item {
            continue;
        }
        d.push(r.depth_base.map_or(r.depth_top, |b| (r.depth_top + b) / 2.0));
        v.push(r.value_num.unwrap_or(f32::NAN));
        t.push(r.value_text.clone().unwrap_or_default());
    }
    (d, v, t)
}

/// A `point_data` track's series in print: measured samples rather than a continuous log.
/// Four displays matching the viewer — points, box plot per depth bin, value-axis histogram
/// per depth bin, and text labels. Statistics come from the shared `distribution` module, so
/// the printed box is the same box the screen drew.
#[allow(clippy::too_many_arguments)]
fn draw_point_series(
    ops: &mut Vec<DrawOp>,
    ps: &crate::layout::PointStyle,
    scale: ScaleType,
    core: &[(String, f32, f32)],
    aux: &[crate::db::AuxRow],
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    use crate::distribution::{bin_by_depth, box_stats, histogram};
    let (depth, value, text) = point_samples(ps, core, aux);
    if depth.is_empty() {
        return;
    }
    let tw = tx1 - tx0;
    // None (not a clamped edge) for anything off-scale — a clamped sample would print at a
    // value it never had.
    let x_at = |v: f32| {
        value_frac(v, ps.min, ps.max, scale).and_then(|f| (0.0..=1.0).contains(&f).then(|| tx0 + f * tw))
    };

    match ps.display.as_deref().unwrap_or("points") {
        "text" => {
            let mut last_y = f64::NEG_INFINITY;
            for i in 0..depth.len().min(text.len()) {
                let d = depth[i];
                if d < page_top || d > page_bot || text[i].is_empty() {
                    continue;
                }
                let y = y_of(d);
                // One label per 3 mm, or a densely described core prints as a black smear.
                if y - last_y < 3.0 {
                    continue;
                }
                last_y = y;
                ops.push(DrawOp::Text {
                    x: tx0 + 0.8,
                    y: y + 0.8,
                    size: 2.2,
                    anchor: Anchor::Start,
                    color: "#333333".into(),
                    bold: false,
                    s: text[i].chars().take(28).collect(),
                });
            }
        }
        "box" | "histogram" => {
            let bin = ps.bin.filter(|b| *b > 0.0).unwrap_or((page_bot - page_top) / 20.0);
            let is_hist = ps.display.as_deref() == Some("histogram");
            for (b_top, b_base, vals) in bin_by_depth(&depth, &value, bin) {
                if b_base < page_top || b_top > page_bot {
                    continue;
                }
                let (y0, y1) = (y_of(b_top.max(page_top)), y_of(b_base.min(page_bot)));
                let h = y1 - y0;
                if h <= 0.0 {
                    continue;
                }
                let mid = (y0 + y1) / 2.0;
                if is_hist {
                    let counts = histogram(&vals, ps.min, ps.max, ps.hist_bins.unwrap_or(12) as usize);
                    let peak = counts.iter().copied().max().unwrap_or(0);
                    if peak == 0 {
                        continue;
                    }
                    let bar_w = tw / counts.len() as f64;
                    for (i, c) in counts.iter().enumerate() {
                        if *c == 0 {
                            continue;
                        }
                        let bar_h = (*c as f64 / peak as f64) * h;
                        ops.push(DrawOp::Rect {
                            x: tx0 + i as f64 * bar_w,
                            y: y1 - bar_h,
                            w: bar_w * 0.92,
                            h: bar_h,
                            fill: Some(ps.color.clone()),
                            stroke: None,
                            sw: 0.0,
                        });
                    }
                    continue;
                }
                let (blo, bhi) = ps.box_edges();
                let Some(st) = box_stats(&vals, blo, bhi, ps.whisker_rule()) else { continue };
                let box_h = (h * 0.6).clamp(1.0, 4.0);
                if let (Some(wl), Some(wh)) = (x_at(st.whisker_lo), x_at(st.whisker_hi)) {
                    ops.push(DrawOp::Line { x1: wl, y1: mid, x2: wh, y2: mid, stroke: "#555555".into(), sw: 0.2 });
                    for x in [wl, wh] {
                        ops.push(DrawOp::Line {
                            x1: x, y1: mid - box_h / 3.0, x2: x, y2: mid + box_h / 3.0,
                            stroke: "#555555".into(), sw: 0.2,
                        });
                    }
                }
                if let (Some(lo), Some(hi)) = (x_at(st.lo), x_at(st.hi)) {
                    ops.push(DrawOp::Fill {
                        pts: vec![
                            (lo.min(hi), mid - box_h / 2.0), (lo.max(hi), mid - box_h / 2.0),
                            (lo.max(hi), mid + box_h / 2.0), (lo.min(hi), mid + box_h / 2.0),
                        ],
                        fill: ps.color.clone(),
                        opacity: 0.5,
                    });
                    ops.push(DrawOp::Rect {
                        x: lo.min(hi), y: mid - box_h / 2.0, w: (hi - lo).abs(), h: box_h,
                        fill: None, stroke: Some(ps.color.clone()), sw: 0.15,
                    });
                }
                if let Some(m) = x_at(st.med) {
                    ops.push(DrawOp::Line {
                        x1: m, y1: mid - box_h / 2.0, x2: m, y2: mid + box_h / 2.0,
                        stroke: ps.color.clone(), sw: 0.4,
                    });
                }
                // Outliers are the whole reason to prefer Tukey — print every one.
                for o in &st.outliers {
                    if let Some(x) = x_at(*o) {
                        ops.push(DrawOp::Rect {
                            x: x - 0.25, y: mid - 0.25, w: 0.5, h: 0.5,
                            fill: Some(ps.color.clone()), stroke: None, sw: 0.0,
                        });
                    }
                }
            }
        }
        _ => {
            for i in 0..depth.len().min(value.len()) {
                let d = depth[i];
                if d < page_top || d > page_bot {
                    continue;
                }
                let Some(x) = x_at(value[i]) else { continue };
                let y = y_of(d);
                // A small diamond, matching the viewer's glyph.
                ops.push(DrawOp::Fill {
                    pts: vec![(x, y - 0.7), (x + 0.7, y), (x, y + 0.7), (x - 0.7, y)],
                    fill: ps.color.clone(),
                    opacity: 1.0,
                });
            }
        }
    }
}

/// Draws one array log — a whole distribution at every depth — as a band, a spaghetti overlay
/// or a density heat map. Mirrors `drawArrayTracks` in `src/ui/logViewPanel.ts`; the two must
/// stay in agreement, and both take their statistics from `crate::distribution` so a band and
/// a point-track box plot answer the same question the same way.
#[allow(clippy::too_many_arguments)]
fn draw_array_series(
    ops: &mut Vec<DrawOp>,
    as_: &crate::layout::ArrayStyle,
    scale: ScaleType,
    rows: &[crate::db::ArrayRow],
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    use crate::distribution::{band, even_indices, histogram};
    let tw = tx1 - tx0;
    // CLAMPED at the track edge, like `draw_curve` and unlike `draw_point_series`. The rule is
    // about what the data is, not about tidiness: a discrete plug drawn at a value it never had
    // is a lie, whereas a continuous reading running past the scale is the ordinary log-display
    // convention and clipping it is what every log viewer does.
    let x_at = |v: f32| value_frac(v, as_.min, as_.max, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    let visible: Vec<&crate::db::ArrayRow> =
        rows.iter().filter(|r| r.depth >= page_top && r.depth <= page_bot).collect();
    if visible.is_empty() {
        return;
    }
    let color = as_.color.clone();

    match as_.display_kind() {
        "spaghetti" => {
            let width = visible.iter().map(|r| r.samples.len()).max().unwrap_or(0);
            for r_idx in even_indices(width, as_.traces.unwrap_or(40) as usize) {
                let mut run: Vec<(f64, f64)> = Vec::new();
                for row in &visible {
                    match row.samples.get(r_idx).copied().and_then(x_at) {
                        Some(x) => run.push((x, y_of(row.depth))),
                        // A realization that produced nothing here BREAKS its own trace. Bridging
                        // the gap would draw a path down the well that this realization never took.
                        None => {
                            if run.len() > 1 {
                                ops.push(DrawOp::Poly {
                                    pts: std::mem::take(&mut run),
                                    stroke: color.clone(),
                                    sw: 0.08,
                                });
                            } else {
                                run.clear();
                            }
                        }
                    }
                }
                if run.len() > 1 {
                    ops.push(DrawOp::Poly { pts: run, stroke: color.clone(), sw: 0.08 });
                }
            }
        }
        "heatmap" => {
            let bins = as_.hist_bins.unwrap_or(32).max(1) as usize;
            let bw = tw / bins as f64;
            for (i, row) in visible.iter().enumerate() {
                // `histogram` DROPS out-of-range values rather than clamping them, which is right
                // here for the same reason it is right on a point track: a heat-map cell is a
                // count AT a value, and a clamped sample would invent a count the data never had.
                let counts = histogram(&row.samples, as_.min, as_.max, bins);
                let peak = counts.iter().copied().max().unwrap_or(0);
                if peak == 0 {
                    continue;
                }
                // Cell extent = half-way to each neighbour, so the column tiles seamlessly at
                // whatever depth sampling the array happens to have been stored at.
                let yc = y_of(row.depth);
                let y_prev = i.checked_sub(1).map(|j| y_of(visible[j].depth));
                let y_next = visible.get(i + 1).map(|n| y_of(n.depth));
                let top = match y_prev {
                    Some(p) => (p + yc) / 2.0,
                    None => yc - y_next.map_or(0.5, |n| (n - yc) / 2.0),
                };
                let bot = match y_next {
                    Some(n) => (n + yc) / 2.0,
                    None => yc + y_prev.map_or(0.5, |p| (yc - p) / 2.0),
                };
                if bot <= top {
                    continue;
                }
                for (b, c) in counts.iter().enumerate() {
                    if *c == 0 {
                        continue;
                    }
                    // Opacity is normalised to THIS depth's peak, matching the point track's
                    // per-bin histogram scaling: it reads the shape of the distribution at each
                    // depth rather than letting one high-count interval flatten the rest.
                    ops.push(DrawOp::Fill {
                        pts: vec![
                            (tx0 + b as f64 * bw, top),
                            (tx0 + (b + 1) as f64 * bw, top),
                            (tx0 + (b + 1) as f64 * bw, bot),
                            (tx0 + b as f64 * bw, bot),
                        ],
                        fill: color.clone(),
                        opacity: *c as f64 / peak as f64,
                    });
                }
            }
        }
        // "band"
        _ => {
            let (lo_p, hi_p) = as_.band_edges();
            let opacity = as_.fill_opacity.unwrap_or(0.3) as f64;
            // Runs of consecutive summarisable depths. A depth with nothing finite is a GAP, so
            // the shading stops there instead of spanning an interval the study said nothing about.
            let mut runs: Vec<Vec<(f64, f64, f64, f64)>> = Vec::new(); // (y, x_lo, x_mid, x_hi)
            let mut run: Vec<(f64, f64, f64, f64)> = Vec::new();
            for row in &visible {
                let point = band(&row.samples, lo_p, hi_p)
                    .and_then(|(lo, med, hi)| Some((x_at(lo)?, x_at(med)?, x_at(hi)?)));
                match point {
                    Some((xl, xm, xh)) => run.push((y_of(row.depth), xl, xm, xh)),
                    None => {
                        if run.len() > 1 {
                            runs.push(std::mem::take(&mut run));
                        } else {
                            run.clear();
                        }
                    }
                }
            }
            if run.len() > 1 {
                runs.push(run);
            }
            for r in &runs {
                // Down the high edge, back up the low edge — one closed polygon per run.
                let mut pts: Vec<(f64, f64)> = r.iter().map(|(y, _, _, xh)| (*xh, *y)).collect();
                pts.extend(r.iter().rev().map(|(y, xl, _, _)| (*xl, *y)));
                ops.push(DrawOp::Fill { pts, fill: color.clone(), opacity });
                if as_.show_median.unwrap_or(true) {
                    ops.push(DrawOp::Poly {
                        pts: r.iter().map(|(y, _, xm, _)| (*xm, *y)).collect(),
                        stroke: color.clone(),
                        sw: 0.25,
                    });
                }
            }
        }
    }
}

/// Qualitative palette for discrete facies/cluster blocks.
/// Keep in sync with FACIES_PALETTE in src/ui/plotCanvas.ts.
const FACIES_PALETTE: [&str; 12] = [
    "#4e79a7", "#f28e2b", "#59a14f", "#e15759", "#b07aa1", "#76b7b2",
    "#edc948", "#ff9da7", "#9c755f", "#8cd17d", "#86bcb6", "#d37295",
];

fn facies_color(class: i64) -> &'static str {
    let n = FACIES_PALETTE.len() as i64;
    FACIES_PALETTE[(((class % n) + n) % n) as usize]
}

/// `fill = "blocks"`: contiguous same-class runs drawn as full-track-width rectangles.
/// NaN and off-page samples break runs; the class value line itself is not drawn.
#[allow(clippy::too_many_arguments)]
fn draw_class_blocks(
    ops: &mut Vec<DrawOp>,
    cs: &crate::layout::CurveStyle,
    vals: &[f32],
    depth: &[f32],
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    let opacity = cs.fill_opacity.unwrap_or(0.85) as f64;
    let push_rect = |ops: &mut Vec<DrawOp>, class: i64, top: f32, bot: f32| {
        let (y0, y1) = (y_of(top.max(page_top)), y_of(bot.min(page_bot)));
        if y1 <= y0 {
            return;
        }
        ops.push(DrawOp::Fill {
            pts: vec![(tx0, y0), (tx1, y0), (tx1, y1), (tx0, y1)],
            fill: facies_color(class).into(),
            opacity,
        });
    };

    let n = vals.len().min(depth.len());
    let mut run_class: Option<i64> = None;
    let mut run_top = 0f32;
    for i in 0..n {
        let d = depth[i];
        let v = vals[i];
        let class = if d < page_top || d > page_bot || !v.is_finite() {
            None
        } else {
            Some(v.round() as i64)
        };
        if class == run_class {
            continue;
        }
        if let Some(c) = run_class {
            push_rect(ops, c, run_top, d);
        }
        run_class = class;
        run_top = d;
    }
    if let (Some(c), true) = (run_class, n > 1) {
        // The last run extends one average sample step past its final sample.
        let avg_step = (depth[n - 1] - depth[0]) / (n as f32 - 1.0);
        push_rect(ops, c, run_top, depth[n - 1] + avg_step);
    }
}

fn draw_track_header(ops: &mut Vec<DrawOp>, track: &crate::layout::Track, tx0: f64, tx1: f64, top: f64, bot: f64) {
    ops.push(DrawOp::Rect {
        x: tx0,
        y: top,
        w: tx1 - tx0,
        h: bot - top,
        fill: Some("#fafafa".into()),
        stroke: Some("#333333".into()),
        sw: 0.2,
    });
    ops.push(DrawOp::Text {
        x: (tx0 + tx1) / 2.0,
        y: top + 4.0,
        size: 2.9,
        anchor: Anchor::Middle,
        color: "#111111".into(),
        bold: true,
        s: track.title.clone(),
    });
    if let Some(c) = track.curves.first() {
        let tag = if track.scale_type == ScaleType::Log { " (log)" } else { "" };
        ops.push(DrawOp::Text {
            x: tx0 + 0.8,
            y: bot - 1.2,
            size: 2.4,
            anchor: Anchor::Start,
            color: "#555555".into(),
            bold: false,
            s: fmt_num(c.min),
        });
        ops.push(DrawOp::Text {
            x: tx1 - 0.8,
            y: bot - 1.2,
            size: 2.4,
            anchor: Anchor::End,
            color: "#555555".into(),
            bold: false,
            s: format!("{}{}", fmt_num(c.max), tag),
        });
    }
}

fn fmt_num(v: f32) -> String {
    if v == 0.0 {
        "0".into()
    } else if v.abs() >= 100.0 || v.fract().abs() < 1e-6 {
        format!("{v:.0}")
    } else {
        format!("{v:.2}")
    }
}

// ---------------------------------------------------------------------------
// SVG serializer.
// ---------------------------------------------------------------------------

pub(crate) fn svg_page(ops: &[DrawOp], pw: f64, ph: f64) -> String {
    let mut s = String::with_capacity(16 * 1024);
    let _ = write!(
        s,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{pw}mm" height="{ph}mm" viewBox="0 0 {pw} {ph}" font-family="Helvetica, Arial, sans-serif">"##,
    );
    let _ = write!(s, r##"<rect x="0" y="0" width="{pw}" height="{ph}" fill="#ffffff"/>"##);
    for op in ops {
        match op {
            DrawOp::Rect { x, y, w, h, fill, stroke, sw } => {
                let fill = fill.as_deref().unwrap_or("none");
                let _ = write!(s, r##"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{}""##, esc(fill));
                if let Some(st) = stroke {
                    let _ = write!(s, r##" stroke="{}" stroke-width="{sw}""##, esc(st));
                }
                s.push_str("/>");
            }
            DrawOp::Line { x1, y1, x2, y2, stroke, sw } => {
                let _ = write!(
                    s,
                    r##"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{}" stroke-width="{sw}"/>"##,
                    esc(stroke),
                );
            }
            DrawOp::Poly { pts, stroke, sw } => {
                let mut d = String::new();
                for (i, (x, y)) in pts.iter().enumerate() {
                    let _ = write!(d, "{}{x:.2},{y:.2}", if i == 0 { "M" } else { " L" });
                }
                let _ = write!(
                    s,
                    r##"<path d="{d}" fill="none" stroke="{}" stroke-width="{sw}" stroke-linejoin="round"/>"##,
                    esc(stroke),
                );
            }
            DrawOp::Fill { pts, fill, opacity } => {
                let mut d = String::new();
                for (i, (x, y)) in pts.iter().enumerate() {
                    let _ = write!(d, "{}{x:.2},{y:.2}", if i == 0 { "M" } else { " L" });
                }
                d.push_str(" Z");
                let _ = write!(s, r##"<path d="{d}" fill="{}" fill-opacity="{opacity:.3}" stroke="none"/>"##, esc(fill));
            }
            DrawOp::Text { x, y, size, anchor, color, bold, s: txt } => {
                let anc = match anchor {
                    Anchor::Start => "start",
                    Anchor::Middle => "middle",
                    Anchor::End => "end",
                };
                let weight = if *bold { r##" font-weight="bold""## } else { "" };
                let _ = write!(
                    s,
                    r##"<text x="{x:.2}" y="{y:.2}" font-size="{size}" text-anchor="{anc}"{weight} fill="{}">{}</text>"##,
                    esc(color),
                    esc(txt),
                );
            }
        }
    }
    s.push_str("</svg>");
    s
}

// ---------------------------------------------------------------------------
// PDF serializer (hand-rolled, base-14 Helvetica — no font files embedded).
// ---------------------------------------------------------------------------

/// Parses "#rgb" / "#rrggbb" to normalized (r,g,b). Defaults to black on bad input.
fn hex_rgb(c: &str) -> (f64, f64, f64) {
    let h = c.trim_start_matches('#');
    let (r, g, b) = match h.len() {
        3 => {
            let p = |i: usize| u8::from_str_radix(&h[i..i + 1].repeat(2), 16).unwrap_or(0);
            (p(0), p(1), p(2))
        }
        6 => {
            let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
            (p(0), p(2), p(4))
        }
        _ => (0, 0, 0),
    };
    (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
}

/// Alpha-blends a colour toward white (PDF fill without an ExtGState alpha channel).
fn blend_white((r, g, b): (f64, f64, f64), opacity: f64) -> (f64, f64, f64) {
    let o = opacity.clamp(0.0, 1.0);
    (r * o + (1.0 - o), g * o + (1.0 - o), b * o + (1.0 - o))
}

fn pdf_escape(s: &str) -> String {
    // Keep it to ASCII (Helvetica/WinAnsi); replace non-ASCII with '-'.
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            c if c.is_ascii() => out.push(c),
            _ => out.push('-'),
        }
    }
    out
}

/// Builds one page's PDF content stream from its draw-ops. mm→pt with a top-left→bottom-left
/// y-flip so the page reads the same as the SVG.
pub(crate) fn pdf_content(ops: &[DrawOp], _pw: f64, ph: f64) -> String {
    let mut s = String::with_capacity(16 * 1024);
    let tx = |x: f64| x * PT_PER_MM;
    let ty = |y: f64| (ph - y) * PT_PER_MM;

    for op in ops {
        match op {
            DrawOp::Rect { x, y, w, h, fill, stroke, sw } => {
                // PDF rect origin is the lower-left corner.
                let x0 = tx(*x);
                let y0 = ty(*y + *h);
                let (wp, hp) = (w * PT_PER_MM, h * PT_PER_MM);
                if let Some(f) = fill {
                    let (r, g, b) = hex_rgb(f);
                    let _ = write!(s, "{r:.3} {g:.3} {b:.3} rg\n{x0:.2} {y0:.2} {wp:.2} {hp:.2} re f\n");
                }
                if let Some(st) = stroke {
                    let (r, g, b) = hex_rgb(st);
                    let _ = write!(
                        s,
                        "{r:.3} {g:.3} {b:.3} RG\n{:.3} w\n{x0:.2} {y0:.2} {wp:.2} {hp:.2} re S\n",
                        sw * PT_PER_MM,
                    );
                }
            }
            DrawOp::Line { x1, y1, x2, y2, stroke, sw } => {
                let (r, g, b) = hex_rgb(stroke);
                let _ = write!(
                    s,
                    "{r:.3} {g:.3} {b:.3} RG\n{:.3} w\n{:.2} {:.2} m {:.2} {:.2} l S\n",
                    sw * PT_PER_MM,
                    tx(*x1),
                    ty(*y1),
                    tx(*x2),
                    ty(*y2),
                );
            }
            DrawOp::Poly { pts, stroke, sw } => {
                if pts.len() < 2 {
                    continue;
                }
                let (r, g, b) = hex_rgb(stroke);
                let _ = write!(s, "{r:.3} {g:.3} {b:.3} RG\n{:.3} w\n1 j\n", sw * PT_PER_MM);
                for (i, (x, y)) in pts.iter().enumerate() {
                    let _ = write!(s, "{:.2} {:.2} {}\n", tx(*x), ty(*y), if i == 0 { "m" } else { "l" });
                }
                s.push_str("S\n");
            }
            DrawOp::Fill { pts, fill, opacity } => {
                if pts.len() < 3 {
                    continue;
                }
                let (r, g, b) = blend_white(hex_rgb(fill), *opacity);
                let _ = write!(s, "{r:.3} {g:.3} {b:.3} rg\n");
                for (i, (x, y)) in pts.iter().enumerate() {
                    let _ = write!(s, "{:.2} {:.2} {}\n", tx(*x), ty(*y), if i == 0 { "m" } else { "l" });
                }
                s.push_str("f\n");
            }
            DrawOp::Text { x, y, size, anchor, color, bold, s: txt } => {
                let (r, g, b) = hex_rgb(color);
                let size_pt = size * PT_PER_MM;
                // Helvetica average advance ≈ 0.5 em — enough to place centred/right labels.
                let width = txt.chars().count() as f64 * size_pt * 0.5;
                let x_pt = match anchor {
                    Anchor::Start => tx(*x),
                    Anchor::Middle => tx(*x) - width / 2.0,
                    Anchor::End => tx(*x) - width,
                };
                let font = if *bold { "/F2" } else { "/F1" };
                let _ = write!(
                    s,
                    "BT {font} {size_pt:.2} Tf\n{r:.3} {g:.3} {b:.3} rg\n{x_pt:.2} {:.2} Td\n({}) Tj\nET\n",
                    ty(*y),
                    pdf_escape(txt),
                );
            }
        }
    }
    s
}

/// Assembles per-page content streams into a single multi-page PDF document.
pub(crate) fn assemble_pdf(streams: &[String], pw: f64, ph: f64) -> Vec<u8> {
    let wp = pw * PT_PER_MM;
    let hp = ph * PT_PER_MM;
    let n = streams.len();

    // Object ids: 1 catalog, 2 pages, 3 F1, 4 F2, then per page (content, page).
    let n_fixed = 4;
    let mut objects: Vec<String> = Vec::new(); // body of each object, in id order

    // 1: Catalog
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".into());

    // 2: Pages (kids filled below)
    let mut kids = String::new();
    for i in 0..n {
        let page_id = n_fixed + 1 + 2 * i + 1; // content then page
        let _ = write!(kids, "{page_id} 0 R ");
    }
    objects.push(format!("<< /Type /Pages /Kids [ {}] /Count {} >>", kids.trim_end(), n));

    // 3, 4: fonts
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".into());

    // Per page: content stream object then page object.
    for (i, stream) in streams.iter().enumerate() {
        let content_id = n_fixed + 1 + 2 * i;
        let page_id = content_id + 1;
        objects.push(format!("<< /Length {} >>\nstream\n{stream}endstream", stream.len()));
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {wp:.2} {hp:.2}] \
             /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {content_id} 0 R >>",
        ));
        let _ = page_id;
    }

    // Serialize with a cross-reference table.
    let mut out = String::from("%PDF-1.7\n%\u{00e2}\u{00e3}\u{00cf}\u{00d3}\n");
    let mut offsets = vec![0usize; objects.len() + 1];
    for (i, body) in objects.iter().enumerate() {
        offsets[i + 1] = out.len();
        out.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
    }
    let xref_pos = out.len();
    out.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
    out.push_str("0000000000 65535 f \n");
    for i in 1..=objects.len() {
        out.push_str(&format!("{:010} 00000 n \n", offsets[i]));
    }
    out.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n",
        objects.len() + 1,
    ));
    out.into_bytes()
}

/// Wraps a single frontend-built content stream as a one-page PDF sized `w_pt`×`h_pt`.
///
/// The stream must already be in PDF user space — points, bottom-left origin, `/F1` (Helvetica)
/// / `/F2` (Helvetica-Bold) for text — exactly what the browser-side `PdfRecorder` in
/// `pdfExport.ts` emits for a single Canvas-2D chart. Reusing [`assemble_pdf`] keeps all the
/// document scaffolding (catalog, xref offsets, font objects) in one tested place; the page
/// size round-trips through mm because `assemble_pdf` multiplies it straight back by `PT_PER_MM`.
pub(crate) fn assemble_single_page_pdf(content: &str, w_pt: f64, h_pt: f64) -> Vec<u8> {
    assemble_pdf(&[content.to_string()], w_pt / PT_PER_MM, h_pt / PT_PER_MM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::layout::standard_layout;
    use uuid::Uuid;

    fn seed_well(conn: &Connection) -> String {
        db::create_schema(conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(conn, wid, "BLSO-COMPOSITE", Some("Balam South"), Some(1800.0), Some(25.0)).unwrap();
        let w = wid.to_string();
        let n = 400;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 40.0 + 60.0 * ((i as f32 / 20.0).sin() * 0.5 + 0.5)).collect();
        let res: Vec<f32> = (0..n).map(|i| 1.0 + (i % 50) as f32).collect();
        let nphi: Vec<f32> = vec![0.25; n];
        let rhob: Vec<f32> = vec![2.4; n];
        db::insert_standard_curves(conn, wid, depths, gr, res, nphi, rhob, vec![f32::NAN; n], vec![f32::NAN; n]).unwrap();
        db::upsert_top(conn, &w, "Top Reservoir", 1050.0, Some("#b0413e")).unwrap();
        db::upsert_zone(conn, &w, "ZoneA", 1040.0, 1120.0).unwrap();
        w
    }

    fn full_spec(w: String, scale: u32, page: PageSize) -> CompositeSpec {
        CompositeSpec { well_id: w, layout: standard_layout(), depth_top: None, depth_bottom: None, scale, page_size: page }
    }

    /// A minimal saved curve style — deliberately built from the JSON a PRE-crossover layout
    /// would have stored, so these tests also prove the new display fields are optional.
    fn saved_style(name: &str) -> crate::layout::CurveStyle {
        serde_json::from_value(serde_json::json!({
            "curve_name": name, "color": "#000000", "min": 0.0, "max": 1.0
        }))
        .unwrap()
    }

    /// Page coordinates come from f32 samples widened to f64, so they land a few ULPs off a
    /// round number. Round to micro-millimetres — far finer than any printer — so these
    /// geometry tests can compare exact expected corners.
    fn round_pts(pts: &[(f64, f64)]) -> Vec<(f64, f64)> {
        pts.iter().map(|(x, y)| ((x * 1e6).round() / 1e6, (y * 1e6).round() / 1e6)).collect()
    }

    fn fills(ops: &[DrawOp]) -> Vec<(String, Vec<(f64, f64)>)> {
        ops.iter()
            .filter_map(|o| match o {
                DrawOp::Fill { pts, fill, .. } => Some((fill.clone(), round_pts(pts))),
                _ => None,
            })
            .collect()
    }

    fn line_pts(ops: &[DrawOp]) -> Vec<(f64, f64)> {
        ops.iter()
            .find_map(|o| match o {
                DrawOp::Poly { pts, .. } => Some(round_pts(pts)),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn layouts_saved_before_crossover_still_load() {
        let cs = saved_style("GR");
        assert!(cs.draw_style.is_none() && cs.fill_to.is_none() && cs.fill_color2.is_none());
    }

    #[test]
    fn blocky_style_holds_each_value_down_to_the_next_sample() {
        let vals = [0.2f32, 0.8, 0.8];
        let depth = [100.0f32, 101.0, 102.0];
        let y = |d: f32| d as f64;

        let mut ops = Vec::new();
        let cs = saved_style("VSH");
        draw_curve(&mut ops, &cs, ScaleType::Linear, &vals, &depth, None, 0.0, 10.0, 0.0, 1000.0, &y);
        // Continuous: one diagonal per interval, so the value slides between sample centres.
        assert_eq!(line_pts(&ops), vec![(2.0, 100.0), (8.0, 101.0), (8.0, 102.0)]);

        let mut blocky = cs.clone();
        blocky.draw_style = Some("step".into());
        let mut ops = Vec::new();
        draw_curve(&mut ops, &blocky, ScaleType::Linear, &vals, &depth, None, 0.0, 10.0, 0.0, 1000.0, &y);
        // Blocky: 0.2 holds at x = 2 all the way to 101 before jumping to 0.8 — no gradient
        // the data never measured.
        let pts = line_pts(&ops);
        assert_eq!(pts[0], (2.0, 100.0));
        assert_eq!(pts[1], (2.0, 101.0));
        assert_eq!(pts[2], (8.0, 101.0));
    }

    #[test]
    fn crossover_shades_each_side_of_the_reference_in_its_own_colour() {
        let mut cs = saved_style("NPHI");
        cs.fill = Some("curve".into());
        cs.fill_to = Some("RHOB".into());
        cs.fill_color = Some("#111111".into()); // left of the reference
        cs.fill_color2 = Some("#222222".into()); // right of it
        // The styled curve starts left of the reference and ends right of it, crossing
        // exactly midway through the interval.
        let vals = [0.2f32, 0.8];
        let refv = [0.8f32, 0.2];
        let depth = [100.0f32, 102.0];
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_curve(
            &mut ops, &cs, ScaleType::Linear, &vals, &depth, Some((&refv, 0.0, 1.0)),
            0.0, 10.0, 0.0, 1000.0, &y,
        );
        let f = fills(&ops);
        assert_eq!(f.len(), 2, "a crossing interval splits into two shaded pieces");
        assert_eq!(f[0].0, "#111111");
        assert_eq!(f[1].0, "#222222");
        // The pieces meet exactly on the crossover — mid-depth, mid-track — instead of one
        // colour bleeding a whole sample past it.
        assert_eq!(f[0].1.last().copied(), Some((5.0, 101.0)));
        assert_eq!(f[1].1[0], (5.0, 101.0));
    }

    #[test]
    fn crossover_without_its_reference_curve_shades_nothing_but_still_draws_the_line() {
        let mut cs = saved_style("NPHI");
        cs.fill = Some("curve".into());
        cs.fill_to = Some("MISSING".into());
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_curve(
            &mut ops, &cs, ScaleType::Linear, &[0.2f32, 0.8], &[100.0f32, 101.0], None,
            0.0, 10.0, 0.0, 1000.0, &y,
        );
        assert!(fills(&ops).is_empty());
        assert_eq!(line_pts(&ops).len(), 2);
    }

    #[test]
    fn a_style_saved_with_fill_none_prints_unshaded() {
        // The properties dialog writes fill = "none" when you pick None. The exporter used to
        // treat any non-"right" value as a left-edge fill, so the print disagreed with screen.
        let mut cs = saved_style("GR");
        cs.fill = Some("none".into());
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_curve(
            &mut ops, &cs, ScaleType::Linear, &[0.2f32, 0.8], &[100.0f32, 101.0], None,
            0.0, 10.0, 0.0, 1000.0, &y,
        );
        assert!(fills(&ops).is_empty(), "None must print unshaded, as the viewer draws it");
    }

    fn point_style(json: serde_json::Value) -> crate::layout::PointStyle {
        serde_json::from_value(json).unwrap()
    }

    /// 20 core plugs over 10 m: φ climbing 0.10 → 0.29, plus one absurd plug.
    fn plugs() -> (Vec<f32>, Vec<f32>) {
        let d: Vec<f32> = (0..20).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let mut v: Vec<f32> = (0..20).map(|i| 0.10 + i as f32 * 0.01).collect();
        v[3] = 0.95; // a plug no sandstone ever had — must survive as an outlier, not vanish
        (d, v)
    }

    #[test]
    fn a_point_series_prints_one_box_per_depth_bin() {
        let (d, v) = plugs();
        let core: Vec<(String, f32, f32)> =
            d.iter().zip(&v).map(|(d, v)| ("CPOR".to_string(), *d, *v)).collect();
        let ps = point_style(serde_json::json!({
            "source": "core", "item": "CPOR", "color": "#5f7350",
            "min": 0.0, "max": 1.0, "display": "box", "bin": 5.0
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_point_series(
            &mut ops, &ps, ScaleType::Linear, &core, &[], 0.0, 10.0, 990.0, 1020.0, &y,
        );
        // Plugs span 1000.0–1009.5, so a 5 m bin gives exactly two bins, each with a filled
        // box, a median rule and a whisker spine.
        let boxes = fills(&ops).len();
        assert_eq!(boxes, 2, "one box body per populated depth bin");
        assert!(ops.iter().filter(|o| matches!(o, DrawOp::Line { .. })).count() >= 6);
        // The 0.95 plug is beyond the Tukey fence, so it prints as its own mark rather than
        // stretching the box that summarises the real plugs.
        assert!(ops.iter().any(|o| matches!(o, DrawOp::Rect { w, .. } if (*w - 0.5).abs() < 1e-9)));
    }

    #[test]
    fn point_samples_off_the_track_scale_are_skipped_not_clamped() {
        let core = vec![
            ("CPOR".to_string(), 1000.0f32, 0.20f32),
            ("CPOR".to_string(), 1001.0, 5.0), // far off a 0–1 porosity axis
        ];
        let ps = point_style(serde_json::json!({
            "source": "core", "item": "CPOR", "color": "#000000", "min": 0.0, "max": 1.0
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_point_series(&mut ops, &ps, ScaleType::Linear, &core, &[], 0.0, 10.0, 990.0, 1020.0, &y);
        assert_eq!(fills(&ops).len(), 1, "the off-scale plug must not print at the track edge");
    }

    #[test]
    fn a_text_point_series_prints_its_labels_from_an_aux_dataset() {
        let aux = vec![
            crate::db::AuxRow {
                dataset: "CORE".into(), depth_top: 1000.0, depth_base: Some(1002.0),
                item: "LITH".into(), value_num: None, value_text: Some("Sandstone, fine".into()),
            },
            crate::db::AuxRow {
                dataset: "CORE".into(), depth_top: 1010.0, depth_base: None,
                item: "LITH".into(), value_num: None, value_text: Some("Shale".into()),
            },
        ];
        let ps = point_style(serde_json::json!({
            "source": "aux", "dataset": "CORE", "item": "LITH", "color": "#000000",
            "min": 0.0, "max": 1.0, "display": "text"
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_point_series(&mut ops, &ps, ScaleType::Linear, &[], &aux, 0.0, 20.0, 990.0, 1020.0, &y);
        let texts: Vec<(f64, String)> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Text { y, s, .. } => Some((*y, s.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0].1, "Sandstone, fine");
        // An interval sample is anchored at its MIDDLE (1000–1002 → 1001), where the
        // description actually applies, not at its top.
        assert!((texts[0].0 - 1001.8).abs() < 0.01);
    }

    fn array_style(json: serde_json::Value) -> crate::layout::ArrayStyle {
        serde_json::from_value(json).unwrap()
    }

    /// 5 depths x 11 realizations, centred on phi = 0.15 and widening downwards. With 11
    /// evenly-spaced values the percentiles land exactly on samples: P10 = centre - 0.8*spread,
    /// P50 = centre, P90 = centre + 0.8*spread — so the band geometry is checkable by hand.
    fn realizations() -> Vec<crate::db::ArrayRow> {
        (0..5)
            .map(|i| {
                let spread = 0.02 * (i as f32 + 1.0);
                crate::db::ArrayRow {
                    depth: 1000.0 + i as f32,
                    samples: (0..=10).map(|k| 0.15 - spread + 2.0 * spread * (k as f32 / 10.0)).collect(),
                }
            })
            .collect()
    }

    #[test]
    fn an_array_band_prints_one_polygon_down_the_high_edge_and_back_up_the_low() {
        let rows = realizations();
        let a = array_style(serde_json::json!({
            "curve_name": "MC_PHIE_REAL", "color": "#4e79a7", "min": 0.0, "max": 1.0
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_array_series(&mut ops, &a, ScaleType::Linear, &rows, 0.0, 100.0, 990.0, 1010.0, &y);

        let f = fills(&ops);
        assert_eq!(f.len(), 1, "one closed polygon for one unbroken run");
        assert_eq!(f[0].1.len(), 10, "5 depths down the high edge, 5 back up the low");
        // Top depth: spread 0.02, so P90 = 0.166 -> x 16.6 and P10 = 0.134 -> x 13.4.
        assert!((f[0].1[0].0 - 16.6).abs() < 1e-3, "high edge {:?}", f[0].1[0]);
        assert_eq!(f[0].1[0].1, 1000.0);
        assert!((f[0].1[9].0 - 13.4).abs() < 1e-3, "low edge closes the polygon {:?}", f[0].1[9]);
        // P50 is the median line, at the centre regardless of how wide the band gets.
        let med = line_pts(&ops);
        assert_eq!(med.len(), 5);
        assert!(med.iter().all(|(x, _)| (x - 15.0).abs() < 1e-3), "median: {med:?}");
    }

    #[test]
    fn the_median_line_can_be_switched_off_without_losing_the_band() {
        let a = array_style(serde_json::json!({
            "curve_name": "MC_PHIE_REAL", "color": "#4e79a7", "min": 0.0, "max": 1.0,
            "show_median": false
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_array_series(&mut ops, &a, ScaleType::Linear, &realizations(), 0.0, 100.0, 990.0, 1010.0, &y);
        assert_eq!(fills(&ops).len(), 1);
        assert!(!ops.iter().any(|o| matches!(o, DrawOp::Poly { .. })));
    }

    #[test]
    fn a_depth_where_nothing_converged_breaks_the_band_instead_of_spanning_it() {
        let mut rows = realizations();
        rows[2].samples = vec![f32::NAN; 11]; // every realization failed at this depth
        let a = array_style(serde_json::json!({
            "curve_name": "MC_PHIE_REAL", "color": "#4e79a7", "min": 0.0, "max": 1.0
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_array_series(&mut ops, &a, ScaleType::Linear, &rows, 0.0, 100.0, 990.0, 1010.0, &y);
        // Two polygons with a hole between them — shading straight through would claim an
        // uncertainty range at a depth the study produced no answer for.
        let f = fills(&ops);
        assert_eq!(f.len(), 2, "the gap must split the shading");
        assert!(f.iter().all(|(_, pts)| pts.len() == 4), "two depths each: {f:?}");
    }

    #[test]
    fn spaghetti_draws_the_asked_for_number_of_traces_and_breaks_them_at_failures() {
        let rows = realizations();
        let a = array_style(serde_json::json!({
            "curve_name": "MC_PHIE_REAL", "color": "#4e79a7", "min": 0.0, "max": 1.0,
            "display": "spaghetti", "traces": 3
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_array_series(&mut ops, &a, ScaleType::Linear, &rows, 0.0, 100.0, 990.0, 1010.0, &y);
        let polys: Vec<usize> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Poly { pts, .. } => Some(pts.len()),
                _ => None,
            })
            .collect();
        assert_eq!(polys, vec![5, 5, 5], "three traces, each spanning all five depths");

        // Realization 5 fails at one depth: its own trace splits, the other two are untouched.
        let mut broken = realizations();
        broken[2].samples[5] = f32::NAN;
        let mut ops2 = Vec::new();
        draw_array_series(&mut ops2, &a, ScaleType::Linear, &broken, 0.0, 100.0, 990.0, 1010.0, &y);
        let mut lens: Vec<usize> = ops2
            .iter()
            .filter_map(|o| match o {
                DrawOp::Poly { pts, .. } => Some(pts.len()),
                _ => None,
            })
            .collect();
        lens.sort_unstable();
        assert_eq!(lens, vec![2, 2, 5, 5], "the failed trace becomes two runs, not one bridged line");
    }

    #[test]
    fn heatmap_cells_off_the_track_scale_are_dropped_not_clamped() {
        let rows: Vec<crate::db::ArrayRow> = (0..5)
            .map(|i| crate::db::ArrayRow {
                depth: 1000.0 + i as f32,
                // The middle depth sits far off the 0..1 track: it must contribute NO cell
                // rather than a false column of density at the track edge.
                samples: vec![if i == 2 { 5.0 } else { 0.5 }; 11],
            })
            .collect();
        let a = array_style(serde_json::json!({
            "curve_name": "MC_PHIE_REAL", "color": "#4e79a7", "min": 0.0, "max": 1.0,
            "display": "heatmap", "hist_bins": 4
        }));
        let mut ops = Vec::new();
        let y = |x: f32| x as f64;
        draw_array_series(&mut ops, &a, ScaleType::Linear, &rows, 0.0, 100.0, 990.0, 1010.0, &y);
        let f = fills(&ops);
        assert_eq!(f.len(), 4, "one occupied cell at each of the four in-range depths");
        // All 11 realizations land in the third of four bins (0.5 -> bin 2), spanning x 50..75.
        assert!(f.iter().all(|(_, pts)| (pts[0].0 - 50.0).abs() < 1e-6 && (pts[1].0 - 75.0).abs() < 1e-6));
        assert!(
            ops.iter().all(|o| matches!(o, DrawOp::Fill { opacity, .. } if (*opacity - 1.0).abs() < 1e-9)),
            "a single occupied bin is this depth's peak, so it draws at full opacity"
        );
    }

    #[test]
    fn an_array_log_round_trips_through_the_store_with_its_realization_order_intact() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "ARRAYS", None, None, None).unwrap();
        let w = wid.to_string();

        let depths: Vec<f32> = vec![1000.0, 1000.5, 1001.0];
        let vals: Vec<Vec<f32>> = vec![
            vec![0.1, 0.2, f32::NAN],
            vec![0.15, 0.25, 0.35],
            vec![], // nothing survived here
        ];
        let n = db::write_array_log(&conn, &w, "MONTECARLO", "MC_PHIE_REAL", &depths, &vals).unwrap();
        assert_eq!(n, 2, "the empty depth is skipped, not stored as a zero-width distribution");

        let back = db::read_array_log(&conn, &w, Some("MONTECARLO"), "mc_phie_real").unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].depth, 1000.0);
        assert_eq!(back[0].samples[0], 0.1);
        assert_eq!(back[0].samples[1], 0.2);
        assert!(back[0].samples[2].is_nan(), "a failed realization keeps its SLOT so index r stays stable");
        assert_eq!(back[1].samples, vec![0.15, 0.25, 0.35]);

        // A re-run replaces its own output rather than unioning two runs' realizations.
        db::write_array_log(&conn, &w, "MONTECARLO", "MC_PHIE_REAL", &[1000.0], &[vec![0.9, 0.8]]).unwrap();
        let after = db::read_array_log(&conn, &w, None, "MC_PHIE_REAL").unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].samples, vec![0.9, 0.8]);

        let cat = db::list_array_curves(&conn, &w).unwrap();
        assert_eq!(cat.len(), 1);
        assert_eq!((cat[0].set_name.as_str(), cat[0].curve_name.as_str()), ("MONTECARLO", "MC_PHIE_REAL"));
        assert_eq!((cat[0].depths, cat[0].width), (1, 2));

        assert_eq!(db::delete_array_log(&conn, &w, "MONTECARLO", "MC_PHIE_REAL").unwrap(), 1);
        assert!(db::list_array_curves(&conn, &w).unwrap().is_empty());
    }

    #[test]
    fn the_core_point_reader_drops_empty_cells_instead_of_reading_them_as_zero() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "POINTS", None, None, None).unwrap();
        let w = wid.to_string();
        conn.execute_batch(&format!(
            "INSERT INTO core_data (well_id, set_name, depth, cpor, cperm, cgd, csw) VALUES
               ('{w}', 'RAW', 1000.0, 0.21, 150.0, NULL, NULL),
               ('{w}', 'RAW', 1000.5, 0.19, NULL, 2.65, NULL);"
        ))
        .unwrap();
        let rows = db::get_core_point_series(&conn, &w).unwrap();
        let names: Vec<&str> = rows.iter().map(|(n, _, _)| n.as_str()).collect();
        assert_eq!(names, vec!["CPOR", "CPERM", "CPOR", "CGD"]);
        // A blank grain-density column must contribute no plug at all — a 0.0 g/cc plug
        // would land at the left edge of a density track and read as real data.
        assert!(!rows.iter().any(|(_, _, v)| *v == 0.0));
    }

    #[test]
    fn the_standard_layout_porosity_track_carries_the_neutron_density_crossover() {
        let t = standard_layout().tracks.into_iter().find(|t| t.title == "NPHI / RHOB").unwrap();
        let nphi = t.curves.iter().find(|c| c.curve_name == "NPHI").unwrap();
        assert_eq!(nphi.fill.as_deref(), Some("curve"));
        assert_eq!(nphi.fill_to.as_deref(), Some("RHOB"));
        // Two distinct colours, or the separation carries no reading.
        assert_ne!(nphi.fill_color, nphi.fill_color2);
        // The reference must live in the SAME track — its own min/max is what positions it.
        assert!(t.curves.iter().any(|c| c.curve_name == "RHOB"));
    }

    #[test]
    fn composite_paginates_and_renders_structure() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn);
        let res = render_composite(&conn, &full_spec(w, 500, PageSize::A4)).unwrap();
        assert!(res.pages.len() >= 2, "expected multi-page, got {}", res.pages.len());
        assert_eq!(res.well_name, "BLSO-COMPOSITE");

        let p0 = &res.pages[0].svg;
        assert!(p0.starts_with("<svg"));
        assert!(p0.contains("width=\"210mm\""));
        assert!(p0.contains("BLSO-COMPOSITE"));
        assert!(p0.contains("Balam South"));
        assert!(p0.contains("1:500"));
        assert!(p0.contains("Top Reservoir"));
        assert!(p0.contains("RES_DEEP"));
        assert!(p0.contains("<path"));

        for pair in res.pages.windows(2) {
            assert!((pair[1].top_depth - pair[0].bottom_depth).abs() < 1e-3);
        }
        assert!((res.pages.first().unwrap().top_depth - 1000.0).abs() < 1e-3);
        assert!((res.pages.last().unwrap().bottom_depth - 1199.5).abs() < 0.6);
    }

    #[test]
    fn print_scale_is_physically_exact() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn);
        let mut spec = full_spec(w, 200, PageSize::A3);
        spec.depth_top = Some(1000.0);
        spec.depth_bottom = Some(1010.0);
        let res = render_composite(&conn, &spec).unwrap();
        assert_eq!(res.pages.len(), 1);
        assert!((1000.0 / spec.scale as f64 - 5.0).abs() < 1e-9);
    }

    #[test]
    fn pdf_is_valid_and_multipage() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn);
        let spec = full_spec(w, 500, PageSize::A4);
        let n_pages = render_composite(&conn, &spec).unwrap().pages.len();
        let pdf = render_composite_pdf(&conn, &spec).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.7"), "PDF header");
        assert!(pdf.ends_with(b"%%EOF\n"), "PDF trailer");
        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(text.matches("/Type /Page ").count(), n_pages, "one Page object per composite page");
        assert!(text.contains("/BaseFont /Helvetica"));
        assert!(text.contains("startxref"));
        // Content streams must carry drawing operators (text-show + stroke).
        assert!(text.contains(" Tj"), "text show operator present");
        assert!(text.contains(" re "), "rectangles present");
    }

    #[test]
    fn single_page_pdf_wraps_a_content_stream_at_point_size() {
        // A trivial content stream (one stroked line) becomes a valid one-page PDF whose
        // MediaBox is exactly the requested point size and whose stream is embedded verbatim.
        let content = "0.000 0.000 0.000 RG\n1.00 w\n10.00 10.00 m\n100.00 90.00 l\nS\n";
        let pdf = assemble_single_page_pdf(content, 300.0, 200.0);
        assert!(pdf.starts_with(b"%PDF-1.7"), "PDF header");
        assert!(pdf.ends_with(b"%%EOF\n"), "PDF trailer");
        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(text.matches("/Type /Page ").count(), 1, "exactly one page");
        assert!(text.contains("/MediaBox [0 0 300.00 200.00]"), "MediaBox at point size");
        assert!(text.contains("10.00 10.00 m"), "content stream embedded verbatim");
        assert!(text.contains("startxref"));
    }

    #[test]
    fn hex_rgb_parses_3_and_6_digit() {
        assert_eq!(hex_rgb("#000000"), (0.0, 0.0, 0.0));
        assert_eq!(hex_rgb("#fff"), (1.0, 1.0, 1.0));
        let (r, _, _) = hex_rgb("#ff0000");
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn nice_step_rounds_sensibly() {
        assert_eq!(nice_step(1.1), 1.0);
        assert_eq!(nice_step(2.0), 2.0);
        assert_eq!(nice_step(4.0), 5.0);
        assert_eq!(nice_step(23.0), 20.0);
    }

    /// Dumps a sample first-page SVG + the full PDF to the scratchpad for eyeballing.
    /// Ignored by default: `cargo test --lib composite::tests::dump_sample -- --ignored`.
    #[test]
    #[ignore]
    fn dump_sample() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn);
        let mut spec = full_spec(w, 200, PageSize::A4);
        spec.depth_top = Some(1040.0);
        spec.depth_bottom = Some(1120.0);
        let res = render_composite(&conn, &spec).unwrap();
        let svg_out = std::env::var("ARSHILLA_SVG_OUT")
            .unwrap_or_else(|_| std::env::temp_dir().join("arshilla_composite_p0.svg").to_string_lossy().into());
        std::fs::write(&svg_out, &res.pages[0].svg).unwrap();
        let pdf = render_composite_pdf(&conn, &spec).unwrap();
        let pdf_out = std::env::var("ARSHILLA_PDF_OUT")
            .unwrap_or_else(|_| std::env::temp_dir().join("arshilla_composite.pdf").to_string_lossy().into());
        std::fs::write(&pdf_out, &pdf).unwrap();
        println!("wrote {svg_out} and {pdf_out}");
    }
}
