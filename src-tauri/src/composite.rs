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
use crate::plotting::{apply_plot_channel_policy, DisplayRange, PlotChannelPolicy};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
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
    pub well_name: String
,
    /// Complete ancestry for every current computed curve carried by this well's
    /// number-bearing composite deliverable. The same JSON is embedded in SVG/PDF
    /// exports so it survives outside the application.
    pub ancestry: Vec<equations::CurveAncestryDisclosure>,
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

/// One picture prepared for printing: the stored bytes plus what the two back-ends need to
/// place them without decoding. `id` is dense across a whole render so the PDF can name each
/// XObject `/Im{id}` and the resources dictionary can be built from the ops alone.
pub(crate) struct PrintImage {
    pub(crate) id: usize,
    pub(crate) mime: String,
    pub(crate) data: Vec<u8>,
    pub(crate) px_w: u32,
    pub(crate) px_h: u32,
    /// JPEG colour components (1 grey / 3 RGB / 4 CMYK) — decides the PDF colour space.
    pub(crate) components: u8,
    /// The plate's own name, so a serializer that cannot embed it can say WHICH plate is
    /// standing behind the frame it drew instead.
    pub(crate) name: String,
}

impl PrintImage {
    /// Whether the PDF exporter can carry this picture's bytes. Only a JPEG can: they go in
    /// untouched under `/DCTDecode`, and there is no other format this writer encodes.
    ///
    /// AUDIT-2026-08-20 finding 42. This is a PDF constraint and it now lives with the PDF
    /// writer, not in the shared draw-op list. It used to be decided when the ops were BUILT,
    /// so the SVG export — which inlines any MIME as a data URI and had no such limit —
    /// printed a red "not embeddable" frame over a plate it could have shown perfectly well.
    /// Everything the store can hold is JPEG, PNG, GIF or WebP (`images::browser_decodable`
    /// gates the import), so every plate is embeddable in the SVG and only JPEG in the PDF.
    pub(crate) fn pdf_embeddable(&self) -> bool {
        self.mime == "image/jpeg"
    }
}

pub(crate) enum DrawOp {
    Rect { x: f64, y: f64, w: f64, h: f64, fill: Option<String>, stroke: Option<String>, sw: f64 },
    /// A raster placed in the given millimetre box. `cover = false` means the box IS the
    /// picture (already fitted to its aspect ratio by the caller, so nothing is distorted);
    /// `cover = true` means fill the box and crop the overhang, which both back-ends do by
    /// clipping to the box rather than by squashing the image.
    Image { x: f64, y: f64, w: f64, h: f64, img: std::sync::Arc<PrintImage>, cover: bool },
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

    let curve_requests: Vec<equations::TrackCurveRequest> = spec
        .layout
        .tracks
        .iter()
        .flat_map(|track| {
            track.curves.iter().map(|curve| equations::TrackCurveRequest {
                curve_name: curve.curve_name.clone(),
                set_name: curve.set_name.clone(),
                // The print reads through `fetch_track_frames`, which never decimated, so this
                // changes nothing here. Set truthfully anyway: a field that says "not a class
                // curve" about the FACIES track would be a false statement waiting for the
                // first reader who believes it.
                class_curve: curve.fill.as_deref() == Some("blocks"),
            })
        })
        .collect();
    let frames = equations::fetch_track_frames(conn, &spec.well_id, &curve_requests, None, None)
        .map_err(|error| error.to_string())?;
    let mut data_top = f32::INFINITY;
    let mut data_bot = f32::NEG_INFINITY;
    for frame in &frames {
        for depth in &frame.depth {
            if depth.is_finite() {
                data_top = data_top.min(*depth);
                data_bot = data_bot.max(*depth);
            }
        }
    }
    // Curve-free point/image/diagram layouts historically inherit the well's standard depth
    // interval. Keep that behavior without using it to resolve any set-qualified curve.
    if !data_top.is_finite() || !data_bot.is_finite() {
        let (fallback_depth, _) =
            equations::fetch_curve_frame(conn, &spec.well_id, &[]).map_err(|error| error.to_string())?;
        for depth in fallback_depth {
            if depth.is_finite() {
                data_top = data_top.min(depth);
                data_bot = data_bot.max(depth);
            }
        }
    }
    if !data_top.is_finite() || !data_bot.is_finite() {
        return Err("no curve data for this well".into());
    }
    let curve_frames: HashMap<String, equations::TrackCurveFrame> = frames
        .into_iter()
        .map(|frame| (frame.curve_name.clone(), frame))
        .collect();
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

    // Depth-registered pictures for any image track, fetched once for the whole render and
    // keyed by dataset. Read whole rather than page by page for the same reason as the array
    // logs — and because a plate that straddles a page break must be placeable on both.
    let mut images: HashMap<String, Vec<PrintEntry>> = HashMap::new();
    let mut next_image_id = 0usize;
    for track in spec.layout.tracks.iter().filter(|t| t.kind == crate::layout::TrackKind::Image) {
        for st in &track.images {
            let key = st.dataset.trim().to_uppercase();
            if key.is_empty() || images.contains_key(&key) {
                continue;
            }
            // NOT unwrap_or_default. CLAUDE.md's rule for this store is that a plate the PDF
            // cannot embed prints a NAMED FRAME, never a silent gap, so a deliverable can be
            // checked against the delivery list - and swallowing the read turned a refused or
            // failed dataset into an empty track that reads as "no plates in this interval".
            // The layout ASKED for this track; if it cannot be honoured the export says why.
            let rows = crate::db::read_images_for_print(conn, &spec.well_id, &key, top, bottom)
                .map_err(|e| format!("cannot print the {key} pictures: {e}"))?;
            let entries = rows
                .into_iter()
                .map(|(info, data)| {
                    let components = crate::images::sniff(&data).map(|m| m.components).unwrap_or(3);
                    let img = std::sync::Arc::new(PrintImage {
                        id: next_image_id,
                        mime: info.mime.clone(),
                        px_w: info.width.max(1) as u32,
                        px_h: info.height.max(1) as u32,
                        components: if components == 0 { 3 } else { components },
                        name: info.name.clone(),
                        data,
                    });
                    next_image_id += 1;
                    // Whether a plate can be EMBEDDED is each serializer's own limit, asked at
                    // write time (`PrintImage::pdf_embeddable`) rather than baked into the ops.
                    PrintEntry { info, img }
                })
                .collect();
            images.insert(key, entries);
        }
    }

    let (pw, ph) = spec.page_size.dims();
    // A named print scale is a ratio of ROCK to PAPER, so it depends on how long a stored depth
    // unit actually is. This used to be `1000.0 / scale` unconditionally and then added the
    // resulting metre count straight onto a stored depth: on a foot project a 100 ft interval
    // printed as if it were 100 m, so the sheet was 3.28084x too long, the stated 1:500 was
    // physically about 1:152, and the page breaks landed in the wrong rock. Metre projects are
    // unchanged — `mm_per_stored_unit(Metres)` is exactly 1000.
    let stored_unit = crate::units::project_depth_unit(conn)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let mm_per_unit = crate::units::mm_per_stored_unit(stored_unit) / spec.scale as f64;

    let track_h_first = ph - MARGIN_T - MARGIN_B - HEADER_H_FIRST - TRACK_HEADER_H;
    let track_h_run = ph - MARGIN_T - MARGIN_B - HEADER_H_RUN - TRACK_HEADER_H;
    let units_per_page_first = track_h_first / mm_per_unit;
    let units_per_page_run = track_h_run / mm_per_unit;

    let mut pages = Vec::new();
    let mut d0 = top as f64;
    let bottom = bottom as f64;
    let mut idx = 0;
    while d0 < bottom - 1e-6 {
        let first = idx == 0;
        let span_this = if first { units_per_page_first } else { units_per_page_run };
        let d1 = (d0 + span_this).min(bottom);
        let ops = build_page(
            spec, &header, &curve_frames, &tops, &zones, &completion, &perforations, &core, &aux,
            &arrays, &images, pw, ph, mm_per_unit, first, d0 as f32, d1 as f32, idx,
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
    let ancestry =
        equations::curve_ancestry_disclosures(conn, std::slice::from_ref(&spec.well_id), None)?;
    let ancestry_json = serde_json::to_string(&ancestry).map_err(|error| error.to_string())?;
    let out = pages
        .into_iter()
        .map(|p| -> Result<CompositePage , String> {
            let mut svg= svg_page(&p.ops, pw, ph);
            if !ancestry.is_empty() {
                svg = embed_ancestry_json_in_svg(&svg, &ancestry_json)?;
            }
            Ok(CompositePage {
                svg,
            top_depth: p.top,
            bottom_depth: p.bot,
            index: p.idx,
        })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompositeResult { pages: out, page_width_mm: pw, page_height_mm: ph, scale: spec.scale, well_name ,
        ancestry,
    })
}

/// Renders the composite to a single multi-page PDF (bytes).
pub fn render_composite_pdf(conn: &Connection, spec: &CompositeSpec) -> Result<Vec<u8>, String> {
    let (pages, pw, ph, _) = render_pages(conn, spec)?;
    let ancestry =
        equations::curve_ancestry_disclosures(conn, std::slice::from_ref(&spec.well_id), None)?;
    let streams: Vec<String> = pages.iter().map(|p| pdf_content(&p.ops, pw, ph)).collect();
    let op_pages: Vec<&[DrawOp]> = pages.iter().map(|p| p.ops.as_slice()).collect();
    let mut pdf = assemble_pdf_with_images(&streams, pw, ph, &collect_images(&op_pages));
    if !ancestry.is_empty() {
        let json = serde_json::to_string(&ancestry).map_err(|error| error.to_string())?;
        pdf = embed_ancestry_json_in_pdf(pdf, &json)?;
    }
    Ok(pdf)
}

/// Embeds the complete record inside SVG bytes. The metadata is machine-readable and does not
/// alter the rendered chart; XML escaping keeps arbitrary source notes from breaking the file.
fn embed_json_in_svg(
    svg: &str,
    metadata_id: &str,
    json: &str,
    subject: &str,
) -> Result<String, String> {
    let insert_at = svg
        .find('>')
        .map(|index| index + 1)
        .ok_or_else(|| format!("cannot embed {subject}: SVG has no root element"))?;
    let mut out = svg.to_string();
    out.insert_str(
        insert_at,
        &format!("<metadata id=\"{metadata_id}\">{}</metadata>", esc(json)),
    );
    Ok(out)
}

pub(crate) fn embed_ancestry_json_in_svg(svg: &str, ancestry_json: &str) -> Result<String, String> {
    embed_json_in_svg(
        svg,
        "sandibumi-curve-ancestry-v1",
        ancestry_json,
        "ancestry",
    )
}

pub(crate) fn embed_plot_bindings_json_in_svg(
    svg: &str,
    bindings_json: &str,
) -> Result<String, String> {
    embed_json_in_svg(
        svg,
        "sandibumi-plot-bindings-v1",
        bindings_json,
        "plot bindings",
    )
}

pub(crate) fn embed_chart_render_record_json_in_svg(
    svg: &str,
    chart_record_json: &str,
) -> Result<String, String> {
    embed_json_in_svg(
        svg,
        "sandibumi-chart-render-record-v1",
        chart_record_json,
        "chart render record",
    )
}

pub(crate) fn embed_paper_export_record_json_in_svg(
    svg: &str,
    paper_record_json: &str,
) -> Result<String, String> {
    embed_json_in_svg(
        svg,
        "sandibumi-paper-export-validated-v1",
        paper_record_json,
        "paper export record",
    )
}

/// Embeds the complete record in a PDF comment immediately before EOF. This preserves every xref
/// offset in the already-assembled document and is the same durable marker used by composites.
fn embed_json_in_pdf(
    mut pdf: Vec<u8>,
    marker_name: &str,
    json: &str,
    subject: &str,
) -> Result<Vec<u8>, String> {
    let marker = format!(
        "% {marker_name}:{}\n",
        B64.encode(json.as_bytes())
    );
    let eof = pdf
        .windows(5)
        .rposition(|window| window == b"%%EOF")
        .ok_or_else(|| format!("cannot embed {subject}: generated PDF has no EOF marker"))?;
    pdf.splice(eof..eof, marker.bytes());
    Ok(pdf)
}

pub(crate) fn embed_ancestry_json_in_pdf(
    pdf: Vec<u8>,
    ancestry_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_pdf(
        pdf,
        "SANDIBUMI_CURVE_ANCESTRY_V1_BASE64",
        ancestry_json,
        "ancestry",
    )
}

pub(crate) fn embed_plot_bindings_json_in_pdf(
    pdf: Vec<u8>,
    bindings_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_pdf(
        pdf,
        "SANDIBUMI_PLOT_BINDINGS_V1_BASE64",
        bindings_json,
        "plot bindings",
    )
}

pub(crate) fn embed_chart_render_record_json_in_pdf(
    pdf: Vec<u8>,
    chart_record_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_pdf(
        pdf,
        "SANDIBUMI_CHART_RENDER_RECORD_V1_BASE64",
        chart_record_json,
        "chart render record",
    )
}

pub(crate) fn embed_paper_export_record_json_in_pdf(
    pdf: Vec<u8>,
    paper_record_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_pdf(
        pdf,
        "SANDIBUMI_PAPER_EXPORT_V1_BASE64",
        paper_record_json,
        "paper export record",
    )
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Adds one standards-compliant PNG `tEXt` chunk before IEND. A raster export therefore keeps the
/// complete record even when copied into a slide or detached from the project database.
fn embed_json_in_png(
    png: &[u8],
    keyword: &[u8],
    json: &str,
    subject: &str,
) -> Result<Vec<u8>, String> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if png.get(..8) != Some(SIGNATURE) {
        return Err(format!("cannot embed {subject}: payload is not a PNG"));
    }
    let mut iend = None;
    let mut offset = 8usize;
    while offset + 12 <= png.len() {
        let length = u32::from_be_bytes(png[offset..offset + 4].try_into().unwrap()) as usize;
        let end = offset
            .checked_add(12 + length)
            .filter(|end| *end <= png.len())
            .ok_or_else(|| format!("cannot embed {subject}: malformed PNG chunk length"))?;
        if &png[offset + 4..offset + 8] == b"IEND" {
            iend = Some(offset);
            break;
        }
        offset = end;
    }
    let iend = iend.ok_or_else(|| format!("cannot embed {subject}: PNG has no IEND chunk"))?;
    let mut data = keyword.to_vec();
    data.push(0);
    data.extend_from_slice(json.as_bytes());
    let length: u32 = data
        .len()
        .try_into()
        .map_err(|_| format!("cannot embed {subject}: PNG metadata is too large"))?;
    let mut chunk = Vec::with_capacity(data.len() + 12);
    chunk.extend_from_slice(&length.to_be_bytes());
    chunk.extend_from_slice(b"tEXt");
    chunk.extend_from_slice(&data);
    let mut crc_bytes = b"tEXt".to_vec();
    crc_bytes.extend_from_slice(&data);
    chunk.extend_from_slice(&png_crc32(&crc_bytes).to_be_bytes());

    let mut out = Vec::with_capacity(png.len() + chunk.len());
    out.extend_from_slice(&png[..iend]);
    out.extend_from_slice(&chunk);
    out.extend_from_slice(&png[iend..]);
    Ok(out)
}

pub(crate) fn embed_ancestry_json_in_png(
    png: &[u8],
    ancestry_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_png(png, b"SandiBumiCurveAncestry", ancestry_json, "ancestry")
}

pub(crate) fn embed_plot_bindings_json_in_png(
    png: &[u8],
    bindings_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_png(
        png,
        b"SandiBumiPlotBindings",
        bindings_json,
        "plot bindings",
    )
}

pub(crate) fn embed_chart_render_record_json_in_png(
    png: &[u8],
    chart_record_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_png(
        png,
        b"SandiBumiChartRecord",
        chart_record_json,
        "chart render record",
    )
}

pub(crate) fn embed_paper_export_record_json_in_png(
    png: &[u8],
    paper_record_json: &str,
) -> Result<Vec<u8>, String> {
    embed_json_in_png(
        png,
        b"SandiBumiPaperExport",
        paper_record_json,
        "paper export record",
    )
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
    curve_frames: &HashMap<String, equations::TrackCurveFrame>,
    tops: &[crate::db::TopEntry],
    zones: &[crate::db::ZoneEntry],
    completion: &[crate::db::AuxRow],
    perforations: &[crate::db::AuxRow],
    core: &[(String, f32, f32)],
    aux: &[crate::db::AuxRow],
    arrays: &HashMap<String, Vec<crate::db::ArrayRow>>,
    images: &HashMap<String, Vec<PrintEntry>>,
    pw: f64,
    ph: f64,
    // Paper millimetres per STORED depth unit — never per metre. See `render_pages`.
    mm_per_unit: f64,
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

    let y_of = |d: f32| grid_top + (d - page_top) as f64 * mm_per_unit;

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
    // 22 mm is the minimum readable spacing between labelled depth ticks, so the step it implies
    // is in STORED units and comes out coarser on a foot project — which is right: the ticks are
    // labelled with stored depths.
    let major = nice_step(22.0 / mm_per_unit);
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
        } else if track.kind == crate::layout::TrackKind::Image {
            for st in &track.images {
                let entries =
                    images.get(&st.dataset.trim().to_uppercase()).map(Vec::as_slice).unwrap_or(&[]);
                draw_image_series(&mut ops, st, entries, tx0, tx1, page_top, page_bot, &y_of);
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
                let request = equations::TrackCurveRequest {
                    curve_name: cs.curve_name.clone(),
                    set_name: cs.set_name.clone(),
                    class_curve: cs.fill.as_deref() == Some("blocks"),
                };
                let Some(frame) = curve_frames.get(&equations::track_curve_key(&request)) else { continue ;
                };
                // Crossover shading needs the reference curve's samples AND its own min/max,
                // taken from the same track — compatible scaling is the whole point.
                let xover = cs.fill_to.as_deref().and_then(|to| {
                    let key = to.trim().to_uppercase();
                    let matches: Vec<&crate::layout::CurveStyle> = track
                        .curves
                        .iter()
                        .filter(|other| other.curve_name.trim().to_uppercase() == key)
                        .collect();
                    let source_set = cs.set_name.as_deref().map(str::trim).filter(|set| !set.is_empty());
                    let rs = matches
                        .iter()
                        .copied()
                        .find(|other| {
                            other.set_name.as_deref().map(str::trim).filter(|set| !set.is_empty()) == source_set
                        })
                        .or_else(|| matches.first().copied())?;
                    let request = equations::TrackCurveRequest {
                        curve_name: rs.curve_name.clone(),
                        set_name: rs.set_name.clone(),
                        class_curve: rs.fill.as_deref() == Some("blocks"),
                    };
                    let reference = curve_frames.get(&equations::track_curve_key(&request))?;
                    Some((reference.value.as_slice(), reference.depth.as_slice(), rs.min, rs.max))
                });
                draw_curve(
                    &mut ops,
                    cs,
                    track.scale_type,
                    &frame.value,
                    &frame.depth,
                    xover,
                    tx0,
                    tx1,
                    page_top,
                    page_bot,
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

/// Where the segment between two samples meets a page boundary.
///
/// AUDIT-2026-08-20 finding 41. A sample outside the page used to END the run, so an interval
/// whose two endpoints straddle a page break satisfied NEITHER page's test and was drawn by
/// neither — a hairline at ordinary sampling, and a white stripe a whole sample interval wide on
/// a blocked curve. The run now reaches the boundary from both sides, so consecutive pages meet.
///
/// Interpolated in DRAWN space (x), never in value space: the renderer joins two mapped points
/// with a straight line, so on a LOG axis a value-space interpolation would land somewhere the
/// unclipped line never passed through. Clipping must be invisible — the pixels inside the page
/// have to be the ones the unclipped line would have put there.
///
/// A `step` curve is not interpolated at all; its caller passes the held value, because a step
/// curve has no slope to cut. That is the same statement `hold_to` already makes on the way out.
fn segment_boundary(
    (d_a, x_a): (f32, f64),
    (d_b, x_b): (f32, f64),
    boundary: f32,
    y_of: &dyn Fn(f32) -> f64,
) -> Option<(f64, f64)> {
    let span = d_b - d_a;
    if !span.is_finite() || span == 0.0 {
        return None;
    }
    let t = ((boundary - d_a) / span) as f64;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }
    Some((x_a + t * (x_b - x_a), y_of(boundary)))
}

/// The curve's on-page runs, each already carried to the page boundary at both ends.
///
/// AUDIT-2026-08-20 finding 41. One producer for both the shaded polygon and the line, which
/// differ in what they DO with a run and never in where the runs are — two copies of this would
/// be two chances for the fill and the stroke it outlines to disagree about a page break.
#[allow(clippy::too_many_arguments)]
fn page_clipped_runs(
    vals: &[f32],
    depth: &[f32],
    x_at: &dyn Fn(f32) -> Option<f64>,
    step: bool,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) -> Vec<Vec<(f64, f64)>> {
    let hold_to = |i: usize| -> Option<f64> {
        let d = *depth.get(i + 1)?;
        Some(y_of(d.clamp(page_top, page_bot)))
    };
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut run: Vec<(f64, f64)> = Vec::new();
    // The last sample that HAD a position, on the page or off it — the off-page one is what a
    // boundary point is interpolated from, so it cannot simply be skipped.
    let mut prev: Option<(f32, f64)> = None;
    for (i, &v) in vals.iter().enumerate() {
        let d = depth[i];
        let here = x_at(v);
        let on_page = d >= page_top && d <= page_bot;
        match (here, on_page) {
            (Some(x), true) => {
                // Entering: begin at the boundary the previous sample lies beyond.
                if run.is_empty() {
                    if let Some((pd, px)) = prev {
                        if pd < page_top || pd > page_bot {
                            let edge = if pd < page_top { page_top } else { page_bot };
                            // A step curve holds the PREVIOUS value down to here, so its boundary
                            // point is that value; interpolating would invent a slope the display
                            // does not have.
                            let point = if step {
                                Some((px, y_of(edge)))
                            } else {
                                segment_boundary((pd, px), (d, x), edge, y_of)
                            };
                            if let Some(point) = point {
                                run.push(point);
                            }
                        }
                    }
                }
                run.push((x, y_of(d)));
                if step {
                    if let Some(y) = hold_to(i) {
                        run.push((x, y));
                    }
                }
            }
            (Some(x), false) => {
                // Leaving: finish at the boundary before the run ends. A step curve already
                // reaches it through `hold_to`, which clamps, so only the sloped case adds one.
                if !run.is_empty() && !step {
                    if let Some((pd, px)) = prev {
                        if pd >= page_top && pd <= page_bot {
                            let edge = if d < page_top { page_top } else { page_bot };
                            if let Some(point) = segment_boundary((pd, px), (d, x), edge, y_of) {
                                run.push(point);
                            }
                        }
                    }
                }
                if run.len() >= 2 {
                    out.push(std::mem::take(&mut run));
                }
                run.clear();
            }
            (None, _) => {
                if run.len() >= 2 {
                    out.push(std::mem::take(&mut run));
                }
                run.clear();
            }
        }
        prev = here.map(|x| (d, x));
    }
    if run.len() >= 2 {
        out.push(run);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn draw_curve(
    ops: &mut Vec<DrawOp>,
    cs: &crate::layout::CurveStyle,
    scale: ScaleType,
    vals: &[f32],
    depth: &[f32],
    xover: Option<(&[f32], &[f32], f32, f32)>,
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
            for run in page_clipped_runs(vals, depth, &x_at, step, page_top, page_bot, y_of) {
                let mut pts = Vec::with_capacity(run.len() + 2);
                pts.push((edge_x, run[0].1));
                pts.extend_from_slice(&run);
                pts.push((edge_x, run.last().unwrap().1));
                ops.push(DrawOp::Fill { pts, fill: fill_color.clone(), opacity });
            }
        }
        _ => {}
    }

    // Curve line: contiguous runs, breaking at NaN and carried to the page edge at a page break.
    for run in page_clipped_runs(vals, depth, &x_at, step, page_top, page_bot, y_of) {
        ops.push(DrawOp::Poly { pts: run, stroke: cs.color.clone(), sw: 0.35 });
    }
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
    reference: (&[f32], &[f32], f32, f32),
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    let (rvals, rdepth, rmin, rmax) = reference;
    let tw = tx1 - tx0;
    let x_a = |v: f32| value_frac(v, cs.min, cs.max, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    let x_b = |v: f32| value_frac(v, rmin, rmax, scale).map(|f| tx0 + f.clamp(0.0, 1.0) * tw);
    let left = cs.fill_color.clone().unwrap_or_else(|| cs.color.clone());
    let right = cs.fill_color2.clone().unwrap_or_else(|| left.clone());
    let opacity = cs.fill_opacity.unwrap_or(0.3) as f64;
    let step = cs.draw_style.as_deref() == Some("step");

    let aligned_reference = interpolate_display_series(rdepth, rvals, depth);
    let n = vals.len().min(depth.len()).min(aligned_reference.len());
    for i in 0..n.saturating_sub(1) {
        let (d0, d1) = (depth[i], depth[i + 1]);
        // AUDIT-2026-08-20 finding 41: CLIP the interval to the page rather than drop it. An
        // interval straddling a page break failed both pages' tests and was shaded by neither,
        // leaving an unshaded stripe a whole sample interval wide across the separation — the
        // one place a reader is most likely to read white as "no crossover here".
        if d1 < page_top || d0 > page_bot || !(d1 > d0) {
            continue;
        }
        let (c0, c1) = (d0.max(page_top), d1.min(page_bot));
        let (Some(a_top), Some(b_top)) = (x_a(vals[i]), x_b(aligned_reference[i])) else { continue ;
        };
        // A stepped curve holds its value across the interval, so both edges stay vertical
        // and the pair can never cross inside one interval.
        let (a_bot, b_bot) = if step {
            (a_top, b_top)
        } else {
            let (Some(a1), Some(b1)) = (x_a(vals[i + 1]), x_b(aligned_reference[i + 1])) else { continue ;
            };
            (a1, b1)
        };
        // Interpolated in DRAWN space, like the curve line's own boundary point: a straight edge
        // between two mapped positions is what the renderer draws, so cutting it anywhere else
        // would move the shading off the separation it describes. A step curve is unaffected —
        // both ends carry the same x, so every cut lands on the same vertical.
        let span = (d1 - d0) as f64;
        let at = |t: f64, u: f64, v: f64| u + (v - u) * t;
        let (t0, t1) = (((c0 - d0) as f64) / span, ((c1 - d0) as f64) / span);
        let (a0, b0) = (at(t0, a_top, a_bot), at(t0, b_top, b_bot));
        let (a1, b1) = (at(t1, a_top, a_bot), at(t1, b_top, b_bot));
        let (y0, y1) = (y_of(c0), y_of(c1));
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

/// Display-only linear interpolation mirrored by `LogCanvasRenderer.makeSampler`. It never
/// writes a re-framed curve: the reference is sampled in memory at the styled curve's native
/// depths, returns missing outside its extent, and does not bridge a missing endpoint.
fn interpolate_display_series(reference_depth: &[f32], reference_value: &[f32], target_depth: &[f32]) -> Vec<f32> {
    let n = reference_depth.len().min(reference_value.len());
    if n < 2 {
        return vec![f32::NAN; target_depth.len()];
    }
    let mut cursor = 0usize;
    target_depth
        .iter()
        .map(|target| {
            if *target < reference_depth[0] || *target > reference_depth[n - 1] {
                return f32::NAN;
            }
            if cursor > n - 2 || reference_depth[cursor] > *target {
                cursor = 0;
            }
            while cursor < n - 2 && reference_depth[cursor + 1] < *target {
                cursor += 1;
            }
            let d0 = reference_depth[cursor];
            let d1 = reference_depth[cursor + 1];
            let v0 = reference_value[cursor];
            let v1 = reference_value[cursor + 1];
            if v0.is_nan() || v1.is_nan() {
                return f32::NAN;
            }
            if d1 == d0 {
                v0
            } else {
                v0 + (v1 - v0) * (*target - d0) / (d1 - d0)
            }
        })
        .collect()
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

/// One picture as the print path carries it: its registration and its bytes. Deliberately no
/// "can this be printed" flag — that is a question about a FORMAT, and each back-end answers
/// it for itself when it writes (AUDIT-2026-08-20 finding 42).
pub(crate) struct PrintEntry {
    pub(crate) info: crate::db::ImageInfo,
    pub(crate) img: std::sync::Arc<PrintImage>,
}

/// The millimetre box one picture occupies, computed identically here and in the viewer.
struct ImageBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    /// True when the box is the requested frame and the picture overfills it (fit = cover);
    /// false when the box has already been fitted to the picture's own aspect ratio.
    cover: bool,
}

/// Places one picture in a track.
///
/// Two placements, and the difference is petrophysical rather than cosmetic. **anchor**
/// centres a fixed-size plate on its sample depth — a thin section is cut from one plug and
/// has no thickness, so stretching it over a guessed interval would invent one. **depth**
/// draws the picture across its own `depth_top..depth_base`, which is what a core photograph
/// of a measured run genuinely occupies; without a base depth it falls back to anchor.
///
/// Aspect ratio is never distorted. `contain` fits the whole picture inside the box (so a
/// deep interval in a narrow track simply leaves white space); `cover` fills the box and
/// crops the overhang. There is deliberately no stretch option — a squashed thin section
/// misstates grain shape, which is the one thing the plate is there to show.
fn image_box(style: &crate::layout::ImageStyle, e: &PrintEntry, tx0: f64, tx1: f64, y_of: &dyn Fn(f32) -> f64) -> ImageBox {
    let track_w = tx1 - tx0;
    let box_w = track_w * style.width_frac() as f64;
    let x = match style.align_kind() {
        "left" => tx0,
        "right" => tx1 - box_w,
        _ => tx0 + (track_w - box_w) / 2.0,
    };
    let aspect = e.img.px_h as f64 / e.img.px_w.max(1) as f64;

    let interval = e.info.depth_base.filter(|b| *b > e.info.depth_top);
    if style.mode_kind() == "depth" {
        if let Some(base) = interval {
            let y0 = y_of(e.info.depth_top);
            let y1 = y_of(base);
            let box_h = (y1 - y0).max(0.3);
            if style.fit_kind() == "cover" {
                return ImageBox { x, y: y0, w: box_w, h: box_h, cover: true };
            }
            if style.fit_kind() == "stretch" {
                // The box IS the picture: `cover: false` makes every back-end draw the image
                // exactly into it. See `ImageStyle::fit` for why this is honest here and nowhere
                // else — a depth strip has no aspect ratio of its own to preserve.
                return ImageBox { x, y: y0, w: box_w, h: box_h, cover: false };
            }
            // contain: shrink the wider dimension until the whole picture fits, centred.
            let (mut w, mut h) = (box_w, box_w * aspect);
            if h > box_h {
                h = box_h;
                w = box_h / aspect.max(1e-6);
            }
            return ImageBox { x: x + (box_w - w) / 2.0, y: y0 + (box_h - h) / 2.0, w, h, cover: false };
        }
    }
    // anchor: the picture's own aspect ratio sets the height, centred on the sample depth.
    let h = box_w * aspect;
    let yc = y_of(e.info.depth_base.map_or(e.info.depth_top, |b| (e.info.depth_top + b) / 2.0));
    ImageBox { x, y: yc - h / 2.0, w: box_w, h, cover: false }
}

/// An `image` track's pictures in print. Mirrors `drawImageTracks` in
/// `src/ui/logViewPanel.ts` — the two must agree, and both take their geometry from
/// [`image_box`].
#[allow(clippy::too_many_arguments)]
fn draw_image_series(
    ops: &mut Vec<DrawOp>,
    style: &crate::layout::ImageStyle,
    entries: &[PrintEntry],
    tx0: f64,
    tx1: f64,
    page_top: f32,
    page_bot: f32,
    y_of: &dyn Fn(f32) -> f64,
) {
    let label = style.label.unwrap_or(true);
    let border = style.border.unwrap_or(true);
    let grid_top = y_of(page_top);
    let grid_bot = y_of(page_bot);
    // Where the previous plate ended, so an overlapping one can be SKIPPED rather than
    // nudged: a thin section moved to make room is a thin section attributed to the wrong
    // sand. Zooming in on screen (or printing at a larger scale) reveals the skipped ones.
    let mut last_bottom = f64::NEG_INFINITY;

    for e in entries {
        let sample_depth = e.info.depth_base.map_or(e.info.depth_top, |b| (e.info.depth_top + b) / 2.0);
        if e.info.depth_top > page_bot || e.info.depth_base.unwrap_or(e.info.depth_top) < page_top {
            continue;
        }
        let mut b = image_box(style, e, tx0, tx1, y_of);
        let true_y = y_of(sample_depth);
        let tick = |ops: &mut Vec<DrawOp>| {
            ops.push(DrawOp::Line {
                x1: tx0,
                y1: true_y,
                x2: tx0 + (tx1 - tx0) * 0.15,
                y2: true_y,
                stroke: "#8a7f70".into(),
                sw: 0.25,
            });
        };
        // DEC-090 (Jauhar, 2026-08-20: "Nudge it, but give caller things, like excel"). A plate
        // is still never CLIPPED - half a photograph across a page break reads as a different
        // picture - but one that does not fit is now slid until it does, and a LEADER is drawn
        // back to a tick at its true depth. That is what earns the nudge: the objection to
        // moving a plate was that a thin section moved to make room is attributed to the wrong
        // sand, and a callout pointing at the depth is exactly the answer to it. Before this the
        // plate was dropped from the page and no later page ever reconsidered it, so roughly a
        // twentieth of a petrography delivery was silently absent from the PDF while the screen
        // showed it in full.
        //
        // The box height is PAPER, not rock - an anchored plate is cut from one plug and has no
        // thickness at all - so sliding it costs nothing a reader can misread once the leader
        // states where the plug is.
        let want_y = b.y;
        let lo = grid_top.max(last_bottom + 0.4);
        let hi = grid_bot - b.h;
        if b.h > (grid_bot - grid_top) + 0.01 || lo > hi + 0.01 {
            // Taller than a whole page, or this page is already full: there is nothing to slide
            // it into, so the depth tick stands alone exactly as it used to.
            tick(ops);
            continue;
        }
        b.y = b.y.clamp(lo, hi);
        if (b.y - want_y).abs() > 0.01 {
            // Moved to fit, so it is a CALLOUT now: the tick stays at the true depth and a leader
            // runs down the margin to the plate's own centre. The centre, not the nearest edge -
            // a page-bottom clamp moves a plate by at most half its height, so it usually still
            // overlaps its own depth and an edge-seeking leader would collapse to nothing in the
            // very case it exists for. What moved is where the picture is CENTRED, and that is
            // what the leader has to restate. Drawn in the margin so it never crosses the plate.
            tick(ops);
            let lx = tx0 + (tx1 - tx0) * 0.15;
            ops.push(DrawOp::Line {
                x1: lx,
                y1: true_y,
                x2: lx,
                y2: b.y + b.h / 2.0,
                stroke: "#8a7f70".into(),
                sw: 0.25,
            });
        }
        last_bottom = b.y + b.h;

        // The plate is DRAWN unconditionally. A back-end that cannot carry this format
        // substitutes the named frame as it writes — see `unembeddable_frame` — so the SVG,
        // which inlines any MIME as a data URI, no longer inherits the PDF's JPEG-only limit.
        ops.push(DrawOp::Image {
            x: b.x,
            y: b.y,
            w: b.w,
            h: b.h,
            img: e.img.clone(),
            cover: b.cover,
        });
        if border {
            ops.push(DrawOp::Rect {
                x: b.x,
                y: b.y,
                w: b.w,
                h: b.h,
                fill: None,
                stroke: Some("#5a5148".into()),
                sw: 0.15,
            });
        }
        // Depth leader: the plate is somewhere in the track, its depth is on the left edge.
        ops.push(DrawOp::Line {
            x1: tx0,
            y1: y_of(sample_depth),
            x2: b.x,
            y2: y_of(sample_depth),
            stroke: "#8a7f70".into(),
            sw: 0.2,
        });
        if label && b.y - 1.0 > grid_top {
            ops.push(DrawOp::Text {
                x: b.x,
                y: b.y - 0.8,
                size: 2.2,
                anchor: Anchor::Start,
                color: "#4a4038".into(),
                bold: false,
                s: e.info.name.clone(),
            });
        }
    }
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
    use crate::distribution::{bin_by_depth, box_stats, default_bin_height, histogram};
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
            // The default is a property of the SERIES, never of the page — see
            // distribution::default_bin_height. `depth` here is the whole series, not the
            // page's slice of it, which is what lets both renderers reach the same answer.
            let bin = ps.bin.filter(|b| *b > 0.0).unwrap_or_else(|| default_bin_height(&depth));
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
                let Some(st) = box_stats(&vals, blo, bhi, ps.whisker_rule()) else { continue ;
                };
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
                // The individual plugs behind the summary. The field's own doc says it: on a
                // sparse cored interval seeing them is often the point - which makes it a
                // deliberate choice the reader made, and the print was silently dropping it
                // from the deliverable while the screen honoured it. Ticks sit ABOVE the box
                // so they never obscure the median, matching the viewer.
                if ps.show_samples.unwrap_or(false) {
                    let tick = box_h / 3.0;
                    let top = mid - box_h / 2.0;
                    for v in &vals {
                        let Some(x) = x_at(*v) else { continue };
                        ops.push(DrawOp::Fill {
                            pts: vec![
                                (x - 0.15, top - tick),
                                (x + 0.15, top - tick),
                                (x + 0.15, top),
                                (x - 0.15, top),
                            ],
                            fill: ps.color.clone(),
                            opacity: 0.55,
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
            let traces = even_indices(width, as_.traces.unwrap_or(40) as usize);
            let mut source = Vec::with_capacity(traces.len() * visible.len());
            for r_idx in &traces {
                for row in &visible {
                    source.push(row.samples.get(*r_idx).copied().unwrap_or(f32::NAN));
                }
            }
            let policy = apply_plot_channel_policy(
                &source,
                PlotChannelPolicy::ArrayWaveform {
                    log_axis: scale == ScaleType::Log,
                    display: DisplayRange { low: as_.min, high: as_.max },
                },
            );
            let mut display_index = 0usize;
            for _r_idx in traces {
                let mut run: Vec<(f64, f64)> = Vec::new();
                for row in &visible {
                    let x = policy
                        .included
                        .get(display_index)
                        .copied()
                        .unwrap_or(false)
                        .then(|| policy.values[display_index])
                        .and_then(x_at);
                    display_index += 1;
                    match x {
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
            ops.push(DrawOp::Text {
                x: tx0 + 0.8,
                y: y_of(page_top) + 2.5,
                size: 2.1,
                anchor: Anchor::Start,
                color: "#555555".into(),
                bold: false,
                s: format!(
                    "waveform clamped={} · non-finite excluded={} · log-domain excluded={}",
                    policy.clamped, policy.non_finite_excluded, policy.log_domain_excluded
                ),
            });
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

/// Neutral grey for a REJECTED sample (SB-MLA-021), deliberately outside the qualitative palette.
/// Keep in sync with `REJECT_COLOR` in src/ui/plotCanvas.ts.
const FACIES_REJECT_COLOR: &str = "#9aa0a6";

/// The print path's class colour, reachable from `ml.rs`'s SB-MLA-021 test. Exposed rather than
/// duplicating the palette in the test, which would pin a copy instead of the thing that draws.
#[cfg(test)]
pub(crate) fn facies_color_for_test(class: i64) -> &'static str {
    facies_color(class)
}

fn facies_color(class: i64) -> &'static str {
    // SB-MLA-021. A negative class is one the algorithm REJECTED, not one of the clusters — and the
    // wrap below would otherwise fold it back onto a real cluster's colour, printing an outlier as a
    // legitimate facies. Any negative, not just CLUSTER_REJECT: a code this renderer does not
    // recognise must not be painted as rock it is not.
    if class < 0 {
        return FACIES_REJECT_COLOR;
    }
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
    // A class-blocks curve has no value axis - the blocks always span the track - so it must
    // neither print one nor stand in for a real curve's scale. The screen gives every curve its
    // own scale line and writes "class blocks" on that one; the print header has a single line,
    // so it takes the first curve that HAS a scale and names the display only when none does.
    let scale_curve = track.curves.iter().find(|c| c.fill.as_deref() != Some("blocks"));
    if scale_curve.is_none() && !track.curves.is_empty() {
        ops.push(DrawOp::Text {
            x: (tx0 + tx1) / 2.0,
            y: bot - 1.2,
            size: 2.4,
            anchor: Anchor::Middle,
            color: "#555555".into(),
            bold: false,
            s: "class blocks".into(),
        });
    }
    if let Some(c) = scale_curve {
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
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{pw}mm" height="{ph}mm" viewBox="0 0 {pw} {ph}" font-family="Helvetica, Arial, sans-serif">"##,
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
            DrawOp::Image { x, y, w, h, img, cover } => {
                // Self-contained: the pixels ride in the file as a data URI, so an SVG handed
                // to a client opens with its plates intact rather than as broken links.
                // `slice` is SVG's own cover — it fills the box and clips, no clip path
                // needed; `none` is exact because the caller already fitted the box to the
                // picture's aspect ratio.
                let par = if *cover { "xMidYMid slice" } else { "none" };
                let b64 = B64.encode(&img.data);
                let _ = write!(
                    s,
                    r##"<image x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" preserveAspectRatio="{par}" href="data:{0};base64,{b64}" xlink:href="data:{0};base64,{b64}"/>"##,
                    esc(&img.mime),
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

/// What a back-end draws in place of a picture it cannot carry: a framed box naming the plate.
///
/// Never a silent gap — a client deliverable has to be checkable against the delivery list, and
/// an empty space reads as "no plate at this depth" rather than as "your PDF writer cannot take
/// a PNG". Kept as draw-ops rather than as content-stream text so it goes through the very same
/// Rect and Text emitters every other frame does.
fn unembeddable_frame(x: f64, y: f64, w: f64, h: f64, name: &str) -> [DrawOp; 2] {
    [
        DrawOp::Rect {
            x,
            y,
            w,
            h,
            fill: Some("#f2efe9".into()),
            stroke: Some("#b0413e".into()),
            sw: 0.25,
        },
        DrawOp::Text {
            x: x + w / 2.0,
            y: y + h / 2.0,
            size: 2.2,
            anchor: Anchor::Middle,
            color: "#b0413e".into(),
            bold: false,
            s: format!("{name} — not embeddable"),
        },
    ]
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
            DrawOp::Image { x, y, w, h, img, .. } if !img.pdf_embeddable() => {
                // This writer encodes JPEG and nothing else, so the plate becomes a named
                // frame HERE — the SVG back-end, which has no such limit, still gets the
                // picture. Recursed through the same emitters so the frame cannot drift from
                // the ordinary Rect and Text it is made of.
                s.push_str(&pdf_content(&unembeddable_frame(*x, *y, *w, *h, &img.name), _pw, ph));
            }
            DrawOp::Image { x, y, w, h, img, cover } => {
                // An image XObject is drawn into the UNIT square, so the `cm` matrix carries
                // the whole placement: width, height, and the lower-left corner in points.
                let (bx, by) = (tx(*x), ty(*y + *h));
                let (bw, bh) = (w * PT_PER_MM, h * PT_PER_MM);
                s.push_str("q\n");
                let (dw, dh, dx, dy) = if *cover {
                    // Clip to the box, then overscale the picture until it covers, centred —
                    // the same crop SVG's `slice` produces, so the two exports agree.
                    let _ = write!(s, "{bx:.2} {by:.2} {bw:.2} {bh:.2} re W n\n");
                    let aspect = img.px_h as f64 / img.px_w.max(1) as f64;
                    let (mut dw, mut dh) = (bw, bw * aspect);
                    if dh < bh {
                        dh = bh;
                        dw = bh / aspect.max(1e-6);
                    }
                    (dw, dh, bx + (bw - dw) / 2.0, by + (bh - dh) / 2.0)
                } else {
                    (bw, bh, bx, by)
                };
                let _ = write!(s, "{dw:.2} 0 0 {dh:.2} {dx:.2} {dy:.2} cm\n/Im{} Do\nQ\n", img.id);
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

/// Gathers every distinct picture referenced by a render's pages, ordered by the id the
/// content streams already wrote (`/Im{id}`), so the resources dictionary can be built from
/// the draw-ops alone rather than threaded through every caller.
pub(crate) fn collect_images(pages: &[&[DrawOp]]) -> Vec<std::sync::Arc<PrintImage>> {
    let mut seen: HashMap<usize, std::sync::Arc<PrintImage>> = HashMap::new();
    for ops in pages {
        for op in ops.iter() {
            if let DrawOp::Image { img, .. } = op {
                // Only what the content stream actually referenced: `pdf_content` draws a
                // named frame for anything else, so an XObject for it would be an unreferenced
                // object carrying the whole picture's bytes.
                if img.pdf_embeddable() {
                    seen.entry(img.id).or_insert_with(|| img.clone());
                }
            }
        }
    }
    let mut out: Vec<_> = seen.into_values().collect();
    out.sort_by_key(|i| i.id);
    out
}

/// Assembles per-page content streams into a single multi-page PDF document.
pub(crate) fn assemble_pdf(streams: &[String], pw: f64, ph: f64) -> Vec<u8> {
    assemble_pdf_with_images(streams, pw, ph, &[])
}

/// As [`assemble_pdf`], plus image XObjects for any picture the pages draw.
///
/// The object bodies are BYTES rather than a String: a JPEG stream is not valid UTF-8, and
/// re-encoding it (base64, hex) would inflate a photographed core by a third for nothing.
/// JPEG bytes go in untouched under `/DCTDecode` — the PDF reader runs the same decoder the
/// camera's file already expects, so nothing is recompressed and nothing is lost.
pub(crate) fn assemble_pdf_with_images(
    streams: &[String],
    pw: f64,
    ph: f64,
    images: &[std::sync::Arc<PrintImage>],
) -> Vec<u8> {
    let wp = pw * PT_PER_MM;
    let hp = ph * PT_PER_MM;
    let n = streams.len();

    // Object ids: 1 catalog, 2 pages, 3 F1, 4 F2, then one per image, then per page
    // (content, page). Images come before the pages so a page can reference them by id.
    let n_fixed = 4;
    let mut objects: Vec<Vec<u8>> = Vec::new(); // body of each object, in id order

    // 1: Catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // 2: Pages (kids filled below)
    let first_page_obj = n_fixed + images.len() + 1;
    let mut kids = String::new();
    for i in 0..n {
        let page_id = first_page_obj + 2 * i + 1; // content then page
        let _ = write!(kids, "{page_id} 0 R ");
    }
    objects.push(format!("<< /Type /Pages /Kids [ {}] /Count {} >>", kids.trim_end(), n).into_bytes());

    // 3, 4: fonts
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());

    // Image XObjects, in id order, so object id = n_fixed + 1 + position.
    let mut xobjects = String::new();
    for (pos, img) in images.iter().enumerate() {
        let obj_id = n_fixed + 1 + pos;
        let _ = write!(xobjects, "/Im{} {obj_id} 0 R ", img.id);
        // Adobe's CMYK JPEGs store inverted values; the Decode array puts them back. Grey
        // and RGB need nothing. Anything not JPEG never reaches here (see draw_image_series).
        let (space, decode) = match img.components {
            1 => ("/DeviceGray", ""),
            4 => ("/DeviceCMYK", " /Decode [1 0 1 0 1 0 1 0]"),
            _ => ("/DeviceRGB", ""),
        };
        let head = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace {space} \
             /BitsPerComponent 8 /Filter /DCTDecode{decode} /Length {} >>\nstream\n",
            img.px_w,
            img.px_h,
            img.data.len(),
        );
        let mut body = head.into_bytes();
        body.extend_from_slice(&img.data);
        body.extend_from_slice(b"\nendstream");
        objects.push(body);
    }
    let resources = if xobjects.is_empty() {
        "/Resources << /Font << /F1 3 0 R /F2 4 0 R >> >>".to_string()
    } else {
        format!("/Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << {} >> >>", xobjects.trim_end())
    };

    // Per page: content stream object then page object.
    for (i, stream) in streams.iter().enumerate() {
        let content_id = first_page_obj + 2 * i;
        objects.push(format!("<< /Length {} >>\nstream\n{stream}endstream", stream.len()).into_bytes());
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {wp:.2} {hp:.2}] \
                 {resources} /Contents {content_id} 0 R >>",
            )
            .into_bytes(),
        );
    }

    // Serialize with a cross-reference table.
    let mut out: Vec<u8> = Vec::with_capacity(64 * 1024);
    out.extend_from_slice("%PDF-1.7\n%\u{00e2}\u{00e3}\u{00cf}\u{00d3}\n".as_bytes());
    let mut offsets = vec![0usize; objects.len() + 1];
    for (i, body) in objects.iter().enumerate() {
        offsets[i + 1] = out.len();
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_pos = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for i in 1..=objects.len() {
        out.extend_from_slice(format!("{:010} 00000 n \n", offsets[i]).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n",
            objects.len() + 1,
        )
        .as_bytes(),
    );
    out
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
        db::insert_well(conn, wid, "SANDI-COMPOSITE", Some("Sandi Field"), Some(1800.0), Some(25.0)).unwrap();
        let w = wid.to_string();
        let n = 400;
        let depths: Vec<f32> = (0..n).map(|i| 1000.0 + i as f32 * 0.5).collect();
        let gr: Vec<f32> = (0..n).map(|i| 40.0 + 60.0 * ((i as f32 / 20.0).sin() * 0.5 + 0.5)).collect();
        let res: Vec<f32> = (0..n).map(|i| 1.0 + (i % 50) as f32).collect();
        let nphi: Vec<f32> = vec![0.25; n];
        let rhob: Vec<f32> = vec![2.4; n];
        db::insert_standard_curves(conn, wid, depths, gr, res, nphi, rhob, vec![f32::NAN; n], vec![f32::NAN; n]).unwrap();
        db::upsert_top(conn, &w, "Top Reservoir", 1050.0, Some("#b0413e")).unwrap();
        db::upsert_md_zone(conn, &w, "ZoneA", 1040.0, 1120.0).unwrap();
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

    /// A set-qualified layout is a source-identity contract in every renderer, not only the
    /// interactive WebGPU view. Give the imported GR a disjoint depth interval so falling back
    /// to the standard GR cannot accidentally satisfy the requested print window.
    #[test]
    fn composite_renders_explicit_set_on_its_native_depth_grid() {
        let conn = Connection::open_in_memory().unwrap();
        let well = seed_well(&conn);
        let curve_id = db::upsert_curve_meta(
            &conn,
            &well,
            "WIRE_1",
            "GR",
            Some("GAPI"),
            Some("GR"),
            Some("LAS import"),
            None,
        )
        .unwrap();
        db::insert_curve_samples(
            &conn,
            &curve_id,
            &[2000.0, 2000.5, 2001.0, 2001.5],
            &[80.0, 85.0, 90.0, 95.0],
        )
        .unwrap();
        let layout: crate::layout::Layout = serde_json::from_value(serde_json::json!({
            "name": "Native set print",
            "tracks": [{
                "title": "GR",
                "width_weight": 1.0,
                "scale_type": "linear",
                "curves": [{
                    "curve_name": "GR",
                    "set_name": "WIRE_1",
                    "color": "#2e7d32",
                    "min": 0.0,
                    "max": 150.0
                }]
            }]
        }))
        .unwrap();
        let spec = CompositeSpec {
            well_id: well,
            layout,
            depth_top: Some(2000.0),
            depth_bottom: Some(2001.5),
            scale: 500,
            page_size: PageSize::A4,
        };

        let (pages, _, _, _) = render_pages(&conn, &spec)
            .expect("set-qualified composite must use the imported set's native interval");
        assert!(
            pages
                .iter()
                .flat_map(|page| page.ops.iter())
                .any(|op| matches!(op, DrawOp::Poly { .. })),
            "the explicit imported curve is rendered"
        );
    }

    /// T-AUX-07, the compatibility half. `layouts_saved_before_crossover_still_load` above covers
    /// the CurveStyle fields; this covers the field the diagram feature added to the TRACK, which
    /// is the one that would break every layout the user has ever saved.
    ///
    /// A layout is stored as JSON in the project. Every layout written before well-diagram tracks
    /// existed has no `kind` key at all, so without `#[serde(default)]` on `Track::kind` the whole
    /// layout fails to deserialize — and a layout that will not load takes the user's track
    /// widths, scales, colours and curve choices with it. Defaulting to `Curves` is what makes an
    /// old layout open as exactly what it was.
    #[test]
    fn a_layout_saved_before_well_diagram_tracks_opens_as_curves() {
        // Deliberately written as a PRE-feature layout would have been stored: no `kind`, no
        // `points`, no `arrays`, no `images` — only the keys that existed at the time.
        let old: crate::layout::Layout = serde_json::from_value(serde_json::json!({
            "name": "My Field Layout",
            "tracks": [{
                "title": "GR",
                "width_weight": 1.0,
                "scale_type": "linear",
                "curves": [{ "curve_name": "GR", "color": "#2e7d32", "min": 0.0, "max": 150.0 }]
            }]
        }))
        .expect("a layout saved before well-diagram tracks must still deserialize");

        assert_eq!(old.name, "My Field Layout");
        let t = &old.tracks[0];
        assert_eq!(t.kind, crate::layout::TrackKind::Curves, "an absent kind means a curve track");
        assert_eq!(t.curves[0].curve_name, "GR");
        // The three later collections default to empty rather than to a missing-field error.
        assert!(t.points.is_empty() && t.arrays.is_empty() && t.images.is_empty());
    }

    /// T-AUX-07, the artwork half. The well diagram is the one track whose content is not a
    /// curve, so nothing else in the render path exercises it — and it is read for DEPTH: an
    /// engineer checks that a perforation sits in the sand and above the contact, and that the
    /// shoe is where the completion report says it is. A casing shoe drawn at the wrong depth is
    /// a picture that argues for perforating the wrong interval.
    #[test]
    fn a_well_diagram_draws_its_strings_shoes_and_perforations_at_the_declared_depths() {
        // A surface string that ends on this page, and a liner that runs past its base. The
        // depths are the completion report's; the y map is 1:1 so a depth reads as a y.
        let casing = vec![
            crate::db::AuxRow {
                dataset: "COMPLETION".into(), depth_top: 1000.0, depth_base: Some(1100.0),
                item: "SURFACE".into(), value_num: Some(9.625), value_text: None,
            },
            crate::db::AuxRow {
                dataset: "COMPLETION".into(), depth_top: 1080.0, depth_base: Some(1190.0),
                item: "LINER".into(), value_num: Some(7.0), value_text: None,
            },
        ];
        let perfs = vec![crate::db::AuxRow {
            dataset: "PERFORATION".into(), depth_top: 1120.0, depth_base: Some(1125.0),
            item: "PERF".into(), value_num: None, value_text: None,
        }];

        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_well_diagram(&mut ops, &casing, &perfs, 0.0, 20.0, 1000.0, 1200.0, &y);
        let cx = 10.0;

        // Casing walls: a symmetric PAIR per string, spanning exactly top to base.
        let walls: Vec<(f64, f64, f64)> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Line { x1, y1, x2, y2, stroke, .. }
                    if stroke == "#5a5a5a" && (x1 - x2).abs() < 1e-9 =>
                {
                    Some((*x1, *y1, *y2))
                }
                _ => None,
            })
            .collect();
        assert_eq!(walls.len(), 4, "two strings, two walls each");
        let surface: Vec<_> = walls.iter().filter(|w| (w.1 - 1000.0).abs() < 1e-9).collect();
        assert_eq!(surface.len(), 2, "the surface string draws a symmetric pair");
        assert!((surface[0].2 - 1100.0).abs() < 1e-9, "and stops at its declared shoe depth");
        assert!(
            ((surface[0].0 - cx) + (surface[1].0 - cx)).abs() < 1e-9,
            "the pair must straddle the track centre"
        );

        // The wider string draws wider. This is the only thing on the page that says which
        // casing is which, so an OD that did not affect the width would make the picture mute.
        let half = |top: f64| -> f64 {
            walls.iter().find(|w| (w.1 - top).abs() < 1e-9).map(|w| (w.0 - cx).abs()).unwrap()
        };
        assert!(half(1000.0) > half(1080.0), "9.625 in must draw wider than 7 in");

        // Shoe markers: a filled square at each wall, at each string's base.
        let shoes: Vec<f64> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Rect { y, fill: Some(f), .. } if f == "#333333" => Some(*y),
                _ => None,
            })
            .collect();
        assert_eq!(shoes.len(), 4, "two shoes, two markers each");
        assert!(shoes.iter().filter(|y| (**y - (1100.0 - 1.2)).abs() < 1e-9).count() == 2);
        assert!(shoes.iter().filter(|y| (**y - (1190.0 - 1.2)).abs() < 1e-9).count() == 2);

        // OD labels at each string's top, in inches.
        let labels: Vec<(f64, String)> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Text { y, s, .. } => Some((*y, s.clone())),
                _ => None,
            })
            .collect();
        assert!(labels.iter().any(|(_, s)| s == "9.625\""), "got {labels:?}");
        assert!(labels.iter().any(|(_, s)| s == "7\""));
        let surf_label = labels.iter().find(|(_, s)| s == "9.625\"").unwrap();
        assert!((surf_label.0 - (1000.0 - 0.8)).abs() < 1e-9, "the label sits at the string top");

        // Perforation ticks stay inside the perforated interval — the assertion an engineer is
        // really making when they look at this track.
        let ticks: Vec<f64> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Line { y1, stroke, .. } if stroke == "#c0392b" => Some(*y1),
                _ => None,
            })
            .collect();
        assert!(!ticks.is_empty(), "a perforated interval must draw ticks");
        for t in &ticks {
            assert!(
                (1120.0..=1125.0).contains(t),
                "a perf tick at {t} m is outside the perforated interval 1120-1125"
            );
        }
    }

    /// The joint: a well-diagram track in a saved layout actually reaches the rendered pages, on
    /// EVERY page rather than only the first. A string that runs the length of the well has to be
    /// redrawn per page — the diagram is not a header block — and a track that quietly rendered
    /// empty after page 1 would look like the casing stopped at the page break.
    #[test]
    fn a_well_diagram_track_is_redrawn_on_every_composite_page() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn); // logged 1000.0 .. 1199.5
        crate::db::insert_aux_data(
            &conn,
            &w,
            "COMPLETION",
            "RAW",
            None,
            &[crate::db::AuxRow {
                dataset: "COMPLETION".into(), depth_top: 1000.0, depth_base: Some(1199.0),
                item: "SURFACE".into(), value_num: Some(9.625), value_text: None,
            }],
        )
        .unwrap();

        let mut spec = full_spec(w, 500, PageSize::A4);
        spec.layout.tracks.push(serde_json::from_value(serde_json::json!({
            "title": "Well", "width_weight": 0.6, "scale_type": "linear",
            "kind": "well_diagram", "curves": []
        }))
        .unwrap());

        let (pages, _pw, _ph, _n) = render_pages(&conn, &spec).unwrap();
        assert!(pages.len() >= 2, "need a multi-page render to prove the diagram repeats");
        for p in &pages {
            let walls = p
                .ops
                .iter()
                .filter(|o| matches!(o, DrawOp::Line { stroke, x1, x2, .. } if stroke == "#5a5a5a" && (x1 - x2).abs() < 1e-9))
                .count();
            assert_eq!(walls, 2, "page {} lost the casing string", p.idx);
        }
    }

    /// `show_samples` reaches the deliverable, not just the screen.
    ///
    /// A box glyph summarises a cored interval; the option draws the individual plugs behind it,
    /// and the field's own doc says why - "on a sparse interval seeing the plugs behind the
    /// summary is often the point". The screen honoured it and the print did not, so a choice
    /// the reader deliberately made was dropped from the PDF with nothing to say so. Pinned from
    /// both sides: OFF must stay clean, or "always draw them" would pass the ON half.
    #[test]
    fn the_plugs_behind_a_box_glyph_reach_the_print_when_the_reader_asked_for_them() {
        let style = |show: bool| -> crate::layout::PointStyle {
            serde_json::from_value(serde_json::json!({
                "source": "core", "item": "CPOR", "color": "#c67139",
                "min": 0.0, "max": 0.4, "display": "box", "show_samples": show
            }))
            .unwrap()
        };
        // Five plugs over one metre, so they land in a single depth bin and get one box.
        let core: Vec<(String, f32, f32)> = [0.10f32, 0.14, 0.18, 0.22, 0.26]
            .iter()
            .enumerate()
            .map(|(i, v)| ("CPOR".to_string(), 100.0 + i as f32 * 0.1, *v))
            .collect();
        let render = |show: bool| -> Vec<DrawOp> {
            let mut ops = Vec::new();
            draw_point_series(
                &mut ops, &style(show), ScaleType::Linear, &core, &[], 0.0, 30.0, 90.0, 110.0,
                &|d: f32| d as f64,
            );
            ops
        };
        // The tick is the only translucent fill the box path emits; the box body is 0.5.
        let ticks = |ops: &[DrawOp]| -> usize {
            ops.iter()
                .filter(|o| matches!(o, DrawOp::Fill { opacity, .. } if (*opacity - 0.55).abs() < 1e-9))
                .count()
        };

        let on = render(true);
        assert!(ticks(&on) >= 5, "every plug must print, got {} tick(s)", ticks(&on));

        let off = render(false);
        assert_eq!(ticks(&off), 0, "and OFF must print none - the option is the reader's");
        assert!(
            off.len() > 3,
            "the box itself still prints either way, or this test proves nothing: {}",
            off.len()
        );
    }

    /// A class-blocks track prints no value axis, and never borrows one curve's scale for another.
    ///
    /// CLAUDE.md already states it - "min/max on a blocks curve is ignored (header shows
    /// `class blocks` instead of editable scale)" - and the screen obeys it per curve row. The
    /// print header carries ONE scale line, so it used to take `curves.first()` whatever that
    /// was: a FACIES track printed `0 ... 8` as though the colours sat on a value axis, and a
    /// track that merely LISTED the blocks curve first printed those meaningless limits instead
    /// of the real curve's. Pinned from both sides so neither half can regress alone.
    #[test]
    fn a_class_blocks_track_prints_no_value_scale_and_never_borrows_a_real_curves_one() {
        let track = |curves: serde_json::Value| -> crate::layout::Track {
            serde_json::from_value(serde_json::json!({
                "title": "Facies", "width_weight": 1.0, "scale_type": "linear", "curves": curves
            }))
            .unwrap()
        };
        let header = |t: &crate::layout::Track| -> Vec<String> {
            let mut ops = Vec::new();
            draw_track_header(&mut ops, t, 0.0, 30.0, 0.0, 10.0);
            ops.iter()
                .filter_map(|o| match o {
                    DrawOp::Text { s, .. } => Some(s.clone()),
                    _ => None,
                })
                .collect()
        };
        let blocks = serde_json::json!({
            "curve_name": "FACIES", "color": "#000000", "min": 0.0, "max": 8.0, "fill": "blocks"
        });
        let gr = serde_json::json!({
            "curve_name": "GR", "color": "#111111", "min": 0.0, "max": 150.0
        });

        // Blocks alone: the display is named, and 8 never appears as a limit.
        let only = header(&track(serde_json::json!([blocks])));
        assert!(only.iter().any(|s| s == "class blocks"), "must name the display: {only:?}");
        assert!(
            !only.iter().any(|s| s.contains('8')),
            "a class index is not a value axis, so its max must not print: {only:?}"
        );

        // Listed FIRST beside a real curve: the real curve's scale is the one that prints.
        let pair = header(&track(serde_json::json!([blocks, gr])));
        assert!(
            pair.iter().any(|s| s.contains("150")),
            "the real curve owns the scale line: {pair:?}"
        );
        assert!(
            !pair.iter().any(|s| s == "class blocks"),
            "and with a real scale present there is nothing to name: {pair:?}"
        );
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
            &mut ops, &cs, ScaleType::Linear, &vals, &depth, Some((&refv, &depth, 0.0, 1.0)),
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
    fn composite_crossover_interpolates_reference_without_reframing_storage() {
        let reference_depth = [100.0f32, 102.0];
        let reference_value = [0.0f32, 1.0];
        let target_depth = [99.0f32, 100.0, 101.0, 102.0, 103.0];

        let sampled = interpolate_display_series(&reference_depth, &reference_value, &target_depth);

        assert!(sampled[0].is_nan() && sampled[4].is_nan(), "display sampling never extrapolates");
        assert_eq!(&sampled[1..4], &[0.0, 0.5, 1.0]);
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

    /// AUDIT-2026-08-20 finding 41. A composite page is a window on the well, not a filter on
    /// its samples: the interval from the last sample above the break to the first sample below
    /// it is real rock, and it used to satisfy neither page's test and be drawn by neither. At
    /// ordinary sampling that is a hairline; on a metre-blocked curve at 1:200 it is a 5 mm
    /// white stripe across the page join, which reads as missing data rather than as a page edge.
    ///
    /// Pinned from BOTH sides, because clipping and not clipping at all both close the gap: the
    /// run must REACH each boundary, and it must stop there.
    #[test]
    fn an_interval_straddling_a_page_break_is_drawn_to_the_boundary_and_never_past_it() {
        let (page_top, page_bot) = (100.0f32, 200.0f32);
        let y = |d: f32| d as f64;
        // One sample above the page, two inside it, one below — so both breaks are straddled.
        let vals = [0.0f32, 0.25, 0.75, 1.0];
        let depth = [80.0f32, 120.0, 180.0, 220.0];

        let mut ops = Vec::new();
        draw_curve(
            &mut ops, &saved_style("VSH"), ScaleType::Linear, &vals, &depth, None,
            0.0, 10.0, page_top, page_bot, &y,
        );
        let pts = line_pts(&ops);
        // Halfway between 80 and 120 in depth is halfway between their two x positions, so the
        // line enters the page carrying the value the log actually has at the page's first metre.
        assert_eq!(pts.first().copied(), Some((1.25, 100.0)), "the run reaches the page top");
        assert_eq!(pts.last().copied(), Some((8.75, 200.0)), "and the page bottom");
        assert!(
            pts.iter().all(|(_, y)| (100.0..=200.0).contains(y)),
            "and stops at both, rather than printing a neighbouring page's rock: {pts:?}"
        );

        // Crossover shading is a second implementation of the same window and used to drop a
        // straddling interval outright. Here ONE interval spans the whole page, so dropping it
        // leaves the entire separation unshaded.
        let mut cs = saved_style("NPHI");
        cs.fill = Some("curve".into());
        cs.fill_to = Some("RHOB".into());
        cs.fill_color = Some("#111111".into());
        let span = [80.0f32, 220.0];
        let mut ops = Vec::new();
        draw_curve(
            &mut ops, &cs, ScaleType::Linear, &[0.2f32, 0.2], &span,
            Some((&[0.8f32, 0.8], &span, 0.0, 1.0)), 0.0, 10.0, page_top, page_bot, &y,
        );
        assert_eq!(
            fills(&ops),
            vec![("#111111".to_string(), vec![(2.0, 100.0), (8.0, 100.0), (8.0, 200.0), (2.0, 200.0)])],
            "the separation is shaded across the whole page and no further"
        );
    }

    fn point_style(json: serde_json::Value) -> crate::layout::PointStyle {
        serde_json::from_value(json).unwrap()
    }

    // --- depth-registered images -------------------------------------------------------

    fn image_style(json: serde_json::Value) -> crate::layout::ImageStyle {
        serde_json::from_value(json).unwrap()
    }

    /// A picture 200 px wide by 100 px tall (aspect 0.5) at the given registration. The MIME is
    /// the caller's, not a constant: whether a back-end can carry a plate is a question about
    /// its FORMAT, and a fixture that is always a JPEG can never ask it (finding 42 — the SVG
    /// test only ever used a JPEG, so nothing noticed the SVG refusing a PNG it could show).
    fn plate(id: usize, name: &str, top: f32, base: Option<f32>, mime: &str) -> PrintEntry {
        PrintEntry {
            info: crate::db::ImageInfo {
                image_id: format!("id-{id}"),
                dataset: "THIN SECTION".into(),
                set_name: "RAW".into(),
                depth_top: top,
                depth_base: base,
                name: name.into(),
                caption: None,
                mime: mime.into(),
                width: 200,
                height: 100,
                src_width: Some(4000),
                src_height: Some(2000),
                source_path: None,
                printable: mime == "image/jpeg",
                bytes: 6,
                ..Default::default()
            },
            img: std::sync::Arc::new(PrintImage {
                id,
                mime: mime.into(),
                data: vec![0xFF, 0xD8, 1, 2, 3, 0xD9],
                px_w: 200,
                px_h: 100,
                components: 3,
                name: name.into(),
            }),
        }
    }

    const JPEG: &str = "image/jpeg";

    fn images(ops: &[DrawOp]) -> Vec<(f64, f64, f64, f64, bool)> {
        ops.iter()
            .filter_map(|o| match o {
                DrawOp::Image { x, y, w, h, cover, .. } => Some((*x, *y, *w, *h, *cover)),
                _ => None,
            })
            .collect()
    }

    /// DEC-090: a plate near a page break is printed as a CALLOUT, never dropped.
    ///
    /// An anchored plate is centred on its depth, so one sitting within half its own height of
    /// the page bottom did not fit the grid - and the print degraded it to a bare tick that no
    /// later page ever reconsidered. Roughly a twentieth of a petrography delivery was absent
    /// from the PDF while the screen showed it in full. Jauhar ruled: "Nudge it, but give
    /// caller things, like excel".
    ///
    /// Pinned from both sides. A plate that FITS must be neither moved nor annotated, or
    /// "always nudge, always draw a leader" would pass the first half and put a callout on
    /// every plate in the deliverable.
    #[test]
    fn a_plate_that_will_not_fit_the_page_is_called_out_rather_than_dropped() {
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let y = |d: f32| d as f64;
        // Track 0..40 and size 0.5 gives a 20 x 10 box; the page runs 1000..1020.
        let render = |depth: f32| -> Vec<DrawOp> {
            let mut ops = Vec::new();
            draw_image_series(
                &mut ops, &st, &[plate(0, "TS-1", depth, None, JPEG)], 0.0, 40.0, 1000.0, 1020.0, &y,
            );
            ops
        };
        // The leader lives in the margin at 15% of the track, and is the only VERTICAL line
        // this path draws - the depth tick is horizontal.
        let leaders = |ops: &[DrawOp]| -> Vec<(f64, f64)> {
            ops.iter()
                .filter_map(|o| match o {
                    DrawOp::Line { x1, y1, x2, y2, .. } if (x1 - x2).abs() < 1e-9 => Some((*y1, *y2)),
                    _ => None,
                })
                .collect()
        };

        // 1017 wants a box of 1012..1022 - two millimetres past the page bottom.
        let over = render(1017.0);
        let im = images(&over);
        assert_eq!(im.len(), 1, "the plate must PRINT, not degrade to a tick");
        let (_x, y0, _w, h, _c) = im[0];
        assert!(
            y0 >= 1000.0 - 1e-6 && y0 + h <= 1020.0 + 1e-6,
            "and it must sit wholly inside the page: {y0}..{}",
            y0 + h
        );
        let ld = leaders(&over);
        assert_eq!(ld.len(), 1, "a moved plate carries exactly one leader: {ld:?}");
        assert!(
            (ld[0].0 - 1017.0).abs() < 1e-6,
            "the leader starts at the TRUE depth, whatever the picture had to do: {:?}",
            ld[0]
        );
        assert!(
            (ld[0].1 - (y0 + h / 2.0)).abs() < 1e-6,
            "and ends on the plate's centre, which is what moved: {:?}",
            ld[0]
        );

        // The other side: a plate with room is left exactly where it was, and says nothing.
        let ok = render(1010.0);
        let im = images(&ok);
        assert_eq!(im.len(), 1);
        assert!(
            (im[0].1 + im[0].3 / 2.0 - 1010.0).abs() < 1e-6,
            "a plate that fits stays centred on its depth: {}",
            im[0].1
        );
        assert!(
            leaders(&ok).is_empty(),
            "and carries no callout - a leader on every plate would say nothing"
        );
    }

    #[test]
    fn an_anchored_plate_keeps_its_aspect_ratio_and_centres_on_its_depth() {
        // A thin section has no thickness, so its height comes from the picture, never from
        // a guessed interval — and it sits ON its sample depth.
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(&mut ops, &st, &[plate(0, "TS-1", 1010.0, None, JPEG)], 0.0, 40.0, 1000.0, 1020.0, &y);
        let im = images(&ops);
        assert_eq!(im.len(), 1);
        let (x, y0, w, h, cover) = im[0];
        assert!((w - 20.0).abs() < 1e-6, "half of a 40-wide track");
        assert!((h - 10.0).abs() < 1e-6, "200x100 px is aspect 0.5, so 20 wide is 10 tall");
        assert!((x - 10.0).abs() < 1e-6, "centred by default");
        assert!((y0 + h / 2.0 - 1010.0).abs() < 1e-6, "centred on the sample depth");
        assert!(!cover);
    }

    #[test]
    fn a_depth_scaled_plate_fits_inside_its_own_interval() {
        // A core photograph of a measured run DOES occupy its interval — but `contain` must
        // never distort it to fill a box the interval makes the wrong shape.
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "mode": "depth", "size": 1.0 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        // 40 mm wide would want 20 mm of height, but the interval is only 4 m deep.
        draw_image_series(
            &mut ops, &st, &[plate(0, "CP-1", 1000.0, Some(1004.0), JPEG)], 0.0, 40.0, 990.0, 1020.0, &y,
        );
        let (x, y0, w, h, cover) = images(&ops)[0];
        assert!((h - 4.0).abs() < 1e-6, "height is capped by the interval");
        assert!((w - 8.0).abs() < 1e-6, "width shrinks with it: aspect 0.5 is preserved");
        assert!((x - 16.0).abs() < 1e-6, "the narrowed picture stays centred");
        assert!((y0 - 1000.0).abs() < 1e-6);
        assert!(!cover);
    }

    #[test]
    fn fit_cover_hands_the_whole_interval_box_to_the_backend_to_crop() {
        let st = image_style(
            serde_json::json!({ "dataset": "THIN SECTION", "mode": "depth", "size": 1.0, "fit": "cover" }),
        );
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(
            &mut ops, &st, &[plate(0, "CP-1", 1000.0, Some(1004.0), JPEG)], 0.0, 40.0, 990.0, 1020.0, &y,
        );
        let (_, _, w, h, cover) = images(&ops)[0];
        assert!(cover, "cover clips rather than shrinking");
        assert!((w - 40.0).abs() < 1e-6 && (h - 4.0).abs() < 1e-6, "the box is the interval, full width");
    }

    #[test]
    fn an_overlapping_plate_is_skipped_never_moved() {
        // Two thin sections 0.5 m apart at a scale where each is 10 mm tall cannot both be
        // drawn. Moving the second would attribute it to a depth it was not cut from, so it
        // is dropped and only its depth tick remains.
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(
            &mut ops,
            &st,
            &[plate(0, "TS-1", 1005.0, None, JPEG), plate(1, "TS-2", 1005.5, None, JPEG)],
            0.0,
            40.0,
            1000.0,
            1020.0,
            &y,
        );
        let im = images(&ops);
        assert_eq!(im.len(), 1, "the second plate is skipped, not nudged");
        assert!((im[0].1 + im[0].3 / 2.0 - 1005.0).abs() < 1e-6, "the survivor keeps its true depth");
    }

    /// AUDIT-2026-08-20 finding 42. "Not embeddable" is a statement about ONE back-end. The PDF
    /// writer encodes JPEG and nothing else, so a PNG plate becomes a named frame there — never
    /// a silent gap, or a deliverable cannot be checked against the delivery list. The SVG
    /// writer inlines any MIME as a data URI and has no such limit, and it used to inherit one
    /// anyway: the same red frame was painted over a plate it could have shown perfectly.
    ///
    /// Both halves are pinned here, because either alone passes a lazier implementation —
    /// refusing everywhere satisfies the frame, and embedding everywhere satisfies the SVG.
    #[test]
    fn a_plate_the_pdf_cannot_embed_still_reaches_the_svg_and_is_named_in_the_pdf() {
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(
            &mut ops, &st, &[plate(0, "TS-PNG", 1010.0, None, "image/png")],
            0.0, 40.0, 1000.0, 1020.0, &y,
        );

        // The SVG carries the picture, under its own MIME, and says nothing about embedding.
        let svg = svg_page(&ops, 210.0, 297.0);
        assert!(svg.contains("data:image/png;base64,"), "the SVG can show a PNG, so it shows it");
        assert!(!svg.contains("not embeddable"), "and must not repeat the PDF's refusal: {svg}");

        // The PDF names it instead, and writes no XObject it never referenced.
        let stream = pdf_content(&ops, 210.0, 297.0);
        assert!(!stream.contains("/Im0 Do"), "a PNG is not a DCTDecode stream: {stream}");
        assert!(collect_images(&[ops.as_slice()]).is_empty(), "so no XObject is written for it");
        let pdf = String::from_utf8_lossy(&assemble_pdf_with_images(&[stream], 210.0, 297.0, &[])).into_owned();
        assert!(pdf.contains("TS-PNG"), "the frame names the plate standing behind it");
    }

    #[test]
    fn the_pdf_embeds_the_jpeg_bytes_untouched_and_references_them() {
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(&mut ops, &st, &[plate(0, "TS-1", 1010.0, None, JPEG)], 0.0, 40.0, 1000.0, 1020.0, &y);
        let stream = pdf_content(&ops, 210.0, 297.0);
        assert!(stream.contains("/Im0 Do"), "the content stream must reference the XObject");
        let imgs = collect_images(&[ops.as_slice()]);
        assert_eq!(imgs.len(), 1);
        let pdf = assemble_pdf_with_images(&[stream], 210.0, 297.0, &imgs);
        assert!(pdf.starts_with(b"%PDF-1.7"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Subtype /Image"), "an image XObject is written");
        assert!(text.contains("/Filter /DCTDecode"), "JPEG rides in as JPEG — never re-encoded");
        assert!(text.contains("/XObject << /Im0 "), "and the page's resources name it");
        // The exact delivered bytes must survive: a re-encode would quietly degrade a plate.
        let needle = [0xFFu8, 0xD8, 1, 2, 3, 0xD9];
        assert!(pdf.windows(needle.len()).any(|w| w == needle), "the JPEG bytes are stored verbatim");
    }

    #[test]
    fn a_page_with_no_images_writes_the_same_pdf_it_always_did() {
        // report.rs and the frontend's single-page path both go through the no-image route;
        // adding XObjects must not change a plain composite by a byte.
        let stream = "0.1 0.1 0.1 rg\n".to_string();
        let old = assemble_pdf(&[stream.clone()], 210.0, 297.0);
        let new = assemble_pdf_with_images(&[stream], 210.0, 297.0, &[]);
        assert_eq!(old, new);
        assert!(!String::from_utf8_lossy(&old).contains("/XObject"));
    }

    #[test]
    fn an_svg_page_carries_its_plates_inline_so_a_delivered_file_is_self_contained() {
        let st = image_style(serde_json::json!({ "dataset": "THIN SECTION", "size": 0.5 }));
        let mut ops = Vec::new();
        let y = |d: f32| d as f64;
        draw_image_series(&mut ops, &st, &[plate(0, "TS-1", 1010.0, None, JPEG)], 0.0, 40.0, 1000.0, 1020.0, &y);
        let svg = svg_page(&ops, 210.0, 297.0);
        assert!(svg.contains("data:image/jpeg;base64,"), "no external file references");
        assert!(svg.contains(r#"preserveAspectRatio="none""#), "the box is already the right shape");
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
        let n = db::write_array_log(&conn, &w, "MONTECARLO", "MC_PHIE_REAL", &depths, &vals, None).unwrap();
        assert_eq!(n, 2, "the empty depth is skipped, not stored as a zero-width distribution");

        let back = db::read_array_log(&conn, &w, Some("MONTECARLO"), "mc_phie_real").unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].depth, 1000.0);
        assert_eq!(back[0].samples[0], 0.1);
        assert_eq!(back[0].samples[1], 0.2);
        assert!(back[0].samples[2].is_nan(), "a failed realization keeps its SLOT so index r stays stable");
        assert_eq!(back[1].samples, vec![0.15, 0.25, 0.35]);

        // A re-run replaces its own output rather than unioning two runs' realizations.
        db::write_array_log(&conn, &w, "MONTECARLO", "MC_PHIE_REAL", &[1000.0], &[vec![0.9, 0.8]], None).unwrap();
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

    /// **A refused array write leaves the curve exactly as it was.**
    ///
    /// This is the REFUSAL, and it is what this test pins. Before it, a duplicated depth reached
    /// the store: the replacement's DELETE committed, the first row of the new delivery went in,
    /// the repeat hit the primary key, and the caller got an engine message naming an internal
    /// table and no depth — with the previous curve already gone and one row of the new one kept.
    /// The `back.len() == 2` assertion below is what fails on that version.
    ///
    /// **What this does NOT exercise is the transaction**, and saying so is the point: the refusal
    /// short-circuits before `with_txn` is ever entered, so nothing here proves the rollback works.
    /// The transaction is there for the failures no pre-check can foresee — an unclean kill mid-
    /// write, I/O, a constraint added later — which is exactly the case `with_txn`'s own doc
    /// describes. It earns its place by being used, not by being tested here.
    #[test]
    fn a_refused_array_write_leaves_the_stored_curve_untouched() {
        let conn = Connection::open_in_memory().unwrap();
        db::create_schema(&conn).unwrap();
        let wid = Uuid::new_v4();
        db::insert_well(&conn, wid, "ARRAYS", None, None, None).unwrap();
        let w = wid.to_string();

        db::write_array_log(&conn, &w, "RAW", "PC_SW", &[1000.0, 1001.0], &[vec![0.9], vec![0.8]], None)
            .unwrap();

        // A second delivery repeating a depth. Refused, and the depth is named.
        let err = db::write_array_log(
            &conn,
            &w,
            "RAW",
            "PC_SW",
            &[2000.0, 2000.0, 2001.0],
            &[vec![0.5], vec![0.4], vec![0.3]],
            None,
        )
        .expect_err("one vector per depth, so this cannot be stored");
        let msg = err.to_string();
        assert!(msg.contains("2000.00"), "the duplicated depth is named: {msg}");
        assert!(!msg.contains("array_logs"), "and not by the table's internal name: {msg}");

        // The first delivery is still there, whole. This is the assertion the transaction earns:
        // the DELETE at the top of the replacement must not survive the refusal.
        let back = db::read_array_log(&conn, &w, Some("RAW"), "PC_SW").unwrap();
        assert_eq!(back.len(), 2, "the stored curve is untouched: {back:?}");
        assert_eq!(back[0].samples, vec![0.9]);
        assert_eq!(back[1].samples, vec![0.8]);

        // The control: a depth whose vector is EMPTY is skipped by the writer and never reaches
        // the table, so two of them are not a duplicate — refusing there would reject a write the
        // store would have taken.
        let n = db::write_array_log(
            &conn,
            &w,
            "RAW",
            "PC_SW",
            &[3000.0, 3000.0, 3001.0],
            &[vec![], vec![], vec![0.2]],
            None,
        )
        .expect("two skipped depths are not a collision");
        assert_eq!(n, 1);
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
        assert_eq!(res.well_name, "SANDI-COMPOSITE");

        let p0 = &res.pages[0].svg;
        assert!(p0.starts_with("<svg"));
        assert!(p0.contains("width=\"210mm\""));
        assert!(p0.contains("SANDI-COMPOSITE"));
        assert!(p0.contains("Sandi Field"));
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

    /// Every major depth label on a page, as (depth, y-in-mm). The labels are the only ops that
    /// carry a depth AND a page position, which is what makes the print scale measurable from
    /// the emitted artwork rather than from the constants that produced it.
    fn depth_labels(ops: &[DrawOp]) -> Vec<(f64, f64)> {
        let mut out: Vec<(f64, f64)> = ops
            .iter()
            .filter_map(|o| match o {
                // The depth column sits at the far left; a track's own header text is higher up
                // and never parses as a bare integer.
                DrawOp::Text { x, y, s, .. } if *x < MARGIN_L + DEPTH_TRACK_W => {
                    s.parse::<f64>().ok().map(|d| (d, *y))
                }
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        out
    }

    /// T-REP-02. `print_scale_is_physically_exact` below checks that 1000/200 is 5.0, which is
    /// arithmetic — it never looks at the page. This measures the scale in the ARTWORK: how far
    /// apart the depth labels actually land.
    ///
    /// The claim is the one the whole print path rests on. At 1:200 one metre of formation must
    /// occupy exactly 5 mm of paper, because someone will put a ruler on the printed log and read
    /// a sand thickness off it. A composite that is out by a few percent still looks entirely
    /// correct — the curves are there, the grid is even, the header says 1:200 — and the error
    /// only surfaces as a thickness that disagrees with the tops.
    #[test]
    fn a_metre_of_formation_occupies_its_declared_millimetres_on_the_page() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn);

        for scale in [200u32, 500, 1000] {
            let (pages, _pw, _ph, _n) =
                render_pages(&conn, &full_spec(w.clone(), scale, PageSize::A4)).unwrap();
            let labels = depth_labels(&pages[0].ops);
            assert!(labels.len() >= 3, "1:{scale} produced too few depth labels to measure");

            let expected_mm_per_m = 1000.0 / scale as f64;
            // Every adjacent pair, not just the ends: a scale that drifted across the page
            // would still pass an end-to-end check.
            for pair in labels.windows(2) {
                let (d0, y0) = pair[0];
                let (d1, y1) = pair[1];
                let mm_per_m = (y1 - y0) / (d1 - d0);
                assert!(
                    (mm_per_m - expected_mm_per_m).abs() < 1e-9,
                    "1:{scale}: {d0} m to {d1} m spans {:.6} mm/m, expected {expected_mm_per_m}",
                    mm_per_m
                );
            }
            // Deeper prints lower. A sign slip here mirrors the log vertically, which is
            // obvious on screen and easy to miss in a page of vector ops.
            assert!(labels[0].1 < labels[labels.len() - 1].1, "1:{scale}: depth must increase downwards");
        }
    }

    /// The same measurement on a FOOT-declared project, from the Codex whole-repository review
    /// of a6565bd9 (P0). The exporter computed `1000 / scale` unconditionally and then added the
    /// resulting metre count straight onto a stored depth, so a foot project drew a 100 ft
    /// interval as if it were 100 m: the sheet came out 3.28084x too long, the header's "1:500"
    /// was physically about 1:152, and the page breaks landed in the wrong rock. Nothing on the
    /// page said so — the curves are there, the grid is even, and the only symptom is a ruler
    /// disagreeing with the tops.
    ///
    /// One physical well, described twice. The claim is not "the numbers match" but "a metre of
    /// ROCK occupies the same paper either way", which is the only thing a print scale can mean.
    /// The pagination half is asserted too, and separately: a page that covers the right amount
    /// of rock at the wrong scale, or the right scale over the wrong span, are different bugs.
    #[test]
    fn a_foot_declared_project_prints_the_same_physical_scale_a_metre_one_does() {
        use crate::units::{DepthUnit, M_PER_FT};

        /// Millimetres of paper per METRE of rock, measured off the artwork's own depth labels,
        /// plus the physical length of rock the first page covers.
        let measure = |feet: bool, scale: u32| -> (f64, f64) {
            let conn = Connection::open_in_memory().unwrap();
            db::create_schema(&conn).unwrap();
            // Declared BEFORE any well exists, which is the only time this is allowed.
            if feet {
                crate::units::set_project_depth_unit(&conn, DepthUnit::Feet).unwrap();
            }
            let wid = Uuid::new_v4();
            db::insert_well(&conn, wid, "SANDI-COMPOSITE", Some("Sandi Field"), None, None).unwrap();
            let w = wid.to_string();
            let n = 400;
            // The same rock: 1000 m to 1199.5 m at a half-metre step, or exactly that in feet.
            let scale_to_stored = |m: f32| if feet { m / M_PER_FT as f32 } else { m };
            let depths: Vec<f32> = (0..n).map(|i| scale_to_stored(1000.0 + i as f32 * 0.5)).collect();
            let gr: Vec<f32> = (0..n).map(|i| 40.0 + 60.0 * ((i as f32 / 20.0).sin() * 0.5 + 0.5)).collect();
            db::insert_standard_curves(
                &conn, wid, depths, gr, vec![10.0; n], vec![0.25; n], vec![2.4; n],
                vec![f32::NAN; n], vec![f32::NAN; n],
            )
            .unwrap();

            let (pages, _pw, _ph, _name) =
                render_pages(&conn, &full_spec(w, scale, PageSize::A4)).unwrap();
            let labels = depth_labels(&pages[0].ops);
            assert!(labels.len() >= 3, "feet={feet} 1:{scale}: too few depth labels to measure");
            let (d0, y0) = labels[0];
            let (d1, y1) = labels[labels.len() - 1];
            let stored_per_metre = if feet { 1.0 / M_PER_FT } else { 1.0 };
            let mm_per_metre_of_rock = (y1 - y0) / (d1 - d0) * stored_per_metre;
            let page_metres = (pages[0].bot - pages[0].top) as f64 / stored_per_metre;
            (mm_per_metre_of_rock, page_metres)
        };

        for scale in [200u32, 500] {
            let (metric_mm, metric_span) = measure(false, scale);
            let (foot_mm, foot_span) = measure(true, scale);
            let expected = 1000.0 / scale as f64;

            assert!(
                (metric_mm - expected).abs() < 1e-9,
                "1:{scale}: the metre project must be unchanged, got {metric_mm} mm per metre"
            );
            assert!(
                (foot_mm - expected).abs() < 1e-6,
                "1:{scale}: a metre of rock must occupy {expected} mm however the project is \
                 declared — the foot project printed it at {foot_mm} mm, a factor of {:.5}",
                foot_mm / expected
            );
            assert!(
                (metric_span - foot_span).abs() < 1e-3,
                "1:{scale}: one page must cover the same rock either way — {metric_span} m \
                 metric vs {foot_span} m foot"
            );
        }
    }

    /// T-REP-02, the pagination half. Page count is not a cosmetic property: it is what says the
    /// depth-per-page came out right. If a page silently held more metres than the scale allows,
    /// the curves would be compressed to fit and the print scale in the header would be a lie.
    ///
    /// The relationships are checked rather than a hardcoded count, and the ratios are
    /// deliberately loose in one direction — the FIRST page carries a taller metadata header
    /// (32 mm against 8 mm) so it holds fewer metres than the pages after it. That is why the
    /// plan says "≈2.5×" rather than exactly 2.5, and why a short well can give the same page
    /// count at two different page sizes.
    #[test]
    fn the_page_count_follows_the_print_scale_and_the_page_size() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn); // logged 1000.0 .. 1199.5

        let count = |scale: u32, page: PageSize| -> usize {
            render_composite(&conn, &full_spec(w.clone(), scale, page)).unwrap().pages.len()
        };
        let a4_200 = count(200, PageSize::A4);
        let a4_500 = count(500, PageSize::A4);
        let a4_1000 = count(1000, PageSize::A4);
        let a3_200 = count(200, PageSize::A3);

        assert!(a4_200 > a4_500, "1:200 must need more pages than 1:500 ({a4_200} vs {a4_500})");
        assert!(a4_500 > a4_1000, "1:500 must need more pages than 1:1000 ({a4_500} vs {a4_1000})");
        assert!(
            a3_200 < a4_200,
            "a taller page holds more rock at the same scale ({a3_200} on A3 vs {a4_200} on A4)"
        );

        // The metres one page holds scale exactly with the denominator. Measured on page ONE,
        // because it is the only page guaranteed to be full at both scales here: the LAST page
        // is clipped at TD, and this well is short enough that 1:500 has no page in between.
        // Comparing two first pages is still like-for-like — both carry the tall header.
        let first_page_metres = |scale: u32| -> f64 {
            let res = render_composite(&conn, &full_spec(w.clone(), scale, PageSize::A4)).unwrap();
            assert!(res.pages.len() >= 2, "1:{scale} fits on one page — page 1 would be clipped at TD");
            let p = &res.pages[0];
            (p.bottom_depth - p.top_depth) as f64
        };
        let m200 = first_page_metres(200);
        let m500 = first_page_metres(500);
        assert!(
            (m500 / m200 - 2.5).abs() < 1e-4,
            "a 1:500 page must hold exactly 2.5x the rock of a 1:200 one ({m500} vs {m200})"
        );

        // Every page set tiles its interval exactly: no gap (rock printed on no page) and no
        // overlap (rock printed twice, which reads as a repeated section).
        for scale in [200u32, 500, 1000] {
            let res = render_composite(&conn, &full_spec(w.clone(), scale, PageSize::A4)).unwrap();
            for pair in res.pages.windows(2) {
                assert!(
                    (pair[1].top_depth - pair[0].bottom_depth).abs() < 1e-3,
                    "1:{scale}: page {} ends at {} and page {} starts at {}",
                    pair[0].index, pair[0].bottom_depth, pair[1].index, pair[1].top_depth
                );
            }
            assert!((res.pages.first().unwrap().top_depth - 1000.0).abs() < 1e-3, "1:{scale} top");
            assert!((res.pages.last().unwrap().bottom_depth - 1199.5).abs() < 0.6, "1:{scale} base");
        }
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

    /// T-REP-03. A depth window that selects no rock must be REFUSED, not returned as an
    /// empty page set. The pane only swaps its preview on success, so a silent empty result
    /// would leave the PREVIOUS window's pages on screen labelled as the new one — the
    /// failure mode the manual step is written to catch.
    ///
    /// Note the order of operations in `render_pages`: the window is CLAMPED against the
    /// logged interval first (`.max(data_top)` / `.min(data_bot)`) and only then checked, so
    /// every refusal below arrives at `!(bottom > top)` by a different route.
    #[test]
    fn a_depth_window_that_selects_no_rock_is_refused_rather_than_rendered() {
        let conn = Connection::open_in_memory().unwrap();
        let w = seed_well(&conn); // logged 1000.0 .. 1199.5
        let win = |top: Option<f32>, bot: Option<f32>| {
            let mut s = full_spec(w.clone(), 500, PageSize::A4);
            s.depth_top = top;
            s.depth_bottom = bot;
            render_composite(&conn, &s)
        };

        // Top below bottom. Both fields hold valid numbers in the logged interval, so nothing
        // upstream can catch this — only the render can.
        assert!(win(Some(1150.0), Some(1050.0)).is_err(), "top below bottom must not render");
        // Wholly below TD, and wholly above the logged top. The clamp collapses each to an
        // inverted pair rather than to an empty-but-valid one.
        assert!(win(Some(9000.0), Some(9100.0)).is_err(), "a window under TD must not render");
        assert!(win(Some(100.0), Some(200.0)).is_err(), "a window above the logged top must not render");
        // Zero thickness is not a window; `!(bottom > top)` is strict for this reason.
        assert!(win(Some(1100.0), Some(1100.0)).is_err(), "a zero-thickness window must not render");

        // A NaN in one field is ABSORBED, not refused — `f32::max`/`min` ignore NaN by
        // definition, so the clamp replaces it with the data bound before the guard sees it,
        // and the result is identical to leaving that field blank. Recorded because the guard
        // is written `!(bottom > top)` rather than `bottom <= top`, which reads as NaN-safety
        // that is in fact unreachable from here. (It is also unreachable over IPC: JSON has no
        // NaN literal, so a frontend `parseFloat("abc")` arrives as `null` -> `None`.)
        let nan_top = win(Some(f32::NAN), Some(1150.0)).expect("a NaN top behaves as blank");
        let blank_top = win(None, Some(1150.0)).expect("blank top renders");
        assert_eq!(nan_top.pages.len(), blank_top.pages.len());
        assert!((nan_top.pages[0].top_depth - blank_top.pages[0].top_depth).abs() < 1e-3);

        // The control, and the reason the clamp is a clamp rather than a fifth error: a window
        // that OVERLAPS the data is honoured over the overlap. You cannot render rock that was
        // never logged, so 1150 -> 9000 renders 1150 -> 1199.5 and the page labels say 1199.5.
        let over = win(Some(1150.0), Some(9000.0)).expect("a partially overlapping window renders");
        assert!((over.pages.first().unwrap().top_depth - 1150.0).abs() < 1e-3);
        assert!(
            (over.pages.last().unwrap().bottom_depth - 1199.5).abs() < 0.6,
            "the page labels must state the logged bottom, not the requested 9000"
        );
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
        let svg_out = std::env::var("SANDIBUMI_SVG_OUT")
            .unwrap_or_else(|_| std::env::temp_dir().join("arshilla_composite_p0.svg").to_string_lossy().into());
        std::fs::write(&svg_out, &res.pages[0].svg).unwrap();
        let pdf = render_composite_pdf(&conn, &spec).unwrap();
        let pdf_out = std::env::var("SANDIBUMI_PDF_OUT")
            .unwrap_or_else(|_| std::env::temp_dir().join("arshilla_composite.pdf").to_string_lossy().into());
        std::fs::write(&pdf_out, &pdf).unwrap();
        println!("wrote {svg_out} and {pdf_out}");
    }
}
