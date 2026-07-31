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

// ---------------------------------------------------------------------------
// Reading a proxy log off the photographs
// ---------------------------------------------------------------------------

/// Curve-name prefix for every measure read off a photograph.
///
/// **It is deliberately not `VSH`, and never will be.** A core photograph's darkness co-varies with
/// shale in most clastic sections, which is not the same statement as being a shale volume: the same
/// dark band is organic-rich mudstone in one core, oil stain in another and a wet patch in a third.
/// A curve called VSH is read by every module downstream as a shale volume, and an uncalibrated one
/// under that name is a wrong answer that computes and plots. So the measure ships under a name that
/// says what was measured, and turning it into a shale volume is a calibration the user makes
/// against their own GR — which is what `compare_curve` below is for. The same reason
/// `GRAIN_D50_APP` is not `GRAIN_D50`.
pub const LOG_PREFIX: &str = "CPHOTO";

/// The three measures, in the order they are reported.
const MEASURES: [&str; 3] = ["DARK", "RED", "TEX"];

/// One run of the proxy-log extraction over a well's live photograph delivery.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreLogSpec {
    pub well_id: String,
    pub dataset: String,
    /// Which way depth runs across the CONDITIONED picture: `"x"` (along the width) or `"y"`.
    #[serde(default = "default_axis")]
    pub axis: String,
    /// The picture is laid out deepest-first — a box photographed the other way up, or a right-to-
    /// left lay-out.
    #[serde(default)]
    pub reverse: bool,
    /// Rows of core in one photograph. The cross-axis is split into this many equal lanes and they
    /// are read in order, so a four-row core box becomes one continuous trace.
    ///
    /// Equal lanes are an APPROXIMATION — a real box has unequal rows and gaps between them — and
    /// the honest alternative is to crop to one row at a time in Condition Core Photos and run this
    /// per row. The default is 1, so nobody gets the approximation without asking.
    #[serde(default = "default_lanes")]
    pub lanes: u32,
    /// Depth step of the output curve, in the project's depth unit.
    #[serde(default = "default_step")]
    pub step: f32,
    /// Report how each measure tracks this curve over the same interval — usually GR. `None` skips
    /// the check, but it is the only thing that says whether the trace is about the rock.
    #[serde(default)]
    pub compare_curve: Option<String>,
    /// Write the curves. `false` measures without writing, so a lay-out can be tried before it
    /// leaves anything in the project.
    #[serde(default)]
    pub write: bool,
}

fn default_axis() -> String {
    "x".to_string()
}
fn default_lanes() -> u32 {
    1
}
/// A round number, not a calibration: it is finer than most wireline sampling and coarse enough
/// that a metre of core is a handful of samples rather than a thousand.
fn default_step() -> f32 {
    0.02
}

/// What one measure came to.
#[derive(Debug, Clone, Serialize)]
pub struct CoreLogCurve {
    pub name: String,
    pub n: usize,
    pub p10: f32,
    pub p50: f32,
    pub p90: f32,
    /// Straight-line agreement with `compare_curve`, NaN when none was named or too few pairs
    /// overlapped. **Signed on purpose**: darkness and GR should both rise into shale, so a strongly
    /// NEGATIVE value on DARK is a finding rather than a weak result — most often the depth axis is
    /// the other way round, and occasionally the dark bands are oil stain rather than clay.
    pub correlation: f32,
    pub pairs: usize,
    /// Evenly spread down the interval for the dialog to draw. Never the first N — that would be the
    /// top of the core rather than the core.
    pub preview: Vec<f32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CoreLogResult {
    pub photographs: usize,
    pub samples: usize,
    pub depth_min: f32,
    pub depth_max: f32,
    pub curves: Vec<CoreLogCurve>,
    pub preview_depth: Vec<f32>,
    /// Written, if the run was asked to write.
    pub written: Vec<String>,
    pub skipped: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Deserialize)]
struct ScanRow {
    image_id: String,
    #[serde(default)]
    dark: Vec<f32>,
    #[serde(default)]
    red: Vec<f32>,
    #[serde(default)]
    tex: Vec<f32>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct ScanOut {
    #[serde(default)]
    results: Vec<ScanRow>,
}

/// How many samples one photograph should give up, from its own depth span.
fn sample_count(span: f32, step: f32) -> usize {
    let step = if step.is_finite() && step > 0.0 { step } else { default_step() };
    ((span / step).round() as i64).clamp(2, 4000) as usize
}

/// Reads the three proxy measures off a well's live photograph delivery.
///
/// **The picture read is the CONDITIONED one**, not the import — which is the whole reason the
/// conditioning came first. A darkness compared across boxes shot under two different lamps is a
/// comparison of the lamps.
///
/// **A photograph with no `depth_base` is refused by name.** It is a point sample: it is anchored at
/// one depth and measures no interval, so there is no axis to read a log along. Stretching it over a
/// guessed thickness would invent every sample in it.
///
/// **The depth range is taken to span the picture end to end along `axis`.** Cropping in Condition
/// Core Photos is therefore also the statement of where the core is in the frame — crop the tray and
/// the tape away, or they are read as rock.
pub fn extract_core_log(conn: &Connection, spec: &CoreLogSpec) -> Result<CoreLogResult, String> {
    let python = find_python().ok_or("no Python interpreter found (see SANDIBUMI_PYTHON)")?;
    let all = crate::db::list_well_images(conn, &spec.well_id, Some(&spec.dataset))
        .map_err(|e| e.to_string())?;
    if all.is_empty() {
        return Err(format!("no pictures in {} for this well", spec.dataset));
    }

    let mut res = CoreLogResult::default();
    let mut wanted = Vec::new();
    for info in &all {
        match info.depth_base.filter(|b| b.is_finite() && *b > info.depth_top) {
            Some(_) => wanted.push(info.clone()),
            None => res.skipped.push(format!(
                "{}: no base depth, so it covers no interval - a photograph anchored at one depth \
                 has no axis to read a log along. Give it a base in Plate Details.",
                info.name
            )),
        }
    }
    if wanted.is_empty() {
        return Err(
            "no photograph in this delivery covers a depth interval - set a base depth in Plate \
             Details, or these are point samples rather than core runs"
                .to_string(),
        );
    }

    let lanes = spec.lanes.max(1);
    let mut depths: Vec<f32> = Vec::new();
    let mut cols: Vec<Vec<f32>> = vec![Vec::new(); MEASURES.len()];

    for batch in wanted.chunks(CHUNK) {
        let mut blobs = Vec::with_capacity(batch.len());
        let mut counts = Vec::with_capacity(batch.len());
        for info in batch {
            // The CONDITIONED copy: `get_well_image`, not `get_well_image_source`.
            let (_, bytes) = crate::db::get_well_image(conn, &info.image_id).map_err(|e| e.to_string())?;
            blobs.push(bytes);
            let span = info.depth_base.unwrap_or(info.depth_top) - info.depth_top;
            counts.push(sample_count(span, spec.step));
        }
        let header = serde_json::json!({
            "axis": if spec.axis.eq_ignore_ascii_case("y") { "y" } else { "x" },
            "reverse": spec.reverse,
            "lanes": lanes,
            "ids": batch.iter().map(|i| i.image_id.clone()).collect::<Vec<_>>(),
            "sizes": blobs.iter().map(|b| b.len()).collect::<Vec<_>>(),
            "counts": counts,
        });
        let out: ScanOut = {
            let mut cmd = Command::new(&python);
            cmd.args(["-c", SCAN_RUNNER]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
            hide_console(&mut cmd);
            let mut child = cmd.spawn().map_err(|e| format!("failed to start python: {e}"))?;
            {
                let stdin = child.stdin.as_mut().ok_or("failed to open python stdin")?;
                stdin.write_all(header.to_string().as_bytes()).map_err(|e| e.to_string())?;
                stdin.write_all(b"\n").map_err(|e| e.to_string())?;
                for b in &blobs {
                    stdin.write_all(b).map_err(|e| e.to_string())?;
                }
            }
            let output = child.wait_with_output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("scan failed");
                return Err(last.trim().to_string());
            }
            serde_json::from_slice(&output.stdout).map_err(|e| format!("bad scan result: {e}"))?
        };
        for row in out.results {
            let Some(info) = batch.iter().find(|i| i.image_id == row.image_id) else { continue };
            if let Some(e) = row.error {
                res.skipped.push(format!("{}: {}", info.name, e));
                continue;
            }
            let n = row.dark.len();
            if n == 0 {
                res.skipped.push(format!("{}: nothing to read", info.name));
                continue;
            }
            let top = info.depth_top;
            let base = info.depth_base.unwrap_or(top);
            for i in 0..n {
                // Each sample sits at the MIDDLE of the slab it averaged, so a curve read at 2 cm
                // is not shifted a centimetre shallow against the log it is compared with.
                depths.push(top + (i as f32 + 0.5) / n as f32 * (base - top));
            }
            cols[0].extend_from_slice(&row.dark);
            cols[1].extend_from_slice(&row.red);
            cols[2].extend_from_slice(&row.tex);
            res.photographs += 1;
        }
    }

    if depths.is_empty() {
        return Err("no photograph could be read".to_string());
    }
    // Photographs overlap and arrive in whatever order the delivery is stored in; a curve has to be
    // monotonic in depth or every reader downstream sees a sawtooth.
    let mut order: Vec<usize> = (0..depths.len()).collect();
    order.sort_by(|a, b| depths[*a].total_cmp(&depths[*b]));
    let sorted_depth: Vec<f32> = order.iter().map(|i| depths[*i]).collect();
    let sorted: Vec<Vec<f32>> = cols.iter().map(|c| order.iter().map(|i| c[*i]).collect()).collect();

    res.samples = sorted_depth.len();
    res.depth_min = sorted_depth[0];
    res.depth_max = sorted_depth[sorted_depth.len() - 1];

    // The yardstick. Interpolated ONTO the photograph's sampling rather than the other way round:
    // the log is the continuous thing, and resampling the photograph would invent nothing but would
    // discard most of what it measured.
    let mut against: Option<Vec<f32>> = None;
    if let Some(name) = spec.compare_curve.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        match crate::equations::fetch_curve_frame(conn, &spec.well_id, &[name.to_string()]) {
            Ok((cd, cols)) => match cols.get(&name.to_uppercase()) {
                Some(cv) if cd.len() == cv.len() && cd.len() >= 2 => {
                    against = Some(sorted_depth.iter().map(|d| crate::tops::interp(&cd, cv, *d)).collect());
                }
                _ => res.notes.push(format!("{name} is not on this well, so nothing was compared.")),
            },
            Err(e) => res.notes.push(format!("{name} could not be read ({e}), so nothing was compared.")),
        }
    }

    let idx = crate::distribution::even_indices(res.samples, 400);
    res.preview_depth = idx.iter().map(|i| sorted_depth[*i]).collect();

    for (k, m) in MEASURES.iter().enumerate() {
        let v = &sorted[k];
        let mut finite: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
        finite.sort_by(f32::total_cmp);
        let (mut correlation, mut pairs) = (f32::NAN, 0usize);
        if let Some(a) = &against {
            let (xs, ys): (Vec<f32>, Vec<f32>) = v
                .iter()
                .zip(a.iter())
                .filter(|(x, y)| x.is_finite() && y.is_finite())
                .map(|(x, y)| (*x, *y))
                .unzip();
            pairs = xs.len();
            correlation = crate::tops::pearson(&xs, &ys).0;
        }
        res.curves.push(CoreLogCurve {
            name: format!("{LOG_PREFIX}_{m}"),
            n: finite.len(),
            p10: crate::distribution::percentile(&finite, 10.0),
            p50: crate::distribution::percentile(&finite, 50.0),
            p90: crate::distribution::percentile(&finite, 90.0),
            correlation,
            pairs,
            preview: idx.iter().map(|i| v[*i]).collect(),
        });
    }

    // Said whether or not the run writes, because it is the answer to "is this trace about the
    // rock" and the user meets that question while choosing a lay-out.
    if let Some(dark) = res.curves.first() {
        if dark.pairs >= 8 && dark.correlation < -0.3 {
            res.notes.push(format!(
                "Darkness runs OPPOSITE to {} ({:+.2}). The usual cause is the depth axis pointing \
                 the other way - try Deepest first. If the lay-out is right, the dark bands are \
                 something other than clay: oil stain, a wet patch, or organic-rich mudstone with \
                 no gamma response.",
                spec.compare_curve.as_deref().unwrap_or("the log"),
                dark.correlation
            ));
        }
    }
    if lanes > 1 {
        res.notes.push(format!(
            "Split into {lanes} equal lanes. A real core box has unequal rows and gaps between \
             them, so this is an approximation - for a careful job, crop to one row at a time in \
             Condition Core Photos and run this per row."
        ));
    }
    res.notes.push(format!(
        "These are IMAGE measures, not petrophysical properties. {LOG_PREFIX}_DARK tracks shale in \
         most clastic sections but is not a shale volume, which is why it is not called VSH - \
         calibrate it against your own GR before quoting it as one."
    ));

    if spec.write {
        let refs: Vec<(&str, &[f32])> = res
            .curves
            .iter()
            .zip(sorted.iter())
            .map(|(c, v)| (c.name.as_str(), v.as_slice()))
            .collect();
        crate::equations::write_computed_curves_batch(conn, &spec.well_id, &sorted_depth, &refs)
            .map_err(|e| e.to_string())?;
        res.written = res.curves.iter().map(|c| c.name.clone()).collect();
    }
    Ok(res)
}

/// numpy + Pillow again, and `sys.stdin.buffer` again.
const SCAN_RUNNER: &str = r#"
import sys, io, json
import numpy as np
from PIL import Image

hdr = json.loads(sys.stdin.buffer.readline())
blobs = [sys.stdin.buffer.read(n) for n in hdr["sizes"]]
axis = hdr.get("axis", "x")
lanes = max(1, int(hdr.get("lanes") or 1))
reverse = bool(hdr.get("reverse"))

results = []
for i, ident in enumerate(hdr["ids"]):
    row = {"image_id": ident}
    try:
        want = int(hdr["counts"][i])
        im = Image.open(io.BytesIO(blobs[i]))
        im.load()
        a = np.asarray(im.convert("RGB"), dtype=np.float32) / np.float32(255.0)
        # Work in a frame where rows run DOWN the core and columns run across it, so one code path
        # serves both lay-outs.
        if axis == "x":
            a = np.transpose(a, (1, 0, 2))
        if reverse:
            a = a[::-1]
        # Lanes are cut on the ACROSS axis and stacked in order, which is what turns a four-row core
        # box into one continuous trace.
        h, w = a.shape[0], a.shape[1]
        edge = w // lanes
        parts = [a[:, k * edge:(k + 1) * edge] for k in range(lanes)] if lanes > 1 else [a]
        a = np.concatenate(parts, axis=0) if lanes > 1 else a

        n = a.shape[0]
        want = max(1, min(want, n))
        # Slab edges, so every pixel belongs to exactly one sample and none is counted twice.
        edges = np.linspace(0, n, want + 1).astype(np.int64)
        lum = (a * np.asarray([0.2126, 0.7152, 0.0722], dtype=np.float32)).sum(axis=2)
        # Normalised redness: (R-G)/(R+G). Illumination cancels in the ratio, so it survives an
        # uneven lamp far better than a raw channel would.
        denom = a[:, :, 0] + a[:, :, 1] + np.float32(1e-6)
        red = (a[:, :, 0] - a[:, :, 1]) / denom

        dark, redv, tex = [], [], []
        for k in range(want):
            lo, hi = edges[k], max(edges[k] + 1, edges[k + 1])
            sl = lum[lo:hi]
            dark.append(float(1.0 - sl.mean()))
            redv.append(float(red[lo:hi].mean()))
            # Spread ACROSS the core within the slab: laminated or conglomeratic rock scatters,
            # a clean massive sand does not.
            tex.append(float(sl.std()))
        row["dark"] = dark
        row["red"] = redv
        row["tex"] = tex
    except Exception as exc:
        row["error"] = str(exc)
    results.append(row)

sys.stdout.write(json.dumps({"results": results}))
"#;

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

    /// Sampling follows the photograph's OWN depth span, and is bounded at both ends.
    #[test]
    fn a_photograph_gives_up_samples_in_proportion_to_the_rock_it_covers() {
        assert_eq!(sample_count(3.0, 0.02), 150);
        assert_eq!(sample_count(0.6, 0.02), 30);
        // A hair-thin span still yields a trace rather than an empty one, and a whole cored well in
        // one frame does not try to produce a hundred thousand samples off a 2400-pixel picture.
        assert_eq!(sample_count(0.001, 0.02), 2);
        assert_eq!(sample_count(500.0, 0.02), 4000);
        // A nonsense step falls back rather than dividing by zero.
        assert_eq!(sample_count(3.0, 0.0), 150);
        assert_eq!(sample_count(3.0, f32::NAN), 150);
    }

    /// The trace runs the way the picture is laid out, and says so against a real log.
    ///
    /// Three claims. **Depth follows the axis the user declared**, so a strip that darkens downward
    /// gives a darkness that rises with depth. **`reverse` genuinely reverses it** — the control
    /// exists because a box photographed the other way up is common, and getting it wrong produces a
    /// trace that is upside down and entirely plausible. And **the agreement with a real curve is
    /// signed**, so the wrong lay-out shows up as a strong NEGATIVE rather than as a weak result
    /// nobody can act on.
    #[test]
    #[ignore = "needs numpy and Pillow"]
    fn the_trace_runs_the_way_the_picture_is_laid_out() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::create_schema(&conn).unwrap();
        let wid = uuid::Uuid::new_v4();
        crate::db::insert_well(&conn, wid, "SANDI-CP-2", None, None, None).unwrap();
        let w = wid.to_string();

        // A vertical strip that darkens steadily downward: clean sand at the top, mudstone below.
        let strip = bmp(40, 400, |_, y| {
            let v = (255.0 - y as f32 * 0.55) as u8;
            (v, v, v)
        });
        crate::db::insert_well_images(
            &conn,
            &w,
            "CORE PHOTO",
            "RUN1",
            None,
            &[crate::db::NewImage {
                depth_top: 1000.0,
                depth_base: Some(1010.0),
                name: "BOX-1".into(),
                mime: "image/bmp".into(),
                width: 40,
                height: 400,
                data: strip,
                printable: true,
                ..Default::default()
            }],
        )
        .unwrap();

        // A GR that rises into the same mudstone, so darkness and gamma should agree. Into
        // `standard_curves`, because that is the depth frame `fetch_curve_frame` resolves against —
        // the same route every module takes to find its inputs.
        for i in 0..=100 {
            let d = 1000.0 + i as f32 * 0.1;
            conn.execute(
                "INSERT INTO standard_curves (well_id, depth, gr, res_deep, nphi, rhob)
                 VALUES (?1, ?2, ?3, 1.0, 0.2, 2.4)",
                duckdb::params![w, d, 40.0 + (d - 1000.0) * 8.0],
            )
            .unwrap();
        }

        let spec = |reverse: bool| CoreLogSpec {
            well_id: w.clone(),
            dataset: "CORE PHOTO".into(),
            axis: "y".into(),
            reverse,
            lanes: 1,
            step: 0.05,
            compare_curve: Some("GR".into()),
            write: false,
        };

        let fwd = extract_core_log(&conn, &spec(false)).expect("read");
        assert_eq!(fwd.photographs, 1);
        assert_eq!(fwd.samples, 200, "10 m at 5 cm");
        assert!((fwd.depth_min - 1000.0).abs() < 0.1 && (fwd.depth_max - 1010.0).abs() < 0.1);
        let dark = fwd.curves.iter().find(|c| c.name.ends_with("_DARK")).expect("darkness");
        assert!(dark.correlation > 0.95, "darkness should track this GR: {}", dark.correlation);

        let back = extract_core_log(&conn, &spec(true)).expect("read");
        let dark_back = back.curves.iter().find(|c| c.name.ends_with("_DARK")).unwrap();
        assert!(
            dark_back.correlation < -0.95,
            "reading the box the other way up must invert it, not weaken it: {}",
            dark_back.correlation
        );
        assert!(
            back.notes.iter().any(|n| n.contains("OPPOSITE")),
            "and the run has to SAY the lay-out looks wrong: {:?}",
            back.notes
        );
        // Never VSH. The measure ships under a name that says what was measured.
        assert!(fwd.curves.iter().all(|c| !c.name.contains("VSH")));

        // A photograph anchored at one depth covers no interval, so there is no axis to read along.
        crate::db::insert_well_images(
            &conn,
            &w,
            "SPOT",
            "RUN1",
            None,
            &[crate::db::NewImage {
                depth_top: 1005.0,
                depth_base: None,
                name: "SPOT-1".into(),
                mime: "image/bmp".into(),
                width: 40,
                height: 400,
                data: bmp(40, 400, |_, _| (128, 128, 128)),
                printable: true,
                ..Default::default()
            }],
        )
        .unwrap();
        let mut s = spec(false);
        s.dataset = "SPOT".into();
        let err = extract_core_log(&conn, &s).unwrap_err();
        assert!(err.contains("covers a depth interval"), "{err}");

        // Writing is a separate decision, so a lay-out can be tried without leaving curves behind.
        assert!(fwd.written.is_empty());
        let mut s = spec(false);
        s.write = true;
        let saved = extract_core_log(&conn, &s).expect("write");
        assert_eq!(saved.written.len(), 3);
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM computed_curves WHERE well_id = ?1 AND curve_name = 'CPHOTO_DARK'",
                duckdb::params![w],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 200);
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
