//! Conditioning a core slab photograph — crop, deskew, white balance and tone.
//!
//! A core photograph arrives as somebody's snapshot: the box slightly rotated on the bench, the
//! tray and the tape measure in frame, and a colour cast from whatever light the core shed had that
//! afternoon. None of that is the rock, and all of it travels into a report.
//!
//! **The conditioning is non-destructive, and `db::well_images` is where that is enforced.** The
//! un-conditioned copy is kept the first time a recipe is baked, every later edit re-renders FROM
//! it, and clearing the recipe restores the import. Editing a recipe must never mean stacking a
//! second correction on the first — a brightness raised twice by eye is a photograph nobody can get
//! back to.
//!
//! **The result is BAKED into `data` rather than applied when the picture is drawn.** The PDF
//! exporter embeds those bytes untouched through a `/DCTDecode` XObject, so a render-time recipe
//! would print the unconditioned photograph while the screen showed the corrected one — silently,
//! and only on the deliverable. Baking also leaves the log view, the composite and the PDF nothing
//! to disagree about.
//!
//! **Everything geometric is stored as a FRACTION of the picture.** A crop rectangle in pixels
//! belongs to whichever copy it was dragged on, and the stored copy is already resampled to a
//! long-edge cap — the same argument that made `fov_um` a field of view rather than a µm/px ratio,
//! and the same one behind the scale-bar tool. It is also what makes the preview trustworthy: the
//! proxy the user tunes against and the full-size bake apply the identical recipe.
//!
//! Rule 7 throughout: numpy + Pillow in ONE subprocess, never embedded, and the runner reads
//! `sys.stdin.buffer`.

use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::python_engine::{find_python, hide_console};

/// Pictures per subprocess when baking. The `petrography::CHUNK` argument: a core-photograph
/// delivery is hundreds of plates at roughly a megabyte each, and one batch would be a gigabyte in
/// flight for no gain.
const CHUNK: usize = 8;

/// Long edge of the picture the dialog tunes against. Big enough to judge a crop edge and a colour
/// cast on, small enough that a slider feels like a slider.
const PREVIEW_PX: u32 = 1100;

/// JPEG quality of the baked copy. Higher than the import's 85 on purpose: this is the SECOND
/// encode of the same pixels, and re-encoding at the same quality compounds the loss.
const BAKE_QUALITY: u8 = 92;

/// A rectangle as FRACTIONS of the picture it was drawn on — see the module note on why it is not
/// pixels. Taken on the rotated picture, because that is what the user dragged across.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct CropBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// What was done to one photograph.
///
/// Every field defaults to "nothing", so an absent or empty recipe is exactly the imported picture
/// and a recipe written by an older build still loads.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct CoreRecipe {
    /// Deskew, in degrees CLOCKWISE. Applied before the crop, so the empty corners a rotation
    /// leaves behind are cropped away rather than printed.
    #[serde(default)]
    pub rotate_deg: f32,
    #[serde(default)]
    pub crop: Option<CropBox>,
    /// Per-channel gains from a neutral patch the user clicked — the colour card, the grey tray,
    /// the white putty. `None` reads the picture as delivered.
    ///
    /// Normalised so the LARGEST gain is 1, which is why it can only darken: pushing a channel
    /// past 1 clips exactly the brightest pixels and distorts their hue, which is the same rule the
    /// thin-section colour correction follows.
    #[serde(default)]
    pub gain: Option<[f32; 3]>,
    /// Manual trim on top of the picked patch, because a core shed rarely has a colour card and a
    /// grey tray is only approximately grey. Blue-to-amber; 0 is no change.
    #[serde(default)]
    pub warmth: f32,
    /// The other white-balance axis, green-to-magenta. 0 is no change.
    #[serde(default)]
    pub tint: f32,
    /// Stops. 0 is no change.
    #[serde(default)]
    pub exposure: f32,
    /// -1 flat, 0 no change, +1 hard.
    #[serde(default)]
    pub contrast: f32,
    /// -1 grey, 0 no change, +1 vivid.
    #[serde(default)]
    pub saturation: f32,
}

impl CoreRecipe {
    /// True when this recipe would change nothing — the test the write path uses to decide between
    /// baking and restoring the import.
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Just the colour half. What "apply to the whole delivery" copies, because a crop belongs to
    /// one photograph and a lamp belongs to the afternoon.
    pub fn colour_only(&self) -> Self {
        Self { rotate_deg: 0.0, crop: None, ..self.clone() }
    }

    /// This picture's own framing, under another picture's light.
    pub fn with_look(&self, look: &CoreRecipe) -> Self {
        Self { rotate_deg: self.rotate_deg, crop: self.crop, ..look.colour_only() }
    }
}

/// One picture conditioned, as the dialog needs to see it.
#[derive(Debug, Clone, Serialize)]
pub struct CorePreview {
    /// The conditioned proxy, base64 PNG.
    pub png: String,
    pub width: i32,
    pub height: i32,
    /// The SAME proxy with no recipe applied — the before half of a before/after. Sent with the
    /// after so the two are the same decode of the same picture at the same size; fetching them
    /// separately would let a stale one linger beside a fresh one.
    pub before_png: String,
    pub before_width: i32,
    pub before_height: i32,
    /// 64 bins per channel of the CONDITIONED proxy — what the user is actually judging. Clipping
    /// at either end is a shape a number cannot show, which is the whole reason it is here.
    pub hist_r: Vec<u32>,
    pub hist_g: Vec<u32>,
    pub hist_b: Vec<u32>,
    /// Set when the call carried a pick: the gains that would neutralise the patch clicked, and the
    /// colour that patch actually is, so the dialog can show a swatch rather than three numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picked_gain: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picked_rgb: Option<[u8; 3]>,
}

/// One photograph and the recipe to bake into it.
#[derive(Debug, Clone, Deserialize)]
pub struct BakeItem {
    pub image_id: String,
    pub recipe: CoreRecipe,
}

/// What a bake came to.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BakeResult {
    pub conditioned: usize,
    /// Pictures whose recipe was cleared, so the import was restored and the kept copy dropped.
    pub restored: usize,
    /// Named, never counted — a silent subset reads as a complete answer.
    pub skipped: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Deserialize)]
struct RunnerOut {
    #[serde(default)]
    results: Vec<RunnerRow>,
}

#[derive(Deserialize)]
struct RunnerRow {
    image_id: String,
    #[serde(default)]
    png: Option<String>,
    #[serde(default)]
    width: i32,
    #[serde(default)]
    height: i32,
    #[serde(default)]
    before_png: Option<String>,
    #[serde(default)]
    before_width: i32,
    #[serde(default)]
    before_height: i32,
    #[serde(default)]
    hist: Option<[Vec<u32>; 3]>,
    #[serde(default)]
    picked_gain: Option<[f32; 3]>,
    #[serde(default)]
    picked_rgb: Option<[u8; 3]>,
    /// Base64 JPEG of the baked full-size picture.
    #[serde(default)]
    jpeg: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// numpy + Pillow, probed in ONE subprocess so the dialog can say what is missing before a photo is
/// opened rather than after.
pub fn core_image_support() -> Result<bool, String> {
    let py = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;
    let mut cmd = Command::new(py);
    cmd.args(["-c", "import numpy, PIL.Image"]).stdout(Stdio::null()).stderr(Stdio::null());
    hide_console(&mut cmd);
    Ok(cmd.status().map(|s| s.success()).unwrap_or(false))
}

fn run_runner(python: &std::path::Path, header: &serde_json::Value, blobs: &[Vec<u8>]) -> Result<RunnerOut, String> {
    let mut cmd = Command::new(python);
    cmd.args(["-c", CORE_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
    {
        let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
        stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        for b in blobs {
            stdin.write_all(b).map_err(|e| e.to_string())?;
        }
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("conditioning failed");
        return Err(last.trim().to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("bad conditioning result: {e}"))
}

/// Renders ONE photograph at preview size under a recipe, with the un-conditioned proxy beside it.
///
/// `pick` is a point in fractions of the ROTATED, CROPPED picture — where the user clicked on what
/// they can see. The gains it returns are computed from the picture BEFORE any colour operation, so
/// clicking the same grey twice gives the same answer instead of compounding.
pub fn preview_core_image(
    conn: &Connection,
    image_id: &str,
    recipe: &CoreRecipe,
    pick: Option<(f32, f32)>,
) -> Result<CorePreview, String> {
    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;
    // The IMPORT, never the baked copy — see `db::get_well_image_source`.
    let (_, bytes) = crate::db::get_well_image_source(conn, image_id).map_err(|e| e.to_string())?;
    let header = serde_json::json!({
        "mode": "preview",
        "max_px": PREVIEW_PX,
        "ids": [image_id],
        "sizes": [bytes.len()],
        "recipes": [recipe],
        "pick": pick.map(|(x, y)| [x, y]),
    });
    let out = run_runner(&python, &header, std::slice::from_ref(&bytes))?;
    let row = out.results.into_iter().next().ok_or("the conditioner returned nothing")?;
    if let Some(e) = row.error {
        return Err(e);
    }
    let hist = row.hist.unwrap_or_default();
    let [hr, hg, hb] = hist;
    Ok(CorePreview {
        png: row.png.unwrap_or_default(),
        width: row.width,
        height: row.height,
        before_png: row.before_png.unwrap_or_default(),
        before_width: row.before_width,
        before_height: row.before_height,
        hist_r: hr,
        hist_g: hg,
        hist_b: hb,
        picked_gain: row.picked_gain,
        picked_rgb: row.picked_rgb,
    })
}

/// Bakes recipes into pictures, keeping each import.
///
/// A recipe that changes nothing RESTORES the picture rather than re-encoding it — a second JPEG
/// pass would degrade the pixels to record a decision to leave them alone.
pub fn bake_core_images(conn: &Connection, items: &[BakeItem]) -> Result<BakeResult, String> {
    let mut res = BakeResult::default();
    if items.is_empty() {
        return Ok(res);
    }
    // The clears need no pixels at all, so they never reach a subprocess — and must not, because a
    // second JPEG pass would degrade the picture to record a decision to leave it alone.
    let (identity, real): (Vec<&BakeItem>, Vec<&BakeItem>) =
        items.iter().partition(|i| i.recipe.is_identity());
    for it in identity {
        if crate::db::clear_image_conditioning(conn, &it.image_id).map_err(|e| e.to_string())? > 0 {
            res.restored += 1;
        }
    }
    if real.is_empty() {
        return Ok(res);
    }

    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;
    for batch in real.chunks(CHUNK) {
        let mut blobs = Vec::with_capacity(batch.len());
        for it in batch {
            let (_, bytes) =
                crate::db::get_well_image_source(conn, &it.image_id).map_err(|e| e.to_string())?;
            blobs.push(bytes);
        }
        let header = serde_json::json!({
            "mode": "bake",
            "quality": BAKE_QUALITY,
            "ids": batch.iter().map(|i| i.image_id.clone()).collect::<Vec<_>>(),
            "sizes": blobs.iter().map(|b| b.len()).collect::<Vec<_>>(),
            "recipes": batch.iter().map(|i| &i.recipe).collect::<Vec<_>>(),
        });
        let out = run_runner(&python, &header, &blobs)?;
        for row in out.results {
            let Some(it) = batch.iter().find(|i| i.image_id == row.image_id) else { continue };
            if let Some(e) = row.error {
                res.skipped.push(format!("{}: {}", it.image_id, e));
                continue;
            }
            let Some(b64) = row.jpeg else {
                res.skipped.push(format!("{}: the conditioner returned no picture", it.image_id));
                continue;
            };
            let bytes = decode_b64(&b64)?;
            let recipe = serde_json::to_string(&it.recipe).map_err(|e| e.to_string())?;
            crate::db::bake_image_conditioned(
                conn,
                &it.image_id,
                &recipe,
                &bytes,
                "image/jpeg",
                row.width,
                row.height,
            )
            .map_err(|e| e.to_string())?;
            res.conditioned += 1;
        }
    }
    if !res.skipped.is_empty() {
        res.notes.push(format!("{} picture(s) left as they were - see the list", res.skipped.len()));
    }
    Ok(res)
}

/// Copies one photograph's LOOK across a whole live delivery, leaving each picture's own framing
/// exactly where it was.
///
/// A core-shed run is shot under one light in one afternoon, so the colour half genuinely belongs
/// to the delivery — but the box sits differently on the bench in every frame, so the crop and the
/// deskew do not. Merging the two is done HERE rather than in the dialog, so what "the look" means
/// is one rule rather than one per caller. Same reasoning as `set_image_delivery_details` refusing
/// to give a core photograph the thin sections' magnification.
pub fn apply_look_to_delivery(
    conn: &Connection,
    well_id: &str,
    dataset: &str,
    look: &CoreRecipe,
) -> Result<BakeResult, String> {
    let existing = crate::db::list_image_recipes(conn, well_id, dataset).map_err(|e| e.to_string())?;
    let mut items = Vec::with_capacity(existing.len());
    for (image_id, json) in existing {
        // An unreadable recipe is treated as no framing rather than refused: the look still
        // applies, and the alternative is one corrupt row blocking a delivery-wide decision.
        let own: CoreRecipe = if json.trim().is_empty() {
            CoreRecipe::default()
        } else {
            serde_json::from_str(&json).unwrap_or_default()
        };
        items.push(BakeItem { image_id, recipe: own.with_look(look) });
    }
    bake_core_images(conn, &items)
}

/// Base64 in, bytes out. Written here rather than pulled in as a dependency because it is nine
/// lines and this is the only caller.
fn decode_b64(s: &str) -> Result<Vec<u8>, String> {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lut = [255u8; 256];
    for (i, c) in A.iter().enumerate() {
        lut[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0u32;
    for b in s.bytes() {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
        let v = lut[b as usize];
        if v == 255 {
            return Err("the conditioner returned a picture that is not base64".to_string());
        }
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Ok(out)
}

/// numpy + Pillow. Reads `sys.stdin.buffer`, never `sys.stdin` — a text stdin decodes with the
/// Windows ANSI codepage while `serde_json` emits UTF-8, and any non-ASCII picture name would
/// arrive as mojibake.
const CORE_RUNNER: &str = r#"
import sys, io, json, base64
import numpy as np
from PIL import Image

hdr = json.loads(sys.stdin.buffer.readline())
blobs = []
for n in hdr["sizes"]:
    blobs.append(sys.stdin.buffer.read(n))

def geometry(im, rc, max_px):
    # Proxy FIRST so a preview is cheap; the recipe is in fractions, so the proxy and the
    # full-size bake describe the same rectangle.
    if max_px:
        im = im.copy()
        im.thumbnail((max_px, max_px), Image.LANCZOS)
    deg = float(rc.get("rotate_deg") or 0.0)
    if abs(deg) > 1e-4:
        # Pillow rotates counter-clockwise; the slider is degrees clockwise, which is how a box
        # tilted to the right reads to the eye.
        im = im.rotate(-deg, resample=Image.BICUBIC, expand=False, fillcolor=(0, 0, 0))
    c = rc.get("crop")
    if c:
        w, h = im.size
        x0 = int(round(max(0.0, min(1.0, c["x"])) * w))
        y0 = int(round(max(0.0, min(1.0, c["y"])) * h))
        x1 = int(round(max(0.0, min(1.0, c["x"] + c["w"])) * w))
        y1 = int(round(max(0.0, min(1.0, c["y"] + c["h"])) * h))
        if x1 - x0 >= 2 and y1 - y0 >= 2:
            im = im.crop((x0, y0, x1, y1))
    return im

def colour(a, rc):
    g = rc.get("gain")
    if g:
        a = a * np.asarray(g, dtype=np.float32)
    w = float(rc.get("warmth") or 0.0)
    t = float(rc.get("tint") or 0.0)
    if abs(w) > 1e-6 or abs(t) > 1e-6:
        # A gentle span: these are a trim on top of a picked patch, not the whole correction.
        a = a * np.asarray([1.0 + 0.30 * w, 1.0 + 0.30 * t, 1.0 - 0.30 * w], dtype=np.float32)
    e = float(rc.get("exposure") or 0.0)
    if abs(e) > 1e-6:
        a = a * np.float32(2.0 ** e)
    c = float(rc.get("contrast") or 0.0)
    if abs(c) > 1e-6:
        a = (a - np.float32(0.5)) * np.float32(1.0 + c) + np.float32(0.5)
    s = float(rc.get("saturation") or 0.0)
    if abs(s) > 1e-6:
        # Rec. 709 luma, so desaturating a red core does not make it darker than a grey one.
        lum = (a * np.asarray([0.2126, 0.7152, 0.0722], dtype=np.float32)).sum(axis=2, keepdims=True)
        a = lum + (a - lum) * np.float32(1.0 + s)
    return np.clip(a, 0.0, 1.0)

def png_b64(a):
    im = Image.fromarray((a * 255.0 + 0.5).astype(np.uint8), "RGB")
    buf = io.BytesIO()
    im.save(buf, format="PNG", optimize=False)
    return base64.b64encode(buf.getvalue()).decode("ascii"), im.size

results = []
mode = hdr.get("mode", "preview")
pick = hdr.get("pick")
quality = int(hdr.get("quality") or 92)
max_px = int(hdr.get("max_px") or 0)

for i, ident in enumerate(hdr["ids"]):
    row = {"image_id": ident}
    try:
        rc = hdr["recipes"][i] or {}
        im = Image.open(io.BytesIO(blobs[i]))
        im.load()
        im = im.convert("RGB")
        im = geometry(im, rc, max_px)
        base = np.asarray(im, dtype=np.float32) / np.float32(255.0)

        if pick is not None:
            h, w = base.shape[0], base.shape[1]
            px = int(round(min(max(pick[0], 0.0), 1.0) * (w - 1)))
            py = int(round(min(max(pick[1], 0.0), 1.0) * (h - 1)))
            r = max(3, int(round(0.01 * max(w, h))))
            patch = base[max(0, py - r):py + r + 1, max(0, px - r):px + r + 1]
            # MEDIAN, not mean: a speck of dust or a highlight on the tray is one pixel away from
            # the grey that was actually clicked, and a mean would take it along.
            med = np.median(patch.reshape(-1, 3), axis=0)
            med = np.maximum(med, 1e-4)
            # Normalised so the LARGEST gain is 1 - it can only darken, and no channel is pushed
            # past 1 and clipped, which would distort the hue of exactly the brightest pixels.
            gains = float(med.min()) / med
            row["picked_gain"] = [float(x) for x in gains]
            row["picked_rgb"] = [int(round(float(x) * 255.0)) for x in med]

        out = colour(base, rc)

        if mode == "bake":
            im2 = Image.fromarray((out * 255.0 + 0.5).astype(np.uint8), "RGB")
            buf = io.BytesIO()
            im2.save(buf, format="JPEG", quality=quality, subsampling=0)
            row["jpeg"] = base64.b64encode(buf.getvalue()).decode("ascii")
            row["width"], row["height"] = im2.size
        else:
            b64, size = png_b64(out)
            row["png"] = b64
            row["width"], row["height"] = size
            b64b, sizeb = png_b64(base)
            row["before_png"] = b64b
            row["before_width"], row["before_height"] = sizeb
            hist = []
            for ch in range(3):
                counts, _ = np.histogram(out[:, :, ch], bins=64, range=(0.0, 1.0))
                hist.append([int(v) for v in counts])
            row["hist"] = hist
    except Exception as exc:
        row["error"] = str(exc)
    results.append(row)

sys.stdout.write(json.dumps({"results": results}))
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe that changes nothing must be recognisable without decoding a picture, because that
    /// is what decides between baking a second JPEG pass and restoring the import.
    #[test]
    fn a_recipe_that_does_nothing_is_not_a_recipe() {
        assert!(CoreRecipe::default().is_identity());
        let mut r = CoreRecipe::default();
        r.exposure = 0.2;
        assert!(!r.is_identity());
        r.exposure = 0.0;
        assert!(r.is_identity(), "back to nothing is nothing again");
        r.crop = Some(CropBox { x: 0.0, y: 0.0, w: 1.0, h: 1.0 });
        assert!(!r.is_identity(), "a crop of the whole picture is still a decision to store");
    }

    /// "Apply to the whole delivery" carries the lamp, never the framing.
    ///
    /// Every photograph in a core-shed run was shot under one light, so the colour half genuinely
    /// belongs to the delivery. The crop and the deskew belong to ONE box on ONE bench, and copying
    /// them across would cut every other box to the wrong rectangle — which is exactly the mistake
    /// `set_image_delivery_details` refuses to make with a magnification.
    #[test]
    fn applying_a_look_to_a_delivery_carries_the_colour_and_not_the_framing() {
        let r = CoreRecipe {
            rotate_deg: 1.4,
            crop: Some(CropBox { x: 0.05, y: 0.1, w: 0.9, h: 0.8 }),
            gain: Some([1.0, 0.94, 0.86]),
            warmth: -0.2,
            tint: 0.05,
            exposure: 0.3,
            contrast: 0.1,
            saturation: -0.05,
        };
        let c = r.colour_only();
        assert_eq!(c.rotate_deg, 0.0);
        assert!(c.crop.is_none());
        assert_eq!(c.gain, r.gain);
        assert_eq!(c.warmth, r.warmth);
        assert_eq!(c.exposure, r.exposure);
        assert_eq!(c.contrast, r.contrast);
        assert_eq!(c.saturation, r.saturation);
        assert_eq!(c.tint, r.tint);
    }

    /// A recipe written by an older build must still load, and an absent field must mean "nothing"
    /// rather than a default somebody chose.
    #[test]
    fn an_older_recipe_still_loads_and_every_missing_field_means_no_change() {
        let r: CoreRecipe = serde_json::from_str(r#"{"exposure":0.5}"#).unwrap();
        assert_eq!(r.exposure, 0.5);
        assert!(r.crop.is_none());
        assert!(r.gain.is_none());
        assert_eq!(r.rotate_deg, 0.0);
        assert_eq!(r.contrast, 0.0);
        assert_eq!(r.saturation, 0.0);
        let empty: CoreRecipe = serde_json::from_str("{}").unwrap();
        assert!(empty.is_identity());
    }

    fn bmp(w: usize, h: usize, px: impl Fn(usize, usize) -> (u8, u8, u8)) -> Vec<u8> {
        let row = w * 3;
        let pad = (4 - row % 4) % 4;
        let pixels = (row + pad) * h;
        let mut out = Vec::with_capacity(54 + pixels);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&((54 + pixels) as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        for _ in 0..6 {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        for y in 0..h {
            let yy = h - 1 - y; // BMP rows run bottom-up
            for x in 0..w {
                let (r, g, b) = px(x, yy);
                out.extend_from_slice(&[b, g, r]);
            }
            out.extend(std::iter::repeat_n(0u8, pad));
        }
        out
    }

    /// The whole road on real pixels: pick a grey, crop, bake, put it back.
    ///
    /// Three claims worth the subprocess. **A picked patch is neutralised by gains normalised so
    /// the largest is 1** — a precise number, and the rule that keeps the correction from clipping
    /// the brightest pixels. **A crop is a FRACTION, so the proxy the user dragged on and the
    /// full-size bake describe the same rectangle** — the property the whole preview rests on, and
    /// it is checked by shape rather than by trusting that they used the same code. And **the
    /// import comes back byte for byte**, through the real bake path rather than the storage call
    /// underneath it.
    #[test]
    #[ignore = "needs numpy and Pillow"]
    fn a_picked_grey_a_crop_and_a_way_back() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-CP-1", None, None, None).unwrap();
        let w = wid.to_string();

        // A 400x200 photograph: warm-lit grey rock on the left, blue tray on the right.
        const ROCK: (u8, u8, u8) = (200, 160, 120);
        let import = bmp(400, 200, |x, _| if x < 200 { ROCK } else { (40, 60, 150) });
        crate::db::insert_well_images(
            &conn,
            &w,
            "CORE PHOTO",
            "RUN1",
            None,
            &[crate::db::NewImage {
                depth_top: 1000.0,
                depth_base: Some(1003.0),
                name: "BOX-1".into(),
                mime: "image/bmp".into(),
                width: 400,
                height: 200,
                data: import.clone(),
                printable: true,
                ..Default::default()
            }],
        )
        .unwrap();
        let id = crate::db::list_well_images(&conn, &w, None).unwrap()[0].image_id.clone();

        // --- the grey pick -------------------------------------------------------------------
        let plain = CoreRecipe::default();
        let picked = preview_core_image(&conn, &id, &plain, Some((0.25, 0.5))).expect("preview");
        let gain = picked.picked_gain.expect("a click returns a gain");
        // min/c per channel: 120/200, 120/160, 120/120. Largest gain is exactly 1, so nothing is
        // pushed past white and clipped.
        for (got, want) in gain.iter().zip([0.6f32, 0.75, 1.0]) {
            assert!((got - want).abs() < 0.02, "gain {gain:?}");
        }
        assert_eq!(picked.picked_rgb, Some([200, 160, 120]), "the swatch is the colour clicked");
        assert!(!picked.png.is_empty() && !picked.before_png.is_empty(), "both halves of before/after");
        assert_eq!(picked.hist_r.len(), 64, "a histogram to judge the exposure on");

        // --- the crop, in fractions ----------------------------------------------------------
        let cropped = CoreRecipe {
            crop: Some(CropBox { x: 0.0, y: 0.0, w: 0.5, h: 1.0 }),
            gain: Some(gain),
            ..Default::default()
        };
        let prev = preview_core_image(&conn, &id, &cropped, None).expect("preview");
        let proxy_ratio = prev.width as f32 / prev.height as f32;
        assert!((proxy_ratio - 1.0).abs() < 0.02, "half of 400x200 is square: {prev:?}");

        let res = bake_core_images(&conn, &[BakeItem { image_id: id.clone(), recipe: cropped }])
            .expect("bake");
        assert_eq!((res.conditioned, res.restored), (1, 0), "{:?}", res.skipped);
        let info = crate::db::list_well_images(&conn, &w, None).unwrap()[0].clone();
        assert_eq!((info.width, info.height), (200, 200), "the same rectangle at full size");
        assert_eq!(info.mime, "image/jpeg");
        assert_ne!(crate::db::get_well_image(&conn, &id).unwrap().1, import, "the pixels changed");
        assert_eq!(
            crate::db::get_well_image_source(&conn, &id).unwrap().1,
            import,
            "and the photograph is still there underneath"
        );

        // --- and back ------------------------------------------------------------------------
        let res = bake_core_images(
            &conn,
            &[BakeItem { image_id: id.clone(), recipe: CoreRecipe::default() }],
        )
        .expect("clear");
        assert_eq!((res.conditioned, res.restored), (0, 1));
        assert_eq!(crate::db::get_well_image(&conn, &id).unwrap().1, import, "byte for byte");
        let info = crate::db::list_well_images(&conn, &w, None).unwrap()[0].clone();
        assert_eq!((info.width, info.height, info.mime.as_str()), (400, 200, "image/bmp"));
    }

    #[test]
    fn base64_round_trips_the_bytes_a_jpeg_is_made_of() {
        // Every residue class of 3, because that is where a base64 decoder goes wrong.
        for n in 0..8usize {
            let bytes: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
            let mut b64 = String::new();
            const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            for c in bytes.chunks(3) {
                let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
                let v = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
                for k in 0..4 {
                    if k <= c.len() {
                        b64.push(A[((v >> (18 - 6 * k)) & 63) as usize] as char);
                    } else {
                        b64.push('=');
                    }
                }
            }
            assert_eq!(decode_b64(&b64).unwrap(), bytes, "n = {n}");
        }
        assert!(decode_b64("!!!!").is_err(), "not base64 is refused rather than half-decoded");
    }
}
