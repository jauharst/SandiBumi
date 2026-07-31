//! Depth-registered pictures: petrographic thin sections, core photographs, SEM plates —
//! anything a laboratory delivers as a raster beside the plugs, shown in its own log track.
//!
//! Three things happen here, in the project's usual **probe -> confirm -> commit** order
//! (the same shape as the core-table wizard):
//!
//! 1. [`probe_image_files`] reads each selected file's HEADER only, reports its true pixel
//!    size, and GUESSES a depth from the filename. The guess is shown in an editable table
//!    and never applied silently — a mis-parsed depth would hang a thin section off the
//!    wrong sand, which is exactly the kind of error nothing downstream can catch.
//! 2. [`prepare_images`] normalizes a whole delivery in ONE Python/Pillow subprocess
//!    (rule 7: subprocess, never embedded — a missing Python must never stop the app).
//!    The stored copy is a JPEG capped on its long edge, because the viewer, the SVG export
//!    and the PDF exporter all need one decodable form, and a 6000x4000 camera original
//!    would bloat a field project for no visible gain at track width.
//! 3. [`import_images`] converts the confirmed depths into the project's depth unit and
//!    commits the delivery as a named image SET (see `db::insert_well_images`).
//!
//! Without Pillow the import still works for pictures the WebView can decode itself
//! (JPEG/PNG/GIF/WebP), stored verbatim; those are flagged `printable = false` when the PDF
//! exporter cannot embed them, so a composite prints a labelled frame instead of a silent
//! gap. TIFF and anything else needs Pillow, and says so by name.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use duckdb::Connection;
use serde::{Deserialize, Serialize};

use crate::python_engine::{find_python, hide_console};
use crate::units::{self, DepthUnit};

/// Long-edge pixel cap for the stored display copy. At a printed track width of a few
/// centimetres, 2400 px is far past what any plate can resolve on paper, while a modern
/// camera original is 3-5x larger in bytes.
pub const DEFAULT_MAX_PX: u32 = 2400;
/// JPEG quality for the display copy. 85 is the usual visually-lossless working point.
pub const DEFAULT_QUALITY: u8 = 85;

// ---------------------------------------------------------------------------
// Raster header sniffing
// ---------------------------------------------------------------------------

/// What a raster's own header says about it, without decoding a single pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterMeta {
    pub mime: &'static str,
    pub width: u32,
    pub height: u32,
    /// Colour components — JPEG only (1 = grey, 3 = RGB, 4 = CMYK). The PDF exporter needs
    /// it to name the right colour space when it embeds the bytes untouched.
    pub components: u8,
}

fn be16(b: &[u8], at: usize) -> u32 {
    ((b[at] as u32) << 8) | b[at + 1] as u32
}
fn be32(b: &[u8], at: usize) -> u32 {
    ((b[at] as u32) << 24) | ((b[at + 1] as u32) << 16) | ((b[at + 2] as u32) << 8) | b[at + 3] as u32
}
fn le16(b: &[u8], at: usize) -> u32 {
    ((b[at + 1] as u32) << 8) | b[at] as u32
}
fn le32(b: &[u8], at: usize) -> u32 {
    ((b[at + 3] as u32) << 24) | ((b[at + 2] as u32) << 16) | ((b[at + 1] as u32) << 8) | b[at] as u32
}

/// Identifies a raster from its magic bytes and reads its dimensions.
///
/// Header-only on purpose: the import wizard shows the true pixel size of every selected
/// file before anything is stored, and doing that by decoding would make selecting a folder
/// of 300 core photographs a minute-long stall.
pub fn sniff(bytes: &[u8]) -> Option<RasterMeta> {
    if bytes.len() >= 24 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        // IHDR is required to be the first chunk: length(4) type(4) then width, height.
        return Some(RasterMeta {
            mime: "image/png",
            width: be32(bytes, 16),
            height: be32(bytes, 20),
            components: 0,
        });
    }
    if bytes.len() >= 4 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        return sniff_jpeg(bytes);
    }
    if bytes.len() >= 10 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        return Some(RasterMeta { mime: "image/gif", width: le16(bytes, 6), height: le16(bytes, 8), components: 0 });
    }
    if bytes.len() >= 26 && bytes.starts_with(b"BM") {
        // BITMAPINFOHEADER: signed 32-bit width/height (height < 0 = top-down).
        let w = le32(bytes, 18) as i32;
        let h = le32(bytes, 22) as i32;
        return Some(RasterMeta {
            mime: "image/bmp",
            width: w.unsigned_abs(),
            height: h.unsigned_abs(),
            components: 0,
        });
    }
    if bytes.len() >= 30 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        // Only the extended (VP8X) form carries the canvas size at a fixed offset; for the
        // plain lossy/lossless forms the dimensions live inside the bitstream, and Pillow
        // fills them in. Reporting 0 is honest — the wizard shows "?" rather than a guess.
        let (w, h) = if &bytes[12..16] == b"VP8X" {
            (
                1 + ((bytes[24] as u32) | ((bytes[25] as u32) << 8) | ((bytes[26] as u32) << 16)),
                1 + ((bytes[27] as u32) | ((bytes[28] as u32) << 8) | ((bytes[29] as u32) << 16)),
            )
        } else {
            (0, 0)
        };
        return Some(RasterMeta { mime: "image/webp", width: w, height: h, components: 0 });
    }
    if bytes.len() >= 8 && (bytes.starts_with(b"II\x2a\x00") || bytes.starts_with(b"MM\x00\x2a")) {
        // TIFF dimensions live in IFD tags that can sit anywhere in the file; Pillow reads
        // them. Recognising the format is still worth it so the wizard can say "needs
        // Pillow" instead of "unrecognised file".
        return Some(RasterMeta { mime: "image/tiff", width: 0, height: 0, components: 0 });
    }
    None
}

/// Walks JPEG segment markers to the frame header. `SOFn` carries precision, height, width
/// and component count; `DHT`/`DAC`/`RSTn` share the 0xC_ range and are skipped by value.
fn sniff_jpeg(bytes: &[u8]) -> Option<RasterMeta> {
    let mut i = 2usize;
    while i + 9 < bytes.len() {
        if bytes[i] != 0xFF {
            i += 1; // fill byte or padding; resynchronise rather than give up
            continue;
        }
        let marker = bytes[i + 1];
        i += 2;
        match marker {
            0xD8 | 0x01 | 0xD0..=0xD7 | 0xFF => continue, // standalone markers, no length
            0xD9 | 0xDA => return None,                   // end of image / start of scan: no SOF found
            _ => {}
        }
        if i + 1 >= bytes.len() {
            return None;
        }
        let len = be16(bytes, i) as usize;
        let is_sof = matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if is_sof {
            if i + 7 >= bytes.len() {
                return None;
            }
            return Some(RasterMeta {
                mime: "image/jpeg",
                height: be16(bytes, i + 3),
                width: be16(bytes, i + 5),
                components: bytes[i + 7],
            });
        }
        if len < 2 {
            return None;
        }
        i += len;
    }
    None
}

/// Whether the WebView can decode this format itself. Used only when Pillow is missing —
/// with Pillow every import becomes a JPEG and the question does not arise.
fn browser_decodable(mime: &str) -> bool {
    matches!(mime, "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/bmp")
}

// ---------------------------------------------------------------------------
// Depth guessed from the filename
// ---------------------------------------------------------------------------

/// One numeric run in a filename, with where it sat.
struct NumTok {
    start: usize,
    end: usize,
    value: f32,
    /// A token qualifies as a depth only if it carries a decimal point or has at least three
    /// integer digits — otherwise the `01` of `SANDI-01` would be read as a depth of 1 m.
    qualifies: bool,
}

fn numeric_tokens(s: &str) -> Vec<NumTok> {
    let b: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if !b[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let mut int_digits = 0usize;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            int_digits += 1;
        }
        let mut has_dot = false;
        if i + 1 < b.len() && b[i] == '.' && b[i + 1].is_ascii_digit() {
            has_dot = true;
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        }
        let text: String = b[start..i].iter().collect();
        if let Ok(v) = text.parse::<f32>() {
            out.push(NumTok { start, end: i, value: v, qualifies: has_dot || int_digits >= 3 });
        }
    }
    out
}

/// Guesses a depth (and, for a photographed interval, a base depth) from a file name.
///
/// `SANDI-01_1523.50.jpg` -> (1523.50, None); `SANDI-01_1523.50-1524.00.jpg` -> a range.
/// A pair is only read as a range when the two tokens are adjacent, separated by a single
/// `-` or `_`, and in increasing depth order — so `SANDI-2021_1523.5` correctly yields the
/// single depth 1523.5 rather than a 2021 m interval. The result is always shown for
/// confirmation before anything is stored.
pub fn parse_depth_from_name(stem: &str) -> (Option<f32>, Option<f32>) {
    let chars: Vec<char> = stem.chars().collect();
    let toks: Vec<NumTok> = numeric_tokens(stem).into_iter().filter(|t| t.qualifies).collect();
    if toks.is_empty() {
        return (None, None);
    }
    if toks.len() >= 2 {
        let a = &toks[toks.len() - 2];
        let b = &toks[toks.len() - 1];
        let adjacent = b.start == a.end + 1 && matches!(chars.get(a.end), Some('-') | Some('_'));
        if adjacent && b.value > a.value {
            return (Some(a.value), Some(b.value));
        }
    }
    (Some(toks[toks.len() - 1].value), None)
}

// ---------------------------------------------------------------------------
// Probe: what the wizard shows before anything is stored
// ---------------------------------------------------------------------------

/// One selected file as the import wizard presents it: what it is, how big it really is,
/// and the depth guessed from its name.
#[derive(Debug, Clone, Serialize)]
pub struct ImageProbe {
    pub path: String,
    pub file_name: String,
    /// Suggested label = the filename stem; editable in the wizard.
    pub name: String,
    pub mime: String,
    /// Pixel size from the header; 0 when only Pillow can tell (TIFF, plain WebP).
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub depth_top: Option<f32>,
    pub depth_base: Option<f32>,
    /// Set when this file cannot be imported at all; the wizard greys the row out.
    pub error: Option<String>,
}

/// Reads the header of every selected file. Never decodes, never writes.
pub fn probe_image_files(paths: &[String]) -> Vec<ImageProbe> {
    let pillow = pillow_available();
    paths
        .iter()
        .map(|p| {
            let path = Path::new(p);
            let file_name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| p.clone());
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| file_name.clone());
            let (depth_top, depth_base) = parse_depth_from_name(&stem);
            let mut probe = ImageProbe {
                path: p.clone(),
                file_name,
                name: stem,
                mime: String::new(),
                width: 0,
                height: 0,
                bytes: 0,
                depth_top,
                depth_base,
                error: None,
            };
            // 64 KiB covers every header form above, including a JPEG carrying a large EXIF
            // thumbnail before its frame header.
            match read_head(p, 65536) {
                Ok((head, size)) => {
                    probe.bytes = size;
                    match sniff(&head) {
                        Some(m) => {
                            probe.mime = m.mime.to_string();
                            probe.width = m.width;
                            probe.height = m.height;
                            if !pillow && !browser_decodable(m.mime) {
                                probe.error = Some(format!(
                                    "{} needs Pillow (pip install pillow) to be read",
                                    m.mime.trim_start_matches("image/").to_uppercase()
                                ));
                            }
                        }
                        None => probe.error = Some("not a recognised image format".into()),
                    }
                }
                Err(e) => probe.error = Some(e),
            }
            probe
        })
        .collect()
}

fn read_head(path: &str, max: usize) -> Result<(Vec<u8>, u64), String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("cannot open: {e}"))?;
    let size = f.metadata().map(|m| m.len()).unwrap_or(0);
    let mut buf = vec![0u8; max.min(size.max(1) as usize)];
    let n = f.read(&mut buf).map_err(|e| format!("cannot read: {e}"))?;
    buf.truncate(n);
    Ok((buf, size))
}

// ---------------------------------------------------------------------------
// Normalization through Pillow (one subprocess for the whole delivery)
// ---------------------------------------------------------------------------

/// Reads each listed file, converts it to a capped RGB/grey JPEG, and writes one JSON
/// header line followed by the JPEG payloads back to back. Keep every message ASCII.
const PILLOW_RUNNER: &str = r#"
import sys, json, io
try:
    from PIL import Image
except Exception as e:
    sys.stderr.write("pillow-missing: %s\n" % e)
    sys.exit(3)
Image.MAX_IMAGE_PIXELS = None
# sys.stdin.buffer, never sys.stdin: a piped child's text stdin decodes with the Windows ANSI
# codepage, while serde_json sends raw UTF-8 — so a file path holding any non-ASCII character
# arrived mangled and the picture failed to open for no visible reason. json.loads takes bytes
# and assumes UTF-8, which is what was actually sent.
req = json.loads(sys.stdin.buffer.readline())
max_px = int(req.get("max_px", 2400))
quality = int(req.get("quality", 85))
meta = []
blobs = []
for path in req["files"]:
    try:
        im = Image.open(path)
        sw, sh = im.size
        im.load()
        if im.mode in ("RGBA", "LA") or (im.mode == "P" and "transparency" in im.info):
            # JPEG has no alpha. Composite onto WHITE, not the default black, so a plate
            # with a cut-out background prints the way it looks on a datasheet.
            rgba = im.convert("RGBA")
            bg = Image.new("RGB", rgba.size, (255, 255, 255))
            bg.paste(rgba, mask=rgba.split()[3])
            im = bg
        elif im.mode not in ("RGB", "L"):
            im = im.convert("RGB")
        w, h = im.size
        if max_px > 0 and max(w, h) > max_px:
            s = float(max_px) / float(max(w, h))
            im = im.resize((max(1, int(round(w * s))), max(1, int(round(h * s)))), Image.LANCZOS)
        buf = io.BytesIO()
        im.save(buf, format="JPEG", quality=quality, optimize=True)
        b = buf.getvalue()
        meta.append({"n": len(b), "w": im.size[0], "h": im.size[1], "src_w": sw, "src_h": sh, "error": None})
        blobs.append(b)
    except Exception as e:
        meta.append({"n": 0, "w": 0, "h": 0, "src_w": 0, "src_h": 0, "error": str(e)})
sys.stdout.write(json.dumps({"images": meta}) + "\n")
sys.stdout.flush()
out = sys.stdout.buffer
for b in blobs:
    out.write(b)
out.flush()
"#;

#[derive(Debug, Deserialize)]
struct PillowMeta {
    n: usize,
    w: u32,
    h: u32,
    src_w: u32,
    src_h: u32,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PillowHeader {
    images: Vec<PillowMeta>,
}

/// One picture ready to be stored.
#[derive(Debug, Clone)]
pub struct PreparedImage {
    pub mime: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub src_width: u32,
    pub src_height: u32,
    /// False = viewer only; the PDF exporter cannot embed this form (see `composite.rs`).
    pub printable: bool,
    pub error: Option<String>,
}

/// Is a usable Pillow reachable? Cheap enough to ask once per wizard opening, and the answer
/// decides whether the wizard offers TIFF at all.
pub fn pillow_available() -> bool {
    let Some(python) = find_python() else { return false };
    let mut cmd = Command::new(python);
    cmd.args(["-c", "import PIL.Image"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    hide_console(&mut cmd);
    matches!(cmd.status(), Ok(s) if s.success())
}

/// Normalizes a whole delivery in one subprocess, falling back to verbatim storage when
/// Pillow is not available.
///
/// One process for the batch rather than one per file: starting Python costs a few hundred
/// milliseconds, which is invisible once but half a minute across a folder of core photos.
pub fn prepare_images(paths: &[String], max_px: u32, quality: u8) -> Vec<PreparedImage> {
    match prepare_with_pillow(paths, max_px, quality) {
        Ok(v) => v,
        Err(reason) => paths.iter().map(|p| prepare_verbatim(p, &reason)).collect(),
    }
}

fn prepare_with_pillow(paths: &[String], max_px: u32, quality: u8) -> Result<Vec<PreparedImage>, String> {
    let python = find_python().ok_or_else(|| {
        "no Python found - install Python 3.10+ with Pillow, or set SANDIBUMI_PYTHON".to_string()
    })?;
    let mut cmd = Command::new(&python);
    cmd.args(["-c", PILLOW_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    let req = serde_json::json!({ "files": paths, "max_px": max_px, "quality": quality });
    {
        let stdin = child.stdin.as_mut().ok_or("python stdin closed")?;
        stdin.write_all(req.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("image conversion failed");
        return Err(if last.contains("pillow-missing") {
            "Pillow is not installed (pip install pillow)".to_string()
        } else {
            last.trim().to_string()
        });
    }
    let nl = out.stdout.iter().position(|&b| b == b'\n').ok_or("malformed image runner output")?;
    let header: PillowHeader =
        serde_json::from_slice(&out.stdout[..nl]).map_err(|e| format!("bad image header: {e}"))?;
    let payload = &out.stdout[nl + 1..];
    let mut offset = 0usize;
    let mut prepared = Vec::with_capacity(header.images.len());
    for m in &header.images {
        if let Some(e) = &m.error {
            prepared.push(PreparedImage {
                mime: String::new(),
                data: Vec::new(),
                width: 0,
                height: 0,
                src_width: 0,
                src_height: 0,
                printable: false,
                error: Some(e.clone()),
            });
            continue;
        }
        let end = offset + m.n;
        if end > payload.len() {
            return Err("image payload truncated".into());
        }
        prepared.push(PreparedImage {
            mime: "image/jpeg".into(),
            data: payload[offset..end].to_vec(),
            width: m.w,
            height: m.h,
            src_width: m.src_w,
            src_height: m.src_h,
            printable: true,
            error: None,
        });
        offset = end;
    }
    Ok(prepared)
}

/// The no-Pillow path: store what the lab delivered, byte for byte, when the WebView can
/// decode it. `reason` explains in the import report why nothing was normalized.
fn prepare_verbatim(path: &str, reason: &str) -> PreparedImage {
    let fail = |e: String| PreparedImage {
        mime: String::new(),
        data: Vec::new(),
        width: 0,
        height: 0,
        src_width: 0,
        src_height: 0,
        printable: false,
        error: Some(e),
    };
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail(format!("cannot read: {e}")),
    };
    let Some(meta) = sniff(&data) else { return fail("not a recognised image format".into()) };
    if !browser_decodable(meta.mime) {
        return fail(format!("{} cannot be read without Pillow ({reason})", meta.mime));
    }
    PreparedImage {
        // Only a JPEG can be embedded in the PDF exporter untouched (see `composite.rs`);
        // anything else displays fine but prints as a labelled frame until Pillow is there.
        printable: meta.mime == "image/jpeg",
        mime: meta.mime.to_string(),
        width: meta.width,
        height: meta.height,
        src_width: meta.width,
        src_height: meta.height,
        data,
        error: None,
    }
}

// ---------------------------------------------------------------------------
// Commit
// ---------------------------------------------------------------------------

/// One confirmed row of the import wizard.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageImportItem {
    pub path: String,
    pub name: String,
    pub depth_top: f32,
    #[serde(default)]
    pub depth_base: Option<f32>,
    #[serde(default)]
    pub caption: Option<String>,
}

/// The confirmed import, as the wizard sends it.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageImportRequest {
    pub well_id: String,
    /// 'THIN SECTION', 'CORE PHOTO', or anything the user types.
    pub dataset: String,
    /// Delivery name; auto-suffixed per well so an import never overwrites an earlier one.
    pub set_name: String,
    /// Unit the depths in `items` are expressed in; converted to the project unit on commit.
    #[serde(default)]
    pub depth_unit: Option<String>,
    #[serde(default)]
    pub max_px: Option<u32>,
    #[serde(default)]
    pub quality: Option<u8>,
    /// Treat the plates' depths as the ones the original core report used, and place them through
    /// the well's core depth record. `#[serde(default)]`, so an older payload still deserializes.
    #[serde(default)]
    pub follow_core: bool,
    pub items: Vec<ImageImportItem>,
}

/// What the import did, in the wizard's own terms.
#[derive(Debug, Clone, Serialize)]
pub struct ImageImportResult {
    pub dataset: String,
    pub set_name: String,
    pub imported: usize,
    /// One entry per file that did not make it, as "name: reason" — never silent.
    pub skipped: Vec<String>,
    pub bytes: i64,
    /// Depth conversion applied, or the Pillow situation, in one sentence.
    pub note: Option<String>,
}

/// Commits a confirmed image delivery: normalize, convert depths, store as a named set.
pub fn import_images(conn: &Connection, req: &ImageImportRequest) -> Result<ImageImportResult, String> {
    if req.items.is_empty() {
        return Err("no images selected".into());
    }
    let exists: bool = conn
        .query_row("SELECT 1 FROM wells WHERE well_id = ?1", duckdb::params![req.well_id], |_| Ok(true))
        .unwrap_or(false);
    if !exists {
        return Err(format!("unknown well '{}'", req.well_id));
    }

    let dataset = {
        let d = req.dataset.trim().to_uppercase();
        if d.is_empty() { "IMAGES".to_string() } else { d }
    };
    let target_set = db_resolve_set(conn, &req.well_id, &dataset, &req.set_name)?;

    // Depth unit: the wizard says what the numbers mean, the project decides what is stored.
    let project_unit = units::project_depth_unit_or_default(conn);
    let entered_unit = req.depth_unit.as_deref().and_then(DepthUnit::from_code).unwrap_or(project_unit);
    let convert = |d: f32| -> f32 { units::convert_depth(d as f64, entered_unit, project_unit) as f32 };

    // A thin section is cut from a plug, so its depth is the core's depth — and if that core has
    // been registered against the log since the lab wrote its report, the plate is out by however
    // far the plug moved. Resolved AFTER the unit conversion, because the record is in the
    // project's own depth unit.
    let core_pairs: Vec<(f32, f32)> = if req.follow_core {
        match crate::db::core_depth_pairs(conn, &req.well_id) {
            Ok(p) => p,
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };
    let mut outside_core = 0usize;
    let place = |d: f32, outside: &mut usize| -> f32 {
        if core_pairs.is_empty() {
            return d;
        }
        let (mapped, ex) = crate::db::map_core_depth(&core_pairs, d);
        if ex {
            *outside += 1;
        }
        mapped
    };

    let paths: Vec<String> = req.items.iter().map(|i| i.path.clone()).collect();
    let prepared = prepare_images(&paths, req.max_px.unwrap_or(DEFAULT_MAX_PX), req.quality.unwrap_or(DEFAULT_QUALITY));

    let mut rows: Vec<crate::db::NewImage> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut bytes = 0i64;
    let mut any_unprintable = false;
    for (item, prep) in req.items.iter().zip(prepared.iter()) {
        if let Some(e) = &prep.error {
            skipped.push(format!("{}: {e}", item.name));
            continue;
        }
        if !item.depth_top.is_finite() {
            skipped.push(format!("{}: no depth given", item.name));
            continue;
        }
        // A base ABOVE the top is a typo, not an interval; storing it would draw a picture
        // upside down over a negative thickness. Fall back to a point sample and say so.
        // The top is placed through the core record and the base takes the SAME offset, so a core
        // photograph keeps the thickness it was logged with. Mapping the two ends independently
        // could invert a thin plate where the correction changes steeply at a barrel boundary.
        let top = place(convert(item.depth_top), &mut outside_core);
        let offset = top - convert(item.depth_top);
        let base = match item.depth_base {
            Some(b) if b.is_finite() && b > item.depth_top => Some(convert(b) + offset),
            Some(b) if b.is_finite() => {
                skipped.push(format!(
                    "{}: base {b} is not below top {} - stored as a point sample",
                    item.name, item.depth_top
                ));
                None
            }
            _ => None,
        };
        bytes += prep.data.len() as i64;
        any_unprintable |= !prep.printable;
        rows.push(crate::db::NewImage {
            depth_top: top,
            depth_base: base,
            name: item.name.clone(),
            caption: item.caption.clone().filter(|c| !c.trim().is_empty()),
            mime: prep.mime.clone(),
            width: prep.width as i32,
            height: prep.height as i32,
            src_width: Some(prep.src_width as i32),
            src_height: Some(prep.src_height as i32),
            source_path: Some(item.path.clone()),
            printable: prep.printable,
            data: prep.data.clone(),
        });
    }
    if rows.is_empty() {
        return Err(format!("no image could be imported ({})", skipped.join("; ")));
    }

    let source = req.items.first().map(|i| i.path.clone());
    let imported = crate::db::insert_well_images(conn, &req.well_id, &dataset, &target_set, source.as_deref(), &rows)
        .map_err(|e| e.to_string())?;

    let mut notes: Vec<String> = Vec::new();
    if entered_unit != project_unit {
        notes.push(format!("depths converted {} -> {}", entered_unit.label(), project_unit.label()));
    }
    if any_unprintable {
        notes.push("some images print as a labelled frame until Pillow is installed".into());
    }
    // Asking to follow a core and getting raw depths must never be silent — the plates would be
    // out by exactly the correction the user believed had been applied.
    if req.follow_core {
        if core_pairs.is_empty() {
            notes.push("no core to follow, depths used as written".into());
        } else if core_pairs.iter().all(|(o, d)| (o - d).abs() <= 1e-4) {
            notes.push("core has not been shifted, so depths are unchanged".into());
        } else if outside_core > 0 {
            notes.push(format!(
                "placed from the core depth record; {outside_core} plate(s) fell outside the cored \
                 interval and were placed by holding the nearest correction"
            ));
        } else {
            notes.push("placed from the core depth record".into());
        }
    }
    Ok(ImageImportResult {
        dataset,
        set_name: target_set,
        imported,
        skipped,
        bytes,
        note: if notes.is_empty() { None } else { Some(notes.join("; ")) },
    })
}

fn db_resolve_set(conn: &Connection, well_id: &str, dataset: &str, desired: &str) -> Result<String, String> {
    crate::db::resolve_image_set_name(conn, well_id, dataset, desired).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1x1 JPEG (baseline, grey) — the smallest real file that exercises the marker walk.
    fn tiny_jpeg() -> Vec<u8> {
        // FFD8, APP0 (JFIF), SOF0 with height=3 width=5 components=3, EOI.
        let mut v = vec![0xFF, 0xD8];
        v.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]); // APP0, length 16
        v.extend_from_slice(b"JFIF\0");
        v.extend_from_slice(&[0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00]);
        v.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x03, 0x00, 0x05, 0x03]);
        v.extend_from_slice(&[0u8; 9]);
        v.extend_from_slice(&[0xFF, 0xD9]);
        v
    }

    fn tiny_png() -> Vec<u8> {
        let mut v = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
        v.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        v.extend_from_slice(b"IHDR");
        v.extend_from_slice(&7u32.to_be_bytes());
        v.extend_from_slice(&11u32.to_be_bytes());
        v.extend_from_slice(&[8, 2, 0, 0, 0]);
        v
    }

    #[test]
    fn jpeg_header_reports_size_and_component_count() {
        let m = sniff(&tiny_jpeg()).expect("jpeg recognised");
        assert_eq!(m.mime, "image/jpeg");
        assert_eq!((m.width, m.height), (5, 3));
        // The PDF exporter picks DeviceRGB vs DeviceGray from this; a wrong count prints
        // a plate in the wrong colour space rather than failing loudly.
        assert_eq!(m.components, 3);
    }

    #[test]
    fn png_header_reports_size() {
        let m = sniff(&tiny_png()).expect("png recognised");
        assert_eq!((m.mime, m.width, m.height), ("image/png", 7, 11));
    }

    #[test]
    fn a_non_image_is_rejected_rather_than_guessed() {
        assert!(sniff(b"DEPTH,CPOR,CPERM\n1523.5,0.21,120\n").is_none());
    }

    #[test]
    fn a_depth_is_read_from_the_filename_but_a_well_number_is_not() {
        assert_eq!(parse_depth_from_name("SANDI-01_1523.50"), (Some(1523.50), None));
        // The `01` must not become a 1 m depth — this is the whole reason a token needs a
        // decimal point or three integer digits to qualify.
        assert_eq!(parse_depth_from_name("SANDI-01"), (None, None));
        assert_eq!(parse_depth_from_name("TS_A"), (None, None));
    }

    #[test]
    fn an_adjacent_increasing_pair_is_read_as_a_photographed_interval() {
        assert_eq!(parse_depth_from_name("CORE_1523.50-1524.00"), (Some(1523.50), Some(1524.00)));
        assert_eq!(parse_depth_from_name("CORE_1523_1524"), (Some(1523.0), Some(1524.0)));
        // A year in the name is not an interval start: the pair must increase.
        assert_eq!(parse_depth_from_name("SANDI-2021_1523.5"), (Some(1523.5), None));
        // Separated by more than one character, so not a pair.
        assert_eq!(parse_depth_from_name("1523.5_TS_1524.0"), (Some(1524.0), None));
    }

    #[test]
    fn a_probe_of_a_missing_file_reports_the_reason_and_keeps_the_row() {
        let p = probe_image_files(&["Z:/no/such/plate.jpg".to_string()]);
        assert_eq!(p.len(), 1);
        assert!(p[0].error.is_some(), "a missing file must be reported, not dropped");
        assert_eq!(p[0].name, "plate");
    }

    /// A genuinely decodable 2x2 greyscale JPEG (Pillow, quality 25, optimized — 159 bytes).
    ///
    /// `tiny_jpeg()` above is a header-only stub: enough for `sniff` to read a size from, and
    /// deliberately so, but Pillow refuses it. The import path really does decode pixels, so a
    /// test that goes through it needs a real file — and this one works on BOTH paths, since a
    /// JPEG is also what the no-Pillow fallback stores verbatim.
    const REAL_JPEG_HEX: &str = concat!(
        "FFD8FFE000104A46494600010100000100010000FFDB0043002016181C1814201C1A1C24222026305034302C2C306246",
        "4A3A5074667A787266706E8090B89C8088AE8A6E70A0DAA2AEBEC4CED0CE7C9AE2F2E0C8F0B8CACEC6FFC0000B080002",
        "000201011100FFC40014000100000000000000000000000000000000FFC4001410010000000000000000000000000000",
        "0000FFDA0008010100003F003FFFD9",
    );

    fn real_jpeg() -> Vec<u8> {
        (0..REAL_JPEG_HEX.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&REAL_JPEG_HEX[i..i + 2], 16).unwrap())
            .collect()
    }

    /// A thin section is cut from a plug, so when that plug is re-registered the plate belongs
    /// with it. This is D2 answered as a deliberate choice rather than an automatic link.
    #[test]
    fn plates_can_follow_the_core_they_were_cut_from() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-PLATE-FOLLOW", None, None, None).unwrap();
        let w = wid.to_string();

        let d: Vec<f32> = (0..20).map(|i| 2000.0 + i as f32).collect();
        let v = vec![0.2f32; 20];
        let nan = vec![f32::NAN; 20];
        crate::db::insert_core_data(&conn, &w, "RAW", None, &d, &v, &nan, &nan, &nan).unwrap();
        crate::db::apply_core_run_shifts(
            &mut conn,
            &w,
            &[crate::db::RunShift { top: 2000.0, base: 2019.0, delta: 2.0 }],
            &[],
        )
        .unwrap();

        // Two real files: a point-sample section and an interval core photograph.
        let dir = std::env::temp_dir();
        let ts = dir.join("sandi_ts_follow.jpg");
        let cp = dir.join("sandi_cp_follow.jpg");
        std::fs::write(&ts, real_jpeg()).unwrap();
        std::fs::write(&cp, real_jpeg()).unwrap();

        let req = ImageImportRequest {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            set_name: "LAB".into(),
            depth_unit: None,
            max_px: None,
            quality: None,
            follow_core: true,
            items: vec![
                ImageImportItem {
                    path: ts.to_string_lossy().into_owned(),
                    name: "TS-1".into(),
                    depth_top: 2005.0,
                    depth_base: None,
                    caption: None,
                },
                ImageImportItem {
                    path: cp.to_string_lossy().into_owned(),
                    name: "CP-1".into(),
                    depth_top: 2010.0,
                    depth_base: Some(2011.0),
                    caption: None,
                },
            ],
        };
        let res = import_images(&conn, &req).expect("import");
        assert_eq!(res.imported, 2, "{:?}", res.skipped);
        assert_eq!(res.note.as_deref(), Some("placed from the core depth record"));

        let live = crate::db::list_well_images(&conn, &w, None).unwrap();
        let plate = |n: &str| live.iter().find(|i| i.name == n).unwrap();
        assert!((plate("TS-1").depth_top - 2007.0).abs() < 1e-3, "2005 + 2 m");
        assert!(plate("TS-1").depth_base.is_none(), "a section stays a point sample");
        assert!((plate("CP-1").depth_top - 2012.0).abs() < 1e-3, "2010 + 2 m");
        assert!(
            (plate("CP-1").depth_base.unwrap() - 2013.0).abs() < 1e-3,
            "the photograph keeps the 1 m it was logged with"
        );

        std::fs::remove_file(&ts).ok();
        std::fs::remove_file(&cp).ok();
    }

    /// Asking to follow a core that is not there must be said out loud — otherwise the plates are
    /// out by exactly the correction the user believed had been applied.
    #[test]
    fn following_a_core_that_is_not_there_says_so() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-PLATE-NOCORE", None, None, None).unwrap();

        let p = std::env::temp_dir().join("sandi_plate_nocore.jpg");
        std::fs::write(&p, real_jpeg()).unwrap();
        let req = ImageImportRequest {
            well_id: wid.to_string(),
            dataset: "THIN SECTION".into(),
            set_name: "LAB".into(),
            depth_unit: None,
            max_px: None,
            quality: None,
            follow_core: true,
            items: vec![ImageImportItem {
                path: p.to_string_lossy().into_owned(),
                name: "TS-1".into(),
                depth_top: 2005.0,
                depth_base: None,
                caption: None,
            }],
        };
        let res = import_images(&conn, &req).expect("import");
        assert_eq!(res.note.as_deref(), Some("no core to follow, depths used as written"));
        let live = crate::db::list_well_images(&conn, &wid.to_string(), None).unwrap();
        assert!((live[0].depth_top - 2005.0).abs() < 1e-3, "depth used as written");
        std::fs::remove_file(&p).ok();
    }
}
