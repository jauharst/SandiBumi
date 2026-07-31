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
use std::path::{Path, PathBuf};
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
    if bytes.len() >= 44 && bytes[0..4] == [1, 0, 0, 0] && &bytes[40..44] == b" EMF" {
        // Enhanced metafile — how a petrography laboratory delivers a vector-illustrated plate
        // book. The four-byte record type is far too weak a magic on its own, so the signature
        // at offset 40 is what identifies it.
        //
        // `rclBounds` is the bounding rectangle in DEVICE units and is inclusive, so a picture
        // 1103 pixels across reads 0..1102. Pillow does the decoding (Windows GDI); reading the
        // size here is what lets a decoration be told from a plate before anything is decoded.
        let l = le32(bytes, 8) as i32;
        let t = le32(bytes, 12) as i32;
        let r = le32(bytes, 16) as i32;
        let b = le32(bytes, 20) as i32;
        return Some(RasterMeta {
            mime: "image/emf",
            width: (r - l + 1).max(0) as u32,
            height: (b - t + 1).max(0) as u32,
            components: 0,
        });
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

// ---------------------------------------------------------------------------
// Plates delivered inside a workbook
// ---------------------------------------------------------------------------

/// One plate lifted out of a petrography workbook, as the wizard will show it.
#[derive(Debug, Clone, Serialize)]
pub struct WorkbookPlate {
    /// Where it was written on the way out — a temporary file the normal importer then reads, so
    /// there is ONE import path rather than two that can drift apart.
    pub path: String,
    pub name: String,
    pub sheet: String,
    /// A, B, C… in the order the pictures were anchored on the sheet.
    pub panel: String,
    pub width: u32,
    pub height: u32,
    /// From the sheet's own header CELL, never from a file name. `None` when the header states no
    /// depth — which is reported rather than filled in.
    pub depth_top: Option<f32>,
    pub depth_base: Option<f32>,
    /// 'ft' or 'm', as the sheet wrote it.
    pub unit: Option<String>,
    /// As stated on the sheet ('10x'). Deliberately NOT turned into a scale: see [`WorkbookProbe`].
    pub magnification: Option<String>,
    pub bytes: u64,
}

/// What a set of workbooks turned out to hold.
#[derive(Debug, Clone, Serialize)]
pub struct WorkbookProbe {
    pub plates: Vec<WorkbookPlate>,
    /// The one depth unit, when every sheet that stated one agreed. `None` when they disagreed or
    /// none said — and then the wizard must ask rather than assume, because a foot read as a metre
    /// puts a plate three times too deep.
    pub depth_unit: Option<String>,
    /// Everything left out and why, one line each — never a silent subset.
    pub notes: Vec<String>,
}

/// Pictures smaller than this on their long edge are DECORATIONS, not plates.
///
/// Round, and in PIXELS for the `min_pore_px` reason: it states what a picture has to be to be a
/// plate, where a byte count would say more about the JPEG quality than about the picture. A
/// workbook carries scale-bar graphics, logos and letterheads anchored beside the photomicrographs;
/// on a real delivery these ran 117x59 and 207x79 against plates of 1920x1080.
pub const MIN_PLATE_PX: u32 = 400;

/// Rows of the header block searched for a depth. A laboratory writes the well and depth at the
/// top of the sheet; searching further down invites a stray number.
pub const WORKBOOK_HEADER_ROWS: u32 = 14;

const WORKBOOK_RUNNER: &str = r#"
import sys, json, os, re, io, zipfile, posixpath
import xml.etree.ElementTree as ET

try:
    import openpyxl
    from PIL import Image
except Exception as e:
    sys.stderr.write("needs openpyxl and Pillow (pip install openpyxl pillow): %s\n" % e)
    sys.exit(1)

# stdin.buffer, never stdin: a piped child's TEXT stdin decodes with the Windows ANSI codepage
# while serde_json emits UTF-8, so a workbook path with any non-ASCII character arrives as
# mojibake and fails with a "no such file" naming a path nobody has.
req = json.loads(sys.stdin.buffer.readline().decode("utf-8"))
out_dir = req["out_dir"]
os.makedirs(out_dir, exist_ok=True)

# A DEPTH IS A NUMBER WITH A UNIT ON IT. Never a bare number: the same header block carries the
# plate number and the plug number, and on this delivery the depth cell reads "4633.50 FT/ 108" -
# taking the bare number would be a coin toss between a depth and a plug. Absent means absent.
DEPTH = re.compile(r"([0-9]+(?:\.[0-9]+)?)\s*(FEET|FEE?T|FT|METRES?|METERS?|M)\b", re.I)
# A range in one cell: "4626.00 - 4641.00 FT".
RANGE = re.compile(
    r"([0-9]+(?:\.[0-9]+)?)\s*[-–]\s*([0-9]+(?:\.[0-9]+)?)\s*(FEET|FEE?T|FT|METRES?|METERS?|M)\b",
    re.I,
)
MAG = re.compile(r"^\s*([0-9]+(?:\.[0-9]+)?)\s*[xX]\s*$")

FT = ("FT", "FEET", "FEE T")


def unit_of(tok):
    t = tok.upper().rstrip(".")
    return "ft" if t.startswith("F") else "m"


def scan(ws, max_rows):
    """Depth, unit and magnification from one plate sheet.

    The depth is read from the HEADER block only - that is where a lab writes it, and widening the
    search invites a stray number further down the sheet. The magnification is looked for over the
    WHOLE sheet, because it is captioned under each panel rather than in the header."""
    depth = base = unit = None
    for row in ws.iter_rows(min_row=1, max_row=max_rows):
        for c in row:
            if c.value is None:
                continue
            t = str(c.value)
            m = RANGE.search(t)
            if m:
                depth, base, unit = float(m.group(1)), float(m.group(2)), unit_of(m.group(3))
                break
            m = DEPTH.search(t)
            if m:
                depth, unit = float(m.group(1)), unit_of(m.group(2))
                break
        if depth is not None:
            break
    mags = set()
    for row in ws.iter_rows():
        for c in row:
            if isinstance(c.value, str):
                m = MAG.match(c.value)
                if m:
                    mags.add(m.group(1) + "x")
    # ONE stated magnification belongs to every plate on the sheet. Two or more and none is
    # attached: a sheet showing the same field at 5x and again at 10x cannot say which picture is
    # which without guessing from where the caption sits, and a magnification on the wrong plate is
    # worse than none.
    mag = next(iter(mags)) if len(mags) == 1 else None
    return depth, base, unit, mag, sorted(mags)


REL_NS = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"


def _resolve(base_dir, target):
    """A relationship Target, which may be package-absolute or relative to its own part."""
    if target.startswith("/"):
        return target.lstrip("/")
    return posixpath.normpath(posixpath.join(base_dir, target))


def _rels(zf, part):
    """{Id: (Type, resolved target)} for one part, or {} when it has no relationships."""
    name = posixpath.join(posixpath.dirname(part), "_rels", posixpath.basename(part) + ".rels")
    try:
        root = ET.fromstring(zf.read(name))
    except Exception:
        return {}
    base = posixpath.dirname(part)
    out = {}
    for r in root:
        rid, typ, tgt = r.get("Id"), r.get("Type", ""), r.get("Target", "")
        if rid and tgt and r.get("TargetMode") != "External":
            out[rid] = (typ, _resolve(base, tgt))
    return out


def sheet_pictures(zf):
    """{sheet name: [picture bytes, ...]} in anchor order, read from the PACKAGE.

    openpyxl is the wrong tool for this half. It DROPS the picture formats it cannot decode -
    WMF and EMF - with a warning that nothing downstream sees, so a delivery of vector plates
    arrives as a workbook that simply appears to hold no pictures. That is a silent subset, and a
    silent subset reads as a complete answer.

    The bytes and the sheet association are both in the package, and unlike the old .xls the
    association is EXPLICIT: workbook -> sheet part -> drawing part -> media part, each step a
    relationship file. So openpyxl is left to do what it is good at - reading the cells the depth
    is written in - and the pictures are read from the zip.
    """
    out = {}
    try:
        book = ET.fromstring(zf.read("xl/workbook.xml"))
    except Exception:
        return out
    book_rels = _rels(zf, "xl/workbook.xml")
    for sheets in book.iter():
        if not sheets.tag.endswith("}sheets"):
            continue
        for sh in sheets:
            name = sh.get("name")
            rid = sh.get(REL_NS + "id")
            if not name or rid not in book_rels:
                continue
            part = book_rels[rid][1]
            blobs = []
            for typ, tgt in _rels(zf, part).values():
                if not typ.endswith("/drawing"):
                    continue
                try:
                    dr = ET.fromstring(zf.read(tgt))
                except Exception:
                    continue
                dr_rels = _rels(zf, tgt)
                # Document order IS anchor order, which is the order the panels appear.
                for blip in dr.iter():
                    if not blip.tag.endswith("}blip"):
                        continue
                    embed = blip.get(REL_NS + "embed")
                    if embed not in dr_rels:
                        continue
                    try:
                        blobs.append(zf.read(dr_rels[embed][1]))
                    except Exception:
                        pass
            out[name] = blobs
    return out


rows = []
notes = []
for path in req["paths"]:
    stem = os.path.splitext(os.path.basename(path))[0]
    try:
        wb = openpyxl.load_workbook(path)
    except Exception as e:
        notes.append("%s: cannot be read (%s)" % (os.path.basename(path), e))
        continue
    try:
        pics = sheet_pictures(zipfile.ZipFile(path))
    except Exception as e:
        notes.append("%s: pictures cannot be read (%s)" % (os.path.basename(path), e))
        pics = {}
    units = set()
    mags = set()
    n_sheets = 0
    bare = 0
    for sname in wb.sheetnames:
        ws = wb[sname]
        imgs = pics.get(sname, [])
        if not imgs:
            bare += 1
            continue
        n_sheets += 1
        depth, base, unit, mag, sheet_mags = scan(ws, req.get("header_rows", 14))
        if unit:
            units.add(unit)
        mags.update(sheet_mags)
        if len(sheet_mags) > 1:
            notes.append("sheet %s: states %s - no magnification attached, it cannot be told which "
                         "picture is which" % (sname, " and ".join(sheet_mags)))
        # Panel order within a sheet is the order the pictures were anchored, which is the order
        # they are read here. A plate photographed in plane light and again under crossed nicols
        # is TWO pictures of ONE depth - they are kept as separate plates rather than merged,
        # because only the user can say which is which.
        kept = 0
        dropped = 0
        for blob in imgs:
            if not blob:
                continue
            # A workbook holds DECORATIONS as well as plates: scale-bar graphics, logos, north
            # arrows, the laboratory's letterhead. The floor is in PIXELS rather than bytes because
            # it states what a picture has to be to be a plate - a byte count says more about the
            # JPEG quality than about the picture. Every drop is COUNTED and reported.
            w = h = 0
            try:
                probe = Image.open(io.BytesIO(blob))
                w, h = probe.size          # header only; nothing is decoded here
            except Exception:
                pass
            if max(w, h) < req.get("min_px", 400):
                dropped += 1
                continue
            ext = "png" if blob[:4] == b"\x89PNG" else ("emf" if blob[:4] == b"\x01\x00\x00\x00" else "jpg")
            panel = chr(ord("A") + kept)
            safe = re.sub(r"[^A-Za-z0-9._-]+", "_", "%s_%s_%s" % (stem, sname, panel))
            fp = os.path.join(out_dir, safe + "." + ext)
            with open(fp, "wb") as fh:
                fh.write(blob)
            rows.append({
                "path": fp,
                "width": w,
                "height": h,
                "sheet": sname,
                "panel": panel,
                "name": "%s %s" % (sname, panel),
                "depth_top": depth,
                "depth_base": base,
                "unit": unit,
                "magnification": mag,
                "bytes": len(blob),
            })
            kept += 1
        if kept == 0:
            notes.append("sheet %s: %d picture(s), none big enough to be a plate" % (sname, dropped))
        elif dropped:
            notes.append("sheet %s: %d decoration(s) dropped" % (sname, dropped))
        if depth is None:
            notes.append("sheet %s: no depth in the header - a bare number is not read as one" % sname)
    if n_sheets == 0:
        notes.append("%s: no worksheet carries a picture" % os.path.basename(path))
    elif bare:
        # A cover sheet or a summary table legitimately holds no picture. Said once per file
        # rather than once per sheet: it is a tally, not a fault, but a delivery whose plates
        # failed to come through would show up here as a large number.
        notes.append("%s: %d worksheet(s) hold no picture" % (os.path.basename(path), bare))
    if len(units) > 1:
        notes.append("%s: sheets state more than one depth unit (%s)" % (
            os.path.basename(path), ", ".join(sorted(units))))
    if mags:
        notes.append("%s: magnification stated as %s - that is not a field of view, so nothing "
                     "dimensional can run until a scale is entered" % (
                         os.path.basename(path), ", ".join(sorted(mags))))

sys.stdout.write(json.dumps({"rows": rows, "notes": notes}))
"#;

/// Lifts every plate out of one or more petrography workbooks into `out_dir`.
///
/// **A petrography delivery does not arrive as a folder of pictures.** It arrives as a workbook
/// with one WORKSHEET per plate: the well, the depth, the plug number and the magnification typed
/// into cells, and the photomicrographs anchored on top. `probe_image_files` takes a list of files
/// and can read none of it, which is the actual first barrier between this suite and a client's
/// rock.
///
/// **This is an EXTRACTOR, not a second importer.** It turns a workbook into plate files plus a
/// depth table and hands them to `import_images`, so normalization, the Pillow cap, the delivery
/// set model, `follow_core`, `fov_um` and `prepared` all apply unchanged. Two importers would
/// eventually disagree about one of those; an extractor plus one importer cannot.
///
/// **The depth comes from the CELL, never from the file name.** `parse_depth_from_name` exists for
/// a folder of loose files and has to guess; here the laboratory wrote the depth down, and a guess
/// beside a stated fact is a bug waiting to happen. It is also read only where a UNIT follows it,
/// because the same header carries the plate number and the plug number — on a real delivery the
/// cell reads `4633.50 FT/ 108`, and taking the bare number would be a coin toss.
///
/// **A magnification is not a field of view and is never converted into one.** Turning `10x` into
/// micrometres needs the camera sensor width and the tube factor, both properties of the
/// laboratory's microscope rather than of the plate. It is carried through as text so the user can
/// see what the sheet claimed, and everything dimensional stays refused until a real scale is
/// entered.
pub fn probe_plate_workbooks(paths: &[String], out_dir: &Path) -> Result<WorkbookProbe, String> {
    // The old .xls is refused BY NAME with the fix, rather than half-read. Its pictures can be
    // recovered by scanning, but tying each one back to its worksheet — and therefore to its
    // depth — needs a full BIFF parser, and a guessed depth association is exactly what this
    // module refuses to produce.
    let mut notes: Vec<String> = Vec::new();
    let usable: Vec<String> = paths
        .iter()
        .filter(|p| {
            let ok = Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("xlsx") || e.eq_ignore_ascii_case("xlsm"));
            if !ok {
                notes.push(format!(
                    "{}: only the newer .xlsx workbook can be read. Open it in Excel and Save As \
                     .xlsx, then import that — the depths live in cells, and reading them out of \
                     the old format without the worksheet they belong to would mean guessing.",
                    Path::new(p).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| (*p).clone())
                ));
            }
            ok
        })
        .cloned()
        .collect();
    if usable.is_empty() {
        return Ok(WorkbookProbe { plates: Vec::new(), depth_unit: None, notes });
    }

    std::fs::create_dir_all(out_dir).map_err(|e| format!("cannot write to {}: {e}", out_dir.display()))?;
    let python = find_python().ok_or_else(|| {
        "no Python with openpyxl was found - set SANDIBUMI_PYTHON to an interpreter that has it"
            .to_string()
    })?;

    let header = serde_json::json!({
        "paths": usable,
        "out_dir": out_dir.to_string_lossy(),
        "header_rows": WORKBOOK_HEADER_ROWS,
        "min_px": MIN_PLATE_PX,
    });

    let mut cmd = Command::new(python);
    cmd.args(["-c", WORKBOOK_RUNNER])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        use std::io::Write as _;
        let mut si = child.stdin.take().ok_or("no stdin")?;
        si.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        si.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().last().unwrap_or("workbook read failed").trim().to_string());
    }

    #[derive(Deserialize)]
    struct Raw {
        rows: Vec<WorkbookRow>,
        notes: Vec<String>,
    }
    #[derive(Deserialize)]
    struct WorkbookRow {
        path: String,
        name: String,
        sheet: String,
        panel: String,
        #[serde(default)]
        width: u32,
        #[serde(default)]
        height: u32,
        #[serde(default)]
        depth_top: Option<f32>,
        #[serde(default)]
        depth_base: Option<f32>,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        magnification: Option<String>,
        #[serde(default)]
        bytes: u64,
    }

    let raw: Raw = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("bad workbook result: {e}"))?;
    notes.extend(raw.notes);

    // ONE unit for the delivery, and only when every sheet that stated one agreed. A mixed
    // workbook returns None so the wizard has to ask: a foot silently read as a metre puts a
    // plate more than three times too deep, and nothing on the log would look wrong.
    let units: std::collections::BTreeSet<String> =
        raw.rows.iter().filter_map(|r| r.unit.clone()).collect();
    let depth_unit = if units.len() == 1 { units.into_iter().next() } else { None };

    let plates: Vec<WorkbookPlate> = raw
        .rows
        .into_iter()
        .map(|r| WorkbookPlate {
            path: r.path,
            name: r.name,
            sheet: r.sheet,
            panel: r.panel,
            width: r.width,
            height: r.height,
            depth_top: r.depth_top,
            depth_base: r.depth_base,
            unit: r.unit,
            magnification: r.magnification,
            bytes: r.bytes,
        })
        .collect();

    let undated = plates.iter().filter(|p| p.depth_top.is_none()).count();
    if undated > 0 {
        notes.push(format!(
            "{undated} plate(s) have no depth - their sheet states none, and a bare number there is \
             the plate or plug number as often as a depth. Type them in before importing, or leave \
             them out."
        ));
    }
    Ok(WorkbookProbe { plates, depth_unit, notes })
}

/// A fresh temporary folder for one workbook extraction.
pub fn workbook_scratch_dir() -> PathBuf {
    std::env::temp_dir().join(format!("sandibumi_plates_{}", uuid::Uuid::new_v4()))
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
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImageImportItem {
    pub path: String,
    pub name: String,
    pub depth_top: f32,
    #[serde(default)]
    pub depth_base: Option<f32>,
    #[serde(default)]
    pub caption: Option<String>,
    /// Width of this plate in micrometres. Overrides the delivery-level value; absent falls back
    /// to it, and absent in both means no scale was declared. `#[serde(default)]`, so an older
    /// payload still deserializes.
    #[serde(default)]
    pub fov_um: Option<f32>,
}

/// The confirmed import, as the wizard sends it.
#[derive(Debug, Clone, Default, Deserialize)]
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
    /// Delivery-level defaults for the plate's scale and preparation. All three are absent by
    /// default and absent is a real answer — a plate with no declared scale is one nothing
    /// dimensional may run on, and a section whose preparation is unknown is one a blue-epoxy
    /// rule must refuse rather than guess at.
    #[serde(default)]
    pub fov_um: Option<f32>,
    /// '' = unknown, 'blue_epoxy', 'plain'.
    #[serde(default)]
    pub prepared: Option<String>,
    /// As the laboratory report names it. Free text on purpose: which stain was used is the lab's
    /// fact, and a vocabulary invented here would be a protocol nobody performed.
    #[serde(default)]
    pub stain: Option<String>,
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

    // Preparation is delivery-level: one impregnation run, one staining bath. An empty string is
    // stored as absent so "unknown" has exactly one representation, and everything downstream can
    // test it once.
    let clean = |s: &Option<String>| -> Option<String> {
        s.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_string)
    };
    let prep_kind = clean(&req.prepared);
    let stain_name = clean(&req.stain);

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
            // Per plate, because magnification genuinely varies within one delivery — that is the
            // whole content of "sometimes". The wizard's delivery-level value only fills the
            // blanks; what is stored belongs to the plate.
            fov_um: item.fov_um.or(req.fov_um).filter(|v| v.is_finite() && *v > 0.0),
            prepared: prep_kind.clone(),
            stain: stain_name.clone(),
        });
    }
    if rows.is_empty() {
        return Err(format!("no image could be imported ({})", skipped.join("; ")));
    }

    let source = req.items.first().map(|i| i.path.clone());
    let imported = crate::db::insert_well_images(conn, &req.well_id, &dataset, &target_set, source.as_deref(), &rows)
        .map_err(|e| e.to_string())?;
    // Record the depth basis, so a later core registration knows whether these plates move too.
    if req.follow_core {
        let _ = crate::db::mark_image_set_on_core(conn, &req.well_id, &dataset, &target_set);
    }

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
mod workbook_tests {
    use super::*;

    /// The old `.xls` is refused BY NAME with the fix, rather than half-read.
    ///
    /// Its pictures can be recovered by scanning the file for JPEG blobs — I checked, and a real
    /// 166 MB delivery yields 52 of them. What cannot be recovered without a full BIFF parser is
    /// which WORKSHEET each picture sat on, and the worksheet is where the depth is. A plate hung
    /// off the wrong sand is a wrong conclusion, so a guessed association is worse than no import.
    /// Runs without Python: the filter is applied before any subprocess is started.
    #[test]
    fn the_old_workbook_format_is_refused_by_name_with_the_fix() {
        let dir = std::env::temp_dir().join("sandibumi_wb_refuse_test");
        let probe = probe_plate_workbooks(&["C:/x/PETROGRAPHY PLATES.xls".to_string()], &dir)
            .expect("a refusal is a result, not an error");
        assert!(probe.plates.is_empty());
        assert_eq!(probe.notes.len(), 1);
        let note = &probe.notes[0];
        assert!(note.contains("PETROGRAPHY PLATES.xls"), "named: {note}");
        assert!(note.contains("Save As"), "the fix is stated: {note}");
        // Nothing was created, because nothing was read.
        assert!(!dir.exists(), "a refusal must not leave a scratch folder behind");
    }

    /// The newer formats are accepted. Kept beside the refusal so nobody "tidies" the filter into
    /// rejecting `.xlsm`, which is the same package with macros in it.
    #[test]
    fn the_newer_workbook_formats_are_accepted() {
        for ext in ["xlsx", "XLSX", "xlsm"] {
            let p = format!("C:/nope/does_not_exist.{ext}");
            let dir = std::env::temp_dir().join(format!("sandibumi_wb_accept_{ext}"));
            // It gets past the extension filter and fails later (no such file / no python), which
            // is the point: it was not turned away for its name.
            let r = probe_plate_workbooks(&[p], &dir);
            let refused_for_its_name = matches!(&r, Ok(pr) if pr.notes.iter().any(|n| n.contains("Save As")));
            assert!(!refused_for_its_name, "{ext} must not be refused as an old workbook");
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    /// **A depth is a number with a unit on it.** The header block of a plate sheet also carries
    /// the plate number and the plug number — on a real delivery the cell reads `4633.50 FT/ 108`
    /// — so reading a bare number would be a coin toss between a depth and a plug. Pinned against
    /// the runner source because that rule lives in the regex.
    #[test]
    fn the_workbook_reader_only_takes_a_depth_that_carries_a_unit() {
        let src = WORKBOOK_RUNNER;
        assert!(src.contains("FEET|FEE?T|FT|METRES?|METERS?|M"), "the unit is part of the match");
        // The magnification is text and must never become a scale: converting 10x to micrometres
        // needs the camera sensor width and the tube factor, neither of which a delivery states.
        assert!(!src.contains("fov_um"), "a magnification must never be turned into a field of view");
        // stdin.buffer, never stdin - the standing rule for every runner in this repo.
        assert!(src.contains("sys.stdin.buffer"), "a piped child's text stdin is cp1252 here");
    }

    /// The decoration floor is in PIXELS and round — the `min_pore_px` argument. A workbook carries
    /// scale-bar graphics and letterheads anchored beside the plates; on a real delivery those ran
    /// 117x59 and 207x79 against plates of 1920x1080.
    #[test]
    fn the_plate_size_floor_is_round_and_in_pixels() {
        assert_eq!(MIN_PLATE_PX % 100, 0, "a round number, not somebody's tuned threshold");
        assert!(MIN_PLATE_PX >= 200 && MIN_PLATE_PX <= 800);
        assert!(WORKBOOK_HEADER_ROWS >= 5 && WORKBOOK_HEADER_ROWS <= 40);
    }

    /// The pictures come from the PACKAGE, never from openpyxl's own list.
    ///
    /// openpyxl DROPS the formats it cannot decode — WMF and EMF — with a warning nothing
    /// downstream sees. A delivered book of vector plates then arrives as a workbook that
    /// appears to hold no pictures at all, which is a silent subset, and a silent subset reads as
    /// a complete answer. Reading the zip is what makes that impossible rather than merely
    /// reported.
    #[test]
    fn the_workbook_reader_takes_its_pictures_from_the_package_not_from_openpyxl() {
        let src = WORKBOOK_RUNNER;
        assert!(src.contains("def sheet_pictures("), "the package reader is gone");
        assert!(
            !src.contains("_images"),
            "openpyxl's own picture list is back — it silently drops WMF and EMF"
        );
        // The association is what makes this safe and the old .xls unsafe: every step of
        // workbook -> sheet -> drawing -> media is an explicit relationship file.
        assert!(src.contains("xl/workbook.xml"));
        assert!(src.contains("_rels"));
        // A worksheet holding nothing is counted rather than skipped in silence.
        assert!(src.contains("hold no picture"));
    }

    /// An EMF plate must be RECOGNISED, or the importer calls a delivered plate an unreadable file.
    ///
    /// The four-byte record type is far too weak a magic on its own — plenty of files begin with a
    /// little-endian 1 — so the ` EMF` signature at offset 40 is what identifies it. `rclBounds`
    /// is inclusive, so a picture 1103 device units across reads 0..1102.
    #[test]
    fn an_enhanced_metafile_plate_is_recognised_rather_than_called_unreadable() {
        let mut v = vec![0u8; 88];
        v[0] = 1; // iType = EMR_HEADER
        let put = |v: &mut Vec<u8>, at: usize, n: i32| {
            v[at..at + 4].copy_from_slice(&n.to_le_bytes());
        };
        put(&mut v, 8, 0); // rclBounds left
        put(&mut v, 12, 0); // top
        put(&mut v, 16, 1102); // right, inclusive
        put(&mut v, 20, 791); // bottom, inclusive
        v[40..44].copy_from_slice(b" EMF");

        let m = sniff(&v).expect("an EMF is a recognised delivery format");
        assert_eq!(m.mime, "image/emf");
        assert_eq!(m.width, 1103);
        assert_eq!(m.height, 792);
        // It is not something the WebView can draw, so without Pillow the importer must say so by
        // name rather than store a plate nothing can display.
        assert!(!browser_decodable(m.mime));

        // The control: the same leading bytes without the signature are NOT an EMF. Without this
        // the check would claim any file starting with a little-endian 1.
        let mut fake = v.clone();
        fake[40..44].copy_from_slice(b"junk");
        assert!(sniff(&fake).is_none(), "the record type alone is not enough to claim EMF");
    }
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

    /// Jauhar, 2026-07-31: the scale is "sometimes yes, sometimes not", and so is the epoxy. So a
    /// delivery holds plates of both kinds, and the absent case has to stay absent — a default
    /// micron-per-pixel would be a microscope setting invented here, and a defaulted "plain" would
    /// let a blue-epoxy rule run on a section nobody impregnated.
    #[test]
    fn a_plate_with_no_declared_scale_or_preparation_stores_neither() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        let p = std::env::temp_dir().join("sandi_ts_noscale.jpg");
        std::fs::write(&p, real_jpeg()).unwrap();

        let req = ImageImportRequest {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            set_name: "LAB".into(),
            items: vec![ImageImportItem {
                path: p.to_string_lossy().into_owned(),
                name: "TS-1".into(),
                depth_top: 2005.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        import_images(&conn, &req).expect("import");

        let live = crate::db::list_well_images(&conn, &w, None).unwrap();
        assert_eq!(live.len(), 1);
        assert!(live[0].fov_um.is_none(), "no scale was declared, so none is stored");
        assert_eq!(live[0].prepared, "", "unknown preparation, not an assumed 'plain'");
        assert_eq!(live[0].stain, "");
    }

    /// The delivery-level value is a convenience that fills the blanks; what is stored belongs to
    /// the PLATE, because magnification genuinely varies within one delivery — which is the whole
    /// content of "sometimes".
    #[test]
    fn a_delivery_scale_fills_the_blanks_and_one_plate_can_overrule_it() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-TS", None, None, None).unwrap();
        let w = wid.to_string();

        let dir = std::env::temp_dir();
        let a = dir.join("sandi_ts_scale_a.jpg");
        let b = dir.join("sandi_ts_scale_b.jpg");
        std::fs::write(&a, real_jpeg()).unwrap();
        std::fs::write(&b, real_jpeg()).unwrap();

        let req = ImageImportRequest {
            well_id: w.clone(),
            dataset: "THIN SECTION".into(),
            set_name: "LAB".into(),
            fov_um: Some(2500.0),
            prepared: Some("blue_epoxy".into()),
            stain: Some("  ".into()), // whitespace is not a stain protocol
            items: vec![
                ImageImportItem {
                    path: a.to_string_lossy().into_owned(),
                    name: "TS-1".into(),
                    depth_top: 2005.0,
                    ..Default::default()
                },
                ImageImportItem {
                    path: b.to_string_lossy().into_owned(),
                    name: "TS-2".into(),
                    depth_top: 2006.0,
                    fov_um: Some(800.0), // taken at a higher magnification
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        import_images(&conn, &req).expect("import");

        let live = crate::db::list_well_images(&conn, &w, None).unwrap();
        let plate = |n: &str| live.iter().find(|i| i.name == n).unwrap();
        assert_eq!(plate("TS-1").fov_um, Some(2500.0));
        assert_eq!(plate("TS-2").fov_um, Some(800.0), "the plate overrules the delivery");
        assert!(live.iter().all(|i| i.prepared == "blue_epoxy"));
        assert!(live.iter().all(|i| i.stain.is_empty()), "blank is not a stain protocol");

        // A wrong entry has to be clearable — a scale that cannot be cleared is worse than one
        // that was never typed, because everything downstream will believe it.
        let id = plate("TS-1").image_id.clone();
        crate::db::set_image_details(&conn, &id, None, None, None).unwrap();
        let live = crate::db::list_well_images(&conn, &w, None).unwrap();
        let ts1 = live.iter().find(|i| i.name == "TS-1").unwrap();
        assert!(ts1.fov_um.is_none() && ts1.prepared.is_empty());
        // And the rest of the delivery is untouched by a per-plate edit.
        assert_eq!(live.iter().find(|i| i.name == "TS-2").unwrap().fov_um, Some(800.0));

        // The bulk path writes the whole live delivery in one statement.
        crate::db::set_image_delivery_details(&conn, &w, "THIN SECTION", Some(1200.0), Some("plain"), None)
            .unwrap();
        let live = crate::db::list_well_images(&conn, &w, None).unwrap();
        assert!(live.iter().all(|i| i.fov_um == Some(1200.0) && i.prepared == "plain"));
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
            &[crate::db::RunShift { top: 2000.0, base: 2019.0, delta: 2.0, ..Default::default() }],
            &crate::db::ShiftTargets::default(),
            &Default::default(),
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
                    fov_um: None,
                },
                ImageImportItem {
                    path: cp.to_string_lossy().into_owned(),
                    name: "CP-1".into(),
                    depth_top: 2010.0,
                    depth_base: Some(2011.0),
                    caption: None,
                    fov_um: None,
                },
            ],
            ..Default::default()
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
                fov_um: None,
            }],
            ..Default::default()
        };
        let res = import_images(&conn, &req).expect("import");
        assert_eq!(res.note.as_deref(), Some("no core to follow, depths used as written"));
        let live = crate::db::list_well_images(&conn, &wid.to_string(), None).unwrap();
        assert!((live[0].depth_top - 2005.0).abs() < 1e-3, "depth used as written");
        std::fs::remove_file(&p).ok();
    }
}

#[cfg(test)]
mod workbook_field_tests {
    use super::*;

    /// The real thing: lift the plates out of a delivered petrography workbook.
    ///
    /// Runs only when `SANDIBUMI_FIELD_FIXTURES` names a folder with a `workbooks/` subfolder of
    /// real `.xlsx` deliveries, and SKIPS with a printed reason otherwise — a fresh clone has no
    /// field data and must still go green. Synthetic workbooks cannot reproduce what a real one
    /// does: the decorations anchored beside the plates, the sheets that state two magnifications,
    /// the sheet whose header omits the depth.
    #[test]
    #[ignore = "needs a real petrography workbook; set SANDIBUMI_FIELD_FIXTURES"]
    fn plates_come_out_of_a_real_petrography_workbook() {
        let Some(root) = crate::field_fixtures::root() else {
            eprintln!("SKIP: set SANDIBUMI_FIELD_FIXTURES to a folder with a workbooks/ subfolder");
            return;
        };
        let dir = root.join("workbooks");
        let Ok(rd) = std::fs::read_dir(&dir) else {
            eprintln!("SKIP: {} does not exist", dir.display());
            return;
        };
        let books: Vec<String> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("xlsx")))
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        if books.is_empty() {
            eprintln!("SKIP: no .xlsx in {}", dir.display());
            return;
        }

        let out = std::env::temp_dir().join("sandibumi_wb_field_test");
        let _ = std::fs::remove_dir_all(&out);
        let probe = probe_plate_workbooks(&books, &out).expect("workbook read");

        assert!(!probe.plates.is_empty(), "a petrography workbook holds plates");
        // Every plate is a real file on disk that the ORDINARY importer can then read - that is
        // the whole design: an extractor feeding one import path, not a second importer.
        for p in &probe.plates {
            assert!(Path::new(&p.path).is_file(), "{} was not written", p.path);
            assert!(p.width >= MIN_PLATE_PX || p.height >= MIN_PLATE_PX, "a decoration got through");
        }
        // The depth comes from the cell. A delivery whose sheets state depths should have them.
        let dated = probe.plates.iter().filter(|p| p.depth_top.is_some()).count();
        assert!(dated > 0, "no plate carried a depth from its sheet");
        eprintln!(
            "{} plate(s) from {} workbook(s); {dated} with a depth; unit {:?}; {} note(s)",
            probe.plates.len(),
            books.len(),
            probe.depth_unit,
            probe.notes.len()
        );
        let _ = std::fs::remove_dir_all(&out);
    }
}
